# Modul `controller`

Character-Controller fuer Platformer mit den drei "feel-good"-Tricks, die ein Spiel von "geht ja" zu "fuehlt sich gut an" bringen:

1. **Coyote-Time** — nach Klippen-Verlassen bleibt das Sprung-Recht noch ein paar Frames aktiv.
2. **Jump-Buffering** — Sprung-Press kurz vor dem Boden zaehlt beim Touchdown.
3. **Variable-Jump-Height** — Sprung-Loslassen schneidet die Aufwaerts-Bewegung (Mario-Style: tippen = klein, halten = gross).

Brauchen `tiled` (fuer die Map) und `tile_collide` (fuer Kollisions-Sweep).

```basic
IMPORT "tiled"
IMPORT "tile_collide"
IMPORT "controller"
```

## Übersicht

### Konstruktor + Lese-Accessor

| Funktion | Rueckgabe |
|---|---|
| `CHAR_NEW(x, y, w, h)` | CHAR_CONTROLLER |
| `CHAR_X(c)` / `CHAR_Y(c)` | FLOAT |
| `CHAR_W(c)` / `CHAR_H(c)` | FLOAT |
| `CHAR_VX(c)` / `CHAR_VY(c)` | FLOAT |
| `CHAR_ON_GROUND(c)` | BOOLEAN |
| `CHAR_ON_WALL_LEFT(c)` / `..._RIGHT(c)` | BOOLEAN |
| `CHAR_FACING(c)` | INTEGER (-1 = links, +1 = rechts) |

### Frame-Input + Update

| Funktion | Wirkung |
|---|---|
| `CHAR_SET_INPUT(c, axis_x, jump_pressed, jump_held)` | Frame-Input setzen |
| `CHAR_UPDATE(c, map, layer_idx)` | Frame ausfuehren: Velocity berechnen, kollidieren, Position updaten |

### Konfiguration

| Funktion | Default |
|---|---|
| `CHAR_SET_MOVE_SPEED(c, speed)` | 2.0 |
| `CHAR_SET_JUMP_VELOCITY(c, vy)` | 6.0 (immer absolut, intern negiert) |
| `CHAR_SET_GRAVITY(c, g)` | 0.25 |
| `CHAR_SET_MAX_FALL(c, max_vy)` | 7.0 |
| `CHAR_SET_COYOTE_TIME(c, frames)` | 6 |
| `CHAR_SET_JUMP_BUFFER(c, frames)` | 6 |
| `CHAR_SET_VARIABLE_JUMP(c, enabled)` | TRUE |
| `CHAR_SET_VARIABLE_JUMP_CUT(c, factor)` | 0.5 (= Mario) |

### Manuelle Position/Velocity (selten gebraucht)

| Funktion | Wirkung |
|---|---|
| `CHAR_SET_POS(c, x, y)` | Position teleport-setzen |
| `CHAR_SET_VX(c, v)` / `CHAR_SET_VY(c, v)` | Velocity setzen (z.B. Knockback) |

## Klassischer Game-Loop

```basic
IMPORT "tiled"
IMPORT "controller"
IMPORT "input"

INPUT_BIND("left",  KEY_LEFT,  KEY_A, JOY_DPAD_LEFT)
INPUT_BIND("right", KEY_RIGHT, KEY_D, JOY_DPAD_RIGHT)
INPUT_BIND("jump",  KEY_SPACE, KEY_W, KEY_UP, JOY_BUTTON_A)

DIM lvl AS TILED_MAP
lvl = TILED_LOAD("levels/level1.json")

DIM player AS CHAR_CONTROLLER
player = CHAR_NEW(50.0, 100.0, 12.0, 14.0)

WHILE NOT QUITREQUESTED()
    INPUT_UPDATE()
    CHAR_SET_INPUT(player,
                   INPUT_AXIS("left", "right"),
                   INPUT_PRESSED("jump"),
                   INPUT_HELD("jump"))
    CHAR_UPDATE(player, lvl, 0)

    CLS()
    ' ... Tiles zeichnen ...
    DRAWIMAGE(sprite, CHAR_X(player), CHAR_Y(player))
    FLIP()
WEND
```

Das sind die einzigen 3 Calls pro Frame: `INPUT_UPDATE` → `CHAR_SET_INPUT` → `CHAR_UPDATE`. Alles andere ist Setup + Render.

## Die drei feel-good-Patterns

### Coyote-Time

Der Spieler verlaesst eine Klippe, sein letzter Touch vom Boden ist ein paar Frames her -- aber er drueckt erst JETZT Sprung. In strenger Physik: "zu spaet". Mit Coyote-Time: "passt schon, vorgemerkt".

```
Frame 1: Boden unter Spieler. Coyote = 6.
Frame 2: Klippe ueberschritten. Coyote = 5. on_ground = FALSE.
Frame 3: Coyote = 4.
Frame 4: User drueckt JUMP! Coyote war noch > 0 -> Sprung erlaubt.
```

