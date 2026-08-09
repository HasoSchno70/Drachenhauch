"""IMAGE_SCALE_NN -- Skalieren ohne Interpolation (Pixelgrafik).

`IMAGE_SCALE` glaettet bilinear. Fuer Fotos ist das richtig, fuer Pixelgrafik
falsch: ein 2x2-Bild auf 64x64 hochskaliert ist danach ein weicher Verlauf
statt vier klarer Bloecke. Der Test misst genau diesen Unterschied an einem
Pixel, der bei bilinearer Glaettung nachweislich eine Mischfarbe traegt.
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
Image = pytest.importorskip("PIL.Image", reason="Pillow fuer die Pixel-Pruefung noetig")


def _run(src, tmp_path, shot=None, frames=3):
    (tmp_path / "s.gb").write_text(src, encoding="utf-8")
    env = dict(os.environ, DHRT_FRAMES=str(frames))
    if shot:
        env["DHRT_SCREENSHOT"] = str(tmp_path / shot)
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "s.gb")], capture_output=True,
                       text=True, encoding="utf-8", env=env, timeout=90, cwd=str(tmp_path))
    r.out = [w for ln in (r.stdout or "").splitlines()
             if not ln.startswith(("WARNING:", "INFO:", "TRACE:")) for w in ln.split()]
    return r


def _pixel(tmp_path, shot, xy):
    from PIL import Image as PImage
    return PImage.open(tmp_path / shot).convert("RGB").getpixel(xy)


# 2x2-Schachbrett: links-oben rot, rechts-oben blau (GenImageChecked).
_SRC = 'DIM src AS IMAGE\nsrc = GENTEX_CHECKED(2, 2, 1, 1, &HFF0000, &H0000FF)\n'


def _loop(draw):
    """Zeichnen gehoert IN die Schleife -- dhrt leert den Befehlspuffer bei
    jedem FLIP, ein einmaliges Zeichnen davor waere im Screenshot-Frame weg."""
    return f'WHILE NOT QUITREQUESTED()\n    CLS(&H000000)\n    {draw}\n    FLIP()\nWEND\n'


def test_nearest_neighbour_keeps_hard_edges(tmp_path):
    gb = ('SCREEN(128, 128, "NN", 1)\n' + _SRC +
          'DIM big AS IMAGE\nbig = IMAGE_SCALE_NN(src, 64, 64)\n'
          + _loop("DRAWIMAGE(big, 0, 0)"))
    r = _run(gb, tmp_path, shot="nn.png")
    assert r.returncode == 0, r.stderr
    # Dicht an der Blockgrenze (Quelltexel-Mitten liegen bei 16 und 48):
    # genau HIER mischt die bilineare Variante, NN muss rein bleiben.
    assert _pixel(tmp_path, "nn.png", (24, 10)) == (255, 0, 0)
    assert _pixel(tmp_path, "nn.png", (31, 10)) == (255, 0, 0)
    assert _pixel(tmp_path, "nn.png", (40, 10)) == (0, 0, 255)


def test_bilinear_scale_blends_the_same_pixel(tmp_path):
    # Gegenprobe: derselbe Punkt mit IMAGE_SCALE traegt eine MISCHFARBE. Ohne
    # sie waere nicht belegt, dass der Test oben ueberhaupt etwas unterscheidet.
    gb = ('SCREEN(128, 128, "BI", 1)\n' + _SRC +
          'DIM big AS IMAGE\nbig = IMAGE_SCALE(src, 64, 64)\n'
          + _loop("DRAWIMAGE(big, 0, 0)"))
    r = _run(gb, tmp_path, shot="bi.png")
    assert r.returncode == 0, r.stderr
    px = _pixel(tmp_path, "bi.png", (31, 10))
    assert px != (255, 0, 0), "IMAGE_SCALE glaettet hier nicht mehr -- Test neu eichen"
    assert px[0] > 0 and px[2] > 0, f"erwartet Mischfarbe, war {px}"


def test_original_stays_untouched(tmp_path):
    # imgfx ist immutable: das Quellbild behaelt seine Groesse.
    gb = ('SCREEN(64, 64, "U", 1)\n' + _SRC +
          'DIM big AS IMAGE\nbig = IMAGE_SCALE_NN(src, 32, 32)\n'
          'PRINT IMAGEWIDTH(src)\nPRINT IMAGEWIDTH(big)\n')
    r = _run(gb, tmp_path, frames=1)
    assert r.returncode == 0, r.stderr
    assert r.out == ["2", "32"]


def test_zero_size_is_rejected(tmp_path):
    gb = ('SCREEN(64, 64, "E", 1)\n' + _SRC +
          'DIM big AS IMAGE\nbig = IMAGE_SCALE_NN(src, 0, 16)\n')
    r = _run(gb, tmp_path, frames=1)
    assert r.returncode != 0 and "IMAGE_SCALE_NN" in r.stderr
