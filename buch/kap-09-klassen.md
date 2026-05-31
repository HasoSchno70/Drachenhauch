# Kapitel 9 — Vom Array-Spaghetti zur Klasse

In Kapitel 8 haben wir mit drei parallelen Arrays gearbeitet — `bullets_x`, `bullets_y`, `bullets_alive`. Es funktioniert. Aber wenn wir mehr brauchen — Bullet-Farbe, Schaden, Owner — brauchen wir mehr Arrays. Und die zusammengehörigen Daten liegen verstreut in drei (bald fünf, sieben) verschiedenen Listen, jeder Index muss synchron bleiben.

Es gibt eine bessere Form: **Klassen**. Ein Bullet ist *ein Ding*, nicht *drei separate Listen mit Sync-Index*. Wir packen alles, was zu einem Bullet gehört — Position, Geschwindigkeit, Lebensstatus, Update-Logik, Draw-Logik — in *einen* benannten Block.

Das ist das Kapitel, in dem aus „Programm" so etwas wie „Software-Architektur" wird. Großes Wort, kleines Beispiel. Aber das Konzept hier wirst du in jedem Spiel, in jeder Sprache wiedersehen.

## Lernziele

Nach diesem Kapitel:

- definierst du Klassen mit Feldern (`DIM`) und Methoden (`SUB`/`FUNCTION`)
- erzeugst Instanzen mit `NEW Klassenname(...)` und der `Init`-Methode
- rufst Methoden mit `obj.method(...)` auf
- ersetzt drei parallel-Arrays durch ein einziges `ARRAY OF Bullet`
- entscheidest selbst, was als Klasse Sinn macht und was nicht

## Die Schmerzpunkte aus Kap 8

Schau dir nochmal den Code aus Kap 8 an. Drei kritische Stellen:

```basic
DIM bullets_x[POOL_SIZE]     AS INTEGER
DIM bullets_y[POOL_SIZE]     AS INTEGER
DIM bullets_alive[POOL_SIZE] AS BOOLEAN
```

```basic
SUB SpawnBullet(x AS INTEGER, y AS INTEGER)
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        IF NOT bullets_alive[i] THEN
            bullets_x[i] = x
            bullets_y[i] = y
            bullets_alive[i] = TRUE
            ...
```

Jeder Bullet wird in **drei** Arrays gleichzeitig manipuliert. Wer eine Eigenschaft vergisst, hat einen subtilen Bug. Wer ein viertes Feld hinzufügen will (sagen wir, Schaden), muss sechs Stellen anfassen: das neue Array deklarieren, in `SpawnBullet` setzen, in `Update` ggf. lesen, in `Draw` ggf. nutzen — und dabei wieder synchron mit dem Index halten.

Das ist die Spaghetti, die wir loswerden wollen.

## Schritt 1: Was eine Klasse ist

Eine **Klasse** ist eine Schablone. Sie sagt: „so sieht ein Bullet aus — diese Felder, diese Methoden". Aus der Schablone produzieren wir **Instanzen** — konkrete Bullet-Objekte mit eigenen Werten. Klasse: der Plan. Instanz: das gebaute Ding.

Syntax:

```basic
CLASS Klassenname
    DIM feld1 AS Typ
    DIM feld2 AS Typ
    ...

    SUB Methode1(parameter AS Typ)
        ' Body
    END SUB

    FUNCTION Methode2(...) AS Typ
        ' Body, mit RETURN
    END FUNCTION
END CLASS
```

Drei Dinge sind drin:

- **Felder** mit `DIM` — die Daten, die jede Instanz für sich hält.
- **Methoden** mit `SUB`/`FUNCTION` — die Operationen, die auf diesen Daten arbeiten.
- **Eine spezielle Methode `Init`** (gleich) — wird automatisch beim Erzeugen aufgerufen.

## Schritt 2: Player als Klasse

Bauen wir eine kleine Player-Klasse, isoliert, in der Konsole — bevor wir den Spielcode anpassen.

