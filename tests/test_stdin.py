"""Die Standardeingabe (Punkt 4 aus docs/allzweck-audit-2.md).

WP A hat „Werkzeug in einer Kette" zum Ziel erklaert -- der halbe Weg fehlte
aber: ein Programm konnte lesen, was ihm als ARGUMENT gegeben wurde, aber
nicht, was ihm GEREICHT wurde. `dir | meinwerkzeug | sort` war damit nicht
schreibbar.

`STDIN()` liefert die Standardeingabe als FILE-Handle -- alles Weitere sind
die Datei-Builtins, die es ohnehin schon gibt (READLINE, READALL$, ENDOFFILE,
READ_BYTES), samt Kodierungsangabe aus Punkt 3.
"""
import pytest


FILTER = ("DIM f AS FILE\n"
          "DIM z AS STRING\n"
          "f = STDIN()\n"
          "WHILE NOT ENDOFFILE(f)\n"
          "    z = READLINE(f)\n"
          "    PRINT UPPER$(z)\n"
          "WEND\n")


def test_zeilen_von_der_standardeingabe(run_gb_roh):
    code, out, err = run_gb_roh(FILTER, eingabe="birne\napfel\n")
    assert code == 0, err
    assert out.split("\n")[:2] == ["BIRNE", "APFEL"]


def test_leere_eingabe_laeuft_null_mal(run_gb_roh):
    """Der Fall, an dem eine INPUT-Schleife frueher ewig lief."""
    code, out, err = run_gb_roh(FILTER + 'PRINT "fertig"\n', eingabe="")
    assert code == 0, err
    assert out.strip() == "fertig"


def test_letzte_zeile_ohne_zeilenumbruch(run_gb_roh):
    """Was `echo -n` liefert -- die Zeile darf nicht verlorengehen."""
    code, out, _ = run_gb_roh(FILTER, eingabe="eins\nzwei")
    assert out.split("\n")[:2] == ["EINS", "ZWEI"]


def test_alles_auf_einmal(run_gb_roh):
    code, out, _ = run_gb_roh("DIM f AS FILE\n"
                              "f = STDIN()\n"
                              "PRINT LEN(READALL$(f))\n", eingabe="abcd")
    assert out.strip() == "4"


def test_nichts_geht_an_stdout_verloren(run_gb_roh):
    """Der Punkt einer Kette: was das Programm PRINTet, ist die Nutzlast --
    kein erfundener Prompt, keine Meldung dazwischen."""
    code, out, _ = run_gb_roh(FILTER, eingabe="a\nb\nc\n")
    assert out == "A\nB\nC\n"


# ------------------------------------------------------- EIN Handle
def test_zweimal_holen_liefert_dasselbe_handle(run_gb_roh):
    """Zwei eigene Handles waeren zwei Lesepuffer auf derselben Leitung: das
    eine liest voraus, dem anderen fehlen die Zeilen."""
    code, out, err = run_gb_roh("DIM f AS FILE\n"
                                "DIM g AS FILE\n"
                                "f = STDIN()\n"
                                "g = STDIN()\n"
                                "PRINT READLINE(f)\n"
                                "PRINT READLINE(g)\n", eingabe="eins\nzwei\n")
    assert code == 0, err
    assert out.split("\n")[:2] == ["eins", "zwei"]


def test_zweite_kodierung_ist_ein_fehler(run_gb_roh):
    """Still wirkungslos waere schlimmer als ein Fehler."""
    code, _, err = run_gb_roh("DIM f AS FILE\n"
                              "DIM g AS FILE\n"
                              "f = STDIN()\n"
                              'g = STDIN("cp1252")\n', eingabe="a\n")
    assert code != 0
    assert "schon mit einer anderen Kodierung" in err


def test_input_und_stdin_lassen_sich_mischen(run_gb_roh):
    """Beide haengen am selben prozessweiten Puffer -- sonst verschluckte
    das eine die Zeilen des anderen."""
    code, out, err = run_gb_roh('DIM name AS STRING\n'
                                "DIM f AS FILE\n"
                                "DIM z AS STRING\n"
                                'INPUT "Name: ", name\n'
                                'PRINT "[" + name + "]"\n'
                                "f = STDIN()\n"
                                "WHILE NOT ENDOFFILE(f)\n"
                                "    z = READLINE(f)\n"
                                '    PRINT "rest: " + z\n'
                                "WEND\n", eingabe="Anna\nzeile1\nzeile2\n")
    assert code == 0, err
    assert "[Anna]" in out
    assert "rest: zeile1" in out
    assert "rest: zeile2" in out


# ------------------------------------------------------- Kodierung + Bytes
def test_kodierung_gilt_auch_hier(run_gb_roh):
    code, out, err = run_gb_roh('DIM f AS FILE\n'
                                'f = STDIN("cp1252")\n'
                                "PRINT READLINE(f)\n",
                                eingabe="Grüße\n".encode("cp1252"))
    assert code == 0, err
    assert out.strip() == "Grüße"


def test_rohe_bytes_von_der_standardeingabe(run_gb_roh):
    """Damit laesst sich auch ein Binaerstrom durchreichen."""
    code, out, err = run_gb_roh("DIM f AS FILE\n"
                                "DIM b AS BUFFER\n"
                                "f = STDIN()\n"
                                "b = READ_BYTES(f, 4)\n"
                                "PRINT BUFFER_TO_HEX$(b)\n",
                                eingabe=b"\x01\x02\x03\xff")
    assert code == 0, err
    assert out.strip().lower().replace(" ", "") == "010203ff"


# ------------------------------------------------------------- Grenzen
def test_seek_sagt_warum_es_nicht_geht(run_gb_roh):
    code, _, err = run_gb_roh("DIM f AS FILE\n"
                              "f = STDIN()\n"
                              "SEEK(f, 0)\n", eingabe="a\n")
    assert code != 0
    assert "zurueckspulen" in err


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
def test_zaehlen_wie_ein_wc(run_gb_roh):
    """Das Muster, um das es geht: Eingabe verdichten, Ergebnis nach stdout."""
    quelle = ("DIM f AS FILE\n"
              "DIM zeilen AS INTEGER\n"
              "DIM zeichen AS INTEGER\n"
              "DIM z AS STRING\n"
              "f = STDIN()\n"
              "WHILE NOT ENDOFFILE(f)\n"
              "    z = READLINE(f)\n"
              "    zeilen = zeilen + 1\n"
              "    zeichen = zeichen + LEN(z)\n"
              "WEND\n"
              "PRINT STR$(zeilen) + " + '"' + " " + '"' + " + STR$(zeichen)\n")
    code, out, err = run_gb_roh(quelle, eingabe="abc\nde\n")
    assert code == 0, err
    assert out.strip() == "2 5"
