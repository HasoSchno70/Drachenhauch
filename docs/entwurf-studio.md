# Entwurf: Drachenhauch Studio

*Eine eigenständige IDE für Programme mit Oberfläche, im Stil von Xojo,
aber mit der Laufzeit `dhrt`. Sie ersetzt den Form-Designer
(`examples/197_form_designer.dh`). Entschieden am 2026-11-01:*

* *Die IDE läuft in der eigenen gui, nicht in GTK.*
* *Ein Fenster ist eine Klasse, Ereignisse sind ihre Methoden.*
* *Ein Projekt ist ein Ordner mit lesbaren Dateien.*
* *Der Name ist „Drachenhauch Studio“.*

**Stand:** Stufe 1 bis 4 sind gebaut (2026-11-01, Ordner `studio/`, Handbuch
[studio.md](studio.md), Prüfungen `tests/pruef/werkzeug_studio.dhtest`,
`tests/pruef/werkzeug_studio2.dhtest`, `tests/pruef/werkzeug_studio3.dhtest` und
`tests/pruef/werkzeug_studio4.dhtest`).
Für Stufe 2 kamen in die Laufzeit schließbare Reiter im Reiterwerk
(`GUI_TABCONTROL_SET "schliessbar"`, `GUI_TABCONTROL_CLOSED`) und
`GUI_TREE_ITEM_RECT`. Für Stufe 3 kopiert `dhrt --export` keine importierten
Quellen mehr als Dateien neben die Exe (sie stecken im Bytecode). Die
Vervollständigung im Code-Feld ist noch offen. Die drei Messungen vor Stufe 1
ergaben:
* Der Neuaufbau kostet bei 1000 Controls 2,7 ms.
* Ein Klick ins Hauptfenster legt es über die Entwurfsfläche;
  `GUI_WINDOW_FRONT` je Bild holt sie zurück.
* Ein Widget beim Namen zu finden braucht es nicht, weil die Basisklasse ihre
  Controls beim Anlegen in Feldern hält.

## Warum der heutige Form-Designer billig wirkt

Das Bildschirmfoto zeigt es in einem Blick:

* Die Palette ist eine Textliste aus 30 Wörtern ohne Sinnbilder und ohne
  Gruppen.
* Der Inspektor rechts ist fast leer: Titel, Breite, Höhe, ein Kästchen,
  eine Klappliste, darunter viel Fläche.
* Die Entwurfsfläche ist ein dunkles Rechteck in einer dunkleren Fläche,
  ohne Raster, ohne Lineal, ohne Hilfslinien.
* Er kennt **ein** Formular und **keinen** Code. Was ein Knopf tut,
  schreibt man woanders.

Die Technik darunter ist nicht das Problem. Die Entwurfsfläche ist ein echtes
`GUI_WINDOW` im Entwurfsmodus (`GUI_WINDOW_DESIGN`); die Laufzeit zeichnet
die Controls genau so, wie das Programm sie später bekommt. Seit dieser
Woche kann die gui außerdem:

* eine Stildatei mit Klassen und Nachfahren,
* die Sans als Vorgabe-Schrift,
* Popover, Toast und Infoleiste,
* ein Dock mit verschiebbaren Leisten,
* Sinnbilder,
* Fließlayout und Größengruppen.

Was fehlt, ist eine Anwendung, die das alles benutzt.

## Was Xojo ausmacht, und was davon übernommen wird

| Xojo | Studio |
|---|---|
| Navigator links: App, Fenster, Module, Klassen, Menüs, Bilder | Baum mit denselben Knoten; unter einem Fenster seine Controls, unter einem Control seine Ereignisse |
| Layout-Editor mit Ausrichtungslinien, die beim Ziehen einrasten | dasselbe auf dem Entwurfsfenster: Linien zu Kanten und Mitten der Nachbarn, Abstandsvorschläge, Raster abschaltbar |
| Bibliothek rechts mit Sinnbild je Control, nach Gruppen | Kacheln mit Sinnbild und Namen (Eingabe, Auswahl, Anzeige, Behälter, Daten, nicht sichtbar), Suchfeld oben |
| Inspektor mit aufklappbaren Gruppen (Position, Aussehen, Verhalten, Schrift) und passenden Editoren | Akkordeon, je Eigenschaft der richtige Editor: Zahl, Text, Kästchen, Klappliste, Farbe (Farbwähler im Popover), Schrift (Schriftwahl), Bild |
| Doppelklick auf ein Control: „Event Handler hinzufügen“ mit allen Ereignissen | derselbe Dialog; das Ereignis landet als Methode in der Klasse des Fensters, der Code-Editor springt hin |
| Code-Editor je Methode | ein Code-Feld je Methode (mit Kopfzeile, die man nicht tippt), Färbung, Vervollständigung hinter `Self.` und den Control-Namen, Fehler an der Methode |
| Run: übersetzen und starten, Debugger | Starten erzeugt den Startcode und ruft `dhrt run`; Fehler springen an die Methode; Debugger über `dhrt debug` wie in der IDE |
| Build: eigenständige Anwendung | `dhrt --export`, wahlweise schlank, mit Symbol |
| nicht sichtbare Controls (Timer) | Timer als Control unter dem Fenster, gebaut über das timer-Modul |
| Menü-Editor | eigener Editor für die Menüleiste eines Fensters, Kürzel als Text wie in `GUI_MENU_ITEM` |

