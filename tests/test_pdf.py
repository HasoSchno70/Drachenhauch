"""Modul `pdf` -- druckfertige Seiten (Punkt 7 des Allzweck-Audits).

Rechnung, Lieferschein, Bericht, Etikett: fuer kaufmaennische Software ist
das fast immer die erste Forderung nach "speichern".

**Geprueft wird mit einem FREMDEN Leser** (PyMuPDF, steht in
requirements.txt): eine selbst geschriebene Datei mit dem eigenen Schreiber
gegenzulesen sagt nichts darueber, ob ein Acrobat sie oeffnet. Genau dieser
Test hat auch den ersten echten Fehler gefunden -- der Titel stand im
Trailer statt in einem Info-Objekt und war fuer jeden Leser unsichtbar.
"""
import hashlib

import pytest

fitz = pytest.importorskip("fitz", reason="PyMuPDF (requirements.txt) nicht installiert")

KOPF = 'IMPORT "pdf"\nDIM p AS PDF\np = PDF_NEW()\n'


def _oeffne(tmp_path, name="a.pdf"):
    return fitz.open(str(tmp_path / name))


# ------------------------------------------------------------- Grundlagen
def test_ein_leeres_dokument_laesst_sich_oeffnen(tmp_path, run_gb):
    run_gb(KOPF + 'PDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    d = _oeffne(tmp_path)
    assert d.page_count == 1


def test_a4_ist_die_vorgabe(tmp_path, run_gb):
    run_gb(KOPF + 'PDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    s = _oeffne(tmp_path)[0]
    assert round(s.rect.width) == 595 and round(s.rect.height) == 842


def test_querformat_und_andere_groessen(tmp_path, run_gb):
    run_gb('IMPORT "pdf"\nDIM p AS PDF\n'
           'p = PDF_NEW("a5", "quer")\nPDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    s = _oeffne(tmp_path)[0]
    assert s.rect.width > s.rect.height
    assert round(s.rect.width) == round(210 * 72 / 25.4)


def test_text_kommt_an(tmp_path, run_gb):
    run_gb(KOPF + 'PDF_TEXT(p, 20, 25, "Rechnung 4711")\nPDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    assert "Rechnung 4711" in _oeffne(tmp_path)[0].get_text()


def test_umlaute_kommen_an(tmp_path, run_gb):
    """WinAnsi (= cp1252) deckt Deutsch ab -- ohne eingebettete Schrift."""
    run_gb(KOPF + 'PDF_TEXT(p, 20, 25, "Gr' + chr(252) + chr(223) + 'e aus K' + chr(246) + 'ln")\n'
           'PDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    assert "Grüße aus Köln" in _oeffne(tmp_path)[0].get_text()


def test_millimeter_von_oben(tmp_path, run_gb):
    """Der Unterschied zu rohem PDF: dort wird in Punkten von UNTEN
    gerechnet. Wer eine Rechnung setzt, denkt in `25 mm vom oberen Rand`."""
    run_gb(KOPF + 'PDF_FONT(p, "helvetica", 10)\n'
           'PDF_TEXT(p, 20, 25, "Marke")\nPDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    s = _oeffne(tmp_path)[0]
    span = s.get_text("dict")["blocks"][0]["lines"][0]["spans"][0]
    x_mm = span["bbox"][0] * 25.4 / 72
    y_mm = span["bbox"][1] * 25.4 / 72
    assert abs(x_mm - 20) < 0.5, x_mm
    # Die Oberkante der Zeile liegt bei 25 mm (auf die Zeilenhoehe genau).
    assert abs(y_mm - 25) < 2.0, y_mm


def test_mehrere_seiten(tmp_path, run_gb):
    out = run_gb(KOPF + 'PDF_TEXT(p, 20, 25, "eins")\n'
                 "PDF_PAGE(p)\n"
                 'PDF_TEXT(p, 20, 25, "zwei")\n'
                 "PRINT PDF_PAGE_COUNT(p)\n"
                 'PDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    assert out.strip() == "2"
    d = _oeffne(tmp_path)
    assert d.page_count == 2
    assert "eins" in d[0].get_text() and "zwei" in d[1].get_text()


def test_schriften_landen_auf_der_seite(tmp_path, run_gb):
    run_gb(KOPF + 'PDF_FONT(p, "helvetica-fett", 18)\nPDF_TEXT(p, 20, 25, "fett")\n'
           'PDF_FONT(p, "times", 11)\nPDF_TEXT(p, 20, 40, "times")\n'
           'PDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    namen = [f[3] for f in _oeffne(tmp_path)[0].get_fonts()]
    assert "Helvetica-Bold" in namen and "Times-Roman" in namen


def test_striche_und_flaechen(tmp_path, run_gb):
    run_gb(KOPF + "PDF_LINE(p, 20, 45, 190, 45)\n"
           "PDF_RECT(p, 20, 50, 40, 10)\n"
           "PDF_RECT_FILL(p, 70, 50, 40, 10)\n"
           'PDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    zeichnungen = _oeffne(tmp_path)[0].get_drawings()
    assert len(zeichnungen) == 3
    arten = {d["type"] for d in zeichnungen}
    assert "s" in arten and "f" in arten     # Strich und Fuellung


def test_farbe_wirkt(tmp_path, run_gb):
    run_gb(KOPF + "PDF_COLOR(p, RGB(220, 30, 30))\n"
           "PDF_RECT_FILL(p, 20, 50, 40, 10)\n"
           'PDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    d = _oeffne(tmp_path)[0].get_drawings()[0]
    r, g, b = d["fill"]
    assert abs(r - 220 / 255) < 0.01 and abs(g - 30 / 255) < 0.01


def test_titel_steht_in_den_angaben(tmp_path, run_gb):
    """Der Fehler, den erst der fremde Leser zeigte: im Trailer sieht ihn
    niemand, er gehoert in ein Info-Objekt."""
    run_gb(KOPF + 'PDF_TITLE(p, "Rechnung 4711")\nPDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    d = _oeffne(tmp_path)
    assert d.metadata["title"] == "Rechnung 4711"
    assert d.metadata["producer"] == "Drachenhauch"


def test_zweimal_dasselbe_ergibt_dieselbe_datei(tmp_path, run_gb):
    """Kein Erstellungsdatum im Dokument -- das macht Pruefungen
    vergleichbar und einen Versionsverlauf lesbar."""
    quelle = KOPF + 'PDF_TEXT(p, 20, 25, "gleich")\nPDF_SAVE(p, "a.pdf")\n'
    run_gb(quelle, base=tmp_path)
    h1 = hashlib.sha256((tmp_path / "a.pdf").read_bytes()).hexdigest()
    run_gb(quelle, base=tmp_path)
    h2 = hashlib.sha256((tmp_path / "a.pdf").read_bytes()).hexdigest()
    assert h1 == h2


def test_einstellungen_gelten_ueber_den_seitenwechsel(tmp_path, run_gb):
    """Schrift und Farbe sind Sache des Dokuments, nicht der Seite -- alles
    andere hiesse, sie nach jedem Seitenwechsel neu zu setzen."""
    run_gb(KOPF + 'PDF_FONT(p, "courier-fett", 14)\n'
           "PDF_PAGE(p)\n"
           'PDF_TEXT(p, 20, 25, "zweite")\n'
           'PDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    assert "Courier-Bold" in [f[3] for f in _oeffne(tmp_path)[1].get_fonts()]


# ------------------------------------------------------------- Breite messen
def test_breite_bei_courier_ist_exakt(tmp_path, run_gb):
    """Bei Courier ist jedes Zeichen 600/1000 der Schriftgroesse breit --
    das ist die Bauart der Schrift, keine Schaetzung."""
    out = run_gb(KOPF + 'PDF_FONT(p, "courier", 10)\n'
                 'PRINT FORMAT$(PDF_TEXT_WIDTH(p, "1234567890"), "%.3f")\n')
    erwartet = 10 * 10 * 0.6 * 25.4 / 72
    assert abs(float(out.strip()) - erwartet) < 0.001


def test_breite_bei_helvetica_verweigert_sich_mit_grund(tmp_path, run_gb):
    """Lieber keine Antwort als eine erfundene: eine geschaetzte Breite
    verschiebt eine Rechnungsspalte still um zwei Millimeter."""
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'PDF_FONT(p, "helvetica", 11)\n'
               'PRINT PDF_TEXT_WIDTH(p, "abc")\n')
    assert "dicktengleich" in str(e.value)


# ----------------------------------------------------------- Fehlerfaelle
def test_unbekannte_schrift_zaehlt_die_moeglichen_auf(run_gb):
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'PDF_FONT(p, "comic sans", 12)\n')
    assert "helvetica" in str(e.value) and "courier" in str(e.value)


def test_unbekannte_seitengroesse(run_gb):
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "pdf"\nDIM p AS PDF\np = PDF_NEW("din a4")\n')
    assert "a4" in str(e.value)


def test_zeichen_ausserhalb_winansi(run_gb):
    """Dieselbe Regel wie beim Schreiben einer cp1252-Datei: kein stilles
    Fragezeichen. Auf einer Rechnung ist ein verschwundenes Zeichen
    schlimmer als eine Meldung."""
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + 'PDF_TEXT(p, 20, 25, "Smiley " + CHR$(128512))\n')
    assert "WinAnsi" in str(e.value)


def test_klammern_im_text_brechen_nichts(tmp_path, run_gb):
    """In PDF begrenzen Klammern die Zeichenkette -- ohne Schutz waere die
    Datei kaputt."""
    run_gb(KOPF + 'PDF_TEXT(p, 20, 25, "Posten (2 Stueck) \\ Rest")\n'
           'PDF_SAVE(p, "a.pdf")\n', base=tmp_path)
    assert "Posten (2 Stueck) \\ Rest" in _oeffne(tmp_path)[0].get_text()
