"""Interaktive `gui`-Beispiele müssen sich beenden lassen.

Der Schleifenkopf eines Fenster-Programms gehört `QUITREQUESTED()`. Wer
stattdessen `WHILE TRUE` oder nur `WHILE NOT KEYPRESSED(KEY_ESCAPE)`
schreibt, baut ein Programm, das

* auf den **Schliessen-Knopf des Fensters nicht reagiert** und
* sich **headless nicht prüfen lässt** -- `DHRT_FRAMES` setzt nur das
  Kennzeichen, das `QUITREQUESTED()` liest; wer nicht fragt, läuft ewig.

Beides ist beim Bau des SFX-Generators am 2026-08-30 passiert, und zwar
zweimal: einmal in `183_sfx_generator.dh` selbst (`WHILE TRUE`) und einmal
in zwei älteren Beispielen, die niemandem aufgefallen waren -- `test_examples.py`
führt nur die **nicht-grafischen** Beispiele aus, diese hier also nicht.

Genau diese Lücke schliesst die Datei: sie startet jedes interaktive
gui-Beispiel headless mit einer kleinen Bildzahl und besteht darauf, dass es
von selbst endet.
"""
import os
import subprocess
import time
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_EXAMPLES = _ROOT / "examples"


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()

pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

# Die Fenster-Programme unter examples/, die eine eigene Ereignisschleife
# drehen. Wer ein neues baut, trägt es hier ein.
_INTERAKTIV = [
    "155_gui_glas",
    "156_gui_alle_widgets",
    "182_gui_tastatur_massstab",
    "183_sfx_generator",
    "184_codefeld",
    "185_partikel_editor",
]


@pytest.mark.parametrize("name", _INTERAKTIV)
def test_beispiel_endet_von_selbst(name):
    pfad = _EXAMPLES / f"{name}.dh"
    assert pfad.exists(), pfad
    env = dict(os.environ, DHRT_FRAMES="20")
    t0 = time.monotonic()
    try:
        r = subprocess.run([str(_DHRT), "run", str(pfad)], capture_output=True,
                           text=True, encoding="utf-8", env=env, timeout=45,
                           cwd=str(_EXAMPLES))
    except subprocess.TimeoutExpired:
        pytest.fail(
            f"{name}.dh endet nicht: DHRT_FRAMES wurde erreicht, das Programm "
            f"läuft weiter. Schleifenkopf auf 'WHILE NOT QUITREQUESTED()' "
            f"umstellen -- sonst reagiert es auch auf den Schliessen-Knopf nicht.")
    # `assert returncode == 0, r.stderr` ist Pflicht und keine Kosmetik: ohne
    # Bildschirm bricht raylib beim Fenster ab, und nur wenn seine Meldung IM
    # FEHLERTEXT steht, macht conftest daraus einen Skip statt eines
    # Fehlschlags.
    assert r.returncode == 0, r.stderr
    # 20 Bilder sind ein Drittel einer Sekunde. Alles jenseits weniger
    # Sekunden heisst: es lief bis zu irgendeiner Grenze statt bis zum
    # Kennzeichen.
    assert time.monotonic() - t0 < 25, f"{name}.dh brauchte auffällig lange"
