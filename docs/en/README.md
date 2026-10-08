# Drachenhauch Manual

Complete reference for the Drachenhauch language, all built-in commands and all built-in modules.

Drachenhauch is a BASIC dialect with Pascal-strict typing and OOP, designed for games. Programs run through **`dhrt`** — the native Rust/raylib runtime, which lexes, parses, compiles and executes the source itself. The IDE and the tools are Drachenhauch programs themselves; Python is left only in the two build scripts.

**Deutsch:** this is the translation of the German manual in [../README.md](../README.md); the IDE shows it when its interface is English. Only the design notes, audits and release notes stay German. Code is not translated, its comments are -- `dhrt pruef uebersetzung` checks that every English file follows its German one (headings, table rows, code, links). Whoever changes a German document updates the translation too.

## Contents

### The language

- **[Switching over](umstieg.md)** — from QBasic, VB, Blitz or Python: what things are called here
- **[Language reference](sprache.md)** — variables, types, operators, control flow (IF, SELECT CASE, WHILE, FOR), functions, classes, try/catch, imports
- **[Foreign libraries](ffi.md)** — `DECLARE … LIB`: call functions from DLLs, `.so` and `.dylib` files (Windows API, C library, device drivers)
- **[Variable scope](scope.md)** — where a variable is visible: globally, in functions, in methods — and why there is no block scoping

### Built-in commands

- **[Standard built-ins](builtins-core.md)** — math, strings, bitwise, maps, file I/O, conversion, time/random
- **[Graphics built-ins](builtins-grafik.md)** — SCREEN, CLS, BOX, CIRCLE, LOADIMAGE, sound, tilemap, input, **`LOAD_ASSETS`** (bulk preloader), **sprite atlas + batch draw**, **Z-layer rendering**
- **[Performance](../PERFORMANCE.md)** (German) — historical benchmark numbers + a list of optimizations shipped (the Python paths they were compared against have been removed since stage B; production = `dhrt`)

### Modules

Each module is activated with `IMPORT "<name>"` and provides its own commands.

> **All modules also run in the native runtime (dhrt)** — most of them are always included there; `db`/`net`/`http` (= `html`) are already part of the standard dev build (`python rust/build_runtime.py`), hardware (`serial`/`usb`/`wifi`/`bt`) is added with `--hardware`. Every module document has a section **"In the native runtime (dhrt)"** at the bottom with its feature flag and peculiarities; overview in [rust-runtime.md](rust-runtime.md).

