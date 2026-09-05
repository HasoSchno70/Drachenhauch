"""gui-Ausbau Punkt 3: Listen -- Eintraege einzeln, Symbole, Farben,
Mehrfachauswahl (Strg/Umschalt), Kaestchen, Doppelklick.

Klicks werden ECHT eingespeist (Automation-Wiedergabe). Widget-Koordinaten
sind fenster-relativ, die Maus spricht Bildschirm -- der Versatz kommt aus
einer Zeichenflaeche bei (0, 0), die beides liefert (dasselbe Muster wie in
den Piloten-Tests; die Titelhoehe zu RATEN setzt jeden Klick daneben).

Braucht ein Fenster, steht darum in `conftest._BRAUCHT_GRAFIK`; speist
Eingabe ein, darum seriell.
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = [pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut"),
              pytest.mark.seriell]

KEY_UP, KEY_DOWN = 1, 2
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7
RL_LCTRL, RL_LSHIFT, RL_SPACE, RL_DOWN = 341, 340, 32, 264
ZEILE = 22            # DROPDOWN_ITEM_H

_KOPF = ('IMPORT "gui"\n'
         'SCREEN(400, 300, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 10, 10, 360, 260)\n'
         'DIM nullpunkt AS GUI_WIDGET : nullpunkt = GUI_CANVAS(w, 0, 0, 1, 1)\n'
         'DIM lb AS GUI_WIDGET\n'
         'lb = GUI_LISTBOX(w, 20, 20, 200, 120, SPLIT$("Apfel|Birne|Kirsche|Dattel", "|"))\n')
# Die Liste liegt bei (20, 20) im Fenster: Zeile k hat ihre Mitte bei
# 20 + k * ZEILE + 11; ein Klick auf den Text sitzt bei x = 20 + 80.
_OFFSET = '    PRINT "O " + STR$(GUI_CANVAS_X(nullpunkt)) + " " + STR$(GUI_CANVAS_Y(nullpunkt))\n'


def _lauf(tmp_path, src, frames=12, events=None):
    if events is not None:
        ev = sorted(events, key=lambda e: e[0])
        zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
        for frame, typ, *params in ev:
            p = (list(params) + [0, 0, 0, 0])[:4]
            zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
        (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
        src = src.replace('SET_WINDOW_POS(-3000, -3000)\n',
                          'SET_WINDOW_POS(-3000, -3000)\nAUTOMATION_PLAY("ev.txt")\n', 1)
    f = tmp_path / "t.dh"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return [ln.rstrip() for ln in r.stdout.splitlines()
            if ln.strip() and not ln.startswith("WARNING:")]


def _schleife(bilder, probe):
    return (f"DIM f AS INTEGER\nFOR f = 1 TO {bilder}\n    GUI_UPDATE()\n"
            f"{probe}    GUI_DRAW()\n    FLIP()\nNEXT\n")


def _versatz(tmp_path):
    out = _lauf(tmp_path, _KOPF + _schleife(2, _OFFSET), frames=3)
    ox, oy = [int(x) for x in out[-1].split()[1:]]
    return ox, oy


def _klick(frame, x, y, knopf=0, *halten):
    ev = [(frame, MOUSE_POSITION, x, y), (frame + 1, MOUSE_POSITION, x, y)]
    ev += [(frame + 1, KEY_DOWN, h) for h in halten]
    ev += [(frame + 1, MOUSE_BUTTON_DOWN, knopf), (frame + 2, MOUSE_BUTTON_UP, knopf)]
    ev += [(frame + 3, KEY_UP, h) for h in halten]
    return ev


def _zeile(ox, oy, k, x=100):
    return ox + 20 + x, oy + 20 + k * ZEILE + 11


# ---------------------------------------------------------------- Eintraege
def test_eintraege_einzeln_und_auswahl_rueckt_mit(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'GUI_LISTBOX_SET_SELECTED(lb, 2)\n'                 # Kirsche
                'PRINT GUI_LISTBOX_ADD(lb, "Ananas", 0)\n'           # vorn einfuegen
                'PRINT GUI_LISTBOX_TEXT(lb)\n'                      # Auswahl meint noch Kirsche
                'GUI_LISTBOX_REMOVE(lb, 0)\n'
                'PRINT GUI_LISTBOX_SELECTED(lb) ; " " ; GUI_LISTBOX_COUNT(lb)\n'
                'GUI_LISTBOX_MOVE(lb, 3, 0)\n'                       # Dattel nach vorn
                'PRINT GUI_LISTBOX_ITEM(lb, 0) ; " " ; GUI_LISTBOX_TEXT(lb)\n'
                'GUI_LISTBOX_SET_ITEM(lb, 0, "Feige")\n'
                'PRINT GUI_LISTBOX_ITEM(lb, 0)\n'
                'GUI_LISTBOX_REMOVE(lb, GUI_LISTBOX_SELECTED(lb))\n'
                'PRINT GUI_LISTBOX_SELECTED(lb)\n', frames=1)
    assert out == ["0", "Kirsche", "2 4", "Dattel Kirsche", "Feige", "-1"]


def test_auch_die_klappliste_kann_eintraege_einzeln(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM dd AS GUI_WIDGET : dd = GUI_DROPDOWN(w, 20, 160, 160, 24, SPLIT$("a|b", "|"))\n'
                'GUI_LISTBOX_ADD(dd, "c")\nGUI_LISTBOX_REMOVE(dd, 0)\n'
                'PRINT GUI_LISTBOX_COUNT(dd) ; " " ; GUI_LISTBOX_ITEM(dd, 1)\n'
                'TRY\n    GUI_LISTBOX_SET(dd, "kaestchen", 1)\nCATCH e\n    PRINT e\nEND TRY\n', frames=1)
    assert out[0] == "2 c"
    assert "GUI_LISTBOX" in out[1], "Kaestchen gibt es nur in der Liste"


def test_falscher_index_im_klartext(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'TRY\n    GUI_LISTBOX_ITEM(lb, 4)\nCATCH e\n    PRINT e\nEND TRY\n'
                'TRY\n    GUI_LISTBOX_SET(lb, "farbe", 1)\nCATCH e2\n    PRINT e2\nEND TRY\n', frames=1)
    assert "Eintrag 4 gibt es nicht" in out[0]
    assert "mehrfachauswahl, kaestchen" in out[1]


# ---------------------------------------------------------------- Mehrfachauswahl
def test_strg_klick_sammelt_umschalt_klick_spannt(tmp_path):
    ox, oy = _versatz(tmp_path)
    ev = (_klick(3, *_zeile(ox, oy, 0))
          + _klick(8, *_zeile(ox, oy, 2), 0, RL_LCTRL)
          + _klick(14, *_zeile(ox, oy, 3), 0, RL_LSHIFT))
    out = _lauf(tmp_path, _KOPF + 'GUI_LISTBOX_SET(lb, "mehrfachauswahl", 1)\n'
                + _schleife(20, '    IF f = 6 OR f = 12 OR f = 19 THEN PRINT STR$(GUI_LISTBOX_SEL_COUNT(lb)) + " " + '
                                'STR$(GUI_LISTBOX_SEL_ROW(lb, 0)) + " " + STR$(GUI_LISTBOX_SEL_ROW(lb, 1)) + " " + '
                                'STR$(GUI_LISTBOX_SELECTED(lb))\n'),
                frames=22, events=ev)
    assert out == ["1 0 -1 0",      # Klick: nur Apfel
                   "2 0 2 2",       # Strg+Klick: Apfel und Kirsche, zuletzt Kirsche
                   "2 2 3 3"]       # Umschalt+Klick: Bereich vom Anker (Kirsche) bis Dattel


def test_ohne_mehrfachauswahl_bleibt_es_eine_zeile(tmp_path):
    ox, oy = _versatz(tmp_path)
    ev = _klick(3, *_zeile(ox, oy, 0)) + _klick(8, *_zeile(ox, oy, 2), 0, RL_LCTRL)
    out = _lauf(tmp_path, _KOPF
                + _schleife(14, '    IF f = 12 THEN PRINT STR$(GUI_LISTBOX_SEL_COUNT(lb)) + " " + STR$(GUI_LISTBOX_SELECTED(lb))\n'),
                frames=16, events=ev)
    assert out == ["1 2"]


def test_pfeil_setzt_die_menge_auf_eine_zeile(tmp_path):
    ox, oy = _versatz(tmp_path)
    ev = _klick(3, *_zeile(ox, oy, 0)) + _klick(8, *_zeile(ox, oy, 2), 0, RL_LCTRL) + [(14, KEY_DOWN, RL_DOWN), (15, KEY_UP, RL_DOWN)]
    out = _lauf(tmp_path, _KOPF + 'GUI_LISTBOX_SET(lb, "mehrfachauswahl", 1)\n'
                + _schleife(20, '    IF f = 19 THEN PRINT STR$(GUI_LISTBOX_SEL_COUNT(lb)) + " " + STR$(GUI_LISTBOX_SEL_ROW(lb, 0))\n'),
                frames=22, events=ev)
    assert out == ["1 3"]


# ---------------------------------------------------------------- Kaestchen
def test_klick_aufs_kaestchen_kippt_nur_den_haken(tmp_path):
    ox, oy = _versatz(tmp_path)
    ev = _klick(3, *_zeile(ox, oy, 1, x=10)) + _klick(8, *_zeile(ox, oy, 3))
    out = _lauf(tmp_path, _KOPF + 'GUI_LISTBOX_SET(lb, "kaestchen", 1)\n'
                'SUB ge()\n    PRINT "change"\nEND SUB\nGUI_ON_CHANGE(lb, ge)\n'
                + _schleife(14, '    IF f = 12 THEN PRINT STR$(IIF(GUI_LISTBOX_CHECKED(lb, 1), 1, 0)) + " " + STR$(GUI_LISTBOX_SELECTED(lb))\n'),
                frames=16, events=ev)
    assert out.count("change") == 2, out
    assert out[-1] == "1 3", "Haken an Birne, Auswahl erst durch den zweiten Klick auf Dattel"


def test_leertaste_kippt_den_haken_der_gewaehlten_zeile(tmp_path):
    ev = [(4, KEY_DOWN, RL_SPACE), (5, KEY_UP, RL_SPACE), (9, KEY_DOWN, RL_SPACE), (10, KEY_UP, RL_SPACE)]
    out = _lauf(tmp_path, _KOPF + 'GUI_LISTBOX_SET(lb, "kaestchen", 1)\n'
                'GUI_LISTBOX_SET_SELECTED(lb, 2)\nGUI_FOCUS(lb)\n'
                # Die Aufnahme laeuft dem Programm um etwa zwei Bilder voraus:
                # der erste Druck kippt bei f = 2, der zweite bei f = 7.
                + _schleife(14, '    IF f = 5 OR f = 12 THEN PRINT GUI_LISTBOX_CHECKED(lb, 2)\n'),
                frames=16, events=ev)
    assert out == ["TRUE", "FALSE"]


# ---------------------------------------------------------------- Doppelklick
def test_doppelklick_meldet_sich_ein_bild_lang(tmp_path):
    ox, oy = _versatz(tmp_path)
    x, y = _zeile(ox, oy, 1)
    ev = [(3, MOUSE_POSITION, x, y), (4, MOUSE_POSITION, x, y), (4, MOUSE_BUTTON_DOWN, 0), (5, MOUSE_BUTTON_UP, 0),
          (7, MOUSE_BUTTON_DOWN, 0), (8, MOUSE_BUTTON_UP, 0)]
    out = _lauf(tmp_path, _KOPF
                + _schleife(14, '    IF GUI_DOUBLE_CLICKED(lb) THEN PRINT "doppel " + STR$(GUI_LISTBOX_SELECTED(lb))\n'),
                frames=16, events=ev)
    assert out == ["doppel 1"]


def test_doppelklick_nur_fuer_listen(tmp_path):
    out = _lauf(tmp_path, _KOPF + 'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, 20, 200, 60, 24)\n'
                'TRY\n    PRINT GUI_DOUBLE_CLICKED(b)\nCATCH e\n    PRINT e\nEND TRY\n'.replace(
                    'GUI_BUTTON(w, 20, 200, 60, 24)', 'GUI_BUTTON(w, "k", 20, 200, 60, 24)'), frames=1)
    assert "GUI_LISTBOX" in out[0]


# ---------------------------------------------------------------- Datei
def test_zusatz_ueberlebt_die_datei_und_fehlt_ohne_zusatz(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM j0 AS STRING : j0 = GUI_TO_JSON(w)\n'
                'PRINT INSTR(j0, CHR$(34) + "list" + CHR$(34) + ":") >= 0\n'
                'GUI_LISTBOX_SET(lb, "kaestchen", 1) : GUI_LISTBOX_SET_CHECKED(lb, 1, TRUE)\n'
                'GUI_LISTBOX_SET(lb, "mehrfachauswahl", 1) : GUI_LISTBOX_SELECT(lb, 3, TRUE)\n'
                'GUI_LISTBOX_COLOR(lb, 0, 255)\n'
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)\n'
                'PRINT GUI_TO_JSON(w2) = j\n'
                'DIM lb2 AS GUI_WIDGET : lb2 = GUI_WINDOW_WIDGET(w2, 1)\n'
                'PRINT GUI_LISTBOX_CHECKED(lb2, 1) ; " " ; GUI_LISTBOX_IS_SELECTED(lb2, 3) ; " " ; GUI_LISTBOX_SEL_COUNT(lb2)\n',
                frames=1)
    assert out == ["FALSE", "TRUE", "TRUE TRUE 1"]
