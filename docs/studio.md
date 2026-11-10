# Drachenhauch Studio

Drachenhauch Studio ist eine IDE für Programme mit Oberfläche, im Stil von
Xojo: Fenster zusammenklicken, Eigenschaften im Inspektor einstellen, später
den Code der Ereignisse schreiben und das Programm starten. Es läuft mit der
Laufzeit `dhrt` und ist selbst ein Drachenhauch-Programm. Der Entwurf mit
allen Stufen steht in [entwurf-studio.md](entwurf-studio.md).

```text
dhrt run studio/studio.dh
dhrt run studio/studio.dh -- meinprojekt
dhrt run studio/studio.dh -- fenster.dhform
```

Ein Ordner mit `projekt.dhstudio` geht als Projekt auf, ein Ordner ohne
bekommt ein neues Projekt, eine `.dhform` geht als einzelnes Fenster auf.

Installiert steht „Drachenhauch Studio“ im Startmenü bzw. im
Anwendungsmenü, und eine `.dhstudio` öffnet per Doppelklick im Studio (unter
Windows, wenn die Verknüpfung beim Installieren gewählt war). Unter macOS und
Linux startet `drachenhauch --studio [projekt]` es aus dem Terminal. In der
IDE: Werkzeuge → Drachenhauch Studio (mit dem Projektordner, wenn er ein
Studio-Projekt ist), und eine `.dhstudio` oder `.dhform` im Projektbaum geht
im Studio auf. Das Studio ersetzt den früheren Form-Designer (bis 2026.45).
Der Startbildschirm der IDE bietet es gleich nach dem Vorspann an
(„Modern – das Studio“).

**Stand: Stufe 4.** Projekte mit mehreren Fenstern, Modulen und Klassen,
Ereignisse und ein Code-Feld je Methode; Starten, Bauen und Fehler an der
Methode, in der sie stehen; Timer, Menüs, Dialoge, die warten, Bilder und
ein Debugger. Dazu eigene Controls und Datenbanken im Projekt.

## Die Oberfläche

| Bereich | was er tut |
|---|---|
| Werkzeugleiste | Neu, Öffnen, Sichern; Fenster, Methode, Ereignis hinzufügen; Rückgängig, Wiederholen, Ausrichten, Zoom; Starten, Anhalten, Prüfen, Bauen |
| Navigator (links oben) | das Projekt als Baum: App, Fenster mit ihren Controls und Ereignissen, Module, Klassen, ihre Methoden und Eigenschaften; ein Klick öffnet |
| Bibliothek (links unten) | alle Bausteine der gui in Gruppen, mit Vorschau; Suchfeld darüber |
| Mitte | Reiter mit Kreuz: je Fenster das Layout, je Methode ihr Code, je Element seine Eigenschaften |
| Inspektor (rechts) | die Eigenschaften des gewählten Controls, ohne Auswahl die des Fensters; ist der Reiter eines anderen Elements vorn (Datenbank, Modul), nur dessen Name und Art |
| Statusleiste | Meldung, Name, Lage und Größe des gewählten Controls, Zoom (Klick: 100 %) |

Die Leisten lassen sich wie in der IDE verschieben, als Reiter stapeln und
abtrennen. Unter Ansicht gibt es ein helles und ein dunkles
Erscheinungsbild.

**Die Oberfläche gibt es auf Deutsch und auf Englisch.** Ansicht ->
„English interface“ schaltet um; die Wahl steht in `studio.json` neben der
`ide.json` und gilt beim nächsten Start. Ohne eigene Wahl folgt das Studio
der Sprache der IDE. Auf Englisch heißen auch die neuen Namen englisch
(`MainWindow`, `Button1`), „Neues Projekt“ bietet englische Vorlagen an
(`studio/vorlagen/en`: dieselben Projekte mit englischen Namen, Texten und
Kommentaren), und die Meldungen der Laufzeit (Probleme, Ausgabe) kommen
englisch. **Die Namen der Ereignisse bleiben, wie sie sind** (`Klick`,
`Geaendert`, `Aktion` ...): sie sind Teil der Methodennamen
(`Knopf1_Klick`), und ein Projekt muss in beiden Sprachen dasselbe bleiben.
Ebenso die Befehle, die das Studio einem Programm mitgibt (`Meldung`,
`Frage`, `Zumachen`).

## Das Projekt

Ein Projekt ist ein Ordner mit lesbaren Dateien:

```text
meinprojekt/
  projekt.dhstudio      Name, Startfenster, die Elemente (JSON)
  App.dh                CLASS App EXTENDS StudioApp
  Hauptfenster.dhform   das Layout, wie GUI_SAVE es schreibt
  Hauptfenster.dh       CLASS Hauptfenster EXTENDS Hauptfenster_Basis
  Werkzeuge.dh          ein Modul: SUBs und FUNCTIONs
  Kunde.dh              eine Klasse
  .gitignore            _studio/ (dort erzeugt Stufe 3 den Startcode)
```

Ein neues Projekt hat die App und ein Fenster `Hauptfenster`. Neues Projekt
(Strg+Umschalt+N) fragt, womit es anfangen soll: **Leer**, **Formular mit
Eingaben** (Felder in einer Spalte, OK und Abbrechen rechts unten,
Prüfregeln), **Adressbuch mit Datenbank** (SQLite, Tabelle mit Filterzeile,
an die Spalten gebundene Felder) oder **Zeichenfläche mit Timer** (ein Ball,
der abprallt). Jede Vorlage ist ein fertiges Projekt, das gleich mit F5
läuft; die Vorlagen liegen als Projektordner unter `studio/vorlagen/` und
werden nie überschrieben. Ein Projekt, an dem noch nichts getan ist, fragt
beim nächsten Neu oder Öffnen nicht nach. Sichern (Strg+S) schreibt alle
Dateien; ein noch nie gesichertes Projekt fragt nach einem Ordner. Ein umbenanntes Element nimmt seine alten Dateien mit, ein
gelöschtes verschwindet beim nächsten Sichern.

| Handgriff | wirkt so |
|---|---|
| Strg+Umschalt+N | neues Projekt |
| Strg+O | Projekt (`projekt.dhstudio`) oder einzelnes Fenster öffnen |
| Strg+N | Fenster hinzufügen (fragt nach dem Namen) |
| Projekt → Modul / Klasse hinzufügen | ein Modul bzw. eine Klasse, ihre Eigenschaften gehen auf |
| Projekt → Als Startfenster | das Fenster, mit dem das Programm beginnt (im Navigator „(Start)“) |
| Projekt → Element löschen | nimmt Fenster, Modul oder Klasse samt Code aus dem Projekt (die App bleibt) |

Ein Name ist ein Bezeichner und im Projekt eindeutig, denn er wird zum Namen
der Klasse bzw. des Moduls. Den Namen eines Fensters ändert man im
Inspektor (ohne Auswahl).

## Code und Ereignisse

Ein Fenster ist eine Klasse, seine Ereignisse sind Methoden. Eine Methode
heißt `<Control>_<Ereignis>` (`knopf1_Klick`), ein Ereignis des Fensters
oder der App nur `<Ereignis>` (`Geoeffnet`). Die Basisklasse, die das
Studio beim Starten erzeugt, verdrahtet sie.

