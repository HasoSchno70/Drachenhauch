"""Tests fuer den Highlighter -- speziell die f-String-Erkennung.

Das eigentliche Token-Coloring laeuft via Qt und ist nicht trivial unit-
testbar; die `_find_fstring_ranges`-Hilfsmethode hingegen ist eine reine
Funktion und deckt den fehleranfaelligen Teil ab (Lookahead, Escape,
Wort-Anfang, mehrere f-Strings in derselben Zeile)."""
from gamebasic.editor_qt.highlighter import GBHighlighter

F = GBHighlighter._find_fstring_ranges


def test_no_fstring_returns_empty():
    assert F('PRINT "normal string"') == []


def test_no_fstring_in_plain_text():
    assert F("foo() + foobar") == []


def test_simple_fstring():
    # 'PRINT f"hi {x}!"' -> f at col 6, f-string spans 10 chars
    assert F('PRINT f"hi {x}!"') == [(6, 10)]


def test_two_fstrings_in_one_line():
    assert F('PRINT f"a" + f"b"') == [(6, 4), (13, 4)]


def test_uppercase_F_recognized():
    assert F('PRINT F"x"') == [(6, 4)]


def test_lowercase_f_at_word_start():
    # 'foo + f"x"' -> f at col 6
    assert F('foo + f"x"') == [(6, 4)]


def test_f_inside_normal_string_ignored():
    # Anfuehrungszeichen vor f bedeutet: f ist Inhalt, kein f-String
    assert F('PRINT "f"') == []


def test_dim_f_is_not_fstring():
    # 'DIM f AS STRING' -- f ist eine Variable, kein f-String-Praefix
    assert F("DIM f AS STRING") == []


def test_f_after_ident_char_is_not_fstring():
    # 'foofbar' -- f ist mitten in einem Wort
    assert F('foofbar"x"') == []


def test_doubled_quotes_inside_fstring():
    # f"hi ""there""!" -- doppelte Quotes sind Escape, nicht Stringende
    src = 'PRINT f"hi ""there""!"'
    assert F(src) == [(6, len(src) - 6)]


def test_unterminated_fstring_extends_to_eol():
    # 'PRINT f"unterminiert' -- 6+14 = bis zum Zeilenende
    src = 'PRINT f"unterminiert'
    assert F(src) == [(6, 14)]
