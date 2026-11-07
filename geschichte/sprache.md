# Sprachfunktionen

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Feld-Untergrenzen, Index auf Aufrufen, `/` als FLOAT, Koroutinen, FUNCREF und Lambdas, Klassen, Comprehensions, SELECT CASE, Tupel, WITH und der Rest.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Felder mit eigener Untergrenze (2026-10-31, Stufe 1)

`DIM tage[1 TO 31] AS INTEGER`, `DIM f[-10 TO 10, 5]`, Grenzen als
Ausdruecke, `[1 TO 0]` leer. Entwurf `docs/entwurf-feld-untergrenze.md`
(Abschnitt 12 = Stand). `DhArray::unten` (value.rs; LEER = alle 0 -- die
schnellen Wege fragen nur `is_empty()`, `mit_unten` macht aus lauter Nullen
wieder leer), `flat_index` meldet `Index 0 ausserhalb [1..31] in Dimension 1`
(**die Dimension zaehlt seither ab 1**, auch bei Feldern ab 0; der Hinweis
"Felder zaehlen ab 0" nur bei Untergrenze 0). Parser: Knoten
`Node::Bereich { von, bis }` nur in DIM-Klammern (`dim_grenze`). Bytecode:
`DECLARE_ARRAY_NAME/LOCAL` mit viertem, freiwilligem Argument (je Dimension
1 = zwei Werte `von, bis`, 0 = Groesse; `Vm::pop_feld_dims`), Klassenfelder
mit `array_unten` im .dhc -- ohne Bereich bleibt beides wie vorher, alte .dhc
laufen. `LBOUND/UBOUND(feld [, dimension])` (Dimension ab 1, leeres Feld
UBOUND = LBOUND - 1). Wer eine Stelle nennt oder liefert, zaehlt in den
Indizes des Feldes: `ARRAY_INSERT/REMOVE_AT`, `ARRAY_INDEXOF/FIND` (nicht
gefunden = `LBOUND - 1`), Slicing (Ergebnis ab 0), `ARRAY_COPY` behaelt die
Grenzen. **Regel fuer neue Feldbefehle:** wer einen Index entgegennimmt oder
liefert, rechnet ueber `unten_von(k)`, sonst zaehlt er bei solchen Feldern
still falsch. **Stufe 3:** der Maschinencode rechnet solche Felder selbst --
`FeldInfo::unten` (nur dann traegt die Beschreibung die Untergrenzen, und
`feld_adresse` zieht sie vor der Pruefung ab; Felder ab 0 bekommen denselben
Code wie vorher, gegen 2026.38 A/B 0,97..0,98), `feld_passt` verlangt die
Form des Baus, `gfeld_lesen` liest die Grenze am Feld; ein Feld mit
Untergrenze laeuft ~60x schneller als in der VM.
Nicht fuer Felder von `STRUCT ... LAYOUT C`. **Stufe 2:** `ARRAY_MAP`
behaelt die Grenzen, `PRINT` zeigt `<ARRAY[1 TO 31] OF ...>`, der Debugger
zaehlt Kinder ab `LBOUND`, `--check` warnt bei festem Index ausserhalb fester
Grenzen (`Ctx::feste_grenzen`; gesammelt in `grenz_warnungen` und erst am
Ende des Bereichs ausgegeben, nur wenn das Feld dort nie ersetzt wird und
nie an einen Aufruf geht -- `grenz_frei`). Tests
`tests/pruef/sprache/feld_untergrenze.dhtest` (27).

## Index und Feld auf dem Ergebnis eines Aufrufs (2026-10-09)

`reihe(5)[0].x`, `ort(7).y`, `LEN(reihe(4))`, `FOR EACH p IN reihe(3)` --
vorher fuer Structs eine Meldung ("erst einer Variable zuweisen"), weil der
Struct-Weg den Puffer zweimal braucht (Stelle und Laenge). Jetzt kennt
`struct_ort` einen `Node::Call` als Basis (`struct_aus_aufruf`), und
`struct_basis_laden` wertet ihn EINMAL aus und merkt sich das Ergebnis in
einem anonymen Platz (`StructOrt::merk`, Cell). Und **geschrieben** werden
darf jetzt auch: Felder, Objekte UND Structs kommen als VERWEIS zurueck
(gemessen), `f()[1].x = 3` / `t()[2] = 4` / `held().hp = 5` aendern also das
Original. Der Parser nimmt dafuer Aufrufe in die linke Seite
(`is_assignment_lookahead` springt ueber Klammern, verlangt dahinter aber
noch `[` oder `.` -- `f(1) = 2` bleibt der alte Fehler; `assign_from_lvalue`
baut `Node::Call` in die Kette). `+=`/`++` darauf ist eine Meldung
(`ziel_ist_aufruf`): der Aufruf liefe zweimal. Tests
`tests/pruef/sprache/index_auf_aufruf.dhtest` (11; gegen 2026.25 fallen 8, ohne den
Zwischenspeicher die 4 Faelle, die die Aufrufe zaehlen).

## `/` liefert immer FLOAT (2026-09-24, Entscheidung des Nutzers)

