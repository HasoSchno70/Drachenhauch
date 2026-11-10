# Drachenhauch 2026.33

*Die Notizen zu dieser Fassung. Neu ist **Funkenflug**, ein Schaltplan-Editor
in Drachenhauch mit GTK 3. Glutfaden spricht jetzt auch Englisch. Die Laufzeit
ist beim Arbeiten mit Texten, Feldern und Objekten deutlich schneller
geworden. **Eine Änderung betrifft bestehende Programme**, sie steht gleich
zuerst.*

## Was sich für bestehende Programme ändert

* **Ein zweites `DIM` desselben Namens mit anderem Typ ist ein Fehler.**
  Gross- und Kleinschreibung zählen nicht, `pd` und `pD` sind also dieselbe
  Variable. Bisher gab es dafür nur eine Warnung, und abgebrochen wurde erst
  beim Laufen, an einer ganz anderen Stelle („Feld 'x' existiert nicht in
  b“). Das gilt im Hauptprogramm, in SUB/FUNCTION und in Methoden, ebenso für
  `CATCH e` mit einem späteren `DIM e AS T`. Ein zweites `DIM` mit
  **gleichem** Typ bleibt eine Warnung. Im Repository löst kein Programm den
  Fehler aus (geprüft mit `dhrt --check` über alle `.dh`-Dateien).

## Neu in dieser Fassung

### Funkenflug — ein Schaltplan-Editor

**`funkenflug/`** (im Hauptordner des Repositorys) ist ein Schaltplan-Editor
in Drachenhauch mit GTK 3 über die fremden Bibliotheken, die Neufassung der
Idee hinter SparkNode. Er kann:

* Schaltpläne mit Hierarchie, Unterblättern in mehreren Instanzen und Bussen
  zeichnen;
* die elektrischen Regeln prüfen (ERC) und eine Netzliste oder SPICE-Datei
  schreiben;
* mit ngspice simulieren;
* eine Lochraster- oder Streifenraster-Platine entwerfen, Gerber-Dateien
  schreiben und die Platine in 3D zeigen;
* Schaltpläne aus KiCad einlesen und KiCad-Symbolbibliotheken als
  Symbolquelle nehmen.

Funkenflug liegt nicht in den Beispielen und kommt darum nicht mit dem
Installer. Starten mit `dhrt run funkenflug/main.dh`, Einzelheiten in
`funkenflug/README.md`.

### Glutfaden auf Englisch, und mehr aus PySticky

Glutfaden (der Kreuzstich-Editor aus 2026.32) bekam vier weitere Stufen:

* **Deutsch und Englisch:** Einstellungen → Allgemein → Sprache (automatisch,
  Deutsch, English), gültig ab dem nächsten Start. Übersetzt sind
  Hauptfenster, alle 28 Dialoge und die Texte aus dem Kern (Verlauf,
  PDF/HTML-Anleitung, Statistik, Meldungen der Dateileser). Die Tafel hat
  1637 Einträge. Gespeichert wird weiter deutsch, eine OXS-Datei ist in beiden
  Sprachen Byte für Byte gleich.
* **Magnetisches Raster**, **Pinsel nach Stiftdruck** und **Zoom mit zwei
  Fingern**.
* **Laufende Auswahlkante**, Aida-Textur, Ziehen von Garnen auf die
  Farbleiste und von Ebenen (verschieben oder zusammenführen), eigene Ordner
  für Bibliothek und Vorlagen, „Neu in Glutfaden“.

### Der Übersetzer warnt bei Kommazahlen an mehr Stellen

`/` liefert immer eine Kommazahl. `dhrt --check` warnte bisher nur bei einer
Zuweisung an eine INTEGER-Variable. Jetzt warnt es auch bei `RETURN` in einer
`FUNCTION … AS INTEGER` und bei einem Argument an einen INTEGER-Parameter
einer eigenen SUB/FUNCTION. Gefunden bei Funkenflug: Dort lag jeder Punkt um
ein halbes Raster daneben, ohne jede Meldung.

## Schneller

* **Grosse Texte:** `MID$`, `INSTR`, `LEN`, `LEFT$`, `RIGHT$`, `INSTRREV`,
  `BETWEEN$` und `REGEX_FIND_POS` finden eine Stelle in Texten ab 16 KB über
  einen Zeichenindex, ohne den Text bis dorthin abzuzählen. Auf einem Text mit
  5 MB: 5000-mal `MID$` 156 → 1 ms (ASCII) bzw. 5124 → 4 ms (mit Umlauten),
  `LEN` 586 → 0 ms.
* **Anhängen ist linear auch an Feldern, Elementen, MAP-Werten und
  BYREF-Parametern.** `k.text = k.text + "x"`, `a[i] = a[i] + "x"` und
  `m["k"] = m["k"] + "x"` kopierten bei jedem Mal den ganzen Text. Bei
  200 000 Mal: 204 → 8 ms.
* **`ARRAY_PUSH`/`ARRAY_POP`** ohne neuen Speicher je Aufruf (87 → 42 ns),
  **`NEW`** nach einer Vorlage je Klasse (360 → 216 ns).
* **mimalloc** verwaltet den Speicher. Unter Windows kostete jeder Block
  ~20 ns, und jeder Text sind zwei Blöcke.
* `a = []` an ein `ARRAY OF INTEGER/FLOAT` bekommt den getypten Speicher wie
  `DIM a[n]`; der Maschinencode darf dann hineinschreiben (40 → 1 ns je
  Element). Einzelne ASCII-Zeichen aus `MID$`/`LEFT$`/`RIGHT$`/`CHR$` kommen
  aus einer Tafel. In `tools/tempo`: text 19,6 → 9,0 ms, maps 79,9 → 50,5 ms,
  ffi 77,3 → 43,4 ms.

## Unter der Haube

**Zahlen.** 2153 Befehle in 48 Modulen (unverändert), 225 Beispiele auf der
obersten Ebene von `examples/`, **5328 Fälle in 431 Prüfsammlungen** (vorher
5287 in 427) und **560 Rust-Testfunktionen**. Bei 2026.32 standen 555; mit
derselben Zählung (nur `#[test]`-Attribute) waren es dort 545.

* **Kira 0.12.5** (vorher 0.12.4): ein Doku-Fix und eine neuere interne
  Warteschlange, für Programme ändert sich nichts.
* Neue Befehle für das Anhängen an Ort und Stelle (`ADD_STORE_FIELD`,
  `_MEMBER`, `_INDEX`) und das Leihen bei BYREF (`LEIHE_LOCAL`,
  `LEIHE_GLOBAL_SLOT`). Ein Platz wird nie geleert, solange jemand anderes
  ihn sehen könnte; unter dem Debugger wird kopiert.

## Was offen bleibt

* **raylib 6.2** ist erschienen, die Rust-Bindung raylib-rs hat aber noch
  keine Fassung dafür. Unter der Bindung 6.0 baut 6.2 nicht, und
  `DrawCapsule` vertauscht in 6.2 still zwei Argumente. Drachenhauch bleibt
  darum bei raylib 6.0, bis raylib-rs nachzieht.
* Funkenflug, Glutfaden und der Fotobetrachter sind nur unter Windows
  ausprobiert (mit dem GTK von Inkscape).
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