| Module | What it does | Docs |
|---|---|---|
| `json` | Parse JSON, read values via path notation | [module-json.md](module-json.md) |
| `db` | SQLite database: CREATE/INSERT/SELECT, transactions | [module-db.md](module-db.md) |
| `tween` | Value interpolation over time (animations, easings) | [module-tween.md](module-tween.md) |
| `zeit` | Date and time as a number: `ZEIT_PARSE`/`ZEIT_PLUS`/`ZEIT_DIFF`, display via `ZEIT_FORMAT$` | [module-zeit.md](module-zeit.md) |
| `video` | Play videos: MP4 with H.264 into an `IMAGE` (`VIDEO_LOAD`/`VIDEO_PLAY`/`VIDEO_DRAW`), without sound | [module-video.md](module-video.md) |
| `timer` | Scheduled actions: `TIMER_AFTER`/`TIMER_EVERY` with FUNCREF callbacks (`TIMER_UPDATE` per frame) + `COOLDOWN` rate limiter | [module-timer.md](module-timer.md) |
| `imgfx` | Image effects: scale, rotate, flip, tint, copy | [module-imgfx.md](module-imgfx.md) |
| `particles` | Particle system with velocity, gravity, lifetime | [module-particles.md](module-particles.md) |
| `physics` | AABB/circle collision, distance, reflect, ray cast | [module-physics.md](module-physics.md) |
| `camera` | World translation and zoom for all drawing commands | [module-camera.md](module-camera.md) |
| `sprite` | Animated sprites from sheets, AABB collision, flip/scale/tint | [module-sprite.md](module-sprite.md) |
| `ui` | Immediate-mode UI: label, button, checkbox, slider | [module-ui.md](module-ui.md) |
| `scene` | Stack-based scene/state manager with per-scene data | [module-scene.md](module-scene.md) |
| `save` | Persistent save slots (JSON backend, type-safe, versioned) | [module-save.md](module-save.md) |
| `astar` | A* pathfinding on tile grids, with diagonals + heuristics | [module-astar.md](module-astar.md) |
| `ecs` | Entity component system with sparse-set storage + **bulk system ops** (`ECS_INTEGRATE_FLOAT`, `ECS_SCALE_FLOAT`, …) | [module-ecs.md](module-ecs.md) |
| `vec2` | Immutable 2D vector with operator overloading (`+ - * / = <>`), lerp, reflect, polar | [module-vec2.md](module-vec2.md) |
| `input` | Action-based input mapping, multi-key bindings, edge detection (PRESSED/RELEASED), virtual axis | [module-input.md](module-input.md) |
| `regex` | Python-compatible pattern matching with a pattern cache: `REGEX_MATCH/TEST/FIND/REPLACE/SPLIT` | [module-regex.md](module-regex.md) |
| `audio` | Extended audio API: channels, pause/resume/fade, pan, music position, tone generation (sine/square/saw/triangle/noise) | [module-audio.md](module-audio.md) |
| `curves` | Animation curves: Bezier, Catmull-Rom, Hermite, smoothstep — pure functions, no state | [module-curves.md](module-curves.md) |
| `net` | TCP + UDP via stdlib sockets, non-blocking by default, UTF-8 encoding | [module-net.md](module-net.md) |
| `html` | HTTP GET/POST/DOWNLOAD and HTML parsing — pure stdlib, no pip needed | [module-html.md](module-html.md) |
| `cloud` | Cloud save + leaderboard against the self-hostable reference server (`cloudserver/`) | [module-cloud.md](module-cloud.md) |
| `bt` | Bluetooth Low Energy (BLE) — scan, connect, read/write characteristics (Python: `bleak`, native: `btleplug`) | [module-bt.md](module-bt.md) |
| `serial` | RS-232 / USB COM — open, read/write, available, flush, timeout (Python: `pyserial`, native: `serialport`) | [module-serial.md](module-serial.md) |
| `usb` | USB HID via `hidapi` — custom controllers, maker boards, programmers | [module-usb.md](module-usb.md) |
| `wifi` | Wi-Fi management (Windows only, via `netsh wlan`): scan, connect, disconnect, profiles | [module-wifi.md](module-wifi.md) |
| `tiled` | [Tiled](https://www.mapeditor.org/) map loader (JSON). Tile layers, object layers, tile + object properties | [module-tiled.md](module-tiled.md) |
| `tile_collide` | Box-vs-tilemap collision: TILE_SWEEP_X/Y with the separate-axis pattern. Classic platformer | [module-tile-collide.md](module-tile-collide.md) |
| `controller` | Character controller with coyote time, jump buffer, variable jump height. Classic "feel-good" platformer mechanics | [module-controller.md](module-controller.md) |
| `gui` | Retained-mode user interfaces: windows and widgets as persistent objects, 22 widget kinds including a table | [module-gui.md](module-gui.md) |
| `chart` | Charts: pie/donut, bar, line/area, gauge, bar gauge, LED — with mouse and tooltip | [module-chart.md](module-chart.md) |
| `g3d` | 3D graphics: shapes, loaded and procedural models, skeletal animation, light and shadows, clicking in 3D space | [module-g3d.md](module-g3d.md) |
| `m3d` | 3D math: VEC3/VEC4/QUAT/MAT4 for hierarchical transforms, custom cameras, instancing | [module-m3d.md](module-m3d.md) |
| `animfsm` | Animation state machine in the Mecanim style, data-driven from `.dhanim` | [module-animfsm.md](module-animfsm.md) |
| `physics2d` | Full 2D rigid-body solver (Rapier2D): stacking, throwing, rolling | [module-physics2d.md](module-physics2d.md) |
| `physics3d` | Full 3D rigid-body solver (Rapier3D) | [module-physics3d.md](module-physics3d.md) |
| `audio` (modulators) | LFO and tweener on the audio thread: tremolo, wobble, filter sweeps without per-frame recalculation | [module-audio-modulatoren.md](module-audio-modulatoren.md) |
| `midi` | Read notes from a connected instrument and send notes out (`--hardware`) | [module-midi.md](module-midi.md) |
| `mqtt` | MQTT 3.1.1 client — the pub/sub protocol of the IoT/maker world | [module-mqtt.md](module-mqtt.md) |
| `httpd` | A small web server running in step with the main loop — a control panel on your home network | [module-httpd.md](module-httpd.md) |
| `ini` | Settings files a human can edit — an INI file is a MAP | [module-ini.md](module-ini.md) |
| `xml` | Read XML — data from other systems, with path navigation | [module-xml.md](module-xml.md) |
| `pdf` | Print-ready pages: invoice, delivery note, report, label | [module-pdf.md](module-pdf.md) |
| `xlsx` | Reports as Excel workbooks: sheets, header row, number and date formats | [module-xlsx.md](module-xlsx.md) |
| `smtp` | Send e-mail: text and HTML, attachments, STARTTLS/TLS | [module-smtp.md](module-smtp.md) |
| `geld` | An amount of money as a value of its own: exact, cannot be mixed with numbers, no lost cents | [module-geld.md](module-geld.md) |
| `firmata` | Control Arduino/ESP32 pins directly, without a sketch of your own | [module-firmata.md](module-firmata.md) |

### Designs and investigations

Papers that **weigh** something rather than describe it — measured,
designed, recommended; none of it is decided.

- **[General-purpose audit 2](../allzweck-audit-2.md)** (German) — what a
  language you are supposed to build software with is missing; eight points,
  worked through
- **[Design: money](../entwurf-geldtyp.md)** (German) — floating point, cents,
  a type of its own? With the finding that `INT(19.99 * 100)` yields **1998**
- **[Investigation: database drivers](../entwurf-datenbanktreiber.md)** (German)
  — what PostgreSQL and MySQL would really cost (crates, build time, size, TLS)

### Tools

- **[Packages](pakete.md)** — `dhrt paket hole github:nutzer/repo@stand`: fetch libraries, pin them and share them
- **[The tools around the language](werkzeuge.md)** — `dhrt --version`, `dhrt test` (run test programs), `dhrt fmt` (uniform formatting), `dhrt --check`, `dhrt doku referenz` (reference generated from the source)
- **[IDE](ide.md)** — the development environment, written in Drachenhauch: tabs, project tree, debugger, profiler, project-wide search, refactorings
- **[Sprite editor](sprite-editor.md)** (`examples/189_sprite_editor.dh`) — pixel art with frames, layers, selection mask, GIF, atlas export
- **[Tilemap editor](tilemap-editor.md)** (`examples/187_tilemap_editor.dh`) — build levels, read and write Tiled JSON
- **[Form designer](form-designer.md)** (`examples/197_form_designer.dh`) — click user interfaces together, Xojo style, F5 runs them
- **[Particle editor](particle-editor.md)** (`examples/185_partikel_editor.dh`) — tune effects live
- **[Animation editor](anim-editor.md)** (`examples/198_anim_fsm_editor.dh`) — state machine for sprite animations
- **[Score editor](score-editor.md)** (`examples/199_notenblatt.dh`) — set notes instead of filling tracker rows
- **[Tracker](tracker.md)** (`examples/190_tracker.dh`) and **[SFX generator](sfx-generator.md)** (`examples/183_sfx_generator.dh`) — music and sound effects
- **[Language server + VS Code](lsp.md)** — `dhrt lsp`: the same diagnostics in other editors, without Python
- **[Recording input](automation.md)** — demo mode, replayable bug reports, automated game tests
- **[Web playground](web-playground.md)** — dhrt as WebAssembly, a link is all it takes

### Internals

Working notes, not user documentation — they explain why something is the way it is:

- **[Stumbling blocks](stolpersteine.md)** — friction points of the language, collected while writing the textbook
- **Doc checker** — `dhrt pruef` sends every `basic` block through the front-end chain (`bloecke`), checks command names in tables and prose (`namen`), the counts in the README (`zaehlungen`), key constants (`konstanten`) and paths/links (`pfade`). Hooked into the test suite; until September 2026 it was two Python scripts in `tools/`
- **[Rust front-end port](../rust-frontend-port.md)** (German) and **[runtime migration](rust-runtime.md)** — how `dhrt` came about
- **[Design: namespaces](../entwurf-namensraeume.md)** (German) — WP I of the general-purpose roadmap. All four stages have been built by now; the document remains as a record of the decisions
- **[Design: removing the Python parser](../entwurf-python-parser-entfernen.md)** (German) — measured what still depends on the second parser, and what a cut would cost
- **[Design: sets](../entwurf-set-builtins.md)** (German) — the last open WP-J point, re-scoped after measuring again
- **[Design: TASK_START](../entwurf-task-start.md)** (German) — DH code in the background; three ways, and the third avoids the Send problem entirely
- **[Design: multiple OS windows](../entwurf-native-fenster.md)** (German) — why there is one window per process, four ways; built is way B: a second window as a second `dhrt` with a text channel (`WINDOW_OPEN`)
- **[Design: phasing out Python](../entwurf-python-abbau.md)** (German) — 110,000 lines of Python measured (58 % tests, 32,000 IDE and editors, two thirds of the installer); ways: toolchain/LSP into dhrt, IDE and editors in Drachenhauch, tests as `dhrt test`; recommendation: toolchain first, then the IDE
- **[Design: applications and speed](../entwurf-anwendungen-und-tempo.md)** (German) — measured against Python: string appending is quadratic, built-ins are looked up by name; what a GUI application still lacks, and in which order
- **[Design: machine code](../entwurf-maschinencode.md)** (German) — how Drachenhauch becomes really fast as a compiled language: a typed intermediate stage, a typed VM, then a Cranelift JIT with the VM as the reference
- **[Design: packages](../entwurf-pakete.md)** (German) — fetch and share libraries without a server of our own: sources, storage in the project, lock file with checksum; built as `dhrt paket`
- **[Design: foreign libraries](../entwurf-ffi.md)** (German) — `DECLARE FUNCTION … LIB` as in VB6/FreeBASIC: types, loading, calling via Cranelift, structs via BUFFER, what remains tricky; built including callbacks, struct layout and variable argument counts
- **[Design: SPEAK](../entwurf-speak.md)** (German) — speech output for games without the gui: WinRT delivers the better voices and a sound instead of a speaker output (12–35 ms per sentence, measured); recommendation: synthesis to PCM through Kira, screen readers first
- **[Design: input methods (IME)](../entwurf-eingabemethoden.md)** (German) — measured: storage is Unicode, display is Latin-1 (the euro sign is a question mark); ways: character set, composition window, preedit, SDL; recommendation: first see, then type
- **[Design: accessibility](../entwurf-barrierefreiheit.md)** (German) — the UIA tree of a dhrt window is empty, measured; AccessKit, speech output and craftsmanship as the ways; recommendation: craftsmanship right away, then AccessKit with Windows first
- **[Design: printing](../entwurf-drucken.md)** (German) — no raylib for paper: building blocks checked, four ways; recommendation: draw the page ourselves (GDI/CUPS as targets of the pdf module), plus a command that opens a file
- **[Release 2026.25](../release-2026.25.md)** (German) — what is new in this version
- **[Release 2026.24](../release-2026.24.md)** (German) — the version before
- **[Release 2026.23](../release-2026.23.md)** (German) — and the one before that
- **[Release 2026.22](../release-2026.22.md)** (German) — and the one before that
- **[Release 2026.21](../release-2026.21.md)** (German) — and the one before that
- **[Release 2026.20](../release-2026.20.md)** (German) — and the one before that
- **[Release 2026.19](../release-2026.19.md)** (German) — and the one before that
- **[Release 2026.18](../release-2026.18.md)** (German) — and the one before that
- **[Release 2026.17](../release-2026.17.md)** (German) — and the one before that
- **[Release 2026.16](../release-2026.16.md)** (German) — and the one before that
- **[Release 2026.15](../release-2026.15.md)** (German) — and the one before that
- **[Release 2026.14](../release-2026.14.md)** (German) — and the one before that
- **[Release 2026.13](../release-2026.13.md)** (German) — and the one before that
- **[Release 2026.12](../release-2026.12.md)** (German) — and the one before that
- **[Release 2026.11](../release-2026.11.md)** (German) — and the one before that
- **[Release 2026.10](../release-2026.10.md)** (German) — and the one before that
- **[Release 2026.9](../release-2026.9.md)** (German) — and the one before that
- **[Release 2026.8](../release-2026.8.md)** (German) — and the one before that
- **[General-purpose roadmap](../allzweck-roadmap.md)** (German) — what is missing so that you can write *anything* with it and not just games (current, audit 2026-08)
- **[General-purpose audit, second round](../allzweck-audit-2.md)** (German) — what is still missing after the completed roadmap for people to *choose* Drachenhauch for building software (2026-08-23)
- **[Command set roadmap](../befehlssatz-roadmap.md)** (German) and **[GUI design note](../gui-module-design.md)** (German) — historical, with a reading note
- **[Renaming GameBasic → Drachenhauch](../umbenennung-drachenhauch.md)** (German) — the checklist from 2026-08

## First program

```basic
PRINT "Hallo, Drachenhauch!"

DIM name AS STRING
INPUT "Wie heisst du?", name
PRINT "Schoen dich zu sehen, ", name
```

Save it as `hallo.dh`, then:

```
dhrt run hallo.dh
```

or open it in the IDE and press F5. The installer ships `dhrt` (with the
option "add to PATH" it is also available in the terminal); if you work from
source, you build it once yourself — how is described in the
[README](../../README.md#aus-dem-quelltext-arbeiten).

## First game

A minimal game loop (graphics run in the native runtime dhrt):

```basic
SCREEN(320, 240, "Mein erstes Spiel", 2)

DIM x AS INTEGER
x = 160

WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(1073741904) THEN     ' LEFT
        x = x - 2
    END IF
    IF KEYPRESSED(1073741903) THEN     ' RIGHT
        x = x + 2
    END IF

    CLS(RGB(20, 20, 30))
    BOX(x, 100, x + 20, 120, RGB(255, 200, 80))
    FLIP()
    SLEEP(16)
WEND
```

The arrow keys move the yellow rectangle. ESC or closing the window ends the program (via `QUITREQUESTED()`).

## Conventions in this manual

- Code blocks show runnable Drachenhauch code (often taken directly from `examples/`).
- Built-in signatures are written compactly: `FUNKTION(arg1, arg2[, optional]) -> RÜCKGABETYP`. `[...]` marks optional arguments.
- Type tags: `INTEGER`, `FLOAT`, `STRING`, `BOOLEAN`, `IMAGE`, `SOUND`, `FILE`, `MAP OF T`, `ARRAY OF T`, plus the external types from modules (e.g. `JSON_HANDLE`, `SPRITE`, `TWEEN`).
- The language is **case-insensitive** for keywords and built-ins (`PRINT`, `print`, `Print` are the same), but identifiers (your own variable/function names) remain distinguishable.

Have fun!