Bewusst **nicht** übernommen werden Xojos Datenbankeditor, Web- und
Mobilziele, die Plugin-Schnittstelle und das binäre Projektformat.

## Das Projekt auf der Platte

```text
meinprojekt/
  projekt.dhstudio          ' JSON: Name, Startfenster, Elemente, Symbol, Ausfuhr
  App.dh                    ' CLASS App: Geoeffnet, Beenden, eigene Methoden
  Hauptfenster.dhform       ' das Layout, wie GUI_SAVE es schreibt
  Hauptfenster.dh           ' CLASS Hauptfenster EXTENDS Hauptfenster_Basis
  Werkzeuge.dh              ' ein Modul: SUBs und FUNCTIONs
  Kunde.dh                  ' eine Klasse
  bilder/                   ' Bilder des Projekts
  _studio/                  ' erzeugt, nie von Hand: nicht ins git
    main.dh                 ' Einstieg: IMPORTs, App anlegen, Startfenster, Bildschleife
    studio_laufzeit.dh      ' StudioApp, StudioFenster, studioLauf
    Hauptfenster_basis.dh   ' CLASS Hauptfenster_Basis: Controls, Aufbau, Verdrahtung
    projekt/                ' der Code des Projekts im Editorstand
```

**Eine Klasse lässt sich in Drachenhauch nicht auf zwei Dateien verteilen.**
Darum zwei Klassen je Fenster:

* `Hauptfenster_Basis` ist erzeugt. Sie hat ein Feld je Control, baut das
  Fenster und verdrahtet jedes Ereignis mit
  `GUI_ON_CLICK(Self.knopf1, Self.knopf1_Klick)`. Für jedes verdrahtete
  Ereignis steht dort eine leere Methode. **Gebaut (Stufe 3) wird anders als
  hier zuerst geplant:** nicht Aufruf für Aufruf, sondern aus dem Layout als
  JSON, das in der Klasse steht (`GUI_FROM_JSON`, die Rückrufnamen
  herausgenommen); die Felder holt `GUI_WINDOW_WIDGET`. So sieht das Fenster
  genau so aus wie im Entwurf, auch für Eigenschaften, die der DH-Code-Weg
  nicht kennt -- und eine `.dhform` braucht es zur Laufzeit trotzdem nicht.
* `Hauptfenster` gehört dem Nutzer. Sie leitet ab und überschreibt die
  Methoden, die er gefüllt hat. Gebundene Methoden werden beim Aufruf
  aufgelöst, die Überschreibung gewinnt also.

So bleibt die Datei des Nutzers lesbar (nur sein Code), und das Projekt
läuft ohne die IDE mit `dhrt run _studio/main.dh`. Eine Ausfuhr braucht keine
`.dhform` neben der Exe.

Den Code des Projekts schreibt das Studio beim Starten als Abbild des
Editors nach `_studio/projekt/`, nicht aus dem Projektordner eingebunden.
Dazu führt `codeSetzen$` eine Zeilenkarte (Methode, Zeile im Rumpf,
Einrückung): eine Meldung `projekt/App.dh:7` wird damit zu
`App.Geoeffnet, Zeile 3`, auch wenn die Datei im Projektordner von Hand
anders geschrieben ist. Für die Prüfung vor dem Start setzt das Studio
dieselben Teile im Speicher zusammen und fragt `CODE_CHECK$`; das END einer
Methode zählt dabei als Zeile hinter dem Rumpf (dort meldet der Übersetzer
ein offenes `IF`).

## Ereignisse

Die gui hat Rückrufe für Klick, Änderung, Enter, Fokus, Fokus weg, Hover
und Verlassen (`GUI_ON_*`). Was Xojo darüber hinaus hat, setzt eine kleine
Bibliothek *laufzeit.dh* im Ordner des Studios aus vorhandenen Abfragen zusammen, je Bild
in der Bildschleife:

**Seit Stufe 4 fragt die Basisklasse alle Ereignisse je Bild ab**
(`GUI_CLICKED`, `GUI_CHANGED`, `GUI_ENTERED`, Fokus und Maus über den Stand
des vorigen Bildes), statt `GUI_ON_*`-Rückrufe zu setzen. Ein Rückruf läuft
innerhalb von `GUI_UPDATE`; eine Methode darin könnte keinen Dialog
aufmachen und auf seine Antwort warten. Abgefragt läuft sie danach, und
`Frage()`, `Meldung()`, `Eingabe()` und `ZeigenModal()` drehen selbst Bilder
(`studioBild`), bis die Antwort da ist.

