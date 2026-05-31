# Kapitel 4 — Schleifen: WHILE, FOR, REPEAT

Wir kommen zum letzten Konsolen-Kapitel — und gleichzeitig zum wichtigsten. Ohne **Schleifen** kein Spiel. Wirklich. Ein Spiel ist im Kern eine ewig laufende Schleife: Eingabe lesen, Welt aktualisieren, Bild zeichnen, von vorne. Sechzig Mal pro Sekunde. Bis der Spieler aufhört.

In diesem Kapitel lernen wir alle Schleifenformen, die GameBasic anbietet — `FOR`, `WHILE`, `REPEAT/UNTIL` — plus die zwei Befehle, mit denen man eine Schleife abbrechen oder überspringen kann: `BREAK` und `CONTINUE`. Am Ende simulieren wir 1000 Schüsse von Star Pilot in einem Augenblick.

## Lernziele

Nach diesem Kapitel:

- schreibst du zählende Schleifen mit `FOR ... TO ... NEXT`, mit und ohne `STEP`
- nutzt du `WHILE ... WEND` für „solange-Schleifen"
- kennst du `REPEAT ... UNTIL` als Variante, die mindestens einmal läuft
- brichst Schleifen mit `BREAK` ab und überspringst Iterationen mit `CONTINUE`
- erzeugst Zufallszahlen mit `RND()` und `RANDOMIZE()`
- erklärst, warum jeder Game-Loop eine Schleife ist

## Wiederholung als Konzept

Du kennst Schleifen schon aus dem Alltag, ohne sie so zu nennen:

- „**Solange** der Wasserkocher noch nicht piept, warte." — `WHILE`
- „**Mache** zehn Liegestütze." — `FOR`
- „**Wiederhole** das Pasta-Wasser-prüfen, **bis** es kocht." — `REPEAT/UNTIL`

Programme machen das genauso, nur viel schneller.

## Schritt 1: FOR — die zählende Schleife

Wenn du **genau weißt, wie oft** du etwas wiederholen willst, ist `FOR` die richtige Wahl. Syntax:

```basic
FOR variable = start TO end
    ' ... was wiederholt werden soll ...
NEXT variable
```

Die Variable zählt von `start` bis `end` — **inklusiv**, beide Endpunkte werden mitgenommen. Beispiel: Quadrate von 1 bis 10:

```basic
DIM i AS INTEGER
FOR i = 1 TO 10
    PRINT f"{i}^2 = {i * i}"
NEXT i
```

Output:

```
1^2 = 1
2^2 = 4
3^2 = 9
...
10^2 = 100
```

Die Variable `i` muss vorher mit `DIM` deklariert sein — wie alle Variablen in GameBasic. Das `NEXT i` am Ende macht zwei Dinge: es markiert das Ende des Schleifen-Bodys und erhöht `i` um 1. Beim ersten Durchgang ist `i = 1`, beim zweiten `i = 2`, und so weiter, bis `i = 10`. Nach dem letzten Durchgang verlässt das Programm die Schleife.

> **Tipp**: Variablen in `FOR`-Schleifen heißen traditionell `i`, `j`, `k` (für nested loops). Nicht originell, aber sofort erkennbar — der Leser weiß: das ist ein Schleifen-Zähler.
>
> **Stolperfalle**: `step` ist ein Schlüsselwort in GameBasic. Du kannst eine Variable nicht `step` nennen — nimm `i`, `iter`, `counter` oder `tick`.

### STEP: andere Schrittweiten

Standardmäßig zählt `FOR` in **Einer-Schritten** hoch. Mit `STEP` änderst du das:

```basic
DIM j AS INTEGER
FOR j = 5 TO 1 STEP -1
    PRINT f"{j}..."
NEXT j
PRINT "Start!"
```

Output:

```
5...
4...
3...
2...
1...
Start!
```

`STEP -1` zählt rückwärts. `STEP 2` würde in Zweier-Schritten zählen, `STEP 10` in Zehnern. Auch Kommazahlen gehen, wenn deine Variable `FLOAT` ist:

```basic
DIM x AS FLOAT
FOR x = 0.0 TO 1.0 STEP 0.1
    PRINT x
NEXT x
```

