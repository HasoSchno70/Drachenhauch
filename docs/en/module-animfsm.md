# Module `animfsm` — animation state machine

A data-driven **animation state machine** in the style of Unity's *Animator/Mecanim*:
named **states** (each bound to a sprite animation), named
**parameters** (`bool`/`float`/`int`/`trigger`) and **transitions** with
conditions. The whole machine lives in a `.dhanim` JSON file — created by the
visual editor **`dhanim`** (nodes = states, arrows = transitions). Instead of
switching `SPRITE_PLAY` by hand in the game code, the game only sets the
parameters each frame; `ANIM_FSM_UPDATE` decides the state and plays the animation.

```basic
IMPORT "animfsm"
IMPORT "sprite"

DIM hero AS SPRITE
hero = SPRITE_NEW(LOADIMAGE("assets/hero_walk.png"), 16, 16)

DIM fsm AS ANIM_FSM
fsm = ANIM_FSM_LOAD("assets/hero.dhanim")
ANIM_FSM_SETUP(fsm, hero)            ' registers the state frames as sprite anims

' --- per frame ---
ANIM_FSM_SET_FLOAT(fsm, "speed", ABS(vx))
IF INPUT_PRESSED("jump") THEN ANIM_FSM_TRIGGER(fsm, "jump")
ANIM_FSM_UPDATE(fsm, hero, dt_ms)    ' advance the animation + state logic
SPRITE_DRAW(hero)
```

## The `.dhanim` format

```json
{
  "version": 1,
  "default": "idle",
  "params": [
    { "name": "speed", "type": "float", "default": 0.0 },
    { "name": "jump",  "type": "trigger" }
  ],
  "states": [
    { "name": "idle", "anim": "idle", "loop": true,  "first": 0, "last": 0, "fps": 2.0  },
    { "name": "run",  "anim": "run",  "loop": true,  "first": 0, "last": 3, "fps": 10.0 },
    { "name": "jump", "anim": "jump", "loop": false, "first": 0, "last": 3, "fps": 14.0 }
  ],
  "transitions": [
    { "from": "idle", "to": "run",  "conditions": [ { "param": "speed", "op": "gt", "value": 5.0 } ] },
    { "from": "run",  "to": "idle", "conditions": [ { "param": "speed", "op": "lt", "value": 5.0 } ] },
    { "from": "*",    "to": "jump", "conditions": [ { "param": "jump", "op": "trigger" } ] },
    { "from": "jump", "to": "idle", "wait_finished": true, "conditions": [] }
  ]
}
```

- **State**: `name` (state name), `anim` (sprite animation name, default = `name`),
  `loop` (true = endless, false = one-shot), optional `first`/`last`/`fps`
  (frame range; `ANIM_FSM_SETUP` registers it as a sprite animation). `x`/`y`
  are editor layout only (the runtime ignores them).
- **Parameter**: `type` ∈ `bool` | `float` | `int` | `trigger`, optional
  `default`. A **trigger** is *consumed* by the next `ANIM_FSM_UPDATE`
  (it only takes effect for one update — like Unity's triggers).
- **Transition**: `from` (`"*"` = **Any State** — a transition from anywhere), `to`,
  optional `wait_finished` (only switch once the current one-shot animation
  has *finished* — ideal for jumps/attacks), and `conditions` (combined with AND).
- **Condition operators**: `gt`/`lt`/`ge`/`le`/`eq`/`ne` (numeric, against
  `value`), `is_true`/`is_false` (bool), `trigger`. Each frame the
  transitions are checked in file order (`from == current` or `"*"`); the
  first one that matches wins.

## API

| Built-in | Effect |
|---|---|
| `ANIM_FSM_LOAD(pfad$) AS ANIM_FSM` | load + validate a `.dhanim`, starting in the default state |
| `ANIM_FSM_SETUP(fsm, sprite)` | register the frame ranges of all states as sprite animations + play the default |
| `ANIM_FSM_UPDATE(fsm, sprite, dt_ms) AS BOOLEAN` | advance the animation, evaluate transitions, play on a change. TRUE = state changed |
| `ANIM_FSM_FORCE(fsm, sprite, state$) AS BOOLEAN` | force a state (without conditions) — e.g. reset/respawn |
| `ANIM_FSM_SET_BOOL/FLOAT/INT(fsm, name$, wert)` | set a parameter |
| `ANIM_FSM_TRIGGER(fsm, name$)` | fire a trigger (consumed by the next UPDATE) |
| `ANIM_FSM_STATE(fsm) AS STRING` | name of the current state |
| `ANIM_FSM_GET_FLOAT/INT/BOOL(fsm, name$)` | read a parameter back |

External type `ANIM_FSM` (reference handle). Implementation
`rust/drachenhauch_runtime/src/animfsm.rs` (pure logic, no graphics state), demo
[examples/111_anim_fsm.dh](../../examples/111_anim_fsm.dh) + data
`examples/assets/hero.dhanim`, tests `tests/pruef/animfsm.dhtest`. Editor: **`dhanim`**
(see [docs/anim-editor.md](anim-editor.md)).
