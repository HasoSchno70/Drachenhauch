"""Token-Typen und Schluesselwoerter."""
from dataclasses import dataclass
from enum import Enum, auto


class TokenType(Enum):
    # Literale
    NUMBER = auto()
    STRING = auto()
    IDENT = auto()

    # Schluesselwoerter
    DIM = auto()
    AS = auto()
    INTEGER = auto()
    FLOAT = auto()
    STRING_TYPE = auto()
    BOOLEAN = auto()
    PRINT = auto()
    INPUT = auto()
    IF = auto()
    THEN = auto()
    ELSE = auto()
    ELSEIF = auto()
    END = auto()
    WHILE = auto()
    WEND = auto()
    FOR = auto()
    TO = auto()
    STEP = auto()
    NEXT = auto()
    SUB = auto()
    FUNCTION = auto()
    RETURN = auto()
    CLASS = auto()
    NEW = auto()
    EXTENDS = auto()
    TRUE = auto()
    FALSE = auto()
    AND = auto()
    OR = auto()
    NOT = auto()
    MOD = auto()
    CONST = auto()
    BREAK = auto()
    CONTINUE = auto()
    IMAGE = auto()
    SOUND = auto()
    ARRAY = auto()
    OF = auto()
    STRUCT = auto()
    FILE = auto()
    MAP = auto()
    TRY = auto()
    CATCH = auto()
    THROW = auto()
    IMPORT = auto()
    SELECT = auto()
    CASE = auto()
    IS = auto()
    REPEAT = auto()
    UNTIL = auto()
    DATA = auto()
    READ = auto()
    RESTORE = auto()
    BYREF = auto()
    ENUM = auto()
    # Bitwise-Operatoren (alle strikt INTEGER, kein Float, kein Bool).
    BAND = auto()
    BOR = auto()
    BXOR = auto()
    BNOT = auto()
    SHL = auto()
    SHR = auto()
    TUPLE = auto()
    WITH = auto()
    STATIC = auto()
    FUNCREF = auto()
    IN = auto()
    ELLIPSIS = auto()    # `...` als Variadic-Marker in Param-Listen
    WHERE = auto()       # Guard-Klausel in CASE
    PROPERTY = auto()    # PROPERTY GET/SET in Klassen
    # GET/SET sind KEINE Keywords -- sie sind kontext-abhaengig nach
    # PROPERTY und werden als Identifier ("get"/"set") gelesen. So bleibt
    # `FUNCTION Get() AS ...` als normale Methode unverandert moeglich.
    OPERATOR = auto()    # OPERATOR <op> in Klassen -- Operator-Overloading.

    # Operatoren / Satzzeichen
    PLUS = auto()
    MINUS = auto()
    STAR = auto()
    SLASH = auto()
    INTDIV = auto()      # \ = ganzzahlige Division
    CARET = auto()
    EQ = auto()
    NEQ = auto()
    LT = auto()
    GT = auto()
    LEQ = auto()
    GEQ = auto()
    LPAREN = auto()
    RPAREN = auto()
    LBRACKET = auto()
    RBRACKET = auto()
    LBRACE = auto()    # `{` -- nur fuer Dict/Set-Comprehensions
    RBRACE = auto()    # `}` -- nur fuer Dict/Set-Comprehensions
    COMMA = auto()
    DOT = auto()
    SEMICOLON = auto()
    COLON = auto()

    # Compound-Assignment-Operatoren (Parser de-sugared sie zu OP=)
    PLUS_EQ = auto()
    MINUS_EQ = auto()
    STAR_EQ = auto()
    SLASH_EQ = auto()

    NEWLINE = auto()
    EOF = auto()


@dataclass
class Token:
    type: TokenType
    value: object
    line: int
    col: int
    end_line: int = 0
    end_col: int = 0

    def __repr__(self):
        return f"{self.type.name}({self.value!r})"


KEYWORDS = {
    "dim": TokenType.DIM,
    "as": TokenType.AS,
    "integer": TokenType.INTEGER,
    "float": TokenType.FLOAT,
    "string": TokenType.STRING_TYPE,
    "boolean": TokenType.BOOLEAN,
    "print": TokenType.PRINT,
    "input": TokenType.INPUT,
    "if": TokenType.IF,
    "then": TokenType.THEN,
    "else": TokenType.ELSE,
    "elseif": TokenType.ELSEIF,
    "elif":   TokenType.ELSEIF,    # kuerzeres Alias, gleiches Token
    "end": TokenType.END,
    "while": TokenType.WHILE,
    "wend": TokenType.WEND,
    "for": TokenType.FOR,
    "to": TokenType.TO,
    "step": TokenType.STEP,
    "next": TokenType.NEXT,
    "sub": TokenType.SUB,
    "function": TokenType.FUNCTION,
    "return": TokenType.RETURN,
    "class": TokenType.CLASS,
    "new": TokenType.NEW,
    "extends": TokenType.EXTENDS,
    "true": TokenType.TRUE,
    "false": TokenType.FALSE,
    "and": TokenType.AND,
    "or": TokenType.OR,
    "not": TokenType.NOT,
    "mod": TokenType.MOD,
    "const": TokenType.CONST,
    "break": TokenType.BREAK,
    "continue": TokenType.CONTINUE,
    "image": TokenType.IMAGE,
    "sound": TokenType.SOUND,
    "array": TokenType.ARRAY,
    "of": TokenType.OF,
    "struct": TokenType.STRUCT,
    "file": TokenType.FILE,
    "map": TokenType.MAP,
    "try": TokenType.TRY,
    "catch": TokenType.CATCH,
    "throw": TokenType.THROW,
    "import": TokenType.IMPORT,
    "select": TokenType.SELECT,
    "case": TokenType.CASE,
    "is": TokenType.IS,
    "repeat": TokenType.REPEAT,
    "until": TokenType.UNTIL,
    "data": TokenType.DATA,
    "read": TokenType.READ,
    "restore": TokenType.RESTORE,
    "byref": TokenType.BYREF,
    "enum": TokenType.ENUM,
    "band": TokenType.BAND,
    "bor": TokenType.BOR,
    "bxor": TokenType.BXOR,
    "bnot": TokenType.BNOT,
    "shl": TokenType.SHL,
    "shr": TokenType.SHR,
    "tuple": TokenType.TUPLE,
    "with": TokenType.WITH,
    "static": TokenType.STATIC,
    "funcref": TokenType.FUNCREF,
    "in": TokenType.IN,
    "where": TokenType.WHERE,
    "property": TokenType.PROPERTY,
    "operator": TokenType.OPERATOR,
}
