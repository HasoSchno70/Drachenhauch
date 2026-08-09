"""Stufe B, Phase 3c: interaktiver Debugger `gbrt debug`.

Skriptet eine stdin-Kommandosession und prueft die stdout-Events
(paused/output/eval-result/finished) inkl. Variablen-Snapshot + Stepping.
Skippt, wenn gbrt nicht gebaut ist.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    base = _ROOT / "rust" / "drachenhauch_runtime" / "target"
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


_GBRT = _find_gbrt()
pytestmark = pytest.mark.skipif(_GBRT is None, reason="gbrt nicht gebaut")


def _debug_session(tmp_path, src, cmds):
    """Schreibt src, fuettert cmds (Liste von dicts) als stdin-Zeilen an
    `gbrt debug`, gibt die stdout-Events (Liste von dicts) zurueck."""
    f = tmp_path / "d.gb"
    f.write_text(src, encoding="utf-8")
    stdin = "".join(json.dumps(c) + "\n" for c in cmds)
    r = subprocess.run([str(_GBRT), "debug", str(f)], input=stdin,
                       capture_output=True, text=True, timeout=30)
    return [json.loads(line) for line in r.stdout.splitlines() if line.strip()]


def test_breakpoint_globals_eval_stop(tmp_path):
    src = 'DIM x AS INTEGER\nx = 5\nPRINT x\n'
    evs = _debug_session(tmp_path, src, [
        {"cmd": "set-breakpoints", "lines": [3]},
        {"cmd": "continue"},
        {"cmd": "eval", "expr": "x * 2"},
        {"cmd": "stop"},
    ])
    kinds = [e["event"] for e in evs]
    assert kinds[0] == "paused" and evs[0]["line"] == 1      # Initial-Pause
    paused3 = [e for e in evs if e["event"] == "paused" and e["line"] == 3]
    assert paused3, "kein Pause am Breakpoint (Zeile 3)"
    glob = {g["name"]: g["value"] for g in paused3[0]["globals"]}
    assert glob.get("x") == "5"
    ev = next(e for e in evs if e["event"] == "eval-result")
    assert ev["value"] == "10" and ev["type"] == "INTEGER"
    assert evs[-1] == {"event": "finished", "reason": "stopped"}


def test_function_locals_by_name(tmp_path):
    src = ('FUNCTION f(a AS INTEGER) AS INTEGER\n'
           '  DIM b AS INTEGER\n'
           '  b = a + 1\n'
           '  RETURN b\n'
           'END FUNCTION\n'
           'DIM r AS INTEGER\n'
           'r = f(10)\n'
           'PRINT r\n')
    evs = _debug_session(tmp_path, src, [
        {"cmd": "set-breakpoints", "lines": [4]},   # RETURN b -> b schon gesetzt
        {"cmd": "continue"},
        {"cmd": "eval", "expr": "a + b"},
        {"cmd": "continue"},
    ])
    p4 = [e for e in evs if e["event"] == "paused" and e["line"] == 4]
    assert p4, "kein Pause in der Funktion (Zeile 4)"
    assert p4[0]["depth"] == 2                       # in der Funktion -> Tiefe 2
    loc = {l["name"]: l["value"] for l in p4[0]["locals"]}
    assert loc.get("a") == "10" and loc.get("b") == "11"
    ev = next(e for e in evs if e["event"] == "eval-result")
    assert ev["value"] == "21"                       # a(10) + b(11)
    assert evs[-1]["event"] == "finished"


def test_step_over_advances_lines(tmp_path):
    src = 'DIM x AS INTEGER\nx = 1\nx = 2\nx = 3\n'
    evs = _debug_session(tmp_path, src, [
        {"cmd": "step-over"},   # Initial-Pause (Z.1) -> Z.2
        {"cmd": "step-over"},   # Z.2 -> Z.3
        {"cmd": "stop"},
    ])
    lines = [e["line"] for e in evs if e["event"] == "paused"]
    assert lines[:3] == [1, 2, 3]
