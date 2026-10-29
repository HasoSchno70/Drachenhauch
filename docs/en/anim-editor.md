# Animation FSM editor — state machines for sprite animations

A **node graph editor** for animation state machines in the style of
Unity's Mecanim, written in Drachenhauch itself:
`examples/198_anim_fsm_editor.dh`. Nodes are **states** (bound to a
sprite animation), arrows are **transitions** with AND-combined
conditions. The result is a `.dhanim` file (JSON) that the runtime module
[`animfsm`](module-animfsm.md) loads with `ANIM_FSM_LOAD`. In the game you
then no longer switch animations by hand — you only set parameters, and the
machine decides the state.

Until 2026-09 there was the Qt editor `dhanim` (Python) for this. It was
removed together with the Python part; 198 reads and writes the same format.

## Starting

In the [IDE](ide.md) via the menu **Tools → Anim FSM editor** (or the
command palette, "Tool: anim FSM editor"). From the command line:

```
dhrt run examples/198_anim_fsm_editor.dh
dhrt run examples/198_anim_fsm_editor.dh -- hero.dhanim
```

A relative file name is taken from the place of the call (`DHRT_START_DIR`).
**Without a file the editor opens the example machine**
`examples/anim_demo.dhanim` — a platformer hero with idle/run/jump/fall,
the parameters `speed`/`grounded`/`jump` and every kind of transition — so
that you immediately see what a finished graph looks like. `Ctrl+N` clears it.

## User interface

At the top across the full width a menu and a toolbar (New, Open, Save ·
Back, Forward · State, Connect, Delete · Arrange, Fit, zoom · Preview), at the
bottom the status bar with the message, the number of states and transitions
and the zoom (a click on it = 100 %). The interface writes in DejaVu Sans.

- **Centre — the graph:** states as nodes, transitions as arrows with a
  head and a label (the way there and back between the same nodes is offset
  to the side). An entry arrow marks the **start state**; the pill
  **"Any State"** is the source for transitions from anywhere.
- **Left — document and parameters:** sprite sheet (image path, frame width
  and height, preview scale) and the list of **parameters**
  (`bool`/`float`/`int`/`trigger`) with their default value.
- **Right — inspector:** depending on the selection, the state (name,
  animation, frame from/to, frames per second, looping, start state) or the
  transition ("only when finished" and up to **six conditions** made of
  parameter, operator and value). Below it **"So läuft der Zustand"** (how the
  state runs): the animation of the selected state (without a selection that
  of the start state), from the sprite sheet itself, at its frames per second;
  a state that runs only once stays briefly on its last frame and then starts
  again.

## Operation

| Action | How |
|---|---|
| Create a state | double-click on free space, or `Einfg` (Insert) |
| Move a state | drag the node (grid 8); the arrow keys nudge the selected one |
| **Drag a transition** | **with the right mouse button** from one node onto another (also from "Any State"); or `L` for connect mode, then with the left button |
| Edit a transition | click the arrow → inspector; `+ Bedingung` (add condition) appends a row, `x` removes it |
| Edit a state | click the node → inspector |
| Start state | in the inspector, or **Bearbeiten → Als Startzustand** (Edit → Set as start state) |
| Parameters | `+`/`-` on the left, name/type/default; renaming carries the conditions along, deleting removes them |
| Delete | `Entf` (Del) — a state takes its transitions with it |
| Undo / Redo | `Strg+Z` / `Strg+Y` (Ctrl+Z / Ctrl+Y) |
| New / Open / Save / Save as | `Strg+N` / `Strg+O` / `Strg+S` / `Strg+Umschalt+S` (Ctrl+N / Ctrl+O / Ctrl+S / Ctrl+Shift+S) |
| Preview | `F5` |
| Zoom | mouse wheel over the graph (around the point under the mouse), `Strg+Plus` / `Strg+Minus` (Ctrl+Plus / Ctrl+Minus), `Strg+0` (Ctrl+0) = 100 % |
| Pan | drag with the middle mouse button, or hold Space and drag with the left |
| Arrange | `Strg+L` (Ctrl+L): levels from the start state, one column per level — what "Any State" reaches stands in the first level after it; one step in the history |
| Fit | `Strg+E` (Ctrl+E): the whole graph into view (automatically on opening) |
| Quit | `Strg+Q` (Ctrl+Q) or the window's close button |

