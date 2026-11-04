# Entwurf: Berichte im Drachenhauch Studio

*Ein Bericht ist eine gedruckte Seite aus den Daten des Projekts, etwa eine
Rechnung, eine Kundenliste oder eine Monatsübersicht. Er wird im Studio auf
einer Papierseite entworfen und im Programm als PDF gesichert, gedruckt oder
in einem Fenster gezeigt. Vorbild ist der Bericht-Editor von Xojo.
Entschieden am 2026-11-03:*

* *Schon die erste Stufe hat Bänder mit Gruppen: Berichtskopf, Seitenkopf,
  Gruppenkopf, Detail, Gruppenfuß, Seitenfuß, Berichtsfuß, mit
  Zwischensummen je Gruppe. Damit geht schon eine Rechnung (Gruppe =
  Rechnung, Detail = Positionen).*
* *Ein Bericht nennt eine Tabelle oder Abfrage des Projekts als Quelle. Die
  Bedingung darf Parameter haben (`rechnung = :id`); im Code ruft man ihn
  mit Werten dafür auf.*
* *Entworfen wird auf einer Seite in Millimetern, mit Linealen. Die Bänder
  stehen untereinander, ihre Höhe lässt sich ziehen.*

**Stand:** Stufe 1 ist gebaut (2026-11-03): `studio/berichte_laufzeit.dh`
(Auslegung), `studio/berichte.dh` (Modell, erzeugter Code),
`studio/berichte_reiter.dh` (Reiter, Vorschau), `PDF_PREVIEW` mit den echten
Schriften; Prüfung `tests/pruef/werkzeug_studio_berichte.dhtest`, Handbuch
[studio.md](studio.md#berichte). Stufe 2 ist gebaut (selber Tag): der Reiter
ist ein Entwurf (Leiste der Elemente, Spalten der Quelle, Eigenschaften,
Ziehen mit Raster und Ausrichtungslinien, Griffe, Bandhöhe, Rahmen,
Rückgängig je Reiter); jedes Band trägt eine Leiste mit seinem Namen statt
einer Namensspalte neben dem Papier -- die Seite wird so breiter, und die
Leiste ist die Stelle, an der man das Band wählt. Prüfung
`tests/pruef/werkzeug_studio_berichte2.dhtest`. Stufe 3 ist gebaut
(2026-11-04): das Beispiel `examples/studio/rechnungen` druckt seine Rechnung
über den Bericht `Rechnungsbeleg` (Quelle: die Abfrage `Belegzeilen` des neuen
Datenbank-Elements `Daten`); dafür kam die Element-Eigenschaft `wo` („Nur
Zeilen mit“, `mwst = 19`) dazu -- eine Summe rechnet nur über die Zeilen, auf
die sie zutrifft, jedes andere Element erscheint nur, wenn sein Band eine
solche Zeile hat. So steht die Steuer je Satz in der Rechnung, ohne
Unterbericht. Geprüft in `werkzeug_studio_berichte.dhtest` und
`werkzeug_studio_beispiel.dhtest` (liest die PDFs). Bildspalten setzt die
Auslegung noch nicht (der Entwurf lehnt sie ab).

## Gemessen vor Stufe 1

* **Das pdf-Modul trägt einen Bericht:** 2000 Zeilen zu vier Feldern mit
  Linie auf 40 Seiten zeichnen sich in 42 ms auf, das Sichern dauert 47 ms
  (107 KB). Eine Seite als Bild (`PDF_PREVIEW`, 800 Punkte breit) braucht
  5 bis 6 ms.
* **Die Vorschau stimmt nicht mit dem Papier überein:** `PDF_PREVIEW` setzt
  Text in raylibs Pixelschrift, das PDF in DejaVu. Breiten, rechtsbündige
  Beträge und Umbrüche sähen im Designer anders aus als gedruckt. Stufe 1
  ändert das in der Laufzeit (unten), damit PDF, Druck und beide Vorschauen
  über einen einzigen Weg laufen.

## Was übernommen wird

| Xojo | hier |
|---|---|
| Report-Element im Projekt | Element `bericht`, Datei `<Name>.dhreport` (JSON) |
| Bänder: Page Header, Group Header, Body, Group Footer, Page Footer | dieselben, dazu Berichtskopf und Berichtsfuß |
| Felder an Spalten eines RowSet | Felder an Spalten der Quelle, auch `kunde.name` über einen Verweis |
| Summen je Gruppe und Bericht | `summe` mit Funktion (Summe, Anzahl, Mittel, Min, Max) und Bereich (Gruppe, Bericht) |
| `Report.Run(rowset, printer)` | `Bericht.Pdf(pfad, werte)`, `.Drucken(werte)`, `.Vorschau(werte)` |

Bewusst nicht: Unterberichte, Diagramme im Bericht, Formeln in einer
eigenen Ausdruckssprache. Was gerechnet werden muss, rechnet eine Abfrage.

## Das Format

```text
{
  "seite": {"format": "A4", "hoch": true, "rand": [15, 15, 15, 15]},
  "quelle": "Daten.positionen",
  "bedingung": "rechnung = :id",
  "sortieren": ["rechnung", "id"],
  "gruppen": [{"spalte": "rechnung"}],
  "baender": [
    {"art": "berichtskopf", "hoehe": 30, "elemente": []},
    {"art": "seitenkopf", "hoehe": 12, "elemente": []},
    {"art": "gruppenkopf", "gruppe": 0, "hoehe": 20, "elemente": []},
    {"art": "detail", "hoehe": 6, "elemente": []},
    {"art": "gruppenfuss", "gruppe": 0, "hoehe": 12, "elemente": []},
    {"art": "seitenfuss", "hoehe": 10, "elemente": []},
    {"art": "berichtsfuss", "hoehe": 10, "elemente": []}
  ]
}
```

Ein Element hat `art`, `x`, `y`, `b`, `h` in mm relativ zu seinem Band:

| Art | Inhalt |
|---|---|
| `text` | fester Text |
| `feld` | `spalte` der Quelle, mit `format` (text, ganz, zahl, geld, datum, wahr) |
| `summe` | `spalte`, `funktion` (summe, anzahl, mittel, min, max), `bereich` (gruppe, bericht), `format` |
| (jede Art) | `wo`: nur Zeilen mit einem Wert (`mwst = 19`, auch `<>`, `<`, `>`, `<=`, `>=`) -- eine Summe rechnet nur über sie, ein anderes Element erscheint nur, wenn das Band eine solche Zeile hat |
| `seitenzahl` | „Seite n von m“ (Vorlage in `text`, `{n}` und `{m}`) |
| `datum` | das Datum des Drucks |
| `linie`, `rechteck` | Strichstärke, Farbe, bei `rechteck` eine Füllung |
| `bild` | ein Bild aus `bilder/` im Projekt oder aus einer Bildspalte (`spalte`) |

Schrift (`sans`, `serif`, `mono`), Größe in pt, fett, kursiv, Farbe und
Ausrichtung (links, mitte, rechts) gelten für die Arten mit Text.

**Seitenumbruch:** Ein Band, das nicht mehr auf die Seite passt, kommt auf
die nächste; vorher der Seitenfuß, danach der Seitenkopf. „Seite n von m“
braucht die Zahl aller Seiten, darum wird erst ausgelegt und dann gezeichnet.

## Im Programm

Je Bericht erzeugt das Studio eine Klasse mit drei Methoden. `werte` ist
eine MAP mit den Parametern der Bedingung:

```text
Rechnungsbericht.Pdf("rechnung.pdf", {"id": 5})
Rechnungsbericht.Drucken({"id": 5})
Rechnungsbericht.Vorschau({"id": 5})
```

`Vorschau` zeigt ein Fenster mit den Seiten, Blättern, Drucken und Sichern.
Die Auslegung steckt in einer Laufzeitdatei des Studios
(`studio_berichte.dh`, wie `studio_laufzeit.dh`), nicht in jeder Klasse.

## In der Laufzeit

`PDF_PREVIEW` zeichnet Text künftig mit derselben Schrift wie das PDF: die
eingebauten DejaVu-Schnitte und mit `PDF_FONT_LOAD` geladene Schriften, in
der Pixelgröße der Vorschau, je Schrift und Größe einmal geladen. Ohne das
sieht keine Vorschau aus wie das Papier.

## Im Studio

* **Element „Bericht“** im Navigator und in „Hinzufügen“.
* **Reiter:** die Seite mit Linealen und Rändern, darauf die Bänder
  untereinander mit ihrem Namen am Rand, darin die Elemente.
* **Leiste der Elemente** (Text, Feld, Summe, Seitenzahl, Datum, Linie,
  Rechteck, Bild) und die **Spalten der Quelle**, die man in ein Band zieht.
* **Eigenschaften** des gewählten Elements, Bandes oder Berichts in einem
  eigenen Bereich des Reiters.
* **Vorschau** mit den Daten der Entwicklungsdatei, die Parameter fragt ein
  kleiner Dialog.
* **Rückgängig** über den JSON-Text je Stand, wie beim Fenster-Entwurf.

## Stufen

1. **Auslegen und Drucken.**
   * Das Format und die Laufzeitdatei: Bänder, Gruppen, Summen,
     Seitenumbruch, Seitenzahlen.
   * PDF, Drucken und das Vorschaufenster im erzeugten Programm, mit
     Parametern.
   * `PDF_PREVIEW` mit den echten Schriften.
   * Im Studio das Element und ein Reiter, der den Bericht zeigt (noch ohne
     Bearbeiten), dazu die Vorschau mit Entwicklungsdaten.
   * Geprüft an Berichten, die als Datei vorliegen, gegen das PDF
     (`pdftext.dh`).
2. **Entwerfen.**
   * Elemente aus der Leiste ablegen, ziehen, Größe ändern, Raster in mm.
   * Bandhöhe ziehen, Bänder und Gruppen anlegen und entfernen.
   * Spalten aus der Quelle in ein Band ziehen.
   * Eigenschaften, Mehrfachauswahl, Ausrichtungslinien, Rückgängig.
3. **Im Einsatz.**
   * Das Beispielprojekt Rechnungen druckt seine Rechnung über einen
     Bericht.
   * Englische Oberfläche, Handbuch.

Jede Stufe wird am Bild vorgelegt, bevor es weitergeht.
