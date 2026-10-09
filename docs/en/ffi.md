# Calling foreign libraries (`DECLARE … LIB`)

What Drachenhauch does not bring along itself is often already on the
computer as a C library: a function of the Windows API, the library of a
measuring device, the system's C library. With `DECLARE … LIB` a
program calls it directly, without a detour through `SHELL` or a new command in
the runtime.

```basic
DECLARE FUNCTION MulDiv LIB "kernel32" (zahl AS LONG, mal AS LONG, durch AS LONG) AS LONG
DECLARE FUNCTION strlen LIB "c" (s AS TEXT) AS ZEIGER

PRINT strlen("Drachenhauch")        ' 12
```

> **Foreign code runs without a safety net.** A wrong pointer, a wrong
> signature or a bug in the library ends the whole program --
> without `CATCH`, without a line number; in the worst case it silently writes into
> someone else's memory. What Drachenhauch can check (argument count, types,
> value ranges, null characters in text), it checks; what happens in the library,
> it does not. For a library you do not trust, there is the
> way out below: [as a task in a separate process](#unsafe-libraries).

## The declaration

```text
DECLARE FUNCTION name LIB "bibliothek" [ALIAS "c_name"] (parameter) AS typ
DECLARE SUB name LIB "bibliothek" [ALIAS "c_name"] (parameter)
```

* **Without `ALIAS`** the name is also the name in the library, spelled
  as in the source code -- in a library, upper/lower case matters
  (`GetTickCount64`, not `gettickcount64`). In the program itself it does not matter,
  as everywhere.
* **With `ALIAS`** the function has a different name in the program than in the library:
  `DECLARE FUNCTION Bytes LIB "kernel32" ALIAS "lstrlenA" (s AS TEXT) AS LONG`.
* The line belongs at the **top level** of the program, not inside a SUB.
  Where exactly it stands does not matter -- even a call before it finds it.
* After that the name is an ordinary function: `--check` counts the
  arguments (an error at compile time) and warns about an argument whose
  type certainly does not fit; hover shows the declaration together with the comment above it,
  *Go to definition* jumps to it.
* A declaration may have the same name as a built-in command
  (`DECLARE FUNCTION abs LIB "c" ...`) -- then the call means the declaration. With
  your own SUB/FUNCTION or a variable of the same name it is an
  error.
* `DECLARE`, `LIB` and `ALIAS` are not keywords; a variable
  may have one of these names.

If you come from VB or QBasic: a `DECLARE` for your own SUBs is not needed
here, and the message says so. `ByVal` in front of a parameter is ignored
(it is the default) -- except in front of a struct, where it means “by value”
(see [Structs by value](#structs-by-value)).

## The types

The type words exist only in the parameter list of a `DECLARE` line -- the
Drachenhauch types do not say how wide a number is in C.

| in `DECLARE` | in C | in Drachenhauch |
|---|---|---|
| `BYTE` / `UBYTE` | `int8_t` / `uint8_t` | INTEGER |
| `SHORT` / `USHORT` | `int16_t` / `uint16_t` | INTEGER |
| `LONG` / `ULONG` | `int32_t` / `uint32_t` (also `int`, Windows `DWORD`) | INTEGER |
| `INTEGER` | `int64_t` | INTEGER |
| `ZEIGER` (also `PTR`) | `void*`, `size_t`, `HWND` ... | INTEGER |
| `SINGLE` | `float` | FLOAT |
| `FLOAT` | `double` | FLOAT |
| `BOOLEAN` | `int` (0 = false) | BOOLEAN |
| `TEXT` (also `CSTR`) | `const char*`, UTF-8 | STRING |
| `WTEXT` (also `WSTR`) | `const wchar_t*` (Windows: UTF-16, otherwise UTF-32) | STRING |
| `BUFFER` | `void*` to the bytes of the buffer | BUFFER |
| `FUNCTION(...) AS typ`, `SUB(...)` | function pointer (callback) | FUNCREF |
| `VALIST` (also `VA_LIST`) | `va_list` (for `vprintf` and friends) | a tuple, an array, a value or NIL |

* **A value that does not fit is an error**: `LONG` with 2^40 aborts
  instead of being silently truncated; so does a floating-point number for `LONG`.
  `ULONG` takes 0 to 4294967295 -- a Windows `INFINITE` is written
  `&HFFFFFFFF`.
* **`long` in C** is 32 bits on Windows, 64 bits on Linux and macOS --
  there it is called `INTEGER`. `size_t` is always `ZEIGER` (pointer).
* **Text** is copied for the call; a null character in the middle of it is an
  error (C would only see the beginning). `NIL` instead of a text passes a
  null pointer.
* **`BUFFER`** passes a pointer to the bytes themselves -- the library
  may write into them, and the program sees it afterwards. This is how structs
  and output texts come back (see below). The program fixes the size
  beforehand; if the library writes beyond it, that is one of the crashes
  from above.
* **`BYREF`** in front of a number type (or `ZEIGER`/`BOOLEAN`) passes a
  pointer to a copy and writes it back after the call -- for
  output parameters. The argument must be a variable, an array element or an
  object member.
* **Return value:** any type except `BUFFER`. `TEXT`/`WTEXT` are copied from the
  pointer (a null pointer becomes `""`); nothing is freed --
  if the library needs that, you
  return `ZEIGER`, read it with `TEXT_AUS_ZEIGER$` and then call the library's
  own release function (see [Pointers](#pointers)).

## Finding the library

The name is **platform-neutral**: `"user32"` becomes `user32.dll`, `"sqlite3"`
becomes `libsqlite3.so` on Linux and `libsqlite3.dylib` on macOS. Two names
are valid everywhere:

* `"c"` -- the system's C library (Windows `ucrtbase.dll`, Linux
  `libc.so.6`, macOS `libSystem`),
* `"m"` -- its mathematics (`pow`, `hypot`; on Windows and macOS the same
  file).

A name with an extension or a path is taken literally (`"libsqlite3.so.0"`,
`"lib/messgeraet.dll"`). The search goes **next to the program, next to `dhrt`
(or the exported executable), then wherever the system searches.**

* **Several names** are separated by `|`: `LIB "libgtk-3-0.dll|libgtk-3.so.0"`.
  The first one that can be loaded wins -- this way one line carries the names
  for all systems where a library is not called the same everywhere.
* **A CONST instead of the text:** `CONST GTK = "libgtk-3-0.dll|libgtk-3.so.0"`
  and then `LIB GTK` in every line. The CONST needs a fixed text;
  in a file with a namespace it belongs to that file.
* **Known only at runtime:** `LIB "$NAME"` takes the contents of the
  environment variable `NAME`, read at the **first call**. So a program
  looks for the library itself and enters it beforehand with
  `SETENV("NAME", pfad)` -- this is how `python.dh` finds the DLL of the Python
  installed on the computer (see [Embedding Python](#embedding-python)).
  The contents may carry several names separated by `|`; `LIB "$NAME|ersatz"` takes
  `ersatz` (fallback) if the variable is empty. If everything is missing, the message names the
  variable.
* **Dependencies on Windows:** with a path (`LIB "C:/Programme/X/x.dll"`)
  Windows looks for the libraries that this DLL itself needs in its folder
  first -- that folder does not have to be in the PATH.
* If a file exists under the name but cannot be loaded, the message
  says so together with the system's reason: usually the file itself is missing a
  library, or it was built for 32 bits.

**Loading happens at the first call**, not at start-up: a program that never takes
the branch also runs without the library. If the library or the function
in it is missing, that is an ordinary runtime error listing the names that were tried --
to be caught with `CATCH`:

```basic
DECLARE FUNCTION messen LIB "messgeraet" () AS FLOAT
TRY
    PRINT messen()
CATCH e
    PRINT "Kein Messgerät: "; e
END TRY
```

## Examples

**Windows API** -- numbers there and back, a struct (see
[Struct layout](#struct-layout)), an output parameter via `BYREF`:

```basic
STRUCT SYSTEMTIME LAYOUT C
    jahr AS USHORT
    monat AS USHORT
    wochentag AS USHORT
    tag AS USHORT
    stunde AS USHORT
    minute AS USHORT
    sekunde AS USHORT
    ms AS USHORT
END STRUCT

DECLARE FUNCTION GetTickCount64 LIB "kernel32" () AS INTEGER
DECLARE SUB GetSystemTime LIB "kernel32" (zeit AS SYSTEMTIME)
DECLARE FUNCTION GetComputerNameW LIB "kernel32" (puffer AS BUFFER, BYREF laenge AS ULONG) AS BOOLEAN

PRINT "seit dem Start: "; GetTickCount64() \ 1000; " s"

DIM st AS SYSTEMTIME
GetSystemTime(st)
PRINT "Jahr "; st.jahr; ", Monat "; st.monat

DIM n AS INTEGER
n = 64                              ' in: room in characters, out: length
DIM b AS BUFFER
b = BUFFER_NEW(128)
IF GetComputerNameW(b, n) THEN
    DIM name AS STRING
    FOR i = 0 TO n - 1
        name = name + CHR$(BUFFER_GET_U16(b, i * 2))
    NEXT
    PRINT "Rechner: "; name
END IF
```

**C library** -- the same lines run on Windows, Linux and macOS:

```basic
DECLARE FUNCTION getenv LIB "c" (name AS TEXT) AS TEXT
DECLARE FUNCTION strtod LIB "c" (s AS TEXT, BYREF ende AS ZEIGER) AS FLOAT
DECLARE FUNCTION hypot LIB "m" (x AS FLOAT, y AS FLOAT) AS FLOAT

PRINT getenv("PATH")
DIM ende AS INTEGER
PRINT strtod("2.5 Meter", ende)     ' 2.5
PRINT hypot(3.0, 4.0)               ' 5.0
```

## Pointers

Some libraries do not return a value but a **pointer** to
memory that belongs to them: a text the caller is supposed to free afterwards,
an array of bytes, a struct. Three commands connect `ZEIGER` and
`BUFFER`:

| Command | what it does |
|---|---|
| `TEXT_AUS_ZEIGER$(zeiger [, breit])` | copies the text behind a pointer up to the null character (UTF-8; with `breit` (wide) = TRUE `wchar_t`, i.e. UTF-16 on Windows); 0 gives `""` |
| `BUFFER_AUS_ZEIGER(zeiger, laenge)` | copies `laenge` (length) bytes behind a pointer into a new BUFFER; 0 bytes give an empty one, a null pointer with a length is an error |
| `BUFFER_ZEIGER(puffer)` | the address of the bytes of a BUFFER, for a struct field that points to another buffer; an empty buffer gives 0 |

```basic
DECLARE FUNCTION malloc LIB "c" (n AS ZEIGER) AS ZEIGER
DECLARE FUNCTION strcpy LIB "c" (ziel AS ZEIGER, quelle AS TEXT) AS ZEIGER
DECLARE SUB free LIB "c" (z AS ZEIGER)

DIM z AS INTEGER
z = malloc(16)                             ' 16 bytes that belong to the C library
strcpy(z, "Grüße")
PRINT TEXT_AUS_ZEIGER$(z)                  ' Grüße
PRINT BUFFER_LEN(BUFFER_AUS_ZEIGER(z, 8))  ' 8: seven bytes of UTF-8 and the null
free(z)                                    ' the memory belongs to the library
```

* **All three copy** -- a text or BUFFER taken from a pointer no longer depends
  on the library's memory afterwards; the library may free it.
* **They trust the pointer.** A wrong or already freed pointer
  is one of the crashes from above; a length that is too large reads past the end.
  What can be checked is checked: null pointer, negative length,
  at most 1 GiB.
* **`BUFFER_ZEIGER` is valid as long as the buffer keeps its size.**
  `BUFFER_RESIZE` moves the bytes somewhere else, and the old pointer then points into
  nothing. The library may only keep the pointer as long as the program leaves
  the buffer unchanged.

## Struct layout

A C struct is a piece of memory with fields at fixed positions. With
`STRUCT name LAYOUT C` you describe it with the same type words as in the
`DECLARE` line -- Drachenhauch works out the positions including alignment:

```basic
STRUCT Punkt LAYOUT C
    x AS LONG
    y AS LONG
END STRUCT

STRUCT Linie LAYOUT C
    farbe AS UBYTE
    ende[2] AS Punkt          ' an array of two embedded structs
    name AS TEXT * 16         ' 16 characters of fixed width
    gewicht AS FLOAT
END STRUCT

DIM l AS Linie                ' a BUFFER of exactly this size, full of zeros
l.ende[1].x = 40
l.name = "Diagonale"
PRINT l.ende[1].x, l.name     ' 40  Diagonale
PRINT SIZEOF(Linie), OFFSETOF(Linie, name)   ' 48  20
```

* **A variable of this type is a BUFFER** (`TYPEOF` says `BUFFER`) --
  it goes straight to a `DECLARE` function whose parameter is `AS BUFFER`
  or directly `AS Linie`, and the library writes into it.
* **Fields:** the type words of the `DECLARE` line except `BUFFER`; `BOOLEAN` is
  4 bytes like `int` (a C `bool` with one byte is `UBYTE`). In addition,
  `name[n] AS typ` for an array of elements, `TEXT * n` or `WTEXT * n`
  for characters of fixed width (read up to the null character; a text that is too long
  is an error) and another `STRUCT … LAYOUT C` -- it may also stand
  further down, but may not contain itself. A pointer to text
  or another buffer is a `ZEIGER` field (see below). `DIM` in front of
  the field is allowed, not required.
* **The rules are those of C:** every field lies at a position that is
  divisible by its size (a struct by that of its strictest
  field), and the struct is long enough for a second one to fit directly behind
  it. **`PACK n`** (`STRUCT Kopf LAYOUT C PACK 1`) limits the
  alignment like `#pragma pack(n)` -- for file formats and the few APIs
  that are packed.
* **`SIZEOF(typ)`** and **`OFFSETOF(struct, feld)`** are fixed numbers at
  compile time; `SIZEOF` also knows the type words (`SIZEOF(ZEIGER)` is 8).
* **Reading and writing check** like the `DECLARE` line: a value that
  does not fit into the field is an error, as is an index out of range or
  a buffer that is shorter than the struct.
* **A struct is a BUFFER, and therefore a reference:** `b = a` shares the bytes;
  `BUFFER_SLICE(a, 0, SIZEOF(Punkt))` makes a copy. Conversely, any
  buffer can be read through a layout (`DIM p AS Punkt : p = roh`).
* A struct lives in a variable, a parameter, a return value or a field of
  a class; many of them in a row are an array of structs (see below).
  Struct names are valid in the whole program, also from a file with a
  namespace.

| Command | what it does |
|---|---|
| `SIZEOF(typ)` | size of a `STRUCT … LAYOUT C` or of a type word in bytes, computed at compile time |
| `OFFSETOF(struct, feld)` | position of a field in the struct in bytes, computed at compile time |

Some structs want their own size in the first field before the call --
on Windows, for example, `MEMORYSTATUSEX` for `GlobalMemoryStatusEx`:

```basic
STRUCT MEMORYSTATUSEX LAYOUT C
    laenge AS ULONG
    auslastung AS ULONG
    gesamt AS INTEGER
    frei AS INTEGER
    seiten_gesamt AS INTEGER
    seiten_frei AS INTEGER
    virtuell_gesamt AS INTEGER
    virtuell_frei AS INTEGER
    erweitert_frei AS INTEGER
END STRUCT
DECLARE FUNCTION GlobalMemoryStatusEx LIB "kernel32" (m AS MEMORYSTATUSEX) AS BOOLEAN

DIM m AS MEMORYSTATUSEX
m.laenge = SIZEOF(MEMORYSTATUSEX)
IF GlobalMemoryStatusEx(m) THEN PRINT "frei: "; m.frei \ 1048576; " MB"
```

On Linux and macOS, `struct utsname` (for `uname`) has fields of fixed width,
65 bytes on Linux and 256 on macOS -- the layout depends on the system there.
You give the function the larger struct and afterwards read through the layout
of the system (`tests/pruef/ffi_struct.dhtest` shows how).

### Arrays of structs and structs in classes

`DIM pts[n] AS Punkt` creates `n` structs **in a row in one buffer**, just
like a C array `Punkt pts[n]` -- so it goes as a `Punkt*` to a library that
wants many at once:

```basic
STRUCT POINT LAYOUT C
    x AS LONG
    y AS LONG
END STRUCT
DECLARE FUNCTION CreatePolygonRgn LIB "gdi32" (pts AS POINT, n AS LONG, modus AS LONG) AS ZEIGER

DIM dreieck[3] AS POINT
dreieck[1].x = 100
dreieck[2].y = 100
DIM r AS INTEGER
r = CreatePolygonRgn(dreieck, LEN(dreieck), 1)
```

* **`LEN(pts)`** counts the structs (`BUFFER_LEN` the bytes); an index out
  of range is an error. The size may be known only at run time
  (`DIM pts[n] AS Punkt`).
* **`ARRAY OF Punkt`** as a parameter or return type of a FUNCTION means
  the same array; `DIM ps AS ARRAY OF Punkt` without a size is an empty one
  that receives an assignment.
* **Several dimensions** as in C: `DIM g[n, 4] AS Punkt` lies row by row
  in one buffer (`g[i, j]` is element `i * 4 + j`). The rear sizes must be
  fixed at compile time (a number or `CONST`), only the first may come at
  run time; every index is checked against its own size. `LEN(g)` counts
  the first dimension, as for every array. As an `ARRAY OF Punkt`
  parameter it is a flat array (`LEN` = all elements) -- the library gets
  the same bytes.
* **An element as a whole is a copy:** `p = pts[2]` copies the bytes into
  a struct of its own, `pts[2] = p` copies them back; the same goes for a
  struct inside a struct (`linie.b = p`). To a library, on the other hand,
  an element goes as a **pointer to its place** in the array
  (`GetCursorPos(pts[2])` writes into the array), and so do `linie.b` and
  an array of numbers inside a struct (`st.werte`).
* **`FOR EACH p IN pts`** yields a copy of the element in every round --
  also over an array inside a struct (`FOR EACH e IN linie.ecken`,
  `FOR EACH w IN st.werte`). Writing a field of the copy (`p.x = 1`) is
  therefore a message; you write into the array with an index. `FOR EACH`
  works over one dimension only, as for every array.
* **Directly on the result of a call** both work: `reihe(5)[0].x`,
  `ort(7).y`, `LEN(reihe(4))`, `FOR EACH p IN reihe(3)`. The call runs
  exactly once. Like an array, a struct comes back as a reference --
  `f()[1].x = 3` writes into the buffer `f` returned.

**In a class** a struct is an ordinary field; every object gets its own
buffer at `NEW`:

```basic
STRUCT Punkt LAYOUT C
    x AS LONG
    y AS LONG
END STRUCT
CLASS Figur
    DIM ort AS Punkt
    DIM ecken[3] AS Punkt          ' an array of structs, fixed size
    SUB setze(x AS INTEGER, y AS INTEGER)
        Self.ort.x = x
        ort.y = y                  ' in a method also without Self
    END SUB
END CLASS

DIM f AS Figur
f = NEW Figur()
f.setze(3, 4)
f.ecken[1].x = 7
PRINT f.ort.x, f.ecken[1].x, LEN(f.ecken)     ' 3  7  3
```

* `f.ort` as a whole is the buffer -- it goes to a library that way, and an
  assignment shares the bytes as with any struct variable.
* An array of structs in a class needs a fixed size.

### Bit fields

Some structs pack several small numbers into one word -- in C
`DWORD fBinary : 1;`. In Drachenhauch the width in bits goes after the type
(as in C) or, as in FreeBASIC, after the name:

```basic
STRUCT Status LAYOUT C
    bereit AS ULONG : 1
    modus AS ULONG : 2
    stufe AS LONG : 5           ' signed: -16 to 15
    fehler : 1 AS BOOLEAN       ' the FreeBASIC way of writing it
    zaehler AS ULONG
END STRUCT

DIM s AS Status
s.modus = 3
s.stufe = -2
PRINT s.modus, s.stufe, HEX$(BUFFER_GET_U32(s, 0))   ' 3  -2  F6
```

* **A bit field reads and writes only its own bits**; a value that does not
  fit is an error (`s.modus = 4` -- "passt nicht in ein Bitfeld mit 2 Bits
  (0 bis 3)", does not fit in a bit field with 2 bits). Signed types (`BYTE`,
  `SHORT`, `LONG`, `INTEGER`) give back a negative number, unsigned ones
  (`UBYTE` … `ULONG`) do not. `BOOLEAN` takes `TRUE`/`FALSE` (its unit is
  4 bytes, like `int`; a C `bool` with one bit is `UBYTE : 1`).
* **How bit fields are laid out is decided by the system's compiler, and
  dhrt follows it:** on Windows (MSVC) consecutive bit fields share a unit
  of their type only if the type has the same size and the bits still fit;
  on Linux and macOS (GCC, Clang) a bit field goes to the next free bit
  position as long as it does not cross a boundary of its type -- even right
  after an ordinary field. The same declaration can therefore have
  different sizes on different systems, just as in C (`char c; int a : 4;`
  is 8 bytes on Windows, 4 elsewhere).
* **Without a name** as in C: `AS ULONG : 3` are padding bits (they cannot
  be read or written), `AS ULONG : 0` ends the current unit -- the next
  field starts in a new one. A bit field with 0 bits never has a name.
* **With `PACK`** the system's rules apply again: MSVC keeps its units and
  aligns them to at most n bytes; GCC and Clang then lay bit fields bit by
  bit, even across the boundary of their type (`PACK 1` with
  `a AS BYTE : 3` and `b AS LONG : 30` puts `b` from bit 3 into five
  bytes). A field with 0 bits aligns to the full type with GCC and Clang,
  even under `PACK`; with MSVC it only has an effect right after a bit
  field.
* **What counts towards the struct's alignment** differs once more:
  padding bits and fields with 0 bits count on Windows and on Linux on
  ARM64, not on Linux on x86-64 and not on macOS. This was measured with
  clang for all five targets and with the real gcc and `cl.exe`.
* `OFFSETOF` of a bit field is an error (it has no position in bytes), and
  there are no arrays of bit fields.

### Structs with pointers

A field of type `char*` or `void*` is a `ZEIGER`. It is set with
`BUFFER_ZEIGER` and read with `TEXT_AUS_ZEIGER$` or `BUFFER_AUS_ZEIGER`:

```basic
STRUCT Eintrag LAYOUT C
    name AS ZEIGER            ' const char*
    laenge AS LONG
END STRUCT

DIM name AS BUFFER
name = BUFFER_CONCAT(BUFFER_FROM_STRING("Drache"), BUFFER_NEW(1))   ' with the null
DIM e AS Eintrag
e.name = BUFFER_ZEIGER(name)
e.laenge = 6
PRINT TEXT_AUS_ZEIGER$(e.name), SIZEOF(Eintrag)   ' Drache  16
```

An element that a library passes by pointer -- such as the two
elements in the comparison of `qsort` -- is also read through the layout:
`DIM a AS Punkt : a = BUFFER_AUS_ZEIGER(zeiger, SIZEOF(Punkt))`.

### Structs by value

Some C functions take a struct not as a pointer (`Punkt*`) but
by value (`Punkt`), or they return one -- `div` returns a `div_t`,
graphics libraries take points and colours this way. For that, **`BYVAL`** goes in front of
the parameter, and the return type is simply the name of the struct:

```basic
STRUCT Komplex LAYOUT C
    re AS FLOAT
    im AS FLOAT
END STRUCT
DECLARE FUNCTION csqrt LIB "m" (BYVAL z AS Komplex) AS Komplex
DECLARE FUNCTION cabs LIB "m" (BYVAL z AS Komplex) AS FLOAT

DIM z AS Komplex
z.re = -4
DIM w AS Komplex
w = csqrt(z)
PRINT w.re, w.im, cabs(z)     ' 0.0  2.0  4.0
```

* **Without `BYVAL` it stays a pointer** -- as before; `p AS Punkt` means
  `Punkt*` in C, `BYVAL p AS Punkt` means `Punkt`. The C declaration tells you
  which one is meant.
* **The function gets a copy.** Whatever it changes in it, the
  program does not see; the buffer must be at least as long as the struct.
* **A return value is a new struct** (a BUFFER of its size), which
  you assign to a variable of that type. If the library returns a
  *pointer* to a struct, the return type stays `ZEIGER`.
* **How the struct travels is decided by the system**, and Drachenhauch follows
  its rules: on Windows x64 a struct of 1, 2, 4 or
  8 bytes goes in a register, any other as a pointer to a copy; on
  Linux and macOS on Intel (System V) up to 16 bytes go in registers
  -- floating-point numbers in the SSE registers, everything else in the integer registers --,
  anything larger on the stack; on ARM (Linux and Apple) up to four
  equal floating-point numbers go in floating-point registers, otherwise up to 16 bytes in
  integer registers and anything larger as a pointer. In the program you see none of
  this.
* **Complex numbers** (`double complex`, `_Dcomplex`) are treated by C exactly like
  a struct of two floating-point numbers -- `csqrt`, `cexp` and friends from the
  C library work with them as shown above.
* In a **callback** a struct works by value as well, see
  [Structs by value in a callback](#structs-by-value-in-a-callback).
* Not (yet) by value: a struct in a function with **`...`**; on ARM, a
  struct of floating-point numbers for which no register is left free after
  eight floating-point arguments. Both produce a message, not a silent error.

## GTK

[GTK](https://www.gtk.org/) is the library most programs on Linux use to
draw their windows. With `DECLARE … LIB` and callbacks
it can be used directly -- the example `207_gtk.dh` builds a window
with a text input, buttons and a label:

```basic
CONST GTK = "libgtk-3-0.dll|libgtk-3.so.0|libgtk-3.0.dylib"
CONST GOBJECT = "libgobject-2.0-0.dll|libgobject-2.0.so.0|libgobject-2.0.0.dylib"

DECLARE FUNCTION gtk_init_check LIB GTK (argc AS ZEIGER, argv AS ZEIGER) AS BOOLEAN
DECLARE FUNCTION gtk_window_new LIB GTK (art AS LONG) AS ZEIGER
DECLARE FUNCTION gtk_button_new_with_label LIB GTK (text AS TEXT) AS ZEIGER
DECLARE SUB gtk_container_add LIB GTK (behaelter AS ZEIGER, kind AS ZEIGER)
DECLARE SUB gtk_widget_show_all LIB GTK (w AS ZEIGER)
DECLARE SUB gtk_main LIB GTK ()
DECLARE SUB gtk_main_quit LIB GTK ()
DECLARE FUNCTION signal LIB GOBJECT ALIAS "g_signal_connect_data" _
    (objekt AS ZEIGER, name AS TEXT, rueckruf AS SUB(objekt AS ZEIGER, daten AS ZEIGER), _
     daten AS ZEIGER, freigabe AS ZEIGER, flags AS LONG) AS ULONG

SUB geklickt(knopf AS INTEGER, daten AS INTEGER)
    PRINT "geklickt"
END SUB

SUB zu(fenster AS INTEGER, daten AS INTEGER)
    gtk_main_quit()
END SUB

IF gtk_init_check(0, 0) THEN
    DIM fenster AS INTEGER
    fenster = gtk_window_new(0)
    DIM knopf AS INTEGER
    knopf = gtk_button_new_with_label("Klick mich")
    gtk_container_add(fenster, knopf)
    signal(knopf, "clicked", geklickt, 0, 0, 0)
    signal(fenster, "destroy", zu, 0, 0, 0)
    gtk_widget_show_all(fenster)
    gtk_main()                     ' GTK's loop; callbacks come from here
END IF
```

* **In C, `g_signal_connect` is only a macro** -- what is called is
  `g_signal_connect_data`. Because every kind of signal has a different callback,
  each form gets its own `DECLARE` line with `ALIAS` to the same
  C function.
* `gtk_main` blocks until `gtk_main_quit` is called; the callbacks run
  on the program's thread. There is no raylib window alongside it --
  a program uses either GTK or its own gui.
* GTK has to be on the computer: on Linux almost always, on macOS via
  Homebrew, on Windows via MSYS2 or from a program that brings
  it along (the example tries the one from Inkscape).
* Properties are set with `g_object_set` -- a function with `...`, see
  [Variable argument count](#variable-argument-count):
  `g_object_set(knopf, "tooltip-text", "Sagt Hallo", NIL)`.

## Embedding Python

Python is itself a C library (`python3XY.dll`, `libpython3.X.so`) --
with it, every package installed in the computer's Python is available:
numpy, Pillow, PySide6 (Qt) and whatever else. The library
`examples/python/python.dh` takes care of the declarations; you copy it
next to your own program and import it:

```text
IMPORT "python/python.dh"

pythonStarten()
pythonAusfuehren("import statistics")
PRINT pythonZahl("statistics.median([3, 1, 4, 1, 5])")     ' 3.0

pythonSetzeText("name", "Drache")
PRINT pythonText$("name.upper() + '!'")                     ' DRACHE!
```

| Command | What it does |
|---|---|
| `pythonStarten([programm$])` | looks for a Python (argument, `DH_PYTHON`, `py -3`, `python3`, `python`) and starts it; a second call does nothing |
| `pythonVorhanden()` | TRUE if one can be started -- for programs that have a way without Python |
| `pythonAusfuehren(code$)` | statements, also several lines (`import`, `def`, `class`) |
| `pythonZahl(a$)`, `pythonGanz(a$)`, `pythonText$(a$)` | evaluate an expression -- as a floating-point number, INTEGER or text |
| `pythonBytes(a$)` | `bytes(...)` as a BUFFER -- a numpy array returns its raw data |
| `pythonSetzeZahl/Ganz/Text/Bytes(name$, wert)` | store a value under a name in Python |

* **Which Python:** the one a call on the command line would
  give you, or the one in `DH_PYTHON` (the path to the executable, for example
  `.venv/Scripts/python.exe`). It is asked once for its library and
  its search path; embedded, it gets the same search path, so
  also the packages of a venv and in the user folder.
* **Arrays** travel as bytes: a BUFFER of floating-point numbers is, in Python,
  `numpy.frombuffer(roh, '<f8')`, and `pythonBytes("feld.astype('<f8')")`
  brings the result back. The example `208_python.dh` computes the
  spectrum of a signal this way -- with numpy or, without numpy, in pure Python.
* **Errors in Python** arrive as errors in Drachenhauch:
  `Python: ZeroDivisionError: division by zero`, for multi-line code with
  the line after it. `CATCH` catches them, and the program goes on afterwards.
* **Qt** works via PySide6 in the embedded Python: a window with
  `QApplication` opens from within the Drachenhauch program. As long as
  `app.exec()` is running, the Drachenhauch program waits.
* **Limits:** Python runs in the same process -- if an extension crashes,
  the program is gone. What Python outputs with `print` does not appear in the
  order of the `PRINT` lines; better fetch results. An exported
  program does not take Python along; the computer needs it.
  `pythonStarten` sets `PYTHONHOME` -- a Python started later inherits
  it.
* Tested on Windows with Python 3.12 and 3.14 (numpy, PySide6). On
  Linux and macOS, `dhrt` loads foreign libraries globally for this (otherwise
  Python's extensions would not find its functions); there the
  Python needs a shared library (`libpython3.X.so`, on Linux for example
  from the package `libpython3-dev`).

## Variable argument count

`...` at the end of the parameters takes any number of further values, as with
`printf` in C:

```basic
DECLARE FUNCTION sprintf LIB "msvcrt|c" (ziel AS BUFFER, format AS TEXT, ...) AS LONG

DIM b AS BUFFER
b = BUFFER_NEW(128)
sprintf(b, "%d Drachen, %.1f Meter, %s", 3, 4.5, "feuerrot")
PRINT TEXT_AUS_ZEIGER$(BUFFER_ZEIGER(b))      ' 3 Drachen, 4.5 Meter, feuerrot
```

* **The type of the further values comes from the value:** INTEGER and BOOLEAN as
  an integer (64 bits -- for `%d` as well as for `%lld`), FLOAT as `double`, a text
  as `const char*` (UTF-8, copied), a BUFFER as a pointer to its bytes,
  `NIL` as a null pointer (the terminator of `g_object_set`). C does not know more
  than that behind `...` anyway -- smaller numbers and `float` become
  `int` or `double` there.
* Before `...` there is at least one fixed parameter; `--check` counts the fixed ones.
* What the function does with the values is decided solely by its format --
  a `%s` for a number is one of the crashes from above.
* **On Windows** the `printf` family lives in `msvcrt`, not in the
  C library `ucrtbase` -- hence `LIB "msvcrt|c"`.
* Every system calls such functions slightly differently (on Linux on x86-64
  a small trampoline sets the register `al`, on Apple ARM the
  further values lie on the stack) -- dhrt takes care of that.

### va_list

Many functions with `...` have a sister that takes the values as **one**
list: `vprintf`, `vsprintf`, `vsnprintf`, on Windows `wvsprintfA` -- and
libraries that pass on a message that way. The parameter is called
`VALIST`, the argument is a tuple of the values:

```basic
DECLARE FUNCTION vsprintf LIB "msvcrt|c" (ziel AS BUFFER, format AS TEXT, werte AS VALIST) AS LONG

DIM b AS BUFFER
b = BUFFER_NEW(128)
vsprintf(b, "%d Drachen, %.1f Meter, %s", (3, 4.5, "feuerrot"))
PRINT TEXT_AUS_ZEIGER$(BUFFER_ZEIGER(b))      ' 3 Drachen, 4.5 Meter, feuerrot
```

* **The list** is a tuple, an array (`[1, 2, 3]`), a single value (for
  exactly one) or `NIL` (none). Each value is passed as after `...`: whole
  numbers with 64 bits, floating-point numbers as `double`, text as a
  copied `const char*`, a BUFFER as a pointer to its bytes.
* **A `va_list` looks different on every system** -- on Windows and on
  Apple ARM a pointer to the values, on Linux and macOS on Intel a struct
  saying that the registers are already used up and everything lies behind
  it, on Linux ARM a similar one. dhrt builds it anew for every call; the
  function may use it up.
* `VALIST` only exists as a parameter -- not as a return type, not with
  `BYREF` and not (yet) in a callback (a `va_list` that a library hands to
  a callback arrives as a `ZEIGER`).

## Callbacks

Some libraries call back: `qsort` asks for every pair which
element comes first, `EnumWindows` reports every window one by one. The
parameter for this is written in the `DECLARE` line the way the callback looks
-- FreeBASIC's notation:

```basic
DECLARE SUB qsort LIB "c" (feld AS BUFFER, n AS ZEIGER, groesse AS ZEIGER, _
                           vergleich AS FUNCTION(a AS ZEIGER, b AS ZEIGER) AS LONG)

FUNCTION zahlBei(z AS INTEGER) AS INTEGER
    RETURN BUFFER_GET_I32(BUFFER_AUS_ZEIGER(z, 4), 0)
END FUNCTION

FUNCTION vergleiche(a AS INTEGER, b AS INTEGER) AS INTEGER
    RETURN SGN(zahlBei(a) - zahlBei(b))
END FUNCTION

DIM b AS BUFFER
b = BUFFER_NEW(12)
BUFFER_SET_I32(b, 0, 42)
BUFFER_SET_I32(b, 4, -7)
BUFFER_SET_I32(b, 8, 13)
qsort(b, 3, 4, vergleiche)                                  ' -7 13 42
qsort(b, 3, 4, FUNCTION(a, b) SGN(zahlBei(b) - zahlBei(a)))   ' 42 13 -7
```

* **The parentheses name the C types** the callback receives, with or
  without names (`FUNCTION(ZEIGER, ZEIGER) AS LONG`). Allowed are the
  number types, `ZEIGER`, `BOOLEAN` and `TEXT`/`WTEXT` (arrives as a STRING);
  no `BUFFER` (the library's memory is a `ZEIGER`) and no `BYREF`.
  A callback returns a number, a `ZEIGER` or `BOOLEAN`, or, as
  `SUB(...)`, nothing. A struct by value works in both directions, see
  below.
* **What you pass is a function**: its name without parentheses, a bound
  method (`zaehler.eins` -- the object comes along) or a lambda. `NIL`
  passes a null pointer. If the number of its parameters does not match, that is
  an error before the library is even called.
* **An error in the callback** arrives at the library call as soon as the library
  returns -- as an ordinary runtime error, catchable with `CATCH`.
  It cannot abort through the library; from the error onwards
  every further callback of this call returns 0, and the library
  finishes its work with these answers. The same applies to a value of the
  wrong type (a BOOLEAN function where `LONG` is required).
* **A callback may itself call libraries**, even the same one.
* **Only on the program's thread.** If a library calls back from its own
  thread (`CreateThread`, `pthread_create`, some drivers), the
  callback is not executed -- it returns 0, and the next library call
  that returns reports it. That may already be the one that started the
  thread (its result is then lost); on Windows,
  `CREATE_SUSPENDED` and your own `ResumeThread` help. The VM is not
  built for several threads.
* **A callback stays valid until the program ends** -- the library
  may keep it and call it later. For the same function it always gets
  the same entry point; a lambda that is created anew in a loop every time
  gets a new one every time (a few bytes that stay until the end).
  An error in a callback that the library calls outside a
  call reports itself at the next library call.

### Structs by value in a callback

If the library passes a struct by value (C: `int f(Punkt p)`) or expects
one back, it is written in the callback just as in the `DECLARE` line --
with `BYVAL` as a parameter, with its name as the return type:

```basic
STRUCT Punkt LAYOUT C
    x AS LONG
    y AS LONG
END STRUCT
DECLARE SUB zeichne LIB "grafik" (n AS LONG, _
    ort AS FUNCTION(BYVAL p AS Punkt, i AS LONG) AS Punkt)

FUNCTION verschiebe(p AS Punkt, i AS INTEGER) AS Punkt
    DIM r AS Punkt
    r.x = p.x + i * 10
    r.y = p.y
    RETURN r
END FUNCTION

zeichne(5, verschiebe)
```

* **The function gets a copy** in a BUFFER the size of the struct --
  whatever it changes in it, the library does not see.
* **It returns a struct** (a BUFFER at least as long as the struct); a
  buffer that is too short or a value of another kind is an error that,
  like any error in a callback, arrives at the library call.
* **Without `BYVAL` it is an error:** if the library passes a *pointer* to
  a struct (C: `Punkt*`), the parameter is `ZEIGER`, and
  `BUFFER_AUS_ZEIGER(z, SIZEOF(Punkt))` reads it.
* How the struct travels is again decided by the system -- the same rules
  as above, only in the opposite direction: what arrives in registers, the
  entry point puts back together.

## Export

`dhrt --export` takes along every library that lies **next to the program**
and puts it next to the executable -- that is where the runtime looks first. A name with
a path (`"lib/messgeraet.dll"`) keeps its folder. Only
libraries whose functions the program actually calls are counted; the
C library (`"c"`, `"m"`) never. Whatever does not lie next to it, the export names
as a hint -- every target computer has a system library like `kernel32`,
a foreign one you have to ship yourself:

```text
  Bibliothek "messgeraet" mitkopiert: messgeraet.dll
  Hinweis: Bibliothek "kernel32" liegt nicht neben dem Programm -- der Zielrechner muss sie haben (eine Systembibliothek wie kernel32 hat er)
```

What is taken along is the file **of the system on which the export runs**
(`messgeraet.dll` on Windows, `libmessgeraet.so` on Linux) -- an
export only runs on that system anyway.

## Unsafe libraries

A library that might crash is called in a **task**
(`TASK_START`): it runs as a separate process. If it crashes, the
main program gets an error when collecting the result, and keeps running.

```basic
DECLARE FUNCTION messen LIB "messgeraet" () AS FLOAT

FUNCTION einmalMessen() AS STRING
    RETURN STR$(messen())
END FUNCTION

DIM a AS INTEGER
a = TASK_START(einmalMessen)
WHILE NOT TASK_READY(a)
    SLEEP(10)
WEND
TRY
    PRINT "Wert: "; TASK_RESULT$(a)
CATCH e
    PRINT e         ' "TASK: the task crashed ..."
END TRY
```

## What does not exist (yet)

* A struct by value behind `...` (see
  [Structs by value](#structs-by-value)).
* C++ names, COM. A C++ library such as
  Qt works via a detour with a C interface -- for example
  [Embedding Python](#embedding-python) with PySide6.
* **In the browser** there are no foreign libraries; a call there is an
  error saying exactly that.

**In machine code:** a loop that calls a foreign function is compiled
like any other. If the function takes and returns only numbers (whole
numbers, `ZEIGER`, `BOOLEAN`, `SINGLE`, `FLOAT`, without `BYREF`), the call
runs in the fast, typed part -- a million calls of `abs` in a loop then
take 54 instead of 116 ms in the VM. **As soon as the program has handed out
a callback, loops with library calls stay in the VM**: a library may keep a
callback and call it at any later call (GTK does exactly that), and the
callback may change variables that a compiled loop is holding itself. That
changes nothing about the result, only about the speed.

## Under the hood

The compiler turns every call into the internal command `__ffi` and passes it
the declaration. For every callback the runtime builds an entry point with
the C signature, which collects the arguments and calls the function via the VM.
The runtime loads the library at the first call
(`libloading`) and builds, once per signature, a small transition with
Cranelift -- the same library that also generates the machine code; it
places the arguments according to the system's C calling convention. Design and
decisions: [entwurf-ffi.md](../entwurf-ffi.md) (German), source code
`rust/drachenhauch_runtime/src/ffi.rs`, tests
`tests/pruef/ffi.dhtest`.
