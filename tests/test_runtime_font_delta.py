"""Regressionstests fuer zwei gbrt-Verhaltensweisen (frueher Stolpersteine):

1. LOADFONT-Groesse "wirkt": ein per LOADFONT geladener Font setzt beim SETFONT
   die Render-Groesse (TEXT_WIDTH skaliert mit der Lade-Groesse). TEXT_SIZE
   uebersteuert weiterhin.
2. DELTA() liefert headless (GBRT_FRAMES gesetzt) einen festen Schritt 1/60 s,
   damit zeitbasierte Spiele deterministisch laufen/testbar sind.

Brauchen die native Runtime + einen TTF (circuitrunner/assets/font.ttf).
"""
from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "gb_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_GBRT = _find_gbrt()
_FONT = _ROOT / "circuitrunner" / "assets" / "font.ttf"
_TILES = _ROOT / "circuitrunner" / "assets" / "tiles.png"


def _run(src: str, tmp_path: Path, env: dict | None = None) -> str:
    f = tmp_path / "t.gb"
    f.write_text(src, encoding="utf-8")
    e = dict(os.environ)
    if env:
        e.update(env)
    r = subprocess.run([str(_GBRT), "run", str(f)], capture_output=True,
                       timeout=60, env=e)
    return r.stdout.decode("utf-8", "replace")


@pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")
@pytest.mark.skipif(not _FONT.exists(), reason="Test-Font fehlt")
def test_loadfont_size_applies_via_setfont(tmp_path):
    fp = _FONT.as_posix()
    src = (
        'SCREEN(320, 120, "t")\n'
        'DIM a AS INTEGER\n'
        'DIM b AS INTEGER\n'
        f'a = LOADFONT("{fp}", 30)\n'
        f'b = LOADFONT("{fp}", 60)\n'
        'SETFONT(a)\n'
        'PRINT TEXT_WIDTH("WIDTH")\n'
        'SETFONT(b)\n'
        'PRINT TEXT_WIDTH("WIDTH")\n'
        'TEXT_SIZE(30)\n'
        'PRINT TEXT_WIDTH("WIDTH")\n'
    )
    out = _run(src, tmp_path)
    nums = [int(x) for x in re.findall(r"\d+", out)]
    assert len(nums) >= 3, out
    w30, w60, w_override = nums[0], nums[1], nums[2]
    # 60px-Font ~ doppelt so breit wie 30px (Toleranz fuer Hinting/Rundung)
    assert 1.7 * w30 <= w60 <= 2.3 * w30, (w30, w60)
    # TEXT_SIZE(30) uebersteuert die 60px-Font-Groesse wieder auf ~30px-Breite
    assert abs(w_override - w30) <= max(2, w30 * 0.1), (w30, w_override)


@pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")
@pytest.mark.skipif(not _TILES.exists(), reason="Test-Tileset fehlt")
def test_drawimagepartex_runs(tmp_path):
    img = _TILES.as_posix()
    src = (
        'SCREEN(200, 200, "t")\n'
        'DIM t AS IMAGE\n'
        f't = LOADIMAGE("{img}")\n'
        'DRAWIMAGEPARTEX(t, 64, 0, 32, 32, 10, 10, 128, 128)\n'
        'PRINT "OK"\n'
    )
    out = _run(src, tmp_path)
    assert "OK" in out, out   # fehlt das Builtin -> Fehler statt "OK"


@pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")
def test_delta_fixed_step_headless(tmp_path):
    src = 'SCREEN(160, 100, "t")\nPRINT DELTA()\n'
    out = _run(src, tmp_path, env={"GBRT_FRAMES": "2"})
    m = re.search(r"[01]\.\d+", out)
    assert m, out
    assert abs(float(m.group(0)) - (1.0 / 60.0)) < 0.005, out
