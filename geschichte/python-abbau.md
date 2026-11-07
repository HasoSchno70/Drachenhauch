# Python-Abbau und alte Python-Laufzeit

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Wie Python Schritt fuer Schritt wegfiel (Stufen B und C, Wege A und D), das Rust-Frontend, die Performance-Arbeit der alten Python-VM.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

> ## ⚠️ STUFE C — der Python-Teil ist GELÖSCHT (2026-09-21)
> Mit `docs/entwurf-python-abbau.md` 7.7 Punkt 7 sind weg: das Paket
> `drachenhauch/` (Qt-IDE, Qt-Werkzeuge, Python-Lexer/-Preprocess, `synth.py`,
> `graphics.py`, modules/__init__.py), `dhrun.py`, die `dh*.cmd`/`dh.sh`-
> Starter, `pyproject.toml`, `requirements.txt`, `tests/*.py` samt
> `conftest.py`, tools/qt_tests_einzeln.py, der Qt-Installer
> (`build_installer.py`, `Drachenhauch.iss/.spec`, `gen_notices.py`) und die
> 14 Qt-`.dhsprite`-Dateien. Logo und Schriftzug liegen jetzt in `daten/bilder/`.
>
> **Folgen für die Arbeit:** Geprüft wird NUR über `dhrt test tests/pruef`
> (`.dhtest`-Sammlungen, Format in `docs/werkzeuge.md`) und Rust-`#[test]`s;
> die CI fährt kein pytest mehr. Neue Funktionen kommen in `rust/`, in die IDE
> oder in die Werkzeuge. Die Fassung steht nur noch in
> `rust/drachenhauch_runtime/Cargo.toml`.
>
> **WICHTIG:** Die vielen Abschnitte weiter unten, die drachenhauch/...py,
> `editor_qt`, pytest, `conftest.py`, `run_gb`, den Qt-Designer oder
> „`tests/test_*.py`" nennen, sind **Geschichte** -- die Dateien gibt es nicht
> mehr. Wo ein Abschnitt eine pytest-Datei als Test nennt, ist der Fall
> umgezogen (Weg D) oder mit Python entfallen.

> ## ⚠️ STUFE B — Tree-Walker + Python-Toolchain ENTFERNT (2026-06-06)
> Früher gab es zusätzlich einen Python-**Tree-Walker** (`interpreter.py`, Referenz
> + Built-in-Host), einen Python-**Compiler** (`compiler.py`/`bytecode.py`/
> `serialize.py` → `.dhc`) und zwei Bytecode-VMs (`vm.py`, `vm_native.pyx`). **Alle
> entfernt** — `dhrt` hat sie abgelöst und ist die EINZIGE Runtime. Ebenfalls weg:
> `builtins_registry.py`, alle `modules/*.py`-Implementierungen (dhrt
> reimplementiert die Module nativ in Rust), `export.py`, `environment.py`, pygame.
>
> **Folgen für die Arbeit:** Neue Builtins/Sprach-Features kommen NUR in
> `rust/drachenhauch_runtime/` (+ ein Fall in einer Pruefsammlung `tests/pruef/*.dhtest`). Es gibt KEIN „beide Pfade" /
> „drei Pfade" / „Parität gegen Tree-Walker" mehr — Korrektheit sichern
> die Pruefsammlungen (`dhrt test tests/pruef`) + Rust-`#[test]`s.
>
> **WICHTIG:** Viele Feature-Abschnitte WEITER UNTEN erwähnen noch
> `interpreter.py` / `compiler.py` / `vm.py` / `vm_native.pyx` / „Tree-Walker" /
> „alle drei Pfade" / „in BEIDEN Pfaden umsetzen". Das sind **historische
> Implementierungsnotizen** — die Dateien existieren nicht mehr; gültig ist heute
> ausschließlich `dhrt` (`rust/drachenhauch_runtime/src/`: `lexer.rs`/`parser.rs`/
> `compiler.rs`/`vm.rs`/`builtins.rs`/Modul-`.rs`).

## Operator-Registry

Modul-eigene Typen koennen arithmetische Operatoren (`+`, `-`, `*`, `/`)
ueberladen, ohne dass interpreter.py / vm.py / vm_native.pyx angefasst werden:

```python
# In drachenhauch/modules/<name>.py
from . import register_operators

def _op_add(a, b):
    if isinstance(a, _MyType) and isinstance(b, _MyType):
        return _MyType(...)
    raise TypeMismatchError("...")

register_operators(_MyType, {"+": _op_add, "-": _op_sub, "*": _op_mul, "/": _op_div})
```

**Dispatch:** Vor dem Standard-Pfad ruft Tree-Walker (`_eval_BinaryOp`) und
beide VMs (`OP.ADD/SUB/MUL/DIV`) `modules.dispatch_binary_op(op, a, b)`.
Wenn `type(a)` oder `type(b)` registriert ist, dispatcht zur Handler-Tabelle;
sonst liefert die Registry `NO_OP_MATCH` und der Standard-Pfad uebernimmt.

**Konvention:** Bei asymmetrischen Operatoren (z.B. `Skalar * Vec2`) muss
der Handler beide Reihenfolgen selbst akzeptieren, weil die Registry nur
einen Handler-Eintrag pro Typ kennt -- siehe `vec2._op_mul` als Pattern.

**Equality:** `=` und `<>` sind nicht in der Registry -- die Standard-
Python-Equality (`__eq__`/`__ne__` auf der Klasse) reicht. Die VMs nutzen
`a == b` direkt.

Wer einen neuen Math-Typ wie `_Mat3x3`, `_Complex` oder `_Quat` einbauen
will, schreibt nur sein Modul -- keine Aenderung an Interpreter oder VMs.

## Kein Cython mehr, kein `setup.py build_ext`

Früher gab es zwei Cython-Module (`array_native.pyx` = `_GBArray`,
`ecs_native.pyx` = ECS-`_World`/`_Component`), die Hot-Path-Code des
Tree-Walkers nach C verschoben. Mit dem Tree-Walker sind auch sie weg, ebenso
`setup.py`, `drachenhauch/interpreter.py` und die `modules/*.py` — die
gesamte Laufzeit-Performance liegt in `dhrt` (Rust). Neuer Cython-Code kommt
nicht mehr dazu; wer Hot-Path-Arbeit hat, macht sie in `builtins.rs`/`vm.rs`.

pygame ist ebenfalls entfernt: Grafik und Audio gibt es ausschließlich in
`dhrt`. Python bedient nur noch die Editoren und das Tooling.

## Performance-Optimierungen im Compiler/VM

Mehrere Stufen, die zusammen die Native-VM auf 3–14× ggue Tree-Walker
bringen. Vollstaendige Bench-Tabelle in [docs/PERFORMANCE.md](docs/PERFORMANCE.md).

### Spezialisierte Numeric-Opcodes

11 Opcodes (`ADD_NN`, `SUB_NN`, `MUL_NN`, `DIV_NN`, `LT_NN`, `GT_NN`,
`LEQ_NN`, `GEQ_NN`, `EQ_NN`, `NEQ_NN`, `NEG_N`). Der Compiler emittiert
sie ueber `_expr_type(e)`-Inference (best-effort, konservativ), wenn
beide Operanden statisch als numerisch bekannt sind. Liest aus:
- Locals via `local_types[slot]`
- Globals via `_global_types[name]`
- Function-Return-Types
- BinaryOp/UnaryOp rekursiv
- **IndexAccess auf typisierte Arrays** (`buf[i]` bei `ARRAY OF INTEGER/FLOAT`
  -> Element-Typ) und **`Self.feld`** mit statisch bekanntem Skalartyp.
  Sicher, weil typisierte Arrays/Felder homogen sind. Damit greift der
  `_NN`-Pfad auch bei `buf[i] + i + j` und `Self.total + n`.

Bei `bench_loop` (tighter Numeric-Loop): **1.46×** auf Native-VM.

FOR-Loop-Bookkeeping (Increment + Bound-Check) wird ebenfalls ueber
spec-ops emittiert.

### 1D-Array-Index-Fast-Path

`LOAD_INDEX`/`STORE_INDEX` in beiden VMs haben einen Fast-Path fuer den
haeufigsten Fall: 1D-`_GBArray`, ein int-Index in Bounds. Ueberspringt die
`isinstance`-Cascade (str/tuple/array) + Index-Validierungs-Loop. Alle
Edge-Cases (String/Tupel/Multidim/OOB/bool) fallen unveraendert in den
generischen Pfad -> identische Fehler, kein Bit-identisch-Risiko. Kein neuer
Opcode, kein Compiler-Eingriff.

