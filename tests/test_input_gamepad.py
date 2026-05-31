"""Tests fuer die Gamepad-Erweiterung des input-Moduls.

Ohne echten Joystick: wir mocken die _JOYSTICKS-Liste mit einem
einfachen Stub-Object, das pygame.joystick.Joystick's API nachbildet.
So testen wir Polling, Buttons, DPad, Axis-Deadzone ohne Hardware.
"""
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError


@pytest.fixture(scope="module", autouse=True)
def _load_input():
    assert load_module("input")


class _MockPad:
    """Stub fuer pygame.joystick.Joystick. Setbar via Test-Helpers."""
    def __init__(self, name="Mock Pad"):
        self._name = name
        self.buttons = [0] * 16
        self.hats = [(0, 0)]
        self.axes = [0.0] * 6

    def init(self):
        pass

    def get_name(self):
        return self._name

    def get_numbuttons(self):
        return len(self.buttons)

    def get_button(self, idx):
        return self.buttons[idx]

    def get_numhats(self):
        return len(self.hats)

    def get_hat(self, idx):
        return self.hats[idx]

    def get_numaxes(self):
        return len(self.axes)

    def get_axis(self, idx):
        return self.axes[idx]


@pytest.fixture
def reset_input(call_builtin):
    """Reset Action-State + Joystick-Mock zwischen Tests."""
    from gamebasic.modules import input as input_mod
    call_builtin("input_reset", [])
    input_mod._JOYSTICKS.clear()
    yield
    call_builtin("input_reset", [])
    input_mod._JOYSTICKS.clear()


@pytest.fixture
def mock_pad(reset_input):
    """Ein gemockter Pad an Slot 0. Den globalen JOYSTICKS-Cache fuellen
    und _ensure_joysticks ausstellen, damit pygame's Joystick-Init nicht
    drueberbuegelt."""
    from gamebasic.modules import input as input_mod
    pad = _MockPad("Test Controller")
    input_mod._JOYSTICKS.append(pad)
    # ensure_joysticks neutralisieren: es wuerde den Cache leeren wenn
    # pygame keine echten Pads sieht
    orig_ensure = input_mod._ensure_joysticks
    input_mod._ensure_joysticks = lambda: None
    yield pad
    input_mod._ensure_joysticks = orig_ensure
    input_mod._JOYSTICKS.clear()


# --- INPUT_JOY_COUNT / NAME -------------------------------------

def test_joy_count_no_pads(call_builtin, reset_input):
    """Ohne Mock + ohne echtes Pad: Count = 0 (oder was pygame liefert)."""
    n = call_builtin("input_joy_count", [])
    # Auf CI/headless ist das immer 0. Auf Dev-Maschinen koennte ein
    # echter Pad angeschlossen sein -- wir akzeptieren n >= 0.
    assert n >= 0


def test_joy_count_with_mock(call_builtin, mock_pad):
    assert call_builtin("input_joy_count", []) == 1


def test_joy_name(call_builtin, mock_pad):
    assert call_builtin("input_joy_name", [0]) == "Test Controller"


def test_joy_name_out_of_bounds(call_builtin, mock_pad):
    assert call_builtin("input_joy_name", [99]) == ""


# --- Button-Polling --------------------------------------------

def test_button_a_bound_and_pressed(call_builtin, mock_pad):
    """JOY_BUTTON_A im Bind, dann auf dem Pad druecken -> HELD = TRUE."""
    call_builtin("input_bind", ["jump", -100])    # JOY_BUTTON_A = -100
    # Pad-Button drucken
    mock_pad.buttons[0] = 1
    # Update poll
    from gamebasic.modules import input as input_mod
    cur = set()
    input_mod._poll_joysticks_into(cur)
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_held", ["jump"]) is True


def test_button_not_pressed_when_released(call_builtin, mock_pad):
    call_builtin("input_bind", ["jump", -100])
    mock_pad.buttons[0] = 0
    from gamebasic.modules import input as input_mod
    cur = set()
    input_mod._poll_joysticks_into(cur)
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_held", ["jump"]) is False


def test_multi_button_bind(call_builtin, mock_pad):
    """JOY_BUTTON_A oder JOY_BUTTON_B triggert dieselbe Action."""
    call_builtin("input_bind", ["attack", -100, -101])  # A oder B
    mock_pad.buttons[1] = 1   # nur B
    from gamebasic.modules import input as input_mod
    cur = set()
    input_mod._poll_joysticks_into(cur)
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_held", ["attack"]) is True


