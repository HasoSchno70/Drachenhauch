# Drachenhauch auf dem Raspberry Pi

Drachenhauch läuft auf dem Raspberry Pi mit der ganzen Entwicklungsumgebung,
mit Spielen und 3D — und es schaltet die Pins des Pi selbst
([`gpio`, `i2c`, `spi`](module-pins.md)).

## Welcher Pi?

- **Raspberry Pi OS in 64 Bit** (Bookworm oder neuer). Ein 32-Bit-System
  geht nicht — gebaut wird nur für `aarch64`.
- **Pi 4 und Pi 5**: alles, auch Fenster, 3D und die IDE.
- **Pi 3 und Zero 2 W**: Konsolenprogramme und die Pins. Ihr Grafikchip kann
  nur OpenGL ES 2.0, Drachenhauch braucht 3.0 — ein Programm mit Fenster
  meldet „Kein Fenster möglich" statt abzustürzen.

## Installieren

Das Paket heißt `Drachenhauch-IDE-<Fassung>-linux-aarch64.tar.gz`. Entpacken
und für den eigenen Benutzer einrichten — ohne `sudo`:

```bash
tar xzf Drachenhauch-IDE-*-linux-aarch64.tar.gz
sh Drachenhauch-IDE-*/install.sh
```

Danach steht die IDE im Startmenü unter *Entwicklung*, `drachenhauch`
startet sie im Terminal und `dhrt` führt Programme aus. Die Beispiele landen
beim ersten Start in `~/Drachenhauch/examples`. `install.sh --entfernen`
räumt wieder ab.

## Grafik

Der Pi kann kein OpenGL 3.3, das Drachenhauch auf Windows, macOS und
gewöhnlichem Linux benutzt, wohl aber **OpenGL ES 3.1**. Das Paket für den Pi
ist darum mit OpenGL ES gebaut (`dhrt --version` sagt „grafik (OpenGL ES)").
Für Programme ändert sich nichts: auch eigene Shader mit `#version 330`
laufen, dhrt schreibt sie beim Laden auf `#version 300 es` um — wie im
Browser.

Der Pi ist langsamer als ein Rechner. Was hilft:

- `POSTFX(shader, 0.5)` rechnet einen Vollbild-Shader in halber Auflösung —
  auf einem Rechner kaum zu sehen, auf dem Pi oft der Unterschied zwischen
  ruckeln und flüssig.
- `SETFPS(30)` statt 60, wenn das Spiel es erlaubt.
- Schatten mit kleinerer Auflösung: `SHADOW_ENABLE(1024)`.

## Pins

GPIO, I2C und SPI gehen ohne Arduino dazwischen — siehe
[Module `gpio`, `i2c` und `spi`](module-pins.md). I2C und SPI muss man
einmal einschalten (`sudo raspi-config` → *Interface Options*).

## Wenn ein Fenster nicht aufgeht

- **Über SSH ohne Bildschirm** gibt es keinen Bildschirm, und ein Programm
  mit Fenster meldet das. Konsolenprogramme und Pins gehen trotzdem; ein
  Fensterprogramm startet man am Pi selbst oder mit `xvfb-run` ohne Anzeige.
- **Was der Grafiktreiber sagt**, zeigt `DHRT_RAYLIB_LOG=info`:

```bash
DHRT_RAYLIB_LOG=info dhrt run spiel.dh
```

  Wichtig sind die Zeilen `Renderer:` und `Version:` — auf einem Pi 5 etwa
  „V3D 7.1" und „OpenGL ES 3.1 Mesa".

## Selbst bauen

Auf einem Pi mit 64-Bit-System (oder einem anderen ARM-Rechner) aus dem
Quelltext, mit Rust und den Bibliotheken für Fenster, Ton und OpenGL ES:

```bash
sudo apt install cmake clang libasound2-dev libgtk-3-dev libudev-dev pkg-config \
  libx11-dev libxrandr-dev libxi-dev libxinerama-dev libxcursor-dev \
  libgles-dev libegl-dev libwayland-dev libxkbcommon-dev wayland-protocols
cd rust/drachenhauch_runtime
CFLAGS=-DMAX_CHAR_PRESSED_QUEUE=256 cargo build --release --features "gles dialogs video db net http pdf physik jit ffi serial midi"
```

Das Feature `gles` ersetzt `graphics`; die Pins brauchen keines.

## Geprüft

Das Paket baut die CI auf einem ARM-Rechner von GitHub und fährt dort unter
`xvfb` mit Mesa (llvmpipe, OpenGL ES 3.2) Licht, Schatten, PBR mit
HDR-Umgebung, Instancing, Shader, Post-FX, Render-Ziele und Programme mit
Fenster (`tests/pruef/grafik/gles.dhtest` und weitere). Die Pins prüft sie gegen
die Simulatoren des Kernels. Auf einem echten Raspberry Pi ist Drachenhauch
**ungeprüft** — sein Grafiktreiber (V3D) ist ein anderer als llvmpipe.
