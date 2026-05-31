"""Tests fuer das tiled-Modul (Tiled-Map-Loader)."""
import json
from pathlib import Path

import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_tiled():
    assert load_module("tiled")


# --- Test-Fixtures: minimale Tiled-JSON-Maps -----------------------

def _basic_map_dict(width=4, height=3, tile_w=16, tile_h=16,
                    tile_data=None, with_solid=False):
    """Erzeugt ein dict im Tiled-JSON-Schema."""
    if tile_data is None:
        # 0 = leer, 1 = solid (wenn Property gesetzt), 2 = "decor"
        tile_data = [
            0, 0, 0, 0,
            0, 1, 0, 0,
            1, 1, 1, 1,
        ]
    tiles_meta = []
    if with_solid:
        tiles_meta = [
            {"id": 0, "properties": [
                {"name": "solid", "type": "bool", "value": True},
            ]},
            # Tile-ID 1 (decor) explizit nicht solid
            {"id": 1, "properties": [
                {"name": "solid", "type": "bool", "value": False},
            ]},
        ]
    return {
        "type": "map",
        "width": width,
        "height": height,
        "tilewidth": tile_w,
        "tileheight": tile_h,
        "tilesets": [{
            "firstgid": 1,
            "name": "tiles",
            "tilewidth": tile_w,
            "tileheight": tile_h,
            "columns": 4,
            "image": "tiles.png",
            "imagewidth": 64,
            "imageheight": 32,
            "tiles": tiles_meta,
        }],
        "layers": [{
            "type": "tilelayer",
            "name": "ground",
            "width": width,
            "height": height,
            "data": tile_data,
            "visible": True,
            "opacity": 1.0,
        }],
    }


@pytest.fixture
def basic_map(tmp_path, call_builtin):
    p = tmp_path / "level.json"
    p.write_text(json.dumps(_basic_map_dict()))
    return call_builtin("tiled_load", [str(p)])


# --- Basis-Laden ---------------------------------------------------

def test_load_basic_map(basic_map, call_builtin):
    assert call_builtin("tiled_width", [basic_map]) == 4
    assert call_builtin("tiled_height", [basic_map]) == 3
    assert call_builtin("tiled_tile_width", [basic_map]) == 16
    assert call_builtin("tiled_tile_height", [basic_map]) == 16


def test_load_missing_file(call_builtin, tmp_path):
    with pytest.raises(GBRuntimeError, match="nicht gefunden"):
        call_builtin("tiled_load", [str(tmp_path / "missing.json")])


def test_load_invalid_json(call_builtin, tmp_path):
    p = tmp_path / "bad.json"
    p.write_text("{not json}")
    with pytest.raises(GBRuntimeError, match="Lesefehler"):
        call_builtin("tiled_load", [str(p)])


def test_load_wrong_type(call_builtin, tmp_path):
    p = tmp_path / "wrong.json"
    p.write_text(json.dumps({"type": "tileset"}))
    with pytest.raises(GBRuntimeError, match="erwartet eine Tiled-Map"):
        call_builtin("tiled_load", [str(p)])


# --- Layers --------------------------------------------------------

def test_layer_count(basic_map, call_builtin):
    assert call_builtin("tiled_layer_count", [basic_map]) == 1


def test_layer_name_and_type(basic_map, call_builtin):
    assert call_builtin("tiled_layer_name", [basic_map, 0]) == "ground"
    assert call_builtin("tiled_layer_type", [basic_map, 0]) == "tile"


def test_layer_index_lookup(basic_map, call_builtin):
    assert call_builtin("tiled_layer_index", [basic_map, "ground"]) == 0
    assert call_builtin("tiled_layer_index", [basic_map, "no_such"]) == -1


def test_layer_idx_out_of_bounds(basic_map, call_builtin):
    with pytest.raises(GBRuntimeError, match="ausserhalb"):
        call_builtin("tiled_layer_name", [basic_map, 99])


# --- Tile-Access --------------------------------------------------

