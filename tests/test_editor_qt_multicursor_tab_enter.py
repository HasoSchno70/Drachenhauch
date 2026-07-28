"""Tests fuer Multi-Cursor + Tab/Enter (Review-Fund: beide Tasten sind
KEINE "printable"-Zeichen (`"\\t".isprintable()`/`"\\r".isprintable()` sind
beide False), fielen im Multi-Cursor-Zweig von keyPressEvent daher bisher
durch bis zum "sonstige Taste -> Multi-Selektion verwerfen"-Fallback --
Tab/Enter mit aktiver Multi-Selektion loeschte die Selektion
stillschweigend und bearbeitete nur den primaeren Cursor."""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _editor(text=""):
    from gamebasic.editor_qt.tabs import TabbedEditorArea
    area = TabbedEditorArea()
    st = area.open_tab(file_path=None, content=text)
    return st.editor


def _key_event(key, text):
    from PySide6.QtCore import QEvent, Qt
    from PySide6.QtGui import QKeyEvent
    return QKeyEvent(QEvent.Type.KeyPress, key, Qt.KeyboardModifier.NoModifier, text)


def _select_word(editor, start, end):
    from PySide6.QtGui import QTextCursor
    c = QTextCursor(editor.document())
    c.setPosition(start)
    c.setPosition(end, QTextCursor.MoveMode.KeepAnchor)
    editor.setTextCursor(c)


def test_multicursor_tab_indents_all_cursors_not_just_primary():
    from PySide6.QtCore import Qt
    editor = _editor("foo\nfoo\n")
    _select_word(editor, 0, 3)                 # "foo" in Zeile 1 -- primaer
    editor._secondary = [(4, 7)]                # "foo" in Zeile 2 -- sekundaer
    editor._refresh_extra_selections()

    editor.keyPressEvent(_key_event(Qt.Key.Key_Tab, "\t"))

    # Multi-Selektion darf NICHT stillschweigend verworfen worden sein --
    # beide Vorkommen von "foo" muessen ersetzt/eingerueckt worden sein.
    text = editor.toPlainText()
    assert text.count("foo") == 0 or editor._secondary != [] or "    " in text
    # Genauer: beide Zeilen wurden bearbeitet (nicht nur die erste).
    lines = text.split("\n")
    assert lines[0] != "foo"
    assert lines[1] != "foo"


def test_multicursor_enter_inserts_newline_at_all_cursors():
    from PySide6.QtCore import Qt
    editor = _editor("ab\nab\n")
    _select_word(editor, 1, 1)                  # zwischen a|b in Zeile 1
    editor._secondary = [(4, 4)]                 # zwischen a|b in Zeile 2
    editor._refresh_extra_selections()

    editor.keyPressEvent(_key_event(Qt.Key.Key_Return, "\r"))

    text = editor.toPlainText()
    # Aus "ab\nab\n" (2 Zeilen) muessen nach 2x Newline-Insert 4 Zeilen werden.
    assert text.count("\n") == 4
    lines = text.split("\n")
    assert lines[0] == "a" and lines[1] == "b"
    assert lines[2] == "a" and lines[3] == "b"


def test_multicursor_cleared_by_arrow_key_unaffected():
    """Regressionsschutz: eine Taste, die WEDER Tab/Enter/Backspace/Delete/
    printable ist (z.B. Pfeiltaste), muss weiterhin die Multi-Selektion
    verwerfen -- das war schon vorher das gewuenschte Verhalten."""
    from PySide6.QtCore import Qt
    editor = _editor("foo\nfoo\n")
    _select_word(editor, 0, 3)
    editor._secondary = [(4, 7)]
    editor._refresh_extra_selections()

    editor.keyPressEvent(_key_event(Qt.Key.Key_Right, ""))

    assert editor._secondary == []
