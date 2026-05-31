"""Decorator-basierte Registrierung fuer GameBasic-Built-ins.

Statt jede Built-in-Funktion mit `if len(args) != X` und Type-Checks von Hand
zu validieren, deklariert man arity und Argument-Typen am Decorator. Der
Wrapper kuemmert sich um die Pruefung und entpackt die args zu Positions-
Argumenten.

Zwei oeffentliche Dicts (`BUILTINS`, `GRAPHICS_BUILTINS`) werden gefuellt.
`interpreter.py` re-exportiert sie unter denselben Namen, damit `vm.py`,
`vm_native.pyx` und `editor.py` ihre Imports nicht aendern muessen.

Beispiele:

    @builtin("SIN", arity=1, types=("num",))
    def _b_sin(x):
        return math.sin(x)

    @builtin(("UPPER$", "UPPER"), arity=1, types=("str",))
    def _b_upper(s):
        return s.upper()

    @builtin("RND", arity=(0, 1))
    def _b_rnd(*args):
        # Eigene Validierung wenn types nicht reicht
        ...

    @graphics_builtin("BEEP", arity=2, types=("num", "num"))
    def _b_beep(freq, ms):
        ...
"""
from .errors import GBRuntimeError, TypeMismatchError


BUILTINS: dict = {}
GRAPHICS_BUILTINS: dict = {}


# Type-Checker pro Spec. `type(v) is X` ist schneller als isinstance() weil
# keine MRO-Suche - und `type(True) is int` ist False, also faellt der Bool-
# Sonderfall (GB betrachtet Bool nicht als Zahl) automatisch raus.

def _check_any(v, fn):
    return v


def _check_num(v, fn):
    t = type(v)
    if t is int or t is float:
        return v
    raise TypeMismatchError(f"{fn} erwartet Zahl, erhalten {type(v).__name__}")


def _check_int(v, fn):
    if type(v) is int:
        return v
    raise TypeMismatchError(f"{fn} erwartet INTEGER, erhalten {type(v).__name__}")


def _check_intish(v, fn):
    t = type(v)
    if t is int:
        return v
    if t is float:
        return int(v)
    raise TypeMismatchError(f"{fn} erwartet Zahl, erhalten {type(v).__name__}")


def _check_str(v, fn):
    if type(v) is str:
        return v
    raise TypeMismatchError(f"{fn} erwartet STRING, erhalten {type(v).__name__}")


def _check_bool(v, fn):
    if type(v) is bool:
        return v
    raise TypeMismatchError(f"{fn} erwartet BOOLEAN, erhalten {type(v).__name__}")


_TYPE_CHECKS = {
    "any":    _check_any,
    "num":    _check_num,
    "int":    _check_int,
    "intish": _check_intish,
    "str":    _check_str,
    "bool":   _check_bool,
}


def _check_type(v, spec: str, fn: str):
    """Single-Dispatch Type-Check (Fassade fuer externe Aufrufer)."""
    try:
        checker = _TYPE_CHECKS[spec]
    except KeyError:
        raise ValueError(f"Unbekannte Typ-Spec: {spec!r}")
    return checker(v, fn)


def _check_arity(primary: str, arity, n: int) -> None:
    if arity is None:
        return
    if isinstance(arity, tuple):
        lo, hi = arity
        if hi is None:  # (lo, None) -> mindestens lo, kein Maximum
            if n < lo:
                raise GBRuntimeError(
                    f"{primary} erwartet >= {lo} Argument(e), erhalten {n}"
                )
            return
        if not (lo <= n <= hi):
            if lo == hi:
                raise GBRuntimeError(
                    f"{primary} erwartet {lo} Argument(e), erhalten {n}"
                )
            raise GBRuntimeError(
                f"{primary} erwartet {lo}..{hi} Argumente, erhalten {n}"
            )
    else:
        if n != arity:
            raise GBRuntimeError(
                f"{primary} erwartet {arity} Argument(e), erhalten {n}"
            )


def _validate_decl(primary: str, arity, types) -> None:
    if types is not None and isinstance(arity, tuple):
        raise ValueError(
            f"{primary}: types= ist nur bei fixer arity zulaessig "
            f"(variable arity selbst validieren)"
        )
    if types is not None and arity is not None and len(types) != arity:
        raise ValueError(
            f"{primary}: types hat {len(types)} Eintraege, arity={arity}"
        )


def _extract_param_names(fn, skip_first: bool = False) -> list:
    """Holt die Parameter-Namen aus einer Funktion via inspect. Wenn die
    Funktion *args nutzt (variable Arity), gibt es einen einzigen Eintrag
    "*args"."""
    import inspect
    try:
        sig = inspect.signature(fn)
    except (ValueError, TypeError):
        return []
    out = []
    for p in sig.parameters.values():
        if p.kind == inspect.Parameter.VAR_POSITIONAL:
            out.append("*args")
        else:
            out.append(p.name)
    if skip_first and out and out[0] != "*args":
        out = out[1:]
    return out


