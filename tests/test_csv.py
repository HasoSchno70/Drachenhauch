"""WP J -- CSV lesen und schreiben.

Vorher war CSV Handarbeit mit `SPLIT$`, und die geht bei der ersten Zelle
schief, die das Trennzeichen enthaelt -- ohne es zu melden:

    Mueller;"Berlin; Mitte";42      SPLIT$(";") liefert vier Felder statt drei

Die Zerlegung selbst pruefen die Rust-`#[test]`s in `src/csv.rs` (Anfuehrungen,
CRLF, leere Felder, kaputte Dateien). Hier geht es um den Weg durch die
Sprache: 2D-ARRAY, Trennzeichen-Argument, Datei-Rundreise.
"""
import pytest

Q = chr(34)
NL = chr(10)


def gb(text: str) -> str:
    """Python-String als GB-AUSDRUCK.

    Ein GB-Literal kann weder Anfuehrungszeichen noch Zeilenumbruch enthalten
    -- beides muss als `CHR$(...)` danebenstehen. (Die erste Fassung dieser
    Tests hat das vergessen und GB-Quelltext erzeugt, der mitten im Literal
    aufhoerte; der Parse-Fehler zeigte auf eine ganz andere Stelle.)
    """
    teile, puffer = [], ""
    for c in text:
        if c in (Q, NL):
            if puffer:
                teile.append(Q + puffer + Q)
                puffer = ""
            teile.append("CHR$(" + str(ord(c)) + ")")
        else:
            puffer += c
    if puffer:
        teile.append(Q + puffer + Q)
    return " + ".join(teile) if teile else Q + Q


def test_gb_helfer_baut_gueltigen_ausdruck(run_gb):
    """Erst den Helfer pruefen -- sonst misst der Rest den Helfer mit."""
    assert run_gb("PRINT " + gb('a"b') + "\n").strip() == 'a"b'
    assert run_gb("PRINT " + gb("x" + NL + "y") + "\n").split("\n")[:2] == ["x", "y"]


def test_trenner_im_feld_ueberlebt(run_gb):
    """Der Fall, an dem SPLIT$ scheitert."""
    quelle = "a;" + Q + "b; mit Semikolon" + Q + ";c"
    out = run_gb(
        "DIM t AS ARRAY OF STRING\n"
        "t = CSV_PARSE(" + gb(quelle) + ', ";")\n'
        "PRINT DIMSIZE(t, 1)\n"
        "PRINT t[0, 1]\n")
    assert out.split("\n")[:2] == ["3", "b; mit Semikolon"]


def test_zeilen_und_spalten(run_gb):
    quelle = "Name;Ort" + NL + "Mueller;Berlin" + NL
    out = run_gb(
        "DIM t AS ARRAY OF STRING\n"
        "t = CSV_PARSE(" + gb(quelle) + ', ";")\n'
        'PRINT STR$(DIMSIZE(t, 0)) + "x" + STR$(DIMSIZE(t, 1))\n')
    assert out.strip() == "2x2", out


def test_komma_ist_die_vorgabe(run_gb):
    out = run_gb(
        "DIM t AS ARRAY OF STRING\n"
        't = CSV_PARSE("a,b,c")\n'
        "PRINT t[0, 2]\n")
    assert out.strip() == "c"


def test_ungleich_lange_zeilen_werden_aufgefuellt(run_gb):
    """Ein GB-Array muss rechteckig sein. Aufgefuellt wird auf die BREITESTE
    Zeile -- abschneiden hiesse, Daten stillschweigend wegzuwerfen."""
    out = run_gb(
        "DIM t AS ARRAY OF STRING\n"
        "t = CSV_PARSE(" + gb("a;b;c" + NL + "d") + ', ";")\n'
        'PRINT STR$(DIMSIZE(t, 1)) + "|" + t[1, 0] + "|" + t[1, 2] + "|"\n')
    assert out.strip() == "3|d||"


def test_row_setzt_nur_noetige_anfuehrungen(run_gb):
    """Unnoetige Anfuehrungszeichen sind erlaubt, machen die Datei aber
    unleserlich und den Diff groesser."""
    out = run_gb(
        "DIM f AS ARRAY OF STRING\n"
        'f = SPLIT$("Name|Ort|mit ; drin", "|")\n'
        'PRINT CSV_ROW$(f, ";")\n')
    assert out.strip() == "Name;Ort;" + Q + "mit ; drin" + Q


def test_rundreise_ueber_datei(run_gb, tmp_path):
    pfad = str(tmp_path / "t.csv").replace("\\", "/")
    quelle = "a;" + Q + "b; und mehr" + Q + ";c" + NL + "d;e;f"
    out = run_gb(
        "DIM t AS ARRAY OF STRING\n"
        "DIM z AS ARRAY OF STRING\n"
        "t = CSV_PARSE(" + gb(quelle) + ', ";")\n'
        'CSV_SAVE("' + pfad + '", t, ";")\n'
        'z = CSV_LOAD("' + pfad + '", ";")\n'
        "PRINT z[0, 1]\n"
        "PRINT z[1, 2]\n")
    assert out.split("\n")[:2] == ["b; und mehr", "f"]


def test_format_und_parse_sind_umkehrbar(run_gb):
    quelle = "x;" + Q + "sagt " + Q + Q + "hallo" + Q + Q + Q + ";z"
    out = run_gb(
        "DIM t AS ARRAY OF STRING\n"
        "DIM z AS ARRAY OF STRING\n"
        "t = CSV_PARSE(" + gb(quelle) + ', ";")\n'
        'z = CSV_PARSE(CSV_FORMAT$(t, ";"), ";")\n'
        "PRINT z[0, 1]\n")
    assert out.strip() == 'sagt "hallo"'


def test_trennzeichen_muss_ein_zeichen_sein(run_gb):
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError, match="genau ein Zeichen"):
        run_gb('PRINT CSV_ROW$(SPLIT$("a|b", "|"), ";;")\n')


def test_fehlende_datei_meldet_den_pfad(run_gb):
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError, match="gibtsnicht"):
        run_gb("DIM t AS ARRAY OF STRING\n"
               't = CSV_LOAD("gibtsnicht_xyz.csv")\n')
