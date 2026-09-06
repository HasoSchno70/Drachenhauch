# Drachenhauch — VS-Code-Erweiterung

Sprachunterstützung für [Drachenhauch](../README.md) (`.dh`-Dateien) in VS Code:

- **Syntax-Highlighting** (TextMate-Grammatik, aus den Schlüsselwörtern des
  Lexers und dem Befehlsindex erzeugt).
- **Language Server** — die Runtime selbst, `dhrt lsp`:
  - Diagnostics (Fehler und Warnungen der Preprocess→Lex→Parse→Compile-Kette)
  - Auto-Completion (Befehle, Schlüsselwörter, Konstanten, Symbole der Datei)
  - Hover-Doku (Befehle + eigene SUB/FUNCTION/CLASS … mit Doc-Kommentar)
  - Gehe zur Definition, Find References
  - Outline / Document Symbols (Klassen mit Methoden verschachtelt)

## Voraussetzungen

Eine gebaute `dhrt` (`python rust/build_runtime.py` im Repo, oder die
Runtime aus dem Installer). Liegt sie nicht im PATH, ihren Pfad in der
Einstellung `drachenhauch.dhrtPath` eintragen. **Python braucht die
Erweiterung nicht.**

## Installation (Entwicklung)

```bash
cd vscode-drachenhauch
npm install            # holt vscode-languageclient
```

Dann in VS Code `F5` (Extension Development Host) — oder als `.vsix` paketieren:

```bash
npm install -g @vscode/vsce
vsce package
```

## Einstellungen

- `drachenhauch.dhrtPath` — Pfad zu `dhrt` bzw. `dhrt.exe` (Vorgabe: `dhrt`,
  also die Suche im PATH), z. B.
  `C:\\Programmieren\\Python\\Drachenhauch\\rust\\drachenhauch_runtime\\target\\release\\dhrt.exe`.
- `drachenhauch.enableLanguageServer` — auf `false` für nur Syntax-Highlighting.

## Grammatik neu erzeugen

Nach Sprach-Änderungen (neue Schlüsselwörter oder Befehle):

```bash
dhrt doku grammatik
```

## Architektur

Der Server liegt in `rust/drachenhauch_runtime/src/lsp.rs` (Rahmung, Verfahren,
Hover-Daten) und `symbole.rs` (Definitionen, Fundstellen, Gliederung), geprüft
über den echten Prozess in `tests/test_dhrt_lsp.py`. Siehe
[docs/lsp.md](../docs/lsp.md).
