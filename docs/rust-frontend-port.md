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
| 2. **Parser** (`ast_nodes`+`parser`) | `src/parser.rs` (geplant) | AST kanonisch serialisieren → vergleichen | offen |
| 3. **Compiler** (`compiler`) | `src/compiler.rs` (geplant) | **Bytecode-`Module`-Gleichheit** ggü. `serialize.py`-`.gbc` | offen |
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

## Prinzip

Der Tree-Walker (`interpreter.py`) bleibt die **Referenz** und der Host der
`@builtin`-Definitionen. Die Rust-Portierung muss bit-identisches Verhalten
liefern; das schärfste Gate ist in Stufe 3 die **Bytecode-Gleichheit** (produziert
der Rust-Compiler exakt das `.gbc`, das Python erzeugt, ist Verhaltensgleichheit
garantiert, da gbrt es schon bit-identisch ausführt).
