# Drachenhauch Studio

Drachenhauch Studio is an IDE for programs with a user interface, in the
style of Xojo: put windows together with the mouse, set properties in the
inspector, later write the code of the events and start the program. It runs
on the `dhrt` runtime and is itself a Drachenhauch program. The design with
all stages is in [entwurf-studio.md](../entwurf-studio.md) (German).

```text
dhrt run studio/studio.dh
dhrt run studio/studio.dh -- fenster.dhform
```

**Status: stage 1.** Design a window, save it and open it again. Projects,
code and starting come with stages 2 and 3. Until then the form designer
(`examples/197_form_designer.dh`, DH code with Ctrl+G) covers the code.

## The user interface

| Area | what it does |
|---|---|
| Toolbar | New, Open, Save, Undo, Redo, Align, Zoom; Start follows with stage 3 |
| Navigator (top left) | the window and its controls as a tree; a click selects |
| Library (bottom left) | all building blocks of the gui in groups, with a preview; search field above |
| Middle | the window being designed, with title bar and size |
| Inspector (right) | the properties of the selected control, without a selection those of the window |
| Status bar | message, name, position and size of the selected control, zoom (click: 100 %) |

The panels can be moved, stacked as tabs and detached, as in the IDE. The View
menu offers a light and a dark appearance.

## Designing a window

| Action | effect |
|---|---|
| click a building block in the library, then click into the window | places it there (its centre at the click) |
| drag a building block into the window | places it where you let go |
| double-click a building block | places it in the middle of the window |
| click a control | selects it; Shift or Ctrl click adds or removes |
| drag a frame over empty space | selects everything entirely inside |
| drag a control | moves the whole selection |
| drag one of the eight handles | changes the size |
| arrow keys | move by one pixel, with Shift by the grid (8) |
| Del | deletes the selection |
| Ctrl+D | duplicates |
| Ctrl+A | selects everything (not while an input field has the focus) |
| Ctrl+Z / Ctrl+Y | undo / redo |
| Ctrl+mouse wheel, Ctrl+Plus/Minus, Ctrl+0 | zoom |
| Esc | cancels placing, otherwise clears the selection |

**Alignment guides:** while dragging and resizing, edges and centres snap to
those of the other controls. They also snap to the edges and the centre of
the window, 20 pixels from the edge and 12 pixels next to a neighbour. A pink
line shows what they snapped to. Where no guide applies, the grid does.
Holding Alt snaps to nothing. Both can be switched off in the View menu.

**Arrange** (menu and toolbar) aligns the selection with the control clicked
last: edges, centres, same width, height or size. Distribute sets equal gaps
from three controls on. Bring to front and send to back change the drawing
order.

## The inspector

The properties are in groups that fold open and closed: identity, position,
content, appearance, behaviour. Only what fits the kind of control is shown.
Every property has its editor:
- text and numbers in fields;
- switches as toggles;
- choices as dropdowns;
- colours in the colour picker, which opens as a popover ("Vorgabe des
  Themas" removes the colour again);
- the anchor as four checkboxes (which edge the control sticks to when the
  window grows).

A change shows in the window at once and applies to all selected controls
that have the property. An undo step is made when the field loses the focus
or Enter comes, not per keystroke. A name must be an identifier and unique in
the window, because it later becomes a field of the window class; an invalid
name is marked red and not applied.

## The file

The studio saves a `.dhform`, the same format that `GUI_SAVE` writes and
`GUI_LOAD` reads. A `.dhform` from the form designer or from a program opens
in the studio; controls without a name get one when opened. New, Open and
Quit ask when there are unsaved changes.

## For tests

| Environment | effect |
|---|---|
| `DH_STUDIO_LOG=file` | writes what the studio does, line by line, plus the positions of library rows, inspector fields and toolbar buttons |
| `DH_STUDIO_GROESSE=WxH` | a fixed window size instead of maximised |
| `DH_STUDIO_AUFNAHME=file` | plays a recording (`AUTOMATION_PLAY`) |
| `DH_STUDIO_ABLAGE=text` | puts text on the clipboard, for typing with Ctrl+V |

The test collection `tests/pruef/werkzeug_studio.dhtest` starts the studio
twice: once to measure, once with real clicks and keys.
