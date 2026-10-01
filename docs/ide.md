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

Über dem Baum, neben dem Projektnamen, steht ein **Filter**: ein Teil des
Dateinamens (Groß/klein egal) lässt nur die passenden Dateien stehen, die
Ordner bleiben und werden beim Aufklappen genauso gefiltert.

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
| 27 | **Knopfarten** und **komfortable Listen** in der Laufzeit; die Befehlspalette zeigt Kürzel und Ordner als Zusatztext rechts, die Hauptknöpfe der Dialoge sind hervorgehoben, der Debugger-Stopp ist rot |
| 28 | **Statusleiste mit Feldern** (Meldung, Stelle der Marke, Umbruch — die beiden rechten anklickbar), die Symbolspur ist eine **Pfadleiste** (ein Klick springt zum Block), und die Werkzeugleiste legt, was nicht passt, in ein **»-Menü** |
| 29 | Im **gesetzten Handbuch** markieren (Ziehen, Doppelklick = Wort, Strg+A) und kopieren (Strg+C); die Suche geht **weiter und zurück** mit Umlauf, markiert die Fundstelle und hebt **alle Fundstellen** hervor |
| 30 | Die Tabelle der Laufzeit kann **Gitter** (`GUI_GRID`, Zellmodus: aktuelle Zelle, Bereich, Tippen bearbeitet, Strg+C/V als Tabulator-Text, Zahlen- und Auswahlspalten); das **Profil** ist ein Gitter, seine Zeilen lassen sich markieren und kopieren |
| 31 | Eine **`.dhform` öffnet den Form-Designer in Drachenhauch** (als Text über die Befehlspalette); der Designer kennt alle 30 Arten und das Gitter, zeigt Felder je Art (Einträge, Spalten, Zellmodus, bearbeitbare Spalten, Spaltenarten, Min/Max/Wert) und schreibt **DH-Code ohne `GUI_LOAD`** (Strg+G) |
| 32 | Die Prüfsammlungen können **vorhandene Programme** prüfen (`--- programm pfad nach MARKE`, `--- inhalt`/`--- ohne datei`) — die Tests des Form-Designers laufen **ohne Python** |
| 33 | Auch **Anim-FSM-Editor und Notenblatt** werden ohne Python geprüft: `--- streichen` nimmt `WINDOW_MAXIMIZE()` aus der Kopie, `--- nachher` lässt ein Drachenhauch-Programm lesen, was gesichert wurde, und Platzhalter gelten auch in Beilagen |
| 34 | Die **IDE selbst** wird zum großen Teil ohne Python geprüft: die Bausteine (`PROCESS_*`, `CODE_*`, Textbereich) und die Fälle der Stände 1 bis 5 stehen in Prüfsammlungen; das Protokoll liest ein `--- nachher`-Programm Zeile für Zeile |
| 35 | Auch die **Stände 6 bis 12** der IDE werden ohne Python geprüft (Einrückung, Vervollständigung, Lesezeichen, Symbolspur, Einstellungen, Schnipsel, Projektbaum, Ersetzen und Umbenennen im Projekt, Werkzeugleiste, Kacheln samt Vorschaubild, Auswahl erweitern, Herauslösen) |
| 36 | Auch die **Stände 13 bis 25** werden ohne Python geprüft (Gerüst, Vorschau vor dem Umbau, Parameter umsortieren, hinzufügen und entfernen, Platzhalter, Aufrufer-Baum, Reiter schließen, Verschieben samt IMPORT, Umbau zurücknehmen und nachbessern, Blöcke auslassen, Formular-Handler, Dateibaum, gesetztes Handbuch, Haken im Baum) |
| 37 | Auch die **Sonderfälle** laufen ohne Python: die Prüfsammlungen können vor dem Lauf ein Hilfsprogramm laufen lassen (`--- vorher`, etwa ein git-Repository), zwischen den Läufen eins (`--- zwischen`, etwa eine Klicklage aus dem Bildschirmfoto messen) und das Programm noch einmal starten (`--- nochmal [in ORDNER]`, Sitzung über einen Neustart). Die Hervorhebung wird im Bild gezählt; in pytest bleibt nur das PDF-Listing |

## Was sie heute kann (Stand 37, 13.09.2026)

