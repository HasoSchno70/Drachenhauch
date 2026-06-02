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
5. Module portieren (gui/ui/physics …). ✅ *erledigt — ALLE Module nativ inkl.
   ui (komplett, mit UI_TABLE) + gui (komplett, mit GUI_TABLE + Callbacks)*
6. 3D-Builtins auf raylibs Mesh/Kamera-API. ✅ *Core-Primitive erledigt (`g3d`)*
7. Editor: „Export → native Exe bundeln". ⬜ *offen*

**Dev-Run-Loop** (quer zu den Schritten): `gbrun.py --native <datei.gb>` —
ein Befehl kompiliert (Python) → `.gbc` → startet `gbrt`. ✅ *erledigt* (siehe
unten).

## Offen / nächste Schritte (Stand 2026-06-01)

Schritte 1–6 fertig; zusätzlich nativ: **Audio inkl. echter FFT** (`AUDIO_FFT`),
**Game-Loop** (`DELTA`/`FPS`/`SETFPS`/`SET_FULLSCREEN`/`SETWINDOWTITLE`/
`SAVESCREENSHOT`), **Shader/Post-Processing** (`SHADER_LOAD`/`SET`/`SET2`/`SET3`/
`POSTFX`, CRT/Bloom/Vignette), **TTF-Fonts** (`LOADFONT`/`SETFONT`/
`TEXT_SPACING`). Die native Runtime deckt damit ein komplettes
2D/3D-Spiel mit Sound, Menüs/Tabellen und GPU-Effekten ab.

**Lohnende nächste Hebel (raylib bietet noch mehr):**
- **3D weiter vertiefen:** 3D-Kameramodi (`UpdateCamera` orbit/first-person),
  Billboards, Ray-Kollision (`GetRayCollisionBox/Sphere/Mesh`),
  Beleuchtung/Material-Shader. (Modell-Laden, GenMesh-Primitive inkl. Heightmap
  sind nun da — siehe unten.)
- **Gamepad:** `IsGamepadButtonDown/Pressed` + `GetGamepadAxisMovement`
  (`JOY_*`/`INPUT_JOY_*` — input-Modul-Lücke, `INPUT_JOY_COUNT`=0 nativ).
- **Schritt 7 — Editor-Export:** `gbrt` + `.gbc` zu einer standalone `.exe`
  bündeln (Spiele ohne Python ausliefern).
- Mittel: Blend-Modes (additiv/multiply), Render-Texturen als GB-Handle,
  2D-Extras (dicke Linien/Gradient/runde Rechtecke/Splines), prozedurale
  Texturen, Sound-Pan/Aliase, Datei-Drag&Drop/Clipboard.

**Bewusst außen vor:** `regex` + Hardware/Netz (`bt`/`serial`/`usb`/`wifi`/
`net`/`html`/`db`); das erweiterte `audio`-Modul (Kanäle/Pan/Ton-Generierung)
bleibt Python-only (Core-Audio ist nativ).

## Dev-Run-Loop: `gbrun.py --native`

Für schnelles Iterieren beim Coden gibt es einen One-Command-Pfad:

```
.venv\Scripts\python.exe gbrun.py --native examples\30_shapes.gb
```

Das kompiliert die `.gb`-Datei (Lexer/Parser/Compiler bleiben in Python),
serialisiert sie in eine **temporäre `.gbc`** und startet `gbrt` im Verzeichnis
der Quelldatei (damit relative Asset-Pfade wie `LOADIMAGE("assets/…")`
stimmen). stdout/stderr und ein etwaiges Grafik-Fenster werden direkt
durchgereicht; der Exit-Code von `gbrt` wird weitergegeben. Fehlt das Binary,
verweist die Meldung auf `rust\build_runtime.py`.

So bleibt Python die Toolchain (und später nur noch der Editor), während die
Ausführung nativ läuft — kein manuelles `serialize` + `gbrt` mehr.

**Im Editor:** Toolbar-/Menü-Button **„Run nativ (gbrt)" (F6)**. Der Editor
kompiliert die Datei in-process in eine temporäre `.gbc` und startet `gbrt`
**direkt** als `QProcess` (nicht über `gbrun.py`) — so beendet der `Stop`-Button
auch den nativen Prozess (kein verwaister gbrt). Output und Laufzeitfehler
(`datei.gb:Zeile`, klickbar) landen in derselben Konsole wie der Python-Run.

