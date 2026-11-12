# Drachenhauch 2026.47

*The notes for this version. It is the version **for a calmer, more
professional IDE**: a review of the interface in thirteen points, two bugs
that showed up right after 2026.46, and eight new queries for the gui with
which a program can read back what it has set. What existing programs may
notice comes first.*

## What changes for existing programs

* **A click goes to the topmost widget.** When two widgets overlapped, the
  one created first used to get the click, while the one created last was
  drawn on top -- that is why the OK button in the "About Drachenhauch"
  window could only be pressed with Enter. Now clicking, drawing and
  `GUI_HIT_TEST` all use the same order. Whoever (unintentionally) relied on
  a widget underneath getting the click creates it last.
* **A label draws its own background and border:**
  `GUI_SET_COLOR(lbl, "bg", ..)` and `"border"` as well as `bg`/`border` from
  the style sheet did not apply to `GUI_LABEL` so far. A style sheet that
  gives labels a background now shows it.
* **The border of a large focused text area is quieter:** above the metric
  `verlauf_hoehe` the accent mixes with the border colour. An accent set on
  the widget applies unchanged.

## New in this version

### IDE

* **After the start screen** the IDE writes in its own font size again --
  before, the larger font of the start screen stayed.
* **The watermark behind the code is off** (the settings, Ctrl+U, switch it
  back on); the logo is on the welcome page.
* **Breadcrumb bar with folders:** above the code it says `examples ›
  154_chart.dh › sub stil_anwenden` instead of only the file name once more. A
  click on a folder shows it in the project tree.
* **Short status bar:** it names the path from the project; its tooltip
  shows the full path.
* **Problems:** the count is in the tab ("Problems (3)"); without problems
  nothing stands next to it any more (the "none" looked like a second tab);
  the breakdown sits on the right, dimmed.
* **The input line** for the running program only appears while a program,
  the debugger or the terminal runs; otherwise the output gets the space.
* **One tone for all areas** instead of four colours (blue, turquoise, brown,
  violet) around the code; the help for the word under the cursor sits in a
  box of its own below the problem list. Own tones still go through
  `farben.json`.
* **Parameter names at calls** only appear on the line with the cursor
  (`"parameter_nur_zeile": false` in the `ide.json` shows them everywhere
  again).
* **Toolbar with larger icons**, the filters above the project tree and the
  output are search fields with a magnifier and a cross.
* **Error or warning:** the handbook now says what the difference is -- an
  error means the program does not compile; a name that exists nowhere is a
  warning, because the program runs up to that line.

### Studio

* In the inspector the rows are called **"Enter-Knopf" and "Esc-Knopf"**
  ("Enter button"/"Esc button" in English) -- the old labels were cut off.

### gui

* **Eight queries for setters:** `GUI_TOOLTIP$`, `GUI_TEXTINPUT_GET` (the
  same keys as `GUI_TEXTINPUT_SET`, such as `suche` or `maxlaenge`),
  `GUI_TEXTINPUT_PLACEHOLDER$`, `GUI_STATUSBAR_TIP$`, `GUI_TOOLBAR_GET`,
  `GUI_DOCK_TITLE$`, `GUI_TEXTAREA_HINT$` and `GUI_TEXTAREA_HINT_COUNT`. What
  a program sets it can now read back -- before, programs and tests had to
  remember the state themselves.

## Under the hood

**Numbers.** 2441 commands (before 2433) in 57 modules, 233 examples at the
top level of `examples/` (233), **5874 cases in 481 test
collections** (5849 in 480) and 670 Rust test functions (670).

* **Rules for tests sharpened** (`CLAUDE.md`): a counter-check falsifies the
  one place, not the whole change -- if a case then only fails at "variable
  not declared", nothing is proven; what a case measures has to move with the
  change; tests ask the gui for its state through the new queries.

## What remains open

* **Real devices are largely untested** (Raspberry Pi, SPI, camera,
  microphone, Arduino, Modbus) -- feedback with hardware is welcome.
* **Full text shaping** (ligatures of other scripts, Indic scripts) is not
  yet possible in the gui.
* **raylib 6.2** is out, but raylib-rs has no version for it yet.
* The macOS package is not notarised, the Windows installer is not signed
  (SmartScreen warns when downloading).
* The finished books and the intro videos are still in the repo (about
  40 MB); in future they are to hang only as downloads on the release.
