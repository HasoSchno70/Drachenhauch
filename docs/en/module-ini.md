# Module `ini`

Settings files in INI format — the format a human is supposed to touch with
an editor.

```basic
IMPORT "ini"
```

## Why not JSON

JSON and CSV already existed. For a file someone edits by hand, both are
unwieldy: JSON does not forgive a single extra comma, CSV has no named
fields. INI has existed for exactly this purpose for decades — and an ESP32
hobbyist already has it on the board anyway.

## An INI file is a MAP

There is **no handle type of its own**. An INI file here is a
`MAP OF STRING` with dotted keys:

```text
[fenster]                    "fenster.breite" -> "1280"
breite=1280        wird zu   "fenster.titel"  -> "Mein Spiel"
titel=Mein Spiel             "ton.laut"       -> "0.8"

[ton]
laut=0.8
```

That way the module needs four commands instead of two dozen getters and
setters — `MAPGETOR`, `MAPPUT`, `MAPKEYS`, `MAPHAS` and `VAL` can already do
all of that, and whoever knows the language therefore knows this module too.

| Function | Purpose |
|---|---|
| `INI_PARSE(text$)` → MAP OF STRING | from a string |
| `INI_LOAD(pfad$[, kodierung$])` → MAP OF STRING | from a file |
| `INI_TEXT$(m)` → STRING | back into INI text |
| `INI_SAVE(pfad$, m[, kodierung$])` | to a file |

## Example

```basic
IMPORT "ini"

DIM cfg AS MAP OF STRING
DIM breite AS INTEGER
DIM titel AS STRING

IF FILEEXISTS("einstellungen.ini") THEN
    cfg = INI_LOAD("einstellungen.ini")
ELSE
    cfg = INI_PARSE("")
END IF

' MAPGETOR returns the default if the key is missing -- the normal case
' for a settings file, and therefore not an error.
breite = VAL(MAPGETOR(cfg, "fenster.breite", "1280"))
titel  = MAPGETOR(cfg, "fenster.titel", "Mein Spiel")

' ... and write it back on exit
MAPPUT(cfg, "fenster.breite", STR$(breite))
INI_SAVE("einstellungen.ini", cfg)
```

## What is read

* `[abschnitt]` (section) — everything after it gets `abschnitt.` as a prefix
* `name = wert` — whitespace on the outside is dropped, inside the value it stays
* `;` and `#` **at the start of a line** are comments. Not in the middle — otherwise
  a path like `C:/a;C:/b` could not be stored
* an `=` in the value stays (`formel=a=b+c` yields `a=b+c`)
* keys **before** the first section keep their bare name
* `"wert"` in quotes — the quotes are dropped, the whitespace inside stays.
  That is the only way to keep leading spaces

**Broken lines do not stop you.** A line without `=` is skipped instead of
aborting the start. That is deliberate: a settings file is edited by a human,
often not a programmer — unlike JSON, where a broken file is almost always a
bug in the *program*.

## What happens when writing

Sections come in the order of their first appearance, keys in their own
order — a file that is read and written again looks as it did before,
instead of being reshuffled on every run. Values that would otherwise come
back differently (whitespace at the edge, a leading `;` or `#`) get
quotes.

**Two limits:**

* **Comments are lost when writing back.** Preserving them would mean
  carrying the original file along — and then it would no longer be a `MAP`,
  but a handle after all.
* **A dot in a key name is not addressable** — it separates section
  and name. The same limit as with JSON paths.

The **encoding** from [text encoding](builtins-core.md#text-encoding) applies
here too: an old settings file is often `cp1252`.

Example: [examples/174_einstellungen.dh](../../examples/174_einstellungen.dh).
