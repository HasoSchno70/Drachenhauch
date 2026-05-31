"""Tests fuer das input-Mapping-Modul.

Da Edge-Detection eine echte Graphics-Schicht braucht, manipulieren wir
fuer die meisten Tests den Modul-State direkt -- so vermeiden wir die
Pygame-Abhaengigkeit und testen die Action -> Keys-Mapping-Logik in
Isolation. Der Tree-Walker und der VM teilen sich denselben State, weil
die Built-ins als Modul-Singletons leben.
"""
import pytest


@pytest.fixture(autouse=True)
def reset_input_state():
    """Vor und nach jedem Test den Input-Modul-State leeren."""
    from gamebasic.modules import input as inp_mod
    inp_mod._state["actions"].clear()
    inp_mod._state["prev_keys"].clear()
    inp_mod._state["cur_keys"].clear()
    yield
    inp_mod._state["actions"].clear()
    inp_mod._state["prev_keys"].clear()
    inp_mod._state["cur_keys"].clear()


def test_bind_then_held_with_key_set(run_gb):
    """Direkt _state.cur_keys setzen, dann INPUT_HELD pruefen."""
    from gamebasic.modules import input as inp_mod
    out = run_gb('''
IMPORT "input"
INPUT_BIND("jump", KEY_SPACE)
PRINT INPUT_BOUND("jump")
''')
    assert out == "TRUE\n"
    inp_mod._state["cur_keys"] = {32}     # KEY_SPACE
    out2 = run_gb('''
IMPORT "input"
PRINT INPUT_HELD("jump")
''')
    # Note: INPUT_BIND ist persistent ueber die zwei run_gb-Calls, weil das
    # Modul-State auf Modulebene lebt.
    assert out2 == "TRUE\n"


def test_bind_multiple_keys(run_gb):
    from gamebasic.modules import input as inp_mod
    run_gb('''
IMPORT "input"
INPUT_BIND("move_left", KEY_LEFT, KEY_A)
''')
    inp_mod._state["cur_keys"] = {ord('a')}
    out = run_gb('IMPORT "input"\nPRINT INPUT_HELD("move_left")')
    assert out == "TRUE\n"
    inp_mod._state["cur_keys"] = {1073741904}     # KEY_LEFT
    out = run_gb('IMPORT "input"\nPRINT INPUT_HELD("move_left")')
    assert out == "TRUE\n"
    inp_mod._state["cur_keys"] = {ord('z')}
    out = run_gb('IMPORT "input"\nPRINT INPUT_HELD("move_left")')
    assert out == "FALSE\n"


def test_pressed_edge_detection(run_gb):
    from gamebasic.modules import input as inp_mod
    run_gb('''
IMPORT "input"
INPUT_BIND("jump", KEY_SPACE)
''')
    # Frame 1: Space neu gedrueckt
    inp_mod._state["prev_keys"] = set()
    inp_mod._state["cur_keys"] = {32}
    out = run_gb('IMPORT "input"\nPRINT INPUT_PRESSED("jump")\nPRINT INPUT_HELD("jump")')
    assert out == "TRUE\nTRUE\n"
    # Frame 2: Space weiter gehalten
    inp_mod._state["prev_keys"] = {32}
    inp_mod._state["cur_keys"] = {32}
    out = run_gb('IMPORT "input"\nPRINT INPUT_PRESSED("jump")\nPRINT INPUT_HELD("jump")')
    assert out == "FALSE\nTRUE\n"


def test_released_edge_detection(run_gb):
    from gamebasic.modules import input as inp_mod
    run_gb('IMPORT "input"\nINPUT_BIND("jump", KEY_SPACE)')
    # Frame: Space gerade losgelassen
    inp_mod._state["prev_keys"] = {32}
    inp_mod._state["cur_keys"] = set()
    out = run_gb('IMPORT "input"\nPRINT INPUT_RELEASED("jump")\nPRINT INPUT_HELD("jump")')
    assert out == "TRUE\nFALSE\n"


