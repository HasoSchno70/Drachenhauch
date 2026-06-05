"""Entity-Component-System fuer GameBasic.

Pragmatisches ECS, optimiert fuer Game-Dev-typische Use-Cases:
- Eine "World" pro Spiel/Scene.
- Entities sind INTEGER-IDs (kein Wrapper, einfach zu halten).
- Components sind benannte Key-Value-Werte (kein typed-Struct-System --
  fuer komplexe Components nutzt man User-Klassen + ECS_ADD_OBJ).

API:
    ECS_NEW_WORLD()                       -> ECS_WORLD
    ECS_NEW_ENTITY(world)                 -> INTEGER (entity-id)
    ECS_DESTROY(world, ent)               -> BOOLEAN  (TRUE wenn entfernt)
    ECS_ALIVE(world, ent)                 -> BOOLEAN
    ECS_COUNT(world)                      -> INTEGER

    ECS_ADD_INT(world, ent, name, value)
    ECS_ADD_FLOAT(world, ent, name, value)
    ECS_ADD_STRING(world, ent, name, value)
    ECS_ADD_BOOL(world, ent, name, value)
    ECS_ADD_OBJ(world, ent, name, value)  ' beliebiger Wert

    ECS_HAS(world, ent, name)             -> BOOLEAN
    ECS_REMOVE(world, ent, name)          -> BOOLEAN

    ECS_GET_INT(world, ent, name)         ' wirft wenn fehlt
    ECS_GET_FLOAT(world, ent, name)
    ECS_GET_STRING(world, ent, name)
    ECS_GET_BOOL(world, ent, name)
    ECS_GET(world, ent, name)             ' beliebiger Wert
    ECS_GET_OR_INT(world, ent, name, default)
    ECS_GET_OR_FLOAT(world, ent, name, default)
    ECS_GET_OR_STRING(world, ent, name, default)
    ECS_GET_OR_BOOL(world, ent, name, default)

    ECS_QUERY(world, name)                -> ARRAY OF INTEGER
    ECS_QUERY2(world, n1, n2)             -> ARRAY OF INTEGER
    ECS_QUERY3(world, n1, n2, n3)         -> ARRAY OF INTEGER

Storage-Architektur (Sparse-Set):
  Jeder Component hat drei parallele Strukturen:
    dense:  list[entity_id]    -- kompakte Liste, eine Eintrag je Halter
    values: list[Any]          -- parallel zu dense, gleicher Index
    sparse: dict[entity_id, dense_index]   -- O(1) Lookup
  Iteration laeuft kontinuierlich ueber `dense`/`values` (cache-freundlich),
  GET/HAS sind ein einzelner Dict-Lookup, ADD/REMOVE laufen swap-mit-letztem
  in O(1) ohne Listen-Verschiebung.

Implementation: ``_World`` und ``_Component`` sind cdef-Klassen in
``ecs_native.pyx``. Hot-Path-Methoden (`get_float`, `add_float`, ...) sind
cpdef auf `_World` -- die @builtin-Wrapper unten sind Thin-Shims, die nur
typcheck-en und delegieren. Ohne die .pyd faellt der Import auf eine
Pure-Python-Variante zurueck (Tree-Walker bleibt benutzbar).
"""
from __future__ import annotations

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type


# ECS-Kernklassen: reine Python-Implementation (Cython `ecs_native` entfernt;
# Tree-Walker = Editor-/Referenzpfad, Performance liegt in der nativen Runtime).
from .ecs_py import _World, _Component


register_type("ecs_world", _World)


# --- Type-Helpers (nur fuer Args, die keine Fast-Path-Methode hat) -

def _check_world(v, fn):
    if not isinstance(v, _World):
        raise TypeMismatchError(f"{fn} erwartet ECS_WORLD")
    return v


def _check_str(v, fn, label="STRING"):
    if not isinstance(v, str):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return v


def _check_int(v, fn, label="INTEGER"):
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return v


def _check_num(v, fn, label="Zahl"):
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return float(v)


