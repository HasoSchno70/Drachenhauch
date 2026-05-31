"""Save-Modul fuer GameBasic - persistente Daten via JSON-Backend.

High-Level-Wrapper um JSON: typsicheres Set/Get auf einer Save-Datei,
Versionsfeld zur Migration, sichere Defaults beim Laden.

Built-ins:

  Lifecycle:
    SAVE_NEW() -> SAVE_HANDLE                 ' leerer Save
    SAVE_LOAD(path$) -> SAVE_HANDLE           ' wirft wenn Datei fehlt
    SAVE_LOAD_OR_NEW(path$) -> SAVE_HANDLE    ' leerer Save wenn Datei fehlt
    SAVE_EXISTS(path$) -> BOOLEAN
    SAVE_WRITE(s, path$)                      ' nach Datei schreiben
    SAVE_DELETE_FILE(path$)                   ' loescht die Save-Datei

  Versionierung (manuell verwaltet, Default 1):
    SAVE_VERSION(s) -> INTEGER
    SAVE_SET_VERSION(s, n)

  Setter (typsicher - was gespeichert wird, kann beim GET wieder geholt werden):
    SAVE_SET_INT(s, key$, value)
    SAVE_SET_FLOAT(s, key$, value)
    SAVE_SET_STRING(s, key$, value)
    SAVE_SET_BOOL(s, key$, value)

  Strikte Getter (werfen bei fehlendem Key oder Typ-Mismatch):
    SAVE_GET_INT(s, key$) -> INTEGER
    SAVE_GET_FLOAT(s, key$) -> FLOAT
    SAVE_GET_STRING(s, key$) -> STRING
    SAVE_GET_BOOL(s, key$) -> BOOLEAN

  Getter mit Default (werfen nie):
    SAVE_GET_INT_OR(s, key$, default) -> INTEGER
    SAVE_GET_FLOAT_OR(s, key$, default) -> FLOAT
    SAVE_GET_STRING_OR(s, key$, default) -> STRING
    SAVE_GET_BOOL_OR(s, key$, default) -> BOOLEAN

  Existenz / Loeschen:
    SAVE_HAS(s, key$) -> BOOLEAN
    SAVE_DELETE(s, key$)         ' idempotent
    SAVE_CLEAR(s)                ' alle Keys weg, Version bleibt
    SAVE_KEYS(s) -> STRING       ' komma-getrennt

Datei-Format (lesbar):

    {
      "_version": 1,
      "data": {
        "highscore": 4200,
        "name": "Anna"
      }
    }

Beim Laden eines fremden / aelteren Save-Files werden fehlende Felder
toleriert (data leer, version=1). So bleibt das Modul vorwaerts-/
rueckwaerts-tolerant.
"""
from __future__ import annotations

import json as _json
import os

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type


class _SaveHandle:
    """Mutable Save-Container."""
    __slots__ = ("version", "data")

    def __init__(self, version: int = 1, data: dict | None = None):
        self.version = version
        self.data = data if data is not None else {}

    def __repr__(self):
        return f"<Save v{self.version} keys={len(self.data)}>"


register_type("save_handle", _SaveHandle)


def _check_handle(v, fn: str) -> _SaveHandle:
    if not isinstance(v, _SaveHandle):
        raise TypeMismatchError(f"{fn} erwartet SAVE_HANDLE (aus SAVE_NEW/SAVE_LOAD)")
    return v


# --- Lifecycle -------------------------------------------------------

@builtin("SAVE_NEW", arity=0)
def _new():
    return _SaveHandle()


def _do_load(path: str) -> _SaveHandle:
    """Eigentliche Load-Logik. Wird von SAVE_LOAD und SAVE_LOAD_OR_NEW
    aufgerufen - separat, damit der @builtin-Wrapper nicht im Weg ist."""
    try:
        with open(path, "r", encoding="utf-8") as fh:
            raw = _json.load(fh)
    except FileNotFoundError:
        raise GBRuntimeError(f"SAVE_LOAD: Datei '{path}' nicht gefunden")
    except OSError as exc:
        raise GBRuntimeError(f"SAVE_LOAD: {exc}")
    except _json.JSONDecodeError as exc:
        raise GBRuntimeError(
            f"SAVE_LOAD: '{path}' ist kein gueltiges JSON ({exc.msg}, Zeile {exc.lineno})"
        )
    return _from_raw(raw)


