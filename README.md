# GameBasic

Ein BASIC-Dialekt mit Pascal-strikter Typisierung und OOP, ausgelegt für Spiele. Programme laufen wahlweise im Tree-Walker, in der Python-VM oder in der Cython-Native-VM — alle drei produzieren bit-identischen Output.

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

## Handbuch

Vollständige Doku im [docs/](docs/README.md)-Ordner:

- **[Sprachreferenz](docs/sprache.md)** — Variablen, Typen, `ENUM`, `SELECT CASE`, Funktionen mit Defaults und Named Arguments, Klassen, Try/Catch, f-Strings
- **[Standard-Built-ins](docs/builtins-core.md)** — Math, Strings, Maps, File-I/O, …
- **[Grafik-Built-ins](docs/builtins-grafik.md)** — Pygame-Backend, Z-Layer, Sprite-Atlas, Asset-Preloader
- **[Performance](docs/PERFORMANCE.md)** — Bench-Zahlen + umgesetzte Optimierungen (Spec-Ops, IC, Typed Arrays, ECS Bulk-Ops, …)
- **[Module](docs/README.md#module)** — `json`, `db`, `tween`, `imgfx`, `particles`, `physics`, `camera`, `sprite`, `ui`, `scene`, `save`, `astar`, `ecs`, `vec2`, `input`, `regex`, `audio`, `curves`, `net`
- **[Code-Editor](docs/editor.md)** — Tastenkürzel, Snippets, Minimap, Multi-Cursor, Sidebar, Run/Bench
- **[Sprite-Editor](docs/sprite-editor.md)** — Pixel-Art-Editor (`gbsprites`): Multi-Frame, Animation, Atlas-Export, Onion-Skin, Tile-Preview

## Beispiele

`examples/` enthält über 50 lauffähige Demos, von "Hallo Welt" bis zum kompletten Mini-Spiel:

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

## Architektur

Pipeline: **Source → Preprocessor → Lexer → Parser → AST**, dann wahlweise **Tree-Walker** (Interpreter) oder **Bytecode → VM** (Python oder Cython).

`gbrun.py --bench <datei>` läuft alle drei Pfade nebeneinander und vergleicht Output — die Kerngarantie: identische Semantik überall.

**In Arbeit — native Rust-Runtime (raylib):** ein vierter Ausführungspfad. Der Python-Compiler serialisiert den Bytecode nach `.gbc` (`py -m gamebasic.serialize`), ein standalone Rust-Crate (`rust/gb_runtime/`, Binary `gbrt`) führt ihn nativ aus. Skalare, Strings, Arrays, Maps, Tupel, OOP (Klassen/Methoden/Properties/Operatoren), Slicing, Comprehensions, TRY/THROW und die puren Builtins laufen bit-identisch zur Python-VM (~25–35× schneller). **Grafik via raylib** (feature-gated): 2D-Primitive, Text, Bilder, Input, headless per Screenshot verifiziert (`rust\build_runtime.py`). **Dev-Loop:** `gbrun.py --native <datei.gb>` kompiliert und startet nativ in einem Befehl; Laufzeitfehler zeigen `datei.gb:Zeile`. **3D** (Modul `g3d`, native-only): Kamera + Würfel/Kugel/Zylinder/Kegel/Ebene/Linien/Gitter über raylibs `begin_mode3D` ([examples/82_3d_intro.gb](examples/82_3d_intro.gb)). Offen: 3D-Modelle (OBJ/GLTF), Editor-Export. Plan & Status in [docs/rust-runtime.md](docs/rust-runtime.md).

Architektur-Details und Erweiterungs-Hinweise in [CLAUDE.md](CLAUDE.md).

## Tests

```
.venv\Scripts\python.exe -m pytest tests/
```

Über 1170 Tests — Built-ins, alle Module, Sprach-Konstrukte, Editor-Features, Example-Smoke-Tests, Bench-Equivalence aller drei Pfade.

## Lizenz

Privat.
