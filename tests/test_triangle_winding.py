"""TRIANGLE/POLYGON (gefuellt) sind wicklungsunabhaengig.

raylib `DrawTriangle`/`DrawTriangleFan` cullen Rueckseiten nach Wicklungs-
reihenfolge -- mit im Uhrzeigersinn (CW) angegebenen Eckpunkten wurde frueher
NICHTS gezeichnet. gbrt dreht die Vertex-Reihenfolge intern um, wenn die signed
area CW ergibt. Dieser Test rendert headless (GBRT_FRAMES + GBRT_SCREENSHOT) je
ein CW- und ein CCW-Dreieck/Polygon und prueft per Pixel, dass BEIDE sichtbar
gefuellt sind (vor dem Fix waere das CW-Exemplar schwarz geblieben).
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


# CW-Dreieck (signed area > 0 in y-down) -- wurde frueher weggecullt --, CCW-
# Dreieck, dazu ein CW- und ein CCW-Polygon (Quadrat). Centroiden liegen nicht
# uebereinander; Hintergrund schwarz, Fuellung magenta.
_PROG = """\
SCREEN(320, 240)
DIM cw[8] AS INTEGER
cw[0] = 40 : cw[1] = 40 : cw[2] = 100 : cw[3] = 40
cw[4] = 100 : cw[5] = 100 : cw[6] = 40 : cw[7] = 100
DIM ccw[8] AS INTEGER
ccw[0] = 200 : ccw[1] = 40 : ccw[2] = 200 : ccw[3] = 100
ccw[4] = 260 : ccw[5] = 100 : ccw[6] = 260 : ccw[7] = 40
' Jeden Frame neu zeichnen (Immediate-Mode); Screenshot beim 2. Frame, damit
' der gerenderte Inhalt im gelesenen Front-Buffer liegt (Double-Buffering).
WHILE NOT QUITREQUESTED()
    CLS(RGB(0, 0, 0))
    ' CW-Dreieck (frueher gecullt) -- Centroid ~ (220, 167)
    TRIANGLE(180, 150, 260, 150, 220, 200, RGB(255, 0, 255))
    ' CCW-Dreieck (zeichnete schon immer) -- Centroid ~ (60, 167)
    TRIANGLE(20, 150, 60, 200, 100, 150, RGB(255, 0, 255))
    ' CW-Quadrat (frueher gecullt) -- Zentrum (70, 70)
    POLYGON(cw, RGB(255, 0, 255))
    ' CCW-Quadrat -- Zentrum (230, 70)
    POLYGON(ccw, RGB(255, 0, 255))
    FLIP()
WEND
"""


def test_triangle_polygon_winding_independent(tmp_path):
    gbrt = _gbrt()
    if gbrt is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    from PIL import Image

    src = tmp_path / "winding.gb"
    src.write_text(_PROG, encoding="utf-8")
    shot = tmp_path / "winding.png"
    env = dict(os.environ, GBRT_FRAMES="2", GBRT_SCREENSHOT=str(shot))
    r = subprocess.run([str(gbrt), "run", str(src)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60, env=env)
    assert r.returncode == 0, f"gbrt Exit {r.returncode}: {r.stderr}"
    assert shot.exists() and shot.stat().st_size > 0, "kein Screenshot erzeugt"

    img = Image.open(shot).convert("RGB")

    def is_magenta(x, y):
        px = img.getpixel((x, y))
        return px[0] > 180 and px[1] < 80 and px[2] > 180

    # CW- UND CCW-Variante muessen beide gefuellt sein.
    assert is_magenta(220, 167), "CW-Dreieck nicht gefuellt (weggecullt)"
    assert is_magenta(60, 167), "CCW-Dreieck nicht gefuellt"
    assert is_magenta(70, 70), "CW-Polygon nicht gefuellt (weggecullt)"
    assert is_magenta(230, 70), "CCW-Polygon nicht gefuellt"
