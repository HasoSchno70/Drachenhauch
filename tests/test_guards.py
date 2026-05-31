"""Tests fuer Pattern-Matching mit Guards in SELECT CASE.

`CASE matches WHERE expr` -- erst muss das Match-Pattern treffen, DANN
muss die WHERE-Expression truthy sein. Beide Bedingungen muessen erfuellt
sein, sonst wird der naechste Case probiert.
"""
import pytest


def test_guard_basic(run_gb, run_vm):
    src = '''
DIM x AS INTEGER
x = 5
DIM flag AS BOOLEAN
flag = TRUE

SELECT CASE x
    CASE 1 TO 10 WHERE flag
        PRINT "klein und flag"
    CASE 1 TO 10
        PRINT "klein ohne flag"
    CASE ELSE
        PRINT "gross"
END SELECT
'''
    expected = "klein und flag\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_guard_false_falls_through(run_gb, run_vm):
    """Guard FALSE -> Match wird verworfen, naechster Case probiert."""
    src = '''
DIM x AS INTEGER
x = 5
DIM flag AS BOOLEAN
flag = FALSE

SELECT CASE x
    CASE 1 TO 10 WHERE flag
        PRINT "wins"
    CASE 1 TO 10
        PRINT "fallback"
END SELECT
'''
    expected = "fallback\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_guard_multiple_cases(run_gb, run_vm):
    """Mehrere Cases mit Guards, plus CASE ohne Guard."""
    src = '''
DIM hp AS INTEGER
hp = 30
DIM healed AS BOOLEAN
healed = TRUE

SELECT CASE hp
    CASE IS <= 0
        PRINT "tot"
    CASE IS <= 50 WHERE healed
        PRINT "low aber bereits geheilt"
    CASE IS <= 50
        PRINT "low ungeheilt"
    CASE ELSE
        PRINT "ok"
END SELECT
'''
    expected = "low aber bereits geheilt\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_guard_can_use_subject(run_gb, run_vm):
    """Die WHERE-Expression kann auf normale Variablen zugreifen --
    die Subject-Variable ist auch da."""
    src = '''
DIM x AS INTEGER
x = 5
DIM minimum AS INTEGER
minimum = 3

SELECT CASE x
    CASE 1 TO 100 WHERE x > minimum
        PRINT "big enough"
    CASE 1 TO 100
        PRINT "in range but too small"
END SELECT
'''
    expected = "big enough\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_guard_with_value_match(run_gb, run_vm):
    src = '''
DIM cmd AS STRING
cmd = "save"
DIM dirty AS BOOLEAN
dirty = TRUE

SELECT CASE cmd
    CASE "save" WHERE dirty
        PRINT "saving..."
    CASE "save"
        PRINT "nothing to save"
    CASE "load"
        PRINT "loading..."
END SELECT
'''
    expected = "saving...\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_guard_with_value_match_clean(run_gb, run_vm):
    src = '''
DIM cmd AS STRING
cmd = "save"
DIM dirty AS BOOLEAN
dirty = FALSE

SELECT CASE cmd
    CASE "save" WHERE dirty
        PRINT "saving..."
    CASE "save"
        PRINT "nothing to save"
END SELECT
'''
    expected = "nothing to save\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_guard_with_in_operator(run_gb, run_vm):
    """Praktisch: Guard mit IN-Test."""
    src = '''
DIM tag AS STRING
tag = "admin"
DIM cmd AS STRING
cmd = "delete"

SELECT CASE cmd
    CASE "delete" WHERE tag IN ("admin", "moderator")
        PRINT "OK"
    CASE "delete"
        PRINT "Permission denied"
END SELECT
'''
    expected = "OK\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_guard_uses_subject_indirectly(run_gb, run_vm):
    """Subject-Wert kann in Guard via gleichnamige Variable benutzt werden."""
    src = '''
DIM n AS INTEGER
n = 50

SELECT CASE n
    CASE 1 TO 100 WHERE n MOD 2 = 0
        PRINT "gerade in 1-100"
    CASE 1 TO 100
        PRINT "ungerade in 1-100"
END SELECT
'''
    expected = "gerade in 1-100\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_guard_else_still_runs_when_no_case_matches(run_gb, run_vm):
    """CASE ELSE bleibt unbeeinflusst -- laeuft, wenn weder Match noch
    Guard zutrifft."""
    src = '''
DIM x AS INTEGER
x = 999

SELECT CASE x
    CASE 1, 2, 3 WHERE TRUE
        PRINT "small"
    CASE 100, 200 WHERE TRUE
        PRINT "med"
    CASE ELSE
        PRINT "fallback"
END SELECT
'''
    expected = "fallback\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
