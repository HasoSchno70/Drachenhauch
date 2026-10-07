# Module: physics

Pure built-in functions for 2D game maths: collision tests, distances, vector operations, ray casts. No "world" manager of its own — the caller keeps position, velocity etc. itself, the module only calculates.

```basic
IMPORT "physics"
```

## Overview

| Built-in | Purpose | Returns |
|---|---|---|
| `PHYSICS_BOX_BOX(x1,y1,w1,h1, x2,y2,w2,h2)` | AABB-AABB overlap | `BOOLEAN` |
| `PHYSICS_CIRCLE_CIRCLE(cx1,cy1,r1, cx2,cy2,r2)` | circle-circle | `BOOLEAN` |
| `PHYSICS_BOX_CIRCLE(bx,by,bw,bh, cx,cy,cr)` | box-circle | `BOOLEAN` |
| `PHYSICS_POINT_BOX(px,py, bx,by,bw,bh)` | point in box | `BOOLEAN` |
| `PHYSICS_POINT_CIRCLE(px,py, cx,cy,cr)` | point in circle | `BOOLEAN` |
| `PHYSICS_DISTANCE(x1,y1, x2,y2)` | Euclidean distance | `FLOAT` |
| `PHYSICS_DISTANCE2(x1,y1, x2,y2)` | squared distance (no sqrt) | `FLOAT` |
| `PHYSICS_LENGTH(vx, vy)` | vector length | `FLOAT` |
| `PHYSICS_NORM_X(vx, vy)` | X part of the normalised vector | `FLOAT` |
| `PHYSICS_NORM_Y(vx, vy)` | Y part of the normalised vector | `FLOAT` |
| `PHYSICS_REFLECT_X(vx,vy, nx,ny)` | X component of the reflection at normal `n` | `FLOAT` |
| `PHYSICS_REFLECT_Y(vx,vy, nx,ny)` | Y component of the reflection at normal `n` | `FLOAT` |
| `PHYSICS_RAY_BOX(rx,ry, dx,dy, bx,by,bw,bh)` | ray-box intersection, `t ∈ [0..1]` or `-1` | `FLOAT` |
| `PHYSICS_RAY_CIRCLE(rx,ry, dx,dy, cx,cy,cr)` | ray-circle intersection, `t ∈ [0..1]` or `-1` | `FLOAT` |
| `PHYSICS_POINT_TRI(px,py, ax,ay, bx,by, cx,cy)` | point in triangle (barycentric — the winding order of the corners does not matter) | `BOOLEAN` |
| `PHYSICS_LINES_HIT(ax,ay,bx,by, cx,cy,dx,dy)` | do two **line segments** intersect? (not the infinite lines) | `BOOLEAN` |
| `PHYSICS_LINES_X(ax,ay,bx,by, cx,cy,dx,dy)` | X of the intersection point — **`NAN` if there is none**, so ask `PHYSICS_LINES_HIT` first | `FLOAT` |
| `PHYSICS_LINES_Y(ax,ay,bx,by, cx,cy,dx,dy)` | Y of the intersection point (likewise `NAN`) | `FLOAT` |
| `PHYSICS_POINT_LINE(px,py, ax,ay, bx,by, dicke)` | does the point lie on the segment? `dicke` (thickness) gives the allowed distance — for clicks on thin lines | `BOOLEAN` |
| `PHYSICS_CIRCLE_LINE(cx,cy,r, ax,ay, bx,by)` | does a circle touch the segment? | `BOOLEAN` |
| `PHYSICS_POINT_POLY(px, py, xs, ys)` | point in polygon (ray method — also works for concave shapes) | `BOOLEAN` |
| `PHYSICS_DISTANCE3(x1,y1,z1, x2,y2,z2)` | distance between two points in space | `FLOAT` |
| `PHYSICS_SPHERE_SPHERE(x1,y1,z1,r1, x2,y2,z2,r2)` | do two spheres touch? — the sphere approximation is enough for most 3D hits | `BOOLEAN` |
| `PHYSICS_BROAD_NEW()` | create a broadphase | `PHYSICS_BROAD` |
| `PHYSICS_BROAD_CLEAR(b)` | discard all entries — once per frame before filling it again | — |
| `PHYSICS_BROAD_ADD(b, x, y, r)` | add a circle; returns its index | `INTEGER` |
| `PHYSICS_BROAD_COUNT(b)` | how many circles are in it? | `INTEGER` |
| `PHYSICS_BROAD_QUERY(b)` | find all overlapping pairs; returns their number | `INTEGER` |
| `PHYSICS_BROAD_PAIR_COUNT(b)` | number of pairs found (like the return value of `QUERY`) | `INTEGER` |
| `PHYSICS_BROAD_PAIR_A(b, i)` | first circle of pair `i` (index from `ADD`) | `INTEGER` |
| `PHYSICS_BROAD_PAIR_B(b, i)` | second circle of pair `i` | `INTEGER` |

