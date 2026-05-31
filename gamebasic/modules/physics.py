"""Physics-Modul fuer GameBasic.

Pure Built-in-Funktionen ohne State - alles ist berechnend, kein World-
Manager. Kollisions-Tests, Vektor-Mathe und Ray-Cast.

Built-ins (alle als IMPORT "physics" zu aktivieren):

  Collision-Tests (returns BOOLEAN):
    PHYSICS_BOX_BOX(x1,y1,w1,h1, x2,y2,w2,h2)
    PHYSICS_CIRCLE_CIRCLE(cx1,cy1,r1, cx2,cy2,r2)
    PHYSICS_BOX_CIRCLE(bx,by,bw,bh, cx,cy,cr)
    PHYSICS_POINT_BOX(px,py, bx,by,bw,bh)
    PHYSICS_POINT_CIRCLE(px,py, cx,cy,cr)

  Distance / Vektor-Mathe (FLOAT):
    PHYSICS_DISTANCE(x1,y1, x2,y2)
    PHYSICS_DISTANCE2(x1,y1, x2,y2)         ' Quadrat-Distanz (schneller)
    PHYSICS_LENGTH(vx, vy)
    PHYSICS_NORM_X(vx, vy)                   ' normalisierter X-Anteil
    PHYSICS_NORM_Y(vx, vy)
    PHYSICS_REFLECT_X(vx, vy, nx, ny)        ' v reflektiert an n
    PHYSICS_REFLECT_Y(vx, vy, nx, ny)

  Ray-Cast (FLOAT, t in [0..1] bei Treffer, sonst -1.0):
    PHYSICS_RAY_BOX(rx,ry, dx,dy, bx,by,bw,bh)
    PHYSICS_RAY_CIRCLE(rx,ry, dx,dy, cx,cy,cr)

Konvention:
- Boxen sind (x, y, w, h) wo (x, y) die linke obere Ecke ist.
- Kreise sind (cx, cy, r) - Center und Radius.
- Strahlen sind (rx, ry, dx, dy) - Origin und Richtungsvektor (NICHT
  normalisiert - die Laenge bestimmt die maximale Strahl-Laenge, t=1
  ist das Ende des Strahls).
- Normal-Vektoren `(nx, ny)` muessen NICHT normalisiert sein - REFLECT
  normalisiert intern.

Multi-Return ist in GB nicht moeglich, deshalb haben Vektor-Operationen
mit X/Y-Output zwei separate Funktionen (NORM_X/NORM_Y, REFLECT_X/
REFLECT_Y). Die Berechnung ist mathematisch beim ersten Call der Hot-
Path - der zweite Call wiederholt die Arbeit. Fuer Performance-kritischen
Code ist das vermeidbar (z.B. einmal NORM_X + dann -NORM_Y/NORM_X-
Verhaeltnis fuer den anderen Anteil), aber fuer normale Spielmechanik
unauffaellig.
"""
from __future__ import annotations

import math as _math

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type


# --- interne Helfer (in Python, nicht GB-Built-in) -------------------

def _box_box(x1, y1, w1, h1, x2, y2, w2, h2) -> bool:
    """AABB-AABB-Overlap-Test."""
    return (x1 < x2 + w2 and x1 + w1 > x2
            and y1 < y2 + h2 and y1 + h1 > y2)


def _box_circle(bx, by, bw, bh, cx, cy, cr) -> bool:
    """Box-Circle: closest point on box, dann Distanz < r."""
    nx = max(bx, min(cx, bx + bw))
    ny = max(by, min(cy, by + bh))
    dx = cx - nx
    dy = cy - ny
    return dx * dx + dy * dy < cr * cr


def _circle_circle(cx1, cy1, r1, cx2, cy2, r2) -> bool:
    dx = cx1 - cx2
    dy = cy1 - cy2
    rsum = r1 + r2
    return dx * dx + dy * dy < rsum * rsum


def _point_box(px, py, bx, by, bw, bh) -> bool:
    return bx <= px < bx + bw and by <= py < by + bh


def _point_circle(px, py, cx, cy, cr) -> bool:
    dx = px - cx
    dy = py - cy
    return dx * dx + dy * dy < cr * cr


# --- Collision-Tests ------------------------------------------------

@builtin("PHYSICS_BOX_BOX", arity=8,
         types=("num", "num", "num", "num", "num", "num", "num", "num"))