@builtin("SAVE_LOAD", arity=1, types=("str",))
def _load(path):
    return _do_load(path)


@builtin("SAVE_LOAD_OR_NEW", arity=1, types=("str",))
def _load_or_new(path):
    """Laedt die Save-Datei oder gibt einen leeren Save zurueck, wenn die
    Datei nicht existiert. Andere Fehler (kaputtes JSON, Berechtigung)
    werden weiterhin geworfen - sonst wuerde man stillschweigend den
    bisherigen Save ueberschreiben."""
    if not os.path.exists(path):
        return _SaveHandle()
    return _do_load(path)


@builtin("SAVE_EXISTS", arity=1, types=("str",))
def _exists(path):
    return os.path.isfile(path)


@builtin("SAVE_WRITE", arity=2, types=("any", "str"))
def _write(s, path):
    s = _check_handle(s, "SAVE_WRITE")
    raw = {"_version": s.version, "data": s.data}
    try:
        with open(path, "w", encoding="utf-8") as fh:
            _json.dump(raw, fh, ensure_ascii=False, indent=2)
    except OSError as exc:
        raise GBRuntimeError(f"SAVE_WRITE: {exc}")
    return None


@builtin("SAVE_DELETE_FILE", arity=1, types=("str",))
def _delete_file(path):
    """Loescht die Datei, wenn sie existiert. Idempotent."""
    try:
        os.remove(path)
    except FileNotFoundError:
        pass
    except OSError as exc:
        raise GBRuntimeError(f"SAVE_DELETE_FILE: {exc}")
    return None


def _from_raw(raw) -> _SaveHandle:
    """Macht aus rohen JSON-Daten ein SaveHandle. Tolerant - fehlende
    Felder werden zu Defaults."""
    if not isinstance(raw, dict):
        raise GBRuntimeError(
            "SAVE_LOAD: Save-Datei muss ein JSON-Objekt sein"
        )
    version = raw.get("_version", 1)
    if not isinstance(version, int) or isinstance(version, bool):
        version = 1
    data = raw.get("data", {})
    if not isinstance(data, dict):
        data = {}
    return _SaveHandle(version=version, data=dict(data))


# --- Version --------------------------------------------------------

@builtin("SAVE_VERSION", arity=1)
def _version(s):
    s = _check_handle(s, "SAVE_VERSION")
    return s.version


@builtin("SAVE_SET_VERSION", arity=2, types=("any", "int"))
def _set_version(s, n):
    s = _check_handle(s, "SAVE_SET_VERSION")
    s.version = n
    return None


# --- Setter ---------------------------------------------------------

@builtin("SAVE_SET_INT", arity=3, types=("any", "str", "int"))
def _set_int(s, key, value):
    s = _check_handle(s, "SAVE_SET_INT")
    s.data[key] = value
    return None


@builtin("SAVE_SET_FLOAT", arity=3, types=("any", "str", "num"))
def _set_float(s, key, value):
    s = _check_handle(s, "SAVE_SET_FLOAT")
    s.data[key] = float(value)
    return None


@builtin("SAVE_SET_STRING", arity=3, types=("any", "str", "str"))
def _set_string(s, key, value):
    s = _check_handle(s, "SAVE_SET_STRING")
    s.data[key] = value
    return None


@builtin("SAVE_SET_BOOL", arity=3, types=("any", "str", "bool"))
def _set_bool(s, key, value):
    s = _check_handle(s, "SAVE_SET_BOOL")
    s.data[key] = value
    return None


# --- Strikte Getter -------------------------------------------------