### Tree-Walker-Dispatch-Cache

`Interpreter._eval`/`_exec` memoisieren die Handler-Methode pro Node-Typ
(`_eval_cache`/`_exec_cache`: `type(node) -> gebundene Methode`) statt pro
Node `getattr(self, f"_eval_{name}")` zu bauen. Wirkt auf ALLEM Tree-Walker-
Code (~13 % bei expression-dichten Frames) -- relevant, weil der Editor-Run
und die Bench-Equivalenz den Tree-Walker nutzen.

### Inline-Cache fuer OOP-Dispatch

`CompiledFunction.caches`-Liste parallel zu `code`. Monomorphic IC fuer
`CALL_METHOD`, `LOAD_MEMBER`, `STORE_MEMBER` auf `_Instance`-Receivern.
Hit-Check: `obj.cls is cache[0]`. Spart `_resolve_method`-Call und
Dict-Lookup. Bei `bench_method_dispatch`: **1.34×**.

Cache wird auch im stub-Pfad (Phase 4a) konsistent kopiert. (Beschreibt den
geloeschten Python-Compiler -- der Rust-Compiler emittiert keine Inline-Caches,
siehe die Stufe-B-Warnung oben.)

### Globals-as-Slots

Compile-Zeit-Aufloesung von Top-Level-DIM/CONST/Enum/For-var/Class-
Static-Namen zu Slot-Indizes (`_global_slots: dict[str, int]`).
Neue Opcodes:
- `LOAD_GLOBAL_SLOT idx`
- `STORE_GLOBAL_SLOT idx`
- `DECLARE_GLOBAL_SLOT (idx, name_idx, type, default)`
- `DECLARE_GLOBAL_CONST_SLOT (idx, name_idx, type)`

Die VM-Pfade fuehren `global_slots: list[_Slot]` parallel zum
`globals_`-Dict. `DECLARE_*_SLOT` schreibt den `_Slot` in BEIDE
Strukturen (gleiches Object), so bleiben name-basierte Ops
(`INPUT_NAME`, `LOAD_NAME`) konsistent.

Pre-registrierte Globals (`KEY_*`, `BLACK`, `WHITE`, `PI`, ...) leben
weiter nur im Dict, weil der Compiler sie nicht statisch erkennt --
sie gehen ueber den Fallback `LOAD_NAME`.

`Struct`/`Array`/`Map`-DIMs werden **nicht** slot-allokiert (ihre Init-
Pfade `DECLARE_STRUCT_NAME` / `DECLARE_ARRAY_NAME` haben spezielle
Allokation -- der Performance-Vorteil waere klein, der Kosten gross).

Bei `bench_loop`: **1.30×** zusaetzlich.

### Constant Folding

`_try_fold(e)` im Compiler — BinaryOp/UnaryOp mit konstanten Operanden
wird zu einem einzelnen `LOAD_CONST`. Konservativ:
- kein Folding bei Bool-in-Arithmetik
- kein Folding bei Division durch 0 (Runtime-Error besser)
- kein Folding bei extrem grossen POW-Werten (Safety-Cap)
- `and`/`or` werden NICHT gefoldet (Short-Circuit-Semantik bewahren)

Hilft Patterns wie `FOR i = 0 TO 100 - 1`, `width / 2`, `2 * 3.14`.

### Typed Array Backing + cdef `_GBArray`

`ARRAY OF INTEGER` nutzt `array.array('q')` (8-Byte signed int) statt
Python-Liste. `ARRAY OF FLOAT` analog `array.array('d')`. Spart
Box/Unbox bei jedem Zugriff. **64-bit-Limit fuer INTEGER-Arrays**
(-9.2e18..9.2e18) — Skalar-`DIM x AS INTEGER` bleibt arbitrary-
precision.

`_GBArray` ist eine reine Python-Klasse (inline in `interpreter.py`; die frühere
Cython-Variante `array_native.pyx` mit typed memoryviews wurde entfernt).
`get_at(indices)` / `set_at(indices, value)` bleiben die Zugriffs-API.

### Convert/Coerce-Fast-Path in der Python-VM

`vm.py` hat ein `_FAST_COERCE`-Dict mit pro-Typ-Funktionen statt einer
generischen `if`/`elif`-Cascade. Trifft den heissesten Pfad jeder
STORE-Op (Local/Global/Field/Index/Parameter-Binding).

