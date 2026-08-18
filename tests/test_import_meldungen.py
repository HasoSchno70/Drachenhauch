"""WP I.4 -- Meldungen, die auf die richtige Datei und Zeile zeigen.

`IMPORT` fuegt Text ein; danach zeigte jede Zeilennummer in die GEMERGTE
Quelle. Eine Datei mit zwei Zeilen bekam so ein "datei.dh:6", und eine
Namenskollision zwischen zwei Bibliotheken nannte weder Zeile noch Datei.

Belegt die Punkte (a) und (d) aus `docs/entwurf-namensraeume.md`.
"""
import pytest


def _lauf(dhrt_pfad, tmp_path, dateien: dict, haupt="main.dh"):
    """Schreibt mehrere .dh-Dateien und laesst `haupt` laufen. Liefert stderr."""
    import subprocess
    for name, inhalt in dateien.items():
        (tmp_path / name).write_text(inhalt, encoding="utf-8")
    r = subprocess.run([dhrt_pfad, "run", str(tmp_path / haupt)],
                       capture_output=True, text=True, encoding="utf-8", timeout=60)
    return (r.stderr or "").replace("\r\n", "\n")


# ------------------------------------------------------- (a) Kollisionen

def test_funktions_kollision_nennt_beide_dateien(dhrt_pfad, tmp_path):
    err = _lauf(dhrt_pfad, tmp_path, {
        "a.dh": "FUNCTION Hilf() AS INTEGER\n    RETURN 1\nEND FUNCTION\n",
        "b.dh": "FUNCTION Hilf() AS INTEGER\n    RETURN 2\nEND FUNCTION\n",
        "main.dh": 'IMPORT "a.dh"\nIMPORT "b.dh"\nPRINT Hilf()\n',
    })
    # Die zweite Deklaration ist die Fundstelle, die erste steht im Text.
    assert "b.dh:1" in err, err
    assert "a.dh:1" in err, err
    assert "hilf" in err.lower()


def test_klassen_kollision_nennt_beide_dateien(dhrt_pfad, tmp_path):
    err = _lauf(dhrt_pfad, tmp_path, {
        "k1.dh": "CLASS Punkt\n    DIM x AS INTEGER\nEND CLASS\n",
        "k2.dh": "CLASS Punkt\n    DIM y AS INTEGER\nEND CLASS\n",
        "main.dh": 'IMPORT "k1.dh"\nIMPORT "k2.dh"\nPRINT 1\n',
    })
    assert "k2.dh:1" in err and "k1.dh:1" in err, err


def test_kollision_ohne_import_nennt_jetzt_auch_eine_zeile(dhrt_pfad, tmp_path):
    """Auch ohne IMPORT eine Verbesserung: vorher stand da gar keine Zeile."""
    err = _lauf(dhrt_pfad, tmp_path, {
        "main.dh": ("FUNCTION F() AS INTEGER\n    RETURN 1\nEND FUNCTION\n"
                    "FUNCTION F() AS INTEGER\n    RETURN 2\nEND FUNCTION\n"),
    })
    assert "main.dh:4" in err, err      # zweite Deklaration
    assert "main.dh:1" in err, err      # erste


def test_kollision_ueber_zwei_ebenen(dhrt_pfad, tmp_path):
    """b.dh zieht c.dh -- die Meldung muss auf c.dh zeigen, nicht auf b.dh."""
    err = _lauf(dhrt_pfad, tmp_path, {
        "a.dh": "FUNCTION Doppelt() AS INTEGER\n    RETURN 1\nEND FUNCTION\n",
        "c.dh": "FUNCTION Doppelt() AS INTEGER\n    RETURN 2\nEND FUNCTION\n",
        "b.dh": 'IMPORT "c.dh"\n',
        "main.dh": 'IMPORT "a.dh"\nIMPORT "b.dh"\nPRINT 1\n',
    })
    assert "c.dh:1" in err and "a.dh:1" in err, err


# ------------------------------------------- (d) Zeilen zeigen richtig

def test_warnung_zeigt_in_die_datei_des_nutzers(dhrt_pfad, tmp_path):
    """Der Ausgangsbefund: eine Datei mit zwei Zeilen bekam 'main.dh:6'."""
    err = _lauf(dhrt_pfad, tmp_path, {
        "d.dh": "DIM zaehler AS INTEGER\nzaehler = 1\n",
        "main.dh": 'IMPORT "d.dh"\nDIM zaehler AS STRING\n',
    })
    assert "main.dh:2:" in err, err     # die eigene, zweite Zeile
    assert "d.dh:1" in err, err         # und die andere Stelle mit Datei
    assert "Zeile 2" not in err, err    # keine nackte gemergte Zahl mehr


def test_parse_fehler_im_modul_zeigt_auf_das_modul(dhrt_pfad, tmp_path):
    err = _lauf(dhrt_pfad, tmp_path, {
        "kaputt.dh": "PRINT 1\nDIM x AS\nPRINT 2\n",
        "main.dh": 'PRINT "eins"\nIMPORT "kaputt.dh"\nPRINT "zwei"\n',
    })
    assert "kaputt.dh:2" in err, err


def test_compile_fehler_im_modul_zeigt_auf_das_modul(dhrt_pfad, tmp_path):
    err = _lauf(dhrt_pfad, tmp_path, {
        "m.dh": "FUNCTION F() AS INTEGER\n    RETURN GIBTESNICHT(1)\nEND FUNCTION\n",
        "main.dh": 'IMPORT "m.dh"\nPRINT F()\n',
    })
    # Die Warnung ueber das unbekannte Builtin gehoert in m.dh, Zeile 2.
    assert "m.dh:2" in err, err


def test_ohne_import_bleibt_die_zeile_wie_gehabt(dhrt_pfad, tmp_path):
    err = _lauf(dhrt_pfad, tmp_path, {
        "main.dh": 'PRINT "a"\nPRINT GIBTESNICHT(1)\n',
    })
    assert "main.dh:2" in err, err


def test_mehrere_module_werden_richtig_zugeordnet(dhrt_pfad, tmp_path):
    """Drei Dateien, drei Warnungen -- jede muss in ihrer eigenen landen."""
    err = _lauf(dhrt_pfad, tmp_path, {
        "e1.dh": "PRINT WEGA(1)\n",
        "e2.dh": "PRINT 0\nPRINT WEGB(1)\n",
        "main.dh": 'IMPORT "e1.dh"\nIMPORT "e2.dh"\nPRINT WEGC(1)\n',
    })
    assert "e1.dh:1" in err, err
    assert "e2.dh:2" in err, err
    assert "main.dh:3" in err, err
