"""ENUM_NAME und die Schritt-Operatoren `++` / `--`.

Beides stand als Kleinkram auf der Luecken-Liste: ein ENUM liess sich nur
in eine Richtung lesen (Name -> Zahl), und fuer `i = i + 1` gab es zwar
`+=`, aber nicht die kuerzeste Form.
"""
import pytest

from drachenhauch.errors import DHRuntimeError, ParseError


# --------------------------------------------------------- ENUM_NAME

ZUSTAND = 'ENUM Zustand = MENUE, SPIELT, PAUSE\n'


def test_enum_name_liefert_den_member(run_gb):
    out = run_gb(ZUSTAND + 'PRINT ENUM_NAME(Zustand, 1)\n')
    assert out == "SPIELT\n"


def test_enum_name_schreibt_gross(run_gb):
    """Wie TYPEOF -- damit ein Vergleich nicht davon abhaengt, wie jemand den
    Member hingeschrieben hat."""
    out = run_gb('ENUM Kleiner = eins, zwei\nPRINT ENUM_NAME(Kleiner, 0)\n')
    assert out == "EINS\n"


def test_enum_name_ohne_treffer_ist_leer(run_gb):
    """Ein Nachschlagen darf danebengehen: ein gespeicherter Wert aus einer
    aelteren Fassung ist der Normalfall, nicht der Ausnahmefall."""
    out = run_gb(ZUSTAND + 'PRINT "["; ENUM_NAME(Zustand, 99); "]"\n')
    assert out == "[]\n"


def test_enum_name_mit_expliziten_werten(run_gb):
    out = run_gb("""
ENUM Recht
    KEINS = 0
    LESEN = 4
    SCHREIBEN = 8
END ENUM
PRINT ENUM_NAME(Recht, 8)
""")
    assert out == "SCHREIBEN\n"


def test_enum_name_verlangt_ein_enum(run_gb):
    with pytest.raises(DHRuntimeError, match="erwartet ein ENUM"):
        run_gb('PRINT ENUM_NAME(5, 1)\n')


# ------------------------------------------------------- ++ und --

def test_schritt_auf_variable(run_gb):
    out = run_gb("""
DIM i AS INTEGER
i++
i++
PRINT i
i--
PRINT i
""")
    assert out == "2\n1\n"


def test_schritt_auf_feld(run_gb):
    out = run_gb("""
CLASS P
    DIM n AS INTEGER
END CLASS
DIM p AS P
p = NEW P()
p.n++
p.n++
PRINT p.n
""")
    assert out == "2\n"


def test_schritt_auf_array_element(run_gb):
    out = run_gb("""
DIM a[3] AS INTEGER
a[1]++
a[1]++
a[1]--
PRINT a[1]
""")
    assert out == "1\n"


def test_schritt_hinter_dem_doppelpunkt(run_gb):
    out = run_gb('DIM i AS INTEGER\ni = 1 : i++ : PRINT i\n')
    assert out == "2\n"


def test_schritt_im_einzeiligen_if(run_gb):
    out = run_gb("""
DIM i AS INTEGER
IF TRUE THEN i++
PRINT i
""")
    assert out == "1\n"


def test_ausdruecke_mit_doppeltem_vorzeichen_bleiben(run_gb):
    """`++`/`--` sind bewusst KEINE Lexer-Token: `5 - -3` und `5 + +3` sind
    gueltige Ausdruecke, und ein `++`-Token wuerde sie umdeuten. Erkannt wird
    nur die Anweisungs-Position mit Zeilenende dahinter."""
    out = run_gb("""
DIM x AS INTEGER
x = 5 - -3
PRINT x
x = 5 + +3
PRINT x
""")
    assert out == "8\n8\n"


def test_schritt_ist_keine_ausdrucksform(run_gb):
    """Bewusst nur eine Anweisung. Der Unterschied zwischen Prae- und Postfix
    ist die haeufigste Fehlerquelle an diesem Operator -- wer keinen Wert
    liefert, hat sie nicht."""
    with pytest.raises((ParseError, DHRuntimeError)):
        run_gb("""
DIM i AS INTEGER
DIM j AS INTEGER
j = i++
PRINT j
""")
