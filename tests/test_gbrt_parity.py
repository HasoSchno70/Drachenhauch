"""TW <-> gbrt Paritaets-Sweep.

Seit dem Entfernen der Python-/Cython-Bytecode-VMs gibt es nur noch zwei
Ausfuehrungspfade: den **Tree-Walker** (Python, Referenz) und die native
Runtime **gbrt** (Rust, Produktion). Dieser Sweep ist der Waechter gegen Drift
zwischen beiden: er fuehrt dieselben Programme durch den Tree-Walker (in-process)
UND durch gbrt (Subprozess, kompiliert via Python-Toolchain -> .gbc) und prueft
bit-identische Ausgabe (modulo Zeilenende: Python CRLF, gbrt LF).

Abgedeckt:
- ~30 Inline-Snippets fuer Sprach-/Compiler-Features (frueher per `run_all`).
- Eine Auswahl deterministischer, RND-/Uhr-/IO-freier Beispiel-Programme.

Skippt komplett, wenn `gbrt` nicht gebaut ist (kein Build-Zwang fuer die Suite).
Programme mit RND/MILLIS/TIME$ sind bewusst NICHT dabei -- gbrt nutzt einen
anderen PRNG/Clock als Python, dort ist Bit-Identitaet per Definition nicht
gegeben (wie schon zwischen den fruehen Pfaden dokumentiert).
"""
import io
import os
import contextlib
import tempfile
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_EXAMPLES = _ROOT / "examples"


def _find_gbrt():
    base = _ROOT / "rust" / "gb_runtime" / "target"
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


_GBRT = _find_gbrt()
pytestmark = pytest.mark.skipif(
    _GBRT is None,
    reason="native Runtime 'gbrt' nicht gebaut (rust/build_runtime.py)")


def _tw(source: str, base: Path | None = None) -> str:
    """Tree-Walker-Ausgabe (in-process)."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.interpreter import Interpreter
    from gamebasic.preprocess import process
    prepped, _ = process(source, base or _ROOT, file_label="<parity>")
    ast = Parser(Lexer(prepped).tokenize()).parse()
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        Interpreter().run(ast)
    return buf.getvalue()


def _gbrt(source: str, base: Path | None = None, cwd: Path | None = None) -> str:
    """Kompiliert `source` zu .gbc und fuehrt es mit gbrt aus -> stdout (LF)."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler
    from gamebasic.serialize import dump_gbc
    from gamebasic.preprocess import process
    prepped, _ = process(source, base or _ROOT, file_label="<parity>")
    ast = Parser(Lexer(prepped).tokenize()).parse()
    module = Compiler().compile(ast)
    fd, tmp = tempfile.mkstemp(suffix=".gbc")
    os.close(fd)
    try:
        dump_gbc(module, tmp)
        res = subprocess.run([str(_GBRT), tmp, "<parity>"],
                             capture_output=True, text=True,
                             cwd=str(cwd) if cwd else None, timeout=60)
        return res.stdout.replace("\r\n", "\n")
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass


# --- Inline-Snippets: Sprach-/Compiler-Feature-Abdeckung ---------------------

