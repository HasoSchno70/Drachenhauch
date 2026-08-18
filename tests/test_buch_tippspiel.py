"""Alle Kapitelstaende des Tippspiel-Buchs starten ohne Fehler.

Ein Lehrbuch, dessen Beispiele nicht mehr laufen, ist schlimmer als keins:
der Leser sucht den Fehler bei sich. Deshalb startet dieser Test jeden
Kapitelstand einmal wirklich -- die Fenster-Kapitel headless mit ein paar
Bildern, die Konsolen-Kapitel ganz normal.

Nicht geprueft wird, wie es AUSSIEHT (dafuer gibt es Screenshots von Hand),
sondern nur: startet, laeuft, endet ohne Fehler und ohne Compiler-Warnung.
"""
from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_CODE = _ROOT / "buch-tippspiel" / "code"


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _find_dhrt()

# Kapitel mit Fenster: brauchen DHRT_FRAMES, sonst laufen sie ewig.
_MIT_FENSTER = {
    "kap01/fenster.dh", "kap04/spielplan.dh", "kap05/tipps.dh",
    "kap06/punkte.dh", "kap07/rangliste.dh", "kap08/zeit.dh",
    "kap09/netz.dh", "kap10/robust.dh", "kap11/sicherung.dh",
    "kap12/politur.dh",
}

# Kapitel ohne Fenster (Konsole).
_KONSOLE = {"kap02/daten.dh", "kap03/regel.dh", "kap13/weitergeben.dh"}

_ALLE = sorted(_MIT_FENSTER | _KONSOLE)

# Kapitel 9 und danach holen den Spielplan aus dem Netz -- aber nur auf
# Knopfdruck. Ohne Klick geht dieser Test also nie ins Netz.


def _lauf(pfad: Path, tmp_path: Path, frames: int | None):
    """Kapitel in einer Kopie starten, damit keine .db im Repo landet."""
    ziel = tmp_path / pfad.parent.name
    ziel.mkdir(parents=True, exist_ok=True)
    kopie = ziel / pfad.name
    shutil.copy(pfad, kopie)

    env = dict(os.environ)
    if frames is not None:
        env["DHRT_FRAMES"] = str(frames)
    return subprocess.run([str(_DHRT), "run", str(kopie)],
                          capture_output=True, timeout=180, env=env)


@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
@pytest.mark.parametrize("rel", _ALLE)
def test_kapitel_laeuft(rel, tmp_path):
    pfad = _CODE / rel
    assert pfad.exists(), f"Kapitelstand fehlt: {rel}"

    r = _lauf(pfad, tmp_path, 30 if rel in _MIT_FENSTER else None)
    aus = r.stdout.decode("utf-8", "replace") + r.stderr.decode("utf-8", "replace")

    assert r.returncode == 0, f"{rel} endete mit {r.returncode}:\n{aus}"
    assert "Laufzeitfehler" not in aus, f"{rel}:\n{aus}"
    assert "Parse-Fehler" not in aus, f"{rel}:\n{aus}"
    # Eine Warnung im Lehrbuch-Beispiel ist ein Fehler: der Leser tippt sie ab.
    assert "Warnung:" not in aus, f"{rel}:\n{aus}"


@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
def test_zielstand_laeuft(tmp_path):
    r = _lauf(_CODE / "tippspiel.dh", tmp_path, 30)
    aus = r.stdout.decode("utf-8", "replace") + r.stderr.decode("utf-8", "replace")
    assert r.returncode == 0, aus
    assert "Laufzeitfehler" not in aus, aus
    assert "Warnung:" not in aus, aus


@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
@pytest.mark.parametrize("rel", ["tippspiel_pruefung.dh", "zeit_pruefung.dh"])
def test_pruefprogramm_ist_gruen(rel, tmp_path):
    """Die Pruefprogramme pruefen die Regel -- hier wird geprueft, dass sie
    ueberhaupt noch gruen sind. (abruf_pruefung.dh geht ins Netz und bleibt
    deshalb aussen vor.)"""
    r = _lauf(_CODE / rel, tmp_path, None)
    aus = r.stdout.decode("utf-8", "replace")
    assert "ALLES GRUEN" in aus, aus
    # Seit der Umstellung auf ASSERT_* (WP E) endet ein Pruefprogramm mit
    # einem Rueckgabewert -- vorher war er IMMER 0, egal was herauskam. Nur
    # den Text zu pruefen wuerde die eigentliche Zusage wieder verlieren.
    assert r.returncode == 0, (r.returncode, aus,
                               r.stderr.decode("utf-8", "replace"))


@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
def test_kapitel_wachsen():
    """Jedes Kapitel baut auf dem vorigen auf. Schrumpft eines, ist beim
    Bearbeiten etwas verlorengegangen."""
    laengen = []
    for rel in ["kap04/spielplan.dh", "kap05/tipps.dh", "kap06/punkte.dh",
                "kap07/rangliste.dh", "kap08/zeit.dh", "kap09/netz.dh",
                "kap10/robust.dh", "kap11/sicherung.dh", "kap12/politur.dh"]:
        laengen.append((rel, len((_CODE / rel).read_text(encoding="utf-8").splitlines())))
    for (a, la), (b, lb) in zip(laengen, laengen[1:]):
        assert lb > la, f"{b} ({lb} Zeilen) ist kuerzer als {a} ({la} Zeilen)"


def test_readme_verweist_auf_alle_kapitel():
    text = (_ROOT / "buch-tippspiel" / "README.md").read_text(encoding="utf-8")
    for nr in range(1, 14):
        assert f"code/kap{nr:02d}" in text, f"Kapitel {nr} fehlt im README"
