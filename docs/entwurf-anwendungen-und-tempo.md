# Entwurf: Was fehlt, damit man Drachenhauch wählt — und wie es schneller wird

Stand 2026-09-24. Anlass: Drachenhauch ist nicht mehr nur ein Spiele-BASIC,
es soll auch für **Anwendungen mit Oberfläche** gewählt werden — und es soll
schnell sein. Hier steht, was dafür gemessen wurde und was fehlt, nach Wirkung
geordnet.

## 1. Tempo — gemessen

Gleiche Aufgaben in `dhrt` 2026.15 (Release-Bau) und CPython 3.12, dieselbe
Windows-Maschine, ohne andere Last, drei Läufe (die Streuung lag unter 5 %):

| Aufgabe | dhrt | Python | Verhältnis |
|---|---|---|---|
| Schleife, 10 Mio. Additionen (FLOAT) | 280 ms | 410 ms | **1,5× schneller** |
| `fib(30)` rekursiv (1,35 Mio. Aufrufe) | 277 ms | 56 ms | 5× langsamer |
| 1 Mio. INTEGER füllen + `SORT` | 96 ms | 72 ms | 1,3× langsamer |
| 200 000 × `MAPPUT` + `MAPGET` mit `STR$` | 180 ms | 55 ms | 3,3× langsamer |
| 100 000 × `s = s + "x"` | 160 ms | 52 ms | 3× langsamer (und quadratisch, s. 1a) |

Je Aufruf, 1 Mio. Mal in einer Schleife (die leere Schleife abgezogen):

| Aufruf | Kosten |
|---|---|
| eigene FUNCTION | ~60 ns |
| `ABS(i)` / `LEN("abc")` | ~43 ns |
| `MAPGET(m, "a")` | ~115 ns |

Zum Vergleich in CPython: ein Aufruf von `abs` kostet dort ~13 ns, der
einer eigenen Funktion ~17 ns.

### 1a. Zeichenketten anhängen ist quadratisch (höchste Priorität)

| Anhängen | Zeit |
|---|---|
| 50 000 × | 50 ms |
| 100 000 × | 171 ms |
| 200 000 × | 1 601 ms |
| 400 000 × | 7 065 ms |

`Value::Str` ist ein unveränderliches `Rc<str>`; `s = s + "x"` kopiert jedes
Mal die ganze Zeichenkette (dazu zwei `fmt`-Kopien in `op::ADD`). Genau das
tut eine Textanwendung ständig: Bericht, CSV, HTML, Protokoll zusammenbauen.
Python ist hier linear, weil es bei Referenzzahl 1 an Ort und Stelle anhängt.

**Erledigt (2026-09-24).** `Value::Str` ist jetzt `Rc<String>`, und
`x = x + e` / `x += e` auf einen lokalen oder globalen Platz wird zu einem
Befehl `ADD_STORE_LOCAL` bzw. `ADD_STORE_GLOBAL_SLOT`. Der lädt `x` wie
bisher VOR `e`; hält der Platz danach noch denselben Text (Zeigervergleich)
und ist `e` ein schlichter Wert, gibt der Platz seinen Verweis ab, und
`Vm::addieren` hängt mit `Rc::get_mut` an Ort und Stelle an. In jedem anderen
Fall (zweiter Halter, `e` hat `x` geändert, Objekt mit `OPERATOR +`) rechnet
er wie ADD + STORE. Eine Kette `x = x + t1 + t2` formt der Compiler zu
`x + (t1 + t2)` um, wenn `x` und `t1` sicher Text sind (auch `STR$`, `CHR$`
… und eigene Funktionen `AS STRING`) und die übrigen Glieder schlichte Werte.

| gemessen | 2026.15 | jetzt |
|---|---|---|
| 400 000 × `s = s + "x"` | 7 065 ms | 16 ms |
| 200 000 × `s = s + STR$(i) + ","` (global) | 9 400 ms | 36 ms |
| 200 000 × `r += CHR$(..) + "-"` (lokal) | 2 661 ms | 47 ms |
| Zahlenschleife 10 Mio. | 285 ms | 245 ms |
| Maps mit `"k" + STR$(i)` | 195 ms | 158 ms |

Nebenbei schneller geworden: Zwischenergebnisse wie `"a" + b + c` hängen
ebenfalls an, statt jedes Mal neu zu kopieren. Die Speicherprüfung von
`STORE_LOCAL`/`STORE_GLOBAL_SLOT` ist ein Makro (`passend!`); als Funktion
wurde sie nicht eingebettet und kostete gemessen 25 %.

**Nicht erfasst** (bleiben beim Kopieren): Felder (`Self.text = Self.text +
z`) und Feldelemente (`a[i] = a[i] + z`). Prüfstein:
`tests/pruef/zeichenketten_anhaengen.dhtest` -- auf der alten Laufzeit
fallen genau die zwei Tempo-Fälle.

### 1b. Builtins werden je Aufruf am NAMEN gesucht

`op::CALL_BUILTIN` reicht den Namen durch eine Kette von über 20
`try_*`-Funktionen (`try_typtest`, `try_array_hof`, `try_scene`, …,
zuletzt der große `match` in `builtins.rs`), und jede vergleicht
Zeichenketten. Bei knapp 2000 Builtins kostet schon `ABS` so viel wie ein
eigener Funktionsaufruf, und `MAPGET` das Doppelte.