| Bereich | Was geht | Kürzel |
|---|---|---|
| Start | Willkommensseite, solange kein Reiter offen ist: oben der Drachenhauch-Schriftzug (aus `daten/bilder/` neben `ide/`; das Logo dort wird auch das Fenstersymbol, ohne die Bilder steht der Titel als Text da), Neu, Öffnen, Beispiele, die zuletzt geöffneten Dateien (Doppelklick), die wichtigsten Kürzel. Vor allem anderen läuft der **Vorspann** (jede `.mp4` in `daten/video/`, eingepasst mit schwarzen Rändern): jede Taste und jeder Klick überspringt ihn, das Kreuz beendet die IDE; abschalten mit „Vorspann beim Start“ in den Einstellungen (`vorspann` in der ide.json). **Welcher** läuft, wählt die Klappliste „Welcher Vorspann“ daneben (`vorspann_datei`): jedes Video dort, benannt nach der Datei (`vorspann_2.mp4` heißt „Vorspann 2“), oder „Abwechselnd“ -- dann kommt bei jedem Start der nächste (`vorspann_zuletzt`). [Ansehen] spielt den gewählten gleich ab. Ein neues Video in `daten/video/` legen genügt, damit es in der Liste steht. Der **Schriftzug** der Willkommensseite und des Info-Fensters (`daten/bilder/schriftzug.png`) ist das letzte Bild von `vorspann_2.mp4`, freigestellt von `tools/schriftzug.dh`; die IDE passt ihn unverzerrt in seinen Kasten ein. Die Sitzung kommt beim nächsten Start wieder (offene Dateien, aktiver Reiter, **Haltepunkte samt Bedingung und Lesezeichen** -- `haltepunkte`/`lesezeichen` in der ide.json), ebenso Thema und Schriftgröße | |
| Dateien | Neu, Öffnen, Datei im Projekt öffnen (Wähler mit unscharfem Filter: `spred` findet `189_sprite_editor.dh`; dazu die zuletzt geöffneten), Zuletzt geöffnet (Untermenü), Sichern, Sichern unter, Reiter schließen und wieder öffnen (Strg+Umschalt+T); bis zu 12 Reiter; Rückfrage bei ungesicherten Änderungen; Listing drucken über einen Druckdialog (Drucker, Kopien, **Schriftgröße, Rand, Zeilennummern, nur die Auswahl** -- gemerkt, und die PDF nimmt dieselben) oder als PDF neben die Quelle (Vorgabe: Courier 9 pt, 20 mm Rand, 66 Zeilen je Seite, Zeilennummern, Kopfzeile mit Seitenzahl; die Zeilenbreite wird an der Schrift gemessen) | Strg+N, Strg+O, Strg+Umschalt+O, Strg+S, Strg+W, Strg+Q, Strg+P |
| Bearbeiten | **Suchleiste** (bleibt offen: Suchfeld mit Weiter/Zurück und Trefferzahl „3 von 12“, Schalter **Groß/klein**, **Ganzes Wort** und **Ausdruck**, alle Treffer im Code eingefärbt; tippen springt zum ersten Treffer, Enter/Umschalt+Enter weiter/zurück, ESC schließt), Ersetzen **einzeln** (ersetzt den markierten Treffer und springt weiter — so sieht man jede Stelle vorher) oder **alle** (ein Schritt für Strg+Z; mit Ausdruck gelten `\1`-Rückverweise, ohne bleibt ein Rückstrich ein Rückstrich), Gehe zu Zeile, Suche im Projekt (alle `.dh` im Projektordner, Treffer unten rechts, Doppelklick öffnet), mit denselben drei Schaltern wie die Suchleiste (sie gelten für Suchen, Ersetzen und Projektsuche gleich), TODO/FIXME-Liste (dieselbe Suche), Befehlspalette (tippen filtert, **Pfeile** wählen, Enter führt aus) | Strg+F, F3, Umschalt+F3, Strg+H, Strg+Umschalt+1, Strg+Alt+Enter, Alt+C, Alt+W, Strg+G, Strg+Umschalt+F, Strg+Umschalt+M, Strg+Umschalt+P |
| Git | Wer hat das geschrieben (`git blame`, Datum und Person je Zeile), was habe ich geändert (`git diff` farbig im Fenster), Verlauf dieser Datei (`git log`); die geänderten Zeilen tragen eine Marke am Rand (nach dem Sichern neu gefragt, nicht je Bild) | Strg+Umschalt+B, Strg+Umschalt+D |
| Schreiben | Umbenennen eines Symbols über die ganze Datei (`CODE_RENAME$`: ganze Wörter, Kommentare und Zeichenketten bleiben; ein krummer Name ändert nichts) — und die **Formulare ziehen mit**: eine `.dhform` nennt ihre Rückrufe beim Namen, und wer das Unterprogramm umbenennt und die Datei stehen lässt, hat einen Knopf, der nichts mehr tut, Schnipsel einfügen (13 Gerüste, `\|` sagt wohin die Marke gehört, die Einrückung der Zeile wird übernommen), eine Marke auf jede Fundstelle des Wortes — danach ändert ein Tippen alle; Alt+Klick legt eine Marke dazu, ESC räumt sie weg. Signaturhilfe: steht die Marke in einer Argumentliste, zeigt die Statuszeile die Signatur des Aufrufs und die Nummer des Arguments | Umschalt+F6, Strg+J, Strg+Umschalt+L |
| Reiter | Jeder Reiter hat ein **Kreuz**, die mittlere Maustaste schließt ihn auch. Trägt er ungesicherte Änderungen, wird gefragt (Sichern / Verwerfen / Abbrechen) — das galt vorher nur beim Beenden, Strg+W nahm sie wortlos mit. **Andere Reiter schließen** lässt nur den vorderen stehen | Strg+W, Strg+Umschalt+W |
| Dateiliste | **Häkchen schränken das Projekt ein** (nur auf Wunsch sichtbar: `Datei → Kästchen im Projektbaum zeigen`, in den Einstellungen oder in der Palette; `baum_haken` in der ide.json -- beim Ausblenden fallen die Haken weg, eine unsichtbare Einschränkung wäre eine Falle): ist etwas angehakt, arbeitet jeder Umbau (Umbenennen, Ersetzen im Projekt, Verschieben), die Projektsuche und das Symbolverzeichnis nur auf dieser Menge -- ein angehakter **Ordner** nimmt alles darunter mit (bis 2026-09-27 machte er das Projekt leer); ohne Haken gilt das ganze Projekt. Der Tooltip über dem Baum sagt es — die Beschriftung über dem Baum sagt es, `Datei → Haken im Projektbaum entfernen` räumt sie ab. Der Projektbaum ist seit Stand 24 ein `GUI_FILETREE`: er liest den Ordner selbst und nur die Äste, die offen sind. Ordner stehen oben und klappen auf Klick auf; eine Datei, die ein anderes Programm anlegt, steht nach spätestens zwei Sekunden da. Strg+Klick sammelt, Umschalt+Klick spannt einen Bereich; **Gewaehlte Dateien oeffnen** macht aus allen Reiter. Beim Sammeln geht noch nichts auf — sonst käme mit jedem Klick ein Reiter dazu, den niemand wollte | Strg+Umschalt+E |
| Einstellungen | Strg+U zeigt alle Schalter an einer Stelle (Thema, Umbruch, Karte, geänderte Zeilen, reguläre Ausdrücke, Vorschlagsliste, Einrückungslinien, Gerüst beim Tippen, Vorschau vor dem Umbau, **Klammern schließen**, **beim Sichern formatieren** -- aus per Vorgabe, nur `.dh`, nie beim automatischen Sichern, und was sich nicht lesen lässt, wird gesichert wie es dasteht --, Schriftgröße des Codes, **Schrift der Listen** -- Gliederung, Ausgabe, Probleme, Vorgabe 18 -- und **Schrift des Handbuchs**, Vorgabe 18, **Schrift der Oberfläche** (Menü, Reiter, Baum, Beschriftungen), Vorgabe 18, **Schriftzug schwach hinter dem Code** (an per Vorgabe; `GUI_TEXTAREA_BACKGROUND`, er füllt das Code-Feld und rollt nicht mit), automatisches Sichern). Sie wirken **sofort**, ohne Übernehmen — ein Thema, das man erst nach dem Schließen sieht, wählt man blind; im Menü stehen sie weiter dort, wo man sie sucht | Strg+U |
| Zeilen | Alles auf ganzen Zeilen, der Auswahl oder der Zeile der Marke, jeder Handgriff ein eigener Undo-Schritt: Kommentar umschalten, Zeile duplizieren, Zeile löschen, Zeilen nach oben/unten, Ein-/Ausrücken (Tab gehört dem Feld selbst), Dokument formatieren (`CODE_FORMAT$`, derselbe Formatierer wie `dhrt fmt`; bei einem Syntaxfehler bleibt alles stehen), Schnipsel einfügen (Strg+J füllt den Filter mit dem Wortanfang links der Marke: wer `for` getippt hat, hat die FOR-Schleife vor sich; **oder einfach `for` tippen und Tabulator** — jeder Schnipsel hat ein kurzes Wort, das ihn aufklappt), automatisch sichern nach einstellbarer Ruhe. Eine neue Zeile übernimmt die **Einrückung** der alten und rückt hinter `SUB`, `FOR`, `THEN` und den anderen Blockwörtern eine Stufe weiter ein; ein `END` oder `NEXT` allein in einer Zeile rückt sie zurück. Und das **Gerüst wächst mit**: hinter `IF x > 0 THEN` setzt der Zeilenumbruch das `END IF` gleich mit darunter, die Marke bleibt dazwischen (`CASE`, `ELSE` und `ELSEIF` nicht — sie rücken ein, schließen aber nichts; abschaltbar). Lesezeichen (blaue Marke) setzen und anspringen — vorwärts und rückwärts **über Dateien hinweg**, dazu die Liste aller | Strg+K, Strg+D, Strg+Umschalt+K, Alt+Hoch/Runter, Alt+Rechts/Links, Umschalt+Alt+F, Strg+F2, F2, Umschalt+F2, Alt+F2 |
| Sprache | Einfärbung des sichtbaren Ausschnitts, dazu jede Fundstelle des Wortes unter der Marke, das zusammengehörende Klammernpaar und ein **Farbfeld** neben jedem `&H`-Literal (ein Klick darauf öffnet den Farbwähler und schreibt die neue Farbe an die Stelle zurück); eine **Symbolspur** über dem Code sagt, in welcher Klasse und welchem Unterprogramm man steht; **Definition hier zeigen** (Alt+F12) blendet zehn Zeilen um die Definition ein, ohne die Stelle zu verlassen; Hilfe zum Wort (unten rechts unter den Problemen, mit eigenem Kopf; ohne Wort steht dort ein Hinweis, was hier erscheint), Vervollständigung — sie geht beim Tippen ab drei Zeichen von selbst auf, der Fokus bleibt dabei im Code-Feld, Strg+Leer holt sie herein, und hat der Name eine **Signatur**, kommt sie als Gerüst mit: aus `CIRC` wird `CIRCLE(x, y, r)` mit markiertem `x`, der **Tabulator** geht zum nächsten Argument; Zur Definition, auch per **Strg+Klick** (solange Strg gehalten ist, zeigt der Zeiger über einem Wort die Hand; gesprungen wird beim Loslassen); **Tooltip beim Überfahren**: verweilt die Maus auf einem Wort, steht dessen Hilfe daneben -- dieselbe wie für das Wort unter der Marke, umbrochen und gekürzt, hinter dem Zeilenende keine; Gliederung links unten (SUB/FUNCTION/CLASS mit Methoden, Doppelklick springt; ohne sie steht „Keine Unterprogramme“ da; die Art steht in der Farbe der Schlüsselwörter, der Name in der der Namen). **Die Codefarben hängen am Thema**: das helle Thema hat seine eigenen, dunkleren Töne, sonst stünden hellgelbe Zeichenketten auf Weiß | Strg+Leer, F12, Strg+Klick |
| Prüfen | Fehlerliste unten rechts, 0,6 s nach der letzten Änderung von selbst; Klick springt zur Zeile; Fehlerzeilen tragen eine orange Marke, und die **Stelle selbst ist unterstrichen** (Wellenlinie, rot für Fehler, gelb für Warnungen) -- verweilt die Maus darauf, steht die Meldung im Tooltip. **Schnellkorrektur** (Strg+.): steht die Marke auf einer Meldung mit Vorschlag, bietet ein Wähler die Korrekturen an -- einen Tippfehler durch den gemeinten Namen ersetzen, ein fehlendes `DIM` anlegen (im Unterprogramm unter seinem Kopf -- dort gibt es zusätzlich die **globale** Variante oben in der Datei, für einen Wert, der den Aufruf überleben soll --, sonst vor dem Block auf oberster Ebene, der Typ aus dem zugewiesenen Wert geraten; ein vorgeschlagener Name steht so da, wie er im Programm geschrieben ist), Klammern um die Werte eines Befehls setzen, `!=` durch `<>`, einen Namen aus einem anderen BASIC durch den hiesigen; der Tooltip sagt, wenn es eine gibt. Dazu: ein fehlendes Blockende (`END IF`, `NEXT`, `END SUB`, `WEND` ...) hinter den Rumpf des offenen Blocks setzen, einen fehlenden `IMPORT` oben einfügen, und eine Variable, die in einem Unterprogramm angelegt und nie benutzt wird, ist ein **Hinweis** (grau, zählt nicht als Problem, Strg+. entfernt die Zeile). Die Vorschläge kommen von der Laufzeit (`CODE_CHECK$`, Feld `korrekturen`), dieselben bekommt VS Code. **Ganzes Projekt prüfen** (Strg+Umschalt+F7): jede `.dh` des Projekts samt Unterordnern (die Haken im Baum gelten), offene Reiter mit dem Stand im Editor, in Schritten mit Zähler in der Statuszeile (ESC bricht ab); die Liste zeigt `datei:zeile: Meldung`, ein Klick öffnet dort. Ein Fehler in einer importierten Datei steht einmal, an seiner eigenen Stelle; Hinweise zählen nicht | Umschalt+F7, Strg+., Strg+Umschalt+F7 |
| Debugger | Haltepunkte (rote Marke) an der Zeile der Schreibmarke oder per **Klick in die Nummernspalte** (links umschalten, rechts mit Bedingung; die Schreibmarke bleibt stehen), bedingte Haltepunkte (violett; `i = 3`, `hp < 10` — der Ausdruck wird im Programm ausgewertet, gehalten wird nur, wenn er wahr ist); Debuggen läuft bis zum ersten Haltepunkt, ohne Haltepunkte steht es in Zeile 1; die angehaltene Zeile ist gelb markiert, Variablen (lokal und global) stehen unten rechts anstelle der Problemliste, dazu die Schritt-Knöpfe; steht das Programm, wertet die Eingabezeile Ausdrücke aus (`? 2 * 21` → `= 42 (INTEGER)`). Rechts neben den Variablen der **Aufrufstapel** (innen oben, jede äußere Zeile ist die, in der sie die innere rief; Doppelklick springt hin) und **Überwachen**: Ausdrücke, die bei jedem Halt neu ausgerechnet werden (Enter im Feld nimmt einen auf, [Entfernen] nimmt den markierten weg; sie bleiben über mehrere Läufe). **Doppelklick auf eine Variable** setzt einen neuen Wert -- ein Ausdruck, gewandelt wie bei einer Zuweisung (in eine INTEGER-Variable kommt kein Text; eine `CONST` ist keine Variable, sie hat der Übersetzer schon eingesetzt). **Bis zur Marke laufen** hält an der Zeile mit der Schreibmarke; ohne laufenden Debugger startet es ihn. Ein Haltepunkt davor hält trotzdem, und „bis hier“ vergeht dabei. **Haltepunkte in importierten Dateien** halten (der Debugger nennt Datei und Zeile, die IDE öffnet die Datei), ein Haltepunkt auf einer Zeile **ohne Code** (Kommentar, Leerzeile, `SUB`-Kopf, `END IF`) rutscht auf die nächste Zeile mit Code, und der rote Punkt wandert mit; hält einer nie, sagt es die Statuszeile. **Während das Programm läuft** gesetzte Haltepunkte gelten sofort, **Anhalten** (F6) hält es an der nächsten Zeile an. Wartet das Programm in einem `INPUT`, sagt es das Debugger-Feld und die **Eingabezeile** schickt die Antwort. Beim Tippen **wandern Haltepunkte und Lesezeichen mit ihrer Zeile**. **Alle Haltepunkte** (Strg+Umschalt+F9) zeigt jeden Haltepunkt jeder Datei mit Bedingung und Quelltext: das Kästchen schaltet einen **an und aus** (ausgeschaltet bleibt er grau stehen und hält nicht), Doppelklick oder [Springen] führt hin, dazu Bedingung ändern, Entfernen, alle an/aus/entfernen; ob an oder aus, steht in der ide.json. **Protokollpunkte** ([Protokollpunkt ...] in der Haltepunkt-Liste, blaue Marke): statt anzuhalten schreibt der Haltepunkt eine Zeile in die Ausgabe (`» spiel.dh:4  i = 3`), Ausdrücke stehen in `{}` (`{{`/`}}` für die Klammern selbst, ein Fehler erscheint als `{? meldung}`). **Trefferzahl** ([Trefferzahl ...]): `5` hält ab dem 5. Mal, `=5` genau beim 5., `%5` jedes 5. Mal; gezählt wird nur, wenn die Bedingung stimmt. **Neu starten** (Strg+Umschalt+F5) stoppt das laufende Programm und startet es in derselben Art (Lauf, Debugger, Profil) neu. Wer während des Debuggens eine Datei ändert, bekommt einmal den Hinweis, dass das Programm noch die alten Zeilen kennt. **Variablen als Baum:** Felder, Tupel, MAPs und Objekte klappen auf (je Ebene höchstens 100 Einträge, drei Ebenen tief, der Rest als „... n weitere“); was aufgeklappt war, ist es beim nächsten Halt wieder; Klassen heißen, wie sie geschrieben wurden (`<Held>`, nicht `<held>`). **Halt am Fehler:** ein Laufzeitfehler, den kein `CATCH` nimmt, hält an seiner Stelle an -- Variablen, Aufrufstapel und Überwachen so, wie sie beim Fehler waren, Ausdrücke lassen sich noch auswerten; F8 (oder ein Schritt) beendet dann. Ein **Grafikprogramm** bleibt im Halt ansprechbar (der Debugger holt die Fensternachrichten ab). **Werte im Code:** im Halt steht grau hinter jeder Zeile vom Kopf des Unterprogramms bis zur Haltezeile, welche Variablen darin vorkommen und was sie gerade haben (`GUI_TEXTAREA_HINTS`); beim Weiterlaufen verschwinden sie. **Werte beim Überfahren:** steht der Debugger in der Datei, zeigt der Tooltip über einem Namen seinen Wert -- samt Mitglieds-Kette (`held.hp`), nie mit Aufruf (`eval` mit `id`); `eval`, Überwachen und die Eingabezeile kennen dafür jetzt auch Felder und Indizes (`held.hp`, `feld[3]`, `karte["a"]`), eine PROPERTY wertet der Debugger nicht aus, weil sie Code ausführen würde | F9, Umschalt+F9, Strg+Umschalt+F9 alle Haltepunkte, Strg+Umschalt+F5 neu starten, Klick/Rechtsklick in die Nummernspalte, F7, F6 anhalten, F8 weiter, F10 drüber, F11 hinein, Umschalt+F11 heraus, Strg+F10 bis zur Marke, Umschalt+F5 stopp, Strg+E Ausdruck |
| Profil | Lauf unter `dhrt profile`; am Ende ein Fenster mit den Zeilen nach Zeit (Anzahl, ms, Anteil, Quelltext), Klick springt zur Zeile. **Nach Funktionen** (Kästchen oben rechts) zählt dieselbe Messung je SUB, FUNCTION und Methode (`Klasse.methode`) zusammen — eine Zeile gehört zur innersten, die sie umschließt, der Rest zum Hauptprogramm; ein Klick springt zum Kopf der Funktion. **Ausgabe leeren** steht unter Ausführen | Strg+Umschalt+Y |
| Ausführen | Starten mit laufender Ausgabe unten links, Eingabezeile für `INPUT`, Stoppen; Export als eigenständiges Programm (`dhrt --export`, nach `<name>_dist/` neben die Quelle, die Ausgabe des Exports läuft unten links mit) | F5, Umschalt+F5, Strg+F6 |
| Werkzeuge | Die Begleit-Editoren in Drachenhauch (SFX-Generator, Partikel-Editor, Tilemap-Editor, Sprite-Editor, Tracker, Form-Designer, Anim-FSM-Editor, Notenblatt) als eigene Programme; die Beispiele als Projekt öffnen | |
| Ansicht | Helles/dunkles Thema (jeder Bereich traegt einen Hauch seiner eigenen Farbe: Projekt und Gliederung blau, Ausgabe gruen, Probleme orange, Debugger violett -- gemischt aus der Grundfarbe des Themas), Vollbild, Schrift größer/kleiner/normal (10 bis 32 px, bleibt gemerkt; **Strg+Plus/Minus/0 und Strg+Rad** -- das Rad überall im Fenster, ein Punkt je Schritt, Plus und Minus auf deutscher wie US-Belegung und am Ziffernblock); Zeilenumbruch (gilt für alle Reiter); Blöcke falten (**je Datei gemerkt**: was beim Schließen des Reiters oder der IDE zugeklappt war, klappt beim nächsten Öffnen wieder zu, sobald die Gliederung die Blöcke kennt; die Blöcke kommen aus `CODE_SYMBOLS$` **und aus der Einrückung** -- alles, unter dem etwas tiefer Eingerücktes steht, also auch eine `FOR`-Schleife; ein Klick auf das Dreieck in der Nummernspalte tut dasselbe); **Seitenleiste** und **untere Leiste** ein- und ausblenden (Alt+1, Alt+2; der Code bekommt die Fläche, gemerkt in der ide.json; die untere kommt von selbst zurück, sobald ein Programm startet, gedebuggt oder profiliert wird); geteilte Ansicht (zwei Reiter nebeneinander); **dieselbe Datei nebeneinander** (Alt+Umschalt+G: links eine zweite Ansicht derselben Datei mit eigener Marke und eigenem Ausschnitt, ein Text und ein Rückgängig für beide; sie folgt dem aktiven Reiter, die Faltpfeile und Haltepunkte stehen nur rechts); Übersichtskarte am rechten Rand (Wort für Wort gezeichnet, heller Kasten für den sichtbaren Ausschnitt, rechts Fehler rot und Warnungen gelb, links die Zeile der Schreibmarke; ein **Klick rollt die Stelle in die Mitte**, **Ziehen** lässt den Ausschnitt folgen -- die Schreibmarke bleibt dabei, wo sie ist, wie beim Mausrad); geänderte Zeilen am Rand; die IDE startet maximiert | Alt+Enter, Strg+Plus, Strg+Minus, Strg+0, Strg+Rad, Alt+Z, F4, Strg+F4, Umschalt+F4, Alt+G, Alt+Umschalt+G, Alt+1, Alt+2 |
| Werkzeugleiste | Unter dem Menü eine Reihe **Sinnbilder** für das, was man ständig braucht: neu, öffnen, sichern, starten, stoppen, debuggen, prüfen, suchen, Zeilenumbruch (kippbar, zieht mit Menü und Alt+Z mit), rechts Handbuch und Einstellungen. Die Leiste ist seit Stand 26 ein Widget der Laufzeit (`GUI_TOOLBAR` mit Einträgen), die Sinnbilder sind dieselben wie in den Menüs. Jeder Knopf ruft denselben Befehl wie sein Menüpunkt und nennt im Tooltip sein Kürzel; abschaltbar unter Ansicht | — |
| Kacheln | Auf der Willkommensseite der **Showcase**: die Beispiele aus `examples/showcase.json` als Kacheln (`GUI_CARD`: die ganze Kachel ist der Knopf) mit Bild, Titel und Kurzbeschreibung in normaler Schriftgröße (dieselbe Liste wie im Qt-Editor). Die Bilder kommen aus `examples/screenshots/` (erzeugt von `tools/showcase_bilder.dh`); fehlt eines, läuft das Beispiel wirklich (`dhrt bild`), eines nach dem anderen im Hintergrund und unsichtbar, und das Bild liegt danach neben der Sitzung. Passen nicht alle Reihen, blättert das **Mausrad** über den Kacheln -- aber nur, wenn dort kein anderes Fenster (Handbuch, Palette ...) darüber liegt. Ohne `showcase.json` stehen acht feste Beispiele da. Ein Klick öffnet das Beispiel | Mausrad |
| Beispiele nach Themen | Alle Beispiele in 16 Themen (Erste Schritte, Spiele, 2D-Grafik, 3D, Klang, Oberflächen, Daten und Netz, Hardware, Werkzeuge ...): *Werkzeuge* -> *Beispiele nach Themen*, der Knopf auf der Willkommensseite oder die Befehlspalette. Je Thema ein Gruppenkopf mit der Zahl, je Beispiel die erste Zeile seines Kopfkommentars (ohne Zierlinien, Dateinamen und interne Vermerke; mitten im Satz umbrochen holt sie die nächste Zeile), rechts der Dateiname. Das Filterfeld sucht in beidem, Enter öffnet den ersten Treffer, [Öffnen und starten] startet gleich. Die Zuordnung steht in `examples/kategorien.json` (von Hand gepflegt; ein Test verlangt, dass jedes Beispiel genau einmal darin steht) | |
| Umbauen | **Auswahl in ein Unterprogramm herauslösen** (Strg+Umschalt+R): die gewählten Zeilen wandern in ein neues `SUB` am Dateiende, an ihrer Stelle steht der Aufruf. Was als Parameter mitmuss, steht nicht im Raten — globale Namen sieht ein SUB ohnehin, also sind es genau die **lokalen** des umgebenden Unterprogramms, die in den Zeilen vorkommen; wer darin auch zugewiesen wird, geht **BYREF**. Danach zählt die IDE die Fehler nach und sagt es, wenn die Auswahl nicht ausgewogen war | Strg+Umschalt+R |
| Umbauen | **Parameter umsortieren, hinzufügen, entfernen** (Strg+Umschalt+U): die Parameter des Unterprogramms unter der Marke umstellen, einen neuen aufnehmen (`hp AS INTEGER = 0` — der Teil hinter dem `=` kommt an jede Aufrufstelle) oder einen wegnehmen — und **die Aufrufe ziehen mit**, in **allen** `.dh` des Projektordners. Die Reihenfolge der Argumente ist die Bedeutung; ein vergessener Aufruf übergibt stumm das Falsche. Ein Aufruf, der eine andere Zahl von Argumenten übergibt (weggelassener Vorgabewert) oder sie **benennt**, wird übergangen und gezählt — dort heißt die Reihenfolge etwas anderes | Strg+Umschalt+U |
| Umbauen | Ein Umbau über das ganze Projekt sammelt **in Schritten**, je Bild ein Häppchen: die Statuszeile zählt mit, **ESC bricht ab**. In einem Bild erledigt, stünde die IDE so lange — bei vierhundert Dateien lange genug, dass man sie für hängend hält, und abbrechen ließe sich nichts, was gar nicht erst zum Zeichnen kommt | ESC |
| Umbauen | **In der Vorschau abwählen**: links stehen die betroffenen Dateien mit Kästchen — was man abwählt, bleibt stehen. Ein Umbau über zwölf Dateien ist selten in allen zwölf gemeint, und „alles oder nichts" hieße dann: von Hand nacharbeiten. Feiner geht es mit **Block auslassen**: die Marke auf eine geänderte Zeile, und der zusammenhängende Block bleibt, wie er war (er steht dann grau mit `~` da). Dahinter liegt keine Textausgabe mehr, sondern eine Folge von Schritten — der Text der Datei entsteht aus denselben Schritten, die man sieht | — |
| Umbauen | **Vorschau vor dem Umbau**: Umbenennen, Im-Projekt-Umbenennen, Im-Projekt-Ersetzen, Herauslösen und die Parameter-Umbauten zeigen erst den Unterschied und fragen (Enter übernimmt, ESC verwirft). Abschaltbar unter Strg+U — **außer wenn es einen Einwand gibt**: gibt es den neuen Namen im Projekt schon, oder zerreißt die Umbenennung eine **Überschreibung** (dieselbe Methode steht in der Oberklasse oder in einer erbenden Klasse — gerufen würde von da an die andere Fassung, ohne Fehlermeldung), geht die Vorschau auf und sagt es oben. Wer einen Einwand nur in die Statuszeile schreibt, hat ihn nicht vorgebracht | — |
| Umbauen | **Verschieben** (Strg+Umschalt+V): ein Unterprogramm oder eine ganze **Klasse** wandert in eine andere Datei des Projekts, samt den Kommentarzeilen darüber — und jede Datei, die es benutzt, bekommt den `IMPORT` der Zieldatei dazu. In Drachenhauch fügt `IMPORT` den Text ein; ohne diesen Teil wäre das Verschieben ein Umbau, der die Übersetzung kaputt macht. Steht die Marke in einer Klasse, ist die **Klasse** gemeint — eine Methode allein wäre ohne sie kein Unterprogramm mehr. Der erste Eintrag im Wähler legt eine **neue Datei** an. Und es geht in beide Richtungen: braucht das Verschobene etwas, das zurückbleibt, importiert die **Zieldatei** die Quelle. Steht die Marke auf einer `CONST`- oder `DIM`-Zeile, ist **die Zeile** die Einheit. Hat das Ziel den Namen schon, sagt die Vorschau es — zwei gleichen Namens in einer Datei sind kein Übersetzungsfehler, der zweite gewinnt einfach | Strg+Umschalt+V |
| Umbauen | **Prüfen** in der Vorschau: `dhrt --check` läuft über die Texte, die geschrieben **würden**, mit der Blockauswahl von jetzt. Wer einen Block auslässt und damit etwas kaputt macht, sieht es vor dem Schreiben — und **welchen** Fehler, nicht nur wie viele: die Meldungen stehen oben im Unterschied. Nur auf Verlangen — ein Umbau über zwölf Dateien bräuchte sonst zwölf Übersetzungsläufe, ehe man den Unterschied überhaupt zu sehen bekommt | — |
| Umbauen | **Letzten Umbau nachbessern** (Strg+Umschalt+G): dieselben Schritte noch einmal, mit derselben Auswahl. Ein ausgelassener Block ließe sich sonst nur zurückholen, indem man den ganzen Umbau zurücknimmt und ihn von vorn macht; gerechnet wird aus den Schritten, nicht aus dem, was gerade in der Datei steht | Strg+Umschalt+G |
| Umbauen | **Umbau zurücknehmen** (Strg+Umschalt+Z): Strg+Z im Code-Feld nimmt nur den Reiter zurück, in dem man steht — ein Umbau über sechs Dateien wäre damit sechsmal zurückzunehmen, und zwar in sechs Reitern, die man dafür erst öffnen muss. Die letzten zehn liegen auf einem **Stapel**, jeder Druck nimmt einen weiter zurück. **Wurde eine der Dateien seit dem Umbau geändert, nimmt es gar nichts zurück** und nennt sie — sonst wäre die spätere Arbeit wortlos weg (halb zurückgenommen wäre schlimmer als gar nicht). Ein zurückgenommener Umbau lässt sich **wiederholen** (Bearbeiten, Palette), mit derselben Probe in der Gegenrichtung; ein neuer Umbau leert diesen Vorrat | Strg+Umschalt+Z |
| Aufrufe | **Wer ruft das auf?** (Umschalt+F12): die Gegenrichtung zu F12 — alle Stellen, an denen das Unterprogramm unter der Marke gerufen wird, über alle Dateien des Projekts. Als **Baum**: unter jedem Aufruf stehen die Aufrufer des Unterprogramms, in dem er steht, drei Ebenen tief; ein Klick springt hin. Gezählt wird auch, wo der Name **ohne Klammern** weitergegeben wird (`f = malen`) — dort wird entschieden, dass er später läuft. Gab es einen Profillauf, steht an jeder Stelle, **wie oft** sie gelaufen ist; ohne Lauf steht dort nichts, eine Null wäre eine Aussage, die niemand gemessen hat. Die Definition steht nicht dabei, sie ist kein Aufruf | Umschalt+F12 |
| Fundstellen | **Alle Fundstellen im Projekt** (Strg+Umschalt+F12): jede Zeile, in der der Name unter der Marke als ganzes Wort steht — eine Variable, eine Konstante, ein Feld, in jeder Schreibweise, nicht in Zeichenketten und Kommentaren, offene Reiter mit dem Stand im Editor. Die Liste steht unten rechts wie bei der Projektsuche, ein Doppelklick springt hin. Das ist Text, keine Namensauflösung: zwei lokale `i` in zwei Unterprogrammen stehen in derselben Liste | Strg+Umschalt+F12 |
| Auswahl | **Erweitern** nimmt die nächstgrößere Klammer: Wort, Zeile, Block, Elternblock, ganze Datei. **Verkleinern** geht denselben Weg zurück — ein Stapel merkt sich jede Stufe, statt sie neu zu erraten | Strg+Umschalt+Hoch / Runter |
| Umbenennen | **Im ganzen Projekt umbenennen** (Strg+Umschalt+F6): der Name unter der Marke, in allen `.dh` des Projektordners. `CODE_RENAME$` lässt Kommentare und Zeichenketten aus, wie beim Umbenennen in einer Datei | Strg+Umschalt+F6 |
| Vergleichen | **Zwei Reiter nebeneinander**: die geteilte Ansicht geht an, und in beiden Feldern bekommt jede abweichende Zeile eine Marke. Verglichen wird ohne git über die längste gemeinsame Teilfolge auf Zeilen — die Reiter müssen dafür nicht gesichert sein, und genau während man tippt will man es wissen | — |
| Vergleichen | **Mit einer anderen Datei vergleichen**: `git diff --no-index` im selben Fenster und derselben Färbung wie git diff. Gemeint ist die im Projektbaum gewählte Datei, wenn es eine andere ist — nur sonst fragt der Datei-Dialog | — |
| Zeilen-Werkzeuge | **Sortieren**, **Doppelte entfernen**, **Leerraum am Zeilenende entfernen** — auf der Auswahl, ohne Auswahl auf der ganzen Datei | — |
| Ersetzen | **Im ganzen Projekt ersetzen** (Strg+Umschalt+H): zählt erst die Stellen und fragt, dann schreibt es alle `.dh` des Projektordners. Ein Reiter mit unge**sicherten** Änderungen bricht es ab — die würden die Datei beim nächsten Sichern wieder überschreiben | Strg+Umschalt+H |
| Ausgabe | **Ausgabe durchsuchen** (Strg+Umschalt+A): die Zeilen des laufenden Programms. Derselbe Text noch einmal heißt weitersuchen. **Zur Stelle einer Meldung:** eine Zeile mit `datei.dh:zeile` (Laufzeit- und Parse-Fehler, Warnungen) ist eingefärbt, Doppelklick oder Enter öffnet die Datei dort -- der Name wird neben dem gelaufenen Programm gesucht, dann im Projekt, und darf Leerzeichen tragen | Strg+Umschalt+A, Doppelklick/Enter |
| Symbole | **Symbol im Projekt suchen** (Strg+Umschalt+S): alle SUB, FUNCTION, CLASS und Methoden über ALLE `.dh` des Projektordners, filterbar; der Filter ist mit dem Wort unter der Marke vorbelegt. Derselbe Index trägt **Zur Definition** und **Definition hier zeigen** über Dateigrenzen: was `CODE_DEFINITION` im eigenen Text nicht findet, steht vielleicht nebenan | Strg+Umschalt+S |
| Hilfe | **Eingebaute Befehle nachschlagen** (Strg+F3): alle Namen, die die Vervollständigung kennt, mit Signatur und Beschreibung, filterbar, mit Knopf zum Einfügen. Handbuch im Fenster, **gesetzt statt roh** — seit Stand 25 vom Widget der Laufzeit (`GUI_RICHTEXT`): Überschriften in drei Größen, Absätze umgebrochen, Aufzählungen mit Punkt, Code-Blöcke dicktengleich, Tabellen mit **Spalten**, fett und kursiv, und **anklickbare Verweise** (ein Verweis auf ein anderes Dokument öffnet es, eine Sprungmarke rollt im selben); der Knopf oben rechts schaltet auf den Quelltext. Seit 2026-09-27 ist es **farbig und navigierbar**: links ein **Inhaltsverzeichnis** aus den Überschriften (ein Klick springt, beim Lesen geht die Markierung mit), **Zurück/Vorwärts** wie im Browser (Knöpfe oben links oder Alt+Links/Rechts über dem Fenster; ein Schritt ist ein anderes Dokument, eine Sprungmarke oder ein Sprung im Inhalt), Codeblöcke in den Farben des Editors, Code im Text auf eigenem Grund, Überschriften in drei Farben, Tabellen mit getöntem Kopf und abwechselnden Zeilen, Zitate als ein Block mit Grund. **Jeder Codeblock hat Knöpfe**: *Kopieren*, *In neuen Reiter* (ein ungesicherter Reiter mit dem Code) und *Starten* (läuft sofort, ohne nach einem Namen zu fragen -- abgelegt als `handbuch_beispiel.dh` neben der `ide.json`). **In allen** sucht über alle Dokumente: links stehen dann je Dokument die Treffer mit Zeilennummer, ein Klick öffnet die Stelle und hebt den Suchtext hervor, *Inhalt* holt das Inhaltsverzeichnis zurück. Der **Quelltext** steht in der Schrift des Editors und ist eingefärbt (Überschriften, `Code`, **fett**, Verweise, Tabellenstriche, Codeblöcke wie im Editor). Das Fenster wächst mit dem Schirm und lässt sich an der Ecke ziehen; Meldungen stehen unter dem Text. F1 schlägt das Wort unter der Schreibmarke in `docs/` nach und öffnet das Dokument mit den meisten Fundstellen in Codeschrift; Klappliste aller Dokumente, Suche im Dokument; Tastenkürzel-Übersicht; **Über Drachenhauch** zeigt Symbol, Schriftzug, Fassung und Ordner (Enter oder ESC schließt). Die Oberfläche schreibt Umlaute; die Suche in Befehlspalette und Wählern nimmt dafür beide Schreibweisen (`oeffnen` findet „Öffnen“) | F1, Strg+F1 |

