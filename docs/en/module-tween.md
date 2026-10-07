# Module `tween`

Value interpolation over time — for animations, smooth movement, fade effects. 13 different easing functions.

```basic
IMPORT "tween"
```

## Overview

| Function | Return value | Meaning |
|---|---|---|
| `TWEEN_NEW(start, end, dauer_ms[, easing$])` | TWEEN (one-shot) | move a value once from `start` to `end` |
| `TWEEN_NEW_LOOP(start, end, dauer_ms[, easing$])` | TWEEN (forever, jumps back) | like TWEEN_NEW, jumps back to the start at the end |
| `TWEEN_NEW_PINGPONG(start, end, dauer_ms[, easing$])` | TWEEN (forever, back & forth) | like TWEEN_NEW, runs backwards at the end |
| `TWEEN_VALUE(t)` | FLOAT | current value -- what you draw |
| `TWEEN_PROGRESS(t)` | FLOAT (0.0 .. 1.0) | how far along is it? (0 = start, 1 = end) |
| `TWEEN_DONE(t)` | BOOLEAN (always FALSE for loop/pingpong) | has it finished? |
| `TWEEN_RESTART(t)` | — | start over |
| `TWEEN_PAUSE(t)`, `TWEEN_RESUME(t)` | — | stop and let it run again |
| `TWEEN_REVERSE(t)` | — | reverse the direction |
| `TWEEN_EASINGS()` | STRING (comma-separated list) | which easing curves are there? |

## Concept

A tween is a value that moves automatically between two end points over a given time. You create it once with `TWEEN_NEW` and query it every frame with `TWEEN_VALUE`.

```basic
DIM t AS TWEEN
t = TWEEN_NEW(0.0, 100.0, 2000, "out_quad")    ' 0 -> 100 in 2000ms

' In the game loop:
DIM x AS FLOAT
x = TWEEN_VALUE(t)
BOX(INT(x), 100, INT(x) + 20, 120, RGB(255, 200, 0))
```

After 2000 ms `TWEEN_VALUE(t)` stays fixed at `100.0`, and `TWEEN_DONE(t)` is TRUE.

## Animation modes: one-shot / loop / pingpong

`TWEEN_NEW` is **one-shot** — it runs once from `start` to `end` and then stays at the end value.

For **continuous background animations** there are two endless variants:

| Built-in | Behaviour | Use case |
|---|---|---|
| `TWEEN_NEW(start, end, ms)` | once, clamps at the end | pop-in, fade-out, transitions |
| `TWEEN_NEW_LOOP(start, end, ms)` | repeats (start → end → start → end …) | conveyor stripes, rotating spinners, BPM-pulsing UI |
| `TWEEN_NEW_PINGPONG(start, end, ms)` | back & forth (start → end → start → end → start …) | idle bobs, breathing scale, swing effects |

Loop and pingpong tweens are **never** "done" — `TWEEN_DONE(t)` always returns `FALSE` for them. They run forever until you pause them.

```basic
' coin floats 4 pixels up and down, 800 ms per half wave
DIM bob AS TWEEN
bob = TWEEN_NEW_PINGPONG(-4.0, 4.0, 800, "inout_sine")

WHILE NOT QUITREQUESTED()
    DIM offset AS FLOAT
    offset = TWEEN_VALUE(bob)
    SPRITE_SET_POS(coin, 100, 60 + offset)
    SPRITE_DRAW(coin)
    FLIP()
    SLEEP(16)
WEND
```

```basic
' loading spinner: 0..360 degrees in 1 second, then back to 0 and again
DIM angle AS TWEEN
angle = TWEEN_NEW_LOOP(0.0, 360.0, 1000, "linear")
```

Want to combine a pop-in **plus** a continuous idle animation? Two tweens in parallel: a one-shot for the spawn, a pingpong for the bob afterwards. The visual result in the frame is `pop_in_value × bob_offset` (scale) or `base + bob_offset` (position).

## Easing functions

Easing determines the "acceleration curve" between the end points.

| Name | Effect |
|---|---|
| `linear` | constant speed (default) |
| `in_quad`, `out_quad`, `inout_quad` | quadratic (gently in/out) |
| `in_cubic`, `out_cubic`, `inout_cubic` | cubic (stronger) |
| `in_sine`, `out_sine`, `inout_sine` | sine (gentle) |
| `in_bounce`, `out_bounce`, `inout_bounce` | bouncing like a ball |
| `in_elastic`, `out_elastic`, `inout_elastic` | overshooting like a spring |
| `in_back`, `out_back`, `inout_back` | slight overshoot (pop effect) |

The complete list is available programmatically via `TWEEN_EASINGS()`.

`in_*`: start slowly, end fast. `out_*`: start fast, end gently. `inout_*`: both.

