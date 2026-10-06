# Fremde Bibliotheken aufrufen (`DECLARE … LIB`)

Was Drachenhauch nicht selbst mitbringt, liegt oft schon als C-Bibliothek
auf dem Rechner: eine Funktion der Windows-API, die Bibliothek eines
Messgeräts, die C-Bibliothek des Systems. Mit `DECLARE … LIB` ruft ein
Programm sie direkt auf, ohne Umweg über `SHELL` oder einen neuen Befehl in
der Laufzeit.

```basic
DECLARE FUNCTION MulDiv LIB "kernel32" (zahl AS LONG, mal AS LONG, durch AS LONG) AS LONG
DECLARE FUNCTION strlen LIB "c" (s AS TEXT) AS ZEIGER

PRINT strlen("Drachenhauch")        ' 12
```

> **Fremder Code läuft ohne Netz.** Ein falscher Zeiger, eine falsche
> Signatur oder ein Fehler in der Bibliothek beendet das ganze Programm --
> ohne `CATCH`, ohne Zeilennummer, im schlimmsten Fall schreibt er still in
> fremden Speicher. Was Drachenhauch prüfen kann (Argumentzahl, Typen,
> Wertebereiche, Nullzeichen im Text), prüft es; was in der Bibliothek
> passiert, nicht. Für eine Bibliothek, der man nicht traut, gibt es den
> Ausweg unten: [als Auftrag in einem eigenen Prozess](#unsichere-bibliotheken).

## Die Deklaration

```text
DECLARE FUNCTION name LIB "bibliothek" [ALIAS "c_name"] (parameter) AS typ
DECLARE SUB name LIB "bibliothek" [ALIAS "c_name"] (parameter)
```

* **Ohne `ALIAS`** ist der Name zugleich der in der Bibliothek, mit der
  Schreibweise aus dem Quelltext -- in einer Bibliothek zählt Groß/klein
  (`GetTickCount64`, nicht `gettickcount64`). Im Programm selbst ist sie wie
  überall egal.
* **Mit `ALIAS`** heißt die Funktion im Programm anders als in der Bibliothek:
  `DECLARE FUNCTION Bytes LIB "kernel32" ALIAS "lstrlenA" (s AS TEXT) AS LONG`.
* Die Zeile gehört auf die **oberste Ebene** des Programms, nicht in eine SUB.
  Wo sie steht, ist egal -- auch ein Aufruf davor findet sie.
* Danach ist der Name eine gewöhnliche Funktion: `--check` zählt die
  Argumente (ein Fehler beim Übersetzen) und warnt vor einem Argument, dessen
  Typ sicher nicht passt; Hover zeigt die Deklaration samt Kommentar darüber,
  Zur Definition springt zu ihr.
* Eine Deklaration darf heißen wie ein eingebauter Befehl
  (`DECLARE FUNCTION abs LIB "c" ...`) -- dann meint der Aufruf sie. Mit
  einer eigenen SUB/FUNCTION oder einer Variable gleichen Namens ist es ein
  Fehler.
* `DECLARE`, `LIB` und `ALIAS` sind keine Schlüsselwörter; eine Variable
  darf so heißen.

Wer aus VB oder QBasic kommt: ein `DECLARE` für eigene SUBs braucht es hier
nicht, die Meldung sagt das. `ByVal` vor einem Parameter wird übergangen
(es ist die Vorgabe).

## Die Typen

Die Typwörter gibt es nur in der Parameterliste einer `DECLARE`-Zeile -- die
Drachenhauch-Typen sagen nicht, wie breit eine Zahl in C ist.

| in `DECLARE` | in C | in Drachenhauch |
|---|---|---|
| `BYTE` / `UBYTE` | `int8_t` / `uint8_t` | INTEGER |
| `SHORT` / `USHORT` | `int16_t` / `uint16_t` | INTEGER |
| `LONG` / `ULONG` | `int32_t` / `uint32_t` (auch `int`, Windows `DWORD`) | INTEGER |
| `INTEGER` | `int64_t` | INTEGER |
| `ZEIGER` (auch `PTR`) | `void*`, `size_t`, `HWND` ... | INTEGER |
| `SINGLE` | `float` | FLOAT |
| `FLOAT` | `double` | FLOAT |
| `BOOLEAN` | `int` (0 = falsch) | BOOLEAN |
| `TEXT` (auch `CSTR`) | `const char*`, UTF-8 | STRING |
| `WTEXT` (auch `WSTR`) | `const wchar_t*` (Windows: UTF-16, sonst UTF-32) | STRING |
| `BUFFER` | `void*` auf die Bytes des Puffers | BUFFER |
| `FUNCTION(...) AS typ`, `SUB(...)` | Funktionszeiger (Rückruf) | FUNCREF |

* **Ein Wert, der nicht passt, ist ein Fehler**: `LONG` mit 2^40 bricht ab,
  statt still abgeschnitten zu werden; eine Kommazahl für `LONG` ebenso.
  `ULONG` nimmt 0 bis 4294967295 -- ein Windows-`INFINITE` schreibt man
  `&HFFFFFFFF`.
* **`long` in C** ist unter Windows 32 Bit, unter Linux und macOS 64 Bit --
  dort heißt es `INTEGER`. `size_t` ist immer `ZEIGER`.
* **Text** wird für den Aufruf kopiert; ein Nullzeichen mitten darin ist ein
  Fehler (C sähe nur den Anfang). `NIL` statt eines Textes übergibt einen
  Nullzeiger.
* **`BUFFER`** übergibt einen Zeiger auf die Bytes selbst -- die Bibliothek
  darf hineinschreiben, und das Programm sieht es danach. So kommen Structs
  und Ausgabetexte zurück (siehe unten). Die Größe legt das Programm vorher
  fest; schreibt die Bibliothek darüber hinaus, ist das einer der Abstürze
  von oben.
* **`BYREF`** vor einem Zahlentyp (oder `ZEIGER`/`BOOLEAN`) übergibt einen
  Zeiger auf eine Kopie und schreibt sie nach dem Aufruf zurück -- für
  Ausgabeparameter. Das Argument muss eine Variable, ein Feld- oder ein
  Objektelement sein.
* **Rückgabe:** jeder Typ außer `BUFFER`. `TEXT`/`WTEXT` werden aus dem
  Zeiger kopiert (ein Nullzeiger wird `""`); freigegeben wird nichts --
  braucht die Bibliothek das, gibt
  man `ZEIGER` zurück, liest ihn mit `TEXT_AUS_ZEIGER$` und ruft danach ihre
  eigene Freigabe (siehe [Zeiger](#zeiger)).

## Die Bibliothek finden

Der Name ist **plattformneutral**: `"user32"` wird `user32.dll`, `"sqlite3"`
unter Linux `libsqlite3.so`, unter macOS `libsqlite3.dylib`. Zwei Namen
gelten überall:

* `"c"` -- die C-Bibliothek des Systems (Windows `ucrtbase.dll`, Linux
  `libc.so.6`, macOS `libSystem`),
* `"m"` -- ihre Mathematik (`pow`, `hypot`; unter Windows und macOS dieselbe
  Datei).

Ein Name mit Endung oder Pfad gilt wörtlich (`"libsqlite3.so.0"`,
`"lib/messgeraet.dll"`). Gesucht wird **neben dem Programm, neben `dhrt`
(bzw. der exportierten Exe), dann wo das System sucht.**

**Geladen wird beim ersten Aufruf**, nicht beim Start: ein Programm, das den
Zweig nie nimmt, läuft auch ohne die Bibliothek. Fehlt sie oder die Funktion
darin, ist das ein gewöhnlicher Laufzeitfehler mit den versuchten Namen --
abzufangen mit `CATCH`:

```basic
DECLARE FUNCTION messen LIB "messgeraet" () AS FLOAT
TRY
    PRINT messen()
CATCH e
    PRINT "Kein Messgerät: "; e
END TRY
```

## Beispiele

**Windows-API** -- Zahlen hin und zurück, ein Struct (siehe
[Struct-Lage](#struct-lage)), ein Ausgabeparameter über `BYREF`:

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
n = 64                              ' hinein: Platz in Zeichen, heraus: Länge
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

**C-Bibliothek** -- dieselben Zeilen laufen unter Windows, Linux und macOS:

```basic
DECLARE FUNCTION getenv LIB "c" (name AS TEXT) AS TEXT
DECLARE FUNCTION strtod LIB "c" (s AS TEXT, BYREF ende AS ZEIGER) AS FLOAT
DECLARE FUNCTION hypot LIB "m" (x AS FLOAT, y AS FLOAT) AS FLOAT

PRINT getenv("PATH")
DIM ende AS INTEGER
PRINT strtod("2.5 Meter", ende)     ' 2.5
PRINT hypot(3.0, 4.0)               ' 5.0
```

## Zeiger

Manche Bibliotheken liefern keinen Wert, sondern einen **Zeiger** auf
Speicher, der ihnen gehört: einen Text, den der Aufrufer danach freigeben
soll, ein Feld von Bytes, ein Struct. Drei Befehle verbinden `ZEIGER` und
`BUFFER`:

| Befehl | was es tut |
|---|---|
| `TEXT_AUS_ZEIGER$(zeiger [, breit])` | kopiert den Text hinter einem Zeiger bis zum Nullzeichen (UTF-8; mit `breit` = TRUE `wchar_t`, also UTF-16 unter Windows); 0 ergibt `""` |
| `BUFFER_AUS_ZEIGER(zeiger, laenge)` | kopiert `laenge` Bytes hinter einem Zeiger in einen neuen BUFFER; 0 Bytes ergeben einen leeren, ein Nullzeiger mit Länge ist ein Fehler |
| `BUFFER_ZEIGER(puffer)` | die Adresse der Bytes eines BUFFER, für ein Struct-Feld, das auf einen anderen Puffer zeigt; ein leerer Puffer ergibt 0 |

```basic
DECLARE FUNCTION malloc LIB "c" (n AS ZEIGER) AS ZEIGER
DECLARE FUNCTION strcpy LIB "c" (ziel AS ZEIGER, quelle AS TEXT) AS ZEIGER
DECLARE SUB free LIB "c" (z AS ZEIGER)

DIM z AS INTEGER
z = malloc(16)                             ' 16 Bytes, die der C-Bibliothek gehoeren
strcpy(z, "Grüße")
PRINT TEXT_AUS_ZEIGER$(z)                  ' Grüße
PRINT BUFFER_LEN(BUFFER_AUS_ZEIGER(z, 8))  ' 8: sieben Bytes UTF-8 und die Null
free(z)                                    ' der Speicher gehört der Bibliothek
```

* **Alle drei kopieren** -- ein Text oder BUFFER aus einem Zeiger hängt
  danach nicht mehr am Speicher der Bibliothek, sie darf ihn freigeben.
* **Sie vertrauen dem Zeiger.** Ein falscher oder schon freigegebener Zeiger
  ist einer der Abstürze von oben; eine zu große Länge liest über das Ende
  hinaus. Geprüft wird, was sich prüfen lässt: Nullzeiger, negative Länge,
  höchstens 1 GiB.
* **`BUFFER_ZEIGER` gilt, solange der Puffer seine Größe behält.**
  `BUFFER_RESIZE` legt die Bytes woanders hin, der alte Zeiger zeigt dann ins
  Leere. Die Bibliothek darf den Zeiger nur so lange halten, wie das Programm
  den Puffer unverändert lässt.

## Struct-Lage

Ein C-Struct ist ein Stück Speicher mit Feldern an festen Stellen. Mit
`STRUCT name LAYOUT C` beschreibt man es mit denselben Typwörtern wie in der
`DECLARE`-Zeile -- die Stellen samt Ausrichtung rechnet Drachenhauch aus:

```basic
STRUCT Punkt LAYOUT C
    x AS LONG
    y AS LONG
END STRUCT

STRUCT Linie LAYOUT C
    farbe AS UBYTE
    ende[2] AS Punkt          ' ein Feld von zwei eingebetteten Structs
    name AS TEXT * 16         ' 16 Zeichen fester Breite
    gewicht AS FLOAT
END STRUCT

DIM l AS Linie                ' ein BUFFER in genau dieser Groesse, voller Nullen
l.ende[1].x = 40
l.name = "Diagonale"
PRINT l.ende[1].x, l.name     ' 40  Diagonale
PRINT SIZEOF(Linie), OFFSETOF(Linie, name)   ' 48  20
```

* **Eine Variable dieses Typs ist ein BUFFER** (`TYPEOF` sagt `BUFFER`) --
  sie geht ohne Umweg an eine `DECLARE`-Funktion, deren Parameter `AS BUFFER`
  oder gleich `AS Linie` heißt, und die Bibliothek schreibt hinein.
* **Felder:** die Typwörter der `DECLARE`-Zeile außer `BUFFER`; `BOOLEAN` ist
  4 Bytes wie `int` (ein C-`bool` mit einem Byte ist `UBYTE`). Dazu
  `name[n] AS typ` für ein Feld von Elementen, `TEXT * n` bzw. `WTEXT * n`
  für Zeichen fester Breite (gelesen bis zum Nullzeichen; ein zu langer Text
  ist ein Fehler) und ein anderer `STRUCT … LAYOUT C` -- er darf auch
  weiter unten stehen, sich aber nicht selbst enthalten. Ein Zeiger auf Text
  oder einen anderen Puffer ist ein `ZEIGER`-Feld (siehe unten). `DIM` vor
  dem Feld ist erlaubt, nicht nötig.
* **Die Regeln sind die von C:** jedes Feld liegt auf einer Stelle, die
  durch seine Größe teilbar ist (ein Struct durch die seines strengsten
  Feldes), und der Struct ist so lang, dass ein zweiter direkt dahinter
  passte. **`PACK n`** (`STRUCT Kopf LAYOUT C PACK 1`) begrenzt die
  Ausrichtung wie `#pragma pack(n)` -- für Dateiformate und die wenigen APIs,
  die gepackt sind.
* **`SIZEOF(typ)`** und **`OFFSETOF(struct, feld)`** sind feste Zahlen beim
  Übersetzen; `SIZEOF` kennt auch die Typwörter (`SIZEOF(ZEIGER)` ist 8).
* **Lesen und Schreiben prüfen** wie die `DECLARE`-Zeile: ein Wert, der
  nicht in das Feld passt, ist ein Fehler, ebenso ein Index außerhalb oder
  ein Puffer, der kürzer ist als der Struct.
* **Ein Struct ist ein BUFFER, also eine Referenz:** `b = a` teilt die Bytes,
  eine Kopie macht `BUFFER_SLICE(a, 0, SIZEOF(Punkt))`. Umgekehrt lässt sich
  jeder Puffer durch eine Lage lesen (`DIM p AS Punkt : p = roh`).
* Ein Struct lebt in einer Variable, einem Parameter oder einer Rückgabe;
  als Feld einer Klasse, in `ARRAY OF` oder `DIM x[n]` geht er (noch) nicht
  -- ein Feld von Structs ist ein Struct mit einem Feld davon
  (`e[10] AS Punkt`). Struct-Namen gelten im ganzen Programm, auch aus
  einer Datei mit Namensraum.

| Befehl | was es tut |
|---|---|
| `SIZEOF(typ)` | Größe eines `STRUCT … LAYOUT C` oder eines Typworts in Bytes, beim Übersetzen gerechnet |
| `OFFSETOF(struct, feld)` | Stelle eines Feldes im Struct in Bytes, beim Übersetzen gerechnet |

Manche Structs wollen vor dem Aufruf ihre eigene Größe im ersten Feld --
unter Windows etwa `MEMORYSTATUSEX` für `GlobalMemoryStatusEx`:

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

Auf Linux und macOS hat `struct utsname` (für `uname`) Felder fester Breite,
65 Bytes unter Linux und 256 unter macOS -- die Lage hängt dort am System.
Man gibt der Funktion den größeren Struct und liest danach durch die Lage
des Systems (`tests/pruef/ffi_struct.dhtest` zeigt es).

### Structs mit Zeigern

Ein Feld vom Typ `char*` oder `void*` ist ein `ZEIGER`. Gesetzt wird es mit
`BUFFER_ZEIGER`, gelesen mit `TEXT_AUS_ZEIGER$` bzw. `BUFFER_AUS_ZEIGER`:

```basic
STRUCT Eintrag LAYOUT C
    name AS ZEIGER            ' const char*
    laenge AS LONG
END STRUCT

DIM name AS BUFFER
name = BUFFER_CONCAT(BUFFER_FROM_STRING("Drache"), BUFFER_NEW(1))   ' mit der Null
DIM e AS Eintrag
e.name = BUFFER_ZEIGER(name)
e.laenge = 6
PRINT TEXT_AUS_ZEIGER$(e.name), SIZEOF(Eintrag)   ' Drache  16
```

Auch ein Element, das eine Bibliothek per Zeiger übergibt -- etwa die zwei
Elemente im Vergleich von `qsort` --, liest man durch die Lage:
`DIM a AS Punkt : a = BUFFER_AUS_ZEIGER(zeiger, SIZEOF(Punkt))`.

## Rückrufe

Manche Bibliotheken rufen zurück: `qsort` fragt für jedes Paar, welches
Element zuerst kommt, `EnumWindows` meldet jedes Fenster einzeln. Der
Parameter dafür heißt in der `DECLARE`-Zeile so, wie der Rückruf aussieht
-- die Schreibweise von FreeBASIC:

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

* **Die Klammern nennen die C-Typen**, die der Rückruf bekommt, mit oder
  ohne Namen (`FUNCTION(ZEIGER, ZEIGER) AS LONG`). Erlaubt sind die
  Zahltypen, `ZEIGER`, `BOOLEAN` und `TEXT`/`WTEXT` (kommt als STRING an);
  kein `BUFFER` (Speicher der Bibliothek ist ein `ZEIGER`) und kein `BYREF`.
  Zurück gibt ein Rückruf eine Zahl, einen `ZEIGER` oder `BOOLEAN`, oder als
  `SUB(...)` nichts.
* **Übergeben wird eine Funktion**: ihr Name ohne Klammern, eine gebundene
  Methode (`zaehler.eins` -- das Objekt kommt mit) oder ein Lambda. `NIL`
  übergibt einen Nullzeiger. Passt die Zahl ihrer Parameter nicht, ist das
  ein Fehler, bevor die Bibliothek überhaupt gerufen wird.
* **Ein Fehler im Rückruf** kommt beim Aufruf der Bibliothek an, sobald sie
  zurückkehrt -- als gewöhnlicher Laufzeitfehler, abfangbar mit `CATCH`.
  Durch die Bibliothek hindurch abbrechen kann er nicht; ab dem Fehler
  liefert jeder weitere Rückruf dieses Aufrufs 0, und die Bibliothek
  arbeitet mit diesen Antworten zu Ende. Dasselbe gilt für einen Wert vom
  falschen Typ (eine BOOLEAN-Funktion, wo `LONG` verlangt ist).
* **Ein Rückruf darf selbst Bibliotheken rufen**, auch dieselbe.
* **Nur auf dem Faden des Programms.** Ruft eine Bibliothek aus einem eigenen
  Faden zurück (`CreateThread`, `pthread_create`, manche Treiber), wird der
  Rückruf nicht ausgeführt -- er liefert 0, und der nächste Aufruf einer
  Bibliothek, der zurückkehrt, meldet es. Das kann schon der sein, der den
  Faden gestartet hat (sein Ergebnis geht dann verloren); unter Windows
  hilft `CREATE_SUSPENDED` und ein eigenes `ResumeThread`. Die VM ist nicht
  für mehrere Fäden gebaut.
* **Ein Rückruf bleibt gültig, bis das Programm endet** -- die Bibliothek
  darf ihn behalten und später rufen. Für dieselbe Funktion bekommt sie
  immer denselben Einstieg; ein Lambda, das in einer Schleife jedes Mal neu
  entsteht, bekommt jedes Mal einen neuen (wenige Bytes, die bis zum Ende
  bleiben). Ein Fehler in einem Rückruf, den die Bibliothek außerhalb eines
  Aufrufs ruft, meldet sich beim nächsten Aufruf einer Bibliothek.

## Export

`dhrt --export` nimmt jede Bibliothek mit, die **neben dem Programm** liegt,
und legt sie neben die Exe -- dort sucht die Laufzeit zuerst. Ein Name mit
Pfad (`"lib/messgeraet.dll"`) behält seinen Ordner. Gezählt werden nur
Bibliotheken, deren Funktionen das Programm auch aufruft; die
C-Bibliothek (`"c"`, `"m"`) nie. Was nicht daneben liegt, nennt der Export
als Hinweis -- eine Systembibliothek wie `kernel32` hat jeder Zielrechner,
eine fremde muss man selbst mitgeben:

```text
  Bibliothek "messgeraet" mitkopiert: messgeraet.dll
  Hinweis: Bibliothek "kernel32" liegt nicht neben dem Programm -- der Zielrechner muss sie haben (eine Systembibliothek wie kernel32 hat er)
```

Mitgenommen wird die Datei **des Systems, auf dem exportiert wird**
(`messgeraet.dll` unter Windows, `libmessgeraet.so` unter Linux) -- ein
Export läuft ohnehin nur auf diesem System.

## Unsichere Bibliotheken

Eine Bibliothek, die abstürzen könnte, ruft man in einem **Auftrag** auf
(`TASK_START`): der läuft als eigener Prozess. Stürzt er ab, bekommt das
Hauptprogramm beim Abholen einen Fehler und läuft weiter.

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
    PRINT e         ' "TASK: der Auftrag ist abgestuerzt ..."
END TRY
```

## Was es (noch) nicht gibt

* **Structs als Wert** übergeben (nur über einen Zeiger) und Bitfelder.
* **Variable Argumentzahl** (`printf`), C++-Namen, COM.
* **Im Browser** gibt es keine fremden Bibliotheken; ein Aufruf ist dort ein
  Fehler mit diesem Satz.

Der Maschinencode nimmt Aufrufe fremder Funktionen nicht in seine Schleifen
auf, sie laufen in der VM -- eine Frage der Geschwindigkeit, nicht der
Richtigkeit.

## Unter der Haube

Der Compiler macht aus jedem Aufruf den internen Befehl `__ffi` und gibt ihm
die Deklaration mit. Für jeden Rückruf baut die Laufzeit einen Einstieg mit
der C-Signatur, der die Argumente einsammelt und die Funktion über die VM
ruft. Die Laufzeit lädt die Bibliothek beim ersten Aufruf
(`libloading`) und baut je Signatur einmal einen kleinen Übergang mit
Cranelift -- derselben Bibliothek, die auch den Maschinencode erzeugt; er
legt die Argumente nach der C-Aufrufkonvention des Systems ab. Entwurf und
Entscheidungen: [entwurf-ffi.md](entwurf-ffi.md), Quelltext
`rust/drachenhauch_runtime/src/ffi.rs`, Prüfungen
`tests/pruef/ffi.dhtest`.
