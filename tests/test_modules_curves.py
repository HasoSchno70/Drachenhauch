"""Tests fuer das curves-Modul (Animation-Curves).

Golden-Tests gegen die native Runtime `gbrt` (Stufe B): IMPORT "curves" + PRINT,
Soll-Ausgabe asserten. Frueher liefen sie via `call_builtin` direkt gegen die
Python-Builtin-Impl (in Phase 8 geloescht).
"""
import pytest

from gamebasic.errors import GBRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


# --- Lerp + Smoothstep ---------------------------------------------

def test_lerp_endpoints(run_gb):
    out = run_gb('IMPORT "curves"\n'
                 'PRINT CURVE_LERP(10.0, 20.0, 0.0)\n'
                 'PRINT CURVE_LERP(10.0, 20.0, 1.0)\n')
    assert _lines(out) == ["10.0", "20.0"]


def test_lerp_midpoint(run_gb):
    out = run_gb('IMPORT "curves"\nPRINT CURVE_LERP(0.0, 100.0, 0.5)\n')
    assert _lines(out) == ["50.0"]


def test_smoothstep_endpoints(run_gb):
    out = run_gb('IMPORT "curves"\n'
                 'PRINT CURVE_SMOOTHSTEP(0.0, 1.0, 0.0)\n'
                 'PRINT CURVE_SMOOTHSTEP(0.0, 1.0, 1.0)\n')
    assert _lines(out) == ["0.0", "1.0"]


def test_smoothstep_midpoint_is_half(run_gb):
    # 3*(0.5)^2 - 2*(0.5)^3 = 0.75 - 0.25 = 0.5
    out = run_gb('IMPORT "curves"\nPRINT CURVE_SMOOTHSTEP(0.0, 1.0, 0.5)\n')
    assert _lines(out) == ["0.5"]


def test_smoothstep_clamps_below_edge0(run_gb):
    out = run_gb('IMPORT "curves"\nPRINT CURVE_SMOOTHSTEP(0.0, 1.0, -10.0)\n')
    assert _lines(out) == ["0.0"]


def test_smoothstep_clamps_above_edge1(run_gb):
    out = run_gb('IMPORT "curves"\nPRINT CURVE_SMOOTHSTEP(0.0, 1.0, 99.0)\n')
    assert _lines(out) == ["1.0"]


def test_smoothstep_zero_edge_diff_returns_zero(run_gb):
    """Defensiv: edge0 == edge1 wuerde durch 0 dividieren -> liefert 0.0."""
    out = run_gb('IMPORT "curves"\nPRINT CURVE_SMOOTHSTEP(5.0, 5.0, 5.0)\n')
    assert _lines(out) == ["0.0"]


def test_smootherstep_endpoints(run_gb):
    out = run_gb('IMPORT "curves"\n'
                 'PRINT CURVE_SMOOTHERSTEP(0.0, 1.0, 0.0)\n'
                 'PRINT CURVE_SMOOTHERSTEP(0.0, 1.0, 1.0)\n')
    assert _lines(out) == ["0.0", "1.0"]


def test_smootherstep_midpoint(run_gb):
    # 6*(0.5)^5 - 15*(0.5)^4 + 10*(0.5)^3 = 0.5
    out = run_gb('IMPORT "curves"\nPRINT CURVE_SMOOTHERSTEP(0.0, 1.0, 0.5)\n')
    assert _lines(out) == ["0.5"]


# --- Bezier ---------------------------------------------------------

def test_bezier_starts_at_p0(run_gb):
    out = run_gb('IMPORT "curves"\nPRINT CURVE_BEZIER(0.0, 1.0, 5.0, 7.0, 10.0)\n')
    assert _lines(out) == ["1.0"]


def test_bezier_ends_at_p3(run_gb):
    out = run_gb('IMPORT "curves"\nPRINT CURVE_BEZIER(1.0, 1.0, 5.0, 7.0, 10.0)\n')
    assert _lines(out) == ["10.0"]


def test_bezier_midpoint_with_aligned_handles(run_gb):
    """P0=0, P1=1/3, P2=2/3, P3=1: linear aufgereihte Handles -> B(0.5)=0.5."""
    out = run_gb('IMPORT "curves"\n'
                 'PRINT CURVE_BEZIER(0.5, 0.0, 0.33333333333333333, '
                 '0.66666666666666667, 1.0)\n')
    assert _lines(out) == ["0.5"]


