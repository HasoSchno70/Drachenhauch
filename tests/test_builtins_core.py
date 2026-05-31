"""Tests fuer Core-Built-ins (Math, Strings, Bitwise, Maps).

Built-ins werden direkt ueber das BUILTINS-Dict aufgerufen - kein Lexer/Parser
in der Test-Schleife, damit die Tests schnell und punktuell sind.
"""
import math
import pytest

from gamebasic.errors import GBRuntimeError, TypeMismatchError


# --- Math ----------------------------------------------------------

def test_sin_cos_zero(call_builtin):
    assert call_builtin("sin", [0.0]) == 0.0
    assert call_builtin("cos", [0.0]) == 1.0


def test_atan2_quadrants(call_builtin):
    assert call_builtin("atan2", [1.0, 0.0]) == pytest.approx(math.pi / 2)
    assert call_builtin("atan2", [0.0, 1.0]) == 0.0


def test_floor_ceil_round(call_builtin):
    assert call_builtin("floor", [3.7]) == 3
    assert call_builtin("ceil", [3.2]) == 4
    assert call_builtin("round", [2.5]) == 2  # bankers' rounding (Python default)


def test_log_one_arg(call_builtin):
    assert call_builtin("log", [math.e]) == pytest.approx(1.0)


def test_log_with_base(call_builtin):
    assert call_builtin("log", [8, 2]) == pytest.approx(3.0)


def test_log_negative_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="muss > 0"):
        call_builtin("log", [-1])


def test_min_max_variadic(call_builtin):
    assert call_builtin("min", [5, 2, 9, 1, 7]) == 1
    assert call_builtin("max", [5, 2, 9, 1, 7]) == 9


def test_min_empty_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match=">= 1"):
        call_builtin("min", [])


def test_clamp(call_builtin):
    assert call_builtin("clamp", [-5, 0, 100]) == 0
    assert call_builtin("clamp", [42, 0, 100]) == 42
    assert call_builtin("clamp", [150, 0, 100]) == 100


def test_sign(call_builtin):
    assert call_builtin("sign", [-3]) == -1
    assert call_builtin("sign", [0]) == 0
    assert call_builtin("sign", [7.5]) == 1


def test_sqr_negative_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="negativer Zahl"):
        call_builtin("sqr", [-1])


def test_abs(call_builtin):
    assert call_builtin("abs", [-7]) == 7
    assert call_builtin("abs", [3.5]) == 3.5


def test_int_truncates_to_floor(call_builtin):
    assert call_builtin("int", [3.7]) == 3
    assert call_builtin("int", [-1.5]) == -2  # math.floor


def test_rgb_packing(call_builtin):
    assert call_builtin("rgb", [255, 128, 64]) == 0xFF8040


def test_rgb_out_of_range_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="0..255"):
        call_builtin("rgb", [300, 0, 0])


# --- Type-Errors ---------------------------------------------------

def test_sin_string_raises(call_builtin):
    with pytest.raises(TypeMismatchError, match="erwartet Zahl"):
        call_builtin("sin", ["nope"])


def test_sin_bool_raises_as_not_number(call_builtin):
    # Bool ist semantisch keine Zahl in GB, auch wenn isinstance(True, int) True ist.
    with pytest.raises(TypeMismatchError):
        call_builtin("sin", [True])


def test_arity_error_message_includes_received_count(call_builtin):
    with pytest.raises(GBRuntimeError, match="erhalten 2"):
        call_builtin("sin", [1.0, 2.0])


# --- Strings -------------------------------------------------------

def test_upper_lower_aliases(call_builtin):
    assert call_builtin("upper$", ["hallo"]) == "HALLO"
    assert call_builtin("upper", ["hallo"]) == "HALLO"
    assert call_builtin("lower$", ["HALLO"]) == "hallo"


def test_left_right_mid(call_builtin):
    assert call_builtin("left$", ["GameBasic", 4]) == "Game"
    assert call_builtin("right$", ["GameBasic", 5]) == "Basic"
    assert call_builtin("mid$", ["GameBasic", 4, 5]) == "Basic"
    assert call_builtin("mid$", ["GameBasic", 4]) == "Basic"  # ohne Anzahl: bis Ende