```basic
CLASS Player
    DIM x      AS INTEGER
    DIM y      AS INTEGER
    DIM lives  AS INTEGER

    SUB Init(start_x AS INTEGER, start_y AS INTEGER)
        x = start_x
        y = start_y
        lives = 3
    END SUB

    SUB MoveBy(dx AS INTEGER, dy AS INTEGER)
        x = x + dx
        y = y + dy
    END SUB

    SUB TakeDamage()
        lives = lives - 1
    END SUB

    FUNCTION IsAlive() AS BOOLEAN
        RETURN lives > 0
    END FUNCTION
END CLASS
```

Drei Felder, vier Methoden. Lass uns die wichtigen Punkte besprechen:

### Methoden-Bodies sehen Felder direkt

In `MoveBy` schreiben wir `x = x + dx` — kein `self.x`, kein `this.x`. GameBasic-Methoden sehen die Felder ihrer Klasse, als wären es lokale Variablen. Wer aus Python oder Java kommt, muss kurz umdenken — es ist aber sehr lesbar.

> **Hintergrund**: GameBasic legt während der Methoden-Ausführung einen extra Scope an, der die Felder der aktuellen Instanz enthält. Wenn du in `MoveBy` `x` schreibst, sucht die Sprache erst lokal (kein Treffer), dann im Instanz-Scope (Treffer: das Feld `x`). Schreibst du `dx`, ist es lokal (Parameter). Funktioniert wie erwartet, ohne `self`-Präfix.

### Init: der Konstruktor

`Init` ist eine reservierte Methode — sie wird automatisch aufgerufen, wenn du `NEW Player(...)` schreibst. Die Argumente in der Klammer gehen an `Init`.

```basic
DIM hero AS Player
hero = NEW Player(100, 200)
```

Das macht zweierlei: ein neues Player-Objekt allokieren, und dann `hero.Init(100, 200)` aufrufen. In `Init` setzen wir die Anfangswerte der Felder.

> **Init muss nicht da sein.** Eine Klasse ohne `Init` ist auch erlaubt — dann ist `NEW Klasse()` nur die leere Allokation, alle Felder bleiben auf ihrem Typ-Default (Integer 0, Boolean FALSE, ...). Wenn du `NEW Klasse(args)` mit Argumenten aufrufst, brauchst du aber eine passende `Init`.

### Felder von außen sehen

Ein Feld liest man von außen mit `obj.feldname`:

```basic
PRINT f"Player startet bei ({hero.x}, {hero.y}) mit {hero.lives} Leben"
```

Output:

```
Player startet bei (100, 200) mit 3 Leben
```

Auch schreiben geht (`hero.lives = 5`) — wobei das oft kein guter Stil ist. Lieber Methoden, die das kontrolliert tun (`hero.TakeDamage()`).

### Methoden aufrufen

Methoden ruft man genauso auf — `obj.methode(args)`:

```basic
hero.MoveBy(10, -5)
hero.TakeDamage()
PRINT f"Lebt: {hero.IsAlive()}"
```

Der ganze Test-Lauf:

```basic
DIM hero AS Player
hero = NEW Player(100, 200)
PRINT f"Player startet bei ({hero.x}, {hero.y}) mit {hero.lives} Leben"

hero.MoveBy(10, -5)
PRINT f"Nach MoveBy: ({hero.x}, {hero.y})"

hero.TakeDamage()
hero.TakeDamage()
PRINT f"Nach 2 Treffern: {hero.lives} Leben, lebt: {hero.IsAlive()}"

hero.TakeDamage()
PRINT f"Nach 3 Treffern: {hero.lives} Leben, lebt: {hero.IsAlive()}"
```

Output:

```
Player startet bei (100, 200) mit 3 Leben
Nach MoveBy: (110, 195)
Nach 2 Treffern: 1 Leben, lebt: TRUE
Nach 3 Treffern: 0 Leben, lebt: FALSE
```

## Schritt 3: Bullet als Klasse

Genauso für Bullets, jetzt mit der **Update-Methode**:

