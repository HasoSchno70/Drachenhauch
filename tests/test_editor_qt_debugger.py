"""Tests fuer den Editor-Debugger (`DebugController`) -- Stufe B: laeuft jetzt
ueber einen `gbrt debug`-Subprozess. Getestet end-to-end ueber die Qt-Signale
(Reader-Thread -> queued signals, via processEvents zugestellt). Skippt ohne
gbrt bzw. ohne PySide6.
"""
import os
import time

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

pytest.importorskip("PySide6")
from gamebasic.editor_qt.debugger import DebugController, _find_gbrt

pytestmark = pytest.mark.skipif(_find_gbrt() is None, reason="gbrt nicht gebaut")


@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


def _pump(app, pred, timeout=10.0):
    """Qt-Eventloop pumpen, bis pred() True ist (oder Timeout)."""
    end = time.time() + timeout
    while time.time() < end:
        app.processEvents()
        if pred():
            return True
        time.sleep(0.01)
    app.processEvents()
    return pred()


def _controller():
    dc = DebugController()
    ev = {"paused": [], "finished": [], "output": [], "failed": []}
    dc.paused.connect(lambda line, fr: ev["paused"].append((line, fr)))
    dc.finished.connect(lambda r: ev["finished"].append(r))
    dc.output.connect(lambda t: ev["output"].append(t))
    dc.failed.connect(lambda m, l: ev["failed"].append((m, l)))
    return dc, ev


def test_breakpoint_pause_globals_and_continue(app, tmp_path):
    dc, ev = _controller()
    src = 'DIM x AS INTEGER\nx = 5\nPRINT x\n'
    dc.set_breakpoints([3])
    assert dc.start(src, str(tmp_path)) is True
    assert _pump(app, lambda: ev["paused"]), "kein paused-Event"
    line, fr = ev["paused"][0]
    assert line == 3
    glob = {g[0]: g[2] for g in fr["globals"]}
    assert glob.get("x") == "5"
    assert dc.is_paused()
    dc.cont()
    assert _pump(app, lambda: ev["finished"]), "kein finished-Event"
    assert ev["finished"][0] == "fertig"
    assert "".join(ev["output"]).strip() == "5"


def test_start_and_halt_then_step_into(app, tmp_path):
    dc, ev = _controller()
    src = 'DIM x AS INTEGER\nx = 1\nx = 2\n'
    dc.set_breakpoints([])                 # ohne BP -> an Zeile 1 anhalten
    assert dc.start(src, str(tmp_path)) is True
    assert _pump(app, lambda: ev["paused"])
    assert ev["paused"][0][0] == 1
    dc.step_into()
    assert _pump(app, lambda: len(ev["paused"]) >= 2)
    assert ev["paused"][1][0] == 2
    dc.stop()
    assert _pump(app, lambda: ev["finished"])


def test_compile_error_reports_failed_with_line(app, tmp_path):
    dc, ev = _controller()
    src = 'PRINT 1\nBREAK\n'               # BREAK ausserhalb Schleife
    dc.set_breakpoints([])
    assert dc.start(src, str(tmp_path)) is True
    assert _pump(app, lambda: ev["finished"]), "Sitzung endete nicht"
    assert ev["failed"], "kein failed-Event bei Compile-Fehler"
    msg, line = ev["failed"][0]
    assert line == 2 or ":2:" in msg


def test_stop_terminates_infinite_loop(app, tmp_path):
    # Review-Fund: stop() sendete frueher NUR das JSON-Kommando ohne
    # Fallback -- haengt/ignoriert gbrt das Stop-Signal, blieben Subprozess
    # + Reader-Thread unbegrenzt am Leben. Der Watchdog garantiert jetzt
    # eine obere Zeitschranke (~3s) bis zum harten Beenden.
    dc, ev = _controller()
    src = 'WHILE TRUE\nWEND\n'
    dc.set_breakpoints([])
    assert dc.start(src, str(tmp_path)) is True
    assert _pump(app, lambda: ev["paused"]), "Start & Halt an Zeile 1 ausgeblieben"
    dc.stop()
    assert _pump(app, lambda: ev["finished"], timeout=8.0), "stop() hat nie beendet"


def test_conditional_breakpoint(app, tmp_path):
    dc, ev = _controller()
    src = ('DIM i AS INTEGER\n'
           'FOR i = 1 TO 5\n'
           '    PRINT i\n'          # Zeile 3
           'NEXT\n')
    dc.set_breakpoints([3], conditions={3: "i = 3"})
    assert dc.start(src, str(tmp_path)) is True
    assert _pump(app, lambda: ev["paused"]), "bedingter BP hielt nicht"
    line, fr = ev["paused"][0]
    assert line == 3
    glob = {g[0]: g[2] for g in fr["globals"]}
    assert glob.get("i") == "3"          # nur bei i==3 angehalten
    dc.stop()
    assert _pump(app, lambda: ev["finished"])
