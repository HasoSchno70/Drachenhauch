# Kapitel 10 — Vererbung: die Enemy-Hierarchie

Star Pilot hat einen Player und Bullets. Was fehlt? Etwas, worauf man schießen kann. In diesem Kapitel kommen **Gegner** ins Spiel — und mit ihnen ein Sprach-Konzept, das du gerade brauchst: **Vererbung**.

Player, Grunt und Bomber haben alle dieselbe Grundstruktur — Position, ein „lebt-Flag", eine `Update`- und eine `Draw`-Methode. Wenn wir das in jeder Klasse einzeln schreiben, schreiben wir denselben Code dreimal. Vererbung ist die Antwort.

## Lernziele

Nach diesem Kapitel:

- definierst du Klassen, die mit `EXTENDS` von einer anderen erben
- legst du gemeinsame Felder und Methoden in einer **Basisklasse** ab
- überschreibst du Methoden in abgeleiteten Klassen
- erkennst du, wann Vererbung *passt* und wann sie *erzwungen* wirkt
- hast du in Star Pilot zwei Enemy-Typen — Grunt und Bomber

## Schritt 1: Was hätten Player, Grunt und Bomber gemeinsam?

Schau, was alle drei brauchen:

| | x, y | alive | Update | Draw |
|---|---|---|---|---|
| **Player** | ja | (im Spiel: lebt immer) | reagiert auf Tasten | Box gelb |
| **Grunt** | ja | ja | fliegt geradeaus runter | Box rot |
| **Bomber** | ja | ja | fliegt zickzack | Box orange |

Vier Felder/Methoden, alle drei haben sie. Das schreit nach **gemeinsamer Basisklasse**.

## Schritt 2: Die Basisklasse Entity

Wir definieren eine Klasse `Entity` — alles, was alle Spielobjekte teilen:

```basic
CLASS Entity
    DIM x      AS INTEGER
    DIM y      AS INTEGER
    DIM w      AS INTEGER
    DIM h      AS INTEGER
    DIM alive  AS BOOLEAN

    SUB Init(start_x AS INTEGER, start_y AS INTEGER)
        x = start_x
        y = start_y
        alive = TRUE
        w = 16
        h = 16
    END SUB

    SUB Update()
        ' Default: tu nichts
    END SUB

    SUB Draw()
        ' Default: zeichne nichts
    END SUB
END CLASS
```

Drei Sachen sind besonders:

- **`Update` und `Draw` haben leere Bodys**. Sie sind als „Platzhalter" gemeint — abgeleitete Klassen werden sie überschreiben. Manche Sprachen nennen das *abstrakte* oder *virtuelle* Methoden. GameBasic hat dafür kein eigenes Schlüsselwort; eine leere Methode reicht.
- **`Init` setzt einen Default für `w` und `h`**. Wer möchte, kann das in der abgeleiteten Klasse überschreiben (Player ist breiter als ein Grunt).
- Es gibt **kein** spezifisches `Draw` hier — die Basisklasse weiß nicht, welche Farbe oder Form eine Subklasse hat. Das überlässt sie dem Erben.

> **Begriff**: das nennt man **abstrakte Klasse** in der OOP-Theorie — eine Klasse, die nie direkt instanziiert wird, sondern nur als Grundlage für andere dient. Du würdest in der Praxis nie `NEW Entity(0, 0)` schreiben — das Resultat wäre ein nutzloses Objekt, das nichts tut.

## Schritt 3: Erben mit EXTENDS

```basic
CLASS Grunt EXTENDS Entity
    SUB Update()
        IF NOT alive THEN RETURN
        y = y + 2
        IF y > 240 THEN alive = FALSE
    END SUB
END CLASS
```

Drei Magie-Punkte:

1. **`EXTENDS Entity`** sagt: „Grunt ist ein Entity-mit-Erweiterungen".
2. **Felder sind geerbt**: `x`, `y`, `w`, `h`, `alive` sind in `Grunt.Update` ohne weiteres verfügbar — auch ohne dass wir sie nochmal in `Grunt` deklarieren.
3. **`Update` wird überschrieben**: die leere Default-Update aus `Entity` wird ersetzt durch unsere Grunt-Bewegung.

