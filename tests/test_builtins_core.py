"""Tests fuer Core-Built-ins (Math, Strings, Bitwise, Time, Collides).

Golden-Tests gegen `dhrt` (Stufe B): PRINT <builtin-call> + Soll-Ausgabe.
Frueher via `call_builtin` gegen die Python-Impl (in Phase 8 geloescht).
"""
import math
import pytest

from drachenhauch.errors import DHRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _p(run_gb, *exprs):
    return _lines(run_gb("".join(f"PRINT {e}\n" for e in exprs)))


# --- Math ----------------------------------------------------------

def test_sin_cos_zero(run_gb):
    assert _p(run_gb, "SIN(0.0)", "COS(0.0)") == ["0.0", "1.0"]


def test_atan2_quadrants(run_gb):
    out = _p(run_gb, "ATAN2(1.0, 0.0)", "ATAN2(0.0, 1.0)")
    assert float(out[0]) == pytest.approx(math.pi / 2)
    assert out[1] == "0.0"


def test_floor_ceil_round(run_gb):
    # round(2.5) == 2: bankers' rounding (wie Python)
    assert _p(run_gb, "FLOOR(3.7)", "CEIL(3.2)", "ROUND(2.5)") == ["3", "4", "2"]


def test_log_one_arg(run_gb):
    assert float(_p(run_gb, "LOG(2.718281828459045)")[0]) == pytest.approx(1.0)


def test_log_with_base(run_gb):
    assert float(_p(run_gb, "LOG(8, 2)")[0]) == pytest.approx(3.0)


def test_log_negative_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="muss > 0"):
        run_gb("PRINT LOG(-1)\n")


def test_min_max_variadic(run_gb):
    assert _p(run_gb, "MIN(5, 2, 9, 1, 7)", "MAX(5, 2, 9, 1, 7)") == ["1", "9"]


def test_min_empty_raises(run_gb):
    # dhrt-Wortlaut: "mind. 1 Argument" (TW sagte ">= 1").
    with pytest.raises(DHRuntimeError, match="mind. 1"):
        run_gb("PRINT MIN()\n")


def test_clamp(run_gb):
    assert _p(run_gb, "CLAMP(-5, 0, 100)", "CLAMP(42, 0, 100)",
              "CLAMP(150, 0, 100)") == ["0", "42", "100"]


def test_sign(run_gb):
    assert _p(run_gb, "SIGN(-3)", "SIGN(0)", "SIGN(7.5)") == ["-1", "0", "1"]


def test_sqr_negative_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="negativer Zahl"):
        run_gb("PRINT SQR(-1)\n")


def test_abs(run_gb):
    assert _p(run_gb, "ABS(-7)", "ABS(3.5)") == ["7", "3.5"]


def test_int_truncates_to_floor(run_gb):
    assert _p(run_gb, "INT(3.7)", "INT(-1.5)") == ["3", "-2"]


def test_rgb_packing(run_gb):
    assert _p(run_gb, "RGB(255, 128, 64)") == [str(0xFF8040)]


def test_rgb_out_of_range_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="0..255"):
        run_gb("PRINT RGB(300, 0, 0)\n")


def test_rgb_nimmt_kommazahlen_und_rundet(run_gb):
    """RGB/RGBA runden eine Kommazahl, statt sie abzulehnen.

    Bis 2026-08-23 waren sie die einzigen Ausreisser unter den Zeichen-
    Befehlen: CIRCLE, BOX, LINE, PLOT, TEXT und SETFPS nehmen alle eine
    Kommazahl -- ausgerechnet die Farbe nicht, obwohl sie am haeufigsten
    ausgerechnet wird. `x * 255 / 640` fuer einen Verlauf brach ab, und
    `dhrt --check` konnte nicht warnen (der Fehler haengt am WERT, nicht
    am Text). Im Einsteigerbuch fiel genau das fuenfmal an.
    """
    assert _p(run_gb, "RGB(12.7, 0, 0)") == [str(13 << 16)]      # kaufmaennisch
    assert _p(run_gb, "RGB(12.2, 0, 0)") == [str(12 << 16)]
    assert _p(run_gb, "RGBA(0, 0, 0, 200.6)") == [str(201 << 24)]
    # Der Anlassfall: ein Verlauf, ausgerechnet aus einer Division.
    assert run_gb("DIM x AS INTEGER\nx = 320\n"
                  "PRINT RGB(x * 255 / 640, 0, 0)\n").strip() == str(128 << 16)


def test_rgb_bereich_gilt_auch_gerundet(run_gb):
    """Gerundet heisst nicht geklemmt -- 255.6 wird 256 und ist zu gross."""
    with pytest.raises(DHRuntimeError, match="0..255"):
        run_gb("PRINT RGB(255.6, 0, 0)\n")


# --- Type-Errors ---------------------------------------------------

def test_sin_string_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="erwartet Zahl"):
        run_gb('PRINT SIN("nope")\n')


def test_sin_bool_raises_as_not_number(run_gb):
    with pytest.raises(DHRuntimeError):
        run_gb("PRINT SIN(TRUE)\n")


def test_arity_error_message_includes_received_count(run_gb):
    with pytest.raises(DHRuntimeError, match="erhalten 2"):
        run_gb("PRINT SIN(1.0, 2.0)\n")


# --- Strings -------------------------------------------------------

