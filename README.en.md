<p align="center">
  <img src="drachenhauch/assets/schriftzug.png" alt="Drachenhauch" width="560">
</p>

<p align="center"><strong>Let your ideas breathe fire.</strong></p>

<p align="center"><em><a href="README.md">Deutsch</a> · English</em></p>

A BASIC dialect with Pascal-strict typing and OOP, built for games. Programs run through **`dhrt`** — the native Rust/raylib runtime, which lexes, parses, compiles and executes the source itself (graphics/audio/3D included). Python is now only the editor/tooling layer.

```basic
IMPORT "sprite"
IMPORT "particles"

SCREEN(320, 240, "Hello World", 2)

DIM hero AS SPRITE
hero = SPRITE_NEW(LOADIMAGE("assets/hero.png"), 16, 16)
SPRITE_SET_POS(hero, 100.0, 100.0)

WHILE NOT QUITREQUESTED()
    CLS()
    SPRITE_DRAW(hero)
    FLIP()
    SLEEP(16)
WEND
```

## Download

**[Download Drachenhauch for Windows](https://github.com/HasoSchno70/Drachenhauch/releases/latest)** — a single installer, about 84 MB, currently version 2026.2.

You do **not** need Python installed. It ships the complete development environment, the `dhrt` runtime, all 165 examples with their assets, the textbook as `.docx` and `.epub`, and the ESP32 skeleton. Windows 64-bit; the file is unsigned, so SmartScreen will speak up on first launch.

## Working from source

```
.venv\Scripts\python.exe dhrun.py            # open the editor
.venv\Scripts\python.exe dhrun.py file.dh    # run a program directly
```

## The textbook

**[Drachenhauch — The Textbook](buch-referenz/buch/)** (German) is two things at once: a course that takes you from the first black window to classes, modules and finished games, and a reference in which **every single command** is explained with a runnable example program. 414 pages, seven parts, 75 chapters, every code sample verified against `dhrt --check`.

```
node build_book.js                    # -> Drachenhauch-Lehrbuch.docx
node build_epub.js                    # -> Drachenhauch-Lehrbuch.epub (for e-readers)
<venv>\python.exe make_book.py        # two-pass build with page numbers in the TOC
```

Both outputs are fed by the same chapter sources (`content/NN_*.js`) — the `.docx` is typeset for A4 and printing, the `.epub` reflows to the reader's font size and supports dark mode.

## Manual

Full documentation lives in the [docs/](docs/README.md) folder (mostly German for now — contributions translating it are welcome):

- **[Language reference](docs/sprache.md)** — variables, types, `ENUM`, `SELECT CASE`, functions with defaults and named arguments, classes, try/catch, f-strings, coroutines (`YIELD` + `CORO_*`)
- **[Standard built-ins](docs/builtins-core.md)** — math, strings, maps, file I/O, …
- **[Graphics built-ins](docs/builtins-grafik.md)** — native runtime (dhrt/raylib), Z-layers, sprite atlas, asset preloader
- **[Performance](docs/PERFORMANCE.md)** — benchmark numbers + optimizations shipped (spec ops, inline caches, typed arrays, ECS bulk ops, …)
- **Modules** — 38 of them, [table below](#modules)
- **[Code editor](docs/editor.md)** — shortcuts, snippets, minimap, multi-cursor, sidebar, run/bench, signature help, **breadcrumbs** (scope path), **peek definition** (Alt+F12), **split view** (Ctrl+\\), **debugger** (breakpoints incl. **conditional** breakpoints/step/variables), **profiler** (hot path per line/function), **git-blame** panel, welcome showcase (demo gallery with screenshots)
- **[Sprite editor](docs/sprite-editor.md)** — pixel-art editor (`dhsprites`): multi-frame, **layers** (visibility/opacity/merge-down, `.dhsprite` v5), animation, atlas export, **export scaling** (1x–8x, nearest-neighbor), **lasso selection** (real pixel mask) + rectangle selection, onion skin (adjustable opacity/range), tile preview
- **[Particle editor](docs/particle-editor.md)** — effect editor (`dhparticles`): tune emitter parameters live with a real-time preview, **preset library** (factory + custom presets), GB code export
- **[Audio Studio](docs/tracker.md#audio-studio)** — combines tracker + SFX generator in **one fullscreen window** with tabs (`dhsound` / `dhrun.py --audio`; `dhsfx`/`dhtracker` open the same window on the matching tab). `F11` fullscreen, `Ctrl+1/2` switch tabs.
- **[SFX generator](docs/sfx-generator.md)** — retro sound effects (sfxr-style, SFX tab): synth with pitch slide/envelope/vibrato/stereo, **SID character** (pulse width/PWM + resonant filter sweep), **preset library** (save your own sounds), export WAV/GB code (`AUDIO_SFX`)
- **[Tracker](docs/tracker.md)** — multi-track music editor (tracker tab), [table below](#tracker)
- **[Score editor](docs/score-editor.md)** — real music-notation display (`dhscore`): place notes by clicking a 5-line staff (treble/bass clef, ledger lines, accidentals), note durations (whole/half/quarter/eighth/sixteenth + dotted + rest), one instrument per track, playback through the shared additive mixer, its own `.json` format **or** direct export/open in the tracker (`dhtracker`)
- **[Tilemap/level editor](docs/tilemap-editor.md)** — paint tiles onto a grid (`dhtilemap`): multiple layers, object layer, **multiple tilesets**, per-tile properties (`solid`/`damage`), pencil/fill/rectangle/eyedropper/**selection** (copy/cut/paste), save/load as Tiled JSON (`TILED_LOAD`), GB code renderer export
- **[Form designer (WYSIWYG)](docs/form-designer.md)** — visual GUI designer in Xojo style (`dhform`): place/configure controls, save as `.dhform`, use in your own code via `GUI_LOAD` or launch with F5
- **[Animation FSM editor](docs/anim-editor.md)** — node graph for animation state machines, Unity-Mecanim style (`dhanim`): visually wire up states (bound to a sprite animation) + parameters + transitions with conditions, save as `.dhanim`, use via `ANIM_FSM_LOAD` ([module `animfsm`](docs/module-animfsm.md)), live preview with F5
- **[Language server + VSCode extension](docs/lsp.md)** — Drachenhauch in any LSP editor: syntax highlighting, diagnostics, completion, hover, goto-definition, references, outline (`py -m drachenhauch.lsp`, `vscode-drachenhauch/`)
- **[Web playground](docs/web-playground.md)** — `dhrt` as WebAssembly in the browser, [table below](#web-playground)
- **[`cloud` module](docs/module-cloud.md)** — cloud save + leaderboard against the bundled, self-hostable reference server [`cloudserver/`](cloudserver/README.md) (Flask + SQLite, shared API-key secret): `CLOUD_CONFIGURE`/`CLOUD_SAVE`/`CLOUD_LOAD`, `LEADERBOARD_SUBMIT`/`LEADERBOARD_FETCH`. Plus **`NUMFMT$`** (core built-in) for idle-/incremental-game-style big-number formatting (`1234567` → `"1.23M"`, K/M/B/T/Qa/Qi/Sx/Sp/Oc/No/Dc, falling back to scientific notation beyond that). Demo [examples/146_cloud_idle.dh](examples/146_cloud_idle.dh)
- **[Connecting an ESP32 / ESP8266](esp32/README.md)** — a ready-made sketch skeleton (Wi-Fi, broker connection, reconnect, receiving) with four marked spots for your own code; **one file for both boards**, compiled for ESP32/ESP8266/ESP32-C3/ESP32-S3. Talks [`mqtt`](docs/module-mqtt.md) to its Drachenhauch counterpart [examples/159_esp32_bruecke.dh](examples/159_esp32_bruecke.dh) — which you can finish **without any board** using `mosquitto_pub`

### Modules

38 modules, available via `IMPORT "name"`. Each has its own page under
[docs/](docs/README.md#module) (mostly German for now).

**Game building blocks**

| Module | What for |
|---|---|
| [`sprite`](docs/module-sprite.md) | animated sheet sprites: position, velocity, named animations, collision |
| [`animfsm`](docs/module-animfsm.md) | animation state machine, Unity-Mecanim style, loaded from `.dhanim` (editor `dhanim`) |
| [`camera`](docs/module-camera.md) | world translation, zoom and rotation for **every** drawing command; follow, screen↔world |
| [`controller`](docs/module-controller.md) | character controller with coyote time, jump buffer and variable jump height |
| [`scene`](docs/module-scene.md) | scene stack (`PUSH`/`POP`/`SWITCH`) with per-scene data |
| [`save`](docs/module-save.md) | save slots backed by JSON, with a version field |
| [`input`](docs/module-input.md) | named actions instead of key codes, edge detection, gamepad |
| [`timer`](docs/module-timer.md) | scheduled actions (`TIMER_AFTER`/`EVERY`) + a `COOLDOWN` rate limiter |
| [`tween`](docs/module-tween.md) | interpolate values smoothly, 13 easings |
| [`curves`](docs/module-curves.md) | Bézier, Catmull-Rom, Hermite, smoothstep — pure functions |
| [`astar`](docs/module-astar.md) | A* pathfinding on a tile grid |
| [`ecs`](docs/module-ecs.md) | entity-component-system with bulk operations for hot loops |

**Physics and maths**

| Module | What for |
|---|---|
| [`physics`](docs/module-physics.md) | pure collision maths: box/circle/ray/segment/polygon, no state |
| [`physics2d`](docs/module-physics2d.md) | **real** 2D rigid bodies (Rapier2D): gravity, stacking, throwing, rolling — [demo](examples/112_physics2d.dh) |
| [`physics3d`](docs/module-physics3d.md) | the same in 3D (Rapier3D) — [demo](examples/107_physics3d.dh) |
| [`vec2`](docs/module-vec2.md) | 2D vector with operator overloading, immutable |
| [`m3d`](docs/module-m3d.md) | VEC3/VEC4/QUAT/MAT4, quaternions, matrices; GPU instancing via `MODEL_INSTANCED` |

**Graphics and sound**

| Module | What for |
|---|---|
| `g3d` | 3D: camera, models (OBJ/GLTF), skeletal animation, PBR, HDR IBL, shadows, normal maps, picking — see [graphics built-ins](docs/builtins-grafik.md) |
| [`particles`](docs/module-particles.md) | particle emitters with gravity, colour gradient over lifetime, five render modes |
| [`imgfx`](docs/module-imgfx.md) | scale, rotate, flip, tint images — including a crisp mode for pixel art |
| [`audio`](docs/module-audio.md) | channels, buses, real-time effects (filter/reverb/delay/distortion/compressor/EQ), synthesis, sampler, `.mod`/`.xm` playback, spatial audio, sample-accurate clock. [Modulators](docs/module-audio-modulatoren.md) keep running on the audio thread even when the frame rate drops |

**User interface**

| Module | What for |
|---|---|
| [`gui`](docs/module-gui.md) | 22 retained-mode widget kinds — including a **professional table** (sort, filter, frozen and reorderable columns, edit cells in place). Glass themes, toggles, knobs, 9-slice skins. [All widgets](examples/156_gui_alle_widgets.dh) · [table](examples/157_gui_tabelle.dh) · [against SQLite](examples/158_gui_tabelle_sqlite.dh) |
| [`ui`](docs/module-ui.md) | the same in immediate mode: nothing to set up, redrawn every frame |
| [`chart`](docs/module-chart.md) | six chart kinds (pie, bar, line, gauge, bar gauge, LED chain), four themes, mouse interaction — [demo](examples/154_chart.dh) |

**Data**

| Module | What for |
|---|---|
| [`json`](docs/module-json.md) | read/write JSON, path access (`"user.name"`, `"items.0"`) |
| [`db`](docs/module-db.md) | SQLite with `?` placeholders and transactions |
| [`regex`](docs/module-regex.md) | match, replace, split |
| [`tiled`](docs/module-tiled.md) | load maps from the Tiled editor, including objects and properties |
| [`tile_collide`](docs/module-tile-collide.md) | box against tilemap, axis by axis — classic platformer physics |
| [`cloud`](docs/module-cloud.md) | cloud save and leaderboard against the bundled server [`cloudserver/`](cloudserver/README.md) |

**Network, hardware, making**

| Module | What for |
|---|---|
| [`net`](docs/module-net.md) | TCP and UDP, non-blocking by default — won't freeze your game loop |
| [`html`](docs/module-html.md) | HTTP GET/POST/download + HTML scraping |
| [`mqtt`](docs/module-mqtt.md) | the IoT world's pub/sub protocol — the way to reach an ESP32 **over Wi-Fi** |
| [`firmata`](docs/module-firmata.md) | drive Arduino/ESP32 pins directly, no sketch of your own needed |
| [`serial`](docs/module-serial.md) | raw COM connection for your own protocols |
| [`usb`](docs/module-usb.md) | USB HID: maker boards, programmers, custom controllers |
| [`bt`](docs/module-bt.md) | Bluetooth Low Energy: scan, connect, read/write characteristics |
| [`wifi`](docs/module-wifi.md) | scan networks, connect, signal strength |

A ready-made sketch skeleton for the board lives in **[esp32/](esp32/README.md)**.

### Tracker

The multi-track music editor inside the [Audio Studio](docs/tracker.md) — a
tracker in the tradition of ProTracker, FastTracker and Renoise.

**Tracks**

| | |
|---|---|
| Channel count | 4–32, configurable; the last one is always the drum channel |
| Accent colour per channel | header, notes, VU meter and fader all pick it up |
| Mixer fader | a real volume slider per track — applies to preview, WAV and the generated GB code |

**Instruments**

| | |
|---|---|
| Library | grand piano, organ, strings, bass, bell … and drums; one sound per track by default |
| Instrument **per note** | a channel is just a voice slot: every single note may bring its own instrument (`Instr:` dropdown) |
| Sample instruments | load WAV/OGG and resample across the whole keyboard (the MOD/XM/IT principle), with a graphical loop editor and a pan slider |
| Keymap / multisample | spread different samples across key zones — also as a drum kit |
| SoundFont | import `.sf2`: real GM and vendor instruments |

**Editing**

| | |
|---|---|
| Patterns | several, each with its own length, arranged into a song |
| Block selection | copy, cut, paste, transpose, interpolate |
| Effect columns | volume, pitch slide/portamento, arpeggio, vibrato, retrigger, sample offset — per note, plus instrument pan |
| Note-off | cut a note deliberately before the next one starts |

**Output**

| | |
|---|---|
| Project | save and load as `.json` |
| GB player | export as frame-driven Drachenhauch code |
| WAV | song mixed offline, **stereo with Amiga hard-panning** → straight into `PLAYMUSIC` |

### Web playground

`dhrt` runs as WebAssembly in the browser — not a cut-down version, the same
runtime. Type source into the text area, hit run. Details in
[docs/web-playground.md](docs/web-playground.md) (German).

| What runs in the browser | How |
|---|---|
| Compiling | dhrt compiles the source **in the browser itself** — no Pyodide, no server |
| Console and graphics | both at once; the render loop yields once per frame (ASYNCIFY) so `WHILE … FLIP() … WEND` doesn't freeze the tab |
| Sound | a custom Kira backend feeds the finished mix into OpenAL buffers, which emscripten maps onto WebAudio; the queue paces itself in real time. Browsers only allow sound after the first click |
| 3D | WebGL 2 — its GLSL ES 3.00 is identical to the desktop GLSL except for the header. PBR, HDR IBL, skybox, shadows, instancing and post-processing are all verified in the browser |
| Files | an `assets/` folder next to the `.dh` ships along as `dhrt.data` — images, fonts and music load from the same paths as on the desktop |
| Shareable links | the source lives in the URL hash: opening a link means seeing it **and** running it |

Built with `rust/build_wasm.py`; the harness lives in `web/`.

## Examples

`examples/` contains 170+ runnable demos, from "Hello World" to a complete mini game:

| File | Shows |
|---|---|
| `01_hello.dh` … `09_shapes.dh` | language basics |
| `10_pong.dh`, `22_tetris.dh`, `23_platformer.dh` | complete games |
| `24_json.dh` … `33_ui.dh` | every module with a demo |
| `32_coinquest.dh` | mini game combining modules + SELECT CASE |
| `49_pong_scene.dh` | Pong, structured with `scene` + `save` (highscore) |
| `50_enum.dh` | ENUM in compact and block form |
| `51_astar.dh` | A* pathfinding with an ASCII render |
| `52_named_args.dh` | named arguments in SUB/FUNCTION/NEW |
| `73_ecs_bullets.dh` | ECS with a per-entity loop (classic pattern) |
| `75_preloader.dh` | `LOAD_ASSETS` — every image/sound from one manifest |
| `76_layers_atlas.dh` | Z-layers + sprite atlas: 600 tiles from one atlas, composited in z-order |
| `77_tiled_platformer.dh` | **mini platformer**: Tiled level + atlas + tile collision + Z-layers + input mapping |
| `98_coroutines.dh` | **coroutines/`YIELD`**: generators, `FOR EACH` drain, send/return dialog, `CORO_RESULT`, method coroutine |
| `154_chart.dh` | **charts**: all six kinds, themes, mouse interaction |
| `156_gui_alle_widgets.dh` | **all 22 GUI widgets** in one fullscreen application, each with a real job |
| `157_gui_tabelle.dh`, `158_gui_tabelle_sqlite.dh` | **professional table** — sort, filter, edit cells; the second one against a real SQLite database |
| `159_esp32_bruecke.dh` | **connect an ESP32** — receive readings, send commands back (sketch in [esp32/](esp32/)) |
| `bench_ecs_movement_v2.dh` | ECS bulk API (`ECS_INTEGRATE_FLOAT`) — 40× faster than a per-entity loop |
| `bench_ecs_systems.dh` | bullet-hell pattern with 8 bulk systems per frame |

## Architecture

Pipeline: **source → preprocessor → lexer → parser → compiler → VM** — **all inside `dhrt`** (Rust). `dhrt run file.dh` is a self-contained end-to-end run with no Python involved. Correctness is guarded by **run_gb golden tests** (`assert run_gb(src) == expected`, spawns `dhrt run`) plus Rust `#[test]`s.

> **History:** Programs used to also run through a Python **tree-walking interpreter** and two Python **bytecode VMs** (a plain Python VM and a Cython VM), guaranteeing "bit-identical output" across all three. As of **Stage B** the tree-walker and the entire Python toolchain (interpreter/compiler/vm/serialize) have been **removed** — `dhrt` is the only runtime and compiles the source itself.

**Native Rust runtime (raylib) — the only runtime.** `dhrt` (`rust/drachenhauch_runtime/`)
lexes, parses, compiles and executes the source itself — a self-contained Rust
front-end, no Python anywhere in the execution path. What runs natively:

| Area | Scope |
|---|---|
| Language | scalars, strings, arrays, maps, tuples, OOP (classes/methods/properties/operators), slicing, comprehensions, `TRY`/`THROW` and every pure built-in |
| Coroutines | `YIELD` via a frame snapshot instead of threads — safe on raylib's main thread, deterministic by construction, and works in a standalone `.exe` ([demo](examples/98_coroutines.dh)) |
| 2D | primitives, text, images, input; z-layers, sprite atlas, render targets, blend modes, procedural textures |
| 3D | camera + primitives ([demo](examples/82_3d_intro.dh)), OBJ/GLTF models, procedural meshes up to heightmap terrain, **skeletal animation** for rigged GLTF/IQM ([demo](examples/108_skeletal_anim.dh)), billboards, ray hits and mouse picking — on real surfaces, not just bounding volumes ([demo](examples/151_picking_flaechen.dh)) |
| Lighting | physically based (Cook-Torrance), up to 4 lights, `MODEL_PBR` for metalness and roughness ([demo](examples/95_pbr.dh)); emissive glow ([demo](examples/110_emissive_glow.dh)), PCF shadows ([demo](examples/93_shadows.dh)), normal maps ([demo](examples/94_normalmap.dh)), depth fog ([demo](examples/92_fog.dh)) and environment light — analytical ([demo](examples/96_ibl.dh)) as well as real HDR cubemap IBL ([demo](examples/99_ibl_hdr.dh)) |
| Sound | **Kira** on its own audio thread (replaced raylib audio on 2026-06-13): sounds, music, `.mod`/`.xm` through a pure-Rust player |
| Interface | `gui` with 22 retained-mode widget kinds (themes, dragging, z-order, focus, FUNCREF callbacks) and `ui` in immediate mode |
| Frame and loop | game loop (`DELTA`/`FPS`/`SETFPS`), GPU shaders and post-processing (`SHADER_LOAD`/`POSTFX`), TTF fonts ([demo](examples/87_ttf_fonts.dh)), gamepad |
| Recording input | `AUTOMATION_RECORD`/`PLAY` for attract mode, replayable bug reports and automated playtests ([docs](docs/automation.md), [demo](examples/153_automation.dh)) |
| Modules | **all of them** — including the formerly Python-only ones: `regex`, `tiled`, `tile_collide`, `controller`, extended `audio`; plus feature-gated `db` (rusqlite), `net`, `mqtt`, `html` (ureq) and the hardware side `serial`, `firmata`, `usb`, `wifi`, `bt` |
| Shipping | `dhrun.py --export` (or Ctrl+F6 in the editor) bundles bytecode + `assets/` into a standalone `.exe` that runs without Python |

That leaves only the editor needing Python. The heavier modules come in with
`build_runtime.py --hardware` or `--full` — **a build without those flags leaves
them out again**, which is the most common reason a hardware example suddenly
stops working. One showcase that exercises nearly all of it at once:
[examples/97_pbr_reactor.dh](examples/97_pbr_reactor.dh) — an audio-reactive ring
of chrome spheres with IBL, shadows, bloom and stereo techno. Plan and status in
[docs/rust-runtime.md](docs/rust-runtime.md).

**Front-end port to Rust — complete.** The entire toolchain (lexer → parser → compiler → preprocessor) has been ported to Rust, each stage verified for output parity against the Python tree-walker. **`dhrt run file.dh` is a self-contained end-to-end run with no Python:** it preprocesses (`IMPORT` resolution for both source files and built-in modules), lexes, parses, compiles and executes — scalars/arithmetic/control flow, arrays/maps, functions, classes/OOP, `SELECT`/`FOR EACH`/tuples/`WITH`/`TRY`/slicing/comprehensions/coroutines. Like `dhrun.py`, it changes into the file's directory so relative `IMPORT` and asset paths resolve correctly (`dhrt file.dh` without `run` works the same way; `.dhc` files still use the direct VM path). Debug entry points: `dhrt --tokens`/`--ast`/`--preprocess`/`--runsrc`. **Self-export without Python:** `dhrt --export file.dh` compiles the source itself and bundles it into a self-contained `.exe` (appends the bytecode to a copy of the runtime, copies `assets/`). Aliased module imports (`IMPORT "json" AS j` → `J_PARSE`, `DIM h AS J_HANDLE`) work natively too. This also makes the **web playground pure Rust WASM**, compiling the source in the browser (no Pyodide): `rust/build_wasm.py file.dh` produces `web/dhrt.{js,wasm}` with the source embedded (the emscripten toolchain on Windows is wired up automatically). **Console and animated graphics both run in the browser** — the GB render loop yields every frame via ASYNCIFY (`emscripten_sleep(0)` inside `flip()`), so `WHILE … FLIP() … WEND` doesn't freeze the tab; **shareable links** pack the source into the URL hash. Plan & stages in [docs/rust-frontend-port.md](docs/rust-frontend-port.md) (German).

Architecture details and extension notes in [CLAUDE.md](CLAUDE.md) (German).

## Tests

```
.venv\Scripts\python.exe -m pytest tests/
```

3090+ tests — built-ins, every module, language constructs, editor features and example smoke tests. Correctness is guarded by **run_gb golden tests** (`assert run_gb(src) == expected`, spawn `dhrt run`) plus Rust `#[test]`s; they skip cleanly if `dhrt` isn't built.

## License

Private.
