# Kapitel 2 — Variablen, Typen, f-Strings

In Kapitel 1 hat unser Programm einen festen Text ausgegeben — was reingetippt war, kam raus. Spiele funktionieren nicht so. Ein Spiel muss sich Dinge **merken**: deinen aktuellen Score, wieviel Leben du noch hast, ob das Spiel gerade läuft oder pausiert ist. Genau dafür gibt es **Variablen**.

In diesem Kapitel bauen wir noch keinen Bildschirm — den haben wir uns für Kapitel 5 aufgehoben. Wir lernen aber alle Bausteine kennen, die der Player später brauchen wird, um sich an seine Welt zu erinnern.

## Lernziele

Nach diesem Kapitel:

- kannst du Variablen mit `DIM ... AS ...` deklarieren
- unterscheidest du die vier Grundtypen: `INTEGER`, `FLOAT`, `STRING`, `BOOLEAN`
- rechnest du mit Variablen
- baust du lesbare Ausgaben mit f-Strings
- benutzt du `CONST` für Werte, die sich nie ändern
- nutzt du Compound-Assignment (`+=`, `-=`) für kürzere Schreibweise

## Was ist eine Variable?

Eine Variable ist ein **Name für einen Wert**. Du sagst dem Programm: „Ich nenne diesen Wert `score`. Wenn ich später `score` schreibe, meine ich diesen Wert. Und ich kann ihn ändern."

Klingt abstrakt — wird konkreter mit einem Beispiel:

```basic
DIM score AS INTEGER
score = 0

PRINT "Score: " + STR$(score)

score = 100
PRINT "Neuer Score: " + STR$(score)
```

Drei Dinge passieren hier:

1. `DIM score AS INTEGER` — wir **deklarieren** eine Variable namens `score`. Sie kann eine Ganzzahl (`INTEGER`) speichern.
2. `score = 0` — wir **setzen** den Wert auf `0`.
3. `score = 100` — wir **ändern** den Wert auf `100`. Der alte Wert ist weg.

Wenn du das Programm laufen lässt, kommt:

```
Score: 0
Neuer Score: 100
```

Das `STR$(score)` werden wir gleich aufdröseln. Jetzt erstmal die Typen.

## Die vier Grundtypen

GameBasic hat vier eingebaute Typen für Werte:

| Typ | Was er speichert | Beispiel |
|---|---|---|
| `INTEGER` | Ganzzahlen, positiv oder negativ | `score = 4200`, `lives = 3` |
| `FLOAT` | Kommazahlen | `zeit = 1.5`, `pi = 3.14159` |
| `STRING` | Text | `name = "Anna"`, `titel = "Star Pilot"` |
| `BOOLEAN` | nur `TRUE` oder `FALSE` | `aktiv = TRUE`, `tot = FALSE` |

Probier alle vier in einem Programm:

```basic
DIM punkte AS INTEGER
DIM zeit   AS FLOAT
DIM name   AS STRING
DIM aktiv  AS BOOLEAN

punkte = 100
zeit   = 12.5
name   = "Pilot Anna"
aktiv  = TRUE

PRINT "Punkte:", punkte
PRINT "Zeit:  ", zeit
PRINT "Name:  ", name
PRINT "Aktiv: ", aktiv
```

Output:

```
Punkte: 100
Zeit:   12.5
Name:   Pilot Anna
Aktiv:  TRUE
```

> **Beobachtung**: bei `PRINT` mit Komma getrennt, bekommt jede Variable Tab-Abstand zur vorherigen — keine Notwendigkeit für `STR$`. Das ist eine BASIC-Konvention seit Anbeginn der Zeit. Praktisch fürs Debuggen.

### Streng heißt streng

GameBasic ist eine **strikt typisierte** Sprache. Eine `INTEGER`-Variable kann nur Ganzzahlen aufnehmen, ein `STRING` nur Text. Wenn du folgendes versuchst:

```basic
DIM score AS INTEGER
score = "hallo"      ' FEHLER
```

