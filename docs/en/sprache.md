# Language reference

Drachenhauch is BASIC with Pascal-strict typing. Anyone who has written QBasic, GW-BASIC or Visual Basic will feel at home right away.

If you come from another BASIC or from Python: [Switching over](umstieg.md) lists what things are called differently.

## Contents

- [Variables and constants](#variables-and-constants)
- [Number literals](#number-literals)
- [Data types](#data-types)
- [What the compiler checks](#what-the-compiler-checks)
- [ENUM](#enum)
- [Compound assignment](#compound-assignment)
- [String interpolation (f-strings)](#string-interpolation-f-strings)
- [Operators](#operators)
- [Strings](#strings)
- [Control flow: IF / ELSE](#control-flow-if--else)
- [SELECT CASE](#select-case)
- [Loops: FOR and WHILE](#loops-for-and-while)
- [BREAK and CONTINUE](#break-and-continue)
- [DATA / READ / RESTORE](#data--read--restore)
- [Line continuation](#line-continuation)
- [Statement separator](#statement-separator)
- [Functions: SUB and FUNCTION](#functions-sub-and-function)
- [Named arguments](#named-arguments)
- [Functions as values: FUNCREF and lambdas](#functions-as-values-funcref-and-lambdas)
- [Coroutines: YIELD](#coroutines-yield)
- [Arrays](#arrays)
- [Maps](#maps)
- [Classes and structures](#classes-and-structures)
- [Try / Catch / Throw](#try--catch--throw)
- [Import](#import)
- [Foreign libraries: DECLARE … LIB](#foreign-libraries-declare--lib)
- [Comments](#comments)

## Variables and constants

Variables must be declared before use. The type comes after `AS`.

```basic
DIM name AS STRING
DIM alter AS INTEGER
DIM groesse AS FLOAT
DIM aktiv AS BOOLEAN

name = "Anna"
alter = 30
groesse = 1.75
aktiv = TRUE
```

Several variables of the same type in one line:

```basic
DIM x, y, z AS INTEGER
DIM vorname, nachname AS STRING

' Mixed with arrays too:
DIM grid[10, 10], score, lives[3] AS INTEGER
```

All variables of a multi-DIM get the same type and their type default as the initial value.
**Unlike VB6**, where `Dim a, b As Integer` only makes `b` an integer:
here the type applies to all names before it — there is no such thing as a variable without a type.

Different types in one line are separated by a comma after the type; each group
may have a start value if it declares exactly one variable:

```basic
DIM breite, hoehe AS INTEGER, titel AS STRING
DIM x, y AS FLOAT, name AS STRING = "Ada", leben AS INTEGER = 3
```

You can give a first value right away — that is the same as `DIM` plus
an assignment below it, so the type check applies in exactly the same way:

```basic
DIM leben AS INTEGER = 3
DIM spieler AS STRING = "Anna"
```

This works for one variable per line; in a CLASS, `SUB Init()` sets the
start values of the fields.

> **No reserved words as variable names.** Some short names are
> keywords and cannot serve as identifiers — besides the
> obvious ones (`to`, `step`, `mod`, `end`, `next`, `new`, `class`, `in`)
> also a few that suggest themselves as variable names: **`map`, `image`, `sound`,
> `input`, `file`, `data`, `read`, `band`**. `DIM band AS INTEGER` then reports
> "'BAND' ist ein reserviertes Wort …" ("'BAND' is a reserved word …"). Workaround: choose another name (`img`,
> `snd`, `karte`, `daten`, …). Common names such as
> `value`, `key`, `count`, `index`, `name`, `type`, `result`, `size`, `pos`,
> `state`, `item`, `text`, `color` are fine, on the other hand.

Constants with `CONST`:

```basic
CONST PI_HALF AS FLOAT = 1.5707963
CONST MAX_LEBEN AS INTEGER = 3
CONST TITEL AS STRING = "Mein Spiel"

' Type can be omitted - it is derived from the value:
CONST FPS = 60
```

Constants may be assigned only once; writing to them later is an error.

If the value of a constant is known at compile time (numbers, strings and
calculations from them, also with other such constants), the compiler
inserts it directly and works out whatever can be worked out: `BREITE \ 2`
then costs no more at run time than `320`. This applies everywhere, also in
a job (`TASK_START`, `dhrt call`) in which the main program does not
run. A constant whose value is only known at run time
(`CONST START = MILLIS()`) remains an ordinary value.

## Number literals

Classic notations:

```basic
DIM dec AS INTEGER
dec = 255

DIM hex AS INTEGER
hex = &HFF                ' Hex (0..F, case-insensitive: &hff)

DIM bin AS INTEGER
bin = &B11010110          ' binary

DIM f AS FLOAT
f = 3.14
f = 1.5e3                 ' exponent: 1500.0 (also 2.5E-3, 1e9)
```

Hex and binary are INTEGER constants only — no floats. All three notations yield the same INTEGER value.
A number with an exponent (`1e9`, `2.5E-3`) is always a float, even without a point -- as with `VAL("1e3")`.

## Data types

| Type | Range | Default |
|---|---|---|
| `INTEGER` | whole number (Python `int`) | `0` |
| `FLOAT` | floating point | `0.0` |
| `STRING` | UTF-8 text | `""` |
| `BOOLEAN` | `TRUE` or `FALSE` | `FALSE` |
| `FILE` | file handle | `NIL` |
| `BUFFER` | mutable byte sequence ([bytes](builtins-core.md#bytes-buffer)) | `NIL` |
| `IMAGE` | image handle (native dhrt) | `NIL` |
| `SOUND` | sound handle (native dhrt) | `NIL` |
| `ARRAY OF T` | multi-dim. array | `NIL` (or with size init: filled) |
| `MAP OF T` | string→T map | empty map |
| `<class name>` | instance | `NIL` |
| `<external type>` | from a module (e.g. `JSON_HANDLE`) | `NIL` |

**Strict typing:** A FLOAT variable does not accept STRINGs. A FLOAT-to-INTEGER cast requires a whole number (`3.0` yes, `3.14` error — use `INT()` then).

**`NIL`:** Class references, images, sounds and external handles are initially `NIL`. You test with `x IS NIL` / `x IS NOT NIL` or the equivalent built-in `IS_NIL(x)`:

```basic
DIM bild AS IMAGE
IF bild IS NIL THEN
    bild = LOADIMAGE("hero.png")
END IF
```

## Type test at run time: `IS` and `TYPEOF`

A reference held polymorphically does not reveal by itself what is inside it.
Two ways ask about it:

```basic
CLASS Tier
END CLASS
CLASS Hund EXTENDS Tier
END CLASS

DIM t AS Tier
t = NEW Hund()

PRINT t IS Hund          ' TRUE
PRINT t IS Tier          ' TRUE  -- every parent class matches too
PRINT t IS NOT Tier      ' FALSE -- equivalent to NOT (t IS Tier)
PRINT TYPEOF(t)          ' "HUND" -- the class name, in upper case
```

To the right of `IS` stands a **type name**, not an expression: a class, a
value type (`INTEGER`, `STRING`, …), a module type (`VEC2`), `NIL`, `ARRAY` or
`MAP`. An unknown name is a **compile error** — a typo would otherwise
silently be `FALSE` forever, and a test that never fires is noticed by
nobody.

`NIL` is not an instance: a `Tier` variable that has not been assigned yet yields
`FALSE` for `t IS Tier` and `TRUE` for `t IS NIL`.

`SELECT CASE` comparisons (`CASE IS > 5`) are not affected — there the
`IS` belongs to the `CASE`.

## What the compiler checks

Types do not only apply at run time. `dhrt --check` (in the editor the yellow
squiggly line, with `dhrt run` a line on stderr) reports beforehand what can be
decided from the text — at **all four places** where the runtime converts a
value to a declared type:

```basic
DIM s AS STRING
s = 5                       ' assignment
f(s)                        ' argument, with FUNCTION f(a AS INTEGER)
p.anzhal                    ' member the class does not have
RETURN "text"               ' in a FUNCTION ... AS INTEGER
```

Plus three cases that do not involve a conversion but go wrong just as
surely:

* **A fractional number assigned to an integer variable** (`n = 7 / 2`) — here the
  runtime rule depends on the value (`n = f * 2.0` works for `f = 1.5` and fails for
  `f = 1.6`), but the line is reported anyway.
* **A typed FUNCTION without any `RETURN`** — it silently returns `NIL`,
  and `NIL` is not an `INTEGER`.
* **A name that no `DIM` or `CONST` declares anywhere** — the classic
  typo. `DIM zaehler` and two lines later `zaehlr = zaehlr + 1`
  aborts at run time, but only when the line runs; in a rarely
  taken branch that can take a long time. The message suggests a
  similar known name if there is one.
* **A name that exists — just not here.** A local variable belongs to
  its `SUB`/`FUNCTION`/method; whoever uses it in a second one gets
  the same run-time abort. The message says so explicitly ("ist an
  dieser Stelle nicht sichtbar", "is not visible at this point"), because otherwise you see the name right there in the
  source and search for a long time for a typo that does not
  exist. If you need it in both places, declare it with `DIM` at the top
  level — there it is global, even inside an `IF` block and even
  for a `SUB` that stands above it in the source.

**All of this is a warning, not an error.** The program can be
compiled and runs. The reason is honesty: the compiler derives the type
of an expression only where it is beyond doubt, and where it does not
know it, it says nothing. A mistake in this derivation must not reject a
running program.

**Where it deliberately stays silent** — so that no message raises a false alarm:

* **Return values of built-ins** are not derived.
* **Classes, MAP and ARRAY** are passed through by the runtime on assignment;
  a message saying it "aborts" would simply be untrue there.
* **`PROPERTY`** — there the setter decides, not the field behind it.
* **A member that a DERIVED class has.** `DIM t AS Tier : t = NEW
  Hund() : t.belle()` is valid and stays silent.
* **Whether every branch reaches a `RETURN`.** It is only reported where there is
  none at all.
* **An assignment that stands *before* its `DIM` in the same subroutine.**
  `t = 1` and only afterwards `DIM t AS INTEGER` also aborts at run time
  — the local slot is only created at the `DIM`. It is not reported: the
  compiler works through the lines in order, and what it does not know yet at
  one point counts as "may still come". The reason is
  the same as above — a message that fires on correct code is
  more expensive than a gap.

## ENUM

Type-safe named constants with a namespace. Instead of magic numbers you write `State.PLAYING`, and the compiler turns that into an `INTEGER` constant. Members are numbered automatically (0, 1, 2, …) or set explicitly.

**Compact form** (one-liner):

```basic
ENUM State = MENU, PLAYING, PAUSED, GAMEOVER

PRINT State.MENU        ' 0
PRINT State.PLAYING     ' 1
PRINT State.PAUSED      ' 2
```

**Block form** with explicit values:

```basic
ENUM Permission
    NONE = 0
    READ = 1
    WRITE = 2
    EXEC = 4
    RW = 3
END ENUM

PRINT Permission.READ + Permission.WRITE    ' 3
```

**Mixed**: mix explicit values and auto-numbering — the next implicit value counts on from the last explicit one:

```basic
ENUM Http = OK = 200, CREATED, ACCEPTED, _
            BAD_REQUEST = 400, UNAUTHORIZED, _
            NOT_FOUND = 404

PRINT Http.OK             ' 200
PRINT Http.CREATED        ' 201  (200+1)
PRINT Http.ACCEPTED       ' 202
PRINT Http.UNAUTHORIZED   ' 401
PRINT Http.NOT_FOUND      ' 404
```

**From the value back to the name** — for debugging and save games:

```basic
ENUM Zustand = MENUE, SPIELT, PAUSE
PRINT ENUM_NAME(Zustand, 1)        ' "SPIELT"
PRINT ENUM_NAME(Zustand, 99)       ' "" -- no member has this value
```

The name comes back in upper case (as with `TYPEOF`), so that a
comparison does not depend on how the member was written. Without
a match, an empty string: a stored value from an older
version is the normal case, not the exception.

**`DIM x AS State`** is equivalent to `DIM x AS INTEGER` — the parser resolves enum types to `INTEGER`:

```basic
ENUM Mood = HAPPY, SAD, ANGRY

DIM m AS Mood
m = Mood.SAD

SELECT CASE m
    CASE Mood.HAPPY
        PRINT "froh"
    CASE Mood.SAD
        PRINT "traurig"
    CASE ELSE
        PRINT "sonstwas"
END SELECT
```

**Member names may be keywords**: `READ`, `FILE`, `DATA`, `NONE` etc. become unambiguous with qualified access (`Name.Member`) — the language allows it.

**Values must be compile-time integer literals.** Compound expressions such as `A + B` are not allowed; use `CONST` instead and write concrete numbers.

## Compound assignment

A more convenient notation for the most common "modify myself" operations — the parser desugars them automatically into `target = target OP value`:

```basic
DIM x AS INTEGER
x = 10
x += 5         ' = x = x + 5  -> 15
x -= 3         ' -> 12
x *= 2         ' -> 24
x /= 4         ' -> 6
```

Also works on array elements and class fields:

```basic
xs[0] += 100
self.health -= damage
```

For the most common case — plus/minus one — there is the short form `++` and
`--`:

```basic
i++            ' = i += 1
i--            ' = i -= 1
p.treffer++
zaehler[3]++
```

Both are **only a statement, not an expression**: there is no `j = i++`.
The difference between prefix and postfix form is the most common
source of errors with this operator — whoever returns no value does not have it.
And because `++` is not a token of its own but is only recognized in
statement position, expressions such as `5 - -3`
remain valid unchanged.

## String interpolation (f-strings)

Instead of gluing with `+` and `STR$`:

```basic
DIM name AS STRING
DIM score AS INTEGER
name = "Anna"
score = 42

' Classic:
PRINT "Hallo, " + name + "! Score: " + STR$(score)

' With an f-string (Python style):
PRINT f"Hallo, {name}! Score: {score}"
```

Inside `f"..."`, expressions in `{...}` are evaluated and turned into strings automatically via `STR$(...)`. Double braces (`{{` and `}}`) are escapes for literal `{` and `}`.

```basic
PRINT f"x = {x + 1}, doppelt = {x * 2}"     ' expressions allowed
PRINT f"{{nicht expr}} aber {x}"             ' "{nicht expr} aber 5"
```

Plain strings (`"..."`) are **not** interpolated — `{name}` stays literal. Only f-strings expand.

## Operators

**Arithmetic:** `+ - * / \ ^ MOD`
- `/` **always** returns a fractional number: `8 / 2` is `4.0`, `9 / 2` is `4.5`
  (since 2026-09-24; before that, `8 / 2` was the integer 4 -- the type depended on the value).
  The result is therefore not suitable as an array index or for commands that require an integer (`MID$`,
  `LEFT$` ...); `dhrt --check` reports an
  index that is certainly a fractional number.
- `\` is integer division (returns INTEGER, INTEGER even with FLOAT input).
- `^` is exponentiation: `2 ^ 8 = 256`.

**Comparison:** `= <> < > <= >=`

**Logic:** `AND OR NOT`. Short-circuit: `AND` and `OR` only evaluate the right operand when necessary.

**String concatenation:** `+`

```basic
DIM g AS STRING
g = "Hallo, " + name + "!"
```

## Strings

String literals are written in double quotes. Double quotes inside them are doubled:

```basic
DIM s AS STRING
s = "Sie sagte ""Hallo""."     ' -> Sie sagte "Hallo".
```

In a normal string, a backslash is a character like any
other: `"\d+"` is a regex pattern, `"assets\tiles.png"` a path. If you
want escape sequences, put a `!` in front (as in FreeBASIC):

```basic
PRINT !"Zeile 1\nZeile 2"          ' line break
PRINT !"Name:\tWert"               ' tab
PRINT !"Sie sagte \"Hallo\"."      ' quotes ("" still works too)
PRINT !"C:\\Spiele\\held.png"      ' a backslash
PRINT f!"Punkte: {p}\n"            ' in an f-string too
```

Allowed are `\n`, `\t`, `\r`, `\\`, `\"`, `\0`, `\e` (escape, 27) and
`\uXXXX` (four hex digits, `\u00E4` is ä). Any other sequence is a
compile error, not silently dropped: whoever writes `!` means
escapes, and a `\q` that turned into `q` would hide a typo.
`CHR$(10)` and `+` of course still work.

For string functions see [Standard built-ins](builtins-core.md): `LEFT$`, `RIGHT$`, `MID$`, `INSTR`, `REPLACE$`, `TRIM$`, `SPLIT$`, `JOIN$`, `UPPER$`, `LOWER$`, `LEN`, `STR$`, `VAL`, `CHR$`, `ASC`, `PADL$`, `PADR$`, `REPEAT$`, `SPACE$`, `HEX$`.

Convention: string functions with a `$` suffix also exist without the suffix (`UPPER$` and `UPPER` are the same).

> **`+` with a string converts the other side automatically** (no
> type error): `"Punkte: " + 42` yields `"Punkte: 42"`, `"ok=" + TRUE` yields
> `"ok=TRUE"`. Convenient for assembling output — but whoever expects strict types
> will be surprised. (`STR$(v)` makes the conversion explicit.)

## Control flow: IF / ELSE

**Block form:**

```basic
IF score > 100 THEN
    PRINT "Sehr gut!"
ELSEIF score > 50 THEN
    PRINT "Geht so."
ELSE
    PRINT "Naja..."
END IF
```

**Single line:**

```basic
IF treffer THEN PRINT "Bumm!"
IF treffer THEN PRINT "Bumm!" ELSE PRINT "Daneben."
```

## SELECT CASE

Multi-way branching — much more readable than nested `IF/ELSEIF` chains. Three match forms, freely combinable:

```basic
SELECT CASE punkte
    CASE 0                     ' exact value
        PRINT "Null"
    CASE 1, 2, 3               ' list
        PRINT "Klein"
    CASE 10 TO 20              ' range (inclusive)
        PRINT "Mittel"
    CASE IS > 100              ' comparison (=, <>, <, >, <=, >=)
        PRINT "Spitze"
    CASE 50, 60 TO 70, IS = 99 ' all forms can be mixed
        PRINT "Spezialfall"
    CASE ELSE                  ' fallback (optional, at most once, must be last)
        PRINT "Anders"
END SELECT
```

**Guarantees:**
- The subject (`punkte`) is evaluated **once**, even for function calls with side effects.
- The first matching CASE wins; after that it stops.
- Also works with STRINGs (`SELECT CASE name CASE "Anna", "Bert" THEN ... END SELECT`).

## Loops: FOR and WHILE

**FOR with a counter:**

```basic
DIM i AS INTEGER
FOR i = 1 TO 10
    PRINT i
NEXT

FOR i = 100 TO 0 STEP -10
    PRINT i
NEXT

DIM x AS FLOAT
FOR x = 0.0 TO 1.0 STEP 0.1
    PRINT x
NEXT
```

**WHILE / WEND** (pre-test — the condition is checked *before* each pass, the body may run 0 times):

```basic
DIM i AS INTEGER
i = 0
WHILE i < 5
    PRINT i
    i = i + 1
WEND
```

**REPEAT / UNTIL** (post-test — the body *always runs at least once*, then the condition is checked; the loop ends when the `UNTIL` expression becomes `TRUE`):

```basic
DIM i AS INTEGER
i = 0
REPEAT
    PRINT i
    i = i + 1
UNTIL i >= 3            ' "repeat UNTIL i >= 3"
```

**DO / LOOP** — the same two patterns, but the condition may be at the top *or* at the bottom, and it may also be missing entirely:

```basic
DO WHILE i < 5          ' head check, like WHILE/WEND
    i = i + 1
LOOP

DO UNTIL i >= 5         ' head check, negated
    i = i + 1
LOOP

DO                      ' foot check: runs at least once
    i = i + 1
LOOP WHILE i < 5

DO                      ' foot check, negated (= REPEAT/UNTIL)
    i = i + 1
LOOP UNTIL i >= 5

DO                      ' endless — ends with BREAK or RETURN
    IF fertig THEN BREAK
LOOP
```

Both at once (`DO WHILE ... LOOP UNTIL ...`) is rejected at compile time. `do` and `loop` are **contextual** and remain ordinary identifiers — `DIM dO AS INTEGER` still works.

Rule of thumb: `WHILE/WEND` if the loop may not need to run at all, `REPEAT/UNTIL` if the body has to run once in any case (e.g. "get input, then check it for validity"). `DO/LOOP` can do both and is the form familiar from other BASIC dialects.

`STEP 0` is an error. With a negative `STEP` the loop runs downwards; the loop is not executed if `start > end` (or the other way round with a negative STEP).

## BREAK and CONTINUE

```basic
FOR i = 1 TO 100
    IF i = 50 THEN
        BREAK            ' leaves the loop immediately
    END IF
    IF i MOD 2 = 0 THEN
        CONTINUE         ' skips the rest, goes to the next iteration
    END IF
    PRINT i
NEXT
```

Works in `FOR`, `FOR EACH`, `WHILE`, `REPEAT/UNTIL` and `DO/LOOP`.

## DATA / READ / RESTORE

Classic BASIC construct for inline data tables right in the source — good for level layouts, sprite definitions, lookup tables, item lists.

```basic
DATA "Anna", 100, "Bert", 75, "Cilly", 50

DIM name AS STRING
DIM score AS INTEGER
DIM i AS INTEGER

FOR i = 0 TO 2
    READ name, score
    PRINT name, score
NEXT
```

Output:
```
Anna 100
Bert 75
Cilly 50
```

**Allowed DATA values**: literals only — numbers (with sign), strings, `TRUE` / `FALSE`. Expressions such as `2 + 3` or variables are not accepted. Examples:

```basic
DATA -5, 3.14, "Hallo", TRUE, -100, "Mit ""Anführungszeichen"""
```

**`RESTORE`** resets the read pointer to the start:

```basic
DATA 1, 2, 3
DIM x AS INTEGER

READ x          ' x = 1
READ x          ' x = 2
RESTORE
READ x          ' x = 1 (from the start)
```

**Where DATA may appear**: anywhere in the source, also inside `SUB`/`FUNCTION`/`CLASS`. At program start, all DATA lines are combined into a single list in source order — as if all DATA statements were at the beginning.

**READ targets** can be variables, array elements or class fields:

```basic
DIM tile_map[10, 10] AS INTEGER
DIM r AS INTEGER
DIM c AS INTEGER
FOR r = 0 TO 9
    FOR c = 0 TO 9
        READ tile_map[r, c]
    NEXT
NEXT
DATA 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
DATA 1, 0, 0, 0, 0, 0, 0, 0, 0, 1
' ... 8 more lines
```

## Line continuation

Long statements can be split across several lines:

```basic
' Implicit: inside open parentheses, newlines are ignored
TEXT(
    x,
    y,
    "Lange Zeile mit vielen Argumenten",
    RGB(220, 220, 230)
)

' Explicit with an underscore at the end of the line
DIM total AS INTEGER
total = a + b + _
        c + d + _
        e
```

Real line breaks are still **not** allowed inside string literals — that is intentional. A line break in the text is `!"...\n..."` (see [Strings](#strings)), `CHR$(10)` or concatenation with `+`.

## Statement separator

Several statements in one line are separated by a colon `:` — as in classic BASIC:

```basic
DIM x AS INTEGER : DIM y AS INTEGER
x = 1 : y = 2 : PRINT x + y       ' "3"

' Handy in compact SUBs
SUB Greet(name AS STRING) : PRINT "Hi " + name : END SUB
```

This is not mandatory — most programs stay more readable with one statement per line. But for init lists or dense dummy code (tests, demo snippets) it is helpful.

## Functions: SUB and FUNCTION

**SUB** (no return value):

```basic
SUB gruessen(name AS STRING)
    PRINT "Hallo, " + name
END SUB

gruessen("Anna")
```

**FUNCTION** (with return value):

```basic
FUNCTION quadriere(n AS INTEGER) AS INTEGER
    RETURN n * n
END FUNCTION

DIM ergebnis AS INTEGER
ergebnis = quadriere(7)        ' = 49
```

Several parameters:

```basic
FUNCTION distanz(x1 AS FLOAT, y1 AS FLOAT, x2 AS FLOAT, y2 AS FLOAT) AS FLOAT
    RETURN SQR((x2 - x1) ^ 2 + (y2 - y1) ^ 2)
END FUNCTION
```

**Default values** for parameters — the caller can omit the last arguments:

```basic
SUB greet(name AS STRING, prefix AS STRING = "Hallo")
    PRINT prefix, name
END SUB

greet("Anna")              ' "Hallo Anna"  (default applies)
greet("Bert", "Hi")        ' "Hi Bert"
```

Default expressions are evaluated **on every call**, in the local scope of the function. That means a later default can refer to an earlier parameter:

```basic
SUB rect_or_square(x AS INTEGER, y AS INTEGER, w AS INTEGER, h AS INTEGER = w)
    PRINT x, y, w, h
END SUB

rect_or_square(0, 0, 50)        ' h = 50 (square default)
rect_or_square(0, 0, 50, 30)    ' h = 30 (explicit)
```

**Rule**: parameters with a default must come *at the end* of the list. A required parameter after a default parameter is a compile error — otherwise it would be ambiguous which position means which.

### Named Arguments

Arguments can also be passed *by name* — `name: wert`. Handy when a SUB has many parameters or you want to set a slot in the middle of the default chain:

```basic
SUB Greet(name AS STRING, _
          age AS INTEGER, _
          greeting AS STRING = "Hallo", _
          suffix AS STRING = "")
    PRINT greeting + ", " + name + " (" + STR$(age) + ")" + suffix
END SUB

' All positional - classic
Greet("Anna", 30)

' Names instead of position - any order
Greet(name: "Bob", age: 25, greeting: "Hi")

' Mixed: positional first, named after
Greet("Cara", 40, suffix: "!")

' Skip a slot in the middle - greeting stays "Hallo"
Greet("Dora", 50, suffix: "?")
```

**Rules**:

- **Positional before named**: all positional arguments must come before the first named argument. `f(a: 1, 2)` is an error.
- **No double assignment**: a slot set both positionally AND by name is an error.
- **An unknown name** is an error — you cannot mistype without the compiler noticing.
- **Required slots** must still be set (either positionally or by name).

**Available for**:

- `SUB`/`FUNCTION` calls
- `NEW Klasse(...)` (passed to the `Init` method)

**Not available for**:

- Built-ins (`ABS`, `CIRCLE`, `JSON_PARSE`, …) — they have no declared parameter names
- Method calls `obj.method(name: ...)` — the class is only known at run time, so the `dhrt` compiler throws here.

### BYREF parameters (multi-return)

With `BYREF`, a parameter is passed **by reference** — the function can change the caller's variable. This is Drachenhauch's answer to multiple return values.

```basic
SUB swap(BYREF a AS INTEGER, BYREF b AS INTEGER)
    DIM tmp AS INTEGER
    tmp = a
    a = b
    b = tmp
END SUB

DIM x AS INTEGER
DIM y AS INTEGER
x = 1
y = 2
swap(x, y)
PRINT x, y       ' "2 1"
```

`BYREF` may appear anywhere in the parameter list and can be mixed with normal parameters. A `FUNCTION` with a `BYREF` parameter can additionally return a regular value:

```basic
FUNCTION divmod(a AS INTEGER, b AS INTEGER, BYREF mod_out AS INTEGER) AS INTEGER
    mod_out = a MOD b
    RETURN a \ b
END FUNCTION

DIM r AS INTEGER
DIM q AS INTEGER
q = divmod(17, 5, r)
PRINT q, r       ' "3 2"
```

**Allowed arguments** for `BYREF` parameters:
- simple variables — `swap(x, y)`
- array elements — `inc(arr[3])`
- class/struct fields — `setpos(player.x)`

**Not allowed**: literals, expressions, function calls — after all, the caller needs a place the value can be written back to.

**Restrictions:**
- `BYREF` may **not** be combined with a default value (what would it mean to pass a default variable by reference?).
- `BYREF` is supported by `dhrt`: at the call site the compiler inserts an lvalue capture plus a post-call write-back (copy-in/copy-out), and the VM returns the final parameter values. (Currently only for direct `SUB`/`FUNCTION` calls — not via `FUNCREF` or method calls whose class is only known at run time.)

**Recursion** works:

```basic
FUNCTION fib(n AS INTEGER) AS INTEGER
    IF n < 2 THEN
        RETURN n
    END IF
    RETURN fib(n - 1) + fib(n - 2)
END FUNCTION
```

**How deep?** Up to **1000** nested calls. Every DH call occupies a
frame on the native stack (~6.6 KB), and the limit is set by the
tightest platform (Linux gives the main thread 8 MB, beyond the reach of any linker
flag) -- so that a program behaves the same everywhere instead of running on one
machine and aborting on the next.

Whoever exceeds the limit gets a perfectly normal error that can be caught with `TRY`/`CATCH`:

```
Laufzeitfehler in spiel.dh:12: Maximale Aufruftiefe (1000) ueberschritten -- unendliche Rekursion?
```

For most tasks that is plenty. Where it gets tight -- a recursive
flood fill covers a field of up to roughly 31x31 tiles this way -- you rewrite the
recursion as a loop with its own queue: instead of calling itself,
you put the next task into an ARRAY and work through it.

## Functions as values: FUNCREF and lambdas

A function is also a value of type `FUNCREF`. There are three ways to
get one:

```basic
FUNCTION quadrat(x AS INTEGER) AS INTEGER
    RETURN x * x
END FUNCTION

DIM f AS FUNCREF
f = quadrat                       ' a named function (name without parentheses)
f = spieler.tick                  ' a method, bound to its object
f = FUNCTION(x) x * x             ' a lambda
PRINT f(7)                        ' 49
```

A FUNCREF is called like a function, also directly from an
expression: `aktionen[i]()`, `addiere(10)(5)`. It is accepted by `SORT(feld,
f)`, `ARRAY_MAP/FILTER/REDUCE/FIND`, `GUI_ON_CLICK` and the other
`GUI_ON_*`, `TIMER_AFTER/EVERY`, `TASK_START` and every parameter of your own
declared `AS FUNCREF`.

### Lambdas

The notation is that of VB.NET:

```basic
DIM doppelt AS FUNCREF : doppelt = FUNCTION(x) x * 2
DIM mal AS FUNCREF : mal = FUNCTION(a AS INTEGER, b AS INTEGER) a * b
GUI_ON_CLICK(knopf, SUB() zaehler += 1)

DIM a AS ARRAY OF INTEGER : a = [5, 3, 9, 1]
SORT(a, FUNCTION(x, y) y - x)                         ' descending
DIM gross AS ARRAY OF INTEGER : gross = ARRAY_FILTER(a, FUNCTION(x) x > 3)
PRINT ARRAY_REDUCE(a, FUNCTION(summe, x) summe + x, 0) ' 18
```

- `FUNCTION(parameter) ausdruck` returns the value of the expression,
  `SUB(parameter) anweisung` executes a statement. Both fit on ONE
  line -- for more, write a named FUNCTION/SUB.
- A parameter without `AS` takes any value; with `AS` it is checked as for
  any function. Default values, `BYREF` and `...` do not exist in a lambda.
- **Local variables are copied when the lambda is created.** A lambda can live
  longer than the call that created it (a callback, a timer) --
  by then that call's variables no longer exist:

  ```basic
  FUNCTION addierer(n AS INTEGER) AS FUNCREF
      RETURN FUNCTION(x) x + n      ' n is copied here
  END FUNCTION
  DIM plus5 AS FUNCREF : plus5 = addierer(5)
  PRINT plus5(1)                    ' 6
  ```

  An assignment to a copied variable would have no effect outside and
  is therefore a compile error.
- **A lambda sees global variables live** -- a callback should show the
  state of NOW. The consequence: a lambda created in the main program inside
  a loop sees the loop variable with its LAST value.
  If each one should get its own, create it in a SUB that receives the
  value as a parameter (there it is local and gets copied).
- In a method, a lambda reaches fields and methods via `Self`
  (`FUNCTION(x) x * Self.faktor`); a field without `Self.` is an error --
  a lambda is not a method.
- No `YIELD` in a lambda, and `TASK_START` only accepts lambdas without copies
  (a job runs in a process of its own).
- A `.dhform` cannot contain a lambda as a callback: its name depends on
  its position in the source.

## Coroutines: YIELD

A `FUNCTION` or `SUB` whose body contains a `YIELD` is a **coroutine**. Calling it does *not* execute the body but returns a `COROUTINE` handle that you advance step by step.

```basic
FUNCTION zaehler() AS INTEGER
    YIELD 1
    YIELD 2
    RETURN 99            ' final value (optional)
END FUNCTION

DIM c AS COROUTINE
c = zaehler()
PRINT CORO_RESUME(c)     ' 1
PRINT CORO_RESUME(c)     ' 2
PRINT CORO_RESUME(c)     ' 99  (now finished: CORO_DONE(c) = TRUE)
```

**Built-ins:**

| Built-in | Effect |
|---|---|
| `CORO_RESUME(c)` | Continue up to the next `YIELD`; returns the YIELD value (or the RETURN value when the coroutine ends). |
| `CORO_SEND(c, v)` | Like `CORO_RESUME`, but the `YIELD` expression in the body evaluates to `v`. |
| `CORO_DONE(c)` | `BOOLEAN` — whether the coroutine has finished. |
| `CORO_RESULT(c)` | Final `RETURN` value (throws if not finished yet). |
| `CORO_CLOSE(c)` | Tear down a suspended coroutine. |

**`YIELD` is an expression.** As a statement (`YIELD v`) it discards the sent value; as an expression it returns the value passed via `CORO_SEND`:

```basic
FUNCTION akkumulator() AS INTEGER
    DIM sum AS INTEGER
    sum = 0
    WHILE TRUE
        sum = sum + (YIELD sum)   ' hands out the sum, receives the next addend
    WEND
END FUNCTION

DIM acc AS COROUTINE
acc = akkumulator()
PRINT CORO_RESUME(acc)     ' 0  (the sent value of the FIRST resume is always NIL)
PRINT CORO_SEND(acc, 10)   ' 10
PRINT CORO_SEND(acc, 5)    ' 15
CORO_CLOSE(acc)
```

**`FOR EACH` and comprehensions** consume a coroutine **eagerly** to the end (the `RETURN` value is not included):

```basic
DIM total AS INTEGER
total = 0
FOR EACH n IN zaehler()
    total = total + n          ' 1 + 2 = 3 (RETURN 99 not included)
NEXT
```

For infinite generators use `CORO_RESUME`/`CORO_DONE` manually instead.

**Semantics & restrictions:**
- `dhrt` suspends a coroutine via a **frame snapshot** (ip/locals/stack are stored at `YIELD` and restored on resume) — no OS thread, deterministic, safe for the raylib main thread.
- **No cross-frame `YIELD`:** a helper with `YIELD` is itself a coroutine; so `YIELD` never runs across a normal function call.
- In `FUNCTION ... AS T`, `YIELD` *and* `RETURN` values are coerced to `T`. A `SUB` coroutine yields without type coercion.
- A manual `WHILE NOT CORO_DONE(c)` loop receives the `RETURN` value on the last (finishing) `CORO_RESUME`. Give the generator a typed `RETURN` so that the assignment to a typed variable works — or use `FOR EACH`.
- Also works in the standalone `.exe` export (same `dhrt` VM).

Complete example: [examples/98_coroutines.dh](../../examples/98_coroutines.dh).

## Arrays

One-dimensional:

```basic
DIM zahlen[10] AS INTEGER         ' 10 elements, all 0
zahlen[0] = 42
zahlen[9] = 99

DIM i AS INTEGER
FOR i = 0 TO LEN(zahlen) - 1
    PRINT zahlen[i]
NEXT
```

Multi-dimensional:

```basic
DIM brett[8, 8] AS INTEGER       ' chessboard
brett[0, 0] = 1
brett[7, 7] = 1

' Number of dimensions and size per dimension:
PRINT DIMCOUNT(brett)             ' 2
PRINT DIMSIZE(brett, 0)           ' 8
PRINT DIMSIZE(brett, 1)           ' 8
```

Arrays of external types (e.g. SPRITE) are possible; the default value is `NIL`:

```basic
DIM coins[5] AS SPRITE
DIM i AS INTEGER
FOR i = 0 TO 4
    coins[i] = SPRITE_NEW(coin_img, 8, 8)
NEXT
```

> **Arrays are passed by reference.** If you pass an array to a
> `SUB`/`FUNCTION`, caller and callee share the same memory —
> changes in the subroutine affect the original (exactly what
> `ARRAY_PUSH`/`SORT`/… and your own in-place routines rely on). If you need **a copy of your own**,
> call `ARRAY_COPY(arr)`.
>
> The same holds for **return values**: an array, an object and a struct
> come back as a reference. An index or field therefore works directly on
> the result of a call (`teile(s)[2]`, `held().hp`), for reading and
> writing -- `held().hp = 0` changes the object `held` returned. Only
> `+=`, `-=`, `++` ... don't work there: the call would run twice.
>
> **Index access is strict, slicing clamps.** A direct index outside the
> bounds throws a run-time error (`Index 5 ausserhalb [0..2]`). A **slice**,
> on the other hand, is silently clamped to the valid bounds: `arr[0:99]` on a
> 3-element array returns 3 elements without an error. (Slicing exists only for 1D arrays.)

**The element type also applies on assignment.** An `ARRAY OF INTEGER` does
not end up in an `ARRAY OF STRING` — for variables, parameters and return values
alike:

```basic
DIM zahlen AS ARRAY OF INTEGER
zahlen = [1, 2, 3]
DIM texte AS ARRAY OF STRING
texte = zahlen        ' error: expected ARRAY OF STRING, got ARRAY OF INTEGER
```

Three cases remain explicitly allowed:

- An **empty literal** `[]` has no element type yet and takes that of the
  target — `DIM namen AS ARRAY OF STRING : namen = []` stays an
  `ARRAY OF STRING`.
- An **`ARRAY OF ANY`** may be given to a narrower target. It can contain any
  value; narrowing is a deliberate decision, and write access
  checks again afterwards.
- A **fresh integer literal** assigned to a FLOAT target is converted:
  `DIM w AS ARRAY OF FLOAT : w = [1, 2, 3]` yields a real FLOAT array.
  An **existing** `ARRAY OF INTEGER`, however, is not — its previous name
  still points to the same cells, and they cannot be INTEGER
  and FLOAT at the same time. If you need the values as FLOAT, copy them.

**Without a size, an array is empty and grows.** `DIM a AS ARRAY OF INTEGER`
yields an array with 0 elements (until 2026-09-21 it was NIL, and the first
`ARRAY_PUSH` reported "erwartet ARRAY" ("expected ARRAY")). `+` joins two arrays into a
new one — with the same element type, or INTEGER with FLOAT:

```basic
DIM teile AS ARRAY OF STRING
ARRAY_PUSH(teile, "Kopf")
teile = teile + ["Arm", "Bein"]
PRINT LEN(teile)                  ' 3
```

## Maps

Keys are always STRINGs; values can be of any type.

A map can be written as a **literal** — the counterpart to the
array literal `[1, 2, 3]`:

```basic
DIM punkte AS MAP OF INTEGER
punkte = {"Anna": 95, "Bert": 78}

DIM leer AS MAP OF STRING
leer = {}                              ' the empty map
```

Key and value are full expressions (`{"x" + STR$(n): n * 10}`), a
trailing comma is allowed. Not to be confused with the
dict comprehension `{k: v FOR x IN ...}` — there a `FOR` stands inside the
braces.

`FOR EACH` runs over a map in two forms:

```basic
FOR EACH name IN punkte                ' only the keys
    PRINT name
NEXT

FOR EACH name, wert IN punkte          ' key AND value
    PRINT name; ": "; wert
NEXT
```

The two-variable form also accepts any other sequence of pairs
(`FOR EACH a, b IN (("x", 9), ("y", 8))`).

The individual commands:

```basic
MAPPUT(punkte, "Anna", 95)
MAPPUT(punkte, "Bert", 78)

PRINT MAPGET(punkte, "Anna")           ' 95
PRINT MAPGETOR(punkte, "Eve", 0)       ' 0 (default)

IF MAPHAS(punkte, "Anna") THEN
    MAPREMOVE(punkte, "Anna")
END IF

PRINT MAPSIZE(punkte)                  ' 1
```

Shorter with square brackets — equivalent to `MAPPUT`/`MAPGET`, including
the same type check; a missing key is an error when reading
(`MAPGETOR` returns a default value instead):

```basic
punkte["Eve"] = 60
punkte["Eve"] += 5
PRINT punkte["Eve"]; LEN(punkte)       ' LEN counts the entries
```

More under [Standard built-ins → Maps](builtins-core.md#maps).

## Classes and structures

**Class:**

```basic
CLASS Player
    DIM x AS FLOAT
    DIM y AS FLOAT
    DIM hp AS INTEGER

    SUB Init(start_x AS FLOAT, start_y AS FLOAT)
        x = start_x
        y = start_y
        hp = 100
    END SUB

    SUB MoveBy(dx AS FLOAT, dy AS FLOAT)
        x = x + dx
        y = y + dy
    END SUB

    FUNCTION IsAlive() AS BOOLEAN
        RETURN hp > 0
    END FUNCTION
END CLASS
```

Usage:

```basic
DIM p AS Player
p = NEW Player(100.0, 50.0)        ' calls Init
p.MoveBy(10.0, 5.0)
PRINT p.x, p.y                     ' 110.0  55.0
IF p.IsAlive() THEN PRINT "noch da"
```

**Method bodies see fields directly** — no `Self.` prefix needed. The `x = start_x` in `Init` automatically sets the class field `x`; `start_x` is a parameter. Drachenhauch resolves name lookups in methods like this: first local variables / parameters, then class fields, then global variables.

**Methods call each other — implicitly:**

```basic
CLASS Counter
    DIM v AS INTEGER

    SUB Init()
        v = 0
        Reset()                ' method of the own class, without Self.
    END SUB

    SUB Reset()
        v = 0
    END SUB
END CLASS
```

Inside a method, an identifier call such as `Reset()` is first looked up in the own class (and superclasses). If a method is found, it is called as `Self.Reset()`. If none matches, resolution falls back to global functions. So methods win over global functions of the same name — as in most OOP languages.

**`Self`** as an identifier returns the current instance:

```basic
CLASS Box
    DIM v AS INTEGER

    FUNCTION Triple() AS INTEGER
        RETURN Self.v * 3      ' Self is this instance
    END FUNCTION

    SUB GiveTo(other AS Container)
        other.Add(Self)        ' Self as an argument to another method
    END SUB
END CLASS
```

**Inheritance with `EXTENDS`:**

```basic
CLASS Hero EXTENDS Player
    DIM weapon AS STRING

    SUB Init(start_x AS FLOAT, start_y AS FLOAT, w AS STRING)
        SUPER.Init(start_x, start_y)   ' the Init of the parent class
        hp = 150                       ' and then its own part
        weapon = w
    END SUB
END CLASS
```

**`SUPER.Methode(...)`** calls the **parent class's** version — even
when the own class overrides it:

```basic
CLASS Hero EXTENDS Player
    FUNCTION Beschreibung() AS STRING
        RETURN SUPER.Beschreibung() + " mit " + weapon
    END FUNCTION
END CLASS
```

The search starts at the parent class *of the place in the source*, not at the
class of the object. That is why it also works across several levels: each
level asks its own parent class instead of calling itself in a circle.
If an intermediate class skips the method, the search continues further up.

`SUPER` is **not** a reserved word — a variable of that name remains
allowed.

**`ABSTRACT`: announce a method without writing it**

```basic
CLASS Form
    ABSTRACT FUNCTION Flaeche() AS FLOAT
    ABSTRACT SUB Zeichne()

    FUNCTION Zeige() AS STRING          ' may use it anyway
        RETURN "Flaeche: " + STR$(Flaeche())
    END FUNCTION
END CLASS
```

An announced method has no body and no `END SUB`/`END FUNCTION`.
Whoever tries to create a class with announcements still open using `NEW` gets
a **compile error** — not only at run time:

```
NEW form: die Klasse kuendigt eine Methode an, ohne sie auszufuellen (zeichne).
```

This way you can write a base class that works with methods that do not
exist in it yet — `Zeige()` above calls `Flaeche()`, and at run time
that ends up in the subclass that filled it in.

`ABSTRACT` is not a reserved word either.

**STRUCT** is a lightweight data-class substitute that is instantiated automatically (no `NEW` needed):

```basic
STRUCT Punkt
    DIM x AS FLOAT
    DIM y AS FLOAT
END STRUCT

DIM p AS Punkt
p.x = 10.0
p.y = 20.0
PRINT p.x, p.y
```

Inside a method you simply access your own fields by name (no `this.` or `self.`).

**`STRUCT name LAYOUT C`** is something else: a piece of memory with fields
at fixed positions, the way a C compiler lays them out -- for foreign libraries and
file formats. The fields carry type words such as `LONG`, `USHORT` or
`TEXT * 16`, and the variable is a `BUFFER`. More in
[Foreign libraries](ffi.md#struct-layout).

## Try / Catch / Throw

```basic
TRY
    DIM s AS STRING
    s = JSON_GET_STRING(handle, "user.name")
    PRINT s
CATCH e
    PRINT "Fehler: ", e
END TRY
```

`THROW <wert>` raises an exception (the value is always a STRING):

```basic
SUB pruefen(score AS INTEGER)
    IF score < 0 THEN
        THROW "Negativer Score nicht erlaubt"
    END IF
END SUB

TRY
    pruefen(-1)
CATCH e
    PRINT e
END TRY
```

The catch variable is optional (`CATCH` without a name) if you do not need the value.

### FINALLY — clean up, no matter how you get out

A `FINALLY` branch **always** runs: after a clean pass, after a
caught error, on an error that is passed on — and also when
the block is left via `RETURN`, `BREAK` or `CONTINUE`.

```basic
DIM f AS FILE
f = OPENFILE("daten.txt", "r")
TRY
    verarbeite(READALL$(f))
CATCH e
    PRINT "Fehler: " + e
FINALLY
    CLOSEFILE(f)        ' happens in EVERY case
END TRY
```

`CATCH` and `FINALLY` are each optional, but at least one of them must be
there. **`TRY ... FINALLY ... END TRY` without `CATCH` catches nothing** — it only
cleans up, and the error then continues outwards. That is usually exactly
what you want: you always want to clean up, but decide in only one place.

With nested `TRY` blocks, the `FINALLY` branches run from the inside
out.

> **The return value is computed before the `FINALLY`.** `RETURN x` returns the
> `x` of that moment — whatever the `FINALLY` branch does to the variable afterwards
> no longer changes the returned value.

> **`TRY ... END TRY` with neither `CATCH` nor `FINALLY`** silently swallows an
> error. That has always been so and stays so — but it is
> rarely what you want.

### Error code: decide without comparing texts

`THROW` optionally takes a code before the message. The code is what you
react to; the message is what you show the user:

```basic
THROW "NETZ", "Server antwortet nicht"
```

In the `CATCH`, `ERROR_CODE$()` returns the code and `ERROR_LINE()` the line in
which the error occurred:

```basic
TRY
    hole_daten()
CATCH e
    SELECT CASE ERROR_CODE$()
        CASE "NETZ"
            PRINT "Später nochmal versuchen"
        CASE "DATEI"
            PRINT "Vorgabe benutzen"
        CASE ELSE
            PRINT f"Unerwartet in Zeile {ERROR_LINE()}: {e}"
    END SELECT
END TRY
```

A built-in run-time error (division by zero, index out of range, …) has
the code `""` — only a `THROW` with two values sets one. Both pieces of information
survive an intervening `FINALLY`.

## Import

**Source module** (include another `.dh` file):

```basic
IMPORT "mathlib.dh"

PRINT Distance(0.0, 0.0, 3.0, 4.0)    ' 5.0 - from mathlib.dh
```

`IMPORT` is textual inclusion — the code from `mathlib.dh` becomes part of the current program. Importing the same file several times is ignored (no endless cycle).

### Where it searches

1. **Next to the importing file.** A project's own copy
   always wins — whoever puts a file next to it wants exactly that one.
2. **`pakete/` from there upwards** — the project's packages that
   `dhrt paket hole` puts there (see [Packages](pakete.md)). If a package
   itself needs a package, that one lies in its own `pakete/` and is found
   first.
3. **Every folder from `DH_PATH`**, in the order given (like
   `PATH` and `PYTHONPATH`; the separator is `;` on Windows, `:` elsewhere).
4. **`<user folder>/.drachenhauch/bibliothek`** — the place for things that
   should be available to all projects on this machine. It is not
   created automatically; whoever creates it meant it.

```bash
set DH_PATH=D:\dh-bibliothek          &:: Windows
export DH_PATH=~/dh-bibliothek         # macOS/Linux
```

If the file is not found anywhere, the message names **all** the places
searched — otherwise you would have to guess why it is missing.

**Built-in modules stay built in.** `IMPORT "json"` always takes the
module, even if a file named `json` lies somewhere in the search path. Without
this exception, a single file in a shared folder could change the
behaviour of *every* program on the machine.

The search applies all the way along: a library may in turn
import, namely relative to **itself** — not to the main program.
`AS` namespaces, `PRIVATE` and classes (see below) work from the
search path just the same as from next door.

### Namespace: `IMPORT "datei.dh" AS name`

Without `AS`, all names of the imported file land in the same flat space as
your own code — two libraries each with a function `Init` cannot
be used together that way. With `AS`, the file gets a space of its own:

```basic
IMPORT "mathe.dh" AS mathe

PRINT mathe.Quadrat(5)      ' 25
PRINT mathe.FAKTOR          ' a CONST from mathe.dh
```

Inside `mathe.dh` nothing changes: there `Quadrat` is still called
`Quadrat`, even when another function of the same file calls it.

**A namespace does not see the globals of your main program.** That is the
real gain — the file no longer depends on which variables you
happened to declare at the top:

```basic
' g.dh
FUNCTION LiestGlobal() AS INTEGER
    RETURN punkte          ' error if g.dh is imported with AS
END FUNCTION
```

Pass the value in as a parameter or declare it in the file itself.
Without `AS`, the old flat access remains allowed — existing programs
do not change.

**`PRIVATE`** hides a name in the namespace. Public is the default:

```basic
' p.dh
PRIVATE CONST GEHEIM AS INTEGER = 42

PRIVATE FUNCTION Intern(x AS INTEGER) AS INTEGER
    RETURN x + GEHEIM
END FUNCTION

FUNCTION Offen(x AS INTEGER) AS INTEGER
    RETURN Intern(x)       ' allowed inside
END FUNCTION
```

`p.Offen(1)` returns `43`; `p.Intern(1)` reports that the name is PRIVATE.
`PRIVATE` comes before `SUB`, `FUNCTION`, `DIM` or `CONST`. In a file without
`AS` it is a marker without effect.

**Classes and structs** also go through the namespace — as a type and
after `NEW`:

```basic
IMPORT "mathe.dh" AS mathe

DIM p AS mathe.Punkt
p = NEW mathe.Punkt()
p.x = 7
```

This way both files may have a class `Punkt`. `ARRAY OF mathe.Punkt`
and `MAP OF mathe.Punkt` work just as well. Inside `mathe.dh`
the class is still simply called `Punkt`.

**ENUMs** as well — with two dots in a row:

```basic
IMPORT "mathe.dh" AS mathe

DIM f AS mathe.Farbe
f = mathe.Farbe.ROT
```

As a type, an ENUM is an `INTEGER`, just like a flat ENUM. This lets
both files have a `Farbe` without getting in each other's way.

**Not to be confused** with `AS` on a built-in module: `IMPORT "json" AS
j` replaces the prefix there (`J_PARSE` instead of `JSON_PARSE`). For `.dh` files
the dot applies, as above.

**Built-in module:** without the `.dh` extension, an internal module is loaded.

```basic
IMPORT "json"
IMPORT "sprite"
IMPORT "camera"
```

List of all modules: see [README](README.md#modules).

The resolution order: first `<name>.dh` is searched for in the current directory; if it does not exist, then `drachenhauch/modules/<name>.py`. This way your own `json.dh` can take precedence over the built-in.

## Foreign libraries: DECLARE … LIB

A function from a DLL, `.so` or `.dylib` is declared once and
then called like one of your own:

```basic
DECLARE FUNCTION strlen LIB "c" (s AS TEXT) AS ZEIGER
PRINT strlen("Drachenhauch")        ' 12
```

The type words of the line (`LONG`, `ZEIGER`, `TEXT`, `BUFFER`, `BYREF` ...)
say how wide a value is in C; the library is loaded on the first call;
`LIB "$NAME"` then takes the name from an environment variable.
A crash in foreign code cannot be caught. Everything else --
including embedding Python (numpy, Qt via PySide6) -- in
[Foreign libraries](ffi.md).

## Comments

With `'` (apostrophe) or `REM` to the end of the line:

```basic
' This is a comment
REM a comment too
PRINT "Hi"        ' inline comment
```

There are no multi-line comments — every line needs its own `'`.

## Built-in constants

| Constant | Value | Purpose |
|---|---|---|
| `PI` | 3.141592653589793 | the circle constant |
| `TRUE` / `FALSE` | bool | Boolean literals |
| `NIL` | — | empty reference value |

Plus all colour constants (`BLACK`, `WHITE`, `RED`, `GREEN`, `BLUE`, `YELLOW`, `CYAN`, `MAGENTA`, `ORANGE`, `PURPLE`, `BROWN`, `PINK`, `DARKRED`, `DARKGREEN`, `DARKBLUE`, `GRAY`, `LIGHTGRAY`, `DARKGRAY`) and key constants (`KEY_ESCAPE`, `KEY_RETURN`, `KEY_SPACE`, `KEY_LEFT/RIGHT/UP/DOWN`, `KEY_A` to `KEY_Z`, `KEY_0` to `KEY_9`, `KEY_F1` to `KEY_F12`, modifiers, navigation block and numeric keypad — complete table in [builtins-grafik.md](builtins-grafik.md#input-keyboard-and-mouse)).
