"""Tests fuer den Debug-Session-Cleanup beim Schliessen des gerade
debuggten Tabs (Review-Fund: `_debug_editor` hielt sonst eine dangling
Referenz auf den entfernten Editor)."""
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
    from drachenhauch.editor_qt.main_window import GameBasicEditor
    return GameBasicEditor(tmp_path)


def test_closing_actively_debugged_tab_stops_session_and_clears_reference(win):
    st = win._new_tab()
    win._debug_editor = st.editor
    win.debugger._mode = "running"    # aktive Sitzung simulieren, ohne echten Subprozess
    stopped = []
    win.debugger.stop = lambda: stopped.append(1)

    win._on_tab_close_requested(st)

    assert stopped == [1]
    assert win._debug_editor is None


def test_closing_other_tab_does_not_touch_debug_session(win):
    st_debugged = win._new_tab()
    st_other = win._new_tab()
    win._debug_editor = st_debugged.editor
    win.debugger._mode = "running"
    stopped = []
    win.debugger.stop = lambda: stopped.append(1)

    win._on_tab_close_requested(st_other)

    assert stopped == []
    assert win._debug_editor is st_debugged.editor


def test_closing_tab_without_active_debug_session_is_noop(win):
    st = win._new_tab()
    win._debug_editor = None
    stopped = []
    win.debugger.stop = lambda: stopped.append(1)

    win._on_tab_close_requested(st)

    assert stopped == []
    assert win._debug_editor is None
