"""Tiled-Map-Loader fuer GameBasic.

Liest `.json`-Maps, die in [Tiled](https://www.mapeditor.org/) erstellt
wurden. Tiled speichert nativ als `.tmx` (XML), kann aber auch
direkt JSON schreiben (File -> Save As -> JSON Map). Wir
unterstuetzen nur JSON; TMX kann spaeter via xml.etree dazukommen,
falls jemand das brauchen sollte.

Workflow:
  1. Level in Tiled bauen (Tilesets, Layer, Objekte mit Custom-Properties)
  2. Save As JSON
  3. Im Spiel:  map = TILED_LOAD("levels/level1.json")
  4. Layer iterieren, Objekte spawnen, Tile-Properties pruefen

Object-Layers sind die typische Stelle, an der das Spiel
Spawn-Punkte, Trigger, Door-Targets etc. holt. Tile-Properties
geben jedem Tile-Type "Charakter" -- z.B. `solid: true` fuer das
Kollisionssystem (siehe `tile_collide`-Modul).

API:
    TILED_LOAD(path)                          -> TILED_MAP
    TILED_WIDTH(m) / TILED_HEIGHT(m)          -> INTEGER  (in Tiles)
    TILED_TILE_WIDTH(m) / TILED_TILE_HEIGHT(m) -> INTEGER (Pixel pro Tile)
    TILED_LAYER_COUNT(m)                      -> INTEGER
    TILED_LAYER_NAME(m, idx)                  -> STRING
    TILED_LAYER_TYPE(m, idx)                  -> "tile" | "object" | "image"
    TILED_LAYER_INDEX(m, name$)               -> INTEGER (-1 wenn nicht da)
    TILED_TILE_AT(m, layer_idx, tx, ty)       -> INTEGER (GID, 0 = leer)
    TILED_TILE_PROP_BOOL(m, gid, key$)        -> BOOLEAN
    TILED_TILE_PROP_INT(m, gid, key$)         -> INTEGER (0 wenn nicht da)
    TILED_TILE_PROP_FLOAT(m, gid, key$)       -> FLOAT
    TILED_TILE_PROP_STRING(m, gid, key$)      -> STRING
    TILED_OBJECT_COUNT(m, layer_name$)        -> INTEGER
    TILED_OBJECT_NAME(m, layer_name$, idx)    -> STRING
    TILED_OBJECT_TYPE(m, layer_name$, idx)    -> STRING (Tiled "type"-Feld)
    TILED_OBJECT_X/Y/WIDTH/HEIGHT(...)        -> FLOAT
    TILED_OBJECT_PROP_*(layer_name$, idx, key$) -> wie TILE_PROP
    TILED_TILESET_IMAGE(m, idx)               -> STRING (Pfad, relativ zur Map)
    TILED_TILESET_COUNT(m)                    -> INTEGER

Solid-Detection: `tile_collide` liest `TILED_TILE_PROP_BOOL(map, gid, "solid")`.
Wer keine Per-Tile-Properties hat: jeder GID != 0 in einer Collision-Layer
gilt als solid (Fallback).
"""
from __future__ import annotations

import json as _json
import os as _os
from pathlib import Path

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type

# Natives Rust-Backend fuer Flood-Fill (einzige Bulk-Op mit lohnender
# Beschleunigung; siehe Kommentar bei TILED_FLOOD_FILL).
try:
    from ..gb_native import tilemap_flood_fill as _tilemap_flood_fill  # type: ignore
except Exception:
    _tilemap_flood_fill = None


# --- Datentypen ---------------------------------------------------

class _TiledTileset:
    __slots__ = ("first_gid", "name", "tile_w", "tile_h",
                 "columns", "image", "image_w", "image_h",
                 "tile_properties")

    def __init__(self):
        self.first_gid = 1
        self.name = ""
        self.tile_w = 0
        self.tile_h = 0
        self.columns = 0
        self.image = ""
        self.image_w = 0
        self.image_h = 0
        # local_tile_id (gid - first_gid) -> {key: value}
        self.tile_properties: dict = {}


