# Modul `pdf`

Druckfertige Seiten schreiben — Rechnung, Lieferschein, Bericht, Etikett.

```basic
IMPORT "pdf"
```

## Millimeter, von oben

PDF selbst rechnet in Punkten und von **unten**. Wer eine Rechnung setzt,
denkt aber in „25 mm vom oberen Rand" — also rechnet dieses Modul so. Nur die
**Schriftgröße bleibt in Punkten**, weil sie jeder so kennt (11 pt).

```basic
DIM p AS PDF
p = PDF_NEW()                        ' A4 hoch
PDF_FONT(p, "sans-fett", 18)
PDF_TEXT(p, 20, 25, "Rechnung 4711") ' 20 mm von links, 25 mm von oben
PDF_LINE(p, 20, 36, 190, 36)
PDF_SAVE(p, "rechnung.pdf")
```

## Übersicht

| Funktion | Zweck |
|---|---|
| `PDF_NEW([größe$[, ausrichtung$]])` → PDF | `a3`…`a6`, `letter`, `legal`; `hoch`/`quer` |
| `PDF_PAGE(p)` | neue Seite beginnen |
| `PDF_PAGE_COUNT(p)` → INTEGER | wie viele bisher |
| `PDF_PAGE_WIDTH(p)` / `PDF_PAGE_HEIGHT(p)` → FLOAT | Seitenmaß in mm |
| `PDF_TITLE(p, titel$)` | Titel in den Dokument-Angaben |
| `PDF_FONT(p, schrift$, größe_pt)` | Schrift und Größe |
| `PDF_FONT_LOAD(p, pfad$, name$)` | eine eigene TrueType-/OpenType-Schrift laden, danach unter `name$` für `PDF_FONT` da |
| `PDF_COLOR(p, farbe)` | `RGB(…)` — gilt für Text, Striche und Flächen |
| `PDF_LINE_WIDTH(p, mm)` | Strichstärke |
| `PDF_TEXT(p, x, y, text$)` | Text setzen |
| `PDF_TEXT_WIDTH(p, text$)` → FLOAT | Breite in mm, in der aktuellen Schrift und Größe |
| `PDF_LINE(p, x1, y1, x2, y2)` | Linie zeichnen |
| `PDF_RECT(p, x, y, b, h)` / `PDF_RECT_FILL(…)` | Umriss / Fläche |
| `PDF_SAVE(p, pfad$)` | schreiben |
| `PDF_PRINT(p[, drucker$[, kopien[, zieldatei$]]])` | **drucken** — Windows über GDI, macOS/Linux über CUPS; siehe [Drucken](#drucken-und-vorschau) |
| `PDF_PREVIEW(p, seite[, breite_px])` → IMAGE | die Seite als Bild, für eine Vorschau im Fenster |
| `PDF_CLOSE(p)` | Speicher freigeben |

Schrift, Größe, Farbe und Strichstärke sind Einstellungen des **Dokuments**,
nicht der Seite — sie gelten nach `PDF_PAGE` weiter.

## Schriften

```text
sans    sans-fett    sans-kursiv    sans-fett-kursiv
serif   serif-fett   serif-kursiv   serif-fett-kursiv
mono    mono-fett    mono-kursiv    mono-fett-kursiv
```

Das sind **DejaVu Sans, DejaVu Serif und DejaVu Sans Mono**, in dhrt
eingebaut. Die Schriften werden **eingebettet** und auf die benutzten Zeichen
beschnitten — das PDF sieht auf jedem Rechner gleich aus, und eine Seite mit
ein paar Zeilen kostet trotzdem nur einige Kilobyte.

**Die alten Namen gehen weiter:** `helvetica`, `times` und `courier` (samt
`-fett`, `-kursiv`, `-fett-kursiv`) meinen `sans`, `serif` und `mono`. Bis
2026-09-14 schrieb das Modul die vierzehn Standardschriften der PDF-Leser,
ohne etwas einzubetten; seit dem Umbau auf [krilla](https://github.com/LaurenzV/krilla)
sieht ein altes Programm etwas anders aus, und seine Texte sind etwas breiter.
`symbol` und `zapfdingbats` gibt es nicht mehr — griechische Buchstaben und
Zeichen wie ✓ ✗ ★ ← → hat `sans` selbst.

**Eine eigene Schrift** lädt `PDF_FONT_LOAD` aus einer `.ttf`- oder
`.otf`-Datei (bei einer Sammlung `.ttc` die erste). Der Name ist frei, darf
aber keine eingebaute Schrift meinen:

```basic
PDF_FONT_LOAD(p, "fonts/NotoSansJP-Regular.ttf", "japanisch")
PDF_FONT(p, "japanisch", 12)
PDF_TEXT(p, 20, 40, "請求書")
```

## Text und Breite

Der Text ist **Unicode** — Deutsch, Französisch, Polnisch, Griechisch,
Kyrillisch gehen mit den eingebauten Schriften. Ein Zeichen, das die gewählte
Schrift **nicht hat** (ein Emoji, ein chinesisches Zeichen in `sans`), ist ein
**Fehler** und wird nicht als leeres Kästchen gesetzt: auf einer Rechnung ist
ein stumm verschwundenes Zeichen schlimmer als eine Meldung. Dann hilft eine
eigene Schrift.

**`PDF_TEXT_WIDTH` misst, was gezeichnet wird** — mit derselben Textformung
(rustybuzz) wie das Setzen, samt Unterschneidung. Beträge lassen sich damit in
jeder Schrift exakt rechtsbündig setzen:

```basic
SUB Rechtsbuendig(p AS PDF, x AS FLOAT, y AS FLOAT, text AS STRING)
    PDF_TEXT(p, x - PDF_TEXT_WIDTH(p, text), y, text)
END SUB

PDF_FONT(p, "sans", 10)
Rechtsbuendig(p, 190, 120, "1.234,56 €")
```

Eine Zahlenspalte in `mono` sieht trotzdem oft besser aus: die Stellen stehen
untereinander.

## Drucken und Vorschau

Das Modul zeichnet seine Seiten **auf**. Dieselbe Seite geht damit auf drei
Ziele: die Datei (`PDF_SAVE`), den Drucker (`PDF_PRINT`) und ein Bild
(`PDF_PREVIEW`). Es gibt keinen PDF-Renderer dahinter; der Renderer ist das
Betriebssystem ([Entwurf](entwurf-drucken.md)).

```basic
PDF_PRINT(p)                                   ' Standarddrucker, eine Kopie
PDF_PRINT(p, "Brother MFC-L3760CDW series", 2) ' Drucker und Kopien
PDF_PRINT(p, "Microsoft Print to PDF", 1, "ausgabe.pdf")   ' in eine Datei, ohne Dialog
DIM bild AS IMAGE : bild = PDF_PREVIEW(p, 1, 600)          ' Seite 1, 600 px breit
PRINT PRINTER_DEFAULT$() : PRINT PRINTERS()                ' für einen eigenen Dialog
```

- **Windows:** GDI auf den Drucker — der Treiber rastert. Die eingebauten
  Schriften werden zu ihren Windows-Geschwistern (`sans` → Arial, `serif` →
  Times New Roman, `mono` → Courier New), eine mit `PDF_FONT_LOAD` geladene
  wird Arial; die Lage jedes Textes bleibt, weil das Programm Positionen
  setzt. Millimeter gelten ab Papierkante; den nicht druckbaren Rand rechnet
  das Modul heraus.
- **macOS/Linux:** die PDF geht an CUPS (`lp -d drucker -n kopien`), das
  PDF versteht — dort mit den eingebetteten Schriften. Eine `zieldatei` ist
  dort die PDF selbst.
- **`zieldatei`** ist für Drucker gedacht, die in eine Datei schreiben —
  „Microsoft Print to PDF" fragt dann **nicht** nach. Genau so prüft
  `tests/test_drucken.py` den Druck: durch einen echten Treiber, zurück-
  gelesen mit PyMuPDF. Gemessen: zwei Seiten in ~0,9 s.
- `PRINTERS()` liefert die Namen, wie das System sie kennt, `PRINTER_DEFAULT$()`
  den Standarddrucker (`""`, wenn es keinen gibt). Eine Falle unter Windows: die
  Einstellung „Standarddrucker von Windows verwalten lassen“ macht den
  **zuletzt benutzten** Drucker zum Standard — nach einem Druck auf „Microsoft
  Print to PDF“ ist das dann der Standard. Wer einen bestimmten Drucker will,
  nennt ihn. Fehlt der Drucker oder
  nimmt er den Auftrag nicht an, ist das ein Fehler mit Namen, kein stilles
  Nichts.
- `PDF_PREVIEW` zeichnet in raylibs Standardschrift auf weißes Papier:
  eine Vorschau, kein Belichter. Höhe nach Seitenverhältnis; braucht ein
  Fenster, weil es ein `IMAGE` ist.

Bewusst nicht: fremde PDFs drucken (das Modul druckt, was es gesetzt hat)
und ein nachgebauter Druckdialog — `PRINTERS()` plus eine Klappliste im
eigenen Fenster tut es, einheitlich und testbar. Duplex und Papierfach
fehlen; nachrüstbar als Treiberfelder.

## Zwei Zusagen

* **Dieselbe Eingabe ergibt dieselbe Datei** — es wird kein Erstellungsdatum
  hineingeschrieben, und die Dokumentkennung entsteht aus dem Inhalt. Das
  macht Prüfungen vergleichbar und einen Versionsverlauf lesbar; wer ein
  Datum braucht, schreibt es sichtbar auf die Seite, wo es hingehört.
* **Die Inhalte sind gepackt** (Deflate), und von jeder Schrift steht nur
  drin, was benutzt wurde.

## Was fehlt

Bilder, Tabellen-Automatik, Zeilenumbruch, Seitenzahlen-Automatik,
Verschlüsselung, Text von rechts nach links in gemischten Zeilen. Das ist ein
Setzkasten, kein Textsatzsystem — wer eine Seite baut, positioniert selbst.
Für den Fall, um den es geht (ein Formular mit festem Aufbau und wechselnden
Zahlen), ist das genau richtig; für einen fließenden Bericht mit Umbruch wäre
es zu wenig. krilla kann Bilder — nachrüstbar.

Beispiel: [examples/176_rechnung_pdf.dh](../examples/176_rechnung_pdf.dh) —
eine vollständige Rechnung mit Kopf, Anschriftfeld, Positionstabelle und
Summenblock.
