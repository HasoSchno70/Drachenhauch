"""Bytecode-Definitionen fuer GameBasic.

Stack-basierte VM. Code ist eine Liste von (op, arg)-Tupeln. arg ist je nach
Op ein int, str, Tupel, oder None.

Variablen-Modell:
- Globale (Top-Level + Konstanten wie BLACK/KEY_*) liegen in einem Dict
  und werden per Name angesprochen (LOAD_NAME / STORE_NAME).
- Lokale (Funktionsparameter und DIMs in Funktionen) liegen in einem
  Index-Array, schneller Zugriff per Slot (LOAD_LOCAL / STORE_LOCAL).
"""
from __future__ import annotations

from dataclasses import dataclass, field
from enum import IntEnum


# Marker-Sentinel fuer dynamische Tupel-Builds (Comprehensions). Wird
# zur Compile-Zeit als const im Const-Pool registriert und vom VM-Op
# `BUILD_TUPLE_DYN` als Stack-Stop-Marker erkannt. Singleton-Object,
# Identity-Vergleich (`is`) ist die billigste und eindeutigste Methode.
class _CompMarker:
    __slots__ = ()
    def __repr__(self):
        return "<COMP-MARKER>"


COMP_MARKER = _CompMarker()


class Op(IntEnum):
    # Stack
    LOAD_CONST     = 1
    POP            = 2
    DUP            = 3

    # Variablen
    LOAD_NAME      = 10    # arg = const-Index des Namens
    STORE_NAME     = 11
    DECLARE_NAME   = 12    # arg = (name_idx, type_idx, default_idx)
    DECLARE_CONST  = 13    # pop value; arg = (name_idx, type_idx_or_None)
    LOAD_LOCAL     = 14    # arg = slot
    STORE_LOCAL    = 15
    DECLARE_LOCAL  = 16    # arg = (slot, type_name, default)

    # Arithmetik
    ADD            = 20
    SUB            = 21
    MUL            = 22
    DIV            = 23
    MOD            = 24
    POW            = 25
    NEG            = 26
    INT_DIV        = 27

    # Vergleich / Logik
    EQ             = 30
    NEQ            = 31
    LT             = 32
    GT             = 33
    LEQ            = 34
    GEQ            = 35
    NOT            = 36

    # Kontrollfluss
    JUMP           = 40    # arg = ip
    JUMP_IF_FALSE  = 41    # pop; springt wenn falsy
    JUMP_IF_TRUE   = 42

    # Aufrufe
    CALL_USER      = 50    # arg = (fn_name, argc)
    CALL_BUILTIN   = 51    # arg = (builtin_name, argc) - dispatched: graphics oder pure
    CALL_METHOD    = 52    # arg = (method_name, argc) - obj liegt unter den Args
    RETURN         = 60
    RETURN_VOID    = 61

    # Bitwise (alle strikt INTEGER, kein FLOAT, kein BOOL)
    BAND           = 62
    BOR            = 63
    BXOR           = 64
    SHL            = 65
    SHR            = 66
    BNOT           = 67    # unaer

    # Tupel
    BUILD_TUPLE    = 68    # arg = n, pop n values, push tuple
    UNPACK_TUPLE   = 69    # arg = n, pop tuple (muss Laenge n haben), push n values

    # Function References (FUNCREF)
    LOAD_FUNCREF   = 53    # arg = const-Index des fn-Namens (str), push _FuncRef
    CALL_VALUE     = 54    # arg = argc, pop n args, pop callee (FuncRef), dispatch

    # Slicing
    SLICE          = 55    # arg = (has_lo, has_hi); pops [target, lo?, hi?], push slice

    # Mitgliedschaft: pop haystack, pop needle, push bool
    IN_OP          = 56

    # Comprehension-Marker + dynamische Tupel-Bildung. Der Compiler pusht
    # einen Marker-Wert vor dem Loop, der Loop-Body pusht jeweils einen
    # Wert pro Iteration, am Ende sammelt BUILD_TUPLE_DYN alle Werte ueber
    # dem Marker (inklusive Marker-Pop) zu einem Tupel.
    BUILD_TUPLE_DYN = 57

    # Spezialisierte Numeric-Numeric Opcodes. Der Compiler emittiert diese,
    # wenn er per statischer Type-Inferenz weiss, dass beide Operanden
    # weder STRING noch BOOL noch User-Klasse sind, sondern INTEGER/FLOAT.
    # Body ist nur `pop b; pop a; push a OP b` -- keine isinstance-Checks,
    # kein Modul-Operator-Dispatch, kein User-Operator-Dispatch.
    # Bit-identisch zu den generischen Ops fuer Numeric-Operanden.
    # Slot-basierte Globals (Compile-Zeit aufgeloest). Spart pro Zugriff
    # eine Dict-Suche -- aus globals_[name] wird global_slots[idx]. Der
    # Compiler erkennt alle Top-Level-DIM/CONST/MultiDim/FOR-var/Enum/
    # Class-Static und weist ihnen einen Slot-Index zu. Pre-registrierte
    # Globals (KEY_*, COLORS, PI) bleiben im dict -- die werden vom
    # Compiler nicht erkannt und gehen weiterhin durch LOAD_NAME.
    LOAD_GLOBAL_SLOT       = 111   # arg = slot_idx
    STORE_GLOBAL_SLOT      = 112   # arg = slot_idx
    DECLARE_GLOBAL_SLOT    = 113   # arg = (slot_idx, type_name, default)
    DECLARE_GLOBAL_CONST_SLOT = 114   # arg = (slot_idx, type_or_None); pop value

    ADD_NN         = 100
    SUB_NN         = 101
    MUL_NN         = 102
    DIV_NN         = 103   # Int/Int mit Rest=0 -> Int, sonst Float (wie OP.DIV)
    LT_NN          = 104
    GT_NN          = 105
    LEQ_NN         = 106
    GEQ_NN         = 107
    EQ_NN          = 108
    NEQ_NN         = 109
    NEG_N          = 110

    # I/O
    PRINT          = 70    # arg = count
    INPUT_NAME     = 71    # arg = (name_idx, has_prompt)
    INPUT_LOCAL    = 72    # arg = (slot, has_prompt)

    # OOP / Member
    NEW_INSTANCE   = 80    # arg = (class_name, argc, has_init_args)
    LOAD_FIELD     = 81    # arg = name_idx (in Methoden: Zugriff auf self.field)
    STORE_FIELD    = 82
    LOAD_MEMBER    = 83    # arg = name_idx, pop obj
    STORE_MEMBER   = 84    # arg = name_idx, pop value, pop obj
    DECLARE_STRUCT_NAME  = 86  # arg = (name_idx, class_name) - auto-init
    DECLARE_STRUCT_LOCAL = 87  # arg = (slot, class_name)
    LOAD_SELF      = 88    # push aktuelles self_obj (innerhalb einer Methode)

    # Arrays
    LOAD_INDEX     = 90    # arg = num_indices, pop indices, pop array
    STORE_INDEX    = 91    # arg = num_indices, pop value, pop indices, pop array
    DECLARE_ARRAY_NAME  = 92  # arg = (name_idx, element_type, num_dims) - dims auf dem Stack
    DECLARE_ARRAY_LOCAL = 93  # arg = (slot, element_type, num_dims)

    # Exceptions
    TRY_BEGIN      = 95    # arg = catch_target ip
    TRY_END        = 96    # pop Try-Handler (kein Exception-Pfad)
    THROW          = 97    # pop value, raise

    # DATA / READ / RESTORE
    PUSH_DATA      = 75    # Liest naechsten Wert aus module.data, push auf Stack
    RESET_DATA_PTR = 76    # data_ptr = 0

    # Ende
    HALT           = 99


