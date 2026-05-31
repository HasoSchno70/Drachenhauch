"""Tests fuer tile_collide-Modul (Box-vs-Tilemap-Sweep)."""
import json

import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_modules():
    assert load_module("tiled")
    assert load_module("tile_collide")


def _make_map(tmp_path, call_builtin, tile_data, width=8, height=4,
              tile_w=16, tile_h=16, with_solid=True):
    """Bau eine simple 8x4-Map mit gegebenem tile_data und optional
    solid-Properties."""
    tiles_meta = []
    if with_solid:
        # Tile-ID 0 (= GID 1) ist solid
        tiles_meta = [{
            "id": 0,
            "properties": [{"name": "solid", "type": "bool", "value": True}],
        }]
    data = {
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
        }],
    }
    p = tmp_path / "level.json"
    p.write_text(json.dumps(data))
    return call_builtin("tiled_load", [str(p)])


# --- TILE_IS_SOLID ------------------------------------------------

def test_solid_with_property(tmp_path, call_builtin):
    # Tile bei (2, 1) ist solid, andere nicht
    data = [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ]
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    assert call_builtin("tile_is_solid", [m, 0, 2, 1]) is True
    assert call_builtin("tile_is_solid", [m, 0, 0, 0]) is False


def test_solid_fallback_no_property(tmp_path, call_builtin):
    """Kein Tileset hat solid-Property: jeder GID > 0 ist solid (Convention
    fuer dedizierte Collision-Layer)."""
    data = [0] * 32
    data[10] = 1  # ein Tile gesetzt
    m = _make_map(tmp_path, call_builtin, data, with_solid=False)
    # Tile-ID 0 (GID 1) wird ohne Property als solid behandelt
    assert call_builtin("tile_is_solid", [m, 0, 2, 1]) is True
    assert call_builtin("tile_is_solid", [m, 0, 0, 0]) is False


def test_solid_out_of_bounds_is_solid(tmp_path, call_builtin):
    """Welt-Rand blockiert -- gleiche Konvention wie viele Tile-Engines."""
    data = [0] * 32
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    assert call_builtin("tile_is_solid", [m, 0, -1, 0]) is True
    assert call_builtin("tile_is_solid", [m, 0, 99, 0]) is True


# --- TILE_AT_PIXEL -----------------------------------------------

def test_tile_at_pixel(tmp_path, call_builtin):
    data = [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 2, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ]
    m = _make_map(tmp_path, call_builtin, data, with_solid=False)
    # Tile-Groesse 16x16. (32, 16) ist Tile (2, 1) = GID 1
    assert call_builtin("tile_at_pixel", [m, 0, 32, 16]) == 1
    assert call_builtin("tile_at_pixel", [m, 0, 48, 16]) == 2
    assert call_builtin("tile_at_pixel", [m, 0, 0, 0]) == 0


# --- TILE_SWEEP_X ------------------------------------------------

def test_sweep_x_no_collision(tmp_path, call_builtin):
    """Box bewegt sich frei in einer leeren Welt."""
    data = [0] * 32
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    result = call_builtin("tile_sweep_x", [m, 0, 10.0, 10.0, 8.0, 8.0, 5.0])
    new_x, hit = result
    assert new_x == 15.0
    assert hit is False


def test_sweep_x_hits_wall_right(tmp_path, call_builtin):
    """Wand bei tile (3, 0). Box rechte Kante stoppt bei 48."""
    # Wand bei tile-x=3, ganze Hoehe
    data = [
        0, 0, 0, 1, 0, 0, 0, 0,
        0, 0, 0, 1, 0, 0, 0, 0,
        0, 0, 0, 1, 0, 0, 0, 0,
        0, 0, 0, 1, 0, 0, 0, 0,
    ]
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    # Box bei x=20, w=10, will sich um 50 nach rechts bewegen.
    # Wand-linke-Kante bei pixel 48. Box rechte Kante = x + w = 30.
    # Nach Sweep: new_x = 48 - 10 = 38, hit=TRUE.
    result = call_builtin("tile_sweep_x", [m, 0, 20.0, 16.0, 10.0, 16.0, 50.0])
    new_x, hit = result
    assert hit is True
    assert new_x == 38.0


def test_sweep_x_hits_wall_left(tmp_path, call_builtin):
    """Wand bei tile (3, 0). Box bewegt sich nach links und stoppt."""
    data = [
        0, 0, 0, 1, 0, 0, 0, 0,
        0, 0, 0, 1, 0, 0, 0, 0,
        0, 0, 0, 1, 0, 0, 0, 0,
        0, 0, 0, 1, 0, 0, 0, 0,
    ]
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    # Box bei x=80, w=10, will sich um -50 nach links bewegen.
    # Wand-rechte-Kante bei pixel 64 (= (3+1)*16). Box linke Kante = x.
    # Nach Sweep: new_x = 64, hit=TRUE.
    result = call_builtin("tile_sweep_x", [m, 0, 80.0, 16.0, 10.0, 16.0, -50.0])
    new_x, hit = result
    assert hit is True
    assert new_x == 64.0


def test_sweep_x_zero_delta(tmp_path, call_builtin):
    data = [0] * 32
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    new_x, hit = call_builtin("tile_sweep_x",
                              [m, 0, 10.0, 10.0, 8.0, 8.0, 0.0])
    assert new_x == 10.0
    assert hit is False