### Laufzeitfehler mit Zeilennummer

Der Compiler stempelt pro Bytecode-Instruktion die **Quell-Zeile** (`stmt.line`
vom Parser) in ein zu `code` paralleles `lines`-Array (`CompiledFunction.lines`,
serialisiert als `"lines"` in der `.gbc`). Die Rust-VM merkt sich die Zeile der
zuletzt ausgeführten Instruktion (`Vm.cur_line`); bei einem propagierenden
Fehler bleibt die **innerste** fehlschlagende Zeile stehen. `gbrun.py --native`
reicht den Quell-Dateinamen als 2. Arg an `gbrt` durch, sodass die Meldung lautet:

```
Laufzeitfehler in spiel.gb:42: Index 10 ausserhalb [0..2] in Dimension 0
```

Die Python/Cython-VMs ignorieren `lines` (additives Feld, kein Recompile nötig);
`gbrt <datei.gbc>` ohne Label nutzt den `.gbc`-Pfad. Zeile `0` (untracked) →
Meldung ohne Zeilenangabe.

**Compile-Fehler** (vor der Ausführung, in Python) tragen ebenfalls eine Zeile:
Parser-Fehler ohnehin, und `CompileError` wird zentral mit der Statement- bzw.
Deklarations-Zeile angereichert (`Compiler._at` + `_stmt`, via
`GameBasicError.set_line`). So zeigt der `--native`/F6-Pfad z. B.
`[Zeile 4] CompileError: SUB 'foo' bereits deklariert` — im Editor als
klickbarer Link in die Quelldatei.

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
Bulk-Draws (`PLOTS`/`BOXES`/…), Layer/Atlas. *(Diese sind inzwischen alle
implementiert — siehe die jeweiligen Schritte unten.)*

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
`QUITREQUESTED`, `SLEEP`.

**Game-Loop-Grundlagen** (in beiden Pfaden, pygame + raylib): `DELTA()`
(Sekunden seit letztem FLIP, framerate-unabhaengige Bewegung), `FPS()`,
`SETFPS(n)` (Ziel-Framerate, 0 = ungedrosselt), `SET_FULLSCREEN(an)` (nativ
echtes `ToggleFullscreen`, nicht mehr No-Op), `SETWINDOWTITLE(s)`,
`SAVESCREENSHOT(pfad)`. Nativ ueber raylibs `GetFrameTime`/`GetFPS`/
`SetTargetFPS`/`ToggleFullscreen`/`SetWindowTitle`/`TakeScreenshot`.

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
  **`UI_TABLE`** ist nun ebenfalls portiert: fixierte Kopfzeile, V/H-Scroll
  (Mausrad + Scrollbalken-Drag inkl. Track-Klick), Per-Zelle-Text- und
  -Hintergrundfarben, Hover-/Selektions-Highlight, klickbare Zeilen
  (Rueckgabe = geklickte Zeile) + `UI_TABLE_SELECTED`/`SET_SELECTED`/
  `HEADER_CLICK` (Sortier-Hook). State per id in `UiState.tables`; nutzt den
  neuen Clip-Stack + Mausrad. Verifiziert per Screenshot
  ([examples/43_ui_table.gb](../examples/43_ui_table.gb), Zell-Farben +
  beide Scrollbars).

**`gui` (Retained-Mode)** — Kern portiert ([gui.rs](../rust/gb_runtime/src/gui.rs)):
Window + Button/Label/Checkbox/Slider/TextInput/Panel, Drag/Z-Order/Fokus/Close,
programmierbares Theme (`GUI_THEME_SET/GET/PRESET`, `GUI_METRIC_SET/GET`,
`GUI_SET_COLOR`), Polling (`GUI_CLICKED/CHECKED/VALUE/TEXT`, Setter). Handles =
INTEGER (Window = Index, Widget = `(win<<20)|idx`); Z-Order über separate
`z_order`-Liste, damit Handles stabil bleiben. Render per Screenshot verifiziert
(`examples/45_gui.gb`); Interaktion ist ein 1:1-Port der (getesteten) Python-
Logik — headless nicht klickbar testbar. **FUNCREF-Callbacks**
(`GUI_ON_CLICK`/`GUI_ON_CHANGE`) feuern: `update()` sammelt ausgeloeste Handler
in `pending`, die VM leert die Queue nach `GUI_UPDATE` und ruft sie (parameter-
los) via `exec` auf — so kann ein Callback die GUI sicher verändern, neue
Events landen nächsten Frame.