def _physics_box_box(x1, y1, w1, h1, x2, y2, w2, h2):
    """True wenn Box1 und Box2 sich ueberlappen."""
    return _box_box(x1, y1, w1, h1, x2, y2, w2, h2)


@builtin("PHYSICS_CIRCLE_CIRCLE", arity=6,
         types=("num", "num", "num", "num", "num", "num"))
def _physics_circle_circle(cx1, cy1, r1, cx2, cy2, r2):
    """True wenn die beiden Kreise sich ueberlappen."""
    return _circle_circle(cx1, cy1, r1, cx2, cy2, r2)


@builtin("PHYSICS_BOX_CIRCLE", arity=7,
         types=("num", "num", "num", "num", "num", "num", "num"))
def _physics_box_circle(bx, by, bw, bh, cx, cy, cr):
    """True wenn Box und Kreis sich ueberlappen."""
    return _box_circle(bx, by, bw, bh, cx, cy, cr)


@builtin("PHYSICS_POINT_BOX", arity=6,
         types=("num", "num", "num", "num", "num", "num"))
def _physics_point_box(px, py, bx, by, bw, bh):
    """True wenn der Punkt in der Box liegt (linke obere Kante einschliesslich,
    rechte/untere ausschliesslich)."""
    return _point_box(px, py, bx, by, bw, bh)


@builtin("PHYSICS_POINT_CIRCLE", arity=5,
         types=("num", "num", "num", "num", "num"))
def _physics_point_circle(px, py, cx, cy, cr):
    """True wenn der Punkt im Kreis liegt (Distanz < r)."""
    return _point_circle(px, py, cx, cy, cr)


# --- Distance / Vektor-Mathe ----------------------------------------

@builtin("PHYSICS_DISTANCE", arity=4,
         types=("num", "num", "num", "num"))
def _physics_distance(x1, y1, x2, y2):
    """Euklidische Distanz zwischen zwei Punkten."""
    dx = x2 - x1
    dy = y2 - y1
    return _math.sqrt(dx * dx + dy * dy)


@builtin("PHYSICS_DISTANCE2", arity=4,
         types=("num", "num", "num", "num"))
def _physics_distance2(x1, y1, x2, y2):
    """Quadrierte Distanz - schneller (kein sqrt). Ideal fuer 'X liegt
    naeher als Y'-Vergleiche, da man beide Seiten quadrieren kann."""
    dx = x2 - x1
    dy = y2 - y1
    return float(dx * dx + dy * dy)


@builtin("PHYSICS_LENGTH", arity=2, types=("num", "num"))
def _physics_length(vx, vy):
    """Laenge des Vektors (vx, vy)."""
    return _math.sqrt(vx * vx + vy * vy)


@builtin("PHYSICS_NORM_X", arity=2, types=("num", "num"))
def _physics_norm_x(vx, vy):
    """X-Anteil des normalisierten Vektors (vx, vy). Bei Null-Vektor: 0."""
    L = _math.sqrt(vx * vx + vy * vy)
    return float(vx / L) if L > 0 else 0.0


@builtin("PHYSICS_NORM_Y", arity=2, types=("num", "num"))
def _physics_norm_y(vx, vy):
    """Y-Anteil des normalisierten Vektors (vx, vy). Bei Null-Vektor: 0."""
    L = _math.sqrt(vx * vx + vy * vy)
    return float(vy / L) if L > 0 else 0.0


@builtin("PHYSICS_REFLECT_X", arity=4,
         types=("num", "num", "num", "num"))
def _physics_reflect_x(vx, vy, nx, ny):
    """X-Komponente des Vektors v reflektiert an Normal n.

    Formel: r = v - 2 * (v . n_hat) * n_hat
    Normal wird intern normalisiert (kein Vorbehandeln noetig).
    """
    nL2 = nx * nx + ny * ny
    if nL2 == 0:
        return float(vx)  # entartete Normal -> kein Reflect
    nL = _math.sqrt(nL2)
    nxh = nx / nL
    nyh = ny / nL
    dot = vx * nxh + vy * nyh
    return float(vx - 2 * dot * nxh)


@builtin("PHYSICS_REFLECT_Y", arity=4,
         types=("num", "num", "num", "num"))
