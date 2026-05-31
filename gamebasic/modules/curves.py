"""Animation-Curves: Bezier, Catmull-Rom, Cubic-Hermite.

Komplementaer zum `tween`-Modul: tween macht Werte-Interpolation ueber
Zeit mit fixen Easing-Kurven (linear, in_quad, out_bounce, ...). Hier
liefern wir die freien Kurven, mit denen der User selbst Bewegungspfade
und Animationen baut.

Built-ins:
    CURVE_BEZIER(t, p0, p1, p2, p3)         -> FLOAT
        Cubic Bezier mit 4 Control-Points (1D).

    CURVE_BEZIER2(t, x0, y0, x1, y1, x2, y2, x3, y3) -> TUPLE(FLOAT, FLOAT)
        2D-Variante; gibt (x, y) zurueck.

    CURVE_CATMULL(t, p0, p1, p2, p3)        -> FLOAT
        Catmull-Rom-Spline durch 4 Punkte (interpoliert zwischen p1 und p2).

    CURVE_CATMULL2(t, x0, y0, x1, y1, x2, y2, x3, y3) -> TUPLE(FLOAT, FLOAT)

    CURVE_HERMITE(t, p0, p1, m0, m1)        -> FLOAT
        Cubic-Hermite zwischen p0 und p1 mit Tangenten m0, m1.

    CURVE_LERP(a, b, t)                     -> FLOAT
        Lineare Interpolation. Praktisch als Building-Block.

    CURVE_SMOOTHSTEP(edge0, edge1, x)       -> FLOAT
        GLSL-style smoothstep -- 0 unter edge0, 1 ueber edge1, sanfter
        S-Kurven-Uebergang dazwischen.

    CURVE_SMOOTHERSTEP(edge0, edge1, x)     -> FLOAT
        Wie smoothstep, aber mit kontinuierlicher 2. Ableitung (Ken Perlin).

Konvention:
    `t` ist ein normalisierter Parameter im Bereich [0, 1] (nicht-streng,
    der Bereich darf ueberschritten werden -- die Kurve extrapoliert dann).
    Die Funktionen sind pure -- kein State, kein Handle-Typ.
"""
from __future__ import annotations

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError


def _check_num(v, fn: str, label: str = "Zahl") -> float:
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn} erwartet {label}, erhalten {type(v).__name__}")
    return float(v)


# --- Lerp + Smoothstep ----------------------------------------------

@builtin("CURVE_LERP", arity=3)
def _b_lerp(a, b, t):
    a = _check_num(a, "CURVE_LERP", "a")
    b = _check_num(b, "CURVE_LERP", "b")
    t = _check_num(t, "CURVE_LERP", "t")
    return a + (b - a) * t


@builtin("CURVE_SMOOTHSTEP", arity=3)
def _b_smoothstep(edge0, edge1, x):
    e0 = _check_num(edge0, "CURVE_SMOOTHSTEP", "edge0")
    e1 = _check_num(edge1, "CURVE_SMOOTHSTEP", "edge1")
    xv = _check_num(x, "CURVE_SMOOTHSTEP", "x")
    if e1 == e0:
        return 0.0
    t = (xv - e0) / (e1 - e0)
    if t < 0.0:
        t = 0.0
    elif t > 1.0:
        t = 1.0
    return t * t * (3.0 - 2.0 * t)


@builtin("CURVE_SMOOTHERSTEP", arity=3)
def _b_smootherstep(edge0, edge1, x):
    e0 = _check_num(edge0, "CURVE_SMOOTHERSTEP", "edge0")
    e1 = _check_num(edge1, "CURVE_SMOOTHERSTEP", "edge1")
    xv = _check_num(x, "CURVE_SMOOTHERSTEP", "x")
    if e1 == e0:
        return 0.0
    t = (xv - e0) / (e1 - e0)
    if t < 0.0:
        t = 0.0
    elif t > 1.0:
        t = 1.0
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0)


# --- Bezier ---------------------------------------------------------

def _bezier_1d(t: float, p0: float, p1: float, p2: float, p3: float) -> float:
    """De-Casteljau-Form (numerisch stabiler als ausmultiplizieren).

    Cubic Bezier: B(t) = (1-t)^3 P0 + 3(1-t)^2 t P1 + 3(1-t) t^2 P2 + t^3 P3.
    """
    u = 1.0 - t
    return (u * u * u * p0
            + 3.0 * u * u * t * p1
            + 3.0 * u * t * t * p2
            + t * t * t * p3)


