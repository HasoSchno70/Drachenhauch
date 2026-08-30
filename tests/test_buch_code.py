"""Die Codebeispiele des Referenzbuchs: uebersetzen sie, und stimmt die Ausgabe?

Zwei Werkzeuge im Buchverzeichnis erledigen die Arbeit; dieser Test ruft sie,
damit sie bei jedem Lauf mitlaufen. Bis zum 2026-08-30 taten das weder die
Suite noch die CI -- `pruef_codebloecke.js` lief nur, wenn jemand daran dachte,
und `pruef_ausgaben.js` gab es gar nicht.

* **uebersetzen**: `pruef_codebloecke.js` schickt jeden Block durch
  `dhrt --check`. Ein Tippfehler im Buch faellt sonst erst dem Leser auf, der
  ihn abtippt.
* **stimmen**: `pruef_ausgaben.js` FUEHRT die Bloecke aus und vergleicht mit
  der `{ out: [...] }`-Angabe. Die stand fuer 332 Bloecke im Buch und war nie
  nachgemessen -- sieben davon waren falsch: die Beispiele wurden bei der
  Umbenennung von GameBasic auf Drachenhauch angepasst, die behaupteten
  Ausgaben nicht (`LEN("Drachenhauch")` stand als 9 statt 12 im Buch).

Braucht Node und ein gebautes `dhrt` -- sonst uebersprungen.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

import pytest

WURZEL = Path(__file__).resolve().parents[1]
BUCH = WURZEL / "buch-referenz" / "buch"
DHRT = WURZEL / "rust" / "drachenhauch_runtime" / "target" / "release" / "dhrt.exe"
DHRT_POSIX = WURZEL / "rust" / "drachenhauch_runtime" / "target" / "release" / "dhrt"


def _node_da() -> bool:
    try:
        return subprocess.run(["node", "--version"], capture_output=True, timeout=30).returncode == 0
    except (OSError, subprocess.SubprocessError):
        return False


def _lauf(werkzeug: str) -> subprocess.CompletedProcess:
    if not _node_da():
        pytest.skip("node nicht verfuegbar")
    if not (DHRT.exists() or DHRT_POSIX.exists()):
        pytest.skip("dhrt nicht gebaut")
    return subprocess.run(
        ["node", str(BUCH / werkzeug)],
        capture_output=True, text=True, encoding="utf-8", cwd=BUCH, timeout=1800,
    )


def test_alle_codebloecke_uebersetzen() -> None:
    r = _lauf("pruef_codebloecke.js")
    assert r.returncode == 0, r.stdout + r.stderr
    # Schranke gegen den leeren Lauf: ohne sie meldete ein kaputter Sammler
    # Erfolg, weil er nichts zu pruefen fand.
    assert "0 mit Befund" in r.stdout, r.stdout
    zahl = int(r.stdout.split("Codebloecke")[0].strip().split()[-1])
    assert zahl > 800, f"nur {zahl} Codebloecke gefunden"


def test_die_behaupteten_ausgaben_stimmen() -> None:
    r = _lauf("pruef_ausgaben.js")
    assert r.returncode == 0, r.stdout + r.stderr
    # Die Schranke faengt den LEEREN Lauf, sie nagelt die Zahl nicht fest: die
    # CI baut dhrt auf posix ohne Grafik und ohne Hardware-Module, dort faellt
    # mehr als Bruchstueck weg. Auf dieser Maschine sind es 211.
    zahl = int(r.stdout.split("ausgefuehrt")[0].strip().split()[-1])
    assert zahl > 100, f"nur {zahl} Ausgabe-Bloecke ausgefuehrt -- laeuft dhrt?"
