# Drachenhauch 2026.53

*The notes for this version. It is the version **of the books**: two new
books are finished -- "Kleine Wunder" teaches the language with programs
worth watching, "Fenster, Formulare, Daten" builds an application first by
hand and then in the Studio. Writing them turned up more than seventy places
where the runtime, the Studio or a message was quietly wrong; they are fixed.
On top of that comes raylib 6.2. What existing programs may notice comes
first.*

## What changes for existing programs

**Compiling**

* **Assigning to a `CONST` is a compile error**, even in code that never
  runs (before: only at run time, and `--check` stayed silent). If a
  parameter, a local variable or a field hides the constant, the assignment
  stays allowed.
* **An ENUM value that does not exist** (`Stoff.SNAD`) is a compile error,
  with a suggestion.
* **A `COROUTINE` variable only takes coroutines.**
* **New warnings:** `a BAND b + c` without parentheses (it reads as
  `a BAND (b + c)`; the same for `BOR`, `BXOR`, `SHL`, `SHR`), a coroutine
  used as a bare statement (it would never run), `TRY … END TRY` without
  `CATCH` and `FINALLY` (it swallows the error, as before).

**Graphics**

* **raylib 6.2** draws differently in a few places where 6.0 was wrong:
  `NGONOUTLINE` with a thickness is as thick as the manual says (before
  about a quarter), circles in images are one column wider on the right,
  the pose of a GLTF animation differs slightly, and the frame timer waits
  actively for about 0.8 ms per frame. Gamepads get raylib's own mappings.
* **`SHADER_LOAD` with a missing file is an error** (with a suggestion)
  instead of returning -1. An unknown uniform in `SHADER_SET…` warns once.
* **`BLEND_MODE("subtract")`** subtracts the colour from the background and
  leaves the alpha alone (before: the other way round, alpha included).
* **`RENDERTARGET_BEGIN` without `RENDERTARGET_END` before `FLIP`** stops
  with a message instead of carrying on black.
* **Point lights** are brighter at a distance (the same fall-off as with
  instancing): at 4 units about half as bright, at 10 a fifth.
  `MODEL_INSTANCED` fades into fog like `MODEL`.
* **`MESH_TORUS` and `MESH_KNOT` are as big as given** -- rings and knots
  appear larger than before.
* **`LOADIMAGE` reloads a file that was written anew** (new handle, the old
  one stays valid) instead of returning the old image from the cache.

**Sound**

* **MOD and XM music is about 5 dB quieter**: the player now applies the
  volume the module itself asks for, and no longer clips.
* **`AUDIO_MOD_REMOVE`** restores the parameter's fixed value instead of
  leaving the modulator's last value in place.
* **`AUDIO_FFT` and `MIC_FFT` decay by the clock.** If you ask more than
  sixty times a second, the bands fall more slowly than before.

**Physics**

* **A removed `physics2d` body reports itself** on every query instead of
  quietly returning (0, 0); `PHYS2D_BODY_EXISTS` asks first.

**User interface**

* **`GUI_VALIDATE_LIVE`** checks a field on leaving only if it changed
  since it got the focus; an empty required field is reported by
  `GUI_VALIDATE`.
* **An empty number field** binds `NULL` instead of an empty text.
* **Row colour and zebra stripes of a table** reach the edge.
* **`GUI_DIALOG`** is at most 640 wide and wraps long lines.
* **Muted text (`muted_fg`) and the selected table row** have a contrast of
  at least 4.5:1 in every theme; some colours are stronger for it.

## New in this version

### Two books

* **[Kleine Wunder](../../buecher/wunder/README.md)** (German) -- the
  language through programs worth watching: a starfield, fireworks,
  fractals, a flock, a theremin, a flight over a landscape, a demo built
  from namespaces. 27 chapters in seven parts, each with the reasons behind
  its decisions, exercises and solutions.
