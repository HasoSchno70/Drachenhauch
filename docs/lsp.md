# Language Server + VSCode-Extension

Neben dem eingebauten Qt-Editor gibt es einen **Language Server (LSP)** und eine
**VSCode-Extension**, die GameBasic-Unterstützung in jeden LSP-fähigen Editor
bringen — mit derselben Sprach-Intelligenz wie der Qt-Editor.

## Language Server (`gamebasic.lsp`)

Start (stdio, JSON-RPC 2.0):

```
.venv\Scripts\python.exe -m gamebasic.lsp
```

Fähigkeiten:

| LSP-Methode | Funktion |
|---|---|
| `publishDiagnostics` | Live-Fehler aus der Pipeline (Preprocess→Lex→Parse→Compile), erste Fehlerstelle |
| `completion` | Keywords, Built-ins, Konstanten, Snippets + im Dokument definierte Symbole (Präfix-gefiltert) |
| `hover` | Signatur + Doku für Built-ins **und** eigene `SUB`/`FUNCTION`/`CLASS`/… (Doc-Kommentar über der Definition) |
| `definition` | Gehe zur Definition |
| `references` | Alle Vorkommen eines Symbols |
| `documentSymbol` | Outline — Klassen mit Methoden/Properties verschachtelt, plus ENUMs |

### Architektur

Die **Feature-Logik** (`gamebasic/lsp/features.py`) ist reine, transport-freie
Funktion (Dokument-Text + Position → LSP-Daten) und damit headless testbar
(`tests/test_lsp_features.py`). Die **Transportschicht** (`gamebasic/lsp/server.py`,
`LspServer`) macht nur JSON-RPC + Dokument-Store + Position/URI-Mapping
(`tests/test_lsp_server.py`, inkl. echtem stdio-Subprozess-Test). Die
Intelligenz teilt sich denselben Code wie der Qt-Editor (`editor_qt/symbols.py`,
`error_check.py`, `builtin_docs.py`, `completer.py`).

## VSCode-Extension (`vscode-gamebasic/`)

- **Syntax-Highlighting** via TextMate-Grammatik, aus den echten Lexer-Keywords +
  registrierten Built-ins/Konstanten **generiert** (`build_grammar.py` →
  `syntaxes/gamebasic.tmLanguage.json`).
- **LSP-Client** (`extension.js`) startet `python -m gamebasic.lsp` und verbindet
  ihn über stdio.

Setup + Einstellungen (vor allem `gamebasic.pythonPath` auf den `.venv`-Python):
siehe [vscode-gamebasic/README.md](../vscode-gamebasic/README.md).
