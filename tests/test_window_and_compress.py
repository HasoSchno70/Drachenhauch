"""Fenster-Zustand/Politur und COMPRESS$/DECOMPRESS$ (Etappe 2 des Ausbaus)."""
import os
import subprocess
from pathlib import Path

import pytest


def _find_gbrt():
    root = Path(__file__).resolve().parent.parent
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    return next((root / "rust" / "gb_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (root / "rust" / "gb_runtime" / "target" / v / exe).exists()), None)


_GBRT = _find_gbrt()
pytestmark = pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")


def _run(src: str, tmp_path, frames: int | None = None):
    p = tmp_path / "t.gb"
    p.write_text(src, encoding="utf-8")
    env = dict(os.environ)
    if frames is not None:
        env["GBRT_FRAMES"] = str(frames)
    return subprocess.run([str(_GBRT), "run", str(p)], capture_output=True, text=True,
                          encoding="utf-8", env=env, timeout=60)


# --------------------------------------------------------------- Kompression
def test_compress_roundtrip_is_exact_and_shrinks(tmp_path):
    r = _run('DIM s AS STRING\nDIM i AS INTEGER\n'
             'FOR i = 1 TO 60\n'
             '    s = s + "Savegame-Zeile " + STR$(i) + " mit viel Wiederholung. "\n'
             'NEXT\n'
             'DIM p AS STRING\np = COMPRESS$(s)\n'
             'PRINT DECOMPRESS$(p) = s\nPRINT LEN(p) < LEN(s)\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.stdout.split() == ["TRUE", "TRUE"]


def test_compress_handles_empty_and_unicode(tmp_path):
    # DEFLATE arbeitet auf Bytes -- der Base64-Umweg muss UTF-8 unversehrt lassen.
    r = _run('PRINT DECOMPRESS$(COMPRESS$("")) = ""\n'
             'PRINT DECOMPRESS$(COMPRESS$("Grueße äöü ✓ 日本")) = "Grueße äöü ✓ 日本"\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.stdout.split() == ["TRUE", "TRUE"]


def test_compress_output_is_plain_ascii(tmp_path):
    # Muss durch JSON/Dateien/Cloud-Slots passen, wo heute BASE64-Ausgaben stehen.
    r = _run('DIM p AS STRING\np = COMPRESS$("hallo welt hallo welt hallo welt")\n'
             'DIM i AS INTEGER\nDIM ok AS BOOLEAN\nok = TRUE\n'
             'FOR i = 0 TO LEN(p) - 1\n'
             '    IF ASC(MID$(p, i, 1)) > 127 THEN ok = FALSE\n'
             'NEXT\nPRINT ok\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.stdout.strip() == "TRUE"


def test_decompress_rejects_garbage(tmp_path):
    r = _run('PRINT DECOMPRESS$("das ist nicht gepackt")\n', tmp_path)
    assert r.returncode != 0
    assert "DECOMPRESS$" in r.stderr


def test_compress_works_without_a_window(tmp_path):
    # Ungated (miniz_oxide statt raylibs CompressData) -- Konsolen-Programme
    # sollen ihre Savegames ebenfalls packen koennen.
    r = _run('PRINT DECOMPRESS$(COMPRESS$("ohne SCREEN")) \n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.stdout.strip() == "ohne SCREEN"


# --------------------------------------------------------------- Fenster
def test_window_state_queries(tmp_path):
    r = _run('SCREEN(64, 64, "T", 1)\n'
             'PRINT WINDOW_MINIMIZED()\nPRINT WINDOW_IS_FULLSCREEN()\n'
             'PRINT GET_TIME() >= 0.0\n', tmp_path, frames=2)
    assert r.returncode == 0, r.stderr
    assert r.stdout.split() == ["FALSE", "FALSE", "TRUE"]


def test_window_icon_accepts_an_image_and_rejects_a_bad_handle(tmp_path):
    ok = _run('SCREEN(64, 64, "T", 1)\n'
              'DIM ico AS IMAGE\nico = GENTEX_COLOR(32, 32, &HFF00FF)\n'
              'WINDOW_ICON(ico)\nWINDOW_OPACITY(0.8)\nPRINT "ok"\n', tmp_path, frames=2)
    assert ok.returncode == 0, ok.stderr
    assert ok.stdout.strip() == "ok"

    bad = _run('SCREEN(64, 64, "T", 1)\nWINDOW_ICON(999)\n', tmp_path, frames=1)
    assert bad.returncode != 0 and "WINDOW_ICON" in bad.stderr


def test_window_opacity_clamps_instead_of_failing(tmp_path):
    r = _run('SCREEN(64, 64, "T", 1)\n'
             'WINDOW_OPACITY(-5.0)\nWINDOW_OPACITY(99.0)\nPRINT "ok"\n', tmp_path, frames=2)
    assert r.returncode == 0, r.stderr


def test_openurl_only_allows_http_schemes(tmp_path):
    # Wache mit Sicherheitsgrund: raylibs OpenURL reicht die Zeichenkette an die
    # Shell weiter -- ein `file:`-Schema waere ein Weg, aus einem harmlos
    # wirkenden GB-Programm heraus ein Programm zu starten.
    for bad in ("file:///C:/Windows/System32/calc.exe", "cmd /c dir", "ftp://x/y"):
        r = _run(f'SCREEN(64, 64, "T", 1)\nOPENURL("{bad}")\n', tmp_path, frames=1)
        assert r.returncode != 0, bad
        assert "OPENURL" in r.stderr
