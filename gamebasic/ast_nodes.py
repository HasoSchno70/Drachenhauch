"""AST-Knoten fuer GameBasic."""
from dataclasses import dataclass, field
from typing import Optional


class Node:
    pass


class Stmt(Node):
    pass


class Expr(Node):
    pass


# --- Ausdruecke -------------------------------------------------------

@dataclass
class NumberLit(Expr):
    value: object  # int oder float


@dataclass
class StringLit(Expr):
    value: str


@dataclass
class BoolLit(Expr):
    value: bool


@dataclass
class Identifier(Expr):
    name: str


@dataclass
class BinaryOp(Expr):
    op: str
    left: Expr
    right: Expr


@dataclass
class UnaryOp(Expr):
    op: str
    operand: Expr


@dataclass
class TernaryExpr(Expr):
    """IIF(cond, then_expr, else_expr) -- nur EIN Zweig wird ausgewertet."""
    cond: Expr
    then_expr: Expr
    else_expr: Expr


@dataclass
class Yield(Expr):
    """YIELD <expr> -- suspendiert die Coroutine und liefert <expr> an den
    Aufrufer. Als Ausdruck evaluiert YIELD zum via CORO_SEND uebergebenen
    Wert (NIL bei CORO_RESUME). `value` ist optional (YIELD ohne Wert)."""
    value: Optional[Expr] = None


@dataclass
class Call(Expr):
    callee: Expr
    args: list   # list[Expr | NamedArg]


@dataclass
class ListComp(Expr):
    """List-Comprehension: `[expr FOR var IN iterable WHERE filter]`.

    Liefert ein TUPLE der transformierten Werte. WHERE-Filter ist optional.
    Iterierbar sind STRING (chars), TUPLE, 1D-ARRAY und MAP (Keys). `var`
    ist eine implizit-deklarierte Schleifenvariable, die nur im Comp-
    Kontext lebt.
    """
    var: str            # Name der Iter-Variable
    iterable: Expr      # Container-Ausdruck
    filter: Optional[Expr]   # Optionaler WHERE-Filter
    transform: Expr     # Wert pro Iteration


@dataclass
class DictComp(Expr):
    """Dict-Comprehension: `{key: val FOR var IN iterable WHERE filter}`.

    Liefert eine MAP. Der `key` muss zur Laufzeit STRING sein
    (GameBasic-MAPs haben STRING-Keys). `value` darf beliebig sein --
    der MAP-Wert-Typ wird beim ersten Eintrag inferred und dann durchgehend
    enforced.
    """
    var: str
    iterable: Expr
    filter: Optional[Expr]
    key: Expr
    value: Expr


@dataclass
class SetComp(Expr):
    """Set-Comprehension: `{expr FOR var IN iterable WHERE filter}`.

    GameBasic hat keinen echten SET-Typ -- wir liefern ein TUPLE mit
    de-duplizierten Werten in der Reihenfolge des ersten Auftretens.
    Praktisch fuer "alle eindeutigen Spieler-Namen" oder aehnliche
    Aggregate.
    """
    var: str
    iterable: Expr
    filter: Optional[Expr]
    transform: Expr


@dataclass
class TupleLit(Expr):
    """Geklammertes Tupel-Literal mit mindestens 2 Elementen:
    `(a, b)`, `(1, 2, 3)`. Eine einzelne geklammerte Expression `(expr)`
    bleibt eine normale Klammer-Gruppierung -- erst das Komma macht's
    zum Tupel.
    """
    elements: list   # list[Expr]


@dataclass
class TupleAssign(Stmt):
    """Destructuring-Zuweisung: `(x, y, z) = expr`.

    targets sind Assignment-Lvalues (Identifier, MemberAccess, IndexAccess) -
    also alles, was rechts von `=` als einzelner Assignment-Pfad legitim waere.
    value muss zur Laufzeit ein Tupel mit exakt len(targets) Elementen ergeben,
    sonst GBRuntimeError.
    """
    targets: list   # list[Expr] - Assignment-Pfade
    value: Expr


@dataclass
class NamedArg(Expr):
    """Ein einzelnes benanntes Argument in einem Funktionsaufruf:
    `func(x: 5, y: 10)` -> Call(callee, [NamedArg("x", 5), NamedArg("y", 10)]).
    Tritt nur als Element von `Call.args` auf - nicht als eigenstaendiger
    Ausdruck.
    """
    name: str
    value: Expr