**`GUI_TABLE`** ist nativ portiert: fixierte Kopfzeile, V/H-Scroll
(Mausrad + Scrollbalken-Drag), Hover-/Selektions-Highlight, klickbare Zeilen,
`GUI_TABLE_HEADERS/ROWS/COL_WIDTHS/SELECTED/SET_SELECTED/CLICKED/ROW_COUNT` +
`GUI_ON_CHANGE` bei Selektionswechsel. Layout aus einer Quelle (`table_geom`).
Dafür hat die Grafik neu: **Mausrad** (`pop_mouse_wheel`) und einen
**Clip-Stack** (`push_clip`/`pop_clip` → raylib-Scissor mit Verschnitt). Render
per Screenshot verifiziert ([examples/84_gui_table.gb](../examples/84_gui_table.gb));
Rust-Unit-Tests decken Callback-Queueing, Tabellen-Layout (`table_geom`) und
Press→Selektion ab (`build_runtime.py --test`). *Noch offen:* der Immediate-
Mode-`UI_TABLE` (separates Modul `ui`).

**Tabellen komplett:** `UI_TABLE` (Immediate-Mode) **und** `GUI_TABLE`
(Retained) sind nativ. Hardware/Netzwerk + `regex` bleiben außen vor.

**ENUM / STATIC CONST:** `_EnumNamespace`/`_ClassStaticNamespace` werden als
`{"ns": {name, members}}` serialisiert und in Rust als `Value::Namespace`
geladen; `LOAD_MEMBER` löst Member case-insensitiv auf. Damit gibt es **keine
serialize-Fehler mehr** (vorher 6).

**Nicht geplant:** Hardware/Netzwerk-Module (`bt`/`serial`/`usb`/`wifi`/`net`/
`html`/`db`) und `regex` (Python-`re` nicht bit-identisch nachbaubar). Das
erweiterte `audio`-Modul (Kanäle, Pan, Ton-Generierung) bleibt vorerst
Python-only — die **Core-Audio-Builtins** sind aber nativ (siehe unten).

## Audio (Core: SFX + Stream-Musik)

Native Audio über raylib (Modul `audio.rs`, feature-gated wie die Grafik —
raylib bundelt den Mixer mit). **Core-Builtins, kein `IMPORT` nötig:**
- `LOADSOUND(pfad$) -> SOUND` (Handle = INTEGER-Index), `PLAYSOUND(sound[,
  loops, lautstaerke])`, `STOPSOUND(sound)`.
- `PLAYMUSIC(pfad$[, loops, lautstaerke])`, `STOPMUSIC()` — ein Stream
  gleichzeitig; `gbrt` ruft `update_stream` pro `FLIP` (sonst stockt die
  Wiedergabe). Musik loopt (raylib-Default).

**Audio-Reaktivität (FFT):** `AUDIO_FFT(bands)` füllt ein `ARRAY OF FLOAT` mit
B logarithmisch verteilten Frequenzband-Pegeln (0..1) des **aktuell hörbaren**
Audios. Dafür hängt `Audio::new` via `AttachAudioMixedProcessor` einen
`extern "C"`-Callback an die gesamte raylib-Audio-Pipeline; der schiebt das
gemischte Mono-Signal in einen globalen Ringpuffer (`try_lock`, Audio-Thread
blockiert nie). `fft_bands` fenstert (Hann), rechnet eine eigene Radix-2-FFT
(1024), bündelt log-spaced, normalisiert per Auto-Gain und glättet per
Peak-Hold. Im pygame-Pfad (kein Mix-Tap) füllt `AUDIO_FFT` Nullen. So tanzen
Spektrum **und** Geometrie der Demo wirklich zur Musik.

