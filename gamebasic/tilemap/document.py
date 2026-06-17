"""Datenmodell des Tilemap-Editors + Tiled-JSON-Serialisierung.

Bewusst Qt-frei: so kann der Export-/Import-Pfad headless getestet werden
(Roundtrip durch das `tiled`-Modul, das diese JSONs wieder einliest).

Format-Konvention (passt 1:1 zu `gamebasic/modules/tiled.py`):
  - EIN eingebettetes Tileset, `firstgid` immer 1 -> lokale Tile-ID = gid - 1.
  - Tile-Daten als CSV-Liste von GIDs (kein base64), row-major, 0 = leer.
  - Per-Tile-Properties als Tiled-`{name,type,value}`-Liste am Tileset.
  - Orthogonale **Tile-Layer** UND **Object-Layer** (`objectgroup`): Objekte mit
    Name/Typ/Properties und Pixel-Koordinaten (Punkte = Spawn-Marker,
    Rechtecke = Trigger/Zonen). `TILED_LOAD` liest sie via `TILED_OBJECT_*`.

Der Editor speichert/laedt genau dieses Format (kein eigenes Projektformat),
damit der Kreis Editor -> `TILED_LOAD` -> Spiel geschlossen ist und die Map
auch im echten Tiled weiterbearbeitet werden kann.
"""
from __future__ import annotations

import copy
import json
import os
from pathlib import Path

# Erlaubte Property-Typen (Tiled-Namen). string ist Default.
PROP_TYPES = ("string", "int", "float", "bool")


def coerce_prop(value, ptype: str):
    """Bringt einen rohen Wert auf den GameBasic/Tiled-Typ `ptype`."""
    if ptype == "bool":
        if isinstance(value, str):
            return value.strip().lower() in ("1", "true", "wahr", "yes", "ja", "x")
        return bool(value)
    if ptype == "int":
        try:
            return int(float(value))
        except (TypeError, ValueError):
            return 0
    if ptype == "float":
        try:
            return float(value)
        except (TypeError, ValueError):
            return 0.0
    return "" if value is None else str(value)


class TileLayer:
    """Eine Tile-Schicht: row-major GID-Liste (0 = leer)."""

    __slots__ = ("name", "visible", "opacity", "tiles", "width", "height")

    def __init__(self, name: str, width: int, height: int, fill: int = 0):
        self.name = name
        self.visible = True
        self.opacity = 1.0
        self.width = width
        self.height = height
        self.tiles = [int(fill)] * (width * height)

    # --- Zellzugriff (Out-of-Bounds = harmlos) ---
    def get(self, x: int, y: int) -> int:
        if x < 0 or y < 0 or x >= self.width or y >= self.height:
            return 0
        return self.tiles[y * self.width + x]

    def set(self, x: int, y: int, gid: int) -> bool:
        """Setzt (x,y) auf gid. Liefert True, wenn sich etwas geaendert hat."""
        if x < 0 or y < 0 or x >= self.width or y >= self.height:
            return False
        i = y * self.width + x
        if self.tiles[i] == gid:
            return False
        self.tiles[i] = int(gid)
        return True

    def fill(self, gid: int) -> None:
        self.tiles = [int(gid)] * (self.width * self.height)

    def resized(self, new_w: int, new_h: int) -> "TileLayer":
        """Neue Layer-Kopie in neuer Groesse, Inhalt oben-links erhalten."""
        out = TileLayer(self.name, new_w, new_h)
        out.visible = self.visible
        out.opacity = self.opacity
        for y in range(min(self.height, new_h)):
            for x in range(min(self.width, new_w)):
                out.tiles[y * new_w + x] = self.tiles[y * self.width + x]
        return out


