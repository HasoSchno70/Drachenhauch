# Module `ecs`

Entity-component system with sparse-set storage. Pragmatic for games: entities are INTEGER IDs, components are named typed values. The architecture is optimised for iterating over component holders (cache-friendly), not for reflection or type hierarchies.

Native implementation in [`rust/drachenhauch_runtime/src/ecs.rs`](../../rust/drachenhauch_runtime/src/ecs.rs) (sparse set + bulk system ops).

```basic
IMPORT "ecs"
```

## Concept

- **World** — container for entities + components. You can have several (e.g. one per scene).
- **Entity** — an INTEGER ID. No wrapper object, no type.
- **Component** — a named typed value (INT, FLOAT, STRING, BOOL, or anything via OBJ), bound to an entity.

An entity can have any number of components. A pattern for games:

```basic
DIM player AS INTEGER
player = ECS_NEW_ENTITY(world)
ECS_ADD_FLOAT(world, player, "px", 100.0)
ECS_ADD_FLOAT(world, player, "py", 100.0)
ECS_ADD_FLOAT(world, player, "vx", 0.0)
ECS_ADD_FLOAT(world, player, "vy", 0.0)
ECS_ADD_INT(world, player, "hp", 100)
ECS_ADD_STRING(world, player, "name", "Hero")
```

Iterating over all "movers" (entities with `px` AND `vx`):

```basic
DIM movers AS ARRAY OF INTEGER
movers = ECS_QUERY2(world, "px", "vx")
DIM i AS INTEGER
FOR i = 0 TO LEN(movers) - 1
    DIM e AS INTEGER
    e = movers[i]
    ECS_ADD_FLOAT(world, e, "px",
                  ECS_GET_FLOAT(world, e, "px")
                  + ECS_GET_FLOAT(world, e, "vx"))
NEXT
```

