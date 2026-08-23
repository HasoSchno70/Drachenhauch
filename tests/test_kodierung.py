"""Textdateien, die nicht UTF-8 sind (Punkt 3 aus docs/allzweck-audit-2.md).

Der Anlass: eine aus Excel exportierte CSV ist auf einem deutschen Windows
**cp1252** -- und war damit gar nicht lesbar (`stream did not contain valid
UTF-8`). Die haeufigste Herkunft von Daten, die jemand auswerten will, war
also die eine, die nicht durch die Tuer passte.

Die Tabelle in `kodierung.rs` ist von Hand geschrieben. Sie wird hier gegen
**Pythons eigene Codecs** geprueft, nicht gegen sich selbst -- eine Tabelle,
die nur mit sich uebereinstimmt, ist wertlos.
"""
import pytest

from drachenhauch.errors import DHRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _schreibe(pfad, text, kodierung):
    pfad.write_bytes(text.encode(kodierung))


# ------------------------------------------------- die Tabelle selbst
@pytest.mark.parametrize("byte", range(0x80, 0xA0))
def test_cp1252_tabelle_stimmt_mit_python_ueberein(byte, tmp_path, run_gb):
    """Die 32 Bytes, in denen sich cp1252 von latin1 unterscheidet -- jedes
    einzeln gegen Pythons Codec.

    Fuenf davon (0x81, 0x8D, 0x8F, 0x90, 0x9D) belegt Windows-1252 gar nicht.
    Python lehnt sie ab, wir bilden sie (wie die WHATWG-Norm, der jeder
    Browser folgt) auf ihren eigenen Codepunkt ab: wer eine alte Datei
    einliest, will sie lesen und nicht an einem Steuerzeichen scheitern, das
    ohnehin niemand gemeint hat.
    """
    f = tmp_path / "b.txt"
    f.write_bytes(bytes([byte]))
    out = run_gb('DIM t AS ARRAY OF STRING\n'
                 't = READLINES("b.txt", "cp1252")\n'
                 "PRINT ASC(t[0])\n", base=tmp_path)
    try:
        erwartet = ord(bytes([byte]).decode("cp1252"))
    except UnicodeDecodeError:
        erwartet = byte          # unbelegt -> zeigt auf sich selbst
    assert _lines(out) == [str(erwartet)]


def test_latin1_ist_die_identitaet_auf_bytes(tmp_path, run_gb):
    f = tmp_path / "b.txt"
    f.write_bytes(bytes([0xE4, 0xF6, 0xFC, 0xDF]))       # ae oe ue ss
    out = run_gb('DIM t AS ARRAY OF STRING\n'
                 't = READLINES("b.txt", "latin1")\n'
                 "PRINT t[0]\n", base=tmp_path)
    assert _lines(out) == ["äöüß"]


# ------------------------------------------------------ der eigentliche Fall
def test_excel_csv_laesst_sich_lesen(tmp_path, run_gb):
    """Der Fall, um den es geht."""
    _schreibe(tmp_path / "kunden.csv",
              "Name;Ort;Umsatz\nMüller;Köln;1250\nSchröder;Münster;830\n", "cp1252")
    out = run_gb('DIM t AS ARRAY OF STRING\n'
                 't = CSV_LOAD("kunden.csv", ";", "cp1252")\n'
                 "PRINT DIMSIZE(t, 0)\n"
                 "PRINT t[1, 0]\n"
                 "PRINT t[2, 1]\n", base=tmp_path)
    assert _lines(out) == ["3", "Müller", "Münster"]


def test_ohne_angabe_erklaert_die_meldung_den_ausweg(tmp_path, run_gb):
    """Vorher stand da wortwoertlich 'stream did not contain valid UTF-8'."""
    _schreibe(tmp_path / "k.csv", "Name\nMüller\n", "cp1252")
    with pytest.raises(DHRuntimeError) as e:
        run_gb('DIM t AS ARRAY OF STRING\nt = READLINES("k.csv")\n', base=tmp_path)
    msg = str(e.value)
    assert "kein UTF-8" in msg
    assert "Zeile 2" in msg          # die Stelle, nicht nur "irgendwo"
    assert "cp1252" in msg           # der Ausweg
    assert "k.csv" in msg            # welche Datei


def test_unbekannte_kodierung_zaehlt_die_moeglichen_auf(tmp_path, run_gb):
    (tmp_path / "k.txt").write_text("a\n", encoding="utf-8")
    with pytest.raises(DHRuntimeError) as e:
        run_gb('DIM t AS ARRAY OF STRING\n'
               't = READLINES("k.txt", "klingonisch")\n', base=tmp_path)
    assert "kenne ich nicht" in str(e.value)
    assert "latin1" in str(e.value)


@pytest.mark.parametrize("name", ["cp1252", "CP-1252", "windows-1252", "ANSI"])
def test_namen_sind_nachsichtig(name, tmp_path, run_gb):
    _schreibe(tmp_path / "k.txt", "Grüße\n", "cp1252")
    out = run_gb('DIM t AS ARRAY OF STRING\n'
                 f't = READLINES("k.txt", "{name}")\n'
                 "PRINT t[0]\n", base=tmp_path)
    assert _lines(out) == ["Grüße"]


# ------------------------------------------------------------- schreiben
def test_schreiben_erzeugt_echte_cp1252_bytes(tmp_path, run_gb):
    run_gb('WRITEALL("raus.txt", "Grüße;Köln", "cp1252")\n', base=tmp_path)
    assert (tmp_path / "raus.txt").read_bytes() == "Grüße;Köln".encode("cp1252")


