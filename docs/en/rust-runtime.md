# Native Rust runtime (raylib) — migration

> **⚠️ Reading note (stage B).** This document describes the migration *and*
> refers in many places to the verification of that time, "bit-identical to the
> Python paths" (tree walker / Python VM / Cython VM), and to
> `interpreter.py`/`vm.py`/`serialize.py`. These Python paths and files have
> **all been removed** — `dhrt` is today the **only** runtime and compiles the
> source code itself. Correctness is now secured by the **test collections**
> (`tests/pruef/*.dhtest`, `dhrt test tests/pruef`) + Rust `#[test]`s. The
> "bit-identical" passages below are therefore historical port verification
> notes, not a current multi-path state.

The goal was modest at first: a **fourth** execution path next to the tree
walker, the Python VM and the Cython VM — a **native Rust runtime** that
executes the same bytecode and (later) renders graphics through **raylib**.
The Python toolchain (lexer → parser → compiler) was to stay unchanged, with
Rust only taking over the execution of the compiled `Module`. In the end,
exactly one of the four paths was left, and it brings its own front end.

## Migration plan (incremental, throw nothing away)

1. **Freeze/serialize the bytecode format** — write `.dhc` (Python) + read it
   (Rust). ✅ *done (spike)*
2. **Rust VM core** — dispatch loop + scalar ops + control flow. Test:
   console programs, `stdout == Python-VM`. ✅ *done (spike)*
