# Kapitel 7 — Funktionen: Code aufräumen mit SUB und FUNCTION

In Kapitel 6 hatten wir am Ende ein Programm mit etwa 30 Zeilen — ein Game-Loop, in dem **alles** drinsteckt: Konstanten oben, Eingabe-Verarbeitung, Position-Clamping, Zeichnen, alles in derselben `WHILE`-Schleife. Jetzt stell dir vor: in Kapitel 8 kommen Bullets dazu, in Kapitel 10 Enemies, in Kapitel 11 Kollisionen. Wenn wir alles in dieselbe Schleife packen, wird das schnell unlesbar.

Die Lösung: **Funktionen**. Wir teilen unsere Schleife in benannte Stücke auf — `Setup()`, `UpdatePlayer()`, `DrawPlayer()` — und der Hauptcode bleibt überschaubar.

## Lernziele

Nach diesem Kapitel:

- erklärst du den Unterschied zwischen `SUB` und `FUNCTION`
- schreibst du Funktionen mit typisierten Parametern
- nutzt du Default-Werte, um Aufrufe zu vereinfachen
- gibst du mit `BYREF` mehrere Werte aus einer Funktion zurück
- hast du den Spielcode in `Setup()` / `UpdatePlayer()` / `DrawPlayer()` aufgeteilt

## Warum Funktionen?

Ein gutes Programm liest sich von oben nach unten wie ein Inhaltsverzeichnis:

```
1. Setup
2. Update Player
3. Update Bullets
4. Update Enemies
5. Detect Collisions
6. Draw everything
```

Statt einem 200-Zeilen-Block, in dem du jedes Mal danach suchen musst, was wo passiert, hast du benannte Stücke. Die Hauptschleife wird zur Inhaltsangabe:

```basic
WHILE NOT QUITREQUESTED()
    UpdatePlayer()
    UpdateBullets()
    UpdateEnemies()
    DetectCollisions()

    CLS(BG_COLOR)
    DrawAll()
    FLIP()
    SLEEP(16)
WEND
```

Das ist das Ziel. In diesem Kapitel machen wir den ersten Schritt dahin.

## SUB: macht etwas

Eine `SUB` ist eine Funktion **ohne Rückgabewert**. Sie macht etwas (zeichnet, setzt eine Variable, druckt Text), liefert aber keinen Wert zurück.

```basic
SUB Greet(name AS STRING)
    PRINT "Hallo, " + name + "!"
END SUB

Greet("Pilot")        ' druckt: Hallo, Pilot!
Greet("Captain")      ' druckt: Hallo, Captain!
```

Der Aufbau:

- `SUB Name(parameter AS Typ, ...)` — Name und Parameterliste
- Body — was getan wird
- `END SUB` — Schluss

Parameter brauchen einen Typ. Du sagst nicht nur `name`, sondern `name AS STRING`. Damit kann GameBasic dich vor Fehlern schützen: `Greet(42)` würde nicht durchgehen, weil `42` kein STRING ist.

## FUNCTION: berechnet etwas

Eine `FUNCTION` ist wie `SUB`, aber **gibt einen Wert zurück**. Wir brauchen das ständig — `Add(3, 4)` soll `7` ergeben, `Sqrt(2)` soll `1.414` ergeben.

```basic
FUNCTION Add(a AS INTEGER, b AS INTEGER) AS INTEGER
    RETURN a + b
END FUNCTION

DIM summe AS INTEGER
summe = Add(3, 4)             ' summe = 7
PRINT f"Direkt: {Add(10, 20)}" ' druckt: Direkt: 30
```

Vier Unterschiede zu `SUB`:

1. Schlüsselwort `FUNCTION` statt `SUB`.
2. Nach der Parameterliste steht `AS Rückgabetyp` — hier `AS INTEGER`.
3. Im Body brauchst du `RETURN wert`.
4. Schluss heißt `END FUNCTION`.

Der Aufruf sieht aus wie ein Wert: du kannst ihn einer Variable zuweisen, in einen f-String einbetten, oder direkt weiterverwenden (`Add(Add(1, 2), 3)`).

> **Wann SUB, wann FUNCTION?** Faustregel: wenn der Aufrufer den Rückgabewert *braucht*, ist es `FUNCTION`. Wenn er nur die Wirkung will, ist es `SUB`. `DrawPlayer()` ist `SUB` (zeichnet, fertig). `Clamp(wert, lo, hi)` ist `FUNCTION` (liefert geklemmten Wert zurück).

