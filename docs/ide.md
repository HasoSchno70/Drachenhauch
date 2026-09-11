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

## Stand 7 (09.09.2026)

| Bereich | Was geht | Kürzel |
|---|---|---|
| Start | Willkommensseite, solange kein Reiter offen ist: Neu, Öffnen, Beispiele, die zuletzt geöffneten Dateien (Doppelklick), die wichtigsten Kürzel. Die Sitzung kommt beim nächsten Start wieder (offene Dateien, aktiver Reiter), ebenso Thema und Schriftgröße | |
| Dateien | Neu, Öffnen, Datei im Projekt öffnen (Wähler mit unscharfem Filter: `spred` findet `189_sprite_editor.dh`; dazu die zuletzt geöffneten), Zuletzt geöffnet (Untermenü), Sichern, Sichern unter, Reiter schließen und wieder öffnen (Strg+Umschalt+T); bis zu 12 Reiter; Rückfrage bei ungesicherten Änderungen; Listing drucken über einen Druckdialog (Drucker, Kopien) oder als PDF neben die Quelle (Courier 9 pt, 66 Zeilen je Seite, Zeilennummern, Kopfzeile mit Seitenzahl) | Strg+N, Strg+O, Strg+Umschalt+O, Strg+S, Strg+W, Strg+Q, Strg+P |
| Bearbeiten | Suchen, Weitersuchen, Ersetzen (alle Treffer), Gehe zu Zeile, Suche im Projekt (alle `.dh` im Projektordner, Treffer unten rechts, Doppelklick öffnet), wahlweise mit **regulärem Ausdruck** (ein Schalter für Suchen, Ersetzen und Projektsuche), TODO/FIXME-Liste (dieselbe Suche), Befehlspalette (tippen filtert, Enter führt aus) | Strg+F, F3, Strg+H, Strg+G, Strg+Umschalt+F, Strg+Umschalt+M, Strg+Umschalt+P |
| Git | Wer hat das geschrieben (`git blame`, Datum und Person je Zeile), was habe ich geändert (`git diff` farbig im Fenster), Verlauf dieser Datei (`git log`); die geänderten Zeilen tragen eine Marke am Rand (nach dem Sichern neu gefragt, nicht je Bild) | Strg+Umschalt+B, Strg+Umschalt+D |
| Schreiben | Umbenennen eines Symbols über die ganze Datei (`CODE_RENAME$`: ganze Wörter, Kommentare und Zeichenketten bleiben; ein krummer Name ändert nichts), Schnipsel einfügen (13 Gerüste, `\|` sagt wohin die Marke gehört, die Einrückung der Zeile wird übernommen), eine Marke auf jede Fundstelle des Wortes — danach ändert ein Tippen alle; Alt+Klick legt eine Marke dazu, ESC räumt sie weg. Signaturhilfe: steht die Marke in einer Argumentliste, zeigt die Statuszeile die Signatur des Aufrufs und die Nummer des Arguments | Umschalt+F6, Strg+J, Strg+Umschalt+L |
| Einstellungen | Strg+U zeigt alle Schalter an einer Stelle (Thema, Umbruch, Karte, geänderte Zeilen, reguläre Ausdrücke, Vorschlagsliste, Schriftgröße, automatisches Sichern). Sie wirken **sofort**, ohne Übernehmen — ein Thema, das man erst nach dem Schließen sieht, wählt man blind; im Menü stehen sie weiter dort, wo man sie sucht | Strg+U |
| Zeilen | Alles auf ganzen Zeilen, der Auswahl oder der Zeile der Marke, jeder Handgriff ein eigener Undo-Schritt: Kommentar umschalten, Zeile duplizieren, Zeile löschen, Zeilen nach oben/unten, Ein-/Ausrücken (Tab gehört dem Feld selbst), Dokument formatieren (`CODE_FORMAT$`, derselbe Formatierer wie `dhrt fmt`; bei einem Syntaxfehler bleibt alles stehen), Schnipsel einfügen (Strg+J füllt den Filter mit dem Wortanfang links der Marke: wer `for` getippt hat, hat die FOR-Schleife vor sich), automatisch sichern nach einstellbarer Ruhe. Eine neue Zeile übernimmt die **Einrückung** der alten und rückt hinter `SUB`, `FOR`, `THEN` und den anderen Blockwörtern eine Stufe weiter ein; ein `END` oder `NEXT` allein in einer Zeile rückt sie zurück. Lesezeichen (blaue Marke) setzen und anspringen — vorwärts und rückwärts **über Dateien hinweg**, dazu die Liste aller | Strg+K, Strg+D, Strg+Umschalt+K, Alt+Hoch/Runter, Alt+Rechts/Links, Umschalt+Alt+F, Strg+F2, F2, Umschalt+F2, Alt+F2 |
| Sprache | Einfärbung des sichtbaren Ausschnitts, dazu jede Fundstelle des Wortes unter der Marke, das zusammengehörende Klammernpaar und ein **Farbfeld** neben jedem `&H`-Literal (ein Klick darauf öffnet den Farbwähler und schreibt die neue Farbe an die Stelle zurück); eine **Symbolspur** über dem Code sagt, in welcher Klasse und welchem Unterprogramm man steht; **Definition hier zeigen** (Alt+F12) blendet zehn Zeilen um die Definition ein, ohne die Stelle zu verlassen; Hilfe zum Wort (Statuszeile), Vervollständigung — sie geht beim Tippen ab drei Zeichen von selbst auf, der Fokus bleibt dabei im Code-Feld, Strg+Leer holt sie herein; Zur Definition, Gliederung links unten (SUB/FUNCTION/CLASS mit Methoden, Doppelklick springt) | Strg+Leer, F12 |
| Prüfen | Fehlerliste unten rechts, 0,6 s nach der letzten Änderung von selbst; Klick springt zur Zeile; Fehlerzeilen tragen eine orange Marke | Umschalt+F7 |
| Debugger | Haltepunkte (rote Marke) an der Zeile der Schreibmarke, bedingte Haltepunkte (violett; `i = 3`, `hp < 10` — der Ausdruck wird im Programm ausgewertet, gehalten wird nur, wenn er wahr ist); Debuggen läuft bis zum ersten Haltepunkt, ohne Haltepunkte steht es in Zeile 1; die angehaltene Zeile ist gelb markiert, Variablen (lokal und global) stehen unten rechts anstelle der Problemliste, dazu die Schritt-Knöpfe; steht das Programm, wertet die Eingabezeile Ausdrücke aus (`? 2 * 21` → `= 42 (INTEGER)`) | F9, Umschalt+F9, F7, F8 weiter, F10 drüber, F11 hinein, Umschalt+F11 heraus, Umschalt+F5 stopp, Strg+E Ausdruck |
| Profil | Lauf unter `dhrt profile`; am Ende ein Fenster mit den Zeilen nach Zeit (Anzahl, ms, Anteil, Quelltext), Klick springt zur Zeile | Strg+Umschalt+Y |
| Ausführen | Starten mit laufender Ausgabe unten links, Eingabezeile für `INPUT`, Stoppen; Export als eigenständiges Programm (`dhrt --export`, nach `<name>_dist/` neben die Quelle, die Ausgabe des Exports läuft unten links mit) | F5, Umschalt+F5, Strg+F6 |
| Werkzeuge | Die Begleit-Editoren in Drachenhauch (SFX-Generator, Partikel-Editor, Tilemap-Editor, Sprite-Editor, Tracker, Form-Designer, Anim-FSM-Editor, Notenblatt) als eigene Programme; die Beispiele als Projekt öffnen | |
| Ansicht | Helles/dunkles Thema, Vollbild, Schrift größer/kleiner (10 bis 32 px, bleibt gemerkt); Zeilenumbruch (gilt für alle Reiter); Blöcke falten (die Blöcke kommen aus `CODE_SYMBOLS$` **und aus der Einrückung** -- alles, unter dem etwas tiefer Eingerücktes steht, also auch eine `FOR`-Schleife; ein Klick auf das Dreieck in der Nummernspalte tut dasselbe); geteilte Ansicht (zwei Reiter nebeneinander); Übersichtskarte am rechten Rand (Wort für Wort gezeichnet, heller Kasten für den sichtbaren Ausschnitt, Klick springt); geänderte Zeilen am Rand; die IDE startet maximiert | Alt+Enter, Alt+Z, F4, Strg+F4, Umschalt+F4, Alt+G |
| Hilfe | **Eingebaute Befehle nachschlagen** (Strg+F3): alle Namen, die die Vervollständigung kennt, mit Signatur und Beschreibung, filterbar, mit Knopf zum Einfügen. Handbuch im Fenster, **gesetzt statt roh**: Überschriften in drei Größen, Absätze umgebrochen, Aufzählungen mit Punkt, Code-Blöcke dicktengleich, Tabellen als Begriffsliste; der Knopf oben rechts schaltet auf den Quelltext. F1 schlägt das Wort unter der Schreibmarke in `docs/` nach und öffnet das Dokument mit den meisten Fundstellen in Codeschrift; Klappliste aller Dokumente, Suche im Dokument; Tastenkürzel-Übersicht | F1, Strg+F1 |

