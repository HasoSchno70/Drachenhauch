# Module `controller`

A character controller for platformers with the three "feel-good" tricks that take a game from "it works" to "it feels good":

1. **Coyote time** — after leaving a ledge, the right to jump stays active for a few more frames.
2. **Jump buffering** — a jump press shortly before touching the ground counts on touchdown.
3. **Variable jump height** — releasing jump cuts the upward movement (Mario style: tap = small, hold = big).

It needs `tiled` (for the map) and `tile_collide` (for the collision sweep).

```basic
IMPORT "tiled"
IMPORT "tile_collide"
IMPORT "controller"
```

## Overview

### Constructor + read accessors

| Function | Returns | Meaning |
|---|---|---|
| `CHAR_NEW(x, y, w, h)` | CHAR_CONTROLLER | create a controller -- (x,y) top-left corner, (w,h) size in pixels |
| `CHAR_X(c)` / `CHAR_Y(c)` | FLOAT | current position |
| `CHAR_W(c)` / `CHAR_H(c)` | FLOAT | size of the collision box |
| `CHAR_VX(c)` / `CHAR_VY(c)` | FLOAT | current velocity per axis |
| `CHAR_ON_GROUND(c)` | BOOLEAN | is the character standing on solid ground? |
| `CHAR_ON_WALL_LEFT(c)` / `CHAR_ON_WALL_RIGHT(c)` | BOOLEAN | is it touching a wall on the left or right? |
| `CHAR_FACING(c)` | INTEGER (-1 = left, +1 = right) | facing direction -- for mirroring the sprite |

### Per-frame input + update

| Function | Effect |
|---|---|
| `CHAR_SET_INPUT(c, axis_x, jump_pressed, jump_held)` | set this frame's input |
| `CHAR_UPDATE(c, map, layer_idx)` | run one frame: compute velocity, collide, update position |

### Configuration

| Function | Default | Meaning |
|---|---|---|
| `CHAR_SET_MOVE_SPEED(c, speed)` | 2.0 | horizontal walking speed (pixels per frame) |
| `CHAR_SET_JUMP_VELOCITY(c, vy)` | 6.0 (always absolute, negated internally) | jump strength |
| `CHAR_SET_GRAVITY(c, g)` | 0.25 | gravity per frame |
| `CHAR_SET_MAX_FALL(c, max_vy)` | 7.0 | maximum falling speed -- without it a character falls through thin floors |
| `CHAR_SET_COYOTE_TIME(c, frames)` | 6 | for this many frames after leaving the edge a jump still counts |
| `CHAR_SET_JUMP_BUFFER(c, frames)` | 6 | pressed this many frames before landing, the jump already counts |
| `CHAR_SET_VARIABLE_JUMP(c, enabled)` | TRUE | shorter jump on a short key press |
| `CHAR_SET_VARIABLE_JUMP_CUT(c, factor)` | 0.5 (= Mario) | the fraction the rising speed drops to on release |

### Manual position/velocity (rarely needed)

| Function | Effect |
|---|---|
| `CHAR_SET_POS(c, x, y)` | teleport to a position |
| `CHAR_SET_VX(c, v)` / `CHAR_SET_VY(c, v)` | set the velocity (e.g. knockback) |

## Classic game loop

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
    ' ... draw the tiles ...
    DRAWIMAGE(sprite, CHAR_X(player), CHAR_Y(player))
    FLIP()
