"""Tests fuer Multi-DIM (`DIM a, b, c AS INTEGER`).

Pruefen Tree-Walker und Python-VM auf identisches Verhalten - genau wie
bei test_select_case.py.
"""
import pytest


CASES = [
    # Basic Multi-DIM mit drei Skalaren
    (
        "DIM a, b, c AS INTEGER\n"
        "a = 1\n"
        "b = 2\n"
        "c = 3\n"
        'PRINT a + b + c\n',
        "6\n",
    ),
    # Strings
    (
        "DIM x, y AS STRING\n"
        'x = "Hallo"\n'
        'y = "Welt"\n'
        'PRINT x + " " + y\n',
        "Hallo Welt\n",
    ),
    # Float
    (
        "DIM a, b AS FLOAT\n"
        "a = 1.5\n"
        "b = 2.5\n"
        "PRINT a + b\n",
        "4.0\n",
    ),
    # Boolean
    (
        "DIM a, b AS BOOLEAN\n"
        "a = TRUE\n"
        "b = FALSE\n"
        "PRINT a\n"
        "PRINT b\n",
        "TRUE\nFALSE\n",
    ),
    # Multi-DIM mit Array gemischt
    (
        "DIM x[3], y AS INTEGER\n"
        "x[0] = 10\n"
        "x[1] = 20\n"
        "x[2] = 30\n"
        "y = 99\n"
        "PRINT x[0] + x[1] + x[2]\n"
        "PRINT y\n",
        "60\n99\n",
    ),
    # Multi-DIM nur Arrays mit unterschiedlichen Groessen
    (
        "DIM small[2], big[5] AS INTEGER\n"
        "small[0] = 1\n"
        "big[4] = 99\n"
        "PRINT small[0]\n"
        "PRINT big[4]\n",
        "1\n99\n",
    ),
    # 2D-Array innerhalb Multi-DIM
    (
        "DIM grid[2, 2], scalar AS INTEGER\n"
        "grid[0, 0] = 7\n"
        "grid[1, 1] = 8\n"
        "scalar = 5\n"
        "PRINT grid[0, 0] + grid[1, 1] + scalar\n",
        "20\n",
    ),
]


@pytest.mark.parametrize("source,expected", CASES)
def test_multi_dim_treewalker(run_gb, source, expected):
    assert run_gb(source) == expected


@pytest.mark.parametrize("source,expected", CASES)
def test_multi_dim_vm(run_vm, source, expected):
    assert run_vm(source) == expected


def test_multi_dim_single_var_still_works(run_gb):
    """Single-DIM darf NICHT durch die Multi-DIM-Aenderung kaputt gehen."""
    src = (
        "DIM x AS INTEGER\n"
        "x = 42\n"
        "PRINT x\n"
    )
    assert run_gb(src) == "42\n"


def test_multi_dim_each_var_initialized_to_default(run_gb):
    """Alle Vars im Multi-DIM kriegen den Typ-Default (0 bei INTEGER)."""
    src = (
        "DIM a, b, c AS INTEGER\n"
        "PRINT a\n"
        "PRINT b\n"
        "PRINT c\n"
    )
    assert run_gb(src) == "0\n0\n0\n"


def test_multi_dim_parser_error_still_requires_as():
    """Ohne AS am Ende soll's weiterhin einen Parser-Fehler geben."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.errors import ParseError
    src = "DIM a, b, c\n"
    with pytest.raises(ParseError):
        Parser(Lexer(src).tokenize()).parse()


def test_multi_dim_ast_shape():
    """Verifiziere AST-Struktur: Single -> Dim, Multi -> MultiDim."""
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.ast_nodes import Dim, MultiDim
    single = Parser(Lexer("DIM x AS INTEGER\n").tokenize()).parse()
    assert isinstance(single.statements[0], Dim)
    multi = Parser(Lexer("DIM a, b, c AS INTEGER\n").tokenize()).parse()
    md = multi.statements[0]
    assert isinstance(md, MultiDim)
    assert len(md.dims) == 3
    names = [d.name for d in md.dims]
    assert names == ["a", "b", "c"]
    types = [d.type_name for d in md.dims]
    assert types == ["integer", "integer", "integer"]
