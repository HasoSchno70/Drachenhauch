"""Tests fuer die Signature-Help-Anbindung im CodeEditor (Debounce +
Popup-Sichtbarkeit). Reine `find_active_call`-Logik siehe
tests/test_signature_help.py."""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _editor(src: str = ""):
    from gamebasic.editor_qt.code_editor import CodeEditor
    ed = CodeEditor()
    ed.set_text(src)
    return ed


def test_cursor_move_debounces_signature_help():
    """Review-Fund: `_update_signature_help` lief vorher SYNCHRON bei jeder
    Cursor-Bewegung (inkl. jedem Tastendruck) -- inkl. eines Voll-Dokument-
    Scans im User-Funktions-Fallback. Jetzt debounced wie Color-Literal-/
    Fold-Scan: direkt nach der Cursor-Bewegung ist noch nichts passiert,
    erst nach Ablauf des Timers."""
    from PySide6.QtGui import QTextCursor

    ed = _editor("PRINT LINE(")
    assert ed._sig_help_timer.isActive()   # set_text() bewegt den Cursor
    ed._sig_help_timer.stop()
    ed._sig_popup.hide()

    cur = ed.textCursor()
    cur.movePosition(QTextCursor.MoveOperation.End)
    ed.setTextCursor(cur)
    assert ed._sig_help_timer.isActive()
    assert not ed._sig_popup.isVisible()   # noch nicht synchron aktualisiert

    ed._sig_help_timer.stop()
    ed._update_signature_help()            # simuliert den Timer-Ablauf
    assert ed._sig_popup.isVisible()


def test_escape_stops_pending_debounce():
    from PySide6.QtCore import Qt
    from PySide6.QtGui import QKeyEvent

    ed = _editor("PRINT LINE(")
    assert ed._sig_help_timer.isActive()

    ev = QKeyEvent(QKeyEvent.Type.KeyPress, Qt.Key.Key_Escape, Qt.KeyboardModifier.NoModifier)
    ed.keyPressEvent(ev)
    assert not ed._sig_help_timer.isActive()
    assert not ed._sig_popup.isVisible()


def test_focus_out_stops_pending_debounce():
    from PySide6.QtCore import Qt
    from PySide6.QtGui import QFocusEvent

    ed = _editor("PRINT LINE(")
    assert ed._sig_help_timer.isActive()

    ed.focusOutEvent(QFocusEvent(QFocusEvent.Type.FocusOut, Qt.FocusReason.OtherFocusReason))
    assert not ed._sig_help_timer.isActive()
    assert not ed._sig_popup.isVisible()
