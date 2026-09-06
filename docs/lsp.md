# Sprachserver + VS Code

Neben dem eingebauten Qt-Editor gibt es einen **Sprachserver (LSP)** und eine
**VS-Code-Erweiterung**, die Drachenhauch-Unterstützung in jeden LSP-fähigen
Editor bringen — mit derselben Diagnose wie der Qt-Editor.

## Sprachserver: `dhrt lsp`

Der Sprachserver ist die Runtime selbst. Start (stdio, JSON-RPC 2.0):

```
dhrt lsp
```

Kein Python, kein zweites Programm: Lexer, Parser, Compiler, der
Befehlsindex und die Hover-Texte liegen in `dhrt`, und der Server benutzt
sie in einem Prozess. Bis September 2026 rechnete ein Python-Server
(`drachenhauch/lsp/`) nach, was `dhrt` beim Übersetzen längst wusste — er
ist mit Weg A aus [entwurf-python-abbau.md](entwurf-python-abbau.md)
entfallen.

| LSP-Methode | Funktion |
|---|---|
| `publishDiagnostics` | Fehler und Warnungen der ganzen Kette (Preprocess → Lex → Parse → Namensraum → Compile), dieselben wie `dhrt --check`, auf die Zeilen des Puffers zurückgerechnet — ein Fehler in einer importierten Datei steht in Zeile 1 mit Herkunft |
| `completion` | Befehle aus dem Index, Schlüsselwörter, Konstanten (Farben, Tasten, `PI`, `TAU`) und die im Dokument definierten Symbole, nach dem Präfix links von der Marke gefiltert |
| `hover` | Signatur und Beschreibung für Befehle (handgepflegte Texte vor den aus `docs/` erzeugten, sonst die Signatur aus dem Index) und für eigene `SUB`/`FUNCTION`/`CLASS`/… (der Kommentarblock über der Definition) |
| `definition` | zur Definition springen |
| `references` | alle Vorkommen eines Symbols |
| `documentSymbol` | Gliederung — Klassen mit Methoden und Properties verschachtelt, dazu ENUMs |

**Diagnose im Hintergrund.** Jeder Tastendruck schickt das ganze Dokument;
die Prüfung einer Datei mit 2 800 Zeilen kostet rund 90 ms. Damit Hover und
Vervollständigung nicht dahinter warten, prüft je Dokument ein eigener Faden
mit Generationszähler: er wartet kurz, ob noch ein Anschlag folgt, und
schickt nur, wenn er der neueste ist.

### Aufbau

* `rust/drachenhauch_runtime/src/lsp.rs` — Rahmung (Content-Length),
  Dokumentspeicher, Verfahren, Hover-Daten. Die Hover-Texte kommen aus zwei
  eingebetteten Dateien: `drachenhauch/editor_qt/builtin_docs.json`
  (handgepflegt, gewinnt) und `builtin_prosa.json` (aus `docs/` erzeugt, siehe
  `dhrt doku`).
* `rust/drachenhauch_runtime/src/symbole.rs` — Definitionen, Fundstellen,
  Blöcke und Kommentar-Doku aus dem Text, mit Kommentaren und Zeichenketten
  ausgeblendet; bewusst kein Lexer, weil ein Sprachserver halb getippten Text
  sieht.
* Prüfstein: `tests/test_dhrt_lsp.py` fährt den echten Prozess über stdio
  (Fähigkeiten, Diagnose, Definition, Hover, Vervollständigung, Fundstellen,
  Gliederung, kaputte Rahmung, sauberes Ende); dazu Rust-Tests in beiden
  Dateien.

Der Qt-Editor benutzt für dieselben Aufgaben noch seine Python-Bausteine
(`editor_qt/symbols.py`, `error_check.py`); sie wandern mit der IDE (Weg C).

## VS-Code-Erweiterung (`vscode-drachenhauch/`)

- **Syntax-Highlighting** über eine TextMate-Grammatik, aus den
  Schlüsselwörtern des Lexers und dem Befehlsindex **erzeugt**
  (`dhrt doku grammatik` → `syntaxes/drachenhauch.tmLanguage.json`).
- **LSP-Client** (`extension.js`) startet `dhrt lsp` und verbindet ihn über
  stdio.

Einstellung `drachenhauch.dhrtPath`: Pfad zur Runtime, falls sie nicht im
PATH liegt. Details: [vscode-drachenhauch/README.md](../vscode-drachenhauch/README.md).
