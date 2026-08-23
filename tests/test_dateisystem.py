"""Dateisystem-Lücken aus Punkt 7 des Allzweck-Audits.

`MKDIR` gab es, ein Gegenstueck zum Loeschen nicht. Ebenso fehlten
Zeitstempel (die Grundlage jeder Sicherung und jedes "was ist neu"),
rekursives Auflisten, Namensmuster und ein Temp-Ordner.

Das Namensmuster ist gegen **Pythons `fnmatch`** geprueft, nicht gegen sich
selbst -- ein selbstgeschriebener Vergleicher, der nur mit sich
uebereinstimmt, ist wertlos.
"""
import fnmatch
import os

import pytest

from drachenhauch.errors import DHRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


@pytest.fixture
def baum(tmp_path):
    """Ein kleiner Dateibaum zum Durchsuchen -- in einem UNTERordner.

    Die `run_gb`-Fixture legt ihre .dh-Datei in `base` ab; laege der Baum
    dort, zaehlte jedes `DIRLIST(".")` sie mit.
    """
    wurzel = tmp_path / "baum"
    wurzel.mkdir()
    (wurzel / "a.csv").write_text("1", encoding="utf-8")
    (wurzel / "b.CSV").write_text("2", encoding="utf-8")
    (wurzel / "x.txt").write_text("3", encoding="utf-8")
    unter = wurzel / "unter"
    unter.mkdir()
    (unter / "tief.csv").write_text("4", encoding="utf-8")
    (unter / "auch.txt").write_text("5", encoding="utf-8")
    return tmp_path


# ------------------------------------------------------------- Muster
def test_dirlist_mit_muster(baum, run_gb):
    out = run_gb('DIM d AS ARRAY OF STRING\n'
                 'd = DIRLIST("baum", "*.csv")\n'
                 'PRINT JOIN$(d, ",")\n', base=baum)
    assert _lines(out) == ["a.csv,b.CSV"]


def test_muster_ignoriert_gross_klein(baum, run_gb):
    """Windows unterscheidet bei Dateinamen nicht, Linux schon -- waere das
    hier plattformabhaengig, liefe dasselbe Programm auf zwei Rechnern
    unterschiedlich."""
    out = run_gb('DIM d AS ARRAY OF STRING\n'
                 'd = DIRLIST("baum", "*.CSV")\n'
                 'PRINT LEN(d)\n', base=baum)
    assert _lines(out) == ["2"]


@pytest.mark.parametrize("muster", ["*.csv", "a*", "?.csv", "*", "*.*", "*a*"])
def test_muster_deckt_sich_mit_fnmatch(muster, baum, run_gb):
    out = run_gb('DIM d AS ARRAY OF STRING\n'
                 f'd = DIRLIST("baum", "{muster}")\n'
                 'PRINT JOIN$(d, "|")\n', base=baum)
    dhrt = sorted(x for x in _lines(out)[0].split("|") if x) if _lines(out) else []
    namen = sorted(os.listdir(baum / "baum"))
    py = sorted(n for n in namen if fnmatch.fnmatch(n.lower(), muster.lower()))
    assert dhrt == py


def test_ohne_muster_kommt_alles(baum, run_gb):
    out = run_gb('DIM d AS ARRAY OF STRING\n'
                 'd = DIRLIST("baum")\n'
                 'PRINT LEN(d)\n', base=baum)
    assert _lines(out) == ["4"]        # 3 Dateien + der Ordner "unter"


# ------------------------------------------------------------ rekursiv
def test_rekursiv_findet_auch_tiefer(baum, run_gb):
    out = run_gb('DIM d AS ARRAY OF STRING\n'
                 'd = DIRLIST_REC("baum", "*.csv")\n'
                 'PRINT JOIN$(d, ",")\n', base=baum)
    assert _lines(out) == ["a.csv,b.CSV,unter/tief.csv"]


def test_rekursiv_trennt_immer_mit_schraegstrich(baum, run_gb):
    """Auch unter Windows -- ein Programm, das die Liste weiterverarbeitet
    oder speichert, soll auf beiden Systemen dieselben Zeichenketten sehen."""
    out = run_gb('DIM d AS ARRAY OF STRING\n'
                 'd = DIRLIST_REC("baum", "tief.csv")\n'
                 'PRINT d[0]\n', base=baum)
    assert _lines(out) == ["unter/tief.csv"]


def test_rekursiv_liefert_nur_dateien(baum, run_gb):
    out = run_gb('DIM d AS ARRAY OF STRING\n'
                 'd = DIRLIST_REC("baum")\n'
                 'PRINT LEN(d)\n', base=baum)
    assert _lines(out) == ["5"]        # der Ordner "unter" zaehlt nicht mit


