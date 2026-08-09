"""Datei-Browser fuer das Projekt (TreeView) mit Modul-Uebersicht.

Aufbau (von oben nach unten):
  * Sektion "Module" -- alle eingebauten Module (ui, gui, json, physics, ...).
    Klick oeffnet die zugehoerige Doku (docs/module-<name>.md) gerendert im
    Markdown-Viewer; Module ohne Doku sind gedimmt. Tooltip zeigt den IMPORT.
  * Sektion "Beispiele & Projekt" -- die .dh-Dateien als Ordner-Baum.

Restyle: Glyph-Icons pro Eintrag, Ordner mit Datei-Zaehler, groessere Zeilen,
Sektions-Header zum Ein-/Ausklappen. Doppelklick / Klick auf eine Datei
emittiert `file_activated(path)` (auch fuer Modul-Doku-.md -- main_window
`_open_file` routet .md automatisch in den Markdown-Viewer).

`Aktualisieren` macht einen vollen Re-Scan. Das Filterfeld filtert live ueber
Datei-Pfade UND Modul-Namen -- Treffer werden aufgeklappt.
"""
from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import QFileSystemWatcher, QTimer, Qt, Signal
from PySide6.QtGui import QColor, QFont
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QLineEdit, QPushButton, QTreeWidget,
    QTreeWidgetItem, QToolButton, QVBoxLayout, QWidget,
)

from .icons import icons
from .theme import COLORS, theme_signals

# Glyphen (Emoji) als leichte Icons -- kein Asset noetig.
_ICON_MODULE = "\U0001F9E9"          # puzzle piece
_ICON_FOLDER = "\U0001F4C1"          # file folder
_ICON_FILE = "\U0001F4C4"            # page
_ICON_CATEGORY = "\U0001F516"        # bookmark -- virtuelle Beispiel-Kategorie
_ICON_SECTION_MOD = "\U0001F9E9"
_ICON_SECTION_FILES = "\U0001F5C2"   # card index dividers


# Virtuelle Kategorien fuer die flache `examples/`-Sammlung (122 Dateien).
# REIN fuer die Explorer-Anzeige -- es werden KEINE Dateien verschoben, daher
# bleiben Pfade in dhrun.py, tests/ und docs/ unberuehrt. Klassifizierung per
# Namens-Heuristik; die ERSTE passende Kategorie gewinnt (Reihenfolge =
# Prioritaet bei Mehrdeutigkeit, z.B. "77_tiled_platformer" faellt unter
# "Spiele" vor "Module"). Substrings werden gegen den Dateinamen geprueft.
_EXAMPLE_CATEGORIES: list[tuple[str, tuple[str, ...]]] = [
    ("Benchmarks", ("bench_",)),
    ("3D & Rendering", (
        "3d", "model", "heightmap", "billboard", "lighting", "_light",
        "fog", "shadow", "normalmap", "pbr", "ibl", "postfx", "shader",
        "instanc", "skeletal", "emissive", "vortex", "schwarm",
    )),
    ("Spiele", (
        "pong", "tetris", "platformer", "coinquest", "amiga",
        "cybermatic", "wobbler",
    )),
    ("Module", (
        "json", "_db", "tween", "imgfx", "physics", "_ui", "gui", "astar",
        "vec2", "input", "html", "net", "ecs", "curves", "preloader",
        "tiled", "audio", "serial", "wifi", "usb", "_bt", "scene", "save",
        "table", "window", "theme", "particle", "camera", "sprite",
        "form_runner", "anim_fsm", "timer", "chiptune", "modplayer",
        "sampler",
    )),
    ("Grafik & Demos", (
        "shapes", "sound", "tilemap", "parallax", "textscroll", "schneefall",
        "hires", "showcase", "demo", "orbital", "layers", "atlas",
        "render_target", "blend_gentex", "2d_extras", "ttf", "collision",
        "editor", "monitors", "screen_native", "overlay", "filedialog",
    )),
    ("Sprache & Grundlagen", (
        "hello", "variables", "loops", "fibonacci", "builtins", "functions",
        "classes", "inheritance", "arrays", "maps", "strings", "math",
        "struct", "files", "try", "enum", "named_args", "bitwise", "tuple",
        "with", "static", "funcref", "slicing", "method_syntax", "qol",
        "props_comp", "dictcomp", "fstring", "operator", "coroutines",
        "select", "language",
    )),
]
_EXAMPLE_CATEGORY_FALLBACK = "Weitere Beispiele"
# Stabile Anzeige-Reihenfolge der Kategorien (Fallback ganz unten).
_EXAMPLE_CATEGORY_ORDER = [name for name, _ in _EXAMPLE_CATEGORIES] + [
    _EXAMPLE_CATEGORY_FALLBACK
]