When you quit with unsaved changes, the editor asks
(Sichern | Verwerfen | Abbrechen — Save | Discard | Cancel); if everything is
saved, it ends at once.

### Preview (F5)

`F5` writes `<name>_vorschau.dh` next to the `.dhanim` and starts it with
`dhrt`: on the left one control per parameter (via the `ui` module — sliders
for `float`/`int`, checkbox for `bool`, button for `trigger`), on the right
the sprite playing the **current state**. That way you test the transitions
without writing game code. The sprite sheet is looked up as entered, next to
the `.dhanim` or from the start folder, and written into the preview program
as an **absolute** path. If the machine is unsaved, `F5` saves it first (one
that has no name yet asks for a file name) — the preview lies next to the
file. A second `F5` ends the running preview and starts the new one.

## Use in a game

```basic
IMPORT "animfsm"
IMPORT "sprite"

SCREEN(320, 240, "Held")
DIM hero AS SPRITE
hero = SPRITE_NEW(LOADIMAGE("assets/hero_walk.png"), 16, 16)
DIM fsm AS ANIM_FSM
fsm = ANIM_FSM_LOAD("assets/hero.dhanim")
ANIM_FSM_SETUP(fsm, hero)

DIM vx AS FLOAT
WHILE NOT QUITREQUESTED()
    vx = 0.0
    IF KEYPRESSED(KEY_RIGHT) THEN vx = 2.0
    ANIM_FSM_SET_FLOAT(fsm, "speed", ABS(vx))
    IF KEYHIT(KEY_SPACE) THEN ANIM_FSM_TRIGGER(fsm, "jump")
    ANIM_FSM_UPDATE(fsm, hero, DELTA() * 1000.0)
    CLS(BLACK)
    SPRITE_DRAW(hero)
    FLIP()
WEND
```

The file format and all commands are described in
[module-animfsm.md](module-animfsm.md).

## How it is built

**The model is the `.dhanim` JSON** (json module), the window only the
view. Every change writes into the JSON; undo is one JSON text per state.
Whatever the inspector does not know passes through unchanged. The graph is
painted with the ordinary drawing commands (`LINEW`, `TRIANGLE`, `BOXROUND`)
in a `SCISSOR` area between two `gui` windows; drawing and hit testing use
the same geometry (a click hits an arrow if it lies at most 7 points beside
the line segment).

Two things that are not obvious:

- `first`, `last`, `x` and `y` must stay **whole numbers** — the `animfsm`
  module reads them as integers; `3.0` would become "not set", and the state
  would have no frames. The editor therefore writes them everywhere with
  `JSON_SET_INT`.
- The "Any State" pill is not stored in the file; the editor remembers its
  position itself.

You drag transitions with the **right** mouse button instead of through a
mode switch as in the old Qt editor: a switch is something you forget to
flip. Connect mode (`L`) is now only the way for those without a right
button.

## Tests

`tests/pruef/werkzeug_animfsm.dhtest` drives the editor with real clicks
from a recording: a double-click creates a state, the right mouse button
drags a transition, dragging and pressing `Strg+Z` (Ctrl+Z) twice, `Entf`
(Del) including transitions, a condition via the inspector, a new parameter,
and `F5` writes a preview that compiles **and** runs. The saved file is
checked with the runtime itself — `ANIM_FSM_LOAD`, `ANIM_FSM_SETUP`, one
step, compare the state name (for the condition: with `speed` 10, `run` is
reached; with 1, it stays `idle`). That the JSON is valid would be the
weaker statement: a transition to a deleted state does not load, and that is
exactly what the case checks after `Entf`. Plus arranging (positions by
level and undo), zooming around a fixed point including hit testing, a real
wheel step, fitting and the animation in the inspector as an image probe. The
prompt on the close button is
checked by `tests/pruef/werkzeug_kreuz.dhtest` with real window messages.

## What the Qt editor had and 198 lacks

- **Renaming and setting the start state by right-clicking a node** — here
  the right button drags transitions; both are done via the inspector or the
  Bearbeiten (Edit) menu.
