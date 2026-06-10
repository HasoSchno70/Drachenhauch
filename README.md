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

- **[Sprachreferenz](docs/sprache.md)** — Variablen, Typen, `ENUM`, `SELECT CASE`, Funktionen mit Defaults und Named Arguments, Klassen, Try/Catch, f-Strings, Coroutines (`YIELD` + `CORO_*`)
- **[Standard-Built-ins](docs/builtins-core.md)** — Math, Strings, Maps, File-I/O, …
- **[Grafik-Built-ins](docs/builtins-grafik.md)** — native Runtime (gbrt/raylib), Z-Layer, Sprite-Atlas, Asset-Preloader
- **[Performance](docs/PERFORMANCE.md)** — Bench-Zahlen + umgesetzte Optimierungen (Spec-Ops, IC, Typed Arrays, ECS Bulk-Ops, …)
- **[Module](docs/README.md#module)** — `json`, `db`, `tween`, `imgfx`, `particles`, `physics`, **`physics3d`** (echte 3D-Starrkörper-Physik via Rapier3D: Schwerkraft/Kollision/Restitution, `PHYS3D_*`, [examples/107_physics3d.gb](examples/107_physics3d.gb)), `camera`, `sprite`, **[`animfsm`](docs/module-animfsm.md)** (Animations-State-Machine, Unity-Mecanim-Stil: States + Parameter + Transitions aus `.gbanim`, Editor `gbanim`), `ui`, `scene`, `save`, `astar`, `ecs`, `vec2`, [`m3d`](docs/module-m3d.md) (3D-Mathe: VEC3/VEC4/QUAT/MAT4 + `MODEL_MATRIX`), `input`, `regex`, `audio`, `curves`, `net`
- **[Code-Editor](docs/editor.md)** — Tastenkürzel, Snippets, Minimap, Multi-Cursor, Sidebar, Run/Bench, Signature-Help, **Breadcrumbs** (Scope-Pfad), **Peek-Definition** (Alt+F12), **Split-View** (Strg+\\), **Debugger** (Breakpoints inkl. **bedingter** Breakpoints/Step/Variablen), **Profiler** (Hotpath pro Zeile/Funktion), **Git-Blame**-Panel, Welcome-Showcase (Demo-Galerie mit Screenshots)
- **[Sprite-Editor](docs/sprite-editor.md)** — Pixel-Art-Editor (`gbsprites`): Multi-Frame, Animation, Atlas-Export, Onion-Skin, Tile-Preview
- **[Partikel-Editor](docs/particle-editor.md)** — Effekt-Editor (`gbparticles`): Emitter-Parameter live tunen mit Echtzeit-Vorschau, **Preset-Bibliothek** (Werks- + eigene Presets), GB-Code-Export
- **[SFX-Generator](docs/sfx-generator.md)** — Retro-Soundeffekte (`gbsfx`, sfxr-Stil): Synth mit Pitch-Slide/Hüllkurve/Vibrato, **Preset-Bibliothek** (eigene Sounds speichern), Export WAV/GB-Code
- **[Tracker](docs/tracker.md)** — mehrspuriger Musik-Editor (`gbtracker`): 3 Ton-Kanäle + Drums, **fertige Instrument-Bibliothek** (Flügel/Orgel/Streicher/Bass/Glocke … + Drums, ein Keyboard-Sound pro Spur), **Sample-Instrumente** (WAV/OGG laden + über die Klaviatur resampeln, MOD/XM/IT-Stil), **Keymap/Multisample + Drumkit** (verschiedene Samples auf Tasten-Zonen), **SoundFont-Import** (`.sf2` — echte GM-/Hersteller-Instrumente), mehrere Patterns einstellbarer Länge + Song-Arrangement, **Effekt-Spalten** (Lautstärke + Pitch-Slide/Portamento pro Note), Projekt-Speichern/Laden (`.json`), Export als frame-basierter GB-Player **oder gerenderte WAV** (Song offline gemischt → `PLAYMUSIC`)
- **[Tilemap-/Level-Editor](docs/tilemap-editor.md)** — Tiles aufs Gitter malen (`gbtilemap`): mehrere Layer, Object-Layer, **mehrere Tilesets**, Per-Tile-Properties (`solid`/`damage`), Stift/Füllen/Rechteck/Pipette/**Auswahl** (Copy/Cut/Paste), Speichern/Laden als Tiled-JSON (`TILED_LOAD`), GB-Code-Renderer-Export
- **[Form-Designer (WYSIWYG)](docs/form-designer.md)** — visueller GUI-Designer im Xojo-Stil (`gbform`): Controls platzieren/konfigurieren, als `.gbform` speichern, per `GUI_LOAD` im eigenen Code nutzen oder mit F5 starten
- **[Animations-FSM-Editor](docs/anim-editor.md)** — Knoten-Graph für Animation-State-Machines im Unity-Mecanim-Stil (`gbanim`): States (an Sprite-Anim gebunden) + Parameter + Transitions mit Bedingungen visuell verdrahten, als `.gbanim` speichern, per `ANIM_FSM_LOAD` ([Modul `animfsm`](docs/module-animfsm.md)) nutzen, Live-Vorschau mit F5
- **[Language Server + VSCode-Extension](docs/lsp.md)** — GameBasic in jedem LSP-Editor: Syntax-Highlighting, Diagnostics, Completion, Hover, Goto-Definition, References, Outline (`py -m gamebasic.lsp`, `vscode-gamebasic/`)
- **[Web-Playground](docs/web-playground.md)** — `gbrt` als WebAssembly im Browser: Quelle im `<textarea>` tippen → gbrt kompiliert **im Browser** (kein Pyodide) → **Konsole UND animierte Grafik im `<canvas>`** (Render-Loop yieldet pro Frame, kein Tab-Freeze). **Teilbare Links** (Quelle im URL-Hash → öffnen = sehen + starten). Build `rust/build_wasm.py`, Harness `web/`

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
| `98_coroutines.gb` | **Coroutines/YIELD**: Generatoren, `FOR EACH`-Drain, send/return-Dialog, `CORO_RESULT`, Methoden-Coroutine |

## Architektur

Pipeline: **Source → Preprocessor → Lexer → Parser → AST**, dann wahlweise **Tree-Walker** (Interpreter, Python) oder **Bytecode → native Runtime `gbrt`** (Rust). Die Python-Toolchain (Lexer/Parser/Compiler) ist für beide gemeinsam.

`gbrun.py --bench <datei>` läuft Tree-Walker und `gbrt` nebeneinander und vergleicht Output — die Kerngarantie: identische Semantik in beiden Pfaden (Sweep in `tests/test_gbrt_parity.py`).

> **Konsolidierung:** Früher gab es zwei zusätzliche Bytecode-VMs in Python (Python-VM, Cython-VM). Beide wurden **entfernt** — `gbrt` hat sie als schnellen/Produktions-Pfad abgelöst. Es bleiben zwei Pfade: Tree-Walker (Referenz) + `gbrt` (nativ).

**Native Rust-Runtime (raylib) — der Produktionspfad.** Der Python-Compiler serialisiert den Bytecode nach `.gbc` (`py -m gamebasic.serialize`), ein standalone Rust-Crate (`rust/gb_runtime/`, Binary `gbrt`) führt ihn nativ aus. Skalare, Strings, Arrays, Maps, Tupel, OOP (Klassen/Methoden/Properties/Operatoren), Slicing, Comprehensions, TRY/THROW und die puren Builtins laufen bit-identisch zum Tree-Walker (~25–35× schneller). **Grafik via raylib** (feature-gated): 2D-Primitive, Text, Bilder, Input, headless per Screenshot verifiziert (`rust\build_runtime.py`). **Dev-Loop:** `gbrun.py --native <datei.gb>` kompiliert und startet nativ in einem Befehl; Laufzeitfehler zeigen `datei.gb:Zeile`. **3D** (Modul `g3d`, native-only): Kamera + Würfel/Kugel/Zylinder/Kegel/Ebene/Linien/Gitter über raylibs `begin_mode3D` ([examples/82_3d_intro.gb](examples/82_3d_intro.gb)), plus **3D-Modelle** — `LOADMODEL` (OBJ/GLTF), prozedurale `MESH_*` (Cube/Sphere/Cylinder/Torus/Knot/Plane/**Heightmap-Terrain**), `MODEL`/`MODEL_EX`/`MODEL_WIRES`, `MODEL_TEXTURE`, **Skelett-Animation** geriggter GLTF/IQM (`MODEL_LOAD_ANIMS` + `MODEL_ANIMATE`, [examples/108_skeletal_anim.gb](examples/108_skeletal_anim.gb)), **Billboards** (`BILLBOARD`), **Ray-Kollision/Picking** (`RAY_HIT_BOX`/`SPHERE`, `PICK_BOX`/`SPHERE`), **Beleuchtung** (Blinn-Phong, bis 4 Lichter: `LIGHT_DIRECTIONAL`/`LIGHT_POINT` + `MODEL_LIT`) und **Kamera-Modi** (`CAMERA3D_UPDATE` orbital/first-person) ([examples/88_3d_models.gb](examples/88_3d_models.gb), [examples/89_heightmap.gb](examples/89_heightmap.gb), [examples/90_billboards_picking.gb](examples/90_billboards_picking.gb), [examples/91_lighting.gb](examples/91_lighting.gb)). **Standalone-Export:** `gbrun.py --export <datei.gb>` (oder Editor *Export → .exe*, Ctrl+F6) bündelt Bytecode + `assets/` in eine eigenständige `.exe`, die ohne Python läuft (gbrt + angehängter Bytecode). **Audio** nativ (Core): `LOADSOUND`/`PLAYSOUND`/`STOPSOUND` + `PLAYMUSIC`/`STOPMUSIC` über raylib. **Retained-GUI** (`gui`) nativ: Fenster + Button/Label/Checkbox/Slider/TextInput/Panel/**Table** (scrollbar, selektierbar), Theme, Drag/Z-Order/Fokus, FUNCREF-Callbacks. Tabellen (`UI_TABLE` + `GUI_TABLE`) nativ. **Game-Loop** (`DELTA`/`FPS`/`SETFPS`), **GPU-Shader/Post-Processing** (`SHADER_LOAD`/`POSTFX` — CRT/Bloom/Vignette) und **TTF-Fonts** (`LOADFONT`/`SETFONT`/`TEXT_SPACING`, [examples/87_ttf_fonts.gb](examples/87_ttf_fonts.gb)) ebenfalls nativ. **Gamepad** nativ (`INPUT_JOY_COUNT/NAME/AXIS` + `JOY_BUTTON_*`/`JOY_DPAD_*`-Bindings über raylib) und **Tiefen-Fog** für beleuchtete Szenen (`LIGHT_FOG`, [examples/92_fog.gb](examples/92_fog.gb)). **Shadow-Mapping** (`SHADOW_ENABLE`/`SHADOW_AREA`/`SHADOW_TARGET` — directional Schlagschatten mit PCF, [examples/93_shadows.gb](examples/93_shadows.gb)) und **Normal-Mapping** (`MODEL_TEXTURE_NORMAL` — Pro-Pixel-Oberflächendetail via TBN, [examples/94_normalmap.gb](examples/94_normalmap.gb)). **PBR** (Cook-Torrance: die native Beleuchtung ist physically-based; `MODEL_PBR` für Metalness/Roughness, [examples/95_pbr.gb](examples/95_pbr.gb)) inkl. **Emissive/Neon-Glow** (`MODEL_EMISSIVE` — Eigenleuchten pro Modell, mit Bloom-`POSTFX` echter Glow, [examples/110_emissive_glow.gb](examples/110_emissive_glow.gb)) inkl. **Image-Based-Lighting** — analytisch (`LIGHT_ENV`, [examples/96_ibl.gb](examples/96_ibl.gb)) **und echtes HDR-Cubemap-IBL** (`LIGHT_ENV_HDR` — lädt ein `.hdr`, berechnet Irradiance/Prefilter/BRDF-LUT; Metalle spiegeln die echte Umgebung, [examples/99_ibl_hdr.gb](examples/99_ibl_hdr.gb)). **Fullscreen-Showcase:** [examples/97_pbr_reactor.gb](examples/97_pbr_reactor.gb) („PBR REACTOR") — FFT-reaktiver Ring aus Chrom-PBR-Kugeln, IBL, Schatten, Bloom + Stereo-Techno (CC0). **Coroutines/`YIELD`** laufen ebenfalls nativ — die Rust-VM suspendiert via Frame-Snapshot (keine Threads, raylib-Main-Thread-sicher, deterministisch), bit-identisch zu den Python-Pfaden inkl. Standalone-`.exe` ([examples/98_coroutines.gb](examples/98_coroutines.gb)). **Module-Voll-Portierung abgeschlossen** — alle früher Python-only-Module laufen jetzt nativ: `regex`, `tiled`, `tile_collide`, `controller`, erweitertes `audio`, sowie feature-gated `db` (rusqlite), `net` (std::net), `html` (ureq) und Hardware/IoT `serial` (serialport), `usb` (hidapi), `wifi` (netsh), `bt` (btleplug/BLE). Damit braucht nur noch der Editor Python; der Rest läuft komplett nativ (`build_runtime.py --hardware` / `--full` für die schweren Module). Plan & Status in [docs/rust-runtime.md](docs/rust-runtime.md).

**Front-End-Portierung nach Rust — abgeschlossen.** Die komplette Toolchain (Lexer → Parser → Compiler → Preprocessor) wurde nach Rust portiert, jede Stufe per Output-Parität gegen den Python-Tree-Walker verifiziert. **`gbrt run datei.gb` ist ein eigenständiger End-to-End-Lauf ohne Python:** preprocesst (`IMPORT`-Auflösung von Quelldateien und Built-in-Modulen), lext, parst, kompiliert und führt aus — Skalare/Arithmetik/Kontrollfluss, Arrays/Maps, Funktionen, Klassen/OOP, `SELECT`/`FOR EACH`/Tupel/`WITH`/`TRY`/Slicing/Comprehensions/Coroutinen. Wie `gbrun.py` wird ins Datei-Verzeichnis gewechselt, sodass relative `IMPORT`- und Asset-Pfade stimmen (`gbrt datei.gb` ohne `run` funktioniert genauso; `.gbc`-Dateien laufen weiter den direkten VM-Pfad). Debug-Einstiege `gbrt --tokens`/`--ast`/`--preprocess`/`--runsrc`. **Selbst-Export ohne Python:** `gbrt --export datei.gb` kompiliert die Quelle selbst und bündelt sie zu einer eigenständigen `.exe` (hängt den Bytecode an eine Kopie der Runtime, kopiert `assets/`). Aliasierte Modul-Imports (`IMPORT "json" AS j` → `J_PARSE`, `DIM h AS J_HANDLE`) funktionieren ebenfalls nativ. Damit ist auch der **Web-Playground ein reines Rust-WASM**, das die Quelle im Browser kompiliert (kein Pyodide): `rust/build_wasm.py datei.gb` erzeugt `web/gbrt.{js,wasm}` mit eingebetteter Quelle (emscripten-Toolchain auf Windows wird automatisch verdrahtet). **Konsole und animierte Grafik laufen im Browser** — der GB-Render-Loop yieldet pro Frame via ASYNCIFY (`emscripten_sleep(0)` in `flip()`), sodass `WHILE … FLIP() … WEND` den Tab nicht einfriert; **teilbare Links** packen die Quelle in den URL-Hash. Plan & Stufen in [docs/rust-frontend-port.md](docs/rust-frontend-port.md).

Architektur-Details und Erweiterungs-Hinweise in [CLAUDE.md](CLAUDE.md).

## Tests

```
.venv\Scripts\python.exe -m pytest tests/
```

Über 1580 Tests — Built-ins, alle Module, Sprach-Konstrukte, Editor-Features, Example-Smoke-Tests (Tree-Walker) und der TW↔`gbrt`-Paritäts-Sweep (`tests/test_gbrt_parity.py`, skippt ohne gebautes `gbrt`).

## Lizenz

Privat.
