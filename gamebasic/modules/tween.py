"""Tweening-Modul fuer GameBasic - Werteinterpolation ueber Zeit.

Built-ins:
    TWEEN_NEW(start, end, dauer_ms[, easing$])         -> TWEEN  (one-shot)
    TWEEN_NEW_LOOP(start, end, dauer_ms[, easing$])    -> TWEEN  (forever)
    TWEEN_NEW_PINGPONG(start, end, dauer_ms[, easing]) -> TWEEN  (forever, hin und her)
    TWEEN_VALUE(t)        -> FLOAT     ' aktueller Wert (start..end)
    TWEEN_PROGRESS(t)     -> FLOAT     ' 0.0..1.0
    TWEEN_DONE(t)         -> BOOLEAN   ' immer FALSE bei loop/pingpong
    TWEEN_RESTART(t)
    TWEEN_PAUSE(t)
    TWEEN_RESUME(t)
    TWEEN_REVERSE(t)                    ' tauscht start/end, restartet
    TWEEN_EASINGS()       -> STRING     ' Liste aller Easings

Easing-Namen (case-insensitive):
    linear,
    in_quad, out_quad, inout_quad,
    in_cubic, out_cubic, inout_cubic,
    in_sine, out_sine, inout_sine,
    in_bounce, out_bounce, inout_bounce,
    in_elastic, out_elastic, inout_elastic,
    in_back, out_back, inout_back
"""
from __future__ import annotations

import math
import time

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type


_START = time.monotonic()


def _now_ms() -> int:
    return int((time.monotonic() - _START) * 1000)


# --- Easing-Funktionen (Robert-Penner-Stil, normalisiert auf t in [0,1]) ----

def _ease_linear(t): return t
def _ease_in_quad(t): return t * t
def _ease_out_quad(t): return 1 - (1 - t) ** 2
def _ease_inout_quad(t):
    return 2 * t * t if t < 0.5 else 1 - 2 * (1 - t) ** 2
def _ease_in_cubic(t): return t ** 3
def _ease_out_cubic(t): return 1 - (1 - t) ** 3
def _ease_inout_cubic(t):
    return 4 * t ** 3 if t < 0.5 else 1 - 4 * (1 - t) ** 3
def _ease_in_sine(t): return 1 - math.cos(t * math.pi / 2)
def _ease_out_sine(t): return math.sin(t * math.pi / 2)
def _ease_inout_sine(t): return -(math.cos(math.pi * t) - 1) / 2


def _ease_out_bounce(t):
    n1, d1 = 7.5625, 2.75
    if t < 1 / d1:
        return n1 * t * t
    if t < 2 / d1:
        t -= 1.5 / d1
        return n1 * t * t + 0.75
    if t < 2.5 / d1:
        t -= 2.25 / d1
        return n1 * t * t + 0.9375
    t -= 2.625 / d1
    return n1 * t * t + 0.984375


def _ease_in_bounce(t):
    return 1 - _ease_out_bounce(1 - t)


def _ease_out_elastic(t):
    if t == 0 or t == 1:
        return t
    c4 = (2 * math.pi) / 3
    return (2 ** (-10 * t)) * math.sin((t * 10 - 0.75) * c4) + 1


def _ease_in_elastic(t):
    if t == 0 or t == 1:
        return t
    c4 = (2 * math.pi) / 3
    return -(2 ** (10 * t - 10)) * math.sin((t * 10 - 10.75) * c4)


def _ease_inout_elastic(t):
    if t == 0 or t == 1:
        return t
    c5 = (2 * math.pi) / 4.5
    if t < 0.5:
        return -(2 ** (20 * t - 10)) * math.sin((20 * t - 11.125) * c5) / 2
    return (2 ** (-20 * t + 10)) * math.sin((20 * t - 11.125) * c5) / 2 + 1


def _ease_inout_bounce(t):
    """Bounce zur Mitte (klassisches Drop-and-rise)."""
    if t < 0.5:
        return (1 - _ease_out_bounce(1 - 2 * t)) / 2
    return (1 + _ease_out_bounce(2 * t - 1)) / 2


