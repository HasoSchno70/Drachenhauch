# GameBasic

BASIC-Dialekt mit Pascal-strikter Typisierung und OOP, ausgelegt für Spiele.
Drei austauschbare Ausführungspfade: Tree-Walking-Interpreter, Python-VM (Bytecode),
Cython-VM (kompilierte Native-Variante). Alle drei produzieren bit-identischen
Output — siehe `gbrun.py --bench <datei.gb>`.

## Verzeichnisstruktur

```
gamebasic/
  __main__.py            # py -m gamebasic <file> (mit Preprocessor)
  lexer.py / tokens.py   # Tokenisierung
  parser.py / ast_nodes.py
  interpreter.py         # Tree-Walker + alle Built-ins
  compiler.py            # AST -> Bytecode (mit Type-Inference + Constant Folding)
  bytecode.py            # Opcodes
  vm.py                  # Python-VM
  vm_native.pyx/.c/.pyd  # Cython-VM (gleiche Semantik)
  array_native.pyx/.pyd  # cdef _GBArray mit typed memoryviews (Hot-Path)
  graphics.py            # Pygame-Wrapper, Camera, Z-Layer-System, Asset-Cache, Lazy-Init
  preprocess.py          # IMPORT-Auflösung (Source UND Built-in-Module)
  builtins_registry.py   # @builtin / @graphics_builtin Decorators
  modules/               # Built-in-Module (json, db, tween, imgfx, particles, camera, ecs, ...)
  modules/ecs_native.pyx # cdef _World/_Component mit Bulk-System-Ops (Native-ECS)
gbrun.py                 # CLI mit Editor, --vm, --bench, --tokens, --ast
examples/*.gb            # Demos -- 1-76+, inkl. bench_*.gb fuer Performance-Vergleiche
tests/                   # pytest-Tests (1100+, alle drei Pfade bit-identisch)
```

## Architektur-Pipeline

```
Source.gb  →  preprocess.process()  →  Lexer  →  Parser  →  AST
                                                              │
              ┌───────────────────────────────────────────────┤
              ▼                                               ▼
        Interpreter.run(ast)                         Compiler.compile(ast)
        (Tree-Walking)                                       │
                                                             ▼
                                                   Module mit Bytecode
                                                             │
                                              ┌──────────────┴──────────────┐
                                              ▼                             ▼
                                         vm.VM().run()              vm_native.VM().run()
                                         (Python)                   (Cython)
```

`gbrun.py` ist Default-Einstiegspunkt (macht `os.chdir(file.parent)` für relative
Asset-Pfade). `py -m gamebasic` funktioniert auch, wechselt aber nicht ins
Datei-Verzeichnis — Programme mit `LOADIMAGE("assets/...")` brauchen `gbrun.py`.

## Built-ins erweitern (innerhalb interpreter.py)

```python
from .builtins_registry import builtin, graphics_builtin

@builtin("BEEP", arity=2, types=("num", "num"))
def _b_beep(freq, ms):
    ...

@graphics_builtin("FOO", arity=(1, 3), types=None)  # variable arity, eigene Validierung
def _g_foo(g, *args):
    ...
```

**type-Specs:**
- `"num"` — int oder float, kein bool
- `"int"` — strikt int
- `"intish"` — akzeptiert num, gibt int zurück (Konvenienz für Grafik-Koordinaten)
- `"str"`, `"bool"`, `"any"`

**arity:** int (exakt), `(min, max)` (Range), `(min, None)` (kein Maximum), oder `None` (selbst prüfen).

**Wichtig:** `types` ist nur bei fixer arity erlaubt. Bei variable arity selbst validieren.

`@builtin` füllt das `BUILTINS`-Dict in `builtins_registry`, das von `interpreter.py`,
`vm.py` und `vm_native.pyx` als Wahrheit konsumiert wird. Niemand muss seine Imports ändern.

## Built-in-Module schreiben

### Modul `gui` (Retained-Mode-GUI)

Persistente Fenster/Widgets (externe Typen `GUI_WINDOW`/`GUI_WIDGET` via `register_type`). Aufbau einmalig, pro Frame `GUI_UPDATE()` + `GUI_DRAW()`; Events per Polling (`GUI_CLICKED`/`CHECKED`/`VALUE`/`TEXT`/`HOVERED`) **oder** FUNCREF-Callbacks (`GUI_ON_CLICK`/`GUI_ON_CHANGE`). Widgets: Button, Label, Checkbox, Slider, TextInput, Panel, **Table** (`GUI_TABLE` — fixierte Kopfzeile, V/H-Scroll via Mausrad+Scrollbalken, persistente Zeilen-Selektion; Daten via `GUI_TABLE_HEADERS/ROWS/COL_WIDTHS`, Polling via `GUI_TABLE_SELECTED/CLICKED/ROW_COUNT`; Layout aus einer Quelle `_table_geom` für Hit-Test + Zeichnen). Window-Drag an der Titelleiste, Z-Order (Klick bringt nach vorne), Fokus, Schliessen-Button, Cyan-Theme (programmierbar). Konstruktoren/Getter sind `@builtin`, nur `GUI_UPDATE`/`GUI_DRAW` sind `graphics_builtin`. Komplement zum Immediate-Mode-`ui`-Modul (dort `UI_WINDOW_BEGIN/END` + `UI_TABLE` mit `UI_TABLE_SELECTED`/`SET_SELECTED`/`HEADER_CLICK`). Doku `docs/module-gui.md`, Demos `examples/45_gui.gb` + `examples/81_table_select.gb`, Tests `tests/test_modules_gui.py`.


Eine Datei in `gamebasic/modules/` reicht — keine Änderung an Lexer, Parser,
Interpreter, VM, Compiler. Beim ersten `IMPORT "modulname"` zur Laufzeit geladen.

```python
# gamebasic/modules/foo.py
from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type

class _FooHandle:
    __slots__ = ("data",)
    def __init__(self, data): self.data = data

register_type("foo_handle", _FooHandle)   # erlaubt DIM x AS FOO_HANDLE in GB

@builtin("FOO_NEW", arity=1, types=("str",))
def _new(s):
    return _FooHandle(s)
```

In GB:
```basic
IMPORT "foo"
DIM x AS FOO_HANDLE
x = FOO_NEW("hallo")
```

**IMPORT-Auflösung in [preprocess.py](gamebasic/preprocess.py):**
1. Existiert `<name>.gb` im aktuellen Verzeichnis? → textuelles Inkludieren (Quellcode-Modul).
2. Sonst: ist `<name>` ein gültiger Modul-Name (alphanumerisch, kein Slash, keine `.gb`-Endung)? → `gamebasic.modules.<name>` laden.
3. Sonst: Fehler.

So kann ein User ein eigenes `json.gb` schreiben, das Vorrang vor dem Built-in hat
— hilfreich für Tests.

## Verfügbare Built-in-Module