### Broadphase: many circles at once

If you test 500 balls against each other with two nested loops, you make
125,000 comparisons per frame. The broadphase puts the circles into a grid and
compares only neighbours — the cost grows linearly with the count instead of quadratically.
The sequence per frame:

```basic
IMPORT "physics"
PHYSICS_BROAD_CLEAR(b)                      ' 1. clear
FOR i = 0 TO n - 1
    idx = PHYSICS_BROAD_ADD(b, x[i], y[i], r[i])   ' 2. fill
NEXT
paare = PHYSICS_BROAD_QUERY(b)              ' 3. search
FOR p = 0 TO paare - 1
    a = PHYSICS_BROAD_PAIR_A(b, p)          ' 4. fetch the pairs
    c = PHYSICS_BROAD_PAIR_B(b, p)
NEXT
```

The numbers returned are the indices from `PHYSICS_BROAD_ADD` — in the
order in which you put the circles in. You should not rely on the order of the
**pairs**.

## Conventions

- **Boxes** are `(x, y, w, h)` with `(x, y)` as the top left corner.
- **Circles** are `(cx, cy, r)` — centre and radius.
- **Rays** are `(rx, ry, dx, dy)` — origin and **direction vector with length**. `dx`/`dy` are not normalised; the length determines the maximum distance, `t = 1` is the end of the ray.
- **Normals** for `REFLECT_*` do **not** have to be normalised in advance — the function does that internally.
- **Vector operations with X/Y output have two separate functions** (`PHYSICS_NORM_X`/`PHYSICS_NORM_Y`). That is a design decision, not a necessity: Drachenhauch can do multiple return values perfectly well, via [`BYREF`](sprache.md#byref-parameters-multi-return). Two pure functions, however, can be written directly into an expression, whereas `BYREF` needs two variables declared beforehand.

## Example: a ball bounces off a wall

```basic
IMPORT "physics"

' reflect the ball velocity at a horizontal wall
DIM vx AS FLOAT
DIM vy AS FLOAT
vx = 5.0
vy = -3.0

' the wall normal points up (0, 1)
DIM nvx AS FLOAT
DIM nvy AS FLOAT
nvx = PHYSICS_REFLECT_X(vx, vy, 0.0, 1.0)
nvy = PHYSICS_REFLECT_Y(vx, vy, 0.0, 1.0)

PRINT nvx, nvy   ' 5.0  3.0  (X stays, Y flips)
```

## Example: does the player hit the enemy hitbox?

```basic
IMPORT "physics"

DIM player_x AS INTEGER
DIM player_y AS INTEGER
DIM enemy_x  AS INTEGER
DIM enemy_y  AS INTEGER

' AABB test (player and enemy both 32×32)
IF PHYSICS_BOX_BOX(player_x, player_y, 32, 32, enemy_x, enemy_y, 32, 32) THEN
    PRINT "Hit!"
END IF
```

## Example: shot trajectory (ray cast)

```basic
IMPORT "physics"

' shot from the player (200, 100), towards the mouse
DIM dx AS FLOAT
DIM dy AS FLOAT
dx = MOUSEX() - 200
dy = MOUSEY() - 100

' Does the shot ray hit the enemy box?
DIM t AS FLOAT
t = PHYSICS_RAY_BOX(200, 100, dx, dy, enemy_x, enemy_y, 32, 32)
IF t >= 0.0 THEN
    ' hit point (scale the ray end by t)
    DIM hit_x AS FLOAT
    DIM hit_y AS FLOAT
    hit_x = 200 + dx * t
    hit_y = 100 + dy * t
    CIRCLE(hit_x, hit_y, 4, RED)
END IF
```

## Performance tips

- **`PHYSICS_DISTANCE2`** is faster than `PHYSICS_DISTANCE`, because no `sqrt` runs. If you only want to check "X closer than Y", compare the squared values (`d² < r²` is equivalent to `d < r` for non-negative numbers).
- **`PHYSICS_BOX_BOX`** is the fastest collision (only 4 comparisons). Ideal as a broad-phase filter before more expensive tests.
- **`PHYSICS_RAY_BOX`** uses the slab algorithm — robust even with `dx=0` or `dy=0`.

## Complete example

[examples/142_physics.dh](../../examples/142_physics.dh) — a ball bouncing through a box world, with obstacle reflection and a mouse ray cast.
