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

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/variable_wie_builtin.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

from drachenhauch.errors import DHRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


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
