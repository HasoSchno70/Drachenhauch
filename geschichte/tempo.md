# Tempo der Laufzeit und Maschinencode

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Zeichenketten, Merkplaetze, Objektlage, Konstanten falten, Superinstruktionen, Rahmenstapel, Maschinencode M3 bis M5.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Zeichenketten anhaengen ist linear (2026-09-24)

`Value::Str` ist `Rc<String>` (vorher `Rc<str>`, jedes Anhaengen kopierte:
400 000 x `s = s + "x"` = 7 s, jetzt 16 ms). `x = x + e` / `x += e` auf einen
lokalen/globalen Platz kompiliert zu `ADD_STORE_LOCAL`/`ADD_STORE_GLOBAL_SLOT`
(Opcodes 120/121): `x` wird wie bisher VOR `e` geladen; haelt der Platz danach
noch DENSELBEN Text (`Rc::ptr_eq`) und ist `e` schlicht (`ist_schlicht`:
Text/Zahl/Bool/NIL), gibt er seinen Verweis ab und `Vm::addieren` haengt per
`Rc::get_mut` an. **`ist_schlicht` ist die Sicherung:** ein `OPERATOR +`, der
wirft, liesse sonst NIL in der Variable (Gegenprobe im Test). Ketten
`x = x + t1 + t2` formt der Compiler zu `x + (t1 + t2)` um -- nur wenn `x` und
`t1` sicher Text sind (`ist_text`: auch `$`-Builtins und FUNCTION `AS STRING`)
und der Rest schlicht (`plus_kette`, `add_store_ziel`). `Value::str_rc` nimmt
`impl Into<String>` -- ein fertiger `String` geht ohne Kopie hinein. **Falle:**
die Typpruefung beim Speichern (`passend!`) MUSS ein Makro bleiben; als
Funktion wurde sie nicht eingebettet und `z = z + i` war 25 % langsamer.
Tests `tests/pruef/sprache/zeichenketten_anhaengen.dhtest`, Doku
`docs/entwurf-anwendungen-und-tempo.md` (1a).

**Felder, Elemente, MAP-Werte, BYREF (2026-10-20):** `k.text = k.text + e`,
`text = text + e` (Feld von Self), `a[i] = a[i] + e`, `m["k"] = m["k"] + e`
werden `ADD_STORE_MEMBER`/`ADD_STORE_FIELD`/`ADD_STORE_INDEX` (126/125/127),
wenn Text im Spiel ist (`anhaengen_lohnt` -- `k.n = k.n + 1` bleibt ADD +
STORE, sonst verloere der Maschinencode Zahlenfelder) und das Ziel keinen
Aufruf hat (`ort_rein`, `gleicher_ort`). Anders als ADD_STORE_LOCAL wird der
Platz NIE geleert: `vm::an_ort_anhaengen` haengt direkt im Platz an, wenn er
noch denselben Text haelt (`Rc::ptr_eq`) und `e` schlicht ist -- Text +
schlicht ruft keinen Code, kann nicht scheitern, und der Platz nimmt Text ohne
Umwandlung (`text_platz_typ`). Mitglied nur ueber den Merkplatz des Befehls
(nie PROPERTY). JIT: `_MEMBER`/`_INDEX` im Wertemodus ueber `w_add_*`,
`_FIELD` bleibt in der VM (String-Felder von Self konnte er nie). BYREF ist
Copy-In/Copy-Out -- Caller-Platz und Parameter teilten den Text, also kopierte
jeder Aufruf: `LEIHE_LOCAL`/`LEIHE_GLOBAL_SLOT` (128/129) NEHMEN den Wert
(Platz = NIL bis zum Zurueckschreiben). Nur wo das niemand sieht
(`byref_leihen`): angesagter STRING, kein TRY um die Stelle (`try_gesamt`,
auch CATCH/FINALLY), Variable in keinem anderen Argument; Local immer, Globale
nur im Hauptprogramm und nur, wenn der Aufgerufene sie nicht nennt und nur
`REINE_TEXTBEFEHLE` ruft (`leih_spur`) und keine Klasse einen OPERATOR hat.
Unter dem Debugger kopiert LEIHE.

## Builtin-Aufruf mit Merkplatz (2026-09-24)

CALL_BUILTIN reichte den Namen bei JEDEM Aufruf durch bis zu 27
`try_*`-Familien, bevor die reinen Befehle in builtins.rs drankamen. Jetzt
merkt sich jede Aufrufstelle in `Instr::familie` (Cell<u8>, 1-basiert) die
Familie, die geantwortet hat, und fragt sie zuerst (`Vm::builtin_rufen` /
`Vm::familie_rufen`, Reihenfolge = Nummern, `BUILTIN_FAMILIEN` = 28). `ABS`
78 -> 39 ms je Million, `CHART_COUNT` 145 -> 39. **Regel fuer neue
Familien:** eine Familie darf nur am NAMEN absagen. Sagt sie je nach
ARGUMENT oder ZUSTAND ab, obwohl eine spaetere denselben Namen kennt, muss
der Name in `builtin_rufen` vom Merken ausgenommen werden -- wie `SORT`
(FUNCREF -> `try_array_hof`, Wahrheitswert -> reiner Befehl). Neue Familie =
neue Nummer in `familie_rufen` UND `BUILTIN_FAMILIEN` anpassen. Eine Nummer
je Befehl lohnt nicht: gemessen kostet Arm 494 des grossen `match` so viel wie
Arm 10. Tests `tests/pruef/laufzeit/builtin_familien.dhtest`.

## Speicher und kleine Wege (2026-10-21)

Gemessen je Grundoperation (ns je Schleifendurchlauf): der Maschinencode
rechnet Zahlen in 0,5-5 ns, aber alles, was Speicher anlegt, lag bei
40-140 ns. Vier Stellen: (1) **mimalloc** als globaler Speicher-Verwalter
(`main.rs`, nicht fuer emscripten) -- jeder Text ist zwei Bloecke (Rc +
String), HeapAlloc kostete je Block ~20 ns; tools/tempo text 19,6 -> 9,0 ms,
maps 79,9 -> 50,5, ffi 77,3 -> 43,4. (2) **`a = []` an ARRAY OF
INTEGER/FLOAT** bekommt in `coerce_array` auch den getypten Speicher
(Cells::Int/Float) wie `DIM a[n]` -- vorher Cells::Val, und der
Maschinencode darf ein Feld von Werten nur lesen: jede Schleife, die
hineinschrieb, lief im Wertemodus (39 -> 1 ns je `a[i] = x`). (3)
`typen::builtin_typ` kennt ASC, INSTR, INSTRREV, MAPSIZE, COUNT, ARRAY_PUSH
als INTEGER -- dann rechnet der Maschinencode mit dem Ergebnis als Zahl
weiter (ASC in der Schleife 42 -> 19 ns). Nur eintragen, was in builtins.rs
auf JEDEM Weg diesen Typ liefert (die Tafel speist auch die Typpruefung des
Compilers; Wahrheitswerte wie CONTAINS bewusst nicht). (4)
`Value::str_stueck`: MID$/LEFT$/RIGHT$/CHR$ liefern ein einzelnes
ASCII-Zeichen und "" aus einer Tafel je Faden statt neu angelegt. Teilen ist
harmlos, angehaengt wird an Ort und Stelle nur ohne zweiten Halter. Tests
`tests/pruef/sprache/array_literal.dhtest` (letzter Fall).

## Globale Felder ueber Plaetze (2026-09-24)

Bis dahin bekamen nur skalare Globals einen Platz; Felder, Maps und
Strukturen im Hauptprogramm liefen ueber ihren NAMEN (`LOAD_NAME`: Name als
String, Hash-Suche, Typ-Klon) -- ein Erbe der Python-Laufzeit, gemessen 2,2x
langsamer als ein lokales Feld. Jetzt ist `is_slot_dim` immer wahr; angelegt
wird weiter unter dem Namen, dann haengt `BIND_GLOBAL_SLOT` (Opcode 122,
`Compiler::bind_global`) denselben `Rc<RefCell<Slot>>` in den Platz. **Nach
JEDEM Anlegen neu**: `DECLARE_ARRAY_NAME` legt bei einem DIM in einer Schleife
ein NEUES Feld an. Ein leerer Platz meldet sich mit dem Namen der Variable
(`Program::global_names`, vom Compiler als `global_names` im .dhc,
`Vm::global_ungesetzt`) -- der Name steht HINTER dem alten Text, weil
`task.dhtest`/`dhrt_call.dhtest` ihn pruefen. Tests
`tests/pruef/sprache/globale_felder.dhtest`.

## Felder und PROPERTYs von Objekten (2026-09-24)

`obj.x` (auch `Self.x` -- das ist LOAD_MEMBER, nicht LOAD_FIELD; LOAD_FIELD
ist nur das nackte `x` in einer Methode) fragte bei JEDEM Zugriff die
Klassenkette ab, ob `x` eine PROPERTY ist (`std::HashSet` je Ebene), und eine
PROPERTY baute je Aufruf `format!("__get_x")` und suchte ihre Methode. Jetzt
rechnet `model::load_program` je Klasse `props_kette` (irgendeine PROPERTY in
der Kette?), `props_alle` (alle Namen) und `prop_get`/`prop_set` (Name ->
Klasse + Methode, aufgeloest wie `resolve_method`: die Klasse selbst zuerst);
`Vm::property_methode` beantwortet beides in einem Schritt, `is_property` ist
weg. `Self.n = Self.n + 1`: 78 -> 55 ms je Million, PROPERTY 178 -> 111 ms.
`STORE_FIELD` baut keinen Namen/Typ/Fehlertext mehr vorab und loest die
Sperre vor `coerce` (`Self.naechster = Self`). Offen: CALL_METHOD sucht die
Methode je Aufruf ueber die Kette und legt per `split_off` eine neue
Argumentliste an (wie jeder Funktionsaufruf). Tests
`tests/pruef/sprache/objektfelder.dhtest` (auf beiden Laufzeiten gleich; Gegenprobe
mit falscher Aufloesung bzw. ohne Kette: 3 Faelle fallen).

