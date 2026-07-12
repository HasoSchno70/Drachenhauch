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

## Manual

Full documentation lives in the [docs/](docs/README.md) folder (mostly German for now — contributions translating it are welcome):

- **[Language reference](docs/sprache.md)** — variables, types, `ENUM`, `SELECT CASE`, functions with defaults and named arguments, classes, try/catch, f-strings, coroutines (`YIELD` + `CORO_*`)
- **[Standard built-ins](docs/builtins-core.md)** — math, strings, maps, file I/O, …
- **[Graphics built-ins](docs/builtins-grafik.md)** — native runtime (gbrt/raylib), Z-layers, sprite atlas, asset preloader
- **[Performance](docs/PERFORMANCE.md)** — benchmark numbers + optimizations shipped (spec ops, inline caches, typed arrays, ECS bulk ops, …)
- **[Modules](docs/README.md#module)** — `json`, `db`, `tween`, `imgfx`, `particles`, `physics`, **[`physics2d`](docs/module-physics2d.md)** (real 2D rigid-body physics via Rapier2D: gravity/collision/restitution/rotation, `PHYS2D_*`, [examples/112_physics2d.gb](examples/112_physics2d.gb)), **`physics3d`** (real 3D rigid-body physics via Rapier3D: gravity/collision/restitution, `PHYS3D_*`, [examples/107_physics3d.gb](examples/107_physics3d.gb)), `camera`, `sprite`, **[`animfsm`](docs/module-animfsm.md)** (animation state machine, Unity-Mecanim style: states + parameters + transitions from `.gbanim`, editor `gbanim`), `ui`, `scene`, `save`, `astar`, `ecs`, `vec2`, [`m3d`](docs/module-m3d.md) (3D math: VEC3/VEC4/QUAT/MAT4 + `MODEL_MATRIX`), `input`, `regex`, `audio` (channels/pan/fades + synth `AUDIO_TONE`/`AUDIO_SFX` — 4-channel chiptune demo [examples/114_chiptune.gb](examples/114_chiptune.gb); **tracker modules `.mod`/`.xm` play directly** — Amiga module player [examples/115_modplayer.gb](examples/115_modplayer.gb); **sampler `SAMPLE_LOAD`/`SAMPLE_PLAY`** — one sample across the whole keyboard, Amiga/Paula style, plus **Paula lo-fi** `AUDIO_LOFI` (8-bit + LED filter) [examples/116_sampler.gb](examples/116_sampler.gb); **mixer buses** `AUDIO_BUS_VOLUME` (SFX/music/master separated) + **real-time effects** per bus `AUDIO_FILTER`/`AUDIO_REVERB`/`AUDIO_DELAY`/`AUDIO_DISTORTION`/`AUDIO_COMPRESSOR`/`AUDIO_EQ` [examples/117_audiofx.gb](examples/117_audiofx.gb) + full showcase "Audio Studio" [examples/118_audio_studio.gb](examples/118_audio_studio.gb); **clock `AUDIO_CLOCK_NEW`/`AUDIO_PLAY_AT`** — sample-accurate music/rhythm timing: start sounds exactly on a clock tick, no polling; **non-linear easings** (`"in"`/`"out"`/`"inout"`) for fades/slides on `AUDIO_PLAY`/`AUDIO_STOP`/`AUDIO_PAN_SLIDE`/`AUDIO_MUSIC_PLAY`/`AUDIO_MUSIC_STOP`; **spatial audio** `AUDIO_LISTENER_NEW`/`AUDIO_EMITTER_NEW`/`AUDIO_PLAY_ON` — panning + distance falloff computed entirely by Kira [examples/139_audio_spatial.gb](examples/139_audio_spatial.gb)), `curves`, `net`, **[`timer`](docs/module-timer.md)** (scheduled actions: `TIMER_AFTER/EVERY` with FUNCREF callbacks + `COOLDOWN` rate limiter, [examples/113_timer.gb](examples/113_timer.gb))
- **[Code editor](docs/editor.md)** — shortcuts, snippets, minimap, multi-cursor, sidebar, run/bench, signature help, **breadcrumbs** (scope path), **peek definition** (Alt+F12), **split view** (Ctrl+\\), **debugger** (breakpoints incl. **conditional** breakpoints/step/variables), **profiler** (hot path per line/function), **git-blame** panel, welcome showcase (demo gallery with screenshots)
- **[Sprite editor](docs/sprite-editor.md)** — pixel-art editor (`gbsprites`): multi-frame, **layers** (visibility/opacity/merge-down, `.gbsprite` v5), animation, atlas export, **export scaling** (1x–8x, nearest-neighbor), **lasso selection** (real pixel mask) + rectangle selection, onion skin (adjustable opacity/range), tile preview
- **[Particle editor](docs/particle-editor.md)** — effect editor (`gbparticles`): tune emitter parameters live with a real-time preview, **preset library** (factory + custom presets), GB code export
- **[Audio Studio](docs/tracker.md#audio-studio)** — combines tracker + SFX generator in **one fullscreen window** with tabs (`gbsound` / `gbrun.py --audio`; `gbsfx`/`gbtracker` open the same window on the matching tab). `F11` fullscreen, `Ctrl+1/2` switch tabs.
- **[SFX generator](docs/sfx-generator.md)** — retro sound effects (sfxr-style, SFX tab): synth with pitch slide/envelope/vibrato/stereo, **SID character** (pulse width/PWM + resonant filter sweep), **preset library** (save your own sounds), export WAV/GB code (`AUDIO_SFX`)
- **[Tracker](docs/tracker.md)** — multi-track music editor (tracker tab): **configurable channel count** (4–32, last channel always drums) with **its own accent color per channel** (header/notes/VU/fader), **per-channel mixer fader** (a real volume slider per track, affects preview/WAV/GB code), **per-note instrument** (like ProTracker/FastTracker/Impulse Tracker/Renoise — a channel is just a voice slot, every note can optionally override its own instrument, `Instr:` dropdown), **ready-made instrument library** (grand piano/organ/strings/bass/bell … + drums, one keyboard sound per track by default), **sample instruments** (load WAV/OGG + resample across the keyboard, MOD/XM/IT style, **graphical loop editor** with draggable waveform markers, **pan as a slider**), **keymap/multisample + drum kit** (different samples across key zones), **SoundFont import** (`.sf2` — real GM/vendor instruments), multiple patterns of adjustable length + song arrangement, **block selection** (copy/cut/paste/transpose/interpolate), **effect columns** (volume, pitch slide/portamento, **arpeggio/vibrato/retrigger/sample offset** per note + instrument pan), **note-off** (cut a note before the next one), project save/load (`.json`), export as a frame-based GB player **or a rendered WAV** (song mixed offline, **stereo + Amiga hard-panning** → `PLAYMUSIC`)
- **[Score editor](docs/score-editor.md)** — real music-notation display (`gbscore`): place notes by clicking a 5-line staff (treble/bass clef, ledger lines, accidentals), note durations (whole/half/quarter/eighth/sixteenth + dotted + rest), one instrument per track, playback through the shared additive mixer, its own `.json` format **or** direct export/open in the tracker (`gbtracker`)
- **[Tilemap/level editor](docs/tilemap-editor.md)** — paint tiles onto a grid (`gbtilemap`): multiple layers, object layer, **multiple tilesets**, per-tile properties (`solid`/`damage`), pencil/fill/rectangle/eyedropper/**selection** (copy/cut/paste), save/load as Tiled JSON (`TILED_LOAD`), GB code renderer export
- **[Form designer (WYSIWYG)](docs/form-designer.md)** — visual GUI designer in Xojo style (`gbform`): place/configure controls, save as `.gbform`, use in your own code via `GUI_LOAD` or launch with F5
- **[Animation FSM editor](docs/anim-editor.md)** — node graph for animation state machines, Unity-Mecanim style (`gbanim`): visually wire up states (bound to a sprite animation) + parameters + transitions with conditions, save as `.gbanim`, use via `ANIM_FSM_LOAD` ([module `animfsm`](docs/module-animfsm.md)), live preview with F5
- **[Language server + VSCode extension](docs/lsp.md)** — GameBasic in any LSP editor: syntax highlighting, diagnostics, completion, hover, goto-definition, references, outline (`py -m gamebasic.lsp`, `vscode-gamebasic/`)
- **[Web playground](docs/web-playground.md)** — `gbrt` as WebAssembly in the browser: type source into a `<textarea>` → gbrt compiles **in the browser** (no Pyodide) → **console AND animated graphics in a `<canvas>`** (the render loop yields per frame, no tab freeze). **Shareable links** (source in the URL hash → opening it = seeing it run). Build via `rust/build_wasm.py`, harness in `web/`
- **[`cloud` module](docs/module-cloud.md)** — cloud save + leaderboard against the bundled, self-hostable reference server [`cloudserver/`](cloudserver/README.md) (Flask + SQLite, shared API-key secret): `CLOUD_CONFIGURE`/`CLOUD_SAVE`/`CLOUD_LOAD`, `LEADERBOARD_SUBMIT`/`LEADERBOARD_FETCH`. Plus **`NUMFMT$`** (core built-in) for idle-/incremental-game-style big-number formatting (`1234567` → `"1.23M"`, K/M/B/T/Qa/Qi/Sx/Sp/Oc/No/Dc, falling back to scientific notation beyond that). Demo [examples/146_cloud_idle.gb](examples/146_cloud_idle.gb)

## Examples

`examples/` contains 50+ runnable demos, from "Hello World" to a complete mini game:

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

## Architecture

Pipeline: **source → preprocessor → lexer → parser → compiler → VM** — **all inside `gbrt`** (Rust). `gbrt run file.gb` is a self-contained end-to-end run with no Python involved. Correctness is guarded by **run_gb golden tests** (`assert run_gb(src) == expected`, spawns `gbrt run`) plus Rust `#[test]`s.

> **History:** Programs used to also run through a Python **tree-walking interpreter** and two Python **bytecode VMs** (a plain Python VM and a Cython VM), guaranteeing "bit-identical output" across all three. As of **Stage B** the tree-walker and the entire Python toolchain (interpreter/compiler/vm/serialize) have been **removed** — `gbrt` is the only runtime and compiles the source itself.

**Native Rust runtime (raylib) — the only runtime.** `gbrt` (`rust/gb_runtime/`) lexes, parses, compiles and executes the source itself — a self-contained Rust front-end, no Python anywhere in the execution path. Scalars, strings, arrays, maps, tuples, OOP (classes/methods/properties/operators), slicing, comprehensions, TRY/THROW and the pure built-ins all run natively. **Graphics via raylib** (feature-gated): 2D primitives, text, images, input, verified headless via screenshot (`rust\build_runtime.py`). **Dev loop:** `gbrun.py --native <file.gb>` compiles and launches natively in one command; runtime errors report `file.gb:line`. **3D** (module `g3d`, native-only): camera + cube/sphere/cylinder/cone/plane/lines/grid via raylib's `begin_mode3D` ([examples/82_3d_intro.gb](examples/82_3d_intro.gb)), plus **3D models** — `LOADMODEL` (OBJ/GLTF), procedural `MESH_*` (cube/sphere/cylinder/torus/knot/plane/**heightmap terrain**), `MODEL`/`MODEL_EX`/`MODEL_WIRES`, `MODEL_TEXTURE`, **skeletal animation** for rigged GLTF/IQM (`MODEL_LOAD_ANIMS` + `MODEL_ANIMATE`, [examples/108_skeletal_anim.gb](examples/108_skeletal_anim.gb)), **billboards** (`BILLBOARD`), **ray collision/picking** (`RAY_HIT_BOX`/`SPHERE`, `PICK_BOX`/`SPHERE`), **lighting** (Blinn-Phong, up to 4 lights: `LIGHT_DIRECTIONAL`/`LIGHT_POINT` + `MODEL_LIT`) and **camera modes** (`CAMERA3D_UPDATE` orbital/first-person) ([examples/88_3d_models.gb](examples/88_3d_models.gb), [examples/89_heightmap.gb](examples/89_heightmap.gb), [examples/90_billboards_picking.gb](examples/90_billboards_picking.gb), [examples/91_lighting.gb](examples/91_lighting.gb)). **Standalone export:** `gbrun.py --export <file.gb>` (or the editor's *Export → .exe*, Ctrl+F6) bundles bytecode + `assets/` into a self-contained `.exe` that runs without Python (gbrt with the bytecode appended). **Audio** natively via **Kira** (cpal, its own audio thread — replaced raylib audio on 2026-06-13): `LOADSOUND`/`PLAYSOUND`/`STOPSOUND`/`UNLOADSOUND` (free buffers to avoid sound accumulation in long songs) + `PLAYMUSIC`/`STOPMUSIC` (including `.mod`/`.xm` via a pure-Rust player). **Retained-mode GUI** (`gui`) native: windows + button/label/checkbox/slider/text-input/panel/**table** (scrollable, selectable), theme, drag/z-order/focus, FUNCREF callbacks. Tables (`UI_TABLE` + `GUI_TABLE`) are native. **Game loop** (`DELTA`/`FPS`/`SETFPS`), **GPU shaders/post-processing** (`SHADER_LOAD`/`POSTFX` — CRT/bloom/vignette) and **TTF fonts** (`LOADFONT`/`SETFONT`/`TEXT_SPACING`, [examples/87_ttf_fonts.gb](examples/87_ttf_fonts.gb)) are native too. **Gamepad** support is native (`INPUT_JOY_COUNT/NAME/AXIS` + `JOY_BUTTON_*`/`JOY_DPAD_*` bindings via raylib) as is **depth fog** for lit scenes (`LIGHT_FOG`, [examples/92_fog.gb](examples/92_fog.gb)). **Shadow mapping** (`SHADOW_ENABLE`/`SHADOW_AREA`/`SHADOW_TARGET` — directional shadows with PCF, [examples/93_shadows.gb](examples/93_shadows.gb)) and **normal mapping** (`MODEL_TEXTURE_NORMAL` — per-pixel surface detail via TBN, [examples/94_normalmap.gb](examples/94_normalmap.gb)). **PBR** (Cook-Torrance: the native lighting is physically based; `MODEL_PBR` for metalness/roughness, [examples/95_pbr.gb](examples/95_pbr.gb)) including **emissive/neon glow** (`MODEL_EMISSIVE` — per-model self-illumination, real glow with bloom `POSTFX`, [examples/110_emissive_glow.gb](examples/110_emissive_glow.gb)) including **image-based lighting** — analytical (`LIGHT_ENV`, [examples/96_ibl.gb](examples/96_ibl.gb)) **and real HDR cubemap IBL** (`LIGHT_ENV_HDR` — loads a `.hdr`, computes irradiance/prefilter/BRDF-LUT maps; metals reflect the actual environment, [examples/99_ibl_hdr.gb](examples/99_ibl_hdr.gb)). **Fullscreen showcase:** [examples/97_pbr_reactor.gb](examples/97_pbr_reactor.gb) ("PBR REACTOR") — an FFT-reactive ring of chrome PBR spheres, IBL, shadows, bloom + stereo techno (CC0). **Coroutines/`YIELD`** also run natively — the Rust VM suspends via a frame snapshot (no threads, safe on the raylib main thread, deterministic) including in a standalone `.exe` ([examples/98_coroutines.gb](examples/98_coroutines.gb)). **Full module port complete** — every module that used to be Python-only now runs natively: `regex`, `tiled`, `tile_collide`, `controller`, extended `audio`, plus feature-gated `db` (rusqlite), `net` (std::net), `html` (ureq) and hardware/IoT `serial` (serialport), `usb` (hidapi), `wifi` (netsh/nmcli/networksetup), `bt` (btleplug/BLE). That leaves only the editor needing Python; everything else runs fully natively (`build_runtime.py --hardware` / `--full` for the heavier modules). Plan & status in [docs/rust-runtime.md](docs/rust-runtime.md) (German).

**Front-end port to Rust — complete.** The entire toolchain (lexer → parser → compiler → preprocessor) has been ported to Rust, each stage verified for output parity against the Python tree-walker. **`gbrt run file.gb` is a self-contained end-to-end run with no Python:** it preprocesses (`IMPORT` resolution for both source files and built-in modules), lexes, parses, compiles and executes — scalars/arithmetic/control flow, arrays/maps, functions, classes/OOP, `SELECT`/`FOR EACH`/tuples/`WITH`/`TRY`/slicing/comprehensions/coroutines. Like `gbrun.py`, it changes into the file's directory so relative `IMPORT` and asset paths resolve correctly (`gbrt file.gb` without `run` works the same way; `.gbc` files still use the direct VM path). Debug entry points: `gbrt --tokens`/`--ast`/`--preprocess`/`--runsrc`. **Self-export without Python:** `gbrt --export file.gb` compiles the source itself and bundles it into a self-contained `.exe` (appends the bytecode to a copy of the runtime, copies `assets/`). Aliased module imports (`IMPORT "json" AS j` → `J_PARSE`, `DIM h AS J_HANDLE`) work natively too. This also makes the **web playground pure Rust WASM**, compiling the source in the browser (no Pyodide): `rust/build_wasm.py file.gb` produces `web/gbrt.{js,wasm}` with the source embedded (the emscripten toolchain on Windows is wired up automatically). **Console and animated graphics both run in the browser** — the GB render loop yields every frame via ASYNCIFY (`emscripten_sleep(0)` inside `flip()`), so `WHILE … FLIP() … WEND` doesn't freeze the tab; **shareable links** pack the source into the URL hash. Plan & stages in [docs/rust-frontend-port.md](docs/rust-frontend-port.md) (German).

Architecture details and extension notes in [CLAUDE.md](CLAUDE.md) (German).

## Tests

```
.venv\Scripts\python.exe -m pytest tests/
```

1890+ tests — built-ins, every module, language constructs, editor features and example smoke tests. Correctness is guarded by **run_gb golden tests** (`assert run_gb(src) == expected`, spawn `gbrt run`) plus Rust `#[test]`s; they skip cleanly if `gbrt` isn't built.

## License

Private.