```basic
' coin banner flies in from the left
DIM banner AS TWEEN
banner = TWEEN_NEW(-200.0, 80.0, 600, "out_quad")

' power-up "pops" in
DIM popup AS TWEEN
popup = TWEEN_NEW(0.0, 1.0, 400, "out_bounce")

' button "wobbles" on click
DIM wackel AS TWEEN
wackel = TWEEN_NEW(0.0, 1.0, 600, "out_elastic")
```

## Pause / resume / restart

```basic
' pause while the game is paused
TWEEN_PAUSE(t)
' ... pause menu ...
TWEEN_RESUME(t)

' restart the animation (e.g. when the user picks up the same power-up again)
TWEEN_RESTART(popup)

' swap end points + restart (e.g. banner flies away to the left again)
TWEEN_REVERSE(banner)
```

## Example: banner slide-in

```basic
IMPORT "tween"

SCREEN(320, 240, "Tween-Demo", 2)

DIM banner AS TWEEN
banner = TWEEN_NEW(-200.0, 80.0, 700, "out_quad")

WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 20, 30))

    DIM y AS INTEGER
    y = ROUND(TWEEN_VALUE(banner))
    BOX(20, y, 300, y + 50, RGB(0, 0, 0))
    RECT(20, y, 300, y + 50, RGB(255, 220, 80))
    TEXT(40, y + 18, "GAME OVER", RGB(255, 255, 255))

    FLIP()
    SLEEP(16)
WEND
```

## Example: HUD score pop

On every pickup the score text should briefly "pop":

```basic
IMPORT "tween"

DIM score_pop AS TWEEN
score_pop = TWEEN_NEW(1.0, 1.0, 0, "linear")        ' initially: nothing to animate

' On pickup:
SUB on_pickup()
    score_pop = TWEEN_NEW(1.6, 1.0, 350, "out_quad")    ' 1.6 -> 1.0 in 350ms
END SUB

' In the draw loop:
DIM scale AS FLOAT
scale = TWEEN_VALUE(score_pop)
IF scale > 1.05 THEN
    TEXT(80, 8, "+100", RGB(255, 220, 80))             ' "pop" indicator
END IF
```

## Example: coin spawn pulsar

So that coins "grow" when they appear:

```basic
IMPORT "tween"

DIM coin_pop[10] AS TWEEN
DIM i AS INTEGER
FOR i = 0 TO 9
    coin_pop[i] = TWEEN_NEW(0.0, 1.0, 300 + RND(200), "out_bounce")
NEXT

' When drawing: only visible once the pulsar is "done"
FOR i = 0 TO 9
    IF TWEEN_VALUE(coin_pop[i]) > 0.5 THEN
        DRAWIMAGE(coin_img, coin_x[i], coin_y[i])
    END IF
NEXT
```

## Complete example

See [examples/26_tween.dh](../../examples/26_tween.dh) — shows linear, out_bounce, pause/resume and reverse using console output.

In the game ([examples/32_coinquest.dh](../../examples/32_coinquest.dh)) tweens are combined for banner slide, coin spawn and pickup pop.

## There is no `TWEEN_UPDATE`

This is deliberate, not a gap. `timer`, `input` and `gui` all require
a `..._UPDATE()` call per frame — `tween` does not:

```basic
DIM t AS TWEEN
t = TWEEN_NEW(0, 100, 500)          ' from 0 to 100 in 500 ms

WHILE NOT QUITREQUESTED()
    x = TWEEN_VALUE(t)              ' done -- no update needed
    ...
WEND
```

A tween computes its value from the clock on **every query**
(`MILLIS()` since program start). So it keeps running whether you query it
or not — and it runs in **real time**, independent of the frame rate.

Two consequences you should know:

- **If the frame rate drops, the tween jumps** instead of slowing down.
  For UI animation that is exactly right; if you tie a game mechanic to a
  tween, a stutter gives you a jump.
- **A tween is not reproducible.** With recorded input
  (`AUTOMATION_PLAY`) it keeps running by the wall clock, not by the
  frames — so two runs do not look exactly the same. `timer` is
  frame-driven and therefore reproducible; that is the difference between
  the two modules.

You can still pause: `TWEEN_PAUSE` / `TWEEN_RESUME` freeze the
elapsed time.

## Tips

- **Choose the right easing**: `out_quad` for "arriving gently", `out_bounce` for an "eye-catching pop", `linear` when constancy matters.
- **Tween duration in ms**: 100-300 ms feels fast, 500-1000 ms calm, > 2000 ms can feel sluggish.
- **Several tweens in parallel**: simply several variables, each with its own tween. They are independent of each other.
- **No `dt` argument**: tweens use `MILLIS()` internally, you do not have to pass a frame time — this simplifies the loop code.
- **Reset on re-trigger**: if an effect can be triggered several times (e.g. score pop), create the tween anew each time with `TWEEN_NEW(...)`.
