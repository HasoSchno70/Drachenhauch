"""Smoke-Tests: alle non-grafischen Examples laufen ohne Fehler durch.

Wir fuehren jedes Beispiel im Tree-Walker UND im Python-VM aus und vergleichen.
Examples mit MILLIS()/TIME$()/RND-ohne-RANDOMIZE sind nicht-deterministisch und
werden uebersprungen. Examples mit SCREEN/Pygame bleiben aussen vor (interaktiv).
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
    "31_sprite",
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
    "28_particles_visual", "29_camera_visual",
]


def _run_example(rel: str):
    """Tree-Walker + Python-VM laufen lassen, gibt (tw_out, vm_out) zurueck.

    CWD wird temporaer auf examples/ umgestellt, damit Programme mit
    LOADIMAGE("assets/...") die Datei finden (gleiches Verhalten wie gbrun.py).
    """
    import io, contextlib, os
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.interpreter import Interpreter
    from gamebasic.compiler import Compiler
    from gamebasic.vm import VM
    from gamebasic.preprocess import process
    try:
        from gamebasic.vm_native import VM as NativeVM  # type: ignore
    except ImportError:
        NativeVM = None

    path = _EXAMPLES / f"{rel}.gb"
    source = path.read_text(encoding="utf-8")
    prepped, _ = process(source, path.parent, file_label=path.name)
    tokens = Lexer(prepped).tokenize()
    ast = Parser(tokens).parse()

    saved_cwd = os.getcwd()
    nv_out = None
    try:
        os.chdir(path.parent)
        tw_buf = io.StringIO()
        with contextlib.redirect_stdout(tw_buf):
            Interpreter().run(ast)

        module = Compiler().compile(ast)
        vm_buf = io.StringIO()
        with contextlib.redirect_stdout(vm_buf):
            VM().run(module)

        if NativeVM is not None:
            nv_buf = io.StringIO()
            with contextlib.redirect_stdout(nv_buf):
                NativeVM().run(module)
            nv_out = nv_buf.getvalue()
    finally:
        os.chdir(saved_cwd)

    return tw_buf.getvalue(), vm_buf.getvalue(), nv_out


@pytest.mark.parametrize("name", _DETERMINISTIC)
def test_deterministic_example_treewalker_eq_vm(name):
    tw, vm, nv = _run_example(name)
    assert tw == vm, f"Tree-Walker und Python-VM divergieren bei {name}"
    if nv is not None:
        assert tw == nv, f"Tree-Walker und Native-VM divergieren bei {name}"


@pytest.mark.parametrize("name", _NON_DETERMINISTIC)
def test_nondeterministic_example_runs(name):
    """Nur pruefen dass es nicht crasht - Outputs werden nicht verglichen."""
    tw, vm, nv = _run_example(name)
    assert tw != ""  # mindestens irgendein Output
    assert vm != ""
    if nv is not None:
        assert nv != ""