## Eine eigene Clamp-Funktion

In Kap 6 haben wir den Player auf den Bildschirm-Bereich begrenzt:

```basic
IF player_x < 0 THEN
    player_x = 0
END IF
IF player_x > WIDTH - PLAYER_W THEN
    player_x = WIDTH - PLAYER_W
END IF
```

Vier Zeilen, zwei `IF`s, klar lesbar — aber wir werden das in zukünftigen Kapiteln *häufig* brauchen (Enemies an den Rand klemmen, Bullets auf gültigem Y-Bereich halten). Jedes Mal vier Zeilen schreiben? Zu viel.

`Clamp` als Funktion:

```basic
FUNCTION Clamp(wert AS INTEGER, lo AS INTEGER, hi AS INTEGER) AS INTEGER
    IF wert < lo THEN RETURN lo
    IF wert > hi THEN RETURN hi
    RETURN wert
END FUNCTION
```

Vier Zeilen Definition — aber dafür ist der Aufruf eine Zeile:

```basic
player_x = Clamp(player_x, 0, WIDTH - PLAYER_W)
```

Das ist die Schönheit von Funktionen: Du **investierst einmal**, sparst **viele Male** im Aufruf.

## Default-Werte

Manche Parameter haben oft denselben Wert. Statt sie jedes Mal anzugeben, kannst du einen **Default** setzen:

```basic
SUB Greet(name AS STRING, prefix AS STRING = "Hallo")
    PRINT prefix + ", " + name + "!"
END SUB

Greet("Pilot")                ' "Hallo, Pilot!"   (Default greift)
Greet("Captain", "Hi")        ' "Hi, Captain!"    (explizit)
```

Wenn der Aufrufer `prefix` nicht angibt, nimmt GameBasic den Default `"Hallo"`. Wenn er ihn angibt, gewinnt das Argument.

> **Regel**: Parameter mit Default müssen **am Ende** der Parameterliste stehen. Ein Pflicht-Parameter nach einem Default-Parameter ist ein Compile-Fehler. Sonst wäre nicht eindeutig, welches Argument an welche Position gehört.

### Defaults dürfen sich auf vorhergehende Parameter beziehen

Eine clevere Variante: der Default eines Parameters darf den Wert eines **früheren** Parameters benutzen:

```basic
SUB BoxOf(width AS INTEGER, height AS INTEGER = width)
    PRINT f"Box {width}x{height}"
END SUB

BoxOf(50)         ' "Box 50x50"   (Quadrat, weil height = width)
BoxOf(50, 30)     ' "Box 50x30"   (Rechteck)
```

Das ist eleganter als zwei separate Funktionen `Square(seite)` und `Rect(width, height)`.

## BYREF: mehrere Werte zurückgeben

Ein Problem, mit dem du irgendwann zu tun bekommst: eine Funktion soll **mehrere** Werte zurückgeben. Zum Beispiel: Quotient *und* Rest aus einer Division. `RETURN` kann nur einen Wert liefern. Was tun?

GameBasic löst das mit `BYREF`. Normalerweise bekommt eine Funktion eine **Kopie** des Arguments — Änderungen wirken sich nicht aufs Original aus. Mit `BYREF` ist das anders: die Funktion bekommt direkten Zugriff auf die Variable des Aufrufers.

Das klassische Beispiel ist `Swap`:

```basic
SUB Swap(BYREF a AS INTEGER, BYREF b AS INTEGER)
    DIM tmp AS INTEGER
    tmp = a
    a = b
    b = tmp
END SUB

DIM x AS INTEGER
DIM y AS INTEGER
x = 1
y = 2
Swap(x, y)
PRINT f"x={x}, y={y}"     ' "x=2, y=1"  (vertauscht!)
```

Innerhalb von `Swap` heißt `a` eigentlich `x` (von außen) — was Swap mit `a` macht, passiert direkt mit `x`. Ohne `BYREF` würde die Funktion gar nichts ändern; sie hätte nur eigene Kopien getauscht.

### DivMod: Funktion + BYREF gemischt

`BYREF` darf auch in einer `FUNCTION` vorkommen — dann hast du sowohl `RETURN`-Wert als auch BYREF-Output. Klassisches Beispiel: `DivMod` gibt Quotient via `RETURN` und Rest via `BYREF`:

```basic
FUNCTION DivMod(a AS INTEGER, b AS INTEGER, BYREF rest AS INTEGER) AS INTEGER
    rest = a MOD b
    RETURN a \ b
END FUNCTION

DIM r AS INTEGER
DIM q AS INTEGER
q = DivMod(17, 5, r)
PRINT f"17 / 5 = {q} Rest {r}"      ' "17 / 5 = 3 Rest 2"
```

> **Stolperfalle**: `BYREF` braucht einen **echten Variablen-Namen** als Argument. `Swap(1, 2)` ist Fehler — `1` ist keine zuweisbare Variable, du kannst der „Ein-Variable" nichts zurückschreiben. Auch `Swap(x + 1, y)` geht nicht.
>
> **Plattform-Hinweis**: BYREF wird im **VM-Pfad** (`gbrun.py --vm`) noch nicht unterstützt — der Compiler wirft einen klaren Fehler. Im **Tree-Walker** (Default, `gbrun.py` ohne `--vm`) funktioniert es voll. Für unser Buch arbeiten wir immer mit dem Tree-Walker; BYREF ist daher ohne Einschränkung benutzbar.

## Refactoring: das Spiel in Funktionen

Jetzt der Hauptteil: wir teilen den Code aus Kap 6 in Funktionen auf. Vier Funktionen sollen rauskommen:

- `Setup()` — einmal beim Start (Player-Position setzen, `SCREEN` öffnen)
- `Clamp(wert, lo, hi)` — der Helper aus dem vorherigen Abschnitt
- `UpdatePlayer()` — pro Frame: Eingabe lesen, Position aktualisieren, clampen
- `DrawPlayer()` — pro Frame: Player zeichnen

Die `WHILE`-Schleife wird zur kurzen Inhaltsangabe.

```basic
CONST WIDTH        AS INTEGER = 320
CONST HEIGHT       AS INTEGER = 240
CONST BG_COLOR     AS INTEGER = &H141E3C
CONST PLAYER_C     AS INTEGER = &HFFDC00
CONST PLAYER_W     AS INTEGER = 40
CONST PLAYER_H     AS INTEGER = 24
CONST PLAYER_SPEED AS INTEGER = 3

DIM player_x AS INTEGER
DIM player_y AS INTEGER

FUNCTION Clamp(wert AS INTEGER, lo AS INTEGER, hi AS INTEGER) AS INTEGER
    IF wert < lo THEN RETURN lo
    IF wert > hi THEN RETURN hi
    RETURN wert
END FUNCTION

SUB Setup()
    player_x = WIDTH / 2 - PLAYER_W / 2
    player_y = HEIGHT - PLAYER_H - 16
    SCREEN(WIDTH, HEIGHT, "Star Pilot", 2)
END SUB

SUB UpdatePlayer()
    IF KEYPRESSED(KEY_LEFT) THEN
        player_x -= PLAYER_SPEED
    END IF
    IF KEYPRESSED(KEY_RIGHT) THEN
        player_x += PLAYER_SPEED
    END IF
    player_x = Clamp(player_x, 0, WIDTH - PLAYER_W)
END SUB

SUB DrawPlayer()
    BOX(player_x, player_y, player_x + PLAYER_W, player_y + PLAYER_H, PLAYER_C)
END SUB

' --- Hauptprogramm ---
Setup()

WHILE NOT QUITREQUESTED()
    UpdatePlayer()

    CLS(BG_COLOR)
    DrawPlayer()
    FLIP()
    SLEEP(16)
WEND
```

Vergleiche mal mit dem Stand von Kap 6: die `WHILE`-Schleife hat **fünf Zeilen** Inhalt statt 15. Auf den ersten Blick siehst du, was passiert. Wenn du wissen willst, wie `UpdatePlayer` *funktioniert*, gehst du eine Stelle nach oben und liest dort. Das nennt man **Trennung der Belange** (separation of concerns) — ein Grundprinzip von gutem Code.

### Funktionen lesen globale Variablen

Beachte: in `Setup()`, `UpdatePlayer()` und `DrawPlayer()` greifen wir direkt auf `player_x` und `player_y` zu — obwohl die mit `DIM` ganz oben deklariert sind, *außerhalb* jeder Funktion. Das funktioniert, weil **globale Variablen** im ganzen Programm sichtbar sind, auch innerhalb von Funktionen.

