# Web-Playground (gbrt → WebAssembly)


> **Stand 2026-08-03: der Web-Build laeuft.** Konsolen- UND Grafik-Programme
> werden im Browser kompiliert und ausgefuehrt (verifiziert: Kreise, Formen und
> Text auf der Leinwand, `gbrt.wasm` 8,8 MB). Der frueher als Kernhuerde
> genannte blockierende Render-Loop ist mit ASYNCIFY kein Problem -- eine
> `FOR`-Schleife mit `FLIP()` laeuft durch und gibt am Ende brav ihre
> `PRINT`-Zeile aus.
>
> Vier Dinge waren dafuer noetig (alle im Repo):
> 1. **`dialogs` vom `graphics`-Feature getrennt.** `rfd` (native Dateidialoge)
>    zieht js-sys/wasm-bindgen in einer Version nach, mit der cpals
>    WebAudio-Host nicht mehr uebersetzt -- und im Browser gibt es ohnehin keine
>    OS-Dialoge.
> 2. **wasm-bindgen fuer emscripten festgenagelt** (`=0.2.100`). Kira zieht cpal
>    fuer JEDES `wasm32`-Ziel mit dem Feature `wasm-bindgen` herein, auch fuer
>    emscripten, wo der WebAudio-Host gar nicht gebraucht wird (Ziel-Filter-
>    Fehler in Kiras Manifest). Er wird also mitkompiliert und muss uebersetzen.
> 3. **Kein Streaming auf wasm.** Kiras `sound::streaming` ist dort
>    wegkonfiguriert und `StaticSoundData::from_file` gibt es nicht. Musik laeuft
>    im Web ueber die Static-Variante (einmal ganz laden), Klaenge ueber
>    `from_cursor` mit selbst gelesenen Bytes.
> 4. **Leinwand-Groesse selbst setzen.** Nachgemessen: die Laufzeit meldet nach
>    `SCREEN(480,320)` brav 480x320, der PUFFER der `<canvas>` blieb aber 1x1 --
>    per CSS gestreckt wurde daraus eine Farbflaeche. `graphics.rs` ruft jetzt
>    `emscripten_set_canvas_element_size` und zieht die Groesse in den ersten
>    acht Bildern nach (raylib setzt sie danach noch einmal selbst).
>
> **Assets kommen mit** (2026-08-03). Liegt neben der `.gb` ein `assets/`-Ordner,
> packt `build_wasm.py` ihn in eine `gbrt.data` und haengt sie ueber
> `--preload-file` ins virtuelle Dateisystem -- Programme laden ihre Bilder,
> Schriften, Shader und Musik danach unter genau demselben Pfad wie auf dem
> Desktop. Nachgemessen im Browser: `FILESIZE("assets/font.ttf")` liefert 168644,
> `LOADFONT` darauf liefert ein gueltiges Handle, und der Text erscheint in der
> eingebetteten Schrift. **`--preload-file` und `--embed-file` schliessen
> einander aus** -- sobald Assets dabei sind, wird auch die Quelle vorgeladen.
> Neben `gbrt.js`/`gbrt.wasm` muss dann zwingend `gbrt.data` mit ausgeliefert
> werden, sonst startet gar nichts.

## Bekannte Grenzen im Browser

Alle nachgemessen, nicht vermutet:

- **Leinwand wird nach Programmende schwarz.** WebGL verwirft den Zeichenpuffer
  nach dem Anzeigen (`preserveDrawingBuffer` ist nicht gesetzt). Aus demselben
  Grund liefert `SAVESCREENSHOT` im Browser ein leeres Bild. Betrifft nur das
  Nachher, nicht das Laufende -- waehrend des Laufs ist alles zu sehen.
