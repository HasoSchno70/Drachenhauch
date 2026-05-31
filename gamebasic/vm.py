"""Bytecode-VM fuer GameBasic (Stack-basiert).

Globale werden als _Slot-Objekte gehalten (name -> _Slot(type, value, const));
__slots__ spart pro LOAD_NAME/STORE_NAME einen Inner-Dict-Lookup.
Lokale liegen in einer Python-Liste, indiziert per Slot.

Built-ins werden zur Laufzeit aus interpreter.BUILTINS aufgeloest. Damit
profitiert die VM ohne weiteren Aufwand von dem schon implementierten
Standardfunktions-Set.
"""
from __future__ import annotations

import math
import sys

from .bytecode import Op, CompiledFunction, Module, VMClassInfo
from .errors import GameBasicError, GBRuntimeError, TypeMismatchError
from .interpreter import (  # type: ignore
    BUILTINS, GRAPHICS_BUILTINS,
    _Instance, _GBArray, _Image, _Sound, _GBFile, _GBMap,
    _GBThrow, _EnumNamespace, _ClassStaticNamespace, _FuncRef,
)
from .modules import dispatch_binary_op as _disp_op, NO_OP_MATCH as _NO_OP


_TYPE_DEFAULTS = {
    "integer": 0,
    "float":   0.0,
    "string":  "",
    "boolean": False,
    "image":   None,
    "sound":   None,
    "file":    None,
    "tuple":   (),
    "funcref": None,
}


def _type_of(v) -> str:
    if v is None:
        return "NIL"
    if isinstance(v, bool):
        return "BOOLEAN"
    if isinstance(v, int):
        return "INTEGER"
    if isinstance(v, float):
        return "FLOAT"
    if isinstance(v, str):
        return "STRING"
    if isinstance(v, tuple):
        return f"TUPLE({len(v)})"
    if isinstance(v, _FuncRef):
        return f"FUNCREF<{v.name}>"
    from gamebasic.modules.vec2 import _Vec2 as _V2
    if isinstance(v, _V2):
        return "VEC2"
    if isinstance(v, _Instance):
        return v.cls.name.upper()
    if isinstance(v, _GBArray):
        return f"ARRAY OF {v.element_type.upper()}"
    if isinstance(v, _Image):
        return "IMAGE"
    if isinstance(v, _Sound):
        return "SOUND"
    from .interpreter import _SpriteAtlas as _SA
    if isinstance(v, _SA):
        return "SPRITE_ATLAS"
    if isinstance(v, _GBFile):
        return "FILE"
    if isinstance(v, _GBMap):
        return f"MAP OF {v.value_type.upper()}"
    return type(v).__name__.upper()


def _is_subclass_of(child, parent) -> bool:
    cur = child
    while cur is not None:
        if cur is parent:
            return True
        cur = cur.parent
    return False


# Coercion-Fast-Path: dict-dispatch + `type(v) is X`. Wird von STORE_NAME,
# STORE_LOCAL, STORE_FIELD, STORE_INDEX und Parameter-Binding benutzt -
# einer der heissesten Pfade in der VM.

def _coerce_any(value, ctx):
    return value


def _coerce_integer(value, ctx):
    t = type(value)
    if t is int:
        return value
    if t is float:
        if not value.is_integer():
            raise TypeMismatchError(
                f"{ctx}: FLOAT {value} kann nicht ohne Verlust nach INTEGER (nutze INT())"
            )
        return int(value)
    if t is bool:
        raise TypeMismatchError(f"{ctx}: Erwartet INTEGER, erhalten BOOLEAN")
    raise TypeMismatchError(f"{ctx}: Erwartet INTEGER, erhalten {_type_of(value)}")


def _coerce_float(value, ctx):
    t = type(value)
    if t is float:
        return value
    if t is int:
        return float(value)
    if t is bool:
        raise TypeMismatchError(f"{ctx}: Erwartet FLOAT, erhalten BOOLEAN")
    raise TypeMismatchError(f"{ctx}: Erwartet FLOAT, erhalten {_type_of(value)}")


def _coerce_string(value, ctx):
    if type(value) is str:
        return value
    raise TypeMismatchError(f"{ctx}: Erwartet STRING, erhalten {_type_of(value)}")


def _coerce_boolean(value, ctx):
    if type(value) is bool:
        return value
    raise TypeMismatchError(f"{ctx}: Erwartet BOOLEAN, erhalten {_type_of(value)}")


def _coerce_image(value, ctx):
    if value is None or isinstance(value, _Image):
        return value
    raise TypeMismatchError(f"{ctx}: Erwartet IMAGE, erhalten {_type_of(value)}")


def _coerce_sound(value, ctx):
    if value is None or isinstance(value, _Sound):
        return value
    raise TypeMismatchError(f"{ctx}: Erwartet SOUND, erhalten {_type_of(value)}")


def _coerce_sprite_atlas(value, ctx):
    from .interpreter import _SpriteAtlas
    if value is None or isinstance(value, _SpriteAtlas):
        return value
    raise TypeMismatchError(
        f"{ctx}: Erwartet SPRITE_ATLAS, erhalten {_type_of(value)}"
    )


def _coerce_file(value, ctx):
    if value is None or isinstance(value, _GBFile):
        return value
    raise TypeMismatchError(f"{ctx}: Erwartet FILE, erhalten {_type_of(value)}")


def _coerce_tuple(value, ctx):
    if isinstance(value, tuple):
        return value
    raise TypeMismatchError(f"{ctx}: Erwartet TUPLE, erhalten {_type_of(value)}")


def _coerce_funcref(value, ctx):
    if isinstance(value, _FuncRef):
        return value
    raise TypeMismatchError(f"{ctx}: Erwartet FUNCREF, erhalten {_type_of(value)}")


_FAST_COERCE = {
    "any":     _coerce_any,
    "integer": _coerce_integer,
    "float":   _coerce_float,
    "string":  _coerce_string,
    "boolean": _coerce_boolean,
    "image":   _coerce_image,
    "sound":   _coerce_sound,
    "sprite_atlas": _coerce_sprite_atlas,
    "file":    _coerce_file,
    "tuple":   _coerce_tuple,
    "funcref": _coerce_funcref,
}


def _coerce(value, target: str, ctx: str, classes: dict | None = None):
    fast = _FAST_COERCE.get(target)
    if fast is not None:
        return fast(value, ctx)
    # MAP-Typ
    if target.startswith("map:"):
        vt = target[4:]
        if value is None:
            return None
        if not isinstance(value, _GBMap):
            raise TypeMismatchError(
                f"{ctx}: Erwartet MAP OF {vt.upper()}, erhalten {_type_of(value)}"
            )
        if value.value_type != vt:
            raise TypeMismatchError(
                f"{ctx}: Erwartet MAP OF {vt.upper()}, "
                f"erhalten MAP OF {value.value_type.upper()}"
            )
        return value
    # Array-Typ
    if target.startswith("array:"):
        elem = target[len("array:"):]
        if value is None:
            return None
        if not isinstance(value, _GBArray):
            raise TypeMismatchError(
                f"{ctx}: Erwartet ARRAY OF {elem.upper()}, erhalten {_type_of(value)}"
            )
        if value.element_type != elem:
            raise TypeMismatchError(
                f"{ctx}: Erwartet ARRAY OF {elem.upper()}, "
                f"erhalten ARRAY OF {value.element_type.upper()}"
            )
        return value
    # Klassen-/Struct-Typ
    if classes is not None:
        target_cls = classes.get(target)
        if target_cls is not None:
            if value is None:
                return None
            if not isinstance(value, _Instance):
                raise TypeMismatchError(
                    f"{ctx}: Erwartet {target}, erhalten {_type_of(value)}"
                )
            if not _is_subclass_of(value.cls, target_cls):
                raise TypeMismatchError(
                    f"{ctx}: Erwartet {target} (oder Unterklasse), erhalten {value.cls.name}"
                )
            return value
    # Externer Typ aus Built-in-Modul (z.B. JSON_HANDLE).
    from .modules import EXTERNAL_TYPES as _EXT_TYPES
    ext_cls = _EXT_TYPES.get(target)
    if ext_cls is not None:
        if value is None or isinstance(value, ext_cls):
            return value
        raise TypeMismatchError(
            f"{ctx}: Erwartet {target.upper()}, erhalten {_type_of(value)}"
        )
    raise GBRuntimeError(f"Unbekannter Zieltyp: {target}")


