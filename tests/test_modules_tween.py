"""Tests fuer das tween-Modul.

Wir testen die deterministischen Pfade: Easing-Werte bei progress=0/1,
Reverse, Restart, Pause/Resume. Die Zeit wird nicht direkt manipuliert -
wir nutzen 0ms-Tweens (immer fertig) und Tweens mit langem Default-Lauf.
"""
import time
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_tween():
    assert load_module("tween")


def test_zero_duration_tween_is_done(call_builtin):
    t = call_builtin("tween_new", [0.0, 100.0, 0])
    assert call_builtin("tween_done", [t]) is True
    assert call_builtin("tween_value", [t]) == 100.0


def test_progress_starts_at_zero(call_builtin):
    t = call_builtin("tween_new", [0.0, 100.0, 60_000])  # 60s, kaum fortgeschritten
    p = call_builtin("tween_progress", [t])
    assert 0.0 <= p < 0.01


def test_value_at_progress_zero_is_start(call_builtin):
    t = call_builtin("tween_new", [50.0, 200.0, 60_000])
    # gleich nach Erstellung, progress ~ 0 -> wert ~ start
    v = call_builtin("tween_value", [t])
    assert abs(v - 50.0) < 5.0


def test_easing_unknown_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="unbekanntes easing"):
        call_builtin("tween_new", [0.0, 1.0, 100, "schwurbel"])


def test_easing_names_are_case_insensitive(call_builtin):
    t = call_builtin("tween_new", [0.0, 1.0, 100, "OUT_BOUNCE"])
    assert t is not None


def test_pause_freezes_progress(call_builtin):
    t = call_builtin("tween_new", [0.0, 100.0, 200])
    time.sleep(0.05)
    call_builtin("tween_pause", [t])
    p1 = call_builtin("tween_progress", [t])
    time.sleep(0.1)
    p2 = call_builtin("tween_progress", [t])
    assert p1 == p2  # eingefroren


def test_resume_continues(call_builtin):
    t = call_builtin("tween_new", [0.0, 100.0, 1000])
    time.sleep(0.05)
    call_builtin("tween_pause", [t])
    p_paused = call_builtin("tween_progress", [t])
    time.sleep(0.1)
    call_builtin("tween_resume", [t])
    time.sleep(0.05)
    p_after = call_builtin("tween_progress", [t])
    assert p_after > p_paused


def test_restart_resets_progress(call_builtin):
    t = call_builtin("tween_new", [0.0, 100.0, 100])
    time.sleep(0.15)
    assert call_builtin("tween_done", [t]) is True
    call_builtin("tween_restart", [t])
    assert call_builtin("tween_progress", [t]) < 0.5


def test_reverse_swaps_endpoints(call_builtin):
    t = call_builtin("tween_new", [0.0, 100.0, 0])  # sofort fertig -> wert=100
    assert call_builtin("tween_value", [t]) == 100.0
    call_builtin("tween_reverse", [t])
    # Nach Reverse: start=100, end=0, restartet. Bei 0ms-Tween wieder sofort am Ende.
    assert call_builtin("tween_value", [t]) == 0.0


def test_negative_duration_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match=">= 0"):
        call_builtin("tween_new", [0.0, 1.0, -1])


def test_non_tween_to_value_raises(call_builtin):
    with pytest.raises(TypeMismatchError, match="erwartet TWEEN"):
        call_builtin("tween_value", ["nicht ein tween"])


# --- Neue Easings ---------------------------------------------------

def test_tween_easings_lists_all(call_builtin):
    s = call_builtin("tween_easings", [])
    for name in ("linear", "in_cubic", "out_cubic", "inout_cubic",
                 "in_bounce", "out_bounce", "inout_bounce",
                 "in_elastic", "out_elastic", "inout_elastic",
                 "in_back", "out_back", "inout_back"):
        assert name in s


def test_inout_bounce_endpoints():
    from gamebasic.modules.tween import _ease_inout_bounce
    assert _ease_inout_bounce(0.0) == 0.0
    assert abs(_ease_inout_bounce(1.0) - 1.0) < 1e-9


def test_in_elastic_endpoints():
    from gamebasic.modules.tween import _ease_in_elastic
    assert _ease_in_elastic(0.0) == 0.0
    assert _ease_in_elastic(1.0) == 1.0


def test_inout_elastic_endpoints():
    from gamebasic.modules.tween import _ease_inout_elastic
    assert _ease_inout_elastic(0.0) == 0.0
    assert _ease_inout_elastic(1.0) == 1.0


def test_back_easings_overshoot():
    """Out_back ueberschiesst das Ziel zwischendurch (Pop-Effekt)."""
    from gamebasic.modules.tween import _ease_out_back
    val = _ease_out_back(0.7)
    assert val > 0.95