def _physics_reflect_y(vx, vy, nx, ny):
    """Y-Komponente des Vektors v reflektiert an Normal n."""
    nL2 = nx * nx + ny * ny
    if nL2 == 0:
        return float(vy)
    nL = _math.sqrt(nL2)
    nxh = nx / nL
    nyh = ny / nL
    dot = vx * nxh + vy * nyh
    return float(vy - 2 * dot * nyh)


# --- Ray-Cast --------------------------------------------------------

@builtin("PHYSICS_RAY_BOX", arity=8,
         types=("num", "num", "num", "num", "num", "num", "num", "num"))
def _physics_ray_box(rx, ry, dx, dy, bx, by, bw, bh):
    """Strahl-Box-Schnitt (Slab-Test).

    Returns t in [0..1] bei Treffer (Treffer-Punkt = origin + t * dir),
    -1 wenn der Strahl die Box nicht trifft im Bereich [0..1].

    Nuetzlich z.B. fuer Schuesse/Geschosse, die in einem Frame eine
    bestimmte Distanz zuruecklegen: dx,dy = velocity * dt.
    """
    # Slab-Method: pro Achse t-Range bestimmen, dann schneiden.
    if dx == 0:
        if rx < bx or rx >= bx + bw:
            return -1.0
        tx_min = -_math.inf
        tx_max = _math.inf
    else:
        t1 = (bx - rx) / dx
        t2 = (bx + bw - rx) / dx
        tx_min = min(t1, t2)
        tx_max = max(t1, t2)
    if dy == 0:
        if ry < by or ry >= by + bh:
            return -1.0
        ty_min = -_math.inf
        ty_max = _math.inf
    else:
        t1 = (by - ry) / dy
        t2 = (by + bh - ry) / dy
        ty_min = min(t1, t2)
        ty_max = max(t1, t2)
    t_enter = max(tx_min, ty_min)
    t_exit = min(tx_max, ty_max)
    if t_enter > t_exit or t_exit < 0 or t_enter > 1:
        return -1.0
    return float(max(t_enter, 0.0))


@builtin("PHYSICS_RAY_CIRCLE", arity=7,
         types=("num", "num", "num", "num", "num", "num", "num"))
def _physics_ray_circle(rx, ry, dx, dy, cx, cy, cr):
    """Strahl-Kreis-Schnitt.

    Returns t in [0..1] beim ersten Eintritt, -1 wenn kein Treffer.
    """
    # Quadratisch loesen: |origin + t*dir - center|^2 = r^2
    fx = rx - cx
    fy = ry - cy
    a = dx * dx + dy * dy
    if a == 0:
        # Strahl mit Laenge 0 -> Punkt gegen Kreis
        if fx * fx + fy * fy <= cr * cr:
            return 0.0
        return -1.0
    b = 2 * (fx * dx + fy * dy)
    c = fx * fx + fy * fy - cr * cr
    disc = b * b - 4 * a * c
    if disc < 0:
        return -1.0
    sq = _math.sqrt(disc)
    t1 = (-b - sq) / (2 * a)
    t2 = (-b + sq) / (2 * a)
    # Erster Treffer, der in [0..1] liegt
    for t in (t1, t2):
        if 0 <= t <= 1:
            return float(t)
    return -1.0


# --- Broadphase-Kollision (O(n) statt O(n^2)) -----------------------
#
# Fuer viele Kreis-Entities pro Frame. Statt jede gegen jede zu testen,
# wird ein Uniform-Grid gebaut und nur Paare in benachbarten Zellen werden
# exakt getestet. Liefert die kollidierenden Index-Paare ueber das
# Accessor-Pattern (wie astar PATH_*). Natives Rust-Backend, sonst Python-
# Fallback (O(n^2), gleiche Paare in gleicher Reihenfolge).
#
#   DIM b AS PHYSICS_BROAD
#   b = PHYSICS_BROAD_NEW()
#   ' --- pro Frame ---
#   PHYSICS_BROAD_CLEAR(b)
#   FOR i = 0 TO n - 1
#       PHYSICS_BROAD_ADD(b, x[i], y[i], r[i])
#   NEXT
#   DIM np AS INTEGER
#   np = PHYSICS_BROAD_QUERY(b)
#   FOR k = 0 TO np - 1
#       a = PHYSICS_BROAD_PAIR_A(b, k)
#       bb = PHYSICS_BROAD_PAIR_B(b, k)
#       ' Narrow-Phase / Kollision aufloesen ...
#   NEXT