* **[Fenster, Formulare, Daten](../../buecher/anwendungen/README.md)**
  (German) -- a media library, first by hand with the gui module, then in
  the Studio: database, binding, queries, a lookup on the net, reports,
  charts, Excel, accessibility, two languages and an installer.
* Both come with the installer and the release as `.docx` and `.epub`. A
  test collection checks every program, every printed output and every
  message in them against dhrt; the pictures are made from the programs.

### Runtime

* **`DIR_USER_DATA$(app$)`** -- the folder for a program's data in the
  user's profile (APPDATA, Application Support or XDG), created on the
  first call.

```basic
DIM ordner AS STRING : ordner = DIR_USER_DATA$("Mediathek")
PRINT ordner
```

* **gui:** `GUI_TEXT_WIDTH`, `GUI_TABLE_ROW_RECT`,
  `GUI_TABLE_GET_CELL_COLOR`, `GUI_TABLE_GET_ROW_COLOR`,
  `GUI_MENU_ENABLED`; `GUI_STATE$` names tables with their selected row,
  tiles and the last announcement to the screen reader.
* **physics2d:** joints with `PHYS2D_ADD_HINGE`, `PHYS2D_ADD_ROPE`,
  `PHYS2D_REMOVE_JOINT`, plus `PHYS2D_BODY_EXISTS`.
* **tween:** `TWEEN_NEW` takes a function of your own as its curve.
* **`v.x`, `v.y`** read the parts of `VEC2`, `VEC3`, `VEC4` and `QUAT`.
* A FUNCREF in a field can be called like a method; a comprehension fits a
  typed array and the reading array commands.
* **Messages that say more:** a mistyped argument name ("Meintest du
  hoehe?"), an error in a default names the header line, "database is
  locked" says what it waits for, an apostrophe in pasted SQL is spotted, a
  violated foreign key names the table and column, an HTTP error says
  whether the network is missing or the time ran out, `LOADIMAGE` and
  `OPENFILE` name the file.
* **The debugger** (`dhrt debug`, in the IDE and the Studio) shows `Self`
  with its own fields first and no longer stops twice on a line with a
  call.

### Studio

* **Project → Complete language table …** collects every text of the
  project and appends the missing ones to the table.
* **"In the user folder"** on the database element: the built program keeps
  its database in the user's data folder and may therefore live under
  "Program Files".
* **Reports:** a barcode (EAN-13), totals over the whole report, A6 lays out
  on A6.
* **Looks:** style files in the project (`StilLaden`) and classes in the
  inspector; `GUI_SCALE` in `App.Geoeffnet` enlarges the program window.

### Community

* **`dhrt paket`** fetches only one folder of a source with `#folder` -- so
  several packages can live in one repository.
* **Three libraries** in `bibliotheken/`: `highscores`, `menu` and
  `dialogbox`.
* **[Gallery](../en/galerie.md)** and **Discussions** with forms for
  questions, your own programs and ideas.

## Under the hood

**Numbers.** 2672 commands (2662) in 57 modules, 237 examples at the top
level of `examples/`, **6285 cases in 525 test collections** (6120 in 518)
and 741 Rust test functions (732).

* **raylib 6.2** through the `sola-raylib` binding instead of `raylib-rs`
  (which is still at 6.0). Where 6.2 draws differently and 6.0 was right
  (lines in images, rounded outlines, spheres), the old picture is back.
* **Machine code:** a loop that hands over to the VM almost every round
  (for instance with `VEC2`) stays in the VM -- before, it ran up to seven
  times slower with machine code than without.
* **Dependencies** as of 21 November, among them rusqlite 0.43, cranelift
  0.138 and xmrs 0.15.
* The packages for macOS and Linux now include `dhrt-mini` as well.

## What remains open

* There are no **English editions** of the two new books yet.
* **`physics3d`** still quietly returns (0, 0) for a removed body.
* **ureq 3** and **zbus 6** are not taken up (a new interface, and two zbus
  in the tree, respectively).
* The macOS package is not notarised and the Windows installer is not
  signed (SmartScreen warns on download).
