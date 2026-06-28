"""Tests fuer die CIRCUIT RUNNER Level-Pipeline (DAT-Konverter + Demo-Bauer).

Deckt das JSON-Levelformat ab, das die .gb-Engine laedt: convert_dat.py
(echtes Chip's-Challenge-Binformat -> JSON) und make_demo_levels.py
(ASCII -> JSON). Reine Python-Pipeline, kein gbrt noetig.
"""
from __future__ import annotations

import importlib.util
import struct
from pathlib import Path

_CR = Path(__file__).resolve().parent.parent / "circuitrunner"


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
