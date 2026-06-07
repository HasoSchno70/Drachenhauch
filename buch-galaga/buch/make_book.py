#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Zwei-Pass-Build des Galaga-Buchs mit korrekten Inhaltsverzeichnis-Seitenzahlen.

Pass 1:  node build_book.js  -> .docx (TOC noch ohne Zahlen) + toc_titles.json
         -> LibreOffice rendert PDF -> Seitenzahl je Ueberschrift messen
         -> toc_pages.json schreiben
Pass 2:  node build_book.js  -> .docx mit eingetragenen Seitenzahlen

Das Layout ist zwischen beiden Pässen stabil, weil das Verzeichnis in beiden
Fällen gleich viele Zeilen (eine Seite) belegt -- nur die Zahlen kommen hinzu.

Aufruf:  <venv>/python.exe make_book.py
(Reines `node build_book.js` funktioniert weiterhin und nutzt die zuletzt
gemessenen Seitenzahlen aus toc_pages.json.)
"""
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SOFFICE = r"C:\Program Files\LibreOffice\program\soffice.exe"
DOCX = os.path.join(HERE, "GameBasic-Buch.docx")
PDF = os.path.join(HERE, "GameBasic-Buch.pdf")
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
    # Seiten 1 (Titel) und 2 (Inhalt) ueberspringen -- ab Seite 3 (Index 2) suchen.
    for t in titles:
        for i in range(2, d.page_count):
            if d[i].search_for(t):
                pages[t] = i + 1
                break
    d.close()
    return pages


def main():
    # Pass 1
    build()
    titles = json.load(open(TITLES, encoding="utf-8"))
    render()
    pages = measure(PDF, titles)
    missing = [t for t in titles if t not in pages]
    if missing:
        print("WARN: keine Seite gefunden fuer:", missing, file=sys.stderr)
    json.dump(pages, open(PAGES, "w", encoding="utf-8"),
              ensure_ascii=False, indent=2)
    # Pass 2 (mit Seitenzahlen)
    build()
    print("Inhaltsverzeichnis-Seiten:")
    for t in titles:
        print(f"  {pages.get(t, '?'):>3}  {t}")


if __name__ == "__main__":
    main()
