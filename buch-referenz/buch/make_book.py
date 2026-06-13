#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Zwei-Pass-Build des GameBasic-Lehrbuchs mit korrekten ToC-Seitenzahlen.

Pass 1:  node build_book.js  -> .docx (ToC noch ohne Zahlen) + toc_titles.json
         -> LibreOffice rendert PDF -> Seitenzahl je Ueberschrift messen
         -> toc_pages.json schreiben
Pass 2:  node build_book.js  -> .docx mit eingetragenen Seitenzahlen

Aufruf:  <venv>\\python.exe make_book.py
(Reines `node build_book.js` nutzt die zuletzt gemessenen Seiten aus
toc_pages.json.)
"""
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SOFFICE = r"C:\Program Files\LibreOffice\program\soffice.exe"
DOCX = os.path.join(HERE, "GameBasic-Lehrbuch.docx")
PDF = os.path.join(HERE, "GameBasic-Lehrbuch.pdf")
TITLES = os.path.join(HERE, "toc_titles.json")
PAGES = os.path.join(HERE, "toc_pages.json")


def build():
    subprocess.run("node build_book.js", cwd=HERE, shell=True, check=True)


def render():
    subprocess.run([SOFFICE, "--headless", "--convert-to", "pdf",
                    "--outdir", HERE, DOCX], cwd=HERE, check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return PDF


def measure(pdf_path, titles):
    import fitz
    d = fitz.open(pdf_path)
    pages = {}
    # Titel (1) + Inhalt (2) ueberspringen -> ab Seite 3 (Index 2) suchen.
    for t in titles:
        for i in range(2, d.page_count):
            if d[i].search_for(t):
                pages[t] = i + 1
                break
    d.close()
    return pages


def main():
    build()
    titles = json.load(open(TITLES, encoding="utf-8"))
    render()
    pages = measure(PDF, titles)
    missing = [t for t in titles if t not in pages]
    if missing:
        print("WARN: keine Seite gefunden fuer:", missing, file=sys.stderr)
    json.dump(pages, open(PAGES, "w", encoding="utf-8"),
              ensure_ascii=False, indent=2)
    build()   # Pass 2 mit Seitenzahlen
    print("ToC-Seiten gemessen:", len(pages), "/", len(titles))


if __name__ == "__main__":
    main()
