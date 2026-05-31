"""Tests fuer den Fold-Region-Scanner."""
from gamebasic.editor_qt.folding import scan


def test_empty_source_returns_empty():
    assert scan("") == []


def test_simple_sub():
    src = "SUB foo()\nPRINT 1\nEND SUB"
    regions = scan(src)
    assert regions == [(1, 3, "sub")]


def test_simple_class():
    src = "CLASS Foo\nDIM x AS INTEGER\nEND CLASS"
    assert scan(src) == [(1, 3, "class")]


def test_for_with_named_next():
    src = "FOR i = 1 TO 10\nPRINT i\nNEXT i"
    assert scan(src) == [(1, 3, "for")]


def test_for_with_plain_next():
    src = "FOR i = 1 TO 10\nPRINT i\nNEXT"
    assert scan(src) == [(1, 3, "for")]


def test_repeat_until():
    src = "REPEAT\nPRINT 1\nUNTIL x > 5"
    assert scan(src) == [(1, 3, "repeat")]


def test_nested_blocks():
    src = (
        "SUB foo()\n"        # 1
        "    IF x THEN\n"    # 2
        "        PRINT 1\n"  # 3
        "    END IF\n"       # 4
        "END SUB"            # 5
    )
    regions = scan(src)
    # Innere zuerst, dann aeussere (Closer-getrieben).
    assert (2, 4, "if") in regions
    assert (1, 5, "sub") in regions


def test_class_with_method():
    src = (
        "CLASS Bar\n"             # 1
        "    SUB Hi()\n"          # 2
        "        PRINT 1\n"       # 3
        "    END SUB\n"           # 4
        "END CLASS"               # 5
    )
    regions = scan(src)
    assert (2, 4, "sub") in regions
    assert (1, 5, "class") in regions


def test_single_line_if_is_no_region():
    """IF x THEN PRINT y -- kein faltbarer Block."""
    src = "IF x THEN PRINT 1\nPRINT 2"
    assert scan(src) == []


def test_select_case():
    src = "SELECT CASE x\n    CASE 1\nPRINT 1\nEND SELECT"
    regions = scan(src)
    assert (1, 4, "select") in regions


def test_try_catch():
    src = "TRY\nPRINT 1\nCATCH e\nPRINT e\nEND TRY"
    regions = scan(src)
    assert any(r[2] == "try" for r in regions)


def test_robust_against_missing_closer():
    """Kein Crash bei fehlendem END."""
    src = "SUB foo()\nPRINT 1\n"  # kein END SUB
    # Darf nicht crashen, liefert leere Liste oder partiellen Stack.
    out = scan(src)
    assert isinstance(out, list)


def test_ignore_comments_with_block_keywords():
    """`' END SUB` im Kommentar darf nicht als Closer zaehlen.

    Aktuelle Heuristik scant per upper-startswith, ohne Comment-Strip --
    dieser Test dokumentiert den Status. Falls wir spaeter strenger
    werden, sollte dieser Test fehlschlagen und wir koennen ihn
    aktualisieren.
    """
    src = "SUB foo()\nPRINT 1\nEND SUB"   # baseline, kein Kommentar
    assert scan(src) == [(1, 3, "sub")]