Der Aufruf:

```basic
DIM g AS Grunt
g = NEW Grunt(100, 0)        ' ruft Entity.Init auf (geerbt)
g.Update()                   ' ruft Grunt.Update auf (ueberschrieben)
PRINT g.alive                ' Feld aus Entity, geerbt
```

Init ist nicht überschrieben — wir benutzen die Init aus `Entity`. Update *ist* überschrieben — die Grunt-Variante läuft.

> **Stolperfalle: Update mit leerem Body in der Basisklasse muss da sein.** Hätten wir in `Entity` die `Update`-Methode komplett **weggelassen**, würde `g.Update()` einen Fehler werfen — `Grunt` hat ja nichts überschrieben, weil's nichts zu überschreiben gab. Praktische Faustregel: in der Basisklasse jede Methode aufführen, die abgeleitete Klassen vielleicht haben — auch leer.

## Schritt 4: Bomber mit eigenem Feld

Bomber bewegen sich zickzack — sie brauchen eine zusätzliche Komponente: die seitliche Richtung. Diese ist nur für Bomber relevant, nicht in der Basisklasse:

```basic
CLASS Bomber EXTENDS Entity
    DIM dx AS INTEGER     ' nur fuer Bomber

    SUB Init(start_x AS INTEGER, start_y AS INTEGER)
        x = start_x
        y = start_y
        alive = TRUE
        w = 22
        h = 18
        dx = 1
    END SUB

    SUB Update()
        IF NOT alive THEN RETURN
        y = y + 1
        x = x + dx
        IF x < 0 OR x > 320 - w THEN
            dx = -dx
        END IF
        IF y > 240 THEN alive = FALSE
    END SUB
END CLASS
```

Was wir hier sehen:

- **Eigenes Feld in der abgeleiteten Klasse**: `dx` gibt's nur in Bomber, nicht in Entity oder Grunt. Vollkommen okay — abgeleitete Klassen dürfen alles, was die Basis nicht hat, hinzufügen.
- **Init wird überschrieben**: weil Bomber andere `w`/`h`-Werte will und das `dx`-Feld initialisieren muss. Der Code wiederholt manche Zeilen aus `Entity.Init` (Position setzen, alive = TRUE) — das ist unschön, aber GameBasic hat (noch) keine Syntax für „rufe Eltern-Init auf". Wir leben mit der Wiederholung.

> **Was wenn ich Eltern-Init aufrufen will?** Aktuell muss man die Init-Logik in der abgeleiteten Klasse manuell wiederholen. In Sprachen wie Python würdest du `super().__init__(x, y)` schreiben — GameBasic hat das in dieser Version nicht. Wenn dich das stört, kannst du eine separate Methode `BaseInit(x, y)` in `Entity` anlegen und sie sowohl aus `Entity.Init` als auch aus `Bomber.Init` aufrufen. Für unser Projekt aber zu viel Aufwand.

## Schritt 5: Player erbt auch

Bislang war `Player` eine eigenständige Klasse. Lass uns auch sie zu einem Entity machen — das vereinheitlicht den Code:

```basic
CLASS Player EXTENDS Entity
    SUB Init(start_x AS INTEGER, start_y AS INTEGER)
        x = start_x
        y = start_y
        alive = TRUE
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
```

Player kennt seine eigene Größe (40×24, ein breiteres Schiff als die Gegner), reagiert auf Tasten in `Update` und zeichnet sich gelb.

## Schritt 6: Bullet — *nicht* von Entity erben

Hier ein wichtiger Punkt: nicht alles muss erben, nur weil es geht. **Bullet** ist *kein* Entity — auch wenn er eine Position hat:

- Bullets haben kein `w`/`h` im klassischen Sinne (sie sind 3×8, schmal und länglich)
- Bullets haben keinen `Spawn`-Lebenszyklus, der zu `Entity.Init` passen würde — sie werden im Pool wiederverwendet
- Bullets fliegen einfach, mit einer einzigen Methode

