# Drachenhauch 2026.49

*The notes for this version. It is the version **of the living interface**:
buttons in shapes of their own, skins from images and animated GIFs,
animations, sounds and sparks from the stylesheet, a command palette in
every program, an inspector, glass, stylesheets by time of day and date,
right-to-left windows and self-drawn widgets. Plus popups beyond the edge
of real windows, tables without a memory limit and full text shaping. What
existing programs may notice comes first.*

## What changes for existing programs

* **Ctrl+Shift+P opens the command palette in every program with a menu
  bar or toolbar.** If the program has a menu shortcut of its own on that
  key, its own wins; `GUI_PALETTE_KEY(FALSE)` switches the key off. While
  the palette is open, no text field below it gets the keys.
* **Programs started from the IDE** open the inspector with
  **Ctrl+Shift+I** (the IDE sets `DHRT_INSPEKTOR=1`). A program started on
  its own notices nothing of it.
* **A click into a right-aligned text field** (`GUI_SET_ALIGN`) now puts the
  caret at the click position; before, it ended up at the end. The
  placeholder of such a field no longer runs over the edge.
* **The IDE's completion** offers only types after `AS` and `OF` --
  `DIM w AS GUI_WIN` used to take the first matching command.

## New in this version

### Look and motion

```basic
IMPORT "gui"
SCREEN(640, 360, "Notizbuch")
DIM w AS GUI_WINDOW : w = GUI_WINDOW("Notizbuch", 10, 10, 600, 320)
DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(w, "Speichern", 20, 20, 160, 44)
GUI_SET_NAME(ok, "speichern")
GUI_STYLESHEET("button { klang: klick }" + CHR$(10) + _
    "button:hover { animation: puls 700ms }" + CHR$(10) + _
    "#speichern { form: pille; effekt: funken }")
```

* **Shapes of their own:** in the stylesheet `form: pille | kreis | raute |
  bild`, in code `GUI_SET_SHAPE`. The shape also counts for hits -- a click
  beside the circle does not hit.
* **Skins from images:** `bild` and `bildrand` (9-slice) per state, also
  for check boxes, radio buttons, switches and a slider's handle; animated
  GIFs and frame sequences from an atlas (`bild: figur.png 8 12fps`) play
  along.
* **Animations:** `puls`, `wackeln`, `schweben`, `huepfen`, `drehen`,
  `blinken` and sequences of your own (`@ablauf`) with position, size,
  opacity and tint -- in the stylesheet or with `GUI_ANIMATE`. Smooth
  layout transitions with `gleiten: 200`.
* **Sounds and particles:** `klang: klick | tick | ping | plopp | fehler |
  erfolg | datei.wav`, `effekt: funken | konfetti | sterne`, in code
  `GUI_SOUND` and `GUI_BURST`. `GUI_SET_SOUNDS` gives a "Sounds" switch.
* **The system's "reduce motion"** (Windows, macOS, GNOME) stops
  animations, GIF skins and particles; `GUI_SET_MOTION` overrides it.
* **Glass:** `unschaerfe: 12` shows the background blurred through.
* **Stylesheet by time of day and date:** `@zeit 20:00-06:00 button { … }`,
  `@datum 12-01..12-26 #titel { … }` -- the gui resolves again as soon as a
  condition flips.
* New examples: `examples/220_formen_und_animationen.dh` and
  `examples/221_klang_funken_palette.dh`.

### Operating and building

* **Command palette** (Ctrl+Shift+P): search over all entries of menus,
  toolbars and context menus; Enter triggers like mouse and shortcut.
  Without a line of code -- `GUI_PALETTE` and friends for the rest.
* **Inspector** (`GUI_INSPECTOR`): the widget under the pointer, the
  matching stylesheet rules with their line and what applies per state.
  `GUI_STYLESHEET_LOAD(pfad$, TRUE)` reloads the stylesheet on save.
* **Self-drawn widgets:** `GUI_CUSTOM(win, x, y, w, h [, rolle$])` is
  painted like a canvas and still has tab focus, clicks, keys
  (`GUI_CUSTOM_KEY`, `GUI_CUSTOM_TEXT$`) and a role for the screen reader.
* **Right to left:** `GUI_WINDOW_RTL` mirrors a window -- positions, menu bar
  and toolbar, the marks of check boxes and switches, a slider grows to the
  left, a text field is right-aligned.
* **Operation as a test case:** `dhrt aufnehmen programm.dh ziel.dhtest`
  records an operation and appends it as a finished case.

### Real windows, tables, text

* **Popups beyond the edge:** context menu, drop-down list, suggestions,
  month and spelling menu of a real window get a window of their own when
  they reach beyond its edge.
* **Canvases in real windows** and torn-off panels:
  `GUI_CANVAS_BEGIN`/`GUI_CANVAS_END`, `GUI_CANVAS_MOUSE`. Palette,
  inspector, glass, opacity and tint work there as well.
* **Virtual tables:** `GUI_TABLE_VIRTUAL(tbl, zeilen)` -- the table asks
  (`GUI_TABLE_NEED_FROM`/`_COUNT`) and only holds what was supplied around
  the visible area (`GUI_TABLE_SUPPLY`).
* **Full text shaping:** Indic and Southeast Asian scripts, ligatures
  (`FONT_LIGATURES`); the caret jumps over whole syllables, synthetic
  italics slant the shaped glyphs.
* **Outwards:** images and file lists via the clipboard,
  `DRAG_OUT_FILES`/`DRAG_OUT_TEXT` drag out of the program (Windows).
* **Studio:** dialogs can be windows of their own (View menu).

## Under the hood

**Numbers.** 2507 commands (before 2462) in 57 modules, 236 examples at the
top level of `examples/` (234), **6004 cases in 503 test collections**
(5919 in 483) and 701 Rust test functions (682).

* **Profile `gegenprobe`:** `python rust/build_runtime.py --gegenprobe`
  builds without LTO -- a rebuild for a counter-check takes about 30
  seconds instead of four minutes.
* **One source per position:** the text position of an input field, the
  shape of a button and the position of a popup are asked by drawing, hits
  and keyboard at the same place.
* **Real windows under `dhrt test`** let the real mouse through -- cases no
  longer depend on where the pointer happens to be.
* **File dialogs:** rfd 0.18; on Linux still the xdg portal with zenity.

## What remains open

* **Right to left:** tables, trees and text areas keep their inside as it is
  (columns, indentation and text from the left).
* **"Reduce motion"** is only checked against the real system on Windows.
* **macOS:** real windows stay with the fallback there until they are
  checked on a Mac; dragging out exists only on Windows.
* **raylib 6.2** is out, but raylib-rs has no version for it yet.
* The macOS package is not notarised, the Windows installer is not signed
  (SmartScreen warns on download).
