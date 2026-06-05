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
| `ASIN(x)`, `ACOS(x)` | Arcus-Sinus/-Cosinus (x in `[-1, 1]`) |
| `SQR(x)` | Quadratwurzel (x ≥ 0) |
| `HYPOT(x, y)` | `SQR(x*x + y*y)` ohne Overflow |
| `POW(b, e)` | b hoch e |
| `EXP(x)` | e^x |
| `LOG(x[, base])` | Logarithmus, default natürlich |
| `DEG(rad)`, `RAD(grad)` | Radiant ↔ Grad |
| `FLOOR(x)`, `CEIL(x)`, `ROUND(x)` | INT-Konvertierung |
| `ROUND(x, dezimalstellen)` → FLOAT | auf N Nachkommastellen runden |
| `MIN(a, b, ...)`, `MAX(a, b, ...)` | variadic |
| `CLAMP(v, lo, hi)` | beschränkt v auf `[lo, hi]` |
| `LERP(a, b, t)` | lineare Interpolation a..b (t nicht geklemmt) |
| `REMAP(v, in_lo, in_hi, out_lo, out_hi)` | linear umskalieren |
| `FRAC(x)` | Nachkommaanteil (vorzeichenbehaftet): `x - TRUNC(x)` |
| `SIGN(x)` | -1, 0 oder 1 |

Konstanten: `PI`, `TAU` (= 2·PI). (`E` ist absichtlich keine Konstante — `e`
ist ein häufiger `CATCH e`-Variablenname; nutze `EXP(1)`.)

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

' Vektor-Länge und Winkel
DIM laenge AS FLOAT
DIM winkel_grad AS FLOAT
laenge = HYPOT(3.0, 4.0)              ' 5.0
winkel_grad = DEG(ATAN2(4.0, 3.0))
```

### Farb-Helfer

| Funktion | Zweck |
|---|---|
| `RGB(r, g, b)` → INTEGER | siehe Konvertierung |
| `RED(c)`, `GREEN(c)`, `BLUE(c)` → INTEGER | Kanal 0..255 aus `0xRRGGBB` |
| `HSV(h, s, v)` → INTEGER | HSV (h in Grad, s/v in `[0,1]`) → `0xRRGGBB` |
| `COLOR_LERP(c1, c2, t)` → INTEGER | zwei Farben kanalweise mischen (t 0..1) |

```basic
PRINT RED(0xFF8000)          ' 255
PRINT HSV(120.0, 1.0, 1.0)   ' 65280 (= 0x00FF00, Grün)
PRINT COLOR_LERP(0, 0xFFFFFF, 0.5)  ' 8421504 (= 0x808080)
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

Erweiterungen *(nur native Runtime)*:

| Funktion | Zweck |
|---|---|
| `LTRIM$(s)`, `RTRIM$(s)` | Whitespace nur links / nur rechts entfernen |
| `REVERSE$(s)` | Zeichen umkehren |
| `STARTSWITH(s, präfix)`, `ENDSWITH(s, suffix)` → BOOLEAN | Anfang/Ende prüfen |
| `CONTAINS(s, teil)` → BOOLEAN | Teilstring enthalten? (Funktionsform von `teil IN s`) |
| `BIN$(n)`, `OCT$(n)` | INTEGER als Binär-/Oktalstring (mit Vorzeichen) |
| `ISNUMERIC(s)` → BOOLEAN | als Zahl parsebar? |
| `TRYVAL(s, default)` → INTEGER/FLOAT | robustes `VAL`: bei Parse-Fehler `default` statt still `0` |

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