## Python-Abbau, Weg A: Werkzeugkette in dhrt (2026-09-06)

Richtung des Nutzers: Python soll wegfallen, alles laeuft ueber Rust und
Drachenhauch (`docs/entwurf-python-abbau.md`, Empfehlung A -> C -> B, D je
Bereich). **A ist gebaut.** Drei Unterbefehle, alle im Repo-Kontext
(`doku::repo_wurzel` sucht von cwd aufwaerts nach `docs/` + Index):
`dhrt lsp` (siehe naechster Abschnitt), `dhrt doku prosa|grammatik|referenz`
(`doku.rs`: Prosa aus `docs/`-Tabellen und -Listen plus Referenzbuch ueber
`node tools/buch_cmd_export.js`; Grammatik aus `lexer::KEYWORDS` + Index +
Handdoku; Referenz aus dem Quelltext ueber `symbole.rs` -- das lag frueher
NICHT in dhrt, "weil der Rust-Lexer Kommentare wegwirft"; der Symbol-Scanner
liest den Text) und `dhrt pruef [bloecke|namen|zaehlungen|konstanten|pfade|
beispiele]` (`pruef.rs`: die Codebloecke laufen IN-PROZESS durch
`check_source`, kein dhrt-Start je Buendel; `GEDULDET`/`GEDULDETE_PFADE` mit
Begruendung je Eintrag; `pfade` prueft Inline-Code UND Markdown-Links, die
Duldung gilt nur fuer Inline-Code). `--pruefen` bei `doku` = schreibt nichts,
Rueckgabe 1 bei Abweichung -- so haengen `builtin_prosa.json` und die
VS-Code-Grammatik an der Suite; `dhrt pruef` laeuft dort ebenfalls (seit
2026-09-16 alles in `tests/pruef/werkzeugkette/doku_pruefungen.dhtest`, vorher vier
pytest-Dateien). **Geloescht:**
`drachenhauch/lsp/`, `drachenhauch/doku.py`, `tools/gen_builtin_prosa.py`,
`tools/pruef_docs.py`, `tools/pruef_doku_aussagen.py`,
`vscode-drachenhauch/build_grammar.py`, die zwei LSP-Testdateien.
`builtin_docs.py` haelt keine Tabelle mehr, sondern laedt `builtin_docs.json`
(dieselbe Datei bettet dhrt ein). `dhrun.py --doku` reicht an
`dhrt doku referenz` durch. **Bleibt Python:** `rust/build_runtime.py` (Bau),
tools/qt_tests_einzeln.py (Qt-Testlaeufer), der Installer (PyInstaller,
solange die IDE Python ist), `tools/*.js` (Node, Buch). **Falle beim Bau:**
`serde` musste als direkte Abhaengigkeit dazu (fuer den 1-Leerzeichen-
Einzug der JSON-Ausgabe; nur `serde_json` reichte nicht, weil der Trait
`Serialize` aus `serde` kommt).

## Python-Abbau, Weg D: Pruefsammlungen fuer `dhrt test` (2026-09-07)

