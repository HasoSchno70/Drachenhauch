"""Tests fuer die Bulk-Tilemap-Operationen (tiled-Modul).

Golden-Tests gegen `gbrt` (Stufe B): Tiled-JSON-Map in `tmp_path`, via TILED_LOAD
geladen (run_gb mit `base=tmp_path`). Frueher via `call_builtin` gegen die
Python-Impl (in Phase 8 geloescht).
"""
import json
import pytest

from gamebasic.errors import GBRuntimeError

_HEAD = 'IMPORT "tiled"\nDIM m AS TILED_MAP\nm = TILED_LOAD("m.json")\n'


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _write_map(tmp_path, data, w=6, h=4, ts=16, layer_type="tilelayer"):
    layer = ({"type": "objectgroup", "name": "obj", "objects": []}
             if layer_type == "objectgroup"
             else {"type": "tilelayer", "name": "g", "width": w, "height": h, "data": data})
    spec = {
        "type": "map", "width": w, "height": h, "tilewidth": ts, "tileheight": ts,
        "tilesets": [{"firstgid": 1, "name": "t", "tilewidth": ts, "tileheight": ts,
                      "columns": 4, "image": "t.png", "imagewidth": 64, "imageheight": 64}],
        "layers": [layer],
    }
    (tmp_path / "m.json").write_text(json.dumps(spec), encoding="utf-8")


def _run(run_gb, tmp_path, data, body, **kw):
    _write_map(tmp_path, data, **kw)
    return _lines(run_gb(_HEAD + body, base=tmp_path))


# --- COUNT_GID ----------------------------------------------------

def test_count_gid(run_gb, tmp_path):
    data = [0, 1, 1, 0, 2, 1] + [0] * 18
    out = _run(run_gb, tmp_path, data,
               "PRINT TILED_COUNT_GID(m, 0, 1)\n"
               "PRINT TILED_COUNT_GID(m, 0, 2)\n"
               "PRINT TILED_COUNT_GID(m, 0, 0)\n")
    assert out == ["3", "1", "20"]


# --- FILL_RECT ----------------------------------------------------

def test_fill_rect(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, [0] * 24,
               "PRINT TILED_FILL_RECT(m, 0, 1, 1, 2, 2, 5)\n"
               "PRINT TILED_COUNT_GID(m, 0, 5)\n"
               "PRINT TILED_TILE_AT(m, 0, 1, 1)\n"
               "PRINT TILED_TILE_AT(m, 0, 2, 2)\n"
               "PRINT TILED_TILE_AT(m, 0, 0, 0)\n")
    assert out == ["4", "4", "5", "5", "0"]


def test_fill_rect_clamps_oob(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, [0] * 24,
               "PRINT TILED_FILL_RECT(m, 0, 4, 2, 10, 10, 7)\n")
    assert out == ["4"]


def test_fill_rect_counts_only_changed(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, [3] * 24,
               "PRINT TILED_FILL_RECT(m, 0, 0, 0, 6, 4, 3)\n")
    assert out == ["0"]


# --- REPLACE ------------------------------------------------------

def test_replace(run_gb, tmp_path):
    data = [1, 2, 1, 2, 1, 2] + [0] * 18
    out = _run(run_gb, tmp_path, data,
               "PRINT TILED_REPLACE(m, 0, 1, 9)\n"
               "PRINT TILED_COUNT_GID(m, 0, 9)\n"
               "PRINT TILED_COUNT_GID(m, 0, 1)\n")
    assert out == ["3", "3", "0"]


# --- FLOOD_FILL ---------------------------------------------------

def test_flood_fill_basic(run_gb, tmp_path):
    data = [0, 0, 0, 1, 0, 0] * 4   # vertikale Wand bei x=3
    out = _run(run_gb, tmp_path, data,
               "PRINT TILED_FLOOD_FILL(m, 0, 0, 0, 5)\n"
               "PRINT TILED_COUNT_GID(m, 0, 5)\n"
               "PRINT TILED_TILE_AT(m, 0, 5, 0)\n"
               "PRINT TILED_TILE_AT(m, 0, 3, 0)\n")
    assert out == ["12", "12", "0", "1"]


def test_flood_fill_noop_same_gid(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, [0] * 24,
               "PRINT TILED_FLOOD_FILL(m, 0, 0, 0, 0)\n")
    assert out == ["0"]


def test_flood_fill_oob_start(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, [0] * 24,
               "PRINT TILED_FLOOD_FILL(m, 0, 99, 99, 5)\n")
    assert out == ["0"]


# --- Validation ---------------------------------------------------

def test_bulk_on_object_layer_raises(run_gb, tmp_path):
    with pytest.raises(GBRuntimeError, match="kein Tile-Layer"):
        _run(run_gb, tmp_path, [0] * 24, layer_type="objectgroup", body=(
            "PRINT TILED_FLOOD_FILL(m, 0, 0, 0, 5)\n"))