`vm::div` rechnet immer in Kommazahlen: `8 / 2` ist `4.0` (vorher INTEGER 4,
der Typ hing am Wert -- fuer M2/M3 nicht vorab festzulegen). Konstanten
falten geht durch dieselbe Funktion, `typ_von` sagt FLOAT. Zuweisung an eine
INTEGER-Variable geht weiter, solange das Ergebnis glatt ist; ein Feld-Index
aus `/` bricht ab ("Array-Index muss INTEGER sein") -- **`--check` warnt vor
jedem Index, dessen Typ sicher FLOAT ist** (`Compiler::index_pruefen`). Der
volle Testlauf fand im Bestand keine Stelle, die am alten Verhalten hing;
geaendert wurden Erwartungen (`4` -> `4.0`), ein Pixel-Mittelpunkt in
`gebundene_methoden_gui.dhtest` (jetzt `\`) und der Satz im Referenzbuch
(`11_operatoren.js` samt `en.json`). Doku `docs/sprache.md`,
`docs/stolpersteine.md` D1. Tests `tests/pruef/sprache/div_and_float_display.dhtest`.

## Coroutines / YIELD

Eine `FUNCTION`/`SUB`, deren Body ein `YIELD` enthaelt, ist eine **Coroutine**.
Ihr Aufruf fuehrt den Body NICHT aus, sondern liefert ein `COROUTINE`-Handle.

```basic
FUNCTION zaehler() AS INTEGER
    YIELD 1
    YIELD 2
    RETURN 99            ' Endwert (optional), via CORO_RESULT abrufbar
END FUNCTION

DIM c AS COROUTINE
c = zaehler()
PRINT CORO_RESUME(c)     ' 1
PRINT CORO_RESUME(c)     ' 2
PRINT CORO_RESUME(c)     ' 99 (beendet) -- CORO_DONE(c) ist jetzt TRUE
```

**API** (Builtins, kein neuer Opcode ausser `YIELD_VALUE`):
- `CORO_RESUME(c)` -- fortsetzen bis zum naechsten YIELD, liefert den YIELD-Wert
  (bzw. den RETURN-Wert, wenn die Coroutine in diesem Schritt endet).
- `CORO_SEND(c, v)` -- wie RESUME, aber der `YIELD`-**Ausdruck** im Body
  evaluiert zu `v`: `DIM x AS INTEGER : x = YIELD 5`. Der Sende-Wert des
  ERSTEN Resume ist immer NIL (wie Python -- nicht lesen).
- `CORO_DONE(c)` -- BOOLEAN, ob beendet (RETURN/Ende/CLOSE).
- `CORO_RESULT(c)` -- finaler RETURN-Wert (wirft, wenn noch nicht beendet).
- `CORO_CLOSE(c)` -- suspendierte Coroutine abbauen (raeumt den Worker-Thread).
- `FOR EACH v IN coro` / Comprehensions -- treiben die Coroutine **eager** bis
  zum Ende (RETURN-Wert nicht enthalten). Vorsicht bei unendlichen Generatoren
  -- dort `CORO_RESUME`/`CORO_DONE` manuell verwenden.

**`YIELD` ist ein Ausdruck** (niedrige Praezedenz): `YIELD v` als Statement
verwirft den Sende-Wert; `x = YIELD v` liest ihn. Operand optional (`YIELD`).

**Mechanismus:** Jede Coroutine laeuft auf einem eigenen **Daemon-Thread** mit
striktem Ping-Pong (Queues; immer nur ein Thread laeuft gleichzeitig). Dadurch
bleibt die Ausgabe deterministisch und **bit-identisch** ueber Tree-Walker,
Python-VM und Cython-VM. Folgen davon:
- **Kein Cross-Frame-Yield:** ein Helfer mit `YIELD` ist selbst eine Coroutine
  (sein Aufruf liefert ein Handle), `YIELD` laeuft also nie ueber einen
  normalen Call hinweg.
- **Typ-Coercion:** in `FUNCTION ... AS T` werden YIELD- UND RETURN-Werte auf
  `T` gecoerct (ein Typ fuer beide Kanaele). SUB-Coroutinen yielden "any".
- **Idiom:** ein manueller `WHILE NOT CORO_DONE(c)`-Resume-Loop bekommt beim
  letzten (beendenden) Aufruf den RETURN-Wert; gibt der Generator einen
  typisierten `RETURN` zurueck, klappt die Zuweisung an eine typisierte
  Variable -- sonst `FOR EACH` nutzen.

**Implementierung:** in dhrt -- der Compiler markiert eine Funktion mit `YIELD`
als Coroutine, die VM legt beim YIELD einen Frame-Schnappschuss in einem
`Value::Coroutine` ab (`vm.rs`), getrieben wird ueber die `CORO_*`-Builtins.
Der frueher hier beschriebene Thread-Aufbau gehoerte zum Python-Tree-Walker
und ist mit Stufe B entfallen; die Frame-Schnappschuss-Fassung steht unten.

**Auch nativ (dhrt/Rust, `--native` + Standalone-`.exe`).** Statt Threads nutzt
die Rust-VM einen **Frame-Snapshot**: `dispatch` liefert `Step::Return | Yield`;
bei YIELD wird der Frame (ip/locals/stack/try_handlers) in einem
`Value::Coroutine` (`CoroState`) abgelegt und beim Resume restauriert. Moeglich
ist das, weil **kein Cross-Frame-YIELD** existiert -- nur der oberste
Coroutine-Frame muss fortsetzbar sein (verschachtelte Calls laufen normal
rekursiv). Kein OS-Thread -> raylib-Main-Thread bleibt sicher, deterministisch
per Konstruktion. Tree-Walker (thread-basiert) und dhrt (Frame-Snapshot) liefern
bit-identisch.

Use-Cases: Cutscene-DSL, prozedurale Generation, Boss-Patterns, NPC-Dialoge.
Doku-Demo [examples/98_coroutines.dh](examples/98_coroutines.dh), Tests
`tests/pruef/sprache/coroutines.dhtest` (Pruefsammlung fuer `dhrt test`, seit 2026-09-07 -- vorher tests/pruef/sprache/coroutines.dhtest).

## Drei Halbheiten geschlossen: MAP-Literal, `FOR EACH k, v`, `DO ... LOOP`

```basic
DIM m AS MAP OF INTEGER
m = {"a": 1, "b": 2}          ' MAP-Literal, Gegenstueck zu [1, 2]
m = {}                        ' die leere MAP

FOR EACH k, v IN m            ' Schluessel UND Wert
    PRINT k; "="; v
NEXT

DO WHILE i < 5 : i = i + 1 : LOOP
DO : i = i + 1 : LOOP UNTIL i >= 5
```

- **MAP-Literal**: der Parser trennt am `FOR` -- `{k: v FOR ...}` bleibt die
  Dict-Comprehension, alles andere ist das Literal. Kompiliert ueber dieselbe
  Sammelstelle (`__dict_from_pairs`), also KEIN neuer Opcode. `{}` war vorher
  ein Parser-Fehler und ist jetzt die leere MAP (Test entsprechend gedreht).
- **`FOR EACH k, v`**: `ForEach` hat ein zweites, optionales Feld `var2`. Bei
  zwei Variablen schiebt der Compiler den Behaelter durch das interne
  `__paare` -- eine MAP liefert dann ihre Paare statt der Schluessel, alles
  andere geht unveraendert durch (ein TUPLE aus 2-Tupeln passt also auch).
  **Die Einzelvariablen-Form bleibt bei den SCHLUESSELN** -- sie umzudeuten
  haette bestehenden Code gebrochen.
- **`DO ... LOOP`**: rein im Parser, ohne AST-Knoten -- Kopfpruefung wird zu
  `While`, Fusspruefung zu `Repeat`. Damit erben beide BREAK/CONTINUE ohne
  Zutun. **`do` und `loop` sind KONTEXTUELL** (wie `each`), keine
  Schluesselwoerter: `DIM dO AS INTEGER` gibt es in `examples/127_filedialog.dh`,
  und ein neues Keyword haette das gebrochen. `DO` zaehlt nur als Schleife,
  wenn WHILE/UNTIL/Zeilenende folgt. Bedingung oben UND unten ist ein Fehler.
- Tests: `tests/pruef/sprache/sprach_symmetrie.dhtest` (18 Faelle, `dhrt test`; bis 2026-09-07 tests/pruef/sprache/sprach_symmetrie.dhtest).

## Laufzeit-Typtest: `IS` + `TYPEOF`

`TYPEOF` liefert bei einer Instanz den **Klassennamen** (gross), nicht mehr
das pauschale `"OBJECT"`; `x IS Typname` prueft samt Vererbungskette:

```basic
DIM t AS Tier
t = NEW Hund()
PRINT TYPEOF(t)      ' "HUND"
PRINT t IS Hund      ' TRUE
PRINT t IS Tier      ' TRUE   -- jede Elternklasse trifft
PRINT t IS NOT NIL   ' TRUE
```

- Rechts von `IS` steht ein **Typname, kein Ausdruck**: Klasse, Werttyp,
  Modultyp, `NIL`, `ARRAY`, `MAP`. Ein unbekannter Name ist ein
  **Uebersetzungsfehler** -- ein Tippfehler waere sonst still fuer immer FALSE.
- `NIL` ist keine Instanz: `t IS Tier` ist FALSE, solange `t` NIL ist.
- **`CASE IS > 5` bleibt unberuehrt** -- dort verschluckt `case_match` sein
  eigenes fuehrendes `IS`, bevor die Vergleichsebene ueberhaupt drankommt.
- Umsetzung: eigener AST-Knoten `IsTyp { wert, typ }` (parser.rs) -- der
  Typname liegt in einem EIGENEN FELD, damit `namensraum.rs` ihn wie jeden
  anderen Typnamen umschreiben kann (`x IS mathe.Punkt`). Der Compiler prueft
  den Namen und emittiert `__is_typ(wert, "name")`; die Vererbungskette laeuft
  `Vm::try_typtest` (vm.rs) ab -- die kennt nur die VM, darum nicht in
  builtins.rs. Kein neuer Opcode. Tests: `tests/pruef/sprache/typtest.dhtest`.

## Function References (FUNCREF)

User-Functions als first-class Werte fuer Higher-Order-Patterns:

```basic
FUNCTION square(x AS INTEGER) AS INTEGER
    RETURN x * x
END FUNCTION

DIM f AS FUNCREF
f = square            ' bare Identifier wird zur FUNCREF
PRINT f(5)            ' 25 -- Aufruf via Variable
```

Use-Cases: Sort-Comparator, Tween-Easing-Callbacks, Event-Handler.

```basic
FUNCTION twice(g AS FUNCREF, x AS INTEGER) AS INTEGER
    RETURN g(g(x))
END FUNCTION
```

**Gebundene Methoden** (2026-08-26): `obj.methode` OHNE Klammern ist eine
FUNCREF, die ihre Instanz mittraegt -- damit koennen die Rueckruf-Schnittstellen
auf ein Objekt zeigen statt auf eine freie SUB plus globale Variable:

```basic
DIM f AS FUNCREF
f = spieler.tick          ' traegt `spieler` mit
f()                       ' ruft spieler.tick() auf
GUI_ON_CLICK(knopf, spieler.klick)
TIMER_EVERY(500, gegner.zucken)
SORT(zahlen, regel.cmp)
```

Nach aussen ist das ein ganz normales FUNCREF (`TYPEOF` sagt `FUNCREF`,
`DIM f AS FUNCREF` nimmt es auf). Details:
- **Aufgeloest wird beim AUFRUF, nicht beim Binden** -- eine als Elternklasse
  gehaltene Instanz ruft die Ueberschreibung des Kindes.
- **Ein Feld gewinnt vor einer gleichnamigen Methode** (sonst haette die
  Erweiterung bestehenden Code umgedeutet); erst wenn kein Feld passt, wird
  nach einer Methode gesucht.
- **Gleichheit** vergleicht Instanz UND Methode; eine gebundene Methode ist nie
  gleich einer freien Funktion desselben Namens.
- **`GUI_SAVE`/`GUI_TO_JSON` schreiben einen gebundenen Handler NICHT** in die
  `.dhform` -- beim Laden gibt es die Instanz nicht, und nur den Namen zu
  schreiben waere eine Luege (er wuerde als freie Funktion gedeutet).
- **`TASK_START` lehnt sie ab**: der Auftrag laeuft als eigener Prozess und
  sieht das Objekt nicht -- das sagt eine eigene Meldung im Klartext.
- Implementierung: `Value::BoundMethod(Rc<(Empfaenger, Methodenname)>)` in
  `value.rs`; erzeugt im `LOAD_MEMBER`-Fallback, ausgefuehrt in `CALL_VALUE`
  (beides `vm.rs`). Was `gui`/`timer` speichern, ist ein `Rueckruf`
  (`value.rs`): Name **und** optionaler Empfaenger -- der Name muss bleiben,
  weil `gui.rs` ihn in die `.dhform` schreibt. Gefeuert wird ueber den einen
  Helfer `Vm::rueckruf_rufen`. Tests: `tests/pruef/sprache/gebundene_methoden.dhtest`
  (inkl. echtem GUI-Klick per Automation-Wiedergabe).

**Lambdas** (seit 2026-10-04, Schreibweise von VB.NET): `FUNCTION(x) x * 2`
(ein Ausdruck) und `SUB() zaehler += 1` (eine Anweisung), beide in EINER
Zeile; Parameter ohne `AS` sind `any`. Der Compiler (`expr_lambda`) macht
daraus eine eigene Funktion `__lambda_<nr>_<kopien>` (die Nummer in
Uebersetzungsreihenfolge -- TASK_START uebersetzt dieselbe Datei und trifft
sie; die Kopienzahl zieht `vm::lambda_kopien` in der Argument-Meldung ab).
**Lokale Variablen werden beim Anlegen KOPIERT** (zusaetzliche Parameter
vorn; an der Stelle `__closure(name, werte...)` -> `Value::Closure`), weil
ein Lambda laenger leben kann als sein Aufruf; eine Zuweisung an eine Kopie
ist ein Uebersetzungsfehler. **Globals sieht es lebendig** (ein Rueckruf
soll den Stand von JETZT zeigen) -- Folge: im Hauptprogramm in einer
Schleife angelegt, sieht es die Laufvariable mit ihrem letzten Wert. Die
freien Namen sammelt `lambda_namen` ueber die JSON-Form des Baums (es gibt
keinen allgemeinen Durchlauf). In einer Methode nur ueber `Self` (der
Kontext des Lambdas hat keine `current_class`; ein nacktes Feld ist ein
Fehler). `Rueckruf::vorab` traegt die Kopien zu GUI/TIMER/SORT,
`Vm::rueckruf_exec` setzt sie vor die Argumente; `ist_gebunden` gilt auch
fuer Lambdas (kein `.dhform`-Eintrag). Dazu `ARRAY_MAP/FILTER/REDUCE/FIND`
(`Vm::array_mit_funktion`, Familie `try_array_hof`) und der Aufruf eines
WERTES (`a[i]()`, `f(1)(2)`, CALL_VALUE). Schleifen mit Lambdas bleiben in
der VM (der Maschinencode kennt CALL_VALUE nicht). Doku `docs/sprache.md`
(Funktionen als Werte), Tests `tests/pruef/sprache/lambda.dhtest`.

**Implementierung:**
- Type-Token `FUNCREF`. AST braucht keinen neuen Node -- bare Identifier
  in Expression-Position wird kontextabhaengig aufgeloest.
- Tree-Walker: `_eval_Identifier` liefert `_FuncRef(name)` wenn der Name
  keine Variable ist und in `self.functions` existiert. `_eval_Call`
  dispatched FuncRef-callees direkt.
- Compiler: `_global_vars`-Set wird vor Phase 5 gefuellt (alle Top-Level
  DIM/CONST/MultiDim). `_expr_Identifier` und `_expr_Call` checken Locals,
  Felder, Globals zuerst -- nur wenn keiner trifft, fallen sie auf
  Function-Lookup zurueck (Tree-Walker-konsistente Vorrang-Reihenfolge).
  Eine User-Variable mit gleichem Namen wie eine Function verschattet
  diese.
- Bytecode: `LOAD_FUNCREF` (53) und `CALL_VALUE` (54). LOAD_FUNCREF nimmt
  den Function-Namen aus dem const-Pool und pusht eine `_FuncRef`-Instanz.
  CALL_VALUE pop n args, pop callee (FuncRef), dispatch via `_exec`.
- VM-Pfade: gleicher Flow. `_FuncRef` aus `drachenhauch.interpreter` importiert.

**Einschraenkung Reihenfolge:** Im VM-Pfad wird `_global_vars` static aus
allen Top-Level-Statements gefuellt. Wer eine Function `foo` erst aufruft
und DANACH eine Variable `foo` deklariert, kriegt im VM einen
"Variable nicht deklariert"-Fehler bei der Function-Verwendung -- der
Tree-Walker ist hier dynamischer. Praxisrelevant ist das selten;
empfohlen: User-Variablen oben deklarieren oder anders benennen.

**Beispiel:** [examples/57_funcref.dh](examples/57_funcref.dh).

## Static Class Members

`STATIC CONST` innerhalb einer Klasse erzeugt klassen-bezogene Konstanten,
zugreifbar via `<ClassName>.<MEMBER>` -- analog zum ENUM-Pattern.

```basic
CLASS Player
    STATIC CONST MAX_HP AS INTEGER = 100
    STATIC CONST DEFAULT_NAME AS STRING = "Hero"
    DIM hp AS INTEGER
    SUB Init()
        Self.hp = Player.MAX_HP        ' Static aus Methode
    END SUB
END CLASS

PRINT Player.MAX_HP                     ' 100
```

**Werte muessen Compile-Zeit-Literale sein:** Number, String, Bool oder
negierte Number. Keine Ausdruecke -- gleiche Strenge wie ENUM-Member,
damit alle drei Pfade konsistent sind.

**Implementierung:** Klassen-Statics werden zur Klassen-Hoisting-Phase als
`_ClassStaticNamespace` gebaut und unter dem Klassen-Namen als globale
CONST registriert. MemberAccess auf das Namespace-Objekt liefert den
Member-Wert -- LOAD_MEMBER in beiden VMs erkennt den Typ explizit.

`_infer_type` bekommt `"class_static"` als neuen Type-String. `Self.hp =
Player.MAX_HP` funktioniert, weil `Player` als CONST-Variable im globalen
Scope existiert (mit dem Namespace als Wert), nicht als Klassen-Konstruktor
-- der NEW-Pfad geht ueber `self.classes[name]`, nicht ueber Identifier-
Lookup.

**Einschraenkungen:**
- Klassen mit Statics duerfen nicht den gleichen Namen wie eine andere
  globale Variable haben. (Klassen ohne Statics sind unbeeinflusst.)
- Static-Werte sind immutable nach Compile-Zeit -- es gibt kein
  `Player.MAX_HP = 200`. Wer Mutable-Class-State will, schreibt eine globale
  Variable mit "DIM ... AS Klasse".

**Beispiel:** [examples/56_static.dh](examples/56_static.dh).

## Properties (PROPERTY GET/SET)

Klassen koennen Property-Accessors deklarieren -- Member-Read und -Write
laufen dann durch User-Code statt direkt aufs Feld:

```basic
CLASS Player
    DIM _hp AS INTEGER

    PROPERTY GET hp() AS INTEGER
        RETURN Self._hp
    END PROPERTY

    PROPERTY SET hp(value AS INTEGER)
        IF value < 0 THEN value = 0
        IF value > 100 THEN value = 100
        Self._hp = value
    END PROPERTY
END CLASS

DIM p AS Player
p = NEW Player()
p.hp = 200      ' Setter laeuft -> clamped zu 100
PRINT p.hp      ' Getter laeuft -> 100
```

**Implementation:** Properties werden intern als Methoden mit den Namen
`__get_<name>` und `__set_<name>` registriert. Die Klasse merkt sich die
Property-Namen in einem `set` (`_ClassInfo.properties` /
`VMClassInfo.properties`). Im MemberAccess- und MemberAssign-Pfad wird
zuerst gegen das Property-Set geprueft, bei Treffer dispatcht der Code
zur Internal-Methode.

**Read-only / Write-only:** Wenn nur GET deklariert ist, wirft `obj.x = v`
einen Fehler. Wenn nur SET, wirft `PRINT obj.x`.

**Inheritance:** Properties werden automatisch vererbt (gleicher MRO-
Lookup wie Methoden).

**`GET`/`SET` sind keine Keywords:** Sie sind kontext-abhaengig nach
`PROPERTY`. So bleiben User-Methoden wie `FUNCTION Get() AS T` unbeeinflusst.

**Beispiel:** [examples/63_props_comp.dh](examples/63_props_comp.dh).

## List-Comprehensions

`[expr FOR var IN container]` und `[expr FOR var IN container WHERE filter]`.
Liefert ein TUPLE der transformierten Werte:

```basic
DIM evens AS TUPLE
evens = [n FOR n IN nums WHERE n MOD 2 = 0]

' Auch mit Method-Calls und Properties
DIM names AS TUPLE
names = [it.name FOR it IN cart WHERE it.price > 5]
```

**Iterable:** STRING (chars), TUPLE, 1D-ARRAY, MAP (Keys).

**Implementation:**
- AST `ListComp(var, iterable, filter, transform)`. Im Parser an LBRACKET
  in Primary-Position.
- Tree-Walker iteriert direkt ueber Python-Iterable.
- Compiler nutzt einen Marker-Singleton (`bytecode.COMP_MARKER`) und einen
  index-basierten Loop. Vor dem Loop wird der iterable durch das Built-in
  `__COMP_ITER` in ein TUPLE umgewandelt -- so funktioniert der gleiche
  LEN+Index-Mechanismus fuer alle Container-Typen.
- Neuer Op `BUILD_TUPLE_DYN=57`: sammelt alle Werte oberhalb des Markers
  zu einem Tupel.
- Iter-Variable wird als anonymer Local-Slot reserviert -- ueberlappt
  nicht mit existierenden globalen Variablen gleichen Namens.

**Bonus:** Strings haben jetzt auch normalen Index-Access (`s[0]`) -- nicht
nur Slicing. Das war fuer die Comprehension noetig und ist eine sinnvolle
generelle Erweiterung.

**Beispiel:** [examples/63_props_comp.dh](examples/63_props_comp.dh).

## SELECT CASE mit Guards

`CASE ... WHERE expr` -- der naechste Case wird probiert, wenn entweder das
Match-Pattern nicht trifft ODER der Guard-Ausdruck falsy ist:

```basic
SELECT CASE hp
    CASE IS <= 0
        ' tot
    CASE IS <= 30 WHERE has_potion
        ' low aber Trank verfuegbar -> heilen
    CASE IS <= 30
        ' low ohne Trank -> fliehen
    CASE ELSE
        ' OK
END SELECT
```

Guard-Expressions koennen auf normale Variablen zugreifen, inkl. der
Subject-Variable. Klassische Use-Cases:
- Permission-Checks: `CASE "delete" WHERE user IN ("admin", "moderator")`
- Numerische Conditions: `CASE 1 TO 100 WHERE n MOD 2 = 0`
- State-Combinationen: `CASE "save" WHERE dirty`

**Implementierung:** Compile-Zeit-Erweiterung von `_stmt_Select`. Nach dem
Match-Erfolg wird die Guard-Expression evaluiert (Subject bleibt am Stack
fuer den Body), und ein zusaetzlicher `JUMP_IF_FALSE` springt zum
naechsten Case wenn der Guard falsch ist. Kein neuer Bytecode -- die
existierenden Ops reichen.

**Kompatibilitaet:** Existierende SELECT-Statements ohne `WHERE` laufen
unveraendert -- der Parser erzeugt `(matches, guard=None, block)`-Tupel,
sowohl 2-Tuple- als auch 3-Tuple-Cases werden in den Pfaden akzeptiert
(Backward-Compat).

## IN-Operator

`x IN container` testet Mitgliedschaft auf String, Tupel, Array oder Map:

```basic
IF "World" IN "Hello World" THEN ...      ' Substring
IF 5 IN (1, 5, 9) THEN ...                ' Tupel
IF "name" IN m THEN ...                   ' Map-Key
IF 20 IN nums THEN ...                    ' Array-Element
```

Maps haben STRING-Keys -- `5 IN map` wirft TypeMismatch. Negation klassisch
mit `NOT (x IN c)`. Praezedenz wie `=`/`<>` (Comparison-Ebene), Bytecode:
neuer Op `IN_OP=56`.

## Variadic-Functions

`...args` als letzter Parameter sammelt alle restlichen Positional-Args
in ein TUPLE:

```basic
SUB log(level AS STRING, ...rest)
    DIM msg AS STRING
    msg = "[" + level + "]"
    DIM j AS INTEGER
    FOR j = 0 TO rest.length() - 1
        msg = msg + " " + STR$(rest[j])
    NEXT
    PRINT msg
END SUB

log("WARN")                          ' rest = ()
log("INFO", "App", "started")        ' rest = ("App", "started")
```

**Einschraenkungen:**
- Variadic muss letzter Parameter sein (Parser-Fehler sonst).
- Variadic-Slot kann nicht mit Named-Arg uebergeben werden (semantisch
  unklar -- werfen).
- TUPLE als Type stuetzt `length()`, `len()` und `[i]`-Index-Access.
- Keine Default-Werte fuer Variadic (immer `()` wenn leer).

**Implementation:** AST `Param.is_variadic`, `CompiledFunction.is_variadic`.
Im Tree-Walker `_resolve_args` und in den VMs `_exec` werden ueberzaehlige
Positional-Args in ein Tupel gesammelt; Default-Resolution wird fuer
variadic-Funktionen umgangen.

**Beispiel:** [examples/62_qol_sprint.dh](examples/62_qol_sprint.dh).

## Method-Syntax auf Built-in-Containern

Strings, Arrays und Maps haben Convenience-Methoden, die zu BUILTINs
delegieren:

```basic
PRINT "hello".upper()           ' "HELLO" (= UPPER$("hello"))
PRINT "  hi  ".trim().upper()   ' Method-Chain: "HI"
PRINT a.length()                ' = LEN(a)
m.put("k", 1)                   ' = MAPPUT(m, "k", 1)
PRINT m.has("k")                ' = MAPHAS(m, "k")
```

**Dispatch-Tabelle:** `interpreter.CONTAINER_METHODS` mappt
`(target_kind, method_name)` zu BUILTIN-Namen. Tree-Walker und beide
VMs konsumieren dieselbe Tabelle (Single-Source-of-Truth).

**Verfuegbare Methoden:**
- String: `upper`, `lower`, `length`/`len`, `trim`, `left`, `right`,
  `mid`, `indexof`, `replace`, `split`, `padl`, `padr`.
- Array: `length`/`len`.
- Map: `put`, `get`, `getor`, `has`, `keys`, `size`/`length`/`len`,
  `remove`, `clear`.

Wer einen weiteren Method-Alias will, fuegt einen Eintrag in
`CONTAINER_METHODS` hinzu -- kein VM/Bytecode-Change noetig.

**User-Klassen-Methoden gewinnen:** Nur wenn der Receiver kein User-
Instanz ist (sondern String/Array/Map), wird die Container-Methoden-
Tabelle konsultiert. So kollidiert `Foo.upper()` mit User-Klasse `Foo`
nicht.

**Beispiel:** [examples/61_method_syntax.dh](examples/61_method_syntax.dh).

## Slicing

`s[a:b]`, `s[a:]`, `s[:b]`, `s[:]` -- liefert Substring (String) oder
neues 1D-Array (Array, echte Kopie). Negative Indices und Step werden
NICHT unterstuetzt -- konsistent mit der existierenden strikten Index-
Validierung. Out-of-bounds (`s[0:1000]` bei `len(s)=11`) wird auf den
gueltigen Bereich geclampt.

```basic
PRINT "Hello World"[6:11]     ' "World"
DIM b AS ARRAY OF INTEGER
b = a[1:4]                    ' echte Kopie, kein Alias
```

**Multi-Dim-Slicing** (`g[0:2, 1:3]`) wird nicht unterstuetzt --
NumPy-Semantik. Slice-Assign (`s[a:b] = ...`) wird ebenfalls bewusst
abgelehnt (Laenge-Match unklar).

**Implementierung:** AST `SliceAccess(target, lo, hi)`. Parser
disambiguiert in `_index_or_slice` anhand des Top-Level-`:`. Bytecode
`SLICE` mit Flag-Tupel `(has_lo, has_hi)` -- die VM popt entsprechend
viele Werte.

**Beispiel:** [examples/60_slicing.dh](examples/60_slicing.dh).

## ELIF / String-Multiplikation

Kleine Quality-of-Life-Erweiterungen:

- **`ELIF`** ist Alias fuer `ELSEIF` -- gleiches Token, kein AST-Change.
- **`"-" * 40`** liefert einen String aus 40 Bindestrichen. Auch
  `40 * "-"`. Negative Counts liefern leeren String. Strikt INTEGER --
  kein Float, kein Bool. Im VM-Pfad wurde gleichzeitig `OP.MUL` strenger
  gemacht (war vorher zu lax bei Bool-Operanden).

## WITH ... END WITH

Klassisches BASIC-Konstrukt fuer kompakte Member-Bursts:

```basic
WITH player
    .x = 100
    .y = 50
    .hp = 100
    .name = "Alice"
END WITH
```

**Semantik:**
- WITH-Ziel wird **einmal** evaluiert (wichtig bei Side-Effects).
- Innerhalb des Body ist `.member` Shortcut fuer `<target>.member`.
- Auch in Read-Position: `len = SQR(.x * .x + .y * .y)`.
- Compound-Assigns funktionieren: `.points += 5`.
- Verschachtelte WITHs erlaubt; innerstes gewinnt (Stack-Semantik).

**Implementierung:** Reines Compile-Zeit-Desugar ohne neuen Bytecode.
- Parser haelt einen `_with_stack: list[str]` mit Compiler-generierten
  Variablen-Namen (`__with_<n>`).
- `_with_stmt` parst, generiert frischen Namen, pusht auf Stack, parst Body,
  popt, gibt `With(var_name, target, body)` zurueck.
- Im `_primary` und `_statement_inner`: wenn aktueller Stack nicht leer und
  Token = DOT, desugar zu `MemberAccess(Identifier(top), name)`.
- Tree-Walker: `_exec_With` setzt `env.vars[var_name] = {"type":"any","value":val}`,
  fuehrt body aus, entfernt den Slot wieder.
- Compiler: `_stmt_With` allokiert anonymen Local-Slot (`_alloc_anon_slot`),
  bindet `var_name -> slot` in `local_slots` waehrend Body-Compile, entfernt
  ihn danach. So wird `Identifier(__with_<n>)` zu `LOAD_LOCAL slot`.
- "any"-Type-Coerce ist passthrough (Tree-Walker `interpreter.py:_coerce`,
  VM `vm.py:_coerce_any`, Cython `vm_native.pyx`).

**Beispiel:** [examples/55_with.dh](examples/55_with.dh).

## Tupel + Destructuring

Mehrfach-Rueckgabewerte ohne BYREF-Krampf.

```basic
FUNCTION minmax(a AS INTEGER, b AS INTEGER) AS TUPLE
    IF a < b THEN RETURN (a, b)
    RETURN (b, a)
END FUNCTION

DIM lo AS INTEGER
DIM hi AS INTEGER
(lo, hi) = minmax(7, 3)        ' Destructuring
```

**Tupel-Literal:** `(a, b, c)` -- mindestens 2 Elemente. Eine einzelne
geklammerte Expression `(expr)` bleibt Klammer-Gruppierung. `(1,)`-Single-
Tupel wie in Python wird NICHT unterstuetzt (kein Use-Case).

**Destructuring-Assignment:** `(t1, t2, ..., tn) = expr`. Die `expr` muss zur
Laufzeit ein Tupel mit exakt n Elementen ergeben -- sonst `DHRuntimeError`.
Targets duerfen Identifier, MemberAccess oder IndexAccess sein
(`(p.x, p.y) = polar_to_cart(r, a)` funktioniert).

**Type-Annotation:** `DIM t AS TUPLE` -- generisch, akzeptiert beliebige
Tupel. Keine Element-Type-Annotation an der Sprachebene; wer striktere
Garantien braucht, prueft selbst beim Destructuring.

**Implementierung:**
- AST: `TupleLit(elements)`, `TupleAssign(targets, value)`.
- Bytecode: `BUILD_TUPLE n` und `UNPACK_TUPLE n` (Ops 68, 69).
- Im Compiler-`_stmt_TupleAssign` werden Member/Index-Targets ueber einen
  anonymen Local-Slot zwischengepuffert (per `_alloc_anon_slot`), weil
  STORE_MEMBER/STORE_INDEX die Receiver-Position vor dem Wert braucht.
- Wertsemantik = Python-`tuple` (immutable). `_fmt` erzeugt `(a, b, c)`
  fuer PRINT.
- Cython-VM muss nach Aenderungen am Tupel-Pfad neu kompiliert werden.

**Praktisch:** Beispiel [examples/54_tuple.dh](examples/54_tuple.dh) zeigt
Min/Max, Vektor-Reflexion, Polar-Konvertierung, Player-State als Tupel.

## Bitwise-Operatoren

Strikt INTEGER (kein FLOAT, kein BOOL). Sechs Operatoren als Keywords:

```basic
a BAND b      ' bit-and
a BOR  b      ' bit-or
a BXOR b      ' bit-xor
a SHL  n      ' shift-left  (n >= 0)
a SHR  n      ' shift-right (n >= 0)
BNOT a        ' unaer, bitweises NICHT (= ~a in Python)
```

**Praezedenz:** Alle binaeren Bitwise auf EINER Ebene, links-assoziativ.
Position zwischen `Vergleich` und `+,-`. Heisst:
- `a BAND b = c` parst als `(a BAND b) = c`.
- `a + b BAND c` parst als `(a + b) BAND c`.
- `1 BOR 2 BAND 3` parst als `((1 BOR 2) BAND 3) = 3` — wer C-Stil-Praezedenz
  will, klammert (`1 BOR (2 BAND 3)`).

`BNOT` liegt im `_unary` neben `-` und unaerem `+` — d.h. tighter binding als
`*`/`/`. `BNOT a BAND b` ist `(BNOT a) BAND b`.

**Type-Strictness:** Bool wird abgelehnt (gleiche Linie wie `_check_num`).
Negativer Shift-Count wirft `DHRuntimeError` statt nichtssagendem Python-Fehler.

**Keine alten Built-ins mehr:** Frueher gab's `BITAND/BITOR/BITXOR/BITNOT/SHL/SHR`
als Funktions-Built-ins. Mit den Operatoren ueberfluessig — entfernt.
`BITAND(a, b)` -> `a BAND b`. Im Tree-Walker (interpreter.py:1009-1027) und
in beiden VMs implementiert (Ops 62-67 in bytecode.py).

## SELECT CASE

Mehrweg-Verzweigung statt verschachtelter `IF/ELSEIF`-Ketten. Drei Match-Formen
pro CASE, beliebig kombinierbar:

```basic
SELECT CASE x
    CASE 1                       ' exakter Wert
        ...
    CASE 2, 3, 4                 ' Liste von Werten
        ...
    CASE 10 TO 20                ' Bereich (inklusiv)
        ...
    CASE IS > 100                ' Vergleich (=, <>, <, >, <=, >=)
        ...
    CASE 1, 5 TO 8, IS = 13      ' alle Formen mischbar
        ...
    CASE ELSE                    ' Fallback (optional, max. einmal, muss letzter sein)
        ...
END SELECT
```

**Garantie:** Subject-Ausdruck wird **einmal** evaluiert (auch bei
Side-Effects in Function-Calls). Der erste passende Case gewinnt.

**Implementierung (lehrreich):** Im Parser zu `Select(subject, cases, else_block)`,
Cases sind `(list[CaseMatch], list[Stmt])`-Tupel mit `kind ∈ {"value", "range"}`.
Im Compiler **kein neuer Bytecode** — der Subject bleibt während aller Match-Tests
auf dem Stack (per `DUP` geklont), Range-Tests werden zu `subj >= lo` (mit
`JUMP_IF_FALSE`) gefolgt von `subj <= hi` (mit `JUMP_IF_TRUE` zum Block) verkettet.
Cython-VM hat es ohne Neukompilation übernommen.

## ENUM

Typsichere Konstanten mit Namespace-Zugriff (`State.PLAYING`):

```basic
ENUM State = MENU, PLAYING, PAUSED       ' compact
ENUM Permission                          ' block
    NONE = 0
    READ = 1
    WRITE = 2
END ENUM
```

Auto-Nummerierung (0, 1, 2, …) oder explizit. Mixed: nach explicit zählt's
weiter (`A, B = 5, C` → A=0, B=5, C=6). Member-Namen dürfen Keywords sein
(`READ`, `FILE`, `DATA`, `NONE`) — der qualifizierte Zugriff ist eindeutig.

**Implementierung:** `EnumDecl(name, members)` AST-Node. Im Tree-Walker
und Compiler zur Compile-Zeit zu einem `_EnumNamespace`-Objekt aufgelöst,
als globale CONST abgelegt. `MemberAccess` erkennt `_EnumNamespace` (in
`interpreter.py`, `vm.py`, `vm_native.pyx`) und liefert den Member-Wert.
Member-Werte müssen Compile-Time-Integer-Literale sein (auch im
Tree-Walker — Konsistenz). `DIM x AS State` löst der Parser zu `INTEGER`
auf, indem er bekannte Enum-Namen in `self._enum_names` trackt.

Keywords als Member-Namen: Parser-Helfer `_consume_member_name` und
DOT-Zugriff in `_postfix` akzeptieren jedes Token mit string-`value`,
nicht nur `IDENT`.

## Named Arguments

`func(name: "Anna", age: 30)` mit Defaults. Lexer-Token `COLON`,
AST-Node `NamedArg(name, value)` als Element von `Call.args`.

**Tree-Walker** (`interpreter.py`): `_resolve_args(decl, raw_args, fn_name)`
mappt positional + named auf Param-Reihenfolge, liefert eine voll-lange
Liste mit `_DEFAULT_SENTINEL` für Slots, die der User nicht belegt hat.
`_invoke` evaluiert Sentinels via Default-Ausdruck im local_env. Funktioniert
mit BYREF (Sentinel-Slots können kein BYREF sein) und mit Param-
referenzierenden Defaults.

**Compiler** (`compiler.py`): `_resolve_named_args(fn, raw_args, fn_name)`
löst zur Compile-Zeit auf — Slots ohne Wert kriegen den evaluierten
Default-Literalwert direkt als `LOAD_CONST` emittiert. `param_names` ist
ein neues Feld auf `CompiledFunction` (Compile-Zeit-Info, VM nutzt es nicht).
Auch `NEW Klasse(...)` wird so resolved (Init-Methode lookup zur Compile-Zeit).

**Einschränkungen im VM-Pfad:**
- Built-ins haben keine deklarierten Param-Namen → werfen.
- Method-Calls (`obj.method(name: ...)`): Klasse erst zur Laufzeit
  bekannt → Compiler wirft. Tree-Walker kann's.

## Self + implizite Methoden-Aufrufe

Innerhalb einer Klassen-Methode:

```basic
CLASS Wave
    SUB Init()
        StartCurrent()         ' impliziter Methoden-Aufruf
    END SUB
    SUB StartCurrent()
        ...
    END SUB
END CLASS
```

`Self` als Identifier liefert die aktuelle Instanz; bare `MethodName(...)`
ohne `Self.`-Präfix dispatcht zuerst gegen die Methoden der eigenen Klasse
(und Superklassen), erst dann gegen globale Funktionen.

**Tree-Walker:** `Interpreter._method_stack: list[(_Instance, _ClassInfo)]`
wird in `_invoke` gepusht/gepoppt. `_eval_Identifier` erkennt `"self"` und
liefert die aktuelle Instanz. `_eval_Call` mit Identifier-callee prüft
zuerst `_resolve_method(current_cls, name)`.

**Compiler:** `_load_var` emittiert für `name == "self"` (innerhalb
`current_class != None`) den neuen Op `LOAD_SELF`. `_expr_Call` bei
Identifier-callee resolved Methoden via `_resolve_method_compile` und
emittiert `LOAD_SELF` + Args + `CALL_METHOD`. Damit Methode A in derselben
Klasse die Methode B sehen kann, registriert Phase 4a vor dem Body-
Kompilieren leere Stub-`CompiledFunction`s in `ci.methods`.

**Bytecode-Op:** `LOAD_SELF = 88` — push `self_obj` (im VM-`_exec` als
Parameter). Implementiert in `vm.py` und `vm_native.pyx`. Wer Self-Code
schreibt, muss daher `vm_native.pyx` neu kompilieren.

## Statement-Trenner Doppelpunkt

`x = 1 : y = 2` — Doppelpunkt trennt Statements wie Newline.
`Parser._consume_terminator` und `_skip_newlines` akzeptieren beide Token.
Funktioniert mit Named-Args nicht in Konflikt, weil dort der `IDENT COLON`-
Lookahead in `_call_arg` läuft (innerhalb von `(` ... `)`), wo der
Terminator gar nicht erst geprüft wird.

## f-Strings (String-Interpolation)

`f"text {expr} text..."` -- der Lexer expandiert das zur Token-Sequenz
`("text" + STR$(expr) + "text" + ...)`. Damit funktionieren f-Strings ohne
einen einzigen Eingriff in Parser, Interpreter, Compiler oder VMs:

```basic
DIM name AS STRING
DIM hp AS INTEGER
name = "Anna"
hp = 75
PRINT f"{name} hat {hp} HP"          ' "Anna hat 75 HP"
PRINT f"max: {MAX(a, b)}"            ' Methodenaufrufe in {} sind ok
PRINT f"literal {{nicht interpoliert}}, aber {hp}"
```

**Eigenschaften:**
- `{{` und `}}` sind Escapes fuer literale geschweifte Klammern.
- Verschachtelte f-Strings sind nicht erlaubt (`f"{f"..."}"`).
- Ausdruecke duerfen `(`, `)`, Methoden-Aufrufe, MemberAccess etc.
  enthalten -- der Tokenizer matched balanced braces.
- Ohne `f`-Prefix bleibt `"hi {name}"` ein wortlich enthaltener String mit
  geschweiften Klammern -- Opt-in.
- Editor-Highlighter erkennt f-Strings als Block und faerbt den ganzen
  Range einheitlich als String (siehe editor_qt/highlighter.py).

**Format-Specs** (`{expr:spec}`): ein Top-Level-`:` im Platzhalter trennt
einen printf-Spec ab -- der Lexer emittiert dann `FORMAT$(expr, "%spec")`
statt `STR$(expr)`:
```basic
PRINT f"FPS {fps:.1f}  Score {score:05d}"   ' "FPS 59.7  Score 00042"
```
Ein `:` innerhalb von `()`/`[]`/`{}` (z.B. Slice `s[0:3]`) oder String-Literalen
zaehlt NICHT als Spec-Trenner -- `_split_fstring_spec` in `lexer.py` trackt
Klammer-/String-Tiefe. Rein Lexer-basiert, daher in allen drei Pfaden gleich.

**Implementierung:** `lexer._scan_fstring` wird beim ersten `f"`-Lookahead
aufgerufen (lexer.py:114-115) und emittiert die
expandierte Token-Sequenz selbst -- mit Sub-Lexer fuer den Ausdrucks-Teil.

**Beispiel:** [examples/69_fstring.dh](examples/69_fstring.dh).

## Kontrollfluss: BREAK / CONTINUE / REPEAT / TRY

Diese sind implementiert (waren in aelteren Doku-Staenden nicht aufgefuehrt):

- **`BREAK`** / **`CONTINUE`** in `FOR`, `FOR EACH`, `WHILE`, `REPEAT`. Auch in
  Single-Line-IF (`IF v = 40 THEN BREAK`). Tree-Walker via `_BreakSignal`/
  `_ContinueSignal`-Exceptions; Compiler via `break_patches`/`continue_patches`-
  Stack (mit `try_depth` fuer korrektes `TRY_END`-Unwinding).
- **`REPEAT ... UNTIL cond`** -- Post-Test-Loop (laeuft mindestens einmal).
- **`WHILE cond ... WEND`** -- Pre-Test-Loop.
- **`TRY ... CATCH [e] ... END TRY`** + **`THROW value`**. Die Catch-Variable
  ist optional und faengt den (String-)Wert. Kein typed Catch -- `THROW` wirft
  beliebige Werte; Module-/Runtime-Fehler kommen als `DHRuntimeError`-Message.

```basic
FOR EACH e IN enemies
    IF e.dead THEN CONTINUE
    IF boss_killed THEN BREAK
    e.update()
NEXT

TRY
    riskante_op()
CATCH msg
    PRINT "Fehler: " + msg
END TRY
```

## FOR EACH

`FOR EACH var IN container ... NEXT` -- iteriert ueber STRING (Zeichen),
TUPLE, 1D-ARRAY oder MAP (Keys):

```basic
FOR EACH b IN bullets
    b.update()
NEXT
FOR EACH k IN scores        ' Map -> Keys
    PRINT k, MAPGET(scores, k)
NEXT
```

`each` ist **kontextuell**, kein Keyword: `FOR each = 1 TO 3` mit einer
Variable namens „each" bleibt ein regulaerer FOR (Disambiguierung im Parser:
FOR EACH nur wenn nach `each` ein IDENT statt `=` folgt).