def test_tile_at_returns_gid(basic_map, call_builtin):
    # Tile-Daten: row 0 leer, row 1 hat solid bei (1,1), row 2 alle solid
    assert call_builtin("tiled_tile_at", [basic_map, 0, 0, 0]) == 0
    assert call_builtin("tiled_tile_at", [basic_map, 0, 1, 1]) == 1
    assert call_builtin("tiled_tile_at", [basic_map, 0, 0, 2]) == 1


def test_tile_at_out_of_bounds_returns_0(basic_map, call_builtin):
    """Out-of-bounds darf kein Error sein -- spart Sweep-Code IF-Checks."""
    assert call_builtin("tiled_tile_at", [basic_map, 0, -1, 0]) == 0
    assert call_builtin("tiled_tile_at", [basic_map, 0, 99, 0]) == 0
    assert call_builtin("tiled_tile_at", [basic_map, 0, 0, 99]) == 0


# --- Tile-Properties -----------------------------------------------

def test_tile_property_solid(tmp_path, call_builtin):
    p = tmp_path / "level.json"
    p.write_text(json.dumps(_basic_map_dict(with_solid=True)))
    m = call_builtin("tiled_load", [str(p)])
    # Tile mit ID 0 (also GID 1) hat solid=True
    assert call_builtin("tiled_tile_prop_bool", [m, 1, "solid"]) is True
    # Tile mit ID 1 (GID 2) hat solid=False
    assert call_builtin("tiled_tile_prop_bool", [m, 2, "solid"]) is False
    # Unbekannte Property gibt Default
    assert call_builtin("tiled_tile_prop_bool", [m, 1, "damage"]) is False


def test_tile_has_prop(tmp_path, call_builtin):
    p = tmp_path / "level.json"
    p.write_text(json.dumps(_basic_map_dict(with_solid=True)))
    m = call_builtin("tiled_load", [str(p)])
    assert call_builtin("tiled_tile_has_prop", [m, 1, "solid"]) is True
    assert call_builtin("tiled_tile_has_prop", [m, 1, "nonexistent"]) is False


def test_tile_property_int_float_string(tmp_path, call_builtin):
    m_data = _basic_map_dict()
    m_data["tilesets"][0]["tiles"] = [{
        "id": 0,
        "properties": [
            {"name": "damage", "type": "int", "value": 5},
            {"name": "speed",  "type": "float", "value": 1.5},
            {"name": "team",   "type": "string", "value": "red"},
        ],
    }]
    p = tmp_path / "props.json"
    p.write_text(json.dumps(m_data))
    m = call_builtin("tiled_load", [str(p)])
    assert call_builtin("tiled_tile_prop_int", [m, 1, "damage"]) == 5
    assert call_builtin("tiled_tile_prop_float", [m, 1, "speed"]) == 1.5
    assert call_builtin("tiled_tile_prop_string", [m, 1, "team"]) == "red"


# --- Object Layers -------------------------------------------------

def test_object_layer(tmp_path, call_builtin):
    data = _basic_map_dict()
    data["layers"].append({
        "type": "objectgroup",
        "name": "spawns",
        "objects": [
            {"id": 1, "name": "player_start", "type": "spawn",
             "x": 100.0, "y": 200.0, "width": 16.0, "height": 16.0,
             "properties": [
                 {"name": "team", "type": "string", "value": "blue"},
                 {"name": "hp",   "type": "int", "value": 100},
             ]},
            {"id": 2, "name": "enemy", "type": "mob",
             "x": 300.0, "y": 220.0, "width": 32.0, "height": 32.0,
             "properties": []},
        ],
    })
    p = tmp_path / "obj.json"
    p.write_text(json.dumps(data))
    m = call_builtin("tiled_load", [str(p)])
    assert call_builtin("tiled_object_count", [m, "spawns"]) == 2
    assert call_builtin("tiled_object_name", [m, "spawns", 0]) == "player_start"
    assert call_builtin("tiled_object_type", [m, "spawns", 0]) == "spawn"
    assert call_builtin("tiled_object_x", [m, "spawns", 0]) == 100.0
    assert call_builtin("tiled_object_y", [m, "spawns", 0]) == 200.0
    assert call_builtin("tiled_object_width", [m, "spawns", 0]) == 16.0
    assert call_builtin("tiled_object_height", [m, "spawns", 0]) == 16.0
    # Object-Properties
    assert call_builtin("tiled_object_prop_string", [m, "spawns", 0, "team"]) == "blue"
    assert call_builtin("tiled_object_prop_int", [m, "spawns", 0, "hp"]) == 100
    # 2. Objekt
    assert call_builtin("tiled_object_name", [m, "spawns", 1]) == "enemy"
    assert call_builtin("tiled_object_type", [m, "spawns", 1]) == "mob"


