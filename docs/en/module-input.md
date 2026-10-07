# Module `input`

Action-based input mapping with edge detection. Instead of scattering key codes (`KEYPRESSED(1073741904)`) all over the game code, the `input` module groups keys into named **actions**. An action can be bound to several keys (WASD + arrow keys at the same time).

```basic
IMPORT "input"
```

## Overview

### Action mapping + edge detection (keyboard **and** gamepad)

| Function | Returns | Effect |
|---|---|---|
| `INPUT_BIND(action, key1, [key2], ...)` | — | action → key list (mixes keyboard + gamepad codes) |
| `INPUT_UNBIND(action)` | — | delete an action |
| `INPUT_RESET()` | — | clear all bindings + state |
| `INPUT_UPDATE()` | — | frame snapshot (required every frame, also polls gamepads) |
| `INPUT_HELD(action)` | BOOLEAN | currently pressed |
| `INPUT_PRESSED(action)` | BOOLEAN | edge: pressed RIGHT NOW |
| `INPUT_RELEASED(action)` | BOOLEAN | edge: just released |
| `INPUT_AXIS(neg, pos)` | INTEGER | `-1 / 0 / 1` as a virtual axis |
| `INPUT_BOUND(action)` | BOOLEAN | is the action registered? |

### Gamepad / joystick

| Function | Returns | Effect |
|---|---|---|
| `INPUT_JOY_COUNT()` | INTEGER | number of connected pads |
| `INPUT_JOY_NAME(slot)` | STRING | pad name (e.g. "Xbox Wireless Controller") |
| `INPUT_JOY_AXIS(slot, axis_name$)` | FLOAT | -1.0..+1.0 with dead zone |

**Gamepad constants** (case-insensitive, like `KEY_*`):

| Constant | Mapping (Xbox) |
|---|---|
| `JOY_BUTTON_A` | A |
| `JOY_BUTTON_B` | B |
| `JOY_BUTTON_X` | X |
| `JOY_BUTTON_Y` | Y |
| `JOY_BUTTON_LB` / `JOY_BUTTON_RB` | LB / RB (bumper) |
| `JOY_BUTTON_BACK` / `JOY_BUTTON_START` | Back/Select, Start/Menu |
| `JOY_BUTTON_LSTICK` / `JOY_BUTTON_RSTICK` | stick click (L3/R3) |
| `JOY_DPAD_UP` / `_DOWN` / `_LEFT` / `_RIGHT` | D-pad directions |

**Axis names** for `INPUT_JOY_AXIS`:

| Name | Effect |
|---|---|
| `"left_x"` / `"left_y"` | left stick (Y: up = -1, down = +1) |
| `"right_x"` / `"right_y"` | right stick |
| `"lt"` / `"rt"` | trigger (rest position 0 or -1, depending on the pad) |

## Concept

Three state concepts per action:

- **HELD** — the action is currently pressed (any of its keys). `TRUE` each frame as long as it is pressed.
- **PRESSED** — edge: the action was `FALSE` in the PREVIOUS frame, now `TRUE`. `TRUE` for exactly one frame. The classic choice for jump/shoot (one action per key press).
- **RELEASED** — edge: was `TRUE`, is now `FALSE`. Exactly one frame.

Edge detection needs the `INPUT_UPDATE()` call at the start of the frame — the module compares the current snapshot with the previous one. Without UPDATE, PRESSED/RELEASED stay `FALSE` forever.

## Classic game loop

```basic
IMPORT "input"

' Set up once before the loop -- keyboard AND gamepad in one action.
INPUT_BIND("move_left",  KEY_LEFT,  KEY_A, JOY_DPAD_LEFT)
INPUT_BIND("move_right", KEY_RIGHT, KEY_D, JOY_DPAD_RIGHT)
INPUT_BIND("jump",       KEY_SPACE, KEY_W, JOY_BUTTON_A, JOY_DPAD_UP)
INPUT_BIND("shoot",      KEY_X,     KEY_RETURN, JOY_BUTTON_X)
INPUT_BIND("quit",       KEY_ESCAPE, JOY_BUTTON_BACK)

DIM x AS INTEGER
x = 100

WHILE NOT QUITREQUESTED()
    INPUT_UPDATE()              ' snapshot for this frame

    IF INPUT_HELD("move_left")  THEN x = x - 2
    IF INPUT_HELD("move_right") THEN x = x + 2

    IF INPUT_PRESSED("jump")  THEN PRINT "JUMP!"
    IF INPUT_PRESSED("shoot") THEN PRINT "BANG!"
    IF INPUT_PRESSED("quit")  THEN BREAK

    CLS()
    BOX(x, 100, x + 20, 120, WHITE)
    FLIP()
WEND
```

## `INPUT_AXIS` — virtual axis

The classic pattern for movement: `axis = -1` if left is pressed, `+1` if right, `0` if nothing or both.

```basic
INPUT_BIND("move_left",  KEY_LEFT,  KEY_A)
INPUT_BIND("move_right", KEY_RIGHT, KEY_D)

DIM ax AS INTEGER
ax = INPUT_AXIS("move_left", "move_right")
x = x + ax * 2     ' negative if left, positive if right
```

The same for the Y axis:

```basic
DIM ay AS INTEGER
ay = INPUT_AXIS("move_up", "move_down")
```

## Re-binding (e.g. settings menu)

`INPUT_BIND` with the same action name **overwrites** the old key list. Ideal for a "reconfigure keys" menu:

```basic
' User clicks "change jump", presses a new key...
INPUT_BIND("jump", new_key_code)   ' old bindings lost
```

## Multi-key bindings

An action triggers as soon as **any** of its keys is pressed. That lets you bind WASD and the arrow keys in parallel, without the user having to switch:

```basic
INPUT_BIND("move_left",  KEY_LEFT, KEY_A, KEY_KP4, KEY_H)   ' 4 Bindings
' INPUT_HELD("move_left") -> TRUE as soon as one of them is pressed
```

## Action names

Are **case-insensitive** (compared in lower case internally). `INPUT_BIND("Jump", ...)` and `INPUT_HELD("jump")` refer to the same action.

## Test reset

`INPUT_RESET()` deletes all bindings + the prev/cur state. Tests call it between cases so that bindings do not persist.

## Analogue stick

`INPUT_JOY_AXIS(slot, axis$)` returns -1.0..+1.0 with a dead zone (default 0.15). The classic pattern for smooth movement with a pad:

```basic
DIM stick_x AS FLOAT
stick_x = INPUT_JOY_AXIS(0, "left_x")

' Stick + D-pad together: take the stronger input
DIM axis AS INTEGER
axis = INPUT_AXIS("move_left", "move_right")    ' from keyboard/D-pad
IF stick_x < -0.3 THEN axis = -1
IF stick_x >  0.3 THEN axis = 1
```

**Triggers** (`lt` / `rt`) have no dead zone -- depending on the pad they report 0..1 or -1..1. If you want shoot-while-holding-RT: simply `IF INPUT_JOY_AXIS(0, "rt") > 0.5 THEN Shoot() END IF`.

## Multiplayer

V1: all connected pads feed into the same action. If you want players separated, build it at the application level: poll the pads individually via `INPUT_JOY_AXIS` and read buttons directly (or via additional `JOY_BUTTON_PAD_N` codes — that is an extension for later).

## Example

- [examples/59_input.dh](../../examples/59_input.dh) — keyboard pattern with multi-bind, axis, edge detection.
- [examples/77_tiled_platformer.dh](../../examples/77_tiled_platformer.dh) — platformer with keyboard + gamepad (stick + D-pad + A to jump).
