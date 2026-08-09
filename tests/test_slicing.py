"""Tests fuer Slicing: `s[a:b]`, `arr[a:b]` etc.

Negative Indices und Step werden bewusst NICHT unterstuetzt -- konsistent
mit der existierenden strikten Index-Validierung.
"""
import pytest


# --- Strings ---------------------------------------------------------

def test_string_slice_basic(run_gb, run_vm):
    src = '''
DIM s AS STRING
s = "Hello World"
PRINT s[0:5]
PRINT s[6:11]
'''
    expected = "Hello\nWorld\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_slice_open_lo(run_gb, run_vm):
    src = 'PRINT "abcdef"[:3]'
    expected = "abc\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_slice_open_hi(run_gb, run_vm):
    src = 'PRINT "abcdef"[3:]'
    expected = "def\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_slice_open_both(run_gb, run_vm):
    """`s[:]` liefert eine Kopie des Strings."""
    src = '''
DIM s AS STRING
s = "kopie"
PRINT s[:]
'''
    expected = "kopie\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_slice_clamps_high_bound(run_gb, run_vm):
    """`hi > len(s)` wird auf len(s) geclampt -- kein Fehler."""
    src = 'PRINT "abc"[0:100]'
    expected = "abc\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_slice_empty_when_lo_ge_hi(run_gb, run_vm):
    src = 'PRINT "[" + "abc"[2:1] + "]"'
    expected = "[]\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_slice_with_variable_bounds(run_gb, run_vm):
    src = '''
DIM s AS STRING
s = "TestString"
DIM lo AS INTEGER
DIM hi AS INTEGER
lo = 4
hi = 10
PRINT s[lo:hi]
'''
    expected = "String\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_slice_negative_throws(run_gb, run_vm):
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError):
        run_gb('PRINT "abc"[-1:2]')
    with pytest.raises(DHRuntimeError):
        run_vm('PRINT "abc"[-1:2]')


# --- Arrays ---------------------------------------------------------

def test_array_slice_basic(run_gb, run_vm):
    src = '''
DIM a[5] AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO 4
    a[i] = i * 10
NEXT
DIM b AS ARRAY OF INTEGER
b = a[1:4]
PRINT b[0]
PRINT b[1]
PRINT b[2]
'''
    expected = "10\n20\n30\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_array_slice_full_copy(run_gb, run_vm):
    """`a[:]` liefert eine echte Kopie -- Mutation der Quelle aendert
    den Slice nicht."""
    src = '''
DIM a[3] AS INTEGER
a[0] = 1 : a[1] = 2 : a[2] = 3
DIM b AS ARRAY OF INTEGER
b = a[:]
a[1] = 999
PRINT b[1]
'''
    expected = "2\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_array_slice_string_array(run_gb, run_vm):
    src = '''
DIM s[3] AS STRING
s[0] = "a"
s[1] = "b"
s[2] = "c"
DIM t AS ARRAY OF STRING
t = s[1:]
PRINT t[0]
PRINT t[1]
'''
    expected = "b\nc\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_array_2d_slice_throws(run_gb, run_vm):
    """Multi-Dim-Arrays unterstuetzen kein Slicing."""
    from drachenhauch.errors import DHRuntimeError
    src = '''
DIM g[3, 3] AS INTEGER
DIM x AS ARRAY OF INTEGER
x = g[0:2]
'''
    with pytest.raises(DHRuntimeError):
        run_gb(src)
    with pytest.raises(DHRuntimeError):
        run_vm(src)


# --- Slice-Assign nicht unterstuetzt -------------------------------

def test_slice_assign_rejected(run_gb, run_vm):
    """`s[a:b] = ...` ist kein Slice-Assign, soll werfen."""
    from drachenhauch.errors import ParseError
    src = '''
DIM a[5] AS INTEGER
a[1:3] = 7
'''
    with pytest.raises(ParseError):
        run_gb(src)
    with pytest.raises(ParseError):
        run_vm(src)


# --- Praktischer Use-Case ------------------------------------------

def test_string_slice_for_parsing(run_gb, run_vm):
    """Klassischer Use-Case: erstes Wort extrahieren."""
    src = '''
DIM s AS STRING
s = "foo bar baz"
DIM space AS INTEGER
space = INSTR(s, " ", 0)
PRINT s[0:space]
PRINT s[space + 1:]
'''
    expected = "foo\nbar baz\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
