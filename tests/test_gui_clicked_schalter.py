"""`GUI_CLICKED` auf Kaestchen, Kippschalter und Radioknoepfen.

Gefunden an zwei Piloten auf einmal: der Aufruf war dort STUMM -- immer
FALSE, ohne Fehler. Der Tilemap- und der Sprite-Editor hatten damit beide
einen toten "sichtbar"-Schalter: das Haekchen kippte, die Ebene blieb
sichtbar. Zwei von zwei Programmen, die es versucht haben, lagen falsch --
dann ist das Schweigen der Fehler, nicht die Erwartung.

Ein Knopf setzt sein `clicked` beim LOSLASSEN, ein Kaestchen kippt schon
beim Druecken. Fuer den Abfragenden ist beides "in diesem Bild angeklickt";
geprueft wird deshalb nur, dass es GENAU EIN Bild lang meldet.

Die Tests speisen Mausklicks ein und tragen darum den `seriell`-Marker --
dasselbe Nadeloehr wie `test_automation.py`.
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
pytestmark = [pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut"),
              pytest.mark.seriell]


_RUMPF_KLICK = '''IMPORT "gui"
SCREEN(320, 240, "Test", 1)
SET_WINDOW_POS(-3000, -3000)
AUTOMATION_PLAY("ev.txt")
DIM w AS GUI_WINDOW
w = GUI_WINDOW("Kasten", 10, 10, 300, 200)
DIM cb AS GUI_WIDGET : cb = GUI_CHECKBOX(w, "an", 20, 20, FALSE)
DIM tg AS GUI_WIDGET : tg = GUI_TOGGLE(w, "kipp", 20, 60, FALSE)
DIM r1 AS GUI_WIDGET : r1 = GUI_RADIO(w, "g", "eins", 20, 100)
DIM r2 AS GUI_WIDGET : r2 = GUI_RADIO(w, "g", "zwei", 20, 130)
DIM n AS INTEGER : n = 0
WHILE NOT QUITREQUESTED()
    GUI_UPDATE()
    IF GUI_CLICKED(%s) THEN n = n + 1
    PRINT "K " + STR$(n) + " " + STR$(GUI_CHECKED(%s))
    CLS(0)
    GUI_DRAW()
    FLIP()
WEND
'''


def _klick_lauf(tmp_path, wdg, x, y, frames=14):
    (tmp_path / "ev.txt").write_text(
        "# Test-Aufnahme\nc 4\n"
        "e 1 7 %d %d 0 0 // Event: test\n"
        "e 2 7 %d %d 0 0 // Event: test\n"
        "e 2 6 0 0 0 0 // Event: test\n"
        "e 3 5 0 0 0 0 // Event: test\n" % (x, y, x, y),
        encoding="utf-8")
    quelle = tmp_path / "k.dh"
    quelle.write_text(_RUMPF_KLICK % (wdg, wdg), encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120, cwd=str(tmp_path),
                       env=dict(os.environ, DHRT_FRAMES=str(frames)))
    assert r.returncode == 0, r.stderr
    zeilen = [ln.split() for ln in (r.stdout or "").splitlines() if ln.startswith("K ")]
    return [(int(z[1]), z[2] == "TRUE") for z in zeilen]


def test_gui_clicked_meldet_ein_kaestchen(tmp_path):
    """Das Kaestchen sitzt bei (20,20) im Fenster (10,10) -- die Titelleiste
    kommt dazu, deshalb wird nicht auf den Punkt geklickt, sondern in seine
    Naehe: (36, 62) liegt sicher darin (Kaestchengroesse ~18)."""
    verlauf = _klick_lauf(tmp_path, "cb", 36, 62)
    assert verlauf[-1][1] is True, "der Haken muss gesetzt sein"
    assert verlauf[-1][0] == 1, "und GUI_CLICKED muss GENAU EIN Bild lang TRUE sein"


def test_gui_clicked_meldet_einen_kippschalter(tmp_path):
    verlauf = _klick_lauf(tmp_path, "tg", 36, 102)
    assert verlauf[-1][1] is True
    assert verlauf[-1][0] == 1


def test_gui_clicked_meldet_einen_radioknopf(tmp_path):
    """Und nur beim WECHSEL: ein zweiter Klick auf den schon gewaehlten
    aendert nichts, also meldet er auch nichts."""
    verlauf = _klick_lauf(tmp_path, "r2", 36, 172)
    assert verlauf[-1][1] is True
    assert verlauf[-1][0] == 1


def test_daneben_geklickt_meldet_nichts(tmp_path):
    """Die Gegenprobe: ohne Treffer bleibt der Zaehler bei 0 -- sonst
    zeigten die drei Tests oben nur, dass irgendetwas hochzaehlt."""
    verlauf = _klick_lauf(tmp_path, "cb", 250, 200)
    assert verlauf[-1] == (0, False)