class _TiledLayer:
    __slots__ = ("name", "type", "width", "height",
                 "tiles", "objects", "image",
                 "visible", "opacity", "_obj_by_name")

    def __init__(self):
        self.name = ""
        self.type = "tile"             # "tile" | "object" | "image"
        self.width = 0
        self.height = 0
        self.tiles: list = []           # list[int] (GIDs, row-major)
        self.objects: list = []         # list[_TiledObject]
        self.image = ""                 # Pfad bei image-Layer
        self.visible = True
        self.opacity = 1.0
        # Index Objekt-Name -> Liste von Indizes, fuer schnellen Lookup
        self._obj_by_name: dict = {}


class _TiledObject:
    __slots__ = ("name", "type", "x", "y", "width", "height",
                 "rotation", "gid", "properties")

    def __init__(self):
        self.name = ""
        self.type = ""
        self.x = 0.0
        self.y = 0.0
        self.width = 0.0
        self.height = 0.0
        self.rotation = 0.0
        self.gid = 0                # 0 = kein Tile-Objekt
        self.properties: dict = {}  # key -> value


class _TiledMap:
    """Geladene Tiled-Map. Praesentiert sich wie ein opaker Handle gegenueber
    GB; alle Zugriffe gehen ueber TILED_*-Builtins."""
    # __weakref__ erlaubt schwache Referenzen (tile_collide cached Solid-Masken
    # in einer WeakKeyDictionary, damit Eintraege beim GC der Map automatisch
    # verschwinden -- sonst kann id()-Reuse stale Masken liefern).
    __slots__ = ("path", "width", "height", "tile_w", "tile_h",
                 "tilesets", "layers", "_layer_by_name", "__weakref__")

    def __init__(self):
        self.path = ""
        self.width = 0
        self.height = 0
        self.tile_w = 0
        self.tile_h = 0
        self.tilesets: list = []          # list[_TiledTileset], sortiert nach first_gid
        self.layers: list = []            # list[_TiledLayer]
        self._layer_by_name: dict = {}    # name -> index

    def __repr__(self):
        return (f"<TILED_MAP {self.width}x{self.height} tiles, "
                f"{len(self.layers)} layers>")

    # --- Tile-Property-Lookup ---
    def _tileset_for_gid(self, gid: int):
        """Welcher Tileset gehoert dieser GID? Lineare Suche; Tilesets sind
        nach first_gid sortiert. Bei typischen 1-3 Tilesets vernachlaessigbar."""
        if gid <= 0:
            return (None, 0)
        best = None
        for ts in self.tilesets:
            if ts.first_gid <= gid:
                best = ts
            else:
                break
        if best is None:
            return (None, 0)
        return (best, gid - best.first_gid)

    def tile_property(self, gid: int, key: str):
        """Liefert die Per-Tile-Property `key` fuer das angegebene GID,
        oder None wenn nicht gesetzt."""
        ts, local = self._tileset_for_gid(gid)
        if ts is None:
            return None
        props = ts.tile_properties.get(local)
        if props is None:
            return None
        return props.get(key)


register_type("tiled_map", _TiledMap)


# --- JSON-Loader --------------------------------------------------

def _parse_properties(prop_list):
    """Tiled-Properties sind eine Liste von {name, type, value}. Wir mappen
    auf ein flaches dict[name -> typed value]."""
    out: dict = {}
    if not isinstance(prop_list, list):
        return out
    for p in prop_list:
        if not isinstance(p, dict):
            continue
        name = p.get("name")
        if not isinstance(name, str):
            continue
        # Typkonvertierung. Tiled liefert die Typen oft schon korrekt im JSON,
        # aber wir normalisieren defensiv.
        val = p.get("value")
        ptype = p.get("type", "string")
        if ptype == "bool":
            val = bool(val)
        elif ptype == "int":
            val = int(val) if val is not None else 0
        elif ptype == "float":
            val = float(val) if val is not None else 0.0
        elif ptype == "string":
            val = str(val) if val is not None else ""
        out[name] = val
    return out


