"""Tests fuer das save-Modul (persistente Save-Slots, JSON-Backend).

Golden-Tests gegen `dhrt` (Stufe B): GB-Programm mit IMPORT "save"; Datei-I/O
laeuft im `base=tmp_path`-Verzeichnis (run_gb legt die .dh dort ab, dhrt chdirt
hin -> relative Save-Pfade landen in tmp_path). Frueher via `call_builtin` gegen
die Python-Impl (in Phase 8 geloescht).
"""
import json
import pytest

from drachenhauch.errors import DHRuntimeError

_PRE = 'IMPORT "save"\nDIM s AS SAVE_HANDLE\ns = SAVE_NEW()\n'


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _run(run_gb, tmp_path, body):
    return _lines(run_gb(body, base=tmp_path))


# --- Lifecycle -------------------------------------------------------

def test_new_returns_handle(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _PRE +
               'PRINT SAVE_VERSION(s)\nPRINT "[" + SAVE_KEYS(s) + "]"\n')
    assert out == ["1", "[]"]


def test_load_missing_file_raises(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="nicht gefunden"):
        _run(run_gb, tmp_path,
             'IMPORT "save"\nDIM s AS SAVE_HANDLE\ns = SAVE_LOAD("fehlt.save")\n')


def test_load_or_new_missing_returns_empty(run_gb, tmp_path):
    out = _run(run_gb, tmp_path,
               'IMPORT "save"\nDIM s AS SAVE_HANDLE\ns = SAVE_LOAD_OR_NEW("fehlt.save")\n'
               'PRINT SAVE_VERSION(s)\nPRINT "[" + SAVE_KEYS(s) + "]"\n')
    assert out == ["1", "[]"]


def test_load_or_new_invalid_json_still_raises(run_gb, tmp_path):
    (tmp_path / "kaputt.save").write_text("nicht echtes json", encoding="utf-8")
    with pytest.raises(DHRuntimeError, match="JSON"):
        _run(run_gb, tmp_path,
             'IMPORT "save"\nDIM s AS SAVE_HANDLE\ns = SAVE_LOAD_OR_NEW("kaputt.save")\n')


def test_exists_true_for_real_file(run_gb, tmp_path):
    (tmp_path / "x.save").write_text("{}", encoding="utf-8")
    assert _run(run_gb, tmp_path,
                'IMPORT "save"\nPRINT SAVE_EXISTS("x.save")\n') == ["TRUE"]


def test_exists_false_for_missing(run_gb, tmp_path):
    assert _run(run_gb, tmp_path,
                'IMPORT "save"\nPRINT SAVE_EXISTS("nope.save")\n') == ["FALSE"]


def test_write_then_load_roundtrip(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _PRE +
               'SAVE_SET_INT(s, "score", 42)\nSAVE_SET_STRING(s, "name", "Anna")\n'
               'SAVE_WRITE(s, "rt.save")\n'
               'DIM l AS SAVE_HANDLE\nl = SAVE_LOAD("rt.save")\n'
               'PRINT SAVE_GET_INT(l, "score")\nPRINT SAVE_GET_STRING(l, "name")\n')
    assert out == ["42", "Anna"]


def test_write_produces_readable_json(run_gb, tmp_path):
    _run(run_gb, tmp_path, _PRE +
         'SAVE_SET_INT(s, "x", 1)\nSAVE_WRITE(s, "p.save")\n')
    text = (tmp_path / "p.save").read_text(encoding="utf-8")
    assert json.loads(text) == {"_version": 1, "data": {"x": 1}}
    assert "\n" in text


def test_delete_file(run_gb, tmp_path):
    (tmp_path / "del.save").write_text("{}", encoding="utf-8")
    out = _run(run_gb, tmp_path,
               'IMPORT "save"\nPRINT SAVE_EXISTS("del.save")\n'
               'SAVE_DELETE_FILE("del.save")\nPRINT SAVE_EXISTS("del.save")\n')
    assert out == ["TRUE", "FALSE"]