(Vorsicht bei FLOAT-Schleifen: durch Rundungsfehler kann die letzte Iteration fehlen oder zuviel sein. In Zweifelsfällen lieber mit `INTEGER` zählen und am Ende durch eine Konstante teilen.)

## Schritt 2: WHILE — die solange-Schleife

`FOR` ist toll wenn du die Iterationszahl kennst. Aber oft weißt du das nicht — z.B. „solange der Spieler nicht das Fenster zumacht". Für solche Fälle ist `WHILE`:

```basic
WHILE bedingung
    ' ... was wiederholt werden soll ...
WEND
```

Die `bedingung` wird **vor jedem Durchlauf** geprüft. Ist sie `TRUE`, läuft der Body einmal und der Test passiert nochmal. Ist sie `FALSE`, geht's nach `WEND` weiter. Ein Beispiel mit `lives`:

```basic
DIM lives AS INTEGER
lives = 3

WHILE lives > 0
    PRINT f"Du hast {lives} Leben."
    lives -= 1
WEND

PRINT "Game Over!"
```

Output:

```
Du hast 3 Leben.
Du hast 2 Leben.
Du hast 1 Leben.
Game Over!
```

Wichtig: wenn die Bedingung schon **am Anfang** falsch ist, läuft der Body **kein einziges Mal**. Setz `lives = 0` und das Programm springt direkt zu `Game Over!`. Das ist meistens richtig, aber gelegentlich willst du genau das Gegenteil:

## Schritt 3: REPEAT/UNTIL — mindestens einmal

Was, wenn du **mindestens einen** Durchlauf garantieren willst, egal was? Beispiel: Würfel-bis-zur-6 — du willst mindestens einmal würfeln, sonst gibt's gar keinen Wurf.

`REPEAT ... UNTIL` macht das:

```basic
DIM wurf    AS INTEGER
DIM zaehler AS INTEGER
zaehler = 0

RANDOMIZE(42)

REPEAT
    wurf = INT(RND() * 6) + 1
    zaehler += 1
    PRINT f"Wurf {zaehler}: {wurf}"
UNTIL wurf = 6

PRINT f"Insgesamt {zaehler} Wuerfe."
```

Output:

```
Wurf 1: 4
Wurf 2: 1
Wurf 3: 2
Wurf 4: 2
Wurf 5: 5
Wurf 6: 5
Wurf 7: 6
Insgesamt 7 Wuerfe.
```

Drei Unterschiede zu `WHILE`:

1. **Reihenfolge**: bei `WHILE` steht die Bedingung **oben**, bei `REPEAT` **unten**.
2. **Garantie**: der Body läuft *immer* mindestens einmal — die Bedingung wird erst danach geprüft.
3. **Bedeutung**: `WHILE x > 0` heißt „solange x > 0". `UNTIL x = 0` heißt „bis x = 0" — die Schleife endet wenn die Bedingung **wahr** wird (das ist bei `WHILE` umgekehrt).

> **Daumenregel**: wenn die Bedingung „komm raus aus der Schleife wenn ..." natürlicher klingt, nimmst du `UNTIL`. Wenn „mache weiter solange ..." natürlicher klingt, nimmst du `WHILE`.

## Schritt 4: BREAK und CONTINUE

Manchmal willst du eine Schleife **vorzeitig verlassen** oder eine **Iteration überspringen**. Dafür gibt's `BREAK` und `CONTINUE`.

**`BREAK`** — aus der Schleife raus, sofort:

```basic
DIM i AS INTEGER
PRINT "Suche Welle 7:"
FOR i = 1 TO 100
    IF i = 7 THEN
        PRINT f"  Gefunden bei Welle {i}!"
        BREAK
    END IF
NEXT i
```

Sobald `i = 7`, wird gedruckt und die Schleife verlassen. Die restlichen 93 Iterationen laufen nicht mehr. Praktisch beim Suchen — wenn du gefunden hast was du wolltest, brauchst du nicht weiter.

**`CONTINUE`** — diese Iteration überspringen, mit der nächsten weitermachen:

