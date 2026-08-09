"""Test: `gbrt --export <datei.gb>` -- Selbst-Export einer eigenstaendigen Exe.

gbrt kompiliert den Quelltext selbst (ohne Python) zu `.gbc` und haengt den
Payload an eine Kopie der eigenen Runtime-Exe. Die erzeugte Exe muss ohne
Python laufen und denselben Output wie der Python-Tree-Walker liefern.
"""
import contextlib
import io
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    base = _ROOT / "rust" / "drachenhauch_runtime" / "target"
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


_GBRT = _find_gbrt()
pytestmark = pytest.mark.skipif(
    _GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")


def test_export_runs_standalone(tmp_path):
    main = tmp_path / "prog.gb"
    main.write_text(
        "FUNCTION fib(n AS INTEGER) AS INTEGER\n"
        "  IF n < 2 THEN RETURN n\n"
        "  RETURN fib(n - 1) + fib(n - 2)\n"
        "END FUNCTION\n"
        "PRINT fib(10)\n"
        "DIM i AS INTEGER\n"
        "FOR i = 1 TO 3\n  PRINT i * i\nNEXT\n",
        encoding="utf-8")
    out_dir = tmp_path / "dist"
    res = subprocess.run([str(_GBRT), "--export", str(main), str(out_dir)],
                         capture_output=True, text=True, encoding="utf-8")
    assert res.returncode == 0, f"--export Exit {res.returncode}: {res.stderr}"

    exe = "prog.exe" if os.name == "nt" else "prog"
    out_exe = out_dir / exe
    assert out_exe.exists(), "exportierte Exe fehlt"
    # Die gebuendelte Exe ist groesser als die nackte Runtime (Payload dran).
    assert out_exe.stat().st_size > _GBRT.stat().st_size

    run = subprocess.run([str(out_exe)], capture_output=True, text=True,
                         encoding="utf-8")
    assert run.returncode == 0, f"Exe Exit {run.returncode}: {run.stderr}"
    # Golden (Stufe B): fib(10)=55, dann i*i fuer i=1..3.
    assert run.stdout.replace("\r\n", "\n") == "55\n1\n4\n9\n"
