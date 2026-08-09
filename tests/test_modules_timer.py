"""Tests fuer das timer-Modul (TIMER_AFTER/EVERY/CANCEL/UPDATE + COOLDOWN).

Golden-Tests gegen dhrt. Zeitbasiert: gewartet wird per Busy-Wait auf
MILLIS() (konsolen-tauglich, kein Grafik-Bezug) mit grosszuegigen Margen
(Timer 20-40ms, Warten doppelt so lang).
"""
import pytest

from drachenhauch.errors import DHRuntimeError

_WAIT = (
    "SUB warte(ms AS INTEGER)\n"
    "    DIM bis AS INTEGER\n"
    "    bis = MILLIS() + ms\n"
    "    WHILE MILLIS() < bis\n"
    "    WEND\n"
    "END SUB\n"
)


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def test_after_fires_once_then_clears(run_gb):
    src = (
        'IMPORT "timer"\n' + _WAIT +
        'SUB peng()\n    PRINT "peng"\nEND SUB\n'
        "DIM id AS INTEGER\n"
        "id = TIMER_AFTER(40, peng)\n"
        "TIMER_UPDATE()\n"                 # noch nicht faellig
        'PRINT "vorher"\n'
        "PRINT TIMER_ACTIVE(id)\n"
        "warte(80)\n"
        "TIMER_UPDATE()\n"                 # jetzt faellig -> peng
        "PRINT TIMER_ACTIVE(id)\n"         # AFTER raeumt sich weg
        "TIMER_UPDATE()\n"                 # feuert nicht erneut
        "PRINT TIMER_COUNT()\n"
    )
    assert _lines(run_gb(src)) == ["vorher", "TRUE", "peng", "FALSE", "0"]


def test_every_refires_until_cancel(run_gb):
    src = (
        'IMPORT "timer"\n' + _WAIT +
        'SUB tick()\n    PRINT "tick"\nEND SUB\n'
        "DIM id AS INTEGER\n"
        "id = TIMER_EVERY(20, tick)\n"
        "warte(40)\nTIMER_UPDATE()\n"
        "warte(40)\nTIMER_UPDATE()\n"
        "TIMER_CANCEL(id)\n"
        "warte(40)\nTIMER_UPDATE()\n"      # abgebrochen -> nichts
        "PRINT TIMER_ACTIVE(id)\n"
        "TIMER_CANCEL(id)\n"               # doppelt canceln = No-Op
        "TIMER_CANCEL(999)\n"              # unbekannte ID = No-Op
    )
    assert _lines(run_gb(src)) == ["tick", "tick", "FALSE"]


def test_after_zero_fires_on_next_update(run_gb):
    src = (
        'IMPORT "timer"\n'
        'SUB f()\n    PRINT "jetzt"\nEND SUB\n'
        "TIMER_AFTER(0, f)\n"
        "TIMER_UPDATE()\n"
    )
    assert _lines(run_gb(src)) == ["jetzt"]


def test_cooldown_locks_and_frees(run_gb):
    src = (
        'IMPORT "timer"\n'
        'PRINT COOLDOWN("a", 60000)\n'     # frei -> TRUE + Sperre
        'PRINT COOLDOWN("a", 60000)\n'     # gesperrt
        'PRINT COOLDOWN("b", 60000)\n'     # andere ID unabhaengig
        'PRINT COOLDOWN("c", 0)\n'         # ms=0 -> nie gesperrt
        'PRINT COOLDOWN("c", 0)\n'
        "TIMER_CLEAR()\n"
        'PRINT COOLDOWN("a", 60000)\n'     # CLEAR gibt frei
    )
    assert _lines(run_gb(src)) == ["TRUE", "FALSE", "TRUE", "TRUE", "TRUE", "TRUE"]


def test_validation_messages(run_gb):
    src = "\n".join([
        'IMPORT "timer"',
        "SUB f()",
        "END SUB",
        "TRY", "    TIMER_AFTER(-1, f)", "CATCH e", "    PRINT e", "END TRY",
        "TRY", "    TIMER_EVERY(0, f)", "CATCH e", "    PRINT e", "END TRY",
        "TRY", "    TIMER_AFTER(10, 5)", "CATCH e", "    PRINT e", "END TRY",
        "TRY", '    COOLDOWN("x", -1)', "CATCH e", "    PRINT e", "END TRY",
    ])
    out = run_gb(src)
    assert "TIMER_AFTER: ms muss >= 0 sein" in out
    assert "TIMER_EVERY: ms muss > 0 sein" in out
    assert "TIMER_AFTER: erwartet FUNCREF" in out
    assert "COOLDOWN: ms muss >= 0 sein" in out


def test_callback_may_register_new_timer(run_gb):
    # Re-Entranz: ein Callback darf selbst Timer registrieren; der neue
    # feuert fruehestens beim naechsten TIMER_UPDATE.
    src = (
        'IMPORT "timer"\n'
        'SUB zweiter()\n    PRINT "zwei"\nEND SUB\n'
        'SUB erster()\n    PRINT "eins"\n    TIMER_AFTER(0, zweiter)\nEND SUB\n'
        "TIMER_AFTER(0, erster)\n"
        "TIMER_UPDATE()\n"                 # feuert nur "eins"
        "PRINT TIMER_COUNT()\n"            # zweiter wartet noch
        "TIMER_UPDATE()\n"                 # jetzt "zwei"
    )
    assert _lines(run_gb(src)) == ["eins", "1", "zwei"]
