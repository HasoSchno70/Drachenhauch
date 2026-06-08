"""Golden-Tests fuer die Robustheits-Fixes (Review-Funde):
1. Falsche Argumentzahl in variadischen Builtins -> klarer Laufzeitfehler statt
   Rust-Panic/Absturz (Sicherheitsnetz via catch_unwind).
2. Referenz-Typen (ARRAY/MAP) sind ueber Identitaet gleich -- u.a. `a = a` TRUE.
3. Ganzzahl-Ueberlauf (+,-,*,^, zu grosses Literal) -> klarer Laufzeitfehler
   statt still falschem Wert.
"""
import pytest

from gamebasic.errors import GameBasicError


# ------------------------------------------------ 1) Arity -> kein Absturz
@pytest.mark.parametrize("src", [
    'PRINT MID$("hi")',
    'PRINT INSTR("abc")',
    'PRINT LOG()',
    'PRINT PADL$("x")',
])
def test_too_few_args_is_clean_error_not_crash(run_gb, src):
    with pytest.raises(GameBasicError, match="(?i)zu wenige Argumente|erwartet"):
        run_gb(src)


# ------------------------------------------------ 2) Referenz-Gleichheit
def test_array_equals_itself(run_gb):
    out = run_gb("DIM a[3] AS INTEGER\na[0]=1\nPRINT a = a\n").strip()
    assert out == "TRUE"


def test_array_alias_equal(run_gb):
    out = run_gb(
        "DIM a[2] AS INTEGER\n"
        "DIM b AS ARRAY OF INTEGER\n"
        "b = a\n"
        "PRINT a = b\n"
    ).strip()
    assert out == "TRUE"


def test_distinct_arrays_not_equal(run_gb):
    # Inhaltsgleich, aber verschiedene Objekte -> Identitaet => FALSE.
    out = run_gb(
        "DIM a[2] AS INTEGER\n"
        "DIM b[2] AS INTEGER\n"
        "PRINT a = b\n"
    ).strip()
    assert out == "FALSE"


def test_map_equals_itself(run_gb):
    out = run_gb(
        'DIM m AS MAP OF INTEGER\n'
        'MAPPUT(m, "k", 1)\n'
        'PRINT m = m\n'
    ).strip()
    assert out == "TRUE"


# ------------------------------------------------ 3) Ganzzahl-Ueberlauf
def test_add_overflow_errors(run_gb):
    with pytest.raises(GameBasicError, match="(?i)Ueberlauf"):
        run_gb("PRINT 9223372036854775807 + 1\n")


def test_pow_overflow_errors(run_gb):
    with pytest.raises(GameBasicError, match="(?i)Ueberlauf"):
        run_gb("PRINT 10 ^ 30\n")


def test_mul_overflow_errors(run_gb):
    with pytest.raises(GameBasicError, match="(?i)Ueberlauf"):
        run_gb("PRINT 1000000000000 * 1000000000000\n")


def test_oversized_int_literal_errors(run_gb):
    with pytest.raises(GameBasicError, match="(?i)zu gross|gross"):
        run_gb("PRINT 99999999999999999999\n")


def test_normal_math_unaffected(run_gb):
    out = run_gb("PRINT 2 + 3 * 4\nPRINT 2 ^ 10\nPRINT 100 - 250\n").splitlines()
    assert out == ["14", "1024", "-150"]


# ------------------------------------------------ MITTEL: Konsistenz
def test_map_of_integer_accepts_whole_float(run_gb):
    # wie ARRAY OF INTEGER / Skalar: ganzzahliges FLOAT wird zu INTEGER.
    out = run_gb(
        'DIM m AS MAP OF INTEGER\n'
        'MAPPUT(m, "k", 3.0)\n'
        'PRINT MAPGET(m, "k")\n'
    ).strip()
    assert out == "3"


def test_map_of_integer_rejects_fractional_float(run_gb):
    with pytest.raises(GameBasicError, match="(?i)INTEGER"):
        run_gb('DIM m AS MAP OF INTEGER\nMAPPUT(m, "k", 3.5)\n')


def test_format_d_rejects_bool(run_gb):
    with pytest.raises(GameBasicError, match="(?i)erwartet Zahl"):
        run_gb('PRINT FORMAT$(TRUE, "%d")\n')


def test_format_d_int_still_ok(run_gb):
    assert run_gb('PRINT FORMAT$(42, "%05d")\n').strip() == "00042"
