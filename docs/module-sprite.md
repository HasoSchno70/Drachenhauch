# Modul `sprite`

Animierte Sprites aus Sheets — mit Position, Geschwindigkeit, mehreren benannten Animationen, Flip, Skalierung, Tinten, AABB-Kollision.

```basic
IMPORT "sprite"
```

## Übersicht

| Funktion | Zweck |
|---|---|
| `SPRITE_NEW(image, fw, fh)` → SPRITE | aus Sheet |
| `SPRITE_SET_POS(sp, x, y)` | Position |
| `SPRITE_SET_VELOCITY(sp, vx, vy)` | Pixel/Sekunde |
| `SPRITE_GET_X(sp)`, `SPRITE_GET_Y(sp)` → FLOAT | aktuelle Position |
| `SPRITE_GET_WIDTH(sp)`, `SPRITE_GET_HEIGHT(sp)` → INTEGER | Frame-Größe |
| `SPRITE_ADD_ANIM(sp, name$, first, last, fps)` | Animation registrieren |
| `SPRITE_PLAY(sp, name$)` | Animation starten (looping) |
| `SPRITE_PLAY_ONCE(sp, name$)` | Animation einmal abspielen |
| `SPRITE_CURRENT_ANIM(sp)` → STRING | aktueller Anim-Name |
| `SPRITE_IS_FINISHED(sp)` → BOOLEAN | bei PLAY_ONCE-Anim am Ende? |
| `SPRITE_SET_FRAME(sp, idx)` | Frame manuell setzen |
| `SPRITE_GET_FRAME(sp)` → INTEGER | aktueller Frame |
| `SPRITE_SET_FLIP(sp, flipX, flipY)` | Spiegelung |
| `SPRITE_SET_SCALE(sp, sx, sy)` | Skalierung (rein visuell) |
| `SPRITE_TINT(sp, color)` | Farb-Tint (RGB-Multi) |
| `SPRITE_TINT_CLEAR(sp)` | Tint zurück |
| `SPRITE_UPDATE(sp, dt_ms)` | Position + Animation fortschreiben |
| `SPRITE_DRAW(sp)` | zeichnen (camera-aware) |
| `SPRITE_COLLIDES(sp1, sp2)` → BOOLEAN | AABB zweier Sprites |
| `SPRITE_HIT_BOX(sp, x, y, w, h)` → BOOLEAN | AABB-Test des Sprites gegen ein Rechteck `(x, y, w, h)` |
| `SPRITE_HIT_POINT(sp, x, y)` → BOOLEAN | liegt der Punkt `(x, y)` im Sprite? (Maus-Klick-Test) |

## Sheet-Layout

Ein Sprite-Sheet ist ein Bild mit gerasterten Frames (alle gleich groß). Frames werden links-nach-rechts, dann Zeile runter ausgelesen:

```
Frame 0  Frame 1  Frame 2  Frame 3
Frame 4  Frame 5  Frame 6  Frame 7
```

Aufruf: `SPRITE_NEW(image, frame_w, frame_h)` — `image` ist das gesamte Sheet, `frame_w` × `frame_h` die Größe **eines** Frames.

Z.B. `assets/hero_walk.png` (64×16) hat 4 Frames à 16×16 horizontal:

```basic
DIM sheet AS IMAGE
sheet = LOADIMAGE("assets/hero_walk.png")

DIM held AS SPRITE
held = SPRITE_NEW(sheet, 16, 16)
```

## Animationen

Animationen sind benannte Frame-Bereiche mit Frames pro Sekunde:

```basic
SPRITE_ADD_ANIM(held, "idle", 0, 0, 1.0)         ' nur Frame 0
SPRITE_ADD_ANIM(held, "walk", 0, 3, 8.0)         ' Frames 0-3 mit 8 fps
SPRITE_ADD_ANIM(held, "punch", 4, 7, 12.0)       ' falls Sheet 8 Frames hat

SPRITE_PLAY(held, "walk")          ' looping
' oder
SPRITE_PLAY_ONCE(held, "punch")    ' einmal abspielen, bleibt am letzten Frame stehen
```

