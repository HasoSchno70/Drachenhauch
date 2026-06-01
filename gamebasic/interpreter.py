"""Tree-Walking-Interpreter fuer GameBasic.

Strikte Typpruefung: Zuweisungen werden auf den deklarierten Typ geprueft.
- INTEGER  -> int
- FLOAT    -> float (akzeptiert auch int und konvertiert)
- STRING   -> str
- BOOLEAN  -> bool
"""
import array as _array_mod
import math
import random

from .ast_nodes import (
    NumberLit, StringLit, BoolLit, Identifier, BinaryOp, UnaryOp, Call,
    Dim, MultiDim, Assign, Print, Input, If, While, For, ExprStmt, Program,
    Param, SubDecl, FunctionDecl, Return,
    ClassDecl, New, MemberAccess, MemberAssign,
    Const, Break, Continue, IndexAccess, IndexAssign,
    Try, Throw, Select, CaseMatch,
    Repeat, Data, Read, Restore,
    EnumDecl, NamedArg,
)


# Sentinel fuer "nimm den Default des Parameters".  Wird vom Caller in die
# args-Liste eingefuegt fuer Slots, die der User per Named-Arg NICHT belegt
# hat (oder die in der positional-Liste fehlen).  `_invoke` erkennt den
# Sentinel und evaluiert den Default-Ausdruck im local_env.
_DEFAULT_SENTINEL = object()

# Mapping: Operator-Symbol -> interner Methoden-Name auf User-Klassen.
# Tree-Walker und beide VMs konsultieren das beim BinaryOp-Dispatch,
# nachdem die Modul-Registry (Vec2 etc.) NO_OP_MATCH geliefert hat.
# Parser legt diese Methoden mit genau diesen Namen ab (siehe
# parser._OPERATOR_NAMES).
_USER_OP_METHODS = {
    "+":   "__op_add__",
    "-":   "__op_sub__",
    "*":   "__op_mul__",
    "/":   "__op_div__",
    "mod": "__op_mod__",
    "=":   "__op_eq__",
    "<>":  "__op_ne__",
    "<":   "__op_lt__",
    ">":   "__op_gt__",
    "<=":  "__op_le__",
    ">=":  "__op_ge__",
}
from .environment import Environment
from .errors import GameBasicError, GBRuntimeError, TypeMismatchError
from .builtins_registry import (
    BUILTINS as _REG_BUILTINS,
    GRAPHICS_BUILTINS as _REG_GFX_BUILTINS,
    builtin,
    graphics_builtin,
    _check_intish,
)


class _ReturnSignal(Exception):
    """Interner Mechanismus, um RETURN den Call-Stack hochzubefoerdern."""
    def __init__(self, value):
        self.value = value


class _BreakSignal(Exception):
    """BREAK-Anweisung: bricht aus der innersten Schleife aus."""


class _ContinueSignal(Exception):
    """CONTINUE-Anweisung: springt zur naechsten Iteration."""


class _GBThrow(Exception):
    """User-erzeugter Fehler via THROW. Wert ist immer STRING."""

    def __init__(self, value: str):
        self.value = value
        super().__init__(value)


# Native cdef-class-Implementation. Drop-in-Ersatz fuer die alte Python-
# Klasse; gleiche Public-API. Zusaetzlich liefert sie cpdef get_at/set_at,
# die die VMs im Hot-Pfad statt arr.values[arr.flat_index(...)] rufen.
# Wenn das gebaute .pyd fehlt (z.B. nach git clone vor setup.py), gibt es
# einen Pure-Python-Fallback, damit der Tree-Walker noch laeuft.
try:
    from .array_native import _GBArray  # type: ignore
except ImportError:
    class _GBArray:  # pragma: no cover -- Fallback
        """Pure-Python-Fallback. Identische Semantik zur cdef-Variante."""
        __slots__ = ("element_type", "dims", "strides", "values")

        _TYPED_BACKING = {"integer": "q", "float": "d"}

        def __init__(self, element_type, dims, default_factory):
            self.element_type = element_type
            self.dims = tuple(int(d) for d in dims)
            strides = []
            acc = 1
            for d in reversed(self.dims):
                strides.append(acc)
                acc *= d
            strides.reverse()
            self.strides = tuple(strides)
            tc = _GBArray._TYPED_BACKING.get(element_type)
            if tc is not None:
                self.values = _array_mod.array(tc, [default_factory()] * acc)
            else:
                self.values = [default_factory() for _ in range(acc)]

        def total_size(self):
            return len(self.values)

        def flat_index(self, indices):
            if len(indices) != len(self.dims):
                raise GBRuntimeError(
                    f"Array hat {len(self.dims)} Dimension(en), "
                    f"erhalten {len(indices)} Index/-e"
                )
            flat = 0
            for k, idx in enumerate(indices):
                if idx < 0 or idx >= self.dims[k]:
                    raise GBRuntimeError(
                        f"Index {idx} ausserhalb "
                        f"[0..{self.dims[k] - 1}] in Dimension {k}"
                    )
                flat += idx * self.strides[k]
            return flat

        def get_at(self, indices):
            return self.values[self.flat_index(indices)]

        def set_at(self, indices, value):
            self.values[self.flat_index(indices)] = value

        def __len__(self):
            return self.dims[0] if self.dims else 0

        def __repr__(self):
            shape = ",".join(str(d) for d in self.dims)
            return f"<ARRAY[{shape}] OF {self.element_type.upper()}>"


class _Image:
    """Wrapper um pygame.Surface - opaque fuer GameBasic."""
    __slots__ = ("surface", "path")

    def __init__(self, surface, path: str = ""):
        self.surface = surface
        self.path = path

    def __repr__(self):
        return f"<IMAGE {self.path or '?'}>"


class _Sound:
    """Wrapper um pygame.mixer.Sound."""
    __slots__ = ("sound", "path")

    def __init__(self, sound, path: str = ""):
        self.sound = sound
        self.path = path

    def __repr__(self):
        return f"<SOUND {self.path or '?'}>"


class _SpriteAtlas:
    """Sprite-Atlas: ein grosses Image-Asset plus benannte Sub-Rects.

    Wird via ATLAS_LOAD aus einem JSON-Manifest geladen. Sub-Rects sind
    in der frames-Dict gespeichert (name -> (x, y, w, h)) und werden von
    ATLAS_DRAW / BATCH_DRAW als Source-Rect fuer pygame-blits verwendet.

    Vorteil gegenueber N einzelnen LOADIMAGE: eine einzige Surface, eine
    einzige Allokation, perfekt fuer pygame.Surface.blits() (Batch).
    """
    __slots__ = ("image", "frames", "path")

    def __init__(self, image, frames: dict, path: str = ""):
        self.image = image          # _Image (haelt die pygame.Surface)
        self.frames = frames        # name -> (x, y, w, h)
        self.path = path

    def __repr__(self):
        return f"<SPRITE_ATLAS {self.path or '?'} frames={len(self.frames)}>"


class _GBFile:
    """Datei-Handle. Mode: 'r', 'w', 'a'."""
    __slots__ = ("handle", "path", "mode")

    def __init__(self, handle, path: str, mode: str):
        self.handle = handle
        self.path = path
        self.mode = mode

    def __repr__(self):
        return f"<FILE {self.path} mode={self.mode}>"


class _GBMap:
    """Hash-Map mit STRING-Schluesseln und typisierten Werten.

    value_type ist ein Type-String wie 'integer', 'float', 'string',
    'boolean', oder ein Klassenname.
    """
    __slots__ = ("value_type", "data")

    def __init__(self, value_type: str):
        self.value_type = value_type
        self.data: dict = {}

    def __len__(self):
        return len(self.data)

    def __repr__(self):
        return f"<MAP[{len(self.data)}] OF {self.value_type.upper()}>"


class _UserFunction:
    __slots__ = ("decl", "kind")  # kind: "sub" | "function"

    def __init__(self, decl, kind: str):
        self.decl = decl
        self.kind = kind


class _ClassInfo:
    __slots__ = ("name", "parent_name", "parent", "field_decls", "methods",
                 "is_struct", "properties")

    def __init__(self, name: str, parent_name, is_struct: bool = False):
        self.name = name
        self.parent_name = parent_name
        self.parent: "_ClassInfo | None" = None
        self.field_decls: list = []
        self.methods: dict = {}
        self.is_struct = is_struct
        # Property-Set: lower-case Property-Namen, fuer die ein Getter
        # ODER Setter existiert. Lookup im MemberAccess/Assign-Pfad
        # entscheidet, ob `obj.name` zu __get_name dispatcht.
        self.properties: set = set()


class _Instance:
    __slots__ = ("cls", "fields")

    def __init__(self, cls: _ClassInfo):
        self.cls = cls
        self.fields: dict = {}        # name -> {"type": str, "value": any}

    def __repr__(self):
        return f"<{self.cls.name}>"


class _FuncRef:
    """First-class Reference auf eine User-Function.

    Erlaubt Higher-Order-Patterns wie Sort-Comparator, Tween-Easing-
    Callbacks etc. Closures werden NICHT unterstuetzt -- der Body sieht
    nur Parameter und Globals (gleiche Regel im Tree-Walker und in beiden
    VMs).
    """
    __slots__ = ("name",)

    def __init__(self, name: str):
        self.name = name

    def __repr__(self):
        return f"<FUNCREF {self.name}>"


class _ClassStaticNamespace:
    """Container fuer Klassen-Statics (`STATIC CONST X = ...`).

    Wird unter dem Klassen-Namen als globale CONST abgelegt. MemberAccess
    `Player.MAX_HP` liefert den Wert. Werte sind Compile-Time-Literale --
    Strings, Numbers, Bools.

    Anders als `_EnumNamespace`: speichert beliebige Wert-Typen, nicht nur
    Integer. Member-Namen sind case-insensitive (lower-case-Vergleich).
    """
    __slots__ = ("name", "members")

    def __init__(self, name: str, members: dict):
        self.name = name
        self.members = members   # lower-name -> value

    def get(self, member: str):
        return self.members.get(member.lower())

    def __repr__(self):
        keys = ", ".join(self.members.keys())
        return f"<CLASS-STATICS {self.name}: {keys}>"


class _EnumNamespace:
    """Container fuer ENUM-Member.

    Wird als Wert einer global deklarierten Konstante abgelegt. Member-
    Access (`Color.RED`) wird vom Interpreter / VM erkannt und liefert
    den Member-Wert (Integer). Member-Namen werden lower-case verglichen
    (passend zur generellen Case-Insensitivity der Sprache).

    Nicht direkt vom GB-User instanziierbar; nur vom Compiler / Interpreter
    beim Verarbeiten eines EnumDecl angelegt.
    """
    __slots__ = ("name", "members", "names")

    def __init__(self, name: str, members: dict):
        self.name = name
        # member-name (lower) -> int
        self.members = members
        # int -> original-cased name (fuer Debug/Repr)
        self.names = {v: k for k, v in members.items()}

    def get(self, member: str):
        return self.members.get(member.lower())

    def __repr__(self):
        keys = ", ".join(self.members.keys())
        return f"<ENUM {self.name}: {keys}>"


# Method-Dispatch-Tabelle: (target_type, method_name) -> builtin_name.
# Erlaubt `arr.length()`, `s.upper()`, `m.has(k)` etc. -- dispatched zur
# Laufzeit zu den entsprechenden BUILTIN-Funktionen, mit dem Receiver als
# erstem Argument. method_name ist case-insensitive, BUILTIN-Lookup erfolgt
# im normalen BUILTINS-Dict.
CONTAINER_METHODS = {
    # String
    ("string", "upper"):    "upper$",
    ("string", "lower"):    "lower$",
    ("string", "length"):   "len",
    ("string", "len"):      "len",
    ("string", "trim"):     "trim$",
    ("string", "left"):     "left$",
    ("string", "right"):    "right$",
    ("string", "mid"):      "mid$",
    ("string", "indexof"):  "instr",
    ("string", "replace"):  "replace$",
    ("string", "split"):    "split$",
    ("string", "padl"):     "padl$",
    ("string", "padr"):     "padr$",
    # Array
    ("array", "length"):    "len",
    ("array", "len"):       "len",
    ("array", "sort"):      "sort",
    ("array", "reverse"):   "reverse",
    ("array", "indexof"):   "array_indexof",
    # Map
    ("map", "put"):         "mapput",
    ("map", "get"):         "mapget",
    ("map", "getor"):       "mapgetor",
    ("map", "has"):         "maphas",
    ("map", "keys"):        "mapkeys",
    ("map", "values"):      "mapvalues",
    ("map", "items"):       "mapitems",
    ("map", "size"):        "mapsize",
    ("map", "length"):      "mapsize",
    ("map", "len"):         "mapsize",
    ("map", "remove"):      "mapremove",
    ("map", "clear"):       "mapclear",
    # Tuple
    ("tuple", "length"):    "len",
    ("tuple", "len"):       "len",
}


def _container_kind(value) -> str:
    """Liefert den Container-Type-Namen fuer den Method-Dispatch, oder ''
    wenn der Wert kein Container ist."""
    if isinstance(value, str):
        return "string"
    if isinstance(value, tuple):
        return "tuple"
    if isinstance(value, _GBArray):
        return "array"
    if isinstance(value, _GBMap):
        return "map"
    return ""


def infer_type(value) -> str:
    """Kanonische Typ-Inferenz fuer untypisierte CONST (Single-Source fuer
    Tree-Walker, vm.py UND vm_native.pyx -- sonst divergieren die Pfade,
    welche Werte ein `CONST X = <expr>` ableiten kann). Deckt die Vereinigung
    aller Typen ab, die irgendein Pfad historisch kannte."""
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, int):
        return "integer"
    if isinstance(value, float):
        return "float"
    if isinstance(value, str):
        return "string"
    if isinstance(value, _Image):
        return "image"
    if isinstance(value, _Sound):
        return "sound"
    if isinstance(value, _SpriteAtlas):
        return "sprite_atlas"
    if isinstance(value, _GBFile):
        return "file"
    if isinstance(value, _GBArray):
        return f"array:{value.element_type}"
    if isinstance(value, _GBMap):
        return f"map:{value.value_type}"
    if isinstance(value, _Instance):
        return value.cls.name
    if isinstance(value, _EnumNamespace):
        return "enum"
    if isinstance(value, _ClassStaticNamespace):
        return "class_static"
    if isinstance(value, _FuncRef):
        return "funcref"
    raise GBRuntimeError("Typ kann nicht abgeleitet werden")


TYPE_DEFAULTS = {
    "integer": 0,
    "float": 0.0,
    "string": "",
    "boolean": False,
    "image": None,
    "sound": None,
    "sprite_atlas": None,
    "file": None,
    "tuple": (),
    "funcref": None,
}