def test_object_layer_not_found(basic_map, call_builtin):
    with pytest.raises(GBRuntimeError, match="nicht gefunden"):
        call_builtin("tiled_object_count", [basic_map, "no_such"])


def test_object_idx_out_of_bounds(tmp_path, call_builtin):
    data = _basic_map_dict()
    data["layers"].append({
        "type": "objectgroup", "name": "obj", "objects": [],
    })
    p = tmp_path / "x.json"
    p.write_text(json.dumps(data))
    m = call_builtin("tiled_load", [str(p)])
    with pytest.raises(GBRuntimeError, match="ausserhalb"):
        call_builtin("tiled_object_name", [m, "obj", 99])


# --- Tileset-Access -----------------------------------------------

def test_tileset_count_and_image(basic_map, call_builtin, tmp_path):
    assert call_builtin("tiled_tileset_count", [basic_map]) == 1
    img = call_builtin("tiled_tileset_image", [basic_map, 0])
    # Pfad ist absolutiert relativ zur Map-Datei
    assert img.endswith("tiles.png")


def test_tileset_firstgid(basic_map, call_builtin):
    assert call_builtin("tiled_tileset_firstgid", [basic_map, 0]) == 1


# --- Type-Checking ------------------------------------------------

def test_non_map_argument_errors(call_builtin):
    with pytest.raises(TypeMismatchError, match="TILED_MAP"):
        call_builtin("tiled_width", ["not a map"])


# --- Tilelayer auf Object-Op -> Error -----------------------------

def test_tile_set_changes_gid(basic_map, call_builtin):
    """TILED_TILE_SET schreibt ein neues GID; nachfolgendes TILE_AT
    liefert es zurueck. Wichtig fuer Pickups / zerstoerbare Tiles."""
    # Initial: row 2 ist 1,1,1,1
    assert call_builtin("tiled_tile_at", [basic_map, 0, 0, 2]) == 1
    old = call_builtin("tiled_tile_set", [basic_map, 0, 0, 2, 0])
    assert old == 1
    assert call_builtin("tiled_tile_at", [basic_map, 0, 0, 2]) == 0


def test_tile_set_out_of_bounds_silent(basic_map, call_builtin):
    """OOB-Set wird ignoriert, kein Crash, kein Error."""
    rv = call_builtin("tiled_tile_set", [basic_map, 0, -1, 0, 5])
    assert rv == 0


def test_tile_set_on_object_layer_errors(tmp_path, call_builtin):
    """Object-Layer hat keine Tiles -> Set ist Error."""
    data = _basic_map_dict()
    data["layers"].append({
        "type": "objectgroup", "name": "obj", "objects": [],
    })
    p = tmp_path / "x.json"
    p.write_text(json.dumps(data))
    m = call_builtin("tiled_load", [str(p)])
    with pytest.raises(GBRuntimeError, match="kein Tile-Layer"):
        call_builtin("tiled_tile_set", [m, 1, 0, 0, 1])


def test_tile_at_on_object_layer_errors(tmp_path, call_builtin):
    data = _basic_map_dict()
    data["layers"].append({
        "type": "objectgroup", "name": "obj", "objects": [],
    })
    p = tmp_path / "x.json"
    p.write_text(json.dumps(data))
    m = call_builtin("tiled_load", [str(p)])
    with pytest.raises(GBRuntimeError, match="kein Tile-Layer"):
        call_builtin("tiled_tile_at", [m, 1, 0, 0])
