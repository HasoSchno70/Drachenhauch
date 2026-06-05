"""Tests fuer Bitwise-Operatoren: BAND, BOR, BXOR, BNOT, SHL, SHR.

Strikt INTEGER -- Bool und Float werden in beiden Pfaden (Tree-Walker und
Python-VM) abgelehnt. Der Cython-Pfad braucht einen Recompile, damit die
neuen Ops sichtbar werden -- siehe CLAUDE.md "Build und Test".
"""
import pytest


# --- BAND / BOR / BXOR ------------------------------------------------

def test_band_basic(run_gb, run_vm):
    src = "PRINT &HF0 BAND &H0F\nPRINT &HFF BAND &H0F\nPRINT &HAA BAND &H55"
    expected = "0\n15\n0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_bor_basic(run_gb, run_vm):
    src = "PRINT &HF0 BOR &H0F\nPRINT 1 BOR 2 BOR 4 BOR 8"
    expected = "255\n15\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_bxor_basic(run_gb, run_vm):
    src = "PRINT &HFF BXOR &H0F\nPRINT 5 BXOR 5\nPRINT 5 BXOR 0"
    expected = "240\n0\n5\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- SHL / SHR -------------------------------------------------------

def test_shl_basic(run_gb, run_vm):
    src = "PRINT 1 SHL 0\nPRINT 1 SHL 8\nPRINT 3 SHL 4"
    expected = "1\n256\n48\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_shr_basic(run_gb, run_vm):
    src = "PRINT 256 SHR 8\nPRINT 255 SHR 4\nPRINT 1 SHR 1"
    expected = "1\n15\n0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_shift_negative_count_throws(run_gb, run_vm):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError):
        run_gb("PRINT 1 SHL -1")
    with pytest.raises(GBRuntimeError):
        run_vm("PRINT 1 SHR -1")


# --- BNOT (unaer) ----------------------------------------------------

def test_bnot_basic(run_gb, run_vm):
    src = "PRINT BNOT 0\nPRINT BNOT 5"
    # Python ~x = -(x+1) -- gleiche Konvention wie GameBasic.
    expected = "-1\n-6\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_bnot_double_application(run_gb, run_vm):
    src = "PRINT BNOT BNOT 42"
    expected = "42\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Praezedenz ------------------------------------------------------

def test_bitwise_lower_than_addition(run_gb, run_vm):
    """`a + b BAND c` parsen als `(a + b) BAND c`, weil + hoehere Praezedenz hat."""
    # 5 + 3 = 8, 8 BAND 6 = 0
    src = "PRINT 5 + 3 BAND 6"
    expected = "0\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_bitwise_higher_than_comparison(run_gb, run_vm):
    """`a BAND b = c` parsen als `(a BAND b) = c`."""
    src = "IF 12 BAND 4 = 4 THEN PRINT \"yes\" ELSE PRINT \"no\""
    expected = "yes\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_bitwise_left_assoc(run_gb, run_vm):
    """Alle Bitwise auf einer Ebene, links-assoziativ."""
    # ((1 BOR 2) BAND 3) = 3 BAND 3 = 3
    src = "PRINT 1 BOR 2 BAND 3"
    expected = "3\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_explicit_paren_for_c_style(run_gb, run_vm):
    """Wer C-Praezedenz will, klammert. (1 BOR (2 BAND 3)) = 3."""
    src = "PRINT 1 BOR (2 BAND 3)"
    expected = "3\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


# --- Type-Strictness -------------------------------------------------

def test_band_rejects_float(run_gb, run_vm):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError):
        run_gb("PRINT 1.5 BAND 1")
    with pytest.raises(GBRuntimeError):
        run_vm("PRINT 1.5 BAND 1")


def test_band_rejects_bool(run_gb, run_vm):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError):
        run_gb("PRINT TRUE BAND 1")
    with pytest.raises(GBRuntimeError):
        run_vm("PRINT TRUE BAND 1")


def test_bnot_rejects_float(run_gb, run_vm):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError):
        run_gb("PRINT BNOT 1.5")
    with pytest.raises(GBRuntimeError):
        run_vm("PRINT BNOT 1.5")


# --- Compound mit Variablen ------------------------------------------

def test_bitwise_with_variables(run_gb, run_vm):
    src = '''
DIM mask AS INTEGER
DIM flags AS INTEGER
mask = &HFF
flags = &HA5
PRINT flags BAND mask
PRINT flags SHR 4
PRINT flags BXOR mask
'''
    expected = "165\n10\n90\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected


def test_bitwise_in_if(run_gb, run_vm):
    """Praktischer Use-Case: Flag-Pruefung."""
    src = '''
CONST FLAG_READ = 1
CONST FLAG_WRITE = 2
CONST FLAG_EXEC = 4
DIM perms AS INTEGER
perms = FLAG_READ BOR FLAG_WRITE
IF perms BAND FLAG_READ <> 0 THEN PRINT "kann lesen"
IF perms BAND FLAG_WRITE <> 0 THEN PRINT "kann schreiben"
IF perms BAND FLAG_EXEC <> 0 THEN PRINT "kann ausfuehren"
'''
    expected = "kann lesen\nkann schreiben\n"
    assert run_gb(src) == expected
    assert run_vm(src) == expected
