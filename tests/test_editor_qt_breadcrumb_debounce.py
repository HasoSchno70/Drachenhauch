"""Tests fuer die Debounce-Anbindung der Breadcrumb-Leiste im MainWindow.

Review-Fund: `cursorPositionChanged` loeste vorher SYNCHRON einen vollen
Dokument-Scan (`scope_path`/`scan_scopes`) aus -- bei jeder Cursor-Bewegung,
nicht nur beim Tippen. Jetzt debounced wie Color-Literal-/Fold-Scan.
"""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


@pytest.fixture
def win(tmp_path):
    from gamebasic.editor_qt.main_window import GameBasicEditor
    return GameBasicEditor(tmp_path)


SRC = "SUB foo()\n    PRINT 1\nEND SUB\n"


def test_cursor_move_schedules_debounced_breadcrumb_refresh(win):
    from PySide6.QtGui import QTextCursor

    st = win._new_tab()
    st.editor.set_text(SRC)
    win._breadcrumb_timer.stop()
    win.breadcrumbs.clear()

    cur = QTextCursor(st.editor.document().findBlockByNumber(1))   # Zeile 2, in foo()
    st.editor.setTextCursor(cur)

    assert win._breadcrumb_timer.isActive()
    # Direkt nach der Cursor-Bewegung noch NICHT aktualisiert (waere vorher
    # synchron passiert).
    assert win.breadcrumbs._scopes == []


def test_breadcrumb_updates_once_debounce_fires(win):
    from PySide6.QtGui import QTextCursor

    st = win._new_tab()
    st.editor.set_text(SRC)

    cur = QTextCursor(st.editor.document().findBlockByNumber(1))
    st.editor.setTextCursor(cur)
    win._breadcrumb_timer.stop()
    win._update_breadcrumbs()               # simuliert den Timer-Ablauf

    assert [s.name for s in win.breadcrumbs._scopes] == ["foo"]
