"""Tests fuer die Sprach-Erweiterungen:
- REPEAT/UNTIL Post-Test-Schleife
- DATA/READ/RESTORE Inline-Daten
- Implizite Zeilenfortsetzung in offenen Klammern
- TIMER + Joystick-Builtins (Registrierung; Werte brauchen pygame)
"""
import pytest


# --- REPEAT / UNTIL ---------------------------------------------------

def test_repeat_basic(run_gb):
    out = run_gb('''
DIM i AS INTEGER
i = 0
REPEAT
    PRINT i
    i = i + 1
UNTIL i >= 3
''')
    assert out == "0\n1\n2\n"


def test_repeat_runs_at_least_once(run_gb):
    """Auch wenn die Bedingung sofort wahr ist, laeuft body einmal."""
    out = run_gb('''
DIM i AS INTEGER
i = 100
REPEAT
    PRINT \"once\"
UNTIL i = 100
''')
    assert out == "once\n"


def test_repeat_break_exits(run_gb):
    out = run_gb('''
DIM i AS INTEGER
i = 0
REPEAT
    PRINT i
    IF i = 1 THEN BREAK
    i = i + 1
UNTIL i >= 100
''')
    assert out == "0\n1\n"


def test_repeat_continue_jumps_to_condition(run_gb):
    out = run_gb('''
DIM i AS INTEGER
i = 0
REPEAT
    i = i + 1
    IF i MOD 2 = 1 THEN CONTINUE
    PRINT i
UNTIL i >= 6
''')
    assert out == "2\n4\n6\n"


# --- DATA / READ / RESTORE -------------------------------------------

def test_data_read_basic(run_gb):
    out = run_gb('''
DATA "Anna", 30
DIM name AS STRING
DIM age AS INTEGER
READ name, age
PRINT name, age
''')
    assert out == "Anna 30\n"


def test_data_read_multiple(run_gb):
    out = run_gb('''
DATA "Anna", 100, "Bert", 75, "Cilly", 50
DIM name AS STRING
DIM score AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO 2
    READ name, score
    PRINT name, score
NEXT
''')
    assert out == "Anna 100\nBert 75\nCilly 50\n"


def test_data_negative_numbers(run_gb):
    out = run_gb('''
DATA -5, -10, 100
DIM x AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO 2
    READ x
    PRINT x
NEXT
''')
    assert out == "-5\n-10\n100\n"


def test_data_floats(run_gb):
    out = run_gb('''
DATA 1.5, -2.75, 3.14
DIM x AS FLOAT
DIM i AS INTEGER
FOR i = 0 TO 2
    READ x
    PRINT x
NEXT
''')
    assert "1.5" in out
    assert "-2.75" in out
    assert "3.14" in out


def test_data_booleans(run_gb):
    out = run_gb('''
DATA TRUE, FALSE, TRUE
DIM b AS BOOLEAN
DIM i AS INTEGER
FOR i = 0 TO 2
    READ b
    PRINT b
NEXT
''')
    assert out == "TRUE\nFALSE\nTRUE\n"


def test_restore_resets_pointer(run_gb):
    out = run_gb('''
DATA 1, 2, 3
DIM x AS INTEGER
READ x
READ x
PRINT x
RESTORE
READ x
PRINT x
''')
    assert out == "2\n1\n"


def test_read_beyond_data_errors(run_gb):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError, match="keine DATA-Werte mehr"):
        run_gb('''
DATA 1
DIM x AS INTEGER
READ x
READ x
''')


def test_data_collected_from_inside_subs(run_gb):
    """Klassisches BASIC: DATA-Statements werden aus dem ganzen Programm
    gesammelt, auch wenn sie in SUB/FUNCTION-Bodies stehen."""
    out = run_gb('''
SUB lade()
    DATA "Hidden", 42
END SUB

DIM name AS STRING
DIM x AS INTEGER
READ name, x
PRINT name, x
''')
    assert out == "Hidden 42\n"


def test_data_into_array_element(run_gb):
    out = run_gb('''
DATA 10, 20, 30
DIM xs[3] AS INTEGER
DIM i AS INTEGER
FOR i = 0 TO 2
    READ xs[i]
NEXT
PRINT xs[0], xs[1], xs[2]
''')
    assert out == "10 20 30\n"


