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

import importlib.util
import json
from pathlib import Path

WURZEL = Path(__file__).resolve().parents[1]
EXT = WURZEL / "vscode-drachenhauch"


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
    """Neu erzeugen darf nichts aendern -- sonst ist sie gedriftet."""
    spec = importlib.util.spec_from_file_location(
        "_build_grammar", EXT / "build_grammar.py")
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    frisch = m.build()
    eingecheckt = json.loads(_pfad_aus_package_json().read_text(encoding="utf-8"))
    assert frisch == eingecheckt, (
        "Grammatik ist veraltet -- neu erzeugen mit\n"
        "  <venv>\\python.exe vscode-drachenhauch\\build_grammar.py")
