#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Zwei-Pass-Build des Tippspiel-Buchs mit Seitenzahlen im Inhaltsverzeichnis.

Pass 1:  node build_book.js  -> .docx (Verzeichnis noch ohne Zahlen)
         -> LibreOffice rendert ein PDF
         -> je Ueberschrift die Seite messen -> toc_pages.json
Pass 2:  node build_book.js  -> .docx mit eingetragenen Seitenzahlen

Das Layout bleibt zwischen beiden Paessen stabil, weil das Verzeichnis in
beiden Faellen gleich viele Zeilen belegt -- nur die Zahlen kommen hinzu.

Aufruf:  python make_book.py
Ohne LibreOffice oder PyMuPDF laeuft `node build_book.js` weiterhin allein --
dann bleibt das Verzeichnis ohne Seitenzahlen, das Dokument entsteht trotzdem.
"""
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SOFFICE = r"C:\Program Files\LibreOffice\program\soffice.exe"
DOCX = os.path.join(HERE, "Drachenhauch-Tippspiel.docx")
PDF = os.path.join(HERE, "Drachenhauch-Tippspiel.pdf")
TITLES = os.path.join(HERE, "toc_titles.json")
PAGES = os.path.join(HERE, "toc_pages.json")


def build():
    subprocess.run("node build_book.js", cwd=HERE, shell=True, check=True)


def render():
    if not os.path.exists(SOFFICE):
        return None
    subprocess.run([SOFFICE, "--headless", "--convert-to", "pdf",
                    "--outdir", HERE, DOCX], cwd=HERE, check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return PDF


def measure(pdf_path, titles):
    """Erste Seite suchen, auf der die Ueberschrift steht.

    Die Titelseite (1) und das Verzeichnis (2) werden uebersprungen -- im
    Verzeichnis steht jede Ueberschrift ja auch, und zwar zuerst.
    """
    import pymupdf
    d = pymupdf.open(pdf_path)
    pages = {}
    for t in titles:
        for i in range(2, d.page_count):
            if d[i].search_for(t):
                pages[t] = i + 1
                break
    d.close()
    return pages


def main():
    print("Pass 1: Dokument bauen ...")
    build()

    pdf = render()
    if pdf is None:
        print("LibreOffice nicht gefunden -- Verzeichnis bleibt ohne Seitenzahlen.")
        return 0

    try:
        import pymupdf  # noqa: F401
    except ImportError:
        print("PyMuPDF fehlt (pip install pymupdf) -- Verzeichnis ohne Seitenzahlen.")
        return 0

    with open(TITLES, encoding="utf-8") as f:
        titles = json.load(f)
    pages = measure(pdf, titles)
    fehlend = [t for t in titles if t not in pages]
    with open(PAGES, "w", encoding="utf-8") as f:
        json.dump(pages, f, ensure_ascii=False, indent=2)
    print(f"Gemessen: {len(pages)} von {len(titles)} Ueberschriften")
    for t in fehlend:
        print("  nicht gefunden:", t)

    print("Pass 2: Dokument mit Seitenzahlen bauen ...")
    build()
    render()   # PDF auf denselben Stand bringen
    print("Fertig:", os.path.basename(DOCX))
    return 0


if __name__ == "__main__":
    sys.exit(main())
