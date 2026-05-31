# Kapitel 3 — Entscheidungen treffen: IF, ELSE, SELECT CASE

In Kapitel 2 hat unser Programm Werte gemerkt — den Score, die Leben, den Highscore. Aber merken alleine reicht nicht: ein Spiel muss auch **reagieren**. Wenn `lives` auf 0 fällt, soll „Game Over" kommen. Wenn der Score den Highscore knackt, soll's eine Trompete geben. Wenn der Spieler in den ersten drei Wellen ist, kommen wenige Gegner; ab Welle zehn fliegen die Aliens dichter.

Diese Reaktionsfähigkeit baut auf zwei Konstrukten: `IF` und `SELECT CASE`.

## Lernziele

Nach diesem Kapitel:

- formulierst du Bedingungen mit `IF / ELSEIF / ELSE / END IF`
- kennst du die Vergleichsoperatoren `=`, `<>`, `<`, `>`, `<=`, `>=`
- kombinierst du Bedingungen mit `AND`, `OR`, `NOT`
- nutzt du `SELECT CASE` für mehrweg-Verzweigungen
- entscheidest du selbständig, wann `IF` besser ist und wann `SELECT CASE`

## Schritt 1: Die einfachste Bedingung

Eine Bedingung ist ein Test, der entweder `TRUE` oder `FALSE` ergibt. Genauso wie der `BOOLEAN`-Typ aus Kap 2 — kein Zufall.

```basic
DIM score AS INTEGER
score = 1500

IF score > 1000 THEN
    PRINT "Stark!"
END IF
```

`IF score > 1000 THEN` heißt: „Wenn der Score größer als 1000 ist, dann ...". Was zwischen `THEN` und `END IF` steht, läuft **nur wenn die Bedingung wahr ist**. Bei `score = 1500` ist sie wahr, also kommt `Stark!`. Setz `score = 200` und du bekommst — nichts. Das Programm überspringt den Block.

> **Stolperfalle**: kein Doppelpunkt, kein `:`. Nach `THEN` steht entweder ein Statement direkt (Einzeiler) oder eine neue Zeile mit `END IF` am Ende des Blocks. C-Programmierer und Python-Leute irritiert die Schreibweise erstmal — aber sie ist die BASIC-Norm seit Jahrzehnten.

### Einzeiler-Form

Wenn du nur eine einzelne Anweisung machen willst, gibt's eine kompakte Form:

```basic
IF score > 1000 THEN PRINT "Stark!"
```

Kein `END IF` nötig — die Anweisung steht direkt hinter `THEN`. Praktisch für Mini-Checks, aber bei mehreren Aktionen oder `ELSE` brauchst du die Block-Form.

## Schritt 2: ELSE — der „sonst"-Fall

Was, wenn der Score *nicht* über 1000 ist? Mit `ELSE` kannst du das angeben:

```basic
IF score > 1000 THEN
    PRINT "Stark!"
ELSE
    PRINT "Da geht noch was."
END IF
```

Eines von beiden läuft immer — nie beide, nie keines. Genau das was du erwartest.

## Schritt 3: Mehrere Stufen mit ELSEIF

Was wenn du **drei oder mehr Stufen** willst? Anfänger, Solide, Profi, Ass je nach Score? Dann reichen `IF/ELSE` nicht — du brauchst `ELSEIF` (zusammengeschrieben, ein Wort).

```basic
DIM score AS INTEGER
score = 1500

IF score < 100 THEN
    PRINT "Rang: Anfaenger"
ELSEIF score < 1000 THEN
    PRINT "Rang: Solide"
ELSEIF score < 5000 THEN
    PRINT "Rang: Profi"
ELSE
    PRINT "Rang: Ass"
END IF
```

Lauf mit `score = 1500` — Output: `Rang: Profi`. GameBasic geht die Bedingungen **von oben nach unten** durch und nimmt die **erste**, die wahr ist:

- `1500 < 100`? Nein.
- `1500 < 1000`? Nein.
- `1500 < 5000`? **Ja** → „Profi" wird gedruckt, Rest übersprungen.

Das `ELSE` am Ende ist optional, aber meistens sinnvoll: es fängt alles ab, was keine der vorherigen Bedingungen erfüllt.

## Vergleichsoperatoren im Überblick

| Operator | Bedeutung |
|---|---|
| `=` | gleich |
| `<>` | ungleich |
| `<` | kleiner als |
| `>` | größer als |
| `<=` | kleiner-gleich |
| `>=` | größer-gleich |

