"""Character-Controller-Toolkit fuer Platformer.

Implementiert die drei "feel-good"-Patterns, die ein Platformer-Spiel
von "geht ja" zu "fuehlt sich gut an" heben:

1. **Coyote-Time** -- nachdem der Spieler eine Klippe verlassen hat,
   bleibt sein Sprung-Recht noch ein paar Frames aktiv. Klassischer
   "ich habe doch noch gedrueckt"-Save.
2. **Jump-Buffering** -- wenn der Spieler kurz vor dem Boden Sprung
   drueckt, wird der Sprung beim Landen ausgefuehrt. Spart das "ich
   muss exakt im richtigen Frame druecken"-Frust.
3. **Variable-Jump-Height** -- Sprung loslassen vor dem Apex schneidet
   die nach-oben-Geschwindigkeit. Tippen = kleiner Sprung, gedrueckt
   halten = grosser Sprung. Mario-Style.

Diese drei zusammen sind das Standard-Repertoire jedes modernen
Platformer-Engines (Celeste/HK/Mario). Wer das richtig hinbekommt, hat
zur Haelfte ein gutes Gefuhl im Spiel.

API:
    CHAR_NEW(x, y, w, h) -> CHAR_CONTROLLER

    CHAR_SET_INPUT(c, axis_x, jump_pressed, jump_held)
    CHAR_UPDATE(c, map, layer_idx)         ' Tile-Kollision-Update

    CHAR_X(c) / CHAR_Y(c)                  ' Position
    CHAR_VX(c) / CHAR_VY(c)                ' Velocity
    CHAR_W(c) / CHAR_H(c)                  ' Box-Groesse
    CHAR_ON_GROUND(c)                      ' BOOLEAN
    CHAR_ON_WALL_LEFT(c) / RIGHT(c)        ' BOOLEAN (Wand beruehrt sich)
    CHAR_FACING(c)                         ' -1 (links) | 1 (rechts)

    CHAR_SET_POS(c, x, y)
    CHAR_SET_GRAVITY(c, g)
    CHAR_SET_MAX_FALL(c, max_vy)
    CHAR_SET_MOVE_SPEED(c, vx_max)
    CHAR_SET_JUMP_VELOCITY(c, jump_vy)     ' Wert ist positive Zahl; intern als -wert verwendet
    CHAR_SET_COYOTE_TIME(c, frames)
    CHAR_SET_JUMP_BUFFER(c, frames)
    CHAR_SET_VARIABLE_JUMP(c, enabled)
    CHAR_SET_VARIABLE_JUMP_CUT(c, factor)  ' bei Loslassen: vy *= factor (0..1)

Klassischer Game-Loop:

    INPUT_UPDATE()
    CHAR_SET_INPUT(player,
                   INPUT_AXIS("left", "right"),
                   INPUT_PRESSED("jump"),
                   INPUT_HELD("jump"))
    CHAR_UPDATE(player, level, 0)

    DRAWIMAGE(sprite, CHAR_X(player), CHAR_Y(player))
"""
from __future__ import annotations

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type

# Lazy zur Vermeidung von Zirkularitaet
from .tile_collide import _sweep_axis, _check_map_args


# --- Controller-State -----------------------------------------------

class _CharController:
    __slots__ = (
        "x", "y", "w", "h", "vx", "vy",
        "on_ground", "on_wall_left", "on_wall_right",
        "facing",
        # Konfiguration
        "move_speed", "jump_velocity", "gravity", "max_fall",
        "coyote_max", "jump_buffer_max",
        "variable_jump", "variable_jump_cut",
        # Interne State-Counter
        "_coyote_counter", "_jump_buffer_counter",
        "_prev_jump_held", "_was_jumping",
        # Aktueller Frame-Input (per CHAR_SET_INPUT gesetzt)
        "_in_axis_x", "_in_jump_pressed", "_in_jump_held",
    )

    def __init__(self, x, y, w, h):
        self.x = float(x)
        self.y = float(y)
        self.w = float(w)
        self.h = float(h)
        self.vx = 0.0
        self.vy = 0.0
        self.on_ground = False
        self.on_wall_left = False
        self.on_wall_right = False
        self.facing = 1                  # 1 = rechts, -1 = links

        # Defaults: anstaendige Platformer-Werte fuer 16x16-Tiles.
        # User kann alle ueber CHAR_SET_*-Builtins anpassen.
        self.move_speed = 2.0
        self.jump_velocity = 6.0         # gespeichert als positive Zahl
        self.gravity = 0.25
        self.max_fall = 7.0
        self.coyote_max = 6              # Frames "gnaedige Klippe"
        self.jump_buffer_max = 6         # Frames "schon gedrueckt"
        self.variable_jump = True
        self.variable_jump_cut = 0.5     # bei Loslassen: vy *= 0.5

        self._coyote_counter = 0
        self._jump_buffer_counter = 0
        self._prev_jump_held = False
        self._was_jumping = False

        self._in_axis_x = 0
        self._in_jump_pressed = False
        self._in_jump_held = False

    def __repr__(self):
        return (f"<CHAR_CONTROLLER pos=({self.x:.1f}, {self.y:.1f}) "
                f"vel=({self.vx:.1f}, {self.vy:.1f}) "
                f"ground={self.on_ground}>")


