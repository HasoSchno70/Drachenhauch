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

### M0 — Messbank (Tage)

Ein Messprogramm unter tools/ (tempo.dh): feste Programme (Aufrufe, Zahlenschleife, Felder,
Objekte, Text, MAPs, ein echtes Spiel aus `examples/` mit `--bilder`), je
fünf Läufe, Bestwert, Tabelle gegen eine zweite `dhrt`. Ohne das ist jede
Aussage „schneller“ ein Gefühl. (Bisher lagen die Programme im Notizordner.)

### M1 — getypte Zwischenstufe im Compiler (2–3 Wochen)

Heute geht der Compiler vom Syntaxbaum direkt zu Bytecode, und die Typen
kennt er nur stellenweise (`statischer_typ`, das nebenbei annimmt, `/` liefere
immer FLOAT — stimmt nicht, `480/2` ist INTEGER). Neu: eine Stufe, in der
**jeder Ausdruck seinen Typ trägt** (oder „unbekannt“: FUNCREF, `any`,
Rückgabe aus Coroutinen). Sie ist die Grundlage für A und B, und sie ist für
sich prüfbar: ein Schalter wie dhrt --typen gibt sie aus, eine Sammlung hält sie fest.

### M2 — VM auf die Typen setzen (2–3 Wochen, Weg A)

- Opcodes auf Slots statt Stapel, wo die Typen feststehen:
  `ADD_II dst, a, b`, `LT_II_JUMP a, b, ziel`, `FOR_II`.
- Aufrufe ohne eigene Locals-Vec: ein gemeinsamer Wertestapel, der Rahmen ist
  ein Fenster darauf.
- Speichern ohne Typprüfung, wo der Compiler den Typ bewiesen hat.

Ziel: `fib(30)` unter 100 ms, Zahlenschleife unter 100 ms. Das kommt auch dem
Web-Bau zugute, der keinen JIT bekommen kann.

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
