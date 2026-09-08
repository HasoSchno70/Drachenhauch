"""Eingabe-Flanken, Maus-Delta/Cursor, Touch und Gesten (Etappe 1 des
raylib-Ausbaus).

Echte Eingaben lassen sich headless nicht erzeugen -- geprueft wird daher, dass
die Builtins existieren, die richtigen Typen mit neutralen Werten liefern
(niemand haengt sich auf, wenn kein Finger/Pad da ist) und dass der Compiler
sie kennt. Die Flanken-Logik selbst liegt vollstaendig in raylib.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/input_edges.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest


def _find_dhrt():
    root = Path(__file__).resolve().parent.parent
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((root / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (root / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _find_dhrt()

pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")


def _run(src: str, tmp_path, frames: int = 2) -> str:
    p = tmp_path / "t.dh"
    p.write_text(src, encoding="utf-8")
    env = dict(os.environ, DHRT_FRAMES=str(frames))
    r = subprocess.run([str(_DHRT), "run", str(p)], capture_output=True, text=True,
                       encoding="utf-8", env=env, timeout=60)
    assert r.returncode == 0, r.stderr
    return r.stdout


def _check(src: str, tmp_path) -> list:
    p = tmp_path / "c.dh"
    p.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "--check", str(p)], capture_output=True, text=True,
                       encoding="utf-8", timeout=60)
    return json.loads((r.stdout or "").strip() or "[]")


def test_edge_builtins_are_known_to_the_compiler(tmp_path):
    # `--check` warnt bei Builtins, die dhrt nicht kennt -- der eingebettete
    # builtin_index.json muss die neuen Namen also enthalten.
    src = ('SCREEN(64, 64, "T", 1)\n'
           'PRINT MOUSE_HIT(0); MOUSE_RELEASED(0); KEYHIT(32); KEYRELEASED(32)\n'
           'PRINT KEYREPEAT(32); MOUSE_ON_SCREEN(); MOUSE_DELTA_X(); MOUSE_DELTA_Y()\n'
           'PRINT TOUCH_COUNT(); TOUCH_X(0); TOUCH_Y(0); TOUCH_ID(0)\n'
           'PRINT GESTURE$(); GESTURE_DRAG_X(); GESTURE_DRAG_Y(); GESTURE_DRAG_ANGLE()\n'
           'PRINT GESTURE_PINCH_X(); GESTURE_PINCH_Y(); GESTURE_PINCH_ANGLE()\n'
           'PRINT GESTURE_HOLD_TIME(); JOYSTICK_ANY_BUTTON()\n'
           'MOUSE_CURSOR("hand")\nMOUSE_SET_POS(1, 1)\n')
    assert _check(src, tmp_path) == []
