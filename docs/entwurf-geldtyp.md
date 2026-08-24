# Entwurf: Geld in Drachenhauch

> **Stand 24.08.2026: Weg A ist gebaut.** `CENT`, `EURO$` und
> `ROUND_HALF_UP` gibt es, und der Abschnitt „Mit Geld rechnen" steht in
> [builtins-core.md](builtins-core.md#mit-geld-rechnen). Der dritte Befehl
> heißt **nicht** `RUNDE_AUF` wie unten vorgeschlagen: das hätte jeder als
> „aufrunden" gelesen, also als `CEIL` — und das ist genau die falsche
> Bedeutung. `ROUND_HALF_UP` steht dafür neben `ROUND`/`FLOOR`/`CEIL`, wo
> es hingehört. Weg B und Weg C bleiben unentschieden.

*Untersuchung, keine Umsetzung.* Punkt 7 des [Allzweck-Audits](allzweck-audit-2.md)
brachte Module hervor, mit denen sich kaufmännische Software schreiben lässt —
Rechnung als PDF, Auswertung als Excel-Mappe, Versand per E-Mail. Damit steht
die Frage im Raum, die vorher niemand stellen musste: **womit wird gerechnet?**

Dieses Papier misst den heutigen Stand, benennt die Fallen, entwirft drei
Wege und empfiehlt einen. Die Entscheidung fällt jemand anders.

## Was heute passiert

Alle Zahlen unten sind gemessen, nicht behauptet — `dhrt run` auf dem Stand
vom 24.08.2026.

| Zeile | Ergebnis | |
|---|---|---|
| `PRINT 0.1 + 0.2` | `0.30000000000000004` | steht so **auf dem Bildschirm** |
| `PRINT 0.1 + 0.2 = 0.3` | `FALSE` | |
| zehnmal `s = s + 0.1` | `0.9999999999999999` | |
| `PRINT INT(19.99 * 100)` | **`1998`** | der Ratschlag „rechne in Cent" fällt selbst hinein |
| `PRINT INT(0.29 * 100)` | **`28`** | 0,29 € ist kein Sonderfall, sondern ein Preis |
| `PRINT ROUND(2.5)` | `2` | kaufmännisch wären es 3 |
| `PRINT ROUND(-2.5)` | `-2` | |
| `PRINT ROUND(2.675, 2)` | `2.67` | |
| `PRINT FORMAT$(1234567.891, "%.2f")` | `1234567.89` | kein Tausenderpunkt, kein Komma, kein € |
| `g = 9223372036854775807 : PRINT g + 1` | **Fehler** statt stillem Umlauf | |

Drei Befunde daraus, in der Reihenfolge ihrer Schwere:

**1. Der Umweg über Cent ist selbst eine Falle.** Die übliche Antwort auf
Fließkomma-Geld lautet „rechne in ganzen Cent". Der Schritt dorthin ist aber
`INT(preis * 100)` — und genau der verliert bei `19.99` und bei `0.29` einen
Cent, weil `19.99` als Fließkommazahl minimal *unter* 19,99 liegt. Wer den
Rat befolgt, ohne `ROUND` statt `INT` zu schreiben, baut sich den Fehler ein,
den er vermeiden wollte. **Das ist heute nirgends dokumentiert.**

**2. `ROUND` rundet zur geraden Zahl** (*round half to even*, in
`builtins.rs` als `round_half_even`, absichtlich wie Python). Kaufmännisch
gerundet wird in Deutschland von der Null weg: 2,5 → 3. Bei einer einzelnen
Rechnungszeile fällt der Unterschied nicht auf, bei tausend Positionen
verschiebt er die Summe. Auch das steht nicht in der Doku — dort heißt es nur
„auf N Nachkommastellen runden".

**3. Es gibt keine Geldanzeige.** `FORMAT$` kann keinen Tausendertrenner,
`NUMFMT$` ist die Kurzform für Idle-Spiele (`1.23M`). Jedes Programm baut
sich `"19,99 EUR"` selbst aus `\`, `MOD` und `FORMAT$` zusammen.

Was **gut** ist und in jedem Weg erhalten bleiben muss: `INTEGER` ist 64 Bit
und **meldet einen Überlauf als Fehler**, statt still umzulaufen. In Cent
gerechnet reicht das für ±92 Billiarden Cent — jede Buchhaltung dieser Welt.

## Weg A — nur Dokumentation und Helfer

Kein neuer Typ. Stattdessen:

* Ein Abschnitt „Mit Geld rechnen" in [builtins-core.md](builtins-core.md):
  in ganzen Cent rechnen, **`ROUND` statt `INT`** beim Umrechnen, und warum.
* `ROUND`s Rundungsregel dokumentieren.
* Drei neue Builtins (**gebaut am 24.08.2026**, Namen wie hier — bis auf
  den dritten, siehe Kopf):
  * `CENT` — Betrag → INTEGER, also gerundet mal hundert statt
    abgeschnitten, aber unter einem Namen, der sagt, was er tut.
  * `EURO$` — Cent (und wahlweise ein Währungskürzel) → `1.234.567,89 €`
    in deutscher Schreibweise.
  * `RUNDE_AUF` — kaufmännisches Runden, von der Null weg, wahlweise auf
    N Nachkommastellen.

**Kosten:** ein Nachmittag, drei Builtins, kein Eingriff in Übersetzer oder
VM. *(Nachgerechnet: hat gestimmt.)* **Nutzen:** die zwei gefährlichen Fallen sind zu, die Anzeige ist da.
**Bleibt offen:** wer trotzdem `FLOAT` nimmt — und das wird die Mehrheit
tun, weil `DIM preis AS FLOAT` das Naheliegende ist — bekommt weiterhin
`0.30000000000000004` und ROUND-Überraschungen. Der Rechner ist richtig, der
Anwender muss es nur wissen.

## Weg B — ein echter Typ `GELD`

Ein neuer Werttyp neben INTEGER/FLOAT/STRING/BOOLEAN.

```basic
DIM preis AS GELD
preis = GELD("19,99")          ' aus Text -- exakt
preis = preis * 3              ' 59,97 exakt, nicht 59.969999999999999
PRINT preis                    ' 19,99
```

**Innen:** ein `i64` in Hundertstel-Cent (vier Nachkommastellen). Vier, nicht
zwei, weil Steuersätze und Rabatte Zwischenergebnisse mit mehr Stellen
erzeugen (19 % von 0,29 € sind 0,0551 €); gerundet wird erst beim Ausweisen.
Wertebereich damit ±922 Billionen Euro. Überlauf ist ein Fehler, wie bei
INTEGER.

**Regeln:**

* `GELD + GELD`, `GELD - GELD` → GELD.
* `GELD * INTEGER`, `GELD * FLOAT` → GELD (Menge mal Preis, Prozentsatz).
* `GELD / GELD` → FLOAT (Verhältnis), `GELD / Zahl` → GELD.
* `GELD + FLOAT` ist ein **Fehler**. Genau diese stillschweigende Mischung
  soll der Typ verhindern; wer es will, schreibt es hin.
* Vergleiche sind exakt — `a = b` bedeutet, was dort steht.

**Der schwierige Punkt, und er ist nicht klein:** ein Literal.
`preis = 19.99` liest der Lexer als Fließkommazahl, und in dem Moment sind
die 19,99 schon nicht mehr exakt. Drei Auswege:

1. Ein Aufruf `GELD` mit dem Betrag als **Text** — ehrlich, aber
   umständlich, und ein Rechtschreibfehler fällt erst zur Laufzeit auf.
2. Suffix wie in C#: `preis = 19.99g`. Neues Lexer-Zeichen, aber der Wert
   entsteht aus dem **Text** und ist exakt.
3. Der Übersetzer kennt den Zieltyp und liest das Literal aus dem Quelltext
   nach. Am schönsten zu lesen (`preis = 19.99`), aber der Lexer müsste den
   ursprünglichen Text jedes Zahlliterals mitführen, und bei
   `f(19.99)` hinge die Bedeutung an der Signatur der Funktion.

**Was am Rand mit dranhängt** — der eigentliche Aufwand:

| Stelle | Was zu tun ist |
|---|---|
| Lexer/Parser | Typname, Literalform (siehe oben) |
| Übersetzer | statische Typherleitung (`statischer_typ`, `passt_nie`), Operatorregeln |
| VM | neue `Value`-Variante, `coerce`, Vergleiche, `PRINT`-Darstellung |
| `db` | SQLite kennt kein DECIMAL — als INTEGER in Hundertstel-Cent ablegen, und das dokumentieren |
| `xlsx` | **Excel rechnet selbst in `double`.** Eine Geldspalte kommt dort als Fließkommazahl an; exakt bleibt nur, was das Format anzeigt |
| `json` | JSON-Zahlen sind `double` → als Zeichenkette schreiben |
| `pdf`, `chart`, `gui` | Anzeige, unkritisch |
| Editor/LSP | Typ in `builtin_index.json`, Hervorhebung, VSCode-Grammatik |

Der `xlsx`-Punkt verdient es, ausgesprochen zu werden: **ein exakter Geldtyp
endet an der Tabellenkalkulation.** Das ist kein Grund gegen ihn — innerhalb
des Programms bleibt gerechnet, was gerechnet wird —, aber es macht die
Zusage kleiner, als sie klingt.

**Kosten:** kein Nachmittag. Sprachkern, VM, vier Module, Werkzeuge.

## Weg C — ein `MONEY`-Modul ohne Sprachänderung

Wie `vec2`: ein externer Typ mit überladenen Operatoren, per
`register_operators` eingehängt.

```basic
IMPORT "geld"
DIM p AS GELD
p = GELD_NEU("19,99")
PRINT GELD_TEXT$(p * 3)
```

**Kosten:** eine `.rs`-Datei, kein Eingriff in Lexer, Parser oder Übersetzer
— dieselbe Schublade wie `vec2` und `m3d`. **Preis:** `DIM p AS GELD`
braucht ein `IMPORT`, es gibt kein Literal (immer `GELD_NEU`), und der Typ
bleibt ein Fremdkörper, den die statische Prüfung nur als „externer Typ"
kennt. Genau so wie VEC2 heute — und das funktioniert dort seit Jahren gut.

## Empfehlung

**Weg A jetzt, Weg C als Zweites, Weg B nicht.**

Begründung:

* Die beiden **gefährlichen** Befunde (der Cent-Umweg und die Rundungsregel)
  sind keine Typfrage. Sie treffen heute jedes Programm, und Weg A schließt
  sie an einem Nachmittag. Alles andere kann warten, das nicht.
* Weg C gibt denen, die exakt rechnen wollen, einen Typ, der wie `vec2`
  aussieht und sich einfügt — ohne dass die Sprache eine dritte Zahlenart
  bekommt, die in jeder Typregel, jeder Fehlermeldung und jedem Modul
  mitgedacht werden muss.
* Weg B kostet einen Eingriff in den Sprachkern und liefert obendrauf
  gegenüber C vor allem eines: das Literal `19.99` ohne Aufruf. Das ist
  schön, aber es ist der teuerste Komfort in diesem Papier — und an der
  Grenze zu Excel und JSON endet die Exaktheit ohnehin.

BASIC-Dialekt bleibt in allen drei Wegen unangetastet; Weg A fügt nicht
einmal ein Schlüsselwort hinzu.

## Weg A, wie er gebaut wurde

```basic
CENT(19.99)                  ' 1999   -- rundet, statt abzuschneiden
CENT("19,99")                ' 1999   -- auch aus Text
CENT("1.234,56")             ' 123456 -- mit Tausendertrenner
EURO$(1999)                  ' 19,99 €
EURO$(123456789)             ' 1.234.567,89 €
EURO$(1999, "CHF")           ' 19,99 CHF
EURO$(1999, "")              ' 19,99
ROUND_HALF_UP(2.5)           ' 3      -- von der Null weg
ROUND_HALF_UP(-2.5)          ' -3
ROUND_HALF_UP(2.675, 2)      ' 2.68
```

Gerundet wird dabei **über die Dezimaldarstellung**, nicht über `x * 100`
— sonst hätte es dieselbe Krankheit wie `INT(19.99 * 100)`. Genauer: über
die kürzeste Dezimalzahl, die wieder auf denselben `FLOAT` zurückliest,
also über *die Zahl, die dasteht*. Deshalb wird aus `2.675` erwartungsgemäß
`2.68` und nicht `2.67`.

`EURO$` nimmt **nur INTEGER**. `EURO$(19.99)` ist ein Fehler, und die
Meldung nennt gleich die Lösung (*„aus 19.99 macht CENT(19.99) die Zahl
1999"*) — eine Anzeige, die aus einer Kommazahl stillschweigend „19,99 €"
macht, würde genau die Rechenweise verschleiern, um die es geht.

Vorführung: [examples/179_kasse.dh](../examples/179_kasse.dh). Der Bon
endet mit dem Fall, der zeigt, dass das kein Haarspalten ist: eine
FLOAT-Summe, die richtig aussieht, sich richtig druckt und sogar gleich
`72.71` ist — und aus der `INT(summe * 100)` trotzdem `7270` macht.
