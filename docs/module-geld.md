# Modul `geld`

Ein Betrag als eigener Wert.

```basic
IMPORT "geld"
```

## Warum, wenn es `CENT` schon gibt

[`CENT`, `EURO$` und `ROUND_HALF_UP`](builtins-core.md#mit-geld-rechnen)
sind eine **Rechenweise**: man rechnet in ganzen Cent und muss daran denken.
Das Modul `geld` macht daraus einen **Typ**, und der denkt mit:

```basic
DIM preis AS GELD
preis = GELD_NEU("19,99")
PRINT preis * 3          ' 59,97 €
PRINT preis + 1.0        ' Fehler -- GELD und Kommazahl mischen sich nicht
```

Genau diese Fehlermeldung ist der Unterschied. In Cent gerechnet ist ein
Betrag ein `INTEGER` wie jeder andere; nichts hindert daran, versehentlich
einen Euro-Wert dazuzuaddieren. Ein GELD lässt sich nicht mit einer
gewöhnlichen Zahl vermischen — dafür muss man `GELD_NEU` oder `GELD_ZAHL`
schreiben und weiß dann, was man tut.

## Übersicht

| Funktion | Zweck |
|---|---|
| `GELD_NEU(betrag)` → GELD | aus Text (`"19,99"`, `"1.234,56"`) oder Zahl |
| `GELD_AUS_CENT(cent)` → GELD | aus ganzen Cent |
| `GELD_CENT(g)` → INTEGER | in ganze Cent, kaufmännisch gerundet |
| `GELD_ZAHL(g)` → FLOAT | bewusst herausgehen (Anzeige in Diagrammen o. Ä.) |
| `GELD_TEXT$(g [, symbol$])` → STRING | `19,99 €`, Symbol wählbar |
| `GELD_RUNDEN(g [, stellen])` → GELD | kaufmännisch, Vorgabe 2 Stellen |
| `GELD_TEILEN(g, anzahl)` → ARRAY OF GELD | aufteilen, **ohne einen Cent zu verlieren** |
| `GELD_ABS(g)` → GELD | Betrag ohne Vorzeichen |

Dazu die Operatoren: `+` `-` zwischen zwei Beträgen, `*` `/` mit einer Zahl,
`/` zwischen zwei Beträgen (ergibt ein Verhältnis als FLOAT), unäres `-`, und
die Vergleiche `=` `<>` `<` `>` `<=` `>=` — **exakt**, nicht ungefähr.

## Vier Nachkommastellen, nicht zwei

Innen ist ein GELD ein `INTEGER` in Hundertstel-Cent. Vier Stellen, weil
Steuersätze und Rabatte Zwischenergebnisse mit mehr Stellen erzeugen:

```basic
PRINT GELD_NEU("0,29") * 0.19          ' 0,0551 €
PRINT GELD_RUNDEN(GELD_NEU("0,29") * 0.19)   ' 0,06 €
```

`PRINT` zeigt die Stellen, die wirklich da sind. Ein noch nicht gerundetes
Zwischenergebnis gleich als „0,06 €" auszugeben wäre bequem und irreführend —
man sähe ihm nicht mehr an, dass es noch nicht gerundet ist. **Gerundet wird
also bewusst**, mit `GELD_RUNDEN`, und zwar dort, wo ein Betrag ausgewiesen
wird.

Der Wertebereich reicht bis rund ±922 Billionen Euro; darüber ist es ein
Fehler und keine still umlaufende Summe.

## Gerechnet wird ganzzahlig — auch mit einem Faktor

`betrag * 0.19` geht **nicht** über Fließkomma. Der Faktor wird in seine
Dezimalziffern zerlegt, und gerechnet wird `wert * 19 / 100` in ganzen
Zahlen. Der Umweg über `FLOAT` wäre genau die Ungenauigkeit, gegen die dieser
Typ antritt.

Dieselbe Regel wie bei `CENT`: gerundet wird **die Zahl, die dasteht** —
`GELD_NEU(19.99)` ergibt 19,99 €, nicht 19,9899…

## Aufteilen ohne Schwund

10,00 € durch drei sind nicht dreimal 3,33 € — ein Cent bliebe liegen.

```basic
DIM t AS ARRAY OF GELD
t = GELD_TEILEN(GELD_NEU("10,00"), 3)
PRINT t[0]      ' 3,34 €
PRINT t[1]      ' 3,33 €
PRINT t[2]      ' 3,33 €
```

Die ersten Teile bekommen den Rest. Aufgeteilt wird in **ganzen Cent** —
Bruchteile eines Cents kann niemand überweisen. Von Hand vergisst man diesen
Rest fast immer; das ist einer der Gründe, warum sich ein eigener Typ lohnt.

## Grenzen

* **Keine Währung im Wert.** Ein GELD ist ein Betrag, kein „19,99 EUR". Wer
  mit mehreren Währungen rechnet, hält sie auseinander — das Modul hilft
  dabei nicht.
* **Kein Literal.** `preis = 19.99` weist einer GELD-Variablen keine 19,99 €
  zu, sondern ist ein Fehler; es braucht `GELD_NEU`. Ein Literal (`19.99g`)
  hätte einen Eingriff in den Sprachkern gebraucht — die Abwägung steht in
  [Entwurf: Geld](entwurf-geldtyp.md).
* **`GELD * GELD` gibt es nicht.** Euro mal Euro wären Quadrat-Euro.
* **Nach außen wird es wieder eine Zahl.** `xlsx`, `json` und `chart` kennen
  kein GELD; dorthin geht es als `GELD_CENT` (ganze Zahl, verlustfrei) oder
  `GELD_ZAHL` (FLOAT, ab da wieder ungenau).

## In der nativen Runtime (dhrt)

`rust/drachenhauch_runtime/src/geld.rs`, ungated — ein Kassenprogramm soll
nicht das Netz mit eingebaut brauchen. Der Wert ist eine eigene
`Value`-Variante; Operatoren laufen über `module_op` in `vm.rs` (wie bei
`vec2`), die Zerlegung von Zahlen und Texten in Ziffern teilt sich das Modul
mit `CENT`/`ROUND_HALF_UP` in `builtins.rs` — es gibt nur **eine**
Rundungsregel im ganzen Haus.

Anders als die übrigen Modul-Typen ist GELD bei der Zuweisung **streng**:
`DIM x AS GELD : x = 5` ist ein Fehler. Ohne das wäre die Trennung, für die
es den Typ gibt, sofort wieder weg.

Beispiel: [examples/180_geld.dh](../examples/180_geld.dh).
