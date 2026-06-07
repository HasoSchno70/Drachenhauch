"""Single-Line-IF: alle ':'-getrennten Statements nach THEN/ELSE gehoeren zum
jeweiligen Zweig (klassisches BASIC, konsistent mit dem Block-IF).

Frueher parste der Single-Line-IF nur EIN Statement nach THEN; ein ':' wurde
zum reinen Terminator und der Rest lief UNBEDINGT -- eine echte Falle
(`IF d>0 THEN d=d-1 : RETURN FALSE` returnte immer)."""


def test_then_colon_chain_conditional(run_gb):
    """Beide ':'-getrennten Statements nach THEN sind an die Bedingung gebunden."""
    src = '''
DIM x AS INTEGER
x = 1
IF x = 2 THEN PRINT "a" : PRINT "b"
PRINT "after"
'''
    assert run_gb(src) == "after\n"


def test_then_colon_chain_runs_when_true(run_gb):
    src = '''
DIM x AS INTEGER
x = 2
IF x = 2 THEN PRINT "a" : PRINT "b"
PRINT "after"
'''
    assert run_gb(src) == "a\nb\nafter\n"


def test_return_after_colon_is_conditional(run_gb):
    """Die historische Falle: RETURN nach ':' darf NICHT unbedingt laufen."""
    src = '''
FUNCTION f(d AS INTEGER) AS BOOLEAN
    IF d > 0 THEN d = d - 1 : RETURN FALSE
    RETURN TRUE
END FUNCTION
PRINT f(5)
PRINT f(0)
'''
    assert run_gb(src) == "FALSE\nTRUE\n"


def test_else_colon_chain(run_gb):
    """ELSE-Zweig sammelt seine ':'-getrennten Statements ebenfalls vollstaendig."""
    src = '''
DIM x AS INTEGER
x = 1
IF x = 1 THEN PRINT "y1" : PRINT "y2" ELSE PRINT "n1" : PRINT "n2"
IF x = 9 THEN PRINT "y1" : PRINT "y2" ELSE PRINT "n1" : PRINT "n2"
'''
    assert run_gb(src) == "y1\ny2\nn1\nn2\n"
