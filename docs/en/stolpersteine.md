# Drachenhauch — pitfalls & inconsistencies (found while writing the textbook)

A collection of friction points in the language and the engine that came up
while writing `buch-referenz/` (all chapters and modules verified against
`dhrt`). **It concerns Drachenhauch itself, not the book** — a backlog for
fixing later. Every item has been reproduced against the current `dhrt`
release build.

Order ≈ by benefit/effort. Finished items at the end.

---

## A — Real gaps / common pitfalls (worth fixing)

### A1. No array literal `[1, 2, 3]`  —  ✅ FIXED (commit 5885dc0)
> The parser now disambiguates: FOR after the first expression = list comprehension,
> otherwise an array literal. Opcode `BUILD_ARRAY` (117), element type taken from the values
> (int/float/string/boolean/any). An empty `[]` -> hint at `DIM ... AS ARRAY OF T`.
> Tests: `tests/pruef/array_literal.dhtest` (`dhrt test`). (Historical text below for the record.)
```basic
DIM a AS ARRAY OF INTEGER
a = [1, 2, 3]      ' -> parse error (7): expected FOR in list comprehension
```
`[...]` is exclusively a list comprehension. There is **no** shorthand for
filling an array with values — you have to use `DIM a[3]` plus individual
assignments. Tuples have `(1, 2, 3)`, arrays have nothing comparable. A very
common expectation among beginners.
**Proposal:** disambiguate in the parser — if `[` is not followed by `expr FOR …`,
treat it as an array literal (`[1,2,3]` → `ARRAY OF INTEGER`, type from the elements).

### A2. `NIL` is not a literal  —  ✅ FIXED (commit f4c8b78)
> NIL is now a keyword literal (lexer/parser/compiler in dhrt + the Python front end
> for the editor/parity). `x = NIL`, `x <> NIL`, `IS_NIL(NIL)` work; the NIL→NULL binding
> promised in the db documentation now really works. Tests: `tests/pruef/nil_literal.dhtest`.
```basic
IF o = NIL THEN ...        ' -> runtime error: Variable 'nil' is not declared (missing DIM?)
IF o <> NIL THEN ...       ' ditto
PRINT IS_NIL(NIL)          ' ditto
```
`NIL` cannot be written out anywhere, even though older documentation and
examples use `<> NIL`. Consequences throughout the book:
- `net`: an empty `NET_TCP_ACCEPT` can only be checked with `IS_NIL(x)`, not `x <> NIL`.
- `db`: the documented “NIL → NULL” binding does not work (you have to leave the
  column out of the INSERT).
**Proposal:** introduce `NIL` as a real keyword literal (at least in
`=`/`<>` comparisons and as an argument). Makes `IS_NIL` half redundant and
the db NULL binding possible. The biggest “contract vs. reality” item.

---

## B — Misleading error messages (small, high impact)

### B1. “Stufe 3e: DIM-Typ 'vec2' noch nicht unterstuetzt” (“stage 3e: DIM type 'vec2' not supported yet”) when an IMPORT is missing  —  ✅ FIXED (commit f4c8b78)
> Now: `Unknown type 'vec2' -- is IMPORT "vec2" missing?` (with several modules, all
> candidates are named). No more “stage 3e” leak in the DIM type message.
> `preprocess::modules_for_type` + `compiler::unknown_dim_type_msg`. Tests: `tests/pruef/dim_type_error.dhtest`.
```basic
DIM v AS VEC2              ' without a preceding IMPORT "vec2"
' -> compile error: stage 3e: DIM type 'vec2' not supported yet
```
Two problems:
1. **“Stufe 3e”** (stage 3e) is the compiler's internal phase naming (front-end port) and
   does not belong in a user message.
2. **“noch nicht unterstuetzt”** (not supported yet) is factually wrong — the type exists, only the
   `IMPORT "vec2"` is missing (external types are resolved from the imports at
   compile time).
**Proposal:** e.g. `Unbekannter Typ 'vec2' — fehlt IMPORT "vec2"?` (“unknown type 'vec2' — missing IMPORT "vec2"?”). Applies to
all external module types (astar_grid, json_handle, …). In general, search all
user-facing messages for “Stufe Nx” leaks at some point.

---

## C — Deviations between documentation and contract

### C1. `NET_UDP_LAST_FROM` returns a STRING, not a TUPLE  *(documentation fixed in this session)*
dhrt returns `"host:port"` as a STRING (so does the golden test
`tests/pruef/modules_net.dhtest`); the old `docs/module-net.md` promised a
`TUPLE (host, port)`. **The documentation was brought in line with reality.**
**Optionally open:** for ergonomics, the engine could return a real
tuple instead (then `peer[0]`/`peer[1]` directly) — that would require undoing the test
and the documentation. Consistent right now, so not a must.

---

## D — Deliberate design decisions that still surprise

(Probably intentional; documented here only in case anyone wants to reconsider them.)