@dataclass
class CompiledFunction:
    """Eine kompilierte Funktion (oder das Top-Level-Hauptprogramm)."""

    name: str
    code: list = field(default_factory=list)            # list[tuple[int, object]]
    constants: list = field(default_factory=list)       # const-Pool
    n_params: int = 0
    n_required: int = 0     # Anzahl Pflicht-Parameter (ohne Default)
    # Variadic-Flag: wenn True, ist der LETZTE Parameter ein TUPLE, in das
    # alle Positional-Args ab Position n_params-1 vom Caller gesammelt
    # werden. Defaults und Variadic schliessen sich aus (Parser-Check).
    is_variadic: bool = False
    # Default-Werte fuer Parameter mit Default. List-Index = Parameter-Index.
    # Slot ist None fuer Pflicht-Parameter, oder ein evaluierter Literal-Wert
    # fuer Defaults. (Im VM-Pfad sind nur Literal-Defaults erlaubt; der
    # Tree-Walker unterstuetzt auch Param-referenzierende Defaults.)
    param_defaults: list = field(default_factory=list)
    # Parameter-Namen in Decl-Reihenfolge (lower-case). Compile-Zeit-Info,
    # die der Compiler fuer das Aufloesen von Named-Args braucht. Die VM
    # selbst nutzt das Feld nicht.
    param_names: list = field(default_factory=list)
    local_types: list = field(default_factory=list)     # parallel zu Slot-Indizes
    local_defaults: list = field(default_factory=list)  # default-Wert je Slot
    return_type: str = ""
    is_sub: bool = True
    is_main: bool = False
    # Inline-Cache pro Bytecode-Instruction (parallel zu code). Lazy von der
    # VM gefuellt. Nutzt monomorphic-IC-Schema:
    #   caches[ip] = None  -> Cache leer / nicht-cacheable Op
    #   caches[ip] = [recv_cls, payload]:
    #       - CALL_METHOD: payload = method (CompiledFunction)
    #       - LOAD/STORE_MEMBER bei Property: payload = getter/setter method
    #       - LOAD/STORE_MEMBER bei Field:    payload = None
    # Hit-Check: cache is not None and obj.cls is cache[0].
    # Bei Polymorphie (selber Call-Site, mehrere Klassen) thrasht der Cache;
    # ein Miss kostet eine Population (dict-Lookup), Korrektheit bleibt.
    caches: list = field(default_factory=list)
    # Quell-Zeilennummer pro Bytecode-Instruction (parallel zu code), gestempelt
    # vom Compiler aus `stmt.line`. Optional -- leer wenn nicht getrackt. Wird
    # von den Python/Cython-VMs ignoriert; die native Rust-Runtime nutzt es fuer
    # Laufzeitfehler-Meldungen mit `datei.gb:Zeile`.
    lines: list = field(default_factory=list)

    @property
    def n_locals(self) -> int:
        return len(self.local_types)