def _parse_tileset(t: dict, base_dir: Path) -> _TiledTileset:
    ts = _TiledTileset()
    ts.first_gid = int(t.get("firstgid", 1))
    ts.name = str(t.get("name", ""))
    ts.tile_w = int(t.get("tilewidth", 0))
    ts.tile_h = int(t.get("tileheight", 0))
    ts.columns = int(t.get("columns", 0))
    img = t.get("image", "")
    if isinstance(img, str) and img:
        ts.image = _os.path.normpath(_os.path.join(str(base_dir), img))
    ts.image_w = int(t.get("imagewidth", 0))
    ts.image_h = int(t.get("imageheight", 0))
    # Per-Tile-Properties: Liste von {id, properties}
    for tile in t.get("tiles", []):
        if not isinstance(tile, dict):
            continue
        tile_id = tile.get("id")
        if not isinstance(tile_id, int):
            continue
        props = _parse_properties(tile.get("properties", []))
        if props:
            ts.tile_properties[tile_id] = props
    return ts


def _parse_object(o: dict) -> _TiledObject:
    obj = _TiledObject()
    obj.name = str(o.get("name", ""))
    obj.type = str(o.get("type") or o.get("class") or "")  # Tiled 1.9+: "class"
    obj.x = float(o.get("x", 0.0))
    obj.y = float(o.get("y", 0.0))
    obj.width = float(o.get("width", 0.0))
    obj.height = float(o.get("height", 0.0))
    obj.rotation = float(o.get("rotation", 0.0))
    obj.gid = int(o.get("gid", 0))
    obj.properties = _parse_properties(o.get("properties", []))
    return obj


def _parse_layer(layer: dict) -> _TiledLayer:
    l = _TiledLayer()
    l.name = str(layer.get("name", ""))
    raw_type = str(layer.get("type", "tilelayer"))
    if raw_type == "tilelayer":
        l.type = "tile"
        l.width = int(layer.get("width", 0))
        l.height = int(layer.get("height", 0))
        data = layer.get("data")
        if isinstance(data, list):
            l.tiles = [int(x) for x in data]
        elif isinstance(data, str):
            # base64-codiert -- nicht v1 unterstuetzt
            raise GBRuntimeError(
                "TILED_LOAD: base64-codierte Tile-Daten werden nicht unterstuetzt. "
                "In Tiled: Edit -> Preferences -> 'Store tile layer data as: CSV' "
                "und Map neu speichern."
            )
        else:
            l.tiles = []
    elif raw_type == "objectgroup":
        l.type = "object"
        for o in layer.get("objects", []):
            obj = _parse_object(o)
            l.objects.append(obj)
            l._obj_by_name.setdefault(obj.name, []).append(len(l.objects) - 1)
    elif raw_type == "imagelayer":
        l.type = "image"
        l.image = str(layer.get("image", ""))
    else:
        # Unbekannter Layer-Typ (group, ...) - als "image" markieren ohne Daten,
        # damit Iteration nicht crasht. v1: groups werden nicht expandiert.
        l.type = raw_type
    l.visible = bool(layer.get("visible", True))
    l.opacity = float(layer.get("opacity", 1.0))
    return l


