"""Eingabe-Feinschliff + HiDPI (Etappe 6 des raylib-Ausbaus).

Drei Luecken, die ein "richtiges" Spiel frueher oder spaeter braucht:
Belegungsdialoge (Tastenname + zuletzt gedrueckte Taste), horizontales
Mausrad, und die Bildschirm-Skalierung fuer HiDPI. Dazu die Tastencodes fuer
Umschalt/Strg/Alt, Navigationsblock und Ziffernblock -- fuer die gab es bis
hierher UEBERHAUPT keine Konstante, "Sprint mit Shift" war nicht abfragbar.

Tastendruecke selbst lassen sich headless nicht erzeugen; geprueft wird
deshalb, was ohne Eingabe belegbar ist: die Namensgebung, der Ruhewert und
dass jede neue Konstante beim Abfragen auch wirklich auf eine Taste
abgebildet wird (ein unbekannter Code liefert stumm FALSE -- genau das soll
hier NICHT passieren).
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    return next((_ROOT / "rust" / "gb_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "gb_runtime" / "target" / v / exe).exists()), None)


_GBRT = _find_gbrt()
pytestmark = pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")


def _run(src, tmp_path, frames=1):
    (tmp_path / "i.gb").write_text(src, encoding="utf-8")
    env = dict(os.environ, GBRT_FRAMES=str(frames))
    r = subprocess.run([str(_GBRT), "run", str(tmp_path / "i.gb")], capture_output=True,
                       text=True, encoding="utf-8", env=env, timeout=90, cwd=str(tmp_path))
    r.lines = [ln for ln in (r.stdout or "").splitlines()
               if not ln.startswith(("WARNING:", "INFO:", "TRACE:"))]
    return r


_HEAD = 'SCREEN(160, 120, "In", 1)\n'


def _prints(exprs, tmp_path):
    return _run(_HEAD + "".join(f"PRINT {e}\n" for e in exprs), tmp_path)


# ------------------------------------------------------------- Tastennamen
def test_special_keys_have_readable_names(tmp_path):
    # GLFW liefert fuer Sondertasten NICHTS -- ohne eigene Tabelle staende im
    # Belegungsdialog eine leere Zeile.
    r = _prints(["KEY_NAME$(KEY_SPACE)", "KEY_NAME$(KEY_LEFT)", "KEY_NAME$(KEY_F5)",
                 "KEY_NAME$(KEY_LSHIFT)", "KEY_NAME$(KEY_KP_ENTER)"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["LEER", "LINKS", "F5", "UMSCHALT", "ENTER"]


def test_printable_key_uses_the_keyboard_layout(tmp_path):
    # A liegt auf QWERTZ wie auf QWERTY an derselben Stelle -- der Name kommt
    # hier wirklich von GLFW, nicht aus der Ersatztabelle.
    r = _prints(["KEY_NAME$(KEY_A)"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["A"]


def test_unknown_code_yields_an_empty_name(tmp_path):
    # Gamepad-Bind-Codes sind negativ und keine Tasten.
    r = _prints(['"[" + KEY_NAME$(JOY_BUTTON_A) + "]"', '"[" + KEY_NAME$(999999) + "]"'], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["[]", "[]"]


def test_key_any_hit_is_idle_without_input(tmp_path):
    r = _prints(["KEY_ANY_HIT()"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["-1"]


# ------------------------------------------------- neue Tastencodes greifen
_NEW_KEYS = ["KEY_LSHIFT", "KEY_RSHIFT", "KEY_LCTRL", "KEY_RCTRL", "KEY_LALT", "KEY_RALT",
             "KEY_LSUPER", "KEY_RSUPER", "KEY_CAPSLOCK", "KEY_INSERT", "KEY_DELETE",
             "KEY_HOME", "KEY_END", "KEY_PAGEUP", "KEY_PAGEDOWN",
             "KEY_KP0", "KEY_KP1", "KEY_KP2", "KEY_KP3", "KEY_KP4", "KEY_KP5",
             "KEY_KP6", "KEY_KP7", "KEY_KP8", "KEY_KP9",
             "KEY_KP_ENTER", "KEY_KP_PLUS", "KEY_KP_MINUS", "KEY_KP_MULTIPLY",
             "KEY_KP_DIVIDE", "KEY_KP_PERIOD"]


def test_every_new_key_constant_maps_to_a_real_key(tmp_path):
    # Ein Code, den die Runtime nicht kennt, faellt still auf FALSE zurueck --
    # die Konstante waere dann wertlos. KEY_NAME$ liefert nur fuer wirklich
    # zugeordnete Tasten etwas, ist hier also der Nachweis.
    r = _prints([f'"{k}=" + KEY_NAME$({k})' for k in _NEW_KEYS], tmp_path)
    assert r.returncode == 0, r.stderr
    leer = [ln for ln in r.lines if ln.endswith("=")]
    assert leer == [], f"nicht zugeordnete Tastencodes: {leer}"


def test_new_keys_are_queryable_and_idle(tmp_path):
    r = _prints(["KEYPRESSED(KEY_LSHIFT)", "KEYHIT(KEY_KP_ENTER)", "KEYRELEASED(KEY_PAGEUP)"],
                tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["FALSE", "FALSE", "FALSE"]


# ------------------------------------------------------------------ Mausrad
def test_mouse_wheel_has_both_axes_and_is_fractional(tmp_path):
    # MOUSEWHEEL liefert nur die vertikale Achse als ganze Zahl. Ohne Eingabe
    # ist hier alles 0 -- geprueft wird, dass die Achsen ueberhaupt da sind und
    # als FLOAT kommen (0.0 statt 0).
    r = _prints(["MOUSEWHEEL_X()", "MOUSEWHEEL_Y()"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["0.0", "0.0"]


def test_mouse_wheel_axes_work_without_a_window(tmp_path):
    # Wie MOUSEWHEEL: ohne SCREEN kein Absturz, sondern 0 (Konsolenprogramme).
    r = _run("PRINT MOUSEWHEEL_X()\nPRINT MOUSEWHEEL_Y()\n", tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["0.0", "0.0"]


# --------------------------------------------------------------- Gamepad-DB
def test_gamepad_mappings_accepts_an_sdl_line(tmp_path):
    line = ("030000005e0400008e02000014010000,Xbox 360 Controller,"
            "a:b0,b:b1,x:b2,y:b3,platform:Windows,")
    r = _prints([f'JOYSTICK_MAPPINGS("{line}")'], tmp_path)
    assert r.returncode == 0, r.stderr
    assert int(r.lines[0]) >= 0


def test_garbage_mappings_do_not_crash(tmp_path):
    r = _prints(['JOYSTICK_MAPPINGS("")', 'JOYSTICK_MAPPINGS("kein mapping")'], tmp_path)
    assert r.returncode == 0, r.stderr
    assert len(r.lines) == 2


# -------------------------------------------------------------------- HiDPI
def test_window_dpi_scale_is_positive(tmp_path):
    r = _prints(["WINDOW_DPI_X() > 0.0", "WINDOW_DPI_Y() > 0.0"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["TRUE", "TRUE"]
