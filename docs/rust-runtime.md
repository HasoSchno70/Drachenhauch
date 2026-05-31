# Native Rust-Runtime (raylib) — Migration

Ein vierter Ausführungspfad neben Tree-Walker, Python-VM und Cython-VM: eine
**native Rust-Runtime**, die denselben Bytecode ausführt und (später) Grafik
über **raylib** rendert. Die Python-Toolchain (Lexer → Parser → Compiler)
bleibt unverändert — Rust übernimmt nur die Ausführung des kompilierten
`Module`s.

## Migrationsplan (inkrementell, nichts wegwerfen)

1. **Bytecode-Format einfrieren/serialisieren** — `.gbc` schreiben (Python) +
   lesen (Rust). ✅ *erledigt (Spike)*
2. **Rust-VM-Kern** — Dispatch-Schleife + Skalar-Ops + Control-Flow. Test:
   Konsolen-Programme, `stdout == Python-VM`. ✅ *erledigt (Spike)*
3. Strings/Arrays/Maps/Structs in Rust nachziehen (echte Rust-Typen statt
   geboxt — die Typ-Strenge von GB hilft). ✅ *erledigt*
4. raylib einbinden, Grafik-Builtins nach Rust. ✅ *erledigt (Core-2D)*
5. Module portieren (gui/ui/physics …) nach Bedarf. 🚧 *in Arbeit (vec2/curves/physics)*
6. 3D-Builtins auf raylibs Mesh/Kamera-API.
7. Editor: „Export → native Exe bundeln".

## Schritt 1: `.gbc`-Serialisierung

[`gamebasic/serialize.py`](../gamebasic/serialize.py) wandelt ein vom
`Compiler` erzeugtes `Module` ([bytecode.py](../gamebasic/bytecode.py)) in eine
selbstbeschreibende JSON-Datei. JSON ist für den Spike bewusst gewählt
(debuggbar); ein kompaktes Binärformat ist später drop-in möglich.

**CLI:**
```
.venv\Scripts\python.exe -m gamebasic.serialize [--pretty] <datei.gb> [out.gbc]
```

> **Wichtig:** `serialize.py` importiert `interpreter.py`, bevor es kompiliert.
> Der Compiler entscheidet anhand der `BUILTINS`-Registry (von den `@builtin`-
> Decorators in `interpreter.py` gefüllt), ob ein Identifier-Call wie `LEN(a)`
> zu `CALL_BUILTIN` wird. Ohne diesen Import sähe der Compiler die Registry
> nicht und erzeugte für `LEN`/`INT`/… kaputten `LOAD_NAME + CALL_VALUE`-
> Bytecode. `gbrun.py` importiert `interpreter` am Modulanfang — dieselbe
> Umgebung wird hier hergestellt.

**Wert-Encoding** (eindeutig dekodierbar — INT/FLOAT/BOOL müssen unterscheidbar
bleiben, da `bool` in Python ein `int`-Subtyp ist und `1` ≠ `1.0` in GB):

| GB-Wert | JSON |
|---|---|
| `None` | `null` |
| `bool` | `{"b": true}` |
| `int` | `123` (Plain-Zahl) |
| `float` | `{"f": 1.5}` |
| `str` | `"text"` |
| `tuple`/`list` | `[…]` |
| `COMP_MARKER` | `{"comp": true}` |
| `_FuncRef` | `{"funcref": "name"}` |

Code-Instruktionen: `[op:int, arg]`, `arg` mit demselben Encoding. Die Rust-VM
weiß pro Opcode, welche Struktur `arg` hat (Index, Slot, Name, Tupel).

**Noch nicht serialisierbar:** Laufzeit-Handles im const-Pool
(IMAGE/SOUND/MAP/ARRAY/Instanzen). Diese kommen mit Schritt 3.

## Schritt 2: Rust-VM-Kern

Standalone-Crate [`rust/gb_runtime/`](../rust/gb_runtime) (getrennt vom
PyO3-Helper-Crate `rust/gb_native/`). Binary `gbrt`.

```
cd rust/gb_runtime
cargo build --release
target/release/gbrt <datei.gbc>
```

**Implementiert:** LOAD_CONST/POP/DUP, Locals (LOAD/STORE/DECLARE), Global-Slots
(LOAD/STORE/DECLARE/DECLARE_CONST), Name-Globals (Fallback), volle Skalar-
Arithmetik (ADD/SUB/MUL/DIV/MOD/POW/NEG/INT_DIV) inkl. der spezialisierten
`_NN`-Opcodes, Vergleiche, Bitwise, Control-Flow (JUMP/JUMP_IF_*), User-Calls
(CALL_USER/RETURN/RETURN_VOID), PRINT, HALT.