@builtin("TILED_LOAD", arity=1, types=("str",))
def _b_tiled_load(path: str) -> _TiledMap:
    """Laedt eine Tiled-JSON-Map. Pfad relativ zum aktuellen
    Arbeitsverzeichnis."""
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = _json.load(f)
    except FileNotFoundError:
        raise GBRuntimeError(f"TILED_LOAD: Datei nicht gefunden: {path}")
    except Exception as exc:
        raise GBRuntimeError(f"TILED_LOAD: Lesefehler '{path}': {exc}")
    if not isinstance(data, dict):
        raise GBRuntimeError("TILED_LOAD: Manifest muss JSON-Object sein")
    if data.get("type") != "map":
        raise GBRuntimeError(
            "TILED_LOAD: erwartet eine Tiled-Map (type='map'), "
            f"erhalten type='{data.get('type')}'"
        )
    base_dir = Path(path).resolve().parent
    m = _TiledMap()
    m.path = str(path)
    m.width = int(data.get("width", 0))
    m.height = int(data.get("height", 0))
    m.tile_w = int(data.get("tilewidth", 0))
    m.tile_h = int(data.get("tileheight", 0))
    # Tilesets (referenzen koennten auch external .json sein, das ist Tiled-
    # "external tileset". v1: nur embedded.)
    for t in data.get("tilesets", []):
        if not isinstance(t, dict):
            continue
        if "source" in t:
            # External tileset -- erst auflösen (laden + mergen mit firstgid)
            ext_src = t["source"]
            ext_path = (base_dir / ext_src).resolve()
            try:
                with open(ext_path, "r", encoding="utf-8") as f:
                    ext_data = _json.load(f)
            except Exception as exc:
                raise GBRuntimeError(
                    f"TILED_LOAD: External-Tileset '{ext_src}' nicht ladbar: {exc}"
                )
            # firstgid kommt aus der Map, alles andere aus dem External
            ext_data["firstgid"] = int(t.get("firstgid", 1))
            ts = _parse_tileset(ext_data, ext_path.parent)
        else:
            ts = _parse_tileset(t, base_dir)
        m.tilesets.append(ts)
    # Nach first_gid sortieren (sollten sie eh schon sein, aber sicher)
    m.tilesets.sort(key=lambda ts: ts.first_gid)
    # Layer
    for i, layer in enumerate(data.get("layers", [])):
        if not isinstance(layer, dict):
            continue
        l = _parse_layer(layer)
        m.layers.append(l)
        m._layer_by_name[l.name] = i
    return m


# --- Accessor-Builtins -------------------------------------------

def _check_map(v, fn: str) -> _TiledMap:
    if not isinstance(v, _TiledMap):
        raise TypeMismatchError(f"{fn} erwartet TILED_MAP")
    return v


def _check_int(v, fn: str, label: str = "INTEGER") -> int:
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return v


def _check_str(v, fn: str, label: str = "STRING") -> str:
    if not isinstance(v, str):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return v


@builtin("TILED_WIDTH", arity=1)
def _b_width(m):
    return _check_map(m, "TILED_WIDTH").width


@builtin("TILED_HEIGHT", arity=1)
def _b_height(m):
    return _check_map(m, "TILED_HEIGHT").height


@builtin("TILED_TILE_WIDTH", arity=1)
def _b_tile_w(m):
    return _check_map(m, "TILED_TILE_WIDTH").tile_w


@builtin("TILED_TILE_HEIGHT", arity=1)
def _b_tile_h(m):
    return _check_map(m, "TILED_TILE_HEIGHT").tile_h


@builtin("TILED_LAYER_COUNT", arity=1)
def _b_layer_count(m):
    return len(_check_map(m, "TILED_LAYER_COUNT").layers)


def _get_layer_by_idx(m: _TiledMap, idx: int, fn: str) -> _TiledLayer:
    if idx < 0 or idx >= len(m.layers):
        raise GBRuntimeError(
            f"{fn}: Layer-Index {idx} ausserhalb [0..{len(m.layers) - 1}]"
        )
    return m.layers[idx]


def _get_layer_by_name(m: _TiledMap, name: str, fn: str) -> _TiledLayer:
    idx = m._layer_by_name.get(name, -1)
    if idx < 0:
        raise GBRuntimeError(f"{fn}: Layer '{name}' nicht gefunden")
    return m.layers[idx]


