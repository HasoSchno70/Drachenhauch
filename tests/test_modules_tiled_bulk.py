"""Tests fuer die Bulk-Tilemap-Operationen (tiled-Modul)."""
import json

import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("tiled")


def _make_map(tmp_path, call_builtin, data, w=6, h=4, ts=16):
    spec = {
        "type": "map", "width": w, "height": h,
        "tilewidth": ts, "tileheight": ts,
        "tilesets": [{
            "firstgid": 1, "name": "t", "tilewidth": ts, "tileheight": ts,
            "columns": 4, "image": "t.png", "imagewidth": 64, "imageheight": 64,
        }],
        "layers": [{"type": "tilelayer", "name": "g", "width": w, "height": h,
                    "data": data}],
    }
    p = tmp_path / "m.json"
    p.write_text(json.dumps(spec))
    return call_builtin("tiled_load", [str(p)])


# --- COUNT_GID ----------------------------------------------------

def test_count_gid(tmp_path, call_builtin):
    data = [0, 1, 1, 0, 2, 1] + [0] * 18
    m = _make_map(tmp_path, call_builtin, data)
    assert call_builtin("tiled_count_gid", [m, 0, 1]) == 3
    assert call_builtin("tiled_count_gid", [m, 0, 2]) == 1
    assert call_builtin("tiled_count_gid", [m, 0, 0]) == 20


# --- FILL_RECT ----------------------------------------------------

def test_fill_rect(tmp_path, call_builtin):
    data = [0] * 24            # 6x4
    m = _make_map(tmp_path, call_builtin, data)
    # Fuelle 2x2 ab (1,1) mit GID 5
    n = call_builtin("tiled_fill_rect", [m, 0, 1, 1, 2, 2, 5])
    assert n == 4
    assert call_builtin("tiled_count_gid", [m, 0, 5]) == 4
    assert call_builtin("tiled_tile_at", [m, 0, 1, 1]) == 5
    assert call_builtin("tiled_tile_at", [m, 0, 2, 2]) == 5
    assert call_builtin("tiled_tile_at", [m, 0, 0, 0]) == 0


def test_fill_rect_clamps_oob(tmp_path, call_builtin):
    data = [0] * 24
    m = _make_map(tmp_path, call_builtin, data)
    # Rechteck ragt ueber den Rand -> geclamped, kein Crash
    n = call_builtin("tiled_fill_rect", [m, 0, 4, 2, 10, 10, 7])
    # x: 4..6 (2 Spalten), y: 2..4 (2 Reihen) = 4 Tiles
    assert n == 4


def test_fill_rect_counts_only_changed(tmp_path, call_builtin):
    data = [3] * 24
    m = _make_map(tmp_path, call_builtin, data)
    # alles ist schon 3 -> 0 geaendert
    assert call_builtin("tiled_fill_rect", [m, 0, 0, 0, 6, 4, 3]) == 0


# --- REPLACE ------------------------------------------------------

def test_replace(tmp_path, call_builtin):
    data = [1, 2, 1, 2, 1, 2] + [0] * 18
    m = _make_map(tmp_path, call_builtin, data)
    n = call_builtin("tiled_replace", [m, 0, 1, 9])
    assert n == 3
    assert call_builtin("tiled_count_gid", [m, 0, 9]) == 3
    assert call_builtin("tiled_count_gid", [m, 0, 1]) == 0


# --- FLOOD_FILL ---------------------------------------------------

def test_flood_fill_basic(tmp_path, call_builtin):
    # 6x4, eine vertikale Wand (GID 1) bei x=3 trennt links/rechts
    data = [
        0, 0, 0, 1, 0, 0,
        0, 0, 0, 1, 0, 0,
        0, 0, 0, 1, 0, 0,
        0, 0, 0, 1, 0, 0,
    ]
    m = _make_map(tmp_path, call_builtin, data)
    # Fluten der linken Region (3 Spalten x 4 Reihen = 12) ab (0,0) mit 5
    n = call_builtin("tiled_flood_fill", [m, 0, 0, 0, 5])
    assert n == 12
    assert call_builtin("tiled_count_gid", [m, 0, 5]) == 12
    # Rechte Seite unberuehrt (noch 0)
    assert call_builtin("tiled_tile_at", [m, 0, 5, 0]) == 0
    # Wand unberuehrt
    assert call_builtin("tiled_tile_at", [m, 0, 3, 0]) == 1


def test_flood_fill_noop_same_gid(tmp_path, call_builtin):
    data = [0] * 24
    m = _make_map(tmp_path, call_builtin, data)
    # Ziel-GID == vorhandene GID -> 0
    assert call_builtin("tiled_flood_fill", [m, 0, 0, 0, 0]) == 0


def test_flood_fill_oob_start(tmp_path, call_builtin):
    data = [0] * 24
    m = _make_map(tmp_path, call_builtin, data)
    assert call_builtin("tiled_flood_fill", [m, 0, 99, 99, 5]) == 0


# --- Validation ---------------------------------------------------

def test_bulk_on_object_layer_raises(tmp_path, call_builtin):
    spec = {
        "type": "map", "width": 4, "height": 4,
        "tilewidth": 16, "tileheight": 16,
        "tilesets": [{"firstgid": 1, "name": "t", "tilewidth": 16,
                      "tileheight": 16, "columns": 1, "image": "x.png",
                      "imagewidth": 16, "imageheight": 16}],
        "layers": [{"type": "objectgroup", "name": "obj", "objects": []}],
    }
    p = tmp_path / "x.json"
    p.write_text(json.dumps(spec))
    m = call_builtin("tiled_load", [str(p)])
    with pytest.raises(GBRuntimeError, match="kein Tile-Layer"):
        call_builtin("tiled_flood_fill", [m, 0, 0, 0, 5])
