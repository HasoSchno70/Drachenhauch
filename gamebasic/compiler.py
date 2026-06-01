"""Compiler: AST -> Bytecode (Phase 3a, Primitive-Subset).

Was unterstuetzt wird:
- INTEGER / FLOAT / STRING / BOOLEAN, DIM, CONST (top-level)
- Arithmetik, Vergleich, AND/OR (short-circuit), NOT
- IF/ELSEIF/ELSE/END IF (block + single-line), WHILE/WEND, FOR/TO/STEP/NEXT
- BREAK / CONTINUE
- SUB, FUNCTION, RETURN (mit Rekursion und voller Parameter-Typpruefung)
- PRINT mit mehreren Items, INPUT mit/ohne Prompt
- Aufruf von BUILTINS-Funktionen (STR$, INT, ABS, RND, ...)

Noch NICHT (Phase 3b/3c):
- CLASS, NEW, MemberAccess, Methoden
- Arrays, STRUCT, Strings-Funktionen, Datei-I/O, Grafik
"""
from __future__ import annotations

from .ast_nodes import (
    NumberLit, StringLit, BoolLit, Identifier, BinaryOp, UnaryOp, Call,
    Dim, MultiDim, Assign, Print, Input, If, While, Repeat, For, ExprStmt, Program,
    Data, Read, Restore,
    Param, SubDecl, FunctionDecl, Return,
    Const, Break, Continue,
    ClassDecl, New, MemberAccess, MemberAssign,
    IndexAccess, IndexAssign,
    Try, Throw, Select, CaseMatch,
    EnumDecl, NamedArg, TupleLit, TupleAssign, With, SliceAccess,
    ListComp, DictComp, SetComp,
)
from .bytecode import Op, CompiledFunction, Module, VMClassInfo, VMFieldDecl
from .errors import GameBasicError
from .builtins_registry import BUILTINS, GRAPHICS_BUILTINS


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


# Operator-zu-Spezialisierungs-Mapping. Wenn der Compiler statisch
# erkennt, dass beide Operanden numerisch sind (kein STRING, kein BOOL,
# keine User-Klasse), emittiert er die _NN-Variante. Diese Ops haben
# einen Hot-Path ohne Operator-Overload-Dispatch und ohne isinstance-
# Cascade -- nur `pop b; pop a; push a OP b`.
_NUMERIC_SPEC_OPS = {
    "+":  Op.ADD_NN,
    "-":  Op.SUB_NN,
    "*":  Op.MUL_NN,
    "/":  Op.DIV_NN,
    "<":  Op.LT_NN,
    ">":  Op.GT_NN,
    "<=": Op.LEQ_NN,
    ">=": Op.GEQ_NN,
    "=":  Op.EQ_NN,
    "<>": Op.NEQ_NN,
}


class CompileError(GameBasicError):
    pass


class _FnCtx:
    """Begleitet die Compilation einer einzelnen Funktion."""

    def __init__(self, name: str, is_main: bool):
        self.name = name
        self.is_main = is_main
        self.code: list = []
        # Quell-Zeile pro Instruction (parallel zu code) + aktuell aktive Zeile,
        # gesetzt vom Statement-Dispatch. Fuer Laufzeitfehler in der nativen
        # Runtime (datei.gb:Zeile).
        self.lines: list = []
        self.cur_line: int = 0
        self.constants: list = []
        self.const_index: dict = {}
        self.local_slots: dict = {}
        self.local_types: list = []
        self.local_defaults: list = []
        self.n_params: int = 0
        self.return_type: str = ""
        self.is_sub: bool = True
        self.current_class: VMClassInfo | None = None
        # Stacks fuer BREAK/CONTINUE-Patches (jeweils Liste pro Schleife);
        # zusaetzlich speichern wir die TRY-Tiefe beim Loop-Start, um beim
        # vorzeitigen Verlassen die noetige Anzahl TRY_END einzufuegen.
        self.break_patches: list = []        # list[(list[int], try_depth)]
        self.continue_patches: list = []     # list[(list[int], try_depth)]
        self.try_depth: int = 0

    def add_const(self, value) -> int:
        try:
            key = (type(value).__name__, value)
            if key in self.const_index:
                return self.const_index[key]
        except TypeError:
            key = None
        idx = len(self.constants)
        self.constants.append(value)
        if key is not None:
            self.const_index[key] = idx
        return idx

    def emit(self, op: int, arg=None) -> int:
        ip = len(self.code)
        self.code.append((op, arg))
        self.lines.append(self.cur_line)
        return ip

    def patch_jump(self, ip: int, target: int):
        old_op, _ = self.code[ip]
        self.code[ip] = (old_op, target)


