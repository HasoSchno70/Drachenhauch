"""GameBasic-Editor (PySide6).

Aufruf via `drachenhauch.editor_qt.launch(project_root, initial_file)`.

Features:
- File-Browser, Tabs, Code-Editor mit Syntax-Highlighting, Zeilennummern,
  Snippets, Auto-Completion (Strg+Space), Hover-Tooltips, Auto-Indent.
- ActivityBar links: Files / Outline / Built-ins toggelbar.
- Run/Stop/Bench mit `QProcess`, Konsole mit stdin-Forwarding und
  klickbaren Datei:Zeile-Fehler-Links.
- Find/Replace (Ctrl+F/H), Goto-Line (Ctrl+G), Find-in-Project (Ctrl+Shift+F).
- Theme-Toggle Dark/Light (Ctrl+T), Settings-Dialog (Ctrl+,).
- Multi-Cursor via Strg+D, Minimap rechts vom Editor (toggelbar).
- Drag&Drop von .gb-Dateien, Workspace-Restore beim Start, Auto-Save +
  Crash-Recovery, Recent-Files, Markdown-Viewer fuer README (F1).
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Optional


__all__ = ["launch"]


_app_instance = None


def _ensure_app():
    """Liefert die `QApplication`-Singleton, erstellt sie falls noetig."""
    from PySide6.QtWidgets import QApplication

    from .theme import global_qss

    global _app_instance
    app = QApplication.instance()
    if app is None:
        app = QApplication(sys.argv)
        app.setStyle("Fusion")
        app.setStyleSheet(global_qss())
        _app_instance = app
    return app


def launch(project_root: Path, initial_file: Optional[Path] = None) -> int:
    """Startet den Editor; liefert App-Exit-Code."""
    from .main_window import GameBasicEditor

    app = _ensure_app()
    win = GameBasicEditor(Path(project_root), initial_file)
    # Immer maximiert starten (fuellt den Bildschirm, behaelt aber Fenster-
    # rahmen/Taskbar -- fuer einen Editor sinnvoller als echtes Vollbild).
    # Die gespeicherte Groesse bleibt die Groesse beim Entmaximieren.
    win.showMaximized()
    return app.exec()
