"""GameBasic-Editor Hauptfenster (PySide6).

Features:
- ActivityBar links: Files / Outline / Built-ins (toggelbar),
- Tabs mit `CodeEditor` (Highlight, Auto-Indent, Snippets, Completer, Tooltips),
- OutputConsole unten mit `QProcess`-Run + klickbare Datei:Zeile-Links,
- Suchen/Ersetzen (Ctrl+F/Ctrl+H), Goto (Ctrl+G), Einstellungen (Ctrl+,),
- Theme-Toggle Dark/Light (Ctrl+T) + persistierte Settings,
- Run/Stop/Bench (F5/Shift+F5/Ctrl+F5).
"""
from __future__ import annotations

from pathlib import Path
from typing import Optional

from PySide6.QtCore import Qt, QTimer, Signal
from PySide6.QtGui import (
    QAction, QCloseEvent, QDragEnterEvent, QDropEvent, QIcon, QKeySequence,
    QPixmap,
)
from PySide6.QtWidgets import (
    QFileDialog, QLabel, QMainWindow, QMessageBox, QSplitter, QStackedWidget,
    QStatusBar, QToolBar, QWidget,
)

from .activity_bar import ActivityBar
from .autosave import (
    AUTOSAVE_INTERVAL_MS, RecoveryEntry, autosave_dir, clear_autosaves,
    read_manifest, slug_for, write_manifest,
)
from .builtins_panel import BuiltinsPanel
from .dialogs import (
    AboutDialog, FindReplaceDialog, GotoLineDialog, RecoveryDialog,
    SettingsDialog, ShortcutsDialog,
)
from .file_browser import FileBrowser
from .find_in_project import FindInProjectDialog
from .formatter import format_source
from .icons import icons
from .markdown_viewer import MarkdownViewer
from .outline_panel import OutlinePanel
from .output_console import OutputConsole
from .palette import command_palette, quick_open
from .settings import (
    RECENT_FILES_MAX, load_recent, load_settings, save_recent, save_settings,
)
from .tabs import TabbedEditorArea, TabState
from .theme import COLORS, global_qss, set_active_theme, theme_signals
from .welcome_panel import WelcomePanel


# Defaults fuer Window-Geometrie und Splitter (groesser als Phase 1).
DEFAULT_WINDOW_W = 1600
DEFAULT_WINDOW_H = 1000
DEFAULT_SIDEBAR_W = 280
DEFAULT_EDITOR_H = 700
DEFAULT_CONSOLE_H = 260


class _ClickableLabel(QLabel):
    """`QLabel`, der auf Klick ein Signal feuert -- fuer Statusbar-Indicators."""
    clicked = Signal()

    def mousePressEvent(self, ev):  # noqa: N802
        if ev.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit()
        super().mousePressEvent(ev)


