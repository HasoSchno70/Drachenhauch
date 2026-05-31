"""Tests fuer das save-Modul."""
import json
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_save():
    assert load_module("save")


@pytest.fixture
def save_handle(call_builtin):
    return call_builtin("save_new", [])


# --- Lifecycle -------------------------------------------------------

def test_new_returns_handle(save_handle):
    assert save_handle is not None
    assert save_handle.version == 1
    assert save_handle.data == {}


def test_load_missing_file_raises(call_builtin, tmp_path):
    path = str(tmp_path / "fehlt.save")
    with pytest.raises(GBRuntimeError, match="nicht gefunden"):
        call_builtin("save_load", [path])


def test_load_or_new_missing_returns_empty(call_builtin, tmp_path):
    path = str(tmp_path / "fehlt.save")
    s = call_builtin("save_load_or_new", [path])
    assert s.data == {}
    assert s.version == 1


def test_load_or_new_invalid_json_still_raises(call_builtin, tmp_path):
    """Wenn die Datei existiert, aber kaputt ist, lieber werfen als
    leise mit leerem Save weiterzumachen - sonst ueberschreiben wir
    den User-Save beim naechsten WRITE."""
    path = tmp_path / "kaputt.save"
    path.write_text("nicht echtes json", encoding="utf-8")
    with pytest.raises(GBRuntimeError, match="JSON"):
        call_builtin("save_load_or_new", [str(path)])


def test_exists_true_for_real_file(call_builtin, tmp_path):
    path = tmp_path / "x.save"
    path.write_text("{}", encoding="utf-8")
    assert call_builtin("save_exists", [str(path)]) is True


def test_exists_false_for_missing(call_builtin, tmp_path):
    assert call_builtin("save_exists", [str(tmp_path / "nope.save")]) is False


def test_write_then_load_roundtrip(call_builtin, tmp_path):
    path = str(tmp_path / "rt.save")
    s = call_builtin("save_new", [])
    call_builtin("save_set_int", [s, "score", 42])
    call_builtin("save_set_string", [s, "name", "Anna"])
    call_builtin("save_write", [s, path])

    loaded = call_builtin("save_load", [path])
    assert call_builtin("save_get_int", [loaded, "score"]) == 42
    assert call_builtin("save_get_string", [loaded, "name"]) == "Anna"


def test_write_produces_readable_json(call_builtin, tmp_path):
    """Save-Dateien sollen menschen-lesbar sein - sonst ist Debugging
    schmerzhaft. Wir verifizieren dass die Datei valides indentiertes
    JSON mit _version und data enthaelt."""
    path = tmp_path / "p.save"
    s = call_builtin("save_new", [])
    call_builtin("save_set_int", [s, "x", 1])
    call_builtin("save_write", [s, str(path)])

    raw = json.loads(path.read_text(encoding="utf-8"))
    assert raw == {"_version": 1, "data": {"x": 1}}
    # indent=2 -> Datei hat Zeilenumbrueche
    assert "\n" in path.read_text(encoding="utf-8")


def test_delete_file(call_builtin, tmp_path):
    path = tmp_path / "del.save"
    path.write_text("{}", encoding="utf-8")
    assert path.exists()
    call_builtin("save_delete_file", [str(path)])
    assert not path.exists()


def test_delete_file_missing_is_idempotent(call_builtin, tmp_path):
    """Loeschen einer Datei, die schon weg ist, soll nicht werfen."""
    call_builtin("save_delete_file", [str(tmp_path / "weg.save")])


# --- Setter / Getter -------------------------------------------------

def test_set_get_int(call_builtin, save_handle):
    call_builtin("save_set_int", [save_handle, "k", 5])
    assert call_builtin("save_get_int", [save_handle, "k"]) == 5


def test_set_get_float(call_builtin, save_handle):
    call_builtin("save_set_float", [save_handle, "k", 1.25])
    assert call_builtin("save_get_float", [save_handle, "k"]) == 1.25


def test_set_get_string(call_builtin, save_handle):
    call_builtin("save_set_string", [save_handle, "k", "hi"])
    assert call_builtin("save_get_string", [save_handle, "k"]) == "hi"


def test_set_get_bool(call_builtin, save_handle):
    call_builtin("save_set_bool", [save_handle, "k", True])
    assert call_builtin("save_get_bool", [save_handle, "k"]) is True


def test_get_missing_key_raises(call_builtin, save_handle):
    with pytest.raises(GBRuntimeError, match="nicht im Save"):
        call_builtin("save_get_int", [save_handle, "fehlt"])


def test_get_wrong_type_raises(call_builtin, save_handle):
    call_builtin("save_set_string", [save_handle, "k", "hi"])
    with pytest.raises(TypeMismatchError, match="nicht INTEGER"):
        call_builtin("save_get_int", [save_handle, "k"])


def test_int_from_integer_float_after_load(call_builtin, tmp_path):
    """JSON unterscheidet 1 und 1.0 nicht zuverlaessig - ein als INT
    gespeicherter Wert kann beim Laden als float zurueckkommen. Der
    strict GET soll das tolerieren wenn der float ganzzahlig ist."""
    path = tmp_path / "x.save"
    # Datei manuell bauen mit float, der ganzzahlig ist
    path.write_text(
        json.dumps({"_version": 1, "data": {"x": 5.0}}),
        encoding="utf-8",
    )
    s = call_builtin("save_load", [str(path)])
    assert call_builtin("save_get_int", [s, "x"]) == 5


