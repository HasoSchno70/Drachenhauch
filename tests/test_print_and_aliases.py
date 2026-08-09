"""Golden-Tests: PRINT-Trenner (',' Leerzeichen / ';' kein Leerzeichen,
trailing-Trenner = kein Newline) + Komfort-String/Random-Aliase."""
import pytest

from drachenhauch.errors import DrachenhauchError


# --------------------------------------------------------- PRINT-Trenner
def test_comma_is_space(run_gb):
    assert run_gb('PRINT "a", "b", "c"\n') == "a b c\n"


def test_semicolon_is_no_space(run_gb):
    assert run_gb('PRINT "a"; "b"; "c"\n') == "abc\n"


def test_mixed_separators(run_gb):
    # x[space]5  dann  5[kein-space]!
    assert run_gb('PRINT "x", 5; "!"\n') == "x 5!\n"


def test_trailing_semicolon_suppresses_newline(run_gb):
    assert run_gb('PRINT "a";\nPRINT "b"\n') == "ab\n"


def test_trailing_comma_suppresses_newline(run_gb):
    # Trailing-Trenner unterdrueckt nur den Newline (kein Trenner ZWISCHEN
    # getrennten PRINT-Statements) -> "a" ohne Newline, dann "b".
    assert run_gb('PRINT "a",\nPRINT "b"\n') == "ab\n"


def test_empty_print_is_blank_line(run_gb):
    assert run_gb('PRINT "x"\nPRINT\nPRINT "y"\n') == "x\n\ny\n"


def test_numbers_with_semicolon(run_gb):
    assert run_gb('PRINT 1; "="; 2\n') == "1=2\n"


def test_print_separators_in_single_line_if(run_gb):
    out = run_gb('IF 1 = 1 THEN PRINT "a"; "b" ELSE PRINT "x"\n')
    assert out == "ab\n"


def test_colon_after_trailing_semicolon(run_gb):
    # trailing ';' direkt vor ':' (Statement-Trenner) -> kein Newline.
    assert run_gb('PRINT "a"; : PRINT "b"\n') == "ab\n"


# --------------------------------------------------------- String-Aliase
def test_ltrim_rtrim(run_gb):
    out = run_gb('PRINT "[" + LTRIM$("  hi  ") + "]"\nPRINT "[" + RTRIM$("  hi  ") + "]"\n')
    assert out.splitlines() == ["[hi  ]", "[  hi]"]


def test_count(run_gb):
    out = run_gb('PRINT COUNT("banana", "a")\nPRINT COUNT("aaaa", "aa")\nPRINT COUNT("x", "")\n')
    assert out.splitlines() == ["3", "2", "0"]


def test_title(run_gb):
    assert run_gb('PRINT TITLE$("hello WORLD foo")\n').strip() == "Hello World Foo"


# --------------------------------------------------------- WEIGHTED_CHOICE
def test_weighted_choice_forced(run_gb):
    # Gewichte [0, 0, 100] -> immer das dritte Element (deterministisch trotz PRNG).
    out = run_gb(
        'DIM v[3] AS STRING\nDIM w[3] AS INTEGER\n'
        'v[0]="a" : v[1]="b" : v[2]="c"\n'
        'w[0]=0 : w[1]=0 : w[2]=100\n'
        'DIM i AS INTEGER\n'
        'FOR i = 1 TO 20\n    PRINT WEIGHTED_CHOICE(v, w)\nNEXT\n')
    assert set(out.split()) == {"c"}


def test_weighted_choice_zero_total_raises(run_gb):
    with pytest.raises(DrachenhauchError, match="Summe der Gewichte"):
        run_gb('DIM v[2] AS STRING\nDIM w[2] AS INTEGER\n'
               'v[0]="a" : v[1]="b"\nw[0]=0 : w[1]=0\n'
               'PRINT WEIGHTED_CHOICE(v, w)\n')


def test_weighted_choice_length_mismatch_raises(run_gb):
    with pytest.raises(DrachenhauchError, match="gleich lang"):
        run_gb('DIM v[2] AS STRING\nDIM w[3] AS INTEGER\n'
               'v[0]="a" : v[1]="b"\n'
               'PRINT WEIGHTED_CHOICE(v, w)\n')