def test_data_type_coercion_string_into_int_errors(run_gb):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError):
        run_gb('''
DATA "nicht eine zahl"
DIM x AS INTEGER
READ x
''')


# --- Implizite Zeilenfortsetzung -------------------------------------

def test_multiline_call_in_parens(run_gb):
    """Newlines innerhalb offener Klammern werden vom Parser ignoriert."""
    out = run_gb('''
DIM s AS STRING
s = REPEAT$(
    "ab",
    3
)
PRINT s
''')
    assert "ababab" in out


def test_multiline_array_index(run_gb):
    out = run_gb('''
DIM xs[3] AS INTEGER
xs[0] = 10
xs[1] = 20
xs[2] = 30
PRINT xs[
    1
]
''')
    assert "20" in out


def test_explicit_underscore_continuation_still_works(run_gb):
    """Backwards-compat: das alte _\\n soll weiter funktionieren."""
    out = run_gb('''
DIM x AS INTEGER
x = 1 + _
    2 + _
    3
PRINT x
''')
    assert out.strip() == "6"


# --- TIMER + Joystick: Registrierung ---------------------------------

def test_timer_builtin_registered():
    from gamebasic.editor_qt.gbrt_meta import builtin_names_lower
    assert "timer" in builtin_names_lower()


def test_joystick_builtins_registered():
    from gamebasic.editor_qt.gbrt_meta import builtin_names_lower
    expected = {
        "joystick_count", "joystick_name",
        "joystick_axis", "joystick_button",
        "joystick_hat_x", "joystick_hat_y",
    }
    assert expected <= builtin_names_lower()


# --- Regression: REPEAT$ funktioniert weiter -------------------------

def test_repeat_string_builtin_still_works(run_gb):
    """REPEAT als bloßes Builtin wurde gedroppt zugunsten des Keywords,
    aber REPEAT$ bleibt - klassische BASIC-Konvention mit $-Suffix."""
    out = run_gb('PRINT REPEAT$("ab", 3)')
    assert "ababab" in out


# --- Hex / Binary-Literale -------------------------------------------

def test_hex_literal(run_gb):
    out = run_gb('PRINT &HFF\nPRINT &h0a\nPRINT &HCAFE')
    assert "255" in out
    assert "10" in out
    assert "51966" in out


def test_binary_literal(run_gb):
    out = run_gb('PRINT &B11010110\nPRINT &b1111')
    assert "214" in out
    assert "15" in out


def test_hex_literal_no_digits_errors(run_gb):
    from gamebasic.errors import LexerError
    with pytest.raises(LexerError, match="Hex-Literal"):
        run_gb('PRINT &H')


def test_amp_without_h_or_b_errors(run_gb):
    from gamebasic.errors import LexerError
    with pytest.raises(LexerError, match="Hex"):
        run_gb('PRINT &Z')


def test_hex_used_in_arithmetic(run_gb):
    out = run_gb('PRINT &H10 + &H20')
    assert "48" in out


# --- FORMAT$ ---------------------------------------------------------

def test_format_integer_padding(run_gb):
    out = run_gb('PRINT FORMAT$(42, "%05d")')
    assert "00042" in out


def test_format_float_precision(run_gb):
    out = run_gb('PRINT FORMAT$(3.14159, "%.2f")')
    assert "3.14" in out


def test_format_string_left_align(run_gb):
    out = run_gb('PRINT FORMAT$("hi", "%-5s|")')
    assert "hi   |" in out


def test_format_hex_via_mask(run_gb):
    out = run_gb('PRINT FORMAT$(255, "%04X")')
    assert "00FF" in out


def test_format_invalid_mask_errors(run_gb):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError, match="FORMAT"):
        run_gb('PRINT FORMAT$("nicht eine zahl", "%d")')


# --- INKEY$ / WAITKEY: nur Registrierung -----------------------------

def test_inkey_waitkey_registered():
    from gamebasic.editor_qt.gbrt_meta import builtin_names_lower
    n = builtin_names_lower()
    assert "inkey$" in n
    assert "waitkey" in n


