"""Tests fuer List-Comprehensions: `[expr FOR var IN container [WHERE filter]]`.

Liefert ein TUPLE der transformierten Werte. Iterierbar sind: STRING,
TUPLE, 1D-ARRAY, MAP (Keys).
"""
import pytest


def test_comp_basic_tuple_source(run_gb, run_vm):
    src = '''
DIM nums AS TUPLE
nums = (1, 2, 3, 4, 5)
DIM doubled AS TUPLE
doubled = [n * 2 FOR n IN nums]
PRINT doubled
'''
    expected = "(2, 4, 6, 8, 10)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_with_filter(run_gb, run_vm):
    src = '''
DIM nums AS TUPLE
nums = (1, 2, 3, 4, 5, 6)
DIM evens AS TUPLE
evens = [n FOR n IN nums WHERE n MOD 2 = 0]
PRINT evens
'''
    expected = "(2, 4, 6)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_empty_filter_result(run_gb, run_vm):
    src = '''
DIM nums AS TUPLE
nums = (1, 2, 3)
DIM nada AS TUPLE
nada = [n FOR n IN nums WHERE n > 100]
PRINT nada
PRINT nada.length()
'''
    expected = "()\n0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_over_string(run_gb, run_vm):
    """String iteriert ueber chars."""
    src = '''
DIM caps AS TUPLE
caps = [c FOR c IN "abc"]
PRINT caps
'''
    expected = "(a, b, c)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_over_array(run_gb, run_vm):
    src = '''
DIM nums[4] AS INTEGER
nums[0] = 10 : nums[1] = 20 : nums[2] = 30 : nums[3] = 40
DIM tripled AS TUPLE
tripled = [n * 3 FOR n IN nums]
PRINT tripled
'''
    expected = "(30, 60, 90, 120)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_uses_outer_variable(run_gb, run_vm):
    """Comp-Body kann auf Variablen aus dem aeusseren Scope zugreifen."""
    src = '''
DIM nums AS TUPLE
nums = (1, 2, 3, 4)
DIM threshold AS INTEGER
threshold = 2
DIM big AS TUPLE
big = [n FOR n IN nums WHERE n > threshold]
PRINT big
'''
    expected = "(3, 4)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_with_method_call_in_transform(run_gb, run_vm):
    src = '''
DIM names AS TUPLE
names = ("alice", "bob", "cilla")
DIM upper AS TUPLE
upper = [n.upper() FOR n IN names]
PRINT upper
'''
    expected = "(ALICE, BOB, CILLA)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_string_filter(run_gb, run_vm):
    """Filter auf String-Container -- nur Vokale."""
    src = r'''
DIM vowels AS TUPLE
vowels = [c FOR c IN "Hallo Welt" WHERE c IN "aeiouAEIOU"]
PRINT vowels
'''
    expected = "(a, o, e)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_var_does_not_leak(run_gb, run_vm):
    """Iter-Variable lebt nur im Comp-Kontext (Tree-Walker stellt
    den vorigen Wert wieder her wenn ueberschattet)."""
    src = '''
DIM x AS INTEGER
x = 999
DIM nums AS TUPLE
nums = (1, 2, 3)
DIM r AS TUPLE
r = [x * 2 FOR x IN nums]
PRINT x
PRINT r
'''
    # Im VM-Pfad ist x als Local-Slot der Comp registriert (anonymer Slot),
    # die globale x bleibt unangetastet. Im Tree-Walker save/restore.
    expected = "999\n(2, 4, 6)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_nested_in_function(run_gb, run_vm):
    src = '''
FUNCTION squares_of(t AS TUPLE) AS TUPLE
    RETURN [n * n FOR n IN t]
END FUNCTION

DIM r AS TUPLE
r = squares_of((1, 2, 3, 4))
PRINT r
'''
    expected = "(1, 4, 9, 16)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_destructuring(run_gb, run_vm):
    """Comp-Result kann via destructuring auseinandergenommen werden."""
    src = '''
DIM nums AS TUPLE
nums = (10, 20, 30)
DIM a AS INTEGER
DIM b AS INTEGER
DIM c AS INTEGER
(a, b, c) = [n + 1 FOR n IN nums]
PRINT a, b, c
'''
    expected = "11 21 31\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_comp_unsupported_iter_throws(run_gb, run_vm):
    from drachenhauch.errors import DHRuntimeError
    src = '''
DIM r AS TUPLE
r = [x FOR x IN 42]
'''
    with pytest.raises(DHRuntimeError):
        run_gb(src)
    with pytest.raises(DHRuntimeError):
        run_vm(src)