def test_int_from_non_integer_float_rejects(call_builtin, save_handle):
    """5.5 ist kein INTEGER - das soll werfen, nicht abrunden."""
    save_handle.data["k"] = 5.5
    with pytest.raises(TypeMismatchError, match="nicht INTEGER"):
        call_builtin("save_get_int", [save_handle, "k"])


# --- Getter mit Default ---------------------------------------------

def test_get_or_returns_default_when_missing(call_builtin, save_handle):
    assert call_builtin("save_get_int_or", [save_handle, "x", 7]) == 7
    assert call_builtin("save_get_string_or", [save_handle, "x", "fb"]) == "fb"
    assert call_builtin("save_get_bool_or", [save_handle, "x", False]) is False
    assert call_builtin("save_get_float_or", [save_handle, "x", 1.5]) == 1.5


def test_get_or_returns_default_on_type_mismatch(call_builtin, save_handle):
    call_builtin("save_set_string", [save_handle, "k", "hi"])
    assert call_builtin("save_get_int_or", [save_handle, "k", 99]) == 99


def test_get_or_returns_value_when_present(call_builtin, save_handle):
    call_builtin("save_set_int", [save_handle, "k", 5])
    assert call_builtin("save_get_int_or", [save_handle, "k", 99]) == 5


# --- Existenz / Loeschen --------------------------------------------

def test_has_and_delete(call_builtin, save_handle):
    call_builtin("save_set_int", [save_handle, "k", 1])
    assert call_builtin("save_has", [save_handle, "k"]) is True
    call_builtin("save_delete", [save_handle, "k"])
    assert call_builtin("save_has", [save_handle, "k"]) is False


def test_delete_missing_is_idempotent(call_builtin, save_handle):
    call_builtin("save_delete", [save_handle, "fehlt"])  # kein Fehler


def test_clear_removes_all_keys(call_builtin, save_handle):
    call_builtin("save_set_int", [save_handle, "a", 1])
    call_builtin("save_set_int", [save_handle, "b", 2])
    call_builtin("save_clear", [save_handle])
    assert call_builtin("save_has", [save_handle, "a"]) is False
    assert call_builtin("save_has", [save_handle, "b"]) is False


def test_clear_keeps_version(call_builtin, save_handle):
    call_builtin("save_set_version", [save_handle, 7])
    call_builtin("save_set_int", [save_handle, "a", 1])
    call_builtin("save_clear", [save_handle])
    assert call_builtin("save_version", [save_handle]) == 7


def test_keys_lists_sorted(call_builtin, save_handle):
    call_builtin("save_set_int", [save_handle, "z", 1])
    call_builtin("save_set_int", [save_handle, "a", 1])
    call_builtin("save_set_int", [save_handle, "m", 1])
    assert call_builtin("save_keys", [save_handle]) == "a, m, z"


# --- Versionierung ---------------------------------------------------

def test_default_version_is_one(call_builtin, save_handle):
    assert call_builtin("save_version", [save_handle]) == 1


def test_set_version_persists_in_file(call_builtin, tmp_path):
    path = tmp_path / "v.save"
    s = call_builtin("save_new", [])
    call_builtin("save_set_version", [s, 3])
    call_builtin("save_set_int", [s, "x", 1])
    call_builtin("save_write", [s, str(path)])

    loaded = call_builtin("save_load", [str(path)])
    assert call_builtin("save_version", [loaded]) == 3


def test_load_tolerates_missing_version(call_builtin, tmp_path):
    """Old-style JSON ohne _version-Feld -> Default 1."""
    path = tmp_path / "old.save"
    path.write_text(
        json.dumps({"data": {"x": 1}}),
        encoding="utf-8",
    )
    s = call_builtin("save_load", [str(path)])
    assert call_builtin("save_version", [s]) == 1
    assert call_builtin("save_get_int", [s, "x"]) == 1


def test_load_tolerates_missing_data(call_builtin, tmp_path):
    """JSON ohne data-Feld -> leerer Save."""
    path = tmp_path / "old.save"
    path.write_text(
        json.dumps({"_version": 5}),
        encoding="utf-8",
    )
    s = call_builtin("save_load", [str(path)])
    assert call_builtin("save_version", [s]) == 5
    assert call_builtin("save_keys", [s]) == ""


def test_load_rejects_non_object(call_builtin, tmp_path):
    """Top-level muss ein Objekt sein, kein Array oder Null."""
    path = tmp_path / "list.save"
    path.write_text("[1, 2, 3]", encoding="utf-8")
    with pytest.raises(GBRuntimeError, match="JSON-Objekt"):
        call_builtin("save_load", [str(path)])


# --- Type-Checking ---------------------------------------------------

def test_non_handle_raises(call_builtin):
    with pytest.raises(TypeMismatchError, match="erwartet SAVE_HANDLE"):
        call_builtin("save_get_int", ["nicht ein handle", "k"])