# --- TILE_SWEEP_Y ------------------------------------------------

def test_sweep_y_falls_onto_ground(tmp_path, call_builtin):
    """Klassisches Platformer-Pattern: Box faellt durch Luft und stoppt
    auf dem Boden-Tile."""
    # Boden bei row 3 (y=48..64)
    data = [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        1, 1, 1, 1, 1, 1, 1, 1,
    ]
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    # Box (10, 20, 8, 16) -- mitten in der Luft. dy = 30 (nach unten).
    # Boden-Top bei pixel 48. Box-Unter-Kante muss bei 48 landen.
    # -> new_y = 48 - 16 = 32, hit=TRUE.
    new_y, hit = call_builtin("tile_sweep_y",
                              [m, 0, 10.0, 20.0, 8.0, 16.0, 30.0])
    assert hit is True
    assert new_y == 32.0


def test_sweep_y_hits_ceiling(tmp_path, call_builtin):
    """Box springt nach oben und stoesst gegen Decke."""
    # Decke bei row 0 (y=0..16)
    data = [
        1, 1, 1, 1, 1, 1, 1, 1,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ]
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    # Box bei (10, 40, 8, 8) springt nach oben mit dy = -40.
    # Decke-Unter-Kante bei pixel 16. Box-Top muss bei 16 landen.
    # -> new_y = 16, hit=TRUE.
    new_y, hit = call_builtin("tile_sweep_y",
                              [m, 0, 10.0, 40.0, 8.0, 8.0, -40.0])
    assert hit is True
    assert new_y == 16.0


def test_sweep_y_falls_freely(tmp_path, call_builtin):
    """Box faellt durch leere Welt -- kein Hit."""
    data = [0] * 32
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    # Map ist 4 Tiles hoch = 64 Pixel.
    # Box bei (10, 10, 8, 8). dy=20 nach unten.
    # Out-of-bounds (Welt-Rand) blockiert -- aber 30 < 64, also keine Kollision.
    new_y, hit = call_builtin("tile_sweep_y",
                              [m, 0, 10.0, 10.0, 8.0, 8.0, 20.0])
    assert hit is False
    assert new_y == 30.0


def test_sweep_y_stops_at_world_edge(tmp_path, call_builtin):
    """Box faellt aus der Welt heraus -- Welt-Rand stoppt sie."""
    data = [0] * 32  # alles leer
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    # Map ist 4*16 = 64 px hoch.
    # Box bei (10, 50, 8, 8). dy=20 -> wuerde bei 70 landen, ausserhalb.
    # Welt-Rand-Tile bei ty=4 (also y=64..) blockiert.
    # Box-Unter-Kante muss bei 64 landen -> new_y = 64 - 8 = 56.
    new_y, hit = call_builtin("tile_sweep_y",
                              [m, 0, 10.0, 50.0, 8.0, 8.0, 20.0])
    assert hit is True
    assert new_y == 56.0


def test_sweep_diagonal_via_two_axes(tmp_path, call_builtin):
    """Diagonal-Bewegung via separate Achsen-Sweeps -- klassisches
    Platformer-Pattern."""
    # L-foermige Wand: Wand rechts bei x=3, Boden bei y=3
    data = [
        0, 0, 0, 1, 0, 0, 0, 0,
        0, 0, 0, 1, 0, 0, 0, 0,
        0, 0, 0, 1, 0, 0, 0, 0,
        1, 1, 1, 1, 0, 0, 0, 0,
    ]
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    # Box bei (20, 20, 8, 8). Bewege diagonal dx=50, dy=50.
    # X-Sweep zuerst: stoppt an Wand bei x=48-8=40
    new_x, hit_x = call_builtin("tile_sweep_x",
                                [m, 0, 20.0, 20.0, 8.0, 8.0, 50.0])
    assert hit_x is True
    assert new_x == 40.0
    # Dann Y-Sweep mit neuem X: Boden bei y=48-8=40
    new_y, hit_y = call_builtin("tile_sweep_y",
                                [m, 0, new_x, 20.0, 8.0, 8.0, 50.0])
    assert hit_y is True
    assert new_y == 40.0


# --- Argument-Validation ------------------------------------------

def test_sweep_invalid_layer(tmp_path, call_builtin):
    """Object-Layer kann nicht fuer Collision genutzt werden."""
    data = {
        "type": "map", "width": 4, "height": 4,
        "tilewidth": 16, "tileheight": 16,
        "tilesets": [{
            "firstgid": 1, "name": "t",
            "tilewidth": 16, "tileheight": 16,
            "columns": 1, "image": "x.png",
            "imagewidth": 16, "imageheight": 16,
        }],
        "layers": [{"type": "objectgroup", "name": "obj", "objects": []}],
    }
    p = tmp_path / "x.json"
    p.write_text(json.dumps(data))
    m = call_builtin("tiled_load", [str(p)])
    with pytest.raises(GBRuntimeError, match="kein Tile-Layer"):
        call_builtin("tile_sweep_x", [m, 0, 0.0, 0.0, 8.0, 8.0, 5.0])


def test_sweep_zero_size_box_errors(tmp_path, call_builtin):
    data = [0] * 32
    m = _make_map(tmp_path, call_builtin, data, with_solid=True)
    with pytest.raises(GBRuntimeError, match="> 0"):
        call_builtin("tile_sweep_x", [m, 0, 10.0, 10.0, 0.0, 8.0, 5.0])
