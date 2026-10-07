# Module `tile_collide`

Box-vs-tilemap collision for platformers. The classic **separate-axis sweep** pattern: resolve the X movement first, then Y. Returns the new position + a hit flag per axis.

Requires a loaded `TILED_MAP` (see the [tiled module](module-tiled.md)).

```basic
IMPORT "tiled"
IMPORT "tile_collide"
```

## Overview

| Function | Returns |
|---|---|
| `TILE_SWEEP_X(map, layer_idx, x, y, w, h, dx)` | TUPLE `(new_x: FLOAT, hit: BOOL)` |
| `TILE_SWEEP_Y(map, layer_idx, x, y, w, h, dy)` | TUPLE `(new_y: FLOAT, hit: BOOL)` |
| `TILE_IS_SOLID(map, layer_idx, tx, ty)` | BOOLEAN (tile coordinates) |
| `TILE_AT_PIXEL(map, layer_idx, px, py)` | INTEGER (GID at a pixel position) |

**Convention:** `x, y, w, h, dx, dy` are in **pixels**. `tx, ty` in **tile units**. `dy > 0` = downwards (gravity in screen coordinates).

## Classic platformer update

```basic
' Per frame:
INPUT_UPDATE()

' Horizontal: user input → velocity
vx = INPUT_AXIS("left", "right") * 2.0

' Jump logic
IF INPUT_PRESSED("jump") AND on_ground THEN
    vy = -5.0
    on_ground = FALSE
END IF

' Gravity
vy = vy + 0.3
IF vy > 6.0 THEN vy = 6.0

' Collision: X sweep first, then Y with the new X
DIM rx AS TUPLE
rx = TILE_SWEEP_X(level, 0, px, py, PW, PH, vx)
px = rx[0]
IF rx[1] THEN vx = 0.0      ' wall -> velocity gone

DIM ry AS TUPLE
ry = TILE_SWEEP_Y(level, 0, px, py, PW, PH, vy)
py = ry[0]
IF ry[1] THEN
    IF vy > 0.0 THEN on_ground = TRUE    ' touching the ground
    vy = 0.0
ELSE
    on_ground = FALSE                    ' nothing below us
END IF
```

This pattern is the standard platformer physics. A separate X and Y sweep prevents "wall climbing" through diagonal hits.

## Solid detection

When is a tile "solid", i.e. able to collide? Two modes:

**Mode 1: per-tile property** (preferred)

In Tiled: select the tile → Custom Properties → `solid: bool = true`. The module automatically uses this mode as soon as **any** tile in the tileset has a `solid` property.

Advantage: you can have a MIX of solid and non-solid tiles in the same layer. The classic case: ground tiles solid, grass-tip decoration not solid (the player can walk through it).

**Mode 2: convention fallback**

If the tileset has NO `solid` properties AT ALL: every GID > 0 in the layer counts as solid. Convention: you create a dedicated collision layer (e.g. `collision`) that contains only wall tiles.

A 1:1 mapping is simpler, because you visually separate the collision data from the rendering.

## World edge

`TILE_IS_SOLID` returns `TRUE` for tile coordinates outside the map (negative, or >= width/height). That way the world edge blocks automatically — the player cannot fall out of the level without you having to catch it in the game code.

If you **want** the player to fall out of the world at the bottom (death plane), check that yourself in addition:

```basic
IF py > MAP_HEIGHT_IN_PIXELS THEN
    Die()
END IF
```

## Order of the two sweeps: why X first?

Mathematically, the "real" movement would be **diagonal**. In practice X and Y are done SEPARATELY, because:

- Pure diagonal sweeps sometimes collide with corners in such a way that the player "hangs" on a wall when trying to move down-left. Separate axes avoid that.
- The order "X then Y" prioritises horizontal movement — the typical platformer feel.
- The standard implementation in almost every 2D engine (Godot, Unity 2D Tilemap, ...) does it this way.

If you want a top-down shooter with real diagonal movement, sweep X and Y independently — the order then doesn't matter, both are treated the same.

## One-way platforms

V1 has no direct support for one-way platforms (a platform you can jump through from below but land on from above). Implementation as user code:

```basic
' Before the Y sweep: check whether the platform is below us AND we are falling
' AND are currently NOT standing on the platform.
' If all yes: treat platform tiles as solid temporarily; otherwise not.
```

That is game-specific. If you need it often, a future `tile_collide_oneway` built-in could lighten the load. Today, though, it lives in BASIC code.

## Slopes

Not in v1 either. Tile-based slope handling needs either a special per-tile property (`slope: "up_right"`) or polygon collision; both are an extension for a later round.

## Ad-hoc point tests

`TILE_AT_PIXEL(map, layer_idx, px, py)` is useful for ad-hoc tests beyond the box sweep: "is there a stone tile under the player right now?".

```basic
DIM gid_below AS INTEGER
gid_below = TILE_AT_PIXEL(level, 0, px + PW / 2, py + PH + 1)
IF TILED_TILE_PROP_STRING(level, gid_below, "type") = "lava" THEN
    Burn()
END IF
```

## Performance

`TILE_SWEEP_X/Y` iterates over the overlapping tiles in the direction of movement — at typical player speeds (< 8 pixels/frame) that is 2-3 tiles. Per frame: 2 sweeps × 2-3 tile checks = 4-6 operations. Negligible even with several thousand colliding entities.

Solid detection is cached on the first call per map — no repeated property lookups.

## Example

[examples/77_tiled_platformer.dh](../../examples/77_tiled_platformer.dh) — the complete pattern with a Tiled map, atlas, tile layer, Z-layer, input mapping and separate-axis sweep.

## In the native runtime (dhrt)

`tile_collide` runs natively (always included) and is **bit-identical** to the Python paths — the axis sweep (`TILE_SWEEP_X/Y`) including tile snapping and solid detection was ported 1:1.
