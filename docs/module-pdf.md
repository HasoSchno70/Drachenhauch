# Modul `pdf`

Druckfertige Seiten schreiben — Rechnung, Lieferschein, Bericht, Etikett.

```basic
IMPORT "pdf"
```

## Millimeter, von oben

PDF selbst rechnet in Punkten und von **unten**. Wer eine Rechnung setzt,
denkt aber in „25 mm vom oberen Rand" — also rechnet dieses Modul so und
dreht die Achse beim Schreiben um. Nur die **Schriftgröße bleibt in Punkten**,
weil sie jeder so kennt (11 pt).

```basic
DIM p AS PDF
p = PDF_NEW()                        ' A4 hoch
PDF_FONT(p, "helvetica-fett", 18)
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
| `PDF_COLOR(p, farbe)` | `RGB(…)` — gilt für Text, Striche und Flächen |
| `PDF_LINE_WIDTH(p, mm)` | Strichstärke |
| `PDF_TEXT(p, x, y, text$)` | Text setzen |
| `PDF_TEXT_WIDTH(p, text$)` → FLOAT | Breite in mm (siehe unten) |
| `PDF_LINE(p, x1, y1, x2, y2)` | Linie zeichnen |
| `PDF_RECT(p, x, y, b, h)` / `PDF_RECT_FILL(…)` | Umriss / Fläche |
| `PDF_SAVE(p, pfad$)` | schreiben |
| `PDF_PRINT(p[, drucker$[, kopien[, zieldatei$]]])` | **drucken** — Windows über GDI, macOS/Linux über CUPS; siehe [Drucken](#drucken-und-vorschau) |
| `PDF_PREVIEW(p, seite[, breite_px])` → IMAGE | die Seite als Bild, für eine Vorschau im Fenster |
| `PDF_CLOSE(p)` | Speicher freigeben |

Schrift, Größe, Farbe und Strichstärke sind Einstellungen des **Dokuments**,
nicht der Seite — sie gelten nach `PDF_PAGE` weiter.

## Die vierzehn Standard-Schriften

```text
helvetica   helvetica-fett   helvetica-kursiv   helvetica-fett-kursiv
times       times-fett       times-kursiv       times-fett-kursiv
courier     courier-fett     courier-kursiv     courier-fett-kursiv
symbol      zapfdingbats
```

Jeder PDF-Leser bringt sie mit — **es wird nichts eingebettet**. Genau daran
hängt der Aufwand eines PDF-Erzeugers: eine TrueType-Datei einzubetten hieße
Tabellen parsen, Untermengen bilden und einen CID-Font aufbauen.

Das hat einen Preis, und der steht hier offen: **Symbol und ZapfDingbats
lassen sich nicht messen**, und ein Zeichen außerhalb von WinAnsi auch
nicht — `PDF_TEXT_WIDTH` meldet das, statt zu raten.

**`PDF_TEXT_WIDTH` kennt Helvetica, Times und Courier** (je alle vier
Schnitte). Bei Courier ist jedes Zeichen 600/1000 der Schriftgröße breit —
die Bauart der Schrift. Für Helvetica und Times liegen die Schriftmaße in
`pdf_masse.rs`: nicht aus dem Gedächtnis, sondern von
`tools/gen_pdf_masse.py` aus den Base-14-Metriken von PyMuPDF erzeugt und im
Test gegen PyMuPDF nachgemessen (lange stand hier, eine geschätzte Breite sei
schlimmer als keine — das galt, solange es nur die Schätzung gab; ohne die
Maße ließ sich in einer Rechnung kein Betrag rechtsbündig setzen).

**Praktische Folge:** Beträge lassen sich in jeder der zwölf Schriften exakt
rechtsbündig setzen:

```basic
SUB Rechtsbuendig(p AS PDF, x AS FLOAT, y AS FLOAT, text AS STRING)
    PDF_TEXT(p, x - PDF_TEXT_WIDTH(p, text), y, text)
END SUB

PDF_FONT(p, "helvetica", 10)
Rechtsbuendig(p, 190, 120, "1.234,56 EUR")
```

Eine Zahlenspalte in Courier sieht trotzdem oft besser aus: die Stellen
stehen untereinander.

## Drucken und Vorschau

Das Modul zeichnet seine Seiten selbst — und zeichnet sie seit 2026-09-05
auch **auf**. Dieselbe Seite geht damit auf drei Ziele: die Datei
(`PDF_SAVE`), den Drucker (`PDF_PRINT`) und ein Bild (`PDF_PREVIEW`). Es gibt
keinen PDF-Renderer dahinter; der Renderer ist das Betriebssystem
([Entwurf](entwurf-drucken.md)).

```basic
PDF_PRINT(p)                                   ' Standarddrucker, eine Kopie
PDF_PRINT(p, "Brother MFC-L3760CDW series", 2) ' Drucker und Kopien
PDF_PRINT(p, "Microsoft Print to PDF", 1, "ausgabe.pdf")   ' in eine Datei, ohne Dialog
DIM bild AS IMAGE : bild = PDF_PREVIEW(p, 1, 600)          ' Seite 1, 600 px breit
PRINT PRINTER_DEFAULT$() : PRINT PRINTERS()                ' für einen eigenen Dialog
```

- **Windows:** GDI auf den Drucker — der Treiber rastert. Die Standard-
  schriften werden zu ihren Windows-Geschwistern (Helvetica → Arial, Times →
  Times New Roman, Courier → Courier New); die Lage jedes Textes bleibt, weil
  das Programm Positionen setzt — rechtsbündige Beträge bleiben rechtsbündig,
  die Buchstaben sind nur nicht Helvetica. Millimeter gelten ab Papierkante;
  den nicht druckbaren Rand rechnet das Modul heraus.
- **macOS/Linux:** die PDF geht an CUPS (`lp -d drucker -n kopien`), das
  PDF versteht. Eine `zieldatei` ist dort die PDF selbst.
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

## Umlaute

Der Text geht als **WinAnsi** (= cp1252) ins PDF — Deutsch, Französisch,
Spanisch sind damit abgedeckt, ohne eine Schrift einzubetten. Ein Zeichen,
das cp1252 nicht kennt (ein Emoji, Griechisch, Kyrillisch), ist ein
**Fehler** und wird nicht durch `?` ersetzt: dieselbe Regel wie beim
Schreiben einer cp1252-Textdatei. Auf einer Rechnung ist ein stumm
verschwundenes Zeichen schlimmer als eine Meldung.

## Zwei Zusagen

* **Dieselbe Eingabe ergibt dieselbe Datei** — es wird kein Erstellungsdatum
  hineingeschrieben. Das macht Prüfungen vergleichbar und einen
  Versionsverlauf lesbar; wer ein Datum braucht, schreibt es sichtbar auf die
  Seite, wo es hingehört.
* **Die Inhalte sind gepackt** (Deflate), eine Seite kostet also selbst bei
  viel Text kaum etwas.

## Was fehlt

Bilder, Tabellen-Automatik, Zeilenumbruch, Seitenzahlen-Automatik,
Verschlüsselung, eingebettete Schriften. Das ist ein Setzkasten, kein
Textsatzsystem — wer eine Seite baut, positioniert selbst. Für den Fall, um
den es geht (ein Formular mit festem Aufbau und wechselnden Zahlen), ist das
genau richtig; für einen fließenden Bericht mit Umbruch wäre es zu wenig.

Beispiel: [examples/176_rechnung_pdf.dh](../examples/176_rechnung_pdf.dh) —
eine vollständige Rechnung mit Kopf, Anschriftfeld, Positionstabelle und
Summenblock.
