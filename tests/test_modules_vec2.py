"""Tests fuer das vec2-Modul: 2D-Vektor mit Operator-Overloading.

Cython-VM braucht einen Recompile fuer die Vec2-Operator-Hooks im
OP_ADD/SUB/MUL/DIV-Pfad.
"""
import pytest


def test_vec2_new_and_print(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
v = VEC2_NEW(3.0, 4.0)
PRINT v
'''
    expected = "Vec2(3.0, 4.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_add(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(1.0, 2.0)
w = VEC2_NEW(3.0, 4.0)
PRINT v + w
'''
    expected = "Vec2(4.0, 6.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_sub(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(5.0, 8.0)
w = VEC2_NEW(2.0, 3.0)
PRINT v - w
'''
    expected = "Vec2(3.0, 5.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_scalar_mul(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
v = VEC2_NEW(3.0, 4.0)
PRINT v * 2.0
PRINT 2.0 * v
PRINT v * 0.5
'''
    expected = "Vec2(6.0, 8.0)\nVec2(6.0, 8.0)\nVec2(1.5, 2.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_scalar_div(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
v = VEC2_NEW(10.0, 20.0)
PRINT v / 2.0
'''
    expected = "Vec2(5.0, 10.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_length(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
v = VEC2_NEW(3.0, 4.0)
PRINT VEC2_LENGTH(v)
'''
    expected = "5.0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_normalize(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
v = VEC2_NEW(3.0, 4.0)
PRINT VEC2_NORMALIZE(v)
'''
    expected = "Vec2(0.6, 0.8)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_dot(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(2.0, 3.0)
w = VEC2_NEW(4.0, -1.0)
PRINT VEC2_DOT(v, w)
'''
    expected = "5.0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_distance(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM a AS VEC2
DIM b AS VEC2
a = VEC2_NEW(0.0, 0.0)
b = VEC2_NEW(3.0, 4.0)
PRINT VEC2_DISTANCE(a, b)
'''
    expected = "5.0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_lerp(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM a AS VEC2
DIM b AS VEC2
a = VEC2_NEW(0.0, 0.0)
b = VEC2_NEW(10.0, 20.0)
PRINT VEC2_LERP(a, b, 0.5)
'''
    expected = "Vec2(5.0, 10.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_equality(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM a AS VEC2
DIM b AS VEC2
DIM c AS VEC2
a = VEC2_NEW(1.0, 2.0)
b = VEC2_NEW(1.0, 2.0)
c = VEC2_NEW(1.0, 3.0)
PRINT a = b
PRINT a = c
PRINT a <> c
'''
    expected = "TRUE\nFALSE\nTRUE\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_chained_ops(run_gb, run_vm):
    """Komplexere Ausdruecke -- (v + w) * 2 - v."""
    src = '''
IMPORT "vec2"
DIM v AS VEC2
DIM w AS VEC2
v = VEC2_NEW(1.0, 1.0)
w = VEC2_NEW(2.0, 3.0)
PRINT (v + w) * 2.0 - v
'''
    expected = "Vec2(5.0, 7.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_reflect(run_gb, run_vm):
    """Reflexion an x-Achse-Normale."""
    src = '''
IMPORT "vec2"
DIM v AS VEC2
DIM n AS VEC2
v = VEC2_NEW(3.0, 4.0)
n = VEC2_NEW(0.0, 1.0)
PRINT VEC2_REFLECT(v, n)
'''
    expected = "Vec2(3.0, -4.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_perp(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
v = VEC2_NEW(1.0, 0.0)
PRINT VEC2_PERP(v)
'''
    expected = "Vec2(-0.0, 1.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_from_angle(run_gb, run_vm):
    """0 rad == Vec2(1, 0); Pi/2 ~= Vec2(0, 1) (up to floating-point)."""
    src = '''
IMPORT "vec2"
PRINT VEC2_FROM_ANGLE(0.0, 1.0)
'''
    expected = "Vec2(1.0, 0.0)\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_vec2_x_y_components(run_gb, run_vm):
    src = '''
IMPORT "vec2"
DIM v AS VEC2
v = VEC2_NEW(3.5, -2.0)
PRINT VEC2_X(v)
PRINT VEC2_Y(v)
'''
    expected = "3.5\n-2.0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