Einstellungen und Sitzung liegen in EINER JSON-Datei im Nutzerprofil
(`%APPDATA%\Drachenhauch\ide.json`, sonst `~/.config/Drachenhauch/ide.json`):
`zuletzt`, `sitzung`, `aktiv`, `hell`, `schrift`, `umbruch`, `karte`,
`git_rand`, `regex`, `vorschlag`, `geruest`, `umbau_vorschau`, `autosichern`, `wiederherstellung_s` und `projekte` (Sitzung je Ordner, die zwölf letzten — wer an zwei Sachen
arbeitet, will beim Wechsel nicht die Reiter der anderen wiederfinden).
Die Umgebungsvariable
`DH_IDE_KONFIG` legt einen anderen Ort fest — die Tests brauchen das, sonst
schriebe jeder Testlauf dem Nutzer eine fremde Sitzung in die Datei.

**Absturz-Wiederherstellung** (seit 18.09.2026). Neben der `ide.json`
liegt ein Ordner `wiederherstellung/`. Jede laufende IDE schreibt dort alle
30 Sekunden (`wiederherstellung_s`) in einen **eigenen** Unterordner
`<kennung>/stand.json`: Pfad und Text jedes Reiters mit ungesicherten
Änderungen, auch unbenannter, dazu die Uhrzeit. Geschrieben wird in eine
Nebendatei und dann umbenannt, damit ein Absturz mitten im Schreiben keinen
halben Stand hinterlässt. Die Uhrzeit ist zugleich das Lebenszeichen: Beim
Start bietet die IDE nur an, was **verwaist** ist — als beendet markiert
oder seit drei Takten ohne Lebenszeichen. Den Stand einer zweiten IDE, die
gerade läuft, lässt sie in Ruhe. (Die Qt-Fassung hatte einen gemeinsamen
Ordner für alle: dort hielt eine zweite IDE die Sicherungen der ersten für
Absturzreste, und wer zuerst sauber beendete, löschte die des anderen.) Die
Frage beim Start hat drei Antworten: **Wiederherstellen** öffnet die Reiter
mit ihrem geretteten Text, *ungesichert* — die Datei auf der Platte bleibt,
wie sie war, bis man selbst sichert. **Verwerfen** löscht die Stände,
**Später** (auch ESC) lässt sie liegen, und der nächste Start fragt wieder.
Ein verwaister Stand ohne Reiter verschwindet still. Beendet man sauber
(nichts ungesichert, oder über die Rückfrage mit Sichern oder Verwerfen),
geht der eigene Stand weg. **Endet die IDE ohne Rückfrage** mit
ungesicherten Reitern, bleibt er als beendet liegen.

