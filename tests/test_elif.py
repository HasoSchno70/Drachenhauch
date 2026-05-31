"""Tests fuer das ELIF-Alias (kuerzere Form von ELSEIF)."""


def test_elif_basic(run_gb, run_vm):
    src = '''
DIM x AS INTEGER
x = 5
IF x = 1 THEN
    PRINT "a"
ELIF x = 2 THEN
    PRINT "b"
ELIF x = 5 THEN
    PRINT "c"
ELSE
    PRINT "d"
END IF
'''
    expected = "c\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_elif_else_chain(run_gb, run_vm):
    """ELIF kann beliebig oft verkettet werden, ELSE bleibt optional."""
    src = '''
DIM g AS INTEGER
g = 90
IF g >= 90 THEN
    PRINT "A"
ELIF g >= 80 THEN
    PRINT "B"
ELIF g >= 70 THEN
    PRINT "C"
END IF
'''
    expected = "A\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_elif_and_elseif_can_mix(run_gb, run_vm):
    """ELIF und ELSEIF sind dasselbe Token -- Mischen ist erlaubt."""
    src = '''
DIM x AS INTEGER
x = 3
IF x = 1 THEN
    PRINT "1"
ELSEIF x = 2 THEN
    PRINT "2"
ELIF x = 3 THEN
    PRINT "3"
END IF
'''
    expected = "3\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
