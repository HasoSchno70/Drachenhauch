# Module `camera`

A global camera for the game world. It translates and zooms **all** drawing commands without you having to rewrite every single call.

```basic
IMPORT "camera"
```

## Overview

| Function | Purpose |
|---|---|
| `CAMERA_SET(x, y[, zoom[, rotation_deg]])` | camera position (world point for the top left of the screen), zoom and optionally rotation |
| `CAMERA_RESET()` | back to identity (0, 0, 1.0, 0°) |
| `CAMERA_SET_ROTATION(rotation_deg)` | set only the rotation (x/y/zoom unchanged) |
| `CAMERA_X()`, `CAMERA_Y()`, `CAMERA_ZOOM()`, `CAMERA_ROTATION()` → FLOAT | read the current values |
| `CAMERA_FOLLOW(target_x, target_y, screen_w, screen_h)` | centres the camera on the target (rotation stays unchanged) |
| `CAMERA_S2W_X(sx[, sy])`, `CAMERA_S2W_Y(sy[, sx])` → FLOAT | screen pixel to world coordinate (e.g. for a mouse click) |
| `CAMERA_SHAKE(staerke[, dauer_ms])` | screen shake: random camera jolting (world pixels), decays linearly over `dauer_ms` (default 300) — runs on its own, no per-frame code. `staerke = 0` stops immediately |

## Concept

As long as the camera is the identity (`(0, 0, 1.0)`), everything draws as usual. As soon as `CAMERA_SET` sets an offset or zoom, all coordinates in `BOX`, `CIRCLE`, `LINE`, `TEXT`, `DRAWIMAGE`, `DRAWTILEMAP`, `SPRITE_DRAW`, `PARTICLE_DRAW` count as **world coordinates** — the camera projects them onto the screen.

```basic
IMPORT "camera"

SCREEN(320, 240, "Camera-Demo", 2)

' World centre (240, 160) onto the screen centre
CAMERA_SET(240.0 - 160.0, 160.0 - 120.0)

CLS()
BOX(240, 160, 260, 180, RGB(255, 0, 0))   ' appears in the screen centre
FLIP()
SLEEP(2000)
```

## Standard pattern: camera follows the hero

```basic
IMPORT "camera"
IMPORT "sprite"

CONST W AS INTEGER = 320
CONST H AS INTEGER = 240
SCREEN(W, H, "Held-Folger", 2)

DIM held AS SPRITE
held = SPRITE_NEW(LOADIMAGE("assets/hero.png"), 16, 16)

WHILE NOT QUITREQUESTED()
    ' Input + update ...

    ' Camera centred on the hero
    CAMERA_FOLLOW(SPRITE_GET_X(held) + 8.0, SPRITE_GET_Y(held) + 8.0, W * 1.0, H * 1.0)

    CLS(RGB(20, 25, 40))

    ' Draw the world (in world coordinates)
    DIM r AS INTEGER
    DIM c AS INTEGER
    FOR r = 0 TO 9
        FOR c = 0 TO 14
            BOX(c * 32, r * 32, c * 32 + 31, r * 32 + 31, RGB(60, 50, 80))
        NEXT
    NEXT

    SPRITE_DRAW(held)

    ' HUD in screen space (camera temporarily off)
    CAMERA_RESET()
    TEXT(4, 4, "HP: 100", RGB(255, 255, 255))

    FLIP()
    SLEEP(16)
WEND
```

`CAMERA_FOLLOW(target_x, target_y, screen_w, screen_h)` sets the camera so that `(target_x, target_y)` lands in the centre of the screen. It leaves the zoom unchanged.

## Zoom

```basic
CAMERA_SET(0.0, 0.0, 2.0)        ' everything twice as large
CAMERA_SET(0.0, 0.0, 0.5)        ' everything half as large (more world visible)
```

- At `zoom = 2.0` **every** drawing — lines, boxes **and images** — is drawn doubled.
- At zoom != 1 images are drawn rescaled every frame; that costs some performance, but is usually fine.
- **Text is NOT zoomed** (only translated). Otherwise text would become blurry at zoom > 1. Anyone who wants large text either uses a larger font or renders it beforehand as an image.

## Rotation

```basic
CAMERA_SET_ROTATION(15.0)      ' camera rotated 15 degrees clockwise
CAMERA_SET(x, y, zoom, 15.0)   ' equivalent, set together with x/y/zoom
```

Rotates **only the position** of each draw around the screen centre (half
the logical `SCREEN` width/height) — positive values turn the camera
clockwise, which makes the world appear to rotate counter-clockwise
(standard camera convention). The point exactly in the centre of the screen
stays fixed, which is why `CAMERA_FOLLOW` works unchanged even with rotation
active.

**Important limitation:** Rotation turns **only positions**, not the outline
of the shapes themselves. A `BOX`/`RECT`/`CIRCLE` lands at the correctly
rotated spot but is still drawn axis-aligned/round — no tilted rectangle.
Images/sprites with their own rotation angle (`DRAWIMAGEROT`, `SPRITE_*`)
do not rotate along automatically either; anyone who wants, say, a vehicle
sprite to turn with the camera adds `CAMERA_ROTATION()` to its own rotation
angle themselves:

```basic
DRAWIMAGEROT(schiff_bild, schiff_x, schiff_y, schiff_winkel + CAMERA_ROTATION(), 1.0)
```

Text has the same limit as with zoom — `TEXT` is only translated, never
rotated (otherwise every line would be distorted).

