"""Konstruktions-Smoke-Tests fuer die Form-Designer-Qt-UI (offscreen).
Faengt Import-/API-/Wiring-Fehler; Interaktion (Maus) ist nicht abgedeckt.
"""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
pytest.importorskip("PySide6")

from PySide6.QtWidgets import QApplication   # noqa: E402
from PySide6.QtGui import QDropEvent, QKeyEvent, QMouseEvent   # noqa: E402
from PySide6.QtCore import Qt, QPointF, QMimeData, QEvent   # noqa: E402

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


# --------------------------------------------------------------- Rendering
def test_canvas_renders_all_kinds(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    from gamebasic.formdesigner import PALETTE
    d = win.canvas.doc
    y = 5
    for sp in PALETTE:
        c = d.add(sp.kind, 5, y); y += 20
        if sp.has_items:
            c.items = ["a", "b", "c"]; c.sel = 1
    # Sonderzustaende mit abdecken
    d.add("checkbox", 5, y).checked = True
    d.add("slider", 60, y).value = 50.0
    p = d.add("progress", 120, y); p.value = 0.7
    d.add("button", 5, y + 20).enabled = False
    d.add("label", 60, y + 20).visible = False
    win.canvas._select(d.controls[0])
    pm = win.canvas.grab()                  # voller Paint-Zyklus, darf nicht crashen
    assert not pm.isNull()
    win.close()


# --------------------------------------------------------------- Fenster-Inspector
def test_window_inspector_shown_when_nothing_selected(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    assert win._insp_stack.currentWidget() is win.win_inspector
    assert win.win_inspector.doc is win.canvas.doc
    b = win.canvas.doc.add("button", 10, 10)
    win.canvas._select(b)
    assert win._insp_stack.currentWidget() is win.inspector     # Control -> Control-Inspector
    win.canvas._select(None)
    assert win._insp_stack.currentWidget() is win.win_inspector # zurueck zum Fenster
    win.close()


def test_inspector_anchor_checkboxes(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    c = win.canvas.doc.add("button", 10, 10); c.anchor = "lrtb"
    win.canvas._select(c)
    ins = win.inspector
    assert ins.a_l.isChecked() and ins.a_r.isChecked()
    assert ins.a_t.isChecked() and ins.a_b.isChecked()
    ins.a_r.setChecked(False)                 # Aenderung -> schreibt c.anchor
    assert c.anchor == "ltb"
    win.close()


def test_window_inspector_edits_doc(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    win.win_inspector.title.setText("Mein Fenster")
    win.win_inspector.resizable.setChecked(True)
    win.win_inspector.sw.setValue(420)
    assert win.canvas.doc.title == "Mein Fenster"
    assert win.canvas.doc.resizable and win.canvas.doc.w == 420
    assert win.history.can_undo                                 # Fenster-Edit undobar
    win.close()


def test_form_resize_handle(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv.doc.w, cv.doc.h = 360, 260
    cv._select(None)
    se = (PAD + cv.doc.w, PAD + cv.doc.h)
    assert cv._form_handle_at(QPointF(*se).toPoint()) == "se"   # Griff-Treffer
    cv._form_resize = "se"                        # Resize-Logik direkt treiben
    cv.mouseMoveEvent(QMouseEvent(QEvent.Type.MouseMove,
        QPointF(PAD + 440, PAD + 300), _NB, _L, _NO))
    _mrelease(cv)
    assert cv.doc.w > 360 and cv.doc.h > 260
    assert win.win_inspector.sw.value() == cv.doc.w
    win.close()


def test_form_resize_respects_min(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv.doc.w, cv.doc.h, cv.doc.min_w = 360, 260, 300
    cv._select(None)
    cv._form_resize = "e"
    cv.mouseMoveEvent(QMouseEvent(QEvent.Type.MouseMove,
        QPointF(PAD + 100, PAD + 130), _NB, _L, _NO))   # ganz klein ziehen
    _mrelease(cv)
    assert cv.doc.w >= 300                       # Min-Breite haelt
    win.close()


# --------------------------------------------------------------- Multi-Select
_L = Qt.MouseButton.LeftButton
_NB = Qt.MouseButton.NoButton
_NO = Qt.KeyboardModifier.NoModifier
_CTRL = Qt.KeyboardModifier.ControlModifier


def _mpress(cv, cx, cy, ctrl=False):
    cv.mousePressEvent(QMouseEvent(QEvent.Type.MouseButtonPress,
        QPointF(PAD + cx, PAD + TITLE_H + cy), _L, _L, _CTRL if ctrl else _NO))


def _mmove(cv, cx, cy):
    cv.mouseMoveEvent(QMouseEvent(QEvent.Type.MouseMove,
        QPointF(PAD + cx, PAD + TITLE_H + cy), _NB, _L, _NO))


def _mrelease(cv):
    cv.mouseReleaseEvent(QMouseEvent(QEvent.Type.MouseButtonRelease,
        QPointF(0, 0), _L, _NB, _NO))


def _has(cv, c):
    return any(x is c for x in cv.selection)


def test_ctrl_click_multi_select(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    b1 = cv.doc.add("button", 16, 16)
    b2 = cv.doc.add("button", 16, 64)
    _mpress(cv, b1.x + 50, b1.y + 14, ctrl=True); _mrelease(cv)
    _mpress(cv, b2.x + 50, b2.y + 14, ctrl=True); _mrelease(cv)
    assert len(cv.selection) == 2 and _has(cv, b1) and _has(cv, b2)
    assert "2 Controls" in win._status.text()
    win.close()


def test_group_drag_moves_all(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    b1 = cv.doc.add("button", 16, 16)
    b2 = cv.doc.add("button", 16, 64)
    cv._select_many([b1, b2])
    _mpress(cv, b1.x + 50, b1.y + 14)                # auf b1 -> Gruppen-Drag
    _mmove(cv, b1.x + 50 + 10, b1.y + 14 + 10)       # Delta (10,10) -> Raster 8
    _mrelease(cv)
    assert (b1.x, b1.y) == (24, 24) and (b2.x, b2.y) == (24, 72)
    assert win.history.can_undo
    win.close()


def test_multi_delete(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    b1 = cv.doc.add("button", 16, 16)
    b2 = cv.doc.add("button", 16, 64)
    b3 = cv.doc.add("button", 16, 112)
    cv._select_many([b1, b2])
    cv.keyPressEvent(QKeyEvent(QEvent.Type.KeyPress, Qt.Key.Key_Delete, _NO))
    assert len(cv.doc.controls) == 1 and cv.doc.controls[0] is b3
    win.close()


def test_rubber_band_select(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    b1 = cv.doc.add("button", 16, 16)
    b2 = cv.doc.add("button", 16, 64)
    _mpress(cv, 220, 220)           # leerer Bereich -> Band starten
    _mmove(cv, 0, 0)                # Rahmen ueberdeckt beide Buttons
    _mrelease(cv)
    assert _has(cv, b1) and _has(cv, b2)
    win.close()


# --------------------------------------------------------------- Ausrichtung
def test_align_snaps_left_edge_and_shows_guide(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    a = cv.doc.add("button", 100, 20)            # linke Kante x=100
    b = cv.doc.add("button", 0, 80)
    cv._select(b)
    cv._move_to(103, 80)                          # nah an a.x=100 (innerhalb 6)
    assert b.x == 100                             # an a's linke Kante gefangen
    assert 100 in cv._guides_v                    # Hilfslinie aktiv
    assert a is not b
    win.close()


def test_align_center(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv.doc.w = 500                               # /2 = 250, weit weg von 150
    a = cv.doc.add("button", 100, 20)            # Mitte bei 150 (100+100/2)
    b = cv.doc.add("button", 0, 80); b.w = 40    # nur die Mitte kann fangen
    cv._select(b)
    cv._move_to(132, 80)                          # Mitte=152, Ziel 150 -> snap
    assert b.x + b.w // 2 == 150                  # an a's Mitte ausgerichtet
    assert 150 in cv._guides_v
    win.close()


def test_guides_cleared_on_release(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv.doc.add("button", 100, 20)
    b = cv.doc.add("button", 0, 80)
    cv._select(b)
    cv._move_to(103, 80)
    assert cv._guides_v
    cv._clear_guides()
    assert not cv._guides_v and not cv._guides_h
    win.close()


# --------------------------------------------------------------- Zoom
def test_zoom_levels_and_clamp(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv.set_zoom(2.0)
    assert cv.zoom == 2.0 and win._zoom_lbl.text() == "200 %"
    cv.set_zoom(99.0); assert cv.zoom == 4.0       # max-Clamp
    cv.set_zoom(0.01); assert cv.zoom == 0.25      # min-Clamp
    win.close()


def test_zoom_coordinate_mapping(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv.set_zoom(2.0)
    wx, wy = (PAD + 40) * 2, (PAD + TITLE_H + 40) * 2
    assert cv._to_ctrl(cv._to_draw(QPointF(wx, wy))) == (40, 40)
    # minimumSize waechst mit dem Zoom (Scrollbarkeit)
    cv.set_zoom(1.0); m1 = cv.minimumWidth()
    cv.set_zoom(2.0); assert cv.minimumWidth() > m1
    win.close()


def test_drop_under_zoom(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv.set_zoom(2.0)
    md = QMimeData(); md.setData(_CONTROL_MIME, b"button")
    pos = QPointF((PAD + 40) * 2, (PAD + TITLE_H + 40) * 2)
    ev = QDropEvent(pos, Qt.DropAction.CopyAction, md,
                    Qt.MouseButton.LeftButton, Qt.KeyboardModifier.NoModifier)
    cv.dropEvent(ev)
    c = cv.doc.controls[-1]
    assert (c.x, c.y) == (40, 40)                  # Drop trotz Zoom korrekt platziert
    win.close()


# --------------------------------------------------------------- Edit-UX
def _press(cv, key, mod=Qt.KeyboardModifier.NoModifier):
    cv.keyPressEvent(QKeyEvent(QEvent.Type.KeyPress, key, mod))


def test_nudge_coalesces_to_one_undo(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    b = cv.doc.add("button", 40, 40)
    cv._select(b)
    u0 = len(win.history._undo)
    _press(cv, Qt.Key.Key_Right); _press(cv, Qt.Key.Key_Right); _press(cv, Qt.Key.Key_Down)
    assert (b.x, b.y) == (42, 41)
    assert len(win.history._undo) == u0 + 1          # ganzer Burst = 1 Schritt
    _press(cv, Qt.Key.Key_Right, Qt.KeyboardModifier.ShiftModifier)
    assert b.x == 50                                  # Shift = GRID-Schritt
    win.undo()
    assert (cv.doc.controls[0].x, cv.doc.controls[0].y) == (40, 40)
    win.close()


def test_duplicate_copy_paste(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv._select(cv.doc.add("button", 10, 10))
    win.duplicate_selected()
    assert len(cv.doc.controls) == 2 and cv.selected is cv.doc.controls[-1]
    cv._select(cv.doc.controls[0])
    win.copy_selected(); win.paste_clip()
    assert len(cv.doc.controls) == 3
    assert win.history.can_undo
    win.close()


def test_z_order_actions(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    a = cv.doc.add("button", 0, 0)
    b = cv.doc.add("button", 0, 0)           # ueberlappt, oben
    cv._select(a)
    win.raise_selected()
    assert cv.doc.control_at(5, 5) is a
    win.lower_selected()
    assert cv.doc.control_at(5, 5) is b
    win.close()


def test_status_readout(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv._select(cv.doc.add("button", 12, 34))
    assert "x=12" in win._status.text() and "100×28" in win._status.text()
    cv._select(None)
    assert win._status.text() == ""
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
