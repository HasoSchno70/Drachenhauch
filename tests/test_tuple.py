"""Tests fuer Tupel-Literale, Tupel-Returns und Destructuring-Assignment.

Cython-VM braucht einen Recompile, bevor BUILD_TUPLE/UNPACK_TUPLE wirken --
hier testen wir nur Tree-Walker und Python-VM.
"""
import pytest


# --- Literale --------------------------------------------------------

def test_tuple_literal(run_gb, run_vm):
    src = '''
DIM t AS TUPLE
t = (1, 2, 3)
PRINT t
'''
    expected = "(1, 2, 3)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_tuple_with_mixed_types(run_gb, run_vm):
    src = '''
DIM t AS TUPLE
t = (42, "hi", TRUE, 3.5)
PRINT t
'''
    expected = "(42, hi, TRUE, 3.5)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_single_paren_is_grouping_not_tuple(run_gb, run_vm):
    """`(5)` bleibt eine normale Klammer-Gruppierung, kein 1-Tupel."""
    src = '''
DIM x AS INTEGER
x = (5)
PRINT x
'''
    expected = "5\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Destructuring ---------------------------------------------------

def test_destructure_two(run_gb, run_vm):
    src = '''
DIM x AS INTEGER
DIM y AS INTEGER
(x, y) = (10, 20)
PRINT x, y
'''
    expected = "10 20\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_destructure_three_mixed_types(run_gb, run_vm):
    src = '''
DIM a AS INTEGER
DIM b AS STRING
DIM c AS FLOAT
(a, b, c) = (5, "hi", 1.5)
PRINT a
PRINT b
PRINT c
'''
    expected = "5\nhi\n1.5\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_destructure_length_mismatch_throws(run_gb, run_vm):
    from drachenhauch.errors import DHRuntimeError
    src = '''
DIM x AS INTEGER
DIM y AS INTEGER
(x, y) = (1, 2, 3)
'''
    with pytest.raises(DHRuntimeError):
        run_gb(src)
    with pytest.raises(DHRuntimeError):
        run_vm(src)


def test_destructure_non_tuple_throws(run_gb, run_vm):
    from drachenhauch.errors import DHRuntimeError
    src = '''
DIM x AS INTEGER
DIM y AS INTEGER
(x, y) = 42
'''
    with pytest.raises(DHRuntimeError):
        run_gb(src)
    with pytest.raises(DHRuntimeError):
        run_vm(src)


# --- Function returns Tuple -----------------------------------------

def test_function_returns_tuple(run_gb, run_vm):
    src = '''
FUNCTION minmax(a AS INTEGER, b AS INTEGER) AS TUPLE
    IF a < b THEN
        RETURN (a, b)
    ELSE
        RETURN (b, a)
    END IF
END FUNCTION

DIM lo AS INTEGER
DIM hi AS INTEGER
(lo, hi) = minmax(7, 3)
PRINT lo, hi
(lo, hi) = minmax(2, 9)
PRINT lo, hi
'''
    expected = "3 7\n2 9\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_function_three_returns(run_gb, run_vm):
    """Klassischer Use-Case: Vektor-Operation mit zwei oder drei Komponenten."""
    src = '''
FUNCTION reflect(vx AS FLOAT, vy AS FLOAT, nx AS FLOAT, ny AS FLOAT) AS TUPLE
    DIM dot AS FLOAT
    dot = vx * nx + vy * ny
    RETURN (vx - 2.0 * dot * nx, vy - 2.0 * dot * ny)
END FUNCTION

DIM rx AS FLOAT
DIM ry AS FLOAT
(rx, ry) = reflect(3.0, 4.0, 1.0, 0.0)
PRINT rx, ry
'''
    # Reflexion an x-Achse-Normale: vx wird negiert, vy bleibt.
    expected = "-3.0 4.0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Tupel-Type-Check -----------------------------------------------

def test_tuple_var_rejects_non_tuple(run_gb, run_vm):
    from drachenhauch.errors import DHRuntimeError
    src = '''
DIM t AS TUPLE
t = 42
'''
    with pytest.raises(DHRuntimeError):
        run_gb(src)
    with pytest.raises(DHRuntimeError):
        run_vm(src)


def test_print_paren_expr_not_tuple(run_gb, run_vm):
    """`PRINT (1 + 2)` soll 3 ausgeben, kein Tupel."""
    src = "PRINT (1 + 2)"
    expected = "3\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Verschachtelte Tupel ------------------------------------------

def test_nested_tuple_in_assign(run_gb, run_vm):
    src = '''
DIM t AS TUPLE
t = ((1, 2), (3, 4))
PRINT t
'''
    expected = "((1, 2), (3, 4))\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Funktionsaufruf mit Tupel als Argument ------------------------

def test_call_with_paren_args_unaffected(run_gb, run_vm):
    """`func(a, b)` ist Aufruf mit 2 Args, kein Tupel-Argument."""
    src = '''
FUNCTION add2(x AS INTEGER, y AS INTEGER) AS INTEGER
    RETURN x + y
END FUNCTION

PRINT add2(3, 4)
'''
    expected = "7\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Destructuring im einzeiligen IF ----------------------------------------
#
# `(a, b) = f()` liess sich zwar an oberster Ebene und im Block-IF schreiben,
# im EINZEILIGEN IF aber nicht: die Zuweisung verbrauchte den Zeilenabschluss,
# den das IF danach selbst erwartete -- der Fehler erschien deshalb erst in
# der FOLGEZEILE ("Erwartet Zeilenende") und zeigte auf die falsche Stelle.
#
# Der Parser hatte den Fall schon vorgesehen (eigener Arm in
# `inline_statement`); uebersehen war nur der doppelte Abschluss. Gefunden
# beim Messen am Code-Feld, wo genau diese Zeile gebraucht wurde.

_ZWEI = '''
FUNCTION zwei() AS TUPLE
    RETURN (7, 8)
END FUNCTION
DIM x AS INTEGER
DIM y AS INTEGER
DIM b AS BOOLEAN
'''


def test_destructuring_im_einzeiligen_if(run_gb):
    out = run_gb(_ZWEI + '''
b = TRUE
IF b THEN (x, y) = zwei()
PRINT x; " "; y
''')
    assert out.strip() == "7 8"


def test_destructuring_im_else_zweig(run_gb):
    out = run_gb(_ZWEI + '''
b = FALSE
IF b THEN (x, y) = (1, 2) ELSE (x, y) = (30, 40)
PRINT x; " "; y
''')
    assert out.strip() == "30 40"


def test_destructuring_mit_doppelpunkt_verkettet(run_gb):
    """Der Doppelpunkt trennt Anweisungen -- die Zuweisung darf ihn nicht
    schon aufgebraucht haben."""
    out = run_gb(_ZWEI + '''
b = TRUE
IF b THEN (x, y) = zwei() : x = x + 100
PRINT x; " "; y
''')
    assert out.strip() == "107 8"


def test_destructuring_wird_nicht_ausgefuehrt_wenn_falsch(run_gb):
    """Gegenprobe: die Zuweisung haengt wirklich an der Bedingung und laeuft
    nicht einfach immer."""
    out = run_gb(_ZWEI + '''
b = FALSE
x = 5 : y = 6
IF b THEN (x, y) = zwei()
PRINT x; " "; y
''')
    assert out.strip() == "5 6"