def _fmt(v) -> str:
    if v is None:
        return "NIL"
    if isinstance(v, bool):
        return "TRUE" if v else "FALSE"
    if isinstance(v, float):
        if v.is_integer():
            return f"{v:.1f}"
        return repr(v)
    if isinstance(v, tuple):
        return "(" + ", ".join(_fmt(x) for x in v) + ")"
    if isinstance(v, _FuncRef):
        return f"<FUNCREF {v.name}>"
    from gamebasic.modules.vec2 import _Vec2 as _V2
    if isinstance(v, _V2):
        return f"Vec2({_fmt(v.x)}, {_fmt(v.y)})"
    if isinstance(v, _Instance):
        return f"<{v.cls.name}>"
    if isinstance(v, _GBArray):
        shape = ",".join(str(d) for d in v.dims) if v.dims else ""
        return f"<ARRAY[{shape}] OF {v.element_type.upper()}>"
    if isinstance(v, _Image):
        return "<IMAGE>"
    if isinstance(v, _Sound):
        return "<SOUND>"
    from .interpreter import _SpriteAtlas as _SA
    if isinstance(v, _SA):
        return f"<SPRITE_ATLAS frames={len(v.frames)}>"
    if isinstance(v, _GBFile):
        return f"<FILE {v.path}>"
    if isinstance(v, _GBMap):
        return f"<MAP[{len(v.data)}] OF {v.value_type.upper()}>"
    return str(v)


def _infer_type(value) -> str:
    # Kanonische Version aus interpreter.py (Single-Source -- alle drei
    # Pfade leiten dieselben Typen ab).
    from .interpreter import infer_type
    return infer_type(value)


def _truthy(v) -> bool:
    if isinstance(v, bool):
        return v
    if isinstance(v, (int, float)):
        return v != 0
    if isinstance(v, str):
        return v != ""
    return v is not None


def _require_int_pair(a, b, op: str):
    """Strikte INTEGER-Pruefung fuer Bitwise-Operatoren. Bool wird abgelehnt."""
    for v in (a, b):
        if isinstance(v, bool) or not isinstance(v, int):
            raise TypeMismatchError(f"{op} erwartet INTEGER, erhalten {_type_of(v)}")


def _require_number(a, b, op: str):
    """Zahlen-Pruefung fuer SUB/DIV/MOD/POW. Bool und Nicht-Zahl werfen --
    deckungsgleich mit Interpreter._require_number (Tree-Walker)."""
    for v in (a, b):
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise TypeMismatchError(f"Operator '{op}' erwartet Zahlen, erhalten {_type_of(v)}")


def _is_property(cls, name: str) -> bool:
    """TRUE wenn `name` als Property irgendwo in der Klassen-Vererbung
    deklariert ist. Lookup case-insensitive."""
    target = name.lower()
    cur = cls
    while cur is not None:
        if target in cur.properties:
            return True
        cur = cur.parent
    return False


def _eval_in(needle, haystack):
    """`needle IN haystack` -- gemeinsamer Helper fuer TW und VM."""
    if haystack is None:
        raise GBRuntimeError("IN: rechte Seite ist NIL")
    if isinstance(haystack, str):
        if not isinstance(needle, str):
            raise TypeMismatchError(
                f"IN bei STRING: linke Seite muss STRING sein, "
                f"erhalten {_type_of(needle)}"
            )
        return needle in haystack
    if isinstance(haystack, tuple):
        return needle in haystack
    if isinstance(haystack, _GBArray):
        if len(haystack.dims) != 1:
            raise GBRuntimeError("IN: nur 1D-Arrays unterstuetzt")
        return needle in haystack.values
    if isinstance(haystack, _GBMap):
        if not isinstance(needle, str):
            raise TypeMismatchError(
                f"IN bei MAP: Key muss STRING sein, erhalten {_type_of(needle)}"
            )
        return needle in haystack.data
    raise TypeMismatchError(
        f"IN: rechte Seite muss STRING, TUPLE, ARRAY oder MAP sein, "
        f"erhalten {_type_of(haystack)}"
    )


# Container-Kind-Erkennung UND Method-Map aus interpreter.py beziehen --
# Single-Source-of-Truth, damit Tree-Walker und VM denselben Dispatch
# verwenden (vermeidet Drift bei kuenftigen Container-Typen).
from .interpreter import (
    _container_kind,
    CONTAINER_METHODS as _CONTAINER_METHODS,
)


def _apply_slice(target, lo_val, hi_val):
    """Slice-Operation auf String oder 1D-Array. None fuer lo/hi heisst
    Default-Bound (0 / len). Negative Indices werden abgelehnt.
    """
    if isinstance(target, str):
        n = len(target)
    elif isinstance(target, _GBArray):
        if len(target.dims) != 1:
            raise GBRuntimeError("Slicing ist nur fuer 1D-Arrays unterstuetzt")
        n = target.dims[0]
    else:
        raise TypeMismatchError(
            f"Slice-Zugriff: Erwartet STRING oder ARRAY, erhalten {_type_of(target)}"
        )
    lo = 0 if lo_val is None else lo_val
    hi = n if hi_val is None else hi_val
    for v, label in ((lo, "lo"), (hi, "hi")):
        if isinstance(v, bool) or not isinstance(v, int):
            raise TypeMismatchError(
                f"Slice-Index ({label}) muss INTEGER sein, erhalten {_type_of(v)}"
            )
    if lo < 0 or hi < 0:
        raise GBRuntimeError("Negative Slice-Indices nicht unterstuetzt")
    if hi > n: hi = n
    if lo > n: lo = n
    if lo > hi: lo = hi
    if isinstance(target, str):
        return target[lo:hi]
    sub = target.values[lo:hi]
    new_dims = [hi - lo]
    result = _GBArray(target.element_type, new_dims,
                      lambda t=target.element_type: _TYPE_DEFAULTS.get(t, None))
    for i, v in enumerate(sub):
        result.values[i] = v
    return result


class _Slot:
    """Globaler-Variablen-Slot. __slots__ ersetzt {"type","value","const"}-dict."""
    __slots__ = ("type", "value", "const")

    def __init__(self, type_: str, value, const: bool = False):
        self.type = type_
        self.value = value
        self.const = const


# Konstanten-Registrierung wie im Tree-Walker (BLACK, WHITE, KEY_*, PI, ...)
def _register_default_globals(globals_dict: dict):
    try:
        from .graphics import COLORS, KEYS
    except Exception:
        COLORS = {}
        KEYS = {}
    for n, val in COLORS.items():
        globals_dict[n] = _Slot("integer", val, True)
    for n, val in KEYS.items():
        globals_dict[n] = _Slot("integer", val, True)
    globals_dict["pi"] = _Slot("float", math.pi, True)