bekommst du einen Fehler — schon bevor das Programm überhaupt läuft. Das fühlt sich am Anfang vielleicht streng an, ist aber dein Freund: **Tippfehler werden früh gefangen**, nicht erst dann, wenn dein Spiel mitten im Boss-Fight abstürzt.

> **Stolperfalle**: `DIM score AS INTEGER` und `score = 3.5` ist ebenfalls Fehler. Eine Kommazahl in eine Ganzzahl-Variable zu stopfen würde Information wegwerfen — GameBasic lässt das nicht zu. Wenn du beides mischen willst, deklarierst du `score AS FLOAT`.

## Mit Variablen rechnen

Die üblichen Rechenoperatoren funktionieren wie erwartet:

```basic
DIM score AS INTEGER
DIM bonus AS INTEGER

score = 100
bonus = 50

score = score + bonus       ' jetzt: 150
PRINT score
```

Operatoren im Überblick:

| Operator | Wirkung | Beispiel |
|---|---|---|
| `+` | Addition (oder Strings zusammenfügen) | `5 + 3` → `8` |
| `-` | Subtraktion | `10 - 4` → `6` |
| `*` | Multiplikation | `7 * 6` → `42` |
| `/` | Division | `10 / 4` → `2.5` |
| `MOD` | Rest bei Division | `10 MOD 3` → `1` |
| `^` | Potenz | `2 ^ 8` → `256` |

Achtung beim Division-Beispiel: `10 / 4` ergibt `2.5`, nicht `2` — die Division liefert eine `FLOAT`, sobald sie nicht aufgeht. Wenn du eine **Ganzzahl-Division** willst (Rest abschneiden), nimmst du `\` (Backslash):

```basic
PRINT 10 \ 4    ' = 2 (Rest weggeworfen)
PRINT 10 / 4    ' = 2.5
```

## Strings zusammensetzen

Texte verbindest du mit `+`. Genau wie Zahlen — nur dass aus zwei Strings ein längerer wird:

```basic
DIM vorname AS STRING
DIM nachname AS STRING
vorname  = "Anna"
nachname = "Sturm"

PRINT vorname + " " + nachname     ' "Anna Sturm"
```

Das Leerzeichen `" "` ist wichtig — sonst kämen die zwei Wörter direkt zusammen: `AnnaSturm`.

### Zahlen in Strings einbauen — der klassische Weg

Wenn du einen Score und Text mischen willst, brauchst du eine Übersetzung — eine Ganzzahl ist kein String, das `+` zwischen Text und Zahl wäre mehrdeutig. GameBasic löst das mit `STR$(...)`: gib eine Zahl rein, kriege einen String raus.

```basic
DIM score AS INTEGER
score = 1234

PRINT "Score: " + STR$(score)         ' "Score: 1234"
```

Das `$` am Ende ist BASIC-Tradition: Funktionen, die einen String zurückgeben, hießen schon immer `STR$`, `LEFT$`, `MID$`. Ein bisschen altmodisch, aber unverwechselbar.

> **Warum das Sternchen?** `STR$` heißt nicht "Stern", sondern "String-Funktion". Das `$` deutet auf den Rückgabe-Typ hin: hier kommt ein Text raus.

### Der Schmerz wächst

Was, wenn du mehrere Werte in einen Satz mischen willst? Combo-Multiplier neben Score, Highscore in Klammern dahinter?

```basic
DIM score AS INTEGER
DIM combo AS INTEGER
score = 1234
combo = 7

PRINT "Score: " + STR$(score) + " (Combo x" + STR$(combo) + ")"
```

Ergibt: `Score: 1234 (Combo x7)`. Aber schau dir das Ding an: eine Zeile ist 60 Zeichen lang, voller `+` und `STR$` und Anführungszeichen. Lesbarkeit: mäßig.

## f-Strings: viel besser

GameBasic hat eine modernere Schreibweise dafür: **f-Strings**. Ein `f` direkt vor den Anführungszeichen macht den String "formatiert" — du kannst Variablen direkt in geschweifte Klammern einbetten.

```basic
DIM score AS INTEGER
DIM combo AS INTEGER
score = 1234
combo = 7

