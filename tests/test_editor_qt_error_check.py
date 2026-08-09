"""Tests fuer die Editor-Diagnostik-Pipeline.

Wir umgehen den Async-Layer und testen `_check_source` direkt -- der ist
synchron und liefert jetzt eine **Liste** aller ParseProblems (leer = sauber).
Der Async-Wrapper drumherum ist Plumbing und wird durch die UI-Smoke-Tests
abgedeckt.
"""
import pytest

from gamebasic.editor_qt.error_check import _check_source, _find_dhrt

# Compile-Phase-Diagnostik (Typ-/Semantikfehler wie unbekannte Typen oder
# doppelte Funktionsnamen) kann NUR dhrt liefern -- der Syntax-only-Fallback
# in _check_source() findet ausschliesslich Lexer-/Parser-Fehler. Ohne
# gebauten dhrt liefert _check_source() fuer diese Faelle eine leere Liste
# statt des erwarteten Compile-Problems.
_needs_dhrt = pytest.mark.skipif(_find_dhrt() is None, reason="native Runtime 'dhrt' nicht gebaut")


def _first(src, base=None):
    """Erstes Problem (oder None) -- Bequemlichkeit fuer die Einzel-Fehler-Tests."""
    probs = _check_source(src, base)
    return probs[0] if probs else None


def test_clean_source_returns_empty():
    src = 'PRINT "hello"\n'
    assert _check_source(src, None) == []


def test_lex_error_phase():
    # Unterminierter String -- dhrt meldet Phase "lex" (Python lumpte lex+parse).
    p = _first('PRINT "no close')
    assert p is not None
    assert p.phase in ("lex", "parse")
    assert p.severity == "error"


def test_parse_error_phase_parse():
    # Parser kommt durch Lexer aber stoplert beim Strukturfehler
    p = _first("IF 1 = 1 THEN\n    PRINT 1\n")
    assert p is not None
    assert p.phase == "parse"


@_needs_dhrt
def test_compile_error_unknown_type():
    # DIM mit unbekanntem Typ -> Compile-Fehler (dhrt nennt den Typnamen).
    p = _first("DIM p AS NoSuchClass\n")
    assert p is not None
    assert p.phase == "compile"
    assert "nosuchclass" in p.message.lower()


@_needs_dhrt
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


@_needs_dhrt
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


# --------------------------------------------------- Checker-Crash sichtbar
# Review-Fund: ein dhrt-Absturz/kaputtes JSON lieferte frueher eine leere
# Liste -- der Editor zeigte "keine Fehler", obwohl die Diagnose selbst
# fehlgeschlagen war. Jetzt kommt ein sichtbares Warning-ParseProblem.

def test_dhrt_crash_yields_diagnostic_warning(tmp_path, monkeypatch):
    import subprocess
    import gamebasic.editor_qt.error_check as ec

    class _FakeProc:
        def communicate(self, timeout=None):
            return "not json {{{", ""

    monkeypatch.setattr(ec, "_find_dhrt", lambda: "dhrt")
    monkeypatch.setattr(subprocess, "Popen", lambda *a, **k: _FakeProc())
    probs = ec._check_source("PRINT 1\n", tmp_path)
    assert len(probs) == 1
    assert probs[0].severity == "warning"
    assert probs[0].phase == "diagnostic"
    assert "fehlgeschlagen" in probs[0].message.lower()


def test_dhrt_timeout_yields_diagnostic_warning(tmp_path, monkeypatch):
    import subprocess
    import gamebasic.editor_qt.error_check as ec

    class _FakeProc:
        def communicate(self, timeout=None):
            raise subprocess.TimeoutExpired(cmd="dhrt", timeout=15)

    monkeypatch.setattr(ec, "_find_dhrt", lambda: "dhrt")
    monkeypatch.setattr(subprocess, "Popen", lambda *a, **k: _FakeProc())
    probs = ec._check_source("PRINT 1\n", tmp_path)
    assert len(probs) == 1
    assert probs[0].severity == "warning"
    assert "timeout" in probs[0].message.lower()


# --------------------------------------------------- Vorherigen Check abbrechen

def test_check_terminates_previous_active_process(tmp_path, monkeypatch):
    """Ein zweiter check()-Aufruf muss den noch laufenden dhrt-Subprozess
    des ersten Aufrufs abbrechen (statt ihn im Hintergrund weiterlaufen zu
    lassen und nur das Ergebnis zu verwerfen)."""
    import threading
    from gamebasic.editor_qt.error_check import LiveErrorChecker

    checker = LiveErrorChecker()
    terminated = threading.Event()

    class _StubProc:
        def terminate(self):
            terminated.set()

    checker._set_active_proc(_StubProc())
    # _run() nicht wirklich starten (kein dhrt/Qt-Loop noetig) -- nur die
    # Cancel-Logik von check() selbst pruefen.
    monkeypatch.setattr(
        "gamebasic.editor_qt.error_check.threading.Thread.start", lambda self: None)
    checker.check("PRINT 1\n", tmp_path)
    assert terminated.is_set()


def test_cancel_terminates_active_process_and_bumps_generation():
    """`cancel()` (Review-Fund: Tab-Close liess den Checker unbeaufsichtigt
    weiterlaufen) muss wie `check()` einen laufenden Subprozess abbrechen --
    aber OHNE einen neuen Check zu starten."""
    from gamebasic.editor_qt.error_check import LiveErrorChecker

    checker = LiveErrorChecker()
    terminated = []

    class _StubProc:
        def terminate(self):
            terminated.append(1)

    checker._set_active_proc(_StubProc())
    gen_before = checker._gen
    checker.cancel()
    assert terminated == [1]
    assert checker._gen == gen_before + 1


def test_cancel_without_active_process_is_noop():
    from gamebasic.editor_qt.error_check import LiveErrorChecker

    checker = LiveErrorChecker()
    checker.cancel()   # kein Absturz ohne laufenden Prozess
