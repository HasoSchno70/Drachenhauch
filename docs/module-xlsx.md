# Modul `xlsx`

Auswertungen als Excel-Mappe schreiben.

```basic
IMPORT "xlsx"
```

## Warum nicht CSV

CSV gab es schon, und zum bloßen Weitergeben von Daten reicht es. Was CSV
nicht kann:

* mehrere Blätter
* eine fette Kopfzeile, Spaltenbreiten
* Zahlen- und Datumsformate
* **Text von Zahl unterscheiden** — eine Postleitzahl `01067` wird beim
  Öffnen einer CSV in Excel zu `1067`

Alles zusammen ist der Unterschied zwischen einer Datenliste und einer
abgabefertigen Auswertung.

## Nur schreibend

Zum **Lesen** einer Tabelle ist CSV der Weg: Excel exportiert es, und seit
der [Textkodierung](builtins-core.md#textkodierung) liest Drachenhauch auch
die cp1252-Fassung, die dabei herauskommt. Ein xlsx-*Leser* müsste geteilte
Zeichenketten und Formatvorlagen auflösen — ein eigenes Modul für einen Fall,
den CSV schon deckt.

## Übersicht

| Funktion | Zweck |
|---|---|
| `XLSX_NEW([blattname$])` → XLSX | neue Mappe (erstes Blatt heißt `Tabelle1`) |
| `XLSX_SHEET(x, name$)` | weiteres Blatt — und es wird das aktive |
| `XLSX_SHEET_COUNT(x)` → INTEGER | wie viele Blätter |
| `XLSX_SET(x, zeile, spalte, text$)` | Textzelle |
| `XLSX_SET_NUM(x, zeile, spalte, zahl)` | Zahlzelle |
| `XLSX_SET_DATE(x, zeile, spalte, zeit[, muster$])` | Datumszelle |
| `XLSX_BOLD(x, zeile, spalte[, an])` | fett |
| `XLSX_BOLD_ROW(x, zeile[, an])` | ganze Zeile fett (die Kopfzeile) |
| `XLSX_FORMAT(x, zeile, spalte, muster$)` | Zahlenformat |
| `XLSX_COL_WIDTH(x, spalte, breite)` | Spaltenbreite in Zeichen |
| `XLSX_SAVE(x, pfad$)` | schreiben |
| `XLSX_CLOSE(x)` | Speicher freigeben |

**Zeile und Spalte sind 0-basiert** — wie überall in Drachenhauch. Excel
selbst zählt ab 1 und benennt Spalten mit Buchstaben; `(0, 0)` ist also `A1`,
`(0, 26)` ist `AA1`.

## Beispiel

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

## Formate

Das Muster ist Excels eigene Schreibweise: `0.00`, `#,##0.00`, `0%`,
`DD.MM.YYYY`, `0.00 "EUR"`. Es wird unverändert in die Datei geschrieben —
was Excel dort versteht, versteht es auch hier.

**Ein Datum ist in Excel eine Zahl mit Format.** `XLSX_SET_DATE` nimmt
Sekunden wie das Modul [`zeit`](module-zeit.md) (und wie `FILETIME`), rechnet
sie in Excels Tageszählung um und setzt `DD.MM.YYYY` — ohne Format stünde in
der Zelle die nackte Tageszahl.

## Grenzen

* **Keine Formeln, keine Diagramme, keine Farben, keine Rahmen.** Das ist ein
  Schreiber für Auswertungen, kein Tabellenprogramm.
* **Blattnamen** dürfen höchstens 31 Zeichen haben und kein `[]:*?/\`
  enthalten — Excels eigene Regeln, hier geprüft statt beim Öffnen.
* **Die Gestaltung überlebt eine neue Zuweisung**: wer erst schreibt und dann
  fett setzt, kann den Wert danach ändern, ohne alles zu wiederholen.

Beispiel: [examples/177_auswertung_xlsx.dh](../examples/177_auswertung_xlsx.dh).
