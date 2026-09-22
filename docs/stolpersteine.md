# Drachenhauch — Stolpersteine & Inkonsistenzen (beim Schreiben des Lehrbuchs aufgefallen)

Sammlung von Sprach-/Engine-Reibungspunkten, die beim Verfassen von
`buch-referenz/` (alle Kapitel + Module gegen `dhrt` verifiziert) auftauchten.
**Nicht das Buch, sondern Drachenhauch selbst** betreffend — als Backlog zum
späteren Beheben. Jeder Punkt ist gegen den aktuellen `dhrt`-release-Build
reproduziert.

Reihenfolge ≈ nach Nutzen/Aufwand. Erledigtes am Ende.

---

## A — Echte Lücken / häufige Stolpersteine (lohnt sich)

### A1. Kein Array-Literal `[1, 2, 3]`  —  ✅ BEHOBEN (commit 5885dc0)
> Parser disambiguiert jetzt: FOR nach dem ersten Ausdruck = List-Comprehension,
> sonst Array-Literal. Opcode `BUILD_ARRAY` (117), Element-Typ aus den Werten
> (int/float/string/boolean/any). Leeres `[]` -> Hinweis auf `DIM ... AS ARRAY OF T`.
> Tests: `tests/pruef/array_literal.dhtest` (`dhrt test`). (Historischer Text unten zur Doku.)
```basic
DIM a AS ARRAY OF INTEGER
a = [1, 2, 3]      ' -> Parse-Fehler (7): Erwartet FOR in List-Comprehension
```
`[...]` ist ausschließlich List-Comprehension. Es gibt **keine** Kurzschreibweise,
ein Array mit Werten zu füllen — man muss `DIM a[3]` + Einzel-Zuweisungen nutzen.
Tupel haben `(1, 2, 3)`, Arrays nichts Vergleichbares. Sehr häufige Erwartung von
Einsteigern.
**Vorschlag:** Im Parser disambiguieren — folgt nach `[` kein `expr FOR …`,
als Array-Literal werten (`[1,2,3]` → `ARRAY OF INTEGER`, Typ aus den Elementen).

### A2. `NIL` ist kein Literal  —  ✅ BEHOBEN (commit f4c8b78)
> NIL ist jetzt ein Keyword-Literal (lexer/parser/compiler in dhrt + Python-Front-End
> fuer Editor/Paritaet). `x = NIL`, `x <> NIL`, `IS_NIL(NIL)` funktionieren; das in der
> db-Doku versprochene NIL→NULL-Binding klappt nun wirklich. Tests: `tests/pruef/nil_literal.dhtest`.
```basic
IF o = NIL THEN ...        ' -> Laufzeitfehler: Variable 'nil' nicht deklariert (DIM fehlt?)
IF o <> NIL THEN ...       ' dito
PRINT IS_NIL(NIL)          ' dito
```
`NIL` lässt sich nirgends ausschreiben, obwohl ältere Doku/Beispiele `<> NIL`
verwenden. Folgen quer durchs Buch:
- `net`: `NET_TCP_ACCEPT`-Leere nur per `IS_NIL(x)` prüfbar, nicht `x <> NIL`.
- `db`: dokumentiertes „NIL → NULL"-Binding geht nicht (man muss die Spalte im
  INSERT weglassen).
**Vorschlag:** `NIL` als echtes Keyword-Literal einführen (mindestens in
`=`/`<>`-Vergleichen und als Argument). Macht `IS_NIL` halb überflüssig und
das db-NULL-Binding möglich. Größter „Vertrag-vs-Realität"-Punkt.

---

## B — Irreführende Fehlermeldungen (klein, hohe Wirkung)

### B1. „Stufe 3e: DIM-Typ 'vec2' noch nicht unterstuetzt" bei fehlendem IMPORT  —  ✅ BEHOBEN (commit f4c8b78)
> Jetzt: `Unbekannter Typ 'vec2' -- fehlt IMPORT "vec2"?` (bei mehreren Modulen werden
> alle Kandidaten genannt). Kein „Stufe 3e"-Leak mehr in der DIM-Typ-Meldung.
> `preprocess::modules_for_type` + `compiler::unknown_dim_type_msg`. Tests: `tests/pruef/dim_type_error.dhtest`.
```basic
DIM v AS VEC2              ' ohne vorheriges IMPORT "vec2"
' -> Compile-Fehler: Stufe 3e: DIM-Typ 'vec2' noch nicht unterstuetzt
```
Zwei Probleme:
1. **„Stufe 3e"** ist Compiler-interne Phasen-Benennung (Front-End-Port) und
   gehört nicht in eine User-Meldung.
2. **„noch nicht unterstuetzt"** ist sachlich falsch — der Typ existiert, nur das
   `IMPORT "vec2"` fehlt (externe Typen werden zur Compile-Zeit aus den Imports
   aufgelöst).