| Handgriff | wirkt so |
|---|---|
| Doppelklick auf ein Control im Entwurf | „Ereignis hinzufügen“ für dieses Control; auf leere Fläche: für das Fenster |
| Strg+Umschalt+E | dasselbe für die Auswahl (oder für das Element des vorderen Reiters) |
| Strg+Umschalt+M | Methode hinzufügen: Name, Parameter (`a AS INTEGER, b AS STRING`), Rückgabe (leer = SUB) |
| Strg+Leertaste | Vorschläge an der Schreibmarke; hinter `Self.` oder `objekt.` gehen sie von selbst auf. In der Liste springt Tippen, Enter oder Tab setzt ein, Esc schließt |
| Klick auf eine Methode im Navigator | öffnet ihren Code in einem Reiter |
| Klick auf „Eigenschaften“ | öffnet DIM, CONST und was sonst außerhalb der Methoden steht |
| Projekt → Methode entfernen | die Methode im vorderen Reiter |
| Kreuz am Reiter, mittlere Maustaste | schließt ihn (nichts geht verloren) |

Die Ereignisse je Art:

| Art | Ereignisse |
|---|---|
| Knopf, Sinnbild-Knopf, Kachel | Klick |
| Kästchen, Umschalter, Optionsfeld | Klick, Geaendert |
| Textfeld | Geaendert, Enter, FokusRein, FokusRaus |
| Textbereich | Geaendert, FokusRein, FokusRaus |
| Liste, Baum | Geaendert, Doppelklick (bei der Liste auch Enter) |
| Tabelle | Geaendert, Doppelklick (auch Enter), Zelle(zeile, spalte) |
| Regler, Zahlenfeld, Klappliste, Wähler, Reiterwerk | Geaendert |
| Zeichenfläche, Diagramm | Zeichnen, Maus, Ziehen(x, y), Rad(schritte) |
| Werkzeugleiste | Knopf(nr) |
| Statusleiste | Feld(nr) |
| jedes Control zusätzlich | MausRein, MausRaus, Abgelegt(zeile, ziel) |
| Timer, Menüeintrag | Aktion |
| Fenster | Geoeffnet, Schliessen, Groesse, Taste(taste), Takt |
| eigenes Control | Geoeffnet, Groesse, Takt |
| App | Geoeffnet, Beenden |

`Zeichnen` läuft in jedem Bild, in dem die Fläche zu sehen ist (nicht auf
einer Reiterseite im Hintergrund), mit der Fläche als Schnittrechteck und in
der Schrift der Oberfläche: der eigenen der gui (`GUI_FONT`), sonst der
eingebauten Sans in der Textgröße des Programms -- nicht in der
Pixelschrift, die `TEXT` sonst nimmt. Was `Zeichnen` an Schrift, Größe und
Stil ändert, gilt nur dort (wie `GFX_PUSH`/`GFX_POP`). Gezeichnet wird in Bildschirmpunkten ab `GUI_CANVAS_X/Y`. Maus, Rad und
Ziehen kommen nicht, solange ein modales Fenster vor der Fläche steht.

Ein **Diagramm** (Bibliothek, Gruppe Zeichnen) ist eine Zeichenfläche mit
einem Diagramm aus dem Modul [`chart`](module-chart.md): die Art (Kuchen,
Donut, Balken, Linie, Fläche, Tacho, Leiste, Lampen) und den Titel stellt der
Inspektor ein, der Entwurf zeigt ein Muster. Im Code ist das Control ein
`StudioDiagramm`, das sich in jedem Bild selbst zeichnet, so groß wie die
Fläche und im Thema der Oberfläche (hell oder dunkel, auch nach einem
Wechsel); ein eigenes `Zeichnen` kommt obendrauf.

```basic
Self.vorrat.Leeren()
Self.vorrat.Hinzufuegen("Obst", 5)
Self.vorrat.Hinzufuegen("Milch", 2, RGB(200, 60, 60))   ' eigene Farbe
Self.vorrat.Setzen("legende", "rechts")                 ' CHART_SET
Self.vorrat.SetzenSchalter("prozent", TRUE)             ' CHART_SET_FLAG
```

Dazu `Reihe(name, werte)` für Linien und gruppierte Balken, `Beschriften`,
`Wert` für Tacho, Leiste und Lampen, `Titel`, `SetzenZahl`, `Anzahl()`;
`Flaeche` ist die Zeichenfläche, `Diagramm` der CHART-Handle für alles
Weitere.

Ein Ereignis mit Klammern ruft seine Methode mit Werten: `Ziehen` mit der
Lage auf der Fläche (solange die Taste nach einem Druck auf die Fläche
hängt), `Rad` mit den Schritten des Mausrads, `Taste` mit dem Tastencode
(`KEY_A`, `KEY_SPACE` ...), solange das Fenster vorn ist und kein
Eingabefeld die Taste nimmt -- ein Kürzel (eine F-Taste, Strg oder Alt dazu)
kommt auch aus einem Eingabefeld an, außer das Feld braucht es selbst
(Strg+A/C/V/X/Z/Y, Pfeile, Pos1, Ende, Rück, Entf); ob Strg gedrückt ist,
fragt die Methode mit `KEYPRESSED(KEY_LCTRL)` --, `Knopf` mit der Nummer des gedrückten Knopfs
der Werkzeugleiste und `Feld` mit der Nummer des angeklickten Felds der
Statusleiste (ab 0, Trenner und Lücken zählen mit), `Zelle` mit
Datenzeile und Spalte jedes Klicks in eine Tabelle -- auch auf einen Knopf
in der Zelle (`GUI_TABLE_CELL_KIND "knopf"`) und auch in der schon gewählten
Zeile, wo `Geaendert` nicht mehr kommt; `Abgelegt` mit der gezogenen Zeile
der Quelle und der Zeile, auf der sie landete, wenn etwas auf ein Control
gezogen wurde, das im Inspektor „Ablage“ ist (die Quelle ist „Ziehbar“: bei
Liste, Baum und Tabelle die Zeile, sonst das Control; woher es kam, sagt
`GUI_DROP_SOURCE()`, was, `GUI_DROP_TEXT()`). `Takt` läuft in jedem Bild nach den
Ereignissen der Controls (die eines eigenen Controls vor denen des
Fensters) -- dort fragt der Code ab, wofür es kein Ereignis gibt, etwa ein
Kontextmenü aus `GUI_CONTEXT`. Die Knöpfe stehen im
Inspektor in einer Zeile, `Text|Sinnbild|Tipp` je Knopf, durch Semikolon
getrennt, `-` ist ein Trenner, `~` eine Lücke (`Neu|plus|Neues Produkt; -;
|rueckgaengig|Rückgängig`); die Felder als `Text|Breite`, dahinter
wahlweise `links`/`mitte`/`rechts` und `klick` für ein Feld, das `Feld`
meldet (Breite 0 teilt sich den Rest). Die Vorschläge kennen den ganzen Text des
Programms samt der erzeugten Basisklasse -- hinter `Self.` stehen also auch
die Controls, Timer und Menüeinträge des Fensters.

Über dem Code steht die Kopfzeile der Methode (`SUB knopf1_Klick()`), die
man nicht tippt; rechts daneben, wozu sie da ist. Das Code-Feld färbt,
schließt Klammern, rückt beim Zeilenumbruch ein und setzt das Ende eines
Blocks mit (`IF` → `END IF`). Strg+Z/Y gehören im Code-Reiter dem Code.

Wird ein Control im Inspektor umbenannt, ziehen seine Ereignis-Methoden mit,
und im Code des Fensters wird der alte Name als ganzes Wort ersetzt.

## Starten und Bauen

| Handgriff | wirkt so |
|---|---|
| F5, Starten | sichert, prüft, erzeugt `_studio/` und startet das Programm mit `dhrt run` |
| Umschalt+F5, Anhalten | beendet das laufende Programm |
| Umschalt+F7, Prüfen | prüft das ganze Projekt, ohne zu starten |
| Strg+B, Bauen | erzeugt `_studio/` und schreibt mit `dhrt --export` eine eigenständige Exe nach `bau/` |

