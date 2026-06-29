"""Tests fuer die CIRCUIT RUNNER Level-Pipeline (DAT-Konverter + Demo-Bauer).

Deckt das JSON-Levelformat ab, das die .gb-Engine laedt: convert_dat.py
(echtes Chip's-Challenge-Binformat -> JSON) und make_demo_levels.py
(ASCII -> JSON). Reine Python-Pipeline, kein gbrt noetig.
"""
from __future__ import annotations

import importlib.util
import os
import re
import shutil
import struct
import subprocess
from pathlib import Path

import pytest

_CR = Path(__file__).resolve().parent.parent / "circuitrunner"


def _find_gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = _CR.parent / "rust" / "gb_runtime" / "target" / variant / exe
        if p.exists():
            return p
    return None


def _load(name):
    spec = importlib.util.spec_from_file_location(name, _CR / f"{name}.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _rle_encode(tiles):
    out = bytearray()
    i, n = 0, len(tiles)
    while i < n:
        t = tiles[i]
        j = i
        while j < n and tiles[j] == t and (j - i) < 255:
            j += 1
        run = j - i
        if run >= 4 or t == 0xFF:
            out += bytes([0xFF, run, t])
            i = j
        else:
            out.append(t)
            i += 1
    return bytes(out)


def _make_dat(upper, lower, *, number=1, time=0, chips=3,
              title=b"TEST ROOM", password="ABCD", hint=b"hi",
              magic=0x0002AAAC):
    up = _rle_encode(upper)
    lo = _rle_encode(lower)
    title_f = title + b"\x00"
    pw = bytes(c ^ 0x99 for c in (password.encode() + b"\x00"))
    hint_f = hint + b"\x00"
    opt = (bytes([0x03, len(title_f)]) + title_f
           + bytes([0x06, len(pw)]) + pw
           + bytes([0x07, len(hint_f)]) + hint_f)
    body = (struct.pack("<HHHH", number, time, chips, 1)
            + struct.pack("<H", len(up)) + up
            + struct.pack("<H", len(lo)) + lo
            + struct.pack("<H", len(opt)) + opt)
    level = struct.pack("<H", len(body)) + body
    return struct.pack("<I", magic) + struct.pack("<H", 1) + level


def test_convert_roundtrip(tmp_path):
    cd = _load("convert_dat")
    upper = [0x00] * 1024
    for x in range(32):
        upper[x] = 1
        upper[31 * 32 + x] = 1
    for y in range(32):
        upper[y * 32] = 1
        upper[y * 32 + 31] = 1
    upper[1 * 32 + 1] = 0x6C            # Spieler
    for cx in (2, 3, 4):
        upper[2 * 32 + cx] = 0x02       # 3 Chips
    upper[5 * 32 + 5] = 0x15            # Exit
    lower = [0x00] * 1024

    p = tmp_path / "test.dat"
    p.write_bytes(_make_dat(upper, lower))
    res = cd.convert(p)

    assert res["ruleset"] == "ms"
    assert len(res["levels"]) == 1
    lv = res["levels"][0]
    assert lv["title"] == "TEST ROOM"
    assert lv["chips"] == 3
    assert lv["password"] == "ABCD"
    assert lv["hint"] == "hi"
    assert len(lv["upper"]) == 2048 and len(lv["lower"]) == 2048
    u = [int(lv["upper"][i:i + 2], 16) for i in range(0, 2048, 2)]
    assert u[1 * 32 + 1] == 0x6C
    assert u[5 * 32 + 5] == 0x15
    assert sum(1 for t in u if t == 0x02) == 3


def test_convert_lynx_magic(tmp_path):
    cd = _load("convert_dat")
    p = tmp_path / "lynx.dat"
    p.write_bytes(_make_dat([0] * 1024, [0] * 1024, magic=0x0102AAAC))
    assert cd.convert(p)["ruleset"] == "lynx"


def test_demo_levels_schema():
    md = _load("make_demo_levels")
    lvls = md.levels()
    assert len(lvls) >= 5
    for lv in lvls:
        assert len(lv["upper"]) == 2048
        assert len(lv["lower"]) == 2048
        u = [int(lv["upper"][i:i + 2], 16) for i in range(0, 2048, 2)]
        # genau ein Spieler-Start (Code 0x6C..0x6F)
        assert sum(1 for t in u if 0x6C <= t <= 0x6F) == 1
        # mindestens ein Exit
        assert any(t in (0x15, 0x39, 0x3A, 0x3B) for t in u)
        # Chip-Zaehler stimmt mit Tiles ueberein
        assert lv["chips"] == sum(1 for t in u if t == 0x02)


# ---------------------------------------------------------------------------
#  Engine-Verhalten (gbrt-Headless-Harness): MS-Monster-Bewegungsreihenfolge
# ---------------------------------------------------------------------------
_GBRT = _find_gbrt()


def _hexgrid(tiles):
    return "".join(f"{t:02x}" for t in tiles)


def _make_reorder_level():
    """32x32-Level mit 4 Monstern; `monsters`-Liste in NICHT-Lesereihenfolge.

    Reading-Order (Map-Scan) waere [(5,2),(3,5),(8,5),(10,10)]; die 0x0A-Liste
    nennt nur (8,5),(5,2),(3,5) -> Engine muss in dieser Reihenfolge laden und
    das ungelistete (10,10) in Lesereihenfolge anhaengen.
    """
    GW = 32
    upper = [0x00] * 1024
    lower = [0x00] * 1024
    upper[1 * GW + 1] = 0x6C            # Spieler-Start
    for (x, y) in [(5, 2), (3, 5), (8, 5), (10, 10)]:
        upper[y * GW + x] = 0x42        # Kaefer (Typ 0), Richtung 2
    upper[15 * GW + 15] = 0x15          # Exit
    return {
        "name": "ReorderTest", "ruleset": "ms",
        "levels": [{
            "title": "Reorder", "number": 1, "time": 0, "chips": 0,
            "hint": "", "password": "ABCD", "width": 32, "height": 32,
            "upper": _hexgrid(upper), "lower": _hexgrid(lower),
            "traps": "", "cloners": "", "monsters": "8,5;5,2;3,5",
        }],
    }


@pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")
def test_monster_move_order_follows_list(tmp_path):
    import json
    assets = _CR / "assets"
    if not (assets / "tiles.png").exists():
        pytest.skip("circuitrunner/assets nicht vorhanden")

    # Engine-Quelle bis VOR die Hauptschleife + Logik-Harness anhaengen
    src = (_CR / "circuitrunner.gb").read_text(encoding="utf-8")
    head = src.split("WHILE NOT QUITREQUESTED()")[0]
    assert "SUB reorder_monsters" in head, "reorder_monsters fehlt in der Engine"
    harness = (
        '\njs = JSON_LOAD("synth.json")\n'
        'nlevels = JSON_LEN(js, "levels")\n'
        'load_level(0)\n'
        'PRINT "NMOB=" + STR$(nmob)\n'
        'DIM kk AS INTEGER\n'
        'FOR kk = 0 TO nmob - 1\n'
        '    PRINT "M " + STR$(mob_x[kk]) + " " + STR$(mob_y[kk])\n'
        'NEXT\n'
    )

    shutil.copytree(assets, tmp_path / "assets")
    (tmp_path / "synth.json").write_text(
        json.dumps(_make_reorder_level()), encoding="utf-8")
    gb = tmp_path / "harness.gb"
    gb.write_text(head + harness, encoding="utf-8")

    r = subprocess.run([str(_GBRT), "run", str(gb)],
                       capture_output=True, timeout=120)
    out = r.stdout.decode("utf-8", "replace")
    assert "NMOB=4" in out, out
    order = [(int(a), int(b))
             for a, b in re.findall(r"M\s+(\d+)\s+(\d+)", out)]
    assert order == [(8, 5), (5, 2), (3, 5), (10, 10)], (order, out)