WAV/OGG/MP3/FLAC je nach raylib-Build. Audio ist **nicht bit-identisch** zur
pygame-Version (anderer Mixer) — wie `RND`/`MILLIS`/`tween` nur funktional.
*Grenze:* `loops` wird nativ (noch) nicht ausgewertet — SFX spielen einmal,
Musik loopt immer. Lifetime-Trick: das `RaylibAudio`-Gerät wird per `Box::leak`
zu `&'static`, damit `Sound`/`Music` in `Vec`/`Option` gehalten werden können
(kein self-referential struct). Demo: [examples/83_audio.gb](../examples/83_audio.gb).

## Schritt 6: 3D-Grafik (Modul `g3d`)

3D ist **native-only**: raylib hat eine echte 3D-Pipeline, pygame nicht. Das
Modul `g3d` registriert die Builtins (damit der Compiler `CALL_BUILTIN`
emittiert); im Python/Tree-Walker-Pfad (F5) werfen sie eine klare Meldung
(„… nur in der nativen Runtime … mit F6"). In `gbrt` rendern sie über raylibs
`begin_mode3D`-API.

**Builtins** ([`g3d.py`](../gamebasic/modules/g3d.py), Rendering in
`graphics.rs`/`vm.rs`):
- `CAMERA3D(px,py,pz, tx,ty,tz, fovy)` — Perspektiv-Kamera (Up = +Y), pro Frame.
- `CUBE` / `CUBE_WIRES` `(x,y,z, w,h,d, farbe)` — gefüllter / Drahtgitter-Quader.
- `SPHERE` / `SPHERE_WIRES` `(x,y,z, r, farbe)`.
- `CYLINDER(x,y,z, r_oben, r_unten, h, farbe)` — `r_oben=0` ⇒ Kegel.
- `PLANE(x,y,z, size_x, size_z, farbe)` — XZ-Ebene.
- `LINE3D(x1,y1,z1, x2,y2,z2, farbe)`, `POINT3D(x,y,z, farbe)`.
- `GRID3D(linien, abstand)` — Boden-Raster.

**Render-Modell** (erweitert das 2D-Recording): 3D-Cmds landen in einer eigenen
Liste `cmds3d`; beim `FLIP` rendert `gbrt` **zuerst** alle 3D-Cmds in einem
`begin_mode3D(cam3d)`-Block, **danach** die 2D-Layer obenauf — das 2D-HUD liegt
also immer über der Szene. Koordinaten sind Welt-Einheiten (kein Screen-Scale),
Farben `0xRRGGBB`. `cmds3d` wird pro Frame geleert; ohne `CAMERA3D` gilt ein
Default-Blick (schräg von vorn-oben auf den Ursprung).

Demo: [examples/82_3d_intro.gb](../examples/82_3d_intro.gb) (Würfel, Kugel,
Zylinder, Kegel, Linien, Gitter + 2D-HUD), per Screenshot verifiziert.

### 3D-Modelle (geladen + prozedural)

Über die Immediate-Mode-Primitive hinaus gibt es **wiederverwendbare
Modell-Handles** (INTEGER), die einmal erzeugt und über beliebig viele Frames
gezeichnet werden — anders als `CUBE`/`SPHERE`, die jeden Frame neu aufgebaut
werden.

- `LOADMODEL(pfad$) -> MODEL` — lädt OBJ/GLTF/IQM/… via raylib `LoadModel`.
- **Prozedurale Meshes** (kein Asset nötig) via `GenMesh*` →
  `LoadModelFromMesh`: `MESH_CUBE(w,h,d)`, `MESH_SPHERE(r, ringe, segmente)`,
  `MESH_CYLINDER(r, h, segmente)`, `MESH_TORUS(r, dicke, rad_seg, seiten)`,
  `MESH_KNOT(r, dicke, rad_seg, seiten)`, `MESH_PLANE(w, l, res_x, res_z)`,
  `MESH_HEIGHTMAP(bild, groesse_x, groesse_y, groesse_z)` (Terrain aus einer
  Graustufen-Image: Helligkeit = Höhe, `groesse_y` skaliert sie).
  Torus/Knot sind neue Formen ggü. den Immediate-Primitiven.
- **Zeichnen:** `MODEL(m, x,y,z, scale, farbe)` (`DrawModel`),
  `MODEL_EX(m, x,y,z, achse_x,achse_y,achse_z, winkel_grad, scale, farbe)`
  (`DrawModelEx` — Rotation um eine Achse), `MODEL_WIRES(m, x,y,z, scale, farbe)`.
- `MODEL_TEXTURE(m, bild)` — ein via `LOADIMAGE` geladenes Bild als
  Diffuse-/Albedo-Map (`MATERIAL_MAP_ALBEDO`) auf das Modell legen.

**Umsetzung** ([graphics.rs](../rust/gb_runtime/src/graphics.rs)): `models:
Vec<Model>` lebt über die ganze Laufzeit (Handles bleiben gültig). Die Draw-
Builtins emittieren `Cmd3D::Model`/`ModelEx`/`ModelWires` mit dem Model-**Index**
(nicht dem Model selbst → `Cmd3D` bleibt `Clone`); der 3D-Pass in `render_scene`
(jetzt mit `models: &[Model]`-Parameter) zeichnet sie via `draw_model[_ex|_wires]`.
GenMesh-Meshes werden mit `make_weak()` an `load_model_from_mesh` übergeben (kein
Doppel-Drop). Handle-Validierung in den Wrappern (`check_model`).

Demo [examples/88_3d_models.gb](../examples/88_3d_models.gb): rotierender Torus,
Knoten (Wireframe) und pulsierende Kugel auf einer Ebene, umkreisende Kamera +
2D-HUD — rein prozedural, **kein Modell-Asset im Repo nötig**. Per Screenshot
verifiziert (inkl. `MODEL_TEXTURE` mit `assets/coin.png` auf einem Würfel).

`MESH_HEIGHTMAP` baut aus dem CPU-`Image` (`self.textures[i].img`, bereits für
imgfx gehalten) via `GenMeshHeightmap` ein Terrain-Mesh. Demo
[examples/89_heightmap.gb](../examples/89_heightmap.gb): texturiertes, von einer
Kamera umkreistes Terrain mit Drahtgitter-Overlay (`examples/assets/heightmap.png`,
ein generiertes 129×129-Graustufen-PNG). Per Screenshot verifiziert.

**Offen (3D):** Beleuchtung/Material-Shader, frei steuerbare Kamera-Modi
(`UpdateCamera`), Billboards, Ray-Kollision.

## Shader / Post-Processing (native)

GPU-Fragment-Shader fuer Ganzbild-Effekte (CRT, Bloom, Vignette, …) — **nur
native** (raylib/OpenGL). Builtins: `SHADER_LOAD(pfad$_oder_glsl$)` (Datei ODER
GLSL-Quelltext → SHADER-Handle/-1), `SHADER_SET`/`SHADER_SET2`/`SHADER_SET3`
(float/vec2/vec3-Uniforms), `POSTFX(h)` (Frame durch den Shader; -1 = aus).

**Render-Modell:** Ist ein Post-Shader aktiv, rendert `FLIP` die ganze Szene
(3D + 2D + Scissor) nicht direkt auf den Screen, sondern in eine
`RenderTexture2D`; danach wird diese Textur full-screen durch
`BeginShaderMode(shader)` praesentiert (Y-flip wegen RT-Konvention). Der
Replay-Code ist generisch (`fn render_scene<D: RaylibDraw>`), laeuft also
identisch auf den Screen *oder* in die RenderTexture — `RaylibDrawHandle` und
`RaylibTextureMode` implementieren beide `RaylibDraw`. Shader-Handles liegen in
`Graphics.shaders`, der aktive Index in `post_shader_idx`.

Im pygame-Pfad sind die Builtins No-Ops (`SHADER_LOAD` → -1) — das Programm
laeuft ohne Effekt statt zu craschen. Beispiel-Shader (GLSL 330):
[examples/assets/shaders/](../examples/assets/shaders/) (`crt.fs`/`bloom.fs`/
`vignette.fs`), Demo [examples/86_postfx_shaders.gb](../examples/86_postfx_shaders.gb)
(zyklisch AUS → CRT → BLOOM → VIGNETTE; CRT + Bloom per Screenshot verifiziert).

## TTF-Fonts (`LOADFONT` / `SETFONT` / `TEXT_SPACING`)

Eigene TrueType-/OpenType-Schriften statt nur des eingebauten Default-Fonts.
**Core-Builtins, kein `IMPORT` nötig** — in beiden Pfaden registriert:

- `LOADFONT(pfad$, groesse) -> FONT` — lädt eine TTF/OTF in der Basis-Größe
  `groesse` (Glyph-Auflösung) und liefert ein **FONT-Handle (INTEGER)**.
- `SETFONT(font)` — aktiviert den Font für nachfolgende `TEXT`-Aufrufe.
  `SETFONT(-1)` schaltet zurück auf den Default-Font.
- `TEXT_SPACING(px)` — Buchstabenabstand für TTF-Text (wirkt nativ über
  `DrawTextEx`; pygame ignoriert es als Näherung).

`TEXT_SIZE` skaliert den aktiven Font weiterhin frei (nativ skaliert raylib die
einmal geladene Glyph-Textur; pygame baut pro Größe eine `pygame.font.Font`).
`TEXT_WIDTH` misst in der **aktiven** Schrift (nativ `MeasureTextEx`) — damit
funktioniert Zentrieren/Rechtsbündig auch mit TTF. `TEXT_BOLD`/`TEXT_ITALIC`
wirken im pygame-Pfad (Synthese), nativ bleiben sie No-Op (raylib hat keine
synthetische Variante — dafür eine fette/kursive Font-Datei laden).

**Native Umsetzung** ([graphics.rs](../rust/gb_runtime/src/graphics.rs)): `fonts:
Vec<Font>` (raylib `load_font_ex`), `active_font` (-1 = Default), `text_spacing`.
`Cmd::Text` trägt jetzt Font-Index + Spacing; beim Replay zeichnet ein gültiger
Index via `draw_text_ex(font, …)`, sonst der Default-`draw_text`. **pygame-Pfad**
([graphics.py](../gamebasic/graphics.py)): `_get_font()` baut bei aktivem TTF ein
`pygame.font.Font(pfad, _font_size)` (pro Größe gecachet).

**Bit-Identität gilt nicht** (Renderer/Font-Metriken unterscheiden sich) — wie
bei der übrigen Grafik nur funktional. Es liegt **kein Font-Asset im Repo**;
Demo [examples/87_ttf_fonts.gb](../examples/87_ttf_fonts.gb) sucht einen
System-Font (`FILEEXISTS`) und fällt sonst auf den Default-Font zurück. Per
Screenshot verifiziert (Größen-Skalierung, Spacing, zentrierter Text via
`TEXT_WIDTH`).

## Showcase-Demo

[examples/85_cybermatic_demo.gb](../examples/85_cybermatic_demo.gb) bündelt in
einem 1280×720-Frame, was die native Runtime kann — **audio-reaktiv** (echte
FFT der laufenden Musik via `AUDIO_FFT`) und mit **Szenen-Wechsel alle 16
Takte**: `TUNNEL` (zufliegende Wireframe-Ringe) → `RING` (Doppelring + Bass-
Kugel + Säule, Kamera-Punch/Shake) → `PLASMA` (audio-reaktives Würfel-Terrain).
Dazu durchgehend ein 2D-Overlay: FFT-Spektrum (`BOXES`-Bulk, oben+unten),
Glow-Funken + Cyber-Regen (zwei Partikelsysteme), pulsierender Titel,
Laufschrift, dezenter Beat-Flash. Nur nativ:
`gbrun.py --native examples\85_cybermatic_demo.gb` (oder F6).

Das Musik-Asset (~15 MB, „Cybermatic pulse" von **Alexandr Zhelanov**,
CC-BY 4.0) liegt **nicht** im Repo (zu groß) — einmalig holen mit
`py examples/assets/download_cybermatic.py`. Die Demo läuft auch ohne (stumm,
via `FILEEXISTS`-Guard). Provenienz/Lizenz: `examples/assets/CREDITS_cybermatic.txt`.
