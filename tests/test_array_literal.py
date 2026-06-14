"""A1: Array-Literale `[a, b, c]`.

Frueher kollidierte `[...]` ausschliesslich mit der List-Comprehension
(`a = [1,2,3]` -> „Erwartet FOR in List-Comprehension"). Jetzt disambiguiert
der Parser: FOR nach dem ersten Ausdruck = Comprehension, sonst Array-Literal.
"""
import pytest
from gamebasic.errors import GameBasicError
from gamebasic.lexer import Lexer
from gamebasic.parser import Parser
from gamebasic.ast_nodes import ArrayLit, ListComp


def test_int_array_literal(run_gb):
    assert run_gb("DIM a AS ARRAY OF INTEGER\n"
                  "a = [10, 20, 30]\n"
                  "PRINT a[0], a[2], LEN(a)\n") == "10 30 3\n"


def test_float_promotion(run_gb):
    # gemischte Zahlen -> FLOAT-Array (Ints hochgezogen)
    assert run_gb("DIM f AS ARRAY OF FLOAT\n"
                  "f = [1.5, 2, 3.5]\n"
                  "PRINT f[1]\n") == "2.0\n"


def test_string_array_literal(run_gb):
    assert run_gb('DIM s AS ARRAY OF STRING\n'
                  's = ["hallo", "welt"]\n'
                  'PRINT s[0]; "-"; s[1]\n') == "hallo-welt\n"


def test_array_literal_as_iterable(run_gb):
    assert run_gb("DIM sum AS INTEGER\n"
                  "sum = 0\n"
                  "FOR EACH n IN [1, 2, 3, 4]\n"
                  "    sum = sum + n\n"
                  "NEXT\n"
                  "PRINT sum\n") == "10\n"


def test_single_element_array(run_gb):
    assert run_gb("DIM a AS ARRAY OF INTEGER\n"
                  "a = [5]\n"
                  "PRINT LEN(a), a[0]\n") == "1 5\n"


def test_trailing_comma(run_gb):
    assert run_gb("PRINT [1, 2, 3,][2]\n") == "3\n"


def test_sort_on_literal_array(run_gb):
    assert run_gb("DIM a AS ARRAY OF INTEGER\n"
                  "a = [3, 1, 2]\n"
                  "SORT(a)\n"
                  "PRINT a[0], a[1], a[2]\n") == "1 2 3\n"


def test_comprehension_still_works(run_gb):
    # `[expr FOR ...]` bleibt eine Comprehension -- inkl. ueber ein Array-Literal.
    assert run_gb("DIM sq AS TUPLE\n"
                  "sq = [x * x FOR x IN [1, 2, 3]]\n"
                  "PRINT sq\n") == "(1, 4, 9)\n"


def test_empty_array_literal_errors(run_gb):
    with pytest.raises(GameBasicError, match=r"Leeres Array-Literal"):
        run_gb("DIM a AS ARRAY OF INTEGER\na = []\n")


# --- Front-End-Paritaet (Python-Parser unterscheidet Literal vs Comp) ----

def test_python_parser_array_literal_node():
    prog = Parser(Lexer("a = [1, 2, 3]\n").tokenize()).parse()
    assert isinstance(prog.statements[0].value, ArrayLit)
    assert len(prog.statements[0].value.elements) == 3


def test_python_parser_comprehension_node():
    prog = Parser(Lexer("a = [x FOR x IN nums]\n").tokenize()).parse()
    assert isinstance(prog.statements[0].value, ListComp)
