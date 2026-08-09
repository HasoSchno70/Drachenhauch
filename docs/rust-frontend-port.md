# Front-End-Portierung nach Rust (Lexer → Parser → Compiler)

**Ziel (erreicht):** `dhrt` erzeugt jetzt selbst aus Quelltext Bytecode und
führt ihn aus — `dhrt run datei.gb` ist ein eigenständiger End-to-End-Lauf
**ohne Python**. Python bleibt nur noch in den Editoren/Tools (+ als
Referenz-Tree-Walker). Damit kann auch der [Web-Playground](web-playground.md)
ein reines Rust-WASM werden (kein Pyodide nötig).

Die Front-End-Toolchain (`lexer`/`tokens`/`parser`/`ast_nodes`/`compiler`/
`preprocess`, ~5.100 Zeilen Python) wurde inkrementell nach Rust portiert —
**jede Stufe gegen Python verifiziert** (cargo + rustc verfügbar). **Alle 5
Stufen fertig.** dhrt führt `.gbc` weiterhin direkt aus (VM-Pfad); `dhrt run`
(bzw. `dhrt datei.gb`) deckt jetzt den vollen Pfad Quelltext → Bytecode → Run ab.

## Stufen

| Stufe | Rust | Verifikation | Status |
|---|---|---|---|
| 1. **Lexer** (`tokens`+`lexer`) | `src/lexer.rs` | Token-Strom `[TYP,wert,zeile]` via `dhrt --tokens` == Python (alle Beispiele + Snippets) | ✅ **fertig** |
| 2. **Parser** (`ast_nodes`+`parser`) | `src/ast.rs` + `src/parser.rs` | AST als kanonisches JSON via `dhrt --ast` == Python (struktureller Vergleich) | ✅ **fertig** |
| 3. **Compiler** (`compiler`) | `src/compiler.rs` | **Output-Parität**: `dhrt --runsrc` (Rust lex+parse+compile+run) == Python-Tree-Walker | 🟡 **3a–3e fertig** |
| 4. **Preprocess** (`IMPORT`) | `src/preprocess.rs` | Merge-Ergebnis-Gleichheit (`dhrt --preprocess` == `process()`) | ✅ **fertig** |
| 5. **Verdrahtung** | `dhrt run datei.gb` (+ `.gb`-Auto-Detect) | Output-Parität (dhrt-self-compiled vs Python-TW) | ✅ **fertig** |

## Stufe 1 — Lexer (fertig)

- [`rust/gb_runtime/src/lexer.rs`](../rust/gb_runtime/src/lexer.rs): `Tt`
  (Token-Typen mit exakt den Python-`TokenType.name`-Strings), `Lexer`,
  f-String-Expansion (inkl. Format-Spec `{x:.2f}` → `FORMAT$`), `&H`/`&B`,
  Zeilenfortsetzung (`_`+Newline, implizit in Klammern), lowercase-Idents,
  `$`-Suffix.
- Debug-Einstieg `dhrt --tokens <datei.gb>` → eine JSON-Zeile `[TYP, wert,
  zeile]` pro Token (kanonisch, umgeht Python-`repr`-Eigenheiten).
- Test [`tests/test_rust_lexer_parity.py`](../tests/test_rust_lexer_parity.py):
  parst beide Seiten als JSON und vergleicht strukturell — **137 Fälle grün**
  (alle `examples/*.gb` + 17 Snippets).
- Grenzen: Integer-Literale als `i64` (Python: bignum) — für reale Programme
  ausreichend; Spalten (`col`) werden in Stufe 1 nicht verglichen (nur Zeile),
  da f-String-Synthetik-Tokens die Spalte teilen. Kommt mit dem Parser, falls
  nötig.

## Stufe 2 — Parser (fertig)

- [`rust/gb_runtime/src/ast.rs`](../rust/gb_runtime/src/ast.rs): `Node`-Enum
  (alle ~50 Knoten aus `ast_nodes.py`) + `Param`/`CaseMatch`, `to_json()`
  emittiert exakt die Dataclass-Feld-Struktur (`{"_": NodeName, ...}`).
- [`rust/gb_runtime/src/parser.rs`](../rust/gb_runtime/src/parser.rs):
  Recursive-Descent-Port (gleiche Präzedenz + Disambiguierungen: Tupel-Assign-
  Lookahead, `FOR EACH`, `IIF`, Slice-vs-Index, WITH-`.member`, List/Dict/Set-
  Comprehensions, Operator-Overloading, Properties). Debug-Einstieg
  `dhrt --ast <datei.gb>`.
