"""Tests fuer:
  - Doppelpunkt als Statement-Separator (`x = 1 : y = 2`)
  - `Self` als Identifier in Methoden
  - Implizite Methoden-Aufrufe innerhalb einer Klasse
"""
import pytest

from gamebasic.errors import GBRuntimeError


@pytest.fixture(params=["tw", "vm"])
def run_either(request, run_gb, run_vm):
    return run_gb if request.param == "tw" else run_vm


# --- Doppelpunkt-Separator ---------------------------------------------

def test_colon_separates_two_assignments(run_either):
    out = run_either(
        'DIM x AS INTEGER : DIM y AS INTEGER\n'
        'x = 1 : y = 2\n'
        'PRINT x\n'
        'PRINT y\n'
    )
    assert out.split() == ["1", "2"]


def test_colon_chain_three_statements(run_either):
    out = run_either(
        'DIM a AS INTEGER : DIM b AS INTEGER : DIM c AS INTEGER\n'
        'a = 1 : b = 2 : c = 3\n'
        'PRINT a + b + c\n'
    )
    assert out.strip() == "6"


def test_colon_in_sub_body(run_either):
    out = run_either(
        'SUB Greet(name AS STRING) : PRINT "Hi " + name : END SUB\n'
        'Greet("Anna")\n'
    )
    assert out.strip() == "Hi Anna"


def test_empty_colon_is_no_op(run_either):
    """Mehrere Doppelpunkte hintereinander sind harmlos."""
    out = run_either(
        'DIM x AS INTEGER\n'
        ' : : :\n'
        'x = 5\n'
        'PRINT x\n'
    )
    assert out.strip() == "5"


# --- Self in Methoden ---------------------------------------------------

def test_self_in_method_returns_instance(run_either):
    out = run_either(
        'CLASS Foo\n'
        '    DIM x AS INTEGER\n'
        '    SUB Init() : x = 42 : END SUB\n'
        '    FUNCTION Get() AS INTEGER : RETURN Self.x : END FUNCTION\n'
        'END CLASS\n'
        'DIM f AS Foo : f = NEW Foo()\n'
        'PRINT f.Get()\n'
    )
    assert out.strip() == "42"


def test_self_outside_method_treated_as_normal_var(run_gb):
    """Ausserhalb eines Methodenkontexts ist `Self` ein normaler
    (undeklarierter) Identifier - sollte einen Fehler werfen."""
    with pytest.raises(GBRuntimeError):
        run_gb('PRINT Self\n')


# --- Implizite Methoden-Aufrufe -----------------------------------------

def test_implicit_method_call_in_init(run_either):
    """Method aus Init aufrufen ohne Self.-Praefix."""
    out = run_either(
        'CLASS Counter\n'
        '    DIM v AS INTEGER\n'
        '    SUB Init() : v = 0 : Reset() : END SUB\n'
        '    SUB Reset() : v = 7 : END SUB\n'
        'END CLASS\n'
        'DIM c AS Counter : c = NEW Counter()\n'
        'PRINT c.v\n'
    )
    assert out.strip() == "7"


def test_implicit_method_call_in_other_method(run_either):
    out = run_either(
        'CLASS Foo\n'
        '    DIM x AS INTEGER\n'
        '    SUB Init() : x = 0 : END SUB\n'
        '    SUB Inc() : x = x + 1 : END SUB\n'
        '    FUNCTION DoubleInc() AS INTEGER\n'
        '        Inc() : Inc()\n'
        '        RETURN x\n'
        '    END FUNCTION\n'
        'END CLASS\n'
        'DIM f AS Foo : f = NEW Foo()\n'
        'PRINT f.DoubleInc()\n'
    )
    assert out.strip() == "2"


def test_implicit_call_resolves_inherited_methods(run_either):
    """Aufruf einer geerbten Methode aus einer Subklassen-Methode."""
    out = run_either(
        'CLASS Base\n'
        '    DIM val AS INTEGER\n'
        '    SUB Init() : val = 0 : END SUB\n'
        '    SUB SetTo(n AS INTEGER) : val = n : END SUB\n'
        'END CLASS\n'
        'CLASS Child EXTENDS Base\n'
        '    SUB DoubleSet(n AS INTEGER)\n'
        '        SetTo(n * 2)\n'
        '    END SUB\n'
        'END CLASS\n'
        'DIM c AS Child : c = NEW Child()\n'
        'c.DoubleSet(5)\n'
        'PRINT c.val\n'
    )
    assert out.strip() == "10"


def test_implicit_method_overrides_global_function(run_either):
    """Wenn es eine globale Funktion mit gleichem Namen UND eine Methode
    gibt, gewinnt die Methode innerhalb der Klasse."""
    out = run_either(
        'SUB Hello() : PRINT "global" : END SUB\n'
        'CLASS Box\n'
        '    SUB Test() : Hello() : END SUB\n'
        '    SUB Hello() : PRINT "method" : END SUB\n'
        'END CLASS\n'
        'DIM b AS Box : b = NEW Box()\n'
        'b.Test()\n'
        'Hello()\n'
    )
    assert out.split("\n")[0] == "method"
    assert out.split("\n")[1] == "global"


# --- Bench-Equivalence --------------------------------------------------

def test_tw_and_vm_match_with_self(run_gb, run_vm):
    """Beide Pfade liefern gleiche Ausgabe bei Self-Nutzung."""
    src = (
        'CLASS Box\n'
        '    DIM v AS INTEGER\n'
        '    SUB Init() : v = 5 : END SUB\n'
        '    SUB Inc() : v = v + 1 : END SUB\n'
        '    FUNCTION Triple() AS INTEGER\n'
        '        Inc() : Inc()\n'
        '        RETURN Self.v * 3\n'
        '    END FUNCTION\n'
        'END CLASS\n'
        'DIM b AS Box : b = NEW Box()\n'
        'PRINT b.Triple()\n'
        'PRINT b.v\n'
    )
    assert run_gb(src) == run_vm(src)
