# Performance-Profile der drei VM-Pfade

> **⚠️ HISTORISCH (vor Stufe B).** Dieses Dokument vergleicht drei
> Ausfuehrungspfade — Tree-Walker, Python-Bytecode-VM, Cython-Native-VM —, die
> seit Stufe B **alle entfernt** sind. Die EINZIGE Runtime ist heute `gbrt`
> (Rust/raylib). Auch `gbrun.py --bench` existiert nicht mehr. Die Zahlen unten
> sind nur noch als Optimierungs-Logbuch interessant; gemessen wird heute gegen
> `gbrt`. Siehe [docs/rust-runtime.md](rust-runtime.md).

## gbrt-Optimierung 2026-06-11 (aktuell)

Performance-Offensive an der Rust-VM selbst — gemessen mit `bench_gbrt.py`
(best-of-5, Prozess-Wandzeit `gbrt run`, Windows 11):

| Bench | vorher | nachher | Speedup |
|---|---|---|---|
| 5M-Iterationen-Loop (Arithmetik) | 664 ms | 203 ms | **3.3×** |
| 6M Array-Lese/Schreib-Zugriffe | 1171 ms | 462 ms | **2.5×** |
| 1M Methodenaufrufe + Self-Feld | 439 ms | 133 ms | **3.3×** |
| 1M Builtin-Calls (ABS/SIN/MIN) | 276 ms | 126 ms | **2.2×** |
| 100× ARRAY_SUM auf 1M-Array (+Fill) | 227 ms | 103 ms | **2.2×** |
| fib(28), ~630k Calls | 186 ms | 123 ms | 1.5× |
| 200k String-Ops | 89 ms | 62 ms | 1.4× |

Die Hebel (Commits `de1eed8`, `0a4c5c7`, `3ae32ec`, `e24c9c9`, `f965389`):

- **Numerische Fast-Paths** in ADD/SUB/MUL/DIV/MOD + Vergleichen — Int/
  Float-Paare rechnen direkt, Modul-/User-Operator-Checks nur im Sonderfall.
  Int-Vergleiche dabei ohne f64-Umweg (korrekter ab 2^53).
- **1D-Index-Fast-Path**: LOAD/STORE_INDEX ohne split_off-Vec (vorher eine
  Allokation PRO Array-Zugriff), Coerce-Fast-Arm beim Element-Store.
- **Allokationsfreie Member-Zugriffe**: Member-Name als &str aus dem
  Const-Pool (statt fmt()-String pro Zugriff), is_property ohne
  to_lowercase, und der format!-Fehlerkontext bei STORE_MEMBER entsteht
  nur noch im Fehlerfall (vorher bei jedem erfolgreichen Store).
- **Frame-Vec-Pooling**: locals/stack-Vecs werden ueber Funktionsaufrufe
  hinweg wiederverwendet; Builtin-Args als Stack-Slice statt split_off.
- **Fusionierter `FOR_NEXT`-Opcode** (116): Inkrement + Weiter-Test +
  Ruecksprung in EINEM Dispatch statt ~9 Einzel-Instruktionen pro
  Iteration; traegt die FOR-Quellzeile (Profiler/Debugger unveraendert).
- **FxHashMap** (rustc-hash) fuer Funktions-/Klassen-/Methoden-/
  Instanzfeld-Lookups (interne Compiler-Keys, kein DoS-Vektor).
- **Typisierte Array-Backings** (`Cells`): ARRAY OF INTEGER/FLOAT speichern
  rohe `Vec<i64>`/`Vec<f64>` statt geboxter Values (8 statt ~24 Byte pro
  Element) -- ARRAY_SUM/FILL/SORT/DRAWTILEMAP/PLOTS laufen ueber rohe
  Slices (Commit `d831253`).
- **Vorab aufgeloeste Funktions-Indizes**: CALL_USER indiziert direkt in
  ein `Vec<Func>` statt pro Aufruf den Namen zu hashen (Commit `921fbf8`).

Semantik unveraendert (Sonderfaelle laufen die alten Pfade), 1910 Tests gruen.

---

Snapshot der relativen Performance der drei (historischen) Ausfuehrungspfade
(Tree-Walker, Python-Bytecode-VM, Cython-Native-VM) bei spielnahen Workloads.