```basic
DIM i AS INTEGER
PRINT "Zahlen 1..15 ohne Vielfache von 3:"
FOR i = 1 TO 15
    IF i MOD 3 = 0 THEN CONTINUE
    PRINT f"  {i}"
NEXT i
```

Output:

```
1
2
4
5
7
8
10
11
13
14
```

Sobald `i MOD 3 = 0` wahr ist (`i` ist Vielfaches von 3), springt `CONTINUE` direkt zu `NEXT i` — der `PRINT` wird übersprungen. Aber die Schleife läuft weiter, anders als bei `BREAK`.

> **Memo**: `BREAK` = aussteigen. `CONTINUE` = nächste Runde.

## Schritt 5: Zufallszahlen — RND und RANDOMIZE

Ein Spiel ohne Zufall ist langweilig. Die Aliens würden immer am gleichen Ort spawnen, das Wetter wäre vorhersagbar. GameBasic hat zwei Funktionen für Zufall:

| Funktion | Wirkung |
|---|---|
| `RND()` | liefert eine zufällige `FLOAT` zwischen 0.0 und 1.0 (exklusiv 1.0) |
| `RANDOMIZE(seed)` | setzt den Zufallsgenerator auf einen festen Startwert |

Aus `RND() * 100` wird eine Zahl zwischen 0 und 99.999. Wenn du eine Ganzzahl willst, nimm `INT(...)`:

```basic
DIM zufall AS INTEGER
zufall = INT(RND() * 100)        ' 0..99
PRINT zufall
```

Für einen Würfel (1 bis 6):

```basic
DIM wurf AS INTEGER
wurf = INT(RND() * 6) + 1        ' 1..6
```

### Reproduzierbarer Zufall — RANDOMIZE