# --- Default-Werte fuer Parameter -----------------------------------

def test_default_param_used_when_omitted(run_gb):
    out = run_gb('''
SUB greet(name AS STRING, prefix AS STRING = "Hallo")
    PRINT prefix, name
END SUB

greet("Anna")
''')
    assert "Hallo Anna" in out


def test_default_param_overridden(run_gb):
    out = run_gb('''
SUB greet(name AS STRING, prefix AS STRING = "Hallo")
    PRINT prefix, name
END SUB

greet("Bert", "Hi")
''')
    assert "Hi Bert" in out


def test_default_in_function(run_gb):
    out = run_gb('''
FUNCTION power(base AS FLOAT, exp AS INTEGER = 2) AS FLOAT
    DIM r AS FLOAT
    r = 1.0
    DIM i AS INTEGER
    FOR i = 1 TO exp
        r = r * base
    NEXT
    RETURN r
END FUNCTION

PRINT power(3.0)
PRINT power(3.0, 4)
''')
    assert "9" in out
    assert "81" in out


def test_default_can_reference_earlier_param(run_gb):
    """h's Default ist w - das soll bei Aufruf-Zeit ausgewertet werden,
    nachdem w schon gesetzt ist. Quadrat-Pattern."""
    out = run_gb('''
SUB show_rect(x AS INTEGER, y AS INTEGER, w AS INTEGER, h AS INTEGER = w)
    PRINT x, y, w, h
END SUB

show_rect(0, 0, 50)
''')
    assert "0 0 50 50" in out


def test_required_after_default_rejected(run_gb):
    from gamebasic.errors import ParseError
    with pytest.raises(ParseError, match="ohne Default"):
        run_gb('''
SUB foo(a AS INTEGER = 1, b AS INTEGER)
END SUB
''')


def test_too_few_args_errors(run_gb):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError, match="Parameter|Argument"):
        run_gb('''
SUB foo(a AS INTEGER, b AS INTEGER = 10)
END SUB

foo()
''')


def test_too_many_args_errors(run_gb):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError, match="Argument"):
        run_gb('''
SUB foo(a AS INTEGER, b AS INTEGER = 10)
END SUB

foo(1, 2, 3)
''')


def test_default_type_coercion(run_gb):
    """Default-Wert soll wie ein normaler Arg auf den Param-Typ gecoerced werden."""
    out = run_gb('''
SUB scale(x AS FLOAT = 1)
    PRINT x
END SUB

scale()
''')
    # Default 1 (INT) -> FLOAT 1.0
    assert "1" in out


# --- Compound Assignment ---------------------------------------------

def test_compound_plus_eq(run_gb):
    out = run_gb('DIM x AS INTEGER\nx = 10\nx += 5\nPRINT x')
    assert "15" in out


def test_compound_minus_eq(run_gb):
    out = run_gb('DIM x AS INTEGER\nx = 10\nx -= 3\nPRINT x')
    assert "7" in out


def test_compound_star_eq(run_gb):
    out = run_gb('DIM x AS INTEGER\nx = 10\nx *= 3\nPRINT x')
    assert "30" in out


def test_compound_slash_eq(run_gb):
    out = run_gb('DIM x AS INTEGER\nx = 12\nx /= 4\nPRINT x')
    assert "3" in out


def test_compound_on_array(run_gb):
    out = run_gb('''
DIM xs[3] AS INTEGER
xs[0] = 100
xs[0] += 25
PRINT xs[0]
''')
    assert "125" in out


# --- String-Interpolation --------------------------------------------

def test_fstring_simple(run_gb):
    out = run_gb('DIM n AS STRING\nn = "Anna"\nPRINT f"Hallo, {n}!"')
    assert "Hallo, Anna!" in out


def test_fstring_int_via_str(run_gb):
    """STR$ wird auto-um den Ausdruck gewickelt - INT funktioniert."""
    out = run_gb('DIM x AS INTEGER\nx = 42\nPRINT f"x = {x}"')
    assert "x = 42" in out


def test_fstring_multiple_exprs(run_gb):
    out = run_gb('DIM a AS INTEGER\nDIM b AS INTEGER\na = 5\nb = 7\nPRINT f"{a} + {b} = {a + b}"')
    assert "5 + 7 = 12" in out


