# Entwurf: Fremde Bibliotheken aufrufen (`DECLARE … LIB`)

Stand 2026-10-05. Punkt „Kein Aufruf fremder DLLs/.so“ aus
[entwurf-anwendungen-und-tempo.md](entwurf-anwendungen-und-tempo.md).
**Stufe 1 ist gebaut (2026-10-05), Stufen 2 und 3 am 2026-10-06, die Struct-Lage am selben Tag** -- Handbuch [ffi.md](ffi.md). Die
Syntax unten steht weiter in Textblöcken, weil es die Fassung des Entwurfs
ist.

**Entschieden am 2026-10-05** (alle vier Fragen wie empfohlen):
`DECLARE … LIB` statt Befehlen; deutsche Typnamen (`ZEIGER`, `TEXT`,
`WTEXT`) mit den C-nahen (`PTR`, `CSTR`, `WSTR`) als zweiter Schreibweise;
Rückrufe erst bei Bedarf (Stufe 3); als Schutz der deutliche Hinweis plus
`TASK_START` für unsichere Bibliotheken, kein Schalter zum Verbieten.

## Worum es geht

Drachenhauch bringt viel selbst mit (Grafik, Ton, Datenbank, Netz, PDF,
Hardware). Was fehlt, liegt oft schon als C-Bibliothek auf dem Rechner: eine
Funktion der Windows-API, die es nicht als Befehl gibt, die Bibliothek eines
Messgeräts, eine Spiel-Bibliothek, `libm`. Heute bleibt dafür nur ein eigener
Prozess (`SHELL`, `PROCESS_START`) oder ein neuer Befehl in der Laufzeit.

Ziel:

```text
DECLARE FUNCTION MessageBoxW LIB "user32" (fenster AS ZEIGER, text AS WTEXT, titel AS WTEXT, art AS LONG) AS LONG
DECLARE FUNCTION strlen LIB "c" (s AS TEXT) AS ZEIGER

PRINT MessageBoxW(0, "Hallo", "Drachenhauch", 0)
PRINT strlen("Drachenhauch")       ' 12
```

## Vorbilder

| | Typen | Fehler fallen auf |
|---|---|---|
| Python `ctypes` | zur Laufzeit (`argtypes`, `restype`), sonst geraten | beim Aufruf, oft als Absturz |
| Python `cffi` | aus einer C-Deklaration im Text | beim Laden |
| VB6/VBA, FreeBASIC, PureBasic | `Declare Function … Lib "…"` im Quelltext | beim Übersetzen |

Drachenhauch ist streng getypt und prüft vor dem Lauf (`dhrt --check`).
Darum der BASIC-Weg: **eine Deklaration im Quelltext, geprüft beim
Übersetzen.**

## Entscheidungen

Zu jeder Frage eine Empfehlung; die mit „offen“ markierten entscheidest du.

### 1. Syntax: `DECLARE` statt Befehlen (offen)

```text
DECLARE FUNCTION name LIB "bibliothek" [ALIAS "c_name"] (parameter) AS typ
DECLARE SUB name LIB "bibliothek" [ALIAS "c_name"] (parameter)
```

* `DECLARE`, `LIB` und `ALIAS` sind **kontextuelle** Wörter wie `each` und
  `do`: nur am Anfang einer Anweisung bzw. an dieser Stelle. Im Bestand heißt
  nichts so (nachgesehen), aber kontextuell bricht auch fremden Code nicht.
* `ALIAS` für C-Namen, die als Drachenhauch-Name nicht taugen oder anders
  heißen sollen (`ALIAS "MessageBoxW"`).
* Danach ist der Name eine gewöhnliche Funktion: Vervollständigung, Hover
  (mit der Deklaration als Signatur), `--check` zählt die Argumente und
  prüft die Typen, Zur Definition springt zur `DECLARE`-Zeile.

**Die Alternative** wären Befehle wie in ctypes:

```text
DIM lib AS INTEGER : lib = LIB_LOAD("user32")
PRINT LIB_CALL(lib, "MessageBoxW", "p,w,w,i:i", 0, "Hallo", "Titel", 0)
```

Ohne neue Syntax, aber die Typen stünden in einer Zeichenkette, die erst
beim Aufruf gelesen wird -- genau die Fehlerquelle, die `--check` sonst
abfängt.
**Empfehlung: `DECLARE`.**

### 2. Typen (offen: die Namen)

Die Drachenhauch-Typen sagen nicht, wie breit eine Zahl in C ist. Darum eigene
Wörter, **nur in der Parameterliste einer `DECLARE`-Zeile** (kein neuer Typ
für `DIM`):

