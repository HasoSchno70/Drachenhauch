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
Studio-Projekt ist), und eine `.dhstudio` im Projektbaum geht im Studio auf.
Der Form-Designer öffnet sein Formular mit Datei → Im Drachenhauch Studio
öffnen (Strg+Umschalt+O).

**Stand: Stufe 4.** Projekte mit mehreren Fenstern, Modulen und Klassen,
Ereignisse und ein Code-Feld je Methode; Starten, Bauen und Fehler an der
Methode, in der sie stehen; Timer, Menüs, Dialoge, die warten, Bilder und
ein Debugger.

## Die Oberfläche

| Bereich | was er tut |
|---|---|
| Werkzeugleiste | Neu, Öffnen, Sichern; Fenster, Methode, Ereignis hinzufügen; Rückgängig, Wiederholen, Ausrichten, Zoom; Starten, Anhalten, Prüfen, Bauen |
| Navigator (links oben) | das Projekt als Baum: App, Fenster mit ihren Controls und Ereignissen, Module, Klassen, ihre Methoden und Eigenschaften; ein Klick öffnet |
| Bibliothek (links unten) | alle Bausteine der gui in Gruppen, mit Vorschau; Suchfeld darüber |
| Mitte | Reiter mit Kreuz: je Fenster das Layout, je Methode ihr Code, je Element seine Eigenschaften |
| Inspektor (rechts) | die Eigenschaften des gewählten Controls, ohne Auswahl die des Fensters |
| Statusleiste | Meldung, Name, Lage und Größe des gewählten Controls, Zoom (Klick: 100 %) |

Die Leisten lassen sich wie in der IDE verschieben, als Reiter stapeln und
abtrennen. Unter Ansicht gibt es ein helles und ein dunkles
Erscheinungsbild.

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

Ein neues Projekt hat die App und ein Fenster `Hauptfenster`. Sichern
(Strg+S) schreibt alle Dateien; ein noch nie gesichertes Projekt fragt nach
einem Ordner. Ein umbenanntes Element nimmt seine alten Dateien mit, ein
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
| Liste, Baum, Tabelle | Geaendert, Doppelklick (bei Liste und Tabelle auch Enter) |
| Regler, Zahlenfeld, Klappliste, Wähler, Reiterwerk | Geaendert |
| Zeichenfläche | Zeichnen, Maus, Ziehen(x, y), Rad(schritte) |
| jedes Control zusätzlich | MausRein, MausRaus |
| Timer, Menüeintrag | Aktion |
| Fenster | Geoeffnet, Schliessen, Groesse, Taste(taste) |
| App | Geoeffnet, Beenden |

Ein Ereignis mit Klammern ruft seine Methode mit Werten: `Ziehen` mit der
Lage auf der Fläche (solange die Taste nach einem Druck auf die Fläche
hängt), `Rad` mit den Schritten des Mausrads, `Taste` mit dem Tastencode
(`KEY_A`, `KEY_SPACE` ...), solange das Fenster vorn ist und kein
Eingabefeld die Taste nimmt. Die Vorschläge kennen den ganzen Text des
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

**Dialoge, die warten.** Die Laufzeit des Projekts bietet:

| Aufruf | was er tut |
|---|---|
| `Meldung(text$ [, titel$])` | eine Meldung mit OK |
| `Frage(text$ [, knoepfe$ [, titel$]])` | Knöpfe mit `\|` getrennt (Vorgabe `Ja\|Nein`), liefert die Nummer des Knopfs; Enter = der erste, ESC = der letzte |
| `Eingabe(text$ [, vorgabe$ [, titel$]])` | fragt nach einem Text, `""` bei Abbrechen |
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
| Pfeiltasten | schieben um einen Punkt, mit Umschalt um das Raster (8) |
| Entf | löscht die Auswahl |
| Strg+D | verdoppelt |
| Strg+A | wählt alles (nicht, solange ein Eingabefeld den Fokus hat) |
| Strg+Z / Strg+Y | Rückgängig / Wiederholen |
| Strg+Mausrad, Strg+Plus/Minus, Strg+0 | Zoom |
| Esc | bricht ein Ablegen ab, sonst hebt es die Auswahl auf |

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

## Der Inspektor