def test_bezier_2d_returns_tuple(run_gb):
    out = run_gb('IMPORT "curves"\n'
                 'PRINT CURVE_BEZIER2(0.5, 0.0, 0.0, 1.0, 1.0, 2.0, 1.0, 3.0, 0.0)\n')
    # Tupel-Ausgabe (x, y)
    assert _lines(out) == ["(1.5, 0.75)"]


def test_bezier_2d_endpoints(run_gb):
    out = run_gb('IMPORT "curves"\n'
                 'PRINT CURVE_BEZIER2(0.0, 10.0, 20.0, 0.0, 0.0, 0.0, 0.0, 30.0, 40.0)\n'
                 'PRINT CURVE_BEZIER2(1.0, 10.0, 20.0, 0.0, 0.0, 0.0, 0.0, 30.0, 40.0)\n')
    assert _lines(out) == ["(10.0, 20.0)", "(30.0, 40.0)"]


# --- Catmull-Rom ---------------------------------------------------

def test_catmull_passes_through_p1(run_gb):
    """Bei t=0 ist CURVE_CATMULL = p1 (mit Default-Tension 0.5)."""
    out = run_gb('IMPORT "curves"\nPRINT CURVE_CATMULL(0.0, 0.0, 5.0, 10.0, 15.0)\n')
    assert _lines(out) == ["5.0"]


def test_catmull_passes_through_p2_at_t1(run_gb):
    """Bei t=1 ist CURVE_CATMULL = p2."""
    out = run_gb('IMPORT "curves"\nPRINT CURVE_CATMULL(1.0, 0.0, 5.0, 10.0, 15.0)\n')
    assert _lines(out) == ["10.0"]


def test_catmull_2d_endpoints(run_gb):
    out = run_gb('IMPORT "curves"\n'
                 'PRINT CURVE_CATMULL2(0.0, 0.0, 0.0, 10.0, 10.0, 20.0, 20.0, 30.0, 30.0)\n'
                 'PRINT CURVE_CATMULL2(1.0, 0.0, 0.0, 10.0, 10.0, 20.0, 20.0, 30.0, 30.0)\n')
    assert _lines(out) == ["(10.0, 10.0)", "(20.0, 20.0)"]


# --- Hermite -------------------------------------------------------

def test_hermite_endpoints(run_gb):
    out = run_gb('IMPORT "curves"\n'
                 'PRINT CURVE_HERMITE(0.0, 5.0, 10.0, 0.0, 0.0)\n'
                 'PRINT CURVE_HERMITE(1.0, 5.0, 10.0, 0.0, 0.0)\n')
    assert _lines(out) == ["5.0", "10.0"]


def test_hermite_with_zero_tangents_is_smooth(run_gb):
    """Mit m0=m1=0 ist Hermite monoton steigend zwischen p0 und p1."""
    out = run_gb('IMPORT "curves"\n'
                 'DIM i AS INTEGER\n'
                 'FOR i = 0 TO 10\n'
                 '    PRINT CURVE_HERMITE(i / 10.0, 0.0, 1.0, 0.0, 0.0)\n'
                 'NEXT\n')
    vals = [float(x) for x in _lines(out)]
    last = -1.0
    for v in vals:
        assert v >= last - 1e-12
        last = v


# --- Type-Checking -------------------------------------------------

def test_lerp_rejects_string(run_gb):
    with pytest.raises(GBRuntimeError, match="erwartet"):
        run_gb('IMPORT "curves"\nPRINT CURVE_LERP("nope", 1.0, 0.5)\n')


def test_bezier_rejects_bool(run_gb):
    with pytest.raises(GBRuntimeError):
        run_gb('IMPORT "curves"\nPRINT CURVE_BEZIER(TRUE, 0.0, 0.0, 0.0, 0.0)\n')


# --- Smoke: Modul-Aufruf via run_gb --------------------------------

def test_curves_in_full_program(run_gb):
    out = run_gb('IMPORT "curves"\n'
                 'PRINT CURVE_LERP(0.0, 100.0, 0.25)\n'
                 'PRINT CURVE_SMOOTHSTEP(0.0, 1.0, 0.5)\n'
                 'PRINT CURVE_BEZIER(0.0, 1.0, 2.0, 3.0, 4.0)\n')
    assert _lines(out) == ["25.0", "0.5", "1.0"]