**But:** this per-entity loop is **slow** (300,000 built-in calls per frame with 500 entities). For hot-path systems use the [bulk system ops](#bulk-system-ops) — typically **40× faster**:

```basic
ECS_INTEGRATE_FLOAT(world, "px", "vx")   ' all entities with px+vx
ECS_INTEGRATE_FLOAT(world, "py", "vy")
```

## World lifecycle

| Function | Return value / effect |
|---|---|
| `ECS_NEW_WORLD()` | ECS_WORLD — empty world |
| `ECS_NEW_ENTITY(w)` | INTEGER — new entity ID (>= 1) |
| `ECS_DESTROY(w, ent)` | BOOLEAN — TRUE if removed, FALSE if it did not exist |
| `ECS_ALIVE(w, ent)` | BOOLEAN |
| `ECS_COUNT(w)` | INTEGER — number of living entities |

`ECS_DESTROY` removes the entity from all component stores (sparse-set cleanup).

## Component add

One built-in per type — the type check happens on add (not on get).

| Function | value type |
|---|---|
| `ECS_ADD_INT(w, ent, name$, value)` | INTEGER |
| `ECS_ADD_FLOAT(w, ent, name$, value)` | FLOAT (an int is accepted + converted) |
| `ECS_ADD_STRING(w, ent, name$, value)` | STRING |
| `ECS_ADD_BOOL(w, ent, name$, value)` | BOOLEAN |
| `ECS_ADD_OBJ(w, ent, name$, value)` | anything (user class, MAP, ARRAY, ...) |

If the entity already holds the component, the value is overwritten. The component name must be passed as a STRING argument (not an identifier — components are created at run time).

## Component has / remove

| Function | Return value |
|---|---|
| `ECS_HAS(w, ent, name$)` | BOOLEAN |
| `ECS_REMOVE(w, ent, name$)` | BOOLEAN — TRUE if removed |

## Component get

`ECS_GET_*` throws if the entity does not have the component (or the entity is dead). If you want "default if missing": `ECS_GET_OR_*`.

| Function | Return value |
|---|---|
| `ECS_GET_INT(w, ent, name$)` | INTEGER |
| `ECS_GET_FLOAT(w, ent, name$)` | FLOAT |
| `ECS_GET_STRING(w, ent, name$)` | STRING |
| `ECS_GET_BOOL(w, ent, name$)` | BOOLEAN |
| `ECS_GET(w, ent, name$)` | anything (untyped) |
| `ECS_GET_OR_INT(w, ent, name$, default)` | INTEGER |
| `ECS_GET_OR_FLOAT(w, ent, name$, default)` | FLOAT |
| `ECS_GET_OR_STRING(w, ent, name$, default)` | STRING |
| `ECS_GET_OR_BOOL(w, ent, name$, default)` | BOOLEAN |

`GET_OR_*` also returns `default` when the component exists but has the wrong type (e.g. STRING where INT is expected). That is defensive — if you want to treat type conflicts as bugs, use the strict `GET_*` variant.

## Query

Returns an `ARRAY OF INTEGER` with the entity IDs that have ALL the named components. The result is a snapshot list — while you iterate over it you can destroy / create entities without the iteration crashing.

| Function | Return value |
|---|---|
| `ECS_QUERY(w, name$)` | ARRAY OF INTEGER |
| `ECS_QUERY2(w, n1$, n2$)` | ARRAY OF INTEGER — entities with n1 AND n2 |
| `ECS_QUERY3(w, n1$, n2$, n3$)` | ARRAY OF INTEGER — entities with n1 AND n2 AND n3 |

**Strategy:** the component with the fewest holders is chosen as the base (smallest iteration loop), then it is tested against the other sparse sets. For a sparse component (e.g. `boss = 1 entity`) a query is O(smallest component count), not O(world size).

## Bulk system ops

The classic ECS performance trap: a BASIC loop that issues 6 built-in calls per entity (`ECS_GET_FLOAT` × 2 + arithmetic + `ECS_ADD_FLOAT` × 1). With 500 entities × 100 frames that is **300,000 built-in calls** — the hot path lies in the built-in dispatch overhead, not in the actual computation.

The bulk ops process a whole component layer in ONE built-in call, internally as a cdef loop. Drastically faster with many entities (40× is realistic).

### Movement patterns

| Function | Effect |
|---|---|
| `ECS_INTEGRATE_FLOAT(w, target$, delta$)` | `target += delta` for all entities with both components |
| `ECS_INTEGRATE_INT(w, target$, delta$)` | INT variant |

Classic use: position += velocity.

```basic
ECS_INTEGRATE_FLOAT(world, "px", "vx")
ECS_INTEGRATE_FLOAT(world, "py", "vy")
```

Returns the number of entities moved (= the intersection).

### Scaling / reset / bounds

| Function | Effect |
|---|---|
| `ECS_SCALE_FLOAT(w, target$, factor)` | `target *= factor` for all holders |
| `ECS_FILL_FLOAT(w, target$, value)` | all values = value |
| `ECS_FILL_INT(w, target$, value)` | INT variant |
| `ECS_CLAMP_FLOAT(w, target$, lo, hi)` | clamp to `[lo, hi]` |

```basic
ECS_SCALE_FLOAT(world, "vx", 0.95)            ' Friction
ECS_SCALE_FLOAT(world, "vy", 0.95)
ECS_CLAMP_FLOAT(world, "px", 0.0, 640.0)      ' bounds on screen
ECS_CLAMP_FLOAT(world, "py", 0.0, 480.0)
```

### Lifecycle

| Function | Effect |
|---|---|
| `ECS_REMOVE_DEAD(w, name$, threshold)` | destroy entities with `value <= threshold` |
| `ECS_COUNT_WITH(w, name$)` | O(1) — number of holders of a component |

```basic
ECS_REMOVE_DEAD(world, "hp", 0)               ' destroy the dead
PRINT "Lebende Bullets:", ECS_COUNT_WITH(world, "px")
```

## Bullet-hell game loop, complete example

```basic
IMPORT "ecs"

DIM world AS ECS_WORLD
world = ECS_NEW_WORLD()

' create 1000 bullets
DIM i AS INTEGER
FOR i = 1 TO 1000
    DIM e AS INTEGER
    e = ECS_NEW_ENTITY(world)
    ECS_ADD_FLOAT(world, e, "px", RND(640) + 0.0)
    ECS_ADD_FLOAT(world, e, "py", RND(480) + 0.0)
    ECS_ADD_FLOAT(world, e, "vx", (RND(40) - 20) * 0.5)
    ECS_ADD_FLOAT(world, e, "vy", (RND(40) - 20) * 0.5)
    ECS_ADD_INT(world, e, "hp", 100 + RND(100))
    ECS_ADD_INT(world, e, "regen", -1)
NEXT

' per frame: 8 bulk system calls instead of 6000 per-entity calls
DIM frame AS INTEGER
FOR frame = 1 TO 200
    ECS_INTEGRATE_FLOAT(world, "px", "vx")
    ECS_INTEGRATE_FLOAT(world, "py", "vy")
    ECS_SCALE_FLOAT(world, "vx", 0.99)
    ECS_SCALE_FLOAT(world, "vy", 0.99)
    ECS_CLAMP_FLOAT(world, "px", 0.0, 640.0)
    ECS_CLAMP_FLOAT(world, "py", 0.0, 480.0)
    ECS_INTEGRATE_INT(world, "hp", "regen")
    ECS_REMOVE_DEAD(world, "hp", 0)
NEXT
```

On the native VM this runs in ~20 ms — 1000 bullets × 200 frames × 8 systems = **1.6 million entity updates**.

Full example: [examples/bench_ecs_systems.dh](../../examples/bench_ecs_systems.dh).

## Storage architecture (sparse set)

Each component has three parallel structures:

```
dense:  list[entity_id]      -- kompakte Liste, eine Eintrag je Halter
values: list[any]            -- parallel zu dense, gleicher Index
sparse: dict[entity_id, dense_index]   -- O(1) Lookup
```

- **Iterating** over `dense` walks a flat list (more cache-friendly than a dict view over thousands of entries).
- **GET/HAS** are a single dict lookup.
- **ADD/REMOVE** run swap-with-last in O(1) — no shifting of lists.

The bulk ops use the sparse-set structure directly: they iterate over the smaller of the two components (for integrate) and query the other sparse set.

## Adding your own bulk ops

ECS is native in `dhrt` ([`rust/drachenhauch_runtime/src/ecs.rs`](../../rust/drachenhauch_runtime/src/ecs.rs)).
A new bulk op is added like this:

1. **A method on `World`** in `ecs.rs` — iterates over the sparse-set storage
   in a Rust loop (iterate over the smaller of the two components, query
   the other).
2. **A built-in arm** in the ECS dispatch (`vm.rs` `try_ecs` or `builtins.rs`) — arity
   and type checks, then delegate to the `World` method.
3. **A golden test** in `tests/` + an entry in `daten/builtin_index.json`.

Sketch for a hypothetical `ECS_ADD_TO(w, target, source, scale)`:

```rust
pub fn add_to_scaled(&mut self, target: &str, source: &str, scale: f64) {
    let (Some(t), Some(s)) = (self.components.get(target), self.components.get(source))
        else { return; };
    // ... Loop ueber die Schnittmenge: t[e] += s[e] * scale ...
}
```

## External type

| Type | Effect |
|---|---|
| `ECS_WORLD` | world container. `DIM w AS ECS_WORLD` |

Entity IDs are INTEGER (no type of their own).
