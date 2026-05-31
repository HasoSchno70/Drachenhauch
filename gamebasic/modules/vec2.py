"""Vec2-Modul fuer GameBasic -- 2D-Vektor mit Operator-Overloading.

Im Gegensatz zu den `PHYSICS_NORM_X/Y` und `PHYSICS_REFLECT_X/Y`-Built-ins
liefert `vec2` einen ECHTEN Vektor-Typ, der mit den arithmetischen
Operatoren `+`, `-`, `*`, `/`, `=`, `<>` direkt verwendet werden kann.

  IMPORT "vec2"
  DIM v AS VEC2
  DIM w AS VEC2
  v = VEC2_NEW(3.0, 4.0)
  w = VEC2_NEW(1.0, 2.0)
  PRINT v + w           ' Vec2(4.0, 6.0)
  PRINT v * 2.0         ' Vec2(6.0, 8.0)
  PRINT VEC2_LENGTH(v)  ' 5.0

Werte sind immutable -- jede Operation erzeugt ein neues VEC2. So gibt es
keine Aliasing-Falle, wenn man `w = v` macht und dann `w` modifiziert.

Operator-Dispatch passiert im Interpreter (`_eval_BinaryOp`) und in beiden
VMs (`OP.ADD/SUB/MUL/DIV/EQ/NEQ`) -- der Check `isinstance(a, _Vec2)`
laeuft VOR den Standard-Pfaden, sodass keine Type-Mismatch-Fehler
geworfen werden, bevor wir die Vec2-Form versuchen.
"""
from __future__ import annotations

import math as _math

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type, register_operators


class _Vec2:
    """Immutable 2D-Vektor. `x` und `y` als FLOAT."""
    __slots__ = ("x", "y")

    def __init__(self, x: float, y: float):
        # Strikt FLOAT -- INTs werden silently zu float konvertiert,
        # Bools werden abgelehnt (gleiche Linie wie der Rest der Sprache).
        if isinstance(x, bool) or not isinstance(x, (int, float)):
            raise TypeMismatchError(f"VEC2: x muss eine Zahl sein, nicht {type(x).__name__}")
        if isinstance(y, bool) or not isinstance(y, (int, float)):
            raise TypeMismatchError(f"VEC2: y muss eine Zahl sein, nicht {type(y).__name__}")
        self.x = float(x)
        self.y = float(y)

    def __repr__(self):
        return f"Vec2({self.x}, {self.y})"

    def __eq__(self, other):
        if not isinstance(other, _Vec2):
            return NotImplemented
        return self.x == other.x and self.y == other.y

    def __ne__(self, other):
        result = self.__eq__(other)
        if result is NotImplemented:
            return result
        return not result

    def __hash__(self):
        return hash((self.x, self.y))


register_type("vec2", _Vec2)


# --- Operator-Handler (Registry-driven Dispatch) --------------------
#
# Tree-Walker und beide VMs rufen `modules.dispatch_binary_op` vor ihrem
# Standard-Pfad. Wenn type(a) oder type(b) zu _Vec2 zaehlt, dispatcht
# die Registry hierhin. Jeder Handler validiert beide Operanden und
# wirft, wenn die Kombination unzulaessig ist.

def _op_add(a, b):
    if isinstance(a, _Vec2) and isinstance(b, _Vec2):
        return _Vec2(a.x + b.x, a.y + b.y)
    raise TypeMismatchError("VEC2 + : beide Operanden muessen VEC2 sein")


def _op_sub(a, b):
    if isinstance(a, _Vec2) and isinstance(b, _Vec2):
        return _Vec2(a.x - b.x, a.y - b.y)
    raise TypeMismatchError("VEC2 - : beide Operanden muessen VEC2 sein")


def _op_mul(a, b):
    """Skalar-Multiplikation: VEC2 * Zahl oder Zahl * VEC2.

    Beide Reihenfolgen sind erlaubt -- der Handler erkennt sie selbst,
    weil die Registry nicht zwischen `a` und `b` unterscheidet."""
    if isinstance(a, _Vec2) and isinstance(b, (int, float)) and not isinstance(b, bool):
        return _Vec2(a.x * b, a.y * b)
    if isinstance(b, _Vec2) and isinstance(a, (int, float)) and not isinstance(a, bool):
        return _Vec2(b.x * a, b.y * a)
    raise TypeMismatchError("VEC2 * : erwartet VEC2 * Zahl oder Zahl * VEC2")


def _op_div(a, b):
    if isinstance(a, _Vec2) and isinstance(b, (int, float)) and not isinstance(b, bool):
        if b == 0:
            raise GBRuntimeError("Division durch 0")
        return _Vec2(a.x / b, a.y / b)
    raise TypeMismatchError("VEC2 / : erwartet VEC2 / Zahl")


