"""Tests fuer Named-Arguments in SUB/FUNCTION/NEW-Aufrufen."""
import pytest

from gamebasic.errors import GBRuntimeError


@pytest.fixture(params=["tw", "vm"])
def run_either(request, run_gb, run_vm):
    return run_gb if request.param == "tw" else run_vm


# --- Grundlegende Aufruf-Formen -------------------------------------

def test_all_named(run_either):
    out = run_either(
        'SUB f(a AS INTEGER, b AS INTEGER)\n'
        '    PRINT a, b\n'
        'END SUB\n'
        'f(a: 1, b: 2)\n'
    )
    assert out.split() == ["1", "2"]


def test_named_can_reorder(run_either):
    out = run_either(
        'SUB f(a AS INTEGER, b AS INTEGER)\n'
        '    PRINT a, b\n'
        'END SUB\n'
        'f(b: 99, a: 7)\n'
    )
    assert out.split() == ["7", "99"]


def test_positional_then_named(run_either):
    out = run_either(
        'SUB f(a AS INTEGER, b AS INTEGER, c AS INTEGER)\n'
        '    PRINT a, b, c\n'
        'END SUB\n'
        'f(1, b: 2, c: 3)\n'
    )
    assert out.split() == ["1", "2", "3"]


# --- Mit Defaults zusammen ------------------------------------------

def test_named_skips_to_later_default(run_either):
    """Named-Arg auf einen Param mit Default - Slots dazwischen kriegen Default."""
    out = run_either(
        'SUB f(a AS INTEGER, b AS INTEGER = 10, c AS INTEGER = 20)\n'
        '    PRINT a, b, c\n'
        'END SUB\n'
        'f(1, c: 3)\n'        # b nimmt Default 10
    )
    assert out.split() == ["1", "10", "3"]


def test_named_only_with_defaults(run_either):
    out = run_either(
        'SUB f(a AS INTEGER = 1, b AS INTEGER = 2, c AS INTEGER = 3)\n'
        '    PRINT a, b, c\n'
        'END SUB\n'
        'f(c: 99)\n'         # a, b auf Defaults
    )
    assert out.split() == ["1", "2", "99"]


def test_default_with_string(run_either):
    out = run_either(
        'SUB g(name AS STRING, greet AS STRING = "Hallo", suffix AS STRING = "!")\n'
        '    PRINT greet + ", " + name + suffix\n'
        'END SUB\n'
        'g(name: "Anna")\n'
        'g("Bob", suffix: "?")\n'
    )
    assert out.split("\n")[0] == "Hallo, Anna!"
    assert out.split("\n")[1] == "Hallo, Bob?"


# --- Function (mit Return) ------------------------------------------

def test_named_args_in_function(run_either):
    out = run_either(
        'FUNCTION add(a AS INTEGER, b AS INTEGER = 10) AS INTEGER\n'
        '    RETURN a + b\n'
        'END FUNCTION\n'
        'PRINT add(a: 5, b: 7)\n'
        'PRINT add(b: 100, a: 1)\n'
        'PRINT add(2)\n'
    )
    assert out.split() == ["12", "101", "12"]


# --- NEW Class(named: ...) ------------------------------------------

def test_named_in_new_class(run_either):
    out = run_either(
        'CLASS P\n'
        '    DIM n AS STRING\n'
        '    DIM a AS INTEGER\n'
        '    SUB Init(n AS STRING, a AS INTEGER = 0)\n'
        '        Self_n = n\n'
        '        Self_a = a\n'
        '    END SUB\n'
        '    SUB Show()\n'
        '        PRINT Self_n, Self_a\n'
        '    END SUB\n'
        'END CLASS\n'.replace("Self_n", "MyName").replace("Self_a", "MyAge")
        + 'CLASS Person\n'
        '    DIM mname AS STRING\n'
        '    DIM mage AS INTEGER\n'
        '    SUB Init(name AS STRING, age AS INTEGER = 0)\n'
        '        mname = name\n'
        '        mage = age\n'
        '    END SUB\n'
        '    SUB Show()\n'
        '        PRINT mname, mage\n'
        '    END SUB\n'
        'END CLASS\n'
        'DIM p AS Person\n'
        'p = NEW Person(name: "Anna", age: 30)\n'
        'p.Show()\n'
        'p = NEW Person("Bob")\n'
        'p.Show()\n'
        'p = NEW Person(age: 50, name: "Cara")\n'
        'p.Show()\n'
    )
    lines = out.strip().split("\n")
    assert "Anna" in lines[0] and "30" in lines[0]
    assert "Bob" in lines[1] and "0" in lines[1]
    assert "Cara" in lines[2] and "50" in lines[2]


# --- Fehler-Faelle ---------------------------------------------------

def test_unknown_name_raises(run_either):
    with pytest.raises(GBRuntimeError, match="Parameter.*schwurbel"):
        run_either(
            'SUB f(a AS INTEGER)\n'
            'END SUB\n'
            'f(schwurbel: 1)\n'
        )


def test_duplicate_named_raises(run_either):
    with pytest.raises(GBRuntimeError, match="doppelt"):
        run_either(
            'SUB f(a AS INTEGER)\n'
            'END SUB\n'
            'f(a: 1, a: 2)\n'
        )


def test_positional_and_named_overlap_raises(run_either):
    """Slot via positional UND named ist auch 'doppelt belegt'."""
    with pytest.raises(GBRuntimeError, match="doppelt"):
        run_either(
            'SUB f(a AS INTEGER, b AS INTEGER)\n'
            'END SUB\n'
            'f(1, a: 2)\n'
        )


def test_positional_after_named_raises(run_either):
    with pytest.raises(GBRuntimeError, match="positional Argument nach Named"):
        run_either(
            'SUB f(a AS INTEGER, b AS INTEGER)\n'
            'END SUB\n'
            'f(a: 1, 2)\n'
        )


def test_missing_required_param_raises(run_either):
    with pytest.raises(GBRuntimeError, match="fehlt"):
        run_either(
            'SUB f(a AS INTEGER, b AS INTEGER)\n'
            'END SUB\n'
            'f(b: 2)\n'        # a fehlt
        )


def test_named_arg_on_builtin_raises(run_either):
    """Built-ins haben keine deklarierten Param-Namen -> Fehler."""
    with pytest.raises(GBRuntimeError,
                       match="Built-in|SUB/FUNCTION"):
        run_either('PRINT ABS(x: -5)\n')


# --- Equivalence-Test -----------------------------------------------

def test_tw_and_vm_match(run_gb, run_vm):
    src = (
        'SUB g(name AS STRING, age AS INTEGER, '
        'greet AS STRING = "Hi", excl AS STRING = ".")\n'
        '    PRINT greet + ", " + name + " (" + STR$(age) + ")" + excl\n'
        'END SUB\n'
        'g("Anna", 30)\n'
        'g(name: "Bob", age: 25)\n'
        'g("Cara", 40, excl: "!")\n'
        'g(name: "Dora", age: 50, greet: "Hey")\n'
    )
    assert run_gb(src) == run_vm(src)
