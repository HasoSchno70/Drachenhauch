"""Tests fuer die Color-Swatches im Code-Editor.

Darstellung: Das Color-Literal (`&HRRGGBB` / `RGB(r,g,b)`) wird mit SEINER Farbe
hinterlegt, die Schrift darueber in lesbarem Kontrast. Geprueft:
1. Der farbige Hintergrund liegt ueber dem Literal und ragt NICHT in
   anschliessenden Code hinein (kein Ueberdecken mehr).
2. Die Schriftfarbe passt sich an (hell auf dunkel, dunkel auf hell).
3. Der Hintergrund waechst mit der Schrift (Hoehe = Zeilenhoehe).
4. Die Flaeche ist anklickbar (`_swatch_at` trifft -> Farbwaehler).
"""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

pytest.importorskip("PySide6")


@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


def _editor(app, text):
    from gamebasic.editor_qt.code_editor import CodeEditor
    ed = CodeEditor()
    ed.set_text(text)
    ed.resize(900, 400)
    ed.show()
    app.processEvents()
    return ed


def _rect(ed, line, occ=0):
    blk = ed.document().findBlockByNumber(line)
    s, e, _c, _k = ed._scan_color_swatches(blk.text())[occ]
    return ed._swatch_rect(blk, s, e)


def test_background_covers_literal_width(app):
    ed = _editor(app, "col = &HFF8800")
    blk = ed.document().findBlockByNumber(0)
    s, e, _c, _k = ed._scan_color_swatches(blk.text())[0]
    r = ed._swatch_rect(blk, s, e)
    lit_w = ed.fontMetrics().horizontalAdvance("&HFF8800")
    # Hintergrund ~ Literalbreite (plus kleiner Innenabstand), Hoehe ~ Zeilenhoehe
    assert r.width() >= lit_w
    assert r.height() >= ed.fontMetrics().height() - 4


def test_contrast_text_color(app):
    from PySide6.QtGui import QColor
    ed = _editor(app, "x = 1")
    dark_bg = ed._swatch_text_color(QColor(10, 10, 10))      # dunkler Hintergrund
    light_bg = ed._swatch_text_color(QColor(240, 240, 240))  # heller Hintergrund
    # auf Dunkel -> helle Schrift, auf Hell -> dunkle Schrift
    assert dark_bg.lightness() > 200
    assert light_bg.lightness() < 80


def test_does_not_overlap_following_code(app):
    # Literale direkt vor ')' bzw. ' : ' -- der Hintergrund darf nie ueber das
    # naechste Nicht-Leerzeichen hinausragen.
    ed = _editor(app, "CLS(&H05060F)\na = &HBE64F0 : b = &HE44040")
    doc = ed.document()
    for ln in (0, 1):
        blk = doc.findBlockByNumber(ln)
        line = blk.text()
        for s, e, _c, _k in ed._scan_color_swatches(line):
            r = ed._swatch_rect(blk, s, e)
            j = e
            while j < len(line) and line[j] in " \t":
                j += 1
            if j < len(line):
                next_x = ed._col_rect(blk, j).left()
                assert r.right() <= next_x, f"Hintergrund ueberdeckt Code @ {e}"


def test_scales_with_font(app):
    from PySide6.QtGui import QFont
    ed = _editor(app, "col = &HFF8800")
    h_small = _rect(ed, 0).height()
    f = ed.font()
    f.setPointSize(f.pointSize() + 8)
    ed.setFont(f)
    app.processEvents()
    h_big = _rect(ed, 0).height()
    assert h_big > h_small


def test_clickable_opens_picker(app, monkeypatch):
    # Klick mitten auf das hinterlegte Literal muss den Farbwaehler ausloesen.
    ed = _editor(app, "col = &HFF8800")
    r = _rect(ed, 0)
    hit = ed._swatch_at(r.center())
    assert hit is not None
    abs_start, abs_end, color, kind = hit
    assert kind == "hex"
    assert color.red() == 0xFF and color.green() == 0x88 and color.blue() == 0x00