def _check_bool(v, fn, label="BOOLEAN"):
    if not isinstance(v, bool):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return v


# --- World-Lifecycle -----------------------------------------------

@builtin("ECS_NEW_WORLD", arity=0)
def _b_new_world():
    return _World()


@builtin("ECS_NEW_ENTITY", arity=1)
def _b_new_entity(world):
    return _check_world(world, "ECS_NEW_ENTITY").new_entity()


@builtin("ECS_DESTROY", arity=2, types=("any", "int"))
def _b_destroy(world, ent):
    return _check_world(world, "ECS_DESTROY").destroy(ent)


@builtin("ECS_ALIVE", arity=2, types=("any", "int"))
def _b_alive(world, ent):
    return _check_world(world, "ECS_ALIVE").alive(ent)


@builtin("ECS_COUNT", arity=1)
def _b_count(world):
    return _check_world(world, "ECS_COUNT").count()


# --- Component-Add / Has / Remove ----------------------------------

@builtin("ECS_ADD_INT", arity=4)
def _b_add_int(world, ent, name, value):
    _check_world(world, "ECS_ADD_INT").add_int(
        ent, _check_str(name, "ECS_ADD_INT", "name"), value
    )


@builtin("ECS_ADD_FLOAT", arity=4)
def _b_add_float(world, ent, name, value):
    _check_world(world, "ECS_ADD_FLOAT").add_float(
        ent, _check_str(name, "ECS_ADD_FLOAT", "name"), value
    )


@builtin("ECS_ADD_STRING", arity=4)
def _b_add_string(world, ent, name, value):
    _check_world(world, "ECS_ADD_STRING").add_string(
        ent, _check_str(name, "ECS_ADD_STRING", "name"), value
    )


@builtin("ECS_ADD_BOOL", arity=4)
def _b_add_bool(world, ent, name, value):
    _check_world(world, "ECS_ADD_BOOL").add_bool(
        ent, _check_str(name, "ECS_ADD_BOOL", "name"), value
    )


@builtin("ECS_ADD_OBJ", arity=4)
def _b_add_obj(world, ent, name, value):
    """Component-Wert ist ein beliebiges Objekt (User-Klasse, MAP, ARRAY, ...)."""
    _check_world(world, "ECS_ADD_OBJ").add_obj(
        ent, _check_str(name, "ECS_ADD_OBJ", "name"), value
    )


@builtin("ECS_HAS", arity=3, types=("any", "int", "str"))
def _b_has(world, ent, name):
    return _check_world(world, "ECS_HAS").has_component(ent, name)


@builtin("ECS_REMOVE", arity=3, types=("any", "int", "str"))
def _b_remove(world, ent, name):
    return _check_world(world, "ECS_REMOVE").remove_component(ent, name)


# --- Component-Get -------------------------------------------------

@builtin("ECS_GET", arity=3)
def _b_get(world, ent, name):
    return _check_world(world, "ECS_GET").get(
        ent, _check_str(name, "ECS_GET", "name")
    )


@builtin("ECS_GET_INT", arity=3)
def _b_get_int(world, ent, name):
    return _check_world(world, "ECS_GET_INT").get_int(
        ent, _check_str(name, "ECS_GET_INT", "name")
    )


@builtin("ECS_GET_FLOAT", arity=3)
def _b_get_float(world, ent, name):
    return _check_world(world, "ECS_GET_FLOAT").get_float(
        ent, _check_str(name, "ECS_GET_FLOAT", "name")
    )


@builtin("ECS_GET_STRING", arity=3)
def _b_get_string(world, ent, name):
    return _check_world(world, "ECS_GET_STRING").get_string(
        ent, _check_str(name, "ECS_GET_STRING", "name")
    )


@builtin("ECS_GET_BOOL", arity=3)
def _b_get_bool(world, ent, name):
    return _check_world(world, "ECS_GET_BOOL").get_bool(
        ent, _check_str(name, "ECS_GET_BOOL", "name")
    )


