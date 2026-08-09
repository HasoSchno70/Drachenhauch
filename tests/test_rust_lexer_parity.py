"""Parity-Test: Rust-Lexer (`dhrt --tokens`) == Python-Lexer.

Erster Wächter der Front-End-Portierung (Lexer -> Parser -> Compiler nach Rust).
Vergleicht den Token-Strom als (Typ, Wert, Zeile) ueber alle Beispiele +
gezielte Sprach-Snippets. Skippt, wenn `dhrt` nicht gebaut ist.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

from gamebasic.lexer import Lexer

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


def _py_tokens(src: str):
    return [[t.type.name, t.value, t.line] for t in Lexer(src).tokenize()]


def _rs_tokens(path: Path):
    out = subprocess.run([str(_DHRT), "--tokens", str(path)],
                         capture_output=True, text=True, encoding="utf-8")
    assert out.returncode == 0, out.stderr
    return [json.loads(line) for line in out.stdout.splitlines() if line.strip()]


def _rs_tokens_src(src: str, tmp_path: Path):
    f = tmp_path / "snippet.gb"
    f.write_text(src, encoding="utf-8")
    return _rs_tokens(f)


# Beispiele, die schon der Python-Lexer nicht akzeptiert, fallen via try/except
# weg -- die wollen wir nicht vergleichen.
def _example_files():
    files = []
    for f in sorted(_EXAMPLES.glob("*.gb")):
        try:
            _py_tokens(f.read_text(encoding="utf-8"))
        except Exception:
            continue
        files.append(f)
    return files


@pytest.mark.parametrize("path", _example_files(), ids=lambda p: p.name)
def test_lexer_parity_examples(path):
    src = path.read_text(encoding="utf-8")
    assert _rs_tokens(path) == _py_tokens(src)


_SNIPPETS = [
    'PRINT "hallo"\n',
    "DIM x AS INTEGER\nx = 5 + 3 * 2\n",
    "DIM s AS STRING\ns = \"a\"\"b\"\n",                 # "" -Escape
    "DIM n AS INTEGER\nn = &HFF + &B1010\n",             # Hex/Binaer (BASIC)
    "DIM n AS INTEGER\nn = 0xFF + 0b1010 + 0xCAFE\n",    # Hex/Binaer (C-Stil)
    "PRINT 0XfF, 0B1, 0, 0.5\n",                         # Gross/klein + Fallback 0 / 0.5
    "PRINT 3.14, 5.0, 100\n",                            # int vs float
    'PRINT f"x={x} y={y:.2f}"\n',                        # f-String + Spec
    'PRINT f"{{nicht}} aber {a+b}"\n',                   # f-String Escapes
    "FOR i = 1 TO 10 STEP 2\nNEXT\n",
    "x += 1\ny -= 2\nz *= 3\nw /= 4\n",                  # Compound-Assign
    "IF a <= b AND c <> d THEN PRINT 1\n",
    "DIM m AS MAP OF INTEGER\n",
    "SUB foo(...rest)\nEND SUB\n",                       # Ellipsis
    "result = call(1, _\n   2, 3)\n",                    # Zeilenfortsetzung _
    "a = (1 +\n     2)\n",                               # implizite Fortsetzung
    "MyVar = OtherVar\n",                                # Idents lowercased
    "x = STR$(5) + CHR$(65)\n",                          # $-Suffix-Builtins
    "' nur ein Kommentar\nREM auch einer\nPRINT 1\n",
]


@pytest.mark.parametrize("src", _SNIPPETS, ids=range(len(_SNIPPETS)))
def test_lexer_parity_snippets(src, tmp_path):
    assert _rs_tokens_src(src, tmp_path) == _py_tokens(src)
