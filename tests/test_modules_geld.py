"""Modul `geld` -- ein Betrag als eigener Wert (Weg C aus
docs/entwurf-geldtyp.md).

Weg A (`CENT`/`EURO$`/`ROUND_HALF_UP`) ist eine **Rechenweise**: in Cent
rechnen und daran denken. Hier ist es ein **Typ**, und der denkt mit --
`betrag + 1.0` ist ein Fehler statt einer stillen Fliesskomma-Rechnung.
"""
import pytest

from drachenhauch.errors import DHRuntimeError

KOPF = 'IMPORT "geld"\n'


def _p(run_gb, zeilen):
    return run_gb(KOPF + zeilen).strip()


# ------------------------------------------------------------------ Grundlagen
def test_aus_text_und_anzeige(run_gb):
    assert _p(run_gb, 'PRINT GELD_NEU("19,99")\n') == "19,99 €"
    assert _p(run_gb, 'PRINT GELD_NEU("1.234,56")\n') == "1.234,56 €"
    assert _p(run_gb, 'PRINT GELD_NEU("-19,99")\n') == "-19,99 €"


def test_aus_einer_zahl_ohne_die_falle(run_gb):
    """19.99 liegt als FLOAT minimal unter 19,99 -- GELD_NEU rundet die Zahl,
    die dasteht."""
    assert _p(run_gb, "PRINT GELD_CENT(GELD_NEU(19.99))\n") == "1999"
    assert _p(run_gb, "PRINT GELD_CENT(GELD_NEU(0.1 + 0.2))\n") == "30"


def test_eine_neue_variable_ist_null_und_nicht_nil(run_gb):
    assert _p(run_gb, "DIM leer AS GELD\nPRINT leer\n") == "0,00 €"


def test_aus_cent_und_zurueck(run_gb):
    assert _p(run_gb, "PRINT GELD_AUS_CENT(1999)\n") == "19,99 €"
    assert _p(run_gb, 'PRINT GELD_CENT(GELD_NEU("19,99"))\n') == "1999"


def test_text_mit_und_ohne_symbol(run_gb):
    assert _p(run_gb, 'PRINT GELD_TEXT$(GELD_NEU("19,99"))\n') == "19,99 €"
    assert _p(run_gb, 'PRINT GELD_TEXT$(GELD_NEU("19,99"), "CHF")\n') == "19,99 CHF"
    assert _p(run_gb, 'PRINT GELD_TEXT$(GELD_NEU("19,99"), "")\n') == "19,99"


# ------------------------------------------------------------------- Rechnen
def test_addieren_und_abziehen(run_gb):
    assert _p(run_gb, 'PRINT GELD_NEU("19,99") + GELD_NEU("0,01")\n') == "20,00 €"
    assert _p(run_gb, 'PRINT GELD_NEU("19,99") - GELD_NEU("20,00")\n') == "-0,01 €"


def test_mal_menge(run_gb):
    assert _p(run_gb, 'PRINT GELD_NEU("19,99") * 3\n') == "59,97 €"
    assert _p(run_gb, 'PRINT 3 * GELD_NEU("19,99")\n') == "59,97 €"


def test_prozent_bleibt_exakt(run_gb):
    """19 % von 72,71 sind 13,8149 -- nicht 13.814899999999998. Gerechnet
    wird ganzzahlig, der Faktor wird in Ziffern zerlegt."""
    assert _p(run_gb, 'PRINT GELD_NEU("72,71") * 0.19\n') == "13,8149 €"
    assert _p(run_gb, 'PRINT GELD_RUNDEN(GELD_NEU("72,71") * 0.19)\n') == "13,81 €"


def test_zwischenergebnisse_werden_nicht_versteckt(run_gb):
    """0,0551 € als "0,06 €" anzuzeigen waere bequem und irrefuehrend."""
    assert _p(run_gb, 'PRINT GELD_NEU("0,29") * 0.19\n') == "0,0551 €"


def test_teilen(run_gb):
    assert _p(run_gb, 'PRINT GELD_NEU("10,00") / 4\n') == "2,50 €"


def test_betrag_durch_betrag_ist_eine_zahl(run_gb):
    assert _p(run_gb, 'PRINT GELD_NEU("10,00") / GELD_NEU("4,00")\n') == "2.5"


def test_vorzeichen_und_betrag(run_gb):
    assert _p(run_gb, 'PRINT -GELD_NEU("19,99")\n') == "-19,99 €"
    assert _p(run_gb, 'PRINT GELD_ABS(GELD_NEU("-19,99"))\n') == "19,99 €"