Was das Programm ausgibt, steht in der Leiste **Ausgabe** unten; daneben
liegen die **Probleme**. Geprüft wird auch ohne Zutun, einen Augenblick nach
jeder Änderung. Ein Befund steht an der Methode, in der er liegt
(`Hauptfenster.knopf1_Klick, Zeile 3`), dort als Wellenlinie, Marke und Text
am Zeilenende; ein Klick in der Liste öffnet die Methode an der Stelle. Was
der Übersetzer erst am Ende einer Methode bemerkt (ein offenes `IF`), steht
„am Ende“. Ein Fehler hält den Start an. Ein Laufzeitfehler kommt ebenso an
seine Methode.

In `_studio/` (nicht ins Repository, siehe `.gitignore`) erzeugt das Studio:

| Datei | was darin steht |
|---|---|
| `main.dh` | der Einstieg: bindet alles ein und ruft `studioLauf` mit App und Startfenster |
| `studio_laufzeit.dh` | `StudioApp`, `StudioFenster` und die Bildschleife |
| `<Fenster>_basis.dh` | je Fenster die Klasse `<Fenster>_Basis`: Controls als Felder, Aufbau, Verdrahtung der Ereignisse, leere Ereignis-Methoden |
| `projekt/<Element>.dh` | der Code des Projekts, so wie er im Editor steht |

Die Nutzerklasse `<Fenster> EXTENDS <Fenster>_Basis` überschreibt die
Ereignis-Methoden. Ein Control ist dort ein Feld (`Self.knopf1`), das Fenster
der gui liefert `Self.Fenster()`; `Self.Zeigen()` und `Self.Zumachen()`
öffnen und schließen ein Fenster, das Startfenster zu schließen beendet das
Programm. Die Basisklasse baut das Fenster aus dem Layout als JSON
(`GUI_FROM_JSON`) statt Aufruf für Aufruf -- so sieht es genau so aus wie im
Entwurf. Ohne Studio startet man das Programm mit
`dhrt run _studio/main.dh`.

## Was ein Programm braucht

**Timer.** In der Bibliothek unter „Unsichtbar“. Ein Timer liegt nicht im
Fenster, sondern in einer Leiste darunter (wie in Xojo); ein Klick wählt
ihn, der Inspektor zeigt Name, Intervall, „Läuft beim Start“ und
„Wiederholen“, Entf löscht ihn. Sein Ereignis heißt `<Name>_Aktion`. Im Code
ist er ein Feld der Fensterklasse mit `Intervall`, `Aktiv`, `Wiederholen`,
`Starten()` und `Stoppen()`.

**Menüs.** Projekt → Menüs des Fensters (Strg+Umschalt+U) öffnet den
Menü-Editor: links der Baum, rechts Text, Name und Kürzel (`Strg+S`, `F5`
...), dazu „Mit Haken“. Ein neuer Eintrag bekommt einen Namen aus seinem
Text (`miOeffnen`), der dem Text folgt, bis man ihn selbst ändert. Erst OK
schreibt die Menüs ins Layout -- ein Rückgängig-Schritt; umbenannte Einträge
nehmen ihre Ereignis-Methoden mit. Benannte Einträge stehen im Navigator
unter dem Fenster, ihr Ereignis heißt `<Name>_Aktion`.

**Kontextmenüs** legt derselbe Editor an: [+ Kontextmenü], darunter Einträge
wie in einem Menü. Sein Name ist ein Feld des Fensters (das Handle aus
`GUI_CONTEXT`), unter „An“ steht das Widget, auf dessen Rechtsklick es
aufgeht -- eine Liste, ein Baum, eine Tabelle wählen dabei ihre Zeile --,
oder das ganze Fenster. Auch ein eigenes Control hat Kontextmenüs (eine
Menüleiste nicht). Im Layout stehen sie unter `kontextmenues`.

**Dialoge, die warten.** Die Laufzeit des Projekts bietet:

| Aufruf | was er tut |
|---|---|
| `Meldung(text$ [, titel$])` | eine Meldung mit OK |
| `Frage(text$ [, knoepfe$ [, titel$]])` | Knöpfe mit `\|` getrennt (Vorgabe `Ja\|Nein`), liefert die Nummer des Knopfs; Enter = der erste, ESC = der letzte |
| `Eingabe(text$ [, vorgabe$ [, titel$]])` | fragt nach einem Text, `""` bei Abbrechen |
| `Auswahl(text$, eintraege [, vorgabe [, titel$]])` | einer aus einer Liste (Klappliste); liefert den Index, `-1` bei Abbrechen |
| `EingabeZahl(text$, vorgabe, min, max [, titel$])` | fragt nach einer ganzen Zahl; liefert `(TRUE, zahl)` oder bei Abbrechen `(FALSE, vorgabe)` |
| `FarbeWaehlen(vorgabe [, titel$])` | eine Farbe im Farbwähler; liefert `&HRRGGBB`, `-1` bei Abbrechen |
| `fenster.ZeigenModal()` | zeigt ein Fenster modal und kehrt erst zurück, wenn es geschlossen ist |
| `Bild(name$)` | ein Bild aus `bilder/` des Projekts als IMAGE |

Alle vier kehren erst mit der Antwort zurück, wie in Xojo oder Visual
Basic. Das geht, weil die erzeugte Basisklasse die Ereignisse **nach** dem
`GUI_UPDATE` abfragt, statt sie als Rückruf der gui laufen zu lassen -- eine
Ereignis-Methode darf darum selbst Bilder drehen. Endet das Programm
währenddessen, liefert `Frage` 0. Timer laufen auch während eines Dialogs.

**Bilder.** Projekt → Bild hinzufügen (Strg+Umschalt+B) kopiert eine Datei nach `bilder/` im
Projektordner; ein Bild-Control wählt seins im Inspektor unter „Bild“ und
zeigt es schon im Entwurf. Beim Starten und Bauen kommen die Bilder nach
`_studio/assets/bilder/`, und `dhrt --export` nimmt `assets/` neben die Exe.

**Sprachen.** Ein Projekt schreibt seine Texte in einer Sprache, der seiner
Quelle. Eine Tafel `sprache/<code>.txt` im Projektordner macht eine zweite
daraus: je Zeile der Text, wie er im Projekt steht, ein Tabulator, die
Übersetzung (`sprache/en.txt`). Von selbst übersetzt werden die Texte der
Fenster (Titel, Beschriftungen, Platzhalter, Tooltips, Spaltenköpfe,
Einträge, Reiter, Meldungen der Prüfregeln, Diagrammtitel, Werkzeugleisten),
die Menüs samt Kürzeln (`Strg+S` wird `Ctrl+S`), die Texte der Berichte und
die Knöpfe der Dialoge oben. Im Code geht jeder Anzeigetext durch `tr$`:

| Aufruf | was er tut |
|---|---|
| `SpracheWaehlen(code$)` | wählt die Sprache (`""` = die der Quelle); `FALSE`, wenn es keine Tafel `sprache/<code>.txt` gibt |
| `Sprache$()` | die gewählte Sprache, `""` = die der Quelle |
| `tr$(text$)` | der Text in der gewählten Sprache; was die Tafel nicht kennt, bleibt |
| `trf$(text$, werte)` | ein Text mit `{0}`, `{1}` ...: erst übersetzt, dann eingesetzt |

