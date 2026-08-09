"""Aritaets-Fehler bei haeufig falsch benutzten Builtins haengen die erwartete
Aufruf-Signatur an, damit man die Argumentform nicht raten muss
(RANDF/RANDINT/CURVE_* -- die Stolpersteine aus dem Galaga-Bau)."""

import pytest
from drachenhauch.errors import DHRuntimeError


def test_randf_arity_shows_signature(run_gb):
    with pytest.raises(DHRuntimeError, match=r"RANDF\(min, max\)"):
        run_gb("PRINT RANDF()")


def test_randint_arity_shows_signature(run_gb):
    with pytest.raises(DHRuntimeError, match=r"RANDINT\(lo, hi\)"):
        run_gb("PRINT RANDINT(5)")


def test_curve_bezier2_arity_shows_signature(run_gb):
    src = 'IMPORT "curves"\nPRINT CURVE_BEZIER2(0.5, 1.0)'
    with pytest.raises(DHRuntimeError, match=r"CURVE_BEZIER2\(t, x0,y0"):
        run_gb(src)


def test_builtin_without_signature_unchanged(run_gb):
    """Builtins ohne Tabellen-Eintrag behalten die schlichte Aritaets-Meldung."""
    with pytest.raises(DHRuntimeError, match=r"ABS: erwartet 1 Argument"):
        run_gb("PRINT ABS()")


def test_correct_arity_still_works(run_gb):
    assert run_gb("PRINT RANDF(0.0, 1.0) >= 0.0") == "TRUE\n"
