# GameBasic — VSCode-Extension

Sprachunterstützung für [GameBasic](../README.md) (`.gb`-Dateien) in VS Code:

- **Syntax-Highlighting** (TextMate-Grammatik, aus den echten Lexer-Keywords +
  registrierten Built-ins/Konstanten generiert).
- **Language Server** (Python, `gamebasic.lsp`):
  - Diagnostics (Live-Fehler aus der Preprocess→Lex→Parse→Compile-Pipeline)
  - Auto-Completion (Keywords, Built-ins, Konstanten, Snippets, Symbole der Datei)
  - Hover-Doku (Built-ins + eigene SUB/FUNCTION/CLASS … mit Doc-Kommentar)
  - Gehe zur Definition, Find References
  - Outline / Document Symbols (Klassen mit Methoden verschachtelt)

## Voraussetzungen

Der Language Server ist Teil des GameBasic-Repos und läuft über den Python des
Projekts. Damit `python -m gamebasic.lsp` das Paket findet, am einfachsten **den
GameBasic-Projektordner in VS Code öffnen** (der Server startet mit diesem Ordner
als Arbeitsverzeichnis).

## Installation (Entwicklung)

```bash
cd vscode-gamebasic
npm install            # holt vscode-languageclient
```

Dann in VS Code `F5` (Extension Development Host) — oder als `.vsix` paketieren:

```bash
npm install -g @vscode/vsce
vsce package
```

## Einstellungen

- `gamebasic.pythonPath` — Python-Interpreter für den Server. **Auf den
  `.venv`-Python des Projekts setzen**, z. B.
  `C:\\Programmieren\\Python\\GameBasic\\.venv\\Scripts\\python.exe`.
- `gamebasic.serverModule` — Server-Modul (Default `gamebasic.lsp`).
- `gamebasic.enableLanguageServer` — auf `false` für nur Syntax-Highlighting.

## Grammatik neu generieren

Nach Sprach-Änderungen (neue Keywords/Built-ins):

```bash
.venv\\Scripts\\python.exe vscode-gamebasic\\build_grammar.py
```

## Architektur

Der Server (`gamebasic/lsp/`) trennt **Feature-Logik** (`features.py`, headless
getestet in `tests/test_lsp_features.py`) von der **JSON-RPC-Transportschicht**
(`server.py`, getestet in `tests/test_lsp_server.py` inkl. echtem stdio-
Subprozess). Die Sprach-Intelligenz teilt sich denselben Code wie der eingebaute
Qt-Editor (`gamebasic/editor_qt/`).
