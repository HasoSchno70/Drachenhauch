"""Tests fuer die CIRCUIT RUNNER Level-Pipeline (DAT-Konverter + Demo-Bauer).

Deckt das JSON-Levelformat ab, das die .gb-Engine laedt: convert_dat.py
(echtes Chip's-Challenge-Binformat -> JSON) und make_demo_levels.py
(ASCII -> JSON). Reine Python-Pipeline, kein dhrt noetig.
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


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for variant in ("release", "debug"):
        p = _CR.parent / "rust" / "drachenhauch_runtime" / "target" / variant / exe
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
#  Engine-Verhalten (dhrt-Headless-Harness): MS-Monster-Bewegungsreihenfolge
# ---------------------------------------------------------------------------
_DHRT = _find_dhrt()


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


def _make_corridor_level():
    """Offenes 32x32-Floor-Feld; ein Feuerball (Typ 1) bei (3,5) Richtung rechts.

    Spieler weit weg bei (1,1). In einem freien Korridor zieht der Feuerball
    geradeaus -- so misst der Test pro world_tick einen Schritt (volles Tempo).
    """
    GW = 32
    upper = [0x00] * 1024
    lower = [0x00] * 1024
    upper[1 * GW + 1] = 0x6C            # Spieler-Start
    upper[5 * GW + 3] = 0x47            # Feuerball (Typ 1), Richtung 3 (rechts)
    upper[1 * GW + 30] = 0x15           # Exit (weit weg)
    return {
        "name": "SpeedTest", "ruleset": "ms",
        "levels": [{
            "title": "Speed", "number": 1, "time": 0, "chips": 0,
            "hint": "", "password": "ABCD", "width": 32, "height": 32,
            "upper": _hexgrid(upper), "lower": _hexgrid(lower),
            "traps": "", "cloners": "", "monsters": "",
        }],
    }


def _run_engine_harness(tmp_path, level, harness):
    """Engine-Quelle bis VOR die Hauptschleife + Harness headless via dhrt laufen
    lassen; gibt stdout zurueck."""
    import json
    assets = _CR / "assets"
    if not (assets / "tiles.png").exists():
        pytest.skip("circuitrunner/assets nicht vorhanden")
    src = (_CR / "circuitrunner.gb").read_text(encoding="utf-8")
    head = src.split("WHILE NOT QUITREQUESTED()")[0]
    shutil.copytree(assets, tmp_path / "assets")
    (tmp_path / "synth.json").write_text(json.dumps(level), encoding="utf-8")
    gb = tmp_path / "harness.gb"
    gb.write_text(head + harness, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(gb)],
                       capture_output=True, timeout=120)
    return r.stdout.decode("utf-8", "replace")


@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
def test_monster_move_order_follows_list(tmp_path):
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
    assert "SUB reorder_monsters" in (_CR / "circuitrunner.gb").read_text(
        encoding="utf-8"), "reorder_monsters fehlt in der Engine"
    out = _run_engine_harness(tmp_path, _make_reorder_level(), harness)
    assert "NMOB=4" in out, out
    order = [(int(a), int(b))
             for a, b in re.findall(r"M\s+(\d+)\s+(\d+)", out)]
    assert order == [(8, 5), (5, 2), (3, 5), (10, 10)], (order, out)


def _make_password_set():
    """3-Level-Set mit Passwoertern (fuer find_password + Save-Tests)."""
    GW = 32

    def grid():
        upper = [0x00] * 1024
        upper[1 * GW + 1] = 0x6C        # Spieler
        upper[5 * GW + 5] = 0x15        # Exit
        return _hexgrid(upper)

    lower = _hexgrid([0x00] * 1024)
    pwords = ["WXYZ", "ABCD", "QWER"]
    levels = []
    for i, pw in enumerate(pwords):
        levels.append({
            "title": f"L{i}", "number": i + 1, "time": 0, "chips": 0,
            "hint": "", "password": pw, "width": 32, "height": 32,
            "upper": grid(), "lower": lower,
            "traps": "", "cloners": "", "monsters": "",
        })
    return {"name": "Test Set!", "ruleset": "ms", "levels": levels}


@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
def test_save_highscore_and_password(tmp_path):
    # Bestzeit-Aufzeichnung (schneller ueberschreibt) + Passwort-Lookup + Set-Key.
    harness = (
        '\njs = JSON_LOAD("synth.json")\n'
        'nlevels = JSON_LEN(js, "levels")\n'
        'cur_setkey = set_key("Test Set!")\n'
        'PRINT "SETKEY " + cur_setkey\n'
        'cur_level = 1\n'
        'load_level(1)\n'
        'start_ms = MILLIS() - 30000\n'
        'record_win()\n'
        'PRINT "BEST1 " + STR$(best_for(1)) + " REC " + STR$(IIF(new_record, 1, 0))\n'
        'start_ms = MILLIS() - 50000\n'
        'record_win()\n'
        'PRINT "BEST2 " + STR$(best_for(1)) + " REC " + STR$(IIF(new_record, 1, 0))\n'
        'start_ms = MILLIS() - 10000\n'
        'record_win()\n'
        'PRINT "BEST3 " + STR$(best_for(1)) + " REC " + STR$(IIF(new_record, 1, 0))\n'
        'PRINT "PW_ABCD " + STR$(find_password("abcd"))\n'
        'PRINT "PW_BAD " + STR$(find_password("ZZZZ"))\n'
        'PRINT "MAX " + STR$(SAVE_GET_INT_OR(sav, cur_setkey + "/max", 0))\n'
    )
    out = _run_engine_harness(tmp_path, _make_password_set(), harness)
    vals = dict(re.findall(r"(\w+)\s+(-?\d+)", out))

    assert "SETKEY test_set_" in out, out
    # erste Aufzeichnung ~30s, neuer Rekord
    b1, b2, b3 = int(vals["BEST1"]), int(vals["BEST2"]), int(vals["BEST3"])
    assert 29 <= b1 <= 31, out
    assert vals_rec(out, 1) == 1, out                 # REC nach 1. Sieg
    assert b2 == b1, out                              # langsamer (50s) ignoriert
    assert vals_rec(out, 2) == 0, out                 # kein Rekord
    assert b3 <= 11, out                              # schneller (10s) ueberschreibt
    assert vals_rec(out, 3) == 1, out
    # Passwort case-insensitiv -> Level 1; unbekannt -> -1
    assert int(vals["PW_ABCD"]) == 1, out
    assert int(vals["PW_BAD"]) == -1, out
    # Fortschritt = cur_level+1
    assert int(vals["MAX"]) == 2, out
    # Save-Datei wurde geschrieben
    assert (tmp_path / "circuitrunner.save").exists(), list(tmp_path.iterdir())


def vals_rec(out, n):
    m = re.search(rf"BEST{n}\s+-?\d+\s+REC\s+(\d+)", out)
    return int(m.group(1)) if m else None


def _make_timed_level():
    """1-Level-Set mit Zeitlimit 200 (fuer den CC-Zeitbonus)."""
    GW = 32
    upper = [0x00] * 1024
    upper[1 * GW + 1] = 0x6C
    upper[5 * GW + 5] = 0x15
    return {"name": "Timed", "ruleset": "ms", "levels": [{
        "title": "T", "number": 1, "time": 200, "chips": 0, "hint": "",
        "password": "ABCD", "width": 32, "height": 32,
        "upper": _hexgrid(upper), "lower": _hexgrid([0x00] * 1024),
        "traps": "", "cloners": "", "monsters": ""}]}


@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
def test_save_timed_bonus(tmp_path):
    # Getimtes Level: Bestwert = verbliebene Zeit (CC-Zeitbonus), hoeher = besser.
    harness = (
        '\njs = JSON_LOAD("synth.json")\n'
        'nlevels = JSON_LEN(js, "levels")\n'
        'cur_setkey = set_key("Timed")\n'
        'cur_level = 0\n'
        'load_level(0)\n'
        'start_ms = MILLIS() - 30000\n'        # 30s gebraucht -> 170 uebrig
        'record_win()\n'
        'PRINT "BEST1 " + STR$(best_for(0)) + " REC " + STR$(IIF(new_record, 1, 0))\n'
        'start_ms = MILLIS() - 50000\n'        # 50s -> 150 uebrig (schlechter)
        'record_win()\n'
        'PRINT "BEST2 " + STR$(best_for(0)) + " REC " + STR$(IIF(new_record, 1, 0))\n'
        'start_ms = MILLIS() - 10000\n'        # 10s -> 190 uebrig (besser)
        'record_win()\n'
        'PRINT "BEST3 " + STR$(best_for(0)) + " REC " + STR$(IIF(new_record, 1, 0))\n'
    )
    out = _run_engine_harness(tmp_path, _make_timed_level(), harness)
    b1 = int(re.search(r"BEST1\s+(-?\d+)", out).group(1))
    b2 = int(re.search(r"BEST2\s+(-?\d+)", out).group(1))
    b3 = int(re.search(r"BEST3\s+(-?\d+)", out).group(1))
    assert 169 <= b1 <= 171, out               # 200 - 30 = 170 uebrig
    assert vals_rec(out, 1) == 1, out
    assert b2 == b1, out                        # 150 uebrig ist schlechter -> ignoriert
    assert vals_rec(out, 2) == 0, out
    assert 189 <= b3 <= 191, out                # 190 uebrig ist besser
    assert vals_rec(out, 3) == 1, out


@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
def test_hint_tile_under_player(tmp_path):
    # Spieler-Start auf einem Hinweis-Stein (lower=0x2F) -> tat(px,py)=T_HINT,
    # damit der Hinweis-Banner waehrend des Spielens erscheint.
    GW = 32
    upper = [0x00] * 1024
    upper[1 * GW + 1] = 0x6C
    upper[5 * GW + 5] = 0x15
    lower = [0x00] * 1024
    lower[1 * GW + 1] = 0x2F                      # Hinweis-Stein unter dem Spieler
    level = {"name": "Hint", "ruleset": "ms", "levels": [{
        "title": "H", "number": 1, "time": 0, "chips": 0,
        "hint": "Pass auf das Wasser auf!", "password": "ABCD",
        "width": 32, "height": 32,
        "upper": _hexgrid(upper), "lower": _hexgrid(lower),
        "traps": "", "cloners": "", "monsters": ""}]}
    harness = (
        '\njs = JSON_LOAD("synth.json")\n'
        'nlevels = JSON_LEN(js, "levels")\n'
        'cur_setkey = set_key("Hint")\n'
        'load_level(0)\n'
        'PRINT "TAT " + STR$(tat(px, py))\n'
        'PRINT "HINTLEN " + STR$(LEN(lvl_hint))\n'
    )
    out = _run_engine_harness(tmp_path, level, harness)
    assert int(re.search(r"TAT\s+(\d+)", out).group(1)) == 0x2F, out
    assert int(re.search(r"HINTLEN\s+(\d+)", out).group(1)) > 0, out


@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
def test_monster_moves_at_player_speed(tmp_path):
    # MON_EVERY=1: normale Monster ziehen jeden Tick einen Schritt (CC-Tempo).
    harness = (
        '\njs = JSON_LOAD("synth.json")\n'
        'nlevels = JSON_LEN(js, "levels")\n'
        'load_level(0)\n'
        'state = "play"\n'
        'want_dir = -1\n'
        'DIM tk AS INTEGER\n'
        'FOR tk = 0 TO 2\n'
        '    want_dir = -1\n'
        '    world_tick()\n'
        'NEXT\n'
        'PRINT "POS " + STR$(mob_x[0]) + " " + STR$(mob_y[0])\n'
    )
    out = _run_engine_harness(tmp_path, _make_corridor_level(), harness)
    m = re.search(r"POS\s+(\d+)\s+(\d+)", out)
    assert m, out
    # Start (3,5), 3 Ticks geradeaus rechts -> (6,5). Bei Halbtempo waere x<6.
    assert (int(m.group(1)), int(m.group(2))) == (6, 5), out
