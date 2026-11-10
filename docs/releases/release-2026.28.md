# Drachenhauch 2026.28

*Die Notizen zu dieser Fassung. Einen Tag nach 2026.27, wieder aus zwei
Dogfooding-Runden: eine 3D-Werkstatt und ein Feed-Leser. Beide wurden mit
echten Tasten und Klicks bedient. **Eine Änderung betrifft bestehende
Programme:** Ein `DIM` in einer Schleife fängt jetzt jede Runde frisch an (siehe
unten).*

## Neu in dieser Fassung

### Ein `DIM` in einer Schleife fängt jede Runde frisch an

Bisher behielt ein `DIM` im Schleifenrumpf den Wert der vorigen Runde. Ein
Feld mit Größe im Hauptprogramm wurde dagegen jedes Mal neu angelegt, in
einer SUB aber wieder nicht. Jetzt gilt überall dasselbe, wie in FreeBASIC
und QBasic: 0, `""`, `FALSE`, ein leeres Feld oder eine leere MAP. Wer einen
Wert über die Runden behalten will, schreibt das `DIM` vor die Schleife:

```basic
DIM summe AS INTEGER          ' vor der Schleife: zählt mit
DIM i AS INTEGER
FOR i = 1 TO 3
    DIM zeile AS STRING       ' in der Schleife: jede Runde leer
    zeile = zeile + "*"
    summe = summe + i
    PRINT zeile; " "; summe   ' * 1, * 3, * 6
NEXT
```

Ein Programm, das sich auf das Behalten verlassen hat, verhält sich jetzt
anders. Eine Suche über alle Beispiele, Werkzeuge und die IDE hat keines
gefunden. Der Maschinencode übersetzt solche Schleifen weiter, und das Tempo
ist unverändert.

### Daten aus dem Netz

* **`ZEIT_PARSE`** liest Zeitzonen und das Datum aus RSS und E-Mail. Die Zeit
  wird in die Ortszeit umgerechnet:

  ```basic
  IMPORT "zeit"
  PRINT ZEIT_TEXT$(ZEIT_PARSE("Sat, 10 Oct 2026 08:30:00 +0200"))
  PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-10-09T12:00:00Z"))
  ```

  Das Programm erkennt ISO mit `Z` oder `+hh:mm` und RFC 822 mit Versatz
  oder Namen (GMT, UT, EST bis PDT, CET/MEZ, CEST/MESZ).
* **`TASK_START`** reicht seine Argumente über die Standardeingabe weiter
  statt über die Kommandozeile. Ein Auftrag, der einen Feed von 200 KB
  zerlegen sollte, scheiterte bisher an der Länge.
* **`HTML_TEXT`** hielt jedes `<` für den Anfang eines Tags. Aus
  `a < b und c > d` wurde `a d`. Jetzt beginnt ein Tag wie im Browser nur
  vor einem Buchstaben, `/`, `!` oder `?`.

### 3D

* **Zahlen mit Exponent** wie `1e9` und `2.5E-3` sind im Quelltext erlaubt;
  vorher waren sie ein Fehler.
* **`KEYHIT("z")`** meint die Taste, auf der das Z steht. `KEY_Z` meint die
  Lage nach der US-Tastatur, auf QWERTZ ist das die Y-Taste. Strg+Z mit
  `KEY_Z` war darum auf deutschen Tastaturen tot.
* **`MODEL_MATRIX` wirft Schatten.** Beleuchtete Modelle können
  halbdurchsichtig sein.
* **`RAY_HIT_BOX` und `RAY_HIT_MODEL`** liefern die Entfernung in
  Welt-Einheiten, auch bei einer nicht normierten Richtung.
* **`MODEL_MATRIX_WIRES`** und **`CUBE_MATRIX_WIRES`** zeigen Gitter und
  Kanten gedrehter Körper.
* **`PHYS3D_SLEEP`/`PHYS2D_SLEEP`** legen ruhende Körper schlafen. Ein Stapel
  aus 1323 Würfeln kostet dann 0,3 statt 20 ms je Schritt.
* **`IMAGE_WIDTH`/`IMAGE_HEIGHT`** sind zweite Namen für
  `IMAGEWIDTH`/`IMAGEHEIGHT`.

### Kleineres

* In der eingebauten Schrift wechseln `€` und andere Zeichen außerhalb von
  Latin-1 nur selbst die Schrift. Vorher sprang die ganze Zeile um.
* Ein falscher Schlüssel bei `GUI_THEME_SET/GET` und `GUI_METRIC_SET/GET`
  nennt die gültigen.

### Zwei neue Beispiele

* **`examples/werkstatt3d`**: eine Bauklotz-Werkstatt. Klötze setzt man per
  Strahl. Dazu gibt es Licht, Schatten, Rückgängig, Sichern und einen
  Einsturz über `physics3d`.
* **`examples/feedleser`**: liest RSS und Atom. Abgerufen wird im
  Hintergrund, die Einträge liegen in SQLite. Dazu kommen eine Vorschau, ein
  Symbol im Infobereich und eine Mitteilung bei Neuem.

## Unter der Haube

**Zahlen.** **2120** Befehle in 48 Modulen (vorher 2112), 225 Beispiele auf
der obersten Ebene von `examples/` (die zwei neuen liegen in eigenen
Ordnern). **4980 Fälle in 387 Prüfsammlungen** (vorher 4939 in 380) und
**546 Rust-Testfunktionen**.

* Den Feed-Leser prüft ein Gegenserver auf 127.0.0.1. Er liefert RSS, RDF
  und Atom, dazu ein Dokument, das kein Feed ist, und eine Adresse, die es
  nicht gibt.
* Die Bilder der Licht-Beispiele sind Punkt für Punkt mit 2026.25
  verglichen.

## Was offen bleibt

* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Ob eine echte Linux-Leiste das Tray-Symbol zeigt, ist nicht ausprobiert.
  Auf einem echten Mac läuft Drachenhauch. Das Paket ist nicht beglaubigt,
  der Installer nicht signiert.
* `ZEIT_PARSE` rechnet die Ortszeit nur unter Windows um; unter macOS und
  Linux gilt UTC.