def test_delete_file_missing_is_idempotent(run_gb, tmp_path):
    out = _run(run_gb, tmp_path,
               'IMPORT "save"\nSAVE_DELETE_FILE("weg.save")\nPRINT "ok"\n')
    assert out == ["ok"]


# --- Setter / Getter -------------------------------------------------

def test_set_get_int(run_gb, tmp_path):
    assert _run(run_gb, tmp_path, _PRE +
                'SAVE_SET_INT(s, "k", 5)\nPRINT SAVE_GET_INT(s, "k")\n') == ["5"]


def test_set_get_float(run_gb, tmp_path):
    assert _run(run_gb, tmp_path, _PRE +
                'SAVE_SET_FLOAT(s, "k", 1.25)\nPRINT SAVE_GET_FLOAT(s, "k")\n') == ["1.25"]


def test_set_get_string(run_gb, tmp_path):
    assert _run(run_gb, tmp_path, _PRE +
                'SAVE_SET_STRING(s, "k", "hi")\nPRINT SAVE_GET_STRING(s, "k")\n') == ["hi"]


def test_set_get_bool(run_gb, tmp_path):
    assert _run(run_gb, tmp_path, _PRE +
                'SAVE_SET_BOOL(s, "k", TRUE)\nPRINT SAVE_GET_BOOL(s, "k")\n') == ["TRUE"]


def test_get_missing_key_raises(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="nicht im Save"):
        _run(run_gb, tmp_path, _PRE + 'PRINT SAVE_GET_INT(s, "fehlt")\n')


def test_get_wrong_type_raises(run_gb, tmp_path):
    # dhrt-Wortlaut: "kein INTEGER" (TW sagte "nicht INTEGER").
    with pytest.raises(DHRuntimeError, match="kein INTEGER"):
        _run(run_gb, tmp_path, _PRE +
             'SAVE_SET_STRING(s, "k", "hi")\nPRINT SAVE_GET_INT(s, "k")\n')


def test_int_from_integer_float_after_load(run_gb, tmp_path):
    """JSON-float 5.0 -> strict GET_INT toleriert ganzzahlige floats."""
    (tmp_path / "x.save").write_text(
        json.dumps({"_version": 1, "data": {"x": 5.0}}), encoding="utf-8")
    out = _run(run_gb, tmp_path,
               'IMPORT "save"\nDIM s AS SAVE_HANDLE\ns = SAVE_LOAD("x.save")\n'
               'PRINT SAVE_GET_INT(s, "x")\n')
    assert out == ["5"]


def test_int_from_non_integer_float_rejects(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="kein INTEGER"):
        _run(run_gb, tmp_path, _PRE +
             'SAVE_SET_FLOAT(s, "k", 5.5)\nPRINT SAVE_GET_INT(s, "k")\n')


# --- Getter mit Default ---------------------------------------------

def test_get_or_returns_default_when_missing(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _PRE +
               'PRINT SAVE_GET_INT_OR(s, "x", 7)\n'
               'PRINT SAVE_GET_STRING_OR(s, "x", "fb")\n'
               'PRINT SAVE_GET_BOOL_OR(s, "x", FALSE)\n'
               'PRINT SAVE_GET_FLOAT_OR(s, "x", 1.5)\n')
    assert out == ["7", "fb", "FALSE", "1.5"]


def test_get_or_returns_default_on_type_mismatch(run_gb, tmp_path):
    assert _run(run_gb, tmp_path, _PRE +
                'SAVE_SET_STRING(s, "k", "hi")\n'
                'PRINT SAVE_GET_INT_OR(s, "k", 99)\n') == ["99"]


def test_get_or_returns_value_when_present(run_gb, tmp_path):
    assert _run(run_gb, tmp_path, _PRE +
                'SAVE_SET_INT(s, "k", 5)\n'
                'PRINT SAVE_GET_INT_OR(s, "k", 99)\n') == ["5"]


# --- Existenz / Loeschen --------------------------------------------

def test_has_and_delete(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _PRE +
               'SAVE_SET_INT(s, "k", 1)\nPRINT SAVE_HAS(s, "k")\n'
               'SAVE_DELETE(s, "k")\nPRINT SAVE_HAS(s, "k")\n')
    assert out == ["TRUE", "FALSE"]