**Das Kreuz des Fensters** (auch Alt+F4) fragt seit dem 18.09. wie
Datei → Beenden: ist etwas ungesichert, kommt die Rückfrage (Alle sichern /
Verwerfen / Abbrechen), sonst endet die IDE sofort. Vorher endete die
Schleife an `QUITREQUESTED()`, und die Arbeit war wortlos weg -- gefunden
beim Bau der Wiederherstellung. Dazu brauchte es zwei Dinge in der Laufzeit:
`WINDOW_CLOSE_REQUESTED()` meldet das Kreuz, ohne das Bildlimit eines
Testlaufs mitzuzählen, und **ESC beendet ein Programm mit gui nicht mehr**
(siehe `docs/builtins-grafik.md`) -- vorher schloss ein ESC im Dialog die
ganze IDE, samt ungesicherter Arbeit. Tests
`tests/pruef/werkzeug_ide_schliessen.dhtest` (echtes `WM_CLOSE` über
PowerShell).

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
- **Neues Projekt aus Vorlage** (Datei → Neues Projekt aus Vorlage, Strg+Alt+N):
  Spiel (Fenster mit Spielschleife, Figur aus `assets/figur.png`, bewegt mit
  den Pfeiltasten und `DELTA()`), Formular-Anwendung (gui mit Textfeld,
  Knopf und Liste), Konsolenprogramm (`main.dh` mit INPUT, `rechnen.dh`
  und eine Prüfsammlung `tests.dhtest`) oder leer; jedes mit LIESMICH.md.
  Name und Ort (Vorgabe: neben dem jetzigen Projekt, [Wählen ...] öffnet
  den Ordnerdialog); ein belegter Ordner wird abgelehnt. Danach ist der
  neue Ordner das Projekt und `main.dh` offen.
