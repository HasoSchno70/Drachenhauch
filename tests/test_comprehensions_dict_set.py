"""Tests fuer Dict- und Set-Comprehensions.

Dict-Comp: `{key: val FOR var IN iterable [WHERE filter]}` -> MAP
Set-Comp:  `{expr FOR var IN iterable [WHERE filter]}` -> TUPLE (deduped)

Wir testen alle drei Pfade -- Tree-Walker, Python-VM, Cython-VM -- via
run_gb / run_vm und einem direkten Cython-Test.
"""
import io
import contextlib

import pytest


# --- Dict-Comprehension ---------------------------------------------

def test_dict_comp_basic_tw(run_gb):
    out = run_gb('''
DIM m AS MAP OF INTEGER
m = {STR$(x) + "sq": x * x FOR x IN (1, 2, 3)}
PRINT MAPGET(m, "1sq")
PRINT MAPGET(m, "2sq")
PRINT MAPGET(m, "3sq")
PRINT MAPSIZE(m)
''')
    lines = [l.strip() for l in out.split("\n") if l.strip()]
    assert lines == ["1", "4", "9", "3"]


def test_dict_comp_with_filter_tw(run_gb):
    out = run_gb('''
DIM m AS MAP OF INTEGER
m = {STR$(x): x FOR x IN (1, 2, 3, 4, 5) WHERE x MOD 2 = 0}
PRINT MAPSIZE(m)
PRINT MAPHAS(m, "2")
PRINT MAPHAS(m, "1")
''')
    lines = [l.strip() for l in out.split("\n") if l.strip()]
    assert lines == ["2", "TRUE", "FALSE"]


def test_dict_comp_string_keys_required(run_gb):
    """Key muss STRING sein -- INTEGER-Keys werfen TypeMismatch."""
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError, match="erwartet STRING"):
        run_gb('''
DIM m AS MAP OF INTEGER
m = {x: x FOR x IN (1, 2, 3)}
''')


def test_dict_comp_vm_matches_tw(run_gb, run_vm):
    src = '''
DIM m AS MAP OF INTEGER
m = {STR$(i): i * 10 FOR i IN (1, 2, 3, 4)}
PRINT MAPGET(m, "1")
PRINT MAPGET(m, "4")
'''
    assert run_gb(src) == run_vm(src)


def test_dict_comp_from_array(run_gb):
    out = run_gb('''
DIM nums[3] AS INTEGER
nums[0] = 5
nums[1] = 10
nums[2] = 15
DIM m AS MAP OF INTEGER
m = {STR$(n): n + 1 FOR n IN nums}
PRINT MAPGET(m, "5")
PRINT MAPGET(m, "10")
PRINT MAPGET(m, "15")
''')
    lines = [l.strip() for l in out.split("\n") if l.strip()]
    assert lines == ["6", "11", "16"]


# --- Set-Comprehension ----------------------------------------------

def test_set_comp_dedupes_tw(run_gb):
    out = run_gb('''
DIM s AS TUPLE
s = {x MOD 3 FOR x IN (0, 1, 2, 3, 4, 5, 6, 7, 8)}
PRINT s
''')
    assert "(0, 1, 2)" in out


def test_set_comp_preserves_first_occurrence_order(run_gb):
    out = run_gb('''
DIM s AS TUPLE
s = {c FOR c IN "banana"}
PRINT s
''')
    # "banana" -> chars b, a, n -> dedup in der ersten Reihenfolge
    # Drachenhauch-PRINT druckt Strings ohne Quotes
    assert "(b, a, n)" in out


def test_set_comp_with_filter(run_gb):
    out = run_gb('''
DIM s AS TUPLE
s = {x FOR x IN (1, 2, 3, 1, 2, 3, 4, 5) WHERE x > 2}
PRINT s
''')
    assert "(3, 4, 5)" in out


def test_set_comp_vm_matches_tw(run_gb, run_vm):
    src = '''
DIM s AS TUPLE
s = {x * 2 FOR x IN (1, 2, 1, 3, 2, 4)}
PRINT s
'''
    assert run_gb(src) == run_vm(src)


def test_set_comp_empty(run_gb):
    out = run_gb('''
DIM s AS TUPLE
s = {x FOR x IN (1, 2, 3) WHERE x > 100}
PRINT s
''')
    assert "()" in out


# --- Cross-VM-Identitaet --------------------------------------------

# (Die früheren Cython-VM-Tests (vm_native) sind entfernt: die Cython-VM wurde
#  längst durch dhrt abgelöst; das Dict/Set-Comp-Verhalten deckt der run_gb-
#  Golden-Test oben + test_dhrt_parity ab.)


# --- Parser-Edge-Cases ----------------------------------------------

def test_parser_empty_brace_block_fails(run_gb):
    """`{}` allein ist kein Comprehension -- Parser-Fehler."""
    from drachenhauch.errors import ParseError
    with pytest.raises(ParseError):
        run_gb('PRINT {}\n')


def test_parser_dict_or_set_disambiguation():
    """`{x FOR ...}` -> SetComp; `{x: y FOR ...}` -> DictComp."""
    from drachenhauch.lexer import Lexer
    from drachenhauch.parser import Parser
    from drachenhauch.ast_nodes import SetComp, DictComp

    set_src = "DIM s AS TUPLE\ns = {x FOR x IN (1, 2, 3)}\n"
    set_ast = Parser(Lexer(set_src).tokenize()).parse()
    set_assign = set_ast.statements[1]
    assert isinstance(set_assign.value, SetComp)

    dict_src = 'DIM m AS MAP OF INTEGER\nm = {STR$(x): x FOR x IN (1, 2, 3)}\n'
    dict_ast = Parser(Lexer(dict_src).tokenize()).parse()
    dict_assign = dict_ast.statements[1]
    assert isinstance(dict_assign.value, DictComp)
