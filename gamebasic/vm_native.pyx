# cython: language_level=3
# cython: boundscheck=False
# cython: wraparound=False
# cython: initializedcheck=False
# cython: cdivision=True
"""Native VM (Cython) - identische Semantik wie vm.py.

Volle Phase-3b/3c-Unterstuetzung: OOP, Arrays, Structs, Strings, Files, Grafik.
"""
import math

from .errors import GameBasicError, GBRuntimeError, TypeMismatchError
from .interpreter import (  # type: ignore
    BUILTINS, GRAPHICS_BUILTINS,
    _Instance, _GBArray, _Image, _Sound, _SpriteAtlas, _GBFile, _GBMap,
    _GBThrow, _EnumNamespace, _ClassStaticNamespace, _FuncRef,
    infer_type as _infer_type_canonical,
)
from .modules import dispatch_binary_op as _disp_op, NO_OP_MATCH as _NO_OP


# Opcodes als cdef int (parallel zu bytecode.Op)
cdef int OP_LOAD_CONST    = 1
cdef int OP_POP           = 2
cdef int OP_DUP           = 3
cdef int OP_LOAD_NAME     = 10
cdef int OP_STORE_NAME    = 11
cdef int OP_DECLARE_NAME  = 12
cdef int OP_DECLARE_CONST = 13
cdef int OP_LOAD_LOCAL    = 14
cdef int OP_STORE_LOCAL   = 15
cdef int OP_DECLARE_LOCAL = 16
cdef int OP_ADD           = 20
cdef int OP_SUB           = 21
cdef int OP_MUL           = 22
cdef int OP_DIV           = 23
cdef int OP_MOD           = 24
cdef int OP_POW           = 25
cdef int OP_NEG           = 26
cdef int OP_INT_DIV       = 27
cdef int OP_EQ            = 30
cdef int OP_NEQ           = 31
cdef int OP_LT            = 32
cdef int OP_GT            = 33
cdef int OP_LEQ           = 34
cdef int OP_GEQ           = 35
cdef int OP_NOT           = 36
cdef int OP_JUMP          = 40
cdef int OP_JUMP_IF_FALSE = 41
cdef int OP_JUMP_IF_TRUE  = 42
cdef int OP_CALL_USER     = 50
cdef int OP_CALL_BUILTIN  = 51
cdef int OP_CALL_METHOD   = 52
cdef int OP_RETURN        = 60
cdef int OP_RETURN_VOID   = 61
cdef int OP_BAND          = 62
cdef int OP_BOR           = 63
cdef int OP_BXOR          = 64
cdef int OP_SHL           = 65
cdef int OP_SHR           = 66
cdef int OP_BNOT          = 67
cdef int OP_BUILD_TUPLE   = 68
cdef int OP_UNPACK_TUPLE  = 69
cdef int OP_LOAD_FUNCREF  = 53
cdef int OP_CALL_VALUE    = 54
cdef int OP_SLICE         = 55
cdef int OP_IN_OP         = 56
cdef int OP_BUILD_TUPLE_DYN = 57
cdef int OP_PRINT         = 70
cdef int OP_INPUT_NAME    = 71
cdef int OP_INPUT_LOCAL   = 72
cdef int OP_NEW_INSTANCE  = 80
cdef int OP_LOAD_FIELD    = 81
cdef int OP_STORE_FIELD   = 82
cdef int OP_LOAD_MEMBER   = 83
cdef int OP_STORE_MEMBER  = 84
cdef int OP_DECLARE_STRUCT_NAME  = 86
cdef int OP_DECLARE_STRUCT_LOCAL = 87
cdef int OP_LOAD_SELF     = 88
cdef int OP_LOAD_INDEX    = 90
cdef int OP_STORE_INDEX   = 91
cdef int OP_DECLARE_ARRAY_NAME  = 92
cdef int OP_DECLARE_ARRAY_LOCAL = 93
cdef int OP_TRY_BEGIN     = 95
cdef int OP_TRY_END       = 96
cdef int OP_THROW         = 97
cdef int OP_HALT          = 99
cdef int OP_PUSH_DATA      = 75
cdef int OP_RESET_DATA_PTR = 76

# Spezialisierte Numeric-Numeric Ops (parallel zu bytecode.Op.ADD_NN etc.)
cdef int OP_ADD_NN        = 100
cdef int OP_SUB_NN        = 101
cdef int OP_MUL_NN        = 102
cdef int OP_DIV_NN        = 103
cdef int OP_LT_NN         = 104
cdef int OP_GT_NN         = 105
cdef int OP_LEQ_NN        = 106
cdef int OP_GEQ_NN        = 107
cdef int OP_EQ_NN         = 108
cdef int OP_NEQ_NN        = 109
cdef int OP_NEG_N         = 110

# Slot-basierte Globals
cdef int OP_LOAD_GLOBAL_SLOT          = 111
cdef int OP_STORE_GLOBAL_SLOT         = 112
cdef int OP_DECLARE_GLOBAL_SLOT       = 113
cdef int OP_DECLARE_GLOBAL_CONST_SLOT = 114


_TYPE_DEFAULTS = {
    "integer": 0,
    "float":   0.0,
    "string":  "",
    "boolean": False,
    "image":   None,
    "sound":   None,
    "sprite_atlas": None,
    "file":    None,
    "tuple":   (),
    "funcref": None,
}


def _is_property_native(cls, name: str) -> bool:
    """Property-Lookup entlang der Vererbung. Identisch zu vm._is_property."""
    target = name.lower()
    cur = cls
    while cur is not None:
        if target in cur.properties:
            return True
        cur = cur.parent
    return False


def _eval_in_native(needle, haystack):
    """`needle IN haystack` -- identisch zu vm.py._eval_in."""
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


cdef str _container_kind_native(value):
    if isinstance(value, str):
        return "string"
    if isinstance(value, tuple):
        return "tuple"
    if isinstance(value, _GBArray):
        return "array"
    if isinstance(value, _GBMap):
        return "map"
    return ""


# Method-Dispatch-Tabelle aus interpreter.py wieder-importieren.
from .interpreter import CONTAINER_METHODS as _CONTAINER_METHODS


def _slice_dispatch(stack, arg):
    """Slice-Dispatch in einer pure-Python-Helper-Funktion. Cython haette
    hier inline Typ-Inferenz-Probleme mit den optionalen pop()-Calls und
    der Bool-Tupel-Auswertung."""
    has_lo, has_hi = arg
    if has_hi:
        hi_val = stack.pop()
    else:
        hi_val = None
    if has_lo:
        lo_val = stack.pop()
    else:
        lo_val = None
    target = stack.pop()
    stack.append(_apply_slice_native(target, lo_val, hi_val))


