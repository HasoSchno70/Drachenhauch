"""Smoke-Tests: alle non-grafischen Examples laufen im Tree-Walker ohne Fehler.

Seit dem Entfernen der Python-/Cython-Bytecode-VMs gibt es nur noch zwei Pfade:
Tree-Walker (hier) und die native Runtime `gbrt`. Die Output-Identitaet
Tree-Walker == gbrt fuer die deterministischen Beispiele prueft der dedizierte
Sweep in `test_gbrt_parity.py`. Examples mit MILLIS()/TIME$()/RND-ohne-RANDOMIZE
sind nicht-deterministisch; Examples mit SCREEN/Pygame bleiben aussen vor.
"""
from pathlib import Path
import pytest

_ROOT = Path(__file__).resolve().parent.parent
_EXAMPLES = _ROOT / "examples"


# Examples die wir laufen lassen + Bench-Equivalence pruefen.
_DETERMINISTIC = [
    "01_hello", "02_variables", "03_loops", "04_fibonacci",
    "06_functions", "07_classes", "08_inheritance",
    "11_arrays", "14_strings", "15_struct",
    "20_try",
    "24_json", "25_db",
    "140_particles",                  # nutzt RANDOMIZE(42)
    "29_camera",
    "21_select",
]

# Programme die zwar laufen, aber non-deterministisch sind (Zeit, ungeseedetes RND,
# oder geteilter Datei-State zwischen Tree-Walker und VM-Lauf).
_NON_DETERMINISTIC = [
    "05_builtins",   # RND ohne RANDOMIZE
    "16_files",      # schreibt highscores.txt -> 2. Lauf sieht andere Daten
    "18_math",       # TIME$() / 100k SIN-Bench
    "19_maps",       # MILLIS()-Zeitmessung im Output
    "26_tween",      # nutzt MILLIS() in der Demo
]

# Pygame-/Asset-abhaengige (brauchen interaktives Fenster oder Asset-Pfad-Setup).
_INTERACTIVE_OR_ASSETS = [
    "09_shapes", "10_pong", "12_sprite", "13_sound",
    "17_tilemap", "21_modules", "22_tetris", "23_platformer",
    "27_imgfx",     # laedt assets/hero.png - braucht gbrun.py (chdir)
    "31_sprite",    # LOADIMAGE/Sprite-Rendering -> nur native (gbrt)
    "28_particles_visual", "141_camera_visual",
]


def _find_gbrt():
    import os
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = _ROOT / "rust" / "gb_runtime" / "target" / variant / exe
        if p.exists():
            return p
    return None


_GBRT = _find_gbrt()


def _run_example(rel: str) -> str:
    """Fuehrt ein Beispiel ueber die native Runtime (`gbrt run`) aus und gibt
    stdout zurueck. gbrt chdirt selbst ins examples/-Verzeichnis (relative
    Asset-/IMPORT-Pfade)."""
    import subprocess
    if _GBRT is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    path = _EXAMPLES / f"{rel}.gb"
    r = subprocess.run([str(_GBRT), "run", str(path)],
                       capture_output=True, text=True, encoding="utf-8", timeout=60)
    assert r.returncode == 0, f"gbrt run {rel} Exit {r.returncode}: {r.stderr}"
    return (r.stdout or "").replace("\r\n", "\n")


@pytest.mark.parametrize("name", _DETERMINISTIC + _NON_DETERMINISTIC)
def test_example_runs(name):
    """Smoke-Test: jedes (nicht-grafische) Beispiel laeuft in gbrt durch und
    produziert Output."""
    out = _run_example(name)
    assert out != "", f"Beispiel {name} hat keinen Output erzeugt"
