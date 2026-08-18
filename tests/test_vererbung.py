"""WP G -- Vererbung rund machen: SUPER.Methode() und ABSTRACT.

Zwei Luecken, die `docs/sprache.md` bis dahin selbst im Beispiel zeigte:
"Eigene Init, ruft super.Init nicht automatisch auf" -- gefolgt von drei
abgeschriebenen Zuweisungen.
"""
import pytest


BASIS = """CLASS Spieler
    DIM x AS FLOAT
    DIM hp AS INTEGER
    SUB Init(sx AS FLOAT)
        x = sx
        hp = 100
    END SUB
    FUNCTION Wer() AS STRING
        RETURN "Spieler"
    END FUNCTION
END CLASS
"""


# ------------------------------------------------------------------ SUPER

def test_super_init_statt_abschreiben(run_gb):
    out = run_gb(BASIS + """CLASS Held EXTENDS Spieler
    DIM waffe AS STRING
    SUB Init(sx AS FLOAT, w AS STRING)
        SUPER.Init(sx)
        hp = 150
        waffe = w
    END SUB
END CLASS
DIM h AS Held
h = NEW Held(10.0, "Schwert")
PRINT h.x
PRINT h.hp
PRINT h.waffe""")
    # x kommt aus der Elternklasse, hp wird danach ueberschrieben.
    assert out.split() == ["10.0", "150", "Schwert"]


def test_super_ruft_die_ueberschriebene_methode(run_gb):
    out = run_gb(BASIS + """CLASS Held EXTENDS Spieler
    FUNCTION Wer() AS STRING
        RETURN SUPER.Wer() + " (Held)"
    END FUNCTION
END CLASS
DIM h AS Held
h = NEW Held(1.0)
PRINT h.Wer()""")
    assert out == "Spieler (Held)\n"


def test_super_ueber_drei_ebenen(run_gb):
    """Jede Ebene fragt ihre EIGENE Elternklasse, nicht die des Objekts. Ohne
    das riefe die mittlere Ebene sich selbst, bis der Stapel voll ist."""
    out = run_gb(BASIS + """CLASS Held EXTENDS Spieler
    FUNCTION Wer() AS STRING
        RETURN SUPER.Wer() + "/Held"
    END FUNCTION
END CLASS
CLASS Meister EXTENDS Held
    FUNCTION Wer() AS STRING
        RETURN SUPER.Wer() + "/Meister"
    END FUNCTION
END CLASS
DIM m AS Meister
m = NEW Meister(1.0)
PRINT m.Wer()""")
    assert out == "Spieler/Held/Meister\n"


def test_super_findet_auch_uebersprungene_ebenen(run_gb):
    """Die Mitte ueberschreibt nichts -- SUPER muss weiter hochsuchen."""
    out = run_gb(BASIS + """CLASS Mitte EXTENDS Spieler
END CLASS
CLASS Unten EXTENDS Mitte
    FUNCTION Wer() AS STRING
        RETURN SUPER.Wer() + "/Unten"
    END FUNCTION
END CLASS
DIM u AS Unten
u = NEW Unten(1.0)
PRINT u.Wer()""")
    assert out == "Spieler/Unten\n"


def test_super_mit_argumenten_und_rueckgabewert(run_gb):
    out = run_gb("""CLASS Rechner
    FUNCTION Plus(a AS INTEGER, b AS INTEGER) AS INTEGER
        RETURN a + b
    END FUNCTION
END CLASS
CLASS LauterRechner EXTENDS Rechner
    FUNCTION Plus(a AS INTEGER, b AS INTEGER) AS INTEGER
        RETURN SUPER.Plus(a, b) * 10
    END FUNCTION
END CLASS
DIM r AS LauterRechner
r = NEW LauterRechner()
PRINT r.Plus(2, 3)""")
    assert out == "50\n"


def test_super_in_einer_sub_ohne_rueckgabewert(run_gb):
    out = run_gb("""CLASS A
    SUB Melde()
        PRINT "A"
    END SUB
END CLASS
CLASS B EXTENDS A
    SUB Melde()
        SUPER.Melde()
        PRINT "B"
    END SUB
END CLASS
DIM b AS B
b = NEW B()
b.Melde()""")
    assert out == "A\nB\n"


def test_super_ohne_elternklasse_ist_ein_fehler(run_gb):
    with pytest.raises(Exception, match="keine Elternklasse"):
        run_gb("CLASS A\n    SUB M()\n        SUPER.M()\n    END SUB\nEND CLASS\n"
               "DIM a AS A\na = NEW A()\na.M()")


def test_super_auf_unbekannte_methode_ist_ein_fehler(run_gb):
    with pytest.raises(Exception, match="haben keine Methode"):
        run_gb(BASIS + "CLASS Held EXTENDS Spieler\n"
                       "    SUB M()\n        SUPER.GibtsNicht()\n    END SUB\n"
                       "END CLASS\n"
                       "DIM h AS Held\nh = NEW Held(1.0)\nh.M()")


def test_super_ausserhalb_einer_methode_ist_ein_fehler(run_gb):
    with pytest.raises(Exception, match="SUPER geht nur in einer Methode"):
        run_gb("SUB frei()\n    SUPER.Irgendwas()\nEND SUB\nfrei()")