class Interpreter:
    def __init__(self):
        # Dispatch-Caches: Node-Typ -> gebundene _eval_/_exec_-Methode.
        # Spart pro Node den f-String-Build + getattr (Hot-Path bei
        # expression-dichten Frames, z.B. Grafik-Demos).
        self._eval_cache: dict = {}
        self._exec_cache: dict = {}
        self.global_env = Environment()
        self.env = self.global_env
        self.functions: dict = {}
        self.classes: dict = {}
        self.call_depth = 0
        self._graphics = None  # lazy
        self._current_line = 0
        # DATA / READ / RESTORE State.  Beim Programmstart sammelt
        # _collect_data_values alle DATA-Literale in self.data; READ
        # liest sequenziell, RESTORE setzt den Pointer zurueck.
        self.data: list = []
        self.data_ptr: int = 0
        # Stack der gerade laufenden Methoden-Kontexte: jedes Element ist
        # ein (_Instance, _ClassInfo)-Tupel.  Wird in _invoke gepusht/gepoppt
        # und ermoeglicht zwei Bequemlichkeiten in Methoden-Bodies:
        #   - `Self` als Identifier liefert die aktuelle Instanz
        #   - bare `Method()` ruft eine Methode der eigenen Klasse auf
        self._method_stack: list = []
        self._register_constants()

    # ------------------------------------------------------------------
    def _register_constants(self):
        from .graphics import COLORS, KEYS
        for name, value in COLORS.items():
            self.global_env.declare(name, "integer", value)
        for name, value in KEYS.items():
            self.global_env.declare(name, "integer", value)
        # Math-Konstante (E wird absichtlich nicht registriert, weil 'e' ein
        # haeufiger CATCH-Variable-Name ist; nutze EXP(1) wenn noetig).
        self.global_env.declare("pi", "float", math.pi)
        self.global_env.get_slot("pi")["const"] = True

    def _get_graphics(self):
        if self._graphics is None:
            from .graphics import Graphics
            self._graphics = Graphics()
            self._graphics._gb_engine = self
        return self._graphics

    def gb_call_function(self, name, args):
        """Ruft eine User-Function per Name auf -- Bruecke fuer Builtin-
        Callbacks (z.B. GUI_ON_CLICK). `args` sind vorab-evaluierte Werte."""
        fn = self.functions.get(name)
        if fn is None:
            raise GBRuntimeError(f"FUNCREF: Funktion '{name}' existiert nicht (mehr)")
        return self._call_user(fn, list(args))

    # ------------------------------------------------------------------
    def run(self, program: Program):
        # Hoisting in zwei Phasen: erst Klassen, dann Funktionen.
        # So koennen Funktionen Klassen als Parametertypen referenzieren
        # und Methoden andere Klassen kennen, unabhaengig der Quelltext-Reihenfolge.
        main_stmts = []
        class_decls = []
        for stmt in program.statements:
            if isinstance(stmt, ClassDecl):
                class_decls.append(stmt)
            elif isinstance(stmt, SubDecl):
                self._register_function(stmt, "sub")
            elif isinstance(stmt, FunctionDecl):
                self._register_function(stmt, "function")
            else:
                main_stmts.append(stmt)
        for cd in class_decls:
            self._register_class(cd)
        for ci in self.classes.values():
            if ci.parent_name is not None:
                parent = self.classes.get(ci.parent_name)
                if parent is None:
                    raise GBRuntimeError(
                        f"CLASS '{ci.name}': Elternklasse '{ci.parent_name}' nicht gefunden"
                    )
                ci.parent = parent
        # Zyklus-Check
        for ci in self.classes.values():
            seen = set()
            cur = ci
            while cur:
                if cur.name in seen:
                    raise GBRuntimeError(
                        f"Vererbungs-Zyklus bei CLASS '{ci.name}'"
                    )
                seen.add(cur.name)
                cur = cur.parent
        # DATA-Werte aus dem gesamten Programm einsammeln (auch aus
        # SUB/FUNCTION-Bodies). Reihenfolge folgt der Source-Reihenfolge.
        self.data = []
        self.data_ptr = 0
        self._collect_data_values(program.statements)

        # Static-Class-Members: pro Klasse mit Statics einen Namespace bauen
        # und unter dem Klassen-Namen als globale CONST registrieren.
        # Werte muessen Compile-Zeit-Literale sein (Number/String/Bool oder
        # negierte Number). Komplexere Defaults sind nicht erlaubt -- gleiche
        # Strenge wie ENUM, damit alle drei Pfade sich gleich verhalten.
        for cd in class_decls:
            if not cd.statics:
                continue
            members: dict = {}
            for c in cd.statics:
                key = c.name.lower()
                if key in members:
                    raise GBRuntimeError(
                        f"CLASS {cd.name}: STATIC CONST '{c.name}' "
                        f"doppelt deklariert"
                    )
                value = self._eval_static_literal(c.value, cd.name, c.name)
                members[key] = value
            ns = _ClassStaticNamespace(cd.name, members)
            if self.global_env.has(cd.name):
                raise GBRuntimeError(
                    f"CLASS {cd.name}: Name ist bereits anderweitig vergeben "
                    f"(STATIC-Members brauchen freien Klassen-Namen im global Scope)"
                )
            self.global_env.declare(cd.name, "class_static", ns)
            self.global_env.get_slot(cd.name)["const"] = True

        try:
            for stmt in main_stmts:
                self._exec(stmt)
        finally:
            if self._graphics is not None:
                self._graphics.shutdown()

    def _collect_data_values(self, stmts: list):
        """Rekursiver Walk durch den AST: alle DATA-Werte in self.data sammeln."""
        for stmt in stmts:
            if isinstance(stmt, Data):
                for lit in stmt.values:
                    self.data.append(self._eval_data_literal(lit))
            # Recurse durch Container-Statements
            elif isinstance(stmt, If):
                self._collect_data_values(stmt.then_block)
                for _, blk in stmt.elseif_branches:
                    self._collect_data_values(blk)
                self._collect_data_values(stmt.else_block)
            elif isinstance(stmt, While):
                self._collect_data_values(stmt.body)
            elif isinstance(stmt, Repeat):
                self._collect_data_values(stmt.body)
            elif isinstance(stmt, For):
                self._collect_data_values(stmt.body)
            elif isinstance(stmt, Select):
                for case in stmt.cases:
                    blk = case[-1]   # case ist (matches, block) oder (matches, guard, block)
                    self._collect_data_values(blk)
                self._collect_data_values(stmt.else_block)
            elif isinstance(stmt, Try):
                self._collect_data_values(stmt.body)
                self._collect_data_values(stmt.catch_block)
            elif isinstance(stmt, SubDecl) or isinstance(stmt, FunctionDecl):
                self._collect_data_values(stmt.body)
            elif isinstance(stmt, ClassDecl):
                for m in stmt.methods:
                    self._collect_data_values(m.body)

    def _eval_static_literal(self, expr, cls_name: str, member_name: str):
        """STATIC-CONST-Wert auswerten: Number, String, Bool, oder negierte
        Number. Keine Ausdruecke -- damit alle drei Pfade konsistent sind.
        """
        if isinstance(expr, NumberLit):
            return expr.value
        if isinstance(expr, StringLit):
            return expr.value
        if isinstance(expr, BoolLit):
            return expr.value
        if isinstance(expr, UnaryOp) and expr.op == "-" and isinstance(expr.operand, NumberLit):
            return -expr.operand.value
        raise GBRuntimeError(
            f"CLASS {cls_name}.{member_name}: STATIC CONST muss ein Literal sein "
            f"(Number, String, Bool oder -Number) -- Ausdruecke werden zur "
            f"Compile-Zeit nicht ausgewertet"
        )

    def _eval_data_literal(self, lit):
        """Evaluiert ein DATA-Literal zur Compile/Load-Zeit."""
        if isinstance(lit, NumberLit):
            return lit.value
        if isinstance(lit, StringLit):
            return lit.value
        if isinstance(lit, BoolLit):
            return lit.value
        if isinstance(lit, UnaryOp) and lit.op == "-":
            inner = self._eval_data_literal(lit.operand)
            return -inner
        raise GBRuntimeError(
            f"Interner Fehler: ungueltiges DATA-Literal {type(lit).__name__}"
        )

    def _register_class(self, decl: ClassDecl):
        if decl.name in self.classes:
            kind = "STRUCT" if decl.is_struct else "CLASS"
            raise GBRuntimeError(f"{kind} '{decl.name}' bereits deklariert")
        ci = _ClassInfo(decl.name, decl.parent, is_struct=decl.is_struct)
        ci.field_decls = list(decl.fields)
        for m in decl.methods:
            if m.name in ci.methods:
                raise GBRuntimeError(
                    f"Methode '{m.name}' in CLASS '{decl.name}' bereits deklariert"
                )
            kind = "function" if isinstance(m, FunctionDecl) else "sub"
            ci.methods[m.name] = _UserFunction(m, kind)
        # Property-Namen sammeln (lowercase). Erlaubt schnellen MemberAccess-
        # und MemberAssign-Lookup. Dass __get/__set-Methoden existieren,
        # ist durch den Parser schon garantiert.
        for pd in (decl.properties or ()):
            ci.properties.add(pd.name.lower())
        self.classes[decl.name] = ci

    def _resolve_method(self, cls: _ClassInfo, name: str):
        cur = cls
        while cur is not None:
            if name in cur.methods:
                return cur.methods[name]
            cur = cur.parent
        return None

    def _is_subclass_of(self, child: _ClassInfo, parent: _ClassInfo) -> bool:
        cur = child
        while cur is not None:
            if cur is parent:
                return True
            cur = cur.parent
        return False

    def _allocate_instance(self, cls: _ClassInfo) -> _Instance:
        inst = _Instance(cls)
        chain = []
        cur = cls
        while cur is not None:
            chain.append(cur)
            cur = cur.parent
        for c in reversed(chain):
            for fd in c.field_decls:
                if fd.array_dims is not None:
                    # Array-Feld: bei der Instanziierung aus den Dims-Ausdruecken bauen.
                    dim_vals = []
                    for de in fd.array_dims:
                        dv = self._eval(de)
                        if isinstance(dv, bool) or not isinstance(dv, int) or dv < 0:
                            raise GBRuntimeError(
                                f"Array-Feld '{fd.name}': ungueltige Groesse"
                            )
                        dim_vals.append(dv)
                    arr = _GBArray(
                        fd.type_name, dim_vals,
                        lambda t=fd.type_name: self._element_default(t),
                    )
                    inst.fields[fd.name] = {
                        "type": f"array:{fd.type_name}", "value": arr,
                    }
                else:
                    # Skalar-Feld; STRUCT-Felder werden auto-init.
                    sub_cls = self.classes.get(fd.type_name)
                    if sub_cls is not None and sub_cls.is_struct:
                        default = self._allocate_instance(sub_cls)
                    else:
                        default = self._default_for(fd.type_name)
                    inst.fields[fd.name] = {
                        "type": fd.type_name, "value": default,
                    }
        return inst

    def _default_for(self, type_name: str):
        if type_name in TYPE_DEFAULTS:
            return TYPE_DEFAULTS[type_name]
        if type_name in self.classes:
            return None  # NIL-Referenz
        if type_name.startswith("map:"):
            return _GBMap(type_name[4:])  # auto-init leere Map
        # Unbekannter Typ -> wir lassen es zur Laufzeit beim Zugriff auffliegen.
        return None

    def _register_function(self, decl, kind: str):
        name = decl.name
        if name in self.functions:
            raise GBRuntimeError(f"{kind.upper()} '{name}' bereits deklariert")
        self.functions[name] = _UserFunction(decl, kind)

    # ---- Statements --------------------------------------------------
    def _exec(self, stmt):
        line = getattr(stmt, "line", 0)
        if line:
            self._current_line = line
        t = type(stmt)
        method = self._exec_cache.get(t)
        if method is None:
            method = getattr(self, f"_exec_{t.__name__}", None)
            if method is None:
                raise GBRuntimeError(f"Unbekanntes Statement: {t.__name__}",
                                     self._current_line)
            self._exec_cache[t] = method
        try:
            method(stmt)
        except GameBasicError as exc:
            if not getattr(exc, "line", 0):
                exc.line = self._current_line
                exc.args = (exc._format(),)
            raise

    def _exec_Dim(self, s: Dim):
        # Bekannter Typ? (Primitiv, Klasse/Struct, ARRAY OF ..., MAP OF ...,
        # oder externer Typ aus Built-in-Modul.)
        from .modules import EXTERNAL_TYPES as _EXT_TYPES
        is_array_type = s.type_name.startswith("array:")
        is_map_type = s.type_name.startswith("map:")
        if (s.type_name not in TYPE_DEFAULTS
                and s.type_name not in self.classes
                and s.type_name not in _EXT_TYPES
                and not is_array_type
                and not is_map_type):
            raise GBRuntimeError(f"Unbekannter Typ '{s.type_name}' bei DIM {s.name}")

        # Multi-dim Array per DIM x[a, b, ...] AS T
        if s.array_dims is not None:
            dim_vals = []
            for de in s.array_dims:
                dv = self._eval(de)
                if isinstance(dv, bool) or not isinstance(dv, int):
                    raise TypeMismatchError(
                        f"Array-Groesse muss INTEGER sein, erhalten {self._type_of(dv)}"
                    )
                if dv < 0:
                    raise GBRuntimeError(f"Array-Groesse darf nicht negativ sein ({dv})")
                dim_vals.append(dv)
            arr = _GBArray(
                s.type_name, dim_vals,
                lambda t=s.type_name: self._element_default(t),
            )
            self.env.declare(s.name, f"array:{s.type_name}", arr)
            return

        # ARRAY OF T ohne Groesse: nicht-zugewiesener Slot (NIL bis spaeterer Zuweisung)
        if is_array_type:
            self.env.declare(s.name, s.type_name, None)
            return

        # STRUCT: automatisch instanziieren (Wert-Semantik in der Initialisierung).
        cls = self.classes.get(s.type_name)
        if cls is not None and cls.is_struct:
            self.env.declare(s.name, s.type_name, self._allocate_instance(cls))
            return

        # Skalar (auch CLASS-Variable startet mit NIL)
        self.env.declare(s.name, s.type_name, self._default_for(s.type_name))

    def _element_default(self, type_name: str):
        """Element-Standardwert fuer Array-Slots: STRUCTs werden auto-allokiert."""
        cls = self.classes.get(type_name)
        if cls is not None and cls.is_struct:
            return self._allocate_instance(cls)
        return self._default_for(type_name)

    def _exec_MultiDim(self, s: MultiDim):
        """Mehrere DIMs gleichen Typs nacheinander ausfuehren."""
        for d in s.dims:
            self._exec_Dim(d)

    def _exec_Const(self, s: Const):
        value = self._eval(s.value)
        if s.type_name is None:
            type_name = self._infer_type(value, f"CONST {s.name}")
        else:
            type_name = s.type_name
            if type_name not in TYPE_DEFAULTS and type_name not in self.classes:
                raise GBRuntimeError(f"Unbekannter Typ '{type_name}' bei CONST {s.name}")
            value = self._coerce(value, type_name, f"CONST {s.name}")
        # Wenn eingebauter CONST mit gleichem Typ und Wert vorhanden -> idempotent
        if self.env.has(s.name):
            existing = self.env.get_slot(s.name)
            if existing.get("const") and existing["type"] == type_name and existing["value"] == value:
                return
        self.env.declare(s.name, type_name, value)
        slot = self.env.get_slot(s.name)
        slot["const"] = True

    def _exec_EnumDecl(self, s: EnumDecl):
        """Wertet alle Member-Werte aus und legt das Enum als globale,
        konstante Variable ab. Implizite Werte: 0, 1, 2, ... (oder
        last_explicit + 1, wenn vorher ein expliziter Wert kam).

        Member-Werte muessen Compile-Time-Integer-Literale sein (auch im
        Tree-Walker - so bleibt das Verhalten zwischen den drei Pfaden
        gleich). Komplexere Werte hat man besser ueber CONST + INTEGER
        und manuelle Konstanten."""
        from .compiler import _eval_int_literal
        members: dict = {}
        next_auto = 0
        for mname, expr in s.members:
            if expr is None:
                value = next_auto
            else:
                value = _eval_int_literal(expr)
                if value is None:
                    raise GBRuntimeError(
                        f"ENUM {s.name}.{mname}: Wert muss ein Integer-Literal "
                        f"sein (z.B. 5 oder -1)"
                    )
            key = mname.lower()
            members[key] = value
            next_auto = value + 1
        ns = _EnumNamespace(s.name, members)
        # Existiert ein Symbol mit gleichem Namen schon, werfen wir - das
        # Programm hat sonst zwei Quellen der Wahrheit.
        if self.global_env.has(s.name):
            existing = self.global_env.get_slot(s.name)
            if not (existing.get("const") and isinstance(existing["value"], _EnumNamespace)
                    and existing["value"].members == members):
                raise GBRuntimeError(
                    f"ENUM {s.name}: Name ist bereits anderweitig vergeben"
                )
            return
        self.global_env.declare(s.name, "enum", ns)
        slot = self.global_env.get_slot(s.name)
        slot["const"] = True

    def _exec_Break(self, s: Break):
        raise _BreakSignal()

    def _exec_Continue(self, s: Continue):
        raise _ContinueSignal()

    def _exec_Throw(self, s: Throw):
        v = self._eval(s.value)
        msg = v if isinstance(v, str) else self._fmt(v)
        raise _GBThrow(msg)

    def _exec_Try(self, s: Try):
        try:
            for st in s.body:
                self._exec(st)
        except _GBThrow as exc:
            self._handle_catch(s, exc.value)
        except (GBRuntimeError, TypeMismatchError) as exc:
            self._handle_catch(s, exc.message if hasattr(exc, "message") else str(exc))

    def _handle_catch(self, s: Try, msg: str):
        if s.catch_var:
            if not self.env.has(s.catch_var):
                self.env.declare(s.catch_var, "string", "")
            slot = self.env.get_slot(s.catch_var)
            if slot.get("const"):
                raise GBRuntimeError(
                    f"CATCH-Variable '{s.catch_var}' ist CONST"
                )
            if slot["type"] != "string":
                raise GBRuntimeError(
                    f"CATCH-Variable '{s.catch_var}' muss STRING sein"
                )
            slot["value"] = msg
        for st in s.catch_block:
            self._exec(st)

    def _exec_IndexAssign(self, s: IndexAssign):
        arr = self._eval(s.target)
        if arr is None:
            raise GBRuntimeError("Index-Zuweisung an NIL-Referenz")
        if not isinstance(arr, _GBArray):
            raise GBRuntimeError(
                f"Index-Zuweisung an Nicht-Array ({self._type_of(arr)})"
            )
        idx_vals = []
        for ie in s.indices:
            idx = self._eval(ie)
            if isinstance(idx, bool) or not isinstance(idx, int):
                raise TypeMismatchError(
                    f"Array-Index muss INTEGER sein, erhalten {self._type_of(idx)}"
                )
            idx_vals.append(idx)
        flat = arr.flat_index(idx_vals)
        value = self._eval(s.value)
        idx_str = ",".join(str(i) for i in idx_vals)
        arr.values[flat] = self._coerce(
            value, arr.element_type, f"Array-Element [{idx_str}]"
        )

    def _exec_ClassDecl(self, s: ClassDecl):
        # Wird normalerweise per Hoisting in run() registriert.
        # Falls ein CLASS innerhalb eines Blocks auftaucht, ignorieren.
        pass

    def _exec_MemberAssign(self, s: MemberAssign):
        obj = self._eval(s.target)
        if obj is None:
            raise GBRuntimeError(f"Zuweisung an '.{s.name}' bei NIL-Referenz")
        if not isinstance(obj, _Instance):
            raise GBRuntimeError(
                f"Zuweisung an '.{s.name}' bei nicht-Objekt ({self._type_of(obj)})"
            )
        # Property-Setter: wenn `s.name` eine Property ist, dispatch zu
        # `__set_<name>`.
        if self._is_property(obj.cls, s.name):
            setter = self._resolve_method(obj.cls, f"__set_{s.name.lower()}")
            if setter is None:
                raise GBRuntimeError(
                    f"Property '{s.name}' in {obj.cls.name} hat keinen Setter "
                    f"(nur GET deklariert -- Property ist read-only)"
                )
            value = self._eval(s.value)
            self._call_method(obj, setter, [value])
            return
        if s.name not in obj.fields:
            raise GBRuntimeError(
                f"Feld '{s.name}' existiert nicht in {obj.cls.name}"
            )
        slot = obj.fields[s.name]
        value = self._eval(s.value)
        slot["value"] = self._coerce(
            value, slot["type"], f"Zuweisung an {obj.cls.name}.{s.name}"
        )

    def _exec_Assign(self, s: Assign):
        slot = self.env.get_slot(s.name)
        if slot.get("const"):
            raise GBRuntimeError(f"CONST '{s.name}' kann nicht ueberschrieben werden")
        value = self._eval(s.value)
        slot["value"] = self._coerce(value, slot["type"], f"Zuweisung an '{s.name}'")

    def _exec_With(self, s):
        # WITH-Ziel einmal evaluieren, in Compiler-generierter Variable
        # speichern. `.member`-Shortcuts im Body sind im Parser bereits zu
        # `MemberAccess(Identifier(var_name), name)` desugared, deshalb muss
        # der Name im aktuellen Scope auffindbar sein.
        target_val = self._eval(s.target)
        # Nicht ueber env.declare -- das wuerde ein potentiell vorhandenes
        # gleichnamiges Slot ueberschreiben. WITH-Vars sind synthetisch und
        # garantiert eindeutig (`__with_<n>`), also einfach reinsetzen.
        self.env.vars[s.var_name] = {"type": "any", "value": target_val}
        try:
            for st in s.body:
                self._exec(st)
        finally:
            # Slot wieder entfernen, damit verschachtelte WITHs sich nicht
            # gegenseitig sehen und garbage-collect schneller passiert.
            self.env.vars.pop(s.var_name, None)

    def _exec_TupleAssign(self, s):
        value = self._eval(s.value)
        if not isinstance(value, tuple):
            raise TypeMismatchError(
                f"Tupel-Destructuring: Erwartet TUPLE, erhalten {self._type_of(value)}"
            )
        if len(value) != len(s.targets):
            raise GBRuntimeError(
                f"Tupel-Destructuring: {len(s.targets)} Ziele, aber Tupel hat "
                f"{len(value)} Element(e)"
            )
        for tgt, v in zip(s.targets, value):
            self._assign_to_lvalue(tgt, v)

    def _assign_to_lvalue(self, target, value):
        """Weist `value` einem beliebigen Assignment-Pfad zu (Identifier,
        MemberAccess, IndexAccess) -- shared zwischen TupleAssign und
        zukuenftigen Multi-Assign-Faellen.
        """
        if isinstance(target, Identifier):
            slot = self.env.get_slot(target.name)
            if slot.get("const"):
                raise GBRuntimeError(f"CONST '{target.name}' kann nicht ueberschrieben werden")
            slot["value"] = self._coerce(value, slot["type"], f"Zuweisung an '{target.name}'")
            return
        if isinstance(target, MemberAccess):
            obj = self._eval(target.target)
            if obj is None or not isinstance(obj, _Instance):
                raise GBRuntimeError(
                    f"Zuweisung an '.{target.name}' bei nicht-Objekt"
                )
            if target.name not in obj.fields:
                raise GBRuntimeError(
                    f"Feld '{target.name}' existiert nicht in {obj.cls.name}"
                )
            slot = obj.fields[target.name]
            slot["value"] = self._coerce(
                value, slot["type"], f"Zuweisung an {obj.cls.name}.{target.name}"
            )
            return
        if isinstance(target, IndexAccess):
            arr = self._eval(target.target)
            if not isinstance(arr, _GBArray):
                raise GBRuntimeError(
                    f"Index-Zuweisung an Nicht-Array ({self._type_of(arr)})"
                )
            idx_vals = []
            for ie in target.indices:
                idx = self._eval(ie)
                if isinstance(idx, bool) or not isinstance(idx, int):
                    raise TypeMismatchError(
                        f"Array-Index muss INTEGER sein, erhalten {self._type_of(idx)}"
                    )
                idx_vals.append(idx)
            flat = arr.flat_index(idx_vals)
            arr.values[flat] = self._coerce(
                value, arr.element_type, f"Array-Element"
            )
            return
        raise GBRuntimeError(f"Ungueltiges Assignment-Ziel: {type(target).__name__}")

    def _exec_Print(self, s: Print):
        parts = [self._fmt(self._eval(it)) for it in s.items]
        print(" ".join(parts))

    def _exec_Input(self, s: Input):
        prompt = ""
        if s.prompt is not None:
            prompt = self._eval(s.prompt) or ""
        if not prompt:
            prompt = "? "
        elif not prompt.endswith(" "):
            prompt = prompt + " "
        # Prompt explizit flushen - Python's input(prompt) puffert das in
        # bestimmten Setups (z.B. Subprocess mit PIPE-stdout), wodurch der
        # Prompt erst NACH der Eingabe in der Konsole erscheint.
        import sys as _sys
        try:
            _sys.stdout.write(prompt)
            _sys.stdout.flush()
        except Exception:
            pass
        try:
            raw = input()
        except EOFError:
            raw = ""
        slot = self.env.get_slot(s.target)
        if slot.get("const"):
            raise GBRuntimeError(f"CONST '{s.target}' kann nicht ueberschrieben werden")
        try:
            if slot["type"] == "integer":
                v = int(raw.strip())
            elif slot["type"] == "float":
                v = float(raw.strip())
            elif slot["type"] == "string":
                v = raw
            elif slot["type"] == "boolean":
                v = raw.strip().lower() in ("true", "wahr", "yes", "ja", "1")
            else:
                raise GBRuntimeError(f"Unbekannter Typ: {slot['type']}")
        except ValueError:
            raise GBRuntimeError(
                f"Eingabe '{raw}' passt nicht zu {slot['type'].upper()}"
            )
        slot["value"] = v

    def _exec_If(self, s: If):
        if self._truthy(self._eval(s.condition)):
            for st in s.then_block:
                self._exec(st)
            return
        for cond, block in s.elseif_branches:
            if self._truthy(self._eval(cond)):
                for st in block:
                    self._exec(st)
                return
        for st in s.else_block:
            self._exec(st)

    def _exec_Select(self, s: Select):
        subject = self._eval(s.subject)
        for case in s.cases:
            # Backward-Compat: alte (matches, block) ohne Guard akzeptieren.
            if len(case) == 3:
                matches, guard, block = case
            else:
                matches, block = case
                guard = None
            if not self._select_matches(subject, matches):
                continue
            if guard is not None and not self._truthy(self._eval(guard)):
                continue
            for st in block:
                self._exec(st)
            return
        for st in s.else_block:
            self._exec(st)

    def _select_matches(self, subject, matches: list) -> bool:
        for m in matches:
            if m.kind == "value":
                if self._values_equal(subject, self._eval(m.values[0])):
                    return True
            elif m.kind == "range":
                lo = self._eval(m.values[0])
                hi = self._eval(m.values[1])
                if self._in_range(subject, lo, hi):
                    return True
            elif m.kind == "is":
                op = m.values[0]
                rhs = self._eval(m.values[1])
                if self._compare(subject, op, rhs):
                    return True
            else:
                raise GBRuntimeError(
                    f"Interner Fehler: unbekannter CASE-Match-Typ '{m.kind}'"
                )
        return False

    def _compare(self, a, op: str, b) -> bool:
        if op == "=":
            return self._values_equal(a, b)
        if op == "<>":
            return not self._values_equal(a, b)
        # Ordnungsvergleiche: Bool-Sonderfall raus, dann Standard.
        if isinstance(a, bool) or isinstance(b, bool):
            raise TypeMismatchError(
                f"CASE IS {op}: BOOLEAN nicht ordnungsvergleichbar"
            )
        try:
            if op == "<":  return a < b
            if op == ">":  return a > b
            if op == "<=": return a <= b
            if op == ">=": return a >= b
        except TypeError as exc:
            raise TypeMismatchError(f"CASE IS {op}: Typen nicht vergleichbar ({exc})")
        raise GBRuntimeError(f"Interner Fehler: unbekannter Operator '{op}'")

    def _values_equal(self, a, b) -> bool:
        # Strikter Vergleich, lehnt aber bool=int gleichsetzungen ab.
        if isinstance(a, bool) != isinstance(b, bool):
            return False
        return a == b

    def _in_range(self, v, lo, hi) -> bool:
        if isinstance(v, bool) or not isinstance(v, (int, float, str)):
            raise TypeMismatchError(
                f"CASE TO: Subject muss Zahl oder STRING sein, "
                f"erhalten {type(v).__name__}"
            )
        if type(lo) != type(hi) and not (
            isinstance(lo, (int, float)) and isinstance(hi, (int, float))
        ):
            raise TypeMismatchError(
                "CASE TO: Bereichsgrenzen muessen denselben Typ haben"
            )
        return lo <= v <= hi

    def _exec_While(self, s: While):
        while self._truthy(self._eval(s.condition)):
            try:
                for st in s.body:
                    self._exec(st)
            except _ContinueSignal:
                continue
            except _BreakSignal:
                return

    def _exec_For(self, s: For):
        start = self._eval(s.start)
        end = self._eval(s.end)
        step = self._eval(s.step) if s.step is not None else 1
        if step == 0:
            raise GBRuntimeError("STEP darf nicht 0 sein")

        if not self.env.has(s.var):
            type_name = "float" if any(isinstance(v, float) for v in (start, end, step)) else "integer"
            self.env.declare(s.var, type_name, TYPE_DEFAULTS[type_name])
        slot = self.env.get_slot(s.var)
        if slot.get("const"):
            raise GBRuntimeError(f"FOR-Variable '{s.var}' ist CONST")
        slot["value"] = self._coerce(start, slot["type"], f"FOR-Variable '{s.var}'")

        while True:
            cur = slot["value"]
            if step > 0 and cur > end:
                break
            if step < 0 and cur < end:
                break
            try:
                for st in s.body:
                    self._exec(st)
            except _ContinueSignal:
                pass  # faellt durch zum Schritt
            except _BreakSignal:
                return
            slot["value"] = self._coerce(
                slot["value"] + step, slot["type"], f"FOR-Variable '{s.var}'"
            )

    def _exec_ForEach(self, s):
        items = self._iter_for_comp(self._eval(s.iterable))
        if not self.env.has(s.var):
            self.env.declare(s.var, "any", None)
        slot = self.env.get_slot(s.var)
        if slot.get("const"):
            raise GBRuntimeError(f"FOR-EACH-Variable '{s.var}' ist CONST")
        slot["type"] = "any"
        for item in items:
            slot["value"] = item
            try:
                for st in s.body:
                    self._exec(st)
            except _ContinueSignal:
                pass
            except _BreakSignal:
                return

    def _exec_Repeat(self, s: Repeat):
        """REPEAT body UNTIL cond - laeuft mindestens einmal,
        wiederholt solange cond falsch ist."""
        while True:
            try:
                for st in s.body:
                    self._exec(st)
            except _ContinueSignal:
                pass  # weiter zur Bedingung
            except _BreakSignal:
                return
            if self._truthy(self._eval(s.condition)):
                return

    def _exec_Data(self, s: Data):
        # DATA-Werte werden im Pre-Pass gesammelt - zur Laufzeit nichts zu tun.
        pass

    def _exec_Read(self, s: Read):
        for target in s.targets:
            if self.data_ptr >= len(self.data):
                raise GBRuntimeError(
                    "READ: keine DATA-Werte mehr (benutze RESTORE zum Reset)"
                )
            value = self.data[self.data_ptr]
            self.data_ptr += 1
            self._read_assign(target, value)

    def _exec_Restore(self, s: Restore):
        self.data_ptr = 0

    def _read_assign(self, target, value):
        """Weist `value` einem READ-Ziel zu. Unterstuetzt Identifier,
        IndexAccess, MemberAccess - alles was auch normale Zuweisungen tun."""
        if isinstance(target, Identifier):
            slot = self.env.get_slot(target.name)
            if slot.get("const"):
                raise GBRuntimeError(
                    f"READ: '{target.name}' ist CONST"
                )
            slot["value"] = self._coerce(
                value, slot["type"], f"READ '{target.name}'"
            )
            return
        if isinstance(target, IndexAccess):
            arr = self._eval(target.target)
            if not isinstance(arr, _GBArray):
                raise GBRuntimeError("READ: Index-Ziel ist kein ARRAY")
            indices = [self._eval(ie) for ie in target.indices]
            i = arr.flat_index(indices)
            arr.values[i] = self._coerce(
                value, arr.element_type, "READ in Array"
            )
            return
        if isinstance(target, MemberAccess):
            obj = self._eval(target.target)
            if not isinstance(obj, _Instance):
                raise GBRuntimeError("READ: Member-Ziel ist kein Objekt")
            field = obj.class_info.field(target.name)
            if field is None:
                raise GBRuntimeError(
                    f"READ: Feld '{target.name}' nicht gefunden"
                )
            obj.fields[target.name]["value"] = self._coerce(
                value, field.type_name, f"READ '.{target.name}'"
            )
            return
        raise GBRuntimeError(
            f"READ: ungueltiges Ziel ({type(target).__name__})"
        )

    def _exec_ExprStmt(self, s: ExprStmt):
        self._eval(s.expr)

    def _exec_SubDecl(self, s: SubDecl):
        # Auch zur Laufzeit erlaubt (z.B. nach IF), nicht nur top-level.
        self._register_function(s, "sub")

    def _exec_FunctionDecl(self, s: FunctionDecl):
        self._register_function(s, "function")

    def _exec_Return(self, s: Return):
        if self.call_depth == 0:
            raise GBRuntimeError("RETURN nur innerhalb SUB/FUNCTION erlaubt")
        value = None if s.value is None else self._eval(s.value)
        raise _ReturnSignal(value)

    # ---- Ausdruecke --------------------------------------------------
    def _eval(self, expr):
        t = type(expr)
        method = self._eval_cache.get(t)
        if method is None:
            method = getattr(self, f"_eval_{t.__name__}", None)
            if method is None:
                raise GBRuntimeError(f"Unbekannter Ausdruck: {t.__name__}")
            self._eval_cache[t] = method
        return method(expr)

    def _eval_NumberLit(self, e: NumberLit):
        return e.value

    def _eval_StringLit(self, e: StringLit):
        return e.value

    def _eval_BoolLit(self, e: BoolLit):
        return e.value

    def _eval_TupleLit(self, e):
        # Wertsemantik = Python-tuple, immutable. `(1, 2, 3)` -> (1, 2, 3).
        return tuple(self._eval(x) for x in e.elements)

    def _eval_ListComp(self, e):
        """List-Comprehension liefert ein TUPLE der transformierten Werte."""
        iterable = self._eval(e.iterable)
        items = self._iter_for_comp(iterable)
        # Iter-Var lebt im aktuellen Scope -- temporaer registriert, damit
        # transform/filter die Variable sehen. Konflikt mit existierender
        # Variable: wir merken den vorherigen Wert und stellen ihn wieder
        # her (klassisches Save/Restore).
        had_var = self.env.has(e.var)
        prev_slot = None
        if had_var:
            prev_slot = dict(self.env.get_slot(e.var))   # Kopie
        result: list = []
        try:
            # Als "any"-Type registrieren -- Iter-Var akzeptiert beliebige
            # Container-Element-Typen.
            self.env.declare(e.var, "any", None) if not had_var else None
            slot = self.env.get_slot(e.var)
            slot["type"] = "any"
            for item in items:
                slot["value"] = item
                if e.filter is not None and not self._truthy(self._eval(e.filter)):
                    continue
                result.append(self._eval(e.transform))
        finally:
            if had_var:
                # Originalen Slot wiederherstellen
                self.env.vars[e.var].update(prev_slot)
            else:
                # Iter-Var war neu -- entfernen
                self.env.vars.pop(e.var, None)
        return tuple(result)

    def _eval_DictComp(self, e):
        """Dict-Comprehension liefert ein _GBMap. Keys muessen STRING sein
        (GameBasic-MAP-Konvention); der Value-Type wird vom ersten
        Eintrag inferred."""
        iterable = self._eval(e.iterable)
        items = self._iter_for_comp(iterable)
        had_var = self.env.has(e.var)
        prev_slot = dict(self.env.get_slot(e.var)) if had_var else None
        result_pairs: list = []
        try:
            if not had_var:
                self.env.declare(e.var, "any", None)
            slot = self.env.get_slot(e.var)
            slot["type"] = "any"
            for item in items:
                slot["value"] = item
                if e.filter is not None and not self._truthy(self._eval(e.filter)):
                    continue
                k = self._eval(e.key)
                if not isinstance(k, str):
                    raise TypeMismatchError(
                        f"Dict-Comprehension: Key muss STRING sein, "
                        f"erhalten {self._type_of(k)}"
                    )
                v = self._eval(e.value)
                result_pairs.append((k, v))
        finally:
            if had_var:
                self.env.vars[e.var].update(prev_slot)
            else:
                self.env.vars.pop(e.var, None)
        # Value-Typ inferred aus dem ersten Eintrag (oder "any" wenn leer).
        if result_pairs:
            value_type = self._type_of(result_pairs[0][1]).lower()
        else:
            value_type = "any"
        m = _GBMap(value_type)
        for k, v in result_pairs:
            m.data[k] = v
        return m

    def _eval_SetComp(self, e):
        """Set-Comprehension: liefert ein TUPLE mit deduplizierten Werten
        in der Reihenfolge des ersten Auftretens. GameBasic hat keinen
        echten SET-Typ -- das ist die pragmatische Naeherung."""
        iterable = self._eval(e.iterable)
        items = self._iter_for_comp(iterable)
        had_var = self.env.has(e.var)
        prev_slot = dict(self.env.get_slot(e.var)) if had_var else None
        seen: list = []     # nutzen Liste statt set, damit unhashable values gehen
        result: list = []
        try:
            if not had_var:
                self.env.declare(e.var, "any", None)
            slot = self.env.get_slot(e.var)
            slot["type"] = "any"
            for item in items:
                slot["value"] = item
                if e.filter is not None and not self._truthy(self._eval(e.filter)):
                    continue
                v = self._eval(e.transform)
                if v not in seen:
                    seen.append(v)
                    result.append(v)
        finally:
            if had_var:
                self.env.vars[e.var].update(prev_slot)
            else:
                self.env.vars.pop(e.var, None)
        return tuple(result)

    def _iter_for_comp(self, container):
        """Liefert ein Python-Iterable fuer Comprehension-Quellen."""
        if isinstance(container, str):
            return list(container)
        if isinstance(container, tuple):
            return list(container)
        if isinstance(container, _GBArray):
            if len(container.dims) != 1:
                raise GBRuntimeError("Comprehension: nur 1D-Arrays unterstuetzt")
            return list(container.values)
        if isinstance(container, _GBMap):
            return list(container.data.keys())
        raise TypeMismatchError(
            f"Comprehension: nicht iterierbar ({self._type_of(container)})"
        )

    def _eval_Identifier(self, e: Identifier):
        # `Self` (case-insensitive) in einer Methode liefert die aktuelle
        # Instanz.  Hat ein User eine eigene Variable namens `self` deklariert,
        # gewinnt dessen Variable - die Spezial-Behandlung greift nur, wenn
        # nichts mit dem Namen im Scope steht.
        if e.name == "self" and self._method_stack and not self.env.has("self"):
            return self._method_stack[-1][0]
        # Bare User-Function-Identifier in Expression-Position: liefere eine
        # FuncRef. Variablen haben Vorrang -- wer eine Variable mit dem
        # gleichen Namen wie eine Function deklariert, sieht die Variable.
        if not self.env.has(e.name) and e.name in self.functions:
            return _FuncRef(e.name)
        return self.env.get(e.name)

    def _eval_TernaryExpr(self, e):
        # Lazy: nur der gewaehlte Zweig wird ausgewertet (Short-Circuit).
        if self._truthy(self._eval(e.cond)):
            return self._eval(e.then_expr)
        return self._eval(e.else_expr)

    def _eval_BinaryOp(self, e: BinaryOp):
        op = e.op
        if op == "and":
            left = self._eval(e.left)
            if not self._truthy(left):
                return False
            return self._truthy(self._eval(e.right))
        if op == "or":
            left = self._eval(e.left)
            if self._truthy(left):
                return True
            return self._truthy(self._eval(e.right))

        left = self._eval(e.left)
        right = self._eval(e.right)

        # Modul-registriertes Operator-Overloading (Vec2 etc.). Liefert
        # NO_OP_MATCH, wenn kein Modul fuer einen der Operanden-Typen
        # registriert ist -- dann faellt's zu Standard-Dispatch durch.
        from .modules import dispatch_binary_op, NO_OP_MATCH
        result = dispatch_binary_op(op, left, right)
        if result is not NO_OP_MATCH:
            return result

        # User-Class Operator-Overloading: wenn `left` eine Instanz mit
        # einer `__op_<name>__`-Methode ist, ruf sie. RHS wird genauso
        # versucht (Reverse-Dispatch), damit `5 + money` funktioniert,
        # wenn Money `__op_add__(other AS INTEGER)` definiert.
        op_method = _USER_OP_METHODS.get(op)
        if op_method is not None:
            if isinstance(left, _Instance):
                m = self._resolve_method(left.cls, op_method)
                if m is not None:
                    return self._invoke(m, [right], left, arg_exprs=[e.right])
            if isinstance(right, _Instance):
                m = self._resolve_method(right.cls, op_method)
                if m is not None:
                    return self._invoke(m, [left], right, arg_exprs=[e.left])

        if op == "+":
            if isinstance(left, str) or isinstance(right, str):
                return self._fmt(left) + self._fmt(right)
            self._require_number(left, right, "+")
            return left + right
        if op == "-":
            self._require_number(left, right, "-")
            return left - right
        if op == "*":
            # String-Repetition: "abc" * 3 -> "abcabcabc". Negative oder
            # Null-Counts liefern den leeren String. Nur strikt INTEGER --
            # Float oder Bool werden abgelehnt.
            if isinstance(left, str) and isinstance(right, int) and not isinstance(right, bool):
                return left * right if right > 0 else ""
            if isinstance(right, str) and isinstance(left, int) and not isinstance(left, bool):
                return right * left if left > 0 else ""
            self._require_number(left, right, "*")
            return left * right
        if op == "/":
            self._require_number(left, right, "/")
            if right == 0:
                raise GBRuntimeError("Division durch 0")
            if isinstance(left, int) and isinstance(right, int) and left % right == 0:
                return left // right
            return left / right
        if op == "\\":
            if isinstance(left, bool) or isinstance(right, bool):
                raise TypeMismatchError("\\ erwartet INTEGER")
            if not isinstance(left, int) or not isinstance(right, int):
                raise TypeMismatchError("\\ erwartet INTEGER (kein FLOAT)")
            if right == 0:
                raise GBRuntimeError("Integer-Division durch 0")
            # Truncation gegen 0 (klassisches BASIC-Verhalten)
            q, r = divmod(left, right)
            if r != 0 and (left < 0) != (right < 0):
                q += 1
            return q
        if op == "mod":
            self._require_number(left, right, "MOD")
            if right == 0:
                raise GBRuntimeError("MOD durch 0")
            return left % right
        if op == "^":
            self._require_number(left, right, "^")
            return left ** right
        if op in ("band", "bor", "bxor", "shl", "shr"):
            self._require_int_pair(left, right, op)
            if op == "band":
                return left & right
            if op == "bor":
                return left | right
            if op == "bxor":
                return left ^ right
            # Shift-Counts muessen nicht-negativ sein. Negative Shifts werden
            # geworfen statt zu Python-Verhalten zu fallen (das wirft auch,
            # aber mit einem nichtssagenden ValueError) - so kriegt der User
            # eine GB-Fehlermeldung.
            if right < 0:
                raise GBRuntimeError(f"{op.upper()}: Shift-Anzahl darf nicht negativ sein")
            if op == "shl":
                return left << right
            return left >> right
        # Comparison-Operatoren einzeln evaluieren -- das alte dict-Pattern
        # rechnete ALLE Vergleiche, was bei Typen ohne Ordnung (z.B. _Vec2)
        # auch fuer `=`/`<>` einen TypeError auf `<` geworfen haette.
        if op == "=":
            return left == right
        if op == "<>":
            return left != right
        if op == "<":
            return left < right
        if op == ">":
            return left > right
        if op == "<=":
            return left <= right
        if op == ">=":
            return left >= right
        if op == "in":
            return self._eval_in(left, right)
        raise GBRuntimeError(f"Unbekannter Operator: {op}")

    def _eval_in(self, needle, haystack):
        """`needle IN haystack` -- pruefe Mitgliedschaft. Funktioniert auf:
            - String IN String  -> Substring-Test (`"foo" IN "barfoo"`)
            - X IN Tuple        -> Element-Test (`5 IN (1, 5, 9)`)
            - X IN Array        -> Element-Test (1D, beliebiger Element-Type)
            - String IN Map     -> Key-Test (Maps haben STRING-Keys in GB)
        """
        if haystack is None:
            raise GBRuntimeError("IN: rechte Seite ist NIL")
        if isinstance(haystack, str):
            if not isinstance(needle, str):
                raise TypeMismatchError(
                    f"IN bei STRING: linke Seite muss STRING sein, "
                    f"erhalten {self._type_of(needle)}"
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
                    f"IN bei MAP: Key muss STRING sein, "
                    f"erhalten {self._type_of(needle)}"
                )
            return needle in haystack.data
        raise TypeMismatchError(
            f"IN: rechte Seite muss STRING, TUPLE, ARRAY oder MAP sein, "
            f"erhalten {self._type_of(haystack)}"
        )

    def _eval_UnaryOp(self, e: UnaryOp):
        v = self._eval(e.operand)
        if e.op == "-":
            if isinstance(v, bool) or not isinstance(v, (int, float)):
                raise TypeMismatchError("Unaeres '-' erwartet Zahl")
            return -v
        if e.op == "not":
            return not self._truthy(v)
        if e.op == "bnot":
            if isinstance(v, bool) or not isinstance(v, int):
                raise TypeMismatchError("BNOT erwartet INTEGER")
            return ~v
        raise GBRuntimeError(f"Unbekannter unaerer Operator: {e.op}")

    def _eval_Call(self, e: Call):
        # Grafik-Builtins haben Zugang zur Graphics-Instanz und werden
        # vorrangig vor Datenfunktionen aufgeloest.
        if isinstance(e.callee, Identifier):
            gfx_handler = GRAPHICS_BUILTINS.get(e.callee.name)
            if gfx_handler is not None:
                self._reject_named_args(e.args, e.callee.name.upper())
                args = [self._eval(a) for a in e.args]
                return gfx_handler(self._get_graphics(), args)

        # Methoden-Aufruf: obj.method(args)
        if isinstance(e.callee, MemberAccess):
            obj = self._eval(e.callee.target)
            if obj is None:
                raise GBRuntimeError(
                    f"Methodenaufruf '.{e.callee.name}' bei NIL-Referenz"
                )
            # Container-Methoden: String/Array/Map haben eingebaute
            # Convenience-Methods, die zu BUILTINs delegieren. Receiver
            # wird zum ersten Argument.
            kind = _container_kind(obj)
            if kind:
                builtin_name = CONTAINER_METHODS.get((kind, e.callee.name.lower()))
                if builtin_name is not None:
                    self._reject_named_args(e.args, e.callee.name.upper())
                    builtin = BUILTINS.get(builtin_name)
                    if builtin is None:
                        raise GBRuntimeError(
                            f"Internal: Container-Method delegiert auf "
                            f"unbekanntes Built-in '{builtin_name}'"
                        )
                    eval_args = [obj] + [self._eval(a) for a in e.args]
                    return builtin(eval_args)
                raise GBRuntimeError(
                    f"{kind.upper()} hat keine Methode '{e.callee.name}'"
                )
            if not isinstance(obj, _Instance):
                raise GBRuntimeError(
                    f"Methodenaufruf bei nicht-Objekt ({self._type_of(obj)})"
                )
            method = self._resolve_method(obj.cls, e.callee.name)
            if method is None:
                raise GBRuntimeError(
                    f"Methode '{e.callee.name}' existiert nicht in {obj.cls.name}"
                )
            args, exprs = self._resolve_args(method.decl, e.args,
                                              e.callee.name.upper())
            return self._call_method(obj, method, args, arg_exprs=exprs)

        if not isinstance(e.callee, Identifier):
            # callee ist ein Ausdruck -- evaluieren und FuncRef erwarten.
            callee_val = self._eval(e.callee)
            if isinstance(callee_val, _FuncRef):
                return self._call_via_funcref(callee_val, e.args)
            raise GBRuntimeError(
                f"Wert vom Typ {self._type_of(callee_val)} ist nicht aufrufbar"
            )
        name = e.callee.name
        # FuncRef-Variable: wenn der Identifier-Name auf eine Variable mit
        # FUNCREF-Wert verweist (und nicht selbst eine User-Function ist),
        # via FuncRef dispatchen.
        if self.env.has(name):
            slot = self.env.get_slot(name)
            v = slot["value"]
            if isinstance(v, _FuncRef):
                return self._call_via_funcref(v, e.args)
        # Implizite Methoden-Aufrufe innerhalb einer Methode: wenn der
        # Identifier eine Methode der aktuellen Klasse (oder einer
        # Superklasse) ist, behandeln wir den Aufruf wie `Self.method(...)`.
        # Globale Funktionen mit gleichem Namen waeren ueberschattet -
        # bewusst, wie in Python und Co.
        if self._method_stack:
            cur_inst, cur_cls = self._method_stack[-1]
            method = self._resolve_method(cur_cls, name)
            if method is not None:
                args, exprs = self._resolve_args(method.decl, e.args, name.upper())
                return self._call_method(cur_inst, method, args, arg_exprs=exprs)
        # Benutzerdefinierte Funktionen haben Vorrang.
        fn = self.functions.get(name)
        if fn is not None:
            args, exprs = self._resolve_args(fn.decl, e.args, name.upper())
            return self._call_user(fn, args, arg_exprs=exprs)
        builtin = BUILTINS.get(name)
        if builtin is None:
            raise GBRuntimeError(f"Unbekannte Funktion: {name.upper()}")
        # Built-ins haben keine Named-Arg-Semantik (kein deklarierter
        # Parameter-Name auf Python-Seite).
        self._reject_named_args(e.args, name.upper())
        args = [self._eval(a) for a in e.args]
        return builtin(args)

    def _call_via_funcref(self, ref, raw_args):
        """Ruft die User-Function auf, auf die `ref` zeigt. Lookup-Fail wirft."""
        fn = self.functions.get(ref.name)
        if fn is None:
            raise GBRuntimeError(f"FUNCREF: Funktion '{ref.name}' existiert nicht (mehr)")
        args, exprs = self._resolve_args(fn.decl, raw_args, ref.name.upper())
        return self._call_user(fn, args, arg_exprs=exprs)

    def _reject_named_args(self, raw_args, fn_name: str):
        """Wirft, wenn raw_args Named-Args enthaelt - fuer Built-ins und
        Grafik-Built-ins, die keine deklarierten Parameter-Namen haben."""
        for a in raw_args:
            if isinstance(a, NamedArg):
                raise GBRuntimeError(
                    f"{fn_name}: Named-Args werden nur fuer SUB/FUNCTION/Init "
                    f"unterstuetzt, nicht fuer Built-ins"
                )

    def _resolve_args(self, decl, raw_args, fn_name: str):
        """Mappe (positional + named) auf die Param-Reihenfolge des Decls.

        Liefert `(values, exprs)` - beide Listen der Laenge `len(decl.params)`.
        Slots, die der Caller weder positional noch named belegt hat, sind in
        beiden Listen `_DEFAULT_SENTINEL`.  `_invoke` evaluiert dort spaeter
        den Default-Ausdruck (oder wirft, wenn der Param Pflicht ist).

        Variadic (`...args`): muss letzter Param sein. Alle Positional-Args
        ab der Variadic-Position werden in ein TUPLE gesammelt. Named-Args
        funktionieren nicht mit Variadic-Args (kein Sinn semantisch).

        Validiert: positional muss vor named kommen, kein doppelt-belegt,
        kein unbekannter Name, nicht zu viele Args.

        Werte werden hier (im Caller-Scope) evaluiert - vor dem Wechsel zum
        local_env. So koennen Args sich auf Caller-Variablen beziehen.
        """
        params = decl.params
        n_total = len(params)
        # Variadic-Erkennung: letzter Param mit is_variadic=True.
        has_variadic = n_total > 0 and getattr(params[-1], "is_variadic", False)
        n_required = (n_total - 1) if has_variadic else n_total

        values: list = [_DEFAULT_SENTINEL] * n_total
        exprs: list = [_DEFAULT_SENTINEL] * n_total

        # Phase 1: positional (bis zum ersten NamedArg)
        pos_count = 0
        for a in raw_args:
            if isinstance(a, NamedArg):
                break
            pos_count += 1
        if not has_variadic and pos_count > n_total:
            raise GBRuntimeError(
                f"{fn_name}: zu viele Argumente (Funktion erwartet {n_total})"
            )
        # Phase 2: keine positional NACH named
        for j in range(pos_count, len(raw_args)):
            if not isinstance(raw_args[j], NamedArg):
                raise GBRuntimeError(
                    f"{fn_name}: positional Argument nach Named-Arg ist nicht "
                    f"erlaubt (alle Named-Args muessen am Ende stehen)"
                )

        # Positional einordnen. Bei Variadic: erst n_required als normale
        # Slots, dann der Rest in das TUPLE-Slot.
        if has_variadic:
            normal_pos = min(pos_count, n_required)
            for i in range(normal_pos):
                values[i] = self._eval(raw_args[i])
                exprs[i] = raw_args[i]
            # Restliche positional -> Variadic-Tuple
            varargs = tuple(self._eval(raw_args[i]) for i in range(normal_pos, pos_count))
            values[n_total - 1] = varargs
            exprs[n_total - 1] = None    # kein einzelner Expr-Quell-Knoten
        else:
            for i in range(pos_count):
                values[i] = self._eval(raw_args[i])
                exprs[i] = raw_args[i]

        # Named einordnen
        param_index = {p.name.lower(): i for i, p in enumerate(params)}
        for j in range(pos_count, len(raw_args)):
            na: NamedArg = raw_args[j]
            key = na.name.lower()
            idx = param_index.get(key)
            if idx is None:
                raise GBRuntimeError(
                    f"{fn_name}: kein Parameter mit Namen '{na.name}'"
                )
            if has_variadic and idx == n_total - 1:
                raise GBRuntimeError(
                    f"{fn_name}: Variadic-Parameter '{na.name}' kann nicht "
                    f"als Named-Arg uebergeben werden"
                )
            if values[idx] is not _DEFAULT_SENTINEL:
                raise GBRuntimeError(
                    f"{fn_name}: Parameter '{na.name}' doppelt belegt "
                    f"(positional und named)"
                )
            values[idx] = self._eval(na.value)
            exprs[idx] = na.value
        # Variadic-Slot leer geblieben? -> leeres Tupel (kein Default).
        if has_variadic and values[n_total - 1] is _DEFAULT_SENTINEL:
            values[n_total - 1] = ()
            exprs[n_total - 1] = None
        return values, exprs

    def _eval_New(self, e: New):
        cls = self.classes.get(e.class_name)
        if cls is None:
            raise GBRuntimeError(f"Klasse '{e.class_name}' nicht gefunden")
        inst = self._allocate_instance(cls)
        if e.args is not None:
            init = self._resolve_method(cls, "init")
            if init is None:
                if len(e.args) > 0:
                    raise GBRuntimeError(
                        f"Klasse {cls.name} hat keine SUB Init - "
                        f"Argumente bei NEW nicht moeglich"
                    )
            else:
                args, exprs = self._resolve_args(init.decl, e.args,
                                                  f"NEW {cls.name}")
                self._call_method(inst, init, args, arg_exprs=exprs)
        return inst

    def _eval_SliceAccess(self, e):
        target = self._eval(e.target)
        if target is None:
            raise GBRuntimeError("Slice-Zugriff auf NIL")
        # Bounds aufloesen. None-Werte werden zu 0 / len(target).
        n = self._slice_length(target)
        lo = 0 if e.lo is None else self._eval(e.lo)
        hi = n if e.hi is None else self._eval(e.hi)
        for v, label in ((lo, "lo"), (hi, "hi")):
            if isinstance(v, bool) or not isinstance(v, int):
                raise TypeMismatchError(
                    f"Slice-Index ({label}) muss INTEGER sein, "
                    f"erhalten {self._type_of(v)}"
                )
        if lo < 0 or hi < 0:
            raise GBRuntimeError("Negative Slice-Indices nicht unterstuetzt")
        # Clamping nach oben (Python-Semantik): hi > n -> n.
        if hi > n:
            hi = n
        if lo > n:
            lo = n
        if lo > hi:
            lo = hi   # leere Sub-Sequenz
        return self._slice_apply(target, lo, hi)

    def _slice_length(self, target) -> int:
        if isinstance(target, str):
            return len(target)
        if isinstance(target, _GBArray):
            if len(target.dims) != 1:
                raise GBRuntimeError(
                    "Slicing ist nur fuer 1D-Arrays unterstuetzt"
                )
            return target.dims[0]
        raise TypeMismatchError(
            f"Slice-Zugriff: Erwartet STRING oder ARRAY, "
            f"erhalten {self._type_of(target)}"
        )

    def _slice_apply(self, target, lo: int, hi: int):
        if isinstance(target, str):
            return target[lo:hi]
        # Array -- neues _GBArray mit dem gleichen element_type.
        sub = target.values[lo:hi]
        new_dims = [hi - lo]
        result = _GBArray(target.element_type, new_dims,
                          lambda t=target.element_type: TYPE_DEFAULTS.get(t, None))
        # _GBArray.__init__ alloziert mit Defaults; wir ueberschreiben mit
        # der Slice-Kopie.
        for i, v in enumerate(sub):
            result.values[i] = v
        return result

    def _eval_IndexAccess(self, e: IndexAccess):
        arr = self._eval(e.target)
        if arr is None:
            raise GBRuntimeError("Index-Zugriff auf NIL")
        # String-Index: einzelner Integer, liefert ein 1-Char-String.
        if isinstance(arr, str):
            if len(e.indices) != 1:
                raise GBRuntimeError("String-Index braucht genau einen Wert")
            idx = self._eval(e.indices[0])
            if isinstance(idx, bool) or not isinstance(idx, int):
                raise TypeMismatchError(
                    f"String-Index muss INTEGER sein, erhalten {self._type_of(idx)}"
                )
            if idx < 0 or idx >= len(arr):
                raise GBRuntimeError(
                    f"String-Index {idx} ausserhalb des Bereichs (Laenge {len(arr)})"
                )
            return arr[idx]
        # Tupel-Index: einzelner Integer-Index. Liefert das Element.
        if isinstance(arr, tuple):
            if len(e.indices) != 1:
                raise GBRuntimeError("Tupel-Index braucht genau einen Wert")
            idx = self._eval(e.indices[0])
            if isinstance(idx, bool) or not isinstance(idx, int):
                raise TypeMismatchError(
                    f"Tupel-Index muss INTEGER sein, erhalten {self._type_of(idx)}"
                )
            if idx < 0 or idx >= len(arr):
                raise GBRuntimeError(
                    f"Tupel-Index {idx} ausserhalb des Bereichs (Laenge {len(arr)})"
                )
            return arr[idx]
        if not isinstance(arr, _GBArray):
            raise GBRuntimeError(
                f"Index-Zugriff auf Nicht-Array ({self._type_of(arr)})"
            )
        idx_vals = []
        for ie in e.indices:
            idx = self._eval(ie)
            if isinstance(idx, bool) or not isinstance(idx, int):
                raise TypeMismatchError(
                    f"Array-Index muss INTEGER sein, erhalten {self._type_of(idx)}"
                )
            idx_vals.append(idx)
        return arr.values[arr.flat_index(idx_vals)]

    def _infer_type(self, value, ctx: str = "") -> str:
        # Delegiert an die kanonische Modul-Funktion -- identisches Verhalten
        # in allen drei Pfaden (siehe infer_type oben). ctx bleibt fuer
        # Signatur-Kompatibilitaet erhalten, fliesst aber nicht mehr in die
        # Meldung ein (sonst divergierte der Throw-Pfad gegen die VMs).
        return infer_type(value)

    def _eval_MemberAccess(self, e: MemberAccess):
        obj = self._eval(e.target)
        if obj is None:
            raise GBRuntimeError(f"Zugriff auf '.{e.name}' bei NIL-Referenz")
        if isinstance(obj, _EnumNamespace):
            val = obj.get(e.name)
            if val is None:
                avail = ", ".join(obj.members.keys())
                raise GBRuntimeError(
                    f"ENUM {obj.name} hat keinen Member '{e.name}' "
                    f"(verfuegbar: {avail})"
                )
            return val
        if isinstance(obj, _ClassStaticNamespace):
            val = obj.get(e.name)
            if val is None:
                avail = ", ".join(obj.members.keys()) or "<keine>"
                raise GBRuntimeError(
                    f"CLASS {obj.name} hat keinen STATIC-Member '{e.name}' "
                    f"(verfuegbar: {avail})"
                )
            return val
        if not isinstance(obj, _Instance):
            raise GBRuntimeError(
                f"Zugriff auf '.{e.name}' bei nicht-Objekt ({self._type_of(obj)})"
            )
        # Property-Lookup: wenn `e.name` als Property in der Klasse oder
        # einer Superklasse existiert, ruf den Getter auf.
        if self._is_property(obj.cls, e.name):
            getter = self._resolve_method(obj.cls, f"__get_{e.name.lower()}")
            if getter is None:
                raise GBRuntimeError(
                    f"Property '{e.name}' in {obj.cls.name} hat keinen Getter "
                    f"(nur SET deklariert)"
                )
            return self._call_method(obj, getter, [])
        if e.name in obj.fields:
            return obj.fields[e.name]["value"]
        method = self._resolve_method(obj.cls, e.name)
        if method is not None:
            raise GBRuntimeError(
                f"'{e.name}' ist eine Methode von {obj.cls.name} - "
                f"benutze {e.name}(...) zum Aufruf"
            )
        raise GBRuntimeError(f"Feld '{e.name}' existiert nicht in {obj.cls.name}")

    def _is_property(self, cls: _ClassInfo, name: str) -> bool:
        """TRUE wenn `name` als Property irgendwo in der Klassen-Vererbung
        deklariert ist."""
        cur = cls
        target = name.lower()
        while cur is not None:
            if target in cur.properties:
                return True
            cur = cur.parent
        return False

    def _call_user(self, fn: "_UserFunction", args: list, arg_exprs=None):
        return self._invoke(fn, args, instance=None, arg_exprs=arg_exprs)

    def _call_method(self, inst: _Instance, fn: "_UserFunction", args: list,
                      arg_exprs=None):
        return self._invoke(fn, args, instance=inst, arg_exprs=arg_exprs)

    def _invoke(self, fn: "_UserFunction", args: list, instance,
                arg_exprs=None):
        decl = fn.decl
        n_total = len(decl.params)
        # `args` kann zwei Formen haben (beide unterstuetzt fuer
        # Backwards-Compat mit ungeresolvtem Aufruf):
        #
        #   1. Resolved (vom _resolve_args-Helper): Laenge == n_total,
        #      ggf. _DEFAULT_SENTINEL fuer unbelegte Slots.
        #   2. Klassisch positional: len(args) <= n_total, keine Sentinels.
        #
        # Wir normalisieren auf Form 1, damit der Body einheitlich ist.
        if len(args) == n_total and any(a is _DEFAULT_SENTINEL for a in args):
            # Form 1 - kein extra Pruefen noetig (Caller hat schon validiert)
            pass
        elif len(args) <= n_total:
            # Form 2 -> auf Form 1 erweitern, nicht-belegte Slots = Sentinel
            args = list(args) + [_DEFAULT_SENTINEL] * (n_total - len(args))
            if arg_exprs is not None:
                arg_exprs = list(arg_exprs) + [_DEFAULT_SENTINEL] * (n_total - len(arg_exprs))
        else:
            raise GBRuntimeError(
                f"{decl.name.upper()}: zu viele Argumente "
                f"(erwartet maximal {n_total}, erhalten {len(args)})"
            )
        # Pflicht-Param-Check pro Slot: Sentinel UND kein Default = Fehler
        for i, p in enumerate(decl.params):
            if args[i] is _DEFAULT_SENTINEL and p.default is None:
                raise GBRuntimeError(
                    f"{decl.name.upper()}: Parameter '{p.name}' fehlt "
                    f"(weder positional noch named angegeben)"
                )
        # Bei Methoden-Aufruf: Felder des Objekts liegen als Scope-Ebene
        # zwischen Locals und Globals -> Methoden koennen Felder direkt
        # ohne Praefix lesen/schreiben.
        if instance is not None:
            instance_env = Environment(self.global_env)
            instance_env.vars = instance.fields  # Aliasing: Schreiben persistiert
            base = instance_env
        else:
            base = self.global_env
        local_env = Environment(base)
        # Wichtig: wir setzen self.env BEVOR wir Default-Ausdruecke evaluieren,
        # damit ein Default frueheren Parameter referenzieren kann.
        prev_env = self.env
        # BYREF-Vorbereitung: in der CALLER-Umgebung lvalues bestimmen, dann
        # zurueck zur lokalen Umgebung wechseln. Die lvalues werden nach der
        # Body-Ausfuehrung benutzt um die geaenderten Werte zurueckzuschreiben.
        # Sentinels (= Default-Slot) koennen kein BYREF sein - die haben keinen
        # caller-side lvalue.
        byref_lvalues: list = [None] * len(decl.params)
        if arg_exprs is not None:
            for i, param in enumerate(decl.params):
                if not param.by_ref:
                    continue
                if arg_exprs[i] is _DEFAULT_SENTINEL:
                    continue
                lv = self._lvalue_of(arg_exprs[i])
                if lv is None:
                    raise GBRuntimeError(
                        f"BYREF-Parameter '{param.name}' braucht eine "
                        f"zuweisbare Variable (Identifier, Member oder "
                        f"Index-Access), erhalten Ausdruck"
                    )
                byref_lvalues[i] = lv
        self.env = local_env
        try:
            for i, param in enumerate(decl.params):
                local_env.declare(
                    param.name, param.type_name,
                    self._default_for(param.type_name),
                )
                if args[i] is _DEFAULT_SENTINEL:
                    # Default-Ausdruck zur Laufzeit auswerten - im local_env
                    # damit frueher gesetzte Parameter referenziert werden
                    # koennen.
                    arg_value = self._eval(param.default)
                else:
                    arg_value = args[i]
                local_env.set(
                    param.name,
                    self._coerce(arg_value, param.type_name,
                                  f"Parameter '{param.name}'"),
                )
        except Exception:
            self.env = prev_env
            raise
        self.call_depth += 1
        # Methoden-Stack pflegen: ermoeglicht `Self` und implizite
        # Methoden-Aufrufe innerhalb von Klassen-Methoden.
        if instance is not None:
            self._method_stack.append((instance, instance.cls))
        try:
            for stmt in decl.body:
                self._exec(stmt)
            if fn.kind == "function":
                raise GBRuntimeError(
                    f"FUNCTION {decl.name.upper()} muss einen Wert mit RETURN zurueckgeben"
                )
            return None
        except _ReturnSignal as r:
            if fn.kind == "sub":
                if r.value is not None:
                    raise GBRuntimeError(
                        f"SUB {decl.name.upper()} darf RETURN nicht mit Wert verwenden"
                    )
                return None
            return self._coerce(
                r.value, decl.return_type, f"RETURN aus {decl.name.upper()}"
            )
        finally:
            # BYREF-Werte zurueck in den Caller schreiben - im prev_env-
            # Kontext, damit lvalues im richtigen Scope wirken. Auch bei
            # Exception/Throw machen wir Copy-Out (sonst gingen Aenderungen
            # verloren - manche User-Faelle erwarten das aber).
            self.env = prev_env
            for i, param in enumerate(decl.params):
                lv = byref_lvalues[i]
                if lv is None:
                    continue
                try:
                    val = local_env.get(param.name)
                    lv(val)
                except Exception:
                    # Copy-Out darf den Original-Fluss nicht stoeren
                    pass
            if instance is not None:
                self._method_stack.pop()
            self.call_depth -= 1

    # ---- BYREF: lvalue-Resolution ------------------------------------
    def _lvalue_of(self, expr):
        """Liefert ein Closure (callable mit value) das den Argument-Ausdruck
        in der aktuellen Umgebung zuweist.

        Returns None wenn `expr` keine zuweisbare Form ist (Literal, BinOp,
        Funktionsaufruf etc.).
        """
        if isinstance(expr, Identifier):
            env = self.env
            name = expr.name
            return lambda v, e=env, n=name: e.set(n, v)
        if isinstance(expr, MemberAccess):
            obj = self._eval(expr.target)
            field = expr.name
            if not isinstance(obj, _Instance):
                return None
            def setter(v, o=obj, f=field):
                if f not in o.fields:
                    raise GBRuntimeError(f"Feld '{f}' existiert nicht")
                # Felder werden im Slot-Format {"value":..., "type":...}
                # gespeichert (siehe Environment.declare/set).
                slot = o.fields[f]
                if isinstance(slot, dict) and "value" in slot:
                    slot["value"] = v
                else:
                    o.fields[f] = v
            return setter
        if isinstance(expr, IndexAccess):
            arr_val = self._eval(expr.target)
            idxs = [self._eval(i) for i in expr.indices]
            if isinstance(arr_val, _GBArray):
                flat = arr_val.flat_index(idxs)
                values = arr_val.values
                return lambda v, lst=values, fi=flat: lst.__setitem__(fi, v)
            return None
        return None

    # ---- Helfer ------------------------------------------------------
    def _coerce(self, value, target: str, ctx: str):
        if target == "integer":
            if isinstance(value, bool):
                raise TypeMismatchError(f"{ctx}: Erwartet INTEGER, erhalten BOOLEAN")
            if isinstance(value, int):
                return value
            if isinstance(value, float):
                if not value.is_integer():
                    raise TypeMismatchError(
                        f"{ctx}: FLOAT {value} kann nicht ohne Verlust nach INTEGER (nutze INT())"
                    )
                return int(value)
            raise TypeMismatchError(f"{ctx}: Erwartet INTEGER, erhalten {self._type_of(value)}")
        if target == "float":
            if isinstance(value, bool):
                raise TypeMismatchError(f"{ctx}: Erwartet FLOAT, erhalten BOOLEAN")
            if isinstance(value, (int, float)):
                return float(value)
            raise TypeMismatchError(f"{ctx}: Erwartet FLOAT, erhalten {self._type_of(value)}")
        if target == "string":
            if isinstance(value, str):
                return value
            raise TypeMismatchError(f"{ctx}: Erwartet STRING, erhalten {self._type_of(value)}")
        if target == "boolean":
            if isinstance(value, bool):
                return value
            raise TypeMismatchError(f"{ctx}: Erwartet BOOLEAN, erhalten {self._type_of(value)}")
        if target == "tuple":
            # Tupel ist generisch -- wir validieren nur, dass es ueberhaupt ein
            # Tupel ist. Element-Typen werden nicht erzwungen (wer striktere
            # Garantien will, prueft selbst beim Destructuring).
            if isinstance(value, tuple):
                return value
            raise TypeMismatchError(f"{ctx}: Erwartet TUPLE, erhalten {self._type_of(value)}")
        if target == "any":
            # "any" ist intern -- z.B. fuer Compiler-generierte Slots wie
            # WITH-Targets. Kein Type-Check, value passiert unveraendert.
            return value
        if target == "funcref":
            if isinstance(value, _FuncRef):
                return value
            raise TypeMismatchError(
                f"{ctx}: Erwartet FUNCREF, erhalten {self._type_of(value)}"
            )
        if target.startswith("map:"):
            value_type = target[4:]
            if value is None:
                return None
            if not isinstance(value, _GBMap):
                raise TypeMismatchError(
                    f"{ctx}: Erwartet MAP OF {value_type.upper()}, erhalten {self._type_of(value)}"
                )
            if value.value_type != value_type:
                raise TypeMismatchError(
                    f"{ctx}: Erwartet MAP OF {value_type.upper()}, "
                    f"erhalten MAP OF {value.value_type.upper()}"
                )
            return value
        if target == "image":
            if value is None or isinstance(value, _Image):
                return value
            raise TypeMismatchError(f"{ctx}: Erwartet IMAGE, erhalten {self._type_of(value)}")
        if target == "sound":
            if value is None or isinstance(value, _Sound):
                return value
            raise TypeMismatchError(f"{ctx}: Erwartet SOUND, erhalten {self._type_of(value)}")
        if target == "sprite_atlas":
            if value is None or isinstance(value, _SpriteAtlas):
                return value
            raise TypeMismatchError(
                f"{ctx}: Erwartet SPRITE_ATLAS, erhalten {self._type_of(value)}"
            )
        if target == "file":
            if value is None or isinstance(value, _GBFile):
                return value
            raise TypeMismatchError(f"{ctx}: Erwartet FILE, erhalten {self._type_of(value)}")
        # Array-Typ (intern als 'array:<element>')
        if target.startswith("array:"):
            elem = target[len("array:"):]
            if value is None:
                return None
            if not isinstance(value, _GBArray):
                raise TypeMismatchError(
                    f"{ctx}: Erwartet ARRAY OF {elem.upper()}, erhalten {self._type_of(value)}"
                )
            if value.element_type != elem:
                raise TypeMismatchError(
                    f"{ctx}: Erwartet ARRAY OF {elem.upper()}, "
                    f"erhalten ARRAY OF {value.element_type.upper()}"
                )
            return value
        # Klassentyp
        target_cls = self.classes.get(target)
        if target_cls is not None:
            if value is None:
                return None
            if not isinstance(value, _Instance):
                raise TypeMismatchError(
                    f"{ctx}: Erwartet {target}, erhalten {self._type_of(value)}"
                )
            if not self._is_subclass_of(value.cls, target_cls):
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
                f"{ctx}: Erwartet {target.upper()}, erhalten {self._type_of(value)}"
            )
        raise GBRuntimeError(f"Unbekannter Zieltyp: {target}")

    def _require_number(self, a, b, op):
        for v in (a, b):
            if isinstance(v, bool) or not isinstance(v, (int, float)):
                raise TypeMismatchError(f"Operator '{op}' erwartet Zahlen, erhalten {self._type_of(v)}")

    def _require_int_pair(self, a, b, op):
        # Bitwise-Operatoren erwarten beidseitig INTEGER. Bool wird bewusst
        # nicht zugelassen (gleiche Linie wie der Rest der Sprache).
        for v in (a, b):
            if isinstance(v, bool) or not isinstance(v, int):
                raise TypeMismatchError(
                    f"{op.upper()} erwartet INTEGER, erhalten {self._type_of(v)}"
                )

    def _type_of(self, value) -> str:
        if value is None:
            return "NIL"
        if isinstance(value, bool):
            return "BOOLEAN"
        if isinstance(value, int):
            return "INTEGER"
        if isinstance(value, float):
            return "FLOAT"
        if isinstance(value, str):
            return "STRING"
        if isinstance(value, tuple):
            return f"TUPLE({len(value)})"
        if isinstance(value, _FuncRef):
            return f"FUNCREF<{value.name}>"
        # Vec2-Type aus dem Built-in-Modul. Lazy Import vermeidet Circular.
        from .modules.vec2 import _Vec2 as _V2
        if isinstance(value, _V2):
            return "VEC2"
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

    def _fmt(self, v) -> str:
        if v is None:
            return "NIL"
        if isinstance(v, bool):
            return "TRUE" if v else "FALSE"
        if isinstance(v, float):
            if v.is_integer():
                return f"{v:.1f}"
            return repr(v)
        if isinstance(v, tuple):
            return "(" + ", ".join(self._fmt(x) for x in v) + ")"
        if isinstance(v, _FuncRef):
            return f"<FUNCREF {v.name}>"
        from .modules.vec2 import _Vec2 as _V2
        if isinstance(v, _V2):
            return f"Vec2({self._fmt(v.x)}, {self._fmt(v.y)})"
        if isinstance(v, _Instance):
            return f"<{v.cls.name}>"
        if isinstance(v, _GBArray):
            shape = ",".join(str(d) for d in v.dims) if v.dims else ""
            return f"<ARRAY[{shape}] OF {v.element_type.upper()}>"
        if isinstance(v, _Image):
            return f"<IMAGE>"
        if isinstance(v, _Sound):
            return f"<SOUND>"
        if isinstance(v, _SpriteAtlas):
            return f"<SPRITE_ATLAS frames={len(v.frames)}>"
        if isinstance(v, _GBFile):
            return f"<FILE {v.path}>"
        if isinstance(v, _GBMap):
            return f"<MAP[{len(v.data)}] OF {v.value_type.upper()}>"
        return str(v)

    def _truthy(self, v) -> bool:
        if isinstance(v, bool):
            return v
        if isinstance(v, (int, float)):
            return v != 0
        if isinstance(v, str):
            return v != ""
        return v is not None


