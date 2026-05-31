"""Tests fuer Method-Syntax auf Built-in-Containern.

`s.upper()`, `arr.length()`, `m.has(k)` etc. dispatchen zu den
entsprechenden BUILTIN-Funktionen mit dem Receiver als erstem Argument.
"""
import pytest


# --- Strings ---------------------------------------------------------

def test_string_upper_lower(run_gb, run_vm):
    src = '''
DIM s AS STRING
s = "Hallo"
PRINT s.upper()
PRINT s.lower()
'''
    expected = "HALLO\nhallo\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_length(run_gb, run_vm):
    src = '''
PRINT "abc".length()
PRINT "".length()
'''
    expected = "3\n0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_trim(run_gb, run_vm):
    src = '''
PRINT "[" + "  hi  ".trim() + "]"
'''
    expected = "[hi]\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_left_right_mid(run_gb, run_vm):
    src = '''
DIM s AS STRING
s = "GameBasic"
PRINT s.left(4)
PRINT s.right(5)
PRINT s.mid(4, 5)
'''
    expected = "Game\nBasic\nBasic\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_indexof(run_gb, run_vm):
    src = '''
PRINT "Hello World".indexof("World", 0)
PRINT "Hello World".indexof("Xyz", 0)
'''
    expected = "6\n-1\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_replace(run_gb, run_vm):
    src = 'PRINT "foo bar".replace("bar", "baz")'
    expected = "foo baz\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_string_chained_methods(run_gb, run_vm):
    """Method-Chain: `s.trim().upper()` hintereinander."""
    src = 'PRINT "  hi  ".trim().upper()'
    expected = "HI\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Arrays ----------------------------------------------------------

def test_array_length(run_gb, run_vm):
    src = '''
DIM a[7] AS INTEGER
PRINT a.length()
'''
    expected = "7\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Map -------------------------------------------------------------

def test_map_methods(run_gb, run_vm):
    src = '''
DIM m AS MAP OF STRING
m.put("name", "Alice")
m.put("city", "Berlin")
PRINT m.size()
PRINT m.has("name")
PRINT m.get("name")
PRINT m.has("unknown")
m.remove("name")
PRINT m.size()
m.clear()
PRINT m.size()
'''
    expected = "2\nTRUE\nAlice\nFALSE\n1\n0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_map_getor_default(run_gb, run_vm):
    src = '''
DIM m AS MAP OF INTEGER
m.put("x", 42)
PRINT m.getor("x", 0)
PRINT m.getor("y", 99)
'''
    expected = "42\n99\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Fehler ----------------------------------------------------------

def test_unknown_method_throws(run_gb, run_vm):
    from gamebasic.errors import GBRuntimeError
    src = 'PRINT "abc".unknown_method()'
    with pytest.raises(GBRuntimeError):
        run_gb(src)
    with pytest.raises(GBRuntimeError):
        run_vm(src)


def test_method_on_int_throws(run_gb, run_vm):
    """`(42).length()` macht keinen Sinn -- klarer Fehler."""
    from gamebasic.errors import GBRuntimeError
    src = '''
DIM x AS INTEGER
x = 42
PRINT x.length()
'''
    with pytest.raises(GBRuntimeError):
        run_gb(src)
    with pytest.raises(GBRuntimeError):
        run_vm(src)


# --- Mit User-Klasse: kein Konflikt ---------------------------------

def test_user_class_methods_unaffected(run_gb, run_vm):
    """User-Klassen mit eigenen Methoden bleiben unangetastet."""
    src = '''
CLASS Foo
    FUNCTION upper() AS STRING
        RETURN "user-upper"
    END FUNCTION
END CLASS

DIM f AS Foo
f = NEW Foo()
PRINT f.upper()
'''
    expected = "user-upper\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
