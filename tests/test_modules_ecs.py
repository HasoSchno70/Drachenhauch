"""Tests fuer das ecs-Modul (Entity-Component-System).

Golden-Tests gegen `dhrt` (Stufe B): DIM w AS ECS_WORLD + Operationen + PRINT.
Frueher via `call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""
import pytest

from drachenhauch.errors import DHRuntimeError

_PRE = 'IMPORT "ecs"\nDIM w AS ECS_WORLD\nw = ECS_NEW_WORLD()\n'


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _run(run_gb, body):
    return _lines(run_gb(_PRE + body))


# --- World-Lifecycle -----------------------------------------------

def test_new_world_is_empty(run_gb):
    assert _run(run_gb, "PRINT ECS_COUNT(w)\n") == ["0"]


def test_new_entity_returns_growing_ids(run_gb):
    assert _run(run_gb,
                "PRINT ECS_NEW_ENTITY(w)\nPRINT ECS_NEW_ENTITY(w)\n") == ["1", "2"]


def test_count_grows(run_gb):
    out = _run(run_gb,
               "ECS_NEW_ENTITY(w)\nECS_NEW_ENTITY(w)\nECS_NEW_ENTITY(w)\n"
               "PRINT ECS_COUNT(w)\n")
    assert out == ["3"]


def test_destroy_returns_true_on_success(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               "PRINT ECS_DESTROY(w, e)\nPRINT ECS_COUNT(w)\n")
    assert out == ["TRUE", "0"]


def test_destroy_returns_false_for_unknown(run_gb):
    assert _run(run_gb, "PRINT ECS_DESTROY(w, 999)\n") == ["FALSE"]


def test_alive(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               "PRINT ECS_ALIVE(w, e)\nECS_DESTROY(w, e)\nPRINT ECS_ALIVE(w, e)\n")
    assert out == ["TRUE", "FALSE"]


# --- Components ----------------------------------------------------

def test_add_int_and_get_int(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_INT(w, e, "hp", 100)\nPRINT ECS_GET_INT(w, e, "hp")\n')
    assert out == ["100"]


def test_add_string(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_STRING(w, e, "name", "Anna")\nPRINT ECS_GET_STRING(w, e, "name")\n')
    assert out == ["Anna"]


def test_add_float(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_FLOAT(w, e, "x", 3.14)\nPRINT ECS_GET_FLOAT(w, e, "x")\n')
    assert out == ["3.14"]


def test_add_bool(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_BOOL(w, e, "alive", TRUE)\nPRINT ECS_GET_BOOL(w, e, "alive")\n')
    assert out == ["TRUE"]


def test_has_returns_false_when_missing(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'PRINT ECS_HAS(w, e, "nope")\n')
    assert out == ["FALSE"]


def test_has_returns_true_after_add(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_INT(w, e, "hp", 50)\nPRINT ECS_HAS(w, e, "hp")\n')
    assert out == ["TRUE"]


def test_remove_returns_true_on_success(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_INT(w, e, "hp", 50)\n'
               'PRINT ECS_REMOVE(w, e, "hp")\nPRINT ECS_HAS(w, e, "hp")\n')
    assert out == ["TRUE", "FALSE"]


def test_remove_returns_false_when_missing(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'PRINT ECS_REMOVE(w, e, "nope")\n')
    assert out == ["FALSE"]


def test_get_missing_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="fehlt bei Entity"):
        run_gb(_PRE + "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'PRINT ECS_GET_INT(w, e, "nope")\n')


def test_get_or_int_returns_default(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'PRINT ECS_GET_OR_INT(w, e, "missing", 42)\n')
    assert out == ["42"]


def test_get_or_int_returns_existing(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_INT(w, e, "hp", 75)\nPRINT ECS_GET_OR_INT(w, e, "hp", 0)\n')
    assert out == ["75"]


def test_destroy_cleans_components(run_gb):
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_INT(w, e, "hp", 100)\nECS_DESTROY(w, e)\n'
               'PRINT ECS_HAS(w, e, "hp")\n')
    assert out == ["FALSE"]


# --- Type-Errors ---------------------------------------------------

def test_get_int_type_mismatch(run_gb):
    with pytest.raises(DHRuntimeError, match="nicht INTEGER"):
        run_gb(_PRE + "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_STRING(w, e, "x", "hi")\nPRINT ECS_GET_INT(w, e, "x")\n')


def test_world_type_check(run_gb):
    with pytest.raises(DHRuntimeError, match="ECS_WORLD"):
        run_gb('IMPORT "ecs"\nPRINT ECS_COUNT("not a world")\n')


def test_add_to_dead_entity_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="nicht in World"):
        run_gb(_PRE + 'ECS_ADD_INT(w, 999, "x", 1)\n')


# --- Query ---------------------------------------------------------

def _entities_from(out_lines):
    """Erste Zeile = Anzahl, danach je eine Entity-ID. Sortiert zurueck."""
    n = int(out_lines[0])
    return sorted(int(x) for x in out_lines[1:1 + n])


def test_query_returns_matching_entities(run_gb):
    out = _run(run_gb,
               "DIM e1 AS INTEGER\nDIM e2 AS INTEGER\nDIM e3 AS INTEGER\n"
               "e1 = ECS_NEW_ENTITY(w)\ne2 = ECS_NEW_ENTITY(w)\ne3 = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_INT(w, e1, "pos", 10)\nECS_ADD_INT(w, e2, "pos", 20)\n'
               'DIM q AS ARRAY OF INTEGER\nq = ECS_QUERY(w, "pos")\n'
               "PRINT LEN(q)\n"
               "DIM i AS INTEGER\nFOR i = 0 TO LEN(q) - 1\n    PRINT q[i]\nNEXT\n")
    assert _entities_from(out) == [1, 2]


def test_query2_intersection(run_gb):
    out = _run(run_gb,
               "DIM e1 AS INTEGER\nDIM e2 AS INTEGER\nDIM e3 AS INTEGER\n"
               "e1 = ECS_NEW_ENTITY(w)\ne2 = ECS_NEW_ENTITY(w)\ne3 = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_INT(w, e1, "pos", 1)\nECS_ADD_INT(w, e1, "vel", 2)\n'
               'ECS_ADD_INT(w, e2, "pos", 3)\nECS_ADD_INT(w, e3, "vel", 4)\n'
               'DIM q AS ARRAY OF INTEGER\nq = ECS_QUERY2(w, "pos", "vel")\n'
               "PRINT LEN(q)\n"
               "DIM i AS INTEGER\nFOR i = 0 TO LEN(q) - 1\n    PRINT q[i]\nNEXT\n")
    assert _entities_from(out) == [1]


def test_query3_intersection(run_gb):
    out = _run(run_gb,
               "DIM e1 AS INTEGER\nDIM e2 AS INTEGER\n"
               "e1 = ECS_NEW_ENTITY(w)\ne2 = ECS_NEW_ENTITY(w)\n"
               'ECS_ADD_INT(w, e1, "a", 1)\nECS_ADD_INT(w, e1, "b", 1)\nECS_ADD_INT(w, e1, "c", 1)\n'
               'ECS_ADD_INT(w, e2, "a", 1)\nECS_ADD_INT(w, e2, "b", 1)\n'
               'DIM q AS ARRAY OF INTEGER\nq = ECS_QUERY3(w, "a", "b", "c")\n'
               "PRINT LEN(q)\n"
               "DIM i AS INTEGER\nFOR i = 0 TO LEN(q) - 1\n    PRINT q[i]\nNEXT\n")
    assert _entities_from(out) == [1]


def test_query_empty_when_no_match(run_gb):
    out = _run(run_gb,
               "ECS_NEW_ENTITY(w)\n"
               'DIM q AS ARRAY OF INTEGER\nq = ECS_QUERY(w, "nonexistent")\n'
               "PRINT LEN(q)\n")
    assert out == ["0"]


# --- ADD_OBJ -------------------------------------------------------

def test_add_obj_with_arbitrary_value(run_gb):
    """ECS_ADD_OBJ speichert beliebige Werte per Referenz (hier eine MAP);
    eine Mutation am zurueckgeholten Objekt schlaegt auf das Original durch."""
    out = _run(run_gb,
               "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
               'DIM m AS MAP OF INTEGER\nMAPPUT(m, "x", 1)\n'
               'ECS_ADD_OBJ(w, e, "data", m)\n'
               'DIM got AS MAP OF INTEGER\ngot = ECS_GET(w, e, "data")\n'
               'PRINT MAPGET(got, "x")\nMAPPUT(got, "x", 99)\nPRINT MAPGET(m, "x")\n')
    assert out == ["1", "99"]


# --- End-to-end: Movement-System --------------------------------

def test_movement_system_via_query(run_gb):
    src = '''
IMPORT "ecs"
DIM w AS ECS_WORLD
w = ECS_NEW_WORLD()
DIM e1 AS INTEGER
DIM e2 AS INTEGER
e1 = ECS_NEW_ENTITY(w)
e2 = ECS_NEW_ENTITY(w)
ECS_ADD_INT(w, e1, "pos", 0)
ECS_ADD_INT(w, e1, "vel", 5)
ECS_ADD_INT(w, e2, "pos", 100)
ECS_ADD_INT(w, e2, "vel", -3)

DIM moved AS ARRAY OF INTEGER
moved = ECS_QUERY2(w, "pos", "vel")
DIM i AS INTEGER
FOR i = 0 TO LEN(moved) - 1
    DIM ent AS INTEGER
    ent = moved[i]
    ECS_ADD_INT(w, ent, "pos",
                ECS_GET_INT(w, ent, "pos") + ECS_GET_INT(w, ent, "vel"))
NEXT
PRINT ECS_GET_INT(w, e1, "pos")
PRINT ECS_GET_INT(w, e2, "pos")
'''
    assert _lines(run_gb(src)) == ["5", "97"]


def test_add_bool_wrong_type_message(run_gb):
    # Wortlaut-Konsistenz: Standard-Muster "NAME erwartet TYP, erhalten X"
    # (frueher der Ausreisser "ECS_ADD_BOOL erwartet value (BOOLEAN)").
    with pytest.raises(DHRuntimeError, match="ECS_ADD_BOOL erwartet BOOLEAN, erhalten"):
        _run(run_gb,
             "DIM e AS INTEGER\ne = ECS_NEW_ENTITY(w)\n"
             'ECS_ADD_BOOL(w, e, "alive", 1)\n')