# `=` und `<>` sind nicht hier -- _Vec2 hat __eq__/__ne__, die normale
# Python-Equality funktioniert (== liefert False wenn other nicht _Vec2 ist,
# was dem alten Verhalten entspricht). VMs nutzen `a == b` direkt.

register_operators(_Vec2, {
    "+": _op_add,
    "-": _op_sub,
    "*": _op_mul,
    "/": _op_div,
})


# --- Konstruktion + Komponenten-Zugriff -----------------------------

@builtin("VEC2_NEW", arity=2, types=("num", "num"))
def _b_new(x, y):
    return _Vec2(x, y)


@builtin("VEC2_ZERO", arity=0)
def _b_zero():
    return _Vec2(0.0, 0.0)


@builtin("VEC2_X", arity=1)
def _b_x(v):
    return _require_vec2(v, "VEC2_X").x


@builtin("VEC2_Y", arity=1)
def _b_y(v):
    return _require_vec2(v, "VEC2_Y").y


# --- Geometrische Operationen ---------------------------------------

@builtin("VEC2_LENGTH", arity=1)
def _b_length(v):
    vv = _require_vec2(v, "VEC2_LENGTH")
    return _math.hypot(vv.x, vv.y)


@builtin("VEC2_LENGTH_SQ", arity=1)
def _b_length_sq(v):
    """Quadratlaenge -- vermeidet sqrt, schneller fuer Vergleiche."""
    vv = _require_vec2(v, "VEC2_LENGTH_SQ")
    return vv.x * vv.x + vv.y * vv.y


@builtin("VEC2_NORMALIZE", arity=1)
def _b_normalize(v):
    vv = _require_vec2(v, "VEC2_NORMALIZE")
    L = _math.hypot(vv.x, vv.y)
    if L == 0:
        # Nullvektor bleibt Nullvektor -- alternative waere ein Throw,
        # aber der Use-Case "noch keine Bewegungsrichtung" ist haeufig.
        return _Vec2(0.0, 0.0)
    return _Vec2(vv.x / L, vv.y / L)


@builtin("VEC2_DOT", arity=2)
def _b_dot(a, b):
    av = _require_vec2(a, "VEC2_DOT")
    bv = _require_vec2(b, "VEC2_DOT")
    return av.x * bv.x + av.y * bv.y


@builtin("VEC2_CROSS", arity=2)
def _b_cross(a, b):
    """2D-Cross-Produkt: liefert die Z-Komponente des 3D-Cross. Vorzeichen
    zeigt Orientierung an: > 0 wenn b links von a, < 0 wenn rechts."""
    av = _require_vec2(a, "VEC2_CROSS")
    bv = _require_vec2(b, "VEC2_CROSS")
    return av.x * bv.y - av.y * bv.x


@builtin("VEC2_DISTANCE", arity=2)
def _b_distance(a, b):
    av = _require_vec2(a, "VEC2_DISTANCE")
    bv = _require_vec2(b, "VEC2_DISTANCE")
    return _math.hypot(av.x - bv.x, av.y - bv.y)


@builtin("VEC2_LERP", arity=3, types=("any", "any", "num"))
def _b_lerp(a, b, t):
    av = _require_vec2(a, "VEC2_LERP")
    bv = _require_vec2(b, "VEC2_LERP")
    return _Vec2(av.x + (bv.x - av.x) * t, av.y + (bv.y - av.y) * t)


@builtin("VEC2_PERP", arity=1)
def _b_perp(v):
    """90 Grad gegen den Uhrzeigersinn rotiert."""
    vv = _require_vec2(v, "VEC2_PERP")
    return _Vec2(-vv.y, vv.x)


@builtin("VEC2_REFLECT", arity=2)
def _b_reflect(v, n):
    """v reflektiert an Normale n (n muss nicht normalisiert sein)."""
    vv = _require_vec2(v, "VEC2_REFLECT")
    nv = _require_vec2(n, "VEC2_REFLECT")
    nlen2 = nv.x * nv.x + nv.y * nv.y
    if nlen2 == 0:
        return vv
    dot = (vv.x * nv.x + vv.y * nv.y) / nlen2
    return _Vec2(vv.x - 2.0 * dot * nv.x, vv.y - 2.0 * dot * nv.y)


@builtin("VEC2_ANGLE", arity=1)
def _b_angle(v):
    """Winkel des Vektors in Radian (atan2(y, x))."""
    vv = _require_vec2(v, "VEC2_ANGLE")
    return _math.atan2(vv.y, vv.x)


@builtin("VEC2_FROM_ANGLE", arity=2, types=("num", "num"))
def _b_from_angle(angle, length):
    return _Vec2(_math.cos(angle) * length, _math.sin(angle) * length)


# --- Helper ----------------------------------------------------------

def _require_vec2(v, fn):
    if not isinstance(v, _Vec2):
        raise TypeMismatchError(f"{fn}: Erwartet VEC2, erhalten {type(v).__name__.upper()}")
    return v
