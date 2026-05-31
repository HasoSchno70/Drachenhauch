# Kapitel 8 — Arrays: viele Schüsse gleichzeitig

In Kapitel 7 haben wir den Code aufgeräumt — gut. In diesem Kapitel passiert das, worauf du wahrscheinlich seit Kapitel 5 wartest: **wir schießen**. Leertaste drücken, ein heller Strich saust nach oben, am Bildschirmrand verschwindet er. So oft wie du willst.

Damit das funktioniert, brauchen wir **mehrere Bullets gleichzeitig** auf dem Bildschirm — bis zu zwanzig. Eine eigene Variable für jeden? `bullet_1_x`, `bullet_2_x`, ..., `bullet_20_x`? Wäre absurd. Stattdessen lernen wir **Arrays** kennen — den klassischen Weg, eine Liste gleichartiger Werte unter einem Namen zu halten.

## Lernziele

Nach diesem Kapitel:

- deklarierst du Arrays mit `DIM zahlen[10] AS INTEGER`
- liest und schreibst du Array-Elemente per Index: `zahlen[i] = ...`
- kennst du das **Pool-Konzept**: Slots, die zwischen lebendig und tot wechseln
- nutzt du **Parallel-Arrays**, um mehrere Eigenschaften pro Element zu speichern
- hast du in Star Pilot die Schießen-Mechanik integriert

## Schritt 1: Was ist ein Array?

Ein Array ist eine **Liste mit fester Größe** — du sagst beim Anlegen, wie viele Elemente reinpassen, und kannst dann auf jedes per Position (= Index) zugreifen.

```basic
DIM zahlen[5] AS INTEGER

zahlen[0] = 10
zahlen[1] = 20
zahlen[2] = 30
zahlen[3] = 40
zahlen[4] = 50
```

Das `[5]` direkt nach dem Namen sagt: dieses Array hat **5 Slots**. Die Slots heißen `zahlen[0]`, `zahlen[1]`, ..., `zahlen[4]`. Beachte: die **Indizierung beginnt bei 0**, nicht bei 1.

> **Stolperfalle**: bei `DIM zahlen[5]` sind die gültigen Indizes `0` bis `4`. Wer `zahlen[5]` schreibt, bekommt einen Fehler — das wäre der „sechste" Slot, den's nicht gibt. Der Computer-Klassiker schlechthin: die meisten Indexfehler stammen von „off-by-one"-Verwechslungen. Merksatz: **N Elemente → Indizes 0 bis N-1**.

### Mit FOR durch ein Array

Arrays und `FOR`-Schleifen gehören zusammen wie Brot und Butter:

```basic
DIM i AS INTEGER
FOR i = 0 TO 4
    PRINT f"zahlen[{i}] = {zahlen[i]}"
NEXT i
```

Output:

```
zahlen[0] = 10
zahlen[1] = 20
zahlen[2] = 30
zahlen[3] = 40
zahlen[4] = 50
```

Statt fünf einzelne `PRINT`-Zeilen zu schreiben, durchlaufen wir das Array mit einer Schleife. Funktioniert genauso für Berechnungen:

```basic
DIM summe AS INTEGER
summe = 0
FOR i = 0 TO 4
    summe += zahlen[i]
NEXT i
PRINT f"Summe: {summe}"        ' "Summe: 150"
```

Was hier so unscheinbar aussieht, ist eines der wichtigsten Patterns in der Programmierung: **„durchlaufe alle Elemente und tu was mit ihnen"**. Du wirst es in jedem Spiel-Loop wiedererkennen.

> **Tipp**: bei `FOR i = 0 TO N - 1` ist es einfach, die `-1` zu vergessen. Wenn du eine Konstante `POOL_SIZE` hast, schreibst du sicherer: `FOR i = 0 TO POOL_SIZE - 1`. Selbst wenn du später `POOL_SIZE` änderst, passt die Schleife sich an.

## Schritt 2: Parallel-Arrays