### D1. `/` returns INTEGER sometimes, FLOAT other times  —  ✅ CHANGED 2026-09-24: always FLOAT
> **New decision (2026-09-24):** `/` always returns a floating-point number, `8 / 2` is
> `4.0`. The occasion was compilation to machine code (`docs/entwurf-maschinencode.md`):
> a type that depends on the value cannot be fixed in advance. `\` divides as integers.
> Assigning to an INTEGER variable still works as long as the result is whole
> (`i = 8 / 2` is 4); an array index from `/` aborts, and `dhrt --check` warns
> about it. In the existing code, the full test run did not find a single place that depended on it.
> The rest of this section describes the state before.
> Author decision (2026-06-14): **`/` stays as it is** (whole → INTEGER, otherwise FLOAT;
> no `4.0` break, book unchanged). Instead, the error message when assigning
> a FLOAT `/` result to an INTEGER variable now points to `\` (integer division)
> or `INT()/ROUND()` — this addresses the actual pain point without
> breaking compatibility. Test: `tests/pruef/div_and_float_display.dhtest`.
```basic
PRINT 8 / 2     ' 4     (whole -> INTEGER)
PRINT 9 / 2     ' 4.5   (not whole -> FLOAT)
PRINT 6 / 3     ' 2     (INTEGER)
```
The **result type of `/` depends on the runtime value**. That makes static
type assumptions harder and is surprising. More consistent (and common in many languages): `/`
always FLOAT, `\` for integer division (already exists). A deliberate choice —
but a real pitfall.

### D2. Float output shows full f64 precision  —  ✅ PARTIAL FIX (commit 2492848)
> General float output is correct (shortest round-trip form) and stays. The only
> real wart — f32-backed **audio volumes** (`0.800000011920929`) — is fixed:
> `AUDIO_BUS_GET_VOLUME`/`AUDIO_GET_VOLUME`/`AUDIO_MUSIC_GET_VOLUME` round to 6 digits
> → `0.8`. Test: `tests/pruef/div_and_float_display.dhtest`. (Analysis below.)
> **Finding after checking:** dhrt's `fmt_float` already uses the *shortest round-trip*
> representation (Rust Display ≈ Python `repr`). `0.1+0.2 -> 0.30000000000000004` and
> `CURVE_SMOOTHERSTEP -> 0.16308000000000003` are the **shortest possible exact** form —
> anything shorter would be wrong (it would round to a different f64). So NO general
> formatting bug. **The only real wart:** f32-backed values that are widened to f64
> (e.g. `AUDIO_BUS_GET_VOLUME(0.8) -> 0.800000011920929`). If desired: fix only there,
> specifically (keep the bus volume as f64 or round when reading). General float printing
> stays as it is.

### D2-old (historical text):
```basic
PRINT 0.1 + 0.2                 ' 0.30000000000000004
PRINT CURVE_SMOOTHERSTEP(0.0,1.0,0.3)  ' 0.16308000000000003
```
Plus an **f32→f64 special case**: `AUDIO_BUS_GET_VOLUME` after `AUDIO_BUS_VOLUME(_, 0.8)`
returns `0.800000011920929` (the bus volume is held internally as f32 and printed
as f64).
**Proposal:** (a) in general, “shortest round-trip” float formatting for PRINT
(ryu/grisu) — `0.1+0.2` would still be `0.30000000000000004` (the shortest exact form), but
many values would get shorter. (b) Round f32 return values (audio volumes) before displaying them as f64,
then `0.8` instead of `0.800000011920929`.

### D3. `MID$` and `INSTR` are 0-based
Unusual for BASIC (classically 1-based). `INSTR` returns `-1` for “not
found” (not `0`). Consistent with the language's 0-index line, but a trap for
people coming from other BASICs. (Changing it would break everything — better just document it
clearly.)

---

## E — Smaller inconsistencies

### E1. Hardware modules can be imported but are dead in the standard build  —  ✅ FIXED
> dhrt now warns **already at the IMPORT** instead of only at the first function call.
> For hardware modules (`serial`/`usb`/`bt`/`wifi`) without the matching Cargo feature,
> the default build reacts in two ways: (a) `dhrt --check` delivers a
> `severity:"warning"` diagnostic on the IMPORT line → the editor marks it live
> while you type; (b) `dhrt run` prints the warning on stderr before the run. The
> later runtime error on the actual call remains (same wording,
> via `vm.rs::unknown_builtin_msg`) — so using it still fails,
> but the user finds out immediately instead of deep inside the program. Deliberately *not*
> shipped in the standard build (pulls in heavy dependencies: tokio/btleplug/hidapi/
> windows). `preprocess.rs`: `missing_hardware_modules` / `missing_hardware_imports_with_lines`
> / `hardware_missing_msg`; wired up in `main.rs` `compile_source` (run) +
> `check_source` (editor). Test: `tests/pruef/dhrt_check.dhtest` (case "ein hardware-import warnt schon beim import").

`IMPORT "wifi"` (likewise serial/usb/bt) is accepted by the preprocessor (they are
in `KNOWN_MODULES`), but **every** function call only throws at runtime
“… Hardware-Modul 'wifi', das in diesem dhrt-Build fehlt. Neu bauen mit:
python rust\build_runtime.py --hardware” (“… hardware module 'wifi', which is missing from this dhrt build. Rebuild with: …”). The IMPORT succeeds, the use does not.
(Historical text.) **Proposal:** either ship it in the standard build, or
warn already at the IMPORT (instead of at the first call).

### E2. Reserved words as variable names (beyond `step`)
`FOR … STEP` makes `step` a keyword. Just as reserved and surprising for graphics and
game programmers: **`band`** (= bitwise AND `BAND`), as well as the type and
operator words `image`, `sound`, `map`, `mod`. A `DIM band AS INTEGER`
fails. Small, but the **error message does not reveal the reason** (see
E3). **Proposal:** keep a list “do not use these names as
variables” in the tutorial/appendix.

### E3. Misleading message for a reserved DIM name — ✅ FIXED
> `DIM band` (both parsers) now reports: “**'BAND' is a reserved word and
> cannot be a variable name — choose another name**” (instead of
> “Erwartet Variablenname nach DIM”, expected a variable name after DIM). Applies to every keyword after `DIM`
> (`STEP`, `MOD`, `MAP`, `IMAGE`, …). For this, `keyword()` in the Rust lexer is
> `pub(crate)`, the Python parser uses `KEYWORDS`.

`DIM band AS INTEGER` used to throw “Erwartet Variablenname nach DIM” (expected a variable name after DIM) — but did not say
that `band` is a reserved word. (Historical text.)

---

## G — Findings while writing the “VORTEX” demo (examples/119, 2026-06-23)

### G1. `FLT()` was missing in dhrt + `--check` stayed silent — ✅ FIXED (built-in **and** systemically)
> **Part 1 (built-in):** `FLT` is now in the dhrt core (`builtins.rs`: `"flt" =>
> Value::Float(need_num(...))`, after `INT`) + in `builtin_index.json`. `FLT(7)/2
> -> 3.5` verified natively.
> **Part 2 (systemic — the actual “never again” fix):** the Rust compiler
> checks every built-in call against the authoritative `builtin_index.json` (embedded via
> `include_str!`, `compiler::is_known_builtin`). Unknown built-ins
> (typos OR tree-walker-only ones like FLT used to be) now produce a **non-fatal
> compile warning with a line number** → `dhrt --check` shows it in the editor (yellow
> squiggle), `dhrt run` on stderr. No blocking (warning, not an error)
> and no false positives: a sweep over ALL examples = 0 wrongly reported
> built-ins (the index is complete). Internal `__` built-ins are excluded.
> Consequence: a new dhrt built-in that you forget to add to `builtin_index.json`
> is noticed from now on. **Drift-protection test**
> `tests/pruef/dhrt_check.dhtest` checks that NO example uses a built-in
> that is not in the index — it immediately found **10 real but unindexed
> built-ins** (`CAMERA_ORBIT`, `WORLD_TO_SCREEN_X/Y`, `SCREEN_TO_WORLD_DIR_X/Y/Z`,
> `RAY_HIT_MODEL`, `PICK_MODEL`, `GETPIXEL`, `CIRCLEOUTLINE` — all from
> “added to the runtime, forgot the index” commits) and they were added
> (index: 1011 → 1021). **Note:** the index is embedded via `include_str!` at dhrt
> BUILD time → rebuild dhrt after changing the index. Further tests:
> `test_unknown_builtin_warns` / `test_known_builtin_no_warning`.

**Original report (2026, two paths):** `FLT(x)` — e.g.
`FLT(MOUSEX())`, used in several examples — ran in the Python
tree-walker of the time, but in the native runtime it threw **at runtime** “Builtin 'FLT'
im Rust-Kern noch nicht verfuegbar” (built-in 'FLT' not yet available in the Rust core). `dhrt --check` did not report it: the
compile was green, the error only came during the run. Workaround back then: `x * 1.0`.

Of the two problems, the first — built-in parity between two paths —
disappeared with the tree-walker. The second survived it and was the
more important one: a diagnostic that waves through calls to unknown built-ins is
dangerous even with only one runtime, because it moves the error from the compile to the
run. That is exactly what part 2 above fixes.

### G2. No vertical mirroring of text / render targets — ✅ FIXED (RT flip)
> `RENDERTARGET_DRAW(rt, x, y[, scale[, tint[, flip_v]]])` now has an optional
> 6th argument `flip_v`: `TRUE` draws the target vertically mirrored
> (`graphics.rs` `RtDraw` then uses the positive instead of the negative source height — the
> raylib RTs are y-mirrored anyway, so the mirror case is practically free).
> That makes real floor reflections possible: draw text once into a render target (cleared to transparent
> beforehand), then stamp it normally + mirrored with `..., TRUE`
> underneath. Used in `examples/119_vortex.dh` (scroller reflection).

(Historical text:) For a floor-mirror scroller you need a vertically
mirrored copy of the text. In dhrt there was **no way** to do that: `TEXT` cannot
flip/rotate; `RENDERTARGET_DRAW` clamps `scale` to `≥ 0` (no flip via
negative scaling); `DRAWIMAGEFLIPPED(img, x, y, fH, fV)` only works on
**images**, but render targets live in their own handle space (`graphics.rs`
`render_targets` vs. the image vec) — passing an RT handle as an image indexes
the wrong object. **Workaround in the demo:** a dimmed, slightly smaller,
*upright* copy below the text (looks like a wet floor), no real
mirroring. **Proposal:** a flip parameter for `RENDERTARGET_DRAW` (raylib
draws RT textures via a negative source height anyway — the mirrored case
is practically free) OR a `RENDERTARGET_DRAW_FLIPPED`. Then real
reflections/mirror effects become possible.

---

## H — Findings from the editor pilots (2026-08-31 / 2026-09-04)

### H1. `DIM red` failed inside a block, not at the top — ✅ FIXED
> **Predefined constants could only be shadowed at the top level.**
> `DIM pi AS INTEGER : pi = 5` ran perfectly at the top of the program and threw, four
> lines further down inside an `IF`/`WHILE`/`FOR`, the runtime error
> “CONST 'pi' cannot be overwritten”.
>
> Every predefined name was affected: the **18 colour names** (`red`, `green`,
> `blue`, `white`, `black`, `gray`, `orange`, `pink`, `brown`, `purple`,
> `cyan`, `magenta`, `yellow`, …), **all `KEY_*`**, `pi` and `tau` — exactly
> the words that a BASIC for graphics and games suggests as variable names.
> Inside a `SUB` it did not happen (there are local slots there).
>
> Three things made it unpleasant: the same line ran at the top and failed in the
> block; **`dhrt --check` stayed silent**; and the message pointed at the constant
> instead of the variable name — you look for the error where there is none.
>
> **Cause:** `collect_globals` (compiler.rs) only walked the *top-level*
> statement list. A `DIM` inside a block therefore got no global slot and
> was created via its NAME — and `DECLARE_NAME` leaves an existing
> entry in place, whereas `DECLARE_GLOBAL_SLOT` replaces it.
> Drachenhauch has no block scope; a `DIM` inside a block *is* global;
> the walk now descends into the blocks (not into SUB/FUNCTION/CLASS
> — those have their own slots).
>
> **Side benefit:** collision detection (E3) used to see only
> siblings. `CONST Modus` at the top and `DIM modus` inside an `IF` slipped
> through — not any more.
>
> Found in the sprite editor (`DIM pi` in the main loop). Tests:
> `tests/pruef/name_collision.dhtest`.

### H2. `dhrt --check` did not report a single typo in a variable name — ✅ FIXED
> **`DIM zaehler` at the top, `zaehlr = zaehlr + 1` in a SUB** — `--check` returned
> `[]`, the program started and printed, and only the call of the SUB brought
> “Variable 'zaehlr' is not declared (missing DIM?)”. The same when reading an
> unknown name, and at the top level too. Because `DIM` is mandatory everywhere in Drachenhauch,
> a misspelled name is **always** an error — just
> one that a rarely taken branch can hide for as long as it likes.
>
> Found in the sprite pilot (`examples/189`, 2700 lines): there,
> `geaendert = TRUE` stood in a new SUB — a variable that does not even exist in THIS
> program (it comes from a sibling pilot).
> Only a test that pressed the button brought it to light.
>
> **Cause:** `load_var`/`store_var` (compiler.rs) have, as their last branch, a
> fallback to `LOAD_NAME`/`STORE_NAME`; every name the compiler
> could not resolve ends up there, and it is only looked up at runtime. **Three**
> paths end there, not two — `INPUT x` (emits `INPUT_NAME`) and the
> `READ` target (which needs the temporary storage for array/index targets and
> therefore does not go through `store_var`) initially stayed silent.
>
> Why this was more work than it sounds, and where the check deliberately stays coarse,
> is in `docs/sprache.md` and in `CLAUDE.md`. The actual proof
> is the run over **all 384 `.dh` files in the repository** (0 reports) — and
> that is exactly what found the two false alarms of the first versions: `DIM x[N] AS T` and
> `CONST` inside a SUB. That is why it is now a test
> (`tests/pruef/dhrt_check.dhtest`) instead of a manual run.
>
> Tests: `tests/pruef/check_unbekannte_namen.dhtest`, `tests/pruef/dhrt_check.dhtest`.

---

### H3. `deg = DEG(winkel)` aborted at runtime — ✅ FIXED
> **A variable named like a command hid the command — even when
> it COULD not hide it.** `DIM deg AS FLOAT : deg = DEG(w)`
> ran, until 2026-09-04, into the message “'deg' is a variable of type FLOAT
> and cannot be called like a function”, and `dhrt --check`
> said nothing about it. Found in example 145, which therefore wrote `eg` instead of `deg`.
> In BASIC, `len = LEN(s)` is everyday code.
>
> **Cause:** for every variable name in front of a parenthesis, the compiler took
> `CALL_VALUE` (the FUNCREF path) without asking for the type.
>
> **Rule now:** if the declared type of the variable is known and not
> FUNCREF, and there is a built-in of that name, `NAME(...)` means the
> built-in — a FLOAT cannot be called. Only a FUNCREF variable
> of that name still calls the variable (it might be meant there);
> the message now says so. The FOR EACH loop variable does not
> go through this path at all and does not hide anything. Tests
> `tests/pruef/variable_wie_builtin.dhtest`.

## I — Switching from other BASICs and from Python (2026-09-21) — ✅ FIXED

> Measured, not estimated: around 150 small programs written the way you
> bring them along from QBasic, FreeBASIC, VB, Blitz or Python, each run against
> `dhrt run`. About thirty of them failed with a message that said
> nothing — usually “Expected end of line” (`GOTO oben`, `LOCATE 1, 1`,
> `EXIT FOR`, `SWAP a, b`, `DIM a AS INTEGER = 5`) — or ran silently wrong.
> Overview for users: [umstieg.md](umstieg.md); tests
> `tests/pruef/umstieg.dhtest` (68 cases).
>
> **Silently wrong, now fixed:**
> - **A name on its own as a statement did nothing.** `meineSub` (without parentheses)
>   loaded the SUB as a FUNCREF and threw it away — no call, no message.
>   Now a name that is not a variable but a SUB/FUNCTION or a
>   command is called (`CLS`, `FLIP`, `meineSub`).
> - **`AND`/`OR` on two integers** are logical and return one of the
>   values (`6 AND 3` = 3, `6 OR 1` = 6); in other BASICs they are bitwise. That
>   stays, but `--check` now warns when both sides are statically INTEGER
>   and names `BAND`/`BOR`.
> - **`DIM a AS ARRAY OF T` without a size was NIL** — the first `ARRAY_PUSH`
>   reported “expected ARRAY”. Now an empty array (global, local, class field).
> - **`VAL` only read texts that are ENTIRELY a number**: `VAL("3 Äpfel")`,
>   `VAL("1e3")` and `VAL("&HFF")` were 0 (the PDF readers therefore had to
>   take “3 0 R” apart themselves). Now the number at the start, with `&H`/`&O`/`&B`,
>   `0x`/`0b` and an exponent.
> - **The message “'=' as a statement” pointed at the NEXT line** (it
>   was produced after the end of the line).
>
> **Works now:** `DIM x AS T = wert` (becomes DIM + assignment, same
> type check; not for class fields), `EXIT FOR/DO/WHILE/REPEAT/SUB` (only
> if it matches the INNERMOST loop, otherwise an error; `EXIT FUNCTION` names
> `RETURN`), `END` on its own (= `EXIT(0)`, also inside blocks — `block_until` does not stop
> at an `END` followed by the end of the line), `END WHILE`, `SWAP a, b`, `LET`,
> `?` as `PRINT`, `INPUT "Frage"; x`, reading/writing `m["k"]`, `LEN(map)`,
> joining arrays with `+`, `RANDOMIZE(TIMER())`. `swap`, `let`, `exit` and
> `do` stay ordinary names (contextual, not keywords).
>
> **Messages that say what it is called here:** GOTO/GOSUB, line numbers,
> REDIM, TYPE, DEF FN, ON ERROR, LINE INPUT, PRINT USING, OPEN … AS #1,
> ELSE IF/ENDIF, MID$ as an assignment, DIM SHARED, `DIM a(10)`, DIM without AS,
> `STRING * 10`, type suffix `x%`, `%` as remainder, `!=`, `==`, `#`/`//` as
> comment, `&` as concatenation, XOR, `f = wert` in FUNCTION f, nested
> SUB (previously “Stufe 3e: … noch nicht unterstuetzt”, stage 3e: … not supported yet), foreign type names
> (DOUBLE, LONG, BOOL …), array of arrays, `RND`/`TIMER` without parentheses, a
> name followed by a value (`LOCATE 1, 1` → “in parentheses”), `INSTR(start, …)`,
> foreign command names (UCASE$, UBOUND, FIX, CINT …). An unknown command
> is now called “Unknown command” at run time instead of “im Rust-Kern noch nicht
> verfuegbar” (not yet available in the Rust core) — that phrase stays reserved for builds without graphics/sound;
> the test collections recognise such a build by it. The tables live in
> `umstieg.rs` (pure, with Rust tests), the statement hints in
> `Parser::umsteiger_hinweis` — it is asked for the statement itself and for
> every position after THEN/ELSE/`:` on the same line.
>
> **Deliberately NOT changed** (they break existing code): `MID$`/`INSTR`
> from 0, `AND`/`OR` logical, commands only with parentheses.
>
> **Second round (2026-09-22), 90 more probes (classes, texts, arrays,
> files, errors, loops):**
> - **`FOR i = 1 TO 3 STEP 0` made dhrt CRASH** (Rust panic, index
>   out of bounds): with a fixed step the direction is known at
>   compile time, only with 0 it was neither forward nor backward, and
>   the branch for the runtime direction read a step slot that did not
>   exist (-1). Now a compile error. Likewise `STEP 0.5` with a loop variable
>   that is certainly an integer (previously: abort in the first step with a
>   message about division).
> - **`READLINE` returns `""` at the end of the file** -- exactly as for an empty
>   line, and there was no way to tell the two apart. New: `EOF(f)`.
> - **`"5" = 5` and `TRUE = 1` are always FALSE** (nothing is converted).
>   That stays, but `--check` warns when both sides are statically known and
>   of a different kind.
> - Works now: `READALL$(pfad)`, `CALL`, `BYVAL`.
> - Messages: FUNCTION without parentheses in a calculation (previously "erwartet
>   Zahlen, erhalten FUNCREF", expected numbers, got FUNCREF), `Me`/`this`, `SUB New`, `OPTIONAL`,
>   `CATCH e AS`, `s[0] = "x"`, `TAB`/`SPC`/`ARRAY_REMOVE`.
> - Measured: none of the new warnings hits any of the 447 `.dh` files in the
>   repository.
>
> **Third round (2026-09-22), 60 probes (graphics, game loop, classes,
> modules):**
> - **`spieler.springen` without parentheses silently did nothing** -- the same trap as
>   a bare name for a SUB, just with methods (a bound
>   method was created that nobody called). Now a call if the type is statically known
>   and the name is a method (not a field, not a PROPERTY).
> - `LOADIMAGE` of a missing file reported raylib's "image data is null,
>   either the file doesnt exist or the image type is unsupported" -- now
>   file name and folder. A NIL as an image/handle is no longer "erwartet
>   Zahl, erhalten NIL" (expected a number, got NIL), but names the cause; the graphics message "erwartet
>   Zahl" (expected a number) names the argument number.
> - Hints for `SCREEN 12`, `LINE (x,y)-(x,y)`, `CIRCLE (x,y),r`, `PSET`,
>   `COLOR`, `KEYDOWN`, `IMPORT json` without quotes, `INHERITS`,
>   `PUBLIC`/`PRIVATE`/field without DIM in a class, an unterminated string,
>   an expression that breaks off at the end of the line.
>
> **Fourth round (2026-09-22), 60 probes (modules json/db/regex, mathematics,
> time, gui, sound, calls, visibility):**
> - **`JSON_PARSE("{\"a\": 1}")` reported "Erwartet Rparen"** (expected Rparen). In a
>   normal string the backslash stays literal, so the string
>   ends at the `\"` -- and the follow-up errors said nothing about that. Now a
>   sentence that points to `!"..."` (it looks for a string that ends in a
>   backslash).
> - **62 commands had only "N..M Argumente" (N..M arguments) as their signature** -- `BOX`, `CIRCLE`,
>   `LINE`, `PLOT`, `TEXT`, `MID$`, `INSTR`, the gui and ui widgets. That is
>   what the IDE showed on hover, in completion and now in the
>   error messages too. 61 of them carry their parameter names (52 taken from the
>   `docs/` tables when the argument count matched EXACTLY, nine
>   by hand); only `GUI_RULE` remains, where the count depends on the kind of rule.
>   **A real bug came to light along the way:** `ATLAS_DRAW_FLIPPED` was listed with 4..6
>   arguments in the index and in the book, but takes 7 (`tint`) -- a correct
>   call would have got a warning. Index and book chapter corrected.
>   A guard keeps it that way (`doku_pruefungen.dhtest`).
> - **Error messages about arguments now name the form of the command**
>   (`vm::mit_signatur`, ONE place in main.rs): "JSON_GET_INT: erwartet 2
>   Argument(e), erhalten 1 -- Aufruf: JSON_GET_INT(h: ?, path: STRING)" (expected 2 argument(s), got 1 -- call: …).
> - **`x = meineSub()`** only aborted at runtime, and at the ASSIGNMENT
>   ("Erwartet INTEGER, erhalten NIL", expected INTEGER, got NIL). Now a compile error at the call.
> - Hints: `TRUNC`, `ZEIT_JAHR` and its siblings, `GUI_WINDOW_NEW`.
>
> **Fifth round (2026-09-23), 100 probes (the road to the first game: sprite,
> tilemap, physics, camera, particles, save games, ECS, files, classes):**
> - **Drawing without `SCREEN` SILENTLY did nothing.** A graphics command creates
>   a HIDDEN window when needed (only for the GL context, so that
>   `LOADIMAGE`/`IMAGE_*` also work in console programs) -- whoever drew to
>   the screen in it got no picture and no word: `BOX(...)`, `CLS()`,
>   even `FLIP()` ran through with return code 0. Now it is an error that names
>   `SCREEN(breite, hoehe, "Titel")` (`vm::ist_schirmbefehl`, 26 names
>   kept by hand; `Graphics::sichtbar` is the distinguishing feature,
>   an active render target is exempt -- there you draw off-screen
>   on purpose).
> - **Drawn, but never `FLIP()`** is the same trap one step further:
>   dhrt collects the commands and only plays them back at FLIP, so the window
>   stays empty. At the end of the program, a sentence on stderr
>   (`Graphics::nie_gezeigt`: visible window, 0 flips, but recorded
>   commands). Two examples in the repository call SCREEN without FLIP -- they do not
>   draw, so the sentence does not hit them.
> - **An unknown command now suggests the real name**
>   (`aehnlich.rs`, new and pure): `CIRLCE` -> `CIRCLE`, `MAKEDIR` -> `MKDIR`,
>   `SPRITE_SET_ANIM` -> `SPRITE_ADD_ANIM`. For this, two rules instead of one --
>   the edit distance catches transpositions, the WORD PARTS catch the missing
>   middle (`PARTICLE_NEW` -> `PARTICLE_SYSTEM_NEW`, distance 7, which every
>   threshold would ignore). The part match ranks AHEAD of a distance match of
>   3, otherwise `PARTICLE_DRAW` would come first. The suggestions for VARIABLES
>   (`compiler::naechster_name`) and for members have since gone through
>   the same file -- before, the compiler had its own calculation.
> - **A file name where a loaded image/sound belongs**
>   (`PLAYSOUND("sprung.wav")`, `DRAWIMAGE("held.png", ...)`,
>   `SPRITE_NEW("held.png", 16, 16)`) only said "erwartet Zahl, erhalten
>   STRING" (expected a number, got STRING). Now the message names the loader, chosen by the EXTENSION
>   (`umstieg::text_statt_zahl`). The same path covers `KEYHIT("a")` (a key
>   is a number: `ASC("a")` or `KEY_A`) and `RGB("FF0000")`
>   (`COLOR_FROM_HEX`) -- both used to end silently in a type message.
> - **The loaders now report their missing file straight away.** `LOADIMAGE` named
>   the file and the search location, `LOADSOUND` returned `"...: os error 2"` and
>   `PLAYMUSIC` even `IoError(Os { code: 2, ... })` -- and that under the
>   name `AUDIO_MUSIC_LOAD`. One source: `builtins::datei_da`.
> - **`DIM p AS Spieler` does not create an object** -- the variable is NIL until a
>   `NEW` comes (with a STRUCT it is created with the declaration, and in
>   other BASICs as well). "Zuweisung an '.hp' bei NIL-Referenz" (assignment to '.hp' on a NIL reference) does not say
>   that; the sentence now names `p = NEW Klasse()`.
> - **`DIM f AS FILE` without `OPENFILE`** reported "WRITE erwartet FILE" (WRITE expects FILE) -- the
>   variable IS declared as FILE, so the message was misleading,
>   and it named WRITE where `WRITELINE` was written.
> - **An index exactly ONE too far** gets the sentence "arrays count from 0:
>   DIM a[3] has a[0] to a[2]". Only then -- for index 99 in an array
>   with three slots it does not help.
> - Documentation: the section "The first program with graphics" in `docs/umstieg.md`
>   (SCREEN, FLIP, loop, loading) and three more lines in the table
>   (`MOUSEBUTTON(0)` is left, not 1). New among the silent
>   differences: `=` on arrays and MAPs asks for the SAME container,
>   not for equal contents (measured: two arrays with the same numbers
>   are `<>`).

## F — Documentation gaps & behaviour traps (review 2026-06-23, all verified)

### F1. `physics3d` was completely undocumented + dead link — ✅ FIXED
> `docs/module-physics2d.md` linked to `module-physics3d.md`, which **did not
> exist** — although PHYS3D is fully implemented (`builtins.rs`, 44 arms:
> `PHYS3D_NEW/ADD_BOX/ADD_SPHERE/STEP/BODY_*/SET_*/…`) and used in `examples/107`.
> **`docs/module-physics3d.md` newly written** (signatures,
> quaternion render idiom via `QUAT_NEW`/`MAT4_TRS`) → link now valid.

### F2. `SPRITE_HIT_BOX` / `SPRITE_HIT_POINT` undocumented — ✅ FIXED
> They exist in `builtins.rs` but were not in any documentation. Added to `module-sprite.md`:
> `SPRITE_HIT_BOX(sp, x, y, w, h)` (AABB against a rectangle),
> `SPRITE_HIT_POINT(sp, x, y)` (point in sprite, click test).

### F3. More reserved words as obvious variable names — ✅ DOCUMENTED
> Verified as reserved and surprising: **`map`, `image`, `sound`, `input`,
> `file`, `data`, `read`, `in`** (besides `band`, `step`, `mod`, …). Harmless:
> `value`, `key`, `count`, `index`, `name`, `type`, `result`, `size`, `pos`,
> `state`, `item`, `text`, `color`. Noted in `sprache.md` (variables).

### F4. Array access is strict, slicing clamps silently — ✅ DOCUMENTED
> `arr[5]` out of range → runtime error `Index 5 ausserhalb [0..2]` (index 5 outside [0..2]). `arr[0:99]`
> on a 3-element array → silently clamped to length 3, no error. The asymmetry is now
> explained in `sprache.md` (arrays). Slicing only for 1D.

### F5. Arrays always by reference — ✅ DOCUMENTED
> Arrays passed to a `SUB`/`FUNCTION` share their memory; changes affect
> the original (`ARRAY_COPY` for a copy). Noted in `sprache.md` (arrays).

### F6. `+` with a string converts silently (number/bool → string) — ✅ DOCUMENTED
> `"x=" + 42` → `"x=42"`, `"f=" + TRUE` → `"f=TRUE"`, no type error. Noted in
> `sprache.md` (strings).

### F7. `TILE_SWEEP_X/Y` return a FLOAT in the tuple — ✅ DOCUMENTED
> `TILE_SWEEP_X(...)` → `(new_x, hit)`, where `new_x` is a **FLOAT**. Anyone
> assigning directly to an INTEGER coordinate needs `INT()/ROUND()`. The return value
> is now shown in `module-tile-collide.md` as `(new_x: FLOAT, hit: BOOL)`.

---

## Already fixed (this/last sessions — no longer open)

- `ATLAS_DRAW_FLIPPED`: flip_x/flip_y now take TRUE/FALSE **or** 1/0; real
  flip_y; tint = arg 7 (was inconsistent/broken before). *(commit 8aa315f)*
- `PHYS3D_ADD_BOX`/`ADD_SPHERE`: the dynamic flag accepts TRUE/FALSE **or** 1/0
  (previously only a number — consistent with physics2d). *(commit 3928fdd)*
- Filled `TRIANGLE`/`POLYGON`: now fill regardless of winding order (previously only visible with
  CCW — raylib back-face culling). *(commit d68efd7)*
- `docs/module-net.md`: return type of `NET_UDP_LAST_FROM` brought in line with
  reality (STRING instead of TUPLE). *(see C1)*
- **A2** (`NIL` literal) + **B1** (misleading DIM type error) — fixed 2026-06-14,
  commit f4c8b78 (details in the respective sections above). Book passages about “NIL
  is not a literal” (chapters 34/69/73 + appendix D) corrected accordingly.
- **`MOUSEBUTTON` documentation wrong**: `builtins-grafik.md` said “1=middle, 2=right”,
  but the raylib runtime maps `0=left, 1=right, 2=middle` (`graphics.rs`
  `mouse_button`). A silent mistake in every mouse game. **Documentation corrected.**
- **`CIRCLEOUTLINE` added**: `CIRCLE` was only filled (an inconsistency —
  `TRIANGLE`/`POLYGON`/`ELLIPSE` have outline variants). `CIRCLEOUTLINE(x,y,r
  [,color])` new in the runtime (uses the ellipse outline, no command of its own).
- **C-style hex/binary literals** `0xFF` / `0b1010` in addition to `&H`/`&B`
  (both lexers), see A1. Wrong `0x` documentation examples made runnable.
- **Undocumented graphics built-ins documented**: `LINEW` (thick line),
  `BOXROUND`/`RECTROUND` (rounded corners), `GRADIENTH`/`GRADIENTV` (gradients),
  `SPLINE`, `BLEND_MODE`, `RGBA`/`ALPHA` (transparency) in `builtins-grafik.md`;
  `MOUSE_GROUND_X/Z/HIT` (cursor → ground plane, the “where does the mouse point in
  3D?” helper) in `rust-runtime.md`. They existed in the runtime but could not be found
  by users.

- **`CAMERA_ORBIT` added**: `CAMERA_ORBIT(tx,ty,tz, radius, yaw, pitch[, fovy])`
  new in the runtime — replaces the manual orbit spherical-coordinate trigonometry
  (pitch clamped against gimbal flip, fovy optional). Maths verified
  exactly via `CAMERA3D_X/Y/Z`.
- **3D projection + mesh picking + GETPIXEL added** (rest of the 3D backlog):
  `WORLD_TO_SCREEN_X/Y` (3D→2D) and `SCREEN_TO_WORLD_DIR_X/Y/Z` (ray direction,
  origin = camera) for general projection; `RAY_HIT_MODEL` + `PICK_MODEL`
  (raycast/picking against loaded meshes, not just box/sphere); `GETPIXEL`
  (read a pixel — `PIXEL` was only a `PLOT` alias). All four verified exactly
  (cube raycast → distance 4.0; projection of the origin → centre of the image; gradient pixel).

> With that, the graphics ergonomics backlog that was found (2D as well as 3D) is done.
