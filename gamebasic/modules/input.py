"""Input-Mapping-Modul fuer GameBasic.

Statt im Spiel ueberall hardcoded Keycodes (`KEYPRESSED(1073741904)`) zu
streuen, gruppiert das input-Modul Tasten zu **Actions**. Jede Action
kann an mehrere Tasten oder Joystick-Buttons gebunden werden -- etwa
WASD + Pfeiltasten + Gamepad-DPad.

Typischer Flow pro Frame:
  1. INPUT_UPDATE()                 ' Snapshot der gedrueckten Tasten
  2. IF INPUT_PRESSED("jump") ...   ' Edge: gerade JETZT gedrueckt
  3. IF INPUT_HELD("move_left") ... ' Held: dauerhaft gedrueckt
  4. FLIP() / SLEEP() etc.

Edge-Detection (`PRESSED`/`RELEASED`) erfordert den `INPUT_UPDATE()`-Call
am Frame-Start -- das Modul vergleicht den aktuellen Snapshot mit dem
vorherigen.

Built-ins:
  INPUT_BIND(action, key1, [key2], [key3], ...)   ' Action -> Keys-Liste
  INPUT_UNBIND(action)                            ' Action loeschen
  INPUT_UPDATE()                                  ' Frame-Snapshot
  INPUT_HELD(action) AS BOOLEAN                   ' Aktuell gedrueckt
  INPUT_PRESSED(action) AS BOOLEAN                ' Edge: jetzt erst
  INPUT_RELEASED(action) AS BOOLEAN               ' Edge: gerade losgelassen
  INPUT_AXIS(neg_action, pos_action) AS INTEGER   ' -1 / 0 / 1 als virtual axis
"""
from __future__ import annotations

from ..builtins_registry import builtin, graphics_builtin
from ..errors import GBRuntimeError, TypeMismatchError


# Modul-State -- pro Process global. Tests setzen ihn zwischen Tests
# zurueck via INPUT_RESET().
_state = {
    "actions": {},        # action-name (str, lower) -> list[int] (keycodes)
    "prev_keys": set(),   # set[int]
    "cur_keys": set(),    # set[int]
}


def _norm_action(name: str) -> str:
    if not isinstance(name, str):
        raise TypeMismatchError(f"Action-Name muss STRING sein, nicht {type(name).__name__}")
    if not name:
        raise GBRuntimeError("Action-Name darf nicht leer sein")
    return name.lower()


# --- Bind / Unbind / Reset ---------------------------------------------

@builtin("INPUT_BIND", arity=(2, None))
def _b_bind(*args):
    """INPUT_BIND(action, key1, [key2], ...) -- mind. ein Key.
    Re-Binding ueberschreibt die alte Liste (idempotent).
    """
    action = _norm_action(args[0])
    keys = []
    for k in args[1:]:
        if isinstance(k, bool) or not isinstance(k, int):
            raise TypeMismatchError(f"INPUT_BIND: Key muss INTEGER sein, nicht {type(k).__name__}")
        keys.append(k)
    _state["actions"][action] = keys
    return None


@builtin("INPUT_UNBIND", arity=1, types=("str",))
def _b_unbind(action):
    _state["actions"].pop(_norm_action(action), None)
    return None


@builtin("INPUT_RESET", arity=0)
def _b_reset():
    """Alle Bindings UND Snapshot loeschen. Hauptsaechlich fuer Tests
    -- in Spielen einmal beim Scene-Wechsel sinnvoll, wenn die Eingaben
    sauber neu konfiguriert werden sollen."""
    _state["actions"].clear()
    _state["prev_keys"].clear()
    _state["cur_keys"].clear()
    return None


# --- Frame-Update + State-Abfrage --------------------------------------

@graphics_builtin("INPUT_UPDATE", arity=0)
def _g_update(g):
    """Verschiebt den aktuellen Snapshot nach prev_keys und liest die jetzt
    gedrueckten Keys von der Graphics-Schicht ein. Einmal pro Frame
    aufrufen -- VOR den anderen INPUT_*-Aufrufen.

    Inkludiert sowohl Tastatur-Keys (positive Codes) als auch
    Joystick-Buttons/-DPad (negative Codes -100..-203). Mehrere Pads
    sammeln sich in dieselbe Menge -- Multi-Player-Setups, die separate
    Pads ansprechen, sollten INPUT_BIND mit pad-spezifischen Actions
    bauen (v2-Feature).
    """
    cur_keys = set(g.keys_pressed())
    _poll_joysticks_into(cur_keys)
    _state["prev_keys"] = _state["cur_keys"]
    _state["cur_keys"] = cur_keys
    return None


