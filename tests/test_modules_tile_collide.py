"""Tests fuer tile_collide-Modul (Box-vs-Tilemap-Sweep).

Golden-Tests gegen `dhrt` (Stufe B): der Test schreibt eine Tiled-JSON-Map in
`tmp_path` und laesst ein GB-Programm sie via TILED_LOAD laden (run_gb mit
`base=tmp_path` legt die .gb daneben, dhrt chdirt dorthin). Frueher via
`call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""
import json
import pytest

from gamebasic.errors import GBRuntimeError

_HEAD = 'IMPORT "tiled"\nIMPORT "tile_collide"\n' \
        'DIM m AS TILED_MAP\nm = TILED_LOAD("level.json")\n'


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _write_level(tmp_path, tile_data, width=8, height=4, tile_w=16, tile_h=16,
                 with_solid=True, name="level.json", layer_type="tilelayer"):
    tiles_meta = []
    if with_solid:
        tiles_meta = [{"id": 0, "properties": [
            {"name": "solid", "type": "bool", "value": True}]}]
    layer = {"type": layer_type, "name": "ground", "width": width,
             "height": height, "data": tile_data}
    if layer_type == "objectgroup":
        layer = {"type": "objectgroup", "name": "obj", "objects": []}
    data = {
        "type": "map", "width": width, "height": height,
        "tilewidth": tile_w, "tileheight": tile_h,
        "tilesets": [{
            "firstgid": 1, "name": "tiles", "tilewidth": tile_w,
            "tileheight": tile_h, "columns": 4, "image": "tiles.png",
            "imagewidth": 64, "imageheight": 32, "tiles": tiles_meta,
        }],
        "layers": [layer],
    }
    (tmp_path / name).write_text(json.dumps(data), encoding="utf-8")


def _run(run_gb, tmp_path, tile_data, body, **kw):
    _write_level(tmp_path, tile_data, **kw)
    return _lines(run_gb(_HEAD + body, base=tmp_path))


_WALL_X3 = [0, 0, 0, 1, 0, 0, 0, 0] * 4
_EMPTY = [0] * 32


# --- TILE_IS_SOLID ------------------------------------------------

def test_solid_with_property(run_gb, tmp_path):
    data = [0] * 32
    data[1 * 8 + 2] = 1   # Tile (2,1) solid
    out = _run(run_gb, tmp_path, data,
               "PRINT TILE_IS_SOLID(m, 0, 2, 1)\nPRINT TILE_IS_SOLID(m, 0, 0, 0)\n")
    assert out == ["TRUE", "FALSE"]


def test_solid_fallback_no_property(run_gb, tmp_path):
    """Kein Tileset mit solid-Property: jeder GID > 0 ist solid."""
    data = [0] * 32
    data[10] = 1
    out = _run(run_gb, tmp_path, data, with_solid=False, body=(
        "PRINT TILE_IS_SOLID(m, 0, 2, 1)\nPRINT TILE_IS_SOLID(m, 0, 0, 0)\n"))
    assert out == ["TRUE", "FALSE"]


def test_solid_out_of_bounds_is_solid(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _EMPTY,
               "PRINT TILE_IS_SOLID(m, 0, -1, 0)\nPRINT TILE_IS_SOLID(m, 0, 99, 0)\n")
    assert out == ["TRUE", "TRUE"]


# --- TILE_AT_PIXEL -----------------------------------------------

def test_tile_at_pixel(run_gb, tmp_path):
    data = [0] * 32
    data[1 * 8 + 2] = 1
    data[1 * 8 + 3] = 2
    out = _run(run_gb, tmp_path, data, with_solid=False, body=(
        "PRINT TILE_AT_PIXEL(m, 0, 32, 16)\n"
        "PRINT TILE_AT_PIXEL(m, 0, 48, 16)\n"
        "PRINT TILE_AT_PIXEL(m, 0, 0, 0)\n"))
    assert out == ["1", "2", "0"]


# --- TILE_SWEEP_X ------------------------------------------------

def test_sweep_x_no_collision(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _EMPTY,
               "PRINT TILE_SWEEP_X(m, 0, 10.0, 10.0, 8.0, 8.0, 5.0)\n")
    assert out == ["(15.0, FALSE)"]


def test_sweep_x_hits_wall_right(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _WALL_X3,
               "PRINT TILE_SWEEP_X(m, 0, 20.0, 16.0, 10.0, 16.0, 50.0)\n")
    assert out == ["(38.0, TRUE)"]


def test_sweep_x_hits_wall_left(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _WALL_X3,
               "PRINT TILE_SWEEP_X(m, 0, 80.0, 16.0, 10.0, 16.0, -50.0)\n")
    assert out == ["(64.0, TRUE)"]


def test_sweep_x_zero_delta(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _EMPTY,
               "PRINT TILE_SWEEP_X(m, 0, 10.0, 10.0, 8.0, 8.0, 0.0)\n")
    assert out == ["(10.0, FALSE)"]


# --- TILE_SWEEP_Y ------------------------------------------------

def test_sweep_y_falls_onto_ground(run_gb, tmp_path):
    data = [0] * 24 + [1] * 8   # Boden in row 3
    out = _run(run_gb, tmp_path, data,
               "PRINT TILE_SWEEP_Y(m, 0, 10.0, 20.0, 8.0, 16.0, 30.0)\n")
    assert out == ["(32.0, TRUE)"]


def test_sweep_y_hits_ceiling(run_gb, tmp_path):
    data = [1] * 8 + [0] * 24   # Decke in row 0
    out = _run(run_gb, tmp_path, data,
               "PRINT TILE_SWEEP_Y(m, 0, 10.0, 40.0, 8.0, 8.0, -40.0)\n")
    assert out == ["(16.0, TRUE)"]


def test_sweep_y_falls_freely(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _EMPTY,
               "PRINT TILE_SWEEP_Y(m, 0, 10.0, 10.0, 8.0, 8.0, 20.0)\n")
    assert out == ["(30.0, FALSE)"]


def test_sweep_y_stops_at_world_edge(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _EMPTY,
               "PRINT TILE_SWEEP_Y(m, 0, 10.0, 50.0, 8.0, 8.0, 20.0)\n")
    assert out == ["(56.0, TRUE)"]


def test_sweep_diagonal_via_two_axes(run_gb, tmp_path):
    # L-Wand: rechts bei x=3, Boden bei y=3
    data = [0, 0, 0, 1, 0, 0, 0, 0] * 3 + [1, 1, 1, 1, 0, 0, 0, 0]
    out = _run(run_gb, tmp_path, data,
               "DIM r AS TUPLE\n"
               "r = TILE_SWEEP_X(m, 0, 20.0, 20.0, 8.0, 8.0, 50.0)\nPRINT r\n"
               "DIM nx AS FLOAT\nnx = r[0]\n"
               "PRINT TILE_SWEEP_Y(m, 0, nx, 20.0, 8.0, 8.0, 50.0)\n")
    assert out == ["(40.0, TRUE)", "(40.0, TRUE)"]


# --- Argument-Validation ------------------------------------------

def test_sweep_invalid_layer(run_gb, tmp_path):
    with pytest.raises(GBRuntimeError, match="kein Tile-Layer"):
        _run(run_gb, tmp_path, _EMPTY, layer_type="objectgroup", body=(
            "PRINT TILE_SWEEP_X(m, 0, 0.0, 0.0, 8.0, 8.0, 5.0)\n"))


def test_sweep_zero_size_box_errors(run_gb, tmp_path):
    with pytest.raises(GBRuntimeError, match="> 0"):
        _run(run_gb, tmp_path, _EMPTY,
             "PRINT TILE_SWEEP_X(m, 0, 10.0, 10.0, 0.0, 8.0, 5.0)\n")
