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

## Die Stände im Überblick

Gewachsen ist sie in Ständen, einer je Runde. Die Tabelle sagt, was in
welchem dazukam — für die Frage „wann kam eigentlich …?"; was sie **heute**
kann, steht darunter vollständig, ohne Stände.

| Stand | Was dazukam |
|---|---|
| 1 | Reiter, Projektbaum, Einfärbung, Fehlerliste, Hilfe zum Wort, Vervollständigung, Suchen/Ersetzen, Starten mit Ausgabe |
| 2 | Debugger, Profil, Suche im Projekt, Befehlspalette, Marken in der Nummernspalte, echte Schriften |
| 3 | Handbuch im Fenster, Drucken und PDF, Werkzeuge-Menü, Ausdrücke im Debugger, Installer ohne Python |
| 4 | Willkommensseite, Sitzung und Einstellungen in einer Datei, Datei im Projekt öffnen, die Handgriffe auf Zeilen, Lesezeichen, Gliederung, bedingte Haltepunkte, Export |
| 5 | Falten, mehrere Schreibmarken, geteilte Ansicht, Übersichtskarte, git blame, Umbenennen, Schnipsel, Signaturhilfe, Sitzung je Projekt |
| 6 | Einrückung beim Zeilenumbruch, Fundstellen und Klammernpaar farbig, Vorschlagsliste beim Tippen, git diff/log, Suche mit regulärem Ausdruck |
| 7 | Einstellungsdialog, Befehlsverzeichnis, Definition hier zeigen, Symbolspur, automatisches Sichern, Farbfelder mit Farbwähler |
| 8 | Schnipsel per Kürzel und Tabulator, Symbolverzeichnis über das Projekt, Definition über Dateigrenzen, Mehrfachauswahl im Projektbaum |
| 9 | Einrückungslinien, Ausgabe durchsuchen, im ganzen Projekt ersetzen, andere Reiter schließen, Über-Kasten |
| 10 | Spaltenauswahl, Zeilen-Werkzeuge, im ganzen Projekt umbenennen, zwei Dateien vergleichen |
| 11 | Werkzeugleiste mit Sinnbildern, Kacheln mit Vorschaubildern der Beispiele |
| 12 | Auswahl erweitern/verkleinern, Auswahl in ein Unterprogramm herauslösen, zwei Reiter nebeneinander vergleichen |
| 13 | Vorschau vor dem Umbau, Parameter umsortieren samt Aufrufen, Gerüst beim Tippen (`END IF` kommt mit) |
| 14 | Parameter umsortieren **im ganzen Projekt**, Aufrufer eines Unterprogramms auflisten, Signatur-Platzhalter bei der Vervollständigung |
| 15 | Einwand, wenn der neue Name schon vergeben ist; Parameter hinzufügen und entfernen; die Aufrufer als **Baum** |
| 16 | Unterprogramm in eine andere Datei verschieben (samt IMPORT); Umbauten auch für Methoden; einen geschriebenen Umbau in einem Zug zurücknehmen |
| 17 | Klassen verschieben; Verschieben legt die Zieldatei an; Einwand, wenn eine Umbenennung eine Überschreibung zerreißt; Pfeiltasten im Wähler |
| 18 | Mattere Oberfläche (der Verlauf klingt auf großen Flächen aus); das Verschobene nimmt mit, was es braucht; mehrfach zurücknehmen; Aufrufer kennen FUNCREF |
| 19 | In der Vorschau einzelne Dateien abwählen; Einwand, wenn der Name im Ziel schon steht; Konstanten und globale Variablen verschieben |
| 20 | In der Vorschau einzelne **Blöcke** auslassen; Umbenennen zieht die Handler in `.dhform` mit; die Aufrufer zeigen die gemessenen Durchläufe |
| 21 | Den letzten Umbau **nachbessern**; die Umbauten sehen **Unterordner**; eine gebrauchte Konstante zählt für den `IMPORT` |
| 22 | Der Projektbaum zeigt die Unterordner; ein Umbau lässt sich **vorab prüfen**; eine neu angelegte Datei bekommt einen Reiter |
| 23 | Der Baum zeigt auch Formulare, Daten und Bilder; ein laufender Umbau lässt sich **abbrechen**; das Prüfen nennt die **Fehler** |
| 24 | Der Projektbaum ist ein **Widget der Laufzeit** (`GUI_FILETREE`): er liest den Ordner selbst, zeigt eine neue Datei von selbst und klappt auf Klick auf; dazu Häkchen im Baum und **Reiter im Fenster** |
| 25 | Das Handbuch ist **gesetzt vom Widget** (`GUI_RICHTEXT`, mit anklickbaren Verweisen); Häkchen im Baum schränken das Projekt ein; ohne Vorschau wird trotzdem geprüft; auch Verschieben und Parameter sammeln in Schritten |
| 26 | Die **Werkzeugleiste ist ein Widget** der Laufzeit (`GUI_TOOLBAR` mit Einträgen, gezeichnete Sinnbilder in Leistengröße, Trenner, Lücke, kippbare Knöpfe); Menüs und Leiste tragen dieselben Sinnbilder, die handgemalten 16-Punkt-Bilder sind weg |

