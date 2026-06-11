"""End-to-End-Test: `gbrt run <datei.gb>` (Stufe 5, eigenstaendig ohne Python).

Stufe 5 macht gbrt eigenstaendig: `gbrt run datei.gb` preprocesst (IMPORT),
lext, parst, kompiliert und fuehrt aus -- ALLES in Rust, ohne Python. Wie
`gbrun.py` wird ins Verzeichnis der Datei gewechselt (chdir), damit relative
Pfade (IMPORT + Laufzeit-Asset/-Datei) stimmen.

Verifikation: stdout von `gbrt run` gegen erwartete Ausgaben. Deckt explizit
den chdir-Effekt ab (relativer Laufzeit-Datei-Zugriff) und den
`gbrt <datei.gb>`-Auto-Detect (ohne `run`). (Hiess historisch "Parity" --
der Vergleichspartner Python-Tree-Walker ist seit Stufe B geloescht.)
"""
import contextlib
import io
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
pytestmark = pytest.mark.skipif(
    _GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")


def _gbrt(argv) -> tuple:
    out = subprocess.run([str(_GBRT), *argv],
                         capture_output=True, text=True, encoding="utf-8")
    return out.returncode, out.stdout.replace("\r\n", "\n")


def _write(d: Path, name: str, content: str) -> Path:
    p = d / name
    p.write_text(content, encoding="utf-8")
    return p


def test_run_with_relative_file_and_imports(tmp_path):
    # lib.gb (Quellcode-IMPORT), vec2 (Modul-IMPORT), data.txt (relativer
    # Laufzeit-Zugriff -- beweist chdir).
    _write(tmp_path, "lib.gb",
           "FUNCTION tri(n AS INTEGER) AS INTEGER\n  RETURN n * (n + 1) \\ 2\nEND FUNCTION\n")
    _write(tmp_path, "data.txt", "hallo\nwelt\n")
    main = _write(tmp_path, "main.gb",
                  'IMPORT "lib.gb"\nIMPORT "vec2"\n'
                  'DIM v AS VEC2\nv = VEC2_NEW(3.0, 4.0)\n'
                  'DIM f AS FILE\nf = OpenFile("data.txt", "r")\n'
                  'PRINT ReadLine(f)\nCloseFile(f)\n'
                  'PRINT tri(10)\nPRINT VEC2_LENGTH(v)\n')
    rc, out = _gbrt(["run", str(main)])
    assert rc == 0, f"gbrt run Exit {rc}"
    # Golden (Stufe B): ReadLine="hallo", tri(10)=55, VEC2_LENGTH(3,4)=5.0.
    assert out == "hallo\n55\n5.0\n"


def test_bare_gb_path_autodetect(tmp_path):
    # `gbrt <datei.gb>` ohne `run` wird wie `run` behandelt (chdir + Quelltext).
    _write(tmp_path, "data.txt", "zeile1\n")
    main = _write(tmp_path, "prog.gb",
                  'DIM f AS FILE\nf = OpenFile("data.txt", "r")\n'
                  'PRINT ReadLine(f)\nCloseFile(f)\nPRINT 6 * 7\n')
    rc, out = _gbrt([str(main)])
    assert rc == 0, f"gbrt <datei.gb> Exit {rc}"
    # Golden (Stufe B): ReadLine="zeile1", 6*7=42.
    assert out == "zeile1\n42\n"