class GameBasicEditor(QMainWindow):
    def __init__(self, project_root: Path, initial_file: Path | None = None):
        super().__init__()
        self.project_root = project_root.resolve()
        self.recent: list[str] = load_recent()
        self.settings = load_settings()
        # Stack der zuletzt geschlossenen Datei-Pfade -- Ctrl+Shift+T poppt.
        self._closed_stack: list[str] = []

        # Theme aus Settings anwenden, BEVOR die Widgets gebaut werden --
        # sonst lesen sie initial die Dark-Werte und korrigieren sich erst
        # nach dem ersten theme-Signal.
        initial_theme = self.settings.get("theme", "dark")
        if initial_theme not in ("dark", "light"):
            initial_theme = "dark"
        set_active_theme(initial_theme)

        self.setWindowTitle("GameBasic Editor")
        self._apply_window_icon()
        # Beim ersten Phase-2-Start: persistierte Phase-1-Geometrie
        # (typisch 1280x800) auf das neue, groessere Default bumpen. Danach
        # gilt der gespeicherte Wert wieder unveraendert -- der Nutzer kann
        # das Fenster jederzeit kleiner machen und die Groesse bleibt.
        if not self.settings.get("phase2_seen"):
            win_w = max(int(self.settings.get("window_w", DEFAULT_WINDOW_W)), DEFAULT_WINDOW_W)
            win_h = max(int(self.settings.get("window_h", DEFAULT_WINDOW_H)), DEFAULT_WINDOW_H)
            self.settings["phase2_seen"] = True
        else:
            win_w = int(self.settings.get("window_w", DEFAULT_WINDOW_W))
            win_h = int(self.settings.get("window_h", DEFAULT_WINDOW_H))
        self.resize(win_w, win_h)

        self._build_ui()
        self._build_actions()
        self._build_toolbar()
        self._build_menubar()
        self._wire_signals()
        self._setup_debugger()
        self._setup_todo_panel()
        self.setAcceptDrops(True)

        # Initial-Auto-Complete-Setting auf alle (zukuenftigen) Editoren.
        self._auto_complete_default = bool(self.settings.get("autocomplete_auto", True))

        # Crash-Recovery: angebotene Snapshots als dirty Tabs einlesen,
        # bevor der Workspace startet.
        recovered = self._maybe_recover_autosaves()

        # Initialdatei explizit -> die hat Vorrang (z.B. CLI-Argument).
        # Andernfalls Workspace wiederherstellen, sonst Welcome zeigen.
        if initial_file is not None and initial_file.exists():
            self._open_file(initial_file)
        elif not recovered:
            self._restore_workspace()
        # Wenn nichts offen ist, bleibt WelcomePanel aktiv (kein leerer Tab).
        self._update_welcome_visibility()

        # Auto-Save-Tick startet automatisch nach Init.
        self._autosave_timer = QTimer(self)
        self._autosave_timer.setInterval(AUTOSAVE_INTERVAL_MS)
        self._autosave_timer.timeout.connect(self._do_autosave)
        self._autosave_timer.start()

        # Theme-Listener: bei Wechsel App-QSS neu setzen (QSS ist statisch
        # gerendert und liest seine Farben zur Build-Zeit).
        theme_signals.changed.connect(self._on_theme_changed)
        self._apply_app_stylesheet()

        self._update_status()
        self._refresh_outline()

    # ----------------------------------------------------- UI-Aufbau
    def _apply_window_icon(self) -> None:
        """Setzt das GameBasic-Logo als Window-/Taskbar-Icon.

        Das Logo ist breit (linkes Symbol + Schriftzug). Wir croppen das
        linke Symbol heraus (smart_square_crop aus dem branding-Modul) und
        verwenden es in mehreren Groessen, damit Windows je nach Kontext
        das richtige picken kann.
        """
        try:
            from .. import branding
            if not branding.is_available():
                return
            from PIL import Image
            from PIL.ImageQt import ImageQt
            pil = Image.open(branding._logo_path()).convert("RGBA")
            pil = branding._smart_square_crop(pil)
            icon = QIcon()
            for size in (16, 24, 32, 48, 64, 128, 256):
                scaled = pil.resize((size, size), Image.LANCZOS)
                qimg = ImageQt(scaled).copy()
                icon.addPixmap(QPixmap.fromImage(qimg))
            self.setWindowIcon(icon)
            from PySide6.QtWidgets import QApplication
            QApplication.instance().setWindowIcon(icon)
        except Exception:
            # Branding ist optional -- ohne Logo startet der Editor weiter.
            pass

    def _add_menubar_logo(self, mb) -> None:
        """Setzt das volle Logo (mit Schriftzug) rechts in die Menubar.

        Volle Logo-Variante statt nur Symbol -- der Branding-Schriftzug
        ist im Editor sonst nicht zu sehen, hier hat er Platz. Hoehe
        wird auf 22px begrenzt (passt in die Menubar-Hoehe), Breite
        proportional.
        """
        try:
            from .. import branding
            if not branding.is_available():
                return
            from PIL import Image
            from PIL.ImageQt import ImageQt
            pil = Image.open(branding._logo_path()).convert("RGBA")
            target_h = 22
            ratio = target_h / pil.size[1]
            target_w = max(1, int(pil.size[0] * ratio))
            scaled = pil.resize((target_w, target_h), Image.LANCZOS)
            qimg = ImageQt(scaled).copy()
            self._menubar_logo_pixmap = QPixmap.fromImage(qimg)
            # Label als Member halten -- ohne harte Referenz wuerde Qt's
            # Reparent-Verhalten bei manchen QMenuBar-Versionen das Widget
            # verlieren (siehe QTBUG-* zu setCornerWidget).
            self._menubar_logo_label = QLabel(self)
            self._menubar_logo_label.setPixmap(self._menubar_logo_pixmap)
            self._menubar_logo_label.setContentsMargins(0, 0, 8, 0)
            self._menubar_logo_label.setToolTip("GameBasic")
            self._menubar_logo_label.setFixedHeight(target_h)
            self._menubar_logo_label.setFixedWidth(target_w + 8)
            mb.setCornerWidget(self._menubar_logo_label, Qt.Corner.TopRightCorner)
            self._menubar_logo_label.show()
        except Exception:
            # Logo ist optional -- Editor laeuft auch ohne.
            pass

    def _build_ui(self) -> None:
        # Outer-Splitter: ActivityBar (fix) | rest (resizable)
        # Realisiert ueber QWidget mit Horizontal-Splitter.
        self._central = QWidget(self)
        from PySide6.QtWidgets import QHBoxLayout
        outer = QHBoxLayout(self._central)
        outer.setContentsMargins(0, 0, 0, 0)
        outer.setSpacing(0)

        self.activity = ActivityBar()
        outer.addWidget(self.activity)

        self.splitter_main = QSplitter(Qt.Orientation.Horizontal)
        self.splitter_main.setChildrenCollapsible(False)
        outer.addWidget(self.splitter_main, 1)
        self.setCentralWidget(self._central)

        # Linke Seite: Stack aus FileBrowser/Outline/Builtins
        self.sidebar_stack = QStackedWidget()
        self.file_browser = FileBrowser(self.project_root)
        self.outline = OutlinePanel()
        self.builtins_panel = BuiltinsPanel()
        self.sidebar_stack.addWidget(self.file_browser)
        self.sidebar_stack.addWidget(self.outline)
        self.sidebar_stack.addWidget(self.builtins_panel)
        self.splitter_main.addWidget(self.sidebar_stack)

        # Rechte Seite: Stack [WelcomePanel | Splitter(Tabs+Konsole)] --
        # beim Start ohne Datei zeigt Welcome, ab erstem Tab dann den
        # normalen Editor-Bereich.
        self.right_stack = QStackedWidget()
        self.splitter_main.addWidget(self.right_stack)

        self.welcome = WelcomePanel(self.project_root, self.recent)
        self.welcome.new_file.connect(self._welcome_new)
        self.welcome.open_dialog.connect(self._open_dialog)
        self.welcome.examples.connect(self._welcome_show_examples)
        self.welcome.show_docs.connect(self._show_readme)
        self.welcome.open_path.connect(self._open_file)
        self.right_stack.addWidget(self.welcome)

        self.splitter_right = QSplitter(Qt.Orientation.Vertical)
        self.splitter_right.setChildrenCollapsible(False)
        self.right_stack.addWidget(self.splitter_right)

        self.tabs = TabbedEditorArea()
        self.splitter_right.addWidget(self.tabs)

        self.console = OutputConsole(self.project_root)
        self.splitter_right.addWidget(self.console)

        self.splitter_main.setStretchFactor(0, 0)
        self.splitter_main.setStretchFactor(1, 1)
        self.splitter_main.setSizes([
            int(self.settings.get("sidebar_width", DEFAULT_SIDEBAR_W)),
            DEFAULT_WINDOW_W - DEFAULT_SIDEBAR_W,
        ])
        self.splitter_right.setStretchFactor(0, 3)
        self.splitter_right.setStretchFactor(1, 1)
        self.splitter_right.setSizes([
            int(self.settings.get("editor_height", DEFAULT_EDITOR_H)),
            int(self.settings.get("console_height", DEFAULT_CONSOLE_H)),
        ])

        self.setStatusBar(QStatusBar())
        self.cursor_label = QLabel("")
        self.statusBar().addPermanentWidget(self.cursor_label)
        # Klickbarer Sprache-Marker. GB ist die einzige Sprache, aber das
        # Schema folgt VSCode-Konvention.
        self.lang_label = QLabel("GameBasic")
        self.lang_label.setStyleSheet("padding: 0 8px;")
        self.statusBar().addPermanentWidget(self.lang_label)
        # Encoding (immer UTF-8 -- der Editor schreibt nur UTF-8).
        self.encoding_label = QLabel("UTF-8")
        self.encoding_label.setStyleSheet("padding: 0 8px;")
        self.statusBar().addPermanentWidget(self.encoding_label)
        # Theme-Indikator -- Klick togglet zwischen Dark/Light.
        self.theme_label = _ClickableLabel()
        self.theme_label.clicked.connect(self._toggle_theme)
        self.theme_label.setToolTip("Klick um zwischen Dark/Light umzuschalten")
        self.theme_label.setStyleSheet("padding: 0 8px;")
        self._refresh_theme_label()
        self.statusBar().addPermanentWidget(self.theme_label)

        # Ausgangszustand der Sidebar wiederherstellen.
        sidebar_active = self.settings.get("sidebar_active", "files")
        sidebar_visible = bool(self.settings.get("sidebar_visible", True))
        if sidebar_active not in ("files", "outline", "builtins"):
            sidebar_active = "files"
        self.activity.set_active(sidebar_active)
        self._show_sidebar_view(sidebar_active)
        self.sidebar_stack.setVisible(sidebar_visible)
        if not sidebar_visible:
            for btn in self.activity._buttons.values():  # type: ignore[attr-defined]
                btn.setChecked(False)

    def _build_actions(self) -> None:
        self.act_new = QAction(icons.get("new"), "Neu", self)
        self.act_new.setShortcut(QKeySequence.StandardKey.New)
        self.act_new.triggered.connect(self._new_tab)

        self.act_open = QAction(icons.get("open"), "Oeffnen ...", self)
        self.act_open.setShortcut(QKeySequence.StandardKey.Open)
        self.act_open.triggered.connect(self._open_dialog)

        self.act_save = QAction(icons.get("save"), "Speichern", self)
        self.act_save.setShortcut(QKeySequence.StandardKey.Save)
        self.act_save.triggered.connect(self._save_active)

        self.act_save_as = QAction(icons.get("save_as"), "Speichern unter ...", self)
        self.act_save_as.setShortcut(QKeySequence.StandardKey.SaveAs)
        self.act_save_as.triggered.connect(self._save_active_as)

        self.act_close_tab = QAction("Tab schliessen", self)
        self.act_close_tab.setShortcut(QKeySequence("Ctrl+W"))
        self.act_close_tab.triggered.connect(self._close_active_tab)

        self.act_reopen_tab = QAction("Geschlossenen Tab wieder oeffnen", self)
        self.act_reopen_tab.setShortcut(QKeySequence("Ctrl+Shift+T"))
        self.act_reopen_tab.triggered.connect(self._reopen_closed_tab)

        self.act_quit = QAction("Beenden", self)
        self.act_quit.setShortcut(QKeySequence("Ctrl+Q"))
        self.act_quit.triggered.connect(self.close)

        self.act_sprite_editor = QAction(
            icons.get("sprite_editor"), "Sprite-Editor oeffnen ...", self,
        )
        self.act_sprite_editor.setShortcut(QKeySequence("Ctrl+Shift+S"))
        self.act_sprite_editor.triggered.connect(self._open_sprite_editor)

        # Edit
        self.act_find = QAction(icons.get("find"), "Suchen ...", self)
        self.act_find.setShortcut(QKeySequence.StandardKey.Find)
        self.act_find.triggered.connect(self._show_find)

        self.act_replace = QAction(icons.get("replace"), "Ersetzen ...", self)
        self.act_replace.setShortcut(QKeySequence.StandardKey.Replace)
        self.act_replace.triggered.connect(self._show_replace)

        self.act_find_in_project = QAction("In Projekt suchen ...", self)
        self.act_find_in_project.setShortcut(QKeySequence("Ctrl+Shift+F"))
        self.act_find_in_project.triggered.connect(self._show_find_in_project)

        self.act_goto = QAction("Gehe zu Zeile ...", self)
        self.act_goto.setShortcut(QKeySequence("Ctrl+G"))
        self.act_goto.triggered.connect(self._show_goto)

        self.act_settings = QAction(icons.get("settings"), "Einstellungen ...", self)
        self.act_settings.setShortcut(QKeySequence("Ctrl+,"))
        self.act_settings.triggered.connect(self._show_settings)

        self.act_quick_open = QAction("Datei suchen (Quick-Open) ...", self)
        self.act_quick_open.setShortcut(QKeySequence("Ctrl+P"))
        self.act_quick_open.triggered.connect(self._show_quick_open)

        self.act_command_palette = QAction("Befehl ausfuehren ...", self)
        self.act_command_palette.setShortcut(QKeySequence("Ctrl+Shift+P"))
        self.act_command_palette.triggered.connect(self._show_command_palette)

        self.act_format_doc = QAction("Indent normalisieren (Dokument formatieren)", self)
        self.act_format_doc.setShortcut(QKeySequence("Shift+Alt+F"))
        self.act_format_doc.triggered.connect(self._format_document)

        # Run
        self.act_run = QAction(icons.get("run"), "Run", self)
        self.act_run.setShortcut(QKeySequence("F5"))
        self.act_run.triggered.connect(self._run_active)

        self.act_stop = QAction(icons.get("stop"), "Stop", self)
        self.act_stop.setShortcut(QKeySequence("Shift+F5"))
        self.act_stop.triggered.connect(self.console.stop_run)
        self.act_stop.setEnabled(False)

        self.act_run_native = QAction(
            icons.get("run"), "Run nativ (gbrt)", self)
        self.act_run_native.setShortcut(QKeySequence("F6"))
        self.act_run_native.setToolTip(
            "Nativ ausfuehren via gbrt-Runtime (F6) -- kompiliert + startet die "
            "native Rust-Runtime")
        self.act_run_native.triggered.connect(self._run_native_active)

        self.act_export = QAction(
            icons.get("save"), "Export → standalone .exe", self)
        self.act_export.setShortcut(QKeySequence("Ctrl+F6"))
        self.act_export.setToolTip(
            "Standalone-.exe buendeln (Ctrl+F6) -- gbrt + Bytecode + assets/ in "
            "einen Ordner; laeuft ohne Python")
        self.act_export.triggered.connect(self._export_active)

        self.act_bench = QAction(icons.get("bench"), "Benchmark (TW vs Python-VM vs Native-VM)", self)
        self.act_bench.setShortcut(QKeySequence("Ctrl+F5"))
        self.act_bench.triggered.connect(self._bench_active)

        # Debugger (Tree-Walker). Start via F7; Steuerung waehrend der Sitzung.
        self.act_debug = QAction(icons.get("run"), "Debuggen (Tree-Walker)", self)
        self.act_debug.setShortcut(QKeySequence("F7"))
        self.act_debug.setToolTip(
            "Debuggen mit Breakpoints + Step (F7) -- Tree-Walker. Klick links "
            "im Gutter setzt Breakpoints.")
        self.act_debug.triggered.connect(self._debug_start)

        self.act_dbg_continue = QAction("Fortsetzen", self)
        self.act_dbg_continue.setShortcut(QKeySequence("F8"))
        self.act_dbg_continue.triggered.connect(self._debug_continue)
        self.act_dbg_step_over = QAction("Step Over", self)
        self.act_dbg_step_over.setShortcut(QKeySequence("F10"))
        self.act_dbg_step_over.triggered.connect(self._debug_step_over)
        self.act_dbg_step_into = QAction("Step Into", self)
        self.act_dbg_step_into.setShortcut(QKeySequence("F11"))
        self.act_dbg_step_into.triggered.connect(self._debug_step_into)
        self.act_dbg_step_out = QAction("Step Out", self)
        self.act_dbg_step_out.setShortcut(QKeySequence("Shift+F11"))
        self.act_dbg_step_out.triggered.connect(self._debug_step_out)
        self.act_dbg_stop = QAction("Debug stoppen", self)
        self.act_dbg_stop.triggered.connect(self._debug_stop)
        for a in (self.act_dbg_continue, self.act_dbg_step_over,
                  self.act_dbg_step_into, self.act_dbg_step_out,
                  self.act_dbg_stop):
            a.setEnabled(False)

        # View
        from .theme import active_theme as _at
        theme_icon_key = "theme_light" if _at() == "dark" else "theme_dark"
        self.act_theme = QAction(icons.get(theme_icon_key),
                                  "Theme wechseln (Dark/Light)", self)
        self.act_theme.setShortcut(QKeySequence("Ctrl+T"))
        self.act_theme.triggered.connect(self._toggle_theme)

        self.act_view_files = QAction("Dateien", self)
        self.act_view_files.setShortcut(QKeySequence("Ctrl+Shift+E"))
        self.act_view_files.triggered.connect(lambda: self._activate_sidebar("files"))

        self.act_view_outline = QAction("Outline", self)
        self.act_view_outline.setShortcut(QKeySequence("Ctrl+Shift+O"))
        self.act_view_outline.triggered.connect(lambda: self._activate_sidebar("outline"))

        self.act_view_builtins = QAction("Built-ins", self)
        self.act_view_builtins.setShortcut(QKeySequence("Ctrl+Shift+B"))
        self.act_view_builtins.triggered.connect(lambda: self._activate_sidebar("builtins"))

        self.act_show_readme = QAction("README anzeigen", self)
        self.act_show_readme.setShortcut(QKeySequence("F1"))
        self.act_show_readme.triggered.connect(self._show_readme)

        self.act_about = QAction(icons.get("info"), "Ueber GameBasic ...", self)
        self.act_about.triggered.connect(self._show_about)

        self.act_shortcuts = QAction("Tastenkuerzel ...", self)
        self.act_shortcuts.triggered.connect(self._show_shortcuts)

        self.act_toggle_minimap = QAction("Minimap anzeigen", self)
        self.act_toggle_minimap.setCheckable(True)
        self.act_toggle_minimap.setChecked(bool(self.settings.get("minimap_visible", True)))
        self.act_toggle_minimap.toggled.connect(self._on_minimap_toggled)

        self.act_toggle_wrap = QAction("Zeilenumbruch", self)
        self.act_toggle_wrap.setCheckable(True)
        self.act_toggle_wrap.setShortcut(QKeySequence("Alt+Z"))
        self.act_toggle_wrap.setChecked(bool(self.settings.get("word_wrap", False)))
        self.act_toggle_wrap.toggled.connect(self._on_wrap_toggled)

        self.act_toggle_todo = QAction("TODO / FIXME-Liste", self)
        self.act_toggle_todo.setCheckable(True)
        self.act_toggle_todo.setShortcut(QKeySequence("Ctrl+Shift+M"))
        self.act_toggle_todo.toggled.connect(
            lambda on: self.todo_dock.setVisible(on))

        # Bookmarks
        self.act_bookmark_toggle = QAction("Bookmark setzen/entfernen", self)
        self.act_bookmark_toggle.setShortcut(QKeySequence("Ctrl+F2"))
        self.act_bookmark_toggle.triggered.connect(
            lambda: self._with_active_editor(lambda e: e.toggle_bookmark()))
        self.act_bookmark_next = QAction("Nächstes Bookmark", self)
        self.act_bookmark_next.setShortcut(QKeySequence("F9"))
        self.act_bookmark_next.triggered.connect(
            lambda: self._with_active_editor(lambda e: e.next_bookmark()))
        self.act_bookmark_prev = QAction("Vorheriges Bookmark", self)
        self.act_bookmark_prev.setShortcut(QKeySequence("Shift+F9"))
        self.act_bookmark_prev.triggered.connect(
            lambda: self._with_active_editor(lambda e: e.prev_bookmark()))

        self.act_fold = QAction(icons.get("fold"), "Block falten/auffalten", self)
        self.act_fold.setShortcut(QKeySequence("Ctrl+Shift+["))
        self.act_fold.triggered.connect(self._fold_at_cursor)

        self.act_unfold_all = QAction(icons.get("unfold"), "Alle aufklappen", self)
        self.act_unfold_all.setShortcut(QKeySequence("Ctrl+Shift+]"))
        self.act_unfold_all.triggered.connect(self._unfold_all)

    def _build_toolbar(self) -> None:
        from PySide6.QtCore import QSize
        tb = QToolBar("Haupt")
        tb.setMovable(False)
        tb.setIconSize(QSize(20, 20))
        self.addToolBar(tb)
        for a in (self.act_new, self.act_open, self.act_save, self.act_save_as):
            tb.addAction(a)
        tb.addSeparator()
        tb.addAction(self.act_run)
        tb.addAction(self.act_run_native)
        tb.addAction(self.act_debug)
        tb.addAction(self.act_stop)
        tb.addAction(self.act_bench)
        tb.addAction(self.act_export)
        tb.addSeparator()
        tb.addAction(self.act_find)
        tb.addAction(self.act_replace)
        tb.addSeparator()
        tb.addAction(self.act_sprite_editor)
        tb.addAction(self.act_theme)
        # Objektnamen fuer farbige Hover-Akzente (siehe theme.global_qss):
        # Run gruen, Stop magenta.
        for act, obj_name in ((self.act_run, "RunButton"),
                              (self.act_run_native, "RunNativeButton"),
                              (self.act_stop, "StopButton")):
            w = tb.widgetForAction(act)
            if w is not None:
                w.setObjectName(obj_name)

    def _build_menubar(self) -> None:
        mb = self.menuBar()
        m_file = mb.addMenu("&Datei")
        m_file.addAction(self.act_new)
        m_file.addAction(self.act_open)
        m_file.addSeparator()
        m_file.addAction(self.act_save)
        m_file.addAction(self.act_save_as)
        m_file.addSeparator()
        self.menu_recent = m_file.addMenu("Zuletzt geoeffnet")
        self._rebuild_recent_menu()
        m_file.addSeparator()
        m_file.addAction(self.act_sprite_editor)
        m_file.addSeparator()
        m_file.addAction(self.act_close_tab)
        m_file.addAction(self.act_reopen_tab)
        m_file.addAction(self.act_quit)

        m_edit = mb.addMenu("&Bearbeiten")
        m_edit.addAction(self.act_find)
        m_edit.addAction(self.act_replace)
        m_edit.addAction(self.act_find_in_project)
        m_edit.addAction(self.act_goto)
        m_edit.addSeparator()
        m_edit.addAction(self.act_quick_open)
        m_edit.addAction(self.act_command_palette)
        m_edit.addAction(self.act_format_doc)
        m_edit.addSeparator()
        m_edit.addAction(self.act_bookmark_toggle)
        m_edit.addAction(self.act_bookmark_next)
        m_edit.addAction(self.act_bookmark_prev)
        m_edit.addSeparator()
        m_edit.addAction(self.act_settings)

        m_view = mb.addMenu("&Ansicht")
        m_view.addAction(self.act_view_files)
        m_view.addAction(self.act_view_outline)
        m_view.addAction(self.act_view_builtins)
        m_view.addSeparator()
        m_view.addAction(self.act_toggle_minimap)
        m_view.addAction(self.act_toggle_wrap)
        m_view.addAction(self.act_toggle_todo)
        m_view.addAction(self.act_fold)
        m_view.addAction(self.act_unfold_all)
        m_view.addAction(self.act_theme)

        m_run = mb.addMenu("Aus&fuehren")
        m_run.addAction(self.act_run)
        m_run.addAction(self.act_run_native)
        m_run.addAction(self.act_stop)
        m_run.addSeparator()
        m_run.addAction(self.act_bench)
        m_run.addAction(self.act_export)

        m_debug = mb.addMenu("&Debug")
        m_debug.addAction(self.act_debug)
        m_debug.addSeparator()
        m_debug.addAction(self.act_dbg_continue)
        m_debug.addAction(self.act_dbg_step_over)
        m_debug.addAction(self.act_dbg_step_into)
        m_debug.addAction(self.act_dbg_step_out)
        m_debug.addSeparator()
        m_debug.addAction(self.act_dbg_stop)

        m_help = mb.addMenu("&Hilfe")
        m_help.addAction(self.act_show_readme)
        m_help.addAction(self.act_shortcuts)
        m_help.addSeparator()
        m_help.addAction(self.act_about)

        # Logo als Corner-Widget rechts -- klein, dezent, immer sichtbar.
        # Muss NACH dem Hinzufuegen aller Menus passieren, sonst wird es
        # von Qt verworfen.
        self._add_menubar_logo(mb)

    def _wire_signals(self) -> None:
        self.file_browser.file_activated.connect(self._open_file)
        self.outline.jump_requested.connect(self._goto_line_in_active)
        self.builtins_panel.insert_requested.connect(self._insert_in_active)

        self.tabs.active_changed.connect(self._on_active_tab_changed)
        self.tabs.dirty_changed.connect(self._update_status)
        self.tabs.close_requested.connect(self._on_tab_close_requested)
        self.tabs.save_requested.connect(self._save_active)
        self.tabs.run_requested.connect(self._run_active)
        self.tabs.tab_opened.connect(self._on_tab_opened)
        self.tabs.reveal_requested.connect(self._on_tab_reveal)
        self.tabs.close_others_requested.connect(self._on_tab_close_others)

        self.activity.selected.connect(self._show_sidebar_view)
        self.activity.sidebar_toggled.connect(self._set_sidebar_visible)

        self.console.process_started.connect(self._on_run_started)
        self.console.process_finished.connect(self._on_run_finished)
        self.console.jump_requested.connect(self._on_console_jump)

    def _on_tab_opened(self, st: TabState) -> None:
        st.editor.cursorPositionChanged.connect(self._update_cursor_label)
        st.editor.set_auto_complete(self._auto_complete_default)
        # Outline triggert auf Inhalts-Aenderungen mit Debounce.
        self._outline_timer = getattr(self, "_outline_timer", None) or self._make_outline_timer()
        st.editor.textChanged.connect(self._schedule_outline_refresh)
        # Minimap-Sichtbarkeit gemaess globaler Toggle-Action.
        st.minimap.setVisible(self.act_toggle_minimap.isChecked())
        # Zeilenumbruch gemaess globaler Toggle-Action.
        st.editor.set_word_wrap(self.act_toggle_wrap.isChecked())
        # Breakpoints des Tabs -> Debugger (wenn dieser Tab debuggt wird).
        st.editor.breakpoints_changed.connect(
            lambda e=st.editor: self._on_breakpoints_changed(e))
        # Goto / Find-Refs / Rename
        st.editor.goto_definition_requested.connect(self._goto_definition)
        st.editor.find_references_requested.connect(self._find_references)
        st.editor.rename_requested.connect(self._rename_symbol)
        # Run-Selection (Shift+F5)
        st.editor.run_selection_requested.connect(self._run_selection)
        # Live-Error -> Statusbar
        st.editor._error_checker.problem_changed.connect(self._on_live_error_changed)
        # IMPORT-Aufloesung: Datei-Verzeichnis (oder Project-Root, falls
        # noch ungespeichert).
        base = st.file_path.parent if st.file_path is not None else self.project_root
        st.editor.set_error_base_path(base)
        # Initialer Check
        st.editor._kick_error_check()
        # Folding-State wiederherstellen (nur fuer benannte Files; "(neu)"
        # hat keinen stabilen Key in den Settings).
        if st.file_path is not None:
            saved = self.settings.get("fold_state", {}).get(str(st.file_path))
            if saved:
                st.editor.apply_folded_starts(saved)

    def _on_live_error_changed(self, problem) -> None:
        # Nur fuer den aktiven Tab anzeigen. WICHTIG: Tabs im Hintergrund
        # halten ihre eigenen LiveErrorChecker-Threads -- ein verspaetetes
        # Resultat von einem inaktiven Tab darf hier nicht zu Statusbar-
        # Updates fuehren (z.B. clearMessage(), wenn Tab A "ok" liefert
        # waehrend Tab B sehr wohl einen Fehler hat).
        st = self.tabs.active
        if st is None:
            return
        sender = self.sender()
        if sender is not None and sender is not st.editor._error_checker:
            return  # stale Update von einem nicht-aktiven Tab
        if problem is None:
            self.statusBar().clearMessage()
            return
        if st.editor.current_error() is problem:
            self.statusBar().showMessage(
                f"Zeile {problem.line}: {problem.message}", 0,
            )

    def _fold_at_cursor(self) -> None:
        st = self.tabs.active
        if st is not None:
            st.editor.fold_block_at_cursor()

    def _unfold_all(self) -> None:
        st = self.tabs.active
        if st is not None:
            st.editor.unfold_all()

    def _on_minimap_toggled(self, on: bool) -> None:
        for st in self.tabs.states:
            st.minimap.setVisible(on)
        self.settings["minimap_visible"] = bool(on)

    def _on_wrap_toggled(self, on: bool) -> None:
        for st in self.tabs.states:
            st.editor.set_word_wrap(on)
        self.settings["word_wrap"] = bool(on)

    def _with_active_editor(self, fn) -> None:
        st = self.tabs.active
        if st is not None:
            fn(st.editor)

    def _make_outline_timer(self) -> QTimer:
        t = QTimer(self)
        t.setSingleShot(True)
        t.setInterval(150)
        t.timeout.connect(self._refresh_outline)
        return t

    def _schedule_outline_refresh(self) -> None:
        if hasattr(self, "_outline_timer"):
            self._outline_timer.start()

    def _refresh_outline(self) -> None:
        st = self.tabs.active
        text = st.editor.get_text() if st is not None else None
        self.outline.update_for(text)
        if getattr(self, "todo_dock", None) is not None and self.todo_dock.isVisible():
            self.todo_panel.update_for(text)

    # ----------------------------------------------------- Aktionen
    def _new_tab(self) -> TabState:
        st = self.tabs.open_tab(file_path=None, content="")
        self._update_status()
        self._update_welcome_visibility()
        return st

    def _open_dialog(self) -> None:
        path_str, _ = QFileDialog.getOpenFileName(
            self, "GB-Datei oeffnen",
            str(self.project_root),
            "GameBasic-Dateien (*.gb);;Alle Dateien (*)",
        )
        if path_str:
            self._open_file(Path(path_str))

    def _open_file(self, path: Path) -> None:
        path = Path(path).resolve()
        # Markdown (z.B. Modul-Doku aus dem Datei-Browser) gerendert im
        # Markdown-Viewer statt als Roh-Text im Code-Tab oeffnen.
        if path.suffix.lower() in (".md", ".markdown"):
            self._show_markdown(path)
            return
        existing = self.tabs.find_tab_for(path)
        if existing is not None:
            self.tabs.activate(existing)
            return
        try:
            content = path.read_text(encoding="utf-8")
        except OSError as exc:
            QMessageBox.warning(
                self, "Fehler beim Lesen",
                f"Konnte {path} nicht oeffnen:\n{exc}",
            )
            return
        self.tabs.open_tab(file_path=path, content=content)
        self._add_to_recent(path)
        self.file_browser.mark_active(path)
        self._update_status()
        self._refresh_outline()
        self._update_welcome_visibility()

    def _save_active(self) -> bool:
        st = self.tabs.active
        if st is None:
            return False
        if st.file_path is None:
            return self._save_active_as()
        return self._write_tab(st, st.file_path)

    def _save_active_as(self) -> bool:
        st = self.tabs.active
        if st is None:
            return False
        path_str, _ = QFileDialog.getSaveFileName(
            self, "Speichern unter",
            str(st.file_path) if st.file_path else str(self.project_root / "neu.gb"),
            "GameBasic-Dateien (*.gb);;Alle Dateien (*)",
        )
        if not path_str:
            return False
        path = Path(path_str)
        if path.suffix.lower() != ".gb":
            path = path.with_suffix(".gb")
        if not self._write_tab(st, path):
            return False
        st.file_path = path
        self.tabs.update_tab_title(st)
        self._add_to_recent(path)
        self.file_browser.refresh()
        self.file_browser.mark_active(path)
        # Base fuer IMPORT-Aufloesung umsetzen.
        st.editor.set_error_base_path(path.parent)
        st.editor._kick_error_check()
        return True

    def _write_tab(self, st: TabState, path: Path) -> bool:
        # Format-on-Save: wenn aktiviert, vor dem Schreiben den Buffer
        # formatieren. Cursor-Zeile wird beibehalten, damit der User nach
        # dem Save nicht orientierungslos ist. Bei Format-Fehlern (sehr
        # selten -- der Formatter ist tolerant gegen Syntax-Fehler) wird
        # die Original-Quelle geschrieben, lieber unformatiert speichern
        # als gar nicht.
        if self.settings.get("format_on_save", False):
            try:
                original = st.editor.get_text()
                formatted = format_source(original)
                if formatted != original:
                    cur_line = st.editor.textCursor().blockNumber() + 1
                    st.editor.set_text(formatted)
                    st.editor.document().setModified(True)
                    st.editor.goto_line(cur_line)
            except Exception:
                pass
        try:
            path.write_text(st.editor.get_text(), encoding="utf-8")
        except OSError as exc:
            QMessageBox.warning(
                self, "Fehler beim Speichern",
                f"Konnte {path} nicht schreiben:\n{exc}",
            )
            return False
        self.tabs.mark_clean(st)
        self.statusBar().showMessage(f"Gespeichert: {path.name}", 3000)
        return True

    def _close_active_tab(self) -> None:
        st = self.tabs.active
        if st is not None:
            self._on_tab_close_requested(st)

    def _on_tab_close_requested(self, st: TabState) -> None:
        if self.tabs.is_dirty(st):
            ans = self._confirm_save(st)
            if ans is None:
                return
            if ans is True and not self._save_tab(st):
                return
        # Pfad fuer Re-Open merken (max 20 Eintraege).
        if st.file_path is not None:
            p = str(st.file_path)
            if p in self._closed_stack:
                self._closed_stack.remove(p)
            self._closed_stack.append(p)
            self._closed_stack = self._closed_stack[-20:]
        # Folding-State persistieren -- nur fuer benannte Files. Beim
        # naechsten Open wird er von `_on_tab_opened` wieder eingespielt.
        if st.file_path is not None:
            self._persist_fold_state(str(st.file_path), st.editor.folded_starts())
        self.tabs.close_tab(st)
        # Autosave-Manifest sofort refreshen: sonst bleibt ein gerade mit
        # "Verwerfen" geschlossener Tab im Manifest sichtbar, bis der
        # naechste 30s-Autosave-Tick es ueberschreibt. Crash dazwischen
        # = der Tab taucht beim Editor-Neustart als Recovery auf, obwohl
        # der User ihn explizit verworfen hat.
        self._do_autosave_impl()
        self._update_welcome_visibility()

    def _on_tab_reveal(self, st: TabState) -> None:
        """Schaltet auf Files-Sidebar und scrollt zur Tab-Datei."""
        if st.file_path is None:
            self.statusBar().showMessage("Tab hat noch keinen Dateipfad.", 2000)
            return
        self._activate_sidebar("files")
        # `mark_active` klappt auch die Tree-Eltern auf + scrollt rein.
        self.file_browser.mark_active(st.file_path)

    def _on_tab_close_others(self, st: TabState) -> None:
        """Schliesst alle Tabs ausser dem angegebenen."""
        for other in list(self.tabs.states):
            if other is st:
                continue
            self._on_tab_close_requested(other)

    def _update_welcome_visibility(self) -> None:
        """Schaltet zwischen Welcome-Panel und Editor-Splitter."""
        if not self.tabs.states:
            self.welcome.update_recent(self.recent)
            self.right_stack.setCurrentIndex(0)
        else:
            self.right_stack.setCurrentIndex(1)

    def _welcome_new(self) -> None:
        self._new_tab()

    def _welcome_show_examples(self) -> None:
        # Sidebar auf Files schalten und auf den examples-Ordner scrollen.
        self._activate_sidebar("files")
        self.statusBar().showMessage(
            "Tipp: Doppelklick auf eine .gb-Datei in der Seitenleiste oeffnet sie.",
            4000,
        )

    def _reopen_closed_tab(self) -> None:
        while self._closed_stack:
            p = self._closed_stack.pop()
            path = Path(p)
            if path.exists():
                self._open_file(path)
                return
        self.statusBar().showMessage("Kein geschlossener Tab zum Wiederoeffnen.", 2500)

    def _save_tab(self, st: TabState) -> bool:
        self.tabs.activate(st)
        return self._save_active()

    def _confirm_save(self, st: TabState) -> Optional[bool]:
        name = st.file_path.name if st.file_path else "(neu)"
        box = QMessageBox(self)
        box.setIcon(QMessageBox.Icon.Question)
        box.setWindowTitle("Ungespeicherte Aenderungen")
        box.setText(f"Aenderungen in {name} speichern?")
        box.setStandardButtons(
            QMessageBox.StandardButton.Yes
            | QMessageBox.StandardButton.No
            | QMessageBox.StandardButton.Cancel
        )
        ans = box.exec()
        if ans == QMessageBox.StandardButton.Yes:
            return True
        if ans == QMessageBox.StandardButton.No:
            return False
        return None

    # ---------------------------------------------------- Edit-Hooks
    def _ensure_find_dialog(self, with_replace: bool) -> FindReplaceDialog:
        attr = "_find_dialog_replace" if with_replace else "_find_dialog_find"
        dlg = getattr(self, attr, None)
        if dlg is None:
            dlg = FindReplaceDialog(self, lambda: self._active_editor(), with_replace)
            setattr(self, attr, dlg)
        return dlg

    def _show_find(self) -> None:
        self._ensure_find_dialog(with_replace=False).show_for()

    def _show_replace(self) -> None:
        self._ensure_find_dialog(with_replace=True).show_for()

    def _show_find_in_project(self) -> None:
        if not hasattr(self, "_find_project_dialog"):
            self._find_project_dialog = FindInProjectDialog(self, self.project_root)
            self._find_project_dialog.open_requested.connect(self._on_project_jump)
        # Selektion als Default-Suchbegriff vorbelegen.
        st = self.tabs.active
        prefill = None
        if st is not None:
            sel = st.editor.textCursor().selectedText()
            if sel:
                prefill = sel
        self._find_project_dialog.show_for(prefill)

    def _on_project_jump(self, path: Path, line: int) -> None:
        self._open_file(path)
        st = self.tabs.active
        if st is not None and line > 0:
            st.editor.goto_line(line)
            st.editor.setFocus()

    def _show_goto(self) -> None:
        if not hasattr(self, "_goto_dialog"):
            self._goto_dialog = GotoLineDialog(self, lambda: self._active_editor())
        self._goto_dialog.exec_for_editor()

    def _show_settings(self) -> None:
        dlg = SettingsDialog(self, dict(self.settings))
        if dlg.exec() != dlg.DialogCode.Accepted:
            return
        new_theme = dlg.selected_theme()
        self.settings["theme"] = new_theme
        self.settings["autocomplete_auto"] = bool(dlg.auto_complete_enabled())
        self.settings["format_on_save"] = bool(dlg.format_on_save_enabled())
        save_settings(self.settings)
        # Theme + Auto-Trigger live anwenden.
        if new_theme != COLORS_active_name():
            set_active_theme(new_theme)
        self._auto_complete_default = self.settings["autocomplete_auto"]
        for st in self.tabs.states:
            st.editor.set_auto_complete(self._auto_complete_default)

    def _active_editor(self):
        st = self.tabs.active
        return st.editor if st is not None else None

    def _goto_definition(self, name: str) -> None:
        from . import symbols as gb_symbols
        st = self.tabs.active
        if st is None:
            return
        d = gb_symbols.find_definition(st.editor.get_text(), name)
        if d is None:
            self.statusBar().showMessage(f"Keine Definition fuer '{name}' gefunden.", 3000)
            return
        st.editor.goto_line(d.line)
        # Selektion auf den Namen setzen, damit es klar ist, was gefunden wurde.
        block = st.editor.document().findBlockByNumber(d.line - 1)
        cur = st.editor.textCursor()
        cur.setPosition(block.position() + d.col - 1)
        cur.setPosition(block.position() + d.col_end - 1, cur.MoveMode.KeepAnchor)
        st.editor.setTextCursor(cur)
        st.editor.setFocus()

    def _find_references(self, name: str) -> None:
        from . import symbols as gb_symbols
        st = self.tabs.active
        if st is None:
            return
        refs = gb_symbols.scan_references(st.editor.get_text(), name)
        if not refs:
            self.statusBar().showMessage(f"Keine Vorkommen von '{name}'.", 3000)
            return
        # Wir verwenden die Konsole als Listing-Surface -- Datei:Line-Format
        # ist via _on_console_jump klickbar.
        self.console.clear()
        active_path = st.file_path or self.project_root / "(neu)"
        self.console.append(f"Vorkommen von '{name}':\n", "info")
        for r in refs:
            self.console.append(f"  {active_path.name}:{r.line}\n")
        self.console.append(f"\n{len(refs)} Treffer.\n", "muted")
        # Konsole sicher sichtbar bringen.
        self.right_stack.setCurrentIndex(1)

    def _rename_symbol(self, name: str) -> None:
        from PySide6.QtWidgets import QInputDialog
        from . import symbols as gb_symbols
        st = self.tabs.active
        if st is None:
            return
        new, ok = QInputDialog.getText(
            self, "Symbol umbenennen",
            f"Neuen Namen fuer '{name}' eingeben:",
            text=name,
        )
        if not ok or not new or new == name:
            return
        # Validierung: Identifier-Pattern.
        import re
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", new):
            QMessageBox.warning(self, "Ungueltig",
                                "Name muss ein gueltiger Identifier sein.")
            return
        text = st.editor.get_text()
        refs = gb_symbols.scan_references(text, name)
        if not refs:
            return
        # Edits absteigend nach Position, damit fruhere keine spaeteren
        # Positionen verschieben.
        lines = text.split("\n")
        edits_by_line: dict[int, list[tuple[int, int]]] = {}
        for r in refs:
            edits_by_line.setdefault(r.line, []).append((r.col, r.col_end))
        for ln, pairs in edits_by_line.items():
            pairs.sort(key=lambda t: -t[0])
            old = lines[ln - 1]
            new_line = old
            for col, col_end in pairs:
                new_line = new_line[:col - 1] + new + new_line[col_end - 1:]
            lines[ln - 1] = new_line
        st.editor.set_text("\n".join(lines))
        st.editor.document().setModified(True)
        self.statusBar().showMessage(
            f"{len(refs)} Vorkommen von '{name}' -> '{new}' ersetzt.", 3000,
        )

    def _goto_line_in_active(self, line: int) -> None:
        st = self.tabs.active
        if st is None:
            return
        st.editor.goto_line(line)
        st.editor.setFocus()

    def _insert_in_active(self, text: str) -> None:
        st = self.tabs.active
        if st is None:
            return
        st.editor.insert_at_cursor(text)
        st.editor.setFocus()

    # ---------------------------------------------------- Run-Hooks
    def _run_active(self) -> None:
        if self.debugger.is_active():
            return
        if not self._ensure_saved_for_run():
            return
        st = self.tabs.active
        assert st is not None and st.file_path is not None
        if self.console.start_run(st.file_path):
            self.tabs.set_running(st.file_path, "py")
            self.statusBar().showMessage(
                f"▶ Laeuft: {st.file_path.name} (Tree-Walker)")

    def _run_native_active(self) -> None:
        if not self._ensure_saved_for_run():
            return
        st = self.tabs.active
        assert st is not None and st.file_path is not None
        if self.console.start_run_native(st.file_path):
            self.tabs.set_running(st.file_path, "native")
            self.statusBar().showMessage(
                f"⚙ Laeuft nativ: {st.file_path.name} (gbrt)")

    def _export_active(self) -> None:
        """Buendelt die aktive Datei zu einer standalone .exe (gbrt + Bytecode +
        assets/). Laeuft ohne Python; Ergebnis-Ordner wird im Explorer geoeffnet."""
        if not self._ensure_saved_for_run():
            return
        st = self.tabs.active
        if st is None or st.file_path is None:
            return
        from .output_console import _find_gbrt
        from gamebasic.export import export_standalone
        from gamebasic.errors import GameBasicError
        from PySide6.QtGui import QGuiApplication, QDesktopServices
        from PySide6.QtCore import QUrl, Qt

        gbrt = _find_gbrt(self.project_root)
        if gbrt is None:
            self.console.append(
                "Native Runtime 'gbrt' nicht gefunden -- erst bauen:\n"
                "  .venv\\Scripts\\python.exe rust\\build_runtime.py\n", "error")
            self.statusBar().showMessage("Export: gbrt fehlt", 4000)
            return
        self.console.append(f"⚙ Exportiere {st.file_path.name} ...\n", "info")
        QGuiApplication.setOverrideCursor(Qt.CursorShape.WaitCursor)
        try:
            exe = export_standalone(st.file_path, gbrt)
        except GameBasicError as exc:
            QGuiApplication.restoreOverrideCursor()
            self.console.append(f"Compile-Fehler: {exc}\n", "error")
            self.statusBar().showMessage("Export fehlgeschlagen", 4000)
            return
        except Exception as exc:  # noqa: BLE001 -- alle Fehler dem User zeigen
            QGuiApplication.restoreOverrideCursor()
            self.console.append(f"Export-Fehler: {exc}\n", "error")
            self.statusBar().showMessage("Export fehlgeschlagen", 4000)
            return
        QGuiApplication.restoreOverrideCursor()
        mb = exe.stat().st_size / (1024 * 1024)
        self.console.append(
            f"✓ Standalone-Export: {exe}  ({mb:.1f} MB)\n"
            f"  Ordner '{exe.parent.name}' weitergeben -- laeuft ohne Python.\n", "info")
        self.statusBar().showMessage(f"Exportiert: {exe.name}", 5000)
        QDesktopServices.openUrl(QUrl.fromLocalFile(str(exe.parent)))

    def _run_selection(self, snippet: str) -> None:
        """Schreibt den selektierten Code in eine temp-Datei und startet
        gbrun.py darauf -- erlaubt schnelles Ausprobieren ohne den Buffer
        zu speichern.

        Pfad-Strategie: temp-Datei landet im Project-Root, nicht im
        System-Tmp -- so funktionieren `LOADIMAGE("assets/...")`-Pfade
        weiter (gbrun.py macht `os.chdir(file.parent)`). Filename ist
        eindeutig (`__selection_<pid>.gb`), damit parallele Editoren
        sich nicht in die Quere kommen.

        Cleanup: nach Process-Finish loeschen wir die Datei. Wenn der
        User waehrend des Runs den Editor schliesst, bleibt sie liegen --
        beim naechsten Start wird sie ueberschrieben.
        """
        if self.console.is_running():
            self.statusBar().showMessage(
                "Programm laeuft schon. Erst stoppen.", 3000)
            return
        snippet = (snippet or "").strip()
        if not snippet:
            self.statusBar().showMessage("Nichts selektiert.", 2000)
            return
        import os
        temp_path = self.project_root / f"__selection_{os.getpid()}.gb"
        try:
            temp_path.write_text(snippet, encoding="utf-8")
        except OSError as exc:
            self.statusBar().showMessage(
                f"Konnte Selection nicht schreiben: {exc}", 4000)
            return
        # Datei nach Run-Ende aufraeumen. Connect ueber single-shot-Lambda
        # mit `connect`-Reference, damit wir uns nach dem Cleanup wieder
        # disconnecten -- sonst feuert es bei jedem zukuenftigen Run.
        def _cleanup(_exit_code, _path=temp_path):
            try:
                self.console.process_finished.disconnect(_cleanup)
            except (TypeError, RuntimeError):
                pass
            try:
                _path.unlink(missing_ok=True)
            except OSError:
                pass
        self.console.process_finished.connect(_cleanup)
        self.console.start_run(temp_path)

    def _bench_active(self) -> None:
        if not self._ensure_saved_for_run():
            return
        st = self.tabs.active
        assert st is not None and st.file_path is not None
        if self.console.start_run(st.file_path, ["--bench"]):
            self.tabs.set_running(st.file_path, "py")
            self.statusBar().showMessage(f"⚡ Benchmark: {st.file_path.name}")

    def _ensure_saved_for_run(self) -> bool:
        if self.console.is_running():
            return False
        st = self.tabs.active
        if st is None:
            return False
        if st.file_path is None:
            if not self._save_active_as():
                return False
        if self.tabs.is_dirty(st):
            if not self._save_active():
                return False
        return True

    def _on_run_started(self) -> None:
        self.act_run.setEnabled(False)
        self.act_run_native.setEnabled(False)
        self.act_bench.setEnabled(False)
        self.act_stop.setEnabled(True)
        self.statusBar().showMessage("Laeuft ...")

    def _on_run_finished(self, _rc: int) -> None:
        self.act_run.setEnabled(True)
        self.act_run_native.setEnabled(True)
        self.act_bench.setEnabled(True)
        self.act_stop.setEnabled(False)
        self.tabs.set_running(None)        # Run-Markierung am Tab abraeumen
        self.statusBar().showMessage("Bereit", 3000)

    # ----------------------------------------------------- Debugger
    def _setup_debugger(self) -> None:
        from PySide6.QtWidgets import QDockWidget
        from .debugger import DebugController
        from .debug_panel import VariablesPanel
        self.debugger = DebugController(self)
        self.variables_panel = VariablesPanel()
        self.var_dock = QDockWidget("Variablen", self)
        self.var_dock.setObjectName("VariablesDock")
        self.var_dock.setWidget(self.variables_panel)
        self.addDockWidget(Qt.DockWidgetArea.RightDockWidgetArea, self.var_dock)
        self.var_dock.hide()
        self._debug_editor = None          # Editor der laufenden Sitzung
        self.debugger.paused.connect(self._on_debug_paused)
        self.debugger.finished.connect(self._on_debug_finished)
        self.debugger.output.connect(self._on_debug_output)
        self.debugger.failed.connect(self._on_debug_failed)

    def _setup_todo_panel(self) -> None:
        from PySide6.QtWidgets import QDockWidget
        from .todo_panel import TodoPanel
        self.todo_panel = TodoPanel()
        self.todo_dock = QDockWidget("TODO / FIXME", self)
        self.todo_dock.setObjectName("TodoDock")
        self.todo_dock.setWidget(self.todo_panel)
        self.addDockWidget(Qt.DockWidgetArea.RightDockWidgetArea, self.todo_dock)
        self.todo_dock.hide()
        self.todo_panel.jump_requested.connect(self._goto_line_in_active)
        # Sichtbarkeit togglet die View-Action; beim Einblenden frisch scannen.
        self.act_toggle_todo.setChecked(False)
        self.todo_dock.visibilityChanged.connect(self._on_todo_visibility)

    def _on_todo_visibility(self, visible: bool) -> None:
        self.act_toggle_todo.setChecked(visible)
        if visible:
            st = self.tabs.active
            self.todo_panel.update_for(st.editor.get_text() if st else None)

    def _on_breakpoints_changed(self, editor) -> None:
        # Nur relevant, wenn dieser Editor gerade debuggt wird -- dann live an
        # den Controller weiterreichen.
        if self.debugger.is_active() and editor is self._debug_editor:
            self.debugger.set_breakpoints(editor.breakpoints())

    def _debug_start(self) -> None:
        if self.debugger.is_active():
            return
        if not self._ensure_saved_for_run():
            return
        st = self.tabs.active
        if st is None or st.file_path is None:
            return
        self._debug_editor = st.editor
        base = st.file_path.parent
        self.console.clear()
        self.console.append(f"🐞 Debug: {st.file_path.name}\n\n", "info")
        self.debugger.set_breakpoints(st.editor.breakpoints())
        if not self.debugger.start(st.editor.get_text(), base):
            self._debug_editor = None
            return
        self._set_debug_controls(active=True, paused=False)
        self.act_debug.setEnabled(False)
        for a in (self.act_run, self.act_run_native, self.act_bench):
            a.setEnabled(False)
        self.var_dock.show()
        self.variables_panel.set_status("läuft ...")
        self.statusBar().showMessage("Debug: läuft ...")

    def _on_debug_paused(self, editor_line: int, frames: object) -> None:
        self._set_debug_controls(active=True, paused=True)
        self.variables_panel.update_frames(frames)
        self.variables_panel.set_status(f"angehalten @ Zeile {editor_line}")
        if self._debug_editor is not None and editor_line > 0:
            self._debug_editor.set_debug_line(editor_line)
            self.statusBar().showMessage(f"Debug: angehalten in Zeile {editor_line}")
        else:
            self.statusBar().showMessage("Debug: angehalten (importierte Datei)")

    def _on_debug_output(self, text: str) -> None:
        self.console.append(text)

    def _on_debug_finished(self, reason: str) -> None:
        if self._debug_editor is not None:
            self._debug_editor.set_debug_line(None)
        self._set_debug_controls(active=False, paused=False)
        self.act_debug.setEnabled(True)
        for a in (self.act_run, self.act_run_native, self.act_bench):
            a.setEnabled(True)
        self.variables_panel.set_status(reason)
        self.variables_panel.clear()
        icon = {"fertig": "✓", "abgebrochen": "■"}.get(reason, "✗")
        self.console.append(f"\n{icon} Debug {reason}.\n",
                            "info" if reason == "fertig" else "muted")
        self.statusBar().showMessage(f"Debug {reason}", 3000)
        self._debug_editor = None

    def _on_debug_failed(self, message: str, editor_line: int) -> None:
        loc = f" (Zeile {editor_line})" if editor_line and editor_line > 0 else ""
        self.console.append(f"Laufzeitfehler{loc}: {message}\n", "error")
        if self._debug_editor is not None and editor_line and editor_line > 0:
            self._debug_editor.set_debug_line(editor_line)

    def _debug_continue(self) -> None:
        if self.debugger.is_paused():
            self._clear_debug_line_keep_session()
            self.debugger.cont()

    def _debug_step_over(self) -> None:
        if self.debugger.is_paused():
            self._clear_debug_line_keep_session()
            self.debugger.step_over()

    def _debug_step_into(self) -> None:
        if self.debugger.is_paused():
            self._clear_debug_line_keep_session()
            self.debugger.step_into()

    def _debug_step_out(self) -> None:
        if self.debugger.is_paused():
            self._clear_debug_line_keep_session()
            self.debugger.step_out()

    def _debug_stop(self) -> None:
        if self.debugger.is_active():
            self.debugger.stop()

    def _clear_debug_line_keep_session(self) -> None:
        if self._debug_editor is not None:
            self._debug_editor.set_debug_line(None)
        self._set_debug_controls(active=True, paused=False)

    def _set_debug_controls(self, *, active: bool, paused: bool) -> None:
        for a in (self.act_dbg_continue, self.act_dbg_step_over,
                  self.act_dbg_step_into, self.act_dbg_step_out):
            a.setEnabled(paused)
        self.act_dbg_stop.setEnabled(active)

    def _on_console_jump(self, file: object, line: int) -> None:
        target_path: Path | None = None
        if isinstance(file, str):
            cand = Path(file)
            if not cand.is_absolute():
                base = self.tabs.active.file_path.parent if (
                    self.tabs.active is not None and self.tabs.active.file_path is not None
                ) else self.project_root
                cand = base / file
            if cand.exists():
                target_path = cand.resolve()
        if target_path is not None:
            self._open_file(target_path)
        st = self.tabs.active
        if st is not None and line > 0:
            st.editor.goto_line(line)
            st.editor.setFocus()

    # ----------------------------------------------------- View
    def _toggle_theme(self) -> None:
        new_theme = "light" if COLORS_active_name() == "dark" else "dark"
        set_active_theme(new_theme)
        self.settings["theme"] = new_theme
        save_settings(self.settings)

    def _on_theme_changed(self, _name: str) -> None:
        self._apply_app_stylesheet()
        self._refresh_theme_label()
        self._refresh_action_icons()

    def _refresh_action_icons(self) -> None:
        """Setzt die QAction-Icons aus der Cache-frischen IconProvider neu."""
        for key, act in (
            ("new",      self.act_new),
            ("open",     self.act_open),
            ("save",     self.act_save),
            ("save_as",  self.act_save_as),  # in der Toolbar sichtbar
            ("find",     self.act_find),
            ("replace",  self.act_replace),
            ("settings", self.act_settings),
            ("run",      self.act_run),
            ("stop",     self.act_stop),
            ("bench",    self.act_bench),
            ("fold",     self.act_fold),
            ("unfold",   self.act_unfold_all),
            ("info",     self.act_about),
            ("sprite_editor", self.act_sprite_editor),
        ):
            act.setIcon(icons.get(key))
        # Theme-Action haengt am aktuellen Theme.
        from .theme import active_theme as _at
        self.act_theme.setIcon(icons.get("theme_light" if _at() == "dark" else "theme_dark"))

    def _refresh_theme_label(self) -> None:
        if not hasattr(self, "theme_label"):
            return
        name = COLORS_active_name()
        # Symbol fuer Sun/Moon -- pure Unicode, keine Asset-Abhaengigkeit.
        glyph = "🌙" if name == "dark" else "☀"
        self.theme_label.setText(f"{glyph}  {name.capitalize()}")

    def _apply_app_stylesheet(self) -> None:
        from PySide6.QtWidgets import QApplication
        QApplication.instance().setStyleSheet(global_qss())

    def _show_about(self) -> None:
        AboutDialog(self).exec()

    def _open_sprite_editor(self) -> None:
        """Oeffnet den Sprite-Editor als zweites Top-Level-Fenster.

        Wir importieren spriteeditor_qt lazy (er bringt 5000+ Zeilen mit)
        und instanziieren `SpriteEditorWindow` direkt -- die `launch()`-
        Funktion wuerde `app.exec()` rufen und unseren laufenden
        Event-Loop reentrieren. Reference auf das Fenster muss erhalten
        bleiben, sonst kassiert Qt es per GC.
        """
        try:
            from .. import spriteeditor_qt as se
        except SystemExit as exc:
            QMessageBox.warning(
                self, "Sprite-Editor nicht verfuegbar",
                str(exc),
            )
            return
        except Exception as exc:
            QMessageBox.warning(
                self, "Sprite-Editor-Fehler",
                f"Konnte Sprite-Editor nicht laden:\n{type(exc).__name__}: {exc}",
            )
            return
        # Reference halten + bei Close verwerfen. Stylesheet bekommt das
        # Sprite-Window EXKLUSIV (per-Window) statt App-weit -- so bleibt
        # unser Editor-Style fuer den Code-Editor unangetastet.
        self._sprite_editor_window = se.SpriteEditorWindow(self.project_root)
        try:
            self._sprite_editor_window.setStyleSheet(se.GLOBAL_QSS)
        except Exception:
            pass
        self._sprite_editor_window.setAttribute(
            Qt.WidgetAttribute.WA_DeleteOnClose, True,
        )
        self._sprite_editor_window.destroyed.connect(
            lambda: setattr(self, "_sprite_editor_window", None),
        )
        self._sprite_editor_window.show()
        self._sprite_editor_window.raise_()
        self._sprite_editor_window.activateWindow()

    def _show_shortcuts(self) -> None:
        """Sammelt alle QActions + Editor-interne Shortcuts und zeigt sie."""
        entries: list[tuple[str, str]] = []

        def add_action_group(title: str, actions):
            entries.append((f"--- {title} ---", ""))
            for act in actions:
                sc = act.shortcut().toString() if act.shortcut() else ""
                entries.append((act.text().replace("&", ""), sc))

        add_action_group("Datei", [
            self.act_new, self.act_open, self.act_save, self.act_save_as,
            self.act_sprite_editor,
            self.act_close_tab, self.act_reopen_tab, self.act_quit,
        ])
        add_action_group("Bearbeiten", [
            self.act_find, self.act_replace, self.act_find_in_project,
            self.act_goto, self.act_quick_open, self.act_command_palette,
            self.act_format_doc, self.act_settings,
        ])
        add_action_group("Ansicht", [
            self.act_view_files, self.act_view_outline, self.act_view_builtins,
            self.act_toggle_minimap, self.act_fold, self.act_unfold_all,
            self.act_theme,
        ])
        add_action_group("Ausfuehren", [
            self.act_run, self.act_run_native, self.act_stop, self.act_bench,
            self.act_export,
        ])
        add_action_group("Hilfe", [
            self.act_show_readme, self.act_shortcuts, self.act_about,
        ])
        # Editor-interne Shortcuts (kein QAction):
        entries.append(("--- Editor (im Code-Bereich) ---", ""))
        entries.extend([
            ("Comment-Toggle", "Ctrl+/"),
            ("Zeile duplizieren", "Ctrl+Shift+D"),
            ("Zeile verschieben hoch/runter", "Alt+Up / Alt+Down"),
            ("Multi-Cursor: naechstes Vorkommen", "Ctrl+D"),
            ("Multi-Cursor verwerfen", "Esc"),
            ("Goto-Definition", "F12 / Ctrl+Click"),
            ("Find-References", "Shift+F12"),
            ("Symbol umbenennen", "F2"),
            ("Auto-Completion manuell", "Ctrl+Space"),
            ("Schriftgroesse +/- /reset", "Ctrl++ / Ctrl+- / Ctrl+0"),
            ("Schriftgroesse via Mausrad", "Ctrl+Wheel"),
            ("Konsole durchsuchen (Output)", "Ctrl+F (in Konsole)"),
        ])
        ShortcutsDialog(self, entries).exec()

    def _format_document(self) -> None:
        st = self.tabs.active
        if st is None:
            return
        original = st.editor.get_text()
        formatted = format_source(original)
        if formatted == original:
            self.statusBar().showMessage("Bereits formatiert.", 2000)
            return
        # Cursor-Zeile merken und nach Format wieder anspringen.
        cur_line = st.editor.textCursor().blockNumber() + 1
        st.editor.set_text(formatted)
        st.editor.document().setModified(True)
        st.editor.goto_line(cur_line)
        self.statusBar().showMessage("Dokument formatiert.", 2000)

    def _show_quick_open(self) -> None:
        quick_open(self, self.project_root, self._open_file)

    def _show_command_palette(self) -> None:
        # Sammlung aller registrierten Aktionen + zusaetzliche Befehle.
        commands: list[tuple[str, str, callable]] = []
        for act in self._all_palette_actions():
            sc = act.shortcut().toString() if act.shortcut() else ""
            commands.append((act.text().replace("&", ""), sc, act.trigger))
        # Plus extra Helfer:
        commands.extend([
            ("Theme: Dark/Light umschalten", "Ctrl+T", self._toggle_theme),
            ("Block am Cursor falten", "Ctrl+Shift+[", self._fold_at_cursor),
            ("Alle Bloecke aufklappen", "Ctrl+Shift+]", self._unfold_all),
        ])
        # De-Dup (Action und Helper koennen kollidieren)
        seen = set()
        uniq = []
        for lbl, sc, cb in commands:
            key = lbl.lower()
            if key in seen:
                continue
            seen.add(key)
            uniq.append((lbl, sc, cb))
        command_palette(self, uniq)

    def _all_palette_actions(self):
        """Liefert alle QActions, die der Command-Palette gezeigt werden."""
        return [
            self.act_new, self.act_open, self.act_save, self.act_save_as,
            self.act_sprite_editor,
            self.act_close_tab, self.act_reopen_tab, self.act_quit,
            self.act_find, self.act_replace, self.act_find_in_project,
            self.act_goto, self.act_settings,
            self.act_run, self.act_run_native, self.act_stop, self.act_bench,
            self.act_view_files, self.act_view_outline, self.act_view_builtins,
            self.act_toggle_minimap, self.act_fold, self.act_unfold_all,
            self.act_theme, self.act_show_readme, self.act_about,
            self.act_quick_open, self.act_command_palette,
        ]

    def _show_readme(self) -> None:
        readme = self.project_root / "README.md"
        if not readme.exists():
            readme = self.project_root / "CLAUDE.md"
        self._show_markdown(readme)

    def _show_markdown(self, path: Path) -> None:
        """Oeffnet eine Markdown-Datei (Doku) gerendert im Markdown-Viewer."""
        if not hasattr(self, "_md_viewer") or self._md_viewer is None:
            self._md_viewer = MarkdownViewer(self.project_root, self)
            self._md_viewer.open_gb_file.connect(self._open_file)
        self._md_viewer.show_file(Path(path))
        self._md_viewer.show()
        self._md_viewer.raise_()

    def _activate_sidebar(self, key: str) -> None:
        # Wenn Sidebar versteckt, zuerst einblenden.
        if not self.sidebar_stack.isVisible():
            self.sidebar_stack.setVisible(True)
        self._show_sidebar_view(key)
        self.activity.set_active(key)

    def _show_sidebar_view(self, key: str) -> None:
        idx = {"files": 0, "outline": 1, "builtins": 2}.get(key, 0)
        self.sidebar_stack.setCurrentIndex(idx)
        if key == "outline":
            self._refresh_outline()

    def _set_sidebar_visible(self, visible: bool) -> None:
        self.sidebar_stack.setVisible(visible)

    # ----------------------------------------------------- Misc
    def _on_active_tab_changed(self, st: TabState | None) -> None:
        if st is None:
            self.setWindowTitle("GameBasic Editor")
            self.file_browser.mark_active(None)
            self._refresh_outline()
            self.statusBar().clearMessage()
            return
        name = st.file_path.name if st.file_path else "(neu)"
        self.setWindowTitle(f"{name} -- GameBasic Editor")
        self.file_browser.mark_active(st.file_path)
        self._update_status()
        self._refresh_outline()
        # Statusbar fuer den neuen aktiven Tab aktualisieren -- der vorige
        # Tab kann eine veraltete Error-Message hinterlassen haben.
        problem = st.editor.current_error()
        if problem is None:
            self.statusBar().clearMessage()
        else:
            self.statusBar().showMessage(
                f"Zeile {problem.line}: {problem.message}", 0,
            )

    def _update_status(self) -> None:
        self._update_cursor_label()

    def _update_cursor_label(self) -> None:
        st = self.tabs.active
        if st is None:
            self.cursor_label.setText("")
            return
        cursor = st.editor.textCursor()
        self.cursor_label.setText(
            f"Zeile {cursor.blockNumber() + 1}, Spalte {cursor.positionInBlock() + 1}"
        )

    def _add_to_recent(self, path: Path) -> None:
        s = str(path)
        if s in self.recent:
            self.recent.remove(s)
        self.recent.insert(0, s)
        self.recent = self.recent[:RECENT_FILES_MAX]
        save_recent(self.recent)
        self._rebuild_recent_menu()

    def _rebuild_recent_menu(self) -> None:
        self.menu_recent.clear()
        # Bei doppelten Datei-Namen den uebergeordneten Ordner als Suffix
        # zeigen, damit sich `pong.gb` und `legacy/pong.gb` unterscheiden.
        paths = [Path(p) for p in self.recent]
        name_counts: dict[str, int] = {}
        for p in paths:
            name_counts[p.name] = name_counts.get(p.name, 0) + 1
        for p in paths:
            if name_counts.get(p.name, 0) > 1:
                # Suffix = Ordner-Name (project-relativ wenn moeglich)
                try:
                    rel_parent = p.parent.relative_to(self.project_root)
                    suffix = str(rel_parent).replace("\\", "/") or p.parent.name
                except ValueError:
                    suffix = p.parent.name
                label = f"{p.name}  —  {suffix}"
            else:
                label = p.name
            act = QAction(label, self)
            act.setToolTip(str(p))
            act.triggered.connect(lambda checked=False, _p=p: self._open_file(_p))
            self.menu_recent.addAction(act)
        if not self.recent:
            empty = QAction("(leer)", self)
            empty.setEnabled(False)
            self.menu_recent.addAction(empty)

    def _persist_fold_state(self, file_path: str, starts: list) -> None:
        """Speichert die eingeklappten Block-Zeilen einer Datei in den
        Settings. Leere Liste -> Eintrag entfernen, damit settings.json
        nicht mit Karteileichen vollwaechst.

        Cap: max 200 Files mit Fold-State (LRU-Verdraengung). Wer 201+
        Files mit Folds hat, ist eh kein typischer Nutzer; die Cap
        verhindert nur dass settings.json ins Megabyte-Land waechst."""
        store = self.settings.setdefault("fold_state", {})
        if starts:
            store[file_path] = list(starts)
        else:
            store.pop(file_path, None)
        # LRU-Cap: bei zu vielen Eintraegen die aeltesten 50 droppen.
        if len(store) > 200:
            for k in list(store)[:len(store) - 150]:
                del store[k]
        save_settings(self.settings)

    # ----------------------------------------------------- Autosave
    def _do_autosave(self) -> None:
        """Schreibt Snapshots aller dirty Tabs in den Autosave-Ordner."""
        try:
            self._do_autosave_impl()
        except Exception:
            # Autosave darf den Editor nie crashen lassen.
            pass

    def _do_autosave_impl(self) -> None:
        folder = autosave_dir()
        entries: list[RecoveryEntry] = []
        for idx, st in enumerate(self.tabs.states):
            if not self.tabs.is_dirty(st):
                continue
            slug = slug_for(st.file_path, idx)
            content = st.editor.get_text()
            try:
                (folder / slug).write_text(content, encoding="utf-8")
            except OSError:
                continue
            entries.append(RecoveryEntry(
                autosave_file=slug,
                original_path=str(st.file_path) if st.file_path else None,
                label=(st.file_path.name if st.file_path else "(neu)"),
            ))
        write_manifest(entries)

    def _maybe_recover_autosaves(self) -> bool:
        """Wenn Snapshots vorliegen, User-Dialog zeigen und auswaehlen.

        Returns True, wenn mindestens ein Tab wiederhergestellt wurde --
        in dem Fall ueberspringen wir das normale Workspace-Restore (die
        Snapshots sind frischer als der gesicherte Workspace-State).
        """
        entries = read_manifest()
        if not entries:
            return False
        dlg = RecoveryDialog(self, entries)
        if dlg.exec() != dlg.DialogCode.Accepted:
            clear_autosaves()
            return False
        any_loaded = False
        for entry in dlg.selected:
            try:
                content = entry.autosave_full_path.read_text(encoding="utf-8")
            except OSError:
                continue
            path: Path | None = None
            if entry.original_path:
                cand = Path(entry.original_path)
                if cand.exists():
                    path = cand.resolve()
            st = self.tabs.open_tab(file_path=path, content=content)
            # Tab als modifiziert markieren -- der Snapshot weicht
            # potentiell vom Original-Datei-Inhalt ab, also ist der User
            # sich seiner Aenderungen bewusst.
            st.editor.document().setModified(True)
            any_loaded = True
        clear_autosaves()
        return any_loaded

    # ----------------------------------------------------- Drag&Drop
    def dragEnterEvent(self, ev: QDragEnterEvent):  # noqa: N802
        # Akzeptiere wenn die Drop-Daten URL(s) auf .gb-Dateien sind --
        # andere Files (.png, .pdf, ...) lehnen wir ab, sonst wirkt der
        # Cursor irrefuehrend "akzeptierend".
        md = ev.mimeData()
        if not md.hasUrls():
            return
        if any(u.toLocalFile().lower().endswith(".gb") for u in md.urls()):
            ev.acceptProposedAction()

    def dropEvent(self, ev: QDropEvent):  # noqa: N802
        for u in ev.mimeData().urls():
            local = u.toLocalFile()
            if local.lower().endswith(".gb"):
                p = Path(local)
                if p.exists():
                    self._open_file(p)
        ev.acceptProposedAction()

    # ----------------------------------------------------- Workspace
    def _snapshot_workspace(self) -> dict:
        """Sammelt offene Tabs + aktiver Tab fuer Persistenz."""
        files: list[str] = []
        active_path: str | None = None
        for st in self.tabs.states:
            if st.file_path is None:
                continue
            files.append(str(st.file_path))
            if self.tabs.active is st:
                active_path = str(st.file_path)
        return {"open_files": files, "active": active_path}

    def _restore_workspace(self) -> None:
        """Oeffnet beim Start die Tabs aus dem letzten Workspace.

        Tabs ohne Datei-Pfad werden nicht persistiert (nichts zu laden),
        nicht-mehr-existierende Pfade werden uebersprungen.
        """
        ws = self.settings.get("workspace") or {}
        if not isinstance(ws, dict):
            return
        files = ws.get("open_files") or []
        active = ws.get("active")
        if not isinstance(files, list):
            return
        opened: list[Path] = []
        for entry in files:
            if not isinstance(entry, str):
                continue
            p = Path(entry)
            if not p.exists():
                continue
            self._open_file(p)
            opened.append(p.resolve())
        # Aktiven Tab restaurieren, falls vorhanden.
        if isinstance(active, str):
            ap = Path(active).resolve()
            st = self.tabs.find_tab_for(ap)
            if st is not None:
                self.tabs.activate(st)

    # ----------------------------------------------------- Close-Flow
    def closeEvent(self, event: QCloseEvent):  # noqa: N802
        for st in list(self.tabs.states):
            if self.tabs.is_dirty(st):
                self.tabs.activate(st)
                ans = self._confirm_save(st)
                if ans is None:
                    event.ignore()
                    return
                if ans is True and not self._save_tab(st):
                    event.ignore()
                    return
        # Geometrie + Splitter persistieren
        self.settings["window_w"] = int(self.size().width())
        self.settings["window_h"] = int(self.size().height())
        if self.splitter_main.sizes():
            self.settings["sidebar_width"] = int(self.splitter_main.sizes()[0])
        if len(self.splitter_right.sizes()) >= 2:
            self.settings["editor_height"] = int(self.splitter_right.sizes()[0])
            self.settings["console_height"] = int(self.splitter_right.sizes()[1])
        self.settings["sidebar_active"] = self.activity._active_key  # type: ignore[attr-defined]
        self.settings["sidebar_visible"] = self.sidebar_stack.isVisible()
        self.settings["workspace"] = self._snapshot_workspace()
        save_settings(self.settings)

        if self.console.is_running():
            self.console.stop_run()
        # Sauberer Close: keine Recovery beim naechsten Start noetig.
        self._autosave_timer.stop()
        clear_autosaves()
        event.accept()


# Hilfs-Funktion -- liefert den Namen des aktuellen Themes. Eigenes
# Modul-Level-Helper damit MainWindow nicht direkt auf den private-State
# in theme.py greift (active_theme() exists, aber wir nutzen es ueber
# diesen Wrapper, sodass unten _toggle_theme auf einen Blick lesbar ist).
def COLORS_active_name() -> str:  # noqa: N802 (sprechender Funktionsname)
    from .theme import active_theme
    return active_theme()