| in `DECLARE` | in C | in Drachenhauch |
|---|---|---|
| `BYTE` / `UBYTE` | `int8_t` / `uint8_t` | INTEGER |
| `SHORT` / `USHORT` | `int16_t` / `uint16_t` | INTEGER |
| `LONG` / `ULONG` | `int32_t` / `uint32_t` (auch `int`, Windows `DWORD`) | INTEGER |
| `INTEGER` | `int64_t` | INTEGER |
| `ZEIGER` | `void*`, `size_t`, `HWND` … (Zeigerbreite) | INTEGER |
| `SINGLE` | `float` | FLOAT |
| `FLOAT` | `double` | FLOAT |
| `BOOLEAN` | `int` (0/1) | BOOLEAN |
| `TEXT` | `const char*` (UTF-8, mit Null am Ende) | STRING |
| `WTEXT` | `const wchar_t*` unter Windows (UTF-16) | STRING |
| `BUFFER` | `void*` auf die Bytes des Puffers | BUFFER |

* **Ein Wert, der nicht passt, ist ein Fehler** -- `LONG` mit 2^40 bricht
  ab, statt still abgeschnitten zu werden.
* `TEXT`/`WTEXT` werden für den Aufruf kopiert. Ein Nullzeichen mitten im
  Text ist ein Fehler (C sähe nur den Anfang).
* **`BYREF`** vor einem Zahlentyp übergibt einen Zeiger auf eine Kopie und
  schreibt sie nach dem Aufruf zurück -- für Ausgabeparameter
  (`BYREF breite AS LONG`). Das Wort gibt es schon.
* **Rückgabe:** Zahlentypen, `BOOLEAN`, `ZEIGER`, `TEXT`/`WTEXT` (aus dem
  Zeiger kopiert; freigegeben wird nichts -- braucht die Bibliothek das, gibt
  man `ZEIGER` zurück und ruft ihre eigene Freigabe).
* Dazu zwei neue Befehle für Zeiger, die eine Bibliothek liefert (beide
  kopieren, beide gefährlich, beide mit diesem Hinweis in der Doku):

  ```text
  t$ = TEXT_AUS_ZEIGER$(z [, wide])
  b = BUFFER_AUS_ZEIGER(z, laenge)
  ```

Offen sind nur die **Namen**: deutsch (`ZEIGER`, `TEXT`, `WTEXT`) passend zum
Rest, oder C-nah (`PTR`, `CSTR`, `WSTR`), die jeder aus anderen BASICs kennt.
**Empfehlung: deutsch, die C-nahen Namen als zweite Schreibweise.**

### 3. Laden

* Über die Crate `libloading` -- klein, verbreitet, reines Rust; sie steht
  schon in `Cargo.lock` (bisher nur für Bau-Werkzeuge), käme also neu in
  die Laufzeit.
* **Der Name ist plattformneutral:** `"user32"` → `user32.dll`;
  `"sqlite3"` → `sqlite3.dll` / `libsqlite3.so` / `libsqlite3.dylib`; `"c"`
  ist die C-Bibliothek des Systems. Ein Name mit Endung oder Pfad wird
  wörtlich genommen.
* **Gesucht wird** neben dem Programm, neben der Exe (für Exporte), dann wo
  das System sucht.
* **Geladen wird beim ersten Aufruf**, nicht beim Start -- ein Programm, das
  den Zweig nie nimmt, läuft auch ohne die Bibliothek. Fehlt sie oder die
  Funktion darin, ist das ein gewöhnlicher Laufzeitfehler mit den versuchten
  Dateinamen, abfangbar mit `CATCH`.

### 4. Aufruf: Cranelift statt libffi

Ein Aufruf mit beliebiger Signatur braucht Maschinencode, der die Argumente
nach der Aufrufkonvention des Systems in Register und Stapel legt. Dafür gibt
es libffi (eine C-Bibliothek, unter Windows mühsam zu bauen) -- oder
**Cranelift, das für den Maschinencode ohnehin in dhrt steckt**: je
Signatur einmal ein kleiner Übergang, dann ein direkter Aufruf.

* Aufrufkonvention C (unter Windows x64 gibt es ohnehin nur eine).
* Neues Feature `ffi`, hängt an `jit`; alle drei Laufzeiten (voll, Konsole,
  Spiel) haben `jit` und bekommen `ffi` mit.
* Im Browser (WASM) gibt es keine fremden Bibliotheken: `DECLARE … LIB` ist
  dort ein Fehler mit diesem Satz.
* **Nicht in der ersten Fassung:** Funktionen mit variabler Argumentzahl
  (`printf`), Structs **als Wert** (nur über Zeiger), C++-Namen, COM.

### 5. Structs über `BUFFER`

Ein C-Struct ist ein Stück Speicher. Das gibt es schon: `BUFFER_NEW` plus
`BUFFER_GET_I32`/`BUFFER_SET_F64` und Co. -- übergeben als `BUFFER`:

```text
DECLARE SUB GetSystemTime LIB "kernel32" (zeit AS BUFFER)
DIM st AS BUFFER : st = BUFFER_NEW(16)          ' SYSTEMTIME: acht WORD
GetSystemTime(st)
PRINT BUFFER_GET_U16(st, 0)                     ' das Jahr
```

Die Lage der Felder (Ausrichtung!) rechnet man selbst. Ein eigenes
`STRUCT … LAYOUT C` wäre bequemer, aber eine eigene Baustelle -- erst, wenn
sich zeigt, dass man es oft braucht. *(Gebaut am 2026-10-06, siehe „Stand
Struct-Lage“ unten.)*

### 6. Rückrufe (offen: ob in der ersten Fassung)

Manche Bibliotheken rufen zurück (`qsort` mit Vergleich, `EnumWindows`).
Dafür müsste Cranelift einen Übergang erzeugen, der aus C in die VM springt,
nur auf demselben Faden, und der so lange lebt, wie die Bibliothek ihn
halten kann. Das ist die schwierigste Stufe. **Empfehlung: erst Stufe 3**,
wenn ein echter Bedarf da ist.

### 7. Sicherheit -- ehrlich

* **Ein Absturz in fremdem Code ist nicht abzufangen.** Ein falscher Zeiger
  beendet den ganzen Prozess, ohne `CATCH`, ohne Zeilennummer; im
  schlimmsten Fall schreibt er still in fremden Speicher. Das steht so in der
  Doku, oben.
* **Ausweg für unsichere Bibliotheken** ist schon da: den Aufruf als
  Auftrag mit `TASK_START` in einem eigenen Prozess laufen lassen -- stürzt
  der ab, lebt das Hauptprogramm weiter. Wie es den Absturz gemeldet
  bekommt, ist in Stufe 1 nachzumessen; einen eigenen Mechanismus braucht
  es dafür nicht.
* **Pakete:** ein Paket mit `DECLARE … LIB` ruft fremden Maschinencode -- das
  ist dasselbe Vertrauen wie heute schon (`SHELL`, Dateien, Netz).
  `dhrt paket liste` könnte es dazusagen.
* Der Maschinencode des JIT nimmt Aufrufe fremder Funktionen zunächst nicht
  in seine Bereiche (sie bleiben in der VM); das ist eine Leistungsfrage, keine
  der Sicherheit.

### 8. Export

* `dhrt --export` kopiert eine Bibliothek, die beim Exportieren **neben dem
  Programm** liegt, mit; Systembibliotheken (`user32`, `c`) natürlich nicht.
  Was er nicht findet, sagt er als Hinweis.
* `--export --schlank` sieht `DECLARE … LIB` und nimmt eine Laufzeit mit
  `ffi` -- da alle drei es haben, ändert sich nichts.

## Stufen

1. **Zahlen, Zeiger, Text, `BUFFER`, `BYREF`.** Prüfung gegen Bibliotheken,
   die überall da sind: unter Windows `kernel32` (`GetCurrentProcessId`,
   `MulDiv`, `lstrlenW`, `GetTickCount64`), `user32` (`GetSystemMetrics`); auf
   Linux/macOS die libc (`strlen`, `abs`, `getpid`) und `libm` (`pow`,
   `hypot` für FLOAT). Dazu die Fehlerfälle: Bibliothek fehlt, Funktion
   fehlt, Wert passt nicht in `LONG`, Nullzeichen im Text, WASM. Gegenprobe je
   Typ: ein verfälschter Übergang fällt in seinem Fall.
2. **Structs über `BUFFER`** samt den beiden Zeiger-Befehlen;
   geprüft mit `GetSystemTime` bzw. `gettimeofday`/`uname`. Export der
   Bibliothek neben dem Programm.
3. **Rückrufe**, wenn gewollt; geprüft mit `qsort` und `EnumWindows`.

Zu jeder Stufe ein Kapitel im Lehrbuch und ein Beispiel.

**Stand Stufe 1 (2026-10-05):** wie oben, mit drei Ergänzungen aus dem Bau.
(1) Der Lexer schreibt Bezeichner klein, in einer Bibliothek zählt aber
Groß/klein -- das Token trägt darum seine Schreibweise mit (`Token::orig`),
und ohne `ALIAS` gilt sie. (2) Gemessen, wie ein Absturz im Auftrag
ankommt: vorher als "unverstaendliche Antwort des Auftrags" mit leerem
Text, jetzt als "der Auftrag ist abgestuerzt (Rueckgabe N)" bzw.
"(Signal N)"; das Hauptprogramm lief in beiden Fällen weiter. (3) `WTEXT`
ist außerhalb von Windows `wchar_t` mit 32 Bit (UTF-32). Zwei Gegenproben
am Übergang selbst (Erweiterung der Rückgabe, SINGLE als FLOAT geladen)
fallen unter x86-64 nicht, weil das Register die unteren Bits ohnehin
richtig hält; die Prüfung sitzt darum an den Stellen davor und danach
(`zahl_platz`, `platz_wert`) und in den Rust-Tests mit 8/16-Bit-Werten.