PRINT REVERSE$("abc")                 ' "cba"
PRINT STARTSWITH("hello", "he")       ' TRUE
PRINT BIN$(10)                        ' "1010"
PRINT TRYVAL("oops", -1)              ' -1  (statt 0 bei VAL)
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
| `SORT(arr)`, `REVERSE(arr)` | 1D IN PLACE sortieren / umkehren |
| `SORT(arr, absteigend)` | mit BOOLEAN-Flag absteigend sortieren *(nur native Runtime)* |
| `SORT(arr, comparator)` | mit FUNCREF-Comparator `cmp(a, b)` → INTEGER (<0/0/>0) sortieren, stabil *(nur native Runtime)* |
| `ARRAY_INDEXOF(arr, v)` → INTEGER | erster Index von v, sonst -1 |

**Aggregate** (1D `ARRAY OF INTEGER`/`FLOAT`) und Helfer:

| Funktion | Zweck |
|---|---|
| `ARRAY_SUM(arr)` → INTEGER/FLOAT | Summe (INTEGER-Array → INTEGER, sonst FLOAT) |
| `ARRAY_AVG(arr)` → FLOAT | Durchschnitt (Array darf nicht leer sein) |
| `ARRAY_MIN(arr)`, `ARRAY_MAX(arr)` | kleinstes / größtes Element |
| `ARRAY_FILL(arr, wert)` | alle Elemente mit `wert` füllen (IN PLACE, jede Dimension) |
| `ARRAY_COPY(arr)` → ARRAY | unabhängige Kopie (gleiche Form/Typ) |

**Dynamische 1D-Arrays** (wachsen/schrumpfen IN PLACE):

| Funktion | Zweck |
|---|---|
| `ARRAY_PUSH(arr, wert)` → INTEGER | Element ans Ende anhängen; liefert die neue Länge |
| `ARRAY_POP(arr)` → T | letztes Element entfernen und zurückgeben (nicht leer) |
| `ARRAY_INSERT(arr, idx, wert)` → INTEGER | an Index `idx` (`0..len`) einfügen; neue Länge |
| `ARRAY_REMOVE_AT(arr, idx)` → T | Element an `idx` entfernen und zurückgeben |
| `REDIM(arr, länge)` | auf `länge` bringen — wächst mit Typ-Default, schrumpft schneidet ab; vorhandene Werte bleiben |

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

' Dynamisch: als Stack/Liste verwenden
DIM stack[0] AS INTEGER
PRINT ARRAY_PUSH(stack, 1)       ' 1 (neue Länge)
PRINT ARRAY_PUSH(stack, 2)       ' 2
PRINT ARRAY_POP(stack)           ' 2
PRINT LEN(stack)                 ' 1
```

> **Nur native Runtime.** Die `ARRAY_*`-Aggregate und die dynamischen Array-Ops
> (`ARRAY_PUSH`/`POP`/`INSERT`/`REMOVE_AT`/`REDIM`) sind ausschließlich in `gbrt`
> implementiert, nicht im Python-Tree-Walker. Sie laufen über gbrts eigenes
> Rust-Frontend — und das nutzen jetzt **alle** Run-/Export-Wege: `gbrun.py
> --native`/`--export`, der **Editor-Run/-Export** und `gbrt run`/`--runsrc`/
> `gbrt --export` rufen `gbrt run`/`gbrt --export` (Rust-Compiler). Nur der
> reine Tree-Walker-Fallback (F5 ohne gebautes gbrt) kennt sie nicht.

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

Pfadbasiert, ohne FILE-Handle *(nur native Runtime)*:

| Funktion | Zweck |
|---|---|
| `WRITEALL(pfad$, text$)` | Text komplett schreiben (überschreibt/erzeugt) |
| `READLINES(pfad$)` → ARRAY OF STRING | Datei als Zeilen-Array lesen |
| `FILESIZE(pfad$)` → INTEGER | Größe in Bytes |
| `DELETEFILE(pfad$)`, `RENAME(alt$, neu$)` | löschen / umbenennen·verschieben |
| `DIREXISTS(pfad$)` → BOOLEAN | Verzeichnis vorhanden? |
| `DIRLIST(pfad$)` → ARRAY OF STRING | Eintragsnamen (sortiert) |
| `MKDIR(pfad$)` | Verzeichnis anlegen (inkl. Eltern) |
| `PATHJOIN(a$, b$, …)` → STRING | Pfadteile mit `/` verbinden |

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

' Pfadbasiert (native Runtime)
MKDIR(PATHJOIN("saves", "level1"))
WRITEALL(PATHJOIN("saves/level1", "progress.txt"), "score=42")
DIM zeilen AS ARRAY OF STRING
zeilen = READLINES(PATHJOIN("saves/level1", "progress.txt"))
PRINT FILESIZE(PATHJOIN("saves/level1", "progress.txt"))
```

