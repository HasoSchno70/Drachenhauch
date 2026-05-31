"""Datei-Browser fuer das Projekt (TreeView) mit Modul-Uebersicht.

Aufbau (von oben nach unten):
  * Sektion "Module" -- alle eingebauten Module (ui, gui, json, physics, ...).
    Klick oeffnet die zugehoerige Doku (docs/module-<name>.md) gerendert im
    Markdown-Viewer; Module ohne Doku sind gedimmt. Tooltip zeigt den IMPORT.
  * Sektion "Beispiele & Projekt" -- die .gb-Dateien als Ordner-Baum.

Restyle: Glyph-Icons pro Eintrag, Ordner mit Datei-Zaehler, groessere Zeilen,
Sektions-Header zum Ein-/Ausklappen. Doppelklick / Klick auf eine Datei
emittiert `file_activated(path)` (auch fuer Modul-Doku-.md -- main_window
`_open_file` routet .md automatisch in den Markdown-Viewer).

`Aktualisieren` macht einen vollen Re-Scan. Das Filterfeld filtert live ueber
Datei-Pfade UND Modul-Namen -- Treffer werden aufgeklappt.
"""
from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QFont
from PySide6.QtWidgets import (
    QFrame, QLabel, QLineEdit, QPushButton, QTreeWidget, QTreeWidgetItem,
    QVBoxLayout, QWidget,
)

from .theme import COLORS, theme_signals

# Glyphen (Emoji) als leichte Icons -- kein Asset noetig.
_ICON_MODULE = "\U0001F9E9"          # puzzle piece
_ICON_FOLDER = "\U0001F4C1"          # file folder
_ICON_FILE = "\U0001F4C4"            # page
_ICON_SECTION_MOD = "\U0001F9E9"
_ICON_SECTION_FILES = "\U0001F5C2"   # card index dividers


def _list_builtin_modules() -> list[str]:
    """Namen aller eingebauten Module (best effort, nie werfen)."""
    try:
        from gamebasic.modules import discover_modules
        return discover_modules()
    except Exception:
        return []


class FileBrowser(QWidget):
    file_activated = Signal(Path)  # Pfad (.gb ODER docs/module-*.md)

    def __init__(self, project_root: Path, parent=None):
        super().__init__(parent)
        self.project_root = project_root
        self.docs_dir = project_root / "docs"
        # Mapping path -> Tree-Item, fuer mark_active-Lookup (nur .gb-Files).
        self._items_by_path: dict[Path, QTreeWidgetItem] = {}
        self._active_path: Path | None = None
        self._sec_modules: QTreeWidgetItem | None = None
        self._sec_files: QTreeWidgetItem | None = None

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        self.header = QFrame()
        header_layout = QVBoxLayout(self.header)
        header_layout.setContentsMargins(10, 8, 10, 6)
        header_layout.setSpacing(5)
        self.title = QLabel("Explorer")
        header_layout.addWidget(self.title)
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

    def _on_theme_changed(self, _name: str) -> None:
        self._apply_style()
        self.refresh()

    # --------------------------------------------------- Daten
    def refresh(self) -> None:
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
                p for p in self.project_root.rglob("*.gb")
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

        for f in files:
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

        # Default: beide Sektionen offen (Module sofort sichtbar), tiefere
        # Ordner zugeklappt.
        self.tree.collapseAll()
        sec_mod.setExpanded(True)
        sec_files.setExpanded(True)

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
        elif kind in ("dir", "section"):
            item.setExpanded(not item.isExpanded())
        # "module_nodoc": keine Aktion.
