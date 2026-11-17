# Drachenhauch 2026.51

*The notes on this version. It is the version **of names and calm**:
commands get a naming guide and, for the old names, new ones you can
predict; your own names must be written the way they were declared; the IDE
becomes calmer and gets the Drachenhauch mist; surfaces get gradients; and
Circuit Runner becomes prettier and more leisurely. What existing programs
may notice comes first.*

## What changes for existing programs

* **New warning: spelled differently from the declaration.** Whoever
  declares `CONST CW` and later writes `cw` gets a warning when compiling
  (on stderr for `dhrt run`, the yellow line in the editor). The program
  runs as before -- upper/lower case still does not matter. Ctrl+. or
  `dhrt fmt --schreibweise` writes the names as declared.
* **A label measures its width by its text** until `GUI_SET_BOUNDS` or
  `GUI_SET_WRAP` sets one; `GUI_GET_W` returns the real width.
* **The font of a button with a gradient** follows the least favourable
  spot of the gradient, and `GUI_A11Y_CHECK$` measures there.
* **Lists in a mirrored window** (`GUI_WINDOW_RTL`) now mirror their inside
  as well.
* **Circuit Runner:** the robots move half as often as before (key T in the
  menu switches to the original); recorded solutions only fit at the
  original speed.

## New in this version

### Command names you can predict

```basic
IMPORT "time"
PRINT TIME_TEXT$(TIME_NOW())
PRINT FILE_EXISTS("notizen.txt")
PRINT ENV_GET("HOME", "?")
```

* **A naming guide** ([command names](../en/builtins-namen.md)): split by
  underscores, the area first (`IMAGE_LOAD`, `WINDOW_SET_TITLE`), `_DOWN`
  means held, `_HIT` just pressed, `_FREE` releases, `_COUNT` is a number of
  things, commands are English.
* **143 new names for old commands**, such as `KEY_DOWN` instead of
  `KEYPRESSED` (which meant "held" while `INPUT_PRESSED` meant "just
  pressed"), `GUI_LISTBOX_ITEMS` instead of `GUI_SET_LISTBOX`, `MAP_GET`,
  `SCREEN_WIDTH`, `GUI_GET_TEXT$`, `TIME_NOW`, `MONEY_ROUND`. **The old ones
  stay valid**; the hover over an old one shows the new one.
* **The `$` is optional on text commands:** `ENV_GET` is `ENV_GET$`,
  `CLIPBOARD_GET$` is `CLIPBOARD_GET`.
* `IMPORT "time"` and `IMPORT "money"` are the modules `zeit` and `geld`.

### Case does not matter, but consistently

```basic
CONST CW = 4
PRINT cw                       ' Warnung: 'cw' ist als 'CW' angelegt ...
```

* Which declaration is meant is decided by the scope: a local `cw` in a SUB
  is spelled correctly there, even next to a global `CW`. Commands and
  keywords everyone writes as they like.
* Across the whole repository (598 files) the check reports nothing.

### Gradients and the Drachenhauch mist

```basic
IMPORT "gui"
GUI_STYLESHEET("button.los { farbverlauf: #2FA8D8 #7B4FD8 30 wandert 6s }" + CHR$(10) + "panel { nebel: 14% }")
```

* **`farbverlauf`** (gradient) in the stylesheet: 2 to 6 colours, an angle,
  optionally wandering without a seam -- for buttons, panels, cards and any
  other surface.
* **`nebel`** (mist): soft wisps drift across a surface; for the window
  background the metric `nebel`, the colour the theme colour `nebel`. It
  stands still while idle, and entirely with `GUI_SET_MOTION("aus")`.
* **Background windows:** `GUI_WINDOW_BACKGROUND(win, TRUE)` keeps a window
  at the back -- a clicked borderless main window no longer hides the
  others.
* **Tabs at the editor:** theme colour `reiter` and metric `reiter_oben`;
  the status bar accepts a background that was set.

### The IDE

* **Calm look:** dark borders flush below the editor, muted folders, the
  active tab in the colour of the editor, and above it all the Drachenhauch
  mist (can be switched off, setting `nebel`).
* Empty output and problem lists say what to do; the status bar takes on a
  colour while a program runs.
* Dialog titles translated, the English language table without duplicates
  and dead entries.

### Example 222 and Circuit Runner

* **Example 222** shows the gui in one tour.
* **Circuit Runner:** monsters and blocks are classes with inheritance; the
  tiles have 128 pixels instead of 64, the board fills the screen height
  smoothly, plus shadows, glow, sparks and glass cards.

## Under the hood

**Numbers.** 2660 commands (2514 before, of which 143 are new names for old
ones) in 57 modules, 237 examples at the top level of `examples/`, **6085
cases in 516 test collections** (6039 in 508) and 728 Rust test functions
(709).

* **New names without a new runtime:** the compiler maps a new name to the
  old one (`namen.rs`); runtime and machine code are untouched.
* **A find along the way:** the scanner for the editor and renaming took the
  `\"` in `!"…\"…"` for the end of the string.
* An array literal of GUI handles (`[a, b]`) fits `ARRAY OF GUI_WIDGET`.

## What remains open

* **An English name for the type `GELD`** is missing on purpose: it would
  hide a class of your own called `Money`.
* **Named arguments** do not exist yet for built-in commands.
* **Flag emoji** are missing in Segoe UI Emoji -- under Windows two letters
  stand there.
* **raylib 6.2** is out, but raylib-rs has no version for it yet.
* The macOS package is not notarised, the Windows installer not signed
  (SmartScreen warns when downloading).