@builtin("TILED_LAYER_NAME", arity=2, types=("any", "int"))
def _b_layer_name(m, idx):
    m = _check_map(m, "TILED_LAYER_NAME")
    return _get_layer_by_idx(m, idx, "TILED_LAYER_NAME").name


@builtin("TILED_LAYER_TYPE", arity=2, types=("any", "int"))
def _b_layer_type(m, idx):
    m = _check_map(m, "TILED_LAYER_TYPE")
    return _get_layer_by_idx(m, idx, "TILED_LAYER_TYPE").type


@builtin("TILED_LAYER_INDEX", arity=2, types=("any", "str"))
def _b_layer_index(m, name):
    m = _check_map(m, "TILED_LAYER_INDEX")
    return m._layer_by_name.get(name, -1)


@builtin("TILED_LAYER_WIDTH", arity=2, types=("any", "int"))
def _b_layer_width(m, idx):
    m = _check_map(m, "TILED_LAYER_WIDTH")
    return _get_layer_by_idx(m, idx, "TILED_LAYER_WIDTH").width


@builtin("TILED_LAYER_HEIGHT", arity=2, types=("any", "int"))
def _b_layer_height(m, idx):
    m = _check_map(m, "TILED_LAYER_HEIGHT")
    return _get_layer_by_idx(m, idx, "TILED_LAYER_HEIGHT").height


@builtin("TILED_TILE_AT", arity=4, types=("any", "int", "int", "int"))
def _b_tile_at(m, layer_idx, tx, ty):
    """Liefert das GID an (tx, ty) im genannten Layer. 0 = leer.
    Out-of-Bounds liefert 0 (kein Crash) -- so kann der Caller
    Tile-Sweep ueber Map-Rand laufen lassen."""
    m = _check_map(m, "TILED_TILE_AT")
    layer = _get_layer_by_idx(m, layer_idx, "TILED_TILE_AT")
    if layer.type != "tile":
        raise GBRuntimeError(
            f"TILED_TILE_AT: Layer {layer_idx} ('{layer.name}') ist kein Tile-Layer"
        )
    if tx < 0 or ty < 0 or tx >= layer.width or ty >= layer.height:
        return 0
    return layer.tiles[ty * layer.width + tx]


@builtin("TILED_TILE_SET", arity=5,
         types=("any", "int", "int", "int", "int"))
def _b_tile_set(m, layer_idx, tx, ty, gid):
    """Setzt das Tile an (tx, ty) im Layer auf `gid`. 0 = Tile entfernen.

    Praktisch fuer Pickups (Coin -> 0 wenn aufgesammelt), zerstoerbare
    Bloecke (Mauer -> 0 nach Bombe) und dynamische Welt-Manipulation.
    Out-of-Bounds wird ignoriert (kein Error). Liefert die ALTE GID --
    so kannst du z.B. checken, ob ueberhaupt was da war.
    """
    m = _check_map(m, "TILED_TILE_SET")
    layer = _get_layer_by_idx(m, layer_idx, "TILED_TILE_SET")
    if layer.type != "tile":
        raise GBRuntimeError(
            f"TILED_TILE_SET: Layer {layer_idx} ('{layer.name}') ist kein Tile-Layer"
        )
    if tx < 0 or ty < 0 or tx >= layer.width or ty >= layer.height:
        return 0
    idx = ty * layer.width + tx
    old = layer.tiles[idx]
    layer.tiles[idx] = int(gid)
    return old


# --- Bulk-Tilemap-Operationen ------------------------------------
# Ganze-Layer- bzw. Regions-Ops fuer prozedurale Generierung, Editor-
# Werkzeuge und dynamische Welt-Manipulation. Hinweis: wie TILED_TILE_SET
# muten diese den Layer; ein bereits gebauter tile_collide-Collider (mit
# gecachter Solid-Maske) sieht die Aenderung nicht -- Bulk-Ops sind fuer
# Level-Setup/Generierung gedacht, nicht fuer den laufenden Frame.

