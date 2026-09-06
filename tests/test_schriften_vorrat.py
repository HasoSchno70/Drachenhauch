"""Fremde Schriften und das Euro-Zeichen (docs/entwurf-eingabemethoden.md,
Weg A): der Grund-Zeichenvorrat reicht bis Kyrillisch, alles andere backt
die Laufzeit auf Zuruf aus den Schriften des Systems nach, und LOADFONT
nimmt eine Zeichenwahl.

Gemessen wird am BILD: derselbe Text einmal echt und einmal als lauter
Fragezeichen -- vor dem Bau waren beide Bilder gleich (`aä????`). Die
Gegenprobe: ein Zeichen, das keine Schrift hat (Privatbereich U+E000),
bleibt ein Fragezeichen, die Bilder sind dann gleich.
"""
import os
import subprocess
import sys
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

_KOPF = ('SCREEN(400, 120, "Schrift", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n')


def _bild(tmp_path, name, src, frames=3):
    f = tmp_path / f"{name}.dh"
    f.write_text(_KOPF + src, encoding="utf-8")
    png = tmp_path / f"{name}.png"
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120,
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DHRT_SCREENSHOT=str(png)),
                       cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    out = [ln.strip() for ln in (r.stdout or "").splitlines()
           if ln.strip() and not ln.startswith(("WARNING:", "INFO:"))]
    return png, out


def _schleife(zeile):
    return ('WHILE NOT QUITREQUESTED()\n'
            '    CLS(0)\n'
            f'    {zeile}\n'
            '    FLIP()\n'
            'WEND\n')


def _pixel_unterschiede(a, b):
    from PIL import Image
    ia = Image.open(a).convert("RGB")
    ib = Image.open(b).convert("RGB")
    assert ia.size == ib.size
    pa, pb = ia.load(), ib.load()
    return sum(1 for x in range(ia.size[0]) for y in range(ia.size[1]) if pa[x, y] != pb[x, y])


def _dunkle_pixel(png):
    from PIL import Image
    im = Image.open(png).convert("RGB")
    px = im.load()
    return sum(1 for x in range(im.size[0]) for y in range(im.size[1]) if sum(px[x, y]) > 300)


def test_euro_griechisch_kyrillisch_sind_keine_fragezeichen(tmp_path):
    # Beide Seiten tragen ein `\u00e4`: so laufen beide durch dieselbe
    # Ausweich-Schrift -- ein reiner ASCII-Text naehme raylibs Bitmapschrift,
    # und der Vergleich saehe zwei Schriften statt zwei Zeichen.
    echt, _ = _bild(tmp_path, "echt", _schleife('TEXT(10, 10, "\u00e4 12,50 \u20ac \u03a9 \u042f \u0151 \u0142")'))
    frage, _ = _bild(tmp_path, "frage", _schleife('TEXT(10, 10, "\u00e4 12,50 ? ? ? ? ?")'))
    assert _pixel_unterschiede(echt, frage) > 30, "Euro, Omega, Ja, o-Doppelakut und l-Strich sehen aus wie Fragezeichen"


def test_zeichen_ohne_schrift_bleibt_ein_fragezeichen(tmp_path):
    # Gegenprobe zum Test davor: U+E000 (Privatbereich) hat keine Schrift --
    # dann ist das Fragezeichen richtig, und beide Bilder sind gleich.
    echt, _ = _bild(tmp_path, "pua", _schleife('TEXT(10, 10, "\u00e4 \ue000 b")'))
    frage, _ = _bild(tmp_path, "puafrage", _schleife('TEXT(10, 10, "\u00e4 ? b")'))
    assert _pixel_unterschiede(echt, frage) == 0


@pytest.mark.skipif(sys.platform != "win32", reason="Systemschriften fuer CJK/Hangul/Emoji nur unter Windows bekannt")
def test_cjk_hangul_emoji_kommen_auf_zuruf(tmp_path):
    # Das erste Bild zeichnet die Zeichen auf, FLIP backt nach -- schon das
    # erste Bild ist richtig; das dritte wird fotografiert.
    echt, _ = _bild(tmp_path, "cjk", _schleife('TEXT(10, 10, "\u00e4 \u65e5\u672c\u8a9e \ud55c\uae00 \U0001f600")'))
    frage, _ = _bild(tmp_path, "cjkfrage", _schleife('TEXT(10, 10, "\u00e4 ??? ?? ?")'))
    assert _pixel_unterschiede(echt, frage) > 100
    assert _dunkle_pixel(echt) > _dunkle_pixel(frage), "Kanji, Hangul und Emoji tragen mehr Tinte als Fragezeichen"


@pytest.mark.skipif(sys.platform != "win32", reason="braucht arial.ttf und msgothic.ttc")
def test_loadfont_mit_zeichenwahl_und_sammlung(tmp_path):
    src = ('DIM f AS INTEGER : DIM g AS INTEGER\n'
           'f = LOADFONT("C:/Windows/Fonts/arial.ttf", 24, "kyrillisch, griechisch")\n'
           'g = LOADFONT("C:/Windows/Fonts/msgothic.ttc", 24, "japanisch")\n'
           'PRINT f ; " " ; g\n'
           'TRY\n    f = LOADFONT("C:/Windows/Fonts/arial.ttf", 24, "klingonisch")\nCATCH e\n    PRINT e\nEND TRY\n'
           'WHILE NOT QUITREQUESTED()\n'
           '    CLS(0)\n'
           '    SETFONT(f) : TEXT(10, 10, "\u041f\u0440\u0438\u0432\u0435\u0442 \u0393\u03b5\u03b9\u03ac")\n'
           '    SETFONT(g) : TEXT(10, 50, "\u6771\u4eac")\n'
           '    FLIP()\n'
           'WEND\n')
    echt, out = _bild(tmp_path, "lf", src)
    assert out[0] == "0 1", out
    assert "unbekannter Schriftblock" in out[1] and "kyrillisch" in out[1]
    frage, _ = _bild(tmp_path, "lffrage",
                     'DIM f AS INTEGER : DIM g AS INTEGER\n'
                     'f = LOADFONT("C:/Windows/Fonts/arial.ttf", 24, "kyrillisch, griechisch")\n'
                     'g = LOADFONT("C:/Windows/Fonts/msgothic.ttc", 24, "japanisch")\n'
                     'WHILE NOT QUITREQUESTED()\n'
                     '    CLS(0)\n'
                     '    SETFONT(f) : TEXT(10, 10, "?????? ????")\n'
                     '    SETFONT(g) : TEXT(10, 50, "??")\n'
                     '    FLIP()\n'
                     'WEND\n')
    assert _pixel_unterschiede(echt, frage) > 100


def test_textbreite_misst_dieselben_laeufe(tmp_path):
    # Die Breite eines Textes mit fremden Zeichen kommt aus denselben
    # Schriften wie das Zeichnen -- ein Layout darf nicht mit einer anderen
    # Schrift rechnen. Belegt ueber Monotonie und Unterschied zum `?`.
    _, out = _bild(tmp_path, "breite",
                   'PRINT TEXT_WIDTH("\u65e5\u672c") ; " " ; TEXT_WIDTH("??") ; " " ; TEXT_WIDTH("\u65e5\u672c\u65e5\u672c")\n'
                   'WHILE NOT QUITREQUESTED()\n    CLS(0) : TEXT(10, 10, "\u65e5\u672c") : FLIP()\nWEND\n', frames=2)
    a, b, c = [int(x) for x in out[0].split()]
    assert a > 0 and c > a, out
    if sys.platform == "win32":
        assert a != b, "zwei Kanji sind nicht so breit wie zwei Fragezeichen"
