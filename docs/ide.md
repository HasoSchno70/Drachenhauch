# Die IDE in Drachenhauch (`ide/ide.dh`)

Eine Entwicklungsumgebung, geschrieben in der Sprache, die sie bedient.
Sie ist Weg C aus [Entwurf: Python abbauen](entwurf-python-abbau.md): die
Qt-IDE (32 000 Zeilen Python) soll durch ein Drachenhauch-Programm abgelöst
werden. Bis das gleichzieht, bleibt die Qt-IDE im Installer.

```
dhrt run ide/ide.dh                 # leer starten
dhrt run ide/ide.dh -- spiel.dh     # mit einer Datei
```

Der Projektbaum links zeigt den Ordner, in dem man beim Start stand, und ein
relativer Dateiname meint eine Datei dort. Das ist nicht selbstverständlich:
`dhrt run` wechselt vor dem Lauf ins Verzeichnis der Quelle, also nach `ide/`.
Damit ein Programm den Ort des Aufrufers trotzdem kennt, hinterlegt `dhrt`
ihn vorher in der Umgebungsvariable `DHRT_START_DIR`.

## Stand 1 (06.09.2026)

| Bereich | Was geht | Kürzel |
|---|---|---|
| Dateien | Neu, Öffnen, Sichern, Sichern unter, Reiter schließen; bis zu 12 Reiter; Rückfrage bei ungesicherten Änderungen | Strg+N, Strg+O, Strg+S, Strg+W, Strg+Q |
| Bearbeiten | Suchen, Weitersuchen, Ersetzen (alle Treffer), Gehe zu Zeile | Strg+F, F3, Strg+H, Strg+G |
| Sprache | Einfärbung des sichtbaren Ausschnitts, Hilfe zum Wort unter der Marke (Statuszeile), Vervollständigung, Zur Definition | Strg+Leer, F12 |
| Prüfen | Fehlerliste unten rechts, 0,6 s nach der letzten Änderung von selbst; Klick springt zur Zeile | F7 |
| Ausführen | Starten mit laufender Ausgabe unten links, Eingabezeile für `INPUT`, Stoppen | F5, Umschalt+F5 |
| Ansicht | Helles/dunkles Thema | |

## Woraus sie gebaut ist

Die IDE braucht von `dhrt` nichts, was ein anderes Programm nicht auch
bekäme. Drei Bausteine kamen mit ihr:

- **Prozesse mit laufender Ausgabe** (`PROCESS_START/READ$/ERR$/WRITE/
  RUNNING/CODE/KILL/CLOSE`, [builtins-core.md](builtins-core.md)). Die
  Ausgabe des gestarteten Programms kommt zeilenweise an, während es
  läuft; `PROCESS_WRITE` reicht Eingaben an sein `INPUT` durch. Ein von
  `PROCESS_START` gestartetes `dhrt` schreibt jede `PRINT`-Zeile sofort
  hinaus (Umgebungsvariable `DHRT_LIVE=1`), sonst käme an einer Leitung
  alles erst am Ende.
- **Sprachdienste als Builtins** (`CODE_CHECK$`, `CODE_HOVER$`,
  `CODE_COMPLETE`, `CODE_DEFINITION`, `CODE_REFERENCES`, `CODE_SYMBOLS$`).
  Derselbe Kern wie `dhrt lsp` und `dhrt --check`, ohne Prozess und ohne
  JSON-RPC.
- **Textbereich-Befehle** (`GUI_TEXTAREA_CURSOR/GOTO/SELECTION$/SELECT/
  INSERT/FIND`, [module-gui.md](module-gui.md)): Marke lesen und setzen,
  Auswahl lesen und setzen, an der Marke einfügen, ab einer Stelle suchen.
  Ohne sie konnte ein Programm einen Textbereich nur ganz lesen und ganz
  schreiben.

## Prüfen ohne hinzusehen

Setzt man `DH_IDE_LOG=<datei>`, schreibt die IDE ihre Ereignisse zeilenweise
mit: `bereit`, `geoeffnet <pfad>`, `geprueft <anzahl>`, `gestartet <pfad>`,
`beendet <code>`, `gesichert`, `projekt <ordner>`, `ende`. So sieht `tests/test_ide.py`, was sie
getan hat; Tasten kommen über `AUTOMATION_PLAY` herein (F5 startet, F7
prüft). Die Bausteine einzeln prüft `tests/test_ide_bausteine.py`.

## Was noch fehlt

Debugger- und Profiler-Fenster (als Clients von `dhrt debug` und
`dhrt profile`), Suche im Projekt, Befehlspalette, Willkommensseite, das
Handbuch im Fenster, Drucken des Listings über das pdf-Modul, die
Begleit-Editoren aus dem Menü, ein Installer ohne PyInstaller. Die Liste
steht in [entwurf-python-abbau.md](entwurf-python-abbau.md), Abschnitt C.
