"""Modul `xlsx` -- Auswertungen als Excel-Mappe (Punkt 7 des Audits).

CSV gab es schon. Was es NICHT kann: mehrere Blaetter, eine fette Kopfzeile,
Spaltenbreiten, Zahlen- und Datumsformate -- alles, was aus einer Datenliste
eine abgabefertige Auswertung macht.

**Gegengelesen wird mit openpyxl** (requirements.txt), also mit einem
FREMDEN Leser. Ohne den hiesse "die Datei ist in Ordnung" nur "mein
Schreiber ist mit sich einig" -- und genau dieser Test hat die fehlende
Standard-Formatvorlage gefunden.
"""
import datetime

import pytest

openpyxl = pytest.importorskip("openpyxl", reason="openpyxl (requirements.txt) fehlt")

from drachenhauch.errors import DHRuntimeError  # noqa: E402

KOPF = 'IMPORT "xlsx"\nDIM x AS XLSX\nx = XLSX_NEW()\n'


def _mappe(tmp_path, name="a.xlsx"):
    return openpyxl.load_workbook(str(tmp_path / name))


# ------------------------------------------------------------- Grundlagen
def test_eine_mappe_laesst_sich_oeffnen(tmp_path, run_gb):
    run_gb(KOPF + 'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    assert _mappe(tmp_path).sheetnames == ["Tabelle1"]


def test_ohne_warnung_lesbar(tmp_path, run_gb):
    """Ein strenger Leser meldete `Workbook contains no default style` --
    die benannte Vorlage gehoert dazu, auch wenn niemand sie benutzt."""
    import warnings
    run_gb(KOPF + 'XLSX_SET(x, 0, 0, "hallo")\nXLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    with warnings.catch_warnings():
        warnings.simplefilter("error")
        openpyxl.load_workbook(str(tmp_path / "a.xlsx"))


def test_text_und_zahl_bleiben_verschieden(tmp_path, run_gb):
    """Der Grund, warum CSV hier nicht reicht: dort ist alles Text, und eine
    Postleitzahl `01067` wird beim Oeffnen zu `1067`."""
    run_gb(KOPF + 'XLSX_SET(x, 0, 0, "01067")\n'
           "XLSX_SET_NUM(x, 0, 1, 1234.5)\n"
           'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    ws = _mappe(tmp_path).active
    assert ws["A1"].value == "01067"
    assert ws["B1"].value == 1234.5
    assert isinstance(ws["B1"].value, float)


def test_umlaute_und_sonderzeichen(tmp_path, run_gb):
    run_gb(KOPF + 'XLSX_SET(x, 0, 0, "Schrauben & <M' + chr(252) + 'ller>")\n'
           'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    assert _mappe(tmp_path).active["A1"].value == "Schrauben & <Müller>"


def test_mehrere_blaetter(tmp_path, run_gb):
    out = run_gb('IMPORT "xlsx"\nDIM x AS XLSX\n'
                 'x = XLSX_NEW("Umsatz")\n'
                 'XLSX_SET(x, 0, 0, "eins")\n'
                 'XLSX_SHEET(x, "Notizen")\n'
                 'XLSX_SET(x, 0, 0, "zwei")\n'
                 "PRINT XLSX_SHEET_COUNT(x)\n"
                 'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    assert out.strip() == "2"
    wb = _mappe(tmp_path)
    assert wb.sheetnames == ["Umsatz", "Notizen"]
    assert wb["Umsatz"]["A1"].value == "eins"
    assert wb["Notizen"]["A1"].value == "zwei"


def test_luecken_bleiben_leer(tmp_path, run_gb):
    """Nur belegte Zellen werden geschrieben -- eine Auswertung mit einer
    Spalte und tausend Zeilen soll keine Tabelle von tausend mal tausend
    anlegen."""
    run_gb(KOPF + 'XLSX_SET(x, 0, 0, "oben")\n'
           'XLSX_SET(x, 10, 3, "weit unten")\n'
           'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    ws = _mappe(tmp_path).active
    assert ws["A1"].value == "oben"
    assert ws["D11"].value == "weit unten"
    assert ws["B5"].value is None


# ------------------------------------------------------------- Gestaltung
def test_fette_kopfzeile(tmp_path, run_gb):
    run_gb(KOPF + 'XLSX_SET(x, 0, 0, "Kunde")\n'
           'XLSX_SET(x, 0, 1, "Betrag")\n'
           'XLSX_SET(x, 1, 0, "normal")\n'
           "XLSX_BOLD_ROW(x, 0)\n"
           'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    ws = _mappe(tmp_path).active
    assert ws["A1"].font.bold and ws["B1"].font.bold
    assert not ws["A2"].font.bold


def test_zahlenformat(tmp_path, run_gb):
    run_gb(KOPF + "XLSX_SET_NUM(x, 0, 0, 1234.5)\n"
           'XLSX_FORMAT(x, 0, 0, "#,##0.00")\n'
           'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    assert _mappe(tmp_path).active["A1"].number_format == "#,##0.00"


def test_datum_kommt_als_datum_an(tmp_path, run_gb):
    """In Excel ist ein Datum eine Zahl MIT Format -- ohne Format staende
    dort die nackte Tageszahl."""
    run_gb('IMPORT "xlsx"\nIMPORT "zeit"\nDIM x AS XLSX\nx = XLSX_NEW()\n'
           'XLSX_SET_DATE(x, 0, 0, ZEIT_PARSE("2026-08-23 00:00:00"))\n'
           'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    c = _mappe(tmp_path).active["A1"]
    assert c.value == datetime.datetime(2026, 8, 23)
    assert c.number_format == "DD.MM.YYYY"


def test_spaltenbreite(tmp_path, run_gb):
    run_gb(KOPF + 'XLSX_SET(x, 0, 0, "lang")\n'
           "XLSX_COL_WIDTH(x, 0, 28)\n"
           'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    assert _mappe(tmp_path).active.column_dimensions["A"].width == 28


def test_gestaltung_ueberlebt_eine_neue_zuweisung(tmp_path, run_gb):
    """Sonst muesste man nach jeder Wertaenderung fett und Format neu
    setzen."""
    run_gb(KOPF + 'XLSX_SET(x, 0, 0, "alt")\n'
           "XLSX_BOLD(x, 0, 0)\n"
           'XLSX_SET(x, 0, 0, "neu")\n'
           'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    c = _mappe(tmp_path).active["A1"]
    assert c.value == "neu" and c.font.bold


def test_viele_spalten_heissen_richtig(tmp_path, run_gb):
    """Spalte 26 ist AA, nicht BA."""
    run_gb(KOPF + 'XLSX_SET(x, 0, 26, "AA")\n'
           'XLSX_SET(x, 0, 27, "AB")\n'
           'XLSX_SAVE(x, "a.xlsx")\n', base=tmp_path)
    ws = _mappe(tmp_path).active
    assert ws["AA1"].value == "AA" and ws["AB1"].value == "AB"


# ----------------------------------------------------------- Fehlerfaelle
def test_blattname_zu_lang(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "xlsx"\nDIM x AS XLSX\nx = XLSX_NEW("' + "x" * 32 + '")\n')
    assert "31 Zeichen" in str(e.value)


def test_blattname_mit_verbotenem_zeichen(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "xlsx"\nDIM x AS XLSX\nx = XLSX_NEW("Q1/Q2")\n')
    assert "erlaubt kein" in str(e.value)


def test_zwei_blaetter_gleichen_namens(run_gb):
    """Excel unterscheidet Blattnamen nicht nach Gross- und
    Kleinschreibung."""
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "xlsx"\nDIM x AS XLSX\nx = XLSX_NEW("Daten")\n'
               'XLSX_SHEET(x, "daten")\n')
    assert "schon ein Blatt" in str(e.value)


def test_negative_zelle(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'XLSX_SET(x, -1, 0, "x")\n')
    assert "negativ" in str(e.value)


def test_jenseits_von_excels_grenzen(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'XLSX_SET(x, 0, 20000, "x")\n')
    assert "16383" in str(e.value)
