# Sprachreferenz

Drachenhauch ist BASIC mit Pascal-strikter Typisierung. Wer schon mal QBasic, GW-BASIC oder Visual Basic geschrieben hat, fühlt sich sofort zuhause.

## Inhalt

- [Variablen und Konstanten](#variablen-und-konstanten)
- [Zahlen-Literale](#zahlen-literale)
- [Datentypen](#datentypen)
- [ENUM](#enum)
- [Compound-Assignment](#compound-assignment)
- [String-Interpolation (f-Strings)](#string-interpolation-f-strings)
- [Operatoren](#operatoren)
- [Strings](#strings)
- [Kontrollfluss: IF / ELSE](#kontrollfluss-if--else)
- [SELECT CASE](#select-case)
- [Schleifen: FOR und WHILE](#schleifen-for-und-while)
- [BREAK und CONTINUE](#break-und-continue)
- [DATA / READ / RESTORE](#data--read--restore)
- [Zeilenfortsetzung](#zeilenfortsetzung)
- [Statement-Trenner](#statement-trenner)
- [Funktionen: SUB und FUNCTION](#funktionen-sub-und-function)
- [Named Arguments](#named-arguments)
- [Coroutines: YIELD](#coroutines-yield)
- [Arrays](#arrays)
- [Maps](#maps)
- [Klassen und Strukturen](#klassen-und-strukturen)
- [Try / Catch / Throw](#try--catch--throw)
- [Import](#import)
- [Kommentare](#kommentare)

## Variablen und Konstanten

Variablen müssen vor Gebrauch deklariert werden. Der Typ steht hinter `AS`.

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

Mehrere Variablen gleichen Typs in einer Zeile:

```basic
DIM x, y, z AS INTEGER
DIM vorname, nachname AS STRING

' Auch mit Arrays gemischt:
DIM grid[10, 10], score, lives[3] AS INTEGER
```

Alle Variablen einer Multi-DIM bekommen denselben Typ und ihren Typ-Default als Anfangswert.

> **Reservierte Wörter nicht als Variablennamen.** Manche kurze Namen sind
> Schlüsselwörter und können nicht als Bezeichner dienen — neben den
> offensichtlichen (`to`, `step`, `mod`, `end`, `next`, `new`, `class`, `in`)
> auch einige, die als Variablenname naheliegen: **`map`, `image`, `sound`,
> `input`, `file`, `data`, `read`, `band`**. `DIM band AS INTEGER` meldet dann
> „'BAND' ist ein reserviertes Wort …". Workaround: anders benennen (`img`,
> `snd`, `karte`, `daten`, …). Unbedenklich sind dagegen gängige Namen wie
> `value`, `key`, `count`, `index`, `name`, `type`, `result`, `size`, `pos`,
> `state`, `item`, `text`, `color`.

Konstanten mit `CONST`:

```basic
CONST PI_HALF AS FLOAT = 1.5707963
CONST MAX_LEBEN AS INTEGER = 3
CONST TITEL AS STRING = "Mein Spiel"

' Typ kann weggelassen werden - wird vom Wert abgeleitet:
CONST FPS = 60
```

Konstanten dürfen nur einmalig zugewiesen werden; späteres Schreiben ist ein Fehler.

## Zahlen-Literale

Klassische Schreibweisen:

```basic
DIM dec AS INTEGER
dec = 255

DIM hex AS INTEGER
hex = &HFF                ' Hex (0..F, case-insensitive: &hff)

DIM bin AS INTEGER
bin = &B11010110          ' Binaer

DIM f AS FLOAT
f = 3.14
```

Hex und Binary sind nur INTEGER-Konstanten — keine Floats. Alle drei Schreibweisen ergeben denselben INTEGER-Wert.

## Datentypen

| Typ | Wertebereich | Default |
|---|---|---|
| `INTEGER` | ganzzahlig (Python-`int`) | `0` |
| `FLOAT` | Gleitkomma | `0.0` |
| `STRING` | UTF-8-Text | `""` |
| `BOOLEAN` | `TRUE` oder `FALSE` | `FALSE` |
| `FILE` | Datei-Handle | `NIL` |
| `BUFFER` | veränderliche Bytefolge ([Bytes](builtins-core.md#bytes-buffer)) | `NIL` |
| `IMAGE` | Bild-Handle (nativ dhrt) | `NIL` |
| `SOUND` | Sound-Handle (nativ dhrt) | `NIL` |
| `ARRAY OF T` | mehrdim. Array | `NIL` (oder mit Größen-Init: gefüllt) |
| `MAP OF T` | String→T-Map | leere Map |
| `<Klassenname>` | Instanz | `NIL` |
| `<Externer Typ>` | aus Modul (z.B. `JSON_HANDLE`) | `NIL` |

**Strikte Typisierung:** Eine FLOAT-Variable nimmt keine STRINGs an. Ein FLOAT-zu-INTEGER-Cast verlangt eine ganzzahlige Zahl (`3.0` ja, `3.14` Fehler — nutze dann `INT()`).

**`NIL`:** Klassenreferenzen, Bilder, Sounds und externe Handles sind initial `NIL`. Der nil-Check läuft über das Builtin **`IS_NIL(x)`** (es gibt KEIN `IS NIL`/`IS NOT NIL`-Sprachkonstrukt):

```basic
DIM bild AS IMAGE
IF IS_NIL(bild) THEN
    bild = LOADIMAGE("hero.png")
END IF
```

## ENUM

Typsichere benannte Konstanten mit Namespace. Statt magischer Zahlen schreibst du `State.PLAYING` und der Compiler macht daraus eine `INTEGER`-Konstante. Member werden auto-nummeriert (0, 1, 2, …) oder explizit gesetzt.

**Compact-Form** (Einzeiler):

```basic
ENUM State = MENU, PLAYING, PAUSED, GAMEOVER

PRINT State.MENU        ' 0
PRINT State.PLAYING     ' 1
PRINT State.PAUSED      ' 2
```

**Block-Form** mit expliziten Werten:

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

**Mixed**: explizite Werte und Auto-Nummerierung mischen — der nächste implizite Wert zählt von dem letzten expliziten weiter:

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

**`DIM x AS State`** ist äquivalent zu `DIM x AS INTEGER` — der Parser löst Enum-Typen zu `INTEGER` auf:

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

**Member-Namen dürfen Keywords sein**: `READ`, `FILE`, `DATA`, `NONE` etc. werden bei qualifiziertem Zugriff (`Name.Member`) eindeutig — die Sprache lässt es zu.

**Werte müssen Compile-Time-Integer-Literale sein.** Mehrgliedrige Ausdrücke wie `A + B` sind nicht erlaubt; nutze stattdessen `CONST` und schreibe konkrete Zahlen.

## Compound-Assignment

Bequemere Schreibweise für die häufigsten "modifiziere mich selbst"-Operationen — der Parser de-sugart sie automatisch zu `target = target OP value`:

```basic
DIM x AS INTEGER
x = 10
x += 5         ' = x = x + 5  -> 15
x -= 3         ' -> 12
x *= 2         ' -> 24
x /= 4         ' -> 6
```

Funktioniert auch auf Array-Elementen und Klassen-Feldern:

```basic
xs[0] += 100
self.health -= damage
```

## String-Interpolation (f-Strings)

Statt mit `+` und `STR$` zu kleben:

```basic
DIM name AS STRING
DIM score AS INTEGER
name = "Anna"
score = 42

' Klassisch:
PRINT "Hallo, " + name + "! Score: " + STR$(score)

' Mit f-String (Python-Stil):
PRINT f"Hallo, {name}! Score: {score}"
```

Im `f"..."` werden Ausdrücke in `{...}` ausgewertet und automatisch via `STR$(...)` stringifiziert. Doppelte Klammern (`{{` und `}}`) sind Escapes für wörtliche `{` und `}`.

```basic
PRINT f"x = {x + 1}, doppelt = {x * 2}"     ' Ausdrücke erlaubt
PRINT f"{{nicht expr}} aber {x}"             ' "{nicht expr} aber 5"
```

Plain-Strings (`"..."`) werden **nicht** interpoliert — `{name}` bleibt buchstäblich. Nur f-Strings expandieren.

## Operatoren

**Arithmetik:** `+ - * / \ ^ MOD`
- `\` ist ganzzahlige Division (gibt INTEGER, INTEGER auch bei FLOAT-Eingabe).
- `^` ist Potenz: `2 ^ 8 = 256`.

**Vergleich:** `= <> < > <= >=`

**Logik:** `AND OR NOT`. Short-Circuit: `AND` und `OR` werten den rechten Operanden nur aus, wenn nötig.

**String-Verkettung:** `+`

```basic
DIM g AS STRING
g = "Hallo, " + name + "!"
```

## Strings

String-Literale stehen in doppelten Anführungszeichen. Doppelte Anführungszeichen darin werden verdoppelt:

```basic
DIM s AS STRING
s = "Sie sagte ""Hallo""."     ' -> Sie sagte "Hallo".
```

String-Funktionen siehe [Standard-Built-ins](builtins-core.md): `LEFT$`, `RIGHT$`, `MID$`, `INSTR`, `REPLACE$`, `TRIM$`, `SPLIT$`, `JOIN$`, `UPPER$`, `LOWER$`, `LEN`, `STR$`, `VAL`, `CHR$`, `ASC`, `PADL$`, `PADR$`, `REPEAT$`, `SPACE$`, `HEX$`.

Konvention: String-Funktionen mit `$`-Suffix existieren auch ohne Suffix (`UPPER$` und `UPPER` sind dasselbe).

> **`+` mit einem String wandelt die andere Seite automatisch um** (kein
> Typfehler): `"Punkte: " + 42` ergibt `"Punkte: 42"`, `"ok=" + TRUE` ergibt
> `"ok=TRUE"`. Bequem fürs Zusammenbauen von Ausgaben — aber wer strikte Typen
> erwartet, wird überrascht. (`STR$(v)` macht die Umwandlung explizit.)

## Kontrollfluss: IF / ELSE

**Block-Form:**

```basic
IF score > 100 THEN
    PRINT "Sehr gut!"
ELSEIF score > 50 THEN
    PRINT "Geht so."
ELSE
    PRINT "Naja..."
END IF
```

**Single-Line:**

```basic
IF treffer THEN PRINT "Bumm!"
IF treffer THEN PRINT "Bumm!" ELSE PRINT "Daneben."
```

## SELECT CASE

Mehrweg-Verzweigung — viel lesbarer als verschachtelte `IF/ELSEIF`-Ketten. Drei Match-Formen, beliebig kombinierbar:

```basic
SELECT CASE punkte
    CASE 0                     ' exakter Wert
        PRINT "Null"
    CASE 1, 2, 3               ' Liste
        PRINT "Klein"
    CASE 10 TO 20              ' Bereich (inklusiv)
        PRINT "Mittel"
    CASE IS > 100              ' Vergleich (=, <>, <, >, <=, >=)
        PRINT "Spitze"
    CASE 50, 60 TO 70, IS = 99 ' alle Formen mischbar
        PRINT "Spezialfall"
    CASE ELSE                  ' Fallback (optional, max. einmal, muss letzter sein)
        PRINT "Anders"
END SELECT
```

**Garantien:**
- Subject (`punkte`) wird **einmal** ausgewertet, auch bei Funktionsaufrufen mit Side-Effects.
- Erster passender CASE gewinnt; danach wird abgebrochen.
- Funktioniert auch mit STRINGs (`SELECT CASE name CASE "Anna", "Bert" THEN ... END SELECT`).

## Schleifen: FOR und WHILE

**FOR mit Zähler:**

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

**WHILE / WEND** (Pre-Test — Bedingung wird *vor* jedem Durchlauf geprüft, Body kann 0-mal laufen):

```basic
DIM i AS INTEGER
i = 0
WHILE i < 5
    PRINT i
    i = i + 1
WEND
```

**REPEAT / UNTIL** (Post-Test — Body läuft *immer mindestens einmal*, dann wird die Bedingung geprüft; Schleife endet wenn `UNTIL`-Ausdruck `TRUE` wird):

```basic
DIM i AS INTEGER
i = 0
REPEAT
    PRINT i
    i = i + 1
UNTIL i >= 3            ' "wiederhole BIS i >= 3"
```

Faustregel: `WHILE/WEND` wenn die Schleife evtl. gar nicht laufen soll, `REPEAT/UNTIL` wenn der Body in jedem Fall einmal durchlaufen muss (z.B. „Eingabe holen, dann auf Validität prüfen").

`STEP 0` ist ein Fehler. Bei negativem `STEP` läuft die Schleife abwärts; Schleife wird nicht ausgeführt wenn `start > end` (oder umgekehrt bei negativem STEP).

## BREAK und CONTINUE

```basic
FOR i = 1 TO 100
    IF i = 50 THEN
        BREAK            ' verlässt die Schleife sofort
    END IF
    IF i MOD 2 = 0 THEN
        CONTINUE         ' überspringt den Rest, geht zur nächsten Iteration
    END IF
    PRINT i
NEXT
```

Funktioniert in `FOR`, `WHILE` und `REPEAT/UNTIL`.

## DATA / READ / RESTORE

Klassische BASIC-Konstruktion für Inline-Datentabellen direkt im Quelltext — gut für Level-Layouts, Sprite-Definitionen, Lookup-Tabellen, Item-Listen.

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

**Erlaubte DATA-Werte**: nur Literale — Zahlen (mit Vorzeichen), Strings, `TRUE` / `FALSE`. Ausdrücke wie `2 + 3` oder Variablen werden nicht akzeptiert. Beispiele:

```basic
DATA -5, 3.14, "Hallo", TRUE, -100, "Mit ""Anführungszeichen"""
```

**`RESTORE`** setzt den Read-Pointer auf den Anfang zurück:

```basic
DATA 1, 2, 3
DIM x AS INTEGER

READ x          ' x = 1
READ x          ' x = 2
RESTORE
READ x          ' x = 1 (von vorn)
```

**Wo DATA stehen darf**: überall im Quelltext, auch innerhalb `SUB`/`FUNCTION`/`CLASS`. Beim Programmstart werden alle DATA-Zeilen in Source-Reihenfolge zu einer einzigen Liste zusammengelegt — als wären alle DATA-Statements am Anfang.

**READ-Ziele** können Variablen, Array-Elemente oder Klassen-Felder sein:

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
' ... weitere 8 Zeilen
```

## Zeilenfortsetzung

Lange Statements lassen sich auf mehrere Zeilen aufteilen:

```basic
' Implizit: innerhalb offener Klammern werden Newlines ignoriert
TEXT(
    x,
    y,
    "Lange Zeile mit vielen Argumenten",
    RGB(220, 220, 230)
)

' Explizit mit Unterstrich am Zeilenende
DIM total AS INTEGER
total = a + b + _
        c + d + _
        e
```

In String-Literalen sind Newlines weiterhin **nicht** erlaubt — das ist Absicht (`CHR$(10)` einfügen oder mit `+` verketten).

## Statement-Trenner

Mehrere Statements in einer Zeile werden mit Doppelpunkt `:` getrennt — wie im klassischen BASIC:

```basic
DIM x AS INTEGER : DIM y AS INTEGER
x = 1 : y = 2 : PRINT x + y       ' "3"

' Praktisch in kompakten SUBs
SUB Greet(name AS STRING) : PRINT "Hi " + name : END SUB
```

Das ist nicht zwingend — die meisten Programme bleiben mit einem Statement pro Zeile lesbarer. Aber für Init-Listen oder dichten Dummy-Code (Tests, Demo-Snippets) ist es hilfreich.

## Funktionen: SUB und FUNCTION

**SUB** (kein Rückgabewert):

```basic
SUB gruessen(name AS STRING)
    PRINT "Hallo, " + name
END SUB

gruessen("Anna")
```

**FUNCTION** (mit Rückgabewert):

```basic
FUNCTION quadriere(n AS INTEGER) AS INTEGER
    RETURN n * n
END FUNCTION

DIM ergebnis AS INTEGER
ergebnis = quadriere(7)        ' = 49
```

Mehrere Parameter:

```basic
FUNCTION distanz(x1 AS FLOAT, y1 AS FLOAT, x2 AS FLOAT, y2 AS FLOAT) AS FLOAT
    RETURN SQR((x2 - x1) ^ 2 + (y2 - y1) ^ 2)
END FUNCTION
```

**Default-Werte** für Parameter — der Aufrufer kann die letzten Argumente weglassen:

```basic
SUB greet(name AS STRING, prefix AS STRING = "Hallo")
    PRINT prefix, name
END SUB

greet("Anna")              ' "Hallo Anna"  (Default greift)
greet("Bert", "Hi")        ' "Hi Bert"
```

Default-Ausdrücke werden **bei jedem Aufruf** ausgewertet, im lokalen Scope der Funktion. Das heißt, ein späterer Default kann auf einen früheren Parameter zugreifen:

```basic
SUB rect_or_square(x AS INTEGER, y AS INTEGER, w AS INTEGER, h AS INTEGER = w)
    PRINT x, y, w, h
END SUB

rect_or_square(0, 0, 50)        ' h = 50 (Quadrat-Default)
rect_or_square(0, 0, 50, 30)    ' h = 30 (explizit)
```

**Regel**: Parameter mit Default müssen *am Ende* der Liste stehen. Ein Pflicht-Parameter nach einem Default-Parameter ist ein Compile-Fehler — sonst wäre nicht eindeutig welche Position welche bedeutet.

### Named Arguments

Argumente lassen sich auch *namentlich* übergeben — `name: wert`. Praktisch wenn eine SUB viele Parameter hat oder du mitten in der Default-Kette einen Slot setzen willst:

```basic
SUB Greet(name AS STRING, _
          age AS INTEGER, _
          greeting AS STRING = "Hallo", _
          suffix AS STRING = "")
    PRINT greeting + ", " + name + " (" + STR$(age) + ")" + suffix
END SUB

' Alle positional - klassisch
Greet("Anna", 30)

' Namen statt Position - Reihenfolge frei
Greet(name: "Bob", age: 25, greeting: "Hi")

' Mix: positional vorne, named hinten
Greet("Cara", 40, suffix: "!")

' Slot in der Mitte ueberspringen - greeting bleibt "Hallo"
Greet("Dora", 50, suffix: "?")
```

**Regeln**:

- **Positional vor Named**: alle positional Argumente müssen vor dem ersten Named-Argument stehen. `f(a: 1, 2)` ist Fehler.
- **Keine Doppel-Belegung**: ein Slot per positional UND named ist Fehler.
- **Unbekannter Name** ist Fehler — du kannst dich nicht vertippen ohne dass der Compiler es merkt.
- **Pflicht-Slots** müssen weiterhin gesetzt sein (entweder positional oder per Name).

**Verfügbar bei**:

- `SUB`/`FUNCTION`-Aufrufen
- `NEW Klasse(...)` (an die `Init`-Methode)

**Nicht verfügbar bei**:

- Built-ins (`ABS`, `CIRCLE`, `JSON_PARSE`, …) — die haben keine deklarierten Param-Namen
- Methoden-Aufrufen `obj.method(name: ...)` — die Klasse steht erst zur Laufzeit fest, daher wirft der `dhrt`-Compiler hier.

### BYREF-Parameter (Multi-Return)

Mit `BYREF` wird ein Parameter **per Referenz** übergeben — die Funktion kann die Variable des Aufrufers ändern. Das ist GBs Lösung für Mehrfach-Rückgabewerte.

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

`BYREF` darf an einer beliebigen Stelle in der Parameterliste stehen und mit normalen Parametern gemischt werden. Eine `FUNCTION` mit `BYREF`-Parameter kann zusätzlich einen regulären Wert zurückgeben:

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

**Erlaubte Argumente** für `BYREF`-Parameter:
- einfache Variablen — `swap(x, y)`
- Array-Elemente — `inc(arr[3])`
- Klassen-/Struct-Felder — `setpos(player.x)`

**Nicht erlaubt**: Literale, Ausdrücke, Funktionsaufrufe — der Aufrufer braucht ja eine Stelle, an die zurückgeschrieben werden kann.

**Einschränkungen:**
- `BYREF` darf **nicht** mit einem Default-Wert kombiniert werden (was würde es heißen, eine Default-Variable per Referenz zu übergeben?).
- `BYREF` wird von `dhrt` unterstützt: der Compiler setzt an der Aufruf­stelle eine lvalue-Erfassung plus Post-Call-Write-Back (Copy-In/Copy-Out), die VM gibt die finalen Parameter­werte zurück. (Aktuell nur bei direkten `SUB`/`FUNCTION`-Aufrufen — nicht über `FUNCREF` oder Methoden­aufrufe, deren Klasse erst zur Laufzeit feststeht.)

**Rekursion** funktioniert:

```basic
FUNCTION fib(n AS INTEGER) AS INTEGER
    IF n < 2 THEN
        RETURN n
    END IF
    RETURN fib(n - 1) + fib(n - 2)
END FUNCTION
```

## Coroutines: YIELD

Eine `FUNCTION` oder `SUB`, deren Body ein `YIELD` enthält, ist eine **Coroutine**. Ihr Aufruf führt den Body *nicht* aus, sondern liefert ein `COROUTINE`-Handle, das man schrittweise weitertreibt.

```basic
FUNCTION zaehler() AS INTEGER
    YIELD 1
    YIELD 2
    RETURN 99            ' Endwert (optional)
END FUNCTION

DIM c AS COROUTINE
c = zaehler()
PRINT CORO_RESUME(c)     ' 1
PRINT CORO_RESUME(c)     ' 2
PRINT CORO_RESUME(c)     ' 99  (jetzt beendet: CORO_DONE(c) = TRUE)
```

**Builtins:**

| Builtin | Wirkung |
|---|---|
| `CORO_RESUME(c)` | Fortsetzen bis zum nächsten `YIELD`; liefert den YIELD-Wert (oder den RETURN-Wert, wenn die Coroutine endet). |
| `CORO_SEND(c, v)` | Wie `CORO_RESUME`, aber der `YIELD`-Ausdruck im Body evaluiert zu `v`. |
| `CORO_DONE(c)` | `BOOLEAN` — ob die Coroutine beendet ist. |
| `CORO_RESULT(c)` | Finaler `RETURN`-Wert (wirft, wenn noch nicht beendet). |
| `CORO_CLOSE(c)` | Suspendierte Coroutine abbauen. |

**`YIELD` ist ein Ausdruck.** Als Statement (`YIELD v`) verwirft es den Sende-Wert; als Ausdruck liefert es den via `CORO_SEND` übergebenen Wert:

```basic
FUNCTION akkumulator() AS INTEGER
    DIM sum AS INTEGER
    sum = 0
    WHILE TRUE
        sum = sum + (YIELD sum)   ' gibt Summe ab, empfängt nächsten Summanden
    WEND
END FUNCTION

DIM acc AS COROUTINE
acc = akkumulator()
PRINT CORO_RESUME(acc)     ' 0  (Sende-Wert des ERSTEN Resume ist immer NIL)
PRINT CORO_SEND(acc, 10)   ' 10
PRINT CORO_SEND(acc, 5)    ' 15
CORO_CLOSE(acc)
```

**`FOR EACH` und Comprehensions** konsumieren eine Coroutine **eager** bis zum Ende (der `RETURN`-Wert ist nicht enthalten):

```basic
DIM total AS INTEGER
total = 0
FOR EACH n IN zaehler()
    total = total + n          ' 1 + 2 = 3 (RETURN 99 nicht dabei)
NEXT
```

Bei unendlichen Generatoren stattdessen manuell `CORO_RESUME`/`CORO_DONE`.

**Semantik & Einschränkungen:**
- `dhrt` suspendiert eine Coroutine via **Frame-Snapshot** (ip/locals/stack werden beim `YIELD` abgelegt und beim Resume restauriert) — kein OS-Thread, deterministisch, raylib-Main-Thread-sicher.
- **Kein Cross-Frame-`YIELD`:** ein Helfer mit `YIELD` ist selbst eine Coroutine; `YIELD` läuft also nie über einen normalen Funktionsaufruf hinweg.
- In `FUNCTION ... AS T` werden `YIELD`- *und* `RETURN`-Werte auf `T` gecoerct. Eine `SUB`-Coroutine yieldet ohne Typ-Coercion.
- Ein manueller `WHILE NOT CORO_DONE(c)`-Loop bekommt beim letzten (beendenden) `CORO_RESUME` den `RETURN`-Wert. Gib dem Generator einen typisierten `RETURN`, damit die Zuweisung an eine typisierte Variable klappt — oder nutze `FOR EACH`.
- Funktioniert auch im Standalone-`.exe`-Export (gleiche `dhrt`-VM).

Vollständiges Beispiel: [examples/98_coroutines.dh](../examples/98_coroutines.dh).

## Arrays

Eindimensional:

```basic
DIM zahlen[10] AS INTEGER         ' 10 Elemente, alle 0
zahlen[0] = 42
zahlen[9] = 99

DIM i AS INTEGER
FOR i = 0 TO LEN(zahlen) - 1
    PRINT zahlen[i]
NEXT
```

Mehrdimensional:

```basic
DIM brett[8, 8] AS INTEGER       ' Schachbrett
brett[0, 0] = 1
brett[7, 7] = 1

' Anzahl Dimensionen und Größe pro Dimension:
PRINT DIMCOUNT(brett)             ' 2
PRINT DIMSIZE(brett, 0)           ' 8
PRINT DIMSIZE(brett, 1)           ' 8
```

Array von externen Typen (z.B. SPRITE) ist möglich; Default-Wert ist `NIL`:

```basic
DIM coins[5] AS SPRITE
DIM i AS INTEGER
FOR i = 0 TO 4
    coins[i] = SPRITE_NEW(coin_img, 8, 8)
NEXT
```

> **Arrays werden per Referenz übergeben.** Übergibt man ein Array an eine
> `SUB`/`FUNCTION`, teilen sich Aufrufer und Aufgerufener denselben Speicher —
> Änderungen im Unterprogramm wirken auf das Original (genau das nutzen die
> `ARRAY_PUSH`/`SORT`/… und eigene In-Place-Routinen). Wer eine **eigene Kopie**
> braucht, ruft `ARRAY_COPY(arr)`.
>
> **Index-Zugriff ist streng, Slicing klemmt.** Ein direkter Index außerhalb der
> Grenzen wirft einen Laufzeitfehler (`Index 5 ausserhalb [0..2]`). Ein **Slice**
> dagegen wird still auf die gültigen Grenzen geklemmt: `arr[0:99]` auf ein
> 3er-Array liefert ohne Fehler 3 Elemente. (Slicing gibt es nur für 1D-Arrays.)

**Der Elementtyp gilt auch bei der Zuweisung.** Ein `ARRAY OF INTEGER` landet
nicht in einem `ARRAY OF STRING` — bei Variablen, Parametern und Rückgabewerten
gleichermaßen:

```basic
DIM zahlen AS ARRAY OF INTEGER
zahlen = [1, 2, 3]
DIM texte AS ARRAY OF STRING
texte = zahlen        ' Fehler: Erwartet ARRAY OF STRING, erhalten ARRAY OF INTEGER
```

Drei Fälle bleiben ausdrücklich erlaubt:

- Ein **leeres Literal** `[]` hat noch keinen Elementtyp und bekommt den des
  Ziels — `DIM namen AS ARRAY OF STRING : namen = []` bleibt ein
  `ARRAY OF STRING`.
- Ein **`ARRAY OF ANY`** darf man einem engeren Ziel geben. Es kann jeden Wert
  enthalten; das Einengen ist eine bewusste Entscheidung, der Schreibzugriff
  prüft danach wieder.
- Ein **frisches Ganzzahl-Literal** an einem FLOAT-Ziel wird umgebaut:
  `DIM w AS ARRAY OF FLOAT : w = [1, 2, 3]` ergibt ein echtes FLOAT-Array.
  Ein **vorhandenes** `ARRAY OF INTEGER` dagegen nicht — sein bisheriger Name
  zeigt ja weiter auf dieselben Zellen, und die können nicht zugleich INTEGER
  und FLOAT sein. Wer die Werte als FLOAT braucht, kopiert sie.

## Maps

Schlüssel sind immer STRINGs, Werte können beliebigen Typ haben.

```basic
DIM punkte AS MAP OF INTEGER
MAPPUT(punkte, "Anna", 95)
MAPPUT(punkte, "Bert", 78)

PRINT MAPGET(punkte, "Anna")           ' 95
PRINT MAPGETOR(punkte, "Eve", 0)       ' 0 (default)

IF MAPHAS(punkte, "Anna") THEN
    MAPREMOVE(punkte, "Anna")
END IF

PRINT MAPSIZE(punkte)                  ' 1
```

Mehr unter [Standard-Built-ins → Maps](builtins-core.md#maps).

## Klassen und Strukturen

**Klasse:**

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

Verwendung:

```basic
DIM p AS Player
p = NEW Player(100.0, 50.0)        ' ruft Init auf
p.MoveBy(10.0, 5.0)
PRINT p.x, p.y                     ' 110.0  55.0
IF p.IsAlive() THEN PRINT "noch da"
```

**Methoden-Bodies sehen Felder direkt** — kein `Self.`-Präfix nötig. Das `x = start_x` in `Init` setzt automatisch das Klassen-Feld `x`, `start_x` ist ein Parameter. Drachenhauch löst Namens-Lookups in Methoden so auf: erst lokale Variablen / Parameter, dann Klassen-Felder, dann globale Variablen.

**Methoden rufen sich gegenseitig auf — implizit:**

```basic
CLASS Counter
    DIM v AS INTEGER

    SUB Init()
        v = 0
        Reset()                ' Methode der eigenen Klasse, ohne Self.
    END SUB

    SUB Reset()
        v = 0
    END SUB
END CLASS
```

Innerhalb einer Methode wird ein Identifier-Aufruf wie `Reset()` zuerst in der eigenen Klasse (und Superklassen) gesucht. Findet sich eine Methode, wird sie als `Self.Reset()` aufgerufen. Wenn keine passt, fällt die Auflösung auf globale Funktionen zurück. Methoden gewinnen also gegen gleichnamige globale Funktionen — wie in den meisten OOP-Sprachen.

**`Self`** als Identifier liefert die aktuelle Instanz:

```basic
CLASS Box
    DIM v AS INTEGER

    FUNCTION Triple() AS INTEGER
        RETURN Self.v * 3      ' Self ist diese Instanz
    END FUNCTION

    SUB GiveTo(other AS Container)
        other.Add(Self)        ' Self als Argument an andere Methode
    END SUB
END CLASS
```

**Vererbung mit `EXTENDS`:**

```basic
CLASS Hero EXTENDS Player
    DIM weapon AS STRING

    SUB Init(start_x AS FLOAT, start_y AS FLOAT, w AS STRING)
        SUPER.Init(start_x, start_y)   ' die Init der Elternklasse
        hp = 150                       ' und danach das Eigene
        weapon = w
    END SUB
END CLASS
```

**`SUPER.Methode(...)`** ruft die Fassung der **Elternklasse** — auch dann,
wenn die eigene Klasse sie überschreibt:

```basic
CLASS Hero EXTENDS Player
    FUNCTION Beschreibung() AS STRING
        RETURN SUPER.Beschreibung() + " mit " + weapon
    END FUNCTION
END CLASS
```

Die Suche beginnt bei der Elternklasse *der Stelle im Quelltext*, nicht bei der
Klasse des Objekts. Deshalb funktioniert es auch über mehrere Ebenen: jede
Ebene fragt ihre eigene Elternklasse, statt sich im Kreis selbst aufzurufen.
Überspringt eine Zwischenklasse die Methode, wird weiter oben gesucht.

`SUPER` ist **kein** reserviertes Wort — eine Variable dieses Namens bleibt
erlaubt.

**`ABSTRACT`: eine Methode ankündigen, ohne sie zu schreiben**

```basic
CLASS Form
    ABSTRACT FUNCTION Flaeche() AS FLOAT
    ABSTRACT SUB Zeichne()

    FUNCTION Zeige() AS STRING          ' darf sie trotzdem benutzen
        RETURN "Flaeche: " + STR$(Flaeche())
    END FUNCTION
END CLASS
```

Eine angekündigte Methode hat keinen Rumpf und kein `END SUB`/`END FUNCTION`.
Wer eine Klasse mit noch offenen Ankündigungen mit `NEW` erzeugen will, bekommt
einen **Fehler beim Übersetzen** — nicht erst zur Laufzeit:

```
NEW form: die Klasse kuendigt eine Methode an, ohne sie auszufuellen (zeichne).
```

So lässt sich eine Basisklasse schreiben, die mit Methoden arbeitet, die es bei
ihr noch gar nicht gibt — `Zeige()` oben ruft `Flaeche()`, und zur Laufzeit
landet das bei der Unterklasse, die sie ausgefüllt hat.

`ABSTRACT` ist ebenfalls kein reserviertes Wort.

**STRUCT** ist ein leichtgewichtiges Daten-Klassen-Substitut, das automatisch instanziert wird (kein `NEW` nötig):

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

Innerhalb einer Methode greift man auf eigene Felder einfach per Name zu (kein `this.` oder `self.`).

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

`THROW <wert>` löst eine Exception aus (Wert ist immer STRING):

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

Die Catch-Variable ist optional (`CATCH` ohne Name), wenn man den Wert nicht braucht.

### FINALLY — aufräumen, egal wie man herauskommt

Ein `FINALLY`-Zweig läuft **immer**: nach einem sauberen Durchlauf, nach einem
gefangenen Fehler, bei einem Fehler, der weitergereicht wird — und auch, wenn
der Block per `RETURN`, `BREAK` oder `CONTINUE` verlassen wird.

```basic
DIM f AS FILE
f = OPENFILE("daten.txt", "r")
TRY
    verarbeite(READALL$(f))
CATCH e
    PRINT "Fehler: " + e
FINALLY
    CLOSEFILE(f)        ' passiert in JEDEM Fall
END TRY
```

`CATCH` und `FINALLY` sind einzeln optional, aber mindestens eines muss da
sein. **`TRY ... FINALLY ... END TRY` ohne `CATCH` fängt nichts** — es räumt
nur auf, und der Fehler läuft danach weiter nach außen. Das ist meistens genau
das Gewollte: aufräumen will man immer, entscheiden nur an einer Stelle.

Bei ineinander liegenden `TRY`-Blöcken laufen die `FINALLY`-Zweige von innen
nach außen.

> **Der Rückgabewert wird vor dem `FINALLY` berechnet.** `RETURN x` liefert das
> `x` von diesem Augenblick — was der `FINALLY`-Zweig danach mit der Variablen
> anstellt, ändert den zurückgegebenen Wert nicht mehr.

> **`TRY ... END TRY` ganz ohne `CATCH` und ohne `FINALLY`** verschluckt einen
> Fehler stillschweigend. Das war schon immer so und bleibt so — aber es ist
> selten das, was man will.

### Fehler-Code: entscheiden, ohne Texte zu vergleichen

`THROW` nimmt wahlweise einen Code vor der Meldung. Der Code ist das, worauf
man reagiert; die Meldung ist das, was man dem Benutzer zeigt:

```basic
THROW "NETZ", "Server antwortet nicht"
```

Im `CATCH` liefert `ERROR_CODE$()` den Code und `ERROR_LINE()` die Zeile, in
der der Fehler entstand:

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

Ein eingebauter Laufzeitfehler (Division durch null, Index außerhalb, …) hat
den Code `""` — nur ein `THROW` mit zwei Werten setzt einen. Beide Angaben
überstehen ein dazwischenliegendes `FINALLY`.

## Import

**Quellcode-Modul** (eine andere `.dh`-Datei einbinden):

```basic
IMPORT "mathlib.dh"

PRINT Distance(0.0, 0.0, 3.0, 4.0)    ' 5.0 - aus mathlib.dh
```

`IMPORT` ist textuelles Inkludieren — der Code aus `mathlib.dh` wird Teil des aktuellen Programms. Mehrfaches Importieren derselben Datei wird ignoriert (kein Endlos-Cycle).

### Namensraum: `IMPORT "datei.dh" AS name`

Ohne `AS` landen alle Namen der importierten Datei im selben flachen Raum wie
dein eigener Code — zwei Bibliotheken mit je einer Funktion `Init` lassen sich
so nicht zusammen benutzen. Mit `AS` bekommt die Datei einen eigenen Raum:

```basic
IMPORT "mathe.dh" AS mathe

PRINT mathe.Quadrat(5)      ' 25
PRINT mathe.FAKTOR          ' eine CONST aus mathe.dh
```

Innerhalb von `mathe.dh` ändert sich nichts: dort heisst `Quadrat` weiterhin
`Quadrat`, auch wenn eine andere Funktion derselben Datei sie aufruft.

**Ein Namensraum sieht die Globals deines Hauptprogramms nicht.** Das ist der
eigentliche Gewinn — die Datei hängt nicht mehr davon ab, welche Variablen du
zufällig oben deklariert hast:

```basic
' g.dh
FUNCTION LiestGlobal() AS INTEGER
    RETURN punkte          ' Fehler, wenn g.dh mit AS importiert wird
END FUNCTION
```

Reiche den Wert als Parameter herein oder deklariere ihn in der Datei selbst.
Ohne `AS` bleibt der alte, flache Zugriff erlaubt — bestehende Programme
ändern sich nicht.

**`PRIVATE`** versteckt einen Namen im Namensraum. Öffentlich ist die Vorgabe:

```basic
' p.dh
PRIVATE CONST GEHEIM AS INTEGER = 42

PRIVATE FUNCTION Intern(x AS INTEGER) AS INTEGER
    RETURN x + GEHEIM
END FUNCTION

FUNCTION Offen(x AS INTEGER) AS INTEGER
    RETURN Intern(x)       ' innen erlaubt
END FUNCTION
```

`p.Offen(1)` liefert `43`; `p.Intern(1)` meldet, dass der Name PRIVATE ist.
`PRIVATE` steht vor `SUB`, `FUNCTION`, `DIM` oder `CONST`. In einer Datei ohne
`AS` ist es ein wirkungsloser Marker.

**Klassen und Structs** gehen ebenfalls über den Namensraum — als Typ und
hinter `NEW`:

```basic
IMPORT "mathe.dh" AS mathe

DIM p AS mathe.Punkt
p = NEW mathe.Punkt()
p.x = 7
```

Damit dürfen beide Dateien eine Klasse `Punkt` haben. `ARRAY OF mathe.Punkt`
und `MAP OF mathe.Punkt` funktionieren ebenso. Innerhalb von `mathe.dh` heisst
die Klasse weiterhin schlicht `Punkt`.

**Noch nicht dabei: ENUMs.** `DIM f AS mathe.Farbe` meldet einen Fehler, der
auf diese Lücke hinweist. Wer ein ENUM aus der Datei braucht, importiert sie
zusätzlich ohne `AS`; dann steht es flach zur Verfügung.

**Nicht zu verwechseln** mit `AS` an einem eingebauten Modul: `IMPORT "json" AS
j` ersetzt dort das Präfix (`J_PARSE` statt `JSON_PARSE`). Für `.dh`-Dateien
gilt der Punkt, wie oben.

**Built-in-Modul:** ohne `.dh`-Endung wird ein internes Modul geladen.

```basic
IMPORT "json"
IMPORT "sprite"
IMPORT "camera"
```

Liste aller Module: siehe [README](README.md#module).

Die Auflösungs-Reihenfolge: erst wird `<name>.dh` im aktuellen Verzeichnis gesucht; existiert sie nicht, dann `drachenhauch/modules/<name>.py`. So kann ein eigenes `json.dh` Vorrang vor dem Built-in haben.

## Kommentare

Mit `'` (Apostroph) oder `REM` bis Zeilenende:

```basic
' Das ist ein Kommentar
REM auch ein Kommentar
PRINT "Hi"        ' Inline-Kommentar
```

Mehrzeilige Kommentare gibt es nicht — jede Zeile braucht eigenes `'`.

## Built-in-Konstanten

| Konstante | Wert | Zweck |
|---|---|---|
| `PI` | 3.141592653589793 | Kreiszahl |
| `TRUE` / `FALSE` | bool | Boolesche Literale |
| `NIL` | — | leerer Referenz-Wert |

Plus alle Farb-Konstanten (`BLACK`, `WHITE`, `RED`, `GREEN`, `BLUE`, `YELLOW`, `CYAN`, `MAGENTA`, `ORANGE`, `PURPLE`, `BROWN`, `PINK`, `DARKRED`, `DARKGREEN`, `DARKBLUE`, `GRAY`, `LIGHTGRAY`, `DARKGRAY`) und Tasten-Konstanten (`KEY_ESCAPE`, `KEY_RETURN`, `KEY_SPACE`, `KEY_LEFT/RIGHT/UP/DOWN`, `KEY_A` bis `KEY_Z`, `KEY_0` bis `KEY_9`, `KEY_F1` bis `KEY_F12`, Modifier, Navigationsblock und Ziffernblock — vollstaendige Tabelle in [builtins-grafik.md](builtins-grafik.md#eingabe-tastatur-und-maus)).