# -------------------------------------------------------------- RMDIR
def test_rmdir_loescht_ein_leeres_verzeichnis(tmp_path, run_gb):
    (tmp_path / "leer").mkdir()
    run_gb('RMDIR("leer")\n', base=tmp_path)
    assert not (tmp_path / "leer").exists()


def test_rmdir_verweigert_ein_volles_und_sagt_wie(tmp_path, run_gb):
    """Ein Aufruf, der versehentlich einen ganzen Baum loescht, ist der
    teuerste Tippfehler, den ein Dateibefehl anrichten kann."""
    (tmp_path / "voll" / "drin").mkdir(parents=True)
    with pytest.raises(DHRuntimeError) as e:
        run_gb('RMDIR("voll")\n', base=tmp_path)
    assert "RMDIR(pfad, TRUE)" in str(e.value)
    assert (tmp_path / "voll").exists()


def test_rmdir_mit_flag_loescht_alles(tmp_path, run_gb):
    (tmp_path / "voll" / "drin").mkdir(parents=True)
    (tmp_path / "voll" / "drin" / "x.txt").write_text("x", encoding="utf-8")
    run_gb('RMDIR("voll", TRUE)\n', base=tmp_path)
    assert not (tmp_path / "voll").exists()


def test_rmdir_nimmt_nur_boolean(tmp_path, run_gb):
    (tmp_path / "leer").mkdir()
    with pytest.raises(DHRuntimeError) as e:
        run_gb('RMDIR("leer", 1)\n', base=tmp_path)
    assert "BOOLEAN" in str(e.value)


# ----------------------------------------------------------- FILETIME
def test_filetime_passt_zur_zeitrechnung_des_moduls(tmp_path, run_gb):
    """Der Grund, warum es FILETIME gibt: `wie alt ist die Datei`. Das geht
    nur, wenn beide Seiten dieselbe Zeitrechnung benutzen."""
    out = run_gb('IMPORT "zeit"\n'
                 'WRITEALL("neu.txt", "x")\n'
                 "DIM alter AS INTEGER\n"
                 'alter = ZEIT_JETZT() - FILETIME("neu.txt")\n'
                 "PRINT alter >= 0 AND alter < 120\n", base=tmp_path)
    assert _lines(out) == ["TRUE"]


def test_filetime_unterscheidet_alt_und_neu(tmp_path, run_gb):
    """Die Grundlage jeder Sicherung: welche Datei ist neuer?"""
    alt = tmp_path / "alt.txt"
    alt.write_text("a", encoding="utf-8")
    os.utime(alt, (0, 0))              # 1970
    out = run_gb('WRITEALL("neu.txt", "b")\n'
                 'PRINT FILETIME("neu.txt") > FILETIME("alt.txt")\n', base=tmp_path)
    assert _lines(out) == ["TRUE"]


def test_filetime_meldet_die_fehlende_datei(tmp_path, run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('PRINT FILETIME("gibtsnicht.txt")\n', base=tmp_path)
    assert "gibtsnicht.txt" in str(e.value)


# ------------------------------------------------------------ Temp
def test_tempdir_zeigt_auf_ein_verzeichnis(tmp_path, run_gb):
    out = run_gb("PRINT DIREXISTS(TEMPDIR$())\n", base=tmp_path)
    assert _lines(out) == ["TRUE"]


def test_tempfile_ist_da_und_leer(tmp_path, run_gb):
    """Angelegt wird sie sofort -- sonst koennte zwischen 'Name ausgedacht'
    und 'Datei geschrieben' ein zweiter Lauf denselben Namen bekommen."""
    out = run_gb("DIM t AS STRING\n"
                 't = TEMPFILE$("dhtest", ".txt")\n'
                 "PRINT FILEEXISTS(t)\n"
                 "PRINT FILESIZE(t)\n"
                 'PRINT ENDSWITH(t, ".txt")\n'
                 "DELETEFILE(t)\n", base=tmp_path)
    assert _lines(out) == ["TRUE", "0", "TRUE"]


def test_tempfile_zweimal_gibt_zwei_namen(tmp_path, run_gb):
    out = run_gb("DIM a AS STRING\n"
                 "DIM b AS STRING\n"
                 "a = TEMPFILE$()\n"
                 "b = TEMPFILE$()\n"
                 "PRINT a <> b\n"
                 "DELETEFILE(a)\n"
                 "DELETEFILE(b)\n", base=tmp_path)
    assert _lines(out) == ["TRUE"]


def test_tempfile_liegt_im_tempordner(tmp_path, run_gb):
    out = run_gb("DIM t AS STRING\n"
                 "t = TEMPFILE$()\n"
                 "PRINT STARTSWITH(t, TEMPDIR$())\n"
                 "DELETEFILE(t)\n", base=tmp_path)
    assert _lines(out) == ["TRUE"]