class VM:
    def __init__(self):
        self.globals: dict = {}
        _register_default_globals(self.globals)
        self._graphics = None  # lazy
        self.data_ptr: int = 0     # READ liest hier sequenziell aus module.data

    def _get_graphics(self):
        if self._graphics is None:
            from .graphics import Graphics
            self._graphics = Graphics()
            self._graphics._gb_engine = self
        return self._graphics

    def gb_call_function(self, name, args):
        """Ruft eine User-Function per Name auf -- Bruecke fuer Builtin-
        Callbacks (z.B. GUI_ON_CLICK). `args` sind vorab-evaluierte Werte."""
        fn = self.module.functions.get(name)
        if fn is None:
            raise GBRuntimeError(f"FUNCREF: Funktion '{name}' existiert nicht (mehr)")
        return self._exec(fn, list(args))

    # ----------------------------------------------------------------
    def run(self, module: Module):
        self.module = module
        self.data_ptr = 0
        # Slot-Liste fuer compile-time-aufgeloeste Globals. Parallel zum
        # globals_-Dict; DECLARE_GLOBAL_SLOT schreibt in beide, sodass
        # name-basierte Ops (LOAD_NAME, INPUT_NAME) den Slot trotzdem
        # finden. Pre-registrierte Globals (KEY_*, PI, ...) leben nur
        # im Dict, weil der Compiler sie nicht slot-allokiert.
        self.global_slots: list = [None] * module.n_globals
        try:
            try:
                self._exec(module.main, [])
            except _Halt:
                return
        finally:
            if self._graphics is not None:
                self._graphics.shutdown()

    def _resolve_method(self, cls, name):
        cache = cls._method_cache
        hit = cache.get(name)
        if hit is not None:
            return hit
        cur = cls
        while cur is not None:
            if name in cur.methods:
                resolved = (cur.methods[name], cur)
                cache[name] = resolved
                return resolved
            cur = cur.parent
        return (None, None)

    def _user_op(self, op_method: str, a, b):
        """User-Class Operator-Dispatch fuer BinaryOp.

        Wenn `a` eine `_Instance` mit Methode `op_method` ist, ruf sie mit
        `b` als Argument. Sonst RHS probieren (Reverse-Dispatch fuer
        `5 + money`-Faelle, wenn Money `__op_add__(other AS INTEGER)`
        anbietet). Liefert `_NO_OP`, wenn keine User-Methode passt --
        dann uebernimmt der Standard-Pfad.
        """
        if isinstance(a, _Instance):
            m, _ = self._resolve_method(a.cls, op_method)
            if m is not None:
                return self._exec(m, [b], self_obj=a)
        if isinstance(b, _Instance):
            m, _ = self._resolve_method(b.cls, op_method)
            if m is not None:
                return self._exec(m, [a], self_obj=b)
        return _NO_OP

    def _allocate_instance(self, cls: VMClassInfo) -> _Instance:
        inst = _Instance(cls)
        chain = []
        cur = cls
        while cur is not None:
            chain.append(cur)
            cur = cur.parent
        for c in reversed(chain):
            for fd in c.fields:
                if fd.array_dims:
                    arr = _GBArray(
                        fd.type_name, list(fd.array_dims),
                        lambda t=fd.type_name: self._element_default(t),
                    )
                    inst.fields[fd.name] = {
                        "type": f"array:{fd.type_name}", "value": arr,
                    }
                else:
                    sub = self.module.classes.get(fd.type_name)
                    if sub is not None and sub.is_struct:
                        default = self._allocate_instance(sub)
                    else:
                        default = _TYPE_DEFAULTS.get(fd.type_name)
                    inst.fields[fd.name] = {
                        "type": fd.type_name, "value": default,
                    }
        return inst

    def _element_default(self, type_name: str):
        cls = self.module.classes.get(type_name)
        if cls is not None and cls.is_struct:
            return self._allocate_instance(cls)
        return _TYPE_DEFAULTS.get(type_name)

    def _coerce(self, value, target, ctx):
        return _coerce(value, target, ctx, self.module.classes)

    # ----------------------------------------------------------------
    def _exec(self, fn: CompiledFunction, args: list, self_obj=None):
        """Fuehrt eine Funktion aus, returniert den RETURN-Wert (oder None)."""
        # Lokale anlegen
        locals_ = list(fn.local_defaults)  # Kopie, gleiche Reihenfolge
        # Variadic: letzter Param ist TUPLE, sammelt alle Positional-Args
        # ab Position n_params-1. Defaults werden fuer Variadic-Slot nicht
        # benutzt (immer leeres Tupel wenn keine Extra-Args).
        if fn.is_variadic:
            normal_n = fn.n_params - 1
            if len(args) < normal_n:
                raise GBRuntimeError(
                    f"{fn.name.upper()}: erwartet mindestens {normal_n} "
                    f"Argument(e), erhalten {len(args)}"
                )
            for i in range(normal_n):
                locals_[i] = self._coerce(args[i], fn.local_types[i],
                                          f"Parameter {i+1}")
            locals_[fn.n_params - 1] = tuple(args[normal_n:])
        else:
            # Parameter binden - mit Defaults wenn weniger Args als Params.
            n_required = fn.n_required or fn.n_params
            if len(args) < n_required or len(args) > fn.n_params:
                if n_required == fn.n_params:
                    msg = f"erwartet {fn.n_params} Argument(e), erhalten {len(args)}"
                else:
                    msg = (f"erwartet {n_required}..{fn.n_params} Argument(e), "
                           f"erhalten {len(args)}")
                raise GBRuntimeError(f"{fn.name.upper()}: {msg}")
            for i, val in enumerate(args):
                locals_[i] = self._coerce(val, fn.local_types[i], f"Parameter {i+1}")
            # Fehlende Parameter mit Default-Werten fuellen (nur Literale im VM)
            for i in range(len(args), fn.n_params):
                default = fn.param_defaults[i] if fn.param_defaults else None
                if default is None:
                    raise GBRuntimeError(
                        f"{fn.name.upper()}: Parameter {i+1} hat keinen Default"
                    )
                locals_[i] = self._coerce(
                    default, fn.local_types[i], f"Default-Parameter {i+1}"
                )

        code = fn.code
        constants = fn.constants
        # Inline-Cache-Slots, parallel zu code. Lazy gefuellt von den
        # Cacheable-Ops (CALL_METHOD, LOAD_MEMBER, STORE_MEMBER auf
        # Instanzen). caches[ip-1] gehoert zur gerade ausgefuehrten Instr.
        caches = fn.caches
        stack: list = []
        ip = 0
        n = len(code)

        # Lokalisierung fuer Speed (Local-Lookups sind in CPython schneller
        # als Attribute-Lookups). Trifft jeden op pro Iteration.
        OP = Op
        push = stack.append
        globals_ = self.globals
        global_slots = self.global_slots
        classes = self.module.classes
        coerce_ = _coerce  # Modul-Level, vermeidet self._coerce-Indirektion

        # Try-Handler-Stack: list[(catch_target_ip, stack_depth_at_try_begin)]
        try_handlers: list = []

        while True:
         try:
          while ip < n:
                  op, arg = code[ip]
                  ip += 1

                  if op == OP.LOAD_CONST:
                      push(constants[arg])
                  elif op == OP.LOAD_LOCAL:
                      push(locals_[arg])
                  elif op == OP.STORE_LOCAL:
                      v = stack.pop()
                      locals_[arg] = coerce_(v, fn.local_types[arg],
                                              f"Lokale Variable [{arg}]", classes)
                  elif op == OP.DECLARE_LOCAL:
                      slot, type_name, default = arg
                      # MAP-Slots werden auto-initialisiert (default ist None vom Compiler).
                      if isinstance(type_name, str) and type_name.startswith("map:"):
                          if not isinstance(locals_[slot], _GBMap):
                              locals_[slot] = _GBMap(type_name[4:])
                      # Idempotent: nur initialisieren, wenn noch nicht da
                      elif locals_[slot] is None and default is not None:
                          locals_[slot] = default
                      # Wenn slot schon einen Wert hat (z.B. Schleifen-Iteration mit DIM
                      # innen), nichts tun -> Wert bleibt erhalten.
                  elif op == OP.LOAD_NAME:
                      name = constants[arg]
                      slot = globals_.get(name)
                      if slot is None:
                          raise GBRuntimeError(
                              f"Variable '{name}' nicht deklariert (DIM fehlt?)"
                          )
                      push(slot.value)
                  elif op == OP.STORE_NAME:
                      name = constants[arg]
                      slot = globals_.get(name)
                      if slot is None:
                          raise GBRuntimeError(
                              f"Variable '{name}' nicht deklariert (DIM fehlt?)"
                          )
                      if slot.const:
                          raise GBRuntimeError(
                              f"CONST '{name}' kann nicht ueberschrieben werden"
                          )
                      v = stack.pop()
                      slot.value = coerce_(v, slot.type,
                                            f"Zuweisung an '{name}'", classes)
                  elif op == OP.DECLARE_NAME:
                      name_idx, type_idx, default_idx = arg
                      name = constants[name_idx]
                      type_name = constants[type_idx]
                      default = constants[default_idx]
                      # MAP-Typ: leere Map allokieren (default = None)
                      if isinstance(type_name, str) and type_name.startswith("map:"):
                          default = _GBMap(type_name[4:])
                      if name in globals_:
                          existing = globals_[name]
                          if existing.const:
                              raise GBRuntimeError(
                                  f"'{name}' ist CONST und kann nicht erneut deklariert werden"
                              )
                          if existing.type != type_name:
                              raise GBRuntimeError(
                                  f"Variable '{name}' war als {existing.type.upper()} "
                                  f"deklariert, jetzt als {type_name.upper()} - Typkonflikt"
                              )
                          # idempotent
                      else:
                          globals_[name] = _Slot(type_name, default, False)
                  elif op == OP.DECLARE_CONST:
                      name_idx, type_idx = arg
                      name = constants[name_idx]
                      type_name = constants[type_idx] if type_idx is not None else None
                      value = stack.pop()
                      if type_name is None:
                          type_name = _infer_type(value)
                      else:
                          value = coerce_(value, type_name, f"CONST {name}", classes)
                      if name in globals_:
                          existing = globals_[name]
                          if (existing.const
                                  and existing.type == type_name
                                  and existing.value == value):
                              continue  # gleicher CONST nochmal -> ok
                          raise GBRuntimeError(f"'{name}' bereits deklariert")
                      globals_[name] = _Slot(type_name, value, True)

                  # --- Slot-basierte Globals (Compile-Zeit-aufgeloest) ---
                  # LOAD/STORE_GLOBAL_SLOT umgehen den Dict-Lookup auf
                  # globals_[name]. Der _Slot-Wert ist im global_slots-
                  # Array per Index direkt erreichbar. DECLARE_GLOBAL_SLOT
                  # schreibt zusaetzlich nach globals_[name], damit name-
                  # basierte Ops (INPUT_NAME, LOAD_NAME) weiterhin funktionieren.
                  elif op == OP.LOAD_GLOBAL_SLOT:
                      push(global_slots[arg].value)
                  elif op == OP.STORE_GLOBAL_SLOT:
                      slot = global_slots[arg]
                      if slot.const:
                          raise GBRuntimeError(
                              "CONST kann nicht ueberschrieben werden"
                          )
                      v = stack.pop()
                      slot.value = coerce_(
                          v, slot.type, "Zuweisung an global", classes,
                      )
                  elif op == OP.DECLARE_GLOBAL_SLOT:
                      slot_idx, name_idx, type_idx, default_idx = arg
                      name = constants[name_idx]
                      type_name = constants[type_idx]
                      default = constants[default_idx]
                      if isinstance(type_name, str) and type_name.startswith("map:"):
                          default = _GBMap(type_name[4:])
                      # Idempotent: falls schon im Dict (z.B. Schleife mit DIM
                      # im Body), bestehenden _Slot behalten und in slot-Liste
                      # spiegeln.
                      if name in globals_:
                          existing = globals_[name]
                          if existing.const:
                              raise GBRuntimeError(
                                  f"'{name}' ist CONST und kann nicht erneut deklariert werden"
                              )
                          if existing.type != type_name:
                              raise GBRuntimeError(
                                  f"Variable '{name}' war als {existing.type.upper()} "
                                  f"deklariert, jetzt als {type_name.upper()} - Typkonflikt"
                              )
                          global_slots[slot_idx] = existing
                      else:
                          slot_obj = _Slot(type_name, default, False)
                          globals_[name] = slot_obj
                          global_slots[slot_idx] = slot_obj
                  elif op == OP.DECLARE_GLOBAL_CONST_SLOT:
                      slot_idx, name_idx, type_idx = arg
                      name = constants[name_idx]
                      type_name = constants[type_idx] if type_idx is not None else None
                      value = stack.pop()
                      if type_name is None:
                          type_name = _infer_type(value)
                      else:
                          value = coerce_(value, type_name, f"CONST {name}", classes)
                      if name in globals_:
                          existing = globals_[name]
                          if (existing.const
                                  and existing.type == type_name
                                  and existing.value == value):
                              # Idempotenter Re-CONST -- slot-Liste auch hier
                              # spiegeln, falls leer (z.B. zweiter run() auf
                              # demselben Modul).
                              global_slots[slot_idx] = existing
                              continue
                          raise GBRuntimeError(f"'{name}' bereits deklariert")
                      slot_obj = _Slot(type_name, value, True)
                      globals_[name] = slot_obj
                      global_slots[slot_idx] = slot_obj

                  # --- Stack-Operationen ---
                  elif op == OP.POP:
                      stack.pop()
                  elif op == OP.DUP:
                      push(stack[-1])

                  # --- Arithmetik ---
                  # Operator-Overloading (Vec2 etc.) laeuft ueber die
                  # Modul-Registry. dispatch_binary_op wird einmal pro Frame
                  # importiert und dann lokal als _disp_op gehalten -- vermeidet
                  # einen Import pro Operation. Danach wird User-Class-
                  # Overloading via self._user_op(<op_method>, ...) probiert.
                  elif op == OP.ADD:
                      b = stack.pop()
                      a = stack.pop()
                      r = _disp_op("+", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_add__", a, b)
                      if r is not _NO_OP:
                          push(r)
                      elif isinstance(a, str) or isinstance(b, str):
                          push(_fmt(a) + _fmt(b))
                      else:
                          _require_number(a, b, "+")
                          push(a + b)
                  elif op == OP.SUB:
                      b = stack.pop()
                      a = stack.pop()
                      r = _disp_op("-", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_sub__", a, b)
                      if r is not _NO_OP:
                          push(r)
                      else:
                          _require_number(a, b, "-")
                          push(a - b)
                  elif op == OP.MUL:
                      b = stack.pop()
                      a = stack.pop()
                      r = _disp_op("*", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_mul__", a, b)
                      if r is not _NO_OP:
                          push(r)
                      elif isinstance(a, str) and isinstance(b, int) and not isinstance(b, bool):
                          push(a * b if b > 0 else "")
                      elif isinstance(b, str) and isinstance(a, int) and not isinstance(a, bool):
                          push(b * a if a > 0 else "")
                      else:
                          # Strikt Zahl * Zahl -- Bool und non-numeric werfen.
                          _require_number(a, b, "*")
                          push(a * b)
                  elif op == OP.DIV:
                      b = stack.pop()
                      a = stack.pop()
                      r = _disp_op("/", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_div__", a, b)
                      if r is not _NO_OP:
                          push(r)
                      else:
                          _require_number(a, b, "/")
                          if b == 0:
                              raise GBRuntimeError("Division durch 0")
                          if isinstance(a, int) and isinstance(b, int) and a % b == 0:
                              push(a // b)
                          else:
                              push(a / b)
                  elif op == OP.INT_DIV:
                      b = stack.pop()
                      a = stack.pop()
                      if isinstance(a, bool) or isinstance(b, bool):
                          raise TypeMismatchError("\\ erwartet INTEGER")
                      if not isinstance(a, int) or not isinstance(b, int):
                          raise TypeMismatchError("\\ erwartet INTEGER (kein FLOAT)")
                      if b == 0:
                          raise GBRuntimeError("Integer-Division durch 0")
                      q, r = divmod(a, b)
                      if r != 0 and (a < 0) != (b < 0):
                          q += 1
                      push(q)
                  elif op == OP.MOD:
                      b = stack.pop()
                      a = stack.pop()
                      r = _disp_op("mod", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_mod__", a, b)
                      if r is not _NO_OP:
                          push(r)
                      else:
                          _require_number(a, b, "MOD")
                          if b == 0:
                              raise GBRuntimeError("MOD durch 0")
                          push(a % b)
                  elif op == OP.POW:
                      b = stack.pop()
                      a = stack.pop()
                      r = _disp_op("^", a, b)
                      if r is not _NO_OP:
                          push(r)
                      else:
                          _require_number(a, b, "^")
                          push(a ** b)
                  elif op == OP.NEG:
                      v = stack.pop()
                      if isinstance(v, bool) or not isinstance(v, (int, float)):
                          raise TypeMismatchError("Unaeres '-' erwartet Zahl")
                      push(-v)

                  # --- Vergleich / Logik ---
                  # Reihenfolge wie im Tree-Walker (interpreter._eval_BinaryOp):
                  # 1. Modul-Operator-Dispatch (_disp_op), dann
                  # 2. User-Klassen-Op (`__op_eq__`/`__op_lt__` etc.), dann
                  # 3. Python-`==`/`<` (was via `__eq__` der Python-Klasse
                  #    trotzdem User-Equality liefern kann -- z.B. Vec2 hat das
                  #    als Python-Hook).
                  elif op == OP.EQ:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("=", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_eq__", a, b)
                      push(a == b if r is _NO_OP else r)
                  elif op == OP.NEQ:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("<>", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_ne__", a, b)
                      push(a != b if r is _NO_OP else r)
                  elif op == OP.LT:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("<", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_lt__", a, b)
                      push(a < b if r is _NO_OP else r)
                  elif op == OP.GT:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op(">", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_gt__", a, b)
                      push(a > b if r is _NO_OP else r)
                  elif op == OP.LEQ:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("<=", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_le__", a, b)
                      push(a <= b if r is _NO_OP else r)
                  elif op == OP.GEQ:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op(">=", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_ge__", a, b)
                      push(a >= b if r is _NO_OP else r)
                  elif op == OP.NOT:
                      push(not _truthy(stack.pop()))

                  # --- Bitwise (strikt INTEGER) ---
                  elif op == OP.BAND:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "BAND")
                      push(a & b)
                  elif op == OP.BOR:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "BOR")
                      push(a | b)
                  elif op == OP.BXOR:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "BXOR")
                      push(a ^ b)
                  elif op == OP.SHL:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "SHL")
                      if b < 0:
                          raise GBRuntimeError("SHL: Shift-Anzahl darf nicht negativ sein")
                      push(a << b)
                  elif op == OP.SHR:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "SHR")
                      if b < 0:
                          raise GBRuntimeError("SHR: Shift-Anzahl darf nicht negativ sein")
                      push(a >> b)
                  elif op == OP.BNOT:
                      v = stack.pop()
                      if isinstance(v, bool) or not isinstance(v, int):
                          raise TypeMismatchError("BNOT erwartet INTEGER")
                      push(~v)

                  # --- Spezialisierte Numeric-Numeric Ops (vom Compiler emittiert,
                  # wenn beide Operanden statisch als INTEGER/FLOAT bekannt sind).
                  # Kein Modul-Operator-Dispatch, kein User-Class-Dispatch, keine
                  # isinstance-Cascade -- direkt `pop b; pop a; push a OP b`.
                  # Bit-identisch zu OP.ADD/SUB/MUL/DIV etc. fuer numerische
                  # Operanden. Bool wird in der Inferenz nie als numerisch
                  # markiert, also sehen wir hier garantiert int oder float.
                  elif op == OP.ADD_NN:
                      b = stack.pop(); a = stack.pop(); push(a + b)
                  elif op == OP.SUB_NN:
                      b = stack.pop(); a = stack.pop(); push(a - b)
                  elif op == OP.MUL_NN:
                      b = stack.pop(); a = stack.pop(); push(a * b)
                  elif op == OP.DIV_NN:
                      b = stack.pop(); a = stack.pop()
                      if b == 0:
                          raise GBRuntimeError("Division durch 0")
                      # Identisch zu OP.DIV: Int/Int mit Rest=0 -> Int, sonst Float.
                      if isinstance(a, int) and isinstance(b, int) and a % b == 0:
                          push(a // b)
                      else:
                          push(a / b)
                  elif op == OP.LT_NN:
                      b = stack.pop(); a = stack.pop(); push(a < b)
                  elif op == OP.GT_NN:
                      b = stack.pop(); a = stack.pop(); push(a > b)
                  elif op == OP.LEQ_NN:
                      b = stack.pop(); a = stack.pop(); push(a <= b)
                  elif op == OP.GEQ_NN:
                      b = stack.pop(); a = stack.pop(); push(a >= b)
                  elif op == OP.EQ_NN:
                      b = stack.pop(); a = stack.pop(); push(a == b)
                  elif op == OP.NEQ_NN:
                      b = stack.pop(); a = stack.pop(); push(a != b)
                  elif op == OP.NEG_N:
                      push(-stack.pop())

                  # --- Tupel ---
                  # Wichtig: NICHT die lokale Variable `n` benutzen --
                  # die haelt die Code-Laenge (siehe `n = len(code)` oben).
                  elif op == OP.BUILD_TUPLE:
                      tlen = arg
                      if tlen == 0:
                          push(())
                      else:
                          t = tuple(stack[-tlen:])
                          del stack[-tlen:]
                          push(t)
                  elif op == OP.UNPACK_TUPLE:
                      tlen = arg
                      t = stack.pop()
                      if not isinstance(t, tuple):
                          raise TypeMismatchError(
                              f"UNPACK_TUPLE: Erwartet TUPLE, erhalten {_type_of(t)}"
                          )
                      if len(t) != tlen:
                          raise GBRuntimeError(
                              f"Tupel-Destructuring: {tlen} Ziele, aber Tupel "
                              f"hat {len(t)} Element(e)"
                          )
                      for v in reversed(t):
                          push(v)

                  # --- Kontrollfluss ---
                  elif op == OP.JUMP:
                      ip = arg
                  elif op == OP.JUMP_IF_FALSE:
                      v = stack.pop()
                      if not _truthy(v):
                          ip = arg
                  elif op == OP.JUMP_IF_TRUE:
                      v = stack.pop()
                      if _truthy(v):
                          ip = arg

                  # --- Aufrufe ---
                  elif op == OP.CALL_USER:
                      fn_name, argc = arg
                      callee = self.module.functions.get(fn_name)
                      if callee is None:
                          raise GBRuntimeError(f"Unbekannte Funktion: {fn_name.upper()}")
                      if argc:
                          call_args = stack[-argc:]
                          del stack[-argc:]
                      else:
                          call_args = []
                      ret = self._exec(callee, call_args)
                      if not callee.is_sub:
                          push(ret)
                      else:
                          # SUB: trotzdem etwas auf den Stack legen, falls als Expr verwendet
                          push(None)
                  elif op == OP.CALL_BUILTIN:
                      fn_name, argc = arg
                      if argc:
                          bargs = stack[-argc:]
                          del stack[-argc:]
                      else:
                          bargs = []
                      gh = GRAPHICS_BUILTINS.get(fn_name)
                      if gh is not None:
                          push(gh(self._get_graphics(), bargs))
                      else:
                          bf = BUILTINS.get(fn_name)
                          if bf is None:
                              raise GBRuntimeError(
                                  f"Unbekannte Funktion: {fn_name.upper()}"
                              )
                          push(bf(bargs))
                  elif op == OP.LOAD_FUNCREF:
                      ref_name = constants[arg]
                      if ref_name not in self.module.functions:
                          raise GBRuntimeError(
                              f"FUNCREF: Funktion '{ref_name}' existiert nicht"
                          )
                      push(_FuncRef(ref_name))
                  elif op == OP.BUILD_TUPLE_DYN:
                      # Sammle alle Werte oberhalb des COMP_MARKER zu einem
                      # Tupel. Marker wird gepoppt. Stack vorher:
                      # [..., MARKER, v0, v1, ..., vN] -> [..., (v0..vN)].
                      from .bytecode import COMP_MARKER
                      idx = len(stack) - 1
                      while idx >= 0 and stack[idx] is not COMP_MARKER:
                          idx -= 1
                      if idx < 0:
                          raise GBRuntimeError(
                              "BUILD_TUPLE_DYN: kein COMP_MARKER auf dem Stack"
                          )
                      values = tuple(stack[idx + 1:])
                      del stack[idx:]
                      push(values)
                  elif op == OP.IN_OP:
                      hay = stack.pop()
                      needle = stack.pop()
                      push(_eval_in(needle, hay))
                  elif op == OP.SLICE:
                      has_lo, has_hi = arg
                      hi_val = stack.pop() if has_hi else None
                      lo_val = stack.pop() if has_lo else None
                      target = stack.pop()
                      push(_apply_slice(target, lo_val, hi_val))
                  elif op == OP.CALL_VALUE:
                      argc = arg
                      if argc:
                          call_args = stack[-argc:]
                          del stack[-argc:]
                      else:
                          call_args = []
                      callee = stack.pop()
                      if not isinstance(callee, _FuncRef):
                          raise GBRuntimeError(
                              f"Wert vom Typ {_type_of(callee)} ist nicht aufrufbar"
                          )
                      tgt = self.module.functions.get(callee.name)
                      if tgt is None:
                          raise GBRuntimeError(
                              f"FUNCREF: Funktion '{callee.name}' existiert nicht (mehr)"
                          )
                      ret = self._exec(tgt, call_args)
                      if not tgt.is_sub:
                          push(ret)
                      else:
                          push(None)
                  elif op == OP.CALL_METHOD:
                      method_name, argc = arg
                      if argc:
                          margs = stack[-argc:]
                          del stack[-argc:]
                      else:
                          margs = []
                      obj = stack.pop()
                      if obj is None:
                          raise GBRuntimeError(
                              f"Methodenaufruf '.{method_name}' bei NIL-Referenz"
                          )
                      # Inline-Cache: monomorphic, bei _Instance mit gleichem cls
                      # ueberspringen wir _container_kind, isinstance und
                      # _resolve_method komplett.
                      cache = caches[ip - 1]
                      if (cache is not None
                              and type(obj) is _Instance
                              and obj.cls is cache[0]):
                          method = cache[1]
                          ret = self._exec(method, margs, self_obj=obj)
                          if not method.is_sub:
                              push(ret)
                          else:
                              push(None)
                          continue
                      # Slow path: Container-/Instance-Dispatch
                      kind = _container_kind(obj)
                      if kind:
                          builtin_name = _CONTAINER_METHODS.get(
                              (kind, method_name.lower()))
                          if builtin_name is None:
                              raise GBRuntimeError(
                                  f"{kind.upper()} hat keine Methode '{method_name}'"
                              )
                          bf = BUILTINS.get(builtin_name)
                          push(bf([obj] + margs))
                          continue
                      if not isinstance(obj, _Instance):
                          raise GBRuntimeError(
                              f"Methodenaufruf '.{method_name}' bei nicht-Objekt ({_type_of(obj)})"
                          )
                      method, _ = self._resolve_method(obj.cls, method_name)
                      if method is None:
                          raise GBRuntimeError(
                              f"Methode '{method_name}' existiert nicht in {obj.cls.name}"
                          )
                      caches[ip - 1] = [obj.cls, method]
                      ret = self._exec(method, margs, self_obj=obj)
                      if not method.is_sub:
                          push(ret)
                      else:
                          push(None)
                  elif op == OP.NEW_INSTANCE:
                      class_name, argc, has_init_args = arg
                      cls = self.module.classes.get(class_name)
                      if cls is None:
                          raise GBRuntimeError(f"Klasse '{class_name}' nicht gefunden")
                      if has_init_args:
                          if argc:
                              init_args = stack[-argc:]
                              del stack[-argc:]
                          else:
                              init_args = []
                          inst = self._allocate_instance(cls)
                          init, _ = self._resolve_method(cls, "init")
                          if init is None:
                              if init_args:
                                  raise GBRuntimeError(
                                      f"Klasse {cls.name} hat keine SUB Init - "
                                      f"Argumente bei NEW nicht moeglich"
                                  )
                          else:
                              self._exec(init, init_args, self_obj=inst)
                          push(inst)
                      else:
                          push(self._allocate_instance(cls))
                  elif op == OP.LOAD_SELF:
                      if self_obj is None:
                          raise GBRuntimeError(
                              "LOAD_SELF (Self) ausserhalb Methodenkontext"
                          )
                      push(self_obj)
                  elif op == OP.LOAD_FIELD:
                      name = constants[arg]
                      if self_obj is None:
                          raise GBRuntimeError(
                              f"LOAD_FIELD '{name}' ausserhalb Methodenkontext"
                          )
                      push(self_obj.fields[name]["value"])
                  elif op == OP.STORE_FIELD:
                      name = constants[arg]
                      if self_obj is None:
                          raise GBRuntimeError(
                              f"STORE_FIELD '{name}' ausserhalb Methodenkontext"
                          )
                      slot = self_obj.fields[name]
                      v = stack.pop()
                      slot["value"] = coerce_(
                          v, slot["type"], f"Zuweisung an {self_obj.cls.name}.{name}",
                          classes,
                      )
                  elif op == OP.LOAD_MEMBER:
                      name = constants[arg]
                      obj = stack.pop()
                      if obj is None:
                          raise GBRuntimeError(f"Zugriff auf '.{name}' bei NIL-Referenz")
                      # Inline-Cache: bei _Instance mit gleichem cls direkt
                      # auf Field oder Property-Getter dispatchen.
                      cache = caches[ip - 1]
                      if (cache is not None
                              and type(obj) is _Instance
                              and obj.cls is cache[0]):
                          getter = cache[1]
                          if getter is None:
                              push(obj.fields[name]["value"])
                          else:
                              push(self._exec(getter, [], self_obj=obj))
                          continue
                      if isinstance(obj, _EnumNamespace):
                          val = obj.get(name)
                          if val is None:
                              avail = ", ".join(obj.members.keys())
                              raise GBRuntimeError(
                                  f"ENUM {obj.name} hat keinen Member '{name}' "
                                  f"(verfuegbar: {avail})"
                              )
                          push(val)
                          continue
                      if isinstance(obj, _ClassStaticNamespace):
                          val = obj.get(name)
                          if val is None:
                              avail = ", ".join(obj.members.keys()) or "<keine>"
                              raise GBRuntimeError(
                                  f"CLASS {obj.name} hat keinen STATIC-Member '{name}' "
                                  f"(verfuegbar: {avail})"
                              )
                          push(val)
                          continue
                      if not isinstance(obj, _Instance):
                          raise GBRuntimeError(
                              f"Zugriff auf '.{name}' bei nicht-Objekt ({_type_of(obj)})"
                          )
                      # Property-Getter-Dispatch
                      if _is_property(obj.cls, name):
                          getter, _ = self._resolve_method(
                              obj.cls, f"__get_{name.lower()}")
                          if getter is None:
                              raise GBRuntimeError(
                                  f"Property '{name}' in {obj.cls.name} hat "
                                  f"keinen Getter (nur SET deklariert)"
                              )
                          caches[ip - 1] = [obj.cls, getter]
                          ret = self._exec(getter, [], self_obj=obj)
                          push(ret)
                          continue
                      if name not in obj.fields:
                          raise GBRuntimeError(
                              f"Feld '{name}' existiert nicht in {obj.cls.name}"
                          )
                      caches[ip - 1] = [obj.cls, None]
                      push(obj.fields[name]["value"])
                  elif op == OP.STORE_MEMBER:
                      name = constants[arg]
                      v = stack.pop()
                      obj = stack.pop()
                      if obj is None:
                          raise GBRuntimeError(f"Zuweisung an '.{name}' bei NIL-Referenz")
                      # Inline-Cache: bei _Instance mit gleichem cls direkt
                      # auf Field-Coerce oder Property-Setter dispatchen.
                      cache = caches[ip - 1]
                      if (cache is not None
                              and type(obj) is _Instance
                              and obj.cls is cache[0]):
                          setter = cache[1]
                          if setter is None:
                              slot = obj.fields[name]
                              slot["value"] = coerce_(
                                  v, slot["type"],
                                  f"Zuweisung an {obj.cls.name}.{name}",
                                  classes,
                              )
                          else:
                              self._exec(setter, [v], self_obj=obj)
                          continue
                      if not isinstance(obj, _Instance):
                          raise GBRuntimeError(
                              f"Zuweisung an '.{name}' bei nicht-Objekt ({_type_of(obj)})"
                          )
                      # Property-Setter-Dispatch
                      if _is_property(obj.cls, name):
                          setter, _ = self._resolve_method(
                              obj.cls, f"__set_{name.lower()}")
                          if setter is None:
                              raise GBRuntimeError(
                                  f"Property '{name}' in {obj.cls.name} hat "
                                  f"keinen Setter (nur GET -- read-only)"
                              )
                          caches[ip - 1] = [obj.cls, setter]
                          self._exec(setter, [v], self_obj=obj)
                          continue
                      if name not in obj.fields:
                          raise GBRuntimeError(
                              f"Feld '{name}' existiert nicht in {obj.cls.name}"
                          )
                      caches[ip - 1] = [obj.cls, None]
                      slot = obj.fields[name]
                      slot["value"] = coerce_(
                          v, slot["type"], f"Zuweisung an {obj.cls.name}.{name}",
                          classes,
                      )
                  elif op == OP.DECLARE_STRUCT_NAME:
                      name_idx, class_name = arg
                      name = constants[name_idx]
                      cls = self.module.classes.get(class_name)
                      if cls is None:
                          raise GBRuntimeError(f"STRUCT '{class_name}' nicht gefunden")
                      if name not in self.globals:
                          self.globals[name] = _Slot(
                              class_name, self._allocate_instance(cls), False,
                          )
                  elif op == OP.DECLARE_STRUCT_LOCAL:
                      slot, class_name = arg
                      if locals_[slot] is None:
                          cls = self.module.classes.get(class_name)
                          if cls is None:
                              raise GBRuntimeError(f"STRUCT '{class_name}' nicht gefunden")
                          locals_[slot] = self._allocate_instance(cls)
                  elif op == OP.LOAD_INDEX:
                      num_dims = arg
                      idx_vals = stack[-num_dims:] if num_dims else []
                      if num_dims:
                          del stack[-num_dims:]
                      arr = stack.pop()
                      # Fast path: 1D-_GBArray, ein int-Index in Bounds. Spart
                      # die isinstance-Cascade + Index-Validierungs-Loop. Alle
                      # Edge-Cases fallen in den generischen Pfad (gleiche Fehler).
                      if num_dims == 1 and type(arr) is _GBArray and len(arr.dims) == 1:
                          ix = idx_vals[0]
                          if type(ix) is int and 0 <= ix < arr.dims[0]:
                              push(arr.values[ix])
                              continue
                      if arr is None:
                          raise GBRuntimeError("Index-Zugriff auf NIL")
                      # String-Index: einzelner Int -> 1-Char-String.
                      if isinstance(arr, str):
                          if len(idx_vals) != 1:
                              raise GBRuntimeError("String-Index braucht genau einen Wert")
                          ix = idx_vals[0]
                          if isinstance(ix, bool) or not isinstance(ix, int):
                              raise TypeMismatchError(
                                  f"String-Index muss INTEGER sein, erhalten {_type_of(ix)}"
                              )
                          if ix < 0 or ix >= len(arr):
                              raise GBRuntimeError(
                                  f"String-Index {ix} ausserhalb des Bereichs (Laenge {len(arr)})"
                              )
                          push(arr[ix])
                          continue
                      # Tupel-Index: einzelner Integer-Index.
                      if isinstance(arr, tuple):
                          if len(idx_vals) != 1:
                              raise GBRuntimeError("Tupel-Index braucht genau einen Wert")
                          ix = idx_vals[0]
                          if isinstance(ix, bool) or not isinstance(ix, int):
                              raise TypeMismatchError(
                                  f"Tupel-Index muss INTEGER sein, erhalten {_type_of(ix)}"
                              )
                          if ix < 0 or ix >= len(arr):
                              raise GBRuntimeError(
                                  f"Tupel-Index {ix} ausserhalb des Bereichs (Laenge {len(arr)})"
                              )
                          push(arr[ix])
                          continue
                      if not isinstance(arr, _GBArray):
                          raise GBRuntimeError(
                              f"Index-Zugriff auf Nicht-Array ({_type_of(arr)})"
                          )
                      for ix in idx_vals:
                          if isinstance(ix, bool) or not isinstance(ix, int):
                              raise TypeMismatchError(
                                  f"Array-Index muss INTEGER sein, erhalten {_type_of(ix)}"
                              )
                      # Fast path: cdef-class get_at fuehrt Bounds-Check und
                      # typed-buffer-Zugriff in C aus.
                      push(arr.get_at(idx_vals))
                  elif op == OP.STORE_INDEX:
                      num_dims = arg
                      v = stack.pop()
                      idx_vals = stack[-num_dims:] if num_dims else []
                      if num_dims:
                          del stack[-num_dims:]
                      arr = stack.pop()
                      # Fast path: 1D-_GBArray, ein int-Index in Bounds.
                      if num_dims == 1 and type(arr) is _GBArray and len(arr.dims) == 1:
                          ix = idx_vals[0]
                          if type(ix) is int and 0 <= ix < arr.dims[0]:
                              arr.values[ix] = coerce_(
                                  v, arr.element_type,
                                  f"Array-Element [{ix}]", classes)
                              continue
                      if arr is None:
                          raise GBRuntimeError("Index-Zuweisung an NIL")
                      if not isinstance(arr, _GBArray):
                          raise GBRuntimeError(
                              f"Index-Zuweisung an Nicht-Array ({_type_of(arr)})"
                          )
                      for ix in idx_vals:
                          if isinstance(ix, bool) or not isinstance(ix, int):
                              raise TypeMismatchError(
                                  f"Array-Index muss INTEGER sein, erhalten {_type_of(ix)}"
                              )
                      arr.set_at(
                          idx_vals,
                          coerce_(
                              v, arr.element_type,
                              f"Array-Element [{','.join(str(i) for i in idx_vals)}]",
                              classes,
                          ),
                      )
                  elif op == OP.DECLARE_ARRAY_NAME:
                      name_idx, elem_type, num_dims = arg
                      name = constants[name_idx]
                      dims = stack[-num_dims:] if num_dims else []
                      if num_dims:
                          del stack[-num_dims:]
                      for d in dims:
                          if isinstance(d, bool) or not isinstance(d, int) or d < 0:
                              raise GBRuntimeError("Array-Groesse muss INTEGER >= 0 sein")
                      arr = _GBArray(
                          elem_type, dims,
                          lambda t=elem_type: self._element_default(t),
                      )
                      self.globals[name] = _Slot(
                          f"array:{elem_type}", arr, False,
                      )
                  elif op == OP.DECLARE_ARRAY_LOCAL:
                      slot, elem_type, num_dims = arg
                      dims = stack[-num_dims:] if num_dims else []
                      if num_dims:
                          del stack[-num_dims:]
                      for d in dims:
                          if isinstance(d, bool) or not isinstance(d, int) or d < 0:
                              raise GBRuntimeError("Array-Groesse muss INTEGER >= 0 sein")
                      arr = _GBArray(
                          elem_type, dims,
                          lambda t=elem_type: self._element_default(t),
                      )
                      locals_[slot] = arr
                  elif op == OP.RETURN:
                      v = stack.pop()
                      if fn.is_sub:
                          raise GBRuntimeError(
                              f"SUB '{fn.name}' darf RETURN nicht mit Wert verwenden"
                          )
                      return coerce_(v, fn.return_type, f"RETURN aus {fn.name.upper()}", classes)
                  elif op == OP.RETURN_VOID:
                      if not fn.is_sub:
                          raise GBRuntimeError(
                              f"FUNCTION '{fn.name}' muss einen Wert mit RETURN zurueckgeben"
                          )
                      return None

                  # --- I/O ---
                  elif op == OP.PRINT:
                      count = arg
                      if count == 0:
                          print()
                      else:
                          items = stack[-count:]
                          del stack[-count:]
                          print(" ".join(_fmt(x) for x in items))
                  elif op == OP.INPUT_NAME:
                      name_idx, has_prompt = arg
                      prompt = stack.pop() if has_prompt else None
                      self._do_input(globals_.get(constants[name_idx]),
                                     constants[name_idx], prompt)
                  elif op == OP.INPUT_LOCAL:
                      slot, has_prompt = arg
                      prompt = stack.pop() if has_prompt else None
                      slot_info = _Slot(fn.local_types[slot], locals_[slot])
                      self._do_input(slot_info, f"slot[{slot}]", prompt)
                      locals_[slot] = slot_info.value

                  # --- Exceptions ---
                  elif op == OP.TRY_BEGIN:
                      try_handlers.append((arg, len(stack)))
                  elif op == OP.TRY_END:
                      try_handlers.pop()
                  elif op == OP.THROW:
                      v = stack.pop()
                      msg = v if isinstance(v, str) else _fmt(v)
                      raise _GBThrow(msg)

                  # --- DATA / READ / RESTORE ---
                  elif op == OP.PUSH_DATA:
                      if self.data_ptr >= len(self.module.data):
                          raise GBRuntimeError(
                              "READ: keine DATA-Werte mehr "
                              "(benutze RESTORE zum Reset)"
                          )
                      push(self.module.data[self.data_ptr])
                      self.data_ptr += 1
                  elif op == OP.RESET_DATA_PTR:
                      self.data_ptr = 0

                  # --- Ende ---
                  elif op == OP.HALT:
                      return None
                  else:
                      raise GBRuntimeError(f"Unbekannter Opcode: {op}")

          return None
         except (_GBThrow, GBRuntimeError, TypeMismatchError) as exc:
          if not try_handlers:
           raise
          target, depth = try_handlers.pop()
          del stack[depth:]
          if isinstance(exc, _GBThrow):
           msg = exc.value
          else:
           msg = exc.message if hasattr(exc, "message") else str(exc)
          stack.append(msg)
          ip = target

    def _do_input(self, slot, name: str, prompt):
        if slot is None:
            raise GBRuntimeError(f"Variable '{name}' nicht deklariert")
        prompt_str = ""
        if prompt is not None:
            prompt_str = prompt or ""
        if not prompt_str:
            prompt_str = "? "
        elif not prompt_str.endswith(" "):
            prompt_str = prompt_str + " "
        # Prompt vor input() explizit flushen, damit er in PIPE-Subprocess-
        # Setups (Editor-Konsole) sofort sichtbar wird.
        import sys as _sys
        try:
            _sys.stdout.write(prompt_str)
            _sys.stdout.flush()
        except Exception:
            pass
        try:
            raw = input()
        except EOFError:
            raw = ""
        try:
            t = slot.type
            if t == "integer":
                v = int(raw.strip())
            elif t == "float":
                v = float(raw.strip())
            elif t == "string":
                v = raw
            elif t == "boolean":
                v = raw.strip().lower() in ("true", "wahr", "yes", "ja", "1")
            else:
                raise GBRuntimeError(f"Unbekannter Typ: {t}")
        except ValueError:
            raise GBRuntimeError(
                f"Eingabe '{raw}' passt nicht zu {slot.type.upper()}"
            )
        slot.value = v


class _Halt(Exception):
    pass


def execute(module: Module) -> int:
    """Convenience: kompletten Lauf, Exit-Code-Konvention."""
    vm = VM()
    try:
        vm.run(module)
        return 0
    except GameBasicError:
        raise
