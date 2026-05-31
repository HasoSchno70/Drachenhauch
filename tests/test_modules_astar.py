"""Tests fuer das astar-Modul."""
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_astar():
    assert load_module("astar")


@pytest.fixture
def grid(call_builtin):
    return call_builtin("astar_new", [10, 10])


# --- Lifecycle / Bounds ---------------------------------------------

def test_new_creates_grid(call_builtin, grid):
    assert call_builtin("astar_width", [grid]) == 10
    assert call_builtin("astar_height", [grid]) == 10


def test_new_zero_dim_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="> 0"):
        call_builtin("astar_new", [0, 5])
    with pytest.raises(GBRuntimeError, match="> 0"):
        call_builtin("astar_new", [5, -1])


def test_set_wall_out_of_bounds(call_builtin, grid):
    with pytest.raises(GBRuntimeError, match="ausserhalb"):
        call_builtin("astar_set_wall", [grid, 99, 99])


def test_clear_removes_walls_and_path(call_builtin, grid):
    call_builtin("astar_set_wall", [grid, 1, 1])
    call_builtin("astar_set_wall", [grid, 2, 2])
    assert call_builtin("astar_is_wall", [grid, 1, 1]) is True
    call_builtin("astar_clear", [grid])
    assert call_builtin("astar_is_wall", [grid, 1, 1]) is False
    assert call_builtin("astar_is_wall", [grid, 2, 2]) is False


# --- Wall-Bitfeld ---------------------------------------------------

def test_set_and_query_wall(call_builtin, grid):
    assert call_builtin("astar_is_wall", [grid, 3, 3]) is False
    call_builtin("astar_set_wall", [grid, 3, 3])
    assert call_builtin("astar_is_wall", [grid, 3, 3]) is True
    call_builtin("astar_set_passable", [grid, 3, 3])
    assert call_builtin("astar_is_wall", [grid, 3, 3]) is False


# --- Pfad-Suche: triviale Faelle ------------------------------------

def test_path_to_self_has_length_one(call_builtin, grid):
    """Start == Ziel: Pfad mit nur einem Element."""
    assert call_builtin("astar_find", [grid, 3, 3, 3, 3]) is True
    assert call_builtin("astar_path_len", [grid]) == 1


def test_straight_line_orthogonal(call_builtin, grid):
    """Kein Wall - direkter horizontaler Pfad."""
    found = call_builtin("astar_find", [grid, 0, 0, 5, 0])
    assert found is True
    assert call_builtin("astar_path_len", [grid]) == 6  # inkl. Start+Ziel
    # Erster und letzter Schritt
    assert (call_builtin("astar_path_x", [grid, 0]),
            call_builtin("astar_path_y", [grid, 0])) == (0, 0)
    assert (call_builtin("astar_path_x", [grid, 5]),
            call_builtin("astar_path_y", [grid, 5])) == (5, 0)


def test_path_around_wall(call_builtin, grid):
    """Wand quer durch - Pfad muss drumherum."""
    # vertikale Wand bei x=2, y=0..4 (nur die Mitte des Wegs sperrt)
    for y in range(0, 5):
        call_builtin("astar_set_wall", [grid, 2, y])
    found = call_builtin("astar_find", [grid, 0, 0, 5, 0])
    assert found is True
    n = call_builtin("astar_path_len", [grid])
    # Mit Manhattan: ohne Wall waere Pfad 6, mit Wall mindestens 12
    # (0,0) -> runter bis y=5 -> rueber zu x=5 -> rauf zu y=0
    assert n >= 11


def test_no_path_when_blocked(call_builtin, grid):
    """Komplett umzaeunter Start: kein Pfad."""
    # Vier Walls um (5,5) herum
    for x, y in [(4, 5), (6, 5), (5, 4), (5, 6),
                 (4, 4), (6, 4), (4, 6), (6, 6)]:
        call_builtin("astar_set_wall", [grid, x, y])
    found = call_builtin("astar_find", [grid, 5, 5, 0, 0])
    assert found is False
    assert call_builtin("astar_path_len", [grid]) == 0
    assert call_builtin("astar_path_cost", [grid]) == 0.0


def test_start_on_wall_no_path(call_builtin, grid):
    """Startpunkt auf einer Wand - kein Pfad."""
    call_builtin("astar_set_wall", [grid, 0, 0])
    found = call_builtin("astar_find", [grid, 0, 0, 5, 5])
    assert found is False


def test_end_on_wall_no_path(call_builtin, grid):
    call_builtin("astar_set_wall", [grid, 5, 5])
    found = call_builtin("astar_find", [grid, 0, 0, 5, 5])
    assert found is False


# --- Diagonal-Bewegung ----------------------------------------------

def test_diagonal_finds_shorter_path(call_builtin, grid):
    """Mit Diagonal sollte (0,0) -> (3,3) genau 4 Schritte sein."""
    call_builtin("astar_set_diagonal", [grid, True])
    call_builtin("astar_find", [grid, 0, 0, 3, 3])
    assert call_builtin("astar_path_len", [grid]) == 4  # inkl. Start
    # Pfadkosten: 3 * sqrt(2) (drei Diagonal-Schritte)
    cost = call_builtin("astar_path_cost", [grid])
    assert abs(cost - 3 * 1.41421356) < 0.01