def test_upper_lower_aliases(run_gb):
    assert _p(run_gb, 'UPPER$("hallo")', 'UPPER("hallo")',
              'LOWER$("HALLO")') == ["HALLO", "HALLO", "hallo"]


def test_left_right_mid(run_gb):
    # "TestString" statt des Produktnamens: der Test lebt davon, dass die
    # Zeichenkette in zwei erkennbare Woerter zerfaellt (frueher Game+Basic).
    # Mit dem Namen als Testdaten bricht er bei jeder Umbenennung -- genau
    # das ist beim Wechsel auf Drachenhauch passiert, weil die EINGABE
    # mitwanderte und die erwarteten Ausgaben nicht.
    assert _p(run_gb, 'LEFT$("TestString", 4)', 'RIGHT$("TestString", 6)',
              'MID$("TestString", 4, 6)', 'MID$("TestString", 4)') == \
        ["Test", "String", "String", "String"]


def test_left_negative_clamped(run_gb):
    assert _p(run_gb, '"[" + LEFT$("abc", -3) + "]"') == ["[]"]


def test_instr_found_and_missing(run_gb):
    assert _p(run_gb, 'INSTR("hello world", "world")', 'INSTR("hello", "xyz")') == ["6", "-1"]


def test_replace(run_gb):
    assert _p(run_gb, 'REPLACE$("a-b-c", "-", "_")') == ["a_b_c"]


def test_trim(run_gb):
    assert _p(run_gb, 'TRIM$("  hi  ")') == ["hi"]


def test_padl_padr_with_default_filler(run_gb):
    assert _p(run_gb, '"[" + PADL$("x", 5) + "]"',
              '"[" + PADR$("x", 5) + "]"') == ["[    x]", "[x    ]"]


def test_padl_with_custom_filler(run_gb):
    assert _p(run_gb, 'PADL$("42", 6, "0")') == ["000042"]


def test_repeat(run_gb):
    assert _p(run_gb, 'REPEAT$("ab", 3)') == ["ababab"]


def test_space(run_gb):
    assert _p(run_gb, '"[" + SPACE$(4) + "]"') == ["[    ]"]


def test_hex(run_gb):
    assert _p(run_gb, "HEX$(255)", "HEX$(51966)") == ["FF", "CAFE"]


def test_chr_asc_roundtrip(run_gb):
    assert _p(run_gb, "CHR$(65)", 'ASC("A")', 'ASC("Abc")') == ["A", "65", "65"]


# --- Conversions ---------------------------------------------------

def test_str_int(run_gb):
    assert _p(run_gb, "STR$(42)") == ["42"]


def test_str_float_keeps_decimal(run_gb):
    assert _p(run_gb, "STR$(3.0)") == ["3.0"]


def test_str_bool(run_gb):
    assert _p(run_gb, "STR$(TRUE)", "STR$(FALSE)") == ["TRUE", "FALSE"]


def test_val(run_gb):
    assert _p(run_gb, 'VAL("42")', 'VAL("3.14")', 'VAL("xyz")') == ["42", "3.14", "0"]


# --- Bitwise -------------------------------------------------------
# Frueher gab es Built-in-Funktionen BITAND/BITOR/...; sie wurden durch native
# Operatoren ersetzt (siehe tests/test_bitwise.py). Hier nur: die alten Namen
# sind keine Built-ins mehr -> Aufruf wirft.

def test_old_bitwise_builtins_are_gone(run_gb):
    # SHL/SHR sind heute Operator-Keywords -> Parse-Fehler; die uebrigen sind
    # schlicht keine Built-ins mehr -> Laufzeitfehler. Beide leiten von
    # DrachenhauchError ab.
    from drachenhauch.errors import DrachenhauchError
    for name in ("BITAND", "BITOR", "BITXOR", "BITNOT", "SHL", "SHR"):
        with pytest.raises(DrachenhauchError):
            run_gb(f"PRINT {name}(0, 0)\n")


# --- Time/Random ---------------------------------------------------

def test_randomize_makes_rnd_deterministic(run_gb):
    src = ("RANDOMIZE(42)\nDIM i AS INTEGER\n"
           "FOR i = 0 TO 4\n    PRINT RND(100)\nNEXT\n"
           "RANDOMIZE(42)\n"
           "FOR i = 0 TO 4\n    PRINT RND(100)\nNEXT\n")
    seq = _lines(run_gb(src))
    assert seq[:5] == seq[5:10]


def test_time_date_format(run_gb):
    t = _p(run_gb, "TIME$()")[0]
    d = _p(run_gb, "DATE$()")[0]
    assert len(t) == 8 and t[2] == ":" and t[5] == ":"
    assert len(d) == 10 and d[4] == "-" and d[7] == "-"


# --- Collides ------------------------------------------------------

def test_collides_overlapping(run_gb):
    assert _p(run_gb, "COLLIDES(0, 0, 10, 10, 5, 5, 10, 10)") == ["TRUE"]


def test_collides_disjoint(run_gb):
    assert _p(run_gb, "COLLIDES(0, 0, 10, 10, 100, 100, 10, 10)") == ["FALSE"]


def test_collides_touching_edges_dont_count(run_gb):
    assert _p(run_gb, "COLLIDES(0, 0, 10, 10, 10, 0, 10, 10)") == ["FALSE"]
