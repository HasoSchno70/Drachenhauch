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

Das hat einen Preis, und der steht hier offen:

**`PDF_TEXT_WIDTH` geht nur bei Courier** (und seinen Schnitten). Dort ist
jedes Zeichen 600/1000 der Schriftgröße breit — das ist die Bauart der
Schrift, keine Schätzung. Für Helvetica und Times steht die Breite jedes
Zeichens in Adobes Schriftmaßen; die liegen hier nicht vor, und sie zu
schätzen wäre die Art Zahl, die auf den ersten Blick stimmt und eine
Rechnungsspalte still um zwei Millimeter verschiebt. Der Aufruf meldet das,
statt zu raten.

**Praktische Folge:** Beschriftungen in Helvetica, **Zahlenspalten in
Courier** — dann lassen sie sich exakt rechtsbündig setzen:

```basic
SUB Rechtsbuendig(p AS PDF, x AS FLOAT, y AS FLOAT, text AS STRING)
    PDF_TEXT(p, x - PDF_TEXT_WIDTH(p, text), y, text)
END SUB

PDF_FONT(p, "courier", 10)
Rechtsbuendig(p, 188, 120, FORMAT$(betrag, "%.2f"))
```

Nebenbei sieht eine Zahlenspalte in einer dicktengleichen Schrift ohnehin
besser aus: die Stellen stehen untereinander.

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
