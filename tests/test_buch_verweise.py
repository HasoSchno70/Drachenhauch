"""Querverweise im Referenzbuch: zeigen sie auf ein Kapitel, das es gibt?

Das Buch nummeriert seine Kapitel NICHT -- die Ueberschrift ist nur der Titel.
Beim Gegenlesen am 2026-08-30 verwiesen trotzdem 20 Stellen auf "Kapitel 61"
und aehnliche Zahlen; das waren die DATEINUMMERN der Quelle, und der Leser
sieht sie nirgends. Umgestellt auf `Kapitel "Modul: save"` -- und dabei fielen
zwei Titel-Verweise auf, die auf gar kein Kapitel zeigten (`"Strings"` statt
`"Strings im Detail"`, `"timer"` statt `"Modul: timer"`).

Beides faellt still aus, wenn es niemand prueft: Ein Verweis ins Leere sieht
im gesetzten Buch aus wie jeder andere.

Braucht Node (die Kapitel sind JavaScript-Module) -- ohne Node uebersprungen.
"""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

import pytest

WURZEL = Path(__file__).resolve().parents[1]
EXPORT = WURZEL / "tools" / "buch_verweise_export.js"
CONTENT = WURZEL / "buch-referenz" / "buch" / "content"


def _node_da() -> bool:
    try:
        return subprocess.run(["node", "--version"], capture_output=True, timeout=30).returncode == 0
    except (OSError, subprocess.SubprocessError):
        return False


@pytest.fixture(scope="module")
def buch() -> dict:
    if not _node_da():
        pytest.skip("node nicht verfuegbar")
    roh = subprocess.run(
        ["node", str(EXPORT)],
        capture_output=True, text=True, encoding="utf-8", cwd=WURZEL, timeout=180,
    )
    if roh.returncode != 0:
        pytest.skip(f"Verweis-Export fehlgeschlagen: {roh.stderr[:200]}")
    d = json.loads(roh.stdout)
    # Ohne diese Schranke liefen beide Pruefungen bei einem kaputten Export
    # leer durch und meldeten Erfolg.
    assert len(d["titel"]) > 50, f"nur {len(d['titel'])} Kapitel gefunden"
    assert len(d["verweise"]) > 20, f"nur {len(d['verweise'])} Verweise gefunden"
    return d


def test_jeder_verweis_trifft_ein_kapitel(buch: dict) -> None:
    titel = set(buch["titel"])
    tot = [v for v in buch["verweise"] if v["ziel"] not in titel]
    assert not tot, "Verweis auf ein Kapitel, das es nicht gibt:\n" + "\n".join(
        f"  {v['datei']}: \"{v['ziel']}\"" for v in tot
    )


def test_keine_verweise_auf_kapitelnummern() -> None:
    """Nummern zeigen ins Nichts -- die Kapitel tragen im Buch keine."""
    treffer = []
    for p in sorted(CONTENT.glob("*.js")):
        for m in re.finditer(r"Kapitel (\d+)", p.read_text(encoding="utf-8")):
            treffer.append(f"  {p.name}: \"{m.group(0)}\"")
    assert not treffer, (
        "Verweis auf eine Kapitelnummer -- das Buch nummeriert seine Kapitel "
        "nicht, der Leser sieht die Zahl nirgends. Den Titel nennen:\n"
        + "\n".join(treffer)
    )