## Was sie heute kann (Stand 26, 13.09.2026)

| Bereich | Was geht | Kürzel |
|---|---|---|
| Start | Willkommensseite, solange kein Reiter offen ist: Neu, Öffnen, Beispiele, die zuletzt geöffneten Dateien (Doppelklick), die wichtigsten Kürzel. Die Sitzung kommt beim nächsten Start wieder (offene Dateien, aktiver Reiter), ebenso Thema und Schriftgröße | |
| Dateien | Neu, Öffnen, Datei im Projekt öffnen (Wähler mit unscharfem Filter: `spred` findet `189_sprite_editor.dh`; dazu die zuletzt geöffneten), Zuletzt geöffnet (Untermenü), Sichern, Sichern unter, Reiter schließen und wieder öffnen (Strg+Umschalt+T); bis zu 12 Reiter; Rückfrage bei ungesicherten Änderungen; Listing drucken über einen Druckdialog (Drucker, Kopien) oder als PDF neben die Quelle (Courier 9 pt, 66 Zeilen je Seite, Zeilennummern, Kopfzeile mit Seitenzahl) | Strg+N, Strg+O, Strg+Umschalt+O, Strg+S, Strg+W, Strg+Q, Strg+P |
| Bearbeiten | Suchen, Weitersuchen, Ersetzen (alle Treffer), Gehe zu Zeile, Suche im Projekt (alle `.dh` im Projektordner, Treffer unten rechts, Doppelklick öffnet), wahlweise mit **regulärem Ausdruck** (ein Schalter für Suchen, Ersetzen und Projektsuche), TODO/FIXME-Liste (dieselbe Suche), Befehlspalette (tippen filtert, **Pfeile** wählen, Enter führt aus) | Strg+F, F3, Strg+H, Strg+G, Strg+Umschalt+F, Strg+Umschalt+M, Strg+Umschalt+P |
| Git | Wer hat das geschrieben (`git blame`, Datum und Person je Zeile), was habe ich geändert (`git diff` farbig im Fenster), Verlauf dieser Datei (`git log`); die geänderten Zeilen tragen eine Marke am Rand (nach dem Sichern neu gefragt, nicht je Bild) | Strg+Umschalt+B, Strg+Umschalt+D |
| Schreiben | Umbenennen eines Symbols über die ganze Datei (`CODE_RENAME$`: ganze Wörter, Kommentare und Zeichenketten bleiben; ein krummer Name ändert nichts) — und die **Formulare ziehen mit**: eine `.dhform` nennt ihre Rückrufe beim Namen, und wer das Unterprogramm umbenennt und die Datei stehen lässt, hat einen Knopf, der nichts mehr tut, Schnipsel einfügen (13 Gerüste, `\|` sagt wohin die Marke gehört, die Einrückung der Zeile wird übernommen), eine Marke auf jede Fundstelle des Wortes — danach ändert ein Tippen alle; Alt+Klick legt eine Marke dazu, ESC räumt sie weg. Signaturhilfe: steht die Marke in einer Argumentliste, zeigt die Statuszeile die Signatur des Aufrufs und die Nummer des Arguments | Umschalt+F6, Strg+J, Strg+Umschalt+L |
| Reiter | Jeder Reiter hat ein **Kreuz**, die mittlere Maustaste schließt ihn auch. Trägt er ungesicherte Änderungen, wird gefragt (Sichern / Verwerfen / Abbrechen) — das galt vorher nur beim Beenden, Strg+W nahm sie wortlos mit. **Andere Reiter schließen** lässt nur den vorderen stehen | Strg+W, Strg+Umschalt+W |
| Dateiliste | **Häkchen schränken das Projekt ein**: ist etwas angehakt, arbeitet jeder Umbau (und die Projektsuche) nur auf dieser Menge — die Beschriftung über dem Baum sagt es, `Datei → Haken im Projektbaum entfernen` räumt sie ab. Der Projektbaum ist seit Stand 24 ein `GUI_FILETREE`: er liest den Ordner selbst und nur die Äste, die offen sind. Ordner stehen oben und klappen auf Klick auf; eine Datei, die ein anderes Programm anlegt, steht nach spätestens zwei Sekunden da. Strg+Klick sammelt, Umschalt+Klick spannt einen Bereich; **Gewaehlte Dateien oeffnen** macht aus allen Reiter. Beim Sammeln geht noch nichts auf — sonst käme mit jedem Klick ein Reiter dazu, den niemand wollte | Strg+Umschalt+E |
| Einstellungen | Strg+U zeigt alle Schalter an einer Stelle (Thema, Umbruch, Karte, geänderte Zeilen, reguläre Ausdrücke, Vorschlagsliste, Einrückungslinien, Gerüst beim Tippen, Vorschau vor dem Umbau, Schriftgröße, automatisches Sichern). Sie wirken **sofort**, ohne Übernehmen — ein Thema, das man erst nach dem Schließen sieht, wählt man blind; im Menü stehen sie weiter dort, wo man sie sucht | Strg+U |
| Zeilen | Alles auf ganzen Zeilen, der Auswahl oder der Zeile der Marke, jeder Handgriff ein eigener Undo-Schritt: Kommentar umschalten, Zeile duplizieren, Zeile löschen, Zeilen nach oben/unten, Ein-/Ausrücken (Tab gehört dem Feld selbst), Dokument formatieren (`CODE_FORMAT$`, derselbe Formatierer wie `dhrt fmt`; bei einem Syntaxfehler bleibt alles stehen), Schnipsel einfügen (Strg+J füllt den Filter mit dem Wortanfang links der Marke: wer `for` getippt hat, hat die FOR-Schleife vor sich; **oder einfach `for` tippen und Tabulator** — jeder Schnipsel hat ein kurzes Wort, das ihn aufklappt), automatisch sichern nach einstellbarer Ruhe. Eine neue Zeile übernimmt die **Einrückung** der alten und rückt hinter `SUB`, `FOR`, `THEN` und den anderen Blockwörtern eine Stufe weiter ein; ein `END` oder `NEXT` allein in einer Zeile rückt sie zurück. Und das **Gerüst wächst mit**: hinter `IF x > 0 THEN` setzt der Zeilenumbruch das `END IF` gleich mit darunter, die Marke bleibt dazwischen (`CASE`, `ELSE` und `ELSEIF` nicht — sie rücken ein, schließen aber nichts; abschaltbar). Lesezeichen (blaue Marke) setzen und anspringen — vorwärts und rückwärts **über Dateien hinweg**, dazu die Liste aller | Strg+K, Strg+D, Strg+Umschalt+K, Alt+Hoch/Runter, Alt+Rechts/Links, Umschalt+Alt+F, Strg+F2, F2, Umschalt+F2, Alt+F2 |
| Sprache | Einfärbung des sichtbaren Ausschnitts, dazu jede Fundstelle des Wortes unter der Marke, das zusammengehörende Klammernpaar und ein **Farbfeld** neben jedem `&H`-Literal (ein Klick darauf öffnet den Farbwähler und schreibt die neue Farbe an die Stelle zurück); eine **Symbolspur** über dem Code sagt, in welcher Klasse und welchem Unterprogramm man steht; **Definition hier zeigen** (Alt+F12) blendet zehn Zeilen um die Definition ein, ohne die Stelle zu verlassen; Hilfe zum Wort (Statuszeile), Vervollständigung — sie geht beim Tippen ab drei Zeichen von selbst auf, der Fokus bleibt dabei im Code-Feld, Strg+Leer holt sie herein, und hat der Name eine **Signatur**, kommt sie als Gerüst mit: aus `CIRC` wird `CIRCLE(x, y, r)` mit markiertem `x`, der **Tabulator** geht zum nächsten Argument; Zur Definition, Gliederung links unten (SUB/FUNCTION/CLASS mit Methoden, Doppelklick springt) | Strg+Leer, F12 |
| Prüfen | Fehlerliste unten rechts, 0,6 s nach der letzten Änderung von selbst; Klick springt zur Zeile; Fehlerzeilen tragen eine orange Marke | Umschalt+F7 |
| Debugger | Haltepunkte (rote Marke) an der Zeile der Schreibmarke, bedingte Haltepunkte (violett; `i = 3`, `hp < 10` — der Ausdruck wird im Programm ausgewertet, gehalten wird nur, wenn er wahr ist); Debuggen läuft bis zum ersten Haltepunkt, ohne Haltepunkte steht es in Zeile 1; die angehaltene Zeile ist gelb markiert, Variablen (lokal und global) stehen unten rechts anstelle der Problemliste, dazu die Schritt-Knöpfe; steht das Programm, wertet die Eingabezeile Ausdrücke aus (`? 2 * 21` → `= 42 (INTEGER)`) | F9, Umschalt+F9, F7, F8 weiter, F10 drüber, F11 hinein, Umschalt+F11 heraus, Umschalt+F5 stopp, Strg+E Ausdruck |
| Profil | Lauf unter `dhrt profile`; am Ende ein Fenster mit den Zeilen nach Zeit (Anzahl, ms, Anteil, Quelltext), Klick springt zur Zeile | Strg+Umschalt+Y |
| Ausführen | Starten mit laufender Ausgabe unten links, Eingabezeile für `INPUT`, Stoppen; Export als eigenständiges Programm (`dhrt --export`, nach `<name>_dist/` neben die Quelle, die Ausgabe des Exports läuft unten links mit) | F5, Umschalt+F5, Strg+F6 |
| Werkzeuge | Die Begleit-Editoren in Drachenhauch (SFX-Generator, Partikel-Editor, Tilemap-Editor, Sprite-Editor, Tracker, Form-Designer, Anim-FSM-Editor, Notenblatt) als eigene Programme; die Beispiele als Projekt öffnen | |
| Ansicht | Helles/dunkles Thema, Vollbild, Schrift größer/kleiner (10 bis 32 px, bleibt gemerkt); Zeilenumbruch (gilt für alle Reiter); Blöcke falten (die Blöcke kommen aus `CODE_SYMBOLS$` **und aus der Einrückung** -- alles, unter dem etwas tiefer Eingerücktes steht, also auch eine `FOR`-Schleife; ein Klick auf das Dreieck in der Nummernspalte tut dasselbe); geteilte Ansicht (zwei Reiter nebeneinander); Übersichtskarte am rechten Rand (Wort für Wort gezeichnet, heller Kasten für den sichtbaren Ausschnitt, Klick springt); geänderte Zeilen am Rand; die IDE startet maximiert | Alt+Enter, Alt+Z, F4, Strg+F4, Umschalt+F4, Alt+G |
| Werkzeugleiste | Unter dem Menü eine Reihe **Sinnbilder** für das, was man ständig braucht: neu, öffnen, sichern, starten, stoppen, debuggen, prüfen, suchen, Zeilenumbruch (kippbar, zieht mit Menü und Alt+Z mit), rechts Handbuch und Einstellungen. Die Leiste ist seit Stand 26 ein Widget der Laufzeit (`GUI_TOOLBAR` mit Einträgen), die Sinnbilder sind dieselben wie in den Menüs. Jeder Knopf ruft denselben Befehl wie sein Menüpunkt und nennt im Tooltip sein Kürzel; abschaltbar unter Ansicht | — |
| Kacheln | Auf der Willkommensseite acht wichtige Beispiele **mit einem Bild davon**. Die Bilder entstehen, indem das Beispiel wirklich läuft (`dhrt bild`), eines nach dem anderen im Hintergrund und unsichtbar; danach liegen sie neben der Sitzung und sind sofort da. Ein Klick öffnet das Beispiel | — |
| Umbauen | **Auswahl in ein Unterprogramm herauslösen** (Strg+Umschalt+R): die gewählten Zeilen wandern in ein neues `SUB` am Dateiende, an ihrer Stelle steht der Aufruf. Was als Parameter mitmuss, steht nicht im Raten — globale Namen sieht ein SUB ohnehin, also sind es genau die **lokalen** des umgebenden Unterprogramms, die in den Zeilen vorkommen; wer darin auch zugewiesen wird, geht **BYREF**. Danach zählt die IDE die Fehler nach und sagt es, wenn die Auswahl nicht ausgewogen war | Strg+Umschalt+R |
| Umbauen | **Parameter umsortieren, hinzufügen, entfernen** (Strg+Umschalt+U): die Parameter des Unterprogramms unter der Marke umstellen, einen neuen aufnehmen (`hp AS INTEGER = 0` — der Teil hinter dem `=` kommt an jede Aufrufstelle) oder einen wegnehmen — und **die Aufrufe ziehen mit**, in **allen** `.dh` des Projektordners. Die Reihenfolge der Argumente ist die Bedeutung; ein vergessener Aufruf übergibt stumm das Falsche. Ein Aufruf, der eine andere Zahl von Argumenten übergibt (weggelassener Vorgabewert) oder sie **benennt**, wird übergangen und gezählt — dort heißt die Reihenfolge etwas anderes | Strg+Umschalt+U |
| Umbauen | Ein Umbau über das ganze Projekt sammelt **in Schritten**, je Bild ein Häppchen: die Statuszeile zählt mit, **ESC bricht ab**. In einem Bild erledigt, stünde die IDE so lange — bei vierhundert Dateien lange genug, dass man sie für hängend hält, und abbrechen ließe sich nichts, was gar nicht erst zum Zeichnen kommt | ESC |
| Umbauen | **In der Vorschau abwählen**: links stehen die betroffenen Dateien mit Kästchen — was man abwählt, bleibt stehen. Ein Umbau über zwölf Dateien ist selten in allen zwölf gemeint, und „alles oder nichts" hieße dann: von Hand nacharbeiten. Feiner geht es mit **Block auslassen**: die Marke auf eine geänderte Zeile, und der zusammenhängende Block bleibt, wie er war (er steht dann grau mit `~` da). Dahinter liegt keine Textausgabe mehr, sondern eine Folge von Schritten — der Text der Datei entsteht aus denselben Schritten, die man sieht | — |
| Umbauen | **Vorschau vor dem Umbau**: Umbenennen, Im-Projekt-Umbenennen, Im-Projekt-Ersetzen, Herauslösen und die Parameter-Umbauten zeigen erst den Unterschied und fragen (Enter übernimmt, ESC verwirft). Abschaltbar unter Strg+U — **außer wenn es einen Einwand gibt**: gibt es den neuen Namen im Projekt schon, oder zerreißt die Umbenennung eine **Überschreibung** (dieselbe Methode steht in der Oberklasse oder in einer erbenden Klasse — gerufen würde von da an die andere Fassung, ohne Fehlermeldung), geht die Vorschau auf und sagt es oben. Wer einen Einwand nur in die Statuszeile schreibt, hat ihn nicht vorgebracht | — |
| Umbauen | **Verschieben** (Strg+Umschalt+V): ein Unterprogramm oder eine ganze **Klasse** wandert in eine andere Datei des Projekts, samt den Kommentarzeilen darüber — und jede Datei, die es benutzt, bekommt den `IMPORT` der Zieldatei dazu. In Drachenhauch fügt `IMPORT` den Text ein; ohne diesen Teil wäre das Verschieben ein Umbau, der die Übersetzung kaputt macht. Steht die Marke in einer Klasse, ist die **Klasse** gemeint — eine Methode allein wäre ohne sie kein Unterprogramm mehr. Der erste Eintrag im Wähler legt eine **neue Datei** an. Und es geht in beide Richtungen: braucht das Verschobene etwas, das zurückbleibt, importiert die **Zieldatei** die Quelle. Steht die Marke auf einer `CONST`- oder `DIM`-Zeile, ist **die Zeile** die Einheit. Hat das Ziel den Namen schon, sagt die Vorschau es — zwei gleichen Namens in einer Datei sind kein Übersetzungsfehler, der zweite gewinnt einfach | Strg+Umschalt+V |
| Umbauen | **Prüfen** in der Vorschau: `dhrt --check` läuft über die Texte, die geschrieben **würden**, mit der Blockauswahl von jetzt. Wer einen Block auslässt und damit etwas kaputt macht, sieht es vor dem Schreiben — und **welchen** Fehler, nicht nur wie viele: die Meldungen stehen oben im Unterschied. Nur auf Verlangen — ein Umbau über zwölf Dateien bräuchte sonst zwölf Übersetzungsläufe, ehe man den Unterschied überhaupt zu sehen bekommt | — |
| Umbauen | **Letzten Umbau nachbessern** (Strg+Umschalt+G): dieselben Schritte noch einmal, mit derselben Auswahl. Ein ausgelassener Block ließe sich sonst nur zurückholen, indem man den ganzen Umbau zurücknimmt und ihn von vorn macht; gerechnet wird aus den Schritten, nicht aus dem, was gerade in der Datei steht | Strg+Umschalt+G |
| Umbauen | **Umbau zurücknehmen** (Strg+Umschalt+Z): Strg+Z im Code-Feld nimmt nur den Reiter zurück, in dem man steht — ein Umbau über sechs Dateien wäre damit sechsmal zurückzunehmen, und zwar in sechs Reitern, die man dafür erst öffnen muss. Die letzten zehn liegen auf einem **Stapel**, jeder Druck nimmt einen weiter zurück | Strg+Umschalt+Z |
| Aufrufe | **Wer ruft das auf?** (Umschalt+F12): die Gegenrichtung zu F12 — alle Stellen, an denen das Unterprogramm unter der Marke gerufen wird, über alle Dateien des Projekts. Als **Baum**: unter jedem Aufruf stehen die Aufrufer des Unterprogramms, in dem er steht, drei Ebenen tief; ein Klick springt hin. Gezählt wird auch, wo der Name **ohne Klammern** weitergegeben wird (`f = malen`) — dort wird entschieden, dass er später läuft. Gab es einen Profillauf, steht an jeder Stelle, **wie oft** sie gelaufen ist; ohne Lauf steht dort nichts, eine Null wäre eine Aussage, die niemand gemessen hat. Die Definition steht nicht dabei, sie ist kein Aufruf | Umschalt+F12 |
| Auswahl | **Erweitern** nimmt die nächstgrößere Klammer: Wort, Zeile, Block, Elternblock, ganze Datei. **Verkleinern** geht denselben Weg zurück — ein Stapel merkt sich jede Stufe, statt sie neu zu erraten | Strg+Umschalt+Hoch / Runter |
| Umbenennen | **Im ganzen Projekt umbenennen** (Strg+Umschalt+F6): der Name unter der Marke, in allen `.dh` des Projektordners. `CODE_RENAME$` lässt Kommentare und Zeichenketten aus, wie beim Umbenennen in einer Datei | Strg+Umschalt+F6 |
| Vergleichen | **Zwei Reiter nebeneinander**: die geteilte Ansicht geht an, und in beiden Feldern bekommt jede abweichende Zeile eine Marke. Verglichen wird ohne git über die längste gemeinsame Teilfolge auf Zeilen — die Reiter müssen dafür nicht gesichert sein, und genau während man tippt will man es wissen | — |
| Vergleichen | **Mit einer anderen Datei vergleichen**: `git diff --no-index` im selben Fenster und derselben Färbung wie git diff. Gemeint ist die im Projektbaum gewählte Datei, wenn es eine andere ist — nur sonst fragt der Datei-Dialog | — |
| Zeilen-Werkzeuge | **Sortieren**, **Doppelte entfernen**, **Leerraum am Zeilenende entfernen** — auf der Auswahl, ohne Auswahl auf der ganzen Datei | — |
| Ersetzen | **Im ganzen Projekt ersetzen** (Strg+Umschalt+H): zählt erst die Stellen und fragt, dann schreibt es alle `.dh` des Projektordners. Ein Reiter mit unge**sicherten** Änderungen bricht es ab — die würden die Datei beim nächsten Sichern wieder überschreiben | Strg+Umschalt+H |
| Ausgabe | **Ausgabe durchsuchen** (Strg+Umschalt+A): die Zeilen des laufenden Programms. Derselbe Text noch einmal heißt weitersuchen | Strg+Umschalt+A |
| Symbole | **Symbol im Projekt suchen** (Strg+Umschalt+S): alle SUB, FUNCTION, CLASS und Methoden über ALLE `.dh` des Projektordners, filterbar; der Filter ist mit dem Wort unter der Marke vorbelegt. Derselbe Index trägt **Zur Definition** und **Definition hier zeigen** über Dateigrenzen: was `CODE_DEFINITION` im eigenen Text nicht findet, steht vielleicht nebenan | Strg+Umschalt+S |
| Hilfe | **Eingebaute Befehle nachschlagen** (Strg+F3): alle Namen, die die Vervollständigung kennt, mit Signatur und Beschreibung, filterbar, mit Knopf zum Einfügen. Handbuch im Fenster, **gesetzt statt roh** — seit Stand 25 vom Widget der Laufzeit (`GUI_RICHTEXT`): Überschriften in drei Größen, Absätze umgebrochen, Aufzählungen mit Punkt, Code-Blöcke dicktengleich, Tabellen mit **Spalten**, fett und kursiv, und **anklickbare Verweise** (ein Verweis auf ein anderes Dokument öffnet es, eine Sprungmarke rollt im selben); der Knopf oben rechts schaltet auf den Quelltext. F1 schlägt das Wort unter der Schreibmarke in `docs/` nach und öffnet das Dokument mit den meisten Fundstellen in Codeschrift; Klappliste aller Dokumente, Suche im Dokument; Tastenkürzel-Übersicht | F1, Strg+F1 |

