"""Konstruktions-/Wiring-Smoke-Tests fuer die gbanim-Qt-UI (offscreen).
Faengt Import-/API-/Verdrahtungs-Fehler; Maus-Interaktion ist nicht abgedeckt.
"""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
pytest.importorskip("PySide6")

from PySide6.QtWidgets import QApplication   # noqa: E402

from gamebasic.animeditor import AnimDoc, Condition, Transition, State  # noqa: E402
from gamebasic.animeditor_qt import AnimEditor, _GraphCanvas, _transition_label  # noqa: E402


def _app():
    return QApplication.instance() or QApplication([])


def test_construct(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    assert win.canvas.doc is not None
    assert win.insp_stack.count() == 3
    win.close()


def test_add_state_and_select_shows_inspector(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    win.add_state_center()
    assert len(win.canvas.doc.states) == 1
    assert win.insp_stack.currentIndex() == 1     # State-Inspector
    win.close()


def test_state_inspector_rename(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    s = win.canvas.doc.add_state(0, 0, "idle")
    win.canvas.doc.add_state(0, 0, "run")
    win.canvas._select(s)
    win.state_insp.name.setText("stand")
    win.state_insp._apply_name()
    assert win.canvas.doc.state_by_name("stand") is not None
    assert win.canvas.doc.state_by_name("idle") is None
    win.close()


def test_transition_inspector_conditions(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    win.canvas.doc.add_state(0, 0, "idle")
    win.canvas.doc.add_state(120, 0, "run")
    win.canvas.doc.add_param("speed", "float")
    t = win.canvas.doc.add_transition("idle", "run")
    win.canvas._select(t)
    assert win.insp_stack.currentIndex() == 2     # Transition-Inspector
    win.trans_insp._add_cond()
    assert len(t.conditions) == 1
    assert t.conditions[0].param == "speed"
    win.close()


def test_param_panel_add_remove(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    win.param_panel._add()
    assert len(win.canvas.doc.params) == 1
    win.param_panel.list.setCurrentRow(0)
    win.param_panel._remove()
    assert len(win.canvas.doc.params) == 0
    win.close()


def test_undo_redo(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    win.add_state_center()
    assert len(win.canvas.doc.states) == 1
    win.undo()
    assert len(win.canvas.doc.states) == 0
    win.redo()
    assert len(win.canvas.doc.states) == 1
    win.close()


def test_save_load_roundtrip(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    win.canvas.doc.add_state(0, 0, "idle")
    win.canvas.doc.add_state(120, 0, "run")
    win.canvas.doc.add_transition("idle", "run")
    p = tmp_path / "x.gbanim"
    win.path = p
    win.save_doc()
    assert p.exists()

    win2 = AnimEditor(tmp_path)
    win2._set_doc(AnimDoc.load(str(p)))
    assert [s.name for s in win2.canvas.doc.states] == ["idle", "run"]
    win.close(); win2.close()


def test_canvas_hit_testing(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    from PySide6.QtCore import QPoint
    s = win.canvas.doc.add_state(100, 100, "idle")
    assert win.canvas._node_at(QPoint(110, 110)) == "idle"
    assert win.canvas._node_at(QPoint(5, 5)) is None
    win.close()


def test_transition_label():
    t = Transition("idle", "run", conditions=[Condition("speed", "gt", 5.0)])
    assert "speed" in _transition_label(t)
    t2 = Transition("jump", "idle", wait_finished=True, conditions=[])
    assert _transition_label(t2) == "fertig"
