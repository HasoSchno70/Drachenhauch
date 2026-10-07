# Switching over from QBasic, VB, Blitz and Python

Drachenhauch looks like the BASIC you know — and in a few places it is
deliberately different: every variable needs a `DIM` with a type, there is no
`GOTO`, arrays start at 0, and commands take their values in parentheses.
This page collects what you bring along from other languages and what it is
called here. Most of it the compiler tells you itself as soon as you write it
the way you are used to.

All cases below are checked in `tests/pruef/umstieg.dhtest`.

## What simply works

Drachenhauch understands these forms from other BASICs the way you know them:

| You write | What happens |
|---|---|
| `DIM hp AS INTEGER = 100` | declare and set the first value in one line |
| `DIM a, i, z AS INTEGER` | all three are INTEGER — unlike VB6, where only `z` would be |
| `DIM n AS INTEGER, s AS STRING` | different types in one line, also with a start value per variable |
| `EXIT FOR`, `EXIT DO`, `EXIT WHILE` | leaves the loop — but only if it is the **innermost** one; otherwise an error instead of silently the wrong one |
| `EXIT SUB` | leaves the subroutine (in a FUNCTION: `RETURN wert`) |
| `END` alone on the line | ends the program, like `EXIT(0)` |
| `WHILE ... END WHILE` | like `WHILE ... WEND` |
| `SWAP a, b` | swaps two variables (also array elements and fields of objects) |
| `LET x = 5` | the `LET` is skipped |
| `? "Hallo"` | short form of `PRINT` |
| `INPUT "Name"; n` | as in QBasic, with `? ` after the prompt; with `,` without it |
| `CLS`, `meineSub`, `spieler.springen` | a name on its own calls the SUB, the command or the method |
| `m["name"] = 5`, `PRINT m["name"]` | MAPs with square brackets, equivalent to `MAPPUT`/`MAPGET` |
| `LEN(m)` | number of entries in a MAP |
| `a = a + [x]` | join two arrays into a new one (same element type) |
| `DIM a AS ARRAY OF INTEGER` + `ARRAY_PUSH(a, 7)` | an array without a size is empty and grows right away |
| `VAL("3 Äpfel")`, `VAL("&HFF")`, `VAL("1e3")` | the number at the **start** of the text; `&H`/`&O`/`&B` and `0x`/`0b` work |
| `RANDOMIZE(TIMER())` | seed from the clock |
| `CALL meineSub(1)`, `SUB s(BYVAL x AS INTEGER)` | `CALL` and `BYVAL` are skipped (arguments are passed by value anyway) |
| `WHILE NOT EOF(f)` | `EOF(f)` tells whether the file has ended — `READLINE` returns `""` there, as for an empty line |
| `READALL$("datei.txt")` | the whole file at once, without opening it first |
| `DECLARE FUNCTION f LIB "user32" ALIAS "X" (ByVal h AS LONG) AS LONG` | as in VB: a function from a DLL (type words see [Foreign libraries](ffi.md)); your own SUBs need no `DECLARE` |

## What is called differently

