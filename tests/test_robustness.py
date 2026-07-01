"""Golden-Tests fuer die Robustheits-Fixes (Review-Funde):
1. Falsche Argumentzahl in variadischen Builtins -> klarer Laufzeitfehler statt
   Rust-Panic/Absturz (Sicherheitsnetz via catch_unwind).
2. Referenz-Typen (ARRAY/MAP) sind ueber Identitaet gleich -- u.a. `a = a` TRUE.
3. Ganzzahl-Ueberlauf (+,-,*,^, zu grosses Literal) -> klarer Laufzeitfehler
   statt still falschem Wert.
4. Abgeschnittener IF/ELSEIF/ELSE-Block (kein END IF vor EOF) -> klarer
   Parse-Fehler "END IF erwartet" statt eines beliebigen Expression-Fehlers
   (die THEN/ELSEIF/ELSE-Body-Schleifen hatten frueher keinen at_end()-Check).
"""
import pytest

from gamebasic.errors import GameBasicError, ParseError


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


# ------------------------------------------------ NIEDRIG: Feinschliff
def test_instr_empty_needle_out_of_range(run_gb):
    out = run_gb(
        'PRINT INSTR("abc", "", 5)\n'   # Start ausserhalb -> -1
        'PRINT INSTR("abc", "", 2)\n'   # in-range -> Start
        'PRINT INSTR("abcdef", "cd")\n'
    ).splitlines()
    assert out == ["-1", "2", "2"]


def test_array_sum_overflow_errors(run_gb):
    with pytest.raises(GameBasicError, match="(?i)Ueberlauf"):
        run_gb("DIM a[2] AS INTEGER\na[0]=9223372036854775807\na[1]=1\nPRINT ARRAY_SUM(a)\n")


def test_curve_bezier2_type_error_names_builtin(run_gb):
    # Fehlertext nennt CURVE_BEZIER2 (frueher Platzhalter "B").
    with pytest.raises(GameBasicError, match="CURVE_BEZIER2"):
        run_gb('IMPORT "curves"\nPRINT CURVE_BEZIER2(0.5,"x",0,0,0,0,0,0,0)\n')


# ------------------------------------------------ 4) Abgeschnittenes Block-IF
@pytest.mark.parametrize("src", [
    'IF 1 = 1 THEN\nPRINT "hi"',                              # kein END IF
    'IF 1 = 1 THEN\nPRINT "a"\nELSEIF 2 = 2 THEN\nPRINT "b"',  # bricht im ELSEIF-Zweig ab
    'IF 1 = 1 THEN\nPRINT "a"\nELSE\nPRINT "b"',               # bricht im ELSE-Zweig ab
])
def test_unterminated_if_is_clean_parse_error(run_gb, src):
    with pytest.raises(ParseError, match="END IF erwartet"):
        run_gb(src)
