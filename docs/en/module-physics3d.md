# Module `physics3d` — real 3D rigid-body physics (Rapier3D)

A **full-fledged 3D physics solver** via [Rapier3D](https://rapier.rs):
gravity, integration, collision resolution and restitution (bounciness).
The 3D counterpart to [`physics2d`](module-physics2d.md) — and the big
difference from the [`physics`](module-physics.md) module, which only provides stateless
collision maths: here real dynamic bodies are simulated.

> **Native runtime only** (`dhrt` / F6 in the editor). The `PHYS_WORLD` holds the
> simulation pipeline; individual bodies are addressed via an **integer index**
> (the return value of `PHYS3D_ADD_*`).

```basic
IMPORT "physics3d"

DIM w AS PHYS_WORLD
w = PHYS3D_NEW()                              ' default gravity (0, -9.81, 0)

' static floor + dynamic cube that lands on it
DIM boden AS INTEGER
boden = PHYS3D_ADD_BOX(w, 0.0, 0.0, 0.0,  10.0, 0.5, 10.0,  FALSE, 0.3)
DIM kiste AS INTEGER
kiste = PHYS3D_ADD_BOX(w, 0.0, 8.0, 0.0,  0.5, 0.5, 0.5,    TRUE,  0.4)

WHILE NOT QUITREQUESTED()
    PHYS3D_STEP(w, DELTA())                   ' one simulation step
    ' ... query the body's position + rotation and render ...
    DIM px AS FLOAT
    px = PHYS3D_BODY_X(w, kiste)
    FLIP()
WEND
```

## World

| Function | Return value | Effect |
|---|---|---|
| `PHYS3D_NEW()` | `PHYS_WORLD` | new world; default gravity `(0, -9.81, 0)` |
| `PHYS3D_SET_GRAVITY(w, gx, gy, gz)` | — | set the gravity vector (e.g. `(0, -20, 0)` for a "heavier" world, `(0,0,0)` for weightlessness) |
| `PHYS3D_STEP(w, dt)` | — | advance the simulation by `dt` seconds (typically `DELTA()` or a fixed step such as `1.0/60.0`) |
| `PHYS3D_COUNT(w)` | INTEGER | number of living bodies |

## Creating bodies

`dynamic` is `TRUE` (moves, falls) or `FALSE` (static, immovable —
floor/walls). `bounce` is the restitution `0..1` (0 = no bounce, 1 = full
bounce). Both flags accept `TRUE`/`FALSE` **or** `1`/`0`. The return value is
the **body index** (INTEGER) for all subsequent calls.

| Function | Return value | Effect |
|---|---|---|
| `PHYS3D_ADD_BOX(w, x, y, z, hx, hy, hz, dynamic, bounce)` | INTEGER (idx) | cuboid at `(x,y,z)` with **half extents** `hx,hy,hz` (i.e. full size `2·h`) |
| `PHYS3D_ADD_SPHERE(w, x, y, z, r, dynamic, bounce)` | INTEGER (idx) | sphere at `(x,y,z)` with radius `r` |

## Querying and controlling bodies

| Function | Return value | Effect |
|---|---|---|
| `PHYS3D_BODY_X/Y/Z(w, idx)` | FLOAT | current position |
| `PHYS3D_BODY_QX/QY/QZ/QW(w, idx)` | FLOAT | rotation as a **quaternion** `(x, y, z, w)` |
| `PHYS3D_SET_POS(w, idx, x, y, z)` | — | set the position directly (teleport) |
| `PHYS3D_SET_VEL(w, idx, vx, vy, vz)` | — | set the linear velocity |
| `PHYS3D_APPLY_IMPULSE(w, idx, ix, iy, iz)` | — | give a one-off impulse (jump, shot, explosion) |
| `PHYS3D_REMOVE(w, idx)` | — | remove a body from the world |
| `PHYS3D_SLEEP(w, idx [, schlafen])` | — | put a body to sleep (without a third argument or TRUE) or wake it (FALSE). A sleeping body costs the solver nothing until a push, a set velocity or an awake body touches it. For freshly built stacks: 1323 boxes lying on each other cost 20 ms per step awake, 0.3 ms asleep. **Careful:** a sleeping body in mid-air stays there until something wakes it |
| `PHYS3D_SLEEPING(w, idx)` | BOOLEAN | is the body asleep? (static and invalid ones: FALSE) |

## Rendering (quaternion → transform)

`PHYS3D_BODY_*` returns the position **and** a rotation quaternion. To draw,
you build a world matrix from them with the [`m3d`](module-m3d.md) module and
render a model with it — the box/sphere is only the collision proxy:

```basic
IMPORT "g3d"
IMPORT "m3d"
IMPORT "physics3d"

DIM one AS VEC3
one = VEC3_NEW(1.0, 1.0, 1.0)

' ... per frame, per body idx:
DIM pos AS VEC3
pos = VEC3_NEW(PHYS3D_BODY_X(w, idx), PHYS3D_BODY_Y(w, idx), PHYS3D_BODY_Z(w, idx))
DIM rot AS QUAT
rot = QUAT_NEW(PHYS3D_BODY_QX(w, idx), PHYS3D_BODY_QY(w, idx), _
               PHYS3D_BODY_QZ(w, idx), PHYS3D_BODY_QW(w, idx))
DIM m AS MAT4
m = MAT4_TRS(pos, rot, one)          ' translation · rotation · scale
MODEL_MATRIX(meinModell, m, WHITE)
```

## Scope

- **`physics3d`** (this module) — *real* dynamic 3D bodies with a solver.
- **[`physics`](module-physics.md)** — only stateless collision/vector maths (no state, no gravity).
- **[`physics2d`](module-physics2d.md)** — the 2D counterpart (Rapier2D).

Complete example: [examples/107_physics3d.dh](../../examples/107_physics3d.dh).