> **Achtung beim `=`**: in `IF lives = 0 THEN ...` ist `=` ein Vergleich („ist `lives` gleich 0?"). In `lives = 3` ist `=` eine Zuweisung. Welcher Sinn gemeint ist, erkennt GameBasic am Kontext — der Operator ist derselbe. C-/Python-Leute sind hier doppeltes `==` gewohnt; in BASIC gibt's das nicht.
>
> **`<>` für ungleich**: andere Sprachen schreiben `!=` oder `≠`; BASIC nutzt seit jeher `<>`. Sieht aus wie „kleiner oder größer" — was logisch dasselbe ist wie „ungleich".

## Schritt 4: Bedingungen kombinieren

Manchmal reicht eine Bedingung nicht. „Highscore-Modus" soll z.B. nur dann aktiviert werden, wenn der Score hoch ist **und** der Spieler noch lebt:

```basic
IF score >= 4000 AND lives > 0 THEN
    PRINT "Highscore-Modus aktiv!"
END IF
```

Drei Verknüpfungen brauchst du:

| Operator | Wann TRUE? |
|---|---|
| `AND` | wenn **beide** Bedingungen wahr sind |
| `OR` | wenn **mindestens eine** der beiden wahr ist |
| `NOT` | kehrt eine Bedingung um (`TRUE` → `FALSE`) |

Ein Beispiel mit allen dreien:

```basic
DIM score      AS INTEGER
DIM lives      AS INTEGER
DIM highscore  AS INTEGER

score     = 4500
lives     = 2
highscore = 5000

IF score >= 4000 AND lives > 0 THEN
    PRINT "Highscore-Modus aktiv!"
END IF

IF lives = 0 THEN
    PRINT "Game Over"
END IF

IF NOT (score >= highscore) THEN
    DIM diff AS INTEGER
    diff = highscore - score
    PRINT f"Noch {diff} Punkte zum Highscore!"
END IF
```

Output:

```
Highscore-Modus aktiv!
Noch 500 Punkte zum Highscore!
```

> **Klammern setzen lohnt sich** sobald du mehrere `AND`/`OR` mischst:
>
> ```basic
> IF (lives > 0 AND score > 1000) OR is_demo THEN ...
> ```
>
> Auch wenn die Vorrang-Regeln klar definiert sind — Klammern machen die Absicht für *dich in sechs Monaten* sichtbar. Schreib sie hin.

## Schritt 5: SELECT CASE — wenn aus IF eine Kaskade wird

Eine Kette aus drei oder vier `ELSEIF` kann unübersichtlich werden, vor allem wenn alle dieselbe Variable testen. Schau nochmal die Score-Bewertung von oben an — `score < 100`, `score < 1000`, `score < 5000`. Die Variable `score` taucht viermal auf, jedes Mal mit demselben Aufbau. Das geht eleganter mit `SELECT CASE`:

```basic
SELECT CASE score
    CASE IS < 100
        PRINT "Rang: Anfaenger"
    CASE IS < 1000
        PRINT "Rang: Solide"
    CASE IS < 5000
        PRINT "Rang: Profi"
    CASE ELSE
        PRINT "Rang: Ass"
END SELECT
```

Du nennst die Variable einmal nach `SELECT CASE`, jedes `CASE` testet einen Wert oder Bereich. `CASE ELSE` ist das Pendant zu `ELSE` — der Fang-alles-Fall.

`SELECT CASE` ist **nicht stärker als IF/ELSEIF** — alles, was hier geht, ginge auch mit `IF`. Es ist *nur* eine kompaktere Schreibweise für den häufigen Fall „eine Variable wird gegen mehrere Werte getestet". Die Unterscheidung bleibt: `IF` nimmst du wenn **verschiedene Variablen** geprüft werden müssen. `SELECT CASE` ist die Wahl wenn **dieselbe Variable** in mehrere Schubladen einsortiert wird.

### Drei Match-Formen pro CASE

`SELECT CASE` kann mehr als nur einzelne Werte. Drei Formen:

**Einzelner Wert**:

```basic
CASE 1
    PRINT "Welle 1: Aufwaermen"
```

**Liste von Werten** (Komma-getrennt):

```basic
CASE 2, 3
    PRINT "Welle 2-3: Schwung kommt rein"
```

**Bereich** mit `TO` (inklusiv):

```basic
CASE 4 TO 9
    PRINT "Welle 4-9: jetzt wird's ernst"
```

**Vergleich** mit `IS`:

```basic
CASE IS >= 10
    PRINT "Welle 10+: Endlos-Modus"
```

Du kannst alle Formen mischen:

```basic
CASE 1, 5 TO 8, IS = 13
    PRINT "Sonderwelle"
```

Das matcht: Welle 1, Wellen 5–8, oder Welle 13.

### Das Star-Pilot-Beispiel

Hier ein Programm, das alle vier Formen einsetzt — ein typischer „bewerte-den-Spielzustand"-Block:

```basic
DIM score AS INTEGER
DIM lives AS INTEGER
DIM wave  AS INTEGER

score = 2400
lives = 2
wave  = 3

SELECT CASE wave
    CASE 1
        PRINT "Welle 1: Aufwaermen"
    CASE 2, 3
        PRINT "Welle 2-3: Schwung kommt rein"
    CASE 4 TO 9
        PRINT "Welle 4-9: jetzt wird's ernst"
    CASE IS >= 10
        PRINT "Welle 10+: Endlos-Modus"
END SELECT
```

Output bei `wave = 3`:

```
Welle 2-3: Schwung kommt rein
```

> **Garantie**: bei einem `SELECT CASE` greift **immer nur ein** `CASE` — der erste, der passt. Die anderen werden übersprungen, auch wenn sie ebenfalls matchen würden. Wer mit `switch` aus C kommt: in BASIC gibt's **kein** Fall-Through.
>
> **Garantie 2**: der Ausdruck nach `SELECT CASE` (hier `wave`) wird genau **einmal** evaluiert, auch wenn er ein teurer Funktionsaufruf wäre. Das ist anders als bei einer ELSEIF-Kette mit `f(x)`-Aufrufen, wo der Aufruf jedes Mal neu passiert.

## Wann `IF`, wann `SELECT CASE`?

Eine Daumenregel:

- **`IF/ELSEIF`**: wenn die Bedingungen *unterschiedliche* Variablen oder verschiedene Aspekte derselben Variable prüfen. Beispiel: `IF lives = 0 ... ELSEIF score < 0 ...`.
- **`SELECT CASE`**: wenn du **eine** Variable in mehrere Schubladen einsortierst. Beispiel: `SELECT CASE wave`.
- **Reine Einzeltests** (eine Bedingung, kein `ELSE`): immer `IF`. Da hat `SELECT CASE` keinen Vorteil.

Im Zweifel: `IF` ist nie falsch, `SELECT CASE` ist meist klarer wenn die Bedingungen alle dieselbe Form haben.

## Übungen

**1. Lebensanzeige.** Schreibe ein Programm mit einer Variable `lives AS INTEGER`. Mit `IF/ELSEIF/ELSE`: gib aus `0` → "Game Over", `1` → "Letztes Leben!", `2` oder `3` → "Vorsicht!", alles andere → "Sicher". Probier verschiedene Werte aus.

**2. Gleicher Code mit SELECT CASE.** Schreibe Übung 1 nochmal — diesmal mit `SELECT CASE`. Welche Form findest du besser lesbar?

**3. Kombinierte Bedingungen.** Schreibe ein Programm, das anhand von `score`, `lives` und `combo` ausgibt:
- "MEGA-COMBO!" wenn `combo >= 5 AND lives > 0`
- "Rette dich!" wenn `lives = 1 AND score > 1000` (du willst den Lauf nicht verlieren)
- "Game Over" wenn `lives = 0`

Was passiert wenn alle drei Bedingungen gleichzeitig wahr sind? Welches `IF` greift?

**4. Stretch — Schwierigkeitsstufen.** Schreibe ein Programm, das anhand `wave` (Wellen-Nummer) eine `enemy_count`-Variable setzt: Welle 1–3 → 5 Gegner, 4–9 → 10 Gegner, 10–19 → 15 Gegner, ab 20 → 20 Gegner. Nutze `SELECT CASE` mit Bereichen. Drucke am Ende `Welle X: Y Gegner`.

## Zusammenfassung

Du hast in diesem Kapitel:

- gelernt, mit `IF/ELSE/ELSEIF/END IF` Entscheidungen zu treffen,
- die Vergleichsoperatoren `=`, `<>`, `<`, `>`, `<=`, `>=` kennengelernt,
- Bedingungen mit `AND`, `OR`, `NOT` kombiniert,
- `SELECT CASE` als kompakte Alternative für Mehrweg-Verzweigungen entdeckt,
- vier Match-Formen pro `CASE` gesehen: einzelner Wert, Liste, Bereich, Vergleich,
- eine Daumenregel für die Wahl zwischen `IF` und `SELECT CASE` mitgenommen.

Im **nächsten Kapitel** kommen Schleifen — und damit die wichtigste Konstruktion in jedem Spiel: der Game-Loop, der Frame für Frame wiederholt wird, bis der Spieler aufhört.

## Code-Stand am Ende des Kapitels

- [`code/kap-03/01_if.gb`](code/kap-03/01_if.gb) — Score-Rang mit `IF/ELSEIF`
- [`code/kap-03/02_select.gb`](code/kap-03/02_select.gb) — gleiche Logik mit `SELECT CASE`
- [`code/kap-03/03_kombiniert.gb`](code/kap-03/03_kombiniert.gb) — `AND`, `OR`, `NOT` in Aktion
- [`code/kap-03/04_game_state.gb`](code/kap-03/04_game_state.gb) — Star Pilots Spielzustand mit allen vier `CASE`-Formen
