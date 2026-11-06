# Drachenhauch on the Raspberry Pi

Drachenhauch runs on the Raspberry Pi with the whole development
environment, with games and 3D — and it switches the Pi's own pins
([`gpio`, `i2c`, `spi`](module-pins.md)).

## Which Pi?

- **Raspberry Pi OS in 64 bit** (Bookworm or newer). A 32-bit system does
  not work — the build is for `aarch64` only.
- **Pi 4 and Pi 5**: everything, including windows, 3D and the IDE.
- **Pi 3 and Zero 2 W**: console programs and the pins. Their graphics chip
  only does OpenGL ES 2.0, Drachenhauch needs 3.0 — a program with a window
  reports "Kein Fenster möglich" (no window possible) instead of crashing.

## Installing

The package is called `Drachenhauch-IDE-<version>-linux-aarch64.tar.gz`.
Unpack it and set it up for your own user — without `sudo`:

```bash
tar xzf Drachenhauch-IDE-*-linux-aarch64.tar.gz
sh Drachenhauch-IDE-*/install.sh
```

After that the IDE is in the start menu under *Development*, `drachenhauch`
starts it in the terminal and `dhrt` runs programs. The examples land in
`~/Drachenhauch/examples` on the first start. `install.sh --entfernen`
removes everything again.

## Graphics

The Pi cannot do OpenGL 3.3, which Drachenhauch uses on Windows, macOS and
ordinary Linux, but it can do **OpenGL ES 3.1**. The package for the Pi is
therefore built with OpenGL ES (`dhrt --version` says "grafik (OpenGL ES)").
Nothing changes for programs: even your own shaders with `#version 330` run,
dhrt rewrites them to `#version 300 es` when loading — as in the browser.

The Pi is slower than a PC. What helps:

- `POSTFX(shader, 0.5)` computes a full-screen shader at half resolution —
  hardly visible on a PC, on the Pi often the difference between stuttering
  and smooth.
- `SETFPS(30)` instead of 60, if the game allows it.
- Shadows with a smaller resolution: `SHADOW_ENABLE(1024)`.

## Pins

GPIO, I2C and SPI work with no Arduino in between — see
[Modules `gpio`, `i2c` and `spi`](module-pins.md). I2C and SPI have to be
switched on once (`sudo raspi-config` → *Interface Options*).

## When a window does not open

- **Over SSH without a screen** there is no screen, and a program with a
  window says so. Console programs and pins still work; start a window
  program on the Pi itself or with `xvfb-run` without a display.
- **What the graphics driver says** is shown by `DHRT_RAYLIB_LOG=info`:

```bash
DHRT_RAYLIB_LOG=info dhrt run spiel.dh
```

  The important lines are `Renderer:` and `Version:` — on a Pi 5 for example
  "V3D 7.1" and "OpenGL ES 3.1 Mesa".

## Building it yourself

On a Pi with a 64-bit system (or another ARM computer) from source, with
Rust and the libraries for windows, sound and OpenGL ES:

```bash
sudo apt install cmake clang libasound2-dev libgtk-3-dev libudev-dev pkg-config \
  libx11-dev libxrandr-dev libxi-dev libxinerama-dev libxcursor-dev \
  libgles-dev libegl-dev libwayland-dev libxkbcommon-dev wayland-protocols
cd rust/drachenhauch_runtime
CFLAGS=-DMAX_CHAR_PRESSED_QUEUE=256 cargo build --release --features "gles dialogs video db net http pdf physik jit ffi serial midi"
```

The feature `gles` replaces `graphics`; the pins need none.

## Tested

The CI builds the package on a GitHub ARM machine and runs light, shadows,
PBR with an HDR environment, instancing, shaders, post-FX, render targets
and programs with a window there under `xvfb` with Mesa (llvmpipe, OpenGL ES
3.2) (`tests/pruef/gles.dhtest` and more). It tests the pins against the
kernel's simulators. On a real Raspberry Pi Drachenhauch is **untested** —
its graphics driver (V3D) is a different one from llvmpipe.