| Modul | Funktionen (Auswahl) | Externer Typ |
|---|---|---|
| `json` | `JSON_PARSE/LOAD/STRINGIFY`, `JSON_GET_STRING/INT/FLOAT/BOOL`, Pfad-Notation `"user.name"` / `"items.0"` | `JSON_HANDLE` |
| `db` | SQLite. `DB_OPEN/CLOSE`, `DB_EXEC/QUERY` mit `?`-Binding, `DB_NEXT`, `DB_GET_*`, `DB_BEGIN/COMMIT/ROLLBACK` | `DB_CONN`, `DB_RESULT` |
| `tween` | Werteinterpolation. 13 Easings (`linear`, `out_bounce`, `out_elastic`, …), Pause/Resume/Reverse | `TWEEN` |
| `imgfx` | `IMAGE_SCALE/ROTATE/FLIP/TINT/COPY` — immutable, geben neues IMAGE zurück | — |
| `particles` | Emitter mit Velocity/Lifetime/Gravity/Color/Size/Fade. `PARTICLE_EMIT/UPDATE/DRAW`. NumPy-vektorisiert. **Render-Modi** `PARTICLE_SET_MODE` (`circle`/`pixel`/`square`/`streak`/`glow`-additiv) + **Farbverlauf** `PARTICLE_SET_COLOR_END` (Start→End ueber die Lebenszeit, z.B. Feuer gelb→rot). | `PARTICLE_SYSTEM` |
| `physics` | Pure Functions: AABB-/Circle-Collision, Distance, Reflect, Normalize, Ray-Cast (Box+Circle). Kein State. Plus **Broadphase** (`PHYSICS_BROAD_NEW/ADD/QUERY/PAIR_A/PAIR_B`): O(n)-Kollisionspaare fuer viele Kreis-Entities (Uniform-Grid, nativ via `gb_native`). | `PHYSICS_BROAD` |
| `camera` | World-Translation+Zoom für **alle** Drawing-Befehle. `CAMERA_SET/RESET/FOLLOW`, `CAMERA_S2W_X/Y` | — |
| `sprite` | Animiertes Sheet-basiertes Sprite. Position+Velocity, benannte Animationen mit FPS, `PLAY`/`PLAY_ONCE`, Flip, AABB-Kollision | `SPRITE` |
| `ui` | Immediate-Mode-UI. `UI_LABEL`, `UI_BUTTON`, `UI_CHECKBOX`, `UI_SLIDER` mit String-IDs für State. Pflicht: `UI_END_FRAME()` vor `FLIP()`. | — |
| `scene` | Stack-basierter Scene-Manager. `SCENE_PUSH/POP/SWITCH/CURRENT`, pro-Scene-Daten via `SCENE_SET_INT/FLOAT/STRING/BOOL` + `_OR`-Variante. | — |
| `save` | Persistente Save-Slots, JSON-Backend, Versionsfeld. `SAVE_NEW/LOAD/LOAD_OR_NEW/WRITE`, `SAVE_SET/GET_INT/FLOAT/STRING/BOOL`. | `SAVE_HANDLE` |
| `astar` | A*-Pathfinding auf Tile-Grid. `ASTAR_NEW/SET_WALL/FIND/PATH_X/PATH_Y`. Manhattan/Euclid/Chebyshev, Diagonal-Toggle, Anti-Cornercutting. | `ASTAR_GRID` |
| `vec2` | 2D-Vektor mit Operator-Overloading (`+`, `-`, `*`, `/`, `=`, `<>`). `VEC2_NEW/X/Y/LENGTH/NORMALIZE/DOT/CROSS/DISTANCE/LERP/PERP/REFLECT/ANGLE/FROM_ANGLE`. Immutable. | `VEC2` |
| `input` | Action-basiertes Input-Mapping mit Edge-Detection. `INPUT_BIND/UNBIND/UPDATE`, `INPUT_HELD/PRESSED/RELEASED/AXIS/BOUND`. Multi-Key-Bindings. **Gamepad-Support**: `JOY_BUTTON_A..Y`, `JOY_DPAD_*` als Bind-Codes, `INPUT_JOY_AXIS(slot, "left_x")` mit Deadzone. | — |
| `regex` | Python-kompatible Pattern-Matching. `REGEX_MATCH/TEST/FIND/FIND_ALL/REPLACE/REPLACE_ONCE/SPLIT`. Pattern-Cache fuer wiederholte Aufrufe. | — |
| `audio` | Erweiterte Audio-API ueber pygame.mixer. Channels, Pause/Resume/Fade, Stereo-Pan, Music-Position. Tone-Generation (`AUDIO_TONE`/`AUDIO_NOISE`) mit Sine/Square/Saw/Triangle/Noise. Liefert kompatible `SOUND`-Objekte (auch fuer `PLAYSOUND` nutzbar). | `AUDIO_CHANNEL` |
| `curves` | Animation-Kurven (komplementaer zu `tween`'s Easings): `CURVE_BEZIER/BEZIER2`, `CURVE_CATMULL/CATMULL2`, `CURVE_HERMITE`, `CURVE_LERP`, `CURVE_SMOOTHSTEP`, `CURVE_SMOOTHERSTEP`. Pure Functions, kein State. | — |
| `net` | TCP + UDP via stdlib-Sockets (cross-platform). Default non-blocking fuer Game-Loops. `NET_TCP_LISTEN/ACCEPT/CONNECT`, `NET_SEND/RECV`, `NET_UDP_BIND/SEND/RECV`. Encoding: UTF-8. | `NET_LISTENER`, `NET_SOCKET`, `NET_UDP` |
| `ecs` | Entity-Component-System. World mit Entity-IDs (INTEGER) und benannten typed Components (INT/FLOAT/STRING/BOOL/OBJ). Query 1/2/3-fach via Component-Intersection. `ECS_NEW_ENTITY`, `ECS_ADD_INT`, `ECS_QUERY2`, etc. Plus **Bulk-System-Ops** (`ECS_INTEGRATE_FLOAT`, `ECS_SCALE_FLOAT`, `ECS_FILL_*`, `ECS_CLAMP_FLOAT`, `ECS_REMOVE_DEAD`, `ECS_COUNT_WITH`) — siehe eigener Abschnitt unten. Native cdef-Implementation in `ecs_native.pyx`. | `ECS_WORLD` |
| `html` | HTTP-GET/POST/DOWNLOAD + HTML-Parsing (pure stdlib). `HTTP_GET/POST/DOWNLOAD`, `HTTP_STATUS/HEADER`, `URL_ENCODE/DECODE`, `HTML_TEXT`, `HTML_FIND_ALL`. | — |
| `bt` | Bluetooth Low Energy (BLE) via `bleak`. Scan, Connect, Service/Characteristic-Listing, Read/Write/Notify auf Characteristics. Externer Dep, IoT/Sensor-Targets. | `BT_HANDLE` |
| `serial` | RS-232 / USB-COM via `pyserial`. `SERIAL_OPEN/READ/WRITE/READLINE/AVAILABLE/FLUSH/TIMEOUT`. | `SERIAL_HANDLE` |
| `usb` | USB-HID via `hidapi`. Maker-Boards, Programmer, Custom-Controller. `USB_LIST/OPEN/READ/WRITE/PRODUCT`. | `USB_HANDLE` |
| `wifi` | WiFi-Management (Windows-only via `netsh wlan`). `WIFI_SCAN/CONNECT/DISCONNECT/CURRENT/SIGNAL/PROFILES`. | — |
| `tiled` | Tiled-Map-Loader (JSON-Format, kein TMX). `TILED_LOAD`, Layer-/Tile-/Object-Access, Per-Tile/Per-Object-Custom-Properties (`solid`, `damage`, ...). Industriestandard fuer 2D-Level-Design. Plus **Bulk-Ops** fuer Generierung/Editor: `TILED_FILL_RECT`, `TILED_REPLACE`, `TILED_COUNT_GID`, `TILED_FLOOD_FILL` (Bucket-Fill, nativ via `gb_native`). | `TILED_MAP` |
| `tile_collide` | Box-vs-Tilemap-Kollision. `TILE_SWEEP_X/Y` mit separat-Achsen-Sweep-Pattern. Solid-Detection via `solid`-Property (mit Convention-Fallback). Klassische Platformer-Physik. Sweep nativ via `gb_native.TileCollider` (Solid-Maske einmal gespiegelt+gecacht), sonst Python-`_sweep_axis`. | — |
| `controller` | Character-Controller mit Coyote-Time, Jump-Buffer, Variable-Jump-Height. `CHAR_NEW/SET_INPUT/UPDATE`, `CHAR_X/Y/VX/VY`, `CHAR_ON_GROUND/WALL_LEFT/RIGHT`. Konfigurable Move-Speed, Jump-Velocity, Gravity, Coyote/Buffer-Frames, Variable-Jump-Cut. | `CHAR_CONTROLLER` |
| `g3d` | **3D-Grafik, NUR native Runtime** (`gbrt`/F6 — pygame kann kein 3D, F5 wirft klare Meldung). Immediate-Primitive: `CAMERA3D`, `CUBE`/`CUBE_WIRES`, `SPHERE`/`SPHERE_WIRES`, `CYLINDER` (Kegel via r_oben=0), `PLANE`, `LINE3D`, `POINT3D`, `GRID3D`. **3D-Modelle** (wiederverwendbare MODEL-Handles): `LOADMODEL` (OBJ/GLTF), prozedural `MESH_CUBE/SPHERE/CYLINDER/TORUS/KNOT/PLANE` + `MESH_HEIGHTMAP` (Terrain aus Graustufen-Image), zeichnen via `MODEL`/`MODEL_EX` (Achsen-Rotation)/`MODEL_WIRES`, `MODEL_TEXTURE` (Diffuse-Map aus LOADIMAGE). **Billboards** `BILLBOARD` (Textur zeigt zur Kamera) + **Ray-Kollision/Picking** `RAY_HIT_BOX`/`RAY_HIT_SPHERE` (Distanz oder -1) und `PICK_BOX`/`PICK_SPHERE` (Mausstrahl, Klick-Selektion). Render via raylib `begin_mode3D` beim FLIP (3D zuerst, 2D-HUD obenauf). Doku `docs/rust-runtime.md` (Schritt 6), Demos `examples/82_3d_intro.gb` + `examples/88_3d_models.gb` + `examples/90_billboards_picking.gb`. | — |

**Zusätzlich als Core-Graphics-Built-ins** (kein IMPORT noetig, registriert in
`interpreter.py` als `@graphics_builtin`):

| Bereich | Funktionen | Externer Typ |
|---|---|---|
| Asset-Cache | `LOAD_ASSETS(manifest.json)` — bulk-Preload mit Alias-Cache. `LOADIMAGE` / `LOADSOUND` cachen automatisch (rohem + abs Pfad). | — |
| Z-Layer | `LAYER_DEFINE(name, z)`, `LAYER(name)`, `LAYER_END()`, `LAYER_CLEAR(name)`. Layer-Surfaces mit SRCALPHA, FLIP composiert in z-Order und cleart. | — |
| Sprite-Atlas | `ATLAS_LOAD(manifest.json)` -> `SPRITE_ATLAS`. `ATLAS_DRAW(atlas, name, x, y)`. `BATCH_DRAW(...)` + `BATCH_FLUSH()` ueber `pygame.Surface.blits()`. Auto-Flush bei FLIP / Layer-Switch / Direct-Draw. | `SPRITE_ATLAS` |
| Bulk-Plot | `PLOTS(xs, ys, color)` — viele Pixel in EINEM Aufruf (vektorisiert via `pygame.surfarray`/numpy), `color` = INT (alle gleich) oder ARRAY OF INT (pro Pixel). Groessenordnungen schneller als `PLOT` in einer Schleife (Starfields, Punktwolken). | — |
| Bulk-Shapes | `BOXES(x1s,y1s,x2s,y2s,color)`, `CIRCLES(xs,ys,rs,color)`, `LINES(x1s,y1s,x2s,y2s,color)` — viele Shapes in EINEM Builtin-Call (spart den Dispatch pro Shape; pygame-Draw bleibt pro Shape). `color` = INT oder ARRAY. | — |
| Bulk-Tilemap | `TILED_FILL_RECT`, `TILED_REPLACE`, `TILED_COUNT_GID`, `TILED_FLOOD_FILL` (Bucket-Fill, nativ via `gb_native`) — siehe `tiled`-Modul. `DRAWTILEMAP` rendert intern via `blits()`-Batch (1 Call statt rows×cols). | — |
| Game-Loop | `DELTA()` — Sekunden seit letztem `FLIP` (framerate-unabhaengige Bewegung: `x = x + speed * DELTA()`). `FPS()` / `SETFPS(n)` (Ziel-Framerate, 0 = ungedrosselt). `SET_FULLSCREEN(an)`, `SETWINDOWTITLE(s$)`, `SAVESCREENSHOT(pfad$)`. Beide Pfade (pygame + native raylib). | — |
| Shader / Post-FX | **Nur native Runtime** (raylib/GPU): `SHADER_LOAD(pfad$_oder_glsl$)` -> SHADER-Handle (oder -1), `SHADER_SET(h, uniform$, f)` / `SHADER_SET2` (vec2) / `SHADER_SET3` (vec3), `POSTFX(h)` (Frame durch Fragment-Shader; -1 = aus). Szene -> RenderTexture -> Shader -> Screen. Im pygame-Pfad No-Op (Szene ohne Effekt). Beispiel-Shader `examples/assets/shaders/` (CRT/Bloom/Vignette), Demo `examples/86_postfx_shaders.gb`. | — |

Module mit eigenem Typ registrieren ihn lowercase (`register_type("json_handle", _JSONHandle)`),
GB-Code schreibt ihn in jeder Casing-Form (`DIM j AS JSON_HANDLE`).

## Convention: Wert-Typen in GB

| GB-Typ | Python-Typ | type-Spec |
|---|---|---|
| INTEGER | `int` (kein bool) | `"int"` |
| FLOAT | `float` | `"num"` (akzeptiert auch int) |
| STRING | `str` | `"str"` |
| BOOLEAN | `bool` | `"bool"` |
| Klasse / Externer Typ | Instanz / Handle | `"any"` (selbst prüfen) |
| ARRAY OF T | `_GBArray` | — (Parser-Form `array:T`) |
| MAP OF T | `_GBMap` | — (Parser-Form `map:T`) |
| FILE / IMAGE / SOUND | `_GBFile` / `_Image` / `_Sound` | — (eigene target-Strings) |
| SPRITE_ATLAS | `_SpriteAtlas` (image + frames-Dict) | — (eigener target-String `"sprite_atlas"`) |

**Bool ist KEINE Zahl** — `_check_num(True)` wirft, weil `isinstance(True, int)` zwar `True`
ist, aber `True` semantisch keine Zahl in GB ist.

## Camera-Wirkung auf Drawing

Wenn `CAMERA_SET` aufgerufen wurde, sind ab da alle Koordinaten in den
core-Grafik-Built-ins (`PLOT`, `LINE`, `BOX`, `RECT`, `CIRCLE`, `TEXT`,
`DRAWIMAGE*`, `DRAWTILEMAP`) **World-Koordinaten**. `TEXT` wird nur translatiert
(nicht gezoomt) — für scharfen HUD-Text vorher `CAMERA_RESET()`.

`PARTICLE_DRAW` ruft intern `g.circle()` und folgt der Camera automatisch.

## Build und Test

**Cython-Native-VM bauen:**
```
.venv\Scripts\python.exe setup.py build_ext --inplace
```
Notwendig nach Änderungen an `vm_native.pyx`. Wenn weggelassen, fällt `gbrun.py --vm`
auf die Python-VM zurück.

**Native Rust-Module bauen** (PyO3, separate Toolchain — `cargo` nötig):
```
.venv\Scripts\python.exe rust\build.py
```
Baut den Crate `rust/gb_native/` (ein Extension-Modul, PyO3 kompiliert nur einmal)
und legt `gamebasic/gb_native.pyd` neben die Cython-Module. Drei Beschleuniger-
Klassen, jede mit Python-Fallback wenn die `.pyd` fehlt:

| Klasse | GB-Modul | Fallback | Speedup | Parität |
|---|---|---|---|---|
| `AStarGrid` | `astar` | `_AStarGrid` | ~70x (grosse Karten) | bit-identische Pfade (counter-FIFO-Tie-Break repliziert) |
| `BroadPhase` | `physics` (`PHYSICS_BROAD_*`) | `_BroadPhasePy` | O(n) statt O(n²), ~300x bei 2000 Entities | identische Paare/Reihenfolge |
| `TileCollider` | `tile_collide` (`TILE_SWEEP_X/Y`) | `_sweep_axis` | ~7x pro Sweep | 0 Mismatches über 40k Fuzz-Fälle |
| `tilemap_flood_fill` | `tiled` (`TILED_FLOOD_FILL`) | `_flood_fill_py` | ~15x (160k Tiles) | identische Tiles + Count |

**Prinzip:** Validierung (Bounds, Typchecks, Fehlermeldungen) liegt immer in den
`@builtin`-Wrappern, nie im nativen Backend — daher backend-unabhängig identisches
Verhalten. Die nativen Klassen sind „dumme", schnelle Container + Kernels. Immutable
Daten (A*-Walls, Tilemap-Solid-Maske) werden einmal nach Rust gespiegelt und gecacht.

**Tests laufen lassen:**
```
.venv\Scripts\python.exe -m pytest tests/ -v
```

**Bench-Vergleich (TW vs Python-VM vs Native-VM):**
```
.venv\Scripts\python.exe gbrun.py --bench examples/<file>.gb
```
"Output: ALLE IDENTISCH" ist die Erwartung für deterministische Programme.
Programme mit `MILLIS()`, `TIME$()`, `RND()` ohne Seed sind erwartet UNTERSCHIEDLICH.

## Häufige Fallstricke

- **Pygame-Banner:** unterdrückt via `PYGAME_HIDE_SUPPORT_PROMPT=hide` in
  `graphics.py:_ensure_pygame`. Bei Tests die Pygame früh laden, ist das nötig
  für `--bench`-Equivalence.
- **`step` ist Schlüsselwort** (FOR…STEP). Variablen entsprechend benennen
  (`i`, `iter`, `tick` statt `step`).
- **`_check_int` (strikt INTEGER) vs `_check_intish`** (akzeptiert num, konvertiert
  zu int — entspricht der `"intish"`-type-Spec für Grafik-Koordinaten).
- **Cython-VM muss nach `_coerce`/`_exec_Dim`-Änderungen neu kompiliert werden** —
  sonst sieht sie weder neue Type-Spec-Erweiterungen noch externe Typen aus Modulen.
- **Variable Arity ohne types** — Decorator entpackt args zu `*args`, Funktion muss
  selbst prüfen.
- **Drei-Pfade-Äquivalenz testen:** Die `run_all`-Fixture (`tests/conftest.py`) führt
  einen Quelltext durch Tree-Walker, Python-VM UND Native-VM und prüft bit-identische
  Ausgabe. Für neue Sprach-/Operator-Features einen `run_all`-Test in
  `tests/test_three_paths.py` ergänzen — sonst bleibt die Native-VM (Produktions-Default)
  ungetestet. Single-Source-Helfer (`_container_kind`, `infer_type`) vermeiden Drift;
  neue solche Logik ebenfalls einmalig in interpreter.py halten und importieren.

## Geplant: Coroutines / YIELD

Aktuell **NICHT implementiert**. Wuerde Stack-Snapshots in den VMs und ein
Generator-Wertobjekt mit suspended state brauchen -- substantieller
Eingriff in die Drei-Pfade-Architektur.

**Skizze fuer eine spaetere Iteration:**
- AST `YieldStmt(value)`. Einer FUNCTION mit YIELD wird zur Generator-
  Function -- ihr Aufruf liefert ein `_Coroutine`-Objekt statt direktes
  Ergebnis.
- Tree-Walker: Python-Generators als Backend (yield from delegate), inkl.
  send/return-Semantik.
- Compiler/VM: braucht heap-allocated Frame-Objekte (statt Stack), damit
  beim YIELD der gesamte Locals/Stack-Snapshot ueberlebt. Op-Codes
  YIELD_VALUE und CORO_NEXT.
- Cython-Pfad: am komplexesten, weil _exec aktuell C-Stack-rekursiv ist.
  Workaround: Trampoline-Loop mit ip-Restore.
- Use-Cases: Cutscene-DSL (`WAIT_FRAMES(60)`, `WAIT_KEY(KEY_SPACE)`),
  procedurale Generation, Boss-Patterns, NPC-Dialoge.

Wer's heute schon braucht, baut state machines mit `SELECT CASE state` --
weniger ergonomisch, aber tut's.

## Input-Mapping (Modul `input`)

Statt hardcoded Keycodes ueberall (`KEYPRESSED(1073741904)`), bindet das
input-Modul Tastenkombinationen an benannte Actions:

```basic
IMPORT "input"
INPUT_BIND("move_left",  KEY_LEFT,  KEY_A)
INPUT_BIND("jump",       KEY_SPACE, KEY_W)

' --- Pro Frame ---
INPUT_UPDATE()                          ' Snapshot
IF INPUT_PRESSED("jump") THEN ...       ' Edge: gerade JETZT
IF INPUT_HELD("move_left") THEN ...     ' Held: dauerhaft
PRINT INPUT_AXIS("move_left", "move_right")   ' -1, 0, +1
```

**Edge-Detection** funktioniert ueber den `INPUT_UPDATE()`-Call am
Frame-Start: das Modul vergleicht den aktuellen Snapshot mit dem
vorigen. Ohne UPDATE bleiben PRESSED/RELEASED bei FALSE haengen.

**Multi-Key-Bindings:** Eine Action kann an N Tasten gebunden sein --
trifft sobald irgendeine davon gedrueckt ist.

**Action-Namen** sind case-insensitive (lower-case-Vergleich). Re-BIND
ueberschreibt die alte Liste.

**Beispiel:** [examples/59_input.gb](examples/59_input.gb).

## VEC2 + Operator-Overloading

Im `vec2`-Modul liefert `VEC2_NEW(x, y)` einen immutable 2D-Vektor.
Die arithmetischen Operatoren `+`, `-`, `*` (Skalar), `/`, `=` und `<>`
sind fuer VEC2 ueberladen:

```basic
IMPORT "vec2"
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(3.0, 4.0)
w = VEC2_NEW(1.0, 2.0)
PRINT v + w           ' Vec2(4.0, 6.0)
PRINT v * 2.0         ' Vec2(6.0, 8.0)
PRINT VEC2_LENGTH(v)  ' 5.0
```

**Operator-Hooks:** Vec2 registriert seine Operatoren ueber die
**Operator-Registry** (siehe Abschnitt unten) -- der Dispatch ist NICHT mehr
hardcoded. Generisches User-Operator-Overloading auf beliebigen Klassen gibt
es ebenfalls (`OPERATOR + (...) END OPERATOR`, siehe „Operator-Overloading auf
User-Klassen"). Wer einen weiteren mathematischen Wert-Typ als Modul einbaut,
ruft `register_operators(Typ, {...})` in seinem Modul -- ohne Eingriff in
interpreter.py / vm.py / vm_native.pyx.

**Werte sind immutable** -- `w = v` aliased nicht; jede Operation erzeugt
ein neues VEC2.

**Beispiel:** [examples/58_vec2.gb](examples/58_vec2.gb).

## Function References (FUNCREF)

User-Functions als first-class Werte fuer Higher-Order-Patterns:

```basic
FUNCTION square(x AS INTEGER) AS INTEGER
    RETURN x * x
END FUNCTION

DIM f AS FUNCREF
f = square            ' bare Identifier wird zur FUNCREF
PRINT f(5)            ' 25 -- Aufruf via Variable
```

Use-Cases: Sort-Comparator, Tween-Easing-Callbacks, Event-Handler.

```basic
FUNCTION twice(g AS FUNCREF, x AS INTEGER) AS INTEGER
    RETURN g(g(x))
END FUNCTION
```

**Closures werden NICHT unterstuetzt** -- der Body sieht nur Parameter und
Globals/CONST. Wer Closure-Verhalten braucht, uebergibt Werte explizit als
Parameter. Das gilt konsistent in allen drei Pfaden -- damit bleibt die
"ALLE IDENTISCH"-Garantie erhalten.

**Implementierung:**
- Type-Token `FUNCREF`. AST braucht keinen neuen Node -- bare Identifier
  in Expression-Position wird kontextabhaengig aufgeloest.
- Tree-Walker: `_eval_Identifier` liefert `_FuncRef(name)` wenn der Name
  keine Variable ist und in `self.functions` existiert. `_eval_Call`
  dispatched FuncRef-callees direkt.
- Compiler: `_global_vars`-Set wird vor Phase 5 gefuellt (alle Top-Level
  DIM/CONST/MultiDim). `_expr_Identifier` und `_expr_Call` checken Locals,
  Felder, Globals zuerst -- nur wenn keiner trifft, fallen sie auf
  Function-Lookup zurueck (Tree-Walker-konsistente Vorrang-Reihenfolge).
  Eine User-Variable mit gleichem Namen wie eine Function verschattet
  diese.
- Bytecode: `LOAD_FUNCREF` (53) und `CALL_VALUE` (54). LOAD_FUNCREF nimmt
  den Function-Namen aus dem const-Pool und pusht eine `_FuncRef`-Instanz.
  CALL_VALUE pop n args, pop callee (FuncRef), dispatch via `_exec`.
- VM-Pfade: gleicher Flow. `_FuncRef` aus `gamebasic.interpreter` importiert.

**Einschraenkung Reihenfolge:** Im VM-Pfad wird `_global_vars` static aus
allen Top-Level-Statements gefuellt. Wer eine Function `foo` erst aufruft
und DANACH eine Variable `foo` deklariert, kriegt im VM einen
"Variable nicht deklariert"-Fehler bei der Function-Verwendung -- der
Tree-Walker ist hier dynamischer. Praxisrelevant ist das selten;
empfohlen: User-Variablen oben deklarieren oder anders benennen.

**Beispiel:** [examples/57_funcref.gb](examples/57_funcref.gb).

## Static Class Members

`STATIC CONST` innerhalb einer Klasse erzeugt klassen-bezogene Konstanten,
zugreifbar via `<ClassName>.<MEMBER>` -- analog zum ENUM-Pattern.

```basic
CLASS Player
    STATIC CONST MAX_HP AS INTEGER = 100
    STATIC CONST DEFAULT_NAME AS STRING = "Hero"
    DIM hp AS INTEGER
    SUB Init()
        Self.hp = Player.MAX_HP        ' Static aus Methode
    END SUB
END CLASS

PRINT Player.MAX_HP                     ' 100
```

**Werte muessen Compile-Zeit-Literale sein:** Number, String, Bool oder
negierte Number. Keine Ausdruecke -- gleiche Strenge wie ENUM-Member,
damit alle drei Pfade konsistent sind.

**Implementierung:** Klassen-Statics werden zur Klassen-Hoisting-Phase als
`_ClassStaticNamespace` gebaut und unter dem Klassen-Namen als globale
CONST registriert. MemberAccess auf das Namespace-Objekt liefert den
Member-Wert -- LOAD_MEMBER in beiden VMs erkennt den Typ explizit.

`_infer_type` bekommt `"class_static"` als neuen Type-String. `Self.hp =
Player.MAX_HP` funktioniert, weil `Player` als CONST-Variable im globalen
Scope existiert (mit dem Namespace als Wert), nicht als Klassen-Konstruktor
-- der NEW-Pfad geht ueber `self.classes[name]`, nicht ueber Identifier-
Lookup.

**Einschraenkungen:**
- Klassen mit Statics duerfen nicht den gleichen Namen wie eine andere
  globale Variable haben. (Klassen ohne Statics sind unbeeinflusst.)
- Static-Werte sind immutable nach Compile-Zeit -- es gibt kein
  `Player.MAX_HP = 200`. Wer Mutable-Class-State will, schreibt eine globale
  Variable mit "DIM ... AS Klasse".

**Beispiel:** [examples/56_static.gb](examples/56_static.gb).

## Properties (PROPERTY GET/SET)

Klassen koennen Property-Accessors deklarieren -- Member-Read und -Write
laufen dann durch User-Code statt direkt aufs Feld:

```basic
CLASS Player
    DIM _hp AS INTEGER

    PROPERTY GET hp() AS INTEGER
        RETURN Self._hp
    END PROPERTY

    PROPERTY SET hp(value AS INTEGER)
        IF value < 0 THEN value = 0
        IF value > 100 THEN value = 100
        Self._hp = value
    END PROPERTY
END CLASS

DIM p AS Player
p = NEW Player()
p.hp = 200      ' Setter laeuft -> clamped zu 100
PRINT p.hp      ' Getter laeuft -> 100
```

**Implementation:** Properties werden intern als Methoden mit den Namen
`__get_<name>` und `__set_<name>` registriert. Die Klasse merkt sich die
Property-Namen in einem `set` (`_ClassInfo.properties` /
`VMClassInfo.properties`). Im MemberAccess- und MemberAssign-Pfad wird
zuerst gegen das Property-Set geprueft, bei Treffer dispatcht der Code
zur Internal-Methode.

**Read-only / Write-only:** Wenn nur GET deklariert ist, wirft `obj.x = v`
einen Fehler. Wenn nur SET, wirft `PRINT obj.x`.

**Inheritance:** Properties werden automatisch vererbt (gleicher MRO-
Lookup wie Methoden).

**`GET`/`SET` sind keine Keywords:** Sie sind kontext-abhaengig nach
`PROPERTY`. So bleiben User-Methoden wie `FUNCTION Get() AS T` unbeeinflusst.

**Beispiel:** [examples/63_props_comp.gb](examples/63_props_comp.gb).

## List-Comprehensions

`[expr FOR var IN container]` und `[expr FOR var IN container WHERE filter]`.
Liefert ein TUPLE der transformierten Werte:

```basic
DIM evens AS TUPLE
evens = [n FOR n IN nums WHERE n MOD 2 = 0]

' Auch mit Method-Calls und Properties
DIM names AS TUPLE
names = [it.name FOR it IN cart WHERE it.price > 5]
```

**Iterable:** STRING (chars), TUPLE, 1D-ARRAY, MAP (Keys).

**Implementation:**
- AST `ListComp(var, iterable, filter, transform)`. Im Parser an LBRACKET
  in Primary-Position.
- Tree-Walker iteriert direkt ueber Python-Iterable.
- Compiler nutzt einen Marker-Singleton (`bytecode.COMP_MARKER`) und einen
  index-basierten Loop. Vor dem Loop wird der iterable durch das Built-in
  `__COMP_ITER` in ein TUPLE umgewandelt -- so funktioniert der gleiche
  LEN+Index-Mechanismus fuer alle Container-Typen.
- Neuer Op `BUILD_TUPLE_DYN=57`: sammelt alle Werte oberhalb des Markers
  zu einem Tupel.
- Iter-Variable wird als anonymer Local-Slot reserviert -- ueberlappt
  nicht mit existierenden globalen Variablen gleichen Namens.

**Bonus:** Strings haben jetzt auch normalen Index-Access (`s[0]`) -- nicht
nur Slicing. Das war fuer die Comprehension noetig und ist eine sinnvolle
generelle Erweiterung.

**Beispiel:** [examples/63_props_comp.gb](examples/63_props_comp.gb).

## SELECT CASE mit Guards

`CASE ... WHERE expr` -- der naechste Case wird probiert, wenn entweder das
Match-Pattern nicht trifft ODER der Guard-Ausdruck falsy ist:

```basic
SELECT CASE hp
    CASE IS <= 0
        ' tot
    CASE IS <= 30 WHERE has_potion
        ' low aber Trank verfuegbar -> heilen
    CASE IS <= 30
        ' low ohne Trank -> fliehen
    CASE ELSE
        ' OK
END SELECT
```

Guard-Expressions koennen auf normale Variablen zugreifen, inkl. der
Subject-Variable. Klassische Use-Cases:
- Permission-Checks: `CASE "delete" WHERE user IN ("admin", "moderator")`
- Numerische Conditions: `CASE 1 TO 100 WHERE n MOD 2 = 0`
- State-Combinationen: `CASE "save" WHERE dirty`

**Implementierung:** Compile-Zeit-Erweiterung von `_stmt_Select`. Nach dem
Match-Erfolg wird die Guard-Expression evaluiert (Subject bleibt am Stack
fuer den Body), und ein zusaetzlicher `JUMP_IF_FALSE` springt zum
naechsten Case wenn der Guard falsch ist. Kein neuer Bytecode -- die
existierenden Ops reichen.

**Kompatibilitaet:** Existierende SELECT-Statements ohne `WHERE` laufen
unveraendert -- der Parser erzeugt `(matches, guard=None, block)`-Tupel,
sowohl 2-Tuple- als auch 3-Tuple-Cases werden in den Pfaden akzeptiert
(Backward-Compat).

## IN-Operator

`x IN container` testet Mitgliedschaft auf String, Tupel, Array oder Map:

```basic
IF "World" IN "Hello World" THEN ...      ' Substring
IF 5 IN (1, 5, 9) THEN ...                ' Tupel
IF "name" IN m THEN ...                   ' Map-Key
IF 20 IN nums THEN ...                    ' Array-Element
```

Maps haben STRING-Keys -- `5 IN map` wirft TypeMismatch. Negation klassisch
mit `NOT (x IN c)`. Praezedenz wie `=`/`<>` (Comparison-Ebene), Bytecode:
neuer Op `IN_OP=56`.

## Variadic-Functions

`...args` als letzter Parameter sammelt alle restlichen Positional-Args
in ein TUPLE:

```basic
SUB log(level AS STRING, ...rest)
    DIM msg AS STRING
    msg = "[" + level + "]"
    DIM j AS INTEGER
    FOR j = 0 TO rest.length() - 1
        msg = msg + " " + STR$(rest[j])
    NEXT
    PRINT msg
END SUB

log("WARN")                          ' rest = ()
log("INFO", "App", "started")        ' rest = ("App", "started")
```

**Einschraenkungen:**
- Variadic muss letzter Parameter sein (Parser-Fehler sonst).
- Variadic-Slot kann nicht mit Named-Arg uebergeben werden (semantisch
  unklar -- werfen).
- TUPLE als Type stuetzt `length()`, `len()` und `[i]`-Index-Access.
- Keine Default-Werte fuer Variadic (immer `()` wenn leer).

**Implementation:** AST `Param.is_variadic`, `CompiledFunction.is_variadic`.
Im Tree-Walker `_resolve_args` und in den VMs `_exec` werden ueberzaehlige
Positional-Args in ein Tupel gesammelt; Default-Resolution wird fuer
variadic-Funktionen umgangen.

**Beispiel:** [examples/62_qol_sprint.gb](examples/62_qol_sprint.gb).

## Method-Syntax auf Built-in-Containern

Strings, Arrays und Maps haben Convenience-Methoden, die zu BUILTINs
delegieren:

```basic
PRINT "hello".upper()           ' "HELLO" (= UPPER$("hello"))
PRINT "  hi  ".trim().upper()   ' Method-Chain: "HI"
PRINT a.length()                ' = LEN(a)
m.put("k", 1)                   ' = MAPPUT(m, "k", 1)
PRINT m.has("k")                ' = MAPHAS(m, "k")
```

**Dispatch-Tabelle:** `interpreter.CONTAINER_METHODS` mappt
`(target_kind, method_name)` zu BUILTIN-Namen. Tree-Walker und beide
VMs konsumieren dieselbe Tabelle (Single-Source-of-Truth).

**Verfuegbare Methoden:**
- String: `upper`, `lower`, `length`/`len`, `trim`, `left`, `right`,
  `mid`, `indexof`, `replace`, `split`, `padl`, `padr`.
- Array: `length`/`len`.
- Map: `put`, `get`, `getor`, `has`, `keys`, `size`/`length`/`len`,
  `remove`, `clear`.

Wer einen weiteren Method-Alias will, fuegt einen Eintrag in
`CONTAINER_METHODS` hinzu -- kein VM/Bytecode-Change noetig.

**User-Klassen-Methoden gewinnen:** Nur wenn der Receiver kein User-
Instanz ist (sondern String/Array/Map), wird die Container-Methoden-
Tabelle konsultiert. So kollidiert `Foo.upper()` mit User-Klasse `Foo`
nicht.

**Beispiel:** [examples/61_method_syntax.gb](examples/61_method_syntax.gb).

## Slicing

`s[a:b]`, `s[a:]`, `s[:b]`, `s[:]` -- liefert Substring (String) oder
neues 1D-Array (Array, echte Kopie). Negative Indices und Step werden
NICHT unterstuetzt -- konsistent mit der existierenden strikten Index-
Validierung. Out-of-bounds (`s[0:1000]` bei `len(s)=11`) wird auf den
gueltigen Bereich geclampt.

```basic
PRINT "Hello World"[6:11]     ' "World"
DIM b AS ARRAY OF INTEGER
b = a[1:4]                    ' echte Kopie, kein Alias
```

**Multi-Dim-Slicing** (`g[0:2, 1:3]`) wird nicht unterstuetzt --
NumPy-Semantik. Slice-Assign (`s[a:b] = ...`) wird ebenfalls bewusst
abgelehnt (Laenge-Match unklar).

**Implementierung:** AST `SliceAccess(target, lo, hi)`. Parser
disambiguiert in `_index_or_slice` anhand des Top-Level-`:`. Bytecode
`SLICE` mit Flag-Tupel `(has_lo, has_hi)` -- die VM popt entsprechend
viele Werte.

**Beispiel:** [examples/60_slicing.gb](examples/60_slicing.gb).

## ELIF / String-Multiplikation

Kleine Quality-of-Life-Erweiterungen:

- **`ELIF`** ist Alias fuer `ELSEIF` -- gleiches Token, kein AST-Change.
- **`"-" * 40`** liefert einen String aus 40 Bindestrichen. Auch
  `40 * "-"`. Negative Counts liefern leeren String. Strikt INTEGER --
  kein Float, kein Bool. Im VM-Pfad wurde gleichzeitig `OP.MUL` strenger
  gemacht (war vorher zu lax bei Bool-Operanden).

## WITH ... END WITH

Klassisches BASIC-Konstrukt fuer kompakte Member-Bursts:

```basic
WITH player
    .x = 100
    .y = 50
    .hp = 100
    .name = "Alice"
END WITH
```

**Semantik:**
- WITH-Ziel wird **einmal** evaluiert (wichtig bei Side-Effects).
- Innerhalb des Body ist `.member` Shortcut fuer `<target>.member`.
- Auch in Read-Position: `len = SQR(.x * .x + .y * .y)`.
- Compound-Assigns funktionieren: `.points += 5`.
- Verschachtelte WITHs erlaubt; innerstes gewinnt (Stack-Semantik).

**Implementierung:** Reines Compile-Zeit-Desugar ohne neuen Bytecode.
- Parser haelt einen `_with_stack: list[str]` mit Compiler-generierten
  Variablen-Namen (`__with_<n>`).
- `_with_stmt` parst, generiert frischen Namen, pusht auf Stack, parst Body,
  popt, gibt `With(var_name, target, body)` zurueck.
- Im `_primary` und `_statement_inner`: wenn aktueller Stack nicht leer und
  Token = DOT, desugar zu `MemberAccess(Identifier(top), name)`.
- Tree-Walker: `_exec_With` setzt `env.vars[var_name] = {"type":"any","value":val}`,
  fuehrt body aus, entfernt den Slot wieder.
- Compiler: `_stmt_With` allokiert anonymen Local-Slot (`_alloc_anon_slot`),
  bindet `var_name -> slot` in `local_slots` waehrend Body-Compile, entfernt
  ihn danach. So wird `Identifier(__with_<n>)` zu `LOAD_LOCAL slot`.
- "any"-Type-Coerce ist passthrough (Tree-Walker `interpreter.py:_coerce`,
  VM `vm.py:_coerce_any`, Cython `vm_native.pyx`).

**Beispiel:** [examples/55_with.gb](examples/55_with.gb).

## Tupel + Destructuring

Mehrfach-Rueckgabewerte ohne BYREF-Krampf.

```basic
FUNCTION minmax(a AS INTEGER, b AS INTEGER) AS TUPLE
    IF a < b THEN RETURN (a, b)
    RETURN (b, a)
END FUNCTION

DIM lo AS INTEGER
DIM hi AS INTEGER
(lo, hi) = minmax(7, 3)        ' Destructuring
```

**Tupel-Literal:** `(a, b, c)` -- mindestens 2 Elemente. Eine einzelne
geklammerte Expression `(expr)` bleibt Klammer-Gruppierung. `(1,)`-Single-
Tupel wie in Python wird NICHT unterstuetzt (kein Use-Case).

**Destructuring-Assignment:** `(t1, t2, ..., tn) = expr`. Die `expr` muss zur
Laufzeit ein Tupel mit exakt n Elementen ergeben -- sonst `GBRuntimeError`.
Targets duerfen Identifier, MemberAccess oder IndexAccess sein
(`(p.x, p.y) = polar_to_cart(r, a)` funktioniert).

**Type-Annotation:** `DIM t AS TUPLE` -- generisch, akzeptiert beliebige
Tupel. Keine Element-Type-Annotation an der Sprachebene; wer striktere
Garantien braucht, prueft selbst beim Destructuring.

**Implementierung:**
- AST: `TupleLit(elements)`, `TupleAssign(targets, value)`.
- Bytecode: `BUILD_TUPLE n` und `UNPACK_TUPLE n` (Ops 68, 69).
- Im Compiler-`_stmt_TupleAssign` werden Member/Index-Targets ueber einen
  anonymen Local-Slot zwischengepuffert (per `_alloc_anon_slot`), weil
  STORE_MEMBER/STORE_INDEX die Receiver-Position vor dem Wert braucht.
- Wertsemantik = Python-`tuple` (immutable). `_fmt` erzeugt `(a, b, c)`
  fuer PRINT.
- Cython-VM muss nach Aenderungen am Tupel-Pfad neu kompiliert werden.

**Praktisch:** Beispiel [examples/54_tuple.gb](examples/54_tuple.gb) zeigt
Min/Max, Vektor-Reflexion, Polar-Konvertierung, Player-State als Tupel.

## Bitwise-Operatoren

Strikt INTEGER (kein FLOAT, kein BOOL). Sechs Operatoren als Keywords:

```basic
a BAND b      ' bit-and
a BOR  b      ' bit-or
a BXOR b      ' bit-xor
a SHL  n      ' shift-left  (n >= 0)
a SHR  n      ' shift-right (n >= 0)
BNOT a        ' unaer, bitweises NICHT (= ~a in Python)
```

**Praezedenz:** Alle binaeren Bitwise auf EINER Ebene, links-assoziativ.
Position zwischen `Vergleich` und `+,-`. Heisst:
- `a BAND b = c` parst als `(a BAND b) = c`.
- `a + b BAND c` parst als `(a + b) BAND c`.
- `1 BOR 2 BAND 3` parst als `((1 BOR 2) BAND 3) = 3` — wer C-Stil-Praezedenz
  will, klammert (`1 BOR (2 BAND 3)`).

`BNOT` liegt im `_unary` neben `-` und unaerem `+` — d.h. tighter binding als
`*`/`/`. `BNOT a BAND b` ist `(BNOT a) BAND b`.

**Type-Strictness:** Bool wird abgelehnt (gleiche Linie wie `_check_num`).
Negativer Shift-Count wirft `GBRuntimeError` statt nichtssagendem Python-Fehler.

**Keine alten Built-ins mehr:** Frueher gab's `BITAND/BITOR/BITXOR/BITNOT/SHL/SHR`
als Funktions-Built-ins. Mit den Operatoren ueberfluessig — entfernt.
`BITAND(a, b)` -> `a BAND b`. Im Tree-Walker (interpreter.py:1009-1027) und
in beiden VMs implementiert (Ops 62-67 in bytecode.py).

## SELECT CASE

Mehrweg-Verzweigung statt verschachtelter `IF/ELSEIF`-Ketten. Drei Match-Formen
pro CASE, beliebig kombinierbar:

```basic
SELECT CASE x
    CASE 1                       ' exakter Wert
        ...
    CASE 2, 3, 4                 ' Liste von Werten
        ...
    CASE 10 TO 20                ' Bereich (inklusiv)
        ...
    CASE IS > 100                ' Vergleich (=, <>, <, >, <=, >=)
        ...
    CASE 1, 5 TO 8, IS = 13      ' alle Formen mischbar
        ...
    CASE ELSE                    ' Fallback (optional, max. einmal, muss letzter sein)
        ...
END SELECT
```

**Garantie:** Subject-Ausdruck wird **einmal** evaluiert (auch bei
Side-Effects in Function-Calls). Der erste passende Case gewinnt.

**Implementierung (lehrreich):** Im Parser zu `Select(subject, cases, else_block)`,
Cases sind `(list[CaseMatch], list[Stmt])`-Tupel mit `kind ∈ {"value", "range"}`.
Im Compiler **kein neuer Bytecode** — der Subject bleibt während aller Match-Tests
auf dem Stack (per `DUP` geklont), Range-Tests werden zu `subj >= lo` (mit
`JUMP_IF_FALSE`) gefolgt von `subj <= hi` (mit `JUMP_IF_TRUE` zum Block) verkettet.
Cython-VM hat es ohne Neukompilation übernommen.

## ENUM

Typsichere Konstanten mit Namespace-Zugriff (`State.PLAYING`):

```basic
ENUM State = MENU, PLAYING, PAUSED       ' compact
ENUM Permission                          ' block
    NONE = 0
    READ = 1
    WRITE = 2
END ENUM
```

Auto-Nummerierung (0, 1, 2, …) oder explizit. Mixed: nach explicit zählt's
weiter (`A, B = 5, C` → A=0, B=5, C=6). Member-Namen dürfen Keywords sein
(`READ`, `FILE`, `DATA`, `NONE`) — der qualifizierte Zugriff ist eindeutig.

**Implementierung:** `EnumDecl(name, members)` AST-Node. Im Tree-Walker
und Compiler zur Compile-Zeit zu einem `_EnumNamespace`-Objekt aufgelöst,
als globale CONST abgelegt. `MemberAccess` erkennt `_EnumNamespace` (in
`interpreter.py`, `vm.py`, `vm_native.pyx`) und liefert den Member-Wert.
Member-Werte müssen Compile-Time-Integer-Literale sein (auch im
Tree-Walker — Konsistenz). `DIM x AS State` löst der Parser zu `INTEGER`
auf, indem er bekannte Enum-Namen in `self._enum_names` trackt.

Keywords als Member-Namen: Parser-Helfer `_consume_member_name` und
DOT-Zugriff in `_postfix` akzeptieren jedes Token mit string-`value`,
nicht nur `IDENT`.

## Named Arguments

`func(name: "Anna", age: 30)` mit Defaults. Lexer-Token `COLON`,
AST-Node `NamedArg(name, value)` als Element von `Call.args`.

**Tree-Walker** (`interpreter.py`): `_resolve_args(decl, raw_args, fn_name)`
mappt positional + named auf Param-Reihenfolge, liefert eine voll-lange
Liste mit `_DEFAULT_SENTINEL` für Slots, die der User nicht belegt hat.
`_invoke` evaluiert Sentinels via Default-Ausdruck im local_env. Funktioniert
mit BYREF (Sentinel-Slots können kein BYREF sein) und mit Param-
referenzierenden Defaults.

**Compiler** (`compiler.py`): `_resolve_named_args(fn, raw_args, fn_name)`
löst zur Compile-Zeit auf — Slots ohne Wert kriegen den evaluierten
Default-Literalwert direkt als `LOAD_CONST` emittiert. `param_names` ist
ein neues Feld auf `CompiledFunction` (Compile-Zeit-Info, VM nutzt es nicht).
Auch `NEW Klasse(...)` wird so resolved (Init-Methode lookup zur Compile-Zeit).

**Einschränkungen im VM-Pfad:**
- Built-ins haben keine deklarierten Param-Namen → werfen.
- Method-Calls (`obj.method(name: ...)`): Klasse erst zur Laufzeit
  bekannt → Compiler wirft. Tree-Walker kann's.

## Self + implizite Methoden-Aufrufe

Innerhalb einer Klassen-Methode:

```basic
CLASS Wave
    SUB Init()
        StartCurrent()         ' impliziter Methoden-Aufruf
    END SUB
    SUB StartCurrent()
        ...
    END SUB
END CLASS
```

`Self` als Identifier liefert die aktuelle Instanz; bare `MethodName(...)`
ohne `Self.`-Präfix dispatcht zuerst gegen die Methoden der eigenen Klasse
(und Superklassen), erst dann gegen globale Funktionen.

**Tree-Walker:** `Interpreter._method_stack: list[(_Instance, _ClassInfo)]`
wird in `_invoke` gepusht/gepoppt. `_eval_Identifier` erkennt `"self"` und
liefert die aktuelle Instanz. `_eval_Call` mit Identifier-callee prüft
zuerst `_resolve_method(current_cls, name)`.

**Compiler:** `_load_var` emittiert für `name == "self"` (innerhalb
`current_class != None`) den neuen Op `LOAD_SELF`. `_expr_Call` bei
Identifier-callee resolved Methoden via `_resolve_method_compile` und
emittiert `LOAD_SELF` + Args + `CALL_METHOD`. Damit Methode A in derselben
Klasse die Methode B sehen kann, registriert Phase 4a vor dem Body-
Kompilieren leere Stub-`CompiledFunction`s in `ci.methods`.

**Bytecode-Op:** `LOAD_SELF = 88` — push `self_obj` (im VM-`_exec` als
Parameter). Implementiert in `vm.py` und `vm_native.pyx`. Wer Self-Code
schreibt, muss daher `vm_native.pyx` neu kompilieren.

## Statement-Trenner Doppelpunkt

`x = 1 : y = 2` — Doppelpunkt trennt Statements wie Newline.
`Parser._consume_terminator` und `_skip_newlines` akzeptieren beide Token.
Funktioniert mit Named-Args nicht in Konflikt, weil dort der `IDENT COLON`-
Lookahead in `_call_arg` läuft (innerhalb von `(` ... `)`), wo der
Terminator gar nicht erst geprüft wird.

## f-Strings (String-Interpolation)

`f"text {expr} text..."` -- der Lexer expandiert das zur Token-Sequenz
`("text" + STR$(expr) + "text" + ...)`. Damit funktionieren f-Strings ohne
einen einzigen Eingriff in Parser, Interpreter, Compiler oder VMs:

```basic
DIM name AS STRING
DIM hp AS INTEGER
name = "Anna"
hp = 75
PRINT f"{name} hat {hp} HP"          ' "Anna hat 75 HP"
PRINT f"max: {MAX(a, b)}"            ' Methodenaufrufe in {} sind ok
PRINT f"literal {{nicht interpoliert}}, aber {hp}"
```

**Eigenschaften:**
- `{{` und `}}` sind Escapes fuer literale geschweifte Klammern.
- Verschachtelte f-Strings sind nicht erlaubt (`f"{f"..."}"`).
- Ausdruecke duerfen `(`, `)`, Methoden-Aufrufe, MemberAccess etc.
  enthalten -- der Tokenizer matched balanced braces.
- Ohne `f`-Prefix bleibt `"hi {name}"` ein wortlich enthaltener String mit
  geschweiften Klammern -- Opt-in.
- Editor-Highlighter erkennt f-Strings als Block und faerbt den ganzen
  Range einheitlich als String (siehe `editor_qt/highlighter.py`).

**Format-Specs** (`{expr:spec}`): ein Top-Level-`:` im Platzhalter trennt
einen printf-Spec ab -- der Lexer emittiert dann `FORMAT$(expr, "%spec")`
statt `STR$(expr)`:
```basic
PRINT f"FPS {fps:.1f}  Score {score:05d}"   ' "FPS 59.7  Score 00042"
```
Ein `:` innerhalb von `()`/`[]`/`{}` (z.B. Slice `s[0:3]`) oder String-Literalen
zaehlt NICHT als Spec-Trenner -- `_split_fstring_spec` in `lexer.py` trackt
Klammer-/String-Tiefe. Rein Lexer-basiert, daher in allen drei Pfaden gleich.

**Implementierung:** `lexer._scan_fstring` wird beim ersten `f"`-Lookahead
aufgerufen ([lexer.py:114-115](gamebasic/lexer.py:114)) und emittiert die
expandierte Token-Sequenz selbst -- mit Sub-Lexer fuer den Ausdrucks-Teil.

**Beispiel:** [examples/69_fstring.gb](examples/69_fstring.gb).

## Kontrollfluss: BREAK / CONTINUE / REPEAT / TRY

Diese sind implementiert (waren in aelteren Doku-Staenden nicht aufgefuehrt):

- **`BREAK`** / **`CONTINUE`** in `FOR`, `FOR EACH`, `WHILE`, `REPEAT`. Auch in
  Single-Line-IF (`IF v = 40 THEN BREAK`). Tree-Walker via `_BreakSignal`/
  `_ContinueSignal`-Exceptions; Compiler via `break_patches`/`continue_patches`-
  Stack (mit `try_depth` fuer korrektes `TRY_END`-Unwinding).
- **`REPEAT ... UNTIL cond`** -- Post-Test-Loop (laeuft mindestens einmal).
- **`WHILE cond ... WEND`** -- Pre-Test-Loop.
- **`TRY ... CATCH [e] ... END TRY`** + **`THROW value`**. Die Catch-Variable
  ist optional und faengt den (String-)Wert. Kein typed Catch -- `THROW` wirft
  beliebige Werte; Module-/Runtime-Fehler kommen als `GBRuntimeError`-Message.

```basic
FOR EACH e IN enemies
    IF e.dead THEN CONTINUE
    IF boss_killed THEN BREAK
    e.update()
NEXT

TRY
    riskante_op()
CATCH msg
    PRINT "Fehler: " + msg
END TRY
```

## FOR EACH

`FOR EACH var IN container ... NEXT` -- iteriert ueber STRING (Zeichen),
TUPLE, 1D-ARRAY oder MAP (Keys):

```basic
FOR EACH b IN bullets
    b.update()
NEXT
FOR EACH k IN scores        ' Map -> Keys
    PRINT k, MAPGET(scores, k)
NEXT
```

`each` ist **kontextuell**, kein Keyword: `FOR each = 1 TO 3` mit einer
Variable namens „each" bleibt ein regulaerer FOR (Disambiguierung im Parser:
FOR EACH nur wenn nach `each` ein IDENT statt `=` folgt).

**Implementierung:** AST-Node `ForEach(var, iterable, body)`. Tree-Walker
iteriert direkt (`_iter_for_comp`, wie Comprehensions). Compiler desugart zu
einem Vorwaerts-Index-Loop ueber `__comp_iter(iterable)` (-> TUPLE) +
`LOAD_INDEX` und nutzt den vorhandenen break/continue-Patch-Stack -- **kein
neuer Bytecode**, beide VMs unveraendert. Loop-Var wird als `"any"` deklariert.

## IIF (Inline-Ternary)

`IIF(cond, then, else)` -- echter **lazy** Ternary, nur EIN Zweig wird
ausgewertet (Short-Circuit):

```basic
dx = IIF(moving_left, -speed, speed)
PRINT IIF(x <> 0, 100 \ x, -1)    ' bei x=0: kein Division-Crash
```

`iif` ohne `(` bleibt ein normaler Bezeichner (kontextuell im Parser).
**Implementierung:** AST-Node `TernaryExpr(cond, then, else)`. Compiler
desugart zu `JUMP_IF_FALSE` (poppt die Bedingung) -- **kein neuer Bytecode**.
Tree-Walker: `_eval_TernaryExpr` evaluiert nur den gewaehlten Zweig.

## Array- & Map-Helfer

Reine Builtins (alle drei Pfade automatisch), auch als Container-Methoden
(`CONTAINER_METHODS`-Tabelle):

- `SORT(arr)` / `arr.sort()` -- 1D-Array IN PLACE aufsteigend (INTEGER/FLOAT/STRING).
- `REVERSE(arr)` / `arr.reverse()` -- 1D-Array IN PLACE umkehren.
- `ARRAY_INDEXOF(arr, v)` / `arr.indexof(v)` -- erster Index oder -1.
- `MAPVALUES(m)` / `m.values()` -- ARRAY aller Werte (Einfuege-Reihenfolge).
- `MAPITEMS(m)` / `m.items()` -- ARRAY von `(key, value)`-TUPELn (gut mit
  `FOR EACH` + Destructuring).

## Module-Imports mit Alias

`IMPORT "modul" AS alias` -- aliased die Built-ins / externen Typen unter
einem ersetzten Praefix:

```basic
IMPORT "json" AS j
DIM h AS J_HANDLE
h = J_PARSE("[1, 2, 3]")
PRINT J_GET_INT(h, "0")     ' 1
```

**Aliasing-Strategie:** GameBasic-Module teilen einen flachen Built-in-
Namespace -- es gibt kein echtes Namespacing. Der Alias dupliziert alle
Built-ins / Typen, deren Name mit `<modul>_` anfaengt, unter `<alias>_`.
Single-word-Namen (z.B. der externe Typ `vec2`) werden komplett ersetzt
(`vec2` -> `v` bei `IMPORT "vec2" AS v`).

**Konvention-basiert:** funktioniert fuer Module, deren Built-in-Praefix
dem Modul-Namen entspricht (json, db, tween, vec2, sprite, ecs, ...).
Module mit abweichendem Praefix (z.B. `imgfx` registriert `IMAGE_*`,
nicht `IMGFX_*`) sind nicht via Alias adressierbar -- der Praefix-Match
liefert dann leer.

**Idempotent + sticky:** zweimal mit unterschiedlichen Aliasen ist OK
(beide werden zusaetzlich registriert), aber doppelt mit demselben Alias
ist no-op.

## Dict/Set-Comprehensions

`{key: val FOR var IN iterable [WHERE filter]}` -- Dict-Comprehension,
liefert eine MAP. `{expr FOR var IN iterable [WHERE filter]}` -- Set-
Comprehension, liefert ein TUPLE mit deduplizierten Werten in der
Reihenfolge des ersten Auftretens.

```basic
' Dict-Comp: Quadrate als Map
DIM squares AS MAP OF INTEGER
squares = {STR$(x) + "sq": x * x FOR x IN (1, 2, 3, 4)}
PRINT MAPGET(squares, "3sq")     ' 9

' Set-Comp: eindeutige Mod-Werte
DIM distinct AS TUPLE
distinct = {x MOD 3 FOR x IN (0, 1, 2, 3, 4, 5, 6, 7, 8)}
PRINT distinct                    ' (0, 1, 2)
```

**Dict-Keys MUESSEN STRING sein** (GameBasic-MAP-Konvention). Der MAP-
Wert-Typ wird beim ersten Eintrag inferiert. Set-Comp ist eine pragmatische
Naeherung -- GameBasic hat keinen echten SET-Typ; das deduplizierte TUPLE
ist die nahe liegende Alternative.

**Implementierung:** Lexer kennt jetzt `LBRACE`/`RBRACE` (nur fuer Comp-
Position). Parser disambiguiert per `:`-Lookahead (Dict) oder direkt
`FOR` (Set). Compiler nutzt das existierende `BUILD_TUPLE_DYN`-Pattern
plus zwei interne Built-ins (`__SET_DEDUP`, `__DICT_FROM_PAIRS`) als
Final-Schritt -- keine neuen Bytecode-Ops noetig, Cython-VM ohne
Aenderung kompatibel.

**Beispiel:** [examples/71_dictcomp.gb](examples/71_dictcomp.gb).

## Operator-Overloading auf User-Klassen

Klassen koennen Operatoren ueberladen, indem sie `OPERATOR <op>`-Methoden
definieren -- analog zu `SUB`/`FUNCTION` im Class-Body:

```basic
CLASS Money
    DIM cents AS INTEGER

    OPERATOR + (other AS Money) AS Money
        DIM r AS Money
        r = NEW Money()
        r.cents = Self.cents + other.cents
        RETURN r
    END OPERATOR

    OPERATOR = (other AS Money) AS BOOLEAN
        RETURN Self.cents = other.cents
    END OPERATOR
END CLASS

DIM a AS Money
a = NEW Money()
a.cents = 100
DIM b AS Money
b = NEW Money()
b.cents = 200
PRINT (a + b).cents     ' 300
PRINT a = b             ' FALSE
```

**Erlaubte Operatoren:** `+`, `-`, `*`, `/`, `MOD`, `=`, `<>`, `<`, `>`,
`<=`, `>=`. Genau ein Parameter (`other`), Rueckgabetyp ist Pflicht.
BYREF und variadic sind nicht erlaubt.

**Implementierung:** Parser konvertiert `OPERATOR + (...)` zu einer Methode
mit reserviertem Namen `__op_add__` (siehe `parser._OPERATOR_NAMES`).
Tree-Walker (`_eval_BinaryOp`) und beide VMs (`OP.ADD/SUB/MUL/DIV/MOD/EQ/...`)
konsultieren via `_user_op(...)` die Methode auf LHS, dann auf RHS
(Reverse-Dispatch). Fallback ist der Standard-Pfad.

**Vererbung:** Operator-Methoden werden ueber die normale MRO gesucht --
Child-Klassen erben sie automatisch.

**Einschraenkungen:**
- Keine reflektierten Operatoren a la Python (`__radd__`). Wer `5 + money`
  unterstuetzen will, definiert `OPERATOR + (other AS INTEGER) AS Money`
  auf `Money` -- der Reverse-Dispatch greift dann.
- Kein Method-Overloading: pro Operator gibt's genau eine Methode.
  `Money + Money` und `Money + INTEGER` koennen nicht gleichzeitig
  definiert werden (man muesste type-switchen im Body).
- Operatoren auf Modul-Typen (Vec2 etc.) gewinnen vor User-Klassen --
  die Modul-Registry wird zuerst konsultiert.

**Beispiel:** [examples/70_operator.gb](examples/70_operator.gb).

## Operator-Registry

Modul-eigene Typen koennen arithmetische Operatoren (`+`, `-`, `*`, `/`)
ueberladen, ohne dass interpreter.py / vm.py / vm_native.pyx angefasst werden:

```python
# In gamebasic/modules/<name>.py
from . import register_operators

def _op_add(a, b):
    if isinstance(a, _MyType) and isinstance(b, _MyType):
        return _MyType(...)
    raise TypeMismatchError("...")

register_operators(_MyType, {"+": _op_add, "-": _op_sub, "*": _op_mul, "/": _op_div})
```

**Dispatch:** Vor dem Standard-Pfad ruft Tree-Walker (`_eval_BinaryOp`) und
beide VMs (`OP.ADD/SUB/MUL/DIV`) `modules.dispatch_binary_op(op, a, b)`.
Wenn `type(a)` oder `type(b)` registriert ist, dispatcht zur Handler-Tabelle;
sonst liefert die Registry `NO_OP_MATCH` und der Standard-Pfad uebernimmt.

**Konvention:** Bei asymmetrischen Operatoren (z.B. `Skalar * Vec2`) muss
der Handler beide Reihenfolgen selbst akzeptieren, weil die Registry nur
einen Handler-Eintrag pro Typ kennt -- siehe `vec2._op_mul` als Pattern.

**Equality:** `=` und `<>` sind nicht in der Registry -- die Standard-
Python-Equality (`__eq__`/`__ne__` auf der Klasse) reicht. Die VMs nutzen
`a == b` direkt.

Wer einen neuen Math-Typ wie `_Mat3x3`, `_Complex` oder `_Quat` einbauen
will, schreibt nur sein Modul -- keine Aenderung an Interpreter oder VMs.

## Asset-Cache + `LOAD_ASSETS`

`Graphics` haelt zwei Caches: `_image_cache` und `_sound_cache`. Sowohl
`LOADIMAGE` als auch `LOADSOUND` pruefen zuerst den Cache und cachen
das Ergebnis unter zwei Schluesseln: dem rohen Pfad UND dem
normalisierten Absolut-Pfad. So treffen verschiedene Pfad-Schreibweisen
(`"sprites/x.png"` vs. `"./sprites/x.png"` vs. absolut) denselben Eintrag.

`LOAD_ASSETS(manifest.json)` praefuellt den Cache aus einem JSON-Manifest:

```json
{
  "images": { "player": "sprites/player.png", "enemy": "sprites/enemy.png" },
  "sounds": [ "sfx/jump.wav", "music/level1.ogg" ]
}
```

Beide Sektionen sind optional, jede kann **Dict** (Alias → Pfad) oder
**Liste** (nur Pfade) sein. Bei Dict-Form ist `LOADIMAGE("player")`
ein Cache-Hit (Alias) **und** `LOADIMAGE("sprites/player.png")` auch
(Pfad-Hit unter Absolut-Pfad). Pfade im Manifest sind relativ zum
Manifest-Verzeichnis.

`LOAD_ASSETS` liefert die Anzahl geladener Assets. Idiomatisch in der
Init-Phase nach `SCREEN(...)` aufrufen, damit Bilder direkt
`convert_alpha`-optimiert werden. Beispiel: [examples/75_preloader.gb](examples/75_preloader.gb).

## Z-Layer-Rendering

Layer sind named Compose-Surfaces mit explizitem z-Wert. Alle
draw-Methoden zeichnen auf `Graphics._buffer`; `LAYER("name")` lenkt
`_buffer` auf die Layer-Surface um. `FLIP` composiert alle Layer in
z-Order auf den `_main_buffer`, blittet zum Screen, und cleart die
Layer (transparent) fuer den naechsten Frame.

```basic
LAYER_DEFINE("bg", 0)
LAYER_DEFINE("sprites", 10)
LAYER_DEFINE("ui", 100)

LAYER("bg");      CLS(SKY); DRAWIMAGE(parallax, 0, 0)
LAYER("sprites"); DRAWIMAGE(player, x, y)
LAYER("ui");      TEXT(10, 10, "Score: 100")
FLIP()   ' composiert in z-Order, cleart fuer naechsten Frame
```

**Builtins:**
- `LAYER_DEFINE(name, z)` — registrieren mit z; redefine aktualisiert z
- `LAYER(name)` — switchen (auto-Define mit auto-z wenn neu)
- `LAYER_END()` — zurueck zum Main-Buffer (optional, FLIP macht's auch)
- `LAYER_CLEAR(name)` — manuell leeren (selten gebraucht)

**Implementation** ([graphics.py](gamebasic/graphics.py)): `_Layer`-Class
mit `name`, `z`, `surface` (SRCALPHA, lazy-allokiert beim ersten USE
nach SCREEN). `_main_buffer` ist das Compose-Target; `_buffer` ist der
aktive Draw-Target (kann auf `_main_buffer` oder eine Layer-Surface
zeigen). FLIP composiert sortiert nach z aufsteigend (niedrigstes z =
hinten, hoechstes z = vorne).

**Backwards-Compat:** Code ohne `LAYER_*`-Calls hat `_layer_order = []`,
der Compose-Pfad in FLIP ist ein No-Op und `_buffer` zeigt direkt auf
`_main_buffer`. Existierende Programme laufen unveraendert.

**Beispiel:** [examples/76_layers_atlas.gb](examples/76_layers_atlas.gb).

## Sprite-Atlas + Batch-Draw

Sprite-Atlas: EIN grosses Image + Dict von `name -> (x, y, w, h)`-Rects.
Mehrere Sub-Sprites teilen sich eine `pygame.Surface` -- ideal fuer
`pygame.Surface.blits()`, das viele Sprites in einem C-Call rendert
(Game-Engine-Pattern fuer Tilemaps, Bullet-Hell, Tile-Drawing).

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/tiles_atlas.json")

' Einzel-Draw (Camera-aware):
ATLAS_DRAW(atlas, "tile_grass", 0, 0)

' Batch-Pattern (schneller bei vielen Sprites):
FOR i = 0 TO 99
    BATCH_DRAW(atlas, "tile_grass", i * 16, 0)
NEXT
BATCH_FLUSH()   ' EIN pygame.Surface.blits()-Call fuer 100 Sprites
```

**Manifest-Format:**
```json
{
  "image": "tiles.png",
  "sprites": {
    "tile_grass": [0,  0, 16, 16],
    "player":     [16, 0, 24, 32]
  }
}
```
Rects sind `[x, y, w, h]`. Bild-Pfad relativ zum Manifest.

**Builtins:**
- `ATLAS_LOAD(json)` -> `SPRITE_ATLAS`
- `ATLAS_DRAW(atlas, name, x, y)` — einzeln, Camera-aware
- `BATCH_DRAW(atlas, name, x, y)` — an Batch-Queue anhaengen
- `BATCH_FLUSH()` — Queue jetzt rendern (pygame.Surface.blits)

**Auto-Flush** an den richtigen Punkten:
- vor FLIP (sonst geht die Queue verloren)
- vor LAYER-Switch (damit Batch zum richtigen Target geht)
- vor ATLAS_DRAW (Direct-Call) -- bewahrt Reihenfolge

**Zoom-Caveat:** Bei `CAMERA_SET`-Zoom ≠ 1 faellt jeder `BATCH_DRAW`
auf `draw_image_part` zurueck (kein Batch-Vorteil, weil pygame nicht
batch-skalieren kann). Translation ist OK.

**Externer Typ:** `SPRITE_ATLAS` ist in TYPE_DEFAULTS / _coerce in
allen drei Pfaden (Tree-Walker, vm.py, vm_native.pyx) registriert.
`DIM x AS SPRITE_ATLAS` funktioniert direkt ohne IMPORT.

## ECS Bulk-System-Ops

Klassische ECS-Performance-Falle: pro-Entity-Loop in BASIC mit 6
Builtin-Calls/Entity. Beispiel-Bench: 500 Entities × 100 Frames mit
`ECS_GET_FLOAT`/`ECS_ADD_FLOAT` → 215 ms auf der Native-VM. Mit
`ECS_INTEGRATE_FLOAT(world, "px", "vx")` → **5 ms (43× schneller)**.

Die Bulk-Ops verarbeiten eine ganze Component-Schicht in einer cdef-
Loop, ohne Python-Dispatch-Overhead pro Entity:

| Builtin | Wirkung |
|---|---|
| `ECS_INTEGRATE_FLOAT(w, target, delta)` | `target += delta` fuer alle Entities mit beiden Components |
| `ECS_INTEGRATE_INT(w, target, delta)` | INT-Variante |
| `ECS_SCALE_FLOAT(w, target, factor)` | `target *= factor` (z.B. Friction) |
| `ECS_FILL_FLOAT(w, target, value)` | alle Werte = value (Reset) |
| `ECS_FILL_INT(w, target, value)` | INT-Variante |
| `ECS_CLAMP_FLOAT(w, target, lo, hi)` | Bounds-Clamp |
| `ECS_REMOVE_DEAD(w, name, threshold)` | Entities mit `value <= threshold` zerstoeren |
| `ECS_COUNT_WITH(w, name)` | O(1) Halter-Zaehlung |

**Implementation:** [gamebasic/modules/ecs_native.pyx](gamebasic/modules/ecs_native.pyx).
`_World` und `_Component` sind cdef-Klassen. Sparse-Set-Ops als cpdef
Methoden. Fast-Path-Methoden auf `_World` (`get_float`, `add_float`,
...) wickeln die ehemals separaten `_check_*` + `_get_value` + `_b_*`-
Funktionen in einem cpdef-Call ab.

**Beispiele:** [examples/bench_ecs_movement_v2.gb](examples/bench_ecs_movement_v2.gb)
(Integrate-only), [examples/bench_ecs_systems.gb](examples/bench_ecs_systems.gb)
(volles Bullet-Hell-Pattern mit 8 Systemen pro Frame).

**Game-Pattern-Lesson:** Wer ein Spiel-Hot-Path-System hat, das ueber
viele Entities laeuft, sollte es als Bulk-Op-Builtin schreiben statt
als pro-Entity-BASIC-Loop. Boilerplate fuer einen neuen Bulk-Builtin:
cpdef-Method auf `_World` in `ecs_native.pyx` + Python-Fallback in
`ecs.py` + `@builtin`-Wrapper.

## Native cdef-Klassen (Performance-Layer)

Drei Cython-Module mit cdef-Klassen, die Hot-Path-Code aus Python in
C verschieben:

| Datei | cdef-Class | Wirkung |
|---|---|---|
| `gamebasic/vm_native.pyx` | `VM` | Die Native-VM. `_exec`-Loop mit C-Slots fuer Stack/Constants/Locals. |
| `gamebasic/array_native.pyx` | `_GBArray` | Typed-Memoryview (`long long[::1]` fuer INT, `double[::1]` fuer FLOAT) ueber `array.array`-Backing. `cdef inline _flat_c` fuer Bounds-Check + Stride-Arithmetik. `get_at`/`set_at` als VM-Fast-Path. |
| `gamebasic/modules/ecs_native.pyx` | `_World`, `_Component` | Sparse-Set in cdef. Fast-Path-Methoden auf `_World` (siehe ECS-Bulk-Ops). |

**Pure-Python-Fallbacks:** Wenn ein `.pyd` fehlt (z.B. nach `git clone`
vor `setup.py build_ext`), greift in `interpreter.py` und `modules/ecs.py`
ein Pure-Python-Fallback. Tree-Walker und Python-VM bleiben benutzbar,
nur ohne den Speed-Bonus.

**Build:**
```
.venv\Scripts\python.exe setup.py build_ext --inplace
```
Setup.py kompiliert alle drei `.pyx`-Files. Nach Aenderungen an
`vm_native.pyx`, `array_native.pyx`, oder `ecs_native.pyx` neu bauen.

## Performance-Optimierungen im Compiler/VM

Mehrere Stufen, die zusammen die Native-VM auf 3–14× ggue Tree-Walker
bringen. Vollstaendige Bench-Tabelle in [docs/PERFORMANCE.md](docs/PERFORMANCE.md).

### Spezialisierte Numeric-Opcodes

11 Opcodes (`ADD_NN`, `SUB_NN`, `MUL_NN`, `DIV_NN`, `LT_NN`, `GT_NN`,
`LEQ_NN`, `GEQ_NN`, `EQ_NN`, `NEQ_NN`, `NEG_N`). Der Compiler emittiert
sie ueber `_expr_type(e)`-Inference (best-effort, konservativ), wenn
beide Operanden statisch als numerisch bekannt sind. Liest aus:
- Locals via `local_types[slot]`
- Globals via `_global_types[name]`
- Function-Return-Types
- BinaryOp/UnaryOp rekursiv
- **IndexAccess auf typisierte Arrays** (`buf[i]` bei `ARRAY OF INTEGER/FLOAT`
  -> Element-Typ) und **`Self.feld`** mit statisch bekanntem Skalartyp.
  Sicher, weil typisierte Arrays/Felder homogen sind. Damit greift der
  `_NN`-Pfad auch bei `buf[i] + i + j` und `Self.total + n`.

Bei `bench_loop` (tighter Numeric-Loop): **1.46×** auf Native-VM.

FOR-Loop-Bookkeeping (Increment + Bound-Check) wird ebenfalls ueber
spec-ops emittiert.

### 1D-Array-Index-Fast-Path

`LOAD_INDEX`/`STORE_INDEX` in beiden VMs haben einen Fast-Path fuer den
haeufigsten Fall: 1D-`_GBArray`, ein int-Index in Bounds. Ueberspringt die
`isinstance`-Cascade (str/tuple/array) + Index-Validierungs-Loop. Alle
Edge-Cases (String/Tupel/Multidim/OOB/bool) fallen unveraendert in den
generischen Pfad -> identische Fehler, kein Bit-identisch-Risiko. Kein neuer
Opcode, kein Compiler-Eingriff.

### Tree-Walker-Dispatch-Cache

`Interpreter._eval`/`_exec` memoisieren die Handler-Methode pro Node-Typ
(`_eval_cache`/`_exec_cache`: `type(node) -> gebundene Methode`) statt pro
Node `getattr(self, f"_eval_{name}")` zu bauen. Wirkt auf ALLEM Tree-Walker-
Code (~13 % bei expression-dichten Frames) -- relevant, weil der Editor-Run
und die Bench-Equivalenz den Tree-Walker nutzen.

### Inline-Cache fuer OOP-Dispatch

`CompiledFunction.caches`-Liste parallel zu `code`. Monomorphic IC fuer
`CALL_METHOD`, `LOAD_MEMBER`, `STORE_MEMBER` auf `_Instance`-Receivern.
Hit-Check: `obj.cls is cache[0]`. Spart `_resolve_method`-Call und
Dict-Lookup. Bei `bench_method_dispatch`: **1.34×**.

Cache wird auch im stub-Pfad (Phase 4a) konsistent kopiert
([compiler.py: stub.caches = compiled.caches](gamebasic/compiler.py)).

### Globals-as-Slots

Compile-Zeit-Aufloesung von Top-Level-DIM/CONST/Enum/For-var/Class-
Static-Namen zu Slot-Indizes (`_global_slots: dict[str, int]`).
Neue Opcodes:
- `LOAD_GLOBAL_SLOT idx`
- `STORE_GLOBAL_SLOT idx`
- `DECLARE_GLOBAL_SLOT (idx, name_idx, type, default)`
- `DECLARE_GLOBAL_CONST_SLOT (idx, name_idx, type)`

Die VM-Pfade fuehren `global_slots: list[_Slot]` parallel zum
`globals_`-Dict. `DECLARE_*_SLOT` schreibt den `_Slot` in BEIDE
Strukturen (gleiches Object), so bleiben name-basierte Ops
(`INPUT_NAME`, `LOAD_NAME`) konsistent.

Pre-registrierte Globals (`KEY_*`, `BLACK`, `WHITE`, `PI`, ...) leben
weiter nur im Dict, weil der Compiler sie nicht statisch erkennt --
sie gehen ueber den Fallback `LOAD_NAME`.

`Struct`/`Array`/`Map`-DIMs werden **nicht** slot-allokiert (ihre Init-
Pfade `DECLARE_STRUCT_NAME` / `DECLARE_ARRAY_NAME` haben spezielle
Allokation -- der Performance-Vorteil waere klein, der Kosten gross).

Bei `bench_loop`: **1.30×** zusaetzlich.

### Constant Folding

`_try_fold(e)` im Compiler — BinaryOp/UnaryOp mit konstanten Operanden
wird zu einem einzelnen `LOAD_CONST`. Konservativ:
- kein Folding bei Bool-in-Arithmetik
- kein Folding bei Division durch 0 (Runtime-Error besser)
- kein Folding bei extrem grossen POW-Werten (Safety-Cap)
- `and`/`or` werden NICHT gefoldet (Short-Circuit-Semantik bewahren)

Hilft Patterns wie `FOR i = 0 TO 100 - 1`, `width / 2`, `2 * 3.14`.

### Typed Array Backing + cdef `_GBArray`

`ARRAY OF INTEGER` nutzt `array.array('q')` (8-Byte signed int) statt
Python-Liste. `ARRAY OF FLOAT` analog `array.array('d')`. Spart
Box/Unbox bei jedem Zugriff. **64-bit-Limit fuer INTEGER-Arrays**
(-9.2e18..9.2e18) — Skalar-`DIM x AS INTEGER` bleibt arbitrary-
precision.

`_GBArray` ist cdef-Class in `array_native.pyx` mit typed memoryviews.
`get_at(indices)` / `set_at(indices, value)` sind die Fast-Path-API,
die die VMs statt `arr.values[arr.flat_index(...)]` rufen.

### Convert/Coerce-Fast-Path in der Python-VM

`vm.py` hat ein `_FAST_COERCE`-Dict mit pro-Typ-Funktionen statt einer
generischen `if`/`elif`-Cascade. Trifft den heissesten Pfad jeder
STORE-Op (Local/Global/Field/Index/Parameter-Binding).

## Sprite-Editor (`gbsprites`)

PySide6-basierter Pixel-Art-Editor in [`gamebasic/spriteeditor_qt.py`](gamebasic/spriteeditor_qt.py)
(UI-Schicht, 4200 LOC) plus Submodul [`gamebasic/spriteeditor/`](gamebasic/spriteeditor/)
mit `document.py` (Datenmodell), `tools.py` (Pixel-Tools), `tool_context.py`
(Tool-Host-Protocol), `icons.py` (programmatische Toolbar-Icons).

**Start:** `gbsprites` (leer) oder `gbsprites datei.png`. Aufruf-Trampoline in
`gbsprites.cmd` → `gbrun.py --sprites`. User-Doku: [docs/sprite-editor.md](docs/sprite-editor.md).

**Tools:** Pencil, Eraser, Bucket, Line, Rect, Ellipse, Eyedropper, Select,
Move, Magic Wand, Spray. Plus Multi-Frame-Animation, Onion-Skin, Symmetrie
X/Y, Tile-Preview-3×3, Palette-Im-/Export (.gpl), Sheet-Import, Crop, Resize,
Farbe-Ersetzen, Flip/Rotate.

**Export-Formate** (alle in `SpriteDoc.save_*`-Methoden):
- `save_native(path)` — .gbsprite (JSON + base64-RGBA pro Frame, mit Frame-Dauern)
- `save_png_single(path)` — einzelnes Frame
- `save_sheet_png(path, layout)` — horizontaler oder vertikaler Sheet
- `save_animated_gif(path, fps, loop)` — GIF mit Transparenz
- `save_sheet_atlas(png_path, json_path, name_prefix, layout)` — **PNG + JSON-Manifest** im Format, das `ATLAS_LOAD(...)` direkt versteht (siehe Sprite-Atlas-Section). Closed-Loop-Workflow: Editor schreibt, Engine liest.

**Atlas-Export-Detail:** Sprite-Namen sind `<png_basename>_<idx>` (z.B. PNG `tiles.png`
→ Sprites `tiles_0`, `tiles_1`, ...). Wer eigene Namen will, editiert die JSON
nach dem Export — oder zukuenftig: per-Frame-Name im Editor (noch nicht
implementiert, `Frame`-Class hat kein `name`-Feld).

**Tests:** `tests/test_spriteeditor_document.py` (Datenmodell, alle Export-Pfade,
inkl. Atlas-Roundtrip durch `ATLAS_LOAD`), `tests/test_spriteeditor_tools.py` (Pixel-Ops,
Bresenham, Brush-Offsets, Symmetrie), `tests/test_spriteeditor_tool_context.py`
(ToolHost-Protocol). 50+ Tests.

**Erweiterung:** neue Tools subclassen `Tool` in `tools.py`, implementieren
`begin/move/end`, registrieren sich in `SpriteEditorWindow._setup_tools()`.
Tool-Konvention im `tools.py`-Header dokumentiert.

## Build und Test (mit Cython-Modulen)

```
.venv\Scripts\python.exe setup.py build_ext --inplace
.venv\Scripts\python.exe -m pytest tests/ -v
.venv\Scripts\python.exe gbrun.py --bench examples/<file>.gb
```

`build_ext` baut drei `.pyx`: `vm_native`, `array_native`,
`modules/ecs_native`. Bei "Zugriff verweigert" auf .pyd-Copy:
hung Python-Prozess killen (`Stop-Process -Name python`), erneut
versuchen.

## Migration-Notizen

Alle Built-ins (~100+ core inkl. Array-/Map-Helfer + ~30 graphics inkl.
Bulk-Plot/-Shapes + Modul-Built-ins) sind über den Decorator-Mechanismus
registriert. Vorher gab es ~10 Zeilen Boilerplate pro
Funktion (arity-Check, Type-Check, Dict-Eintrag); jetzt ist es eine Decorator-Zeile
plus die eigentliche Logik. Migration-Detail-Status (was vor/nach dem Refactor
existierte): aus `git log` nachvollziehbar.
