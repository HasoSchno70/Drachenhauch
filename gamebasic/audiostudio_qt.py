"""GameBasic **Audio Studio** -- vereint Tracker, SFX-Generator und
Instrument-Bibliothek in EINEM fullscreen-Fenster (Reiter).

Die Einzeleditoren ([TrackerEditor](trackereditor_qt.py) /
[SfxGenerator](sfxeditor_qt.py)) bleiben eigenstaendig konstruierbar und
getestet -- das Studio bettet sie als Tab-Seiten ein (eine `QMainWindow`
ist selbst ein `QWidget`). So bleibt der ganze bestehende Funktionsumfang
(Instrument-Bibliothek, Keymap/SF2, WAV-Render, Undo) unveraendert
erhalten, nur eben unter einem Dach.

Start: `gbsound` / `gbrun.py --audio` (oder weiter `gbtracker`/`gbsfx` --
beide oeffnen jetzt das Studio auf dem passenden Tab). F11 schaltet echtes
Vollbild, Strg+1/2 wechselt die Tabs.
"""
from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import Qt
from PySide6.QtGui import QKeySequence, QShortcut
from PySide6.QtWidgets import QApplication, QMainWindow, QTabWidget

from .editor_qt.theme import global_qss
from .trackereditor_qt import TrackerEditor
from .sfxeditor_qt import SfxGenerator
from .tracker.song import Song

# Tab-Namen -> Index (fuer launch(..., tab=) und select_tab()).
_TAB_INDEX = {"tracker": 0, "music": 0, "sfx": 1, "sound": 1}


class AudioStudio(QMainWindow):
    """Vereintes Sound-Werkzeug: Tracker + SFX als Tabs in einem Fenster."""

    def __init__(self, project_root: Path, initial_file: Path | None = None):
        super().__init__()
        self.project_root = Path(project_root)
        self.setWindowTitle("GameBasic Audio Studio")
        self.resize(1280, 900)

        self.tabs = QTabWidget()
        self.tabs.setDocumentMode(True)
        self.tabs.setMovable(False)
        self.setCentralWidget(self.tabs)

        # Einzeleditoren als Tab-Seiten einbetten.
        self.tracker = TrackerEditor(self.project_root)
        self.sfx = SfxGenerator(self.project_root)
        self.tabs.addTab(self.tracker, "🎹  Tracker / Song")
        self.tabs.addTab(self.sfx, "💥  SFX-Generator")

        # Shortcut-Kollision aufloesen: beide Editoren binden Strg+Z/Y auf
        # ihr eigenes Undo. In einem gemeinsamen Top-Level-Fenster waeren das
        # mehrdeutige Shortcuts ("ambiguous shortcut overload") -- darum den
        # Kontext jedes eingebetteten Shortcuts auf den jeweiligen Tab-Teilbaum
        # beschraenken (feuert nur, wenn dieser Editor den Fokus hat).
        for ed in (self.tracker, self.sfx):
            for sc in ed.findChildren(QShortcut):
                sc.setContext(Qt.ShortcutContext.WidgetWithChildrenShortcut)

        # Datei vorab in den Tracker laden (wie trackereditor_qt.launch).
        if initial_file and Path(initial_file).exists():
            try:
                self.tracker.song = Song.load_json(str(initial_file))
                self.tracker.path = Path(initial_file)
                self.tracker.cur = 0
                self.tracker._reload_all()
                self.tracker.undo.reset()
                self.tracker.dirty = False
                self.tracker._update_title()
            except Exception:
                pass

        # Studio-weite Shortcuts (Kontext Top-Level-Fenster).
        QShortcut(QKeySequence("Ctrl+1"), self,
                  activated=lambda: self.tabs.setCurrentIndex(0))
        QShortcut(QKeySequence("Ctrl+2"), self,
                  activated=lambda: self.tabs.setCurrentIndex(1))
        QShortcut(QKeySequence("F11"), self, activated=self._toggle_fullscreen)

    def select_tab(self, name: str) -> None:
        """Tab per Name waehlen ('tracker'/'music' oder 'sfx'/'sound')."""
        self.tabs.setCurrentIndex(_TAB_INDEX.get((name or "").lower(), 0))

    def _toggle_fullscreen(self) -> None:
        if self.isFullScreen():
            self.showMaximized()
        else:
            self.showFullScreen()

    def closeEvent(self, event) -> None:  # noqa: N802
        # TrackerEditor.closeEvent greift hier NICHT -- als eingebettete
        # Tab-Seite (kein Top-Level-Fenster) feuert Qt keinen Close-Event auf
        # ihr, wenn der Vorfahr (dieses Studio-Fenster) geschlossen wird.
        # Also den Dirty-Check hier direkt aufrufen (SFX-Generator hat kein
        # Save/Load-Dateimodell -- nichts zu verlieren, keine Pruefung noetig).
        if self.tracker._confirm_dirty():
            self.tracker._mixer.stop()
            event.accept()
        else:
            event.ignore()


def launch(project_root: Path, initial_file: Path | None = None,
           tab: str = "tracker") -> int:
    app = QApplication.instance()
    if app is None:
        # Fusion-Style NUR bei frischer QApplication erzwingen -- siehe
        # trackereditor_qt.launch() fuer die volle Begruendung.
        app = QApplication([])
        app.setStyle("Fusion")
    app.setStyleSheet(global_qss())
    win = AudioStudio(project_root, initial_file)
    win.select_tab(tab)
    win.showMaximized()          # "ordentlich Platz" -- Vollbild-Fenster
    return app.exec()
