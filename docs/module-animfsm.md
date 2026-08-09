# Modul `animfsm` — Animations-State-Machine

Datengetriebene **Animation-State-Machine** im Stil von Unitys *Animator/Mecanim*:
benannte **States** (jeder an eine Sprite-Animation gebunden), benannte
**Parameter** (`bool`/`float`/`int`/`trigger`) und **Transitions** mit
Bedingungen. Die komplette Maschine liegt als `.gbanim`-JSON vor — erzeugt vom
visuellen Editor **`dhanim`** (Knoten = States, Pfeile = Transitions). Statt im
Spielcode von Hand `SPRITE_PLAY` zu schalten, setzt das Spiel pro Frame nur die
Parameter; `ANIM_FSM_UPDATE` entscheidet den Zustand und spielt die Animation.

```basic
IMPORT "animfsm"
IMPORT "sprite"

DIM hero AS SPRITE
hero = SPRITE_NEW(LOADIMAGE("assets/hero_walk.png"), 16, 16)

DIM fsm AS ANIM_FSM
fsm = ANIM_FSM_LOAD("assets/hero.gbanim")
ANIM_FSM_SETUP(fsm, hero)            ' registriert die State-Frames als Sprite-Anims

' --- pro Frame ---
ANIM_FSM_SET_FLOAT(fsm, "speed", ABS(vx))
IF INPUT_PRESSED("jump") THEN ANIM_FSM_TRIGGER(fsm, "jump")
ANIM_FSM_UPDATE(fsm, hero, dt_ms)    ' Animation voran + Zustandslogik
SPRITE_DRAW(hero)
```

## Das `.gbanim`-Format

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

- **State**: `name` (Zustandsname), `anim` (Sprite-Animationsname, default = `name`),
  `loop` (true = endlos, false = one-shot), optional `first`/`last`/`fps`
  (Frame-Range; `ANIM_FSM_SETUP` registriert sie als Sprite-Animation). `x`/`y`
  sind nur Editor-Layout (Laufzeit ignoriert sie).
- **Parameter**: `type` ∈ `bool` | `float` | `int` | `trigger`, optionaler
  `default`. Ein **Trigger** wird beim nächsten `ANIM_FSM_UPDATE` *konsumiert*
  (wirkt nur für ein Update — wie Unitys Trigger).
- **Transition**: `from` (`"*"` = **Any State** — Übergang von überall), `to`,
  optional `wait_finished` (erst wechseln, wenn die aktuelle one-shot-Animation
  *fertig* ist — ideal für Sprung/Angriff), und `conditions` (UND-verknüpft).
- **Condition-Operatoren**: `gt`/`lt`/`ge`/`le`/`eq`/`ne` (numerisch, gegen
  `value`), `is_true`/`is_false` (bool), `trigger`. Pro Frame werden die
  Transitions in Datei-Reihenfolge geprüft (`from == current` oder `"*"`); die
  erste passende gewinnt.

## API

| Builtin | Wirkung |
|---|---|
| `ANIM_FSM_LOAD(pfad$) AS ANIM_FSM` | `.gbanim` laden + validieren, im Default-State |
| `ANIM_FSM_SETUP(fsm, sprite)` | Frame-Ranges aller States als Sprite-Animationen registrieren + Default spielen |
| `ANIM_FSM_UPDATE(fsm, sprite, dt_ms) AS BOOLEAN` | Animation voranschreiben, Transitions auswerten, bei Wechsel spielen. TRUE = State gewechselt |
| `ANIM_FSM_FORCE(fsm, sprite, state$) AS BOOLEAN` | State erzwingen (ohne Bedingungen) — z.B. Reset/Respawn |
| `ANIM_FSM_SET_BOOL/FLOAT/INT(fsm, name$, wert)` | Parameter setzen |
| `ANIM_FSM_TRIGGER(fsm, name$)` | Trigger feuern (beim nächsten UPDATE konsumiert) |
| `ANIM_FSM_STATE(fsm) AS STRING` | aktueller State-Name |
| `ANIM_FSM_GET_FLOAT/INT/BOOL(fsm, name$)` | Parameter zurücklesen |

Externer Typ `ANIM_FSM` (Referenz-Handle). Implementierung
`rust/gb_runtime/src/animfsm.rs` (reine Logik, kein Grafik-State), Demo
[examples/111_anim_fsm.gb](../examples/111_anim_fsm.gb) + Daten
`examples/assets/hero.gbanim`, Tests `tests/test_animfsm.py`. Editor: **`dhanim`**
(siehe [docs/anim-editor.md](anim-editor.md)).
