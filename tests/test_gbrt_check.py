"""Stufe B, Phase 4-Vorbereitung: `gbrt --check` Diagnostik.

Verifiziert, dass gbrt gueltigen Code sauber meldet ([]) und die strukturellen
Compile-/Syntax-Fehler MIT Zeilennummer liefert (Voraussetzung, um die Editor/
LSP-Diagnostik vom Python-Compiler auf gbrt umzustellen). Skippt ohne gbrt.
"""
import json
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
pytestmark = pytest.mark.skipif(_GBRT is None, reason="gbrt nicht gebaut")


def _check(tmp_path, src):
    f = tmp_path / "c.gb"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_GBRT), "--check", str(f)],
                       capture_output=True, text=True, timeout=30)
    assert r.returncode == 0, r.stderr      # Exit 0 auch bei Diagnosen
    return json.loads(r.stdout)


def test_clean_program_no_diagnostics(tmp_path):
    assert _check(tmp_path, 'DIM x AS INTEGER\nx = 5\nPRINT x\n') == []


def test_parse_error_has_line(tmp_path):
    d = _check(tmp_path, 'PRINT "a"\nFOR i = 1 TO\n')
    assert len(d) == 1 and d[0]["phase"] == "parse" and d[0]["line"] == 2


def test_compile_error_break_outside_loop_has_line(tmp_path):
    d = _check(tmp_path, 'PRINT "a"\nBREAK\n')
    assert len(d) == 1
    assert d[0]["phase"] == "compile" and d[0]["line"] == 2
    assert "BREAK" in d[0]["message"]


def test_compile_error_return_outside_function_has_line(tmp_path):
    d = _check(tmp_path, 'PRINT 1\nRETURN 5\n')
    assert d and d[0]["phase"] == "compile" and d[0]["line"] == 2


def test_all_examples_check_clean():
    """Kein Fehlalarm: jedes gueltige Beispiel meldet [] (Null-False-Positive)."""
    bad = []
    for f in sorted(_EXAMPLES.glob("*.gb")):
        if "_smoketest" in f.name:
            continue
        r = subprocess.run([str(_GBRT), "--check", str(f)],
                           capture_output=True, text=True, timeout=30)
        if r.stdout.strip() != "[]":
            bad.append((f.name, r.stdout.strip()))
    assert not bad, f"Fehlalarme bei gueltigem Code: {bad}"
