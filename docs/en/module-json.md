# Module `json`

Parse, read, **build** and output JSON. Loads a file or a string, lets you access fields via path notation — and since 2026-08 also set them.

```basic
IMPORT "json"
```

## Overview

| Function | Return value | Meaning |
|---|---|---|
| `JSON_PARSE(s$)` | JSON_HANDLE | read JSON from a string |
| `JSON_LOAD(path$)` | JSON_HANDLE | read JSON from a file |
| `JSON_STRINGIFY(h)` | STRING (compact) | back to text -- without line breaks, for saving or sending |
| `JSON_PRETTY(h)` | STRING (indented) | back to text -- indented, for reading |
| `JSON_GET_STRING(h, path$)` | STRING | read the value at the path as text (`"nutzer.name"`, `"posten.0"`) |
| `JSON_GET_INT(h, path$)` | INTEGER | read the value at the path as a whole number |
| `JSON_GET_FLOAT(h, path$)` | FLOAT | read the value at the path as a decimal number |
| `JSON_GET_BOOL(h, path$)` | BOOLEAN | read the value at the path as a truth value |
| `JSON_HAS(h, path$)` | BOOLEAN | is there anything at the path at all? |
| `JSON_LEN(h, path$)` | INTEGER | length of a list or number of fields of an object |
| `JSON_TYPE(h, path$)` | STRING | what is at the path? (`object`, `array`, `string`, `number`, `boolean`, `null`) |
| `JSON_KEYS(h, path$)` | ARRAY OF STRING | field names of an object |
| `JSON_GET_JSON(h, pfad$)` | JSON_HANDLE | subtree at the path as a document of its own (**copy**) — the counterpart to `JSON_SET_JSON`, for example to carry a piece you do not understand along unchanged |

**Writing:**

| Function | Effect |
|---|---|
| `JSON_NEW_OBJECT()` | empty `{}` → JSON_HANDLE |
| `JSON_NEW_ARRAY()` | empty `[]` → JSON_HANDLE |
| `JSON_SET_STRING(h, pfad$, wert$)` | set a field (create or replace) |
| `JSON_SET_INT(h, pfad$, wert)` | set a field as a whole number |
| `JSON_SET_FLOAT(h, pfad$, wert)` | set a field as a decimal number |
| `JSON_SET_BOOL(h, pfad$, wert)` | set a field as a truth value |
| `JSON_SET_NULL(h, pfad$)` | set a field to null |
| `JSON_SET_JSON(h, pfad$, andere)` | insert a whole document (**copy**) |
| `JSON_APPEND_STRING/INT/FLOAT/BOOL/JSON(h, pfad$, wert)` | append to the array at the path |
| `JSON_APPEND_NULL(h, pfad$)` | append a `null` to the array at the path — for lists with empty slots |
| `JSON_REMOVE(h, pfad$)` | remove → BOOLEAN (was anything there?) |

## Path notation

Paths use dot notation: `"user.name"` accesses the field `name` in the sub-object `user` of an object. Array indices are also written with a dot: `"items.0.title"` addresses the `title` field of the first array entry.

The empty path `""` points to the root element.

## Example

```basic
IMPORT "json"

DIM doc AS STRING
doc = "{""user"":{""name"":""Anna"",""alter"":30,""aktiv"":true},""hobbies"":[""Lesen"",""Code""],""score"":98.5}"

DIM j AS JSON_HANDLE
j = JSON_PARSE(doc)

PRINT JSON_GET_STRING(j, "user.name")       ' "Anna"
PRINT JSON_GET_INT(j, "user.alter")         ' 30
PRINT JSON_GET_BOOL(j, "user.aktiv")        ' TRUE
PRINT JSON_GET_FLOAT(j, "score")            ' 98.5

' iterate over the array
DIM i AS INTEGER
FOR i = 0 TO JSON_LEN(j, "hobbies") - 1
    PRINT JSON_GET_STRING(j, "hobbies." + STR$(i))
NEXT

' check existence
IF JSON_HAS(j, "user.email") THEN
    PRINT JSON_GET_STRING(j, "user.email")
ELSE
    PRINT "Keine Email"
END IF
```

## Reading a file

```basic
IMPORT "json"

IF FILEEXISTS("settings.json") THEN
    DIM cfg AS JSON_HANDLE
    cfg = JSON_LOAD("settings.json")
    DIM max AS INTEGER
    max = JSON_GET_INT(cfg, "max_lives")
ELSE
    PRINT "settings.json nicht gefunden, nutze Defaults"
END IF
```