**Stand Stufe 2 (2026-10-06):** `TEXT_AUS_ZEIGER$` und
`BUFFER_AUS_ZEIGER` wie oben, dazu ein dritter Befehl, den der Bau gezeigt
hat: `BUFFER_ZEIGER(puffer)`. Ohne ihn kann ein Struct kein Feld haben, das
auf einen anderen Puffer zeigt -- und mit ihm liest `TEXT_AUS_ZEIGER$` auch
ein Textfeld mitten in einem Struct (`uname`). Er gilt, solange der Puffer
seine Groesse behaelt (`BUFFER_RESIZE` legt die Bytes neu an). Alle drei
liegen in Befehlsfamilie 28 und duerfen in Maschinencode-Bereiche (sie rufen
keinen fremden Code). Geprueft mit `GetCommandLineW/A`, `_strdup`/`_wcsdup`
+ `free`, `GlobalMemoryStatusEx` (Groessenfeld vorher setzen),
`GetSystemTimeAsFileTime` -> `FileTimeToSystemTime` unter Windows und
`strdup`/`wcsdup`, `gettimeofday` gegen `time`, `uname` auf Linux/macOS.
**Export:** die Bibliotheken stehen als Signatur in den Konstanten des
uebersetzten Programms -- nur gerufene, eine bloss deklarierte kommt nicht
mit. Kopiert wird die Datei unter dem Namen des exportierenden Systems
(`dateinamen`); "c"/"m" nie, alles andere, was nicht daneben liegt, nennt ein
Hinweis. Unter Windows belegt mit einer umbenannten Kopie von
`ucrtbase.dll`, die die exportierte Exe aus ihrem Ordner laedt, nachdem die
Kopie neben der Quelle geloescht ist.

**Stand Stufe 3 (2026-10-06):** Rückrufe in der Schreibweise von
FreeBASIC -- der Typ des Parameters IST die Signatur des Rückrufs:
`vergleich AS FUNCTION(a AS ZEIGER, b AS ZEIGER) AS LONG` bzw. `AS SUB(...)`.
Verworfen wurde eine eigene Deklaration (`DECLARE CALLBACK name ...`, wie
VB.NETs Delegates): sie bräuchte einen neuen Knoten, eine Regel für
Namensräume und eine Reihenfolge; so bleibt alles in der einen Zeile.
Übergeben wird jede Funktion (FUNCREF, gebundene Methode, Lambda). Für jede
baut Cranelift einen Einstieg mit der C-Signatur, der die Argumente in
8-Byte-Plätze legt und `eingang(nummer, plaetze, rueck)` ruft; die VM kennt
ffi.rs über einen Zeiger, der vor jedem `__ffi` gesetzt wird (wie
`Kontext::vm` im Maschinencode). Drei Regeln, die der Bau festgelegt hat:
(1) Ein Fehler im Rückruf darf nicht durch die C-Rahmen laufen (ein Panic
durch `extern "C"` bricht ab) -- er wird gemerkt, weitere Rückrufe liefern
0, und der Aufruf meldet ihn nach der Rückkehr. (2) Ein Rückruf aus einem
fremden Faden wird nicht ausgeführt, er liefert 0 und meldet sich beim
nächsten Aufruf -- geprüft mit `CreateThread` bzw. `pthread_create`.
(3) Einstiege bleiben bis zum Ende, je (Signatur, Funktion) einer -- eine
Bibliothek darf sie behalten. Geprüft mit `qsort` (FUNCREF, Lambda, ein
Rückruf, der selbst die C-Bibliothek ruft) und `EnumWindows` (mit
gebundener Methode). Gegenproben: ohne die Antwort des Rückrufs und ohne
das Melden des Fehlers fallen die vier Fälle; der Einstieg selbst hat einen
Rust-Test mit allen Breiten.

