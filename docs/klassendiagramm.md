# Klassendiagramm

`examples/210_klassendiagramm.dh` zeigt alle Klassen und Structs eines
Projekts als Kästen: oben der Name, darunter Felder, Konstanten,
Eigenschaften und Methoden mit ihrem Typ. Vererbung ist ein Pfeil mit
offenem Dreieck zur Vorfahrin (wie in UML), ein Feld vom Typ einer anderen
Klasse (auch `ARRAY OF` oder `MAP OF`) eine gestrichelte Linie dorthin.

```
dhrt run examples/210_klassendiagramm.dh -- <projektordner oder datei>
```

In der IDE: **Werkzeuge → Klassendiagramm** (nimmt den Projektordner).

## Bedienung

| Was | Wie |
|---|---|
| Kasten verschieben | ziehen -- die Lage wird gemerkt (`.klassendiagramm.json` im Projektordner) |
| Ansicht verschieben | auf freier Fläche ziehen oder das Mausrad |
| In den Code springen | Doppelklick auf einen Kasten: auf ein Mitglied dessen Zeile, sonst die Kopfzeile der Klasse |
| Neu einlesen | F5 |
| Neu anordnen | L -- Ebenen nach der Vererbung, die gemerkten Lagen werden überschrieben |
| Als Bild sichern | P -- das ganze Diagramm als `klassendiagramm.png` im Projektordner |

Der Sprung geht über die Ausgabe des Werkzeugs: es schreibt eine Zeile
`DH_IDE_OEFFNEN|pfad|zeile`, und die IDE, die es gestartet hat, öffnet die
Datei dort. Jedes Werkzeug kann das; allein gestartet steht die Zeile nur in
der Konsole.

## Was es liest

Die Klassen kommen aus `CODE_CLASSES$`, ohne zu übersetzen -- eine Datei mit
einem Fehler kommt trotzdem ins Bild. Ordner, die mit `.` oder `_` beginnen,
sowie `pakete/` und `target/` bleiben draußen, wie bei den Umbauten der IDE.
Steht derselbe Klassenname in zwei Dateien, gilt der erste. Eine Vorfahrin,
die das Projekt nicht kennt, steht klein über dem Kasten.

Geprüft wird in `tests/pruef/werkzeug_klassendiagramm.dhtest`.
