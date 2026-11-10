# Drachenhauch 2026.32

*Die Notizen zu dieser Fassung. Ihr größter Teil ist **Glutfaden**: der
Kreuzstich-Editor PySticky, in 21 Stufen nach Drachenhauch mit GTK 3 als
Oberfläche gebracht. Dabei kamen viele Stolpersteine in Laufzeit und
Sprache ans Licht — BYREF bei Methoden, Namensräume, Fehlerorte, fremde
Bibliotheken —, die jetzt beseitigt sind. Dazu zwei Runden mit echten
Programmen (Texteditor, Fotobetrachter) und die Ortszeit unter macOS und
Linux. **Einige Änderungen betreffen bestehende Programme** (gleich zuerst).*

## Was sich für bestehende Programme ändert

* **BYREF bei Methoden schreibt zurück.** `obj.m(a, b)` mit BYREF-Parametern
  lief bisher still wie BYVAL — die Werte kamen nie zurück. Steht die Klasse
  beim Übersetzen fest (auch bei `Self`), kommen sie jetzt zurück. Ist der Typ
  des Objekts nicht bekannt und wird ein BYREF-Parameter übergeben, ist das ein
  Fehler mit dem Hinweis auf `DIM x AS Klasse` (vorher: still BYVAL). Eine
  Überschreibung, die BYREF an anderen Stellen hat, ist ebenfalls ein Fehler.
* **Unter macOS und Linux gilt die Ortszeit.** Bisher war dort die ganze
  Ortszeit UTC: `DATE$`, `TIME$` und `ZEIT_JETZT` gingen in Berlin im Sommer
  zwei Stunden nach, und `ZEIT_PARSE` mit Zone sowie `FILETIME` rechneten
  nicht um. Jetzt kommen Felder und Versatz (samt Sommerzeit am jeweiligen
  Tag) aus der C-Bibliothek, mit den Regeln aus `TZ` bzw. `/etc/localtime`.
  Wer dort Zeiten gespeichert hat, sieht sie um den Zonenversatz verschoben.
  Unter Windows ändert sich nichts.
* **Fehler in Rückrufen fremder Bibliotheken.** Nach einem Fehler laufen
  weitere Rückrufe weiter (bisher lieferte ab dem ersten jeder stumm 0 — in
  einer GTK-Oberfläche reagierte danach kein Knopf mehr). Gemeldet wird der
  Fehler von dem Aufruf, der den Rückruf ausgelöst hat, nicht mehr vom
  nächsten zurückkehrenden irgendwo später.
* **Fehlerorte nennen die echte Datei.** Ein Laufzeitfehler in einer
  importierten Datei sagt `hilfe.dh:3` statt der Zeile in der
  zusammengefügten Quelle unter dem Namen der Hauptdatei; ein Rückruf-Fehler
  sagt `(fall.dh:6)` statt `(Zeile 6)`. `dhrt --check` liefert die Zeile in
  der eigenen Datei und ein neues Feld `file`. Wer Fehlertexte vergleicht oder
  die JSON-Ausgabe auswertet, sieht anderen Text.
* **`TYPEOF` bei Klassen aus einem Namensraum** liefert `KERN.KNOTEN` statt
  des internen `KERN@KNOTEN`; dieselben lesbaren Namen stehen jetzt in
  `--check`, im Sprachserver und in `CODE_CHECK$`.
* **`ZIP_WRITE`** nimmt in `inhalte` Texte und Puffer; alles andere ist ein
  Fehler. Bisher landete ein Puffer still als „<BUFFER n Bytes>“ im Archiv.
* **JSON-Pfade:** `\.` steht für einen Punkt im Schlüssel, `\\` für einen
  Rückstrich (dafür `JSON_KEY$`). Jede andere Folge mit Rückstrich bleibt
  wörtlich — betroffen sind nur Pfade, die `\.` oder `\\` schon enthielten.

## Neu in dieser Fassung

### Glutfaden — ein Kreuzstich-Editor in Drachenhauch

