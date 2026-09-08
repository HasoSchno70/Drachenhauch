"""Modul `pdf` -- druckfertige Seiten (Punkt 7 des Allzweck-Audits).

Rechnung, Lieferschein, Bericht, Etikett: fuer kaufmaennische Software ist
das fast immer die erste Forderung nach "speichern".

**Geprueft wird mit einem FREMDEN Leser** (PyMuPDF, steht in
requirements.txt): eine selbst geschriebene Datei mit dem eigenen Schreiber
gegenzulesen sagt nichts darueber, ob ein Acrobat sie oeffnet. Genau dieser
Test hat auch den ersten echten Fehler gefunden -- der Titel stand im
Trailer statt in einem Info-Objekt und war fuer jeden Leser unsichtbar.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/pdf.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import hashlib

import pytest

fitz = pytest.importorskip("fitz", reason="PyMuPDF (requirements.txt) nicht installiert")

KOPF = 'IMPORT "pdf"\nDIM p AS PDF\np = PDF_NEW()\n'


def _oeffne(tmp_path, name="a.pdf"):
    return fitz.open(str(tmp_path / name))


# ------------------------------------------------------------- Grundlagen


def test_zweimal_dasselbe_ergibt_dieselbe_datei(tmp_path, run_gb):
    """Kein Erstellungsdatum im Dokument -- das macht Pruefungen
    vergleichbar und einen Versionsverlauf lesbar."""
    quelle = KOPF + 'PDF_TEXT(p, 20, 25, "gleich")\nPDF_SAVE(p, "a.pdf")\n'
    run_gb(quelle, base=tmp_path)
    h1 = hashlib.sha256((tmp_path / "a.pdf").read_bytes()).hexdigest()
    run_gb(quelle, base=tmp_path)
    h2 = hashlib.sha256((tmp_path / "a.pdf").read_bytes()).hexdigest()
    assert h1 == h2


# ------------------------------------------------------------- Breite messen


# ----------------------------------------------------------- Fehlerfaelle
