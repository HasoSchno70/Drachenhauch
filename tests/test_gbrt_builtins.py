"""Golden-Tests fuer **gbrt-only**-Builtins.

Seit 2026-06-05 werden neue Builtins nur noch in der nativen Runtime `gbrt`
(Rust) implementiert -- der Python-Tree-Walker (`interpreter.py`) wird nicht
mehr erweitert. Diese Builtins lassen sich daher nicht per TW==gbrt-Paritaet
testen; stattdessen laeuft das Programm durch gbrt und der Output wird gegen
erwartete Literale geprueft (Golden-Test).

Die Quelle wird ueber gbrts EIGENEN Rust-Compiler (`gbrt --runsrc`) kompiliert
und ausgefuehrt -- NICHT ueber die Python-Toolchain. Wichtig: die Python-
`compiler.py` erkennt gbrt-only-Builtins nicht (sie stehen nicht mehr im Python-
`BUILTINS`-Registry) und wuerde sie als User-Calls fehlkompilieren. gbrts Rust-
Compiler emittiert CALL_BUILTIN fuer jeden unbekannten Call -> der gbrt-VM-Pfad
loest sie nativ auf.
"""
import os
import subprocess
import tempfile
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


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


def _gbrt(source: str) -> str:
    """Schreibt `source` in eine temp .gb und fuehrt sie via `gbrt --runsrc`
    (Rust-Frontend: preprocess->lex->parse->compile->VM); stdout mit LF."""
    fd, tmp = tempfile.mkstemp(suffix=".gb")
    os.close(fd)
    Path(tmp).write_text(source, encoding="utf-8")
    try:
        res = subprocess.run([str(_GBRT), "--runsrc", tmp],
                             capture_output=True, text=True, timeout=60)
        return res.stdout.replace("\r\n", "\n")
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass


def _run(source: str) -> str:
    return _gbrt(source).strip()


# --- WP1: Array-Aggregate ---------------------------------------------------

def test_array_sum_avg_min_max_int():
    src = '''DIM a[3] AS INTEGER
a[0]=5 : a[1]=9 : a[2]=1
PRINT ARRAY_SUM(a), ARRAY_AVG(a), ARRAY_MIN(a), ARRAY_MAX(a)'''
    assert _run(src) == "15 5.0 1 9"


def test_array_aggregate_float():
    src = '''DIM f[3] AS FLOAT
f[0]=1.5 : f[1]=2.5 : f[2]=4.0
PRINT ARRAY_SUM(f), ARRAY_MIN(f), ARRAY_MAX(f)
PRINT ARRAY_AVG(f)'''
    assert _run(src) == "8.0 1.5 4.0\n2.6666666666666665"


def test_array_fill_coerces_int_to_float():
    src = '''DIM b[3] AS INTEGER
ARRAY_FILL(b, 7)
PRINT b[0], b[1], b[2]
DIM f[2] AS FLOAT
ARRAY_FILL(f, 9)
PRINT f[0], f[1]'''
    assert _run(src) == "7 7 7\n9.0 9.0"


def test_array_copy_is_independent():
    src = '''DIM a[3] AS INTEGER
a[0]=5 : a[1]=9 : a[2]=1
DIM c AS ARRAY OF INTEGER
c = ARRAY_COPY(a)
c[0] = 100
PRINT a[0], c[0]'''
    assert _run(src) == "5 100"


def test_array_avg_empty_errors():
    # 0-grosses Array -> ARRAY_AVG wirft; "before" kommt noch, "after" nicht.
    src = '''PRINT "before"
DIM a[0] AS INTEGER
PRINT ARRAY_AVG(a)
PRINT "after"'''
    assert _run(src) == "before"