# GET_OR_*-Varianten: schicken nicht durch _World.get_*, weil deren
# Verhalten "wirft auf Fehler" ist. Wir prueffen direkt auf das
# Sparse-Set.

@builtin("ECS_GET_OR_INT", arity=4)
def _b_get_or_int(world, ent, name, default):
    world = _check_world(world, "ECS_GET_OR_INT")
    name = _check_str(name, "ECS_GET_OR_INT", "name")
    default = _check_int(default, "ECS_GET_OR_INT", "default")
    c = world.components.get(name)
    if c is None:
        return default
    v = c.get_or(ent, None)
    if v is None or isinstance(v, bool) or not isinstance(v, int):
        return default
    return v


@builtin("ECS_GET_OR_FLOAT", arity=4)
def _b_get_or_float(world, ent, name, default):
    world = _check_world(world, "ECS_GET_OR_FLOAT")
    name = _check_str(name, "ECS_GET_OR_FLOAT", "name")
    default = _check_num(default, "ECS_GET_OR_FLOAT", "default")
    c = world.components.get(name)
    if c is None:
        return default
    v = c.get_or(ent, None)
    if v is None or isinstance(v, bool) or not isinstance(v, (int, float)):
        return default
    return float(v)


@builtin("ECS_GET_OR_STRING", arity=4)
def _b_get_or_string(world, ent, name, default):
    world = _check_world(world, "ECS_GET_OR_STRING")
    name = _check_str(name, "ECS_GET_OR_STRING", "name")
    default = _check_str(default, "ECS_GET_OR_STRING", "default")
    c = world.components.get(name)
    if c is None:
        return default
    v = c.get_or(ent, None)
    if not isinstance(v, str):
        return default
    return v


@builtin("ECS_GET_OR_BOOL", arity=4)
def _b_get_or_bool(world, ent, name, default):
    world = _check_world(world, "ECS_GET_OR_BOOL")
    name = _check_str(name, "ECS_GET_OR_BOOL", "name")
    default = _check_bool(default, "ECS_GET_OR_BOOL", "default")
    c = world.components.get(name)
    if c is None:
        return default
    v = c.get_or(ent, None)
    if not isinstance(v, bool):
        return default
    return v


# --- Query ---------------------------------------------------------

def _query_intersect(world, names, fn):
    """Liefert sortierte Liste der Entity-IDs, die ALLE Components haben.

    Strategie: mit dem Component mit den wenigsten Haltern starten, ueber
    dessen `dense`-Liste iterieren und gegen die anderen Sparse-Sets
    pruefen. Die Iteration ueber `dense` ist eine flache Liste -- viel
    cache-freundlicher als ein Dict-View ueber tausende Entries."""
    if not names:
        return []
    comps = []
    for name in names:
        c = world.components.get(name)
        if c is None or len(c.dense) == 0:
            return []
        comps.append(c)
    # Mit dem kleinsten Component anfangen
    comps.sort(key=lambda c: len(c.dense))
    base = comps[0]
    others = comps[1:]
    if not others:
        return sorted(base.dense)
    out = []
    for ent in base.dense:
        if all(c.has(ent) for c in others):
            out.append(ent)
    out.sort()
    return out


def _make_array_of_int(values):
    """Liefert einen 1D `_GBArray` von INTEGER-Entity-IDs."""
    from ..interpreter import _GBArray
    arr = _GBArray("integer", [len(values)], lambda: 0)
    for i, v in enumerate(values):
        arr.values[i] = v
    return arr


@builtin("ECS_QUERY", arity=2, types=("any", "str"))
def _b_query(world, name):
    world = _check_world(world, "ECS_QUERY")
    return _make_array_of_int(_query_intersect(world, [name], "ECS_QUERY"))


@builtin("ECS_QUERY2", arity=3, types=("any", "str", "str"))
def _b_query2(world, n1, n2):
    world = _check_world(world, "ECS_QUERY2")
    return _make_array_of_int(_query_intersect(world, [n1, n2], "ECS_QUERY2"))