WEND
```

Those are the only 3 calls per frame: `INPUT_UPDATE` → `CHAR_SET_INPUT` → `CHAR_UPDATE`. Everything else is setup + rendering.

## The three feel-good patterns

### Coyote time

The player leaves a ledge, their last contact with the ground was a few frames ago -- but they only press jump NOW. In strict physics: "too late". With coyote time: "fine, noted".

```
Frame 1: Boden unter Spieler. Coyote = 6.
Frame 2: Klippe ueberschritten. Coyote = 5. on_ground = FALSE.
Frame 3: Coyote = 4.
Frame 4: User drueckt JUMP! Coyote war noch > 0 -> Sprung erlaubt.
```

Default: 6 frames (~100 ms at 60 fps). Playability between "noticeably forgiving" and "feels cheaty". If you want a very strict Souls-like platforming section: `CHAR_SET_COYOTE_TIME(c, 0)`.

### Jump buffering

The player is still in the air but already presses jump — which the character would only need on landing. The buffer remembers the press:

```
Frame 10: User drueckt JUMP. on_ground = FALSE. Buffer = 6.
Frame 11: Fall, Buffer = 5.
Frame 12: Fall, Buffer = 4.
Frame 13: Landung! on_ground = TRUE. Buffer = 3 > 0 -> Sprung feuert sofort.
```

Default: 6 frames. The result: if you press jump while falling, the character jumps IMMEDIATELY on landing — without you having to press with extreme precision.

### Variable jump height

The "Mario" trick: a short tap on jump = small jump; holding it long = big jump. Mechanics:

- On the first frame of the jump: `vy = -jump_velocity` (full speed).
- If the user **releases** jump while `vy < 0` (still going up): `vy *= variable_jump_cut` (default 0.5).

```
Frame 1: JUMP gedrueckt + gehalten. vy = -6.
Frame 2: Noch gehalten. vy = -5.75 (Gravitation).
Frame 3: User LOSGELASSEN. Cut: vy = -5.5 * 0.5 = -2.75. + Gravitation.
Frame 4: Fall...
```

The cut applies once per jump. Pressing again afterwards is a different action (doesn't matter, the cut has already happened).

If you don't want the cut (e.g. for a Sonic-style fixed jump height):
`CHAR_SET_VARIABLE_JUMP(c, FALSE)` or `CHAR_SET_VARIABLE_JUMP_CUT(c, 1.0)`.

## Wall detection

`CHAR_ON_WALL_LEFT/RIGHT` is a 1-pixel probe: is there a wall tile 1px to the left/right of the player? It is only set when the player is **not** on_ground (ground contact = no clinging to walls).

Use cases:
- **Wall jump**: on `on_wall_left` + a jump press: `vx += positive`, `vy = -jump`.
- **Wall slide**: `IF on_wall_left OR on_wall_right THEN max_fall = 2.0 END IF` (slower fall).

These are not in the engine in v1 -- you build them as BASIC code from the flags. Example:

```basic
IF CHAR_ON_WALL_LEFT(player) AND CHAR_VY(player) > 0.0 THEN
    CHAR_SET_VY(player, 1.0)        ' wall slide
END IF

IF CHAR_ON_WALL_LEFT(player) AND INPUT_PRESSED("jump") THEN
    CHAR_SET_VX(player, 4.0)        ' away from the wall
    CHAR_SET_VY(player, -5.0)       ' upwards
END IF
```

## Knockback / damage / effects

`CHAR_SET_VX` and `CHAR_SET_VY` overwrite the velocity directly. The classic use is hit reactions:

```basic
IF PlayerHitByEnemy() THEN
    DIM dir AS INTEGER
    dir = -CHAR_FACING(player)             ' away from the enemy
    CHAR_SET_VX(player, dir * 4.0)
    CHAR_SET_VY(player, -3.0)              ' small bump upwards
    invuln_timer = 60
END IF
```

## What v1 does NOT do

- **Wall slide / wall jump as a built-in**: you build them yourself from the flags (see above).
- **Slopes**: `tile_collide` only handles axis-aligned boxes. Slopes need extended collision -- a separate extension, not a controller topic.
- **Double jump**: same story, you build it around the buffer mechanism with your own counter.
- **Pickups, damage**: that is game logic, not the controller. controller only delivers movement + status flags.

## External type

| Type | Effect |
|---|---|
| `CHAR_CONTROLLER` | stateful character controller. `DIM p AS CHAR_CONTROLLER` |

## Example

[examples/77_tiled_platformer.dh](../../examples/77_tiled_platformer.dh) — a full platformer with the controller, tile collision, pickups, keyboard + gamepad.

## In the native runtime (dhrt)

`controller` runs natively (always included) and is **bit-identical** to the Python paths — the complete platformer physics (coyote time, jump buffer, variable jump) was ported 1:1 (verified over a 40-frame simulation).
