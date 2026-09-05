"""gui-Ausbau Punkt 6: senkrechter Schieber, unbestimmter Fortschritt,
Bildmodi, Baumsymbole, rollbares Panel, Ziehen zwischen Widgets.

Maus wird ECHT eingespeist (Automation-Wiedergabe) -- Ziehen laesst sich
nicht anders pruefen. Der Versatz Fenster -> Bildschirm kommt aus einer
Zeichenflaeche bei (0, 0). Seriell, weil Eingabe eingespeist wird.

Cursorformen (GUI_CURSORS) sind hier NICHT geprueft: raylib kann die Form
setzen, aber nicht zuruecklesen. Was bleibt, ist die Gegenprobe im Bild.
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
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION, MOUSE_WHEEL = 5, 6, 7, 8
RL_UP = 265
ZEILE = 22            # DROPDOWN_ITEM_H

_KOPF = ('IMPORT "gui"\n'
         'SCREEN(500, 400, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 0, 0, 500, 400)\n'
         'GUI_WINDOW_CHROME(w, FALSE)\n'
         'DIM nullpunkt AS GUI_WIDGET : nullpunkt = GUI_CANVAS(w, 0, 0, 1, 1)\n')
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


def _zug(frame, x0, y0, x1, y1, schritte=4):
    """Druecken bei (x0,y0), in Schritten nach (x1,y1) fahren, loslassen."""
    ev = [(frame, MOUSE_POSITION, x0, y0), (frame + 1, MOUSE_BUTTON_DOWN, 0)]
    for k in range(1, schritte + 1):
        x = x0 + (x1 - x0) * k // schritte
        y = y0 + (y1 - y0) * k // schritte
        ev.append((frame + 1 + k, MOUSE_POSITION, x, y))
    ev.append((frame + 2 + schritte, MOUSE_BUTTON_UP, 0))
    return ev


# ---------------------------------------------------------------- Schieber
def test_senkrechter_schieber_waechst_nach_oben(tmp_path):
    ox, oy = _versatz(tmp_path)
    src = _KOPF + ('DIM s AS GUI_WIDGET : s = GUI_VSLIDER(w, 40, 20, 200, 0, 100, 0)\n'
                   'PRINT GUI_GET_W(s) ; " " ; GUI_GET_H(s)\n'
                   + _schleife(16, '    PRINT "V " + STR$(INT(GUI_VALUE(s)))\n'))
    # Von unten nach oben ziehen: unten ist 0, oben 100.
    ev = _zug(3, ox + 48, oy + 218, ox + 48, oy + 20)
    out = _lauf(tmp_path, src, frames=17, events=ev)
    b, h = [int(x) for x in out[0].split()]
    assert h == 200 and b < 40, "GUI_VSLIDER: h ist die Laenge, die Breite kommt vom Thema"
    werte = [int(l.split()[1]) for l in out if l.startswith("V ")]
    assert werte[-1] >= 98, werte
    assert werte == sorted(werte), "waehrend des Zugs nur gestiegen"


def test_pfeil_hoch_erhoeht_auch_senkrecht(tmp_path):
    src = _KOPF + ('DIM s AS GUI_WIDGET : s = GUI_VSLIDER(w, 40, 20, 200, 0, 100, 50)\n'
                   'GUI_FOCUS(s)\n' + _schleife(8, '    PRINT "V " + STR$(INT(GUI_VALUE(s)))\n'))
    ev = [(3, KEY_DOWN, RL_UP), (4, KEY_UP, RL_UP)]
    out = _lauf(tmp_path, src, frames=9, events=ev)
    werte = [int(l.split()[1]) for l in out if l.startswith("V ")]
    assert werte[0] == 50 and werte[-1] == 55, werte


# ---------------------------------------------------------------- Fortschritt, Bild, Baum
def test_fortschritt_unbestimmt_bild_und_baum_headless(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM p AS GUI_WIDGET : p = GUI_PROGRESS(w, 10, 10, 200, 20)\n'
                'GUI_PROGRESS_SET(p, "unbestimmt", 1)\n'
                'DIM b AS IMAGE : b = GENTEX_CHECKED(40, 20, 10, 10, RED, YELLOW)\n'
                'DIM i AS GUI_WIDGET : i = GUI_IMAGE(w, 10, 40, 100, 60, b)\n'
                'PRINT GUI_IMAGE_MODE_GET(i)\n'
                'GUI_IMAGE_MODE(i, "kacheln")\n'
                'PRINT GUI_IMAGE_MODE_GET(i)\n'
                'DIM t AS GUI_WIDGET : t = GUI_TREE(w, 10, 110, 200, 100)\n'
                'DIM n AS INTEGER : n = GUI_TREE_ADD(t, -1, "Wurzel")\n'
                'GUI_TREE_ICON(t, n, b) : GUI_TREE_COLOR(t, n, GREEN)\n'
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)\n'
                'PRINT GUI_TO_JSON(w2) = j ; " " ; INSTR(j, "indeterminate") >= 0 ; " " ; INSTR(j, "kacheln") >= 0\n'
                'TRY\n    GUI_IMAGE_MODE(i, "schief")\nCATCH e\n    PRINT e\nEND TRY\n'
                'TRY\n    GUI_TREE_ICON(t, 7, b)\nCATCH e2\n    PRINT e2\nEND TRY\n'
                'TRY\n    GUI_PROGRESS_SET(p, "farbe", 1)\nCATCH e3\n    PRINT e3\nEND TRY\n'
                'GUI_UPDATE() : GUI_DRAW() : FLIP()\nPRINT "gezeichnet"\n', frames=2)
    assert out[0] == "strecken" and out[1] == "kacheln"
    assert out[2] == "TRUE TRUE TRUE"
    assert "kein Modus" in out[3] and "Knoten-id" in out[4] and "unbestimmt" in out[5]
    assert out[6] == "gezeichnet"


# ---------------------------------------------------------------- Panel
_PANEL = ('DIM pn AS GUI_WIDGET : pn = GUI_PANEL(w, 20, 20, 200, 120, "")\n'
          'DIM k1 AS GUI_WIDGET : k1 = GUI_BUTTON(w, "Oben", 30, 30, 100, 26)\n'
          'DIM k2 AS GUI_WIDGET : k2 = GUI_BUTTON(w, "Unten", 30, 300, 100, 26)\n'
          'GUI_PANEL_ADD(pn, k1) : GUI_PANEL_ADD(pn, k2)\n')


def test_panel_rollt_und_klemmt(tmp_path):
    out = _lauf(tmp_path, _KOPF + _PANEL +
                'GUI_UPDATE()\n'
                'PRINT GUI_PANEL_SCROLL_GET(pn) ; " " ; GUI_HIT_TEST(40, 40) = k1 ; " " ; GUI_HIT_TEST(40, 310) = k2\n'
                'GUI_PANEL_SCROLL(pn, 5000)\nGUI_UPDATE()\n'
                'PRINT GUI_PANEL_SCROLL_GET(pn)\n'
                # Nach dem Rollen ist k1 herausgerollt (nicht zu treffen), k2 sichtbar.
                'PRINT GUI_HIT_TEST(40, 40) = k1 ; " " ; GUI_HIT_TEST(40, 300 + 13 - GUI_PANEL_SCROLL_GET(pn)) = k2\n'
                'PRINT GUI_GET_Y(k2)\n'
                'GUI_PANEL_REMOVE(pn, k2)\nGUI_UPDATE()\n'
                'PRINT GUI_PANEL_SCROLL_GET(pn) ; " " ; GUI_HIT_TEST(40, 310) = k2\n'
                'TRY\n    GUI_PANEL_ADD(pn, pn)\nCATCH e\n    PRINT e\nEND TRY\n', frames=1)
    assert out[0] == "0 TRUE FALSE", "unten liegt ausserhalb des Panels: nicht zu treffen"
    max_scroll = int(out[1])
    assert 0 < max_scroll < 300, "geklemmt auf Inhalt minus Sichtflaeche"
    assert out[2] == "FALSE TRUE"
    assert int(out[3]) == 300, "GUI_GET_Y bleibt die Lage im Fenster -- Rollen ist nur ein Blick-Versatz"
    assert out[4] == "0 TRUE", "ohne das tiefe Kind rollt nichts mehr, und der Knopf ist wieder frei"
    assert "selbst" in out[5]


def test_mausrad_ueber_dem_panel_rollt(tmp_path):
    ox, oy = _versatz(tmp_path)
    src = _KOPF + _PANEL + _schleife(10, '    PRINT "S " + STR$(GUI_PANEL_SCROLL_GET(pn))\n')
    ev = [(3, MOUSE_POSITION, ox + 100, oy + 100), (4, MOUSE_WHEEL, 0, -2), (5, MOUSE_WHEEL, 0, -2)]
    out = _lauf(tmp_path, src, frames=11, events=ev)
    werte = [int(l.split()[1]) for l in out if l.startswith("S ")]
    assert werte[0] == 0 and werte[-1] > 0, werte


# ---------------------------------------------------------------- Ziehen
_LISTEN = ('DIM a AS GUI_WIDGET : a = GUI_LISTBOX(w, 20, 20, 160, 110, ["Apfel", "Birne", "Kirsche"])\n'
           'DIM b AS GUI_WIDGET : b = GUI_LISTBOX(w, 240, 20, 160, 110, ["Dattel"])\n'
           'GUI_DRAGGABLE(a, TRUE) : GUI_DROP_TARGET(a, TRUE) : GUI_DROP_TARGET(b, TRUE)\n')
_PROBE = ('    IF GUI_DROPPED(b) THEN PRINT "B " + GUI_DROP_TEXT() + " " + STR$(GUI_DRAG_INDEX()) + " " + STR$(GUI_DROP_INDEX()) + " " + STR$(GUI_DROP_SOURCE() = a)\n'
          '    IF GUI_DROPPED(a) THEN PRINT "A " + GUI_DROP_TEXT() + " " + STR$(GUI_DRAG_INDEX()) + " " + STR$(GUI_DROP_INDEX())\n'
          '    IF GUI_DRAGGING() = a THEN PRINT "Z " + GUI_DROP_TEXT()\n'
          '    PRINT "L " + GUI_LISTBOX_ITEM(a, 0) + " " + GUI_LISTBOX_ITEM(a, 2)\n')


def test_ziehen_von_liste_zu_liste(tmp_path):
    ox, oy = _versatz(tmp_path)
    src = _KOPF + _LISTEN + _schleife(16, _PROBE)
    # Zeile 1 ("Birne") von a auf die erste Zeile von b ziehen.
    ev = _zug(3, ox + 60, oy + 20 + ZEILE + 11, ox + 300, oy + 20 + 11)
    out = _lauf(tmp_path, src, frames=17, events=ev)
    assert any(l.startswith("Z Birne") for l in out), "waehrend des Zugs meldet GUI_DRAGGING die Quelle"
    treffer = [l for l in out if l.startswith("B ")]
    assert treffer == ["B Birne 1 0 TRUE"], out
    # Die Quelle bleibt unveraendert -- was mit dem Eintrag geschieht,
    # entscheidet das Programm.
    assert out[-1] == "L Apfel Kirsche"


def test_ziehen_in_derselben_liste_sortiert_um(tmp_path):
    ox, oy = _versatz(tmp_path)
    src = _KOPF + _LISTEN + _schleife(16, _PROBE)
    # "Apfel" (Zeile 0) auf Zeile 2 ziehen -> Reihenfolge Birne, Kirsche, Apfel.
    ev = _zug(3, ox + 60, oy + 20 + 11, ox + 60, oy + 20 + 2 * ZEILE + 11)
    out = _lauf(tmp_path, src, frames=17, events=ev)
    assert [l for l in out if l.startswith("A ")] == ["A Apfel 0 2"]
    assert out[-1] == "L Birne Apfel"


def test_ein_klick_ohne_bewegung_ist_kein_zug(tmp_path):
    ox, oy = _versatz(tmp_path)
    src = _KOPF + _LISTEN + _schleife(12, _PROBE)
    ev = [(3, MOUSE_POSITION, ox + 60, oy + 31), (4, MOUSE_BUTTON_DOWN, 0), (6, MOUSE_BUTTON_UP, 0)]
    out = _lauf(tmp_path, src, frames=13, events=ev)
    assert not any(l.startswith(("Z ", "A ", "B ")) for l in out), out
    assert out[-1] == "L Apfel Kirsche"


def test_losgelassen_neben_einer_ablage_verpufft(tmp_path):
    ox, oy = _versatz(tmp_path)
    src = _KOPF + _LISTEN + _schleife(14, _PROBE)
    ev = _zug(3, ox + 60, oy + 31, ox + 200, oy + 300)
    out = _lauf(tmp_path, src, frames=15, events=ev)
    assert any(l.startswith("Z Apfel") for l in out)
    assert not any(l.startswith(("A ", "B ")) for l in out), out
    assert out[-1] == "L Apfel Kirsche"


def test_ziehen_ueberlebt_die_datei(tmp_path):
    out = _lauf(tmp_path, _KOPF + _LISTEN +
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)\n'
                'PRINT GUI_TO_JSON(w2) = j ; " " ; INSTR(j, "draggable") >= 0 ; " " ; INSTR(j, "drop_target") >= 0\n', frames=1)
    assert out == ["TRUE TRUE TRUE"]
