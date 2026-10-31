# Entwurf: Felder mit eigener Untergrenze

**Stand:** 2026-10-30, entschieden 2026-10-31 (Abschnitt 11); nichts davon gebaut.

```
DIM tage[1 TO 31] AS INTEGER
DIM feld[-10 TO 10, -10 TO 10] AS FLOAT
```

Heute zählt jedes Feld ab 0: `DIM a[10]` hat `a[0]` bis `a[9]`. Wer aus
FreeBASIC, Visual Basic oder Pascal kommt, schreibt `DIM a(1 TO 10)` und
bekommt eine Umsteiger-Meldung. Wer ein Koordinatenfeld um den Ursprung
braucht, rechnet den Versatz selbst (`feld[x + 10, y + 10]`). Dieser Entwurf
klärt, wo die Untergrenze lebt, was sie an Befehlen, Maschinencode und
Meldungen berührt, und in welcher Reihenfolge man es baut.

## 1. Was heute da ist

* `DhArray` (`value.rs`) hält `dims` (Anzahl je Dimension), `strides` und die
  Zellen (`Cells::Int`, `Cells::Float`, `Cells::Val`). `flat_index` prüft
  `0 <= idx < dims[k]` und meldet sonst
  `Index 10 ausserhalb [0..9] in Dimension 0`, bei genau eins zu viel mit dem
  Hinweis `Felder zaehlen ab 0`.
* `DIM a[n]` wird zu `DECLARE_ARRAY_NAME`/`DECLARE_ARRAY_LOCAL` mit der
  Zahl der Dimensionen als Argument; die Größen liegen auf dem Stapel
  (`stmt_dim` in `compiler.rs`). In einer Schleife legt `dim_frisch` das Feld
  neu an.
* Der Maschinencode liest und schreibt Felder von INTEGER/FLOAT direkt:
  `feld_beschreiben`/`feld_passt` entscheiden beim Bauen, beim Eintritt legt
  die VM je Feld Zeiger, Größen und Schritte in die Beschreibung
  (`jit.rs`). Eine Funktion liest ein globales Feld über `gfeld_lesen`, das
  dieselbe Prüfung `0 <= idx < dims` macht.
* `umstieg.rs` antwortet auf `LBOUND` mit „Felder beginnen immer bei 0" und
  auf `UBOUND` mit `LEN(feld) - 1`.

## 2. Wo die Untergrenze lebt

Die eigentliche Entscheidung. Zwei Wege:

### A. Nur im Compiler

Der Compiler weiß aus dem `DIM`, dass `tage` bei 1 anfängt, und zieht an
jeder Stelle `tage[i]` die 1 ab. Die Laufzeit bleibt unberührt, der
Maschinencode auch.

**Das trägt nicht:** ein Feld wandert. Übergeben an
`SUB summe(a AS ARRAY OF INTEGER)`, zurückgegeben aus einer FUNCTION,
gespeichert in einem Objektfeld, in einer MAP -- an all diesen Stellen weiß
der Compiler nicht mehr, woher das Feld kam. `summe` läse `a[1]` als zweites
Element, ohne Fehler. Genau die Art Fehler, die man erst am falschen Ergebnis
merkt.

### B. Am Feld selbst *(Empfehlung)*

`DhArray` bekommt ein Feld `unten: Vec<i64>` -- **leer heißt „alle 0"**, so
bleibt jedes bestehende Feld, wie es ist, und kostet keinen Platz.
`flat_index` zieht die Untergrenze ab, bevor es prüft:

```
idx' = idx - unten[k]      (nur wenn unten nicht leer ist)
0 <= idx' < dims[k]
```

Die Untergrenze reist damit überall hin, wohin das Feld reist: Parameter,
Rückgabe, Objektfeld, `b = a` (ein Verweis, dieselben Grenzen). Das ist
auch, was FreeBASIC und Visual Basic tun -- die Grenzen gehören zum Feld,
nicht zur Variable.

**Folge, die man sagen muss:** `tage = [1, 2, 3]` ersetzt das Feld durch ein
neues, und das beginnt bei 0. Die Variable hat keine Grenzen, nur das Feld
darin. Das ist dieselbe Regel wie heute bei der Größe (`DIM a[31]` und dann
`a = [1, 2]` hat danach zwei Plätze) und gehört als Stolperstein in die Doku.

## 3. Die Sprache

```
DIM tage[1 TO 31] AS INTEGER          ' 31 Plaetze, tage[1] .. tage[31]
DIM feld[-10 TO 10, -10 TO 10]        ' 21 x 21
DIM gemischt[5, 1 TO 3] AS STRING     ' erste Dimension ab 0, zweite ab 1
DIM von AS INTEGER : von = 1990
DIM jahre[von TO von + 35] AS FLOAT   ' Grenzen duerfen Ausdruecke sein
```

* `TO` steht innerhalb der eckigen Klammern sonst nirgends -- der Parser kann
  `a TO b` dort eindeutig lesen.
