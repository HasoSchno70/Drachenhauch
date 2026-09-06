"""Die VSCode-Grammatik muss existieren, passen und aktuell sein.

Hintergrund: Bei der Umbenennung wurde der INHALT der Grammatik angepasst
(`scopeName: source.drachenhauch`), der DATEINAME aber nicht. `package.json`
zeigte danach auf `drachenhauch.tmLanguage.json`, im Repo lag
`gamebasic.tmLanguage.json` -- die Extension hatte also gar keine Grammatik
mehr. Aufgefallen ist es erst beim Durchsehen der Commits.

Nebenbei kam heraus, dass die eingecheckte Datei 223 Builtins nicht kannte:
sie war generiert und seit Monaten nicht neu erzeugt worden. Ein generiertes
Artefakt driftet lautlos -- deshalb prueft der dritte Test, dass ein
Neu-Erzeugen nichts aendert.
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import pytest

WURZEL = Path(__file__).resolve().parents[1]
EXT = WURZEL / "vscode-drachenhauch"


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = WURZEL / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


def _pfad_aus_package_json() -> Path:
    d = json.loads((EXT / "package.json").read_text(encoding="utf-8"))
    gram = d["contributes"]["grammars"][0]["path"]
    return (EXT / gram).resolve()


def test_grammatik_liegt_dort_wo_package_json_sie_sucht():
    ziel = _pfad_aus_package_json()
    assert ziel.exists(), (
        f"package.json verweist auf {ziel.name}, die Datei fehlt -- "
        "die Extension haette keine Syntaxfarben")


def test_keine_verwaiste_grammatik_daneben():
    """Zwei Grammatiken im Ordner heisst: eine davon liest niemand."""
    ziel = _pfad_aus_package_json()
    andere = [p.name for p in (EXT / "syntaxes").glob("*.tmLanguage.json")
              if p.resolve() != ziel]
    assert not andere, f"verwaiste Grammatik(en): {andere}"


def test_grammatik_ist_aktuell():
    """Neu erzeugen darf nichts aendern -- sonst ist sie gedriftet. Erzeugt
    wird sie seit 2026-09-06 von `dhrt doku grammatik` (vorher
    `build_grammar.py` in Python)."""
    dhrt = _dhrt()
    if dhrt is None:
        pytest.skip("native Runtime 'dhrt' nicht gebaut")
    r = subprocess.run([str(dhrt), "doku", "grammatik", "--pruefen"], capture_output=True,
                       text=True, encoding="utf-8", cwd=str(WURZEL), timeout=60)
    assert r.returncode == 0, "Grammatik ist veraltet -- neu erzeugen mit dhrt doku grammatik: " + r.stdout


def test_grammatik_kennt_neue_befehle():
    """Der Sinn der Erzeugung: ein Befehl, der im Index steht, wird gefaerbt."""
    text = _pfad_aus_package_json().read_text(encoding="utf-8")
    assert "SPEAK_SOUND" in text and "GUI_ANNOUNCE" in text
    assert "WHILE" in text and "KEY_SPACE" in text