Ein Bullet ist nicht nur eine Zahl — er hat mindestens drei Eigenschaften: **X-Position**, **Y-Position** und ob er **lebt** (oder den Slot frei hält). Wie speichern wir das?

Eine Möglichkeit: drei separate Arrays, alle gleich groß:

```basic
CONST POOL_SIZE AS INTEGER = 5

DIM bullets_x[POOL_SIZE]      AS INTEGER
DIM bullets_y[POOL_SIZE]      AS INTEGER
DIM bullets_alive[POOL_SIZE]  AS BOOLEAN
```

Index `i` in **allen drei Arrays** gehört zum selben Bullet. `bullets_x[3]`, `bullets_y[3]`, `bullets_alive[3]` — das ist der vierte Bullet (Index 3) mit seinen drei Eigenschaften.

Diese Anordnung heißt **Parallel-Arrays**. Sie ist nicht die schönste Lösung — in Kap 9 lernen wir Klassen kennen, dann wird `Bullet.x`, `Bullet.y`, `Bullet.alive` — aber für jetzt ist sie funktional und einfach zu verstehen.

> **Trade-off**: Parallel-Arrays haben einen Performance-Vorteil (eng aneinander liegende Daten, gut für CPU-Cache) und einen Lesbarkeits-Nachteil (man muss beim Sehen von `bullets_x[7]` mitdenken: „Index 7 in welchen Arrays sonst noch?"). Profis nennen das **Struct-of-Arrays** vs. **Array-of-Structs**. Wir kümmern uns hier nicht um Performance — wir nehmen Parallel-Arrays nur, weil wir Klassen noch nicht haben.

## Schritt 3: Das Pool-Konzept

Wir wollen bis zu 20 Bullets gleichzeitig haben. Aber **nicht alle Slots sind immer aktiv** — wenn der Player gerade einen Schuss abgegeben hat, ist nur einer aktiv, der Rest „leer". Wenn er hektisch fünfmal feuert, sind fünf aktiv.

Wir lösen das mit einem **Pool**: ein festes Array, dessen Slots zwischen „lebt" und „tot" wechseln. Die `bullets_alive`-Liste merkt sich pro Slot, was los ist.

**Lebenszyklus eines Slots**:

1. **Anfang**: alle Slots sind tot (`bullets_alive[i] = FALSE`).
2. **Schießen**: ersten freien Slot suchen, mit Position füllen, auf TRUE setzen.
3. **Update**: alle TRUE-Slots nach oben bewegen. Wer den Bildschirm verlässt → wieder auf FALSE.
4. **Wiederverwenden**: tote Slots stehen für den nächsten Schuss zur Verfügung.

Pool ist nicht spielspezifisch — du wirst dasselbe Muster bei Enemies (Kap 10), Particles (Kap 13) und allem, was zur Laufzeit dynamisch entsteht und vergeht, wiedersehen.

### Spawn: ersten freien Slot finden

```basic
SUB SpawnBullet(x AS INTEGER, y AS INTEGER)
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        IF NOT bullets_alive[i] THEN
            bullets_x[i] = x
            bullets_y[i] = y
            bullets_alive[i] = TRUE
            RETURN
        END IF
    NEXT i
    ' Pool voll - Schuss verworfen, kein Drama
END SUB

```

Wir gehen die Slots durch, bis wir einen toten finden. Dann füllen wir ihn und beenden die SUB mit `RETURN`. Wenn keiner frei ist, läuft die Schleife durch und wir tun nichts — der Schuss „geht verloren". Bei einer Pool-Größe von 20 passiert das praktisch nie, also kein Problem.

### Update: alle aktiven Bullets nach oben

```basic
SUB UpdateBullets()
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        IF bullets_alive[i] THEN
            bullets_y[i] -= BULLET_SPEED
            IF bullets_y[i] < -BULLET_H THEN
                bullets_alive[i] = FALSE
            END IF
        END IF
    NEXT i
END SUB
```

Pro Frame: jeden Slot prüfen. Lebt er? Dann nach oben bewegen (`y -= BULLET_SPEED`). Ist er aus dem Bild? Dann auf tot setzen.

> **Aha-Moment**: warum `bullets_y[i] < -BULLET_H` und nicht `< 0`? Weil `bullets_y[i]` die **obere Kante** des Bullets ist. Wenn die nur ein Pixel über dem oberen Bildrand ist (`y = -1`), ist der **Rest** des Bullets noch sichtbar — er müsste also weiter dargestellt werden. Erst wenn die obere Kante so weit oben ist, dass auch die untere Kante (`y + BULLET_H`) verschwunden ist, ist der Bullet wirklich aus dem Bild. Das ist bei `y = -BULLET_H`.

### Draw: alle aktiven Bullets zeichnen

```basic
SUB DrawBullets()
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        IF bullets_alive[i] THEN
            BOX(bullets_x[i], bullets_y[i], _
                bullets_x[i] + BULLET_W, bullets_y[i] + BULLET_H, BULLET_C)
        END IF
    NEXT i
END SUB
```

Dasselbe Pattern wie Update — durchlaufen, prüfen, was tun. Diesmal zeichnen.

> **Zeilenfortsetzung**: Der Underscore `_` am Ende einer Zeile ist GameBasics Art zu sagen „die nächste Zeile gehört dazu". Praktisch wenn ein Funktions-Aufruf zu lang für eine Zeile wird.

## Schritt 4: Edge-Detection beim Schießen

Eine kleine, aber wichtige Subtilität. Naiv würden wir schreiben:

```basic
IF KEYPRESSED(KEY_SPACE) THEN
    SpawnBullet(...)
END IF
```

Das Problem: bei 60 FPS und gehaltener Leertaste würde jeder Frame einen neuen Bullet erzeugen — 60 Bullets pro Sekunde. Pool wäre in einem Drittel-Sekunde voll, danach kein Schuss mehr.

Besser: **nur beim Übergang von „nicht gedrückt" zu „gerade gedrückt" schießen**. Das nennt man Edge-Detection (Flanken-Erkennung).

```basic
DIM space_was_pressed AS BOOLEAN

' ... in UpdatePlayer() ...
DIM space_now AS BOOLEAN
space_now = KEYPRESSED(KEY_SPACE)
IF space_now AND NOT space_was_pressed THEN
    SpawnBullet(...)
END IF
space_was_pressed = space_now
```

Schritt für Schritt:

- Wir merken uns über Frames hinweg, ob die Taste **letzten** Frame gedrückt war (`space_was_pressed`).
- Wenn sie **diesen** Frame gedrückt ist (`space_now = TRUE`) **aber letzten Frame nicht** (`NOT space_was_pressed`), dann ist das eine frische Druckaktion → schießen.
- Am Ende merken wir uns den aktuellen Status für den nächsten Frame.

Ergebnis: **ein Bullet pro Drücker**, egal wie lange du die Taste hältst. Loslassen + nochmal drücken = nächster Bullet.

> **Erweiterung für später**: Edge-Detection ist die einfachste Lösung. Manche Spiele lassen dich aber **automatisch** schießen, solange du hältst — nur eben nicht 60-mal pro Sekunde, sondern z.B. fünfmal. Dafür braucht man `MILLIS()` und einen Cooldown-Timer (eine Variable, die merkt, wann der letzte Schuss war). Das ist Übung 2 unten.

## Schritt 5: Alles zusammen

Hier der vollständige Spielcode. Vergleiche mit dem `main.gb` aus Kap 7 — der Player-Teil ist gleich, dazugekommen sind die Bullet-Arrays und die Funktionen `SpawnBullet`, `UpdateBullets`, `DrawBullets`. Außerdem hat `UpdatePlayer` jetzt die Schuss-Logik mit Edge-Detection.

```basic
CONST WIDTH        AS INTEGER = 320
CONST HEIGHT       AS INTEGER = 240
CONST BG_COLOR     AS INTEGER = &H141E3C
CONST PLAYER_C     AS INTEGER = &HFFDC00
CONST BULLET_C     AS INTEGER = &HFFFFFF
CONST PLAYER_W     AS INTEGER = 40
CONST PLAYER_H     AS INTEGER = 24
CONST PLAYER_SPEED AS INTEGER = 3
CONST BULLET_W     AS INTEGER = 3
CONST BULLET_H     AS INTEGER = 8
CONST BULLET_SPEED AS INTEGER = 5
CONST POOL_SIZE    AS INTEGER = 20

DIM player_x AS INTEGER
DIM player_y AS INTEGER

DIM bullets_x[POOL_SIZE]     AS INTEGER
DIM bullets_y[POOL_SIZE]     AS INTEGER
DIM bullets_alive[POOL_SIZE] AS BOOLEAN

DIM space_was_pressed AS BOOLEAN

FUNCTION Clamp(wert AS INTEGER, lo AS INTEGER, hi AS INTEGER) AS INTEGER
    IF wert < lo THEN RETURN lo
    IF wert > hi THEN RETURN hi
    RETURN wert
END FUNCTION

SUB Setup()
    player_x = WIDTH / 2 - PLAYER_W / 2
    player_y = HEIGHT - PLAYER_H - 16
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        bullets_alive[i] = FALSE
    NEXT i
    space_was_pressed = FALSE
    SCREEN(WIDTH, HEIGHT, "Star Pilot", 2)
END SUB

SUB SpawnBullet(x AS INTEGER, y AS INTEGER)
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        IF NOT bullets_alive[i] THEN
            bullets_x[i] = x
            bullets_y[i] = y
            bullets_alive[i] = TRUE
            RETURN
        END IF
    NEXT i
END SUB

SUB UpdatePlayer()
    IF KEYPRESSED(KEY_LEFT) THEN
        player_x -= PLAYER_SPEED
    END IF
    IF KEYPRESSED(KEY_RIGHT) THEN
        player_x += PLAYER_SPEED
    END IF
    player_x = Clamp(player_x, 0, WIDTH - PLAYER_W)

    DIM space_now AS BOOLEAN
    space_now = KEYPRESSED(KEY_SPACE)
    IF space_now AND NOT space_was_pressed THEN
        SpawnBullet(player_x + PLAYER_W / 2 - BULLET_W / 2, player_y - BULLET_H)
    END IF
    space_was_pressed = space_now
END SUB

SUB UpdateBullets()
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        IF bullets_alive[i] THEN
            bullets_y[i] -= BULLET_SPEED
            IF bullets_y[i] < -BULLET_H THEN
                bullets_alive[i] = FALSE
            END IF
        END IF
    NEXT i
END SUB

SUB DrawPlayer()
    BOX(player_x, player_y, player_x + PLAYER_W, player_y + PLAYER_H, PLAYER_C)
END SUB

SUB DrawBullets()
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        IF bullets_alive[i] THEN
            BOX(bullets_x[i], bullets_y[i], _
                bullets_x[i] + BULLET_W, bullets_y[i] + BULLET_H, BULLET_C)
        END IF
    NEXT i
END SUB

' --- Hauptprogramm ---
Setup()

WHILE NOT QUITREQUESTED()
    UpdatePlayer()
    UpdateBullets()

    CLS(BG_COLOR)
    DrawPlayer()
    DrawBullets()
    FLIP()
    SLEEP(16)
WEND
```

Run drücken. Pfeiltasten zum Bewegen, Leertaste zum Schießen. Du solltest weiße Striche aus deinem Schiff fliegen sehen, schön nach oben bis zum Bildschirmrand.

> **Beobachte**: drücke Leertaste **schnell hintereinander** — pro Druck ein Bullet. Halte sie gedrückt — nur **ein** Bullet beim ersten Frame, dann nichts mehr (bis du loslässt und neu drückst). Genau das wollten wir.

## Vorgriff: warum das in Kap 9 schöner wird

Schau dir den Code an. Drei `bullets_*`-Arrays. Wer einen vierten Wert pro Bullet braucht (sagen wir, `bullets_dx` für seitliche Bewegung), legt ein viertes Array an. Wer fünf Felder hat, fünf Arrays. Die Logik ist verstreut:

```basic
bullets_x[i] = x
bullets_y[i] = y
bullets_alive[i] = TRUE
' ... ggf. bullets_dx[i] = ..., bullets_color[i] = ..., bullets_owner[i] = ...
```

In Kapitel 9 lernen wir **Klassen**. Statt parallel-Arrays schreiben wir:

```basic
CLASS Bullet
    DIM x AS INTEGER
    DIM y AS INTEGER
    DIM alive AS BOOLEAN

    SUB Update()
        ' ...
    END SUB
END CLASS

DIM bullets[POOL_SIZE] AS Bullet
```

Ein Array von `Bullet`-Objekten. Pro Bullet alle Eigenschaften zusammen, die Logik (Update, Draw) als Methode an der Klasse. Viel sauberer.

Aber **erst** mussten wir Arrays als Konzept lernen — und wir mussten den Schmerz der Parallel-Arrays kurz spüren, damit Klassen sich später wie eine Erleichterung anfühlen.

## Übungen

**1. Pool kleiner machen.** Setze `POOL_SIZE` von 20 auf 5. Drücke wild Leertaste. Was passiert? Verstehst du, warum Bullets „verschluckt" werden? (Antwort steht im Kapitel — der Pool ist voll, neue Schüsse werden verworfen.)

**2. Schussfrequenz mit Cooldown.** Statt Edge-Detection: erlaube **kontinuierliches** Schießen, aber begrenze auf maximal alle 200 ms einen Schuss. Hinweis: lege eine Variable `last_shot_ms AS INTEGER` an, und prüfe `MILLIS() - last_shot_ms >= 200` bevor du spawnst. Bei jedem Schuss `last_shot_ms = MILLIS()`.

**3. Diagonal-Bullets.** Erweitere die Bullet-Logik um `bullets_dx[POOL_SIZE]` — eine X-Bewegungs-Komponente. In `UpdateBullets` zusätzlich `bullets_x[i] += bullets_dx[i]`. Spawn-Funktion bekommt einen weiteren Parameter `dx`. Probier `SpawnBullet(..., -1)` und `SpawnBullet(..., +1)` — schräge Schüsse!

**4. Stretch — Drei-Schuss-Salve.** Beim Drücken der Leertaste sollen **drei** Bullets gleichzeitig spawnen — einer gerade nach oben, einer leicht nach links, einer leicht nach rechts. Nutze die diagonale Variante aus Übung 3.

## Zusammenfassung

Du hast in diesem Kapitel:

- Arrays als Listen mit fester Größe kennengelernt,
- mit Index-Zugriff (`array[i]`) gelesen und geschrieben,
- Parallel-Arrays für Bullet-Eigenschaften benutzt (X, Y, alive),
- das **Pool-Konzept** verstanden — Slots, die zwischen tot und lebendig wechseln,
- den Spielcode um Schießen-Mechanik erweitert: Edge-Detection, Spawn, Update, Draw,
- gesehen, dass das alles in Kap 9 mit Klassen viel kompakter wird.

Im **nächsten Kapitel** kommen die **Klassen**. Wir packen Player und Bullet jeweils in eine eigene `CLASS`, und der Code wird endgültig lesbar — mit `bullet.Update()` und `bullet.Draw()` statt verstreuten Helper-Funktionen.

## Code-Stand am Ende des Kapitels

- [`code/kap-08/01_arrays.gb`](code/kap-08/01_arrays.gb) — Array-Grundlagen mit Summe und Maximum
- [`code/kap-08/02_bullets_pool.gb`](code/kap-08/02_bullets_pool.gb) — Pool-Mechanik isoliert in der Konsole
- [`code/kap-08/main.gb`](code/kap-08/main.gb) — Star Pilot mit Schieß-Mechanik und Bullet-Pool
