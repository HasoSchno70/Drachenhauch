"""A*-Pathfinding-Modul fuer GameBasic.

Klassisches A* auf einem rechtwinkligen Tile-Grid. Optional mit
Diagonal-Bewegung. Drei Heuristiken (manhattan, euclid, chebyshev).

Built-ins:

  Grid-Lifecycle:
    ASTAR_NEW(w, h) -> ASTAR_GRID         ' alles passierbar
    ASTAR_CLEAR(g)                        ' alle Walls entfernen
    ASTAR_WIDTH(g) -> INTEGER
    ASTAR_HEIGHT(g) -> INTEGER

  Hindernisse:
    ASTAR_SET_WALL(g, x, y)               ' markiert Tile als unpassierbar
    ASTAR_SET_PASSABLE(g, x, y)
    ASTAR_IS_WALL(g, x, y) -> BOOLEAN

  Konfiguration:
    ASTAR_SET_DIAGONAL(g, allow)          ' BOOLEAN
    ASTAR_SET_HEURISTIC(g, name$)         ' "manhattan" | "euclid" | "chebyshev"
    ASTAR_SET_DIAGONAL_COST(g, cost)      ' Kosten fuer eine Diagonal-Step (Default 1.41421)
                                          ' Orthogonal-Steps kosten immer 1.

  Pfad-Suche (mutiert das Grid - der Pfad bleibt bis zum naechsten FIND
  oder CLEAR_PATH erhalten):
    ASTAR_FIND(g, sx, sy, ex, ey) -> BOOLEAN  ' TRUE = Pfad gefunden
    ASTAR_PATH_LEN(g) -> INTEGER              ' 0 wenn kein Pfad
    ASTAR_PATH_X(g, idx) -> INTEGER           ' 0..LEN-1
    ASTAR_PATH_Y(g, idx) -> INTEGER
    ASTAR_PATH_COST(g) -> FLOAT               ' Gesamtkosten des Pfads
    ASTAR_CLEAR_PATH(g)                       ' Pfad-Daten ohne Re-Suche entfernen

Konvention:
- Tile (0,0) ist links oben.
- Pfad enthaelt Start UND Ziel (PATH_LEN >= 2 wenn Pfad existiert).
- Tile-Koordinaten sind Integer; Out-of-Bounds wirft.

Backend: Wenn das native Rust-Modul `gamebasic.astar_native` gebaut ist
(siehe rust/astar_native/), uebernimmt dessen `AStarGrid` die Suche --
10-40x schneller bei grossen Karten. Sonst faellt das Modul auf die reine
Python-Implementierung (`_AStarGrid`) zurueck. Beide haben dieselbe Methoden-
API und dasselbe counter-FIFO-Tie-Breaking, liefern also denselben Pfad.
Alle Validierung (Bounds, w/h > 0, Heuristik-Namen, Typcheck) liegt in den
Wrappern unten -- damit ist das Verhalten backend-unabhaengig identisch.

Pfad-Such-Performance (Python): O((w*h) log(w*h)) mit heapq. Fuer 200x200-
Karten ist das Sub-Millisekunde-Bereich; nativ entsprechend schneller.
"""
from __future__ import annotations

import heapq
import math

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type


# --- Native-Backend (Rust, optional) -------------------------------

try:
    from ..gb_native import AStarGrid as _NativeGrid  # type: ignore
except Exception:
    _NativeGrid = None


# --- Heuristiken (Python-Backend) -----------------------------------

def _heuristic_manhattan(x1, y1, x2, y2):
    return abs(x1 - x2) + abs(y1 - y2)


def _heuristic_euclid(x1, y1, x2, y2):
    return math.sqrt((x1 - x2) ** 2 + (y1 - y2) ** 2)


def _heuristic_chebyshev(x1, y1, x2, y2):
    """Chebyshev = max(|dx|, |dy|). Optimal wenn Diagonal-Cost == 1."""
    return max(abs(x1 - x2), abs(y1 - y2))


