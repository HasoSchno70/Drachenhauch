"""MILLIS/TIMER: die Stoppuhr des Programms.

Frueherer Stolperstein: die Doku sagte "ms seit Programmstart", die Runtime
gab Millisekunden seit 1970 zurueck (gemessen: 1786881140256). Differenzen
stimmten dadurch, aber jede Annahme "faengt bei 0 an" war falsch -- und die
Systemuhr kann springen (Zeitumstellung, NTP), mitten in einer Messung.

Jetzt: eine monotone Uhr ab Programmstart. Datum und Uhrzeit liefert
`ZEIT_JETZT()` aus dem Modul `zeit`.
"""
import pytest

from drachenhauch.errors import DHRuntimeError


def _zahlen(out):
    import re
    return [float(x) for x in re.findall(r"-?\d+(?:\.\d+)?", out)]


def test_millis_faengt_klein_an(run_gb):
    """Beim Start steht die Uhr nahe null -- nicht bei 1,78 Billionen."""
    zahlen = _zahlen(run_gb('PRINT MILLIS()\n'))
    assert zahlen, "keine Ausgabe"
    assert 0 <= zahlen[0] < 5000, f"MILLIS() = {zahlen[0]} -- das ist keine Stoppuhr"


def test_millis_zaehlt_vorwaerts(run_gb):
    out = run_gb(
        'DIM a AS INTEGER : a = MILLIS()\n'
        'DIM i AS INTEGER : DIM z AS INTEGER : z = 0\n'
        'FOR i = 1 TO 2000000\n'
        '    z = z + 1\n'
        'NEXT\n'
        'DIM b AS INTEGER : b = MILLIS()\n'
        'PRINT b >= a\n'
        'PRINT b - a < 60000\n')
    assert out.split() == ["TRUE", "TRUE"], out


def test_timer_ist_dieselbe_uhr_in_sekunden(run_gb):
    """TIMER() * 1000 und MILLIS() muessen denselben Zeitpunkt meinen --
    frueher hatten sie verschiedene Nullpunkte."""
    zahlen = _zahlen(run_gb(
        'DIM i AS INTEGER : DIM z AS INTEGER : z = 0\n'
        'FOR i = 1 TO 2000000\n'
        '    z = z + 1\n'
        'NEXT\n'
        'PRINT MILLIS()\n'
        'PRINT TIMER() * 1000.0\n'))
    assert len(zahlen) >= 2, zahlen
    ms, timer_ms = zahlen[0], zahlen[1]
    # Zwischen den beiden Zeilen vergehen ein paar Millisekunden.
    assert abs(timer_ms - ms) < 50, (ms, timer_ms)


def test_millis_und_zeit_jetzt_sind_zwei_paar_schuhe(run_gb):
    """Die Stoppuhr ist klein, die Wanduhr gross. Wer sie verwechselt, soll
    es am Zahlenbereich sofort merken."""
    zahlen = _zahlen(run_gb(
        'IMPORT "zeit"\n'
        'PRINT MILLIS()\n'
        'PRINT ZEIT_JETZT()\n'))
    assert len(zahlen) >= 2, zahlen
    assert zahlen[0] < 5000, zahlen
    assert zahlen[1] > 1_600_000_000, zahlen   # nach 2020


# --- GUI_CONFIRM: Knopf-Beschriftung ----------------------------------
#
# Der Dialog selbst blockiert auf eine Antwort und laesst sich im Test nicht
# klicken. Pruefbar ist aber, was DAVOR passiert: welche Stil-Angaben die
# Runtime annimmt und welche sie mit Klartext zurueckweist.

def test_gui_confirm_lehnt_unbekannten_stil_ab(run_gb):
    with pytest.raises(DHRuntimeError, match=r'"janein"'):
        run_gb('SCREEN(200, 100, "t")\n'
               'DIM x AS BOOLEAN\n'
               'x = GUI_CONFIRM("Titel", "Text", "vielleicht")\n')


def test_gui_confirm_nimmt_drei_argumente(run_gb):
    """Frueher warnte der Compiler bei drei Argumenten ('ueberzaehlige werden
    ignoriert') -- der Stil kam dann nie an."""
    out = run_gb('PRINT "kompiliert"\n'
                 'IF FALSE THEN\n'
                 '    DIM x AS BOOLEAN\n'
                 '    x = GUI_CONFIRM("Titel", "Text", "janein")\n'
                 'END IF\n')
    assert "kompiliert" in out
    assert "Warnung" not in out, out
