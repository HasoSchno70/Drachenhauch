# Drachenhauch 2026.48

*The notes for this version. It is the version **of real windows**: a gui
window can become a window of the system of its own -- beyond the edge of
the program, onto the second screen, as a modal dialog above the main
window; dock panels tear off and dock back. Plus large tables that stay
smooth even with a million rows, eleven queries, undo as a command and the
system's look. What existing programs may notice comes first.*

## What changes for existing programs

* **During a modal dialog its own menus work.** Until now every menu was
  locked during a modal dialog -- including the context menu and the menu
  bar in the dialog itself. Now the dialog's work, those of the windows
  behind it stay locked.
* **`GUI_DOCK_WHERE$` knows a new place:** `fenster` for a torn-off panel.
  A program that evaluates the answer with `SELECT CASE` sees this value
  only when a panel is torn off.
* **Recordings (`AUTOMATION_RECORD`)** write additional lines `n ...` at the
  end of the file for real windows. raylib's reader skips them; old
  recordings play unchanged.

## New in this version

### Real windows

```basic
IMPORT "gui"
SCREEN(800, 600, "Editor")
DIM palette AS GUI_WINDOW : palette = GUI_WINDOW("Werkzeuge", 520, 10, 220, 300)
IF NOT GUI_WINDOW_NATIVE(palette, TRUE) THEN PRINT "bleibt im Hauptfenster"
```

* **`GUI_WINDOW_NATIVE(win, on)`** turns a gui window into a window of the
  system -- with the system's title bar and close box, a taskbar entry of
  its own, free beyond the edge and onto the second monitor. Widgets,
  events, style sheet and data stay the same. `GUI_WINDOW_IS_NATIVE` asks.
* **Modal dialogs as real windows:** the main window becomes the owner (the
  dialog stays above it, without a taskbar entry of its own; on Linux with
  X11 via `WM_TRANSIENT_FOR`), the main window is dimmed, Enter and ESC
  work in the dialog.
* **Context menu, tooltip, suggestions, month, cell choice** and the
  composition of an input method appear in the window they belong to.
* **Screens with different scaling:** a window on a 150 % screen next to a
  100 % one is drawn correspondingly larger there, text set sharp;
  `GUI_WINDOW_NATIVE_SCALE` tells the factor.
* **Dock panels tear off:** dragged beyond the edge of the main window, a
  panel becomes a window of its own; dragging its head over an area docks
  it back. This holds for both kinds -- panels with a window of their own
  (`GUI_DOCK_WINDOW`) and panels made of widgets of the host window
  (`GUI_DOCK_ADD`, as in the IDE and the Studio).
  `GUI_DOCK_MOVE(..., "fenster")` tears off from the program,
  `GUI_DOCK_STATE$` remembers the position.
* **Screen readers** find the widgets in the window they are in and operate
  them there; **recordings** include the real windows.
* **Where there are no real windows** (browser, macOS for now, without a
  screen), everything stays in the main window -- a program runs the same
  everywhere.
* New example: `examples/219_echte_fenster.dh` (palette, modal dialog,
  tear-off panels).

### IDE and Studio

* **IDE:** the settings (Ctrl+U) can be a window of their own -- switch
  "As a separate window", off by default. The IDE's panels (outline,
  output, problems ...) can be dragged out.
* **Studio:** the report preview has a button "As a separate window" --
  onto the second screen while the Studio stays free beside it.

### gui

* **Large tables:** measured with a million rows of five columns -- setting
  a cell (also sorted or filtered) 3 ms instead of 1.4 s, sorting 0.13 s
  instead of 1.3 s, filtering 0.04 s instead of 0.14 s, the table's memory
  about a quarter less. Only the visible rows are drawn anyway.
* **Eleven queries for setters:** `GUI_GET_ALIGN$`, `GUI_GET_ANCHOR$`,
  `GUI_GET_FONT_SIZE`, `GUI_GET_ICON$`/`GUI_GET_ICON`, `GUI_GET_ROUND`,
  `GUI_GET_MIN_W`/`_H`, `GUI_GET_WRAP`, `GUI_GET_TAB_INDEX`, `GUI_STYLE_GET`.
* **Undo as a command:** `GUI_UNDO`, `GUI_REDO`, `GUI_CAN_UNDO`,
  `GUI_CAN_REDO`, `GUI_UNDO_CLEAR` -- the same path as Ctrl+Z; a menu
  "Edit → Undo" can now grey out properly.
* **The system's look:** `GUI_SYSTEM_DARK`, `GUI_SYSTEM_ACCENT` and
  `GUI_THEME_PRESET("system")` follow the dark mode and the accent colour
  of Windows, macOS and Linux.

## Under the hood

**Numbers.** 2462 commands (before 2441) in 57 modules, 234 examples at the
top level of `examples/` (233), **5919 cases in 483 test collections**
(5874 in 481) and 682 Rust test functions (670).

* **Test helpers for real windows:** window messages to a particular window
  (the window sender knows `+` for messages in the same frame), pixels from
  a covered window (`PrintWindow`), a window's owner, a foreign UIA reader
  per window.
* **Findings while building:** a press holds the pointer (Windows sometimes
  reports leaving before the press); when a program moves a window under
  the pointer, Windows reports the real pointer afterwards.

## What remains open

* **Popups beyond the edge of a real window** do not exist yet.
* **macOS:** real windows stay with the fallback there until they are
  checked on a Mac; screen readers on macOS and Linux are unchecked.
* **Tables without a memory limit** (a virtual mode that fetches rows only
  on request) are still missing.
* **Full text shaping** (ligatures of other fonts, Indic scripts) is not yet
  possible in the gui.
* **raylib 6.2** is out, but raylib-rs has no version for it yet.
* The macOS package is not notarised, the Windows installer is not signed
  (SmartScreen warns on download).
