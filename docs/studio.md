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

**Stand: Stufe 2.** Projekte mit mehreren Fenstern, Modulen und Klassen,
Ereignisse und ein Code-Feld je Methode. Starten und Bauen kommen mit
Stufe 3 (erst sie erzeugt die Basisklassen der Fenster).

## Die Oberfläche

| Bereich | was er tut |
|---|---|
| Werkzeugleiste | Neu, Öffnen, Sichern; Fenster, Methode, Ereignis hinzufügen; Rückgängig, Wiederholen, Ausrichten, Zoom; Starten folgt mit Stufe 3 |
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
oder der App nur `<Ereignis>` (`Geoeffnet`). Die Basisklasse, die Stufe 3
erzeugt, verdrahtet sie.

| Handgriff | wirkt so |
|---|---|
| Doppelklick auf ein Control im Entwurf | „Ereignis hinzufügen“ für dieses Control; auf leere Fläche: für das Fenster |
| Strg+Umschalt+E | dasselbe für die Auswahl (oder für das Element des vorderen Reiters) |
| Strg+Umschalt+M | Methode hinzufügen: Name, Parameter (`a AS INTEGER, b AS STRING`), Rückgabe (leer = SUB) |
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
| Liste, Baum | Geaendert, Doppelklick |
| Regler, Zahlenfeld, Klappliste, Wähler, Reiterwerk, Tabelle | Geaendert |
| Zeichenfläche | Zeichnen, Maus |
| jedes Control zusätzlich | MausRein, MausRaus |
| Fenster | Geoeffnet, Schliessen, Groesse |
| App | Geoeffnet, Beenden |

Über dem Code steht die Kopfzeile der Methode (`SUB knopf1_Klick()`), die
man nicht tippt; rechts daneben, wozu sie da ist. Das Code-Feld färbt,
schließt Klammern, rückt beim Zeilenumbruch ein und setzt das Ende eines
Blocks mit (`IF` → `END IF`). Strg+Z/Y gehören im Code-Reiter dem Code.

Wird ein Control im Inspektor umbenannt, ziehen seine Ereignis-Methoden mit,
und im Code des Fensters wird der alte Name als ganzes Wort ersetzt.

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

## Für Prüfungen

| Umgebung | wirkt so |
|---|---|
| `DH_STUDIO_LOG=datei` | schreibt, was das Studio tut, zeilenweise, dazu die Lagen von Bibliothekszeilen, Inspektorfeldern, Leistenknöpfen, Navigatorknoten und Reitern |
| `DH_STUDIO_GROESSE=BxH` | feste Fenstergröße statt maximiert |
| `DH_STUDIO_AUFNAHME=datei` | spielt eine Aufnahme ab (`AUTOMATION_PLAY`) |
| `DH_STUDIO_ABLAGE=text` | legt Text in die Zwischenablage, zum Tippen per Strg+V |

Die Prüfsammlungen `tests/pruef/werkzeug_studio.dhtest` (Stufe 1) und
`tests/pruef/werkzeug_studio2.dhtest` (Stufe 2) starten das Studio zweimal:
einmal zum Messen, einmal mit echten Klicks und Tasten. Das Projektmodell
(`studio/projekt.dh`) prüfen sie auch ohne Fenster.
