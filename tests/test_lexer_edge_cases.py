"""Lexer-Randfaelle aus dem Clean-Code-Review des Python-Frontends.

Der Python-Lexer fuehrt nichts mehr aus (das macht dhrt), sondern bedient
NUR die Editor-Schicht: Highlighting, LSP, Completion, Outline, Folding,
Formatter. Fehler hier zeigen sich also als kaputte Editor-Features -- im
schlimmsten Fall als Ausnahme mitten in Qts Paint-Pfad.
"""
import pytest

from gamebasic.lexer import Lexer
from gamebasic.errors import LexerError
from gamebasic.tokens import TokenType


def _lex(src):
    return [t for t in Lexer(src).tokenize()
            if t.type not in (TokenType.NEWLINE, TokenType.EOF)]


# --- ASCII-Ziffern statt str.isdigit() -------------------------------

@pytest.mark.parametrize("src", [
    "PRINT 5²",        # Hochzahl 2 -- warf frueher ValueError
    "PRINT 5³",        # Hochzahl 3
    "PRINT ٣",         # arabisch-indische 3 -- lexte frueher STILL als 3
    "PRINT &H٣",       # dieselbe Ziffer im Hex-Literal
])
def test_non_ascii_digits_raise_clean_lexer_error(src):
    """`str.isdigit()` ist auch fuer '²'/'٣' True, `int()` aber nicht
    fuer alle -- das erzeugte einmal eine ungefangene ValueError (der
    Highlighter faengt nur LexerError) und einmal eine stille Abweichung
    zu dhrt, das solche Zeichen ablehnt. Beides muss jetzt ein sauberer,
    fangbarer LexerError sein."""
    with pytest.raises(LexerError):
        Lexer(src).tokenize()


@pytest.mark.parametrize("src,expected", [
    ("PRINT 5", 5),
    ("PRINT 1.5", 1.5),
    ("PRINT &HFF", 255),
    ("PRINT 0xFF", 255),
    ("PRINT &B1010", 10),
    ("PRINT 0b1010", 10),
])
def test_ascii_number_literals_still_work(src, expected):
    """Regression: die ASCII-Einschraenkung darf normale Literale nicht brechen."""
    nums = [t.value for t in _lex(src) if t.type == TokenType.NUMBER]
    assert nums == [expected]


# --- f-String-Randfaelle ---------------------------------------------

@pytest.mark.parametrize("src", [
    'PRINT f"{ }"',      # nur Whitespace -- ergab frueher argumentloses str$()
    'PRINT f"{}"',       # ganz leer
    'PRINT f"{x:}"',     # Doppelpunkt ohne Spec -- ergab frueher `str$(x :)`
])
def test_empty_fstring_placeholder_or_spec_raises(src):
    """Statt eines Muell-Tokenstroms, der erst weit spaeter als
    verwirrender Parse-Fehler auffaellt, sofort klar melden."""
    with pytest.raises(LexerError):
        Lexer(src).tokenize()


def test_fstring_with_spec_still_becomes_format():
    toks = _lex('PRINT f"{x:.1f}"')
    names = [t.value for t in toks if t.type == TokenType.IDENT]
    assert "format$" in names
    assert any(t.type == TokenType.STRING and t.value == "%.1f" for t in toks)


def test_fstring_without_spec_still_becomes_str():
    toks = _lex('PRINT f"{x}"')
    names = [t.value for t in toks if t.type == TokenType.IDENT]
    assert "str$" in names


# --- Token-Spans ------------------------------------------------------

@pytest.mark.parametrize("src", [
    'PRINT "a"\nPRINT f"a{x}b"',      # f-String NICHT auf Zeile 1
    'PRINT f"{x}"',
    'PRINT f"{a + b}"',
    'DIM x AS INTEGER\nx = 1\nPRINT f"v={x}"',
])
def test_no_token_has_an_inverted_span(src):
    """Tokens aus f-String-Platzhaltern bekamen line/col vom f-String, aber
    end_line/end_col blieben die des Sub-Lexers (Zeile 1) -- das Ende lag
    also VOR dem Start. Der Highlighter clamped das weg, jeder andere
    Konsument (LSP-Semantic-Tokens, Hover, Selection-Ranges) haette einen
    rueckwaerts laufenden Bereich bekommen."""
    for t in Lexer(src).tokenize():
        assert (t.end_line, t.end_col) >= (t.line, t.col), (
            f"{t.type.name} start=({t.line},{t.col}) end=({t.end_line},{t.end_col})")
