"""Tests fuer die Editor-Diagnostik-Pipeline.

Wir umgehen den Async-Layer und testen `_check_source` direkt -- der ist
synchron und liefert jetzt eine **Liste** aller ParseProblems (leer = sauber).
Der Async-Wrapper drumherum ist Plumbing und wird durch die UI-Smoke-Tests
abgedeckt.
"""
from gamebasic.editor_qt.error_check import _check_source


def _first(src, base=None):
    """Erstes Problem (oder None) -- Bequemlichkeit fuer die Einzel-Fehler-Tests."""
    probs = _check_source(src, base)
    return probs[0] if probs else None


def test_clean_source_returns_empty():
    src = 'PRINT "hello"\n'
    assert _check_source(src, None) == []


def test_lex_error_phase():
    # Unterminierter String -- gbrt meldet Phase "lex" (Python lumpte lex+parse).
    p = _first('PRINT "no close')
    assert p is not None
    assert p.phase in ("lex", "parse")
    assert p.severity == "error"


def test_parse_error_phase_parse():
    # Parser kommt durch Lexer aber stoplert beim Strukturfehler
    p = _first("IF 1 = 1 THEN\n    PRINT 1\n")
    assert p is not None
    assert p.phase == "parse"


def test_compile_error_unknown_type():
    # DIM mit unbekanntem Typ -> Compile-Fehler (gbrt nennt den Typnamen).
    p = _first("DIM p AS NoSuchClass\n")
    assert p is not None
    assert p.phase == "compile"
    assert "nosuchclass" in p.message.lower()


def test_compile_error_duplicate_function():
    src = (
        "SUB foo()\n"
        "    PRINT 1\n"
        "END SUB\n"
        "SUB foo()\n"
        "    PRINT 2\n"
        "END SUB\n"
    )
    p = _first(src)
    assert p is not None
    assert p.phase == "compile"


def test_lex_error_beats_compile_error():
    """Wenn die Pipeline bei einem fruehen Fehler stoppt, kommt nicht der
    spaetere Compile-Fehler durch."""
    src = 'PRINT "broken\nDIM p AS NoSuch\n'
    p = _first(src)
    assert p is not None
    assert p.phase in ("lex", "parse")    # frueher Fehler (Lexer), nicht compile


def test_clean_compile_with_imports():
    """vec2-Modul muss laden + sauber kompilieren."""
    src = (
        'IMPORT "vec2"\n'
        "DIM v AS VEC2\n"
        "v = VEC2_NEW(1.0, 2.0)\n"
        "PRINT v\n"
    )
    assert _check_source(src, None) == []


def test_severity_default_is_error():
    p = _first("DIM x AS NoSuch\n")
    assert p is not None
    assert p.severity == "error"


def test_hardware_import_is_warning():
    """E1: `IMPORT "wifi"` ohne Hardware-Build erscheint als Warnung in der
    Diagnose-Liste -- sauberes Programm, daher genau ein Eintrag (Warning).
    Ein Hardware-Build meldet nichts; beide Faelle sind gueltig."""
    probs = _check_source('IMPORT "wifi"\nPRINT 1\n', None)
    warns = [p for p in probs if p.severity == "warning"]
    errs = [p for p in probs if p.severity != "warning"]
    assert not errs                       # sauberes Programm: keine Errors
    if warns:                             # Default-Build: Hardware-Modul-Warnung
        assert warns[0].line == 1
        assert "wifi" in warns[0].message.lower()