`SPRITE_PLAY` ist **idempotent**: wenn die Animation bereits läuft (gleicher Name + Loop-Modus), passiert nichts. Das macht sicheren Aufruf in jedem Frame:

```basic
' Im Game-Loop, je nach Bewegung:
IF velocity_aktiv THEN
    SPRITE_PLAY(held, "walk")
ELSE
    SPRITE_PLAY(held, "idle")
END IF
```

(Würde bei jedem Frame ein Reset auslösen, wäre der Walk-Cycle ständig auf Frame 0. Deshalb ist es idempotent.)

## Update + Draw im Loop

```basic
IMPORT "sprite"

SCREEN(320, 240, "Sprite-Demo", 2)

DIM held AS SPRITE
held = SPRITE_NEW(LOADIMAGE("assets/hero_walk.png"), 16, 16)
SPRITE_SET_POS(held, 100.0, 100.0)
SPRITE_ADD_ANIM(held, "idle", 0, 0, 1.0)
SPRITE_ADD_ANIM(held, "walk", 0, 3, 8.0)
SPRITE_PLAY(held, "idle")

DIM last_ms AS INTEGER
last_ms = MILLIS()

WHILE NOT QUITREQUESTED()
    DIM now_ms AS INTEGER
    DIM dt AS INTEGER
    now_ms = MILLIS()
    dt = now_ms - last_ms
    last_ms = now_ms

    DIM vx AS FLOAT
    DIM vy AS FLOAT
    vx = 0.0
    vy = 0.0
    IF KEYPRESSED(KEY_LEFT) THEN
        vx = -80.0
        SPRITE_SET_FLIP(held, TRUE, FALSE)
    END IF
    IF KEYPRESSED(KEY_RIGHT) THEN
        vx = 80.0
        SPRITE_SET_FLIP(held, FALSE, FALSE)
    END IF
    IF KEYPRESSED(KEY_UP) THEN
        vy = -80.0
    END IF
    IF KEYPRESSED(KEY_DOWN) THEN
        vy = 80.0
    END IF
    SPRITE_SET_VELOCITY(held, vx, vy)
    IF vx <> 0.0 OR vy <> 0.0 THEN
        SPRITE_PLAY(held, "walk")
    ELSE
        SPRITE_PLAY(held, "idle")
    END IF

    SPRITE_UPDATE(held, dt)        ' Position + Animation

    CLS(RGB(20, 30, 50))
    SPRITE_DRAW(held)              ' zeichnen
    FLIP()
    SLEEP(16)
WEND
```

## Flip / Scale / Tint

Können kombiniert werden — alle wirken auf jeden Draw, bis sie wieder zurückgesetzt werden.

**Flip:**

```basic
SPRITE_SET_FLIP(held, TRUE, FALSE)   ' horizontal gespiegelt
SPRITE_SET_FLIP(held, FALSE, TRUE)   ' vertikal gespiegelt
SPRITE_SET_FLIP(held, FALSE, FALSE)  ' zurueck
```

Klassisch: bei Bewegung nach links `flip_x = TRUE`.

**Scale:**

```basic
SPRITE_SET_SCALE(held, 2.0, 2.0)     ' doppelt so gross
SPRITE_SET_SCALE(held, 1.0, 1.0)     ' zurueck
```

`SPRITE_SET_SCALE` ist **rein visuell** — `SPRITE_GET_WIDTH`, `SPRITE_GET_HEIGHT` und `SPRITE_COLLIDES` arbeiten weiter mit der originalen Frame-Größe. Das ist gewollt: Skalieren z.B. für Pickup-Pop-Effekt sollte nicht die Kollisionsbox ändern.

