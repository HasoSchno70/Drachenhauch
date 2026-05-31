"""Tests fuer den Code-Formatter (Indent-Normalisierung)."""
from gamebasic.editor_qt.formatter import format_source


def _lines(src: str) -> list[str]:
    return src.split("\n")


def test_simple_sub_block():
    src = "SUB foo()\nPRINT 1\nEND SUB"
    expected = "SUB foo()\n    PRINT 1\nEND SUB"
    assert format_source(src) == expected


def test_nested_if_in_sub():
    src = "SUB foo()\nPRINT 1\nIF x THEN\nPRINT 2\nEND IF\nEND SUB"
    expected = (
        "SUB foo()\n"
        "    PRINT 1\n"
        "    IF x THEN\n"
        "        PRINT 2\n"
        "    END IF\n"
        "END SUB"
    )
    assert format_source(src) == expected


def test_else_dedents_one_level():
    src = "IF x THEN\nPRINT 1\nELSE\nPRINT 2\nEND IF"
    out = format_source(src)
    expected = (
        "IF x THEN\n"
        "    PRINT 1\n"
        "ELSE\n"
        "    PRINT 2\n"
        "END IF"
    )
    assert out == expected


def test_elseif_dedents_one_level():
    src = "IF x THEN\nA\nELSEIF y THEN\nB\nEND IF"
    expected = (
        "IF x THEN\n"
        "    A\n"
        "ELSEIF y THEN\n"
        "    B\n"
        "END IF"
    )
    assert format_source(src) == expected


def test_select_case():
    src = (
        "SELECT CASE x\n"
        "CASE 1\n"
        "PRINT 1\n"
        "CASE ELSE\n"
        "PRINT 0\n"
        "END SELECT"
    )
    expected = (
        "SELECT CASE x\n"
        "    CASE 1\n"
        "        PRINT 1\n"
        "    CASE ELSE\n"
        "        PRINT 0\n"
        "END SELECT"
    )
    assert format_source(src) == expected


def test_for_next():
    src = "FOR i = 1 TO 10\nPRINT i\nNEXT"
    expected = "FOR i = 1 TO 10\n    PRINT i\nNEXT"
    assert format_source(src) == expected


def test_single_line_if_is_no_block():
    """`IF x THEN PRINT 1` darf den Folge-Indent NICHT erhoehen."""
    src = "IF x THEN PRINT 1\nPRINT 2"
    expected = "IF x THEN PRINT 1\nPRINT 2"
    assert format_source(src) == expected


def test_class_with_methods():
    src = (
        "CLASS Player\n"
        "DIM hp AS INTEGER\n"
        "SUB Init()\n"
        "Self.hp = 0\n"
        "END SUB\n"
        "END CLASS"
    )
    out = format_source(src).split("\n")
    assert out[0] == "CLASS Player"
    assert out[1] == "    DIM hp AS INTEGER"
    assert out[2] == "    SUB Init()"
    assert out[3] == "        Self.hp = 0"
    assert out[4] == "    END SUB"
    assert out[5] == "END CLASS"


def test_strips_existing_indent():
    """Vorhandene falsche Einrueckung wird normalisiert."""
    src = "        SUB foo()\n  PRINT 1\n              END SUB"
    expected = "SUB foo()\n    PRINT 1\nEND SUB"
    assert format_source(src) == expected


def test_empty_lines_remain_empty():
    src = "SUB foo()\n\nPRINT 1\n\nEND SUB"
    out = format_source(src)
    parts = out.split("\n")
    assert parts[0] == "SUB foo()"
    assert parts[1] == ""
    assert parts[2] == "    PRINT 1"
    assert parts[3] == ""
    assert parts[4] == "END SUB"


def test_idempotent():
    """Zweimal formatieren liefert dasselbe Resultat wie einmal."""
    src = "SUB foo()\nIF x THEN\nPRINT 1\nELSE\nPRINT 2\nEND IF\nEND SUB"
    once = format_source(src)
    twice = format_source(once)
    assert once == twice
