"""Tests fuer die Editor-Diagnostik-Pipeline.

Wir umgehen den Async-Layer und testen `_check_source` direkt -- der ist
synchron und liefert (Optional[ParseProblem]). Der Async-Wrapper drumherum
ist Plumbing und wird durch die UI-Smoke-Tests abgedeckt.
"""
from gamebasic.editor_qt.error_check import _check_source


def test_clean_source_returns_none():
    src = 'PRINT "hello"\n'
    assert _check_source(src, None) is None


def test_lex_error_phase_parse():
    # Unterminierter String
    p = _check_source('PRINT "no close', None)
    assert p is not None
    assert p.phase == "parse"
    assert p.severity == "error"


def test_parse_error_phase_parse():
    # Parser kommt durch Lexer aber stoplert beim Strukturfehler
    p = _check_source("IF 1 = 1 THEN\n    PRINT 1\n", None)
    assert p is not None
    assert p.phase == "parse"


def test_compile_error_unknown_type():
    # DIM mit unbekanntem Typ -> CompileError
    src = "DIM p AS NoSuchClass\n"
    p = _check_source(src, None)
    assert p is not None
    assert p.phase == "compile"
    assert "Unbekannt" in p.message or "notatype" in p.message.lower()


def test_compile_error_duplicate_function():
    src = (
        "SUB foo()\n"
        "    PRINT 1\n"
        "END SUB\n"
        "SUB foo()\n"
        "    PRINT 2\n"
        "END SUB\n"
    )
    p = _check_source(src, None)
    assert p is not None
    assert p.phase == "compile"


def test_lex_error_beats_compile_error():
    """Wenn die Pipeline bei einem fruehen Fehler stoppt, kommt nicht der
    spaetere Compile-Fehler durch."""
    # Unterminierter String UND unbekannter Typ -- wir sehen nur den ersten.
    src = 'PRINT "broken\nDIM p AS NoSuch\n'
    p = _check_source(src, None)
    assert p is not None
    assert p.phase == "parse"


def test_clean_compile_with_imports():
    """vec2-Modul muss laden + sauber kompilieren."""
    src = (
        'IMPORT "vec2"\n'
        "DIM v AS VEC2\n"
        "v = VEC2_NEW(1.0, 2.0)\n"
        "PRINT v\n"
    )
    assert _check_source(src, None) is None


def test_severity_default_is_error():
    p = _check_source("DIM x AS NoSuch\n", None)
    assert p is not None
    assert p.severity == "error"