def test_press_edge_detection(call_builtin, mock_pad):
    """INPUT_PRESSED muss Edge erkennen (war FALSE, ist jetzt TRUE)."""
    call_builtin("input_bind", ["jump", -100])
    from gamebasic.modules import input as input_mod
    # Frame 1: nicht gedrueckt
    mock_pad.buttons[0] = 0
    cur = set()
    input_mod._poll_joysticks_into(cur)
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_pressed", ["jump"]) is False
    # Frame 2: jetzt gedrueckt
    mock_pad.buttons[0] = 1
    cur = set()
    input_mod._poll_joysticks_into(cur)
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_pressed", ["jump"]) is True
    # Frame 3: noch gedrueckt -- PRESSED nicht mehr (kein Edge)
    cur = set()
    input_mod._poll_joysticks_into(cur)
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_pressed", ["jump"]) is False
    assert call_builtin("input_held", ["jump"]) is True


# --- DPad -----------------------------------------------------

def test_dpad_up(call_builtin, mock_pad):
    call_builtin("input_bind", ["up", -200])    # JOY_DPAD_UP
    mock_pad.hats[0] = (0, 1)                   # pygame: y>0 = oben
    from gamebasic.modules import input as input_mod
    cur = set()
    input_mod._poll_joysticks_into(cur)
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_held", ["up"]) is True


def test_dpad_diagonal(call_builtin, mock_pad):
    """DPad in 2 Richtungen gleichzeitig (rechts + oben)."""
    call_builtin("input_bind", ["right", -203])
    call_builtin("input_bind", ["up", -200])
    mock_pad.hats[0] = (1, 1)
    from gamebasic.modules import input as input_mod
    cur = set()
    input_mod._poll_joysticks_into(cur)
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_held", ["right"]) is True
    assert call_builtin("input_held", ["up"]) is True


# --- Mixed Tastatur + Pad ------------------------------------

def test_keyboard_and_gamepad_in_same_action(call_builtin, mock_pad):
    """Action ist sowohl an KEY_SPACE als auch an JOY_BUTTON_A gebunden."""
    call_builtin("input_bind", ["jump", 32, -100])   # KEY_SPACE + JOY_A
    # Druecke Joy-A, Tastatur leer
    mock_pad.buttons[0] = 1
    from gamebasic.modules import input as input_mod
    cur = set()
    input_mod._poll_joysticks_into(cur)
    # Keine Tastatur-Eingabe -- cur enthaelt nur Joy-Codes
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_held", ["jump"]) is True
    # Pad loslassen, Tastatur Space druecken
    mock_pad.buttons[0] = 0
    cur = {32}
    input_mod._state["prev_keys"] = input_mod._state["cur_keys"]
    input_mod._state["cur_keys"] = cur
    assert call_builtin("input_held", ["jump"]) is True


# --- INPUT_JOY_AXIS -----------------------------------------

def test_joy_axis_zero_when_no_pad(call_builtin, reset_input):
    """Ohne Pad: Achse immer 0.0."""
    assert call_builtin("input_joy_axis", [0, "left_x"]) == 0.0


def test_joy_axis_left_x(call_builtin, mock_pad):
    mock_pad.axes[0] = 0.5
    assert call_builtin("input_joy_axis", [0, "left_x"]) == pytest.approx(0.5)


def test_joy_axis_deadzone(call_builtin, mock_pad):
    """Werte innerhalb der Deadzone (|v| < 0.15) werden zu 0 geclamped."""
    mock_pad.axes[0] = 0.1
    assert call_builtin("input_joy_axis", [0, "left_x"]) == 0.0
    mock_pad.axes[0] = -0.1
    assert call_builtin("input_joy_axis", [0, "left_x"]) == 0.0
    mock_pad.axes[0] = 0.16   # gerade ausserhalb
    assert call_builtin("input_joy_axis", [0, "left_x"]) == pytest.approx(0.16)


def test_joy_axis_triggers_no_deadzone(call_builtin, mock_pad):
    """Triggers (lt/rt) bekommen keine Deadzone -- ihre Ruhelage ist
    Hardware-abhaengig (-1 oder 0)."""
    mock_pad.axes[4] = 0.05    # leichtes Antippen
    assert call_builtin("input_joy_axis", [0, "lt"]) == pytest.approx(0.05)


def test_joy_axis_unknown_name_errors(call_builtin, mock_pad):
    with pytest.raises(GBRuntimeError, match="unbekannte Achse"):
        call_builtin("input_joy_axis", [0, "weird_axis"])


def test_joy_axis_out_of_range_axis(call_builtin, mock_pad):
    """Pad hat nur 6 Achsen. right_y / rt sind 3, 5 -- ist OK. Aber
    ein Pad mit nur 2 Achsen -> hoehere Achsen geben 0.0."""
    # Set numaxes auf 2 -- nur left_x und left_y verfuegbar
    mock_pad.axes = [0.0, 0.0]
    assert call_builtin("input_joy_axis", [0, "right_x"]) == 0.0
    assert call_builtin("input_joy_axis", [0, "rt"]) == 0.0
