# Sprite editor (`examples/189_sprite_editor.dh`)

A pixel-art editor written in Drachenhauch itself: several frames, layers, a
real selection mask, undo, palettes in GIMP format and outputs the game loads
directly — strip PNG with atlas JSON for `ATLAS_LOAD`, a runnable program for
`SPRITE_NEW`, a state machine for `ANIM_FSM_LOAD` and an animated GIF.

It replaces the former Qt editor `dhsprites`. What that one could do in
addition is listed at the end.

## Starting

In the IDE ([`ide/ide.dh`](ide.md)) via the menu **Tools → Sprite editor**
— it runs there as a separate process. Without the IDE:

```
dhrt run examples/189_sprite_editor.dh
```

The editor starts full screen with an empty sprite of 32×32 pixels. It does
not take a file on the command line, but through **[Oeffnen]** (Open).

## Operation

| Input | Effect |
|---|---|
| Left mouse button | draw with the selected tool |
| Right mouse button (drag) | pan the view |
| Mouse wheel over the canvas | zoom (1 to 32) |
| `P` `E` `F` `L` `R` `O` `Y` `I` `S` `Q` `Z` `V` | choose a tool (see below) |
| `T` | 3×3 tile view on/off |
| `1`..`6` | palette colour 1 to 6 |
| `+` / `-` | brush larger / smaller (1 to 6 pixels) |
| `Strg+Z` / `Strg+Y` | undo / redo (24 steps) |
| `Strg+C` / `Strg+X` / `Strg+V` | copy / cut / paste the selection |
| `Strg+D` | clear the selection |
| `Entf` | empty the selection |
| `ESC` or the window's close button | quit |

(`Strg` = Ctrl, `Entf` = Del.)

If the sprite has changed since it was last saved as `.dhsprite`, ESC and the
close button ask first (**Sichern | Verwerfen | Abbrechen** — Save | Discard |
Cancel). An export as a strip or PNG does not count as saved — the layers are
gone there.

## Tools