- **Ton ist hoerbar** (seit 2026-08-04) -- aber erst nach dem ersten Klick.
  cpal hat keinen emscripten-Host mehr (sein WebAudio-Host haengt an
  wasm-bindgen-JS-Glue, das emscripten nicht liefert), also lief frueher jedes
  Programm mit Ton in `NoDefaultOutputDevice` und starb. gbrt bringt fuer den
  Browser deshalb ein **eigenes Kira-Backend** mit
  (`src/web_audio.rs`): Kiras `Backend`-Vertrag reicht uns den `Renderer`
  durch, wir nehmen den fertigen Mix und schieben ihn in eine Warteschlange von
  **OpenAL**-Puffern -- emscripten setzt OpenAL intern auf WebAudio um.

  **Die Warteschlange taktet sich selbst:** pro Bild wird nur so weit
  nachgefuellt, wie WebAudio leergespielt hat. Im Beharrungszustand ist das
  exakt Echtzeit, ohne irgendwo eine Uhr abzulesen. Nachgemessen im Browser:
  nach einem Lauf steht die Wanduhr bei 8,35 s und `AUDIO_MUSIC_POSITION()` bei
  8,50 s -- die Musik-Uhr folgt der echten Zeit auf 2 % genau, und das kann sie
  nur, wenn die Puffer tatsaechlich abgespielt werden. **Abgehoert und fuer
  sauber befunden** (04.08.2026) -- das Messen allein haette das Stottern aus
  der Ratenverwechslung unten nicht aufgedeckt.

  **Autoplay-Sperre:** eine `AudioContext` startet angehalten, bis der Nutzer
  die Seite einmal angefasst hat. Bis dahin verbraucht WebAudio nichts, die
  Warteschlange bleibt voll -- und weil die Uhr an ihr haengt, steht auch die
  Wiedergabe. Ein Programm, dessen Ablauf an `AUDIO_MUSIC_POSITION` haengt,
  wartet also auf den ersten Klick. Der Playground weist darauf hin.

  **Mit DER Rate rechnen, die der Browser fahrt.** Nachgemessen: die
  `AudioContext` lief hier mit **48000 Hz**, unsere Puffer waren mit 44100
  angemeldet. WebAudio rechnet dann JEDEN Puffer einzeln um -- und weil bei so
  einer Umrechnung die Nachbarpuffer fehlen, entsteht an jeder Naht ein Sprung:
  es stottert im Puffertakt, obwohl nichts leerlauft. Die Rate kommt deshalb
  aus `alcGetIntegerv(ALC_FREQUENCY)` und wird an Kira durchgereicht.

  **`-lopenal` ist Pflicht.** Das von raylib mitgebrachte `-lal` ist bloss ein
  leeres Archiv; die JS-Umsetzung steckt in `libopenal.js` und kommt nur ueber
  `-lopenal` dazu. Ohne das Flag uebersetzt und linkt alles, und der Browser
  bricht erst beim Start ab: *"alcOpenDevice: function import requires a
  callable"*.

  **Nur `FLIP` fuellt nach.** Ein Konsolenprogramm ohne Bildschleife bekommt im
  Browser keinen Takt -- dort bleibt es still.
- ~~Eigene Shader in Desktop-GLSL uebersetzen nicht.~~ **Erledigt seit
  2026-08-03: 3D und Post-Effekte laufen.** Der Web-Build faehrt jetzt
  **WebGL 2** statt WebGL 1 (`opengl_es_30` + `MIN/MAX_WEBGL_VERSION=2`).
  Dessen Sprache GLSL ES 3.00 ist bis auf die Versionszeile und die
  Genauigkeits-Angaben dasselbe wie Desktop-GLSL 330 -- statt alles auf die
  aeltere Sprache herunterzuschreiben (und dabei IBL zu verlieren), wird also
  das Ziel angehoben. `fuer_ziel_uebersetzen` in `graphics.rs` tauscht beim
  Laden nur den Kopf; der Rumpf bleibt EINER. Das gilt auch fuer
  `SHADER_LOAD`, ein Desktop-Shader laeuft also unveraendert im Browser.
- **Keine Nicht-Zweierpotenz-Texturen mit Wiederholung**
  (`GL: NPOT textures extension not found`) -- eine Warnung, kein Fehler.

## Die Demo im Browser (Stand 2026-08-04)

`gbdemo/` **laeuft vollstaendig** -- mit Bild und Ton, wie auf dem Desktop.
Im Browser nachgesehen:

| | |
|---|---|
| Ablaufplan an `AUDIO_MUSIC_POSITION` | laeuft in Echtzeit |
| Tracker-Modul (261 KB `.mod`) | wird gespielt, **hoerbar** (nach dem ersten Klick) |
| Spektrum-Saeulen, Sinus-Scroller, Logo | vollstaendig |
| Szenen-Spruenge mit den Zifferntasten | funktionieren |
| 2D-Szenen (Titel, Bulk-Linien, Physik-Logo) | vollstaendig |
| Post-Effekt (Plasma, Tunnel, Glimmen) | vollstaendig |
| 3D: Wuerfelfeld mit `MODEL_INSTANCED` (1600 Wuerfel) | vollstaendig |
| 3D: `MODEL_PBR` + `LIGHT_ENV_HDR` + `SKYBOX` + Schatten | vollstaendig |