def test_unbind(run_gb):
    out = run_gb('''
IMPORT "input"
INPUT_BIND("jump", KEY_SPACE)
PRINT INPUT_BOUND("jump")
INPUT_UNBIND("jump")
PRINT INPUT_BOUND("jump")
''')
    assert out == "TRUE\nFALSE\n"


def test_held_unbound_throws(run_gb):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError):
        run_gb('IMPORT "input"\nPRINT INPUT_HELD("not_bound")')


def test_axis_negative(run_gb):
    from gamebasic.modules import input as inp_mod
    run_gb('''
IMPORT "input"
INPUT_BIND("left", KEY_A)
INPUT_BIND("right", KEY_D)
''')
    inp_mod._state["cur_keys"] = {ord('a')}
    out = run_gb('IMPORT "input"\nPRINT INPUT_AXIS("left", "right")')
    assert out == "-1\n"


def test_axis_positive(run_gb):
    from gamebasic.modules import input as inp_mod
    run_gb('''
IMPORT "input"
INPUT_BIND("left", KEY_A)
INPUT_BIND("right", KEY_D)
''')
    inp_mod._state["cur_keys"] = {ord('d')}
    out = run_gb('IMPORT "input"\nPRINT INPUT_AXIS("left", "right")')
    assert out == "1\n"


def test_axis_zero_when_both_or_neither(run_gb):
    from gamebasic.modules import input as inp_mod
    run_gb('''
IMPORT "input"
INPUT_BIND("left", KEY_A)
INPUT_BIND("right", KEY_D)
''')
    # Beide gedrueckt -> 0
    inp_mod._state["cur_keys"] = {ord('a'), ord('d')}
    out = run_gb('IMPORT "input"\nPRINT INPUT_AXIS("left", "right")')
    assert out == "0\n"
    # Keiner gedrueckt -> 0
    inp_mod._state["cur_keys"] = set()
    out = run_gb('IMPORT "input"\nPRINT INPUT_AXIS("left", "right")')
    assert out == "0\n"


def test_rebind_overrides(run_gb):
    """Re-BIND mit neuen Keys ueberschreibt die alte Liste."""
    from gamebasic.modules import input as inp_mod
    run_gb('IMPORT "input"\nINPUT_BIND("jump", KEY_SPACE)')
    run_gb('IMPORT "input"\nINPUT_BIND("jump", KEY_W)')
    # Space sollte jetzt nicht mehr triggern
    inp_mod._state["cur_keys"] = {32}
    out = run_gb('IMPORT "input"\nPRINT INPUT_HELD("jump")')
    assert out == "FALSE\n"
    # W sollte triggern
    inp_mod._state["cur_keys"] = {ord('w')}
    out = run_gb('IMPORT "input"\nPRINT INPUT_HELD("jump")')
    assert out == "TRUE\n"


def test_action_name_case_insensitive(run_gb):
    from gamebasic.modules import input as inp_mod
    run_gb('IMPORT "input"\nINPUT_BIND("Jump", KEY_SPACE)')
    inp_mod._state["cur_keys"] = {32}
    out = run_gb('IMPORT "input"\nPRINT INPUT_HELD("JUMP")')
    assert out == "TRUE\n"
    out = run_gb('IMPORT "input"\nPRINT INPUT_HELD("jump")')
    assert out == "TRUE\n"


def test_vm_path(run_vm):
    """Modul muss auch im VM-Pfad funktionieren."""
    from gamebasic.modules import input as inp_mod
    out = run_vm('''
IMPORT "input"
INPUT_BIND("fire", KEY_SPACE)
PRINT INPUT_BOUND("fire")
''')
    assert out == "TRUE\n"
    inp_mod._state["cur_keys"] = {32}
    out = run_vm('IMPORT "input"\nPRINT INPUT_HELD("fire")')
    assert out == "TRUE\n"
