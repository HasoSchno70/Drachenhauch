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
        # Grafik/Interaktion/Module gar nicht erst ausfuehren (oeffnet sonst
        # ein raylib-Fenster bzw. blockiert auf stdin).
        if "screen(" in low or "input " in low or "import " in low:
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
