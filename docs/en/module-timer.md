# Module `timer`

Scheduled actions without MILLIS bookkeeping (native in `dhrt`): "do X in 2
seconds", "do Y every 500 ms" and cooldowns in one line. Closes the gap
between the raw time built-ins (`MILLIS`/`TIMER`/`DELTA`) and what games
really need -- spawners, delayed effects, fire rates.

```basic
IMPORT "timer"
```

## Timer: AFTER / EVERY

| Function | Effect |
|---|---|
| `TIMER_AFTER(ms, fn)` → INTEGER | call the FUNCREF once after `ms`; returns a timer ID |
| `TIMER_EVERY(ms, fn)` → INTEGER | call the FUNCREF every `ms` (`ms > 0`); returns a timer ID |
| `TIMER_CANCEL(id)` | cancel a timer (fired/unknown ID = no-op) |
| `TIMER_ACTIVE(id)` → BOOLEAN | is the timer still running? |
| `TIMER_COUNT()` → INTEGER | number of active timers |
| `TIMER_CLEAR()` | discard all timers + cooldowns (e.g. on a scene change) |
| `TIMER_UPDATE()` | **call once per frame** -- fires the callbacks that are due |

`TIMER_UPDATE()` follows the same pattern as `INPUT_UPDATE()`/`GUI_UPDATE()`:
without the call at the start of the frame nothing happens. The callbacks are
parameterless `SUB`s/`FUNCTION`s as a FUNCREF (bare name) — or a
**method on an instance** (`TIMER_EVERY(500, gegner.zucken)`), in which case
`Self` in the body points to that object.

```basic
IMPORT "timer"

SUB explodiere()
    PRINT "BOOM"
END SUB

SUB spawne()
    PRINT "neuer Gegner"
END SUB

TIMER_AFTER(2000, explodiere)        ' once in 2s
DIM spawner AS INTEGER
spawner = TIMER_EVERY(500, spawne)   ' every 500ms

WHILE NOT QUITREQUESTED()
    TIMER_UPDATE()                   ' fire the callbacks that are due
    ' ... game logic ...
    FLIP()
WEND

TIMER_CANCEL(spawner)
```

**Semantic details:**

- A `TIMER_EVERY` fires at most **once** per `TIMER_UPDATE` -- after a
  lag/`SLEEP` there is no catch-up burst, the next due time is always
  `now + ms`.
- Timer IDs stay stable (tombstones): `TIMER_CANCEL` on one ID does not
  change any other IDs.
- Callbacks may register or cancel timers themselves; newly registered
  timers become due at the **next** `TIMER_UPDATE` at the earliest.
- `TIMER_AFTER(0, fn)` fires at the next `TIMER_UPDATE` -- handy for "run
  at the end of the frame / decoupled".

## COOLDOWN -- a rate limiter in one line

`COOLDOWN(id$, ms)` is based on string IDs (like the immediate-mode `ui`
module, no handle needed): returns `TRUE` if the ID is free -- **and then
immediately starts the lock**. As long as the lock is running, it returns `FALSE`.

```basic
' Fire rate: at most every 250ms
IF INPUT_HELD("fire") AND COOLDOWN("schuss", 250) THEN
    Schiesse()
END IF

' Sound spam protection (replaces the MILLIS pattern from docs/module-audio.md)
IF COOLDOWN("hit_sfx", 100) THEN AUDIO_PLAY(hit_sound)
```

`COOLDOWN` needs no `TIMER_UPDATE` -- the check happens at the call.

## Scope

- **Animating values over time** → module `tween` (easings) or `curves`.
- **Complex timed sequences** (cutscenes, boss phases) → coroutines
  (`YIELD` + `CORO_RESUME`).
- **Frame delta for movement** → `DELTA()`; **stopwatch** → `TIMER()`;
  **timestamp** → `MILLIS()`.

## Example

[examples/113_timer.dh](../../examples/113_timer.dh) — spawner via
`TIMER_EVERY`, delayed explosion via `TIMER_AFTER`, fire cooldown.