register_type("char_controller", _CharController)


# --- Helpers ------------------------------------------------------

def _check_ctrl(v, fn: str) -> _CharController:
    if not isinstance(v, _CharController):
        raise TypeMismatchError(f"{fn} erwartet CHAR_CONTROLLER")
    return v


def _check_num(v, fn: str, label: str = "Zahl") -> float:
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn}: {label} muss Zahl sein")
    return float(v)


# --- Konstruktor + Konfiguration ----------------------------------

@builtin("CHAR_NEW", arity=4, types=("num", "num", "num", "num"))
def _b_new(x, y, w, h):
    """Erzeugt einen Character-Controller an Pixel-Position (x, y) mit
    Box-Groesse (w, h). Defaults sind anstaendige Platformer-Werte
    fuer 16x16-Tiles -- per CHAR_SET_*-Builtins ueberschreibbar.
    """
    if w <= 0 or h <= 0:
        raise GBRuntimeError("CHAR_NEW: w und h muessen > 0 sein")
    return _CharController(x, y, w, h)


@builtin("CHAR_SET_POS", arity=3, types=("any", "num", "num"))
def _b_set_pos(c, x, y):
    c = _check_ctrl(c, "CHAR_SET_POS")
    c.x = float(x)
    c.y = float(y)
    return None


@builtin("CHAR_SET_MOVE_SPEED", arity=2, types=("any", "num"))
def _b_set_speed(c, v):
    c = _check_ctrl(c, "CHAR_SET_MOVE_SPEED")
    c.move_speed = max(0.0, float(v))
    return None


@builtin("CHAR_SET_GRAVITY", arity=2, types=("any", "num"))
def _b_set_gravity(c, g):
    c = _check_ctrl(c, "CHAR_SET_GRAVITY")
    c.gravity = float(g)
    return None


@builtin("CHAR_SET_MAX_FALL", arity=2, types=("any", "num"))
def _b_set_max_fall(c, v):
    c = _check_ctrl(c, "CHAR_SET_MAX_FALL")
    c.max_fall = max(0.0, float(v))
    return None


@builtin("CHAR_SET_JUMP_VELOCITY", arity=2, types=("any", "num"))
def _b_set_jump_velocity(c, v):
    """Spring-Geschwindigkeit als positive Zahl. Intern wird sie negiert
    angewandt (Y-Achse nach unten = positiv)."""
    c = _check_ctrl(c, "CHAR_SET_JUMP_VELOCITY")
    c.jump_velocity = abs(float(v))
    return None


@builtin("CHAR_SET_COYOTE_TIME", arity=2, types=("any", "int"))
def _b_set_coyote(c, frames):
    c = _check_ctrl(c, "CHAR_SET_COYOTE_TIME")
    c.coyote_max = max(0, int(frames))
    return None


@builtin("CHAR_SET_JUMP_BUFFER", arity=2, types=("any", "int"))
def _b_set_buffer(c, frames):
    c = _check_ctrl(c, "CHAR_SET_JUMP_BUFFER")
    c.jump_buffer_max = max(0, int(frames))
    return None


@builtin("CHAR_SET_VARIABLE_JUMP", arity=2, types=("any", "bool"))
def _b_set_variable_jump(c, enabled):
    c = _check_ctrl(c, "CHAR_SET_VARIABLE_JUMP")
    c.variable_jump = bool(enabled)
    return None