- **Bilder und Klänge:** steht die Maus im Code auf einer Zeichenkette, die
  ein vorhandenes Bild nennt (`LOADIMAGE("assets/figur.png")`), erscheint
  es nach kurzer Ruhe neben dem Zeiger samt Maßen -- gesucht neben der
  Datei, dann im Projekt. Ein Bild oder Klang aus dem Projektbaum öffnet
  eine **Vorschau** in der IDE (Bild mit Maßen und Größe, Klang mit
  [Abspielen]/[Stopp]); [Mit dem System öffnen] gibt es weiter.
- **Quellcodeverwaltung** (Bearbeiten → Quellcodeverwaltung, Strg+Alt+G): die
  geänderten Dateien des git-Repositorys als Liste; das Kästchen merkt eine
  Datei für die nächste Übergabe vor (`git add`) oder nimmt sie zurück,
  rechts steht ihr Unterschied (eine neue Datei ganz). Darunter Nachricht
  und **Übergeben** (Enter im Feld tut dasselbe), **Änderung verwerfen**
  (mit Rückfrage; nicht, solange die Datei ungesichert in einem Reiter
  steht, und keine neue Datei -- das hieße löschen), **Holen**/**Senden**
  (`git pull`/`push` als eigener Prozess, die Ausgabe läuft unten mit).
  Oben steht der Zweig samt Vorsprung/Rückstand, daneben die Auswahl der
  Zweige mit **Wechseln** und **Neu ...** (legt einen Zweig am jetzigen
  Stand an, die Änderungen kommen mit). **Beiseitelegen**/**Zurückholen**
  (`git stash push -u` / `pop`). Wechseln und Beiseitelegen verweigern sich,
  solange ein Reiter ungesichert ist, und ziehen offene Reiter danach von
  der Platte nach. Eine Datei mit Konflikt heißt in der Liste „Konflikt“;
  **Bearbeiten → Konflikt unter der Marke** ersetzt den Block
  `<<<<<<< … ======= … >>>>>>>` um die Marke durch meine, ihre oder beide
  Fassungen (ein Schritt, Strg+Z nimmt ihn zurück). **Ansicht → Wer hat
  diese Zeile geschrieben** zeigt gedämpft am Ende der Zeile mit der
  Marke, wer sie zuletzt geändert hat, wann und mit welcher Nachricht
  (`git blame`, nur für gesicherte Reiter).
