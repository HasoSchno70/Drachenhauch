# Module `vec2`

Immutable 2D vector with operator overloading. Vec2 values are **immutable** — every operation creates a new Vec2. That makes code readable and bug-free: nobody accidentally mutates someone else's vector.

```basic
IMPORT "vec2"
```

## Overview

| Function | Returns | Effect |
|---|---|---|
| `VEC2_NEW(x, y)` | VEC2 | constructor |
| `VEC2_ZERO()` | VEC2 | (0, 0) |
| `VEC2_X(v)` / `VEC2_Y(v)` | FLOAT | read components |
| `VEC2_LENGTH(v)` | FLOAT | Euclidean length |
| `VEC2_LENGTH_SQ(v)` | FLOAT | squared length (saves the sqrt) |
| `VEC2_NORMALIZE(v)` | VEC2 | unit vector (NIL for a zero vector) |
| `VEC2_DOT(a, b)` | FLOAT | dot product |
| `VEC2_CROSS(a, b)` | FLOAT | 2D cross product (scalar) |
| `VEC2_DISTANCE(a, b)` | FLOAT | Euclidean distance |
| `VEC2_LERP(a, b, t)` | VEC2 | interpolation `a + (b - a) * t` |
| `VEC2_PERP(v)` | VEC2 | 90° counter-clockwise: `(-y, x)` |
| `VEC2_REFLECT(v, normal)` | VEC2 | reflection off a wall normal |
| `VEC2_ANGLE(v)` | FLOAT | angle in radians (from the +x axis) |
| `VEC2_FROM_ANGLE(angle, length)` | VEC2 | polar → Cartesian |

## Operator overloading

The arithmetic operators work directly on Vec2:

```basic
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(3.0, 4.0)
w = VEC2_NEW(1.0, 2.0)

PRINT v + w            ' Vec2(4.0, 6.0)
PRINT v - w            ' Vec2(2.0, 2.0)
PRINT v * 2.0          ' Vec2(6.0, 8.0)   scalar multiplication
PRINT 2.0 * v          ' Vec2(6.0, 8.0)   both orders work
PRINT v / 2.0          ' Vec2(1.5, 2.0)
PRINT v = w            ' FALSE
PRINT v <> w           ' TRUE
```

**Multiplication Vec2 * Vec2** is not defined — for the dot product use `VEC2_DOT(a, b)`, for component-wise multiplication `VEC2_NEW(VEC2_X(a) * VEC2_X(b), VEC2_Y(a) * VEC2_Y(b))`.

## Classic game patterns

**Player movement with velocity:**

```basic
DIM pos AS VEC2
DIM vel AS VEC2
pos = VEC2_NEW(100.0, 100.0)
vel = VEC2_NEW(0.0, 0.0)

' Per frame:
vel = vel * 0.95       ' Friction
pos = pos + vel        ' Movement
```

**Steering: direction towards a target:**

```basic
DIM target AS VEC2
DIM direction AS VEC2
target = VEC2_NEW(400.0, 300.0)
direction = VEC2_NORMALIZE(target - pos)
DIM speed AS FLOAT
speed = 3.0
vel = direction * speed
```

**Reflection off a wall:**

```basic
DIM wall_normal AS VEC2
wall_normal = VEC2_NEW(0.0, -1.0)    ' floor, normal points upwards
vel = VEC2_REFLECT(vel, wall_normal)
```

**Rotation by 90° (classic for the turret pattern):**

```basic
DIM forward AS VEC2
DIM right AS VEC2
forward = VEC2_FROM_ANGLE(angle, 1.0)
right = VEC2_PERP(forward)       ' 90° counter-clockwise
```

**Distance check (squared saves the sqrt):**

```basic
IF VEC2_LENGTH_SQ(target - pos) < 100.0 THEN     ' < 10 pixels
    PRINT "Ziel erreicht!"
END IF
```

## Immutability

Vec2 is **immutable**: after `w = v`, `w` and `v` do point to the same object, but every operation creates a new Vec2. So there is no way to change anything in `v` through `w`.

```basic
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(1.0, 2.0)
w = v                       ' "alias" -- but irrelevant, because immutable
v = v + VEC2_NEW(10.0, 0.0) ' v is now a NEW Vec2
PRINT w                     ' Vec2(1.0, 2.0) -- unchanged
PRINT v                     ' Vec2(11.0, 2.0)
```

## External type

| Type | Effect |
|---|---|
| `VEC2` | immutable 2D vector. `DIM v AS VEC2` |

## Example

[examples/58_vec2.dh](../../examples/58_vec2.dh) shows the full API including the operator overloads in a small demo (player movement, distance check, reflection).