**Implementierung:** AST-Node `ForEach(var, iterable, body)`. Tree-Walker
iteriert direkt (`_iter_for_comp`, wie Comprehensions). Compiler desugart zu
einem Vorwaerts-Index-Loop ueber `__comp_iter(iterable)` (-> TUPLE) +
`LOAD_INDEX` und nutzt den vorhandenen break/continue-Patch-Stack -- **kein
neuer Bytecode**, beide VMs unveraendert. Loop-Var wird als `"any"` deklariert.

## IIF (Inline-Ternary)

`IIF(cond, then, else)` -- echter **lazy** Ternary, nur EIN Zweig wird
ausgewertet (Short-Circuit):

```basic
dx = IIF(moving_left, -speed, speed)
PRINT IIF(x <> 0, 100 \ x, -1)    ' bei x=0: kein Division-Crash
```

`iif` ohne `(` bleibt ein normaler Bezeichner (kontextuell im Parser).
**Implementierung:** AST-Node `TernaryExpr(cond, then, else)`. Compiler
desugart zu `JUMP_IF_FALSE` (poppt die Bedingung) -- **kein neuer Bytecode**.
Tree-Walker: `_eval_TernaryExpr` evaluiert nur den gewaehlten Zweig.

## Array- & Map-Helfer

Reine Builtins (alle drei Pfade automatisch), auch als Container-Methoden
(`CONTAINER_METHODS`-Tabelle):

