# GameBasic Handbuch

Vollständige Referenz für die Sprache GameBasic, alle eingebauten Befehle und alle Built-in-Module.

GameBasic ist ein BASIC-Dialekt mit Pascal-strikter Typisierung und OOP, ausgelegt für Spiele. Programme laufen wahlweise im Tree-Walker, in der Python-VM oder in der Cython-Native-VM — alle drei produzieren identischen Output.

## Inhalt

### Die Sprache

- **[Sprachreferenz](sprache.md)** — Variablen, Typen, Operatoren, Kontrollfluss (IF, SELECT CASE, WHILE, FOR), Funktionen, Klassen, Try/Catch, Imports

### Eingebaute Befehle

- **[Standard-Built-ins](builtins-core.md)** — Math, Strings, Bitwise, Maps, File-I/O, Konvertierung, Zeit/Random
- **[Grafik-Built-ins](builtins-grafik.md)** — SCREEN, CLS, BOX, CIRCLE, LOADIMAGE, Sound, Tilemap, Eingabe, **`LOAD_ASSETS`** (Bulk-Preloader), **Sprite-Atlas + Batch-Draw**, **Z-Layer-Rendering**
- **[Performance](PERFORMANCE.md)** — Bench-Zahlen aller drei Pfade + Liste umgesetzter Optimierungen

### Module

Jedes Modul wird mit `IMPORT "<name>"` aktiviert und stellt eigene Befehle bereit.

| Modul | Was es kann | Doku |
|---|---|---|
| `json` | JSON parsen, Werte über Pfad-Notation lesen | [module-json.md](module-json.md) |
| `db` | SQLite-Datenbank: CREATE/INSERT/SELECT, Transaktionen | [module-db.md](module-db.md) |
| `tween` | Werteinterpolation über Zeit (Animationen, Easings) | [module-tween.md](module-tween.md) |
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
| `bt` | Bluetooth Low Energy (BLE) via `bleak` — Scan, Connect, Read/Write Characteristics | [module-bt.md](module-bt.md) |
| `serial` | RS-232 / USB-COM via `pyserial` — Open, Read/Write, Available, Flush, Timeout | [module-serial.md](module-serial.md) |
| `usb` | USB-HID via `hidapi` — Custom-Controller, Maker-Boards, Programmer | [module-usb.md](module-usb.md) |
| `wifi` | WiFi-Management (Windows-only via `netsh wlan`): Scan, Connect, Disconnect, Profile | [module-wifi.md](module-wifi.md) |
| `tiled` | [Tiled](https://www.mapeditor.org/)-Map-Loader (JSON). Tile-Layer, Object-Layer, Tile- + Object-Properties | [module-tiled.md](module-tiled.md) |
| `tile_collide` | Box-vs-Tilemap-Kollision: TILE_SWEEP_X/Y mit separat-Achsen-Pattern. Klassischer Platformer | [module-tile-collide.md](module-tile-collide.md) |
| `controller` | Character-Controller mit Coyote-Time, Jump-Buffer, Variable-Jump-Height. Klassische "feel-good"-Platformer-Mechanik | [module-controller.md](module-controller.md) |

### Werkzeug

- **[Code-Editor](editor.md)** — Tastenkürzel, Snippets, Sidebar, Run/Bench, Find in Project
- **[Sprite-Editor (`gbsprites`)](sprite-editor.md)** — Pixel-Art-Editor mit Multi-Frame, Animation, Onion-Skin, Sheet/Atlas/GIF-Export, Palette-Tools

## Erstes Programm

```basic
PRINT "Hallo, GameBasic!"

DIM name AS STRING
INPUT "Wie heisst du?", name
PRINT "Schoen dich zu sehen, ", name
```

Speichern als `hallo.gb`, dann:

```
.venv\Scripts\python.exe gbrun.py hallo.gb
```

## Erstes Spiel

Ein minimaler Game-Loop mit Pygame:

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

- Codeblöcke zeigen lauffähigen GameBasic-Code (oft direkt aus `examples/` entnommen).
- Built-in-Signaturen werden kompakt notiert: `FUNKTION(arg1, arg2[, optional]) -> RÜCKGABETYP`. `[...]` markiert optionale Argumente.
- Type-Tags: `INTEGER`, `FLOAT`, `STRING`, `BOOLEAN`, `IMAGE`, `SOUND`, `FILE`, `MAP OF T`, `ARRAY OF T`, plus die externen Typen aus Modulen (z.B. `JSON_HANDLE`, `SPRITE`, `TWEEN`).
- Die Sprache ist **case-insensitive** für Schlüsselwörter und Built-ins (`PRINT`, `print`, `Print` sind gleich), aber identifier (eigene Variablen-/Funktionsnamen) bleiben unterscheidbar.

Viel Spaß!