- **Variable einführen / einsetzen, SUB in FUNCTION** (Bearbeiten, Strg+Alt+V /
  Strg+Alt+I / Strg+Alt+U): ein markierter Ausdruck wird eine Variable
  (`DIM name AS TYP : name = …` vor der Anweisung, der Typ geraten und im
  Kasten zu ändern); eine Variable, die genau einmal belegt wird,
  verschwindet, und an jeder Lesestelle steht der Ausdruck -- verweigert,
  wenn sie ein zweites Mal belegt, an ein Unterprogramm mit BYREF-Parameter
  übergeben wird, der Ausdruck etwas aufruft und mehrfach gebraucht würde
  oder sich eine seiner Variablen dazwischen ändert. Eine SUB mit genau einem
  BYREF-Parameter wird eine FUNCTION: der Parameter wird lokal, `EXIT SUB`
  wird `RETURN r`, und jeder Aufruf `name(a, x)` als Anweisung wird im
  ganzen Projekt `x = name(a)` -- verweigert, wenn die SUB den
  hineingegebenen Wert liest, bevor sie ihn setzt. Alle drei gehen durch die
  Umbau-Vorschau.
- **Live-Vorschau** (Ansicht → Live-Vorschau, Strg+Alt+L): nach jeder Pause
  beim Tippen läuft das Programm im Reiter 30 Bilder lang über `dhrt bild`,
  das letzte Bild steht in einem eigenen Fenster. Gerechnet wird eine
  versteckte Kopie neben der Datei (`.dh_live_vorschau.dh`, damit relative
  Bilder und IMPORTs gefunden werden), die danach wieder weg ist; die
  Eingabe des Programms ist zu, nach 10 s wird abgebrochen. Ein Programm
  ohne Fenster oder mit einem Fehler zeigt die Meldung statt eines Bildes.
- **Aufrufzähler** (Ansicht → Aufrufzähler über Unterprogrammen, an per
  Vorgabe): hinter jeder SUB/FUNCTION steht gedämpft, wie oft ihr Name im
  Projekt benutzt wird ("3 Aufrufe", "nicht aufgerufen") -- auch in anderen
  Dateien, als FUNCREF und als Methode, nicht im Kommentar, nicht in einer
  Zeichenkette. Ein Klick darauf zeigt "Wer ruft das auf?". Gezählt wird
  Text (`CODE_NAMES`), keine Namensauflösung: zwei Methoden gleichen Namens
  zählen zusammen. Im Debugger-Halt stehen dort die Werte, danach wieder die
  Zähler.
- **Prüfsammlungen** (Ausführen → Prüfsammlung ausführen, Strg+Alt+F5): `dhrt
  test` über die `.dhtest` im Reiter (sonst über den Projektordner), die
  Ausgabe läuft mit; jeder fehlgeschlagene Fall steht mit Datei und Zeile in
  der Liste unten rechts, ein Klick springt zu ihm. Strg+Alt+Umschalt+F5
  nimmt nur den Fall unter der Marke (`dhrt test --fall Name`).
  **Im Überblick** (Strg+Alt+T): alle `.dhtest` des Projekts als Baum, je
  Fall ein Punkt -- grün bestanden, rot fehlgeschlagen, grau noch nicht
  gelaufen oder übersprungen; eine Sammlung ist rot, sobald ein Fall darin
  rot ist, und steht dann offen. [Alle ausführen], [Auswahl ausführen]
  (eine Sammlung ganz oder einen Fall allein), [Fehlgeschlagene] (nur die
  roten, je einer mit `--fall`), Doppelklick öffnet den Fall. Der Stand
  gilt bis zum nächsten Lauf, auch über das Schließen des Fensters.
