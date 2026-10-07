# Language server + VS Code

Besides the built-in Qt editor there is a **language server (LSP)** and a
**VS Code extension** that bring Drachenhauch support to any LSP-capable
editor — with the same diagnostics as the Qt editor.

## Language server: `dhrt lsp`

The language server is the runtime itself. Start (stdio, JSON-RPC 2.0):

```
dhrt lsp
```

No Python, no second program: lexer, parser, compiler, the command index and
the hover texts live in `dhrt`, and the server uses them in one process. Until
September 2026 a Python server (`drachenhauch/lsp/`) recomputed what `dhrt`
already knew when compiling — it was dropped with path A from
[entwurf-python-abbau.md](../entwurf-python-abbau.md) (German).

| LSP method | Function |
|---|---|
| `publishDiagnostics` | errors, warnings and hints (unused variables in subroutines, severity 4, `unnecessary`) from the whole chain (preprocess → lex → parse → namespace → compile), the same as `dhrt --check`, mapped back onto the lines of the buffer — an error in an imported file appears on line 1 with its origin |
| `completion` | commands from the index, keywords, constants (colours, keys, `PI`, `TAU`) and the symbols defined in the document, filtered by the prefix left of the cursor |
| `hover` | signature and description for commands (hand-written texts before the ones generated from `docs/`, otherwise the signature from the index) and for your own `SUB`/`FUNCTION`/`CLASS`/… (the comment block above the definition) |
| `definition` | jump to definition |
| `references` | all occurrences of a symbol |
| `documentSymbol` | outline — classes with methods and properties nested, plus ENUMs |
| `codeAction` | quick fixes (`quickfix`) for the messages on the requested lines: insert the suggested name, add a missing `DIM`, set brackets, replace `!=` with `<>`, a missing block end, a missing `IMPORT`, remove an unused variable -- read from the message itself, so the fix does what the message suggests |

**Diagnostics in the background.** Every keystroke sends the whole document;
checking a file with 2,800 lines costs about 90 ms. So that hover and
completion do not wait behind it, a separate thread per document checks with
a generation counter: it waits briefly to see whether another keystroke
follows, and only sends if it is the newest.

### Structure

* `rust/drachenhauch_runtime/src/lsp.rs` — framing (Content-Length),
  document store, methods, hover data. The hover texts come from two embedded
  files: `daten/builtin_docs.json` (hand-written, wins) and
  `builtin_prosa.json` (generated from `docs/`, see `dhrt doku`).
* `rust/drachenhauch_runtime/src/symbole.rs` — definitions, occurrences,
  blocks and comment docs from the text, with comments and strings blanked
  out; deliberately not a lexer, because a language server sees half-typed
  text.
* Touchstone: `tests/pruef/dhrt_lsp.dhtest` drives the real process over
  stdio (capabilities, diagnostics, definition, hover, completion,
  occurrences, outline, broken framing, clean shutdown); plus Rust tests in
  both files.

The IDE in Drachenhauch (`ide/ide.dh`) queries the same core through the
commands `CODE_CHECK$`, `CODE_HOVER$`, `CODE_COMPLETE` and so on; the former
Python building blocks of the Qt editor were deleted on 2026-09-21.

## VS Code extension (`vscode-drachenhauch/`)

- **Syntax highlighting** via a TextMate grammar, **generated** from the
  lexer's keywords and the command index
  (`dhrt doku grammatik` → `syntaxes/drachenhauch.tmLanguage.json`).
- **LSP client** (`extension.js`) starts `dhrt lsp` and connects to it over
  stdio.

Setting `drachenhauch.dhrtPath`: path to the runtime if it is not on the
PATH. Details: [vscode-drachenhauch/README.md](../../vscode-drachenhauch/README.md).
