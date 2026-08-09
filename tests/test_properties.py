"""Tests fuer PROPERTY GET/SET in Klassen.

Properties werden intern als Methoden mit Namen `__get_<name>` und
`__set_<name>` registriert. MemberAccess und MemberAssign dispatchen
zu diesen Methoden, wenn der Name eine deklarierte Property ist.
"""
import pytest


def test_property_basic_getter_setter(run_gb, run_vm):
    src = '''
CLASS Box
    DIM _v AS INTEGER

    PROPERTY GET v() AS INTEGER
        RETURN Self._v
    END PROPERTY

    PROPERTY SET v(value AS INTEGER)
        Self._v = value
    END PROPERTY
END CLASS

DIM b AS Box
b = NEW Box()
b.v = 42
PRINT b.v
b.v = 99
PRINT b.v
'''
    expected = "42\n99\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_property_with_validation(run_gb, run_vm):
    """Klassischer Use-Case: Setter clampt Werte."""
    src = '''
CLASS HP
    DIM _value AS INTEGER

    PROPERTY GET value() AS INTEGER
        RETURN Self._value
    END PROPERTY

    PROPERTY SET value(v AS INTEGER)
        IF v < 0 THEN v = 0
        IF v > 100 THEN v = 100
        Self._value = v
    END PROPERTY
END CLASS

DIM h AS HP
h = NEW HP()
h.value = 50
PRINT h.value
h.value = 200
PRINT h.value
h.value = -10
PRINT h.value
'''
    expected = "50\n100\n0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_property_read_only(run_gb, run_vm):
    """Property mit nur GET -> Setter-Aufruf wirft."""
    from drachenhauch.errors import DHRuntimeError
    src = '''
CLASS Const42
    PROPERTY GET answer() AS INTEGER
        RETURN 42
    END PROPERTY
END CLASS

DIM c AS Const42
c = NEW Const42()
PRINT c.answer
c.answer = 99
'''
    with pytest.raises(DHRuntimeError):
        run_gb(src)
    with pytest.raises(DHRuntimeError):
        run_vm(src)


def test_property_write_only(run_gb, run_vm):
    """Property mit nur SET -> Getter-Aufruf wirft."""
    from drachenhauch.errors import DHRuntimeError
    src = '''
CLASS WriteOnly
    DIM _value AS INTEGER

    PROPERTY SET value(v AS INTEGER)
        Self._value = v
    END PROPERTY
END CLASS

DIM w AS WriteOnly
w = NEW WriteOnly()
w.value = 5
PRINT w.value
'''
    with pytest.raises(DHRuntimeError):
        run_gb(src)
    with pytest.raises(DHRuntimeError):
        run_vm(src)


def test_property_computed(run_gb, run_vm):
    """Property kann berechnet sein -- kein einzelnes Backing-Field."""
    src = '''
CLASS Point
    DIM x AS FLOAT
    DIM y AS FLOAT

    PROPERTY GET length() AS FLOAT
        RETURN SQR(Self.x * Self.x + Self.y * Self.y)
    END PROPERTY
END CLASS

DIM p AS Point
p = NEW Point()
p.x = 3.0
p.y = 4.0
PRINT p.length
'''
    expected = "5.0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_property_inherited(run_gb, run_vm):
    """Property in Basisklasse -> auch von Subklasse zugreifbar."""
    src = '''
CLASS Shape
    DIM _name AS STRING

    PROPERTY GET name() AS STRING
        RETURN Self._name
    END PROPERTY

    PROPERTY SET name(v AS STRING)
        Self._name = "[" + v + "]"
    END PROPERTY
END CLASS

CLASS Circle EXTENDS Shape
    DIM r AS FLOAT
END CLASS

DIM c AS Circle
c = NEW Circle()
c.name = "circle"
PRINT c.name
'''
    expected = "[circle]\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_property_setter_in_method(run_gb, run_vm):
    """Setter kann aus einer Methode aufgerufen werden."""
    src = '''
CLASS Counter
    DIM _n AS INTEGER

    PROPERTY GET n() AS INTEGER
        RETURN Self._n
    END PROPERTY

    PROPERTY SET n(v AS INTEGER)
        Self._n = v
    END PROPERTY

    SUB Increment()
        Self.n = Self.n + 1
    END SUB
END CLASS

DIM c AS Counter
c = NEW Counter()
c.n = 5
c.Increment()
c.Increment()
PRINT c.n
'''
    expected = "7\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_property_does_not_collide_with_method(run_gb, run_vm):
    """User-Methode `Get()` mit Pascal-Namen bleibt unbeeinflusst (kein
    Konflikt mit dem internen `__get_xyz`-Naming)."""
    src = '''
CLASS Foo
    DIM val AS INTEGER

    FUNCTION Get() AS INTEGER
        RETURN Self.val * 2
    END FUNCTION
END CLASS

DIM f AS Foo
f = NEW Foo()
f.val = 21
PRINT f.Get()
'''
    expected = "42\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_property_set_and_field_distinct(run_gb, run_vm):
    """Property-Name und Backing-Field-Name muessen verschieden sein --
    sonst koennte der Setter nicht das Feld setzen ohne in eine
    Endlos-Schleife zu fallen. Hier nutzen wir die ueblicher Konvention
    `_x` als Backing-Field, `x` als Property."""
    src = '''
CLASS Counter
    DIM _x AS INTEGER

    PROPERTY GET x() AS INTEGER
        RETURN Self._x
    END PROPERTY

    PROPERTY SET x(v AS INTEGER)
        Self._x = v * 10
    END PROPERTY
END CLASS

DIM c AS Counter
c = NEW Counter()
c.x = 5
PRINT c.x
PRINT c._x
'''
    expected = "50\n50\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
