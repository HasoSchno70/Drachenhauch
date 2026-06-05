"""Stufe B, Phase 3: Quell-Zeilen-Tracking + `gbrt profile`.

Verifiziert, dass gbrts Compiler echte Zeilennummern emittiert (Voraussetzung
fuer Profiler/Debugger/Laufzeitfehler-Zeilen) und dass `gbrt profile` pro Zeile
Count + Zeit liefert. Skippt, wenn gbrt nicht gebaut ist.
"""
import json
import os
import subprocess
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
pytestmark = pytest.mark.skipif(_GBRT is None, reason="gbrt nicht gebaut")

_PROG = (
    'DIM total AS INTEGER\n'      # Zeile 1
    'total = 0\n'                 # 2
    'FOR i = 1 TO 1000\n'         # 3
    '    total = total + i\n'     # 4
    'NEXT\n'                      # 5
    'PRINT "sum=" + STR$(total)\n'  # 6
)


def _write(tmp_path, src):
    f = tmp_path / "p.gb"
    f.write_text(src, encoding="utf-8")
    return f


def test_compiler_emits_real_lines(tmp_path):
    """`gbrt --dumpbc` -> main.lines ist NICHT mehr alles Null."""
    f = _write(tmp_path, _PROG)
    r = subprocess.run([str(_GBRT), "--dumpbc", str(f)],
                       capture_output=True, text=True, timeout=30)
    assert r.returncode == 0, r.stderr
    lines = json.loads(r.stdout)["main"]["lines"]
    assert any(l != 0 for l in lines)
    assert set(lines) <= {0, 1, 2, 3, 4, 6}     # echte Quell-Zeilen


def test_profile_line_counts(tmp_path):
    """`gbrt profile` -> Schleifen-Zeilen haben die erwarteten Counts."""
    f = _write(tmp_path, _PROG)
    r = subprocess.run([str(_GBRT), "profile", str(f)],
                       capture_output=True, text=True, timeout=30)
    assert r.returncode == 0, r.stderr
    d = json.loads(r.stdout)
    assert d["output"].replace("\r\n", "\n") == "sum=500500\n"
    assert d["total_time"] >= 0.0 and d["stopped"] is False
    counts = {ln["line"]: ln["count"] for ln in d["lines"]}
    assert counts.get(3) == 1001     # FOR-Test: 1000 Durchlaeufe + 1 Abbruch
    assert counts.get(4) == 1000     # Schleifenkoerper


def test_runtime_error_has_line(tmp_path):
    """Bonus: gbrt-Laufzeitfehler tragen jetzt die Quell-Zeile."""
    f = _write(tmp_path, 'PRINT "a"\nPRINT 1 \\ 0\n')
    r = subprocess.run([str(_GBRT), "run", str(f)],
                       capture_output=True, text=True, timeout=30)
    assert r.returncode != 0
    assert ":2:" in r.stderr      # Division-durch-0 in Zeile 2
