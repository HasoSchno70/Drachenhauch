"""Eingabe-Flanken, Maus-Delta/Cursor, Touch und Gesten (Etappe 1 des
raylib-Ausbaus).

Echte Eingaben lassen sich headless nicht erzeugen -- geprueft wird daher, dass
die Builtins existieren, die richtigen Typen mit neutralen Werten liefern
(niemand haengt sich auf, wenn kein Finger/Pad da ist) und dass der Compiler
sie kennt. Die Flanken-Logik selbst liegt vollstaendig in raylib.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest


def _find_gbrt():
    root = Path(__file__).resolve().parent.parent
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    return next((root / "rust" / "gb_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (root / "rust" / "gb_runtime" / "target" / v / exe).exists()), None)


_GBRT = _find_gbrt()

pytestmark = pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")


def _run(src: str, tmp_path, frames: int = 2) -> str:
    p = tmp_path / "t.gb"
    p.write_text(src, encoding="utf-8")
    env = dict(os.environ, GBRT_FRAMES=str(frames))
    r = subprocess.run([str(_GBRT), "run", str(p)], capture_output=True, text=True,
                       encoding="utf-8", env=env, timeout=60)
    assert r.returncode == 0, r.stderr
    return r.stdout


def _check(src: str, tmp_path) -> list:
    p = tmp_path / "c.gb"
    p.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_GBRT), "--check", str(p)], capture_output=True, text=True,
                       encoding="utf-8", timeout=60)
    return json.loads((r.stdout or "").strip() or "[]")


def test_edge_builtins_are_known_to_the_compiler(tmp_path):
    # `--check` warnt bei Builtins, die gbrt nicht kennt -- der eingebettete
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


def test_edges_are_false_without_input(tmp_path):
    out = _run('SCREEN(64, 64, "T", 1)\n'
               'PRINT MOUSE_HIT(0)\nPRINT MOUSE_RELEASED(0)\n'
               'PRINT KEYHIT(32)\nPRINT KEYRELEASED(32)\nPRINT KEYREPEAT(32)\n', tmp_path)
    assert out.split() == ["FALSE"] * 5


def test_held_and_edge_are_separate_builtins(tmp_path):
    # KEYPRESSED/MOUSEBUTTON bleiben "gehalten" -- die Namen sind historisch und
    # duerfen ihre Bedeutung nicht aendern, sonst brechen bestehende Programme.
    out = _run('SCREEN(64, 64, "T", 1)\n'
               'PRINT KEYPRESSED(32)\nPRINT MOUSEBUTTON(0)\n', tmp_path)
    assert out.split() == ["FALSE", "FALSE"]


def test_mouse_delta_and_position_are_floats(tmp_path):
    out = _run('SCREEN(64, 64, "T", 1)\n'
               'PRINT MOUSE_DELTA_X()\nPRINT MOUSE_DELTA_Y()\n', tmp_path)
    assert out.split() == ["0.0", "0.0"]           # FLOAT, nicht INTEGER


def test_mouse_cursor_accepts_known_shapes(tmp_path):
    shapes = ["default", "arrow", "ibeam", "text", "crosshair", "cross", "hand",
              "pointer", "resize_ew", "resize_ns", "resize_nwse", "resize_nesw",
              "resize_all", "move", "not_allowed", "no", "HAND"]
    src = 'SCREEN(64, 64, "T", 1)\n' + "".join(
        f'MOUSE_CURSOR("{s}")\n' for s in shapes) + 'PRINT "ok"\n'
    assert _run(src, tmp_path).strip() == "ok"


def test_mouse_cursor_rejects_unknown_shape(tmp_path):
    p = tmp_path / "bad.gb"
    p.write_text('SCREEN(64, 64, "T", 1)\nMOUSE_CURSOR("quatsch")\n', encoding="utf-8")
    r = subprocess.run([str(_GBRT), "run", str(p)], capture_output=True, text=True,
                       encoding="utf-8", env=dict(os.environ, GBRT_FRAMES="1"), timeout=60)
    assert r.returncode != 0
    assert "MOUSE_CURSOR" in r.stderr and "quatsch" in r.stderr


def test_touch_and_gestures_are_neutral_without_a_touchscreen(tmp_path):
    out = _run('SCREEN(64, 64, "T", 1)\n'
               'PRINT TOUCH_COUNT()\nPRINT "[" + GESTURE$() + "]"\n'
               'PRINT GESTURE_HOLD_TIME()\nPRINT GESTURE_PINCH_ANGLE()\n', tmp_path)
    assert out.split() == ["0", "[]", "0.0", "0.0"]


def test_joystick_any_button_reports_minus_one_when_idle(tmp_path):
    # raylib meldet UNKNOWN(0) wenn nichts anliegt -- als -1 durchgereicht,
    # damit 0 nicht faelschlich wie ein echter Knopf aussieht.
    out = _run('SCREEN(64, 64, "T", 1)\nPRINT JOYSTICK_ANY_BUTTON()\n', tmp_path)
    assert out.strip() == "-1"


def test_joystick_edges_reject_an_invalid_pad(tmp_path):
    p = tmp_path / "j.gb"
    p.write_text('SCREEN(64, 64, "T", 1)\nPRINT JOYSTICK_HIT(99, 7)\n', encoding="utf-8")
    r = subprocess.run([str(_GBRT), "run", str(p)], capture_output=True, text=True,
                       encoding="utf-8", env=dict(os.environ, GBRT_FRAMES="1"), timeout=60)
    assert r.returncode != 0 and "JOYSTICK_HIT" in r.stderr