# --- Eingebaute Funktionen --------------------------------------------

@builtin(("STR$", "STR"), arity=1, types=("any",))
def _b_str(v):
    if isinstance(v, bool):
        return "TRUE" if v else "FALSE"
    if isinstance(v, float) and v.is_integer():
        return f"{v:.1f}"
    return str(v)


@builtin("VAL", arity=1, types=("str",))
def _b_val(s):
    s = s.strip()
    try:
        if "." in s:
            return float(s)
        return int(s)
    except ValueError:
        return 0


@builtin("INT", arity=1, types=("num",))
def _b_int(v):
    return int(math.floor(v))


@builtin("ABS", arity=1, types=("num",))
def _b_abs(v):
    return abs(v)


@builtin("IS_NIL", arity=1)
def _b_is_nil(v):
    """TRUE wenn der Wert NIL ist (Python-`None`).

    Praktisch fuer Modul-Built-ins, die in Non-Blocking-Modus optionale
    Werte zurueckgeben: `NET_TCP_ACCEPT(lst)` liefert NIL wenn keine
    Connection wartet, `MAPGETOR` braucht keine NIL-Pruefung weil's einen
    Default hat -- aber `NET_TCP_ACCEPT` und aehnliche schon.
    """
    return v is None


@builtin("RANGE", arity=(1, 3))
def _b_range(*args):
    """RANGE(n) -> (0, 1, ..., n-1)
    RANGE(start, stop) -> (start, start+1, ..., stop-1)
    RANGE(start, stop, step) -> (start, start+step, ...)

    Liefert ein TUPLE der Integers. Sehr praktisch fuer Comprehensions:
    `[i * i FOR i IN RANGE(10)]`. Negative Steps funktionieren genau so
    wie in Python (`RANGE(10, 0, -1)`).
    """
    # Strikte INTEGER-Validierung selbst (variable arity erlaubt kein
    # `types`-Argument im Decorator).
    for v in args:
        if isinstance(v, bool) or not isinstance(v, int):
            raise TypeMismatchError(
                f"RANGE: Argument muss INTEGER sein, erhalten {type(v).__name__}"
            )
    if len(args) == 1:
        return tuple(range(args[0]))
    if len(args) == 2:
        return tuple(range(args[0], args[1]))
    if args[2] == 0:
        raise GBRuntimeError("RANGE: step darf nicht 0 sein")
    return tuple(range(args[0], args[1], args[2]))