def test_fstring_double_brace_escapes(run_gb):
    out = run_gb('DIM x AS INTEGER\nx = 5\nPRINT f"{{not expr}} {x}"')
    assert "{not expr} 5" in out


def test_fstring_empty(run_gb):
    out = run_gb('PRINT "x:" + f""')
    assert out.strip() == "x:"


def test_fstring_only_expr(run_gb):
    out = run_gb('DIM x AS INTEGER\nx = 5\nPRINT f"{x}"')
    assert out.strip() == "5"


def test_fstring_unterminated_errors(run_gb):
    from gamebasic.errors import LexerError
    with pytest.raises(LexerError):
        run_gb('PRINT f"hello')


def test_fstring_empty_expr_errors(run_gb):
    from gamebasic.errors import LexerError
    with pytest.raises(LexerError, match="Leerer"):
        run_gb('PRINT f"{}"')


# --- REPEAT/DATA/READ/RESTORE/Defaults (frueher Python-VM, jetzt Tree-Walker) ---

def _run_vm(src):
    # Stufe B: laeuft jetzt ueber gbrt (frueher Tree-Walker/Python-VM).
    # Eigenstaendiger Runner, damit die test_vm_*-Tests unveraendert bleiben.
    import os as _os
    import subprocess as _sp
    import tempfile as _tf
    from pathlib import Path as _P
    from gamebasic.errors import GBRuntimeError, ParseError, LexerError
    root = _P(__file__).resolve().parent.parent
    exe = "gbrt.exe" if _os.name == "nt" else "gbrt"
    gbrt = None
    for v in ("release", "debug"):
        p = root / "rust" / "gb_runtime" / "target" / v / exe
        if p.exists():
            gbrt = p
            break
    if gbrt is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    fd, tmp = _tf.mkstemp(suffix=".gb", prefix="_gbtest_")
    _os.close(fd)
    try:
        _P(tmp).write_text(src, encoding="utf-8")
        r = _sp.run([str(gbrt), "run", tmp], capture_output=True,
                    text=True, encoding="utf-8", timeout=60)
    finally:
        try:
            _os.unlink(tmp)
        except OSError:
            pass
    if r.returncode != 0:
        stderr = r.stderr or ""
        if "Parse-Fehler" in stderr:
            raise ParseError(stderr.strip())
        if "Lexer-Fehler" in stderr:
            raise LexerError(stderr.strip())
        raise GBRuntimeError(stderr.strip())
    return (r.stdout or "").replace("\r\n", "\n")


def test_vm_repeat_until():
    out = _run_vm('''
DIM i AS INTEGER
i = 0
REPEAT
    PRINT i
    i = i + 1
UNTIL i >= 3
''')
    assert out == "0\n1\n2\n"


def test_vm_data_read():
    out = _run_vm('''
DATA "Anna", 100
DIM name AS STRING
DIM score AS INTEGER
READ name, score
PRINT name, score
''')
    assert "Anna 100" in out


def test_vm_data_restore():
    out = _run_vm('''
DATA 1, 2, 3
DIM x AS INTEGER
READ x
READ x
RESTORE
READ x
PRINT x
''')
    assert out.strip() == "1"


def test_vm_default_param():
    out = _run_vm('''
SUB greet(name AS STRING, prefix AS STRING = "Hallo")
    PRINT prefix, name
END SUB

greet("Anna")
''')
    assert "Hallo Anna" in out


def test_vm_compound_assign():
    out = _run_vm('DIM x AS INTEGER\nx = 10\nx += 5\nPRINT x')
    assert "15" in out


def test_vm_fstring():
    out = _run_vm('DIM n AS STRING\nn = "Bob"\nPRINT f"hi {n}"')
    assert "hi Bob" in out


# (Der frühere Test, dass der Compiler param-referenzierende Defaults ablehnt,
#  ist entfernt: gbrt UNTERSTÜTZT sie jetzt (Callee-Prolog, Stufe-B-Härtung) --
#  siehe test_default_can_reference_earlier_param. Der Python-Compiler wird in
#  Phase 8 gelöscht.)


