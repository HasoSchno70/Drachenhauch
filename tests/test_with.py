"""Tests fuer WITH ... END WITH-Block.

Innerhalb des Body ist `.member` Shortcut fuer `<with-target>.member`.
Cython-VM braucht nach BUILD_TUPLE/UNPACK_TUPLE-Recompile bereits den
neuen Stand -- WITH selbst fuegt keine neuen Ops hinzu, der Mechanismus
ist Compile-Zeit-Desugar.
"""
import pytest


def test_with_basic_member_assign(run_gb, run_vm):
    src = '''
CLASS Player
    DIM x AS INTEGER
    DIM y AS INTEGER
    DIM hp AS INTEGER
END CLASS

DIM p AS Player
p = NEW Player()
WITH p
    .x = 10
    .y = 20
    .hp = 100
END WITH
PRINT p.x, p.y, p.hp
'''
    expected = "10 20 100\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_with_member_in_expression(run_gb, run_vm):
    """`.member` als Read-Expression, nicht nur als Assign-Target."""
    src = '''
CLASS Vec
    DIM x AS FLOAT
    DIM y AS FLOAT
END CLASS

DIM v AS Vec
v = NEW Vec()
v.x = 3.0
v.y = 4.0
DIM len AS FLOAT
WITH v
    len = SQR(.x * .x + .y * .y)
END WITH
PRINT len
'''
    expected = "5.0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_with_target_evaluated_once(run_gb, run_vm):
    """Side-Effects im WITH-Target laufen genau einmal."""
    src = '''
CLASS Box
    DIM v AS INTEGER
END CLASS

DIM counter AS INTEGER
counter = 0

FUNCTION getbox() AS Box
    counter = counter + 1
    DIM b AS Box
    b = NEW Box()
    b.v = 42
    RETURN b
END FUNCTION

WITH getbox()
    .v = 100
    .v = .v + 1
    .v = .v + 1
END WITH
PRINT counter
'''
    expected = "1\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_with_compound_assign(run_gb, run_vm):
    """`.x += 5` im WITH-Block."""
    src = '''
CLASS Score
    DIM points AS INTEGER
END CLASS

DIM s AS Score
s = NEW Score()
s.points = 10
WITH s
    .points += 5
    .points *= 2
END WITH
PRINT s.points
'''
    expected = "30\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_nested_with_innermost_wins(run_gb, run_vm):
    """Verschachtelte WITHs -- innerster Block bezieht sich auf eigenes Ziel."""
    src = '''
CLASS Outer
    DIM tag AS STRING
    DIM inner AS Inner
END CLASS

CLASS Inner
    DIM val AS INTEGER
END CLASS

DIM o AS Outer
o = NEW Outer()
o.inner = NEW Inner()
WITH o
    .tag = "outer"
    WITH .inner
        .val = 99
    END WITH
END WITH
PRINT o.tag
PRINT o.inner.val
'''
    expected = "outer\n99\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_with_chains_subaccess(run_gb, run_vm):
    """`.inner.x = 1` chained access from WITH-Target."""
    src = '''
CLASS Inner
    DIM v AS INTEGER
END CLASS

CLASS Outer
    DIM inner AS Inner
END CLASS

DIM o AS Outer
o = NEW Outer()
o.inner = NEW Inner()
WITH o
    .inner.v = 7
END WITH
PRINT o.inner.v
'''
    expected = "7\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_with_dotted_name_invisible_to_user(run_gb, run_vm):
    """User-Code kann `__with_0` nicht sehen oder neu deklarieren --
    der Name ist Compiler-intern, nicht user-zugaenglich.
    Dieser Test verifiziert nur, dass User-Variablen nicht von WITH
    durcheinander gebracht werden.
    """
    src = '''
CLASS P
    DIM v AS INTEGER
END CLASS

DIM x AS INTEGER
x = 999
DIM p AS P
p = NEW P()
WITH p
    .v = 1
END WITH
PRINT x
'''
    expected = "999\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_punkt_zuweisung_im_einzeiligen_if(run_gb):
    """`IF ok THEN .hp = 0` in einem WITH-Block.

    Der Parser hatte den Fall schon vorgesehen, aber die Zuweisung
    verbrauchte den Zeilenabschluss, den das einzeilige IF danach selbst
    erwartete -- der Fehler erschien in der FOLGEZEILE und zeigte damit auf
    die falsche Stelle.
    """
    out = run_gb('''
CLASS P
    DIM hp AS INTEGER
END CLASS
DIM p AS P
p = NEW P()
p.hp = 5
DIM ok AS BOOLEAN
ok = TRUE
WITH p
    IF ok THEN .hp = 0
END WITH
PRINT p.hp
''')
    assert out.strip() == "0"


def test_punkt_zuweisung_haengt_an_der_bedingung(run_gb):
    """Gegenprobe: bei falscher Bedingung bleibt der Wert stehen."""
    out = run_gb('''
CLASS P
    DIM hp AS INTEGER
END CLASS
DIM p AS P
p = NEW P()
p.hp = 5
DIM ok AS BOOLEAN
ok = FALSE
WITH p
    IF ok THEN .hp = 0
END WITH
PRINT p.hp
''')
    assert out.strip() == "5"
