# Tilemap/level editor (`examples/187_tilemap_editor.dh`)

A tool for painting 2D levels from a tileset image onto a grid — with
several layers, object layers for spawns and zones, properties per tile and
several tilesets per map. It is itself a Drachenhauch program and saves
**Tiled JSON**, exactly the format that the [`tiled` module](module-tiled.md)
reads with `TILED_LOAD` (and that [Tiled](https://www.mapeditor.org/) itself
opens).

It replaces the former Qt editor `dhtilemap`; it does not need Python.

## Starting

In the [IDE](ide.md): **Tools → Tilemap editor** (or "Tool: tilemap editor"
in the command palette). From the command line:

```
dhrt run examples/187_tilemap_editor.dh
```

The editor opens full screen and begins with an empty map of 40×30 tiles and
the built-in tileset `examples/assets/editor_tileset.png`.

## Layout

At the top the toolbar (New, Open, Save, DH code, the six tools, Undo/Redo,
switches "Gitter" (grid) and "Nummern" (numbers)). On the left the column
with tileset choice, tile palette, layer list and below it — depending on the
kind of the active layer — either the properties of the selected tile or the
object list. On the right the map, at the bottom a status line.

## Tools

| Tool | Key | Effect |
|---|---|---|
| Pen | `P` | paints the selected tile, also while dragging |
| Eraser | `E` | empties cells, also while dragging |
| Fill | `F` | fills the connected area of identical tiles |
| Rectangle | `R` | dragging fills the rectangle on release |
| Eyedropper | `I` | picks up the tile under the mouse |
| Selection | `S` | drags a rectangle for the clipboard |

The active tool has a coloured background in the toolbar. The eyedropper
also switches to the **tileset** the picked-up tile belongs to, and scrolls
the palette so that it is visible — otherwise the next stroke would paint a
different tile.

**Selection and clipboard:** `Strg+C` (Ctrl+C) copies the selected area,
`Strg+X` (Ctrl+X) cuts it, `Strg+V` (Ctrl+V) pastes it at the selection
(without a selection at the tile under the mouse), `Entf` (Del) empties the
selection. Whatever sticks out over the map edge when pasting is dropped —
the map does not grow in the process. All of this only applies on tile
layers.

**View:** the mouse wheel over the map zooms (1 to 6, the point under the
pointer stays put), the right mouse button or the arrow keys pan.
"Nummern" (numbers) writes the tile number into each cell (from zoom 2;
below that it would not be readable).

**Undo/Redo:** `Strg+Z` / `Strg+Y` (Ctrl+Z / Ctrl+Y) or the buttons, 16
steps. A stroke, a rectangle, a fill is **one** step each.

## Tilesets

The dropdown above the palette chooses which tileset the palette shows;
`+` appends another PNG (at most eight). The runtime assigns each tileset's
`firstgid` itself from the tile count — a range set by hand that overlaps
would silently destroy the mapping of **all** tiles. Painting uses the GID
range of the tileset shown, but each tile is drawn with **its own** tileset;
a layer may contain tiles from all of them.

The palette is a fixed-size viewport (8×4 tiles): a larger tileset is
scrolled with the mouse wheel, sideways with Shift.

**Removing** a tileset again is deliberately not offered: the GIDs after it
would shift, and every tile painted with them would afterwards silently point
to a different image.

If a tileset's image is missing when opening, the status line reports it, and
its tiles stay empty instead of being filled with a foreign image.

## Layers

The list shows the layers in **drawing order**: the first lies at the back,
further down means further to the front. Below it:

- **Anlegen** (Create), **Umbenennen** (Rename — name from the field above),
  **Weg** (Remove — the last layer stays),
- **Nach hinten** / **Nach vorne** (Backward / Forward) — named after their
  effect in the picture, not after the list. The tiles stay with their layer,
  and a recorded undo step moves along instead of writing into a foreign
  layer afterwards,
- **sichtbar** (visible) — Tiled saves the visibility too, so it is more
  than a display setting,
- **Objekt-Ebene** (Object layer) creates a layer without tiles.

## Object layers

The **kind of the active layer decides what the mouse does** — a second
toolbar would be a switch you forget. On an object layer:

- **Dragging** creates a rectangle (zone, trigger),
- **Clicking** without dragging creates a **point** (in Tiled an object with
  width and height 0, for example a spawn),
- a click on an existing object **selects it** instead of laying a new one
  on top.

Name and type of the next object are in the two fields on the left;
**Übernehmen** (Apply) uses them to change the selected one, **Objekt weg**
(Remove object) or `Entf` (Del) removes it. Coordinates are pixels as in
Tiled. In the game you read them with
`TILED_OBJECT_COUNT/NAME/TYPE/X/Y/WIDTH/HEIGHT`.

## Properties per tile

On a tile layer the left column shows the properties of the tile selected in
the palette. They belong to the **tileset**, so they apply to every
occurrence of that tile.

- **solid** is a checkbox of its own — this is exactly the property that
  `tile_collide` asks for. When **unchecked** it is **removed**, not set to
  FALSE: as soon as any tile carries a `solid`, only those with
  `solid = TRUE` block; a leftover FALSE would keep that switch flipped
  without anyone seeing it.
- Arbitrary **keys/values** via the two fields and **Setzen** (Set). The
  value is interpreted: `true`/`wahr` and `false`/`falsch` become BOOLEAN, a
  number with a dot FLOAT, without a dot INTEGER, anything else text (`1-2`
  stays text).
- **Weg** (Remove) removes the property selected in the list.

## Saving, opening, quitting

**Speichern** (Save, `Strg+S`) writes Tiled JSON; the first time, a file
dialog asks for the name, after that the same file is overwritten.
**Öffnen** (Open, `Strg+O`) reads Tiled JSON, also from Tiled itself.
**Neu** (New) asks for width and height (4 to 128).

The window's close button and `ESC` ask (Sichern | Verwerfen | Abbrechen —
Save | Discard | Cancel) if the map is not saved; otherwise the editor ends
at once.