def _apply_slice_native(target, lo_val, hi_val):
    """Slice-Operation auf String oder 1D-Array. Identisch zu vm.py."""
    cdef int n
    cdef int lo, hi, i
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
    # Type-Check vor der int-Konvertierung -- Bool wird abgelehnt.
    for v, label in ((lo_val, "lo"), (hi_val, "hi")):
        if v is None:
            continue
        if isinstance(v, bool) or not isinstance(v, int):
            raise TypeMismatchError(
                f"Slice-Index ({label}) muss INTEGER sein, erhalten {_type_of(v)}"
            )
    lo = 0 if lo_val is None else lo_val
    hi = n if hi_val is None else hi_val
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
    for i in range(len(sub)):
        result.values[i] = sub[i]
    return result


cdef str _type_of(value):
    if value is None:
        return "NIL"
    if isinstance(value, tuple):
        return f"TUPLE({len(value)})"
    if isinstance(value, _FuncRef):
        return f"FUNCREF<{value.name}>"
    from gamebasic.modules.vec2 import _Vec2 as _V2
    if isinstance(value, _V2):
        return "VEC2"
    if isinstance(value, bool):
        return "BOOLEAN"
    if isinstance(value, int):
        return "INTEGER"
    if isinstance(value, float):
        return "FLOAT"
    if isinstance(value, str):
        return "STRING"
    if isinstance(value, _Instance):
        return value.cls.name.upper()
    if isinstance(value, _GBArray):
        return f"ARRAY OF {value.element_type.upper()}"
    if isinstance(value, _Image):
        return "IMAGE"
    if isinstance(value, _Sound):
        return "SOUND"
    if isinstance(value, _SpriteAtlas):
        return "SPRITE_ATLAS"
    if isinstance(value, _GBFile):
        return "FILE"
    if isinstance(value, _GBMap):
        return f"MAP OF {value.value_type.upper()}"
    return type(value).__name__.upper()


cdef bint _is_subclass_of(child, parent):
    cur = child
    while cur is not None:
        if cur is parent:
            return True
        cur = cur.parent
    return False


cdef _require_number(a, b, str op):
    """Zahlen-Pruefung fuer SUB/DIV/MOD/POW. Bool und Nicht-Zahl werfen --
    deckungsgleich mit Interpreter._require_number und vm.py."""
    cdef object v
    for v in (a, b):
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise TypeMismatchError(f"Operator '{op}' erwartet Zahlen, erhalten {_type_of(v)}")


cdef _require_int_pair(a, b, str op):
    """Strikte INTEGER-Pruefung fuer Bitwise-Operatoren. Bool wird abgelehnt --
    deckungsgleich mit vm.py._require_int_pair und Interpreter._require_int_pair
    (inkl. 'erhalten <typ>'-Suffix)."""
    cdef object v
    for v in (a, b):
        if isinstance(v, bool) or not isinstance(v, int):
            raise TypeMismatchError(f"{op} erwartet INTEGER, erhalten {_type_of(v)}")


cdef _coerce(value, str target, str ctx, dict classes):
    if target == "integer":
        if isinstance(value, bool):
            raise TypeMismatchError(f"{ctx}: Erwartet INTEGER, erhalten BOOLEAN")
        if isinstance(value, int):
            return value
        if isinstance(value, float):
            if not (<double>value).is_integer():
                raise TypeMismatchError(
                    f"{ctx}: FLOAT {value} kann nicht ohne Verlust nach INTEGER (nutze INT())"
                )
            return int(value)
        raise TypeMismatchError(f"{ctx}: Erwartet INTEGER, erhalten {_type_of(value)}")
    if target == "float":
        if isinstance(value, bool):
            raise TypeMismatchError(f"{ctx}: Erwartet FLOAT, erhalten BOOLEAN")
        if isinstance(value, (int, float)):
            return float(value)
        raise TypeMismatchError(f"{ctx}: Erwartet FLOAT, erhalten {_type_of(value)}")
    if target == "string":
        if isinstance(value, str):
            return value
        raise TypeMismatchError(f"{ctx}: Erwartet STRING, erhalten {_type_of(value)}")
    if target == "boolean":
        if isinstance(value, bool):
            return value
        raise TypeMismatchError(f"{ctx}: Erwartet BOOLEAN, erhalten {_type_of(value)}")
    if target == "tuple":
        if isinstance(value, tuple):
            return value
        raise TypeMismatchError(f"{ctx}: Erwartet TUPLE, erhalten {_type_of(value)}")
    if target == "any":
        # Intern fuer Compiler-generierte Slots (z.B. WITH-Ziele,
        # Tupel-Destructuring-Tempvars). Kein Type-Check.
        return value
    if target == "funcref":
        if isinstance(value, _FuncRef):
            return value
        raise TypeMismatchError(f"{ctx}: Erwartet FUNCREF, erhalten {_type_of(value)}")
    if target == "image":
        if value is None or isinstance(value, _Image):
            return value
        raise TypeMismatchError(f"{ctx}: Erwartet IMAGE, erhalten {_type_of(value)}")
    if target == "sound":
        if value is None or isinstance(value, _Sound):
            return value
        raise TypeMismatchError(f"{ctx}: Erwartet SOUND, erhalten {_type_of(value)}")
    if target == "sprite_atlas":
        if value is None or isinstance(value, _SpriteAtlas):
            return value
        raise TypeMismatchError(
            f"{ctx}: Erwartet SPRITE_ATLAS, erhalten {_type_of(value)}"
        )
    if target == "file":
        if value is None or isinstance(value, _GBFile):
            return value
        raise TypeMismatchError(f"{ctx}: Erwartet FILE, erhalten {_type_of(value)}")
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
    if target.startswith("array:"):
        elem = target[6:]
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
    target_cls = classes.get(target) if classes is not None else None
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
    # Externer Typ aus Built-in-Modul (z.B. JSON_HANDLE, DB_CONN, TWEEN, ...).
    from .modules import EXTERNAL_TYPES as _EXT_TYPES
    ext_cls = _EXT_TYPES.get(target)
    if ext_cls is not None:
        if value is None or isinstance(value, ext_cls):
            return value
        raise TypeMismatchError(
            f"{ctx}: Erwartet {target.upper()}, erhalten {_type_of(value)}"
        )
    raise GBRuntimeError(f"Unbekannter Zieltyp: {target}")


cdef str _infer_type(value):
    # Delegiert an die kanonische Modul-Funktion aus interpreter.py
    # (Single-Source -- alle drei Pfade leiten dieselben Typen ab).
    # DECLARE_CONST ist ein Cold-Path, der Python-Call ist unkritisch.
    return _infer_type_canonical(value)