**Bit-Identitäts-kritische Semantik** (1:1 aus `gamebasic/vm.py`):
- `_fmt`: `NIL`/`TRUE`/`FALSE`; float-ganzzahlig → `{:.1f}` (immer Fixpunkt,
  auch `1e16` → `10000000000000000.0`), nicht-ganzzahlig → Python-`repr` mit
  E-Notation-Schwelle (`decpt <= -4` oder `> 16`), nachgebaut über Rusts `{:e}`.
  Bit-identisch verifiziert für kleine (`1e-09`), normale und große Beträge.
- DIV: int/int mit Rest 0 → int, sonst float; `b==0` wirft.
- INT_DIV (`\`): Trunkierung Richtung 0 (= Rust `i64`-Division).
- MOD: Ergebnis-Vorzeichen = Vorzeichen des Divisors (Python-Semantik).
- integer-Coercion: float-ohne-Nachkomma OK, `bool` wirft.

**Seit Schritt 3 zusätzlich:** Name-Globals (`DECLARE_NAME`/`DECLARE_CONST`),
Tupel (`BUILD_TUPLE`/`UNPACK_TUPLE`/`BUILD_TUPLE_DYN`), `IN`, Slicing, Arrays
(multidim, `LOAD/STORE_INDEX`, `DECLARE_ARRAY_*`), Maps, String-Index, OOP
(`NEW_INSTANCE`, Felder, Member, Methoden, `LOAD_SELF`, Properties, Operator-
Overloading, Structs), FUNCREF (`LOAD_FUNCREF`/`CALL_VALUE`), Container-Methoden
(`"x".upper()`), DATA/READ (`PUSH_DATA`/`RESET_DATA_PTR`), `TRY`/`THROW`/`CATCH`,
und eine **Registry pure/deterministischer Builtins** (`builtins.rs`): `STR$`,
`VAL`, `INT`, `ABS`, `LEN`, `CHR$`/`ASC`, `SQR`, `SIN`/`COS`/`TAN`/`ATAN`/
`ATAN2`, `FLOOR`/`CEIL`/`ROUND` (Banker's), `LOG`/`EXP`/`POW`, `MIN`/`MAX`/
`CLAMP`/`SIGN`, `UPPER$`/`LOWER$`, `LEFT$`/`RIGHT$`/`MID$`/`INSTR`/`REPLACE$`/
`TRIM$`/`SPLIT$`/`JOIN$`, `PADL$`/`PADR$`/`SPACE$`/`REPEAT$`/`HEX$`/`FORMAT$`,
`RGB`, `MAP*`, `SORT`/`REVERSE`/`ARRAY_INDEXOF`, `RANGE`, Comprehension-Helfer.

**Seit Schritt 4 zusätzlich:** `RND`/`RANDOMIZE`/`MILLIS`/`TIMER` (PRNG bzw.
SystemTime — NICHT bit-identisch, per Definition).

**Noch nicht im Kern:** Datei-I/O, ENUM/STATIC-Namespaces (deren const-Pool-
Handles noch nicht serialisierbar sind), Module (`IMPORT` → Schritt 5),
Bulk-Draws (`PLOTS`/`BOXES`/…), Layer/Atlas, Audio.

### Validierung

`stdout` der Rust-VM ist bit-identisch zur Python-VM (modulo OS-Newline:
Python schreibt auf Windows `\r\n`, Rust `\n` — semantisch identisch).
Verifiziert per Vollsweep: **30 Beispiele bit-identisch** (inkl. OOP, Arrays,
Maps, Tupel, Strings, alle Benchmarks), 0 echte Mismatches.

## Schritt 4: raylib-Grafik

Grafik ist **feature-gated** (`graphics`, default aus): der pure VM-Kern baut
ohne C-Toolchain. Mit Grafik wird [`raylib`](https://crates.io/crates/raylib)
(raylib-rs 5.5) eingebunden.

**Bit-Identität gilt NICHT für Pixel** (raylib ≠ pygame-Renderer) — nur
`PRINT`/stdout bleibt bit-identisch. Grafik wird per Screenshot verifiziert.

### Build (mit Grafik)

raylib kompiliert seine C-Quellen via **cmake** und braucht **libclang** für
die FFI-Bindings (bindgen). Der Helfer setzt die Umgebung:

```
.venv\Scripts\python.exe rust\build_runtime.py            # release, mit Grafik
.venv\Scripts\python.exe rust\build_runtime.py --no-graphics
```

Voraussetzungen (Windows): VS C++ Build Tools (liefern `cl.exe` + gebündeltes
cmake), LLVM für `libclang.dll` (`winget install LLVM.LLVM`). `cl.exe` findet die
`cc`-Crate automatisch; cmake-PATH und `LIBCLANG_PATH` setzt `build_runtime.py`.

### Builtins ([`graphics.rs`](../rust/gb_runtime/src/graphics.rs))

`SCREEN`, `CLS`, `FLIP`, `PLOT`, `LINE`, `BOX` (gefüllt), `RECT` (Umriss),
`CIRCLE`, `TRIANGLE`/`TRIANGLEOUTLINE`, `ELLIPSE`/`ELLIPSEOUTLINE`, `ARC`,
`POLYGON`/`POLYGONOUTLINE`, `TEXT`/`TEXT_SIZE`/`TEXT_WIDTH`/`TEXT_HEIGHT`,
`TEXT_BOLD`/`TEXT_ITALIC` (No-Op — Default-Font), `LOADIMAGE`/`DRAWIMAGE`/
`IMAGEWIDTH`/`IMAGEHEIGHT`, `KEYPRESSED`, `MOUSEX`/`MOUSEY`/`MOUSEBUTTON`,
`QUITREQUESTED`, `SLEEP`, `SET_FULLSCREEN` (No-Op headless).

**Bulk-Draws:** `PLOTS`/`BOXES`/`CIRCLES`/`LINES` (Koordinaten-Arrays + Farbe als
INT oder ARRAY). **Bilder erweitert:** `DRAWIMAGEPART`, `DRAWIMAGEFLIPPED`,
`LOAD_ASSETS` (Manifest-Vorladen + Alias/Pfad-Cache, `LOADIMAGE("alias")` trifft).
**Z-Layer:** `LAYER_DEFINE`/`LAYER`/`LAYER_END`/`LAYER_CLEAR` — FLIP komponiert
alle Layer aufsteigend nach z. **Sprite-Atlas:** `ATLAS_LOAD` (JSON-Manifest:
`{"image":..., "sprites":{name:[x,y,w,h]}}`), `ATLAS_DRAW`/`ATLAS_DRAW_FLIPPED`,
`BATCH_DRAW`/`BATCH_FLUSH` (im Recording-Modell flusht alles beim FLIP).

**Modell:** Draw-Builtins hängen `Cmd`s an eine Liste; `CLS` leert sie + merkt
die Clear-Farbe; `FLIP` rendert alle Cmds in einem `begin_drawing`-Block und
präsentiert. So muss kein raylib-Draw-Handle über Builtin-Aufrufe gehalten
werden. Farben sind `0xRRGGBB`-INTEGER → raylib `Color`.

**Vordefinierte Globals:** Farben (`BLACK`/`WHITE`/`RED`/…), Tasten (`KEY_*` als
SDL/pygame-Keycodes, in `KEYPRESSED` auf raylib-Keys gemappt) und `PI` werden
mit Python-identischen Werten vorregistriert (`register_default_globals`).

### Headless-Verifizierung

`gbrt` rendert headless und schreibt einen Screenshot, gesteuert per ENV:

```
GBRT_FRAMES=3 GBRT_SCREENSHOT=out.png gbrt programm.gbc
```

Nach `GBRT_FRAMES` Frames liefert `QUITREQUESTED()` `true` (Loop endet sauber);
beim Erreichen der Grenze wird das PNG gespeichert (auch wenn das Programm eine
feste `FOR`-Schleife statt `QUITREQUESTED` nutzt). **Hinweis:** raylibs
`TakeScreenshot` legt die Datei relativ zum Arbeitsverzeichnis ab.

Verifiziert (visuell per Screenshot) an allen 7 IMPORT-freien Grafik-Beispielen:
`30_shapes` (alle Formen), `44_language_showcase`, `34_schneefall`,
`39_textscroll`, `40_parallax`, `75_preloader` (LOAD_ASSETS + Alias-Cache),
`76_layers_atlas` (ATLAS_LOAD + BATCH_DRAW + Z-Layer-Compositing).

## Schritt 5: Module (IMPORT)

Modul-Builtins erreichen den Compiler automatisch: `IMPORT "x"` lädt im
Preprocessor das Python-Modul `gamebasic.modules.x`, dessen `@builtin`-
Decorators die `BUILTINS`-Registry füllen → der Compiler emittiert
`CALL_BUILTIN`. Die Rust-VM muss nur die jeweiligen Builtins implementieren —
**keine Serializer-Änderung nötig**.

**Portiert (pur, bit-identisch verifiziert):**
- `vec2` — `Value::Vec2(f64,f64)` (immutabler Wert-Typ) + Operator-Overloading
  (`+`/`-`/`*`/`/` via `module_op`, vor User-Operatoren). `VEC2_NEW/ZERO/X/Y/
  LENGTH/LENGTH_SQ/NORMALIZE/DOT/CROSS/DISTANCE/LERP/PERP/REFLECT/ANGLE/FROM_ANGLE`.
- `curves` — `CURVE_LERP/SMOOTHSTEP/SMOOTHERSTEP/BEZIER/BEZIER2/CATMULL/CATMULL2/HERMITE`.
- `physics` (pure) — `PHYSICS_BOX_BOX/CIRCLE_CIRCLE/BOX_CIRCLE/POINT_BOX/POINT_CIRCLE/
  DISTANCE/DISTANCE2/LENGTH/NORM_X/NORM_Y/REFLECT_X/REFLECT_Y/RAY_BOX/RAY_CIRCLE`.
  (Broadphase mit externem Typ noch nicht.)
- `input` — `INPUT_BIND/UNBIND/RESET/UPDATE/HELD/PRESSED/RELEASED/AXIS/BOUND`.
  Edge-Detection über prev/cur-Snapshots; Tastenstatus via raylib (`INPUT_UPDATE`
  ohne Fenster = keine Tasten, wie pygame ohne Display → Konsolen-Demos
  bit-identisch). Gamepad (`INPUT_JOY_*`) noch nicht (`INPUT_JOY_COUNT`=0).
- `camera` — `CAMERA_SET/RESET/X/Y/ZOOM/FOLLOW/S2W_X/S2W_Y`. World→Screen-Transform
  (`w2s`/`ssize`) wird in allen Draw-Methoden angewandt; TEXT-Position transformiert,
  Font-Größe bleibt. 29_camera_visual rendert korrekt.
- `sprite` — `Value::Sprite` (Sheet-Animation). `SPRITE_NEW/SET_POS/SET_VELOCITY/
  GET_X/Y/WIDTH/HEIGHT/SET_FLIP/SET_SCALE/TINT/TINT_CLEAR/ADD_ANIM/PLAY/PLAY_ONCE/
  CURRENT_ANIM/IS_FINISHED/SET_FRAME/GET_FRAME/UPDATE/COLLIDES/HIT_BOX/HIT_POINT`
  (in `builtins.rs`) + `SPRITE_DRAW` (Sheet-Frame als Sub-Rect, Camera-aware,
  Flip/Scale/Tint, in `graphics.rs`). 31_sprite_visual + 66_sprite_editor rendern.
  *Grenze:* Konsolen-Demos ohne `SCREEN` gehen nicht (Texturen brauchen GL-Kontext).
- `tween` — `Value::Tween`, 19 Easings, `TWEEN_NEW/_LOOP/_PINGPONG/VALUE/PROGRESS/
  DONE/RESTART/PAUSE/RESUME/REVERSE/EASINGS`. **Zeitbasiert** (Wall-Clock) → wie
  `RND`/`MILLIS` NICHT bit-identisch, aber funktional (47_collision_easing rendert).
- `json` — `Value::Json` (serde_json mit `preserve_order`). `JSON_PARSE/LOAD/
  STRINGIFY/PRETTY/GET_STRING/INT/FLOAT/BOOL/HAS/LEN/TYPE`, Pfad-Navigation
  (`"user.name"`/`"items.0"`). Bit-identisch inkl. `STRINGIFY` (Key-Reihenfolge).

- `scene` — Stack-Scene-Manager (VM-globaler State). `SCENE_PUSH/POP/SWITCH/
  CURRENT/DEPTH/HAS/RESET/SET_INT|FLOAT|STRING|BOOL/GET_*(_OR)/HAS_KEY/DELETE`.
  Bit-identisch.
- `save` — Save-Slots (`Value::Save`). `SAVE_NEW/LOAD/LOAD_OR_NEW/EXISTS/WRITE/
  DELETE_FILE/VERSION/SET_VERSION/SET_*/GET_*(_OR)/HAS/DELETE/CLEAR/KEYS`.
  JSON-Datei-Backend (serde_json pretty). Bit-identisch (inkl. Datei-Roundtrip).

**Core File-I/O** (kein Modul): `OPENFILE`/`CLOSEFILE`/`READLINE`/`READALL$`/
`ENDOFFILE`/`WRITELINE`/`WRITE`/`FILEEXISTS` (`Value::File`). Bit-identisch.

- `astar` — A*-Pathfinding (`Value::AStar`, [astar.rs](../rust/gb_runtime/src/astar.rs)
  portiert aus dem PyO3-Helper `gb_native`). `ASTAR_NEW/CLEAR/WIDTH/HEIGHT/SET_WALL/
  SET_PASSABLE/IS_WALL/SET_DIAGONAL/SET_HEURISTIC/SET_DIAGONAL_COST/FIND/PATH_LEN/
  PATH_X/PATH_Y/PATH_COST/CLEAR_PATH`. Bit-identisch inkl. counter-FIFO-Tie-Break.

- `particles` — `Value::Particles`. `PARTICLE_SYSTEM_NEW/SET_POS/COUNT/CLEAR/
  SET_VELOCITY/SET_LIFETIME/SET_GRAVITY/SET_COLOR/SET_SIZE/SET_FADE/SET_MODE/
  SET_COLOR_END/EMIT/UPDATE/DRAW`. 5 Render-Modi (circle/pixel/square/streak/
  glow), Fade + Farbverlauf. RNG-Emit → wie `tween` zeitabhängig/nicht
  bit-identisch, aber funktional (78_particle_catalog rendert alle Modi).

- `imgfx` — `IMAGE_SCALE/ROTATE/FLIP/TINT/COPY` (immutable, neues Handle). Über
  raylib-`Image` (CPU-Pixel) transformiert + `LoadTextureFromImage`. Texturen
  werden als `Tex { tex, img }` (GPU+CPU) gehalten.

- `ecs` — Entity-Component-System ([ecs.rs](../rust/gb_runtime/src/ecs.rs),
  Sparse-Set). `ECS_NEW_WORLD/NEW_ENTITY/DESTROY/ALIVE/COUNT`, `ADD_INT/FLOAT/
  STRING/BOOL/OBJ`, `HAS/REMOVE/GET*/GET_OR_*`, `QUERY/QUERY2/QUERY3` (sortierte
  Intersection), Bulk-Ops `INTEGRATE_FLOAT/INT/SCALE_FLOAT/FILL_*/CLAMP_FLOAT/
  REMOVE_DEAD/COUNT_WITH`. Bit-identisch.

- `ui` (Immediate-Mode) — `UI_LABEL/BUTTON/CHECKBOX/SLIDER/PROGRESS/PANEL/RADIO/
  END_FRAME/RESET` + Theme (`UI_THEME_SET/GET`, `UI_METRIC_SET/GET`,
  `UI_THEME_PRESET` dark/light/retro/contrast). State per String-ID auf der VM.
  **Neu portiert:** `UI_TEXTFIELD`/`UI_TEXTFIELD_SET` (Tastatur via neuem
  `Graphics::pop_text_input` = raylib `get_char_pressed`-Drain, Backspace-Edge,
  blinkender Caret) und `UI_WINDOW_BEGIN/END` (verschiebbare Immediate-Mode-
  Fenster: Offset-Threading durch ALLE Widgets via `UiState.offset_x/y`,
  Input-Gating überdeckter Fenster via `ui_mouse_gated`, Titel-Drag +
  Einklapp-Pfeil, Z-Order über Vorframe-Hit-Test `active_win`/`hover_win` in
  `UI_END_FRAME`). Beide per Headless-Screenshot verifiziert.
  *Noch vertagt:* **`UI_TABLE`** (Scrolling, Spaltenbreiten, Zell-Farben, zwei
  Scrollbars mit Drag, klickbare Zeilen — ~300 LOC, eigener Pass).

**Noch offen (Module):** `UI_TABLE`, `gui` (Retained-Mode, 859 LOC).
Hardware/Netzwerk + `regex` bleiben außen vor.

**ENUM / STATIC CONST:** `_EnumNamespace`/`_ClassStaticNamespace` werden als
`{"ns": {name, members}}` serialisiert und in Rust als `Value::Namespace`
geladen; `LOAD_MEMBER` löst Member case-insensitiv auf. Damit gibt es **keine
serialize-Fehler mehr** (vorher 6).

**Nicht geplant:** Hardware/Netzwerk-Module (`bt`/`serial`/`usb`/`wifi`/`net`/
`html`/`db`/`audio`) und `regex` (Python-`re` nicht bit-identisch nachbaubar).