@builtin("CURVE_BEZIER", arity=5)
def _b_bezier(t, p0, p1, p2, p3):
    """Cubic Bezier 1D durch 4 Control-Points.

    P0 ist Start, P3 ist Ende, P1/P2 sind Tangenten-Handles."""
    return _bezier_1d(
        _check_num(t, "CURVE_BEZIER", "t"),
        _check_num(p0, "CURVE_BEZIER", "p0"),
        _check_num(p1, "CURVE_BEZIER", "p1"),
        _check_num(p2, "CURVE_BEZIER", "p2"),
        _check_num(p3, "CURVE_BEZIER", "p3"),
    )


@builtin("CURVE_BEZIER2", arity=9)
def _b_bezier2(t, x0, y0, x1, y1, x2, y2, x3, y3):
    """Cubic Bezier 2D -> Tupel (x, y). Praktisch fuer Bewegungspfade."""
    tv = _check_num(t, "CURVE_BEZIER2", "t")
    xs = (_check_num(x0, "CURVE_BEZIER2", "x0"),
          _check_num(x1, "CURVE_BEZIER2", "x1"),
          _check_num(x2, "CURVE_BEZIER2", "x2"),
          _check_num(x3, "CURVE_BEZIER2", "x3"))
    ys = (_check_num(y0, "CURVE_BEZIER2", "y0"),
          _check_num(y1, "CURVE_BEZIER2", "y1"),
          _check_num(y2, "CURVE_BEZIER2", "y2"),
          _check_num(y3, "CURVE_BEZIER2", "y3"))
    return (_bezier_1d(tv, *xs), _bezier_1d(tv, *ys))


# --- Catmull-Rom ----------------------------------------------------

def _catmull_1d(t: float, p0: float, p1: float, p2: float, p3: float,
                tension: float = 0.5) -> float:
    """Catmull-Rom-Spline. Klassische Form mit tension=0.5
    (entspricht "centripetal Catmull-Rom" wenn man die Knoten gleichmaessig
    nimmt, was wir hier tun)."""
    t2 = t * t
    t3 = t2 * t
    return (
        tension * (
            (2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3
        )
    )


@builtin("CURVE_CATMULL", arity=5)
def _b_catmull(t, p0, p1, p2, p3):
    """Catmull-Rom 1D. Interpoliert zwischen p1 und p2; p0 und p3 sind
    Tangenten-Stuetzen (Vorgaenger / Nachfolger)."""
    return _catmull_1d(
        _check_num(t, "CURVE_CATMULL", "t"),
        _check_num(p0, "CURVE_CATMULL", "p0"),
        _check_num(p1, "CURVE_CATMULL", "p1"),
        _check_num(p2, "CURVE_CATMULL", "p2"),
        _check_num(p3, "CURVE_CATMULL", "p3"),
    )


@builtin("CURVE_CATMULL2", arity=9)
def _b_catmull2(t, x0, y0, x1, y1, x2, y2, x3, y3):
    """Catmull-Rom 2D -> Tupel (x, y)."""
    tv = _check_num(t, "CURVE_CATMULL2", "t")
    xs = (_check_num(x0, "CURVE_CATMULL2", "x0"),
          _check_num(x1, "CURVE_CATMULL2", "x1"),
          _check_num(x2, "CURVE_CATMULL2", "x2"),
          _check_num(x3, "CURVE_CATMULL2", "x3"))
    ys = (_check_num(y0, "CURVE_CATMULL2", "y0"),
          _check_num(y1, "CURVE_CATMULL2", "y1"),
          _check_num(y2, "CURVE_CATMULL2", "y2"),
          _check_num(y3, "CURVE_CATMULL2", "y3"))
    return (_catmull_1d(tv, *xs), _catmull_1d(tv, *ys))


# --- Hermite --------------------------------------------------------

@builtin("CURVE_HERMITE", arity=5)
def _b_hermite(t, p0, p1, m0, m1):
    """Cubic-Hermite zwischen p0 und p1 mit Tangenten m0 (am Start) und
    m1 (am Ende). H(t) = h00*p0 + h10*m0 + h01*p1 + h11*m1 mit den
    klassischen Hermite-Basis-Funktionen."""
    t = _check_num(t, "CURVE_HERMITE", "t")
    p0 = _check_num(p0, "CURVE_HERMITE", "p0")
    p1 = _check_num(p1, "CURVE_HERMITE", "p1")
    m0 = _check_num(m0, "CURVE_HERMITE", "m0")
    m1 = _check_num(m1, "CURVE_HERMITE", "m1")
    t2 = t * t
    t3 = t2 * t
    h00 = 2.0 * t3 - 3.0 * t2 + 1.0
    h10 = t3 - 2.0 * t2 + t
    h01 = -2.0 * t3 + 3.0 * t2
    h11 = t3 - t2
    return h00 * p0 + h10 * m0 + h01 * p1 + h11 * m1