The toolbar at the top carries all twelve as buttons, next to them
**[Zurueck]**/**[Vor]** (Undo/Redo) and the switches **gefuellt** (filled),
**Spiegel X**, **Spiegel Y** (mirror X/Y), **Zwiebelhaut** (onion skin) and
**Raster** (grid).

| Key | Tool | Effect |
|---|---|---|
| `P` | Pen | set pixels, brush size 1..6 |
| `E` | Eraser | make pixels transparent |
| `F` | Fill | fill a connected area of the same colour and opacity |
| `L` | Line | drag a line |
| `R` | Rectangle | rectangle — outline, filled with **gefuellt** |
| `O` | Ellipse | ellipse — outline, filled with **gefuellt** |
| `Y` | Spray | random pixels within the brush radius |
| `I` | Eyedropper | pick a colour from the image |
| `S` | Selection | rectangle selection (a click without dragging clears it) |
| `Q` | Lasso | freeform selection: trace around the area, releasing closes it |
| `Z` | Magic wand | select all connected pixels of the same colour |
| `V` | Move | lift the selection and put it down elsewhere — without a selection, the whole layer |

**Spiegel X/Y** also paint mirrored while drawing (for figures, symbols,
logos). **Zwiebelhaut** (onion skin) puts the previous frame faintly under the
current one.

### The selection is a mask

Rectangle, lasso and magic wand all write into the same **pixel mask**, not
into a frame — and everything that draws asks it. As long as a selection
exists, no stroke lands outside it. Copy, cut and Del only affect the
selected pixels; when moving, the content is lifted (a hole remains at the
old place, not a copy) and the mask moves along. Semi-transparent pixels keep
their opacity in the process.

## Left column: colour, palette, canvas

- **Farbe** (Colour) — a colour picker, below it the **palette** with 16
  slots.
- **[Palette laden]** / **[Palette sichern]** (Load/Save palette) — GIMP
  palettes (`.gpl`), the format spoken by GIMP, Aseprite, Krita and the
  palette collections on the web. When loading, malformed lines are skipped
  instead of reported (the files are edited by hand); if the file has more
  than 16 colours, the status line says how many did not fit.
- **Pinsel** (Brush) — size 1 to 6.
- **3×3 tile view** (`T`) — shows the image eight times all around. You only
  see a seam next to its repetition; the zoom refits for this.
- **[Statistik]** (Statistics) — pixels, colours and the most frequent
  colour, in total and per frame.
- **[Zuschneiden]** (Crop) — to the content, measured over **all** layers,
  hidden ones too (otherwise content you just do not see would vanish).
- **[Groesse aendern]** (Resize) — new width and height; an added border is
  transparent.
- **[Spiegeln |]**, **[Spiegeln --]**, **[90 >]**, **[< 90]** — mirror and
  rotate by quarter turns. All four apply to the **whole** sprite (all frames,
  all layers); a rotation swaps width and height. They are **not** in the
  history — it records one layer per step, and an undo afterwards would turn
  back only one. The history is therefore cleared; the opposite direction
  undoes a transformation.
- **[DH-Code]** and **[dhanim]** — see Outputs.

## Right column: frames, layers, preview, ranges

### Frames

**[Neu]** (New) appends an empty frame, **[Kopie]** (Copy) a copy of the
current one, **[Weg]** (Remove) removes it. **[Name]** gives the frame a
name — it becomes its key in the atlas (`ATLAS_DRAW(atlas, "kopf", ...)`
instead of `"bild_0"`). Names are identifiers: letters, digits, `_` and `-`
are allowed, everything else becomes an underscore. A dot would not work,
because the json module reads a key with a dot as a path.

**Dauer** (Duration) sets a time of its own in milliseconds per frame;
**0 means "the tempo slider applies"**. That way a pose is held and the run
in between is not. The time is shown in the frame list, moves along when
copying (the name does not — it is an identifier) and applies to preview and
GIF alike.

### Layers

Layers apply to **all** frames, as in Aseprite: hiding a layer hides it
everywhere. **[Neu]**, **[Weg]** and **sichtbar** (visible). Drawing happens
on the selected layer; the display and all outputs except the `.dhsprite`
show the visible layers combined.

### Preview and ranges

The **preview** plays continuously, the tempo comes from the slider (1 to 24
frames per second); frames with their own duration keep their time.

**Ranges** are named animations — name, from, to, fps, exactly what
`SPRITE_ADD_ANIM` needs. **[Hinzu]** (Add), **[Aendern]** (Change),
**[Weg]**; the preview plays the selected one. If a frame is deleted, the
ranges move along, otherwise the preview would afterwards play something
else.

## Limits

| | |
|---|---|
| Edge length | 4 to 256 pixels |
| Frames | 64 |
| Layers | 8 |
| Ranges | 8 |
| Undo | 24 steps |

A frame of a layer is only created when it is needed — a sprite with one
frame also occupies only one. Only the history slots at sprite size are
allocated in advance; the limit of 256 pixels depends on them.

## Files

### `.dhsprite` — the editor's own format

**[Sichern]** (Save) writes two files: the description `name.dhsprite`
(JSON) and next to it the grid `name.dhsprite.png` — columns are the frames,
rows the layers. Any image viewer opens the PNG; the JSON carries
dimensions, layer names and visibility and, if present, frame names,
individual durations and ranges:

```json
{
  "format": "dhsprite-gitter-1",
  "bild": "held.dhsprite.png",
  "breite": 16, "hoehe": 16, "bilder": 4,
  "ebenen": [ { "name": "Ebene 1", "sichtbar": true } ],
  "bildnamen": [ "stand", "", "", "" ],
  "bilddauern": [ 400, 0, 0, 0 ],
  "bereiche": [ { "name": "lauf", "von": 1, "bis": 3, "fps": 8 } ]
}
```

The last three blocks are optional. When loading, the grid is looked for
**next to** the description, not under the name entered — so a moved pair
stays together.

This editor does not open files of the former Qt version (JSON with base64
pixels).

### Opening

**[Oeffnen]** (Open) accepts two things, the extension decides: a
`.dhsprite` with all layers, or a strip PNG with its atlas JSON next to it.
With a strip, the frames come back on one layer — it **is** the combined
image; the frame names come along via the atlas keys.

## Outputs

| Button | Result |
|---|---|
| **[PNG]** | the current frame (visible layers) as PNG |
| **[Streifen]** (Strip) | all frames side by side as PNG, plus the atlas JSON of the same name |
| **[GIF]** | animated GIF with transparency; frames with their own duration keep it |
| **[DH-Code]** | a runnable program `name.dh` together with the sheet `name.png` |
| **[dhanim]** | the ranges as a state machine `.dhanim` |

### Strip + atlas

The JSON has exactly the form that `ATLAS_LOAD` reads — the key is the frame
name, otherwise `bild_<n>`; a duplicate name gets `_<n>` appended:

```json
{
  "image": "held.png",
  "sprites": {
    "stand":  [0,  0, 16, 16],
    "bild_1": [16, 0, 16, 16]
  }
}
```

In the game:

```basic
SCREEN(320, 200, "Atlas", 1)
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("held.json")
WHILE NOT QUITREQUESTED()
    CLS(BLACK)
    ATLAS_DRAW(atlas, "stand", 40, 40)
    ATLAS_DRAW(atlas, "bild_1", 60, 40)
    FLIP()
WEND
```

### DH code

**[DH-Code]** writes program and sheet in one go and under the same name —
anyone who had only the code would have a reference into the void. The
program loads the sheet with `SPRITE_NEW`, adds one `SPRITE_ADD_ANIM` line
per range (without ranges an `"idle"` over all frames at the slider's tempo)
and plays the first one.

### dhanim

**[dhanim]** writes one state per range; the first is the start state
(without ranges an `"idle"` over everything). Transitions and parameters stay
empty — when which state changes into which is a statement about the game.
`ANIM_FSM_LOAD` loads the file as it is; you add the transitions in the
[animation FSM editor](anim-editor.md).

## Typical workflows

### Walk cycle with a held pose

1. **[Neu]** (New) in the toolbar, 16×16.
2. Draw frame 1, press **[Kopie]** (Copy) three times on the right and vary
   them; with **Zwiebelhaut** (onion skin) you see the previous frame
   underneath.
3. Give frame 1 a **Dauer** (duration) of 400 ms; the others follow the
   tempo.
4. Create a range `lauf` from 2 to 4.
5. **[Sichern]** (Save) as `.dhsprite` (working state), **[DH-Code]** for a
   program that runs immediately.

### Tile set

1. 16×16, one tile per frame, each named with **[Name]**.
2. `T` for the tile view — does it tile without a seam?
3. **[Streifen]** (Strip) writes PNG and atlas; in the game
   `ATLAS_DRAW(atlas, "gras", x, y)`.

## What the former Qt version could do in addition

- Export scaling (1×–8×) for all image outputs.
- Sheet import of an arbitrary PNG with the frame size given, pasting an
  image from the system clipboard as a new frame.
- Layers per frame with opacity, reordering and merging.
- Frame operations on the frame list: reverse, append ping-pong, reduce to
  the current frame.
- Replace colour, take the palette from the sprite, adjustable onion skin
  (opacity, up to three frames in each direction), file browser.

## Testing

The promises of this page are checked by
[`tests/pruef/werkzeug_sprite.dhtest`](../../tests/pruef/werkzeug_sprite.dhtest)
with real mouse paths via recording playback: selection, lasso and magic
wand, moving, palettes against a foreign reader, statistics, cropping,
ranges, DH code and `.dhanim` (started or loaded by the runtime), atlas
keys, rotating and mirroring, individual durations in the GIF and the limits
of 64 frames and 8 layers.