3. Bring strings/arrays/maps/structs over to Rust (real Rust types instead of
   boxed ones — Drachenhauch's strict typing helps). ✅ *done*
4. Bring in raylib, graphics built-ins to Rust. ✅ *done (core 2D)*
5. Port the modules (gui/ui/physics …). ✅ *done — ALL modules native incl.
   ui (complete, with UI_TABLE) + gui (complete, with GUI_TABLE + callbacks)*
6. 3D built-ins on raylib's mesh/camera API. ✅ *core primitives done (`g3d`)*
7. Editor: "Export → bundle a native exe". ✅ *done (bytecode + assets bundled
   into one standalone `.exe` — see below)*

**Dev run loop** (across the steps): `dhrt run <datei.dh>` — one command that
compiles the source itself and runs it. ✅ *done* (see below; until 2026-09-21
there was the Python launcher `dhrun.py` in front of it).

## Open / next steps (as of 2026-06-01)

Steps 1–6 finished; native in addition: **audio incl. a real FFT**
(`AUDIO_FFT`), **game loop** (`DELTA`/`FPS`/`SETFPS`/`SET_FULLSCREEN`/
`SETWINDOWTITLE`/`SAVESCREENSHOT`; raylib's own screenshot on F12 is switched
off, F12 belongs to the program), **shaders/post-processing**
(`SHADER_LOAD`/`SET`/`SET2`/`SET3`/`POSTFX`, CRT/bloom/vignette), **TTF fonts**
(`LOADFONT`/`SETFONT`/`TEXT_SPACING`). The native runtime thereby covers a
complete 2D/3D game with sound, menus/tables and GPU effects.

**Worthwhile next levers (raylib offers more):**
- **Complete 3D stack:** models, GenMesh incl. heightmap, textures,
  **normal maps**, **PBR + analytic IBL** (`LIGHT_ENV`) **+ real HDR cubemap
  IBL** (`LIGHT_ENV_HDR`), billboards, ray collision/picking, lighting,
  **fog**, **shadows**, **camera modes** — see below.
- 2D polish **done (2026-06-04):** 2D extras (`LINEW`/`BOXROUND`/`RECTROUND`/
  `GRADIENTV`/`GRADIENTH`/`SPLINE`, dual-path), blend modes (`BLEND_MODE`,
  native), procedural textures (`GENTEX_PERLIN`/`GRADIENT`/`CHECKED`/`COLOR`,
  native), clipboard + drag&drop (`CLIPBOARD_GET/SET`,
  `FILES_DROPPED`/`FILE_DROPPED`, native), **render targets**
  (`RENDERTARGET_NEW`/`BEGIN`/`END`/`DRAW`, dual-path — dhrt: its own command
  buffer per target, rendered to the RenderTexture on FLIP before the main
  scene). Sound pan already exists (`AUDIO_PAN`). Demos
  `examples/100_2d_extras.dh`, `101_blend_gentex.dh`, `102_render_target.dh`.
- **Still open (small):** sound aliases (`LoadSoundAlias`, overlapping playback
  of the same sound); render target trails (currently cleared to transparent
  each frame).

**Full native port (ongoing):** the goal is to make ALL modules native, so
that only the editors need Python. Heavy/optional modules are feature-gated
along the way (the standard `.exe` stays lean).

- **Phase 1 — DONE (game logic, always on):** `regex` (crate `regex`, pattern
  cache, `\1`→`${1}` replacement translation), `tiled` (TILED_*, 36 — JSON
  loader via serde_json, properties, objects, bulk ops incl. flood fill),
  `tile_collide` (TILE_*, 4 — box sweep port), `controller` (CHAR_*, 24 —
  platformer physics). All verified bit-identical to the Python paths
  (`tiled`/`tile_collide`/`controller` against `examples/levels/level1.json`,
  `examples/77_tiled_platformer.dh` renders natively).
- **Phase 2 — DONE (extended `audio`, `graphics` gate):** AUDIO_* (27 — mixer
  lifecycle, channel playback with volume/pan/pause/resume/stop, music
  streaming with volume/position/queue, tone generation AUDIO_TONE/AUDIO_NOISE
  via in-RAM WAV → `new_wave_from_memory`/`new_sound_from_wave`). Functional
  (audio is not part of the bit-identical guarantee). SOUND/AUDIO_CHANNEL =
  INTEGER handles; raylib has no independent mixer channels, so a "channel"
  controls the playback of exactly its sound (volume tracked per handle, since
  raylib has no getter). Fade/loops=N are simplified (raylib cannot do this
  directly). *(As of 2026-06-01, i.e. before the backend switch: since 13 June
  audio runs on Kira, and there fades are native tweens and `loops=N` is
  implemented in the `AUDIO_*` family.)*

### Audio backend: Kira (cpal) — replaced raylib audio on 2026-06-13

raylib audio had a structural weakness: **music streaming was refilled from
the game loop (FLIP) through `UpdateMusicStream`** — on heavy frames the
buffer could underrun → crackling.

The audio backend therefore runs on **[Kira](https://crates.io/crates/kira)**
(cpal) in [`src/audio.rs`](../../rust/drachenhauch_runtime/src/audio.rs): its
own audio thread, completely decoupled from the game loop (no stuttering),
native tweens for fades/pan, an FFT tap as an effect on the main track
(replaces raylib's `AttachAudioMixedProcessor`), MOD/XM through a pure Rust
player (`xmrs`/`xmrsplayer` — no C). Included with `--features graphics`
(raylib stays for window/GL/input). In Kira, volume is in decibels (`db()`
helper), pan −1..1; `vm.rs` calls the audio API unchanged.

**Tracker modules** are **streamed in real time**: a Kira custom `Sound`
(`ModuleSound`) polls the pure Rust player `xmrs` per audio block on the audio
thread. Loaded immediately (no pre-render), exact endless looping (the player
counts the loops via `set_max_loop_count`), little RAM. Control
(volume/fade/pitch/pause/stop/position) through `Arc<ModShared>` atomics; a
pitch resampler (fractional read position + linear interpolation) and a volume
ramp (click-free fades) sit inside the sound. The module is leaked at start
(`Box::into_raw` → `'static` borrow for the player) and freed again by the
sound in its `Drop` (drop the player first, then `Box::from_raw`). Stream
formats (ogg/mp3/wav/flac) stream from disk through Kira.

Obvious extensions thanks to Kira (mixer tracks/effects): **buses** (separate
SFX/music masters) and **real-time effects** (filter/reverb/delay as new
built-ins, e.g. for SID character without a buffer bake).
- **Phase 3 — DONE (data/network, feature-gated):**
  - `db` (DB_*, 17, feature `db` → `rusqlite` bundled): SQLite, `?` binding,
    DB_QUERY loads rows eagerly (avoids self-referential cursors). DB_CONN/
    DB_RESULT = INTEGER handles. Verified **bit-identical** (CRUD, rowid, typed
    getters, rowcount).
  - `net` (NET_*, 19, feature `net`, only stdlib `std::net`): TCP listeners/
    sockets + UDP, non-blocking by default, INTEGER handles. Loopback verified.
  - `html` (HTTP_*/HTML_*/URL_*, 10, feature `http` → `ureq`+rustls): HTTP
    GET/POST/DOWNLOAD (https/TLS), URL encode/decode, HTML text/tag find/attr
    as a hand-written scanner. HTTP_GET verified (status 200, body), HTML
    parsing bit-identical (except non-ASCII console encoding -- an OS artefact
    like CRLF).
  - **Cargo features:** `db`/`net`/`http` + the aggregate `full`.
    `build_runtime.py` builds the standard dev set `graphics db net http`
    (`--no-data` leaves them out). If a feature is missing, the built-in
    returns the "not available" error.
- **Phase 4 — DONE (hardware/IoT, feature-gated):**
  - `serial` (SERIAL_*, 10, feature `serial` → `serialport`): RS-232/USB COM.
    Ports/Open/Read/Write/Readline/Available/Flush/Timeout. COM1 detected
    natively.
  - `usb` (USB_*, 9, feature `usb` → `hidapi`): HID. List/Open/Open_Path/Read/
    Write/Product/Manufacturer/Serial. latin-1 byte<->STRING. Devices listed.
  - `wifi` (WIFI_*, 8, feature `wifi`, Windows via `netsh wlan`, Linux via
    `nmcli`, macOS via `networksetup`/`airport`): Available/Current/Signal/
    Scan/Connect/Disconnect/Profiles/Delete. Windows branch verified;
    Linux/macOS new (2026-07, cross-platform migration phase 3) and NOT tested
    on real hardware, see docs/module-wifi.md.
  - `bt` (BT_*, 8, feature `bt` → `btleplug`+`tokio`): BLE, async->sync through
    a global tokio runtime (block_on per call). Scan/Connect/Services/
    Characteristics/Read/Write, latin-1. **BT_SCAN found real BLE devices**
    (with RSSI). Address->peripheral from the last scan for BT_CONNECT.
  - Handles = INTEGER index into cfg-gated VM vecs. `build_runtime.py
    --hardware` adds them. The default dev build leaves hardware out (keeps the
    heavy deps tokio/btleplug/windows out of the normal build).
  - **Early warning at IMPORT:** if a program imports a hardware module that
    the current build lacks, dhrt warns at the IMPORT already — `dhrt run` on
    stderr before the run, `dhrt --check` as `severity:"warning"` on the IMPORT
    line (editor marker). The runtime error at the first call remains in
    addition (`vm.rs::unknown_builtin_msg`). Logic in `preprocess.rs`
    (`missing_hardware_modules` / `missing_hardware_imports_with_lines`).

### Warnings from `dhrt --check`

Neither blocks (`severity:"warning"`), both show up as markers in the editor:

- **Shadowed constant:** a local variable has the same name as a global CONST.
- **Second `DIM` of the same name with a different type.** The effect depends
  on the level and is silent in both cases: in the main program the
  declaration *executed* last wins (the variable changes its type at run
  time), in a function the **first** one does (`Ctx::declare_local` returns
  the existing slot and discards the new type). The error then blows up far
  away (`Array-Index muss INTEGER sein, erhalten FLOAT`) and there it no longer
  shows its cause. The same name **with the same type** is reported too since
  2026-09-30, but only if it is a SECOND `DIM` statement and the block of the
  first one encloses it (both at the top, or the first at the top and the
  second in an IF below it) -- in the IDE a new `DIM miGross` overwrote the
  old one that way, and its shortcut triggered the wrong menu entry. `DIM` in
  a loop body (one statement) and `DIM` in two separate blocks (two IFs one
  after the other: a helper variable, `dim_noch_offen`) stay silent. A local
  `DIM` that shadows a global is not a case for this either (different
  scope). The comparison ignores upper and lower case, because Drachenhauch
  does not distinguish them. Logic: `compiler.rs::warn_dim_typ_wechsel`
  (state per `Ctx`, so automatically separate per function), tests
  `tests/pruef/dhrt_check.dhtest`.

**Full native port COMPLETE (2026-06-03):** all 12 modules that were
previously Python-only now run natively in dhrt. Only the editors still needed
Python back then (since 2026-09-21 not even they do).

### Real HDR cubemap IBL (`LIGHT_ENV_HDR`) — done

Replaces the analytic `LIGHT_ENV` approximation with real environment maps
from an `.hdr`. Port of raylib's `shaders_basic_pbr` / learnopengl IBL, purely
through raylib-rs `ffi`/`rlgl` (like the shadow mapping). Built-in
**`LIGHT_ENV_HDR(pfad$ [, intensität])`** (native-only). The pipeline runs
once in [`graphics.rs`](../../rust/drachenhauch_runtime/src/graphics.rs)
(`light_env_hdr()`):

1. **Load the `.hdr`** — its own Radiance RGBE decoder (`load_hdr_rgbe`) →
   RGBA32F 2D float texture via `rlLoadTexture`. (Needed because raylib-sys is
   built **without** `SUPPORT_FILEFORMAT_HDR` — `LoadImage("*.hdr")` fails.)
2. **equirect → cubemap** (512², `ibl_render_cube` + `EQUIRECT_FS`, 6 faces via
   `rlLoadDrawCube`).
3. **Irradiance cubemap** (32², `IRRADIANCE_FS` convolution) — diffuse IBL.
4. **Prefilter cubemap** (128² + roughness mips, `PREFILTER_FS` GGX importance)
   — specular IBL.
5. **BRDF LUT** (512², 2D, `BRDF_FS` + `rlLoadDrawQuad`) — once.
6. **PBR `LIGHT_FS`** extended with `samplerCube irradianceMap/prefilterMap` +
   `sampler2D brdfLUT` + a **`useIBLMaps` gate**. Set → real maps
   (`texture(irradianceMap,N)`, `textureLod(prefilterMap,R,rough·4)`,
   `texture(brdfLUT,vec2(NoV,rough))`); otherwise the **existing analytic
   `LIGHT_ENV` path** (default `useIBLMaps=0` → no `.hdr` needed, all demos
   unchanged). `envIntensity` remains the shared on/off gate.

The maps live as GL textures in `Graphics` (`ibl_irradiance/_prefilter/_brdf`)
and are bound in `render_scene` in the draw context to slots 11/12/13
(cubemaps via `rlEnableTextureCubemap`, the BRDF LUT 2D via `rlEnableTexture`;
material maps use 0..2, shadow 10 → no clash). Dispatch `"light_env_hdr"` in
[`vm.rs`](../../rust/drachenhauch_runtime/src/vm.rs), native-only stub in
`g3d.py`.

**Three rlgl stumbling blocks** (for anyone rebuilding this):
- `rlFramebufferAttach` ends with `glBindFramebuffer(0)` → **after every
  attach, `rlEnableFramebuffer` the FBO again** (otherwise the cube lands on
  the screen and the cubemap stays black).
- Empty **float cubemaps** (R32/R16) are rejected by `rlLoadTextureCubemap` →
  **R8G8B8A8** (LDR; very bright values clamp, good enough for reflections).
- The prefilter cubemap needs the **full mip chain** (128→1 = 8 levels),
  otherwise it is not *mipmap-complete* with `LINEAR_MIPMAP_LINEAR` and
  samples completely black; only the first 5 roughness levels are prefiltered
  (`MAX_REFLECTION_LOD=4`).

**Skybox** `SKYBOX(an)` additionally draws the env cubemap as a visible 3D
background (its own `SKYBOX_VS/FS`; `mat3(matView)` removes the translation →
camera-centred/infinite; drawn first in the 3D pass with `rlDisableDepthMask`
+ `rlDisableBackfaceCulling`, models draw on top of it). For this the env
cubemap is kept after the IBL generation (`ibl_env`). Without `LIGHT_ENV_HDR`
it is a no-op.

Demo [examples/99_ibl_hdr.dh](../../examples/99_ibl_hdr.dh): a row of chrome
metal spheres (`MODEL_PBR` metalness 1, roughness gradient) reflect the HDRI
**in front of the environment visible as a skybox**; a `FILEEXISTS` guard
falls back to analytic `LIGHT_ENV` without an `.hdr`. **Asset:**
`dhrt run examples/assets/download_hdri.dh` fetches a CC0 1k HDRI (Poly Haven,
kloofendal_43d_clear) as `examples/assets/ibl_env.hdr` (gitignored). Verified
by screenshot (mirror→diffuse across the roughness row). Bit identity does not
apply (GPU/3D); the analytic `96_ibl` stays unchanged.

## Dev run loop: `dhrt run`

For fast iteration while coding, one command is enough:

```
dhrt run examples\30_shapes.dh
```

`dhrt` brings its own front end (preprocess → lex → parse → compile → VM, all
in Rust) — **no Python, no `.dhc` on disk**. It starts in the directory of the
source file (so that relative asset paths like `LOADIMAGE("assets/…")` are
right); `dhrt` stores the caller's location as `DHRT_START_DIR`. Arguments for
the program come after `--` (`dhrt run spiel.dh -- level2`);
`dhrt run --bilder N datei.dh` ends the program by itself after N frames.
`dhrt datei.dh` without `run` does the same.

Until 2026-09-21 there was the Python launcher `dhrun.py` (`--native`) in front
of it; it was removed along with the Python toolchain.

**In the IDE** (`dhrt run ide/ide.dh`, see [ide.md](ide.md)): F5 starts the
program as its own `dhrt run` process; output and runtime errors
(`datei.dh:Zeile`) land in the output, and a double click on such a line jumps
to the spot.

### Runtime errors with line numbers

The compiler stamps the **source line** of every bytecode instruction into a
`lines` array parallel to `code` (stored in the `.dhc` as `"lines"`). The Rust
VM remembers the line of the instruction executed last (`Vm.cur_line`); when
an error propagates, the **innermost** failing line is kept. `dhrt run` knows
the source file name, so the message reads:

```
Laufzeitfehler in spiel.dh:42: Index 10 ausserhalb [0..2] in Dimension 0
```

`dhrt <datei.dhc>` without a label uses the `.dhc` path. Line `0` (untracked)
→ message without a line number.

**Compile errors** (before execution) carry the line where they arise as
well — parser errors as much as compiler errors (e.g. a SUB declared twice).
`dhrt --check datei.dh` reports them as JSON without starting the program.

### Messages in English (`DHRT_LANG=en`)

With `DHRT_LANG=en` (also `en_US` etc.) dhrt writes its messages in English:

```
Runtime error in game.dh:42: Index 10 outside [0..2] in dimension 0
game.dh:3: Parse error (12): Expected END IF, reached the end of the program
```

In the code the messages are still created in German; they are only
translated where they reach a human: the console (`dhrt run`),
`dhrt --check`, the language server (`dhrt lsp`), `CODE_CHECK$` and the
debugger (`dhrt debug`). The catalogue
[`daten/meldungen.en.txt`](../../daten/meldungen.en.txt) has, per line, a
German template, a tab and the English version; `{}` stands for an inserted
piece (in English also `{2}` for a different order). Whatever a placeholder
captures is translated in turn — which is why the parts are enough for
composite messages (`{} -- Aufruf: {}`, `{} Meintest du {}?`, the hints for
people coming from other BASICs). The most specific template wins.

- **Whatever no template matches stays German** — the correct German sentence
  is better than a guessed translation. The first round covered lexer,
  parser, compiler, IMPORT, the hints for people coming from other BASICs and
  the general runtime errors, the second the commands and modules (arrays,
  maps, text, numbers, files, buffers, JSON, XML, database, network, mail,
  processes, PDF, Tiled, hardware …). Measured against the 562 error messages
  of the test collections, 530 (94 %) then arrived in English. The third round
  added gui, graphics and sound (350 templates). Deliberately left German are
  the outputs of the command-line tools (`dhrt test`, `dhrt paket`, the
  export), the internal reasons of the machine code (`dhrt --jit`) and the
  error texts the operating system itself supplies ("Das System kann die
  angegebene Datei nicht finden", the system cannot find the file specified).
- **Placeholders are only `{}` and `{number}`.** An example with curly braces
  in the English version (`f"{value:.2f}"`, `` `{...}` ``) stays text -- until
  the third round it silently disappeared.
- **What a program sees stays German**: the value of a `CATCH` variable does
  not depend on an environment variable.
- **Quick fixes** still read the German sentence; only their title is
  translated.
- **`dhrt test`** removes `DHRT_LANG` from its cases (their expectations are
  written in German); a case that wants English sets it in `--- umgebung`.
- **`dhrt pruef meldungen`** checks the catalogue against the source code:
  every German template must still appear with its fixed pieces in a literal,
  and the placeholders must match. Where a message is put together from
  several literals, `[[wort]]` (a whole literal of its own) or `[[]]` (just
  the seam) marks the spot. `dhrt pruef meldungen --offen` lists, the other way
  round, which messages in the source code have no template yet (counted per
  file; pieces of composite messages appear there too, even if the whole is
  translated).
- **The best translation wins, not the topmost template**: what counts is
  the fixed characters of ALL templates that apply. Otherwise a general joint
  like `{} -- {}` would cut a sentence at the wrong place.

The IDE sets `DHRT_LANG=en` when its user interface is English.

## Step 1: `.dhc` serialization

A compiled program is a self-describing JSON file (`.dhc`). JSON was chosen
deliberately for the spike (debuggable); a compact binary format can be
dropped in later. At first the Python toolchain wrote it
(`drachenhauch/serialize.py`); today `dhrt` produces it itself — you can look
at it with

```
dhrt --dumpbc <datei.dh>
```

(indented JSON on stdout). To run it: `dhrt <datei.dhc>`; `dhrt --export`
appends the same `.dhc` to a copy of the exe.

**Value encoding** (uniquely decodable — INT/FLOAT/BOOL must remain
distinguishable, because `bool` is a subtype of `int` in Python and `1` ≠
`1.0` in Drachenhauch):

| DH value | JSON |
|---|---|
| `None` | `null` |
| `bool` | `{"b": true}` |
| `int` | `123` (plain number) |
| `float` | `{"f": 1.5}` |
| `str` | `"text"` |
| `tuple`/`list` | `[…]` |
| `COMP_MARKER` | `{"comp": true}` |
| `_FuncRef` | `{"funcref": "name"}` |

Code instructions: `[op:int, arg]`, `arg` with the same encoding. The Rust VM
knows per opcode which structure `arg` has (index, slot, name, tuple).

**Not yet serializable:** runtime handles in the const pool
(IMAGE/SOUND/MAP/ARRAY/instances). These come with step 3.

## Step 2: Rust VM core

Standalone crate [`rust/drachenhauch_runtime/`](../../rust/drachenhauch_runtime)
(separate from the PyO3 helper crate `rust/gb_native/`). Binary `dhrt`.

```
cd rust/drachenhauch_runtime
cargo build --release
target/release/dhrt <datei.dhc>
```

**Implemented:** LOAD_CONST/POP/DUP, locals (LOAD/STORE/DECLARE), global slots
(LOAD/STORE/DECLARE/DECLARE_CONST), name globals (fallback), full scalar
arithmetic (ADD/SUB/MUL/DIV/MOD/POW/NEG/INT_DIV) incl. the specialized `_NN`
opcodes, comparisons, bitwise, control flow (JUMP/JUMP_IF_*), user calls
(CALL_USER/RETURN/RETURN_VOID), PRINT, HALT.

**Bit-identity-critical semantics** (1:1 from `drachenhauch/vm.py`):
- `_fmt`: `NIL`/`TRUE`/`FALSE`; integral float → `{:.1f}` (always fixed point,
  even `1e16` → `10000000000000000.0`), non-integral → Python `repr` with the
  E notation threshold (`decpt <= -4` or `> 16`), rebuilt via Rust's `{:e}`.
  Verified bit-identical for small (`1e-09`), normal and large magnitudes.
- DIV: int/int with remainder 0 → int, otherwise float; `b==0` throws.
- INT_DIV (`\`): truncation towards 0 (= Rust `i64` division).
- MOD: sign of the result = sign of the divisor (Python semantics).
- integer coercion: float without a fractional part OK, `bool` throws.

**Additionally since step 3:** name globals (`DECLARE_NAME`/`DECLARE_CONST`),
tuples (`BUILD_TUPLE`/`UNPACK_TUPLE`/`BUILD_TUPLE_DYN`), `IN`, slicing, arrays
(multidimensional, `LOAD/STORE_INDEX`, `DECLARE_ARRAY_*`), maps, string index,
OOP (`NEW_INSTANCE`, fields, members, methods, `LOAD_SELF`, properties,
operator overloading, structs), FUNCREF (`LOAD_FUNCREF`/`CALL_VALUE`),
container methods (`"x".upper()`), DATA/READ (`PUSH_DATA`/`RESET_DATA_PTR`),
`TRY`/`THROW`/`CATCH`, and a **registry of pure/deterministic built-ins**
(`builtins.rs`): `STR$`, `VAL`, `INT`, `ABS`, `LEN`, `CHR$`/`ASC`, `SQR`,
`SIN`/`COS`/`TAN`/`ATAN`/`ATAN2`, `FLOOR`/`CEIL`/`ROUND` (banker's),
`LOG`/`EXP`/`POW`, `MIN`/`MAX`/`CLAMP`/`SIGN`, `UPPER$`/`LOWER$`,
`LEFT$`/`RIGHT$`/`MID$`/`INSTR`/`REPLACE$`/`TRIM$`/`SPLIT$`/`JOIN$`,
`PADL$`/`PADR$`/`SPACE$`/`REPEAT$`/`HEX$`/`FORMAT$`, `RGB`, `MAP*`,
`SORT`/`REVERSE`/`ARRAY_INDEXOF`, `RANGE`, comprehension helpers.

**Additionally since step 4:** `RND`/`RANDOMIZE`/`MILLIS`/`TIMER` (PRNG or
SystemTime — NOT bit-identical, by definition).

**Not yet in the core:** file I/O, ENUM/STATIC namespaces (whose const pool
handles are not yet serializable), modules (`IMPORT` → step 5), bulk draws
(`PLOTS`/`BOXES`/…), layers/atlas. *(All of these have since been
implemented — see the respective steps below.)*

**Coroutines/`YIELD` — supported natively (frame snapshot):** the Python VMs
drive coroutines with threads; the native Rust VM instead uses a **frame
snapshot** (no OS threads → the raylib main thread stays safe, deterministic
by construction). `dispatch` returns `Step::Return | Yield`; on `YIELD_VALUE`
(opcode 115) the frame (`ip`/`locals`/`stack`/`try_handlers`) is stored in a
`Value::Coroutine` (`CoroState`) and restored on `CORO_RESUME`/`SEND`. The
`CoroState` holds a raw `*const Func` to the `Func` — unchanged for the whole
run time of the program — so `Value` needs no lifetime parameter. The
single-frame solution is possible because Drachenhauch allows **no
cross-frame `YIELD`** (a helper with `YIELD` is itself a coroutine): only the
topmost coroutine frame has to be resumable, nested normal calls keep running
recursively on the native stack. `CORO_*` built-ins go through `try_coro`
(they need VM state); `FOR EACH`/comprehensions over a coroutine drain it via
`__comp_iter`. Verified bit-identical to all three Python paths incl. the
standalone `.exe`
([examples/98_coroutines.dh](../../examples/98_coroutines.dh)).

### Validation

The `stdout` of the Rust VM is bit-identical to the Python VM (modulo the OS
newline: Python writes `\r\n` on Windows, Rust `\n` — semantically identical).
Verified by a full sweep: **30 examples bit-identical** (incl. OOP, arrays,
maps, tuples, strings, all benchmarks), 0 real mismatches.

## Step 4: raylib graphics

Graphics is **feature-gated** (`graphics`, off by default): the pure VM core
builds without a C toolchain. With graphics,
[`raylib`](https://crates.io/crates/raylib) (raylib-rs 6.0,
[raylib-rs/raylib-rs](https://github.com/raylib-rs/raylib-rs)) is brought in.

**Bit identity does NOT apply to pixels** (only the native runtime renders
graphics) — only `PRINT`/stdout stays bit-identical. Graphics is verified by
screenshot.

### Build (with graphics)

raylib compiles its C sources via **cmake** and needs **libclang** for the FFI
bindings (bindgen). The helper sets up the environment:

```
python rust\build_runtime.py            # release, mit Grafik
python rust\build_runtime.py --no-graphics
```

Prerequisites (Windows): VS C++ Build Tools (provide `cl.exe` + a bundled
cmake), LLVM for `libclang.dll` (`winget install LLVM.LLVM`). The `cc` crate
finds `cl.exe` automatically; `build_runtime.py` sets the cmake PATH and
`LIBCLANG_PATH`.

**Cross-platform (Linux/macOS): experimental, not yet verified on real
hardware** — development/CI have so far run exclusively under Windows.
`build_runtime.py` recently learned to detect the operating system and on
Linux/macOS looks for `cmake`/`clang` only via `PATH` (installation through the
package manager assumed: `apt install cmake clang libclang-dev` or
`brew install cmake llvm`), instead of searching fixed paths as under Windows.
The `wifi` feature stays Windows-only for now (`netsh`-based) — on other
systems it does build along, but `WIFI_*` built-ins fail at run time with a
clear error message.

### Built-ins ([`graphics.rs`](../../rust/drachenhauch_runtime/src/graphics.rs))

`SCREEN`, `CLS`, `FLIP`, `PLOT`, `LINE`, `BOX` (filled), `RECT` (outline),
`CIRCLE`, `TRIANGLE`/`TRIANGLEOUTLINE`, `ELLIPSE`/`ELLIPSEOUTLINE`, `ARC`,
`POLYGON`/`POLYGONOUTLINE`, `TEXT`/`TEXT_SIZE`/`TEXT_WIDTH`/`TEXT_HEIGHT`,
`TEXT_BOLD`/`TEXT_ITALIC` (no-op — default font), `LOADIMAGE`/`DRAWIMAGE`/
`IMAGEWIDTH`/`IMAGEHEIGHT`, `KEYPRESSED`, `MOUSEX`/`MOUSEY`/`MOUSEBUTTON`,
`QUITREQUESTED`, `SLEEP`.

**Game loop basics** (native in dhrt/raylib): `DELTA()` (seconds since the last
FLIP, frame-rate-independent movement), `FPS()`, `SETFPS(n)` (target frame
rate, 0 = unthrottled), `SET_FULLSCREEN(an)` (natively a real
`ToggleFullscreen`, no longer a no-op), `SETWINDOWTITLE(s)`,
`SAVESCREENSHOT(pfad)`. Natively through raylib's `GetFrameTime`/`GetFPS`/
`SetTargetFPS`/`ToggleFullscreen`/`SetWindowTitle`/`TakeScreenshot`.

**Bulk draws:** `PLOTS`/`BOXES`/`CIRCLES`/`LINES` (coordinate arrays + colour as
INT or ARRAY). **Images extended:** `DRAWIMAGEPART`, `DRAWIMAGEFLIPPED`,
`LOAD_ASSETS` (manifest preloading + alias/path cache, `LOADIMAGE("alias")`
hits). **Z layers:** `LAYER_DEFINE`/`LAYER`/`LAYER_END`/`LAYER_CLEAR` — FLIP
composites all layers in ascending z order. **Sprite atlas:** `ATLAS_LOAD`
(JSON manifest: `{"image":..., "sprites":{name:[x,y,w,h]}}`),
`ATLAS_DRAW`/`ATLAS_DRAW_FLIPPED`, `BATCH_DRAW`/`BATCH_FLUSH` (in the recording
model everything flushes on FLIP).

**Model:** draw built-ins append `Cmd`s to a list; `CLS` empties it +
remembers the clear colour; `FLIP` renders all cmds in one `begin_drawing`
block and presents. That way no raylib draw handle has to be held across
built-in calls. Colours are `&HRRGGBB` INTEGERs → raylib `Color`.

**Predefined globals:** colours (`BLACK`/`WHITE`/`RED`/…), keys (`KEY_*` as SDL2
keycodes, mapped to raylib keys in `KEYPRESSED`) and `PI` are pre-registered
with values identical to Python (`register_default_globals`).

### Headless verification

`dhrt` renders headless and writes a screenshot, controlled through ENV:

```
DHRT_FRAMES=3 DHRT_SCREENSHOT=out.png dhrt programm.dhc
```

After `DHRT_FRAMES` frames `QUITREQUESTED()` returns `true` (the loop ends
cleanly); when the limit is reached the PNG is saved (even if the program uses
a fixed `FOR` loop instead of `QUITREQUESTED`). **Note:** raylib's
`TakeScreenshot` places the file relative to the working directory.

The same limit **without an environment variable**:
`dhrt run --bilder 20 spiel.dh`. You need this when checking from inside a
PROGRAM — `PROCESS_START` strips `DHRT_FRAMES` from its children so that a
launched game does not die after N frames. A program that never asks
`QUITREQUESTED()` keeps running anyway: **that is exactly how you spot it**
(`tests/pruef/beispiele_gui_enden.dhtest`).

#### Contact sheet: checking a SEQUENCE instead of a moment

A single screenshot shows a moment. But much goes wrong only over time --
something flips over too early, an edge stays behind, a movement stutters.
`DHRT_CONTACT` captures frames at fixed intervals and puts them, labelled, as
a grid into ONE PNG:

```
DHRT_FRAMES=480 DHRT_CONTACT=bogen.png dhrt run demo.dh
```

| Variable | Effect |
|---|---|
| `DHRT_CONTACT` | path of the grid PNG (switches the capture on) |
| `DHRT_CONTACT_MAX` | how many frames (default 12) |
| `DHRT_CONTACT_COLS` | columns in the grid (default 4) |
| `DHRT_CONTACT_EVERY` | interval in frames; if not given, spread evenly over `DHRT_FRAMES` |

The tiles are scaled down to at most 480 pixels wide (readable enough for
judging, a handy file size) and carry their frame number. The file is written
as soon as `DHRT_CONTACT_MAX` frames are together or `DHRT_FRAMES` is reached.
Without `DHRT_CONTACT` nothing changes.

Verified (visually by screenshot) on all 7 IMPORT-free graphics examples:
`30_shapes` (all shapes), `44_language_showcase`, `34_schneefall`,
`39_textscroll`, `40_parallax`, `75_preloader` (LOAD_ASSETS + alias cache),
`76_layers_atlas` (ATLAS_LOAD + BATCH_DRAW + z-layer compositing).

## Step 5: Modules (IMPORT)

Module built-ins reach the compiler automatically: `IMPORT "x"` loads the
Python module `drachenhauch.modules.x` in the preprocessor, whose `@builtin`
decorators fill the `BUILTINS` registry → the compiler emits `CALL_BUILTIN`.
The Rust VM only has to implement the respective built-ins — **no serializer
change needed**.

**Ported (pure, verified bit-identical):**
- `vec2` — `Value::Vec2(f64,f64)` (immutable value type) + operator overloading
  (`+`/`-`/`*`/`/` via `module_op`, before user operators). `VEC2_NEW/ZERO/X/Y/
  LENGTH/LENGTH_SQ/NORMALIZE/DOT/CROSS/DISTANCE/LERP/PERP/REFLECT/ANGLE/FROM_ANGLE`.
- `curves` — `CURVE_LERP/SMOOTHSTEP/SMOOTHERSTEP/BEZIER/BEZIER2/CATMULL/CATMULL2/HERMITE`.
- `physics` (pure) — `PHYSICS_BOX_BOX/CIRCLE_CIRCLE/BOX_CIRCLE/POINT_BOX/POINT_CIRCLE/
  DISTANCE/DISTANCE2/LENGTH/NORM_X/NORM_Y/REFLECT_X/REFLECT_Y/RAY_BOX/RAY_CIRCLE`.
  (Broadphase with an external type not yet.)
- `input` — `INPUT_BIND/UNBIND/RESET/UPDATE/HELD/PRESSED/RELEASED/AXIS/BOUND`.
  Edge detection via prev/cur snapshots; key state via raylib (`INPUT_UPDATE`
  without a window = no keys → console demos bit-identical). **Gamepad** is
  now native: the `JOY_BUTTON_*`/`JOY_DPAD_*` bind codes (negative) are
  registered as globals, and `key_down(negative code)` polls **all** connected
  pads via `IsGamepadButtonDown` (like Python's `_poll_joysticks_into`) —
  `INPUT_BIND("jump", JOY_BUTTON_A)` + `INPUT_HELD/PRESSED` work.
  `INPUT_JOY_COUNT` (contiguous from slot 0), `INPUT_JOY_NAME(idx)`,
  `INPUT_JOY_AXIS(pad, "left_x"|…|"rt")` (dead zone 0.15 for sticks) via
  raylib's gamepad API. Axis names → `GamepadAxis` indices (Xbox layout).
  Without a pad: count 0, axes 0.0, buttons false (no crash).
- `camera` — `CAMERA_SET/RESET/X/Y/ZOOM/FOLLOW/S2W_X/S2W_Y`. The world→screen
  transform (`w2s`/`ssize`) is applied in all draw methods; the TEXT position
  is transformed, the font size stays. 141_camera_visual renders correctly.
- `sprite` — `Value::Sprite` (sheet animation). `SPRITE_NEW/SET_POS/SET_VELOCITY/
  GET_X/Y/WIDTH/HEIGHT/SET_FLIP/SET_SCALE/TINT/TINT_CLEAR/ADD_ANIM/PLAY/PLAY_ONCE/
  CURRENT_ANIM/IS_FINISHED/SET_FRAME/GET_FRAME/UPDATE/COLLIDES/HIT_BOX/HIT_POINT`
  (in `builtins.rs`) + `SPRITE_DRAW` (sheet frame as a sub-rect, camera-aware,
  flip/scale/tint, in `graphics.rs`). 143_sprite_visual + 66_sprite_editor
  render. *Limit:* console demos without `SCREEN` do not work (textures need a
  GL context).
- `tween` — `Value::Tween`, 19 easings, `TWEEN_NEW/_LOOP/_PINGPONG/VALUE/PROGRESS/
  DONE/RESTART/PAUSE/RESUME/REVERSE/EASINGS`. **Time-based** (wall clock) → like
  `RND`/`MILLIS` NOT bit-identical, but functional (47_collision_easing
  renders).
- `json` — `Value::Json` (serde_json with `preserve_order`). `JSON_PARSE/LOAD/
  STRINGIFY/PRETTY/GET_STRING/INT/FLOAT/BOOL/HAS/LEN/TYPE`, path navigation
  (`"user.name"`/`"items.0"`). Bit-identical incl. `STRINGIFY` (key order).

- `scene` — stack scene manager (VM-global state). `SCENE_PUSH/POP/SWITCH/
  CURRENT/DEPTH/HAS/RESET/SET_INT|FLOAT|STRING|BOOL/GET_*(_OR)/HAS_KEY/DELETE`.
  Bit-identical.
- `save` — save slots (`Value::Save`). `SAVE_NEW/LOAD/LOAD_OR_NEW/EXISTS/WRITE/
  DELETE_FILE/VERSION/SET_VERSION/SET_*/GET_*(_OR)/HAS/DELETE/CLEAR/KEYS`.
  JSON file backend (serde_json pretty). Bit-identical (incl. file round trip).

**Core file I/O** (no module): `OPENFILE`/`CLOSEFILE`/`READLINE`/`READALL$`/
`ENDOFFILE`/`WRITELINE`/`WRITE`/`FILEEXISTS` (`Value::File`). Bit-identical.

- `astar` — A* pathfinding (`Value::AStar`,
  [astar.rs](../../rust/drachenhauch_runtime/src/astar.rs) ported from the PyO3
  helper `gb_native`). `ASTAR_NEW/CLEAR/WIDTH/HEIGHT/SET_WALL/
  SET_PASSABLE/IS_WALL/SET_DIAGONAL/SET_HEURISTIC/SET_DIAGONAL_COST/FIND/PATH_LEN/
  PATH_X/PATH_Y/PATH_COST/CLEAR_PATH`. Bit-identical incl. the counter-FIFO tie
  break.

- `particles` — `Value::Particles`. `PARTICLE_SYSTEM_NEW/SET_POS/COUNT/CLEAR/
  SET_VELOCITY/SET_LIFETIME/SET_GRAVITY/SET_COLOR/SET_SIZE/SET_FADE/SET_MODE/
  SET_COLOR_END/EMIT/UPDATE/DRAW`. 5 render modes (circle/pixel/square/streak/
  glow), fade + colour gradient. RNG emit → like `tween` time-dependent/not
  bit-identical, but functional (78_particle_catalog renders all modes).

- `imgfx` — `IMAGE_SCALE/ROTATE/FLIP/TINT/COPY` (immutable, new handle).
  Transformed through a raylib `Image` (CPU pixels) + `LoadTextureFromImage`.
  Textures are held as `Tex { tex, img }` (GPU+CPU).

- `ecs` — entity component system
  ([ecs.rs](../../rust/drachenhauch_runtime/src/ecs.rs), sparse set).
  `ECS_NEW_WORLD/NEW_ENTITY/DESTROY/ALIVE/COUNT`, `ADD_INT/FLOAT/
  STRING/BOOL/OBJ`, `HAS/REMOVE/GET*/GET_OR_*`, `QUERY/QUERY2/QUERY3` (sorted
  intersection), bulk ops `INTEGRATE_FLOAT/INT/SCALE_FLOAT/FILL_*/CLAMP_FLOAT/
  REMOVE_DEAD/COUNT_WITH`. Bit-identical.

- `ui` (immediate mode) — `UI_LABEL/BUTTON/CHECKBOX/SLIDER/PROGRESS/PANEL/RADIO/
  END_FRAME/RESET` + theme (`UI_THEME_SET/GET`, `UI_METRIC_SET/GET`,
  `UI_THEME_PRESET` dark/light/retro/contrast). State per string ID on the VM.
  **Newly ported:** `UI_TEXTFIELD`/`UI_TEXTFIELD_SET` (keyboard via the new
  `Graphics::pop_text_input` = draining raylib's `get_char_pressed`, Backspace
  edge, blinking caret) and `UI_WINDOW_BEGIN/END` (movable immediate-mode
  windows: offset threading through ALL widgets via `UiState.offset_x/y`, input
  gating of covered windows via `ui_mouse_gated`, title drag + collapse arrow,
  z-order via a previous-frame hit test `active_win`/`hover_win` in
  `UI_END_FRAME`). Both verified by headless screenshot.
  **`UI_TABLE`** has now been ported as well: fixed header row, V/H scrolling
  (mouse wheel + scrollbar drag incl. track click), per-cell text and
  background colours, hover/selection highlight, clickable rows (return value
  = clicked row) + `UI_TABLE_SELECTED`/`SET_SELECTED`/`HEADER_CLICK` (sorting
  hook). State per id in `UiState.tables`; uses the new clip stack + mouse
  wheel. Verified by screenshot
  ([examples/43_ui_table.dh](../../examples/43_ui_table.dh), cell colours +
  both scrollbars).

**`gui` (retained mode)** — core ported
([gui.rs](../../rust/drachenhauch_runtime/src/gui.rs)): window +
button/label/checkbox/slider/text input/panel, drag/z-order/focus/close,
programmable theme (`GUI_THEME_SET/GET/PRESET`, `GUI_METRIC_SET/GET`,
`GUI_SET_COLOR`), polling (`GUI_CLICKED/CHECKED/VALUE/TEXT`, setters). Handles
= INTEGER (window = index, widget = `(win<<20)|idx`); z-order through a
separate `z_order` list so that handles stay stable. Rendering verified by
screenshot (`examples/45_gui.dh`); the interaction is a 1:1 port of the
(tested) Python logic — not click-testable headless. **FUNCREF callbacks**
(`GUI_ON_CLICK`/`GUI_ON_CHANGE`) fire: `update()` collects triggered handlers
in `pending`, the VM empties the queue after `GUI_UPDATE` and calls them
(without parameters) via `exec` — so a callback can safely change the GUI, and
new events land in the next frame.

**`GUI_TABLE`** has been ported natively: fixed header row, V/H scrolling
(mouse wheel + scrollbar drag), hover/selection highlight, clickable rows,
`GUI_TABLE_HEADERS/ROWS/COL_WIDTHS/SELECTED/SET_SELECTED/CLICKED/ROW_COUNT` +
`GUI_ON_CHANGE` on selection change. Layout from one source (`table_geom`).
For this, the graphics gained a **mouse wheel** (`pop_mouse_wheel`) and a
**clip stack** (`push_clip`/`pop_clip` → raylib scissor with intersection).
Rendering verified by screenshot
([examples/84_gui_table.dh](../../examples/84_gui_table.dh)); Rust unit tests
cover callback queueing, table layout (`table_geom`) and press→selection
(`build_runtime.py --test`). *Still open:* the immediate-mode `UI_TABLE`
(separate module `ui`).

**Tables complete:** `UI_TABLE` (immediate mode) **and** `GUI_TABLE`
(retained) are native. Hardware/network + `regex` stay outside.

**ENUM / STATIC CONST:** `_EnumNamespace`/`_ClassStaticNamespace` are
serialized as `{"ns": {name, members}}` and loaded in Rust as
`Value::Namespace`; `LOAD_MEMBER` resolves members case-insensitively. With
that there are **no more serialize errors** (previously 6).

**Not planned:** hardware/network modules (`bt`/`serial`/`usb`/`wifi`/`net`/
`html`/`db`) and `regex` (Python's `re` cannot be rebuilt bit-identically).
The extended `audio` module (channels, pan, tone generation) stays Python-only
for now — the **core audio built-ins**, however, are native (see below).

## Audio (core: SFX + stream music)

Native audio through **Kira/cpal** (module `audio.rs`, feature-gated like the
graphics — with `--features graphics`). How the backend works is described
above under
[Audio backend: Kira](#audio-backend-kira-cpal--replaced-raylib-audio-on-2026-06-13);
here only the built-ins. **Core built-ins, no `IMPORT` needed:**
- `LOADSOUND(pfad$) -> SOUND` (handle = INTEGER index), `PLAYSOUND(sound[,
  loops, lautstaerke])`, `STOPSOUND(sound)`.
- `PLAYMUSIC(pfad$[, loops, lautstaerke])`, `STOPMUSIC()` — one stream at a
  time. Refilling happens on the audio thread, not in the game loop: there is
  **no** `update_stream` call per `FLIP` (exactly the point where raylib audio
  crackled). Music loops endlessly — `play_music` passes on `loops = -1`,
  which becomes a `loop_region(0.0..)` in Kira.

**Audio reactivity (FFT):** `AUDIO_FFT(bands)` fills an `ARRAY OF FLOAT` with
B logarithmically spaced frequency band levels (0..1) of the **currently
audible** audio. For this, `Audio::new` attaches a Kira `Effect`
(`FftTapEffect`) to the **main track**; its `process` runs on the audio thread
and pushes the mixed mono signal into a global ring buffer (`try_lock`, the
audio thread never blocks). `fft_bands` applies a window (Hann), computes its
own radix-2 FFT (`FFT_N = 1024`), bundles log-spaced bands, normalizes with
auto-gain and smooths with peak hold. (`AUDIO_FFT` is native; without the mix
tap it fills zeros.) This way both the spectrum **and** the geometry of the
demo really dance to the music.

WAV/OGG/MP3/FLAC — decoded by **symphonia** (through Kira: `bundle-flac`,
`bundle-mp3`, `codec-pcm`, `codec-vorbis`, `format-ogg`, `format-riff`), plus
MOD/XM through `xmrs`. So the set depends on the crate, not on the build flag.
Audio is **not** part of the bit-identical guarantee — like `RND`/`MILLIS`/
`tween` it is only functional.

*Limit:* the two **legacy built-ins** do not evaluate `loops` — `PLAYSOUND`
plays once, `PLAYMUSIC` always loops, the third argument is discarded. Whoever
needs a finite number of repetitions takes the `AUDIO_*` family: `AUDIO_PLAY`
passes `loops` down to `start_slot` and counts them down.
Demo: [examples/83_audio.dh](../../examples/83_audio.dh).

## Step 6: 3D graphics (module `g3d`)

3D requires raylib's pipeline. The module `g3d` registers the built-ins (so
the compiler emits `CALL_BUILTIN`); rendering goes through raylib's
`begin_mode3D` API. Built without the graphics feature (`--no-graphics`), they
return the "not available" error like the other graphics built-ins.

**The commands and how to use them are in [module-g3d.md](module-g3d.md)** —
this page only covers how they are built.

**Render model** (extends the 2D recording): 3D cmds land in a list of their
own, `cmds3d`; on `FLIP` `dhrt` renders **first** all 3D cmds in a
`begin_mode3D(cam3d)` block, **then** the 2D layers on top — so the 2D HUD
always lies above the scene. Coordinates are world units (no screen scale),
colours `&HRRGGBB`. `cmds3d` is emptied each frame; without `CAMERA3D` a
default view applies (looking at the origin obliquely from front-top).

Demo: [examples/82_3d_intro.dh](../../examples/82_3d_intro.dh) (cube, sphere,
cylinder, cone, lines, grid + 2D HUD), verified by screenshot.

### 3D models (loaded + procedural)

Reusable model handles (INTEGER) instead of primitives rebuilt every frame.
Commands: see [module-g3d.md](module-g3d.md#models).

**Implementation** ([graphics.rs](../../rust/drachenhauch_runtime/src/graphics.rs)):
`models: Vec<Model>` lives for the whole run time (handles stay valid). The
draw built-ins emit `Cmd3D::Model`/`ModelEx`/`ModelWires` with the model
**index** (not the model itself → `Cmd3D` stays `Clone`); the 3D pass in
`render_scene` (now with a `models: &[Model]` parameter) draws them via
`draw_model[_ex|_wires]`. GenMesh meshes are passed to `load_model_from_mesh`
with `make_weak()` (no double drop). Handle validation in the wrappers
(`check_model`).

Demo [examples/88_3d_models.dh](../../examples/88_3d_models.dh): rotating
torus, knot (wireframe) and pulsating sphere on a plane, orbiting camera + 2D
HUD — purely procedural, **no model asset needed in the repo**. Verified by
screenshot (incl. `MODEL_TEXTURE` with `assets/coin.png` on a cube).

`MESH_HEIGHTMAP` builds a terrain mesh from the CPU `Image`
(`self.textures[i].img`, already held for imgfx) via `GenMeshHeightmap`. Demo
[examples/89_heightmap.dh](../../examples/89_heightmap.dh): textured terrain
orbited by a camera, with a wireframe overlay
(`examples/assets/heightmap.png`, a generated 129×129 greyscale PNG). Verified
by screenshot.

### Billboards + ray collision / picking

`BILLBOARD` draws via `DrawBillboard` (its own `Cmd3D::Billboard` with a
texture index; the 3D pass has the camera **and** `textures` at hand). The hit
tests build on raylib's `GetRayCollision*`; `PICK_*` puts the mouse ray
(`GetScreenToWorldRay`) in front, `RAY_HIT_*` takes it from the caller. The
direction is normalized before the test — otherwise the distance would come
out in multiples of the direction's length (raylib's raw behaviour).

What the commands do and how to use them: [module-g3d.md](module-g3d.md#clicking-and-hits).

### Camera modes (`UpdateCamera`)

`cam3d` lives across frames; `CAMERA3D_UPDATE(mode)` passes it on to raylib's
`UpdateCamera` (1=free, 2=orbital, 3=first_person, 4=third_person) — raylib
reads keyboard and mouse itself in the process. The getters return position
and target exactly (verified).

### Lighting (PBR / Cook-Torrance, up to 4 lights)

Real per-pixel lighting through the embedded lighting shader (GLSL 330, as
`const LIGHT_VS`/`LIGHT_FS` in the crate — **no shader asset needed**). The
fragment shader uses the **Cook-Torrance PBR model** (GGX normal distribution,
Smith geometry, Fresnel-Schlick) with Reinhard tone mapping + gamma; analytic
lights (directional/point), no IBL:

- `LIGHT_ENABLE()` loads the lighting shader (once) and caches the uniform
  locations (`viewPos`, `ambient`).
- `LIGHT_AMBIENT(farbe, intensitaet)` — base brightness.
- `LIGHT_DIRECTIONAL(dx,dy,dz, farbe)` (sun, direction) /
  `LIGHT_POINT(x,y,z, farbe)` (point light) → light index (max. 4, otherwise
  `-1`).
- `LIGHT_SET_POS/COLOR/ENABLED(idx, …)` — animate lights per frame.
- `MODEL_LIT(modell)` — attaches the lighting shader to the model's materials
  (sets `material.shader` directly via the ffi field; the shader stays alive
  in `Graphics.light_shader`). Only after that is the model lit.
- `MODEL_PBR(modell, metalness, roughness)` — PBR material parameters (each
  0..1): `metalness` 0 = dielectric (plastic/stone), 1 = metal (the specular
  takes on the albedo colour); `roughness` 0 = mirror-like, 1 = matte. Stored
  per model (`pbr_params` map, default 0 / 0.6) and set as a uniform in
  `render_scene` before each model draw — together with `useNormalMap`. Albedo
  = `colDiffuse` (MODEL tint) × `texture0`. Demo
  [examples/95_pbr.dh](../../examples/95_pbr.dh): sphere grid metalness ×
  roughness (verified by screenshot).
- `LIGHT_ENV(himmel, boden, intensität)` — **analytic image-based lighting**
  (`intensität` 0 = off). The environment is a vertical colour gradient
  (ground→sky); the shader adds diffuse hemisphere irradiance + a sky
  **reflection** blurred depending on roughness (reflected view vector) + the
  analytic environment BRDF (Karis approximation instead of a LUT), with
  roughness Fresnel. **No HDR asset, no cubemap passes** — a pure shader
  extension. Only with this do metals (`metalness` 1) look truly metallic
  (they reflect the environment instead of staying dark). Demo
  [examples/96_ibl.dh](../../examples/96_ibl.dh): metal vs. dielectric row.
- `LIGHT_FOG(farbe, dichte)` — exponential depth fog for the lit models
  (`dichte 0` = off). Distant objects fade into the fog colour; in the fragment
  shader `mix(fogColor, finalColor, 1/exp((dist·dichte)²))`. Tip:
  `CLS(fogColor)` for a seamless horizon. Verified by screenshot (a row of
  columns disappears into the haze,
  [examples/92_fog.dh](../../examples/92_fog.dh)). *Limit:* only affects
  `MODEL_LIT` models (they use the lighting shader), not immediate
  primitives/grid.

Before the 3D pass, `flip()` calls `update_light_uniforms()`: `viewPos`
(= camera position), `ambient` and all `lights[i].*` uniforms are written to
the shader. raylib binds `matModel`/`matNormal`/`mvp` automatically through
the standard uniform names; only `matModel` is set explicitly in the `locs`.
Demo [examples/91_lighting.dh](../../examples/91_lighting.dh): sun + moving
point light on sphere/cube/torus, orbital camera. Verified by screenshot
(diffuse + specular highlights + the point light's light cone on the ground).

### Shadows (shadow mapping)

Real cast shadows through a depth pass from the light's point of view (port of
the official raylib `shaders_shadowmap` example into the recording pipeline):

- `SHADOW_ENABLE([auflösung])` — creates a **sampleable depth FBO**
  (`rlLoadFramebuffer` + `rlLoadTextureDepth(res, res, false)` +
  `rlFramebufferAttach` as `RL_ATTACHMENT_DEPTH/TEXTURE2D`; the default
  `RenderTexture` depth is a renderbuffer, i.e. *not* sampleable) and caches
  the shader locations. Default 1024, clamped to 256…4096.
- `SHADOW_AREA(größe, distanz)` — half the edge length of the orthographic
  shadow frustum + the distance of the light camera (smaller = sharper).
- `SHADOW_TARGET(x,y,z)` — centre of the shadow area (e.g. let it follow the
  player).

**Sequence per frame** (`render_shadow_map`, before the main pass): an
orthographic light camera is built from the first `LIGHT_DIRECTIONAL`;
`rlEnableFramebuffer` → `rlViewport(res)` → `BeginMode3D(lightCam)` → all
`MODEL`/`MODEL_EX` draws are rendered into the depth map via
`ffi::DrawModel*`. `lightVP = lightView·lightProj` (from
`rlGetMatrixModelview/Projection`) goes into the lighting shader as a uniform;
the depth texture is bound to texture unit 10 (`rlActiveTextureSlot`/
`rlEnableTexture`, the sampler uniform set to 10 — material maps use 0…2, no
clash). The fragment shader transforms every point into light space and
compares with the depth map (**3×3 PCF** + a normal-dependent bias against
shadow acne); in shadow 15 % of the direct light remains (ambient untouched).
`shadowsEnabled` uniform (default 0) → without `SHADOW_ENABLE` no effect at
all, existing lighting demos unchanged.

*Casters/receivers:* `MODEL_LIT` models (immediate primitives/grid cast no
shadows). One shadow-casting directional light. Demo
[examples/93_shadows.dh](../../examples/93_shadows.dh): a floating
sphere/cube/torus cast soft shadows on the ground (verified by screenshot).

### Normal mapping

Per-pixel surface detail without more geometry, integrated into the lighting
shader:

- `MODEL_TEXTURE_NORMAL(modell, bild)` — puts a normal map loaded via
  `LOADIMAGE` (tangent space, RGB = normal·0.5+0.5) onto a `MODEL_LIT` model
  (`MATERIAL_MAP_NORMAL` = shader sampler `texture2`).
- `MODEL_LIT` now additionally generates the **tangents**
  (`gen_mesh_tangents` on all meshes) — a prerequisite for the TBN basis in the
  shader.

**Shader:** the VS passes `vertexTangent` (world space) through; the FS builds
a TBN matrix from `fragNormal` + `fragTangent`, samples the normal map and
perturbs the normal per pixel. A `useNormalMap` uniform (default 0) gates this
— set per model in `render_scene` (1 only for models in `normal_mapped`). That
way lit models without a normal map stay **pixel-exact** as before (no
dependency on a default texture). Demo
[examples/94_normalmap.dh](../../examples/94_normalmap.dh): plate + sphere on
the left with, on the right without a normal map under a circling point light
— on the left the wave bumps move, on the right it stays smooth (verified by
screenshot; normal map `examples/assets/normal_waves.png` generated
procedurally).

With this the native 3D stack is **complete**: models, meshes incl. heightmap,
textures, normal maps, **PBR + analytic IBL (`LIGHT_ENV`) + real HDR cubemap
IBL (`LIGHT_ENV_HDR`)**, billboards, picking, lighting incl. fog and shadows,
camera modes (see "Real HDR cubemap IBL" above). Third-person collision is
gameplay logic (not an engine primitive).

## Shaders / post-processing (native)

GPU fragment shaders for full-screen effects (CRT, bloom, vignette, …) —
**native only** (raylib/OpenGL). Built-ins: `SHADER_LOAD(pfad$_oder_glsl$)`
(file OR GLSL source → SHADER handle/-1), `SHADER_SET`/`SHADER_SET2`/
`SHADER_SET3` (float/vec2/vec3 uniforms), `POSTFX(h)` (frame through the
shader; -1 = off).

**Render model:** if a post shader is active, `FLIP` renders the whole scene
(3D + 2D + scissor) not directly to the screen but into a `RenderTexture2D`;
then this texture is presented full-screen through `BeginShaderMode(shader)`
(Y flip because of the RT convention). The replay code is generic
(`fn render_scene<D: RaylibDraw>`), so it runs identically to the screen *or*
into the RenderTexture — `RaylibDrawHandle` and `RaylibTextureMode` both
implement `RaylibDraw`. Shader handles live in `Graphics.shaders`, the active
index in `post_shader_idx`.

Example shaders (GLSL 330):
[examples/assets/shaders/](../../examples/assets/shaders/) (`crt.fs`/`bloom.fs`/
`vignette.fs`), demo
[examples/86_postfx_shaders.dh](../../examples/86_postfx_shaders.dh)
(cycling OFF → CRT → BLOOM → VIGNETTE; CRT + bloom verified by screenshot).

## TTF fonts (`LOADFONT` / `SETFONT` / `TEXT_SPACING`)

Your own TrueType/OpenType fonts instead of only the built-in default font.
**Core built-ins, no `IMPORT` needed:**

- `LOADFONT(pfad$, groesse) -> FONT` — loads a TTF/OTF at the base size
  `groesse` (glyph resolution) and returns a **FONT handle (INTEGER)**.
- `SETFONT(font)` — activates the font for subsequent `TEXT` calls.
  `SETFONT(-1)` switches back to the default font.
- `TEXT_SPACING(px)` — letter spacing for TTF text (works natively via
  `DrawTextEx`).

`TEXT_SIZE` still scales the active font freely (natively raylib scales the
glyph texture loaded once). `TEXT_WIDTH` measures in the **active** font
(natively `MeasureTextEx`) — so centring/right alignment works with TTF too.
`TEXT_BOLD`/`TEXT_ITALIC` are natively a no-op (raylib has no synthetic
variant — load a bold/italic font file for that).

**Native implementation**
([graphics.rs](../../rust/drachenhauch_runtime/src/graphics.rs)):
`fonts: Vec<Font>` (raylib `load_font_ex`), `active_font` (-1 = default),
`text_spacing`. `Cmd::Text` now carries the font index + spacing; on replay a
valid index draws via `draw_text_ex(font, …)`, otherwise the default
`draw_text`.

**Bit identity does not apply** (renderer/font metrics differ) — like the rest
of the graphics, only functional. There is **no font asset in the repo**; demo
[examples/87_ttf_fonts.dh](../../examples/87_ttf_fonts.dh) looks for a system
font (`FILEEXISTS`) and otherwise falls back to the default font. Verified by
screenshot (size scaling, spacing, centred text via `TEXT_WIDTH`).

## Showcase demos

[examples/97_pbr_reactor.dh](../../examples/97_pbr_reactor.dh) — **"PBR
REACTOR"**, the full-screen showpiece of the **new** graphics pipeline:
chrome-shiny **PBR** spheres (one per FFT band, smoothed = gentle
"breathing") around a rotating chrome knot on a mirroring metal floor, with a
**scene change every 14 s** (RING → WAVE → HELIX), lit by **image-based
lighting** (`LIGHT_ENV`, animated sky colour) + a sun with **shadows**
(`SHADOW_ENABLE`) + two bass-pulsing point lights, plus **fog**, **bloom**
(`POSTFX`), glow sparks on the kick and a 2D FFT spectrum. Everything **truly
FFT-reactive** (`AUDIO_FFT`) to a stereo techno track. `SET_FULLSCREEN(TRUE)`,
the camera circles with a bass punch. Music: "Technological Messup" by
**josepharaoh99**, **CC0** — fetch it once with
`dhrt run examples/assets/download_techno.dh` (also runs without it, silently,
via a `FILEEXISTS` guard). Start: `dhrt run examples/97_pbr_reactor.dh`.

[examples/85_cybermatic_demo.dh](../../examples/85_cybermatic_demo.dh)
bundles into one 1280×720 frame what the native runtime can do —
**audio-reactive** (a real FFT of the running music via `AUDIO_FFT`) and with a
**scene change every 16 bars**: `TUNNEL` (wireframe rings flying towards you) →
`RING` (double ring + bass sphere + column, camera punch/shake) → `PLASMA`
(audio-reactive cube terrain). Plus a continuous 2D overlay: FFT spectrum
(`BOXES` bulk, top+bottom), glow sparks + cyber rain (two particle systems),
pulsating title, scrolling text, subtle beat flash. Start:
`dhrt run examples/85_cybermatic_demo.dh` (or F5 in the IDE).

The music asset (~15 MB, "Cybermatic pulse" by **Alexandr Zhelanov**,
CC-BY 4.0) is **not** in the repo (too big) — fetch it once with
`dhrt run examples/assets/download_cybermatic.dh`. The demo also runs without
it (silently, via a `FILEEXISTS` guard). Provenance/licence:
`examples/assets/CREDITS_cybermatic.txt`.

## Step 7: Standalone export (`dhrt --export` / IDE)

Bundle a Drachenhauch program into a standalone `.exe` that runs **without
Python** — ship games without a toolchain on the end user's machine.

**Principle (no recompile):** the compiled bytecode (`.dhc`) is **appended**
to a copy of `dhrt.exe`. At start `dhrt` detects the payload and runs it.
Appending data to the end of a PE `.exe` does not break it (the same principle
as PyInstaller onefile) — the PE loader ignores trailing bytes. Layout of the
**last 16 bytes** of the bundled exe:

```
[u64 Länge der .dhc-Bytes, little-endian][8 Byte Magic "DHRTPAY1"]
```

The `.dhc` bytes sit directly in front of this footer.

**Runtime side** ([main.rs](../../rust/drachenhauch_runtime/src/main.rs)):
`embedded_gbc()` reads its own exe (`current_exe()`), searches for the magic
footer **backwards** and extracts the bytecode.

> **Why backwards — and what that means for signing.** The footer does not
> have to stick to the end of the file. `signtool` (Windows) and `codesign`
> (macOS) also append their certificate block at the end; if the runtime only
> looked in the last 16 bytes, a signed game would no longer find itself and
> would behave like a bare `dhrt`. That is why the order is **export first,
> then sign** — the other way round does not work, because appending destroys
> any signature (measured: `Valid` becomes `NotSigned`). The search runs from
> the end of the file towards the front and checks every candidate (length
> field plausible? payload valid UTF-8?), so that a chance occurrence of the
> eight bytes in the signature block or in the JSON does not lead it astray.
> Secured by Rust `#[test]`s in `main.rs` and
> [tests/pruef/dhrt_export.dhtest](../../tests/pruef/dhrt_export.dhtest).
>
> **Only whoever holds the file can sign it.** A certificate of the
> Drachenhauch publisher does not help exported games — they are created on
> other people's computers. Whoever wants to ship their games signed needs a
> certificate of their own and signs the finished `.exe` themselves. If a payload is present (bundle mode), `dhrt` changes into the exe's directory
(so that relative asset paths are right when double-clicked from anywhere) and
runs the embedded bytecode. Without a payload it stays in dev mode
(`dhrt datei.dhc`). Both paths share `run_dhc_text(text, label)`.

**Export side** (`export_main` in
[main.rs](../../rust/drachenhauch_runtime/src/main.rs)): `dhrt` compiles the
source to `.dhc`, appends `<gbc><len><magic>` to a copy of its **own** exe and
writes `<out>/<name>.exe`. The `assets/` folder next to the source is copied
along (convention for `LOADIMAGE("assets/…")` & co.). Until 2026-09-21 there
was also the Python version `drachenhauch/export.py` (`dhrun.py --export`); it
has been removed.

**Call:**
```
dhrt --export examples/89_heightmap.dh [ausgabe-ordner]
```
Default output: `<quelle>_dist/`. In the **IDE** (`dhrt run ide/ide.dh`):
**Ctrl+F6** bundles the active file into `<name>_dist/` next to the source,
lean (see below); the export's output scrolls along at the bottom left (see
[ide.md](ide.md)).

**Lean export** (`--schlank`): the full runtime carries everything (graphics,
sound, PDF with embedded fonts, database, network, video, hardware) and is
about 33 MB in size. Next to it, two smaller ones live in `laufzeiten/`:
`dhrt-konsole` (data, network, mail, machine code; 16 MB) and `dhrt-spiel`
(graphics, sound, dialogs, physics, machine code; 21 MB). With `--schlank` the
export takes the smallest one that lacks none of the commands the program
calls -- even in a subroutine that never runs. **The runtimes themselves are
asked** (`dhrt-konsole --fehlende name …`: every name is called with
deliberately nonsensical arguments; a command that exists fails on them, a
missing one says that it is missing), not a list -- a list would be silently
wrong with the next new command, and the program would only break at the
customer's. The export says which one it took (`Schlank: dhrt-konsole (16,1 MB
statt 32,9 MB)`), and if none fits, why (`dhrt-konsole fehlt PDF_NEW`). The
machine code stays in both: without it programs would compute up to a hundred
times slower. The small ones are built with `python rust/build_runtime.py
--laufzeiten`; the installer and the packages for macOS and Linux ship them.
Checked in
[tests/pruef/export_schlank.dhtest](../../tests/pruef/export_schlank.dhtest).

```
dhrt --export werkzeug.dh --schlank
```

Verified: `89_heightmap.dh` exported (~3.7 MB exe), the exe started **without
arguments from a different directory** loads the copied
`assets/heightmap.png` and renders the terrain (screenshot).

**Asset bundling:** the export copies (a) an `assets/` folder next to the
source as before AND (b) **every file referenced in the source as a string
literal** that exists relative to the source — even via `../` (e.g. a game in
`code/` with `LOADIMAGE("../assets/sprites/x.png")` and assets in
`../assets/`).

**Databases are not assets.** The literal search cannot know *what* a file
name is there for: with `LOADIMAGE("held.png")` copying it along is right,
with `DB_OPEN("spiel.db")` it is wrong — the program creates that file itself,
and it contains the data of whoever does the export. The export therefore
skips files that are recognizable **by their content** as an SQLite database
(the first 16 bytes read `SQLite format 3\0`) — by content and not by
extension, because a database can also be called `spielstand.dat` and a
`notizen.db` does not have to be one. It says during the export which file it
skipped.

Whoever really wants to ship a **prepared** database (a dictionary, a level
collection) adds it with `--mit-daten`:

```
dhrt --export spiel.dh --mit-daten
```

**The output folder is cleared.** Without that, everything from the previous
run that no longer belongs would stay behind — the `.exe` is overwritten, a
database next to it is not, and it would silently go out with it. Only the
`<quelle>_dist/` **chosen by the export itself** is cleared, and only if it
contains an exe with our payload magic: the proof that the folder comes from
an earlier export. An output folder given by hand stays untouched (with a
hint) — anything could be in there.

**By name instead of as a number.** The export lists the files it copied
along. Before, it only said "1 referenzierte Asset-Datei(en) mitkopiert" (1
referenced asset file(s) copied) — a foreign database in the bundle did not
stand out that way.
Such paths are placed into the bundle with the `../` stripped
(`assets/sprites/x.png`). At run time `resolve_asset_path` (builtins.rs) finds
the bundled copy: if the original path does not exist, leading `../` are
stripped and the search is repeated — applies to LOADIMAGE/LOADSOUND/PLAYMUSIC/
SHADER_LOAD/LOADMODEL/TILED_LOAD/LOAD_ASSETS/ATLAS_LOAD/FILEEXISTS/
LIGHT_ENV_HDR. In dev mode (the original exists) nothing changes. Absolute
paths are not bundled.

**Limits:** only string **literals** are recognized (paths put together at run
time are not — then copy the assets into the output folder manually). The
`.dhc` is embedded uncompressed (JSON); the exe size equals `dhrt` + bytecode.
Cross-compiling is not planned — the export bundles the `dhrt` of the current
platform.
