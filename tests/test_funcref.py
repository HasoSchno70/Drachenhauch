"""Tests fuer FUNCREF (First-class Function References).

Bare User-Function-Identifier in Expression-Position liefern eine
FUNCREF, die spaeter via `f(args)` aufgerufen werden kann. Closures
werden NICHT unterstuetzt (Body sieht nur Params + Globals).
"""
import pytest


def test_funcref_basic(run_gb, run_vm):
    src = '''
FUNCTION square(x AS INTEGER) AS INTEGER
    RETURN x * x
END FUNCTION

DIM f AS FUNCREF
f = square
PRINT f(5)
'''
    expected = "25\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_funcref_reassign(run_gb, run_vm):
    src = '''
FUNCTION square(x AS INTEGER) AS INTEGER
    RETURN x * x
END FUNCTION

FUNCTION cube(x AS INTEGER) AS INTEGER
    RETURN x * x * x
END FUNCTION

DIM f AS FUNCREF
f = square
PRINT f(4)
f = cube
PRINT f(3)
'''
    expected = "16\n27\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_funcref_passed_as_arg(run_gb, run_vm):
    """FUNCREF als Parameter -- klassischer Higher-Order-Use-Case."""
    src = '''
FUNCTION twice(f AS FUNCREF, x AS INTEGER) AS INTEGER
    RETURN f(f(x))
END FUNCTION

FUNCTION inc(x AS INTEGER) AS INTEGER
    RETURN x + 1
END FUNCTION

PRINT twice(inc, 5)
'''
    expected = "7\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_funcref_print(run_gb, run_vm):
    src = '''
FUNCTION foo(x AS INTEGER) AS INTEGER
    RETURN x
END FUNCTION

DIM f AS FUNCREF
f = foo
PRINT f
'''
    expected = "<FUNCREF foo>\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_funcref_no_closure_locals(run_gb, run_vm):
    """Lambdas/FuncRefs sehen keine Locals des umgebenden Scopes -- sie
    laufen wie eine globale Function. Globale CONST sind aber sichtbar.
    """
    src = '''
CONST FACTOR = 10

FUNCTION scale(x AS INTEGER) AS INTEGER
    RETURN x * FACTOR
END FUNCTION

DIM f AS FUNCREF
f = scale
PRINT f(7)
'''
    expected = "70\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_funcref_compose(run_gb, run_vm):
    """Kombination: FUNCREF an FUNCREF, beide gleich aufrufbar."""
    src = '''
FUNCTION add1(x AS INTEGER) AS INTEGER
    RETURN x + 1
END FUNCTION

FUNCTION mul2(x AS INTEGER) AS INTEGER
    RETURN x * 2
END FUNCTION

FUNCTION apply(f AS FUNCREF, g AS FUNCREF, x AS INTEGER) AS INTEGER
    RETURN g(f(x))
END FUNCTION

PRINT apply(add1, mul2, 5)
PRINT apply(mul2, add1, 5)
'''
    expected = "12\n11\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_funcref_rejects_non_funcref(run_gb, run_vm):
    from gamebasic.errors import GBRuntimeError
    src = '''
DIM f AS FUNCREF
f = 42
'''
    with pytest.raises(GBRuntimeError):
        run_gb(src)
    with pytest.raises(GBRuntimeError):
        run_vm(src)


def test_calling_non_callable_throws(run_gb, run_vm):
    """`x()` mit x = 42 wirft."""
    from gamebasic.errors import GBRuntimeError, GBRuntimeError
    src = '''
DIM x AS INTEGER
x = 42
PRINT x()
'''
    with pytest.raises((GBRuntimeError, GBRuntimeError)):
        run_gb(src)
    with pytest.raises((GBRuntimeError, GBRuntimeError)):
        run_vm(src)


def test_user_var_shadows_function(run_gb, run_vm):
    """Eine User-Variable mit gleichem Namen wie eine Function "verschattet"
    diese -- die Variable gewinnt beim Identifier-Lookup. Konsistent
    zwischen Tree-Walker und VM.
    """
    src = '''
FUNCTION foo(x AS INTEGER) AS INTEGER
    RETURN x
END FUNCTION

DIM foo AS INTEGER
foo = 99
PRINT foo
'''
    expected = "99\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