# --- Joystick-Polling ------------------------------------------------

# Joystick-Codes (parallel zur graphics.KEYS-Konvention):
_JOY_BUTTON_BASE = -100
_JOY_BUTTON_MAP = {     # pygame-button-index -> negativer Action-Code
    0: -100,            # A
    1: -101,            # B
    2: -102,            # X
    3: -103,            # Y
    4: -104,            # LB
    5: -105,            # RB
    6: -106,            # Back
    7: -107,            # Start
    8: -108,            # L-Stick click
    9: -109,            # R-Stick click
}
# DPad ("hat") -> negative Codes
_JOY_DPAD_UP    = -200
_JOY_DPAD_DOWN  = -201
_JOY_DPAD_LEFT  = -202
_JOY_DPAD_RIGHT = -203

# Joystick-Instanz-Cache. pygame.joystick.Joystick(i) braucht .init(),
# danach kann man buttons/achsen pollen. Cache pro Index.
_JOYSTICKS: list = []      # list[pygame.joystick.Joystick] -- initialisiert lazily


def _ensure_joysticks():
    """No-op im konsolen-only Tree-Walker: echtes Gamepad-Polling gibt es nur in
    der nativen Runtime (gbrt, raylib). `_JOYSTICKS` bleibt leer (Getter liefern
    0/""/0.0). Tests injizieren Stub-Pads direkt in `_JOYSTICKS`."""
    return


def _poll_joysticks_into(out_set: set):
    """Erweitert out_set um die gerade gedrueckten Joystick-Buttons und
    DPad-Richtungen. Achsen sind separat via INPUT_JOY_AXIS abrufbar
    -- die passen nicht in eine bool-Menge."""
    _ensure_joysticks()
    if not _JOYSTICKS:
        return
    for j in _JOYSTICKS:
        # Buttons
        try:
            n_btn = j.get_numbuttons()
        except Exception:
            continue
        for btn_idx in range(min(n_btn, 10)):   # nur die ersten 10 mappen
            code = _JOY_BUTTON_MAP.get(btn_idx)
            if code is None:
                continue
            try:
                if j.get_button(btn_idx):
                    out_set.add(code)
            except Exception:
                pass
        # DPad (Hat 0) -> 4 Buttons
        try:
            if j.get_numhats() > 0:
                hx, hy = j.get_hat(0)
                if hx < 0:
                    out_set.add(_JOY_DPAD_LEFT)
                elif hx > 0:
                    out_set.add(_JOY_DPAD_RIGHT)
                if hy > 0:                  # pygame: y > 0 = nach OBEN
                    out_set.add(_JOY_DPAD_UP)
                elif hy < 0:
                    out_set.add(_JOY_DPAD_DOWN)
        except Exception:
            pass


# --- Joystick-Public-API --------------------------------------------

@builtin("INPUT_JOY_COUNT", arity=0)
def _b_joy_count():
    """Wie viele Pads sind aktuell angeschlossen? 0 wenn keiner oder
    pygame-mixer nicht initialisiert."""
    _ensure_joysticks()
    return len(_JOYSTICKS)


@builtin("INPUT_JOY_NAME", arity=1, types=("int",))
def _b_joy_name(idx):
    """Name des Pads im Slot `idx`. Leer wenn nicht angeschlossen."""
    _ensure_joysticks()
    if idx < 0 or idx >= len(_JOYSTICKS):
        return ""
    try:
        return _JOYSTICKS[idx].get_name()
    except Exception:
        return ""


_DEFAULT_DEADZONE = 0.15


