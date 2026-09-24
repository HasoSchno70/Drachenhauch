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

**Vorschlag:** `Value::Str(Rc<String>)` statt `Rc<str>` und ein eigener
Opcode für `x = x + ausdruck`, wenn `x` eine STRING-Variable ist: den Wert
aus dem Platz nehmen (`mem::take`), mit `Rc::get_mut` anhängen, zurücklegen.
Dazu im `ADD` für zwei Zeichenketten `format!` durch `String::with_capacity`
+ `push_str` ersetzen. Prüfstein: 400 000 × Anhängen unter 50 ms, und ein
Fall, bei dem eine zweite Variable denselben Text hält, darf ihn nicht
mitverändern.

### 1b. Builtins werden je Aufruf am NAMEN gesucht

`op::CALL_BUILTIN` reicht den Namen durch eine Kette von über 20
`try_*`-Funktionen (`try_typtest`, `try_array_hof`, `try_scene`, …,
zuletzt der große `match` in `builtins.rs`), und jede vergleicht
Zeichenketten. Bei knapp 2000 Builtins kostet schon `ABS` so viel wie ein
eigener Funktionsaufruf, und `MAPGET` das Doppelte.

**Vorschlag:** beim Laden (`model::specialize_args`, dort wird schon der
Index eigener Funktionen aufgelöst) je Aufrufstelle eine Builtin-Nummer
vergeben und über eine Sprungtabelle (`fn`-Zeiger je Nummer) aufrufen.
Zwischenschritt mit wenig Risiko: je Aufrufstelle merken, welche
`try_*`-Familie zuletzt geantwortet hat, und dort zuerst fragen. **Vorsicht:**
einige Familien antworten je nach ARGUMENT und nicht nur je Namen (z. B.
Feld-Befehle mit FUNCREF); die brauchen einen Rückfall auf die volle Kette.
Prüfstein: `ABS` in der Schleife unter 15 ns.

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

1. Zeichenketten anhängen linear (1a) — der einzige Punkt, an dem eine
   gewöhnliche Anwendung heute spürbar hängt.
2. Builtin-Aufruf über eine Nummer (1b) — betrifft jedes Programm.
3. Mehrere Variablen in einem `DIM`, englische Fehlermeldungen.
4. Schlanker Export, Tray/Benachrichtigung.
5. Aufrufe ohne Zwischen-`Vec` (1c), danach bei Bedarf ein Frame-Stapel.

Jeder Tempo-Punkt bekommt eine eigene Messung in `tests/pruef/` mit einer
Obergrenze, die großzügig genug für die CI-Rechner ist, aber den
quadratischen bzw. namensbasierten Weg sicher durchfallen lässt.