- Test [`tests/test_rust_parser_parity.py`](../tests/test_rust_parser_parity.py):
  **96 grün** (alle parsbaren `examples/*.gb` + 41 Snippets).
- Gotchas: (a) `.line` ist in Python KEIN Dataclass-Feld → fällt bei der
  Serialisierung raus → reiner Struktur-Vergleich (Rust trackt `line` in Stufe 2
  nicht). (b) `Param.by_ref` hält in Python das BYREF-**Token** (oder None) statt
  eines bool — der Test normalisiert auf `bool` (Rust nutzt korrekt bool).
  (c) `CASE IS <op>`: erstes `values`-Element ist ein roher Operator-String, kein
  `StringLit` → eigener `CaseVal::Op`-Zweig.

## Stufe 3 — Compiler (in Arbeit)

[`rust/gb_runtime/src/compiler.rs`](../rust/gb_runtime/src/compiler.rs): AST →
`.gbc`-JSON. Debug-/Run-Einstieg `dhrt --runsrc <datei.gb>` macht **alles in
Rust** (lex+parse+compile+run).

**Gate-Entscheidung: Output-Parität statt byte-exaktem Bytecode.** dhrt's VM
implementiert den vollen Opcode-Satz — sowohl generische (`ADD`, `LOAD_NAME`)
als auch optimierte (`ADD_NN`, Slot-Globals, Inline-Caches). Der Rust-Compiler
emittiert die **generischen** Opcodes (kein Constant-Folding, keine `_NN`, keine
Inline-Caches) — das Verhalten ist identisch, der Code viel kleiner. Verifiziert
wird per stdout-Vergleich `dhrt --runsrc` == Python-Tree-Walker
([`tests/test_rust_compiler_parity.py`](../tests/test_rust_compiler_parity.py)),
dasselbe Korrektheits-Prinzip wie `test_dhrt_parity`. (Performance-Parität —
Optimierungs-Opcodes — kann später nachgezogen werden, ohne Verhalten zu ändern.)

**Stufe 3a (fertig):** main-only — Skalar-Globals (Slot-basiert), CONST,
Arithmetik/Vergleich/Logik (`and`/`or`-Short-Circuit)/Bitwise/Unär, PRINT,
Builtin-Calls, IF/ELSEIF/ELSE, WHILE, BREAK/CONTINUE.

**Stufe 3b (fertig):** `FOR ... TO ... STEP` (konstante + Laufzeit-Richtung,
Temp-Local-Slots in main), Arrays (`DIM x[n,m]` → `DECLARE_ARRAY_NAME`; `ARRAY
OF`/`MAP OF` → `DECLARE_NAME`), Index-Zugriff/-Zuweisung (`LOAD/STORE_INDEX`),
`INPUT`, `DATA`/`READ`/`RESTORE` (rekursive Werte-Sammlung). **31 Tests grün.**
Local-Slots (`LOAD/STORE/DECLARE_LOCAL`) jetzt unterstützt. Nicht unterstützte
Konstrukte liefern `Err("Stufe 3c/3d: ...")` → der Sweep überspringt sie.
*Bekannte dhrt-Grenze (nicht 3b-spezifisch):* sizeless `DIM x AS ARRAY OF T`
wird von dhrt nicht leer initialisiert (auch bei Python-kompiliertem `.gbc`).

**Stufe 3c (fertig):** User-`SUB`/`FUNCTION` — Stub-Phase (Forward-Refs +
Rekursion), Body-Kompilierung in eigenem Ctx (Params als Locals), `RETURN`/
`RETURN_VOID`, `CALL_USER` mit Named-Arg-/Default-Auflösung + Variadic, FUNCREF
(`LOAD_FUNCREF` für bare Funktionsnamen, `CALL_VALUE`). **40 Tests grün** (fib-
Rekursion, Defaults, Named-Args, Variadic, Higher-Order via FUNCREF). Container-
Methoden-Aufrufe (`rest.length()`) → noch `Err("Stufe 3d: ...")`.

**Stufe 3d (fertig):** Klassen/Structs — Klassen-Registry (Felder/Methoden-Sigs/
Properties/parent_name), Methoden in eigenem Ctx (`current_class`, `Self`→
`LOAD_SELF`, Felder→`LOAD/STORE_FIELD`, implizite Methoden-Calls), `NEW`
(`NEW_INSTANCE`, Named-Args via Init-Sig), Member-Zugriff/-Assign (`LOAD/STORE_
MEMBER`), Methoden-/Container-Calls (`CALL_METHOD`), Vererbung (parent_name →
MRO zur Laufzeit), Properties (GET/SET als `__get_`/`__set_`-Methoden + property-
set), Operatoren (`__op_*`), STRUCT (Auto-Init `DECLARE_STRUCT_NAME`), STATIC
CONST + ENUM (als `{"ns":...}`-Namespace im const-Pool). **50 Tests grün.**