def _get_tile_layer(m, layer_idx, fn):
    m = _check_map(m, fn)
    layer = _get_layer_by_idx(m, layer_idx, fn)
    if layer.type != "tile":
        raise GBRuntimeError(
            f"{fn}: Layer {layer_idx} ('{layer.name}') ist kein Tile-Layer"
        )
    return (m, layer)


@builtin("TILED_COUNT_GID", arity=3, types=("any", "int", "int"))
def _b_count_gid(m, layer_idx, gid):
    """Zaehlt, wie oft `gid` im Layer vorkommt. (Python: list.count ist
    bereits C-schnell -- kein nativer Pfad noetig.)"""
    m, layer = _get_tile_layer(m, layer_idx, "TILED_COUNT_GID")
    return layer.tiles.count(int(gid))


@builtin("TILED_FILL_RECT", arity=7,
         types=("any", "int", "int", "int", "int", "int", "int"))
def _b_fill_rect(m, layer_idx, tx, ty, w, h, gid):
    """Fuellt das Tile-Rechteck (tx, ty, w, h) mit `gid`. Out-of-Bounds wird
    geclamped. Liefert die Anzahl tatsaechlich geaenderter Tiles. (In-place
    ohne Vollkopie -- Python ist hier so schnell wie ein nativer Pfad.)"""
    m, layer = _get_tile_layer(m, layer_idx, "TILED_FILL_RECT")
    gid = int(gid)
    lw, lh = layer.width, layer.height
    x0 = max(0, tx)
    y0 = max(0, ty)
    x1 = min(lw, tx + w)
    y1 = min(lh, ty + h)
    tiles = layer.tiles
    n = 0
    for yy in range(y0, y1):
        base = yy * lw
        for xx in range(x0, x1):
            i = base + xx
            if tiles[i] != gid:
                tiles[i] = gid
                n += 1
    return n


@builtin("TILED_REPLACE", arity=4, types=("any", "int", "int", "int"))
def _b_replace(m, layer_idx, from_gid, to_gid):
    """Ersetzt alle `from_gid` durch `to_gid` im Layer. Liefert die Anzahl
    ersetzter Tiles. (Voller Scan in-place -- Python genuegt.)"""
    m, layer = _get_tile_layer(m, layer_idx, "TILED_REPLACE")
    from_gid = int(from_gid)
    to_gid = int(to_gid)
    tiles = layer.tiles
    n = 0
    for i, g in enumerate(tiles):
        if g == from_gid:
            tiles[i] = to_gid
            n += 1
    return n


def _flood_fill_py(tiles, width, height, tx, ty, gid):
    """Python-Fallback fuer Flood-Fill -- identisches Ergebnis wie der native
    Pfad (Flood-Fill ist deterministisch unabhaengig von der Stack-Reihenfolge)."""
    if tx < 0 or ty < 0 or tx >= width or ty >= height:
        return 0
    target = tiles[ty * width + tx]
    if target == gid:
        return 0
    stack = [(tx, ty)]
    n = 0
    while stack:
        cx, cy = stack.pop()
        if cx < 0 or cy < 0 or cx >= width or cy >= height:
            continue
        i = cy * width + cx
        if tiles[i] != target:
            continue
        tiles[i] = gid
        n += 1
        stack.append((cx + 1, cy))
        stack.append((cx - 1, cy))
        stack.append((cx, cy + 1))
        stack.append((cx, cy - 1))
    return n


