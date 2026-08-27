"""Gebundene Methoden als FUNCREF (`f = spieler.tick`).

Eine Methode OHNE Klammern ist ein FUNCREF-Wert, der seine Instanz
mittraegt. Damit koennen die Rueckruf-Schnittstellen der Laufzeit
(GUI_ON_*, TIMER_*, SORT) auf ein Objekt zeigen, statt eine freie SUB plus
globale Variable zu verlangen.

Hier steht nur, was OHNE Fenster laeuft -- damit die Datei auch im
grafikfreien dhrt-Bau der posix-CI durchlaeuft. Der GUI-Teil (echter Klick
ueber die Automation-Wiedergabe) liegt in
`test_gebundene_methoden_gui.py`, das in `conftest._BRAUCHT_GRAFIK` steht.
"""
import pytest

from drachenhauch.errors import DHRuntimeError

ZAEHLER = """
CLASS Zaehler
    DIM stand AS INTEGER
    SUB tick()
        Self.stand = Self.stand + 1
    END SUB
    FUNCTION mal(x AS INTEGER) AS INTEGER
        RETURN x * Self.stand
    END FUNCTION
END CLASS
"""


def test_gebundene_sub_wirkt_auf_ihre_instanz(run_gb):
    out = run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
DIM f AS FUNCREF
f = z.tick
f()
f()
PRINT z.stand
""")
    assert out == "2\n"


def test_gebundene_funktion_mit_argument_und_rueckgabe(run_gb):
    out = run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
z.stand = 3
DIM f AS FUNCREF
f = z.mal
PRINT f(7)
""")
    assert out == "21\n"


def test_zwei_instanzen_bleiben_getrennt(run_gb):
    out = run_gb(ZAEHLER + """
DIM a AS Zaehler
DIM b AS Zaehler
a = NEW Zaehler()
b = NEW Zaehler()
DIM fa AS FUNCREF
DIM fb AS FUNCREF
fa = a.tick
fb = b.tick
fa()
fa()
fb()
PRINT a.stand; " "; b.stand
""")
    assert out == "2 1\n"


def test_ueberschriebene_methode_gewinnt(run_gb):
    """Aufgeloest wird beim AUFRUF, nicht beim Binden -- eine als Elternklasse
    gehaltene Instanz ruft trotzdem die Ueberschreibung des Kindes."""
    out = run_gb("""
CLASS Tier
    SUB laut()
        PRINT "..."
    END SUB
END CLASS
CLASS Hund EXTENDS Tier
    SUB laut()
        PRINT "Wuff"
    END SUB
END CLASS
DIM t AS Tier
t = NEW Hund()
DIM f AS FUNCREF
f = t.laut
f()
""")
    assert out == "Wuff\n"


def test_ist_nach_aussen_ein_funcref(run_gb):
    out = run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
DIM f AS FUNCREF
f = z.tick
PRINT TYPEOF(f)
PRINT f
""")
    assert out == "FUNCREF\n<FUNCREF zaehler.tick>\n"


def test_gleichheit_vergleicht_instanz_und_methode(run_gb):
    out = run_gb(ZAEHLER + """
DIM a AS Zaehler
DIM b AS Zaehler
a = NEW Zaehler()
b = NEW Zaehler()
PRINT a.tick = a.tick
PRINT a.tick = b.tick
PRINT a.tick = a.mal
""")
    assert out == "TRUE\nFALSE\nFALSE\n"


def test_unbekannter_name_bleibt_ein_fehler(run_gb):
    with pytest.raises(DHRuntimeError, match="existiert nicht"):
        run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
DIM f AS FUNCREF
f = z.gibtsnicht
""")


def test_feld_gewinnt_vor_methode(run_gb):
    """Heisst ein Feld wie eine Methode, liefert der Zugriff weiterhin das
    Feld -- sonst haette diese Erweiterung bestehenden Code umgedeutet."""
    out = run_gb("""
CLASS K
    DIM wert AS INTEGER
    SUB wert2()
    END SUB
END CLASS
DIM k AS K
k = NEW K()
k.wert = 42
PRINT k.wert
""")
    assert out == "42\n"


def test_timer_ruft_gebundene_methode(run_gb):
    out = run_gb(ZAEHLER + """
IMPORT "timer"
DIM z AS Zaehler
z = NEW Zaehler()
TIMER_AFTER(0, z.tick)
TIMER_UPDATE()
PRINT z.stand
""")
    assert out == "1\n"


def test_sort_mit_gebundenem_vergleicher(run_gb):
    out = run_gb("""
CLASS Regel
    DIM absteigend AS BOOLEAN
    FUNCTION cmp(a AS INTEGER, b AS INTEGER) AS INTEGER
        IF Self.absteigend THEN RETURN b - a
        RETURN a - b
    END FUNCTION
END CLASS
DIM r AS Regel
r = NEW Regel()
r.absteigend = TRUE
DIM a AS ARRAY OF INTEGER
a = [3, 1, 2]
SORT(a, r.cmp)
PRINT a[0]; a[1]; a[2]
""")
    assert out == "321\n"


def test_task_start_lehnt_gebundene_methode_ab(run_gb):
    """Ein Auftrag laeuft in einem eigenen Prozess -- die Instanz gibt es dort
    nicht. Das muss klar gesagt werden statt still ins Leere zu laufen."""
    with pytest.raises(DHRuntimeError, match="an ein Objekt gebunden"):
        run_gb(ZAEHLER + """
DIM z AS Zaehler
z = NEW Zaehler()
TASK_START(z.tick)
""")