| From other BASICs | In Drachenhauch |
|---|---|
| `a$ = "x"` without DIM | `DIM a$ AS STRING` — every variable needs DIM and a type |
| `DIM a(10)` | `DIM a[10] AS INTEGER` — square brackets, index 0 to 9 |
| `DIM SHARED g` | not needed: whatever is declared with `DIM` at the top is visible to all SUBs |
| `x%`, `wert!`, `d#` | no type characters — `DIM x AS INTEGER` |
| `DOUBLE`, `SINGLE` | `FLOAT` (always 64 bits) |
| `LONG`, `SHORT`, `BYTE` | `INTEGER` (always 64 bits) |
| `BOOL` | `BOOLEAN` |
| `STRING * 10` | strings have no fixed length — `LEFT$`/`PADR$` bring them to width |
| `TYPE ... END TYPE` | `STRUCT ... END STRUCT`, fields with `DIM` |
| `REDIM a(20)` | array without size + `ARRAY_PUSH`, or `DIM a[20] AS INTEGER` again |
| `LBOUND` / `UBOUND` | `0` / `LEN(a) - 1` |
| `GOTO`, `GOSUB`, line numbers | do not exist — loops (`WHILE`, `DO`, `FOR`) and `SUB` |
| `ON ERROR GOTO` | `TRY ... CATCH meldung ... END TRY` |
| `DEF FN q(x) = x * x` | `FUNCTION q(x AS INTEGER) AS INTEGER` … `RETURN x * x` |
| `f = ergebnis` in `FUNCTION f` | `RETURN ergebnis` |
| `ELSE IF` (two words), `ENDIF` | `ELSEIF` (one word), `END IF` (two) |
| `LOCATE 1, 1`, `COLOR 14`, `SLEEP 100` | values in parentheses: `SLEEP(100)` |
| `r = RND`, `t = TIMER` | with parentheses: `RND()`, `TIMER()` |
| `UCASE$`, `LCASE$`, `STRING$` | `UPPER$`, `LOWER$`, `REPEAT$` |
| `FIX` | `INT` rounds **down**; towards zero `IIF(x < 0, -INT(-x), INT(x))` |
| `CINT`, `CDBL`, `CSTR` | `INT`/`ROUND`, `FLT`, `STR$` |
| `INSTR(start, text, suche)` | `INSTR(text, suche, start)` — start position at the end |
| `MID$(s, 1, 1) = "x"` | put the string back together: `LEFT$(s, i) + neu + MID$(s, i + 1)` |
| `PRINT USING "##.##"; x` | `PRINT FORMAT$(x, "%5.2f")` or `f"{x:.2f}"` |
| `LINE INPUT s$` | `INPUT s$` already reads the whole line |
| `OPEN "f" FOR OUTPUT AS #1` | `f = OPENFILE("f", "w")`, `WRITELINE(f, text)`, `CLOSEFILE(f)` |
| `"a" & "b"` | `"a" + "b"` |
| `a XOR b` | `a BXOR b` (bits) or `a <> b` (truth values) |
| `6 AND 3` as bits | `6 BAND 3` — `AND`/`OR` are logical here (warning when compiling) |
| `IF s = 5` with `s` a string, `IF b = 1` with `b` a BOOLEAN | different kinds are never equal: `VAL(s) = 5`, `IF b THEN` (warning when compiling) |
| `FOR i = 0 TO 1 STEP 0.5` with INTEGER `i` | `DIM i AS FLOAT` — a fractional step needs a FLOAT loop variable; `STEP 0` is an error |
| `x = zwei + 1` with `FUNCTION zwei()` | `zwei() + 1` — without parentheses a FUNCTION is only its name |
| `Me.x`, `this.x` | `Self.x` |
| `SUB New()` as constructor | `SUB Init()` |
| `OPTIONAL x AS INTEGER` | `x AS INTEGER = 0` (default value) |
| `CATCH e AS STRING` | `CATCH e` — the message is always a STRING |
| `s[0] = "x"` | strings are not changed in place: `s = "x" + MID$(s, 1)` |
| `SCREEN 12` | `SCREEN(640, 480, "Titel")` — there are no screen modes |
| `LINE (x1, y1)-(x2, y2), farbe` | `LINE(x1, y1, x2, y2, farbe)`; `CIRCLE(x, y, r, farbe)` |
| `PSET`, `COLOR 14` | `PLOT(x, y, farbe)`; the colour is an argument of every drawing command |
| `KEYDOWN` | `KEYPRESSED(KEY_A)` (held) or `KEYHIT(KEY_A)` (just pressed) |
| `IMPORT math` | `IMPORT "json"` in quotes, your own files with extension: `IMPORT "helfer.dh"` |
| `CLASS B INHERITS A` | `CLASS B EXTENDS A` |
| `PUBLIC x AS INTEGER` in a class | `DIM x AS INTEGER` — everything in a class is reachable from outside |
| an expression across two lines | end the line with ` _` |
| `TAB`, `SPC` | `PADR$(text, n)`, `SPACE$(3)` |
| `x = meineSub()` | a SUB returns no value — `FUNCTION ... AS <Typ>` with `RETURN` |
| `TRUNC` | `INT` rounds down; towards zero `IIF(x < 0, -INT(-x), INT(x))` |
| `ZEIT_JAHR` | `ZEIT_TEIL(t, "jahr")` |
| `GUI_WINDOW_NEW` | `GUI_WINDOW(titel$, x, y, breite, höhe)` |
| `ARRAY_REMOVE` | `ARRAY_REMOVE_AT(a, i)` |
| `MAKEDIR` | `MKDIR(pfad$)` — for every unknown command the message suggests the real name |
| `MouseDown(1)` for left | `MOUSEBUTTON(0)` — 0 left, 1 right, 2 middle |
| `KEYHIT("a")` | `KEYHIT(ASC("a"))` or `KEYHIT(KEY_A)` — a key is a number |
| `RGB("FF0000")` | `COLOR_FROM_HEX("#FF0000")`, in source code `&HFF0000` |
| `PLAYSOUND("sprung.wav")` | load it first: `s = LOADSOUND("sprung.wav")`, then `PLAYSOUND(s)` (likewise `DRAWIMAGE`, `SPRITE_NEW` with `LOADIMAGE`) |

