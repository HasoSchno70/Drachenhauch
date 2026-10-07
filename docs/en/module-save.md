# Module `save`

High-level save/load with a JSON backend. Type-safe setters/getters, a version field, tolerant loading of old save files.

```basic
IMPORT "save"
```

## Overview

| Function | Returns / effect |
|---|---|
| `SAVE_NEW()` | SAVE_HANDLE — empty save |
| `SAVE_LOAD(path$)` | SAVE_HANDLE — throws if the file is missing |
| `SAVE_LOAD_OR_NEW(path$)` | SAVE_HANDLE — empty if the file is missing |
| `SAVE_EXISTS(path$)` | BOOLEAN — does the file exist? |
| `SAVE_WRITE(s, path$)` | write to a file |
| `SAVE_DELETE_FILE(path$)` | idempotent |
| `SAVE_VERSION(s)` / `SAVE_SET_VERSION(s, n)` | read/write the version field |
| `SAVE_SET_INT/FLOAT/STRING/BOOL(s, key$, value)` | setter — store a value under a key |
| `SAVE_GET_INT/FLOAT/STRING/BOOL(s, key$)` | strict — a missing key is an error |
| `SAVE_GET_INT_OR/FLOAT_OR/STRING_OR/BOOL_OR(s, key$, default)` | with fallback — returns `default` if the key is missing or has the wrong type |
| `SAVE_HAS(s, key$)` | BOOLEAN — is the key set? |
| `SAVE_DELETE(s, key$)` | idempotent |
| `SAVE_CLEAR(s)` | all keys gone, version stays |
| `SAVE_KEYS(s)` | STRING — sorted list |

## Concept

A save file is a flat map of key → value. Values are primitive types (integer, float, string, bool). Optionally a version field for migration.

```basic
IMPORT "save"

DIM s AS SAVE_HANDLE
s = SAVE_LOAD_OR_NEW("highscore.save")

DIM hi AS INTEGER
hi = SAVE_GET_INT_OR(s, "highscore", 0)

' Compare the player's value with the current round ...
IF score > hi THEN
    SAVE_SET_INT(s, "highscore", score)
    SAVE_WRITE(s, "highscore.save")
END IF
```

## Life cycle

Three ways to get a save handle:

```basic
DIM s AS SAVE_HANDLE

' Start empty
s = SAVE_NEW()

' File must exist - otherwise error
s = SAVE_LOAD("save.dat")

' Load the file, or empty if not present (typical game start)
s = SAVE_LOAD_OR_NEW("save.dat")
```

`SAVE_LOAD_OR_NEW` is the "right" way for program start: if the file exists, it is loaded; if it does not exist, you get an empty save. With broken JSON it throws anyway — otherwise the next `SAVE_WRITE` would overwrite the botched save.

## Strict vs. tolerant getters

**Strict** (`SAVE_GET_*`): throws if the key is missing or the type does not match. Good when you are sure the value has to be there:

```basic
DIM name AS STRING
name = SAVE_GET_STRING(s, "player_name")     ' throws if not there
```

**With default** (`SAVE_GET_*_OR`): always returns a value. On a type mismatch the default is returned (no cast):

```basic
DIM hi AS INTEGER
hi = SAVE_GET_INT_OR(s, "highscore", 0)      ' default 0 if missing or not an int
```

Tip: use `_OR` for anything "optional" (highscore, selected difficulty), and the strict getters when you depend on consistent state when loading anyway.

## File format

Save files are human-readable, indented JSON:

```json
{
  "_version": 1,
  "data": {
    "highscore": 4200,
    "player_name": "Anna",
    "completed_tutorial": true,
    "music_volume": 0.7
  }
}
```

So when debugging you can simply open them in a text editor.

## Versioning

If your save schema changes, use the version field:

```basic
DIM s AS SAVE_HANDLE
s = SAVE_LOAD_OR_NEW("save.dat")

IF SAVE_VERSION(s) < 2 THEN
    ' Migration v1 -> v2: rename old fields, set new defaults ...
    DIM oldname AS STRING
    oldname = SAVE_GET_STRING_OR(s, "name", "")
    SAVE_SET_STRING(s, "player_name", oldname)
    SAVE_DELETE(s, "name")
    SAVE_SET_VERSION(s, 2)
    SAVE_WRITE(s, "save.dat")
END IF
```

The default version is `1`. Save files without a `_version` field are interpreted as `1` (backwards tolerant).

## Tolerance when loading

So that an old save file does not torpedo the whole game start:

- **Missing `_version`**: → 1
- **Missing `data`**: → empty (no keys)
- **Top level not an object** (e.g. a JSON array): → throws (irreparable)
- **Keys with unexpected types**: strict getters throw, `_OR` getters return the default

JSON does not reliably distinguish `1` from `1.0` — `SAVE_GET_INT` therefore also accepts whole-number floats (`5.0` → `5`), but throws on `5.5`.

## External type

`SAVE_HANDLE` — an opaque wrapper around the version field + data map.

## See also

- [`scene`](module-scene.md) — per-scene data has the same API shape (type-safe setters/getters with an `_OR` variant), but only lives for the lifetime of the scene
- [`json`](module-json.md) — low-level JSON if you need more complex nested structures
- Complete example: [`examples/49_pong_scene.dh`](../../examples/49_pong_scene.dh) — Pong with `pong.save` for a persisted highscore