@builtin("__COMP_ITER", arity=1)
def _b_comp_iter(v):
    """Internal: wandelt einen Container in ein TUPLE der iterierbaren
    Werte um. Wird vom Compiler vor jeder List-Comprehension aufgerufen,
    damit der Loop immer ueber ein Tupel laeuft (uniform LEN+Index).
    """
    if isinstance(v, str):
        return tuple(v)
    if isinstance(v, tuple):
        return v
    if isinstance(v, _GBArray):
        if len(v.dims) != 1:
            raise GBRuntimeError("Comprehension: nur 1D-Arrays unterstuetzt")
        return tuple(v.values)
    if isinstance(v, _GBMap):
        return tuple(v.data.keys())
    raise TypeMismatchError(
        f"Comprehension: nicht iterierbar ({type(v).__name__.upper()})"
    )


@builtin("__SET_DEDUP", arity=1)
def _b_set_dedup(v):
    """Internal: Set-Comprehension liefert ein TUPLE mit deduplizierten
    Werten in der Reihenfolge des ersten Auftretens. Aufgerufen vom
    Compiler nach BUILD_TUPLE_DYN."""
    if not isinstance(v, tuple):
        raise TypeMismatchError("__SET_DEDUP erwartet TUPLE")
    seen: list = []
    out: list = []
    for x in v:
        if x not in seen:
            seen.append(x)
            out.append(x)
    return tuple(out)