cdef str _fmt(value):
    if value is None:
        return "NIL"
    if isinstance(value, bool):
        return "TRUE" if value else "FALSE"
    if isinstance(value, float):
        if (<double>value).is_integer():
            return f"{value:.1f}"
        return repr(value)
    if isinstance(value, tuple):
        return "(" + ", ".join(_fmt(x) for x in value) + ")"
    if isinstance(value, _FuncRef):
        return f"<FUNCREF {value.name}>"
    from gamebasic.modules.vec2 import _Vec2 as _V2
    if isinstance(value, _V2):
        return f"Vec2({_fmt(value.x)}, {_fmt(value.y)})"
    if isinstance(value, _Instance):
        return f"<{value.cls.name}>"
    if isinstance(value, _GBArray):
        shape = ",".join(str(d) for d in value.dims) if value.dims else ""
        return f"<ARRAY[{shape}] OF {value.element_type.upper()}>"
    if isinstance(value, _Image):
        return "<IMAGE>"
    if isinstance(value, _Sound):
        return "<SOUND>"
    if isinstance(value, _SpriteAtlas):
        return f"<SPRITE_ATLAS frames={len(value.frames)}>"
    if isinstance(value, _GBFile):
        return f"<FILE {value.path}>"
    if isinstance(value, _GBMap):
        return f"<MAP[{len(value.data)}] OF {value.value_type.upper()}>"
    return str(value)


cdef bint _truthy(value):
    if isinstance(value, bool):
        return value
    if isinstance(value, (int, float)):
        return value != 0
    if isinstance(value, str):
        return value != ""
    return value is not None


cdef class _Slot:
    """Globaler-Variablen-Slot. cdef-Attribut-Zugriff (.value/.type/.is_const)
    ist im Hot-Path schneller als dict-Lookup ({"type","value","const"}).
    Parallel zu vm.py._Slot -- 'is_const' statt 'const', weil 'const' in
    Cython ein reserviertes Wort ist."""
    cdef public str type
    cdef public object value
    cdef public bint is_const

    def __init__(self, str type_, value, bint is_const=False):
        self.type = type_
        self.value = value
        self.is_const = is_const


cdef _register_default_globals(dict globals_dict):
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