# "Back"-Easings: leichtes Zurueckschwingen am Anfang/Ende - typisch fuer
# UI-Pop-In-Animationen. c1/c3 sind Standard-Penner-Konstanten.
def _ease_in_back(t):
    c1 = 1.70158
    c3 = c1 + 1
    return c3 * t ** 3 - c1 * t ** 2


def _ease_out_back(t):
    c1 = 1.70158
    c3 = c1 + 1
    return 1 + c3 * (t - 1) ** 3 + c1 * (t - 1) ** 2


def _ease_inout_back(t):
    c1 = 1.70158
    c2 = c1 * 1.525
    if t < 0.5:
        return ((2 * t) ** 2 * ((c2 + 1) * 2 * t - c2)) / 2
    return ((2 * t - 2) ** 2 * ((c2 + 1) * (2 * t - 2) + c2) + 2) / 2


_EASINGS = {
    "linear": _ease_linear,
    "in_quad": _ease_in_quad,
    "out_quad": _ease_out_quad,
    "inout_quad": _ease_inout_quad,
    "in_cubic": _ease_in_cubic,
    "out_cubic": _ease_out_cubic,
    "inout_cubic": _ease_inout_cubic,
    "in_sine": _ease_in_sine,
    "out_sine": _ease_out_sine,
    "inout_sine": _ease_inout_sine,
    "in_bounce": _ease_in_bounce,
    "out_bounce": _ease_out_bounce,
    "inout_bounce": _ease_inout_bounce,
    "in_elastic": _ease_in_elastic,
    "out_elastic": _ease_out_elastic,
    "inout_elastic": _ease_inout_elastic,
    "in_back": _ease_in_back,
    "out_back": _ease_out_back,
    "inout_back": _ease_inout_back,
}


# --- Tween-Klasse ----------------------------------------------------

class _Tween:
    __slots__ = ("start", "end", "duration", "easing_name", "easing",
                 "start_ms", "paused_at", "mode")

    def __init__(self, start: float, end: float, duration: int,
                 easing_name: str, easing, mode: str = "once"):
        self.start = start
        self.end = end
        self.duration = duration  # in ms
        self.easing_name = easing_name
        self.easing = easing
        self.start_ms = _now_ms()
        self.paused_at = None  # None = laeuft, int = elapsed_ms beim Pausieren
        # mode: "once" (klassisch, klemmt am Ende) | "loop" (springt wieder
        # an den Start) | "pingpong" (start->end->start->...)
        self.mode = mode

    def __repr__(self):
        state = "paused" if self.paused_at is not None else "running"
        return (f"<Tween {self.start}->{self.end} {self.duration}ms "
                f"{self.easing_name} {self.mode} {state}>")


register_type("tween", _Tween)


def _check_tween(v, fn: str) -> _Tween:
    if not isinstance(v, _Tween):
        raise TypeMismatchError(f"{fn} erwartet TWEEN")
    return v


def _elapsed_ms(t: _Tween) -> int:
    if t.paused_at is not None:
        return t.paused_at
    return _now_ms() - t.start_ms


def _progress(t: _Tween) -> float:
    """Liefert linearen Fortschritt 0..1 (Easing wird vom Caller angewandt).

    - once: klemmt am Ende auf 1.0
    - loop: wrapt; nach Erreichen springt's wieder auf 0
    - pingpong: spiegelt; alternierend forward/backward
    """
    if t.duration == 0:
        return 1.0
    e = _elapsed_ms(t)
    if e <= 0:
        return 0.0
    if t.mode == "loop":
        return (e % t.duration) / t.duration
    if t.mode == "pingpong":
        cycle = e % (2 * t.duration)
        if cycle <= t.duration:
            return cycle / t.duration
        return 2.0 - cycle / t.duration
    # mode "once" (default)
    if e >= t.duration:
        return 1.0
    return e / t.duration


# --- Built-ins -------------------------------------------------------

