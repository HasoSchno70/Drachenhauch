"""Tests fuer die SELECT CASE-Konstruktion.

Wir nutzen die run_gb / run_vm Fixtures damit wir auch sicherstellen, dass
Tree-Walker und Python-VM beide identisch arbeiten.
"""
import pytest


SELECT_VALUE = """\
DIM x AS INTEGER
x = 2
SELECT CASE x
    CASE 1
        PRINT "eins"
    CASE 2
        PRINT "zwei"
    CASE 3
        PRINT "drei"
    CASE ELSE
        PRINT "anders"
END SELECT
"""

SELECT_LIST = """\
DIM x AS INTEGER
x = 3
SELECT CASE x
    CASE 1, 2, 3
        PRINT "klein"
    CASE 4, 5, 6
        PRINT "mittel"
    CASE ELSE
        PRINT "anders"
END SELECT
"""

SELECT_RANGE = """\
DIM x AS INTEGER
x = 50
SELECT CASE x
    CASE 0 TO 10
        PRINT "winzig"
    CASE 11 TO 100
        PRINT "klein"
    CASE 101 TO 1000
        PRINT "mittel"
    CASE ELSE
        PRINT "gross"
END SELECT
"""

SELECT_MIXED = """\
DIM x AS INTEGER
x = 4
SELECT CASE x
    CASE 1, 2 TO 5, 9
        PRINT "ausgewaehlt"
    CASE ELSE
        PRINT "nicht"
END SELECT
"""

SELECT_STRING = """\
DIM s AS STRING
s = "hilfe"
SELECT CASE s
    CASE "start", "go"
        PRINT "spiel"
    CASE "hilfe", "?"
        PRINT "info"
    CASE ELSE
        PRINT "unbekannt"
END SELECT
"""

SELECT_ELSE_FIRES_WHEN_NO_MATCH = """\
DIM x AS INTEGER
x = 99
SELECT CASE x
    CASE 1
        PRINT "eins"
    CASE 2 TO 10
        PRINT "klein"
    CASE ELSE
        PRINT "fallback"
END SELECT
"""

SELECT_NO_ELSE_NO_MATCH = """\
DIM x AS INTEGER
x = 99
PRINT "vor"
SELECT CASE x
    CASE 1
        PRINT "eins"
END SELECT
PRINT "nach"
"""

SELECT_FIRST_MATCH_WINS = """\
DIM x AS INTEGER
x = 5
SELECT CASE x
    CASE 1 TO 10
        PRINT "erster"
    CASE 5
        PRINT "zweiter (nie)"
END SELECT
"""

SELECT_IS_GT = """\
DIM x AS INTEGER
x = 150
SELECT CASE x
    CASE IS <= 0
        PRINT "negativ"
    CASE IS < 100
        PRINT "klein"
    CASE IS >= 100
        PRINT "gross"
END SELECT
"""

SELECT_IS_NEQ = """\
DIM x AS INTEGER
x = 5
SELECT CASE x
    CASE IS <> 5
        PRINT "anders"
    CASE ELSE
        PRINT "fuenf"
END SELECT
"""

SELECT_IS_MIXED_WITH_VALUE = """\
DIM x AS INTEGER
x = 3
SELECT CASE x
    CASE 1, IS > 100
        PRINT "eins-oder-gross"
    CASE 2 TO 5, IS = 10
        PRINT "klein-oder-zehn"
    CASE ELSE
        PRINT "anders"
END SELECT
"""

SELECT_NESTED = """\
DIM a AS INTEGER
DIM b AS INTEGER
a = 1
b = 20
SELECT CASE a
    CASE 1
        SELECT CASE b
            CASE 0 TO 10
                PRINT "a=1, b klein"
            CASE 11 TO 100
                PRINT "a=1, b mittel"
        END SELECT
    CASE 2
        PRINT "a=2"
END SELECT
"""

SELECT_SIDE_EFFECT_ONCE = """\
DIM aufrufe AS INTEGER
aufrufe = 0

FUNCTION zaehl() AS INTEGER
    aufrufe = aufrufe + 1
    RETURN 5
END FUNCTION

SELECT CASE zaehl()
    CASE 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
        PRINT "match"
    CASE ELSE
        PRINT "nope"
END SELECT
PRINT "aufrufe=", aufrufe
"""


@pytest.mark.parametrize("source,expected", [
    (SELECT_VALUE, "zwei\n"),
    (SELECT_LIST, "klein\n"),
    (SELECT_RANGE, "klein\n"),
    (SELECT_MIXED, "ausgewaehlt\n"),
    (SELECT_STRING, "info\n"),
    (SELECT_ELSE_FIRES_WHEN_NO_MATCH, "fallback\n"),
    (SELECT_NO_ELSE_NO_MATCH, "vor\nnach\n"),
    (SELECT_FIRST_MATCH_WINS, "erster\n"),
    (SELECT_NESTED, "a=1, b mittel\n"),
    (SELECT_IS_GT, "gross\n"),
    (SELECT_IS_NEQ, "fuenf\n"),
    (SELECT_IS_MIXED_WITH_VALUE, "klein-oder-zehn\n"),
])
def test_select_treewalker(run_gb, source, expected):
    assert run_gb(source) == expected


@pytest.mark.parametrize("source,expected", [
    (SELECT_VALUE, "zwei\n"),
    (SELECT_LIST, "klein\n"),
    (SELECT_RANGE, "klein\n"),
    (SELECT_MIXED, "ausgewaehlt\n"),
    (SELECT_STRING, "info\n"),
    (SELECT_ELSE_FIRES_WHEN_NO_MATCH, "fallback\n"),
    (SELECT_NO_ELSE_NO_MATCH, "vor\nnach\n"),
    (SELECT_FIRST_MATCH_WINS, "erster\n"),
    (SELECT_NESTED, "a=1, b mittel\n"),
    (SELECT_IS_GT, "gross\n"),
    (SELECT_IS_NEQ, "fuenf\n"),
    (SELECT_IS_MIXED_WITH_VALUE, "klein-oder-zehn\n"),
])
def test_select_vm(run_vm, source, expected):
    assert run_vm(source) == expected


def test_subject_evaluated_once_treewalker(run_gb):
    out = run_gb(SELECT_SIDE_EFFECT_ONCE)
    assert "match\n" in out
    assert "aufrufe= 1\n" in out


def test_subject_evaluated_once_vm(run_vm):
    out = run_vm(SELECT_SIDE_EFFECT_ONCE)
    assert "match\n" in out
    assert "aufrufe= 1\n" in out


def test_select_parse_error_double_else(run_gb):
    from drachenhauch.errors import ParseError
    src = """\
DIM x AS INTEGER
x = 1
SELECT CASE x
    CASE ELSE
        PRINT "a"
    CASE ELSE
        PRINT "b"
END SELECT
"""
    with pytest.raises(ParseError):
        run_gb(src)


def test_select_parse_error_case_after_else(run_gb):
    from drachenhauch.errors import ParseError
    src = """\
DIM x AS INTEGER
x = 1
SELECT CASE x
    CASE ELSE
        PRINT "a"
    CASE 1
        PRINT "b"
END SELECT
"""
    with pytest.raises(ParseError):
        run_gb(src)