def test_orthogonal_only_path_length(call_builtin, grid):
    """Ohne Diagonal: (0,0)->(3,3) braucht 6 Schritte (3 rechts + 3 runter + Start)."""
    call_builtin("astar_set_diagonal", [grid, False])
    call_builtin("astar_find", [grid, 0, 0, 3, 3])
    assert call_builtin("astar_path_len", [grid]) == 7  # inkl. Start


def test_anti_cornercutting(call_builtin, grid):
    """Diagonal-Step durch eine Wand-Ecke darf nicht erlaubt sein.

    Layout: Wall bei (1,0) und (0,1). Eine Diagonale (0,0)->(1,1)
    wuerde durch die Ecke schneiden - das verbietet Anti-Cornercutting."""
    call_builtin("astar_set_diagonal", [grid, True])
    call_builtin("astar_set_wall", [grid, 1, 0])
    call_builtin("astar_set_wall", [grid, 0, 1])
    found = call_builtin("astar_find", [grid, 0, 0, 1, 1])
    # (0,0) ist isoliert: keine ortho Nachbarn frei, keine Diagonal-Step erlaubt
    assert found is False


# --- Heuristiken ----------------------------------------------------

def test_set_unknown_heuristic_raises(call_builtin, grid):
    with pytest.raises(GBRuntimeError, match="unbekannte Heuristik"):
        call_builtin("astar_set_heuristic", [grid, "schwurbel"])


def test_heuristic_case_insensitive(call_builtin, grid):
    call_builtin("astar_set_heuristic", [grid, "MANHATTAN"])
    call_builtin("astar_find", [grid, 0, 0, 5, 5])
    assert call_builtin("astar_path_len", [grid]) == 11


def test_chebyshev_with_diagonal(call_builtin, grid):
    """Chebyshev passt zu uniformer Diagonal-Cost (alles 1).
    Mit Diagonal-Cost auf 1 gesetzt, ist (0,0)->(5,5) genau 5 Schritte + Start."""
    call_builtin("astar_set_diagonal", [grid, True])
    call_builtin("astar_set_diagonal_cost", [grid, 1])
    call_builtin("astar_set_heuristic", [grid, "chebyshev"])
    call_builtin("astar_find", [grid, 0, 0, 5, 5])
    assert call_builtin("astar_path_len", [grid]) == 6  # 5 Diagonal-Schritte + Start


# --- Type-Checking --------------------------------------------------

def test_non_grid_raises(call_builtin):
    with pytest.raises(TypeMismatchError, match="erwartet ASTAR_GRID"):
        call_builtin("astar_find", ["nicht ein grid", 0, 0, 1, 1])


def test_diagonal_cost_must_be_positive(call_builtin, grid):
    with pytest.raises(GBRuntimeError, match="> 0"):
        call_builtin("astar_set_diagonal_cost", [grid, 0])
    with pytest.raises(GBRuntimeError, match="> 0"):
        call_builtin("astar_set_diagonal_cost", [grid, -1])


# --- Index-Bounds beim Pfad-Lesen -----------------------------------

def test_path_x_y_out_of_bounds(call_builtin, grid):
    call_builtin("astar_find", [grid, 0, 0, 3, 0])  # path_len = 4
    with pytest.raises(GBRuntimeError, match="ausserhalb"):
        call_builtin("astar_path_x", [grid, 99])


# --- CLEAR_PATH und PATH_COST ---------------------------------------

def test_clear_path_resets(call_builtin, grid):
    call_builtin("astar_find", [grid, 0, 0, 3, 0])
    assert call_builtin("astar_path_len", [grid]) > 0
    call_builtin("astar_clear_path", [grid])
    assert call_builtin("astar_path_len", [grid]) == 0
    assert call_builtin("astar_path_cost", [grid]) == 0.0


def test_path_cost_orthogonal(call_builtin, grid):
    """Pfad ohne Diagonal: cost == n_steps (also path_len - 1)."""
    call_builtin("astar_find", [grid, 0, 0, 5, 0])
    assert call_builtin("astar_path_cost", [grid]) == pytest.approx(5.0)


# --- Integrations-Test: realistische Karte --------------------------

def test_maze_finds_path(call_builtin):
    """8x8-Labyrinth mit zwei Korridoren, A* findet den Weg."""
    g = call_builtin("astar_new", [8, 8])
    # Vertikale Wand zwischen x=3 und x=4, ausser bei y=6
    for y in range(0, 6):
        call_builtin("astar_set_wall", [g, 3, y])
    # (Korridor bei y=6,7 frei)
    found = call_builtin("astar_find", [g, 0, 0, 7, 0])
    assert found is True
    n = call_builtin("astar_path_len", [g])
    # Mindestens 14 Schritte: 6 runter, 4 rueber, 6 rauf = 16, plus Start
    assert n >= 15
