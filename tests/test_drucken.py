"""Drucken durch einen ECHTEN Druckertreiber (docs/entwurf-drucken.md, Pruefstein 1).

"Microsoft Print to PDF" nimmt in DOCINFO einen Ausgabepfad und fragt dann
nicht nach -- der Test druckt dorthin und liest die Datei mit PyMuPDF zurueck,
samt der rechten Kante des Betrags. Auf macOS/Linux ist die Zieldatei die PDF
selbst (CUPS wuerde sie drucken); derselbe Test prueft dort denselben Inhalt --
nur eben nicht durch einen Treiber.

Druckerliste, PDF_PREVIEW und die Fehlermeldungen stehen seit 2026-09-16 in
tests/pruef/drucken.dhtest. Dieser Fall bleibt hier, weil die Datei des Treibers
kein PDF von dhrt ist: der Leser in Drachenhauch (pdftext.dh) kennt nur das,
was krilla schreibt, und keine Textlagen.
"""
import os
import subprocess
import sys
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

_SEITE = ('IMPORT "pdf"\nIMPORT "imgfx"\n'
          'DIM p AS PDF : p = PDF_NEW()\n'
          'PDF_TITLE(p, "Drucktest")\n'
          'PDF_FONT(p, "helvetica-fett", 18)\n'
          'PDF_TEXT(p, 20, 25, "Rechnung 2026-0002")\n'
          'PDF_FONT(p, "helvetica", 11)\n'
          'PDF_TEXT(p, 20, 40, "Anna Berger")\n'
          'PDF_LINE(p, 20, 50, 190, 50)\n'
          'PDF_RECT_FILL(p, 20, 60, 40, 10)\n'
          'PDF_TEXT(p, 190 - PDF_TEXT_WIDTH(p, "85,39 EUR"), 80, "85,39 EUR")\n'
          'PDF_PAGE(p)\n'
          'PDF_TEXT(p, 20, 25, "Seite zwei")\n')


def _lauf(tmp_path, src, frames=1, erwartet_ok=True):
    f = tmp_path / "t.dh"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=180,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    if erwartet_ok:
        assert r.returncode == 0, (r.stdout, r.stderr)
    return [ln.strip() for ln in (r.stdout or "").splitlines()
            if ln.strip() and not ln.startswith(("WARNING:", "INFO:"))]


def _drucker(tmp_path):
    out = _lauf(tmp_path, 'DIM n AS STRING\nFOR EACH n IN PRINTERS()\n    PRINT "D " + n\nNEXT\n'
                          'PRINT "[" + PRINTER_DEFAULT$() + "]"\n')
    return [z[2:] for z in out if z.startswith("D ")], out[-1].strip("[]")


def test_druck_durch_den_treiber_in_eine_datei(tmp_path):
    fitz = pytest.importorskip("fitz", reason="PyMuPDF nicht installiert")
    liste, _ = _drucker(tmp_path)
    if sys.platform == "win32":
        if "Microsoft Print to PDF" not in liste:
            pytest.skip("kein 'Microsoft Print to PDF' auf dieser Maschine")
        drucker = "Microsoft Print to PDF"
    else:
        drucker = "egal"          # ohne GDI ist die Zieldatei die PDF selbst
    # Windows kann "den zuletzt benutzten Drucker zum Standard machen"
    # (Einstellung "Standarddrucker von Windows verwalten lassen") -- der
    # Test druckt auf Print to PDF und verschoebe damit den Standard des
    # Entwicklers. Vorher merken, nachher zuruecksetzen.
    _, standard_vorher = _drucker(tmp_path)
    try:
        out = _lauf(tmp_path, _SEITE +
                    f'PDF_PRINT(p, "{drucker}", 1, "druck.pdf")\n'
                    'PRINT "gedruckt"\n')
    finally:
        if sys.platform == "win32" and standard_vorher:
            _, standard_nachher = _drucker(tmp_path)
            if standard_nachher != standard_vorher:
                subprocess.run(["powershell", "-NoProfile", "-Command",
                                f'(New-Object -ComObject WScript.Network).SetDefaultPrinter("{standard_vorher}")'],
                               capture_output=True, timeout=60)
    assert out[-1] == "gedruckt"
    d = fitz.open(str(tmp_path / "druck.pdf"))
    assert d.page_count == 2, "zwei Seiten, wie im Dokument"
    text = d[0].get_text()
    for erwartet in ("Rechnung 2026-0002", "Anna Berger", "85,39 EUR"):
        assert erwartet in text, text
    assert "Seite zwei" in d[1].get_text()
    # Der Betrag steht rechts: sein Kasten endet nahe 190 mm (= 538 pt).
    kaesten = [b for b in d[0].get_text("blocks") if "85,39" in b[4]]
    assert kaesten, "der Betrag hat einen Textkasten"
    rechts_pt = kaesten[0][2]
    assert abs(rechts_pt - 190 * 72 / 25.4) < 12, f"rechte Kante bei {rechts_pt:.0f} pt statt ~538"