class Compiler:
    def __init__(self):
        self.functions: dict = {}
        self.classes: dict = {}        # name -> VMClassInfo
        self.fn: _FnCtx | None = None
        # Top-Level-Dim/Const-Namen -- damit der FUNCREF-Pfad eine globale
        # Variable nicht versehentlich als FuncRef emittiert. User-Variable
        # gewinnt ueber Function-mit-gleichem-Namen (konsistent mit dem
        # Tree-Walker, wo `env.has(name)` Vorrang hat).
        self._global_vars: set = set()
        # Top-Level-Variablen-Typen fuer die statische Type-Inference --
        # parallel zu `_global_vars`, aber Wert ist der declared type
        # (lower-case, normalisiert). Wird in der Compile-Eintrittsphase
        # gefuellt; `_expr_type` konsultiert es fuer Identifier, die nicht
        # als Local existieren.
        self._global_types: dict = {}
        # Slot-Indizes fuer Compile-Zeit-bekannte Globals. name -> slot.
        # Wird parallel zu _global_vars befuellt. Compiler emittiert dann
        # LOAD_GLOBAL_SLOT / STORE_GLOBAL_SLOT / DECLARE_GLOBAL_SLOT statt
        # die name-basierten Varianten. Pre-registrierte Globals
        # (KEY_*, PI, BLACK, ...) sind NICHT enthalten -- sie gehen
        # weiterhin durch das globals_-Dict.
        self._global_slots: dict = {}

    # -------- Eintritt --------------------------------------------------
    def compile(self, program: Program) -> Module:
        fn_decls = []
        cls_decls = []
        main_stmts = []
        for stmt in program.statements:
            if isinstance(stmt, (SubDecl, FunctionDecl)):
                fn_decls.append(stmt)
            elif isinstance(stmt, ClassDecl):
                cls_decls.append(stmt)
            else:
                main_stmts.append(stmt)
        # Top-Level-Variable-Namen sammeln (DIM und CONST und MultiDim).
        # Brauchen wir vor jedem _expr_Identifier-Lookup -- sonst wuerde
        # `DIM foo AS INTEGER : foo = 99 : PRINT foo` nach einer Function
        # `foo` fragen statt der Variable.
        # Gleichzeitig _global_types fuellen -- die statische Type-Inference
        # fuer Top-Level-Code (FOR-Loops in main etc.) braucht das, sonst
        # bleiben Globals immer "unbekannt" und die spec-ops feuern nie.
        # Gleichzeitig _global_slots fuellen, damit Slot-basierter Zugriff
        # (LOAD_GLOBAL_SLOT) emittiert werden kann. Die Slot-Indizes sind
        # stabil in Source-Reihenfolge -- Funktionen koennen Globals
        # referenzieren, die spaeter im Hauptprogramm deklariert sind
        # (Forward-Reference), weil das Pre-Scan vor der Funktions-
        # Kompilierung laeuft.
        def _alloc_slot(name):
            if name not in self._global_slots:
                self._global_slots[name] = len(self._global_slots)

        # Struct-Klassen-Namen sammeln. Top-Level-DIMs auf einen STRUCT-
        # Typ bekommen KEINEN Slot, weil ihre Init ueber DECLARE_STRUCT_NAME
        # laeuft (special-case-Allokation eines _Instance + Auto-Felder).
        # Slot-version waere moeglich, lohnt sich aber nicht -- Structs
        # sind im Hot-Path selten.
        _struct_names = {cd.name for cd in cls_decls if cd.is_struct}

        def _is_simple_dim(dim_node):
            """True wenn der Dim slot-kompatibel ist (scalar/class-instance).
            False fuer ARRAY/MAP/STRUCT, die spezielle Init-Ops haben."""
            if getattr(dim_node, "array_dims", None) is not None:
                return False
            t = (dim_node.type_name or "").lower()
            if t.startswith("array:") or t.startswith("map:"):
                return False
            if dim_node.type_name in _struct_names:
                return False
            return True

        for s in main_stmts:
            if isinstance(s, Dim):
                self._global_vars.add(s.name)
                self._global_types[s.name] = (s.type_name or "").lower()
                if _is_simple_dim(s):
                    _alloc_slot(s.name)
            elif isinstance(s, MultiDim):
                for d in s.dims:
                    self._global_vars.add(d.name)
                    self._global_types[d.name] = (d.type_name or "").lower()
                    if _is_simple_dim(d):
                        _alloc_slot(d.name)
            elif isinstance(s, Const):
                self._global_vars.add(s.name)
                _alloc_slot(s.name)
                # CONST-Typ aus dem Wert ableiten (best-effort, nur Literale)
                if isinstance(s.value, NumberLit):
                    v = s.value.value
                    if isinstance(v, bool):
                        self._global_types[s.name] = "boolean"
                    elif isinstance(v, int):
                        self._global_types[s.name] = "integer"
                    else:
                        self._global_types[s.name] = "float"
                elif isinstance(s.value, BoolLit):
                    self._global_types[s.name] = "boolean"
                elif isinstance(s.value, StringLit):
                    self._global_types[s.name] = "string"
                # andere CONST-Initializer: unbekannt (kein Eintrag)
            elif isinstance(s, EnumDecl):
                _alloc_slot(s.name)
            elif isinstance(s, For):
                # Top-Level FOR-Var bekommt automatisch INTEGER + Slot.
                _alloc_slot(s.var)
                if s.var not in self._global_types:
                    self._global_types[s.var] = "integer"

        # Klassen-Statics: cd.name wird zur globalen CONST mit dem
        # _ClassStaticNamespace als Wert. Auch ein Slot.
        for cd in cls_decls:
            if cd.statics:
                _alloc_slot(cd.name)

        # Phase 1: Klassen registrieren (Felder + Methodennamen)
        for cd in cls_decls:
            self._at(cd, lambda cd=cd: self._register_class(cd))
        # Eltern-Referenzen aufloesen
        for ci in self.classes.values():
            if ci.parent_name:
                p = self.classes.get(ci.parent_name)
                if p is None:
                    raise CompileError(
                        f"CLASS '{ci.name}': Elternklasse '{ci.parent_name}' nicht gefunden"
                    )
                ci.parent = p

        # Phase 2: Funktions-Stubs (Forward-Referenzen)
        for d in fn_decls:
            self._at(d, lambda d=d: self._register_stub(d))

        # Phase 3: Funktionen kompilieren
        for d in fn_decls:
            self._at(d, lambda d=d: self._compile_function(d, current_class=None))

        # Phase 4a: Methoden-Stubs eintragen, BEVOR irgendeine kompiliert wird.
        # So kann beim Kompilieren von Methode A bereits eine Methode B
        # (in derselben Klasse) als implizit-aufrufbar erkannt werden.
        from .bytecode import CompiledFunction as _CF
        for cd in cls_decls:
            ci = self.classes[cd.name]
            for m in cd.methods:
                ci.methods[m.name] = _CF(name=m.name)

        # Phase 4b: Methoden kompilieren (Klassen-Reihenfolge)
        for cd in cls_decls:
            ci = self.classes[cd.name]
            for m in cd.methods:
                fn = self._at(m, lambda m=m, ci=ci:
                              self._compile_function(m, current_class=ci))
                ci.methods[m.name] = fn

        # Phase 5: Hauptprogramm
        ctx = _FnCtx("__main__", is_main=True)
        self.fn = ctx
        # Klassen-Statics als CONST in den globalen Scope hoisten -- vor
        # allen User-Statements, damit Top-Level-Code sie sehen kann.
        for cd in cls_decls:
            self._emit_class_statics(cd)
        for s in main_stmts:
            self._stmt(s)
        ctx.emit(Op.HALT)
        main_fn = self._finalize(ctx)

        # Phase 6: DATA-Werte aus dem ganzen Programm einsammeln (Source-
        # Reihenfolge, auch aus SUB/FUNCTION/CLASS-Bodies).
        data = []
        _collect_data(program.statements, data)

        return Module(
            main=main_fn,
            functions=self.functions,
            classes=self.classes,
            data=data,
            n_globals=len(self._global_slots),
        )

    def _register_class(self, decl: ClassDecl):
        if decl.name in self.classes:
            kind = "STRUCT" if decl.is_struct else "CLASS"
            raise CompileError(f"{kind} '{decl.name}' bereits deklariert")
        ci = VMClassInfo(
            name=decl.name,
            parent_name=decl.parent or "",
            is_struct=decl.is_struct,
        )
        for fd in decl.fields:
            dims: tuple = ()
            if fd.array_dims is not None:
                # Compile-time-konstante Dims erforderlich
                resolved = []
                for de in fd.array_dims:
                    if not isinstance(de, NumberLit) or not isinstance(de.value, int):
                        raise CompileError(
                            f"Array-Feld '{fd.name}' braucht konstante INTEGER-Groesse "
                            f"(in der VM)"
                        )
                    resolved.append(de.value)
                dims = tuple(resolved)
            ci.fields.append(VMFieldDecl(
                name=fd.name,
                type_name=fd.type_name,
                array_dims=dims,
            ))
        # Property-Namen sammeln (lowercase). Erlaubt schnellen MemberAccess-
        # und MemberAssign-Lookup in der VM.
        for pd in (decl.properties or ()):
            ci.properties.add(pd.name.lower())
        self.classes[decl.name] = ci

    def _register_stub(self, decl):
        is_sub = isinstance(decl, SubDecl)
        n_params = len(decl.params)
        # BYREF-Parameter sind im VM-/Native-VM-Pfad noch nicht implementiert.
        # Der Tree-Walker macht sie korrekt; wer Bytecode-Speed braucht,
        # arbeitet aktuell mit Return-Werten + ARRAY-OF-T fuer Multi-Return.
        for p in decl.params:
            if p.by_ref:
                raise CompileError(
                    f"{decl.name}: BYREF-Parameter '{p.name}' werden im "
                    f"VM-Pfad noch nicht unterstuetzt - bitte den Tree-"
                    f"Walker verwenden (gbrun.py ohne --vm)."
                )
        # Default-Werte fuer optionale Parameter zu Compile-Zeit auswerten.
        # Nur Literale unterstuetzt - Param-referenzierende Defaults
        # erfordern den Tree-Walker (mit klarer Fehlermeldung).
        param_defaults = []
        n_required = 0
        for p in decl.params:
            if p.default is None:
                param_defaults.append(None)
                if any(d is not None for d in param_defaults):
                    # Sollte vom Parser schon abgefangen sein.
                    pass
                else:
                    n_required += 1
            else:
                param_defaults.append(_eval_literal_default(p.default, decl.name))
        # n_required = Anzahl bis zum ersten Default
        n_required = next(
            (i for i, d in enumerate(param_defaults) if d is not None),
            n_params,
        )
        is_variadic = bool(decl.params) and getattr(decl.params[-1], "is_variadic", False)
        fn = CompiledFunction(
            name=decl.name,
            n_params=n_params,
            n_required=n_required,
            param_defaults=param_defaults,
            param_names=[p.name.lower() for p in decl.params],
            local_types=[p.type_name for p in decl.params],
            local_defaults=[_TYPE_DEFAULTS.get(p.type_name) for p in decl.params],
            return_type=("" if is_sub else decl.return_type),
            is_sub=is_sub,
            is_variadic=is_variadic,
        )
        if decl.name in self.functions:
            kind = "SUB" if is_sub else "FUNCTION"
            raise CompileError(f"{kind} '{decl.name}' bereits deklariert")
        self.functions[decl.name] = fn

    def _compile_function(self, decl, current_class=None) -> CompiledFunction:
        is_sub = isinstance(decl, SubDecl)
        ctx = _FnCtx(decl.name, is_main=False)
        ctx.is_sub = is_sub
        ctx.return_type = "" if is_sub else decl.return_type
        ctx.current_class = current_class
        for p in decl.params:
            slot = len(ctx.local_slots)
            ctx.local_slots[p.name] = slot
            ctx.local_types.append(p.type_name)
            ctx.local_defaults.append(_TYPE_DEFAULTS.get(p.type_name))
        ctx.n_params = len(decl.params)
        self.fn = ctx
        for s in decl.body:
            self._stmt(s)
        if is_sub:
            ctx.emit(Op.RETURN_VOID)
        else:
            ctx.emit(Op.LOAD_CONST, ctx.add_const(
                f"__missing_return:{decl.name}"
            ))
            ctx.emit(Op.HALT)

        compiled = self._finalize(ctx)
        # Param-Namen + -Defaults sind Compile-Zeit-Info aus dem decl.
        # (Stub-Funktionen haben das schon - hier nur fuer Methoden noetig.)
        compiled.param_names = [p.name.lower() for p in decl.params]
        method_defaults = []
        for p in decl.params:
            if p.default is None:
                method_defaults.append(None)
            else:
                method_defaults.append(_eval_literal_default(p.default, decl.name))
        compiled.param_defaults = method_defaults
        # n_required = Index des ersten Default-Slots (oder n_params wenn keiner)
        compiled.n_required = next(
            (i for i, d in enumerate(method_defaults) if d is not None),
            len(decl.params),
        )
        compiled.is_variadic = bool(decl.params) and getattr(
            decl.params[-1], "is_variadic", False)
        # Top-level Funktionen haben Stubs
        if current_class is None:
            stub = self.functions[decl.name]
            stub.code = compiled.code
            stub.constants = compiled.constants
            stub.local_types = compiled.local_types
            stub.local_defaults = compiled.local_defaults
            stub.caches = compiled.caches
            stub.lines = compiled.lines
            return stub
        return compiled

    def _finalize(self, ctx: _FnCtx) -> CompiledFunction:
        return CompiledFunction(
            name=ctx.name,
            code=ctx.code,
            constants=ctx.constants,
            n_params=ctx.n_params,
            local_types=list(ctx.local_types),
            local_defaults=list(ctx.local_defaults),
            return_type=ctx.return_type,
            is_sub=ctx.is_sub,
            is_main=ctx.is_main,
            caches=[None] * len(ctx.code),  # Inline-Cache-Slots, lazy gefuellt
            lines=list(ctx.lines),
        )

    # -------- Statements -----------------------------------------------
    def _at(self, node, thunk):
        """Fuehrt `thunk()` aus und haengt `node.line` an eine CompileError, die
        noch keine Zeile traegt. So bekommen Compile-Fehler eine Quell-Zeile
        (`[Zeile N]`), genau wie Parser-Fehler -- relevant fuer die native
        Runtime / den --native-Pfad, wo es keinen Tree-Walker-Fallback gibt."""
        try:
            return thunk()
        except CompileError as e:
            e.set_line(getattr(node, "line", 0))
            raise

    def _stmt(self, s):
        # Aktuelle Quell-Zeile fuer alle Instruktionen dieses Statements
        # stempeln (der Parser haengt `.line` an jedes Statement).
        ln = getattr(s, "line", 0)
        if ln:
            self.fn.cur_line = ln
        method = getattr(self, f"_stmt_{type(s).__name__}", None)
        if method is None:
            raise CompileError(
                f"VM unterstuetzt {type(s).__name__} noch nicht (Phase 3b/3c)",
                line=ln,
            )
        # Compile-Fehler im Statement-Body kriegen die Statement-Zeile.
        try:
            method(s)
        except CompileError as e:
            e.set_line(ln)
            raise

    def _stmt_MultiDim(self, s: MultiDim):
        """Mehrere DIMs gleichen Typs: einfach hintereinander emittieren.

        Die Bytecode-Ausgabe ist identisch zu mehreren `_stmt_Dim`-Aufrufen,
        deshalb sieht die VM nur normale Single-DIM-Operationen.
        """
        for d in s.dims:
            self._stmt_Dim(d)

    def _stmt_Dim(self, s: Dim):
        from .modules import EXTERNAL_TYPES as _EXT_TYPES
        type_name = s.type_name
        is_known_class = type_name in self.classes
        is_array_t = type_name.startswith("array:")
        is_map_t = type_name.startswith("map:")
        is_primitive = type_name in _TYPE_DEFAULTS
        is_external = type_name in _EXT_TYPES

        # Case: Multi-dim Array mit Groessen
        if s.array_dims is not None:
            elem = type_name
            if (elem not in _TYPE_DEFAULTS and elem not in self.classes
                    and elem not in _EXT_TYPES):
                raise CompileError(f"Unbekannter Array-Element-Typ: {elem}")
            for de in s.array_dims:
                self._expr(de)
            num_dims = len(s.array_dims)
            if self.fn.is_main:
                name_idx = self.fn.add_const(s.name)
                self.fn.emit(Op.DECLARE_ARRAY_NAME, (name_idx, elem, num_dims))
            else:
                if s.name in self.fn.local_slots:
                    # Idempotent (Schleifenkoerper). Dim-Werte vom Stack popppen.
                    for _ in range(num_dims):
                        self.fn.emit(Op.POP)
                    return
                slot = len(self.fn.local_slots)
                self.fn.local_slots[s.name] = slot
                self.fn.local_types.append(f"array:{elem}")
                self.fn.local_defaults.append(None)
                self.fn.emit(Op.DECLARE_ARRAY_LOCAL, (slot, elem, num_dims))
            return

        # Case: ARRAY OF T (groessenlos) - reiner Slot
        if is_array_t:
            if self.fn.is_main:
                name_idx = self.fn.add_const(s.name)
                type_idx = self.fn.add_const(type_name)
                default_idx = self.fn.add_const(None)
                self.fn.emit(Op.DECLARE_NAME, (name_idx, type_idx, default_idx))
            else:
                self._declare_local(s.name, type_name)
            return

        # Case: MAP OF T - VM allokiert leere Map automatisch
        if is_map_t:
            if self.fn.is_main:
                name_idx = self.fn.add_const(s.name)
                type_idx = self.fn.add_const(type_name)
                default_idx = self.fn.add_const(None)  # VM ersetzt durch leere Map
                self.fn.emit(Op.DECLARE_NAME, (name_idx, type_idx, default_idx))
            else:
                self._declare_local(s.name, type_name)
            return

        # Case: STRUCT - auto-init
        cls = self.classes.get(type_name)
        if cls is not None and cls.is_struct:
            if self.fn.is_main:
                name_idx = self.fn.add_const(s.name)
                self.fn.emit(Op.DECLARE_STRUCT_NAME, (name_idx, type_name))
            else:
                if s.name in self.fn.local_slots:
                    return
                slot = len(self.fn.local_slots)
                self.fn.local_slots[s.name] = slot
                self.fn.local_types.append(type_name)
                self.fn.local_defaults.append(None)
                self.fn.emit(Op.DECLARE_STRUCT_LOCAL, (slot, type_name))
            return

        # Case: Klasse, Primitive oder externer Typ - default je Typ
        # (None fuer Klassen und externe Typen).
        if not (is_primitive or is_known_class or is_array_t or is_map_t
                or is_external):
            raise CompileError(f"Unbekannter Typ '{type_name}' bei DIM {s.name}")
        default = _TYPE_DEFAULTS.get(type_name)  # None fuer Klassen/extern
        if self.fn.is_main:
            name_idx = self.fn.add_const(s.name)
            type_idx = self.fn.add_const(type_name)
            default_idx = self.fn.add_const(default)
            slot = self._global_slots.get(s.name)
            if slot is not None:
                self.fn.emit(
                    Op.DECLARE_GLOBAL_SLOT,
                    (slot, name_idx, type_idx, default_idx),
                )
            else:
                self.fn.emit(Op.DECLARE_NAME, (name_idx, type_idx, default_idx))
        else:
            self._declare_local(s.name, type_name)

    def _declare_local(self, name: str, type_name: str) -> int:
        if name in self.fn.local_slots:
            slot = self.fn.local_slots[name]
            if self.fn.local_types[slot] != type_name:
                raise CompileError(
                    f"DIM '{name}': Typkonflikt mit vorheriger Deklaration"
                )
            return slot  # idempotent (z.B. DIM in Schleife)
        slot = len(self.fn.local_slots)
        self.fn.local_slots[name] = slot
        self.fn.local_types.append(type_name)
        default = _TYPE_DEFAULTS.get(type_name)  # None fuer Klassen / Array-Slots
        self.fn.local_defaults.append(default)
        self.fn.emit(Op.DECLARE_LOCAL, (slot, type_name, default))
        return slot

    def _alloc_temp_local(self, type_name: str) -> int:
        """Allokiert einen anonymen Slot (z.B. fuer FOR-end/step)."""
        slot = len(self.fn.local_slots)
        self.fn.local_slots[f"__tmp_{slot}"] = slot
        self.fn.local_types.append(type_name)
        self.fn.local_defaults.append(_TYPE_DEFAULTS.get(type_name, None))
        self.fn.emit(Op.DECLARE_LOCAL, (slot, type_name, _TYPE_DEFAULTS[type_name]))
        return slot

    def _stmt_Const(self, s: Const):
        self._expr(s.value)
        if not self.fn.is_main:
            raise CompileError(
                "CONST in der VM nur auf Top-Level erlaubt (Phase 3a)"
            )
        name_idx = self.fn.add_const(s.name)
        type_idx = self.fn.add_const(s.type_name)  # darf None sein -> abgeleitet
        slot = self._global_slots.get(s.name)
        if slot is not None:
            self.fn.emit(
                Op.DECLARE_GLOBAL_CONST_SLOT, (slot, name_idx, type_idx)
            )
        else:
            self.fn.emit(Op.DECLARE_CONST, (name_idx, type_idx))

    def _emit_class_statics(self, cd: ClassDecl):
        """Klassen-Statics als globale CONST mit _ClassStaticNamespace-Wert
        registrieren -- analog zu ENUM. Werte muessen Compile-Zeit-Literale
        sein, sonst CompileError.
        """
        if not cd.statics:
            return
        from .interpreter import _ClassStaticNamespace
        members: dict = {}
        for c in cd.statics:
            key = c.name.lower()
            if key in members:
                raise CompileError(
                    f"CLASS {cd.name}: STATIC CONST '{c.name}' "
                    f"doppelt deklariert"
                )
            value = _eval_static_class_literal(c.value, cd.name, c.name)
            members[key] = value
        ns = _ClassStaticNamespace(cd.name, members)
        name_idx = self.fn.add_const(cd.name)
        type_idx = self.fn.add_const(None)   # _infer_type liefert "class_static"
        val_idx = self.fn.add_const(ns)
        self.fn.emit(Op.LOAD_CONST, val_idx)
        slot = self._global_slots.get(cd.name)
        if slot is not None:
            self.fn.emit(
                Op.DECLARE_GLOBAL_CONST_SLOT, (slot, name_idx, type_idx)
            )
        else:
            self.fn.emit(Op.DECLARE_CONST, (name_idx, type_idx))

    def _stmt_EnumDecl(self, s: EnumDecl):
        """ENUM zur Compile-Zeit aufloesen: Member-Werte muessen Literale
        sein. Das Resultat ist ein _EnumNamespace im Const-Pool, der dann
        wie eine normale CONST-Variable global abgelegt wird.

        Member-Access (`State.MENU`) wird ganz normal ueber LOAD_MEMBER
        gehandhabt - die VM-Schleife erkennt _EnumNamespace dort und liefert
        den Member-Wert."""
        if not self.fn.is_main:
            raise CompileError("ENUM nur auf Top-Level erlaubt")
        from .interpreter import _EnumNamespace
        members: dict = {}
        next_auto = 0
        for mname, expr in s.members:
            if expr is None:
                value = next_auto
            else:
                value = _eval_int_literal(expr)
                if value is None:
                    raise CompileError(
                        f"ENUM {s.name}.{mname}: Wert muss ein Integer-Literal sein "
                        f"(z.B. 5 oder -1) - Ausdruecke werden zur Compile-Zeit nicht "
                        f"ausgewertet"
                    )
            members[mname.lower()] = value
            next_auto = value + 1
        ns = _EnumNamespace(s.name, members)
        name_idx = self.fn.add_const(s.name)
        # type_idx = const-Index von None -> VM ruft _infer_type, das fuer
        # _EnumNamespace "enum" liefert. So bleibt _coerce uneingebunden.
        type_idx = self.fn.add_const(None)
        val_idx = self.fn.add_const(ns)
        self.fn.emit(Op.LOAD_CONST, val_idx)
        slot = self._global_slots.get(s.name)
        if slot is not None:
            self.fn.emit(
                Op.DECLARE_GLOBAL_CONST_SLOT, (slot, name_idx, type_idx)
            )
        else:
            self.fn.emit(Op.DECLARE_CONST, (name_idx, type_idx))

    def _stmt_Assign(self, s: Assign):
        self._expr(s.value)
        self._store_var(s.name)

    def _stmt_Print(self, s: Print):
        for item in s.items:
            self._expr(item)
        self.fn.emit(Op.PRINT, len(s.items))

    def _stmt_Input(self, s: Input):
        if s.prompt is not None:
            self._expr(s.prompt)
            has_prompt = True
        else:
            has_prompt = False
        if s.target in self.fn.local_slots:
            slot = self.fn.local_slots[s.target]
            self.fn.emit(Op.INPUT_LOCAL, (slot, has_prompt))
        else:
            name_idx = self.fn.add_const(s.target)
            self.fn.emit(Op.INPUT_NAME, (name_idx, has_prompt))

    def _stmt_ExprStmt(self, s: ExprStmt):
        self._expr(s.expr)
        self.fn.emit(Op.POP)

    def _stmt_If(self, s: If):
        end_jumps: list[int] = []
        # Bedingungs-Block
        self._expr(s.condition)
        false_jump = self.fn.emit(Op.JUMP_IF_FALSE, None)
        for st in s.then_block:
            self._stmt(st)
        end_jumps.append(self.fn.emit(Op.JUMP, None))
        # ELSEIF-Aeste
        for cond, block in s.elseif_branches:
            self.fn.patch_jump(false_jump, len(self.fn.code))
            self._expr(cond)
            false_jump = self.fn.emit(Op.JUMP_IF_FALSE, None)
            for st in block:
                self._stmt(st)
            end_jumps.append(self.fn.emit(Op.JUMP, None))
        # ELSE-Ast
        self.fn.patch_jump(false_jump, len(self.fn.code))
        for st in s.else_block:
            self._stmt(st)
        end = len(self.fn.code)
        for ip in end_jumps:
            self.fn.patch_jump(ip, end)

    def _stmt_Select(self, s: Select):
        """SELECT CASE wird als Stack-basierter Vergleich kompiliert: Subject
        liegt auf dem Stack und wird pro Match per DUP geklont. Kein anonymer
        Slot, kein neuer Bytecode noetig.

        Optionaler Guard: nach dem Match wird die Guard-Expression evaluiert
        und ihr Wert mit JUMP_IF_FALSE gepoppt. Bei Guard-FALSE springt die
        Logik zum naechsten Case (gleiche Stelle wie skip_jump bei No-Match).
        Subject bleibt waehrend Guard-Eval auf dem Stack -- die Guard kann
        also auf Variablen UND auf das Subject zugreifen.
        """
        self._expr(s.subject)            # Stack: [subj]
        end_jumps: list = []
        for case in s.cases:
            if len(case) == 3:
                matches, guard, block = case
            else:
                matches, block = case
                guard = None
            block_jumps: list = []       # JUMP_IF_TRUE-Sprünge zum block_start
            for m in matches:
                self._emit_case_match(m, block_jumps)
            # Kein Match: Stack [subj], JUMP zum naechsten case
            skip_jump = self.fn.emit(Op.JUMP, None)
            block_start = len(self.fn.code)
            for ip in block_jumps:
                self.fn.patch_jump(ip, block_start)
            # Block-Pfad: Stack [subj]. Wenn Guard, eval erst und check.
            guard_skip = None
            if guard is not None:
                self._expr(guard)         # Stack: [subj, guard_val]
                guard_skip = self.fn.emit(Op.JUMP_IF_FALSE, None)
                # JUMP_IF_FALSE poppt guard_val, Stack zurueck zu [subj].
            self.fn.emit(Op.POP)         # Stack: []
            for st in block:
                self._stmt(st)
            end_jumps.append(self.fn.emit(Op.JUMP, None))
            next_case_pos = len(self.fn.code)
            self.fn.patch_jump(skip_jump, next_case_pos)
            if guard_skip is not None:
                self.fn.patch_jump(guard_skip, next_case_pos)
        # Alle cases ohne Match -> Stack [subj], else_block ausfuehren
        self.fn.emit(Op.POP)
        for st in s.else_block:
            self._stmt(st)
        end_pos = len(self.fn.code)
        for ip in end_jumps:
            self.fn.patch_jump(ip, end_pos)

    def _emit_case_match(self, m: CaseMatch, block_jumps: list) -> None:
        """Stack vor und nach: [subj]. Bei Match wird ein JUMP_IF_TRUE-IP
        in block_jumps gesammelt (Sprung erfolgt erst spaeter, ans block_start)."""
        if m.kind == "value":
            self.fn.emit(Op.DUP)
            self._expr(m.values[0])
            self.fn.emit(Op.EQ)
            block_jumps.append(self.fn.emit(Op.JUMP_IF_TRUE, None))
        elif m.kind == "range":
            # subj < lo? -> kein Match, weiter zum naechsten Match.
            self.fn.emit(Op.DUP)
            self._expr(m.values[0])      # lo
            self.fn.emit(Op.GEQ)         # subj >= lo
            skip_to_next = self.fn.emit(Op.JUMP_IF_FALSE, None)
            # subj <= hi? -> Match
            self.fn.emit(Op.DUP)
            self._expr(m.values[1])      # hi
            self.fn.emit(Op.LEQ)         # subj <= hi
            block_jumps.append(self.fn.emit(Op.JUMP_IF_TRUE, None))
            self.fn.patch_jump(skip_to_next, len(self.fn.code))
        elif m.kind == "is":
            op_to_code = {
                "=":  Op.EQ,  "<>": Op.NEQ,
                "<":  Op.LT,  ">":  Op.GT,
                "<=": Op.LEQ, ">=": Op.GEQ,
            }
            op = m.values[0]
            self.fn.emit(Op.DUP)
            self._expr(m.values[1])
            self.fn.emit(op_to_code[op])
            block_jumps.append(self.fn.emit(Op.JUMP_IF_TRUE, None))
        else:
            raise CompileError(
                f"Interner Fehler: unbekannter CASE-Match-Typ '{m.kind}'"
            )

    def _stmt_Repeat(self, s: Repeat):
        """REPEAT body UNTIL cond - laeuft mind. einmal, repeat solange
        cond falsch ist."""
        loop_start = len(self.fn.code)
        self.fn.break_patches.append(([], self.fn.try_depth))
        self.fn.continue_patches.append(([], self.fn.try_depth))
        for st in s.body:
            self._stmt(st)
        # CONTINUE springt zur Bedingungs-Pruefung (= aktuelle Stelle)
        cond_pos = len(self.fn.code)
        cont_patches, _ = self.fn.continue_patches.pop()
        for ip in cont_patches:
            self.fn.patch_jump(ip, cond_pos)
        self._expr(s.condition)
        # JUMP_IF_FALSE -> wieder an Schleifen-Anfang (TRUE = abbrechen)
        self.fn.emit(Op.JUMP_IF_FALSE, loop_start)
        end = len(self.fn.code)
        break_patches, _ = self.fn.break_patches.pop()
        for ip in break_patches:
            self.fn.patch_jump(ip, end)

    def _stmt_Data(self, s: Data):
        # DATA-Werte werden im _collect_data_pass eingesammelt - zur
        # Laufzeit nichts zu tun.
        pass

    def _stmt_Read(self, s: Read):
        """READ target, target, ...: pop next data value, assign to target."""
        for target in s.targets:
            # PUSH_DATA holt den naechsten Wert aufs Stack, dann
            # die normale Assignment-Logik fuer das Ziel.
            self.fn.emit(Op.PUSH_DATA, None)
            self._emit_store_target(target, ctx="READ")

    def _stmt_Restore(self, s: Restore):
        self.fn.emit(Op.RESET_DATA_PTR, None)

    def _emit_store_target(self, target, ctx: str = "Assignment"):
        """Erzeugt Store-Code fuer ein Assignment-Target. Erwartet den
        Wert OBEN AUF DEM STACK; emittiert die passende Store-Sequenz."""
        if isinstance(target, Identifier):
            # Lokale oder globale Variable
            slot = self.fn.local_slots.get(target.name)
            if slot is not None:
                self.fn.emit(Op.STORE_LOCAL, slot)
                return
            name_idx = self.fn.add_const(target.name)
            self.fn.emit(Op.STORE_NAME, name_idx)
            return
        if isinstance(target, IndexAccess):
            # Wert ist auf Stack. Wir brauchen: array, indices..., value -> STORE_INDEX
            # Aktuelle Reihenfolge: value oben. Wir muessen erst array+indices
            # darunter pushen, dann ein "swap"... ohne SWAP-Op machen wir's
            # so: erst array+indices+value korrekt aufbauen, indem wir den
            # value vorher in einen temporaeren Slot speichern.
            # Einfacher: nutze einen Hilfs-temp-slot.
            tmp = self._reserve_temp("__read_tmp")
            self.fn.emit(Op.STORE_LOCAL, tmp)
            self._expr(target.target)
            for ie in target.indices:
                self._expr(ie)
            self.fn.emit(Op.LOAD_LOCAL, tmp)
            self.fn.emit(Op.STORE_INDEX, len(target.indices))
            return
        if isinstance(target, MemberAccess):
            tmp = self._reserve_temp("__read_tmp")
            self.fn.emit(Op.STORE_LOCAL, tmp)
            self._expr(target.target)
            self.fn.emit(Op.LOAD_LOCAL, tmp)
            name_idx = self.fn.add_const(target.name)
            self.fn.emit(Op.STORE_FIELD, name_idx)
            return
        raise CompileError(f"{ctx}: ungueltiges Ziel ({type(target).__name__})")

    def _reserve_temp(self, name: str) -> int:
        """Reserviert einen lokalen Slot fuer interne Hilfs-Variablen
        (z.B. fuer komplexe Store-Targets in READ).  Idempotent pro Name."""
        if name in self.fn.local_slots:
            return self.fn.local_slots[name]
        slot = len(self.fn.local_slots)
        self.fn.local_slots[name] = slot
        self.fn.local_types.append("any")
        self.fn.local_defaults.append(None)
        return slot

    def _stmt_While(self, s: While):
        loop_start = len(self.fn.code)
        self._expr(s.condition)
        exit_jump = self.fn.emit(Op.JUMP_IF_FALSE, None)
        self.fn.break_patches.append(([], self.fn.try_depth))
        self.fn.continue_patches.append(([], self.fn.try_depth))
        for st in s.body:
            self._stmt(st)
        # CONTINUE springt zurueck zur Bedingung
        cont_patches, _ = self.fn.continue_patches.pop()
        for ip in cont_patches:
            self.fn.patch_jump(ip, loop_start)
        self.fn.emit(Op.JUMP, loop_start)
        end = len(self.fn.code)
        self.fn.patch_jump(exit_jump, end)
        break_patches, _ = self.fn.break_patches.pop()
        for ip in break_patches:
            self.fn.patch_jump(ip, end)

    def _stmt_For(self, s: For):
        # 1) Schleifenvariable: ggf. auto-deklarieren als INTEGER
        if not self.fn.is_main and s.var not in self.fn.local_slots:
            self._declare_local(s.var, "integer")
        if self.fn.is_main:
            name_idx = self.fn.add_const(s.var)
            type_idx = self.fn.add_const("integer")
            default_idx = self.fn.add_const(0)
            slot = self._global_slots.get(s.var)
            if slot is not None:
                self.fn.emit(
                    Op.DECLARE_GLOBAL_SLOT,
                    (slot, name_idx, type_idx, default_idx),
                )
            else:
                self.fn.emit(Op.DECLARE_NAME, (name_idx, type_idx, default_idx))

        # Optimierungs-Heuristik: konstanter positiver STEP -> einfacher Vorwaerts-Loop
        const_pos_step = (
            s.step is None
            or (isinstance(s.step, NumberLit)
                and isinstance(s.step.value, (int, float))
                and s.step.value > 0)
        )
        const_neg_step = (
            isinstance(s.step, NumberLit)
            and isinstance(s.step.value, (int, float))
            and s.step.value < 0
        )

        # Initialwert: var = start
        self._expr(s.start)
        self._store_var(s.var)

        # end in Temp-Slot
        end_slot = self._alloc_temp_local("integer")
        self._expr(s.end)
        self.fn.emit(Op.STORE_LOCAL, end_slot)

        # step nur dann in Slot speichern, wenn nicht-konstant
        step_slot = -1
        step_value = 1 if s.step is None else (
            s.step.value if isinstance(s.step, NumberLit) else None
        )
        if step_value is None:
            step_slot = self._alloc_temp_local("integer")
            self._expr(s.step)
            self.fn.emit(Op.STORE_LOCAL, step_slot)

        loop_start = len(self.fn.code)
        self.fn.break_patches.append(([], self.fn.try_depth))
        self.fn.continue_patches.append(([], self.fn.try_depth))
        exit_jumps: list[int] = []

        # Spezialisierte _NN-Ops fuer FOR-Bookkeeping. var und end_slot sind
        # garantiert numerisch (sonst kracht der _coerce schon vorher), also
        # sind die spec-ops semantisch identisch und sparen den Operator-
        # Dispatch + isinstance-Cascade pro Iteration. Spuerbarer Effekt bei
        # tighten Loops.
        if const_pos_step:
            # if var > end -> exit
            self._load_var(s.var)
            self.fn.emit(Op.LOAD_LOCAL, end_slot)
            self.fn.emit(Op.GT_NN)
            exit_jumps.append(self.fn.emit(Op.JUMP_IF_TRUE, None))
        elif const_neg_step:
            # if var < end -> exit
            self._load_var(s.var)
            self.fn.emit(Op.LOAD_LOCAL, end_slot)
            self.fn.emit(Op.LT_NN)
            exit_jumps.append(self.fn.emit(Op.JUMP_IF_TRUE, None))
        else:
            # Allgemeiner Fall: zur Laufzeit Richtung pruefen
            self.fn.emit(Op.LOAD_LOCAL, step_slot)
            self.fn.emit(Op.LOAD_CONST, self.fn.add_const(0))
            self.fn.emit(Op.LT_NN)
            neg_jump = self.fn.emit(Op.JUMP_IF_TRUE, None)
            self._load_var(s.var)
            self.fn.emit(Op.LOAD_LOCAL, end_slot)
            self.fn.emit(Op.GT_NN)
            exit_jumps.append(self.fn.emit(Op.JUMP_IF_TRUE, None))
            body_jump = self.fn.emit(Op.JUMP, None)
            self.fn.patch_jump(neg_jump, len(self.fn.code))
            self._load_var(s.var)
            self.fn.emit(Op.LOAD_LOCAL, end_slot)
            self.fn.emit(Op.LT_NN)
            exit_jumps.append(self.fn.emit(Op.JUMP_IF_TRUE, None))
            self.fn.patch_jump(body_jump, len(self.fn.code))

        # body
        for st in s.body:
            self._stmt(st)

        # CONTINUE landet beim Inkrement
        increment_target = len(self.fn.code)
        cont_patches, _ = self.fn.continue_patches.pop()
        for ip in cont_patches:
            self.fn.patch_jump(ip, increment_target)

        # var += step  (konstanter step -> direkter LOAD_CONST + ADD_NN)
        self._load_var(s.var)
        if step_value is not None:
            self.fn.emit(Op.LOAD_CONST, self.fn.add_const(step_value))
        else:
            self.fn.emit(Op.LOAD_LOCAL, step_slot)
        self.fn.emit(Op.ADD_NN)
        self._store_var(s.var)
        self.fn.emit(Op.JUMP, loop_start)

        end = len(self.fn.code)
        for ip in exit_jumps:
            self.fn.patch_jump(ip, end)
        break_patches, _ = self.fn.break_patches.pop()
        for ip in break_patches:
            self.fn.patch_jump(ip, end)

    def _stmt_ForEach(self, s):
        # Desugar zu einem Vorwaerts-Index-Loop ueber __comp_iter(iterable):
        #   __it = __comp_iter(iterable)   (TUPLE der Elemente)
        #   __i = 0; __n = LEN(__it)
        #   WHILE __i < __n: var = __it[__i]; <body>; __i += 1
        # Nutzt LOAD_INDEX (kennt Tupel) + den break/continue-Patch-Stack
        # wie _stmt_For -> BREAK/CONTINUE funktionieren unveraendert.
        # Loop-Var als "any" deklarieren (nimmt beliebige Element-Typen).
        if self.fn.is_main:
            if s.var not in self.fn.local_slots:
                name_idx = self.fn.add_const(s.var)
                type_idx = self.fn.add_const("any")
                default_idx = self.fn.add_const(None)
                gslot = self._global_slots.get(s.var)
                if gslot is not None:
                    self.fn.emit(Op.DECLARE_GLOBAL_SLOT,
                                 (gslot, name_idx, type_idx, default_idx))
                else:
                    self.fn.emit(Op.DECLARE_NAME, (name_idx, type_idx, default_idx))
        else:
            if s.var not in self.fn.local_slots:
                self._declare_local(s.var, "any")

        it_slot = self._alloc_temp_local("tuple")
        self._expr(s.iterable)
        self.fn.emit(Op.CALL_BUILTIN, ("__comp_iter", 1))
        self.fn.emit(Op.STORE_LOCAL, it_slot)
        idx_slot = self._alloc_temp_local("integer")
        self.fn.emit(Op.LOAD_CONST, self.fn.add_const(0))
        self.fn.emit(Op.STORE_LOCAL, idx_slot)
        len_slot = self._alloc_temp_local("integer")
        self.fn.emit(Op.LOAD_LOCAL, it_slot)
        self.fn.emit(Op.CALL_BUILTIN, ("len", 1))
        self.fn.emit(Op.STORE_LOCAL, len_slot)

        loop_start = len(self.fn.code)
        self.fn.break_patches.append(([], self.fn.try_depth))
        self.fn.continue_patches.append(([], self.fn.try_depth))
        # if __i >= __n -> exit
        self.fn.emit(Op.LOAD_LOCAL, idx_slot)
        self.fn.emit(Op.LOAD_LOCAL, len_slot)
        self.fn.emit(Op.GEQ_NN)
        exit_jump = self.fn.emit(Op.JUMP_IF_TRUE, None)
        # var = __it[__i]
        self.fn.emit(Op.LOAD_LOCAL, it_slot)
        self.fn.emit(Op.LOAD_LOCAL, idx_slot)
        self.fn.emit(Op.LOAD_INDEX, 1)
        self._store_var(s.var)
        # body
        for st in s.body:
            self._stmt(st)
        # CONTINUE -> Inkrement
        inc_target = len(self.fn.code)
        cont_patches, _ = self.fn.continue_patches.pop()
        for ip in cont_patches:
            self.fn.patch_jump(ip, inc_target)
        # __i += 1
        self.fn.emit(Op.LOAD_LOCAL, idx_slot)
        self.fn.emit(Op.LOAD_CONST, self.fn.add_const(1))
        self.fn.emit(Op.ADD_NN)
        self.fn.emit(Op.STORE_LOCAL, idx_slot)
        self.fn.emit(Op.JUMP, loop_start)
        end = len(self.fn.code)
        self.fn.patch_jump(exit_jump, end)
        break_patches, _ = self.fn.break_patches.pop()
        for ip in break_patches:
            self.fn.patch_jump(ip, end)

    def _stmt_Break(self, s: Break):
        if not self.fn.break_patches:
            raise CompileError("BREAK ausserhalb Schleife")
        patches, loop_depth = self.fn.break_patches[-1]
        # TRY_END fuer alle Try-Bloecke zwischen Loop-Start und hier emittieren
        for _ in range(self.fn.try_depth - loop_depth):
            self.fn.emit(Op.TRY_END)
        ip = self.fn.emit(Op.JUMP, None)
        patches.append(ip)

    def _stmt_Continue(self, s: Continue):
        if not self.fn.continue_patches:
            raise CompileError("CONTINUE ausserhalb Schleife")
        patches, loop_depth = self.fn.continue_patches[-1]
        for _ in range(self.fn.try_depth - loop_depth):
            self.fn.emit(Op.TRY_END)
        ip = self.fn.emit(Op.JUMP, None)
        patches.append(ip)

    def _stmt_Try(self, s: Try):
        catch_jmp = self.fn.emit(Op.TRY_BEGIN, None)
        self.fn.try_depth += 1
        for st in s.body:
            self._stmt(st)
        self.fn.try_depth -= 1
        self.fn.emit(Op.TRY_END)
        end_jmp = self.fn.emit(Op.JUMP, None)
        # Catch-Branch: Stack hat den (string) Exception-Wert oben
        self.fn.patch_jump(catch_jmp, len(self.fn.code))
        if s.catch_var:
            # Auto-deklariere als STRING wenn noch nicht vorhanden
            if self.fn.is_main:
                # Top-Level: globale STRING-Variable
                if s.catch_var not in self.fn.local_slots:
                    name_idx = self.fn.add_const(s.catch_var)
                    type_idx = self.fn.add_const("string")
                    default_idx = self.fn.add_const("")
                    self.fn.emit(Op.DECLARE_NAME, (name_idx, type_idx, default_idx))
            else:
                # Funktion: lokale STRING-Variable
                if s.catch_var not in self.fn.local_slots:
                    self._declare_local(s.catch_var, "string")
            self._store_var(s.catch_var)
        else:
            self.fn.emit(Op.POP)
        for st in s.catch_block:
            self._stmt(st)
        self.fn.patch_jump(end_jmp, len(self.fn.code))

    def _stmt_Throw(self, s: Throw):
        self._expr(s.value)
        self.fn.emit(Op.THROW)

    def _stmt_MemberAssign(self, s: MemberAssign):
        # obj.field = value  ->  PUSH obj, PUSH value, STORE_MEMBER name
        self._expr(s.target)
        self._expr(s.value)
        self.fn.emit(Op.STORE_MEMBER, self.fn.add_const(s.name))

    def _stmt_IndexAssign(self, s: IndexAssign):
        # arr[i,j] = value -> PUSH arr, PUSH i, PUSH j, PUSH value, STORE_INDEX n
        self._expr(s.target)
        for ix in s.indices:
            self._expr(ix)
        self._expr(s.value)
        self.fn.emit(Op.STORE_INDEX, len(s.indices))

    def _stmt_With(self, s: With):
        """`WITH expr / body / END WITH`. Strategie:
            1. Reserviere einen anonymen Local-Slot fuer das WITH-Ziel.
            2. Eval expr und STORE_LOCAL.
            3. Registriere den Compiler-generierten Var-Namen im local_slots-
               Dict, damit `LOAD_NAME __with_<n>` (vom Parser-Desugar) zu
               LOAD_LOCAL slot wird.
            4. Body kompilieren.
            5. De-registrieren (Slot bleibt belegt, Name ist frei -- damit
               nachfolgender Code ihn nicht versehentlich findet).
        """
        slot = self._alloc_anon_slot("any")
        self._expr(s.target)
        self.fn.emit(Op.STORE_LOCAL, slot)
        # Name -> Slot binden, damit Identifier-Lookup im Body trifft.
        self.fn.local_slots[s.var_name] = slot
        try:
            for st in s.body:
                self._stmt(st)
        finally:
            del self.fn.local_slots[s.var_name]

    def _stmt_TupleAssign(self, s):
        """`(t1, t2, ..., tn) = expr`. Strategie:
            PUSH expr            -> [tuple]
            UNPACK_TUPLE n       -> [val_0, val_1, ..., val_{n-1}]
                                    (val_0 ist top -- siehe vm.py-Doku)
            store t_0
            store t_1
            ...
        Targets sind Identifier, MemberAccess oder IndexAccess. Fuer Member/
        Index muss der Receiver+Indizes BEVOR der Value emittiert werden, was
        UNPACK_TUPLE-Output unter dem Receiver vergraben wuerde -- daher
        zwischenspeichern wir den Wert in einen anonymen Local-Slot.
        """
        from .ast_nodes import Identifier as _Id, MemberAccess as _MA, IndexAccess as _IA
        self._expr(s.value)
        self.fn.emit(Op.UNPACK_TUPLE, len(s.targets))
        # Stack hat jetzt: [..., t_0, t_1, ..., t_{n-1}] mit t_0 als top.
        for tgt in s.targets:
            if isinstance(tgt, _Id):
                self._store_var(tgt.name)
            elif isinstance(tgt, _MA):
                # Wert ist top of stack. Wir muessen Receiver darunter pushen
                # und dann STORE_MEMBER. Trick: in temporaeren Slot zwischen-
                # speichern.
                tmp_slot = self._alloc_anon_slot("any")
                self.fn.emit(Op.STORE_LOCAL, tmp_slot)
                self._expr(tgt.target)
                self.fn.emit(Op.LOAD_LOCAL, tmp_slot)
                self.fn.emit(Op.STORE_MEMBER, self.fn.add_const(tgt.name))
            elif isinstance(tgt, _IA):
                tmp_slot = self._alloc_anon_slot("any")
                self.fn.emit(Op.STORE_LOCAL, tmp_slot)
                self._expr(tgt.target)
                for ix in tgt.indices:
                    self._expr(ix)
                self.fn.emit(Op.LOAD_LOCAL, tmp_slot)
                self.fn.emit(Op.STORE_INDEX, len(tgt.indices))
            else:
                raise CompileError(
                    f"Ungueltiges Tupel-Assignment-Ziel: {type(tgt).__name__}"
                )

    def _alloc_anon_slot(self, type_name: str) -> int:
        """Reserviert einen anonymen Local-Slot fuer Compiler-Zwischenwerte
        (z.B. Tupel-Destructuring mit Member/Index-Targets). Der Slot ist
        nicht durch einen Identifier ansprechbar -- nur per LOAD_LOCAL/
        STORE_LOCAL mit dem zurueckgegebenen Index.

        Im Top-Level/Main-Frame gibt's keine echten Locals -- dann nutzen wir
        einen Pseudo-Namen mit einem fuer User unzugaenglichen Praefix.
        """
        idx = len(self.fn.local_types)
        self.fn.local_types.append(type_name)
        self.fn.local_defaults.append(None)
        self.fn.local_slots[f"__anon_{idx}"] = idx
        return idx

    def _stmt_Return(self, s: Return):
        if self.fn.is_main:
            raise CompileError("RETURN nur in SUB/FUNCTION")
        if s.value is not None:
            if self.fn.is_sub:
                raise CompileError(
                    f"SUB '{self.fn.name}' darf RETURN nicht mit Wert verwenden"
                )
            self._expr(s.value)
            self.fn.emit(Op.RETURN)
        else:
            if not self.fn.is_sub:
                raise CompileError(
                    f"FUNCTION '{self.fn.name}' braucht einen Wert bei RETURN"
                )
            self.fn.emit(Op.RETURN_VOID)

    # -------- Expressions ----------------------------------------------
    def _expr(self, e):
        method = getattr(self, f"_expr_{type(e).__name__}", None)
        if method is None:
            raise CompileError(
                f"VM unterstuetzt Ausdruck {type(e).__name__} noch nicht (Phase 3b/3c)"
            )
        method(e)

    def _expr_NumberLit(self, e: NumberLit):
        self.fn.emit(Op.LOAD_CONST, self.fn.add_const(e.value))

    def _expr_StringLit(self, e: StringLit):
        self.fn.emit(Op.LOAD_CONST, self.fn.add_const(e.value))

    def _expr_BoolLit(self, e: BoolLit):
        self.fn.emit(Op.LOAD_CONST, self.fn.add_const(e.value))

    def _expr_TupleLit(self, e):
        for el in e.elements:
            self._expr(el)
        self.fn.emit(Op.BUILD_TUPLE, len(e.elements))

    def _expr_Identifier(self, e: Identifier):
        # Bare User-Function in Expression-Position liefert eine FUNCREF --
        # ABER eine User-Variable mit gleichem Namen verschattet die Function.
        # Vorrang-Reihenfolge (gleich wie Tree-Walker `env.has` zuerst):
        #   Locals (in Funktionen) -> Felder (in Methoden) ->
        #   Global-Vars (Top-Level DIM/CONST) -> Function -> Global-Lookup.
        if (e.name not in self.fn.local_slots
                and not (self.fn.current_class is not None and self._is_field(e.name))
                and e.name not in self._global_vars
                and e.name in self.functions):
            self.fn.emit(Op.LOAD_FUNCREF, self.fn.add_const(e.name))
            return
        self._load_var(e.name)

    # --- Constant Folding (Compile-Zeit-Auswertung) -------------------------
    # Reduziert konstante Sub-Expressions zu einem einzigen LOAD_CONST.
    # Hilft besonders bei FOR-Bounds (`0 TO N - 1`), Math-Konstanten
    # (`2 * PI`) und Initialisierungen (`width / 2`). Wird vor jedem
    # BinaryOp/UnaryOp probiert; bei Erfolg wird der ganze Sub-Tree zu
    # einem Literal kollabiert.
    #
    # Rueckgabe: (True, value) wenn gefoldet; (False, None) sonst.
    # Wichtig: nicht folden, wenn das Resultat zur Laufzeit fehlschlagen
    # wuerde (Division durch 0, Overflow, Bool-in-Arithmetik) -- der
    # Runtime-Fehler ist die bessere User-Experience.
    def _try_fold(self, e):
        if isinstance(e, NumberLit):
            return (True, e.value)
        if isinstance(e, StringLit):
            return (True, e.value)
        if isinstance(e, BoolLit):
            return (True, e.value)
        if isinstance(e, UnaryOp):
            ok, v = self._try_fold(e.operand)
            if not ok:
                return (False, None)
            if e.op == "-":
                if isinstance(v, bool) or not isinstance(v, (int, float)):
                    return (False, None)
                return (True, -v)
            if e.op == "not":
                return (True, not bool(v))
            if e.op == "bnot":
                if isinstance(v, bool) or not isinstance(v, int):
                    return (False, None)
                return (True, ~v)
            return (False, None)
        if isinstance(e, BinaryOp):
            op = e.op
            if op in ("and", "or"):
                # Short-circuit: nicht folden (Side-Effects-Semantik)
                return (False, None)
            ok_a, a = self._try_fold(e.left)
            if not ok_a:
                return (False, None)
            ok_b, b = self._try_fold(e.right)
            if not ok_b:
                return (False, None)
            # Bool-in-Arithmetik: GB lehnt das ab. Nicht folden, damit
            # der Runtime-Type-Error klar bleibt. Equality erlauben.
            a_is_bool = isinstance(a, bool)
            b_is_bool = isinstance(b, bool)
            if a_is_bool or b_is_bool:
                if op == "=":
                    return (True, a == b)
                if op == "<>":
                    return (True, a != b)
                return (False, None)
            try:
                if op == "+":
                    if isinstance(a, str) and isinstance(b, str):
                        return (True, a + b)
                    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
                        return (True, a + b)
                    return (False, None)
                if op == "-":
                    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
                        return (True, a - b)
                    return (False, None)
                if op == "*":
                    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
                        return (True, a * b)
                    if isinstance(a, str) and isinstance(b, int):
                        return (True, a * b if b > 0 else "")
                    if isinstance(b, str) and isinstance(a, int):
                        return (True, b * a if a > 0 else "")
                    return (False, None)
                if op == "/":
                    if not (isinstance(a, (int, float)) and isinstance(b, (int, float))):
                        return (False, None)
                    if b == 0:
                        return (False, None)  # Runtime-Error besser
                    if isinstance(a, int) and isinstance(b, int) and a % b == 0:
                        return (True, a // b)
                    return (True, a / b)
                if op == "mod":
                    if not (isinstance(a, (int, float)) and isinstance(b, (int, float))):
                        return (False, None)
                    if b == 0:
                        return (False, None)
                    return (True, a % b)
                if op == "\\":
                    if not (isinstance(a, int) and isinstance(b, int)):
                        return (False, None)
                    if b == 0:
                        return (False, None)
                    q, r = divmod(a, b)
                    if r != 0 and (a < 0) != (b < 0):
                        q += 1
                    return (True, q)
                if op == "^":
                    if not (isinstance(a, (int, float)) and isinstance(b, (int, float))):
                        return (False, None)
                    # Safety-Cap: extrem grosse Integer-Pow zur Compile-
                    # Zeit nicht vorberechnen (kann den const-pool fluten).
                    if isinstance(b, int) and b > 64 and isinstance(a, int) and abs(a) > 2:
                        return (False, None)
                    return (True, a ** b)
                if op in ("=", "<>", "<", ">", "<=", ">="):
                    # Vergleich nur fuer gleiche oder numerisch-mischbare Typen
                    if isinstance(a, str) != isinstance(b, str):
                        return (False, None)
                    if op == "=":  return (True, a == b)
                    if op == "<>": return (True, a != b)
                    if op == "<":  return (True, a < b)
                    if op == ">":  return (True, a > b)
                    if op == "<=": return (True, a <= b)
                    if op == ">=": return (True, a >= b)
                if op in ("band", "bor", "bxor", "shl", "shr"):
                    if not (isinstance(a, int) and isinstance(b, int)):
                        return (False, None)
                    if op == "band": return (True, a & b)
                    if op == "bor":  return (True, a | b)
                    if op == "bxor": return (True, a ^ b)
                    if op == "shl":
                        if b < 0 or b > 64:
                            return (False, None)
                        return (True, a << b)
                    if op == "shr":
                        if b < 0:
                            return (False, None)
                        return (True, a >> b)
            except Exception:
                return (False, None)
            return (False, None)
        return (False, None)

    # --- Statische Type-Inference (best-effort, konservativ) ----------------
    # Liefert einen der: "integer", "float", "num" (numerisch, int oder float
    # unklar), "string", "boolean", "" (unbekannt). Wird ausschliesslich vom
    # Compiler genutzt, um zu entscheiden, ob eine spezialisierte _NN-Op
    # statt der generischen Op emittiert werden kann. Bei "" faellt der
    # Compiler immer auf die generische Op zurueck -- also nie unsafe.
    def _expr_type(self, e) -> str:
        if isinstance(e, NumberLit):
            v = e.value
            if isinstance(v, bool):  # paranoia: bool ist KEINE Zahl
                return "boolean"
            if isinstance(v, int):
                return "integer"
            return "float"
        if isinstance(e, StringLit):
            return "string"
        if isinstance(e, BoolLit):
            return "boolean"
        if isinstance(e, Identifier):
            if self.fn is not None:
                slot = self.fn.local_slots.get(e.name)
                if slot is not None:
                    t = (self.fn.local_types[slot] or "").lower()
                    if t in ("integer", "float", "string", "boolean"):
                        return t
            # Globals: Type aus _global_types holen. Funktioniert sowohl im
            # Top-Level-Hauptprogramm als auch in SUB/FUNCTION/Methoden, die
            # auf globale Variablen zugreifen.
            t = self._global_types.get(e.name, "")
            if t in ("integer", "float", "string", "boolean"):
                return t
            return ""
        if isinstance(e, UnaryOp):
            if e.op in ("-", "+"):
                t = self._expr_type(e.operand)
                if t in ("integer", "float", "num"):
                    return t
                return ""
            if e.op == "not":
                return "boolean"
            if e.op == "bnot":
                return "integer" if self._expr_type(e.operand) == "integer" else ""
            return ""
        if isinstance(e, BinaryOp):
            op = e.op
            if op in ("and", "or", "=", "<>", "<", ">", "<=", ">=", "in"):
                return "boolean"
            if op in ("+", "-", "*"):
                lt = self._expr_type(e.left)
                rt = self._expr_type(e.right)
                if lt == "integer" and rt == "integer":
                    return "integer"
                if (lt in ("integer", "float", "num")
                        and rt in ("integer", "float", "num")):
                    return "num"
                return ""
            if op == "/":
                lt = self._expr_type(e.left)
                rt = self._expr_type(e.right)
                # Int/Int kann int oder float liefern (siehe OP_DIV-Semantik) -> "num".
                if (lt in ("integer", "float", "num")
                        and rt in ("integer", "float", "num")):
                    return "num"
                return ""
            if op == "mod":
                lt = self._expr_type(e.left)
                rt = self._expr_type(e.right)
                if lt == "integer" and rt == "integer":
                    return "integer"
                if (lt in ("integer", "float", "num")
                        and rt in ("integer", "float", "num")):
                    return "num"
                return ""
            if op in ("\\", "band", "bor", "bxor", "shl", "shr"):
                return "integer"
            return ""
        if isinstance(e, Call):
            if isinstance(e.callee, Identifier):
                fn = self.functions.get(e.callee.name)
                if fn is not None:
                    rt = (fn.return_type or "").lower()
                    if rt in ("integer", "float", "string", "boolean"):
                        return rt
            return ""
        # Index-Zugriff auf ein typisiertes Array -> Element-Typ. Sicher, weil
        # typisierte Arrays homogen sind (Element ist garantiert int/float/str).
        # Damit greift der _NN-Hot-Path bei `buf[i] + ...` (bench_array_rw).
        if isinstance(e, IndexAccess):
            base = self._raw_decl_type(e.target)
            if base.startswith("array:"):
                elem = base[6:]
                if elem in ("integer", "float", "string"):
                    return elem
                return ""
            if base == "string":
                return "string"   # String-Index liefert 1-Zeichen-String
            return ""
        # Feld-Zugriff `Self.feld` mit statisch bekanntem Skalartyp -> dieser
        # Typ. Damit greift der _NN-Hot-Path bei `Self.total + n`
        # (bench_method_dispatch). Konservativ: nur echte Felder, nur Self.
        if isinstance(e, MemberAccess):
            ft = self._field_type(e)
            if ft in ("integer", "float", "string", "boolean"):
                return ft
            return ""
        return ""

    def _expr_TernaryExpr(self, e):
        # Lazy Ternary (IIF): cond auswerten, via JUMP_IF_FALSE genau einen
        # Zweig auf den Stack. Kein neuer Bytecode -- nutzt vorhandene Spruenge
        # (JUMP_IF_FALSE poppt die Bedingung). Beide VMs ohne Aenderung.
        self._expr(e.cond)
        jf = self.fn.emit(Op.JUMP_IF_FALSE, None)
        self._expr(e.then_expr)
        jend = self.fn.emit(Op.JUMP, None)
        self.fn.patch_jump(jf, len(self.fn.code))     # else-Zweig
        self._expr(e.else_expr)
        self.fn.patch_jump(jend, len(self.fn.code))   # Ende

    def _expr_BinaryOp(self, e: BinaryOp):
        # Constant Folding: ganze konstante Sub-Expr zu einem LOAD_CONST
        # reduzieren. Faengt z.B. `100 - 1`, `2 * PI` (wenn PI vorab via
        # NumberLit-CONST gefoldet waere -- aktuell ist PI ein Global,
        # also greift Fold nicht; folgender Compiler-Pass koennte CONST-
        # Werte einbringen).
        ok, value = self._try_fold(e)
        if ok:
            self.fn.emit(Op.LOAD_CONST, self.fn.add_const(value))
            return
        op = e.op
        if op == "and":
            # short-circuit
            self._expr(e.left)
            self.fn.emit(Op.DUP)
            jmp = self.fn.emit(Op.JUMP_IF_FALSE, None)
            self.fn.emit(Op.POP)             # bisheriger TRUE-Wert weg
            self._expr(e.right)
            self.fn.patch_jump(jmp, len(self.fn.code))
            return
        if op == "or":
            self._expr(e.left)
            self.fn.emit(Op.DUP)
            jmp = self.fn.emit(Op.JUMP_IF_TRUE, None)
            self.fn.emit(Op.POP)
            self._expr(e.right)
            self.fn.patch_jump(jmp, len(self.fn.code))
            return
        # Spezialisierter Pfad: wenn beide Operanden statisch numerisch sind,
        # emittieren wir die _NN-Variante, die isinstance-Checks und
        # Operator-Dispatch (Modul-Registry + User-Class-Operator) ueberspringt.
        # Bool und String werden in der Inferenz nie als "numerisch" markiert.
        spec = _NUMERIC_SPEC_OPS.get(op)
        if spec is not None:
            lt = self._expr_type(e.left)
            rt = self._expr_type(e.right)
            if (lt in ("integer", "float", "num")
                    and rt in ("integer", "float", "num")):
                self._expr(e.left)
                self._expr(e.right)
                self.fn.emit(spec)
                return
        # Generischer Pfad
        self._expr(e.left)
        self._expr(e.right)
        op_to_code = {
            "+":  Op.ADD, "-": Op.SUB, "*": Op.MUL, "/": Op.DIV,
            "mod": Op.MOD, "^": Op.POW, "\\": Op.INT_DIV,
            "=":  Op.EQ, "<>": Op.NEQ,
            "<":  Op.LT, ">":  Op.GT, "<=": Op.LEQ, ">=": Op.GEQ,
            "band": Op.BAND, "bor": Op.BOR, "bxor": Op.BXOR,
            "shl":  Op.SHL,  "shr": Op.SHR,
            "in":   Op.IN_OP,
        }
        self.fn.emit(op_to_code[op])

    def _expr_UnaryOp(self, e: UnaryOp):
        ok, value = self._try_fold(e)
        if ok:
            self.fn.emit(Op.LOAD_CONST, self.fn.add_const(value))
            return
        if e.op == "-":
            is_num = self._expr_type(e.operand) in ("integer", "float", "num")
            self._expr(e.operand)
            self.fn.emit(Op.NEG_N if is_num else Op.NEG)
            return
        self._expr(e.operand)
        if e.op == "not":
            self.fn.emit(Op.NOT)
        elif e.op == "bnot":
            self.fn.emit(Op.BNOT)
        else:
            raise CompileError(f"Unbekannter unaerer Operator: {e.op}")

    def _expr_Call(self, e: Call):
        # Methoden-Aufruf: obj.method(args)
        if isinstance(e.callee, MemberAccess):
            # VM-Pfad: Methoden-Calls mit Named-Args werden nicht
            # unterstuetzt, weil die Klasse (und damit die Param-Namen) erst
            # zur Laufzeit feststeht. Der Tree-Walker kann beides; im
            # VM-Pfad muss man positional rufen.
            self._reject_named_args(e.args, e.callee.name + " (Methode)")
            self._expr(e.callee.target)   # push obj
            for a in e.args:
                self._expr(a)
            self.fn.emit(Op.CALL_METHOD, (e.callee.name, len(e.args)))
            return
        if not isinstance(e.callee, Identifier):
            raise CompileError("Aufrufbare Werte werden noch nicht unterstuetzt")
        name = e.callee.name
        # Implizite Methoden-Aufrufe: wenn wir gerade eine Methode kompilieren
        # und der Identifier eine Methode der eigenen Klasse (oder einer
        # Superklasse) ist, behandeln wir den Aufruf wie `Self.name(...)`.
        if (self.fn.current_class is not None
                and self._resolve_method_compile(self.fn.current_class, name) is not None):
            self._reject_named_args(e.args, name + " (Methode)")
            self.fn.emit(Op.LOAD_SELF)
            for a in e.args:
                self._expr(a)
            self.fn.emit(Op.CALL_METHOD, (name, len(e.args)))
            return
        # User-Variable mit gleichem Namen wie eine Function verschattet
        # diese -- dann ist `name(...)` ein FUNCREF-Call, kein User-Function-
        # Direktaufruf. Locals und Top-Level-Globals werden hier geprueft.
        is_local_var = name in self.fn.local_slots
        is_global_var = name in self._global_vars
        if (name in self.functions
                and not is_local_var
                and not is_global_var):
            fn = self.functions[name]
            if fn.is_variadic:
                # Variadic: keine Named-Args, keine Default-Resolution.
                # Wir emittieren ALLE Args als positional, der CALL_USER-
                # Handler in der VM sammelt die ueberzaehligen ins Tupel.
                self._reject_named_args(e.args, name.upper())
                for a in e.args:
                    self._expr(a)
                self.fn.emit(Op.CALL_USER, (name, len(e.args)))
                return
            resolved = self._resolve_named_args(fn, e.args, name.upper())
            for action in resolved:
                self._emit_resolved_arg(action)
            self.fn.emit(Op.CALL_USER, (name, len(resolved)))
            return
        # Identifier ist weder Function noch Methode -- pruefen, ob's eine
        # FUNCREF-Variable im Scope ist. Erkennungsheuristik: Local-Slot
        # mit type "funcref", oder ein bekannter Builtin-Name (dann
        # CALL_BUILTIN). Da der Compiler den Type globaler Variablen nicht
        # ueberall zuverlaessig kennt, dispatchen wir bei nicht-Builtins
        # und nicht-Functions per CALL_VALUE -- die VM prueft dann zur
        # Laufzeit, ob der Wert ein FuncRef ist.
        if name in BUILTINS or name in GRAPHICS_BUILTINS:
            self._reject_named_args(e.args, name.upper())
            for a in e.args:
                self._expr(a)
            self.fn.emit(Op.CALL_BUILTIN, (name, len(e.args)))
            return
        # FUNCREF-Variable: callee laden, args, dann CALL_VALUE.
        self._reject_named_args(e.args, name.upper())
        self._load_var(name)
        for a in e.args:
            self._expr(a)
        self.fn.emit(Op.CALL_VALUE, len(e.args))

    def _expr_New(self, e: New):
        cls = self.classes.get(e.class_name)
        if cls is None:
            raise CompileError(f"Klasse '{e.class_name}' nicht gefunden")
        if e.args is None:
            self.fn.emit(Op.NEW_INSTANCE, (e.class_name, 0, False))
            return
        # Named-Args bei NEW: wir wuerden die Init-Methode brauchen, um auf
        # Param-Namen zuzugreifen.  Init-Methoden werden aber erst _nach_
        # _expr_New (in Phase 4) kompiliert. Wir akzeptieren NEW Class(name: ..)
        # nur, wenn alle Named-Args zu positional umgesortiert werden koennen,
        # _und_ wir die Klasse zu diesem Zeitpunkt schon kennen. Conservative
        # check: wenn keine NamedArgs, alter Pfad. Sonst: Init-Decl
        # nachschlagen ueber den Klassen-AST (existiert in self.classes
        # nicht direkt, aber wir kennen die Methods aus Phase 1).
        has_named = any(isinstance(a, NamedArg) for a in e.args)
        if not has_named:
            for a in e.args:
                self._expr(a)
            self.fn.emit(Op.NEW_INSTANCE, (e.class_name, len(e.args), True))
            return
        # Init-Methode der Klasse aufloesen (entlang der Vererbung)
        init_fn = self._resolve_method_compile(cls, "init")
        if init_fn is None:
            raise CompileError(
                f"NEW {e.class_name} mit Named-Args, aber Klasse hat keine Init-Methode"
            )
        resolved = self._resolve_named_args(init_fn, e.args, f"NEW {e.class_name}")
        for action in resolved:
            self._emit_resolved_arg(action)
        self.fn.emit(Op.NEW_INSTANCE, (e.class_name, len(resolved), True))

    def _resolve_method_compile(self, cls, method_name: str):
        """Sucht eine Methode entlang der Vererbungshierarchie (Compile-Zeit).
        Returns CompiledFunction oder None."""
        cur = cls
        while cur is not None:
            m = cur.methods.get(method_name)
            if m is not None:
                return m
            cur = cur.parent
        return None

    def _reject_named_args(self, raw_args, ctx: str):
        for a in raw_args:
            if isinstance(a, NamedArg):
                raise CompileError(
                    f"{ctx}: Named-Args werden im VM-Pfad nur fuer "
                    f"SUB/FUNCTION-Aufrufe und NEW Class(...) unterstuetzt"
                )

    def _resolve_named_args(self, fn, raw_args, fn_name: str):
        """Mappe (positional + named) auf Param-Reihenfolge des Decls.

        Liefert eine Liste von "Aktionen" der Laenge n_total:
        - ("expr", expr): den Ausdruck zur Laufzeit auswerten und pushen.
        - ("default", value): den evaluierten Default-Literal-Wert pushen.

        Die VM braucht von der ganzen Mechanik nichts zu wissen - der
        Compiler emittiert eine vollstaendige positional-Liste.
        """
        n_total = fn.n_params
        param_names = fn.param_names or []
        defaults = fn.param_defaults or []
        slots: list = [None] * n_total

        # Positional bis zum ersten NamedArg
        pos_count = 0
        for a in raw_args:
            if isinstance(a, NamedArg):
                break
            pos_count += 1
        if pos_count > n_total:
            raise CompileError(
                f"{fn_name}: zu viele Argumente (Funktion erwartet maximal {n_total})"
            )
        for j in range(pos_count, len(raw_args)):
            if not isinstance(raw_args[j], NamedArg):
                raise CompileError(
                    f"{fn_name}: positional Argument nach Named-Arg ist nicht erlaubt"
                )

        for i in range(pos_count):
            slots[i] = ("expr", raw_args[i])

        # Named einordnen
        param_index = {name: i for i, name in enumerate(param_names)}
        for j in range(pos_count, len(raw_args)):
            na: NamedArg = raw_args[j]
            key = na.name.lower()
            idx = param_index.get(key)
            if idx is None:
                raise CompileError(
                    f"{fn_name}: kein Parameter mit Namen '{na.name}'"
                )
            if slots[idx] is not None:
                raise CompileError(
                    f"{fn_name}: Parameter '{na.name}' doppelt belegt "
                    f"(positional und named)"
                )
            slots[idx] = ("expr", na.value)

        # Defaults / Pflicht-Check
        actions: list = []
        for i in range(n_total):
            if slots[i] is not None:
                actions.append(slots[i])
            else:
                if i >= len(defaults) or defaults[i] is None:
                    pname = param_names[i] if i < len(param_names) else f"#{i+1}"
                    raise CompileError(
                        f"{fn_name}: Parameter '{pname}' fehlt "
                        f"(weder positional noch named angegeben)"
                    )
                actions.append(("default", defaults[i]))
        return actions

    def _emit_resolved_arg(self, action):
        kind, payload = action
        if kind == "expr":
            self._expr(payload)
        elif kind == "default":
            self.fn.emit(Op.LOAD_CONST, self.fn.add_const(payload))
        else:
            raise CompileError(f"Internal: unbekannte resolved-arg-Aktion '{kind}'")

    def _expr_MemberAccess(self, e: MemberAccess):
        self._expr(e.target)
        self.fn.emit(Op.LOAD_MEMBER, self.fn.add_const(e.name))

    def _expr_IndexAccess(self, e: IndexAccess):
        self._expr(e.target)
        for ix in e.indices:
            self._expr(ix)
        self.fn.emit(Op.LOAD_INDEX, len(e.indices))

    def _expr_ListComp(self, e: ListComp):
        """List-Comprehension via Marker + dynamischem Tupel-Build.

        Strategie:
            push COMP_MARKER
            iter, len = ... (in anonymen Slots)
            counter = 0
            loop:
                if counter >= len: jump end
                var = iter[counter]
                if filter and not filter: jump skip
                push transform-value
                skip:
                counter += 1
                jump loop
            end:
            BUILD_TUPLE_DYN
        """
        from .bytecode import COMP_MARKER
        # Marker pushen
        self.fn.emit(Op.LOAD_CONST, self.fn.add_const(COMP_MARKER))
        # iterable -> Tuple via __COMP_ITER, dann in Slot. Damit funktioniert
        # der Loop einheitlich fuer String/Tuple/Array/Map (Keys).
        iter_slot = self._alloc_anon_slot("any")
        self._expr(e.iterable)
        self.fn.emit(Op.CALL_BUILTIN, ("__comp_iter", 1))
        self.fn.emit(Op.STORE_LOCAL, iter_slot)
        # Laenge in Slot via LEN
        self.fn.emit(Op.LOAD_LOCAL, iter_slot)
        self.fn.emit(Op.CALL_BUILTIN, ("len", 1))
        len_slot = self._alloc_anon_slot("integer")
        self.fn.emit(Op.STORE_LOCAL, len_slot)
        # counter in Slot
        cnt_slot = self._alloc_anon_slot("integer")
        self.fn.emit(Op.LOAD_CONST, self.fn.add_const(0))
        self.fn.emit(Op.STORE_LOCAL, cnt_slot)
        # Iter-Var als Local registrieren waehrend Body-Compile.
        var_name = e.var
        prev_slot = self.fn.local_slots.get(var_name)
        var_slot = self._alloc_anon_slot("any")
        self.fn.local_slots[var_name] = var_slot
        try:
            loop_start = len(self.fn.code)
            # counter < len?
            self.fn.emit(Op.LOAD_LOCAL, cnt_slot)
            self.fn.emit(Op.LOAD_LOCAL, len_slot)
            self.fn.emit(Op.LT)
            cond_jump = self.fn.emit(Op.JUMP_IF_FALSE, None)
            # var = iter[counter]
            self.fn.emit(Op.LOAD_LOCAL, iter_slot)
            self.fn.emit(Op.LOAD_LOCAL, cnt_slot)
            self.fn.emit(Op.LOAD_INDEX, 1)
            self.fn.emit(Op.STORE_LOCAL, var_slot)
            # Filter
            skip_jump = None
            if e.filter is not None:
                self._expr(e.filter)
                skip_jump = self.fn.emit(Op.JUMP_IF_FALSE, None)
            # Transform-Wert auf den Stack
            self._expr(e.transform)
            # Filter-Skip springt hier hin (vor Counter-Increment)
            if skip_jump is not None:
                self.fn.patch_jump(skip_jump, len(self.fn.code))
            # counter += 1
            self.fn.emit(Op.LOAD_LOCAL, cnt_slot)
            self.fn.emit(Op.LOAD_CONST, self.fn.add_const(1))
            self.fn.emit(Op.ADD)
            self.fn.emit(Op.STORE_LOCAL, cnt_slot)
            # Loop zurueck
            self.fn.emit(Op.JUMP, loop_start)
            # End
            end_pos = len(self.fn.code)
            self.fn.patch_jump(cond_jump, end_pos)
            # Stack ist jetzt [..., MARKER, val0, val1, ...] -- BUILD
            self.fn.emit(Op.BUILD_TUPLE_DYN)
        finally:
            # Iter-Var wieder de-registrieren (oder vorigen Slot wiederherstellen)
            if prev_slot is None:
                del self.fn.local_slots[var_name]
            else:
                self.fn.local_slots[var_name] = prev_slot

    def _expr_SetComp(self, e: SetComp):
        """Set-Comprehension: gleicher Loop wie ListComp, aber am Ende
        ruft `__SET_DEDUP` das Tupel auf eine deduplizierte Variante."""
        # Body wie ListComp wiederverwenden -- wir bauen einen ListComp-AST
        # mit der gleichen Semantik und delegieren.
        proxy = ListComp(e.var, e.iterable, e.filter, e.transform)
        self._expr_ListComp(proxy)
        self.fn.emit(Op.CALL_BUILTIN, ("__set_dedup", 1))

    def _expr_DictComp(self, e: DictComp):
        """Dict-Comprehension: pro Iteration pushen wir ein 2-Tupel
        (key, value); BUILD_TUPLE_DYN baut daraus ein TUPLE-of-Pairs;
        `__DICT_FROM_PAIRS` macht das _GBMap draus."""
        from .bytecode import COMP_MARKER
        self.fn.emit(Op.LOAD_CONST, self.fn.add_const(COMP_MARKER))
        iter_slot = self._alloc_anon_slot("any")
        self._expr(e.iterable)
        self.fn.emit(Op.CALL_BUILTIN, ("__comp_iter", 1))
        self.fn.emit(Op.STORE_LOCAL, iter_slot)
        self.fn.emit(Op.LOAD_LOCAL, iter_slot)
        self.fn.emit(Op.CALL_BUILTIN, ("len", 1))
        len_slot = self._alloc_anon_slot("integer")
        self.fn.emit(Op.STORE_LOCAL, len_slot)
        cnt_slot = self._alloc_anon_slot("integer")
        self.fn.emit(Op.LOAD_CONST, self.fn.add_const(0))
        self.fn.emit(Op.STORE_LOCAL, cnt_slot)
        var_name = e.var
        prev_slot = self.fn.local_slots.get(var_name)
        var_slot = self._alloc_anon_slot("any")
        self.fn.local_slots[var_name] = var_slot
        try:
            loop_start = len(self.fn.code)
            self.fn.emit(Op.LOAD_LOCAL, cnt_slot)
            self.fn.emit(Op.LOAD_LOCAL, len_slot)
            self.fn.emit(Op.LT)
            cond_jump = self.fn.emit(Op.JUMP_IF_FALSE, None)
            self.fn.emit(Op.LOAD_LOCAL, iter_slot)
            self.fn.emit(Op.LOAD_LOCAL, cnt_slot)
            self.fn.emit(Op.LOAD_INDEX, 1)
            self.fn.emit(Op.STORE_LOCAL, var_slot)
            skip_jump = None
            if e.filter is not None:
                self._expr(e.filter)
                skip_jump = self.fn.emit(Op.JUMP_IF_FALSE, None)
            # Pair (key, value) als 2-Tupel pushen
            self._expr(e.key)
            self._expr(e.value)
            self.fn.emit(Op.BUILD_TUPLE, 2)
            if skip_jump is not None:
                self.fn.patch_jump(skip_jump, len(self.fn.code))
            self.fn.emit(Op.LOAD_LOCAL, cnt_slot)
            self.fn.emit(Op.LOAD_CONST, self.fn.add_const(1))
            self.fn.emit(Op.ADD)
            self.fn.emit(Op.STORE_LOCAL, cnt_slot)
            self.fn.emit(Op.JUMP, loop_start)
            end_pos = len(self.fn.code)
            self.fn.patch_jump(cond_jump, end_pos)
            self.fn.emit(Op.BUILD_TUPLE_DYN)
            # Tuple-of-pairs -> _GBMap
            self.fn.emit(Op.CALL_BUILTIN, ("__dict_from_pairs", 1))
        finally:
            if prev_slot is None:
                del self.fn.local_slots[var_name]
            else:
                self.fn.local_slots[var_name] = prev_slot

    def _expr_SliceAccess(self, e: SliceAccess):
        # Stack-Layout fuer SLICE: [target, lo?, hi?] mit Flag-Argumenten,
        # damit die VM wei0, wieviele Werte zu poppen sind.
        self._expr(e.target)
        if e.lo is not None:
            self._expr(e.lo)
        if e.hi is not None:
            self._expr(e.hi)
        self.fn.emit(Op.SLICE, (e.lo is not None, e.hi is not None))

    # -------- Helfer ---------------------------------------------------
    def _is_field(self, name: str) -> bool:
        cls = self.fn.current_class
        while cls is not None:
            for fd in cls.fields:
                if fd.name == name:
                    return True
            cls = cls.parent
        return False

    def _raw_decl_type(self, e) -> str:
        """Voller deklarierter Typ-String eines Identifiers (z.B.
        'array:integer', 'string') -- fuer Index-/Member-Typinferenz.
        '' wenn unbekannt oder kein Identifier."""
        if not isinstance(e, Identifier):
            return ""
        if self.fn is not None:
            slot = self.fn.local_slots.get(e.name)
            if slot is not None:
                return (self.fn.local_types[slot] or "").lower()
        return self._global_types.get(e.name, "")

    def _field_type(self, e) -> str:
        """Skalartyp eines `Self.feld`-Zugriffs, wenn statisch bekannt.
        Konservativ: nur echte Felder der aktuellen Klasse (inkl. geerbter),
        nur ueber `Self` (Receiver eindeutig). Properties/obj.feld -> ''."""
        tgt = e.target
        if not (isinstance(tgt, Identifier) and tgt.name == "self"):
            return ""
        if self.fn is None or self.fn.current_class is None:
            return ""
        cls = self.fn.current_class
        while cls is not None:
            for fd in cls.fields:
                if fd.name == e.name:
                    return (fd.type_name or "").lower()
            cls = cls.parent
        return ""

    def _load_var(self, name: str):
        if name in self.fn.local_slots:
            self.fn.emit(Op.LOAD_LOCAL, self.fn.local_slots[name])
            return
        # Innerhalb einer Methode: `Self` (case-insensitive im Lexer auf
        # "self" normalisiert) liefert die aktuelle Instanz.
        if name == "self" and self.fn.current_class is not None:
            self.fn.emit(Op.LOAD_SELF)
            return
        if self._is_field(name):
            self.fn.emit(Op.LOAD_FIELD, self.fn.add_const(name))
            return
        # Slot-Pfad fuer compile-time-bekannte Globals -- spart pro
        # Zugriff einen Dict-Lookup auf globals_.
        slot = self._global_slots.get(name)
        if slot is not None:
            self.fn.emit(Op.LOAD_GLOBAL_SLOT, slot)
            return
        self.fn.emit(Op.LOAD_NAME, self.fn.add_const(name))

    def _store_var(self, name: str):
        if name in self.fn.local_slots:
            self.fn.emit(Op.STORE_LOCAL, self.fn.local_slots[name])
            return
        if self._is_field(name):
            self.fn.emit(Op.STORE_FIELD, self.fn.add_const(name))
            return
        slot = self._global_slots.get(name)
        if slot is not None:
            self.fn.emit(Op.STORE_GLOBAL_SLOT, slot)
            return
        self.fn.emit(Op.STORE_NAME, self.fn.add_const(name))


# --- Modul-Helper -------------------------------------------------------

def _eval_literal_default(expr, fn_name: str):
    """Evaluiert einen Default-Parameter-Ausdruck zur Compile-Zeit.

    Im VM-Pfad sind nur Literale erlaubt - Param-referenzierende Defaults
    erfordern den Tree-Walker. Das deckt 90% der Use-Cases ab.
    """
    if isinstance(expr, NumberLit):
        return expr.value
    if isinstance(expr, StringLit):
        return expr.value
    if isinstance(expr, BoolLit):
        return expr.value
    if isinstance(expr, UnaryOp) and expr.op == "-":
        inner = _eval_literal_default(expr.operand, fn_name)
        return -inner
    raise CompileError(
        f"{fn_name}: Default-Parameter muessen Literale sein im VM-Pfad "
        f"(NUMBER, STRING, TRUE, FALSE, -NUMBER). Mit dem Tree-Walker "
        f"sind beliebige Ausdruecke erlaubt."
    )


def _collect_data(stmts: list, out: list) -> None:
    """Rekursiver Walk durch den AST: alle DATA-Werte in `out` sammeln."""
    for stmt in stmts:
        if isinstance(stmt, Data):
            for lit in stmt.values:
                out.append(_eval_data_literal(lit))
        elif isinstance(stmt, If):
            _collect_data(stmt.then_block, out)
            for _, blk in stmt.elseif_branches:
                _collect_data(blk, out)
            _collect_data(stmt.else_block, out)
        elif isinstance(stmt, While):
            _collect_data(stmt.body, out)
        elif isinstance(stmt, Repeat):
            _collect_data(stmt.body, out)
        elif isinstance(stmt, For):
            _collect_data(stmt.body, out)
        elif isinstance(stmt, Select):
            for case in stmt.cases:
                blk = case[-1]
                _collect_data(blk, out)
            _collect_data(stmt.else_block, out)
        elif isinstance(stmt, Try):
            _collect_data(stmt.body, out)
            _collect_data(stmt.catch_block, out)
        elif isinstance(stmt, (SubDecl, FunctionDecl)):
            _collect_data(stmt.body, out)
        elif isinstance(stmt, ClassDecl):
            for m in stmt.methods:
                _collect_data(m.body, out)


def _eval_int_literal(expr):
    """Liefert den Integer-Wert eines Compile-Time-Literals oder None.

    Akzeptiert NumberLit (nur ganzzahlig), UnaryOp(-, NumberLit), und
    UnaryOp(+, NumberLit). Andere Ausdruecke -> None (-> Fehler beim Caller).
    Nur fuer ENUM-Member-Werte gedacht.
    """
    if isinstance(expr, NumberLit):
        v = expr.value
        if isinstance(v, int) and not isinstance(v, bool):
            return v
        return None
    if isinstance(expr, UnaryOp) and expr.op in ("-", "+"):
        inner = _eval_int_literal(expr.operand)
        if inner is None:
            return None
        return -inner if expr.op == "-" else inner
    return None


def _eval_static_class_literal(expr, cls_name: str, member_name: str):
    """STATIC CONST fuer Klassen: Number, String, Bool, oder negierte
    Number. Wirft CompileError bei Ausdruecken."""
    if isinstance(expr, NumberLit):
        return expr.value
    if isinstance(expr, StringLit):
        return expr.value
    if isinstance(expr, BoolLit):
        return expr.value
    if isinstance(expr, UnaryOp) and expr.op == "-" and isinstance(expr.operand, NumberLit):
        return -expr.operand.value
    raise CompileError(
        f"CLASS {cls_name}.{member_name}: STATIC CONST muss ein Literal sein "
        f"(Number, String, Bool oder -Number)"
    )


def _eval_data_literal(lit):
    if isinstance(lit, NumberLit):
        return lit.value
    if isinstance(lit, StringLit):
        return lit.value
    if isinstance(lit, BoolLit):
        return lit.value
    if isinstance(lit, UnaryOp) and lit.op == "-":
        return -_eval_data_literal(lit.operand)
    raise CompileError(
        f"DATA: ungueltiges Literal {type(lit).__name__}"
    )