- `SORT(arr)` / `arr.sort()` -- 1D-Array IN PLACE aufsteigend (INTEGER/FLOAT/STRING).
- `REVERSE(arr)` / `arr.reverse()` -- 1D-Array IN PLACE umkehren.
- `ARRAY_INDEXOF(arr, v)` / `arr.indexof(v)` -- erster Index oder -1.
- `MAPVALUES(m)` / `m.values()` -- ARRAY aller Werte (Einfuege-Reihenfolge).
- `MAPITEMS(m)` / `m.items()` -- ARRAY von `(key, value)`-TUPELn (gut mit
  `FOR EACH` + Destructuring).

## Module-Imports mit Alias

`IMPORT "modul" AS alias` -- aliased die Built-ins / externen Typen unter
einem ersetzten Praefix:

```basic
IMPORT "json" AS j
DIM h AS J_HANDLE
h = J_PARSE("[1, 2, 3]")
PRINT J_GET_INT(h, "0")     ' 1
```

**Aliasing-Strategie:** Drachenhauch-Module teilen einen flachen Built-in-
Namespace -- es gibt kein echtes Namespacing. Der Alias dupliziert alle
Built-ins / Typen, deren Name mit `<modul>_` anfaengt, unter `<alias>_`.
Single-word-Namen (z.B. der externe Typ `vec2`) werden komplett ersetzt
(`vec2` -> `v` bei `IMPORT "vec2" AS v`).

