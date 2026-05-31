"""Tests fuer das particles-Modul. Ohne Pygame - testet nur Logik.

Random-Seed gesetzt damit Aging deterministisch wird.
"""
import random
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_particles():
    assert load_module("particles")


def test_new_starts_empty(call_builtin):
    sys = call_builtin("particle_system_new", [100.0, 200.0])
    assert call_builtin("particle_count", [sys]) == 0


def test_emit_increases_count(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    call_builtin("particle_emit", [sys, 5])
    assert call_builtin("particle_count", [sys]) == 5
    call_builtin("particle_emit", [sys, 3])
    assert call_builtin("particle_count", [sys]) == 8


def test_clear(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    call_builtin("particle_emit", [sys, 5])
    call_builtin("particle_clear", [sys])
    assert call_builtin("particle_count", [sys]) == 0


def test_update_zero_dt_no_change(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    call_builtin("particle_emit", [sys, 5])
    call_builtin("particle_update", [sys, 0])
    assert call_builtin("particle_count", [sys]) == 5


def test_update_kills_old_particles(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    call_builtin("particle_set_lifetime", [sys, 100, 100])  # exakt 100ms
    call_builtin("particle_emit", [sys, 5])
    # Nach 50ms: alle leben
    call_builtin("particle_update", [sys, 50])
    assert call_builtin("particle_count", [sys]) == 5
    # Nach weiteren 100ms (gesamt 150ms): alle tot
    call_builtin("particle_update", [sys, 100])
    assert call_builtin("particle_count", [sys]) == 0


def test_set_lifetime_invalid_raises(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    with pytest.raises(GBRuntimeError, match="ms_min >= 0"):
        call_builtin("particle_set_lifetime", [sys, -1, 100])
    with pytest.raises(GBRuntimeError, match="ms_max >= ms_min"):
        call_builtin("particle_set_lifetime", [sys, 200, 100])


def test_set_velocity_invalid_raises(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    with pytest.raises(GBRuntimeError, match="max muss >= min"):
        call_builtin("particle_set_velocity", [sys, 100.0, 50.0, 0.0, 0.0])


def test_set_color_out_of_range_raises(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    with pytest.raises(GBRuntimeError, match="0..0xFFFFFF"):
        call_builtin("particle_set_color", [sys, 0x1000000])


def test_set_size_invalid_raises(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    with pytest.raises(GBRuntimeError, match="min >= 1"):
        call_builtin("particle_set_size", [sys, 0, 5])


def test_emit_negative_count_raises(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    with pytest.raises(GBRuntimeError, match=">= 0"):
        call_builtin("particle_emit", [sys, -1])


def test_emit_creates_numpy_arrays(call_builtin):
    """Nach emit() sind die internen Arrays NumPy mit korrekter Laenge."""
    import numpy as np
    sys = call_builtin("particle_system_new", [10.0, 20.0])
    call_builtin("particle_emit", [sys, 100])
    assert isinstance(sys._xs, np.ndarray)
    assert sys._xs.shape == (100,)
    assert sys._xs.dtype == np.float32
    # Alle starten an (10, 20)
    assert (sys._xs == 10.0).all()
    assert (sys._ys == 20.0).all()


def test_update_kills_aged_particles(call_builtin):
    """Nach Lifetime-Ablauf werden Partikel entfernt."""
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    call_builtin("particle_set_lifetime", [sys, 100, 100])
    call_builtin("particle_emit", [sys, 50])
    assert call_builtin("particle_count", [sys]) == 50
    # 200ms vergehen lassen -> alle tot
    call_builtin("particle_update", [sys, 200])
    assert call_builtin("particle_count", [sys]) == 0


def test_update_applies_gravity_vectorized(call_builtin):
    """Gravity wirkt vektorisiert auf alle Partikel."""
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    call_builtin("particle_set_velocity", [sys, 0.0, 0.0, 0.0, 0.0])
    call_builtin("particle_set_lifetime", [sys, 10000, 10000])
    call_builtin("particle_set_gravity", [sys, 0.0, 100.0])  # 100 px/s^2
    call_builtin("particle_emit", [sys, 5])
    # 1 Sekunde update -> vy = 100, y = 100
    call_builtin("particle_update", [sys, 1000])
    # Alle 5 Partikel haben die gleiche y-Position
    ys = sys._ys.tolist()
    assert all(abs(y - 100.0) < 0.01 for y in ys), ys


def test_clear_resets_arrays(call_builtin):
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    call_builtin("particle_emit", [sys, 50])
    assert call_builtin("particle_count", [sys]) == 50
    call_builtin("particle_clear", [sys])
    assert call_builtin("particle_count", [sys]) == 0
    assert sys._xs.shape == (0,)


def test_particles_performance_at_5000(call_builtin):
    """Performance-Smoke: 5000 Partikel update in unter 50 ms."""
    import time
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    call_builtin("particle_set_lifetime", [sys, 10000, 10000])  # bleibt lange
    call_builtin("particle_emit", [sys, 5000])
    t0 = time.perf_counter()
    for _ in range(10):
        call_builtin("particle_update", [sys, 16])  # 16ms wie 60fps
    elapsed = (time.perf_counter() - t0) * 1000  # ms
    per_update = elapsed / 10
    # Bei 5000 Partikeln sollte ein Update <5ms sein (NumPy-vektorisiert)
    assert per_update < 50, f"5000-Partikel-update zu langsam: {per_update:.2f}ms"


def test_set_pos_changes_emit_origin(call_builtin):
    """Neue Partikel werden ab geänderter Pos emittiert (intern: sys.x/y)."""
    sys = call_builtin("particle_system_new", [0.0, 0.0])
    call_builtin("particle_set_velocity", [sys, 0.0, 0.0, 0.0, 0.0])  # keine Bewegung
    call_builtin("particle_set_pos", [sys, 999.0, 888.0])
    call_builtin("particle_emit", [sys, 1])
    # Nach 0ms-Update keine Verschiebung
    call_builtin("particle_update", [sys, 0])
    # Internen Zustand pruefen ueber die NumPy-Arrays
    assert float(sys._xs[0]) == 999.0
    assert float(sys._ys[0]) == 888.0
