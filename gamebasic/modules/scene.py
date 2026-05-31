"""Scene-Modul fuer GameBasic - Stack-basierter Scene-/State-Manager.

Idee: Spiele bestehen meist aus mehreren Bildschirmen (Menue, Spiel,
Pause, Game-Over). Statt globale Flags zu jonglieren, wird der jeweils
aktive Bildschirm als String-Name auf einen Stack gepusht. Pro Scene
gibt es einen eigenen Daten-Bucket fuer kurzzeitigen State (z.B. die
gewaehlte Schwierigkeit im Menue, der Score in Playing).

Built-ins:
    SCENE_PUSH(name$)              - Scene oben auf den Stack
    SCENE_POP()                    - Oberste entfernen, vorherige aktiv
    SCENE_SWITCH(name$)            - Komplett ersetzen (Stack leeren + push)
    SCENE_CURRENT() -> STRING      - Aktiver Name ("" wenn Stack leer)
    SCENE_DEPTH() -> INTEGER       - Anzahl Scenes auf dem Stack
    SCENE_HAS(name$) -> BOOLEAN    - Ist Name irgendwo im Stack?
    SCENE_RESET()                  - Stack komplett leeren (Test/Cleanup)

Pro-Scene-Daten (operieren auf der OBERSTEN Scene):
    SCENE_SET_INT(key$, value)
    SCENE_SET_FLOAT(key$, value)
    SCENE_SET_STRING(key$, value)
    SCENE_SET_BOOL(key$, value)
    SCENE_GET_INT(key$)            - wirft, wenn Key fehlt oder falscher Typ
    SCENE_GET_FLOAT(key$)
    SCENE_GET_STRING(key$)
    SCENE_GET_BOOL(key$)
    SCENE_HAS_KEY(key$) -> BOOLEAN
    SCENE_DELETE(key$)             - Idempotent (kein Fehler bei fehlendem Key)

SCENE_GET_* mit Default:
    SCENE_GET_INT_OR(key$, default)
    SCENE_GET_FLOAT_OR(key$, default)
    SCENE_GET_STRING_OR(key$, default)
    SCENE_GET_BOOL_OR(key$, default)

Lebenszyklus der Daten:
- SCENE_PUSH erzeugt einen frischen, leeren Daten-Bucket fuer die neue
  Scene. Der Bucket der darunter liegenden Scene bleibt unangetastet
  und ist nach SCENE_POP wieder erreichbar.
- SCENE_SWITCH("X") wirft *alle* alten Buckets weg.
- Beim Wechsel zurueck zu einer Scene mit demselben Namen via PUSH/POP
  ist der Bucket also weg - das ist Absicht (POP loescht den oberen).
"""
from __future__ import annotations

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError


# --- Modul-State (pro Prozess) --------------------------------------

# Stack als Liste von (name, data_dict)-Tupeln. Letztes Element = oben.
_stack: list = []


def _top():
    """Liefert (name, data) der obersten Scene. Wirft, wenn Stack leer."""
    if not _stack:
        raise GBRuntimeError(
            "Scene-Stack ist leer - SCENE_PUSH oder SCENE_SWITCH zuerst aufrufen"
        )
    return _stack[-1]


def _check_str(v, fn: str, what: str) -> str:
    if not isinstance(v, str):
        raise TypeMismatchError(f"{fn}: {what} muss STRING sein")
    return v


# --- Stack-Operationen ----------------------------------------------

@builtin("SCENE_PUSH", arity=1, types=("str",))
def _push(name):
    """Legt eine neue Scene auf den Stack. Vorherige bleibt erhalten."""
    _stack.append((name, {}))
    return None


@builtin("SCENE_POP", arity=0)
def _pop():
    """Entfernt die oberste Scene. Wirft, wenn Stack leer."""
    if not _stack:
        raise GBRuntimeError("SCENE_POP: Stack ist bereits leer")
    _stack.pop()
    return None


@builtin("SCENE_SWITCH", arity=1, types=("str",))
def _switch(name):
    """Leert den Stack komplett und legt `name` als einzige Scene neu an.

    Aequivalent zu: alle SCENE_POP() bis leer, dann SCENE_PUSH(name).
    Sinnvoll fuer Menue -> Spiel -> Game-Over -> Menue, wo man keinen
    Rueckweg erlauben will.
    """
    _stack.clear()
    _stack.append((name, {}))
    return None