**Stand Struct-Lage (2026-10-06):** `STRUCT name LAYOUT C [PACK n]` mit
Feldern `name[ [n] ] AS typ` (DIM davor erlaubt). Typen sind die der
DECLARE-Zeile ohne `BUFFER`, dazu `TEXT * n`/`WTEXT * n` und ein anderer
solcher Struct (beliebige Reihenfolge, kein Kreis). **Zur Laufzeit gibt es
keinen neuen Werttyp:** eine Variable dieses Typs ist ein BUFFER in der
Größe des Structs (`DIM` legt ihn an, voller Nullen); der Compiler kennt die
Lage (`cstruct::lagen_rechnen`, Regeln von C, `PACK` wie `#pragma pack`)
und übersetzt `st.feld`, `st.a[i].b` und Zuweisungen daran in die internen
Befehle `__struct_get`/`__struct_set` mit fester Stelle; ein Laufzeit-Index
geht über `__struct_index` (Grenze). Lesen und Schreiben prüfen wie die
DECLARE-Zeile (`zahl_platz`/`platz_wert`), dazu die Länge des Puffers. Der
Typname steht als angesagter Typ der Variable (`angesagter_typ`), die VM
reicht ihn durch wie jeden unbekannten. `SIZEOF(typ)`/`OFFSETOF(struct,
feld)` werden beim Übersetzen zu Zahlen. Ein DECLARE-Parameter `AS name`
geht als BUFFER. **Nicht** (noch; seit 2026-10-07 doch, siehe unten): Struct als Feld einer Klasse, `ARRAY OF`
Struct und `DIM x[n] AS` Struct (ein Feld von Structs ist ein Struct mit
einem Feld davon), Bitfelder, Struct als Wert übergeben. Das Modul braucht
das Feature `ffi` nicht -- es fasst nur den eigenen Puffer an und taugt so
auch für Dateiformate. Geprüft in `tests/pruef/ffi_struct.dhtest` (10) und
mit Rust-Tests der Lage; Gegenproben ohne Ausrichtung bzw. ohne
Laufzeit-Index lassen vier Fälle fallen.

**Stand GTK (2026-10-06):** Ein Versuch mit GTK 3 (das von Inkscape unter
Windows) trug auf Anhieb: Fenster, Box, Eingabefeld, Knöpfe, Signale über
`g_signal_connect_data` und ein Zeitgeber (`g_timeout_add`), alles mit Stufe
1 und 3. Drei Unbequemlichkeiten sind behoben: (1) **mehrere Namen in
`LIB`**, durch `|` getrennt (`dateinamen` zerlegt; der erste, der sich laden
lässt, gilt), (2) **`LIB name` mit einer CONST** (der Parser schreibt den
Namen mit dem Vorsatz `LIB_CONST`, der Compiler setzt den Text aus
`konst_werte` ein, `namensraum.rs` benennt die CONST mit um), (3) **unter
Windows sucht ein voller Pfad die Abhängigkeiten der DLL in ihrem Ordner**
(`LOAD_WITH_ALTERED_SEARCH_PATH`; der Pfad braucht dafür Rückstriche -- mit
Schrägstrichen schlug es fehl). Dazu sagt die Meldung jetzt, wenn eine Datei
da ist, sich aber nicht laden lässt, samt Grund des Systems. Beispiel
`examples/207_gtk.dh` (mit `--probe` drückt es sich selbst). Offen bleibt
die variable Argumentzahl (`g_object_set`).

**Stand variable Argumentzahl (2026-10-06):** `...` als letzter Parameter
(`params` = `("...", "*")`, Signatur `*`). Cranelift kennt keine variadischen
Aufrufe -- jeder Aufruf mit weiteren Werten bekommt eine gewöhnliche
Signatur, die das System genauso aufruft (`variadisch_rufen`): Windows x64
nimmt Kommazahlen als Bitmuster in Ganzzahl-Plätzen (die gerufene Funktion
liest aus der Registersicherung), Linux ARM gewöhnlich, Linux/macOS x86-64
gewöhnlich über ein Sprungbrett in `global_asm!`, das `al = 8` setzt und zum
Ziel springt (`sprungbrett::ZIEL`), Apple-ARM füllt die acht
Ganzzahl-Register mit Nullen auf, damit die weiteren Werte auf dem Stapel
landen. Der Typ der weiteren Werte kommt aus dem Wert. Geprüft unter
Windows (sprintf aus msvcrt mit elf gemischten Werten, nur Kommazahlen,
sscanf, wsprintfA) -- Linux x86-64 und macOS ARM nur durch die CI; Linux
ARM nicht. Gegenprobe: Kommazahlen unter Windows in Gleitkomma-Registern
lassen den Fall mit zehn Kommazahlen fallen. Das GTK-Beispiel setzt jetzt
einen Tooltip mit `g_object_set`.