@builtin("CHAR_SET_VARIABLE_JUMP_CUT", arity=2, types=("any", "num"))
def _b_set_jump_cut(c, factor):
    """Wenn der Spieler Sprung loslaesst waehrend er nach oben fliegt:
    vy = vy * factor. 0.0 = vollstaendig stoppen (extrem 'tap-jump'),
    0.5 = halbieren (Mario-Standard), 1.0 = kein Variable-Jump (off)."""
    c = _check_ctrl(c, "CHAR_SET_VARIABLE_JUMP_CUT")
    f = float(factor)
    if f < 0.0:
        f = 0.0
    elif f > 1.0:
        f = 1.0
    c.variable_jump_cut = f
    return None


# --- Frame-Input + Update -----------------------------------------

@builtin("CHAR_SET_INPUT", arity=4, types=("any", "int", "bool", "bool"))
def _b_set_input(c, axis_x, jump_pressed, jump_held):
    """Setzt den Input fuer diesen Frame.

    Args:
        axis_x: -1 (links), 0, +1 (rechts). Klassisch aus INPUT_AXIS.
        jump_pressed: TRUE wenn Sprung-Taste gerade GEDRUECKT wurde
                      (Edge-Detection, aus INPUT_PRESSED).
        jump_held: TRUE wenn Sprung-Taste aktuell GEHALTEN wird
                   (fuer Variable-Jump, aus INPUT_HELD).
    """
    c = _check_ctrl(c, "CHAR_SET_INPUT")
    ax = int(axis_x)
    if ax < -1:
        ax = -1
    elif ax > 1:
        ax = 1
    c._in_axis_x = ax
    c._in_jump_pressed = bool(jump_pressed)
    c._in_jump_held = bool(jump_held)
    return None


@builtin("CHAR_UPDATE", arity=3, types=("any", "any", "int"))
def _b_update(c, m, layer_idx):
    """Fuehrt einen Frame-Update aus: Velocity berechnen, Tile-Kollision
    aufloesen, Position + Status-Flags aktualisieren.

    Args:
        c: CHAR_CONTROLLER
        m: TILED_MAP
        layer_idx: Index der Tile-Layer mit der Collision-Geometrie.
    """
    c = _check_ctrl(c, "CHAR_UPDATE")
    m, layer = _check_map_args(m, layer_idx, "CHAR_UPDATE")

    # 1. Coyote-Counter aktualisieren
    if c.on_ground:
        c._coyote_counter = c.coyote_max
    elif c._coyote_counter > 0:
        c._coyote_counter -= 1

    # 2. Jump-Buffer SETZEN, falls Press. Decrement passiert am Frame-Ende
    #    nach jump-try -- damit der Buffer in der gleichen Frame ueberlebt,
    #    in der wir landen.
    if c._in_jump_pressed:
        c._jump_buffer_counter = c.jump_buffer_max

    # 3. Variable-Jump-Cut: wenn der Spieler Sprung losgelassen hat
    #    waehrend er nach OBEN fliegt, schneide vy. Muss VOR Gravitation +
    #    Sweep passieren, damit der Cut tatsaechlich auf die Bewegung
    #    dieses Frames wirkt.
    if c.variable_jump and c._was_jumping:
        if c._prev_jump_held and not c._in_jump_held and c.vy < 0.0:
            c.vy *= c.variable_jump_cut
            c._was_jumping = False
    c._prev_jump_held = c._in_jump_held

    # 4. Horizontale Geschwindigkeit aus Input. Bei loslassen wird das
    #    horizontale Movement abrupt 0 -- typisches Mario/Celeste-Pattern
    #    (kein Schlittern), simpel und gut steuerbar. Wer wollen, kann
    #    Friction selbst aufsetzen, indem er vx zwischen Frames mitnimmt.
    c.vx = c._in_axis_x * c.move_speed
    if c._in_axis_x > 0:
        c.facing = 1
    elif c._in_axis_x < 0:
        c.facing = -1

    # 5. Gravitation + Terminal-Velocity
    c.vy += c.gravity
    if c.vy > c.max_fall:
        c.vy = c.max_fall

    # 6. Kollision: X-Sweep zuerst, dann Y mit dem neuen X. Klassisches
    #    separat-Achsen-Pattern (siehe tile_collide-Modul).
    new_x, hit_x = _sweep_axis(m, layer, c.x, c.y, c.w, c.h, c.vx, axis=0)
    c.x = new_x
    if hit_x:
        c.vx = 0.0

    new_y, hit_y = _sweep_axis(m, layer, c.x, c.y, c.w, c.h, c.vy, axis=1)
    c.y = new_y
    if hit_y:
        if c.vy > 0.0:
            c.on_ground = True
            c._was_jumping = False    # gelandet -> Jump-State reset
        c.vy = 0.0
    else:
        c.on_ground = False

    # 7. Sprung-Try -- KRITISCH: nach dem Y-Sweep, damit der gerade
    #    erst gesetzte on_ground-State greift. Sonst feuert das Jump-
    #    Buffering NICHT, wenn der Spieler in genau diesem Frame landet.
    #    Wenn der Sprung jetzt feuert, wird vy negativ; erst der naechste
    #    Frame bewegt den Spieler tatsaechlich nach oben.
    can_jump = c.on_ground or c._coyote_counter > 0
    if c._jump_buffer_counter > 0 and can_jump:
        c.vy = -c.jump_velocity
        c.on_ground = False
        c._coyote_counter = 0
        c._jump_buffer_counter = 0
        c._was_jumping = True

    # 8. Wand-Detection -- 1-px-Probe an den Seiten
    c.on_wall_left = False
    c.on_wall_right = False
    if not c.on_ground:
        _, ph_l = _sweep_axis(m, layer, c.x, c.y, c.w, c.h, -1.0, axis=0)
        c.on_wall_left = ph_l
        _, ph_r = _sweep_axis(m, layer, c.x, c.y, c.w, c.h, 1.0, axis=0)
        c.on_wall_right = ph_r

    # 9. Jump-Buffer DECREMENT (am Frame-Ende, nach jump-try). So bleibt
    #    der Buffer in der Landing-Frame noch verfuegbar fuer den Sprung.
    if not c._in_jump_pressed and c._jump_buffer_counter > 0:
        c._jump_buffer_counter -= 1

    return None


