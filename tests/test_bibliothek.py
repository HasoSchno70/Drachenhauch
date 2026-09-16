"""Bibliotheken teilen: die PYTHON-Seite der IMPORT-Suche.

Wo dhrt sucht (neben der Datei, DH_PATH, `<Benutzerordner>/.drachenhauch/
bibliothek`), pruefen seit 2026-09-16 die Faelle in
`tests/pruef/bibliothek.dhtest`. Hier bleibt nur, was die Qt-Editoren
brauchen: `drachenhauch/preprocess.py` loest IMPORTs fuer die Zeilen-Herkunft
ein zweites Mal auf, und beide Seiten muessen dieselben Orte kennen -- sonst
zeigt der Editor Fehler in Programmen, die laufen. Die Datei geht mit den
Qt-Editoren.
"""
import os
from pathlib import Path

import pytest

BIB = ("' Kleine Bibliothek.\n"
       "FUNCTION Tage(von AS INTEGER, bis AS INTEGER) AS INTEGER\n"
       "    RETURN (bis - von) \\ 86400\n"
       "END FUNCTION\n")


@pytest.fixture
def projekt(tmp_path):
    """Ein Projekt und eine Bibliothek daneben -- getrennte Ordner."""
    (tmp_path / "lib").mkdir()
    (tmp_path / "proj").mkdir()
    (tmp_path / "lib" / "zeitraum.dh").write_text(BIB, encoding="utf-8")
    (tmp_path / "proj" / "haupt.dh").write_text(
        'IMPORT "zeitraum.dh"\nPRINT Tage(0, 172800)\n', encoding="utf-8")
    return tmp_path


def test_python_und_rust_suchen_dieselben_orte(monkeypatch, tmp_path):
    """Der Editor loest IMPORTs in Python noch einmal auf (Zeilen-Herkunft).
    Kennt eine der beiden Seiten einen Ordner mehr, zeigt der Editor Fehler
    in Programmen, die laufen -- oder schweigt zu welchen, die brechen."""
    from drachenhauch.preprocess import bibliothekspfade
    heim = tmp_path / "heim"
    monkeypatch.setenv("DH_PATH", os.pathsep.join(["/a", "/b"]))
    monkeypatch.setenv("USERPROFILE", str(heim))
    monkeypatch.setenv("HOME", str(heim))
    py = [str(p) for p in bibliothekspfade()]
    assert py[:2] == [str(Path("/a")), str(Path("/b"))]
    assert py[-1] == str(heim / ".drachenhauch" / "bibliothek")


def test_python_findet_die_bibliothek_auch(projekt, monkeypatch):
    """Dieselbe Datei, derselbe Fund -- sonst driftet der Editor weg."""
    from drachenhauch.preprocess import process
    monkeypatch.setenv("DH_PATH", str(projekt / "lib"))
    quelle = (projekt / "proj" / "haupt.dh").read_text(encoding="utf-8")
    merged, _ = process(quelle, projekt / "proj", file_label="haupt.dh")
    assert "FUNCTION Tage" in merged


def test_python_meldet_dieselben_orte(projekt, monkeypatch):
    from drachenhauch.errors import LexerError
    from drachenhauch.preprocess import process
    monkeypatch.delenv("DH_PATH", raising=False)
    quelle = (projekt / "proj" / "haupt.dh").read_text(encoding="utf-8")
    with pytest.raises(LexerError) as e:
        process(quelle, projekt / "proj", file_label="haupt.dh")
    assert "nicht gefunden" in str(e.value)
    assert ".drachenhauch" in str(e.value)