def _build_tween(args, fn_name: str, mode: str) -> _Tween:
    """Gemeinsame Validierung fuer TWEEN_NEW / _LOOP / _PINGPONG."""
    start = args[0]
    end = args[1]
    duration = args[2]
    if isinstance(start, bool) or not isinstance(start, (int, float)):
        raise TypeMismatchError(f"{fn_name}: start muss Zahl sein")
    if isinstance(end, bool) or not isinstance(end, (int, float)):
        raise TypeMismatchError(f"{fn_name}: end muss Zahl sein")
    if isinstance(duration, bool) or not isinstance(duration, int):
        raise TypeMismatchError(f"{fn_name}: dauer_ms muss INTEGER sein")
    if duration < 0:
        raise GBRuntimeError(f"{fn_name}: dauer_ms muss >= 0 sein")
    if mode in ("loop", "pingpong") and duration == 0:
        raise GBRuntimeError(
            f"{fn_name}: dauer_ms muss > 0 sein (sonst keine Bewegung)"
        )
    easing_name = "linear"
    if len(args) == 4:
        ename = args[3]
        if not isinstance(ename, str):
            raise TypeMismatchError(f"{fn_name}: easing muss STRING sein")
        easing_name = ename.lower()
    easing = _EASINGS.get(easing_name)
    if easing is None:
        raise GBRuntimeError(
            f"{fn_name}: unbekanntes easing '{easing_name}' "
            f"(erlaubt: {', '.join(sorted(_EASINGS.keys()))})"
        )
    return _Tween(float(start), float(end), duration, easing_name, easing, mode)


@builtin("TWEEN_NEW", arity=(3, 4))
def _new(*args):
    """Klassischer One-Shot-Tween. Klemmt am Ende, TWEEN_DONE wird TRUE."""
    return _build_tween(args, "TWEEN_NEW", "once")


@builtin("TWEEN_NEW_LOOP", arity=(3, 4))
def _new_loop(*args):
    """Loopender Tween: nach `dauer_ms` springt er zurueck auf start und
    laeuft erneut. TWEEN_DONE bleibt immer FALSE - das laeuft forever.
    Gut fuer rotierende UI-Spinner, durchgehende Conveyor-Belt-Streifen
    etc."""
    return _build_tween(args, "TWEEN_NEW_LOOP", "loop")


@builtin("TWEEN_NEW_PINGPONG", arity=(3, 4))
def _new_pingpong(*args):
    """Pingpong-Tween: laeuft start->end, dann end->start, dann wieder
    start->end usw. - jede Halbwelle in `dauer_ms`. TWEEN_DONE bleibt FALSE.
    Gut fuer Idle-Bobs (Position oder Skala oszilliert sanft)."""
    return _build_tween(args, "TWEEN_NEW_PINGPONG", "pingpong")


@builtin("TWEEN_VALUE", arity=1)
def _value(t):
    t = _check_tween(t, "TWEEN_VALUE")
    eased = t.easing(_progress(t))
    return t.start + (t.end - t.start) * eased


@builtin("TWEEN_PROGRESS", arity=1)
def _progress_fn(t):
    t = _check_tween(t, "TWEEN_PROGRESS")
    return _progress(t)


@builtin("TWEEN_DONE", arity=1)
def _done(t):
    """TRUE wenn der Tween-Endpunkt erreicht ist. Loop- und Pingpong-Tweens
    werden nie 'done' - die laufen forever."""
    t = _check_tween(t, "TWEEN_DONE")
    if t.mode in ("loop", "pingpong"):
        return False
    return _progress(t) >= 1.0


@builtin("TWEEN_RESTART", arity=1)
def _restart(t):
    t = _check_tween(t, "TWEEN_RESTART")
    t.start_ms = _now_ms()
    t.paused_at = None
    return None


@builtin("TWEEN_PAUSE", arity=1)
def _pause(t):
    t = _check_tween(t, "TWEEN_PAUSE")
    if t.paused_at is None:
        t.paused_at = _elapsed_ms(t)
    return None


@builtin("TWEEN_RESUME", arity=1)
def _resume(t):
    t = _check_tween(t, "TWEEN_RESUME")
    if t.paused_at is not None:
        t.start_ms = _now_ms() - t.paused_at
        t.paused_at = None
    return None


@builtin("TWEEN_REVERSE", arity=1)
def _reverse(t):
    t = _check_tween(t, "TWEEN_REVERSE")
    t.start, t.end = t.end, t.start
    t.start_ms = _now_ms()
    t.paused_at = None
    return None


@builtin("TWEEN_EASINGS", arity=0)
def _easings():
    """Liefert eine Komma-getrennte Liste aller verfuegbaren Easing-Namen."""
    return ", ".join(sorted(_EASINGS.keys()))
