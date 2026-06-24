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
    """Kein Fehlalarm: jedes gueltige Beispiel meldet keine *Errors*
    (Null-False-Positive). Warnungen (z.B. fehlendes Hardware-Modul im
    Default-Build, siehe Hardware-Beispiele 35-38) sind erlaubt -- sie sind
    keine Fehler, sondern ein bewusster Hinweis."""
    bad = []
    for f in sorted(_EXAMPLES.glob("*.gb")):
        if "_smoketest" in f.name:
            continue
        r = subprocess.run([str(_GBRT), "--check", str(f)],
                           capture_output=True, text=True, timeout=30)
        diags = json.loads(r.stdout or "[]")
        errors = [d for d in diags if d.get("severity") != "warning"]
        if errors:
            bad.append((f.name, errors))
    assert not bad, f"Fehlalarme bei gueltigem Code: {bad}"


def test_hardware_import_warns_at_import(tmp_path):
    """E1: `IMPORT "wifi"` (serial/usb/bt analog) wird im Default-Build (ohne
    --hardware) schon beim IMPORT als Warnung gemeldet -- nicht erst beim ersten
    Funktionsaufruf zur Laufzeit. Ein Hardware-Build meldet stattdessen nichts;
    beide Faelle sind gueltig."""
    d = _check(tmp_path, 'IMPORT "wifi"\nPRINT 1\n')
    if d:  # Default-Build: genau eine Warnung auf der IMPORT-Zeile
        assert len(d) == 1, d
        w = d[0]
        assert w["severity"] == "warning"
        assert w["line"] == 1
        assert "wifi" in w["message"].lower()
        assert "--hardware" in w["message"]


def test_unknown_builtin_warns(tmp_path):
    """G1 (systemisch): Aufruf eines Builtins, das gbrt nicht kennt (Tippfehler
    oder nur-Tree-Walker wie frueher FLT), wird schon von --check als Warnung
    gemeldet -- nicht erst zur Laufzeit."""
    d = _check(tmp_path, 'DIM x AS INTEGER\nx = NOTAREALBUILTIN(5)\n')
    assert len(d) == 1, d
    w = d[0]
    assert w["severity"] == "warning"
    assert w["phase"] == "compile"
    assert w["line"] == 2
    assert "NOTAREALBUILTIN" in w["message"]


def test_known_builtin_no_warning(tmp_path):
    """Echte Builtins (inkl. FLT) loesen KEINE Warnung aus."""
    assert _check(tmp_path, 'DIM x AS FLOAT\nx = FLT(3)\nPRINT INT(x)\n') == []