def test_back_easings_endpoints():
    from gamebasic.modules.tween import (
        _ease_in_back, _ease_out_back, _ease_inout_back,
    )
    for fn in (_ease_in_back, _ease_out_back, _ease_inout_back):
        assert abs(fn(0.0)) < 1e-9
        assert abs(fn(1.0) - 1.0) < 1e-9


def test_tween_new_with_new_easing(call_builtin):
    """Neue Easings auch via TWEEN_NEW(start, end, dur, easing) verfuegbar."""
    t = call_builtin("tween_new", [0.0, 100.0, 1000, "out_back"])
    assert t is not None
    t = call_builtin("tween_new", [0.0, 100.0, 1000, "inout_elastic"])
    assert t is not None


# --- Loop & Pingpong-Modi -------------------------------------------

def test_loop_never_done(call_builtin):
    """Loop-Tween wird niemals 'done' - egal wie viel Zeit vergeht."""
    t = call_builtin("tween_new_loop", [0.0, 1.0, 100])
    time.sleep(0.25)   # 250 ms = 2.5 Loops
    assert call_builtin("tween_done", [t]) is False


def test_pingpong_never_done(call_builtin):
    t = call_builtin("tween_new_pingpong", [0.0, 1.0, 100])
    time.sleep(0.25)
    assert call_builtin("tween_done", [t]) is False


def test_loop_value_wraps(call_builtin):
    """Loop-Wert wrappt: vor dem Cycle-Ende fast bei 100, danach fast bei 0."""
    from gamebasic.modules import tween as tw_mod
    t = call_builtin("tween_new_loop", [0.0, 100.0, 1000])
    # Stelle uns selbst die elapsed Zeit ein, indem wir start_ms zurueckdrehen.
    now = tw_mod._now_ms()
    # 999 ms vergangen -> kurz vor Cycle-Ende
    t.start_ms = now - 999
    assert call_builtin("tween_value", [t]) > 95.0
    # 1001 ms vergangen -> 1 ms in den naechsten Cycle (wrap auf ~0)
    t.start_ms = now - 1001
    assert call_builtin("tween_value", [t]) < 5.0
    # 1500 ms = halb durch 2. Cycle -> Wert ~50
    t.start_ms = now - 1500
    val = call_builtin("tween_value", [t])
    assert 40.0 < val < 60.0


def test_pingpong_returns_to_start(call_builtin):
    """Nach 2 * dauer_ms (kompletter Pingpong-Zyklus) Wert wieder am Start."""
    from gamebasic.modules import tween as tw_mod
    t = call_builtin("tween_new_pingpong", [0.0, 100.0, 1000])
    now = tw_mod._now_ms()
    t.start_ms = now - 2000   # 2 Halbwellen = voller Zyklus
    val = call_builtin("tween_value", [t])
    assert val < 5.0


def test_pingpong_midcycle_at_end(call_builtin):
    """Bei genau 1 * dauer_ms (eine Halbwelle voll) ist der Wert am End-Punkt."""
    from gamebasic.modules import tween as tw_mod
    t = call_builtin("tween_new_pingpong", [0.0, 100.0, 1000])
    now = tw_mod._now_ms()
    t.start_ms = now - 1000
    val = call_builtin("tween_value", [t])
    assert val > 95.0


def test_pingpong_quarter_and_three_quarter(call_builtin):
    """Bei 1/4 Halbwelle: ~25; bei 3/4 (= 1.75 von 2): ~25 (Rueckweg)."""
    from gamebasic.modules import tween as tw_mod
    t = call_builtin("tween_new_pingpong", [0.0, 100.0, 1000])
    now = tw_mod._now_ms()
    # 250 ms = Viertel-Halbwelle, Wert ~25
    t.start_ms = now - 250
    assert 20.0 < call_builtin("tween_value", [t]) < 30.0
    # 1750 ms = 3/4 in der zweiten Halbwelle (= Rueckweg) -> auch ~25
    t.start_ms = now - 1750
    assert 20.0 < call_builtin("tween_value", [t]) < 30.0


def test_loop_zero_duration_rejected(call_builtin):
    """Loop ohne Dauer waere endlose Stillstand-Bewegung -> Fehler."""
    with pytest.raises(GBRuntimeError, match=">"):
        call_builtin("tween_new_loop", [0.0, 1.0, 0])


def test_pingpong_zero_duration_rejected(call_builtin):
    with pytest.raises(GBRuntimeError, match=">"):
        call_builtin("tween_new_pingpong", [0.0, 1.0, 0])


def test_loop_works_with_easing(call_builtin):
    """Easing-Param wird auch von loop/pingpong akzeptiert."""
    t = call_builtin("tween_new_loop", [0.0, 1.0, 100, "out_cubic"])
    assert t is not None
    t = call_builtin("tween_new_pingpong", [0.0, 1.0, 100, "inout_sine"])
    assert t is not None