**`examples/glutfaden`** ist PySticky (Python/Qt) als Drachenhauch-Programm
mit GTK 3 über die fremden Bibliotheken: Zeichnen mit Teil- und Rückstichen,
Ebenen, Rückgängig, Bildimport, Export als PDF, HTML, Bild und OXS, PCStitch
und Pattern Maker lesen, Diamond Painting, Stickpfad, Statistik, Planung,
Farbverwaltung, Einstellungen mit eigenen Tastenkürzeln, Drucken, ein helles
und ein dunkles Erscheinungsbild. Jede Stufe ist gegen PySticky selbst
geprüft, wo es etwas zu vergleichen gab; wo PySticky irrte, steht die
Abweichung mit Begründung im Code. Das Programm läuft unter Windows mit dem
GTK von Inkscape.

### Bilder

* **`IMAGE_SCALE(bild, b, h [, filter$])`** nimmt einen Filter: `lanczos`,
  `bikubisch`, `box`, `hamming`, `linear` — gerechnet wie Pillows
  `Image.resize`. Ohne Filter bleibt es beim bilinearen Glätten.
* **`IMAGE_ENHANCE(bild, art$, faktor)`** mit `helligkeit`, `kontrast` und
  `saettigung`, wie Pillows `ImageEnhance`.

### PDF

* **`PDF_CIRCLE`/`PDF_CIRCLE_FILL`**, **`PDF_POLYGON`/`PDF_POLYGON_FILL`**
  (Vieleck aus zwei Feldern), **`PDF_OPACITY`/`PDF_GET_OPACITY`** (echte
  Durchsicht im PDF; Druck und `PDF_PREVIEW` mischen gegen Weiß) und
  **`PDF_DASH(p, strich_mm [, luecke_mm])`** für gestrichelte Linien —
  `PDF_DASH(p, 0)` macht sie wieder durchgezogen.
* **`PDF_TEXT`** nimmt einen Winkel (Grad gegen den Uhrzeigersinn), `PDF_NEW`
  kennt `a2`, `a1` und `a0`.

```basic
DIM p AS PDF : p = PDF_NEW("a4")
PDF_DASH(p, 2, 1)
PDF_RECT(p, 20, 20, 60, 30)
PDF_DASH(p, 0)
PDF_OPACITY(p, 0.4)
PDF_CIRCLE_FILL(p, 50, 35, 10)
PDF_TEXT(p, 20, 70, "schräg", 30)
PDF_SAVE(p, "probe.pdf")
```

### Zeit

* **`ZEIT_UNIX()`** (Unix-Zeit in Sekunden, UTC, mit Bruchteil),
  **`ZEIT_AUS_UNIX(sekunden)`** und **`ZEIT_NACH_UNIX(zeit)`** — die Brücke
  zwischen Zeitstempeln von außen und den Zeitpunkten in Ortszeit, samt
  Sommerzeit.

### JSON

* **`JSON_KEY$(name$)`** maskiert einen Schlüssel als Pfadstück: ein Schlüssel
  mit Punkt (etwa ein Dateiname) war bisher weder lesbar noch schreibbar, weil
  der Punkt den Pfad trennt.

```basic
IMPORT "json"
DIM j AS JSON_HANDLE : j = JSON_PARSE("{}")
JSON_SET_INT(j, "groessen." + JSON_KEY$("bild.png"), 42)
PRINT JSON_GET_INT(j, "groessen." + JSON_KEY$("bild.png"))
```

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

* **`examples/glutfaden`** — siehe oben.
* **`examples/texteditor`** — Reiter, Suchen und Ersetzen, Gehe zu Zeile,
  Zoom; Kodierung und Zeilenenden bleiben beim Speichern, wie sie waren, und
  lassen sich in der Statusleiste umstellen.
* **`examples/fotos`** — ein Fotobetrachter mit GTK 3 als Oberfläche:
  Übersicht mit Vorschaubildern, Einzelansicht mit Blättern, Drehen (auch
  nach den Kameradaten), Originalgröße, Diaschau, Vollbild.

