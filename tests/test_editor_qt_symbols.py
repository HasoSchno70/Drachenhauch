"""Tests fuer den Symbol-Index (Goto-Definition / Find-References /
Hover-Doc)."""
from gamebasic.editor_qt.symbols import (
    scan_definitions, scan_references, find_definition,
    all_user_identifier_positions, extract_user_doc,
)


def _names(defs):
    return [(d.name, d.kind, d.line) for d in defs]


def test_empty_source():
    assert scan_definitions("") == []
    assert scan_references("", "x") == []
    assert find_definition("", "foo") is None


def test_sub_definition():
    defs = scan_definitions("SUB greet(name AS STRING)\nEND SUB")
    assert ("greet", "sub", 1) in _names(defs)
    assert ("name", "param", 1) in _names(defs)


def test_function_definition():
    defs = scan_definitions("FUNCTION add(a AS INTEGER, b AS INTEGER) AS INTEGER\nEND FUNCTION")
    n = _names(defs)
    assert ("add", "function", 1) in n
    assert ("a", "param", 1) in n
    assert ("b", "param", 1) in n


def test_class_struct_enum():
    src = (
        "CLASS Player\nEND CLASS\n"
        "STRUCT Point\nEND STRUCT\n"
        "ENUM State\n  A = 0\nEND ENUM"
    )
    defs = scan_definitions(src)
    n = _names(defs)
    assert ("Player", "class", 1) in n
    assert ("Point", "struct", 3) in n
    assert ("State", "enum", 5) in n


def test_dim_and_const_definitions():
    src = "DIM hp AS INTEGER\nCONST MAX AS INTEGER = 10"
    n = _names(scan_definitions(src))
    assert ("hp", "dim", 1) in n
    assert ("MAX", "const", 2) in n


def test_scan_references_word_boundary():
    src = "DIM x AS INTEGER\nx = x + 1\nDIM xy AS INTEGER"
    refs = scan_references(src, "x")
    # 3 Vorkommen von "x" als ganzes Wort -- "xy" zaehlt nicht.
    assert len(refs) == 3
    assert all(r.name.lower() == "x" for r in refs)


def test_scan_references_case_insensitive():
    src = "DIM Player AS INTEGER\nPLAYER = 1\nplayer = 2"
    refs = scan_references(src, "player")
    assert len(refs) == 3


def test_scan_references_ignores_strings_and_comments():
    src = (
        "DIM x AS INTEGER\n"
        "PRINT \"x is here\"\n"          # in String -- nicht zaehlen
        "' x in comment\n"                # in Kommentar -- nicht zaehlen
        "x = 5"
    )
    refs = scan_references(src, "x")
    # 2 echte Vorkommen: DIM x ... und x = 5.
    assert len(refs) == 2


def test_find_definition_first_match():
    src = "DIM Player AS INTEGER\nCLASS Player\nEND CLASS"
    d = find_definition(src, "Player")
    assert d is not None
    assert d.line == 1   # erste Definition gewinnt


def test_find_definition_none_when_missing():
    assert find_definition("PRINT 1", "Foo") is None


def test_all_user_identifier_positions_filters_keywords():
    """KEYWORDS (DIM, AS, INTEGER, ...) duerfen nicht als User-Idents
    auftauchen."""
    src = "DIM x AS INTEGER\nx = 5"
    by_name = all_user_identifier_positions(src)
    assert "x" in by_name
    assert "dim" not in by_name
    assert "integer" not in by_name


def test_param_with_byref():
    src = "SUB swap(BYREF a AS INTEGER, BYREF b AS INTEGER)\nEND SUB"
    defs = scan_definitions(src)
    n = _names(defs)
    assert ("a", "param", 1) in n
    assert ("b", "param", 1) in n


# ---------- extract_user_doc ----------

def test_extract_user_doc_no_match_returns_none():
    assert extract_user_doc("PRINT 1\n", "foo") is None


def test_extract_user_doc_sub_no_comment():
    src = "SUB greet(name AS STRING)\n    PRINT \"hi \" + name\nEND SUB\n"
    sig, doc = extract_user_doc(src, "greet")
    assert sig == "SUB greet(name AS STRING)"
    assert doc == ""


def test_extract_user_doc_sub_with_apostrophe_comment():
    src = (
        "' Begruessung des Spielers\n"
        "' Erwartet einen STRING und gibt nichts zurueck.\n"
        "SUB greet(name AS STRING)\n"
        "    PRINT \"hi \" + name\n"
        "END SUB\n"
    )
    sig, doc = extract_user_doc(src, "greet")
    assert sig == "SUB greet(name AS STRING)"
    assert "Begruessung" in doc
    assert "Erwartet" in doc
    assert "\n" in doc  # mehrzeilig


def test_extract_user_doc_with_rem_comment():
    src = (
        "REM Quadriert die Eingabe\n"
        "FUNCTION square(x AS INTEGER) AS INTEGER\n"
        "    RETURN x * x\n"
        "END FUNCTION\n"
    )
    sig, doc = extract_user_doc(src, "square")
    assert sig == "FUNCTION square(x AS INTEGER) AS INTEGER"
    assert doc == "Quadriert die Eingabe"


def test_extract_user_doc_blank_line_breaks_block():
    """Eine leere Zeile zwischen Doc und Decl bricht den Block ab."""
    src = (
        "' Diese Zeile ist NICHT Doku, sie gehoert zu was anderem\n"
        "\n"
        "SUB foo()\n"
        "END SUB\n"
    )
    sig, doc = extract_user_doc(src, "foo")
    assert sig == "SUB foo()"
    assert doc == ""


def test_extract_user_doc_strips_inline_comment_from_signature():
    src = (
        "SUB foo(x AS INTEGER)   ' inline comment\n"
        "END SUB\n"
    )
    sig, _doc = extract_user_doc(src, "foo")
    assert sig == "SUB foo(x AS INTEGER)"


def test_extract_user_doc_class_signature():
    src = (
        "' Spieler-Klasse mit HP und Position\n"
        "CLASS Player\n"
        "    DIM hp AS INTEGER\n"
        "END CLASS\n"
    )
    sig, doc = extract_user_doc(src, "Player")
    assert sig == "CLASS Player"
    assert "Spieler" in doc


def test_extract_user_doc_const():
    src = (
        "' Maximale Anzahl Spieler\n"
        "CONST MAX_PLAYERS AS INTEGER = 4\n"
    )
    sig, doc = extract_user_doc(src, "MAX_PLAYERS")
    assert "MAX_PLAYERS" in sig
    assert doc == "Maximale Anzahl Spieler"


def test_extract_user_doc_case_insensitive_lookup():
    src = "SUB Foo()\nEND SUB\n"
    assert extract_user_doc(src, "foo") is not None
    assert extract_user_doc(src, "FOO") is not None
    assert extract_user_doc(src, "Foo") is not None


def test_extract_user_doc_multiline_param_list():
    src = (
        "SUB process(\n"
        "    a AS INTEGER,\n"
        "    b AS INTEGER,\n"
        "    c AS INTEGER)\n"
        "END SUB\n"
    )
    sig, _doc = extract_user_doc(src, "process")
    # Multi-Line-Param-Liste wird zu Single-Line zusammengefuehrt.
    assert sig.startswith("SUB process(")
    assert sig.endswith(")")
    assert "a AS INTEGER" in sig
    assert "c AS INTEGER" in sig
