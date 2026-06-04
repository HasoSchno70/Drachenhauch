# Web-Playground (gbrt → WebAssembly)

GameBasic-Programme im Browser laufen lassen — die **native Runtime `gbrt`**
(Rust/raylib) als WebAssembly via emscripten, mit Grafik im `<canvas>` und
Konsolen-Ausgabe daneben.

> **Status: experimentell / Gerüst.** Der Code-Pfad ist vorbereitet
> (cfg-gegateter WASM-Einstieg in `main.rs`, Build-Skript, Web-Harness), aber
> ein lauffähiges `gbrt.wasm` **muss lokal mit voller Toolchain gebaut werden**
> (emscripten + Rust-wasm-Target) und ist hier nicht beigelegt/verifiziert.
> Siehe **Grenzen** unten.

> **Kein Pyodide mehr nötig (seit Front-End-Port).** Früher musste die `.gb`
> in Python zu `.gbc` vorkompiliert werden, bevor sie der Browser ausführen
> konnte — Live-Editieren im Browser hätte Pyodide gebraucht. Jetzt enthält
> `gbrt` die komplette Front-End-Kette (Preprocess → Lexer → Parser → Compiler,
> alle Stufen in Rust), also kompiliert die WASM-Runtime die **Quelle direkt im
> Browser**. Der Build bettet `program.gb` (Quelle) ein; `main.rs` liest
> `/program.gb` zuerst und kompiliert es selbst, mit `/program.gbc` als Fallback.
> Damit ist ein echtes Live-Playground (Quelle tippen → kompilieren → laufen)
> rein in Rust-WASM möglich — ohne Python/Pyodide im Browser.

## Bestandteile

| Datei | Rolle |
|---|---|
| `rust/build_wasm.py` | `.gb` → `web/program.gb` (Quelle, im Browser kompiliert) + `web/program.gbc` (Fallback), dann `cargo`+emscripten-Build → `web/gbrt.{js,wasm}` |
| `rust/gb_runtime/src/main.rs` | `#[cfg(target_os = "emscripten")]`-Zweig kompiliert+führt `/program.gb` aus (Fallback `/program.gbc`) aus dem virtuellen FS |
| `web/index.html` | Seite mit `<canvas id="canvas">` + Output-Bereich + Run-Button |
| `web/playground.js` | emscripten-`Module`-Konfig: stdout→Div, Canvas binden, `callMain()` |

## Bauen

Voraussetzungen:

1. **emscripten** (`emcc` im PATH) — <https://emscripten.org>
2. **Rust-Target:** `rustup target add wasm32-unknown-emscripten`

Dann:

```bash
.venv\Scripts\python.exe rust\build_wasm.py examples\01_hello.gb
```

Das Skript ist tolerant: fehlt die Toolchain, kompiliert es nur `program.gbc`
und druckt den manuellen Build-Befehl. Mit vollständiger Toolchain entstehen
`web/gbrt.js` + `web/gbrt.wasm`.

## Starten

WASM braucht echtes HTTP (kein `file://`):

```bash
py -m http.server -d web 8000
# -> http://localhost:8000
```

Run klicken → `Module.callMain()` startet `gbrt`, das `/program.gbc` ausführt.

## Architektur-Hinweis: der Render-Loop

Die VM treibt ihren Frame-/Render-Loop **blockierend** in `vm.run()` (erst am
Programmende kehrt sie zurück, stdout wird gepuffert). Im Browser darf der
Main-Thread aber nicht blockieren. Zwei Wege:

- **ASYNCIFY** (gewählt): emscripten entrollt den Stack an Blockier-Punkten —
  der bestehende blockierende Loop läuft unverändert, nur langsamer und mit
  größerem `.wasm`. `build_wasm.py` setzt `-s ASYNCIFY`.
- **Umbau auf `emscripten_set_main_loop`** (perspektivisch schneller): die VM
  müsste pro Frame zurückkehren statt selbst zu loopen — größerer Eingriff in
  `vm.rs`/`graphics.rs`. Nicht umgesetzt.

## Grenzen (Stand jetzt)

- **Nicht in dieser Umgebung gebaut/verifiziert** — raylib-rs’ Web-Support ist
  experimentell; die emscripten-Flags (`USE_GLFW=3`, `ASYNCIFY`, …) können je
  nach raylib-rs-Version Anpassung brauchen.
- **Compiler bleibt Python:** `.gb → .gbc` macht die Python-Toolchain. Der
  Playground führt eine **vorab kompilierte** `.gbc` aus. Ein *Live*-Editor
  („tippen → ausführen“) im Browser bräuchte zusätzlich den Compiler im
  Browser — am ehesten via **Pyodide** (CPython-WASM) parallel zur gbrt-WASM.
  Das ist der nächste Ausbauschritt.
- **Hardware-/Netz-Module** (`db/net/http/serial/usb/wifi/bt`) sind im
  Web-Build nicht sinnvoll/verfügbar (nur `--features graphics`).
- **Audio/Threads:** Web-Audio + Coroutinen-Threads brauchen ggf. zusätzliche
  emscripten-Flags (`-s AUDIO_WORKLET`, `-pthread`) — offen.

## Nächste Schritte

1. Toolchain lokal aufsetzen, `build_wasm.py` durchlaufen lassen, Flags
   iterativ fixen bis `01_hello` (Konsole) im Browser läuft.
2. Eine Grafik-Demo (z. B. `examples/03_*`) mit ASYNCIFY testen.
3. Pyodide für Live-Kompilierung ergänzen (Editor-Textfeld → `.gbc` im Browser
   → `FS.writeFile('/program.gbc')` → `callMain()`).
