# Drachenhauch Studio

Drachenhauch Studio ist eine IDE für Programme mit Oberfläche, im Stil von
Xojo: Fenster zusammenklicken, Eigenschaften im Inspektor einstellen, später
den Code der Ereignisse schreiben und das Programm starten. Es läuft mit der
Laufzeit `dhrt` und ist selbst ein Drachenhauch-Programm. Der Entwurf mit
allen Stufen steht in [entwurf-studio.md](entwurf-studio.md).

```text
dhrt run studio/studio.dh
dhrt run studio/studio.dh -- fenster.dhform
```

**Stand: Stufe 1.** Ein Fenster entwerfen, sichern und wieder öffnen.
Projekte, Code und Starten kommen mit den Stufen 2 und 3. Bis dahin gibt es
für den Code den Form-Designer (`examples/197_form_designer.dh`, DH-Code mit
Strg+G).

## Die Oberfläche

| Bereich | was er tut |
|---|---|
| Werkzeugleiste | Neu, Öffnen, Sichern, Rückgängig, Wiederholen, Ausrichten, Zoom; Starten folgt mit Stufe 3 |
| Navigator (links oben) | das Fenster und seine Controls als Baum; ein Klick wählt |
| Bibliothek (links unten) | alle Bausteine der gui in Gruppen, mit Vorschau; Suchfeld darüber |
| Mitte | das Fenster, das man entwirft, mit Titelleiste und Größe |
| Inspektor (rechts) | die Eigenschaften des gewählten Controls, ohne Auswahl die des Fensters |
| Statusleiste | Meldung, Name, Lage und Größe des gewählten Controls, Zoom (Klick: 100 %) |

Die Leisten lassen sich wie in der IDE verschieben, als Reiter stapeln und
abtrennen. Unter Ansicht gibt es ein helles und ein dunkles
Erscheinungsbild.

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

## Die Datei

Gesichert wird eine `.dhform`, dasselbe Format, das `GUI_SAVE` schreibt und
`GUI_LOAD` liest. Eine `.dhform` aus dem Form-Designer oder aus einem Programm
geht im Studio auf; Controls ohne Namen bekommen beim Öffnen einen. Neu,
Öffnen und Beenden fragen bei ungesicherten Änderungen nach.

## Für Prüfungen

| Umgebung | wirkt so |
|---|---|
| `DH_STUDIO_LOG=datei` | schreibt, was das Studio tut, zeilenweise, dazu die Lagen von Bibliothekszeilen, Inspektorfeldern und Leistenknöpfen |
| `DH_STUDIO_GROESSE=BxH` | feste Fenstergröße statt maximiert |
| `DH_STUDIO_AUFNAHME=datei` | spielt eine Aufnahme ab (`AUTOMATION_PLAY`) |
| `DH_STUDIO_ABLAGE=text` | legt Text in die Zwischenablage, zum Tippen per Strg+V |

Die Prüfsammlung `tests/pruef/werkzeug_studio.dhtest` startet das Studio
zweimal: einmal zum Messen, einmal mit echten Klicks und Tasten.