**Stand Structs als Wert (2026-10-06):** `BYVAL p AS Punkt` übergibt den
Struct als Wert (ohne `BYVAL` bleibt es der Zeiger), eine Rückgabe
`AS Punkt` liefert ihn als Wert. Die Signatur trägt die Lage mit jeder Zahl
samt Stelle (`cstruct::wert_text`); `ffi::plan` zerlegt sie nach der Regel
des Systems in Stellen des Übergangs: Windows x64 (1/2/4/8 Bytes als
Ganzzahl, sonst Zeiger auf eine Kopie, Rückgabe RAX oder versteckte Adresse),
System V (Achtergruppen INTEGER/SSE, MEMORY über Cranelifts
`StructArgument`, mit Zählung der freien Register), AAPCS64 (HFA in
V-Registern, bis 16 Bytes in X-Registern, sonst Zeiger, Rückgabe über X8 mit
`StructReturn`). Die Rust-Tests rufen `extern "C"`-Funktionen mit
`#[repr(C)]`-Structs -- Rust hält sich an die C-Konvention, darum sind sie
auf jedem CI-System der Vergleich --, dazu `tests/pruef/ffi_struct_wert.dhtest`
mit `div`/`lldiv`, `csqrt`/`cabs` (komplexe Zahlen reisen wie ein Struct aus
zwei Kommazahlen), `PtInRect` und CoreGraphics. Gegenprobe: unter Windows
jeden Struct als Zeiger -- drei Rust-Tests und zwei Fälle fallen. Nicht:
Struct als Wert in Rückrufen und hinter `...`, eine HFA ohne freie
V-Register.

**Stand Struct als Wert im Rückruf (2026-10-07):** `FUNCTION(BYVAL p AS
Punkt) AS Punkt` im Rückruf. Der Parser schreibt `{name}`, der Compiler
setzt die Lage ein (`r` + Rückgabe + Parameter, ein Struct als `{lage}`,
`ffi::rueckruf_lesen`). Der Einstieg (`uebergang::Bauer::rueckruf_plan`)
liest `ffi::plan` in Gegenrichtung: was in Registern ankommt (`Ueber`), legt
er in einen Bereich je Struct und reicht `eingang` einen Zeiger darauf;
kam der Struct als Zeiger (`Platz`, Windows und ARM über 16 Bytes) oder auf
dem Stapel (`Kopie`, Cranelift gibt dem Gerufenen bei `StructArgument` die
Adresse), steht dieser Zeiger im Platz. Die Rückgabe legt `eingang` in den
Rückgabe-Bereich; in Teilen (`Teile`) geht sie in die Register, über die
versteckte Adresse (`StructReturn`) wird sie Byte für Byte in Struct-Länge
dorthin kopiert -- die Adresse selbst gibt Cranelift zurück. Geprüft wie
bei den Aufrufen: Rust ruft den Einstieg wie eine C-Funktion mit
`#[repr(C)]`-Structs (`structs_im_rueckruf`: 3, 8, 12, 16, 24 und 32 Bytes,
volle Register davor), dazu unter Windows `CallWindowProcW` mit einem
Struct im Register und einem als Zeiger. Gegenprobe: die Teile um vier
Bytes verschoben abgelegt -- der Rust-Test und der Windows-Fall fallen.

**Stand Bitfelder (2026-10-07):** `a AS LONG : 3` (wie C) oder `a : 3 AS
LONG` (wie FreeBASIC); der Parser haengt die Breite an den Typ (`long:3`),
ein Doppelpunkt ohne Zahl dahinter trennt weiter Anweisungen. Die Lage
rechnet `cstruct::lagen_rechnen` nach der Regel des Systems
(`cstruct::Bitregel`): **MSVC** (Windows) teilt eine Einheit nur bei gleich
grossem Typ und solange die Bits passen, sonst eine neue Einheit an der
naechsten Ausrichtung ihres Typs; **GCC/Clang** (Linux, macOS, auch ARM)
setzen an die naechste freie Bitstelle, solange keine Grenze des Typs
ueberschritten wird -- auch in die Einheit eines gewoehnlichen Feldes davor.
Beide Regeln standen nicht im Gedaechtnis, sondern wurden mit
`clang --target=x86_64-pc-windows-msvc|x86_64-linux-gnu|aarch64-apple-darwin
-Xclang -fdump-record-layouts-simple` an elf Deklarationen nachgesehen; die
Rust-Tests rechnen beide auf jedem System. Ein Feld traegt `bits`/`bit`,
`__struct_get/_set` bekommen `l3@5` als Art (Breite, unterstes Bit) und
lesen/schreiben die Einheit mit Maske; ein Wert ausserhalb der Breite ist
ein Fehler, mit Vorzeichen wird erweitert. Als Wert (`wert_text`) ist eine
Einheit eine Ganzzahl, je Einheit einmal. **Bewusst nicht:** Bitfelder mit
`PACK` (GCC packt dann bitweise ueber Grenzen, MSVC nicht -- ein drittes
Regelwerk), Breite 0 und unbenannte Felder. Geprueft an `BuildCommDCBA`
(das DCB traegt 13 Bitfelder), dazu je System die Lage von `char c; int
a : 4`. Gegenprobe: Windows nach GCC-Regel -- der Windows-Fall faellt;
ohne die Grenzpruefung von GCC -- der Rust-Test faellt.