```basic
CLASS Bullet
    DIM x      AS INTEGER
    DIM y      AS INTEGER
    DIM speed  AS INTEGER
    DIM alive  AS BOOLEAN

    SUB Init(start_x AS INTEGER, start_y AS INTEGER)
        x = start_x
        y = start_y
        speed = 5
        alive = TRUE
    END SUB

    SUB Update()
        IF NOT alive THEN RETURN
        y = y - speed
        IF y < -10 THEN
            alive = FALSE
        END IF
    END SUB
END CLASS
```

Die `Update`-Methode kennt sich selbst aus — sie weiß, welcher Bullet sie ist (welche `x`, `y`, `speed` sie hat). Aufruf:

```basic
DIM b AS Bullet
b = NEW Bullet(160, 210)

DIM frame AS INTEGER
FOR frame = 1 TO 10
    b.Update()
NEXT frame
PRINT f"Nach 10 Frames: y = {b.y}"
```

Output: nach 10 Frames ist `b.y` um `10 * 5 = 50` Pixel kleiner — perfekt nachvollziehbar.

## Schritt 4: Array von Klassen-Instanzen

Jetzt die zentrale Frage: wie wird aus den drei `bullets_*`-Arrays **ein** `bullets[]`?

```basic
DIM bullets[POOL_SIZE] AS Bullet
```

Das deklariert ein Array von 20 Bullet-Slots. **Aber Achtung**: nach dieser Zeile sind alle Slots **NIL** (also: kein Bullet drin, nur Platzhalter). Du musst jeden Slot mit `NEW Bullet(...)` befüllen, bevor du ihn benutzen kannst:

```basic
DIM i AS INTEGER
FOR i = 0 TO POOL_SIZE - 1
    bullets[i] = NEW Bullet()
NEXT i
```

Das passiert in `Setup()`. Danach steht in jedem Slot ein echter Bullet — alle bisher tot, mit `alive = FALSE`. Bereit für den ersten Schuss.