@builtin("__DICT_FROM_PAIRS", arity=1)
def _b_dict_from_pairs(v):
    """Internal: baut ein _GBMap aus einem TUPLE von 2-Tupeln. Aufgerufen
    vom Compiler fuer Dict-Comprehension nach BUILD_TUPLE_DYN."""
    if not isinstance(v, tuple):
        raise TypeMismatchError("__DICT_FROM_PAIRS erwartet TUPLE")
    pairs: list = []
    for p in v:
        if not isinstance(p, tuple) or len(p) != 2:
            raise GBRuntimeError(
                "__DICT_FROM_PAIRS: erwartet 2-Tupel (key, value)"
            )
        k, val = p
        if not isinstance(k, str):
            raise TypeMismatchError(
                f"Dict-Comprehension: Key muss STRING sein, "
                f"erhalten {type(k).__name__.upper()}"
            )
        pairs.append((k, val))
    if pairs:
        # Type-Inferenz vom ersten Wert. Lazy-Import fuer _type_of -- wir
        # nutzen die einfache Variante hier.
        first_v = pairs[0][1]
        if isinstance(first_v, bool):
            value_type = "boolean"
        elif isinstance(first_v, int):
            value_type = "integer"
        elif isinstance(first_v, float):
            value_type = "float"
        elif isinstance(first_v, str):
            value_type = "string"
        else:
            value_type = "any"
    else:
        value_type = "any"
    m = _GBMap(value_type)
    for k, val in pairs:
        m.data[k] = val
    return m


