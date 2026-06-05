"""Tests fuer den Split-Editor-View (zweiter Editor, geteiltes Dokument)."""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _area():
    from gamebasic.editor_qt.tabs import TabbedEditorArea
    return TabbedEditorArea()


def test_toggle_split_creates_and_removes_secondary():
    area = _area()
    st = area.open_tab(file_path=None, content="PRINT 1")
    assert st.split_editor is None
    # An -> zweiter Editor existiert
    assert area.toggle_split(st) is True
    assert st.split_editor is not None
    # geteiltes Dokument
    assert st.split_editor.document() is st.editor.document()
    # Aus -> wieder weg
    assert area.toggle_split(st) is False
    assert st.split_editor is None


def test_split_shares_text_both_directions():
    area = _area()
    st = area.open_tab(file_path=None, content="A")
    area.toggle_split(st)
    # Edit im Primaer -> im Split sichtbar (gleiches Dokument)
    st.editor.set_text("HELLO")
    assert st.split_editor.toPlainText() == "HELLO"
    # Edit im Split -> im Primaer sichtbar
    st.split_editor.set_text("WORLD")
    assert st.editor.toPlainText() == "WORLD"


def test_toggle_split_no_active_is_safe():
    area = _area()
    # Ohne offenen Tab -> kein Crash, liefert False
    assert area.toggle_split(None) is False


def test_split_editor_in_splitter():
    area = _area()
    st = area.open_tab(file_path=None, content="X")
    area.toggle_split(st)
    # Beide Editoren haengen im selben Splitter
    assert st.editor_split.count() == 2
    area.toggle_split(st)
    assert st.editor_split.count() == 1
