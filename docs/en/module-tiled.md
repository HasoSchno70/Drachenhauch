# Module `tiled`

**Read, modify, create and write** maps in the [Tiled](https://www.mapeditor.org/) JSON format. Tiled is the industry standard for 2D level design — almost every indie 2D engine workflow goes through Tiled.

Until 2026-08-30 the module was a pure reader: a loaded map could be changed in memory, but not written back, and a new one could not be created at all. That ruled out everything that **builds** maps instead of just using them — an editor, a generator, a conversion tool. The sections [Creating and saving maps](#creating-and-saving-maps) and [Changing layers](#changing-layers) close that gap.

```basic
IMPORT "tiled"
```

**Workflow:**

1. Build the level in Tiled (tilesets, layers, object layers with custom properties)
2. **File → Save As → JSON Map** (Tiled saves natively as `.tmx`, but can also write JSON directly)
3. In the game: `m = TILED_LOAD("level.json")`
4. Iterate over the layers, spawn objects, check tile properties

**TMX (XML) is not supported in v1** — saving as JSON is the only thing the artist has to do differently than usual.

## Overview

### Map metadata

| Function | Returns |
|---|---|
| `TILED_LOAD(path$)` | TILED_MAP |
| `TILED_WIDTH(m)` / `TILED_HEIGHT(m)` | INTEGER (in tiles) |
| `TILED_TILE_WIDTH(m)` / `TILED_TILE_HEIGHT(m)` | INTEGER (pixels per tile) |

### Layers

| Function | Returns | Meaning |
|---|---|---|
| `TILED_LAYER_COUNT(m)` | INTEGER | how many layers does the map have? |
| `TILED_LAYER_NAME(m, idx)` | STRING | name of the layer (index from 0) |
| `TILED_LAYER_TYPE(m, idx)` | `"tile"`, `"object"`, `"image"` | kind of layer |
| `TILED_LAYER_INDEX(m, name$)` | INTEGER (-1 if not present) | find a layer by its name |
| `TILED_LAYER_WIDTH/HEIGHT(m, idx)` | INTEGER (tile layers only) | size of the layer in tiles |

### Tile data

| Function | Returns | Meaning |
|---|---|---|
| `TILED_TILE_AT(m, layer_idx, tx, ty)` | INTEGER (GID, 0 = empty; OOB = 0) | which tile is at this position? |
| `TILED_TILE_SET(m, layer_idx, tx, ty, gid)` | INTEGER (old GID). 0 = delete the tile. OOB = no-op. | set a tile |
| `TILED_TILE_PROP_BOOL(m, gid, key$)` | BOOLEAN (FALSE if not set) | read a property of the tile -- maintained per tile in Tiled (`solid`, `damage`, ...) |
| `TILED_TILE_PROP_INT(m, gid, key$)` | INTEGER (0 if not set) | property of the tile as an integer |
| `TILED_TILE_PROP_FLOAT(m, gid, key$)` | FLOAT | property of the tile as a floating-point number |
| `TILED_TILE_PROP_STRING(m, gid, key$)` | STRING | property of the tile as text |
| `TILED_TILE_HAS_PROP(m, gid, key$)` | BOOLEAN |  |

### Object layers

| Function | Returns | Meaning |
|---|---|---|
| `TILED_OBJECT_COUNT(m, layer_name$)` | INTEGER | how many objects does this object layer have? |
| `TILED_OBJECT_NAME(m, layer_name$, idx)` | STRING | name of the object (index from 0) |
| `TILED_OBJECT_TYPE(m, layer_name$, idx)` | STRING (Tiled "type" or "class") | type of the object, as entered in Tiled |
| `TILED_OBJECT_X/Y(...)` | FLOAT | position of the object in pixels |
| `TILED_OBJECT_WIDTH/HEIGHT(...)` | FLOAT | size of the object in pixels |
| `TILED_OBJECT_PROP_BOOL/INT/FLOAT/STRING(...)` | custom property |  |

### Tilesets

| Function | Returns | Meaning |
|---|---|---|
| `TILED_TILESET_COUNT(m)` | INTEGER | how many tilesets does the map bring along? |
| `TILED_TILESET_IMAGE(m, idx)` | STRING (absolute path to the tileset image) | image file of the tileset |
| `TILED_TILESET_FIRSTGID(m, idx)` | INTEGER | first GID of this tileset -- this is how you map a GID to its tileset |
| `TILED_FILL_RECT(m, layer_idx, tx, ty, w, h, gid)` | INTEGER | fill a rectangle with a GID; returns the number of changed tiles |
| `TILED_REPLACE(m, layer_idx, from_gid, to_gid)` | INTEGER | replace every GID `from_gid` with `to_gid` — swap a tileset without a loop |
| `TILED_COUNT_GID(m, layer_idx, gid)` | INTEGER | how often does this GID occur in the layer? |
| `TILED_FLOOD_FILL(m, layer_idx, tx, ty, gid)` | INTEGER | paint bucket: recolour the connected area starting at `(tx,ty)` |

## Creating and saving maps

| Function | Returns | Meaning |
|---|---|---|
| `TILED_NEW(breite, hoehe, kachel_w, kachel_h)` | TILED_MAP | create an empty map -- without a layer and without a tileset |
| `TILED_ADD_LAYER(m, name$)` | INTEGER (index) | append an empty tile layer |
| `TILED_ADD_TILESET(m, bild$, kachelzahl)` | INTEGER (index) | append a tileset; the runtime assigns the `firstgid` itself |
| `TILED_TILESET_TILES(m, idx)` | INTEGER | how many tiles does this tileset have? |
| `TILED_ADD_OBJECT_LAYER(m, name$)` | INTEGER (index) | append an object layer |
| `TILED_ADD_OBJECT(m, ebene$, name$, typ$, x, y, w, h)` | INTEGER (index) | put an object into an object layer |
| `TILED_TILE_SET_PROP(m, gid, key$, wert)` | — | set a property of a tile |
| `TILED_TILE_REMOVE_PROP(m, gid, key$)` | — | remove a property of a tile |
| `TILED_OBJECT_SET_PROP(m, ebene$, idx, key$, wert)` | — | set a property of an object |
| `TILED_OBJECT_REMOVE_PROP(m, ebene$, idx, key$)` | — | remove a property of an object |
| `TILED_REMOVE_OBJECT(m, ebene$, idx)` | — | remove an object |
| `TILED_OBJECT_SET_NAME(m, ebene$, idx, name$)` | — | rename an object |
| `TILED_OBJECT_SET_TYPE(m, ebene$, idx, typ$)` | — | change its type |
| `TILED_OBJECT_SET_RECT(m, ebene$, idx, x, y, w, h)` | — | change position and size |
| `TILED_TILE_PROP_KEYS(m, gid)` | ARRAY OF STRING | which properties does this tile have? |
| `TILED_OBJECT_PROP_KEYS(m, ebene$, idx)` | ARRAY OF STRING | the same for an object |
| `TILED_SAVE(m, pfad$)` | — | write the map as Tiled JSON |

```basic
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(40, 30, 16, 16)
DIM ts AS INTEGER : ts = TILED_ADD_TILESET(m, "tiles.png", 32)
DIM boden AS INTEGER : boden = TILED_ADD_LAYER(m, "Boden")
TILED_FILL_RECT(m, boden, 0, 0, 40, 30, 1)
TILED_SAVE(m, "level.json")
```

**The `firstgid` is not set by hand.** It is the first global ID of the
tileset; the first one gets 1, every further one follows on after the previous one.
Overlapping ranges would be the most common and most unpleasant source of errors --
they silently destroy the mapping of **all** tiles, without anything giving
an error message. That is why `TILED_ADD_TILESET` needs the
tile count: only with it can the next GID be computed.

**What gets written is real Tiled JSON** — embedded tilesets, `type: "map"`,
tile data as a list of numbers. This is verified not against its own reader, but
against a **foreign** one: the json module, which reads the file by the rules of the
Tiled format (`tests/pruef/tiled_schreiben.dhtest`; until 2026-09-14 it was
the data model of the since-deleted Qt editor `dhtilemap`). A format that only its
own writer can read back is not verified, merely self-consistent.

### Creating object layers and properties

```basic
DIM sp AS INTEGER : sp = TILED_ADD_OBJECT_LAYER(m, "spawns")
DIM held AS INTEGER
held = TILED_ADD_OBJECT(m, "spawns", "held", "spawn", 32.0, 48.0, 16.0, 16.0)
TILED_OBJECT_SET_PROP(m, "spawns", held, "leben", 3)

TILED_TILE_SET_PROP(m, 5, "solid", TRUE)      ' GID, not the local number
TILED_TILE_SET_PROP(m, 5, "damage", 10)
```

**One setter instead of four.** The readers are typed
(`TILED_TILE_PROP_BOOL/INT/FLOAT/STRING`), because there the caller says what
they expect. When writing that is unnecessary: Drachenhauch already distinguishes BOOLEAN,
INTEGER, FLOAT and STRING in the value, so the value carries its type with
it. Everything else (ARRAY, MAP, handles) is rejected — Tiled knows exactly
these four kinds, and an array would silently decay into text on saving.

The **GID** is the address when setting, too, not the local number — just as when
reading. The property is stored with the tileset under the local number,
but that is bookkeeping a program does not need to know about. Flip bits are
masked exactly as when reading; otherwise a mirrored tile would store its
property somewhere else than an unmirrored one.

`TILED_OBJECT_SET_RECT` sets **all four** values at once, not four
separate setters: in an editor moving and resizing belong to the
same gesture, and a half-updated rectangle is an error you
cannot see.

`TILED_REMOVE_OBJECT` lets the indices **behind it move up** — like any
list. If you remembered one, fetch it again afterwards; that is the only
surprise about it.

**What does this tile actually have?** `TILED_TILE_HAS_PROP` only answers
a question you already know. `TILED_TILE_PROP_KEYS` (and
`TILED_OBJECT_PROP_KEYS`) return the keys themselves — **sorted**, because
the storage is a HashMap, and a list that reshuffles itself on every refresh
would be impossible to work with.

A `TILED_TILE_REMOVE_PROP` that takes away a tile's last property
also removes its entry entirely: `TILED_SAVE` lists every tile that has
one, and one with an empty list would be noise in the file.

`TILED_SAVE` writes properties **sorted by name** and the tiles
by number — the same map saved twice gives the same file. The
object IDs apply to the **whole map** (not per layer), and
`nextobjectid` lies above all of them, so that Tiled does not hand out duplicates while
editing. A loaded file with missing or duplicate IDs gets
new ones.

**Limits:** `TILED_NEW` only creates orthogonal, finite maps (no
isometric, no `infinite`). Objects are always rectangles — polygons,
ellipses and points (Tiled: `"point": true`) cannot be created, although the
Qt editor reads an object with width and height 0 as a point.
Object layers cannot be turned back into tile layers.
Very large maps are rejected (over 4 million tiles), so that a
typo in the size does not eat up the memory.

## Changing layers

| Function | Returns | Meaning |
|---|---|---|
| `TILED_LAYER_RENAME(m, idx, name$)` | — | rename a layer |
| `TILED_LAYER_VISIBLE(m, idx)` | BOOLEAN | is the layer shown? |
| `TILED_LAYER_SET_VISIBLE(m, idx, an)` | — | show or hide a layer |
| `TILED_REMOVE_LAYER(m, idx)` | — | remove a layer |
| `TILED_MOVE_LAYER(m, von, nach)` | — | move a layer to another position |

Visibility is **not just display** -- Tiled saves it in the file.
Without `TILED_LAYER_SET_VISIBLE`, a hidden layer therefore could not be saved
that way at all: it came back visible on the next load.

**The order is the DRAWING order**, not just a display matter:
layer 0 is at the back, the last one at the front. `TILED_MOVE_LAYER` takes the layer
out and inserts it again at `nach` (0 = all the way at the back) -- it does NOT swap,
because for a jump over several positions that would be something else.

**Watch out, indices shift.** `TILED_REMOVE_LAYER` moves all layers
behind it up, and `TILED_MOVE_LAYER` reorders them. If you remembered a layer number,
fetch it again afterwards (`TILED_LAYER_INDEX(m, name$)`). The
name index is kept up to date on renaming, removing and moving -- an
old name returns -1 afterwards.

## Concept: GIDs vs. local tile IDs

Tiled assigns **global tile IDs (GIDs)** across all tilesets. If a tileset with `firstGid=1` contains 8 tiles, its tile IDs are `1..8`. A second tileset with `firstGid=9` continues from there.

GID = 0 means "empty tile" — every layer cell that is not filled contains 0.

`TILED_TILE_AT(...)` always returns GIDs. If you want to render the tile as a sub-sprite from a `SPRITE_ATLAS`, you have to maintain the GID → atlas name mapping yourself (see the example below) or set up the atlas so that the sprite indices match the GIDs.

## Custom properties

Tiled's biggest killer feature: **everything** can have custom properties — tiles, objects, layers, even the whole map. In the editor: right-click → "Custom Properties" → new property with name + type (bool, int, float, string, color).

**Example workflow for a platformer:**

- Tile type `grass` gets the property `solid: true` (all wall tiles)
- Tile type `spikes` gets `damage: 1` and `solid: false` (a trigger without blocking)
- Object type `enemy` gets `hp: 30`, `speed: 1.2`, `pattern: "patrol"`
- Object type `door` gets `target_level: "level_2"`

In the code:

```basic
' For every spawn object
DIM hp AS INTEGER
hp = TILED_OBJECT_PROP_INT(m, "enemies", i, "hp")

' In the tile test (in a damage system loop):
IF TILED_TILE_PROP_INT(m, gid, "damage") > 0 THEN
    HurtPlayer(...)
END IF
```

## Object layer pattern

Tiled allows free rectangles / polygons / points / ellipses as "objects" in a dedicated layer. Classic use cases:

- **Spawn points** — player start, enemy spawn, item spawn
- **Trigger zones** — door trigger, save point, cutscene trigger
- **Waypoints** — patrol path, AI navigation

On loading you iterate through all objects in the layer and spawn accordingly:

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

## Layer convention

A typical setup:

| Layer name | Type | Purpose |
|---|---|---|
| `background` | tile | parallax background |
| `decor` | tile | non-solid decoration tiles |
| `ground` | tile | **collision layer** (with solid tiles) |
| `foreground` | tile | in front of the player (stone foreground) |
| `spawns` | object | spawn points, triggers |
| `paths` | object | patrol waypoints |

In the game: the `tile_collide` module takes ONE tile layer as the collision layer (typically `ground`). Other layers are only rendered.

## Tile rendering: pattern with a sprite atlas

Rendering tile layers: each frame iterate over all tile cells and draw the filled tiles from the atlas. (`BATCH_DRAW` below is just a second name for `ATLAS_DRAW` and `BATCH_FLUSH` is a no-op -- see the graphics documentation; they don't batch anything.)

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/tiles_atlas.json")

' tile names per tile ID (atlas sprite names)
DIM names[8] AS STRING
names[0] = "grass"     ' GID 1
names[1] = "stone"     ' GID 2
' ...

' Per frame:
DIM tx AS INTEGER
DIM ty AS INTEGER
FOR ty = 0 TO TILED_HEIGHT(m) - 1
    FOR tx = 0 TO TILED_WIDTH(m) - 1
        DIM gid AS INTEGER
        gid = TILED_TILE_AT(m, 0, tx, ty)    ' 0 = ground layer index
        IF gid > 0 THEN
            BATCH_DRAW(atlas, names[gid - 1],
                       tx * TILED_TILE_WIDTH(m),
                       ty * TILED_TILE_HEIGHT(m))
        END IF
    NEXT
NEXT
BATCH_FLUSH()
```

For large maps (e.g. 100×100) + camera scrolling it pays off to iterate only over the visible area — pseudocode:

```basic
DIM first_tx AS INTEGER
DIM last_tx AS INTEGER
first_tx = MAX(0, cam_x / TW)
last_tx  = MIN(TILED_WIDTH(m) - 1, (cam_x + screen_w) / TW + 1)
' ... likewise for ty ...
```

## External tilesets

Tiled allows tilesets as an external `.json` file (for reuse in several maps). The map references them via `"source": "tileset.json"`. The module resolves that automatically — you don't have to do anything extra.

What is **not** supported:
- TMX (XML format) — save the map as JSON
- Base64/zlib/gzip-encoded tile data — in Tiled: `Edit → Preferences → "Store tile layer data as: CSV"` (default)
- Group layers — they are ignored, their sub-layers are not expanded
- Hex / iso maps — the module is tailored to orthogonal maps

## External type

| Type | Effect |
|---|---|
| `TILED_MAP` | a loaded Tiled map. `DIM m AS TILED_MAP` |

## An editor as an example

[`examples/187_tilemap_editor.dh`](../../examples/187_tilemap_editor.dh) is a
complete tilemap editor in Drachenhauch: pen/eraser/fill/rectangle/
eyedropper/selection, clipboard, undo, layers, loading and saving.
It uses exactly the commands of this section and is the shortest way
to see them working together.

## Example

[examples/77_tiled_platformer.dh](../../examples/77_tiled_platformer.dh) shows the full pattern: load a Tiled level, batch the tile layers, use an object layer for the player spawn, use tile properties as the collision source.

Complete workflow including collision: see the [tile_collide module](module-tile-collide.md).

## In the native runtime (dhrt)

`tiled` runs natively (always included, JSON loader via serde_json) and is **bit-identical** to the Python paths — including external tilesets, tile/object properties and bulk ops (`TILED_FILL_RECT`/`REPLACE`/`COUNT_GID`/`FLOOD_FILL`).
