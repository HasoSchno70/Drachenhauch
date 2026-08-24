"""Mit Geld rechnen: `CENT`, `EURO$`, `ROUND_HALF_UP` (Weg A aus
docs/entwurf-geldtyp.md).

Der Anlass steht in einer einzigen Zeile: `INT(19.99 * 100)` ergibt **1998**.
Der uebliche Rat gegen Fliesskomma-Geld ("rechne in ganzen Cent") faellt also
selbst in die Falle, gegen die er gedacht ist -- weil 19.99 als
Fliesskommazahl minimal UNTER 19,99 liegt und INT abschneidet.
"""
import pytest

from drachenhauch.errors import DHRuntimeError


def _p(run_gb, ausdruck):
    return run_gb(f"PRINT {ausdruck}\n").strip()


# --------------------------------------------------------- der eigentliche Grund
def test_die_falle_gibt_es_wirklich(run_gb):
    """Nicht nur behauptet -- so verhaelt sich der Rechner heute."""
    assert _p(run_gb, "INT(19.99 * 100)") == "1998"
    assert _p(run_gb, "INT(0.29 * 100)") == "28"


def test_und_cent_faellt_nicht_hinein(run_gb):
    assert _p(run_gb, "CENT(19.99)") == "1999"
    assert _p(run_gb, "CENT(0.29)") == "29"
    assert _p(run_gb, "CENT(0.1 + 0.2)") == "30"


# ------------------------------------------------------------------------ CENT
def test_negative_betraege(run_gb):
    assert _p(run_gb, "CENT(-19.99)") == "-1999"
    assert _p(run_gb, "CENT(-0.005)") == "-1", "kaufmaennisch: von der Null weg"


def test_null_und_ganze_zahlen(run_gb):
    assert _p(run_gb, "CENT(0)") == "0"
    assert _p(run_gb, "CENT(7)") == "700"
    assert _p(run_gb, "CENT(7.0)") == "700"


def test_mehr_als_zwei_nachkommastellen_werden_gerundet(run_gb):
    assert _p(run_gb, "CENT(1.005)") == "101"
    assert _p(run_gb, "CENT(1.004)") == "100"
    assert _p(run_gb, "CENT(9.999)") == "1000"


def test_cent_aus_text(run_gb):
    """Damit ein Betrag aus einer CSV-Datei nicht erst durch FLOAT muss."""
    assert _p(run_gb, 'CENT("19,99")') == "1999"
    assert _p(run_gb, 'CENT("19.99")') == "1999"
    assert _p(run_gb, 'CENT("-19,99")') == "-1999"
    assert _p(run_gb, 'CENT(" 42 ")') == "4200"


def test_tausendertrenner_im_text(run_gb):
    assert _p(run_gb, 'CENT("1.234,56")') == "123456", "deutsch"
    assert _p(run_gb, 'CENT("1,234.56")') == "123456", "englisch"
    assert _p(run_gb, 'CENT("1.234.567")') == "123456700", "mehrfach = Tausender"


def test_text_mit_waehrung_ist_ein_fehler(run_gb):
    """Lieber ein Fehler als still die 19 aus "19,99 EUR" zu nehmen."""
    with pytest.raises(DHRuntimeError) as e:
        run_gb('PRINT CENT("19,99 EUR")\n')
    assert "kein Betrag" in str(e.value)


def test_leerer_text(run_gb):
    with pytest.raises(DHRuntimeError):
        run_gb('PRINT CENT("")\n')


# ----------------------------------------------------------------------- EURO$
def test_deutsche_schreibweise(run_gb):
    assert _p(run_gb, "EURO$(1999)") == "19,99 €"
    assert _p(run_gb, "EURO$(123456789)") == "1.234.567,89 €"
    assert _p(run_gb, "EURO$(-1999)") == "-19,99 €"
    assert _p(run_gb, "EURO$(5)") == "0,05 €"
    assert _p(run_gb, "EURO$(0)") == "0,00 €"
    assert _p(run_gb, "EURO$(100000)") == "1.000,00 €"


