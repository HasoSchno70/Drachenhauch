"""Drei Stellen, an denen genau eine Haelfte fehlte.

Das Array-Literal gab es, das MAP-Literal nicht. `FOR EACH` lief ueber die
Schluessel einer MAP, an den Wert kam man nur ueber den Umweg
`MAPITEMS` + Destructuring. Und von den drei BASIC-Schleifenformen fehlte
ausgerechnet die gelaeufigste (`DO ... LOOP`).

Keine der drei braucht neue Laufzeit-Mechanik: das MAP-Literal benutzt
dieselbe Sammelstelle wie die Dict-Comprehension, die Paar-Form von
`FOR EACH` einen Vorschalt-Aufruf, und `DO ... LOOP` wird im Parser zu
`WHILE` bzw. `REPEAT`.
"""
import pytest

from drachenhauch.errors import DHRuntimeError, ParseError


# ------------------------------------------------------- MAP-Literal

def test_map_literal(run_gb):
    out = run_gb("""
DIM m AS MAP OF INTEGER
m = {"a": 1, "b": 2, "c": 3}
PRINT MAPSIZE(m); " "; MAPGET(m, "b")
""")
    assert out == "3 2\n"


def test_leeres_map_literal(run_gb):
    out = run_gb("""
DIM m AS MAP OF INTEGER
m = {}
PRINT MAPSIZE(m)
MAPPUT(m, "x", 5)
PRINT MAPGET(m, "x")
""")
    assert out == "0\n5\n"


def test_map_literal_mit_ausdruecken(run_gb):
    """Schluessel und Wert sind volle Ausdruecke, keine Literale."""
    out = run_gb("""
DIM n AS INTEGER
n = 2
DIM m AS MAP OF INTEGER
m = {"x" + STR$(n): n * 10}
PRINT MAPGET(m, "x2")
""")
    assert out == "20\n"


def test_map_literal_nachgestelltes_komma(run_gb):
    out = run_gb("""
DIM m AS MAP OF STRING
m = {"a": "eins", "b": "zwei",}
PRINT MAPGET(m, "b")
""")
    assert out == "zwei\n"


def test_map_literal_verlangt_string_schluessel(run_gb):
    """MAP-Schluessel sind STRING -- das gilt fuer das Literal wie ueberall."""
    with pytest.raises(DHRuntimeError, match="MAP-Schluessel"):
        run_gb("""
DIM m AS MAP OF INTEGER
m = {5: 1}
PRINT MAPSIZE(m)
""")


def test_dict_comprehension_bleibt_unberuehrt(run_gb):
    """`{k: v FOR ...}` und `{v FOR ...}` duerfen sich nicht verschieben --
    die Klammer traegt jetzt drei Bedeutungen."""
    out = run_gb("""
DIM q AS MAP OF INTEGER
q = {STR$(x): x * x FOR x IN (1, 2, 3)}
PRINT MAPGET(q, "3")
PRINT {x MOD 3 FOR x IN (0, 1, 2, 3, 4)}
""")
    assert out == "9\n(0, 1, 2)\n"


# --------------------------------------------- FOR EACH mit zwei Variablen

def test_foreach_paar_ueber_map(run_gb):
    out = run_gb("""
DIM m AS MAP OF INTEGER
m = {"a": 1, "b": 2}
FOR EACH k, v IN m
  PRINT k; "="; v
NEXT
""")
    assert out == "a=1\nb=2\n"


def test_foreach_einzelvariable_bleibt_bei_den_schluesseln(run_gb):
    """Die alte Form darf sich NICHT auf Paare umstellen -- sonst haette die
    Erweiterung bestehenden Code umgedeutet."""
    out = run_gb("""
DIM m AS MAP OF INTEGER
m = {"a": 1, "b": 2}
FOR EACH k IN m
  PRINT k;
NEXT
PRINT ""
""")
    assert out == "ab\n"


def test_foreach_paar_ueber_tupel_von_paaren(run_gb):
    out = run_gb("""
FOR EACH a, b IN (("x", 9), ("y", 8))
  PRINT a; b;
NEXT
PRINT ""
""")
    assert out == "x9y8\n"


def test_foreach_paar_mit_break_und_continue(run_gb):
    out = run_gb("""
DIM m AS MAP OF INTEGER
m = {"a": 1, "b": 2, "c": 3}
FOR EACH k, v IN m
  IF v = 2 THEN CONTINUE
  IF v = 3 THEN BREAK
  PRINT k
NEXT
""")
    assert out == "a\n"


# ----------------------------------------------------------- DO ... LOOP

def test_do_while(run_gb):
    out = run_gb("""
DIM i AS INTEGER
DO WHILE i < 3
  i = i + 1
LOOP
PRINT i
""")
    assert out == "3\n"


def test_do_until(run_gb):
    out = run_gb("""
DIM i AS INTEGER
DO UNTIL i >= 3
  i = i + 1
LOOP
PRINT i
""")
    assert out == "3\n"


def test_loop_while_laeuft_mindestens_einmal(run_gb):
    """Fusspruefung: der Rumpf laeuft, bevor zum ersten Mal geprueft wird."""
    out = run_gb("""
DIM z AS INTEGER
z = 99
DO
  z = z + 1
LOOP WHILE FALSE
PRINT z
""")
    assert out == "100\n"


def test_loop_until(run_gb):
    out = run_gb("""
DIM n AS INTEGER
DO
  n = n + 1
LOOP UNTIL n >= 3
PRINT n
""")
    assert out == "3\n"


def test_do_ohne_bedingung_braucht_break(run_gb):
    out = run_gb("""
DIM c AS INTEGER
DIM summe AS INTEGER
DO
  c = c + 1
  IF c > 5 THEN BREAK
  IF c MOD 2 = 0 THEN CONTINUE
  summe = summe + c
LOOP
PRINT summe
""")
    assert out == "9\n"


def test_do_verschachtelt(run_gb):
    out = run_gb("""
DIM a AS INTEGER
DIM b AS INTEGER
DIM t AS INTEGER
DO WHILE a < 2
  a = a + 1
  b = 0
  DO WHILE b < 2
    b = b + 1
    t = t + 1
  LOOP
LOOP
PRINT t
""")
    assert out == "4\n"


def test_bedingung_nur_an_einer_stelle(run_gb):
    """Beim UEBERSETZEN abgelehnt, nicht erst beim Laufen -- die Form ist
    am Text entscheidbar."""
    with pytest.raises(ParseError, match="nicht an beide Stellen"):
        run_gb("""
DIM i AS INTEGER
DO WHILE i < 3
  i = i + 1
LOOP UNTIL i > 9
""")


def test_do_bleibt_ein_gewoehnlicher_name(run_gb):
    """`do` und `loop` sind KONTEXTUELL, keine Schluesselwoerter -- sonst
    waere `DIM dO AS INTEGER` in bestehendem Code (examples/127_filedialog.dh)
    ploetzlich ein Fehler."""
    out = run_gb("""
DIM dO AS INTEGER
DIM loop AS INTEGER
dO = 7
loop = 8
PRINT dO + loop
""")
    assert out == "15\n"
