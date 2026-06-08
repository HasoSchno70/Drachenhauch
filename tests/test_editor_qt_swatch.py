"""Tests fuer die Color-Swatches im Code-Editor.

Geprueft werden die beiden frueheren Macken:
1. Swatches ueberdeckten den Code, wenn direkt hinter dem Color-Literal weiterer
   Code stand (z.B. `&HFF0000 : ...` oder `RGB(...)` vor `)`). -> Bei fehlendem
   Platz wird jetzt ein farbiger Unterstrich UNTER dem Literal gezeichnet, der
   nie Nachbar-Text ueberlappt.
2. Swatches wuchsen beim Vergroessern der Schrift nicht mit. -> Groesse haengt
   jetzt an der Schrift-Hoehe.
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


def test_end_of_line_literal_is_square_box(app):
    ed = _editor(app, "col = &HFF8800")
    r = _rect(ed, 0)
    size = ed._swatch_metrics()[0]
    assert r.width() == size and r.height() == size      # quadratisches Kaestchen


def test_literal_before_paren_uses_underline(app):
    # &H05060F steht direkt vor ')' -> kein Platz -> Unterstrich (breit + duenn)
    ed = _editor(app, "CLS(&H05060F)")
    r = _rect(ed, 0)
    size = ed._swatch_metrics()[0]
    assert r.height() < size                  # duenn (kein volles Kaestchen)
    assert r.width() > r.height()             # spannt das Literal -> breit


def test_swatch_never_overlaps_following_code(app):
    # Mehrere Literale mit ' : '-Trennern: das Kaestchen/der Unterstrich darf nie
    # ueber das naechste Nicht-Leerzeichen hinausragen.
    ed = _editor(app, "a = &HBE64F0 : b = &HE44040 : c = &H46B4FF")
    blk = ed.document().findBlockByNumber(0)
    line = blk.text()
    for s, e, _c, _k in ed._scan_color_swatches(line):
        r = ed._swatch_rect(blk, s, e)
        # naechstes Nicht-Leerzeichen hinter dem Literal
        j = e
        while j < len(line) and line[j] in " \t":
            j += 1
        if j < len(line):
            next_x = ed._col_rect(blk, j).left()
            # Box-Variante sitzt RECHTS vom Literal -> darf next_x nicht ueberdecken.
            # Unterstrich-Variante endet am Literal-Ende (immer < next_x).
            assert r.right() <= next_x, f"Swatch ueberdeckt Code bei Spalte {e}"


def test_swatch_scales_with_font(app):
    from PySide6.QtGui import QFont
    ed = _editor(app, "col = &HFF8800")
    small = ed._swatch_metrics()[0]
    f = ed.font()
    f.setPointSize(f.pointSize() + 8)
    ed.setFont(f)
    app.processEvents()
    big = ed._swatch_metrics()[0]
    assert big > small