Gewählt wird vor dem ersten Fenster, in `App.Geoeffnet`; ohne Wahl ist es
Englisch, wenn `DHRT_LANG` englisch ist und die Tafel `en.txt` da ist. Ein
Wechsel gilt für Fenster, die danach aufgebaut werden. Was das Programm aus
einem Widget zurückliest (der Text einer Klappliste), ist die Übersetzung --
verglichen wird mit `tr$("...")`. Mit `DH_STUDIO_SPRACHLUECKEN=datei`
schreibt die Laufzeit jeden Text, der in der Tafel fehlt, einmal in die
Datei.

**Debugger.**

| Handgriff | wirkt so |
|---|---|
| Klick in die Nummernspalte, F9 | Haltepunkt an der Zeile an/aus (roter Punkt) |
| F7, Debuggen | erzeugt `_studio/` und startet das Programm unter `dhrt debug` |
| F5 | weiter, solange es hält |
| F10, F11, Umschalt+F11 | Schritt, Schritt hinein, Schritt heraus |
| Umschalt+F5 | beendet das Programm |

Hält es, geht der Code-Reiter der Methode auf, die Zeile ist markiert, und
die Leiste **Debugger** unten zeigt die Variablen (bei `Self` die Felder des
Fensters) und den Aufrufstapel. Die Stelle rechnet dieselbe Zeilenkarte um,
die auch Fehler an ihre Methode bringt.

## Fenster entwerfen

| Handgriff | wirkt so |
|---|---|
| Baustein in der Bibliothek anklicken, dann ins Fenster klicken | legt ihn dort ab (die Mitte an der Klickstelle) |
| Baustein ins Fenster ziehen | legt ihn dort ab, wo man loslässt |
| Doppelklick auf einen Baustein | legt ihn mitten ins Fenster |
| Klick auf ein Control | wählt es; Umschalt- oder Strg-Klick nimmt dazu oder weg |
| Rahmen über leere Fläche ziehen | wählt alles, was ganz darin liegt |
| Control ziehen | verschiebt die ganze Auswahl |
| an einem der acht Griffe ziehen | ändert die Größe |
| an der rechten oder unteren Kante oder der Ecke des Fensters ziehen | ändert die Größe des Fensters; verankerte Controls gehen mit (ebenso, wenn man Breite oder Höhe im Inspektor tippt) |
| Pfeiltasten | schieben um einen Punkt, mit Umschalt um das Raster (8) |
| Entf | löscht die Auswahl |
| Strg+D | verdoppelt |
| Strg+A | wählt alles (nicht, solange ein Eingabefeld den Fokus hat) |
| Strg+Z / Strg+Y | Rückgängig / Wiederholen |
| Strg+Mausrad, Strg+Plus/Minus, Strg+0 | Zoom |
| Esc | bricht ein Ablegen ab, sonst hebt es die Auswahl auf |

Ein Fenster, das bei 100 % nicht in die Mitte passt, geht bei 75 oder
50 % auf; was über die Mitte hinausreicht, wird abgeschnitten, statt über
Inspektor und Ausgabe zu liegen. Die Höhe im Inspektor ist die des
Fensters samt Titelleiste (ein Control unten muss also über ihr enden; nur
das Startfenster läuft im Programm ohne Titelleiste). Hat das Fenster Menüs,
steht die Menüleiste über dem Inhalt: im Programm wird das Fenster um die
Leiste höher.

**Ausrichtungslinien:** Beim Ziehen und beim Ändern der Größe rasten die
Kanten und Mitten an denen der anderen Controls ein. Einrasten gibt es auch
an den Rändern und der Mitte des Fensters, 20 Punkte vom Rand und 12 Punkte
neben einem Nachbarn. Eine rosa Linie zeigt, woran. Wo keine Linie greift,
gilt das Raster. Mit gehaltener Alt-Taste rastet nichts ein. Beides lässt
sich unter Ansicht abschalten.

**Anordnen** (Menü und Werkzeugleiste) richtet die Auswahl am zuletzt
angeklickten Control aus: Kanten, Mitten, gleiche Breite, Höhe oder Größe.
Verteilen setzt ab drei Controls gleiche Lücken. Nach vorn und nach hinten
ändern die Reihenfolge, in der gezeichnet wird.

**Layouts** (Bibliothek, Gruppe Behälter: Zeile, Spalte, Raster) ordnen
ihre Controls selbst: nebeneinander, untereinander oder in Zeilen und
Spalten, mit Abstand und Rand. Im Entwurf ist ein Layout gestrichelt
umrandet, ein leeres sagt „Controls hierher ziehen“. Ein Baustein, der auf
einem Layout landet, kommt hinein, ebenso die Controls, die man
hineinzieht -- mehrere gewählte zusammen, von links nach rechts bzw. von
oben nach unten; eine blaue Marke zeigt die Stelle. Ein Kind wird nicht
verschoben, sondern umgesetzt: es bleibt stehen, sein Umriss folgt der Maus,
und losgelassen kommt es an die Stelle der Marke -- neben jedem Layout
losgelassen steht es danach frei. Das gilt auch für mehrere gewählte Kinder
auf einmal. Die Pfeiltasten rücken ein Kind eine Stelle weiter. Ein
**Abstandhalter** (ebenfalls Gruppe Behälter) ist freier Platz in einem
Layout: mit Gewicht 1 nimmt er den Rest und schiebt, was danach kommt, ans
Ende. Im Entwurf ist er eine Feder zwischen den Nachbarn; ein Klick wählt
ihn, der Inspektor zeigt sein Gewicht, die Pfeiltasten rücken ihn, Entf
löscht ihn. Verdoppeln nimmt bei einem Layout die Kinder mit, Löschen
lässt sie frei stehen. Im Navigator stehen die Kinder unter ihrem Layout.
Wer das Layout im Inspektor verankert (etwa links, rechts, oben), lässt es
mit dem Fenster wachsen, und es verteilt seine Kinder neu -- im erzeugten
Programm legt die Laufzeit sie genauso wie im Entwurf.

## Der Inspektor

Die Eigenschaften stehen in Gruppen, die auf- und zuklappen: Kennung, Lage,
Layout, Inhalt, Aussehen, Verhalten. Zu sehen ist nur, was zur Art des
Controls passt: ein Layout hat seine Anordnung, Spalten (beim Raster),
Abstand, Rand, Ausrichtung, „Kinder dehnen“ und einen Rahmen; ein Kind
eines Layouts hat statt Links, Oben und Anker sein Gewicht (0 = eigene
Größe, sonst sein Anteil am freien Platz).
Jede Eigenschaft hat ihren Editor:
- Text und Zahlen in Feldern;
- Schalter als Kippschalter;
- Auswahlen als Klappliste;
- Farben im Farbwähler, der als Popover aufgeht ("Vorgabe des Themas" nimmt
  die Farbe wieder weg);
- der Anker als vier Kästchen (an welchen Rand das Control klebt, wenn das
  Fenster wächst).

Eine Änderung wirkt sofort im Fenster und gilt für alle gewählten Controls,
die die Eigenschaft haben. Ein Rückgängig-Schritt entsteht, wenn das Feld den
Fokus abgibt oder Enter kommt, nicht je Anschlag. Ein Name muss ein
Bezeichner sein und im Fenster eindeutig, denn er wird später zum Feld der
Fensterklasse; ein ungültiger Name wird rot markiert und nicht übernommen.

Ohne Auswahl zeigt der Inspektor das Fenster, dazu **Standard-Knopf (Enter)**
und **Abbrechen-Knopf (Esc)**: eine Klappliste mit den Knöpfen des Fensters.
Im Programm drückt Enter dann den einen und Esc den anderen -- sein
`_Klick` läuft wie bei einem Mausklick (Enter auch aus einem einzeiligen
Textfeld; ein Textbereich behält sein Enter). Gemerkt wird der Knopf beim Namen; wird er umbenannt, zieht die
Wahl mit, wird er gelöscht, fällt sie weg.

