"""Tests fuer `find_active_call` (Signature-Help-Parameter-Erkennung,
gamebasic.editor_qt.signature_help). Bisher ohne Testabdeckung."""
from gamebasic.editor_qt.signature_help import find_active_call


def test_simple_call_first_arg():
    assert find_active_call("FOO(") == ("FOO", 0)


def test_second_arg_after_comma():
    assert find_active_call("FOO(1, ") == ("FOO", 1)


def test_no_open_call_returns_none():
    assert find_active_call("x = 1") is None


def test_closed_call_returns_none():
    assert find_active_call("FOO(1, 2)") is None


def test_pure_grouping_paren_without_name_returns_none():
    assert find_active_call("(1 + ") is None


def test_index_bracket_returns_none():
    assert find_active_call("arr[") is None


def test_nested_call_reports_innermost():
    assert find_active_call("FOO(BAR(1,2), ") == ("FOO", 1)
    assert find_active_call("FOO(BAR(1, ") == ("BAR", 1)


def test_string_with_comma_and_paren_ignored():
    assert find_active_call('FOO("a,b(", ') == ("FOO", 1)


def test_line_comment_apostrophe_ignored():
    text = "' see also zoom(\nFOO(1, "
    assert find_active_call(text) == ("FOO", 1)


# --- REM-Kommentare (Review-Fund) -----------------------------------------

def test_rem_comment_with_unbalanced_paren_does_not_leave_phantom_frame():
    """Vorher: eine unbalancierte Klammer in einem REM-Kommentar erzeugte
    einen Phantom-Stack-Frame, der nach einem spaeteren, unabhaengigen
    echten Aufruf an der Basis des Stacks haengen blieb -- der Cursor an
    eigentlich unbeteiligtem Top-Level-Code zeigte dann faelschlich eine
    "aktive" Funktion an."""
    text = "REM see also zoom(\nx = 1\nFOO(1, 2)\ny = 2"
    assert find_active_call(text) is None


def test_rem_comment_swallows_open_call_until_next_line():
    text = "REM FOO(1, 2\nBAR(3, "
    assert find_active_call(text) == ("BAR", 1)


def test_rem_embedded_in_longer_identifier_not_treated_as_comment():
    # "REM" als Teilstring MITTEN in einem laengeren Bezeichner (PREMIUM)
    # ist kein Kommentar -- kein Wort-Ende nach den 3 Zeichen.
    assert find_active_call("PREMIUM(") == ("PREMIUM", 0)


def test_rem_prefix_of_longer_identifier_not_treated_as_comment():
    assert find_active_call("REMIUM(") == ("REMIUM", 0)


def test_rem_trailing_comment_with_unbalanced_paren_does_not_leave_open_call():
    # Review-Fund: REM ist an JEDEM Wort-Anfang ein Kommentar bis Zeilenende
    # -- nicht nur am Zeilenanfang (wie auch symbols._strip_comment_and_
    # strings es handhabt: dort zaehlt nur "Zeichen davor ist kein
    # Identifier-Zeichen", nicht Zeilenanfang). Ein gaengiger Stil wie
    # "x = 1 REM siehe auch zoom(" wurde vorher NICHT als Kommentar erkannt
    # -- die unbalancierte Klammer darin erzeugte einen Phantom-Stack-
    # Frame, der den naechsten, unabhaengigen FOO(...)-Aufruf danach
    # faelschlich weiter offen erscheinen liess.
    text = "x = 1 REM siehe auch zoom(\nFOO(1, 2)\ny = 2"
    assert find_active_call(text) is None


def test_rem_case_insensitive():
    text = "rem zoom(\nFOO(1, "
    assert find_active_call(text) == ("FOO", 1)
