# Standard built-ins

All built-in commands that are available without `IMPORT`. Graphics commands (SCREEN, BOX, …) are documented separately in [builtins-grafik.md](builtins-grafik.md).

## Contents

- [Output (PRINT)](#output-print)
- [Conversion](#conversion)
- [Math](#math)
- [Strings](#strings)
- [Bitwise](#bitwise)
- [Arrays](#arrays)
- [Maps](#maps)
- [File I/O](#file-io)
- [Bytes (BUFFER)](#bytes-buffer)
- [Operating system](#operating-system)
- [Checking and reporting](#checking-and-reporting)
- [Time & random](#time--random)
- [Types & encoding](#types--encoding)
- [Checksums and identity](#checksums-and-identity)
- [Game helpers](#game-helpers)

## Output (PRINT)

`PRINT` outputs one or more values separated by `,` or `;`:

- **`,`** separates with a **space**: `PRINT "x", 5` → `x 5`
- **`;`** separates **without** any gap: `PRINT "x"; 5` → `x5`
- a **trailing** `,` or `;` **suppresses the line break** (the next
  output follows directly): `PRINT "Laden...";` then `PRINT "fertig"` → `Laden...fertig`
- `PRINT` without arguments outputs an empty line.

```basic
PRINT "a", "b", "c"     ' a b c
PRINT "a"; "b"; "c"     ' abc
PRINT "Score: "; punkte ' Score: 42   (no space after the colon string)
PRINT "x = ";           ' no newline
PRINT x                 ' continues on the same line
```

## Conversion

| Function | Purpose |
|---|---|
| `STR$(v)` → STRING | Value to string. Bools become `"TRUE"`/`"FALSE"`, floats like `3.0` become `"3.0"`. |
| `VAL(s$)` → INTEGER/FLOAT | The number at the start of the text (`VAL("3 Äpfel")` = 3), also `&HFF`/`&O17`/`&B101`/`0x1F`/`0b11`. With `.` or an exponent → FLOAT, otherwise INT. No number at the start → `0`. |
| `INT(v)` → INTEGER | Number to INT (`floor`). `INT(3.7)` = 3, `INT(-1.5)` = -2. |
| `ABS(v)` | Absolute value. |
| `CHR$(n)` → STRING | Unicode code point to a 1-character string. `CHR$(65)` = `"A"`. |
| `ASC(s$)` → INTEGER | Code point of the first character. `ASC("Anna")` = 65. |
| `RGB(r, g, b)` → INTEGER | 3 numbers (0..255) to a 24-bit colour. `RGB(255, 128, 0)` = `&HFF8000`. Decimal numbers are **rounded** (so `x * 255 / 640` works) -- there is no clamping, 255.6 is still too large. |
| `HEX$(n)` → STRING | INT to upper-case hex (without prefix). `HEX$(255)` = `"FF"`. |
| `NUMFMT$(zahl [, nachkommastellen])` → STRING | write large numbers briefly (`1.2K`, `3.4M`) — for scores and idle games |
| `ENUM_NAME(enum, wert)` → STRING | return the name for an ENUM value — for display and debugging |

```basic
DIM s AS STRING
s = STR$(3.14)              ' "3.14"

DIM n AS INTEGER
n = VAL("42")               ' 42
n = VAL("3.7")              ' 3 (truncated, because INT is the default)

DIM f AS FLOAT
f = VAL("3.7")              ' 3.7

PRINT CHR$(65) + CHR$(66)   ' "AB"
PRINT HEX$(RGB(255, 0, 0))  ' "FF0000"
```

> **Hex and binary literals** come in two notations: classic BASIC with
> `&H`/`&B` **or** C style with `0x`/`0b`. `&HFF8000` = `0xFF8000` = 16744448,
> `&B1010` = `0b1010` = 10. This lets you give colours directly as a literal:
> `BOX(0, 0, 9, 9, 0xFF8000)`.

## Math

| Function | Purpose |
|---|---|
| `SIN(x)`, `COS(x)`, `TAN(x)` | Trigonometry (x in radians) |
| `ATAN(x)`, `ATAN2(y, x)` | Arc tangent |
| `ASIN(x)`, `ACOS(x)` | Arc sine/cosine (x in `[-1, 1]`) |
| `SQR(x)` | Square root (x ≥ 0) |
| `HYPOT(x, y)` | `SQR(x*x + y*y)` without overflow |
| `POW(b, e)` | b to the power of e |
| `EXP(x)` | e^x |
| `LOG(x[, base])` | Logarithm, natural by default |
| `DEG(rad)`, `RAD(grad)` | Radians ↔ degrees |
| `FLOOR(x)`, `CEIL(x)`, `ROUND(x)` | INT conversion |
| `ROUND(x, dezimalstellen)` → FLOAT | round to N decimal places, **to the even number** (see below) |
| `ROUND_HALF_UP(x[, dezimalstellen])` | commercial rounding: away from zero |
| `MIN(a, b, ...)`, `MAX(a, b, ...)` | variadic |
| `CLAMP(v, lo, hi)` | restricts v to `[lo, hi]` |
| `LERP(a, b, t)` | linear interpolation a..b (t not clamped) |
| `REMAP(v, in_lo, in_hi, out_lo, out_hi)` | rescale linearly |
| `FRAC(x)` | fractional part (signed): `x - TRUNC(x)` |
| `SIGN(x)` | -1, 0 or 1 |
| `LOG10(x)` | base-10 logarithm |
| `CLAMP01(v)` | restrict to `[0, 1]` |
| `WRAP(v, lo, hi)` | fold v cyclically into `[lo, hi)` (angle/index wrap-around) |
| `PINGPONG(t, len)` | swing back and forth in `[0, len]` (triangle wave) |
| `MOVETOWARD(cur, ziel, maxd)` | move cur by at most `maxd` towards `ziel` |
| `SMOOTHSTEP(e0, e1, x)` | smooth 0→1 transition (Hermite), clamped |
| `APPROX(a, b[, eps])` → BOOLEAN | `\|a-b\| ≤ eps` (default `1e-6`) |

### Rounding: two rules, and they are not the same

`ROUND` rounds the half **to the even number** (*round half to even*, like
Python): `ROUND(2.5)` is `2`, `ROUND(3.5)` is `4`. This is the rule that
produces no systematic drift across many values — right for
measurements, graphics and statistics.

Commercial rounding, by contrast, rounds **away from zero**: 2.5 → 3, −2.5 → −3.
That is what `ROUND_HALF_UP` is for. On a single line the
difference goes unnoticed; across a thousand line items it shifts the total.

```basic
PRINT ROUND(2.5)             ' 2
PRINT ROUND_HALF_UP(2.5)     ' 3
PRINT ROUND_HALF_UP(-2.5)    ' -3
```

**`ROUND_HALF_UP` rounds the number that is written there.** As a
floating-point number, `2.675` is really `2.67499999999999982…`; whoever rounds
this binary expansion is formally right (`ROUND(2.675, 2)` is `2.67`) and in
practice ends up with an invoice nobody can follow. `ROUND_HALF_UP(2.675, 2)`
returns `2.68`.

### Calculating with money

| Function | Returns | Purpose |
|---|---|---|
| `CENT(betrag)` | INTEGER | convert an amount into whole cents — **rounds** instead of truncating; also accepts written amounts (`"19,99"`, `"1.234,56"`) |
| `EURO$(cent [, symbol$])` | STRING | display a cent amount in German notation (`1999` becomes `19,99 €`) |


**Floating point and money do not mix.** This is not a quirk of
Drachenhauch, but of binary fractions: 0.1 can no more be written exactly in them
than 1/3 can in the decimal system.

```basic
PRINT 0.1 + 0.2              ' 0.30000000000000004
PRINT 0.1 + 0.2 = 0.3        ' FALSE
```

The answer to that is: **calculate in whole cents.** An `INTEGER` is
64 bits wide and reports an overflow as an error instead of silently wrapping around —
that is enough for ±92 quadrillion cents.

The road there has a trap, though, and it is one the advice
itself falls into:

```basic
PRINT INT(19.99 * 100)       ' 1998  (!)
PRINT INT(0.29 * 100)        ' 28    (!)
```

As a floating-point number, `19.99` lies slightly *below* 19.99, and `INT`
truncates. That is why there is **`CENT`**, which rounds instead of truncating — and which
does the same job for written amounts too, for example from a CSV file:

```basic
PRINT CENT(19.99)            ' 1999
PRINT CENT(0.29)             ' 29
PRINT CENT("19,99")          ' 1999
PRINT CENT("1.234,56")       ' 123456
```

For a written amount the rule is: if **both** separators occur,
the rear one separates the decimal places (`1.234,56` German, `1,234.56`
English — both give 123456). If a character occurs **several times**, they are
thousands separators (`1.234.567`). A single separator separates the
decimal places. Anything else — currency codes, letters — is an error
and is not silently cut off.

Display is done with **`EURO$`**, in German notation:

```basic
PRINT EURO$(1999)            ' 19,99 €
PRINT EURO$(123456789)       ' 1.234.567,89 €
PRINT EURO$(-1999)           ' -19,99 €
PRINT EURO$(1999, "CHF")     ' 19,99 CHF
PRINT EURO$(1999, "")        ' 19,99
```

`EURO$` takes **whole cents**, not a decimal number. `EURO$(19.99)` is an error,
and the message says straight away what was meant — because a display that silently
turned 19.99 into "19,99 €" would hide exactly the way of calculating
this is all about.

A complete invoice then looks like this:

```basic
DIM preis AS FLOAT
DIM menge AS INTEGER
DIM summe AS INTEGER          ' in cents!
preis = 19.99
menge = 3
summe = CENT(preis) * menge                   ' 5997
summe = summe + ROUND_HALF_UP(summe * 0.19)   ' tax, rounded commercially
PRINT EURO$(summe)                            ' 71,36 €
```

**The rule in one sentence:** calculate in `INTEGER` cents, convert with `CENT`,
round with `ROUND_HALF_UP`, display with `EURO$` — and use `FLOAT` only where
a percentage or a factor is involved.

**The limit, stated openly:** this is a way of calculating, not a data type of its own. Whoever
writes `DIM preis AS FLOAT` and sums with it directly still gets
`0.30000000000000004` — the language does not stop them. What a real
money type would cost and why it is nevertheless not coming is described in
[Draft: money](../entwurf-geldtyp.md) (German).

**Perlin noise** — deterministic: the same input always returns the same
value, roughly between `-1` and `1`. For procedural generation (terrain, caves,
organic movement).

| Function | Purpose |
|---|---|
| `NOISE(x)` → FLOAT | noise value along a line — for example for a wandering flicker |
| `NOISE2(x, y)` → FLOAT | noise value on a surface — terrain, clouds, marble |
| `NOISE3(x, y, z)` → FLOAT | noise value in space — caves, or a surface plus time as the third axis |
| `FBM(x, y, oktaven)` → FLOAT | several noise layers stacked: coarse shape plus fine detail |
| `FBM3(x, y, z, oktaven)` → FLOAT | the same in space |

At integer grid points Perlin is 0 by definition — whoever queries `NOISE2(1, 2)`
therefore always gets 0 and should use non-integer values.

```basic
PRINT WRAP(370, 0, 360)      ' 10.0
PRINT PINGPONG(2.5, 2)       ' 1.5
PRINT MOVETOWARD(0, 10, 3)   ' 3.0
PRINT ROUND(NOISE2(1.5, 2.5), 3)  ' reproducible noise value
```

Constants: `PI`, `TAU` (= 2·PI). (`E` is deliberately not a constant — `e`
is a common `CATCH e` variable name; use `EXP(1)`.)

```basic
PRINT SIN(PI / 2)            ' 1.0
PRINT SQR(2)                 ' 1.4142...
PRINT MIN(5, 2, 9, 1, 7)     ' 1
PRINT MAX(5, 2, 9, 1, 7)     ' 9
PRINT CLAMP(150, 0, 100)     ' 100
PRINT SIGN(-7)               ' -1
PRINT ROUND(3.14159, 2)      ' 3.14
PRINT DEG(PI)                ' 180.0
PRINT LERP(0.0, 10.0, 0.25)  ' 2.5
PRINT REMAP(5, 0, 10, 0, 100) ' 50.0

' vector length and angle
DIM laenge AS FLOAT
DIM winkel_grad AS FLOAT
laenge = HYPOT(3.0, 4.0)              ' 5.0
winkel_grad = DEG(ATAN2(4.0, 3.0))
```

### Colour helpers

| Function | Purpose |
|---|---|
| `RGB(r, g, b)` → INTEGER | see Conversion |
| `RED(c)`, `GREEN(c)`, `BLUE(c)` → INTEGER | channel 0..255 from `&HRRGGBB` |
| `HSV(h, s, v)` → INTEGER | HSV (h in degrees, s/v in `[0,1]`) → `&HRRGGBB` |
| `COLOR_LERP(c1, c2, t)` → INTEGER | mix two colours channel by channel (t 0..1) |

```basic
PRINT RED(&HFF8000)          ' 255
PRINT HSV(120.0, 1.0, 1.0)   ' 65280 (= &H00FF00, green)
PRINT COLOR_LERP(0, &HFFFFFF, 0.5)  ' 8421504 (= &H808080)
```

## Compressing text

| Function | Returns | Purpose |
|---|---|---|
| `COMPRESS$(text$)` | STRING | pack text (DEFLATE); the result is Base64, because strings are UTF-8 here and raw compressor output would not be |
| `DECOMPRESS$(gepackt$)` | STRING | unpack again |

For savegame-like text this saves roughly a factor of nine. It
also runs without a window, so in pure console programs.

## Strings

Functions with a `$` suffix also exist without it (`UPPER$` ≡ `UPPER`).

| Function | Purpose |
|---|---|
| `LEN(s)` → INTEGER | length (also for arrays; for a MAP the number of entries) |
| `UPPER$(s)`, `LOWER$(s)` | upper/lower case |
| `LEFT$(s, n)`, `RIGHT$(s, n)` | first/last n characters |
| `MID$(s, start[, n])` | substring from position start (0-based), n characters or to the end |
| `INSTR(s, sub[, start])` → INTEGER | position of sub in s, or -1 |
| `REPLACE$(s, alt, neu)` | replace all occurrences |
| `TRIM$(s)` | remove whitespace at the front/back |
| `SPLIT$(s, delim)` → ARRAY OF STRING | split |
| `JOIN$(arr, delim)` → STRING | join (array OF STRING) |
| `PADL$(s, breite[, fill])`, `PADR$(...)` | pad on the left/right |
| `REPEAT$(s, n)` | concatenate s n times |
| `SPACE$(n)` | n spaces |
| `HEX$(n)` | INT as a hex string |
| `FORMAT$(value, mask)` | printf style. `FORMAT$(42, "%05d")` → `"00042"`. Six specifiers: `%d`/`%i`, `%f`, `%x`, `%X`, `%s` — plus `%%` for a literal percent sign. Width, zeros and precision as usual (`%05d`, `%-6d`, `%.2f`). Anything else reports "unknown specifier" |

Extensions *(native runtime only)*:

| Function | Purpose |
|---|---|
| `LTRIM$(s)`, `RTRIM$(s)` | remove whitespace on the left only / right only |
| `REVERSE$(s)` | reverse the characters |
| `STARTSWITH(s, präfix)`, `ENDSWITH(s, suffix)` → BOOLEAN | check start/end |
| `CONTAINS(s, teil)` → BOOLEAN | substring contained? (function form of `teil IN s`) |
| `COUNT(s, teil)` → INTEGER | number of non-overlapping occurrences of `teil` |
| `TITLE$(s)` | first letter of each word upper case, the rest lower case |
| `CAMEL$(s)`, `SNAKE$(s)`, `PASCAL$(s)` | identifier notations: `"max hp"`, `"MaxHp"`, `"max_hp"` or `"max-hp"` become `maxHp`, `max_hp`, `MaxHp`. Words are separated by whitespace, `_`, `-`, `.`, the change from lower to upper case and the end of an abbreviation (`HTTPServer` → `http_server`) |
| `BETWEEN$(text, links, rechts[, ab])` | what lies between the first `links` (from character `ab`, 0-based like `MID$`) and the next `rechts`; `""` if one of them is missing |
| `REPLACE_BETWEEN$(text, links, rechts, neu)` | replace every piece between `links` and `rechts` with `neu`, the markers stay (`"{{name}}"` → `"{{?}}"`) |
| `NOSPACES$(s)` | remove all whitespace (spaces, tabs, line breaks) — for input like `"DE89 3704 0044"` |
| `BIN$(n)`, `OCT$(n)` | INTEGER as a binary/octal string (with sign) |
| `ISNUMERIC(s)` → BOOLEAN | parseable as a number? |
| `TRYVAL(s, default)` → INTEGER/FLOAT | robust `VAL`: on a parse error `default` instead of a silent `0` |

```basic
PRINT UPPER$("hallo")           ' "HALLO"
PRINT LEFT$("Drachenhauch", 4)     ' "Game"
PRINT MID$("Drachenhauch", 4, 5)   ' "Basic"
PRINT INSTR("hello world", "world")   ' 6
PRINT REPLACE$("a-b-c", "-", "_")     ' "a_b_c"

DIM teile AS ARRAY OF STRING
teile = SPLIT$("Anna,Bert,Cilly", ",")
PRINT JOIN$(teile, " | ")             ' "Anna | Bert | Cilly"

PRINT PADL$("42", 6, "0")             ' "000042"
PRINT REPEAT$("=*", 5)                ' "=*=*=*=*=*"
PRINT HEX$(&HCAFE)                    ' "CAFE"

PRINT REVERSE$("abc")                 ' "cba"
PRINT STARTSWITH("hello", "he")       ' TRUE
PRINT BIN$(10)                        ' "1010"
PRINT TRYVAL("oops", -1)              ' -1  (instead of 0 with VAL)
```

## Bitwise

Bitwise arithmetic works through **operators**, not functions. All of them take
INTEGER and return INTEGER; negative values as in Python (two's complement,
arbitrarily large).

| Operator | in C/Python |
|---|---|
| `a BAND b` | `a & b` |
| `a BOR b` | `a \| b` |
| `a BXOR b` | `a ^ b` |
| `BNOT a` | `~a` |
| `a SHL n` | `a << n` |
| `a SHR n` | `a >> n` |

```basic
DIM flags AS INTEGER
flags = 0
flags = flags BOR (1 SHL 0)         ' set bit 0
flags = flags BOR (1 SHL 3)         ' set bit 3
PRINT "Flags = 0x" + HEX$(flags)    ' "0x9"
PRINT "Bit 3? ", (flags BAND (1 SHL 3)) <> 0  ' TRUE
```

The parentheses around `1 SHL 3` are needed: all binary bit operators sit on
**one** precedence level and are evaluated from the left, so `a BOR 1 SHL 3`
would be `(a BOR 1) SHL 3`.

> There used to be the functions `BITAND`/`BITOR`/`BITXOR`/`BITNOT`/`SHL`/
> `SHR` for this. They have been **removed** — with the operators they would be duplicates.

## Arrays

See also [Language reference → Arrays](sprache.md#arrays).

| Function | Purpose |
|---|---|
| `LEN(arr)` → INTEGER | number of elements (1st dimension) |
| `DIMCOUNT(arr)` → INTEGER | number of dimensions |
| `DIMSIZE(arr, n)` → INTEGER | size of the n-th dimension (0-based) |
| `SORT(arr)`, `REVERSE(arr)` | sort / reverse a 1D array IN PLACE |
| `SORT(arr, absteigend)` | sort descending with a BOOLEAN flag *(native runtime only)* |
| `SORT(arr, comparator)` | sort with a FUNCREF comparator `cmp(a, b)` → INTEGER (<0/0/>0), stable. A bound method (`regel.cmp`) is allowed too *(native runtime only)* |
| `ARRAY_INDEXOF(arr, v)` → INTEGER | first index of v, otherwise -1 |
| `ARRAY_MAP(arr, funktion)` → ARRAY | new array with `funktion(x)` for each element, e.g. `ARRAY_MAP(a, FUNCTION(x) x * 2)` |
| `ARRAY_FILTER(arr, funktion)` → ARRAY | new array with the elements for which `funktion(x)` returns TRUE (same element type) |
| `ARRAY_REDUCE(arr, funktion, start)` | folds the array: `funktion(bisher, x)` for each element, starting with `start` |
| `ARRAY_FIND(arr, funktion)` → INTEGER | index of the first element for which `funktion(x)` returns TRUE, otherwise -1 |

**Aggregates** (1D `ARRAY OF INTEGER`/`FLOAT`) and helpers:

| Function | Purpose |
|---|---|
| `ARRAY_SUM(arr)` → INTEGER/FLOAT | sum (INTEGER array → INTEGER, otherwise FLOAT) |
| `ARRAY_AVG(arr)` → FLOAT | average (the array must not be empty) |
| `ARRAY_MIN(arr)`, `ARRAY_MAX(arr)` | smallest / largest element |
| `ARRAY_FILL(arr, wert)` | fill all elements with `wert` (IN PLACE, every dimension) |
| `ARRAY_COPY(arr)` → ARRAY | independent copy (same shape/type) |

**Dynamic 1D arrays** (grow/shrink IN PLACE):

| Function | Purpose |
|---|---|
| `ARRAY_PUSH(arr, wert)` → INTEGER | append an element at the end; returns the new length |
| `ARRAY_POP(arr)` → T | remove and return the last element (not empty) |
| `ARRAY_INSERT(arr, idx, wert)` → INTEGER | insert at index `idx` (`0..len`); new length |
| `ARRAY_REMOVE_AT(arr, idx)` → T | remove and return the element at `idx` |
| `REDIM(arr, länge)` | bring to `länge` — grows with the type default, shrinking cuts off; existing values stay |

```basic
DIM matrix[3, 4] AS INTEGER

PRINT DIMCOUNT(matrix)           ' 2
PRINT DIMSIZE(matrix, 0)         ' 3
PRINT DIMSIZE(matrix, 1)         ' 4
PRINT LEN(matrix)                ' 3 (= DIMSIZE 0)

DIM werte[3] AS INTEGER
werte[0] = 5 : werte[1] = 9 : werte[2] = 1
PRINT ARRAY_SUM(werte), ARRAY_AVG(werte)   ' 15  5.0
PRINT ARRAY_MIN(werte), ARRAY_MAX(werte)   ' 1  9

' Dynamic: use as a stack/list
DIM stack[0] AS INTEGER
PRINT ARRAY_PUSH(stack, 1)       ' 1 (new length)
PRINT ARRAY_PUSH(stack, 2)       ' 2
PRINT ARRAY_POP(stack)           ' 2
PRINT LEN(stack)                 ' 1
```

> The `ARRAY_*` aggregates and the dynamic array ops
> (`ARRAY_PUSH`/`POP`/`INSERT`/`REMOVE_AT`/`REDIM`) are implemented in `dhrt`
> (`builtins.rs`/`vm.rs`) and work along every path — `dhrt run` (also from the IDE),
> `--runsrc` and the `dhrt --export` standalone build.

## Maps

`MAP OF T` with STRING keys and values of type T.

| Function | Purpose |
|---|---|
| `MAPPUT(m, key$, value)` | set / overwrite |
| `MAPGET(m, key$)` → T | read, **throws** if the key is missing |
| `MAPGETOR(m, key$, default)` → T | read with a default |
| `MAPHAS(m, key$)` → BOOLEAN | existence |
| `MAPREMOVE(m, key$)` → BOOLEAN | TRUE if removed |
| `MAPSIZE(m)` → INTEGER | count |
| `MAPKEYS(m)` → ARRAY OF STRING | all keys |
| `MAPCLEAR(m)` | empty |

```basic
DIM scores AS MAP OF INTEGER
MAPPUT(scores, "Anna", 95)
MAPPUT(scores, "Bert", 78)
MAPPUT(scores, "Cilly", 99)

DIM keys AS ARRAY OF STRING
keys = MAPKEYS(scores)
DIM i AS INTEGER
FOR i = 0 TO LEN(keys) - 1
    PRINT keys[i], ": ", MAPGET(scores, keys[i])
NEXT

PRINT "Eve: ", MAPGETOR(scores, "Eve", 0)  ' 0 (default)
```

Memo pattern (Fibonacci cache):

```basic
DIM cache AS MAP OF INTEGER

FUNCTION fib(n AS INTEGER) AS INTEGER
    IF n < 2 THEN
        RETURN n
    END IF
    IF MAPHAS(cache, STR$(n)) THEN
        RETURN MAPGET(cache, STR$(n))
    END IF
    DIM v AS INTEGER
    v = fib(n - 1) + fib(n - 2)
    MAPPUT(cache, STR$(n), v)
    RETURN v
END FUNCTION
```

## ZIP

Backups, collections of receipts, export — the usual way to pass several files on
as one.

| Function | Purpose |
|---|---|
| `ZIP_LIST(archiv$)` → ARRAY OF STRING | names of all entries |
| `ZIP_READ$(archiv$, name$)` → STRING | one entry as text |
| `ZIP_READ(archiv$, name$)` → BUFFER | one entry as bytes |
| `ZIP_EXTRACT(archiv$, ordner$)` → INTEGER | extract everything, returns the number of files |
| `ZIP_CREATE(archiv$, dateien)` → INTEGER | archive from a list of file paths |
| `ZIP_WRITE(archiv$, namen, inhalte)` → INTEGER | archive from names and texts, without a detour through files |

```basic
DIM namen AS ARRAY OF STRING
DIM inhalte AS ARRAY OF STRING
DIM drin AS ARRAY OF STRING

namen = SPLIT$("brief.txt|unter/notiz.txt", "|")
inhalte = SPLIT$("Hallo Welt|zweite Datei", "|")
ZIP_WRITE("sicherung.zip", namen, inhalte)

drin = ZIP_LIST("sicherung.zip")
PRINT JOIN$(drin, ", ")
PRINT ZIP_EXTRACT("sicherung.zip", "entpackt")
```

**When extracting, the target of each write is checked.** An archive may
contain entries like `../../autoexec.bat` or `C:/Windows/x.dll`; whoever simply
appends the name from the archive to the target folder writes
outside of it — the attacker picks the file, you extract it. Such
entries are **skipped**, not written — a
drive letter like `C:` also counts as an escape attempt on **every** system,
not only on Windows (on Linux `C:` would otherwise simply be a folder name). `ZIP_EXTRACT` returns the
number of files actually created, so that a difference from the
length of `ZIP_LIST` stands out.

**`ZIP_CREATE` stores only the file name**, not the path under which the
file lay — otherwise an archive would carry the directory structure of the machine
it was created on out into the world. Whoever wants a folder structure *inside* the archive
uses `ZIP_WRITE` and gives the names themselves (`unter/notiz.txt`).

Compression uses Deflate. `ZIP_WRITE` and `ZIP_CREATE` each create the archive
**anew**; appending to an existing archive is not possible.

## Tasks: your own functions in the background

`TASK_START` lets **your own** function compute on the side while the
main loop keeps running:

```basic
DIM auftrag AS INTEGER
auftrag = TASK_START(BerechneKarte, 4242)

WHILE NOT TASK_READY(auftrag)
    ' in a game: FLIP() is here anyway, that already slows it down
    ' in a console program: SLEEP(10), otherwise the
    ' loop spins at 100 % of a core
    SLEEP(10)
WEND

PRINT TASK_RESULT$(auftrag)
```

**Do not wait without a pause.** `TASK_READY` only asks and returns
immediately — a loop around it therefore runs as fast as the machine
can and burns a core while the task is supposed to be computing next to it. In
a game, `FLIP()` slows the loop down to the frame rate anyway. In a
console program, put a `SLEEP(10)` in it.

| Function | Purpose |
|---|---|
| `TASK_START(funktion[, arg1, arg2, …])` → INTEGER | start, returns the task number |
| `TASK_READY(auftrag)` → BOOLEAN | is the result there? |
| `TASK_RESULT$(auftrag)` → STRING | fetch the result (**once**) |
| `TASK_CANCEL(auftrag)` | discard the result |
| `TASK_PENDING()` → INTEGER | tasks that have not been fetched yet |

You write the function **without parentheses**: `TASK_START(BerechneKarte, 42)`.

**A task runs as a separate process** — not as a thread. Three
things follow from that which you need to know:

**1. A task does not see your globals.** Not even a `CONST` at the top
level. The main program does not run in the task process at all:

```basic
CONST FAKTOR AS INTEGER = 10

FUNCTION Nutzt() AS INTEGER
    RETURN FAKTOR        ' error when started as a task
END FUNCTION
```

Pass the function what it needs as parameters. The message explains this
when it happens. The same limit applies to a module imported with `AS` —
and there, as here, it is the real gain: the function no longer depends
on whatever happens to be at the top of the program.

**2. Arguments and result are a number, text or BOOLEAN.** You may
pass as many as you like — `TASK_START(Zeichne, 3, "rot", TRUE)` —, but an
object or an array handle cannot cross a process boundary. Whoever needs more
passes JSON through. A text with spaces stays **one** argument.
The result always comes back as a STRING; for a number, therefore,
`VAL(TASK_RESULT$(a))`.

**3. A start costs about 12 ms.** For "compute on the side while the loop
spins" that goes unnoticed. For a thousand small tasks per second it is
the wrong command — then compute in the main program.

**`TASK_PENDING` counts what has not been fetched yet** — not what is still computing.
A finished task that has not been fetched counts too. That is why

```basic
WHILE TASK_PENDING() > 0      ' waits forever if fetching only happens afterwards
WEND
```

is wrong: ask per task with `TASK_READY`. The same applies to
`SHELL_PENDING` and `DB_QUERY_PENDING`.

What the task itself outputs with `PRINT` does **not** come back — a
task computes, it does not talk. Example:
[examples/170_task.dh](../../examples/170_task.dh).


## Opening a file: `OPENDOC`

`OPENDOC(pfad$)` opens a **local file with its default program** —
the written PDF in the viewer, the spreadsheet in the spreadsheet application.
The counterpart to `OPENURL`, which deliberately only allows `http://` and `https://`.
Limited to document and image extensions (pdf, png, jpg, txt, csv,
json, xml, html, xlsx, docx, …): `.exe` and the like are an error, not a launch.
For programs there is `SHELL`, deliberately kept separate.

## A second window: `WINDOW_OPEN`

Drachenhauch knows **one OS window per process** (raylib keeps its state
globally; the trade-off is described in [entwurf-native-fenster.md](../entwurf-native-fenster.md) (German)).
A second window is therefore a **second `dhrt`** that runs its own
program with its own `SCREEN` — with its own entry in the taskbar,
on any monitor, and a crash there does not take the first one down with it. The two
are connected by a **line-based text channel**:

```basic
' in the main program
DIM k AS INTEGER
k = WINDOW_OPEN("rechnung_fenster.dh", "rechnungen.db", 7)   ' arguments as on the command line
WINDOW_SEND(k, "neu")
DIM post AS STRING : post = WINDOW_RECV$(k)                   ' "" if nothing is there
IF NOT WINDOW_ALIVE(k) THEN PRINT "Fenster ist zu"

' in the child program (rechnung_fenster.dh)
DIM id AS INTEGER : id = VAL(ARG$(1))
IF PARENT_ALIVE() THEN PARENT_SEND("geaendert " + STR$(id))
IF PARENT_RECV$() = "neu" THEN laden()
```

| Function | Purpose |
|---|---|
| `WINDOW_OPEN(datei$[, arg1, …])` → INTEGER | start a second `dhrt run datei$`; numbers, texts, BOOLEAN become command-line arguments (`ARGC`/`ARG$`) |
| `WINDOW_SEND(fenster, text$)` | send one line — anything sent before the connection is delivered as soon as the child calls `PARENT_*` |
| `WINDOW_RECV$(fenster)` → STRING | next line from the child or `""` |
| `WINDOW_ALIVE(fenster)` → BOOLEAN | is the process still running? |
| `WINDOW_CLOSE(fenster)` | end the process |
| `PARENT_SEND(text$)` / `PARENT_RECV$()` | in the child: to the parent / from the parent |
| `PARENT_ALIVE()` → BOOLEAN | in the child: is there a parent, and is it alive? `FALSE` if the program was started on its own — that way the same program also runs without a parent |

What you need to know:

- **No shared state.** An `IMAGE`, an array, an object belongs to one
  process. Whatever needs to go across becomes text — JSON if it needs structure. A
  document lives in a file or database that both open. The same
  limit as with `TASK_START`, for the same reason.
- **A message is one line.** A line break in it is an error.
  The queue holds 1024 lines, the oldest one drops out — whoever does not
  read does not block the other side.
- **If the connection drops, the child exits.** A window without a
  main program would be a zombie nobody closes any more. If the
  main program ends normally, it takes its windows with it.
- **Times, measured:** ~0.4 s until the child shows its first frame (a
  process start). 1000 messages in one burst there and back: ~60 ms. A
  **single** round trip costs one to three frames of the child (17–50 ms at
  60 Hz), because the child reads its mail once per frame — that is the design,
  not the channel; whoever needs it faster sets `SETFPS(0)` in the child.
- The channel is a TCP connection on `127.0.0.1` (port in the environment variable
  `DHRT_ELTERN_PORT`), not stdin/stdout — the child may keep using `PRINT`.
  The web build has no processes; there `WINDOW_OPEN` is an
  error.

Example: the invoice manager `examples/196_rechnungen.dh` opens an invoice with
Ctrl+F in `examples/196_rechnung_fenster.dh`; both work
on the same SQLite file, the child reports `geaendert <id>`, the manager
sends `neu` when it has saved itself.

## Sets

A set is a `MAP OF INTEGER` whose values nobody cares about —
not a type of its own. The commands spare you the `STR$()` detour and the
dummy value:

```basic
DIM gesehen AS MAP OF INTEGER

SET_ADD(gesehen, id)
IF SET_HAS(gesehen, id) THEN PRINT "kenne ich schon"
PRINT SET_SIZE(gesehen)
```

| Function | Purpose |
|---|---|
| `SET_ADD(menge, wert)` | add; already in it = no effect |
| `SET_HAS(menge, wert)` → BOOLEAN | membership |
| `SET_REMOVE(menge, wert)` → BOOLEAN | remove; `TRUE` if it was in it |
| `SET_SIZE(menge)` → INTEGER | number of elements |
| `SET_ITEMS(menge)` → ARRAY | elements in insertion order |
| `SET_CLEAR(menge)` | remove all elements |

**A set holds one kind of element.** Elements may be `INTEGER` **or**
`STRING`, but not mixed — the first addition fixes the kind,
every later deviation reports an error:

```basic
SET_ADD(m, 5)
SET_ADD(m, "5")     ' error: this set contains numbers
```

This is on purpose. Internally keys are always strings; without this rule
`5` and `"5"` would land on the same entry, and afterwards the set would have one
element instead of two — silently and without a hint. `SET_HAS` checks the kind too:
`SET_HAS(zahlen, "5")` is a typo, not a question, and silently returning `FALSE`
would be the least friendly answer to it.

That is also how `SET_ITEMS` knows what it has to return: `ARRAY OF INTEGER`
for a number set, `ARRAY OF STRING` for a text set. After
`SET_CLEAR` the kind is open again.

**The order is guaranteed**, not random: `SET_ITEMS` returns the
elements in the order they were added. That makes output reproducible.

Because a set remains a MAP, `MAPSIZE`, `MAPKEYS` and the
other map commands keep working on it — useful for looking inside, but `MAPPUT`
with a value of your own bypasses the element-kind check. Whoever wants a set
sticks to the `SET_` commands.

## CSV

The most common data exchange of all — and impossible to do properly with `SPLIT$`.
As soon as a field contains the separator, `SPLIT$` returns too many
fields and says nothing about it:

```text
Mueller;"Berlin; Mitte";42      SPLIT$(";") -> vier Felder statt drei
```

| Function | Purpose |
|---|---|
| `CSV_PARSE(text$[, trenner$])` → ARRAY OF STRING | split text, 2D (rows × columns) |
| `CSV_LOAD(pfad$[, trenner$[, kodierung$]])` → ARRAY OF STRING | read a file, 2D |
| `CSV_FORMAT$(tabelle[, trenner$])` → STRING | 2D array as CSV text |
| `CSV_SAVE(pfad$, tabelle[, trenner$[, kodierung$]])` | 2D array into a file |
| `CSV_ROW$(felder[, trenner$])` → STRING | a single row from a 1D array |

```basic
DIM t AS ARRAY OF STRING
t = CSV_LOAD("kunden.csv", ";")

PRINT DIMSIZE(t, 0)     ' rows
PRINT DIMSIZE(t, 1)     ' columns
PRINT t[1, 2]           ' second row, third column
```

**Separator:** without a second argument, a comma. In the German
locale Excel writes `;` — then `CSV_LOAD(pfad$, ";")`. It must be exactly **one** character;
anything else reports an error instead of silently taking the first
one.

**Quotation marks** per RFC 4180: a field may contain separators, line breaks
and quotation marks if it is enclosed in `"`; a `"` in the field is
doubled. When writing, Drachenhauch sets quotation marks **only where they are
needed** — unnecessary ones make the file unreadable and the diff bigger.

**Rows of unequal length** are padded to the *widest* one (with
empty strings), because a DH array must be rectangular. Cutting off would throw data
away without saying so.

**Broken files** do not abort the import: a missing
closing quote reads to the end of the file. `\r\n` and `\n` both count
as a line end, a BOM at the start of the file is cut off (otherwise
the first column would forever be called `﻿Name`).

Example: [examples/169_csv.dh](../../examples/169_csv.dh).

### Searching, tidying up, temporary storage

**Name patterns** know `*` (any number of characters) and `?` (exactly one) —
and **always** ignore upper/lower case. Windows does not distinguish
in file names, Linux does; if this were platform-dependent, the same
program would behave differently on two machines.

```basic
DIM tabellen AS ARRAY OF STRING
tabellen = DIRLIST("daten", "*.csv")          ' this folder only
tabellen = DIRLIST_REC("daten", "*.csv")      ' all subfolders too
```

`DIRLIST_REC` returns **only files** (folders are not payload) as paths
relative to the start folder, always separated by `/` — on Windows too, so that
a saved list looks the same on both systems. The pattern applies
to the file name, not to the whole path.

**Without a second argument, `RMDIR` deletes only an EMPTY directory.** A
call that accidentally wipes out a whole tree is the most expensive
typo a file command can cause — deleting the contents too has to be
written out:

```basic
RMDIR("bau")              ' error if anything is still inside (and says so)
RMDIR("bau", TRUE)        ' with everything inside it
```

**`FILETIME`** counts in the same time reckoning as the module
[`zeit`](module-zeit.md) — seconds since 1970 in local time. That is exactly why
the question it is all about works:

```basic
IMPORT "zeit"
IF ZEIT_JETZT() - FILETIME("sicherung.zip") > 86400 THEN
    PRINT "Die Sicherung ist älter als ein Tag."
END IF
```

**`TEMPFILE$` creates the file immediately, empty**, instead of just making up
a name: otherwise a second run could get the same name between "name made up" and
"file written". Tidying up remains the
program's job (`DELETEFILE`).

## Text encoding

Drachenhauch reads and writes text files in **UTF-8**. That is the right
default — except that Excel on a German Windows, when exporting as "CSV
(comma delimited)", writes **Windows-1252**, and such a file used to be
unreadable altogether:

```text
READLINES("kunden.csv")  ->  stream did not contain valid UTF-8
```

All text readers and writers therefore accept an encoding as their **last argument**:

```basic
DIM t AS ARRAY OF STRING
t = CSV_LOAD("kunden.csv", ";", "cp1252")     ' from Excel
t = READLINES("alt.txt", "latin1")

WRITEALL("fuer_excel.csv", inhalt, "cp1252")  ' so that Excel likes it again

' Line by line (for large files) the specification belongs on OPENFILE:
DIM f AS FILE
f = OPENFILE("gross.csv", "r", "cp1252")
```

| Name | also written as |
|---|---|
| `utf8` (default) | `utf-8` |
| `cp1252` | `windows-1252`, `ansi` |
| `latin1` | `iso-8859-1` |

Upper/lower case, hyphens and spaces do not matter — whoever specifies an
encoding has usually copied it from somewhere.

**Five things you should know:**

1. **Without a specification everything stays as before** (UTF-8). No existing program
   changes its behaviour.
2. **The error message now names the way out** — file, line, the offending
   byte and the hint at `cp1252`. Before, it was a passed-through
   Rust text that said neither what was wrong nor what you could do.
3. **`latin1` can never fail** (every byte is a character), and neither can `cp1252`
   — the five byte values that Windows-1252 officially leaves unassigned
   are mapped to their own code point, just as every browser does.
   Whoever reads in an old file wants to read it and not fail on a
   control character.
4. **When WRITING, a missing character is an error**, not a `?`. `latin1`
   has no euro sign; an invoice in which it silently turns into a
   question mark is worse than one that never gets created at all.
   (`cp1252` has the euro sign — that is exactly why both exist.)
5. **A BOM at the start of the file is always dropped**, in every text reader. Otherwise
   the first column would forever be called `﻿Name`.

**Not included: UTF-16.** Excel writes that for "Unicode Text (*.txt)". It
needs BOM detection, two byte orders and surrogate pairs — a decision of its own,
not an addendum to this one.

## File I/O

| Function | Purpose |
|---|---|
| `OPENFILE(pfad$, modus$[, kodierung$])` → FILE | modes: `"r"` read, `"w"` write anew, `"a"` append |
| `CLOSEFILE(f)` | closes |
| `READLINE(f)` → STRING | next line (without `\n`); at the end of the file `""` like an empty line, so ask `EOF(f)` |
| `EOF(f)` → BOOLEAN | TRUE when there is nothing more to read from the file: `WHILE NOT EOF(f) : PRINT READLINE(f) : WEND` |
| `READALL$(f)` → STRING | read the whole rest |
| `ENDOFFILE(f)` → BOOLEAN | at the end? |
| `WRITELINE(f, text$)` | writes + `\n` |
| `WRITE(f, text$)` | writes without `\n` |
| `FILEEXISTS(p$)` → BOOLEAN | file present? |

Path-based, without a FILE handle *(native runtime only)*:

| Function | Purpose |
|---|---|
| `WRITEALL(pfad$, text$[, kodierung$])` | write text completely (overwrites/creates) |
| `READLINES(pfad$[, kodierung$])` → ARRAY OF STRING | read a file as an array of lines |
| `FILESIZE(pfad$)` → INTEGER | size in bytes |
| `DELETEFILE(pfad$)`, `RENAME(alt$, neu$)` | delete / rename·move |
| `DIREXISTS(pfad$)` → BOOLEAN | directory present? |
| `DIRLIST(pfad$[, muster$])` → ARRAY OF STRING | entry names (sorted), optionally filtered |
| `DIRLIST_REC(pfad$[, muster$])` → ARRAY OF STRING | as above, but recursive — files only, relative paths with `/` |
| `RMDIR(pfad$[, mit_inhalt])` | delete a directory (empty; with `TRUE` including its contents) |
| `FILETIME(pfad$)` → INTEGER | last modified, seconds as in the module `zeit` |
| `TEMPDIR$()` → STRING | the system's temp directory |
| `TEMPFILE$([praefix$[, endung$]])` → STRING | a free name in the temp directory, **created empty** |
| `MKDIR(pfad$)` | create a directory (including parents) |
| `COPYFILE(src$, dst$)` | copy a file |
| `APPENDFILE(pfad$, text$[, kodierung$])` | append text at the end (creates the file) |
| `PATHJOIN(a$, b$, …)` → STRING | join path parts with `/` |
| `BASENAME(pfad$)` → STRING | last path component (file/folder name) |
| `DIRNAME(pfad$)` → STRING | directory part (without the last component) |
| `REALPATH$(pfad$)` → STRING | the full, cleaned-up path: absolute, `.`/`..` resolved, in the spelling on disk; if the file does not exist, only made absolute and cleaned up |
| `SAMEFILE(a$, b$)` → BOOLEAN | do both paths mean the same file? A text comparison goes wrong with `/` versus `\`, with a `..` in the path, with short 8.3 names and (on Windows) with upper/lower case |

```basic
' Writing
DIM out AS FILE
out = OPENFILE("scores.txt", "w")
WRITELINE(out, "Anna 95")
WRITELINE(out, "Bert 78")
CLOSEFILE(out)

' Reading
IF FILEEXISTS("scores.txt") THEN
    DIM inp AS FILE
    inp = OPENFILE("scores.txt", "r")
    WHILE NOT ENDOFFILE(inp)
        PRINT READLINE(inp)
    WEND
    CLOSEFILE(inp)
END IF

' Path-based (native runtime)
MKDIR(PATHJOIN("saves", "level1"))
WRITEALL(PATHJOIN("saves/level1", "progress.txt"), "score=42")
DIM zeilen AS ARRAY OF STRING
zeilen = READLINES(PATHJOIN("saves/level1", "progress.txt"))
PRINT FILESIZE(PATHJOIN("saves/level1", "progress.txt"))
```

## Bytes (BUFFER)

`STRING` is **UTF-8 text**. It cannot carry every byte sequence at all, and `LEN`
counts characters in it, not bytes. As soon as it is about *data* rather than text —
custom file formats, images, protocols, checksums — a second
type is needed. That is `BUFFER`: a mutable sequence of bytes.

```basic
DIM b AS BUFFER
b = BUFFER_NEW(4)          ' 4 bytes, all 0
BUFFER_SET(b, 0, 222)
PRINT BUFFER_TO_HEX$(b)    ' de000000
```

`BUFFER` needs **no `IMPORT`** and is a **reference type** like `ARRAY`:
pass it to a `SUB` and both sides share the same bytes.

### Basics

| Function | Purpose |
|---|---|
| `BUFFER_NEW(groesse)` → BUFFER | new buffer, filled with zeros |
| `BUFFER_LEN(b)` → INTEGER | length in **bytes** |
| `BUFFER_GET(b, pos)` → INTEGER | read a byte 0..255 |
| `BUFFER_SET(b, pos, byte)` | write a byte 0..255 (modifies in place) |
| `BUFFER_FILL(b, byte)` | fill everything with one byte |
| `BUFFER_RESIZE(b, groesse)` | grows with zeros, shrinks by cutting off |
| `BUFFER_SLICE(b, von, bis)` → BUFFER | **copy** of the bytes `[von, bis)` |
| `BUFFER_CONCAT(a, b)` → BUFFER | new buffer made of both |
| `BUFFER_INDEXOF(b, nadel [, ab])` → INTEGER | first occurrence, otherwise `-1` |

As with arrays: **an index out of range is an error, a slice clamps.**
`BUFFER_GET(b, 99)` on a 4-byte buffer throws; `BUFFER_SLICE(b, 0, 99)`
simply returns the 4 bytes that are there.

A byte outside 0..255 is an error too and is **not** silently
trimmed — otherwise it would only show up in the finished output file.

### Text, hex and Base64

| Function | Purpose |
|---|---|
| `BUFFER_FROM_STRING(text$)` → BUFFER | text as UTF-8 bytes |
| `BUFFER_TO_STRING$(b)` → STRING | bytes as UTF-8 text |
| `BUFFER_TO_HEX$(b)` / `BUFFER_FROM_HEX(s$)` | hex text (`"deadbeef"`) |
| `BUFFER_TO_BASE64$(b)` / `BUFFER_FROM_BASE64(s$)` | Base64 |

`BUFFER_TO_STRING$` is **strict**: if the bytes are not valid UTF-8, there is
an error instead of silently replaced characters — a `?` in the wrong
place falsifies the data and is noticed only much later. For data that is not
meant to be text at all, `BUFFER_TO_HEX$` is the right tool.

`BUFFER_FROM_HEX` allows spaces (`"de ad be ef"`), because hex dumps
are usually written in groups.

`BUFFER_FROM_BASE64` returns **raw bytes** — unlike `BASE64_DECODE`,
which demands valid UTF-8 and throws otherwise.

### Compressing

| Function | Purpose |
|---|---|
| `BUFFER_DEFLATE(b [, format$])` → BUFFER | compress bytes with Deflate (`"zlib"` default, `"roh"` = raw) |
| `BUFFER_INFLATE(b [, format$])` → BUFFER | decompress compressed bytes (`"zlib"` default, `"roh"` = raw) |

Unlike `COMPRESS$`/`DECOMPRESS$` (text in, Base64 out), these two
work on **raw bytes**. The default `"zlib"` is the format with a header and
checksum in which Deflate sits in PDF pages (`/FlateDecode`), PNG images and
many network protocols; `"roh"` is the bare stream (that is how it lies in
ZIP archives). Broken data is an error; at most 512 MB are decompressed,
so that a few kilobytes cannot blow up memory.

### Packing numbers

| Function | Purpose |
|---|---|
| `BUFFER_GET_I16/U16/I32/U32/I64(b, pos [, reihenfolge$])` → INTEGER | read an integer |
| `BUFFER_GET_F32/F64(b, pos [, reihenfolge$])` → FLOAT | read a floating-point number |
| `BUFFER_SET_I16/U16/I32/U32/I64(b, pos, wert [, reihenfolge$])` | write an integer |
| `BUFFER_SET_F32/F64(b, pos, wert [, reihenfolge$])` | write a floating-point number |

`reihenfolge$` is `"le"` (little-endian, **default**) or `"be"`
(big-endian):

```basic
DIM b AS BUFFER
b = BUFFER_NEW(8)
BUFFER_SET_I32(b, 0, 1000)          ' e8030000
BUFFER_SET_I32(b, 4, 1000, "be")    ' 000003e8
PRINT BUFFER_TO_HEX$(b)             ' e8030000000003e8
```

Whoever writes a buffer and reads it back themselves can ignore the default
— both sides use the same one. Only those who serve a **foreign**
format need the specification: PNG, ZIP and most network protocols are big-endian.

A value that does not fit the width is an error
(`BUFFER_SET_U16(b, 0, 70000)`). Silently cut off, a completely different
number would come back out.

### Binary files

| Function | Purpose |
|---|---|
| `READALL_BYTES(pfad$)` → BUFFER | the whole file as bytes |
| `WRITEALL_BYTES(pfad$, b)` | bytes into a file (overwrites) |
| `READ_BYTES(datei, anzahl)` → BUFFER | up to `anzahl` bytes from the handle |
| `WRITE_BYTES(datei, b)` | bytes to the handle |
| `SEEK(datei, position)` | set the position (0 = start) |
| `TELL(datei)` → INTEGER | current position |

```basic
' Piece by piece through a large file without loading it fully into memory
DIM f AS FILE
DIM stueck AS BUFFER
DIM gesamt AS INTEGER
f = OPENFILE("gross.bin", "r")
REPEAT
    stueck = READ_BYTES(f, 65536)
    gesamt = gesamt + BUFFER_LEN(stueck)
UNTIL BUFFER_LEN(stueck) = 0
CLOSEFILE(f)
PRINT gesamt
```

**At the end of the file, `READ_BYTES` returns less than requested** — down to
nothing at all. That is not an error but the usual stop condition.

> **There are no separate binary modes `"rb"`/`"wb"`.** Drachenhauch files are
> always byte-exact: there is no CRLF translation and no Ctrl-Z-as-end-of-file
> as in old BASICs. Separate modes would pretend a difference that
> does not exist — `READ_BYTES`/`WRITE_BYTES`/`SEEK` work on the same
> handles from `OPENFILE(pfad, "r"/"w"/"a")` as `READLINE`/`WRITELINE`.
>
> What does not exist (yet): a mode that reads and writes **at the same time**.
> Whoever wants to change a file at one place reads it with
> `READALL_BYTES`, changes the buffer and writes it back with `WRITEALL_BYTES`.

## Operating system

This turns a program into a **tool**: it accepts arguments,
reads its environment, calls other programs and tells its caller whether it
worked.

| Function | Purpose |
|---|---|
| `ARGC()` → INTEGER | number of arguments for this program |
| `ARG$(n)` → STRING | argument no. `n` (0-based); out of range → `""` |
| `GETENV$(name$ [, vorgabe$])` → STRING | read an environment variable |
| `SETENV(name$, wert$)` | set an environment variable (this program + its children) |
| `CWD$()` → STRING | current working directory |
| `CHDIR(pfad$)` | change the working directory |
| `EXEPATH$()` → STRING | path of the running program file |
| `EXIT([code])` | end immediately, `code` = return value (0..255, default 0) |
| `EPRINT(text)` | line to **stderr** instead of stdout |
| `SHELL(programm$, ...)` → INTEGER | start a program, wait, return value |
| `SHELL_OUT$(programm$, ...)` → STRING | like `SHELL`, but collects the output |
| `STDIN([kodierung$])` → FILE | standard input as a file handle |

```basic
' A tool that expects a file
DIM pfad AS STRING
IF ARGC() < 1 THEN
    EPRINT("Verwendung: zaehle <datei>")
    EXIT(2)
END IF
pfad = ARG$(0)
IF NOT FILEEXISTS(pfad) THEN
    EPRINT("Nicht gefunden: " + pfad)
    EXIT(1)
END IF
PRINT LEN(READLINES(pfad))
```

**Where the arguments come from** — the difference matters:

| Call | What the program sees |
|---|---|
| `dhrt run werkzeug.dh -- a b` | `a`, `b` |
| `dhrt run werkzeug.dh a b` | **nothing** |
| `werkzeug.exe a b` (exported) | `a`, `b` |

When started via `dhrt`, everything after a standalone `--` belongs to the
program, everything before it to the runtime. Without `--` the program gets no
arguments. This is on purpose: otherwise `dhrt` could never add a switch of its own
without breaking existing programs. The **exported `.exe`**
is the program itself — there is nothing to separate there, all arguments
belong to it, without `--`.

**`EXIT` is not an error.** It ends the program immediately and is **not** caught by
`TRY`/`CATCH` — an `EXIT` in the middle of a `TRY` block really
runs out and does not land in the `CATCH`. Values outside 0..255 are
an error instead of being silently truncated (the operating system only transmits the
lower byte; `EXIT(256)` would otherwise quietly turn into "all good").

**`EPRINT` is a built-in, not a statement** — so with parentheses
(`EPRINT("text")`, not `EPRINT "text"` as with `PRINT`).

### Standard input: `dir | meinwerkzeug | sort`

`ARG$` reads what the program is *given*. `STDIN()` reads what is
*handed* to it — and for that it brings no new reading vocabulary, but an
entirely ordinary **FILE handle**:

```basic
DIM f AS FILE
DIM z AS STRING
f = STDIN()
WHILE NOT ENDOFFILE(f)
    z = READLINE(f)
    PRINT UPPER$(z)
WEND
```

So `READLINE`, `READALL$`, `ENDOFFILE` and `READ_BYTES` apply unchanged,
and so does the encoding from the section above: `STDIN("cp1252")`.

**There is exactly ONE handle.** Every call of `STDIN()` returns the same one —
two would be two read buffers on the same pipe: one reads ahead, the
other misses the lines. A second call with a *different* encoding is
therefore an error and not silently ineffective.

**`INPUT` and `STDIN()` can be mixed.** Both hang off the same buffer,
so no line is lost when a program first asks for something and
then passes the rest through.

**`SEEK` and `TELL` do not work here** — a pipe cannot be
rewound, and both say so instead of trying.

**`INPUT` remains the tool for humans.** It prints its prompt (the
editor console needs that) and at the end of input returns empty strings
endlessly — it cannot report the end. Whoever processes a data stream
uses `STDIN()`; there `ENDOFFILE` is the stop condition.

A complete tool that can do *both* — files as arguments **or**
standard input, the way `wc` and `grep` do — is in
[examples/172_filter.dh](../../examples/172_filter.dh).

**`SHELL` takes the arguments one by one**, not as a command line:

```basic
SHELL("git", "commit", "-m", "Nachricht mit Leerzeichen")   ' correct
```

That way there are no quoting rules to learn, and a file name with spaces
does not fall apart into two arguments. Whoever really needs a shell (pipes,
redirections) calls it explicitly — `SHELL("cmd", "/c", "dir | more")` —
and is then subject to its own quoting rules.

`SHELL` passes the child program's output straight through to the console;
`SHELL_OUT$` collects its **stdout** and returns it as a STRING, while
its **stderr** stays stderr — otherwise error messages would mix unnoticed
into the payload.

### Where is the program itself?

`CWD$()` says **from where** work is done; `EXEPATH$()` says **what is
running** — under `dhrt run spiel.dh`, therefore, the runtime; in a game built with
`dhrt --export`, its own `.exe`. These are
different questions, and for two things the second one matters:

```basic
' Start the same runtime once more -- SHELL does not know "dhrt" as the name
' of the running runtime, PROCESS_START does.
PRINT SHELL_OUT$(EXEPATH$(), "run", "helfer.dh")

' Look for a fixture file NEXT TO the exe instead of in the start directory
DIM daten AS STRING
daten = PATHJOIN(DIRNAME(EXEPATH$()), "spielstand.json")
```

### A program in the background

`SHELL` and `SHELL_OUT$` wait until the child program has finished. If that takes
longer than one frame, everything stands still. For this there is the same thing
to poll:

| Function | Effect |
|---|---|
| `SHELL_START(programm$, ...)` → INTEGER | starts in the background, returns the task number |
| `SHELL_READY(auftrag)` → BOOLEAN | has the process finished? |
| `SHELL_RESULT$(auftrag)` → STRING | fetch stdout, free the slot |
| `SHELL_CODE()` → INTEGER | return value of the **most recently fetched** task |
| `SHELL_ERR$()` → STRING | its stderr |
| `SHELL_CANCEL(auftrag)`, `SHELL_PENDING()` | discard / count |

```basic
DIM auftrag AS INTEGER
DIM ausgabe AS STRING
auftrag = SHELL_START("git", "log", "--oneline")

WHILE NOT SHELL_READY(auftrag)
    ' ... keep working here, draw, listen for keys ...
    SLEEP(1)
WEND
ausgabe = SHELL_RESULT$(auftrag)
PRINT SHELL_CODE()
```

`SHELL_CODE()` and `SHELL_ERR$()` take **no** argument: they belong to the
most recently fetched task — the same pattern as `HTTP_STATUS()` for the most recently
fetched response. A program that does not even start reports this on
**fetching**, not on starting.

### A program with live output

`SHELL_START` delivers the output only at the end. A development environment
has to show what a program prints *while* it runs, and send it input
— for that there are processes with a pipe in both directions:

| Function | Effect |
|---|---|
| `PROCESS_START(programm$, ...)` → INTEGER | starts with open input and output; `"dhrt"` as the program means this runtime itself. An array of texts among the arguments is spread out (`PROCESS_START("dhrt", "run", datei$, "--", argumente)`), a MAP of texts gives the child additional environment variables (only it, not your own program) |
| `PROCESS_READ$(prozess)` → STRING | what has arrived on stdout since the last call (empty = nothing new) |
| `PROCESS_ERR$(prozess)` → STRING | the same for stderr |
| `PROCESS_WRITE(prozess, text$)` | send to the input — an `INPUT` in the child needs the line ending `CHR$(10)` |
| `PROCESS_CLOSE_INPUT(prozess)` | close the input (end of file for the child) |
| `PROCESS_RUNNING(prozess)` → BOOLEAN | is it still running? |
| `PROCESS_CODE(prozess)` → INTEGER | return value; `-1` while it is running or if it was aborted |
| `PROCESS_FRONT(prozess)` | brings the child's visible window to the front (Windows; TRUE if there was one). A program in the background may not bring itself to the front there — only whoever is currently in front may do so; without a window the child gets permission for its first one. Elsewhere FALSE |
| `PROCESS_KILL(prozess)`, `PROCESS_CLOSE(prozess)` | end it (including everything the child started itself — `cmd /C ping ...` also ends the `ping`) / free the slot |

```basic
DIM p AS INTEGER
p = PROCESS_START("dhrt", "run", "spiel.dh")
WHILE PROCESS_RUNNING(p)
    DIM t AS STRING : t = PROCESS_READ$(p)
    IF t <> "" THEN PRINT t;          ' show it as soon as it arrives
    SLEEP(16)
WEND
PRINT "beendet mit " ; PROCESS_CODE(p)
PROCESS_CLOSE(p)
```

The output arrives in chunks, not in lines — whoever needs lines collects
up to the next `CHR$(10)` (that is what the IDE does in `ide/ide.dh`). A child
does **not** inherit `DHRT_FRAMES` and the screenshot variables, otherwise a
program started from the IDE would die after the frames of the IDE test. When the
parent program ends, running children are terminated.

### Language services for editors: `CODE_*`

The same functions that `dhrt lsp` offers a foreign editor via LSP —
here without a second process and without JSON-RPC, for an IDE written in Drachenhauch.
Lines and columns count from 1, in characters.

| Function | Returns | Effect |
|---|---|---|
| `CODE_CHECK$(quelltext$[, basis$])` | STRING (JSON) | the front-end chain like `dhrt --check`: a list of `{zeile, spalte, laenge, schwere, meldung}` with `schwere` = `fehler`/`warnung`; `spalte` (from 1) and `laenge` say where in the line the message belongs -- the location of a syntax error, otherwise a name mentioned in the message, otherwise the line without indentation; `basis$` = folder for `IMPORT "x.dh"` (default: the working directory). Lines are those of the buffer, even with imports |
| `CODE_HOVER$(quelltext$, zeile, spalte)` | STRING | signature and description of the word at that position as Markdown; empty if there is no word there |
| `CODE_COMPLETE(quelltext$, zeile, spalte)` | ARRAY OF STRING | suggestions for the word beginning to the left of the position: own symbols, commands, keywords, constants |
| `CODE_DEFINITION(quelltext$, zeile, spalte)` | TUPLE (zeile, spalte) | where the word is defined; `(-1, -1)` if there is no definition in this text |
| `CODE_REFERENCES(quelltext$, zeile, spalte)` | ARRAY OF INTEGER | the lines of all occurrences |
| `CODE_TYPES$(quelltext$[, basis$])` | STRING (JSON) | type hints for an editor: for each place without a written type -- today the variables of `FOR EACH` -- `{"zeile", "name", "typ"}` (line from 1; `MAP` yields keys as STRING, in the pair form plus the value); what the compiler does not know is left out |
| `CODE_NAMES(quelltext$)` | MAP OF INTEGER | how often each name is used (lower case, including `$`): without comments and strings and without the place where a SUB/FUNCTION defines it -- calls, FUNCREFs and methods after a dot count |
| `CODE_SYMBOLS$(quelltext$)` | STRING (JSON) | the outline: `{name, art, von, bis, kinder}` — classes with their methods; `art` is `class`, `struct`, `sub`, `function`, `property` or `enum` |
| `CODE_FORMAT$(quelltext$ [, einruecken])` | STRING | the source formatted as by `dhrt fmt`: keywords in upper case, trailing whitespace removed, with `einruecken` (default TRUE) the blocks re-indented; empty if the source cannot be read (syntax error) |
| `CODE_RENAME$(quelltext$, zeile, spalte, neu$)` | STRING | the source with the name at this position renamed to `neu$` everywhere — whole words, without comments and strings. Empty if there is no name there, it is a keyword or `neu$` is not a name |

```basic
DIM j AS JSON_HANDLE
j = JSON_PARSE(CODE_CHECK$(GUI_TEXT(feld), DIRNAME(pfad)))
DIM i AS INTEGER
FOR i = 0 TO JSON_LEN(j, "") - 1
    PRINT JSON_GET_INT(j, STR$(i) + ".zeile") ; ": " ; JSON_GET_STRING(j, STR$(i) + ".meldung")
NEXT
```

> **`CWD$()` is not the directory you started from.** At startup `dhrt`
> changes into the directory of the `.dh` file (so that
> `LOADIMAGE("assets/…")` works from anywhere), the exported `.exe`
> into the exe directory. A path the user passes as an argument is
> therefore relative to *their* directory, not to `CWD$()` — when in doubt, ask the
> user for an absolute path.

## Checking and reporting

With these a program checks itself — and says afterwards whether it worked.

| Function | Purpose |
|---|---|
| `ASSERT(bedingung [, meldung$])` | fails if the condition is `FALSE` |
| `ASSERT_EQ(ist, soll [, was$])` | the same, the message shows **both** values |
| `ASSERT_COLLECT(an)` | collect mode on/off (default: off) |
| `ASSERT_COUNT()` → INTEGER | how many checks have run |
| `ASSERT_FAILED()` → INTEGER | how many of them failed |
| `ASSERT_REPORT()` → INTEGER | print the summary, return the number of failures |
| `LOG_DEBUG/INFO/WARN/ERROR(text)` | message with a timestamp to **stderr** |

### Two ways of checking

**Precondition in the running program** — here a failure is meant to *abort*.
That is the default:

```basic
ASSERT(spieler_zahl > 0, "ohne Spieler geht es nicht")
```

If it fails, the program ends with a runtime error including file and
line — just like any other error.

**Test program** — here you want to see *all* errors, not just the first one.
For that, switch on collect mode once at the start:

```basic
ASSERT_COLLECT(TRUE)

ASSERT_EQ(punkte(2, 1, 2, 1), 4, "exakt getroffen")
ASSERT_EQ(punkte(1, 0, 0, 1), 0, "falsche Tendenz")
ASSERT(tendenz(1, 0) > 0,       "Heimsieg")

IF ASSERT_REPORT() > 0 THEN
    EXIT(1)
END IF
```

Output on a failure:

```
FEHL  Zeile 4: falsche Tendenz: erhalten 2, erwartet 0     <- stderr
FEHLER: 1 von 3 Pruefungen                                 <- stdout
```

The **return value** is the point: only with it can a script, a Makefile
or a CI tell "ran through" apart from "found errors".

**Separation of stdout and stderr:** the failures go to stderr, the
summary to stdout. A `pruefung > bericht.txt` therefore yields a clean
report, while the details remain in the terminal.

> **`ASSERT` demands a `BOOLEAN`.** `ASSERT(anzahl)` is an error and
> not "true because not zero" — a check that accidentally always
> passes is worse than none at all. So write a comparison:
> `ASSERT(anzahl > 0)`.

> **`ASSERT_EQ` compares like the language's `=` operator**, including
> `1 = 1.0`. A second notion of when two values are equal would be
> the surest way to gamble away trust in the checks.

### Reporting

```basic
LOG_INFO("Saison 2026 geladen")
LOG_WARN("Kein Netz -- arbeite mit den gespeicherten Daten")
LOG_ERROR("Datenbank nicht lesbar")
```

Output: `20:45:43 INFO  Saison 2026 geladen` — to **stderr**, so that `PRINT`
can be passed through as payload.

How much of it appears is controlled by the environment variable **`DH_LOG`**:

| `DH_LOG` | what appears |
|---|---|
| `debug` | everything |
| *(not set)* / `info` | INFO, WARN, ERROR — **default** |
| `warn` | WARN, ERROR |
| `error` | ERROR only |
| `aus` | nothing |

So `LOG_DEBUG` stays silent until someone switches it on — debug messages can
stay in the code without cluttering normal operation:

```bash
DH_LOG=debug dhrt run werkzeug.dh
```

## Time & random

| Function | Purpose |
|---|---|
| `MILLIS()` → INTEGER | ms since program start (stopwatch) |
| `TIMER()` → FLOAT | the same clock in seconds |
| `TIME$()` → STRING | current time `"HH:MM:SS"` |
| `DATE$()` → STRING | current date `"YYYY-MM-DD"` |
| `VERSION$()` → STRING | the version of the runtime that is currently running (the same number as `dhrt --version`) |
| `RND()` → FLOAT | random number in `[0, 1)` |
| `RND(n)` → INTEGER | random INT in `[0, n)` |
| `RANDINT(lo, hi)` → INTEGER | random INT in `[lo, hi]` (inclusive) |
| `RANDF(lo, hi)` → FLOAT | random FLOAT in `[lo, hi)` |
| `CHOICE(array)` → T | random element of a 1D array |
| `WEIGHTED_CHOICE(werte, gewichte)` → T | element from `werte`, chosen in proportion to `gewichte` (1D arrays of equal length, weights ≥ 0). Loot tables. |
| `SHUFFLE(array)` | shuffles a 1D array IN PLACE (Fisher-Yates) |
| `RANDOMIZE([seed])` | set the random seed (without an argument: system seed; a decimal number like `TIMER()` works too) |

```basic
PRINT TIME$(), " - ", DATE$()

DIM t1 AS INTEGER
t1 = MILLIS()
DIM i AS INTEGER
DIM s AS FLOAT
s = 0.0
FOR i = 0 TO 100000
    s = s + SIN(i * 0.001)
NEXT
PRINT "Zeit: ", MILLIS() - t1, "ms"
' MILLIS is a stopwatch from program start, not a clock time: it starts at 0
' and runs on evenly, even when the system time jumps
' (daylight saving change, NTP). For date and time, ZEIT_JETZT() from the
' module "zeit" is responsible, which can also calculate with it.

' Reproducible dice
RANDOMIZE(42)
FOR i = 1 TO 5
    PRINT RND(6) + 1
NEXT
```

## Types & encoding

| Function | Purpose |
|---|---|
| `TYPEOF(x)` → STRING | runtime type name, e.g. `"INTEGER"`, `"STRING"`, `"VEC3"`, `"MAT4"`. For an instance the **class name** in upper case (`"HUND"`) |
| `ISNUM(x)`, `ISINT(x)`, `ISSTR(x)`, `ISBOOL(x)` → BOOLEAN | type predicates (a bool is NOT a number) |
| `BASE64_ENCODE(s$)` → STRING | Base64-encode UTF-8 text |
| `BASE64_DECODE(s$)` → STRING | Base64 to UTF-8 text (throws on invalid input) |
| `CRC32(s$)` → INTEGER | CRC-32 checksum of the UTF-8 bytes |
| `HASH(s$)` → INTEGER | stable 64-bit hash (FNV-1a) — save integrity, buckets |

```basic
PRINT TYPEOF(3.0)                       ' FLOAT
PRINT TYPEOF(NEW Hund())                ' HUND -- not "OBJECT"
PRINT ISINT(5), ISINT(3.0)              ' TRUE FALSE
PRINT BASE64_ENCODE("Hi!")              ' "SGkh"
PRINT BASE64_DECODE("SGkh")             ' "Hi!"
PRINT CRC32("hello")                    ' 907060870
```

## Checksums and identity

`CRC32` and `HASH` above are there for **recognition** — they say "probably
the same data". As soon as someone could *forge* the answer, they are not
enough: signatures, tokens, receipts, password derivations need something
else.

| Function | Purpose |
|---|---|
| `SHA256$(daten)` → STRING | SHA-256 as hex (64 characters) |
| `SHA1$(daten)` → STRING | SHA-1 — only for compatibility with existing things |
| `MD5$(daten)` → STRING | MD5 — only for compatibility |
| `SHA256_FILE$(pfad$)`, `SHA1_FILE$`, `MD5_FILE$` → STRING | the same for a file, read block by block |
| `HMAC_SHA256$(schluessel, daten)` → STRING | signature with a secret key |
| `SECURE_EQUALS(a, b)` → BOOLEAN | comparison in constant time |
| `UUID4$()` → STRING | random unique identifier |
| `RANDOM_BYTES(anzahl)` → BUFFER | random bytes from the operating system |

`daten` and `schluessel` may be `STRING` (then the UTF-8 bytes) or `BUFFER`
— a signature is formed over the bytes that are actually transmitted,
and for a file upload those are available as a `BUFFER`.

```basic
PRINT SHA256$("abc")
' ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad

PRINT SHA256_FILE$("grosses_archiv.zip")   ' reads block by block, no matter how large
```

### Verifying a signature

The case all of this exists for — a service sends data and a
signature, the program recalculates:

```basic
DIM erwartet AS STRING
erwartet = HMAC_SHA256$(geheimnis, nutzlast)

IF SECURE_EQUALS(erwartet, signatur_aus_der_kopfzeile) THEN
    PRINT "echt"
ELSE
    PRINT "gefälscht oder verändert"
END IF
```

> **`SECURE_EQUALS` instead of `=`.** An ordinary comparison stops at the first
> unequal character. Whoever wants to guess a signature measures the time and has
> it character by character after a few hundred attempts. `SECURE_EQUALS`
> always runs all the way through. That matters precisely for the comparison
> `HMAC_SHA256$` exists for in the first place.

### Randomness that is not a game

```basic
DIM token AS STRING
token = BUFFER_TO_HEX$(RANDOM_BYTES(32))
```

> **`RANDOM_BYTES` is not `RND`.** `RND` hangs off `RANDOMIZE`: the same seed
> gives the same sequence. For a dice game that is right and even
> desirable (reproducible levels) — for a password, a session key
> or a salt it would be a mistake. `RANDOM_BYTES` comes from the operating system's
> source of randomness and is not impressed by `RANDOMIZE`.

> **`MD5$` and `SHA1$` are considered broken.** They are here because you need them
> to play along — ETags, old checksum lists, git object names. For
> a security decision *of your own*, `SHA256$` is the answer.

## Game helpers

| Function | Purpose |
|---|---|
| `COLLIDES(x1, y1, w1, h1, x2, y2, w2, h2)` → BOOLEAN | AABB collision of two rectangles. Touching does not count (equal edges → FALSE). |

```basic
DIM held_x AS FLOAT
DIM held_y AS FLOAT
held_x = 100.0
held_y = 50.0

IF COLLIDES(held_x, held_y, 16, 16, 110, 60, 16, 16) THEN
    PRINT "Treffer!"
END IF
```

For more complex sprite collision see the [sprite module](module-sprite.md) with `SPRITE_COLLIDES`.

## Aliases & naming conventions

**BASIC aliases** (same behaviour, classic spelling — native runtime only):

| Alias | Canonical |
|---|---|
| `SGN(x)` | `SIGN(x)` |
| `SQRT(x)` | `SQR(x)` |
| `AUDIO_SET_VOLUME(ch, v)` | `AUDIO_VOLUME(ch, v)` |
| `AUDIO_MUSIC_SET_VOLUME(v)` | `AUDIO_MUSIC_VOLUME(v)` |

Container method `arr.join(trenner)` ≡ `JOIN$(arr, trenner)` (array OF STRING).

**Conventions / stumbling blocks** (deliberately NOT renamed — for clarification only):

- **`$` suffix only in the core.** String built-ins of the core language exist with and
  without `$` (`UPPER$` ≡ `UPPER`, `LEFT$` ≡ `LEFT`, …). **Module built-ins** carry
  no `$` (e.g. `JSON_GET_STRING`, not `JSON_GET_STRING$`).
- **Two sound APIs.** `PLAYSOUND`/`STOPSOUND` (core, simple) and the
  `audio` module (`AUDIO_PLAY`/`AUDIO_STOP`/channels/fades). `AUDIO_*` objects
  are compatible with `PLAYSOUND`. Both stay — `audio` is the more powerful one.
- **The suffix `2` is ambiguous** and context-dependent: `PHYSICS_DISTANCE2` =
  *squared* distance (faster), `LINE3D`/Vec2 functions mean *2D*, and
  `VEC2_*` is the type name. No uniform scheme — read it from the function name.
- **`SPRITE_COLLIDE` vs `SPRITE_COLLIDES`.** The `sprite` module uses
  `SPRITE_COLLIDES` (with `S`). Watch out for the core `COLLIDES` (AABB from raw values)
  — a different use case.
- **`CAMERA_X` (2D) vs `CAMERA3D_X`.** The 2D `camera` module and the native
  3D camera (`g3d`) have separate getters — do not mix them up.
