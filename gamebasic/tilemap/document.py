"""Datenmodell des Tilemap-Editors + Tiled-JSON-Serialisierung.

Bewusst Qt-frei: so kann der Export-/Import-Pfad headless getestet werden
(Roundtrip durch das `tiled`-Modul, das diese JSONs wieder einliest).

Format-Konvention (passt 1:1 zu `gamebasic/modules/tiled.py`):
  - EIN eingebettetes Tileset, `firstgid` immer 1 -> lokale Tile-ID = gid - 1.
  - Tile-Daten als CSV-Liste von GIDs (kein base64), row-major, 0 = leer.
  - Per-Tile-Properties als Tiled-`{name,type,value}`-Liste am Tileset.
  - Nur orthogonale Tile-Layer (Object-Layer kann spaeter dazukommen).

Der Editor speichert/laedt genau dieses Format (kein eigenes Projektformat),
damit der Kreis Editor -> `TILED_LOAD` -> Spiel geschlossen ist und die Map
auch im echten Tiled weiterbearbeitet werden kann.
"""
from __future__ import annotations

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


class TileMapDoc:
    """Vollstaendige Tilemap: Geometrie, ein Tileset, N Tile-Layer, Props."""

    def __init__(self, width: int = 20, height: int = 15,
                 tile_w: int = 16, tile_h: int = 16):
        self.width = int(width)
        self.height = int(height)
        self.tile_w = int(tile_w)
        self.tile_h = int(tile_h)
        # Tileset (ein einzelnes, firstgid=1)
        self.tileset_image = ""        # Pfad wie gespeichert (relativ zur Map)
        self.tileset_image_abs = ""    # absoluter Pfad zum Laden des Pixmaps
        self.tileset_image_w = 0
        self.tileset_image_h = 0
        self.columns = 0
        self.tile_count = 0
        # lokale Tile-ID (gid-1) -> {key: typed value}
        self.tile_properties: dict[int, dict] = {}
        # Property-Typen merken (Tiled braucht den Typ beim Export)
        self.tile_property_types: dict[int, dict[str, str]] = {}
        self.layers: list[TileLayer] = [TileLayer("Boden", self.width, self.height)]
        self.path = ""                 # zuletzt gespeicherter Map-Pfad
        self.dirty = False

    # ------------------------------------------------------ Tileset
    def set_tileset(self, abs_path: str, image_w: int, image_h: int) -> None:
        """Setzt das Tileset aus einem geladenen Bild. Spalten/Tile-Anzahl
        ergeben sich aus Bildgroesse / Tile-Groesse (margin/spacing = 0)."""
        self.tileset_image_abs = os.path.abspath(abs_path)
        self.tileset_image = abs_path
        self.tileset_image_w = int(image_w)
        self.tileset_image_h = int(image_h)
        self.columns = max(0, image_w // self.tile_w) if self.tile_w else 0
        rows = max(0, image_h // self.tile_h) if self.tile_h else 0
        self.tile_count = self.columns * rows

    def tile_src_rect(self, local_id: int):
        """Quell-Rechteck (sx, sy, w, h) eines lokalen Tiles im Tileset-Bild."""
        if self.columns <= 0:
            return (0, 0, self.tile_w, self.tile_h)
        cx = local_id % self.columns
        cy = local_id // self.columns
        return (cx * self.tile_w, cy * self.tile_h, self.tile_w, self.tile_h)

    # ------------------------------------------------------ Properties
    def set_property(self, local_id: int, key: str, value, ptype: str) -> None:
        if ptype not in PROP_TYPES:
            ptype = "string"
        self.tile_properties.setdefault(local_id, {})[key] = coerce_prop(value, ptype)
        self.tile_property_types.setdefault(local_id, {})[key] = ptype
        self.dirty = True

    def remove_property(self, local_id: int, key: str) -> None:
        self.tile_properties.get(local_id, {}).pop(key, None)
        self.tile_property_types.get(local_id, {}).pop(key, None)
        self.dirty = True

    def properties_of(self, local_id: int) -> dict:
        return self.tile_properties.get(local_id, {})

    # ------------------------------------------------------ Layer-Ops
    def add_layer(self, name: str | None = None) -> int:
        n = name or f"Layer {len(self.layers) + 1}"
        self.layers.append(TileLayer(n, self.width, self.height))
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

    # ------------------------------------------------------ Flood-Fill
    def flood_fill(self, layer_idx: int, x: int, y: int, gid: int) -> int:
        """Bucket-Fill der 4-verbundenen Region. Liefert Anzahl Tiles."""
        layer = self.layers[layer_idx]
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
        image_rel = self.tileset_image
        if map_path and self.tileset_image_abs:
            try:
                image_rel = os.path.relpath(
                    self.tileset_image_abs, Path(map_path).resolve().parent
                ).replace("\\", "/")
            except ValueError:
                image_rel = self.tileset_image

        tiles_meta = []
        for lid in sorted(self.tile_properties):
            props = self.tile_properties[lid]
            if not props:
                continue
            types = self.tile_property_types.get(lid, {})
            plist = [{"name": k, "type": types.get(k, "string"), "value": v}
                     for k, v in props.items()]
            tiles_meta.append({"id": lid, "properties": plist})

        tileset = {
            "firstgid": 1,
            "name": Path(self.tileset_image or "tileset").stem or "tileset",
            "tilewidth": self.tile_w,
            "tileheight": self.tile_h,
            "tilecount": self.tile_count,
            "columns": self.columns,
            "margin": 0,
            "spacing": 0,
            "image": image_rel,
            "imagewidth": self.tileset_image_w,
            "imageheight": self.tileset_image_h,
        }
        if tiles_meta:
            tileset["tiles"] = tiles_meta

        layers = []
        for i, l in enumerate(self.layers):
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
            "nextobjectid": 1,
            "tilesets": [tileset],
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
        tss = data.get("tilesets") or []
        if tss:
            ts = tss[0]
            img = ts.get("image", "")
            if img:
                abs_img = (base_dir / img).resolve()
                doc.tileset_image = img.replace("\\", "/")
                doc.tileset_image_abs = str(abs_img)
            doc.tileset_image_w = int(ts.get("imagewidth", 0))
            doc.tileset_image_h = int(ts.get("imageheight", 0))
            doc.columns = int(ts.get("columns", 0))
            doc.tile_count = int(ts.get("tilecount", 0))
            for tile in ts.get("tiles", []):
                lid = tile.get("id")
                if not isinstance(lid, int):
                    continue
                for p in tile.get("properties", []):
                    name = p.get("name")
                    if not isinstance(name, str):
                        continue
                    ptype = p.get("type", "string")
                    doc.set_property(lid, name, p.get("value"), ptype)

        doc.layers = []
        for layer in data.get("layers", []):
            if layer.get("type") != "tilelayer":
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
        tileset_rel = self.tileset_image or "tileset.png"
        sw = self.width * self.tile_w
        sh = self.height * self.tile_h
        scale = max(1, min(4, 960 // max(1, sw)))
        return f'''' === Auto-generiert vom GameBasic-Tilemap-Editor ===
IMPORT "tiled"

SCREEN({sw}, {sh}, "Tilemap", {scale})

DIM tileset AS IMAGE
tileset = LOADIMAGE("{tileset_rel}")

DIM lvl AS TILED_MAP
lvl = TILED_LOAD("{map_rel}")

CONST COLS = {self.columns}
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
    FOR li = 0 TO NL - 1
        IF TILED_LAYER_TYPE(lvl, li) = "tile" THEN
            FOR ty = 0 TO MH - 1
                FOR tx = 0 TO MW - 1
                    DIM g AS INTEGER
                    g = TILED_TILE_AT(lvl, li, tx, ty)
                    IF g > 0 THEN
                        DIM lid AS INTEGER : lid = g - 1
                        DRAWIMAGEPART(tileset, (lid MOD COLS) * TW, (lid \\ COLS) * TH, TW, TH, tx * TW, ty * TH)
                    END IF
                NEXT
            NEXT
        END IF
    NEXT
    FLIP()
WEND
'''