* `obergrenze < untergrenze - 1` ist ein Fehler beim Anlegen;
  `[1 TO 0]` ist ein leeres Feld (wie `DIM a[0]`).
* Gilt für globale und lokale Felder und für Felder als Klassenfeld.
  **Nicht** für Felder von `STRUCT … LAYOUT C` (dort ist die Lage die von C,
  und C zählt ab 0) -- dort bleibt es ein Fehler mit Hinweis.
* `DIM a(1 TO 10)` mit runden Klammern bleibt eine Umsteiger-Meldung, nennt
  aber jetzt die richtige Form `DIM a[1 TO 10]`.

### Neue Befehle

| Befehl | was es tut |
|---|---|
| `LBOUND` (feld [, dimension]) | kleinster Index; Dimension ab 1, ohne Angabe die erste |
| `UBOUND` (feld [, dimension]) | größter Index; bei einem leeren Feld `LBOUND - 1` |

Dimension **ab 1** wie in FreeBASIC und Visual Basic: genau die Leute, die
`LBOUND` schreiben, kennen es so. Die Umsteiger-Hinweise für `LBOUND`/`UBOUND`
in `umstieg.rs` fallen weg.

## 4. Was sich an bestehenden Befehlen ändert

Grundregel: **wer Zellen der Reihe nach nimmt, merkt nichts; wer eine
Stelle nennt oder liefert, zählt in den Indizes des Feldes.**

| Befehl | mit Untergrenze |
|---|---|
| `LEN`, `FOR EACH`, `SORT`, `REVERSE`, `ARRAY_SUM/AVG/MIN/MAX`, `JOIN$`, `ARRAY_FILL` | unverändert -- sie zählen bzw. gehen der Reihe nach |
| `ARRAY_PUSH`, `ARRAY_POP` | unverändert; die Obergrenze wächst bzw. schrumpft |
| `ARRAY_INSERT(a, idx, w)`, `ARRAY_REMOVE_AT(a, idx)` | `idx` in den Indizes des Feldes |
| `ARRAY_INDEXOF`, `ARRAY_FIND` | liefern den Index des Feldes; **nicht gefunden = `LBOUND - 1`** (wie .NETs `Array.IndexOf`). Bei einem Feld ab 0 bleibt das `-1` |
| `ARRAY_COPY`, `ARRAY_MAP` | behalten die Grenzen (dieselbe Form) |
| `ARRAY_FILTER`, `ARRAY_OF_TYPE`, Slicing `a[2:5]` | liefern eine neue Liste ab 0 -- ihre Plätze entsprechen keinen Plätzen des alten Feldes mehr |
| Slicing `a[von:bis]` | `von`/`bis` in den Indizes des Feldes |

**Der eine heikle Punkt** ist `ARRAY_INDEXOF`: Programme prüfen heute
`IF ARRAY_INDEXOF(a, x) >= 0`. Für ein Feld ab 0 ändert sich nichts. Für ein
Feld ab −10 wäre `-1` aber ein gültiger Index -- darum `LBOUND - 1`, und die
Doku sagt: vergleiche mit `LBOUND` des Feldes, nicht mit 0.

## 5. Meldungen

* `Index 0 ausserhalb [1..31] in Dimension 1` -- die Grenzen so, wie sie im
  `DIM` stehen.
* Der Hinweis „Felder zaehlen ab 0" steht nur noch bei Feldern ab 0.
* **Dimension in der Meldung ab 1** statt ab 0, damit sie zu `LBOUND` mit Dimension 2
  passt. Das ändert drei Erwartungen in den Prüfsammlungen und die
  englische Vorlage in `daten/meldungen.en.txt`. Bei 0 zu bleiben hieße,
  dass Meldung und `LBOUND` verschieden zählen (entschieden: ab 1).

## 6. Bytecode und `.dhc`

`DECLARE_ARRAY_NAME`/`_LOCAL` bekommen ein viertes, freiwilliges Argument
`mit_grenzen`. Ist es wahr, liegen auf dem Stapel je Dimension **zwei** Werte
(Untergrenze, Obergrenze) statt einem. Ohne das Argument bleibt alles, wie es
ist -- jede vorhandene `.dhc` und jedes exportierte Spiel läuft weiter.
`dim_frisch` gibt die Grenzen mit, sonst finge ein `DIM` in einer Schleife ab
der zweiten Runde bei 0 an.

## 7. Maschinencode

Zwei Stufen:

1. **Erst bleibt ein Feld mit Untergrenze der VM.** `feld_passt` und
   `feld_beschreiben` lehnen ein Feld mit nicht leerem `unten` ab,
   `gfeld_lesen` ebenso. Die Schleife läuft dann im Wertemodus oder in der VM
   -- langsamer, aber richtig, und es gibt keine zweite Rechnung, die
   abweichen könnte.