**Tint:**

```basic
SPRITE_TINT(held, RGB(255, 100, 100))   ' rot getoent (z.B. wenn getroffen)
SPRITE_TINT(held, RGB(255, 255, 255))   ' = no-op (weiss tint = neutral)
SPRITE_TINT_CLEAR(held)                 ' Tint zurueck
```

Der Tint multipliziert pro Pixel mit der Farbe (RGB-Multi). `RGB(255,255,255)` lässt unverändert; `RGB(0,0,0)` macht schwarz.

## Kollision

```basic
DIM coin AS SPRITE
coin = SPRITE_NEW(coin_img, 8, 8)
SPRITE_SET_POS(coin, 100.0, 100.0)

' Im Game-Loop:
IF SPRITE_COLLIDES(held, coin) THEN
    PRINT "Eingesammelt!"
END IF
```

`SPRITE_COLLIDES` ist AABB-Test (axis-aligned bounding box). Berührungen zählen nicht — exakt aneinanderliegende Kanten ergeben FALSE.

## Pickup-Pop-Pattern (mit Tween)

Beim Sammeln einer Coin: kurzes Aufblitzen + Größen-Pop, dann verschwinden.

```basic
IMPORT "sprite"
IMPORT "tween"

' ... Setup ...

DIM coin_popping[10] AS BOOLEAN
DIM coin_pickup[10] AS TWEEN

' Beim Treffer:
SUB on_collect(idx AS INTEGER)
    coin_popping[idx] = TRUE
    coin_pickup[idx] = TWEEN_NEW(1.0, 2.2, 200, "out_quad")
    SPRITE_TINT(coins[idx], RGB(255, 255, 255))
END SUB

' Im Update-Loop:
DIM i AS INTEGER
FOR i = 0 TO 9
    IF coin_popping[i] THEN
        DIM s AS FLOAT
        s = TWEEN_VALUE(coin_pickup[i])
        SPRITE_SET_SCALE(coins[i], s, s)
        IF TWEEN_DONE(coin_pickup[i]) THEN
            coin_popping[i] = FALSE
            ' coin jetzt komplett "weg" - nicht mehr zeichnen
        END IF
    END IF
NEXT
```

## Camera-aware

`SPRITE_DRAW` zeichnet via `g.draw_image_part` (oder `draw_image` bei aktivem Flip/Scale/Tint), das respektiert automatisch die Camera (siehe [Camera-Modul](module-camera.md)). Position-Setter nutzen World-Koordinaten — Camera kümmert sich um Konvertierung.

## Komplettes Beispiel

- [examples/31_sprite.gb](../examples/31_sprite.gb) — Logik-Test ohne Grafik-Fenster: Animations-Timing, PLAY_ONCE, Velocity, Kollision
- [examples/31_sprite_visual.gb](../examples/31_sprite_visual.gb) — interaktiv: Held auf Schachbrett, sammelt Goldstücke
- [examples/32_coinquest.gb](../examples/32_coinquest.gb) — komplettes Spiel mit Sprites, Tween-Pop, Particles

## Tipps

- **Pro-Animation einen Sheet-Bereich**: idle (Frame 0), walk (1-3), jump (4), punch (5-7) — sauber getrennt.
- **fps = 0** ist erlaubt — die Animation friert dann ein (sinnvoll für statische "idle"-States, aber dann reicht auch eine 1-Frame-Anim).
- **Velocity nicht jeden Frame neu setzen** wenn sie konstant bleibt — `SPRITE_SET_VELOCITY(s, 0.0, 0.0)` ist okay aber jeder Aufruf macht Float-Arbeit. Bei Performance-kritischem Code nur wenn sich was ändert.
- **Bei Pickup**: erst Anim/Tint setzen, dann das Sprite "tot" markieren, im Drawing weiter zeichnen bis Animation done — sieht viel runder aus als sofortiges Verschwinden.
