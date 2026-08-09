"""Tests fuer das tween-Modul (Werte-Interpolation).

Golden-Tests gegen `dhrt` (Stufe B): IMPORT "tween" + PRINT. tween interpoliert
ueber die Wall-Clock (MILLIS), daher ist nur die DETERMINISTISCHE Surface
golden-testbar: 0ms-Tweens (sofort fertig), Reverse, Loop/Pingpong-"nie fertig",
Easing-Liste, Validierung. Die frueheren Tests mit `time.sleep` (Pause/Resume/
Restart) und mit direkter Manipulation interner Felder (`t.start_ms`, Import von
`_ease_*`) entfallen -- nicht deterministisch bzw. Python-intern (in Phase 8 weg).
Easing-ENDPUNKTE bleiben abgedeckt: ein 0ms-Tween (progress=1) liefert exakt den
Endwert fuer jedes Easing (auch Overshoot-Easings wie out_back).
"""
import pytest

from gamebasic.errors import GBRuntimeError

_PRE = 'IMPORT "tween"\nDIM t AS TWEEN\n'


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


# --- Basis: 0ms-Tween + Progress/Value ------------------------------

def test_zero_duration_tween_is_done(run_gb):
    out = _lines(run_gb(_PRE + "t = TWEEN_NEW(0.0, 100.0, 0)\n"
                        "PRINT TWEEN_DONE(t)\nPRINT TWEEN_VALUE(t)\n"))
    assert out == ["TRUE", "100.0"]


def test_progress_starts_at_zero(run_gb):
    out = _lines(run_gb(_PRE + "t = TWEEN_NEW(0.0, 100.0, 60000)\n"
                        "PRINT TWEEN_PROGRESS(t) < 0.01\n"))
    assert out == ["TRUE"]


def test_value_at_progress_zero_is_start(run_gb):
    # Gleich nach Erstellung, progress ~0 -> wert ~start (50), mit Toleranz.
    out = _lines(run_gb(_PRE + "t = TWEEN_NEW(50.0, 200.0, 60000)\n"
                        "PRINT TWEEN_VALUE(t) < 56.0\n"))
    assert out == ["TRUE"]


# --- Easings --------------------------------------------------------

def test_easing_unknown_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="unbekanntes easing"):
        run_gb(_PRE + 't = TWEEN_NEW(0.0, 1.0, 100, "schwurbel")\n')


def test_easing_names_are_case_insensitive(run_gb):
    out = _lines(run_gb(_PRE + 't = TWEEN_NEW(0.0, 1.0, 100, "OUT_BOUNCE")\n'
                        "PRINT TWEEN_DONE(t)\n"))
    assert out == ["FALSE"]   # 100ms Tween, gerade erstellt -> laeuft noch


def test_tween_easings_lists_all(run_gb):
    out = run_gb(_PRE.replace("DIM t AS TWEEN\n", "") + "PRINT TWEEN_EASINGS()\n")
    for name in ("linear", "in_cubic", "out_cubic", "inout_cubic",
                 "in_bounce", "out_bounce", "inout_bounce",
                 "in_elastic", "out_elastic", "inout_elastic",
                 "in_back", "out_back", "inout_back"):
        assert name in out


def test_tween_new_with_new_easing(run_gb):
    out = _lines(run_gb(_PRE + 't = TWEEN_NEW(0.0, 100.0, 1000, "out_back")\n'
                        "PRINT TWEEN_DONE(t)\n"
                        't = TWEEN_NEW(0.0, 100.0, 1000, "inout_elastic")\n'
                        "PRINT TWEEN_DONE(t)\n"))
    assert out == ["FALSE", "FALSE"]


def test_easing_endpoints_via_zero_duration(run_gb):
    """0ms-Tween (progress=1) liefert exakt den Endwert -- auch Overshoot-
    Easings (out_back) enden bei 1.0, nicht beim Overshoot-Peak."""
    body = _PRE
    for ease in ("linear", "out_back", "in_back", "inout_back",
                 "out_bounce", "in_elastic", "inout_elastic", "out_elastic"):
        body += f't = TWEEN_NEW(0.0, 100.0, 0, "{ease}")\nPRINT TWEEN_VALUE(t)\n'
    out = _lines(run_gb(body))
    assert len(out) == 8
    for v in out:
        assert abs(float(v) - 100.0) < 0.01   # Endwert (float-tolerant)


# --- Reverse --------------------------------------------------------

def test_reverse_swaps_endpoints(run_gb):
    out = _lines(run_gb(_PRE + "t = TWEEN_NEW(0.0, 100.0, 0)\n"
                        "PRINT TWEEN_VALUE(t)\n"
                        "TWEEN_REVERSE(t)\nPRINT TWEEN_VALUE(t)\n"))
    assert out == ["100.0", "0.0"]


# --- Validierung ----------------------------------------------------

def test_negative_duration_raises(run_gb):
    with pytest.raises(GBRuntimeError, match=">= 0"):
        run_gb(_PRE + "t = TWEEN_NEW(0.0, 1.0, -1)\n")


def test_non_tween_to_value_raises(run_gb):
    with pytest.raises(GBRuntimeError, match="erwartet TWEEN"):
        run_gb('IMPORT "tween"\nPRINT TWEEN_VALUE("nicht ein tween")\n')


# --- Loop & Pingpong-Modi -------------------------------------------

def test_loop_never_done(run_gb):
    """Loop-Tween ist per Definition nie 'done' (auch sofort nach Erstellung)."""
    out = _lines(run_gb(_PRE + "t = TWEEN_NEW_LOOP(0.0, 1.0, 100)\n"
                        "PRINT TWEEN_DONE(t)\n"))
    assert out == ["FALSE"]


def test_pingpong_never_done(run_gb):
    out = _lines(run_gb(_PRE + "t = TWEEN_NEW_PINGPONG(0.0, 1.0, 100)\n"
                        "PRINT TWEEN_DONE(t)\n"))
    assert out == ["FALSE"]


def test_loop_zero_duration_rejected(run_gb):
    with pytest.raises(GBRuntimeError, match=">"):
        run_gb(_PRE + "t = TWEEN_NEW_LOOP(0.0, 1.0, 0)\n")


def test_pingpong_zero_duration_rejected(run_gb):
    with pytest.raises(GBRuntimeError, match=">"):
        run_gb(_PRE + "t = TWEEN_NEW_PINGPONG(0.0, 1.0, 0)\n")


def test_loop_works_with_easing(run_gb):
    out = _lines(run_gb(_PRE + 't = TWEEN_NEW_LOOP(0.0, 1.0, 100, "out_cubic")\n'
                        "PRINT TWEEN_DONE(t)\n"
                        't = TWEEN_NEW_PINGPONG(0.0, 1.0, 100, "inout_sine")\n'
                        "PRINT TWEEN_DONE(t)\n"))
    assert out == ["FALSE", "FALSE"]
