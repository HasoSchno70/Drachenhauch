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

import random as _random

import numpy as np

from ..builtins_registry import builtin, graphics_builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type


class _ParticleSystem:
    __slots__ = ("x", "y",
                 "vx_min", "vx_max", "vy_min", "vy_max",
                 "lifetime_min", "lifetime_max",
                 "gravity_x", "gravity_y",
                 "color", "size_min", "size_max",
                 "fade", "mode", "color_end", "has_color_end",
                 # NumPy-Arrays - aktive Partikel-Daten
                 "_xs", "_ys", "_vxs", "_vys",
                 "_lifetimes", "_ages", "_sizes", "_colors")

    def __init__(self, x: float, y: float):
        self.x = float(x)
        self.y = float(y)
        # Defaults: leichter Funkenregen mit Schwerkraft.
        self.vx_min = -50.0
        self.vx_max = 50.0
        self.vy_min = -100.0
        self.vy_max = -50.0
        self.lifetime_min = 500
        self.lifetime_max = 1000
        self.gravity_x = 0.0
        self.gravity_y = 200.0
        self.color = 0xFFFFFF
        self.size_min = 2
        self.size_max = 4
        self.fade = True
        # Render-Modus + optionaler Farbverlauf ueber die Lebenszeit.
        self.mode = "circle"          # circle | pixel | square | streak | glow
        self.color_end = 0x000000
        self.has_color_end = False
        # Initial leere Arrays. Float32 fuer Position/Velocity (geringerer
        # Speicher, ausreichend Praezision fuer Pixel-Koordinaten); int32
        # fuer Ganzzahlen.
        self._xs = np.empty(0, dtype=np.float32)
        self._ys = np.empty(0, dtype=np.float32)
        self._vxs = np.empty(0, dtype=np.float32)
        self._vys = np.empty(0, dtype=np.float32)
        self._lifetimes = np.empty(0, dtype=np.int32)
        self._ages = np.empty(0, dtype=np.int32)
        self._sizes = np.empty(0, dtype=np.int32)
        self._colors = np.empty(0, dtype=np.int32)

    def __repr__(self):
        return (f"<PARTICLE_SYSTEM @({self.x:.0f},{self.y:.0f}) "
                f"{self.count()} aktiv>")

    def count(self) -> int:
        return int(self._xs.shape[0])

    def clear(self) -> None:
        self._xs = np.empty(0, dtype=np.float32)
        self._ys = np.empty(0, dtype=np.float32)
        self._vxs = np.empty(0, dtype=np.float32)
        self._vys = np.empty(0, dtype=np.float32)
        self._lifetimes = np.empty(0, dtype=np.int32)
        self._ages = np.empty(0, dtype=np.int32)
        self._sizes = np.empty(0, dtype=np.int32)
        self._colors = np.empty(0, dtype=np.int32)

    def emit(self, count: int) -> None:
        if count <= 0:
            return
        # Random aus dem Python-`random`-Modul: das ist GB's RANDOMIZE-Seed
        # und gibt Determinismus, der mit dem Bench-Vergleich kompatibel ist.
        # Per-Partikel-Loop ist hier okay, weil emit selten ist (typisch
        # 5-50 Partikel pro Frame). Der Hot-Path ist update() - der bleibt
        # vektorisiert.
        new_xs = np.full(count, self.x, dtype=np.float32)
        new_ys = np.full(count, self.y, dtype=np.float32)
        new_vxs = np.array(
            [_random.uniform(self.vx_min, self.vx_max)
             for _ in range(count)],
            dtype=np.float32,
        )
        new_vys = np.array(
            [_random.uniform(self.vy_min, self.vy_max)
             for _ in range(count)],
            dtype=np.float32,
        )
        new_lifetimes = np.array(
            [_random.randint(self.lifetime_min, self.lifetime_max)
             for _ in range(count)],
            dtype=np.int32,
        )
        new_ages = np.zeros(count, dtype=np.int32)
        new_sizes = np.array(
            [_random.randint(self.size_min, self.size_max)
             for _ in range(count)],
            dtype=np.int32,
        )
        new_colors = np.full(count, self.color, dtype=np.int32)
        # Hintenanhaengen
        self._xs = np.concatenate([self._xs, new_xs])
        self._ys = np.concatenate([self._ys, new_ys])
        self._vxs = np.concatenate([self._vxs, new_vxs])
        self._vys = np.concatenate([self._vys, new_vys])
        self._lifetimes = np.concatenate([self._lifetimes, new_lifetimes])
        self._ages = np.concatenate([self._ages, new_ages])
        self._sizes = np.concatenate([self._sizes, new_sizes])
        self._colors = np.concatenate([self._colors, new_colors])

    def update(self, dt_ms: int) -> None:
        if self._xs.shape[0] == 0:
            return
        dt = dt_ms / 1000.0
        # 1) Alterung. dt_ms ist int -> in-place add.
        self._ages += np.int32(dt_ms)
        # 2) Tote rauswerfen. mask sind die Lebenden.
        alive = self._ages < self._lifetimes
        if not bool(alive.all()):
            self._xs = self._xs[alive]
            self._ys = self._ys[alive]
            self._vxs = self._vxs[alive]
            self._vys = self._vys[alive]
            self._lifetimes = self._lifetimes[alive]
            self._ages = self._ages[alive]
            self._sizes = self._sizes[alive]
            self._colors = self._colors[alive]
            if self._xs.shape[0] == 0:
                return
        # 3) Geschwindigkeit (Gravity-Beschleunigung)
        if self.gravity_x:
            self._vxs += np.float32(self.gravity_x * dt)
        if self.gravity_y:
            self._vys += np.float32(self.gravity_y * dt)
        # 4) Position
        self._xs += self._vxs * np.float32(dt)
        self._ys += self._vys * np.float32(dt)


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