cdef class VM:
    cdef public dict _globals
    cdef public object module
    cdef public list _global_slots
    cdef object _graphics
    cdef public int data_ptr

    def __init__(self):
        self._globals = {}
        _register_default_globals(self._globals)
        self._global_slots = []
        self._graphics = None
        self.data_ptr = 0     # READ liest hier sequenziell aus module.data

    @property
    def globals(self):
        return self._globals

    cdef _get_graphics(self):
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
        return self._exec(fn, list(args), None)

    def run(self, module):
        self.module = module
        self.data_ptr = 0
        # Slot-Liste fuer Compile-Zeit-bekannte Globals -- parallel zum
        # globals_-Dict (siehe vm.py fuer Begruendung). Pre-registrierte
        # Globals (KEY_*, PI) leben nur im Dict.
        self._global_slots = [None] * module.n_globals
        try:
            self._exec(module.main, [], None)
        finally:
            if self._graphics is not None:
                self._graphics.shutdown()

    cdef _resolve_method(self, cls, name):
        cur = cls
        while cur is not None:
            if name in cur.methods:
                return cur.methods[name]
            cur = cur.parent
        return None

    cdef _user_op(self, str op_method, a, b):
        """User-Class Operator-Dispatch fuer BinaryOp (parallel zu vm.py).

        Liefert _NO_OP, wenn weder a noch b eine _Instance mit der Methode
        sind. Sonst das Resultat des Methoden-Aufrufs (a hat Vorrang vor b).
        """
        if isinstance(a, _Instance):
            m = self._resolve_method(a.cls, op_method)
            if m is not None:
                return self._exec(m, [b], a)
        if isinstance(b, _Instance):
            m = self._resolve_method(b.cls, op_method)
            if m is not None:
                return self._exec(m, [a], b)
        return _NO_OP

    cdef _allocate_instance(self, cls):
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

    cdef _element_default(self, str type_name):
        cls = self.module.classes.get(type_name)
        if cls is not None and cls.is_struct:
            return self._allocate_instance(cls)
        return _TYPE_DEFAULTS.get(type_name)

    cdef _exec(self, object fn, list args, object self_obj):
        cdef int ip = 0
        cdef int op
        cdef int n
        cdef int slot
        cdef int count
        cdef int argc
        cdef int n_params
        cdef int num_dims
        cdef int i

        cdef list code         = fn.code
        cdef list constants    = fn.constants
        cdef list local_types  = fn.local_types
        cdef list local_defs   = fn.local_defaults
        cdef list locals_      = list(local_defs)
        cdef list stack        = []
        cdef dict globals_     = self._globals
        cdef list global_slots = self._global_slots
        cdef dict classes      = self.module.classes
        # Inline-Cache: parallel zu code, monomorphic. caches[ip-1] gehoert
        # zur gerade ausgefuehrten Instr. None = Cache leer; sonst eine
        # 2-elementige Liste [recv_cls, payload].
        cdef list caches       = fn.caches
        cdef object cache
        cdef object slot_obj
        cdef _Slot gslot

        cdef object instr, arg
        cdef object value, a, b
        cdef object slot_dict
        cdef object name, type_name, fn_name, method_name, class_name, elem_type
        cdef object call_args
        cdef object callee, method
        cdef object bf, gh
        cdef object existing, prompt, obj
        cdef object cls
        cdef list idx_vals, dims, parts
        cdef list try_handlers = []
        cdef object exc, msg
        cdef int target, depth
        cdef bint has_prompt, has_init_args
        cdef bint is_sub = fn.is_sub

        n_params = fn.n_params
        # Variadic: letzter Param ist TUPLE, sammelt alle ueberzaehligen Args.
        if fn.is_variadic:
            normal_n = n_params - 1
            if len(args) < normal_n:
                raise GBRuntimeError(
                    f"{fn.name.upper()}: erwartet mindestens {normal_n} "
                    f"Argument(e), erhalten {len(args)}"
                )
            for i in range(normal_n):
                locals_[i] = _coerce(args[i], local_types[i],
                                      f"Parameter {i+1}", classes)
            locals_[n_params - 1] = tuple(args[normal_n:])
        else:
            # Default-Parameter-aware: n_required = Anzahl ohne Default.
            # Wenn fn keine Defaults hat, ist n_required == n_params (alles
            # required). Tree-Walker und Python-VM verwenden dieselbe Logik.
            n_required = fn.n_required if fn.n_required else n_params
            if len(args) < n_required or len(args) > n_params:
                if n_required == n_params:
                    raise GBRuntimeError(
                        f"{fn.name.upper()}: erwartet {n_params} Argument(e), "
                        f"erhalten {len(args)}"
                    )
                raise GBRuntimeError(
                    f"{fn.name.upper()}: erwartet {n_required}..{n_params} "
                    f"Argument(e), erhalten {len(args)}"
                )
            for i in range(len(args)):
                locals_[i] = _coerce(args[i], local_types[i], f"Parameter {i+1}", classes)
            # Fehlende Parameter mit kompilierten Defaults fuellen
            for i in range(len(args), n_params):
                default = fn.param_defaults[i] if fn.param_defaults else None
                if default is None:
                    raise GBRuntimeError(
                        f"{fn.name.upper()}: Parameter {i+1} hat keinen Default"
                    )
                locals_[i] = _coerce(default, local_types[i],
                                      f"Default-Parameter {i+1}", classes)

        n = len(code)

        while True:
         try:
          while ip < n:
                  instr = code[ip]
                  op = instr[0]
                  arg = instr[1]
                  ip += 1

                  # --- Stack ---
                  if op == OP_LOAD_CONST:
                      stack.append(constants[<int>arg])
                  elif op == OP_LOAD_LOCAL:
                      stack.append(locals_[<int>arg])
                  elif op == OP_STORE_LOCAL:
                      slot = <int>arg
                      value = stack.pop()
                      locals_[slot] = _coerce(value, local_types[slot],
                                               f"Lokale Variable [{slot}]", classes)
                  elif op == OP_DECLARE_LOCAL:
                      slot = <int>arg[0]
                      type_name = arg[1]
                      # MAP-Typ: leere Map allokieren
                      if isinstance(type_name, str) and type_name.startswith("map:"):
                          if not isinstance(locals_[slot], _GBMap):
                              locals_[slot] = _GBMap(type_name[4:])
                      elif locals_[slot] is None and arg[2] is not None:
                          locals_[slot] = arg[2]
                  elif op == OP_LOAD_NAME:
                      name = constants[<int>arg]
                      gslot = globals_.get(name)
                      if gslot is None:
                          raise GBRuntimeError(f"Variable '{name}' nicht deklariert (DIM fehlt?)")
                      stack.append(gslot.value)
                  elif op == OP_STORE_NAME:
                      name = constants[<int>arg]
                      gslot = globals_.get(name)
                      if gslot is None:
                          raise GBRuntimeError(f"Variable '{name}' nicht deklariert (DIM fehlt?)")
                      if gslot.is_const:
                          raise GBRuntimeError(f"CONST '{name}' kann nicht ueberschrieben werden")
                      value = stack.pop()
                      gslot.value = _coerce(value, gslot.type,
                                            f"Zuweisung an '{name}'", classes)
                  elif op == OP_DECLARE_NAME:
                      name = constants[<int>arg[0]]
                      type_name = constants[<int>arg[1]]
                      value = constants[<int>arg[2]]
                      # MAP-Typ: frische Map allokieren (default ist None)
                      if isinstance(type_name, str) and type_name.startswith("map:"):
                          value = _GBMap(type_name[4:])
                      existing = globals_.get(name)
                      if existing is None:
                          globals_[name] = _Slot(type_name, value, False)
                      else:
                          if existing.is_const:
                              raise GBRuntimeError(
                                  f"'{name}' ist CONST und kann nicht erneut deklariert werden"
                              )
                          if existing.type != type_name:
                              raise GBRuntimeError(
                                  f"Variable '{name}' war als {existing.type.upper()} "
                                  f"deklariert, jetzt als {type_name.upper()} - Typkonflikt"
                              )
                  elif op == OP_DECLARE_CONST:
                      name = constants[<int>arg[0]]
                      type_name = constants[<int>arg[1]] if arg[1] is not None else None
                      value = stack.pop()
                      if type_name is None:
                          type_name = _infer_type(value)
                      else:
                          value = _coerce(value, type_name, f"CONST {name}", classes)
                      if name in globals_:
                          existing = globals_[name]
                          if (existing.is_const
                                  and existing.type == type_name
                                  and existing.value == value):
                              pass  # idempotenter Re-CONST
                          else:
                              raise GBRuntimeError(f"'{name}' bereits deklariert")
                      else:
                          globals_[name] = _Slot(type_name, value, True)

                  # --- Slot-basierte Globals -------------------------
                  # Parallel zum Dict-Pfad. Dict-Lookup (globals_[name])
                  # entfaellt im Hot-Path; der Slot-Eintrag ist dasselbe
                  # _Slot-Objekt wie globals_[name], also bleibt name-
                  # basierte Sicht (INPUT_NAME etc.) synchron.
                  elif op == OP_LOAD_GLOBAL_SLOT:
                      gslot = global_slots[<int>arg]
                      stack.append(gslot.value)
                  elif op == OP_STORE_GLOBAL_SLOT:
                      gslot = global_slots[<int>arg]
                      if gslot.is_const:
                          raise GBRuntimeError(
                              "CONST kann nicht ueberschrieben werden"
                          )
                      value = stack.pop()
                      gslot.value = _coerce(
                          value, gslot.type,
                          "Zuweisung an global", classes,
                      )
                  elif op == OP_DECLARE_GLOBAL_SLOT:
                      slot_idx = <int>arg[0]
                      name = constants[<int>arg[1]]
                      type_name = constants[<int>arg[2]]
                      default = constants[<int>arg[3]]
                      if isinstance(type_name, str) and type_name.startswith("map:"):
                          default = _GBMap(type_name[4:])
                      if name in globals_:
                          existing = globals_[name]
                          if existing.is_const:
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
                  elif op == OP_DECLARE_GLOBAL_CONST_SLOT:
                      slot_idx = <int>arg[0]
                      name = constants[<int>arg[1]]
                      type_name = constants[<int>arg[2]] if arg[2] is not None else None
                      value = stack.pop()
                      if type_name is None:
                          type_name = _infer_type(value)
                      else:
                          value = _coerce(value, type_name, f"CONST {name}", classes)
                      if name in globals_:
                          existing = globals_[name]
                          if (existing.is_const
                                  and existing.type == type_name
                                  and existing.value == value):
                              global_slots[slot_idx] = existing
                          else:
                              raise GBRuntimeError(f"'{name}' bereits deklariert")
                      else:
                          slot_obj = _Slot(type_name, value, True)
                          globals_[name] = slot_obj
                          global_slots[slot_idx] = slot_obj
                  elif op == OP_POP:
                      stack.pop()
                  elif op == OP_DUP:
                      stack.append(stack[len(stack) - 1])

                  # --- Arithmetik ---
                  # Modul-registriertes Operator-Overloading (Vec2 etc.)
                  # laeuft ueber dispatch_binary_op. _disp_op und _NO_OP sind
                  # Modul-Top-Level-Imports (siehe oben). User-Class
                  # Operator-Methoden (`OPERATOR + (other) AS T`) werden
                  # nach Registry-Miss via self._user_op probiert.
                  elif op == OP_ADD:
                      b = stack.pop()
                      a = stack.pop()
                      r = _disp_op("+", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_add__", a, b)
                      if r is not _NO_OP:
                          stack.append(r)
                      elif isinstance(a, str) or isinstance(b, str):
                          stack.append(_fmt(a) + _fmt(b))
                      else:
                          _require_number(a, b, "+")
                          stack.append(a + b)
                  elif op == OP_SUB:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("-", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_sub__", a, b)
                      if r is not _NO_OP:
                          stack.append(r)
                      else:
                          _require_number(a, b, "-")
                          stack.append(a - b)
                  elif op == OP_MUL:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("*", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_mul__", a, b)
                      if r is not _NO_OP:
                          stack.append(r)
                      elif isinstance(a, str) and isinstance(b, int) and not isinstance(b, bool):
                          stack.append(a * b if b > 0 else "")
                      elif isinstance(b, str) and isinstance(a, int) and not isinstance(a, bool):
                          stack.append(b * a if a > 0 else "")
                      else:
                          # Strikt Zahl * Zahl -- Bool und non-numeric werfen.
                          _require_number(a, b, "*")
                          stack.append(a * b)
                  elif op == OP_DIV:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("/", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_div__", a, b)
                      if r is not _NO_OP:
                          stack.append(r)
                      else:
                          _require_number(a, b, "/")
                          if b == 0:
                              raise GBRuntimeError("Division durch 0")
                          elif isinstance(a, int) and isinstance(b, int) and a % b == 0:
                              stack.append(a // b)
                          else:
                              stack.append(a / b)
                  elif op == OP_INT_DIV:
                      b = stack.pop(); a = stack.pop()
                      if isinstance(a, bool) or isinstance(b, bool):
                          raise TypeMismatchError("\\ erwartet INTEGER")
                      if not isinstance(a, int) or not isinstance(b, int):
                          raise TypeMismatchError("\\ erwartet INTEGER (kein FLOAT)")
                      if b == 0:
                          raise GBRuntimeError("Integer-Division durch 0")
                      q, r = divmod(a, b)
                      if r != 0 and (a < 0) != (b < 0):
                          q += 1
                      stack.append(q)
                  elif op == OP_MOD:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("mod", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_mod__", a, b)
                      if r is not _NO_OP:
                          stack.append(r)
                      else:
                          _require_number(a, b, "MOD")
                          if b == 0:
                              raise GBRuntimeError("MOD durch 0")
                          stack.append(a % b)
                  elif op == OP_POW:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("^", a, b)
                      if r is not _NO_OP:
                          stack.append(r)
                      else:
                          _require_number(a, b, "^")
                          stack.append(a ** b)
                  elif op == OP_NEG:
                      value = stack.pop()
                      if isinstance(value, bool) or not isinstance(value, (int, float)):
                          raise TypeMismatchError("Unaeres '-' erwartet Zahl")
                      stack.append(-value)

                  # --- Vergleich / Logik ---
                  # Reihenfolge wie im Tree-Walker (interpreter._eval_BinaryOp):
                  # 1. Modul-Operator-Dispatch (_disp_op), dann
                  # 2. User-Klassen-Op (`__op_eq__` etc.), dann
                  # 3. Python-`==`/`<` etc.
                  elif op == OP_EQ:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("=", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_eq__", a, b)
                      stack.append(a == b if r is _NO_OP else r)
                  elif op == OP_NEQ:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("<>", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_ne__", a, b)
                      stack.append(a != b if r is _NO_OP else r)
                  elif op == OP_LT:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("<", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_lt__", a, b)
                      stack.append(a < b if r is _NO_OP else r)
                  elif op == OP_GT:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op(">", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_gt__", a, b)
                      stack.append(a > b if r is _NO_OP else r)
                  elif op == OP_LEQ:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op("<=", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_le__", a, b)
                      stack.append(a <= b if r is _NO_OP else r)
                  elif op == OP_GEQ:
                      b = stack.pop(); a = stack.pop()
                      r = _disp_op(">=", a, b)
                      if r is _NO_OP:
                          r = self._user_op("__op_ge__", a, b)
                      stack.append(a >= b if r is _NO_OP else r)
                  elif op == OP_NOT:
                      stack.append(not _truthy(stack.pop()))

                  # --- Bitwise (strikt INTEGER) ---
                  elif op == OP_BAND:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "BAND")
                      stack.append(a & b)
                  elif op == OP_BOR:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "BOR")
                      stack.append(a | b)
                  elif op == OP_BXOR:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "BXOR")
                      stack.append(a ^ b)
                  elif op == OP_SHL:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "SHL")
                      if b < 0:
                          raise GBRuntimeError("SHL: Shift-Anzahl darf nicht negativ sein")
                      stack.append(a << b)
                  elif op == OP_SHR:
                      b = stack.pop(); a = stack.pop()
                      _require_int_pair(a, b, "SHR")
                      if b < 0:
                          raise GBRuntimeError("SHR: Shift-Anzahl darf nicht negativ sein")
                      stack.append(a >> b)
                  elif op == OP_BNOT:
                      value = stack.pop()
                      if isinstance(value, bool) or not isinstance(value, int):
                          raise TypeMismatchError("BNOT erwartet INTEGER")
                      stack.append(~value)

                  # --- Spezialisierte Numeric-Numeric Ops -------------------
                  # Compiler emittiert diese, wenn beide Operanden statisch
                  # als INTEGER/FLOAT bekannt sind. Kein Modul-Operator-
                  # Dispatch, kein User-Class-Dispatch, keine isinstance-
                  # Cascade. Bit-identisch zu OP_ADD/SUB/MUL/DIV etc. fuer
                  # numerische Operanden.
                  elif op == OP_ADD_NN:
                      b = stack.pop(); a = stack.pop(); stack.append(a + b)
                  elif op == OP_SUB_NN:
                      b = stack.pop(); a = stack.pop(); stack.append(a - b)
                  elif op == OP_MUL_NN:
                      b = stack.pop(); a = stack.pop(); stack.append(a * b)
                  elif op == OP_DIV_NN:
                      b = stack.pop(); a = stack.pop()
                      if b == 0:
                          raise GBRuntimeError("Division durch 0")
                      # Identisch zu OP_DIV: Int/Int mit Rest=0 -> Int, sonst Float.
                      if isinstance(a, int) and isinstance(b, int) and a % b == 0:
                          stack.append(a // b)
                      else:
                          stack.append(a / b)
                  elif op == OP_LT_NN:
                      b = stack.pop(); a = stack.pop(); stack.append(a < b)
                  elif op == OP_GT_NN:
                      b = stack.pop(); a = stack.pop(); stack.append(a > b)
                  elif op == OP_LEQ_NN:
                      b = stack.pop(); a = stack.pop(); stack.append(a <= b)
                  elif op == OP_GEQ_NN:
                      b = stack.pop(); a = stack.pop(); stack.append(a >= b)
                  elif op == OP_EQ_NN:
                      b = stack.pop(); a = stack.pop(); stack.append(a == b)
                  elif op == OP_NEQ_NN:
                      b = stack.pop(); a = stack.pop(); stack.append(a != b)
                  elif op == OP_NEG_N:
                      stack.append(-stack.pop())

                  # --- Tupel ---
                  elif op == OP_BUILD_TUPLE:
                      tlen = <int>arg
                      if tlen == 0:
                          stack.append(())
                      else:
                          t = tuple(stack[len(stack) - tlen:])
                          del stack[len(stack) - tlen:]
                          stack.append(t)
                  elif op == OP_UNPACK_TUPLE:
                      tlen = <int>arg
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
                      # Reverse-push: erstes Element liegt oben.
                      for vv in reversed(t):
                          stack.append(vv)

                  # --- Kontrollfluss ---
                  elif op == OP_JUMP:
                      ip = <int>arg
                  elif op == OP_JUMP_IF_FALSE:
                      value = stack.pop()
                      if not _truthy(value):
                          ip = <int>arg
                  elif op == OP_JUMP_IF_TRUE:
                      value = stack.pop()
                      if _truthy(value):
                          ip = <int>arg

                  # --- Aufrufe ---
                  elif op == OP_CALL_USER:
                      fn_name = arg[0]
                      argc = <int>arg[1]
                      callee = self.module.functions.get(fn_name)
                      if callee is None:
                          raise GBRuntimeError(f"Unbekannte Funktion: {fn_name.upper()}")
                      if argc > 0:
                          call_args = stack[-argc:]
                          del stack[-argc:]
                      else:
                          call_args = []
                      value = self._exec(callee, call_args, None)
                      if not callee.is_sub:
                          stack.append(value)
                      else:
                          stack.append(None)
                  elif op == OP_CALL_BUILTIN:
                      fn_name = arg[0]
                      argc = <int>arg[1]
                      if argc > 0:
                          call_args = stack[-argc:]
                          del stack[-argc:]
                      else:
                          call_args = []
                      gh = GRAPHICS_BUILTINS.get(fn_name)
                      if gh is not None:
                          stack.append(gh(self._get_graphics(), call_args))
                      else:
                          bf = BUILTINS.get(fn_name)
                          if bf is None:
                              raise GBRuntimeError(f"Unbekannte Funktion: {fn_name.upper()}")
                          stack.append(bf(call_args))
                  elif op == OP_LOAD_FUNCREF:
                      ref_name = constants[<int>arg]
                      if ref_name not in self.module.functions:
                          raise GBRuntimeError(
                              f"FUNCREF: Funktion '{ref_name}' existiert nicht"
                          )
                      stack.append(_FuncRef(ref_name))
                  elif op == OP_BUILD_TUPLE_DYN:
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
                      stack.append(values)
                  elif op == OP_IN_OP:
                      hay = stack.pop()
                      needle = stack.pop()
                      stack.append(_eval_in_native(needle, hay))
                  elif op == OP_SLICE:
                      _slice_dispatch(stack, arg)
                  elif op == OP_CALL_VALUE:
                      argc = <int>arg
                      if argc > 0:
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
                      ret = self._exec(tgt, call_args, None)
                      if not tgt.is_sub:
                          stack.append(ret)
                      else:
                          stack.append(None)
                  elif op == OP_CALL_METHOD:
                      method_name = arg[0]
                      argc = <int>arg[1]
                      if argc > 0:
                          call_args = stack[-argc:]
                          del stack[-argc:]
                      else:
                          call_args = []
                      obj = stack.pop()
                      if obj is None:
                          raise GBRuntimeError(
                              f"Methodenaufruf '.{method_name}' bei NIL-Referenz"
                          )
                      # Inline-Cache: monomorphic, ueberspringt Container-Check
                      # und _resolve_method bei _Instance mit gleichem cls.
                      cache = caches[ip - 1]
                      if (cache is not None
                              and type(obj) is _Instance
                              and obj.cls is cache[0]):
                          method = cache[1]
                          value = self._exec(method, call_args, obj)
                          if not method.is_sub:
                              stack.append(value)
                          else:
                              stack.append(None)
                          continue
                      # Container-Method-Dispatch -- arr.length(), s.upper() etc.
                      kind = _container_kind_native(obj)
                      if kind:
                          builtin_name = _CONTAINER_METHODS.get(
                              (kind, method_name.lower()))
                          if builtin_name is None:
                              raise GBRuntimeError(
                                  f"{kind.upper()} hat keine Methode '{method_name}'"
                              )
                          stack.append(BUILTINS[builtin_name]([obj] + call_args))
                          continue
                      if not isinstance(obj, _Instance):
                          raise GBRuntimeError(
                              f"Methodenaufruf '.{method_name}' bei nicht-Objekt ({_type_of(obj)})"
                          )
                      method = self._resolve_method(obj.cls, method_name)
                      if method is None:
                          raise GBRuntimeError(
                              f"Methode '{method_name}' existiert nicht in {obj.cls.name}"
                          )
                      caches[ip - 1] = [obj.cls, method]
                      value = self._exec(method, call_args, obj)
                      if not method.is_sub:
                          stack.append(value)
                      else:
                          stack.append(None)
                  elif op == OP_RETURN:
                      value = stack.pop()
                      if is_sub:
                          raise GBRuntimeError(
                              f"SUB '{fn.name}' darf RETURN nicht mit Wert verwenden"
                          )
                      return _coerce(value, fn.return_type, f"RETURN aus {fn.name.upper()}", classes)
                  elif op == OP_RETURN_VOID:
                      if not is_sub:
                          raise GBRuntimeError(
                              f"FUNCTION '{fn.name}' muss einen Wert mit RETURN zurueckgeben"
                          )
                      return None

                  # --- I/O ---
                  elif op == OP_PRINT:
                      count = <int>arg
                      if count == 0:
                          print()
                      else:
                          parts = stack[-count:]
                          del stack[-count:]
                          print(" ".join(_fmt(x) for x in parts))
                  elif op == OP_INPUT_NAME:
                      has_prompt = <bint>arg[1]
                      prompt = stack.pop() if has_prompt else None
                      gslot = globals_.get(constants[<int>arg[0]])
                      if gslot is None:
                          raise GBRuntimeError(
                              f"Variable '{constants[<int>arg[0]]}' nicht deklariert"
                          )
                      gslot.value = self._do_input(gslot.type, prompt)
                  elif op == OP_INPUT_LOCAL:
                      slot = <int>arg[0]
                      has_prompt = <bint>arg[1]
                      prompt = stack.pop() if has_prompt else None
                      locals_[slot] = self._do_input(local_types[slot], prompt)

                  # --- OOP ---
                  elif op == OP_NEW_INSTANCE:
                      class_name = arg[0]
                      argc = <int>arg[1]
                      has_init_args = <bint>arg[2]
                      cls = classes.get(class_name)
                      if cls is None:
                          raise GBRuntimeError(f"Klasse '{class_name}' nicht gefunden")
                      if has_init_args:
                          if argc > 0:
                              call_args = stack[-argc:]
                              del stack[-argc:]
                          else:
                              call_args = []
                          obj = self._allocate_instance(cls)
                          method = self._resolve_method(cls, "init")
                          if method is None:
                              if call_args:
                                  raise GBRuntimeError(
                                      f"Klasse {cls.name} hat keine SUB Init - "
                                      f"Argumente bei NEW nicht moeglich"
                                  )
                          else:
                              self._exec(method, call_args, obj)
                          stack.append(obj)
                      else:
                          stack.append(self._allocate_instance(cls))
                  elif op == OP_LOAD_SELF:
                      if self_obj is None:
                          raise GBRuntimeError(
                              "LOAD_SELF (Self) ausserhalb Methodenkontext"
                          )
                      stack.append(self_obj)
                  elif op == OP_LOAD_FIELD:
                      name = constants[<int>arg]
                      if self_obj is None:
                          raise GBRuntimeError(
                              f"LOAD_FIELD '{name}' ausserhalb Methodenkontext"
                          )
                      stack.append(self_obj.fields[name]["value"])
                  elif op == OP_STORE_FIELD:
                      name = constants[<int>arg]
                      if self_obj is None:
                          raise GBRuntimeError(
                              f"STORE_FIELD '{name}' ausserhalb Methodenkontext"
                          )
                      slot_dict = self_obj.fields[name]
                      value = stack.pop()
                      slot_dict["value"] = _coerce(
                          value, slot_dict["type"],
                          f"Zuweisung an {self_obj.cls.name}.{name}", classes,
                      )
                  elif op == OP_LOAD_MEMBER:
                      name = constants[<int>arg]
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
                              stack.append(obj.fields[name]["value"])
                          else:
                              stack.append(self._exec(getter, [], obj))
                          continue
                      if isinstance(obj, _EnumNamespace):
                          val = obj.get(name)
                          if val is None:
                              avail = ", ".join(obj.members.keys())
                              raise GBRuntimeError(
                                  f"ENUM {obj.name} hat keinen Member '{name}' "
                                  f"(verfuegbar: {avail})"
                              )
                          stack.append(val)
                          continue
                      if isinstance(obj, _ClassStaticNamespace):
                          val = obj.get(name)
                          if val is None:
                              avail = ", ".join(obj.members.keys()) or "<keine>"
                              raise GBRuntimeError(
                                  f"CLASS {obj.name} hat keinen STATIC-Member '{name}' "
                                  f"(verfuegbar: {avail})"
                              )
                          stack.append(val)
                          continue
                      if not isinstance(obj, _Instance):
                          raise GBRuntimeError(
                              f"Zugriff auf '.{name}' bei nicht-Objekt ({_type_of(obj)})"
                          )
                      # Property-Getter-Dispatch
                      if _is_property_native(obj.cls, name):
                          getter = self._resolve_method(obj.cls, f"__get_{name.lower()}")
                          if getter is None:
                              raise GBRuntimeError(
                                  f"Property '{name}' in {obj.cls.name} hat "
                                  f"keinen Getter (nur SET deklariert)"
                              )
                          caches[ip - 1] = [obj.cls, getter]
                          stack.append(self._exec(getter, [], obj))
                          continue
                      if name not in obj.fields:
                          raise GBRuntimeError(f"Feld '{name}' existiert nicht in {obj.cls.name}")
                      caches[ip - 1] = [obj.cls, None]
                      stack.append(obj.fields[name]["value"])
                  elif op == OP_STORE_MEMBER:
                      name = constants[<int>arg]
                      value = stack.pop()
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
                              slot_dict = obj.fields[name]
                              slot_dict["value"] = _coerce(
                                  value, slot_dict["type"],
                                  f"Zuweisung an {obj.cls.name}.{name}", classes,
                              )
                          else:
                              self._exec(setter, [value], obj)
                          continue
                      if not isinstance(obj, _Instance):
                          raise GBRuntimeError(
                              f"Zuweisung an '.{name}' bei nicht-Objekt ({_type_of(obj)})"
                          )
                      # Property-Setter-Dispatch
                      if _is_property_native(obj.cls, name):
                          setter = self._resolve_method(obj.cls, f"__set_{name.lower()}")
                          if setter is None:
                              raise GBRuntimeError(
                                  f"Property '{name}' in {obj.cls.name} hat "
                                  f"keinen Setter (read-only)"
                              )
                          caches[ip - 1] = [obj.cls, setter]
                          self._exec(setter, [value], obj)
                          continue
                      if name not in obj.fields:
                          raise GBRuntimeError(f"Feld '{name}' existiert nicht in {obj.cls.name}")
                      caches[ip - 1] = [obj.cls, None]
                      slot_dict = obj.fields[name]
                      slot_dict["value"] = _coerce(
                          value, slot_dict["type"],
                          f"Zuweisung an {obj.cls.name}.{name}", classes,
                      )
                  elif op == OP_DECLARE_STRUCT_NAME:
                      name = constants[<int>arg[0]]
                      class_name = arg[1]
                      cls = classes.get(class_name)
                      if cls is None:
                          raise GBRuntimeError(f"STRUCT '{class_name}' nicht gefunden")
                      if name not in globals_:
                          globals_[name] = _Slot(
                              class_name, self._allocate_instance(cls), False
                          )
                  elif op == OP_DECLARE_STRUCT_LOCAL:
                      slot = <int>arg[0]
                      class_name = arg[1]
                      if locals_[slot] is None:
                          cls = classes.get(class_name)
                          if cls is None:
                              raise GBRuntimeError(f"STRUCT '{class_name}' nicht gefunden")
                          locals_[slot] = self._allocate_instance(cls)

                  # --- Arrays ---
                  elif op == OP_LOAD_INDEX:
                      num_dims = <int>arg
                      if num_dims > 0:
                          idx_vals = stack[-num_dims:]
                          del stack[-num_dims:]
                      else:
                          idx_vals = []
                      obj = stack.pop()
                      # Fast path: 1D-_GBArray, ein int-Index in Bounds
                      # (haeufigstes Pattern: Pixel-Buffer, Tilemap, Listen).
                      # Spart die isinstance-Cascade + Index-Validierungs-Loop.
                      # Alle Edge-Cases (String/Tupel/Multidim/OOB/bool) fallen
                      # in den generischen Pfad -> identische Semantik/Fehler.
                      if num_dims == 1 and type(obj) is _GBArray and len(obj.dims) == 1:
                          value = idx_vals[0]
                          if type(value) is int and 0 <= value < obj.dims[0]:
                              stack.append(obj.get_at(idx_vals))
                              continue
                      if obj is None:
                          raise GBRuntimeError("Index-Zugriff auf NIL")
                      # String-Index: einzelner Int -> 1-Char-String.
                      if isinstance(obj, str):
                          if len(idx_vals) != 1:
                              raise GBRuntimeError("String-Index braucht genau einen Wert")
                          value = idx_vals[0]
                          if isinstance(value, bool) or not isinstance(value, int):
                              raise TypeMismatchError(
                                  f"String-Index muss INTEGER sein, erhalten {_type_of(value)}"
                              )
                          if value < 0 or value >= len(obj):
                              raise GBRuntimeError(
                                  f"String-Index {value} ausserhalb des Bereichs (Laenge {len(obj)})"
                              )
                          stack.append(obj[value])
                          continue
                      # Tupel-Index: einzelner Integer-Wert.
                      if isinstance(obj, tuple):
                          if len(idx_vals) != 1:
                              raise GBRuntimeError("Tupel-Index braucht genau einen Wert")
                          value = idx_vals[0]
                          if isinstance(value, bool) or not isinstance(value, int):
                              raise TypeMismatchError(
                                  f"Tupel-Index muss INTEGER sein, erhalten {_type_of(value)}"
                              )
                          if value < 0 or value >= len(obj):
                              raise GBRuntimeError(
                                  f"Tupel-Index {value} ausserhalb des Bereichs (Laenge {len(obj)})"
                              )
                          stack.append(obj[value])
                          continue
                      if not isinstance(obj, _GBArray):
                          raise GBRuntimeError(
                              f"Index-Zugriff auf Nicht-Array ({_type_of(obj)})"
                          )
                      for value in idx_vals:
                          if isinstance(value, bool) or not isinstance(value, int):
                              raise TypeMismatchError(
                                  f"Array-Index muss INTEGER sein, erhalten {_type_of(value)}"
                              )
                      # Fast path: cdef-class get_at fuehrt Bounds-Check und
                      # typed-buffer-Zugriff in reinem C aus.
                      stack.append(obj.get_at(idx_vals))
                  elif op == OP_STORE_INDEX:
                      num_dims = <int>arg
                      value = stack.pop()
                      if num_dims > 0:
                          idx_vals = stack[-num_dims:]
                          del stack[-num_dims:]
                      else:
                          idx_vals = []
                      obj = stack.pop()
                      # Fast path: 1D-_GBArray, ein int-Index in Bounds.
                      if num_dims == 1 and type(obj) is _GBArray and len(obj.dims) == 1:
                          a = idx_vals[0]
                          if type(a) is int and 0 <= a < obj.dims[0]:
                              obj.set_at(idx_vals, _coerce(
                                  value, obj.element_type,
                                  f"Array-Element [{a}]", classes))
                              continue
                      if obj is None:
                          raise GBRuntimeError("Index-Zuweisung an NIL")
                      if not isinstance(obj, _GBArray):
                          raise GBRuntimeError(
                              f"Index-Zuweisung an Nicht-Array ({_type_of(obj)})"
                          )
                      for a in idx_vals:
                          if isinstance(a, bool) or not isinstance(a, int):
                              raise TypeMismatchError(
                                  f"Array-Index muss INTEGER sein, erhalten {_type_of(a)}"
                              )
                      obj.set_at(
                          idx_vals,
                          _coerce(
                              value, obj.element_type,
                              f"Array-Element [{','.join(str(i) for i in idx_vals)}]",
                              classes,
                          ),
                      )
                  elif op == OP_DECLARE_ARRAY_NAME:
                      name = constants[<int>arg[0]]
                      elem_type = arg[1]
                      num_dims = <int>arg[2]
                      if num_dims > 0:
                          dims = stack[-num_dims:]
                          del stack[-num_dims:]
                      else:
                          dims = []
                      for value in dims:
                          if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                              raise GBRuntimeError("Array-Groesse muss INTEGER >= 0 sein")
                      obj = _GBArray(
                          elem_type, dims,
                          lambda t=elem_type: self._element_default(t),
                      )
                      globals_[name] = _Slot(f"array:{elem_type}", obj, False)
                  elif op == OP_DECLARE_ARRAY_LOCAL:
                      slot = <int>arg[0]
                      elem_type = arg[1]
                      num_dims = <int>arg[2]
                      if num_dims > 0:
                          dims = stack[-num_dims:]
                          del stack[-num_dims:]
                      else:
                          dims = []
                      for value in dims:
                          if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                              raise GBRuntimeError("Array-Groesse muss INTEGER >= 0 sein")
                      locals_[slot] = _GBArray(
                          elem_type, dims,
                          lambda t=elem_type: self._element_default(t),
                      )

                  # --- Exceptions ---
                  elif op == OP_TRY_BEGIN:
                      try_handlers.append((<int>arg, len(stack)))
                  elif op == OP_TRY_END:
                      try_handlers.pop()
                  elif op == OP_THROW:
                      value = stack.pop()
                      msg = value if isinstance(value, str) else _fmt(value)
                      raise _GBThrow(msg)

                  # --- DATA / READ / RESTORE ---
                  elif op == OP_PUSH_DATA:
                      if self.data_ptr >= len(self.module.data):
                          raise GBRuntimeError(
                              "READ: keine DATA-Werte mehr "
                              "(benutze RESTORE zum Reset)"
                          )
                      stack.append(self.module.data[self.data_ptr])
                      self.data_ptr += 1
                  elif op == OP_RESET_DATA_PTR:
                      self.data_ptr = 0

                  elif op == OP_HALT:
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

    cdef _do_input(self, str t, prompt):
        cdef str prompt_str = ""
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
                f"Eingabe '{raw}' passt nicht zu {t.upper()}"
            )
        return v