def test_anderes_symbol_oder_gar_keins(run_gb):
    assert _p(run_gb, 'EURO$(1999, "CHF")') == "19,99 CHF"
    assert _p(run_gb, 'EURO$(1999, "")') == "19,99"


def test_euro_nimmt_keine_kommazahl_und_sagt_warum(run_gb):
    """Der haeufigste Denkfehler -- und die Fehlermeldung ist gleich die
    Anleitung."""
    with pytest.raises(DHRuntimeError) as e:
        run_gb("PRINT EURO$(19.99)\n")
    msg = str(e.value)
    assert "GANZE CENT" in msg and "CENT(19.99)" in msg and "1999" in msg


# --------------------------------------------------------------- ROUND_HALF_UP
def test_kaufmaennisch_gegen_ROUND(run_gb):
    """`ROUND` rundet zur GERADEN Zahl (wie Python); kaufmaennisch wird von
    der Null weg gerundet. Beides ist richtig -- nur nicht dasselbe."""
    assert _p(run_gb, "ROUND(2.5)") == "2"
    assert _p(run_gb, "ROUND_HALF_UP(2.5)") == "3"
    assert _p(run_gb, "ROUND(3.5)") == "4"
    assert _p(run_gb, "ROUND_HALF_UP(3.5)") == "4"
    assert _p(run_gb, "ROUND_HALF_UP(-2.5)") == "-3"


def test_auf_nachkommastellen(run_gb):
    assert _p(run_gb, "ROUND_HALF_UP(2.675, 2)") == "2.68"
    assert _p(run_gb, "ROUND_HALF_UP(1.0 / 3.0, 4)") == "0.3333"
    assert _p(run_gb, "ROUND_HALF_UP(9.99, 1)") == "10.0"


def test_gerundet_wird_was_dasteht(run_gb):
    """2.675 ist als FLOAT in Wahrheit 2.67499999999999982... Wer die
    Binaerentwicklung rundet, bekommt formal Recht (2,67) und praktisch eine
    Rechnung, die niemand nachvollziehen kann."""
    assert _p(run_gb, "ROUND(2.675, 2)") == "2.67"
    assert _p(run_gb, "ROUND_HALF_UP(2.675, 2)") == "2.68"


def test_negative_stellenzahl(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb("PRINT ROUND_HALF_UP(1.5, -1)\n")
    assert ">= 0" in str(e.value)


# ----------------------------------------------------------- zusammen gerechnet
def test_eine_ganze_rechnung_geht_auf(run_gb):
    """Der Sinn der Uebung: in Cent summiert stimmt am Ende der Cent."""
    src = (
        "DIM preis AS ARRAY OF FLOAT\n"
        "preis = [19.99, 0.29, 4.95, 100.0]\n"
        "DIM menge AS ARRAY OF INTEGER\n"
        "menge = [3, 7, 1, 2]\n"
        "DIM summe AS INTEGER\n"
        "DIM i AS INTEGER\n"
        "summe = 0\n"
        "FOR i = 0 TO LEN(preis) - 1\n"
        "    summe = summe + CENT(preis[i]) * menge[i]\n"
        "NEXT\n"
        "PRINT EURO$(summe)\n"
        "PRINT EURO$(ROUND_HALF_UP(summe * 0.19))\n"
    )
    # 3*1999 + 7*29 + 495 + 2*10000 = 5997 + 203 + 495 + 20000 = 26695
    assert run_gb(src).split("\n")[0].strip() == "266,95 €"
    # 19 % davon = 5072,05 Cent -> kaufmaennisch 5072
    assert run_gb(src).split("\n")[1].strip() == "50,72 €"


def test_zehnmal_zehn_cent_sind_ein_euro(run_gb):
    """In FLOAT ergibt dieselbe Schleife 0.9999999999999999."""
    src = ("DIM s AS INTEGER\nDIM i AS INTEGER\ns = 0\n"
           "FOR i = 1 TO 10\n    s = s + CENT(0.1)\nNEXT\n"
           "PRINT EURO$(s)\nPRINT s = 100\n")
    assert run_gb(src).strip().split("\n") == ["1,00 €", "TRUE"]