**Konvention-basiert:** funktioniert fuer Module, deren Built-in-Praefix
dem Modul-Namen entspricht (json, db, tween, vec2, sprite, ecs, ...).
Module mit abweichendem Praefix (z.B. `imgfx` registriert `IMAGE_*`,
nicht `IMGFX_*`) sind nicht via Alias adressierbar -- der Praefix-Match
liefert dann leer.

**Idempotent + sticky:** zweimal mit unterschiedlichen Aliasen ist OK
(beide werden zusaetzlich registriert), aber doppelt mit demselben Alias
ist no-op.

## Dict/Set-Comprehensions

`{key: val FOR var IN iterable [WHERE filter]}` -- Dict-Comprehension,
liefert eine MAP. `{expr FOR var IN iterable [WHERE filter]}` -- Set-
Comprehension, liefert ein TUPLE mit deduplizierten Werten in der
Reihenfolge des ersten Auftretens.

```basic
' Dict-Comp: Quadrate als Map
DIM squares AS MAP OF INTEGER
squares = {STR$(x) + "sq": x * x FOR x IN (1, 2, 3, 4)}
PRINT MAPGET(squares, "3sq")     ' 9

' Set-Comp: eindeutige Mod-Werte
DIM distinct AS TUPLE
distinct = {x MOD 3 FOR x IN (0, 1, 2, 3, 4, 5, 6, 7, 8)}
PRINT distinct                    ' (0, 1, 2)
```

