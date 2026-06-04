"""Tests fuer das Tilemap-Editor-Datenmodell (Qt-frei).

Kern-Garantie: was der Editor exportiert, liest das `tiled`-Modul
(`TILED_LOAD` + Accessors) ohne Fehler und mit identischen Werten zurueck.
"""
import json

import pytest

from gamebasic.tilemap import TileMapDoc, TileLayer
from gamebasic.tilemap.document import coerce_prop
from gamebasic.modules import tiled as T


def call(fn, *args):
    """Ruft einen @builtin-gewrappten Loader/Accessor mit Positional-Args."""
    return fn(list(args))


# --------------------------------------------------------------- Basis

def test_new_doc_has_one_layer_zero_filled():
    doc = TileMapDoc(10, 8, 16, 16)
    assert len(doc.layers) == 1
    assert doc.layers[0].width == 10 and doc.layers[0].height == 8
    assert all(g == 0 for g in doc.layers[0].tiles)
    assert len(doc.layers[0].tiles) == 80


def test_layer_set_get_and_bounds():
    l = TileLayer("x", 4, 4)
    assert l.set(1, 2, 7) is True
    assert l.get(1, 2) == 7
    assert l.set(1, 2, 7) is False          # unveraendert
    assert l.get(-1, 0) == 0                 # OOB harmlos
    assert l.set(99, 99, 5) is False


def test_set_tileset_computes_columns_and_count():
    doc = TileMapDoc(5, 5, 16, 16)
    doc.set_tileset("tiles.png", 64, 48)     # 4 cols x 3 rows
    assert doc.columns == 4
    assert doc.tile_count == 12


def test_tile_src_rect():
    doc = TileMapDoc(5, 5, 16, 16)
    doc.set_tileset("tiles.png", 64, 48)
    assert doc.tile_src_rect(0) == (0, 0, 16, 16)
    assert doc.tile_src_rect(5) == (16, 16, 16, 16)   # col1,row1


def test_coerce_prop_types():
    assert coerce_prop("true", "bool") is True
    assert coerce_prop("x", "bool") is True
    assert coerce_prop("0", "bool") is False
    assert coerce_prop("42", "int") == 42
    assert coerce_prop("3.5", "float") == 3.5
    assert coerce_prop(7, "string") == "7"


def test_flood_fill():
    doc = TileMapDoc(4, 4, 16, 16)
    n = doc.flood_fill(0, 0, 0, 9)
    assert n == 16                            # ganze leere Flaeche
    assert all(g == 9 for g in doc.layers[0].tiles)
    assert doc.flood_fill(0, 0, 0, 9) == 0    # schon 9 -> nichts


def test_resize_preserves_topleft():
    doc = TileMapDoc(4, 4, 16, 16)
    doc.layers[0].set(0, 0, 3)
    doc.layers[0].set(3, 3, 4)
    doc.resize(2, 2)
    assert doc.width == 2 and doc.layers[0].get(0, 0) == 3
    assert doc.layers[0].get(3, 3) == 0       # weggeschnitten (jetzt OOB)


def test_layer_ops():
    doc = TileMapDoc(4, 4, 16, 16)
    i = doc.add_layer("Deko")
    assert i == 1 and len(doc.layers) == 2
    j = doc.move_layer(1, -1)
    assert j == 0 and doc.layers[0].name == "Deko"
    doc.remove_layer(0)
    assert len(doc.layers) == 1
    doc.remove_layer(0)                        # letzte Layer bleibt
    assert len(doc.layers) == 1


# --------------------------------------------------------------- Roundtrip

def _build_sample(tmp_path):
    doc = TileMapDoc(6, 4, 16, 16)
    img = tmp_path / "tiles.png"
    img.write_bytes(b"\x89PNG\r\n")           # Dateiinhalt egal fuer TILED_LOAD
    doc.set_tileset(str(img), 64, 32)         # 4x2 = 8 tiles
    doc.layers[0].set(0, 0, 1)
    doc.layers[0].set(5, 3, 4)
    doc.add_layer("Deko")
    doc.layers[1].set(2, 1, 3)
    doc.set_property(0, "solid", "true", "bool")
    doc.set_property(0, "damage", "5", "int")
    doc.set_property(3, "name", "coin", "string")
    return doc


def test_to_tiled_dict_shape(tmp_path):
    doc = _build_sample(tmp_path)
    d = doc.to_tiled_dict(str(tmp_path / "level.json"))
    assert d["type"] == "map"
    assert d["width"] == 6 and d["tilewidth"] == 16
    assert len(d["layers"]) == 2
    assert d["layers"][0]["type"] == "tilelayer"
    assert d["layers"][0]["data"][0] == 1
    ts = d["tilesets"][0]
    assert ts["firstgid"] == 1 and ts["columns"] == 4
    assert ts["image"] == "tiles.png"         # relativ zur Map
    # Properties am Tileset
    by_id = {t["id"]: t for t in ts["tiles"]}
    assert 0 in by_id and 3 in by_id


def test_roundtrip_through_tiled_loader(tmp_path):
    """Editor-Export -> TILED_LOAD -> identische Werte."""
    doc = _build_sample(tmp_path)
    path = str(tmp_path / "level.json")
    doc.save_json(path)

    m = call(T._b_tiled_load, path)
    assert call(T._b_width, m) == 6
    assert call(T._b_height, m) == 4
    assert call(T._b_tile_w, m) == 16
    assert call(T._b_layer_count, m) == 2
    assert call(T._b_layer_name, m, 0) == "Boden"
    assert call(T._b_layer_name, m, 1) == "Deko"
    # Tile-Werte
    assert call(T._b_tile_at, m, 0, 0, 0) == 1
    assert call(T._b_tile_at, m, 0, 5, 3) == 4
    assert call(T._b_tile_at, m, 1, 2, 1) == 3
    assert call(T._b_tile_at, m, 0, 1, 1) == 0
    # Per-Tile-Properties (gid 1 -> local id 0)
    assert call(T._b_tile_prop_bool, m, 1, "solid") is True
    assert call(T._b_tile_prop_int, m, 1, "damage") == 5
    assert call(T._b_tile_prop_string, m, 4, "name") == "coin"
    # Tileset
    assert call(T._b_ts_count, m) == 1
    assert call(T._b_ts_firstgid, m, 0) == 1


def test_save_load_json_roundtrip(tmp_path):
    """Editor-eigener Load liest den eigenen Save deckungsgleich zurueck."""
    doc = _build_sample(tmp_path)
    path = str(tmp_path / "level.json")
    doc.save_json(path)

    doc2 = TileMapDoc.load_json(path)
    assert doc2.width == doc.width and doc2.height == doc.height
    assert doc2.columns == doc.columns
    assert len(doc2.layers) == 2
    assert doc2.layers[0].get(0, 0) == 1
    assert doc2.layers[1].get(2, 1) == 3
    assert doc2.properties_of(0)["solid"] is True
    assert doc2.properties_of(0)["damage"] == 5
    assert doc2.properties_of(3)["name"] == "coin"


def test_gb_code_compiles(tmp_path):
    """Der exportierte GB-Code muss lexen+parsen+kompilieren."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler
    from gamebasic.preprocess import process

    doc = _build_sample(tmp_path)
    code = doc.gb_code(str(tmp_path / "level.json"))
    prepped, _ = process(code, tmp_path, file_label="<tilemap>")
    ast = Parser(Lexer(prepped).tokenize()).parse()
    Compiler().compile(ast)                   # wirft bei Fehler