### Mouse click with active rotation

As long as the rotation is 0, the one-argument form of `CAMERA_S2W_X`/
`CAMERA_S2W_Y` is enough as before. **As soon as there is rotation, the
inverse mixes x and y** — the one-argument form then ignores the rotation
(backwards compatible for old code), and you need the two-argument form with
both screen coordinates:

```basic
DIM mx AS FLOAT
DIM my AS FLOAT
mx = MOUSEX() * 1.0
my = MOUSEY() * 1.0
' Careful: the axis itself ALWAYS comes first -- so for S2W_Y it is (my, mx),
' not (mx, my).
DIM wx AS FLOAT
DIM wy AS FLOAT
wx = CAMERA_S2W_X(mx, my)
wy = CAMERA_S2W_Y(my, mx)
```

Complete example including values recomputed by hand:
[examples/29_camera.dh](../../examples/29_camera.dh) (section "Rotation").

## HUD in screen space

When the camera is active and you want to draw HUD text in screen coordinates:

```basic
' during the game frame:
CAMERA_FOLLOW(spieler_x, spieler_y, W, H)
CLS()
' ... draw the world (world coords) ...

CAMERA_RESET()
' ... draw the HUD (screen coords) ...
TEXT(4, 4, "Score: " + STR$(score), RGB(255, 255, 255))

FLIP()
```

`CAMERA_RESET()` is cheap — no need to worry about performance when calling it several times per frame.

## Mouse click → world position

When the user clicks somewhere and you want to know **which world coordinate** that is:

```basic
IF MOUSEBUTTON(0) THEN
    DIM wx AS FLOAT
    DIM wy AS FLOAT
    wx = CAMERA_S2W_X(MOUSEX() * 1.0)
    wy = CAMERA_S2W_Y(MOUSEY() * 1.0)
    PRINT "Klick in Welt: ", wx, wy
END IF
```

With an identity camera this simply returns the mouse position; with a translated or zoomed camera the correct world coordinate.

## Camera behaviour in BOX/CIRCLE/LINE

The sizes are scaled with the zoom:

| Call | at zoom 1.0 | at zoom 2.0 |
|---|---|---|
| `CIRCLE(100, 100, 10, …)` | circle radius 10 at (100, 100) | circle radius 20 at (200, 200) |
| `BOX(0, 0, 100, 100, …)` | 100×100 pixels | 200×200 pixels |
| `LINE(0, 0, 100, 0, …)` | 100 pixels long | 200 pixels long (line thickness stays 1) |

The camera understands "world coordinates" as the **top left**: `CAMERA_SET(x, y, ...)` means: the world point `(x, y)` lands at screen `(0, 0)`. That is consistent with the `DRAWTILEMAP(sx, sy)` conventions.

## Example: scrolling world editor

```basic
IMPORT "camera"

SCREEN(320, 240, "Scroller", 2)

DIM cam_x AS FLOAT
DIM cam_y AS FLOAT
DIM zoom AS FLOAT
cam_x = 0.0
cam_y = 0.0
zoom = 1.0

WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(KEY_LEFT) THEN
        cam_x = cam_x - 4.0
    END IF
    IF KEYPRESSED(KEY_RIGHT) THEN
        cam_x = cam_x + 4.0
    END IF
    IF KEYPRESSED(KEY_UP) THEN
        cam_y = cam_y - 4.0
    END IF
    IF KEYPRESSED(KEY_DOWN) THEN
        cam_y = cam_y + 4.0
    END IF
    IF KEYPRESSED(43) THEN     ' +
        zoom = zoom * 1.05
    END IF
    IF KEYPRESSED(45) THEN     ' -
        zoom = zoom / 1.05
        IF zoom < 0.2 THEN
            zoom = 0.2
        END IF
    END IF

    CAMERA_SET(cam_x, cam_y, zoom)

    CLS(RGB(20, 20, 35))

    ' large world
    DIM r AS INTEGER
    DIM c AS INTEGER
    FOR r = 0 TO 19
        FOR c = 0 TO 29
            DIM color AS INTEGER
            IF (r + c) MOD 2 = 0 THEN
                color = RGB(60, 80, 120)
            ELSE
                color = RGB(80, 120, 60)
            END IF
            BOX(c * 32, r * 32, c * 32 + 31, r * 32 + 31, color)
        NEXT
    NEXT

    ' HUD
    CAMERA_RESET()
    TEXT(4, 4, "cam=(" + STR$(ROUND(cam_x)) + "," + STR$(ROUND(cam_y)) + ") zoom=" + STR$(zoom), RGB(255, 255, 255))

    FLIP()
    SLEEP(16)
WEND
```

## Complete example

- [examples/29_camera.dh](../../examples/29_camera.dh) — logic test without a graphics window (S2W conversion, FOLLOW)
- [examples/141_camera_visual.dh](../../examples/141_camera_visual.dh) — interactive with arrow keys + zoom

## Tip: no camera push/pop

There is **no** camera stack (no `CAMERA_PUSH/POP`). If you want to draw a HUD over a moving background, the pattern is:

```basic
CAMERA_SET(welt_x, welt_y, zoom)
' draw the world ...

CAMERA_RESET()
' draw the HUD ...
```

Anyone who really needs nested cameras (e.g. a mini-map): read the current values with `CAMERA_X/Y/ZOOM()`, `CAMERA_SET` for the other camera, then switch back with `CAMERA_SET(alt_x, alt_y, alt_zoom)`.