> **Stolperfalle**: ohne den `NEW Bullet()`-Loop würden die Slots NIL bleiben, und beim ersten `bullets[i].alive` käme ein Fehler („Zugriff auf NIL-Referenz").

### Spawn: wie vorher, aber mit Methoden

Statt drei Array-Zuweisungen rufen wir jetzt eine Methode:

```basic
SUB SpawnBullet(at_x AS INTEGER, at_y AS INTEGER)
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        IF NOT bullets[i].alive THEN
            bullets[i].Spawn(at_x, at_y)
            RETURN
        END IF
    NEXT i
END SUB
```

`bullets[i].Spawn(at_x, at_y)` — Methode auf einem Array-Element. GameBasic löst das so auf: erst `bullets[i]` liefert den Bullet im Slot, dann wird `.Spawn(...)` darauf aufgerufen.

Die `Spawn`-Methode (selbst in der Bullet-Klasse) macht das, was vorher die drei Array-Zuweisungen waren:

```basic
SUB Spawn(at_x AS INTEGER, at_y AS INTEGER)
    x = at_x
    y = at_y
    alive = TRUE
END SUB
```

> **Warum Spawn und nicht einfach Init?** `Init` läuft einmal beim `NEW`. Ein Bullet im Pool wird aber **mehrfach** lebendig — nach Tod kann derselbe Slot wiederverwendet werden. `Spawn` ist die „erweck-mich-wieder"-Operation; `Init` setzt die einmaligen Werte beim Erzeugen.

### Update und Draw

Identisches Muster:

```basic
FOR i = 0 TO POOL_SIZE - 1
    bullets[i].Update()
NEXT i
```

Die Logik (was passiert beim Update?) lebt **in der Klasse**, nicht in einer freien Funktion außerhalb. Der Hauptcode sagt nur: „Jeder Bullet, mach dein Ding."

## Schritt 5: Das vollständige Spiel

Hier der komplette Code mit Klassen. Vergleiche mit dem Stand aus Kap 8 — die `WHILE`-Schleife ist gleich kurz, aber die Logik ist klar zugeordnet:

```basic
CONST WIDTH        AS INTEGER = 320
CONST HEIGHT       AS INTEGER = 240
CONST BG_COLOR     AS INTEGER = &H141E3C
CONST PLAYER_C     AS INTEGER = &HFFDC00
CONST BULLET_C     AS INTEGER = &HFFFFFF
CONST PLAYER_SPEED AS INTEGER = 3
CONST BULLET_SPEED AS INTEGER = 5
CONST POOL_SIZE    AS INTEGER = 20

FUNCTION Clamp(wert AS INTEGER, lo AS INTEGER, hi AS INTEGER) AS INTEGER
    IF wert < lo THEN RETURN lo
    IF wert > hi THEN RETURN hi
    RETURN wert
END FUNCTION

CLASS Player
    DIM x AS INTEGER
    DIM y AS INTEGER
    DIM w AS INTEGER
    DIM h AS INTEGER

    SUB Init(start_x AS INTEGER, start_y AS INTEGER)
        x = start_x
        y = start_y
        w = 40
        h = 24
    END SUB

    SUB Update()
        IF KEYPRESSED(KEY_LEFT) THEN
            x = x - PLAYER_SPEED
        END IF
        IF KEYPRESSED(KEY_RIGHT) THEN
            x = x + PLAYER_SPEED
        END IF
        x = Clamp(x, 0, WIDTH - w)
    END SUB

    SUB Draw()
        BOX(x, y, x + w, y + h, PLAYER_C)
    END SUB
END CLASS

CLASS Bullet
    DIM x      AS INTEGER
    DIM y      AS INTEGER
    DIM w      AS INTEGER
    DIM h      AS INTEGER
    DIM alive  AS BOOLEAN

    SUB Init()
        w = 3
        h = 8
        alive = FALSE
    END SUB

    SUB Spawn(at_x AS INTEGER, at_y AS INTEGER)
        x = at_x
        y = at_y
        alive = TRUE
    END SUB

    SUB Update()
        IF NOT alive THEN RETURN
        y = y - BULLET_SPEED
        IF y < -h THEN
            alive = FALSE
        END IF
    END SUB

    SUB Draw()
        IF alive THEN
            BOX(x, y, x + w, y + h, BULLET_C)
        END IF
    END SUB
END CLASS

DIM player AS Player
DIM bullets[POOL_SIZE] AS Bullet
DIM space_was_pressed AS BOOLEAN

SUB Setup()
    player = NEW Player(WIDTH / 2 - 20, HEIGHT - 24 - 16)

    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        bullets[i] = NEW Bullet()
    NEXT i

    space_was_pressed = FALSE
    SCREEN(WIDTH, HEIGHT, "Star Pilot", 2)
END SUB

SUB SpawnBullet(at_x AS INTEGER, at_y AS INTEGER)
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        IF NOT bullets[i].alive THEN
            bullets[i].Spawn(at_x, at_y)
            RETURN
        END IF
    NEXT i
END SUB

SUB UpdateAll()
    player.Update()

    DIM space_now AS BOOLEAN
    space_now = KEYPRESSED(KEY_SPACE)
    IF space_now AND NOT space_was_pressed THEN
        SpawnBullet(player.x + player.w / 2 - 1, player.y - 8)
    END IF
    space_was_pressed = space_now

    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        bullets[i].Update()
    NEXT i
END SUB

SUB DrawAll()
    player.Draw()
    DIM i AS INTEGER
    FOR i = 0 TO POOL_SIZE - 1
        bullets[i].Draw()
    NEXT i
END SUB

Setup()

WHILE NOT QUITREQUESTED()
    UpdateAll()

    CLS(BG_COLOR)
    DrawAll()
    FLIP()
    SLEEP(16)
WEND
```

Run drücken. Spielt sich identisch zu Kap 8 — Player bewegt sich, Bullets fliegen. Aber der Code ist anders strukturiert: jede Klasse weiß selbst, wie sie sich bewegt und zeichnet.

> **Beobachte**: in `UpdateAll` rufen wir nur `player.Update()` und `bullets[i].Update()` auf — **die Logik selbst** steht in den Klassen. Wenn du in Kap 11 Kollisionen hinzufügst, schreibst du eine Methode `Bullet.HitsRect(...)` und nicht eine freie Funktion mit fünf Parametern.

## STRUCT vs CLASS: ein kurzer Ausblick

GameBasic hat noch ein zweites verwandtes Konstrukt — `STRUCT`. Syntaktisch fast identisch zur `CLASS`, aber mit zwei Unterschieden:

- **STRUCT-Variablen sind „direkt"**: `DIM v AS Vector2` legt sofort ein Vector2-Objekt an, kein `NEW` nötig.
- **STRUCTs verhalten sich wie Werte**: bei Zuweisung wird **kopiert**, nicht referenziert. Bei `CLASS` sind zwei Variablen, die auf dieselbe Instanz zeigen, *dieselbe* Instanz — bei `STRUCT` sind es zwei unabhängige Kopien.

Wann was?

- **CLASS** für „lebendige" Spielobjekte: Player, Enemy, Bullet — Dinge mit Identität, Lebenszyklus, viel Logik.
- **STRUCT** für reine Daten: ein 2D-Punkt, eine RGB-Farbe, eine Bounding-Box. Kleine, immutable-artige Wertehaufen.

Wir nutzen für Star Pilot durchgehend `CLASS` — STRUCTs sind Stoff für später, wenn dir der Unterschied im richtigen Moment wichtig wird.

## Übungen

**1. Player erweitern.** Füge der `Player`-Klasse ein Feld `lives AS INTEGER` hinzu (Init mit 3) und eine Methode `TakeDamage()` (lives - 1). Drucke nach jedem Frame `player.lives` per `TEXT(...)` (siehe [Grafik-Built-ins](../docs/builtins-grafik.md)) ins Spielfenster oben links.

**2. Reset-Methode.** Schreibe eine Methode `Player.Reset()`, die Position und Lives auf Anfangswerte zurücksetzt. Idee: bei Druck auf `KEY_R` während des Spiels alles auf Null. Praktisch zum schnellen Testen.

**3. Doppelte Geschwindigkeit für einen Bullet-Typ.** Erweitere `Bullet` um ein Feld `is_super AS BOOLEAN` (Default FALSE in `Init`). In `Spawn` einen weiteren Parameter `super = FALSE`. Wenn `is_super = TRUE`, doppelte Geschwindigkeit in `Update`. Wenn der Player **Shift+Leertaste** drückt, spawn er einen Super-Bullet.

**4. Stretch — Vektor-STRUCT.** Schreibe eine `STRUCT Vector2` mit Feldern `x`, `y` (FLOAT). Statt zwei separate Felder `x`, `y` in Player und Bullet zu haben, könntest du jeder Klasse ein Feld `pos AS Vector2` geben. Das ist mehr Tipparbeit für ein kleines Projekt wie unseres, aber bei großen Spielen Standard. Probier es als Übung — und beobachte, was sich am Code ändert.

## Zusammenfassung

Du hast in diesem Kapitel:

- den Sprung von „Daten in parallelen Listen" zu „Objekt mit Daten und Logik" gemacht,
- Klassen mit Feldern und Methoden definiert,
- Instanzen mit `NEW` erzeugt und `Init` als Konstruktor benutzt,
- ein `ARRAY OF Bullet` statt drei parallele Arrays verwendet,
- den Unterschied zwischen `CLASS` (lebendige Objekte) und `STRUCT` (reine Werte) kennengelernt.

Im **nächsten Kapitel** kommen die Gegner — und damit zum ersten Mal **Vererbung**. Player, Bullet und Enemy haben alle eine Position, ein Update, ein Draw. Statt das in jeder Klasse zu wiederholen, schreiben wir eine Basisklasse `Entity` und leiten die anderen davon ab.

## Code-Stand am Ende des Kapitels

- [`code/kap-09/01_class_player.gb`](code/kap-09/01_class_player.gb) — Player-Klasse isoliert mit MoveBy/TakeDamage/IsAlive
- [`code/kap-09/02_class_bullet.gb`](code/kap-09/02_class_bullet.gb) — Bullet-Klasse mit Update-Methode (Konsolen-Test)
- [`code/kap-09/main.gb`](code/kap-09/main.gb) — Star Pilot mit Player- und Bullet-Klassen statt Parallel-Arrays