_HEURISTICS = {
    "manhattan": _heuristic_manhattan,
    "euclid": _heuristic_euclid,
    "chebyshev": _heuristic_chebyshev,
}


# --- A*-Suche (Python-Backend) --------------------------------------

# Orthogonal- und (optional) Diagonal-Nachbarn.
_ORTHO = ((1, 0), (-1, 0), (0, 1), (0, -1))
_DIAG = ((1, 1), (1, -1), (-1, 1), (-1, -1))


def _astar_search(g: "_AStarGrid", sx: int, sy: int, ex: int, ey: int):
    """Liefert (path, cost) - path ist [] bei Misserfolg, sonst inkl. Start+Ziel."""
    if g.walls[g.idx(sx, sy)]:
        return [], 0.0
    if g.walls[g.idx(ex, ey)]:
        return [], 0.0

    # heap-Eintraege: (f, counter, x, y). counter dient als Tiebreaker
    # damit kein direkter Vergleich von (x,y)-Tupeln noetig ist.
    open_heap: list = []
    counter = 0
    heapq.heappush(open_heap, (0.0, counter, sx, sy))
    came_from: dict = {}
    g_score: dict = {(sx, sy): 0.0}
    closed: set = set()
    h = g.heuristic
    diag_cost = g.diagonal_cost

    while open_heap:
        _, _, cx, cy = heapq.heappop(open_heap)
        if (cx, cy) == (ex, ey):
            # Pfad rekonstruieren
            path = [(cx, cy)]
            cur = (cx, cy)
            while cur in came_from:
                cur = came_from[cur]
                path.append(cur)
            path.reverse()
            return path, g_score[(ex, ey)]
        if (cx, cy) in closed:
            continue
        closed.add((cx, cy))

        # Nachbarn
        for dx, dy in _ORTHO:
            nx, ny = cx + dx, cy + dy
            if not g.in_bounds(nx, ny):
                continue
            if g.walls[g.idx(nx, ny)]:
                continue
            tentative = g_score[(cx, cy)] + 1.0
            if tentative < g_score.get((nx, ny), float("inf")):
                came_from[(nx, ny)] = (cx, cy)
                g_score[(nx, ny)] = tentative
                f = tentative + h(nx, ny, ex, ey)
                counter += 1
                heapq.heappush(open_heap, (f, counter, nx, ny))
        if g.diagonal:
            for dx, dy in _DIAG:
                nx, ny = cx + dx, cy + dy
                if not g.in_bounds(nx, ny):
                    continue
                if g.walls[g.idx(nx, ny)]:
                    continue
                # Anti-Cornercutting: Diagonalbewegung nur wenn beide
                # angrenzenden Orthogonal-Zellen frei sind. Sonst koennte
                # eine Einheit durch eine Wand-Ecke "hindurchschluepfen".
                if g.walls[g.idx(cx + dx, cy)] or g.walls[g.idx(cx, cy + dy)]:
                    continue
                tentative = g_score[(cx, cy)] + diag_cost
                if tentative < g_score.get((nx, ny), float("inf")):
                    came_from[(nx, ny)] = (cx, cy)
                    g_score[(nx, ny)] = tentative
                    f = tentative + h(nx, ny, ex, ey)
                    counter += 1
                    heapq.heappush(open_heap, (f, counter, nx, ny))

    return [], 0.0


# --- Python-Grid (Fallback) -----------------------------------------

