"""Bausteine des Referenzbuchs: sind sie richtig befuellt?

Die Hinweiskaesten haben eine Falle in der Argumentfolge: `H.tip` nimmt
(TITEL, text), `H.note` und `H.warn` dagegen (TEXT, titel). Wer sich vertut,
uebergibt einen ganzen Absatz als Titel. Es stuerzt nichts ab -- gesetzt wird
ein Kasten, dessen Ueberschrift 145 Zeichen Fettdruck ist und dessen Rumpf
leer bleibt. Genau das steckte am 2026-08-30 fuenfmal im Buch.

Dazu zwei weitere Formfehler, die still durchgehen: ein Befehlseintrag ohne
Beschreibung oder ohne Signatur.

Braucht Node (die Kapitel sind JavaScript-Module) -- ohne Node uebersprungen.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

import pytest

WURZEL = Path(__file__).resolve().parents[1]
EXPORT = WURZEL / "tools" / "buch_struktur_export.js"


def _node_da() -> bool:
    try:
        return subprocess.run(["node", "--version"], capture_output=True, timeout=30).returncode == 0
    except (OSError, subprocess.SubprocessError):
        return False


@pytest.fixture(scope="module")
def struktur() -> dict:
    if not _node_da():
        pytest.skip("node nicht verfuegbar")
    roh = subprocess.run(
        ["node", str(EXPORT)],
        capture_output=True, text=True, encoding="utf-8", cwd=WURZEL, timeout=180,
    )
    if roh.returncode != 0:
        pytest.skip(f"Struktur-Export fehlgeschlagen: {roh.stderr[:200]}")
    d = json.loads(roh.stdout)
    # Ohne diese Schranke liefen alle Pruefungen bei einem kaputten Export
    # leer durch und meldeten Erfolg.
    assert d["kapitel"] > 50, f"nur {d['kapitel']} Kapitel gefunden"
    return d


def test_jeder_tipp_hat_einen_rumpf(struktur: dict) -> None:
    """H.tip(TITEL, text) -- mit nur einem Argument wird der Text zur Ueberschrift."""
    fehlt = struktur["tips_ohne_rumpf"]
    assert not fehlt, (
        "H.tip ohne Rumpf -- der Text steht als Ueberschrift und wird ganz fett "
        "gesetzt. Die Argumentfolge ist (Titel, Text):\n"
        + "\n".join(f"  {t['datei']}: \"{t['titel']}\"" for t in fehlt)
    )


def test_jeder_befehl_hat_beschreibung_und_signatur(struktur: dict) -> None:
    ohne = struktur["cmds_ohne_text"] + struktur["cmds_ohne_sig"]
    assert not ohne, "Eintrag ohne Beschreibung oder Signatur:\n" + "\n".join(
        f"  {e['datei']}: {e['name']}" for e in ohne
    )


def test_zwischenueberschriften_bleiben_kurz(struktur: dict) -> None:
    """Eine Ueberschrift ueber 70 Zeichen ist meist ein verrutschter Absatz."""
    lang = struktur["lange_h2"]
    assert not lang, "Zwischenueberschrift zu lang:\n" + "\n".join(
        f"  {e['datei']}: \"{e['text'][:80]}\"" for e in lang
    )
