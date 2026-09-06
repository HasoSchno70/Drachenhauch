"""Pruefbare Aussagen in docs/ und README gegen die Wirklichkeit.

Ergaenzt `test_docs_codebloecke.py`: der prueft ```basic-BLOECKE gegen den
Compiler, dieser hier die Stellen daneben -- Befehlsnamen in Tabellen und
Fliesstext, Zaehlungen wie "39 Module", Tasten-Konstanten, Pfade und Links.

Der Pruefer ist `dhrt pruef` (bis 2026-09-06 `tools/pruef_doku_aussagen.py`
in Python; Weg A aus docs/entwurf-python-abbau.md). Entstanden aus einem
systematischen Durchgang durch docs/, der sieben falsche Aussagen fand. Vier
davon waren Verhaltensaussagen und nur durch Nachmessen zu finden; drei waren
Zahlen, die niemand nachzaehlt -- und genau die haelt dieser Test ab jetzt fest.
"""
from __future__ import annotations

import os
import shutil
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


def test_befehlsnamen_in_prosa_existieren():
    """Jeder `NAME(`-Verweis in Tabellen/Fliesstext muss ein echtes Builtin
    sein. Faengt Tippfehler und umbenannte Befehle, die kein Codeblock
    abdeckt."""
    code, out = _pruef("namen")
    assert code == 0, out


def test_befehlsnamen_pruefung_schlaegt_an(tmp_path):
    """Gegenprobe: ein erfundener Befehl faellt auf, ein Schluesselwort nicht."""
    (tmp_path / "probe.md").write_text(
        "Mit `GIBTSNICHT_XY(1)` und `PRINT(1)` und `ABS(x)`.\n", encoding="utf-8")
    code, out = _pruef("namen", str(tmp_path))
    assert code == 1 and "GIBTSNICHT_XY" in out and "PRINT" not in out.split("Befund")[1], out


def test_zaehlungen_stimmen():
    """'39 Module', '183 Beispiele' -- Zahlen veralten beim naechsten Commit."""
    code, out = _pruef("zaehlungen")
    assert code == 0, out


def test_beispiel_zaehlung_kommt_aus_der_versionsverwaltung(tmp_path):
    """Gegen die Zaehlung darf kein Fremdkoerper im Arbeitsverzeichnis stehen.

    Die Zahl kam frueher aus `examples.glob("*.dh")` -- und genau darin lag
    ein sporadischer Fehlschlag (2026-08-27, etwa jeder fuenfte parallele
    Lauf, beim Wiederholen gruen). Ursache ist keine Test-Datei: die
    Live-Diagnose des Editors legt ihre Temp-Datei NEBEN die gepruefte
    Quelle -- sie muss dort liegen, damit `IMPORT "x.dh"` und relative
    Asset-Pfade aufloesen. Wer waehrend des Testlaufs ein Beispiel im Editor
    offen hat, dem blinkt fuer ~40 ms ein `examples/tmpXXXX.dh` ins
    Verzeichnis; faellt der glob hinein, meldet der Pruefer
    "sagt 195, tatsaechlich 196".

    Nachgestellt wird das hier in einem EIGENEN Wegwerf-Repo -- eine
    Streudatei im echten `examples/` wuerde parallel laufenden Sweeps genau
    denselben Streich spielen.
    """
    if shutil.which("git") is None:
        pytest.skip("git nicht installiert -- dann zaehlt die Rueckfallebene")

    def git(*args):
        subprocess.run(["git", *args], cwd=repo, check=True, capture_output=True)

    repo = tmp_path / "repo"
    (repo / "examples").mkdir(parents=True)
    (repo / "examples" / "01_hallo.dh").write_text("PRINT 1\n", encoding="utf-8")
    (repo / "examples" / "02_welt.dh").write_text("PRINT 2\n", encoding="utf-8")
    git("init")
    git("config", "user.email", "t@t.de")
    git("config", "user.name", "Tester")
    git("add", "examples")
    git("-c", "commit.gpgsign=false", "commit", "-m", "zwei Beispiele")

    assert _pruef("beispiele", str(repo))[1].strip() == "2 versioniert"
    # Der Fremdkoerper aus der Live-Diagnose -- unversioniert, also unsichtbar.
    (repo / "examples" / "tmp9k3x.dh").write_text("PRINT 1\n", encoding="utf-8")
    assert _pruef("beispiele", str(repo))[1].strip() == "2 versioniert"
    assert len(list((repo / "examples").glob("*.dh"))) == 3   # der glob saehe ihn


