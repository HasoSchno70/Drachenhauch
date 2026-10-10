# Module `pdf`

Write print-ready pages — invoice, delivery note, report, label.

```basic
IMPORT "pdf"
```

## Millimetres, from the top

PDF itself works in points and from the **bottom**. Someone laying out an
invoice, however, thinks in "25 mm from the top edge" — so that is how this
module works. Only the **font size stays in points**, because that is how
everyone knows it (11 pt).

```basic
DIM p AS PDF
p = PDF_NEW()                        ' A4 portrait
PDF_FONT(p, "sans-fett", 18)
PDF_TEXT(p, 20, 25, "Rechnung 4711") ' 20 mm from the left, 25 mm from the top
PDF_LINE(p, 20, 36, 190, 36)
PDF_SAVE(p, "rechnung.pdf")
```

## Overview

| Function | Purpose |
|---|---|
| `PDF_NEW([größe$[, ausrichtung$]])` → PDF | `a3`…`a6`, `letter`, `legal`; `hoch`/`quer` (portrait/landscape) |
| `PDF_PAGE(p)` | start a new page -- `PDF_NEW` already creates the first one; while it is empty, `PDF_PAGE` does not add a second |
| `PDF_PAGE_COUNT(p)` → INTEGER | how many so far |
| `PDF_PAGE_WIDTH(p)` / `PDF_PAGE_HEIGHT(p)` → FLOAT | page size in mm |
| `PDF_TITLE(p, titel$)` | title in the document information |
| `PDF_FONT(p, schrift$, größe_pt)` | font and size |
| `PDF_FONT_LOAD(p, pfad$, name$)` | load your own TrueType/OpenType font, afterwards available to `PDF_FONT` as `name$` |
| `PDF_COLOR(p, farbe)` | `RGB(…)` — applies to text, strokes and fills |
| `PDF_LINE_WIDTH(p, mm)` | stroke width |
| `PDF_TEXT(p, x, y, text$)` | set text |
| `PDF_TEXT_WIDTH(p, text$)` → FLOAT | width in mm, in the current font and size |
| `PDF_LINE(p, x1, y1, x2, y2)` | draw a line |
| `PDF_RECT(p, x, y, b, h)` / `PDF_RECT_FILL(…)` | outline / fill |
| `PDF_SAVE(p, pfad$)` | write |
| `PDF_PRINT(p[, drucker$[, kopien[, zieldatei$]]])` | **print** — Windows via GDI, macOS/Linux via CUPS; see [Printing](#printing-and-preview) |
| `PDF_PREVIEW(p, seite[, breite_px])` → IMAGE | the page as an image, for a preview in a window |
| `PDF_CLOSE(p)` | free the memory |

Font, size, colour and stroke width are settings of the **document**,
not of the page — they stay in effect after `PDF_PAGE`.

## Fonts

```text
sans    sans-fett    sans-kursiv    sans-fett-kursiv
serif   serif-fett   serif-kursiv   serif-fett-kursiv
mono    mono-fett    mono-kursiv    mono-fett-kursiv
```

These are **DejaVu Sans, DejaVu Serif and DejaVu Sans Mono**, built into
dhrt (`fett` = bold, `kursiv` = italic). The fonts are **embedded** and
subset to the characters used — the PDF looks the same on every machine,
and a page with a few lines still costs only a few kilobytes.

**The old names still work:** `helvetica`, `times` and `courier` (together with
`-fett`, `-kursiv`, `-fett-kursiv`) mean `sans`, `serif` and `mono`. Until
2026-09-14 the module wrote the fourteen standard fonts of the PDF readers
without embedding anything; since the switch to [krilla](https://github.com/LaurenzV/krilla)
an old program looks slightly different, and its texts are slightly wider.
`symbol` and `zapfdingbats` no longer exist — `sans` itself has Greek
letters and characters such as ✓ ✗ ★ ← →.

**Your own font** is loaded by `PDF_FONT_LOAD` from a `.ttf` or
`.otf` file (for a `.ttc` collection, the first one). The name is free, but
must not refer to a built-in font:

```basic
PDF_FONT_LOAD(p, "fonts/NotoSansJP-Regular.ttf", "japanisch")
PDF_FONT(p, "japanisch", 12)
PDF_TEXT(p, 20, 40, "請求書")
```

## Text and width

The text is **Unicode** — German, French, Polish, Greek and
Cyrillic work with the built-in fonts. A character that the chosen
font **does not have** (an emoji, a Chinese character in `sans`) is an
**error** and is not set as an empty box: on an invoice a silently vanished
character is worse than a message. In that case your own font helps.

**`PDF_TEXT_WIDTH` measures what is drawn** — with the same text shaping
(rustybuzz) as the setting itself, kerning included. With it, amounts can be
set exactly right-aligned in any font:

```basic
SUB Rechtsbuendig(p AS PDF, x AS FLOAT, y AS FLOAT, text AS STRING)
    PDF_TEXT(p, x - PDF_TEXT_WIDTH(p, text), y, text)
END SUB

PDF_FONT(p, "sans", 10)
Rechtsbuendig(p, 190, 120, "1.234,56 €")
```

A column of numbers in `mono` still often looks better: the digits line up
underneath each other.

## Printing and preview

The module **records** its pages. The same page can therefore go to three
destinations: the file (`PDF_SAVE`), the printer (`PDF_PRINT`) and an image
(`PDF_PREVIEW`). There is no PDF renderer behind it; the renderer is the
operating system ([design, German](../entwurf-drucken.md)).

```basic
PDF_PRINT(p)                                   ' default printer, one copy
PDF_PRINT(p, "Brother MFC-L3760CDW series", 2) ' printer and copies
PDF_PRINT(p, "Microsoft Print to PDF", 1, "ausgabe.pdf")   ' into a file, without a dialog
DIM bild AS IMAGE : bild = PDF_PREVIEW(p, 1, 600)          ' page 1, 600 px wide
PRINT PRINTER_DEFAULT$() : PRINT PRINTERS()                ' for a dialog of your own
```

- **Windows:** GDI on the printer — the driver rasterises. The built-in
  fonts become their Windows siblings (`sans` → Arial, `serif` →
  Times New Roman, `mono` → Courier New), one loaded with `PDF_FONT_LOAD`
  becomes Arial; the position of every text stays, because the program sets
  positions. Millimetres count from the paper edge; the module subtracts the
  unprintable margin.
- **macOS/Linux:** the PDF goes to CUPS (`lp -d drucker -n kopien`), which
  understands PDF — there with the embedded fonts. A `zieldatei` there is
  the PDF itself.
- **`zieldatei`** (target file) is meant for printers that write into a file —
  "Microsoft Print to PDF" then does **not** ask. That is exactly how
  `tests/pruef/drucken.dhtest` checks printing: through a real driver,
  read back with our own reader `tests/pruef/_hilfen/pdftext.dh` —
  the text of both pages and, from the text matrices, the position of the
  right-aligned amount. Measured: two pages in ~0.9 s.
- `PRINTERS()` returns the names as the system knows them, `PRINTER_DEFAULT$()`
  the default printer (`""` if there is none). A trap on Windows: the
  setting "Let Windows manage my default printer" makes the **most recently
  used** printer the default — after printing to "Microsoft
  Print to PDF" that is then the default. If you want a particular printer,
  name it. If the printer is missing or
  does not accept the job, that is an error with the name, not a silent
  nothing.
- `PDF_PREVIEW` draws in raylib's default font on white paper:
  a preview, not an imagesetter. The height follows the aspect ratio; it needs a
  window, because it is an `IMAGE`.

Deliberately not included: printing foreign PDFs (the module prints what it has set)
and a recreated print dialog — `PRINTERS()` plus a dropdown in your
own window does the job, uniformly and testably. Duplex and paper tray
are missing; they can be added later as driver fields.

## Two promises

* **The same input gives the same file** — no creation date is written
  into it, and the document ID is derived from the content. That
  makes checks comparable and a version history readable; if you need a
  date, write it visibly on the page, where it belongs.
* **The content is compressed** (Deflate), and of each font only what was
  used is included.

## What is missing

Images, automatic tables, line wrapping, automatic page numbers,
encryption, right-to-left text in mixed lines. This is a
type case, not a typesetting system — whoever builds a page positions things themselves.
For the case it is meant for (a form with a fixed layout and changing
numbers) that is exactly right; for a flowing report with wrapping it would
be too little. krilla can do images — they can be added later.

Example: [examples/176_rechnung_pdf.dh](../../examples/176_rechnung_pdf.dh) —
a complete invoice with header, address field, item table and
totals block.
