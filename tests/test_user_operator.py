"""Tests fuer Operator-Overloading auf User-Klassen.

Validierung in zwei Phasen: 1) Parser akzeptiert/rejectet OPERATOR-Decls,
2) BinaryOp-Dispatch in Tree-Walker und VM ruft die richtige Methode.

Wir testen alle drei Pfade (TW, Python-VM, Cython-VM) ueber `run_gb` /
`run_vm`, plus einige Pfade ueber den Compiler-AST direkt.
"""
import pytest

from gamebasic.errors import ParseError, GBRuntimeError


# --- Parser-Validierung ----------------------------------------------

def _parse(src):
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    return Parser(Lexer(src).tokenize()).parse()


def test_parser_accepts_plus_operator():
    src = """
CLASS A
    OPERATOR + (other AS A) AS A
        RETURN Self
    END OPERATOR
END CLASS
"""
    ast = _parse(src)
    methods = [m.name for m in ast.statements[0].methods]
    assert "__op_add__" in methods


@pytest.mark.parametrize("op,internal", [
    ("+", "__op_add__"), ("-", "__op_sub__"),
    ("*", "__op_mul__"), ("/", "__op_div__"),
    ("MOD", "__op_mod__"),
    ("=", "__op_eq__"), ("<>", "__op_ne__"),
    ("<", "__op_lt__"), (">", "__op_gt__"),
    ("<=", "__op_le__"), (">=", "__op_ge__"),
])
def test_parser_recognizes_all_operators(op, internal):
    src = f"""
CLASS A
    OPERATOR {op} (other AS A) AS A
        RETURN Self
    END OPERATOR
END CLASS
"""
    ast = _parse(src)
    assert ast.statements[0].methods[0].name == internal


def test_parser_rejects_zero_params():
    src = """
CLASS A
    OPERATOR + () AS A
        RETURN Self
    END OPERATOR
END CLASS
"""
    with pytest.raises(ParseError, match="genau 1 Parameter"):
        _parse(src)


def test_parser_rejects_two_params():
    src = """
CLASS A
    OPERATOR + (a AS A, b AS A) AS A
        RETURN Self
    END OPERATOR
END CLASS
"""
    with pytest.raises(ParseError, match="genau 1 Parameter"):
        _parse(src)


def test_parser_rejects_byref_param():
    src = """
CLASS A
    OPERATOR + (BYREF other AS A) AS A
        RETURN Self
    END OPERATOR
END CLASS
"""
    with pytest.raises(ParseError, match="BYREF"):
        _parse(src)


def test_parser_rejects_unknown_operator():
    src = """
CLASS A
    OPERATOR ^ (other AS A) AS A
        RETURN Self
    END OPERATOR
END CLASS
"""
    with pytest.raises(ParseError, match="erwartet einen der Operatoren"):
        _parse(src)


# --- Runtime: Tree-Walker + VM Identitaet ----------------------------

_MONEY_SRC = """
CLASS Money
    DIM cents AS INTEGER

    OPERATOR + (other AS Money) AS Money
        DIM r AS Money
        r = NEW Money()
        r.cents = Self.cents + other.cents
        RETURN r
    END OPERATOR

    OPERATOR - (other AS Money) AS Money
        DIM r AS Money
        r = NEW Money()
        r.cents = Self.cents - other.cents
        RETURN r
    END OPERATOR

    OPERATOR = (other AS Money) AS BOOLEAN
        RETURN Self.cents = other.cents
    END OPERATOR

    OPERATOR <> (other AS Money) AS BOOLEAN
        RETURN Self.cents <> other.cents
    END OPERATOR

    OPERATOR < (other AS Money) AS BOOLEAN
        RETURN Self.cents < other.cents
    END OPERATOR
END CLASS
"""


def test_tw_plus(run_gb):
    src = _MONEY_SRC + """
DIM a AS Money
DIM b AS Money
DIM r AS Money
a = NEW Money()
b = NEW Money()
a.cents = 100
b.cents = 250
r = a + b
PRINT r.cents
"""
    assert "350" in run_gb(src)


