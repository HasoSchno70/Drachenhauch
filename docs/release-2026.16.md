# Drachenhauch 2026.16

*Die Notizen zu dieser Fassung. Vier Tage und 44 Pull Requests nach 2026.15,
und fast alle drehen sich um eine Frage: wie wird Drachenhauch richtig
schnell? Die Antwort steckt jetzt in der Laufzeit. Heiße Schleifen und
Zahlenfunktionen werden beim Laufen in Maschinencode übersetzt, und der
Interpreter darunter ist auch schneller geworden. Dazu kommen Mauszeiger, die
zu dem passen, worauf sie zeigen.*

## Neu in dieser Fassung

### Maschinencode

Die Laufzeit übersetzt jetzt **Funktionen und Schleifen während des Laufens
in Maschinencode**. Dafür nimmt sie Cranelift, den Übersetzer, den auch
Wasmtime benutzt. Am Programm ändert sich dafür nichts. Eine Funktion wird beim
Start übersetzt, eine Schleife beim ersten Rücksprung, und zwar mit den Typen,
die Drachenhauch ohnehin kennt (`n AS INTEGER` steht im Quelltext, erraten muss
die Laufzeit nichts). Was der Maschinencode nicht kann, übernimmt der
Interpreter mitten im Lauf: ein Befehl, der in einen Fehler läuft, eine Taste,
eine Zeichnung. Fehlermeldung und Zeile kommen immer vom Interpreter, und
**das Ergebnis ist dasselbe** — jede Prüfsammlung läuft in der CI zweimal,
einmal mit Maschinencode und einmal ohne.

Gemessen mit der Messbank (`dhrt run bench_dhrt.dh`), jeweils der beste von
fünf Läufen auf derselben Maschine:

| Programm | 2026.15 | 2026.16 |
|---|---|---|
| Funktionsaufrufe (`fib`) | 271 ms | 3,1 ms |
| Ganzzahl-Schleifen | 545 ms | 9,0 ms |
| Kommazahlen | 289 ms | 5,6 ms |
| Felder | 169 ms | 11,4 ms |
| Objekte mit Methoden | 138 ms | 11,6 ms |
| Teilchen (Objekte in Feldern) | 636 ms | 27,6 ms |
| eingebaute Zahlenbefehle | 266 ms | 3,0 ms |
| Zeichenketten | 4250 ms | 38,4 ms |
| MAPs | 191 ms | 78,0 ms |

`fib(30)` braucht damit 3 ms, Node auf derselben Maschine 6,5 ms. `DHRT_JIT=aus` schaltet den
Maschinencode ab, `dhrt --jit datei.dh` sagt je Funktion, ob sie übersetzt
wird und warum nicht, und `DHRT_JIT_BILANZ=1` zählt am Ende, welche Schleifen
noch im Interpreter gedreht haben.

### Der Interpreter darunter

Auch ohne Maschinencode läuft fast alles schneller, meist um das 1,1- bis
3-fache, bei Zeichenketten um das 80-fache (`x = x + e` kopierte bis dahin den
ganzen Text in jeder Runde).

* **Zeichenketten wachsen an Ort und Stelle.**
* **Befehle, Felder, Objekte, Aufrufe:** jede Aufrufstelle merkt sich, wo ihr
  Befehl oder ihre Methode steht. Globale Felder und Objektfelder haben feste
  Plätze.
* **Konstanten werden beim Übersetzen ausgerechnet**, `CONST` mit festem Wert
  eingesetzt.
* **`LEFT$`, `MID$`, `RIGHT$` und `INSTR`** zählen nur noch bis zur Stelle.
  Vorher legten sie bei jedem Aufruf den ganzen Text als Zeichenliste an.
* **MAPs** halten ihre Einträge dicht in Einfügereihenfolge: eine Million
  Schlüssel brauchen 160 statt 214 MB.

### Mauszeiger

Die Oberfläche zeigt jetzt den Zeiger, der zu der Stelle passt, auf die er
zeigt. Das gilt nicht nur für das Widget, sondern auch für die Stelle darin: die
Schreibmarke über Text, die Hand über einem Verweis oder einem
anklickbaren Feld der Statusleiste, Doppelpfeile an einer Tabellenkante, ein
Fadenkreuz im Farbwähler, beim Ziehen die Hand oder „geht nicht“. Neu sind
außerdem `warten`, `arbeitet` und `hilfe` (unter Windows die echten
Systemzeiger, die Sanduhr dreht sich), `kopieren`, `stift` und `pipette` sowie
eigene Zeiger aus einem Bild (`MOUSE_CURSOR_NEW`). `GUI_SET_CURSOR` gibt einem
Widget seinen eigenen Zeiger, `MOUSE_CURSOR("auto")` gibt den Zeiger an die
Oberfläche zurück.

### Sprache

* **`/` liefert immer eine Kommazahl**: `8 / 2` ist `4.0` statt `4`. Bisher
  hing der Typ am Wert und ließ sich nicht vorab festlegen. Ganzzahlig teilt
  weiter `\`. Einer INTEGER-Variablen lässt sich das Ergebnis zuweisen, solange
  es glatt aufgeht. `dhrt --check` warnt vor einem Feld-Index, der sicher eine
  Kommazahl ist.
* **`DIM` mit mehreren Gruppen**: `DIM n AS INTEGER, s AS STRING = "x"`.
* Überall heißt das Kürzel jetzt `dh` (`DH_*`, „DH-Code“) statt des alten
  `gb` aus GameBasic-Zeiten.

### Werkzeuge

* `dhrt test --schnell` lässt die langsamen Sammlungen (IDE, Werkzeuge, echte
  Fenster) aus: rund 12 statt 50 Minuten.
* `dhrt --typen datei.dh` zeigt, welchen Typ der Übersetzer für jeden Ausdruck
  kennt.
* `dhrt --version` nennt jetzt auch Maschinencode und Mail.

## Unter der Haube

**Zahlen.** Die Befehlsreferenz wächst von 1993 auf **1997** Einträge; es
bleiben 48 Module und 216 Beispiele. **4401 Fälle in 297 Prüfsammlungen**
(vorher 3981 in 283) und **439 Rust-Testfunktionen** (vorher 431). Die Laufzeit
braucht Rust 1.98. Der Windows-Installer wächst von 54 auf **55 MB** (Cranelift).

Wie der Maschinencode gebaut ist und welche Wege verworfen wurden, steht in
`docs/entwurf-maschinencode.md`.

## Was offen bleibt

* Programme mit sehr vielen MAP-Zugriffen gewinnen noch am wenigsten, etwa das
  2,5-fache.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt. Der Maschinencode braucht dort eine Berechtigung
  (`allow-unsigned-executable-memory`), die das Paket mitbringt.
* Der Installer ist nicht signiert; SmartScreen meldet sich beim ersten Start.
