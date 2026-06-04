# Front-End-Portierung nach Rust (Lexer → Parser → Compiler)

**Ziel:** `gbrt` soll perspektivisch ohne Python aus Quelltext Bytecode erzeugen
und ausführen — dann läuft Python nur noch in den Editoren/Tools (+ als
Referenz-Tree-Walker). Damit wird auch der [Web-Playground](web-playground.md)
ein einziges reines Rust-WASM (kein Pyodide nötig).

Heute macht gbrt nur die **Ausführung** (`.gbc` → VM). Die Front-End-Toolchain
(`lexer`/`tokens`/`parser`/`ast_nodes`/`compiler`/`preprocess`, ~5.100 Zeilen
Python) wird inkrementell nach Rust portiert — **jede Stufe gegen Python
verifiziert** (cargo + rustc sind verfügbar, also hier beweisbar).

## Stufen

| Stufe | Rust | Verifikation | Status |
|---|---|---|---|
| 1. **Lexer** (`tokens`+`lexer`) | `src/lexer.rs` | Token-Strom `[TYP,wert,zeile]` via `gbrt --tokens` == Python (alle Beispiele + Snippets) | ✅ **fertig** |
| 2. **Parser** (`ast_nodes`+`parser`) | `src/ast.rs` + `src/parser.rs` | AST als kanonisches JSON via `gbrt --ast` == Python (struktureller Vergleich) | ✅ **fertig** |
| 3. **Compiler** (`compiler`) | `src/compiler.rs` | **Output-Parität**: `gbrt --runsrc` (Rust lex+parse+compile+run) == Python-Tree-Walker | 🟡 **3a+3b fertig** |
| 4. **Preprocess** (`IMPORT`) | `src/preprocess.rs` (geplant) | Merge-Ergebnis-Gleichheit | offen |
| 5. **Verdrahtung** | `gbrt run datei.gb` / `--export` | Output-Parität (gbrt-self-compiled vs Python-TW) | offen |

## Stufe 1 — Lexer (fertig)

- [`rust/gb_runtime/src/lexer.rs`](../rust/gb_runtime/src/lexer.rs): `Tt`
  (Token-Typen mit exakt den Python-`TokenType.name`-Strings), `Lexer`,
  f-String-Expansion (inkl. Format-Spec `{x:.2f}` → `FORMAT$`), `&H`/`&B`,
  Zeilenfortsetzung (`_`+Newline, implizit in Klammern), lowercase-Idents,
  `$`-Suffix.
- Debug-Einstieg `gbrt --tokens <datei.gb>` → eine JSON-Zeile `[TYP, wert,
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
  `gbrt --ast <datei.gb>`.
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
`.gbc`-JSON. Debug-/Run-Einstieg `gbrt --runsrc <datei.gb>` macht **alles in
Rust** (lex+parse+compile+run).

**Gate-Entscheidung: Output-Parität statt byte-exaktem Bytecode.** gbrt's VM
implementiert den vollen Opcode-Satz — sowohl generische (`ADD`, `LOAD_NAME`)
als auch optimierte (`ADD_NN`, Slot-Globals, Inline-Caches). Der Rust-Compiler
emittiert die **generischen** Opcodes (kein Constant-Folding, keine `_NN`, keine
Inline-Caches) — das Verhalten ist identisch, der Code viel kleiner. Verifiziert
wird per stdout-Vergleich `gbrt --runsrc` == Python-Tree-Walker
([`tests/test_rust_compiler_parity.py`](../tests/test_rust_compiler_parity.py)),
dasselbe Korrektheits-Prinzip wie `test_gbrt_parity`. (Performance-Parität —
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
*Bekannte gbrt-Grenze (nicht 3b-spezifisch):* sizeless `DIM x AS ARRAY OF T`
wird von gbrt nicht leer initialisiert (auch bei Python-kompiliertem `.gbc`).

**Nächste Teil-Stufen** (je eigener Commit, Korpus wächst):
3c User-`SUB`/`FUNCTION` (Locals, Params, Defaults, Variadic, `CALL_USER`,
FUNCREF) · 3d Klassen/Structs (`NEW`, Member, `Self`, Properties, Operatoren,
Statics, ENUM, member/index-READ-Ziel) · 3e Comprehensions, `SELECT`, Tupel,
`WITH`, `TRY`, `FOR EACH`, Coroutinen/`YIELD` · dann `gbrt run datei.gb`
(Stufe 5) + Preprocess (Stufe 4).

## Prinzip

Der Tree-Walker (`interpreter.py`) bleibt die **Referenz** und der Host der
`@builtin`-Definitionen. Die Rust-Portierung muss bit-identisches Verhalten
liefern; das schärfste Gate ist in Stufe 3 die **Bytecode-Gleichheit** (produziert
der Rust-Compiler exakt das `.gbc`, das Python erzeugt, ist Verhaltensgleichheit
garantiert, da gbrt es schon bit-identisch ausführt).