@builtin("TILED_FLOOD_FILL", arity=5, types=("any", "int", "int", "int", "int"))
def _b_flood_fill(m, layer_idx, tx, ty, gid):
    """Bucket-Fill: ersetzt die zusammenhaengende (4-verbundene) Region der
    GID an (tx, ty) durch `gid`. Liefert die Anzahl gefuellter Tiles.
    Praktisch fuer prozedurale Generierung und Editor-Bucket-Fill.
    Nativ via gb_native (BFS in Rust) wenn gebaut, sonst Python."""
    m, layer = _get_tile_layer(m, layer_idx, "TILED_FLOOD_FILL")
    tx = int(tx)
    ty = int(ty)
    gid = int(gid)
    if _tilemap_flood_fill is not None:
        new_tiles, n = _tilemap_flood_fill(
            layer.tiles, layer.width, layer.height, tx, ty, gid
        )
        layer.tiles = new_tiles
        return n
    return _flood_fill_py(layer.tiles, layer.width, layer.height, tx, ty, gid)


def _tile_prop(m, gid, key, default, fn):
    val = _check_map(m, fn).tile_property(gid, key)
    if val is None:
        return default
    return val


@builtin("TILED_TILE_PROP_BOOL", arity=3, types=("any", "int", "str"))
def _b_tile_prop_bool(m, gid, key):
    v = _tile_prop(m, gid, key, False, "TILED_TILE_PROP_BOOL")
    return bool(v)


@builtin("TILED_TILE_PROP_INT", arity=3, types=("any", "int", "str"))
def _b_tile_prop_int(m, gid, key):
    v = _tile_prop(m, gid, key, 0, "TILED_TILE_PROP_INT")
    if isinstance(v, bool):
        return 1 if v else 0
    return int(v) if isinstance(v, (int, float)) else 0


@builtin("TILED_TILE_PROP_FLOAT", arity=3, types=("any", "int", "str"))
def _b_tile_prop_float(m, gid, key):
    v = _tile_prop(m, gid, key, 0.0, "TILED_TILE_PROP_FLOAT")
    if isinstance(v, bool):
        return 1.0 if v else 0.0
    return float(v) if isinstance(v, (int, float)) else 0.0


@builtin("TILED_TILE_PROP_STRING", arity=3, types=("any", "int", "str"))
def _b_tile_prop_string(m, gid, key):
    v = _tile_prop(m, gid, key, "", "TILED_TILE_PROP_STRING")
    return str(v) if v is not None else ""


@builtin("TILED_TILE_HAS_PROP", arity=3, types=("any", "int", "str"))
def _b_tile_has_prop(m, gid, key):
    return _check_map(m, "TILED_TILE_HAS_PROP").tile_property(gid, key) is not None


# --- Objects ----------------------------------------------------

@builtin("TILED_OBJECT_COUNT", arity=2, types=("any", "str"))
def _b_obj_count(m, layer_name):
    m = _check_map(m, "TILED_OBJECT_COUNT")
    layer = _get_layer_by_name(m, layer_name, "TILED_OBJECT_COUNT")
    if layer.type != "object":
        raise GBRuntimeError(
            f"TILED_OBJECT_COUNT: Layer '{layer_name}' ist kein Object-Layer"
        )
    return len(layer.objects)


def _get_obj(m, layer_name, idx, fn) -> _TiledObject:
    layer = _get_layer_by_name(m, layer_name, fn)
    if layer.type != "object":
        raise GBRuntimeError(f"{fn}: Layer '{layer_name}' ist kein Object-Layer")
    if idx < 0 or idx >= len(layer.objects):
        raise GBRuntimeError(
            f"{fn}: Object-Index {idx} ausserhalb [0..{len(layer.objects) - 1}]"
        )
    return layer.objects[idx]


@builtin("TILED_OBJECT_NAME", arity=3, types=("any", "str", "int"))
def _b_obj_name(m, layer_name, idx):
    return _get_obj(_check_map(m, "TILED_OBJECT_NAME"),
                    layer_name, idx, "TILED_OBJECT_NAME").name


@builtin("TILED_OBJECT_TYPE", arity=3, types=("any", "str", "int"))
def _b_obj_type(m, layer_name, idx):
    return _get_obj(_check_map(m, "TILED_OBJECT_TYPE"),
                    layer_name, idx, "TILED_OBJECT_TYPE").type