**Stufe 3e (fertig):** `SELECT CASE` (value/range/`IS`/Guard-`WHERE`/`ELSE`),
`FOR EACH` (String/Tupel/Array/Map-Keys, Desugar zu Index-Loop über
`__comp_iter`), `REPEAT…UNTIL`, Tupel-Literal (`BUILD_TUPLE`) +
Destructuring (`UNPACK_TUPLE`, Identifier/Member/Index-Ziele), `WITH`
(anonymer Slot + `.member`-Desugar), `TRY`/`CATCH`/`THROW`
(`TRY_BEGIN`/`TRY_END`/`THROW` + `try_depth`-Tracking, damit BREAK/CONTINUE
über Try-Blöcke korrekt `TRY_END` emittieren), `SliceAccess` (`SLICE`),
List-/Set-/Dict-Comprehensions (Marker + `BUILD_TUPLE_DYN`, dann
`__set_dedup`/`__dict_from_pairs`), `IIF` (`TernaryExpr`, lazy), Coroutinen
(`YIELD`→`YIELD_VALUE`, `is_coroutine`-Flag aus `body_has_yield`). Dazu die
Werttypen aus `_TYPE_DEFAULTS` erkannt (`TUPLE`/`COROUTINE`/`FUNCREF`/`IMAGE`/
`SOUND`/`FILE`/`SPRITE_ATLAS`) als skalare DIM-Typen. **71 Tests grün.** Gotcha:
`compiler._collect_data` rekursiert in `SELECT`/`TRY`, aber NICHT in
`FOR EACH`/`WITH` — der Rust-Port spiegelt das exakt (sonst weichen die
DATA-Arrays ab).

## Stufe 4 — Preprocess / `IMPORT` (fertig)

[`rust/gb_runtime/src/preprocess.rs`](../rust/gb_runtime/src/preprocess.rs):
Port von `gamebasic/preprocess.py`. `process(source, base)` expandiert
`IMPORT`-Zeilen rekursiv **vor dem Lexen** und liefert `(merged_source,
imported_modules)`:
- **Quellcode-IMPORT** (`IMPORT "helper.gb"` / relativer Pfad): Datei lesen,
  rekursiv preprocessen, mit den exakten Markern `' === IMPORT … ===` /
  `' === END IMPORT … ===` inlinen. `seen`-Set (kanonisierte Pfade) verhindert
  Doppel-Inkludierung → `' [IMPORT bereits inkludiert: …]`.
- **Built-in-Modul** (`IMPORT "json"` / `… AS j`): Zeile wird zu
  `' === IMPORT MODULE json[ AS j] ===`. dhrt hat die Modul-Builtins nativ —
  kein echtes Inlining nötig. Der Modul-Name wird gesammelt.
- **Externe Modul-Typen:** `preprocess::external_types(mods)` mappt importierte
  Module auf die Typen, die sie registrieren (`vec2`→`vec2`, `json`→`json_handle`,
  …, Tabelle `MODULE_TYPES`). `compile_to_gbc(ast, external_types)` akzeptiert
  diese dann als skalare `DIM`-Typen (Default NIL) — so kompiliert `DIM v AS VEC2`
  nach `IMPORT "vec2"`.

`dhrt --runsrc` schaltet den Preprocessor jetzt vor; `dhrt --preprocess <datei>`
gibt die gemergte Quelle aus. **Gate = Merge-Ergebnis-Gleichheit** gegen
`process()` ([`tests/test_rust_preprocess_parity.py`](../tests/test_rust_preprocess_parity.py),
7 Tests: Quellcode-/Modul-/Alias-/nested+duplicate-/Trailing-Comment-IMPORT,
fehlender Import = Fehler in beiden, plus End-to-End `--runsrc`-Output-Parität).
Gotchas: (a) `MODULES`/`MODULE_TYPES` müssen mit `modules.discover_modules()`/
`register_type` synchron bleiben (29 Module hardcoded — dhrt hat sie nativ);
(b) CRLF-Dateien: jede Zeile wird vor dem IMPORT-Regex `\r`-getrimmt (Python liest
Textmodus, `\r\n`→`\n`); (c) Modul-Erkennung ist case-insensitiv (`load_module`
importiert `name.lower()`). **Grenze:** aliasierte Modul-Builtins (`J_PARSE` aus
`IMPORT "json" AS j`) sind eine Python-Registry-Laufzeit-Trick — dhrt hat sie
(noch) nicht; das Merge-Ergebnis stimmt, der aliasierte *Aufruf* liefe in dhrt
nicht.

