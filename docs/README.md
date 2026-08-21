# Drachenhauch Handbuch

Vollständige Referenz für die Sprache Drachenhauch, alle eingebauten Befehle und alle Built-in-Module.

Drachenhauch ist ein BASIC-Dialekt mit Pascal-strikter Typisierung und OOP, ausgelegt für Spiele. Programme laufen über **`dhrt`** — die native Rust/raylib-Runtime, die Quelltext selbst lext, parst, kompiliert und ausführt. Python ist nur noch Editor-/Tooling-Schicht.

## Inhalt

### Die Sprache

- **[Sprachreferenz](sprache.md)** — Variablen, Typen, Operatoren, Kontrollfluss (IF, SELECT CASE, WHILE, FOR), Funktionen, Klassen, Try/Catch, Imports
- **[Variablen-Scope](scope.md)** — wo eine Variable gilt: global, in Funktionen, in Methoden — und warum es kein Block-Scoping gibt

### Eingebaute Befehle

- **[Standard-Built-ins](builtins-core.md)** — Math, Strings, Bitwise, Maps, File-I/O, Konvertierung, Zeit/Random
- **[Grafik-Built-ins](builtins-grafik.md)** — SCREEN, CLS, BOX, CIRCLE, LOADIMAGE, Sound, Tilemap, Eingabe, **`LOAD_ASSETS`** (Bulk-Preloader), **Sprite-Atlas + Batch-Draw**, **Z-Layer-Rendering**
- **[Performance](PERFORMANCE.md)** — historische Bench-Zahlen + Liste umgesetzter Optimierungen (die verglichenen Python-Pfade sind seit Stufe B entfernt; Produktion = `dhrt`)

### Module

Jedes Modul wird mit `IMPORT "<name>"` aktiviert und stellt eigene Befehle bereit.

> **Alle Module laufen auch in der nativen Runtime (dhrt)** — die meisten sind dort immer dabei; `db`/`net`/`http` (= `html`) sind im Standard-Dev-Build (`python rust/build_runtime.py`) schon dabei, Hardware (`serial`/`usb`/`wifi`/`bt`) kommt zusätzlich mit `--hardware` dazu. Jede Modul-Doku hat unten einen Abschnitt **„In der nativen Runtime (dhrt)"** mit Feature-Flag und Eigenheiten; Überblick in [rust-runtime.md](rust-runtime.md).