def test_hin_und_zurueck(tmp_path, run_gb):
    out = run_gb('WRITEALL("r.txt", "Straße in Köln", "cp1252")\n'
                 "DIM t AS ARRAY OF STRING\n"
                 't = READLINES("r.txt", "cp1252")\n'
                 "PRINT t[0]\n", base=tmp_path)
    assert _lines(out) == ["Straße in Köln"]


def test_anhaengen_kann_die_kodierung_auch(tmp_path, run_gb):
    run_gb('WRITEALL("a.txt", "Köln" + CHR$(10), "cp1252")\n'
           'APPENDFILE("a.txt", "Münster" + CHR$(10), "cp1252")\n', base=tmp_path)
    assert (tmp_path / "a.txt").read_bytes() == "Köln\nMünster\n".encode("cp1252")


def test_csv_save_mit_kodierung(tmp_path, run_gb):
    run_gb("DIM t AS ARRAY OF STRING\n"
           't = CSV_PARSE("Müller;Köln", ";")\n'
           'CSV_SAVE("k.csv", t, ";", "cp1252")\n', base=tmp_path)
    assert (tmp_path / "k.csv").read_bytes().startswith("Müller;Köln".encode("cp1252"))


def test_zeichen_das_es_nicht_gibt_ist_ein_fehler(tmp_path, run_gb):
    """Kein stilles Fragezeichen: eine Rechnung, in der aus dem Euro-Zeichen
    unbemerkt ein '?' wird, ist schlimmer als eine, die gar nicht entsteht."""
    with pytest.raises(DHRuntimeError) as e:
        run_gb('WRITEALL("r.txt", "Preis 5 " + CHR$(8364), "latin1")\n', base=tmp_path)
    assert "U+20AC" in str(e.value)
    assert "utf8" in str(e.value)


def test_euro_geht_in_cp1252(tmp_path, run_gb):
    """cp1252 hat das Euro-Zeichen (0x80), latin1 nicht -- genau dafuer gibt
    es beide."""
    run_gb('WRITEALL("r.txt", CHR$(8364), "cp1252")\n', base=tmp_path)
    assert (tmp_path / "r.txt").read_bytes() == b"\x80"


# ------------------------------------------------- zeilenweise (grosse Dateien)
def test_openfile_liest_zeilenweise_mit_kodierung(tmp_path, run_gb):
    _schreibe(tmp_path / "gross.txt", "Müller\nSchröder\nWeiß\n", "cp1252")
    out = run_gb("DIM f AS FILE\n"
                 "DIM z AS STRING\n"
                 'f = OPENFILE("gross.txt", "r", "cp1252")\n'
                 "WHILE NOT ENDOFFILE(f)\n"
                 "    z = READLINE(f)\n"
                 "    PRINT z\n"
                 "WEND\n"
                 "CLOSEFILE(f)\n", base=tmp_path)
    assert _lines(out) == ["Müller", "Schröder", "Weiß"]


def test_openfile_schreibt_mit_kodierung(tmp_path, run_gb):
    run_gb("DIM f AS FILE\n"
           'f = OPENFILE("w.txt", "w", "cp1252")\n'
           'WRITELINE(f, "Grüße")\n'
           "CLOSEFILE(f)\n", base=tmp_path)
    assert (tmp_path / "w.txt").read_bytes() == "Grüße\n".encode("cp1252")


def test_readall_auf_dem_handle_kann_es_auch(tmp_path, run_gb):
    _schreibe(tmp_path / "g.txt", "Köln", "cp1252")
    out = run_gb("DIM f AS FILE\n"
                 'f = OPENFILE("g.txt", "r", "cp1252")\n'
                 "PRINT READALL$(f)\n"
                 "CLOSEFILE(f)\n", base=tmp_path)
    assert _lines(out) == ["Köln"]


# ------------------------------------------------------------------ BOM
def test_bom_faellt_weg(tmp_path, run_gb):
    """Excel schreibt ihn -- sonst hiesse die erste Spalte fuer immer
    '\\ufeffName'."""
    (tmp_path / "b.csv").write_bytes("﻿Name;Ort\n".encode("utf-8"))
    out = run_gb('DIM t AS ARRAY OF STRING\n'
                 't = READLINES("b.csv")\n'
                 "PRINT LEFT$(t[0], 4)\n", base=tmp_path)
    assert _lines(out) == ["Name"]


def test_bom_faellt_auch_beim_zeilenweisen_lesen_weg(tmp_path, run_gb):
    (tmp_path / "b.csv").write_bytes("﻿Name\nOrt\n".encode("utf-8"))
    out = run_gb("DIM f AS FILE\n"
                 'f = OPENFILE("b.csv", "r")\n'
                 "PRINT LEFT$(READLINE(f), 4)\n"
                 "CLOSEFILE(f)\n", base=tmp_path)
    assert _lines(out) == ["Name"]


# ------------------------------------------------------------ unveraendert
def test_utf8_bleibt_die_vorgabe(tmp_path, run_gb):
    """Kein bestehendes Programm aendert sein Verhalten."""
    (tmp_path / "u.txt").write_text("Grüße 😀\n", encoding="utf-8")
    out = run_gb('DIM t AS ARRAY OF STRING\n'
                 't = READLINES("u.txt")\n'
                 "PRINT t[0]\n"
                 "PRINT LEN(t[0])\n", base=tmp_path)
    assert _lines(out) == ["Grüße 😀", "7"]