## Stufe 5 — Verdrahtung / `dhrt run` (fertig)

[`src/main.rs`](../rust/gb_runtime/src/main.rs): `dhrt run datei.gb` ist der
eigenständige End-to-End-Lauf — preprocess → lex → parse → compile → VM, alles
in Rust. `run_main` kanonisiert den Pfad, wechselt **ins Datei-Verzeichnis**
(`set_current_dir`, wie `gbrun.py` `os.chdir(file.parent)`), damit relative
IMPORT- **und** Laufzeit-Pfade (`OpenFile("data.txt")`, `LOADIMAGE("assets/…")`)
stimmen, und nutzt den Dateinamen als Label für Laufzeitfehler. Komfort:
`dhrt datei.gb` (ohne `run`, Endung `.gb`) wird genauso behandelt; `.gbc`-Pfade
laufen weiter den direkten VM-Pfad. Die Front-End-Kette ist in
`compile_and_run_source(source, base, label)` gebündelt (geteilt mit `--runsrc`,
das **ohne** chdir läuft — Dev-/Parity-Einstieg).

Gate: stdout von `dhrt run` == Python-Tree-Walker mit demselben chdir
([`tests/test_rust_run_parity.py`](../tests/test_rust_run_parity.py), 2 Tests:
relativer Laufzeit-Datei-Zugriff + Quellcode- + Modul-IMPORT, sowie der
`dhrt <datei.gb>`-Auto-Detect). Graphics-Smoke-Test (raylib) headless verifiziert.

Damit ist die Toolchain **end-to-end ohne Python** lauffähig — Ziel der
Portierung erreicht.

### Anschluss-Features (erledigt)

- **Selbst-Export** `dhrt --export datei.gb [out_dir]`: kompiliert die Quelle
  selbst → `.gbc` und hängt den Payload (`<gbc><u64 len><DHRTPAY1>`) an eine
  Kopie der eigenen Runtime-Exe — eine eigenständige `.exe`, ohne Python, ganz
  ohne `gbrun.py`/`export.py`. `assets/` neben der Quelle wird mitkopiert. Test
  [`tests/test_rust_export.py`](../tests/test_rust_export.py) (exportiert,
  startet die Exe, vergleicht stdout mit dem Tree-Walker).
- **Aliasierte Modul-IMPORTs** `IMPORT "json" AS j`: `preprocess::compile_env`
  liefert neben den externen Typen (inkl. aliasierter wie `j_handle`/`v`) eine
  `(alias, modul)`-Liste; der Compiler bildet aliasierte Builtin-Namen
  (`j_parse` → `json_parse`, `v_new` → `vec2_new`) auf den kanonischen zurück,
  sodass dhrt sie nativ findet. Test in
  [`tests/test_rust_preprocess_parity.py`](../tests/test_rust_preprocess_parity.py)
  (`test_e2e_runsrc_module_alias`).
- **WASM-Quellkompilierung im Browser (gebaut + verifiziert 2026-06-04):** der
  emscripten-Einstieg in `main.rs` kompiliert `/program.gb` selbst (Fallback
  `/program.gbc`), `build_wasm.py` bettet die Quelle ein → der Web-Playground
  braucht **kein Pyodide** mehr. Toolchain installiert (emscripten 6.0.0 +
  Rust-Target); `build_wasm.py` verdrahtet das Windows-emscripten-Env selbst
  (`setup_emscripten_env`: CC/CXX/AR/Linker → `.exe`, bindgen-Includes,
  Ninja-Generator, cmake/ninja auf PATH). Verifiziert: `node web/dhrt.js` ==
  Python-Tree-Walker (aus eingebetteter Quelle). Details:
  [docs/web-playground.md](web-playground.md).

## Prinzip

Der Tree-Walker (`interpreter.py`) bleibt die **Referenz** und der Host der
`@builtin`-Definitionen. Die Rust-Portierung muss bit-identisches Verhalten
liefern; das schärfste Gate ist in Stufe 3 die **Bytecode-Gleichheit** (produziert
der Rust-Compiler exakt das `.gbc`, das Python erzeugt, ist Verhaltensgleichheit
garantiert, da dhrt es schon bit-identisch ausführt).
