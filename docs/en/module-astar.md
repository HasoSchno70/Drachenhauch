# Module `astar`

A* pathfinding on a tile grid. Three heuristics, optional diagonal movement, anti-corner-cutting.

```basic
IMPORT "astar"
```

## Overview

| Function | Return value / effect |
|---|---|
| `ASTAR_NEW(w, h)` | ASTAR_GRID — everything passable |
| `ASTAR_CLEAR(g)` | delete all walls + the path |
| `ASTAR_WIDTH(g)` / `ASTAR_HEIGHT(g)` | INTEGER — size of the grid in tiles |
| `ASTAR_SET_WALL(g, x, y)` | mark a tile as impassable |
| `ASTAR_SET_PASSABLE(g, x, y)` | free it again |
| `ASTAR_IS_WALL(g, x, y)` | BOOLEAN — is this tile blocked? |
| `ASTAR_SET_DIAGONAL(g, allow)` | BOOLEAN — diagonal movement on/off |
| `ASTAR_SET_HEURISTIC(g, name$)` | `"manhattan"` \| `"euclid"` \| `"chebyshev"` |
| `ASTAR_SET_DIAGONAL_COST(g, cost)` | default `√2` ≈ 1.414 |
| `ASTAR_FIND(g, sx, sy, ex, ey)` | BOOLEAN — `TRUE` if a path was found |
| `ASTAR_PATH_LEN(g)` | INTEGER — `0` if there is no path |
| `ASTAR_PATH_X(g, idx)` / `ASTAR_PATH_Y(g, idx)` | INTEGER — tile coordinate of the waypoint (0 = start) |
| `ASTAR_PATH_COST(g)` | FLOAT — total cost |
| `ASTAR_CLEAR_PATH(g)` | delete only the path data, walls stay |

## Concept

A grid only knows **passable** vs. **wall**. `ASTAR_FIND(start, end)` searches for the shortest path and stores it internally. `PATH_LEN` / `PATH_X` / `PATH_Y` read it out.

```basic
IMPORT "astar"

DIM grid AS ASTAR_GRID
grid = ASTAR_NEW(20, 15)

ASTAR_SET_WALL(grid, 5, 5)
ASTAR_SET_WALL(grid, 5, 6)
ASTAR_SET_WALL(grid, 5, 7)

DIM ok AS BOOLEAN
ok = ASTAR_FIND(grid, 0, 0, 19, 14)

IF ok THEN
    DIM i AS INTEGER
    FOR i = 0 TO ASTAR_PATH_LEN(grid) - 1
        PRINT ASTAR_PATH_X(grid, i), ASTAR_PATH_Y(grid, i)
    NEXT i
END IF
```

The path contains **the start AND the goal** — so for a single step to a direct neighbour, `ASTAR_PATH_LEN` = 2.

## Diagonal movement

By default only 4-way orthogonal movement (up/down/left/right). `ASTAR_SET_DIAGONAL(grid, TRUE)` adds the four diagonals — paths usually get noticeably shorter.

```basic
ASTAR_SET_DIAGONAL(grid, TRUE)
ASTAR_FIND(grid, 0, 0, 3, 3)
PRINT ASTAR_PATH_LEN(grid)        ' 4 (start + 3 diagonal steps)
PRINT ASTAR_PATH_COST(grid)       ' ~4.24  (3 * √2)
```

**Anti-corner-cutting**: diagonal steps are only allowed if both adjacent orthogonal cells are free. Otherwise a unit could slip through the corner of an "L"-shaped wall.

```
  ##.    Diagonal von (0,0) -> (1,1) ist NICHT erlaubt,
  #..    weil (1,0) und (0,1) Walls sind.
  ...
```

## Heuristics

A* uses a heuristic to estimate how far a node still is from the goal. The better the heuristic fits, the fewer nodes the algorithm has to expand.

| Name | Description | Optimal for |
|---|---|---|
| `manhattan` (default) | `|dx| + |dy|` | orthogonal movement only |
| `euclid` | `√(dx² + dy²)` | any diagonal cost |
| `chebyshev` | `max(|dx|, |dy|)` | diagonal cost == 1 (uniform) |

```basic
ASTAR_SET_DIAGONAL(grid, TRUE)
ASTAR_SET_DIAGONAL_COST(grid, 1.0)        ' a diagonal step costs 1, like orthogonal
ASTAR_SET_HEURISTIC(grid, "chebyshev")    ' optimal for this configuration
```

Heuristic names are case-insensitive (`"MANHATTAN"` works too).

## Reading the path

After a successful `ASTAR_FIND` there are two ways to consume the path:

**Index-based**:

```basic
DIM i AS INTEGER
FOR i = 0 TO ASTAR_PATH_LEN(grid) - 1
    DIM x AS INTEGER
    DIM y AS INTEGER
    x = ASTAR_PATH_X(grid, i)
    y = ASTAR_PATH_Y(grid, i)
    ' i=0 is the start, i=PATH_LEN-1 is the goal
    DrawTile(x, y)
NEXT i
```

**Step by step for AI**:

```basic
' only the next step for a moving unit
IF ASTAR_PATH_LEN(grid) >= 2 THEN
    DIM nx AS INTEGER
    DIM ny AS INTEGER
    nx = ASTAR_PATH_X(grid, 1)        ' index 0 is the current position
    ny = ASTAR_PATH_Y(grid, 1)
    MoveUnitTo(nx, ny)
END IF
```

## Performance

Heap-based open list (`heapq` internally). The complexity is `O((w·h) log(w·h))` in the worst case. For a 200×200 grid with moderate obstacles a search takes well under a millisecond.

If you update the map often but search rarely: call `ASTAR_NEW` once, then just add/remove walls and call `ASTAR_FIND` repeatedly. `ASTAR_CLEAR_PATH` only throws away the last path, not the wall configuration.

## External type

`ASTAR_GRID` — an opaque wrapper around the wall bit field, the configuration and the last path.

## Example: ASCII rendering

```basic
IMPORT "astar"

DIM grid AS ASTAR_GRID
grid = ASTAR_NEW(8, 6)
ASTAR_SET_DIAGONAL(grid, TRUE)

' a vertical wall between start and goal
DIM y AS INTEGER
FOR y = 0 TO 4
    ASTAR_SET_WALL(grid, 4, y)
NEXT y

ASTAR_FIND(grid, 0, 0, 7, 0)

DIM hits[8, 6] AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO ASTAR_PATH_LEN(grid) - 1
    hits[ASTAR_PATH_X(grid, i), ASTAR_PATH_Y(grid, i)] = 1
NEXT i

DIM x AS INTEGER
FOR y = 0 TO 5
    DIM row AS STRING
    row = ""
    FOR x = 0 TO 7
        IF ASTAR_IS_WALL(grid, x, y) THEN
            row = row + "#"
        ELSEIF hits[x, y] = 1 THEN
            row = row + "*"
        ELSE
            row = row + "."
        END IF
    NEXT x
    PRINT row
NEXT y
```

## See also

- [`physics`](module-physics.md) — collision tests + vector maths for pixel-accurate collisions (complementary: A* works on tiles, physics on points)
- Complete example: [`examples/51_astar.dh`](../../examples/51_astar.dh) — a maze with an ASCII path visualisation
