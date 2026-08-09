# Modul `tiled`

Loader fuer [Tiled](https://www.mapeditor.org/)-Maps im JSON-Format. Tiled ist Industriestandard fuer 2D-Level-Design — fast jeder Indie-2D-Engine-Workflow geht durch Tiled.

```basic
IMPORT "tiled"
```

**Workflow:**

1. In Tiled das Level bauen (Tilesets, Layer, Object-Layer mit Custom-Properties)
2. **File → Save As → JSON Map** (Tiled speichert nativ als `.tmx`, kann aber auch direkt JSON schreiben)
3. Im Spiel: `map = TILED_LOAD("level.json")`
4. Layer iterieren, Objekte spawnen, Tile-Properties pruefen

**TMX (XML) wird in v1 nicht unterstuetzt** — saving as JSON ist die einzige Sache, die der Artist anders machen muss als gewohnt.

## Übersicht

### Map-Metadata

| Funktion | Rueckgabe |
|---|---|
| `TILED_LOAD(path$)` | TILED_MAP |
| `TILED_WIDTH(m)` / `TILED_HEIGHT(m)` | INTEGER (in Tiles) |
| `TILED_TILE_WIDTH(m)` / `TILED_TILE_HEIGHT(m)` | INTEGER (Pixel pro Tile) |

### Layer

| Funktion | Rueckgabe |
|---|---|
| `TILED_LAYER_COUNT(m)` | INTEGER |
| `TILED_LAYER_NAME(m, idx)` | STRING |
| `TILED_LAYER_TYPE(m, idx)` | `"tile"`, `"object"`, `"image"` |
| `TILED_LAYER_INDEX(m, name$)` | INTEGER (-1 wenn nicht da) |
| `TILED_LAYER_WIDTH(m, idx)` / `..._HEIGHT(m, idx)` | INTEGER (nur Tile-Layer) |

### Tile-Daten

| Funktion | Rueckgabe |
|---|---|
| `TILED_TILE_AT(m, layer_idx, tx, ty)` | INTEGER (GID, 0 = leer; OOB = 0) |
| `TILED_TILE_SET(m, layer_idx, tx, ty, gid)` | INTEGER (alte GID). 0 = Tile loeschen. OOB = no-op. |
| `TILED_TILE_PROP_BOOL(m, gid, key$)` | BOOLEAN (FALSE wenn nicht gesetzt) |
| `TILED_TILE_PROP_INT(m, gid, key$)` | INTEGER (0 wenn nicht gesetzt) |
| `TILED_TILE_PROP_FLOAT(m, gid, key$)` | FLOAT |
| `TILED_TILE_PROP_STRING(m, gid, key$)` | STRING |
| `TILED_TILE_HAS_PROP(m, gid, key$)` | BOOLEAN |

### Object-Layer

| Funktion | Rueckgabe |
|---|---|
| `TILED_OBJECT_COUNT(m, layer_name$)` | INTEGER |
| `TILED_OBJECT_NAME(m, layer_name$, idx)` | STRING |
| `TILED_OBJECT_TYPE(m, layer_name$, idx)` | STRING (Tiled "type" oder "class") |
| `TILED_OBJECT_X/Y(...)` | FLOAT |
| `TILED_OBJECT_WIDTH/HEIGHT(...)` | FLOAT |
| `TILED_OBJECT_PROP_BOOL/INT/FLOAT/STRING(...)` | Custom Property |

### Tileset

| Funktion | Rueckgabe |
|---|---|
| `TILED_TILESET_COUNT(m)` | INTEGER |
| `TILED_TILESET_IMAGE(m, idx)` | STRING (absoluter Pfad zum Tileset-Bild) |
| `TILED_TILESET_FIRSTGID(m, idx)` | INTEGER |

## Konzept: GIDs vs. lokale Tile-IDs

Tiled vergibt **globale Tile-IDs (GIDs)** ueber alle Tilesets hinweg. Wenn ein Tileset mit `firstGid=1` 8 Tiles enthaelt, sind dessen Tile-IDs `1..8`. Ein zweites Tileset mit `firstGid=9` setzt fort.

GID = 0 bedeutet "leeres Tile" — in jeder Layer-Cell, die nicht gefuellt ist, steht 0.

`TILED_TILE_AT(...)` liefert immer GIDs. Wer das Tile als Sub-Sprite aus einem `SPRITE_ATLAS` rendern will, muss die Mappung GID → Atlas-Name selbst pflegen (siehe Beispiel unten) oder den Atlas so anlegen, dass die Sprite-Indizes mit den GIDs uebereinstimmen.

## Custom Properties

Das groesste Tiled-Killer-Feature: **alles** kann Custom-Properties haben — Tiles, Objects, Layers, sogar die ganze Map. Im Editor: Rechtsklick → "Custom Properties" → Neuer Property mit Name + Type (bool, int, float, string, color).

**Beispiel-Workflow fuer Platformer:**

- Tile-Type `grass` bekommt Property `solid: true` (alle Wand-Tiles)
- Tile-Type `spikes` bekommt `damage: 1` und `solid: false` (Trigger ohne Block)
- Object-Type `enemy` bekommt `hp: 30`, `speed: 1.2`, `pattern: "patrol"`
- Object-Type `door` bekommt `target_level: "level_2"`

Im Code:

```basic
' Bei jedem Spawn-Objekt
DIM hp AS INTEGER
hp = TILED_OBJECT_PROP_INT(m, "enemies", i, "hp")

' Beim Tile-Test (in einer Damage-System-Schleife):
IF TILED_TILE_PROP_INT(m, gid, "damage") > 0 THEN
    HurtPlayer(...)
END IF
```

## Object-Layer Pattern

Tiled erlaubt freie Rechtecke / Polygone / Points / Ellipsen als "Objects" in einem dedizierten Layer. Klassische Use-Cases:

- **Spawn-Punkte** — Player-Start, Enemy-Spawn, Item-Spawn
- **Trigger-Zonen** — Door-Trigger, Speicherpunkt, Cutscene-Auslöser
- **Wegpunkte** — Patrol-Pfad, AI-Navigation

Beim Laden iteriert man durch alle Objects im Layer und spawnt entsprechend:

```basic
DIM n AS INTEGER
n = TILED_OBJECT_COUNT(m, "spawns")
DIM i AS INTEGER
FOR i = 0 TO n - 1
    DIM obj_type AS STRING
    obj_type = TILED_OBJECT_TYPE(m, "spawns", i)
    SELECT CASE obj_type
        CASE "player_start"
            player_x = TILED_OBJECT_X(m, "spawns", i)
            player_y = TILED_OBJECT_Y(m, "spawns", i)
        CASE "enemy"
            SpawnEnemy(
                TILED_OBJECT_X(m, "spawns", i),
                TILED_OBJECT_Y(m, "spawns", i),
                TILED_OBJECT_PROP_INT(m, "spawns", i, "hp"))
        CASE "door"
            SpawnDoor(
                TILED_OBJECT_X(m, "spawns", i),
                TILED_OBJECT_Y(m, "spawns", i),
                TILED_OBJECT_PROP_STRING(m, "spawns", i, "target"))
    END SELECT
NEXT
```

## Layer-Konvention

Typisches Setup:

| Layer-Name | Type | Zweck |
|---|---|---|
| `background` | tile | Parallax-Hintergrund |
| `decor` | tile | nicht-solid Deko-Tiles |
| `ground` | tile | **Collision-Layer** (mit solid-Tiles) |
| `foreground` | tile | vor dem Player (Stein-Vordergrund) |
| `spawns` | object | Spawn-Punkte, Trigger |
| `paths` | object | Patrol-Wegpunkte |

Im Spiel: das `tile_collide`-Modul nimmt EIN Tile-Layer als Collision-Layer (typisch `ground`). Andere Layer werden nur gerendert.

## Tile-Rendering: Pattern mit Sprite-Atlas

Tile-Layer rendern: pro Frame ueber alle Tile-Cells iterieren, gefuellte Tiles aus dem Atlas zeichnen. (`BATCH_DRAW` unten ist nur ein Zweitname fuer `ATLAS_DRAW` und `BATCH_FLUSH` ein No-Op -- siehe Grafik-Doku; sie buendeln nichts.)

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/tiles_atlas.json")

' Tile-Namen pro Tile-ID (Atlas-Sprite-Namen)
DIM names[8] AS STRING
names[0] = "grass"     ' GID 1
names[1] = "stone"     ' GID 2
' ...

' Pro Frame:
DIM tx AS INTEGER
DIM ty AS INTEGER
FOR ty = 0 TO TILED_HEIGHT(map) - 1
    FOR tx = 0 TO TILED_WIDTH(map) - 1
        DIM gid AS INTEGER
        gid = TILED_TILE_AT(map, 0, tx, ty)    ' 0 = ground-Layer-Index
        IF gid > 0 THEN
            BATCH_DRAW(atlas, names[gid - 1],
                       tx * TILED_TILE_WIDTH(map),
                       ty * TILED_TILE_HEIGHT(map))
        END IF
    NEXT
NEXT
BATCH_FLUSH()
```

Bei grossen Maps (z.B. 100×100) + Camera-Scrolling lohnt es sich, nur den sichtbaren Bereich zu iterieren — Pseudo:

```basic
DIM first_tx AS INTEGER
DIM last_tx AS INTEGER
first_tx = MAX(0, cam_x / TW)
last_tx  = MIN(TILED_WIDTH(map) - 1, (cam_x + screen_w) / TW + 1)
' ... analog fuer ty ...
```

## Externe Tilesets

Tiled erlaubt Tilesets als externe `.json`-Datei (zur Wiederverwendung in mehreren Maps). Die Map referenziert sie via `"source": "tileset.json"`. Das Modul loest das automatisch auf — du musst nichts extra tun.

Was **nicht** unterstuetzt wird:
- TMX (XML-Format) — Map als JSON speichern
- Base64/zlib/gzip-codierte Tile-Daten — In Tiled: `Edit → Preferences → "Store tile layer data as: CSV"` (Default)
- Group-Layer — werden ignoriert, ihre Sub-Layer nicht expandiert
- Hex / Iso-Maps — Modul ist auf orthogonale Maps zugeschnitten

## Externer Typ

| Typ | Wirkung |
|---|---|
| `TILED_MAP` | Geladene Tiled-Map. `DIM m AS TILED_MAP` |

## Beispiel

[examples/77_tiled_platformer.dh](../examples/77_tiled_platformer.dh) zeigt das volle Pattern: Tiled-Level laden, Tile-Layer batchen, Object-Layer fuer Player-Spawn nutzen, Tile-Properties als Collision-Source verwenden.

Vollstaendiger Workflow inklusive Kollision: siehe [tile_collide-Modul](module-tile-collide.md).

## In der nativen Runtime (dhrt)

`tiled` laeuft nativ (immer dabei, JSON-Loader via serde_json) und ist **bit-identisch** zu den Python-Pfaden — inkl. External-Tilesets, Tile-/Object-Properties und Bulk-Ops (`TILED_FILL_RECT`/`REPLACE`/`COUNT_GID`/`FLOOD_FILL`).