def test_tasten_konstanten_stehen_in_der_doku():
    """Jede Konstante aus DEFAULT_KEYS muss in docs/ auffindbar sein.

    Die Liste in `builtins-grafik.md` gab sich als vollstaendig aus und war
    es nie: erst fehlten `KEY_F1`..`KEY_F12` und die Modifier, nach der
    ersten Korrektur immer noch Navigationsblock und Ziffernblock. Von Hand
    aufzaehlen ist offensichtlich die falsche Methode -- darum zaehlt das
    jetzt die Runtime selbst gegen.
    """
    code, out = _pruef("konstanten")
    assert code == 0, out


def test_pfadverweise_zeigen_auf_existierende_dateien():
    """`drachenhauch/interpreter.py` in einer Umsetzungs-Checkliste.

    Der Tree-Walker ist seit Stufe B entfernt, die Checkliste in
    befehlssatz-roadmap.md schickte trotzdem jeden, der ein Builtin baut, als
    ERSTEN Schritt in diese Datei. Solche Verweise verrotten leise -- niemand
    klickt eine Doku systematisch durch.
    """
    code, out = _pruef("pfade")
    assert code == 0, out


# ------------------------------------------------------------- CLAUDE.md
# CLAUDE.md wird seit dem Sweep am 2026-08-29 mitgeprueft -- aus demselben
# Grund wie `docs/`, nur mit mehr Gewicht: sie ist die Datei, die als Erstes
# gelesen wird, wenn jemand sich in diesem Repo zurechtfinden will. Der Sweep
# fand dort vier tote Markdown-Links (auf den geloeschten Tree-Walker, den
# Python-Compiler, das Python-ECS und eine geloeschte Testdatei), ein
# umbenanntes Beispiel und einen Modul-Verweis auf eine Datei, die es seit
# Stufe B nicht mehr gibt.

def test_claude_md_befehlsnamen_existieren():
    code, out = _pruef("namen", str(WURZEL), "--nur", "CLAUDE.md")
    assert code == 0, out


def test_claude_md_pfade_und_links_leben():
    code, out = _pruef("pfade", str(WURZEL), "--nur", "CLAUDE.md")
    assert code == 0, out


def test_markdown_links_werden_ueberhaupt_geprueft(tmp_path):
    """Regression fuer die Pruefung selbst: sie sah lange nur Inline-Code
    (`pfad.py`) und uebersah Links -- ausgerechnet die Form, in der die vier
    toten Verweise in CLAUDE.md standen.

    Mitgeprueft wird gleich, was NICHT anschlagen darf: ein Ordner-Ziel, eine
    Zeilennummer am Ende und ein Ziel im Netz. Und dass GEDULDETE_PFADE fuer
    Links NICHT gilt: eine historische Nennung schreibt man als Inline-Code,
    ein Link verspricht 'hier kannst du hinspringen'.
    """
    (tmp_path / "probe.md").write_text(
        "[tot](drachenhauch/gibtsnicht.py)\n"
        "[lebt](rust/build_wasm.py)\n"
        "[ordner](docs/)\n"
        "[mit Zeile](drachenhauch/lexer.py:114)\n"
        "[netz](https://example.invalid/x.py)\n"
        "`drachenhauch/interpreter.py` als Inline-Code ist geduldet\n"
        "[klick](drachenhauch/interpreter.py)\n", encoding="utf-8")
    # Das Repo ist die Wahrheit fuer "gibt es": geprueft wird die Probe im
    # Repo-Kontext, nur die Datei liegt woanders.
    code, out = _pruef("pfade", str(tmp_path), "--nur", "probe.md")
    assert code == 1, out
    befunde = [z for z in out.splitlines() if z.strip().startswith("->")]
    assert len(befunde) == 2, out
    assert "gibtsnicht.py" in out and "[...](drachenhauch/interpreter.py)" in out
