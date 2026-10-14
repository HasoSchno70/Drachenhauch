# Drachenhauch 2026.32

*Die Notizen zu dieser Fassung. Zwei Runden, in denen echte Programme
gebaut wurden — ein Texteditor und ein Fotobetrachter mit GTK als
Oberfläche —, dazu die Ortszeit unter macOS und Linux. **Zwei Änderungen
betreffen bestehende Programme** (gleich zuerst).*

## Was sich für bestehende Programme ändert

* **Unter macOS und Linux gilt die Ortszeit.** Bisher war dort die ganze
  Ortszeit UTC: `DATE$`, `TIME$` und `ZEIT_JETZT` gingen in Berlin im Sommer
  zwei Stunden nach, und `ZEIT_PARSE` mit Zone sowie `FILETIME` rechneten
  nicht um. Jetzt kommen Felder und Versatz (samt Sommerzeit am jeweiligen
  Tag) aus der C-Bibliothek, mit den Regeln aus `TZ` bzw. `/etc/localtime`.
  Wer dort Zeiten gespeichert hat, sieht sie um den Zonenversatz verschoben.
  Unter Windows ändert sich nichts.
* **Nach einem Fehler in einem Rückruf laufen weitere Rückrufe weiter.**
  Bisher lieferte ab dem ersten Fehler jeder weitere Rückruf stumm 0 — in
  einer GTK-Oberfläche reagierte danach kein Knopf mehr, nicht einmal der
  zum Schließen. Gemeldet wird weiterhin der erste Fehler, sobald die
  Bibliothek zurückkehrt, abfangbar mit `CATCH`.

## Neu in dieser Fassung

### Dateien und Kodierung

* **`FILE_ENCODING$(pfad$)`** sagt, welche Kodierung `"auto"` für eine Datei
  wählt: `utf8`, `utf8bom`, `cp1252` oder `binaer`. Jeder Name außer
  `binaer` geht beim Schreiben wieder hinein — ein Editor, der eine
  cp1252-Datei öffnet und sichert, schreibt sie nicht mehr still als UTF-8
  zurück.
* **Die Kodierung `utf8bom`** (auch `utf-8-bom`, `utf-8-sig`) liest wie
  `utf8` und schreibt das BOM, das Excel und viele Windows-Programme
  erwarten — nur vor eine leere Datei, `WRITELINE` und `APPENDFILE` setzen
  kein zweites.

```basic
DIM kod AS STRING : kod = FILE_ENCODING$("alt.txt")
DIM f AS FILE : f = OPENFILE("alt.txt", "r", kod)
DIM text AS STRING : text = READALL$(f)
CLOSEFILE(f)
WRITEALL("alt.txt", text + "!", kod)     ' dieselbe Kodierung, dasselbe BOM
```

### Schrift

* **`SETFONT(font, groesse)`**: scharf laden, klein zeichnen in einem
  Aufruf. `SETFONT` übernimmt sonst die Größe, in der die Schrift geladen
  wurde — wer bei 32 lud, bekam Menü und Statusleiste in 32.

### Fremde Bibliotheken

* **Ein Fehler im Rückruf hängt das Programm nicht mehr stumm auf.** Kehrt
  die Bibliothek nicht zurück (`gtk_main`, eine Nachrichtenschleife), steht
  der Fehler nach einer Sekunde in der Fehlerausgabe, mit Zeile und der
  Funktion, die noch läuft; jeder weitere sofort. Bisher sah man nichts.
* **Die Zeile stimmt:** `ERROR_LINE()` nennt die Zeile im Rückruf, an der
  es scheiterte. Bisher trug jeder spätere Fehler die Zeile des ersten.
* **Unter Windows kommt der Ordner einer über den vollen Pfad geladenen
  Bibliothek in den Suchpfad** — lädt sie selbst Module nach (GdkPixbuf
  seine Bildlader, GStreamer und Qt ihre Plugins), finden die ihre
  Abhängigkeiten. GTK aus Inkscape brach vorher beim ersten Symbol ab.

### Neue Beispiele

* **`examples/texteditor`** — Reiter, Suchen und Ersetzen, Gehe zu Zeile,
  Zoom; Kodierung und Zeilenenden bleiben beim Speichern, wie sie waren, und
  lassen sich in der Statusleiste umstellen. Oberfläche in DejaVu Sans, Text
  in DejaVu Sans Mono.
* **`examples/fotos`** — ein Fotobetrachter mit GTK 3 als Oberfläche und
  eigenem Stylesheet: Übersicht mit Vorschaubildern, Einzelansicht mit
  Blättern, Drehen (auch nach den Kameradaten), Originalgröße, Diaschau,
  Vollbild.

### Kleineres

* `IF … THEN DIM …` und ein `FOR`, `WHILE` oder `SELECT` im einzeiligen IF
  sagen jetzt, wie es geht, statt nur „Unerwartetes Token DIM“.
* Die Meldung zu einem unbekannten Kodierungsnamen zählt auch `utf8bom` und
  `auto` auf.

## Unter der Haube

**Zahlen.** **2141** Befehle in 48 Modulen (vorher 2140), 225 Beispiele auf
der obersten Ebene von `examples/` (die zwei neuen liegen in eigenen
Ordnern). **5098 Fälle in 403 Prüfsammlungen** (vorher 5080 in 400) und
**537 Rust-Testfunktionen**. Gezählt werden jetzt nur die Testattribute: die
bisherige Zählung nahm Kommentare mit, die `#[test]` erwähnen — bei 2026.31
stand deshalb 558 statt 537.

* Der Wächter für Rückruf-Fehler ist ein eigener Faden; er schreibt nur,
  wenn ein Fehler nach einer Sekunde noch nicht abgeholt ist. Ein kurzer
  Aufruf wie `qsort` bleibt still, der Fehler kommt dort wie bisher beim
  Rückkehren an.
* Die Ortszeit unter macOS und Linux kommt aus `localtime_r`; `struct tm` ist
  unter glibc, macOS und musl gleich aufgebaut, eine neue Abhängigkeit
  braucht es nicht.

## Was offen bleibt

* Der Fotobetrachter ist nur unter Windows ausprobiert (mit dem GTK von
  Inkscape); unter Linux und macOS überspringt sich sein Fall, wo kein GTK
  mit Bildschirm da ist.
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