`dhrt test` lief bisher nur Pruefprogramme (`*_pruefung.dh`, ein Programm
mit ASSERT). Die Golden-Tests der Sprache sind aber Ausgabevergleiche --
gemessen 1084 der 3949 pytest-Tests in 79 Dateien (`assert run_gb(src) ==
"..."`). Dafuer gibt es jetzt **Pruefsammlungen** `*.dhtest`
(`pruefsammlung.rs`: Parser + Bewertung mit Rust-Tests; Laeufer in
`main.rs::sammlung_laufen`): `=== Name` beginnt einen Fall, dann der
Quelltext, dann `--- erwartet` (zeilenweise, Leerzeilen am Ende zaehlen
beiderseits nicht; `--- erwartet ungefaehr` laesst Zahlen um 1e-6
abweichen -- fuer alles, was `pytest.approx` brauchte, weil libm auf drei
Systemen in der letzten Stelle anders rundet), `--- enthaelt`, `--- fehler` (Abbruch +
Teiltext der Meldung), `--- datei name` (Beilage neben dem Programm), `--- verzeichnis name`,
`--- umgebung`. Jeder Fall ist ein eigener `dhrt run` in einem eigenen
Verzeichnis, die Faelle einer Datei laufen parallel (bis 8 Faeden);
`--filter Text` waehlt Faelle. KEIN_FENSTER-Meldungen und (mit
`DHRT_OHNE_GRAFIK=1`) fehlende Grafik-Builtins heissen "uebersprungen",
wie in conftest.py. Doku `docs/werkzeuge.md`. **Die Sammlungen liegen unter
`tests/pruef/`**; die CI ruft `dhrt test tests/pruef` direkt auf (bis
2026-09-20 ueber einen pytest-Anker), das Format
prueft `tests/pruef/werkzeugkette/dhrt_test_format.dhtest` am echten Laeufer. **Umgezogen und
aus `tests/` geloescht: 73 pytest-Dateien, 74 Sammlungen mit 1540 Faellen**
(Stand 2026-09-07, `dhrt test tests/pruef` laeuft sie in ~6 s; die 74. ist
`rekursionstiefe.dhtest`, dessen pytest-Datei mit drei Tests bleibt, die
`vm.rs`/`build.rs` LESEN statt etwas laufen zu lassen). Der Weg dorthin war
nicht Quelltext-Umbau, sondern **Aufzeichnung**: ein Wegwerf-Plugin (nicht
im Repo) hing sich in pytest an `run_gb`/`run_all`, schrieb je Aufruf
Quelltext, Ausgabe bzw. Fehlermeldung und die Dateien in `tmp_path` mit, und
ZWEI Laeufe mussten dasselbe liefern -- so fielen `TIME$`, Dateizeiten und
alles Zufaellige von selbst heraus. Ein Modul zog nur um, wenn JEDER seiner
Tests bestanden hatte und aufgezeichnet war; Faelle, deren Test mit
`pytest.approx` oder Vergleichen arbeitete oder lange Kommazahlen ausgibt,
bekamen `--- erwartet ungefaehr`. **Der dritte Schritt holte die 14 Module
nach, die beim zweiten liegen blieben** -- drei Ursachen, drei Mittel:
(1) "kein Aufruf" bei `[tw]/[vm]`-Laeufen war ein Loch des PLUGINS, nicht
der Tests: die `run_either`-Fixture hielt die urspruengliche `run_gb` fest,
bevor das Plugin sie umwickelte -- `run_either` mit umwickeln, und
gleiche Aufrufe aus `[tw]` und `[vm]` (oder `run_gb` + `run_vm`) werden EIN
Fall. (2) Binaere Beilagen: **`--- datei name base64`** (Block darf
umbrochen sein; `Fall::dateien` traegt seither `Vec<u8>`) -- fuer die
cp1252-Dateien von ini/xml/kodierung und die ZIP-Slip-Archive, die Pythons
`zipfile` mit `../`-Namen baut und dhrt selbst nie schriebe. (3) Zufall
(`UUID4$`, `RANDOM_BYTES`), Uhr (`TIME$`, `ZEIT_JETZT`) und absolute
Pfade im Quelltext (zip, csv, tiled) wurden zu SELBSTPRUEFENDEN Faellen: das
Programm prueft die Eigenschaft (Form per `REGEX_TEST`, Eindeutigkeit ueber
eine MAP, Pfadende per `RIGHT$`) und gibt TRUE aus; die 256-KB-Datei der
Hash-Tests entsteht im Programm (`BUFFER_NEW` + `WRITEALL_BYTES`), der
Erwartungswert stammt weiter von `hashlib`. **Der vierte Schritt (2026-09-08)
holte die Grafik-Tests**: `--- bild` prueft Punkte am Bildschirmfoto (der
Laeufer setzt `DHRT_SCREENSHOT` selbst und `DHRT_FRAMES=2`, wenn die
Umgebung des Falls keins nennt -- das erste Bild liegt bei doppelter
Pufferung noch nicht im Bild) oder, mit Namen, an einer Datei, die das
Programm geschrieben hat: `groesse B H`, `X Y #RRGGBB [+-N]`, `X Y nicht
#RRGGBB [+-N]`, `X Y = X2 Y2`, `X Y <> X2 Y2`. Die Toleranz je Kanal ist
das, was die pytest-Proben mit `g > 180 and r < 80` meinten; die Logik
(`Probe`, `bild_pruefen`) liegt in pruefsammlung.rs mit Rust-Tests, das
Dekodieren in main.rs hinter dem Grafik-Feature ueber raylibs
`Image::load_image` (ohne raylib ist der Fall uebersprungen, nicht falsch --
und raylibs INFO-Zeile beim Laden muss man abschalten, sonst steht sie in
der Bilanz). Die uebrigen Grafik-Tests starten dhrt selbst per
`subprocess.run` mit `DHRT_FRAMES` -- der Aufzeichner wickelt seither auch
`subprocess.run` um (Quelle aus dem Pfad im Aufruf, `DHRT_*` aus dem env,
Dateien vorher/nachher; die `_gbtest_`-Aufrufe der Fixture nimmt er aus,
sonst durchsucht er das System-Temp und faellt an gesperrten Dateien).
Stand danach: 96 Sammlungen mit 1729 Faellen, 90
pytest-Dateien geloescht. **Der fuenfte Schritt (selber Tag) nahm den Rest
der Konsolen- und gui-Goldens auf einmal**: der Aufzeichner lief ueber ALLE
verbleibenden Nicht-Qt-Dateien (zweimal, 1460 Tests je Lauf), und was
stabil war, wurde Sammlung -- 28 Dateien ganz, 24 teilweise. Dazu zwei
Formatstuecke: **`--- ton datei.wav`** prueft eine WAV, die das Programm mit
`AUDIO_SAVE_WAV` geschrieben hat (`kanaele`, `bits`, `abtastrate`, `dauer S
+-T`, `spitze X +-T`, `pegel VON BIS <X | >X | X +-T` als Spitzenwert je
10-ms-Fenster des ersten Kanals -- die `_huelle` der pytest-Tests --,
`kanaele gleich|verschieden`; eigener RIFF-Leser `wav_lesen` in
pruefsammlung.rs, weil `sprache::wav_lesen` die Kanaele mittelt), und
**`--- seriell`** vor dem ersten Fall laesst die Faelle einer Datei
NACHEINANDER laufen -- gebraucht von gui_text_formular, dessen Faelle ueber
die Zwischenablage tippen: parallel pastete ein Fall den Text des anderen.
Was NICHT umzog, steht als Ausnahmeliste im Sammlungsbauer mit Grund je
Test: Benutzerordner/DH_PATH (bibliothek), MILLIS-Werte (builtins_uhr),
Dateiname oder Spalte in einer Meldung, BOM in der Quelle, fremde
Umgebungsvariablen, gemischtes stdout/stderr, Portamento (Nulldurchgaenge
zaehlen). **Zwei Funde aus der ersten CI dieses Schritts:** (1) die
Shell-Tests riefen `cmd /c` und trugen in pytest `skipif(sys.platform !=
"win32")` -- dafuer **`--- system windows`** (auch posix/macos/linux):
der Fall gilt nur dort, anderswo ist er uebersprungen, nicht falsch.
(2) Ein Bau ohne Audio meldet "im Rust-Kern noch nicht verfuegbar" --
und ein Fall, der die Meldung mit TRY/CATCH faengt und AUSGIBT, endet mit
0 und der Meldung auf STDOUT; die Ueberspringregel prueft darum beide
Kanaele und nicht mehr den Rueckgabewert. **Der sechste Schritt (selber
Tag) nahm die `run_gb_roh`-Tests**: `--- eingabe` (Standardeingabe fuer
INPUT/STDIN(), `base64` fuer Bytes, Leerzeile am Ende = Endumbruch; der
Laeufer schreibt sie in ein gepipetes stdin und schliesst es -- ohne den
Block bleibt stdin `null`), `--- argumente` (eine Zeile je Argument, hinter
`--`), `--- rueckgabe N` (EXIT(N); ohne Angabe weiter 0) und `--- stderr`
(jede Zeile in der Fehlerausgabe, OHNE Abbruch -- fuer EPRINT; `fehler`
verlangt weiter Exit ungleich 0). Zwei Fallen dabei: der Kommentar aus dem
Docstring stand VOR dem Quelltext und verschob jede Zeilennummer in einer
Meldung um eins (jetzt dahinter), und `_dedent` nahm die fuehrende Leerzeile
weg, die pytest mitgeschrieben hatte (jetzt bleibt sie). Stand danach:
151 Sammlungen mit 2403 Faellen in ~65 s, 118 pytest-Dateien geloescht. Nicht uebertragbar und darum in pytest geblieben (Stand damals; gui_table_frozen_edge, gui_draw_window, gui_bindung und schriften_vorrat zogen am 2026-09-16 doch um):
gui_table_frozen_edge (Kantenprobe mit Schwellwert), gui_draw_window und schriften_vorrat (vergleichen zwei Laeufe miteinander), gui_bindung (SQLite-Datei), die 13 image_io-Tests mit Pillow als fremdem Leser fuer PNG/BMP/GIF, drei Shader-Bildproben, der Instancing-Render in m3d, drei Kontaktbogen-Tests und ein Index-Leser in input_edges. Bewusst NICHT umgezogen:
test_dateisystem.py (Dateizeiten, Gross/Klein je System -- seit 2026-09-16 doch
umgezogen, siehe unten), alles mit Ton
oder echter Eingabe-Wiedergabe (seriell), und Tests, die Quelltext oder
Bauskripte lesen. Regel fuer den weiteren Umzug: **eine pytest-Datei wird geloescht,
sobald ihre Faelle in einer Sammlung liegen** -- nie beides pflegen; ein
Verweis in der Doku wandert mit (`dhrt pruef pfade` findet ihn). Ein
Golden aus einer Aufzeichnung ist STRENGER als der Test davor (er prueft die
ganze Ausgabe, nicht eine Eigenschaft) -- wer einen solchen Fall aendert,
aendert die Erwartung bewusst. **Betriebssystem-Fehlertexte sind lokalisiert**
("Das System kann die angegebene Datei nicht finden" hier, "No such file or
directory" auf den Laeufern, und der os-error-Code weicht auch ab) -- in
`--- fehler` steht darum nur der eigene Teil der Meldung (Befehl, Pfad); die
ersten CI-Laeufe fanden genau zwei solche Faelle unter 1230. Und ein
Fehlerfall traegt den zufaelligen Namen der pytest-Testdatei
(`_gbtest_xxx.dh: Compile-Fehler: ...`) -- der Vergleich zweier Laeufe muss
ihn vorher abschneiden, sonst gilt jeder Compile-Fehler als instabil.

## Front-End-Portierung nach Rust (Lexer → Parser → Compiler)

**Laufendes Ziel:** Python langfristig nur noch in den Editoren — `dhrt` soll
selbst aus Quelltext Bytecode erzeugen (heute macht das die Python-Toolchain).
Die Front-End-Stufen werden **inkrementell** nach Rust portiert, **jede gegen
Python verifiziert** (cargo+rustc vorhanden → hier beweisbar). **ALLE 5 STUFEN
FERTIG:** Lexer (1) + Parser (2) + Compiler 3a–3e (3) + Preprocess/IMPORT (4) +
Verdrahtung (5). **`dhrt run datei.dh` ist ein eigenständiger End-to-End-Lauf
ohne Python** (preprocess→lex→parse→compile→VM, chdir ins Datei-Verzeichnis für
relative Asset-/IMPORT-Pfade; `dhrt datei.dh` ohne `run` per `.dh`-Auto-Detect
genauso, `.dhc` läuft weiter den direkten VM-Pfad). Debug-Einstiege
`dhrt --tokens` / `--ast` / `--preprocess` / `--runsrc` geben Token-Strom bzw.
AST bzw. gemergte Quelle aus bzw. führen ohne chdir aus (Dev/Parity).
Parity: tests/test_rust_lexer_parity.py
(215) + tests/test_rust_preprocess_parity.py (8)
+ [`tests/pruef/laufzeit/rust_run_parity.dhtest`](tests/pruef/laufzeit/rust_run_parity.dhtest) (2) = 225.
**Parser- und Compiler-Parity gibt es NICHT MEHR** (dieser Absatz nannte lange
beide, mit 107 bzw. 71 Tests): sie verglichen gegen den Python-Parser und
-Compiler, und die sind mit Stufe B gelöscht. Das Compiler-Gate ist stattdessen Output-Parität — siehe unten.
Dateien: [`src/lexer.rs`](rust/drachenhauch_runtime/src/lexer.rs),
[`src/ast.rs`](rust/drachenhauch_runtime/src/ast.rs),
[`src/parser.rs`](rust/drachenhauch_runtime/src/parser.rs),
[`src/preprocess.rs`](rust/drachenhauch_runtime/src/preprocess.rs),
[`src/compiler.rs`](rust/drachenhauch_runtime/src/compiler.rs). Plan/Stufen/Gotchas:
[docs/rust-frontend-port.md](docs/rust-frontend-port.md).
**Compiler-Gate = Output-Parität** (`dhrt --runsrc` stdout == Python-TW), NICHT
byte-exakter Bytecode: dhrt's VM kann beide Opcode-Formen, der Rust-Compiler
emittiert die generischen (kein Folding/`_NN`/IC) → identisches Verhalten, viel
weniger Code. **3a–3e fertig:** Skalar/Arithmetik/IF/WHILE/Builtins (3a) + FOR/Arrays/Index/
INPUT/DATA-READ/Locals (3b) + User-SUB/FUNCTION (Rekursion/CALL_USER/Named-Args/
Defaults/Variadic/FUNCREF, 3c) + Klassen/Structs (NEW/Member/Self/Methoden-
Calls/Vererbung/Properties/Operatoren/STATIC/ENUM, 3d) + SELECT/FOR EACH/REPEAT/
Tupel+Destructuring/WITH/TRY-CATCH-THROW/Slicing/List-Set-Dict-Comprehensions/
IIF/Coroutinen-YIELD + TUPLE/COROUTINE/FUNCREF/IMAGE-DIM-Typen (3e). Nicht-
unterstützt → `Err("Stufe 3e: ...")`. **Stufe 4 (Preprocess) fertig:**
`src/preprocess.rs` portiert `preprocess.process()` — `IMPORT "datei.dh"`
rekursiv inlinen (mit `' === IMPORT … ===`-Markern + `seen`-Dedup),
`IMPORT "modul"[ AS x]` → Kommentar (dhrt hat Modul-Builtins nativ). Importierte
Module liefern ihre externen Typen (`MODULE_TYPES`) an `compile_to_gbc(ast,
external_types)` → `DIM v AS VEC2` kompiliert nach `IMPORT "vec2"`. `--runsrc`
schaltet Preprocess vor. **Stufe 5 (Verdrahtung) fertig:** `dhrt run datei.dh`
(+ `dhrt datei.dh` per `.dh`-Auto-Detect) — eigenständiger End-to-End-Lauf ohne
Python, chdir ins Datei-Verzeichnis (relative IMPORT-/Laufzeit-Pfade); gemeinsame
Kette `compile_and_run_source` (geteilt mit `--runsrc`, das ohne chdir läuft).
Tree-Walker bleibt Referenz + `@builtin`-Host. AST-Parity-Gotchas: `.line` kein
Feld, `Param.by_ref` ist ein Token. 3e-Gotcha: `_collect_data` rekursiert in
SELECT/TRY, aber NICHT FOR EACH/WITH. 4-Gotcha: `MODULES`/`MODULE_TYPES` hardcoded,
mit `modules.discover_modules()` synchron halten. **Anschluss-Features fertig:**
(a) **Selbst-Export** `dhrt --export datei.dh [out]` — kompiliert Quelle → `.dhc`,
hängt Payload an Kopie der eigenen Exe (wie `export.py`, aber ohne Python),
kopiert `assets/`. (b) **Aliasierte Modul-IMPORTs** `IMPORT "json" AS j` —
`compile_env` liefert `(alias, modul)`-Paare, der Compiler bildet `j_parse`→
`json_parse` / `v`→`vec2` zurück (dhrt findet sie nativ), aliasierte externe
Typen (`j_handle`/`v`) sind gültige DIM-Typen. (c) **WASM (gebaut + verifiziert)** — emscripten-Einstieg
kompiliert `/program.dh` selbst (kein Pyodide), `build_wasm.py` bettet die Quelle
ein + verdrahtet das Windows-emscripten-Env automatisch (`setup_emscripten_env`:
CC/CXX/AR/Linker→`.exe`, bindgen-Includes, Ninja). `node web/dhrt.js` ==
Tree-Walker verifiziert. Toolchain (emscripten 6.0.0 + wasm-Target) installiert.

## Build und Test (Kurzform)

```
python rust\build_runtime.py                                        # Runtime dhrt
rust\drachenhauch_runtime\target\release\dhrt test tests\pruef       # Pruefsammlungen
rust\drachenhauch_runtime\target\release\dhrt run ide\ide.dh          # die IDE
rust\drachenhauch_runtime\target\release\dhrt run examples\<datei>.dh # ein Programm
```
