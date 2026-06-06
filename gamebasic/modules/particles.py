"""Partikel-System fuer GameBasic (NumPy-vektorisiert).

Built-ins (System-Lifecycle):
    PARTICLE_SYSTEM_NEW(x, y)            -> PARTICLE_SYSTEM
    PARTICLE_SET_POS(sys, x, y)
    PARTICLE_CLEAR(sys)
    PARTICLE_COUNT(sys)                  -> INTEGER

Konfiguration (sinnvolle Defaults gesetzt - nur ueberschreiben was noetig):
    PARTICLE_SET_VELOCITY(sys, vx_min, vx_max, vy_min, vy_max)
    PARTICLE_SET_LIFETIME(sys, ms_min, ms_max)
    PARTICLE_SET_GRAVITY(sys, gx, gy)
    PARTICLE_SET_COLOR(sys, color)
    PARTICLE_SET_SIZE(sys, px_min, px_max)
    PARTICLE_SET_FADE(sys, fade)         ' Helligkeit nimmt ueber Alter ab

Im Game-Loop:
    PARTICLE_EMIT(sys, count)            ' count Partikel ausstossen
    PARTICLE_UPDATE(sys, dt_ms)          ' Physik
    PARTICLE_DRAW(sys)                   ' Zeichnen (braucht SCREEN)

Velocity ist Pixel/Sekunde. Gravity ist Pixel/Sekunde**2. Lifetime ist ms.
Defaults: Partikel fliegen leicht nach oben mit Streuung, fallen durch
Schwerkraft (200 px/s**2), Lebensdauer 500-1000ms, Groesse 2-4 px, weiss,
mit Fade.

Implementierung: Position/Velocity/Lifetime/Age/Size/Color werden als
NumPy-Arrays gehalten. emit() konkateniert vektorisiert, update() macht
Aging/Filter/Gravity/Position-Integration in Bulk-Operationen. Bei vielen
Partikeln (>500) ist das Groessenordnungen schneller als Python-Loops.
"""
from __future__ import annotations

from ..builtins_registry import builtin, graphics_builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type
# Die pure Sim-Klasse lebt jetzt in gamebasic/particle_sim.py (extrahiert, damit
# der Partikel-Editor sie ohne die Builtin-Registry nutzen kann -- Stufe B).
from ..particle_sim import _ParticleSystem


register_type("particle_system", _ParticleSystem)


# --- Helfer ----------------------------------------------------------

def _check_sys(v, fn: str) -> _ParticleSystem:
    if not isinstance(v, _ParticleSystem):
        raise TypeMismatchError(f"{fn} erwartet PARTICLE_SYSTEM")
    return v


def _check_num(v, fn: str, name: str) -> float:
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn}: {name} muss Zahl sein")
    return float(v)


def _check_int(v, fn: str, name: str) -> int:
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn}: {name} muss INTEGER sein")
    return v


# --- System-Lifecycle ------------------------------------------------

@builtin("PARTICLE_SYSTEM_NEW", arity=2)
def _new(x, y):
    x = _check_num(x, "PARTICLE_SYSTEM_NEW", "x")
    y = _check_num(y, "PARTICLE_SYSTEM_NEW", "y")
    return _ParticleSystem(x, y)


@builtin("PARTICLE_SET_POS", arity=3)
def _set_pos(sys, x, y):
    sys = _check_sys(sys, "PARTICLE_SET_POS")
    sys.x = _check_num(x, "PARTICLE_SET_POS", "x")
    sys.y = _check_num(y, "PARTICLE_SET_POS", "y")
    return None


@builtin("PARTICLE_COUNT", arity=1)
def _count(sys):
    sys = _check_sys(sys, "PARTICLE_COUNT")
    return sys.count()


@builtin("PARTICLE_CLEAR", arity=1)
def _clear(sys):
    sys = _check_sys(sys, "PARTICLE_CLEAR")
    sys.clear()
    return None


# --- Konfiguration ---------------------------------------------------

@builtin("PARTICLE_SET_VELOCITY", arity=5)
def _set_velocity(sys, vx_min, vx_max, vy_min, vy_max):
    sys = _check_sys(sys, "PARTICLE_SET_VELOCITY")
    sys.vx_min = _check_num(vx_min, "PARTICLE_SET_VELOCITY", "vx_min")
    sys.vx_max = _check_num(vx_max, "PARTICLE_SET_VELOCITY", "vx_max")
    sys.vy_min = _check_num(vy_min, "PARTICLE_SET_VELOCITY", "vy_min")
    sys.vy_max = _check_num(vy_max, "PARTICLE_SET_VELOCITY", "vy_max")
    if sys.vx_max < sys.vx_min or sys.vy_max < sys.vy_min:
        raise GBRuntimeError("PARTICLE_SET_VELOCITY: max muss >= min sein")
    return None


