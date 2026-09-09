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

## Stand 5 (09.09.2026)

| Bereich | Was geht | Kürzel |
|---|---|---|
| Start | Willkommensseite, solange kein Reiter offen ist: Neu, Öffnen, Beispiele, die zuletzt geöffneten Dateien (Doppelklick), die wichtigsten Kürzel. Die Sitzung kommt beim nächsten Start wieder (offene Dateien, aktiver Reiter), ebenso Thema und Schriftgröße | |
| Dateien | Neu, Öffnen, Datei im Projekt öffnen (Wähler mit unscharfem Filter: `spred` findet `189_sprite_editor.dh`; dazu die zuletzt geöffneten), Zuletzt geöffnet (Untermenü), Sichern, Sichern unter, Reiter schließen; bis zu 12 Reiter; Rückfrage bei ungesicherten Änderungen; Listing drucken über einen Druckdialog (Drucker, Kopien) oder als PDF neben die Quelle (Courier 9 pt, 66 Zeilen je Seite, Zeilennummern, Kopfzeile mit Seitenzahl) | Strg+N, Strg+O, Strg+Umschalt+O, Strg+S, Strg+W, Strg+Q, Strg+P |
| Bearbeiten | Suchen, Weitersuchen, Ersetzen (alle Treffer), Gehe zu Zeile, Suche im Projekt (alle `.dh` im Projektordner, Treffer unten rechts, Doppelklick öffnet), TODO/FIXME-Liste (dieselbe Suche), Befehlspalette (tippen filtert, Enter führt aus), Wer hat das geschrieben (`git blame` über die Datei, Datum und Person je Zeile in derselben Liste) | Strg+F, F3, Strg+H, Strg+G, Strg+Umschalt+F, Strg+Umschalt+M, Strg+Umschalt+P, Strg+Umschalt+B |
| Schreiben | Umbenennen eines Symbols über die ganze Datei (`CODE_RENAME$`: ganze Wörter, Kommentare und Zeichenketten bleiben; ein krummer Name ändert nichts), Schnipsel einfügen (13 Gerüste, `\|` sagt wohin die Marke gehört, die Einrückung der Zeile wird übernommen), eine Marke auf jede Fundstelle des Wortes — danach ändert ein Tippen alle; Alt+Klick legt eine Marke dazu, ESC räumt sie weg. Signaturhilfe: steht die Marke in einer Argumentliste, zeigt die Statuszeile die Signatur des Aufrufs und die Nummer des Arguments | Umschalt+F6, Strg+J, Strg+Umschalt+L |
| Zeilen | Alles auf ganzen Zeilen, der Auswahl oder der Zeile der Marke, jeder Handgriff ein eigener Undo-Schritt: Kommentar umschalten, Zeile duplizieren, Zeile löschen, Zeilen nach oben/unten, Ein-/Ausrücken (Tab gehört dem Feld selbst), Dokument formatieren (`CODE_FORMAT$`, derselbe Formatierer wie `dhrt fmt`; bei einem Syntaxfehler bleibt alles stehen), Lesezeichen (blaue Marke) setzen und anspringen | Strg+K, Strg+D, Strg+Umschalt+K, Alt+Hoch/Runter, Alt+Rechts/Links, Umschalt+Alt+F, Strg+F2, F2 |
| Sprache | Einfärbung des sichtbaren Ausschnitts, Hilfe zum Wort unter der Marke (Statuszeile), Vervollständigung, Zur Definition, Gliederung links unten (SUB/FUNCTION/CLASS mit Methoden, Doppelklick springt) | Strg+Leer, F12 |
| Prüfen | Fehlerliste unten rechts, 0,6 s nach der letzten Änderung von selbst; Klick springt zur Zeile; Fehlerzeilen tragen eine orange Marke | Umschalt+F7 |
| Debugger | Haltepunkte (rote Marke) an der Zeile der Schreibmarke, bedingte Haltepunkte (violett; `i = 3`, `hp < 10` — der Ausdruck wird im Programm ausgewertet, gehalten wird nur, wenn er wahr ist); Debuggen läuft bis zum ersten Haltepunkt, ohne Haltepunkte steht es in Zeile 1; die angehaltene Zeile ist gelb markiert, Variablen (lokal und global) stehen unten rechts anstelle der Problemliste, dazu die Schritt-Knöpfe; steht das Programm, wertet die Eingabezeile Ausdrücke aus (`? 2 * 21` → `= 42 (INTEGER)`) | F9, Umschalt+F9, F7, F8 weiter, F10 drüber, F11 hinein, Umschalt+F11 heraus, Umschalt+F5 stopp, Strg+E Ausdruck |
| Profil | Lauf unter `dhrt profile`; am Ende ein Fenster mit den Zeilen nach Zeit (Anzahl, ms, Anteil, Quelltext), Klick springt zur Zeile | Strg+Umschalt+Y |
| Ausführen | Starten mit laufender Ausgabe unten links, Eingabezeile für `INPUT`, Stoppen; Export als eigenständiges Programm (`dhrt --export`, nach `<name>_dist/` neben die Quelle, die Ausgabe des Exports läuft unten links mit) | F5, Umschalt+F5, Strg+F6 |
| Werkzeuge | Die Begleit-Editoren in Drachenhauch (SFX-Generator, Partikel-Editor, Tilemap-Editor, Sprite-Editor, Tracker, Form-Designer, Anim-FSM-Editor, Notenblatt) als eigene Programme; die Beispiele als Projekt öffnen | |
| Ansicht | Helles/dunkles Thema, Vollbild, Schrift größer/kleiner (10 bis 32 px, bleibt gemerkt); Zeilenumbruch (gilt für alle Reiter); Blöcke falten (die Blöcke kommen aus `CODE_SYMBOLS$`, ein Klick auf das Dreieck in der Nummernspalte tut dasselbe); geteilte Ansicht (zwei Reiter nebeneinander); Übersichtskarte am rechten Rand (ein Strich je Zeile, heller Kasten für den sichtbaren Ausschnitt, Klick springt); die IDE startet maximiert | Alt+Enter, Alt+Z, F4, Strg+F4, Umschalt+F4, Alt+G |
| Hilfe | Handbuch im Fenster, **gesetzt statt roh**: Überschriften in drei Größen, Absätze umgebrochen, Aufzählungen mit Punkt, Code-Blöcke dicktengleich, Tabellen als Begriffsliste; der Knopf oben rechts schaltet auf den Quelltext. F1 schlägt das Wort unter der Schreibmarke in `docs/` nach und öffnet das Dokument mit den meisten Fundstellen in Codeschrift; Klappliste aller Dokumente, Suche im Dokument; Tastenkürzel-Übersicht | F1, Strg+F1 |