class MapObject:
    """Ein Objekt eines Object-Layers (Spawn-Punkt / Trigger / Zone).

    Koordinaten in **Pixeln** (wie Tiled). Ein Punkt-Objekt hat
    `width == height == 0` (Tiled: `"point": true`), ein Rechteck eine Groesse.
    """

    __slots__ = ("name", "type", "x", "y", "width", "height",
                 "properties", "property_types")

    def __init__(self, x: float, y: float, width: float = 0.0,
                 height: float = 0.0, name: str = "", otype: str = ""):
        self.name = str(name)
        self.type = str(otype)
        self.x = float(x)
        self.y = float(y)
        self.width = float(width)
        self.height = float(height)
        self.properties: dict = {}        # key -> typed value
        self.property_types: dict = {}    # key -> ptype

    def is_point(self) -> bool:
        return self.width == 0.0 and self.height == 0.0

    def set_property(self, key: str, value, ptype: str) -> None:
        if ptype not in PROP_TYPES:
            ptype = "string"
        self.properties[key] = coerce_prop(value, ptype)
        self.property_types[key] = ptype

    def remove_property(self, key: str) -> None:
        self.properties.pop(key, None)
        self.property_types.pop(key, None)


class ObjectLayer:
    """Eine Object-Schicht (`objectgroup`): geordnete Liste von `MapObject`."""

    __slots__ = ("name", "visible", "opacity", "objects")

    def __init__(self, name: str):
        self.name = str(name)
        self.visible = True
        self.opacity = 1.0
        self.objects: list[MapObject] = []

    def add(self, obj: MapObject) -> int:
        self.objects.append(obj)
        return len(self.objects) - 1

    def remove(self, obj: MapObject) -> None:
        if obj in self.objects:
            self.objects.remove(obj)

    def object_at(self, px: float, py: float) -> MapObject | None:
        """Oberstes Objekt, das den Pixel (px,py) enthaelt (Punkte mit
        kleiner Trefferzone). Iteriert von oben (zuletzt hinzugefuegt)."""
        for obj in reversed(self.objects):
            if obj.is_point():
                if abs(px - obj.x) <= 6 and abs(py - obj.y) <= 6:
                    return obj
            elif (obj.x <= px <= obj.x + obj.width
                    and obj.y <= py <= obj.y + obj.height):
                return obj
        return None

    def resized(self, _new_w: int, _new_h: int) -> "ObjectLayer":
        """Bei Karten-Resize bleiben Objekte an ihren Pixel-Positionen."""
        out = ObjectLayer(self.name)
        out.visible = self.visible
        out.opacity = self.opacity
        out.objects = copy.deepcopy(self.objects)
        return out