@builtin("LEN", arity=1)
def _b_len(v):
    if isinstance(v, str):
        return len(v)
    if isinstance(v, tuple):
        return len(v)
    if isinstance(v, _GBArray):
        # 1D: Anzahl Elemente; mehrdim: erste Dimension (Anzahl Zeilen).
        return v.dims[0] if v.dims else 0
    raise TypeMismatchError("LEN erwartet STRING, TUPLE oder ARRAY")


@builtin("DIMSIZE", arity=2)
def _b_dimsize(arr, n):
    if not isinstance(arr, _GBArray):
        raise TypeMismatchError("DIMSIZE erwartet ARRAY")
    if isinstance(n, bool) or not isinstance(n, int):
        raise TypeMismatchError("DIMSIZE: zweites Argument muss INTEGER sein")
    if n < 0 or n >= len(arr.dims):
        raise GBRuntimeError(
            f"DIMSIZE: Dimension {n} ausserhalb [0..{len(arr.dims) - 1}]"
        )
    return arr.dims[n]


@builtin("DIMCOUNT", arity=1)
def _b_dimcount(arr):
    if not isinstance(arr, _GBArray):
        raise TypeMismatchError("DIMCOUNT erwartet ARRAY")
    return len(arr.dims)


@builtin(("CHR$", "CHR"), arity=1, types=("int",))
def _b_chr(n):
    return chr(n)


@builtin("ASC", arity=1, types=("str",))
def _b_asc(s):
    if len(s) == 0:
        raise TypeMismatchError("ASC erwartet nicht-leeren STRING")
    return ord(s[0])


@builtin("SQR", arity=1, types=("num",))
def _b_sqr(v):
    if v < 0:
        raise GBRuntimeError("SQR von negativer Zahl")
    return math.sqrt(v)


@builtin("RND", arity=(0, 1))
def _b_rnd(*args):
    if not args:
        return random.random()
    n = args[0]
    if isinstance(n, bool) or not isinstance(n, int):
        raise TypeMismatchError("RND erwartet INTEGER oder kein Argument")
    if n <= 0:
        raise GBRuntimeError("RND erwartet positives INTEGER")
    return random.randint(0, n - 1)


@builtin(("UPPER$", "UPPER"), arity=1, types=("str",))
def _b_upper(s):
    return s.upper()


@builtin(("LOWER$", "LOWER"), arity=1, types=("str",))
def _b_lower(s):
    return s.lower()


@builtin("RGB", arity=3, types=("int", "int", "int"))
def _b_rgb(r, g, b):
    for v in (r, g, b):
        if v < 0 or v > 255:
            raise GBRuntimeError("RGB-Werte muessen 0..255 sein")
    return (r << 16) | (g << 8) | b


# --- Math-Builtins -----------------------------------------------------

def _check_num(v, fn: str):
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn} erwartet Zahl, erhalten {type(v).__name__}")
    return v


@builtin("SIN", arity=1, types=("num",))
def _b_sin(x):
    return math.sin(x)


@builtin("COS", arity=1, types=("num",))
def _b_cos(x):
    return math.cos(x)


@builtin("TAN", arity=1, types=("num",))
def _b_tan(x):
    return math.tan(x)


@builtin("ATAN", arity=1, types=("num",))
def _b_atan(x):
    return math.atan(x)


@builtin("ATAN2", arity=2, types=("num", "num"))
def _b_atan2(y, x):
    return math.atan2(y, x)


@builtin("FLOOR", arity=1, types=("num",))
def _b_floor(x):
    return int(math.floor(x))


@builtin("CEIL", arity=1, types=("num",))
def _b_ceil(x):
    return int(math.ceil(x))


@builtin("ROUND", arity=1, types=("num",))
def _b_round(x):
    return int(round(x))


@builtin("LOG", arity=(1, 2))
def _b_log(*args):
    x = _check_num(args[0], "LOG")
    if x <= 0:
        raise GBRuntimeError("LOG: Argument muss > 0 sein")
    if len(args) == 2:
        return math.log(x, _check_num(args[1], "LOG"))
    return math.log(x)


@builtin("EXP", arity=1, types=("num",))
def _b_exp(x):
    return math.exp(x)


@builtin("POW", arity=2, types=("num", "num"))
def _b_pow(base, exponent):
    return base ** exponent


@builtin("MIN", arity=(1, None))
def _b_min(*args):
    for v in args:
        _check_num(v, "MIN")
    return min(args)


@builtin("MAX", arity=(1, None))
def _b_max(*args):
    for v in args:
        _check_num(v, "MAX")
    return max(args)


@builtin("CLAMP", arity=3, types=("num", "num", "num"))
def _b_clamp(v, lo, hi):
    if v < lo: return lo
    if v > hi: return hi
    return v


@builtin("SIGN", arity=1, types=("num",))
def _b_sign(v):
    if v > 0: return 1
    if v < 0: return -1
    return 0


# --- Zeit / Random ----------------------------------------------------

import time as _time
_START_MONOTONIC = _time.monotonic()


@builtin("MILLIS", arity=0)
def _b_millis():
    return int((_time.monotonic() - _START_MONOTONIC) * 1000)


@builtin("RANDOMIZE", arity=(0, 1))
def _b_randomize(*args):
    if args:
        seed = args[0]
        if isinstance(seed, bool) or not isinstance(seed, int):
            raise TypeMismatchError("RANDOMIZE erwartet INTEGER-Seed")
        random.seed(seed)
    else:
        random.seed()
    return None


@builtin(("TIME$", "TIME"), arity=0)
def _b_time_str():
    return _time.strftime("%H:%M:%S")


@builtin(("DATE$", "DATE"), arity=0)
def _b_date_str():
    return _time.strftime("%Y-%m-%d")


# --- String-Builtins (zusaetzlich) ------------------------------------

@builtin(("PADL$", "PADL"), arity=(2, 3))
def _b_padl(*args):
    s = _check_str(args[0], "PADL$")
    n = args[1]
    if isinstance(n, bool) or not isinstance(n, int):
        raise TypeMismatchError("PADL$: Breite muss INTEGER sein")
    fill = " "
    if len(args) == 3:
        fill = _check_str(args[2], "PADL$")
        if not fill:
            fill = " "
    return s.rjust(n, fill[0])


@builtin(("PADR$", "PADR"), arity=(2, 3))
def _b_padr(*args):
    s = _check_str(args[0], "PADR$")
    n = args[1]
    if isinstance(n, bool) or not isinstance(n, int):
        raise TypeMismatchError("PADR$: Breite muss INTEGER sein")
    fill = " "
    if len(args) == 3:
        fill = _check_str(args[2], "PADR$")
        if not fill:
            fill = " "
    return s.ljust(n, fill[0])


@builtin("FORMAT$", arity=2)
def _b_format(value, mask):
    """printf-Stil-Formatierung:  FORMAT$(42, "%05d") -> "00042".

    Akzeptiert genau einen Wert und eine Mask (STRING). Fuer mehrere
    Werte mit + verketten:
        FORMAT$(score, "%5d") + " | " + FORMAT$(time_s, "%6.2f")
    """
    if not isinstance(mask, str):
        raise TypeMismatchError("FORMAT$: zweites Argument muss STRING sein")
    # Bool-Sonderfall: Python's % d wuerde 1/0 daraus machen, aber GameBasic
    # zeigt TRUE/FALSE textuell - geben wir explizit als TRUE/FALSE in %s.
    if isinstance(value, bool):
        if "%s" in mask or "%S" in mask:
            value = "TRUE" if value else "FALSE"
        # %d und Co. funktionieren mit int(value) -> 1/0
    try:
        return mask % (value,)
    except (TypeError, ValueError) as exc:
        raise GBRuntimeError(
            f"FORMAT$: '{mask}' passt nicht zu Wert {value!r} ({exc})"
        )


@builtin("REPEAT$", arity=2)
def _b_repeat(s, n):
    s = _check_str(s, "REPEAT$")
    if isinstance(n, bool) or not isinstance(n, int):
        raise TypeMismatchError("REPEAT$: n muss INTEGER sein")
    return s * max(0, n)


@builtin(("SPACE$", "SPACE"), arity=1, types=("int",))
def _b_space(n):
    return " " * max(0, n)


@builtin(("HEX$", "HEX"), arity=1, types=("int",))
def _b_hex(n):
    if n < 0:
        return "-" + hex(-n)[2:].upper()
    return hex(n)[2:].upper()


# --- Bitwise als Operatoren -------------------------------------------
# Frueher gab es BITAND/BITOR/BITXOR/BITNOT/SHL/SHR als Built-in-Funktionen.
# Mit den nativen Operatoren `BAND/BOR/BXOR/BNOT/SHL/SHR` sind die obsolet
# und wurden entfernt -- `a BAND b` ersetzt `BITAND(a, b)`. Das ist im
# Source-Tree dokumentiert in den Beispielen 18_math.gb und 53_bitwise.gb.


# --- Map-Builtins -----------------------------------------------------

def _coerce_map_value(v, value_type: str):
    """Pruefen/konvertieren des Werts gegen den Map-Werttyp (best effort)."""
    if value_type == "integer":
        if isinstance(v, bool) or not isinstance(v, int):
            raise TypeMismatchError(f"Map-Wert: Erwartet INTEGER, erhalten {type(v).__name__}")
        return v
    if value_type == "float":
        if isinstance(v, bool):
            raise TypeMismatchError("Map-Wert: Erwartet FLOAT, erhalten BOOLEAN")
        if not isinstance(v, (int, float)):
            raise TypeMismatchError("Map-Wert: Erwartet FLOAT")
        return float(v)
    if value_type == "string":
        if not isinstance(v, str):
            raise TypeMismatchError("Map-Wert: Erwartet STRING")
        return v
    if value_type == "boolean":
        if not isinstance(v, bool):
            raise TypeMismatchError("Map-Wert: Erwartet BOOLEAN")
        return v
    # Klassen / Arrays / sonstige: trust
    return v


@builtin("MAPPUT", arity=3)
def _b_map_put(m, k, v):
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPPUT erwartet MAP als 1. Argument")
    if not isinstance(k, str):
        raise TypeMismatchError("MAPPUT: Schluessel muss STRING sein")
    m.data[k] = _coerce_map_value(v, m.value_type)
    return None


@builtin("MAPGET", arity=2)
def _b_map_get(m, k):
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPGET erwartet MAP")
    if not isinstance(k, str):
        raise TypeMismatchError("MAPGET: Schluessel muss STRING sein")
    if k not in m.data:
        raise GBRuntimeError(f"MAPGET: Schluessel '{k}' nicht in Map")
    return m.data[k]


@builtin("MAPGETOR", arity=3)
def _b_map_get_or(m, k, default):
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPGETOR erwartet MAP")
    if not isinstance(k, str):
        raise TypeMismatchError("MAPGETOR: Schluessel muss STRING sein")
    return m.data.get(k, default)


@builtin("MAPHAS", arity=2)
def _b_map_has(m, k):
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPHAS erwartet MAP")
    if not isinstance(k, str):
        raise TypeMismatchError("MAPHAS: Schluessel muss STRING sein")
    return k in m.data


@builtin("MAPREMOVE", arity=2)
def _b_map_remove(m, k):
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPREMOVE erwartet MAP")
    if not isinstance(k, str):
        raise TypeMismatchError("MAPREMOVE: Schluessel muss STRING sein")
    return m.data.pop(k, None) is not None


@builtin("MAPSIZE", arity=1)
def _b_map_size(m):
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPSIZE erwartet MAP")
    return len(m.data)


@builtin("MAPKEYS", arity=1)
def _b_map_keys(m):
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPKEYS erwartet MAP")
    keys = list(m.data.keys())
    arr = _GBArray("string", [len(keys)], lambda: "")
    for i, k in enumerate(keys):
        arr.values[i] = k
    return arr


@builtin("MAPCLEAR", arity=1)
def _b_map_clear(m):
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPCLEAR erwartet MAP")
    m.data.clear()
    return None


@builtin("MAPVALUES", arity=1)
def _b_map_values(m):
    """Liefert ein ARRAY mit allen Werten der Map (Einfuege-Reihenfolge)."""
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPVALUES erwartet MAP")
    vals = list(m.data.values())
    vt = m.value_type
    default = TYPE_DEFAULTS.get(vt, None)
    arr = _GBArray(vt, [len(vals)], lambda d=default: d)
    for i, v in enumerate(vals):
        arr.values[i] = v
    return arr


@builtin("MAPITEMS", arity=1)
def _b_map_items(m):
    """Liefert ein ARRAY von (key, value)-TUPELn -- praktisch zum Iterieren:
    FOR i = 0 TO LEN(items)-1 : (k, v) = items[i] : ... NEXT"""
    if not isinstance(m, _GBMap):
        raise TypeMismatchError("MAPITEMS erwartet MAP")
    items = list(m.data.items())
    arr = _GBArray("tuple", [len(items)], lambda: ())
    for i, kv in enumerate(items):
        arr.values[i] = (kv[0], kv[1])
    return arr


# --- Array-Helfer -------------------------------------------------------

def _check_array_1d(v, fn: str):
    if not isinstance(v, _GBArray):
        raise TypeMismatchError(f"{fn} erwartet ARRAY")
    if len(v.dims) != 1:
        raise GBRuntimeError(f"{fn}: nur 1D-Arrays unterstuetzt")
    return v


@builtin("SORT", arity=1)
def _b_sort(arr):
    """Sortiert ein 1D-Array IN PLACE aufsteigend (INTEGER/FLOAT/STRING)."""
    arr = _check_array_1d(arr, "SORT")
    if arr.element_type not in ("integer", "float", "string"):
        raise GBRuntimeError(
            "SORT: nur ARRAY OF INTEGER/FLOAT/STRING sortierbar")
    vals = sorted(arr.values)
    for i, v in enumerate(vals):
        arr.values[i] = v
    return None