class _AStarGrid:
    """Ein A*-Grid (Python-Fallback). Enthaelt das Wall-Bitfeld und das
    letzte Such-Ergebnis. Die Methoden-API ist deckungsgleich mit dem
    nativen Rust-`AStarGrid`, damit die Built-in-Wrapper backend-agnostisch
    sind."""
    __slots__ = ("w", "h", "walls", "diagonal", "heuristic_name",
                 "heuristic", "diagonal_cost", "path", "_path_cost")

    def __init__(self, w: int, h: int):
        # w/h-Validierung liegt im Wrapper ASTAR_NEW (backend-uniform).
        self.w = w
        self.h = h
        # Flach gespeichert (row-major). True = Wand.
        self.walls = [False] * (w * h)
        self.diagonal = False
        self.heuristic_name = "manhattan"
        self.heuristic = _heuristic_manhattan
        self.diagonal_cost = math.sqrt(2)   # ~1.414
        # Letztes Such-Ergebnis: list[(x, y)], leer wenn kein Pfad.
        self.path: list = []
        self._path_cost: float = 0.0

    def idx(self, x: int, y: int) -> int:
        return y * self.w + x

    def in_bounds(self, x: int, y: int) -> bool:
        return 0 <= x < self.w and 0 <= y < self.h

    # --- Methoden-API (parallel zu astar_native.AStarGrid) ---
    def set_wall(self, x: int, y: int):
        self.walls[self.idx(x, y)] = True

    def set_passable(self, x: int, y: int):
        self.walls[self.idx(x, y)] = False

    def is_wall(self, x: int, y: int) -> bool:
        return self.walls[self.idx(x, y)]

    def clear(self):
        for i in range(len(self.walls)):
            self.walls[i] = False
        self.path = []
        self._path_cost = 0.0

    def set_diagonal(self, allow: bool):
        self.diagonal = allow

    def set_heuristic(self, key: str):
        # key ist vom Wrapper bereits lowercased + validiert.
        self.heuristic = _HEURISTICS[key]
        self.heuristic_name = key

    def set_diagonal_cost(self, cost: float):
        self.diagonal_cost = float(cost)

    def find(self, sx: int, sy: int, ex: int, ey: int) -> bool:
        path, cost = _astar_search(self, sx, sy, ex, ey)
        self.path = path
        self._path_cost = cost
        return len(path) > 0

    def path_len(self) -> int:
        return len(self.path)

    def path_x(self, idx: int) -> int:
        return self.path[idx][0]

    def path_y(self, idx: int) -> int:
        return self.path[idx][1]

    def path_cost(self) -> float:
        return float(self._path_cost)

    def clear_path(self):
        self.path = []
        self._path_cost = 0.0

    def __repr__(self):
        n_walls = sum(1 for w in self.walls if w)
        return f"<AStar {self.w}x{self.h}, {n_walls} walls, {self.heuristic_name}>"


# Aktives Grid-Backend: nativ wenn gebaut, sonst Python. Der registrierte
# Typ und der Typcheck haengen am aktiven Backend -- DIM x AS ASTAR_GRID und
# das Ergebnis von ASTAR_NEW sind dann dieselbe Klasse.
_GridClass = _NativeGrid if _NativeGrid is not None else _AStarGrid

register_type("astar_grid", _GridClass)


def _check_grid(v, fn: str):
    if not isinstance(v, _GridClass):
        raise TypeMismatchError(f"{fn} erwartet ASTAR_GRID")
    return v


def _check_xy(g, x: int, y: int, fn: str):
    if not g.in_bounds(x, y):
        raise GBRuntimeError(
            f"{fn}: ({x},{y}) ausserhalb Grid 0..{g.w - 1}, 0..{g.h - 1}"
        )


# --- Lifecycle ------------------------------------------------------

@builtin("ASTAR_NEW", arity=2, types=("int", "int"))
def _new(w, h):
    if w <= 0 or h <= 0:
        raise GBRuntimeError(
            f"ASTAR_NEW: Breite und Hoehe muessen > 0 sein (erhalten {w}x{h})"
        )
    return _GridClass(w, h)


@builtin("ASTAR_CLEAR", arity=1)
def _clear(g):
    g = _check_grid(g, "ASTAR_CLEAR")
    g.clear()
    return None


@builtin("ASTAR_WIDTH", arity=1)
def _width(g):
    g = _check_grid(g, "ASTAR_WIDTH")
    return g.w


@builtin("ASTAR_HEIGHT", arity=1)
def _height(g):
    g = _check_grid(g, "ASTAR_HEIGHT")
    return g.h


# --- Hindernisse ----------------------------------------------------

