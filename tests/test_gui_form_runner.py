"""Form-Runner (Xojo-Stil): ein .gbform aus einer Datei laden und pruefen, dass
Struktur + Zustaende korrekt rekonstruiert werden. GUI_LOAD ist ein reines
Builtin (kein SCREEN) -> headless testbar.

Das Feuern der Handler galt hier lange als "braucht Maus/SCREEN, nur in der
Demo examples/105_form_runner.dh manuell verifiziert" -- und in genau dieser
Luecke sass ein Fehler, der JEDEN vom Form-Designer erzeugten Handler
unbrauchbar machte. Mit AUTOMATION_PLAY laesst sich ein Klick einspeisen,
also wird es unten geprueft.
"""
import json


_FORM = {
    "title": "Einstellungen", "x": 230, "y": 120, "w": 360, "h": 250,
    "movable": True, "closable": True, "visible": True,
    "widgets": [
        {"kind": "label", "x": 20, "y": 40, "w": 70, "h": 16, "text": "Name:"},
        {"kind": "textinput", "x": 95, "y": 36, "w": 225, "h": 26, "placeholder": "dein Name"},
        {"kind": "checkbox", "x": 20, "y": 78, "w": 16, "h": 16, "text": "Sound an",
         "checked": True, "on_change": "on_sound"},
        {"kind": "slider", "x": 120, "y": 112, "w": 200, "h": 14,
         "min": 0.0, "max": 100.0, "value": 70.0, "on_change": "on_volume"},
        {"kind": "dropdown", "x": 140, "y": 142, "w": 180, "h": 24,
         "items": ["Einfach", "Mittel", "Schwer"], "sel": 1, "on_change": "on_diff"},
        {"kind": "button", "x": 20, "y": 196, "w": 145, "h": 32, "text": "Speichern",
         "on_click": "on_save"},
    ],
}


def _write_form(tmp_path):
    (tmp_path / "f.gbform").write_text(json.dumps(_FORM), encoding="utf-8")


def test_form_loads_structure(run_gb, tmp_path):
    _write_form(tmp_path)
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM frm AS GUI_WINDOW\nfrm = GUI_LOAD("f.gbform")\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(frm)\n'
        'PRINT f"{GUI_WINDOW_GET_X(frm)},{GUI_WINDOW_GET_W(frm)}"\n'
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(frm, 1))\n'           # textinput
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(frm, 5))\n',          # button
        base=tmp_path)
    assert out.splitlines() == ["6", "230,360", "textinput", "button"]


def test_form_restores_states(run_gb, tmp_path):
    _write_form(tmp_path)
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM frm AS GUI_WINDOW\nfrm = GUI_LOAD("f.gbform")\n'
        'PRINT GUI_CHECKED(GUI_WINDOW_WIDGET(frm, 2))\n'        # TRUE
        'PRINT GUI_VALUE(GUI_WINDOW_WIDGET(frm, 3))\n'          # 70.0
        'PRINT GUI_DROPDOWN_TEXT(GUI_WINDOW_WIDGET(frm, 4))\n'  # Mittel (sel=1)
        'PRINT GUI_TEXT(GUI_WINDOW_WIDGET(frm, 1))\n',          # "" (Platzhalter, kein Text)
        base=tmp_path)
    assert out.splitlines() == ["TRUE", "70.0", "Mittel", ""]


# ------------------------------------------------------- Handler feuern lassen
# Das ging bisher als "braucht Maus/SCREEN, manuell verifiziert" durch -- und
# genau in dieser Luecke sass ein Fehler, der JEDEN vom Form-Designer erzeugten
# Handler betraf. Mit AUTOMATION_PLAY laesst sich ein Klick einspeisen, also
# wird es jetzt geprueft.

def _klick_aufnahme(tmp_path, name, x, y):
    """raylib-Aufnahmedatei: Maus hinsetzen, druecken, loslassen.
    Event-Typen wie in tests/test_automation.py (5=up, 6=down, 7=position)."""
    zeilen = ["# Klick", "c 4",
              f"e 0 7 {x} {y} 0 0 // hin",
              f"e 1 7 {x} {y} 0 0 // halten",
              "e 2 6 0 0 0 0 // druecken",
              "e 4 5 0 0 0 0 // loslassen"]
    (tmp_path / name).write_text("\n".join(zeilen) + "\n", encoding="utf-8")


def _form_mit_knopf(tmp_path, handler: str):
    form = {"title": "T", "x": 0, "y": 0, "w": 200, "h": 120,
            "chrome": False, "visible": True,
            "widgets": [{"kind": "button", "x": 20, "y": 20, "w": 100, "h": 28,
                         "text": "OK", "on_click": handler}]}
    (tmp_path / "f.gbform").write_text(json.dumps(form), encoding="utf-8")


import os                                    # noqa: E402
import subprocess                            # noqa: E402
import pytest                                # noqa: E402
from .conftest import _DHRT                  # noqa: E402


def _lauf(tmp_path, handler: str):
    if _DHRT is None:
        pytest.skip("native Runtime 'dhrt' nicht gebaut")
    _form_mit_knopf(tmp_path, handler)
    _klick_aufnahme(tmp_path, "klick.txt", 70, 34)
    (tmp_path / "a.dh").write_text(
        'IMPORT "gui"\n'
        'SCREEN(200, 120, "T", 1)\n'
        'DIM frm AS GUI_WINDOW\n'
        'frm = GUI_LOAD("f.gbform")\n'
        'GUI_WINDOW_CHROME(frm, FALSE)\n'
        'AUTOMATION_PLAY("klick.txt")\n'
        f'SUB {handler}()\n    PRINT "GEFEUERT"\nEND SUB\n'
        'WHILE NOT QUITREQUESTED()\n'
        '    GUI_UPDATE()\n    CLS(0)\n    GUI_DRAW()\n    FLIP()\n'
        'WEND\n', encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", "a.dh"], cwd=str(tmp_path),
                       capture_output=True, text=True, encoding="utf-8",
                       timeout=120, env=dict(os.environ, DHRT_FRAMES="10"))
    zeilen = [ln for ln in (r.stdout or "").splitlines()
              if not ln.startswith(("WARNING:", "INFO:", "TRACE:"))]
    return r, zeilen


def test_handler_aus_der_datei_feuert(tmp_path):
    r, zeilen = _lauf(tmp_path, "aufok")
    assert r.returncode == 0, r.stderr
    assert "GEFEUERT" in zeilen, zeilen


def test_handler_feuert_auch_mit_grossbuchstaben(tmp_path):
    """Der Compiler legt Funktionsnamen KLEIN ab, der Form-Designer schreibt
    aber `btn1Click` ins .gbform -- der Callback-Lookup fand den erzeugten
    `SUB btn1Click()` deshalb nie ("Funktion 'btn1Click' existiert nicht").
    Da JEDER automatisch erzeugte Handler einen Grossbuchstaben traegt, war
    damit kein einziger davon benutzbar."""
    r, zeilen = _lauf(tmp_path, "btn1Click")
    assert r.returncode == 0, r.stderr
    assert "GEFEUERT" in zeilen, zeilen