Loading the map in your own game:

```basic
IMPORT "tiled"
DIM lvl AS TILED_MAP
lvl = TILED_LOAD("level.json")
PRINT TILED_WIDTH(lvl); "x"; TILED_HEIGHT(lvl)
```

## DH code

**DH-Code** (DH code) writes three files under **one** name: the program
(`karte.dh`), the map as Tiled JSON (`karte.json`) and every tileset image
(`karte_1.png`, `karte_2.png`, …). The image comes along even though the map
names its path: the path points to where the tileset lay while editing, and
anyone passing the folder on would otherwise have a map pointing at nothing.

The program loads everything, maps each GID to its tileset via the
`firstgid` chain and draws every visible tile layer with `DRAWIMAGEPART`; it
does not draw object layers. It runs unchanged with `dhrt run karte.dh`.

## Limits

What the Qt version could do and this editor cannot:

- The tile size is fixed at **16×16** (`CONST KACHEL`). A map with a
  different tile size is cut up wrongly.
- A map has at most **128×128** tiles; a larger one is rejected when opening.
  An existing map cannot be enlarged.
- No **Save as**: after the first save, it always goes to the same file.
- Objects have **no properties** and cannot be moved with the mouse — only
  renamed, retyped and removed.
- **Undo** covers only tile changes, not layers, objects and tile
  properties.
- No dimming of the other layers, no removing a tileset (see above).

## Testing

`tests/pruef/werkzeug_tilemap.dhtest` drives the editor with real clicks and
keys: DH code including the image of the generated renderer (empty and
painted map), the interpretation of property values, setting and removing
`solid`, object layers (drag, click, select, `Entf`), several tilesets with
eyedropper and palette scrolling, reordering the layers including undo. The
written file is read with the json module according to the rules of the
Tiled format, not with the editor's own writer.
