"""Eine Variable, die wie ein Builtin heisst -- `deg = DEG(x)`, `len = LEN(s)`.

In BASIC ist das Alltag, und bis 2026-09-04 brach es zur Laufzeit ab:
"'deg' ist eine Variable vom Typ FLOAT und kann nicht wie eine Funktion
aufgerufen werden" -- waehrend `--check` schwieg. Gefunden am Beispiel 145
(`DIM deg AS FLOAT : deg = DEG(winkel)`), das deshalb umbenannt wurde.

Die Regel jetzt: hat die Variable einen bekannten Typ, der kein FUNCREF
ist, meint `NAME(...)` den Builtin -- eine FLOAT laesst sich nicht
aufrufen. Nur eine FUNCREF-Variable dieses Namens ruft weiter die Variable
(und der Sonderfall, dass der Compiler den Typ nicht kennt, weil der Name
zweimal mit verschiedenem Typ deklariert ist). Die FOR-EACH-Laufvariable
verdeckt nicht -- sie laeuft nicht ueber den Variablen-Weg des Aufrufs.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

from drachenhauch.errors import DHRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def test_globale_variable_mit_builtin_namen(run_gb):
    out = run_gb("DIM deg AS FLOAT\ndeg = DEG(3.141592653589793)\nPRINT INT(deg + 0.5)\n"
                 'DIM len AS INTEGER\nlen = LEN("abc")\nPRINT len\n')
    assert _lines(out) == ["180", "3"]


def test_lokale_variable_mit_builtin_namen(run_gb):
    out = run_gb("SUB t()\n    DIM sqr AS FLOAT\n    sqr = SQR(16.0)\n    PRINT sqr\nEND SUB\nt()\n")
    assert _lines(out) == ["4.0"]


def test_parameter_mit_builtin_namen(run_gb):
    out = run_gb("FUNCTION f(abs AS INTEGER) AS INTEGER\n    RETURN ABS(abs) + abs\nEND FUNCTION\n"
                 "PRINT f(-3)\n")
    assert _lines(out) == ["0"]


def test_eine_funcref_dieses_namens_ruft_weiter_die_variable(run_gb):
    """Der Grund, warum es nicht einfach 'immer der Builtin' heisst."""
    out = run_gb("FUNCTION quadrat(x AS INTEGER) AS INTEGER\n    RETURN x * x\nEND FUNCTION\n"
                 "DIM abs AS FUNCREF\nabs = quadrat\nPRINT abs(-5)\n")
    assert _lines(out) == ["25"]


def test_eine_leere_funcref_dieses_namens_gibt_die_klare_meldung(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('DIM len AS FUNCREF\nPRINT LEN("ab")\n')
    m = str(e.value)
    assert "kann nicht wie eine Funktion aufgerufen werden" in m
    assert "FUNCREF" in m, "die Meldung sagt jetzt, welche Variablen ueberhaupt verdecken"


def test_die_for_each_laufvariable_verdeckt_nicht(run_gb):
    out = run_gb('FOR EACH len IN (1, 2)\n    PRINT LEN("ab")\nNEXT\n')
    assert _lines(out) == ["2", "2"]


def _find_dhrt():
    root = Path(__file__).resolve().parent.parent
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((root / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (root / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


def test_check_bleibt_still(tmp_path):
    """Kein Fehlalarm fuer den Alltagsfall -- der Sweep ueber alle Beispiele
    (test_dhrt_check) haelt das fuer den Bestand fest, hier fuer den Fall."""
    exe = _find_dhrt()
    if exe is None:
        pytest.skip("dhrt nicht gebaut")
    f = tmp_path / "d.dh"
    f.write_text("DIM deg AS FLOAT\ndeg = DEG(1.0)\nPRINT deg\n", encoding="utf-8")
    r = subprocess.run([str(exe), "--check", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=60)
    assert json.loads(r.stdout.strip() or "[]") == [], r.stdout
