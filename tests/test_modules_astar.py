"""Tests fuer das astar-Modul (A*-Pathfinding).

Golden-Tests gegen `dhrt` (Stufe B): jeder Test ist ein eigenstaendiges
GB-Programm (DIM g AS ASTAR_GRID + Operationen + PRINT). Frueher via
`call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""
import pytest

from gamebasic.errors import GBRuntimeError

# Praeludium: 10x10-Grid in 'g' anlegen.
_PRE = 'IMPORT "astar"\nDIM g AS ASTAR_GRID\ng = ASTAR_NEW(10, 10)\n'


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _run(run_gb, body):
    return _lines(run_gb(_PRE + body))


# --- Lifecycle / Bounds ---------------------------------------------

def test_new_creates_grid(run_gb):
    assert _run(run_gb, "PRINT ASTAR_WIDTH(g)\nPRINT ASTAR_HEIGHT(g)\n") == ["10", "10"]


def test_new_zero_dim_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="> 0"):
        run_gb('IMPORT "astar"\nDIM g AS ASTAR_GRID\ng = ASTAR_NEW(0, 5)\n')
    with pytest.raises(GBRuntimeError, match="> 0"):
        run_gb('IMPORT "astar"\nDIM g AS ASTAR_GRID\ng = ASTAR_NEW(5, -1)\n')


def test_set_wall_out_of_bounds(run_gb):
    with pytest.raises(GBRuntimeError, match="ausserhalb"):
        run_gb(_PRE + "ASTAR_SET_WALL(g, 99, 99)\n")


def test_clear_removes_walls_and_path(run_gb):
    out = _run(run_gb,
               "ASTAR_SET_WALL(g, 1, 1)\n"
               "ASTAR_SET_WALL(g, 2, 2)\n"
               "PRINT ASTAR_IS_WALL(g, 1, 1)\n"
               "ASTAR_CLEAR(g)\n"
               "PRINT ASTAR_IS_WALL(g, 1, 1)\n"
               "PRINT ASTAR_IS_WALL(g, 2, 2)\n")
    assert out == ["TRUE", "FALSE", "FALSE"]


# --- Wall-Bitfeld ---------------------------------------------------

def test_set_and_query_wall(run_gb):
    out = _run(run_gb,
               "PRINT ASTAR_IS_WALL(g, 3, 3)\n"
               "ASTAR_SET_WALL(g, 3, 3)\n"
               "PRINT ASTAR_IS_WALL(g, 3, 3)\n"
               "ASTAR_SET_PASSABLE(g, 3, 3)\n"
               "PRINT ASTAR_IS_WALL(g, 3, 3)\n")
    assert out == ["FALSE", "TRUE", "FALSE"]


# --- Pfad-Suche: triviale Faelle ------------------------------------

def test_path_to_self_has_length_one(run_gb):
    out = _run(run_gb, "PRINT ASTAR_FIND(g, 3, 3, 3, 3)\nPRINT ASTAR_PATH_LEN(g)\n")
    assert out == ["TRUE", "1"]


def test_straight_line_orthogonal(run_gb):
    out = _run(run_gb,
               "PRINT ASTAR_FIND(g, 0, 0, 5, 0)\n"
               "PRINT ASTAR_PATH_LEN(g)\n"
               "PRINT ASTAR_PATH_X(g, 0)\nPRINT ASTAR_PATH_Y(g, 0)\n"
               "PRINT ASTAR_PATH_X(g, 5)\nPRINT ASTAR_PATH_Y(g, 5)\n")
    assert out == ["TRUE", "6", "0", "0", "5", "0"]


def test_path_around_wall(run_gb):
    body = "".join(f"ASTAR_SET_WALL(g, 2, {y})\n" for y in range(5))
    out = _run(run_gb, body + "PRINT ASTAR_FIND(g, 0, 0, 5, 0)\nPRINT ASTAR_PATH_LEN(g)\n")
    assert out[0] == "TRUE"
    assert int(out[1]) >= 11


def test_no_path_when_blocked(run_gb):
    walls = [(4, 5), (6, 5), (5, 4), (5, 6), (4, 4), (6, 4), (4, 6), (6, 6)]
    body = "".join(f"ASTAR_SET_WALL(g, {x}, {y})\n" for x, y in walls)
    out = _run(run_gb, body +
               "PRINT ASTAR_FIND(g, 5, 5, 0, 0)\n"
               "PRINT ASTAR_PATH_LEN(g)\nPRINT ASTAR_PATH_COST(g)\n")
    assert out == ["FALSE", "0", "0.0"]


def test_start_on_wall_no_path(run_gb):
    out = _run(run_gb, "ASTAR_SET_WALL(g, 0, 0)\nPRINT ASTAR_FIND(g, 0, 0, 5, 5)\n")
    assert out == ["FALSE"]


def test_end_on_wall_no_path(run_gb):
    out = _run(run_gb, "ASTAR_SET_WALL(g, 5, 5)\nPRINT ASTAR_FIND(g, 0, 0, 5, 5)\n")
    assert out == ["FALSE"]


# --- Diagonal-Bewegung ----------------------------------------------

def test_diagonal_finds_shorter_path(run_gb):
    out = _run(run_gb,
               "ASTAR_SET_DIAGONAL(g, TRUE)\n"
               "ASTAR_FIND(g, 0, 0, 3, 3)\n"
               "PRINT ASTAR_PATH_LEN(g)\nPRINT ASTAR_PATH_COST(g)\n")
    assert out[0] == "4"
    assert abs(float(out[1]) - 3 * 1.41421356) < 0.01


def test_orthogonal_only_path_length(run_gb):
    out = _run(run_gb,
               "ASTAR_SET_DIAGONAL(g, FALSE)\n"
               "ASTAR_FIND(g, 0, 0, 3, 3)\n"
               "PRINT ASTAR_PATH_LEN(g)\n")
    assert out == ["7"]


def test_anti_cornercutting(run_gb):
    out = _run(run_gb,
               "ASTAR_SET_DIAGONAL(g, TRUE)\n"
               "ASTAR_SET_WALL(g, 1, 0)\n"
               "ASTAR_SET_WALL(g, 0, 1)\n"
               "PRINT ASTAR_FIND(g, 0, 0, 1, 1)\n")
    assert out == ["FALSE"]


# --- Heuristiken ----------------------------------------------------

def test_set_unknown_heuristic_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="unbekannte Heuristik"):
        run_gb(_PRE + 'ASTAR_SET_HEURISTIC(g, "schwurbel")\n')


def test_heuristic_case_insensitive(run_gb):
    out = _run(run_gb,
               'ASTAR_SET_HEURISTIC(g, "MANHATTAN")\n'
               "ASTAR_FIND(g, 0, 0, 5, 5)\n"
               "PRINT ASTAR_PATH_LEN(g)\n")
    assert out == ["11"]


def test_chebyshev_with_diagonal(run_gb):
    out = _run(run_gb,
               "ASTAR_SET_DIAGONAL(g, TRUE)\n"
               "ASTAR_SET_DIAGONAL_COST(g, 1)\n"
               'ASTAR_SET_HEURISTIC(g, "chebyshev")\n'
               "ASTAR_FIND(g, 0, 0, 5, 5)\n"
               "PRINT ASTAR_PATH_LEN(g)\n")
    assert out == ["6"]


# --- Type-Checking --------------------------------------------------

def test_non_grid_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="erwartet ASTAR_GRID"):
        run_gb('IMPORT "astar"\nPRINT ASTAR_FIND("nicht ein grid", 0, 0, 1, 1)\n')


def test_diagonal_cost_must_be_positive(run_gb):
    with pytest.raises(GBRuntimeError, match="> 0"):
        run_gb(_PRE + "ASTAR_SET_DIAGONAL_COST(g, 0)\n")
    with pytest.raises(GBRuntimeError, match="> 0"):
        run_gb(_PRE + "ASTAR_SET_DIAGONAL_COST(g, -1)\n")


# --- Index-Bounds beim Pfad-Lesen -----------------------------------

def test_path_x_y_out_of_bounds(run_gb):
    with pytest.raises(GBRuntimeError, match="ausserhalb"):
        run_gb(_PRE + "ASTAR_FIND(g, 0, 0, 3, 0)\nPRINT ASTAR_PATH_X(g, 99)\n")


# --- CLEAR_PATH und PATH_COST ---------------------------------------

def test_clear_path_resets(run_gb):
    out = _run(run_gb,
               "ASTAR_FIND(g, 0, 0, 3, 0)\n"
               "PRINT ASTAR_PATH_LEN(g)\n"
               "ASTAR_CLEAR_PATH(g)\n"
               "PRINT ASTAR_PATH_LEN(g)\nPRINT ASTAR_PATH_COST(g)\n")
    assert out[0] != "0"
    assert out[1:] == ["0", "0.0"]


def test_path_cost_orthogonal(run_gb):
    out = _run(run_gb, "ASTAR_FIND(g, 0, 0, 5, 0)\nPRINT ASTAR_PATH_COST(g)\n")
    assert float(out[0]) == pytest.approx(5.0)


# --- Integrations-Test: realistische Karte --------------------------

def test_maze_finds_path(run_gb):
    body = ('IMPORT "astar"\nDIM g AS ASTAR_GRID\ng = ASTAR_NEW(8, 8)\n'
            + "".join(f"ASTAR_SET_WALL(g, 3, {y})\n" for y in range(6))
            + "PRINT ASTAR_FIND(g, 0, 0, 7, 0)\nPRINT ASTAR_PATH_LEN(g)\n")
    out = _lines(run_gb(body))
    assert out[0] == "TRUE"
    assert int(out[1]) >= 15