* **Fenster:** `Geoeffnet` nach dem Aufbau, `Schliessen` über
  `GUI_WINDOW_CLOSED`, `Groesse` bei geänderter Fenstergröße.
* **Listen und Bäume:** `Doppelklick` über `GUI_DOUBLE_CLICKED`.
* **Zeichenfläche:** `Zeichnen` je Bild nach `GUI_DRAW`, beschnitten auf die
  Fläche; `Maus` bei einem Klick darauf. `Ziehen` und `Rad` sind noch offen.
* **Timer:** ein Feld vom Typ `StudioTimer` (Intervall, Aktiv,
  Wiederholen), je Bild gefragt (`studioFaellig`), nicht über `TIMER_EVERY`
  -- ein Rückruf liefe wieder innerhalb einer Aktualisierung.
* **Tasten:** `Taste` im Fenster mit Fokus (noch offen).

Die Basisklasse `StudioFenster` in derselben Bibliothek trägt das Fenster,
öffnet (`Zeigen`) und schließt es (`Zumachen`). Mehrere Exemplare desselben
Fensters gehen, weil alles an `Self` hängt. Beim Namen kennt sie ihre
Controls nicht -- die Felder reichen.

## Die Oberfläche der IDE

* **Werkzeugleiste oben:** Neu, Öffnen, Sichern; Starten, Anhalten, Bauen;
  Rückgängig und Wiederholen.
* **Navigator links** als Dock-Leiste.
* **In der Mitte Reiter:** je Fenster der Layout-Editor oder der Code einer
  Methode.
* **Rechts** Bibliothek und Inspektor als Reiter einer Leiste, wie in Xojo.
* **Unten** Ausgabe und Probleme.

Gestaltet über eine eigene Stildatei *studio.dhstil*, nicht über
Einzelbefehle: Flächen mit Rundung und Schatten, Sinnbilder in der
Akzentfarbe, ruhiger Hintergrund mit hellerer Entwurfsfläche, Raster als
Punkte. Ein helles und ein dunkles Thema.

Der Code-Editor übernimmt aus der IDE, was über Befehle der Laufzeit geht:
Färbung (`SYNTAX_SPANS`), Fehler (`CODE_CHECK$`), Vervollständigung und Hover
(`CODE_COMPLETE`, `CODE_HOVER$` mit Ordner, auch hinter `Self.`),
Klammern schließen, Einrückung. `ide/ide.dh` wird nicht eingebunden: das sind
20 000 Zeilen, die an einer anderen Fensteraufteilung hängen.

## Stufen

1. **Gerüst und Aussehen.**
   * Programm `studio/` aus mehreren Dateien.
   * Dock, Werkzeugleiste, Stildatei.
   * Bibliothek mit Sinnbildern und Gruppen.
   * Inspektor mit Gruppen und echten Editoren.
   * Entwurfsfläche mit Raster, Griffen, Mehrfachauswahl, Ausrichtungslinien
     und Rückgängig.
   * Ein Fenster, noch ohne Projekt.
   * Am Bild abgenommen, bevor es weitergeht.
2. **Projekt und Code.**
   * Das Projektformat und der Navigator.
   * „Ereignis hinzufügen“ und der Code-Editor je Methode.
   * App, mehrere Fenster, Module, Klassen.
3. **Starten und Bauen.** (gebaut)
   * Startcode und Basisklassen erzeugen, `dhrt run` mit Ausgabe.
   * Fehler an der Methode anzeigen.
   * Bauen über `dhrt --export`.
4. **Was ein Programm braucht.** (gebaut)
   * Menü-Editor, Timer, Dialoge und modale Fenster.
   * Bilder im Projekt.
   * Debugger über `dhrt debug`.
5. **Ablösen.**
   * Ein Beispielprojekt (etwa die Rechnungen aus `examples/196_rechnungen.dh`
     als Studio-Projekt).
   * Der Form-Designer 197 verweist auf das Studio.
   * Die IDE öffnet `.dhstudio` im Studio.

**Geprüft wird je Stufe** mit einer Prüfsammlung `werkzeug_studio*.dhtest`.
Die Klicks sind echt, über Aufnahmen und Fensternachrichten. Erzeugtes läuft
durch `dhrt --check` und wird gestartet. Gegenproben mit gezielten
Verfälschungen.

## Offene Punkte, die vor Stufe 1 gemessen werden

* Wie viele Widgets verträgt ein Entwurfsfenster, das bei jeder Änderung
  neu gebaut wird (heute `GUI_WINDOW_DESTROY` + `GUI_FROM_JSON`)? Ab welcher
  Größe muss der Designer Widgets einzeln nachziehen?
* Reichen `GUI_WINDOW_DESIGN` und das Zeichnen über einer Zeichenfläche für
  Griffe und Ausrichtungslinien, oder braucht es eine eigene obere Schicht
  im Entwurfsmodus?
* Fehlt der gui ein Befehl, ein Widget beim Namen zu finden? Die Bibliothek
  kann die Widgets eines Fensters durchgehen. Bei vielen Fenstern wäre ein
  Befehl billiger.