def test_vergleichen_ist_exakt(run_gb):
    src = ('DIM a AS GELD\nDIM b AS GELD\n'
           'a = GELD_NEU("0,10") + GELD_NEU("0,20")\n'
           'b = GELD_NEU("0,30")\n'
           "PRINT a = b\nPRINT a < b\nPRINT a >= b\n")
    assert _p(run_gb, src).split("\n") == ["TRUE", "FALSE", "TRUE"]


def test_in_float_geht_dieselbe_rechnung_schief(run_gb):
    """Der Gegenbeweis in derselben Sprache -- damit der Test nicht nur
    behauptet, wozu der Typ gut ist."""
    assert _p(run_gb, "PRINT 0.1 + 0.2 = 0.3\n") == "FALSE"


def test_runden_auf_stellen(run_gb):
    assert _p(run_gb, 'PRINT GELD_RUNDEN(GELD_NEU("0,005"))\n') == "0,01 €"
    assert _p(run_gb, 'PRINT GELD_RUNDEN(GELD_NEU("-0,005"))\n') == "-0,01 €"
    # GELD_NEU haelt vier Stellen: aus 0,12345 wird beim Anlegen 0,1235,
    # daraus auf drei Stellen kaufmaennisch 0,124.
    assert _p(run_gb, 'PRINT GELD_NEU("0,12345")\n') == "0,1235 €"
    assert _p(run_gb, 'PRINT GELD_RUNDEN(GELD_NEU("0,12345"), 3)\n') == "0,124 €"


# -------------------------------------------------------------------- Aufteilen
def test_aufteilen_verliert_keinen_cent(run_gb):
    src = ('DIM t AS ARRAY OF GELD\nDIM s AS GELD\nDIM i AS INTEGER\n'
           't = GELD_TEILEN(GELD_NEU("10,00"), 3)\n'
           "s = GELD_AUS_CENT(0)\n"
           "FOR i = 0 TO 2\n    PRINT t[i]\n    s = s + t[i]\nNEXT\n"
           "PRINT s\n")
    assert _p(run_gb, src).split("\n") == ["3,34 €", "3,33 €", "3,33 €", "10,00 €"]


def test_aufteilen_ohne_teile(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'PRINT GELD_TEILEN(GELD_NEU("1,00"), 0)\n')
    assert "groesser als 0" in str(e.value)


# ----------------------------------------------------------------- Fehlerfaelle
def test_geld_und_zahl_mischen_sich_nicht(run_gb):
    """Der eigentliche Zweck des Typs."""
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'PRINT GELD_NEU("1,00") + 1.0\n')
    assert "mischen sich nicht" in str(e.value)


def test_euro_mal_euro(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'PRINT GELD_NEU("1,00") * GELD_NEU("2,00")\n')
    assert "Quadrat-Euro" in str(e.value)


def test_eine_zahl_landet_nicht_still_in_einer_geld_variablen(run_gb):
    """Anders als bei den uebrigen Modul-Typen ist GELD hier streng -- sonst
    waere die Trennung, fuer die es den Typ gibt, sofort wieder weg."""
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + "DIM x AS GELD\nx = 5\n")
    assert "GELD_NEU(5)" in str(e.value)


def test_builtin_will_wirklich_geld(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + "PRINT GELD_CENT(5)\n")
    assert "erwartet GELD" in str(e.value)


def test_text_ohne_betrag(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'PRINT GELD_NEU("neunzehn")\n')
    assert "kein Betrag" in str(e.value)


def test_durch_null(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'PRINT GELD_NEU("1,00") / 0\n')
    assert "Division durch 0" in str(e.value)


# --------------------------------------------------------- zusammen mit Weg A
def test_beide_wege_kommen_auf_dasselbe(run_gb):
    """CENT/EURO$ (Weg A) und das Modul (Weg C) muessen dieselbe Zahl
    liefern -- sie teilen sich die Rundungsregel."""
    src = ('DIM a AS INTEGER\nDIM b AS GELD\n'
           "a = CENT(19.99) * 3\n"
           'b = GELD_NEU(19.99) * 3\n'
           "PRINT EURO$(a)\nPRINT b\nPRINT a = GELD_CENT(b)\n")
    assert _p(run_gb, src).split("\n") == ["59,97 €", "59,97 €", "TRUE"]
