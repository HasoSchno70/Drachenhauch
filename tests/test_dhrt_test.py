"""`dhrt test` mit Pruefsammlungen (`*.dhtest`) -- Weg D des Python-Abbaus.

Die Golden-Tests der Sprache ziehen nach und nach aus pytest in Sammlungen
unter `tests/pruef/` um (eine Datei, viele Faelle, erwartete Ausgabe im
Klartext). Bis pytest ganz faellt, ist DIESER Test der Anker, der sie in der
CI mitlaufen laesst: er ruft `dhrt test tests/pruef` und muss gruen sein.

Dazu die Regeln des Formats, geprueft am echten Laeufer: ein Fehlschlag wird
mit Fall und Zeile genannt, `--- fehler` verlangt den Abbruch, `--- datei`
legt Beilagen neben das Programm, `--filter` waehlt Faelle aus.
"""
from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
PRUEF = _ROOT / "tests" / "pruef"


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")


def _test(*args, cwd=None):
    return subprocess.run([str(_DHRT), "test", *args], capture_output=True, text=True,
                          encoding="utf-8", errors="replace", timeout=600, cwd=cwd)


def test_alle_sammlungen_unter_tests_pruef_sind_gruen():
    """Der Anker: was aus pytest umgezogen ist, laeuft hier weiter mit."""
    r = _test(str(PRUEF))
    assert r.returncode == 0, r.stdout + r.stderr
    assert "Faelle" in r.stdout and " fehl" in r.stdout, r.stdout
    assert " 0 fehl" in r.stdout, r.stdout


def test_ein_fehlschlag_nennt_fall_und_zeile(tmp_path):
    (tmp_path / "s.dhtest").write_text(
        "=== stimmt\nPRINT 1 + 1\n--- erwartet\n2\n"
        "=== stimmt nicht\nPRINT 1 + 1\n--- erwartet\n3\n"
        "=== faellt nicht\nPRINT 1 \\ 0\n--- fehler\nDivision\n"
        "=== faellt nicht obwohl es sollte\nPRINT 5\n--- fehler\nDivision\n",
        encoding="utf-8")
    r = _test("s.dhtest", cwd=str(tmp_path))
    assert r.returncode == 1, r.stdout
    assert "FEHL  Zeile 5: stimmt nicht: Ausgabezeile 1: erwartet '3', erhalten '2'" in r.stdout, r.stdout
    assert "Zeile 13: faellt nicht obwohl es sollte: ein Fehler war erwartet" in r.stdout, r.stdout
    assert "2 von 4 Faellen" in r.stdout, r.stdout


def test_beilage_umgebung_und_filter(tmp_path):
    (tmp_path / "s.dhtest").write_text(
        '=== liest die Beilage\nIMPORT "json"\nDIM h AS JSON_HANDLE : h = JSON_LOAD("daten.json")\n'
        'PRINT JSON_GET_INT(h, "x")\n--- datei daten.json\n{"x": 42}\n--- erwartet\n42\n'
        '=== sieht die Umgebung\nPRINT GETENV$("DH_PROBE")\n--- umgebung\nDH_PROBE=hallo\n--- erwartet\nhallo\n'
        '=== enthaelt reicht\nPRINT "a"\nPRINT "b"\nPRINT "c"\n--- enthaelt\nb\n'
        '=== kaputt\nPRINT 1 \\ 0\n',
        encoding="utf-8")
    r = _test("s.dhtest", cwd=str(tmp_path))
    assert r.returncode == 1 and "1 von 4 Faellen" in r.stdout, r.stdout
    assert "kaputt: Rueckgabewert" in r.stdout, r.stdout
    r = _test("s.dhtest", "--filter", "Beilage", cwd=str(tmp_path))
    assert r.returncode == 0 and "1 Faelle" in r.stdout, r.stdout


def test_eine_kaputte_sammlung_ist_ein_fehler_der_datei(tmp_path):
    (tmp_path / "s.dhtest").write_text("=== a\nPRINT 1\n=== a\nPRINT 2\n", encoding="utf-8")
    r = _test("s.dhtest", cwd=str(tmp_path))
    assert r.returncode == 1 and "gibt es schon" in r.stdout, r.stdout
