#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Zwei-Pass-Build des Drachenhauch-Lehrbuchs mit korrekten ToC-Seitenzahlen.

Pass 1:  node build_book.js  -> .docx (ToC noch ohne Zahlen) + toc_titles.json
         -> LibreOffice rendert PDF -> Seitenzahl je Ueberschrift messen
         -> toc_pages.json schreiben
Pass 2:  node build_book.js  -> .docx mit eingetragenen Seitenzahlen

Aufruf:  <venv>\\python.exe make_book.py
(Reines `node build_book.js` nutzt die zuletzt gemessenen Seiten aus
toc_pages.json.)
"""
import glob
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SOFFICE = r"C:\Program Files\LibreOffice\program\soffice.exe"
DOCX = os.path.join(HERE, "Drachenhauch-Lehrbuch.docx")
PDF = os.path.join(HERE, "Drachenhauch-Lehrbuch.pdf")
TITLES = os.path.join(HERE, "toc_titles.json")
PAGES = os.path.join(HERE, "toc_pages.json")

# Druckdienste (epubli & Co.) verlangen: KEINE Transparenz + Bilder ~300 dpi.
# Die dhrt-Screenshots kommen als 480x320-RGBA (Alpha = Transparenz, und bei
# Anzeige in ~12,7 cm Breite nur ~96 dpi). prepare_images() macht sie druckfertig:
# Alpha auf Weiss flachklopfen (-> RGB) und ganzzahlig hochskalieren, bis die
# native Breite >= MIN_PRINT_W ist (bei 480 px -> Faktor 4 = 1920 px -> ~384 dpi).
# Idempotent: bereits aufbereitete Bilder (RGB, breit genug) werden uebersprungen.
# LibreOffice exportiert dann OHNE Aufloesungs-Reduktion (Filter unten).
MIN_PRINT_W = 1500


def prepare_images():
    from PIL import Image
    changed = 0
    for f in sorted(glob.glob(os.path.join(HERE, "images", "*.png"))):
        im = Image.open(f)
        if im.mode == "RGB" and im.width >= 1400:
            continue                                    # schon druckfertig
        if im.mode in ("RGBA", "LA", "PA") or "transparency" in im.info:
            im = im.convert("RGBA")
            bg = Image.new("RGB", im.size, (255, 255, 255))
            bg.paste(im, mask=im.split()[-1])           # Alpha auf Weiss
            im = bg
        else:
            im = im.convert("RGB")
        upscaled = im.width < 1400
        if upscaled:
            factor = -(-MIN_PRINT_W // im.width)         # ceil-Division
            im = im.resize((im.width * factor, im.height * factor), Image.NEAREST)
        im.save(f)
        changed += 1
    if changed:
        print("Bilder druckfertig gemacht:", changed,
              "(Alpha flach -> RGB; kleine zusaetzlich hochskaliert)")


def build():
    subprocess.run("node build_book.js", cwd=HERE, shell=True, check=True)


# PDF ohne Bild-Aufloesungs-Reduktion exportieren (volle 300+ dpi der Bilder
# bleiben erhalten -> erfuellt die Druck-Qualitaetspruefung).
_PDF_FILTER = ('pdf:writer_pdf_Export:'
               '{"ReduceImageResolution":{"type":"boolean","value":false}}')


def render():
    subprocess.run([SOFFICE, "--headless", "--convert-to", _PDF_FILTER,
                    "--outdir", HERE, DOCX], cwd=HERE, check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return PDF


def _heading_on_page(page, title):
    """TRUE, wenn der Titel auf der Seite als UEBERSCHRIFT vorkommt (Span mit
    Schriftgroesse >= 15: Kapitel=17, Teil=26; Fliesstext=11, h2=13, Code/cmd<=12
    werden so ausgeschlossen). Verhindert Treffer auf Fliesstext-Erwaehnungen
    (z.B. Querverweise wie ``... Maps (Kapitel 18)``)."""
    for b in page.get_text("dict")["blocks"]:
        for ln in b.get("lines", []):
            for s in ln.get("spans", []):
                if s["size"] >= 15 and title in s["text"]:
                    return True
    return False


def measure(pdf_path, titles):
    import fitz
    d = fitz.open(pdf_path)
    pages = {}
    # Titel (1) + Inhalt (2) ueberspringen -> ab Seite 3 (Index 2) suchen.
    # Monoton vorwaerts: Titel stehen in Dokument-Reihenfolge, jede Ueberschrift
    # liegt auf/ nach der vorigen -> kein Ruecksprung auf fruehe Erwaehnungen.
    start = 2
    for t in titles:
        found = None
        for i in range(start, d.page_count):
            if _heading_on_page(d[i], t):
                found = i
                break
        if found is None:  # Fallback: einfache Textsuche ab start
            for i in range(start, d.page_count):
                if d[i].search_for(t):
                    found = i
                    break
        if found is not None:
            pages[t] = found + 1
            start = found  # gleiche Seite fuer Folge-Titel erlaubt (h1+h2)
    d.close()
    return pages


def main():
    prepare_images()
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