def test_left_negative_clamped(call_builtin):
    assert call_builtin("left$", ["abc", -3]) == ""


def test_instr_found_and_missing(call_builtin):
    assert call_builtin("instr", ["hello world", "world"]) == 6
    assert call_builtin("instr", ["hello", "xyz"]) == -1


def test_replace(call_builtin):
    assert call_builtin("replace$", ["a-b-c", "-", "_"]) == "a_b_c"


def test_trim(call_builtin):
    assert call_builtin("trim$", ["  hi  "]) == "hi"


def test_padl_padr_with_default_filler(call_builtin):
    assert call_builtin("padl$", ["x", 5]) == "    x"
    assert call_builtin("padr$", ["x", 5]) == "x    "


def test_padl_with_custom_filler(call_builtin):
    assert call_builtin("padl$", ["42", 6, "0"]) == "000042"


def test_repeat(call_builtin):
    assert call_builtin("repeat$", ["ab", 3]) == "ababab"


def test_space(call_builtin):
    assert call_builtin("space$", [4]) == "    "


def test_hex(call_builtin):
    assert call_builtin("hex$", [255]) == "FF"
    assert call_builtin("hex$", [0xCAFE]) == "CAFE"


def test_chr_asc_roundtrip(call_builtin):
    assert call_builtin("chr$", [65]) == "A"
    assert call_builtin("asc", ["A"]) == 65
    assert call_builtin("asc", ["Abc"]) == 65  # nur erstes Zeichen


# --- Conversions ---------------------------------------------------

def test_str_int(call_builtin):
    assert call_builtin("str$", [42]) == "42"


def test_str_float_keeps_decimal(call_builtin):
    assert call_builtin("str$", [3.0]) == "3.0"


def test_str_bool(call_builtin):
    assert call_builtin("str$", [True]) == "TRUE"
    assert call_builtin("str$", [False]) == "FALSE"


def test_val(call_builtin):
    assert call_builtin("val", ["42"]) == 42
    assert call_builtin("val", ["3.14"]) == 3.14
    assert call_builtin("val", ["xyz"]) == 0


# --- Bitwise -------------------------------------------------------
# Frueher gab es Built-in-Funktionen BITAND/BITOR/BITXOR/BITNOT/SHL/SHR.
# Sie wurden durch native Operatoren ersetzt -- siehe tests/test_bitwise.py
# fuer die Operator-Tests. Hier testen wir nur, dass die alten Built-ins
# wirklich verschwunden sind (verhindert versehentliche Re-Registrierung).

def test_old_bitwise_builtins_are_gone(call_builtin):
    import pytest
    for name in ("bitand", "bitor", "bitxor", "bitnot", "shl", "shr"):
        with pytest.raises(KeyError):
            call_builtin(name, [0, 0])


# --- Time/Random ---------------------------------------------------

def test_randomize_makes_rnd_deterministic(call_builtin):
    call_builtin("randomize", [42])
    seq1 = [call_builtin("rnd", [100]) for _ in range(5)]
    call_builtin("randomize", [42])
    seq2 = [call_builtin("rnd", [100]) for _ in range(5)]
    assert seq1 == seq2


def test_time_date_format(call_builtin):
    t = call_builtin("time$", [])
    d = call_builtin("date$", [])
    # Form HH:MM:SS und YYYY-MM-DD
    assert len(t) == 8 and t[2] == ":" and t[5] == ":"
    assert len(d) == 10 and d[4] == "-" and d[7] == "-"


# --- Collides ------------------------------------------------------

def test_collides_overlapping(call_builtin):
    # Zwei sich ueberlappende Rechtecke
    assert call_builtin("collides", [0, 0, 10, 10, 5, 5, 10, 10]) is True


def test_collides_disjoint(call_builtin):
    assert call_builtin("collides", [0, 0, 10, 10, 100, 100, 10, 10]) is False


def test_collides_touching_edges_dont_count(call_builtin):
    # x1+w1 = x2 -> noch keine Ueberlappung
    assert call_builtin("collides", [0, 0, 10, 10, 10, 0, 10, 10]) is False