| Modul | Was es kann | Doku |
|---|---|---|
| `json` | JSON parsen, Werte über Pfad-Notation lesen | [module-json.md](module-json.md) |
| `db` | SQLite-Datenbank: CREATE/INSERT/SELECT, Transaktionen | [module-db.md](module-db.md) |
| `tween` | Werteinterpolation über Zeit (Animationen, Easings) | [module-tween.md](module-tween.md) |
| `zeit` | Datum und Uhrzeit als Zahl: `ZEIT_PARSE`/`ZEIT_PLUS`/`ZEIT_DIFF`, Anzeige über `ZEIT_FORMAT$` | [module-zeit.md](module-zeit.md) |
| `timer` | Geplante Aktionen: `TIMER_AFTER`/`TIMER_EVERY` mit FUNCREF-Callbacks (`TIMER_UPDATE` pro Frame) + `COOLDOWN`-Ratenbegrenzer | [module-timer.md](module-timer.md) |
| `imgfx` | Bild-Effekte: Scale, Rotate, Flip, Tint, Copy | [module-imgfx.md](module-imgfx.md) |
| `particles` | Partikel-System mit Velocity, Gravity, Lifetime | [module-particles.md](module-particles.md) |
| `physics` | AABB-/Circle-Collision, Distance, Reflect, Ray-Cast | [module-physics.md](module-physics.md) |
| `camera` | World-Translation und Zoom für alle Drawing-Befehle | [module-camera.md](module-camera.md) |
| `sprite` | Animierte Sprites aus Sheets, AABB-Kollision, Flip/Scale/Tint | [module-sprite.md](module-sprite.md) |
| `ui` | Immediate-Mode-UI: Label, Button, Checkbox, Slider | [module-ui.md](module-ui.md) |
| `scene` | Stack-basierter Scene-/State-Manager mit Pro-Scene-Daten | [module-scene.md](module-scene.md) |
| `save` | Persistente Save-Slots (JSON-Backend, typsicher, versioniert) | [module-save.md](module-save.md) |
| `astar` | A*-Pathfinding auf Tile-Grids, mit Diagonal + Heuristiken | [module-astar.md](module-astar.md) |
| `ecs` | Entity-Component-System mit Sparse-Set-Storage + **Bulk-System-Ops** (`ECS_INTEGRATE_FLOAT`, `ECS_SCALE_FLOAT`, …) | [module-ecs.md](module-ecs.md) |
| `vec2` | Immutable 2D-Vektor mit Operator-Overloading (`+ - * / = <>`), Lerp, Reflect, Polar | [module-vec2.md](module-vec2.md) |
| `input` | Action-basiertes Input-Mapping, Multi-Key-Bindings, Edge-Detection (PRESSED/RELEASED), virtuelle Achse | [module-input.md](module-input.md) |
| `regex` | Python-kompatible Pattern-Matching mit Pattern-Cache: `REGEX_MATCH/TEST/FIND/REPLACE/SPLIT` | [module-regex.md](module-regex.md) |
| `audio` | Erweiterte Audio-API: Channels, Pause/Resume/Fade, Pan, Music-Position, Tone-Generation (Sine/Square/Saw/Triangle/Noise) | [module-audio.md](module-audio.md) |
| `curves` | Animation-Kurven: Bezier, Catmull-Rom, Hermite, Smoothstep — pure Functions, kein State | [module-curves.md](module-curves.md) |
| `net` | TCP + UDP via stdlib-Sockets, non-blocking by default, UTF-8-Encoding | [module-net.md](module-net.md) |
| `html` | HTTP-GET/POST/DOWNLOAD und HTML-Parsing — pure stdlib, kein pip noetig | [module-html.md](module-html.md) |
| `cloud` | Cloud-Save + Leaderboard gegen den selbst hostbaren Referenz-Server (`cloudserver/`) | [module-cloud.md](module-cloud.md) |
| `bt` | Bluetooth Low Energy (BLE) — Scan, Connect, Read/Write Characteristics (Python: `bleak`, nativ: `btleplug`) | [module-bt.md](module-bt.md) |
| `serial` | RS-232 / USB-COM — Open, Read/Write, Available, Flush, Timeout (Python: `pyserial`, nativ: `serialport`) | [module-serial.md](module-serial.md) |
| `usb` | USB-HID via `hidapi` — Custom-Controller, Maker-Boards, Programmer | [module-usb.md](module-usb.md) |
| `wifi` | WiFi-Management (Windows-only via `netsh wlan`): Scan, Connect, Disconnect, Profile | [module-wifi.md](module-wifi.md) |
| `tiled` | [Tiled](https://www.mapeditor.org/)-Map-Loader (JSON). Tile-Layer, Object-Layer, Tile- + Object-Properties | [module-tiled.md](module-tiled.md) |
| `tile_collide` | Box-vs-Tilemap-Kollision: TILE_SWEEP_X/Y mit separat-Achsen-Pattern. Klassischer Platformer | [module-tile-collide.md](module-tile-collide.md) |
| `controller` | Character-Controller mit Coyote-Time, Jump-Buffer, Variable-Jump-Height. Klassische "feel-good"-Platformer-Mechanik | [module-controller.md](module-controller.md) |
| `gui` | Retained-Mode-Oberflächen: Fenster und Widgets als persistente Objekte, 22 Widget-Arten inkl. Tabelle | [module-gui.md](module-gui.md) |
| `chart` | Diagramme: Kuchen/Donut, Balken, Linie/Fläche, Tacho, Leiste, LED — mit Maus und Tooltip | [module-chart.md](module-chart.md) |
| `m3d` | 3D-Mathematik: VEC3/VEC4/QUAT/MAT4 für hierarchische Transforms, eigene Kameras, Instancing | [module-m3d.md](module-m3d.md) |
| `animfsm` | Animations-State-Machine im Mecanim-Stil, datengetrieben aus `.dhanim` | [module-animfsm.md](module-animfsm.md) |
| `physics2d` | Vollwertiger 2D-Starrkörper-Solver (Rapier2D): Stapeln, Werfen, Rollen | [module-physics2d.md](module-physics2d.md) |
| `physics3d` | Vollwertiger 3D-Starrkörper-Solver (Rapier3D) | [module-physics3d.md](module-physics3d.md) |
| `audio` (Modulatoren) | LFO und Tweener auf dem Audio-Thread: Tremolo, Wobble, Filter-Sweeps ohne Nachrechnen pro Frame | [module-audio-modulatoren.md](module-audio-modulatoren.md) |
| `mqtt` | MQTT-3.1.1-Client — das Pub/Sub-Protokoll der IoT-/Maker-Welt | [module-mqtt.md](module-mqtt.md) |
| `firmata` | Arduino-/ESP32-Pins direkt steuern, ohne eigenen Sketch | [module-firmata.md](module-firmata.md) |

### Werkzeug

- **[Code-Editor](editor.md)** — Tastenkürzel, Snippets, Sidebar, Run/Debug/Profile, Find in Project
- **[Sprite-Editor (`dhsprites`)](sprite-editor.md)** — Pixel-Art-Editor mit Multi-Frame, Animation, Onion-Skin, Sheet/Atlas/GIF-Export, Palette-Tools
- **[Tilemap-Editor (`dhtilemap`)](tilemap-editor.md)** — Level bauen, Tiled-JSON lesen und schreiben
- **[Form-Designer (`dhform`)](form-designer.md)** — Oberflächen zusammenklicken, Xojo-Stil, F5 startet sie
- **[Partikel-Editor (`dhparticles`)](particle-editor.md)** — Effekte live einstellen, mit Preset-Bibliothek
- **[Animations-Editor (`dhanim`)](anim-editor.md)** — Zustandsautomat für Sprite-Animationen
- **[Notenblatt-Editor (`dhscore`)](score-editor.md)** — Noten setzen statt Tracker-Zeilen füllen
- **[Tracker](tracker.md)** und **[SFX-Generator](sfx-generator.md)** — Musik und Geräusche (beide im Audio Studio)
- **[Sprachserver + VSCode](lsp.md)** — dieselbe Diagnose in fremden Editoren
- **[Eingabe aufzeichnen](automation.md)** — Demo-Modus, nachspielbare Fehlerberichte, automatische Spieltests
- **[Web-Playground](web-playground.md)** — dhrt als WebAssembly, ein Link genügt

### Interna

Arbeitsnotizen, keine Anwender-Doku — sie erklären, warum etwas so ist, wie es ist:

- **[Stolpersteine](stolpersteine.md)** — Reibungspunkte der Sprache, beim Schreiben des Lehrbuchs gesammelt
- **Doku-Prüfer** — `tools/pruef_docs.py` schickt jeden `basic`-Block durch den Compiler, `tools/pruef_doku_aussagen.py` prüft Befehlsnamen in Tabellen/Fließtext und die Zählungen im README. Beide hängen an der Testsuite
- **[Rust-Front-End-Portierung](rust-frontend-port.md)** und **[Runtime-Migration](rust-runtime.md)** — wie `dhrt` entstand
- **[Entwurf: Namensräume](entwurf-namensraeume.md)** — WP I der Allzweck-Roadmap. Alle vier Stufen sind inzwischen gebaut; das Dokument bleibt als Protokoll der Entscheidungen
- **[Entwurf: Python-Parser entfernen](entwurf-python-parser-entfernen.md)** — gemessen, was noch am zweiten Parser hängt, und was ein Schnitt kostet
- **[Entwurf: Mengen](entwurf-set-builtins.md)** — der letzte offene WP-J-Punkt, nach dem Nachmessen neu zugeschnitten
- **[Entwurf: TASK_START](entwurf-task-start.md)** — GB-Code im Hintergrund; drei Wege, und der dritte umgeht das Send-Problem ganz
- **[Release 2026.8](release-2026.8.md)** — was in dieser Fassung neu ist
- **[Allzweck-Roadmap](allzweck-roadmap.md)** — was fehlt, damit man damit *alles* schreiben kann und nicht nur Spiele (aktuell, Audit 2026-08)
- **[Befehlssatz-Roadmap](befehlssatz-roadmap.md)** und **[GUI-Design-Notiz](gui-module-design.md)** — historisch, mit Lesehinweis
- **[Umbenennung GameBasic → Drachenhauch](umbenennung-drachenhauch.md)** — die Checkliste von 2026-08

## Erstes Programm

```basic
PRINT "Hallo, Drachenhauch!"

DIM name AS STRING
INPUT "Wie heisst du?", name
PRINT "Schoen dich zu sehen, ", name
```

Speichern als `hallo.dh`, dann:

```
.venv\Scripts\python.exe dhrun.py hallo.dh
```

Das venv muss dafür stehen — wie es entsteht, steht im
[README](../README.md#aus-dem-quelltext-arbeiten). Wer stattdessen das
System-Python nimmt, bekommt Fehler über fehlende Pakete.

## Erstes Spiel

Ein minimaler Game-Loop (Grafik läuft in der nativen Runtime dhrt):

```basic
SCREEN(320, 240, "Mein erstes Spiel", 2)

DIM x AS INTEGER
x = 160

WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(1073741904) THEN     ' LEFT
        x = x - 2
    END IF
    IF KEYPRESSED(1073741903) THEN     ' RIGHT
        x = x + 2
    END IF

    CLS(RGB(20, 20, 30))
    BOX(x, 100, x + 20, 120, RGB(255, 200, 80))
    FLIP()
    SLEEP(16)
WEND
```

Pfeiltasten bewegen das gelbe Rechteck. ESC oder Fenster schließen beendet (über `QUITREQUESTED()`).

## Konventionen in diesem Handbuch

- Codeblöcke zeigen lauffähigen Drachenhauch-Code (oft direkt aus `examples/` entnommen).
- Built-in-Signaturen werden kompakt notiert: `FUNKTION(arg1, arg2[, optional]) -> RÜCKGABETYP`. `[...]` markiert optionale Argumente.
- Type-Tags: `INTEGER`, `FLOAT`, `STRING`, `BOOLEAN`, `IMAGE`, `SOUND`, `FILE`, `MAP OF T`, `ARRAY OF T`, plus die externen Typen aus Modulen (z.B. `JSON_HANDLE`, `SPRITE`, `TWEEN`).
- Die Sprache ist **case-insensitive** für Schlüsselwörter und Built-ins (`PRINT`, `print`, `Print` sind gleich), aber identifier (eigene Variablen-/Funktionsnamen) bleiben unterscheidbar.

Viel Spaß!