**Erledigt (2026-09-24), mit dem Zwischenschritt.** Jede CALL_BUILTIN-Stelle
merkt sich in `Instr::familie`, welche Familie beim ersten Mal geantwortet
hat, und fragt beim nächsten Mal dort zuerst (`Vm::builtin_rufen`,
`Vm::familie_rufen`); sagt sie ab, läuft die ganze Kette wie bisher. Das
ist richtig, weil jede Familie am NAMEN entscheidet — geprüft an allen 27:
die einzige Ausnahme ist `SORT` (mit FUNCREF `try_array_hof`, mit
Wahrheitswert der reine Befehl), und `SORT` merkt sich nichts.

| je 1 Mio. Aufrufe | 2026.15 | jetzt |
|---|---|---|
| `ABS(i)` | 78 ms | 39 ms |
| `CHART_COUNT(c)` (Arm 494 im großen `match`) | 145 ms | 39 ms |
| `MAPGET(m, "a")` | 152 ms | 98 ms |
| Map-Test (`MAPPUT`/`MAPGET` mit `STR$`) | 195 ms | 117 ms |

**Die Nummer je Befehl lohnt sich danach nicht mehr:** mit dem Merkplatz
kostet ein Befehl hinten im `match` der reinen Befehle (Arm 494 von ~700)
genauso viel wie einer vorn — der Rust-Compiler verzweigt dort nicht linear.
Teuer war allein die Kette der 27 Familien davor. Was bleibt (~24 ns je
Aufruf gegen ~13 ns in CPython), ist Aufrufaufwand (`catch_unwind`,
Argumentprüfung), kein Suchen mehr. Prüfstein:
`tests/pruef/builtin_familien.dhtest` (Gegenprobe: ohne die
`SORT`-Ausnahme fällt der erste Fall).

### 1c. Funktionsaufrufe

Ein Aufruf holt Puffer aus einem Pool (gut), aber `stack.split_off` legt je
Aufruf einen neuen `Vec` für die Argumente an, und der Aufruf läuft rekursiv
über den nativen Stack (`exec → run_frame → dispatch`). Vorschlag: Argumente
direkt aus dem Stapel in die Locals verschieben (`drain`), ohne
Zwischen-`Vec`. Größerer Schritt, später: ein Frame-Stapel statt Rekursion
(macht auch die Tiefengrenze von 1000 überflüssig).

### 1d. Was schon schnell ist

Zahlenschleifen sind schneller als Python, Sortieren liegt nah dran, der
Leerlauf einer GUI-Anwendung kostet ~2 % eines Kerns. Beim Zeichnen ist
raylib ohnehin nicht der Engpass.

## 2. Was einer Anwendung fehlt

Das `gui`-Modul ist inzwischen breit (Tabellen/Gitter, Baum, Formulare mit
Prüfung und Datenbank-Bindung, Menüs, Dialoge, Drucken, Barrierefreiheit,
Form-Designer). Was jemand, der eine Geschäfts- oder Werkzeug-Anwendung
schreiben will, trotzdem vermisst:

| Lücke | Warum es zählt | Aufwand |
|---|---|---|
| **Fehlermeldungen nur auf Deutsch** | Wer außerhalb des deutschen Sprachraums sucht, scheitert an der ersten Meldung. Ein Katalog (`DHRT_LANG=en`) nach dem Muster von `i18n/en.json` im Buch | mittel |
| **Handbuch überwiegend Deutsch** | README und Buch gibt es englisch, `docs/` nicht | groß |
| **`DIM a AS INTEGER, b AS INTEGER`** geht nicht (Parse-Fehler) | das schreibt jeder Umsteiger als Erstes; gehört in die Umstiegsliste | klein |
| **Anwendungsgröße 27,5 MB** | jede exportierte Exe trägt die volle Laufzeit (3D, Audio, PDF-Schriften). Ein Export ohne ungenutzte Module (`--export --schlank`) | mittel |
| **Installer nicht signiert, macOS nicht beglaubigt** | SmartScreen/Gatekeeper warnen — für Anwendungen an Kunden ein Ausschlussgrund. Die Wege sind gebaut (`DH_SIGN_*`, `DH_MAC_SIGNATUR`), es fehlt das Zertifikat | Geld, kein Code |
| **Kein Tray-Symbol, keine System-Benachrichtigung** | typisch für Werkzeuge, die im Hintergrund laufen | klein |
| **Oberfläche ist selbst gezeichnet** | sieht auf jedem System gleich aus, aber nicht „nativ“; dafür gibt es Themen und Maßstab. Kein Handlungsbedarf, aber ehrlich sagen | — |
| **Kein Paketverzeichnis** | Bibliotheken gehen über `IMPORT "datei.dh"` und `DH_PATH`; ein `dhrt paket hole <url>` würde Teilen erleichtern | mittel |
| **Keine anonymen Funktionen** | gebundene Methoden decken Rückrufe ab; für `SORT`/Filter wären kurze Lambdas bequemer | mittel |
| **Kein Aufruf fremder DLLs/.so** | ein eigenes FFI (`LIB_LOAD`/`LIB_CALL`) öffnet vorhandene C-Bibliotheken | groß, heikel |

## 3. Empfohlene Reihenfolge

1. ~~Zeichenketten anhängen linear (1a)~~ — erledigt 2026-09-24.
2. ~~Builtin-Aufruf ohne Suche (1b)~~ — erledigt 2026-09-24 über den
   Merkplatz je Aufrufstelle.
3. Mehrere Variablen in einem `DIM`, englische Fehlermeldungen.
4. Schlanker Export, Tray/Benachrichtigung.
5. Aufrufe ohne Zwischen-`Vec` (1c), danach bei Bedarf ein Frame-Stapel.

Jeder Tempo-Punkt bekommt eine eigene Messung in `tests/pruef/` mit einer
Obergrenze, die großzügig genug für die CI-Rechner ist, aber den
quadratischen bzw. namensbasierten Weg sicher durchfallen lässt.