System: Windows 11, Python 3.12.10, MSVC 14.50 fuer Cython-Build.

## Aktuelle Benches

| Benchmark              |  Tree-Walker |   Python-VM |   Native-VM | TW→Cy | Py→Cy | Ident |
|------------------------|--------------|-------------|-------------|-------|-------|-------|
| bench_loop             |     419.2 ms |    558.9 ms |     55.0 ms | 7.6×  | 10.2× | yes   |
| bench_fib              |    2125.3 ms |   1452.7 ms |    149.4 ms | 14.2× |  9.7× | yes   |
| bench_array_rw         |    4733.3 ms |   8130.5 ms |   1187.1 ms | 4.0×  |  6.8× | yes   |
| bench_method_dispatch  |    1706.0 ms |   2445.1 ms |    319.1 ms | 5.4×  |  7.7× | yes   |
| bench_builtin_math     |    3421.8 ms |   4432.1 ms |    733.5 ms | 4.7×  |  6.0× | yes   |
| bench_string_concat    |      63.7 ms |     78.6 ms |     24.8 ms | 2.6×  |  3.2× | yes   |
| bench_ecs_movement     |     655.2 ms |    805.5 ms |    215.1 ms | 3.0×  |  3.7× | yes   |
| **bench_ecs_movement_v2** | **13.3 ms**|  12.6 ms    |   **5.1 ms**| 2.6×  |  2.5× | yes   |
| bench_ecs_systems      |      39.2 ms |     40.2 ms |     21.0 ms | 1.9×  |  1.9× | yes   |
| bench_dict_comp        |      85.4 ms |    157.1 ms |     32.1 ms | 2.7×  |  4.9× | yes   |

`Ident: yes` heisst bit-identischer Output ueber alle drei Pfade.

`bench_ecs_movement_v2` zeigt das ECS-Bulk-System-Pattern: dieselbe
Workload wie `bench_ecs_movement` (500 Entities × 100 Frames), aber per
`ECS_INTEGRATE_FLOAT` statt pro-Entity-Loop in BASIC -- **40× schneller**.

## Umgesetzte Optimierungen

Chronologisch gewachsen ueber mehrere Runden:

### 1. Spezialisierte Numeric-Opcodes (`*_NN`)

11 neue Opcodes (`ADD_NN`, `SUB_NN`, `MUL_NN`, `DIV_NN`, `LT_NN`,
`GT_NN`, `LEQ_NN`, `GEQ_NN`, `EQ_NN`, `NEQ_NN`, `NEG_N`). Der Compiler
emittiert sie, wenn beide Operanden statisch als numerisch erkannt
sind (`_expr_type`-Inference auf Locals, Globals, BinaryOp-Result).
Der Hot-Path skippt damit Modul-Operator-Dispatch (`_disp_op` fuer
Vec2 etc.), User-Class-Operator-Dispatch (`_user_op`) und die
`isinstance`-Cascade. Bei `bench_loop` (tighter Numeric-Loop):
**1.46× Speedup** auf der Native-VM.

### 2. FOR-Loop-Bookkeeping als Spec-Ops

`_stmt_For` emittiert `ADD_NN`/`GT_NN`/`LT_NN` fuer Increment +
Bound-Check. Verstaerkt Punkt 1 bei jedem FOR-Loop.

### 3. Inline-Cache fuer OOP-Dispatch

`CompiledFunction.caches`-Liste parallel zu `code`. Monomorphic IC fuer
`CALL_METHOD`, `LOAD_MEMBER`, `STORE_MEMBER` auf `_Instance`.
Hit-Check: `obj.cls is cache[0]` (Pointer-Compare). Spart
`_resolve_method`-Aufruf und Dict-Lookup. **bench_method_dispatch:
1.34×** Speedup.

Anmerkung: in einer FRUEHEREN Iteration hat IC in der Python-VM nicht
funktioniert (zusaetzlicher Branch-Code in `_exec` hat mehr gekostet
als der gesparte Lookup). Die jetzige Implementation arbeitet auch in
der Cython-VM und gewinnt dort messbar -- der Cython-Wrapper macht
Attribut-Zugriff billiger als Dict-Lookup.

### 4. Typed Array Backing

