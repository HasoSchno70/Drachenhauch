# Module `sprite`

Animated sprites from sheets — with position, velocity, several named animations, flip, scaling, tinting, AABB collision.

```basic
IMPORT "sprite"
```

## Overview

| Function | Purpose |
|---|---|
| `SPRITE_NEW(image, fw, fh)` → SPRITE | from a sheet |
| `SPRITE_SET_POS(sp, x, y)` | position |
| `SPRITE_SET_VELOCITY(sp, vx, vy)` | pixels/second |
| `SPRITE_GET_X(sp)`, `SPRITE_GET_Y(sp)` → FLOAT | current position |
| `SPRITE_GET_WIDTH(sp)`, `SPRITE_GET_HEIGHT(sp)` → INTEGER | frame size |
| `SPRITE_ADD_ANIM(sp, name$, first, last, fps)` | register an animation |
| `SPRITE_PLAY(sp, name$)` | start an animation (looping) |
| `SPRITE_PLAY_ONCE(sp, name$)` | play an animation once |
| `SPRITE_CURRENT_ANIM(sp)` → STRING | current animation name |
| `SPRITE_IS_FINISHED(sp)` → BOOLEAN | has a PLAY_ONCE animation reached the end? |
| `SPRITE_SET_FRAME(sp, idx)` | set the frame manually |
| `SPRITE_GET_FRAME(sp)` → INTEGER | current frame |
| `SPRITE_SET_FLIP(sp, flipX, flipY)` | mirroring |
| `SPRITE_SET_SCALE(sp, sx, sy)` | scaling (purely visual) |
| `SPRITE_TINT(sp, color)` | colour tint (RGB multiply) |
| `SPRITE_TINT_CLEAR(sp)` | reset the tint |
| `SPRITE_UPDATE(sp, dt_ms)` | advance position + animation |
| `SPRITE_DRAW(sp)` | draw (camera-aware) |
| `SPRITE_COLLIDES(sp1, sp2)` → BOOLEAN | AABB of two sprites |
| `SPRITE_HIT_BOX(sp, x, y, w, h)` → BOOLEAN | AABB test of the sprite against a rectangle `(x, y, w, h)` |
| `SPRITE_HIT_POINT(sp, x, y)` → BOOLEAN | is the point `(x, y)` inside the sprite? (mouse click test) |
| `SPRITE_COLLIDE(a, b)` → BOOLEAN | do two sprites overlap? (rectangle against rectangle) |

## Sheet layout

A sprite sheet is an image with frames on a grid (all the same size). Frames are read left to right, then down a row:

```
Frame 0  Frame 1  Frame 2  Frame 3
Frame 4  Frame 5  Frame 6  Frame 7
```

Call: `SPRITE_NEW(image, frame_w, frame_h)` — `image` is the whole sheet, `frame_w` × `frame_h` the size of **one** frame.

E.g. `assets/hero_walk.png` (64×16) has 4 frames of 16×16 horizontally:

```basic
DIM sheet AS IMAGE
sheet = LOADIMAGE("assets/hero_walk.png")

DIM held AS SPRITE
held = SPRITE_NEW(sheet, 16, 16)
```

## Animations

Animations are named frame ranges with frames per second:

```basic
SPRITE_ADD_ANIM(held, "idle", 0, 0, 1.0)         ' only frame 0
SPRITE_ADD_ANIM(held, "walk", 0, 3, 8.0)         ' frames 0-3 at 8 fps
SPRITE_ADD_ANIM(held, "punch", 4, 7, 12.0)       ' if the sheet has 8 frames

SPRITE_PLAY(held, "walk")          ' looping
' or
SPRITE_PLAY_ONCE(held, "punch")    ' play once, stops at the last frame
```

`SPRITE_PLAY` is **idempotent**: if the animation is already running (same name + loop mode), nothing happens. That makes it safe to call in every frame:

```basic
' In the game loop, depending on movement:
IF velocity_aktiv THEN
    SPRITE_PLAY(held, "walk")
ELSE
    SPRITE_PLAY(held, "idle")
END IF
```

(If it triggered a reset every frame, the walk cycle would be stuck on frame 0. That is why it is idempotent.)

## Update + draw in the loop

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

    SPRITE_UPDATE(held, dt)        ' position + animation

    CLS(RGB(20, 30, 50))
    SPRITE_DRAW(held)              ' draw
    FLIP()
    SLEEP(16)
