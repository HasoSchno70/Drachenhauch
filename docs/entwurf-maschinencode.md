# Entwurf: Drachenhauch als übersetzte Sprache — richtig schnell

Stand 2026-09-24. Frage des Nutzers: wie wird Drachenhauch als
Compilersprache *richtig* schnell? Vorher erledigt ist die Kleinarbeit aus
`docs/entwurf-anwendungen-und-tempo.md` (Anhängen, Builtin-Familien, globale
Felder, Objektfelder, Aufrufe, Falten, JSON-Umweg, geteilter Dispatch).
Damit ist ausgereizt, was eine Stapel-VM mit `Value`-Enum ohne Umbau hergibt.

## 1. Wo wir stehen — gemessen

Dieselbe Maschine, dasselbe Programm (fib in Drachenhauch, Python und JavaScript), heute:

| Aufgabe | dhrt (Stand #241) | CPython | Node (V8, JIT) |
|---|---|---|---|
| `fib(30)`, 1,35 Mio. Aufrufe | 210 ms | 57 ms | 7 ms |
| 10 Mio. FLOAT-Additionen | 219 ms | 405 ms | 6 ms |

Die Schleife ist schneller als Python, aber **30- bis 35-mal langsamer als
Maschinencode**. Das ist kein Feinschliff mehr, das ist die Bauart:

- **Jeder Wert ist ein `Value`** (Enum, 24 Byte, mit Tag). `s = s + i * 0.5`
  liest zwei Tags, prüft sie, baut ein neues `Value`, schreibt es auf den
  Stapel, holt es wieder herunter und prüft beim Speichern den Typ — für eine
  einzige Maschinen-Addition.
- **Jede Instruktion geht durch `match instr.op`** und oft noch durch ein
  `match` auf `Arg`. Ein Sprung pro Instruktion, den die CPU schlecht
  vorhersagt.
- **Jeder Aufruf baut einen Rahmen** (Locals-Vec aus dem Vorrat, Stapel,
  `try_handlers`, Rückkehr über `Step`). Bei `fib` ist das fast die ganze Zeit.

Dabei ist Drachenhauch **streng typisiert** (Pascal-Art): in `fib` steht
`n AS INTEGER`, und der Compiler weiß das. Die VM prüft trotzdem zur Laufzeit,
was zur Übersetzungszeit feststeht. **Das ist der eigentliche Hebel** — eine
dynamische Sprache wie JavaScript muss sich Typen erst erraten, Drachenhauch
hat sie geschrieben.

## 2. Wege, die es gibt

| Weg | Wirkung | Kosten | Urteil |
|---|---|---|---|
| **A** VM umbauen: getypte Opcodes auf Slots, verschmolzene Vergleich+Sprung, Rahmen ohne Zuteilung | 2–3× | Wochen, geringes Risiko, gilt überall (auch Web) | **ja, zuerst** |
| **B** Maschinencode mit **Cranelift** zur Laufzeit (JIT) | 10–30× bei Zahlen/Aufrufen, 2–3× bei Text/Maps | Monate; +2–4 MB in `dhrt` | **ja, das Ziel** |
| C Maschinencode mit Cranelift *vorab* (Objektdatei, AOT) | wie B, ohne Startzeit | braucht einen Linker beim Export — den hat ein Nutzer nicht | später, wenn B steht |
| D nach C oder Rust übersetzen und fremden Compiler rufen | wie B/C | Nutzer bräuchte gcc/rustc; Exporte wären an deren Fassung gebunden | nein |
| E LLVM | etwas besserer Code als Cranelift | 50+ MB, langsames Übersetzen, schwierig zu bauen (drei Systeme) | nein |

**Warum Cranelift:** reines Rust (passt zu `cargo`, kein cmake/libclang wie
bei raylib), gebaut für schnelles Übersetzen (wasmtime übersetzt damit ganze
Programme beim Laden), Ziele x86-64 und arm64 — also Windows, Linux und Mac.
Der Code ist etwa auf dem Niveau von LLVM `-O1`; für eine Spielsprache reicht
das bei Weitem.

**Warum zur Laufzeit statt vorab:** dann braucht `dhrt --export` keinen
Linker, eine exportierte Exe ist weiter „Laufzeit + angehängtes Programm“, und
übersetzt wird beim Start (für ein Spiel mit ein paar tausend Zeilen im
Bereich von Millisekunden).

## 3. Die Lehre aus zwei Laufzeiten — ausdrücklich

Ein JIT ist **wieder ein zweiter Ausführungsweg**. Genau den hat das Projekt
mit Stufe B abgeschafft, und der Grund war gut: zwei Wege laufen
auseinander. Deshalb drei Regeln, die nicht verhandelbar sind:

1. **Die VM bleibt die Wahrheit.** Was der JIT nicht sicher kann, läuft in der
   VM — eine Funktion wird ganz übersetzt oder gar nicht, nie halb.
2. **Der JIT rechnet nur das Einfache selbst** (Ganzzahl/Kommazahl/BOOLEAN,
   Sprünge, Aufrufe übersetzter Funktionen). Alles andere — Zeichenketten,
   Felder, MAPs, Objekte, Builtins — ruft er als **dieselbe Rust-Funktion**,
   die auch die VM ruft. Es gibt keine zweite Fassung von `addieren` oder
   `MAPGET`.
3. **Jede Prüfsammlung läuft zweimal**: `DHRT_JIT=aus` und `DHRT_JIT=immer`
   (jede übersetzbare Funktion sofort übersetzt), dieselbe Erwartung. Das ist
   der Ersatz für die alte Parität gegen den Tree-Walker — nur dass die
   Referenz diesmal nicht gelöscht wird.

Dazu gehört die **genaue Semantik**, die der Maschinencode einhalten muss und
die leicht vergessen wird: Ganzzahl-Überlauf ist ein Fehler („Ganzzahl-Ueberlauf
bei '+'“), nicht Umlauf; Division durch null; `MOD` mit negativem Vorzeichen
(`-17 MOD 5` ist 3); Fehlermeldungen mit Zeile. Jede davon bekommt einen Fall,
der unter beiden Schaltern läuft.

## 4. Die Schritte

Jeder Schritt endet mit einer Messung gegen den Stand davor und wird einzeln
gemergt.

### M0 — Messbank (Tage) — erledigt 2026-09-24

`bench_dhrt.dh` nimmt per Vorgabe die acht Programme unter `tools/tempo/`
(Aufrufe, Kommazahlen, Ganzzahlen, Felder, Objekte, Text, Maps, Builtins).
Jedes misst seinen Kern selbst und gibt ihn als `ZEIT <ms>` aus -- die
Wandzeit traegt rund 40 ms Start mit. `--gegen pfad\zu\dhrt.exe` misst eine
zweite Laufzeit abwechselnd mit und gibt A/B aus. Erste Tabelle in
`docs/PERFORMANCE.md` (Messbank). Ein echtes Spiel ist nicht dabei: es
braucht ein Fenster, und die Bank soll auf jeder Maschine laufen.

**Der erste Fund der Bank:** `LEFT$`/`RIGHT$`/`MID$`/`INSTR` legten je Aufruf
den ganzen Text als Zeichenliste an -- behoben (`text.dh` 4144 -> 1135 ms).
Uebrig bleibt, dass `MID$(s, i, 1)` in einer Schleife ueber den Text mit dem
Quadrat der Laenge waechst; das ist ein Punkt fuer M4 (Zeichenketten, die
wissen, dass sie nur ASCII enthalten, oder einen Zeichenindex tragen).

### M1 — getypte Zwischenstufe im Compiler (2–3 Wochen) — erster Schritt 2026-09-24

Heute geht der Compiler vom Syntaxbaum direkt zu Bytecode, und die Typen
kennt er nur stellenweise. Neu ist `Compiler::typ_von` mit dem Typ aus
`typen.rs`: jedem Ausdruck ein Typ, oder `?` (unbekannt), wo er nicht
feststeht. `statischer_typ` bleibt, was es war -- ein vorsichtiger Helfer
fuer Warnungen, der vereinfachen darf; `typ_von` ist fuer den Code und darf
es nicht.

**Geprueft wird nicht an Beispielen, sondern am ganzen Bestand:** mit
`DHRT_TYPEN_PRUEFEN=1` setzt der Compiler hinter jeden getypten Ausdruck
eine Probe (`TYP_PRUEFEN`), die abbricht, wenn der Wert einen anderen Typ
hat. Die Pruefsammlung laeuft damit (Kinder erben die Variable). Der erste
Lauf fand genau eine falsche Regel -- "ein Befehl auf `$` liefert Text" --,
denn `SPLIT$` liefert ein Feld; 284 Proben in 49 Sammlungen schlugen an.
Seither kennt `typ_von` nur Befehle aus einer Tabelle, deren Typen
nachgemessen sind (`typen::builtin_typ`). `dhrt --typen datei.dh` zeigt, was
der Compiler je Ausdruck weiss.

**Wo der Typ am WERT haengt** (gemessen, nicht angenommen):

| Ausdruck | Ergebnis |
|---|---|
| `a / b` mit zwei INTEGER | bis 2026-09-24 INTEGER, wenn es aufgeht (`480 / 2`), sonst FLOAT -- seither immer FLOAT |
| `a ^ b` mit zwei INTEGER | INTEGER (`2 ^ 3`) oder FLOAT (`2 ^ -1`) |
| `a AND b`, `a OR b` | einer der beiden WERTE (`6 AND 3` ist 3) |
| `VAL(t)` | INTEGER oder FLOAT, je nach Text |
| `MIN(1, 2.0)` | der kleinere Wert samt seinem Typ (INTEGER) |

`typen.rs` fuehrt dafuer `ZAHL` (INTEGER oder FLOAT). **`/` ist seit
2026-09-24 immer FLOAT** (Entscheidung des Nutzers): `PRINT 480 / 2` zeigt
`240.0`, `feld[n / 2]` bricht ab, und `dhrt --check` warnt vor einem Index,
der sicher eine Kommazahl ist. Der volle Testlauf fand im ganzen Bestand
keine Stelle, die am alten Verhalten hing -- nur Erwartungen, die `4` statt
`4.0` zeigten, und im Referenzbuch den Satz, der das alte Verhalten
beschrieb. `^`, `VAL` und `MIN` bleiben wertabhaengig.

Offen in M1: Methoden ohne statisch bekannte Klasse, FUNCREF-Aufrufe, die
Laufvariable von FOR EACH, Modultypen (VEC2 & Co.) und die meisten Builtins
bleiben `?`. Das ist erlaubt (dort bleibt M2 beim heutigen Code), aber es
begrenzt, wie viel M2 bringt. Gemessen ueber alle `examples/*.dh` mit
`dhrt --typen`: **76 % von 86 700 Ausdruecken sind getypt.** Die groessten
Posten im Rest sind Builtins (`RGB`, `GUI_*`, `KEYPRESSED`, `MIN`/`MAX`) und
Modulhandles (`GUI_WINDOW`, `JSON_HANDLE`) -- fuer die Rechenschleifen, um
die es M2/M3 geht, zaehlen die wenig; fuer die naechsten Schritte heisst das:
Builtins mit Zahlen zuerst in die Tabelle.

### M2 — VM auf die Typen setzen (2–3 Wochen, Weg A)

- Opcodes auf Slots statt Stapel, wo die Typen feststehen:
  `ADD_II dst, a, b`, `LT_II_JUMP a, b, ziel`, `FOR_II`.
- Aufrufe ohne eigene Locals-Vec: ein gemeinsamer Wertestapel, der Rahmen ist
  ein Fenster darauf.
- Speichern ohne Typprüfung, wo der Compiler den Typ bewiesen hat.

Ziel: `fib(30)` unter 100 ms, Zahlenschleife unter 100 ms. Das kommt auch dem
Web-Bau zugute, der keinen JIT bekommen kann.

**Erster Schritt 2026-09-24 -- was gemessen wurde, bevor gebaut wurde:**

| Messung | Ergebnis |
|---|---|
| leere Schleife (nur FOR_NEXT) | 3,2 ns je Durchlauf |
| `s = i` (LOAD_LOCAL + STORE_LOCAL) | +9,5 ns |
| `s = i + i` (noch ein LOAD + ADD) | +8 ns |
| ein Aufruf `f(n)` ohne Rumpf | ~45 ns |
| dieselbe Schleife mit lokalen statt globalen Variablen | nur 5 % schneller |

Ein trivialer Befehl kostet also 4-5 ns. **Zwei Versuche brachten nichts und
sind wieder draussen:** (1) die Typangabe je Variable als Byte statt als
Zeichenkette vergleichen (A/B 0,99-1,09 -- der Textvergleich war nicht der
Engpass); (2) eine kleine eigene Schleife fuer die 15 haeufigsten Befehle, in
der Hoffnung, dass der Compiler dort Register frei hat (`dispatch` ist riesig).
Die Befehle wurden darin kaum billiger (3,7 ns), und jedes Verlassen der
Schleife fuer einen fremden Befehl kostete mehr, als sie sparte --
`zahlen.dh` wurde 20 % LANGSAMER. Die Lehre: ein Befehl tut hier echte Arbeit
(Wert klonen, Stapel, Typ pruefen, alten Wert freigeben); der Hebel ist die
ANZAHL der Befehle, nicht die Schleife um sie.

**Gebaut: Superinstruktionen.** `model::verschmelzen` markiert beim Laden
Folgen aus zwei Operanden (LOAD_LOCAL/LOAD_CONST/LOAD_GLOBAL_SLOT), einem
Rechen- oder Vergleichsbefehl und optional STORE_LOCAL, STORE_GLOBAL_SLOT
oder JUMP_IF_FALSE/TRUE -- nur, wo kein Sprung in die Folge fuehrt. Der
Bytecode bleibt dabei unveraendert stehen, Sprungziele und Zeilen stimmen
also weiter. `Vm::verschmolzen` fuehrt eine markierte Folge in einem Schritt
aus, aber nur den sicheren Fall; bei allem anderen (Ueberlauf, Division
durch 0, Umwandlung beim Speichern, konstanter oder leerer Platz,
Kommazahl-Gleichheit) laeuft sie Befehl fuer Befehl wie immer, mit derselben
Meldung. `DHRT_OHNE_VERSCHMELZEN=1` schaltet es ab, `dhrt --verschmolzen
datei.dh` listet die Stellen. Gemessen gegen den Stand davor: `aufrufe.dh`
(fib) 0,86, `ganzzahl.dh` 0,88, die uebrigen gleich. Offen fuer die naechsten
Schritte: der Aufruf selbst (~45 ns) und Folgen, deren zweiter Operand
schon auf dem Stapel liegt (`i MOD 3 = 0` verschmilzt heute nur zur Haelfte).

**Zweiter Schritt 2026-09-25: Rahmenstapel.** Ein Aufruf einer schlichten
Funktion (CALL_USER, keine Coroutine, kein BYREF) stieg bisher ueber
`exec -> exec_inner -> run_frame -> dispatch` neu in den Interpreter ein. Jetzt
legt `dispatch` den Zustand des Aufrufers auf `Vm::rahmen` und laeuft im
selben Aufruf mit dem Gerufenen weiter; RETURN (auch RETURN_VOID, HALT, Ende
des Codes) holt ihn zurueck. Fehler wickelt `run_frame` Rahmen fuer Rahmen
ab: erst die TRY-Handler des innersten, dann die des Aufrufers -- wie vorher,
als jeder Aufruf sein eigenes `run_frame` hatte; die Fehlerzeile nimmt die
Funktion, in der der Fehler FIEL. EXIT und die Stop-Signale raeumen alle
Rahmen ab. Nicht unter Profiler/Debugger/Stop (die fuehren einen Stapel je
`exec`). Dazu bindet `bind_params` im haeufigsten Fall die Parameter direkt,
statt jeden Platz erst mit seiner Vorgabe zu belegen. Gemessen: ein leerer
Aufruf 41 -> 29 ns, `aufrufe.dh` (fib) 0,85 gegen Schritt 1. Tests
`tests/pruef/rahmenstapel.dhtest` (Erwartungen = Ausgabe des Baus davor; die
erste Fassung nannte bei einem Fehler drei Ebenen tief die Zeile des
Aufrufers -- gefunden von genau diesem Vergleich).

**Dritter Schritt 2026-09-25: Methoden auf den Rahmenstapel -- gemessen und
verworfen.** Dafuer muss `dispatch` das `Self` je Rahmen fuehren (eine
Variable, die ueber die ganze Schleife lebt) und jeder Rahmen das `Self` von
Aufrufer und Gerufenem tragen. Ergebnis: ein Methodenaufruf 37 -> 35 ns, aber
JEDER Befehl in `dispatch` wurde teurer -- ein freier Aufruf 30 -> 35 ns, fib
1,07; mit schmalerer Buchfuehrung (freie Funktionen erben das `Self`) sogar
40 ns und fib 1,09. Die zusaetzliche lebende Variable und der Block kosten die
heisse Schleife mehr, als der Methodenaufruf spart. **Lehre fuer M3:** der
Rest liegt nicht mehr im Aufruf, sondern daran, dass `dispatch` als Ganzes
schlecht in Register passt -- das loest erst Maschinencode.

**Dabei gefunden: `MID$`/`INSTR`/`REGEX_FIND_POS` zaehlten Zeichen einzeln.**
`text.dh` war im Methoden-Bau 1,6-mal langsamer, obwohl es keine Funktion
aufruft: die Schleife je Zeichen in `builtins::zeichen_stelle` haengt an der
Lage des Codes. Jetzt nimmt sie bei reinem ASCII vor der Stelle die
Byte-Stelle direkt (`is_ascii` prueft wortweise), `zeichen_zahl` und
`zeichen_stelle_genau` dasselbe fuer die Zaehlungen in INSTR und
REGEX_FIND_POS. 28000 x `MID$` in 200000 Zeichen: 700 -> 23 ms, `text.dh`
0,07.

### M3 — Cranelift für Zahlenfunktionen (4–6 Wochen, Weg B)

Feature `jit` (standardmäßig aus). Übersetzt wird eine FUNCTION/SUB, wenn alle
ihre Locals, Parameter und Rückgaben INTEGER, FLOAT oder BOOLEAN sind und sie
nur Rechnen, IF/SELECT, FOR/WHILE/REPEAT und Aufrufe solcher Funktionen
enthält. Ein Fehler (Überlauf, Division durch null) springt in eine
Ausstiegsroutine, die dieselbe Meldung mit derselben Zeile baut wie die VM.

- `DHRT_JIT=aus|auto|immer`; `auto` übersetzt nach N Aufrufen oder beim ersten
  heißen Schleifenrücksprung.
- Debugger, Profiler, `dhrt call`/TASK_START: zunächst VM (sie brauchen die
  Zeile je Instruktion).
- CI: jede Sammlung zweimal (Abschnitt 3), auf allen drei Systemen.

Ziel: `fib(30)` unter 20 ms, Zahlenschleife unter 20 ms.

**Erster Schritt 2026-09-25 -- reine Zahlenfunktionen, gebaut** (`src/jit.rs`,
Feature `jit` mit Cranelift 0.134 -- 0.135 und neuer verlangen Rust 1.96).
Uebersetzt wird eine Funktion, wenn sie REIN ist: Parameter/Locals/Rueckgabe
INTEGER, FLOAT oder BOOLEAN (`any`-Locals, deren Art aus den Zuweisungen
folgt, gehen mit), und sie nur rechnet (+ - * / \ MOD, Vorzeichen,
Vergleiche, NOT), springt (IF, SELECT, WHILE, REPEAT, FOR ueber INTEGER) und
andere reine Funktionen ruft. Eine abstrakte Ausfuehrung ueber den Bytecode
bestimmt je Stelle die Art jedes Stapelplatzes und Locals; widersprechen sich
zwei Wege, bleibt die Funktion in der VM. Ruft sie eine, die in der VM
bleibt, bleibt sie auch dort (ganz oder gar nicht, Regel 1).

**Fehler: die VM rechnet nach.** Weil eine reine Funktion keine
Nebenwirkung hat, gibt der Maschinencode bei JEDEM Fehlerfall auf --
Ueberlauf, Division durch 0, `MIN \ -1`, FLOAT -> INTEGER mit
Nachkommastellen oder ausserhalb des Bereichs, `<` mit NaN, fehlendes
RETURN, zu tiefe Rekursion (Zaehler im `Kontext`, gleiche Grenze wie `exec`)
-- und die VM rechnet denselben Aufruf noch einmal. Meldung und Zeile kommen
damit von der VM selbst; es gibt keine zweite Fassung einer Fehlermeldung.
Nach 8 Rueckfaellen nimmt eine Funktion nur noch die VM. Die VM ruft den
Maschinencode nur, wenn jedes Argument genau passt (Umwandlungen und ihre
Fehler bleiben bei ihr), und nicht unter Profiler/Debugger/Stop.

Schalter: `DHRT_JIT=immer` uebersetzt beim Start jede reine Funktion (sonst
laeuft alles in der VM wie bisher -- `auto` kommt spaeter), `dhrt --jit
datei.dh` nennt je Funktion "uebersetzt" oder den Grund.

| gemessen | VM | Maschinencode |
|---|---|---|
| `fib(30)` (`tools/tempo/aufrufe.dh`) | 144 ms | 3,3 ms |
| 10 Mio. `s = s + i * 0.5` in einer Funktion | 189 ms | 5,1 ms |
| 10 Mio. `IF i MOD 3 = 0 THEN s = s + i` in einer Funktion | 382 ms | 8,1 ms |

Die Tempo-Programme, die im Hauptprogramm rechnen, aendern sich nicht --
dort sind die Variablen global (M4 Punkt 1). Ueber alle `.dh` des Repos:
85 Funktionen werden uebersetzt, keine laesst Cranelift scheitern; die
haeufigsten Gruende, in der VM zu bleiben, sind STRING (701, nur der erste
Grund zaehlt), globale Variablen (488) und eingebaute Befehle (95).

Pruefung: `tests/pruef/jit.dhtest` (jeder Randfall zweimal, Erwartung =
Ausgabe der VM; Gegenprobe: ohne die MOD-Korrektur bzw. ohne die
Ueberlaufpruefung beim Multiplizieren fallen genau die Maschinencode-Faelle),
die ganze Suite unter `DHRT_JIT=immer` (lokal `--schnell`: 3580 ok, 0 fehl),
und die CI laeuft die Sammlungen zweimal (Windows zweiter Lauf `--schnell`,
Linux/macOS ganz -- macOS ist zugleich arm64).

### M4 — Breite (laufend)

Nach Wirkung, jeweils mit eigener Messung:
1. **Globale Variablen** (feste Slots, seit #235/#236 gibt es die) und
   **Felder von INTEGER/FLOAT** mit Grenzprüfung inline.
2. **Objektfelder mit fester Lage**: eine Klasse kennt ihre Felder zur
   Übersetzungszeit — `Self.x` wird ein Speicherzugriff mit festem Versatz
   statt einer Suche.
3. **Zeichenketten und MAPs** über die Laufzeit-Funktionen (Regel 2);
   `STRING` als Zeiger, Referenzzählen bleibt beim Rust-Code.
4. **Builtins**: je Familie ein Aufruf mit fester Nummer (die Nummern gibt es
   seit #234) — für Grafikbefehle ist der Befehl selbst teuer, nicht der Weg
   dorthin.
5. Methoden (Merkplatz je Aufrufstelle wie in der VM, #238).

### M5 — Standard an (wenn M4 1–3 stehen)

- `auto` wird Vorgabe, die CI läuft weiter beide Schalter.
- Export: nichts zu tun (übersetzt wird beim Start).
- **macOS**: Hardened Runtime verbietet ausführbaren Speicher ohne die
  Berechtigung `com.apple.security.cs.allow-jit` — sie muss in die Signatur
  (`installer/bauen.dh`), sonst stürzt die beglaubigte App beim ersten
  übersetzten Aufruf ab.
- **Web**: kein JIT (emscripten), dort bleibt M2.

## 5. Was bewusst nicht kommt

- Kein eigener Registerzuteiler, keine eigenen Optimierungen jenseits dessen,
  was Cranelift mitbringt — der Gewinn liegt im Wegfall der Typprüfungen und
  des Dispatch, nicht in cleveren Schleifen.
- Keine Übersetzung von Coroutinen (YIELD), TRY/CATCH und `any`-Code in M3/M4;
  das ist die VM, bis gemessen wird, dass es sich lohnt.
- Kein AOT (Weg C), solange B den Export nicht bremst.

## 6. Grobe Rechnung

| Schritt | Aufwand | `fib(30)` | Zahlenschleife |
|---|---|---|---|
| heute | — | 210 ms | 219 ms |
| M2 (VM getypt) | ~5 Wochen inkl. M0/M1 | ~100 ms | ~100 ms |
| M3 (JIT Zahlen) | +4–6 Wochen | ~15 ms | ~15 ms |
| M4 (Breite) | laufend | — | Objekte/Felder 5–10×, Text/Maps 2–3× |

Die Zahlen für M2/M3 sind Schätzungen aus dem, was Stapel-VMs und
Cranelift-Code anderswo erreichen, nicht gemessen — M0 ist genau dafür da,
sie nach jedem Schritt zu ersetzen.