## Eigene Controls

Aus mehreren Controls wird ein eigener Baustein, wie ein ContainerControl in
Xojo: Projekt -> „Eigenes Control hinzufügen …“ legt ein Element an, das wie
ein Fenster entworfen wird -- nur ohne Titel und Menüs (Timer gehen). Es hat
eigenen Code und Ereignisse für seine Teile (`knopf_Klick`), dazu die
Ereignisse `Geoeffnet` und `Groesse` des Controls selbst.

In der Bibliothek stehen alle eigenen Controls unter **Projekt**. Man legt
sie ins Fenster wie einen Knopf und bekommt eine Instanz (`zaehler1`). Sie
lässt sich verschieben, größer ziehen, verankern und in Zeilen, Spalten und
Raster legen; im Entwurf zeigt sie ihren echten Inhalt, gestrichelt
umrandet, und ein Klick auf einen ihrer Teile wählt die ganze Instanz.

```basic
' im Code des Controls Zaehler
SUB knopf_Klick()
    Self.n = Self.n + 1
    GUI_SET_TEXT(Self.anzeige, STR$(Self.n))
END SUB

FUNCTION Wert() AS INTEGER
    RETURN Self.n
END FUNCTION

' im Code des Fensters
SUB knopfSumme_Klick()
    Meldung(STR$(Self.zaehler1.Wert() + Self.zaehler2.Wert()))
END SUB
```

Im Code des Fensters ist eine Instanz ein Objekt der Klasse des Controls:
ihre Methoden und Felder sind erreichbar (`Self.zaehler1.Wert()`,
`Self.zaehler1.anzeige`). Dazu bringt jedes Control `Fenster()`,
`Behaelter()`, `Breite()`, `Hoehe()` und `Sichtbar(an)` mit.

**Wie es läuft:** eine Instanz ist im Programm ein leerer Layout-Behälter --
er nimmt die Anker, das Gewicht im Layout und die Lage --, und die Teile des
Controls stehen im selben Fenster dahinter. Je Bild legt die Laufzeit sie
dorthin, wo der Behälter gerade liegt, nach ihren Ankern im Control: ein
Teil mit Anker links und rechts wächst mit der Instanz.

**Eigene Ereignisse und Eigenschaften.** Im Inspektor des Controls stehen
unter „Eigenes Control“ zwei Felder: **Ereignisse** (Namen durch Semikolon,
`Erreicht; Geleert`) und **Eigenschaften** (`Ziel AS INTEGER = 3; Vorsatz =
"Anzahl: "` -- ohne `AS` ein STRING, dazu FLOAT und BOOLEAN; Anführungszeichen
halten Leerzeichen am Rand). Jede Eigenschaft ist ein Feld der Klasse des
Controls mit dieser Vorgabe. Eine Instanz hat im Inspektor das Feld
**Werte** (`Ziel = 2; Vorsatz = "Erste: "`); ein Name, den das Control nicht
kennt, oder ein Wert, der nicht zum Typ passt, fällt weg.

```basic
' im Code des Controls Zaehler
SUB knopf_Klick()
    Self.n = Self.n + 1
    IF Self.n = Self.Ziel THEN Self.Ausloesen("Erreicht")
END SUB

' im Code des Fensters
SUB zaehler1_Erreicht()
    Meldung("Ziel erreicht")
END SUB
```

`Self.Ausloesen("Name")` meldet eines der Ereignisse; das Fenster ruft dann
seine Methode `<Instanz>_<Ereignis>` -- nach dem Takt des Controls im selben
Bild. Ein Doppelklick auf die Instanz bietet ihre Ereignisse an wie bei
jedem anderen Control.

**Controls in Controls.** Ein Control darf Instanzen anderer Controls
enthalten, wie ein Fenster. Ihre Ereignisse kommen im äußeren Control an
(`z1_Erreicht` im Code von `Zwei`), und das kann sie mit `Ausloesen` weiter
ans Fenster melden; im Fenster heißt es dann `Self.zwei1.z1.Wert()`. Die
Bibliothek bietet in einem Control die anderen an, aber nicht das Control
selbst und keines, das es schon enthält -- das wäre ein Kreis. Steht doch
einer in der Datei (von Hand geschrieben), bleibt die Instanz ein leerer
Platz, und der Entwurf sagt „… enthält sich selbst“. Wird ein Control
umbenannt, nehmen alle Instanzen in den anderen Fenstern und Controls den
neuen Namen mit, auch in deren Rückgängig-Ständen. Löschen geht erst, wenn
keine Instanz mehr darauf zeigt; sonst nennt die Statuszeile, wo noch eine
steht.

## Datenbanken

Projekt → Datenbank hinzufügen legt eine SQLite-Datenbank als Element an.
Ihr Aufbau steht in `<Name>.dhdb` (JSON) im Projekt, die Daten in einer
SQLite-Datei im Projektordner (Vorgabe `<name>.sqlite`, oben rechts im
Reiter änderbar). Im Navigator stehen unter der Datenbank ihre Tabellen; ein
Klick öffnet den Reiter der Datenbank: links die Tabellen mit Neu,
Umbenennen und Löschen, rechts entweder ihre **Spalten** oder ihre
**Daten**.

| Art | in SQLite | getippt |
|---|---|---|
| Text | TEXT | wie er ist |
| Ganze Zahl | INTEGER | `-4` |
| Kommazahl | REAL | `2,5` oder `2.5` |
| Geld | INTEGER in Cent | `19,99` (steht als 1999 in der Datei) |
| Datum | TEXT | `2026-11-03` |
| Ja/Nein | INTEGER 0/1 | `ja`/`nein` |
| Bild | BLOB (die Bytes der Bilddatei) | mit **Bild …** aus einer Datei |

Jede Tabelle hat die Spalte `id INTEGER PRIMARY KEY` von selbst -- die
Datenbindung (`GUI_FORM_LOAD`/`GUI_FORM_SAVE`) braucht genau sie. Im
Spalten-Gitter benennt man eine Spalte in ihrer Zelle um, wählt die Art aus
der Liste und kreuzt **Pflicht** (NOT NULL; leer gilt dann der Leerwert der
Art oder die Vorgabe) und **Eindeutig** (ein eigener Index) an. Im
Daten-Gitter tippt man Werte in die Zellen (auch mehrere auf einmal mit
Strg+V); leer ist NULL, außer bei einer Pflichtspalte. Angezeigt werden die
ersten 1000 Zeilen.

**Jede Änderung gilt sofort für die Datei im Projektordner**, in einer
Transaktion: Geht sie nicht (ein Eindeutig auf doppelte Werte, ein Wert, der
nicht zur Art passt), steht die Meldung in der Statuszeile, und Datei und
Aufbau bleiben, wie sie waren. Eine Spalte zu ändern baut die Tabelle neu
auf und kopiert ihre Zeilen um (SQLite kann eine Spalte nicht ändern);
Spalten und Tabellen, die das Programm selbst angelegt hat, bleiben dabei
stehen. Löschen und Umbenennen lassen sich nicht zurücknehmen.

