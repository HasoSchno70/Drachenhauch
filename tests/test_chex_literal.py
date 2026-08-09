"""C-Stil Hex-/Binaer-Literale: 0xFF / 0b1010.

Zusaetzlich zur klassischen BASIC-Form &H/&B akzeptieren beide Lexer
(Python-Front-End + dhrt) jetzt auch die C-Schreibweise. `0`/`0.5` duerfen
dabei NICHT als Praefix fehlgedeutet werden.
"""
from drachenhauch.lexer import Lexer
from drachenhauch.tokens import TokenType


# --- Laufzeit-Semantik (dhrt) -------------------------------------------

def test_hex_literal_value(run_gb):
    assert run_gb("PRINT 0xFF\n") == "255\n"


def test_binary_literal_value(run_gb):
    assert run_gb("PRINT 0b1010\n") == "10\n"


def test_uppercase_prefix(run_gb):
    assert run_gb("PRINT 0XCAFE\n") == "51966\n"


def test_equivalent_to_basic_form(run_gb):
    assert run_gb("PRINT 0xFF8000 = &HFF8000\n") == "TRUE\n"


def test_hex_in_const(run_gb):
    assert run_gb("CONST C AS INTEGER = 0x141E3C\nPRINT C\n") == "1318460\n"


def test_zero_and_float_unaffected(run_gb):
    # 0 und 0.5 duerfen NICHT als Hex/Binaer fehlgedeutet werden
    assert run_gb("PRINT 0\nPRINT 0.5\n") == "0\n0.5\n"


# --- Front-End-Paritaet (Python-Lexer kennt die Literale ebenfalls) ------

def _first_number(src: str):
    toks = [t for t in Lexer(src).tokenize() if t.type == TokenType.NUMBER]
    assert toks, "kein NUMBER-Token"
    return toks[0].value


def test_python_lexer_hex():
    assert _first_number("x = 0xFF\n") == 255


def test_python_lexer_binary():
    assert _first_number("x = 0b1010\n") == 10


def test_python_lexer_uppercase():
    assert _first_number("x = 0XCAFE\n") == 51966


def test_python_lexer_zero_fallback():
    # "0" alleine bleibt Dezimal-0 (kein Hex-/Binaer-Fehlgriff)
    assert _first_number("x = 0\n") == 0
