"""Tests fuer das ecs-Modul (Entity-Component-System)."""
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_ecs():
    assert load_module("ecs")


@pytest.fixture
def world(call_builtin):
    return call_builtin("ecs_new_world", [])


# --- World-Lifecycle -----------------------------------------------

def test_new_world_is_empty(call_builtin, world):
    assert call_builtin("ecs_count", [world]) == 0


def test_new_entity_returns_growing_ids(call_builtin, world):
    e1 = call_builtin("ecs_new_entity", [world])
    e2 = call_builtin("ecs_new_entity", [world])
    assert e1 == 1
    assert e2 == 2


def test_count_grows(call_builtin, world):
    call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_new_entity", [world])
    assert call_builtin("ecs_count", [world]) == 3


def test_destroy_returns_true_on_success(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    assert call_builtin("ecs_destroy", [world, e]) is True
    assert call_builtin("ecs_count", [world]) == 0


def test_destroy_returns_false_for_unknown(call_builtin, world):
    assert call_builtin("ecs_destroy", [world, 999]) is False


def test_alive(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    assert call_builtin("ecs_alive", [world, e]) is True
    call_builtin("ecs_destroy", [world, e])
    assert call_builtin("ecs_alive", [world, e]) is False


# --- Components ----------------------------------------------------

def test_add_int_and_get_int(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_int", [world, e, "hp", 100])
    assert call_builtin("ecs_get_int", [world, e, "hp"]) == 100


def test_add_string(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_string", [world, e, "name", "Anna"])
    assert call_builtin("ecs_get_string", [world, e, "name"]) == "Anna"


def test_add_float(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_float", [world, e, "x", 3.14])
    assert abs(call_builtin("ecs_get_float", [world, e, "x"]) - 3.14) < 1e-9


def test_add_bool(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_bool", [world, e, "alive", True])
    assert call_builtin("ecs_get_bool", [world, e, "alive"]) is True


def test_has_returns_false_when_missing(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    assert call_builtin("ecs_has", [world, e, "nope"]) is False


def test_has_returns_true_after_add(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_int", [world, e, "hp", 50])
    assert call_builtin("ecs_has", [world, e, "hp"]) is True


def test_remove_returns_true_on_success(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_int", [world, e, "hp", 50])
    assert call_builtin("ecs_remove", [world, e, "hp"]) is True
    assert call_builtin("ecs_has", [world, e, "hp"]) is False


def test_remove_returns_false_when_missing(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    assert call_builtin("ecs_remove", [world, e, "nope"]) is False


def test_get_missing_raises(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    with pytest.raises(GBRuntimeError, match="fehlt bei Entity"):
        call_builtin("ecs_get_int", [world, e, "nope"])


def test_get_or_int_returns_default(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    val = call_builtin("ecs_get_or_int", [world, e, "missing", 42])
    assert val == 42


def test_get_or_int_returns_existing(call_builtin, world):
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_int", [world, e, "hp", 75])
    val = call_builtin("ecs_get_or_int", [world, e, "hp", 0])
    assert val == 75


def test_destroy_cleans_components(call_builtin, world):
    """Nach Destroy darf das Component nicht mehr in der Storage sein
    (sonst wuerden Queries Geister-Entities zurueckliefern)."""
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_int", [world, e, "hp", 100])
    call_builtin("ecs_destroy", [world, e])
    assert call_builtin("ecs_has", [world, e, "hp"]) is False


# --- Type-Errors ---------------------------------------------------

def test_get_int_type_mismatch(call_builtin, world):
    """ECS_GET_INT auf einem STRING-Component wirft."""
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_string", [world, e, "x", "hi"])
    with pytest.raises(TypeMismatchError, match="nicht INTEGER"):
        call_builtin("ecs_get_int", [world, e, "x"])


def test_world_type_check(call_builtin):
    with pytest.raises(TypeMismatchError, match="ECS_WORLD"):
        call_builtin("ecs_count", ["not a world"])


def test_add_to_dead_entity_raises(call_builtin, world):
    with pytest.raises(GBRuntimeError, match="nicht in World"):
        call_builtin("ecs_add_int", [world, 999, "x", 1])


# --- Query ---------------------------------------------------------

def test_query_returns_matching_entities(call_builtin, world):
    e1 = call_builtin("ecs_new_entity", [world])
    e2 = call_builtin("ecs_new_entity", [world])
    e3 = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_int", [world, e1, "pos", 10])
    call_builtin("ecs_add_int", [world, e2, "pos", 20])
    # e3 hat KEINE pos
    arr = call_builtin("ecs_query", [world, "pos"])
    # _GBArray.values ist die flache Liste
    assert sorted(arr.values) == [e1, e2]


def test_query2_intersection(call_builtin, world):
    e1 = call_builtin("ecs_new_entity", [world])
    e2 = call_builtin("ecs_new_entity", [world])
    e3 = call_builtin("ecs_new_entity", [world])
    # e1: pos+vel; e2: nur pos; e3: nur vel
    call_builtin("ecs_add_int", [world, e1, "pos", 1])
    call_builtin("ecs_add_int", [world, e1, "vel", 2])
    call_builtin("ecs_add_int", [world, e2, "pos", 3])
    call_builtin("ecs_add_int", [world, e3, "vel", 4])
    arr = call_builtin("ecs_query2", [world, "pos", "vel"])
    # arr.values ist array.array fuer ARRAY OF INTEGER -- list() fuer Vergleich.
    assert list(arr.values) == [e1]


def test_query3_intersection(call_builtin, world):
    e1 = call_builtin("ecs_new_entity", [world])
    e2 = call_builtin("ecs_new_entity", [world])
    for name in ("a", "b", "c"):
        call_builtin("ecs_add_int", [world, e1, name, 1])
    call_builtin("ecs_add_int", [world, e2, "a", 1])
    call_builtin("ecs_add_int", [world, e2, "b", 1])
    # e2 hat a+b, aber kein c
    arr = call_builtin("ecs_query3", [world, "a", "b", "c"])
    assert list(arr.values) == [e1]


def test_query_empty_when_no_match(call_builtin, world):
    call_builtin("ecs_new_entity", [world])
    arr = call_builtin("ecs_query", [world, "nonexistent"])
    assert list(arr.values) == []


# --- ADD_OBJ -------------------------------------------------------

def test_add_obj_with_arbitrary_value(call_builtin, world):
    """ECS_ADD_OBJ akzeptiert beliebige Python-Objekte (z.B. fuer
    User-Klasse-Instances)."""
    e = call_builtin("ecs_new_entity", [world])
    custom = {"x": 1, "y": 2}
    call_builtin("ecs_add_obj", [world, e, "data", custom])
    got = call_builtin("ecs_get", [world, e, "data"])
    assert got is custom


# --- End-to-end: Movement-System --------------------------------

def test_movement_system_via_query(run_gb):
    """Klassisches ECS-Pattern: zwei Entities mit pos+vel, ein Frame
    Movement-System angewandt."""
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
    out = run_gb(src)
    lines = [l.strip() for l in out.split("\n") if l.strip()]
    assert lines[0] == "5"     # 0 + 5
    assert lines[1] == "97"    # 100 + (-3)
