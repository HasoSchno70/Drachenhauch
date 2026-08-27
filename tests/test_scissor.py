"""SCISSOR -- das Clip-Rechteck fuer 2D-Zeichnen.

Die Mechanik lag laengst in `graphics.rs` (das `gui`-Modul clippt seine
Fenster damit), war aber nie als Befehl herausgefuehrt. Fuer eigene
scrollbare Flaechen, Minikarten oder geteilte Bildschirme blieb nur der
Umweg ueber ein RENDERTARGET.

Geprueft wird am gerenderten BILD, nicht am Rueckgabewert: dass ein Clip
gesetzt wurde, sagt nichts darueber, ob auch etwas abgeschnitten wird.
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")


def _bild(tmp_path, quelle, name="s"):
    src = tmp_path / f"{name}.dh"
    src.write_text(quelle, encoding="utf-8")
    shot = tmp_path / f"{name}.png"
    env = dict(os.environ, DHRT_FRAMES="2", DHRT_SCREENSHOT=str(shot))
    r = subprocess.run([str(_DHRT), "run", str(src)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60, env=env)
    assert r.returncode == 0, f"dhrt Exit {r.returncode}: {r.stderr}"
    assert shot.exists() and shot.stat().st_size > 0, "kein Screenshot erzeugt"
    from PIL import Image
    return Image.open(shot).convert("RGB")


def _gruen(img, x, y):
    px = img.getpixel((x, y))
    return px[1] > 180 and px[0] < 80 and px[2] < 80


# Ein Kasten von (20,20) bis (220,220), aber nur (50,50)-(150,150) freigegeben.
_CLIP = """\
SCREEN(320, 240)
WHILE NOT QUITREQUESTED()
    CLS(RGB(0, 0, 0))
    SCISSOR(50, 50, 100, 100)
    BOX(20, 20, 220, 220, RGB(0, 255, 0))
    SCISSOR_END()
    FLIP()
WEND
"""


def test_ausserhalb_wird_abgeschnitten(tmp_path):
    img = _bild(tmp_path, _CLIP)
    # innerhalb des Clips: gezeichnet
    assert _gruen(img, 100, 100), "innerhalb des Clips fehlt die Fuellung"
    assert _gruen(img, 51, 51), "linke obere Clip-Ecke fehlt"
    assert _gruen(img, 148, 148), "rechte untere Clip-Ecke fehlt"
    # ausserhalb: der Kasten reicht dorthin, darf aber nicht sichtbar sein
    assert not _gruen(img, 30, 30), "oberhalb/links des Clips wurde gezeichnet"
    assert not _gruen(img, 200, 200), "unterhalb/rechts des Clips wurde gezeichnet"
    assert not _gruen(img, 100, 30), "oberhalb des Clips wurde gezeichnet"


# Zwei geschachtelte Clips: der innere muss mit dem aeusseren GESCHNITTEN
# werden, nicht ihn ersetzen. Aeusserer (50,50)-(150,150), innerer
# (100,100)-(300,300) -> sichtbar bleibt nur (100,100)-(150,150).
_VERSCHACHTELT = """\
SCREEN(320, 240)
WHILE NOT QUITREQUESTED()
    CLS(RGB(0, 0, 0))
    SCISSOR(50, 50, 100, 100)
    SCISSOR(100, 100, 200, 200)
    BOX(0, 0, 319, 239, RGB(0, 255, 0))
    SCISSOR_END()
    SCISSOR_END()
    FLIP()
WEND
"""


def test_verschachtelte_clips_schneiden_sich(tmp_path):
    img = _bild(tmp_path, _VERSCHACHTELT, "v")
    assert _gruen(img, 120, 120), "Schnittmenge beider Clips fehlt"
    assert not _gruen(img, 60, 60), "nur im AEUSSEREN Clip -- haette wegfallen muessen"
    assert not _gruen(img, 200, 200), "nur im INNEREN Clip -- haette wegfallen muessen"


# Nach SCISSOR_END muss wieder alles gezeichnet werden.
_DANACH = """\
SCREEN(320, 240)
WHILE NOT QUITREQUESTED()
    CLS(RGB(0, 0, 0))
    SCISSOR(50, 50, 20, 20)
    SCISSOR_END()
    BOX(200, 200, 300, 230, RGB(0, 255, 0))
    FLIP()
WEND
"""


def test_nach_dem_ende_wieder_frei(tmp_path):
    img = _bild(tmp_path, _DANACH, "d")
    assert _gruen(img, 250, 215), "nach SCISSOR_END wurde weiter geclippt"


def test_tiefe_wird_gezaehlt(run_gb):
    """SCISSOR_DEPTH sagt, wie viele Clips offen sind -- fuer Code, der nicht
    weiss, ob sein Aufrufer schon einen gesetzt hat."""
    out = run_gb("""
SCREEN(64, 48)
PRINT SCISSOR_DEPTH()
SCISSOR(0, 0, 10, 10)
PRINT SCISSOR_DEPTH()
SCISSOR(0, 0, 5, 5)
PRINT SCISSOR_DEPTH()
SCISSOR_END()
SCISSOR_END()
PRINT SCISSOR_DEPTH()
""")
    assert out.split() == ["0", "1", "2", "0"]


def test_ende_ohne_anfang_ist_ein_fehler(run_gb):
    """Ein SCISSOR_END zu viel naehme dem umgebenden Code (etwa einem
    `gui`-Fenster) seinen Clip weg -- der Fehler zeigte sich sonst erst als
    Zeichnen ueber den Rand hinaus."""
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError, match="kein Clip offen"):
        run_gb('SCREEN(64, 48)\nSCISSOR_END()\n')


def test_negative_masse_werden_abgelehnt(run_gb):
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError, match="nicht negativ"):
        run_gb('SCREEN(64, 48)\nSCISSOR(0, 0, -5, 10)\n')