> **Trade-off**: globale Variablen sind bequem, aber gefährlich bei größeren Programmen — jede Funktion kann sie ändern, das ist schwer nachvollziehbar. In Kap 9 werden wir die Player-Daten in eine `Class Player` verpacken; dann sind sie sauber gekapselt. Für jetzt sind globale Variablen aber okay.

### Funktionen vor oder nach dem Aufruf?

In unserem Beispiel stehen die Funktionen *oben*, das Hauptprogramm *unten*. Das ist Konvention, aber **nicht zwingend** — GameBasic kümmert sich darum, dass alle Funktionen schon „bekannt" sind, bevor das Hauptprogramm läuft. Du kannst auch das Hauptprogramm zuerst schreiben und die Funktionen darunter:

```basic
' Hauptprogramm hier
Setup()
WHILE NOT QUITREQUESTED()
    UpdatePlayer()
    ...
WEND

' Funktions-Definitionen weiter unten
SUB Setup()
    ...
END SUB
...
```

Das Programm läuft genauso. Welche Anordnung dir lieber ist, ist Geschmack — viele finden es lesbarer, das Hauptprogramm ganz unten zu haben (so wie wir's machen), weil das den Lesefluss wie ein Buch ergibt: erst kommen die Werkzeuge, dann der Einsatz.

## Übungen

**1. Distanz-Funktion.** Schreibe eine `FUNCTION Distance(x1 AS FLOAT, y1 AS FLOAT, x2 AS FLOAT, y2 AS FLOAT) AS FLOAT`, die den euklidischen Abstand zwischen zwei Punkten zurückgibt (`SQR((x2-x1)^2 + (y2-y1)^2)`). Teste mit `Distance(0, 0, 3, 4)` — soll `5.0` ergeben.

**2. Default-Spielerei.** Schreibe eine `SUB Spawn(x AS INTEGER, y AS INTEGER, hp AS INTEGER = 1, speed AS INTEGER = 2)`, die mit `PRINT` ausgibt was sie spawnen würde. Ruf sie viermal mit unterschiedlich vielen Argumenten auf — von `Spawn(10, 20)` bis `Spawn(10, 20, 5, 4)`.

**3. Clamp und FLOAT.** Wir haben `Clamp` für `INTEGER` geschrieben. Was ist mit `FLOAT`? Schreibe eine zweite Variante — gleicher Code, nur Typen geändert. Du kannst sie einfach `ClampF` nennen (eigene Konvention).

**4. Stretch — Multi-Return.** Schreibe eine `SUB GetMouseGrid(BYREF gx AS INTEGER, BYREF gy AS INTEGER, tile_size AS INTEGER = 16)`, die die Mausposition (`MOUSE_X()`, `MOUSE_Y()`) durch die Tile-Größe teilt und in `gx`, `gy` zurückgibt. So bekommst du raus, in welcher „Kachel" die Maus gerade ist. (Für unser Spiel brauchen wir das später nicht direkt, aber als Übung mit BYREF gut.)

## Zusammenfassung

Du hast in diesem Kapitel:

- den Unterschied zwischen `SUB` und `FUNCTION` verstanden,
- typisierte Parameter und Default-Werte kennengelernt,
- `BYREF` für Multi-Return-Funktionen wie `Swap` und `DivMod` benutzt,
- den Spielcode in vier saubere Funktionen aufgeteilt — `Setup`, `Clamp`, `UpdatePlayer`, `DrawPlayer`,
- gesehen, dass globale Variablen aus Funktionen heraus gelesen und geschrieben werden können (mit dem Trade-off von Globalen).

Im **nächsten Kapitel** wird's spaßiger: wir lassen den Player **schießen**. Dafür brauchen wir Arrays — viele Bullets gleichzeitig auf dem Bildschirm, jeder mit eigener Position.

## Code-Stand am Ende des Kapitels

- [`code/kap-07/01_sub_function.gb`](code/kap-07/01_sub_function.gb) — `SUB` und `FUNCTION` zum Anfangen
- [`code/kap-07/02_defaults.gb`](code/kap-07/02_defaults.gb) — Default-Werte, auch mit Bezug auf vorhergehende Parameter
- [`code/kap-07/03_byref_swap.gb`](code/kap-07/03_byref_swap.gb) — `Swap` und `DivMod` mit `BYREF`
- [`code/kap-07/main.gb`](code/kap-07/main.gb) — Star Pilot mit `Setup`/`UpdatePlayer`/`DrawPlayer`/`Clamp`
