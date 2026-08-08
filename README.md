# GameBasic

*Deutsch · [English](README.en.md)*

Ein BASIC-Dialekt mit Pascal-strikter Typisierung und OOP, ausgelegt für Spiele. Programme laufen über **`gbrt`** — die native Rust/raylib-Runtime, die Quelltext selbst lext, parst, kompiliert und ausführt (Grafik/Audio/3D inklusive). Python ist nur noch Editor-/Tooling-Schicht.

```basic
IMPORT "sprite"
IMPORT "particles"

SCREEN(320, 240, "Hallo Welt", 2)

DIM held AS SPRITE
held = SPRITE_NEW(LOADIMAGE("assets/hero.png"), 16, 16)
SPRITE_SET_POS(held, 100.0, 100.0)

WHILE NOT QUITREQUESTED()
    CLS()
    SPRITE_DRAW(held)
    FLIP()
    SLEEP(16)
WEND
```

## Schnellstart

```
.venv\Scripts\python.exe gbrun.py            # Editor öffnen
.venv\Scripts\python.exe gbrun.py datei.gb   # Programm direkt ausführen
```

## Das Lehrbuch

**[GameBasic — Das Lehrbuch](buch-referenz/buch/)** ist beides zugleich: ein Kurs, der vom ersten schwarzen Fenster bis zu Klassen, Modulen und fertigen Spielen führt, und ein Nachschlagewerk, in dem **jeder einzelne Befehl** mit einem lauffähigen Beispielprogramm erklärt wird. 414 Seiten, sieben Teile, 75 Kapitel, alle Codebeispiele gegen `gbrt --check` verifiziert.

```
node build_book.js                    # -> GameBasic-Lehrbuch.docx
node build_epub.js                    # -> GameBasic-Lehrbuch.epub (fürs Lesegerät)
<venv>\python.exe make_book.py        # Zwei-Pass-Build mit Seitenzahlen im Inhalt
```

Dieselben Kapitelquellen (`content/NN_*.js`) speisen beide Ausgaben — das `.docx` ist auf A4 gesetzt und zum Drucken, das `.epub` fließt in die Schriftgröße des Lesers und kann Nachtmodus.

## Handbuch

Vollständige Doku im [docs/](docs/README.md)-Ordner:

- **[Sprachreferenz](docs/sprache.md)** — Variablen, Typen, `ENUM`, `SELECT CASE`, Funktionen mit Defaults und Named Arguments, Klassen, Try/Catch, f-Strings, Coroutines (`YIELD` + `CORO_*`)
- **[Standard-Built-ins](docs/builtins-core.md)** — Math, Strings, Maps, File-I/O, …
- **[Grafik-Built-ins](docs/builtins-grafik.md)** — native Runtime (gbrt/raylib), Z-Layer, Sprite-Atlas, Asset-Preloader
- **[Performance](docs/PERFORMANCE.md)** — Bench-Zahlen + umgesetzte Optimierungen (Spec-Ops, IC, Typed Arrays, ECS Bulk-Ops, …)
- **Module** — 38 Stück, [Tabelle unten](#module)
- **[Code-Editor](docs/editor.md)** — Tastenkürzel, Snippets, Minimap, Multi-Cursor, Sidebar, Run/Bench, Signature-Help, **Breadcrumbs** (Scope-Pfad), **Peek-Definition** (Alt+F12), **Split-View** (Strg+\\), **Debugger** (Breakpoints inkl. **bedingter** Breakpoints/Step/Variablen), **Profiler** (Hotpath pro Zeile/Funktion), **Git-Blame**-Panel, Welcome-Showcase (Demo-Galerie mit Screenshots)
- **[Sprite-Editor](docs/sprite-editor.md)** — Pixel-Art-Editor (`gbsprites`): Multi-Frame, **Ebenen** (Sichtbarkeit/Deckkraft/Merge-Down, `.gbsprite` v5), Animation, Atlas-Export, **Export-Skalierung** (1x–8x, Nearest-Neighbor), **Lasso-Auswahl** (echte Pixel-Maske) + Rechteck-Auswahl, Onion-Skin (Deckkraft/Reichweite einstellbar), Tile-Preview
- **[Partikel-Editor](docs/particle-editor.md)** — Effekt-Editor (`gbparticles`): Emitter-Parameter live tunen mit Echtzeit-Vorschau, **Preset-Bibliothek** (Werks- + eigene Presets), GB-Code-Export
- **[Audio Studio](docs/tracker.md#audio-studio)** — vereint Tracker + SFX-Generator in **einem fullscreen Fenster** mit Reitern (`gbsound` / `gbrun.py --audio`; `gbsfx`/`gbtracker` öffnen denselben auf dem passenden Tab). `F11` Vollbild, `Strg+1/2` Tabwechsel.
- **[SFX-Generator](docs/sfx-generator.md)** — Retro-Soundeffekte (sfxr-Stil, SFX-Tab): Synth mit Pitch-Slide/Hüllkurve/Vibrato/Stereo, **SID-Charakter** (Pulsbreite/PWM + resonanter Filter-Sweep), **Preset-Bibliothek** (eigene Sounds speichern), Export WAV/GB-Code (`AUDIO_SFX`)
- **[Tracker](docs/tracker.md)** — mehrspuriger Musik-Editor (Tracker-Tab), [Tabelle unten](#tracker)
- **[Notenblatt-Editor](docs/score-editor.md)** — echte Notensatz-Darstellung (`gbscore`): Noten per Klick auf ein 5-Linien-System setzen (Violin-/Bassschlüssel, Hilfslinien, Vorzeichen), Notendauern (ganze/halbe/Viertel/Achtel/Sechzehntel + punktiert + Pause), ein Instrument pro Spur, Wiedergabe über den geteilten additiven Mixer, eigenes `.json`-Format **oder** direkter Export/Öffnen im Tracker (`gbtracker`)
- **[Tilemap-/Level-Editor](docs/tilemap-editor.md)** — Tiles aufs Gitter malen (`gbtilemap`): mehrere Layer, Object-Layer, **mehrere Tilesets**, Per-Tile-Properties (`solid`/`damage`), Stift/Füllen/Rechteck/Pipette/**Auswahl** (Copy/Cut/Paste), Speichern/Laden als Tiled-JSON (`TILED_LOAD`), GB-Code-Renderer-Export
- **[Form-Designer (WYSIWYG)](docs/form-designer.md)** — visueller GUI-Designer im Xojo-Stil (`gbform`): Controls platzieren/konfigurieren, als `.gbform` speichern, per `GUI_LOAD` im eigenen Code nutzen oder mit F5 starten
- **[Animations-FSM-Editor](docs/anim-editor.md)** — Knoten-Graph für Animation-State-Machines im Unity-Mecanim-Stil (`gbanim`): States (an Sprite-Anim gebunden) + Parameter + Transitions mit Bedingungen visuell verdrahten, als `.gbanim` speichern, per `ANIM_FSM_LOAD` ([Modul `animfsm`](docs/module-animfsm.md)) nutzen, Live-Vorschau mit F5
- **[Language Server + VSCode-Extension](docs/lsp.md)** — GameBasic in jedem LSP-Editor: Syntax-Highlighting, Diagnostics, Completion, Hover, Goto-Definition, References, Outline (`py -m gamebasic.lsp`, `vscode-gamebasic/`)
- **[Web-Playground](docs/web-playground.md)** — `gbrt` als WebAssembly im Browser, [Tabelle unten](#web-playground)
- **[`cloud`-Modul](docs/module-cloud.md)** — Cloud-Save + Leaderboard gegen den mitgelieferten, selbst hostbaren Referenz-Server [`cloudserver/`](cloudserver/README.md) (Flask + SQLite, geteiltes API-Key-Secret): `CLOUD_CONFIGURE`/`CLOUD_SAVE`/`CLOUD_LOAD`, `LEADERBOARD_SUBMIT`/`LEADERBOARD_FETCH`. Plus **`NUMFMT$`** (core-Builtin) für Idle-/Incremental-Game-taugliche Big-Number-Formatierung (`1234567` → `"1.23M"`, K/M/B/T/Qa/Qi/Sx/Sp/Oc/No/Dc, danach wissenschaftliche Notation). Demo [examples/146_cloud_idle.gb](examples/146_cloud_idle.gb)
- **[ESP32 / ESP8266 anbinden](esp32/README.md)** — fertiges Sketch-Grundgerüst (WLAN, Broker-Verbindung, Wiederverbinden, Empfang) mit vier markierten Stellen für eigenen Code; **eine Datei für beide Boards**, übersetzt für ESP32/ESP8266/ESP32-C3/ESP32-S3. Redet über [`mqtt`](docs/module-mqtt.md) mit dem GameBasic-Gegenstück [examples/159_esp32_bruecke.gb](examples/159_esp32_bruecke.gb) — das sich mit `mosquitto_pub` auch **ohne Board** fertig entwickeln lässt

### Module

38 Module, per `IMPORT "name"` verfügbar. Jedes hat eine eigene Seite unter
[docs/](docs/README.md#module).

**Spiel-Bausteine**

| Modul | Wofür |
|---|---|
| [`sprite`](docs/module-sprite.md) | animierte Sheet-Sprites: Position, Velocity, benannte Animationen, Kollision |
| [`animfsm`](docs/module-animfsm.md) | Animations-Zustandsautomat im Unity-Mecanim-Stil, aus `.gbanim` (Editor `gbanim`) |
| [`camera`](docs/module-camera.md) | Weltverschiebung, Zoom und Drehung für **alle** Zeichenbefehle; Folgen, Bildschirm↔Welt |
| [`controller`](docs/module-controller.md) | Figuren-Steuerung mit Coyote-Zeit, Sprung-Puffer und variabler Sprunghöhe |
| [`scene`](docs/module-scene.md) | Szenen-Stapel (`PUSH`/`POP`/`SWITCH`) mit Daten pro Szene |
| [`save`](docs/module-save.md) | Speicherstände als JSON, mit Versionsfeld |
| [`input`](docs/module-input.md) | benannte Aktionen statt Tastencodes, Flankenerkennung, Gamepad |
| [`timer`](docs/module-timer.md) | geplante Aktionen (`TIMER_AFTER`/`EVERY`) + `COOLDOWN`-Ratenbegrenzer |
| [`tween`](docs/module-tween.md) | Werte weich überführen, 13 Verlaufskurven |
| [`curves`](docs/module-curves.md) | Bézier, Catmull-Rom, Hermite, Smoothstep — reine Funktionen |
| [`astar`](docs/module-astar.md) | A*-Wegfindung auf einem Kachelgitter |
| [`ecs`](docs/module-ecs.md) | Entity-Component-System mit Massen-Operationen für heiße Schleifen |

**Physik und Mathematik**

| Modul | Wofür |
|---|---|
| [`physics`](docs/module-physics.md) | reine Kollisionsmathematik: Rechteck/Kreis/Strahl/Strecke/Polygon, kein Zustand |
| [`physics2d`](docs/module-physics2d.md) | **echte** 2D-Starrkörper (Rapier2D): Schwerkraft, Stapeln, Werfen, Rollen — [Demo](examples/112_physics2d.gb) |
| [`physics3d`](docs/module-physics3d.md) | dasselbe in 3D (Rapier3D) — [Demo](examples/107_physics3d.gb) |
| [`vec2`](docs/module-vec2.md) | 2D-Vektor mit überladenen Operatoren, unveränderlich |
| [`m3d`](docs/module-m3d.md) | VEC3/VEC4/QUAT/MAT4, Quaternionen, Matrizen; GPU-Instancing über `MODEL_INSTANCED` |

**Grafik und Klang**

| Modul | Wofür |
|---|---|
| `g3d` | 3D: Kamera, Modelle (OBJ/GLTF), Skelett-Animation, PBR, HDR-IBL, Schatten, Normal-Maps, Picking — siehe [Grafik-Built-ins](docs/builtins-grafik.md) |
| [`particles`](docs/module-particles.md) | Partikel-Emitter mit Schwerkraft, Farbverlauf über die Lebenszeit, fünf Zeichenarten |
| [`imgfx`](docs/module-imgfx.md) | Bilder skalieren, drehen, spiegeln, einfärben — auch kantentreu für Pixelgrafik |
| [`audio`](docs/module-audio.md) | Kanäle, Busse, Echtzeit-Effekte (Filter/Hall/Echo/Verzerrer/Kompressor/EQ), Synthese, Sampler, `.mod`/`.xm`-Wiedergabe, räumliches Audio, taktgenaue Uhr. [Modulatoren](docs/module-audio-modulatoren.md) laufen auf dem Audio-Thread weiter, auch wenn die Bildrate einbricht |

**Oberfläche**

| Modul | Wofür |
|---|---|
| [`gui`](docs/module-gui.md) | 22 Widget-Arten mit bleibendem Zustand — darunter eine **professionelle Tabelle** (sortieren, filtern, feste und umsortierbare Spalten, Zellen bearbeiten). Plastische Glas-Themen, Kippschalter, Drehregler, 9-Slice-Skins. [Alle Widgets](examples/156_gui_alle_widgets.gb) · [Tabelle](examples/157_gui_tabelle.gb) · [an SQLite](examples/158_gui_tabelle_sqlite.gb) |
| [`ui`](docs/module-ui.md) | dasselbe als Immediate-Mode: kein Aufbau, alles pro Bild neu gezeichnet |
| [`chart`](docs/module-chart.md) | sechs Diagrammarten (Kuchen, Balken, Linie, Tacho, Leiste, LED-Kette), vier Themen, Maus-Interaktion — [Demo](examples/154_chart.gb) |

**Daten**

| Modul | Wofür |
|---|---|
| [`json`](docs/module-json.md) | JSON lesen/schreiben, Pfad-Zugriff (`"user.name"`, `"items.0"`) |
| [`db`](docs/module-db.md) | SQLite mit `?`-Platzhaltern und Transaktionen |
| [`regex`](docs/module-regex.md) | Muster suchen, ersetzen, trennen |
| [`tiled`](docs/module-tiled.md) | Karten aus dem Tiled-Editor laden, inklusive Objekte und Eigenschaften |
| [`tile_collide`](docs/module-tile-collide.md) | Kasten-gegen-Kachelkarte, achsenweise — klassische Plattformer-Physik |
| [`cloud`](docs/module-cloud.md) | Spielstand und Bestenliste gegen den mitgelieferten Server [`cloudserver/`](cloudserver/README.md) |

**Netz, Hardware, Basteln**

| Modul | Wofür |
|---|---|
| [`net`](docs/module-net.md) | TCP und UDP, von Haus aus nicht blockierend — friert den Spielablauf nicht ein |
| [`html`](docs/module-html.md) | HTTP GET/POST/Download + HTML auslesen |
| [`mqtt`](docs/module-mqtt.md) | das Pub/Sub-Protokoll der IoT-Welt — der Weg zum ESP32 **über WLAN** |
| [`firmata`](docs/module-firmata.md) | Arduino-/ESP32-Pins direkt schalten, ohne eigenen Sketch |
| [`serial`](docs/module-serial.md) | rohe COM-Verbindung für eigene Protokolle |
| [`usb`](docs/module-usb.md) | USB-HID: Bastelboards, Programmieradapter, eigene Controller |
| [`bt`](docs/module-bt.md) | Bluetooth Low Energy: scannen, verbinden, Charakteristiken lesen/schreiben |
| [`wifi`](docs/module-wifi.md) | Netze suchen, verbinden, Signalstärke |

Ein fertiges Sketch-Grundgerüst fürs Board liegt in **[esp32/](esp32/README.md)**.

### Tracker

Der mehrspurige Musik-Editor im [Audio Studio](docs/tracker.md) — ein Tracker
in der Tradition von ProTracker, FastTracker und Renoise.

**Spuren**

| | |
|---|---|
| Kanalzahl | 4–32 einstellbar, der letzte ist immer der Schlagzeug-Kanal |
| Akzentfarbe je Kanal | Kopfzeile, Noten, Aussteuerung und Regler übernehmen sie |
| Mixer-Fader | echter Lautstärkeregler pro Spur — wirkt beim Vorhören, in der WAV und im erzeugten GB-Code |

**Instrumente**

| | |
|---|---|
| Bibliothek | Flügel, Orgel, Streicher, Bass, Glocke … und Schlagzeug; ein Klang pro Spur als Vorgabe |
| Instrument **pro Note** | ein Kanal ist nur ein Stimmen-Platz: jede einzelne Note darf ihr eigenes Instrument mitbringen (`Instr:`-Auswahl) |
| Sample-Instrumente | WAV/OGG laden und über die ganze Klaviatur resampeln (MOD/XM/IT-Prinzip), mit grafischem Schleifen-Editor und Panorama-Regler |
| Keymap / Multisample | verschiedene Samples auf Tastenbereiche verteilen — auch als Schlagzeug-Satz |
| SoundFont | `.sf2` einlesen: echte GM- und Hersteller-Instrumente |

**Bearbeiten**

| | |
|---|---|
| Patterns | mehrere, jeweils eigene Länge, zu einem Song angeordnet |
| Block-Auswahl | Kopieren, Ausschneiden, Einfügen, Transponieren, Zwischenwerte berechnen |
| Effekt-Spalten | Lautstärke, Tonhöhen-Gleiten/Portamento, Arpeggio, Vibrato, Retrigger, Sample-Versatz — je Note, dazu Instrument-Panorama |
| Note-Off | eine Note gezielt vor der nächsten abschneiden |

**Ausgabe**

| | |
|---|---|
| Projekt | speichern und laden als `.json` |
| GB-Player | Export als bildweise abgespielter GameBasic-Code |
| WAV | Song offline gemischt, **Stereo mit Amiga-Hard-Panning** → direkt für `PLAYMUSIC` |

### Web-Playground

`gbrt` läuft als WebAssembly im Browser — nicht als abgespeckte Fassung,
sondern als dieselbe Runtime. Quelle ins Textfeld tippen, starten, fertig.
Ausführlich in [docs/web-playground.md](docs/web-playground.md).

| Was im Browser läuft | Wie |
|---|---|
| Übersetzen | gbrt kompiliert die Quelle **selbst im Browser** — kein Pyodide, kein Server |
| Konsole und Grafik | beides gleichzeitig; der Zeichen-Ablauf gibt pro Bild ab (ASYNCIFY), damit `WHILE … FLIP() … WEND` den Reiter nicht einfriert |
| Ton | ein eigenes Kira-Backend schiebt den fertigen Mix in OpenAL-Puffer, die emscripten auf WebAudio abbildet; die Warteschlange taktet sich von selbst in Echtzeit. Browser lassen Klang erst nach dem ersten Klick zu |
| 3D | WebGL 2 — dessen GLSL ES 3.00 ist bis auf den Kopf identisch zum Desktop-GLSL. PBR, HDR-IBL, Skybox, Schatten, Instancing und Post-Effekte sind im Browser nachgewiesen |
| Dateien | ein `assets/`-Ordner neben der `.gb` kommt als `gbrt.data` mit — Bilder, Schriften und Musik liegen unter denselben Pfaden wie auf dem Desktop |
| Teilbare Links | die Quelle steckt im URL-Anker: Link öffnen heißt sehen **und** starten |

Gebaut wird mit `rust/build_wasm.py`, das Gerüst liegt in `web/`.

## Beispiele

`examples/` enthält über 170 lauffähige Demos, von "Hallo Welt" bis zum kompletten Mini-Spiel:

| Datei | Zeigt |
|---|---|
| `01_hello.gb` … `09_shapes.gb` | Sprach-Grundlagen |
| `10_pong.gb`, `22_tetris.gb`, `23_platformer.gb` | komplette Spiele |
| `24_json.gb` … `33_ui.gb` | jedes Modul mit Demo |
| `32_coinquest.gb` | Mini-Spiel mit Modulen + SELECT CASE |
| `49_pong_scene.gb` | Pong, strukturiert mit `scene` + `save` (Highscore) |
| `50_enum.gb` | ENUM in Compact- und Block-Form |
| `51_astar.gb` | A*-Pathfinding mit ASCII-Render |
| `52_named_args.gb` | Named Arguments in SUB/FUNCTION/NEW |
| `73_ecs_bullets.gb` | ECS mit pro-Entity-Loop (klassisches Pattern) |
| `75_preloader.gb` | `LOAD_ASSETS` — alle Bilder/Sounds aus einem Manifest |
| `76_layers_atlas.gb` | Z-Layer + Sprite-Atlas + Batch-Draw kombiniert (600 Tiles in einem `BATCH_FLUSH`) |
| `bench_ecs_movement_v2.gb` | ECS-Bulk-API (`ECS_INTEGRATE_FLOAT`) — 40× schneller als pro-Entity-Loop |
| `bench_ecs_systems.gb` | Bullet-Hell-Pattern mit 8 Bulk-Systemen pro Frame |
| `77_tiled_platformer.gb` | **Mini-Platformer**: Tiled-Level + Atlas + Tile-Kollision + Z-Layer + Input-Mapping |
| `98_coroutines.gb` | **Coroutines/YIELD**: Generatoren, `FOR EACH`-Drain, send/return-Dialog, `CORO_RESULT`, Methoden-Coroutine |
| `154_chart.gb` | **Diagramme**: alle sechs Arten, Themen, Maus-Interaktion |
| `156_gui_alle_widgets.gb` | **alle 22 GUI-Widgets** in einer Vollbild-Anwendung, jedes mit echter Aufgabe |
| `157_gui_tabelle.gb`, `158_gui_tabelle_sqlite.gb` | **professionelle Tabelle** — sortieren, filtern, Zellen bearbeiten; die zweite an einer echten SQLite-Datenbank |
| `159_esp32_bruecke.gb` | **ESP32 anbinden** — Messwerte empfangen, Befehle zurückschicken (Sketch in [esp32/](esp32/)) |

## Architektur

Pipeline: **Source → Preprocessor → Lexer → Parser → Compiler → VM** — **alles in `gbrt`** (Rust). `gbrt run datei.gb` ist ein eigenständiger End-to-End-Lauf ohne Python. Korrektheit sichern **run_gb-Golden-Tests** (`assert run_gb(src) == expected`, spawnt `gbrt run`) + Rust-`#[test]`s.

> **Geschichte:** Früher liefen Programme zusätzlich über einen Python-**Tree-Walker** und zwei Python-**Bytecode-VMs** (Python-VM, Cython-VM), mit „bit-identischem Output" als Garantie. Seit **Stufe B** sind Tree-Walker + Python-Toolchain (interpreter/compiler/vm/serialize) **alle entfernt** — `gbrt` ist die einzige Runtime und kompiliert den Quelltext selbst.

**Native Rust-Runtime (raylib) — die einzige Runtime.** `gbrt` (`rust/gb_runtime/`) lext, parst, kompiliert und führt den Quelltext selbst aus — ein eigenständiges Rust-Frontend, kein Python im Ausführungspfad. Skalare, Strings, Arrays, Maps, Tupel, OOP (Klassen/Methoden/Properties/Operatoren), Slicing, Comprehensions, TRY/THROW und die puren Builtins laufen nativ. **Grafik via raylib** (feature-gated): 2D-Primitive, Text, Bilder, Input, headless per Screenshot verifiziert (`rust\build_runtime.py`). **Dev-Loop:** `gbrun.py --native <datei.gb>` kompiliert und startet nativ in einem Befehl; Laufzeitfehler zeigen `datei.gb:Zeile`. **3D** (Modul `g3d`, native-only): Kamera + Würfel/Kugel/Zylinder/Kegel/Ebene/Linien/Gitter über raylibs `begin_mode3D` ([examples/82_3d_intro.gb](examples/82_3d_intro.gb)), plus **3D-Modelle** — `LOADMODEL` (OBJ/GLTF), prozedurale `MESH_*` (Cube/Sphere/Cylinder/Torus/Knot/Plane/**Heightmap-Terrain**), `MODEL`/`MODEL_EX`/`MODEL_WIRES`, `MODEL_TEXTURE`, **Skelett-Animation** geriggter GLTF/IQM (`MODEL_LOAD_ANIMS` + `MODEL_ANIMATE`, [examples/108_skeletal_anim.gb](examples/108_skeletal_anim.gb)), **Billboards** (`BILLBOARD`), **Ray-Kollision/Picking** (`RAY_HIT_BOX`/`SPHERE`, `PICK_BOX`/`SPHERE` — und auf echter Fläche statt Hüllkörper `RAY_HIT_TRI`/`QUAD`, `PICK_TRI`/`QUAD`, [examples/151_picking_flaechen.gb](examples/151_picking_flaechen.gb)), **Beleuchtung** (Blinn-Phong, bis 4 Lichter: `LIGHT_DIRECTIONAL`/`LIGHT_POINT` + `MODEL_LIT`) und **Kamera-Modi** (`CAMERA3D_UPDATE` orbital/first-person) ([examples/88_3d_models.gb](examples/88_3d_models.gb), [examples/89_heightmap.gb](examples/89_heightmap.gb), [examples/90_billboards_picking.gb](examples/90_billboards_picking.gb), [examples/91_lighting.gb](examples/91_lighting.gb)). **Standalone-Export:** `gbrun.py --export <datei.gb>` (oder Editor *Export → .exe*, Ctrl+F6) bündelt Bytecode + `assets/` in eine eigenständige `.exe`, die ohne Python läuft (gbrt + angehängter Bytecode). **Audio** nativ über **Kira** (cpal, eigener Audio-Thread — löste 2026-06-13 raylib-Audio ab): `LOADSOUND`/`PLAYSOUND`/`STOPSOUND`/`UNLOADSOUND` (Puffer freigeben gegen Sound-Akkumulation in langen Songs) + `PLAYMUSIC`/`STOPMUSIC` (inkl. `.mod`/`.xm` via reinem Rust-Player). **Retained-GUI** (`gui`) nativ: Fenster + Button/Label/Checkbox/Slider/TextInput/Panel/**Table** (scrollbar, selektierbar), Theme, Drag/Z-Order/Fokus, FUNCREF-Callbacks. Tabellen (`UI_TABLE` + `GUI_TABLE`) nativ. **Eingabe aufzeichnen und abspielen** (`AUTOMATION_RECORD`/`STOP`/`PLAY` — Demo-/Attract-Modus, nachspielbare Fehlerberichte, automatische Spieltests; [docs/automation.md](docs/automation.md), [examples/153_automation.gb](examples/153_automation.gb)). **Game-Loop** (`DELTA`/`FPS`/`SETFPS`), **GPU-Shader/Post-Processing** (`SHADER_LOAD`/`POSTFX` — CRT/Bloom/Vignette) und **TTF-Fonts** (`LOADFONT`/`SETFONT`/`TEXT_SPACING`, [examples/87_ttf_fonts.gb](examples/87_ttf_fonts.gb)) ebenfalls nativ. **Gamepad** nativ (`INPUT_JOY_COUNT/NAME/AXIS` + `JOY_BUTTON_*`/`JOY_DPAD_*`-Bindings über raylib) und **Tiefen-Fog** für beleuchtete Szenen (`LIGHT_FOG`, [examples/92_fog.gb](examples/92_fog.gb)). **Shadow-Mapping** (`SHADOW_ENABLE`/`SHADOW_AREA`/`SHADOW_TARGET` — directional Schlagschatten mit PCF, [examples/93_shadows.gb](examples/93_shadows.gb)) und **Normal-Mapping** (`MODEL_TEXTURE_NORMAL` — Pro-Pixel-Oberflächendetail via TBN, [examples/94_normalmap.gb](examples/94_normalmap.gb)). **PBR** (Cook-Torrance: die native Beleuchtung ist physically-based; `MODEL_PBR` für Metalness/Roughness, [examples/95_pbr.gb](examples/95_pbr.gb)) inkl. **Emissive/Neon-Glow** (`MODEL_EMISSIVE` — Eigenleuchten pro Modell, mit Bloom-`POSTFX` echter Glow, [examples/110_emissive_glow.gb](examples/110_emissive_glow.gb)) inkl. **Image-Based-Lighting** — analytisch (`LIGHT_ENV`, [examples/96_ibl.gb](examples/96_ibl.gb)) **und echtes HDR-Cubemap-IBL** (`LIGHT_ENV_HDR` — lädt ein `.hdr`, berechnet Irradiance/Prefilter/BRDF-LUT; Metalle spiegeln die echte Umgebung, [examples/99_ibl_hdr.gb](examples/99_ibl_hdr.gb)). **Fullscreen-Showcase:** [examples/97_pbr_reactor.gb](examples/97_pbr_reactor.gb) („PBR REACTOR") — FFT-reaktiver Ring aus Chrom-PBR-Kugeln, IBL, Schatten, Bloom + Stereo-Techno (CC0). **Coroutines/`YIELD`** laufen ebenfalls nativ — die Rust-VM suspendiert via Frame-Snapshot (keine Threads, raylib-Main-Thread-sicher, deterministisch) inkl. Standalone-`.exe` ([examples/98_coroutines.gb](examples/98_coroutines.gb)). **Module-Voll-Portierung abgeschlossen** — alle früher Python-only-Module laufen jetzt nativ: `regex`, `tiled`, `tile_collide`, `controller`, erweitertes `audio`, sowie feature-gated `db` (rusqlite), `net` (std::net), `mqtt` (MQTT-3.1.1-Client fuer ESP32/IoT, auf `net` aufbauend), `html` (ureq) und Hardware/IoT `serial` (serialport), `firmata` (Arduino/ESP32-Pin-Steuerung ueber StandardFirmata, auf `serial` aufbauend), `usb` (hidapi), `wifi` (netsh), `bt` (btleplug/BLE). Damit braucht nur noch der Editor Python; der Rest läuft komplett nativ (`build_runtime.py --hardware` / `--full` für die schweren Module). Plan & Status in [docs/rust-runtime.md](docs/rust-runtime.md).

**Front-End-Portierung nach Rust — abgeschlossen.** Die komplette Toolchain (Lexer → Parser → Compiler → Preprocessor) wurde nach Rust portiert, jede Stufe per Output-Parität gegen den Python-Tree-Walker verifiziert. **`gbrt run datei.gb` ist ein eigenständiger End-to-End-Lauf ohne Python:** preprocesst (`IMPORT`-Auflösung von Quelldateien und Built-in-Modulen), lext, parst, kompiliert und führt aus — Skalare/Arithmetik/Kontrollfluss, Arrays/Maps, Funktionen, Klassen/OOP, `SELECT`/`FOR EACH`/Tupel/`WITH`/`TRY`/Slicing/Comprehensions/Coroutinen. Wie `gbrun.py` wird ins Datei-Verzeichnis gewechselt, sodass relative `IMPORT`- und Asset-Pfade stimmen (`gbrt datei.gb` ohne `run` funktioniert genauso; `.gbc`-Dateien laufen weiter den direkten VM-Pfad). Debug-Einstiege `gbrt --tokens`/`--ast`/`--preprocess`/`--runsrc`. **Selbst-Export ohne Python:** `gbrt --export datei.gb` kompiliert die Quelle selbst und bündelt sie zu einer eigenständigen `.exe` (hängt den Bytecode an eine Kopie der Runtime, kopiert `assets/`). Aliasierte Modul-Imports (`IMPORT "json" AS j` → `J_PARSE`, `DIM h AS J_HANDLE`) funktionieren ebenfalls nativ. Damit ist auch der **Web-Playground ein reines Rust-WASM**, das die Quelle im Browser kompiliert (kein Pyodide): `rust/build_wasm.py datei.gb` erzeugt `web/gbrt.{js,wasm}` mit eingebetteter Quelle (emscripten-Toolchain auf Windows wird automatisch verdrahtet). **Konsole und animierte Grafik laufen im Browser** — der GB-Render-Loop yieldet pro Frame via ASYNCIFY (`emscripten_sleep(0)` in `flip()`), sodass `WHILE … FLIP() … WEND` den Tab nicht einfriert; **teilbare Links** packen die Quelle in den URL-Hash. Plan & Stufen in [docs/rust-frontend-port.md](docs/rust-frontend-port.md).

Architektur-Details und Erweiterungs-Hinweise in [CLAUDE.md](CLAUDE.md).

## Tests

```
.venv\Scripts\python.exe -m pytest tests/
```

Über 3090 Tests — Built-ins, alle Module, Sprach-Konstrukte, Editor-Features und Example-Smoke-Tests. Korrektheit sichern **run_gb-Golden-Tests** (`assert run_gb(src) == expected`, spawnen `gbrt run`) + Rust-`#[test]`s; sie skippen ohne gebautes `gbrt`.

## Lizenz

Privat.
