"""Pure Partikel-Simulation (NumPy-vektorisiert) -- ohne GameBasic-Laufzeit.

Extrahiert aus `modules/particles.py` (Stufe B), damit der Partikel-Editor
(`particleeditor_qt`) die Vorschau-Simulation nutzen kann, ohne die Built-in-
Registry / den Tree-Walker zu importieren -- Voraussetzung fuers Entfernen von
`modules/particles.py` in Phase 8. Reine numpy/random-Logik (EMIT/UPDATE/CLEAR);
das Rendering (PARTICLE_DRAW) ist nativ in gbrt und nicht Teil dieser Klasse.
"""
from __future__ import annotations

import random as _random

import numpy as np


class ParticleSystem:
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
        # sorted(): random.randint(a, b) verlangt a <= b und wirft sonst
        # ValueError. Der Editor haelt min<=max schon per UI-Sync ein, aber
        # diese Klasse wird auch direkt genutzt (Tests/zukuenftige Aufrufer)
        # -- ohne eigene Absicherung wuerde ein vertauschtes min/max hier
        # abstuerzen statt einfach das Intervall andersrum zu lesen.
        life_lo, life_hi = sorted((self.lifetime_min, self.lifetime_max))
        new_lifetimes = np.array(
            [_random.randint(life_lo, life_hi)
             for _ in range(count)],
            dtype=np.int32,
        )
        new_ages = np.zeros(count, dtype=np.int32)
        size_lo, size_hi = sorted((self.size_min, self.size_max))
        new_sizes = np.array(
            [_random.randint(size_lo, size_hi)
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


# Rueckwaerts-kompatibler Alias (modules/particles.py + Editor nutzen den Namen).
_ParticleSystem = ParticleSystem