Wir lassen Bullet eine **eigenständige Klasse** ohne `EXTENDS`. Vererbung soll Code sparen, nicht eine Hierarchie aufbauen, „weil's modern wirkt".

> **Komposition vs Vererbung**: ein häufiges Diskussionsthema in OOP. Vererbung sagt „X ist ein Y" (Player **ist ein** Entity). Komposition sagt „X **hat ein** Y" (Player **hat eine** Position). Wenn die „ist ein"-Beziehung nicht natürlich passt, nimm Vererbung *nicht*. Faustregel: bei Zweifeln Komposition. Bei eindeutiger Spezialisierung Vererbung. Player und Grunt sind beide eindeutig „Entitäten im Spiel" — Vererbung passt. Bullet ist eher ein „Projektil-Effekt" — Komposition (oder eigenständige Klasse) reicht.

## Schritt 7: Pool für Enemies

Genau wie Bullets bekommen Enemies einen Pool. Pro Enemy-Typ ein eigenes Array:

```basic
DIM grunts[ENEMY_POOL]  AS Grunt
DIM bombers[ENEMY_POOL] AS Bomber
```

Das `Spawn`-Pattern aus Kap 9 wenden wir an. Statt einer eigenen `Spawn`-Methode auf jedem Enemy-Typ können wir auch hier eine pro Typ schreiben:

```basic
SUB Spawn(at_x AS INTEGER)
    x = at_x
    y = -16
    alive = TRUE
END SUB
```

Im Hauptcode finden wir den ersten freien Slot und rufen `.Spawn(at_x)` auf:

```basic
SUB SpawnGrunt(at_x AS INTEGER)
    DIM i AS INTEGER
    FOR i = 0 TO ENEMY_POOL - 1
        IF NOT grunts[i].alive THEN
            grunts[i].Spawn(at_x)
            RETURN
        END IF
    NEXT i
END SUB
```

Identisch wie bei Bullets — du erkennst das Pattern wieder.

## Schritt 8: Spawn-Timer und der Spielfluss

Jetzt brauchen wir einen Mechanismus, der **regelmäßig** Gegner spawnt. Eine Variable `spawn_timer`, die jeden Frame um 1 sinkt; bei 0 wird gespawnt und der Timer auf 60 gesetzt:

```basic
spawn_timer = spawn_timer - 1
IF spawn_timer <= 0 THEN
    spawn_timer = 60
    IF INT(RND() * 2) = 0 THEN
        SpawnGrunt(INT(RND() * (WIDTH - 16)))
    ELSE
        SpawnBomber(INT(RND() * (WIDTH - 22)))
    END IF
END IF
```

Bei 60 FPS heißt das: jede Sekunde ein neuer Gegner. `INT(RND() * 2)` ergibt 0 oder 1 — wir wechseln also zufällig zwischen Grunt und Bomber.

## Der vollständige Spielcode

Bringen wir alles zusammen. Vergleiche mit Kap 9 — jetzt mit `Entity`, `Grunt`, `Bomber` und dem Spawn-Timer:

```basic
CONST WIDTH        AS INTEGER = 320
CONST HEIGHT       AS INTEGER = 240
CONST BG_COLOR     AS INTEGER = &H141E3C
CONST PLAYER_C     AS INTEGER = &HFFDC00
CONST BULLET_C     AS INTEGER = &HFFFFFF
CONST GRUNT_C      AS INTEGER = &HFF6677
CONST BOMBER_C     AS INTEGER = &HFFAA33
CONST PLAYER_SPEED AS INTEGER = 3
CONST BULLET_SPEED AS INTEGER = 5
CONST BULLET_POOL  AS INTEGER = 20
CONST ENEMY_POOL   AS INTEGER = 15

FUNCTION Clamp(wert AS INTEGER, lo AS INTEGER, hi AS INTEGER) AS INTEGER
    IF wert < lo THEN RETURN lo
    IF wert > hi THEN RETURN hi
    RETURN wert
END FUNCTION

CLASS Entity
    DIM x      AS INTEGER
    DIM y      AS INTEGER
    DIM w      AS INTEGER
    DIM h      AS INTEGER
    DIM alive  AS BOOLEAN

    SUB Init(start_x AS INTEGER, start_y AS INTEGER)
        x = start_x
        y = start_y
        alive = TRUE
        w = 16
        h = 16
    END SUB

    SUB Update()
    END SUB

    SUB Draw()
    END SUB
END CLASS

' ... Player, Grunt, Bomber, Bullet wie oben gezeigt ...
```

