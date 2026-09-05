"""gui-Ausbau Punkt 5: Layout -- Automass und Layout-Behaelter.

Bis 2026-09-04 war jedes Widget absolut positioniert, mit Ankern fuer die
Groessenaenderung. Jetzt: `GUI_AUTOSIZE` (und 0 = automatisch beim Anlegen)
misst die Groesse am Inhalt, `GUI_LAYOUT` verteilt Kinder als Zeile, Spalte
oder Raster -- mit Gewichten fuer den Restplatz, Leerraeumen, Rand und
Abstand, beliebig geschachtelt.

Alles headless pruefbar: die Geometrie kommt aus GUI_GET_X/Y/W/H nach einem
GUI_UPDATE. Braucht ein Fenster (`_BRAUCHT_GRAFIK`), aber keine Eingabe.
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
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

# Randloses Fenster bei (0,0): Widget-Koordinaten sind dann zugleich
# Fenster-Koordinaten, ohne Titelhoehe im Kopf.
_KOPF = ('IMPORT "gui"\n'
         'SCREEN(500, 400, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 0, 0, 500, 400)\n'
         'GUI_WINDOW_CHROME(w, FALSE)\n')


def _lauf(tmp_path, src, frames=2):
    f = tmp_path / "t.dh"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=90,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return [ln.strip() for ln in (r.stdout or "").splitlines()
            if ln.strip() and not ln.startswith(("WARNING:", "INFO:"))]


def _geo(name):
    return f'PRINT GUI_GET_X({name}) ; " " ; GUI_GET_Y({name}) ; " " ; GUI_GET_W({name}) ; " " ; GUI_GET_H({name})\n'


def _zahlen(zeile):
    return [int(x) for x in zeile.split()]


# ---------------------------------------------------------------- Automass
def test_null_heisst_nach_inhalt(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM a AS GUI_WIDGET : a = GUI_BUTTON(w, "OK", 10, 10, 0, 0)\n'
                'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, "Ein deutlich laengerer Text", 10, 50, 0, 0)\n'
                'GUI_UPDATE()\n' + _geo("a") + _geo("b"))
    ax, ay, aw, ah = _zahlen(out[0])
    bx, by, bw, bh = _zahlen(out[1])
    assert (ax, ay) == (10, 10) and aw >= 28 and ah >= 26
    assert bw > aw * 3, "die Breite folgt dem Text"
    assert bh == ah


def test_autosize_misst_sofort_und_set_bounds_hebt_es_auf(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM a AS GUI_WIDGET : a = GUI_BUTTON(w, "Knopf mit Text", 10, 10, 30, 10)\n'
                'GUI_AUTOSIZE(a)\n' + _geo("a") +
                'GUI_SET_BOUNDS(a, 10, 10, 30, 10)\nGUI_UPDATE()\n' + _geo("a"))
    assert _zahlen(out[0])[2] > 60 and _zahlen(out[0])[3] >= 26
    assert _zahlen(out[1]) == [10, 10, 30, 10], "eine gesetzte Groesse bleibt"


def test_beschriftung_nach_inhalt(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM l AS GUI_WIDGET : l = GUI_LABEL(w, "Zwei" + CHR$(10) + "Zeilen", 10, 10)\n'
                'GUI_AUTOSIZE(l)\n' + _geo("l"))
    x, y, bw, bh = _zahlen(out[0])
    assert bh > 30, "zwei Zeilen hoch"


# ---------------------------------------------------------------- Zeile / Spalte
_ZEILE = ('DIM z AS GUI_WIDGET : z = GUI_LAYOUT(w, "zeile", 10, 10, 400, 40)\n'
          'GUI_LAYOUT_SET(z, "abstand", 0)\n'
          'DIM a AS GUI_WIDGET : a = GUI_BUTTON(w, "A", 0, 0, 40, 30)\n'
          'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, "B", 0, 0, 100, 30)\n'
          'DIM c AS GUI_WIDGET : c = GUI_BUTTON(w, "C", 0, 0, 50, 30)\n')


def test_zeile_gewichte_teilen_den_restplatz(tmp_path):
    out = _lauf(tmp_path, _KOPF + _ZEILE +
                'GUI_LAYOUT_ADD(z, a) : GUI_LAYOUT_ADD(z, b, 1) : GUI_LAYOUT_ADD(z, c, 3)\n'
                'GUI_UPDATE()\n' + _geo("a") + _geo("b") + _geo("c"))
    a, b, c = (_zahlen(o) for o in out)
    assert a == [10, 10, 40, 40], "Gewicht 0 = eigene Breite, quer gedehnt auf die Hoehe"
    assert b[0] == 50 and b[2] == 90, "1 von 4 Teilen des Rests (360)"
    assert c[0] == 140 and c[2] == 270, "3 von 4 Teilen, bis zum rechten Rand (410)"


def test_leerraum_schiebt_nach_rechts(tmp_path):
    out = _lauf(tmp_path, _KOPF + _ZEILE +
                'GUI_LAYOUT_ADD(z, a) : GUI_LAYOUT_SPACER(z) : GUI_LAYOUT_ADD(z, b)\n'
                'GUI_UPDATE()\n' + _geo("a") + _geo("b"))
    a, b = (_zahlen(o) for o in out)
    assert a[0] == 10 and b[0] == 310 and b[2] == 100, "B klebt am rechten Rand"


def test_abstand_rand_und_ausrichtung(tmp_path):
    out = _lauf(tmp_path, _KOPF + _ZEILE +
                'GUI_LAYOUT_SET(z, "abstand", 8) : GUI_LAYOUT_SET(z, "rand", 5)\n'
                'GUI_LAYOUT_SET(z, "dehnen", 0) : GUI_LAYOUT_SET(z, "ausrichtung", 1)\n'
                'GUI_LAYOUT_ADD(z, a) : GUI_LAYOUT_ADD(z, b)\n'
                'GUI_UPDATE()\n' + _geo("a") + _geo("b"))
    a, b = (_zahlen(o) for o in out)
    assert a == [15, 15, 40, 30], "Rand 5, quer zentriert in 30 von 30"
    assert b[0] == 15 + 40 + 8


def test_spalte_stapelt_und_dehnt_quer(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM s AS GUI_WIDGET : s = GUI_LAYOUT(w, "spalte", 20, 20, 200, 300)\n'
                'GUI_LAYOUT_SET(s, "abstand", 4)\n'
                'DIM l AS GUI_WIDGET : l = GUI_LABEL(w, "Name", 0, 0)\n'
                'DIM t AS GUI_WIDGET : t = GUI_TEXTINPUT(w, 0, 0, 0, 0)\n'
                'DIM f AS GUI_WIDGET : f = GUI_TEXTAREA(w, 0, 0, 50, 50)\n'
                'GUI_LAYOUT_ADD(s, l) : GUI_LAYOUT_ADD(s, t) : GUI_LAYOUT_ADD(s, f, 1)\n'
                'GUI_UPDATE()\n' + _geo("l") + _geo("t") + _geo("f"))
    l, t, f = (_zahlen(o) for o in out)
    assert l[0] == 20 and l[1] == 20 and l[2] == 200
    assert t[1] == 20 + l[3] + 4 and t[2] == 200 and t[3] >= 26
    assert f[1] == t[1] + t[3] + 4 and f[1] + f[3] == 320, "der Textbereich fuellt den Rest bis unten"


# ---------------------------------------------------------------- Raster + Schachtelung
def test_raster_verteilt_auf_spalten_und_zeilen(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM r AS GUI_WIDGET : r = GUI_LAYOUT(w, "raster:2", 0, 0, 200, 200)\n'
                'GUI_LAYOUT_SET(r, "abstand", 0)\n'
                'DIM k1 AS GUI_WIDGET : k1 = GUI_BUTTON(w, "1", 0, 0, 10, 20)\n'
                'DIM k2 AS GUI_WIDGET : k2 = GUI_BUTTON(w, "2", 0, 0, 10, 30)\n'
                'DIM k3 AS GUI_WIDGET : k3 = GUI_BUTTON(w, "3", 0, 0, 10, 20)\n'
                'GUI_LAYOUT_ADD(r, k1) : GUI_LAYOUT_ADD(r, k2) : GUI_LAYOUT_ADD(r, k3)\n'
                'GUI_UPDATE()\n' + _geo("k1") + _geo("k2") + _geo("k3"))
    k1, k2, k3 = (_zahlen(o) for o in out)
    assert k1 == [0, 0, 100, 20] and k2 == [100, 0, 100, 30], "zwei Spalten je 100 breit, gedehnt"
    assert k3 == [0, 30, 100, 20], "zweite Zeile beginnt unter der hoechsten Zelle der ersten"


def test_geschachtelt_und_anker_am_behaelter(tmp_path):
    """Eine Spalte mit einer Knopfzeile unten; der Behaelter klebt an allen
    Kanten -- nach GUI_WINDOW_SET_BOUNDS fliessen die Kinder mit."""
    out = _lauf(tmp_path, _KOPF +
                'DIM s AS GUI_WIDGET : s = GUI_LAYOUT(w, "spalte", 0, 0, 500, 400)\n'
                'GUI_SET_ANCHOR(s, "lrtb")\nGUI_LAYOUT_SET(s, "abstand", 0)\n'
                'DIM f AS GUI_WIDGET : f = GUI_TEXTAREA(w, 0, 0, 10, 10)\n'
                'DIM z AS GUI_WIDGET : z = GUI_LAYOUT(w, "zeile", 0, 0, 10, 30)\n'
                'GUI_LAYOUT_SET(z, "abstand", 0)\n'
                'DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(w, "OK", 0, 0, 80, 30)\n'
                'GUI_LAYOUT_SPACER(z) : GUI_LAYOUT_ADD(z, ok)\n'
                'GUI_LAYOUT_ADD(s, f, 1) : GUI_LAYOUT_ADD(s, z)\n'
                'GUI_UPDATE()\n' + _geo("ok") + _geo("f") +
                'GUI_WINDOW_SET_BOUNDS(w, 0, 0, 600, 300)\nGUI_UPDATE()\nGUI_UPDATE()\n' + _geo("ok") + _geo("f"))
    ok1, f1, ok2, f2 = (_zahlen(o) for o in out)
    assert ok1 == [420, 370, 80, 30] and f1 == [0, 0, 500, 370]
    assert ok2 == [520, 270, 80, 30] and f2 == [0, 0, 600, 270], "nach der Groessenaenderung neu verteilt"


# ---------------------------------------------------------------- Regeln
def test_ein_widget_steckt_in_hoechstens_einem_behaelter(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM z1 AS GUI_WIDGET : z1 = GUI_LAYOUT(w, "zeile", 0, 0, 100, 30)\n'
                'DIM z2 AS GUI_WIDGET : z2 = GUI_LAYOUT(w, "zeile", 0, 100, 100, 30)\n'
                'DIM a AS GUI_WIDGET : a = GUI_BUTTON(w, "A", 0, 0, 40, 30)\n'
                'GUI_LAYOUT_ADD(z1, a) : GUI_LAYOUT_ADD(z2, a)\n'
                'GUI_UPDATE()\n' + _geo("a"))
    assert _zahlen(out[0])[1] == 100, "der spaetere Behaelter gewinnt, der fruehere laesst los"


def test_kreise_und_fremde_fenster_sind_fehler(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM s AS GUI_WIDGET : s = GUI_LAYOUT(w, "spalte", 0, 0, 100, 100)\n'
                'DIM r AS GUI_WIDGET : r = GUI_LAYOUT(w, "raster:2", 0, 0, 10, 10)\n'
                'GUI_LAYOUT_ADD(s, r)\n'
                'TRY\n    GUI_LAYOUT_ADD(r, s)\nCATCH e\n    PRINT e\nEND TRY\n'
                'TRY\n    GUI_LAYOUT_ADD(s, s)\nCATCH e2\n    PRINT e2\nEND TRY\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_WINDOW("B", 10, 10, 100, 100)\n'
                'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w2, "x", 0, 0, 40, 20)\n'
                'TRY\n    GUI_LAYOUT_ADD(s, b)\nCATCH e3\n    PRINT e3\nEND TRY\n'
                'TRY\n    GUI_LAYOUT(w, "kreis", 0, 0, 10, 10)\nCATCH e4\n    PRINT e4\nEND TRY\n'
                'TRY\n    GUI_LAYOUT_SET(s, "farbe", 1)\nCATCH e5\n    PRINT e5\nEND TRY\n', frames=1)
    assert "schon steckt" in out[0]
    assert "selbst" in out[1]
    assert "anderen Fenster" in out[2]
    assert "zeile, spalte oder raster:N" in out[3]
    assert "abstand, rand, ausrichtung, dehnen, rahmen" in out[4]


def test_ein_behaelter_ist_luft_fuer_klicks(tmp_path):
    """GUI_HIT_TEST ueber einem Behaelter liefert das Kind darunter -- oder
    nichts, nie den Behaelter. Sonst schluckte er jeden Klick auf seine
    Kinder, die in der Reihenfolge nach ihm kommen."""
    out = _lauf(tmp_path, _KOPF +
                'DIM z AS GUI_WIDGET : z = GUI_LAYOUT(w, "zeile", 0, 0, 300, 40)\n'
                'DIM a AS GUI_WIDGET : a = GUI_BUTTON(w, "A", 0, 0, 40, 30)\n'
                'GUI_LAYOUT_ADD(z, a)\nGUI_UPDATE()\n'
                'PRINT GUI_HIT_TEST(10, 10) = a\n'
                'PRINT GUI_HIT_TEST(200, 10) = z\n'
                'PRINT GUI_FOCUSED()\n', frames=1)
    assert out == ["TRUE", "FALSE", "-1"]


def test_layout_ueberlebt_die_datei(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM s AS GUI_WIDGET : s = GUI_LAYOUT(w, "raster:3", 0, 0, 300, 100)\n'
                'GUI_LAYOUT_SET(s, "abstand", 2) : GUI_LAYOUT_SET(s, "rand", 4) : GUI_LAYOUT_SET(s, "dehnen", 0)\n'
                'DIM a AS GUI_WIDGET : a = GUI_BUTTON(w, "A", 0, 0, 0, 0)\n'
                'GUI_LAYOUT_ADD(s, a, 2) : GUI_LAYOUT_SPACER(s)\n'
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)\n'
                'PRINT GUI_TO_JSON(w2) = j\n'
                'PRINT INSTR(j, "raster") >= 0 ; " " ; INSTR(j, "auto_w") >= 0\n', frames=1)
    assert out == ["TRUE", "TRUE TRUE"]
