# Module `physics2d` — real 2D rigid-body physics (Rapier2D)

A **full-fledged 2D physics solver** via [Rapier2D](https://rapier.rs):
gravity, integration, collision resolution, restitution (bounciness) and
friction. The 2D counterpart to [`physics3d`](module-physics3d.md) — and the big
difference from the [`physics`](module-physics.md) module, which only provides stateless
collision maths: here real dynamic bodies are simulated.

```basic
IMPORT "physics2d"

DIM world AS PHYS2D_WORLD
world = PHYS2D_NEW()

' static floor + a falling box
PHYS2D_ADD_BOX(world, 240, 400, 240, 10, FALSE, 0.2)      ' wide, static
DIM box AS INTEGER
box = PHYS2D_ADD_BOX(world, 240, 40, 16, 16, TRUE, 0.3)   ' dynamic, bounces

' --- per frame ---
PHYS2D_STEP(world, DELTA())
DIM x AS FLOAT
x = PHYS2D_BODY_X(world, box)
DIM y AS FLOAT
y = PHYS2D_BODY_Y(world, box)
DIM ang AS FLOAT
ang = PHYS2D_BODY_ANGLE(world, box)       ' radians -> rotate the sprite
```

## Conventions

- **Screen coordinates:** Y grows **downwards** (as with all DH draw
  commands). The default gravity therefore pulls downwards (positive Y).
- **Pixel scale:** internally `length_unit = 100` is set (1 "metre" = 100 px),
  so that pixel-sized worlds stay stable. The default gravity is
  `(0, 980)` px/s² (≈ Earth's acceleration). Your own units via
  `PHYS2D_SET_GRAVITY`.
- **Box sizes are half extents:** `PHYS2D_ADD_BOX(w, x, y, hw, hh, …)` — `hw`/`hh`
  are **half** the width/height (Rapier cuboid). A 32×32 sprite ⇒ `hw=hh=16`.
- **`dynamic`/lock flags** accept `TRUE`/`FALSE` **or** `1`/`0`.
- **The body index** is a stable INTEGER (even after `REMOVE` smaller
  indices stay valid — tombstone slots).

## API

| Built-in | Effect |
|---|---|
| `PHYS2D_NEW() AS PHYS2D_WORLD` | new world (default gravity 0,980) |
| `PHYS2D_SET_GRAVITY(w, gx, gy)` | set the gravity |
| `PHYS2D_ADD_BOX(w, x, y, hw, hh, dynamic, bounce) AS INTEGER` | rectangle (half extents), restitution `bounce` 0..1 |
| `PHYS2D_ADD_CIRCLE(w, x, y, r, dynamic, bounce) AS INTEGER` | circle body |
| `PHYS2D_STEP(w, dt)` | simulate one time step (dt in s, internally clamped to 0.0001..0.05) |
| `PHYS2D_BODY_X/Y(w, idx)` | position |
| `PHYS2D_BODY_ANGLE(w, idx) AS FLOAT` | rotation angle in radians (for sprite rotation) |
| `PHYS2D_BODY_VX/VY(w, idx)` | linear velocity (e.g. "am I falling?") |
| `PHYS2D_SET_VEL(w, idx, vx, vy)` | set the velocity |
| `PHYS2D_APPLY_IMPULSE(w, idx, ix, iy)` | impulse (e.g. jump/shot) |
| `PHYS2D_SET_POS(w, idx, x, y)` | teleport the position |
| `PHYS2D_SET_DYNAMIC(w, idx, dynamic)` | switch between static and dynamic — for structures that should first **stand** and then collapse (wall, logo, tower). Also wakes the body up; otherwise Rapier lets resting bodies sleep and a block that has just been made dynamic would hang motionless in the air |
| `PHYS2D_IS_DYNAMIC(w, idx) AS BOOLEAN` | is the body dynamic? |
| `PHYS2D_SLEEP(w, idx [, schlafen])` | put a body to sleep (without a third argument or TRUE) or wake it (FALSE) — like `PHYS3D_SLEEP`: a freshly built stack costs nothing until something pushes it. A sleeping body in mid-air stays there until something wakes it |
| `PHYS2D_SLEEPING(w, idx) AS BOOLEAN` | is the body asleep? |
| `PHYS2D_LOCK_ROTATION(w, idx, locked)` | lock the rotation — e.g. so that a character does not topple over |
| `PHYS2D_REMOVE(w, idx)` | remove a body |
| `PHYS2D_COUNT(w) AS INTEGER` | number of living bodies |

## Pattern: coupling a sprite to a body

The position comes from `BODY_X`/`BODY_Y`, the rotation from `BODY_ANGLE` (radians).
With **`DRAWIMAGEROT`** (centred, rotated sprite blit) you couple an image
directly to a body — `BODY_ANGLE` is in radians, the blit expects degrees, hence
`DEG(...)`:

```basic
DRAWIMAGEROT(img, PHYS2D_BODY_X(world, id), PHYS2D_BODY_Y(world, id), _
             DEG(PHYS2D_BODY_ANGLE(world, id)), 2.0)   ' centred, scaled 2x
```

Complete example: [examples/145_physics2d_sprites.dh](../../examples/145_physics2d_sprites.dh)
(tumbling sprites). For a pure vector look,
[examples/112_physics2d.dh](../../examples/112_physics2d.dh) draws boxes as rotated
line outlines and circles with a spin line.

## Platformer tip

For a character that does not topple over: a dynamic box + `PHYS2D_LOCK_ROTATION(w,
id, TRUE)`, movement via `PHYS2D_SET_VEL` (set the horizontal component,
leave the vertical one to the physics) and jumping via `PHYS2D_APPLY_IMPULSE(w, id, 0,
-impuls)`. (For purely tile-based platformers,
[`tile_collide`](../../CLAUDE.md)/`controller` remains the lighter choice — `physics2d` is
for *real* dynamics: stacking, throwing, rolling, ragdolls, sandbox.)

External type `PHYS2D_WORLD`. Implementation
`rust/drachenhauch_runtime/src/physics2d.rs` (pure Rust, ungated), demo
[examples/112_physics2d.dh](../../examples/112_physics2d.dh), tests
`tests/pruef/physics2d.dhtest`.