Einstellungen und Sitzung liegen in EINER JSON-Datei im Nutzerprofil
(`%APPDATA%\Drachenhauch\ide.json`, sonst `~/.config/Drachenhauch/ide.json`):
`zuletzt`, `sitzung`, `aktiv`, `hell`, `schrift`, `umbruch`, `karte`,
`git_rand`, `regex`, `vorschlag`, `geruest`, `umbau_vorschau`, `autosichern` und `projekte` (Sitzung je Ordner, die zwölf letzten — wer an zwei Sachen
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
- **Spaltenauswahl** (`GUI_TEXTAREA_SELECT_COLUMNS`, mit der Maus **Alt
  gedrückt halten und ziehen**): ein Rechteck statt eines Laufs. Jede Zeile
  bekommt ihre eigene Marke samt Auswahl — die Maschinerie dafür lag seit
  den mehreren Schreibmarken schon da. Eine zu kurze Zeile bekommt ihre
  Marke am Ende, statt still herauszufallen.
- **Einrückungslinien** (`GUI_TEXTAREA_SET(ta, "einzugslinien", 1)`): ein
  feiner Strich je Stufe, unter dem Text. Die Breite einer Stufe wird an
  `tabbreite` Leerzeichen gemessen, und eine leere Zeile nimmt die kleinere
  Tiefe ihrer Nachbarn — sonst risse die Linie in jedem Absatz auf.
- **Abkürzungen am Tabulator** (`GUI_TEXTAREA_ABBREV` +
  `GUI_TEXTAREA_ABBREV_HIT`): steht eines der genannten Wörter links der
  Marke, meldet der Tabulator es, statt einzurücken. Was an seine Stelle
  kommt, setzt die IDE — die Laufzeit kennt keine Schnipsel.
- **Mehrfachauswahl im Baum** (`GUI_TREE_SET "mehrfachauswahl"` +
  `GUI_TREE_SEL_COUNT/SEL_NODE/IS_SELECTED/SELECT/CLEAR_SELECTION`):
  dieselben Abfragen wie bei Liste und Tabelle. Der Baum war die letzte
  Auswahl-Art ohne sie.
- **Farbfelder** (`GUI_TEXTAREA_SWATCHES` + `GUI_TEXTAREA_SWATCH_CLICKED`):
  ein kleines Quadrat zu einem Stück Text, am **Ende** seiner Zeile — direkt
  dahinter läge es auf dem nächsten Zeichen. Welche Stelle im Text eine
  Farbe meint, weiß wieder nur der Aufrufer.
- **Das mitwachsende Gerüst** (`GUI_TEXTAREA_CLOSE_WORDS`): öffnet die Zeile
  einen Block, setzt der Zeilenumbruch die schließende Zeile gleich mit
  darunter. Die Paare (`IF` → `END IF`) stehen in der IDE — die Laufzeit
  kennt keine Sprache. Was schon geschlossen ist, bekommt keinen zweiten
  Abschluss.

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

Jeder **Umbau** geht durch eine Stelle: die neuen Texte vormerken, dann
`umbauStarten`. Ist die Vorschau an, zeigt sie den Unterschied und fragt;
ist sie aus, wird gleich geschrieben — ein Weg, nicht zwei, die
auseinanderlaufen können. Geschrieben wird auf zweierlei Art: in einen
Reiter über Auswahl + `GUI_TEXTAREA_INSERT` (also **ein** Undo-Schritt;
`GUI_SET_TEXT` leerte den Verlauf) oder auf die Platte, wobei ein offener
Reiter mitgezogen wird. Den Unterschied rechnet dieselbe längste
gemeinsame Teilfolge wie das Nebeneinander, gefärbt wie `git diff`.

Das **Umsortieren der Parameter** arbeitet auf dem ganzen Text, nicht Zeile
für Zeile: ein Aufruf darf über mehrere Zeilen gehen, und die Argumente
werden als Textstücke vertauscht — was zwischen ihnen steht, bleibt stehen.
Die Definition braucht keinen Sonderfall: `SUB name(` sieht für den Sucher
aus wie ein Aufruf, und ihre „Argumente" sind die Parameter. Zeichenketten
und Kommentare überspringt er, ein Komma in `PRINT "a, b"` ist keine
Argumentgrenze.

## Prüfen ohne hinzusehen

Setzt man `DH_IDE_LOG=<datei>`, schreibt die IDE ihre Ereignisse zeilenweise
mit: `bereit`, `geoeffnet <pfad>`, `geprueft <anzahl>`, `gestartet <pfad>`,
`beendet <code>`, `gesichert`, `geschlossen <nummer>`, `projekt <ordner>`, `haltepunkt <zeile> an|aus`,
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
`wieder auf <pfad>`, `spur <pfad>`, `peek <zeile> [<datei>]`,
`symbolindex <anzahl>`, `symbol <datei> <zeile>`, `baum offen <anzahl>`,
`befehle <anzahl>`, `linien an|aus`, `andere zu <anzahl>`,
`ausgabe treffer <zeile>`, `marke naechste <anzahl>`,
`projekt ersetzt <anzahl>` (-1 = abgebrochen, ein Reiter war ungesichert),
`ueber <fassung>`, `marken enden <anzahl>`, `zeilen <art> <anzahl>`,
`projekt umbenannt <anzahl>` (-1 = abgebrochen), `vergleich <zeilen>`,
`leiste an|aus`, `kachel <datei>`, `vorschau <datei>`,
`vorschau fertig <nummer>`, `vorschau neu`, `erweitern <was>`,
`verkleinern <tiefe>`, `herausgeloest <name> <parameter> <mehr fehler>`,
`reiter vergleich <links> <rechts>`,
`befehl eingefuegt <name>`, `einstellungen auf`, `autosichern <s>`,
`geruest an|aus`, `umbau schalter an|aus`,
`umbau vorschau <dateien> <weniger> <mehr>`, `umbau verworfen`,
`parameter <anzahl>` (0 = keins gefunden),
`parameter umgestellt <stellen> <uebergangen>` (-1 = abgebrochen, ein
anderer Reiter war ungesichert), `aufrufer <anzahl>`,
`platzhalter <nummer> von <anzahl>`, `umbau einwand`,
`parameter neu <deklaration>`, `parameter weg <nummer>`,
`aufrufer baum <knoten>`, `aufrufer sprung <zeile>`,
`verschieben <name> <zeilen> <imports>` (0 = ging nicht),
`umbau zurueck <dateien>`, `umbau abgewaehlt alles`,
`umbau block <nummer> aus|an` (-1 = keine geänderte Zeile),
`umbau nachbessern <dateien>` (0 = keiner da), `umbau nachgebessert <dateien>`,
`umbau geprueft <fehler>`, `umbau neue datei <anzahl>`,
`umbau abgebrochen`, `extern <datei>`,
`aufrufer gemessen <anzahl>`,
`auto gesichert`, `farbfeld <stelle>`, `farbe <wert>`, `leistenknopf <befehl>`, `ende`. So sieht `tests/test_ide.py`, was sie
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

Auch die Liste aus Stand 7 ist abgearbeitet: Schnipsel per Tippen und
Tabulator, Mehrfach-Auswahl in der Dateiliste, ein Symbolverzeichnis über
das ganze Projekt, und Definition und Vorschau über Dateigrenzen hinweg.

Auch die Liste aus Stand 12 ist abgearbeitet: die Vorschau vor dem Umbau
(statt hinterher die Fehler zu zählen), das Umsortieren der Parameter samt
Aufrufen und das Schlüsselwort-Gerüst, das beim Tippen mitwächst.

Auch die Liste aus Stand 13 ist abgearbeitet: der Umbau über
Dateigrenzen, die Liste der Aufrufer und die Signatur-Platzhalter.

Auch die Liste aus Stand 14 ist abgearbeitet: der Einwand beim Umbenennen
auf einen vergebenen Namen, das Hinzufügen und Entfernen von Parametern,
und die Aufrufer als Baum.

Auch die Liste aus Stand 15 ist abgearbeitet: das Verschieben in eine
andere Datei samt IMPORT, die Umbauten für Methoden (die Marke darf dafür
auch **in** der Argumentliste stehen) und das Zurücknehmen in einem Zug.

Auch die Liste aus Stand 16 ist abgearbeitet: Klassen verschieben, das
Anlegen der Zieldatei und der Einwand bei einer zerrissenen Überschreibung.

Auch die Liste aus Stand 17 ist abgearbeitet: das Verschobene nimmt mit,
was es selbst braucht, der Umbau lässt sich mehrfach zurücknehmen, und die
Aufrufer-Liste kennt FUNCREF. Dazu kam eine **mattere Oberfläche**: der
Verlauf des Glas-Themas ist für Knöpfe gemacht, und auf den großen Flächen
einer Entwicklungsumgebung sah er aus wie ein Schatten quer über die halbe
Höhe. Die neue Metrik `verlauf_hoehe` lässt ihn über 120 Pixeln ausklingen.

Auch die Liste aus Stand 18 ist abgearbeitet: der Einwand beim verdeckten
Namen, das Verschieben von Konstanten und globalen Variablen, und das
Abwählen in der Vorschau.

Auch die Liste aus Stand 19 ist abgearbeitet: einzelne Blöcke auslassen,
die Formular-Handler ziehen mit, und die Aufrufer zeigen die gemessenen
Durchläufe.

Auch die Liste aus Stand 20 ist abgearbeitet: das Nachbessern, die
Unterordner und die gebrauchte Konstante.

**Wo die Umbauten suchen.** Alles, was „im ganzen Projekt" heißt, geht seit
Stand 21 auch durch die Unterordner, und seit Stand 22 zeigt der
Projektbaum sie als Äste. Er listet seit Stand 23 auch, was neben dem
Quelltext liegt (`.dhform`, `.dhsprite`, `.json`, `.md`, `.csv`, `.png`
und so fort). Was der Editor bearbeiten kann, geht in einen Reiter; ein
Bild oder ein Klang an das Programm des Systems. Geprüft wird nur `.dh` —
eine `.json` durch den Übersetzer zu schicken füllt die Liste mit
Meldungen über etwas, das gar kein Programm ist. Übergangen werden Ordner, die mit `.`
oder `_` anfangen, dazu `target` und `__pycache__`: dort liegt Erzeugtes,
kein Quelltext, und ein Bau-Ordner kann zehntausend Dateien haben.

Auch die Liste aus Stand 21 ist abgearbeitet: die Unterordner im Baum, das
Prüfen vor dem Schreiben und der Reiter für eine neu angelegte Datei.

Auch die Liste aus Stand 22 ist abgearbeitet: die anderen Dateien im Baum,
das Abbrechen und die genannten Fehler.

**Der Projektbaum ist seit Stand 24 kein Eigenbau mehr.** Er war
Handarbeit: alle Dateien je Muster holen, sortieren, die Ordner-Knoten
daraus bauen und eine Buchführung mitschleppen, die Knoten-Nummern gegen
Dateinamen hält. Das ist jetzt ein Widget der Laufzeit
(`GUI_FILETREE`, siehe [module-gui.md](module-gui.md)), und zwei Dinge
fallen dabei ab, die vorher fehlten: eine **frisch angelegte Datei** steht
von selbst im Baum (alle zwei Sekunden nachgesehen; bis Stand 23 erst nach
dem nächsten Öffnen), und ein **Klick auf einen Ordner** klappt ihn um
statt nur das schmale Dreieck. Dafür sind Ordner jetzt **zu**, bis jemand
hineinsieht — der Baum liest sonst einen Ordner, in den niemand schaut.

Auch die Liste aus Stand 24 ist abgearbeitet: alle vier Umbauten sammeln
in Schritten, ein Umbau wird auch **ohne Vorschau** geprüft (und macht sie
auf, wenn er Fehler dazubringt), und die **Häkchen** im Baum sind das
Mittel, einen Umbau auf eine selbst gewählte Menge von Dateien zu
beschränken.

Gegen die Qt-IDE ist seit Stand 9 nichts Benennbares mehr offen. Was als
Nächstes anstünde: die gesetzte Ansicht des Handbuchs kennt **keine
Textauswahl** (kopieren geht nur im Quelltext, und dafür ist der Knopf
oben rechts da); die Suche im gesetzten Handbuch springt zur ersten
Fundstelle **ab der aktuellen Stelle**, kennt aber kein Zurück; und beim
Verschieben gibt es weiterhin **kein Rückgängig über Dateigrenzen** außer
dem einen Schritt, den `umbauZurueck` hält. Ein
Installer ohne Python gibt es seit Stand 3:
`installer/Drachenhauch-IDE.iss` packt `dhrt.exe`, `ide/`, `docs/` und die
Beispiele -- 33 MB statt 92; die Qt-IDE bleibt daneben installierbar, bis
diese hier gleichzieht. Die Liste steht in
[entwurf-python-abbau.md](entwurf-python-abbau.md), Abschnitt C.
