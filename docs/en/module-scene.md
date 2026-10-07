# Module `scene`

Stack-based scene/state manager for games. Instead of juggling global flags (`isInMenu`, `isPaused`, `gameOver`), the currently active screen is pushed onto a stack as a string name. Each scene has its own data bucket for short-lived state.

```basic
IMPORT "scene"
```

## Overview

| Function | Returns / effect |
|---|---|
| `SCENE_PUSH(name$)` | scene on top of the stack |
| `SCENE_POP()` | remove the top one, the previous one becomes active |
| `SCENE_SWITCH(name$)` | replace the stack completely |
| `SCENE_CURRENT()` | STRING — active name (`""` if empty) |
| `SCENE_DEPTH()` | INTEGER — how many scenes are on the stack? |
| `SCENE_HAS(name$)` | BOOLEAN — anywhere on the stack? |
| `SCENE_RESET()` | empty the stack completely |
| `SCENE_SET_INT/FLOAT/STRING/BOOL(key$, value)` | set data in the top scene |
| `SCENE_GET_INT/FLOAT/STRING/BOOL(key$)` | strict — throws if missing / wrong type |
| `SCENE_GET_INT_OR/FLOAT_OR/STRING_OR/BOOL_OR(key$, default)` | with fallback — returns `default` if the key is missing or has the wrong type |
| `SCENE_HAS_KEY(key$)` | BOOLEAN — is the key set in the top scene? |
| `SCENE_DELETE(key$)` | idempotent |

## Concept

Three operations are enough for most games:

- **PUSH/POP**: a pause overlay over the running game — `SCENE_PUSH("pause")` → pause active, `SCENE_POP()` → back to the game.
- **SWITCH**: a complete change with no way back — menu → game → game over. `SCENE_SWITCH("playing")` throws the stack away and starts with the new scene.
- **CURRENT**: your game loop asks every frame whose turn it is and dispatches via `SELECT CASE`.

```basic
IMPORT "scene"

SCENE_SWITCH("menu")

WHILE NOT QUITREQUESTED()
    SELECT CASE SCENE_CURRENT()
        CASE "menu"
            UpdateMenu()
            DrawMenu()
        CASE "playing"
            UpdatePlaying()
            DrawPlaying()
        CASE "gameover"
            UpdateGameOver()
            DrawGameOver()
    END SELECT
    FLIP()
    SLEEP(16)
WEND
```

## Per-scene data

Each scene has its own data bucket. You cannot access it directly by scene name — operations always act on the **top** scene.

```basic
SCENE_PUSH("playing")
SCENE_SET_INT("score", 0)
SCENE_SET_INT("lives", 3)

' ... at some point later, in the update loop:
DIM s AS INTEGER
s = SCENE_GET_INT("score")
SCENE_SET_INT("score", s + 100)
```

Strict getters throw on a missing key or wrong type — good for catching typos early. Tolerant getters with `_OR(default)` never throw:

```basic
DIM hi AS INTEGER
hi = SCENE_GET_INT_OR("highscore", 0)   ' no error if never set
```

### Data life cycle

- `SCENE_PUSH` creates a fresh, **empty** data bucket. The bucket of the scene underneath stays untouched and is reachable again after `SCENE_POP`.
- `SCENE_SWITCH` throws away *all* old buckets.
- `SCENE_POP` deletes the top bucket — even if you immediately follow up with `SCENE_PUSH("gleichname")` (same name), the new one is empty.

## Example: pause overlay

```basic
IMPORT "scene"

SCENE_SWITCH("playing")
SCENE_SET_INT("score", 0)

WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(KEY_P) THEN
        IF SCENE_CURRENT() = "playing" THEN
            SCENE_PUSH("pause")
        ELSEIF SCENE_CURRENT() = "pause" THEN
            SCENE_POP()
        END IF
    END IF

    SELECT CASE SCENE_CURRENT()
        CASE "playing"
            ' game mechanics
            ...
        CASE "pause"
            ' game is frozen - only draw the overlay
            DrawPlayingFrozen()
            DrawPauseOverlay()
    END SELECT
    FLIP()
WEND
```

## External type

The module registers no handle type visible to DH code — the stack lives in the module state. A test should call `SCENE_RESET()` before each test run.

## See also

- [`save`](module-save.md) — high-level save/load with a JSON backend, complements scene well (persisting highscores, run settings)
- Complete example: [`examples/49_pong_scene.dh`](../../examples/49_pong_scene.dh) — Pong with menu/playing/game-over scenes