@builtin("REVERSE", arity=1)
def _b_reverse(arr):
    """Kehrt ein 1D-Array IN PLACE um."""
    arr = _check_array_1d(arr, "REVERSE")
    arr.values.reverse()
    return None


@builtin("ARRAY_INDEXOF", arity=2, types=("any", "any"))
def _b_array_indexof(arr, value):
    """Erster Index von `value` im 1D-Array, oder -1 wenn nicht enthalten."""
    arr = _check_array_1d(arr, "ARRAY_INDEXOF")
    n = arr.total_size()
    for i in range(n):
        if arr.values[i] == value:
            return i
    return -1


@builtin("COLLIDES", arity=8,
         types=("num", "num", "num", "num", "num", "num", "num", "num"))
def _b_collides(x1, y1, w1, h1, x2, y2, w2, h2):
    """AABB-Kollision: rect1 (x,y,w,h) vs rect2 (x,y,w,h)."""
    return (x1 < x2 + w2 and x1 + w1 > x2
            and y1 < y2 + h2 and y1 + h1 > y2)


# --- String-Funktionen --------------------------------------------------

def _check_str(v, fn: str) -> str:
    if not isinstance(v, str):
        raise TypeMismatchError(f"{fn} erwartet STRING, erhalten {type(v).__name__}")
    return v


@builtin(("LEFT$", "LEFT"), arity=2)
def _b_left(s, n):
    s = _check_str(s, "LEFT$")
    if isinstance(n, bool) or not isinstance(n, int):
        raise TypeMismatchError("LEFT$: Anzahl muss INTEGER sein")
    if n < 0:
        n = 0
    return s[:n]


@builtin(("RIGHT$", "RIGHT"), arity=2)
def _b_right(s, n):
    s = _check_str(s, "RIGHT$")
    if isinstance(n, bool) or not isinstance(n, int):
        raise TypeMismatchError("RIGHT$: Anzahl muss INTEGER sein")
    if n <= 0:
        return ""
    return s[-n:] if n < len(s) else s


@builtin(("MID$", "MID"), arity=(2, 3))
def _b_mid(*args):
    """MID$(s, start[, anzahl]) - 0-basierter Index, wie Arrays."""
    s = _check_str(args[0], "MID$")
    start = args[1]
    if isinstance(start, bool) or not isinstance(start, int):
        raise TypeMismatchError("MID$: Start muss INTEGER sein")
    if start < 0:
        start = 0
    if len(args) == 3:
        n = args[2]
        if isinstance(n, bool) or not isinstance(n, int):
            raise TypeMismatchError("MID$: Anzahl muss INTEGER sein")
        return s[start:start + max(n, 0)]
    return s[start:]


@builtin("INSTR", arity=(2, 3))
def _b_instr(*args):
    """INSTR(haystack, needle[, start]) -> 0-basierter Index oder -1."""
    haystack = _check_str(args[0], "INSTR")
    needle = _check_str(args[1], "INSTR")
    start = 0
    if len(args) == 3:
        s = args[2]
        if isinstance(s, bool) or not isinstance(s, int):
            raise TypeMismatchError("INSTR: Start muss INTEGER sein")
        start = max(0, s)
    return haystack.find(needle, start)


@builtin(("REPLACE$", "REPLACE"), arity=3, types=("str", "str", "str"))
def _b_replace(s, old, new):
    return s.replace(old, new)


@builtin(("TRIM$", "TRIM"), arity=1, types=("str",))
def _b_trim(s):
    return s.strip()


@builtin(("SPLIT$", "SPLIT"), arity=2, types=("str", "str"))
def _b_split(s, delim):
    """SPLIT$(s, delim) -> ARRAY OF STRING."""
    if delim == "":
        parts = list(s)
    else:
        parts = s.split(delim)
    arr = _GBArray("string", [len(parts)], lambda: "")
    for i, p in enumerate(parts):
        arr.values[i] = p
    return arr


@builtin(("JOIN$", "JOIN"), arity=2)
def _b_join(arr, delim):
    """JOIN$(array, trenner) -> STRING. Arbeitet auf ARRAY OF STRING."""
    if not isinstance(arr, _GBArray) or arr.element_type != "string":
        raise TypeMismatchError("JOIN$: erstes Argument muss ARRAY OF STRING sein")
    delim = _check_str(delim, "JOIN$")
    return delim.join(arr.values)


# --- Datei-I/O ----------------------------------------------------------

def _check_file(v, fn: str) -> "_GBFile":
    if v is None:
        raise GBRuntimeError(f"{fn}: Datei ist NIL")
    if not isinstance(v, _GBFile):
        raise TypeMismatchError(f"{fn} erwartet FILE")
    if v.handle is None:
        raise GBRuntimeError(f"{fn}: Datei ist bereits geschlossen")
    return v


@builtin("OPENFILE", arity=2, types=("str", "str"))
def _b_open_file(path, mode):
    if mode not in ("r", "w", "a"):
        raise GBRuntimeError(f"OPENFILE: ungueltiger Modus '{mode}' (erlaubt: r, w, a)")
    try:
        handle = open(path, mode, encoding="utf-8")
    except OSError as exc:
        raise GBRuntimeError(f"OPENFILE: {exc}")
    return _GBFile(handle, path, mode)


@builtin("CLOSEFILE", arity=1)
def _b_close_file(f):
    if not isinstance(f, _GBFile):
        raise TypeMismatchError("CLOSEFILE erwartet FILE")
    if f.handle is not None:
        f.handle.close()
        f.handle = None
    return None


@builtin("READLINE", arity=1)
def _b_read_line(f):
    f = _check_file(f, "READLINE")
    if f.mode != "r":
        raise GBRuntimeError("READLINE: Datei wurde nicht im Lese-Modus geoeffnet")
    line = f.handle.readline()
    return line.rstrip("\r\n")


@builtin(("READALL$", "READALL"), arity=1)
def _b_read_all(f):
    f = _check_file(f, "READALL$")
    if f.mode != "r":
        raise GBRuntimeError("READALL$: Datei wurde nicht im Lese-Modus geoeffnet")
    return f.handle.read()


@builtin("ENDOFFILE", arity=1)
def _b_eof(f):
    f = _check_file(f, "ENDOFFILE")
    pos = f.handle.tell()
    nxt = f.handle.read(1)
    if nxt == "":
        return True
    f.handle.seek(pos)
    return False


@builtin("WRITELINE", arity=2)
def _b_write_line(f, text):
    f = _check_file(f, "WRITELINE")
    if f.mode not in ("w", "a"):
        raise GBRuntimeError("WRITELINE: Datei wurde nicht im Schreib-Modus geoeffnet")
    f.handle.write(_check_str(text, "WRITELINE") + "\n")
    return None


@builtin("WRITE", arity=2)
def _b_write_text(f, text):
    f = _check_file(f, "WRITE")
    if f.mode not in ("w", "a"):
        raise GBRuntimeError("WRITE: Datei wurde nicht im Schreib-Modus geoeffnet")
    f.handle.write(_check_str(text, "WRITE"))
    return None


@builtin("FILEEXISTS", arity=1, types=("str",))
def _b_file_exists(p):
    import os
    return os.path.isfile(p)


# Alle Built-ins werden ueber @builtin in builtins_registry registriert.
# BUILTINS ist ein Alias auf das Registry-Dict, damit vm.py / vm_native.pyx /
# editor.py ihre Imports nicht aendern muessen.
BUILTINS = _REG_BUILTINS


# --- Grafik-Builtins --------------------------------------------------
# Jede Funktion erhaelt (graphics, args). Argumente werden vorher geprueft
# und in INTEGER konvertiert, wo das Schema es erfordert.
#
# `_check_int` ist hier ein Alias auf `_check_intish` aus der Registry --
# beide Funktionen waren historisch identisch (akzeptieren num, liefern
# int zurueck), wurden aber separat gepflegt. Der Alias haelt alle
# Bestands-Aufrufe (`_check_int(args[i], "PLOT")` etc.) lebendig, ohne
# zwei Definitionen mit gleichem Verhalten zu fuehren.
_check_int = _check_intish


@graphics_builtin("SCREEN", arity=(2, 5))
def _g_screen(g, *args):
    w = _check_int(args[0], "SCREEN")
    h = _check_int(args[1], "SCREEN")
    title = args[2] if len(args) >= 3 else "GameBasic"
    if not isinstance(title, str):
        raise TypeMismatchError("SCREEN: Titel muss STRING sein")
    scale = _check_int(args[3], "SCREEN") if len(args) >= 4 else 1
    if scale < 1:
        raise GBRuntimeError("SCREEN: skala muss >= 1 sein")
    fs = False
    if len(args) >= 5:
        if not isinstance(args[4], bool):
            raise TypeMismatchError("SCREEN: fullscreen muss BOOLEAN sein")
        fs = bool(args[4])
    g.screen(w, h, title, scale, fullscreen=fs)
    return None


@graphics_builtin("SET_FULLSCREEN", arity=1, types=("bool",))
def _g_set_fullscreen(g, fs):
    g.set_fullscreen(bool(fs))
    return None


@graphics_builtin("DELTA", arity=0)
def _g_delta(g):
    """Sekunden seit dem letzten FLIP() -- fuer framerate-unabhaengige Bewegung
    (`x = x + speed * DELTA()`)."""
    return float(g.delta())


@graphics_builtin("FPS", arity=0)
def _g_fps(g):
    """Aktuelle Bilder/Sekunde (gleitender Mittelwert)."""
    return int(g.fps())


@graphics_builtin("SETFPS", arity=1, types=("int",))
def _g_setfps(g, n):
    """Ziel-Framerate fuer FLIP setzen (0 = ungedrosselt)."""
    if n < 0:
        raise GBRuntimeError("SETFPS: Wert muss >= 0 sein")
    g.set_target_fps(n)
    return None


@graphics_builtin("SAVESCREENSHOT", arity=1, types=("str",))
def _g_savescreenshot(g, path):
    """Speichert den aktuellen Frame als Bilddatei (Endung bestimmt das Format)."""
    g.save_screenshot(path)
    return None


@graphics_builtin("SETWINDOWTITLE", arity=1, types=("str",))
def _g_setwindowtitle(g, title):
    """Aendert den Fenstertitel zur Laufzeit."""
    g.set_window_title(title)
    return None


@graphics_builtin("CLS", arity=(0, 1))
def _g_cls(g, *args):
    color = _check_int(args[0], "CLS") if args else 0
    g.cls(color)
    return None


@graphics_builtin("PLOT", arity=(2, 3))
def _g_plot(g, *args):
    x = _check_int(args[0], "PLOT")
    y = _check_int(args[1], "PLOT")
    color = _check_int(args[2], "PLOT") if len(args) == 3 else 0xFFFFFF
    g.plot(x, y, color)
    return None


def _plots_seq(a, name):
    """Liefert die 1D-Werteliste eines GB-ARRAYs fuer PLOTS."""
    vals = getattr(a, "values", None)
    if vals is None:
        raise TypeMismatchError(
            f"PLOTS: {name} muss ein 1D-ARRAY OF INTEGER sein")
    return vals


@graphics_builtin("PLOTS", arity=3)
def _g_plots(g, *args):
    """Bulk-Plot: PLOTS(xs, ys, color) zeichnet viele Pixel in einem Aufruf.
    `color` ist ein INTEGER (alle gleich) oder ein ARRAY OF INTEGER (pro Pixel).
    Viel schneller als PLOT in einer Schleife -- ideal fuer Starfields,
    Punktwolken, Pixel-Partikel."""
    xs = _plots_seq(args[0], "xs")
    ys = _plots_seq(args[1], "ys")
    col = args[2]
    if isinstance(col, bool):
        raise TypeMismatchError("PLOTS: color muss INTEGER oder ARRAY sein")
    if isinstance(col, int):
        colors = int(col)
    else:
        colors = getattr(col, "values", None)
        if colors is None:
            raise TypeMismatchError(
                "PLOTS: color muss INTEGER oder ARRAY OF INTEGER sein")
    g.plot_many(xs, ys, colors)
    return None


def _bulk_arr(a, fn, name):
    """Werteliste eines 1D-GB-ARRAYs fuer Bulk-Draw-Builtins."""
    vals = getattr(a, "values", None)
    if vals is None:
        raise TypeMismatchError(f"{fn}: {name} muss ein 1D-ARRAY sein")
    return vals


def _bulk_color_arg(col, fn):
    """color-Argument fuer Bulk-Draw: INTEGER (alle gleich) oder ARRAY OF INT."""
    if isinstance(col, bool):
        raise TypeMismatchError(f"{fn}: color muss INTEGER oder ARRAY sein")
    if isinstance(col, int):
        return int(col)
    v = getattr(col, "values", None)
    if v is None:
        raise TypeMismatchError(
            f"{fn}: color muss INTEGER oder ARRAY OF INTEGER sein")
    return v


@graphics_builtin("BOXES", arity=5)
def _g_boxes(g, *args):
    """Bulk: BOXES(x1s, y1s, x2s, y2s, color) -- viele gefuellte Rechtecke
    in einem Aufruf. Schneller als BOX in einer Schleife (kein Builtin-
    Dispatch pro Shape)."""
    g.box_many(_bulk_arr(args[0], "BOXES", "x1s"),
               _bulk_arr(args[1], "BOXES", "y1s"),
               _bulk_arr(args[2], "BOXES", "x2s"),
               _bulk_arr(args[3], "BOXES", "y2s"),
               _bulk_color_arg(args[4], "BOXES"))
    return None


@graphics_builtin("CIRCLES", arity=4)
def _g_circles(g, *args):
    """Bulk: CIRCLES(xs, ys, rs, color) -- viele Kreise in einem Aufruf."""
    g.circle_many(_bulk_arr(args[0], "CIRCLES", "xs"),
                  _bulk_arr(args[1], "CIRCLES", "ys"),
                  _bulk_arr(args[2], "CIRCLES", "rs"),
                  _bulk_color_arg(args[3], "CIRCLES"))
    return None


@graphics_builtin("LINES", arity=5)
def _g_lines(g, *args):
    """Bulk: LINES(x1s, y1s, x2s, y2s, color) -- viele Linien in einem Aufruf."""
    g.line_many(_bulk_arr(args[0], "LINES", "x1s"),
                _bulk_arr(args[1], "LINES", "y1s"),
                _bulk_arr(args[2], "LINES", "x2s"),
                _bulk_arr(args[3], "LINES", "y2s"),
                _bulk_color_arg(args[4], "LINES"))
    return None


@graphics_builtin("LINE", arity=(4, 5))
def _g_line(g, *args):
    coords = [_check_int(a, "LINE") for a in args[:4]]
    color = _check_int(args[4], "LINE") if len(args) == 5 else 0xFFFFFF
    g.line(*coords, color)
    return None


@graphics_builtin("BOX", arity=(4, 5))
def _g_box(g, *args):
    coords = [_check_int(a, "BOX") for a in args[:4]]
    color = _check_int(args[4], "BOX") if len(args) == 5 else 0xFFFFFF
    g.box(*coords, color)
    return None


@graphics_builtin("RECT", arity=(4, 5))
def _g_rect(g, *args):
    coords = [_check_int(a, "RECT") for a in args[:4]]
    color = _check_int(args[4], "RECT") if len(args) == 5 else 0xFFFFFF
    g.rect(*coords, color)
    return None


@graphics_builtin("CIRCLE", arity=(3, 4))
def _g_circle(g, *args):
    x = _check_int(args[0], "CIRCLE")
    y = _check_int(args[1], "CIRCLE")
    r = _check_int(args[2], "CIRCLE")
    color = _check_int(args[3], "CIRCLE") if len(args) == 4 else 0xFFFFFF
    g.circle(x, y, r, color)
    return None


@graphics_builtin("TRIANGLE", arity=(6, 7))
def _g_triangle(g, *args):
    coords = [_check_int(args[i], "TRIANGLE") for i in range(6)]
    color = _check_int(args[6], "TRIANGLE") if len(args) == 7 else 0xFFFFFF
    g.triangle(*coords, color=color)
    return None


@graphics_builtin("TRIANGLEOUTLINE", arity=(6, 8))
def _g_triangle_outline(g, *args):
    coords = [_check_int(args[i], "TRIANGLEOUTLINE") for i in range(6)]
    color = (_check_int(args[6], "TRIANGLEOUTLINE")
             if len(args) >= 7 else 0xFFFFFF)
    width = (_check_int(args[7], "TRIANGLEOUTLINE")
             if len(args) >= 8 else 1)
    g.triangle_outline(*coords, color=color, width=width)
    return None


@graphics_builtin("POLYGON", arity=(1, 2))
def _g_polygon(g, *args):
    points = args[0]
    color = _check_int(args[1], "POLYGON") if len(args) == 2 else 0xFFFFFF
    g.polygon(points, color)
    return None


@graphics_builtin("POLYGONOUTLINE", arity=(1, 3))
def _g_polygon_outline(g, *args):
    points = args[0]
    color = (_check_int(args[1], "POLYGONOUTLINE")
             if len(args) >= 2 else 0xFFFFFF)
    width = (_check_int(args[2], "POLYGONOUTLINE")
             if len(args) >= 3 else 1)
    g.polygon_outline(points, color=color, width=width)
    return None


@graphics_builtin("ELLIPSE", arity=(4, 5))
def _g_ellipse(g, *args):
    coords = [_check_int(args[i], "ELLIPSE") for i in range(4)]
    color = _check_int(args[4], "ELLIPSE") if len(args) == 5 else 0xFFFFFF
    g.ellipse(*coords, color=color)
    return None


@graphics_builtin("ELLIPSEOUTLINE", arity=(4, 6))
def _g_ellipse_outline(g, *args):
    coords = [_check_int(args[i], "ELLIPSEOUTLINE") for i in range(4)]
    color = (_check_int(args[4], "ELLIPSEOUTLINE")
             if len(args) >= 5 else 0xFFFFFF)
    width = (_check_int(args[5], "ELLIPSEOUTLINE")
             if len(args) >= 6 else 1)
    g.ellipse_outline(*coords, color=color, width=width)
    return None


@graphics_builtin("ARC", arity=(6, 8))
def _g_arc(g, *args):
    coords = [_check_int(args[i], "ARC") for i in range(4)]
    start_angle = _check_num(args[4], "ARC")
    end_angle = _check_num(args[5], "ARC")
    color = _check_int(args[6], "ARC") if len(args) >= 7 else 0xFFFFFF
    width = _check_int(args[7], "ARC") if len(args) >= 8 else 1
    g.arc(*coords, start_angle=start_angle, end_angle=end_angle,
          color=color, width=width)
    return None