def _get_typed(s: _SaveHandle, key: str, expected: str, fn: str):
    if key not in s.data:
        raise GBRuntimeError(f"{fn}: Key '{key}' nicht im Save")
    val = s.data[key]
    if expected == "int":
        if isinstance(val, bool) or not isinstance(val, int):
            # JSON kennt keinen Unterschied zwischen 1 und 1.0; akzeptiere
            # ganzzahlige Floats.
            if isinstance(val, float) and val.is_integer():
                return int(val)
            raise TypeMismatchError(
                f"{fn}: Key '{key}' ist {type(val).__name__}, nicht INTEGER"
            )
        return val
    if expected == "float":
        if isinstance(val, bool) or not isinstance(val, (int, float)):
            raise TypeMismatchError(
                f"{fn}: Key '{key}' ist {type(val).__name__}, nicht FLOAT"
            )
        return float(val)
    if expected == "string":
        if not isinstance(val, str):
            raise TypeMismatchError(
                f"{fn}: Key '{key}' ist {type(val).__name__}, nicht STRING"
            )
        return val
    if expected == "bool":
        if not isinstance(val, bool):
            raise TypeMismatchError(
                f"{fn}: Key '{key}' ist {type(val).__name__}, nicht BOOLEAN"
            )
        return val
    raise GBRuntimeError(f"{fn}: interner Fehler, unbekannter Typ '{expected}'")


@builtin("SAVE_GET_INT", arity=2, types=("any", "str"))
def _get_int(s, key):
    s = _check_handle(s, "SAVE_GET_INT")
    return _get_typed(s, key, "int", "SAVE_GET_INT")


@builtin("SAVE_GET_FLOAT", arity=2, types=("any", "str"))
def _get_float(s, key):
    s = _check_handle(s, "SAVE_GET_FLOAT")
    return _get_typed(s, key, "float", "SAVE_GET_FLOAT")


@builtin("SAVE_GET_STRING", arity=2, types=("any", "str"))
def _get_string(s, key):
    s = _check_handle(s, "SAVE_GET_STRING")
    return _get_typed(s, key, "string", "SAVE_GET_STRING")


@builtin("SAVE_GET_BOOL", arity=2, types=("any", "str"))
def _get_bool(s, key):
    s = _check_handle(s, "SAVE_GET_BOOL")
    return _get_typed(s, key, "bool", "SAVE_GET_BOOL")


# --- Getter mit Default ---------------------------------------------

@builtin("SAVE_GET_INT_OR", arity=3, types=("any", "str", "int"))
def _get_int_or(s, key, default):
    s = _check_handle(s, "SAVE_GET_INT_OR")
    val = s.data.get(key)
    if isinstance(val, bool):
        return default
    if isinstance(val, int):
        return val
    if isinstance(val, float) and val.is_integer():
        return int(val)
    return default


@builtin("SAVE_GET_FLOAT_OR", arity=3, types=("any", "str", "num"))
def _get_float_or(s, key, default):
    s = _check_handle(s, "SAVE_GET_FLOAT_OR")
    val = s.data.get(key)
    if isinstance(val, bool):
        return float(default)
    if isinstance(val, (int, float)):
        return float(val)
    return float(default)


@builtin("SAVE_GET_STRING_OR", arity=3, types=("any", "str", "str"))
def _get_string_or(s, key, default):
    s = _check_handle(s, "SAVE_GET_STRING_OR")
    val = s.data.get(key)
    if isinstance(val, str):
        return val
    return default


@builtin("SAVE_GET_BOOL_OR", arity=3, types=("any", "str", "bool"))
def _get_bool_or(s, key, default):
    s = _check_handle(s, "SAVE_GET_BOOL_OR")
    val = s.data.get(key)
    if isinstance(val, bool):
        return val
    return default


# --- Existenz / Loeschen --------------------------------------------

@builtin("SAVE_HAS", arity=2, types=("any", "str"))
def _has(s, key):
    s = _check_handle(s, "SAVE_HAS")
    return key in s.data


@builtin("SAVE_DELETE", arity=2, types=("any", "str"))
def _delete(s, key):
    s = _check_handle(s, "SAVE_DELETE")
    s.data.pop(key, None)
    return None


@builtin("SAVE_CLEAR", arity=1)
def _clear(s):
    s = _check_handle(s, "SAVE_CLEAR")
    s.data.clear()
    return None


@builtin("SAVE_KEYS", arity=1)
def _keys(s):
    """Komma-getrennte Liste aller Keys (lex. sortiert). Vor allem zum
    Debugging - GB hat noch keine Sprach-Iteration ueber Maps."""
    s = _check_handle(s, "SAVE_KEYS")
    return ", ".join(sorted(s.data.keys()))
