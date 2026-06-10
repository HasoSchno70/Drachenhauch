# Web-Playground (gbrt → WebAssembly)

GameBasic-Programme im Browser laufen lassen — die **native Runtime `gbrt`**
(Rust/raylib) als WebAssembly via emscripten, mit Grafik im `<canvas>` und
Konsolen-Ausgabe daneben.

> **Status: baut & läuft, inkl. Grafik (verifiziert 2026-06-10).** Mit
> installierter Toolchain (emscripten 6.0.0 + Rust-Target
> `wasm32-unknown-emscripten`) erzeugt `rust/build_wasm.py` ein lauffähiges
> `web/gbrt.js` + `web/gbrt.wasm`. **Konsolen- UND Grafik-Programme laufen im
> Browser** — animierte Demos im `<canvas>` ohne den Tab einzufrieren (im Browser
> per Preview verifiziert: bewegtes Sprite + laufender Frame-Zähler). gbrt
> kompiliert die eingebettete **Quelle** selbst im WASM (kein Pyodide). Die
> Build-Artefakte (`gbrt.js`/`.wasm`/`program.gb`/`.gbc`) sind gitignored.
> **Teilbare Links:** „Link teilen" packt die Quelle in den URL-Hash — wer den
> Link öffnet, sieht und startet genau dieses Programm.

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
| `web/index.html` | Live-Editor (`<textarea id="src">`) + `<canvas id="canvas">` + Output-Bereich + Run-/Teilen-Button |
| `web/playground.js` | Live-Playground: Editor→`sessionStorage`, Reload für frische Runtime, schreibt die Quelle nach `/program.gb`, `callMain()`; stdout→Div (Module.print + console.log-Fallback). Einmal-Run-Flag `gb_run` macht hängende Programme reload-erholbar. **Teilbare Links:** Quelle base64url im URL-Hash (`#gb=…`); ein geöffneter Link lädt + startet das Programm. |

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

## Architektur-Hinweis: der Render-Loop (gelöst)

Die VM treibt ihren Frame-/Render-Loop **blockierend**: das GB-Programm hat die
Schleife (`WHILE NOT QUITREQUESTED() … FLIP() … WEND`), die VM kehrt erst am
Programmende zurück. Im Browser darf der Main-Thread aber nicht blockieren, sonst
hängt der Tab.

**Lösung (umgesetzt): ASYNCIFY + ein Yield pro Frame.** `build_wasm.py` setzt
`-s ASYNCIFY`; allein reicht das nicht, weil ohne einen Yield-Punkt der
blockierende Loop nie ans Browser-Event-Loop zurückgibt. Deshalb ruft
**`graphics.rs::flip()` unter `cfg(target_os="emscripten")` `emscripten_sleep(0)`**
direkt nach dem Präsentieren (`EndDrawing`). ASYNCIFY wickelt dabei den gesamten
Rust-Stack ab, gibt die Kontrolle an den Browser (Canvas compositet, Input-Events
werden zugestellt) und setzt beim nächsten Tick genau dort fort. Damit kooperiert
der unveränderte GB-Render-Loop mit dem Browser — **kein Umbau auf
`emscripten_set_main_loop` nötig**, die VM-/Coroutinen-Logik bleibt unangetastet.

> Der theoretisch sauberere Weg (`emscripten_set_main_loop`, VM kehrt pro Frame
> zurück) wäre ein großer Eingriff in `vm.rs`; der ASYNCIFY-Yield ist ~3 Zeilen
> und liefert dasselbe Ergebnis. Der befürchtete ASYNCIFY-vs-`-fwasm-exceptions`-
> Konflikt erwies sich als bloße Linker-Warnung — der Build läuft.

## Stand & Grenzen (verifiziert 2026-06-10)

- **Konsolen-Programme: laufen im Browser ✅.** Quelle tippen → *Ausführen* →
  korrekte Ausgabe. gbrt kompiliert die `.gb`-Quelle **selbst im WASM**
  (Front-End-Port) — kein Pyodide, kein vorab kompiliertes `.gbc`.
- **Grafik-Programme: laufen im Browser ✅.** Der Render-Loop yieldet pro Frame
  (ASYNCIFY-Yield in `flip()`, siehe oben) → animierte Demos im Canvas, der Tab
  bleibt reaktionsfähig (im Browser per Preview geprüft: bewegtes Objekt +
  hochzählender Frame-Counter, kein Einfrieren). Das Harness bleibt **hang-sicher**
  (Einmal-Run-Flag `gb_run`), falls ein Programm doch eng-busy läuft.
- **Teilbare Links ✅.** *Link teilen* base64url-kodiert die Quelle in den
  URL-Hash (`#gb=…`, reine Client-JS, kein Backend). Ein geöffneter Link lädt die
  Quelle in den Editor **und startet sie direkt** (frische Runtime). *Ausführen*
  hält den Hash aktuell, sodass die URL jederzeit teilbar ist.
- **Hardware-/Netz-Module** (`db/net/http/serial/usb/wifi/bt`) sind im
  Web-Build nicht verfügbar (nur `--features graphics`).
- **Audio/Threads:** Web-Audio + Coroutinen-Threads brauchen ggf. zusätzliche
  emscripten-Flags (`-s AUDIO_WORKLET`, `-pthread`) — offen.

## Nächste Schritte

1. ~~Konsole im Browser~~ ✅ erledigt (Live-Editor, verifiziert).
2. ~~Grafik im Browser~~ ✅ erledigt (ASYNCIFY-Yield in `flip()`, verifiziert).
3. ~~Teilbare Links~~ ✅ erledigt (Quelle im URL-Hash).
4. **Optional:** Web-Audio (`-s AUDIO_WORKLET`), Gamepad/Touch-Input fürs Handy,
   eine kleine Beispiel-Galerie aus vorgefertigten Share-Links.
