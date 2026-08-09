"""Tests fuer INPUT-Statement: Prompt-Flush + Lesen ueber stdin.

Wir starten gbrun.py als echter Subprocess (so wie es der Editor macht)
und pipen Eingaben via stdin. Verifiziert end-to-end, dass INPUT in der
Editor-Konsole zuverlaessig funktioniert.
"""
import os
import sys
import subprocess
import textwrap
from pathlib import Path

import pytest


_ROOT = Path(__file__).resolve().parent.parent
_GBRUN = _ROOT / "gbrun.py"


def _gbrt_available() -> bool:
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    return any((_ROOT / "rust" / "drachenhauch_runtime" / "target" / variant / exe).exists()
               for variant in ("release", "debug"))


# gbrun.py ruft intern "gbrt run" auf -- ohne gebauten gbrt bricht der
# Subprozess mit einer "Native Runtime nicht gefunden"-Meldung ab statt das
# GB-Programm auszufuehren. Anders als die run_gb-Fixture (conftest.py)
# nutzt dieses Modul gbrun.py direkt als echten Subprozess (End-to-End-Test
# der Editor-Konsole), daher der eigene Skip hier.
pytestmark = pytest.mark.skipif(not _gbrt_available(), reason="native Runtime 'gbrt' nicht gebaut")


def _run_with_stdin(source: str, stdin: str, tmp_path: Path) -> tuple[int, str]:
    src_file = tmp_path / "prog.gb"
    src_file.write_text(source, encoding="utf-8")
    venv_py = (_ROOT / ".venv" / "Scripts" / "python.exe"
               if os.name == "nt"
               else _ROOT / ".venv" / "bin" / "python")
    if not venv_py.exists():
        venv_py = Path(sys.executable)
    proc = subprocess.run(
        [str(venv_py), str(_GBRUN), str(src_file)],
        input=stdin,
        capture_output=True,
        text=True,
        cwd=str(_ROOT),
        encoding="utf-8",
        errors="replace",
        timeout=15,
    )
    return proc.returncode, proc.stdout


def test_input_string_with_stdin(tmp_path):
    src = textwrap.dedent('''
        DIM name AS STRING
        INPUT "Wie heisst du? ", name
        PRINT "Hallo, " + name + "!"
    ''')
    rc, out = _run_with_stdin(src, "Anna\n", tmp_path)
    assert rc == 0
    # Prompt + User-Antwort in der Ausgabe (unmittelbar zusammen wenn
    # gepipted; entscheidend: 'Hallo, Anna!' steht drin)
    assert "Hallo, Anna!" in out


def test_input_integer_with_stdin(tmp_path):
    src = textwrap.dedent('''
        DIM n AS INTEGER
        INPUT "Zahl? ", n
        PRINT n * 2
    ''')
    rc, out = _run_with_stdin(src, "21\n", tmp_path)
    assert rc == 0
    assert "42" in out


def test_input_prompt_appears_before_blocking(tmp_path):
    """Wichtig fuer Editor-Konsole: Prompt MUSS vor input() erscheinen,
    sonst sieht der User nichts und das Programm haengt scheinbar.

    Wir geben absichtlich ZU WENIG stdin, der Prompt muss aber dennoch
    in stdout sichtbar sein bevor das Programm haengt/exit'd.
    """
    src = textwrap.dedent('''
        DIM x AS INTEGER
        INPUT "Bitte: ", x
        PRINT x
    ''')
    # Leerer stdin -> input() bekommt EOF -> raw="" -> int("") wirft ValueError
    # Was wir hier testen: der Prompt ist im stdout sichtbar.
    rc, out = _run_with_stdin(src, "", tmp_path)
    assert "Bitte:" in out


def test_input_multiple_with_stdin(tmp_path):
    """Mehrfache INPUTs, jede liest eine Zeile."""
    src = textwrap.dedent('''
        DIM a AS INTEGER
        DIM b AS INTEGER
        INPUT "A? ", a
        INPUT "B? ", b
        PRINT a + b
    ''')
    rc, out = _run_with_stdin(src, "10\n5\n", tmp_path)
    assert rc == 0
    assert "15" in out