## Feste Lage der Objektfelder (2026-09-25, M4 Schritt 4a)

Eine Instanz ist `Rc<Layout>` + `Vec<Value>` (`value::Instance`); die Lage
(Name -> Platz, Typ je Platz) gehoert der Klasse (`ClassInfo::layout`, in
`model::programm_bauen` gerechnet, Vorfahren zuerst -- ein in der
Unterklasse neu deklariertes Feld bleibt auf dem Platz der Vorfahrin und
nimmt ihren Typ). Die Feldbefehle und CALL_METHOD merken sich (Lage, Platz
bzw. Methode) in `Instr::merk` -- EIN Merkfeld fuer beide, weil ein
groesseres `Instr` Zahlenschleifen 7 % kostete; verglichen wird die LAGE,
nicht der Klassenname. Wer ein Feld liest oder schreibt, geht ueber
`feld_platz`; eine PROPERTY darf NIE in den Merkplatz. `objekte.dh`
125 -> 95 ms; `ganzzahl.dh` +4 % durch die Codelage (gemessen, drei
Aufteilungen probiert, siehe `docs/entwurf-maschinencode.md` M4). Tests
`tests/pruef/sprache/objektlage.dhtest`.

## Aufrufe ohne Zwischenliste, Methoden mit Merkplatz (2026-09-24)

`CALL_USER`/`CALL_METHOD` verschieben die Argumente per `stack.drain(split..)`
direkt in die Locals -- `bind_params`, `exec`, `exec_byref` sind dafuer
generisch (`A: IntoIterator, A::IntoIter: ExactSizeIterator`; eine `Vec`
geht weiter hinein). Eine Coroutine bekommt weiter eine Liste (sie laeuft
spaeter). Parameter nehmen `passend!` statt `coerce`. `CALL_METHOD` merkt sich
in `Instr::methode` (Cell mit rohen Zeigern auf `ClassInfo` und `Func` --
gueltig, weil das Programm nach dem Laden nicht mehr veraendert wird, wie
`CoroState::fn_ptr`) die Klasse und Methode des letzten Aufrufs; getroffen
wird ueber den NAMEN (`ClassInfo::name`), weil jedes Objekt eine eigene Kopie
des Klassennamens traegt. **Der Namensvergleich ist die Sicherung** --
ohne ihn ruft an einer Stelle mit wechselnden Klassen jedes Objekt die
Methode der ersten (Gegenprobe im Test). fib(30) 280 -> 228 ms, Methode
96 -> 86 ms. Tests `tests/pruef/laufzeit/aufrufe.dhtest`.

## Konstanten falten (2026-09-24)

Der Compiler faltet `+ - * / \ MOD ^` und das Vorzeichen ueber Literalen und
globalen CONSTs mit festem Wert (`Compiler::falten`, Einstieg am Anfang von
`expr`). **Gerechnet wird mit `vm::konstant_rechnen`/`konstant_negieren` --
denselben Funktionen wie die Befehle**, darum kann das Ergebnis nicht von der
Laufzeit abweichen (Test: jede Rechenart gefaltet gegen ueber Variablen;
Gegenprobe `\` als `/` faellt). Nicht gefaltet wird bei Fehler (Ueberlauf,
Division durch 0 -- bleibt Laufzeitfehler in DER Zeile), NaN/unendlich,
Texten > 4 KB, und Vergleiche/AND/OR (an ihnen haengen Warnungen). Die
CONST-Werte sammelt `collect_globals` in `konst_werte` (mit Typangabe nur,
wenn der Wert passt, Int -> FLOAT wird Kommazahl; zwei verschiedene Werte =
nie einsetzen); ein lokaler Platz oder ein Feld gleichen Namens verdeckt.
**Folge, vom Nutzer entschieden:** ein Auftrag (TASK_START / `dhrt call`) sieht
jetzt CONSTs mit festem Wert; `GLOBAL_UNGESETZT` sagt das. `x + 2 * FAKTOR`:
225 -> 150 ms je 10 Mio. **Nebenbefund, nicht angefasst:** `statischer_typ`
nimmt fuer `/` immer FLOAT an, die Laufzeit liefert bei glatter Teilung aber
INTEGER (`480 / 2` = 240). Tests `tests/pruef/sprache/konstanten_falten.dhtest`.

## `dhrt run` ohne JSON-Umweg (2026-09-24)

Der Compiler baute fuer jeden Lauf das komplette .dhc-JSON, und `model::
load_program` zerlegte es sofort wieder (Erbe des gemeinsamen Formats mit
Python). Jetzt liefert `build_func` `FuncTeile` (die Befehlsliste wird
UEBERNOMMEN, nicht kopiert); daraus macht `zu_json` das .dhc-JSON (`--export`,
`--dumpbc`, `--check`, Profiler, Debugger) oder `zu_func` direkt die `Func`
(`compile_to_program` -> `run`/`call`, `main.rs::compile_source_programm`).
**Beide Wege gehen durch `model::func_bauen` und `programm_bauen`** (neutrale
Elemente, Merkplaetze, Aufruf-Indizes, PROPERTY-Tabellen) -- wer dort etwas
nachtraegt, muss es nur einmal tun. `add_const` sucht Dubletten ueber
`Ctx::const_index` (JSON-Text als Schluessel) statt linear. IDE-Start
128 -> 98 ms. Tests `tests/pruef/laufzeit/direkt_und_dhc.dhtest` -- **Falle beim
Testen:** Vorgabewerte eines direkten Funktionsaufrufs setzt der Compiler
beim Uebersetzen ein, `param_defaults` wird nur bei Methoden/FUNCREF gelesen;
der Fall braucht darum beides, sonst prueft er das Feld gar nicht.

## `dispatch` geteilt: heisse und seltene Befehle (2026-09-24)

`Vm::dispatch` wird bei JEDEM Aufruf einer Drachenhauch-Funktion neu betreten
(`exec -> run_frame -> dispatch`), und sein Stapelrahmen war 4800 Byte gross
-- groesser als eine Seite, Windows tastet ihn dann bei jedem Eintritt Seite
fuer Seite an (`__chkstk`). Gemessen mit einer Fassung mit Symbolen
(`CARGO_PROFILE_RELEASE_DEBUG=1`, `CARGO_TARGET_DIR=C:/dhsym` -- ein langer
Zielpfad bricht den raylib-Bau) und `llvm-pdbutil dump -symbols dhrt.pdb`
(S_FRAMEPROC). **Die aufgerufenen Hilfsfunktionen waren es NICHT**
(`#[inline(never)]` auf addieren/user_op/coerce/... brachte 4800 -> 4720);
der Rahmen kommt aus den Armen selbst. Jetzt stehen 41 seltene Befehle
(DECLARE_*, PRINT, INPUT_*, TRY/THROW/FIN_END, Tupel, SLICE, IN, NEW,
CALL_VALUE/SUPER, YIELD, HALT, ...) in `Vm::dispatch_selten`
(`#[inline(never)]`, liefert `Some(step)` zum Zurueckkehren), `dispatch`
faengt sie mit `_ =>` ab. Rahmen 4800 -> ~2900 Byte, Code 156 -> 87 KB;
fib(30) -8 %, Zahlenschleife -12 %. **Regel:** ein neuer Befehl, der nicht in
heissen Schleifen laeuft, gehoert in `dispatch_selten`; und wer `dispatch`
waechst, misst den Rahmen nach (unter 4096 halten). `builtin_rufen` NICHT auf
`inline(never)` setzen -- das kostete Builtin-Aufrufe 15 %.

## Messbank und getypte Stufe (2026-09-24, M0/M1 aus `docs/entwurf-maschinencode.md`)

**M0:** `dhrt run bench_dhrt.dh` misst per Vorgabe `tools/tempo/*.dh`; ein
Programm, das `ZEIT <ms>` ausgibt, wird an dieser Zahl gemessen (die Wandzeit
traegt ~40 ms Start), `--gegen exe` misst eine zweite Laufzeit ABWECHSELND
mit und gibt A/B aus. Erster Fund: `LEFT$/RIGHT$/MID$/INSTR` bauten je Aufruf
den ganzen Text als `Vec<char>` -- jetzt `builtins::zeichen_stelle` (zaehlt
nur bis zur Stelle). Tests `werkzeug_bench.dhtest`,
`zeichenketten_stellen.dhtest` (Randfaelle mit Umlaut/Emoji gegen die
Ausgabe von 2026.15).

**M1:** `Compiler::typ_von` (Typ aus `typen.rs`) -- fuer den CODE, muss
stimmen; `statischer_typ` bleibt der vorsichtige Helfer fuer Warnungen.
**`ZAHL`** = INTEGER oder FLOAT je nach Wert: `^` auf zwei INTEGER,
`VAL`, `MIN` gemischt; `AND`/`OR` liefern einen der WERTE (`verbinden`).
Vergleiche mit unbekanntem oder Klassen-Glied bleiben `?` (OPERATOR).
Builtins nur aus der nachgemessenen Tabelle `typen::builtin_typ` -- **"endet
auf `$` = Text" ist falsch** (`SPLIT$` liefert ein Feld; so fand es der erste
Probelauf, 284 Proben). `dhrt --typen datei.dh` listet je Ausdruck den Typ.
**`DHRT_TYPEN_PRUEFEN=1`** setzt hinter jeden getypten Ausdruck `TYP_PRUEFEN`
(Opcode 123, in `dispatch_selten`), die bei falschem Typ mit "der Compiler
sagt X, der Wert ist Y" abbricht -- **wer `typ_von` aendert, laesst
`dhrt test tests/pruef` damit laufen**; das ist der Beleg, nicht die
Beispiele. Ueber `examples/` sind 76 % der Ausdruecke getypt. Tests
`tests/pruef/laufzeit/typen.dhtest` (samt Gegenprobe der Probe ueber ein verfaelschtes
.dhc), Rust-Tests in `typen.rs`.