# --- Anweisungen ------------------------------------------------------

@dataclass
class Dim(Stmt):
    name: str
    type_name: str
    array_dims: Optional[list] = None   # None = Skalar, Liste von Expr = Array-Groessen je Dimension


@dataclass
class MultiDim(Stmt):
    """DIM mit mehreren Variablen gleichen Typs:
    `DIM a, b, c AS INTEGER` oder `DIM x[10], y, z[5, 5] AS INTEGER`.

    Wird im Interpreter als Folge einzelner Dim-Statements ausgefuehrt;
    der Compiler emittiert die einzelnen DIM-Bytecodes hintereinander -
    deshalb sieht die VM nur normale Dim-Operationen.
    """
    dims: list   # list[Dim]


@dataclass
class Const(Stmt):
    name: str
    type_name: Optional[str]   # None -> wird vom Wert abgeleitet
    value: Expr


@dataclass
class Break(Stmt):
    pass


@dataclass
class Continue(Stmt):
    pass


@dataclass
class IndexAccess(Expr):
    target: Expr
    indices: list   # list[Expr]


@dataclass
class SliceAccess(Expr):
    """Slice-Zugriff: `s[lo:hi]`, `s[lo:]`, `s[:hi]`, `s[:]`.

    Funktioniert auf Strings (liefert Substring) und 1D-Arrays (liefert
    neues Array). Negative Indices und Step werden NICHT unterstuetzt.
    `lo`/`hi` koennen None sein -- dann werden die Default-Bounds
    (0 bzw. len) verwendet.
    """
    target: Expr
    lo: Optional[Expr]
    hi: Optional[Expr]


@dataclass
class IndexAssign(Stmt):
    target: Expr
    indices: list   # list[Expr]
    value: Expr


@dataclass
class Try(Stmt):
    body: list
    catch_var: str   # "" wenn kein Name angegeben
    catch_block: list


@dataclass
class Throw(Stmt):
    value: Expr


@dataclass
class Assign(Stmt):
    name: str
    value: Expr


@dataclass
class Print(Stmt):
    items: list           # list[Expr]
    newline: bool = True  # False wenn mit ; abgeschlossen


@dataclass
class Input(Stmt):
    prompt: Optional[Expr]
    target: str


@dataclass
class If(Stmt):
    condition: Expr
    then_block: list
    elseif_branches: list  # list[(Expr, list[Stmt])]
    else_block: list


@dataclass
class CaseMatch:
    """Eine Match-Spec innerhalb eines CASE: entweder ein einzelner Wert,
    eine Liste von Werten, oder ein Bereich. kind ist "value" oder "range".
    Bei "value": values = [Expr]; bei "range": values = [Expr_lo, Expr_hi]."""
    kind: str
    values: list


@dataclass
class Select(Stmt):
    subject: Expr
    cases: list         # list[(list[CaseMatch], list[Stmt])]
    else_block: list    # leer wenn kein CASE ELSE


@dataclass
class While(Stmt):
    condition: Expr
    body: list


@dataclass
class For(Stmt):
    var: str
    start: Expr
    end: Expr
    step: Optional[Expr]
    body: list


@dataclass
class ForEach(Stmt):
    """FOR EACH var IN container ... NEXT -- iteriert ueber String (Zeichen),
    TUPLE, 1D-ARRAY oder MAP (Keys)."""
    var: str
    iterable: Expr
    body: list


@dataclass
class Repeat(Stmt):
    """REPEAT body UNTIL condition.

    Post-Test-Schleife: body laeuft mindestens einmal, wird wiederholt
    solange condition FALSE ist (also "bis" condition TRUE wird).
    """
    body: list
    condition: Expr


@dataclass
class Data(Stmt):
    """DATA literal1, literal2, ...
    Werte werden beim Programmstart in die globale Data-Liste eingefuegt
    (Reihenfolge = Source-Reihenfolge). Selbst kein Effekt zur Laufzeit.
    """
    values: list   # Liste von NumberLit / StringLit / BoolLit / UnaryOp(-, Number)


@dataclass
class Read(Stmt):
    """READ var, var, ...
    Holt nacheinander Werte aus der Data-Liste, weist sie den Targets zu.
    Targets sind Identifier, IndexAccess oder MemberAccess (jedes
    Assignment-Ziel).
    """
    targets: list


