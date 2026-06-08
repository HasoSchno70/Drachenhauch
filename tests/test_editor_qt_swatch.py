"""Tests fuer die Color-Literal-Hervorhebung im Code-Editor.

Darstellung: Das Color-Literal (`&HRRGGBB` / `RGB(r,g,b)`) wird mit SEINER Farbe
hinterlegt, die Schrift darueber im Kontrast -- umgesetzt als `ExtraSelection`
(Teil des Text-Renderings, daher kein Flackern/Verschwinden bei Teil-Repaints).
Geprueft:
1. Alle Literale im Dokument werden erkannt und gecacht (`_color_literals`).
2. Jedes Literal liefert eine ExtraSelection mit Hintergrund = Farbe und
   passender (kontrastreicher) Schriftfarbe.
3. Die Schriftfarbe passt sich an (hell auf dunkel, dunkel auf hell).
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
    ed.resize(1000, 400)
    ed.show()
    app.processEvents()
    return ed


def test_all_literals_detected(app):
    # Mehrere Literale pro Zeile, in Klammern, vor ':' -- alle muessen rein.
    ed = _editor(app, "\n".join([
        "a = &HBE64F0 : b = &HE44040 : c = &H46B4FF",
        "CLS(&H05060F)",
        "col = &HFF8800",
        "x = RGB(255, 128, 0)",
    ]))
    # 3 + 1 + 1 + 1 = 6 Literale
    assert len(ed._color_literals) == 6
    kinds = [k for *_x, k in ed._color_literals]
    assert kinds.count("hex") == 5 and kinds.count("rgb") == 1


def test_extra_selections_have_color_background(app):
    ed = _editor(app, "col = &HFF8800")
    # Es gibt genau ein Literal -> eine ExtraSelection mit dessen Farbe als BG.
    bgs = [s.format.background().color() for s in ed.extraSelections()
           if s.format.background().style() != 0]
    assert any(c.red() == 0xFF and c.green() == 0x88 and c.blue() == 0x00
               for c in bgs)


def test_contrast_text_color(app):
    from PySide6.QtGui import QColor
    ed = _editor(app, "x = 1")
    dark_bg = ed._swatch_text_color(QColor(10, 10, 10))      # dunkler Hintergrund
    light_bg = ed._swatch_text_color(QColor(240, 240, 240))  # heller Hintergrund
    assert dark_bg.lightness() > 200      # auf Dunkel -> helle Schrift
    assert light_bg.lightness() < 80      # auf Hell -> dunkle Schrift


def test_cache_updates_on_edit(app):
    ed = _editor(app, "col = &HFF8800")
    assert len(ed._color_literals) == 1
    ed.set_text("a = 1\nb = 2")          # keine Literale mehr
    app.processEvents()
    assert ed._color_literals == []
    ed.set_text("p = &H102030 : q = &H405060")
    app.processEvents()
    assert len(ed._color_literals) == 2


def test_rgba_and_hexa_detected(app):
    ed = _editor(app, "\n".join([
        "a = RGBA(0, 128, 255, 64)",
        "b = &H80FF8800",          # AARRGGBB
        "c = RGB(10, 20, 30)",
        "d = &HFF8800",
    ]))
    by_kind = {kind: (col, s, e) for s, e, col, kind in ed._color_literals}
    assert set(by_kind) == {"rgba", "hexa", "rgb", "hex"}
    # RGBA: RGB-Teil korrekt, Alpha im QColor erhalten (fuers Editieren).
    col, _s, _e = by_kind["rgba"]
    assert (col.red(), col.green(), col.blue(), col.alpha()) == (0, 128, 255, 64)
    # &H80FF8800 -> alpha 0x80, rgb FF8800
    colh, _s, _e = by_kind["hexa"]
    assert (colh.red(), colh.green(), colh.blue(), colh.alpha()) == (255, 136, 0, 0x80)


def test_rgba_swatch_background_is_opaque(app):
    # Auch bei niedrigem Alpha soll der Editor die Farbe DECKEND zeigen.
    from PySide6.QtWidgets import QTextEdit
    ed = _editor(app, "x = RGBA(0, 128, 255, 20)")
    found = None
    for sel in ed.extraSelections():
        bg = sel.format.background().color()
        if (bg.red(), bg.green(), bg.blue()) == (0, 128, 255):
            found = bg
            break
    assert found is not None, "kein Hintergrund fuer das RGBA-Literal"
    assert found.alpha() == 255      # deckend gezeichnet


def test_rgba_clickable_kind(app):
    ed = _editor(app, "x = RGBA(0, 128, 255, 64)")
    start, end, _c, _k = ed._color_literals[0]
    cur = ed.textCursor(); cur.setPosition((start + end) // 2)
    hit = ed._swatch_at(ed.cursorRect(cur).center())
    assert hit is not None and hit[3] == "rgba"


def test_clickable_opens_picker(app):
    # Klick mitten auf das Literal muss den Farbwaehler-Treffer liefern.
    ed = _editor(app, "col = &HFF8800")
    start, end, _c, _k = ed._color_literals[0]
    mid = (start + end) // 2
    cur = ed.textCursor()
    cur.setPosition(mid)
    pos = ed.cursorRect(cur).center()
    hit = ed._swatch_at(pos)
    assert hit is not None
    _s, _e, color, kind = hit
    assert kind == "hex"
    assert color.red() == 0xFF and color.green() == 0x88 and color.blue() == 0x00
