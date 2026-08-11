"""Die Codebeispiele in docs/ muessen syntaktisch laufen.

Ein eingecheckter Pruefer, den niemand aufruft, findet nichts. Deshalb haengt
`tools/pruef_docs.py` hier an der Testsuite.

Beim ersten Lauf fand er elf Beispiele, die beim Abtippen abbrechen -- drei
Variablen mit reservierten Namen (`data`, `sound`, `map`), zwei entfernte
Builtins (`BITOR`/`SHL`), `FUNCREF(f)` als Aufruf (gab es nie), `EXIT WHILE`
(heisst `BREAK`) und zweimal `IMPORT "a" : IMPORT "b"`.
"""
from __future__ import annotations

import importlib.util
from pathlib import Path

import pytest

WURZEL = Path(__file__).resolve().parents[1]


def _pruefer():
    spec = importlib.util.spec_from_file_location(
        "_pruef_docs", WURZEL / "tools" / "pruef_docs.py")
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def test_alle_codebloecke_parsen():
    m = _pruefer()
    dhrt = m.finde_dhrt()
    if dhrt is None:
        pytest.skip("native Runtime 'dhrt' nicht gebaut")
    bloecke, funde = m.pruefe(WURZEL / "docs", dhrt)
    assert bloecke > 100, "kaum Codebloecke gefunden -- Erkennung kaputt?"
    if funde:
        zeilen = "\n".join(f"  {n}:{z}  {q[:70]}  -> {msg[:70]}"
                           for n, z, q, msg in funde)
        pytest.fail(f"{len(funde)} Codebeispiel(e) in docs/ parsen nicht:\n{zeilen}")


def test_keine_toten_links_in_den_docs():
    """Relative Links muessen auf etwas zeigen, das es gibt.

    Gefunden hatte das acht Stueck: dreimal ein Pfad relativ zum falschen
    Ordner, fuenfmal Python-Dateien, die mit Stufe B geloescht wurden.
    """
    import re
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
    import re
    index = (WURZEL / "docs" / "README.md").read_text(encoding="utf-8")
    verlinkt = set(re.findall(r"\(([A-Za-z0-9_.-]+\.md)\)", index))
    alle = {p.name for p in (WURZEL / "docs").glob("*.md")} - {"README.md"}
    fehlen = sorted(alle - verlinkt)
    assert not fehlen, "nicht im Index verlinkt: " + ", ".join(fehlen)
