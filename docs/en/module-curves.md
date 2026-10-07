# Module `curves`

Animation curves: Bezier, Catmull-Rom, Hermite, smoothstep, lerp. Pure functions — no state, no setup. Complementary to the [`tween`](module-tween.md) module, which animates values over time; `curves` computes individual points on a curve.

```basic
IMPORT "curves"
```

## Overview

| Function | Returns | Purpose |
|---|---|---|
| `CURVE_LERP(a, b, t)` | FLOAT | linear interpolation `a + (b - a) * t` |
| `CURVE_SMOOTHSTEP(e0, e1, x)` | FLOAT | S-curve (`3t² - 2t³`), clamped to `[0, 1]` |
| `CURVE_SMOOTHERSTEP(e0, e1, x)` | FLOAT | C²-continuous S-curve (`6t⁵ - 15t⁴ + 10t³`) |
| `CURVE_BEZIER(t, p0, p1, p2, p3)` | FLOAT | cubic Bezier 1D |
| `CURVE_BEZIER2(t, x0,y0, x1,y1, x2,y2, x3,y3)` | TUPLE (x, y) | cubic Bezier 2D |
| `CURVE_CATMULL(t, p0, p1, p2, p3)` | FLOAT | Catmull-Rom 1D |
| `CURVE_CATMULL2(t, x0,y0, x1,y1, x2,y2, x3,y3)` | TUPLE (x, y) | Catmull-Rom 2D |
| `CURVE_BSPLINE(t, p0, p1, p2, p3)` | FLOAT | cubic B-spline: passes smoothly by the points instead of through them (like `SPLINE_BASIS`) |
| `CURVE_BSPLINE2(t, x0,y0, x1,y1, x2,y2, x3,y3)` | TUPLE (x, y) | B-spline 2D |
| `CURVE_HERMITE(t, p0, p1, m0, m1)` | FLOAT | cubic Hermite with tangents |

## Concept

All curves take a parameter `t` (typically `0..1`) and return an interpolated value. The curve types differ in **how** the control points determine the shape:

- **Lerp:** two points, straight line. Trivial.
- **Smoothstep / smootherstep:** two points, soft acceleration at the start/end ("ease-in-out").
- **Bezier:** four points. P0/P3 are start/end, P1/P2 are **tangent handles** (the curve heads towards them but does not pass through them).
- **Catmull-Rom:** four points. The curve passes **through** P1 and P2; P0 and P3 are tangent supports (predecessor/successor).
- **Hermite:** two points + two tangent vectors. Maximum control.

## Lerp and smoothstep

The simplest building blocks for animations:

```basic
DIM t AS FLOAT
t = 0.3                       ' 30% progress
PRINT CURVE_LERP(100.0, 200.0, t)       ' 130.0  (linear)
PRINT CURVE_SMOOTHSTEP(0.0, 1.0, t)     ' ~0.216 (S-curve)
PRINT CURVE_SMOOTHERSTEP(0.0, 1.0, t)   ' ~0.163 (gentler)
```

`CURVE_SMOOTHSTEP(e0, e1, x)` normalises `x` to the range `[e0, e1]` (clamped), then applies the S-curve. The classic choice for fade-in/out:

```basic
DIM fade AS FLOAT
fade = CURVE_SMOOTHSTEP(0.0, 1.0, t)   ' t in [0..1] -> Alpha
```

## Bezier — visually intuitive

Four points. P0 is the start, P3 is the end. P1 and P2 pull the curve in their direction, but it does not pass through them.

```basic
' Jump trajectory: start, high point, then fall, landing
DIM pos AS TUPLE
DIM t AS FLOAT
FOR t = 0.0 TO 1.0 STEP 0.02
    pos = CURVE_BEZIER2(t,   0.0, 200.0,
                              100.0,  50.0,
                              200.0,  50.0,
                              300.0, 200.0)
    DRAWIMAGE(player, pos[0], pos[1])
NEXT
```

P0=(0,200) start bottom left, P1=(100,50) tangent handle at the top, P2=(200,50) tangent handle at the top, P3=(300,200) end bottom right. Result: a smooth jump trajectory.

## Catmull-Rom — through the points

Unlike Bezier, Catmull-Rom passes **through** P1 and P2. P0 and P3 determine the tangents at P1 (from P2-P0) and P2 (from P3-P1).

Handy for **splines through waypoints**:

```basic
' 4 waypoints of a patrol. Interpolate between wp1 and wp2,
' wp0 and wp3 are the neighbours (for a smooth curve).
DIM pos AS TUPLE
pos = CURVE_CATMULL2(t,
                     wp0_x, wp0_y,
                     wp1_x, wp1_y,
                     wp2_x, wp2_y,
                     wp3_x, wp3_y)
```

For a path of N waypoints you iterate through the segments: between i=1 and i=2, then i=2 and i=3, and so on — each segment uses 4 consecutive waypoints.

## Hermite — explicit tangents

`CURVE_HERMITE(t, p0, p1, m0, m1)` interpolates from `p0` to `p1` with explicit tangent vectors `m0` (at the start) and `m1` (at the end). For when you want to prescribe the velocity at the start and end:

```basic
' Slow-In, Slow-Out:
PRINT CURVE_HERMITE(t, 0.0, 100.0, 0.0, 0.0)
' Start with 0 tangent, end with 0 tangent -> like SmoothStep

' Fast-Start, Slow-Out:
PRINT CURVE_HERMITE(t, 0.0, 100.0, 300.0, 0.0)
```

## `curves` vs `tween`

Both animate values, but differently:

| Aspect | `tween` | `curves` |
|---|---|---|
| Model | "animate from a to b in N ms" | "what is the value at t = 0.3?" |
| State | TWEEN handle, update per frame | no state, just a function |
| Use | long animation life cycle | path sampling, procedural |
| Simple lerp | `TWEEN_NEW(0, 100, 1000, "linear")` | `CURVE_LERP(0, 100, t)` |

For **movement over time**: `tween`. For **determining a point on a curve**: `curves`. The two can be combined: a `tween` animates `t` from 0 to 1 over 2 seconds, and then `CURVE_BEZIER2(t, ...)` is the position.

## Game patterns

**Camera smoothing towards a target:**

```basic
cam_x = CURVE_LERP(cam_x, target_x, 0.1)   ' 10% closer every frame
cam_y = CURVE_LERP(cam_y, target_y, 0.1)
```

**Health bar animation:**

```basic
DIM display_hp AS FLOAT
display_hp = CURVE_LERP(display_hp, actual_hp, 0.15)   ' follow smoothly
DRAW_BAR(display_hp)
```

**Cutscene camera move via Bezier:**

```basic
' t runs from 0 to 1 over 3 seconds
DIM t AS FLOAT
t = (MILLIS() - cutscene_start) / 3000.0
IF t > 1.0 THEN t = 1.0

DIM pos AS TUPLE
pos = CURVE_BEZIER2(t, start_x, start_y, ctrl1_x, ctrl1_y,
                       ctrl2_x, ctrl2_y, end_x, end_y)
CAMERA_SET(pos[0], pos[1])
```

## Example

[examples/74_curves_path.dh](../../examples/74_curves_path.dh) shows the curve types visually — an animation runs along different spline types.
