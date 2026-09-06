"""Die Codebeispiele in docs/ muessen syntaktisch laufen.

Ein eingecheckter Pruefer, den niemand aufruft, findet nichts. Deshalb haengt
`dhrt pruef bloecke` hier an der Testsuite (bis 2026-09-06 `tools/pruef_docs.py`
in Python; Weg A aus docs/entwurf-python-abbau.md -- die Bloecke laufen jetzt
IN-PROZESS durch dieselbe Front-End-Kette wie `--check`, kein dhrt-Start je
Buendel mehr).

Beim ersten Lauf fand er elf Beispiele, die beim Abtippen abbrechen -- drei
Variablen mit reservierten Namen (`data`, `sound`, `map`), zwei entfernte
Builtins (`BITOR`/`SHL`), `FUNCREF(f)` als Aufruf (gab es nie), `EXIT WHILE`
(heisst `BREAK`) und zweimal `IMPORT "a" : IMPORT "b"`.
"""
from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

import pytest

WURZEL = Path(__file__).resolve().parents[1]


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = WURZEL / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


def _pruef(*args):
    dhrt = _dhrt()
    if dhrt is None:
        pytest.skip("native Runtime 'dhrt' nicht gebaut")
    r = subprocess.run([str(dhrt), "pruef", *args], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", cwd=str(WURZEL), timeout=300)
    return r.returncode, r.stdout + r.stderr


def test_alle_codebloecke_parsen():
    code, out = _pruef("bloecke")
    m = re.search(r"(\d+) Codebloecke", out)
    assert m and int(m.group(1)) > 100, "kaum Codebloecke gefunden -- Erkennung kaputt?\n" + out
    assert code == 0, "Codebeispiel(e) in docs/ parsen nicht:\n" + out


def test_notation_gilt_nicht_als_fehler(tmp_path):
    """`...`, `[optional]`, `->` und `FOR each` sind Schreibweise, kein
    Programm -- ohne den Filter meldete der Pruefer 27 Fehler, die alle mit
    Absicht so dastehen. Und ein echter Fehler wird weiter gemeldet."""
    (tmp_path / "probe.md").write_text(
        "```basic\nPRINT ...\nSPEAK(text$ [, unterbrechen])\n```\n\n```basic\nPRINT 1\n```\n",
        encoding="utf-8")
    code, out = _pruef("bloecke", str(tmp_path))
    assert code == 0, out
    assert "2 Codebloecke" in out
    (tmp_path / "kaputt.md").write_text("```basic\nDIM x AS\n```\n", encoding="utf-8")
    code, out = _pruef("bloecke", str(tmp_path))
    assert code == 1 and "kaputt.md:2" in out, out


def test_keine_toten_links_in_den_docs():
    """Relative Links muessen auf etwas zeigen, das es gibt.

    Gefunden hatte das acht Stueck: dreimal ein Pfad relativ zum falschen
    Ordner, fuenfmal Python-Dateien, die mit Stufe B geloescht wurden.
    """
    muster = re.compile(r"\[[^\]]*\]\(([^)\s]+)\)")
    kaputt = []
    for datei in sorted((WURZEL / "docs").glob("*.md")):
        for treffer in muster.finditer(datei.read_text(encoding="utf-8")):
            ziel = treffer.group(1)
            if ziel.startswith(("http://", "https://", "#", "mailto:")):
                continue
            pfad = ziel.split("#")[0]
            if pfad and not (datei.parent / pfad).resolve().exists():
                kaputt.append(f"  {datei.name} -> {ziel}")
    assert not kaputt, "tote Links in docs/:\n" + "\n".join(kaputt)


def test_jede_doku_ist_vom_index_erreichbar():
    """Sonst schreibt jemand 181 Zeilen, die niemand findet (scope.md)."""
    index = (WURZEL / "docs" / "README.md").read_text(encoding="utf-8")
    verlinkt = set(re.findall(r"\(([A-Za-z0-9_.-]+\.md)\)", index))
    alle = {p.name for p in (WURZEL / "docs").glob("*.md")} - {"README.md"}
    fehlen = sorted(alle - verlinkt)
    assert not fehlen, "nicht im Index verlinkt: " + ", ".join(fehlen)
