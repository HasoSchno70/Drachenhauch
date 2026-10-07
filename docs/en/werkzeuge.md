# The tools around the language

`dhrt` does more than run programs. This page collects what sits next to it:
checking, testing, formatting, looking at what is inside.

An overview of all subcommands is built into the runtime itself:

```bash
dhrt --help
```

## `dhrt --version` — what is inside this binary?

```text
$ dhrt --version
dhrt 2026.8
dabei: grafik, dialoge, datenbank, netz, http, seriell, usb, bluetooth, wlan
```

The second line is not decoration. A build **without** `--hardware` leaves
out `serial`, `usb`, `bt` and `wifi`, and nothing about the program tells you
so — until now the message only came with the first call, deep inside the
program:

```text
fehlt: seriell, usb, bluetooth, wlan (neu bauen mit: python rust/build_runtime.py --hardware)
```

The version comes from `rust/drachenhauch_runtime/Cargo.toml` (written there
with three parts; `2026.13.0` is `2026.13` to the outside world); until
2026-09-21 `pyproject.toml` and the Python version stood next to it.
`tests/pruef/dhrt_werkzeuge.dhtest` checks that the runtime reports what is
written there.

## `dhrt bild` — a picture of the running program

```bash
dhrt bild examples/112_physics2d.dh vorschau.png 90
```

Runs the program for the given number of frames (90 if none is given) and
saves the last one as a PNG. The **window is pushed out of sight** while this
happens — taking a picture should not get in the way of your work; what is
saved is the draw buffer, not the screen.

**After the picture the program ends** (since 2026-09-15). Otherwise the
frame limit only acts through `QUITREQUESTED()`: a program with `WHILE TRUE`
saved its picture and then kept running forever, and whoever was waiting for
the process — the textbook tools `shoot.dh`, the IDE cards — waited forever.

The runtime could already do the same through the environment variables
`DHRT_FRAMES` and `DHRT_SCREENSHOT` (see docs/rust-runtime.md). From inside a
**program**, though, there was no way to get at them: `PROCESS_START` strips
them from the child so that a launched game does not die after N frames. This
is how the IDE makes the preview pictures for its example cards.

## `dhrt test` — running test programs

The building blocks have existed since WP E (`ASSERT`, `ASSERT_COLLECT`,
`ASSERT_REPORT` and the return value through `EXIT`). What was missing was
the roof over them:

```bash
dhrt test                    # unterhalb des aktuellen Verzeichnisses
dhrt test code/              # nur dort
dhrt test code/eins_pruefung.dh
```

```text
  ok      code\abruf_pruefung.dh  (0.82s)
  FEHLER  code\zeit_pruefung.dh  (Rueckgabewert 1, 0.03s)
          FEHLER: 1 von 12 Pruefungen
          FEHL  Zeile 44: Sommerzeit: erhalten 3600, erwartet 7200

2 Datei(en), 1 ok, 1 mit Fehlern  (0.85s)
```

**What gets found is `*_pruefung.dh`** — the rule was read off the existing
code, not invented: `buch-tippspiel/code/` has always named its four test
programs that way. A file named explicitly runs even without this ending;
whoever writes it down means it.

**The return value is 0 only if everything passed** — which makes the call
fit for a chain or a CI. "Nothing found" is *not* a failure, otherwise the
chain would trip over a project that is still empty.

Two things you should know:

* **Each file runs as its own process.** Same reason as with `TASK_START`:
  the process boundary is the promise. A test program that crashes, opens a
  window or leaves globals behind cannot harm the next one.
* **The child's standard input is empty.** A forgotten `INPUT` would
  otherwise wait for input that never comes, and the whole run hangs.

A test program looks like this — aborting (the default) or collecting:

```basic
' Aborts at the first failure:
ASSERT(punkte(3, 1, 3, 1) = 4, "exakter Tipp")
ASSERT_EQ(tendenz(2, 1), 1, "Heimsieg")

' ... or let all checks run and take stock at the end:
ASSERT_COLLECT(TRUE)
ASSERT_EQ(a, b, "was geprüft wird")
ASSERT_REPORT()
IF ASSERT_FAILED() > 0 THEN EXIT(1)
```

### Test collections: many cases in one file (`*.dhtest`)

A test program checks itself with `ASSERT`. Most of the language's tests,
however, are output comparisons: a short program, an expected output. For
those you do not need one program per file, but a **collection** (since
2026-09-07, path D of the [Python removal](../entwurf-python-abbau.md)
(German)):

