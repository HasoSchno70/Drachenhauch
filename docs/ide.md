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

## Stand 2 (06.09.2026)

| Bereich | Was geht | Kürzel |
|---|---|---|
| Dateien | Neu, Öffnen, Sichern, Sichern unter, Reiter schließen; bis zu 12 Reiter; Rückfrage bei ungesicherten Änderungen | Strg+N, Strg+O, Strg+S, Strg+W, Strg+Q |
| Bearbeiten | Suchen, Weitersuchen, Ersetzen (alle Treffer), Gehe zu Zeile, Suche im Projekt (alle `.dh` im Projektordner, Treffer unten rechts, Doppelklick öffnet), Befehlspalette (tippen filtert, Enter führt aus) | Strg+F, F3, Strg+H, Strg+G, Strg+Umschalt+F, Strg+Umschalt+P |
| Sprache | Einfärbung des sichtbaren Ausschnitts, Hilfe zum Wort unter der Marke (Statuszeile), Vervollständigung, Zur Definition | Strg+Leer, F12 |
| Prüfen | Fehlerliste unten rechts, 0,6 s nach der letzten Änderung von selbst; Klick springt zur Zeile; Fehlerzeilen tragen eine orange Marke | Umschalt+F7 |
| Debugger | Haltepunkte (rote Marke) an der Zeile der Schreibmarke; Debuggen läuft bis zum ersten Haltepunkt, ohne Haltepunkte steht es in Zeile 1; die angehaltene Zeile ist gelb markiert, Variablen (lokal und global) stehen unten rechts anstelle der Problemliste, dazu die Schritt-Knöpfe | F9, F7, F8 weiter, F10 drüber, F11 hinein, Umschalt+F11 heraus, Umschalt+F5 stopp |
| Profil | Lauf unter `dhrt profile`; am Ende ein Fenster mit den Zeilen nach Zeit (Anzahl, ms, Anteil, Quelltext), Klick springt zur Zeile | Strg+Umschalt+Y |
| Ausführen | Starten mit laufender Ausgabe unten links, Eingabezeile für `INPUT`, Stoppen | F5, Umschalt+F5 |
| Ansicht | Helles/dunkles Thema, Vollbild; die IDE startet maximiert | Alt+Enter |

## Woraus sie gebaut ist

Die IDE braucht von `dhrt` nichts, was ein anderes Programm nicht auch
bekäme. Vier Bausteine kamen mit ihr:

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
- **Marken im Textbereich** (`GUI_TEXTAREA_MARKS(ta, zeilen, farben)`): ein
  Punkt in der Nummernspalte und ein Farbhauch über der Zeile, für
  Haltepunkte, die angehaltene Zeile und Fehlerzeilen. Sie hängen an der
  Zeilennummer, die IDE setzt sie neu, sobald sich etwas ändert.

Der Debugger ist ein Client von `dhrt debug`: das Kind schreibt Ereignisse
als JSON-Zeilen auf stdout (`paused` mit Zeile, Tiefe, `locals`, `globals`;
`output`; `finished`; `error`) und nimmt Kommandos auf stdin, aber nur,
solange es steht (`continue`, `step-over`, `step-into`, `step-out`,
`set-breakpoints`, `stop`). Beim ersten Halt in Zeile 1 schickt die IDE die
Haltepunkte und lässt es weiterlaufen, wenn es welche gibt. Der Profiler
(`dhrt profile`) liefert am Ende eine JSON-Zeile mit `total_time`, `lines`
(Zeile, Anzahl, Zeit) und der Programmausgabe.

Schrift: eine Textschrift für die Oberfläche (Segoe UI, sonst Arial oder
DejaVu Sans) und eine dicktengleiche für den Code (Consolas, sonst Menlo
oder DejaVu Sans Mono), beide in 32 px geladen und in 16 gezeichnet. Ohne
Fund bleibt raylibs Bitmapschrift.

## Prüfen ohne hinzusehen

Setzt man `DH_IDE_LOG=<datei>`, schreibt die IDE ihre Ereignisse zeilenweise
mit: `bereit`, `geoeffnet <pfad>`, `geprueft <anzahl>`, `gestartet <pfad>`,
`beendet <code>`, `gesichert`, `projekt <ordner>`, `haltepunkt <zeile> an|aus`,
`debug gestartet`, `debug pause <zeile>`, `debug beendet`, `profil <zeilen>`,
`suche <treffer>`, `palette <befehl>`, `ende`. So sieht `tests/test_ide.py`, was sie
getan hat; Tasten kommen über `AUTOMATION_PLAY` herein (F5 startet, F7
prüft). Die Bausteine einzeln prüft `tests/test_ide_bausteine.py`.

## Was noch fehlt

Willkommensseite, das Handbuch im Fenster, Drucken des Listings über das
pdf-Modul, die Begleit-Editoren aus dem Menü, Ausdruck-Auswertung im
Debugger (`eval` kann das Kind schon), bedingte Haltepunkte, ein Installer
ohne PyInstaller. Die Liste
steht in [entwurf-python-abbau.md](entwurf-python-abbau.md), Abschnitt C.
