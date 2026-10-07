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
(es ist die Vorgabe) -- außer vor einem Struct, dort heißt es „als Wert“
(siehe [Structs als Wert](#structs-als-wert)).

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

* **Mehrere Namen** stehen durch `|` getrennt: `LIB "libgtk-3-0.dll|libgtk-3.so.0"`.
  Der erste, der sich laden lässt, gilt -- so trägt eine Zeile die Namen
  aller Systeme, wo eine Bibliothek nicht überall gleich heißt.
* **Statt des Textes eine CONST:** `CONST GTK = "libgtk-3-0.dll|libgtk-3.so.0"`
  und dann `LIB GTK` in jeder Zeile. Die CONST braucht einen festen Text;
  in einer Datei mit Namensraum gehört sie dieser Datei.
* **Erst zur Laufzeit bekannt:** `LIB "$NAME"` nimmt den Inhalt der
  Umgebungsvariablen `NAME`, gelesen beim **ersten Aufruf**. Ein Programm
  sucht die Bibliothek also selbst und trägt sie vorher mit
  `SETENV("NAME", pfad)` ein -- so findet `python.dh` die DLL des Pythons,
  das auf dem Rechner liegt (siehe [Python einbetten](#python-einbetten)).
  Der Inhalt darf mehrere Namen mit `|` tragen; `LIB "$NAME|ersatz"` nimmt
  `ersatz`, wenn die Variable leer ist. Fehlt alles, nennt die Meldung die
  Variable.
* **Abhängigkeiten unter Windows:** Bei einem Pfad (`LIB "C:/Programme/X/x.dll"`)
  sucht Windows die Bibliotheken, die diese DLL selbst braucht, zuerst in
  ihrem Ordner -- der muss nicht im PATH stehen.
* Liegt eine Datei unter dem Namen da, lässt sich aber nicht laden, sagt
  die Meldung das samt dem Grund des Systems: meist fehlt ihr selbst eine
  Bibliothek, oder sie ist für 32 Bit gebaut.

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

### Bitfelder

Manche Structs packen mehrere kleine Zahlen in ein Wort -- in C
`DWORD fBinary : 1;`. In Drachenhauch steht die Breite in Bits hinter dem
Typ (wie in C) oder, wie in FreeBASIC, hinter dem Namen:

```basic
STRUCT Status LAYOUT C
    bereit AS ULONG : 1
    modus AS ULONG : 2
    stufe AS LONG : 5           ' mit Vorzeichen: -16 bis 15
    fehler : 1 AS BOOLEAN       ' die Schreibweise von FreeBASIC
    zaehler AS ULONG
END STRUCT

DIM s AS Status
s.modus = 3
s.stufe = -2
PRINT s.modus, s.stufe, HEX$(BUFFER_GET_U32(s, 0))   ' 3  -2  F6
```

* **Ein Bitfeld liest und schreibt nur seine Bits**; was nicht hineinpasst,
  ist ein Fehler (`s.modus = 4` -- „passt nicht in ein Bitfeld mit 2 Bits
  (0 bis 3)“). Mit Vorzeichen (`BYTE`, `SHORT`, `LONG`, `INTEGER`) kommt eine
  negative Zahl zurück, ohne (`UBYTE` … `ULONG`) nicht. `BOOLEAN` nimmt
  `TRUE`/`FALSE` (seine Einheit sind 4 Bytes wie bei `int`; ein C-`bool`
  mit einem Bit ist `UBYTE : 1`).
* **Wie Bitfelder liegen, legt der Compiler des Systems fest, und dhrt hält
  sich daran:** unter Windows (MSVC) teilen sich aufeinander folgende
  Bitfelder eine Einheit ihres Typs nur, wenn der Typ gleich groß ist und
  die Bits noch passen; unter Linux und macOS (GCC, Clang) kommt ein
  Bitfeld an die nächste freie Bitstelle, solange es keine Grenze seines
  Typs überschreitet -- auch direkt hinter ein gewöhnliches Feld. Dieselbe
  Deklaration kann darum auf den Systemen verschieden lang sein, genau wie
  in C (`char c; int a : 4;` ist unter Windows 8 Bytes, sonst 4).
* `OFFSETOF` eines Bitfelds ist ein Fehler (es hat keine Stelle in Bytes),
  ein Feld von Bitfeldern gibt es nicht, und mit `PACK` gehen Bitfelder
  (noch) nicht -- dort weichen die Compiler voneinander ab. Ein Bitfeld mit
  0 Bits (C: `int : 0;`) gibt es nicht; ein unbenanntes Füllfeld bekommt
  einfach einen Namen.

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

### Structs als Wert

Manche C-Funktion nimmt einen Struct nicht als Zeiger (`Punkt*`), sondern
als Wert (`Punkt`), oder sie gibt einen zurück -- `div` liefert ein `div_t`,
Grafik-Bibliotheken nehmen Punkte und Farben so. Dafür steht **`BYVAL`** vor
dem Parameter, und die Rückgabe heißt einfach wie der Struct:

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

* **Ohne `BYVAL` bleibt es ein Zeiger** -- so wie bisher; `p AS Punkt` heißt
  in C `Punkt*`, `BYVAL p AS Punkt` heißt `Punkt`. Die C-Deklaration sagt,
  welches gemeint ist.
* **Die Funktion bekommt eine Kopie.** Was sie daran ändert, sieht das
  Programm nicht; der Puffer muss mindestens so lang sein wie der Struct.
* **Eine Rückgabe ist ein neuer Struct** (ein BUFFER in seiner Größe), den
  man einer Variable des Typs zuweist. Liefert die Bibliothek einen
  *Zeiger* auf einen Struct, ist die Rückgabe weiter `ZEIGER`.
* **Wie der Struct reist, entscheidet das System**, und Drachenhauch hält
  sich an dessen Regeln: unter Windows x64 geht ein Struct mit 1, 2, 4 oder
  8 Bytes in einem Register, jeder andere als Zeiger auf eine Kopie; unter
  Linux und macOS auf Intel (System V) gehen bis zu 16 Bytes in Registern
  -- Kommazahlen in den SSE-, alles andere in den Ganzzahl-Registern --,
  Größeres auf dem Stapel; auf ARM (Linux und Apple) gehen bis zu vier
  gleiche Kommazahlen in Gleitkomma-Registern, sonst bis zu 16 Bytes in
  Ganzzahl-Registern und Größeres als Zeiger. Im Programm sieht man davon
  nichts.
* **Komplexe Zahlen** (`double complex`, `_Dcomplex`) behandelt C genau wie
  einen Struct aus zwei Kommazahlen -- `csqrt`, `cexp` und Co. aus der
  C-Bibliothek gehen damit wie oben.
* Im **Rückruf** geht ein Struct ebenso als Wert, siehe
  [Structs als Wert im Rückruf](#structs-als-wert-im-rückruf).
* Nicht (noch) als Wert: ein Struct in einer Funktion mit **`...`**; auf ARM
  ein Struct aus Kommazahlen, für den hinter acht Kommazahl-Argumenten kein
  Register mehr frei ist. Beide sind eine Meldung, kein stiller Fehler.

## GTK

[GTK](https://www.gtk.org/) ist die Bibliothek, mit der unter Linux die
meisten Programme ihre Fenster zeichnen. Mit `DECLARE … LIB` und Rückrufen
lässt sie sich direkt benutzen -- das Beispiel `207_gtk.dh` baut ein Fenster
mit Eingabefeld, Knöpfen und einer Beschriftung:

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
    gtk_main()                     ' die Schleife von GTK; Rueckrufe kommen von hier
END IF
```

* **`g_signal_connect` ist in C nur ein Makro** -- gerufen wird
  `g_signal_connect_data`. Weil jede Signalart einen anderen Rückruf hat,
  bekommt jede Form eine eigene `DECLARE`-Zeile mit `ALIAS` auf dieselbe
  C-Funktion.
* `gtk_main` blockiert, bis `gtk_main_quit` gerufen wird; die Rückrufe laufen
  auf dem Faden des Programms. Ein raylib-Fenster daneben gibt es nicht --
  ein Programm nimmt GTK oder die eigene gui.
* GTK muss auf dem Rechner sein: unter Linux fast immer, unter macOS über
  Homebrew, unter Windows über MSYS2 oder aus einem Programm, das es
  mitbringt (das Beispiel versucht das von Inkscape).
* Eigenschaften setzt `g_object_set` -- eine Funktion mit `...`, siehe
  [Variable Argumentzahl](#variable-argumentzahl):
  `g_object_set(knopf, "tooltip-text", "Sagt Hallo", NIL)`.

## Python einbetten

Python ist selbst eine C-Bibliothek (`python3XY.dll`, `libpython3.X.so`) --
mit ihr laufen alle Pakete, die im Python des Rechners installiert sind:
numpy, Pillow, PySide6 (Qt) und was sonst. Die Bibliothek
`examples/python/python.dh` erledigt die Deklarationen; man kopiert sie
neben das eigene Programm und importiert sie:

```text
IMPORT "python/python.dh"

pythonStarten()
pythonAusfuehren("import statistics")
PRINT pythonZahl("statistics.median([3, 1, 4, 1, 5])")     ' 3.0

pythonSetzeText("name", "Drache")
PRINT pythonText$("name.upper() + '!'")                     ' DRACHE!
```

| Befehl | Was er tut |
|---|---|
| `pythonStarten([programm$])` | sucht ein Python (Argument, `DH_PYTHON`, `py -3`, `python3`, `python`) und startet es; ein zweiter Aufruf tut nichts |
| `pythonVorhanden()` | TRUE, wenn sich eins starten lässt -- für Programme mit einem Weg ohne Python |
| `pythonAusfuehren(code$)` | Anweisungen, auch mehrere Zeilen (`import`, `def`, `class`) |
| `pythonZahl(a$)`, `pythonGanz(a$)`, `pythonText$(a$)` | einen Ausdruck auswerten -- als Kommazahl, INTEGER oder Text |
| `pythonBytes(a$)` | `bytes(...)` als BUFFER -- ein numpy-Feld liefert seine Rohdaten |
| `pythonSetzeZahl/Ganz/Text/Bytes(name$, wert)` | einen Wert unter einem Namen in Python ablegen |

* **Welches Python:** das, das ein Aufruf auf der Kommandozeile liefern
  würde, oder das in `DH_PYTHON` (der Pfad zum Programm, etwa
  `.venv/Scripts/python.exe`). Es wird einmal nach seiner Bibliothek und
  seinem Suchpfad gefragt; eingebettet bekommt es denselben Suchpfad, also
  auch die Pakete eines venv und im Benutzerordner.
* **Felder** gehen als Bytes: ein BUFFER aus Kommazahlen ist in Python
  `numpy.frombuffer(roh, '<f8')`, und `pythonBytes("feld.astype('<f8')")`
  bringt das Ergebnis zurück. Das Beispiel `208_python.dh` rechnet so das
  Spektrum eines Signals -- mit numpy oder, ohne numpy, in reinem Python.
* **Fehler in Python** kommen als Fehler in Drachenhauch an:
  `Python: ZeroDivisionError: division by zero`, bei mehrzeiligem Code mit
  der Zeile dahinter. `CATCH` fängt sie ab, danach geht es weiter.
* **Qt** geht über PySide6 im eingebetteten Python: ein Fenster mit
  `QApplication` öffnet sich aus dem Drachenhauch-Programm heraus. Solange
  `app.exec()` läuft, steht das Drachenhauch-Programm.
* **Grenzen:** Python läuft im selben Prozess -- stürzt eine Erweiterung ab,
  ist das Programm weg. Was Python mit `print` ausgibt, steht nicht in der
  Reihenfolge der `PRINT`-Zeilen; Ergebnisse besser holen. Ein exportiertes
  Programm nimmt Python nicht mit, der Rechner braucht es.
  `pythonStarten` setzt `PYTHONHOME` -- ein später gestartetes Python erbt
  das.
* Geprüft unter Windows mit Python 3.12 und 3.14 (numpy, PySide6). Unter
  Linux und macOS lädt `dhrt` fremde Bibliotheken dafür global (sonst
  fänden Pythons Erweiterungen dessen Funktionen nicht); dort braucht das
  Python eine gemeinsame Bibliothek (`libpython3.X.so`, unter Linux etwa
  aus dem Paket `libpython3-dev`).

## Variable Argumentzahl

`...` am Ende der Parameter nimmt beliebig viele weitere Werte, wie bei
`printf` in C:

```basic
DECLARE FUNCTION sprintf LIB "msvcrt|c" (ziel AS BUFFER, format AS TEXT, ...) AS LONG

DIM b AS BUFFER
b = BUFFER_NEW(128)
sprintf(b, "%d Drachen, %.1f Meter, %s", 3, 4.5, "feuerrot")
PRINT TEXT_AUS_ZEIGER$(BUFFER_ZEIGER(b))      ' 3 Drachen, 4.5 Meter, feuerrot
```

* **Der Typ der weiteren Werte kommt aus dem Wert:** INTEGER und BOOLEAN als
  ganze Zahl (64 Bit -- für `%d` wie für `%lld`), FLOAT als `double`, ein Text
  als `const char*` (UTF-8, kopiert), ein BUFFER als Zeiger auf seine Bytes,
  `NIL` als Nullzeiger (der Abschluss von `g_object_set`). Mehr kennt C
  hinter `...` ohnehin nicht -- kleinere Zahlen und `float` werden dort zu
  `int` bzw. `double`.
* Vor `...` steht mindestens ein fester Parameter; `--check` zählt die festen.
* Was die Funktion mit den Werten macht, entscheidet allein ihr Format --
  ein `%s` für eine Zahl ist einer der Abstürze von oben.
* **Unter Windows** liegt die `printf`-Familie in `msvcrt`, nicht in der
  C-Bibliothek `ucrtbase` -- daher `LIB "msvcrt|c"`.
* Jedes System ruft solche Funktionen etwas anders (unter Linux auf x86-64
  setzt ein kleines Sprungbrett das Register `al`, auf Apple-ARM liegen die
  weiteren Werte auf dem Stapel) -- das übernimmt dhrt.

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
  `SUB(...)` nichts. Ein Struct als Wert geht in beide Richtungen, siehe
  unten.
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

### Structs als Wert im Rückruf

Übergibt die Bibliothek einen Struct als Wert (C: `int f(Punkt p)`) oder
erwartet sie einen zurück, steht er im Rückruf genauso wie in der
`DECLARE`-Zeile -- mit `BYVAL` als Parameter, mit seinem Namen als
Rückgabe:

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

* **Die Funktion bekommt eine Kopie** in einem BUFFER der Größe des Structs
  -- was sie daran ändert, sieht die Bibliothek nicht.
* **Zurück gibt sie einen Struct** (einen BUFFER, mindestens so lang wie
  er); ein zu kurzer Puffer oder ein Wert anderer Art ist ein Fehler, der
  wie jeder Fehler im Rückruf beim Aufruf der Bibliothek ankommt.
* **Ohne `BYVAL` ist es ein Fehler:** übergibt die Bibliothek einen
  *Zeiger* auf einen Struct (C: `Punkt*`), heißt der Parameter `ZEIGER`, und
  `BUFFER_AUS_ZEIGER(z, SIZEOF(Punkt))` liest ihn.
* Wie der Struct reist, entscheidet wieder das System -- dieselben Regeln
  wie oben, nur in Gegenrichtung: was in Registern ankommt, setzt der
  Einstieg wieder zusammen.

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

* Ein Struct als Wert hinter `...` (siehe
  [Structs als Wert](#structs-als-wert)); Bitfelder mit `PACK` oder mit
  0 Bits (siehe [Bitfelder](#bitfelder)).
* C++-Namen, COM, `va_list`-Funktionen (`vprintf`). Eine C++-Bibliothek wie
  Qt geht über einen Umweg mit C-Schnittstelle -- etwa
  [Python einbetten](#python-einbetten) mit PySide6.
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