try:
    from ..gb_native import BroadPhase as _NativeBroad  # type: ignore
except Exception:
    _NativeBroad = None


class _BroadPhasePy:
    """Python-Fallback. Gleiche Methoden-API wie das native BroadPhase,
    gleiche Paar-Reihenfolge (pro i aufsteigende j > i)."""
    __slots__ = ("xs", "ys", "rs", "pairs")

    def __init__(self):
        self.xs = []
        self.ys = []
        self.rs = []
        self.pairs = []

    def clear(self):
        self.xs.clear()
        self.ys.clear()
        self.rs.clear()
        self.pairs.clear()

    def add(self, x, y, r):
        self.xs.append(x)
        self.ys.append(y)
        self.rs.append(r)
        return len(self.xs) - 1

    def count(self):
        return len(self.xs)

    def query(self):
        self.pairs = []
        xs, ys, rs = self.xs, self.ys, self.rs
        n = len(xs)
        for i in range(n):
            xi, yi, ri = xs[i], ys[i], rs[i]
            for j in range(i + 1, n):
                ddx = xi - xs[j]
                ddy = yi - ys[j]
                rsum = ri + rs[j]
                if ddx * ddx + ddy * ddy < rsum * rsum:
                    self.pairs.append((i, j))
        return len(self.pairs)

    def pair_count(self):
        return len(self.pairs)

    def pair_a(self, i):
        return self.pairs[i][0]

    def pair_b(self, i):
        return self.pairs[i][1]

    def __repr__(self):
        return f"<BroadPhase {len(self.xs)} entities, {len(self.pairs)} pairs>"


_BroadClass = _NativeBroad if _NativeBroad is not None else _BroadPhasePy

register_type("physics_broad", _BroadClass)


def _check_broad(v, fn: str):
    if not isinstance(v, _BroadClass):
        raise TypeMismatchError(f"{fn} erwartet PHYSICS_BROAD")
    return v


@builtin("PHYSICS_BROAD_NEW", arity=0)
def _broad_new():
    return _BroadClass()


@builtin("PHYSICS_BROAD_CLEAR", arity=1)
def _broad_clear(b):
    b = _check_broad(b, "PHYSICS_BROAD_CLEAR")
    b.clear()
    return None


@builtin("PHYSICS_BROAD_ADD", arity=4, types=("any", "num", "num", "num"))
def _broad_add(b, x, y, r):
    """Fuegt einen Kreis (x, y, r) hinzu. Liefert den Entity-Index (0-basiert)."""
    b = _check_broad(b, "PHYSICS_BROAD_ADD")
    if r < 0:
        raise GBRuntimeError("PHYSICS_BROAD_ADD: Radius muss >= 0 sein")
    return b.add(float(x), float(y), float(r))


@builtin("PHYSICS_BROAD_COUNT", arity=1)
def _broad_count(b):
    b = _check_broad(b, "PHYSICS_BROAD_COUNT")
    return b.count()


@builtin("PHYSICS_BROAD_QUERY", arity=1)
def _broad_query(b):
    """Berechnet alle ueberlappenden Paare und liefert die Anzahl."""
    b = _check_broad(b, "PHYSICS_BROAD_QUERY")
    return b.query()


@builtin("PHYSICS_BROAD_PAIR_COUNT", arity=1)
def _broad_pair_count(b):
    b = _check_broad(b, "PHYSICS_BROAD_PAIR_COUNT")
    return b.pair_count()


@builtin("PHYSICS_BROAD_PAIR_A", arity=2, types=("any", "int"))
def _broad_pair_a(b, i):
    b = _check_broad(b, "PHYSICS_BROAD_PAIR_A")
    if i < 0 or i >= b.pair_count():
        raise GBRuntimeError(
            f"PHYSICS_BROAD_PAIR_A: Index {i} ausserhalb [0..{b.pair_count() - 1}]"
        )
    return b.pair_a(i)


@builtin("PHYSICS_BROAD_PAIR_B", arity=2, types=("any", "int"))
def _broad_pair_b(b, i):
    b = _check_broad(b, "PHYSICS_BROAD_PAIR_B")
    if i < 0 or i >= b.pair_count():
        raise GBRuntimeError(
            f"PHYSICS_BROAD_PAIR_B: Index {i} ausserhalb [0..{b.pair_count() - 1}]"
        )
    return b.pair_b(i)