def test_super_bleibt_als_variablenname_erlaubt(run_gb):
    """SUPER ist bewusst kein Schluesselwort -- bestehender Code darf es
    weiter als Namen benutzen."""
    assert run_gb("DIM super AS INTEGER\nsuper = 5\nPRINT super") == "5\n"


# --------------------------------------------------------------- ABSTRACT

FORM = """CLASS Form
    DIM name AS STRING
    SUB Init(n AS STRING)
        name = n
    END SUB
    ABSTRACT FUNCTION Flaeche() AS FLOAT
    FUNCTION Zeige() AS STRING
        RETURN name + ": " + STR$(Flaeche())
    END FUNCTION
END CLASS
"""


def test_abstract_wird_von_der_unterklasse_ausgefuellt(run_gb):
    out = run_gb(FORM + """CLASS Quadrat EXTENDS Form
    DIM a AS FLOAT
    SUB Init(seite AS FLOAT)
        SUPER.Init("Quadrat")
        a = seite
    END SUB
    FUNCTION Flaeche() AS FLOAT
        RETURN a * a
    END FUNCTION
END CLASS
DIM q AS Quadrat
q = NEW Quadrat(3.0)
PRINT q.Zeige()""")
    assert out == "Quadrat: 9.0\n"


def test_die_basisklasse_ruft_die_ausgefuellte_methode(run_gb):
    """`Zeige()` steht in Form und ruft `Flaeche()` -- zur Laufzeit landet das
    bei der Unterklasse."""
    out = run_gb(FORM + """CLASS Zwei EXTENDS Form
    SUB Init()
        SUPER.Init("Zwei")
    END SUB
    FUNCTION Flaeche() AS FLOAT
        RETURN 2.0
    END FUNCTION
END CLASS
DIM z AS Zwei
z = NEW Zwei()
PRINT z.Zeige()""")
    assert out == "Zwei: 2.0\n"


def test_new_auf_die_unfertige_klasse_ist_ein_fehler(run_gb):
    # Und zwar beim UEBERSETZEN, nicht erst zur Laufzeit -- der Compiler kennt
    # alle Klassen, es gibt keinen Grund zu warten.
    with pytest.raises(Exception, match="ohne sie auszufuellen"):
        run_gb(FORM + 'DIM f AS Form\nf = NEW Form("x")')


def test_die_meldung_nennt_die_fehlende_methode(run_gb):
    with pytest.raises(Exception, match="flaeche"):
        run_gb(FORM + 'DIM f AS Form\nf = NEW Form("x")')


def test_halb_ausgefuellte_unterklasse_bleibt_unfertig(run_gb):
    with pytest.raises(Exception, match="zeichne"):
        run_gb("""CLASS Form
    ABSTRACT FUNCTION Flaeche() AS FLOAT
    ABSTRACT SUB Zeichne()
END CLASS
CLASS Halb EXTENDS Form
    FUNCTION Flaeche() AS FLOAT
        RETURN 1.0
    END FUNCTION
END CLASS
DIM h AS Halb
h = NEW Halb()""")


def test_zwei_offene_methoden_werden_beide_genannt(run_gb):
    with pytest.raises(Exception, match="flaeche, zeichne"):
        run_gb("""CLASS Form
    ABSTRACT FUNCTION Flaeche() AS FLOAT
    ABSTRACT SUB Zeichne()
END CLASS
DIM f AS Form
f = NEW Form()""")


def test_ganz_ausgefuellte_unterklasse_geht(run_gb):
    out = run_gb("""CLASS Form
    ABSTRACT FUNCTION Flaeche() AS FLOAT
    ABSTRACT SUB Zeichne()
END CLASS
CLASS Voll EXTENDS Form
    FUNCTION Flaeche() AS FLOAT
        RETURN 1.0
    END FUNCTION
    SUB Zeichne()
        PRINT "gezeichnet"
    END SUB
END CLASS
DIM v AS Voll
v = NEW Voll()
v.Zeichne()
PRINT v.Flaeche()""")
    assert out == "gezeichnet\n1.0\n"


def test_abstract_mit_parametern(run_gb):
    out = run_gb("""CLASS Basis
    ABSTRACT FUNCTION Mal(n AS INTEGER) AS INTEGER
END CLASS
CLASS Doppelt EXTENDS Basis
    FUNCTION Mal(n AS INTEGER) AS INTEGER
        RETURN n * 2
    END FUNCTION
END CLASS
DIM d AS Doppelt
d = NEW Doppelt()
PRINT d.Mal(21)""")
    assert out == "42\n"


def test_abstract_bleibt_als_variablenname_erlaubt(run_gb):
    """ABSTRACT ist bewusst kein Schluesselwort geworden -- sonst waere
    `DIM abstract AS INTEGER` in bestehendem Code ploetzlich ein Fehler."""
    assert run_gb("DIM abstract AS INTEGER\nabstract = 7\nPRINT abstract") == "7\n"


def test_abstract_allein_bleibt_im_class_body_unerwartet(run_gb):
    with pytest.raises(Exception, match="CLASS-Body"):
        run_gb("CLASS A\n    abstract\nEND CLASS")