@graphics_builtin("TEXT", arity=(3, 4))
def _g_text(g, *args):
    x = _check_int(args[0], "TEXT")
    y = _check_int(args[1], "TEXT")
    s = args[2]
    if not isinstance(s, str):
        raise TypeMismatchError("TEXT: drittes Argument muss STRING sein")
    color = _check_int(args[3], "TEXT") if len(args) == 4 else 0xFFFFFF
    g.text(x, y, s, color)
    return None


@graphics_builtin("TEXT_SIZE", arity=1, types=("intish",))
def _g_text_size(g, size):
    g.text_size(size)
    return None


@graphics_builtin("TEXT_BOLD", arity=1, types=("bool",))
def _g_text_bold(g, on):
    g.text_bold(on)
    return None


@graphics_builtin("TEXT_ITALIC", arity=1, types=("bool",))
def _g_text_italic(g, on):
    g.text_italic(on)
    return None


@graphics_builtin("TEXT_WIDTH", arity=1, types=("str",))
def _g_text_width(g, s):
    return g.text_width(s)


@graphics_builtin("TEXT_HEIGHT", arity=0)
def _g_text_height(g):
    return g.text_height()


@graphics_builtin("SCROLL", arity=2, types=("intish", "intish"))
def _g_scroll(g, dx, dy):
    g.scroll(dx, dy)
    return None


@graphics_builtin("TIMER", arity=0)
def _g_timer(g):
    return g.timer()


@graphics_builtin("JOYSTICK_COUNT", arity=0)
def _g_joystick_count(g):
    return g.joystick_count()


@graphics_builtin("JOYSTICK_NAME", arity=1, types=("intish",))
def _g_joystick_name(g, idx):
    return g.joystick_name(idx)


@graphics_builtin("JOYSTICK_AXIS", arity=2, types=("intish", "intish"))
def _g_joystick_axis(g, idx, axis):
    return g.joystick_axis(idx, axis)


@graphics_builtin("JOYSTICK_BUTTON", arity=2, types=("intish", "intish"))
def _g_joystick_button(g, idx, btn):
    return g.joystick_button(idx, btn)


@graphics_builtin("JOYSTICK_HAT_X", arity=2, types=("intish", "intish"))
def _g_joystick_hat_x(g, idx, hat):
    return g.joystick_hat_x(idx, hat)


@graphics_builtin("JOYSTICK_HAT_Y", arity=2, types=("intish", "intish"))
def _g_joystick_hat_y(g, idx, hat):
    return g.joystick_hat_y(idx, hat)


@graphics_builtin("INKEY$", arity=0)
def _g_inkey(g):
    """Non-blocking: naechstes getipptes Zeichen oder leerer STRING."""
    return g.inkey()


@graphics_builtin("WAITKEY", arity=0)
def _g_waitkey(g):
    """Blockiert bis Taste gedrueckt; gibt SDL-Keycode (INTEGER) zurueck."""
    return g.waitkey()


@graphics_builtin("FLIP", arity=0)
def _g_flip(g):
    g.flip()
    return None


@graphics_builtin("SLEEP", arity=1, types=("intish",))
def _g_sleep(g, ms):
    g.sleep_ms(ms)
    return None


@graphics_builtin("KEYPRESSED", arity=1, types=("intish",))
def _g_keypressed(g, code):
    return g.key_pressed(code)


@graphics_builtin("QUITREQUESTED", arity=0)
def _g_quit_requested(g):
    return g.quit_requested()


@graphics_builtin("MOUSEX", arity=0)
def _g_mouse_x(g):
    return g.mouse_x()


@graphics_builtin("MOUSEY", arity=0)
def _g_mouse_y(g):
    return g.mouse_y()


@graphics_builtin("MOUSEBUTTON", arity=1, types=("intish",))
def _g_mouse_button(g, n):
    return g.mouse_button(n)


@graphics_builtin("LOADIMAGE", arity=1, types=("str",))
def _g_load_image(g, path):
    return _Image(g.load_image(path), path)


@graphics_builtin("DRAWIMAGE", arity=3)
def _g_draw_image(g, img, x, y):
    if img is None:
        raise GBRuntimeError("DRAWIMAGE: Bild ist NIL")
    if not isinstance(img, _Image):
        raise TypeMismatchError("DRAWIMAGE: erstes Argument muss IMAGE sein")
    g.draw_image(img.surface,
                 _check_int(x, "DRAWIMAGE"),
                 _check_int(y, "DRAWIMAGE"))
    return None


@graphics_builtin("DRAWIMAGEPART", arity=7)
def _g_draw_image_part(g, img, sx, sy, sw, sh, x, y):
    if img is None or not isinstance(img, _Image):
        raise TypeMismatchError("DRAWIMAGEPART: erstes Argument muss IMAGE sein")
    sx, sy, sw, sh, x, y = (_check_int(a, "DRAWIMAGEPART")
                             for a in (sx, sy, sw, sh, x, y))
    g.draw_image_part(img.surface, sx, sy, sw, sh, x, y)
    return None


@graphics_builtin("DRAWIMAGEFLIPPED", arity=(3, 5))
def _g_draw_image_flipped(g, *args):
    img = args[0]
    if img is None or not isinstance(img, _Image):
        raise TypeMismatchError("DRAWIMAGEFLIPPED: erstes Argument muss IMAGE sein")
    x = _check_int(args[1], "DRAWIMAGEFLIPPED")
    y = _check_int(args[2], "DRAWIMAGEFLIPPED")
    flip_x = bool(args[3]) if len(args) >= 4 else False
    flip_y = bool(args[4]) if len(args) >= 5 else False
    g.draw_image_flipped(img.surface, x, y, flip_x, flip_y)
    return None


@graphics_builtin("IMAGEWIDTH", arity=1)
def _g_image_width(g, img):
    if not isinstance(img, _Image):
        raise TypeMismatchError("IMAGEWIDTH erwartet IMAGE")
    return img.surface.get_width()


@graphics_builtin("IMAGEHEIGHT", arity=1)
def _g_image_height(g, img):
    if not isinstance(img, _Image):
        raise TypeMismatchError("IMAGEHEIGHT erwartet IMAGE")
    return img.surface.get_height()


@graphics_builtin("LOADSOUND", arity=1, types=("str",))
def _g_load_sound(g, path):
    return _Sound(g.load_sound(path), path)


@graphics_builtin("ATLAS_LOAD", arity=1, types=("str",))
def _g_atlas_load(g, path):
    """Laedt einen Sprite-Atlas aus einer JSON-Datei. Manifest-Format:

        { "image": "atlas.png",
          "sprites": {
            "tile_grass": [0, 0, 32, 32],
            "player_idle": [32, 0, 24, 32]
          } }

    Pfad zum Bild ist relativ zum Manifest. Liefert ein SPRITE_ATLAS,
    das du mit ATLAS_DRAW(atlas, "name", x, y) oder BATCH_DRAW(...) +
    BATCH_FLUSH() benutzt.
    """
    return g.load_sprite_atlas(path)


@graphics_builtin("ATLAS_DRAW", arity=4, types=("any", "str", "intish", "intish"))
def _g_atlas_draw(g, atlas, name, x, y):
    """Zeichnet ein Atlas-Sub-Sprite an (x, y). Camera-aware. Pending
    Batch wird vorher geflusht (richtige Reihenfolge)."""
    if not isinstance(atlas, _SpriteAtlas):
        raise TypeMismatchError(
            "ATLAS_DRAW: erstes Argument muss SPRITE_ATLAS sein"
        )
    g.atlas_draw(atlas, name, x, y)
    return None


@graphics_builtin("ATLAS_DRAW_FLIPPED",
                   arity=(4, 6),
                   types=None)
def _g_atlas_draw_flipped(g, *args):
    """ATLAS_DRAW_FLIPPED(atlas, name$, x, y[, flip_x[, flip_y]]).

    Zeichnet ein Atlas-Sub-Sprite mit horizontaler/vertikaler Spiegelung.
    Camera-aware. flip_x = TRUE spiegelt links/rechts, flip_y oben/unten.
    Klassisch fuer Charakter-Sprites: ein "walk_right"-Frame deckt auch
    "walk_left" ab.

        ATLAS_DRAW_FLIPPED(mario, "walk_a", x, y, TRUE)   ' nach links
    """
    if len(args) < 4:
        raise GBRuntimeError(
            "ATLAS_DRAW_FLIPPED: erwartet (atlas, name, x, y[, flip_x[, flip_y]])"
        )
    atlas = args[0]
    if not isinstance(atlas, _SpriteAtlas):
        raise TypeMismatchError(
            "ATLAS_DRAW_FLIPPED: erstes Argument muss SPRITE_ATLAS sein"
        )
    name = args[1]
    if not isinstance(name, str):
        raise TypeMismatchError("ATLAS_DRAW_FLIPPED: name muss STRING sein")
    x = _check_int(args[2], "ATLAS_DRAW_FLIPPED")
    y = _check_int(args[3], "ATLAS_DRAW_FLIPPED")
    flip_x = bool(args[4]) if len(args) >= 5 else False
    flip_y = bool(args[5]) if len(args) >= 6 else False
    g.atlas_draw_flipped(atlas, name, x, y, flip_x, flip_y)
    return None


@graphics_builtin("BATCH_DRAW", arity=4, types=("any", "str", "intish", "intish"))
def _g_batch_draw(g, atlas, name, x, y):
    """Haengt einen Atlas-Sub-Sprite an die Batch-Queue. Erst BATCH_FLUSH
    (oder FLIP / Layer-Wechsel / Direct-Draw) rendert. Erwartet pygame.
    Surface.blits() im Hot-Pfad -- spart Python-Overhead bei hunderten
    von Sprites (Tilemap, Bullet-Hell)."""
    if not isinstance(atlas, _SpriteAtlas):
        raise TypeMismatchError(
            "BATCH_DRAW: erstes Argument muss SPRITE_ATLAS sein"
        )
    g.atlas_draw_batch(atlas, name, x, y)
    return None


@graphics_builtin("BATCH_FLUSH", arity=0)
def _g_batch_flush(g):
    """Rendert die gequeueten BATCH_DRAW-Sprites jetzt. FLIP und
    Layer-Wechsel rufen das implizit auf."""
    g.batch_flush()
    return None


@graphics_builtin("LAYER_DEFINE", arity=2, types=("str", "int"))
def _g_layer_define(g, name, z):
    """Registriert einen Z-Layer mit explizitem z-Wert. Layer mit
    niedrigerem z werden HINTEN gezeichnet. Mehrfach-Define aktualisiert
    nur das z. SCREEN muss noch nicht aufgerufen sein -- die Surface
    wird lazy beim ersten LAYER(name)-Aufruf allokiert.

    Klassiker:
        LAYER_DEFINE("bg", 0)
        LAYER_DEFINE("sprites", 10)
        LAYER_DEFINE("ui", 100)
    """
    g.layer_define(name, z)
    return None


@graphics_builtin("LAYER", arity=1, types=("str",))
def _g_layer(g, name):
    """Switcht den aktiven Layer. Alle folgenden draw-Calls (CLS, PLOT,
    DRAWIMAGE, TEXT, ...) gehen auf diese Layer-Surface. FLIP composiert
    alle Layer in z-Order auf den Screen.

    Layer, die nicht zuvor mit LAYER_DEFINE registriert wurden, kriegen
    auto-generiertes z (next-after-highest).
    """
    g.layer_use(name)
    return None


@graphics_builtin("LAYER_END", arity=0)
def _g_layer_end(g):
    """Verlaesst den Layer-Kontext. Folgende draw-Calls gehen direkt auf
    den Main-Buffer (kein Layer-Compose-Detour). FLIP ruft das implizit
    auf -- der Aufruf ist optional."""
    g.layer_end()
    return None


@graphics_builtin("LAYER_CLEAR", arity=1, types=("str",))
def _g_layer_clear(g, name):
    """Cleart einen Layer manuell (transparent). FLIP cleart alle Layer
    automatisch -- diese Funktion ist nur fuer Spezialfaelle."""
    g.layer_clear(name)
    return None


@graphics_builtin("LOAD_ASSETS", arity=1, types=("str",))
def _g_load_assets(g, manifest_path):
    """Lese ein JSON-Manifest und lade alle gelisteten Bilder/Sounds vorab
    in den Asset-Cache. Liefert die Gesamtanzahl der geladenen Assets.

    Manifest-Format:
        {
          "images": { "player": "sprites/player.png",
                      "enemy":  "sprites/enemy.png" },
          "sounds": [ "sfx/jump.wav", "music/level1.ogg" ]
        }

    Beide Sektionen sind optional. Jede kann entweder ein Object (Alias ->
    Pfad) ODER eine Liste von Pfaden sein. Pfade sind relativ zum
    Manifest-Verzeichnis.

    Nach LOAD_ASSETS:
      - LOADIMAGE("player") trifft den Cache (Alias-Hit)
      - LOADIMAGE("sprites/player.png") trifft den Cache (Pfad-Hit)
      - LOADIMAGE absolut oder relativ-zu-cwd trifft auch den Cache
        (Pfad wird normalisiert)

    Reihenfolge-Hinweis: am besten NACH SCREEN(...) aufrufen, damit Bilder
    direkt convert_alpha-optimiert werden -- sonst passiert die Konvertierung
    erst beim ersten DRAWIMAGE.
    """
    import json as _json
    import os as _os
    try:
        with open(manifest_path, "r", encoding="utf-8") as f:
            manifest = _json.load(f)
    except FileNotFoundError:
        raise GBRuntimeError(
            f"LOAD_ASSETS: Manifest nicht gefunden: {manifest_path}"
        )
    except Exception as exc:
        raise GBRuntimeError(
            f"LOAD_ASSETS: Manifest-Lesefehler '{manifest_path}': {exc}"
        )
    if not isinstance(manifest, dict):
        raise GBRuntimeError(
            "LOAD_ASSETS: Manifest muss ein JSON-Object sein"
        )
    manifest_dir = _os.path.dirname(_os.path.abspath(manifest_path))

    def _resolve(rel):
        if not isinstance(rel, str):
            raise GBRuntimeError(
                f"LOAD_ASSETS: Pfad muss STRING sein, erhalten {type(rel).__name__}"
            )
        return _os.path.normpath(_os.path.join(manifest_dir, rel))

    count = 0
    images = manifest.get("images")
    if isinstance(images, dict):
        for alias, rel in images.items():
            if not isinstance(alias, str):
                raise GBRuntimeError(
                    "LOAD_ASSETS: Image-Alias muss STRING sein"
                )
            full = _resolve(rel)
            surf = g.load_image(full)
            g.cache_image_under(alias, surf)
            count += 1
    elif isinstance(images, list):
        for rel in images:
            g.load_image(_resolve(rel))
            count += 1
    elif images is not None:
        raise GBRuntimeError(
            "LOAD_ASSETS: 'images' muss Object oder Array sein"
        )

    sounds = manifest.get("sounds")
    if isinstance(sounds, dict):
        for alias, rel in sounds.items():
            if not isinstance(alias, str):
                raise GBRuntimeError(
                    "LOAD_ASSETS: Sound-Alias muss STRING sein"
                )
            full = _resolve(rel)
            snd = g.load_sound(full)
            g.cache_sound_under(alias, snd)
            count += 1
    elif isinstance(sounds, list):
        for rel in sounds:
            g.load_sound(_resolve(rel))
            count += 1
    elif sounds is not None:
        raise GBRuntimeError(
            "LOAD_ASSETS: 'sounds' muss Object oder Array sein"
        )

    return count


@graphics_builtin("PLAYSOUND", arity=(1, 3))
def _g_play_sound(g, *args):
    s = args[0]
    if s is None or not isinstance(s, _Sound):
        raise TypeMismatchError("PLAYSOUND: erstes Argument muss SOUND sein")
    loops = _check_int(args[1], "PLAYSOUND") if len(args) >= 2 else 0
    if len(args) == 3:
        v = args[2]
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise TypeMismatchError("PLAYSOUND: Lautstaerke muss Zahl sein (0..1)")
        volume = float(v)
    else:
        volume = 1.0
    g.play_sound(s.sound, loops, volume)
    return None


@graphics_builtin("STOPSOUND", arity=1)
def _g_stop_sound(g, s):
    if not isinstance(s, _Sound):
        raise TypeMismatchError("STOPSOUND erwartet SOUND")
    g.stop_sound(s.sound)
    return None


@graphics_builtin("PLAYMUSIC", arity=(1, 3))
def _g_play_music(g, *args):
    if not isinstance(args[0], str):
        raise GBRuntimeError("PLAYMUSIC erwartet (pfad$[, loops[, lautstaerke]])")
    loops = _check_int(args[1], "PLAYMUSIC") if len(args) >= 2 else -1
    if len(args) == 3:
        v = args[2]
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise TypeMismatchError("PLAYMUSIC: Lautstaerke muss Zahl sein")
        volume = float(v)
    else:
        volume = 1.0
    g.play_music(args[0], loops, volume)
    return None


@graphics_builtin("STOPMUSIC", arity=0)
def _g_stop_music(g):
    g.stop_music()
    return None


@graphics_builtin("DRAWTILEMAP", arity=6)
def _g_draw_tilemap(g, tileset, map_arr, tw, th, sx, sy):
    """DRAWTILEMAP(tileset, map, tileW, tileH, screenX, screenY)
    map ist 2D ARRAY OF INTEGER mit Tile-Indizes; -1 = transparent.
    Tileset wird als horizontaler oder gerasterter Strip interpretiert.
    """
    if not isinstance(tileset, _Image):
        raise TypeMismatchError("DRAWTILEMAP: tileset muss IMAGE sein")
    if not isinstance(map_arr, _GBArray) or map_arr.element_type != "integer":
        raise TypeMismatchError("DRAWTILEMAP: map muss ARRAY OF INTEGER sein")
    if len(map_arr.dims) != 2:
        raise GBRuntimeError("DRAWTILEMAP: map muss 2D sein (zeilen x spalten)")
    tw = _check_int(tw, "DRAWTILEMAP")
    th = _check_int(th, "DRAWTILEMAP")
    sx = _check_int(sx, "DRAWTILEMAP")
    sy = _check_int(sy, "DRAWTILEMAP")
    rows, cols = map_arr.dims
    # Batch: ein blits()-Call statt rows*cols Einzel-blits (bei zoom==1).
    g.draw_tilemap(tileset.surface, map_arr.values, rows, cols, tw, th, sx, sy)
    return None


# Alle Grafik-Builtins werden ueber @graphics_builtin in builtins_registry
# registriert. GRAPHICS_BUILTINS ist ein Alias auf das Registry-Dict.
GRAPHICS_BUILTINS = _REG_GFX_BUILTINS