## Superinstruktionen (2026-09-24, M2 Schritt 1)

`model::verschmelzen` markiert beim Laden (`Instr::schnell`, Arten `VS_*`)
Folgen `a b OP [STORE_LOCAL | STORE_GLOBAL_SLOT | JUMP_IF_*]` mit a/b aus
LOAD_LOCAL/LOAD_CONST/LOAD_GLOBAL_SLOT und OP aus + - * / MOD und den
Vergleichen -- nur, wenn kein Sprungziel (JUMP*, FOR_NEXT-Rumpf, TRY_BEGIN)
IN die Folge zeigt. Der Bytecode bleibt stehen; `Vm::verschmolzen` fuehrt die
Folge vor dem grossen `match` in einem Schritt aus und gibt bei allem, was
nicht der sichere Fall ist, `None` zurueck -- dann laeuft sie Befehl fuer
Befehl mit denselben Meldungen. **Regel fuer Aenderungen:** der schnelle Weg
darf nichts veraendern, bevor feststeht, dass er gelingt, und nie selbst einen
Fehler erzeugen. Nicht unter Profiler/Debugger/Stop. `DHRT_OHNE_VERSCHMELZEN=1`
schaltet ab (auch fuer A/B in derselben Exe), `dhrt --verschmolzen datei.dh`
listet die Stellen. fib 0,86, Ganzzahl-Schleife 0,88. **Gemessen und
verworfen:** Typangabe als Byte statt Text (kein Effekt) und eine eigene
kleine Schleife fuer die haeufigsten Befehle (20 % langsamer bei gemischtem
Code) -- Einzelheiten `docs/entwurf-maschinencode.md` M2. Tests
`tests/pruef/laufzeit/verschmelzen.dhtest` (jeder Fall mit und ohne; Gegenprobe: ohne
Typpruefung beim globalen Speichern faellt "umwandeln beim speichern").

## Rahmenstapel (2026-09-25, M2 Schritt 2)

CALL_USER auf eine schlichte Funktion (keine Coroutine, kein BYREF, nicht
unter Profiler/Debugger/Stop) steigt nicht mehr ueber `exec -> run_frame ->
dispatch` neu ein: `dispatch` schiebt den Aufrufer als `Rahmen` auf
`Vm::rahmen` und laeuft mit dem Gerufenen weiter, RETURN/RETURN_VOID/HALT/
Code-Ende kehren per `zurueck!` zurueck (nur oberhalb der `basis`, die jeder
`run_frame` beim Eintritt nimmt). `run_frame` wickelt Fehler Rahmen fuer
Rahmen ab (`rahmen_verlassen`) und nimmt die Fehlerzeile aus der Funktion,
in der der Fehler FIEL -- nicht aus der, mit der `dispatch` betreten wurde.
EXIT/Stop raeumen alle Rahmen oberhalb der Basis ab. `bind_params` bindet
den Normalfall direkt. Leerer Aufruf 41 -> 29 ns, fib 0,85. Tests
`tests/pruef/laufzeit/rahmenstapel.dhtest` (Erwartung = Ausgabe des Baus davor).
**Methoden (CALL_METHOD) bewusst NICHT auf dem Rahmenstapel** -- gemessen:
das je Rahmen gefuehrte `Self` machte jeden Befehl in `dispatch` teurer
(freier Aufruf 30 -> 35..40 ns, fib 1,07..1,09) fuer 2 ns je Methode.
**Zeichenstellen mit ASCII-Abkuerzung** (`builtins::zeichen_stelle`,
`zeichen_stelle_genau`, `zeichen_zahl`): MID$/LEFT$/RIGHT$/INSTR/
REGEX_FIND_POS zaehlen nur noch, wenn vor der Stelle Nicht-ASCII steht --
`text.dh` 700 -> 50 ms; die Schleife je Zeichen schwankte vorher um den
Faktor 1,6 mit der Codelage. Fall "ascii davor und umlaut dahinter" in
`zeichenketten_stellen.dhtest` (gegen den Bau davor gleich).

## Maschinencode fuer reine Zahlenfunktionen (2026-09-25, M3 Schritt 1)

