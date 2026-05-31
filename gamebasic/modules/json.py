"""JSON-Modul fuer GameBasic.

Built-ins:
    JSON_PARSE(s$)            -> JSON-Handle
    JSON_LOAD(path$)          -> JSON-Handle (aus Datei)
    JSON_STRINGIFY(handle)    -> STRING (kompakt)
    JSON_PRETTY(handle)       -> STRING (eingerueckt)
    JSON_GET_STRING(h, path$) -> STRING
    JSON_GET_INT(h, path$)    -> INTEGER
    JSON_GET_FLOAT(h, path$)  -> FLOAT
    JSON_GET_BOOL(h, path$)   -> BOOLEAN
    JSON_HAS(h, path$)        -> BOOLEAN
    JSON_LEN(h, path$)        -> INTEGER (laenge eines Objekts oder Arrays)
    JSON_TYPE(h, path$)       -> STRING ("object","array","string",
                                          "number","boolean","null")

Pfad-Syntax: Punktnotation. `"user.name"` greift auf das Feld `name` im
Sub-Objekt `user` zu. `"items.0.title"` adressiert Index 0 eines Arrays.
Leerer Pfad `""` zeigt auf das Wurzel-Element.
"""
from __future__ import annotations

import json as _json

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type


class _JSONHandle:
    """Opaque Hülle um ein geparstes JSON-Wertobjekt."""
    __slots__ = ("data",)

    def __init__(self, data):
        self.data = data

    def __repr__(self):
        return f"<JSON {_short_repr(self.data)}>"


register_type("json_handle", _JSONHandle)


def _short_repr(v):
    s = _json.dumps(v, ensure_ascii=False)
    return s if len(s) <= 40 else s[:37] + "..."


def _check_handle(v, fn: str) -> _JSONHandle:
    if not isinstance(v, _JSONHandle):
        raise TypeMismatchError(f"{fn} erwartet JSON-Handle (aus JSON_PARSE)")
    return v


def _resolve_path(data, path: str, fn: str):
    """Folge dem Pfad und gib das adressierte Element zurueck."""
    if path == "":
        return data
    cur = data
    for part in path.split("."):
        if isinstance(cur, list):
            try:
                idx = int(part)
            except ValueError:
                raise GBRuntimeError(
                    f"{fn}: Pfadelement '{part}' ist kein Array-Index"
                )
            if idx < 0 or idx >= len(cur):
                raise GBRuntimeError(
                    f"{fn}: Index {idx} ausserhalb [0..{len(cur) - 1}]"
                )
            cur = cur[idx]
        elif isinstance(cur, dict):
            if part not in cur:
                raise GBRuntimeError(
                    f"{fn}: Schluessel '{part}' nicht im Objekt"
                )
            cur = cur[part]
        else:
            raise GBRuntimeError(
                f"{fn}: Pfad '{path}' fuehrt durch nicht-Container"
            )
    return cur


def _try_resolve(data, path: str):
    """Wie _resolve_path, gibt (found, value) zurueck statt zu werfen."""
    if path == "":
        return True, data
    cur = data
    for part in path.split("."):
        if isinstance(cur, list):
            try:
                idx = int(part)
            except ValueError:
                return False, None
            if idx < 0 or idx >= len(cur):
                return False, None
            cur = cur[idx]
        elif isinstance(cur, dict):
            if part not in cur:
                return False, None
            cur = cur[part]
        else:
            return False, None
    return True, cur


@builtin("JSON_PARSE", arity=1, types=("str",))
def _parse(s):
    try:
        return _JSONHandle(_json.loads(s))
    except _json.JSONDecodeError as exc:
        raise GBRuntimeError(f"JSON_PARSE: {exc.msg} (Zeile {exc.lineno}, Spalte {exc.colno})")


@builtin("JSON_LOAD", arity=1, types=("str",))
def _load(path):
    try:
        with open(path, "r", encoding="utf-8") as fh:
            return _JSONHandle(_json.load(fh))
    except OSError as exc:
        raise GBRuntimeError(f"JSON_LOAD: {exc}")
    except _json.JSONDecodeError as exc:
        raise GBRuntimeError(
            f"JSON_LOAD: {path}: {exc.msg} (Zeile {exc.lineno})"
        )


@builtin("JSON_STRINGIFY", arity=1)
def _stringify(h):
    h = _check_handle(h, "JSON_STRINGIFY")
    return _json.dumps(h.data, ensure_ascii=False, separators=(",", ":"))


@builtin("JSON_PRETTY", arity=1)
def _pretty(h):
    h = _check_handle(h, "JSON_PRETTY")
    return _json.dumps(h.data, ensure_ascii=False, indent=2)


@builtin("JSON_GET_STRING", arity=2, types=("any", "str"))
def _get_string(h, path):
    h = _check_handle(h, "JSON_GET_STRING")
    v = _resolve_path(h.data, path, "JSON_GET_STRING")
    if not isinstance(v, str):
        raise TypeMismatchError(
            f"JSON_GET_STRING: Pfad '{path}' ist kein String "
            f"(ist {type(v).__name__})"
        )
    return v


@builtin("JSON_GET_INT", arity=2, types=("any", "str"))
def _get_int(h, path):
    h = _check_handle(h, "JSON_GET_INT")
    v = _resolve_path(h.data, path, "JSON_GET_INT")
    if isinstance(v, bool) or not isinstance(v, int):
        if isinstance(v, float) and v.is_integer():
            return int(v)
        raise TypeMismatchError(
            f"JSON_GET_INT: Pfad '{path}' ist kein Integer "
            f"(ist {type(v).__name__})"
        )
    return v


@builtin("JSON_GET_FLOAT", arity=2, types=("any", "str"))
def _get_float(h, path):
    h = _check_handle(h, "JSON_GET_FLOAT")
    v = _resolve_path(h.data, path, "JSON_GET_FLOAT")
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(
            f"JSON_GET_FLOAT: Pfad '{path}' ist keine Zahl "
            f"(ist {type(v).__name__})"
        )
    return float(v)


@builtin("JSON_GET_BOOL", arity=2, types=("any", "str"))
def _get_bool(h, path):
    h = _check_handle(h, "JSON_GET_BOOL")
    v = _resolve_path(h.data, path, "JSON_GET_BOOL")
    if not isinstance(v, bool):
        raise TypeMismatchError(
            f"JSON_GET_BOOL: Pfad '{path}' ist kein Boolean "
            f"(ist {type(v).__name__})"
        )
    return v


@builtin("JSON_HAS", arity=2, types=("any", "str"))
def _has(h, path):
    h = _check_handle(h, "JSON_HAS")
    found, _ = _try_resolve(h.data, path)
    return found


@builtin("JSON_LEN", arity=2, types=("any", "str"))
def _len(h, path):
    h = _check_handle(h, "JSON_LEN")
    v = _resolve_path(h.data, path, "JSON_LEN")
    if isinstance(v, (list, dict, str)):
        return len(v)
    raise TypeMismatchError(
        f"JSON_LEN: Pfad '{path}' adressiert weder Array, Objekt noch String "
        f"(ist {type(v).__name__})"
    )


@builtin("JSON_TYPE", arity=2, types=("any", "str"))
def _type(h, path):
    h = _check_handle(h, "JSON_TYPE")
    found, v = _try_resolve(h.data, path)
    if not found:
        return "missing"
    if v is None:
        return "null"
    if isinstance(v, bool):
        return "boolean"
    if isinstance(v, (int, float)):
        return "number"
    if isinstance(v, str):
        return "string"
    if isinstance(v, list):
        return "array"
    if isinstance(v, dict):
        return "object"
    return "unknown"