`_GBArray` nutzt `array.array('q')` fuer `ARRAY OF INTEGER` und
`array.array('d')` fuer `ARRAY OF FLOAT` statt Python-Liste. Spart
Box/Unbox bei jedem Zugriff. Plus: cdef-Class `_GBArray` in
`array_native.pyx` mit typed memoryview (`long long[::1]` /
`double[::1]`) und `cdef inline _flat_c` fuer Bounds-Check + Stride-
Arithmetik in C. Neue Fast-Path-API `get_at(indices)` / `set_at(...)`,
die die VMs statt `arr.values[arr.flat_index(...)]` rufen.
**bench_array_rw: 1.51×** Gesamtspeedup ueber alle Aenderungen.

INTEGER-Werte sind auf 64-bit begrenzt (-9.2e18..9.2e18). Skalare
INTEGER-Variablen sind unbeeinflusst (arbitrary precision).

### 5. Cdef-Klassen fuer ECS (`_World`, `_Component`)

`gamebasic/modules/ecs_native.pyx`: Sparse-Set-Storage in cdef-class
mit cpdef-Methoden. `_World` hat Fast-Path-Methoden (`get_float`,
`add_float`, ...), die die gesamte `@builtin`-Wrapper-Pipeline in
einem cpdef-Call abwickeln. **bench_ecs_movement: 1.32×.**

### 6. ECS Bulk-System-Ops (der grosse Hebel)

Statt pro-Entity-Loop in BASIC mit 6 Builtin-Calls/Entity gibt es:
`ECS_INTEGRATE_FLOAT/INT`, `ECS_SCALE_FLOAT`, `ECS_FILL_FLOAT/INT`,
`ECS_CLAMP_FLOAT`, `ECS_REMOVE_DEAD`, `ECS_COUNT_WITH`. Jedes ist EIN
Builtin-Call, der die gesamte Component-Schicht in einer cdef-Loop
verarbeitet. `bench_ecs_movement_v2` ersetzt damit den BASIC-Loop:
**40× schneller** als die direkte Schreibweise. Diese Differenz zeigt,
wo der Hauptkostenpunkt fuer ECS-Code wirklich liegt -- nicht im
Storage, sondern im Builtin-Dispatch-Overhead.

### 7. Globals-as-Slots

Compile-Zeit-Aufloesung von Top-Level-DIM/CONST/Enum/For-var/Class-
Static-Namen zu Slot-Indizes. Neue Opcodes `LOAD_GLOBAL_SLOT`,
`STORE_GLOBAL_SLOT`, `DECLARE_GLOBAL_SLOT`, `DECLARE_GLOBAL_CONST_SLOT`.
Spart pro Zugriff einen Dict-Lookup auf `globals_`. Pre-registrierte
Globals (KEY_*, PI, COLORS) leben weiter im Dict (Compiler erkennt
sie nicht statisch). **bench_loop: 1.30×** zusaetzlich.

### 8. Constant Folding

`_try_fold(e)` im Compiler -- BinaryOp/UnaryOp mit konstanten Operanden
wird zu einem einzelnen `LOAD_CONST`. Konservativ (kein Folding bei
Bool-in-Arithmetik, Division durch 0, sehr grossen POW). Hilft
besonders `FOR i = 0 TO 100 - 1`-Patterns. **5-10%** im Schnitt.

### 9. Cython-VM Asset-Cache

`Graphics.load_image` / `load_sound` cachen unter rohem Pfad UND
absolutiertem Pfad. `LOAD_ASSETS(manifest_path)` pre-warmt den Cache
aus einem JSON-Manifest. Doppelte LOADIMAGE-Calls (z.B. in Scene-
Wechseln) sind danach kostenlos.

## Was nicht funktioniert hat (und warum)

**Inline-Cache in der Python-VM (zurueckgerollt in fruehen Iterationen,
neu integriert).** In der Python-VM ist jeder zusaetzliche `elif`-
Branch in `_exec` direkter Code-Path-Cost -- selbst wenn der neue
Branch gar nicht betreten wird. Die jetzige Implementation kommt
besser durch, weil sie in der Cython-VM dominant gewinnt und in der
Python-VM zumindest nicht verliert.