## Checking types

`JSON_TYPE(h, path)` returns a STRING: `"object"`, `"array"`, `"string"`, `"number"`, `"boolean"`, `"null"` or `"missing"` (if the path does not exist).

```basic
SELECT CASE JSON_TYPE(j, "user.alter")
    CASE "number"
        PRINT "Alter ist eine Zahl"
    CASE "string"
        PRINT "Alter ist ein String!?"
    CASE "missing"
        PRINT "Alter fehlt"
END SELECT
```

## Building JSON

Until 2026-08 JSON could only be read. Anyone who wanted to write some glued
strings together — and broke at the first quotation mark in a name.
A handle is therefore **mutable**:

```basic
IMPORT "json"

DIM h AS JSON_HANDLE
h = JSON_NEW_OBJECT()
JSON_SET_STRING(h, "name", "Anna")
JSON_SET_INT(h, "alter", 30)
JSON_SET_BOOL(h, "aktiv", TRUE)

' intermediate levels are created automatically:
JSON_SET_STRING(h, "adresse.ort", "Koeln")

PRINT JSON_STRINGIFY(h)
' {"name":"Anna","alter":30,"aktiv":true,"adresse":{"ort":"Koeln"}}
```

Lists grow with `JSON_APPEND_*`. Here the empty path `""` means the
document itself:

```basic
DIM posten AS JSON_HANDLE
posten = JSON_NEW_ARRAY()
JSON_APPEND_STRING(posten, "", "Schraube")
JSON_APPEND_STRING(posten, "", "Mutter")
JSON_SET_JSON(h, "posten", posten)

' then keep filling directly at the target:
JSON_APPEND_STRING(h, "posten", "Unterlegscheibe")
```

And an object can now also be walked through — `JSON_LEN` always returned the
number of its keys for an object, but there was no way to get at the keys
themselves:

```basic
DIM schluessel AS ARRAY OF STRING
DIM i AS INTEGER
schluessel = JSON_KEYS(h, "adresse")
FOR i = 0 TO LEN(schluessel) - 1
    PRINT schluessel[i], JSON_GET_STRING(h, "adresse." + schluessel[i])
NEXT
```

### Six rules worth reading once

1. **A handle is a reference**, like MAP and ARRAY. `b = a` does not create a
   copy — whoever changes `b` changes `a`.
2. **`JSON_SET_JSON` inserts a COPY.** A JSON tree cannot share a
   subtree with another one; later changes to the source
   do not carry over.
3. **Missing intermediate levels are created as objects.** That is exactly why
   `JSON_SET_STRING(h, "kunde.adresse.ort", "Koeln")` is one call and not three.
4. **A number segment creates nothing.** `"posten.0"` on a fresh
   document could mean an array or an object with the key `"0"` —
   both are valid JSON, and the wrong choice is only noticed by the receiver.
   Instead of guessing, the message says how an array is created. So arrays
   are created with `JSON_NEW_ARRAY` and filled with `JSON_APPEND_*`.
5. **When writing, the empty path `""` does NOT mean the root.** When reading
   it does; when setting it would mean "throw away the whole document", and an
   accidentally empty variable must not do that — `JSON_SET_*` rejects it.
   With `JSON_APPEND_*` it is allowed (nothing is lost there) and means the
   document itself.
6. **The order of the keys stays the insertion order**, even after
   `JSON_REMOVE`. For JSON it is meaningless, but whoever signs a body
   or compares two outputs sees the difference.

**A dot in a key name cannot be addressed** — it separates the
path segments. That applies when reading as well as when writing.

## Round trip

```basic
PRINT JSON_STRINGIFY(j)              ' compact: {"user":{"name":"Anna",...}}
PRINT JSON_PRETTY(j)                 ' formatted with indentation
```

## Error handling

Strict type getters throw if the value does not match the requested type — catch them with `TRY/CATCH`:

```basic
TRY
    DIM x AS INTEGER
    x = JSON_GET_INT(j, "user.name")     ' name is a STRING -> error
CATCH e
    PRINT "Fehler: ", e
END TRY
```

`JSON_GET_INT` also accepts integral floats (`5.0` → `5`), but not `3.14` → that throws.

`JSON_PARSE` and `JSON_LOAD` throw on invalid JSON (unbalanced brackets, trailing comma, …).

## Complete example

Reading: [examples/24_json.dh](../../examples/24_json.dh).
Building (a REST body and a configuration file):
[examples/171_json_bauen.dh](../../examples/171_json_bauen.dh).