@dataclass
class VMFieldDecl:
    """Feld-Deklaration in einer Klasse fuer die VM."""
    name: str
    type_name: str       # Element- oder Skalartyp
    array_dims: tuple = ()   # () = Skalar, sonst Tuple von kompilezeit-bekannten Groessen


@dataclass
class VMClassInfo:
    """Klassen-Metadaten fuer die VM."""
    name: str
    parent_name: str = ""
    parent: object = None     # _VMClassInfo or None, zur Laufzeit aufgeloest
    is_struct: bool = False
    fields: list = field(default_factory=list)    # list[VMFieldDecl] (eigene)
    methods: dict = field(default_factory=dict)   # name -> CompiledFunction (eigene)
    # Property-Set: lower-case Property-Namen, die in dieser Klasse oder
    # einer Superklasse einen Getter ODER Setter haben. Gleicher Mechanismus
    # wie im Tree-Walker (_ClassInfo.properties).
    properties: set = field(default_factory=set)
    # MRO-Cache: name -> (CompiledFunction, defining_VMClassInfo). Lazy gefuellt
    # vom ersten _resolve_method()-Lookup. GB-Klassen sind nach Definition
    # immutable, also keine Invalidation noetig.
    _method_cache: dict = field(default_factory=dict)


@dataclass
class Module:
    """Kompiliertes Programm: Hauptfunktion + benannte Funktionen + Klassen."""
    main: CompiledFunction
    functions: dict     # name -> CompiledFunction
    classes: dict = field(default_factory=dict)   # name -> VMClassInfo
    # DATA-Werte aus dem Quelltext (Source-Reihenfolge, alle DATA-Statements).
    # READ liest sequenziell, RESTORE setzt die VM-interne Position auf 0.
    data: list = field(default_factory=list)
    # Anzahl der vom Compiler erkannten Top-Level-Globals (DIM/CONST/...).
    # Die VM alloziert eine Liste dieser Groesse fuer Slot-basierten Zugriff.
    # Pre-registrierte Globals (KEY_*, COLORS, PI) gehen weiterhin durch
    # das globals_-Dict.
    n_globals: int = 0