@builtin("SCENE_CURRENT", arity=0)
def _current():
    """Aktive Scene oder "" bei leerem Stack."""
    if not _stack:
        return ""
    return _stack[-1][0]


@builtin("SCENE_DEPTH", arity=0)
def _depth():
    return len(_stack)


@builtin("SCENE_HAS", arity=1, types=("str",))
def _has(name):
    """True wenn die genannte Scene irgendwo im Stack liegt."""
    for n, _ in _stack:
        if n == name:
            return True
    return False


@builtin("SCENE_RESET", arity=0)
def _reset():
    """Setzt den Modul-State komplett zurueck. Vor allem fuer Tests."""
    _stack.clear()
    return None


# --- Pro-Scene-Daten: Setter -----------------------------------------

@builtin("SCENE_SET_INT", arity=2, types=("str", "int"))
def _set_int(key, value):
    _, data = _top()
    data[key] = ("int", value)
    return None


@builtin("SCENE_SET_FLOAT", arity=2, types=("str", "num"))
def _set_float(key, value):
    _, data = _top()
    data[key] = ("float", float(value))
    return None


@builtin("SCENE_SET_STRING", arity=2, types=("str", "str"))
def _set_string(key, value):
    _, data = _top()
    data[key] = ("string", value)
    return None


@builtin("SCENE_SET_BOOL", arity=2, types=("str", "bool"))
def _set_bool(key, value):
    _, data = _top()
    data[key] = ("bool", value)
    return None


# --- Pro-Scene-Daten: Strikte Getter (werfen bei Fehler) -------------

def _get_typed(key: str, expected: str, fn: str):
    _, data = _top()
    if key not in data:
        raise GBRuntimeError(f"{fn}: Key '{key}' nicht in Scene")
    typ, val = data[key]
    if typ != expected:
        raise TypeMismatchError(
            f"{fn}: Key '{key}' ist {typ.upper()}, nicht {expected.upper()}"
        )
    return val


@builtin("SCENE_GET_INT", arity=1, types=("str",))
def _get_int(key):
    return _get_typed(key, "int", "SCENE_GET_INT")


@builtin("SCENE_GET_FLOAT", arity=1, types=("str",))
def _get_float(key):
    return _get_typed(key, "float", "SCENE_GET_FLOAT")


@builtin("SCENE_GET_STRING", arity=1, types=("str",))
def _get_string(key):
    return _get_typed(key, "string", "SCENE_GET_STRING")


@builtin("SCENE_GET_BOOL", arity=1, types=("str",))
def _get_bool(key):
    return _get_typed(key, "bool", "SCENE_GET_BOOL")


# --- Pro-Scene-Daten: Getter mit Default (werfen nie) ----------------

@builtin("SCENE_GET_INT_OR", arity=2, types=("str", "int"))
def _get_int_or(key, default):
    _, data = _top()
    if key in data and data[key][0] == "int":
        return data[key][1]
    return default


@builtin("SCENE_GET_FLOAT_OR", arity=2, types=("str", "num"))
def _get_float_or(key, default):
    _, data = _top()
    if key in data and data[key][0] == "float":
        return data[key][1]
    return float(default)


@builtin("SCENE_GET_STRING_OR", arity=2, types=("str", "str"))
def _get_string_or(key, default):
    _, data = _top()
    if key in data and data[key][0] == "string":
        return data[key][1]
    return default


@builtin("SCENE_GET_BOOL_OR", arity=2, types=("str", "bool"))
def _get_bool_or(key, default):
    _, data = _top()
    if key in data and data[key][0] == "bool":
        return data[key][1]
    return default


# --- Pro-Scene-Daten: Existenz / Loeschen ----------------------------

@builtin("SCENE_HAS_KEY", arity=1, types=("str",))
def _has_key(key):
    _, data = _top()
    return key in data


@builtin("SCENE_DELETE", arity=1, types=("str",))
def _delete(key):
    _, data = _top()
    data.pop(key, None)
    return None
