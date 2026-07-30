"""Bild- und Text-Ausbau (Etappe 3): Faltung, Alpha-Operationen, Dither,
Palette, neue Textur-Generatoren, animierte GIFs, Bitmap-Fonts."""
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


def _run(src: str, tmp_path, frames: int = 2):
    p = tmp_path / "t.gb"
    p.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_GBRT), "run", str(p)], capture_output=True, text=True,
                       encoding="utf-8", env=dict(os.environ, GBRT_FRAMES=str(frames)),
                       timeout=60, cwd=str(tmp_path))
    # raylib schreibt seine Meldungen auf STDOUT (bekannte Eigenheit, vgl.
    # profiler.py) -- ungefiltert landen sie mitten in der Programmausgabe.
    r.out = [w for ln in (r.stdout or "").splitlines()
             if not ln.startswith(("WARNING:", "INFO:", "TRACE:"))
             for w in ln.split()]
    return r


_SCREEN = 'SCREEN(200, 160, "T", 1)\n'


# ------------------------------------------------------------------ Faltung
def test_convolve_identity_keeps_the_image_size(tmp_path):
    r = _run(_SCREEN +
             'DIM k[9] AS FLOAT\nk[4] = 1.0\n'          # Identitaets-Kern
             'DIM a AS IMAGE\na = GENTEX_COLOR(32, 24, &HFF8800)\n'
             'DIM b AS IMAGE\nb = IMAGE_CONVOLVE(a, k)\n'
             'PRINT IMAGEWIDTH(b)\nPRINT IMAGEHEIGHT(b)\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["32", "24"]


def test_convolve_rejects_a_non_square_kernel(tmp_path):
    r = _run(_SCREEN + 'DIM k[6] AS FLOAT\n'
             'DIM a AS IMAGE\na = GENTEX_COLOR(8, 8, 255)\n'
             'DIM b AS IMAGE\nb = IMAGE_CONVOLVE(a, k)\n', tmp_path)
    assert r.returncode != 0 and "quadratisch" in r.stderr


def test_convolve_rejects_an_even_kernel_side(tmp_path):
    # 16 Werte waeren 4x4 -- ohne Mittelpunkt ist eine Faltung nicht definiert.
    r = _run(_SCREEN + 'DIM k[16] AS FLOAT\n'
             'DIM a AS IMAGE\na = GENTEX_COLOR(8, 8, 255)\n'
             'DIM b AS IMAGE\nb = IMAGE_CONVOLVE(a, k)\n', tmp_path)
    assert r.returncode != 0 and "ungerade" in r.stderr


# ------------------------------------------------------------------- Dither
def test_dither_accepts_only_the_three_real_16bit_formats(tmp_path):
    ok = _run(_SCREEN + 'DIM a AS IMAGE\na = GENTEX_PERLIN(32, 32, 4.0)\n'
              'DIM b AS IMAGE\nb = IMAGE_DITHER(a, 5, 6, 5, 0)\n'
              'DIM c AS IMAGE\nc = IMAGE_DITHER(a, 5, 5, 5, 1)\n'
              'DIM d AS IMAGE\nd = IMAGE_DITHER(a, 4, 4, 4, 4)\n'
              'PRINT IMAGEWIDTH(d)\n', tmp_path)
    assert ok.returncode == 0, ok.stderr
    assert ok.stdout.strip() == "32"


def test_dither_rejects_a_combination_raylib_cannot_produce(tmp_path):
    # raylib warnt sonst nur und laesst ein Bild mit ungueltigem Format zurueck
    # -- die Textur wird dann schwarz, ohne dass jemand etwas merkt.
    r = _run(_SCREEN + 'DIM a AS IMAGE\na = GENTEX_COLOR(8, 8, 255)\n'
             'DIM b AS IMAGE\nb = IMAGE_DITHER(a, 2, 2, 2, 0)\n', tmp_path)
    assert r.returncode != 0
    assert "IMAGE_DITHER" in r.stderr and "5,6,5,0" in r.stderr


# ------------------------------------------------------------------ Palette
def test_palette_returns_at_most_the_requested_number(tmp_path):
    r = _run(_SCREEN + 'DIM a AS IMAGE\na = GENTEX_PERLIN(48, 48, 5.0)\n'
             'DIM p AS ARRAY OF INTEGER\np = IMAGE_PALETTE(a, 6)\n'
             'PRINT p.length() <= 6\nPRINT p.length() > 0\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE", "TRUE"]


def test_palette_of_a_single_colour_image(tmp_path):
    r = _run(_SCREEN + 'DIM a AS IMAGE\na = GENTEX_COLOR(16, 16, &HFF8800)\n'
             'DIM p AS ARRAY OF INTEGER\np = IMAGE_PALETTE(a, 8)\n'
             'PRINT p.length()\nPRINT HEX$(p[0])\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["1", "FF8800"]


# ------------------------------------------------------------- Alpha-Ops
def test_alpha_operations_return_usable_images(tmp_path):
    r = _run(_SCREEN +
             'DIM a AS IMAGE\na = GENTEX_RADIAL(32, 32, &HFFFFFF, &H000000)\n'
             'DIM m AS IMAGE\nm = GENTEX_GRADIENT(32, 32, &HFFFFFF, &H000000, TRUE)\n'
             'DIM b AS IMAGE\nb = IMAGE_ALPHA_MASK(a, m)\n'
             'DIM c AS IMAGE\nc = IMAGE_ALPHA_PREMULTIPLY(b)\n'
             'DIM d AS IMAGE\nd = IMAGE_ALPHA_CROP(c, 0.1)\n'
             'PRINT IMAGEWIDTH(b)\nPRINT IMAGEWIDTH(d) <= 32\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["32", "TRUE"]


# --------------------------------------------------------- Textur-Generatoren
def test_new_texture_generators_have_the_requested_size(tmp_path):
    r = _run(_SCREEN +
             'DIM a AS IMAGE\na = GENTEX_CELLULAR(40, 30, 12)\n'
             'DIM b AS IMAGE\nb = GENTEX_NOISE(40, 30, 0.2)\n'
             'DIM c AS IMAGE\nc = GENTEX_GRADIENT_BOX(40, 30, 0.5, &HFF0000, &H0000FF)\n'
             'PRINT IMAGEWIDTH(a)\nPRINT IMAGEHEIGHT(a)\nPRINT IMAGEWIDTH(b)\nPRINT IMAGEWIDTH(c)\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["40", "30", "40", "40"]


# Animierte GIFs (LOADGIF) wurden bewusst NICHT umgesetzt -- siehe die
# Begruendung in graphics.rs: raylib laesst `width`/`height` bei einem Bild,
# und raylib-rs macht `Image` ausdruecklich readonly. Fuer Animationen ist der
# Sprite-Blatt-Weg (ATLAS_*, `sprite`-Modul) vorgesehen und schneller.


# ------------------------------------------------------------------ Font
def test_bitmap_font_from_an_image(tmp_path):
    # Bitmap-Font aus einem Bild: raylib trennt die Zeichen an der Trennfarbe.
    # Geprueft wird, dass ein FONT-Handle entsteht und TEXT damit laeuft.
    r = _run(_SCREEN +
             'DIM sheet AS IMAGE\nsheet = GENTEX_COLOR(64, 16, &HFFFFFF)\n'
             'DIM f AS INTEGER\nf = LOADFONT_IMAGE(sheet, &HFF00FF, 32)\n'
             'SETFONT(f)\nTEXT_LINE_SPACING(20)\n'
             'CLS(0)\nTEXT(4, 4, "AB")\nFLIP()\nPRINT f >= 0\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert " ".join(r.out) == "TRUE"


def test_bitmap_font_rejects_a_bad_image_handle(tmp_path):
    r = _run(_SCREEN + 'DIM f AS INTEGER\nf = LOADFONT_IMAGE(999, &HFF00FF, 32)\n', tmp_path)
    assert r.returncode != 0 and "LOADFONT_IMAGE" in r.stderr
