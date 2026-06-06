"""Tests fuer das Tilemap-Editor-Datenmodell (Qt-frei).

Kern-Garantie: was der Editor exportiert, liest die native Runtime `gbrt`
(`IMPORT "tiled"` + `TILED_LOAD` + Accessors) ohne Fehler und mit identischen
Werten zurueck. Frueher gegen das Python-`tiled`-Modul (in Phase 8 geloescht).
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

from gamebasic.tilemap import TileMapDoc, TileLayer
from gamebasic.tilemap.document import coerce_prop, MapObject, ObjectLayer


_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "gb_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_GBRT = _find_gbrt()


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _run_in(tmp_path, body):
    """GB-Programm im Verzeichnis `tmp_path` ueber gbrt ausfuehren (damit
    relative TILED_LOAD-Pfade die dort gespeicherte JSON finden)."""
    if _GBRT is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    fd = tmp_path / "_t.gb"
    fd.write_text(body, encoding="utf-8")
    r = subprocess.run([str(_GBRT), "run", str(fd)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    assert r.returncode == 0, f"gbrt run Exit {r.returncode}: {r.stderr}"
    return _lines(r.stdout.replace("\r\n", "\n"))


def _check_compiles(tmp_path, src):
    """`gbrt --check` auf `src`; gibt die Fehler-Diagnosen zurueck (leer = ok)."""
    if _GBRT is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    fd = tmp_path / "_check.gb"
    fd.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_GBRT), "--check", str(fd)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    diags = json.loads(r.stdout or "[]")
    return [d for d in diags if d.get("severity") == "error"]


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
    doc.save_json(str(tmp_path / "level.json"))

    out = _run_in(tmp_path,
        'IMPORT "tiled"\nDIM m AS TILED_MAP\nm = TILED_LOAD("level.json")\n'
        "PRINT TILED_WIDTH(m)\nPRINT TILED_HEIGHT(m)\nPRINT TILED_TILE_WIDTH(m)\n"
        "PRINT TILED_LAYER_COUNT(m)\n"
        "PRINT TILED_LAYER_NAME(m, 0)\nPRINT TILED_LAYER_NAME(m, 1)\n"
        "PRINT TILED_TILE_AT(m, 0, 0, 0)\nPRINT TILED_TILE_AT(m, 0, 5, 3)\n"
        "PRINT TILED_TILE_AT(m, 1, 2, 1)\nPRINT TILED_TILE_AT(m, 0, 1, 1)\n"
        'PRINT TILED_TILE_PROP_BOOL(m, 1, "solid")\n'
        'PRINT TILED_TILE_PROP_INT(m, 1, "damage")\n'
        'PRINT TILED_TILE_PROP_STRING(m, 4, "name")\n'
        "PRINT TILED_TILESET_COUNT(m)\nPRINT TILED_TILESET_FIRSTGID(m, 0)\n")
    assert out == ["6", "4", "16", "2", "Boden", "Deko",
                   "1", "4", "3", "0", "TRUE", "5", "coin", "1", "1"]


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


# --------------------------------------------------------------- Multi-Tileset

def _build_multi(tmp_path):
    doc = TileMapDoc(6, 4, 16, 16)
    a = tmp_path / "a.png"; a.write_bytes(b"\x89PNG\r\n")
    b = tmp_path / "b.png"; b.write_bytes(b"\x89PNG\r\n")
    doc.add_tileset(str(a), 64, 32)   # 8 Tiles, firstgid 1
    doc.add_tileset(str(b), 32, 32)   # 4 Tiles, firstgid 9
    return doc


def test_add_tileset_assigns_firstgids(tmp_path):
    doc = _build_multi(tmp_path)
    assert [t.firstgid for t in doc.tilesets] == [1, 9]
    assert [t.tile_count for t in doc.tilesets] == [8, 4]
    assert doc.active_tileset == 1            # zuletzt hinzugefuegt
    # gid-Aufloesung
    assert doc.gid_to_tileset(1) == (0, 0)
    assert doc.gid_to_tileset(8) == (0, 7)
    assert doc.gid_to_tileset(9) == (1, 0)
    assert doc.gid_to_tileset(12) == (1, 3)
    assert doc.gid_to_tileset(99) == (-1, -1)
    assert doc.local_to_gid(1, 2) == 11


def test_facade_targets_active_tileset(tmp_path):
    doc = _build_multi(tmp_path)
    doc.active_tileset = 0
    assert doc.columns == 4 and doc.tile_count == 8
    doc.active_tileset = 1
    assert doc.columns == 2 and doc.tile_count == 4
    # Properties landen am aktiven Tileset
    doc.set_property(0, "solid", "true", "bool")
    assert doc.tilesets[1].tile_properties[0]["solid"] is True
    assert 0 not in doc.tilesets[0].tile_properties


def test_remove_tileset_reassigns_firstgids(tmp_path):
    doc = _build_multi(tmp_path)
    doc.add_tileset(str(tmp_path / "a.png"), 64, 32)   # 3. Tileset, firstgid 13
    assert [t.firstgid for t in doc.tilesets] == [1, 9, 13]
    assert doc.remove_tileset(1) is True
    assert [t.firstgid for t in doc.tilesets] == [1, 9]   # neu vergeben


def test_multi_tileset_roundtrip_through_loader(tmp_path):
    """Zwei Tilesets + platzierte Tiles aus beiden -> TILED_LOAD liest beide."""
    doc = _build_multi(tmp_path)
    doc.active_tileset = 0
    doc.layers[0].set(0, 0, 1)              # Tileset 0
    doc.active_tileset = 1
    doc.set_property(0, "solid", "true", "bool")   # ts1 local 0 = gid 9
    doc.layers[0].set(1, 0, 9)              # Tileset 1, local 0
    doc.layers[0].set(2, 0, 12)             # Tileset 1, local 3
    doc.save_json(str(tmp_path / "multi.json"))

    out = _run_in(tmp_path,
        'IMPORT "tiled"\nDIM m AS TILED_MAP\nm = TILED_LOAD("multi.json")\n'
        "PRINT TILED_TILESET_COUNT(m)\n"
        "PRINT TILED_TILESET_FIRSTGID(m, 0)\nPRINT TILED_TILESET_FIRSTGID(m, 1)\n"
        "PRINT TILED_TILE_AT(m, 0, 0, 0)\nPRINT TILED_TILE_AT(m, 0, 1, 0)\n"
        "PRINT TILED_TILE_AT(m, 0, 2, 0)\n"
        'PRINT TILED_TILE_PROP_BOOL(m, 9, "solid")\n')
    assert out == ["2", "1", "9", "1", "9", "12", "TRUE"]


def test_multi_tileset_editor_load_select(tmp_path, monkeypatch):
    """Headless: Map mit 2 Tilesets oeffnen, Tileset im Editor wechseln."""
    monkeypatch.setenv("QT_QPA_PLATFORM", "offscreen")
    try:
        from PySide6.QtWidgets import QApplication
        from gamebasic.tilemapeditor_qt import TileMapEditor
    except Exception:
        pytest.skip("PySide6 nicht verfuegbar")
    QApplication.instance() or QApplication([])
    doc = _build_multi(tmp_path)
    doc.layers[0].set(0, 0, 9)
    path = str(tmp_path / "m.json")
    doc.save_json(path)

    ed = TileMapEditor(tmp_path)
    ed.doc = TileMapDoc.load_json(path)
    ed._load_tileset_pixmaps()
    ed._refresh_views()
    assert len(ed.doc.tilesets) == 2
    assert ed.ts_combo.count() == 2
    # Tileset 0 aktiv -> Palette-gid = firstgid 1
    ed.doc.active_tileset = 0
    ed._on_palette_select(2)
    assert ed.canvas.current_gid == 3
    # Auf Tileset 1 wechseln -> gid = firstgid 9
    ed._on_tileset_changed(1)
    ed._on_palette_select(0)
    assert ed.canvas.current_gid == 9


# --------------------------------------------------------------- Object-Layer

def _build_with_objects(tmp_path):
    doc = TileMapDoc(6, 4, 16, 16)
    img = tmp_path / "tiles.png"
    img.write_bytes(b"\x89PNG\r\n")
    doc.set_tileset(str(img), 64, 32)
    doc.layers[0].set(0, 0, 1)
    idx = doc.add_object_layer("Spawns")
    ol = doc.layers[idx]
    p = MapObject(48, 32, name="player", otype="spawn")   # Punkt (w=h=0)
    p.set_property("hp", "100", "int")
    ol.add(p)
    r = MapObject(16, 0, 32, 48, name="zone", otype="trigger")  # Rechteck
    r.set_property("active", "true", "bool")
    ol.add(r)
    return doc


def test_object_layer_model_basics():
    doc = TileMapDoc(4, 4, 16, 16)
    i = doc.add_object_layer("Obj")
    assert i == 1 and isinstance(doc.layers[1], ObjectLayer)
    ol = doc.layers[1]
    p = MapObject(10, 20)
    assert p.is_point()
    r = MapObject(0, 0, 16, 16)
    assert not r.is_point()
    ol.add(p); ol.add(r)
    assert ol.object_at(10, 20) is p          # Punkt-Trefferzone
    assert ol.object_at(8, 8) is r            # im Rechteck
    assert ol.object_at(200, 200) is None


def test_object_layer_resize_keeps_pixel_coords():
    doc = TileMapDoc(8, 8, 16, 16)
    doc.add_object_layer("Obj")
    doc.layers[1].add(MapObject(100, 64, name="x"))
    doc.resize(4, 4)                          # Karte schrumpft
    assert isinstance(doc.layers[1], ObjectLayer)
    o = doc.layers[1].objects[0]
    assert o.x == 100 and o.y == 64 and o.name == "x"


def test_object_layer_to_tiled_dict_shape(tmp_path):
    doc = _build_with_objects(tmp_path)
    d = doc.to_tiled_dict(str(tmp_path / "level.json"))
    assert len(d["layers"]) == 2
    og = d["layers"][1]
    assert og["type"] == "objectgroup" and og["name"] == "Spawns"
    assert len(og["objects"]) == 2
    o0 = og["objects"][0]
    assert o0["name"] == "player" and o0["type"] == "spawn"
    assert o0["x"] == 48 and o0["y"] == 32 and o0.get("point") is True
    assert {"name": "hp", "type": "int", "value": 100} in o0["properties"]
    assert d["nextobjectid"] == 3            # 2 Objekte vergeben -> naechste 3


def test_object_layer_roundtrip_through_tiled_loader(tmp_path):
    """Editor-Export mit Object-Layer -> TILED_LOAD -> TILED_OBJECT_* lesbar."""
    doc = _build_with_objects(tmp_path)
    doc.save_json(str(tmp_path / "level.json"))

    out = _run_in(tmp_path,
        'IMPORT "tiled"\nDIM m AS TILED_MAP\nm = TILED_LOAD("level.json")\n'
        "PRINT TILED_LAYER_COUNT(m)\nPRINT TILED_LAYER_TYPE(m, 1)\n"
        'PRINT TILED_OBJECT_COUNT(m, "Spawns")\n'
        'PRINT TILED_OBJECT_NAME(m, "Spawns", 0)\nPRINT TILED_OBJECT_TYPE(m, "Spawns", 0)\n'
        'PRINT TILED_OBJECT_X(m, "Spawns", 0)\nPRINT TILED_OBJECT_Y(m, "Spawns", 0)\n'
        'PRINT TILED_OBJECT_WIDTH(m, "Spawns", 0)\n'
        'PRINT TILED_OBJECT_PROP_INT(m, "Spawns", 0, "hp")\n'
        'PRINT TILED_OBJECT_NAME(m, "Spawns", 1)\n'
        'PRINT TILED_OBJECT_WIDTH(m, "Spawns", 1)\nPRINT TILED_OBJECT_HEIGHT(m, "Spawns", 1)\n'
        'PRINT TILED_OBJECT_PROP_BOOL(m, "Spawns", 1, "active")\n')
    assert out == ["2", "object", "2", "player", "spawn", "48.0", "32.0",
                   "0.0", "100", "zone", "32.0", "48.0", "TRUE"]


def test_object_layer_save_load_json_roundtrip(tmp_path):
    doc = _build_with_objects(tmp_path)
    path = str(tmp_path / "level.json")
    doc.save_json(path)
    doc2 = TileMapDoc.load_json(path)
    assert len(doc2.layers) == 2
    assert isinstance(doc2.layers[1], ObjectLayer)
    objs = doc2.layers[1].objects
    assert [o.name for o in objs] == ["player", "zone"]
    assert objs[0].is_point()
    assert objs[0].properties["hp"] == 100
    assert objs[1].width == 32 and objs[1].height == 48
    assert objs[1].properties["active"] is True


def test_gb_code_compiles(tmp_path):
    """Der exportierte GB-Code muss in gbrt lexen+parsen+kompilieren."""
    doc = _build_sample(tmp_path)
    code = doc.gb_code(str(tmp_path / "level.json"))
    assert _check_compiles(tmp_path, code) == []


def test_gb_code_compiles_with_object_layer(tmp_path):
    """Auch mit Object-Layer bleibt der exportierte GB-Code kompilierbar."""
    doc = _build_with_objects(tmp_path)
    code = doc.gb_code(str(tmp_path / "level.json"))
    assert _check_compiles(tmp_path, code) == []


# --------------------------------------------------------------- Editor-UI

# --------------------------------------------------------------- Regionen (Select)

def test_get_stamp_clear_region():
    doc = TileMapDoc(6, 6, 16, 16)
    l = doc.layers[0]
    for x in range(2):
        for y in range(2):
            l.set(x + 1, y + 1, 10 + y * 2 + x)
    block = doc.get_region(0, 1, 1, 2, 2)
    assert block == [[10, 11], [12, 13]]
    # An anderer Stelle stempeln
    assert doc.stamp_region(0, 3, 3, block) is True
    assert doc.get_region(0, 3, 3, 2, 2) == [[10, 11], [12, 13]]
    # Region leeren
    assert doc.clear_region(0, 1, 1, 2, 2) is True
    assert doc.get_region(0, 1, 1, 2, 2) == [[0, 0], [0, 0]]


def test_stamp_region_clips_out_of_bounds():
    doc = TileMapDoc(3, 3, 16, 16)
    # 2x2-Block an der Ecke -> nur (2,2) liegt drin
    doc.stamp_region(0, 2, 2, [[5, 6], [7, 8]])
    assert doc.layers[0].get(2, 2) == 5
    assert doc.layers[0].get(3, 2) == 0      # OOB ignoriert


def test_region_ops_ignore_object_layer():
    doc = TileMapDoc(4, 4, 16, 16)
    doc.add_object_layer("Obj")
    oi = len(doc.layers) - 1
    assert doc.get_region(oi, 0, 0, 2, 2) == []
    assert doc.stamp_region(oi, 0, 0, [[1]]) is False


def test_editor_select_copy_paste(tmp_path, monkeypatch):
    """Headless: Tile-Auswahl kopieren + woanders einfuegen, mit Undo."""
    monkeypatch.setenv("QT_QPA_PLATFORM", "offscreen")
    try:
        from PySide6.QtWidgets import QApplication
        from gamebasic.tilemapeditor_qt import TileMapEditor, TOOL_SELECT
    except Exception:
        pytest.skip("PySide6 nicht verfuegbar")
    QApplication.instance() or QApplication([])
    ed = TileMapEditor(tmp_path)
    c = ed.canvas
    c.doc.layers[0].set(0, 0, 5)
    c.doc.layers[0].set(1, 0, 6)
    ed._set_tool(TOOL_SELECT)
    # Auswahl (0,0)-(1,0) simulieren
    c._sel_rect = (0, 0, 1, 0)
    assert c.copy_selection() is True
    assert c.tile_clipboard == [[5, 6]]
    # Hover auf (0,2) -> dort einfuegen (Auswahl-Ursprung hat aber Vorrang)
    c._sel_rect = None
    c._hover = (0, 2)
    assert c.paste_clipboard() is True
    assert c.doc.layers[0].get(0, 2) == 5
    assert c.doc.layers[0].get(1, 2) == 6
    # Undo macht das Einfuegen rueckgaengig
    ed._undo()
    assert c.doc.layers[0].get(0, 2) == 0


def test_editor_select_cut_and_delete(tmp_path, monkeypatch):
    monkeypatch.setenv("QT_QPA_PLATFORM", "offscreen")
    try:
        from PySide6.QtWidgets import QApplication
        from gamebasic.tilemapeditor_qt import TileMapEditor, TOOL_SELECT
    except Exception:
        pytest.skip("PySide6 nicht verfuegbar")
    QApplication.instance() or QApplication([])
    ed = TileMapEditor(tmp_path)
    c = ed.canvas
    c.doc.layers[0].set(2, 2, 9)
    ed._set_tool(TOOL_SELECT)
    c._sel_rect = (2, 2, 2, 2)
    assert c.cut_selection() is True
    assert c.tile_clipboard == [[9]]
    assert c.doc.layers[0].get(2, 2) == 0    # ausgeschnitten -> leer
    ed._undo()
    assert c.doc.layers[0].get(2, 2) == 9    # Cut rueckgaengig


def test_editor_object_layer_undo_redo(tmp_path, monkeypatch):
    """Headless: Object-Layer im Editor anlegen, Objekt + Undo/Redo + Delete."""
    monkeypatch.setenv("QT_QPA_PLATFORM", "offscreen")
    try:
        from PySide6.QtWidgets import QApplication
        from gamebasic.tilemapeditor_qt import TileMapEditor
    except Exception:
        pytest.skip("PySide6 nicht verfuegbar")
    QApplication.instance() or QApplication([])
    ed = TileMapEditor(tmp_path)
    ed._add_object_layer()
    li = ed.canvas.active_layer
    ol = ed.doc.layers[li]
    assert isinstance(ol, ObjectLayer)
    before = json.loads(json.dumps([]))      # leere Liste als Vorher-Snapshot
    import copy as _c
    ol.add(MapObject(24, 24, name="hero", otype="spawn"))
    ed._on_obj_committed(li, before, _c.deepcopy(ol.objects))
    assert len(ol.objects) == 1
    ed._undo(); assert len(ed.doc.layers[li].objects) == 0
    ed._redo(); assert len(ed.doc.layers[li].objects) == 1
    ed.canvas.selected_obj = ed.doc.layers[li].objects[0]
    ed.canvas.delete_selected()
    assert len(ed.doc.layers[li].objects) == 0
    ed._undo(); assert len(ed.doc.layers[li].objects) == 1
