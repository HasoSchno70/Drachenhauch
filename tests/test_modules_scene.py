"""Tests fuer das scene-Modul (Stack-basierter Scene-Manager).

Golden-Tests gegen `dhrt` (Stufe B): jeder Test ist ein eigenstaendiges
GB-Programm (frischer dhrt-Prozess = frischer Scene-Stack). Frueher liefen sie
via `call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""
import pytest

from gamebasic.errors import GBRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


# --- Stack-Operationen -----------------------------------------------

def test_empty_stack_current_is_blank(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'PRINT "[" + SCENE_CURRENT() + "]"\n'
                 'PRINT SCENE_DEPTH()\n')
    assert _lines(out) == ["[]", "0"]


def test_push_makes_scene_current(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("menu")\n'
                 'PRINT SCENE_CURRENT()\n'
                 'PRINT SCENE_DEPTH()\n')
    assert _lines(out) == ["menu", "1"]


def test_push_pop_roundtrip(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("menu")\n'
                 'SCENE_PUSH("playing")\n'
                 'PRINT SCENE_CURRENT()\n'
                 'PRINT SCENE_DEPTH()\n'
                 'SCENE_POP()\n'
                 'PRINT SCENE_CURRENT()\n'
                 'PRINT SCENE_DEPTH()\n')
    assert _lines(out) == ["playing", "2", "menu", "1"]


def test_pop_on_empty_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="bereits leer"):
        run_gb('IMPORT "scene"\nSCENE_POP()\n')


def test_switch_replaces_stack(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("a")\n'
                 'SCENE_PUSH("b")\n'
                 'SCENE_PUSH("c")\n'
                 'PRINT SCENE_DEPTH()\n'
                 'SCENE_SWITCH("x")\n'
                 'PRINT SCENE_DEPTH()\n'
                 'PRINT SCENE_CURRENT()\n')
    assert _lines(out) == ["3", "1", "x"]


def test_has_finds_in_middle(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("a")\n'
                 'SCENE_PUSH("b")\n'
                 'SCENE_PUSH("c")\n'
                 'PRINT SCENE_HAS("a")\n'
                 'PRINT SCENE_HAS("b")\n'
                 'PRINT SCENE_HAS("zzz")\n')
    assert _lines(out) == ["TRUE", "TRUE", "FALSE"]


# --- Pro-Scene-Daten -------------------------------------------------

def test_set_get_int(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("s")\n'
                 'SCENE_SET_INT("score", 42)\n'
                 'PRINT SCENE_GET_INT("score")\n')
    assert _lines(out) == ["42"]


def test_set_get_string(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("s")\n'
                 'SCENE_SET_STRING("name", "Anna")\n'
                 'PRINT SCENE_GET_STRING("name")\n')
    assert _lines(out) == ["Anna"]


def test_set_get_float(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("s")\n'
                 'SCENE_SET_FLOAT("t", 1.5)\n'
                 'PRINT SCENE_GET_FLOAT("t")\n')
    assert _lines(out) == ["1.5"]


def test_set_get_bool(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("s")\n'
                 'SCENE_SET_BOOL("paused", TRUE)\n'
                 'PRINT SCENE_GET_BOOL("paused")\n')
    assert _lines(out) == ["TRUE"]


def test_get_missing_key_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="nicht in Scene"):
        run_gb('IMPORT "scene"\nSCENE_PUSH("s")\nPRINT SCENE_GET_INT("fehlt")\n')


def test_get_wrong_type_raises(run_gb):
    # dhrt-Wortlaut: "Key 'x' hat falschen Typ" (TW sagte "ist INT, nicht STRING").
    with pytest.raises(GBRuntimeError, match="falschen Typ"):
        run_gb('IMPORT "scene"\nSCENE_PUSH("s")\n'
               'SCENE_SET_INT("x", 1)\nPRINT SCENE_GET_STRING("x")\n')


def test_get_or_returns_default_when_missing(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("s")\n'
                 'PRINT SCENE_GET_INT_OR("x", 99)\n'
                 'PRINT SCENE_GET_STRING_OR("x", "fallback")\n'
                 'PRINT SCENE_GET_BOOL_OR("x", TRUE)\n'
                 'PRINT SCENE_GET_FLOAT_OR("x", 3.14)\n')
    assert _lines(out) == ["99", "fallback", "TRUE", "3.14"]


def test_get_or_returns_default_when_wrong_type(run_gb):
    """get_or vergleicht den Typ - bei Mismatch liefert er Default."""
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("s")\n'
                 'SCENE_SET_STRING("x", "hello")\n'
                 'PRINT SCENE_GET_INT_OR("x", 42)\n')
    assert _lines(out) == ["42"]


def test_get_or_returns_value_when_present(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("s")\n'
                 'SCENE_SET_INT("x", 7)\n'
                 'PRINT SCENE_GET_INT_OR("x", 99)\n')
    assert _lines(out) == ["7"]


def test_has_key_and_delete(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("s")\n'
                 'SCENE_SET_INT("x", 1)\n'
                 'PRINT SCENE_HAS_KEY("x")\n'
                 'SCENE_DELETE("x")\n'
                 'PRINT SCENE_HAS_KEY("x")\n')
    assert _lines(out) == ["TRUE", "FALSE"]


def test_delete_missing_is_idempotent(run_gb):
    # darf nicht werfen
    out = run_gb('IMPORT "scene"\nSCENE_PUSH("s")\nSCENE_DELETE("x")\nPRINT "ok"\n')
    assert _lines(out) == ["ok"]


def test_set_without_scene_raises(run_gb):
    """Ohne Scene auf dem Stack ist set/get sinnlos und wirft."""
    with pytest.raises(GBRuntimeError, match="Stack ist leer"):
        run_gb('IMPORT "scene"\nSCENE_SET_INT("x", 1)\n')


# --- Daten sind pro Scene -------------------------------------------

def test_data_does_not_leak_between_scenes(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("a")\n'
                 'SCENE_SET_INT("x", 1)\n'
                 'SCENE_PUSH("b")\n'
                 'PRINT SCENE_HAS_KEY("x")\n'
                 'SCENE_SET_INT("x", 99)\n'
                 'PRINT SCENE_GET_INT("x")\n'
                 'SCENE_POP()\n'
                 'PRINT SCENE_GET_INT("x")\n')
    assert _lines(out) == ["FALSE", "99", "1"]


def test_switch_drops_old_data(run_gb):
    out = run_gb('IMPORT "scene"\n'
                 'SCENE_PUSH("a")\n'
                 'SCENE_SET_INT("x", 1)\n'
                 'SCENE_SWITCH("b")\n'
                 'PRINT SCENE_HAS_KEY("x")\n')
    assert _lines(out) == ["FALSE"]