**Vorschlag:** z. B. `Unbekannter Typ 'vec2' — fehlt IMPORT "vec2"?`. Gilt für
alle externen Modul-Typen (astar_grid, json_handle, …). Generell mal alle
user-facing Meldungen nach „Stufe Nx"-Lecks absuchen.

---

## C — Doku-/Vertrags-Abweichungen

### C1. `NET_UDP_LAST_FROM` liefert STRING, nicht TUPLE  *(Doku diese Session gefixt)*
dhrt gibt `"host:port"` als STRING zurück (so auch der Golden-Test
`tests/pruef/modules_net.dhtest`), die alte `docs/module-net.md` versprach ein
`TUPLE (host, port)`. **Doku wurde an die Realität angeglichen.**
**Optional offen:** Für Ergonomie könnte die Engine stattdessen ein echtes
Tupel liefern (dann `peer[0]`/`peer[1]` direkt) — würde Test + Doku-Rückbau
verlangen. Aktuell konsistent, also kein Muss.

---

## D — Bewusste Design-Entscheidungen, die trotzdem überraschen

(Vermutlich Absicht; hier nur dokumentiert, falls man sie je überdenken will.)

### D1. `/` liefert mal INTEGER, mal FLOAT  —  ✅ ENTSCHIEDEN: nicht-brechend (commit 2492848)
> Autor-Entscheidung (2026-06-14): **`/` bleibt wie es ist** (glatt → INTEGER, sonst FLOAT;
> kein `4.0`-Bruch, Buch unverändert). Stattdessen weist die Fehlermeldung beim Zuweisen
> eines FLOAT-`/`-Ergebnisses an eine INTEGER-Variable jetzt auf `\` (Ganzzahl-Division)
> bzw. `INT()/ROUND()` hin — der eigentliche Schmerzpunkt ist damit adressiert, ohne
> Kompatibilität zu brechen. Test: `tests/pruef/div_and_float_display.dhtest`.
```basic
PRINT 8 / 2     ' 4     (glatt -> INTEGER)
PRINT 9 / 2     ' 4.5   (nicht glatt -> FLOAT)
PRINT 6 / 3     ' 2     (INTEGER)
```
Der **Ergebnistyp von `/` hängt vom Laufzeitwert ab**. Das erschwert statische
Typ-Annahmen und überrascht. Konsistenter (und in vielen Sprachen üblich): `/`
immer FLOAT, `\` für Integer-Division (existiert bereits). Bewusst so gewählt —
aber ein echter Stolperstein.

### D2. Float-Ausgabe zeigt volle f64-Präzision  —  ✅ TEILFIX (commit 2492848)
> Allgemeine Float-Ausgabe ist korrekt (kürzeste round-trip-Form) und bleibt. Der einzige
> echte Wart — f32-gestützte **Audio-Lautstärken** (`0.800000011920929`) — ist gefixt:
> `AUDIO_BUS_GET_VOLUME`/`AUDIO_GET_VOLUME`/`AUDIO_MUSIC_GET_VOLUME` runden auf 6 Stellen
> → `0.8`. Test: `tests/pruef/div_and_float_display.dhtest`. (Analyse unten.)
> **Befund nach Prüfung:** dhrts `fmt_float` nutzt bereits die *kürzeste round-trip-fähige*
> Darstellung (Rust-Display ≈ Python-`repr`). `0.1+0.2 -> 0.30000000000000004` und
> `CURVE_SMOOTHERSTEP -> 0.16308000000000003` sind die **kürzest mögliche exakte** Form —
> kürzer ginge nur falsch (würde auf einen anderen f64 runden). Also KEIN allgemeiner
> Formatierungs-Bug. **Einziger echter Wart:** f32-gestützte Werte, die zu f64 verbreitert
> werden (z.B. `AUDIO_BUS_GET_VOLUME(0.8) -> 0.800000011920929`). Falls gewünscht: nur dort
> gezielt fixen (Bus-Volume als f64 halten oder beim Lesen runden). Der allgemeine Float-Druck
> bleibt wie er ist.

### D2-alt (historischer Text):
```basic
PRINT 0.1 + 0.2                 ' 0.30000000000000004
PRINT CURVE_SMOOTHERSTEP(0.0,1.0,0.3)  ' 0.16308000000000003
```
Plus ein **f32→f64-Sonderfall**: `AUDIO_BUS_GET_VOLUME` nach `AUDIO_BUS_VOLUME(_, 0.8)`
liefert `0.800000011920929` (Bus-Volume wird intern als f32 gehalten, als f64
gedruckt).
**Vorschlag:** (a) Allgemein „shortest round-trip"-Float-Formatierung beim PRINT
(ryu/grisu) — `0.1+0.2` bliebe zwar `0.30000000000000004` (kürzeste exakte), aber
viele Werte würden kürzer. (b) f32-Rückgaben (Audio-Volumes) vor der f64-Anzeige
runden, dann `0.8` statt `0.800000011920929`.

### D3. `MID$` und `INSTR` sind 0-basiert
Untypisch für BASIC (klassisch 1-basiert). `INSTR` liefert `-1` bei „nicht
gefunden" (nicht `0`). Konsistent mit der 0-Index-Linie der Sprache, aber für
BASIC-Umsteiger eine Falle. (Ändern würde alles brechen — eher nur klar
dokumentieren.)

---

## E — Kleinere Inkonsistenzen

### E1. Hardware-Module sind importierbar, aber im Standard-Build tot  —  ✅ BEHOBEN
> dhrt warnt jetzt **schon beim IMPORT** statt erst beim ersten Funktionsaufruf.
> Auf Hardware-Module (`serial`/`usb`/`bt`/`wifi`) ohne das passende Cargo-Feature
> reagiert der Default-Build zweifach: (a) `dhrt --check` liefert eine
> `severity:"warning"`-Diagnose auf der IMPORT-Zeile → der Editor markiert das live
> beim Tippen; (b) `dhrt run` druckt die Warnung vor dem Lauf auf stderr. Der
> spätere Laufzeitfehler beim tatsächlichen Aufruf bleibt (gleicher Wortlaut,
> via `vm.rs::unknown_builtin_msg`) — die Nutzung schlägt also weiterhin fehl,
> aber der User erfährt es sofort statt erst tief im Programm. Bewusst *nicht*
> im Standard-Build mitgeliefert (zieht schwere Deps: tokio/btleplug/hidapi/
> windows). `preprocess.rs`: `missing_hardware_modules` / `missing_hardware_imports_with_lines`
> / `hardware_missing_msg`; verdrahtet in `main.rs` `compile_source` (run) +
> `check_source` (Editor). Test: `tests/pruef/dhrt_check.dhtest` (Fall "ein hardware-import warnt schon beim import").

`IMPORT "wifi"` (serial/usb/bt ebenso) wird vom Preprocessor akzeptiert (stehen
in `KNOWN_MODULES`), aber **jeder** Funktionsaufruf wirft erst zur Laufzeit
„… Hardware-Modul 'wifi', das in diesem dhrt-Build fehlt. Neu bauen mit:
python rust\build_runtime.py --hardware". Der IMPORT gelingt, die Nutzung nicht.
(Historischer Text.) **Vorschlag:** entweder im Standard-Build mitliefern, oder
schon beim IMPORT (statt erst beim ersten Call) warnen.

### E2. Reservierte Wörter als Variablennamen (über `step` hinaus)
`FOR … STEP` macht `step` zum Keyword. Genauso reserviert und für Grafik-/
Spiel-Coder überraschend: **`band`** (= Bitwise-AND `BAND`), sowie die Typ-/
Operator-Wörter `image`, `sound`, `map`, `mod`. Ein `DIM band AS INTEGER`
schlägt fehl. Klein, aber die **Fehlermeldung verrät den Grund nicht** (siehe
E3). **Vorschlag:** im Tutorial/Anhang eine Liste „diese Namen nicht als
Variable verwenden" führen.

### E3. Irreführende Meldung bei reserviertem DIM-Namen — ✅ BEHOBEN
> `DIM band` (beide Parser) meldet jetzt: „**'BAND' ist ein reserviertes Wort und
> kann kein Variablenname sein — waehle einen anderen Namen**" (statt
> „Erwartet Variablenname nach DIM"). Gilt für jedes Keyword nach `DIM`
> (`STEP`, `MOD`, `MAP`, `IMAGE`, …). `keyword()` im Rust-Lexer ist dafür
> `pub(crate)`, der Python-Parser nutzt `KEYWORDS`.

`DIM band AS INTEGER` warf früher „Erwartet Variablenname nach DIM" — sagte aber
nicht, dass `band` ein reserviertes Wort ist. (Historischer Text.)

---

## G — Befunde beim Schreiben der „VORTEX"-Demo (examples/119, 2026-06-23)

### G1. `FLT()` fehlte in dhrt + `--check` schwieg — ✅ BEHOBEN (Builtin **und** systemisch)
> **Teil 1 (Builtin):** `FLT` ist jetzt im dhrt-Kern (`builtins.rs`: `"flt" =>
> Value::Float(need_num(...))`, nach `INT`) + im `builtin_index.json`. `FLT(7)/2
> -> 3.5` nativ verifiziert.
> **Teil 2 (systemisch — der eigentliche „nie wieder"-Fix):** Der Rust-Compiler
> prüft jeden Builtin-Aufruf gegen den maßgeblichen `builtin_index.json` (per
> `include_str!` eingebettet, `compiler::is_known_builtin`). Unbekannte Builtins
> (Tippfehler ODER nur-Tree-Walker wie früher FLT) ergeben jetzt eine **nicht-
> fatale Compile-Warnung mit Zeile** → `dhrt --check` zeigt sie im Editor (gelbe
> Schlängellinie), `dhrt run` auf stderr. Kein Blockieren (Warning, kein Error)
> und keine False-Positives: Sweep über ALLE examples = 0 fälschlich gemeldete
> Builtins (der Index ist vollständig). Interne `__`-Builtins ausgenommen.
> Folge: ein neues dhrt-Builtin, das man im `builtin_index.json` zu ergänzen
> vergisst, fällt ab sofort auf. **Drift-Schutz-Test**
> `tests/pruef/dhrt_check.dhtest` prüft, dass KEIN Beispiel ein Builtin
> nutzt, das nicht im Index steht — er fand sofort **10 echte, aber unindizierte
> Builtins** (`CAMERA_ORBIT`, `WORLD_TO_SCREEN_X/Y`, `SCREEN_TO_WORLD_DIR_X/Y/Z`,
> `RAY_HIT_MODEL`, `PICK_MODEL`, `GETPIXEL`, `CIRCLEOUTLINE` — alle aus
> „in-die-Runtime-ergänzt, Index vergessen"-Commits) und sie wurden nachgetragen
> (Index: 1011 → 1021). **Hinweis:** der Index wird per `include_str!` zur
> dhrt-BAU-Zeit eingebettet → nach Index-Änderung dhrt neu bauen. Weitere Tests:
> `test_unknown_builtin_warns` / `test_known_builtin_no_warning`.

**Ursprünglicher Bericht (2026, zwei Pfade):** `FLT(x)` — z. B.
`FLT(MOUSEX())`, in mehreren examples genutzt — lief im damaligen Python-
Tree-Walker, warf in der nativen Runtime aber **zur Laufzeit** „Builtin 'FLT'
im Rust-Kern noch nicht verfuegbar". `dhrt --check` meldete es nicht: der
Compile war grün, der Fehler kam erst beim Lauf. Workaround damals: `x * 1.0`.

Von den beiden Problemen ist das erste — Builtin-Parität zwischen zwei Pfaden —
mit dem Tree-Walker verschwunden. Das zweite hat es überlebt und war das
wichtigere: eine Diagnostik, die Aufrufe unbekannter Builtins durchwinkt, ist
auch mit nur einer Runtime gefährlich, weil sie den Fehler vom Compile in den
Lauf verschiebt. Genau das behebt Teil 2 oben.

### G2. Kein vertikales Spiegeln von Text / Render-Targets — ✅ BEHOBEN (RT-Flip)
> `RENDERTARGET_DRAW(rt, x, y[, scale[, tint[, flip_v]]])` hat jetzt ein optionales
> 6. Argument `flip_v`: `TRUE` zeichnet das Target vertikal gespiegelt
> (`graphics.rs` `RtDraw` nutzt dann die positive statt negativer Quell-Höhe — die
> raylib-RTs sind ohnehin y-gespiegelt, der Mirror-Fall ist also quasi gratis).
> Damit gehen echte Boden-Reflexionen: Text einmal in ein (transparent
> vorgecleartes) Render-Target zeichnen, dann normal + `..., TRUE` gespiegelt
> darunter stempeln. Genutzt in `examples/119_vortex.dh` (Scroller-Reflexion).

(Historischer Text:) Für einen Boden-Spiegel-Scroller braucht man eine vertikal
gespiegelte Textkopie. In dhrt gibt es dafür **keinen Weg**: `TEXT` kann nicht
flippen/rotieren; `RENDERTARGET_DRAW` clampt `scale` auf `≥ 0` (kein Flip über
negative Skalierung); `DRAWIMAGEFLIPPED(img, x, y, fH, fV)` arbeitet nur auf
**Images**, Render-Targets liegen aber in einem eigenen Handle-Raum (`graphics.rs`
`render_targets` vs. Image-Vec) — ein RT-Handle als Image durchzureichen indexiert
das falsche Objekt. **Workaround in der Demo:** gedimmte, leicht verkleinerte,
*aufrechte* Kopie unter dem Text (sieht aus wie nasser Boden), keine echte
Spiegelung. **Vorschlag:** einen Flip-Parameter an `RENDERTARGET_DRAW` (raylib
zeichnet RT-Texturen ohnehin über eine negative Source-Höhe — der gespiegelte Fall
ist quasi gratis) ODER ein `RENDERTARGET_DRAW_FLIPPED`. Dann gehen echte
Reflexionen/Mirror-Effekte.

---

## H — Befunde aus den Editor-Piloten (2026-08-31 / 2026-09-04)

### H1. `DIM red` in einem Block scheiterte, oben nicht — ✅ BEHOBEN
> **Vorbelegte Konstanten liessen sich nur auf oberster Ebene verschatten.**
> `DIM pi AS INTEGER : pi = 5` lief oben im Programm einwandfrei und warf vier
> Zeilen tiefer in einem `IF`/`WHILE`/`FOR` den Laufzeitfehler
> „CONST 'pi' kann nicht ueberschrieben werden".
>
> Betroffen war jeder vorbelegte Name: die **18 Farbnamen** (`red`, `green`,
> `blue`, `white`, `black`, `gray`, `orange`, `pink`, `brown`, `purple`,
> `cyan`, `magenta`, `yellow`, …), **alle `KEY_*`**, `pi` und `tau` — also
> genau die Wörter, die ein BASIC für Grafik und Spiele als Variablennamen
> nahelegt. In einer `SUB` trat es nicht auf (dort gibt es lokale Plätze).
>
> Drei Dinge machten es unangenehm: dieselbe Zeile lief oben und scheiterte im
> Block; **`dhrt --check` schwieg**; und die Meldung zeigte auf die Konstante
> statt auf den Variablennamen — man sucht den Fehler dort, wo keiner ist.
>
> **Ursache:** `collect_globals` (compiler.rs) lief nur über die *oberste*
> Anweisungsliste. Ein `DIM` im Block bekam deshalb keinen globalen Platz und
> wurde über seinen NAMEN angelegt — und `DECLARE_NAME` lässt einen schon
> vorhandenen Eintrag stehen, während `DECLARE_GLOBAL_SLOT` ihn ersetzt.
> Drachenhauch kennt keine Block-Gültigkeit, ein `DIM` im Block *ist* global;
> der Durchlauf steigt jetzt in die Blöcke hinab (nicht in SUB/FUNCTION/CLASS
> — die haben eigene Plätze).
>
> **Nebenertrag:** Die Kollisions-Erkennung (E3) sah bis dahin nur
> Geschwister. `CONST Modus` oben und `DIM modus` in einem `IF` fiel ihr
> durch — jetzt nicht mehr.
>
> Gefunden im Sprite-Editor (`DIM pi` in der Hauptschleife). Tests:
> `tests/pruef/name_collision.dhtest`.

### H2. `dhrt --check` meldete keinen einzigen Tippfehler im Variablennamen — ✅ BEHOBEN
> **`DIM zaehler` oben, `zaehlr = zaehlr + 1` in einer SUB** — `--check` lieferte
> `[]`, das Programm startete und druckte, und erst der Aufruf der SUB brachte
> „Variable 'zaehlr' nicht deklariert (DIM fehlt?)". Dasselbe beim Lesen eines
> unbekannten Namens und auch auf oberster Ebene. Weil `DIM` in Drachenhauch
> überall Pflicht ist, ist ein verschriebener Name **immer** ein Fehler — nur
> einer, den ein selten genommener Zweig beliebig lange verdeckt.
>
> Gefunden am Sprite-Piloten (`examples/189`, 2700 Zeilen): dort stand
> `geaendert = TRUE` in einer neuen SUB — eine Variable, die es in DIESEM
> Programm gar nicht gibt (sie stammt aus einem Schwester-Piloten).
> Herausgebracht hat es erst ein Test, der den Knopf drückte.
>
> **Ursache:** `load_var`/`store_var` (compiler.rs) haben als letzten Zweig einen
> Rückfall auf `LOAD_NAME`/`STORE_NAME`; dort landet jeder Name, den der Compiler
> nicht auflösen konnte, und gesucht wird er erst zur Laufzeit. **Drei** Wege
> enden dort, nicht zwei — `INPUT x` (emittiert `INPUT_NAME`) und das
> `READ`-Ziel (das den Zwischenspeicher für Feld-/Index-Ziele braucht und
> deshalb nicht über `store_var` geht) blieben zunächst weiter still.
>
> Warum das mehr Arbeit war, als es klingt, und wo die Prüfung bewusst grob
> bleibt, steht in `docs/sprache.md` und in `CLAUDE.md`. Der eigentliche Beleg
> ist der Lauf über **alle 384 `.dh`-Dateien des Repos** (0 Meldungen) — und
> genau der fand die zwei Fehlalarme der ersten Fassungen: `DIM x[N] AS T` und
> `CONST` innerhalb einer SUB. Er ist deshalb inzwischen ein Test
> (`tests/pruef/dhrt_check.dhtest`), nicht mehr ein Aufruf von Hand.
>
> Tests: `tests/pruef/check_unbekannte_namen.dhtest`, `tests/pruef/dhrt_check.dhtest`.

---

### H3. `deg = DEG(winkel)` brach zur Laufzeit ab — ✅ BEHOBEN
> **Eine Variable, die wie ein Befehl heißt, verdeckte den Befehl — auch wenn
> sie ihn gar nicht verdecken KONNTE.** `DIM deg AS FLOAT : deg = DEG(w)`
> lief bis 2026-09-04 in die Meldung „'deg' ist eine Variable vom Typ FLOAT
> und kann nicht wie eine Funktion aufgerufen werden", und `dhrt --check`
> schwieg dazu. Gefunden am Beispiel 145, das deshalb `eg` statt `deg`
> schrieb. In BASIC ist `len = LEN(s)` Alltag.
>
> **Ursache:** der Compiler nahm bei jedem Variablennamen vor der Klammer
> `CALL_VALUE` (den FUNCREF-Weg), ohne den Typ zu fragen.
>
> **Regel jetzt:** ist der angesagte Typ der Variablen bekannt und kein
> FUNCREF, und gibt es einen Builtin dieses Namens, meint `NAME(...)` den
> Builtin — eine FLOAT lässt sich nicht aufrufen. Nur eine FUNCREF-Variable
> dieses Namens ruft weiter die Variable (dort könnte sie gemeint sein);
> die Meldung sagt das jetzt dazu. Die FOR-EACH-Laufvariable läuft gar
> nicht über diesen Weg und verdeckt nicht. Tests
> `tests/pruef/variable_wie_builtin.dhtest`.

## I — Umstieg aus anderen BASICs und aus Python (2026-09-21) — ✅ BEHOBEN

> Gemessen, nicht geschätzt: rund 150 kleine Programme so geschrieben, wie man
> sie aus QBasic, FreeBASIC, VB, Blitz oder Python mitbringt, jedes gegen
> `dhrt run`. Etwa dreißig davon scheiterten mit einer Meldung, die nichts
> sagte — meist „Erwartet Zeilenende" (`GOTO oben`, `LOCATE 1, 1`,
> `EXIT FOR`, `SWAP a, b`, `DIM a AS INTEGER = 5`) — oder liefen still falsch.
> Übersicht für Nutzer: [umstieg.md](umstieg.md); Tests
> `tests/pruef/umstieg.dhtest` (68 Fälle).
>
> **Still falsch, jetzt behoben:**
> - **Ein Name allein als Anweisung tat nichts.** `meineSub` (ohne Klammern)
>   lud die SUB als FUNCREF und warf sie weg — kein Aufruf, keine Meldung.
>   Jetzt ruft ein Name, der keine Variable, aber eine SUB/FUNCTION oder ein
>   Befehl ist, auf (`CLS`, `FLIP`, `meineSub`).
> - **`AND`/`OR` auf zwei Ganzzahlen** sind logisch und liefern einen der
>   Werte (`6 AND 3` = 3, `6 OR 1` = 6); in anderen BASICs bitweise. Das
>   bleibt so, aber `--check` warnt jetzt, wenn beide Seiten statisch INTEGER
>   sind, und nennt `BAND`/`BOR`.
> - **`DIM a AS ARRAY OF T` ohne Größe war NIL** — das erste `ARRAY_PUSH`
>   meldete „erwartet ARRAY". Jetzt ein leeres Feld (global, lokal, Klassenfeld).
> - **`VAL` las nur Texte, die GANZ eine Zahl sind**: `VAL("3 Äpfel")`,
>   `VAL("1e3")` und `VAL("&HFF")` waren 0 (die PDF-Leser mussten deshalb
>   „3 0 R" selbst zerlegen). Jetzt die Zahl am Anfang, mit `&H`/`&O`/`&B`,
>   `0x`/`0b` und Exponent.
> - **Die Meldung „'=' als Anweisung" zeigte auf die NÄCHSTE Zeile** (sie
>   entstand nach dem Zeilenende).
>
> **Geht jetzt:** `DIM x AS T = wert` (wird zu DIM + Zuweisung, gleiche
> Typprüfung; nicht bei Klassenfeldern), `EXIT FOR/DO/WHILE/REPEAT/SUB` (nur
> wenn es zur INNERSTEN Schleife passt, sonst Fehler; `EXIT FUNCTION` nennt
> `RETURN`), `END` allein (= `EXIT(0)`, auch in Blöcken — `block_until` hält
> an einem `END` mit Zeilenende nicht an), `END WHILE`, `SWAP a, b`, `LET`,
> `?` als `PRINT`, `INPUT "Frage"; x`, `m["k"]` lesen/schreiben, `LEN(map)`,
> Felder mit `+` verbinden, `RANDOMIZE(TIMER())`. `swap`, `let`, `exit` und
> `do` bleiben gewöhnliche Namen (kontextuell, keine Schlüsselwörter).
>
> **Meldungen, die sagen, wie es hier heißt:** GOTO/GOSUB, Zeilennummern,
> REDIM, TYPE, DEF FN, ON ERROR, LINE INPUT, PRINT USING, OPEN … AS #1,
> ELSE IF/ENDIF, MID$ als Zuweisung, DIM SHARED, `DIM a(10)`, DIM ohne AS,
> `STRING * 10`, Typzeichen `x%`, `%` als Rest, `!=`, `==`, `#`/`//` als
> Kommentar, `&` als Verbinder, XOR, `f = wert` in FUNCTION f, verschachtelte
> SUB (vorher „Stufe 3e: … noch nicht unterstuetzt"), fremde Typnamen
> (DOUBLE, LONG, BOOL …), Feld von Feldern, `RND`/`TIMER` ohne Klammern, ein
> Name mit Wert dahinter (`LOCATE 1, 1` → „in Klammern"), `INSTR(start, …)`,
> fremde Befehlsnamen (UCASE$, UBOUND, FIX, CINT …). Ein unbekannter Befehl
> heißt beim Laufen jetzt „Unbekannter Befehl" statt „im Rust-Kern noch nicht
> verfuegbar" — diese Wendung bleibt den Bauten ohne Grafik/Ton vorbehalten,
> an ihr erkennen die Prüfsammlungen einen solchen Bau. Die Tabellen liegen in
> `umstieg.rs` (rein, mit Rust-Tests), die Anweisungs-Hinweise in
> `Parser::umsteiger_hinweis` — gefragt wird für die Anweisung selbst und für
> jede Stelle hinter THEN/ELSE/`:` derselben Zeile.
>
> **Bewusst NICHT geändert** (sie brechen bestehenden Code): `MID$`/`INSTR`
> ab 0, `AND`/`OR` logisch, Befehle nur mit Klammern.
>
> **Zweite Runde (2026-09-22), 90 weitere Proben (Klassen, Texte, Felder,
> Dateien, Fehler, Schleifen):**
> - **`FOR i = 1 TO 3 STEP 0` brachte dhrt zum ABSTURZ** (Rust-Panic, index
>   out of bounds): bei fester Schrittweite ist die Richtung zur
>   Uebersetzungszeit bekannt, nur bei 0 war sie weder vor noch zurueck, und
>   der Zweig fuer die Laufzeit-Richtung las einen Schritt-Platz, den es nicht
>   gab (-1). Jetzt ein Uebersetzungsfehler. Ebenso `STEP 0.5` mit sicher
>   ganzzahliger Laufvariable (vorher: Abbruch im ersten Schritt mit einer
>   Meldung ueber Division).
> - **`READLINE` liefert am Dateiende `""`** -- genau wie bei einer leeren
>   Zeile, und es gab keinen Weg, beides zu unterscheiden. Neu `EOF(f)`.
> - **`"5" = 5` und `TRUE = 1` sind immer FALSE** (nichts wird umgewandelt).
>   Bleibt so, aber `--check` warnt, wenn beide Seiten statisch bekannt und von
>   verschiedener Art sind.
> - Geht jetzt: `READALL$(pfad)`, `CALL`, `BYVAL`.
> - Meldungen: FUNCTION ohne Klammern in einer Rechnung (vorher "erwartet
>   Zahlen, erhalten FUNCREF"), `Me`/`this`, `SUB New`, `OPTIONAL`,
>   `CATCH e AS`, `s[0] = "x"`, `TAB`/`SPC`/`ARRAY_REMOVE`.
> - Gemessen: keine der neuen Warnungen trifft eine der 447 `.dh`-Dateien des
>   Repos.

## F — Doku-Lücken & Verhaltens-Fallen (Review 2026-06-23, alle verifiziert)

### F1. `physics3d` war komplett undokumentiert + toter Link — ✅ BEHOBEN
> `docs/module-physics2d.md` verlinkte auf `module-physics3d.md`, die **nicht
> existierte** — obwohl PHYS3D voll implementiert ist (`builtins.rs`, 44 Arme:
> `PHYS3D_NEW/ADD_BOX/ADD_SPHERE/STEP/BODY_*/SET_*/…`) und in `examples/107`
> genutzt wird. **`docs/module-physics3d.md` neu geschrieben** (Signaturen,
> Quaternion-Render-Idiom über `QUAT_NEW`/`MAT4_TRS`) → Link jetzt gültig.

### F2. `SPRITE_HIT_BOX` / `SPRITE_HIT_POINT` undokumentiert — ✅ BEHOBEN
> Existieren in `builtins.rs`, standen in keiner Doku. In `module-sprite.md`
> ergänzt: `SPRITE_HIT_BOX(sp, x, y, w, h)` (AABB gegen Rechteck),
> `SPRITE_HIT_POINT(sp, x, y)` (Punkt-im-Sprite, Klick-Test).

### F3. Mehr reservierte Wörter als naheliegende Variablennamen — ✅ DOKUMENTIERT
> Verifiziert reserviert und überraschend: **`map`, `image`, `sound`, `input`,
> `file`, `data`, `read`, `in`** (neben `band`, `step`, `mod`, …). Unbedenklich:
> `value`, `key`, `count`, `index`, `name`, `type`, `result`, `size`, `pos`,
> `state`, `item`, `text`, `color`. In `sprache.md` (Variablen) als Notiz.

### F4. Array-Zugriff streng, Slicing klemmt still — ✅ DOKUMENTIERT
> `arr[5]` außerhalb → Laufzeitfehler `Index 5 ausserhalb [0..2]`. `arr[0:99]`
> auf ein 3er-Array → still auf Länge 3 geklemmt, kein Fehler. Asymmetrie jetzt
> in `sprache.md` (Arrays) erklärt. Slicing nur für 1D.

### F5. Arrays immer per Referenz — ✅ DOKUMENTIERT
> An `SUB`/`FUNCTION` übergebene Arrays teilen den Speicher; Änderungen wirken
> aufs Original (`ARRAY_COPY` für eine Kopie). In `sprache.md` (Arrays) vermerkt.

### F6. `+` mit String koppelt still (Zahl/Bool → String) — ✅ DOKUMENTIERT
> `"x=" + 42` → `"x=42"`, `"f=" + TRUE` → `"f=TRUE"`, kein Typfehler. In
> `sprache.md` (Strings) vermerkt.

### F7. `TILE_SWEEP_X/Y` geben FLOAT im Tupel — ✅ DOKUMENTIERT
> `TILE_SWEEP_X(...)` → `(new_x, hit)`, wobei `new_x` ein **FLOAT** ist. Wer
> direkt an eine INTEGER-Koordinate zuweist, braucht `INT()/ROUND()`. Rückgabe
> in `module-tile-collide.md` jetzt als `(new_x: FLOAT, hit: BOOL)` ausgewiesen.

---

## Bereits behoben (diese/letzte Sessions — nicht mehr offen)

- `ATLAS_DRAW_FLIPPED`: flip_x/flip_y nehmen jetzt TRUE/FALSE **oder** 1/0; echtes
  flip_y; tint = Arg7 (war vorher inkonsistent/kaputt). *(commit 8aa315f)*
- `PHYS3D_ADD_BOX`/`ADD_SPHERE`: dynamic-Flag akzeptiert TRUE/FALSE **oder** 1/0
  (vorher nur Zahl — konsistent mit physics2d). *(commit 3928fdd)*
- Gefüllte `TRIANGLE`/`POLYGON`: füllen jetzt wicklungsunabhängig (vorher nur bei
  CCW sichtbar — raylib-Back-Face-Culling). *(commit d68efd7)*
- `docs/module-net.md`: `NET_UDP_LAST_FROM`-Rückgabetyp an die Realität
  angeglichen (STRING statt TUPLE). *(siehe C1)*
- **A2** (`NIL`-Literal) + **B1** (irreführender DIM-Typ-Fehler) — gefixt 2026-06-14,
  commit f4c8b78 (Details bei den jeweiligen Abschnitten oben). Buch-Stellen zu „NIL
  ist kein Literal" (Kap 34/69/73 + Anhang D) entsprechend korrigiert.
- **`MOUSEBUTTON`-Doku falsch**: `builtins-grafik.md` sagte „1=mitte, 2=rechts",
  die raylib-Runtime mappt aber `0=links, 1=rechts, 2=mitte` (`graphics.rs`
  `mouse_button`). Stiller Fehlgriff bei jedem Maus-Spiel. **Doku korrigiert.**
- **`CIRCLEOUTLINE` ergänzt**: `CIRCLE` war nur gefüllt (Inkonsistenz —
  `TRIANGLE`/`POLYGON`/`ELLIPSE` haben Kontur-Varianten). `CIRCLEOUTLINE(x,y,r
  [,color])` neu in der Runtime (nutzt die Ellipsen-Kontur, kein eigener Cmd).
- **C-Stil Hex-/Binär-Literale** `0xFF` / `0b1010` zusätzlich zu `&H`/`&B`
  (beide Lexer), siehe A1. Falsche `0x`-Doku-Beispiele lauffähig gemacht.
- **Undokumentierte Grafik-Builtins dokumentiert**: `LINEW` (dicke Linie),
  `BOXROUND`/`RECTROUND` (runde Ecken), `GRADIENTH`/`GRADIENTV` (Verläufe),
  `SPLINE`, `BLEND_MODE`, `RGBA`/`ALPHA` (Transparenz) in `builtins-grafik.md`;
  `MOUSE_GROUND_X/Z/HIT` (Cursor → Boden-Ebene, der „Wohin zeigt die Maus in
  3D?"-Helfer) in `rust-runtime.md`. Waren in der Runtime vorhanden, aber für
  den User nicht auffindbar.

- **`CAMERA_ORBIT` ergänzt**: `CAMERA_ORBIT(tx,ty,tz, radius, yaw, pitch[, fovy])`
  neu in der Runtime — ersetzt die manuelle Orbit-Kugelkoordinaten-Trigonometrie
  (Pitch gegen Gimbal-Flip geklemmt, fovy optional). Math per `CAMERA3D_X/Y/Z`
  exakt verifiziert.
- **3D-Projektion + Mesh-Picking + GETPIXEL ergänzt** (Rest des 3D-Backlogs):
  `WORLD_TO_SCREEN_X/Y` (3D→2D) und `SCREEN_TO_WORLD_DIR_X/Y/Z` (Strahl-Richtung,
  Ursprung = Kamera) für allgemeine Projektion; `RAY_HIT_MODEL` + `PICK_MODEL`
  (Raycast/Picking gegen geladene Meshes, nicht nur Box/Sphere); `GETPIXEL`
  (Pixel lesen — `PIXEL` war nur ein `PLOT`-Alias). Alle vier exakt verifiziert
  (Würfel-Raycast → Distanz 4.0; Projektion Ursprung → Bildmitte; Gradient-Pixel).

> Damit ist der gefundene Grafik-Ergonomie-Backlog (2D wie 3D) abgearbeitet.