**Stand va_list (2026-10-07):** Typwort `VALIST` (`VA_LIST`, Zeichen `a`),
das Argument ist ein Tupel, ein Feld, ein Wert oder NIL. Die Werte kommen
wie hinter `...` in 8-Byte-Plaetze; davor setzt `ffi::valist_kopf` je
System (`ValistForm`), was C als `va_list` erwartet: **Windows und
Apple-ARM** sind `char*` -- der Zeiger auf die Plaetze genuegt; **System V
x86-64** ist ein Feld aus einem `__va_list_tag` (`gp_offset`, `fp_offset`,
`overflow_arg_area`, `reg_save_area`), als Parameter ein Zeiger darauf: mit
`gp_offset` 48 und `fp_offset` 176 gelten alle Register als verbraucht, und
`va_arg` liest alles aus dem Ueberlaufbereich, also den Plaetzen
(nachgesehen in der WSL mit gcc gegen die glibc, bevor es gebaut wurde);
**AAPCS64 ausser Apple** ist ein Struct mit 32 Bytes (`__stack`, `__gr_top`,
`__vr_top`, `__gr_offs`, `__vr_offs`), der als Wert ueber 16 Bytes ohnehin
als Zeiger auf eine Kopie reist -- mit beiden Offsets 0 liest `va_arg` ab
`__stack`. Linux-ARM ist nach dem ABI gebaut, aber ungeprueft (die CI hat
kein Linux-ARM). Ein Rust-Test ruft `vsprintf` der echten C-Bibliothek
(in der CI auf allen drei Systemen), dazu `tests/pruef/ffi.dhtest` mit
`vsprintf` und `wvsprintfA`. Gegenprobe: Kommazahlen als SINGLE abgelegt --
Rust-Test und Fall fallen. Nicht: `VALIST` in einem Rueckruf (der Rueckruf
muesste `va_arg` nachbauen), als Rueckgabe oder mit BYREF.

**Stand Maschinencode (2026-10-07):** `__ffi` war in `befehl_im_bereich`
pauschal gesperrt. Die Sperre ist noetig, sobald fremder Code
Drachenhauch-Code rufen KANN -- und das kann nicht nur ein Aufruf mit
Rueckruf-Parameter, sondern jeder, sobald das Programm einmal einen Rueckruf
vergeben hat (die Bibliothek merkt ihn sich; GTK ruft ihn aus `gtk_main`).
Darum gilt: `__ffi` im Bereich nur, solange **kein Rueckruf vergeben** ist
(`ffi::rueckrufe_vergeben`, gefragt beim Bauen) und die Signatur **keinen
Rueckruf nimmt** (`ffi::ohne_drachenhauch_code`, gefragt je Aufruf -- sonst
steigt der Bereich vor dem Befehl aus). Belegt mit einem Fall, in dem
`CallWindowProcW(proc AS ZEIGER, ...)` einen per `EncodePointer`
gemerkten Rueckruf ruft, der eine Globale der Schleife aendert: ohne die
Sperre rechnet der Maschinencode 120 statt 1020.
Gemessen brachte das Erlauben allein nichts (abs je Million: 198 ms gegen
196 ms in der VM): mit `__ffi` fiel der ganze Bereich in den Wertemodus, und
der Aufrufweg selbst kostete ~200 ns. Zwei Schritte danach: (1) der
Aufrufweg -- die Signatur wird an ihrem `Rc` erkannt statt je Aufruf
gehasht (`aufruf_wert`, Zuletzt-Liste), Plaetze bis 16 Argumente auf dem
Stapel, der Typname nur im Fehlerfall: VM 202 -> 116 ms; (2) **getypt**:
eine Signatur nur aus Zahlen ist im Bereich `Art::S` (Zeiger auf die
Konstante), der Aufruf geht ueber den Helfer `ffi_zahlen` ->
`ffi::zahlen_rufen` (dieselben Pruefungen und derselbe Uebergang ueber
`rufen_mit`); jeder Fehler kommt dort VOR dem eigentlichen Aufruf, also
darf der Bereich davor aussteigen, und die VM ruft und meldet. Nicht in
uebersetzten Funktionen (die VM rechnet dort bei einem Ausstieg die ganze
Funktion nach -- ein Fremdaufruf hat Nebenwirkungen). Ergebnis 201 -> 54 ms
mit Maschinencode (`tools/tempo/ffi.dh`, best of 5, gegen den Bau davor;
die uebrigen Messungen unveraendert). Fund dabei: der Ausstieg mitten im
Bereich schrieb nur Zahlen und Objekte in den Stapel der VM zurueck -- mit
der Signatur auf dem Stapel las die VM Muell als Zeiger und stuerzte ab.