**Dict-Keys MUESSEN STRING sein** (Drachenhauch-MAP-Konvention). Der MAP-
Wert-Typ wird beim ersten Eintrag inferiert. Set-Comp ist eine pragmatische
Naeherung -- Drachenhauch hat keinen echten SET-Typ; das deduplizierte TUPLE
ist die nahe liegende Alternative.

**Implementierung:** Lexer kennt jetzt `LBRACE`/`RBRACE` (nur fuer Comp-
Position). Parser disambiguiert per `:`-Lookahead (Dict) oder direkt
`FOR` (Set). Compiler nutzt das existierende `BUILD_TUPLE_DYN`-Pattern
plus zwei interne Built-ins (`__SET_DEDUP`, `__DICT_FROM_PAIRS`) als
Final-Schritt -- keine neuen Bytecode-Ops noetig, Cython-VM ohne
Aenderung kompatibel.

**Beispiel:** [examples/71_dictcomp.dh](examples/71_dictcomp.dh).

## Operator-Overloading auf User-Klassen

Klassen koennen Operatoren ueberladen, indem sie `OPERATOR <op>`-Methoden
definieren -- analog zu `SUB`/`FUNCTION` im Class-Body:

```basic
CLASS Money
    DIM cents AS INTEGER

    OPERATOR + (other AS Money) AS Money
        DIM r AS Money
        r = NEW Money()
        r.cents = Self.cents + other.cents
        RETURN r
    END OPERATOR

    OPERATOR = (other AS Money) AS BOOLEAN
        RETURN Self.cents = other.cents
    END OPERATOR
END CLASS

DIM a AS Money
a = NEW Money()
a.cents = 100
DIM b AS Money
b = NEW Money()
b.cents = 200
PRINT (a + b).cents     ' 300
PRINT a = b             ' FALSE
```

