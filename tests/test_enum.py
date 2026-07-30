"""Tests fuer ENUM-Statement.

Wir testen Tree-Walker UND Python-VM, damit ENUM-Verhalten zwischen den
Pfaden konsistent bleibt. Cython-VM ist optional und wird (wenn verfuegbar)
in test_enum_native.py separat gepflegt.
"""
import pytest

from gamebasic.errors import ParseError, GBRuntimeError


# Beide Helper geben stdout zurueck, identisch verwendbar.
@pytest.fixture(params=["tw", "vm"])
def run_either(request, run_gb, run_vm):
    return run_gb if request.param == "tw" else run_vm


# --- Compact-Form ---------------------------------------------------

def test_compact_form_auto_values(run_either):
    out = run_either(
        'ENUM State = MENU, PLAYING, PAUSED\n'
        'PRINT State.MENU\n'
        'PRINT State.PLAYING\n'
        'PRINT State.PAUSED\n'
    )
    assert out.split() == ["0", "1", "2"]


def test_compact_form_explicit_values(run_either):
    out = run_either(
        'ENUM Color = RED = 1, GREEN = 2, BLUE = 4\n'
        'PRINT Color.RED\n'
        'PRINT Color.GREEN\n'
        'PRINT Color.BLUE\n'
    )
    assert out.split() == ["1", "2", "4"]


def test_compact_form_mixed_values(run_either):
    """Nach explicit weiterzaehlen ab dem expliziten Wert + 1."""
    out = run_either(
        'ENUM E = A, B = 10, C, D = 20, E\n'
        'PRINT E.A\n'
        'PRINT E.B\n'
        'PRINT E.C\n'
        'PRINT E.D\n'
        'PRINT E.E\n'
    )
    assert out.split() == ["0", "10", "11", "20", "21"]


def test_compact_form_negative_value(run_either):
    out = run_either(
        'ENUM E = LOW = -1, ZERO, ONE\n'
        'PRINT E.LOW\n'
        'PRINT E.ZERO\n'
        'PRINT E.ONE\n'
    )
    assert out.split() == ["-1", "0", "1"]


# --- Block-Form -----------------------------------------------------

def test_block_form_auto_values(run_either):
    out = run_either(
        'ENUM Dir\n'
        '    NORTH\n'
        '    SOUTH\n'
        '    EAST\n'
        '    WEST\n'
        'END ENUM\n'
        'PRINT Dir.NORTH\n'
        'PRINT Dir.SOUTH\n'
        'PRINT Dir.EAST\n'
        'PRINT Dir.WEST\n'
    )
    assert out.split() == ["0", "1", "2", "3"]


def test_block_form_explicit_values(run_either):
    out = run_either(
        'ENUM Permission\n'
        '    NONE = 0\n'
        '    READ = 1\n'
        '    WRITE = 2\n'
        '    EXEC = 4\n'
        '    RW = 3\n'
        'END ENUM\n'
        'PRINT Permission.NONE\n'
        'PRINT Permission.READ\n'
        'PRINT Permission.WRITE\n'
        'PRINT Permission.EXEC\n'
        'PRINT Permission.RW\n'
    )
    assert out.split() == ["0", "1", "2", "4", "3"]


# --- Keyword-Member-Namen -------------------------------------------

def test_keyword_member_names_allowed(run_either):
    """READ, FILE, DATA, NONE etc. sind GB-Keywords, sollten aber als
    qualifizierte ENUM-Member funktionieren."""
    out = run_either(
        'ENUM Tag = NEW, READ, DATA, FILE\n'
        'PRINT Tag.NEW\n'
        'PRINT Tag.READ\n'
        'PRINT Tag.DATA\n'
        'PRINT Tag.FILE\n'
    )
    assert out.split() == ["0", "1", "2", "3"]


# --- DIM x AS EnumName ----------------------------------------------

def test_dim_as_enum_works_like_integer(run_either):
    out = run_either(
        'ENUM State = MENU, PLAYING\n'
        'DIM s AS State\n'
        'PRINT s\n'                 # default integer 0
        's = State.PLAYING\n'
        'PRINT s\n'
    )
    assert out.split() == ["0", "1"]


def test_dim_as_enum_accepts_int_assignment(run_either):
    """ENUM-Wert ist Integer, also ist auch direkter Int-Zuweisung erlaubt."""
    out = run_either(
        'ENUM State = MENU, PLAYING\n'
        'DIM s AS State\n'
        's = 42\n'
        'PRINT s\n'
    )
    assert out.split() == ["42"]


def test_enum_value_in_arithmetic(run_either):
    out = run_either(
        'ENUM E = A = 10, B = 20\n'
        'PRINT E.A + E.B\n'
        'PRINT E.B - E.A\n'
    )
    assert out.split() == ["30", "10"]


def test_enum_in_select_case(run_either):
    out = run_either(
        'ENUM Mood = HAPPY, SAD, ANGRY\n'
        'DIM m AS Mood\n'
        'm = Mood.SAD\n'
        'SELECT CASE m\n'
        '    CASE Mood.HAPPY\n'
        '        PRINT "froh"\n'
        '    CASE Mood.SAD\n'
        '        PRINT "traurig"\n'
        '    CASE ELSE\n'
        '        PRINT "sonstwas"\n'
        'END SELECT\n'
    )
    assert out.strip() == "traurig"


# --- Fehler-Faelle --------------------------------------------------

def test_unknown_member_raises(run_either):
    with pytest.raises(GBRuntimeError, match="hat keinen Member"):
        run_either(
            'ENUM E = A, B\n'
            'PRINT E.SCHWURBEL\n'
        )


def test_duplicate_member_raises(run_gb):
    with pytest.raises(ParseError, match="doppelt"):
        run_gb('ENUM E = A, A, B\n')


def test_no_members_raises(run_gb):
    with pytest.raises(ParseError, match="mindestens ein"):
        run_gb('ENUM Empty\nEND ENUM\n')


def test_non_literal_value_raises(run_either):
    """ENUM-Member-Werte muessen Compile-Time-Integer-Literale sein - keine
    Variablen, keine Funktionsaufrufe."""
    with pytest.raises(GBRuntimeError, match="Literal"):
        run_either(
            'CONST N AS INTEGER = 5\n'
            'ENUM E = A = N\n'
        )


def test_redeclaration_with_different_members_raises(run_gb):
    """Tree-Walker: zweite ENUM-Deklaration mit anderen Members -> Fehler."""
    with pytest.raises(GBRuntimeError, match="anderweitig vergeben"):
        run_gb(
            'ENUM E = A, B\n'
            'ENUM E = X, Y\n'
        )


def test_redeclaration_idempotent(run_gb):
    """Identische zweite Deklaration ist OK (z.B. via doppeltem IMPORT)."""
    out = run_gb(
        'ENUM E = A, B\n'
        'ENUM E = A, B\n'
        'PRINT E.A\n'
    )
    assert out.strip() == "0"


# --- Bench-Equivalence ----------------------------------------------

def test_tw_and_vm_produce_same_output(run_gb, run_vm):
    src = (
        'ENUM Color\n'
        '    RED = 1\n'
        '    GREEN = 2\n'
        '    BLUE = 4\n'
        'END ENUM\n'
        'DIM c AS Color\n'
        'c = Color.GREEN\n'
        'PRINT c\n'
        'PRINT Color.RED + Color.BLUE\n'
    )
    assert run_gb(src) == run_vm(src)
