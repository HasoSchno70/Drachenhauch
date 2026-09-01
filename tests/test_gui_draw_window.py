"""`GUI_DRAW_WINDOW` -- ein Fenster ueber dem selbst Gezeichneten.

Die Luecke, die das schliesst: `GUI_DRAW` zeichnet Fenster und
Zeichenflaechen in EINEM Durchgang, und der Inhalt einer Zeichenflaeche
entsteht per Bauart DANACH -- das Programm malt selbst hinein. Ein Fenster,
das ueber einer Zeichenflaeche liegt, verschwindet damit hinter deren
Inhalt. Gefunden am Sprite-Piloten: dort ging der Kasten "Neues Sprite" auf,
sperrte die Eingabe und war nicht zu sehen -- das Programm wirkte
eingefroren, und kein Test hat das gemerkt.

Geprueft wird deshalb am BILD, nicht an einem Rueckgabewert: derselbe
Ablauf einmal mit und einmal ohne den Aufruf. Ohne ihn muss der Punkt in der
Fenstermitte die zugemalte Farbe tragen, mit ihm nicht. Die Gegenprobe
steckt damit im Test selbst.
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

# Das Fenster liegt mittig; (160, 120) ist sein Inneres.
_ZUGEMALT = (255, 0, 255)

_RUMPF = '''IMPORT "gui"
SCREEN(320, 240, "Test", 1)
SET_WINDOW_POS(-3000, -3000)
DIM w AS GUI_WINDOW
w = GUI_WINDOW("Kasten", 60, 60, 200, 120)
%s
WHILE NOT QUITREQUESTED()
    CLS(&H000000)
    GUI_UPDATE()
    GUI_DRAW()
    ' Das steht hier fuer den Inhalt einer Zeichenflaeche: gemalt wird
    ' NACH GUI_DRAW, also ueber dem Fenster.
    BOX(0, 0, 319, 239, &HFF00FF)
%s
    FLIP()
WEND
'''


def _bild(tmp_path, oben, aufbau="", frames=3):
    quelle = tmp_path / "a.dh"
    quelle.write_text(_RUMPF % (aufbau, oben), encoding="utf-8")
    ziel = tmp_path / "bild.png"
    r = subprocess.run([str(_DHRT), "run", str(quelle)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120, cwd=str(tmp_path),
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DHRT_SCREENSHOT=str(ziel)))
    return r, ziel


def _punkt(pfad, x=160, y=120):
    from PIL import Image
    return Image.open(pfad).convert("RGB").getpixel((x, y))


def test_ohne_den_aufruf_liegt_das_gemalte_ueber_dem_fenster(tmp_path):
    """Der Ausgangszustand -- und der Grund, warum es den Befehl gibt."""
    pytest.importorskip("PIL")
    r, ziel = _bild(tmp_path, "")
    assert r.returncode == 0, r.stderr
    assert _punkt(ziel) == _ZUGEMALT, "ohne GUI_DRAW_WINDOW deckt das Gemalte zu"


def test_mit_dem_aufruf_liegt_das_fenster_obenauf(tmp_path):
    pytest.importorskip("PIL")
    r, ziel = _bild(tmp_path, "    GUI_DRAW_WINDOW(w)")
    assert r.returncode == 0, r.stderr
    assert _punkt(ziel) != _ZUGEMALT, "mit GUI_DRAW_WINDOW liegt das Fenster oben"


def test_unsichtbares_fenster_zeichnet_nichts(tmp_path):
    """Kein Fehler, nur kein Bild -- der Aufruf darf unbedingt in der
    Bildschleife stehen, ohne dass das Programm ihn absichern muss."""
    pytest.importorskip("PIL")
    r, ziel = _bild(tmp_path, "    GUI_DRAW_WINDOW(w)",
                    aufbau="GUI_WINDOW_VISIBLE(w, FALSE)")
    assert r.returncode == 0, r.stderr
    assert _punkt(ziel) == _ZUGEMALT


def test_zerstoertes_fenster_ist_kein_fehler(tmp_path):
    """Wie bei allen gui-Handles bleibt die Nummer nach GUI_WINDOW_DESTROY
    gueltig (Grabstein) -- sonst muesste jedes Programm mitfuehren, ob sein
    Fenster noch lebt."""
    r, _ = _bild(tmp_path, "    GUI_DRAW_WINDOW(w)", aufbau="GUI_WINDOW_DESTROY(w)")
    assert r.returncode == 0, r.stderr


def test_unbekanntes_handle_meldet_sich(tmp_path):
    """Ein NICHT vergebenes Handle ist etwas anderes als ein zerstoertes --
    das ist ein Tippfehler und muss auffallen."""
    r, _ = _bild(tmp_path, "    GUI_DRAW_WINDOW(4242)")
    assert r.returncode != 0
    assert "GUI_DRAW_WINDOW" in (r.stderr + r.stdout)


# ------------------------------------------------------- Tooltip / obenauf
# Diese drei tragen `seriell` selbst: sie speisen eine Mausposition ein, und
# das geht durch dasselbe Nadeloehr wie bei `test_automation.py` (wer
# waehrenddessen die echte Maus bewegt, schiebt sie in die Wiedergabe). Die
# fuenf Tests darueber brauchen keine Eingabe und duerfen parallel laufen --
# deshalb der Marker je Test statt der ganzen Datei in `_SERIELL`.
#
# Der Tooltip liegt ueber ALLEN Fenstern und ist damit vom selben Problem
# betroffen -- schlimmer sogar: er folgt der Maus und landet deshalb
# staendig ueber einer Zeichenflaeche.
#
# Zweimal zeichnen waere hier keine Loesung: er hat einen halbdurchsichtigen
# Schlagschatten, und der wuerde dort dunkler, wo das eigene Zeichnen die
# erste Fassung NICHT zugedeckt hat. Darum laesst `GUI_DRAW(FALSE)` die
# Schicht weg und `GUI_DRAW_TOP()` holt sie nach.
_RUMPF_TT = '''IMPORT "gui"
SCREEN(320, 240, "Test", 1)
SET_WINDOW_POS(-3000, -3000)
AUTOMATION_PLAY("ev.txt")
DIM w AS GUI_WINDOW
w = GUI_WINDOW("Kasten", 10, 10, 300, 110)
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "Knopf", 10, 60, 90, 26)
GUI_TOOLTIP(b, "HILFETEXT")
WHILE NOT QUITREQUESTED()
    CLS(&H000000)
    GUI_UPDATE()
    %s
%s
%s
    FLIP()
WEND
'''

# Der Zeiger ruht auf dem Knopf; der Tooltip erscheint rechts UNTER ihm --
# also im zugemalten Streifen.
_MAUS = (60, 105)
_TT_PUNKT = (100, 134)


def _tooltip_bild(tmp_path, draw, malen, top, frames=45):
    (tmp_path / "ev.txt").write_text(
        "# Test-Aufnahme\nc 1\ne 0 7 %d %d 0 0 // Event: test\n" % _MAUS,
        encoding="utf-8")
    quelle = tmp_path / "t.dh"
    quelle.write_text(_RUMPF_TT % (draw, malen, top), encoding="utf-8")
    ziel = tmp_path / "tt.png"
    r = subprocess.run([str(_DHRT), "run", str(quelle)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120, cwd=str(tmp_path),
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DHRT_SCREENSHOT=str(ziel)))
    return r, ziel


_MALEN = "    BOX(0, 120, 319, 239, &HFF00FF)"


@pytest.mark.seriell
def test_tooltip_liegt_ohne_top_unter_dem_gemalten(tmp_path):
    pytest.importorskip("PIL")
    r, ziel = _tooltip_bild(tmp_path, "GUI_DRAW()", _MALEN, "")
    assert r.returncode == 0, r.stderr
    assert _punkt(ziel, *_TT_PUNKT) == _ZUGEMALT


@pytest.mark.seriell
def test_gui_draw_top_holt_den_tooltip_nach_oben(tmp_path):
    pytest.importorskip("PIL")
    r, ziel = _tooltip_bild(tmp_path, "GUI_DRAW(FALSE)", _MALEN, "    GUI_DRAW_TOP()")
    assert r.returncode == 0, r.stderr
    assert _punkt(ziel, *_TT_PUNKT) != _ZUGEMALT


@pytest.mark.seriell
def test_gui_draw_false_laesst_den_tooltip_ganz_weg(tmp_path):
    """Ohne Zumalen und ohne GUI_DRAW_TOP darf gar kein Tooltip erscheinen --
    sonst waere das Argument wirkungslos und der Test oben bewiese nur, dass
    GUI_DRAW_TOP zeichnet."""
    pytest.importorskip("PIL")
    r, mit = _tooltip_bild(tmp_path, "GUI_DRAW()", "", "")
    assert r.returncode == 0, r.stderr
    farbe_mit = _punkt(mit, *_TT_PUNKT)
    r2, ohne = _tooltip_bild(tmp_path, "GUI_DRAW(FALSE)", "", "")
    assert r2.returncode == 0, r2.stderr
    assert _punkt(ohne, *_TT_PUNKT) != farbe_mit, "GUI_DRAW(FALSE) muss ihn weglassen"
    assert _punkt(ohne, *_TT_PUNKT) == (0, 0, 0), "dort ist dann der leere Hintergrund"