- **Eigene Schnipsel** (Bearbeiten → Eigene Schnipsel bearbeiten): eine
  Textdatei `schnipsel.txt` neben der `ide.json`. Jeder Schnipsel beginnt mit
  `=== Name | kuerzel` und reicht bis zum nächsten; `|` im Text ist die
  Stelle der Marke. Nach dem Sichern gilt die Datei sofort; ein eigenes
  Kürzel, das es schon gibt, **ersetzt** den eingebauten Schnipsel (in Strg+J
  steht dann „eigen, ersetzt"). Ein Kürzel, das kein Wort ist, bleibt weg —
  der Schnipsel ist dann nur über Strg+J zu haben. **Auswahl als Schnipsel
  sichern ...** hängt den markierten Code mit „Name | Kürzel" an die Datei.
- **Eigene Tastenbelegung** (Hilfe → Tastenkürzel ändern ...): Befehl wählen,
  neues Kürzel eintippen (leer = keins). Gespeichert wird in `tasten.json`
  neben der `ide.json`, und zwar nur, was von der Vorgabe abweicht
  (`{"neu": "Strg+Alt+N"}`); die Datei lässt sich auch von Hand bearbeiten
  und gilt nach dem Sichern. Nimmt ein Kürzel einem anderen Befehl seine
  Taste weg, verliert der sie — zwei Befehle auf einer Taste feuerten beide.
  Unbekannte Befehle und Kürzel, die die IDE nicht kennt, stehen in der
  Ausgabe. Zurücksetzen löscht die Datei. Dahinter steht **eine Tafel** aller
  Menüpunkte mit Befehl und Vorgabe-Kürzel (`menueBefehl`); aus ihr feuern
  auch die Klicks (vorher 136 einzelne Zeilen in der Bildschleife).
  Umbelegen lässt sich, was einen Menüpunkt hat.
- **Terminal** (Ansicht → Terminal, Alt+3, oder der Reiter „Terminal" über
  der Ausgabe): die Eingabezeile unten schickt Befehle an `cmd` (Windows)
  bzw. `sh`, die Ausgabe läuft mit, Fehler rot, ein Rückgabewert ungleich 0
  steht dahinter. Jeder Befehl ist ein eigener Prozess im Ordner des
  Terminals (zuerst der Projektordner); **`cd` wirkt** trotzdem, weil das
  Skript am Ende seinen Ordner meldet. Eine mit `set` gesetzte Variable
  gilt dagegen nur für den einen Befehl. Läuft ein Befehl, geht die
  Eingabezeile an ihn (`set /p`, ein `INPUT`); **Abbrechen** beendet ihn
  samt allem, was er gestartet hat. Pfeil hoch/runter blättert durch die
  Befehle, `cls`/`clear` leert. Startet man ein Programm (F5), schaltet der
  Bereich auf die Ausgabe zurück. Unter Windows kommen Umlaute richtig an
  (`chcp 65001`).
- **Parameternamen an Aufrufen** (Ansicht, an per Vorgabe): hinter einer
  Zeile mit einem Aufruf, dem ein festes Argument (Zahl, Text, TRUE/FALSE)
  mitgegeben wird, steht gedämpft die Signatur -- `kreis(100, 50, 20)` →
  `kreis(x, y, r)`, auch für eingebaute Befehle (`MID$(s, start, n)`,
  optionale Parameter ohne Klammern). Ein Hinweis je Zeile (der erste
  solche Aufruf). Die Signaturen merkt sich die IDE bis zum nächsten
  Sichern.
- **Beim Einfügen einrücken** (Ansicht, an per Vorgabe): ein mehrzeiliger
  Block, eingefügt mit Strg+V oder über das Kontextmenü, bekommt die
  Einrückung der Stelle, an der er landet; sein innerer Aufbau bleibt (der
  Bezug ist die kleinste Einrückung der weiteren Zeilen -- so passt auch ein
  Block, der ab seinem ersten Wort kopiert wurde). Mitten in einer Zeile
  richten sich die weiteren Zeilen nach deren Einrückung. Das Anpassen ist
  ein eigener Rückgängig-Schritt: Strg+Z holt zuerst den Block, wie er
  kopiert wurde. Mit Tabulatoren wird nichts angefasst, und in **Spalte 0**
  auch nicht -- dort sagt nichts, wohin der Block gehört, er bleibt, wie er
  kopiert wurde (vorher rückte er auf 0 und verlor seine Einrückung).
- **Nächstes / voriges Problem** (Bearbeiten, Alt+F8 bzw. Umschalt+Alt+F8
-- F8 allein gehört dem Debugger): die Marke springt zum nächsten Fehler
  bzw. zur nächsten Warnung der Datei (Hinweise nicht), am Ende wieder von
  vorn; die Meldung steht in der Statuszeile.
- **Zur letzten Änderung** (Bearbeiten, Strg+Umschalt+Rücktaste): zurück
  an die Stelle, an der zuletzt getippt wurde, auch in einer anderen Datei;
  Strg+Alt+Links führt wieder dorthin, wo man vorher war.
- **Mit gesicherter Fassung vergleichen** (Bearbeiten, auch im
  Kontextmenü eines Reiters): zeigt den Unterschied zwischen Reiter und
  Datei auf der Platte als farbigen diff (drei Zeilen Umfeld); von dort
  **Sichern** oder **Verwerfen** (ein Rückgängig-Schritt -- Strg+Z holt die
  Änderungen zurück).
- **Groß/klein und Zeilen verbinden** (Bearbeiten → Zeilen, Palette):
  GROSSBUCHSTABEN, kleinbuchstaben, Anfangsbuchstaben Groß auf der Auswahl
  (ohne Auswahl auf dem Wort unter der Marke); die Auswahl bleibt auf dem
  geänderten Text. **Zeilen verbinden** macht aus den Zeilen der Auswahl
  (ohne Auswahl: diese und die nächste) eine: die Einrückung der ersten
  bleibt, dazwischen ein Leerzeichen (nicht hinter `(`/`[`, nicht vor
  `)`/`]`/`,`), leere Zeilen fallen weg, ein ` _` am Zeilenende
  verschwindet mit. Jeweils ein Rückgängig-Schritt.
- **Klammerpaare farbig** (Ansicht, an per Vorgabe): jede Klammertiefe hat
  eine eigene Farbe (drei im Wechsel, in `farben.json` als `klammer1`..
  `klammer3`). Klammern in Zeichenketten und Kommentaren zählen nicht; die
  Tiefe beginnt je Zeile neu, außer die vorige endet mit ` _`.
- **Suchtreffer einzeln ersetzen:** die Treffer von Im Projekt suchen
  (Strg+Umschalt+F) tragen Haken, alle an. Rechtsklick auf die Liste:
  Treffer anspringen, Diesen Treffer ersetzen ..., Angehakte Treffer
  ersetzen ..., Alle/Keinen anhaken; auch Bearbeiten → Angehakte
  Suchtreffer ersetzen .... Ersetzt wird nur in den gewählten Zeilen, mit
  denselben Schaltern wie die Suche (Groß/klein, ganzes Wort, Ausdruck),
  über die Vorschau des Umbaus und mit Zurücknehmen. Eine offene Datei
  bekommt den Text in ihrem Reiter (ungesichert), die anderen auf der
  Platte. Zeigt die Liste wieder Probleme, sind die Haken weg.
- **Rechtsklick ins Code-Feld:** Ausschneiden, Kopieren, Einfügen, Zur
  Definition, Wer ruft das auf?, Umbenennen, Schnellkorrektur, Haltepunkt
  setzen/entfernen, Im Projektbaum zeigen. Der Rechtsklick setzt vorher die
  Marke an die Stelle (in einer Auswahl bleibt sie), die Befehle meinen also
  das angeklickte Wort bzw. die angeklickte Zeile. In der Nummernspalte
  bleibt der Rechtsklick der für die Bedingung eines Haltepunkts.
- **Rechtsklick auf einen Reiter:** Schließen, Andere schließen, Rechts
  davon schließen, Pfad kopieren, Im Projektbaum zeigen -- jeweils für den
  angeklickten Reiter, nicht den vorderen. Beim Sammel-Schließen bleibt ein
  Reiter mit ungesicherten Änderungen offen (die Statuszeile sagt, wie
  viele).
- **Im Projektbaum zeigen** (Bearbeiten, Strg+Alt+B): wählt die Datei des
  vorderen Reiters im Projektbaum, klappt die Ordner darüber auf und rollt
  die Zeile ins Bild; eine ausgeblendete Seitenleiste kommt zurück. Liegt
  die Datei nicht im Projektordner, sagt es die Statuszeile.
- **Typ-Hinweise** (Ansicht → Typ-Hinweise, an per Vorgabe): hinter einer
  Zeile mit `FOR EACH` steht gedämpft blau, was die Variable hält
  (`p: INTEGER`, bei einer MAP in der Paar-Form `k: STRING, v: FLOAT`) --
  ihr Typ steht sonst nirgends. Was der Übersetzer nicht weiß (ein Tupel,
  das Ergebnis von `SPLIT$`), steht nicht da. Die Hinweise kommen mit der
  Prüfung (`CODE_TYPES$`).
- **Eigene Farben** (Ansicht → Eigene Farben bearbeiten): öffnet
  `farben.json` neben der `ide.json`, beim ersten Mal mit den Werten beider
  Themen als Vorlage. Je Thema (`"dunkel"`, `"hell"`) stehen die
  Code-Farben (`grund`, `kommentar`, `text`, `zahl`, `schluessel`,
  `operator`, `fundstelle`, `klammer`, `suchtreffer`, `typ`), die Töne der
  Bereiche (`projekt`, `ausgabe`, `probleme`, `debugger`) und unter
  `"oberflaeche"` jeder Schlüssel von `GUI_THEME_SET` (`accent`,
  `window_bg`, `widget_bg`, `text_fg` ...), Farben als `"#RRGGBB"`. Nur
  was anders sein soll, muss drinstehen; nach dem Sichern (Strg+S) gilt die
  Datei sofort. Unbekannte Namen und falsche Farben meldet die Ausgabe.
- **Dateien im Projektbaum** (Rechtsklick auf eine Zeile): Neue Datei,
  Neuer Ordner (im gewählten Ordner bzw. im Ordner der gewählten Datei),
  Umbenennen, Löschen und Pfad kopieren; dieselben Befehle stehen in der
  Befehlspalette. Der Rechtsklick wählt die Zeile, öffnet sie aber nicht.
  **Umbenennen** zieht offene Reiter, Haltepunkte, Lesezeichen und
  Startargumente mit, und jedes `IMPORT "..."` im Projekt, das die Datei
  meint, bekommt den neuen Namen (in einem offenen Reiter als eigener
  Rückgängig-Schritt, auf der Platte sonst). **Löschen** fragt vorher,
  löscht endgültig (einen Ordner samt Inhalt) und schließt die Reiter der
  gelöschten Dateien; den Projektordner selbst löscht die IDE nicht.
- **Zurück zur vorigen Stelle / Vorwärts** (Bearbeiten, Strg+Alt+Links /
  Strg+Alt+Rechts, auch die Seitentasten der Maus): gemerkt wird jeder
  Sprung in eine andere Datei oder um mindestens acht Zeilen, ohne dass sich
  der Text geändert hat -- Zur Definition, Problemliste, Suche, Gliederung,
  ein Klick weit weg. Bild auf/ab und Pos1/Ende zählen nicht. Höchstens 50
  Stellen; ein neuer Sprung nimmt die Vorwärts-Liste weg.
- **Ausgabe filtern und sichern:** das Feld „Filter ...“ über der Ausgabe
  zeigt nur Zeilen, die den Text enthalten (Groß/klein egal, leer = alle);
  Doppelklick auf eine Fehlerzeile springt weiter an ihre Stelle. Rechtsklick
  in die Ausgabe: Zeile kopieren, Angezeigte Zeilen kopieren, Ausgabe
  sichern ... (die angezeigten Zeilen als Textdatei), Ausgabe leeren.
- **Fehlertext am Zeilenende** (Ansicht → Fehlertext am Zeilenende, an per
  Vorgabe): die Meldung von Prüfen steht hinter ihrer Zeile, rot bei einem
  Fehler, gelb bei einer Warnung -- lesbar, ohne die Maus hinzubewegen.
  Mehrere Meldungen einer Zeile stehen in einem Hinweis, lange werden
  gekürzt (der ganze Satz steht in der Problemliste und im Tooltip);
  Aufrufzähler und Profilzahlen derselben Zeile folgen dahinter.
- **Startargumente und Umgebung je Datei** (Ausführen → Startargumente ...
  bzw. Umgebung für den Start ...): gilt für die Datei im vorderen Reiter,
  für Starten, Debuggen und Profil, und bleibt in der `ide.json` (`start`).
  Argumente werden an Leerzeichen getrennt, `"in Anführungszeichen"` bleibt
  eins (`ARG$(0)` ...). Die Umgebung schreibt man `NAME=wert NAME2="mit
  Leerzeichen"`; sie gilt nur für das gestartete Programm, nicht für die
  IDE und nicht für den nächsten Start einer anderen Datei. Leer = keine.
- **Beim Sichern neu starten** (Ausführen, aus per Vorgabe): läuft ein
  Programm (F5), startet Strg+S es neu -- auch wenn eine importierte Datei
  gesichert wurde, und ohne den Reiter zu wechseln. Debugger und Profil
  bleiben unberührt, das automatische Sichern startet nie neu.
- **Profil im Code** (Ansicht → Profil im Code, an per Vorgabe): nach
  Ausführen → Profil aufnehmen (Strg+Umschalt+Y) liegt hinter jeder gemessenen
  Zeile ein Farbband, blassgelb bei wenig Zeit bis kräftig rot bei der
  teuersten Zeile, und an den 30 teuersten stehen die Zahlen am Zeilenende
  (`0.42 ms  12.3 %  30000x`). Die Bänder wandern beim Tippen mit ihrer Zeile
  (`GUI_TEXTAREA_LINE_COLORS`), auch in einem später geöffneten Reiter einer
  gemessenen Datei. Gemessen ist, was gelaufen ist: nach größeren Änderungen
  neu messen, oder Befehlspalette → „Profil im Code entfernen" (die Tabelle
  im Profilfenster bleibt).
