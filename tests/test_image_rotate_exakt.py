"""IMAGE_ROTATE_CW / IMAGE_ROTATE_CCW -- exakte Vierteldrehung.

`IMAGE_ROTATE` rechnet trigonometrisch und tastet dabei neu ab. Fuer
Pixelgrafik ist das falsch, und zwar AUCH bei 90 Grad -- der Fall, in dem
eine Drehung eigentlich verlustfrei ist. Genau dieser Unterschied wird hier
gemessen, und die Gegenprobe (dieselbe Drehung mit `IMAGE_ROTATE`) steht mit
im Test: ohne sie waere nicht belegt, dass es ueberhaupt etwas zu verbessern
gab.

Gelesen wird mit `GETPIXEL` statt aus einem Bildschirmfoto -- eine Drehung
ist eine Aussage ueber PUNKTE, und ein Foto haette das Zeichnen mit im Weg.
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

# 4x2 mit drei Marken in verschiedenen Ecken -- nicht quadratisch, damit der
# Groessentausch mitgeprueft wird, und asymmetrisch, damit sich CW und CCW
# ueberhaupt unterscheiden koennen.
_QUELLE = '''IMPORT "imgfx"
DIM b AS IMAGE : b = IMAGE_NEW(4, 2, RGB(20, 20, 20))
IMAGE_DRAW_RECT(b, 0, 0, 1, 1, RGB(255, 0, 0))
IMAGE_DRAW_RECT(b, 3, 0, 1, 1, RGB(0, 255, 0))
IMAGE_DRAW_RECT(b, 3, 1, 1, 1, RGB(0, 0, 255))

SUB zeig(i AS IMAGE)
    PRINT IMAGEWIDTH(i); "x"; IMAGEHEIGHT(i)
    DIM y AS INTEGER
    FOR y = 0 TO IMAGEHEIGHT(i) - 1
        DIM z AS STRING : z = ""
        DIM x AS INTEGER
        FOR x = 0 TO IMAGEWIDTH(i) - 1
            z = z + IIF(x = 0, "", " ") + HEX$(GETPIXEL(i, x, y))
        NEXT
        PRINT z
    NEXT
END SUB
'''


def _run(src, tmp_path):
    (tmp_path / "s.dh").write_text(_QUELLE + src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "s.dh")], capture_output=True,
                       text=True, encoding="utf-8", errors="replace",
                       env=dict(os.environ, DHRT_FRAMES="1"), timeout=90, cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return [ln.strip() for ln in (r.stdout or "").splitlines() if ln.strip()]


def test_cw_sortiert_die_punkte_nur_um(tmp_path):
    """Rechtsdrehung: die linke obere Ecke wandert nach rechts oben, und die
    Masse tauschen (4x2 -> 2x4). Jede Farbe muss sich wiederfinden."""
    zeilen = _run("zeig(IMAGE_ROTATE_CW(b))", tmp_path)
    assert zeilen[0] == "2x4"
    assert zeilen[1:] == ["141414 FF0000",
                          "141414 141414",
                          "141414 141414",
                          "FF FF00"]


def test_ccw_dreht_in_die_andere_richtung(tmp_path):
    zeilen = _run("zeig(IMAGE_ROTATE_CCW(b))", tmp_path)
    assert zeilen[0] == "2x4"
    assert zeilen[1:] == ["FF00 FF",
                          "141414 141414",
                          "141414 141414",
                          "FF0000 141414"]


def test_vier_vierteldrehungen_geben_das_original(tmp_path):
    """Die schaerfste Zusage: verlustfrei. Wer irgendwo neu abtastet, kommt
    hier nicht wieder beim Ausgangsbild heraus."""
    zeilen = _run("zeig(IMAGE_ROTATE_CW(IMAGE_ROTATE_CW("
                  "IMAGE_ROTATE_CW(IMAGE_ROTATE_CW(b)))))", tmp_path)
    assert zeilen == ["4x2",
                      "FF0000 141414 141414 FF00",
                      "141414 141414 141414 FF"]


def test_cw_und_ccw_heben_sich_auf(tmp_path):
    zeilen = _run("zeig(IMAGE_ROTATE_CCW(IMAGE_ROTATE_CW(b)))", tmp_path)
    assert zeilen == ["4x2",
                      "FF0000 141414 141414 FF00",
                      "141414 141414 141414 FF"]


def test_image_rotate_verliert_die_punkte(tmp_path):
    """Die Gegenprobe, und der Grund fuer die beiden neuen Befehle:
    `IMAGE_ROTATE(b, 90.0)` laesst von den drei Marken NICHTS uebrig und
    verwaescht sogar die einfarbige Flaeche.

    Faellt dieser Test eines Tages, weil raylib genauer geworden ist, dann
    ist das eine gute Nachricht -- aber die Doku und die Begruendung der
    beiden Befehle muessen dann nachgezogen werden, nicht der Test
    stillschweigend angepasst.
    """
    zeilen = _run("zeig(IMAGE_ROTATE(b, 90.0))", tmp_path)
    inhalt = " ".join(zeilen[1:])
    for marke in ("FF0000", "FF00 ", " FF"):
        assert marke not in inhalt + " ", \
            "IMAGE_ROTATE haelt die Marke %r -- die Begruendung neu pruefen" % marke


def test_das_quellbild_bleibt_unberuehrt(tmp_path):
    """imgfx ist unveraenderlich -- eine Drehung liefert ein NEUES Bild."""
    zeilen = _run("DIM r AS IMAGE : r = IMAGE_ROTATE_CW(b)\nzeig(b)", tmp_path)
    assert zeilen[0] == "4x2"
