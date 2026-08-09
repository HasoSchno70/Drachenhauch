"""Tests fuer die farbige Minimap (Code-Uebersicht rechts).

Die Minimap zerlegt jede Zeile in dieselben Token-Klassen wie der Editor-
Highlighter (`highlighter.line_color_spans`) und zeichnet pro Token einen
farbigen Strich -- so spiegelt die Uebersicht die Code-Farben.
"""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

pytest.importorskip("PySide6")


@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


def _minimap(app, text):
    from PySide6.QtWidgets import QPlainTextEdit
    from drachenhauch.editor_qt.minimap import Minimap
    ed = QPlainTextEdit()
    ed.setPlainText(text)
    mm = Minimap(ed)
    mm._rebuild_lines()
    return mm


def _keys(segs):
    return {k for (_s, _l, k) in segs}


def test_segments_are_classified_like_highlighter(app):
    mm = _minimap(app, "\n".join([
        'FOR i = 0 TO 9',
        'PRINT "hallo"   \' Kommentar',
        'DIM x AS INTEGER',
    ]))
    assert "ctrl" in _keys(mm._segs[0])      # FOR/TO
    line1 = _keys(mm._segs[1])
    assert "string" in line1                  # "hallo"
    assert "comment" in line1                 # ' Kommentar
    line2 = _keys(mm._segs[2])
    assert "decl" in line2 and "type" in line2  # DIM/AS + INTEGER


def test_blank_lines_have_no_segments(app):
    mm = _minimap(app, "PRINT 1\n\n   \nPRINT 2\n")
    assert mm._segs[1] == []
    assert mm._segs[2] == []


def test_whitespace_runs_removed(app):
    # Fuehrende Einrueckung erzeugt KEIN Segment, verschiebt aber die Startspalte.
    mm = _minimap(app, "        PRINT 1")
    segs = mm._segs[0]
    assert segs, "Zeile sollte sichtbare Token haben"
    # Erstes Segment beginnt erst nach der Einrueckung (Spalte 8).
    assert segs[0][0] >= 8


def test_large_file_falls_back_to_monochrome(app):
    from drachenhauch.editor_qt.minimap import COLOR_MAX_LINES
    big = "\n".join("PRINT 1" for _ in range(COLOR_MAX_LINES + 5))
    mm = _minimap(app, big)
    # Fallback: pro nicht-leerer Zeile genau ein neutrales (key=None) Segment.
    sample = mm._segs[0]
    assert len(sample) == 1 and sample[0][2] is None
