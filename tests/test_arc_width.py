"""ARC(x1, y1, x2, y2, start_rad, end_rad[, color[, width]]) -- der optionale
`width`-Parameter war in docs/builtins-grafik.md dokumentiert (und in
examples/30_shapes.gb bereits als 8. Argument uebergeben!), aber die
Implementierung in vm.rs/graphics.rs las nur 7 Argumente und zeichnete immer
einen 1px-Strich (Cmd::Poly ohne Dicke). Dieser Test rendert headless
(DHRT_FRAMES + DHRT_SCREENSHOT) einen duennen und einen dicken Bogen und
prueft per Pixel-Sample knapp neben der Bogen-Mittellinie: beim duennen Bogen
bleibt der Punkt Hintergrundfarbe, beim dicken Bogen ist er gefuellt.
"""
import os
import subprocess
from pathlib import Path

import pytest


def _dhrt():
    root = Path(__file__).resolve().parent.parent
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((root / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (root / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


# Zwei identische Viertelkreis-Boegen (Bounding-Box 100x100, Radius 50,
# Winkel 0..PI/2 -- Mittelpunkt der Kurve bei t=PI/4 liegt ~35px diagonal vom
# Box-Zentrum entfernt). Links duenn (kein width-Arg -> alter 1px-Pfad),
# rechts mit width=10 (neuer Thick-Pfad via DrawSplineLinear).
_PROG = """\
SCREEN(320, 240)
WHILE NOT QUITREQUESTED()
    CLS(RGB(0, 0, 0))
    ' Duenner Bogen (kein width-Arg) -- BBox (40,40)-(140,140), Zentrum (90,90)
    ARC(40, 40, 140, 140, 0.0, 1.5707963, RGB(255, 0, 255))
    ' Dicker Bogen (width=10) -- BBox (200,40)-(300,140), Zentrum (250,90)
    ARC(200, 40, 300, 140, 0.0, 1.5707963, RGB(255, 0, 255), 10)
    FLIP()
WEND
"""


def test_arc_width_makes_stroke_thicker(tmp_path):
    dhrt = _dhrt()
    if dhrt is None:
        pytest.skip("native Runtime 'dhrt' nicht gebaut")
    from PIL import Image

    src = tmp_path / "arc_width.gb"
    src.write_text(_PROG, encoding="utf-8")
    shot = tmp_path / "arc_width.png"
    env = dict(os.environ, DHRT_FRAMES="2", DHRT_SCREENSHOT=str(shot))
    r = subprocess.run([str(dhrt), "run", str(src)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60, env=env)
    assert r.returncode == 0, f"dhrt Exit {r.returncode}: {r.stderr}"
    assert shot.exists() and shot.stat().st_size > 0, "kein Screenshot erzeugt"

    img = Image.open(shot).convert("RGB")

    def is_magenta(x, y):
        px = img.getpixel((x, y))
        return px[0] > 180 and px[1] < 80 and px[2] > 180

    # Auf der Kurve selbst (t=PI/4, Radius 50) sind BEIDE Boegen sichtbar.
    assert is_magenta(125, 55), "duenner Bogen: Punkt auf der Kurve nicht gefuellt"
    assert is_magenta(285, 55), "dicker Bogen: Punkt auf der Kurve nicht gefuellt"

    # 4px radial nach innen versetzt (Radius 46, gleicher Winkel): der
    # duenne 1px-Strich trifft diesen Punkt nicht, der 10px-dicke Strich
    # (Halbbreite 5px) schon -- das ist der eigentliche Width-Beweis.
    assert not is_magenta(123, 58), "duenner Bogen: width wirkt (sollte er nicht)"
    assert is_magenta(283, 58), "dicker Bogen: width=10 hat keine sichtbare Wirkung"