@builtin("PARTICLE_SET_LIFETIME", arity=3)
def _set_lifetime(sys, ms_min, ms_max):
    sys = _check_sys(sys, "PARTICLE_SET_LIFETIME")
    ms_min = _check_int(ms_min, "PARTICLE_SET_LIFETIME", "ms_min")
    ms_max = _check_int(ms_max, "PARTICLE_SET_LIFETIME", "ms_max")
    if ms_min < 0 or ms_max < ms_min:
        raise GBRuntimeError(
            "PARTICLE_SET_LIFETIME: ms_min >= 0 und ms_max >= ms_min noetig"
        )
    sys.lifetime_min = ms_min
    sys.lifetime_max = ms_max
    return None


@builtin("PARTICLE_SET_GRAVITY", arity=3)
def _set_gravity(sys, gx, gy):
    sys = _check_sys(sys, "PARTICLE_SET_GRAVITY")
    sys.gravity_x = _check_num(gx, "PARTICLE_SET_GRAVITY", "gx")
    sys.gravity_y = _check_num(gy, "PARTICLE_SET_GRAVITY", "gy")
    return None


@builtin("PARTICLE_SET_COLOR", arity=2, types=("any", "int"))
def _set_color(sys, color):
    sys = _check_sys(sys, "PARTICLE_SET_COLOR")
    if color < 0 or color > 0xFFFFFF:
        raise GBRuntimeError("PARTICLE_SET_COLOR: Farbe muss 0..0xFFFFFF sein")
    sys.color = color
    return None


@builtin("PARTICLE_SET_SIZE", arity=3)
def _set_size(sys, smin, smax):
    sys = _check_sys(sys, "PARTICLE_SET_SIZE")
    smin = _check_int(smin, "PARTICLE_SET_SIZE", "min")
    smax = _check_int(smax, "PARTICLE_SET_SIZE", "max")
    if smin < 1 or smax < smin:
        raise GBRuntimeError("PARTICLE_SET_SIZE: min >= 1 und max >= min noetig")
    sys.size_min = smin
    sys.size_max = smax
    return None


@builtin("PARTICLE_SET_FADE", arity=2, types=("any", "bool"))
def _set_fade(sys, fade):
    sys = _check_sys(sys, "PARTICLE_SET_FADE")
    sys.fade = fade
    return None


_MODES = ("circle", "pixel", "square", "streak", "glow")


@builtin("PARTICLE_SET_MODE", arity=2, types=("any", "str"))
def _set_mode(sys, mode):
    """Render-Modus:
        "circle" - gefuellte Kreise (Default)
        "pixel"  - einzelne Pixel (sehr schnell, Bulk-Plot; dichte Wolken)
        "square" - gefuellte Quadrate (Voxel/Bloecke)
        "streak" - Striche entlang der Flugrichtung (Regen, Funken, Warp)
        "glow"   - additive Leucht-Blobs (Feuer, Magie, Energie)
    """
    sys = _check_sys(sys, "PARTICLE_SET_MODE")
    key = mode.lower()
    if key not in _MODES:
        raise GBRuntimeError(
            f"PARTICLE_SET_MODE: unbekannter Modus '{mode}' "
            f"(erlaubt: {', '.join(_MODES)})"
        )
    sys.mode = key
    return None


@builtin("PARTICLE_SET_COLOR_END", arity=2, types=("any", "int"))
def _set_color_end(sys, color):
    """Zweite Farbe: Partikel interpolieren ueber ihre Lebenszeit von der
    Start-Farbe (PARTICLE_SET_COLOR) zu dieser End-Farbe. Erzeugt z.B.
    Feuer (gelb -> rot) oder Plasma (weiss -> blau). Mit PARTICLE_SET_FADE
    kombinierbar (zusaetzliches Abdunkeln am Lebensende)."""
    sys = _check_sys(sys, "PARTICLE_SET_COLOR_END")
    if color < 0 or color > 0xFFFFFF:
        raise GBRuntimeError(
            "PARTICLE_SET_COLOR_END: Farbe muss 0..0xFFFFFF sein")
    sys.color_end = color
    sys.has_color_end = True
    return None


# --- Game-Loop -------------------------------------------------------

@builtin("PARTICLE_EMIT", arity=2, types=("any", "int"))
def _emit(sys, count):
    sys = _check_sys(sys, "PARTICLE_EMIT")
    if count < 0:
        raise GBRuntimeError("PARTICLE_EMIT: count muss >= 0 sein")
    sys.emit(count)
    return None


@builtin("PARTICLE_UPDATE", arity=2, types=("any", "int"))
def _update(sys, dt_ms):
    sys = _check_sys(sys, "PARTICLE_UPDATE")
    if dt_ms < 0:
        raise GBRuntimeError("PARTICLE_UPDATE: dt_ms muss >= 0 sein")
    sys.update(dt_ms)
    return None


@graphics_builtin("PARTICLE_DRAW", arity=1)
def _draw(g, sys):
    sys = _check_sys(sys, "PARTICLE_DRAW")
    # Rendering laeuft nur in der nativen Runtime (gbrt). Partikel-Logik
    # (EMIT/UPDATE/SET_*) bleibt im konsolen-only Tree-Walker nutzbar.
    raise GBRuntimeError(
        "PARTICLE_DRAW (Partikel-Rendering) laeuft nur in der nativen Runtime "
        "(gbrt) -- per F6 bzw. 'gbrun.py --native' starten.")