class Tileset:
    """Ein Tileset: Bild + Geometrie + `firstgid` + Per-Tile-Properties.

    GIDs eines Tilesets liegen in `[firstgid, firstgid + tile_count)`. Die
    lokale Tile-ID ist `gid - firstgid`.
    """

    __slots__ = ("name", "image", "image_abs", "image_w", "image_h",
                 "tile_w", "tile_h", "columns", "tile_count", "firstgid",
                 "tile_properties", "tile_property_types")

    def __init__(self, name: str, tile_w: int, tile_h: int):
        self.name = str(name)
        self.image = ""            # Pfad wie gespeichert (relativ zur Map)
        self.image_abs = ""        # absoluter Pfad zum Laden des Pixmaps
        self.image_w = 0
        self.image_h = 0
        self.tile_w = int(tile_w)
        self.tile_h = int(tile_h)
        self.columns = 0
        self.tile_count = 0
        self.firstgid = 1
        # lokale Tile-ID -> {key: typed value} / {key: ptype}
        self.tile_properties: dict[int, dict] = {}
        self.tile_property_types: dict[int, dict[str, str]] = {}

    def set_image(self, abs_path: str, image_w: int, image_h: int) -> None:
        self.image_abs = os.path.abspath(abs_path)
        self.image = abs_path
        self.image_w = int(image_w)
        self.image_h = int(image_h)
        self.columns = max(0, image_w // self.tile_w) if self.tile_w else 0
        rows = max(0, image_h // self.tile_h) if self.tile_h else 0
        self.tile_count = self.columns * rows

    def tile_src_rect(self, local_id: int):
        """Quell-Rechteck (sx, sy, w, h) eines lokalen Tiles im Bild."""
        if self.columns <= 0:
            return (0, 0, self.tile_w, self.tile_h)
        cx = local_id % self.columns
        cy = local_id // self.columns
        return (cx * self.tile_w, cy * self.tile_h, self.tile_w, self.tile_h)

    def contains_gid(self, gid: int) -> bool:
        return self.firstgid <= gid < self.firstgid + max(0, self.tile_count)


class TileMapDoc:
    """Vollstaendige Tilemap: Geometrie, N Tilesets, N Tile-/Object-Layer."""

    def __init__(self, width: int = 20, height: int = 15,
                 tile_w: int = 16, tile_h: int = 16):
        self.width = int(width)
        self.height = int(height)
        self.tile_w = int(tile_w)
        self.tile_h = int(tile_h)
        # Mehrere Tilesets (sortiert nach firstgid, erstes firstgid=1).
        self.tilesets: list[Tileset] = []
        self.active_tileset = 0
        self.layers: list[TileLayer | ObjectLayer] = [
            TileLayer("Boden", self.width, self.height)
        ]
        self.path = ""                 # zuletzt gespeicherter Map-Pfad
        self.dirty = False

    # ------------------------------------------------------ Tileset-Geometrie
    def recompute_firstgids(self) -> None:
        """Vergibt firstgids fortlaufend (erstes = 1)."""
        gid = 1
        for ts in self.tilesets:
            ts.firstgid = gid
            gid += max(0, ts.tile_count)

    @property
    def active_ts(self) -> Tileset | None:
        if 0 <= self.active_tileset < len(self.tilesets):
            return self.tilesets[self.active_tileset]
        return None

    def add_tileset(self, abs_path: str, image_w: int, image_h: int,
                    name: str | None = None) -> int:
        ts = Tileset(name or Path(abs_path).stem or "tileset",
                     self.tile_w, self.tile_h)
        ts.set_image(abs_path, image_w, image_h)
        self.tilesets.append(ts)
        self.recompute_firstgids()
        self.active_tileset = len(self.tilesets) - 1
        self.dirty = True
        return self.active_tileset

    def remove_tileset(self, idx: int) -> bool:
        """Entfernt ein Tileset. GIDs platzierter Tiles werden NICHT migriert
        -- daher nur erlaubt/sinnvoll, solange noch nichts darauf gemalt ist
        (UI prueft das). Liefert True bei Erfolg."""
        if not (0 <= idx < len(self.tilesets)):
            return False
        del self.tilesets[idx]
        self.recompute_firstgids()
        self.active_tileset = min(self.active_tileset, len(self.tilesets) - 1)
        self.active_tileset = max(0, self.active_tileset)
        self.dirty = True
        return True

    def set_tileset(self, abs_path: str, image_w: int, image_h: int) -> None:
        """Backward-compat / Single-Tileset-Workflow: ersetzt das ERSTE
        Tileset (oder legt es an)."""
        if self.tilesets:
            ts = self.tilesets[0]
            ts.name = Path(abs_path).stem or ts.name
            ts.set_image(abs_path, image_w, image_h)
            self.recompute_firstgids()
            self.active_tileset = 0
            self.dirty = True
        else:
            self.add_tileset(abs_path, image_w, image_h)

    def gid_to_tileset(self, gid: int) -> tuple[int, int]:
        """gid -> (tileset_index, local_id). (-1, -1) wenn keiner passt."""
        for i, ts in enumerate(self.tilesets):
            if ts.contains_gid(gid):
                return (i, gid - ts.firstgid)
        return (-1, -1)

    def local_to_gid(self, ts_index: int, local_id: int) -> int:
        if 0 <= ts_index < len(self.tilesets):
            return self.tilesets[ts_index].firstgid + local_id
        return 0

    # ------------------------------------------------------ Facade (aktiv)
    # Aelterer Code (Palette/Canvas) spricht das AKTIVE Tileset an.
    @property
    def columns(self) -> int:
        ts = self.active_ts
        return ts.columns if ts else 0

    @property
    def tile_count(self) -> int:
        ts = self.active_ts
        return ts.tile_count if ts else 0

    @property
    def tileset_image(self) -> str:
        ts = self.active_ts
        return ts.image if ts else ""

    @property
    def tileset_image_abs(self) -> str:
        ts = self.active_ts
        return ts.image_abs if ts else ""

    @property
    def tileset_image_w(self) -> int:
        ts = self.active_ts
        return ts.image_w if ts else 0

    @property
    def tileset_image_h(self) -> int:
        ts = self.active_ts
        return ts.image_h if ts else 0

    def tile_src_rect(self, local_id: int):
        ts = self.active_ts
        if ts is None:
            return (0, 0, self.tile_w, self.tile_h)
        return ts.tile_src_rect(local_id)

    @property
    def tile_properties(self) -> dict[int, dict]:
        ts = self.active_ts
        return ts.tile_properties if ts else {}

    @property
    def tile_property_types(self) -> dict[int, dict]:
        ts = self.active_ts
        return ts.tile_property_types if ts else {}

    # ------------------------------------------------------ Properties (aktiv)
    def set_property(self, local_id: int, key: str, value, ptype: str) -> None:
        ts = self.active_ts
        if ts is None:
            return
        if ptype not in PROP_TYPES:
            ptype = "string"
        ts.tile_properties.setdefault(local_id, {})[key] = coerce_prop(value, ptype)
        ts.tile_property_types.setdefault(local_id, {})[key] = ptype
        self.dirty = True

    def remove_property(self, local_id: int, key: str) -> None:
        ts = self.active_ts
        if ts is None:
            return
        ts.tile_properties.get(local_id, {}).pop(key, None)
        ts.tile_property_types.get(local_id, {}).pop(key, None)
        self.dirty = True

    def properties_of(self, local_id: int) -> dict:
        ts = self.active_ts
        return ts.tile_properties.get(local_id, {}) if ts else {}

    # ------------------------------------------------------ Layer-Ops
    def add_layer(self, name: str | None = None) -> int:
        n = name or f"Layer {len(self.layers) + 1}"
        self.layers.append(TileLayer(n, self.width, self.height))
        self.dirty = True
        return len(self.layers) - 1

    def add_object_layer(self, name: str | None = None) -> int:
        n = name or f"Objekte {len(self.layers) + 1}"
        self.layers.append(ObjectLayer(n))
        self.dirty = True
        return len(self.layers) - 1

    def remove_layer(self, idx: int) -> None:
        if 0 <= idx < len(self.layers) and len(self.layers) > 1:
            del self.layers[idx]
            self.dirty = True

    def move_layer(self, idx: int, delta: int) -> int:
        j = idx + delta
        if 0 <= idx < len(self.layers) and 0 <= j < len(self.layers):
            self.layers[idx], self.layers[j] = self.layers[j], self.layers[idx]
            self.dirty = True
            return j
        return idx

    # ------------------------------------------------------ Geometrie
    def resize(self, width: int, height: int) -> None:
        width, height = int(width), int(height)
        self.layers = [l.resized(width, height) for l in self.layers]
        self.width, self.height = width, height
        self.dirty = True

    # ------------------------------------------------------ Regionen (Select)
    def get_region(self, layer_idx: int, x0: int, y0: int,
                   w: int, h: int) -> list[list[int]]:
        """2D-Block (h Zeilen x w Spalten) von GIDs ab (x0,y0). Out-of-bounds
        liefert 0 (TileLayer.get clamped)."""
        layer = self.layers[layer_idx]
        if not isinstance(layer, TileLayer):
            return []
        return [[layer.get(x0 + dx, y0 + dy) for dx in range(w)]
                for dy in range(h)]

    def stamp_region(self, layer_idx: int, x0: int, y0: int,
                     block: list[list[int]]) -> bool:
        """Stempelt einen 2D-GID-Block ab (x0,y0). Out-of-bounds-Zellen
        werden ignoriert. Liefert True, wenn sich etwas geaendert hat."""
        layer = self.layers[layer_idx]
        if not isinstance(layer, TileLayer):
            return False
        changed = False
        for dy, row in enumerate(block):
            for dx, gid in enumerate(row):
                if layer.set(x0 + dx, y0 + dy, int(gid)):
                    changed = True
        if changed:
            self.dirty = True
        return changed

    def clear_region(self, layer_idx: int, x0: int, y0: int,
                     w: int, h: int) -> bool:
        """Setzt eine Region auf 0 (leer). Liefert True bei Aenderung."""
        return self.stamp_region(layer_idx, x0, y0,
                                 [[0] * w for _ in range(h)])

    # ------------------------------------------------------ Flood-Fill
    def flood_fill(self, layer_idx: int, x: int, y: int, gid: int) -> int:
        """Bucket-Fill der 4-verbundenen Region. Liefert Anzahl Tiles."""
        layer = self.layers[layer_idx]
        if not isinstance(layer, TileLayer):
            return 0  # Flood-Fill nur auf Tile-Layern
        if x < 0 or y < 0 or x >= layer.width or y >= layer.height:
            return 0
        target = layer.get(x, y)
        if target == gid:
            return 0
        stack = [(x, y)]
        n = 0
        w, h, tiles = layer.width, layer.height, layer.tiles
        while stack:
            cx, cy = stack.pop()
            if cx < 0 or cy < 0 or cx >= w or cy >= h:
                continue
            i = cy * w + cx
            if tiles[i] != target:
                continue
            tiles[i] = gid
            n += 1
            stack.extend(((cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)))
        if n:
            self.dirty = True
        return n

    # ------------------------------------------------------ Tiled-JSON
    def to_tiled_dict(self, map_path: str | None = None) -> dict:
        """Serialisiert in das Tiled-JSON-Format, das `TILED_LOAD` liest.

        Der Tileset-Bildpfad wird relativ zu `map_path` geschrieben (wenn
        gesetzt), sonst der gespeicherte Pfad unveraendert uebernommen."""
        def _rel(abs_path: str, fallback: str) -> str:
            if map_path and abs_path:
                try:
                    return os.path.relpath(
                        abs_path, Path(map_path).resolve().parent
                    ).replace("\\", "/")
                except ValueError:
                    return fallback
            return fallback

        tilesets_json = []
        for ts in self.tilesets:
            tiles_meta = []
            for lid in sorted(ts.tile_properties):
                props = ts.tile_properties[lid]
                if not props:
                    continue
                types = ts.tile_property_types.get(lid, {})
                plist = [{"name": k, "type": types.get(k, "string"), "value": v}
                         for k, v in props.items()]
                tiles_meta.append({"id": lid, "properties": plist})
            ts_json = {
                "firstgid": ts.firstgid,
                "name": ts.name or Path(ts.image or "tileset").stem or "tileset",
                "tilewidth": ts.tile_w,
                "tileheight": ts.tile_h,
                "tilecount": ts.tile_count,
                "columns": ts.columns,
                "margin": 0,
                "spacing": 0,
                "image": _rel(ts.image_abs, ts.image),
                "imagewidth": ts.image_w,
                "imageheight": ts.image_h,
            }
            if tiles_meta:
                ts_json["tiles"] = tiles_meta
            tilesets_json.append(ts_json)

        layers = []
        next_obj_id = 1
        for i, l in enumerate(self.layers):
            if isinstance(l, ObjectLayer):
                objs = []
                for obj in l.objects:
                    od = {
                        "id": next_obj_id,
                        "name": obj.name,
                        "type": obj.type,
                        "x": obj.x, "y": obj.y,
                        "width": obj.width, "height": obj.height,
                        "rotation": 0,
                        "visible": True,
                    }
                    if obj.is_point():
                        od["point"] = True
                    if obj.properties:
                        od["properties"] = [
                            {"name": k,
                             "type": obj.property_types.get(k, "string"),
                             "value": v}
                            for k, v in obj.properties.items()
                        ]
                    objs.append(od)
                    next_obj_id += 1
                layers.append({
                    "type": "objectgroup",
                    "id": i + 1,
                    "name": l.name,
                    "x": 0, "y": 0,
                    "opacity": l.opacity,
                    "visible": l.visible,
                    "draworder": "topdown",
                    "objects": objs,
                })
            else:
                layers.append({
                    "type": "tilelayer",
                    "id": i + 1,
                    "name": l.name,
                    "width": l.width,
                    "height": l.height,
                    "x": 0, "y": 0,
                    "opacity": l.opacity,
                    "visible": l.visible,
                    "data": list(l.tiles),
                })

        return {
            "type": "map",
            "version": "1.10",
            "tiledversion": "1.10.2",
            "orientation": "orthogonal",
            "renderorder": "right-down",
            "infinite": False,
            "width": self.width,
            "height": self.height,
            "tilewidth": self.tile_w,
            "tileheight": self.tile_h,
            "nextlayerid": len(self.layers) + 1,
            "nextobjectid": next_obj_id,
            "tilesets": tilesets_json,
            "layers": layers,
        }

    @classmethod
    def from_tiled_dict(cls, data: dict, base_dir: str | Path) -> "TileMapDoc":
        """Liest eine Tiled-JSON-Map (wie vom Editor geschrieben) zurueck.
        Toleriert das, was `TILED_LOAD` auch akzeptiert (embedded Tileset,
        CSV-Tile-Daten). Mehrere Tilesets: nur das erste wird uebernommen."""
        base_dir = Path(base_dir)
        doc = cls(
            int(data.get("width", 20)),
            int(data.get("height", 15)),
            int(data.get("tilewidth", 16)),
            int(data.get("tileheight", 16)),
        )
        for tsd in data.get("tilesets") or []:
            ts = Tileset(
                str(tsd.get("name", "tileset")),
                int(tsd.get("tilewidth", doc.tile_w)),
                int(tsd.get("tileheight", doc.tile_h)),
            )
            img = tsd.get("image", "")
            if img:
                abs_img = (base_dir / img).resolve()
                ts.image = img.replace("\\", "/")
                ts.image_abs = str(abs_img)
            ts.image_w = int(tsd.get("imagewidth", 0))
            ts.image_h = int(tsd.get("imageheight", 0))
            ts.columns = int(tsd.get("columns", 0))
            ts.tile_count = int(tsd.get("tilecount", 0))
            ts.firstgid = int(tsd.get("firstgid", 1))
            for tile in tsd.get("tiles", []):
                lid = tile.get("id")
                if not isinstance(lid, int):
                    continue
                for p in tile.get("properties", []):
                    name = p.get("name")
                    if not isinstance(name, str):
                        continue
                    ptype = p.get("type", "string")
                    if ptype not in PROP_TYPES:
                        ptype = "string"
                    ts.tile_properties.setdefault(lid, {})[name] = coerce_prop(
                        p.get("value"), ptype)
                    ts.tile_property_types.setdefault(lid, {})[name] = ptype
            doc.tilesets.append(ts)
        # firstgids aus der Datei beibehalten (externe Maps koennen Luecken
        # haben) -- nur nach firstgid sortieren fuer die Resolver-Reihenfolge.
        doc.tilesets.sort(key=lambda t: t.firstgid)
        doc.active_tileset = 0

        doc.layers = []
        for layer in data.get("layers", []):
            ltype = layer.get("type")
            if ltype == "objectgroup":
                ol = ObjectLayer(str(layer.get("name", "Objekte")))
                ol.visible = bool(layer.get("visible", True))
                ol.opacity = float(layer.get("opacity", 1.0))
                for o in layer.get("objects", []):
                    mo = MapObject(
                        float(o.get("x", 0.0)), float(o.get("y", 0.0)),
                        float(o.get("width", 0.0)), float(o.get("height", 0.0)),
                        str(o.get("name", "")),
                        str(o.get("type") or o.get("class") or ""),
                    )
                    for p in o.get("properties", []):
                        name = p.get("name")
                        if not isinstance(name, str):
                            continue
                        mo.set_property(name, p.get("value"),
                                        p.get("type", "string"))
                    ol.objects.append(mo)
                doc.layers.append(ol)
                continue
            if ltype != "tilelayer":
                continue
            lw = int(layer.get("width", doc.width))
            lh = int(layer.get("height", doc.height))
            l = TileLayer(str(layer.get("name", "Layer")), lw, lh)
            l.visible = bool(layer.get("visible", True))
            l.opacity = float(layer.get("opacity", 1.0))
            raw = layer.get("data")
            if isinstance(raw, list):
                l.tiles = [int(x) for x in raw][:lw * lh]
                l.tiles += [0] * (lw * lh - len(l.tiles))
            doc.layers.append(l)
        if not doc.layers:
            doc.layers = [TileLayer("Boden", doc.width, doc.height)]
        doc.dirty = False
        return doc

    def save_json(self, path: str) -> None:
        data = self.to_tiled_dict(path)
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=1)
        self.path = path
        self.dirty = False

    @classmethod
    def load_json(cls, path: str) -> "TileMapDoc":
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        doc = cls.from_tiled_dict(data, Path(path).resolve().parent)
        doc.path = path
        return doc

    # ------------------------------------------------------ GB-Code-Export
    def gb_code(self, map_path: str | None = None) -> str:
        """Selbststaendiges GB-Programm, das diese Map per `TILED_LOAD` laedt
        und Tile fuer Tile via DRAWIMAGEPART rendert (Quell-Rect aus gid-1)."""
        map_rel = "level.json"
        if map_path:
            map_rel = Path(map_path).name
        # Tileset-Liste fuer den Renderer (mind. ein Eintrag, damit das
        # erzeugte Programm immer kompiliert).
        ts_render = [(ts.image or f"tileset{i}.png",
                      max(1, ts.columns), ts.firstgid)
                     for i, ts in enumerate(self.tilesets)]
        if not ts_render:
            ts_render = [("tileset.png", 1, 1)]
        # Bild-DIMs
        img_dims = "\n".join(
            f'DIM ts{i} AS IMAGE\nts{i} = LOADIMAGE("{rel}")'
            for i, (rel, _cols, _fg) in enumerate(ts_render))
        # gid -> Tileset aufloesen (absteigend nach firstgid)
        order = sorted(range(len(ts_render)),
                       key=lambda i: ts_render[i][2], reverse=True)
        chain_lines = []
        for k, i in enumerate(order):
            _rel, cols, fg = ts_render[i]
            kw = "IF" if k == 0 else "ELSEIF"
            chain_lines.append(f"                        {kw} g >= {fg} THEN")
            chain_lines.append(f"                            lid = g - {fg}")
            chain_lines.append(
                f"                            DRAWIMAGEPART(ts{i}, "
                f"(lid MOD {cols}) * TW, (lid \\ {cols}) * TH, TW, TH, "
                f"tx * TW, ty * TH)")
        chain_lines.append("                        END IF")
        resolve_chain = "\n".join(chain_lines)
        sw = self.width * self.tile_w
        sh = self.height * self.tile_h
        scale = max(1, min(4, 960 // max(1, sw)))
        # Hinweis-Block fuer Object-Layer (werden nicht gerendert -- Spawn-/
        # Trigger-Daten fuer die Spiel-Logik). Zeigt die TILED_OBJECT_*-API.
        obj_layers = [l for l in self.layers if isinstance(l, ObjectLayer)]
        obj_comment = ""
        if obj_layers:
            names = ", ".join(repr(l.name) for l in obj_layers)
            obj_comment = (
                f"' Object-Layer ({names}) auslesen, z.B. Spawns setzen:\n"
                f"'   DIM n AS INTEGER : n = TILED_OBJECT_COUNT(lvl, "
                f"{obj_layers[0].name!r})\n"
                "'   FOR oi = 0 TO n - 1\n"
                "'     px = TILED_OBJECT_X(lvl, " + repr(obj_layers[0].name)
                + ", oi) : py = TILED_OBJECT_Y(lvl, " + repr(obj_layers[0].name)
                + ", oi)\n"
                "'     nm$ = TILED_OBJECT_NAME(lvl, " + repr(obj_layers[0].name)
                + ", oi)\n"
                "'   NEXT\n")
        return f'''' === Auto-generiert vom GameBasic-Tilemap-Editor ===
IMPORT "tiled"
{obj_comment}

SCREEN({sw}, {sh}, "Tilemap", {scale})

{img_dims}

DIM lvl AS TILED_MAP
lvl = TILED_LOAD("{map_rel}")

DIM TW AS INTEGER : TW = TILED_TILE_WIDTH(lvl)
DIM TH AS INTEGER : TH = TILED_TILE_HEIGHT(lvl)
DIM MW AS INTEGER : MW = TILED_WIDTH(lvl)
DIM MH AS INTEGER : MH = TILED_HEIGHT(lvl)
DIM NL AS INTEGER : NL = TILED_LAYER_COUNT(lvl)

DIM running AS BOOLEAN
running = TRUE
WHILE running
    IF KEYPRESSED(KEY_ESCAPE) THEN running = FALSE
    CLS(RGB(24, 24, 32))
    DIM li AS INTEGER
    DIM tx AS INTEGER
    DIM ty AS INTEGER
    DIM g AS INTEGER
    DIM lid AS INTEGER
    FOR li = 0 TO NL - 1
        IF TILED_LAYER_TYPE(lvl, li) = "tile" THEN
            FOR ty = 0 TO MH - 1
                FOR tx = 0 TO MW - 1
                    g = TILED_TILE_AT(lvl, li, tx, ty)
                    IF g > 0 THEN
{resolve_chain}
                    END IF
                NEXT
            NEXT
        END IF
    NEXT
    FLIP()
WEND
'''
