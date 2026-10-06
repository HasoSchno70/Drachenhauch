# Drachenhauch 2026.22

*Die Notizen zu dieser Fassung. Ein Tag und fünf Pull Requests nach
2026.21: Drachenhauch ruft fremde Bibliotheken auf -- die Windows-API, die
C-Bibliothek des Systems, den Treiber eines Messgeräts.*

## Neu in dieser Fassung

### Fremde Bibliotheken: `DECLARE … LIB`

Was Drachenhauch nicht selbst mitbringt, liegt oft schon als C-Bibliothek auf
dem Rechner. Eine Zeile deklariert eine Funktion daraus, danach ruft man sie
auf wie eine eigene:

```basic
DECLARE FUNCTION MulDiv LIB "kernel32" (zahl AS LONG, mal AS LONG, durch AS LONG) AS LONG
DECLARE FUNCTION hypot LIB "m" (x AS FLOAT, y AS FLOAT) AS FLOAT

PRINT MulDiv(10, 6, 4)      ' 15
PRINT hypot(3.0, 4.0)       ' 5.0
```

* **Eigene Typwörter** in der Zeile sagen, wie breit ein Wert in C ist:
  `BYTE`, `SHORT`, `LONG` (auch vorzeichenlos), `INTEGER`, `ZEIGER`,
  `SINGLE`, `FLOAT`, `BOOLEAN`, `TEXT` (UTF-8), `WTEXT` (`wchar_t`) und
  `BUFFER`. Ein Wert, der nicht passt, ist ein Fehler statt still
  abgeschnitten zu werden.
* **`BYREF`** für Ausgabeparameter, **`ALIAS`** für einen anderen Namen in
  der Bibliothek. `"c"` und `"m"` meinen auf jedem System dessen
  C-Bibliothek, andere Namen werden zur Datei des Systems (`user32.dll`,
  `libsqlite3.so`, `libsqlite3.dylib`).
* Die Bibliothek wird erst beim ersten Aufruf geladen; fehlt sie, ist das
  ein gewöhnlicher Fehler, den `CATCH` abfängt. `--check` zählt die
  Argumente schon beim Übersetzen, Hover und Zur Definition kennen die
  Zeile.
* **Zeiger und Structs:** `TEXT_AUS_ZEIGER$` und `BUFFER_AUS_ZEIGER`
  kopieren Text und Bytes heraus, die eine Bibliothek als Zeiger liefert;
  `BUFFER_ZEIGER` gibt die Adresse eines Puffers, damit ein Struct auf einen
  anderen zeigen kann. Ein Struct selbst ist ein `BUFFER`.
* **Rückrufe:** ein Parameter `AS FUNCTION(a AS ZEIGER, b AS ZEIGER) AS LONG`
  bekommt eine Funktion des Programms -- ihren Namen, eine gebundene Methode
  oder ein Lambda. So sortiert `qsort` mit einem Vergleich in Drachenhauch,
  und `EnumWindows` meldet jedes Fenster. Ein Fehler im Rückruf kommt beim
  Aufruf an, sobald die Bibliothek zurückkehrt.
* **Export:** `dhrt --export` nimmt jede Bibliothek mit, die neben dem
  Programm liegt.

```basic
DECLARE SUB qsort LIB "c" (feld AS BUFFER, n AS ZEIGER, groesse AS ZEIGER, _
                           vergleich AS FUNCTION(a AS ZEIGER, b AS ZEIGER) AS LONG)
qsort(zahlen, 5, 4, FUNCTION(a, b) SGN(zahlBei(a) - zahlBei(b)))
```

Fremder Code läuft ohne Netz: ein falscher Zeiger beendet das ganze Programm.
Für eine Bibliothek, der man nicht traut, gibt es den Weg über einen Auftrag
(`TASK_START`) -- stürzt der ab, läuft das Hauptprogramm weiter und bekommt
einen Fehler.

Mehr in [Fremde Bibliotheken](ffi.md), dazu ein neues Kapitel im Lehrbuch
und das Beispiel `206_fremde_bibliotheken.dh`.

## Behoben

* Die macOS-Prüfung im Paket-Lauf kannte den Ordner `laufzeiten/` neben
  `dhrt` nicht und brach den Bau des `.dmg` ab.
* Ein Absturz in einem Auftrag (`TASK_START`) kam als "unverständliche
  Antwort" mit leerem Text an -- jetzt als "der Auftrag ist abgestürzt
  (Rückgabe N)".

## Unter der Haube

**Zahlen.** Die Befehlsreferenz wächst von 2102 auf **2105** Einträge, es
bleiben 48 Module; **222** Beispiele (vorher 221). **4807 Fälle in 368
Prüfsammlungen** (vorher 4768 in 367) und **503 Rust-Testfunktionen**
(vorher 493).

* Der Aufruf braucht keine C-Bibliothek wie libffi: je Signatur baut
  Cranelift -- dasselbe, das den Maschinencode erzeugt -- einen kleinen
  Übergang, je Rückruf einen Einstieg mit der C-Signatur. Geladen wird über
  `libloading`.
* Ein Fehler in einem Rückruf darf nicht durch die Rahmen der Bibliothek
  laufen; er wird gemerkt und nach der Rückkehr gemeldet. Ein Rückruf aus
  einem fremden Faden wird nicht ausgeführt (die VM läuft auf einem Faden)
  und meldet sich beim nächsten Aufruf.
* Geprüft gegen Bibliotheken, die überall da sind: unter Windows `kernel32`
  und `user32`, auf allen drei Systemen die C-Bibliothek (`qsort`,
  `strdup`, `gettimeofday`, `uname`, `pthread_create` ...). Gezielt
  verfälschte Fassungen der Laufzeit fallen je in ihrem Prüffall.

## Was offen bleibt

* Structs als Wert (nur über einen Zeiger), Funktionen mit variabler
  Argumentzahl wie `printf`, C++ und COM.
* Das Handbuch (`docs/`) gibt es nur auf Deutsch.
* Ein Tray-Symbol unter macOS und Linux.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt; der Installer ist nicht signiert.