**Eine Datei, die es schon gibt** (aus einem anderen Programm), nimmt
Projekt → Datenbank aus Datei übernehmen: die Datei kommt in den
Projektordner (liegt sie schon dort, bleibt sie), ihr Aufbau wird das
Schema -- Arten, Pflicht, Vorgaben, Eindeutig (UNIQUE über eine Spalte) und
Verweise (FOREIGN KEY auf eine `id`). Was keine Art des Studios genau trifft
(`DATE`, `TIMESTAMP`, `DEFAULT CURRENT_TIMESTAMP`, eine
Pflichtspalte ohne Vorgabe), steht wörtlich daneben und gilt, bis man Art
oder Vorgabe im Gitter ändert; so baut der Abgleich danach keine Tabelle
neu auf, und die Datei bleibt Zeichen für Zeichen, wie sie war. Was das
Schema nicht fassen kann (UNIQUE über zwei Spalten, eine Tabelle ohne `id`),
bleibt in der Datei und steht in einer Meldung; eine Tabelle ohne `id` zeigt
und bearbeitet der Reiter trotzdem, binden lässt sie sich nicht.

Im Programm ist die Datenbank eine globale Verbindung unter ihrem Namen,
offen, bevor die App startet:

```text
DIM r AS DB_RESULT : r = DB_QUERY(Daten, "SELECT name FROM kunden ORDER BY name")
```

Aus dem Studio gestartet (F5) arbeitet das Programm auf der Datei im
Projektordner -- was es schreibt, steht danach im Daten-Gitter. Bauen legt
die Datei neben das gebaute Programm, wenn dort noch keine liegt; eine, in
der schon gearbeitet wurde, bleibt. Das gebaute Programm legt fehlende
Tabellen, Spalten und Indexe selbst an und benennt um, was im Studio
umbenannt wurde -- der Reihe nach, solange es dort den alten Namen noch
gibt und den neuen nicht (eine Umbenennung nur in Groß/klein zählt nicht,
SQLite sieht darin denselben Namen). Gelöschtes lässt es stehen. Das
Element zu löschen nimmt nur den Aufbau aus dem Projekt; die Datei mit den
Daten bleibt liegen.

**Fenster an eine Tabelle binden.** Im Navigator stehen unter jeder Tabelle
ihre Spalten. Eine Spalte oder die ganze Tabelle ins offene Fenster gezogen
legt je Spalte eine Beschriftung und ein passendes Feld an, an die Spalte
gebunden: Text, Zahl und Geld als Textfeld (Geld mit Komma), Datum als
Textfeld für `JJJJ-MM-TT`, Ja/Nein als Kästchen. Ein Doppelklick auf eine
Spalte legt ihr Feld unter das letzte Control. Hat das Fenster noch keine
Tabelle, bekommt es diese; im Inspektor steht sie in der Gruppe **Daten**
(Tabelle am Fenster, Spalte an jedem Feld).

| Methode | tut |
|---|---|
| `Laden(id)` | holt die Zeile in die Felder, `DatensatzId` ist dann `id` |
| `Sichern()` | prüft die Regeln der Felder und schreibt (neue Zeile bei `DatensatzId` 0) |
| `Neu()` | leert die Felder und setzt die Vorgaben der Spalten |
| `Loeschen()` | löscht die Zeile, die gerade dasteht |
| `Ungesichert()` | hat sich seit Laden oder Sichern etwas geändert? |

Eine Liste, eine Tabelle oder eine Klappliste zeigt die Zeilen einer Tabelle,
wenn im Inspektor ihre **Datenquelle** gewählt ist; **Angezeigte Spalten**
nennt welche (durch Semikolon; leer: bei einer Tabelle alle, sonst die
erste). Das Fenster füllt sie beim Öffnen; `Neuladen()` holt sie neu, und
`IdVon(liste)` liefert die id der gewählten Zeile. Zeigt eine Liste die
Tabelle des Fensters, lädt ein Klick auf eine Zeile sie in die Felder, und
nach Sichern und Löschen steht die Liste schon richtig.

```text
SUB knopfNeu_Klick()
    Self.Neu()
END SUB

SUB knopfSichern_Klick()
    IF NOT Self.Sichern() THEN Meldung("Bitte die Eingaben prüfen.")
END SUB
```

**Verweise.** Im Spalten-Gitter nennt **Verweis auf** eine andere Tabelle
derselben Datenbank: die Spalte hält dann die id einer ihrer Zeilen (sie
wird eine ganze Zahl). Ins Fenster gezogen wird eine solche Spalte eine
Klappliste, die die erste Textspalte der Zieltabelle zeigt und die id
bindet. In den **Angezeigten Spalten** einer Liste darf auch eine Spalte der
Zieltabelle stehen: `nr;kunde.name` zeigt neben der Nummer den Namen des
Kunden, dessen id in `kunde` steht. Das Studio legt dafür einen LEFT JOIN um
die Tabelle (eine Zeile ohne Kunden bleibt, mit leerem Namen); die Spalte
heißt im Ergebnis `kunde.name`, und unter diesem Namen kennen sie auch
**Sortieren nach**, die **Bedingung** (`"kunde.ort" = 'Köln'`) und das
Suchfeld.

**Beim Löschen** macht aus dem Verweis einen Fremdschlüssel (`REFERENCES …
(id)`) und sagt, was mit einer Zeile geschieht, wenn die Zeile gelöscht wird,
auf die sie zeigt: *verbieten* (das Löschen scheitert, solange etwas auf sie
zeigt), *mitlöschen* (`ON DELETE CASCADE`: die Posten gehen mit ihrer
Rechnung) oder *leeren* (`ON DELETE SET NULL`, nicht bei einer
Pflichtspalte). Leer bleibt es ein Verweis ohne Regel. Der Abgleich baut die
Tabelle dafür um, die Zeilen bleiben -- auch ein Verweis ins Leere, er gilt
erst für Änderungen danach. dhrt hat Fremdschlüssel immer an; ein
gebautes Programm legt eine neue Datei so an, eine ältere ergänzt es nicht
(das tut der Abgleich im Studio). Eine übernommene Datei bringt ihr
`ON DELETE` mit.

**Bilder.** Eine Spalte der Art **Bild** hält die Bytes einer Bilddatei
(PNG, JPG, BMP oder GIF). Im Daten-Gitter steht ihre Größe; **Bild …** setzt
das Bild der gewählten Zelle aus einer Datei, Entf macht sie leer. Ins
Fenster gezogen wird eine Bildspalte ein Bild-Control an dieser Spalte
(Inspektor: **Bildspalte**; sie hängt nicht an der Bindung des Formulars).
Es zeigt das Bild mit `Laden(id)`; `BildSetzen(control, pfad)` und
`BildLeeren(control)` ändern es, und mit `Sichern()` kommt es in die Datei.
Eine Tabelle mit Datenquelle zeigt eine Bildspalte als Bild in der Zelle.

**Bedingung, Sortierung, Suche.** Eine Liste mit Datenquelle hat im
Inspektor dazu **Sortieren nach** (eine Spalte der Quelle) samt
**Absteigend**, eine **Bedingung** (SQL wie hinter WHERE, etwa
`ort <> 'Bonn'`; das Studio probiert sie an der Datei im Projektordner und
zeigt einen Fehler am Feld -- auch einen Namen in doppelten
Anführungszeichen, den es in der Quelle nicht gibt: SQLite nähme ihn sonst
still als Text, und die Liste bliebe leer) und ein **Suchfeld** (ein Textfeld des Fensters:
sobald darin getippt wird, zeigt die Liste nur Zeilen, in deren angezeigten
Spalten der Text vorkommt).

**Abfragen.** [Abfrage …] im Reiter der Datenbank legt ein benanntes SELECT
an; links stehen die Abfragen unter den Tabellen, rechts ihr SQL und das
Ergebnis (die ersten 1000 Zeilen). Gesichert wird ein SQL erst, wenn es an
der Datei geht -- sonst bleibt der Stand davor. Eine Abfrage ist eine
Datenquelle wie eine Tabelle (mit einer Spalte `id` liefert `IdVon` sie, sonst
0); aus dem Navigator ins Fenster gezogen wird sie eine Tabelle. Im Code läuft
sie über ihren Namen, auch mit dem der Datenbank davor:

```text
DIM r AS DB_RESULT : r = Abfrage("koelner")
WHILE DB_NEXT(r)
    PRINT DB_GET_STRING(r, 1)
WEND
```

## Berichte

Ein **Bericht** ist eine gedruckte Seite aus den Daten des Projekts, etwa
eine Rechnung oder eine Liste (Projekt → Bericht hinzufügen …). Seine
Beschreibung steht in `<Name>.dhreport` (JSON); wie sie aufgebaut ist und
was noch kommt, steht im [Entwurf](entwurf-berichte.md). Ein Bericht hat
**Bänder** untereinander: Berichtskopf, Seitenkopf, Gruppenkopf, Detail (je
Zeile der Quelle), Gruppenfuß, Seitenfuß, Berichtsfuß. Darin stehen
Elemente in mm: fester Text, Felder (eine Spalte der Quelle, auch
`kunde.name` über einen Verweis), Summen (Summe, Anzahl, Mittel, Min, Max je
Gruppe oder Bericht), Seitenzahl („Seite {n} von {m}“), Datum, Linien,
Rechtecke und Bilder.

Die **Quelle** ist eine Tabelle oder Abfrage einer Datenbank des Projekts;
die **Bedingung** darf Parameter haben (`rechnung = :id`). Passt ein Band
nicht mehr auf die Seite, kommt es auf die nächste, mit Seitenfuß davor und
Seitenkopf danach.

Der Reiter eines Berichts ist sein **Entwurf**: die Seite in Millimetern,
jedes Band mit einer Leiste, die seinen Namen trägt, darin die Elemente.

* **Ablegen:** oben die Leiste der Elemente (Text, Feld, Summe, Seitenzahl,
  Datum, Linie, Rechteck, Bild) -- eine Art wählen, dann in ein Band klicken
  (Vorgabegröße) oder ein Rechteck ziehen. Esc bricht ab.
* **Spalten der Quelle** stehen rechts oben; eine Spalte in ein Band ziehen
  macht im Gruppen- oder Berichtsfuß eine Summe, im Seitenkopf eine
  Überschrift, sonst ein Feld (Geld und Zahlen rechtsbündig); eine Bildspalte
  wird ein Bild-Element. Ein Doppelklick legt sie ins Detail.
* **Ziehen** verschiebt, was gewählt ist (Raster 1 mm, Alt ohne); dabei rasten
  die Kanten und Mitten an anderen Elementen und an den Rändern ein, eine
  rosa Linie zeigt es. Ein einzelnes Element darf das Band wechseln. Acht
  Griffe ändern die Größe, die Unterkante eines Bandes seine Höhe.
* **Wählen:** Klick, mit Umschalt oder Strg dazu, ein Rahmen über leere
  Fläche wählt alles, was er berührt; ein Klick auf die Leiste eines Bandes
  wählt das Band, ein Klick neben alle Bänder den Bericht. Pfeiltasten
  schieben um 1 mm (mit Umschalt 5 mm), Entf löscht, Strg+D verdoppelt,
  Strg+A wählt alle Elemente, Strg+Z/Strg+Y nehmen zurück und wiederholen.
* **Eigenschaften** darunter gehören dem, was gewählt ist: beim Element Lage
  und Maße in mm, Text, Spalte, Format, Funktion einer Summe und worüber sie
  rechnet (Gruppe oder Bericht), „Nur Zeilen mit“ (unten), Schrift, Größe, fett, kursiv, Farbe (`#RRGGBB`), Ausrichtung, Strich,
  Füllung, Bild; beim Band die Höhe und bei einer Gruppe ihre Spalte, dazu
  „Band entfernen“ bzw. „Gruppe entfernen“; beim Bericht Titel, Quelle,
  Bedingung, Sortierung (`a; b`), Papier, Hochformat und Ränder (`o; r; u; l`
  oder eine Zahl für alle), dazu „Band dazu“ (was fehlt) und „Gruppe dazu“
  (innen, mit Kopf und Fuß). Text übernimmt Enter oder das Verlassen des
  Feldes, eine Auswahl gilt sofort; sind mehrere Elemente gewählt, gilt der
  Wert für alle, zu denen er passt.

**Nur Zeilen mit** schränkt ein Element auf Zeilen mit einem Wert ein:
`mwst = 19` (auch `<>`, `<`, `>`, `<=`, `>=`; Text darf in Anführungszeichen
stehen, Zahlen werden als Zahlen verglichen). Eine Summe rechnet dann nur über
diese Zeilen, jedes andere Element erscheint nur, wenn sein Band mindestens
eine solche Zeile hat. So steht in einer Rechnung die Steuer je Satz: im
Berichtsfuß „MwSt 19 %“ und eine Summe der Steuer, beide mit `mwst = 19`,
darunter dasselbe für 7 % -- eine Rechnung ohne Positionen zu 7 % zeigt diese
Zeile nicht. Was sich nicht so lesen lässt, nimmt das Feld nicht an.

Strg+Rad zoomt, das Rad rollt (mit Umschalt seitwärts). **Vorschau …** setzt
den Bericht mit den Daten der Entwicklungsdatei, fragt nach den Parametern und
blättert durch die Seiten.

Ein **Bild-Element** zeigt eine Datei aus `bilder/` (Eigenschaft Bild) oder,
wenn es eine **Spalte** nennt, je Zeile das Bild aus dieser Bildspalte -- etwa
ein Foto je Artikel im Detail. Ein Bild wird ins Rechteck eingepasst, ohne es
zu verzerren; eine leere Zelle oder Bytes, die kein Bild sind, lassen es
leer.

Im Programm ist ein Bericht ein Objekt unter seinem Namen:

```text
Rechnungsbericht.Pdf("rechnung.pdf", {"id": 5})
Rechnungsbericht.Drucken({"id": 5})
Rechnungsbericht.Vorschau({"id": 5})
DIM p AS PDF : p = Rechnungsbericht.Erzeugen({"id": 5})
```

`Vorschau` zeigt ein Fenster mit den Seiten, Drucken, „Als PDF …“ und
Schließen und kehrt zurück, wenn es zu ist. Fehlt ein Parameter, ist das ein
Fehler mit seinem Namen.

## Ein einzelnes Fenster

Eine `.dhform` (dasselbe Format, das `GUI_SAVE` schreibt und `GUI_LOAD`
liest) geht auch ohne Projekt auf, aus der IDE oder aus einem
Programm; Controls ohne Namen bekommen beim Öffnen einen. Solange nur das
Fenster bearbeitet wird, schreibt Sichern genau diese Datei. Kommt Code oder
ein weiteres Element dazu, wird daraus ein Projekt, und Sichern fragt nach
einem Ordner. Neu, Öffnen und Beenden fragen bei ungesicherten Änderungen
nach.

## Das Beispielprojekt

`examples/studio/rechnungen/` ist eine kleine Rechnungsverwaltung (nach dem
Piloten `examples/196_rechnungen.dh`) als Studio-Projekt: Datei → Öffnen, den
Ordner wählen, F5.