WEND
```

## Flip / Scale / Tint

They can be combined — all of them apply to every draw until they are reset.

**Flip:**

```basic
SPRITE_SET_FLIP(held, TRUE, FALSE)   ' mirrored horizontally
SPRITE_SET_FLIP(held, FALSE, TRUE)   ' mirrored vertically
SPRITE_SET_FLIP(held, FALSE, FALSE)  ' back
```

Classic: when moving left, `flip_x = TRUE`.

**Scale:**

```basic
SPRITE_SET_SCALE(held, 2.0, 2.0)     ' twice as large
SPRITE_SET_SCALE(held, 1.0, 1.0)     ' back
```

`SPRITE_SET_SCALE` is **purely visual** — `SPRITE_GET_WIDTH`, `SPRITE_GET_HEIGHT` and `SPRITE_COLLIDES` keep working with the original frame size. That is intended: scaling, e.g. for a pickup pop effect, should not change the collision box.

**Tint:**

```basic
SPRITE_TINT(held, RGB(255, 100, 100))   ' tinted red (e.g. when hit)
SPRITE_TINT(held, RGB(255, 255, 255))   ' = no-op (white tint = neutral)
SPRITE_TINT_CLEAR(held)                 ' reset the tint
```

The tint multiplies each pixel by the colour (RGB multiply). `RGB(255,255,255)` leaves it unchanged; `RGB(0,0,0)` makes it black.

## Collision

```basic
DIM coin AS SPRITE
coin = SPRITE_NEW(coin_img, 8, 8)
SPRITE_SET_POS(coin, 100.0, 100.0)

' In the game loop:
IF SPRITE_COLLIDES(held, coin) THEN
    PRINT "Eingesammelt!"
END IF
```

`SPRITE_COLLIDES` is an AABB test (axis-aligned bounding box). Touching does not count — edges lying exactly against each other yield FALSE.

## Pickup pop pattern (with tween)

When collecting a coin: a short flash + a size pop, then it disappears.

```basic
IMPORT "sprite"
IMPORT "tween"

' ... setup ...

DIM coin_popping[10] AS BOOLEAN
DIM coin_pickup[10] AS TWEEN

' On a hit:
SUB on_collect(idx AS INTEGER)
    coin_popping[idx] = TRUE
    coin_pickup[idx] = TWEEN_NEW(1.0, 2.2, 200, "out_quad")
    SPRITE_TINT(coins[idx], RGB(255, 255, 255))
END SUB

' In the update loop:
DIM i AS INTEGER
FOR i = 0 TO 9
    IF coin_popping[i] THEN
        DIM s AS FLOAT
        s = TWEEN_VALUE(coin_pickup[i])
        SPRITE_SET_SCALE(coins[i], s, s)
        IF TWEEN_DONE(coin_pickup[i]) THEN
            coin_popping[i] = FALSE
            ' coin now completely "gone" - do not draw it any more
        END IF
    END IF
NEXT
```

## Camera-aware

`SPRITE_DRAW` draws via `g.draw_image_part` (or `draw_image` when flip/scale/tint is active), which automatically respects the camera (see [camera module](module-camera.md)). The position setters use world coordinates — the camera takes care of the conversion.

## Complete example

- [examples/31_sprite.dh](../../examples/31_sprite.dh) — logic test without a graphics window: animation timing, PLAY_ONCE, velocity, collision
- [examples/143_sprite_visual.dh](../../examples/143_sprite_visual.dh) — interactive: hero on a chessboard, collects gold coins
- [examples/32_coinquest.dh](../../examples/32_coinquest.dh) — complete game with sprites, tween pop, particles

## Tips

- **One sheet region per animation**: idle (frame 0), walk (1-3), jump (4), punch (5-7) — cleanly separated.
- **fps = 0** is allowed — the animation then freezes (sensible for static "idle" states, but then a 1-frame animation is enough too).
- **Do not set the velocity anew every frame** if it stays constant — `SPRITE_SET_VELOCITY(s, 0.0, 0.0)` is fine, but every call does floating-point work. In performance-critical code only when something changes.
- **On pickup**: first set the animation/tint, then mark the sprite as "dead", keep drawing it until the animation is done — looks much rounder than disappearing instantly.