def _attach_metadata(wrapper, fn, primary: str, arity, types, kind: str,
                     skip_first_param: bool):
    wrapper.__name__ = getattr(fn, "__name__", primary)
    wrapper.__wrapped__ = fn
    wrapper._gb_primary = primary
    wrapper._gb_arity = arity
    wrapper._gb_types = types
    wrapper._gb_kind = kind  # "core" oder "graphics"
    wrapper._gb_param_names = _extract_param_names(fn, skip_first=skip_first_param)


def _arity_msg(primary: str, n: int, got: int) -> str:
    return f"{primary} erwartet {n} Argument(e), erhalten {got}"


def _make_core_wrapper(fn, primary: str, arity, types):
    """Wrapper fuer BUILTINS: Aufruf-Signatur ist wrapper(args).

    Spezialisiert auf arity 0/1/2/3 (deckt ~85% aller Builtins ab) - vermeidet
    fuer den Hot-Path die per-Call Listen-Allokation und das zip().
    """
    _validate_decl(primary, arity, types)

    wrapper = _build_core_specialized(fn, primary, arity, types)
    if wrapper is None:
        # Generischer Fallback: variable Arity oder Arity >= 4.
        if types is not None:
            checkers = tuple(_TYPE_CHECKS[t] for t in types)
            def wrapper(args):
                _check_arity(primary, arity, len(args))
                return fn(*[c(a, primary) for c, a in zip(checkers, args)])
        else:
            def wrapper(args):
                _check_arity(primary, arity, len(args))
                return fn(*args)

    _attach_metadata(wrapper, fn, primary, arity, types, "core",
                     skip_first_param=False)
    return wrapper


def _build_core_specialized(fn, primary, arity, types):
    """Gibt einen spezialisierten Wrapper fuer fixe arity 0..3 zurueck, sonst None."""
    if not isinstance(arity, int) or arity > 3:
        return None
    if types is not None:
        cks = tuple(_TYPE_CHECKS[t] for t in types)
        if arity == 0:
            def wrapper(args):
                if len(args) != 0:
                    raise GBRuntimeError(_arity_msg(primary, 0, len(args)))
                return fn()
        elif arity == 1:
            c0 = cks[0]
            def wrapper(args):
                if len(args) != 1:
                    raise GBRuntimeError(_arity_msg(primary, 1, len(args)))
                return fn(c0(args[0], primary))
        elif arity == 2:
            c0, c1 = cks
            def wrapper(args):
                if len(args) != 2:
                    raise GBRuntimeError(_arity_msg(primary, 2, len(args)))
                return fn(c0(args[0], primary), c1(args[1], primary))
        else:  # arity == 3
            c0, c1, c2 = cks
            def wrapper(args):
                if len(args) != 3:
                    raise GBRuntimeError(_arity_msg(primary, 3, len(args)))
                return fn(c0(args[0], primary),
                          c1(args[1], primary),
                          c2(args[2], primary))
        return wrapper
    # Untyped
    if arity == 0:
        def wrapper(args):
            if len(args) != 0:
                raise GBRuntimeError(_arity_msg(primary, 0, len(args)))
            return fn()
    elif arity == 1:
        def wrapper(args):
            if len(args) != 1:
                raise GBRuntimeError(_arity_msg(primary, 1, len(args)))
            return fn(args[0])
    elif arity == 2:
        def wrapper(args):
            if len(args) != 2:
                raise GBRuntimeError(_arity_msg(primary, 2, len(args)))
            return fn(args[0], args[1])
    else:  # arity == 3
        def wrapper(args):
            if len(args) != 3:
                raise GBRuntimeError(_arity_msg(primary, 3, len(args)))
            return fn(args[0], args[1], args[2])
    return wrapper


def _make_graphics_wrapper(fn, primary: str, arity, types):
    """Wrapper fuer GRAPHICS_BUILTINS: Aufruf-Signatur ist wrapper(g, args).

    Die dekorierte Funktion erhaelt (g, *checked_args) als Positions-Argumente.
    Spezialisiert wie der core-Wrapper.
    """
    _validate_decl(primary, arity, types)

    wrapper = _build_graphics_specialized(fn, primary, arity, types)
    if wrapper is None:
        if types is not None:
            checkers = tuple(_TYPE_CHECKS[t] for t in types)
            def wrapper(g, args):
                _check_arity(primary, arity, len(args))
                return fn(g, *[c(a, primary) for c, a in zip(checkers, args)])
        else:
            def wrapper(g, args):
                _check_arity(primary, arity, len(args))
                return fn(g, *args)

    _attach_metadata(wrapper, fn, primary, arity, types, "graphics",
                     skip_first_param=True)
    return wrapper


