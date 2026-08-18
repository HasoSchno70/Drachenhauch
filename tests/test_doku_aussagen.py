"""Pruefbare Aussagen in docs/ und README gegen die Wirklichkeit.

Ergaenzt `test_docs_codebloecke.py`: der prueft ```basic-BLOECKE gegen den
Compiler, dieser hier die Stellen daneben -- Befehlsnamen in Tabellen und
Fliesstext, und Zaehlungen wie "39 Module".

Entstanden aus einem systematischen Durchgang durch docs/, der sieben
falsche Aussagen fand. Vier davon waren Verhaltensaussagen und nur durch
Nachmessen zu finden; drei waren Zahlen, die niemand nachzaehlt -- und
genau die haelt dieser Test ab jetzt fest.
"""
from __future__ import annotations

import importlib.util
from pathlib import Path

WURZEL = Path(__file__).resolve().parents[1]


def _pruefer():
    spec = importlib.util.spec_from_file_location(
        "_pruef_aussagen", WURZEL / "tools" / "pruef_doku_aussagen.py")
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def test_befehlsnamen_in_prosa_existieren():
    """Jeder `NAME(`-Verweis in Tabellen/Fliesstext muss ein echtes Builtin
    sein. Faengt Tippfehler und umbenannte Befehle, die kein Codeblock
    abdeckt."""
    m = _pruefer()
    funde = m.pruefe_namen(WURZEL / "docs")
    assert not funde, "\n".join(
        f"{d}:{z}  {w}  -> {msg}" for d, z, w, msg in funde)


def test_zaehlungen_stimmen():
    """'39 Module', '183 Beispiele' -- Zahlen veralten beim naechsten Commit."""
    m = _pruefer()
    funde = m.zaehlungen()
    assert not funde, "\n".join(
        f"{d}:{z}  {w}  -> {msg}" for d, z, w, msg in funde)


def test_geduldete_namen_sind_begruendet():
    """Die Ausnahmeliste soll eine bewusste Entscheidung bleiben, kein
    Abstellgleis -- jeder Eintrag braucht einen Grund im Klartext."""
    m = _pruefer()
    for name, grund in m.GEDULDET.items():
        assert len(grund) > 20, f"{name}: Grund zu duenn ({grund!r})"


def test_tasten_konstanten_stehen_in_der_doku():
    """Jede Konstante aus DEFAULT_KEYS muss in docs/ auffindbar sein.

    Die Liste in `builtins-grafik.md` gab sich als vollstaendig aus und war
    es nie: erst fehlten `KEY_F1`..`KEY_F12` und die Modifier, nach der
    ersten Korrektur immer noch Navigationsblock und Ziffernblock. Von Hand
    aufzaehlen ist offensichtlich die falsche Methode -- darum zaehlt das
    jetzt die Runtime selbst gegen.
    """
    m = _pruefer()
    funde = m.konstanten(WURZEL / "docs")
    assert not funde, "\n".join(
        f"{d}:{z}  {w}  -> {msg}" for d, z, w, msg in funde)


def test_pfadverweise_zeigen_auf_existierende_dateien():
    """`drachenhauch/interpreter.py` in einer Umsetzungs-Checkliste.

    Der Tree-Walker ist seit Stufe B entfernt, die Checkliste in
    befehlssatz-roadmap.md schickte trotzdem jeden, der ein Builtin baut, als
    ERSTEN Schritt in diese Datei. Solche Verweise verrotten leise -- niemand
    klickt eine Doku systematisch durch.
    """
    m = _pruefer()
    funde = m.pfade(WURZEL / "docs")
    assert not funde, "\n".join(
        f"{d}:{z}  {w}  -> {msg}" for d, z, w, msg in funde)


def test_geduldete_pfade_sind_begruendet():
    """Wie GEDULDET: die Ausnahmeliste braucht Gruende, keine Eintraege."""
    m = _pruefer()
    for pfad, grund in m.GEDULDETE_PFADE.items():
        assert len(grund) > 20, f"{pfad}: Grund zu duenn ({grund!r})"