`src/jit.rs`, Feature `jit` (Cranelift **0.136**, Rust 1.98), **seit M5
an per Vorgabe; `DHRT_JIT=aus` schaltet ab** (Funktionen beim Start,
Schleifen beim ersten Ruecksprung). Uebersetzt wird nur eine REINE Funktion: Parameter/Locals/
Rueckgabe INTEGER/FLOAT/BOOLEAN, nur Rechnen/Vergleichen/Springen/FOR ueber
INTEGER und Aufrufe reiner Funktionen -- keine Globals, Builtins, Ausgabe.
`analysieren` verfolgt je Stelle die Art jedes Stapelplatzes und Locals
(abstrakte Ausfuehrung; zwei Wege mit verschiedenen Arten = VM), `erzeugen`
baut daraus Cranelift-Code, ein `trampolin` je Funktion ist der Einstieg fuer
die VM (CALL_USER, nur wenn jedes Argument genau passt, nicht unter
Profiler/Debugger/Stop). **Bei JEDEM Fehlerfall gibt der Maschinencode auf
und die VM rechnet den Aufruf nach** (rein = keine Nebenwirkung) -- Meldung
und Zeile kommen von ihr, es gibt keine zweite Fassung. Wer eine Regel der
VM aendert (`div`, `modulo`, `int_div`, `cmp`, `coerce`, `value_eq`), muss
`rechnen`/`vergleichen`/`wandeln` in jit.rs mitziehen -- `jit.dhtest` faengt
es (jeder Fall mit und ohne). `dhrt --jit datei.dh` nennt je Funktion den
Grund. fib(30) 144 -> 3,3 ms, Zahlenschleife in einer Funktion 189 -> 5 ms.
Die CI laeuft die Sammlungen zweimal. Einzelheiten `docs/entwurf-maschinencode.md` M3.
**Globale Zahl-Variablen (M4 Schritt 1) ueber einen Schatten**: der
Maschinencode liest ueber `global_holen` in `Kontext::schatten` (Merkbyte je
Platz: 0 frei, 1 geholt, 2 geaendert) und schreibt nur dorthin; `Jit::rufen`
uebernimmt die geaenderten Plaetze NUR nach einem fehlerfreien Aufruf, sonst
rechnet die VM von vorn. **Wer das aendert, braucht den Fall "hochzaehlen vor
einem fehler zaehlt einmal"** -- ein Wert, den die VM beim Nachrechnen
ueberschreibt, verraet ein falsches Uebernehmen nicht.
**Schleifen einzeln (M4 Schritt 2):** beim ersten Ruecksprung (FOR_NEXT/
JUMP rueckwaerts) wird die Schleife als Bereich mit den AKTUELLEN Arten der
Locals uebersetzt (`Jit::schleife`, Merker `Instr::schleife`: 0 unversucht,
1 nie, n>=2 Bereich n-2); Rueckgabe = Stelle, an der die VM weitermacht, -1 =
aufgegeben. Globale, die keine gerufene Funktion beruehrt, leben im Bereich
als Variable (`befoerdert`) und gehen an jedem Ausgang in den Schatten.
**Beim Aufgeben rechnet die VM die GANZE Schleife ab dem Kopf** -- darum darf
der Bereich Locals/Globale nur an Ausgaengen schreiben (Fall "schleife ruft
eine funktion mit global und laeuft ueber"). `DECLARE_GLOBAL_SLOT` mit schon
vorhandenem Platz ist im Bereich ein Nichts (sonst blieb die aeussere von zwei
Schleifen VM). `DHRT_JIT_BILANZ=1` nennt Zahlen und Gruende. zahlen.dh
236 -> 6 ms, ganzzahl.dh 488 -> 10 ms.
**Felder und Aussteigen mitten drin (M4 Schritt 3):** ein Bereich liest
und schreibt Felder von INTEGER/FLOAT direkt (`Art::Feld(nr)`,
`FeldInfo`; die VM beschreibt sie bei JEDEM Eintritt im Kontext: Zeiger,
Groessen, Schritte -- zwischen Eintritten darf ein Feld wachsen). Dafuer
steigt jeder Bereich jetzt am FEHLERHAFTEN Befehl aus statt aufzugeben
(`Bauer::punkt`, `Aussteig`, Rueckgabe `-2 - nr`): Locals, Globale und
Stapel VOR dem Befehl gehen zurueck, die VM fuehrt ihn selbst aus und
meldet ihren Fehler -- Geschriebenes wird nie doppelt gerechnet. **Wer
einen neuen Befehl in `erzeugen` einbaut, muss `bau.punkt` vor jeder
Pruefung gueltig lassen** (es wird am Anfang jedes Befehls gesetzt).
Ausnahme: ruft der Bereich eine Funktion, die Globale schreibt
(`schreibt_globale`), gibt er wie frueher auf und darf keine Felder
schreiben. felder.dh 92 -> 13 ms.

**Objekte im Maschinencode (M4 Schritt 4b):** `Art::Obj(k)` = Zeiger auf
`RefCell<Instance>`, Felder ueber die Helfer `feld_lesen_*`/`feld_setzen_*`
(die Lage eines `Value` ist nicht festgelegt, also kein direkter
Speicherzugriff). Elemente von Feldern von Werten/Tupeln (`Elem::Wert`) und
Objekt-Felder werden beim Lesen auf ihre Klasse geprueft -- sonst
aussteigen. Globale ohne festen Zahlentyp (Objekte, FOR-EACH-Variable)
fuehrt ein Bereich wie Locals (`dyn_glob`, `umleiten`). **Die Laufvariable
von FOR EACH im Hauptprogramm hat seither einen globalen Platz**
(`collect_globals`) statt ueber ihren Namen zu laufen. teilchen.dh
661 -> 177 ms (VM) -> 27 ms (Maschinencode).

**Wertemodus (M4 Schritt 5):** scheitert ein Bereich getypt (Text, MAP,
Befehl), baut `Jit::wertebereich` ihn mit `Art::W`: Werte liegen als
`Value` in `Kontext::werte` (Local s = Platz s, Stapeltiefe d = Platz
`n_gesamt + d`), der Maschinencode ruft je Befehl einen Helfer `w_*`.
Werte werden beim Eintritt aus der VM VERSCHOBEN und an jedem Ausgang
zurueck. Befehle nur mit `familie == BUILTIN_FAMILIEN` (also schon einmal
von der VM an dieser Stelle gerufen). **`s = s + e` gibt den Platz von `s`
vorher ab** (`w_op`, `lokal`), sonst kopiert jede Runde. text.dh 53 -> 41 ms.

**Befehle aller Familien im Wertemodus (M4 Schritt 6):** `w_builtin_roh`
ruft ueber `Kontext::vm` `Vm::builtin_rufen`, ausser `befehl_im_bereich`
sagt nein (Familien 2/4/8, `TIMER_UPDATE`, `GUI_UPDATE`, `ASSERT` --
alles, was Drachenhauch-Code ruft, sieht sonst Locals/Globale, die der
Bereich gerade haelt). **Wer eine Familie oder einen Befehl dazubaut, der
Rueckrufe feuert, traegt ihn dort ein.** Ein gescheiterter Befehl wird
nicht nachgerechnet: `Jit::meldung_nehmen` im Haken der VM gibt den Fehler
an seiner Stelle aus. familien.dh 200 -> 16,5 ms.

**Methodenaufrufe in Bereichen (M4 Schritt 7a):** `w_methode` ->
`Vm::methode_rufen` (derselbe Weg wie CALL_METHOD). `methoden_harmlos`
prueft beim Bauen alle Methoden des Namens in allen Klassen samt
Operatoren, PROPERTYs und allem Gerufenen: keine Globale, die der Bereich
beruehrt, kein FUNCREF/SUPER/Coroutine/Rueckruf. **Wer einen Befehl
dazubaut, der Drachenhauch-Code ruft, traegt ihn dort UND in
`befehl_im_bereich` ein.** `w_op` rechnet mit `vm::wert_rechnen`, nicht mit
`konstant_rechnen` (das hat die Grenzen des Faltens).

**Methodenrumpfe als Maschinencode (M4 Schritt 7b):** Methodentafel
`Globale::tafel` -- je Klasse jede sichtbare Methode (auch geerbte) mit der
Lage DIESER Klasse, sonst loeste `Self.hilfe()` statisch falsch auf.
Aus der VM ueber `Jit::methode` (nur bei genau passender Lage), aus
getyptem Code statisch ueber `Obj(k)`. **Feldschreibungen gehen ins
Journal** (`Kontext::journal`), damit ein aufgebender Aufruf von vorn
gerechnet werden darf; im Bereich nur waehrend eines Methodenaufrufs.
objekte.dh 97 -> 12 ms.

**Reine Zahlenbefehle im Maschinencode (M4 Schritt 8):** `zahl_befehl`
nennt ABS/INT/FLOOR/CEIL/ROUND/SGN/FLT/SQR/SIN/COS/TAN/ATAN/EXP/LOG/ASIN/
ACOS/DEG/RAD/FRAC/ATAN2/HYPOT/LERP/MIN/MAX/CLAMP mit Zahlen-Argumenten,
`zahl_rechnen` baut sie nach builtins.rs (Ganzzahl geht als Kommazahl
hinein wie `need_num`); jeder Fehlerfall steigt aus. **Wer einen dieser
Befehle in builtins.rs aendert, zieht `zahl_rechnen` mit** -- `jit.dhtest`
faengt es. builtins.dh 107 -> 3,3 ms.

**Spielschleifen (M4 Schritt 9):** `NOT` auf einen Wert im Wertemodus
(`W_WAHR` + Bit kippen) und vorbelegte Konstanten (`vorbelegt`: Farben,
Tasten, PI/TAU aus `LOAD_NAME` als Zahl). Beispiele: 377 -> 343 Schleifen
in der VM, `WHILE NOT QUITREQUESTED()` blockiert nicht mehr.

**PRINT im Wertemodus (M4 Schritt 10):** `Vm::drucken` ist der Rumpf von
`op::PRINT`, `w_drucken` ruft ihn aus dem Maschinencode. Beispiele: 343 ->
304 Schleifen in der VM.

**Bilanz mit Runden (M4 Schritt 11):** `DHRT_JIT_BILANZ=1` nennt am Ende
die zehn Schleifen, die am oeftesten in der VM gedreht haben, samt Grund --
**den naechsten Schritt danach waehlen, nicht nach der Zahl der
Schleifen** (eine Startschleife mit drei Runden wog sonst wie die
Hauptschleife). Handles (IMAGE, SPRITE_ATLAS, SOUND ...) sind Ganzzahlen
und duerfen im Bereich gelesen werden.

**Spaet gebundene Befehle (M4 Schritt 12):** ein Befehl ohne bekannte
Familie (Zweig lief vor dem ersten Ruecksprung nicht) haelt die Schleife
nicht mehr in der VM; `w_builtin_roh` steigt beim Laufen aus, solange die
Familie fehlt oder verboten ist. Runden in der VM ueber die Beispiele
708 451 -> 248 320 (Schritte 11+12).

**Globale ohne Platz (M4 Schritt 13):** ein Befehl, der eine Globale
anfasst, die es beim Bauen noch nicht gibt (DIM im seltenen Zweig), ist ein
fester Ausgang (`globale_fehlt`); taucht der Platz auf, wird die Schleife
neu gebaut. Runden in der VM ueber die Beispiele 248 320 -> 156 616.

**Globale Felder in Funktionen, RGB (M4 Schritt 14):** eine FUNKTION
liest ein globales Feld von INTEGER/FLOAT ueber den Helfer `gfeld_lesen`
(sie hat keine Feldbeschreibung wie ein Bereich); die Elementart kommt
statisch aus der Anlage im Hauptprogramm (`Globale::felder`). Nur lesen,
bei jedem Fehler rechnet die VM die Funktion nach. `RGB`/`RGBA` mit
Ganzzahlen stehen in `zahl_befehl`. Runden 156 616 -> 132 738.

**VM-Funktionen aus Bereichen (M4 Schritt 15):** eine Funktion, die nicht
uebersetzt ist, ruft ein Bereich im Wertemodus ueber `w_funktion` ->
`Vm::funktion_rufen` (`Bereich::vm_fns`); geprueft wie Methoden
(`gerufenes_harmlos`: keine Globale des Bereichs, keine Umwege). BYREF und
Coroutinen bleiben in der VM. Runden 132 738 -> 73 456.

**Tupel im Wertemodus (M4 Schritt 16):** `UNPACK_TUPLE`/`BUILD_TUPLE`
ueber `w_auspacken`/`w_tupel`; passt ein Tupel nicht, steigt der Bereich
vor dem Befehl aus und die VM meldet. Runden 73 456 -> 40 114.

**Mitglieder von Werten (M4 Schritt 17):** `obj.x` auf einem Objekt als
Wert (Wertemodus) ueber `w_mitglied_lesen`/`_setzen` -> `Vm::member_laden`/
`member_setzen`; PROPERTYs dieses Namens prueft `mitglied_harmlos` wie
Methoden. Runden 40 114 -> 18 233.

**FOR mit Kommazahl- oder Wert-Grenze (M4 Schritt 18):** die Grenze einer
FOR-Schleife darf F oder (Wertemodus) W sein; Locals ohne Typ (`any`, etwa
die verborgene Grenze) sind im Wertemodus ein Wert, aber seit Schritt 19
nur im dritten Versuch (`Bereich::any_w`). Runden 18 233 -> 9 033.

**Laufzeit statt Runden (M4 Schritt 19):** die VM-Runden haben zweimal in
die falsche Richtung gezeigt (teilchen.dh mit Maschinencode LANGSAMER als
die VM). Liegen Mitglieder von Werten in einer inneren Schleife, wird die
aeussere nicht im Wertemodus gebaut (`innere_schleife_mit`). **Wer am
Maschinencode aendert, laesst `dhrt run bench_dhrt.dh` mit und ohne
`DHRT_JIT=aus` laufen**, nicht nur die Rundenzaehlung.

**maps.dh (M4 Schritt 20):** Text zusammensetzen im Wertemodus wie
`Vm::addieren` (ein Text in passender Groesse; vorher langsamer als die
VM), `w_speichern` mit schnellem Weg wie `passend!`, der Index der MAP mit
foldhash (zufaelliger Startwert -- Schluessel koennen von aussen kommen)
und `DhMap::put_str`. maps.dh 107 -> 81 ms.

**MAP ueber indexmap (M4 Schritt 21):** `DhMap` ist eine
`indexmap::IndexMap<String, Value>` mit foldhash (dichte Eintraege in
Einfuege-Reihenfolge, der Index haelt nur Positionen -- wie Pythons `dict`);
Loeschen mit `shift_remove`. Eine Million Schluessel 214 -> 160 MB, Loeschen
493 -> 288 ms. Die Laufzeit-Typen heissen `DhMap`/`DhArray`/`DhFile` (vorher
`Gb*`, von GameBasic).

**Mauszeiger (2026-09-27):** die gui meldet je Bild einen Zeigerwunsch nach
der STELLE im Widget (`Gui::zeiger_ueber`: Tabellenkante, Nummernspalte,
Verweis, Farbfeld ...), Graphics setzt beim FLIP (`zeiger_anwenden`, nur bei
Wechsel). `MOUSE_CURSOR` bleibt stehen und hat Vorrang, `"auto"` gibt an die
gui zurueck; `GUI_SET_CURSOR(wdg, form$)`, `MOUSE_CURSOR_GET$()`. Neue Stellen
bekommen ihren Zeiger aus derselben Geometrie wie Treffertest und Zeichnen.
Tests `tests/pruef/gui/gui_mauszeiger.dhtest`.
**Eigene Zeigerbilder (Teil 3):** raylib kennt nur zehn Formen. Dazu kommen
`warten`/`arbeitet`/`hilfe` -- unter Windows die echten Systemzeiger ueber
einen Subclass, der `WM_SETCURSOR` vor GLFW beantwortet (`systemzeiger.rs`;
per `SetCursor` gesetzt kaeme GLFW bei der naechsten Bewegung wieder durch) --,
gezeichnete `kopieren`/`stift`/`pipette` (`zeigerbilder.rs`, 16er-Raster mit
selbst gelegtem schwarzem Rand, doppelt vergroessert) und
`MOUSE_CURSOR_NEW(bild, bx, by)` -> Name `eigen1` ... Bilder gehen ueber
`glfwCreateCursor`/`glfwSetCursor` direkt an das GLFW in raylib, einmal
angelegt und wiederverwendet. Nachgemessen am echten Windows-Zeiger
(`GetCursorInfo`), die Gegenprobe ohne Abfangen zeigt den Pfeil.

**Handbuch schoener, Schriftzug hinter dem Code (2026-09-27):**
`GUI_RICHTEXT` setzt Codebloecke eingefaerbt (`syntax::spans` je Zeile, die
Stuecke LUECKENLOS nach Breite des Vorspanns, sonst lieferte Kopieren
Leerzeichen dazwischen; Farben per `GUI_RICHTEXT_SET "farbe_*"`), Code im
Text kleiner auf eigenem Grund (`rt_code_gr`, `RtLauf::dy`), Ueberschriften
in drei Farben, Tabellen mit Kopf- und Zebra-Band, Zitate als EIN Block,
Verweise mit durchgehender Linie. Baender sind Zeilen ohne Text
(`RtZeile::band`), die VOR ihrem Inhalt stehen. Neu `GUI_RICHTEXT_HEADINGS/
HEADING_LEVELS/GOTO_HEADING/HEADING_AT` und `GUI_TEXTAREA_BACKGROUND(ta,
bild, deckkraft)` (Cmd `TextureRectTint`). **Fund:** `GUI_RICHTEXT_FIND` und
`_SCROLL` direkt nach `SET_TEXT` arbeiteten auf dem ALTEN Satz (gesetzt wird
erst in GUI_UPDATE) -- jetzt vorgemerkt (`satz_aktuell`, `ziel_suche`),
FIND liefert dann 0. Die IDE: Handbuch mit Inhaltsverzeichnis, Vor/Zurueck,
Farben des Editors; Einstellungen "Schrift der Oberflaeche" (Vorgabe 18) und
"Schriftzug hinter dem Code". Tests `tests/pruef/gui/gui_gesetzter_text_gestaltung.dhtest`,
`tests/pruef/ide/werkzeug_ide_handbuch.dhtest`.

**Codeknoepfe, Suche ueber alle Dokumente, farbiger Quelltext (2026-09-27):**
`GUI_RICHTEXT_CODE_BUTTONS(rt, "Kopieren|Starten")` legt einen Streifen
oben in jeden Codeblock mit rechtsbuendigen Knoepfen (`RichState::
codebloecke` = je Block y, Hoehe, Code, Knopf-Rechtecke; eine Quelle fuer
Zeichnen und Klick); ein Klick darauf beginnt KEINE Auswahl und meldet ein
Bild lang `GUI_RICHTEXT_CODE_ACTION$` (+ on_click), der Code kommt aus
`GUI_RICHTEXT_CODE$`; `.dhform` `code_knoepfe`. Die IDE: Kopieren / In neuen
Reiter (ungesichert markiert) / Starten (`handbuch_beispiel.dh` neben der
ide.json, ueber `prozessStartenDatei`, das aus `prozessStarten` geloest
ist). "In allen" (`hbSucheAlle`) listet Treffer je Dokument in der
Inhaltsliste (`hbTrModus`, Koepfe per `GUI_LISTBOX_HEADER`, Zeile als
Detail); `handbuchLaden` fuellt das Inhaltsverzeichnis dann NICHT neu.
Quelltext-Ansicht in `codeFont`, eingefaerbt von `hbQuelleFaerben`.
**Testfalle:** "In neuen Reiter" holt das Hauptfenster nach vorn, danach
treffen Klicks das Handbuch nicht mehr -- und Knopflagen haengen an
Schriftbreiten; die Tests klicken darum immer den RECHTEN Knopf und stellen
die Reihenfolge vorher um. Tests `tests/pruef/gui/gui_richtext_codeknoepfe.dhtest`
und drei Faelle in `werkzeug_ide_handbuch.dhtest`.

**Karte zum Anklicken, Schnellkorrektur (2026-09-28):** die
Uebersichtskarte der IDE fragte `MOUSE_HIT(1)` -- das ist die RECHTE Taste
(0 = links), ein Linksklick tat seit Stand 5 nichts. Jetzt rollt ein Klick
die Stelle in die Mitte, Ziehen folgt (`kartenZug`), die Marke bleibt: neu
**`GUI_TEXTAREA_SCROLL(ta, zeile)`** (rollt wie das Rad, setzt `rad_stand`,
sonst zoege die Editierschleife den Ausschnitt zur Marke zurueck). Die Karte
zeigt dazu Fehler/Warnungen rechts und die Zeile der Marke links.
**Schnellkorrektur**: `lsp::korrekturen` liest die MELDUNG ("Meintest du",
"heisst in Drachenhauch X(", "nirgends ... angelegt", Klammern, `!=`) und
liefert Aenderungen; `diagnose` haengt sie als `data.korrekturen` an,
`CODE_CHECK$` gibt sie ab 1 gezaehlt als `korrekturen` heraus, `dhrt lsp`
als `textDocument/codeAction` (quickfix). Ein fehlendes DIM kommt unter den
Kopf des Unterprogramms bzw. vor den Block auf oberster Ebene -- in einer
Schleife setzte es die Variable jede Runde zurueck; der Typ ist aus dem Wert
geraten (`typ_raten`). Die IDE: Strg+. (Kuerzel `.`/`,` versteht
`kuerzel_parsen` seither), Waehler `paletteArt = "korrektur"`, angewendet
ueber SELECT + INSERT von hinten. Tests `tests/pruef/gui/gui_textarea_scroll.dhtest`,
`code_korrekturen.dhtest`, `werkzeug_ide_karte.dhtest`,
`werkzeug_ide_korrektur.dhtest`, ein codeAction-Fall in `dhrt_lsp.dhtest`,
Rust-Tests `korrekturen_aus_der_meldung`, `typ_raten_aus_dem_wert`,
`punkt_und_komma`; sieben Verfaelschungen fallen je in ihrem Fall.

**Mehr Schnellkorrekturen, Werte im Code (2026-09-28):** `lsp::korrekturen`
kennt dazu fehlende Blockenden (drei Meldungsformen: "X erwartet,
Programmende erreicht", "Erwartet IF nach END" -- ein END SUB kam, waehrend
das IF offen war --, "WEND (oder END WHILE) erwartet"; `blockende_stelle`
setzt es hinter den tiefer eingerueckten Rumpf des letzten offenen Kopfes,
nicht ans Dateiende), einen fehlenden IMPORT (beide Satzformen; hinter die
vorhandenen IMPORTs, sonst hinter den Kopfkommentar) und **unbenutzte
Variablen**: neuer Hinweis (LSP-Schwere 4, `CODE_CHECK$` "hinweis", nur
DIM in Unterprogrammen mit einem Vorkommen im Rumpf -- eine globale kann
eine importierende Datei benutzen), Korrektur "Zeile entfernen" nur ohne
Aufruf im Wert. Eine Aenderung kann jetzt ueber das Zeilenende gehen
(`bis == usize::MAX` intern, `bis_zeile`/`bis_spalte` in `CODE_CHECK$`).
Die IDE zeigt Hinweise grau, zaehlt sie nicht als Problem (`geprueft`
bleibt Fehler+Warnungen, eigene Zeile `hinweise N`), keine Marke, nicht auf
der Karte. **Werte im Code:** neu `GUI_TEXTAREA_HINTS(ta, zeilen, texte
[, farbe])` -- Text hinter dem Zeilenende, folgt seiner Zeile wie eine
Marke (`marken_abgleichen`), `GUI_SET_TEXT` loescht. Die IDE schreibt im
Halt (`debugWerteZeigen`) hinter die Zeilen vom Kopf des Unterprogramms bis
zur Haltezeile die vorkommenden Variablen (lokal vor global, ohne
Kommentare/Texte, hoechstens vier je Zeile); `debugWerteLoeschen` beim
Weiterlaufen und am Ende. Tests `gui_textarea_hints.dhtest`,
`werkzeug_ide_werte.dhtest`, Faelle in `code_korrekturen.dhtest` und
`werkzeug_ide_korrektur.dhtest`, Rust-Test `blockende_import_und_unbenutzt`;
fuenf Verfaelschungen fallen je in ihrem Fall.

**Mitlaufender Blockkopf (2026-09-29):** `GUI_TEXTAREA_SET(ta,
"kopfzeilen", n)` -- ist die Kopfzeile eines Blocks aus
`GUI_TEXTAREA_FOLDABLE` oben hinausgerollt, der Block aber noch im Bild,
steht sie oben angeheftet (hoechstens n, aussen zuerst). EINE Quelle
`Gui::ta_koepfe(wdg, rows, sicht, scroll)` fuer Zeichnen, Klick (springt
hin), `GUI_TEXTAREA_POS_AT`/Tooltip-Treffer (unter einem Kopf: kein Text),
Nummernspalte (kein Haltepunkt auf der verdeckten Zeile) und das Mitziehen
der Marke (rollt zurueck, bis sie frei steht); die Koepfe verdecken selbst
Zeilen, darum wird gerechnet, bis die Liste stehen bleibt.
`GUI_TEXTAREA_SELECT`/`GOTO` zentrieren die Marke und sind davon
unberuehrt. Die IDE: Ansicht -> Mitlaufender Blockkopf (ide.json
`blockkopf`, an per Vorgabe, drei Koepfe). Tests
`tests/pruef/gui/gui_blockkopf.dhtest` (4, je mit Gegenprobe ohne Koepfe),
`tests/pruef/ide/werkzeug_ide_blockkopf.dhtest`.

**Ganzes Projekt pruefen (2026-09-29):** Strg+Umschalt+F7 in der IDE
(`projektPruefenStarten`/`-Schritt`, 8 Dateien je Bild, ESC bricht ab,
Liste im Such-Modus `problemModus = 1` ueber `sucheDateien`/`sucheZeilen`).
Eine Meldung "in lib.dh:2 -> ..." (Fehler in einer importierten Datei, von
jeder importierenden gemeldet) wird auf ihre eigene Stelle umgeschrieben
und ueber (kanonischer Pfad, Zeile, Meldung) nur einmal gezeigt; Hinweise
zaehlen nicht. Protokoll `projekt geprueft F W betroffen dateien`. Test
`tests/pruef/ide/werkzeug_ide_projektpruefung.dhtest` (ohne das Entdoppeln
faellt er).

**Aufrufzaehler, Pruefsammlungen in der IDE (2026-09-29):** hinter jeder
SUB/FUNCTION steht als Hinweis (`GUI_TEXTAREA_HINTS`), wie oft ihr Name im
Projekt benutzt wird; ein Klick oeffnet "Wer ruft das auf?". Gezaehlt ueber
neu **`CODE_NAMES(quelle$)` -> MAP OF INTEGER** (`syntax::namen_zaehlen`:
Namen ohne Kommentare/Texte, ohne die Definition -- der Name hinter
SUB/FUNCTION in DERSELBEN Zeile, nicht hinter END/EXIT SUB; die erste
Fassung zaehlte `END SUB` + naechste Zeile als Definition). In
Drachenhauch Zeichen fuer Zeichen gezaehlt kostete ide.dh 310 ms je
Pruefung, ueber CODE_NAMES 22 ms; Dateien, die nicht offen sind, merkt
sich die IDE als Text ";name=n;" bis zum naechsten Sichern (`zaehlCache`).
Neu in der Laufzeit **`GUI_TEXTAREA_SET(ta, "hinweise_klickbar", 1)` +
`GUI_TEXTAREA_HINT_CLICKED(ta)`** (Rechtecke aus EINER Quelle
`ta_hinweis_rects` fuer Zeichnen, Klick, Zeiger). Im Debugger-Halt stehen
die Werte dort, `debugWerteLoeschen` holt die Zaehler zurueck.
**Pruefsammlungen:** Strg+Alt+F5 (`pruefsammlungStarten`, eigener Prozess
`testProz` neben dem Programmlauf) laesst `dhrt test` ueber die .dhtest im
Reiter oder den Projektordner laufen; FEHL-Zeilen kommen mit Datei und
Zeile in die Liste (`problemModus = 1`; `problemAnspringen` nimmt jetzt
auch absolute Wege). Strg+Alt+Umschalt+F5 nimmt den Fall unter der Marke
ueber neu **`dhrt test --fall Name`** (genau dieser Name -- `--filter`
sucht Teiltexte, "geht" traf auch "geht nicht"). Tests
`werkzeug_ide_aufrufzaehler.dhtest`, `werkzeug_ide_pruefsammlung.dhtest`,
ein Klick-Fall in `gui_textarea_hints.dhtest`, Rust-Test
`namen_zaehlen_ohne_definition_kommentar_text`.

**Eigene Schnipsel, eigene Tastenbelegung, Terminal (2026-09-29):** die
136 Zeilen `IF GUI_CLICKED(miX) THEN befehl("k")` der Bildschleife sind EINE
Tafel (`menueBefehl(mi, kennung, vorgabe[, klickt])`, `menueKlicks`) -- aus
ihr liest auch die Tastenbelegung. `tasten.json` neben der ide.json haelt
nur Abweichungen; `tastenAnwenden` stellt erst alles auf die Vorgabe, dann
die Datei darueber, und nimmt einem anderen Befehl die Taste weg
(`kuerzelForm$` vergleicht "Umschalt+Strg+O" = "strg+shift+o"). **Falle:**
`Strg+Alt+N` gehoerte schon "Neues Projekt aus Vorlage" -- Konflikte sind
kein Randfall. `schnipsel.txt` (`=== Name | kuerzel`, dann der Text) kommt
zu den eingebauten, ein gleiches Kuerzel ersetzt; beide gelten nach dem
Sichern im Reiter (Haken in `sichern`, der Projektordner wandert dabei
nicht). `konfigOrdner$` gibt "." fuer eine ide.json ohne Ordner (die
Tests). **Terminal** (Alt+3, Reiterknoepfe ueber der Ausgabe): je Befehl
ein Skript (`cmd /Q /D /C x.cmd` bzw. `sh x.sh`) mit `cd` am Anfang und
einer Marke `__DH_TERMINAL_ORT__` am Ende -- so wirkt `cd` ohne dauerhafte
Shell (deren Eingabeaufforderung kaeme ohne Zeilenende). `chcp 65001` fuer
Umlaute. **Fund in der Laufzeit:** `PROCESS_KILL` beendete nur das Kind --
`cmd /C ping` abgebrochen liess den ping laufen, und der hielt die GEERBTEN
Leitungen offen (der Testlaeufer wartete 20 s auf ihn). Jetzt `taskkill /T`
unter Windows bzw. eigene Prozessgruppe (`process_group(0)`) + `kill -KILL
-pgid` unter Unix (`prozess::baum_beenden`). **Testfalle:** eine Beilage mit
`===`-Zeilen haelt der Laeufer fuer Faelle -- die schnipsel.txt schreibt ein
`--- vorher`. Tests `werkzeug_ide_eigenes.dhtest` (6, darunter echte Tasten
mit Gegenprobe), `werkzeug_ide_terminal.dhtest` (2, `--- system windows`),
Rust-Test `kill_beendet_auch_die_enkel`; vier Verfaelschungen der IDE und
eine der Laufzeit fallen je in ihrem Fall.

**Parameternamen an Aufrufen, beim Einfuegen einruecken (2026-09-30):**
`parameterSammeln` (in `pruefen`) sucht ueber SYNTAX_SPANS je Zeile den
ersten `name(` mit festem Argument (`festesArgument`), Signatur ueber
`signaturArgs(..., mitOptionalen = TRUE)` (CODE_HOVER$), gemerkt in
`paramCache` bis zum Sichern, hoechstens 60 neue Namen je Pruefung.
`einfuegenAnpassen` rueckt einen mehrzeiligen Block um (Bezug = kleinste
Einrueckung der weiteren Zeilen), als eigener Undo-Schritt; Strg+V wird in
der Aenderungs-Erkennung der Hauptschleife erkannt (Tastenflanke ODER
`strgVBilder` -- die Erkennung laeuft vor GUI_UPDATE, der Text kommt ein
Bild spaeter an). **Fund in der Laufzeit:** das Ausruecken von
`GUI_TEXTAREA_INDENT_WORDS` (`aus`-Liste) griff auch nach einem
EINGEFUEGTEN Block -- endete er mit `NEXT`/`END IF`, verlor die Zeile
ihre Einrueckung (der Kommentar im Code behauptete das Gegenteil). Jetzt
nicht bei mehrzeiligem Einfuegen (`eingefuegt` in gui.rs); ein einzelnes
eingefuegtes Wort zaehlt weiter wie getippt, weil die Pruefsammlungen ueber
Strg+V "tippen". Tests `werkzeug_ide_parameter_einfuegen.dhtest` (3), ein
Fall in `gui_auto_einzug.dhtest`; fuenf Verfaelschungen der IDE fallen.

**Naechstes Problem, letzte Aenderung, mit gesicherter Fassung vergleichen
(2026-09-30):** nur IDE. `problemSpringen` ueber `probZeile/probSpalte`
aus `wellenSetzen` (Hinweise ausgenommen, Umlauf), Alt+F8 -- F8 gehoert dem
Debugger. `aenderungMerken` haengt an der Aenderungs-Erkennung der
Hauptschleife (nicht beim ersten Bild nach dem Oeffnen: da ist der gemerkte
Text noch leer), `zurLetztenAenderung` legt die aktuelle Stelle auf den
Zurueck-Stapel; Strg+Umschalt+Rueck, weil Strg+Alt+Buchstabe auf deutschen
Tastaturen AltGr ist (Strg+Alt+E = €). `diffText$` = diff -U3 aus derselben
LCS-Rechnung wie der Reiter-Vergleich, im Fenster `winVgl` mit
`diffFaerben`; Verwerfen setzt den Text zurueck, gegen den verglichen wurde
(`vglPlatte`), nicht `tabGesichert`. Tests
`werkzeug_ide_problem_aenderung_vergleich.dhtest` (4, echte Tasten und Klicks);
sechs Verfaelschungen fallen je in ihrem Fall.

**Gross/klein, Zeilen verbinden, Klammerpaare farbig, Suchtreffer einzeln
ersetzen (2026-09-30):** alles in der IDE, ohne Laufzeit-Aenderung.
`fallAendern` (Auswahl oder Wort, Auswahl danach auf der NEUEN Laenge --
`UPPER$("ß")` ist "SS"), `zeilenVerbinden` (SELECT ganzer Zeilen + INSERT =
ein Undo-Schritt). `klammernFaerben` in `faerben`: nur `operator`-Abschnitte
der Laenge 1 aus SYNTAX_SPANS, Tiefe je Zeile neu ausser nach ` _`, drei
Farben `F_KLAMMER1..3` (auch in farben.json); der Faerbe-Riegel kennt den
Schalter. Suchtreffer: `projektSuchen` schaltet Haken an (nur ein Suchtext),
`trefferNachziehen` nimmt sie weg, sobald die Liste etwas anderes zeigt
(Zahl oder erste Zeile anders -- die vielen Stellen, die `lstProbleme`
fuellen, muessen nichts davon wissen); `trefferErsetzen` baut je Datei den
Text aus Reiter oder Platte, ersetzt nur in den gewaehlten Zeilen und geht
durch `umbauVormerken`/`umbauStarten`. Tests
`werkzeug_ide_text_klammern_treffer.dhtest` (4, Klammerfarben am Bild);
fuenf Verfaelschungen fallen je in ihrem Fall.

**Kontextmenues fuer Code-Feld und Reiter, Im Projektbaum zeigen
(2026-09-30):** neu **`GUI_CONTEXT_TABS(menu)`** + **`GUI_TAB_CONTEXT(win)`**
(`Menu::ziel` = -2, `Window::tab_kontext`; im Rechtsklick-Zweig von
`menu_input` zuerst geprueft, ueber `tab_slots`) -- und der Rueckfall aufs
Fenstermenue nimmt jetzt nur `ziel == -1` (mit `< 0` oeffnete das
Reiter-Menue ueberall im Fenster; vom Test gefunden). Ein gebundenes Menue
eines Textbereichs oeffnet NICHT in der Nummernspalte (`ta_rand_zeile`), und
das Widget bekommt seinen Rechtsklick trotzdem (`Gui::kontext_neu` --
vorher setzte der Rechtsklick die Marke nicht, und "Haltepunkt" aus dem
Menue traf die alte Zeile). `GUI_FILETREE_SELECT` rollt die Zeile ins Bild.
Die IDE: `ctxCode` wandert je Bild an das Code-Feld unter der Maus
(`kontextNachziehen`, GUI_HIT_TEST), Eintraege ueber eine eigene Liste
(`ctxEintrag`) statt der Menue-Tafel; `reiterSchliessenAusser` laesst
Ungesichertes offen; `imBaumZeigen` (Strg+Alt+B). Tests
`werkzeug_ide_kontextmenues.dhtest` (4, echte Rechtsklicks, Menue per
Anfangsbuchstabe), drei Faelle in `gui_menue_bedienung.dhtest`, einer in
`gui_dateibaum.dhtest`; vier Verfaelschungen der IDE fallen je in ihrem Fall.

**Typ-Hinweise und eigene Farben (2026-09-30):** neu **`CODE_TYPES$(quelle$
[, basis$])`** -> JSON `[{zeile, name, typ}]` fuer Stellen ohne
geschriebenen Typ -- die Variablen von FOR EACH (`DIM x = wert` ohne `AS`
gibt es nicht). Gesammelt in `stmt_foreach` ueber einen Thread-Speicher
`TYP_HINWEISE` (compiler.rs, `typ_hinweise_sammeln`), weil der Weg ueber
`check_source` keine Einstellungen durchreicht; `lsp::typ_hinweise` rechnet
auf Pufferzeilen um und laesst Importe weg. Elementtyp aus `typ_von` des
Behaelters (Feld -> T, MAP -> STRING bzw. Paar STRING + T, Text -> STRING,
sonst nichts). Die IDE haengt sie in `hinweiseZeigen` an (`F_TYP`,
`typ_hinweise`). **Eigene Farben:** `farben.json` neben der ide.json, je
Thema Code-Farben, Bereichstoene (`TON_*` sind dafuer Variablen) und
`oberflaeche` = GUI_THEME_SET-Schluessel; `themaAnwenden` ist die EINE
Stelle fuer Umschalten, Start und Sichern der Datei. Tests
`werkzeug_ide_typen_farben.dhtest` (3), ein Fall in `ide_bausteine.dhtest`;
fuenf Verfaelschungen der IDE fallen je in ihrem Fall.

**Projektbaum-Dateien, Zurueck/Vorwaerts, Ausgabefilter (2026-09-30):**
neu in der Laufzeit **`GUI_CONTEXT_WIDGET(menu, wdg)`** -- ein
Kontextmenue gehoert einem Widget (`Menu::ziel`, -1 = Fenster) und oeffnet
nur beim Rechtsklick darauf; gebundene gewinnen vor dem des Fensters, und
der Rechtsklick erreicht das Widget im selben Bild (Liste/Baum/Tabelle
waehlen ihre Zeile -- `baumAbfragen` oeffnet darum bei `MOUSE_HIT(1)` nichts).
Die IDE: Kontextmenue am Baum (`baumFragen`/`baumAusfuehren`/
`baumUmbenennen`/`baumLoeschen`); Umbenennen sucht die IMPORTs VOR dem
`RENAME` (`importsSuchen`, `SAMEFILE` fragt die Platte) und zieht
Reiter/Haltepunkte/Lesezeichen/Startargumente ueber `wegUmziehen$` mit.
**Zurueck/Vorwaerts** beobachtet die Marke je Bild (`navNachziehen`) statt
an den ~30 Sprungstellen: Dateiwechsel oder >= 8 Zeilen bei gleicher
Textlaenge, ohne Bild auf/ab/Pos1/Ende; `navSperre` haelt den eigenen
Sprung heraus. Ausgabe: Filterfeld (`GUI_LISTBOX_FILTER`) und Kontextmenue;
`ausgabeSichernNach` schreibt die ANGEZEIGTEN Zeilen. Tests
`werkzeug_ide_baum_navigation.dhtest` (6, der erste mit echtem Rechtsklick
und Menue per Tastatur), zwei Faelle in `gui_menue_bedienung.dhtest`; fuenf
Verfaelschungen der IDE fallen je in ihrem Fall.

**Fehlertext am Zeilenende, Start je Datei, Neustart beim Sichern
(2026-09-30):** `GUI_TEXTAREA_HINTS(ta, zeilen, texte, farben)` nimmt als
viertes Argument auch ein FELD (eine Farbe je Hinweis, -1 = die gedaempfte
Vorgabe; `Widget::hinweise_farben`), und `PROCESS_START` breitet ein Feld
zu einzelnen Argumenten aus und nimmt eine MAP als Umgebung NUR des Kindes
(`Prozess::starten_mit`) -- SETENV bliebe fuer jeden spaeteren Start
stehen, ein "unset" gibt es nicht. Die IDE: `pruefen` sammelt Fehler und
Warnungen je Zeile (`fehlerSammeln`, gekuerzt auf 90 Zeichen, ein Fehler
faerbt rot), `hinweiseZeigen` stellt sie VOR Zaehler/Profil/Blame und gibt
die Farben mit (`fehler_am_ende`); Argumente/Umgebung je Datei in der
ide.json als Liste `start` (Pfad als Schluessel ginge wegen der
Punkt-Notation nicht), zerlegt von `startWoerter`/`startUmgebung`, an
`prozessStartenDatei`; `neustart_beim_sichern` stoppt in `sichern` ein
laufendes F5-Programm und startet ueber `neustartVomSichern` DIESELBE Datei
ohne Reiterwechsel (nicht beim Autosichern, nicht Debugger/Profil). Tests
`werkzeug_ide_fehlertext_start.dhtest` (5, Farben am Bild), je zwei Faelle in
`ide_bausteine.dhtest` und `gui_textarea_hints.dhtest`; vier Verfaelschungen
der IDE fallen je in ihrem Fall.

**Profil im Code, englische Oberflaeche (2026-09-29):** neu in der
Laufzeit **`GUI_TEXTAREA_LINE_COLORS(ta, zeilen, farben)`** +
`_GET` -- ein Band hinter ganzen Zeilen (Deckkraft im obersten Byte, ohne
sie 0x40), getrennt von den Marken, wandert mit (`zeilenfarben` in
`marken_abgleichen`), gezeichnet UNTER den Marken. Die IDE setzt es nach
`profilAnzeigen` (`waermeFeld`/`waermeAnwenden`, gelb->rot nach Anteil an
der teuersten Zeile, Zahlen an den 30 teuersten ueber `hinweiseZeigen`, das
Aufrufzaehler, Blame und Profil zusammenfuehrt -- `zaehlerSetzen_` nimmt
darum nicht mehr ALLE Hinweise weg). **Englisch:** jeder Anzeigetext geht
durch `tr$("...")` (752 Texte, gewickelt per Skript je Aufrufart:
Menuepunkt `tr$`+`kz$`, Palette/Werkzeugleiste in ihrer Hilfsroutine,
sonst jeder Text in Anzeigezeilen); Kennungen aus Kleinbuchstaben und
Code-Vorlagen sind ausgenommen. Tabelle `ide/sprache/en.txt` (TSV, **kein
JSON** -- ein Punkt im Schluessel waere ein Pfad), Zeilen ohne Tabulator
sind Kommentare (am `#` allein zu gehen verlor "# Tastenkuerzel der IDE").
`kz$` schreibt Kuerzel englisch, `kuerzelForm$` kennt beide Namen.
Gewechselt wird beim naechsten Start. Installer und Pakete nehmen
`ide/sprache/*.txt` mit. Tests `gui_zeilenfarben.dhtest` (2, am Bild),
`werkzeug_ide_profil_waerme.dhtest`, `werkzeug_ide_englisch.dhtest` (4, darunter
Vollstaendigkeit der Tabelle und ein echtes Ctrl+N); vier Verfaelschungen
fallen je in ihrem Fall.

**Quellcodeverwaltung und Live-Vorschau in der IDE (2026-09-29):**
Strg+Alt+G (`qvZeigen`/`qvAuffrischen`/`qvNachziehen`): `git status
--porcelain=v1 -b -uall` im Wurzelordner des Repositorys, das Kaestchen ist
die Index-Spalte (umgelegt = `git add` bzw. `git restore --staged`),
Uebergeben mit `git commit -m`, Verwerfen mit `git restore --staged
--worktree` nach Rueckfrage (nicht bei ungesichertem Reiter, nicht bei
neuen Dateien), pull/push als Prozess `gitProz` mit Ausgabe. Strg+Alt+L
(`liveNachziehen`): 0,8 s nach der letzten Aenderung `dhrt bild` ueber eine
versteckte Kopie neben der Datei, Bild im Fenster `winLive`, Meldung mit
dem echten Dateinamen statt der Kopie. **Falle:** der Vorschauordner ist
bei relativem `DH_IDE_KONFIG` relativ, und `dhrt bild` wechselt in den
Ordner des Programms -- das PNG kam dort an, `livePng$` macht den Weg
absolut (dieselbe Falle wie bei den Kachelbildern in Stufe 35). Tests
`werkzeug_ide_git.dhtest` (Repository im `--- vorher`, Gegenprobe ohne
`git add` faellt), `werkzeug_ide_live.dhtest` (Farbe des Bildes, Kopie
danach weg, Meldung ohne Fenster).

**Pruefsammlungen im Ueberblick (2026-09-29):** Strg+Alt+T,
`testUebersichtZeigen`/`-Fuellen`: Baum aller .dhtest mit Punkt je Fall
(IMAGE_FILL_ELLIPSE als Sinnbild). Der Stand kommt aus der Ausgabe von
`dhrt test` (`testZeile`): eine `ok`/`FEHLER`-Zeile setzt alle gelaufenen
Faelle der Datei auf bestanden, `FEHL`/`uebersprungen  Zeile N` nehmen es je
Fall zurueck (Zeile N = die `===`-Zeile). Laeufe gehen jetzt ueber eine
Warteschlange (`testLaufStarten(ziele, faelle)`, "Fehlgeschlagene" = je
roter Fall ein `--fall`). **Falle:** `dhrt test` nennt den Weg, wie er ihm
gegeben wurde, auch als 8.3-Kurzname -- der Schluessel geht ueber
`REALPATH$`. Beim Nachsehen im Bild: ein SAVESCREENSHOT direkt nach dem
Lauf zeigte einen alten Stand (grau), 30 Bilder spaeter stimmte er. Test
`werkzeug_ide_testuebersicht.dhtest` (alle, Fehlgeschlagene, Fehler
behoben, keine mehr). Werte beim Ueberfahren und Protokollpunkte gab es
schon (siehe Debugger in `docs/ide.md`).

**Projekt aus Vorlage, Bild beim Ueberfahren, Asset-Vorschau
(2026-09-29):** Strg+Alt+N (`neuProjektAnlegen(art, name, ort)`: Spiel mit
erzeugtem `assets/figur.png`, Formular, Konsole mit `rechnen.dh` und
`tests.dhtest`, leer; LIESMICH.md; belegter Ordner/krummer Name = FALSE
mit Meldung). Bild beim Ueberfahren (`bildPfadBei$` sucht die
Zeichenkette unter der Maus, `bildTippNachziehen` zeigt `winBildTipp` nach
TIPP_RUHE Bildern neben dem Zeiger). `dateiOeffnen` gibt Bilder und Klaenge
an `assetZeigen` (Vorschau-Fenster) statt an OPENDOC. **Der Fund dabei:**
ein Fenster, das per `GUI_WINDOW_VISIBLE` erscheint, ohne dass ein Widget
darin den Fokus bekommt, liegt HINTER dem Hauptfenster -- die Proben mit
"gezeigt = TRUE" waren gruen, zu sehen war nichts. Der Bild-Tipp und die
Live-Vorschau (aus #292, dort unbemerkt) werden darum nach `GUI_DRAW` mit
`GUI_DRAW_WINDOW` noch einmal obenauf gezeichnet, die Asset-Vorschau holt
sich den Fokus. Beide Proben lesen jetzt die Farbe vom BILDSCHIRMFOTO
(einige Bilder nach dem Setzen -- im selben Bild zeigt es den alten
Stand); ohne das Zeichnen obenauf fallen sie. Tests
`werkzeug_ide_neues_projekt.dhtest` (uebersetzt jedes main.dh, laeuft die
Pruefsammlung und das Konsolenprogramm), `werkzeug_ide_asset.dhtest`,
`werkzeug_ide_live.dhtest` (Schirm-Probe dazu).

**Zweige, Beiseitelegen, Blame an der Zeile, Konflikte, drei Umbauten
(2026-09-29):** Git-Fenster: `qvZweigeFuellen`/`qvWechseln`/
`qvZweigAnlegen` (`git switch [-c]`), `qvBeiseite` (`stash push -u`/`pop`);
beide nur mit gesicherten Reitern (`qvReiterSauber`), danach
`qvReiterNachziehen` von der Platte. Blame an der Zeile der Marke
(`blameFuer$`: `git blame --porcelain` einmal je Datei, gemerkt bis zum
Sichern; Hinweise gehen jetzt zusammen mit den Aufrufzaehlern ueber
`hinweiseZeigen`). `konfliktLoesen(1|2|3)` ersetzt den Block um die Marke in
EINEM Schritt (SELECT + INSERT ganzer Zeilen). Umbauten: `variableEinfuehren`
(Typ ueber `typRaten$`), `variableEinsetzen` (verweigert bei zweiter
Belegung, BYREF-Uebergabe an ein EIGENES Unterprogramm -- `hatByref`; an
PRINT & Co. uebergeben ist harmlos, die erste Fassung lehnte auch das ab --,
Aufruf im Ausdruck bei mehreren Stellen, geaenderter Variable dazwischen;
ersetzt ueber `wortErsetzen$`, weil CODE_RENAME$ keinen Ausdruck als neuen
Namen nimmt), `subInFunction` (genau ein BYREF, nicht vor dem Setzen
gelesen, keine Methode; Aufrufe als Anweisung im ganzen Projekt ueber
`sfAufrufZeile$`). **Falle:** die stille Vorab-Pruefung ohne Vorschau
prueft eine Datei, die eine andere umgebaute IMPORTiert, gegen deren
ALTEN Stand auf der Platte -- dann geht die Vorschau mit einem Scheinfehler
auf; die Probe bestaetigt dort. Tests `werkzeug_ide_umbauten_variable.dhtest`,
`werkzeug_ide_git_zweige.dhtest`.

**Standard an (M5):** Maschinencode ist Vorgabe, `DHRT_JIT=aus` laesst
alles in der VM; die CI faehrt beide Wege (Hauptlauf mit, zweiter Lauf
`DHRT_JIT=aus`). **Wer einen Fall schreibt, der die VM allein meint, setzt
`DHRT_JIT=aus` in `--- umgebung`** (so die VM-Haelfte von `jit.dhtest`).
macOS: die App wird mit `installer/posix/dhrt.entitlements` signiert --
Cranelift macht Speicher per mprotect ausfuehrbar (kein MAP_JIT), dafuer
braucht Hardened Runtime `allow-unsigned-executable-memory`, `allow-jit`
reicht NICHT.

## vm.rs und gui.rs in Teilen (2026-11-08)

`vm.rs` (15 300 Zeilen, davon ein `impl Vm` mit 12 900) und `gui.rs`
(23 600, ein `impl Gui` mit 19 000) sind nach Themen zerlegt: die Typen und
der Kern bleiben (`vm.rs` 4 500 Zeilen: Zustand, exec, dispatch, Koroutinen,
OOP, Rueckrufe; `gui.rs` 6 000: Typen, Konstruktion, Fenster-Flags,
Widget-Konstruktoren), die Methoden stehen in `vm_*.rs` (7 Dateien) und
`gui_*.rs` (14 neue neben den alten sechs). Nur verschoben -- ein Skript hat
ganze Abschnitte an ihren Kommentar-Kopfzeilen ausgeschnitten und in den
Teilen ein nacktes `fn` zu `pub(super) fn` gemacht; dazu `pub(super)` an den
Feldern von `ObjektVorlage` und an `LEERE_LAGE`. Die zwei Riesen bleiben
ganze Funktionen: `try_graphics` (3 100 Zeilen, `vm_grafik.rs`) und
`try_gui_innen` (1 600, `vm_gui.rs`).

Belegt: dieselben Warnungen vor und nachher (mit und ohne Features), 670
`#[test]` vorher wie nachher, alle 658 gelaufenen gruen, `--schnell` gruen
bis auf zwei GTK-Proben von Glutfaden (`stufe15` Lineal-Marke, `stufe22`
Ebenen-Stelle), die unter Last auch mit der Laufzeit von vorher fallen (5
von 6 Laeufen bei sechs gleichzeitigen). Tempo (`bench_dhrt.dh`, je drei
Laeufe): ohne Maschinencode alles gleich; mit ihm `familien.dh` 15,7 ->
17,5 ms (ein `ZEIT_WOCHENTAG` je Runde, rund 9 ns mehr je Aufruf) und
`felder.dh` 12,6 -> 11,3 ms, der Rest gleich -- die Lage des Codes nach dem
Linken, nicht ein anderer Weg (`lto = true` fuegt ohnehin alles zusammen).
