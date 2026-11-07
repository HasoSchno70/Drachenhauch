# Fremde Bibliotheken (DECLARE ... LIB)

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Alle Stufen der FFI: Rueckrufe, Structs, Bitfelder, va_list, GTK, Python einbetten.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Fremde Bibliotheken: `DECLARE … LIB` (2026-10-05, Stufe 1)

`ffi.rs`, Entwurf `docs/entwurf-ffi.md`, Handbuch `docs/ffi.md`, Feature
`ffi` (haengt an `jit`, in allen drei Laufzeiten). Parser: `declare_lib`
(kontextuelle Woerter DECLARE/LIB/ALIAS, Typwoerter nur in dieser Zeile ueber
`ffi::typ_zeichen`, Knoten `Node::DeclareLib`). **Der Lexer schreibt
Bezeichner klein, in einer Bibliothek zaehlt Gross/klein** -- `Token::orig`
traegt die Schreibweise des Quelltexts, ohne ALIAS gilt sie. Compiler:
`register_ffi` (nach den SUB/FUNCTION-Stubs, vor den Rumpfen), ein Aufruf
wird `CALL_BUILTIN "__ffi"` mit der Signatur als erstem Wert
(`ffi::signatur_text`); mit BYREF liefert `__ffi` ein Tupel (Ergebnis,
letzter, ..., erster BYREF-Wert), `UNPACK_TUPLE` + `emit_byref_writeback`
schreiben zurueck. Die Argumentzahl ist ein Uebersetzungsfehler, ein
sicher falscher Typ eine Warnung. Laufzeit: Befehlsfamilie 28
(`BUILTIN_FAMILIEN` = 29), Bibliothek beim ERSTEN Aufruf (`libloading`,
neben Programm/Exe/System, `ffi::dateinamen`: "c"/"m" plattformneutral),
je Signatur ein Cranelift-Uebergang `fn(ziel, args: *const u64, rueck: *mut
u64)` (`uebergang::Bauer`, eigenes JITModule). Kleine Ganzzahlen tragen
sext/uext in der Signatur; ein Wert, der nicht passt, ist ein Fehler
(`zahl_platz`). Maschinencode-Bereiche nehmen `__ffi` seit 2026-10-07 auf,
**aber nur solange kein Rueckruf vergeben ist** (`ffi::rueckrufe_vergeben`
beim Bauen, `ffi::ohne_drachenhauch_code` je Aufruf): eine Bibliothek darf
einen gemerkten Rueckruf bei JEDEM Aufruf rufen, und der aendert womoeglich
eine Globale, die der Bereich haelt (Gegenprobe: 120 statt 1020). Mit nur
Zahlen geht der Aufruf getypt (`Art::S` = Zeiger auf die Signatur-Konstante,
Helfer `ffi_zahlen` -> `ffi::zahlen_rufen`), nur in Bereichen, nicht in
Funktionen. `ffi.dh` der Messbank 201 -> 54 ms. Ein Absturz im Auftrag (TASK_START) meldet jetzt
"der Auftrag ist abgestuerzt (Rueckgabe N|Signal N)" statt "unverstaendliche
Antwort". `symbole.rs` kennt die Zeile als Definition (Hover, Springen,
Vervollstaendigung). **Gegenproben:** eine Verfaelschung der Rueckgabe-
Erweiterung oder SINGLE-als-FLOAT IM Uebergang faellt unter x86-64 nicht
(das Register haelt die unteren Bits richtig) -- geprueft wird an
`zahl_platz`/`platz_wert` und in den Rust-Tests. Tests
`tests/pruef/sprache/ffi.dhtest` (39; Windows kernel32/user32, sonst die C-Bibliothek
auf allen drei Systemen), Rust-Tests in ffi.rs und symbole.rs, Beispiel
`examples/206_fremde_bibliotheken.dh`, Lehrbuch `79c_bibliotheken.js`.
**Stufe 2 (2026-10-06):** `TEXT_AUS_ZEIGER$(z [, breit])`,
`BUFFER_AUS_ZEIGER(z, n)` und `BUFFER_ZEIGER(b)` (`ffi::zeiger_befehl`,
Familie 28 neben `__ffi`; kopieren, vertrauen dem Zeiger, ohne Feature ffi
ein Fehler "ohne das Feature"). `BUFFER_ZEIGER` gilt nur, solange der Puffer
seine Groesse behaelt. **Export** (`bibliotheken_mitnehmen` in main.rs): die
Bibliotheken stehen als Signatur in den Konstanten des .dhc -- nur gerufene
kommen mit, unter dem Dateinamen DIESES Systems; "c"/"m" nie, Fehlendes als
Hinweis. **Stufe 3 (2026-10-06): Rueckrufe** in FreeBASIC-Schreibweise --
`cmp AS FUNCTION(a AS ZEIGER, b AS ZEIGER) AS LONG` bzw. `AS SUB(...)`; der
Parser macht daraus `@` + Rueckgabe + Parameter (`ffi_rueckruf`), die
Signatur `r...` (`Param::rr`). Uebergeben wird jede Funktion (FUNCREF,
gebundene Methode, Lambda); je (Signatur, Funktion) EIN Einstieg
(`uebergang::Bauer::rueckruf`, Cranelift, bleibt bis zum Ende), der ueber
`eingang` -> `Vm::wert_rufen` laeuft (VM-Zeiger per `ffi::vm_setzen` vor
jedem `__ffi`). **Ein Fehler im Rueckruf darf nicht durch die C-Rahmen**
(Panic durch `extern "C"` bricht ab): gemerkt in `RR_FEHLER`, weitere
Rueckrufe liefern 0, `rufen_mit` meldet ihn nach der Rueckkehr. **Fremder
Faden** (`HAUPTFADEN`): nicht ausgefuehrt, 0, gemeldet beim naechsten
Aufruf. Tests in `ffi.dhtest` (qsort, EnumWindows, CreateThread,
pthread_create, Parserfehler), Rust-Test `rueckruf_einstieg_legt_plaetze_ab`.
**Struct-Lage (2026-10-06):** `STRUCT name LAYOUT C [PACK n]` (Parser
`struct_layout`, Knoten `Node::StructLayout`, Lagen in `cstruct.rs` nach den
Regeln von C). Eine Variable dieses Typs ist ein BUFFER (DIM:
`__struct_neu`); der angesagte Typ ist der Struct-Name, und
`Compiler::struct_ort` macht aus `st.a[i].b` einen Zugriff mit fester Stelle
(`__struct_get`/`__struct_set`, Index ueber `__struct_index`, alle in
`cstruct::befehl`, reine Familie, ohne Feature ffi). `SIZEOF`/`OFFSETOF`
faltet `groesse_oder_lage` beim Uebersetzen (`SIZEOF(FLOAT)` faengt der
Parser ab, FLOAT ist ein Schluesselwort). DECLARE-Parameter `AS name` ->
BUFFER (`#name` vom Parser). WITH uebernimmt den Typ. **Seit 2026-10-07
auch Felder von Structs und Structs in Klassen:** `DIM pts[n] AS Punkt` =
EIN Puffer (Typ `punkt[]`; `ARRAY OF Punkt` wird ueber `typ_norm` dazu --
auch in `angesagter_typ`, Parametern und Rueckgaben, sonst lehnt die VM den
Puffer ab), Index gegen die Pufferlaenge (`__struct_index_puffer`), `LEN` =
Zahl der Structs; Klassenfeld = `FieldInfo::struct_bytes` -> eigener Puffer
je Objekt in `allocate_instance`, Zugriff ueber `struct_objektfeld`
(LOAD_MEMBER; ein ganzes Objektfeld ist ein gewoehnlicher Wert). **Folgen
(selber Tag):** mehrere Dimensionen wie in C (Typ `punkt[,4]` traegt die
hinteren, festen Groessen, `struct_feldtyp`; je Dimension ein Eintrag in
`dyn_`), ein Element als Ganzes ist eine KOPIE (Typzeichen `#`, Groesse in
`zeichen`), an eine Bibliothek geht es aber als Zeiger an seine Stelle
(Tupel (Puffer, Stelle), `struct_zeiger` -> `__ffi`), `FOR EACH` liefert
Kopien und verbietet, ein Feld der Laufvariable zu schreiben
(`foreach_kopien`). Nicht: Index direkt auf einem Aufruf (Meldung). Tests
`tests/pruef/sprache/ffi_struct.dhtest`, Rust-Tests in cstruct.rs.
**GTK (2026-10-06):** `LIB "a.dll|liba.so.0"` (mehrere Namen, `dateinamen`
zerlegt an `|`), `LIB name` mit einer CONST (Vorsatz `ffi::LIB_CONST`,
eingesetzt in `register_ffi` aus `konst_werte`, umbenannt in
`namensraum.rs`), unter Windows laedt ein voller Pfad mit
`LOAD_WITH_ALTERED_SEARCH_PATH` (`ffi::laden`; **nur mit Rueckstrichen** --
mit Schraegstrichen findet Windows die Abhaengigkeiten nicht), und eine
vorhandene, nicht ladbare Datei hat eine eigene Meldung. Beispiel
`examples/207_gtk.dh`, Faelle in `ffi.dhtest` (GTK-Faelle ueberspringen sich
ohne GTK; lokal ueber das GTK von Inkscape).
**Variable Argumentzahl (2026-10-06):** `...` als letzter DECLARE-Parameter
(Signatur `*`, `--check` zaehlt die festen). Cranelift kann keine
variadischen Aufrufe -- `ffi::variadisch_rufen` baut je Aufruf eine
gewoehnliche Signatur nach den Regeln des Systems: Windows x64 Kommazahlen als
Bitmuster in Ganzzahl-Plaetzen (`KOMMA_IN_FP_REGISTER`), System V x86-64 ueber
das Sprungbrett `sprungbrett` (`global_asm!`, setzt `al = 8`), Apple-ARM
fuellt die acht X-Register mit Nullen (Rest auf den Stapel). Typ der
weiteren Werte aus dem Wert. Unter Windows liegt `sprintf` in `msvcrt`, nicht
in ucrtbase (`LIB "msvcrt|c"`). Faelle `variadisch:` in ffi.dhtest.
**Python einbetten (2026-10-06):** `LIB "$NAME"` liest die Umgebungsvariable
NAME beim ERSTEN Aufruf (`ffi::namen_einsetzen`, auch `$A|ersatz`; fehlt
alles, nennt die Meldung die Variable; `dateinamen` laesst `$`-Teile fuer
den Export weg). Unter Linux/macOS laedt `ffi::laden` jetzt RTLD_GLOBAL --
Pythons Erweiterungen (numpy) brauchen die Zeichen von libpython, ohne
dagegen gelinkt zu sein. Die Bibliothek `examples/python/python.dh` fragt
ein Python (Argument, `DH_PYTHON`, `py -3`, `python3`, `python`) nach DLL,
`base_prefix` und `sys.path`, setzt `DH_PYTHON_DLL` + `PYTHONHOME`, startet
es und uebernimmt den Suchpfad; sie benutzt nur die stabile Schnittstelle
(`Py_CompileString`/`PyEval_EvalCode` statt `PyRun_*`, das python3.dll nicht
hat) und einen Hilfscode in Python, der nie wirft (Fehler nach `_dh_fehler`).
**Falle fuer Bibliotheken:** kurze lokale Namen (`n`, `h`, `b`) verdecken
Konstanten des importierenden Programms -- darum dort lange Namen. Beispiel
`examples/208_python.dh` (Spektrum mit numpy oder reinem Python), Tests
`tests/pruef/sprache/python_einbetten.dhtest` (uebersprungen nur ohne Python, nicht
wenn es nicht startet) und zwei `$`-Faelle in ffi.dhtest; gegen einen Bau
ohne `$NAME` fallen alle fuenf.
**Structs als Wert (2026-10-06):** `BYVAL p AS Punkt` (ohne BYVAL bleibt es
der Zeiger, wie bisher) und `DECLARE FUNCTION f(...) AS Punkt`. Der Parser
schreibt `~name`, der Compiler `x` + `cstruct::wert_text` (Lage mit allen
Zahlen samt Stelle) in die Signatur. `ffi::plan(Konvention, ...)` zerlegt je
System -- Win64 (1/2/4/8 Bytes als I64, sonst Zeiger auf eine Kopie;
Rueckgabe RAX oder versteckte Adresse), SysV (Achtergruppen INTEGER/SSE,
MEMORY = `StructArgument`, zaehlt freie Register: passt er nicht ganz, geht
er auf den Stapel und die Register bleiben fuer spaetere Argumente), Aapcs
(HFA in V-Registern, bis 16 Bytes in X-Registern, sonst Zeiger; passt ein
X-Struct nicht ganz, fuellen Nullwerte die Register; Rueckgabe ueber X8 =
`StructReturn`) -- in `Stelle`n (`Laden::Platz/Ueber/Kopie/Null/Versteckt`),
aus denen `Bauer::holen_plan` den Uebergang baut. Kopien in `Vec<u64>` (8
ausgerichtet, Laden ueber das Ende eines 12-Byte-Structs bleibt im
Speicher). Nicht: hinter `...`, HFA ohne freie V-Register (eine Meldung). Rust-Tests rufen `extern "C"`-Funktionen mit
`#[repr(C)]`-Structs auf (Rust haelt die C-Konvention -- der Vergleich auf
allen drei CI-Systemen), `plaene_je_konvention` prueft alle drei Plaene auf
jedem System; Sammlung `tests/pruef/sprache/ffi_struct_wert.dhtest` (div/lldiv,
csqrt/cabs(f), PtInRect, CoreGraphics).
**Im Rueckruf (2026-10-07):** `FUNCTION(BYVAL p AS Punkt) AS Punkt`; Parser
`{name}`, Signatur `{lage}` (`ffi::rueckruf_lesen` -> `RrTeil`). Der
Einstieg `Bauer::rueckruf_plan` liest `ffi::plan` in GEGENrichtung: `Ueber`
-> Bereich je Struct, im Platz ein Zeiger darauf; `Platz`/`Kopie` -> der
ankommende Zeiger (Cranelift gibt bei `StructArgument` dem Gerufenen die
Adresse). Rueckgabe in Teilen aus dem Rueckgabe-Bereich, ueber die
versteckte Adresse Byte-genau kopiert (Cranelift gibt sie selbst zurueck).
Ohne BYVAL im Rueckruf ist ein Struct eine Meldung (`ZEIGER` +
BUFFER_AUS_ZEIGER). Rust-Test `structs_im_rueckruf` (Rust ruft den Einstieg
als `extern "C" fn` mit `#[repr(C)]`-Structs), Windows-Faelle ueber
`CallWindowProcW` in `ffi_struct_wert.dhtest`.
**Bitfelder (2026-10-07):** `a AS LONG : 3` oder `a : 3 AS LONG`, Typ im
Baum `long:3`. Lage nach `cstruct::Bitregel` -- MSVC unter Windows (neue
Einheit bei anderer Typgroesse oder vollem Wort), GCC/Clang sonst (naechste
freie Bitstelle, keine Typgrenze ueberschreiten, auch hinter ein
gewoehnliches Feld); beide mit `clang -Xclang -fdump-record-layouts-simple`
fuer drei Ziele nachgesehen, `lagen_rechnen_mit` laesst die Rust-Tests beide
auf jedem System rechnen. `Feld::bits/bit`, die Art fuer
`__struct_get/_set` ist `l3@5`. Tests
`ffi_struct.dhtest` (DCB ueber `BuildCommDCBA`, Lage je System). **Mit PACK,
ohne Namen und mit 0 Bits (selber Tag):** drei Regeln (`Msvc`, `Gcc`,
`GccArm` = Linux ARM64, `bitregel()`), gemessen mit clang fuer fuenf Ziele
und dem echten gcc/`cl.exe`. GCC unter PACK Bit an Bit ueber die Typgrenze
(Lesen/Schreiben ueber `u128`, bis neun Bytes); `AS LONG : 0` beendet die
Einheit (MSVC nur hinter einem Bitfeld), `AS LONG : 3` = Fuellbits ohne
Namen (Feld mit leerem Namen, `Lage::feld` findet es nicht); wer zur
Ausrichtung des Structs zaehlt, unterscheidet die drei Regeln. Rust-Test
`pack_und_null_bits_nach_drei_regeln` (32 Structs aus clang).
**va_list (2026-10-07):** Typwort `VALIST`/`VA_LIST` (Zeichen `a`),
Argument Tupel/Feld/Wert/NIL, Werte wie hinter `...` in 8-Byte-Plaetzen;
`ffi::valist_kopf` je `ValistForm`: Windows + Apple-ARM `char*` (nur die
Plaetze), System V Kopf `{gp_offset 48, fp_offset 176, overflow = Plaetze,
reg_save}` (alle Register "verbraucht", vorher in der WSL mit gcc gegen
glibc probiert), Linux-ARM `{__stack = Plaetze, 0, 0, 0}` (32 Bytes, reist
als Zeiger -- ungeprueft). Nicht im Rueckruf/als Rueckgabe/BYREF. Tests:
Rust `valist_ueber_vsprintf` (echte C-Bibliothek, alle CI-Systeme),
`ffi.dhtest` (`vsprintf`, `wvsprintfA`).
