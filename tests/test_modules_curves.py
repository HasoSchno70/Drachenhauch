"""Tests fuer das curves-Modul (Animation-Curves)."""
import math
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_curves():
    assert load_module("curves")


# --- Lerp + Smoothstep ---------------------------------------------

def test_lerp_endpoints(call_builtin):
    assert call_builtin("curve_lerp", [10.0, 20.0, 0.0]) == 10.0
    assert call_builtin("curve_lerp", [10.0, 20.0, 1.0]) == 20.0


def test_lerp_midpoint(call_builtin):
    assert call_builtin("curve_lerp", [0.0, 100.0, 0.5]) == 50.0


def test_smoothstep_endpoints(call_builtin):
    assert call_builtin("curve_smoothstep", [0.0, 1.0, 0.0]) == 0.0
    assert call_builtin("curve_smoothstep", [0.0, 1.0, 1.0]) == 1.0


def test_smoothstep_midpoint_is_half(call_builtin):
    # 3*(0.5)^2 - 2*(0.5)^3 = 0.75 - 0.25 = 0.5
    assert abs(call_builtin("curve_smoothstep", [0.0, 1.0, 0.5]) - 0.5) < 1e-12


def test_smoothstep_clamps_below_edge0(call_builtin):
    assert call_builtin("curve_smoothstep", [0.0, 1.0, -10.0]) == 0.0


def test_smoothstep_clamps_above_edge1(call_builtin):
    assert call_builtin("curve_smoothstep", [0.0, 1.0, 99.0]) == 1.0


def test_smoothstep_zero_edge_diff_returns_zero(call_builtin):
    """Defensiv: edge0 == edge1 wuerde durch 0 dividieren -> liefert 0.0."""
    assert call_builtin("curve_smoothstep", [5.0, 5.0, 5.0]) == 0.0


def test_smootherstep_endpoints(call_builtin):
    assert call_builtin("curve_smootherstep", [0.0, 1.0, 0.0]) == 0.0
    assert call_builtin("curve_smootherstep", [0.0, 1.0, 1.0]) == 1.0


def test_smootherstep_midpoint(call_builtin):
    # Ken Perlin's Smootherstep: 6t^5 - 15t^4 + 10t^3, bei t=0.5:
    # 6*(0.5)^5 - 15*(0.5)^4 + 10*(0.5)^3 = 0.1875 - 0.9375 + 1.25 = 0.5
    assert abs(call_builtin("curve_smootherstep", [0.0, 1.0, 0.5]) - 0.5) < 1e-12


# --- Bezier ---------------------------------------------------------

def test_bezier_starts_at_p0(call_builtin):
    assert call_builtin("curve_bezier", [0.0, 1.0, 5.0, 7.0, 10.0]) == 1.0


def test_bezier_ends_at_p3(call_builtin):
    assert call_builtin("curve_bezier", [1.0, 1.0, 5.0, 7.0, 10.0]) == 10.0


def test_bezier_midpoint_with_aligned_handles(call_builtin):
    """P0=0, P1=1/3, P2=2/3, P3=1: Bezier mit linear-aufgereihten Handles
    -> Ergebnis ist linear, also B(0.5) = 0.5."""
    val = call_builtin("curve_bezier", [0.5, 0.0, 1.0/3, 2.0/3, 1.0])
    assert abs(val - 0.5) < 1e-12


def test_bezier_2d_returns_tuple(call_builtin):
    result = call_builtin("curve_bezier2", [0.5, 0.0, 0.0, 1.0, 1.0, 2.0, 1.0, 3.0, 0.0])
    assert isinstance(result, tuple)
    assert len(result) == 2


def test_bezier_2d_endpoints(call_builtin):
    start = call_builtin("curve_bezier2", [0.0, 10.0, 20.0, 0.0, 0.0, 0.0, 0.0, 30.0, 40.0])
    end = call_builtin("curve_bezier2", [1.0, 10.0, 20.0, 0.0, 0.0, 0.0, 0.0, 30.0, 40.0])
    assert start == (10.0, 20.0)
    assert end == (30.0, 40.0)


# --- Catmull-Rom ---------------------------------------------------

def test_catmull_passes_through_p1(call_builtin):
    """Bei t=0 ist CURVE_CATMULL = p1 (mit Default-Tension 0.5)."""
    val = call_builtin("curve_catmull", [0.0, 0.0, 5.0, 10.0, 15.0])
    assert abs(val - 5.0) < 1e-9


def test_catmull_passes_through_p2_at_t1(call_builtin):
    """Bei t=1 ist CURVE_CATMULL = p2."""
    val = call_builtin("curve_catmull", [1.0, 0.0, 5.0, 10.0, 15.0])
    assert abs(val - 10.0) < 1e-9


def test_catmull_2d_endpoints(call_builtin):
    start = call_builtin("curve_catmull2",
                         [0.0, 0.0, 0.0, 10.0, 10.0, 20.0, 20.0, 30.0, 30.0])
    end = call_builtin("curve_catmull2",
                       [1.0, 0.0, 0.0, 10.0, 10.0, 20.0, 20.0, 30.0, 30.0])
    assert abs(start[0] - 10.0) < 1e-9
    assert abs(start[1] - 10.0) < 1e-9
    assert abs(end[0] - 20.0) < 1e-9
    assert abs(end[1] - 20.0) < 1e-9


# --- Hermite -------------------------------------------------------

def test_hermite_endpoints(call_builtin):
    start = call_builtin("curve_hermite", [0.0, 5.0, 10.0, 0.0, 0.0])
    end = call_builtin("curve_hermite", [1.0, 5.0, 10.0, 0.0, 0.0])
    assert abs(start - 5.0) < 1e-12
    assert abs(end - 10.0) < 1e-12


def test_hermite_with_zero_tangents_is_smooth(call_builtin):
    """Mit m0=m1=0 ist Hermite eine Smoothstep-aehnliche Kurve --
    d.h. monoton steigend zwischen p0 und p1."""
    last = -1.0
    for i in range(11):
        t = i / 10.0
        v = call_builtin("curve_hermite", [t, 0.0, 1.0, 0.0, 0.0])
        assert v >= last - 1e-12  # monoton (numerisch tolerant)
        last = v


# --- Type-Checking -------------------------------------------------

def test_lerp_rejects_string(call_builtin):
    with pytest.raises(TypeMismatchError, match="erwartet"):
        call_builtin("curve_lerp", ["nope", 1.0, 0.5])


def test_bezier_rejects_bool(call_builtin):
    with pytest.raises(TypeMismatchError):
        call_builtin("curve_bezier", [True, 0.0, 0.0, 0.0, 0.0])


# --- Smoke: Modul-Aufruf via run_gb --------------------------------

def test_curves_in_full_program(run_gb):
    src = '''
IMPORT "curves"
PRINT CURVE_LERP(0.0, 100.0, 0.25)
PRINT CURVE_SMOOTHSTEP(0.0, 1.0, 0.5)
PRINT CURVE_BEZIER(0.0, 1.0, 2.0, 3.0, 4.0)
'''
    out = run_gb(src)
    # 25.0, 0.5, 1.0
    lines = [l.strip() for l in out.split("\n") if l.strip()]
    assert lines[0] == "25.0"
    assert lines[1] == "0.5"
    assert lines[2] == "1.0"