Einstellungen und Sitzung liegen in EINER JSON-Datei im Nutzerprofil
(`%APPDATA%\Drachenhauch\ide.json`, sonst `~/.config/Drachenhauch/ide.json`):
`zuletzt`, `sitzung`, `aktiv`, `hell`, `schrift`, `umbruch`, `karte` und
`projekte` (Sitzung je Ordner, die zwölf letzten — wer an zwei Sachen
arbeitet, will beim Wechsel nicht die Reiter der anderen wiederfinden).
Die Umgebungsvariable
`DH_IDE_KONFIG` legt einen anderen Ort fest — die Tests brauchen das, sonst
schriebe jeder Testlauf dem Nutzer eine fremde Sitzung in die Datei.

## Woraus sie gebaut ist

Die IDE braucht von `dhrt` nichts, was ein anderes Programm nicht auch
bekäme. Diese Bausteine kamen mit ihr:

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
- **Der Auswahlbereich** (`GUI_TEXTAREA_SELECTION_RANGE(ta)` → Zeile und
  Spalte von Anfang und Ende): der markierte Text allein
  (`GUI_TEXTAREA_SELECTION$`) sagt nicht, WELCHE Zeilen gemeint sind — und
  Kommentar umschalten, Einrücken oder Zeilen verschieben arbeiten auf
  Zeilen. Geschrieben wird dann über `GUI_TEXTAREA_SELECT` +
  `GUI_TEXTAREA_INSERT`, damit jeder Handgriff ein eigener Undo-Schritt
  bleibt (`GUI_SET_TEXT` würde den Verlauf leeren).
- **Der Formatierer als Builtin** (`CODE_FORMAT$(quelltext$)`): derselbe
  Kern wie `dhrt fmt`, ohne Prozess und ohne Datei. Leer, wenn sich die
  Quelle nicht lesen lässt — an kaputtem Code rückt niemand herum.
- **Fenstertitel nachträglich** (`GUI_WINDOW_TITLE(win, titel$)`): die
  Befehlspalette und der Datei-Wähler sind dasselbe Fenster.
- **Faltung** (`GUI_TEXTAREA_FOLDABLE/FOLD/FOLD_ALL/FOLDED/FOLDS`): welche
  Zeilen einen Block bilden, sagt das Programm — die Laufzeit kennt hier
  keine Sprache und zählt keine Einrückung. Sie sitzt in `ta_rows`, der
  einen Quelle sichtbarer Zeilen, und wirkt damit für Zeichnen, Klick,
  Pfeile, Schreibmarke und Scroll auf einmal.
- **Mehrere Schreibmarken** (`GUI_TEXTAREA_ADD_CARET/CARETS/CLEAR_CARETS`,
  Alt+Klick, ESC): jede Änderung läuft durch **eine** Stelle, die sie an
  jeder Marke von hinten nach vorn anwendet. Mit einer Marke ist das genau
  der Weg von vorher.
- **Umbenennen** (`CODE_RENAME$`): dieselben Fundstellen wie
  `CODE_REFERENCES`, aber mit Spalten — die wirft der Referenz-Befehl weg.
- **Sichtbarkeit lesen** (`GUI_WINDOW_SHOWN(win)`): ohne den Getter muss ein
  Programm sich merken, was es selbst gesetzt hat, und liegt daneben,
  sobald der Nutzer das Fenster über sein Kreuz schließt.

