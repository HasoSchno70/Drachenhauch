# Web playground (dhrt → WebAssembly)


> **As of 2026-08-03: the web build runs.** Console AND graphics programs are
> compiled and executed in the browser (verified: circles, shapes and text on
> the canvas, `dhrt.wasm` 8.8 MB). The blocking render loop, formerly named as
> the core obstacle, is no problem with ASYNCIFY -- a `FOR` loop with `FLIP()`
> runs through and dutifully prints its `PRINT` line at the end.
>
> Four things were needed for this (all in the repo):
> 1. **`dialogs` split off from the `graphics` feature.** `rfd` (native file
>    dialogs) pulls in js-sys/wasm-bindgen in a version with which cpal's
>    WebAudio host no longer compiles -- and in the browser there are no OS
>    dialogs anyway.
> 2. **wasm-bindgen pinned for emscripten** (`=0.2.100`). Kira pulls in cpal
>    with the feature `wasm-bindgen` for EVERY `wasm32` target, including
>    emscripten, where the WebAudio host is not needed at all (a target filter
>    mistake in Kira's manifest). So it gets compiled along and has to compile.
> 3. **No streaming on wasm.** Kira's `sound::streaming` is configured away
>    there, and `StaticSoundData::from_file` does not exist. On the web, music
>    runs through the static variant (loaded fully once), sounds through
>    `from_cursor` with bytes read by ourselves.
> 4. **Set the canvas size ourselves.** Measured: after `SCREEN(480,320)` the
>    runtime dutifully reports 480x320, but the BUFFER of the `<canvas>`
>    stayed 1x1 -- stretched by CSS, that became a coloured area. `graphics.rs`
>    now calls `emscripten_set_canvas_element_size` and keeps adjusting the
>    size during the first eight frames (raylib sets it once more by itself
>    after that).
>
> **Assets come along** (2026-08-03). If there is an `assets/` folder next to
> the `.dh`, `build_wasm.py` packs it into a `dhrt.data` and mounts it into
> the virtual file system via `--preload-file` -- programs then load their
> images, fonts, shaders and music under exactly the same path as on the
> desktop. Measured in the browser: `FILESIZE("assets/font.ttf")` returns
> 168644, `LOADFONT` on it returns a valid handle, and the text appears in
> the embedded font. **`--preload-file` and `--embed-file` exclude each
> other** -- as soon as assets are included, the source is preloaded too.
> `dhrt.data` must then be shipped next to `dhrt.js`/`dhrt.wasm`, otherwise
> nothing starts at all.

## Known limits in the browser

All measured, not guessed:

- **The canvas turns black after the program ends.** WebGL discards the
  drawing buffer after presenting it (`preserveDrawingBuffer` is not set). For
  the same reason, `SAVESCREENSHOT` returns an empty image in the browser.
  Affects only the aftermath, not the running program -- while it runs,
  everything is visible.
- **Sound is audible** (since 2026-08-04) -- but only after the first click.
  cpal no longer has an emscripten host (its WebAudio host depends on
  wasm-bindgen JS glue that emscripten does not provide), so formerly every
  program with sound ran into `NoDefaultOutputDevice` and died. dhrt therefore
  brings its **own Kira backend** for the browser (`src/web_audio.rs`): Kira's
  `Backend` contract hands us the `Renderer`, we take the finished mix and
  push it into a queue of **OpenAL** buffers -- emscripten maps OpenAL onto
  WebAudio internally.

  **The queue clocks itself:** each frame refills only as much as WebAudio
  has played out. In the steady state that is exactly real time, without
  reading a clock anywhere. Measured in the browser: after a run the wall
  clock stands at 8.35 s and `AUDIO_MUSIC_POSITION()` at 8.50 s -- the music
  clock follows real time to within 2 %, and it can only do that if the
  buffers are actually being played. **Listened to and found clean**
  (04.08.2026) -- measuring alone would not have uncovered the stutter from
  the rate mix-up below.

  **Autoplay block:** an `AudioContext` starts suspended until the user has
  touched the page once. Until then WebAudio consumes nothing, the queue stays
  full -- and because the clock hangs on it, playback stands still too. A
  program whose flow depends on `AUDIO_MUSIC_POSITION` therefore waits for
  the first click. The playground points this out.

  **Compute with THE rate the browser runs at.** Measured: the
  `AudioContext` ran at **48000 Hz** here, our buffers were registered at
  44100. WebAudio then resamples EACH buffer on its own -- and because such a
  conversion lacks the neighbouring buffers, a jump appears at every seam: it
  stutters at the buffer rate, although nothing runs dry. The rate therefore
  comes from `alcGetIntegerv(ALC_FREQUENCY)` and is passed on to Kira.

  **`-lopenal` is mandatory.** The `-lal` that raylib brings along is merely
  an empty archive; the JS implementation lives in `libopenal.js` and only
  comes in via `-lopenal`. Without the flag everything compiles and links, and
  the browser only aborts at start-up: *"alcOpenDevice: function import
  requires a callable"*.

  **Only `FLIP` refills.** A console program without a frame loop gets no
  clock in the browser -- it stays silent there.
- ~~Custom shaders in desktop GLSL do not compile.~~ **Done since
  2026-08-03: 3D and post effects run.** The web build now uses **WebGL 2**
  instead of WebGL 1 (`opengl_es_30` + `MIN/MAX_WEBGL_VERSION=2`). Its
  language GLSL ES 3.00 is the same as desktop GLSL 330 except for the
  version line and the precision qualifiers -- so instead of rewriting
  everything down to the older language (and losing IBL in the process), the
  target is raised. `fuer_ziel_uebersetzen` in `graphics.rs` only swaps the
  header when loading; the body stays ONE. That also applies to
  `SHADER_LOAD`, so a desktop shader runs unchanged in the browser.
- **No non-power-of-two textures with repeat**
  (`GL: NPOT textures extension not found`) -- a warning, not an error.

## The demo in the browser (as of 2026-08-04)

`dhdemo/` **runs completely** -- with picture and sound, as on the desktop.
Checked in the browser:

| | |
|---|---|
| Schedule driven by `AUDIO_MUSIC_POSITION` | runs in real time |
| Tracker module (261 KB `.mod`) | is played, **audible** (after the first click) |
| Spectrum bars, sine scroller, logo | complete |
| Scene jumps with the number keys | work |
| 2D scenes (title, bulk lines, physics logo) | complete |
| Post effect (plasma, tunnel, glow) | complete |
| 3D: cube field with `MODEL_INSTANCED` (1600 cubes) | complete |
| 3D: `MODEL_PBR` + `LIGHT_ENV_HDR` + `SKYBOX` + shadows | complete |

So the demo runs **completely** in the browser.

### The finding that blocked 3D in the browser

The shader was not to blame. After the switch to WebGL 2 all shaders compiled
and linked without errors -- **and still every `MODEL_LIT` model was
invisible**. Measured (draw calls hooked in the browser and `getError()`
queried): `drawArrays` returned `INVALID_OPERATION`.

Cause: without an HDR environment, the two `samplerCube` uniforms stayed at
their default -- texture unit 0, where `texture0` already sits as a
`sampler2D`. **Two different sampler types on one unit are an error in
WebGL 2**, and the draw call is discarded. Desktop drivers are lenient there,
which is why it only shows up in the browser. The units are now always set.

The lesson from this is general: **a shader that compiled without errors says
nothing about whether anything is drawn.** When geometry disappears,
`getError()` after the draw call is the measurement that holds.

### Two build traps that can cost hours

* **cargo does not track `EMCC_CFLAGS`.** If you only change a linker flag,
  cargo sees unchanged sources, skips linking -- and the old `.wasm` stays
  in place. The build reports success, the flag is not in it. That is exactly
  how it looked when `STACK_SIZE` "did not help": it was never in the binary.
  `build_wasm.py` now deletes the output itself as soon as the flags change
  (`erzwinge_relink`).
* **The "done" status in the playground lies for graphics programs.**
  ASYNCIFY lets `callMain` return as early as the first frame; the program
  keeps running afterwards. Whoever reads "the program has ended" from that
  looks for bugs in the wrong place.

Run Drachenhauch programs in the browser — the **native runtime `dhrt`**
(Rust/raylib) as WebAssembly via emscripten, with graphics in the `<canvas>`
and console output beside it.

> **Status: builds & runs, graphics included (verified 2026-06-10).** With the
> toolchain installed (emscripten 6.0.0 + Rust target
> `wasm32-unknown-emscripten`), `rust/build_wasm.py` produces a working
> `web/dhrt.js` + `web/dhrt.wasm`. **Console AND graphics programs run in the
> browser** — animated demos in the `<canvas>` without freezing the tab
> (verified in the browser via preview: moving sprite + running frame
> counter). dhrt compiles the embedded **source** itself inside the WASM (no
> Pyodide). The build artefacts (`dhrt.js`/`.wasm`/`program.dh`/`.dhc`) are
> gitignored. **Shareable links:** “Share link” packs the source into the URL
> hash — whoever opens the link sees and starts exactly this program.

> **The Windows toolchain is wired up automatically.** `build_wasm.py`
> (`setup_emscripten_env`) finds an emsdk (env `EMSDK` or `%USERPROFILE%\emsdk`)
> and sets by itself: `CC/CXX/AR/Linker` to the `.exe` variants
> (`emcc.exe`/`em++.exe`/`emar.exe` — otherwise cc-rs smuggles `cmd /c
> emcc.bat` into the CFLAGS), `BINDGEN_EXTRA_CLANG_ARGS` with clang builtin +
> sysroot includes (otherwise bindgen does not find `stdarg.h`),
> `CMAKE_GENERATOR=Ninja` + cmake/ninja from the VS BuildTools on the PATH.
> That way `python rust/build_wasm.py datei.dh` runs without manual env
> setup. To install emsdk: `git clone …/emsdk`,
> `python emsdk/emsdk.py install latest` + `… activate latest`,
> `rustup target add wasm32-unknown-emscripten`.

> **No Pyodide needed any more (since the front-end port).** Formerly the
> `.dh` had to be precompiled to `.dhc` in Python before the browser could run
> it — live editing in the browser would have needed Pyodide. Now `dhrt`
> contains the complete front-end chain (preprocess → lexer → parser →
> compiler, all stages in Rust), so the WASM runtime compiles the **source
> directly in the browser**. The build embeds `program.dh` (source); `main.rs`
> reads `/program.dh` first and compiles it itself, with `/program.dhc` as a
> fallback. This makes a real live playground (type source → compile → run)
> possible purely in Rust WASM — without Python/Pyodide in the browser.

## Components

| File | Role |
|---|---|
| `rust/build_wasm.py` | `.dh` → `web/program.dh` (source, compiled in the browser) + `web/program.dhc` (fallback), then `cargo`+emscripten build → `web/dhrt.{js,wasm}` |
| `rust/drachenhauch_runtime/src/main.rs` | `#[cfg(target_os = "emscripten")]` branch compiles and runs `/program.dh` (fallback `/program.dhc`) from the virtual FS |
| `web/index.html` | live editor (`<textarea id="src">`) + `<canvas id="canvas">` + output area + run/share button |
| `web/playground.js` | live playground: editor→`sessionStorage`, reload for a fresh runtime, writes the source to `/program.dh`, `callMain()`; stdout→div (Module.print + console.log fallback). The one-shot run flag `dh_run` makes hanging programs recoverable by reload. **Shareable links:** source as base64url in the URL hash (`#gb=…`); an opened link loads and starts the program. |

## Building

Prerequisites:

1. **emscripten** (`emcc` on the PATH) — <https://emscripten.org>
2. **Rust target:** `rustup target add wasm32-unknown-emscripten`

Then:

```bash
python rust/build_wasm.py examples/01_hello.dh
```

`build_wasm.py` only needs a Python 3 with the standard library (no venv).
The script is tolerant: if the toolchain is missing, it only creates
`web/program.dh` and prints the manual build command. With a complete
toolchain, `web/dhrt.js` + `web/dhrt.wasm` are produced.

## Starting

WASM needs real HTTP (no `file://`):

```bash
py -m http.server -d web 8000
# -> http://localhost:8000
```

Click Run → `Module.callMain()` starts `dhrt`, which executes `/program.dhc`.

## Architecture note: the render loop (solved)

The VM drives its frame/render loop **blocking**: the DH program owns the
loop (`WHILE NOT QUITREQUESTED() … FLIP() … WEND`), and the VM only returns
at the end of the program. In the browser, however, the main thread must not
block, otherwise the tab hangs.

**Solution (implemented): ASYNCIFY + one yield per frame.** `build_wasm.py`
sets `-s ASYNCIFY`; that alone is not enough, because without a yield point
the blocking loop never hands back to the browser's event loop. That is why
**`graphics.rs::flip()` calls `emscripten_sleep(0)` under
`cfg(target_os="emscripten")`** directly after presenting (`EndDrawing`).
ASYNCIFY unwinds the entire Rust stack, hands control to the browser (the
canvas is composited, input events are delivered) and resumes exactly there
on the next tick. With that, the unchanged DH render loop cooperates with the
browser — **no rewrite to `emscripten_set_main_loop` needed**, the VM and
coroutine logic remain untouched.

> The theoretically cleaner way (`emscripten_set_main_loop`, the VM returns
> each frame) would be a big intervention in `vm.rs`; the ASYNCIFY yield is
> ~3 lines and gives the same result. The feared conflict between ASYNCIFY
> and `-fwasm-exceptions` turned out to be a mere linker warning — the build
> works.

## Status & limits (verified 2026-06-10)

- **Console programs: run in the browser ✅.** Type source → *Run* → correct
  output. dhrt compiles the `.dh` source **itself inside the WASM** (front-end
  port) — no Pyodide, no precompiled `.dhc`.
- **Graphics programs: run in the browser ✅.** The render loop yields each
  frame (ASYNCIFY yield in `flip()`, see above) → animated demos in the
  canvas, the tab stays responsive (checked in the browser via preview:
  moving object + counting frame counter, no freezing). The harness stays
  **hang-safe** (one-shot run flag `dh_run`) in case a program does run in a
  tight busy loop.
- **Shareable links ✅.** *Share link* base64url-encodes the source into the
  URL hash (`#gb=…`, pure client JS, no backend). An opened link loads the
  source into the editor **and starts it directly** (fresh runtime). *Run*
  keeps the hash up to date, so the URL can be shared at any time.
- **Hardware/network modules** (`db/net/http/serial/usb/wifi/bt`) are not
  available in the web build (only `--features graphics`).
- **Audio:** audible through a custom Kira backend that pushes the finished
  mix into OpenAL buffers (emscripten maps OpenAL onto WebAudio) — details
  above under “Known limits”.

## Next steps

1. ~~Console in the browser~~ ✅ done (live editor, verified).
2. ~~Graphics in the browser~~ ✅ done (ASYNCIFY yield in `flip()`, verified).
3. ~~Shareable links~~ ✅ done (source in the URL hash).
4. ~~Shipping assets~~ ✅ done (`--preload-file`, `dhrt.data`, verified).
5. ~~Audio in the browser~~ ✅ done — silent at first via Kira's
   `MockBackend`, **audible** since 2026-08-04 through a custom backend with
   OpenAL output.
6. ~~Shaders for the web~~ ✅ done — via WebGL 2 instead of a port to
   GLSL ES 1.00 (3D, IBL, skybox, shadows and post effects verified).
7. ~~Example gallery~~ ✅ done (`web/beispiele.js`, six programs without file
   access).
8. ~~Touch input~~ ✅ verified: `TOUCH_COUNT`/`TOUCH_X`/`TOUCH_Y` report the
   right spot in the browser (synthetic touch at 40 %/60 % of a 320×200
   canvas → 128.3/119.5). The **gestures** (`GESTURE$`) stayed empty in the
   test — whether that is due to the synthetic events or a real gap was not
   resolved.
9. **Open:** a real device to cross-check against (phone browser). Sound
   itself is ticked off — listened to on 04.08.2026 and found clean.
