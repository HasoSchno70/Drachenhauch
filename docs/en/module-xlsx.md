# Module `xlsx`

Writing reports as Excel workbooks.

```basic
IMPORT "xlsx"
```

## Why not CSV

CSV already existed, and for merely passing data on it is enough. What CSV
cannot do:

* multiple sheets
* a bold header row, column widths
* number and date formats
* **tell text from numbers** — a postcode `01067` becomes `1067` when a CSV
  is opened in Excel

All of that together is the difference between a data list and a report
ready to hand in.

## Write-only

For **reading** a table, CSV is the way: Excel exports it, and since
[text encoding](builtins-core.md#text-encoding) Drachenhauch also reads
the cp1252 version that comes out of it. An xlsx *reader* would have to
resolve shared strings and styles — a module of its own for a case CSV
already covers.

## Overview

| Function | Purpose |
|---|---|
| `XLSX_NEW([sheet$])` → XLSX | new workbook (first sheet is called `Tabelle1`) |
| `XLSX_SHEET(x, name$)` | another sheet — and it becomes the active one |
| `XLSX_SHEET_COUNT(x)` → INTEGER | how many sheets |
| `XLSX_SET(x, row, column, text$)` | text cell |
| `XLSX_SET_NUM(x, row, column, number)` | number cell |
| `XLSX_SET_DATE(x, row, column, time[, pattern$])` | date cell |
| `XLSX_BOLD(x, row, column[, on])` | make a cell bold |
| `XLSX_BOLD_ROW(x, row[, on])` | whole row bold (the header row) |
| `XLSX_FORMAT(x, row, column, pattern$)` | number format |
| `XLSX_COL_WIDTH(x, column, width)` | column width in characters |
| `XLSX_SAVE(x, path$)` | write |
| `XLSX_CLOSE(x)` | free memory |

**Row and column are 0-based** — as everywhere in Drachenhauch. Excel
itself counts from 1 and names columns with letters; so `(0, 0)` is `A1`,
`(0, 26)` is `AA1`.

## Example

```basic
IMPORT "xlsx"

DIM x AS XLSX
x = XLSX_NEW("Umsatz")

XLSX_SET(x, 0, 0, "Kunde")
XLSX_SET(x, 0, 1, "Betrag")
XLSX_BOLD_ROW(x, 0)
XLSX_COL_WIDTH(x, 0, 28)

XLSX_SET(x, 1, 0, "Schrauben & Muttern GmbH")
XLSX_SET_NUM(x, 1, 1, 1234.5)
XLSX_FORMAT(x, 1, 1, "#,##0.00")

XLSX_SAVE(x, "auswertung.xlsx")
```

## Formats

The pattern is Excel's own notation: `0.00`, `#,##0.00`, `0%`,
`DD.MM.YYYY`, `0.00 "EUR"`. It is written to the file unchanged —
whatever Excel understands there, it understands here too.

**In Excel a date is a number with a format.** `XLSX_SET_DATE` takes
seconds like the [`zeit`](module-zeit.md) module (and like `FILETIME`),
converts them into Excel's day count and sets `DD.MM.YYYY` — without a
format the cell would show the bare day number.

## Limits

* **No formulas, no charts, no colours, no borders.** This is a writer for
  reports, not a spreadsheet program.
* **Sheet names** may have at most 31 characters and must not contain
  `[]:*?/\` — Excel's own rules, checked here instead of when opening.
* **Formatting survives a new assignment**: if you write first and then set
  bold, you can change the value afterwards without repeating everything.

Example: [examples/177_auswertung_xlsx.dh](../../examples/177_auswertung_xlsx.dh).