def _classify_example(file_name: str) -> str:
    """Ordnet einen examples/-Dateinamen einer virtuellen Kategorie zu."""
    low = file_name.lower()
    for cat, keys in _EXAMPLE_CATEGORIES:
        if any(k in low for k in keys):
            return cat
    return _EXAMPLE_CATEGORY_FALLBACK


def _list_builtin_modules() -> list[str]:
    """Namen aller eingebauten Module (best effort, nie werfen)."""
    try:
        from drachenhauch.modules import discover_modules
        return discover_modules()
    except Exception:
        return []


class FileBrowser(QWidget):
    file_activated = Signal(Path)  # Pfad (.dh ODER docs/module-*.md)

    def __init__(self, project_root: Path, parent=None,
                 initial_expanded: set | None = None):
        super().__init__(parent)
        self.project_root = project_root
        self.docs_dir = project_root / "docs"
        # Mapping path -> Tree-Item, fuer mark_active-Lookup (nur .dh-Files).
        self._items_by_path: dict[Path, QTreeWidgetItem] = {}
        self._active_path: Path | None = None
        self._sec_modules: QTreeWidgetItem | None = None
        self._sec_files: QTreeWidgetItem | None = None
        # Beim ALLERERSTEN Aufbau (Baum noch leer, siehe `refresh()`) gibt es
        # keinen In-Session-Zustand zum Erhalten -- hier kommt stattdessen der
        # ueber App-Neustarts persistierte Zustand rein (main_window.py laedt
        # ihn aus settings.json und dekodiert ihn ueber `decode_expanded`).
        # Wird nach dem ersten `refresh()` verbraucht (siehe dort).
        self._initial_expanded: set | None = initial_expanded or None

        # Review-Fund: der Baum synchronisierte sich bisher NUR ueber den
        # manuellen "Aktualisieren"-Button -- Dateien, die extern (anderes
        # Programm, git checkout, ein zweiter Editor-Prozess) angelegt/
        # entfernt/umbenannt wurden, blieben bis zum naechsten Klick
        # unsichtbar bzw. als stale Eintrag stehen. `QFileSystemWatcher`
        # beobachtet Projekt-Root + jedes Verzeichnis mit .dh-Dateien
        # (Pfad-Liste wird bei jedem refresh() neu gesetzt); ein kurzes
        # Debounce (300ms) buendelt mehrere schnelle Aenderungen (z.B. ein
        # `git checkout`) zu EINEM Re-Scan statt vieler.
        self._watcher = QFileSystemWatcher(self)
        self._watcher.directoryChanged.connect(self._on_fs_changed)
        self._watcher.fileChanged.connect(self._on_fs_changed)
        self._watch_debounce = QTimer(self)
        self._watch_debounce.setSingleShot(True)
        self._watch_debounce.setInterval(300)
        self._watch_debounce.timeout.connect(self.refresh)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self.header = QFrame()
        header_layout = QVBoxLayout(self.header)
        header_layout.setContentsMargins(10, 8, 10, 6)
        header_layout.setSpacing(5)
        # Titelzeile: "Explorer" links, rechts zwei Knoepfe zum Alles
        # aus-/einklappen des Baums.
        title_row = QHBoxLayout()
        title_row.setContentsMargins(0, 0, 0, 0)
        title_row.setSpacing(2)
        self.title = QLabel("Explorer")
        title_row.addWidget(self.title)
        title_row.addStretch(1)
        self.expand_btn = QToolButton()
        self.expand_btn.setIcon(icons.get("unfold"))
        self.expand_btn.setAutoRaise(True)
        self.expand_btn.setToolTip("Alles ausklappen")
        self.expand_btn.clicked.connect(self.tree_expand_all)
        title_row.addWidget(self.expand_btn)
        self.collapse_btn = QToolButton()
        self.collapse_btn.setIcon(icons.get("fold"))
        self.collapse_btn.setAutoRaise(True)
        self.collapse_btn.setToolTip("Alles einklappen")
        self.collapse_btn.clicked.connect(self.tree_collapse_all)
        title_row.addWidget(self.collapse_btn)
        header_layout.addLayout(title_row)
        self.filter_entry = QLineEdit()
        self.filter_entry.setPlaceholderText("filtern (Datei oder Modul) ...")
        self.filter_entry.textChanged.connect(self._apply_filter)
        header_layout.addWidget(self.filter_entry)
        layout.addWidget(self.header)

        self.tree = QTreeWidget()
        self.tree.setHeaderHidden(True)
        self.tree.setUniformRowHeights(True)
        self.tree.setRootIsDecorated(True)
        self.tree.setIndentation(14)
        self.tree.setExpandsOnDoubleClick(False)
        self.tree.itemActivated.connect(self._on_activated)
        self.tree.itemClicked.connect(self._on_activated)
        layout.addWidget(self.tree, 1)

        self.refresh_row = QFrame()
        rl = QVBoxLayout(self.refresh_row)
        rl.setContentsMargins(8, 4, 8, 8)
        self.refresh_btn = QPushButton("Aktualisieren")
        self.refresh_btn.clicked.connect(self.refresh)
        rl.addWidget(self.refresh_btn)
        layout.addWidget(self.refresh_row)

        self._apply_style()
        theme_signals.changed.connect(self._on_theme_changed)
        self.refresh()

    # --------------------------------------------------- Expand/Collapse
    def tree_expand_all(self) -> None:
        """Klappt den gesamten Explorer-Baum auf."""
        self.tree.expandAll()

    def tree_collapse_all(self) -> None:
        """Klappt den gesamten Explorer-Baum zu (nur die Sektionen bleiben)."""
        self.tree.collapseAll()

    # --------------------------------------------------- Expand-State erhalten
    @staticmethod
    def _expand_key(item: QTreeWidgetItem):
        """Stabiler Schluessel (kind, id) eines aufklappbaren Knotens. Ueberlebt den
        Rebuild in refresh() (Items werden dabei neu erzeugt) -- dir=Pfad,
        section/catgroup=Name. Leaf-/Modul-Items liefern None."""
        k = item.data(0, Qt.ItemDataRole.UserRole)
        return k if isinstance(k, tuple) else None

    def _collect_expanded(self) -> set:
        """Schluessel aller aktuell aufgeklappten Knoten."""
        keys: set = set()

        def walk(item: QTreeWidgetItem) -> None:
            for i in range(item.childCount()):
                ch = item.child(i)
                if ch.isExpanded():
                    k = self._expand_key(ch)
                    if k is not None:
                        keys.add(k)
                walk(ch)

        walk(self.tree.invisibleRootItem())
        return keys

    def _restore_expanded(self, keys: set) -> None:
        """Klappt genau die Knoten wieder auf, deren Schluessel in `keys` ist
        (alle anderen bleiben zugeklappt -- so bleibt der Nutzer-Zustand erhalten)."""
        def walk(item: QTreeWidgetItem) -> None:
            for i in range(item.childCount()):
                ch = item.child(i)
                k = self._expand_key(ch)
                if k is not None and k in keys:
                    ch.setExpanded(True)
                walk(ch)

        walk(self.tree.invisibleRootItem())

    # ------------------------------------------- Persistenz ueber Neustarts
    def encode_expanded(self) -> list:
        """JSON-taugliche Form des aktuellen Auf-/Zu-Zustands (fuer
        `settings.json`, key "file_browser_expanded") -- `Path`-Anteile der
        `_expand_key`-Tupel werden zu `str`, der Rest (Section-/Kategorie-
        Namen) ist schon JSON-tauglich."""
        out = []
        for kind, ident in self._collect_expanded():
            out.append([kind, str(ident) if isinstance(ident, Path) else ident])
        return out

    @staticmethod
    def decode_expanded(entries) -> set:
        """Kehrfunktion zu `encode_expanded` -- baut aus der gespeicherten
        Liste wieder `_expand_key`-kompatible Tupel (Pfad-Kinds als `Path`)."""
        keys: set = set()
        if not isinstance(entries, list):
            return keys
        for entry in entries:
            if not (isinstance(entry, list) and len(entry) == 2):
                continue
            kind, ident = entry
            if not isinstance(kind, str) or not isinstance(ident, str):
                continue
            if kind in ("dir", "moduledoc"):
                keys.add((kind, Path(ident)))
            else:
                keys.add((kind, ident))
        return keys

    # --------------------------------------------------- Styling
    def _apply_style(self) -> None:
        c = COLORS
        self.header.setStyleSheet(f"background-color: {c['bg_panel']}; border: 0;")
        self.title.setStyleSheet(
            f"color: {c['fg']}; font-weight: bold; font-size: 11pt;")
        self.tree.setStyleSheet(
            f"""
            QTreeWidget {{
                background-color: {c['bg_alt']};
                color: {c['fg']};
                border: 0;
                outline: 0;
            }}
            QTreeWidget::item {{
                padding: 4px 4px;
                border: 0;
            }}
            QTreeWidget::item:hover {{
                background-color: {c['bg_hover']};
            }}
            QTreeWidget::item:selected {{
                background-color: {c['sel']};
                color: {c['fg']};
            }}
            """
        )
        self.refresh_row.setStyleSheet(f"background-color: {c['bg_alt']};")
        # Aktualisieren-Button: expliziter Akzent-Verlauf (dunkel -> Akzent) wie die
        # Primaer-Buttons der App. Direkt am Button (nicht ueber die globale
        # accent-Property), damit es unabhaengig vom QSS-Timing/Parent-Stylesheet
        # sicher greift -- und bei Theme-Wechsel ueber _apply_style neu gesetzt wird.
        acc = QColor(c["accent"])
        top, bot = acc.darker(280).name(), acc.darker(150).name()
        top_h, bot_h = acc.darker(230).name(), acc.darker(120).name()
        self.refresh_btn.setStyleSheet(
            f"""
            QPushButton {{
                background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                    stop:0 {top}, stop:1 {bot});
                /* Bewusst hartcodiert statt aus COLORS['fg'] -- der Verlauf
                   oben (acc.darker(150..280)) ist IMMER dunkel, unabhaengig
                   vom aktiven Theme; COLORS['fg'] ist im Light-Theme dunkel
                   und waere hier falsch (schlechter Kontrast). */
                color: #EAFBFB;
                border: 0;
                border-radius: 6px;
                padding: 7px 14px;
                font-weight: 600;
            }}
            QPushButton:hover {{
                background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                    stop:0 {top_h}, stop:1 {bot_h});
            }}
            QPushButton:pressed {{
                background: {c['accent_soft']};
            }}
            """
        )

    def _on_theme_changed(self, _name: str) -> None:
        self._apply_style()
        # Icons sind theme-abhaengig -> neu setzen (Cache wurde geleert).
        self.expand_btn.setIcon(icons.get("unfold"))
        self.collapse_btn.setIcon(icons.get("fold"))
        self.refresh()

    # --------------------------------------------------- Daten
    def refresh(self) -> None:
        # Auf-/Zu-Zustand der Ordner/Sektionen ueber den Rebuild hinweg erhalten:
        # was offen war, bleibt offen; was zu war, bleibt zu. Nur beim allerersten
        # Aufbau (Baum noch leer) gibt es keinen Zustand -> Default unten.
        had_state = self.tree.topLevelItemCount() > 0
        prev_expanded = self._collect_expanded() if had_state else set()

        self.tree.clear()
        self._items_by_path.clear()
        root = self.tree.invisibleRootItem()

        muted = QColor(COLORS["fg_muted"])
        accent = QColor(COLORS["accent"])
        fg = QColor(COLORS["fg"])

        bold = QFont()
        bold.setBold(True)

        # --- Sektion: Module ---------------------------------------
        modules = _list_builtin_modules()
        sec_mod = QTreeWidgetItem([f"{_ICON_SECTION_MOD}  Module ({len(modules)})"])
        sec_mod.setData(0, Qt.ItemDataRole.UserRole, ("section", "modules"))
        sec_mod.setForeground(0, accent)
        sec_mod.setFont(0, bold)
        root.addChild(sec_mod)
        self._sec_modules = sec_mod

        for name in modules:
            doc = self.docs_dir / f"module-{name}.md"
            has_doc = doc.exists()
            it = QTreeWidgetItem([f"{_ICON_MODULE}  {name}"])
            if has_doc:
                it.setData(0, Qt.ItemDataRole.UserRole, ("moduledoc", doc))
                it.setForeground(0, fg)
                it.setToolTip(0, f'IMPORT "{name}"  -  Doku oeffnen')
            else:
                it.setData(0, Qt.ItemDataRole.UserRole, ("module_nodoc", name))
                it.setForeground(0, muted)
                it.setToolTip(0, f'IMPORT "{name}"  (keine Doku)')
            sec_mod.addChild(it)

        # --- Sektion: Beispiele & Projekt --------------------------
        sec_files = QTreeWidgetItem([f"{_ICON_SECTION_FILES}  Beispiele & Projekt"])
        sec_files.setData(0, Qt.ItemDataRole.UserRole, ("section", "files"))
        sec_files.setForeground(0, accent)
        sec_files.setFont(0, bold)
        root.addChild(sec_files)
        self._sec_files = sec_files

        files = sorted(
            (
                p for p in self.project_root.rglob("*.dh")
                if not p.name.startswith("_") and ".venv" not in p.parts
            ),
            key=lambda p: (str(p.parent).lower(), p.name.lower()),
        )

        # Datei-Zaehler je Verzeichnis (rekursiv), fuer die Ordner-Labels.
        dir_counts: dict[Path, int] = {}
        for f in files:
            d = f.parent
            while True:
                dir_counts[d] = dir_counts.get(d, 0) + 1
                if d == self.project_root:
                    break
                d = d.parent

        dir_items: dict[Path, QTreeWidgetItem] = {}

        def get_or_create_dir(d: Path) -> QTreeWidgetItem:
            if d == self.project_root:
                return sec_files
            if d in dir_items:
                return dir_items[d]
            parent_item = get_or_create_dir(d.parent)
            cnt = dir_counts.get(d, 0)
            it = QTreeWidgetItem([f"{_ICON_FOLDER}  {d.name}  ({cnt})"])
            it.setForeground(0, accent)
            it.setData(0, Qt.ItemDataRole.UserRole, ("dir", d))
            parent_item.addChild(it)
            dir_items[d] = it
            return it

        # Die flache examples/-Sammlung wird virtuell nach Kategorie gruppiert
        # (kein Datei-Move). Pro-Kategorie-Zaehler vorab fuer die Labels.
        examples_dir = self.project_root / "examples"
        cat_counts: dict[str, int] = {}
        for f in files:
            if f.parent == examples_dir:
                cat_counts[_classify_example(f.name)] = (
                    cat_counts.get(_classify_example(f.name), 0) + 1)
        cat_items: dict[str, QTreeWidgetItem] = {}

        def get_or_create_category(cat: str) -> QTreeWidgetItem:
            if cat in cat_items:
                return cat_items[cat]
            ex_item = get_or_create_dir(examples_dir)
            it = QTreeWidgetItem(
                [f"{_ICON_CATEGORY}  {cat}  ({cat_counts.get(cat, 0)})"])
            it.setForeground(0, accent)
            # Virtuell -- kein Pfad; im Activate/Filter wie ein Ordner behandelt.
            it.setData(0, Qt.ItemDataRole.UserRole, ("catgroup", cat))
            ex_item.addChild(it)
            cat_items[cat] = it
            return it

        for f in files:
            if f.parent == examples_dir:
                parent_item = get_or_create_category(_classify_example(f.name))
            else:
                parent_item = get_or_create_dir(f.parent)
            file_item = QTreeWidgetItem([f"{_ICON_FILE}  {f.name}"])
            file_item.setData(0, Qt.ItemDataRole.UserRole, ("file", f))
            try:
                rel = f.relative_to(self.project_root)
                file_item.setToolTip(0, str(rel).replace("\\", "/"))
            except ValueError:
                pass
            parent_item.addChild(file_item)
            self._items_by_path[f] = file_item

        # Kategorie-Knoten in stabile, sinnvolle Reihenfolge bringen (sie
        # entstehen sonst in Datei-Iterations-Reihenfolge).
        ex_item = dir_items.get(examples_dir)
        if ex_item is not None:
            ordered = sorted(
                (ex_item.takeChild(0) for _ in range(ex_item.childCount())),
                key=lambda c: _EXAMPLE_CATEGORY_ORDER.index(
                    c.data(0, Qt.ItemDataRole.UserRole)[1])
                if c.data(0, Qt.ItemDataRole.UserRole)[0] == "catgroup"
                else len(_EXAMPLE_CATEGORY_ORDER),
            )
            for c in ordered:
                ex_item.addChild(c)

        # Watcher-Pfade an den frischen Baum anpassen: Projekt-Root + jedes
        # Verzeichnis, das (rekursiv) mindestens eine .dh-Datei enthaelt.
        # Alte Pfade zuerst komplett entfernen (ein Verzeichnis kann seit dem
        # letzten refresh() keine .dh-Dateien mehr haben oder ganz weg sein).
        old_watched = self._watcher.directories()
        if old_watched:
            self._watcher.removePaths(old_watched)
        watch_dirs = [str(self.project_root)] + [str(d) for d in dir_counts]
        if watch_dirs:
            self._watcher.addPaths(watch_dirs)

        if had_state:
            # Vorherigen Auf-/Zu-Zustand wiederherstellen (neue Items sind per
            # Default zugeklappt -> nur die zuvor offenen wieder aufklappen).
            self._restore_expanded(prev_expanded)
        elif self._initial_expanded:
            # Erststart NACH einem App-Neustart: der zuletzt gespeicherte
            # Auf-/Zu-Zustand (aus settings.json) ersetzt den harten Default.
            self.tree.collapseAll()
            self._restore_expanded(self._initial_expanded)
            self._initial_expanded = None  # nur beim ersten Aufbau anwenden
        else:
            # Erststart ohne gespeicherten Zustand: beide Sektionen offen
            # (Module sofort sichtbar), tiefere Ordner zugeklappt.
            self.tree.collapseAll()
            sec_mod.setExpanded(True)
            sec_files.setExpanded(True)

    def _on_fs_changed(self, _path: str) -> None:
        """`QFileSystemWatcher`-Signal (Datei ODER Verzeichnis) -- startet
        das Debounce neu, statt sofort zu `refresh()`en (mehrere schnelle
        Aenderungen, z.B. ein `git checkout`, sollen EINEN Re-Scan ausloesen,
        nicht einen pro Datei)."""
        self._watch_debounce.start()

    # --------------------------------------------------- Filter
    def _apply_filter(self, _q: str = "") -> None:
        q = self.filter_entry.text().strip().lower()

        def visit(item: QTreeWidgetItem) -> bool:
            data = item.data(0, Qt.ItemDataRole.UserRole)
            kind = data[0] if isinstance(data, tuple) else None
            if not q:
                self_match = True
            elif kind == "section":
                self_match = False        # Sektion folgt ihren Kindern
            elif kind == "file":
                p = data[1]
                hay = (item.text(0) or "").lower()
                try:
                    hay = str(p.relative_to(self.project_root)).lower().replace("\\", "/")
                except Exception:
                    pass
                self_match = q in hay
            else:
                self_match = q in (item.text(0) or "").lower()
            child_match = False
            for i in range(item.childCount()):
                if visit(item.child(i)):
                    child_match = True
            visible = self_match or child_match
            item.setHidden(not visible)
            if q and child_match:
                item.setExpanded(True)
            return visible

        root = self.tree.invisibleRootItem()
        for i in range(root.childCount()):
            visit(root.child(i))
        if not q:
            self.tree.collapseAll()
            if self._sec_modules is not None:
                self._sec_modules.setExpanded(True)
            if self._sec_files is not None:
                self._sec_files.setExpanded(True)
            if self._active_path is not None:
                self._expand_to(self._active_path)

    # --------------------------------------------------- Markieren / Aktivieren
    def mark_active(self, path: Path | None) -> None:
        self._active_path = path
        self.tree.clearSelection()
        if path is not None and path in self._items_by_path:
            it = self._items_by_path[path]
            it.setSelected(True)
            self._expand_to(path)
            self.tree.scrollToItem(it)

    def _expand_to(self, path: Path) -> None:
        it = self._items_by_path.get(path)
        if it is None:
            return
        parent = it.parent()
        while parent is not None:
            parent.setExpanded(True)
            parent = parent.parent()

    def _on_activated(self, item: QTreeWidgetItem, _col: int = 0) -> None:
        data = item.data(0, Qt.ItemDataRole.UserRole)
        if not isinstance(data, tuple):
            return
        kind, payload = data
        if kind == "file":
            self.file_activated.emit(payload)
        elif kind == "moduledoc":
            # payload = docs/module-*.md -> main_window._open_file routet .md
            # automatisch in den Markdown-Viewer.
            self.file_activated.emit(payload)
        elif kind in ("dir", "section", "catgroup"):
            item.setExpanded(not item.isExpanded())
        # "module_nodoc": keine Aktion.