Default: 6 Frames (~100 ms bei 60 fps). Spielbarkeit zwischen "spuerbar gnaedig" und "feels cheaty". Wer eine sehr strenge Souls-like Plattform-Sektion will: `CHAR_SET_COYOTE_TIME(c, 0)`.

### Jump-Buffering

Der Spieler ist noch in der Luft, drueckt aber schon Sprung — den der Spielcharakter braeuchte erst beim Landen. Buffer merkt sich den Press:

```
Frame 10: User drueckt JUMP. on_ground = FALSE. Buffer = 6.
Frame 11: Fall, Buffer = 5.
Frame 12: Fall, Buffer = 4.
Frame 13: Landung! on_ground = TRUE. Buffer = 3 > 0 -> Sprung feuert sofort.
```

Default: 6 Frames. Resultat: drueckst du Sprung im Fall-Moment, springt der Charakter SOFORT wenn er landet — ohne dass du extrem genau gedrueckt haben musst.

### Variable-Jump-Height

Der "Mario"-Trick: kurzer Tap auf Sprung = kleiner Sprung; langes Halten = grosser Sprung. Mechanik:

- Beim ersten Frame des Sprungs: `vy = -jump_velocity` (volle Geschwindigkeit).
- Wenn der User Sprung **loslaesst** waehrend `vy < 0` (noch nach oben): `vy *= variable_jump_cut` (Default 0.5).

```
Frame 1: JUMP gedrueckt + gehalten. vy = -6.
Frame 2: Noch gehalten. vy = -5.75 (Gravitation).
Frame 3: User LOSGELASSEN. Cut: vy = -5.5 * 0.5 = -2.75. + Gravitation.
Frame 4: Fall...
```

Der Cut wirkt einmal pro Sprung. Wer dann wieder druckt: ist eine andere Aktion (egal, der Cut ist schon gewesen).

Wer den Cut nicht will (z.B. fuer Sonic-style fixe Sprung-Hoehe):
`CHAR_SET_VARIABLE_JUMP(c, FALSE)` oder `CHAR_SET_VARIABLE_JUMP_CUT(c, 1.0)`.

## Wand-Detection

`CHAR_ON_WALL_LEFT/RIGHT` ist eine 1-Pixel-Probe: gibt es nach 1px links/rechts vom Spieler einen Wand-Tile? Wird nur gesetzt wenn der Spieler **nicht** on_ground ist (Bodenkontakt = keine Wand-Klamme).

Use-Cases:
- **Wall-Jump**: bei `on_wall_left` + Sprung-Press: `vx += positiv`, `vy = -jump`.
- **Wall-Slide**: `IF on_wall_left OR on_wall_right THEN max_fall = 2.0 END IF` (langsamer Fall).

Diese sind v1 nicht in der Engine -- du baust sie als BASIC-Code mit den Flags. Beispiel:

```basic
IF CHAR_ON_WALL_LEFT(player) AND CHAR_VY(player) > 0.0 THEN
    CHAR_SET_VY(player, 1.0)        ' Wall-Slide
END IF

IF CHAR_ON_WALL_LEFT(player) AND INPUT_PRESSED("jump") THEN
    CHAR_SET_VX(player, 4.0)        ' weg von der Wand
    CHAR_SET_VY(player, -5.0)       ' nach oben
END IF
```

## Knockback / Damage / Effekte

`CHAR_SET_VX` und `CHAR_SET_VY` ueberschreiben direkt die Velocity. Klassisch fuer Treffer-Reaktionen:

```basic
IF PlayerHitByEnemy() THEN
    DIM dir AS INTEGER
    dir = -CHAR_FACING(player)             ' weg vom Feind
    CHAR_SET_VX(player, dir * 4.0)
    CHAR_SET_VY(player, -3.0)              ' kleiner Bump nach oben
    invuln_timer = 60
END IF
```

## Was v1 NICHT macht

- **Wall-Slide / Wall-Jump als built-in**: musst du selbst aus den Flags bauen (s.o.).
- **Slopes**: `tile_collide` macht nur axis-aligned-Boxes. Schraegen brauchen erweiterte Kollision -- separate Erweiterung, kein controller-Thema.
- **Double-Jump**: gleiche Story, baust du um den Buffer-Mechanismus herum mit eigenem Counter.
- **Pickups, Damage**: das ist Game-Logik, nicht Controller. controller liefert nur Bewegung + Status-Flags.

## Externer Typ

| Typ | Wirkung |
|---|---|
| `CHAR_CONTROLLER` | Stateful Character-Controller. `DIM p AS CHAR_CONTROLLER` |

## Beispiel

[examples/77_tiled_platformer.dh](../examples/77_tiled_platformer.dh) — voller Platformer mit Controller, Tile-Kollision, Pickups, Tastatur + Gamepad.

## In der nativen Runtime (dhrt)

`controller` laeuft nativ (immer dabei) und ist **bit-identisch** zu den Python-Pfaden — die komplette Platformer-Physik (Coyote-Time, Jump-Buffer, Variable-Jump) wurde 1:1 portiert (ueber eine 40-Frame-Simulation verifiziert).