@builtin("ASTAR_SET_WALL", arity=3, types=("any", "int", "int"))
def _set_wall(g, x, y):
    g = _check_grid(g, "ASTAR_SET_WALL")
    _check_xy(g, x, y, "ASTAR_SET_WALL")
    g.set_wall(x, y)
    return None


@builtin("ASTAR_SET_PASSABLE", arity=3, types=("any", "int", "int"))
def _set_passable(g, x, y):
    g = _check_grid(g, "ASTAR_SET_PASSABLE")
    _check_xy(g, x, y, "ASTAR_SET_PASSABLE")
    g.set_passable(x, y)
    return None


@builtin("ASTAR_IS_WALL", arity=3, types=("any", "int", "int"))
def _is_wall(g, x, y):
    g = _check_grid(g, "ASTAR_IS_WALL")
    _check_xy(g, x, y, "ASTAR_IS_WALL")
    return g.is_wall(x, y)


# --- Konfiguration --------------------------------------------------

@builtin("ASTAR_SET_DIAGONAL", arity=2, types=("any", "bool"))
def _set_diagonal(g, allow):
    g = _check_grid(g, "ASTAR_SET_DIAGONAL")
    g.set_diagonal(allow)
    return None


@builtin("ASTAR_SET_HEURISTIC", arity=2, types=("any", "str"))
def _set_heuristic(g, name):
    g = _check_grid(g, "ASTAR_SET_HEURISTIC")
    key = name.lower()
    if key not in _HEURISTICS:
        raise GBRuntimeError(
            f"ASTAR_SET_HEURISTIC: unbekannte Heuristik '{name}' "
            f"(erlaubt: {', '.join(sorted(_HEURISTICS.keys()))})"
        )
    g.set_heuristic(key)
    return None


@builtin("ASTAR_SET_DIAGONAL_COST", arity=2, types=("any", "num"))
def _set_diagonal_cost(g, cost):
    g = _check_grid(g, "ASTAR_SET_DIAGONAL_COST")
    if cost <= 0:
        raise GBRuntimeError(
            f"ASTAR_SET_DIAGONAL_COST: cost muss > 0 sein (erhalten {cost})"
        )
    g.set_diagonal_cost(float(cost))
    return None


# --- Pfad-Suche -----------------------------------------------------

@builtin("ASTAR_FIND", arity=5, types=("any", "int", "int", "int", "int"))
def _find(g, sx, sy, ex, ey):
    g = _check_grid(g, "ASTAR_FIND")
    _check_xy(g, sx, sy, "ASTAR_FIND")
    _check_xy(g, ex, ey, "ASTAR_FIND")
    return g.find(sx, sy, ex, ey)


@builtin("ASTAR_PATH_LEN", arity=1)
def _path_len(g):
    g = _check_grid(g, "ASTAR_PATH_LEN")
    return g.path_len()


@builtin("ASTAR_PATH_X", arity=2, types=("any", "int"))
def _path_x(g, idx):
    g = _check_grid(g, "ASTAR_PATH_X")
    if idx < 0 or idx >= g.path_len():
        raise GBRuntimeError(
            f"ASTAR_PATH_X: Index {idx} ausserhalb [0..{g.path_len() - 1}]"
        )
    return g.path_x(idx)


@builtin("ASTAR_PATH_Y", arity=2, types=("any", "int"))
def _path_y(g, idx):
    g = _check_grid(g, "ASTAR_PATH_Y")
    if idx < 0 or idx >= g.path_len():
        raise GBRuntimeError(
            f"ASTAR_PATH_Y: Index {idx} ausserhalb [0..{g.path_len() - 1}]"
        )
    return g.path_y(idx)


@builtin("ASTAR_PATH_COST", arity=1)
def _path_cost(g):
    g = _check_grid(g, "ASTAR_PATH_COST")
    return g.path_cost()


@builtin("ASTAR_CLEAR_PATH", arity=1)
def _clear_path(g):
    g = _check_grid(g, "ASTAR_CLEAR_PATH")
    g.clear_path()
    return None