Den vollständigen Code findest du in [`code/kap-10/main.gb`](code/kap-10/main.gb) — er ist zu lang, um ihn hier komplett abzudrucken, aber jedes Stück hast du in diesem Kapitel gesehen.

Run drücken. Du solltest jetzt sehen:

- **Gelber Player** unten, du bewegst ihn mit Pfeiltasten
- **Weiße Bullets**, die du mit Leertaste schießt
- **Rote Grunts**, die geradeaus von oben runterfallen
- **Orange Bomber**, die im Zickzack runterkommen

Was noch fehlt: die Bullets sollten die Enemies **treffen**. Das ist Kapitel 11.

## Übungen

**1. Drifter — der dritte Enemy-Typ.** Schreibe eine `CLASS Drifter EXTENDS Entity`, deren `Update`-Methode den Gegner *seitlich schaukelnd* runterfliegen lässt — z.B. `x = startposition + INT(SIN(y * 0.05) * 30)`. Die Sinuskurve ergibt die wellenartige Bewegung. (`SIN` und `COS` sind eingebaute Funktionen.)

**2. Hit-Points.** Erweitere `Entity` um ein Feld `hp AS INTEGER` (Default 1 in `Init`). Eine Methode `TakeDamage(n AS INTEGER)`, die `hp` reduziert und `alive = FALSE` setzt wenn `hp <= 0`. In Kap 11 nutzen wir das, um den Bomber zwei Treffer halten zu lassen.

**3. Bullet als Entity-Subklasse?** Versuche aus Bullet eine Klasse zu machen, die `Entity` erbt. Welche Probleme stellst du fest? Welche Lösungen findest du? (Tipp: was bedeutet `Init(start_x, start_y)` für ein Pool-Bullet, der erst mit `Spawn` aktiviert wird?)

**4. Stretch — Wave-Logik.** Statt fester Spawn-Rate von 1/Sekunde: ändere den Timer so, dass nach 10 Spawn-Events eine kurze Pause (3 Sekunden) kommt, und danach die Rate doppelt so hoch ist (Spawn alle 30 Frames = halbe Sekunde). Vorgriff auf Kap 12, wo wir das mit ENUM und einem richtigen Wave-Manager strukturieren.

## Zusammenfassung

Du hast in diesem Kapitel:

- die `EXTENDS`-Syntax für Vererbung kennengelernt,
- eine Basisklasse `Entity` mit gemeinsamen Feldern und Default-Methoden gebaut,
- Player, Grunt und Bomber als abgeleitete Klassen geschrieben,
- ein neues Feld `dx` an Bomber hinzugefügt — abgeleitete Klassen dürfen erweitern,
- erkannt, dass Bullet *nicht* in die Entity-Hierarchie gehört (Komposition statt Vererbung),
- den Spawn-Timer als einfachen Wave-Mechanismus eingebaut.

Im **nächsten Kapitel** schließen wir den Game-Loop kreislaufmäßig: Bullets treffen Enemies, Enemies treffen den Player, Score wächst, Lives sinken. Dafür bekommen wir das `physics`-Modul und damit echte Kollisions-Erkennung.

## Code-Stand am Ende des Kapitels

- [`code/kap-10/01_entity.gb`](code/kap-10/01_entity.gb) — Entity-Basisklasse mit erstem Grunt-Erben (Konsolen-Test)
- [`code/kap-10/02_grunt_bomber.gb`](code/kap-10/02_grunt_bomber.gb) — Grunt und Bomber mit unterschiedlicher Update-Logik
- [`code/kap-10/main.gb`](code/kap-10/main.gb) — Star Pilot mit Player + Bullets + zwei Enemy-Typen + Spawn-Timer