PRINT f"Score: {score} (Combo x{combo})"
```

Output identisch wie oben — `Score: 1234 (Combo x7)` — aber zehnmal lesbarer. Du siehst auf einen Blick, wo welcher Wert hin soll.

f-Strings sind nicht auf Zahlen beschränkt: Strings, Booleans, alles geht drin:

```basic
DIM name AS STRING
DIM lives AS INTEGER
DIM aktiv AS BOOLEAN
name = "Pilot Anna"
lives = 3
aktiv = TRUE

PRINT f"Spieler {name}: {lives} Leben, aktiv: {aktiv}"
' "Spieler Pilot Anna: 3 Leben, aktiv: TRUE"
```

> **Tipp**: ab jetzt im Buch nutzen wir fast immer f-Strings. Die klassische `+ STR$(...)`-Form lernst du der Vollständigkeit halber, aber im Alltag schreibst du f-Strings.

> **Zwei geschweifte Klammern, wenn du eine echte Klammer im String willst**: `f"Hallo {{Welt}}"` druckt `Hallo {Welt}`. Selten gebraucht, aber gut zu wissen.

## CONST: Werte, die sich nie ändern

Manche Werte ändern sich während des Spiels nie: die Spielfeld-Breite, die maximale Anzahl Leben, der Punktewert pro Treffer. Statt sie als normale Variablen zu deklarieren (und damit das Risiko einzugehen, sie versehentlich zu überschreiben), nimmst du `CONST`:

```basic
CONST MAX_LIVES   AS INTEGER = 3
CONST POINTS_HIT  AS INTEGER = 100
CONST BONUS_LEVEL AS INTEGER = 1000
```

Im Programm benutzt du sie wie normale Variablen:

```basic
DIM lives AS INTEGER
lives = MAX_LIVES        ' = 3
```

Aber:

```basic
MAX_LIVES = 5            ' FEHLER - CONST darf nicht ueberschrieben werden
```

Konvention: Konstanten schreibt man in `GROSSBUCHSTABEN_MIT_UNTERSTRICH`. Das macht sie auf einen Blick erkennbar — du weißt, dass `MAX_LIVES` sich nie ändern wird, anders als `lives`.

## Compound-Assignment: kürzer schreiben

Die Konstruktion `score = score + 100` taucht in Spielen ständig auf. GameBasic hat dafür eine Kurzform: `+=`. Sie macht genau dasselbe, ist aber kompakter:

```basic
score += 100        ' identisch zu: score = score + 100
lives -= 1          ' identisch zu: lives = lives - 1
combo *= 2          ' identisch zu: combo = combo * 2
```

Drei dieser Operatoren musst du dir merken — sie tauchen in fast jedem Spiel-Code auf:

| Operator | Bedeutung |
|---|---|
| `+=` | dazu addieren |
| `-=` | abziehen |
| `*=` | multiplizieren |
| `/=` | dividieren |

Hier ist alles zusammen, eine Mini-Spielsimulation ohne Bild:

```basic
CONST MAX_LIVES   AS INTEGER = 3
CONST POINTS_HIT  AS INTEGER = 100
CONST BONUS_LEVEL AS INTEGER = 1000

DIM score AS INTEGER
DIM lives AS INTEGER
score = 0
lives = MAX_LIVES

' Drei Treffer
score += POINTS_HIT
score += POINTS_HIT
score += POINTS_HIT
PRINT f"Nach 3 Treffern: {score}"

' Level geschafft - Bonus
score += BONUS_LEVEL
PRINT f"Mit Level-Bonus:  {score}"

' Ein Leben verloren
lives -= 1
PRINT f"Leben uebrig:     {lives}"
```

Output:

```
Nach 3 Treffern: 300
Mit Level-Bonus:  1300
Leben uebrig:     2
```

## Star Pilot: der erste Spielzustand

Bringen wir alles zusammen. Hier ist ein Programm, das den **vollständigen Anfangs-Zustand unseres Spiels** in Variablen hält und ausgibt — noch ohne Bildschirm, aber mit allen Daten, die wir später zeichnen werden.

```basic
' Star Pilot - der erste Spielzustand. Noch ohne Bild, nur in Variablen.

