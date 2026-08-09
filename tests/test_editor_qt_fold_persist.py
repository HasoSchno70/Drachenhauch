"""Tests fuer Folding-State-Persistierung im Editor."""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


SRC = """SUB foo()
    PRINT 1
END SUB
SUB bar()
    PRINT 2
END SUB
"""


def test_folded_starts_initially_empty():
    from drachenhauch.editor_qt.code_editor import CodeEditor
    ed = CodeEditor()
    ed.set_text(SRC)
    assert ed.folded_starts() == []


def test_apply_then_get_roundtrip():
    from drachenhauch.editor_qt.code_editor import CodeEditor
    ed = CodeEditor()
    ed.set_text(SRC)
    ed.apply_folded_starts([1, 4])
    assert ed.folded_starts() == [1, 4]


def test_apply_ignores_invalid_starts():
    """Wenn die gespeicherte Start-Zeile nicht mehr zu einem Block
    passt (Datei extern bearbeitet), wird sie ignoriert -- kein Crash."""
    from drachenhauch.editor_qt.code_editor import CodeEditor
    ed = CodeEditor()
    ed.set_text(SRC)
    ed.apply_folded_starts([1, 999])  # 999 ist kein gueltiger Start
    assert ed.folded_starts() == [1]


def test_apply_empty_is_noop():
    from drachenhauch.editor_qt.code_editor import CodeEditor
    ed = CodeEditor()
    ed.set_text(SRC)
    ed.apply_folded_starts([])
    assert ed.folded_starts() == []


def test_unfold_all_clears_state():
    from drachenhauch.editor_qt.code_editor import CodeEditor
    ed = CodeEditor()
    ed.set_text(SRC)
    ed.apply_folded_starts([1, 4])
    ed.unfold_all()
    assert ed.folded_starts() == []
