# GameBasic

*[Deutsch](README.md) · English*

A BASIC dialect with Pascal-strict typing and OOP, built for games. Programs run through **`gbrt`** — the native Rust/raylib runtime, which lexes, parses, compiles and executes the source itself (graphics/audio/3D included). Python is now only the editor/tooling layer.

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

## Quick start

```
.venv\Scripts\python.exe gbrun.py            # open the editor
.venv\Scripts\python.exe gbrun.py file.gb    # run a program directly
```

## The textbook

**[GameBasic — The Textbook](buch-referenz/buch/)** (German) is two things at once: a course that takes you from the first black window to classes, modules and finished games, and a reference in which **every single command** is explained with a runnable example program. 414 pages, seven parts, 75 chapters, every code sample verified against `gbrt --check`.

```
node build_book.js                    # -> GameBasic-Lehrbuch.docx
node build_epub.js                    # -> GameBasic-Lehrbuch.epub (for e-readers)
<venv>\python.exe make_book.py        # two-pass build with page numbers in the TOC
```

Both outputs are fed by the same chapter sources (`content/NN_*.js`) — the `.docx` is typeset for A4 and printing, the `.epub` reflows to the reader's font size and supports dark mode.

## Manual

Full documentation lives in the [docs/](docs/README.md) folder (mostly German for now — contributions translating it are welcome):