# Cache fuer Glow-Sprites: (size, quantisierte_farbe) -> pygame.Surface.
# Farbe auf 4 Bit/Kanal quantisiert, damit Farbverlaeufe nicht beliebig
# viele Eintraege erzeugen. Gekappt, um Speicher zu begrenzen.
_GLOW_CACHE: dict = {}


def _glow_surface(g, size, color):
    key = (size, color & 0xF0F0F0)
    surf = _GLOW_CACHE.get(key)
    if surf is None:
        pg = g._pygame
        d = size * 2 + 1
        yy, xx = np.ogrid[-size:size + 1, -size:size + 1]
        dist = np.sqrt(xx * xx + yy * yy) / max(size, 1)
        fall = np.clip(1.0 - dist, 0.0, 1.0) ** 2          # weicher Abfall
        r = (color >> 16) & 0xFF
        gg = (color >> 8) & 0xFF
        b = color & 0xFF
        rgb = np.zeros((d, d, 3), dtype=np.uint8)
        rgb[..., 0] = (r * fall).astype(np.uint8)
        rgb[..., 1] = (gg * fall).astype(np.uint8)
        rgb[..., 2] = (b * fall).astype(np.uint8)
        surf = pg.surfarray.make_surface(rgb)
        if len(_GLOW_CACHE) < 512:
            _GLOW_CACHE[key] = surf
    return surf


@graphics_builtin("PARTICLE_DRAW", arity=1)
def _draw(g, sys):
    sys = _check_sys(sys, "PARTICLE_DRAW")
    n = sys.count()
    if n == 0:
        return None

    # --- Farbe pro Partikel vektorisiert berechnen ---------------------
    # life_t in [0,1]: 0 = gerade geboren, 1 = stirbt jetzt.
    lifetimes = np.maximum(sys._lifetimes, 1).astype(np.float32)
    life_t = np.clip(sys._ages.astype(np.float32) / lifetimes, 0.0, 1.0)

    if not sys.has_color_end and not sys.fade:
        final_colors = sys._colors
    else:
        sr = ((sys._colors >> 16) & 0xFF).astype(np.float32)
        sg = ((sys._colors >> 8) & 0xFF).astype(np.float32)
        sb = (sys._colors & 0xFF).astype(np.float32)
        if sys.has_color_end:
            er = (sys.color_end >> 16) & 0xFF
            eg = (sys.color_end >> 8) & 0xFF
            eb = sys.color_end & 0xFF
            inv = 1.0 - life_t
            sr = sr * inv + er * life_t
            sg = sg * inv + eg * life_t
            sb = sb * inv + eb * life_t
        if sys.fade:
            f = 1.0 - life_t
            sr = sr * f
            sg = sg * f
            sb = sb * f
        rr = np.clip(sr, 0, 255).astype(np.int32)
        gg = np.clip(sg, 0, 255).astype(np.int32)
        bb = np.clip(sb, 0, 255).astype(np.int32)
        final_colors = (rr << 16) | (gg << 8) | bb

    xs = sys._xs.astype(np.int32).tolist()
    ys = sys._ys.astype(np.int32).tolist()
    sizes = sys._sizes.tolist()
    cols = final_colors.tolist()
    mode = sys.mode

    if mode == "pixel":
        # Bulk-Plot: alle Pixel in EINEM Aufruf (vektorisiert).
        g.plot_many(xs, ys, cols)
    elif mode == "square":
        for i in range(n):
            s = sizes[i]
            g.box(xs[i] - s, ys[i] - s, xs[i] + s, ys[i] + s, cols[i])
    elif mode == "streak":
        vxs = sys._vxs.tolist()
        vys = sys._vys.tolist()
        for i in range(n):
            x = xs[i]
            y = ys[i]
            g.line(x, y, int(x - vxs[i] * 0.04), int(y - vys[i] * 0.04), cols[i])
    elif mode == "glow":
        pg = g._pygame
        add = pg.BLEND_RGB_ADD
        buf = g._buffer
        for i in range(n):
            s = sizes[i]
            if s < 1:
                s = 1
            surf = _glow_surface(g, s, cols[i])
            sx, sy = g._w2s(xs[i], ys[i])
            buf.blit(surf, (sx - s, sy - s), special_flags=add)
    else:  # "circle"
        for i in range(n):
            g.circle(xs[i], ys[i], sizes[i], cols[i])
    return None
