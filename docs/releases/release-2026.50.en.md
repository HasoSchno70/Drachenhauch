# Drachenhauch 2026.50

*The notes for this version. It is the version **of the frugal and
accessible interface**: programs sleep while nothing happens (the IDE needs
a twelfth of the CPU time when idle), the stylesheet adapts to window size,
theme and high contrast, a built-in check finds what makes an interface
hard to use, right-to-left windows are now mirrored inside as well, and
emoji come in colour. What existing programs may notice comes first.*

## What changes for existing programs

* **Coloured buttons have the more readable text.** On a button with its
  own background colour (primary, success, warning) the gui now picks the
  text by the higher contrast: on the cyan of the dark theme, on green and
  orange there is dark text instead of white (before 2.1:1 to 3.4:1). The
  accent of `modern_light` is slightly darker (`#1D6DCE`).
* **Emoji come in colour.** The text colour no longer applies to them, only
  its opacity.
* **Mirrored windows (`GUI_WINDOW_RTL`)** now also mirror text areas, tables
  and trees inside, and their scroll bars sit on the left.
* **`WINDOW_WAIT_EVENTS`** waits per frame exactly as long as nothing is
  pending: after an input it runs on for 0.35 s without waiting, with a
  fixed frame count (`DHRT_FRAMES`) it never waits.
* **Caret and tooltip** count in seconds instead of frames -- at 60 frames
  a second as before.

## New in this version

### Sleeping while nothing happens

```basic
IMPORT "gui"
SCREEN(420, 200, "Ruhig", 1)
DIM w AS GUI_WINDOW : w = GUI_WINDOW("Ruhig", 10, 10, 400, 180)
DIM name AS GUI_WIDGET : name = GUI_TEXTINPUT(w, 20, 20, 240, 28, "Name")
WINDOW_WAIT_EVENTS(1)
WHILE NOT QUITREQUESTED()
    GUI_UPDATE()
    CLS(&H202830)
    GUI_DRAW()
    FLIP()
WEND
```

* With `WINDOW_WAIT_EVENTS(sekunden)` `FLIP` sleeps until the next input --
  and the gui wakes the program itself when it needs a frame: to flip the
  caret, for a tooltip that is due, a toast, an animation.
  `GUI_NEXT_FRAME()` and `WINDOW_IDLE_WAIT()` tell how things stand.
* **IDE and Studio** sleep like this while no program runs: measured, the
  IDE needs 2.2 % of a core when idle instead of 27.5 %.
* **Programs from the Studio** sleep with `Self.Leerlauf = TRUE` in
  `App.Geoeffnet`; timers wake them in time.

### Stylesheet by situation

* In front of a rule there can now also be `@breite < 600`,
  `@hoehe >= 500`, `@dunkel` / `@hell`, `@hochkontrast` and `@rtl` /
  `@ltr` -- evaluated per window, combinable with `@zeit` and `@datum`.
  When a window becomes narrow or the theme dark, the gui resolves anew.
* High contrast comes from the system (Windows, macOS, GNOME);
  `GUI_SET_CONTRAST` overrides, `GUI_HIGH_CONTRAST()` and
  `GUI_SYSTEM_HIGH_CONTRAST()` ask.

### Checking accessibility

* **`GUI_A11Y_CHECK$([win])`** reports line by line what makes an interface
  hard to use: too little text contrast (WCAG 4.5:1), a widget without a
  name for the screen reader, targets too small (WCAG 2.5.8, with the
  spacing exception) and a tab order that jumps against the layout. The
  inspector names the findings for the selected widget.
* A button with only an icon is now called like its tooltip for the screen
  reader.

### Right to left, inside as well

* In a mirrored window text areas are right-aligned (long lines show their
  end on the right), tables start with their first column on the right,
  trees indent from the right; clicks, selection, reordering, column width
  and arrow keys follow. `GUI_SET_ALIGN` now also applies to text areas,
  `GUI_TABLE_GET_COL_WIDTHS` asks for the column widths.

### Emoji in colour

* In `TEXT` as in the gui: the runtime draws the colour glyphs of the emoji
  font itself (COLR for Segoe UI Emoji, bitmaps for Apple Color Emoji and
  Noto Color Emoji). Sequences with a joiner and skin tones are put
  together into one character by text shaping.

## Under the hood

**Numbers.** 2514 commands (before 2507) in 57 modules, 236 examples at the
top level of `examples/`, **6039 cases in 508 test collections** (6004 in
503) and 709 Rust test functions (701).

* **One source per position:** the gui reports its frame needs at the place
  that draws by the clock; the lines of a text area align in one place
  (`ta_versatz`), the columns of a table mirror in `col_x`.
* **Findings of the own check:** eight contrast errors in the built-in
  themes, fixed; all six themes pass `GUI_A11Y_CHECK$`.

## What remains open

* **Lists** do not yet mirror their inside in a mirrored window.
* **Flag emoji** are missing from Segoe UI Emoji -- on Windows two letters
  appear there. Colour emoji on macOS and Linux are built, but only looked
  at on Windows.
* **High contrast** is only checked against the real system on Windows.
* **raylib 6.2** is out, but raylib-rs has no version for it yet.
* The macOS package is not notarised, the Windows installer is not signed
  (SmartScreen warns on download).