## The first program with graphics

Four lines you may leave out in QBasic or Blitz, but not here:

```basic
SCREEN(320, 200, "Mein Spiel")        ' 1. the window first
WHILE NOT QUITREQUESTED()
    CLS(BLACK)
    CIRCLE(160, 100, 20, YELLOW)
    FLIP()                            ' 2. shows what was drawn
WEND
```

- **`SCREEN` comes before the first drawing command.** Without a window, `BOX`
  is an error with exactly this sentence — previously it went on drawing
  invisibly.
- **`FLIP()` shows the frame.** Drachenhauch collects the drawing commands and
  plays them back at `FLIP`; without `FLIP` the window stays empty (the program
  says so at the end).
- **`WHILE NOT QUITREQUESTED()`** — without the loop the program is over after
  the first frame and the window closes immediately.
- **Images and sounds are loaded, not named:** `b = LOADIMAGE("held.png")`
  and then `DRAWIMAGE(b, x, y)`.

## From Python, C and JavaScript

| Familiar | In Drachenhauch |
|---|---|
| `# Kommentar`, `// Kommentar` | `' Kommentar` or `REM` |
| `a == b`, `a != b` | `a = b`, `a <> b` |
| `7 % 3` | `7 MOD 3` |
| `None`, `null` | `NIL` |
| `double`, `long`, `bool`, `str` | `FLOAT`, `INTEGER`, `BOOLEAN`, `STRING` |
| `list`, `dict` | `ARRAY OF T`, `MAP OF T` |
| `"a\nb"`, `"{\"a\": 1}"` | `!"a\nb"`, `!"{\"a\": 1}"` — escape sequences and quotes inside the text only with `!` in front |
| `x++`, `x += 1` | both work |

## Three differences that get no message

They are intentional and therefore run without a warning — you have to know
them:

- **`MID$` and `INSTR` count from 0.** `MID$("Hallo", 1, 2)` is `"al"`, and
  `INSTR` returns `-1` when nothing is found (not `0`).
- **Arrays start at 0.** `DIM a[3] AS INTEGER` has `a[0]` to `a[2]`.
- **`/` can return a fractional number.** `7 / 2` is `3.5`; for integer
  division there is `\` (`7 \ 2` is `3`).
- **`=` on arrays and MAPs asks for THE SAME container**, not for equal
  contents: two arrays with the same numbers are `<>`. Contents are compared
  element by element.
- **`b = a` does not copy an array**, it gives it a second name — whoever
  writes `b[0]` afterwards changes `a[0]` too. The same applies to MAPs and
  objects. A real copy is `b = a[:]`.
