"""Parity-Test: Rust-Parser (`dhrt --ast`) == Python-Parser.

Stufe 2 der Front-End-Portierung. Serialisiert den Python-AST kanonisch
(`{"_": NodeName, feld: ...}` aus den Dataclass-Feldern; `.line` ist KEIN
Feld -> faellt raus, also reiner Struktur-Vergleich) und vergleicht gegen
`dhrt --ast`. Skippt, wenn `dhrt` nicht gebaut ist.
"""
import dataclasses
import json
import os
import subprocess
from pathlib import Path

import pytest

from gamebasic.lexer import Lexer
from gamebasic.parser import Parser

_ROOT = Path(__file__).resolve().parent.parent
_EXAMPLES = _ROOT / "examples"


def _find_dhrt():
    base = _ROOT / "rust" / "drachenhauch_runtime" / "target"
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(
    _DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")


def _ser(x):
    """AST -> kanonische JSON-Struktur (nur Dataclass-Felder)."""
    if x is None or isinstance(x, (str, int, float, bool)):
        return x
    if isinstance(x, (list, tuple)):
        return [_ser(e) for e in x]
    if dataclasses.is_dataclass(x):
        d = {"_": type(x).__name__}
        for f in dataclasses.fields(x):
            v = getattr(x, f.name)
            # Param.by_ref haelt in Python das BYREF-Token (oder None) statt
            # eines echten bool -- semantisch aber ein Flag. Rust nutzt bool;
            # fuer den fairen Struktur-Vergleich auf bool normalisieren.
            if f.name == "by_ref":
                v = bool(v)
            d[f.name] = _ser(v)
        return d
    raise TypeError(f"nicht serialisierbar: {type(x)}")


def _py_ast(src: str):
    ast = Parser(Lexer(src).tokenize()).parse()
    return _ser(ast)


def _rs_ast(path: Path):
    out = subprocess.run([str(_DHRT), "--ast", str(path)],
                         capture_output=True, text=True, encoding="utf-8")
    assert out.returncode == 0, out.stderr
    return json.loads(out.stdout)


def _rs_ast_src(src: str, tmp_path: Path):
    f = tmp_path / "snippet.gb"
    f.write_text(src, encoding="utf-8")
    return _rs_ast(f)


def _example_files():
    files = []
    for f in sorted(_EXAMPLES.glob("*.gb")):
        try:
            _py_ast(f.read_text(encoding="utf-8"))
        except Exception:
            continue          # was schon Python nicht parst -> nicht vergleichen
        files.append(f)
    return files


@pytest.mark.parametrize("path", _example_files(), ids=lambda p: p.name)
def test_parser_parity_examples(path):
    src = path.read_text(encoding="utf-8")
    assert _rs_ast(path) == _py_ast(src)


_SNIPPETS = [
    "DIM x AS INTEGER\nx = 5\n",
    "DIM a, b, c AS INTEGER\n",
    "DIM g[10, 20] AS FLOAT\n",
    "CONST MAX AS INTEGER = 100\n",
    "x = a + b * c - d / e MOD f\n",
    "x = a BAND b BOR c SHL 2\n",
    "x = -a ^ 2\n",                                  # unary + power
    "x = NOT a AND b OR c\n",
    "IF a < b THEN PRINT 1 ELSE PRINT 2\n",
    "IF a THEN\n  x = 1\nELSEIF b THEN\n  x = 2\nELSE\n  x = 3\nEND IF\n",
    "FOR i = 1 TO 10 STEP 2\n  PRINT i\nNEXT i\n",
    "FOR EACH e IN items\n  PRINT e\nNEXT\n",
    "WHILE x > 0\n  x = x - 1\nWEND\n",
    "REPEAT\n  x = x + 1\nUNTIL x >= 10\n",
    "SELECT CASE n\nCASE 1, 2, 3\n  PRINT 1\nCASE 10 TO 20\n  PRINT 2\n"
    "CASE IS >= 100 WHERE flag\n  PRINT 3\nCASE ELSE\n  PRINT 4\nEND SELECT\n",
    "SUB greet(name AS STRING, BYREF n AS INTEGER, ...rest)\n  PRINT name\nEND SUB\n",
    "FUNCTION add(a AS INTEGER, b AS INTEGER = 5) AS INTEGER\n  RETURN a + b\nEND FUNCTION\n",
    "CLASS Player EXTENDS Entity\n  DIM hp AS INTEGER\n  STATIC CONST MAX AS INTEGER = 100\n"
    "  SUB Init()\n    Self.hp = 1\n  END SUB\n"
    "  PROPERTY GET hp() AS INTEGER\n    RETURN Self.hp\n  END PROPERTY\n"
    "  OPERATOR + (other AS Player) AS Player\n    RETURN Self\n  END OPERATOR\n"
    "END CLASS\n",
    "STRUCT Point\n  DIM x AS INTEGER\n  DIM y AS INTEGER\nEND STRUCT\n",
    "ENUM State = MENU, PLAYING, PAUSED\n",
    "ENUM Perm\n  NONE = 0\n  READ = 1\n  WRITE\nEND ENUM\n",
    "p = NEW Player()\nq = NEW Vec(1, 2)\n",
    "(lo, hi) = minmax(7, 3)\n",
    "(p.x, arr[i]) = pair()\n",
    "t = (1, 2, 3)\n",
    "x = arr[1:5]\ny = arr[:3]\nz = arr[2:]\nw = s[:]\n",
    "x = obj.field.sub[i].method(1, 2)\n",
    "evens = [n FOR n IN nums WHERE n MOD 2 = 0]\n",
    "m = {k: v FOR k IN keys}\ns = {x MOD 3 FOR x IN nums}\n",
    "x = IIF(a > 0, 1, -1)\n",
    "result = func(1, name: \"x\", flag: TRUE)\n",
    "WITH player\n  .x = 1\n  .y = .x + 2\nEND WITH\n",
    "TRY\n  risky()\nCATCH e\n  PRINT e\nEND TRY\n",
    "THROW \"boom\"\n",
    "DATA 1, -2, 3.5, \"hi\", TRUE\nREAD a, b, c\nRESTORE\n",
    "x += 1\narr[i] -= 2\nobj.f *= 3\n",
    "x = a IN (1, 2, 3)\n",
    "FUNCTION gen() AS INTEGER\n  YIELD 1\n  x = YIELD 2\n  RETURN 9\nEND FUNCTION\n",
    "PRINT f\"x={x} y={y:.2f}\"\n",
    "x = 1 : y = 2 : z = 3\n",
]


@pytest.mark.parametrize("src", _SNIPPETS, ids=range(len(_SNIPPETS)))
def test_parser_parity_snippets(src, tmp_path):
    assert _rs_ast_src(src, tmp_path) == _py_ast(src)
