# Entwurf: Drucken aus Drachenhauch

> **Stand 05.09.2026: A und C sind gebaut.** `OPENDOC(pfad$)` öffnet eine
> Datei mit ihrem Standardprogramm (Endungsliste); das pdf-Modul zeichnet
> seine Befehle auf und spielt sie auf drei Ziele: `PDF_SAVE`, `PDF_PRINT`
> (GDI unter Windows, `lp` unter macOS/Linux; dazu `PRINTERS()` und
> `PRINTER_DEFAULT$()`) und `PDF_PREVIEW` (die Seite als `IMAGE`). Siehe
> [module-pdf.md](module-pdf.md#drucken-und-vorschau). Prüfstein 1, gemessen:
> zwei Seiten durch „Microsoft Print to PDF“ in eine Datei in ~0,9 s, PyMuPDF
> findet Nummer, Name und den rechtsbündigen Betrag an seiner Stelle
> (`tests/test_drucken.py`). Die Rechnungsverwaltung hat einen Druckdialog aus
> Bordmitteln (Klappliste der Drucker, Kopien, Vorschau) und „PDF öffnen“.
> B ist der Unix-Teil von C; D bleibt ungebaut.

*Untersuchung, keine Umsetzung.* Der zweite Architekturpunkt der Lückenliste
nach dem sechsten Piloten (nach den [Fenstern](entwurf-native-fenster.md)):
die Rechnungsverwaltung schreibt eine PDF-Datei — und dann? Ein Programm, das
Rechnungen schreibt, will sie auf Papier bringen, ohne dass der Nutzer erst
den Explorer öffnet. Dieses Papier misst den Stand, prüft die fertigen
Bausteine, die sich einbinden ließen (so, wie raylib und Kira eingebunden
sind), entwirft vier Wege und empfiehlt einen. Die Entscheidung fällt jemand
anders.

Alle Angaben sind geprüft, nicht angenommen — Stand 05.09.2026, diese
Maschine (Windows 11) und docs.rs.

## 1. Was heute geht

| | Befehle | Grenze |
|---|---|---|
| Seiten setzen | `pdf`-Modul: `PDF_NEW/PAGE/FONT/COLOR/TEXT/TEXT_WIDTH/LINE/RECT/RECT_FILL/SAVE`, 14 Standardschriften, Millimeter von oben, Textbreite für Helvetica/Times/Courier | **keine Bilder**, keine eingebetteten Schriften (nur WinAnsi), keine Tabellenhilfe |
| Tabellen | `xlsx`-Modul | ist eine Mappe, keine Seite |
| Bildschirm sichern | `SAVESCREENSHOT`, `IMAGE_SAVE` (png/bmp/jpg/tga) | eine Datei, kein Papier |
| Fremde Programme | `SHELL`, `SHELL_START`, `SHELL_OUT$` | beliebig — also auch gefährlich; keine Hilfe beim Finden des richtigen |
| Etwas öffnen | `OPENURL` | **nur `http://` und `https://`**, absichtlich: raylib reicht die Zeichenkette an die Shell weiter |
| Drucker | — | kein Befehl kennt einen Drucker |

Ein Drachenhauch-Programm kann heute also eine PDF schreiben und ihren Pfad in
die Statuszeile stellen. Alles Weitere tut der Nutzer von Hand.

**Diese Maschine, gemessen:**

```text
Get-Printer:  Brother MFC-L3760CDW series Printer   (Standard, WSD)
              Brother MFC-L3760CDW series           (Netz)
              Brother PC-FAX v.3.2
              Microsoft Print to PDF                (PORTPROMPT:)
              OneNote (Desktop)
.pdf          -> MSEdgePDF, Verben: open, runas   -- KEIN "print"
lp / lpr      -> nicht vorhanden (kein CUPS unter Windows)
Ghostscript   -> nicht installiert
```

Zwei Folgen daraus, die für jede Windows-Maschine ohne Acrobat gelten: der
Weg „PowerShell `Start-Process -Verb Print`" scheitert, weil Edge das
Druck-Verb nicht anmeldet — und es gibt **kein Bordmittel, das eine PDF-Datei
still zum Drucker bringt**. Wer unter Windows druckt, zeichnet die Seite
selbst (GDI) oder braucht ein Programm, das PDF versteht.

## 2. Was „Drucken" konkret heißt

| Fall | Beispiel | Anteil |
|---|---|---|
| **a** ein Dokument auf Papier | Rechnung, Lieferschein, Etikett, Bericht | der Alltag |
| **b** Druckerwahl, Kopien, Papierfach, Duplex | „auf den Etikettendrucker, zweimal" | dazu, selten ohne a |
| **c** Vorschau im Programm | sehen, was rauskommt, bevor es rauskommt | ein Komfort, den Nutzer erwarten |
| **d** den Bildschirm drucken | ein Diagramm, eine Karte | selten; `SAVESCREENSHOT` + a |

Fall a ist die Entscheidung. Fall b ist bei den meisten Wegen billig
mitzunehmen (Druckerliste, Kopienzahl), Duplex und Papierfach nicht.

## 3. Fertige Bausteine, geprüft

Der Wunsch war ausdrücklich: Fertiges einbinden, wie raylib und Kira. Für
den Druck gibt es **kein raylib** — keine Bibliothek, die auf allen drei
Systemen eine Seite entgegennimmt und druckt. Es gibt Spooler-Anbindungen und
PDF-Werkzeuge:

| Baustein | Was er kann | Was er nicht kann | Urteil |
|---|---|---|---|
| **`printers` 2.3.0** (Rust, MIT; CUPS unter Unix, winspool unter Windows) | Drucker auflisten, Standarddrucker, Datei oder Bytes an den Spooler geben, Kopien, `document-format` | Er **rendert nichts**: unter Windows gehen die Bytes als RAW an den Drucker — das funktioniert nur, wenn der Drucker das Format selbst versteht (manche Laserdrucker können PDF direkt, „Microsoft Print to PDF" und die meisten Tintenstrahler nicht). Die Doku verweist für PDF auf Ghostscript. Kein Dialog. | unter Unix brauchbar (CUPS versteht PDF), unter Windows eine Lotterie |
| **CUPS `lp`** (macOS, Linux; schon da) | `lp -d drucker datei.pdf` — CUPS wandelt PDF selbst | gibt es unter Windows nicht | der Unix-Weg, ohne Abhängigkeit |
| **`windows` 0.61** (Rust; **schon im Baum** über andere Abhängigkeiten) | Win32 GDI + winspool: Standarddrucker, `CreateDC`, `StartDoc`/`StartPage`, `TextOut`, Linien, Rechtecke, Bitmaps, `PrintDlg` | nur Windows | der Windows-Weg, ohne neue Abhängigkeit |
| **`printpdf` 0.12.8** (Rust, MIT) | PDF erzeugen **mit eingebetteten TrueType-Schriften und Bildern**, Vektoren; 16 Abhängigkeiten | druckt nicht, rendert nicht zu Pixeln (nur SVG) | ein Kandidat für die nächste Stufe des **pdf-Moduls** (Bilder, Umlaute in eigenen Schriften), nicht fürs Drucken |
| **`pdfium-render`** (Rust, MIT/Apache) | PDF zu Bitmap rendern (Chromiums Pdfium) | braucht eine **fremde Pdfium-DLL** neben der Exe oder statisch aus fremder Quelle | nein — dasselbe Urteil wie bei jeder Abhängigkeit, die nicht `cargo build` allein bringt |
| Systemdialog `PrintDlg`/`PrintDlgEx` (Windows) | Druckerwahl, Kopien, Seitenbereich, DEVMODE | nur Windows; unter macOS `NSPrintOperation`, unter Linux GTK — drei APIs | für Fall b möglich, aber ein eigener gui-Dialog mit Druckerliste ist einheitlicher und testbar |

Der Befund dahinter: **Drachenhauch zeichnet seine Seiten selbst.** Das
pdf-Modul ist kein Wrapper um einen PDF-Renderer, es schreibt Text, Linien
und Rechtecke in Millimetern. Damit braucht es zum Drucken keinen
PDF-Renderer — es braucht nur ein zweites **Ziel** für dieselben Befehle.

## 4. Vier Wege

### A. Die Datei öffnen lassen

Ein Befehl, der eine **lokale Datei mit ihrem Standardprogramm** öffnet (das
Gegenstück zu `OPENURL`, das heute nur `http` darf): die PDF geht in Edge
oder Vorschau auf, der Nutzer druckt mit Strg+P. Begrenzt auf Dokument-
Endungen (pdf, png, jpg, txt, csv, xlsx, html), damit aus dem Befehl kein
Programmstarter wird.

```text
OPENDOC("rechnung_2026-0002.pdf")     ' oeffnet den Standard-Betrachter
```

- **Kosten:** ~40 Zeilen (`ShellExecute` / `open` / `xdg-open`), ein Tag mit
  Tests und Doku.
- **Was fehlt:** Druckerwahl, still drucken, Vorschau im Programm. Fall a
  nur mit einem Klick des Nutzers.
- **Risiko:** keins. Nützlich unabhängig von allem Weiteren (auch für xlsx,
  csv, Bilder).

### B. Die Datei an den Drucker geben

`printers`-Crate oder `lp`: die fertige PDF an den Spooler. Unter macOS und
Linux **funktioniert das** — CUPS versteht PDF. Unter Windows nur bei
Druckern mit eigener PDF-Sprache; „Microsoft Print to PDF", die Tinten-
strahler und die Faxe bekommen Rohbytes und drucken Müll oder nichts. Man
müsste dem Nutzer sagen: „geht bei manchen Druckern".

- **Kosten:** klein (eine Abhängigkeit oder `SHELL("lp", ...)`).
- **Risiko:** hoch auf der Plattform, auf der die meisten Nutzer sind. Ein
  Befehl, der still nichts tut, ist schlimmer als keiner.

### C. Die Seite selbst zeichnen — das pdf-Modul bekommt Ziele

Das pdf-Modul zeichnet Text, Linien und Rechtecke ohnehin selbst. Heute
schreibt es sie sofort in den PDF-Inhaltsstrom; würde es sie **aufzeichnen**
(wie `graphics.rs` seine Zeichenbefehle), ließe sich dieselbe Seite auf drei
Ziele abspielen:

1. **PDF** — wie heute (`PDF_SAVE`).
2. **Drucker** — unter Windows über GDI (`windows`-Crate, schon im Baum):
   Standarddrucker oder gewählter, `StartDoc`, je Seite `StartPage`,
   Millimeter → Geräteeinheiten über `GetDeviceCaps`, Text mit den GDI-
   Entsprechungen der Standardschriften (Helvetica → Arial, Times → Times
   New Roman, Courier → Courier New), Linien und Rechtecke; unter macOS und
   Linux die PDF an `lp`. Dazu eine Druckerliste und der Standarddrucker.
3. **Vorschau** — die Seite als `IMAGE` gerendert, damit ein Programm sie in
   einem gui-Fenster zeigt (Fall c), mit den Mitteln, die `graphics.rs`
   hat.

```text
DIM p AS PDF : p = PDF_NEW()
... wie heute ...
PDF_SAVE(p, "rechnung.pdf")                  ' Ziel 1
PDF_PRINT(p)                                 ' Ziel 2: Standarddrucker
PDF_PRINT(p, "Brother MFC-L3760CDW", 2)      ' Drucker und Kopien
DIM bild AS IMAGE : bild = PDF_PREVIEW(p, 1) ' Ziel 3: Seite 1 als Bild
PRINT PRINTER_DEFAULT$() : PRINT PRINTERS()  ' fuer einen eigenen Dialog
```

- **Was man bekommt:** Fall a still und richtig auf jedem Drucker, der
  einen Windows-Treiber hat (der Treiber rastert, nicht wir); Fall b als
  Druckerliste und Kopien; Fall c ohne fremden Renderer. Rechtsbündige
  Zahlen bleiben rechtsbündig, weil das Programm Positionen setzt und keine
  GDI-Breiten braucht. Kein Ghostscript, keine DLL, keine neue Abhängigkeit.
- **Die Grenzen:** nur unsere Primitive — Bilder erst, wenn das pdf-Modul
  Bilder kann (dann in allen drei Zielen zugleich). GDI-Schriften sind nicht
  Helvetica: das Bild auf Papier weicht von der PDF um die Breite einiger
  Buchstaben ab. Kein Duplex, kein Papierfach (das sind DEVMODE-Felder;
  nachrüstbar, aber nicht am Anfang). Unter macOS/Linux ist Ziel 2 die PDF
  über `lp` — also Weg B, wo er funktioniert.
- **Prüfbar, und das ist der Punkt:** „Microsoft Print to PDF" nimmt in
  `DOCINFO` einen Ausgabepfad und fragt dann **nicht** nach — der Test
  druckt die Rechnung dorthin, PyMuPDF liest die Datei und findet Nummer und
  Brutto. Das läuft auf dem Windows-Runner der CI, denn den Drucker bringt
  Windows mit. Ein Druckweg, den ein Test durch einen echten Treiber
  schickt, ist mehr als die meisten Werkzeuge vorweisen.
- **Aufwand:** Aufzeichnung im pdf-Modul ~100 Zeilen, GDI-Ziel ~400,
  Vorschau ~150, `lp`-Weg ~40, Druckerliste (winspool `EnumPrinters` /
  `lpstat -a`) ~80; dazu Tests, Doku, Lehrbuch, der Pilot bekommt einen
  Druck-Knopf. Drei bis vier Tage.
- **Risiko:** mittel und bekannt: GDI-Koordinaten (Geräteeinheiten,
  druckbarer Bereich beginnt nicht bei 0), Schriftauswahl je Sprache,
  ein Drucker, der offline ist (StartDoc schlägt fehl → Meldung). Nichts
  davon global oder plattformübergreifend versteckt.

### D. Native Druckdialoge und ein PDF-Renderer

Das Qt-Niveau: `PrintDlgEx` mit allen Treiberoptionen, macOS
`NSPrintOperation`, Linux GTK-Druckdialog, dazu Pdfium für Vorschau und
Druck beliebiger PDFs (auch fremder). Drei Plattformen, drei APIs, eine
fremde DLL. Wochen, und die DLL widerspricht der Regel, dass `cargo build`
alles bringt.

- **Nicht verhältnismäßig.** Was D über C hinaus bringt (Duplex,
  Papierfach, fremde PDFs drucken), braucht ein Rechnungsprogramm nicht am
  Anfang, und Duplex lässt sich C später als DEVMODE-Feld nachreichen.

## 5. Nebeneinander

| | A öffnen | B an Spooler | C Seite zeichnen | D nativ |
|---|---|---|---|---|
| Fall a (Dokument still drucken) | nein (Nutzer klickt) | Unix ja, Windows Lotterie | **ja** | ja |
| Fall b (Drucker, Kopien) | nein | teils | Liste + Kopien | alles |
| Fall c (Vorschau) | im fremden Programm | nein | **ja, als IMAGE** | ja |
| Neue Abhängigkeit | keine | `printers` oder keine | **keine** (`windows` ist da) | Pdfium-DLL |
| Automatisch prüfbar | Datei existiert | kaum | **ja, über Print-to-PDF + PyMuPDF** | kaum |
| Aufwand | 1 Tag | 1 Tag | 3–4 Tage | Wochen |
| Web-Bau | Befehl = Fehler | Fehler | Fehler | Fehler |

## 6. Empfehlung

**A sofort, C als Bauentscheidung — B nur als der Unix-Teil von C, D nicht.**

A ist einen Tag wert, unabhängig vom Drucken: ein Programm, das eine Datei
schreibt, soll sie zeigen können. C ist der einzige Weg, der auf Windows
still und richtig druckt, ohne Ghostscript und ohne DLL — und der einzige,
den ein Test durch einen echten Treiber schicken kann. Dass er nur unsere
eigenen Seiten druckt und keine fremden PDFs, ist keine Schwäche: das
pdf-Modul ist genau dafür da, und ein Rechnungsprogramm druckt seine
Rechnung, nicht beliebige Dateien.

Prüfsteine für C, jeder messbar:

1. Der Pilot druckt eine Rechnung auf „Microsoft Print to PDF" in eine
   Zieldatei; PyMuPDF findet dort Nummer, Kundenname und Brutto — auf dem
   Windows-Runner der CI, ohne Dialog.
2. Dieselbe Rechnung als Vorschau: `IMAGE` mit der Seitenbreite in Pixeln,
   in einem gui-Fenster gezeigt; der Test prüft, dass das Bild nicht leer
   ist und die Textzeilen dort liegen, wo das PDF sie hat.
3. Unter macOS/Linux geht die PDF an `lp`; in der CI gibt es kein CUPS,
   dort wird nur die Meldung geprüft („kein Drucker").

Was C **nicht** bekommen sollte: einen Renderer für fremde PDFs und einen
nachgebauten Windows-Druckdialog. Das eine ist eine DLL, das andere sind
drei Plattformen — beides der Anfang von D.

## 7. Was ohne Entscheidung schon geht

- `SHELL_START("cmd", "/c", "start", "", pfad$)` öffnet unter Windows eine
  Datei mit ihrem Standardprogramm — der Weg A mit Bordmitteln, nur ohne
  die Schutzliste der Endungen und nicht plattformübergreifend.
- Unter macOS/Linux druckt `SHELL("lp", "-d", drucker$, pfad$)` eine PDF
  heute schon — CUPS versteht sie.
- Die Rechnungsverwaltung schreibt ihre PDF neben die Datenbank und nennt
  den Pfad in der Statuszeile; bis A gebaut ist, ist das der Weg.