- **Englische Oberfläche** (Ansicht → „Language: English (at next start)",
  bzw. zurück „Sprache: Deutsch"): Menüs, Knöpfe, Befehlspalette, Tooltips,
  Dialoge und Statuszeile sind englisch, Kürzel heißen dann `Ctrl+Shift+O`
  (die Laufzeit versteht beide Schreibweisen, auch in `tasten.json`).
  Gewechselt wird beim nächsten Start (`sprache` in der `ide.json`,
  `DH_IDE_SPRACHE=en` übersteuert). Jeder Anzeigetext geht durch `tr$("...")`,
  die Tabelle steht in `ide/sprache/en.txt` (je Zeile Deutsch, Tabulator,
  Englisch); `tests/pruef/werkzeug_ide_englisch.dhtest` meldet jeden Text ohne
  Übersetzung. Deutsch bleiben: das Handbuch, die Beschreibungen der
  Beispiele, Meldungen in der Ausgabe (etwa zur Tastenbelegung) und die
  Werkzeuge 183–199.
- **Mitlaufender Blockkopf** (`GUI_TEXTAREA_SET(ta, "kopfzeilen", 3)`, Ansicht
  → Mitlaufender Blockkopf, an per Vorgabe): ist der Kopf einer SUB, einer
  Schleife oder eines IF oben hinausgerollt, der Block aber noch im Bild,
  steht die Kopfzeile oben angeheftet — höchstens drei, außen zuerst, auf
  eigenem Grund mit Kante. Ein Klick darauf springt hin. Die Blöcke sind
  dieselben wie für die Faltung; die Marke rutscht nie unter einen Kopf.
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
- **Klammern schließen** (`GUI_TEXTAREA_PAIRS`): `(`, `[`, `{` und `"`
  bekommen ihr Gegenstück gleich dahinter; wer es trotzdem tippt, tritt nur
  darüber, und die Rücktaste zwischen einem leeren Paar nimmt beide. Das
  einfache Hochkomma fehlt mit Absicht — es beginnt einen Kommentar.
  Abschaltbar in den Einstellungen (zweite Spalte) und in der Palette.

Der Debugger ist ein Client von `dhrt debug`: das Kind schreibt Ereignisse
als JSON-Zeilen auf stdout (`paused` mit `line`, `file`, Tiefe, `locals`,
`globals`, `stack` -- innen zuerst, je `name`, `line` und `file` -- und
`watches` -- je `expr` und `value` oder `error`; `watches` als eigenes
Ereignis nach `set-watches`; `breakpoints` als Antwort auf
`set-breakpoints`, je `file`, `line`, `actual` (wo er wirklich hält) und
`verified`; `run-to-error`, wenn `run-to` nirgends halten kann (fremde
Datei, kein Code darunter) -- das Programm steht dann weiter; `input` mit
`line`, wenn das Programm in einem `INPUT` wartet;
`set-result`/`set-error`; `eval-result`/`eval-error`; `output`;
`finished`; `error` mit `line` und `file`) und nimmt Kommandos auf stdin
(`continue`, `step-over`, `step-into`, `step-out`, `set-breakpoints`,
`set-watches` mit `exprs`, `run-to` mit `line` und optional `file`, `set`
mit `name` und `value` (ein Ausdruck), `eval`, `input` mit `text`,
`pause`, `stop`). Jede Variable in `locals`/`globals` kann `children` tragen
(gleich aufgebaut: `name`, `type`, `value`, `children`), eine Instanz hat als
`type` ihren Klassennamen. Ein nicht abgefangener Fehler meldet sich erst als
`paused` mit `reason: "error"` und `message` (an der Fehlerstelle, `eval` und
`set-watches` gehen noch), dann nach dem nächsten Kommando als `error`.

**Zeilen sind Zeilen der Datei, nicht der zusammengefügten Quelle.** Bis
2026-09-27 zählte der Debugger nach dem Einsetzen der `IMPORT`s: mit einem
`IMPORT` am Anfang hielt er „in Zeile 7“, obwohl er in Zeile 3 stand, und
ein Haltepunkt in Zeile 4 traf nie. Ein Eintrag in `breakpoints` kann außerdem `hits` (Trefferzahl wie oben) und
`log` (Protokolltext; dann meldet sich der Haltepunkt als Ereignis `log` mit
`text`, `line`, `file`, statt anzuhalten) tragen. Bedingungen und Ausdrücke
rechnen `MOD`, `\` und `^` wie die VM (bis 2026-09-27 kannte der Debugger sie
nicht, und `i MOD 2 = 0` hielt bei jedem Durchlauf). `set-breakpoints` nimmt zwei Formen,
auch gemischt: `lines` (+ `conditions`, Zeile → Ausdruck) meint die
gestartete Datei, `breakpoints` ist eine Liste von `{file, line,
condition}` für jede Datei des Programms. Ein Haltepunkt auf einer Zeile
ohne Code rutscht zur nächsten Zeile mit Code **derselben** Datei.

Gelesen wird stdin von einem eigenen Faden. **Was während des Laufens
kommt, gilt erst beim nächsten Halt** -- außer `pause` und einem
`set-breakpoints`/`set-watches` mit `"now": true`: die wirken an der
nächsten Zeile. Ohne den Schalter bliebe ein Skript, das alle Kommandos
vorab schickt, richtig: sein `set-breakpoints` hinter einem `continue`
meint den Halt danach. `input`-Kommandos warten, bis ein `INPUT` sie
abholt; `stop` davor gewinnt.

Beim ersten Halt schickt die IDE Haltepunkte (alle Dateien, `now`) und
überwachte Ausdrücke und wartet auf die `breakpoints`-Antwort: steht dort
ein Haltepunkt genau hier oder hält keiner, bleibt sie stehen, sonst läuft
das Programm weiter (oder bis zur Marke, wenn sie so gestartet wurde).
Beim Tippen wandern die Marken im Code-Feld mit ihrer Zeile
(`GUI_TEXTAREA_MARKS_GET`), die IDE zieht Haltepunkte und Lesezeichen
danach nach. Pfade vergleicht sie mit `SAMEFILE` -- `helfer.dh` kann als
`C:\...\helfer.dh` oder `C:/.../helfer.dh` ankommen.

**Lichtstreif** (`GUI_WINDOW_GLOW`): beim Start und wenn ein Programm
losgeht, streift ein warmes Licht schräg von rechts nach links über das
Fenster, und die Rahmen blitzen auf, wo es sie kreuzt (Art `rahmen`).
Abschaltbar in den Einstellungen (zweite Spalte, `glanz` in der ide.json);
ein Testlauf mit `DHRT_FRAMES` zeigt ihn nur mit `DH_IDE_GLANZ=1`. Der Profiler
(`dhrt profile`) liefert am Ende eine JSON-Zeile mit `total_time`, `lines`
(Zeile, `file`, Anzahl, Zeit), der Programmausgabe und bei einem Fehler
`error`, `error_line` und `error_file`. Wie beim Debugger sind es Zeilen der
DATEI (bis 2026-09-27 der zusammengefügten Quelle -- nach einem `IMPORT`
stimmte keine); die Profiltabelle nennt Zeilen anderer Dateien als
`helfer.dh:3`, die Funktionsansicht rechnet die Bereiche je Datei, und ein
Klick springt in die richtige Datei.

Vor dem Starten, Debuggen und Profilieren sichert die IDE **alle** geänderten
Reiter mit Namen, nicht nur den vorderen -- eine importierte Datei liest das
Programm von der Platte. Ein Ordner als Startargument
(`dhrt run ide/ide.dh -- projekt`) wird das Projekt. Den Rand mit den
geänderten Zeilen fragt git nur, wenn die Datei in einem Repository liegt
(`.git` aufwärts gesucht, ohne Prozess).

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
`hbsuche <y> <anzahl> <markierter text>`,
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
`auto gesichert`, `sicherung <anzahl> [beendet]`, `sicherung bleibt|entfernt`,
`wiederherstellung angeboten <anzahl>|verworfen|spaeter`,
`wiederhergestellt <pfad>|(neu)`, `farbfeld <stelle>`, `farbe <wert>`, `leistenknopf <befehl>`, `statusfeld <nr>`, `pfad sprung <zeile>`, `ende`. So sehen die Prüfsammlungen, was sie
getan hat; Tasten kommen über `AUTOMATION_PLAY` herein (F5 startet, F7
prüft). Die Stände 1 bis 5 prüft `tests/pruef/werkzeug_ide.dhtest` ohne
Python, die Stände 6 bis 12 `tests/pruef/werkzeug_ide_6_12.dhtest`, die
Stände 13 bis 25 `tests/pruef/werkzeug_ide_13_25.dhtest`, was mehrere Läufe,
ein git-Repository oder eine Bildmessung braucht
`tests/pruef/werkzeug_ide_sonderfaelle.dhtest`, die Absturz-Wiederherstellung
`tests/pruef/werkzeug_ide_wiederherstellung.dhtest`, die Bausteine einzeln
`tests/pruef/ide_bausteine.dhtest`. Seit Stand 38 ist keiner der Fälle mehr
in pytest: auch das PDF-Listing liegt in den Sonderfällen, seine gepackten
Seiten entpackt `BUFFER_INFLATE`.

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

Gegen die Qt-IDE ist seit Stand 9 auf Menü-Ebene nichts mehr offen; die kleineren Lücken in Tastatur, Maus und Panels stehen in `docs/entwurf-python-abbau.md`, Abschnitt 7.3. Seit
Stand 29 lässt sich im gesetzten Handbuch **markieren und kopieren**, und
die Suche geht **weiter und zurück** (Enter / Umschalt+Enter, zwei Knöpfe),
markiert die Fundstelle und hebt alle anderen hervor. Was als Nächstes
anstünde: beim Verschieben gibt es weiterhin **kein Rückgängig über
Dateigrenzen** außer den Schritten, die `umbauZurueck` hält. Ein
Installer ohne Python gibt es seit Stand 3:
`installer/Drachenhauch-IDE.iss` packt `dhrt.exe`, `ide/`, `docs/` und die
Beispiele -- 33 MB statt 92; die Qt-IDE bleibt daneben installierbar, bis
diese hier gleichzieht. **Gebaut wird er seit 2026-09-16 in Drachenhauch**
(`dhrt run installer/bauen.dh`: Fassung aus `VERSION$()`, Lizenzen über
`installer/lizenzen.dh`, dann ISCC); nur die Laufzeit selbst baut weiterhin
`rust/build_runtime.py` -- eine laufende `.exe` lässt sich nicht
überschreiben. **Seit 2026-09-19 packt dasselbe `bauen.dh` auch für macOS
(`.dmg`) und Linux (`.tar.gz` mit `install.sh`)**; der Starter kopiert die
Beispiele beim Start in `Dokumente/Drachenhauch/examples` und sagt der IDE
über `DH_IDE_BEISPIELE`, wo sie liegen (Einzelheiten in
[installer/README.md](../installer/README.md)). Die Liste steht in
[entwurf-python-abbau.md](entwurf-python-abbau.md), Abschnitt C.
