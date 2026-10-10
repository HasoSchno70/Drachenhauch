# Module `particles`

Particle systems: sparks, smoke, explosions, trails. Configurable in velocity, lifetime, gravity, colour and size.

```basic
IMPORT "particles"
```

## Overview

| Function | Purpose |
|---|---|
| `PARTICLE_SYSTEM_NEW(x, y)` → PARTICLE_SYSTEM | new system at (x, y) |
| `PARTICLE_SET_POS(sys, x, y)` | move the emitter |
| `PARTICLE_SET_VELOCITY(sys, vx_min, vx_max, vy_min, vy_max)` | spread of the start velocities (pixels/s) |
| `PARTICLE_SET_LIFETIME(sys, ms_min, ms_max)` | lifetime spread in ms |
| `PARTICLE_SET_GRAVITY(sys, gx, gy)` | constant acceleration (pixels/s²) |
| `PARTICLE_SET_COLOR(sys, color)` | colour (24-bit RGB) |
| `PARTICLE_SET_SIZE(sys, px_min, px_max)` | pixel size |
| `PARTICLE_SET_FADE(sys, fade)` | TRUE: brightness decreases with age |
| `PARTICLE_EMIT(sys, count)` | emit n particles immediately |
| `PARTICLE_UPDATE(sys, dt_ms)` | advance the physics |
| `PARTICLE_DRAW(sys)` | draw (camera-aware) |
| `PARTICLE_COUNT(sys)` → INTEGER | number of living particles |
| `PARTICLE_CLEAR(sys)` | delete all |
| `PARTICLE_SET_MODE(sys, modus$)` | appearance of the particles: `circle`, `pixel`, `square`, `streak` or `glow` |
| `PARTICLE_SET_COLOR_END(sys, farbe)` | colour at the end of the lifetime — the particles change colour on the way (fire: yellow to red) |

## Defaults

`PARTICLE_SYSTEM_NEW(x, y)` comes with sensible defaults:

- Velocity: `(-50..50, -100..-50)` — a light shower of sparks upwards
- Lifetime: `500..1000` ms
- Gravity: `(0, 200)` — they fall down
- Colour: white
- Size: `2..4` pixels
- Fade: TRUE

That means: just `PARTICLE_SYSTEM_NEW` + `PARTICLE_EMIT` is enough for visible sparks — configuration only comes in when you want something different.

## Standard game loop

```basic
IMPORT "particles"

SCREEN(320, 240, "Funken-Demo", 2)

DIM funken AS PARTICLE_SYSTEM
funken = PARTICLE_SYSTEM_NEW(160.0, 120.0)
PARTICLE_SET_COLOR(funken, RGB(255, 200, 80))    ' golden yellow

DIM last_ms AS INTEGER
last_ms = MILLIS()

WHILE NOT QUITREQUESTED()
    DIM now_ms AS INTEGER
    DIM dt AS INTEGER
    now_ms = MILLIS()
    dt = now_ms - last_ms
    last_ms = now_ms

    ' Mouse position as the emitter position
    PARTICLE_SET_POS(funken, MOUSEX() * 1.0, MOUSEY() * 1.0)

    ' While the mouse is held: strong jet, otherwise a gentle trail
    IF MOUSEBUTTON(0) THEN
        PARTICLE_EMIT(funken, 5)
    ELSE
        PARTICLE_EMIT(funken, 1)
    END IF

    PARTICLE_UPDATE(funken, dt)

    CLS(RGB(0, 0, 30))
    PARTICLE_DRAW(funken)
    TEXT(8, 8, "Partikel: " + STR$(PARTICLE_COUNT(funken)), RGB(200, 200, 200))
    FLIP()
    SLEEP(16)
WEND
```

## Effect recipes

**Sparks (impact, coin pickup):**

```basic
DIM s AS PARTICLE_SYSTEM
s = PARTICLE_SYSTEM_NEW(0.0, 0.0)
PARTICLE_SET_COLOR(s, RGB(255, 220, 80))
PARTICLE_SET_VELOCITY(s, -120.0, 120.0, -180.0, -40.0)   ' all-round spread
PARTICLE_SET_LIFETIME(s, 300, 600)
PARTICLE_SET_GRAVITY(s, 0.0, 400.0)
PARTICLE_SET_SIZE(s, 2, 4)

' On the trigger:
PARTICLE_SET_POS(s, treffer_x, treffer_y)
PARTICLE_EMIT(s, 25)
```

**Smoke (slow, weightless):**

