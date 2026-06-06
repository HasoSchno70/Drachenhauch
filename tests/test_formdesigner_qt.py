"""Konstruktions-Smoke-Tests fuer die Form-Designer-Qt-UI (offscreen).
Faengt Import-/API-/Wiring-Fehler; Interaktion (Maus) ist nicht abgedeckt.
"""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
pytest.importorskip("PySide6")

from PySide6.QtWidgets import QApplication   # noqa: E402
from PySide6.QtGui import QDropEvent   # noqa: E402
from PySide6.QtCore import Qt, QPointF, QMimeData   # noqa: E402

from gamebasic.formdesigner_qt import (   # noqa: E402
    FormDesigner, _palette_icon, _CONTROL_MIME, PAD, TITLE_H,
)


def _app():
    return QApplication.instance() or QApplication([])


def test_construct_and_palette(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    assert win.canvas.doc is not None
    assert win.palette.count() >= 12          # alle Palette-Eintraege
    win.close()


def test_select_updates_inspector(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    c = win.canvas.doc.add("button", 10, 10)
    win.canvas._select(c)
    assert win.inspector._c is c
    win.close()


def test_inspector_apply_updates_control(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    c = win.canvas.doc.add("button", 10, 10)
    win.canvas._select(c)
    win.inspector.text.setText("Hallo")
    win.inspector.on_click.setText("on_ok")
    win.inspector.sx.setValue(42)
    win.inspector._apply()
    assert c.text == "Hallo" and c.on_click == "on_ok" and c.x == 42
    win.close()


def test_save_load_roundtrip(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    win.canvas.doc.title = "MyForm"
    win.canvas.doc.add("dropdown", 20, 20).items = ["a", "b"]
    p = tmp_path / "f.gbform"
    win.canvas.doc.save(str(p))
    win2 = FormDesigner(tmp_path)
    from gamebasic.formdesigner import FormDoc
    win2.canvas.set_doc(FormDoc.load(str(p)))
    assert win2.canvas.doc.title == "MyForm"
    assert win2.canvas.doc.controls[0].items == ["a", "b"]
    win.close(); win2.close()


def test_undo_redo_place(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    pre = win.canvas.doc.to_dict()
    win.canvas.doc.add("button", 10, 10)
    win.canvas.commit_history(pre)
    assert len(win.canvas.doc.controls) == 1 and win.history.can_undo
    win.undo()
    assert len(win.canvas.doc.controls) == 0
    win.redo()
    assert len(win.canvas.doc.controls) == 1
    win.close()


def test_double_click_creates_handler_and_opens_code(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    b = win.canvas.doc.add("button", 10, 10)
    win.canvas.handler_requested.emit(b)        # = Doppelklick
    assert b.on_click == "btn1Click"
    assert win.code_panel.current == "btn1Click"
    assert "btn1Click" in win.canvas.doc.code
    assert win.history.can_undo                 # Handler-Erzeugung undobar
    win.close()


def test_code_edit_stored_and_coalesced(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    b = win.canvas.doc.add("button", 10, 10)
    win.canvas.handler_requested.emit(b)
    win.code_panel.editor.setPlainText('PRINT "x"')
    assert win.canvas.doc.code["btn1Click"] == 'PRINT "x"'
    depth = len(win.history._undo)
    win.code_panel.editor.setPlainText('PRINT "xy"')   # gleiche Sitzung
    assert len(win.history._undo) == depth             # kein neuer Checkpoint
    win.close()


def test_double_click_eventless_no_handler(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    lbl = win.canvas.doc.add("label", 10, 10)
    win.canvas.handler_requested.emit(lbl)
    assert lbl.on_click == "" and not win.canvas.doc.code
    win.close()


# --------------------------------------------------------------- Multi-Form
def test_add_and_switch_forms(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    assert len(win.forms) == 1                      # ein Start-Formular
    win.active.doc.title = "Main"
    from gamebasic.formdesigner import FormDoc
    win._add_open_form(FormDoc(title="Second"))
    assert len(win.forms) == 2 and win.active_index == 1
    assert win.form_list.count() == 2
    win._switch_to(0)
    assert win.canvas.doc.title == "Main"
    win.close()


def test_per_form_undo_isolation(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    from gamebasic.formdesigner import FormDoc
    win._add_open_form(FormDoc(title="B"))          # Form 1, aktiv
    pre = win.canvas.doc.to_dict()
    win.canvas.doc.add("button", 0, 0)
    win.canvas.commit_history(pre)
    assert win.history.can_undo
    win._switch_to(0)                               # andere Form
    assert not win.history.can_undo                 # Undo leckt nicht
    win.close()


def test_close_form_keeps_at_least_one(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    from gamebasic.formdesigner import FormDoc
    win._add_open_form(FormDoc(title="B"))
    win.close_form()                                # nicht dirty -> kein Dialog
    assert len(win.forms) == 1
    win.close_form()                                # letztes -> wird durch leeres ersetzt
    assert len(win.forms) == 1
    win.close()


def test_project_save_load_roundtrip(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    from gamebasic.formdesigner import FormDoc
    win.active.doc.title = "Main"
    win.active.path = tmp_path / "main.gbform"
    win._add_open_form(FormDoc(title="Settings"), tmp_path / "settings.gbform")
    win._switch_to(0)
    win.set_main_form()
    win.project_path = tmp_path / "app.gbproj"
    win.save_project()
    assert (tmp_path / "app.gbproj").exists()

    win2 = FormDesigner(tmp_path)
    win2.load_project_file(str(tmp_path / "app.gbproj"))
    assert len(win2.forms) == 2
    assert win2.active.doc.title == "Main"          # main-Formular aktiv
    assert {f.doc.title for f in win2.forms} == {"Main", "Settings"}
    win.close(); win2.close()


# --------------------------------------------------------------- Palette + DnD
def test_bigger_window(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    assert win.width() >= 1400 and win.height() >= 800
    win.close()


def test_palette_has_graphical_icons(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    assert win.palette.dragEnabled()
    for i in range(win.palette.count()):
        assert not win.palette.item(i).icon().isNull()
    from gamebasic.formdesigner import PALETTE
    for sp in PALETTE:
        assert not _palette_icon(sp.kind).isNull()
    win.close()


def test_palette_mime_carries_kind(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    it = win.palette.item(0)
    md = win.palette.mimeData([it])
    assert md.hasFormat(_CONTROL_MIME)
    assert bytes(md.data(_CONTROL_MIME)).decode() == it.data(Qt.ItemDataRole.UserRole)
    win.close()


def test_drop_places_control(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    n0 = len(cv.doc.controls)
    md = QMimeData(); md.setData(_CONTROL_MIME, b"slider")
    pos = QPointF(PAD + 40, PAD + TITLE_H + 40)        # -> Control (40, 40)
    ev = QDropEvent(pos, Qt.DropAction.CopyAction, md,
                    Qt.MouseButton.LeftButton, Qt.KeyboardModifier.NoModifier)
    cv.dropEvent(ev)
    assert len(cv.doc.controls) == n0 + 1
    c = cv.doc.controls[-1]
    assert c.kind == "slider" and (c.x, c.y) == (40, 40)
    assert win.history.can_undo and cv.selected is c   # undobar + selektiert
    win.close()


def test_export_gb_writes_file(tmp_path, monkeypatch):
    _app()
    win = FormDesigner(tmp_path)
    win.active.doc.add("button", 10, 10).on_click = "go"
    win.active.doc.code["go"] = 'PRINT "go"'
    out = tmp_path / "out.gb"
    monkeypatch.setattr("PySide6.QtWidgets.QFileDialog.getSaveFileName",
                        staticmethod(lambda *a, **k: (str(out), "")))
    win.export_gb_code()
    assert out.exists()
    txt = out.read_text(encoding="utf-8")
    assert "GUI_BUTTON(" in txt and "GUI_ON_CLICK(" in txt and "SUB go()" in txt
    win.close()
