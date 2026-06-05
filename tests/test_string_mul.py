"""Tests fuer String-Multiplikation: `s * n` und `n * s`."""
import pytest


def test_string_mul_basic(run_gb, run_vm):
    src = '''
PRINT "ab" * 3
PRINT "-" * 10
'''
    expected = "ababab\n----------\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_mul_int_first(run_gb, run_vm):
    """3 * "ab" muss genauso funktionieren wie "ab" * 3."""
    src = 'PRINT 3 * "ab"'
    expected = "ababab\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_mul_zero(run_gb, run_vm):
    src = 'PRINT "x" * 0'
    expected = "\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_mul_negative_yields_empty(run_gb, run_vm):
    """Negative Counts liefern leeren String -- konsistent zu Python.
    (`"x" * -1` wirft NICHT, weil es als 'wiederhole 0-mal' gelesen wird.)
    """
    src = 'PRINT "[" + "x" * -3 + "]"'
    expected = "[]\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_mul_with_variable(run_gb, run_vm):
    src = '''
DIM n AS INTEGER
n = 5
PRINT "*" * n
'''
    expected = "*****\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_mul_rejects_float(run_gb, run_vm):
    """`"x" * 3.0` wird abgelehnt -- nur strikt INTEGER."""
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError):
        run_gb('PRINT "x" * 3.0')
    with pytest.raises(GBRuntimeError):
        run_vm('PRINT "x" * 3.0')


def test_string_mul_rejects_bool(run_gb, run_vm):
    """`"x" * TRUE` wird abgelehnt -- Bool ist keine Zahl in GB."""
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError):
        run_gb('PRINT "x" * TRUE')
    with pytest.raises(GBRuntimeError):
        run_vm('PRINT "x" * TRUE')


def test_string_mul_for_separator_line(run_gb, run_vm):
    """Praktischer Use-Case: Trennlinien."""
    src = '''
PRINT "=" * 30
PRINT "Header"
PRINT "=" * 30
'''
    expected = "==============================\nHeader\n==============================\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
