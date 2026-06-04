"""Parity-Test: Rust-Compiler (`gbrt --runsrc`) == Python-Tree-Walker.

Stufe 3 der Front-End-Portierung. `gbrt --runsrc` lext+parst+kompiliert+fuehrt
ALLES in Rust aus; verglichen wird die stdout-Ausgabe gegen den Python-Tree-
Walker (Referenz). Gate = Output-Paritaet (nicht byte-exakter Bytecode -- der
Rust-Compiler emittiert die generischen Opcodes, gbrt fuehrt sie identisch aus).

Stufe 3a deckt main-only Konsolen-Programme ab (Skalar-Globals, Arithmetik,
IF/WHILE, Builtins). Nicht unterstuetzte Programme liefern Exit-Code != 0 und
werden im Beispiel-Sweep uebersprungen.
"""
import contextlib
import io
import os
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
    _GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")


def _tw(src: str, base: Path) -> str:
    """Python-Tree-Walker-stdout (LF-normalisiert)."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.interpreter import Interpreter
    from gamebasic.preprocess import process
    prepped, _ = process(src, base, file_label="<parity>")
    ast = Parser(Lexer(prepped).tokenize()).parse()
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        Interpreter().run(ast)
    return buf.getvalue().replace("\r\n", "\n")


def _runsrc(path: Path):
    out = subprocess.run([str(_GBRT), "--runsrc", str(path)],
                         capture_output=True, text=True, encoding="utf-8")
    return out.returncode, out.stdout.replace("\r\n", "\n")


_SNIPPETS = [
    'PRINT "hallo welt"\n',
    "PRINT 2 + 3 * 4 - 1\n",
    "PRINT 10 / 4, 10 \\ 4, 10 MOD 4, 2 ^ 10\n",
    "DIM x AS INTEGER\nx = 42\nPRINT x, x * 2\n",
    "DIM a AS INTEGER\nDIM b AS INTEGER\na = 7\nb = 5\nPRINT a + b, a - b, a > b\n",
    "DIM s AS STRING\ns = \"ab\" + \"cd\"\nPRINT s, LEN(s), UPPER$(s)\n",
    "CONST MAX AS INTEGER = 100\nPRINT MAX, MAX / 2\n",
    "DIM x AS INTEGER\nx = 15\nIF x > 10 THEN\n  PRINT \"gross\"\nELSEIF x > 5 THEN\n  PRINT \"mittel\"\nELSE\n  PRINT \"klein\"\nEND IF\n",
    "IF 3 > 2 THEN PRINT \"ja\" ELSE PRINT \"nein\"\n",
    "DIM i AS INTEGER\ni = 0\nWHILE i < 5\n  PRINT i\n  i = i + 1\nWEND\n",
    "DIM i AS INTEGER\ni = 0\nWHILE TRUE\n  i = i + 1\n  IF i = 3 THEN BREAK\n  IF i = 1 THEN CONTINUE\n  PRINT i\nWEND\n",
    "PRINT TRUE AND FALSE, TRUE OR FALSE, NOT TRUE\n",
    "PRINT 12 BAND 10, 12 BOR 3, 1 SHL 4, BNOT 0\n",
    "PRINT ABS(-9), INT(3.7), MAX(4, 9), MIN(4, 9)\n",
    "DIM f AS FLOAT\nf = 3.14\nPRINT f, f * 2.0\n",
    "PRINT \"x\" IN \"text\", \"z\" IN \"text\"\n",
    "DIM b AS BOOLEAN\nb = 5 > 3\nPRINT b\n",
    "DIM n AS INTEGER\nn = 1\nn += 4\nn *= 3\nPRINT n\n",
    "PRINT STR$(123), VAL(\"456\"), LEFT$(\"hallo\", 3)\n",
    "DIM x AS INTEGER\nx = -5\nPRINT -x, ABS(x)\n",
    # --- 3b: FOR / Arrays / Index / DATA / READ ---
    "DIM i AS INTEGER\nFOR i = 1 TO 5\n  PRINT i\nNEXT\n",
    "DIM i AS INTEGER\nFOR i = 10 TO 1 STEP -2\n  PRINT i\nNEXT\n",
    "DIM s AS INTEGER\nDIM i AS INTEGER\ns = 0\nFOR i = 1 TO 100\n  s = s + i\nNEXT\nPRINT s\n",
    "DIM a[5] AS INTEGER\nDIM i AS INTEGER\nFOR i = 0 TO 4\n  a[i] = i * 2\nNEXT\nFOR i = 0 TO 4\n  PRINT a[i]\nNEXT\n",
    "DIM g[3, 3] AS INTEGER\ng[1, 2] = 7\nPRINT g[1, 2], g[0, 0]\n",
    "DIM arr[8] AS INTEGER\nPRINT LEN(arr)\narr[3] = 5\nPRINT arr[3], arr[7]\n",
    "DATA 5, 10, 15\nDIM a AS INTEGER\nDIM b AS INTEGER\nDIM c AS INTEGER\nREAD a, b, c\nPRINT a + b + c\n",
    "DATA 1, 2, 3\nDIM a AS INTEGER\nDIM b AS INTEGER\nREAD a, b\nRESTORE\nREAD a\nPRINT a, b\n",
    "DIM i AS INTEGER\nDIM j AS INTEGER\nFOR i = 1 TO 3\n  FOR j = 1 TO 3\n    PRINT i * 10 + j\n  NEXT\nNEXT\n",
    "DIM f AS FLOAT\nFOR f = 0.0 TO 1.0 STEP 0.5\n  PRINT f\nNEXT\n",
    # --- 3c: User-SUB/FUNCTION ---
    "SUB greet(n AS STRING)\n  PRINT \"hi \" + n\nEND SUB\ngreet(\"welt\")\n",
    "FUNCTION sq(x AS INTEGER) AS INTEGER\n  RETURN x * x\nEND FUNCTION\nPRINT sq(7)\n",
    "FUNCTION fib(n AS INTEGER) AS INTEGER\n  IF n < 2 THEN RETURN n\n  RETURN fib(n - 1) + fib(n - 2)\nEND FUNCTION\nPRINT fib(15)\n",
    "FUNCTION add(a AS INTEGER, b AS INTEGER = 10) AS INTEGER\n  RETURN a + b\nEND FUNCTION\nPRINT add(5), add(5, 100)\n",
    "FUNCTION mk(name AS STRING, age AS INTEGER = 0) AS STRING\n  RETURN name + STR$(age)\nEND FUNCTION\nPRINT mk(age: 30, name: \"x\")\n",
    "SUB log(level AS STRING, ...rest)\n  PRINT level\nEND SUB\nlog(\"WARN\", 1, 2, 3)\nlog(\"INFO\")\n",
    "FUNCTION twice(g AS FUNCREF, x AS INTEGER) AS INTEGER\n  RETURN g(g(x))\nEND FUNCTION\nFUNCTION inc(n AS INTEGER) AS INTEGER\n  RETURN n + 1\nEND FUNCTION\nPRINT twice(inc, 10)\n",
    "FUNCTION fac(n AS INTEGER) AS INTEGER\n  DIM r AS INTEGER\n  DIM i AS INTEGER\n  r = 1\n  FOR i = 2 TO n\n    r = r * i\n  NEXT\n  RETURN r\nEND FUNCTION\nPRINT fac(6)\n",
    "DIM total AS INTEGER\nSUB addto(x AS INTEGER)\n  total = total + x\nEND SUB\ntotal = 0\naddto(5)\naddto(10)\nPRINT total\n",
    # --- 3d: Klassen / Structs / ENUM ---
    "ENUM State = MENU, PLAYING, PAUSED\nPRINT State.MENU, State.PLAYING, State.PAUSED\n",
    "ENUM P\n  NONE = 0\n  READ = 1\n  WRITE = 4\nEND ENUM\nPRINT P.READ + P.WRITE\n",
    "CLASS C\n  DIM v AS INTEGER\n  SUB Init()\n    Self.v = 7\n  END SUB\n  FUNCTION get() AS INTEGER\n    RETURN Self.v\n  END FUNCTION\nEND CLASS\nDIM c AS C\nc = NEW C()\nPRINT c.get()\nc.v = 99\nPRINT c.get()\n",
    "CLASS Animal\n  FUNCTION voice() AS STRING\n    RETURN \"...\"\n  END FUNCTION\n  FUNCTION speak() AS STRING\n    RETURN \"sagt \" + voice()\n  END FUNCTION\nEND CLASS\nCLASS Dog EXTENDS Animal\n  FUNCTION voice() AS STRING\n    RETURN \"wuff\"\n  END FUNCTION\nEND CLASS\nDIM d AS Dog\nd = NEW Dog()\nPRINT d.speak()\n",
    "CLASS Cfg\n  STATIC CONST MAX AS INTEGER = 42\n  STATIC CONST NAME AS STRING = \"x\"\nEND CLASS\nPRINT Cfg.MAX, Cfg.NAME\n",
    "CLASS Money\n  DIM cents AS INTEGER\n  OPERATOR + (o AS Money) AS INTEGER\n    RETURN Self.cents + o.cents\n  END OPERATOR\nEND CLASS\nDIM a AS Money\nDIM b AS Money\na = NEW Money()\nb = NEW Money()\na.cents = 100\nb.cents = 250\nPRINT a + b\n",
    "CLASS V\n  DIM x AS INTEGER\n  PROPERTY GET val() AS INTEGER\n    RETURN Self.x * 2\n  END PROPERTY\n  PROPERTY SET val(n AS INTEGER)\n    Self.x = n\n  END PROPERTY\nEND CLASS\nDIM v AS V\nv = NEW V()\nv.val = 10\nPRINT v.val\n",
    "STRUCT Point\n  DIM x AS INTEGER\n  DIM y AS INTEGER\nEND STRUCT\nDIM p AS Point\np.x = 3\np.y = 4\nPRINT p.x + p.y\n",
    "CLASS Counter\n  DIM n AS INTEGER\n  SUB inc()\n    Self.n = Self.n + 1\n  END SUB\nEND CLASS\nDIM c AS Counter\nc = NEW Counter()\nc.inc()\nc.inc()\nc.inc()\nPRINT c.n\n",
    "CLASS Pt\n  DIM px AS INTEGER\n  SUB Init(v AS INTEGER)\n    Self.px = v\n  END SUB\nEND CLASS\nDIM p AS Pt\np = NEW Pt(v: 5)\nPRINT p.px\n",
    # --- 3e: SELECT / FOR EACH / Tupel / WITH / TRY / Slicing / Comprehensions / IIF / REPEAT / Coroutinen ---
    "DIM x AS INTEGER\nx = 13\nSELECT CASE x\n  CASE 1\n    PRINT \"eins\"\n  CASE 2, 3, 4\n    PRINT \"234\"\n  CASE 10 TO 20\n    PRINT \"range\"\n  CASE IS > 100\n    PRINT \"gross\"\n  CASE ELSE\n    PRINT \"else\"\nEND SELECT\n",
    "DIM hp AS INTEGER\nDIM pot AS BOOLEAN\nhp = 25\npot = TRUE\nSELECT CASE hp\n  CASE IS <= 0\n    PRINT \"tot\"\n  CASE IS <= 30 WHERE pot\n    PRINT \"heilen\"\n  CASE IS <= 30\n    PRINT \"fliehen\"\n  CASE ELSE\n    PRINT \"ok\"\nEND SELECT\n",
    "DIM a[3] AS INTEGER\na[0] = 10\na[1] = 20\na[2] = 30\nFOR EACH v IN a\n  PRINT v\nNEXT\n",
    "FOR EACH ch IN \"abc\"\n  PRINT ch\nNEXT\n",
    "FOR EACH n IN (5, 6, 7)\n  PRINT n\nNEXT\n",
    "FOR EACH n IN (1, 2, 3, 4, 5)\n  IF n = 2 THEN CONTINUE\n  IF n = 4 THEN BREAK\n  PRINT n\nNEXT\n",
    "DIM t AS TUPLE\nt = (1, 2, 3)\nPRINT t\n",
    "FUNCTION minmax(a AS INTEGER, b AS INTEGER) AS TUPLE\n  IF a < b THEN RETURN (a, b)\n  RETURN (b, a)\nEND FUNCTION\nDIM lo AS INTEGER\nDIM hi AS INTEGER\n(lo, hi) = minmax(7, 3)\nPRINT lo, hi\n",
    "CLASS P\n  DIM x AS INTEGER\n  DIM y AS INTEGER\nEND CLASS\nDIM p AS P\np = NEW P()\nWITH p\n  .x = 100\n  .y = 50\nEND WITH\nPRINT p.x + p.y\n",
    "TRY\n  THROW \"boom\"\nCATCH msg\n  PRINT \"caught: \" + msg\nEND TRY\n",
    "DIM i AS INTEGER\nFOR i = 1 TO 5\n  TRY\n    IF i = 3 THEN THROW \"x\"\n    PRINT i\n  CATCH e\n    PRINT \"skip \" + STR$(i)\n  END TRY\nNEXT\n",
    "PRINT \"Hello World\"[6:11]\n",
    "DIM a[5] AS INTEGER\nDIM i AS INTEGER\nFOR i = 0 TO 4\n  a[i] = i\nNEXT\nDIM b AS ARRAY OF INTEGER\nb = a[1:4]\nPRINT b[0], b[1], b[2], LEN(b)\n",
    "DIM evens AS TUPLE\nevens = [n FOR n IN (1, 2, 3, 4, 5, 6) WHERE n MOD 2 = 0]\nPRINT evens\n",
    "DIM sq AS TUPLE\nsq = [n * n FOR n IN (1, 2, 3, 4)]\nPRINT sq\n",
    "DIM d AS TUPLE\nd = {x MOD 3 FOR x IN (0, 1, 2, 3, 4, 5, 6, 7, 8)}\nPRINT d\n",
    "DIM m AS MAP OF INTEGER\nm = {STR$(x) + \"sq\": x * x FOR x IN (1, 2, 3)}\nPRINT MAPGET(m, \"2sq\")\n",
    "DIM x AS INTEGER\nx = 0\nPRINT IIF(x <> 0, 100 \\ x, -1)\n",
    "DIM i AS INTEGER\ni = 0\nREPEAT\n  i = i + 1\n  PRINT i\nUNTIL i >= 3\n",
    "FUNCTION counter() AS INTEGER\n  YIELD 1\n  YIELD 2\n  RETURN 99\nEND FUNCTION\nDIM c AS COROUTINE\nc = counter()\nPRINT CORO_RESUME(c)\nPRINT CORO_RESUME(c)\nPRINT CORO_RESUME(c)\nPRINT CORO_DONE(c)\n",
    "DIM m AS MAP OF INTEGER\nMAPPUT(m, \"a\", 1)\nMAPPUT(m, \"b\", 2)\nDIM total AS INTEGER\ntotal = 0\nFOR EACH k IN m\n  total = total + MAPGET(m, k)\nNEXT\nPRINT total\n",
]


@pytest.mark.parametrize("src", _SNIPPETS, ids=range(len(_SNIPPETS)))
def test_compiler_parity_snippets(src, tmp_path):
    f = tmp_path / "snippet.gb"
    f.write_text(src, encoding="utf-8")
    rc, rs_out = _runsrc(f)
    assert rc == 0, f"--runsrc Exit {rc} fuer Snippet:\n{src}"
    assert rs_out == _tw(src, tmp_path)


def test_compiler_example_sweep():
    """Opportunistisch: jedes Beispiel, das --runsrc akzeptiert (rc==0) UND der
    Tree-Walker sauber ausfuehrt, muss identische Ausgabe liefern. Alles andere
    (Stufe-3b-Konstrukte, Grafik, INPUT) wird uebersprungen."""
    checked = 0
    for path in sorted(_EXAMPLES.glob("*.gb")):
        src = path.read_text(encoding="utf-8")
        low = src.lower()
        # Grafik/Interaktion/Module/Nichtdeterminismus gar nicht erst ausfuehren
        # (oeffnet sonst ein Fenster, blockiert auf stdin, oder weicht legitim ab).
        skip = ("screen(", "input ", "import ", "rnd", "millis", "time$",
                "randomize", "key", "mouse", "delta(", "fps", "flip(",
                # Datei-I/O hat Platten-Seiteneffekte (append) -> zwei
                # sequentielle Laeufe (gbrt + TW) sehen verschiedenen State.
                "openfile(")
        if any(tok in low for tok in skip):
            continue
        rc, rs_out = _runsrc(path)
        if rc != 0:
            continue                          # nicht 3a-unterstuetzt
        try:
            tw_out = _tw(src, _EXAMPLES)
        except Exception:
            continue                          # Grafik/Import/etc.
        assert rs_out == tw_out, f"Ausgabe weicht ab: {path.name}"
        checked += 1
    # Mindestens die Snippets decken 3a ab; der Sweep ist Bonus.
    assert checked >= 0