# --- Lese-Accessoren ----------------------------------------------

@builtin("CHAR_X", arity=1)
def _b_x(c):
    return _check_ctrl(c, "CHAR_X").x


@builtin("CHAR_Y", arity=1)
def _b_y(c):
    return _check_ctrl(c, "CHAR_Y").y


@builtin("CHAR_W", arity=1)
def _b_w(c):
    return _check_ctrl(c, "CHAR_W").w


@builtin("CHAR_H", arity=1)
def _b_h(c):
    return _check_ctrl(c, "CHAR_H").h


@builtin("CHAR_VX", arity=1)
def _b_vx(c):
    return _check_ctrl(c, "CHAR_VX").vx


@builtin("CHAR_VY", arity=1)
def _b_vy(c):
    return _check_ctrl(c, "CHAR_VY").vy


@builtin("CHAR_ON_GROUND", arity=1)
def _b_on_ground(c):
    return _check_ctrl(c, "CHAR_ON_GROUND").on_ground


@builtin("CHAR_ON_WALL_LEFT", arity=1)
def _b_on_wall_left(c):
    return _check_ctrl(c, "CHAR_ON_WALL_LEFT").on_wall_left


@builtin("CHAR_ON_WALL_RIGHT", arity=1)
def _b_on_wall_right(c):
    return _check_ctrl(c, "CHAR_ON_WALL_RIGHT").on_wall_right


@builtin("CHAR_FACING", arity=1)
def _b_facing(c):
    return _check_ctrl(c, "CHAR_FACING").facing


# --- Velocity-Override (selten gebraucht, aber praktisch fuer Knockback etc.)

@builtin("CHAR_SET_VX", arity=2, types=("any", "num"))
def _b_set_vx(c, v):
    c = _check_ctrl(c, "CHAR_SET_VX")
    c.vx = float(v)
    return None


@builtin("CHAR_SET_VY", arity=2, types=("any", "num"))
def _b_set_vy(c, v):
    c = _check_ctrl(c, "CHAR_SET_VY")
    c.vy = float(v)
    return None