Einstellungen und Sitzung liegen in EINER JSON-Datei im Nutzerprofil
(`%APPDATA%\Drachenhauch\ide.json`, sonst `~/.config/Drachenhauch/ide.json`):
`zuletzt`, `sitzung`, `aktiv`, `hell`, `schrift`, `umbruch`, `karte`,
`git_rand`, `regex`, `vorschlag`, `autosichern` und `projekte` (Sitzung je Ordner, die zwölf letzten — wer an zwei Sachen
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
- **Einrückung beim Zeilenumbruch** (`GUI_TEXTAREA_SET(ta, "auto_einzug", 1)`
  + `GUI_TEXTAREA_INDENT_WORDS`): die Laufzeit übernimmt die Einrückung der
  laufenden Zeile -- das ist sprachfrei; welche Wörter eine Stufe mehr oder
  weniger bedeuten, sagt die IDE. Dasselbe Prinzip wie bei der Faltung.
- **Farbfelder** (`GUI_TEXTAREA_SWATCHES` + `GUI_TEXTAREA_SWATCH_CLICKED`):
  ein kleines Quadrat zu einem Stück Text, am **Ende** seiner Zeile — direkt
  dahinter läge es auf dem nächsten Zeichen. Welche Stelle im Text eine
  Farbe meint, weiß wieder nur der Aufrufer.

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
`marken <anzahl>`, `blame <zeilen>`, `hbansicht gesetzt|quelltext`,
`git diff|log <zeilen>`, `git rand <zeilen>`, `lesezeichen liste <anzahl>`,
`wieder auf <pfad>`, `spur <pfad>`, `peek <zeile>`, `befehle <anzahl>`,
`befehl eingefuegt <name>`, `einstellungen auf`, `autosichern <s>`,
`auto gesichert`, `farbfeld <stelle>`, `farbe <wert>`, `ende`. So sieht `tests/test_ide.py`, was sie
getan hat; Tasten kommen über `AUTOMATION_PLAY` herein (F5 startet, F7
prüft). Die Bausteine einzeln prüft `tests/test_ide_bausteine.py`.

## Was noch fehlt

Die Liste aus Stand 4 ist abgearbeitet: gerenderte Markdown-Ansicht,
Codefaltung, Übersichtskarte, geteilter Editor, mehrere Schreibmarken,
Schnipsel, Signaturhilfe, Umbenennen, Git-Blame, Sitzung je Projekt,
Zeilenumbruch und Symbole in den Menüs sind alle da. Zwei Bausteine der
Laufzeit kamen dafür dazu (Faltung und mehrere Schreibmarken im
Textbereich), dazu `CODE_RENAME$` und `GUI_WINDOW_SHOWN`.

Auch die Liste aus Stand 5 ist abgearbeitet: Faltung nach Einrückung,
Git-Diff und -Verlauf, Suche mit regulären Ausdrücken, Lesezeichen über
Dateien hinweg und eine Übersichtskarte, die Wörter zeigt. Dazu kamen die
Einrückung beim Zeilenumbruch, die Vorschlagsliste beim Tippen, die
Hervorhebung von Fundstellen und Klammernpaar und das Wiederöffnen eines
geschlossenen Reiters.

Auch die Liste aus Stand 6 ist abgearbeitet: Einstellungsdialog,
Befehlsverzeichnis, Peek, Symbolspur, automatisches Sichern, Farbfelder
mit Farbwähler und der vorgefüllte Schnipsel-Filter.

Offen bleibt gegen die Qt-IDE: ein Schnipsel, der sich durch Tippen
seines Namens und Tabulator allein aufklappt (der Tabulator gehört im
Code-Feld dem Einrücken), Mehrfach-Auswahl in der Dateiliste, ein
Verzeichnis der eigenen Symbole über alle Dateien des Projekts, und die
Vorschau einer Definition, die in einer ANDEREN Datei steht (heute
findet `CODE_DEFINITION` nur, was im selben Text steht). Ein
Installer ohne Python gibt es seit Stand 3:
`installer/Drachenhauch-IDE.iss` packt `dhrt.exe`, `ide/`, `docs/` und die
Beispiele -- 33 MB statt 92; die Qt-IDE bleibt daneben installierbar, bis
diese hier gleichzieht. Die Liste steht in
[entwurf-python-abbau.md](entwurf-python-abbau.md), Abschnitt C.
