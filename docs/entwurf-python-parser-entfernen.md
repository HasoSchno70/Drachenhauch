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
| weitere betroffene Tests | 11 Dateien, 2639 Zeilen (siehe unten) |
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

## 3. Die eigentliche Arbeit: die 11 Testdateien

Nicht der Löschbefehl ist der Aufwand, sondern diese Dateien (2639 Zeilen):

```text
test_animeditor_document.py     test_multi_dim.py
test_array_literal.py           test_nil_literal.py
test_byref.py                   test_parser_with_lvalue.py
test_chex_literal.py            test_user_operator.py
test_comprehensions_dict_set.py test_formdesigner_document.py
test_lexer_edge_cases.py
```

Sie zerfallen in zwei Gruppen, und die Trennung ist **vor** dem Löschen zu
machen:

- **Sprach-Tests**, die zufällig über den Python-Parser gehen (`test_byref`,
  `test_multi_dim`, `test_nil_literal`, `test_array_literal`,
  `test_comprehensions_dict_set`, `test_user_operator`, `test_chex_literal`).
  Sie prüfen echtes Sprachverhalten und gehören auf die `run_gb`-Fixture
  umgezogen — dann prüfen sie sogar mehr als vorher, nämlich die Runtime, die
  wirklich läuft.
- **Parser-Tests**, die die Python-Innereien prüfen (`test_parser_with_lvalue`,
  `test_lexer_edge_cases`). Sie verschwinden mit ihrem Gegenstand.
- **Unklar**, weil sie Editor-Dokumente prüfen und den Parser nur nebenbei
  benutzen: `test_animeditor_document`, `test_formdesigner_document`. Erst
  ansehen.

**Wer umzieht statt wegwirft, gewinnt Abdeckung.** Wer pauschal löscht,
verliert sie. Das ist der Grund, warum dieser Schnitt ein eigenes Paket ist
und kein Nachmittag.

## 4. Was dagegen spricht

Ehrlichkeitshalber, auch wenn ich es für leichter halte:

- **Der Editor verliert seine Notfall-Diagnostik.** Wer die Runtime nie baut,
  sieht künftig gar keine Syntaxfehler mehr statt des ersten.
- **Der Parity-Test war ein Wächter.** Er hat bei Sprachänderungen zuverlässig
  gemeldet, wenn etwas nicht zusammenpasste — allerdings nur zwischen zwei
  Dingen, von denen eines ohnehin niemand ausführt.
- **`--tokens`/`--ast` in Python ist bequem**, wenn man am Parser selbst
  arbeitet. Dieses Argument entfällt mit dem Parser.

## 5. Zu entscheiden

1. **Schneiden oder behalten?** Die Zahlen oben sind der Stand; die Neigung
   ist schneiden.
2. **Falls schneiden: die 11 Testdateien zuerst triagieren** (Abschnitt 3) und
   die Sprach-Tests auf `run_gb` umziehen — als eigener Commit *vor* dem
   Löschen, damit die Abdeckung nie unter den heutigen Stand fällt.
3. **`__main__.py` ganz aufgeben** oder als dünnen Vorspann vor
   `dhrt --tokens`/`--ast` behalten?
