"""Tests fuer das tiled-Modul (Tiled-Map-Loader).

Golden-Tests gegen `dhrt` (Stufe B): Tiled-JSON-Map in `tmp_path`, via TILED_LOAD
geladen (run_gb mit `base=tmp_path`). Frueher via `call_builtin` gegen die
Python-Impl (in Phase 8 geloescht).
"""
import json
import pytest

from drachenhauch.errors import DHRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _basic_map_dict(width=4, height=3, tile_w=16, tile_h=16,
                    tile_data=None, with_solid=False):
    if tile_data is None:
        tile_data = [0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1]
    tiles_meta = []
    if with_solid:
        tiles_meta = [
            {"id": 0, "properties": [{"name": "solid", "type": "bool", "value": True}]},
            {"id": 1, "properties": [{"name": "solid", "type": "bool", "value": False}]},
        ]
    return {
        "type": "map", "width": width, "height": height,
        "tilewidth": tile_w, "tileheight": tile_h,
        "tilesets": [{
            "firstgid": 1, "name": "tiles", "tilewidth": tile_w, "tileheight": tile_h,
            "columns": 4, "image": "tiles.png", "imagewidth": 64, "imageheight": 32,
            "tiles": tiles_meta,
        }],
        "layers": [{
            "type": "tilelayer", "name": "ground", "width": width, "height": height,
            "data": tile_data, "visible": True, "opacity": 1.0,
        }],
    }


def _load(run_gb, tmp_path, mapdict, body, name="level.json"):
    (tmp_path / name).write_text(json.dumps(mapdict), encoding="utf-8")
    src = (f'IMPORT "tiled"\nDIM m AS TILED_MAP\nm = TILED_LOAD("{name}")\n' + body)
    return _lines(run_gb(src, base=tmp_path))


def _load_raw(run_gb, tmp_path, text, body, name="x.json"):
    (tmp_path / name).write_text(text, encoding="utf-8")
    src = (f'IMPORT "tiled"\nDIM m AS TILED_MAP\nm = TILED_LOAD("{name}")\n' + body)
    return _lines(run_gb(src, base=tmp_path))


# --- Basis-Laden ---------------------------------------------------

def test_load_basic_map(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _basic_map_dict(),
                "PRINT TILED_WIDTH(m)\nPRINT TILED_HEIGHT(m)\n"
                "PRINT TILED_TILE_WIDTH(m)\nPRINT TILED_TILE_HEIGHT(m)\n")
    assert out == ["4", "3", "16", "16"]


def test_load_missing_file(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="nicht gefunden"):
        run_gb('IMPORT "tiled"\nDIM m AS TILED_MAP\nm = TILED_LOAD("missing.json")\n',
               base=tmp_path)


def test_load_invalid_json(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="Lesefehler"):
        _load_raw(run_gb, tmp_path, "{not json}", "", name="bad.json")


def test_load_wrong_type(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="erwartet eine Tiled-Map"):
        _load_raw(run_gb, tmp_path, json.dumps({"type": "tileset"}), "", name="wrong.json")


# --- Layers --------------------------------------------------------

def test_layer_count(run_gb, tmp_path):
    assert _load(run_gb, tmp_path, _basic_map_dict(),
                 "PRINT TILED_LAYER_COUNT(m)\n") == ["1"]


def test_layer_name_and_type(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _basic_map_dict(),
                "PRINT TILED_LAYER_NAME(m, 0)\nPRINT TILED_LAYER_TYPE(m, 0)\n")
    assert out == ["ground", "tile"]


def test_layer_index_lookup(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _basic_map_dict(),
                'PRINT TILED_LAYER_INDEX(m, "ground")\n'
                'PRINT TILED_LAYER_INDEX(m, "no_such")\n')
    assert out == ["0", "-1"]


def test_layer_idx_out_of_bounds(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="ausserhalb"):
        _load(run_gb, tmp_path, _basic_map_dict(), "PRINT TILED_LAYER_NAME(m, 99)\n")


# --- Tile-Access --------------------------------------------------

def test_tile_at_returns_gid(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _basic_map_dict(),
                "PRINT TILED_TILE_AT(m, 0, 0, 0)\n"
                "PRINT TILED_TILE_AT(m, 0, 1, 1)\n"
                "PRINT TILED_TILE_AT(m, 0, 0, 2)\n")
    assert out == ["0", "1", "1"]


def test_tile_at_out_of_bounds_returns_0(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _basic_map_dict(),
                "PRINT TILED_TILE_AT(m, 0, -1, 0)\n"
                "PRINT TILED_TILE_AT(m, 0, 99, 0)\n"
                "PRINT TILED_TILE_AT(m, 0, 0, 99)\n")
    assert out == ["0", "0", "0"]


# --- Tile-Properties -----------------------------------------------

def test_tile_property_solid(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _basic_map_dict(with_solid=True),
                'PRINT TILED_TILE_PROP_BOOL(m, 1, "solid")\n'
                'PRINT TILED_TILE_PROP_BOOL(m, 2, "solid")\n'
                'PRINT TILED_TILE_PROP_BOOL(m, 1, "damage")\n')
    assert out == ["TRUE", "FALSE", "FALSE"]


def test_tile_has_prop(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _basic_map_dict(with_solid=True),
                'PRINT TILED_TILE_HAS_PROP(m, 1, "solid")\n'
                'PRINT TILED_TILE_HAS_PROP(m, 1, "nonexistent")\n')
    assert out == ["TRUE", "FALSE"]


