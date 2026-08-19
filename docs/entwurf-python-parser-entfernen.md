# Entwurf: den Python-Parser entfernen (Stufe C)

**Stand 2026-08-19.** Kein Beschluss, sondern eine Rechnung mit den Zahlen, die
ich gemessen habe — damit die Entscheidung nicht aus dem Bauch kommt.

Ausgelöst hat ihn eine Rückfrage: *„du schreibst manchmal von zwei Parsern,
Python soll doch nur noch der Editor sein"*. Die Rückfrage war berechtigt, und
die Antwort lautet: beim **Ausführen** ist Python längst nicht mehr beteiligt —
aber ein zweiter Parser lebt weiter, und er kostet bei jeder Spracherweiterung
Arbeit.

## 1. Was gemessen ist

### Der Stapel

| Datei | Zeilen |
|---|---|
| `drachenhauch/parser.py` | 1963 |
| `drachenhauch/lexer.py` | 530 |
| `drachenhauch/ast_nodes.py` | 487 |
| `drachenhauch/tokens.py` | 216 |
| **Summe** | **3196** |

`drachenhauch/preprocess.py` (116 Zeilen) gehört **nicht** dazu. Sie hängt
nicht am Parser und wird von sechs Stellen gebraucht (`error_check.py`,
`output_console.py`, `dhrun.py`, `modules/__init__.py`, `__main__.py` und
`tools/pruef_doku_aussagen.py`). Sie bleibt.

### Wer den Parser noch benutzt

Außerhalb der Tests **zwei** Stellen:

1. **`drachenhauch/__main__.py`** — nur `--tokens` und `--ast` zur Fehlersuche.
2. **`editor_qt/error_check.py`** — als Rückfall, wenn `dhrt` nicht gebaut ist.

Bemerkenswert: außer `parser.py`, `lexer.py` und `__main__.py` importiert
**keine** Datei in `drachenhauch/` den Lexer oder die Tokens. Insbesondere
hängt die Syntaxhervorhebung des Qt-Editors nicht daran. Der Stapel ist
stärker isoliert, als sein Umfang vermuten lässt.

### Was der Rückfall wirklich leistet

`_check_syntax_only` in `error_check.py` ist **29 Zeilen** und hat drei enge
Grenzen — alle schon im Quelltext dokumentiert:

1. Er läuft **nur**, wenn `_find_dhrt()` nichts findet. Ist die Runtime da,
   wird er nie berührt.
2. Er liefert **genau ein** Problem (`Optional[ParseProblem]`) — das erste
   Syntaxproblem, dann ist Schluss. `dhrt --check` gibt **alle** Fehler *und*
   Warnungen als Liste.
3. Er kennt **keinen Compiler**: keine unbekannten Builtins, keine
   Arity-Prüfung, keine Namensraum-Meldungen aus WP I.

Der Fall, in dem er greift, ist also: `dhrt` ist nicht gebaut — und dann kann
der Nutzer sein Programm ohnehin nicht starten. Der Run-Knopf sagt in genau
dieser Lage bereits *„Native Runtime 'dhrt' nicht gefunden. Einmalig bauen
mit: `rust\build_runtime.py`"*.

### Was er kostet

| | |
|---|---|
| Code | 3196 Zeilen |
| Parity-Tests | 4 Dateien, 458 Zeilen |
| weitere betroffene Tests | 11 Dateien, aber nur ~20 Einzelstellen (Abschnitt 3) |
| laufend | **Doppelarbeit bei jeder Sprachänderung** |

Der letzte Punkt ist der eigentliche. `tests/test_rust_parser_parity.py`
vergleicht beide ASTs **Feld für Feld**; jedes neue Sprachmerkmal muss deshalb
in beiden Parsern gebaut werden. In dieser Sitzung traf es `PRIVATE` (WP I.1)
und die punktierten Typnamen (WP I.2) — beide Male ohne jeden Gewinn für die
Ausführung.

## 2. Vorschlag

**Streichen.** `parser.py`, `lexer.py`, `ast_nodes.py`, `tokens.py` und die
vier Parity-Testdateien entfernen; `preprocess.py` bleibt.

Die beiden Nutzer werden ersetzt:

- **`__main__.py`** → `dhrt --tokens` und `dhrt --ast` gibt es bereits
  (`main.rs:177` und `:180`). Der Debug-Zweck ist vollständig gedeckt.
- **`error_check.py`** → statt 29 Zeilen Ersatzdiagnostik eine ehrliche
  Meldung: *„Diagnose nicht verfügbar — Runtime bauen mit
  `rust\build_runtime.py`"*. Das ist dieselbe Auskunft, die der Run-Knopf in
  derselben Lage schon gibt.

## 3. Die Triage der 11 Testdateien (durchgeführt 2026-08-19)

**Meine erste Schätzung war um eine Größenordnung zu hoch.** Ich hatte den
Aufwand nach Dateigröße bemessen — 2639 Zeilen — und das war falsch. Die
Dateien benutzen `run_gb` längst und fassen den Parser nur an wenigen Stellen
an:

| Datei | Zeilen | Parser-Stellen | `run_gb`-Stellen |
|---|---|---|---|
| `test_formdesigner_document.py` | 881 | 2 | 26 |
| `test_user_operator.py` | 307 | 1 | 25 |
| `test_animeditor_document.py` | 282 | 1 | 4 |
| `test_parser_with_lvalue.py` | 221 | 1 | **0** |
| `test_array_literal.py` | 216 | 3 | 46 |
| `test_byref.py` | 203 | 1 | 22 |
| `test_comprehensions_dict_set.py` | 157 | 3 | 24 |
| `test_multi_dim.py` | 131 | 4 | 8 |
| `test_lexer_edge_cases.py` | 95 | 4 | **0** |
| `test_nil_literal.py` | 86 | 3 | 10 |
| `test_chex_literal.py` | 60 | 1 | 12 |

Zu tun sind also **rund 20 einzelne Stellen**, nicht 2639 Zeilen. Sie zerfallen
in vier Arten, und nur eine davon ist heikel:

### A. „Es parst überhaupt" — ersetzbar, verliert nichts

`test_byref.py:162`, `test_multi_dim.py:114`, `test_animeditor_document.py:237`,
`test_formdesigner_document.py:495` und `:557`.

Sie rufen den Parser und prüfen nur, dass keine Ausnahme fliegt — bei den
beiden Editor-Dokumenten, dass **erzeugter** `.dh`-Code gültig ist. Das kann
`dhrt --check` besser: es prüft zusätzlich den Compiler.

### B. AST-Gestalt — hier steckt die Entscheidung

`test_multi_dim.py:122-124` (`Dim`/`MultiDim`), `test_nil_literal.py:69`
(`NilLit`), `test_array_literal.py:209/215` (`ArrayLit`/`ListComp`),
`test_comprehensions_dict_set.py:150/155` (`SetComp`/`DictComp`),
`test_user_operator.py:19`.

Sie prüfen die **Form des Python-AST** — dass `DIM a, b, c` einen
`MultiDim`-Knoten ergibt, nicht bloß dass es funktioniert. Beim Umzug auf
`run_gb` wird daraus ein Verhaltenstest. Das ist **kein gleichwertiger
Ersatz**, aber vermutlich der richtige: mit einer einzigen Runtime zählt, was
sie tut, nicht wie ein Baum aussieht, den niemand mehr ausführt. Wer die Form
wirklich festhalten will, hat `dhrt --ast`.

### C. Token-Ebene

`test_nil_literal.py:64`, `test_chex_literal.py:41` — Behauptungen über den
Tokenstrom. Gleiche Überlegung wie B; `dhrt --tokens` gibt es.

### D. Verschwinden mit ihrem Gegenstand

`test_parser_with_lvalue.py` (221 Zeilen) und `test_lexer_edge_cases.py`
(95 Zeilen) benutzen **kein** `run_gb` — sie prüfen ausschließlich
Python-Innereien und haben ohne sie keinen Gegenstand mehr.

### Reihenfolge

1. **A** umstellen (5 Stellen, mechanisch, kein Verlust).
2. **B und C** entscheiden: als Verhaltenstest über `run_gb` weiterführen, oder
   als Gestalt-Test über `dhrt --ast`/`--tokens` neu schreiben. Erst danach
   anfassen.
3. **D** löschen — zusammen mit dem Parser, nicht vorher.

## 4. Was dagegen spricht

Ehrlichkeitshalber, auch wenn ich es für leichter halte:

- **Der Editor verliert seine Notfall-Diagnostik.** Wer die Runtime nie baut,
  sieht künftig gar keine Syntaxfehler mehr statt des ersten.
- **Der Parity-Test war ein Wächter.** Er hat bei Sprachänderungen zuverlässig
  gemeldet, wenn etwas nicht zusammenpasste — allerdings nur zwischen zwei
  Dingen, von denen eines ohnehin niemand ausführt.
- **`--tokens`/`--ast` in Python ist bequem**, wenn man am Parser selbst
  arbeitet. Dieses Argument entfällt mit dem Parser.

## 5. Stand und was noch zu entscheiden ist

**Erledigt:**

- **Schritt 1** (2026-08-19): die beiden Produktnutzer sind abgelöst.
  `error_check.py` sagt ohne `dhrt` ehrlich, dass es keine Diagnose gibt;
  `__main__.py` reicht `--tokens`/`--ast` an `dhrt` durch. Damit hängt der
  Parser **nur noch an Tests** — an keinem ausgelieferten Verhalten mehr.
  Frage 3 von unten hat sich damit von selbst beantwortet.
- **Schritt 2** (2026-08-19): die Triage steht in Abschnitt 3.

**Offen — eine Frage, und sie ist die einzige inhaltliche:**

Was geschieht mit den Tests der Art **B und C** (AST-Gestalt und Tokenstrom)?

- *Als Verhaltenstest über `run_gb` weiterführen.* Einfach, prüft die Runtime,
  die wirklich läuft — verliert aber die Zusage, dass `DIM a, b, c` genau
  einen `MultiDim`-Knoten ergibt.
- *Als Gestalt-Test über `dhrt --ast` / `dhrt --tokens` neu schreiben.* Hält
  die Zusage, kostet aber neue Testinfrastruktur für ein JSON-Format, das
  bisher niemand als Schnittstelle behandelt hat.

Meine Neigung ist die erste: mit einer einzigen Runtime zählt, was sie tut.
Aber es ist ein echter Verlust, und deshalb steht er hier und wird nicht
nebenbei entschieden.

Danach ist der Rest mechanisch: Art A umstellen, Art D löschen, die vier
Quelldateien und vier Parity-Tests entfernen.