| Element | Art | Was es tut |
|---|---|---|
| App | App | füllt die Datenbank beim ersten Start, bevor das erste Fenster aufgeht |
| Daten | Datenbank | die Tabellen in `rechnungen.db` mit Verweisen, dazu die Abfrage `Belegzeilen` (Positionsnummer, Netto, Steuer und Brutto je Zeile, Anschrift) |
| Datenbank | Modul | Beispieldaten beim ersten Start; nächste Rechnungsnummer |
| Ausgabe | Modul | Datum und Beträge für die Anzeige, CSV-Export |
| Rechnung | Klasse | eine Rechnung samt Positionen: laden, rechnen, sichern (eine Transaktion), als PDF und in der Vorschau über den Bericht |
| Hauptfenster | Fenster | alle Rechnungen mit Filterzeile, Menü mit Kürzeln, Neu/Bearbeiten/PDF/Vorschau/Löschen |
| RechnungFenster | Fenster | eine Rechnung anlegen oder ändern, modal |
| KundenFenster, ArtikelFenster | Fenster | Stammdaten über die Datenbindung (Inspektor: Bindung) |
| Rechnungsbeleg | Bericht | die Rechnung aus `Belegzeilen` mit `rechnung = :id`: Absender und Anschrift im Berichtskopf, Spaltenkopf auf jeder Seite, Netto, Steuer je Satz und Gesamt im Berichtsfuß |

Was man daran sieht: Geld steht als INTEGER in Cent, die Steuer wird je
Position gerundet; die Felder der Stammdaten sind an die Spalten gebunden,
Laden und Sichern sind also je ein Aufruf (`GUI_FORM_LOAD`/`GUI_FORM_SAVE`),
und die Regeln (Pflichtfeld, PLZ, E-Mail) prüft `GUI_VALIDATE`; ein
Doppelklick auf eine Rechnung öffnet sie (`tabelle_Doppelklick`); ungesicherte
Eingaben werden mit `Frage(..., "Sichern|Verwerfen|Abbrechen")` erfragt, und
`ZeigenModal` wartet, bis das Fenster zu ist. Die Rechnung druckt der Bericht:
`Rechnung.alsPdf` ist ein Aufruf, `Rechnungsbeleg.Pdf(pfad, {"id": Self.id})`.
Was gerechnet werden muss, rechnet die Abfrage. Die Datenbank liegt im
Projektordner (das Element Daten legt sie an, der Reiter zeigt ihre Daten);
PDFs und die CSV landen beim Starten aus dem Studio in `_studio/`.

**Stolperstein beim Benennen:** Namen kennen keine Groß-/Kleinschreibung. Eine
Methode `zeigen(id)` in einem Fenster überschreibt das geerbte `Zeigen()` --
`dhrt --check` (und damit die Problemliste) warnt, wenn eine Methode eine
geerbte mit einer anderen Zahl von Parametern überschreibt.

**Das große Beispiel:** [Kuehlschrank-Drachenhauch](https://github.com/HasoSchno70/Kuehlschrank-Drachenhauch) (ein eigenes
Repo) ist eine Kühlschrank-Verwaltung (der Port einer PySide6-App, siehe sein README)
mit 13 Fenstern und 5 eigenen Controls, einer vorhandenen SQLite-Datei als
Datenbank-Element, Werkzeug- und Statusleiste, Timern, Diagrammen, einem
Bericht, Netzabfragen im Hintergrund und einer englischen Oberfläche
(`sprache/en.txt`, siehe „Sprachen“ oben). Wie es Stufe für Stufe entstand
und welche Lücken des Studios es dabei gefunden hat, steht in
`geschichte/kuehlschrank.md`.

## Für Prüfungen

| Umgebung | wirkt so |
|---|---|
| `DH_STUDIO_LOG=datei` | schreibt, was das Studio tut, zeilenweise, dazu die Lagen von Bibliothekszeilen, Inspektorfeldern, Leistenknöpfen, Navigatorknoten und Reitern |
| `DH_STUDIO_GROESSE=BxH` | feste Fenstergröße statt maximiert |
| `DH_STUDIO_AUFNAHME=datei` | spielt eine Aufnahme ab (`AUTOMATION_PLAY`) |
| `DH_STUDIO_ABLAGE=text` | legt Text in die Zwischenablage, zum Tippen per Strg+V |
| `DH_STUDIO_LAUF_BILDER=n` | Starten lässt das Programm nur n Bilder laufen (`dhrt run --bilder n`) |
| `DH_STUDIO_DATEI=pfad` | Bild hinzufügen nimmt diese Datei statt eines Dialogs |
| `DH_STUDIO_SPRACHE=en` | die Sprache der Oberfläche, vor jeder Einstellung (`de` oder `en`) |
| `DH_STUDIO_KONFIG=datei` | `studio.json` an diesem Ort; die `ide.json` wird dann nicht gefragt |
| `DH_STUDIO_SPRACHPROBE=1` | öffnet jeden Dialog einmal, schreibt seine Texte und die Zahl fehlender Übersetzungen ins Protokoll und endet |
| `DH_STUDIO_LOESCHEN=element` | fragt beim Start wie „Element löschen …“ für dieses Element |
| `DH_STUDIO_TEMPO=1` | schreibt (mit `DH_STUDIO_LOG`) je Bild `tempo <ms>`: wie lange das Studio gerechnet hat, ohne das Warten auf das nächste Bild |
| `DH_STUDIO_OEFFNEN=element[.tabelle]` | öffnet beim Start ein Element wie ein Klick im Navigator (bei einer Datenbank auf Wunsch eine Tabelle) |

Die Prüfsammlungen `tests/pruef/studio/werkzeug_studio.dhtest` (Stufe 1) und
`tests/pruef/studio/werkzeug_studio2.dhtest` (Stufe 2) starten das Studio zweimal:
einmal zum Messen, einmal mit echten Klicks und Tasten. Das Projektmodell
(`studio/projekt.dh`) prüfen sie auch ohne Fenster.
`tests/pruef/studio/werkzeug_studio3.dhtest` (Stufe 3) drückt F5 und Strg+B, lässt
das erzeugte Programm mit einem Klick laufen und prüft, an welcher Methode
und Zeile ein Fehler landet. `tests/pruef/studio/werkzeug_studio4.dhtest` (Stufe 4)
legt Timer und Menüs an, lässt das erzeugte Programm Fragen, ein modales
Fenster, Timer, Menükürzel und ein Bild durchlaufen und hält im Debugger.
`tests/pruef/studio/werkzeug_studio_datenbank.dhtest` prüft die Datenbanken: den
Kern ohne Fenster (an der Datei, wie SQLite sie sieht), das erzeugte
Programm an einer älteren Datei und das Studio mit echten Klicks in beide
Gitter, F5 und Bauen; `tests/pruef/studio/werkzeug_studio_datenbank2.dhtest` die
Bindung: Methoden und Listen im erzeugten Fenster samt Klick auf eine Zeile,
Spalten ins Fenster ziehen und doppelklicken, die Auswahlen im Inspektor;
`tests/pruef/studio/werkzeug_studio_datenbank3.dhtest` Verweise, Bedingung,
Sortierung, Suchfeld und Abfragen: den Kern, das erzeugte Fenster und das
Studio mit echten Klicks im Reiter, im Navigator und im Inspektor;
`tests/pruef/studio/werkzeug_studio_datenbank4.dhtest` Bildspalten (Kern, erzeugtes
Fenster samt Bild in der Tabellenzelle, „Bild …“ und Ziehen im Studio) und
`tests/pruef/studio/werkzeug_studio_verbunde.dhtest` Verbunde über Verweise;
`tests/pruef/studio/werkzeug_studio_berichte.dhtest` Berichte (Auslegung samt
Summen und Seiten, F5 mit Parametern, Reiter und Vorschau),
`tests/pruef/studio/werkzeug_studio_berichte2.dhtest` ihren Entwurf (ablegen,
ziehen, Griffe, Spalten, Eigenschaften, Bänder und Gruppen, Rückgängig).