@builtin("ECS_QUERY3", arity=4, types=("any", "str", "str", "str"))
def _b_query3(world, n1, n2, n3):
    world = _check_world(world, "ECS_QUERY3")
    return _make_array_of_int(
        _query_intersect(world, [n1, n2, n3], "ECS_QUERY3")
    )


# --- Bulk-System-Ops -----------------------------------------------

@builtin("ECS_INTEGRATE_FLOAT", arity=3, types=("any", "str", "str"))
def _b_integrate_float(world, target, delta):
    """Fuer alle Entities, die `target` UND `delta` haben: target += delta.

    Klassisches ECS-Pattern -- ersetzt eine Iteration in BASIC ueber
    LEN(query) Entities mit pro-Entity ECS_GET_FLOAT/ECS_ADD_FLOAT durch
    einen einzigen Builtin-Call, der den gesamten System-Tick in einer
    C-Loop abwickelt. Liefert die Anzahl der bewegten Entities.

    Beispiel:
        ECS_INTEGRATE_FLOAT(world, "px", "vx")   ' Position += Velocity
        ECS_INTEGRATE_FLOAT(world, "py", "vy")
    """
    return _check_world(world, "ECS_INTEGRATE_FLOAT").integrate_float(target, delta)


@builtin("ECS_INTEGRATE_INT", arity=3, types=("any", "str", "str"))
def _b_integrate_int(world, target, delta):
    """INT-Variante von ECS_INTEGRATE_FLOAT. target += delta fuer alle
    Entities mit beiden Components."""
    return _check_world(world, "ECS_INTEGRATE_INT").integrate_int(target, delta)


@builtin("ECS_SCALE_FLOAT", arity=3, types=("any", "str", "num"))
def _b_scale_float(world, target, factor):
    """Multipliziert alle Werte eines FLOAT-Components mit `factor`.
    Klassiker fuer Friction:  ECS_SCALE_FLOAT(world, "vel", 0.95).
    Liefert Anzahl skalierter Entities.
    """
    return _check_world(world, "ECS_SCALE_FLOAT").scale_float(target, float(factor))


@builtin("ECS_FILL_FLOAT", arity=3, types=("any", "str", "num"))
def _b_fill_float(world, target, value):
    """Setzt alle Werte eines FLOAT-Components auf `value`. Liefert
    Anzahl Halter."""
    return _check_world(world, "ECS_FILL_FLOAT").fill_float(target, float(value))


@builtin("ECS_FILL_INT", arity=3, types=("any", "str", "int"))
def _b_fill_int(world, target, value):
    """Setzt alle Werte eines INT-Components auf `value`. Liefert
    Anzahl Halter."""
    return _check_world(world, "ECS_FILL_INT").fill_int(target, int(value))


@builtin("ECS_CLAMP_FLOAT", arity=4, types=("any", "str", "num", "num"))
def _b_clamp_float(world, target, lo, hi):
    """Klemmt alle Werte eines FLOAT-Components auf [lo, hi]. Useful
    fuer Bounds-Checks (Position auf Screen halten). Liefert Anzahl
    Halter."""
    return _check_world(world, "ECS_CLAMP_FLOAT").clamp_float(
        target, float(lo), float(hi)
    )


@builtin("ECS_REMOVE_DEAD", arity=3, types=("any", "str", "num"))
def _b_remove_dead(world, name, threshold):
    """Zerstoert alle Entities, deren `name`-Component-Wert <= `threshold`
    ist. Klassiker:  ECS_REMOVE_DEAD(world, "hp", 0).  Liefert Anzahl
    zerstoerter Entities."""
    return _check_world(world, "ECS_REMOVE_DEAD").remove_dead(
        name, float(threshold)
    )


@builtin("ECS_COUNT_WITH", arity=2, types=("any", "str"))
def _b_count_with(world, name):
    """Zaehlt Entities, die `name` als Component haben. O(1) -- liest
    nur die Sparse-Set-Laenge, kein Scan."""
    return _check_world(world, "ECS_COUNT_WITH").count_with(name)