## Die Fragen dazu (entschieden, siehe oben)

1. **`DECLARE … LIB`** (empfohlen) oder Befehle wie ctypes?
2. **Typnamen:** deutsch (`ZEIGER`, `TEXT`, `WTEXT`) mit C-nahen als zweiter
   Schreibweise (empfohlen), oder nur C-nah?
3. **Rückrufe** gleich mit, oder erst wenn es einen Bedarf gibt (empfohlen)?
4. Reicht als Schutz der Hinweis plus `TASK_START` für unsichere
   Bibliotheken (empfohlen), oder soll es einen Schalter geben, der
   `DECLARE … LIB` ganz verbietet (etwa für Unterrichtsrechner)?

**Stand Felder von Structs und Structs in Klassen (2026-10-07):**
`DIM pts[n] AS Punkt` ist EIN Puffer mit `n * SIZEOF` Bytes, die Structs
hintereinander wie in C -- als `Punkt*` geht er so an `CreatePolygonRgn`,
und `qsort` sortiert an Ort und Stelle. Intern heisst der Typ `punkt[]`;
`ARRAY OF Punkt` (Parameter, Rueckgabe, `DIM` ohne Groesse) wird dazu
(`Compiler::typ_norm`) -- sonst lehnte die VM den Puffer als "kein ARRAY" ab.
Die Groesse darf zur Laufzeit feststehen; der Index prueft gegen die Laenge
des Puffers (`__struct_index_puffer`), `LEN` zaehlt die Structs
(`__struct_anzahl`). In einer Klasse traegt das Feld `struct_bytes`
(`FieldInfo`/`model::FieldDecl`, auch im .dhc), und `allocate_instance`
legt je Objekt einen Puffer an; `obj.p.x`, `Self.p.x` und `p.x` in einer
Methode gehen ueber `struct_objektfeld` (Klasse aus `typ_von`, Feldtyp aus
`feld_typ`, geladen mit LOAD_MEMBER statt `expr`, sonst kaeme der Compiler
wieder hierher). Ein ganzes Objektfeld (`k.p`) bleibt ein gewoehnlicher
Wert. **Bewusst nicht:** mehrere Dimensionen, `FOR EACH`, ein Element als
Ganzes als Wert, ein Index direkt auf einem Aufruf (dafuer muesste er fuer
die Laenge zweimal laufen) -- je eine Meldung. Gegenproben: `struct_bytes`
beim Laden einer .dhc uebergangen -> der .dhc-Fall faellt; die Pruefung des
Index gegen den Puffer abgeschaltet -> der Index-Fall faellt; NEW ohne
Puffer -> der Klassenfall faellt.

**Stand Folgen der Felder von Structs (2026-10-07):** (1) **Mehrere
Dimensionen** wie in C: der Typ traegt die hinteren Groessen (`punkt[,4]`,
`struct_feldtyp`), sie muessen beim Uebersetzen feststehen
(`feste_groesse` ueber `falten`), die erste sagt der Puffer. Der Index wird
in `struct_ort` zu je einem Eintrag in `dyn_` je Dimension (die erste mit
der Zeilengroesse gegen den Puffer, die uebrigen gegen ihre feste Zahl) --
`struct_stelle` brauchte dafuer nichts Neues. Globale tragen den Typ schon
aus `merke_global_typ`, sonst kennte ein Zugriff aus einer SUB nur eine
Dimension. (2) **Ein Element als Ganzes** (`p = pts[2]`, `pts[2] = p`,
`linie.b = p`) ist eine KOPIE: Typzeichen `#` mit der Groesse in `zeichen`;
`cstruct::schreiben` kopiert die Quelle vor dem Borgen (`pts[1] = pts[1]`
borgte sonst denselben RefCell zweimal). (3) **An eine Bibliothek** geht
ein Element dagegen als Zeiger an seine Stelle: der Compiler legt
(Puffer, Stelle) als Tupel ab (`struct_zeiger`), `__ffi` macht bei `p`
daraus den Zeiger -- sonst schriebe `GetCursorPos(pts[2])` in eine Kopie,
ohne dass es jemand merkt. (4) **`FOR EACH`** (`foreach_struct`) liefert je
Runde eine Kopie, die Laufvariable bekommt den Struct-Typ; ein Feld der
Kopie zu schreiben ist eine Meldung (`foreach_kopien`), weil es ins Leere
ginge. Geht auch ueber ein Feld im Struct (`st.werte`, `l.ecken`), nur
ueber eine Dimension, ohne Paar-Form.