DIM score        AS INTEGER
DIM lives        AS INTEGER
DIM highscore    AS INTEGER
DIM player_name  AS STRING

score        = 0
lives        = 3
highscore    = 4200
player_name  = "Anonymous"

PRINT "=== Star Pilot - Status ==="
PRINT f"Spieler:   {player_name}"
PRINT f"Score:     {score}"
PRINT f"Leben:     {lives}"
PRINT f"Highscore: {highscore}"

' Ein Treffer
score += 100
PRINT ""
PRINT "Nach erstem Treffer:"
PRINT f"Score:     {score}"
```

Wenn du das laufen lässt:

```
=== Star Pilot - Status ===
Spieler:   Anonymous
Score:     0
Leben:     3
Highscore: 4200

Nach erstem Treffer:
Score:     100
```

Diese Variablen werden uns durchs ganze Buch begleiten. In Kapitel 5 zeichnen wir den Score auf den Bildschirm; in Kapitel 16 speichern wir den Highscore in einer Datei, sodass er das Ende des Programms überlebt. Aber die Namen — `score`, `lives`, `highscore`, `player_name` — sind heute schon richtig.

## Übungen

**1. Eigener Spieler-Status.** Schreibe ein Programm, das deine Lieblings-Spielfigur beschreibt: Name, Lebenspunkte (INTEGER), Schaden pro Schuss (FLOAT), aktiv ja/nein (BOOLEAN). Gib alles mit f-Strings aus.

**2. Rechnen mit Score.** Lege eine Variable `score` an (Anfangswert 0). Simuliere fünf Treffer (jeweils +100 Punkte) und einen Bonus (+500). Gib nach jedem Schritt den aktuellen Score aus.

**3. Klassisch vs. f-String.** Schreibe eine Zeile auf zwei Arten: einmal mit `+` und `STR$`, einmal mit f-String. Welche findest du lesbarer? Welche ist kürzer?

**4. Stretch — Highscore-Vergleich.** Lege zwei Variablen `score` und `highscore` an. Berechne `differenz = highscore - score`. Gib eine Meldung aus: `Du brauchst noch <differenz> Punkte zum Highscore!`. Ändere die Werte und schau, ob deine Formel auch funktioniert wenn `score > highscore` (negative Differenz).

## Zusammenfassung

Du hast in diesem Kapitel:

- gelernt, wie Variablen mit `DIM` deklariert werden,
- die vier Grundtypen `INTEGER`, `FLOAT`, `STRING`, `BOOLEAN` kennengelernt,
- mit Variablen gerechnet,
- f-Strings als deutlich bequemere Alternative zu `+ STR$(...)` entdeckt,
- `CONST` für unveränderliche Werte gesehen,
- Compound-Assignment (`+=`, `-=`) als Kurzform,
- die ersten Variablen unseres Star-Pilot-Spiels (`score`, `lives`, `highscore`, `player_name`) angelegt.

Im **nächsten Kapitel** treffen wir Bedingungen: wir lassen das Programm Entscheidungen treffen — was passiert, wenn `score > highscore`? Wann ist das Spiel vorbei? Dafür brauchen wir `IF` und `SELECT CASE`.

## Code-Stand am Ende des Kapitels

- [`code/kap-02/01_variablen.gb`](code/kap-02/01_variablen.gb) — die vier Grundtypen demonstriert
- [`code/kap-02/02_score.gb`](code/kap-02/02_score.gb) — Star Pilots erster Spielzustand in Variablen
- [`code/kap-02/03_fstring.gb`](code/kap-02/03_fstring.gb) — klassisch vs. f-String, direkter Vergleich
- [`code/kap-02/04_const_compound.gb`](code/kap-02/04_const_compound.gb) — `CONST` und `+=` / `-=` in einem Mini-Spielfluss
