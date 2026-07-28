"""Tests fuer den Ressourcen-Cleanup beim Schliessen des GESAMTEN Fensters
(Review-Fund: closeEvent stoppte bisher nur einen laufenden Konsolen-Run --
ein aktiver Debugger/Profiler oder ein noch laufender Live-Error-Checker-
Subprozess pro Tab wurde verwaist zurueckgelassen, statt terminiert zu
werden. Der Einzel-Tab-Close-Pfad (_on_tab_close_requested/tabs.close_tab)
hatte das bereits korrekt -- dieser Test deckt den Fenster-weiten Pfad ab)."""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


@pytest.fixture
def win(tmp_path, monkeypatch):
    from gamebasic.editor_qt import main_window as mw
    # Reale Settings-/Autosave-Persistenz beim Test-closeEvent() vermeiden --
    # sonst wuerde ein Testlauf die echten %APPDATA%/GameBasic-Dateien des
    # Users ueberschreiben.
    monkeypatch.setattr(mw, "save_settings", lambda *_a, **_kw: None)
    monkeypatch.setattr(mw, "clear_autosaves", lambda *_a, **_kw: None)
    return mw.GameBasicEditor(tmp_path)


def _fake_close_event():
    from PySide6.QtGui import QCloseEvent
    return QCloseEvent()


def test_close_event_stops_active_debugger(win):
    win.debugger._mode = "running"
    stopped = []
    win.debugger.stop = lambda: stopped.append(1)
    win.debugger.is_active = lambda: True

    win.closeEvent(_fake_close_event())

    assert stopped == [1]


def test_close_event_stops_running_profiler(win):
    stopped = []
    win.profiler.is_running = lambda: True
    win.profiler.stop = lambda: stopped.append(1)

    win.closeEvent(_fake_close_event())

    assert stopped == [1]


def test_close_event_cancels_every_tab_live_error_checker(win):
    st1 = win._new_tab()
    st2 = win._new_tab()
    cancelled = []
    st1.editor._error_checker.cancel = lambda: cancelled.append(st1)
    st2.editor._error_checker.cancel = lambda: cancelled.append(st2)

    win.closeEvent(_fake_close_event())

    assert st1 in cancelled and st2 in cancelled


def test_close_event_noop_when_nothing_active(win):
    dbg_stopped = []
    prof_stopped = []
    win.debugger.stop = lambda: dbg_stopped.append(1)
    win.profiler.stop = lambda: prof_stopped.append(1)

    win.closeEvent(_fake_close_event())

    assert dbg_stopped == []
    assert prof_stopped == []