`RANDOMIZE(seed)` setzt den Zufallsgenerator auf einen festen Startwert. Das klingt paradox („zufällig auf einem festen Wert"?), ist aber Gold wert: dasselbe `seed` ergibt **dieselbe Zufalls-Sequenz**. Praktisch fürs Debuggen — wenn dein Spiel bei einer bestimmten Welle abstürzt, kannst du mit demselben `seed` den genauen Verlauf reproduzieren.

```basic
RANDOMIZE(42)                       ' fester Seed
PRINT INT(RND() * 100)              ' immer dieselbe Zahl
PRINT INT(RND() * 100)              ' und dieselbe als zweite
```

Ohne `RANDOMIZE` (oder mit `RANDOMIZE()` ohne Argument) startet der Generator von der aktuellen Uhrzeit — dann sind die Zahlen wirklich zufällig.

## Schritt 6: 1000 Schüsse simulieren

Bringen wir alles zusammen — `FOR` plus `RND` plus ein bisschen Mathe. Wir simulieren 1000 Schüsse von Star Pilot, mit einer Trefferquote von 35%, und schauen wie lange das dauert.

```basic
CONST SCHUSS_ANZAHL  AS INTEGER = 1000
CONST TREFFER_CHANCE AS INTEGER = 35

DIM treffer    AS INTEGER
DIM fehlschuss AS INTEGER
DIM i          AS INTEGER

RANDOMIZE(1234)

DIM start_ms AS INTEGER
start_ms = MILLIS()

FOR i = 1 TO SCHUSS_ANZAHL
    IF INT(RND() * 100) < TREFFER_CHANCE THEN
        treffer += 1
    ELSE
        fehlschuss += 1
    END IF
NEXT i

DIM dauer AS INTEGER
dauer = MILLIS() - start_ms

PRINT f"Schuesse:     {SCHUSS_ANZAHL}"
PRINT f"Treffer:      {treffer}"
PRINT f"Fehlschuesse: {fehlschuss}"
PRINT f"Trefferquote: {INT(treffer * 100 / SCHUSS_ANZAHL)}%"
PRINT f"Dauer:        {dauer} ms"
```

Output (bei `seed = 1234`):

```
Schuesse:     1000
Treffer:      336
Fehlschuesse: 664
Trefferquote: 33%
Dauer:        0 ms
```

Knapp 33% Treffer — nahe an unserer Wunschquote von 35%, dem Zufall sei Dank. Und das Ganze in unter einer Millisekunde. So schnell sind moderne Computer. Du wirst dich daran gewöhnen.

> **`MILLIS()`** liefert die Anzahl der Millisekunden seit Programmstart — eine Standardmethode zum Messen, wie lange ein Code-Block braucht. Im Spiel verwenden wir es später (Kap 8) auch fürs Timing — z.B. „nicht öfter als alle 200 ms schießen".

## Vorgriff: der Game-Loop

Was du gerade über Schleifen gelernt hast, ist die Grundlage für **jeden** Game-Loop. Im nächsten Kapitel schreibst du den ersten:

```basic
WHILE NOT QUITREQUESTED()
    CLS()
    BOX(...)
    FLIP()
    SLEEP(16)
WEND
```

Eine `WHILE`-Schleife. So lange das Fenster offen ist (`NOT QUITREQUESTED()`), wird der Body wiederholt: Bildschirm löschen, malen, anzeigen, kurz warten. Das war's. Die ganze restliche Komplexität eines Spiels passiert *innerhalb* dieser Schleife — und wir bauen sie Stück für Stück die nächsten Kapitel auf.

## Übungen

**1. FizzBuzz.** Klassische Programmier-Übung: gib alle Zahlen von 1 bis 30 aus, aber statt Vielfache von 3 schreib `Fizz`, statt Vielfache von 5 `Buzz`, statt Vielfache von beiden `FizzBuzz`. Tipp: `i MOD 3 = 0` und `i MOD 5 = 0`.

**2. Würfle bis Doppelsechs.** Würfle zwei Würfel gleichzeitig mit `RND()`. Wiederhole, bis beide Würfel auf 6 fallen. Zähle die Würfe. Probier mit verschiedenen `RANDOMIZE()`-Seeds, was sich ändert.

**3. Schleifen-Wahl.** Schreibe folgendes dreimal — einmal mit `FOR`, einmal mit `WHILE`, einmal mit `REPEAT`: gib die Zahlen 1 bis 5 aus. Welche Form findest du am natürlichsten?

**4. Stretch — Spielsimulation.** Erweitere `05_simulation.gb`: pro Schuss kostet die Munition 5 Punkte vom Score, ein Treffer bringt 100 Punkte. Bei `score < 0` brich mit `BREAK` ab — Spiel verloren. Wieviele Schüsse hält der Spieler durch (mit verschiedenen `TREFFER_CHANCE`-Werten)?

## Zusammenfassung

Du hast in diesem Kapitel:

- die drei Schleifenformen kennengelernt: `FOR` (zählend), `WHILE` (solange), `REPEAT/UNTIL` (mindestens einmal),
- mit `STEP` die Schrittweite in `FOR`-Schleifen kontrolliert,
- `BREAK` (aussteigen) und `CONTINUE` (überspringen) eingesetzt,
- Zufallszahlen mit `RND()` und reproduzierbar mit `RANDOMIZE(seed)`,
- 1000 Schüsse Star Pilot in einer Millisekunde simuliert,
- den Game-Loop angekündigt, den wir im nächsten Kapitel bauen.

Damit ist die **Konsolen-Phase abgeschlossen**. Du beherrschst jetzt Variablen, Bedingungen und Schleifen — die drei Säulen jeder imperativen Programmiersprache. Im **nächsten Kapitel** öffnen wir das erste Spielfenster und zeichnen den ersten Player. Das hast du dir verdient.

## Code-Stand am Ende des Kapitels

- [`code/kap-04/01_for.gb`](code/kap-04/01_for.gb) — Quadrate und Countdown mit `FOR`/`STEP`
- [`code/kap-04/02_while.gb`](code/kap-04/02_while.gb) — Lebens-Countdown mit `WHILE`
- [`code/kap-04/03_repeat.gb`](code/kap-04/03_repeat.gb) — würfeln bis zur 6 mit `REPEAT/UNTIL`
- [`code/kap-04/04_break_continue.gb`](code/kap-04/04_break_continue.gb) — `BREAK` und `CONTINUE` in Aktion
- [`code/kap-04/05_simulation.gb`](code/kap-04/05_simulation.gb) — Star Pilot 1000-Schuss-Simulation mit Zufall und Performance-Messung