**Erlaubte Operatoren:** `+`, `-`, `*`, `/`, `MOD`, `=`, `<>`, `<`, `>`,
`<=`, `>=`. Genau ein Parameter (`other`), Rueckgabetyp ist Pflicht.
BYREF und variadic sind nicht erlaubt.

**Implementierung:** Parser konvertiert `OPERATOR + (...)` zu einer Methode
mit reserviertem Namen `__op_add__` (siehe `parser._OPERATOR_NAMES`).
Tree-Walker (`_eval_BinaryOp`) und beide VMs (`OP.ADD/SUB/MUL/DIV/MOD/EQ/...`)
konsultieren via `_user_op(...)` die Methode auf LHS, dann auf RHS
(Reverse-Dispatch). Fallback ist der Standard-Pfad.

**Vererbung:** Operator-Methoden werden ueber die normale MRO gesucht --
Child-Klassen erben sie automatisch.

**Einschraenkungen:**
- Keine reflektierten Operatoren a la Python (`__radd__`). Wer `5 + money`
  unterstuetzen will, definiert `OPERATOR + (other AS INTEGER) AS Money`
  auf `Money` -- der Reverse-Dispatch greift dann.
- Kein Method-Overloading: pro Operator gibt's genau eine Methode.
  `Money + Money` und `Money + INTEGER` koennen nicht gleichzeitig
  definiert werden (man muesste type-switchen im Body).
- Operatoren auf Modul-Typen (Vec2 etc.) gewinnen vor User-Klassen --
  die Modul-Registry wird zuerst konsultiert.

**Beispiel:** [examples/70_operator.dh](examples/70_operator.dh).
