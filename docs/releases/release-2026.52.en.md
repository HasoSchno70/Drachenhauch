# Drachenhauch 2026.52

*The notes for this version. It is the version **of the open door**: whoever
sees Drachenhauch for the first time has a program of their own in five
minutes, commands take named arguments, dhrt speaks the language of the
system, an exported "Hello world" is 16 instead of 45 MB, and a page says
what stays fixed from version to version. What existing programs may notice
comes first.*

## What changes for existing programs

* **Messages in the language of the system.** Without `DHRT_LANG`, dhrt
  writes its messages in German on a German system and in English on any
  other (before: always German). What a program sees in a `CATCH` variable
  stays German. If you compare output, set `DHRT_LANG=de`.
* **Without a setting, the IDE and the Studio follow the system as well.** On
  an English Windows they start in English; in the IDE, View → "Sprache:
  Deutsch" switches at the next start, and the Studio follows the IDE's
  setting.
* **`dhrt --export` takes the smallest runtime that fits on its own**
  (before only with `--schlank`, which still works). `--voll` takes the full
  one -- for instance for a program that checks with `IMPORT … OPTIONAL` and
  `MODULE_AVAILABLE` at run time what the build can do.
* **A `.dhc` from another version is rejected** ("Diese .dhc stammt aus dhrt
  …"), instead of running differently without a word. One without a version
  (compiled before 2026.52) runs as before.
* **`DIRLIST(path, "")`** returns all entries (before: none).
* **The signatures name English parameter names** (`color` instead of
  `farbe`) -- in the hover, in completion and in the manuals. Parameters with
  a default are no longer part of the completion scaffold.

## New in this version

### Named arguments for commands

```basic
SCREEN(320, 200, title: "Benannt")
CIRCLE(x: 160, y: 100, r: 40, color: RGB(255, 120, 40))
PRINT MID$("Drachenhauch", start: 0, n: 7)
```

* **Every command** takes `name: value`, the names are in its signature. The
  `$` on the name is optional, a keyword may be a name (`new:`, `to:`), and a
  position left out before it gets its default -- 91 defaults were measured
  against the code. Where there is none, the message says what to pass as
  well.
* **The names are English and the same everywhere** (rule 8 in
  [Command names](../en/builtins-namen.md)); `dhrt pruef parameter` keeps the
  manual's tables in line with the index.
* **Variants merged:** `DRAW_IMAGE` takes `w`, `h`, `angle`, `tint`,
  `flip_x`, `flip_y` and a section (`sx, sy, sw, sh`), `DIR_LIST` takes
  `recursive`. The old commands stay.
* **`ELLIPSE_OUTLINE`, `TRIANGLE_OUTLINE`, `POLYGON_OUTLINE`** now really
  draw thicker with `width` (the signature named it, the code never read it);
  `IMAGE_DRAW_IMAGE` takes `tint:` alone (`qb`/`qh = -1` = up to the edge);
  `RANGE` calls its first parameter `start_or_stop`.

### English on equal terms

* The language follows the system (Windows: the language of the user
  interface, otherwise `LANGUAGE`/`LC_ALL`/`LC_MESSAGES`/`LANG`, on macOS
  without them the system setting). New: `SYSTEM_LANGUAGE$()` and
  `RUNTIME_LANGUAGE$()`.
* The IDE and the Studio pass their language on to programs they start, the
  playground the language of the page.

### Small programs

* New smallest runtime **`dhrt-mini`** (core and machine code, 16 MB) next
  to `dhrt-konsole` (22 MB) and `dhrt-spiel` (35 MB); the full one is 45 MB.
  The export takes the smallest one that lacks none of the program's
  commands, and skips a small runtime of another version.

### Stability and change log

* [Stability](../en/stabilitaet.md) says what stays fixed (language, command
  names with no expiry date, arguments, parameter names, files, exported
  programs, command line) and what may change.
* The [change log](../en/aenderungen.md) collects the section "What changes
  for existing programs" from all release notes (`dhrt doku aenderungen`).

### Your own program in five minutes

* [Quick start](../en/schnellstart.md): playground, install, first game,
  first window, sharing -- with sizes.
* **Three starter projects in the playground** (mini game, form, paint),
  reachable directly with `?beispiel=`, in German and English.
* The website's start page and the README say in one sentence what
  Drachenhauch is, give four reasons and three ways in.
* **Physics in the playground** works again (the web build lacked it since
  2026.21), and when painting, the picture stays.

## Under the hood

**Numbers.** 2662 commands (2660) in 57 modules, 237 examples at the top
level of `examples/`, **6120 cases in 518 test collections** (6085 in 516)
and 732 Rust test functions (728).

* **Named arguments cost nothing at run time:** the compiler puts the values
  into the order of the signature, after that the call looks like a
  positional one.
* **1030 signatures** got English names; where the manual had the better one
  (`key`, `conn`, `handle`), the index followed. A Rust test keeps German
  names out.
* **`dhrt test` plays a German system for its cases**
  (`DHRT_SYSTEM_LANG=de`), so that the expectations hold on English CI
  runners.
* **Every `.dhc` carries the version that compiled it** (`"dhrt"`).
* `dhrt fmt` leaves a keyword used as an argument name in lower case.

## What remains open

* **11 parameters have no default**, because leaving them out means
  something no value says (keeping a window's glow colour, for instance);
  whoever names a parameter after one passes it as well.
* **An English name for the type `GELD`** is missing on purpose: it would
  hide a class of your own called `Money`.
* **Flag emoji** are missing in Segoe UI Emoji -- on Windows two letters
  stand there instead.
* **raylib 6.2** is out, but raylib-rs has no version for it yet.
* The macOS package is not notarized, the Windows installer is not signed
  (SmartScreen warns when downloading).