```dhtest
' Kopfkommentar bis zum ersten Fall
=== Zähler liefert drei Werte
FUNCTION z() AS INTEGER
    YIELD 1
    YIELD 2
END FUNCTION
DIM c AS COROUTINE : c = z()
PRINT CORO_RESUME(c)
PRINT CORO_RESUME(c)
--- erwartet
1
2
=== Division durch Null bricht ab
PRINT 1 \ 0
--- fehler
Division durch Null
```

`=== Name` starts a case, then comes its source code, then sections. The
section markers are literal syntax and stay German:

| Section | Meaning |
|---|---|
| `--- erwartet` | (expected) the output, line by line; blank lines at the end do not count on either side |
| `--- erwartet ungefaehr` | (approximately expected) like `erwartet`, but numbers may differ by 1e-6 (relative or absolute) — for `SIN`, `SQR` and anything that rounds the last digit differently on three operating systems; compared word by word, non-numbers must be equal |
| `--- enthaelt` | (contains) every line of the block must appear in the output |
| `--- fehler` | (error) the program must abort (return value not 0), and every line of the block appears in the message |
| `--- datei name` | (file) a fixture file that lies next to the program before the run (JSON, map, text); a blank line at the end of the block is the final line break — **a recording for `AUTOMATION_PLAY` needs it**, otherwise raylib's reader loses the last line, and that is usually the release of a key |
| `--- datei name base64` | the same fixture file as bytes — for anything that is not UTF-8 (a cp1252 file, a ZIP archive, an image); the block is Base64 and may be wrapped |
| `--- verzeichnis name` | (directory) an empty directory next to the program, for `DIRLIST`, `RMDIR` and anything that wants to see folders |
| `--- umgebung` | (environment) `NAME=WERT` per line, for example `DHRT_FRAMES=1` |
| `--- eingabe` | (input) what the program reads from standard input (`INPUT`, `STDIN()`); a blank line at the end of the block is the final line break, `--- eingabe base64` for bytes |
| `--- argumente` | (arguments) one line per program argument, they end up after `--` (`ARGC`, `ARG$`) |
| `--- rueckgabe 3` | (return) the return value the program must deliver (`EXIT(3)`); 0 if not given |
| `--- stderr` | every line of the block appears in the error output — for `EPRINT`, without the program having to abort (that is what `fehler` demands) |
| `--- system windows` | the case only applies there (also `posix`, `macos`, `linux`, several separated by spaces); elsewhere it counts as skipped — for `SHELL("cmd", "/c", …)` and anything that needs a particular operating system |
| `--- ton datei.wav` | (sound) probes on a WAV file the program wrote (channels, bit depth, duration, peak, level per time window) — see below |
| `--- bild` | (picture) pixel probes on the **screenshot** after the run — the runner sets `DHRT_SCREENSHOT` itself and, if the environment names none, `DHRT_FRAMES=2`; `--- bild name.png` checks a file the program wrote instead (`IMAGE_SAVE`) |
| `--- programm pfad [nach MARKE]` | (program) the case runs an **existing program** (path relative to the collection — the IDE, an editor); its source code is inserted after the first line `MARKE`, or at the beginning without a marker. The program runs as a copy in `_programm/` in the case folder; the case folder itself is the caller's location (`DHRT_START_DIR`) |
| `--- ersetzen ALT` | (replace) only with `--- programm`: the text `ALT` (rest of the header line) must appear **exactly once** in the program and is replaced by the block; `{sammlung}` and `{fall}` apply inside it. For a file dialog no recording can reach (`FILE_SAVE_DIALOG(…)` → `"raus.dh"`), or for reading off an input field — exactly this call is replaced, everything after it stays the real code. Several blocks allowed; present twice or not at all is an error |
| `--- einschub nach MARKE` | (insertion after) only with `--- programm`: a **further** insertion after the first line `MARKE` (several allowed, inserted after the case's source code). Typically the source code sits at the top (helper functions, `DIM`) and an insertion after `FLIP()` measures each frame — only there is the window geometry settled, so that is where it can write the recording with real click positions and play it back |
| `--- inhalt datei` | (contents) after the run every line of the block appears in this file (log, saved file; path relative to the case folder) |
| `--- ohne datei` | (without) no line of the block appears in it (a file that does not exist contains nothing either) |
| `--- streichen ZEILE` | (delete) only with `--- programm`: the first line that reads exactly like this is left out of the copy (e.g. `WINDOW_MAXIMIZE()`, so that clicks hit fixed positions); if it is missing, the case is an error. Several blocks allowed |
| `--- nachher` | (afterwards) a second program that runs in the case folder after the run — its output is compared with `erwartet`/`enthaelt`/`fehler` instead of that of the main run. This way Drachenhauch itself reads what a tool saved (`JSON_*`, `ANIM_FSM_LOAD`, `PROCESS_START("dhrt", "--check", …)`). The main run must end cleanly first; at most one per case |
| `--- vorher` | (before) a helper program in Drachenhauch **before** the main run, in the case folder — for example to create a git repository (`PROCESS_START("git", …)`) or prepare a file that is only meant to be changed after a commit; it must end cleanly. At most one per case |
| `--- zwischen` | (in between) a helper program **after** the main run and before `--- nachher`, in the case folder — for example to measure a click position from the screenshot of the first run and write the recording for the next one, or to delete a recording again |
| `--- nochmal [in ORDNER]` | (again) the program **once more**, with the case's environment; the lines of the block are its arguments. Without `in` in the case folder, otherwise in this subfolder (it is created) — for anything a tool keeps across a restart. `--- zwischen` and `--- nochmal` run in the order of the file; without `--- nachher` the expectations apply to the last run |

In `--- umgebung`, `--- argumente` and in text fixture files (`--- datei`
without `base64`) two placeholders are available: `{sammlung}` is the folder
of the `.dhtest` file, `{fall}` the case folder — both with forward slashes so
they stay valid inside a JSON fixture file too. For a tool that needs an
absolute path (`DH_IDE_WURZEL`, the sprite sheet of a `.dhanim`), or for a
case that reads a file of the project.

**Windows without real input:** every program `dhrt test` starts (case,
helper program, test program, and whatever these start in turn) gets
`DHRT_OHNE_EINGABE=1`. A window then opens without focus, lets the mouse pass
through to the window below and gives the foreground back if it gets it after
all (maximizing, `WINDOW_FOCUS`). Recordings (`AUTOMATION_PLAY`) and posted
window messages (`_hilfen/fenstersender.ps1`) still arrive -- so you can keep
working during a run without typing into a case. Before this, "BSCREEN"
appeared instead of "SCREEN" in a case's IDE, and a mouse over the window
took a case's tooltip away. `DHRT_OHNE_EINGABE=0` in `--- umgebung` switches
it off for one case.

**Testing tools without Python** (since stage 32): this is how the form
designer's tests run in `tests/pruef/werkzeug_formdesigner.dhtest`. Two ways
side by side — real clicks through a recording (insertion
`AUTOMATION_PLAY("ev.txt")` after `SETFPS(60)`, fixture file
`--- datei _programm/ev.txt`), or directly through the tool's subroutines
(insertion after the line at which it is ready; the insertion calls, prints
and ends with `EXIT(0)`). The second way checks the logic without screen
positions and is therefore the more robust one where the mouse is not the
point.

```dhtest
=== ablegen und sichern
AUTOMATION_PLAY("ev.txt")
--- programm ../../examples/197_form_designer.dh nach SETFPS(60)
--- argumente
neu.dhform
--- umgebung
DHRT_FRAMES=120
DH_FORM_LOG=fd.log
--- datei _programm/ev.txt
...
--- inhalt _programm/fd.log
neu button 248 184
--- inhalt neu.dhform
"kind": "button"
```

Since stage 33 the animation FSM editor and the score editor run this way too
(`tests/pruef/werkzeug_animfsm.dhtest`, `tests/pruef/werkzeug_notenblatt.dhtest`):
the copy without `WINDOW_MAXIMIZE()` (`--- streichen`), and what was saved is
read by a `--- nachher` program — for the animation editor the runtime itself
(load, set up, one step), for the tracker export a fixed grid.
Since stage 34 the IDE as well (`tests/pruef/werkzeug_ide.dhtest`, states 1
to 5): the `nachher` program reads its log with `READLINES` and checks exact
lines, orders and counts — `--- inhalt` only searches for substrings, and
`geprueft 1` would also be found in `geprueft 10`. States 6 to 12 follow in
`tests/pruef/werkzeug_ide_6_12.dhtest`, 13 to 25 in
`tests/pruef/werkzeug_ide_13_25.dhtest` (a text with quotation marks or a line
break gets into the reading program through `CHR$(34)`/`CHR$(10)` there —
neither is allowed inside a string); an IDE preset comes in there as the
fixture file `ide.json` in `_programm/`. **Trap:** a tool that derives further
paths from a relative path and hands them on to a child (`dhrt bild` changes
into the directory of its source) needs the path absolute — use `{fall}` in
`--- umgebung` for that.

The lines of a `--- bild` block:

| Probe | Meaning |
|---|---|
| `groesse 320 240` | (size) width and height |
| `100 100 #00FF00` | the pixel has exactly this colour (`#RGB` works too) |
| `100 100 #00FF00 +-40` | each channel may differ by 40 — for anti-aliasing and drivers that round differently |
| `30 30 nicht #00FF00 +-40` | the pixel does **not** have this colour (`nicht` = not) |
| `99 60 <> 100 60` / `99 60 = 100 60` | two pixels against each other, when the absolute colour does not matter (an edge is there or it is not) |

A picture check needs the graphics build; without raylib the case counts as
skipped, not as failed.

`--- ton datei.wav` correspondingly checks a WAV file the program wrote with
`AUDIO_SAVE_WAV` — the runner reads it itself, without raylib:

| Probe | Meaning |
|---|---|
| `kanaele 1` / `bits 16` / `abtastrate 44100` | (channels / bits / sample rate) header data of the file |
| `dauer 0.5 +-0.005` | (duration) length in seconds |
| `spitze 0.7 +-0.01` | (peak) largest magnitude across all channels (amplitudes lie in -1..1) |
| `pegel 0 300 < 0.01` | (level) every 10 ms window between 0 and 300 ms stays below 0.01 (silence) |
| `pegel 300 500 > 0.4` | every window above it (the tone is sounding) |
| `pegel 300 310 0.5 +-0.06` | every window inside the band (a sustain level) |
| `kanaele verschieden` / `kanaele gleich` | (different / equal) left channel against right channel |

The level is the peak value per 10 ms window of the first channel — the
envelope, coarsely sampled. Example:

```dhtest
=== SCISSOR schneidet ab
SCREEN(320, 240)
WHILE NOT QUITREQUESTED()
    CLS(RGB(0, 0, 0))
    SCISSOR(50, 50, 100, 100)
    BOX(20, 20, 220, 220, RGB(0, 255, 0))
    SCISSOR_END()
    FLIP()
WEND
--- bild
100 100 #00FF00 +-80
30 30 nicht #00FF00 +-80
```

Without an expectation a case counts as passed if it ends with 0. **Every
case runs as its own process in its own directory**, the cases of one file in
parallel — unless the file carries the line `--- seriell` before the first
case, then they run one after another (for cases that share the clipboard, a
fixed port or the sound card); `dhrt test datei.dhtest --filter Text` runs
only the cases whose name contains the text, `--fall Name` exactly the one
with this name. A file with the line `--- langsam` before the first case is
left out by `dhrt test --schnell` (the tools, the IDE and everything that
drives real windows in real time) — the summary names every file left out
with `langsam <name>`, so that nobody mistakes a quick run for a full one.
Both switches are lines of their own without `'` in front -- written as a
comment (`' --- seriell`) they would not apply, which is why that is an
error. The CI always runs everything. A case that fails on a machine without
a display or sound card counts as skipped, not as failed (recognized by the
message "Kein Fenster moeglich" (no window possible), which `dhrt` prints
instead of crashing; `DHRT_KEIN_FENSTER=1` fakes exactly this error, for
checking on a computer with a display); with `DHRT_OHNE_GRAFIK=1` also one
that is missing a graphics command in the build without raylib. **This also
holds when the case itself ends with 0** and only its CHILD got no window — a
case that starts a program catches its failure, after all, and prints it as
text. The decision is therefore made on the result: only what would otherwise
count as failed is skipped; a case that EXPECTS the message stays green
(otherwise `kein_fenster.dhtest` would silently never have been checked).
**A case may also skip itself**: the line `UEBERSPRINGEN: <grund>` on stdout
or stderr, and the reason then appears in the summary. This is meant for
external tools that are not present everywhere (node, git, cargo); the
alternative would be to **invent** the expected lines — green and worthless.
The summary names files and cases:

```text
  ok      tests\pruef\coroutines.dhtest  (13 Faelle, 0.61s)
  FEHLER  tests\pruef\array_literal.dhtest  (1 von 23 Faellen, 0.90s)
          FEHL  Zeile 61: leeres Feld: Ausgabezeile 1: erwartet '0', erhalten '1'

2 Datei(en), 1 ok, 1 mit Fehlern; 36 Faelle, 35 ok, 1 fehl, 0 uebersprungen  (1.52s)
```

The project's collections live under `tests/pruef/`; the CI calls
`dhrt test tests/pruef` directly (until 2026-09-20 through a pytest anchor).
The format itself is checked by `tests/pruef/dhrt_test_format.dhtest` — on a
collection the case writes, with expected messages, line numbers and summary.

## `dhrt fmt` — writing consistently

```bash
dhrt fmt datei.dh ...            # Schlüsselwörter groß, Leerraum am Ende weg
dhrt fmt --einruecken datei.dh   # zusätzlich neu einrücken
dhrt fmt --pruefen datei.dh      # schreibt nicht, meldet nur (Rückgabewert 1)
```

**The default is lossless.** It writes keywords in upper case and removes
whitespace at the end of lines — nothing more. No line breaking, no
indentation, nothing inside a line. Drachenhauch ignores upper and lower case,
so everyone writes differently (`If x Then`, `if x then`, `IF x THEN`); that
is exactly what this run makes consistent, and it does so at the lexer's
**token positions**. An `end` in a string or a comment therefore stays
untouched.

What is written in upper case is the word that stands there (`elif` →
`ELIF`), not its canonical name (`ELSEIF`): the spelling is made consistent,
not the vocabulary. Names, built-ins and classes stay as they are.

**`--einruecken` (indent) is deliberately not the default.** The formatter
only knows the blocks of the *language*. A group set by hand, which the
language does not know, gets flattened:

```basic
RENDERTARGET_BEGIN(badge)
    CLS(&H1A2438)             ' indented so you can see the group --
    CIRCLE(60, 52, 28, ...)   ' to the language these are ordinary calls
RENDERTARGET_END()
```

Both really occurred in `examples/` (there also a comment aligned under its
predecessor). A tool that overwrites its user's intended structure must not do
so in passing — so only when you ask for it explicitly.

What `--einruecken` does: one unit of four spaces per level, `CASE` inside
`SELECT` (the house style), `ELSE`/`ELSEIF`/`CATCH`/`FINALLY` one level left
of their body, single-line `IF … THEN …` without indentation, `ENUM` only in
the block form. **Continuation lines (`_`) are left alone** — whoever lines up
their parameters one below the other had a reason for it.

**A file with a syntax error is not touched.** Shifting broken code around
helps nobody; the call reports it and leaves the file alone.

## `dhrt doku referenz` — reference from the source code

Whoever writes their own library (`zeitraum.dh`, `tabellen.dh`) should not
have to describe it a second time by hand — a hand-maintained reference
drifts from day one.

```bash
dhrt doku referenz mathe.dh                     # nach stdout
dhrt doku referenz lib/*.dh -o docs/referenz.md # in eine Datei
```

What comes out is Markdown: `# Referenz`, beneath it one section per file
(sorted by path) `## datei.dh` with constants, enumerations, structures,
classes, functions and procedures in this order, for each entry the name, the
signature as a code block and the description (if it is missing:
*(nicht beschrieben)*, "not described"). With `-o` the file is written and
`geschrieben: <ziel>` is reported. A file that does not exist is an error
(return value 2), before anything is written. The `*` in the example is
expanded by the shell (Bash); `dhrt` itself does not search for patterns —
under cmd or PowerShell name the files one by one.

What is taken is the **signature** and the **comment block directly above
it** — the same source the hover in the IDE and in `dhrt lsp` comes from
(`rust/drachenhauch_runtime/src/symbole.rs`); there is no second idea of what
a signature is. A comment block at the very start of the file describes the
file itself.

```basic
' Distance between two points in the plane.
' Always returns a positive value.
FUNCTION Distanz(x1 AS FLOAT, y1 AS FLOAT, x2 AS FLOAT, y2 AS FLOAT) AS FLOAT
    RETURN SQR((x2 - x1) ^ 2 + (y2 - y1) ^ 2)
END FUNCTION
```

becomes a section with the signature and both sentences.

**`PRIVATE` stays out.** It belongs to the module; a reference that lists it
promises something that disappears with the next refactoring.

**Why not through the lexer:** the Rust lexer throws comments away (it does
not need them). The reference therefore reads the text through the symbol
scanner (`symbole.rs`), which keeps the lines. Until 2026-09-06 the tool lived
in Python as `dhrun.py --doku` — for exactly this reason.

## `dhrt --check` — compiling without running

```bash
dhrt --check datei.dh [weitere.dh ...]
```

Prints the problems found as JSON (empty = clean) — this is what the editor
shows live while you type. The return value is 0 even when something is
found; that is how the editor tells "problems found" apart from "tool
broken".

What gets found is described in
[Language reference → What the compiler checks](sprache.md#what-the-compiler-checks).
