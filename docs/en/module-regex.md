# Module `regex`

Python-compatible regex matching with a pattern cache. Handy for text processing, score parsers, save-file migration, chat filters, highscore lists — anything where `INSTR` falls short.

```basic
IMPORT "regex"
```

## Overview

| Function | Returns | Effect |
|---|---|---|
| `REGEX_MATCH(text, pattern)` | BOOLEAN | Full match (pattern must cover the whole text) |
| `REGEX_TEST(text, pattern)` | BOOLEAN | Looks for an occurrence anywhere in the text |
| `REGEX_FIND(text, pattern)` | STRING | First match (empty string if not found) |
| `REGEX_FIND_POS(text, pattern [, ab])` | TUPLE | Position of the first match as (start, length) in characters counted from 0 like INSTR, (-1, 0) without a match; `ab` (from) continues the search from this character and still sees the characters before it (a word boundary `\b` at the start of the search only applies where there really is one) |
| `REGEX_ESCAPE$(text)` | STRING | Escapes all special characters: the result matches exactly the text itself -- for a search that can run literally or as an expression |
| `REGEX_FIND_ALL(text, pattern)` | ARRAY OF STRING | All non-overlapping matches |
| `REGEX_REPLACE(text, pattern, repl)` | STRING | Replaces all matches |
| `REGEX_REPLACE_ONCE(text, pattern, repl)` | STRING | Replaces only the first match |
| `REGEX_SPLIT(text, pattern)` | ARRAY OF STRING | Splits at every match |

## Pattern syntax

The patterns are **Python regex** (`re` module). The most important building blocks:

| Pattern | Meaning |
|---|---|
| `.` | any single character (except newline) |
| `\d` `\D` | digit / non-digit |
| `\w` `\W` | word character (a-z A-Z 0-9 _) / non-word |
| `\s` `\S` | whitespace / non-whitespace |
| `\b` | word boundary |
| `[abc]` | one of a, b, c |
| `[^abc]` | NONE of a, b, c |
| `[a-z]` | range |
| `*` `+` `?` | 0+, 1+, 0-or-1 repetitions (greedy) |
| `*?` `+?` | repetition, lazy/non-greedy |
| `{n}` `{n,m}` | exactly n / between n and m repetitions |
| `\|` | alternative (a\|b = a or b) |
| `(...)` | capture group |
| `^` `$` | start / end |

**Note:** Backslashes in BASIC strings are not escape sequences. `"\d+"` is correct in the pattern; you do not need `"\\d+"`.

## Match vs. test vs. find

```basic
PRINT REGEX_MATCH("123",   "\d+")          ' TRUE  -- whole text matches
PRINT REGEX_MATCH("123 hi", "\d+")         ' FALSE -- "hi" does not match too
PRINT REGEX_TEST("123 hi", "\d+")          ' TRUE  -- "123" is in there
PRINT REGEX_FIND("123 hi 456", "\d+")      ' "123" -- first match
```

`REGEX_FIND` returns an empty string if nothing is found — not NIL, so no NIL check is needed:

```basic
DIM number AS STRING
number = REGEX_FIND(input_line, "\d+")
IF number <> "" THEN
    PRINT "Gefunden:", number
END IF
```

## All matches as an array

`REGEX_FIND_ALL` returns an `ARRAY OF STRING` with all matches:

```basic
DIM nums AS ARRAY OF STRING
nums = REGEX_FIND_ALL("Hp: 80, Mp: 30, Lv: 5", "\d+")
DIM i AS INTEGER
FOR i = 0 TO LEN(nums) - 1
    PRINT nums[i]              ' "80", "30", "5"
NEXT
```

**Capture groups:** If the pattern has parentheses, the FIRST group is extracted (not the whole match). Example:

```basic
DIM ips AS ARRAY OF STRING
ips = REGEX_FIND_ALL("ip 10.0.0.1, fw 192.168.1.1", "ip (\d+\.\d+\.\d+\.\d+)")
PRINT ips[0]                   ' "10.0.0.1" (capture group, not "ip 10.0.0.1")
```

## Replace

`REGEX_REPLACE` replaces all matches with `repl`. Backslash references for capture groups:

```basic
PRINT REGEX_REPLACE("Hello WORLD", "WORLD", "Drachenhauch")
' "Hello Drachenhauch"

' Swap with capture groups
PRINT REGEX_REPLACE("Anna 30, Bob 25", "(\w+) (\d+)", "\2 (\1)")
' "30 (Anna), 25 (Bob)"
```

`REGEX_REPLACE_ONCE` replaces only the first occurrence — handy for "fix the first bug, leave the rest":

```basic
PRINT REGEX_REPLACE_ONCE("ha ha ha", "ha", "OK")
' "OK ha ha"
```

## Split

`REGEX_SPLIT` is `SPLIT$` on steroids — the separator pattern is a regex:

```basic
' Splitting on any whitespace (spaces, tabs, newlines mixed):
DIM parts AS ARRAY OF STRING
parts = REGEX_SPLIT("foo   bar\tbaz", "\s+")
PRINT LEN(parts)               ' 3
PRINT parts[0]; "|"; parts[1]; "|"; parts[2]
' "foo|bar|baz"

' CSV with optional whitespace around the comma:
parts = REGEX_SPLIT("a, b , c,d", "\s*,\s*")
' ["a", "b", "c", "d"]
```

## Performance: pattern cache

`regex` compiles each pattern once and caches the result. If you use the same pattern in a loop, it is not recompiled every time.

```basic
' These 1000 iterations compile the pattern ONCE:
DIM i AS INTEGER
FOR i = 0 TO 999
    IF REGEX_TEST(lines[i], "^\d+:") THEN ...
NEXT
```

## Practical patterns

**Parsing the score from a highscore line:**

```basic
DIM line AS STRING
line = "  3. Bob ........... 12500 pts"
DIM score AS STRING
score = REGEX_FIND(line, "\d+(?= pts)")   ' positive lookahead
PRINT score                                ' "12500"
```

**Chat word filter:**

```basic
DIM clean AS STRING
clean = REGEX_REPLACE(user_message, "(damn|hell)", "***")
```

**Checking a date format:**

```basic
IF REGEX_MATCH(save_date, "\d{4}-\d{2}-\d{2}") THEN
    ' YYYY-MM-DD correct
END IF
```

## Edge cases

- **Invalid pattern:** If the pattern is invalid from the regex point of view (e.g. unbalanced parentheses), `regex` throws a `DHRuntimeError` with the Python error message.
- **Empty text:** `REGEX_FIND("", ".*")` matches (the `.*` matches the empty string). `REGEX_FIND("", "x")` returns `""`.
- **REGEX_SPLIT with a pattern that matches the start:** returns an empty first string. Standard Python behaviour.

## In the native runtime (dhrt)

`regex` runs in `dhrt` (`dhrt run`, standalone `.exe`) — always included (no feature flag, uses the Rust `regex` crate). The patterns follow Python `re` notation; for the usual patterns (character classes, anchors, quantifiers, groups, alternation). **Not supported** (Rust `regex` limitation): backreferences (`\1`) *in the pattern* as well as lookahead/lookbehind. In `REGEX_REPLACE`, Python backrefs (`\1`, `\g<name>`) are automatically translated into Rust syntax.
