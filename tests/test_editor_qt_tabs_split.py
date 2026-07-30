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


def test_split_editor_breakpoint_bookmark_fold_target_shared_document():
    """Review-Fund: der Split-Editor wird als `CodeEditor()` (eigenes
    temporaeres Default-Dokument) konstruiert und ERST danach per
    setDocument() auf das geteilte Dokument umgehaengt. Ohne ein
    setDocument()-Override zeigten die drei _LineTracker (Breakpoints/
    Bookmarks/Folds) weiterhin auf das verworfene temporaere Dokument --
    ein Klick im Split-View-Gutter war dadurch ein stiller No-Op."""
    area = _area()
    st = area.open_tab(file_path=None, content="PRINT 1\nPRINT 2\nPRINT 3\n")
    area.toggle_split(st)
    ed2 = st.split_editor

    assert ed2._breakpoints._document is ed2.document()
    assert ed2._bookmarks._document is ed2.document()
    assert ed2._folded._document is ed2.document()

    ed2.toggle_breakpoint(2)
    assert 2 in ed2._breakpoints.lines()


def test_theme_change_after_split_does_not_touch_dead_highlighter():
    """Review-Fund: der Split-Editor legt in `__init__` einen `GBHighlighter`
    auf seinem temporaeren Default-Dokument an. `setDocument()` gibt dieses
    Dokument her und zerstoert es -- der Highlighter stirbt mit, `_highlighter`
    zeigte danach auf ein totes C++-Objekt. Der naechste Theme-Wechsel lief
    deshalb in ein "Internal C++ object (GBHighlighter) already deleted"
    (im gemeinsamen Testlauf massenhaft in der Konsole zu sehen)."""
    import shiboken6
    area = _area()
    st = area.open_tab(file_path=None, content="PRINT 1")
    area.toggle_split(st)
    ed2 = st.split_editor

    # Der Highlighter des Split-Editors ist tatsaechlich weg ...
    assert ed2._highlighter is None
    # ... und der Primaer-Editor hat weiterhin seinen eigenen.
    assert shiboken6.isValid(st.editor._highlighter)

    # Theme-Wechsel darf nicht mehr werfen.
    ed2._on_theme_changed("dark")
    st.editor._on_theme_changed("dark")


def test_close_tab_cancels_live_error_check():
    """Review-Fund: close_tab() liess einen laufenden Live-Error-Check
    (Worker-Thread + `gbrt --check`-Subprozess) unangetastet weiterlaufen
    -- der haelt eine Referenz auf den Editor/sein Dokument bis zu 15s
    laenger als noetig am Leben."""
    area = _area()
    st = area.open_tab(file_path=None, content="PRINT 1")

    calls = []
    st.editor._error_checker.cancel = lambda: calls.append(1)
    area.close_tab(st)
    assert calls == [1]