def test_delete_missing_is_idempotent(run_gb, tmp_path):
    assert _run(run_gb, tmp_path, _PRE +
                'SAVE_DELETE(s, "fehlt")\nPRINT "ok"\n') == ["ok"]


def test_clear_removes_all_keys(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _PRE +
               'SAVE_SET_INT(s, "a", 1)\nSAVE_SET_INT(s, "b", 2)\nSAVE_CLEAR(s)\n'
               'PRINT SAVE_HAS(s, "a")\nPRINT SAVE_HAS(s, "b")\n')
    assert out == ["FALSE", "FALSE"]


def test_clear_keeps_version(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _PRE +
               'SAVE_SET_VERSION(s, 7)\nSAVE_SET_INT(s, "a", 1)\nSAVE_CLEAR(s)\n'
               'PRINT SAVE_VERSION(s)\n')
    assert out == ["7"]


def test_keys_lists_sorted(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _PRE +
               'SAVE_SET_INT(s, "z", 1)\nSAVE_SET_INT(s, "a", 1)\nSAVE_SET_INT(s, "m", 1)\n'
               'PRINT SAVE_KEYS(s)\n')
    assert out == ["a, m, z"]


# --- Versionierung ---------------------------------------------------

def test_default_version_is_one(run_gb, tmp_path):
    assert _run(run_gb, tmp_path, _PRE + 'PRINT SAVE_VERSION(s)\n') == ["1"]


def test_set_version_persists_in_file(run_gb, tmp_path):
    out = _run(run_gb, tmp_path, _PRE +
               'SAVE_SET_VERSION(s, 3)\nSAVE_SET_INT(s, "x", 1)\nSAVE_WRITE(s, "v.save")\n'
               'DIM l AS SAVE_HANDLE\nl = SAVE_LOAD("v.save")\nPRINT SAVE_VERSION(l)\n')
    assert out == ["3"]


def test_load_tolerates_missing_version(run_gb, tmp_path):
    (tmp_path / "old.save").write_text(json.dumps({"data": {"x": 1}}), encoding="utf-8")
    out = _run(run_gb, tmp_path,
               'IMPORT "save"\nDIM s AS SAVE_HANDLE\ns = SAVE_LOAD("old.save")\n'
               'PRINT SAVE_VERSION(s)\nPRINT SAVE_GET_INT(s, "x")\n')
    assert out == ["1", "1"]


def test_load_tolerates_missing_data(run_gb, tmp_path):
    (tmp_path / "old.save").write_text(json.dumps({"_version": 5}), encoding="utf-8")
    out = _run(run_gb, tmp_path,
               'IMPORT "save"\nDIM s AS SAVE_HANDLE\ns = SAVE_LOAD("old.save")\n'
               'PRINT SAVE_VERSION(s)\nPRINT "[" + SAVE_KEYS(s) + "]"\n')
    assert out == ["5", "[]"]


def test_load_rejects_non_object(run_gb, tmp_path):
    (tmp_path / "list.save").write_text("[1, 2, 3]", encoding="utf-8")
    with pytest.raises(DHRuntimeError, match="JSON-Objekt"):
        _run(run_gb, tmp_path,
             'IMPORT "save"\nDIM s AS SAVE_HANDLE\ns = SAVE_LOAD("list.save")\n')


# --- Type-Checking ---------------------------------------------------

def test_non_handle_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="erwartet SAVE_HANDLE"):
        run_gb('IMPORT "save"\nPRINT SAVE_GET_INT("nicht ein handle", "k")\n')


def test_set_bool_wrong_type_message(run_gb, tmp_path):
    # Wortlaut-Konsistenz: Standard-Muster "NAME erwartet TYP, erhalten X"
    # (frueher der Ausreisser "SAVE_SET_BOOL: BOOLEAN noetig").
    with pytest.raises(DHRuntimeError, match="SAVE_SET_BOOL erwartet BOOLEAN, erhalten"):
        _run(run_gb, tmp_path, _PRE + 'SAVE_SET_BOOL(s, "k", 1)\n')
