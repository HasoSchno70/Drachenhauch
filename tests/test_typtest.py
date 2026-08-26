"""Laufzeit-Typtest: `TYPEOF` bei Objekten und der `IS`-Operator.

Polymorphie funktionierte laengst -- man konnte einer polymorph gehaltenen
Referenz aber nicht ansehen, was sie ist: `TYPEOF` sagte pauschal "OBJECT",
und einen Typtest gab es gar nicht. Beides ist hier abgedeckt.
"""
import pytest

from drachenhauch.errors import DHRuntimeError

TIERE = """
CLASS Tier
END CLASS
CLASS Hund EXTENDS Tier
END CLASS
CLASS Dackel EXTENDS Hund
END CLASS
CLASS Katze EXTENDS Tier
END CLASS
"""


# ------------------------------------------------------------- TYPEOF

def test_typeof_nennt_die_klasse(run_gb):
    out = run_gb(TIERE + """
DIM t AS Tier
t = NEW Hund()
PRINT TYPEOF(t)
""")
    assert out == "HUND\n"


def test_typeof_bleibt_bei_werttypen_wie_bisher(run_gb):
    out = run_gb("""
DIM i AS INTEGER
DIM s AS STRING
DIM b AS BOOLEAN
DIM f AS FLOAT
PRINT TYPEOF(i); " "; TYPEOF(s); " "; TYPEOF(b); " "; TYPEOF(f)
PRINT TYPEOF(NIL)
""")
    assert out == "INTEGER STRING BOOLEAN FLOAT\nNIL\n"


def test_typeof_schreibt_gross_unabhaengig_von_der_deklaration(run_gb):
    """Alle anderen TYPEOF-Antworten sind gross -- ein Vergleich soll nicht
    davon abhaengen, wie jemand die Klasse hingeschrieben hat."""
    out = run_gb("""
CLASS GrossKlein
END CLASS
DIM g AS GrossKlein
g = NEW GrossKlein()
PRINT TYPEOF(g) = "GROSSKLEIN"
""")
    assert out == "TRUE\n"


# ----------------------------------------------------------------- IS

def test_is_trifft_die_eigene_klasse_und_jede_elternklasse(run_gb):
    out = run_gb(TIERE + """
DIM t AS Tier
t = NEW Dackel()
PRINT t IS Dackel
PRINT t IS Hund
PRINT t IS Tier
PRINT t IS Katze
""")
    assert out == "TRUE\nTRUE\nTRUE\nFALSE\n"


def test_is_auf_werttypen(run_gb):
    out = run_gb("""
DIM i AS INTEGER
DIM s AS STRING
i = 5
PRINT i IS INTEGER; " "; i IS STRING; " "; s IS STRING
""")
    assert out == "TRUE FALSE TRUE\n"


def test_is_nil_und_is_not_nil(run_gb):
    """Die Doku nannte `IS NIL` lange als fehlend -- jetzt gibt es beides."""
    out = run_gb(TIERE + """
DIM t AS Tier
PRINT t IS NIL; " "; t IS NOT NIL
t = NEW Hund()
PRINT t IS NIL; " "; t IS NOT NIL
""")
    assert out == "TRUE FALSE\nFALSE TRUE\n"


def test_is_not_ist_dasselbe_wie_verneintes_is(run_gb):
    out = run_gb(TIERE + """
DIM t AS Tier
t = NEW Katze()
PRINT (t IS NOT Hund) = (NOT (t IS Hund))
""")
    assert out == "TRUE\n"


def test_nil_ist_keine_instanz(run_gb):
    out = run_gb(TIERE + """
DIM t AS Tier
PRINT t IS Tier
""")
    assert out == "FALSE\n"


def test_is_auf_array_und_map(run_gb):
    out = run_gb("""
DIM a AS ARRAY OF INTEGER
DIM m AS MAP OF INTEGER
a = [1, 2]
PRINT a IS ARRAY; " "; m IS MAP; " "; a IS MAP
""")
    assert out == "TRUE TRUE FALSE\n"


def test_is_auf_modultyp(run_gb):
    out = run_gb("""
IMPORT "vec2"
DIM v AS VEC2
v = VEC2_NEW(1.0, 2.0)
PRINT v IS VEC2; " "; v IS INTEGER
""")
    assert out == "TRUE FALSE\n"


def test_is_auf_struct(run_gb):
    out = run_gb("""
STRUCT P
    DIM x AS INTEGER
END STRUCT
DIM p AS P
PRINT p IS P
""")
    assert out == "TRUE\n"


def test_tippfehler_im_typnamen_ist_ein_uebersetzungsfehler(run_gb):
    """Ohne diese Pruefung waere `x IS Gegnr` still fuer immer FALSE -- ein
    Test, der nie zuschlaegt, faellt niemandem auf."""
    with pytest.raises(DHRuntimeError, match="IS: unbekannter Typ"):
        run_gb("""
CLASS Gegner
END CLASS
DIM g AS Gegner
g = NEW Gegner()
PRINT g IS Gegnr
""")


def test_fehlender_import_wird_benannt(run_gb):
    with pytest.raises(DHRuntimeError, match='fehlt ein IMPORT'):
        run_gb("""
DIM x AS INTEGER
PRINT x IS VEC2
""")


def test_select_case_is_bleibt_unberuehrt(run_gb):
    """`CASE IS > 5` verschluckt sein IS im Case-Parser -- der neue Operator
    auf der Vergleichsebene darf daran nichts aendern."""
    out = run_gb("""
DIM i AS INTEGER
i = 7
SELECT CASE i
    CASE IS > 5
        PRINT "gross"
    CASE ELSE
        PRINT "klein"
END SELECT
""")
    assert out == "gross\n"


def test_is_in_bedingungen_und_ausdruecken(run_gb):
    out = run_gb(TIERE + """
DIM t AS Tier
t = NEW Hund()
IF t IS Hund AND t IS NOT Katze THEN PRINT "ja"
DIM b AS BOOLEAN
b = t IS Tier
PRINT b
""")
    assert out == "ja\nTRUE\n"


def test_is_ueber_namensraum(run_gb, tmp_path):
    """Der Typname hinter IS wird umgeschrieben wie bei DIM und NEW."""
    (tmp_path / "zoo.dh").write_text(
        "CLASS Tier\nEND CLASS\n", encoding="utf-8")
    out = run_gb("""
IMPORT "zoo.dh" AS zoo
DIM t AS zoo.Tier
t = NEW zoo.Tier()
PRINT t IS zoo.Tier
""", base=tmp_path)
    assert out == "TRUE\n"