## Zeit & Random

| Funktion | Zweck |
|---|---|
| `MILLIS()` → INTEGER | ms seit Programmstart |
| `TIME$()` → STRING | aktuelle Uhrzeit `"HH:MM:SS"` |
| `DATE$()` → STRING | aktuelles Datum `"YYYY-MM-DD"` |
| `RND()` → FLOAT | Zufallszahl in `[0, 1)` |
| `RND(n)` → INTEGER | Zufalls-INT in `[0, n)` |
| `RANDINT(lo, hi)` → INTEGER | Zufalls-INT in `[lo, hi]` (inklusiv) |
| `RANDF(lo, hi)` → FLOAT | Zufalls-FLOAT in `[lo, hi)` |
| `CHOICE(array)` → T | zufälliges Element eines 1D-Arrays |
| `SHUFFLE(array)` | mischt ein 1D-Array IN PLACE (Fisher-Yates) |
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

## Aliase & Namenskonventionen

**BASIC-Aliase** (gleiches Verhalten, klassische Schreibweise — nur native Runtime):

| Alias | Kanonisch |
|---|---|
| `SGN(x)` | `SIGN(x)` |
| `SQRT(x)` | `SQR(x)` |
| `AUDIO_SET_VOLUME(ch, v)` | `AUDIO_VOLUME(ch, v)` |
| `AUDIO_MUSIC_SET_VOLUME(v)` | `AUDIO_MUSIC_VOLUME(v)` |

Container-Methode `arr.join(trenner)` ≡ `JOIN$(arr, trenner)` (Array OF STRING).

**Konventionen / Stolpersteine** (bewusst NICHT umbenannt — nur zur Klarstellung):

- **`$`-Suffix nur im Core.** String-Builtins der Kernsprache gibt es mit und
  ohne `$` (`UPPER$` ≡ `UPPER`, `LEFT$` ≡ `LEFT`, …). **Modul-Builtins** führen
  kein `$` (z.B. `JSON_GET_STRING`, nicht `JSON_GET_STRING$`).
- **Zwei Sound-APIs.** `PLAYSOUND`/`STOPSOUND` (Core, einfach) und das
  `audio`-Modul (`AUDIO_PLAY`/`AUDIO_STOP`/Channels/Fades). `AUDIO_*`-Objekte
  sind mit `PLAYSOUND` kompatibel. Beide bleiben — `audio` ist die mächtigere.
- **Suffix `2` ist mehrdeutig** und kontextabhängig: `PHYSICS_DISTANCE2` =
  *quadrierte* Distanz (schneller), `LINE3D`/Vec2-Funktionen meinen *2D*, und
  `VEC2_*` ist der Typname. Kein einheitliches Schema — am Funktionsnamen ablesen.
- **`SPRITE_COLLIDE` vs `SPRITE_COLLIDES`.** Das `sprite`-Modul nutzt
  `SPRITE_COLLIDES` (mit `S`). Auf den Core-`COLLIDES` (AABB von Rohwerten)
  achten — anderer Anwendungsfall.
- **`CAMERA_X` (2D) vs `CAMERA3D_X`.** Das 2D-`camera`-Modul und die native
  3D-Kamera (`g3d`) haben getrennte Getter — nicht verwechseln.
