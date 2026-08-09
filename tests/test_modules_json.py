"""Tests fuer das json-Modul.

Golden-Tests gegen `dhrt` (Stufe B): IMPORT "json" + JSON_PARSE in ein
JSON_HANDLE, dann abfragen + PRINT. Frueher via `call_builtin` gegen die
Python-Impl (in Phase 8 geloescht).
"""
import pytest

from drachenhauch.errors import DHRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def test_external_type_is_usable(run_gb):
    """`DIM h AS JSON_HANDLE` kompiliert + laeuft -> dhrt kennt den Typ."""
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""a"": 1}")\n'
                 'PRINT JSON_GET_INT(h, "a")\n')
    assert _lines(out) == ["1"]


def test_parse_simple_object(run_gb):
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""a"": 1, ""b"": ""x""}")\n'
                 'PRINT JSON_GET_INT(h, "a")\n'
                 'PRINT JSON_GET_STRING(h, "b")\n')
    assert _lines(out) == ["1", "x"]


def test_parse_nested_path(run_gb):
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""u"": {""name"": ""Anna"", ""age"": 30}}")\n'
                 'PRINT JSON_GET_STRING(h, "u.name")\n'
                 'PRINT JSON_GET_INT(h, "u.age")\n')
    assert _lines(out) == ["Anna", "30"]


def test_parse_array_index_in_path(run_gb):
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""items"": [10, 20, 30]}")\n'
                 'PRINT JSON_GET_INT(h, "items.0")\n'
                 'PRINT JSON_GET_INT(h, "items.2")\n')
    assert _lines(out) == ["10", "30"]


def test_get_int_accepts_integer_float(run_gb):
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""x"": 5.0}")\n'
                 'PRINT JSON_GET_INT(h, "x")\n')
    assert _lines(out) == ["5"]


def test_get_int_rejects_non_integer_float(run_gb):
    with pytest.raises(DHRuntimeError, match="kein Integer"):
        run_gb('IMPORT "json"\n'
               'DIM h AS JSON_HANDLE\n'
               'h = JSON_PARSE("{""x"": 3.14}")\n'
               'PRINT JSON_GET_INT(h, "x")\n')


def test_get_string_rejects_number(run_gb):
    with pytest.raises(DHRuntimeError, match="kein String"):
        run_gb('IMPORT "json"\n'
               'DIM h AS JSON_HANDLE\n'
               'h = JSON_PARSE("{""x"": 1}")\n'
               'PRINT JSON_GET_STRING(h, "x")\n')


def test_has_present_and_missing(run_gb):
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""a"": 1}")\n'
                 'PRINT JSON_HAS(h, "a")\n'
                 'PRINT JSON_HAS(h, "b")\n')
    assert _lines(out) == ["TRUE", "FALSE"]


def test_len_array_object_string(run_gb):
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""a"": [1,2,3], ""b"": {""x"": 1, ""y"": 2}, ""c"": ""hi""}")\n'
                 'PRINT JSON_LEN(h, "a")\n'
                 'PRINT JSON_LEN(h, "b")\n'
                 'PRINT JSON_LEN(h, "c")\n')
    assert _lines(out) == ["3", "2", "2"]


def test_type_classification(run_gb):
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""i"": 1, ""f"": 1.5, ""s"": ""x"", ""b"": true, '
                 '""n"": null, ""a"": [], ""o"": {}}")\n'
                 'PRINT JSON_TYPE(h, "i")\n'
                 'PRINT JSON_TYPE(h, "f")\n'
                 'PRINT JSON_TYPE(h, "s")\n'
                 'PRINT JSON_TYPE(h, "b")\n'
                 'PRINT JSON_TYPE(h, "n")\n'
                 'PRINT JSON_TYPE(h, "a")\n'
                 'PRINT JSON_TYPE(h, "o")\n'
                 'PRINT JSON_TYPE(h, "missing")\n')
    assert _lines(out) == ["number", "number", "string", "boolean",
                           "null", "array", "object", "missing"]


def test_stringify_roundtrip(run_gb):
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""a"": 1, ""b"": [2, 3]}")\n'
                 'PRINT JSON_STRINGIFY(h)\n')
    assert _lines(out) == ['{"a":1,"b":[2,3]}']


def test_pretty_indents(run_gb):
    out = run_gb('IMPORT "json"\n'
                 'DIM h AS JSON_HANDLE\n'
                 'h = JSON_PARSE("{""a"": 1}")\n'
                 'PRINT JSON_PRETTY(h)\n')
    assert "\n" in out
    assert "  " in out


def test_parse_invalid_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="JSON_PARSE"):
        run_gb('IMPORT "json"\n'
               'DIM h AS JSON_HANDLE\n'
               'h = JSON_PARSE("{not valid json")\n')


def test_path_into_non_container_raises(run_gb):
    # dhrt-Wortlaut: "Pfad 'x.y' nicht aufloesbar" (TW sagte "nicht-Container").
    with pytest.raises(DHRuntimeError, match="nicht aufloesbar"):
        run_gb('IMPORT "json"\n'
               'DIM h AS JSON_HANDLE\n'
               'h = JSON_PARSE("{""x"": 1}")\n'
               'PRINT JSON_GET_INT(h, "x.y")\n')


def test_handle_check_in_get(run_gb):
    with pytest.raises(DHRuntimeError, match="JSON-Handle"):
        run_gb('IMPORT "json"\nPRINT JSON_GET_STRING("nicht ein handle", "x")\n')