@dataclass
class Restore(Stmt):
    """RESTORE setzt den Data-Read-Pointer auf 0 zurueck.
    (Klassisches BASIC kennt RESTORE label - hier vereinfacht ohne Labels.)
    """
    pass


@dataclass
class EnumDecl(Stmt):
    """ENUM Name = M1, M2 [, ...]
       ENUM Name
           M1 [= expr]
           M2 [= expr]
       END ENUM

    Members ohne expliziten Wert werden auto-nummeriert: 0, 1, 2, ...
    Wenn ein Member einen expliziten Wert hat, zaehlt der naechste
    automatische Member von dort weiter (= klassisches Pascal/C-Verhalten).
    members ist list[(name: str, expr: Optional[Expr])] - die Auswertung
    der expliziten Werte passiert zur Laufzeit, weil Defaults Konstanten-
    Ausdruecke sein duerfen.
    """
    name: str
    members: list   # list[(str, Optional[Expr])]


@dataclass
class ExprStmt(Stmt):
    expr: Expr


@dataclass
class Param:
    name: str
    type_name: str
    default: Optional["Expr"] = None    # Default-Ausdruck oder None
    by_ref: bool = False                # BYREF -> Caller-Variable wird modifiziert
    is_variadic: bool = False           # `...args` -- sammelt rest-Args als TUPLE


@dataclass
class SubDecl(Stmt):
    name: str
    params: list   # list[Param]
    body: list


@dataclass
class FunctionDecl(Stmt):
    name: str
    params: list           # list[Param]
    return_type: str
    body: list


@dataclass
class Return(Stmt):
    value: Optional[Expr]


# --- OOP --------------------------------------------------------------

@dataclass
class PropertyDecl(Stmt):
    """`PROPERTY GET/SET name(...) ... END PROPERTY` innerhalb einer Klasse.

    Properties werden intern als Methoden mit dem Namen `__get_<name>`
    bzw. `__set_<name>` gespeichert. Der Parser erzeugt fuer jeden
    GET/SET-Block einen FunctionDecl/SubDecl mit diesem internen Namen.
    Die Klasse merkt sich die Property-Namen in einem Set, sodass
    `obj.name` und `obj.name = x` zur Laufzeit zu Property-Calls
    werden.
    """
    name: str
    kind: str          # "get" oder "set"
    func: object       # FunctionDecl (bei get) oder SubDecl (bei set)


@dataclass
class ClassDecl(Stmt):
    name: str
    parent: Optional[str]   # Name der Elternklasse oder None
    fields: list            # list[Dim]
    methods: list           # list[SubDecl | FunctionDecl]
    is_struct: bool = False
    # Static-Members: liste von Const-Nodes. Werte muessen Compile-Time-
    # Literale sein (gleiche Regel wie ENUM-Member). Zugriff via
    # `<ClassName>.<MEMBER>` -- Klassennamen werden als CONST in den globalen
    # Scope mit einem _ClassStaticNamespace registriert.
    statics: list = field(default_factory=list)
    # Property-Deklarationen. Jede Property kann optional einen Getter
    # und/oder einen Setter haben -- der Parser fuegt fuer jeden GET/SET-
    # Block einen Eintrag hier ein und ergaenzt parallel die Methoden-
    # Liste mit der internen Repraesentation (`__get_name`/`__set_name`).
    properties: list = field(default_factory=list)


@dataclass
class New(Expr):
    class_name: str
    args: Optional[list]    # None = keine Klammern; [] = Init() mit 0 Args


@dataclass
class MemberAccess(Expr):
    target: Expr
    name: str


@dataclass
class MemberAssign(Stmt):
    target: Expr
    name: str
    value: Expr


@dataclass
class With(Stmt):
    """WITH expr / body / END WITH

    Innerhalb des Body referenziert `.member` automatisch die Member von
    `expr`. Der Parser erzeugt einen impliziten Variablen-Namen
    (`__with_<n>`), in den `expr` einmal gespeichert wird, und desuger jedes
    `.member` zu `MemberAccess(Identifier(var_name), name)`.

    Verschachtelte WITHs sind erlaubt -- innerstes gewinnt (Stack-Semantik).
    """
    var_name: str       # Compiler-generierter, intern unsichtbarer Slot-Name
    target: Expr        # Wird einmal evaluiert
    body: list          # list[Stmt]


@dataclass
class Program(Node):
    statements: list
