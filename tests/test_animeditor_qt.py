"""Konstruktions-/Wiring-Smoke-Tests fuer die dhanim-Qt-UI (offscreen).
Faengt Import-/API-/Verdrahtungs-Fehler; Maus-Interaktion ist nicht abgedeckt.
"""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
pytest.importorskip("PySide6")

from PySide6.QtWidgets import QApplication   # noqa: E402

from drachenhauch.animeditor import AnimDoc, Condition, Transition, State  # noqa: E402
from drachenhauch.animeditor_qt import AnimEditor, _GraphCanvas, _transition_label  # noqa: E402


def _app():
    return QApplication.instance() or QApplication([])


@pytest.fixture(autouse=True)
def _no_blocking_close_dialog(monkeypatch):
    """`AnimEditor.closeEvent` fragt jetzt bei ungespeicherten Aenderungen
    nach (Speichern/Verwerfen/Abbrechen) -- ohne Mock wuerde das echte
    `QMessageBox.question()` in den meisten Tests hier (die `win.close()`
    nach einer dirty-machenden Aktion aufrufen, aber das Dialog-Verhalten
    selbst nicht testen wollen) unter offscreen-Qt auf ein nie kommendes
    Nutzer-Event warten -> Test haengt. Default: "Verwerfen" (Discard),
    wie es die einzelnen Dialog-spezifischen Tests unten explizit selbst
    ueberschreiben."""
    from PySide6.QtWidgets import QMessageBox
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.No)


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


def test_state_inspector_rename_is_undoable(tmp_path):
    """Regression: `_StateInspector` frueher ohne `before_mutation` -> der
    History-Snapshot landete NACH der Mutation (falscher Zeitpunkt), Undo
    war entweder ein No-Op oder entfernte den falschen Eintrag."""
    _app()
    win = AnimEditor(tmp_path)
    s = win.canvas.doc.add_state(0, 0, "idle")
    win.canvas._select(s)
    win.state_insp.name.setText("stand")
    win.state_insp._apply_name()
    assert win.canvas.doc.state_by_name("stand") is not None
    win.undo()
    assert win.canvas.doc.state_by_name("idle") is not None
    assert win.canvas.doc.state_by_name("stand") is None
    win.close()


def test_state_inspector_frame_range_is_undoable(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    s = win.canvas.doc.add_state(0, 0, "idle")
    win.canvas._select(s)
    win.state_insp.set_state(win.canvas.doc, s)
    # setValue() loest ueber valueChanged bereits automatisch _apply() aus --
    # ein weiterer manueller _apply()-Aufruf wuerde eine dritte (ueberzaehlige)
    # Mutation simulieren. Nur EIN Feld aendern = genau EIN Undo-Schritt,
    # wie bei einer echten Nutzer-Eingabe in genau dieses Spinbox.
    win.state_insp.first.setValue(5)
    assert s.first == 5
    win.undo()
    # undo() ersetzt win.canvas.doc komplett durch einen neu deserialisierten
    # Snapshot -- `s` ist danach ein Objekt aus dem ALTEN Doc, nicht mehr Teil
    # des live-Dokuments. Frisch nachschlagen statt die stale Referenz pruefen.
    restored = win.canvas.doc.state_by_name("idle")
    assert restored.first == 0
    win.close()


def test_transition_inspector_add_cond_is_single_undo_step(tmp_path):
    """Regression: `_add_cond`/`remove_cond` emittierten `changed` zweimal
    (vor UND nach der Mutation) als impliziten Snapshot-Trick -> ein einzelnes
    Hinzufuegen brauchte zwei Strg+Z. Jetzt: genau ein `before_mutation`."""
    _app()
    win = AnimEditor(tmp_path)
    win.canvas.doc.add_state(0, 0, "idle")
    win.canvas.doc.add_state(120, 0, "run")
    win.canvas.doc.add_param("speed", "float")
    t = win.canvas.doc.add_transition("idle", "run")
    win.canvas._select(t)
    win.trans_insp._add_cond()
    assert len(t.conditions) == 1
    win.undo()
    # Wie oben: nach undo() ist win.canvas.doc ein neues Objekt -> `t` ist stale.
    assert len(win.canvas.doc.transitions[0].conditions) == 0
    win.close()


def test_rejected_rename_does_not_pollute_undo_stack(tmp_path, monkeypatch):
    """Regression: `_rename_dialog`/Transition-Drop emittierten
    `before_mutation` VOR dem Versuch -- ein abgelehnter (Duplikat-)Name
    liess trotzdem einen wirkungslosen Snapshot auf dem Undo-Stack."""
    from PySide6.QtWidgets import QInputDialog
    _app()
    win = AnimEditor(tmp_path)
    win.canvas.doc.add_state(0, 0, "idle")
    win.canvas.doc.add_state(0, 0, "run")
    before = win.history.can_undo
    monkeypatch.setattr(QInputDialog, "getText", lambda *a, **k: ("run", True))  # Duplikat
    win.canvas._rename_dialog("idle")
    assert win.canvas.doc.state_by_name("idle") is not None   # abgelehnt, unveraendert
    assert win.history.can_undo == before
    win.close()


def test_close_event_prompts_when_dirty(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QMessageBox
    _app()
    win = AnimEditor(tmp_path)
    win.add_state_center()
    assert win.dirty is True
    called = []
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: called.append(1) or QMessageBox.StandardButton.Cancel)
    from PySide6.QtGui import QCloseEvent
    ev = QCloseEvent()
    win.closeEvent(ev)
    assert called == [1]
    assert not ev.isAccepted()


def test_new_doc_discards_dirty_only_after_confirmation(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QMessageBox
    _app()
    win = AnimEditor(tmp_path)
    win.add_state_center()
    assert win.dirty is True
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.No)   # verwerfen
    win.new_doc()
    assert len(win.canvas.doc.states) == 0
    win.close()


def test_save_load_roundtrip(tmp_path):
    _app()
    win = AnimEditor(tmp_path)
    win.canvas.doc.add_state(0, 0, "idle")
    win.canvas.doc.add_state(120, 0, "run")
    win.canvas.doc.add_transition("idle", "run")
    p = tmp_path / "x.dhanim"
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