@builtin("INPUT_JOY_AXIS", arity=2, types=("int", "str"))
def _b_joy_axis(pad_idx, axis_name):
    """Liest eine analoge Achse: -1.0..+1.0 (mit Deadzone).

    axis_name (case-insensitive):
        "left_x", "left_y"      Linker Stick (Y: oben = -1, unten = +1)
        "right_x", "right_y"    Rechter Stick
        "lt", "rt"              Trigger (0..+1, in Ruhelage 0)

    Mapping nach pygame-Achsen-Konvention (Xbox-Layout):
        0 = left_x,  1 = left_y,  2 = right_x bzw. lt (Controller-abhaengig),
        3 = right_y, 4 = lt, 5 = rt.
    """
    _ensure_joysticks()
    if pad_idx < 0 or pad_idx >= len(_JOYSTICKS):
        return 0.0
    j = _JOYSTICKS[pad_idx]
    name = axis_name.lower()
    axis_map = {
        "left_x":  0,
        "left_y":  1,
        "right_x": 2,
        "right_y": 3,
        "lt":      4,
        "rt":      5,
    }
    idx = axis_map.get(name)
    if idx is None:
        raise GBRuntimeError(
            f"INPUT_JOY_AXIS: unbekannte Achse '{axis_name}' "
            f"(erlaubt: left_x, left_y, right_x, right_y, lt, rt)"
        )
    try:
        if idx >= j.get_numaxes():
            return 0.0
        v = float(j.get_axis(idx))
    except Exception:
        return 0.0
    # Deadzone fuer Sticks -- Trigger lassen wir unangetastet (sie sind
    # in Ruhelage schon nahe 0 oder bei -1, je nach Pad).
    if name.startswith("left_") or name.startswith("right_"):
        if abs(v) < _DEFAULT_DEADZONE:
            return 0.0
    return v


@builtin("INPUT_HELD", arity=1, types=("str",))
def _b_held(action):
    keys = _state["actions"].get(_norm_action(action))
    if keys is None:
        raise GBRuntimeError(f"INPUT_HELD: Action '{action}' nicht gebunden")
    cur = _state["cur_keys"]
    return any(k in cur for k in keys)


@builtin("INPUT_PRESSED", arity=1, types=("str",))
def _b_pressed(action):
    """Edge-Detect: irgendein gebundener Key ist JETZT gedrueckt, war aber
    im vorigen Snapshot nicht gedrueckt."""
    keys = _state["actions"].get(_norm_action(action))
    if keys is None:
        raise GBRuntimeError(f"INPUT_PRESSED: Action '{action}' nicht gebunden")
    cur = _state["cur_keys"]
    prev = _state["prev_keys"]
    return any((k in cur and k not in prev) for k in keys)


@builtin("INPUT_RELEASED", arity=1, types=("str",))
def _b_released(action):
    """Edge-Detect: war im vorigen Snapshot gedrueckt, JETZT nicht mehr."""
    keys = _state["actions"].get(_norm_action(action))
    if keys is None:
        raise GBRuntimeError(f"INPUT_RELEASED: Action '{action}' nicht gebunden")
    cur = _state["cur_keys"]
    prev = _state["prev_keys"]
    return any((k in prev and k not in cur) for k in keys)


def _action_held(action: str) -> bool:
    """Helper -- ohne Decorator-Wrapper, fuer interne Calls."""
    keys = _state["actions"].get(_norm_action(action))
    if keys is None:
        raise GBRuntimeError(f"Action '{action}' nicht gebunden")
    cur = _state["cur_keys"]
    return any(k in cur for k in keys)


@builtin("INPUT_AXIS", arity=2, types=("str", "str"))
def _b_axis(neg_action, pos_action):
    """Virtual Axis: -1 wenn nur neg_action gedrueckt, +1 wenn nur
    pos_action, 0 sonst (auch wenn beide). Praktisch fuer
    Move-Left/Move-Right -> -1/0/+1 zu konvertieren.
    """
    neg = _action_held(neg_action)
    pos = _action_held(pos_action)
    if neg and not pos:
        return -1
    if pos and not neg:
        return 1
    return 0


@builtin("INPUT_BOUND", arity=1, types=("str",))
def _b_bound(action):
    """TRUE wenn Action gebunden ist (mit mind. 1 Key). Praktisch zum
    Pruefen, bevor man INPUT_HELD/PRESSED aufruft -- die werfen sonst.
    """
    keys = _state["actions"].get(_norm_action(action))
    return keys is not None and len(keys) > 0
