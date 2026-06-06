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
    "28_particles",                   # nutzt RANDOMIZE(42)
    "29_camera",
    "30_select",
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
    "28_particles_visual", "29_camera_visual",
]


def _run_example(rel: str) -> str:
    """Fuehrt ein Beispiel im Tree-Walker aus und gibt stdout zurueck.

    CWD wird temporaer auf examples/ umgestellt, damit Programme mit
    LOADIMAGE("assets/...") die Datei finden (gleiches Verhalten wie gbrun.py).
    """
    import io, contextlib, os
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.interpreter import Interpreter
    from gamebasic.preprocess import process
    # preprocess laedt die Built-in-Module nicht mehr (Stufe B) -> der
    # Tree-Walker braucht sie explizit. (TW + dieser Helper werden in Phase 8
    # entfernt.)
    from gamebasic.modules import load_all_modules
    load_all_modules()

    path = _EXAMPLES / f"{rel}.gb"
    source = path.read_text(encoding="utf-8")
    prepped, _ = process(source, path.parent, file_label=path.name)
    ast = Parser(Lexer(prepped).tokenize()).parse()

    saved_cwd = os.getcwd()
    try:
        os.chdir(path.parent)
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            Interpreter().run(ast)
    finally:
        os.chdir(saved_cwd)
    return buf.getvalue()


@pytest.mark.parametrize("name", _DETERMINISTIC + _NON_DETERMINISTIC)
def test_example_runs_treewalker(name):
    """Smoke-Test: jedes (nicht-grafische) Beispiel laeuft im Tree-Walker durch
    und produziert Output. Output-Identitaet gegen gbrt -> test_gbrt_parity.py."""
    out = _run_example(name)
    assert out != "", f"Beispiel {name} hat keinen Output erzeugt"