**Operator-Overload-Dispatch entfernen (verworfen).** Modul-Operator-
Registry (Vec2 etc.) + User-Class-Operator-Methoden sind in jedem
generischen `ADD`/`SUB` der vorgeschaltete Check. Sie ganz zu entfernen
wuerde Backwards-Kompatibilitaet brechen. Loesung: spec-ops (Punkt 1)
umgehen das, wenn Compiler statisch sieht, dass kein User-Type
involviert ist.

## Beobachtungen

**Native-VM gewinnt durchgaengig.** Speedup gegenueber Tree-Walker: 1.9×
bis 14.2×, im Median ~4×. Bei Function-Call-heavy Code (Rekursion) am
groessten.

**Python-VM ist haeufig langsamer als Tree-Walker.** Konsistent: bei
`bench_array_rw`, `bench_method_dispatch`, `bench_dict_comp` liegt die
Stack-VM 20–40% hinter dem Tree-Walker. Nur bei Function-Call-heavy
Code (`bench_fib`) zahlt sich der Compile-Pass aus. Erklaerung: der
Tree-Walker ruft Built-ins direkt; die Python-VM addiert pro Op
Stack-Push/Pop und Dispatch-Cost in einer langen `elif`-Kette.

**Bulk-Builtins schlagen alles andere.** `bench_ecs_movement` (pro-
Entity-Loop in BASIC) braucht 215 ms auf der Native-VM.
`bench_ecs_movement_v2` (selbe Workload, aber per
`ECS_INTEGRATE_FLOAT`) braucht 5 ms -- **43× schneller**. Lesson:
*wenn der Hot-Path ueber Builtin-Aufrufe geht, ist die Wahl der
Builtin-API wichtiger als jede VM-Optimierung*.

## Empfehlungen

**Default fuer Production: `gbrun.py --vm`** (Native-VM). 2.6–14×
schneller als der Tree-Walker.

**Tree-Walker fuer Entwicklung.** Schnelle Iteration, bessere
Stack-Traces, kein Compile-Step.

**Python-VM** ist primaer Reference-Implementation fuer den Cython-
Port. Wer das `.pyd` nicht baut, kriegt mit der Python-VM ein
funktionales Aequivalent (ohne Geschwindigkeitsvorteil).

**Game-Pattern: Bulk-Ops bevorzugen.** Statt pro-Entity-Loops in BASIC
mit ECS_GET/ADD: `ECS_INTEGRATE_FLOAT`, `ECS_SCALE_FLOAT` etc. Statt
einzelner `DRAWIMAGE`-Calls fuer Tiles: `DRAWTILEMAP` bzw. die Massen-
Builtins (`PLOTS`/`BOXES`/`CIRCLES`/`LINES`). **Nicht** `BATCH_DRAW`/`BATCH_FLUSH`
auf einem Sprite-Atlas.

## Reproduktion (historisch)

> Nicht mehr lauffähig: `setup.py build_ext` und `gbrun.py --bench` sind mit den
> Python-Pfaden entfernt. Heute misst man direkt gegen `gbrt`
> (`gbrt run examples/bench_fib.gb`).

```
python setup.py build_ext --inplace      # Cython-VMs bauen  (entfernt)
python gbrun.py --bench examples/bench_fib.gb                # (entfernt)
```

## Offene Pfade (mit ehrlicher Cost-Benefit-Einschaetzung)

| Idee | Erwartung | Aufwand | Kommentar |
|---|---|---|---|
| Computed-Goto-Dispatch (Cython) | 10–20% generell | Mittel | C-Switch via Cython 3 |
| Spezialisierter LOAD_INDEX_1D_INT/FLOAT | 1.5–2× auf bench_array_rw | Klein | 2 neue Ops + Inference |
| Builtin Pre-Resolution (Compiler) | 5–10% | Klein | name → fn-Pointer im arg |
| Frame-Pool fuer Funktions-Calls | 10–20% auf rekursive Workloads | Mittel | Heap-allocated frames im VM |
| Tracing-JIT | 5–20× je nach Pattern | Sehr hoch | PyPy-Niveau Komplexitaet |

Pragmatischer Rat: alles darueber hinaus ist Liebhaberei. Die aktuelle
Engine ist fuer 2D-Spiele schnell genug (60 fps mit tausenden Entities).
