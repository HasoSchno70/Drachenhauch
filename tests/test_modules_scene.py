"""Tests fuer das scene-Modul."""
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_scene():
    assert load_module("scene")


@pytest.fixture(autouse=True)
def _reset_stack(call_builtin):
    """Vor jedem Test einen sauberen Stack herstellen. Wir koennen das nicht
    auf module-scope haben, weil Tests den Stack mutieren."""
    call_builtin("scene_reset", [])
    yield
    call_builtin("scene_reset", [])


# --- Stack-Operationen -----------------------------------------------

def test_empty_stack_current_is_blank(call_builtin):
    assert call_builtin("scene_current", []) == ""
    assert call_builtin("scene_depth", []) == 0


def test_push_makes_scene_current(call_builtin):
    call_builtin("scene_push", ["menu"])
    assert call_builtin("scene_current", []) == "menu"
    assert call_builtin("scene_depth", []) == 1


def test_push_pop_roundtrip(call_builtin):
    call_builtin("scene_push", ["menu"])
    call_builtin("scene_push", ["playing"])
    assert call_builtin("scene_current", []) == "playing"
    assert call_builtin("scene_depth", []) == 2
    call_builtin("scene_pop", [])
    assert call_builtin("scene_current", []) == "menu"
    assert call_builtin("scene_depth", []) == 1


def test_pop_on_empty_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="bereits leer"):
        call_builtin("scene_pop", [])


def test_switch_replaces_stack(call_builtin):
    call_builtin("scene_push", ["a"])
    call_builtin("scene_push", ["b"])
    call_builtin("scene_push", ["c"])
    assert call_builtin("scene_depth", []) == 3
    call_builtin("scene_switch", ["x"])
    assert call_builtin("scene_depth", []) == 1
    assert call_builtin("scene_current", []) == "x"


def test_has_finds_in_middle(call_builtin):
    call_builtin("scene_push", ["a"])
    call_builtin("scene_push", ["b"])
    call_builtin("scene_push", ["c"])
    assert call_builtin("scene_has", ["a"]) is True
    assert call_builtin("scene_has", ["b"]) is True
    assert call_builtin("scene_has", ["zzz"]) is False


# --- Pro-Scene-Daten -------------------------------------------------

def test_set_get_int(call_builtin):
    call_builtin("scene_push", ["s"])
    call_builtin("scene_set_int", ["score", 42])
    assert call_builtin("scene_get_int", ["score"]) == 42


def test_set_get_string(call_builtin):
    call_builtin("scene_push", ["s"])
    call_builtin("scene_set_string", ["name", "Anna"])
    assert call_builtin("scene_get_string", ["name"]) == "Anna"


def test_set_get_float(call_builtin):
    call_builtin("scene_push", ["s"])
    call_builtin("scene_set_float", ["t", 1.5])
    assert call_builtin("scene_get_float", ["t"]) == pytest.approx(1.5)


def test_set_get_bool(call_builtin):
    call_builtin("scene_push", ["s"])
    call_builtin("scene_set_bool", ["paused", True])
    assert call_builtin("scene_get_bool", ["paused"]) is True


def test_get_missing_key_raises(call_builtin):
    call_builtin("scene_push", ["s"])
    with pytest.raises(GBRuntimeError, match="nicht in Scene"):
        call_builtin("scene_get_int", ["fehlt"])


def test_get_wrong_type_raises(call_builtin):
    call_builtin("scene_push", ["s"])
    call_builtin("scene_set_int", ["x", 1])
    with pytest.raises(TypeMismatchError, match="ist INT, nicht STRING"):
        call_builtin("scene_get_string", ["x"])


def test_get_or_returns_default_when_missing(call_builtin):
    call_builtin("scene_push", ["s"])
    assert call_builtin("scene_get_int_or", ["x", 99]) == 99
    assert call_builtin("scene_get_string_or", ["x", "fallback"]) == "fallback"
    assert call_builtin("scene_get_bool_or", ["x", True]) is True
    assert call_builtin("scene_get_float_or", ["x", 3.14]) == pytest.approx(3.14)


def test_get_or_returns_default_when_wrong_type(call_builtin):
    """get_or vergleicht den Typ - bei Mismatch liefert er Default,
    nicht einen Cast oder Fehler."""
    call_builtin("scene_push", ["s"])
    call_builtin("scene_set_string", ["x", "hello"])
    assert call_builtin("scene_get_int_or", ["x", 42]) == 42


def test_get_or_returns_value_when_present(call_builtin):
    call_builtin("scene_push", ["s"])
    call_builtin("scene_set_int", ["x", 7])
    assert call_builtin("scene_get_int_or", ["x", 99]) == 7


def test_has_key_and_delete(call_builtin):
    call_builtin("scene_push", ["s"])
    call_builtin("scene_set_int", ["x", 1])
    assert call_builtin("scene_has_key", ["x"]) is True
    call_builtin("scene_delete", ["x"])
    assert call_builtin("scene_has_key", ["x"]) is False


def test_delete_missing_is_idempotent(call_builtin):
    call_builtin("scene_push", ["s"])
    call_builtin("scene_delete", ["x"])  # darf nicht werfen


def test_set_without_scene_raises(call_builtin):
    """Ohne Scene auf dem Stack ist set/get sinnlos und wirft."""
    with pytest.raises(GBRuntimeError, match="Stack ist leer"):
        call_builtin("scene_set_int", ["x", 1])


# --- Daten sind pro Scene -------------------------------------------

def test_data_does_not_leak_between_scenes(call_builtin):
    call_builtin("scene_push", ["a"])
    call_builtin("scene_set_int", ["x", 1])
    call_builtin("scene_push", ["b"])
    # Frischer Bucket - x existiert hier nicht
    assert call_builtin("scene_has_key", ["x"]) is False
    call_builtin("scene_set_int", ["x", 99])
    assert call_builtin("scene_get_int", ["x"]) == 99
    # Pop -> wieder a, x = 1
    call_builtin("scene_pop", [])
    assert call_builtin("scene_get_int", ["x"]) == 1


def test_switch_drops_old_data(call_builtin):
    call_builtin("scene_push", ["a"])
    call_builtin("scene_set_int", ["x", 1])
    call_builtin("scene_switch", ["b"])
    assert call_builtin("scene_has_key", ["x"]) is False