def _build_graphics_specialized(fn, primary, arity, types):
    if not isinstance(arity, int) or arity > 3:
        return None
    if types is not None:
        cks = tuple(_TYPE_CHECKS[t] for t in types)
        if arity == 0:
            def wrapper(g, args):
                if len(args) != 0:
                    raise GBRuntimeError(_arity_msg(primary, 0, len(args)))
                return fn(g)
        elif arity == 1:
            c0 = cks[0]
            def wrapper(g, args):
                if len(args) != 1:
                    raise GBRuntimeError(_arity_msg(primary, 1, len(args)))
                return fn(g, c0(args[0], primary))
        elif arity == 2:
            c0, c1 = cks
            def wrapper(g, args):
                if len(args) != 2:
                    raise GBRuntimeError(_arity_msg(primary, 2, len(args)))
                return fn(g, c0(args[0], primary), c1(args[1], primary))
        else:  # arity == 3
            c0, c1, c2 = cks
            def wrapper(g, args):
                if len(args) != 3:
                    raise GBRuntimeError(_arity_msg(primary, 3, len(args)))
                return fn(g, c0(args[0], primary),
                          c1(args[1], primary),
                          c2(args[2], primary))
        return wrapper
    # Untyped
    if arity == 0:
        def wrapper(g, args):
            if len(args) != 0:
                raise GBRuntimeError(_arity_msg(primary, 0, len(args)))
            return fn(g)
    elif arity == 1:
        def wrapper(g, args):
            if len(args) != 1:
                raise GBRuntimeError(_arity_msg(primary, 1, len(args)))
            return fn(g, args[0])
    elif arity == 2:
        def wrapper(g, args):
            if len(args) != 2:
                raise GBRuntimeError(_arity_msg(primary, 2, len(args)))
            return fn(g, args[0], args[1])
    else:  # arity == 3
        def wrapper(g, args):
            if len(args) != 3:
                raise GBRuntimeError(_arity_msg(primary, 3, len(args)))
            return fn(g, args[0], args[1], args[2])
    return wrapper


def _register(names, kind: str, wrapper):
    target = BUILTINS if kind == "core" else GRAPHICS_BUILTINS
    for n in names:
        target[n.lower()] = wrapper


def builtin(name, arity=None, types=None):
    """Registriert eine Funktion in BUILTINS.

    name:   String oder Tuple/List von Strings (Aliase, z.B. ("STR$","STR")).
    arity:  int (exakt) oder (lo, hi)-Tuple. None = beliebig (Funktion prueft selbst).
    types:  Tuple von Type-Specs ("num","int","str","bool","any"). Nur bei fixer arity.

    Die dekorierte Funktion erhaelt die Argumente als Positions-Parameter:

        @builtin("SIN", arity=1, types=("num",))
        def _b_sin(x): return math.sin(x)
    """
    names = (name,) if isinstance(name, str) else tuple(name)
    primary = names[0]

    def decorator(fn):
        wrapper = _make_core_wrapper(fn, primary, arity, types)
        _register(names, "core", wrapper)
        return wrapper
    return decorator


# --- Signatur-Formatter (fuer Editor-Tooltips) ----------------------

_GB_TYPE_LABEL = {
    "num":    "Zahl",
    "int":    "INTEGER",
    "intish": "Zahl",
    "str":    "STRING",
    "bool":   "BOOLEAN",
    "any":    "?",
}


def signature_text(name: str) -> str:
    """Formatiert die Signatur eines Built-ins fuer Tooltips.

    Beispiele:
        SIN(x: Zahl)
        JSON_GET_STRING(h, path: STRING)
        RND(*args)        ' bei variabler Arity ohne Type-Spec
        MIN(*args)        ' bei (min, None)-Arity
        TWEEN_NEW(start: Zahl, end: Zahl, dauer_ms[, easing])

    Gibt einen leeren String zurueck wenn der Name kein Built-in ist.
    """
    wrapper = BUILTINS.get(name.lower()) or GRAPHICS_BUILTINS.get(name.lower())
    if wrapper is None or not hasattr(wrapper, "_gb_primary"):
        return ""
    primary = wrapper._gb_primary
    arity = wrapper._gb_arity
    types = wrapper._gb_types
    param_names = list(wrapper._gb_param_names)

    # Variable Arity oder *args -> generische Darstellung
    if param_names == ["*args"] or any(p == "*args" for p in param_names):
        if isinstance(arity, tuple):
            lo, hi = arity
            if hi is None:
                return f"{primary}({lo}+ Argumente)"
            if lo == hi:
                return f"{primary}({lo} Argumente)"
            return f"{primary}({lo}..{hi} Argumente)"
        return f"{primary}(*args)"

    # Fixe arity mit Parameter-Namen
    parts = []
    for i, pname in enumerate(param_names):
        if types is not None and i < len(types):
            label = _GB_TYPE_LABEL.get(types[i], types[i])
            parts.append(f"{pname}: {label}")
        else:
            parts.append(pname)
    return f"{primary}({', '.join(parts)})"


def graphics_builtin(name, arity=None, types=None):
    """Registriert eine Funktion in GRAPHICS_BUILTINS.

    Die dekorierte Funktion erhaelt den Graphics-Kontext als erstes
    Argument, gefolgt von den geprueften Built-in-Argumenten:

        @graphics_builtin("BEEP", arity=2, types=("num", "num"))
        def _g_beep(g, freq, ms): ...
    """
    names = (name,) if isinstance(name, str) else tuple(name)
    primary = names[0]

    def decorator(fn):
        wrapper = _make_graphics_wrapper(fn, primary, arity, types)
        _register(names, "graphics", wrapper)
        return wrapper
    return decorator
