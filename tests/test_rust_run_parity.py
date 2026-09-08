"""End-to-End-Test: `dhrt run <datei.dh>` (Stufe 5, eigenstaendig ohne Python).

Stufe 5 macht dhrt eigenstaendig: `dhrt run datei.dh` preprocesst (IMPORT),
lext, parst, kompiliert und fuehrt aus -- ALLES in Rust, ohne Python. Wie
`dhrun.py` wird ins Verzeichnis der Datei gewechselt (chdir), damit relative
Pfade (IMPORT + Laufzeit-Asset/-Datei) stimmen.

Verifikation: stdout von `dhrt run` gegen erwartete Ausgaben. Deckt explizit
den chdir-Effekt ab (relativer Laufzeit-Datei-Zugriff) und den
`dhrt <datei.dh>`-Auto-Detect (ohne `run`). (Hiess historisch "Parity" --
der Vergleichspartner Python-Tree-Walker ist seit Stufe B geloescht.)

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/rust_run_parity.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import contextlib
import io
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


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


def _dhrt(argv) -> tuple:
    out = subprocess.run([str(_DHRT), *argv],
                         capture_output=True, text=True, encoding="utf-8")
    return out.returncode, out.stdout.replace("\r\n", "\n")


def _write(d: Path, name: str, content: str) -> Path:
    p = d / name
    p.write_text(content, encoding="utf-8")
    return p


def test_bare_gb_path_autodetect(tmp_path):
    # `dhrt <datei.dh>` ohne `run` wird wie `run` behandelt (chdir + Quelltext).
    _write(tmp_path, "data.txt", "zeile1\n")
    main = _write(tmp_path, "prog.dh",
                  'DIM f AS FILE\nf = OpenFile("data.txt", "r")\n'
                  'PRINT ReadLine(f)\nCloseFile(f)\nPRINT 6 * 7\n')
    rc, out = _dhrt([str(main)])
    assert rc == 0, f"dhrt <datei.dh> Exit {rc}"
    # Golden (Stufe B): ReadLine="zeile1", 6*7=42.
    assert out == "zeile1\n42\n"