- **[Language reference](docs/sprache.md)** — variables, types, `ENUM`, `SELECT CASE`, functions with defaults and named arguments, classes, try/catch, f-strings, coroutines (`YIELD` + `CORO_*`)
- **[Standard built-ins](docs/builtins-core.md)** — math, strings, maps, file I/O, …
- **[Graphics built-ins](docs/builtins-grafik.md)** — native runtime (gbrt/raylib), Z-layers, sprite atlas, asset preloader
- **[Performance](docs/PERFORMANCE.md)** — benchmark numbers + optimizations shipped (spec ops, inline caches, typed arrays, ECS bulk ops, …)
- **Modules** — 38 of them, [table below](#modules)
- **[Code editor](docs/editor.md)** — shortcuts, snippets, minimap, multi-cursor, sidebar, run/bench, signature help, **breadcrumbs** (scope path), **peek definition** (Alt+F12), **split view** (Ctrl+\\), **debugger** (breakpoints incl. **conditional** breakpoints/step/variables), **profiler** (hot path per line/function), **git-blame** panel, welcome showcase (demo gallery with screenshots)
- **[Sprite editor](docs/sprite-editor.md)** — pixel-art editor (`gbsprites`): multi-frame, **layers** (visibility/opacity/merge-down, `.gbsprite` v5), animation, atlas export, **export scaling** (1x–8x, nearest-neighbor), **lasso selection** (real pixel mask) + rectangle selection, onion skin (adjustable opacity/range), tile preview
- **[Particle editor](docs/particle-editor.md)** — effect editor (`gbparticles`): tune emitter parameters live with a real-time preview, **preset library** (factory + custom presets), GB code export
- **[Audio Studio](docs/tracker.md#audio-studio)** — combines tracker + SFX generator in **one fullscreen window** with tabs (`gbsound` / `gbrun.py --audio`; `gbsfx`/`gbtracker` open the same window on the matching tab). `F11` fullscreen, `Ctrl+1/2` switch tabs.
- **[SFX generator](docs/sfx-generator.md)** — retro sound effects (sfxr-style, SFX tab): synth with pitch slide/envelope/vibrato/stereo, **SID character** (pulse width/PWM + resonant filter sweep), **preset library** (save your own sounds), export WAV/GB code (`AUDIO_SFX`)
- **[Tracker](docs/tracker.md)** — multi-track music editor (tracker tab), [table below](#tracker)
- **[Score editor](docs/score-editor.md)** — real music-notation display (`gbscore`): place notes by clicking a 5-line staff (treble/bass clef, ledger lines, accidentals), note durations (whole/half/quarter/eighth/sixteenth + dotted + rest), one instrument per track, playback through the shared additive mixer, its own `.json` format **or** direct export/open in the tracker (`gbtracker`)
- **[Tilemap/level editor](docs/tilemap-editor.md)** — paint tiles onto a grid (`gbtilemap`): multiple layers, object layer, **multiple tilesets**, per-tile properties (`solid`/`damage`), pencil/fill/rectangle/eyedropper/**selection** (copy/cut/paste), save/load as Tiled JSON (`TILED_LOAD`), GB code renderer export
- **[Form designer (WYSIWYG)](docs/form-designer.md)** — visual GUI designer in Xojo style (`gbform`): place/configure controls, save as `.gbform`, use in your own code via `GUI_LOAD` or launch with F5
- **[Animation FSM editor](docs/anim-editor.md)** — node graph for animation state machines, Unity-Mecanim style (`gbanim`): visually wire up states (bound to a sprite animation) + parameters + transitions with conditions, save as `.gbanim`, use via `ANIM_FSM_LOAD` ([module `animfsm`](docs/module-animfsm.md)), live preview with F5
- **[Language server + VSCode extension](docs/lsp.md)** — GameBasic in any LSP editor: syntax highlighting, diagnostics, completion, hover, goto-definition, references, outline (`py -m gamebasic.lsp`, `vscode-gamebasic/`)
- **[Web playground](docs/web-playground.md)** — `gbrt` as WebAssembly in the browser: type source into a `<textarea>` → gbrt compiles **in the browser** (no Pyodide) → **console AND animated graphics in a `<canvas>`** (the render loop yields per frame, no tab freeze). **Shareable links** (source in the URL hash → opening it = seeing it run). **Assets** (an `assets/` folder next to the `.gb`) ship along as `gbrt.data` — images, fonts and music load from the same paths as on the desktop. **Audio is audible** — a custom Kira backend feeds the finished mix into OpenAL buffers, which emscripten maps onto WebAudio; the queue paces itself in real time (browsers only allow sound after the first click). **3D works too** — the web build targets WebGL 2, whose GLSL ES 3.00 is identical to our desktop GLSL except for the header: PBR, HDR IBL, skybox, shadows, instancing and post-processing are all verified in the browser. Build via `rust/build_wasm.py`, harness in `web/`
- **[`cloud` module](docs/module-cloud.md)** — cloud save + leaderboard against the bundled, self-hostable reference server [`cloudserver/`](cloudserver/README.md) (Flask + SQLite, shared API-key secret): `CLOUD_CONFIGURE`/`CLOUD_SAVE`/`CLOUD_LOAD`, `LEADERBOARD_SUBMIT`/`LEADERBOARD_FETCH`. Plus **`NUMFMT$`** (core built-in) for idle-/incremental-game-style big-number formatting (`1234567` → `"1.23M"`, K/M/B/T/Qa/Qi/Sx/Sp/Oc/No/Dc, falling back to scientific notation beyond that). Demo [examples/146_cloud_idle.gb](examples/146_cloud_idle.gb)
- **[Connecting an ESP32 / ESP8266](esp32/README.md)** — a ready-made sketch skeleton (Wi-Fi, broker connection, reconnect, receiving) with four marked spots for your own code; **one file for both boards**, compiled for ESP32/ESP8266/ESP32-C3/ESP32-S3. Talks [`mqtt`](docs/module-mqtt.md) to its GameBasic counterpart [examples/159_esp32_bruecke.gb](examples/159_esp32_bruecke.gb) — which you can finish **without any board** using `mosquitto_pub`

### Modules

38 modules, available via `IMPORT "name"`. Each has its own page under
[docs/](docs/README.md#module) (mostly German for now).

**Game building blocks**

| Module | What for |
|---|---|
| [`sprite`](docs/module-sprite.md) | animated sheet sprites: position, velocity, named animations, collision |
| [`animfsm`](docs/module-animfsm.md) | animation state machine, Unity-Mecanim style, loaded from `.gbanim` (editor `gbanim`) |
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
| [`physics2d`](docs/module-physics2d.md) | **real** 2D rigid bodies (Rapier2D): gravity, stacking, throwing, rolling — [demo](examples/112_physics2d.gb) |
| [`physics3d`](docs/module-physics3d.md) | the same in 3D (Rapier3D) — [demo](examples/107_physics3d.gb) |
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
| [`gui`](docs/module-gui.md) | 22 retained-mode widget kinds — including a **professional table** (sort, filter, frozen and reorderable columns, edit cells in place). Glass themes, toggles, knobs, 9-slice skins. [All widgets](examples/156_gui_alle_widgets.gb) · [table](examples/157_gui_tabelle.gb) · [against SQLite](examples/158_gui_tabelle_sqlite.gb) |
| [`ui`](docs/module-ui.md) | the same in immediate mode: nothing to set up, redrawn every frame |
| [`chart`](docs/module-chart.md) | six chart kinds (pie, bar, line, gauge, bar gauge, LED chain), four themes, mouse interaction — [demo](examples/154_chart.gb) |

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
| GB player | export as frame-driven GameBasic code |
| WAV | song mixed offline, **stereo with Amiga hard-panning** → straight into `PLAYMUSIC` |

## Examples

`examples/` contains 170+ runnable demos, from "Hello World" to a complete mini game:

| File | Shows |
|---|---|
| `01_hello.gb` … `09_shapes.gb` | language basics |
| `10_pong.gb`, `22_tetris.gb`, `23_platformer.gb` | complete games |
| `24_json.gb` … `33_ui.gb` | every module with a demo |
| `32_coinquest.gb` | mini game combining modules + SELECT CASE |
| `49_pong_scene.gb` | Pong, structured with `scene` + `save` (highscore) |
| `50_enum.gb` | ENUM in compact and block form |
| `51_astar.gb` | A* pathfinding with an ASCII render |
| `52_named_args.gb` | named arguments in SUB/FUNCTION/NEW |
| `73_ecs_bullets.gb` | ECS with a per-entity loop (classic pattern) |
| `75_preloader.gb` | `LOAD_ASSETS` — every image/sound from one manifest |
| `76_layers_atlas.gb` | Z-layers + sprite atlas + batch draw combined (600 tiles in one `BATCH_FLUSH`) |
| `bench_ecs_movement_v2.gb` | ECS bulk API (`ECS_INTEGRATE_FLOAT`) — 40× faster than a per-entity loop |
| `bench_ecs_systems.gb` | bullet-hell pattern with 8 bulk systems per frame |
| `77_tiled_platformer.gb` | **mini platformer**: Tiled level + atlas + tile collision + Z-layers + input mapping |
| `98_coroutines.gb` | **coroutines/`YIELD`**: generators, `FOR EACH` drain, send/return dialog, `CORO_RESULT`, method coroutine |
| `154_chart.gb` | **charts**: all six kinds, themes, mouse interaction |
| `156_gui_alle_widgets.gb` | **all 22 GUI widgets** in one fullscreen application, each with a real job |
| `157_gui_tabelle.gb`, `158_gui_tabelle_sqlite.gb` | **professional table** — sort, filter, edit cells; the second one against a real SQLite database |
| `159_esp32_bruecke.gb` | **connect an ESP32** — receive readings, send commands back (sketch in [esp32/](esp32/)) |

## Architecture

Pipeline: **source → preprocessor → lexer → parser → compiler → VM** — **all inside `gbrt`** (Rust). `gbrt run file.gb` is a self-contained end-to-end run with no Python involved. Correctness is guarded by **run_gb golden tests** (`assert run_gb(src) == expected`, spawns `gbrt run`) plus Rust `#[test]`s.

> **History:** Programs used to also run through a Python **tree-walking interpreter** and two Python **bytecode VMs** (a plain Python VM and a Cython VM), guaranteeing "bit-identical output" across all three. As of **Stage B** the tree-walker and the entire Python toolchain (interpreter/compiler/vm/serialize) have been **removed** — `gbrt` is the only runtime and compiles the source itself.

**Native Rust runtime (raylib) — the only runtime.** `gbrt` (`rust/gb_runtime/`) lexes, parses, compiles and executes the source itself — a self-contained Rust front-end, no Python anywhere in the execution path. Scalars, strings, arrays, maps, tuples, OOP (classes/methods/properties/operators), slicing, comprehensions, TRY/THROW and the pure built-ins all run natively. **Graphics via raylib** (feature-gated): 2D primitives, text, images, input, verified headless via screenshot (`rust\build_runtime.py`). **Dev loop:** `gbrun.py --native <file.gb>` compiles and launches natively in one command; runtime errors report `file.gb:line`. **3D** (module `g3d`, native-only): camera + cube/sphere/cylinder/cone/plane/lines/grid via raylib's `begin_mode3D` ([examples/82_3d_intro.gb](examples/82_3d_intro.gb)), plus **3D models** — `LOADMODEL` (OBJ/GLTF), procedural `MESH_*` (cube/sphere/cylinder/torus/knot/plane/**heightmap terrain**), `MODEL`/`MODEL_EX`/`MODEL_WIRES`, `MODEL_TEXTURE`, **skeletal animation** for rigged GLTF/IQM (`MODEL_LOAD_ANIMS` + `MODEL_ANIMATE`, [examples/108_skeletal_anim.gb](examples/108_skeletal_anim.gb)), **billboards** (`BILLBOARD`), **ray collision/picking** (`RAY_HIT_BOX`/`SPHERE`, `PICK_BOX`/`SPHERE`), **lighting** (Blinn-Phong, up to 4 lights: `LIGHT_DIRECTIONAL`/`LIGHT_POINT` + `MODEL_LIT`) and **camera modes** (`CAMERA3D_UPDATE` orbital/first-person) ([examples/88_3d_models.gb](examples/88_3d_models.gb), [examples/89_heightmap.gb](examples/89_heightmap.gb), [examples/90_billboards_picking.gb](examples/90_billboards_picking.gb), [examples/91_lighting.gb](examples/91_lighting.gb)). **Standalone export:** `gbrun.py --export <file.gb>` (or the editor's *Export → .exe*, Ctrl+F6) bundles bytecode + `assets/` into a self-contained `.exe` that runs without Python (gbrt with the bytecode appended). **Audio** natively via **Kira** (cpal, its own audio thread — replaced raylib audio on 2026-06-13): `LOADSOUND`/`PLAYSOUND`/`STOPSOUND`/`UNLOADSOUND` (free buffers to avoid sound accumulation in long songs) + `PLAYMUSIC`/`STOPMUSIC` (including `.mod`/`.xm` via a pure-Rust player). **Retained-mode GUI** (`gui`) native: windows + button/label/checkbox/slider/text-input/panel/**table** (scrollable, selectable), theme, drag/z-order/focus, FUNCREF callbacks. Tables (`UI_TABLE` + `GUI_TABLE`) are native. **Game loop** (`DELTA`/`FPS`/`SETFPS`), **GPU shaders/post-processing** (`SHADER_LOAD`/`POSTFX` — CRT/bloom/vignette) and **TTF fonts** (`LOADFONT`/`SETFONT`/`TEXT_SPACING`, [examples/87_ttf_fonts.gb](examples/87_ttf_fonts.gb)) are native too. **Gamepad** support is native (`INPUT_JOY_COUNT/NAME/AXIS` + `JOY_BUTTON_*`/`JOY_DPAD_*` bindings via raylib) as is **depth fog** for lit scenes (`LIGHT_FOG`, [examples/92_fog.gb](examples/92_fog.gb)). **Shadow mapping** (`SHADOW_ENABLE`/`SHADOW_AREA`/`SHADOW_TARGET` — directional shadows with PCF, [examples/93_shadows.gb](examples/93_shadows.gb)) and **normal mapping** (`MODEL_TEXTURE_NORMAL` — per-pixel surface detail via TBN, [examples/94_normalmap.gb](examples/94_normalmap.gb)). **PBR** (Cook-Torrance: the native lighting is physically based; `MODEL_PBR` for metalness/roughness, [examples/95_pbr.gb](examples/95_pbr.gb)) including **emissive/neon glow** (`MODEL_EMISSIVE` — per-model self-illumination, real glow with bloom `POSTFX`, [examples/110_emissive_glow.gb](examples/110_emissive_glow.gb)) including **image-based lighting** — analytical (`LIGHT_ENV`, [examples/96_ibl.gb](examples/96_ibl.gb)) **and real HDR cubemap IBL** (`LIGHT_ENV_HDR` — loads a `.hdr`, computes irradiance/prefilter/BRDF-LUT maps; metals reflect the actual environment, [examples/99_ibl_hdr.gb](examples/99_ibl_hdr.gb)). **Fullscreen showcase:** [examples/97_pbr_reactor.gb](examples/97_pbr_reactor.gb) ("PBR REACTOR") — an FFT-reactive ring of chrome PBR spheres, IBL, shadows, bloom + stereo techno (CC0). **Coroutines/`YIELD`** also run natively — the Rust VM suspends via a frame snapshot (no threads, safe on the raylib main thread, deterministic) including in a standalone `.exe` ([examples/98_coroutines.gb](examples/98_coroutines.gb)). **Full module port complete** — every module that used to be Python-only now runs natively: `regex`, `tiled`, `tile_collide`, `controller`, extended `audio`, plus feature-gated `db` (rusqlite), `net` (std::net), `mqtt` (an MQTT 3.1.1 client for ESP32/IoT, built on `net`), `html` (ureq) and hardware/IoT `serial` (serialport), `firmata` (Arduino/ESP32 pin control over StandardFirmata, built on `serial`), `usb` (hidapi), `wifi` (netsh/nmcli/networksetup), `bt` (btleplug/BLE). That leaves only the editor needing Python; everything else runs fully natively (`build_runtime.py --hardware` / `--full` for the heavier modules). Plan & status in [docs/rust-runtime.md](docs/rust-runtime.md) (German).

**Front-end port to Rust — complete.** The entire toolchain (lexer → parser → compiler → preprocessor) has been ported to Rust, each stage verified for output parity against the Python tree-walker. **`gbrt run file.gb` is a self-contained end-to-end run with no Python:** it preprocesses (`IMPORT` resolution for both source files and built-in modules), lexes, parses, compiles and executes — scalars/arithmetic/control flow, arrays/maps, functions, classes/OOP, `SELECT`/`FOR EACH`/tuples/`WITH`/`TRY`/slicing/comprehensions/coroutines. Like `gbrun.py`, it changes into the file's directory so relative `IMPORT` and asset paths resolve correctly (`gbrt file.gb` without `run` works the same way; `.gbc` files still use the direct VM path). Debug entry points: `gbrt --tokens`/`--ast`/`--preprocess`/`--runsrc`. **Self-export without Python:** `gbrt --export file.gb` compiles the source itself and bundles it into a self-contained `.exe` (appends the bytecode to a copy of the runtime, copies `assets/`). Aliased module imports (`IMPORT "json" AS j` → `J_PARSE`, `DIM h AS J_HANDLE`) work natively too. This also makes the **web playground pure Rust WASM**, compiling the source in the browser (no Pyodide): `rust/build_wasm.py file.gb` produces `web/gbrt.{js,wasm}` with the source embedded (the emscripten toolchain on Windows is wired up automatically). **Console and animated graphics both run in the browser** — the GB render loop yields every frame via ASYNCIFY (`emscripten_sleep(0)` inside `flip()`), so `WHILE … FLIP() … WEND` doesn't freeze the tab; **shareable links** pack the source into the URL hash. Plan & stages in [docs/rust-frontend-port.md](docs/rust-frontend-port.md) (German).

Architecture details and extension notes in [CLAUDE.md](CLAUDE.md) (German).

## Tests

```
.venv\Scripts\python.exe -m pytest tests/
```

3090+ tests — built-ins, every module, language constructs, editor features and example smoke tests. Correctness is guarded by **run_gb golden tests** (`assert run_gb(src) == expected`, spawn `gbrt run`) plus Rust `#[test]`s; they skip cleanly if `gbrt` isn't built.

## License

Private.
