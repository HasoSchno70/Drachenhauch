"""Die Kante des festen Spalten-Blocks in der gui-Tabelle.

Sie war zuerst in der AKZENTfarbe gezeichnet und immer sichtbar. Beides war
falsch: der Akzent bedeutet im Rest der Oberflaeche "ausgewaehlt/aktiv", hier
haette er "ab hier scrollt es" bedeutet -- zwei Dinge in einer Farbe. Und bei
stehender Tabelle erklaert die Kante gar nichts, dort ist sie nur eine
willkuerliche Markierung mitten im Bild.

Geprueft wird die ruhende Tabelle per Pixel-Sample: an der Blockgrenze darf
KEINE Akzentfarbe stehen. (Der gescrollte Fall braucht eine echte Maus auf der
Bildlaufleiste; er ist ueber zwei gerenderte Bilder von Hand abgenommen.)
"""
import os
import subprocess
from pathlib import Path

import pytest


def _gbrt():
    root = Path(__file__).resolve().parent.parent
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    return next((root / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (root / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


# Fenster bei (0,0), Titel 22 -> Inhalt ab y=22. Tabelle bei (0,0) im Inhalt.
# Zwei feste Spalten a 100 px -> die Blockgrenze liegt bei x = 1 + 200 = 201.
_PROG = """\
IMPORT "gui"
SCREEN(600, 300, "T", 1)
GUI_THEME_PRESET("glas_dunkel")
DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 600, 278)
DIM t AS GUI_WIDGET : t = GUI_TABLE(w, 0, 0, 560, 240)
DIM k AS ARRAY OF STRING : k = SPLIT$("A|B|C|D|E|F", "|")
GUI_TABLE_HEADERS(t, k)
DIM br AS ARRAY OF INTEGER : br = [100, 100, 100, 100, 100, 100]
GUI_TABLE_COL_WIDTHS(t, br)
GUI_TABLE_SET(t, "feste_spalten", 2)
DIM z AS ARRAY OF STRING
z = SPLIT$("a|b|c|d|e|f", "|") : GUI_TABLE_ADD_ROW(t, z)
z = SPLIT$("g|h|i|j|k|l", "|") : GUI_TABLE_ADD_ROW(t, z)
WHILE NOT QUITREQUESTED()
    GUI_UPDATE()
    CLS(0)
    GUI_DRAW()
    FLIP()
WEND
"""


def test_blockgrenze_zeigt_keinen_akzent_wenn_nicht_gescrollt(tmp_path):
    gbrt = _gbrt()
    if gbrt is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    from PIL import Image

    src = tmp_path / "kante.gb"
    src.write_text(_PROG, encoding="utf-8")
    shot = tmp_path / "kante.png"
    env = dict(os.environ, GBRT_FRAMES="3", GBRT_SCREENSHOT=str(shot))
    r = subprocess.run([str(gbrt), "run", str(src)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60, env=env)
    assert r.returncode == 0, f"gbrt Exit {r.returncode}: {r.stderr}"
    assert shot.exists() and shot.stat().st_size > 0, "kein Screenshot erzeugt"

    img = Image.open(shot).convert("RGB")

    def akzent(x, y):
        """Cyan-Akzent des Themas: deutlich mehr Blau als Rot."""
        px = img.getpixel((x, y))
        return px[2] > 120 and px[2] - px[0] > 60

    # Gegenprobe ZUERST: die Tabelle muss ueberhaupt gezeichnet sein, sonst
    # wuerde der eigentliche Test auch auf einem leeren Bild gruen. Beweis ist
    # eine Spaltentrennung an der bekannten Stelle x=100 (erste Spalte 100 px
    # breit) -- die gibt es nur, wenn die Tabelle mit ihren Spalten da ist.
    # (Ein blosser Helligkeits-Schwellwert war hier zu wackelig: die Kopfzeile
    # des dunklen Themas liegt bei (22,27,34) und faellt durch fast jede
    # Schwelle, die man raet.)
    assert img.getpixel((99, 60)) != img.getpixel((100, 60)), "keine Spaltenkante bei x=100"

    # Die Blockgrenze liegt hinter zwei Spalten a 100 px, also bei x=200.
    # Ein paar Punkte drumherum, weil die Kante nur 1 px breit ist.
    for x in range(196, 206):
        for y in (60, 80, 100, 140):
            assert not akzent(x, y), (
                f"Akzent-Kante bei ({x},{y}) obwohl nicht gescrollt: {img.getpixel((x, y))}")
