"""Die Standardeingabe (Punkt 4 aus docs/allzweck-audit-2.md).

WP A hat „Werkzeug in einer Kette" zum Ziel erklaert -- der halbe Weg fehlte
aber: ein Programm konnte lesen, was ihm als ARGUMENT gegeben wurde, aber
nicht, was ihm GEREICHT wurde. `dir | meinwerkzeug | sort` war damit nicht
schreibbar.

`STDIN()` liefert die Standardeingabe als FILE-Handle -- alles Weitere sind
die Datei-Builtins, die es ohnehin schon gibt (READLINE, READALL$, ENDOFFILE,
READ_BYTES), samt Kodierungsangabe aus Punkt 3.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/stdin.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import pytest


FILTER = ("DIM f AS FILE\n"
          "DIM z AS STRING\n"
          "f = STDIN()\n"
          "WHILE NOT ENDOFFILE(f)\n"
          "    z = READLINE(f)\n"
          "    PRINT UPPER$(z)\n"
          "WEND\n")


# ------------------------------------------------------- EIN Handle


def test_zweite_kodierung_ist_ein_fehler(run_gb_roh):
    """Still wirkungslos waere schlimmer als ein Fehler."""
    code, _, err = run_gb_roh("DIM f AS FILE\n"
                              "DIM g AS FILE\n"
                              "f = STDIN()\n"
                              'g = STDIN("cp1252")\n', eingabe="a\n")
    assert code != 0
    assert "schon mit einer anderen Kodierung" in err


# ------------------------------------------------------- Kodierung + Bytes


# ------------------------------------------------------------- Grenzen


def test_tell_ebenso(run_gb_roh):
    code, _, err = run_gb_roh("DIM f AS FILE\n"
                              "f = STDIN()\n"
                              "PRINT TELL(f)\n", eingabe="a\n")
    assert code != 0
    assert "keine Position" in err


def test_unbekannte_kodierung(run_gb_roh):
    code, _, err = run_gb_roh('DIM f AS FILE\nf = STDIN("klingonisch")\n', eingabe="")
    assert code != 0
    assert "kenne ich nicht" in err


# --------------------------------------------------- eine echte kleine Kette