```basic
DIM rauch AS PARTICLE_SYSTEM
rauch = PARTICLE_SYSTEM_NEW(0.0, 0.0)
PARTICLE_SET_COLOR(rauch, RGB(180, 180, 180))
PARTICLE_SET_VELOCITY(rauch, -8.0, 8.0, -30.0, -10.0)
PARTICLE_SET_LIFETIME(rauch, 1500, 2500)
PARTICLE_SET_GRAVITY(rauch, 0.0, 0.0)                    ' no falling
PARTICLE_SET_SIZE(rauch, 4, 8)
PARTICLE_SET_FADE(rauch, TRUE)
```

**Explosion (big, short):**

```basic
DIM bumm AS PARTICLE_SYSTEM
bumm = PARTICLE_SYSTEM_NEW(0.0, 0.0)
PARTICLE_SET_COLOR(bumm, RGB(255, 100, 30))
PARTICLE_SET_VELOCITY(bumm, -200.0, 200.0, -200.0, 200.0)   ' radial
PARTICLE_SET_LIFETIME(bumm, 200, 500)
PARTICLE_SET_GRAVITY(bumm, 0.0, 0.0)
PARTICLE_SET_SIZE(bumm, 3, 6)

' On an explosion:
PARTICLE_EMIT(bumm, 80)
```

**Rain (long, even):**

```basic
DIM regen AS PARTICLE_SYSTEM
regen = PARTICLE_SYSTEM_NEW(0.0, 0.0)
PARTICLE_SET_COLOR(regen, RGB(180, 200, 255))
PARTICLE_SET_VELOCITY(regen, -10.0, 10.0, 100.0, 200.0)
PARTICLE_SET_LIFETIME(regen, 1500, 2500)
PARTICLE_SET_GRAVITY(regen, 0.0, 50.0)                  ' slight extra acceleration
PARTICLE_SET_SIZE(regen, 1, 2)
PARTICLE_SET_FADE(regen, FALSE)

' In the loop: emit per frame, spread across the window width
PARTICLE_SET_POS(regen, RND(SCREEN_W) * 1.0, 0.0)
PARTICLE_EMIT(regen, 2)
```

## Velocity & gravity

- **Velocity** is pixels per second — `(-50, 50)` means: a random horizontal speed between -50 and 50 px/s.
- **Gravity** is pixels per second² — `(0, 200)` means: accelerating downwards, 200 px/s² (about 1/10 of Earth's gravity).
- A negative `vy` means "upwards" (in screen coordinates Y is positive downwards).

## Update with dt

`PARTICLE_UPDATE(sys, dt_ms)` must be called every frame — the particles age and move. `dt_ms` is the frame time (difference to the previous `MILLIS()`). **Milliseconds, not seconds:** `DELTA()` returns seconds -- use `PARTICLE_UPDATE(sys, INT(DELTA() * 1000))`. A number between 0 and 1 is an error that says exactly that (otherwise the particles would never age). The same holds for `SPRITE_UPDATE` and `ANIM_FSM_UPDATE`.

```basic
DIM now_ms AS INTEGER
DIM dt AS INTEGER
now_ms = MILLIS()
dt = now_ms - last_ms
last_ms = now_ms

PARTICLE_UPDATE(funken, dt)
```

Anyone who wants to pause the loop (e.g. a pause menu) simply does not call `PARTICLE_UPDATE` — the particles "freeze".

## Camera-aware

`PARTICLE_DRAW` draws via `CIRCLE` and therefore respects the camera (see [camera module](module-camera.md)). World coordinates in `PARTICLE_SET_POS` are enough — the camera takes care of the screen conversion.

## Performance

In `dhrt` the particles are a list of Rust values (position, velocity,
lifetime, age, size, colour). `PARTICLE_UPDATE` goes over all of them in ONE
built-in call: ageing, gravity, position, and every particle that has reached
its lifetime drops out -- no call per particle from the program.

`PARTICLE_EMIT` draws velocity, lifetime and size from the same random
generator as `RND`, so a run after `RANDOMIZE(seed)` is reproducible.

`PARTICLE_DRAW` computes each particle's colour from its age when fading or
using a colour gradient, then draws one circle (or the chosen shape) per
particle.

## Complete example

- [examples/140_particles.dh](../../examples/140_particles.dh) — console logic test with `RANDOMIZE(42)`
- [examples/28_particles_visual.dh](../../examples/28_particles_visual.dh) — interactive, sparks follow the mouse