def test_tile_property_int_float_string(run_gb, tmp_path):
    d = _basic_map_dict()
    d["tilesets"][0]["tiles"] = [{"id": 0, "properties": [
        {"name": "damage", "type": "int", "value": 5},
        {"name": "speed", "type": "float", "value": 1.5},
        {"name": "team", "type": "string", "value": "red"},
    ]}]
    out = _load(run_gb, tmp_path, d,
                'PRINT TILED_TILE_PROP_INT(m, 1, "damage")\n'
                'PRINT TILED_TILE_PROP_FLOAT(m, 1, "speed")\n'
                'PRINT TILED_TILE_PROP_STRING(m, 1, "team")\n')
    assert out == ["5", "1.5", "red"]


# --- Object Layers -------------------------------------------------

def _map_with_objects():
    d = _basic_map_dict()
    d["layers"].append({
        "type": "objectgroup", "name": "spawns", "objects": [
            {"id": 1, "name": "player_start", "type": "spawn",
             "x": 100.0, "y": 200.0, "width": 16.0, "height": 16.0,
             "properties": [{"name": "team", "type": "string", "value": "blue"},
                            {"name": "hp", "type": "int", "value": 100}]},
            {"id": 2, "name": "enemy", "type": "mob",
             "x": 300.0, "y": 220.0, "width": 32.0, "height": 32.0, "properties": []},
        ]})
    return d


def test_object_layer(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _map_with_objects(),
                'PRINT TILED_OBJECT_COUNT(m, "spawns")\n'
                'PRINT TILED_OBJECT_NAME(m, "spawns", 0)\n'
                'PRINT TILED_OBJECT_TYPE(m, "spawns", 0)\n'
                'PRINT TILED_OBJECT_X(m, "spawns", 0)\n'
                'PRINT TILED_OBJECT_Y(m, "spawns", 0)\n'
                'PRINT TILED_OBJECT_WIDTH(m, "spawns", 0)\n'
                'PRINT TILED_OBJECT_HEIGHT(m, "spawns", 0)\n'
                'PRINT TILED_OBJECT_PROP_STRING(m, "spawns", 0, "team")\n'
                'PRINT TILED_OBJECT_PROP_INT(m, "spawns", 0, "hp")\n'
                'PRINT TILED_OBJECT_NAME(m, "spawns", 1)\n'
                'PRINT TILED_OBJECT_TYPE(m, "spawns", 1)\n')
    assert out == ["2", "player_start", "spawn", "100.0", "200.0", "16.0",
                   "16.0", "blue", "100", "enemy", "mob"]


def test_object_layer_not_found(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="nicht gefunden"):
        _load(run_gb, tmp_path, _basic_map_dict(),
              'PRINT TILED_OBJECT_COUNT(m, "no_such")\n')


def test_object_idx_out_of_bounds(run_gb, tmp_path):
    d = _basic_map_dict()
    d["layers"].append({"type": "objectgroup", "name": "obj", "objects": []})
    with pytest.raises(DHRuntimeError, match="ausserhalb"):
        _load(run_gb, tmp_path, d, 'PRINT TILED_OBJECT_NAME(m, "obj", 99)\n')


# --- Tileset-Access -----------------------------------------------

def test_tileset_count_and_image(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _basic_map_dict(),
                "PRINT TILED_TILESET_COUNT(m)\nPRINT TILED_TILESET_IMAGE(m, 0)\n")
    assert out[0] == "1"
    assert out[1].endswith("tiles.png")


def test_tileset_firstgid(run_gb, tmp_path):
    assert _load(run_gb, tmp_path, _basic_map_dict(),
                 "PRINT TILED_TILESET_FIRSTGID(m, 0)\n") == ["1"]


# --- Type-Checking ------------------------------------------------

def test_non_map_argument_errors(run_gb):
    with pytest.raises(DHRuntimeError, match="TILED_MAP"):
        run_gb('IMPORT "tiled"\nPRINT TILED_WIDTH("not a map")\n')


# --- TILE_SET + Object-Layer-Fehler -------------------------------

def test_tile_set_changes_gid(run_gb, tmp_path):
    out = _load(run_gb, tmp_path, _basic_map_dict(),
                "PRINT TILED_TILE_AT(m, 0, 0, 2)\n"
                "PRINT TILED_TILE_SET(m, 0, 0, 2, 0)\n"
                "PRINT TILED_TILE_AT(m, 0, 0, 2)\n")
    assert out == ["1", "1", "0"]


def test_tile_set_out_of_bounds_silent(run_gb, tmp_path):
    assert _load(run_gb, tmp_path, _basic_map_dict(),
                 "PRINT TILED_TILE_SET(m, 0, -1, 0, 5)\n") == ["0"]


def test_tile_set_on_object_layer_errors(run_gb, tmp_path):
    d = _basic_map_dict()
    d["layers"].append({"type": "objectgroup", "name": "obj", "objects": []})
    with pytest.raises(DHRuntimeError, match="kein Tile-Layer"):
        _load(run_gb, tmp_path, d, "PRINT TILED_TILE_SET(m, 1, 0, 0, 1)\n")


def test_tile_at_on_object_layer_errors(run_gb, tmp_path):
    d = _basic_map_dict()
    d["layers"].append({"type": "objectgroup", "name": "obj", "objects": []})
    with pytest.raises(DHRuntimeError, match="kein Tile-Layer"):
        _load(run_gb, tmp_path, d, "PRINT TILED_TILE_AT(m, 1, 0, 0)\n")