### Behobene Stolpersteine

* **Namensräume** (`IMPORT "x.dh" AS kern`): Konstanten in `CASE`-Zweigen
  wurden nicht umbenannt („Variable nicht deklariert“); eine Sammeldatei ohne
  eigene Namen galt nicht als Namensraum; Klassenfelder in Methoden wurden
  als Namen von außen behandelt (bei gleichnamiger Globale brach das Programm
  ab, bei gleichnamigem Modulnamen wurde still umgeleitet); eine Methode, die
  wie eine Funktion des Moduls heißt, wurde mit umbenannt. Die Meldung für
  einen Namen, den eine Datei ohne `AS` einbindet, nennt Datei und fehlenden
  `IMPORT … AS`.
* **`CATCH e` mit späterem `DIM e`** im Hauptprogramm brach mit „globale
  Variable noch nicht gesetzt“ ab; ein `DIM e` mit anderem Typ warnt jetzt
  mit der Zeile des `CATCH`.
* **Maschinencode:** `BAND`, `BOR`, `BXOR`, `SHL`, `SHR` und `BNOT` fehlten —
  Schleifen damit blieben in der VM (bei einer Million Punkten 217 → 151 ms).
* **`JOIN$`** nimmt auch Felder mit Zahlen und Wahrheitswerten (geschrieben
  wie `STR$`).
* **`a IS b`** mit einer Variablen rechts sagt jetzt, dass `=` gemeint ist,
  statt „unbekannter Typ“.
* **`--check`** warnt bei einem Vergleich mit einem Text, den `JSON_TYPE` nie
  liefert (etwa `"bool"` statt `"boolean"`), und zählt die möglichen auf.
* `IF … THEN DIM …` und ein `FOR`, `WHILE` oder `SELECT` im einzeiligen IF
  sagen jetzt, wie es geht, statt nur „Unerwartetes Token DIM“. Die Meldung
  zu einem unbekannten Kodierungsnamen zählt auch `utf8bom` und `auto` auf.

## Unter der Haube

**Zahlen.** **2153** Befehle in 48 Modulen (vorher 2140), 225 Beispiele auf
der obersten Ebene von `examples/` (die neuen liegen in eigenen Ordnern).
**5287 Fälle in 427 Prüfsammlungen** (vorher 5080 in 400) und **555
Rust-Testfunktionen** (vorher 547). Gezählt werden jetzt nur die
Testattribute: die bisherige Zählung nahm Kommentare mit, die `#[test]`
erwähnen — bei 2026.31 stand deshalb 558 statt 547.

* BYREF bei Methoden ist ein eigener Befehl `CALL_METHOD_BYREF`; er wird nur
  erzeugt, wenn die Klasse beim Übersetzen feststeht.
* Der Wächter für Rückruf-Fehler ist ein eigener Faden; er schreibt nur,
  wenn ein Fehler nach einer Sekunde noch nicht abgeholt ist. Ein kurzer
  Aufruf wie `qsort` bleibt still, der Fehler kommt dort beim Rückkehren an.
* Die Ortszeit unter macOS und Linux kommt aus `localtime_r`; `struct tm` ist
  unter glibc, macOS und musl gleich aufgebaut, eine neue Abhängigkeit
  braucht es nicht.
* `IMAGE_SCALE` mit Filter und `IMAGE_ENHANCE` sind Punkt für Punkt gegen
  Pillow geprüft (`bild_wie_pillow`).

## Was offen bleibt

* Glutfaden und der Fotobetrachter sind nur unter Windows ausprobiert (mit
  dem GTK von Inkscape); unter Linux und macOS überspringen sich ihre Fälle,
  wo kein GTK mit Bildschirm da ist. Von PySticky fehlen noch das
  PDF-Passwort (die PDF-Bibliothek verschlüsselt nicht), Erweiterungen
  (Plugins) und andockbare Leisten.
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