_SNIPPETS = {
    "arithmetic": 'PRINT 1 + 2 * 3 - 4 \\ 2 : PRINT 7 MOD 3 : PRINT 2 ^ 10',
    "float_fmt": 'PRINT 1.0 : PRINT 3.14159 : PRINT 1.0 / 3.0 : PRINT 100000000000000.0 : PRINT 0.0001',
    "int_div_neg": 'PRINT -7 \\ 2 : PRINT -7 MOD 2 : PRINT 7 MOD -2',
    "strings": 'PRINT "Hello".upper() : PRINT "  hi  ".trim() : PRINT LEN("abc")',
    "string_mul": 'PRINT "-" * 10 : PRINT 3 * "ab"',
    "slicing": 'PRINT "Hello World"[6:11] : PRINT "abcdef"[2:]',
    "arrays": '''DIM a[3] AS INTEGER
a[0] = 3
a[1] = 1
a[2] = 2
SORT(a)
PRINT a[0], a[1], a[2]
PRINT ARRAY_INDEXOF(a, 2)''',
    "maps": '''DIM m AS MAP OF INTEGER
MAPPUT(m, "x", 5)
MAPPUT(m, "y", 9)
PRINT MAPGET(m, "x"), MAPHAS(m, "x"), MAPHAS(m, "z")''',
    "tuples": '''FUNCTION mm(a AS INTEGER, b AS INTEGER) AS TUPLE
IF a < b THEN RETURN (a, b)
RETURN (b, a)
END FUNCTION
DIM lo AS INTEGER
DIM hi AS INTEGER
(lo, hi) = mm(7, 3)
PRINT lo, hi''',
    "oop": '''CLASS P
DIM hp AS INTEGER
SUB Init()
Self.hp = 100
END SUB
FUNCTION get() AS INTEGER
RETURN Self.hp
END FUNCTION
END CLASS
DIM p AS P
p = NEW P()
PRINT p.get()''',
    "inheritance": '''CLASS A
FUNCTION who() AS STRING
RETURN "A"
END FUNCTION
FUNCTION tag() AS STRING
RETURN "tag-" + Self.who()
END FUNCTION
END CLASS
CLASS B EXTENDS A
FUNCTION who() AS STRING
RETURN "B"
END FUNCTION
END CLASS
DIM b AS B
b = NEW B()
PRINT b.who()
PRINT b.tag()''',
    "operator_overload": '''CLASS Money
DIM cents AS INTEGER
OPERATOR + (other AS Money) AS Money
DIM r AS Money
r = NEW Money()
r.cents = Self.cents + other.cents
RETURN r
END OPERATOR
END CLASS
DIM a AS Money
a = NEW Money()
a.cents = 100
DIM b AS Money
b = NEW Money()
b.cents = 50
PRINT (a + b).cents''',
    "properties": '''CLASS C
DIM _v AS INTEGER
PROPERTY GET v() AS INTEGER
RETURN Self._v
END PROPERTY
PROPERTY SET v(value AS INTEGER)
IF value > 100 THEN value = 100
Self._v = value
END PROPERTY
END CLASS
DIM c AS C
c = NEW C()
c.v = 200
PRINT c.v''',
    "static": '''CLASS Cfg
STATIC CONST MAX AS INTEGER = 42
END CLASS
PRINT Cfg.MAX''',
    "enum": '''ENUM State = MENU, PLAYING, PAUSED
PRINT State.PLAYING''',
    "select_case": '''DIM x AS INTEGER
x = 15
SELECT CASE x
CASE 1 TO 9
PRINT "low"
CASE 10 TO 20
PRINT "mid"
CASE ELSE
PRINT "high"
END SELECT''',
    "select_guard": '''DIM hp AS INTEGER
DIM pot AS BOOLEAN
hp = 20
pot = TRUE
SELECT CASE hp
CASE IS <= 30 WHERE pot
PRINT "heal"
CASE IS <= 30
PRINT "flee"
END SELECT''',
    "in_op": 'PRINT "World" IN "Hello World" : PRINT 5 IN (1, 5, 9)',
    "bitwise": 'PRINT 6 BAND 3 : PRINT 6 BOR 1 : PRINT 1 SHL 4 : PRINT BNOT 0',
    "ternary_iif": 'PRINT IIF(5 > 3, "yes", "no") : PRINT IIF(0 <> 0, 1, -1)',
    "for_each": '''FOR EACH ch IN "abc"
PRINT ch
NEXT''',
    "list_comp": '''DIM nums AS TUPLE
nums = (1, 2, 3, 4, 5, 6)
PRINT [n * n FOR n IN nums WHERE n MOD 2 = 0]''',
    "dict_comp": '''DIM sq AS MAP OF INTEGER
sq = {STR$(x): x * x FOR x IN (1, 2, 3)}
PRINT MAPGET(sq, "3")''',
    "set_comp": 'PRINT {x MOD 3 FOR x IN (0, 1, 2, 3, 4, 5)}',
    "variadic": '''SUB logv(level AS STRING, ...rest)
DIM s AS STRING
s = "[" + level + "]"
DIM j AS INTEGER
FOR j = 0 TO LEN(rest) - 1
s = s + " " + STR$(rest[j])
NEXT
PRINT s
END SUB
logv("INFO", "a", "b", "c")''',
    "tuple_method": '''DIM t AS TUPLE
t = (10, 20, 30)
PRINT t.length(), t.len(), LEN(t)''',
    "with": '''CLASS Pt
DIM x AS INTEGER
DIM y AS INTEGER
END CLASS
DIM p AS Pt
p = NEW Pt()
WITH p
.x = 3
.y = 4
END WITH
PRINT p.x, p.y''',
    "funcref": '''FUNCTION sq(x AS INTEGER) AS INTEGER
RETURN x * x
END FUNCTION
DIM f AS FUNCREF
f = sq
PRINT f(7)''',
    "fstring": '''DIM name AS STRING
DIM hp AS INTEGER
name = "Anna"
hp = 75
PRINT f"{name} hat {hp} HP" : PRINT f"FPS {59.7:.1f} Score {42:05d}"''',
    "named_args": '''FUNCTION greet(name AS STRING, greeting AS STRING = "Hallo") AS STRING
RETURN greeting + ", " + name
END FUNCTION
PRINT greet(name: "Bob")
PRINT greet("Eve", greeting: "Hi")''',
    "try_throw": '''TRY
THROW "boom"
CATCH msg
PRINT "caught: " + msg
END TRY''',
    "break_continue": '''DIM i AS INTEGER
FOR i = 0 TO 10
IF i = 3 THEN CONTINUE
IF i = 6 THEN BREAK
PRINT i
NEXT''',
    "coroutine": '''FUNCTION counter() AS INTEGER
YIELD 1
YIELD 2
RETURN 99
END FUNCTION
DIM c AS COROUTINE
c = counter()
PRINT CORO_RESUME(c)
PRINT CORO_RESUME(c)
PRINT CORO_RESUME(c)
PRINT CORO_DONE(c)''',
    "select_case_string": '''DIM cmd AS STRING
cmd = "save"
SELECT CASE cmd
CASE "load", "save"
PRINT "io"
CASE ELSE
PRINT "other"
END SELECT''',
}