Damit laeuft die Demo im Browser **vollstaendig**.

### Der Fund, der 3D im Browser blockierte

Der Shader war nicht schuld. Nach der Umstellung auf WebGL 2 uebersetzten und
verlinkten alle Shader fehlerfrei -- **und trotzdem war jedes `MODEL_LIT`-Modell
unsichtbar**. Nachgemessen (Zeichenaufrufe im Browser umhaengt und `getError()`
abgefragt): `drawArrays` lieferte `INVALID_OPERATION`.

Ursache: ohne HDR-Umgebung blieben die beiden `samplerCube`-Uniformen auf ihrer
Vorgabe -- Textureinheit 0, wo schon `texture0` als `sampler2D` liegt. **Zwei
verschiedene Sampler-Arten auf einer Einheit sind in WebGL 2 ein Fehler**, und
der Zeichenaufruf wird verworfen. Desktop-Treiber sind da nachsichtig, deshalb
faellt es nur im Browser auf. Die Einheiten werden jetzt immer gesetzt.

Die Lehre daraus ist allgemein: **ein fehlerfrei uebersetzter Shader sagt
nichts darueber, ob gezeichnet wird.** Wenn Geometrie verschwindet, ist
`getError()` nach dem Zeichenaufruf die Messung, die traegt.

### Zwei Fallen beim Bauen, die Stunden kosten koennen

* **cargo verfolgt `EMCC_CFLAGS` nicht.** Aendert man nur ein Linker-Flag,
  sieht cargo unveraenderte Quellen, ueberspringt das Linken -- und die alte
  `.wasm` bleibt liegen. Der Build meldet Erfolg, das Flag ist nicht drin.
  Genau so sah es aus, als `STACK_SIZE` "nicht half": es war nie im Binary.
  `build_wasm.py` loescht die Ausgabe jetzt selbst, sobald sich die Flags
  aendern (`erzwinge_relink`).
* **Der Status "fertig" im Playground luegt bei Grafik-Programmen.** ASYNCIFY
  laesst `callMain` schon beim ersten Bild zurueckkehren; das Programm laeuft
  danach weiter. Wer daraus "das Programm ist zu Ende" liest, sucht Fehler an
  der falschen Stelle.

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
- **Audio:** hörbar über ein eigenes Kira-Backend, das den fertigen Mix in
  OpenAL-Puffer schiebt (emscripten setzt OpenAL auf WebAudio um) — Details
  oben unter „Bekannte Grenzen".

## Nächste Schritte

1. ~~Konsole im Browser~~ ✅ erledigt (Live-Editor, verifiziert).
2. ~~Grafik im Browser~~ ✅ erledigt (ASYNCIFY-Yield in `flip()`, verifiziert).
3. ~~Teilbare Links~~ ✅ erledigt (Quelle im URL-Hash).
4. ~~Assets mitliefern~~ ✅ erledigt (`--preload-file`, `gbrt.data`, verifiziert).
5. ~~Audio im Browser~~ ✅ erledigt — zuerst stumm über Kiras `MockBackend`,
   seit 2026-08-04 **hörbar** über ein eigenes Backend mit OpenAL-Ausgabe.
6. ~~Shader fürs Web~~ ✅ erledigt — über WebGL 2 statt eines Ports nach
   GLSL ES 1.00 (3D, IBL, Skybox, Schatten und Post-Effekte verifiziert).
7. ~~Beispiel-Galerie~~ ✅ erledigt (`web/beispiele.js`, sechs Programme ohne
   Dateizugriff).
8. ~~Touch-Input~~ ✅ verifiziert: `TOUCH_COUNT`/`TOUCH_X`/`TOUCH_Y` melden im
   Browser die richtige Stelle (synthetische Berührung bei 40 %/60 % einer
   320×200-Leinwand → 128,3/119,5). Die **Gesten** (`GESTURE$`) blieben im Test
   leer — ob das an den synthetischen Ereignissen liegt oder eine echte Lücke
   ist, wurde nicht geklärt.
9. **Offen:** ein echtes Gerät zum Gegenprüfen (Handy-Browser). Der Ton selbst
   ist abgehakt — am 04.08.2026 abgehört und für sauber befunden.
