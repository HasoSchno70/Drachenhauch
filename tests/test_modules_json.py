"""Tests fuer das json-Modul."""
import pytest

from gamebasic.modules import load_module, EXTERNAL_TYPES
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_json():
    assert load_module("json")


@pytest.fixture
def parse(call_builtin):
    return lambda s: call_builtin("json_parse", [s])


def test_module_registers_external_type():
    assert "json_handle" in EXTERNAL_TYPES


def test_parse_simple_object(parse, call_builtin):
    h = parse('{"a": 1, "b": "x"}')
    assert call_builtin("json_get_int", [h, "a"]) == 1
    assert call_builtin("json_get_string", [h, "b"]) == "x"


def test_parse_nested_path(parse, call_builtin):
    h = parse('{"u": {"name": "Anna", "age": 30}}')
    assert call_builtin("json_get_string", [h, "u.name"]) == "Anna"
    assert call_builtin("json_get_int", [h, "u.age"]) == 30


def test_parse_array_index_in_path(parse, call_builtin):
    h = parse('{"items": [10, 20, 30]}')
    assert call_builtin("json_get_int", [h, "items.0"]) == 10
    assert call_builtin("json_get_int", [h, "items.2"]) == 30


def test_get_int_accepts_integer_float(parse, call_builtin):
    h = parse('{"x": 5.0}')
    assert call_builtin("json_get_int", [h, "x"]) == 5


def test_get_int_rejects_non_integer_float(parse, call_builtin):
    h = parse('{"x": 3.14}')
    with pytest.raises(TypeMismatchError, match="kein Integer"):
        call_builtin("json_get_int", [h, "x"])


def test_get_string_rejects_number(parse, call_builtin):
    h = parse('{"x": 1}')
    with pytest.raises(TypeMismatchError, match="kein String"):
        call_builtin("json_get_string", [h, "x"])


def test_has_present_and_missing(parse, call_builtin):
    h = parse('{"a": 1}')
    assert call_builtin("json_has", [h, "a"]) is True
    assert call_builtin("json_has", [h, "b"]) is False


def test_len_array_object_string(parse, call_builtin):
    h = parse('{"a": [1,2,3], "b": {"x": 1, "y": 2}, "c": "hi"}')
    assert call_builtin("json_len", [h, "a"]) == 3
    assert call_builtin("json_len", [h, "b"]) == 2
    assert call_builtin("json_len", [h, "c"]) == 2


def test_type_classification(parse, call_builtin):
    h = parse('{"i": 1, "f": 1.5, "s": "x", "b": true, "n": null, "a": [], "o": {}}')
    assert call_builtin("json_type", [h, "i"]) == "number"
    assert call_builtin("json_type", [h, "f"]) == "number"
    assert call_builtin("json_type", [h, "s"]) == "string"
    assert call_builtin("json_type", [h, "b"]) == "boolean"
    assert call_builtin("json_type", [h, "n"]) == "null"
    assert call_builtin("json_type", [h, "a"]) == "array"
    assert call_builtin("json_type", [h, "o"]) == "object"
    assert call_builtin("json_type", [h, "missing"]) == "missing"


def test_stringify_roundtrip(parse, call_builtin):
    h = parse('{"a": 1, "b": [2, 3]}')
    out = call_builtin("json_stringify", [h])
    # Kompakt - Reihenfolge erhalten (Python 3.7+ dict)
    assert out == '{"a":1,"b":[2,3]}'


def test_pretty_indents(parse, call_builtin):
    h = parse('{"a": 1}')
    out = call_builtin("json_pretty", [h])
    assert "\n" in out
    assert "  " in out


def test_parse_invalid_raises(parse):
    with pytest.raises(GBRuntimeError, match="JSON_PARSE"):
        parse('{not valid json')


def test_path_into_non_container_raises(parse, call_builtin):
    h = parse('{"x": 1}')
    with pytest.raises(GBRuntimeError, match="nicht-Container"):
        call_builtin("json_get_int", [h, "x.y"])


def test_handle_check_in_get(call_builtin):
    with pytest.raises(TypeMismatchError, match="JSON-Handle"):
        call_builtin("json_get_string", ["nicht ein handle", "x"])
