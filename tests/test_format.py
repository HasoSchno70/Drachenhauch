"""FORMAT$ -- welche Spezifizierer es wirklich gibt.

Aus dem Doku-Durchgang durch `builtins-core.md`: dort stand, die Maske folge
"Python's `%`-Operator". Tut sie nicht -- es sind genau sechs, und `%e`, in
Python selbstverstaendlich, meldet einen Fehler. Diese Tests halten die Menge
fest, damit Doku und Runtime nicht wieder auseinanderlaufen.
"""
import pytest


def test_format_spezifizierer_sind_vollstaendig_dokumentiert(run_gb):
    """builtins-core.md behauptete, die Maske folge "Python's %-Operator".

    Tut sie nicht: es gibt genau sechs Spezifizierer. `%e` -- in Python
    selbstverstaendlich -- meldet einen Fehler. Dieser Test haelt die Menge
    fest, damit die Doku und die Runtime nicht wieder auseinanderlaufen.
    """
    out = run_gb(
        'PRINT FORMAT$(42, "%05d")\n'
        'PRINT FORMAT$(42, "%i")\n'
        'PRINT FORMAT$(3.14159, "%.2f")\n'
        'PRINT FORMAT$(255, "%x")\n'
        'PRINT FORMAT$(255, "%X")\n'
        'PRINT FORMAT$("hi", "%s")\n'
        'PRINT FORMAT$(50, "%d%%")\n')
    assert out.split("\n")[:7] == ["00042", "42", "3.14", "ff", "FF", "hi", "50%"]


def test_unbekannter_spezifizierer_meldet(run_gb):
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError, match="unbekannter Spezifizierer"):
        run_gb('PRINT FORMAT$(1234567, "%e")\n')
