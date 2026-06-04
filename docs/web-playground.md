# Web-Playground (gbrt → WebAssembly)

GameBasic-Programme im Browser laufen lassen — die **native Runtime `gbrt`**
(Rust/raylib) als WebAssembly via emscripten, mit Grafik im `<canvas>` und
Konsolen-Ausgabe daneben.

> **Status: baut & läuft (verifiziert 2026-06-04).** Mit installierter Toolchain
> (emscripten 6.0.0 + Rust-Target `wasm32-unknown-emscripten`) erzeugt
> `rust/build_wasm.py` ein lauffähiges `web/gbrt.js` + `web/gbrt.wasm`. Verifiziert
> mit `examples/01_hello.gb`: unter Node (`node web/gbrt.js`) **bit-identische
> Ausgabe zum Python-Tree-Walker** — und zwar aus der eingebetteten **Quelle**
> (gbrt kompiliert im WASM selbst, kein Pyodide). Die Build-Artefakte
> (`gbrt.js`/`.wasm`/`program.gb`/`.gbc`) sind gitignored, nicht eingecheckt.
> Grafik-Demos (Render-Loop) brauchen den Browser-Canvas — siehe **Grenzen**.

> **Windows-Toolchain wird automatisch verdrahtet.** `build_wasm.py`
> (`setup_emscripten_env`) findet ein emsdk (Env `EMSDK` oder `%USERPROFILE%\emsdk`)
> und setzt selbst: `CC/CXX/AR/Linker` auf die `.exe`-Varianten
> (`emcc.exe`/`em++.exe`/`emar.exe` — sonst schmuggelt cc-rs `cmd /c emcc.bat`
> in die CFLAGS), `BINDGEN_EXTRA_CLANG_ARGS` mit clang-Builtin- + Sysroot-Include
> (sonst findet bindgen `stdarg.h` nicht), `CMAKE_GENERATOR=Ninja` + cmake/ninja
> aus den VS-BuildTools auf PATH. So läuft `python rust/build_wasm.py datei.gb`
> ohne manuelles Env-Setup. emsdk installieren: `git clone …/emsdk`,
> `python emsdk/emsdk.py install latest` + `… activate latest`,
> `rustup target add wasm32-unknown-emscripten`.

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
| `web/index.html` | Live-Editor (`<textarea id="src">`) + `<canvas id="canvas">` + Output-Bereich + Run-Button |
| `web/playground.js` | Live-Playground: Editor→`sessionStorage`, Reload für frische Runtime, schreibt die Quelle nach `/program.gb`, `callMain()`; stdout→Div (Module.print + console.log-Fallback). Einmal-Run-Flag `gb_run` macht hängende Programme reload-erholbar. |

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

- **ASYNCIFY** (gesetzt): `build_wasm.py` setzt `-s ASYNCIFY`. **Reicht für
  Grafik aber NICHT aus**, weil raylib im `PLATFORM_WEB`-Modus auf
  `emscripten_set_main_loop` ausgelegt ist und in einem blockierenden Loop
  nicht ans Browser-Event-Loop yieldet — ein `WHILE TRUE … FLIP()` blockiert
  daher den Main-Thread (Tab hängt). Zusätzlich kollidiert ASYNCIFY mit dem
  von rustc emittierten `-fwasm-exceptions` (Linker-Warnung).
- **Umbau auf `emscripten_set_main_loop`** (nötig für Grafik im Browser): die VM
  müsste pro Frame zurückkehren statt selbst zu loopen — größerer Eingriff in
  `vm.rs`/`graphics.rs` (cfg `target_os="emscripten"`). **Nicht umgesetzt.**

## Stand & Grenzen (verifiziert 2026-06-04)

- **Konsolen-Programme: laufen im Browser ✅.** Im echten Browser geprüft (über
  den Live-Editor): Quelle tippen → *Ausführen* → korrekte Ausgabe. gbrt
  kompiliert die `.gb`-Quelle **selbst im WASM** (Front-End-Port) — **kein
  Pyodide**, kein vorab kompiliertes `.gbc` nötig.
- **Grafik-Programme: hängen aktuell ⚠️.** Der blockierende Render-Loop yieldet
  nicht (siehe oben) → der Tab friert ein. Behebung = `emscripten_set_main_loop`-
  Umbau (offen). Das Harness ist deshalb **hang-sicher**: *Ausführen* setzt ein
  Einmal-Run-Flag (`gb_run`) in `sessionStorage`; ein simples Neuladen führt ein
  hängendes Programm NICHT erneut aus (Editor-Inhalt bleibt in `gb_src` erhalten).
- **Hardware-/Netz-Module** (`db/net/http/serial/usb/wifi/bt`) sind im
  Web-Build nicht verfügbar (nur `--features graphics`).
- **Audio/Threads:** Web-Audio + Coroutinen-Threads brauchen ggf. zusätzliche
  emscripten-Flags (`-s AUDIO_WORKLET`, `-pthread`) — offen.

## Nächste Schritte

1. ~~Konsole im Browser~~ ✅ erledigt (Live-Editor, verifiziert).
2. **Grafik:** gbrt unter `cfg(target_os="emscripten")` auf
   `emscripten_set_main_loop` umbauen (pro Frame zurückkehren) statt
   blockierendem Loop — dann animieren Grafik-Demos im Canvas, ohne den Tab zu
   blockieren. Ggf. ASYNCIFY/`-fwasm-exceptions`-Konflikt auflösen
   (`-C panic=abort` o. ä.).