Der Debugger ist ein Client von `dhrt debug`: das Kind schreibt Ereignisse
als JSON-Zeilen auf stdout (`paused` mit Zeile, Tiefe, `locals`, `globals`;
`output`; `finished`; `error`) und nimmt Kommandos auf stdin, aber nur,
solange es steht (`continue`, `step-over`, `step-into`, `step-out`,
`set-breakpoints`, `stop`). Beim ersten Halt in Zeile 1 schickt die IDE die
Haltepunkte und lässt es weiterlaufen, wenn es welche gibt. Der Profiler
(`dhrt profile`) liefert am Ende eine JSON-Zeile mit `total_time`, `lines`
(Zeile, Anzahl, Zeit) und der Programmausgabe.

Wo Handbuch und Beispiele liegen: `docs/` und `examples/` neben `ide/`, im
Repo wie in der Installation; die Umgebungsvariable `DH_IDE_WURZEL`
übersteuert das (die Tests brauchen es, weil ihre IDE-Kopie woanders
liegt). Fehlen die Beispiele neben `ide/`, sieht die IDE unter
`%PUBLIC%\Documents\Drachenhauch\examples` nach, wohin der Installer
sie legt.

Schrift: eine Textschrift für die Oberfläche (Segoe UI, sonst Arial oder
DejaVu Sans) und eine dicktengleiche für den Code (Consolas, sonst Menlo
oder DejaVu Sans Mono), beide in 32 px geladen und in 16 gezeichnet. Ohne
Fund bleibt raylibs Bitmapschrift.

## Prüfen ohne hinzusehen

Setzt man `DH_IDE_LOG=<datei>`, schreibt die IDE ihre Ereignisse zeilenweise
mit: `bereit`, `geoeffnet <pfad>`, `geprueft <anzahl>`, `gestartet <pfad>`,
`beendet <code>`, `gesichert`, `projekt <ordner>`, `haltepunkt <zeile> an|aus`,
`debug gestartet`, `debug pause <zeile>`, `debug beendet`, `profil <zeilen>`,
`suche <treffer>`, `palette <befehl>`, `handbuch <datei> <zeile>`, `pdf <pfad>`,
`gedruckt <drucker> <kopien>`, `werkzeug <datei>`, `eval <wert>`,
`haltepunkt <zeile> bedingt <ausdruck>`, `lesezeichen <zeile> an|aus`,
`lesezeichen sprung <zeile>`, `kommentar <von>-<bis>`, `dupliziert <von>-<bis>`,
`geloescht <von>-<bis>`, `verschoben <von>-<bis> <richtung>`, `formatiert`,
`gliederung <anzahl>`, `schnell <pfad>`, `export <pfad>`,
`exportiert <code> <ordner>`, `kuerzel <anzahl>`, `falte <zeile> zu|auf`,
`falten alle <anzahl>`, `geteilt <reiter> x <lage>+<breite> <lage>+<breite>`,
`geteilt aus`, `karte an|aus`, `karte sprung <zeile>`,
`umbenannt <anzahl> <name>`, `schnipsel <name>`, `signatur <text>`,
`marken <anzahl>`, `blame <zeilen>`, `hbansicht gesetzt|quelltext`, `ende`. So sieht `tests/test_ide.py`, was sie
getan hat; Tasten kommen über `AUTOMATION_PLAY` herein (F5 startet, F7
prüft). Die Bausteine einzeln prüft `tests/test_ide_bausteine.py`.

## Was noch fehlt

Die Liste aus Stand 4 ist abgearbeitet: gerenderte Markdown-Ansicht,
Codefaltung, Übersichtskarte, geteilter Editor, mehrere Schreibmarken,
Schnipsel, Signaturhilfe, Umbenennen, Git-Blame, Sitzung je Projekt,
Zeilenumbruch und Symbole in den Menüs sind alle da. Zwei Bausteine der
Laufzeit kamen dafür dazu (Faltung und mehrere Schreibmarken im
Textbereich), dazu `CODE_RENAME$` und `GUI_WINDOW_SHOWN`.

Was gegen die Qt-IDE offen bleibt, ist jetzt kleinteiliger: Faltung nach
Einrückung statt nach Blöcken (heute faltet nur, was `CODE_SYMBOLS$`
kennt), Git-Diff und -Verlauf im Fenster (nur `blame` ist da), eine Suche
mit regulären Ausdrücken, Lesezeichen über Dateien hinweg, und eine
Übersichtskarte, die den Text zeichnet statt nur seine Länge. Ein
Installer ohne Python gibt es seit Stand 3:
`installer/Drachenhauch-IDE.iss` packt `dhrt.exe`, `ide/`, `docs/` und die
Beispiele -- 33 MB statt 92; die Qt-IDE bleibt daneben installierbar, bis
diese hier gleichzieht. Die Liste steht in
[entwurf-python-abbau.md](entwurf-python-abbau.md), Abschnitt C.
