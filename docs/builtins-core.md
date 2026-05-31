# Standard-Built-ins

Alle eingebauten Befehle, die ohne `IMPORT` verfügbar sind. Grafik-Befehle (SCREEN, BOX, …) sind separat in [builtins-grafik.md](builtins-grafik.md) dokumentiert.

## Inhalt

- [Konvertierung](#konvertierung)
- [Math](#math)
- [Strings](#strings)
- [Bitwise](#bitwise)
- [Arrays](#arrays)
- [Maps](#maps)
- [Datei-I/O](#datei-io)
- [Zeit & Random](#zeit--random)
- [Spiel-Helfer](#spiel-helfer)

## Konvertierung

| Funktion | Zweck |
|---|---|
| `STR$(v)` → STRING | Wert nach String. Bools werden zu `"TRUE"`/`"FALSE"`, Floats wie `3.0` zu `"3.0"`. |
| `VAL(s$)` → INTEGER/FLOAT | String zu Zahl. Mit `.` → FLOAT, sonst INT. Ungültiges → `0`. |
| `INT(v)` → INTEGER | Zahl zu INT (`floor`). `INT(3.7)` = 3, `INT(-1.5)` = -2. |
| `ABS(v)` | Absolutbetrag. |
| `CHR$(n)` → STRING | Unicode-Codepoint zu 1-Zeichen-String. `CHR$(65)` = `"A"`. |
| `ASC(s$)` → INTEGER | Codepoint des ersten Zeichens. `ASC("Anna")` = 65. |
| `RGB(r, g, b)` → INTEGER | 3 INTs (0..255) zu 24-Bit-Farbe. `RGB(255, 128, 0)` = `0xFF8000`. |
| `HEX$(n)` → STRING | INT zu Hex-Großbuchstaben (ohne `0x`). `HEX$(255)` = `"FF"`. |

```basic
DIM s AS STRING
s = STR$(3.14)              ' "3.14"

DIM n AS INTEGER
n = VAL("42")               ' 42
n = VAL("3.7")              ' 3 (truncated, weil INT-Default)

DIM f AS FLOAT
f = VAL("3.7")              ' 3.7

PRINT CHR$(65) + CHR$(66)   ' "AB"
PRINT HEX$(RGB(255, 0, 0))  ' "FF0000"
```

## Math

| Funktion | Zweck |
|---|---|
| `SIN(x)`, `COS(x)`, `TAN(x)` | Trigonometrie (x in Radiant) |
| `ATAN(x)`, `ATAN2(y, x)` | Arcus-Tangens |
| `SQR(x)` | Quadratwurzel (x ≥ 0) |
| `POW(b, e)` | b hoch e |
| `EXP(x)` | e^x |
| `LOG(x[, base])` | Logarithmus, default natürlich |
| `FLOOR(x)`, `CEIL(x)`, `ROUND(x)` | INT-Konvertierung |
| `MIN(a, b, ...)`, `MAX(a, b, ...)` | variadic |
| `CLAMP(v, lo, hi)` | beschränkt v auf `[lo, hi]` |
| `SIGN(x)` | -1, 0 oder 1 |

```basic
PRINT SIN(PI / 2)            ' 1.0
PRINT SQR(2)                 ' 1.4142...
PRINT MIN(5, 2, 9, 1, 7)     ' 1
PRINT MAX(5, 2, 9, 1, 7)     ' 9
PRINT CLAMP(150, 0, 100)     ' 100
PRINT SIGN(-7)               ' -1

' Vektor-Länge und Winkel
DIM laenge AS FLOAT
DIM winkel_grad AS FLOAT
laenge = SQR(3 * 3 + 4 * 4)            ' 5.0
winkel_grad = ATAN2(4.0, 3.0) * 180.0 / PI
```

## Strings

Funktionen mit `$`-Suffix gibt es auch ohne (`UPPER$` ≡ `UPPER`).

| Funktion | Zweck |
|---|---|
| `LEN(s)` → INTEGER | Länge (auch für Arrays) |
| `UPPER$(s)`, `LOWER$(s)` | Groß-/Kleinschreibung |
| `LEFT$(s, n)`, `RIGHT$(s, n)` | erste/letzte n Zeichen |
| `MID$(s, start[, n])` | Teilstring ab Position start (0-basiert), n Zeichen oder bis Ende |
| `INSTR(s, sub[, start])` → INTEGER | Position von sub in s, oder -1 |
| `REPLACE$(s, alt, neu)` | alle Vorkommen ersetzen |
| `TRIM$(s)` | Whitespace vorne/hinten weg |
| `SPLIT$(s, delim)` → ARRAY OF STRING | Zerlegen |
| `JOIN$(arr, delim)` → STRING | Vereinen (Array OF STRING) |
| `PADL$(s, breite[, fill])`, `PADR$(...)` | Auffüllen links/rechts |
| `REPEAT$(s, n)` | s n-mal aneinanderhängen |
| `SPACE$(n)` | n Leerzeichen |
| `HEX$(n)` | INT als Hex-String |
| `FORMAT$(value, mask)` | printf-Stil. `FORMAT$(42, "%05d")` → `"00042"`. Mask folgt Python's `%`-Operator (`%d`, `%s`, `%.2f`, `%05X`, …) |

```basic
PRINT UPPER$("hallo")           ' "HALLO"
PRINT LEFT$("GameBasic", 4)     ' "Game"
PRINT MID$("GameBasic", 4, 5)   ' "Basic"
PRINT INSTR("hello world", "world")   ' 6
PRINT REPLACE$("a-b-c", "-", "_")     ' "a_b_c"

DIM teile AS ARRAY OF STRING
teile = SPLIT$("Anna,Bert,Cilly", ",")
PRINT JOIN$(teile, " | ")             ' "Anna | Bert | Cilly"

PRINT PADL$("42", 6, "0")             ' "000042"
PRINT REPEAT$("=*", 5)                ' "=*=*=*=*=*"
PRINT HEX$(0xCAFE)                    ' "CAFE"
```

## Bitwise

Alle nehmen INTEGER, geben INTEGER. Negative Werte wie in Python (Zweierkomplement, beliebig groß).

| Funktion | Operator |
|---|---|
| `BITAND(a, b)` | `a & b` |
| `BITOR(a, b)` | `a \| b` |
| `BITXOR(a, b)` | `a ^ b` |
| `BITNOT(a)` | `~a` |
| `SHL(a, n)` | `a << n` |
| `SHR(a, n)` | `a >> n` |

```basic
DIM flags AS INTEGER
flags = 0
flags = BITOR(flags, SHL(1, 0))     ' Bit 0 setzen
flags = BITOR(flags, SHL(1, 3))     ' Bit 3 setzen
PRINT "Flags = 0x" + HEX$(flags)    ' "0x9"
PRINT "Bit 3? ", BITAND(flags, SHL(1, 3)) <> 0  ' TRUE
```

## Arrays

Siehe auch [Sprachreferenz → Arrays](sprache.md#arrays).

| Funktion | Zweck |
|---|---|
| `LEN(arr)` → INTEGER | Anzahl Elemente (1. Dimension) |
| `DIMCOUNT(arr)` → INTEGER | Anzahl Dimensionen |
| `DIMSIZE(arr, n)` → INTEGER | Größe der n-ten Dimension (0-basiert) |

```basic
DIM matrix[3, 4] AS INTEGER

PRINT DIMCOUNT(matrix)           ' 2
PRINT DIMSIZE(matrix, 0)         ' 3
PRINT DIMSIZE(matrix, 1)         ' 4
PRINT LEN(matrix)                ' 3 (= DIMSIZE 0)
```

## Maps

`MAP OF T` mit STRING-Schlüsseln und Werten vom Typ T.

| Funktion | Zweck |
|---|---|
| `MAPPUT(m, key$, value)` | setzen / überschreiben |
| `MAPGET(m, key$)` → T | lesen, **wirft** wenn Schlüssel fehlt |
| `MAPGETOR(m, key$, default)` → T | lesen mit Default |
| `MAPHAS(m, key$)` → BOOLEAN | Existenz |
| `MAPREMOVE(m, key$)` → BOOLEAN | TRUE wenn entfernt |
| `MAPSIZE(m)` → INTEGER | Anzahl |
| `MAPKEYS(m)` → ARRAY OF STRING | alle Schlüssel |
| `MAPCLEAR(m)` | leeren |

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

Memo-Pattern (Fibonacci-Cache):

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

## Datei-I/O

| Funktion | Zweck |
|---|---|
| `OPENFILE(pfad$, modus$)` → FILE | Modi: `"r"` lesen, `"w"` neu schreiben, `"a"` anhängen |
| `CLOSEFILE(f)` | schließt |
| `READLINE(f)` → STRING | nächste Zeile (ohne `\n`) |
| `READALL$(f)` → STRING | Rest komplett lesen |
| `ENDOFFILE(f)` → BOOLEAN | beim Ende? |
| `WRITELINE(f, text$)` | schreibt + `\n` |
| `WRITE(f, text$)` | schreibt ohne `\n` |
| `FILEEXISTS(p$)` → BOOLEAN | Datei vorhanden? |

```basic
' Schreiben
DIM out AS FILE
out = OPENFILE("scores.txt", "w")
WRITELINE(out, "Anna 95")
WRITELINE(out, "Bert 78")
CLOSEFILE(out)

' Lesen
IF FILEEXISTS("scores.txt") THEN
    DIM inp AS FILE
    inp = OPENFILE("scores.txt", "r")
    WHILE NOT ENDOFFILE(inp)
        PRINT READLINE(inp)
    WEND
    CLOSEFILE(inp)
END IF
```

## Zeit & Random

| Funktion | Zweck |
|---|---|
| `MILLIS()` → INTEGER | ms seit Programmstart |
| `TIME$()` → STRING | aktuelle Uhrzeit `"HH:MM:SS"` |
| `DATE$()` → STRING | aktuelles Datum `"YYYY-MM-DD"` |
| `RND()` → FLOAT | Zufallszahl in `[0, 1)` |
| `RND(n)` → INTEGER | Zufalls-INT in `[0, n)` |
| `RANDOMIZE([seed])` | Zufalls-Seed setzen (ohne Arg: System-Seed) |

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

' Reproduzierbare Würfel
RANDOMIZE(42)
FOR i = 1 TO 5
    PRINT RND(6) + 1
NEXT
```

## Spiel-Helfer

| Funktion | Zweck |
|---|---|
| `COLLIDES(x1, y1, w1, h1, x2, y2, w2, h2)` → BOOLEAN | AABB-Kollision zweier Rechtecke. Berührungen zählen nicht (Kanten gleich → FALSE). |

```basic
DIM held_x AS FLOAT
DIM held_y AS FLOAT
held_x = 100.0
held_y = 50.0

IF COLLIDES(held_x, held_y, 16, 16, 110, 60, 16, 16) THEN
    PRINT "Treffer!"
END IF
```

Für komplexere Sprite-Kollision siehe [Sprite-Modul](module-sprite.md) mit `SPRITE_COLLIDES`.
