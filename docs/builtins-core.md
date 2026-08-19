# Standard-Built-ins

Alle eingebauten Befehle, die ohne `IMPORT` verfügbar sind. Grafik-Befehle (SCREEN, BOX, …) sind separat in [builtins-grafik.md](builtins-grafik.md) dokumentiert.

## Inhalt

- [Ausgabe (PRINT)](#ausgabe-print)
- [Konvertierung](#konvertierung)
- [Math](#math)
- [Strings](#strings)
- [Bitwise](#bitwise)
- [Arrays](#arrays)
- [Maps](#maps)
- [Datei-I/O](#datei-io)
- [Bytes (BUFFER)](#bytes-buffer)
- [Betriebssystem](#betriebssystem)
- [Prüfen und Melden](#prüfen-und-melden)
- [Zeit & Random](#zeit--random)
- [Typen & Encoding](#typen--encoding)
- [Prüfsummen und Identität](#prüfsummen-und-identität)
- [Spiel-Helfer](#spiel-helfer)

## Ausgabe (PRINT)

`PRINT` gibt eine oder mehrere durch `,` oder `;` getrennte Werte aus:

- **`,`** trennt mit einem **Leerzeichen**: `PRINT "x", 5` → `x 5`
- **`;`** trennt **ohne** Zwischenraum: `PRINT "x"; 5` → `x5`
- ein **abschließendes** `,` oder `;` **unterdrückt den Zeilenumbruch** (die nächste
  Ausgabe schließt direkt an): `PRINT "Laden...";` dann `PRINT "fertig"` → `Laden...fertig`
- `PRINT` ohne Argumente gibt eine Leerzeile aus.

```basic
PRINT "a", "b", "c"     ' a b c
PRINT "a"; "b"; "c"     ' abc
PRINT "Score: "; punkte ' Score: 42   (kein Leerzeichen nach dem Doppelpunkt-String)
PRINT "x = ";           ' kein Newline
PRINT x                 ' schließt an
```

## Konvertierung

| Funktion | Zweck |
|---|---|
| `STR$(v)` → STRING | Wert nach String. Bools werden zu `"TRUE"`/`"FALSE"`, Floats wie `3.0` zu `"3.0"`. |
| `VAL(s$)` → INTEGER/FLOAT | String zu Zahl. Mit `.` → FLOAT, sonst INT. Ungültiges → `0`. |
| `INT(v)` → INTEGER | Zahl zu INT (`floor`). `INT(3.7)` = 3, `INT(-1.5)` = -2. |
| `ABS(v)` | Absolutbetrag. |
| `CHR$(n)` → STRING | Unicode-Codepoint zu 1-Zeichen-String. `CHR$(65)` = `"A"`. |
| `ASC(s$)` → INTEGER | Codepoint des ersten Zeichens. `ASC("Anna")` = 65. |
| `RGB(r, g, b)` → INTEGER | 3 INTs (0..255) zu 24-Bit-Farbe. `RGB(255, 128, 0)` = `&HFF8000`. |
| `HEX$(n)` → STRING | INT zu Hex-Großbuchstaben (ohne Präfix). `HEX$(255)` = `"FF"`. |

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

> **Hex- und Binär-Literale** gehen in zwei Schreibweisen: klassisch BASIC mit
> `&H`/`&B` **oder** im C-Stil mit `0x`/`0b`. `&HFF8000` = `0xFF8000` = 16744448,
> `&B1010` = `0b1010` = 10. So lassen sich Farben direkt als Literal angeben:
> `BOX(0, 0, 9, 9, 0xFF8000)`.

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
| `LOG10(x)` | Zehnerlogarithmus |
| `CLAMP01(v)` | auf `[0, 1]` beschränken |
| `WRAP(v, lo, hi)` | v zyklisch in `[lo, hi)` falten (Winkel/Index-Umlauf) |
| `PINGPONG(t, len)` | in `[0, len]` hin- und herpendeln (Dreieckswelle) |
| `MOVETOWARD(cur, ziel, maxd)` | cur um max. `maxd` Richtung `ziel` bewegen |
| `SMOOTHSTEP(e0, e1, x)` | weicher 0→1-Übergang (Hermite), geklemmt |
| `APPROX(a, b[, eps])` → BOOLEAN | `\|a-b\| ≤ eps` (Default `1e-6`) |

**Perlin-Noise** (deterministisch, Wert in ~`[-1, 1]`, gleiche Eingabe → gleicher
Wert): `NOISE(x)`, `NOISE2(x, y)`, `NOISE3(x, y, z)` und fraktal `FBM(x, y, oktaven)`,
`FBM3(x, y, z, oktaven)`. Für prozedurale Generierung (Terrain, Höhlen, organische
Bewegung). An ganzzahligen Gitterpunkten ist Perlin definitionsgemäß 0.

```basic
PRINT WRAP(370, 0, 360)      ' 10.0
PRINT PINGPONG(2.5, 2)       ' 1.5
PRINT MOVETOWARD(0, 10, 3)   ' 3.0
PRINT ROUND(NOISE2(1.5, 2.5), 3)  ' reproduzierbarer Rauschwert
```

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
| `RED(c)`, `GREEN(c)`, `BLUE(c)` → INTEGER | Kanal 0..255 aus `&HRRGGBB` |
| `HSV(h, s, v)` → INTEGER | HSV (h in Grad, s/v in `[0,1]`) → `&HRRGGBB` |
| `COLOR_LERP(c1, c2, t)` → INTEGER | zwei Farben kanalweise mischen (t 0..1) |

```basic
PRINT RED(&HFF8000)          ' 255
PRINT HSV(120.0, 1.0, 1.0)   ' 65280 (= &H00FF00, Grün)
PRINT COLOR_LERP(0, &HFFFFFF, 0.5)  ' 8421504 (= &H808080)
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
| `FORMAT$(value, mask)` | printf-Stil. `FORMAT$(42, "%05d")` → `"00042"`. Sechs Spezifizierer: `%d`/`%i`, `%f`, `%x`, `%X`, `%s` — dazu `%%` für ein wörtliches Prozentzeichen. Breite, Nullen und Genauigkeit wie gewohnt (`%05d`, `%-6d`, `%.2f`). Alles andere meldet „unbekannter Spezifizierer“ |

Erweiterungen *(nur native Runtime)*:

| Funktion | Zweck |
|---|---|
| `LTRIM$(s)`, `RTRIM$(s)` | Whitespace nur links / nur rechts entfernen |
| `REVERSE$(s)` | Zeichen umkehren |
| `STARTSWITH(s, präfix)`, `ENDSWITH(s, suffix)` → BOOLEAN | Anfang/Ende prüfen |
| `CONTAINS(s, teil)` → BOOLEAN | Teilstring enthalten? (Funktionsform von `teil IN s`) |
| `COUNT(s, teil)` → INTEGER | Anzahl nicht-überlappender Vorkommen von `teil` |
| `TITLE$(s)` | Anfangsbuchstabe jedes Wortes groß, Rest klein |
| `BIN$(n)`, `OCT$(n)` | INTEGER als Binär-/Oktalstring (mit Vorzeichen) |
| `ISNUMERIC(s)` → BOOLEAN | als Zahl parsebar? |
| `TRYVAL(s, default)` → INTEGER/FLOAT | robustes `VAL`: bei Parse-Fehler `default` statt still `0` |

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
PRINT TRYVAL("oops", -1)              ' -1  (statt 0 bei VAL)
```

## Bitwise

Bitweise Rechnung läuft über **Operatoren**, nicht über Funktionen. Alle nehmen
INTEGER und geben INTEGER; negative Werte wie in Python (Zweierkomplement,
beliebig groß).

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
flags = flags BOR (1 SHL 0)         ' Bit 0 setzen
flags = flags BOR (1 SHL 3)         ' Bit 3 setzen
PRINT "Flags = 0x" + HEX$(flags)    ' "0x9"
PRINT "Bit 3? ", (flags BAND (1 SHL 3)) <> 0  ' TRUE
```

Die Klammern um `1 SHL 3` sind nötig: alle binären Bit-Operatoren liegen auf
**einer** Präzedenzebene und werden von links abgearbeitet, `a BOR 1 SHL 3`
wäre also `(a BOR 1) SHL 3`.

> Früher gab es dafür die Funktionen `BITAND`/`BITOR`/`BITXOR`/`BITNOT`/`SHL`/
> `SHR`. Sie sind **entfernt** — mit den Operatoren wären sie doppelt.

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

> Die `ARRAY_*`-Aggregate und die dynamischen Array-Ops
> (`ARRAY_PUSH`/`POP`/`INSERT`/`REMOVE_AT`/`REDIM`) sind in `dhrt` implementiert
> (`builtins.rs`/`vm.rs`) und laufen über alle Wege — `dhrun.py`/Editor-Run,
> `dhrt run`/`--runsrc` und den `dhrt --export`-Standalone-Build.

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

## ZIP

Sicherungen, Belegsammlungen, Export — der übliche Weg, mehrere Dateien als
eine weiterzugeben.

| Funktion | Zweck |
|---|---|
| `ZIP_LIST(archiv$)` → ARRAY OF STRING | Namen aller Einträge |
| `ZIP_READ$(archiv$, name$)` → STRING | einen Eintrag als Text |
| `ZIP_READ(archiv$, name$)` → BUFFER | einen Eintrag als Bytes |
| `ZIP_EXTRACT(archiv$, ordner$)` → INTEGER | alles entpacken, liefert die Zahl der Dateien |
| `ZIP_CREATE(archiv$, dateien)` → INTEGER | Archiv aus einer Liste von Dateipfaden |
| `ZIP_WRITE(archiv$, namen, inhalte)` → INTEGER | Archiv aus Namen und Texten, ohne Umweg über Dateien |

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

**Beim Entpacken wird geprüft, wohin geschrieben wird.** Ein Archiv darf
Einträge wie `../../autoexec.bat` oder `C:/Windows/x.dll` enthalten; wer den
Namen aus dem Archiv einfach an den Zielordner hängt, schreibt damit
außerhalb davon — der Angreifer wählt die Datei, du entpackst sie. Solche
Einträge werden **übersprungen**, nicht geschrieben. `ZIP_EXTRACT` liefert die
Zahl der wirklich entstandenen Dateien zurück, damit ein Unterschied zur
Länge von `ZIP_LIST` auffällt.

**`ZIP_CREATE` speichert nur den Dateinamen**, nicht den Pfad, unter dem die
Datei lag — sonst trägt ein Archiv die Verzeichnisstruktur des Rechners nach
außen, auf dem es entstanden ist. Wer eine Ordnerstruktur *im* Archiv haben
will, nimmt `ZIP_WRITE` und gibt die Namen selbst an (`unter/notiz.txt`).

Komprimiert wird mit Deflate. `ZIP_WRITE` und `ZIP_CREATE` legen das Archiv
jeweils **neu** an; ein Anhängen an ein bestehendes Archiv gibt es nicht.

## CSV

Der häufigste Datenaustausch überhaupt — und mit `SPLIT$` nicht richtig zu
machen. Sobald ein Feld das Trennzeichen enthält, liefert `SPLIT$` zu viele
Felder und sagt nichts davon:

```text
Mueller;"Berlin; Mitte";42      SPLIT$(";") -> vier Felder statt drei
```

| Funktion | Zweck |
|---|---|
| `CSV_PARSE(text$[, trenner$])` → ARRAY OF STRING | Text zerlegen, 2D (Zeilen × Spalten) |
| `CSV_LOAD(pfad$[, trenner$])` → ARRAY OF STRING | Datei lesen, 2D |
| `CSV_FORMAT$(tabelle[, trenner$])` → STRING | 2D-Array als CSV-Text |
| `CSV_SAVE(pfad$, tabelle[, trenner$])` | 2D-Array in eine Datei |
| `CSV_ROW$(felder[, trenner$])` → STRING | eine einzelne Zeile aus 1D-Array |

```basic
DIM t AS ARRAY OF STRING
t = CSV_LOAD("kunden.csv", ";")

PRINT DIMSIZE(t, 0)     ' Zeilen
PRINT DIMSIZE(t, 1)     ' Spalten
PRINT t[1, 2]           ' zweite Zeile, dritte Spalte
```

**Trennzeichen:** ohne zweites Argument ein Komma. Excel schreibt im deutschen
Gebietsschema `;` — dann `CSV_LOAD(pfad$, ";")`. Es muss genau **ein** Zeichen
sein; alles andere meldet einen Fehler, statt stillschweigend das erste zu
nehmen.

**Anführungszeichen** nach RFC 4180: ein Feld darf Trennzeichen, Zeilenumbrüche
und Anführungszeichen enthalten, wenn es in `"` steht; ein `"` im Feld wird
verdoppelt. Beim Schreiben setzt Drachenhauch Anführungszeichen **nur, wo sie
nötig sind** — unnötige machen die Datei unleserlich und den Diff größer.

**Ungleich lange Zeilen** werden auf die *breiteste* aufgefüllt (mit
Leerstrings), weil ein GB-Array rechteckig sein muss. Abschneiden würde Daten
wegwerfen, ohne es zu sagen.

**Kaputte Dateien** brechen den Import nicht ab: eine fehlende
Schluss-Anführung liest bis zum Dateiende. `\r\n` und `\n` gelten
gleichermaßen als Zeilenende, ein BOM am Dateianfang wird abgeschnitten (sonst
hieße die erste Spalte für immer `﻿Name`).

Beispiel: [examples/169_csv.dh](../examples/169_csv.dh).

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
| `COPYFILE(src$, dst$)` | Datei kopieren |
| `APPENDFILE(pfad$, text$)` | Text ans Ende hängen (legt die Datei an) |
| `PATHJOIN(a$, b$, …)` → STRING | Pfadteile mit `/` verbinden |
| `BASENAME(pfad$)` → STRING | letzter Pfad-Bestandteil (Datei-/Ordnername) |
| `DIRNAME(pfad$)` → STRING | Verzeichnis-Anteil (ohne letzten Bestandteil) |

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

## Bytes (BUFFER)

`STRING` ist **UTF-8-Text**. Er kann gar nicht jede Bytefolge tragen, und `LEN`
zählt darin Zeichen, nicht Bytes. Sobald es um *Daten* statt um Text geht —
eigene Dateiformate, Bilder, Protokolle, Prüfsummen — braucht es einen zweiten
Typ. Das ist `BUFFER`: eine veränderliche Folge von Bytes.

```basic
DIM b AS BUFFER
b = BUFFER_NEW(4)          ' 4 Bytes, alle 0
BUFFER_SET(b, 0, 222)
PRINT BUFFER_TO_HEX$(b)    ' de000000
```

`BUFFER` braucht **kein `IMPORT`** und ist ein **Referenz-Typ** wie `ARRAY`:
gibt man ihn an eine `SUB`, teilen sich beide Seiten dieselben Bytes.

### Grundlagen

| Funktion | Zweck |
|---|---|
| `BUFFER_NEW(groesse)` → BUFFER | neuer Puffer, mit Nullen gefüllt |
| `BUFFER_LEN(b)` → INTEGER | Länge in **Bytes** |
| `BUFFER_GET(b, pos)` → INTEGER | Byte 0..255 lesen |
| `BUFFER_SET(b, pos, byte)` | Byte 0..255 schreiben (verändert in place) |
| `BUFFER_FILL(b, byte)` | alles mit einem Byte füllen |
| `BUFFER_RESIZE(b, groesse)` | wächst mit Nullen, schrumpft durch Abschneiden |
| `BUFFER_SLICE(b, von, bis)` → BUFFER | **Kopie** der Bytes `[von, bis)` |
| `BUFFER_CONCAT(a, b)` → BUFFER | neuer Puffer aus beiden |
| `BUFFER_INDEXOF(b, nadel [, ab])` → INTEGER | erste Fundstelle, sonst `-1` |

Wie bei Arrays gilt: **ein Index daneben ist ein Fehler, ein Slice klemmt.**
`BUFFER_GET(b, 99)` auf einen 4-Byte-Puffer wirft; `BUFFER_SLICE(b, 0, 99)`
liefert einfach die vorhandenen 4 Bytes.

Ein Byte außerhalb 0..255 ist ebenfalls ein Fehler und wird **nicht** still
beschnitten — das fiele sonst erst in der fertigen Ausgabedatei auf.

### Text, Hex und Base64

| Funktion | Zweck |
|---|---|
| `BUFFER_FROM_STRING(text$)` → BUFFER | Text als UTF-8-Bytes |
| `BUFFER_TO_STRING$(b)` → STRING | Bytes als UTF-8-Text |
| `BUFFER_TO_HEX$(b)` / `BUFFER_FROM_HEX(s$)` | Hex-Text (`"deadbeef"`) |
| `BUFFER_TO_BASE64$(b)` / `BUFFER_FROM_BASE64(s$)` | Base64 |

`BUFFER_TO_STRING$` ist **streng**: sind die Bytes kein gültiges UTF-8, gibt es
einen Fehler statt stillschweigend ersetzter Zeichen — ein `?` an der falschen
Stelle fälscht die Daten und fällt erst viel später auf. Für Daten, die gar
kein Text sein sollen, ist `BUFFER_TO_HEX$` das richtige Werkzeug.

`BUFFER_FROM_HEX` erlaubt Leerzeichen (`"de ad be ef"`), weil Hex-Dumps
üblicherweise gruppiert geschrieben werden.

`BUFFER_FROM_BASE64` liefert **rohe Bytes** — im Unterschied zu `BASE64_DECODE`,
das gültiges UTF-8 verlangt und sonst wirft.

### Zahlen packen

| Funktion | Zweck |
|---|---|
| `BUFFER_GET_I16/U16/I32/U32/I64(b, pos [, reihenfolge$])` → INTEGER | Ganzzahl lesen |
| `BUFFER_GET_F32/F64(b, pos [, reihenfolge$])` → FLOAT | Gleitkomma lesen |
| `BUFFER_SET_I16/U16/I32/U32/I64(b, pos, wert [, reihenfolge$])` | Ganzzahl schreiben |
| `BUFFER_SET_F32/F64(b, pos, wert [, reihenfolge$])` | Gleitkomma schreiben |

`reihenfolge$` ist `"le"` (little-endian, **Vorgabe**) oder `"be"`
(big-endian):

```basic
DIM b AS BUFFER
b = BUFFER_NEW(8)
BUFFER_SET_I32(b, 0, 1000)          ' e8030000
BUFFER_SET_I32(b, 4, 1000, "be")    ' 000003e8
PRINT BUFFER_TO_HEX$(b)             ' e8030000000003e8
```

Wer einen Puffer selbst schreibt und wieder liest, kann die Vorgabe ignorieren
— beide Seiten benutzen dieselbe. Die Angabe braucht nur, wer ein **fremdes**
Format bedient: PNG, ZIP und die meisten Netz-Protokolle sind big-endian.

Ein Wert, der nicht in die Breite passt, ist ein Fehler
(`BUFFER_SET_U16(b, 0, 70000)`). Still abgeschnitten käme eine völlig andere
Zahl wieder heraus.

### Binärdateien

| Funktion | Zweck |
|---|---|
| `READALL_BYTES(pfad$)` → BUFFER | ganze Datei als Bytes |
| `WRITEALL_BYTES(pfad$, b)` | Bytes in eine Datei (überschreibt) |
| `READ_BYTES(datei, anzahl)` → BUFFER | bis zu `anzahl` Bytes vom Handle |
| `WRITE_BYTES(datei, b)` | Bytes ans Handle |
| `SEEK(datei, position)` | Position setzen (0 = Anfang) |
| `TELL(datei)` → INTEGER | aktuelle Position |

```basic
' Stückweise durch eine große Datei, ohne sie ganz in den Speicher zu holen
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

**`READ_BYTES` liefert am Dateiende weniger als angefordert** — bis hin zu
gar nichts. Das ist kein Fehler, sondern die übliche Abbruchbedingung.

> **Es gibt keine eigenen Binär-Modi `"rb"`/`"wb"`.** Drachenhauch-Dateien sind
> immer byte-genau: es gibt keine CRLF-Übersetzung und kein Ctrl-Z-als-Dateiende
> wie in alten BASICs. Getrennte Modi würden einen Unterschied vorgaukeln, den
> es nicht gibt — `READ_BYTES`/`WRITE_BYTES`/`SEEK` arbeiten auf denselben
> Handles aus `OPENFILE(pfad, "r"/"w"/"a")` wie `READLINE`/`WRITELINE`.
>
> Was es (noch) nicht gibt: einen Modus, der **gleichzeitig** liest und
> schreibt. Wer eine Datei an einer Stelle ändern will, liest sie mit
> `READALL_BYTES`, ändert den Puffer und schreibt ihn mit `WRITEALL_BYTES`
> zurück.

## Betriebssystem

Damit wird aus einem Programm ein **Werkzeug**: es nimmt Argumente entgegen,
liest seine Umgebung, ruft andere Programme auf und sagt seinem Aufrufer, ob es
geklappt hat.

| Funktion | Zweck |
|---|---|
| `ARGC()` → INTEGER | Anzahl der Argumente für dieses Programm |
| `ARG$(n)` → STRING | Argument Nr. `n` (0-basiert); außerhalb → `""` |
| `GETENV$(name$ [, vorgabe$])` → STRING | Umgebungsvariable lesen |
| `SETENV(name$, wert$)` | Umgebungsvariable setzen (dieses Programm + seine Kinder) |
| `CWD$()` → STRING | aktuelles Arbeitsverzeichnis |
| `CHDIR(pfad$)` | Arbeitsverzeichnis wechseln |
| `EXIT([code])` | sofort beenden, `code` = Rückgabewert (0..255, Vorgabe 0) |
| `EPRINT(text)` | Zeile nach **stderr** statt stdout |
| `SHELL(programm$, ...)` → INTEGER | Programm starten, warten, Rückgabewert |
| `SHELL_OUT$(programm$, ...)` → STRING | wie `SHELL`, sammelt aber die Ausgabe ein |

```basic
' Ein Werkzeug, das eine Datei erwartet
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

**Woher die Argumente kommen** — der Unterschied ist wichtig:

| Aufruf | Was das Programm sieht |
|---|---|
| `dhrt run werkzeug.dh -- a b` | `a`, `b` |
| `dhrt run werkzeug.dh a b` | **nichts** |
| `werkzeug.exe a b` (exportiert) | `a`, `b` |

Beim Start über `dhrt` gehört alles hinter einem alleinstehenden `--` dem
Programm, alles davor der Runtime. Ohne `--` bekommt das Programm keine
Argumente. Das ist Absicht: sonst könnte `dhrt` sich keinen eigenen Schalter
mehr zulegen, ohne bestehende Programme zu brechen. Die **exportierte `.exe`**
ist selbst das Programm — dort gibt es nichts zu trennen, alle Argumente
gehören ihr, ohne `--`.

**`EXIT` ist kein Fehler.** Es beendet das Programm sofort und wird von
`TRY`/`CATCH` **nicht** gefangen — ein `EXIT` mitten in einem `TRY`-Block läuft
also wirklich hinaus und landet nicht im `CATCH`. Werte außerhalb 0..255 sind
ein Fehler statt still gekappt zu werden (das Betriebssystem überträgt nur das
untere Byte, aus `EXIT(256)` würde sonst klammheimlich „alles gut").

**`EPRINT` ist ein Builtin, kein Statement** — also mit Klammern
(`EPRINT("text")`, nicht `EPRINT "text"` wie bei `PRINT`).

**`SHELL` nimmt die Argumente einzeln**, nicht als eine Kommandozeile:

```basic
SHELL("git", "commit", "-m", "Nachricht mit Leerzeichen")   ' richtig
```

So gibt es keine Quoting-Regeln zu lernen, und ein Dateiname mit Leerzeichen
zerfällt nicht in zwei Argumente. Wer wirklich eine Shell braucht (Pipes,
Umleitungen), ruft sie ausdrücklich auf — `SHELL("cmd", "/c", "dir | more")` —
und unterliegt dann deren eigenen Quoting-Regeln.

`SHELL` reicht die Ausgabe des Kindprogramms direkt zur Konsole durch;
`SHELL_OUT$` sammelt dessen **stdout** ein und liefert es als STRING, während
sein **stderr** stderr bleibt — sonst mischten sich Fehlermeldungen unbemerkt
in die Nutzdaten.

### Ein Programm im Hintergrund

`SHELL` und `SHELL_OUT$` warten, bis das Kindprogramm fertig ist. Dauert das
länger als ein Bild, steht alles still. Dafür gibt es dieselbe Sache zum
Nachsehen:

| Funktion | Wirkung |
|---|---|
| `SHELL_START(programm$, ...)` → INTEGER | startet im Hintergrund, liefert die Auftragsnummer |
| `SHELL_READY(auftrag)` → BOOLEAN | ist der Prozess fertig? |
| `SHELL_RESULT$(auftrag)` → STRING | stdout abholen, Platz freigeben |
| `SHELL_CODE()` → INTEGER | Rückgabewert des **zuletzt abgeholten** Auftrags |
| `SHELL_ERR$()` → STRING | dessen stderr |
| `SHELL_CANCEL(auftrag)`, `SHELL_PENDING()` | verwerfen / zählen |

```basic
DIM auftrag AS INTEGER
DIM ausgabe AS STRING
auftrag = SHELL_START("git", "log", "--oneline")

WHILE NOT SHELL_READY(auftrag)
    ' ... hier weiterarbeiten, zeichnen, auf Tasten hören ...
    SLEEP(1)
WEND
ausgabe = SHELL_RESULT$(auftrag)
PRINT SHELL_CODE()
```

`SHELL_CODE()` und `SHELL_ERR$()` nehmen **kein** Argument: sie gehören zum
zuletzt abgeholten Auftrag — dasselbe Muster wie `HTTP_STATUS()` zur zuletzt
geholten Antwort. Ein Programm, das gar nicht erst startet, meldet sich beim
**Abholen**, nicht beim Starten.

> **`CWD$()` ist nicht das Verzeichnis, aus dem du gestartet hast.** `dhrt`
> wechselt beim Start ins Verzeichnis der `.dh`-Datei (damit
> `LOADIMAGE("assets/…")` von überall funktioniert), die exportierte `.exe`
> ins Exe-Verzeichnis. Ein Pfad, den der Benutzer als Argument übergibt, ist
> also relativ zu *seinem* Verzeichnis, nicht zu `CWD$()` — im Zweifel den
> Benutzer nach einem absoluten Pfad fragen.

## Prüfen und Melden

Damit prüft sich ein Programm selbst — und sagt hinterher, ob es geklappt hat.

| Funktion | Zweck |
|---|---|
| `ASSERT(bedingung [, meldung$])` | schlägt fehl, wenn die Bedingung `FALSE` ist |
| `ASSERT_EQ(ist, soll [, was$])` | dasselbe, die Meldung zeigt **beide** Werte |
| `ASSERT_COLLECT(an)` | Sammel-Modus ein/aus (Vorgabe: aus) |
| `ASSERT_COUNT()` → INTEGER | wie viele Prüfungen gelaufen sind |
| `ASSERT_FAILED()` → INTEGER | wie viele davon fehlgeschlagen sind |
| `ASSERT_REPORT()` → INTEGER | Bilanz ausgeben, Zahl der Fehlschläge zurückgeben |
| `LOG_DEBUG/INFO/WARN/ERROR(text)` | Meldung mit Uhrzeit nach **stderr** |

### Zwei Arten zu prüfen

**Vorbedingung im laufenden Programm** — hier soll ein Fehlschlag *abbrechen*.
Das ist die Vorgabe:

```basic
ASSERT(spieler_zahl > 0, "ohne Spieler geht es nicht")
```

Schlägt sie fehl, endet das Programm mit einem Laufzeitfehler samt Datei und
Zeile — genau wie jeder andere Fehler auch.

**Prüfprogramm** — hier will man *alle* Fehler sehen, nicht nur den ersten.
Dafür einmal am Anfang den Sammel-Modus einschalten:

```basic
ASSERT_COLLECT(TRUE)

ASSERT_EQ(punkte(2, 1, 2, 1), 4, "exakt getroffen")
ASSERT_EQ(punkte(1, 0, 0, 1), 0, "falsche Tendenz")
ASSERT(tendenz(1, 0) > 0,       "Heimsieg")

IF ASSERT_REPORT() > 0 THEN
    EXIT(1)
END IF
```

Ausgabe bei einem Fehlschlag:

```
FEHL  Zeile 4: falsche Tendenz: erhalten 2, erwartet 0     <- stderr
FEHLER: 1 von 3 Pruefungen                                 <- stdout
```

Der **Rückgabewert** ist der Punkt: erst damit kann ein Skript, ein Makefile
oder eine CI zwischen „lief durch" und „hat Fehler gefunden" unterscheiden.

**Trennung von stdout und stderr:** die Fehlschläge gehen nach stderr, die
Bilanz nach stdout. Ein `pruefung > bericht.txt` liefert also einen sauberen
Bericht, während die Einzelheiten weiter im Terminal stehen.

> **`ASSERT` verlangt einen `BOOLEAN`.** `ASSERT(anzahl)` ist ein Fehler und
> nicht etwa „wahr, weil nicht null" — eine Prüfung, die aus Versehen immer
> durchgeht, ist schlimmer als gar keine. Also einen Vergleich schreiben:
> `ASSERT(anzahl > 0)`.

> **`ASSERT_EQ` vergleicht wie der `=`-Operator** der Sprache, inklusive
> `1 = 1.0`. Eine zweite Vorstellung davon, wann zwei Werte gleich sind, wäre
> die sicherste Art, Vertrauen in die Prüfungen zu verspielen.

### Melden

```basic
LOG_INFO("Saison 2026 geladen")
LOG_WARN("Kein Netz -- arbeite mit den gespeicherten Daten")
LOG_ERROR("Datenbank nicht lesbar")
```

Ausgabe: `20:45:43 INFO  Saison 2026 geladen` — nach **stderr**, damit `PRINT`
als Nutzdaten durchgereicht werden kann.

Wie viel davon erscheint, steuert die Umgebungsvariable **`DH_LOG`**:

| `DH_LOG` | was erscheint |
|---|---|
| `debug` | alles |
| *(nicht gesetzt)* / `info` | INFO, WARN, ERROR — **Vorgabe** |
| `warn` | WARN, ERROR |
| `error` | nur ERROR |
| `aus` | nichts |

`LOG_DEBUG` schweigt also, bis jemand es einschaltet — Debug-Meldungen können
im Code stehen bleiben, ohne den Normalbetrieb zuzumüllen:

```bash
DH_LOG=debug dhrt run werkzeug.dh
```

## Zeit & Random

| Funktion | Zweck |
|---|---|
| `MILLIS()` → INTEGER | ms seit Programmstart (Stoppuhr) |
| `TIMER()` → FLOAT | dieselbe Uhr in Sekunden |
| `TIME$()` → STRING | aktuelle Uhrzeit `"HH:MM:SS"` |
| `DATE$()` → STRING | aktuelles Datum `"YYYY-MM-DD"` |
| `RND()` → FLOAT | Zufallszahl in `[0, 1)` |
| `RND(n)` → INTEGER | Zufalls-INT in `[0, n)` |
| `RANDINT(lo, hi)` → INTEGER | Zufalls-INT in `[lo, hi]` (inklusiv) |
| `RANDF(lo, hi)` → FLOAT | Zufalls-FLOAT in `[lo, hi)` |
| `CHOICE(array)` → T | zufälliges Element eines 1D-Arrays |
| `WEIGHTED_CHOICE(werte, gewichte)` → T | Element aus `werte`, gewählt proportional zu `gewichte` (1D-Arrays gleicher Länge, Gewichte ≥ 0). Loot-Tabellen. |
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
' MILLIS ist eine Stoppuhr ab Programmstart, keine Uhrzeit: sie faengt bei 0
' an und laeuft gleichmaessig weiter, auch wenn die Systemzeit springt
' (Zeitumstellung, NTP). Fuer Datum und Uhrzeit ist ZEIT_JETZT() aus dem
' Modul "zeit" zustaendig, das damit auch rechnen kann.

' Reproduzierbare Würfel
RANDOMIZE(42)
FOR i = 1 TO 5
    PRINT RND(6) + 1
NEXT
```

## Typen & Encoding

| Funktion | Zweck |
|---|---|
| `TYPEOF(x)` → STRING | Laufzeit-Typname, z.B. `"INTEGER"`, `"STRING"`, `"VEC3"`, `"MAT4"`, `"OBJECT"` |
| `ISNUM(x)`, `ISINT(x)`, `ISSTR(x)`, `ISBOOL(x)` → BOOLEAN | Typ-Prädikate (Bool ist KEINE Zahl) |
| `BASE64_ENCODE(s$)` → STRING | UTF-8-Text Base64-kodieren |
| `BASE64_DECODE(s$)` → STRING | Base64 zu UTF-8-Text (wirft bei ungültiger Eingabe) |
| `CRC32(s$)` → INTEGER | CRC-32-Prüfsumme der UTF-8-Bytes |
| `HASH(s$)` → INTEGER | stabiler 64-Bit-Hash (FNV-1a) — Save-Integrität, Buckets |

```basic
PRINT TYPEOF(3.0)                       ' FLOAT
PRINT ISINT(5), ISINT(3.0)              ' TRUE FALSE
PRINT BASE64_ENCODE("Hi!")              ' "SGkh"
PRINT BASE64_DECODE("SGkh")             ' "Hi!"
PRINT CRC32("hello")                    ' 907060870
```

## Prüfsummen und Identität

`CRC32` und `HASH` oben sind zum **Wiedererkennen** da — sie sagen „vermutlich
dieselben Daten". Sobald jemand die Antwort *fälschen* könnte, reichen sie
nicht: Signaturen, Tokens, Belege, Passwort-Ableitungen brauchen etwas
anderes.

| Funktion | Zweck |
|---|---|
| `SHA256$(daten)` → STRING | SHA-256 als Hex (64 Zeichen) |
| `SHA1$(daten)` → STRING | SHA-1 — nur für Verträglichkeit mit Bestehendem |
| `MD5$(daten)` → STRING | MD5 — nur für Verträglichkeit |
| `SHA256_FILE$(pfad$)`, `SHA1_FILE$`, `MD5_FILE$` → STRING | dasselbe für eine Datei, blockweise gelesen |
| `HMAC_SHA256$(schluessel, daten)` → STRING | Signatur mit geheimem Schlüssel |
| `SECURE_EQUALS(a, b)` → BOOLEAN | Vergleich in konstanter Zeit |
| `UUID4$()` → STRING | zufällige eindeutige Kennung |
| `RANDOM_BYTES(anzahl)` → BUFFER | Zufallsbytes vom Betriebssystem |

`daten` und `schluessel` dürfen `STRING` (dann die UTF-8-Bytes) oder `BUFFER`
sein — eine Signatur bildet man über die Bytes, die wirklich übertragen werden,
und die liegen bei einem Datei-Upload als `BUFFER` vor.

```basic
PRINT SHA256$("abc")
' ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad

PRINT SHA256_FILE$("grosses_archiv.zip")   ' liest blockweise, egal wie groß
```

### Eine Signatur prüfen

Der Fall, für den das alles da ist — ein Dienst schickt Daten und eine
Signatur, das Programm rechnet nach:

```basic
DIM erwartet AS STRING
erwartet = HMAC_SHA256$(geheimnis, nutzlast)

IF SECURE_EQUALS(erwartet, signatur_aus_der_kopfzeile) THEN
    PRINT "echt"
ELSE
    PRINT "gefälscht oder verändert"
END IF
```

> **`SECURE_EQUALS` statt `=`.** Ein gewöhnlicher Vergleich bricht beim ersten
> ungleichen Zeichen ab. Wer eine Signatur erraten will, misst die Zeit und hat
> sie nach ein paar hundert Versuchen Zeichen für Zeichen. `SECURE_EQUALS`
> läuft immer vollständig durch. Das ist genau bei dem Vergleich wichtig, für
> den `HMAC_SHA256$` überhaupt existiert.

### Zufall, der kein Spiel ist

```basic
DIM token AS STRING
token = BUFFER_TO_HEX$(RANDOM_BYTES(32))
```

> **`RANDOM_BYTES` ist nicht `RND`.** `RND` hängt an `RANDOMIZE`: dieselbe Saat
> liefert dieselbe Folge. Für ein Würfelspiel ist das richtig und sogar
> erwünscht (reproduzierbare Level) — für ein Passwort, einen Sitzungs-Schlüssel
> oder ein Salz wäre es ein Fehler. `RANDOM_BYTES` kommt aus der Zufallsquelle
> des Betriebssystems und lässt sich von `RANDOMIZE` nicht beeindrucken.

> **`MD5$` und `SHA1$` gelten als gebrochen.** Sie stehen hier, weil man sie
> zum Mitspielen braucht — ETags, alte Prüfsummen-Listen, git-Objektnamen. Für
> eine *eigene* Sicherheitsentscheidung ist `SHA256$` die Antwort.

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