2. **Dann die Untergrenze in die Beschreibung.** Beim Eintritt legt die VM
   je Dimension zusätzlich `unten[k]` ab (bei leerem `unten` eine 0), und der
   erzeugte Code zieht sie vor der Bereichsprüfung ab -- ein Befehl je Index.
   Vorher und nachher `dhrt run bench_dhrt.dh` mit und ohne `DHRT_JIT=aus`
   messen (`tools/tempo/felder.dh`); kostet die Subtraktion bei Feldern ab 0
   messbar, bekommt `FeldInfo` einen Schalter „ab 0" und der Code für diese
   Felder bleibt, wie er ist.

Die Fälle in `jit.dhtest` laufen wie immer mit und ohne Maschinencode; der
Vergleich der beiden ist die Prüfung, dass Stufe 2 dasselbe rechnet.

## 8. Weitere Stellen

* **Debugger und IDE:** die Anzeige eines Feldes nennt die Indizes ab
  `LBOUND` (Variablenfenster, Werte am Zeilenende).
* **`dhrt --check`:** ein konstanter Index außerhalb fester Grenzen
  (`tage[0]` bei `DIM tage[1 TO 31]`) lässt sich beim Übersetzen erkennen --
  als Warnung, nicht als Fehler (die Grenzen können zur Laufzeit andere sein,
  wenn das Feld ersetzt wurde).
* **Hover und Vervollständigung** zeigen `DIM tage[1 TO 31] AS INTEGER` wie
  geschrieben; `LBOUND`/`UBOUND` kommen in `builtin_index.json` und in
  `docs/builtins-core.md`.
* **Doku:** `docs/sprache.md` (Abschnitt Arrays) samt `docs/en/sprache.md`,
  `docs/umstieg.md` (die Zeile zu `DIM a(10)`), `docs/stolpersteine.md` (die
  Grenzen gehören zum Feld, nicht zur Variable).

## 9. Prüfungen

Eine Sammlung `tests/pruef/feld_untergrenze.dhtest`, jeder Fall mit und ohne
Maschinencode:

* Anlegen, Lesen, Schreiben an beiden Rändern; ein Index davor und dahinter
  ist ein Fehler mit den Grenzen aus dem `DIM`.
* Zwei Dimensionen mit negativer Untergrenze; gemischt ab 0 und ab 1.
* `LBOUND`/`UBOUND` je Dimension, leeres Feld.
* Ein Feld als Parameter, als Rückgabe, als Objektfeld, über `b = a` -- die
  Grenzen bleiben. **Die Gegenprobe:** `a = [...]` beginnt wieder bei 0.
* `ARRAY_PUSH`/`POP`, `INSERT`/`REMOVE_AT`, `INDEXOF` mit
  `LBOUND - 1` bei negativer Untergrenze, Slicing.
* `DIM` mit Grenzen in einer Schleife (zweite Runde fängt wieder bei der
  Untergrenze an).
* Eine alte `.dhc` ohne das neue Argument läuft unverändert.
* Gegenproben: ohne die Subtraktion in `flat_index` bzw. im Maschinencode
  fallen die Fälle an den Rändern.

## 10. Reihenfolge und Aufwand

1. `DhArray::unten`, `flat_index`, Parser, Compiler, Bytecode, `LBOUND`/`UBOUND`,
   Meldungen -- der Maschinencode lehnt solche Felder ab (Stufe 1). Etwa ein
   Arbeitstag samt Prüfsammlung.
2. Die Befehle aus Abschnitt 4, Debugger-Anzeige, `--check`-Warnung, Doku.
3. Maschinencode Stufe 2, mit Messung.

Ein grober Blick in den Bestand zur Einordnung: Ausdrücke der Form
`[name - 1]` stehen 119-mal in 24 Dateien unter `examples/` und `ide/`. Nicht
jeder davon ist ein Ausgleich für die Zählung ab 0 -- oft ist es der
Nachbar in einer Schleife --, die Zahl sagt also nur, dass das Muster
vorkommt, nicht wie oft es fehlt.

## 11. Entschieden (2026-10-31)

Die drei Fragen, die der Entwurf offen ließ, sind wie vorgeschlagen
entschieden:

* **Slicing** liefert eine neue Liste ab 0, wie `ARRAY_FILTER` -- ihre
  Plätze entsprechen keinen Plätzen des alten Feldes mehr. `von` und `bis`
  zählen dabei in den Indizes des Feldes.
* **Die Dimension in Meldungen zählt ab 1**, passend zu `LBOUND` mit
  Dimension 2. Die drei betroffenen Erwartungen in den Prüfsammlungen und
  die englische Vorlage ziehen mit.
* **`ARRAY_INDEXOF` und `ARRAY_FIND` liefern bei „nicht gefunden"
  `LBOUND - 1`** -- bei einem Feld ab 0 also weiter `-1`. Die Doku sagt dazu:
  vergleiche mit `LBOUND` des Feldes, nicht mit 0.