def test_tw_minus(run_gb):
    src = _MONEY_SRC + """
DIM a AS Money
DIM b AS Money
DIM r AS Money
a = NEW Money()
b = NEW Money()
a.cents = 500
b.cents = 175
r = a - b
PRINT r.cents
"""
    assert "325" in run_gb(src)


def test_tw_eq(run_gb):
    src = _MONEY_SRC + """
DIM a AS Money
DIM b AS Money
DIM c AS Money
a = NEW Money()
b = NEW Money()
c = NEW Money()
a.cents = 100
b.cents = 100
c.cents = 200
PRINT a = b
PRINT a = c
"""
    out = run_gb(src)
    lines = [l for l in out.split("\n") if l.strip()]
    assert lines == ["TRUE", "FALSE"]


def test_tw_neq(run_gb):
    src = _MONEY_SRC + """
DIM a AS Money
DIM b AS Money
a = NEW Money()
b = NEW Money()
a.cents = 100
b.cents = 200
PRINT a <> b
"""
    assert "TRUE" in run_gb(src)


def test_tw_lt(run_gb):
    src = _MONEY_SRC + """
DIM a AS Money
DIM b AS Money
a = NEW Money()
b = NEW Money()
a.cents = 50
b.cents = 100
PRINT a < b
PRINT b < a
"""
    out = run_gb(src)
    lines = [l for l in out.split("\n") if l.strip()]
    assert lines == ["TRUE", "FALSE"]


def test_vm_matches_tw_for_plus(run_gb, run_vm):
    src = _MONEY_SRC + """
DIM a AS Money
DIM b AS Money
DIM r AS Money
a = NEW Money()
b = NEW Money()
a.cents = 100
b.cents = 250
r = a + b
PRINT r.cents
"""
    assert run_gb(src) == run_vm(src)


def test_vm_matches_tw_for_eq(run_gb, run_vm):
    src = _MONEY_SRC + """
DIM a AS Money
DIM b AS Money
a = NEW Money()
b = NEW Money()
a.cents = 100
b.cents = 100
PRINT a = b
"""
    assert run_gb(src) == run_vm(src)


# --- Inheritance: geerbte Operator-Methode --------------------------

def test_inherited_operator_used(run_gb, run_vm):
    """Operator-Methoden werden ueber die Vererbungs-Kette gesucht --
    Child-Klassen erben sie automatisch (gleiche MRO wie normale Methoden)."""
    src = """
CLASS Base
    DIM x AS INTEGER
    OPERATOR + (other AS Base) AS Base
        DIM r AS Base
        r = NEW Base()
        r.x = Self.x + other.x
        RETURN r
    END OPERATOR
END CLASS

CLASS Child EXTENDS Base
END CLASS

DIM a AS Child
DIM b AS Child
DIM r AS Base
a = NEW Child()
b = NEW Child()
a.x = 5
b.x = 7
r = a + b
PRINT r.x
"""
    assert "12" in run_gb(src)
    assert run_gb(src) == run_vm(src)


# --- Mehrere Operator-Decls + Method-Chaining -----------------------

def test_chained_operators(run_gb):
    src = _MONEY_SRC + """
DIM a AS Money
DIM b AS Money
DIM c AS Money
DIM r AS Money
a = NEW Money()
b = NEW Money()
c = NEW Money()
a.cents = 100
b.cents = 200
c.cents = 50
r = a + b - c
PRINT r.cents
"""
    assert "250" in run_gb(src)


# --- Builtin-Path nicht beeintraechtigt -----------------------------

def test_int_int_still_works_after_introducing_ops(run_gb):
    """Sanity: User-Class-Op darf nicht den INT+INT-Pfad shadowen."""
    out = run_gb("PRINT 2 + 3\n")
    assert "5" in out


def test_string_concat_still_works(run_gb):
    out = run_gb('PRINT "hi" + "!"\n')
    assert "hi!" in out