@pytest.mark.parametrize("name", sorted(_SNIPPETS))
def test_snippet_tw_eq_gbrt(name):
    src = _SNIPPETS[name]
    tw = _tw(src).replace("\r\n", "\n")
    nv = _gbrt(src)
    assert tw == nv, f"TW != gbrt fuer Snippet '{name}':\n  TW={tw!r}\n  gbrt={nv!r}"


# --- Beispiel-Programme (deterministisch, RND-/Uhr-/IO-frei, ohne SCREEN) -----

# RND-/Uhr-/IO-/SCREEN-frei und ohne von Fehler-Texten abhaengige Ausgabe
# (Fehlermeldungs-Wortlaut darf zwischen Python und Rust abweichen, z.B. 24_json).
_PARITY_EXAMPLES = [
    "01_hello", "02_variables", "03_loops", "04_fibonacci",
    "06_functions", "07_classes", "08_inheritance",
    "11_arrays", "14_strings", "15_struct", "20_try", "30_select",
]


@pytest.mark.parametrize("name", _PARITY_EXAMPLES)
def test_example_tw_eq_gbrt(name):
    path = _EXAMPLES / f"{name}.gb"
    source = path.read_text(encoding="utf-8")
    tw = _tw(source, base=path.parent).replace("\r\n", "\n")
    nv = _gbrt(source, base=path.parent, cwd=path.parent)
    assert tw == nv, f"TW != gbrt fuer Beispiel '{name}'"