Die Eigenschaften stehen in Gruppen, die auf- und zuklappen: Kennung, Lage,
Inhalt, Aussehen, Verhalten. Zu sehen ist nur, was zur Art des Controls passt.
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

## Ein einzelnes Fenster

Eine `.dhform` (dasselbe Format, das `GUI_SAVE` schreibt und `GUI_LOAD`
liest) geht auch ohne Projekt auf, aus dem Form-Designer oder aus einem
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
| App | App | öffnet die Datenbank, bevor das erste Fenster aufgeht |
| Datenbank | Modul | SQLite-Datei `rechnungen.db`, beim ersten Start mit Beispieldaten; nächste Rechnungsnummer |
| Ausgabe | Modul | Datum und Beträge für die Anzeige, CSV-Export |
| Rechnung | Klasse | eine Rechnung samt Positionen: laden, rechnen, sichern (eine Transaktion), als PDF |
| Hauptfenster | Fenster | alle Rechnungen mit Filterzeile, Menü mit Kürzeln, Neu/Bearbeiten/PDF/Löschen |
| RechnungFenster | Fenster | eine Rechnung anlegen oder ändern, modal |
| KundenFenster, ArtikelFenster | Fenster | Stammdaten über die Datenbindung (Inspektor: Bindung) |

Was man daran sieht: Geld steht als INTEGER in Cent, die Steuer wird je
Position gerundet; die Felder der Stammdaten sind an die Spalten gebunden,
Laden und Sichern sind also je ein Aufruf (`GUI_FORM_LOAD`/`GUI_FORM_SAVE`),
und die Regeln (Pflichtfeld, PLZ, E-Mail) prüft `GUI_VALIDATE`; ein
Doppelklick auf eine Rechnung öffnet sie (`tabelle_Doppelklick`); ungesicherte
Eingaben werden mit `Frage(..., "Sichern|Verwerfen|Abbrechen")` erfragt, und
`ZeigenModal` wartet, bis das Fenster zu ist. Die Datenbank, PDFs und die CSV
landen beim Starten aus dem Studio in `_studio/`.

**Stolperstein beim Benennen:** Namen kennen keine Groß-/Kleinschreibung. Eine
Methode `zeigen(id)` in einem Fenster überschreibt das geerbte `Zeigen()` --
`dhrt --check` (und damit die Problemliste) warnt, wenn eine Methode eine
geerbte mit einer anderen Zahl von Parametern überschreibt.

## Für Prüfungen

| Umgebung | wirkt so |
|---|---|
| `DH_STUDIO_LOG=datei` | schreibt, was das Studio tut, zeilenweise, dazu die Lagen von Bibliothekszeilen, Inspektorfeldern, Leistenknöpfen, Navigatorknoten und Reitern |
| `DH_STUDIO_GROESSE=BxH` | feste Fenstergröße statt maximiert |
| `DH_STUDIO_AUFNAHME=datei` | spielt eine Aufnahme ab (`AUTOMATION_PLAY`) |
| `DH_STUDIO_ABLAGE=text` | legt Text in die Zwischenablage, zum Tippen per Strg+V |
| `DH_STUDIO_LAUF_BILDER=n` | Starten lässt das Programm nur n Bilder laufen (`dhrt run --bilder n`) |
| `DH_STUDIO_DATEI=pfad` | Bild hinzufügen nimmt diese Datei statt eines Dialogs |

Die Prüfsammlungen `tests/pruef/werkzeug_studio.dhtest` (Stufe 1) und
`tests/pruef/werkzeug_studio2.dhtest` (Stufe 2) starten das Studio zweimal:
einmal zum Messen, einmal mit echten Klicks und Tasten. Das Projektmodell
(`studio/projekt.dh`) prüfen sie auch ohne Fenster.
`tests/pruef/werkzeug_studio3.dhtest` (Stufe 3) drückt F5 und Strg+B, lässt
das erzeugte Programm mit einem Klick laufen und prüft, an welcher Methode
und Zeile ein Fehler landet. `tests/pruef/werkzeug_studio4.dhtest` (Stufe 4)
legt Timer und Menüs an, lässt das erzeugte Programm Fragen, ein modales
Fenster, Timer, Menükürzel und ein Bild durchlaufen und hält im Debugger.
