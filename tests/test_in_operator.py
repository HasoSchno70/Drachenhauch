"""Tests fuer den IN-Operator: Mitgliedschaftspruefung auf String,
Tupel, Array und Map.
"""
import pytest


def test_in_string_substring(run_gb, run_vm):
    src = '''
PRINT "foo" IN "barfoobaz"
PRINT "xyz" IN "barfoobaz"
PRINT "" IN "anything"
'''
    expected = "TRUE\nFALSE\nTRUE\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_in_tuple(run_gb, run_vm):
    src = '''
PRINT 5 IN (1, 5, 9)
PRINT 7 IN (1, 5, 9)
PRINT "x" IN ("a", "b", "x", "z")
'''
    expected = "TRUE\nFALSE\nTRUE\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_in_array(run_gb, run_vm):
    src = '''
DIM nums[3] AS INTEGER
nums[0] = 10 : nums[1] = 20 : nums[2] = 30
PRINT 20 IN nums
PRINT 99 IN nums
'''
    expected = "TRUE\nFALSE\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_in_map_key(run_gb, run_vm):
    src = '''
DIM m AS MAP OF STRING
m.put("name", "Alice")
m.put("city", "Berlin")
PRINT "name" IN m
PRINT "city" IN m
PRINT "unknown" IN m
'''
    expected = "TRUE\nTRUE\nFALSE\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_in_negation_with_not(run_gb, run_vm):
    """Klassisches `NOT (x IN c)` fuer Anti-Mitgliedschaft."""
    src = '''
DIM banned[2] AS STRING
banned[0] = "spam"
banned[1] = "scam"
DIM word AS STRING
word = "ham"
IF NOT (word IN banned) THEN
    PRINT "allowed"
ELSE
    PRINT "blocked"
END IF
'''
    expected = "allowed\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_in_in_select_case(run_gb, run_vm):
    """`x IN (...)` als Bedingung in IF/SELECT funktioniert."""
    src = '''
DIM x AS INTEGER
x = 3
IF x IN (1, 2, 3) THEN
    PRINT "small"
ELIF x IN (4, 5, 6) THEN
    PRINT "medium"
ELSE
    PRINT "large"
END IF
'''
    expected = "small\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_in_string_needle_not_string_throws(run_gb, run_vm):
    """`5 IN "abc"` -- Mismatch, Needle muss String sein."""
    from gamebasic.errors import TypeMismatchError
    with pytest.raises(TypeMismatchError):
        run_gb('PRINT 5 IN "abc"')
    with pytest.raises(TypeMismatchError):
        run_vm('PRINT 5 IN "abc"')


def test_in_map_non_string_key_throws(run_gb, run_vm):
    from gamebasic.errors import TypeMismatchError
    src = '''
DIM m AS MAP OF INTEGER
m.put("a", 1)
PRINT 5 IN m
'''
    with pytest.raises(TypeMismatchError):
        run_gb(src)
    with pytest.raises(TypeMismatchError):
        run_vm(src)


def test_in_unsupported_type_throws(run_gb, run_vm):
    """`5 IN 42` -- weder String, Tuple, Array, noch Map -> Fehler."""
    from gamebasic.errors import TypeMismatchError
    with pytest.raises(TypeMismatchError):
        run_gb("PRINT 5 IN 42")
    with pytest.raises(TypeMismatchError):
        run_vm("PRINT 5 IN 42")


def test_in_chained_in_complex_expression(run_gb, run_vm):
    """IN kombiniert mit AND/OR."""
    src = '''
DIM x AS INTEGER
x = 5
IF x > 0 AND x IN (1, 5, 9) THEN PRINT "match"
'''
    expected = "match\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