@builtin("TILED_OBJECT_X", arity=3, types=("any", "str", "int"))
def _b_obj_x(m, layer_name, idx):
    return _get_obj(_check_map(m, "TILED_OBJECT_X"),
                    layer_name, idx, "TILED_OBJECT_X").x


@builtin("TILED_OBJECT_Y", arity=3, types=("any", "str", "int"))
def _b_obj_y(m, layer_name, idx):
    return _get_obj(_check_map(m, "TILED_OBJECT_Y"),
                    layer_name, idx, "TILED_OBJECT_Y").y


@builtin("TILED_OBJECT_WIDTH", arity=3, types=("any", "str", "int"))
def _b_obj_w(m, layer_name, idx):
    return _get_obj(_check_map(m, "TILED_OBJECT_WIDTH"),
                    layer_name, idx, "TILED_OBJECT_WIDTH").width


@builtin("TILED_OBJECT_HEIGHT", arity=3, types=("any", "str", "int"))
def _b_obj_h(m, layer_name, idx):
    return _get_obj(_check_map(m, "TILED_OBJECT_HEIGHT"),
                    layer_name, idx, "TILED_OBJECT_HEIGHT").height


def _obj_prop(m, layer_name, idx, key, default, fn):
    obj = _get_obj(_check_map(m, fn), layer_name, idx, fn)
    return obj.properties.get(key, default)


@builtin("TILED_OBJECT_PROP_BOOL", arity=4, types=("any", "str", "int", "str"))
def _b_obj_prop_bool(m, layer_name, idx, key):
    v = _obj_prop(m, layer_name, idx, key, False, "TILED_OBJECT_PROP_BOOL")
    return bool(v)


@builtin("TILED_OBJECT_PROP_INT", arity=4, types=("any", "str", "int", "str"))
def _b_obj_prop_int(m, layer_name, idx, key):
    v = _obj_prop(m, layer_name, idx, key, 0, "TILED_OBJECT_PROP_INT")
    if isinstance(v, bool):
        return 1 if v else 0
    return int(v) if isinstance(v, (int, float)) else 0


@builtin("TILED_OBJECT_PROP_FLOAT", arity=4, types=("any", "str", "int", "str"))
def _b_obj_prop_float(m, layer_name, idx, key):
    v = _obj_prop(m, layer_name, idx, key, 0.0, "TILED_OBJECT_PROP_FLOAT")
    if isinstance(v, bool):
        return 1.0 if v else 0.0
    return float(v) if isinstance(v, (int, float)) else 0.0


@builtin("TILED_OBJECT_PROP_STRING", arity=4, types=("any", "str", "int", "str"))
def _b_obj_prop_string(m, layer_name, idx, key):
    v = _obj_prop(m, layer_name, idx, key, "", "TILED_OBJECT_PROP_STRING")
    return str(v) if v is not None else ""


# --- Tilesets ---------------------------------------------------

@builtin("TILED_TILESET_COUNT", arity=1)
def _b_ts_count(m):
    return len(_check_map(m, "TILED_TILESET_COUNT").tilesets)


@builtin("TILED_TILESET_IMAGE", arity=2, types=("any", "int"))
def _b_ts_image(m, idx):
    m = _check_map(m, "TILED_TILESET_IMAGE")
    if idx < 0 or idx >= len(m.tilesets):
        raise GBRuntimeError(
            f"TILED_TILESET_IMAGE: Index {idx} ausserhalb [0..{len(m.tilesets) - 1}]"
        )
    return m.tilesets[idx].image


@builtin("TILED_TILESET_FIRSTGID", arity=2, types=("any", "int"))
def _b_ts_firstgid(m, idx):
    m = _check_map(m, "TILED_TILESET_FIRSTGID")
    if idx < 0 or idx >= len(m.tilesets):
        raise GBRuntimeError(
            f"TILED_TILESET_FIRSTGID: Index {idx} ausserhalb"
        )
    return m.tilesets[idx].first_gid
