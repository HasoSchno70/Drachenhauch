"""Tests fuer das particles-Modul (Emitter-Logik).

Golden-Tests gegen `dhrt` (Stufe B): IMPORT "particles" + PARTICLE_COUNT/EMIT/
UPDATE/CLEAR + PRINT. Frueher via `call_builtin` gegen die Python-Impl (in Phase 8
geloescht). Tests, die fueher die internen NumPy-Arrays (`sys._xs`/`_ys`) lasen
(Gravity-/Pos-/Performance-Interna), entfallen -- die Partikel-Positionen sind in
GB ohne Render-Pfad nicht beobachtbar; die beobachtbare Surface (Count, Aging,
Validierung) ist hier abgedeckt.
"""
import pytest

from drachenhauch.errors import DHRuntimeError

_PRE = ('IMPORT "particles"\nDIM s AS PARTICLE_SYSTEM\n'
        's = PARTICLE_SYSTEM_NEW(0.0, 0.0)\n')


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def test_new_starts_empty(run_gb):
    out = _lines(run_gb('IMPORT "particles"\nDIM s AS PARTICLE_SYSTEM\n'
                        's = PARTICLE_SYSTEM_NEW(100.0, 200.0)\nPRINT PARTICLE_COUNT(s)\n'))
    assert out == ["0"]


def test_emit_increases_count(run_gb):
    out = _lines(run_gb(_PRE +
                        "PARTICLE_EMIT(s, 5)\nPRINT PARTICLE_COUNT(s)\n"
                        "PARTICLE_EMIT(s, 3)\nPRINT PARTICLE_COUNT(s)\n"))
    assert out == ["5", "8"]


def test_clear(run_gb):
    out = _lines(run_gb(_PRE +
                        "PARTICLE_EMIT(s, 5)\nPARTICLE_CLEAR(s)\nPRINT PARTICLE_COUNT(s)\n"))
    assert out == ["0"]


def test_update_zero_dt_no_change(run_gb):
    out = _lines(run_gb(_PRE +
                        "PARTICLE_EMIT(s, 5)\nPARTICLE_UPDATE(s, 0)\nPRINT PARTICLE_COUNT(s)\n"))
    assert out == ["5"]


def test_update_kills_old_particles(run_gb):
    out = _lines(run_gb(_PRE +
                        "PARTICLE_SET_LIFETIME(s, 100, 100)\nPARTICLE_EMIT(s, 5)\n"
                        "PARTICLE_UPDATE(s, 50)\nPRINT PARTICLE_COUNT(s)\n"
                        "PARTICLE_UPDATE(s, 100)\nPRINT PARTICLE_COUNT(s)\n"))
    assert out == ["5", "0"]


def test_update_kills_aged_particles(run_gb):
    out = _lines(run_gb(_PRE +
                        "PARTICLE_SET_LIFETIME(s, 100, 100)\nPARTICLE_EMIT(s, 50)\n"
                        "PRINT PARTICLE_COUNT(s)\n"
                        "PARTICLE_UPDATE(s, 200)\nPRINT PARTICLE_COUNT(s)\n"))
    assert out == ["50", "0"]


def test_emit_many(run_gb):
    """Massen-Emit funktioniert (frueher numpy-Array-Shape-Check)."""
    out = _lines(run_gb(_PRE + "PARTICLE_EMIT(s, 5000)\nPRINT PARTICLE_COUNT(s)\n"))
    assert out == ["5000"]


# --- Validierung ---------------------------------------------------

def test_set_lifetime_invalid_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="ms_min >= 0"):
        run_gb(_PRE + "PARTICLE_SET_LIFETIME(s, -1, 100)\n")
    with pytest.raises(DHRuntimeError, match="ms_max >= ms_min"):
        run_gb(_PRE + "PARTICLE_SET_LIFETIME(s, 200, 100)\n")


def test_set_velocity_invalid_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="max muss >= min"):
        run_gb(_PRE + "PARTICLE_SET_VELOCITY(s, 100.0, 50.0, 0.0, 0.0)\n")


def test_set_color_out_of_range_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="0..0xFFFFFF"):
        run_gb(_PRE + "PARTICLE_SET_COLOR(s, 16777217)\n")


def test_set_size_invalid_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="min >= 1"):
        run_gb(_PRE + "PARTICLE_SET_SIZE(s, 0, 5)\n")


def test_emit_negative_count_raises(run_gb):
    with pytest.raises(DHRuntimeError, match=">= 0"):
        run_gb(_PRE + "PARTICLE_EMIT(s, -1)\n")


# --- Ausgerechnete Werte statt Pflicht-INT() -----------------------------
# Groessen, Dauern und Stueckzahlen werden im Programm typischerweise
# AUSGERECHNET (dt * 1000.0). Dort ein INTEGER zu verlangen zwang den Aufrufer
# zu einem INT(...) um jeden Ausdruck -- und wer es vergass, bekam einen
# Typfehler fuer etwas, das rechnerisch in Ordnung war.
def test_kommazahlen_werden_gerundet_statt_abgelehnt(run_gb):
    src = """
IMPORT "particles"
DIM p AS PARTICLE_SYSTEM
p = PARTICLE_SYSTEM_NEW(0, 0)
PARTICLE_SET_SIZE(p, 2.0, 6.4)
PARTICLE_SET_LIFETIME(p, 100.0, 900.6)
PARTICLE_EMIT(p, 5.0)
PARTICLE_UPDATE(p, 16.7)
PRINT PARTICLE_COUNT(p)
"""
    assert run_gb(src).strip() == "5"
