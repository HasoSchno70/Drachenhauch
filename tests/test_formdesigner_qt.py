"""Konstruktions-Smoke-Tests fuer die Form-Designer-Qt-UI (offscreen).
Faengt Import-/API-/Wiring-Fehler; Interaktion (Maus) ist nicht abgedeckt.
"""
import json
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
pytest.importorskip("PySide6")

from PySide6.QtWidgets import QApplication   # noqa: E402
from PySide6.QtGui import QCloseEvent, QDropEvent, QKeyEvent, QMouseEvent   # noqa: E402
from PySide6.QtCore import Qt, QPointF, QMimeData, QEvent   # noqa: E402

from gamebasic.formdesigner_qt import (   # noqa: E402
    FormDesigner, _Inspector, _palette_icon, _CONTROL_MIME, PAD, TITLE_H,
)


def _app():
    return QApplication.instance() or QApplication([])


@pytest.fixture(autouse=True)
def _no_modal_dialogs(monkeypatch):
    """Kein Test darf einen modalen Dialog oeffnen -- `exec()` im geteilten
    pytest-Prozess haelt den Lauf an (bzw. segfaultet). `win.close()` fragt
    jetzt bei ungespeicherten Formularen nach, also standardmaessig
    "Verwerfen" antworten. Tests, die den Dialog selbst pruefen, patchen
    `QMessageBox.question` danach erneut."""
    from PySide6.QtWidgets import QMessageBox
    monkeypatch.setattr(QMessageBox, "question",
                        staticmethod(lambda *a, **k: QMessageBox.StandardButton.Discard))
    monkeypatch.setattr(QMessageBox, "warning", staticmethod(lambda *a, **k: None))
    monkeypatch.setattr(QMessageBox, "critical", staticmethod(lambda *a, **k: None))


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
def test_rulers_toggle_and_paint(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    assert win.canvas.show_rulers
    c = win.canvas.doc.add("button", 10, 10)
    win.canvas._select_many([c])
    assert not win.canvas.grab().isNull()        # Paint mit Linealen + Highlight
    win.act_rulers.setChecked(False)
    assert not win.canvas.show_rulers
    assert not win.canvas.grab().isNull()        # Paint ohne Lineale
    win.close()


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


def test_inspector_color_and_font(tmp_path, monkeypatch):
    _app()
    win = FormDesigner(tmp_path)
    c = win.canvas.doc.add("label", 10, 10)
    win.canvas._select(c)
    ins = win.inspector
    ins.sfont.setValue(20)                       # Schriftgroesse
    assert c.font_size == 20
    from PySide6.QtGui import QColor
    monkeypatch.setattr("PySide6.QtWidgets.QColorDialog.getColor",
                        staticmethod(lambda *a, **k: QColor(255, 0, 0)))
    ins._pick_color()                            # Farb-Waehler
    assert c.color == 0xFF0000
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


def test_arrange_align_and_undo(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    a = cv.doc.add("button", 10, 10)
    b = cv.doc.add("button", 50, 80)
    cv._select_many([a, b])
    win._align("left")
    assert a.x == 10 and b.x == 10
    assert win.history.can_undo
    win.undo()                               # Undo ersetzt die Control-Objekte
    assert cv.doc.controls[1].x == 50        # zurueck (via Doc, nicht alte Referenz)
    win.close()


def test_arrange_same_size_uses_primary(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    a = cv.doc.add("button", 0, 0); a.w, a.h = 60, 20
    b = cv.doc.add("button", 0, 40); b.w, b.h = 120, 40   # zuletzt -> primary
    cv._select_many([a, b])
    assert cv.selected is b
    win._same_size("both")
    assert (a.w, a.h) == (120, 40)           # an primary angeglichen
    win.close()


def test_arrange_toolbar_icons_and_trigger(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    acts = win.arrange_bar.actions()
    icon_acts = [a for a in acts if not a.isSeparator()]
    assert len(icon_acts) == 11                  # 6 align + 3 same + 2 distribute
    assert all(not a.icon().isNull() for a in icon_acts)
    # Toolbar-Button feuert dieselbe Logik wie das Menue
    cv = win.canvas
    a = cv.doc.add("button", 10, 10)
    b = cv.doc.add("button", 60, 70)
    cv._select_many([a, b])
    icon_acts[0].trigger()                       # Linksbuendig
    assert a.x == b.x == 10
    win.close()


def test_arrange_needs_selection(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    a = win.canvas.doc.add("button", 7, 7)
    win.canvas._select(a)
    win._align("left")                       # nur 1 selektiert -> no-op
    assert a.x == 7 and not win.history.can_undo
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


# ------------------------------------------------- Canvas-Interaktion (Review)
_R = Qt.MouseButton.RightButton


def _mhover(cv, cx, cy):
    """Mausbewegung OHNE gedrueckte Taste."""
    cv.mouseMoveEvent(QMouseEvent(QEvent.Type.MouseMove,
        QPointF(PAD + cx, PAD + TITLE_H + cy), _NB, _NB, _NO))


def test_small_control_can_be_dragged_not_resized(tmp_path):
    # Die 8 Griff-Trefferzonen ueberdeckten ein 16x16-Control (Checkbox/Radio =
    # Palette-Default) vollstaendig -- jeder Zieh-Versuch hat es auf 8x8
    # geschrumpft statt es zu verschieben.
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    c = cv.doc.add("checkbox", 48, 48)
    assert (c.w, c.h) == (16, 16)
    cv._select(c)
    _mpress(cv, c.x + c.w // 2, c.y + c.h // 2)       # Mitte = Verschieben
    assert cv._resize_handle is None and cv._drag
    _mmove(cv, c.x + c.w // 2 + 40, c.y + c.h // 2 + 40)
    _mrelease(cv)
    assert (c.w, c.h) == (16, 16)                     # Groesse unveraendert
    assert (c.x, c.y) != (48, 48)                     # aber verschoben
    win.close()


def test_small_control_corner_still_resizes(tmp_path):
    # Der Innenbereich gehoert dem Verschieben -- die Ecke muss trotzdem greifen.
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    c = cv.doc.add("checkbox", 48, 48)
    cv._select(c)
    _mpress(cv, c.x + c.w, c.y + c.h)                 # exakt der SE-Griff
    assert cv._resize_handle == "se"
    win.close()


def test_hover_after_interrupted_drag_does_not_move(tmp_path):
    # Verschluckt etwas das Release (modales Kontextmenue, Fokusverlust), blieb
    # `_drag` gesetzt und jede Mausbewegung schleppte das Control mit.
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    c = cv.doc.add("button", 16, 16)
    cv._select(c)
    _mpress(cv, 66, 30)   # Control-Mitte
    _mmove(cv, 106, 70)
    pos = (c.x, c.y)
    _mhover(cv, 200, 150)                             # nur Hovern, keine Taste
    assert (c.x, c.y) == pos
    assert not cv._drag and cv._pending is None
    win.close()


def test_right_click_keeps_multi_selection(tmp_path):
    # Rechtsklick auf ein Gruppenmitglied warf die Auswahl weg -- das
    # Kontextmenue-"Loeschen" erwischte danach nur ein Control von fuenf.
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    a = cv.doc.add("button", 16, 16)
    b = cv.doc.add("button", 16, 60)
    cv._select_many([a, b])
    # Signal blocken: `_show_context_menu` wuerde `menu.exec()` modal oeffnen
    # und den Testprozess anhalten.
    cv.blockSignals(True)
    cv.contextMenuEvent(type("E", (), {
        "pos": lambda self=None: QPointF(PAD + 20, PAD + TITLE_H + 20).toPoint(),
        "globalPos": lambda self=None: QPointF(0, 0).toPoint()})())
    cv.blockSignals(False)
    assert len(cv.selection) == 2 and cv.selected is a
    win.close()


def test_right_button_does_not_place_or_band(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    cv.place_kind = "button"
    cv.mousePressEvent(QMouseEvent(QEvent.Type.MouseButtonPress,
        QPointF(PAD + 40, PAD + TITLE_H + 40), _R, _R, _NO))
    assert len(cv.doc.controls) == 0 and cv.place_kind == "button"
    assert not cv._band
    win.close()


def test_selection_ops_use_identity_not_equality(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    a = cv.doc.add("button", 16, 16)
    b = cv.doc.add("button", 16, 16)
    a.name = b.name = ""                              # feldgleich
    assert a == b and a is not b
    cv._select_many([a, b])
    cv._toggle_select(b)          # b raus -- `remove()` traf vorher a (erstes gleiches)
    assert len(cv.selection) == 1 and cv.selection[0] is a
    win.close()


def test_group_drag_keeps_spacing_at_the_left_edge(tmp_path):
    # Der 0-Clamp wirkte pro Control: die Gruppe wurde am Rand zusammengedrueckt.
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    a = cv.doc.add("button", 8, 8)
    b = cv.doc.add("button", 80, 8)
    gap = b.x - a.x
    cv._select_many([a, b])
    _mpress(cv, 12, 12)
    _mmove(cv, -200, 12)                              # weit ueber den Rand
    _mrelease(cv)
    assert a.x == 0 and b.x - a.x == gap
    win.close()


def test_resize_never_produces_negative_coordinates(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    c = cv.doc.add("button", 100, 100)
    cv._select(c)
    _mpress(cv, c.x, c.y)                             # NW-Griff
    assert cv._resize_handle == "nw"
    _mmove(cv, -80, -70)                              # ueber die Formularkante
    _mrelease(cv)
    assert c.x >= 0 and c.y >= 0 and c.w >= 1 and c.h >= 1
    win.close()


def test_control_stays_inside_the_form(tmp_path):
    # Weit nach rechts/unten gezogen lag das Control ausserhalb der
    # Zeichenflaeche: unsichtbar, nicht anklickbar, so gespeichert.
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    c = cv.doc.add("button", 16, 16)
    cv._select(c)
    _mpress(cv, 66, 30)   # Control-Mitte
    _mmove(cv, 4000, 3000)
    _mrelease(cv)
    assert c.x + c.w <= cv.doc.w
    assert c.y + c.h <= cv.doc.h - TITLE_H
    assert cv.doc.control_at(c.x + 2, c.y + 2) is c   # weiter erreichbar
    win.close()


def test_set_doc_clears_gesture_state(tmp_path):
    # Undo/Formularwechsel mitten im Ziehen: die Flags durften nicht ueberleben.
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    c = cv.doc.add("button", 16, 16)
    cv._select(c)
    _mpress(cv, 66, 30)   # Control-Mitte
    assert cv._drag
    cv.set_doc(cv.doc)
    assert not cv._drag and cv._pending is None and cv.place_kind is None
    win.close()


def test_delete_during_drag_makes_one_undo_step(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    c = cv.doc.add("button", 16, 16)
    win.history.clear()
    cv._select(c)
    _mpress(cv, 66, 30)   # Control-Mitte
    _mmove(cv, 106, 70)
    _press(cv, Qt.Key.Key_Delete)
    _mrelease(cv)
    assert len(cv.doc.controls) == 0
    assert len(win.history._undo) == 1
    win.close()


def test_progress_preview_uses_min_max_like_the_runtime(tmp_path):
    # Die Canvas las `value` roh als 0..1 -- ein Balken mit max=100/value=25
    # sah randvoll aus, lief zur Laufzeit aber bei 25 %.
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    from gamebasic.formdesigner_qt import _progress_frac
    p = cv.doc.add("progress", 10, 10)
    p.min, p.max, p.value = 0.0, 100.0, 25.0
    assert _progress_frac(p) == 0.25                   # vorher 1.0 = randvoll
    p.min, p.max, p.value = 0.0, 1.0, 0.5
    assert _progress_frac(p) == 0.5                    # 0..1 unveraendert
    p.min, p.max = 5.0, 5.0
    assert _progress_frac(p) == 0.0                    # Nullspanne, keine Division
    assert not cv.grab().isNull()                      # paintEvent laeuft
    win.close()


# ------------------------------------------- Lebenszyklus / Speichern (Review)
def _mb():
    from PySide6.QtWidgets import QMessageBox
    return QMessageBox


def test_close_is_blocked_when_the_user_cancels(tmp_path, monkeypatch):
    # closeEvent hatte gar keinen Schutz: das X verwarf alles kommentarlos.
    _app()
    win = FormDesigner(tmp_path)
    win.canvas.doc.add("button", 10, 10)
    win._mark_dirty()
    monkeypatch.setattr(_mb(), "question",
                        staticmethod(lambda *a, **k: _mb().StandardButton.Cancel))
    assert win._confirm_dirty() is False
    ev = QCloseEvent()
    win.closeEvent(ev)
    assert not ev.isAccepted()                     # Fenster bleibt offen
    monkeypatch.setattr(_mb(), "question",
                        staticmethod(lambda *a, **k: _mb().StandardButton.Discard))
    win.close()


def test_confirm_dirty_covers_all_forms_not_just_the_active(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    from gamebasic.formdesigner import FormDoc
    win._mark_dirty()                              # Formular 0 dirty
    win._add_open_form(FormDoc(title="B"))         # wechselt auf 1 (sauber)
    seen = {}

    def _q(parent, title, text, *a, **k):
        seen["text"] = text
        return _mb().StandardButton.Discard

    import unittest.mock
    with unittest.mock.patch.object(_mb(), "question", _q):
        assert win._confirm_dirty() is True
    assert "unbenannt" in seen["text"]             # das INAKTIVE Formular kam vor
    win.close()


def test_open_project_asks_before_dropping_forms(tmp_path, monkeypatch):
    _app()
    win = FormDesigner(tmp_path)
    from gamebasic.formdesigner import FormProject
    win.canvas.doc.title = "WICHTIG"
    win._mark_dirty()
    proj = tmp_path / "p.gbproj"
    FormProject(forms=[], main="").save(str(proj))
    monkeypatch.setattr(_mb(), "question",
                        staticmethod(lambda *a, **k: _mb().StandardButton.Cancel))
    win.load_project_file(str(proj))
    assert win.forms[0].doc.title == "WICHTIG"     # nichts verworfen
    monkeypatch.setattr(_mb(), "question",
                        staticmethod(lambda *a, **k: _mb().StandardButton.Discard))
    win.close()


def test_save_failure_keeps_dirty_and_does_not_take_the_path(tmp_path, monkeypatch):
    # save_form_as setzte den Pfad VOR dem Schreiben -- nach einem Fehlschlag
    # zeigte der Titel eine Datei an, die nie existiert hat.
    _app()
    win = FormDesigner(tmp_path)
    win._mark_dirty()
    bad = str(tmp_path / "gibtsnicht" / "x.gbform")
    monkeypatch.setattr("gamebasic.formdesigner_qt.QFileDialog.getSaveFileName",
                        staticmethod(lambda *a, **k: (bad, "")))
    assert win.save_form_as() is False             # kein Traceback
    assert win.path is None and win.active.dirty
    win.close()


def test_undo_back_to_saved_state_clears_the_star(tmp_path, monkeypatch):
    _app()
    win = FormDesigner(tmp_path)
    p = tmp_path / "f.gbform"
    monkeypatch.setattr("gamebasic.formdesigner_qt.QFileDialog.getSaveFileName",
                        staticmethod(lambda *a, **k: (str(p), "")))
    assert win.save_form_as() is True
    assert not win.active.dirty
    pre = win.canvas.doc.to_dict()                 # Aenderung mit Checkpoint
    win.canvas.doc.add("button", 10, 10)
    win.canvas.commit_history(pre)
    win._mark_dirty()
    assert win.active.dirty
    win.undo()
    assert not win.active.dirty                    # zurueck auf dem Dateistand
    assert "*" not in win.windowTitle()
    win.close()


def test_undo_refreshes_the_form_navigator(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    pre = win.canvas.doc.to_dict()
    win.canvas.doc.title = "NeuerTitel"
    win.canvas.commit_history(pre)
    win._mark_dirty(); win._refresh_form_list()
    assert "NeuerTitel" in win.form_list.item(0).text()
    win.undo()
    assert "NeuerTitel" not in win.form_list.item(0).text()
    win.close()


def test_open_form_twice_switches_instead_of_duplicating(tmp_path, monkeypatch):
    _app()
    win = FormDesigner(tmp_path)
    from gamebasic.formdesigner import FormDoc
    p = tmp_path / "a.gbform"
    FormDoc(title="A").save(str(p))
    monkeypatch.setattr("gamebasic.formdesigner_qt.QFileDialog.getOpenFileName",
                        staticmethod(lambda *a, **k: (str(p), "")))
    win.open_form(); n = len(win.forms)
    win.open_form()
    assert len(win.forms) == n                     # kein zweiter Puffer
    win.close()


def test_project_keeps_forms_outside_its_directory(tmp_path):
    # `relative_to` kann nicht nach oben -- der Fallback schrieb den blossen
    # Dateinamen, die Form war beim naechsten Oeffnen verschwunden.
    _app()
    proj_dir = tmp_path / "projekt"; proj_dir.mkdir()
    extern = tmp_path / "shared"; extern.mkdir()
    win = FormDesigner(proj_dir)
    from gamebasic.formdesigner import FormDoc
    win.active.doc.title = "Haupt"; win.active.path = proj_dir / "haupt.gbform"
    win._add_open_form(FormDoc(title="Extern"), extern / "extern.gbform")
    win.project_path = proj_dir / "app.gbproj"
    win.save_project()
    rel = json.loads((proj_dir / "app.gbproj").read_text(encoding="utf-8"))["forms"]
    assert any(".." in r for r in rel)             # zeigt wirklich nach draussen
    win2 = FormDesigner(proj_dir)
    win2.load_project_file(str(proj_dir / "app.gbproj"))
    assert {f.doc.title for f in win2.forms} == {"Haupt", "Extern"}
    win.close(); win2.close()


def test_missing_form_stays_in_the_manifest(tmp_path):
    # Stumm uebersprungen UND danach dauerhaft aus dem .gbproj geloescht.
    _app()
    from gamebasic.formdesigner import FormDoc, FormProject
    FormDoc(title="OK").save(str(tmp_path / "ok.gbform"))
    FormProject(forms=["ok.gbform", "weg.gbform"], main="ok.gbform").save(
        str(tmp_path / "p.gbproj"))
    win = FormDesigner(tmp_path)
    win.load_project_file(str(tmp_path / "p.gbproj"))
    assert win.unresolved == ["weg.gbform"]
    win.save_project()
    forms = json.loads((tmp_path / "p.gbproj").read_text(encoding="utf-8"))["forms"]
    assert "weg.gbform" in forms
    win.close()


def test_main_form_survives_saving_into_a_subdirectory(tmp_path):
    _app()
    sub = tmp_path / "sub"; sub.mkdir()
    win = FormDesigner(tmp_path)
    from gamebasic.formdesigner import FormDoc
    win.active.doc.title = "A"; win.active.path = sub / "a.gbform"
    win._add_open_form(FormDoc(title="B"), sub / "b.gbform")
    win.set_main_form()                            # B ist aktiv -> Startformular
    win.project_path = tmp_path / "app.gbproj"
    win.save_project()
    m = json.loads((tmp_path / "app.gbproj").read_text(encoding="utf-8"))["main"]
    assert m == "sub/b.gbform"                     # nicht auf a zurueckgefallen
    win.close()


def test_uppercase_gbproj_opens_as_project_not_as_form(tmp_path):
    # `gbform Projekt.GBPROJ` lud das Manifest als Formular (case-sensitiver
    # Suffix-Vergleich); ein Strg+S danach hat die Projektdatei ueberschrieben.
    from gamebasic.formdesigner import FormDoc, FormProject
    from gamebasic.formdesigner_qt import open_initial
    FormDoc(title="A").save(str(tmp_path / "a.gbform"))
    p = tmp_path / "Projekt.GBPROJ"
    FormProject(forms=["a.gbform"], main="a.gbform").save(str(p))
    _app()
    win = FormDesigner(tmp_path)
    assert open_initial(win, p) is True
    assert [f.doc.title for f in win.forms] == ["A"]      # als Projekt geladen
    assert all(f.path != p for f in win.forms)            # Manifest nie als Form
    win.close()


def test_open_initial_reports_a_missing_file(tmp_path):
    from gamebasic.formdesigner_qt import open_initial
    _app()
    win = FormDesigner(tmp_path)
    assert open_initial(win, tmp_path / "tippfehler.gbform") is False
    assert open_initial(win, tmp_path) is False           # Verzeichnis
    win.close()


# ------------------------------------------------------- F5-Run-Pfad (Review)
def test_run_form_uses_the_shared_gbrt_lookup(tmp_path):
    # Die lokale Kopie kannte den PyInstaller-Fall nicht -> F5 war in der
    # installierten IDE unmoeglich ("Runtime nicht gebaut").
    import gamebasic.formdesigner_qt as fq
    from gamebasic.editor_qt.gbrt_locate import find_gbrt
    assert fq._find_gbrt is find_gbrt


def test_run_form_reports_a_broken_program_instead_of_failing_silently(
        tmp_path, monkeypatch):
    # `Popen` lief ohne Ausgabe-Capture und ohne Exit-Code-Pruefung: jeder
    # Fehler im erzeugten Programm endete in einem stummen F5.
    import gamebasic.formdesigner_qt as fq
    if fq._find_gbrt(tmp_path) is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    _app()
    win = FormDesigner(tmp_path)
    b = win.canvas.doc.add("button", 10, 10)
    b.on_click = "kaputt"                              # Handler mit Syntaxfehler
    win.canvas.doc.code["kaputt"] = "PRINT ((("
    shown, started = {}, []
    monkeypatch.setattr(_mb(), "critical",
                        staticmethod(lambda p, t, msg, *a, **k: shown.update(t=t, msg=msg)))
    monkeypatch.setattr(FormDesigner, "_spawn",
                        lambda self, cmd, cwd: started.append(cmd))
    win.run_form()
    assert not started                                 # gar nicht erst gestartet
    assert "Fehler" in shown.get("msg", "") or "Zeile" in shown.get("msg", "")
    win.close()


def test_run_form_starts_a_valid_form_and_cleans_up_afterwards(tmp_path, monkeypatch):
    import gamebasic.formdesigner_qt as fq
    if fq._find_gbrt(tmp_path) is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    _app()
    win = FormDesigner(tmp_path)
    win.canvas.doc.add("button", 10, 10)

    class _FakeProc:
        def __init__(self): self.killed = False
        def poll(self): return None if not self.killed else 0
        def terminate(self): self.killed = True

    proc = _FakeProc()
    monkeypatch.setattr(FormDesigner, "_spawn", lambda self, cmd, cwd: proc)
    win.run_form()
    run_dir = win._run_dir
    assert run_dir is not None and (run_dir / "run.gb").exists()
    assert (run_dir / "form.gbform").exists()          # neben der .gb, fuer GUI_LOAD
    win.run_form()                                     # zweites F5
    assert proc.killed                                 # alter Prozess beendet
    assert not run_dir.exists()                        # alter Temp-Ordner weg
    leftover = win._run_dir
    win.close()
    assert not leftover.exists()                       # closeEvent raeumt auf


# --------------------------------------- Inspector / Bearbeiten-Ops (Review)
def test_renaming_a_handler_keeps_its_code_body(tmp_path):
    # `doc.code` ist nach Namen geschluesselt -- der Rumpf blieb unter dem
    # alten Schluessel liegen, das Panel zeigte leer, der Export nur ' TODO.
    _app()
    win = FormDesigner(tmp_path)
    b = win.canvas.doc.add("button", 10, 10)
    win.canvas.handler_requested.emit(b)                  # legt btn1Click an
    win.code_panel.editor.setPlainText('PRINT "wichtig"')
    win.canvas._select(b)
    win.inspector.on_click.setText("SpeichernClick")
    win.inspector._apply()
    assert win.canvas.doc.code.get("SpeichernClick") == 'PRINT "wichtig"'
    assert "btn1Click" not in win.canvas.doc.code
    assert 'PRINT "wichtig"' in win.canvas.doc.generate_gb_code()
    win.close()


def test_rename_does_not_steal_a_shared_handler(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    a = win.canvas.doc.add("button", 10, 10)
    b = win.canvas.doc.add("button", 10, 60)
    a.on_click = b.on_click = "gemeinsam"
    win.canvas.doc.code["gemeinsam"] = "PRINT 1"
    win.canvas._select(a)
    win.inspector.set_control(a)
    win.inspector.on_click.setText("nurA")
    win.inspector._apply()
    assert win.canvas.doc.code["gemeinsam"] == "PRINT 1"   # b braucht ihn noch
    assert "nurA" not in win.canvas.doc.code
    win.close()


def test_inspector_edits_group_placeholder_visible_and_sel(tmp_path):
    # Ohne `group` landen ALLE RadioButtons in derselben leeren Gruppe und
    # schliessen sich nicht gegenseitig aus -- der Designer war fuer Radios
    # unbrauchbar. `placeholder`/`visible`/`sel` fehlten ebenfalls.
    _app()
    win = FormDesigner(tmp_path)
    r = win.canvas.doc.add("radio", 10, 10)
    win.canvas._select(r)
    win.inspector.group.setText("schwierigkeit")
    win.inspector.visible.setChecked(False)
    win.inspector._apply()
    assert r.group == "schwierigkeit" and r.visible is False
    assert 'GUI_RADIO(frm, "schwierigkeit"' in win.canvas.doc.generate_gb_code()

    t = win.canvas.doc.add("textinput", 10, 60)
    win.canvas._select(t)
    win.inspector.placeholder.setText("dein Name")
    win.inspector._apply()
    assert t.placeholder == "dein Name"

    d = win.canvas.doc.add("dropdown", 10, 110)
    win.canvas._select(d)
    win.inspector.ssel.setValue(2)
    win.inspector._apply()
    assert d.sel == 2
    win.close()


def test_shortening_items_clamps_the_selection(tmp_path):
    # sel blieb out-of-range -> GUI_DROPDOWN_SET_SELECTED(dd1, 2) bei einem
    # einzigen Eintrag, das Programm fiel zur Laufzeit um.
    _app()
    win = FormDesigner(tmp_path)
    d = win.canvas.doc.add("dropdown", 10, 10)
    win.canvas._select(d)
    win.inspector.ssel.setValue(2)
    win.inspector._apply()
    win.inspector.items.setPlainText("nur eins")
    win.inspector._apply()
    assert d.sel <= len(d.items) - 1
    assert "SET_SELECTED" not in win.canvas.doc.generate_gb_code() or d.sel == 0
    win.close()


def test_window_inspector_keeps_max_above_min(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    wi = win.win_inspector
    wi.set_doc(win.canvas.doc)
    wi.minw.setValue(900); wi.maxw.setValue(200)
    wi._apply()
    assert win.canvas.doc.max_w >= win.canvas.doc.min_w
    assert win.canvas._clamp_fw(500) >= win.canvas.doc.min_w
    win.close()


def test_no_empty_undo_step_for_ineffective_operations(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    a = win.canvas.doc.add("button", 16, 16)
    b = win.canvas.doc.add("button", 16, 60)     # schon linksbuendig
    win.canvas._select_many([a, b])
    win.history.clear()
    win._align("left")
    assert len(win.history._undo) == 0
    win.canvas._select(b)                        # b ist bereits das vorderste
    win.raise_selected()
    assert len(win.history._undo) == 0
    win.close()


def test_duplicate_and_paste_handle_the_whole_selection(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    a = win.canvas.doc.add("button", 16, 16)
    b = win.canvas.doc.add("button", 16, 60)
    win.canvas._select_many([a, b])
    win.duplicate_selected()
    assert len(win.canvas.doc.controls) == 4     # beide dupliziert
    win.canvas._select_many([a, b])
    win.copy_selected()
    n = len(win.canvas.doc.controls)
    win.paste_clip()
    assert len(win.canvas.doc.controls) == n + 2
    win.close()


def test_code_panel_keeps_its_handler_across_undo(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    a = win.canvas.doc.add("button", 10, 10); a.on_click = "AClick"
    b = win.canvas.doc.add("button", 10, 60); b.on_click = "BClick"
    win.canvas.doc.code.update({"AClick": "", "BClick": ""})
    win.code_panel.set_doc(win.canvas.doc)
    win.code_panel.show_handler("BClick")
    pre = win.canvas.doc.to_dict()
    win.canvas.doc.title = "X"
    win.canvas.commit_history(pre)
    win.undo()
    assert win.code_panel.current == "BClick"    # nicht auf AClick gesprungen
    win.close()


def test_double_click_on_an_existing_handler_does_not_dirty(tmp_path):
    _app()
    win = FormDesigner(tmp_path)
    b = win.canvas.doc.add("button", 10, 10)
    win.canvas.handler_requested.emit(b)         # legt den Handler an
    win.active.dirty = False
    win.canvas.handler_requested.emit(b)         # zweites Mal: aendert nichts
    assert win.active.dirty is False
    win.close()


def test_code_panel_follows_a_delete_on_the_canvas(tmp_path):
    # Die Handler-Combo zeigte weiter den Handler des geloeschten Controls und
    # schrieb Edits in einen `code`-Eintrag, den es nicht mehr gibt.
    _app()
    win = FormDesigner(tmp_path)
    cv = win.canvas
    b = cv.doc.add("button", 16, 16)
    cv.handler_requested.emit(b)                       # legt btn1Click an
    assert win.code_panel.current == "btn1Click"
    cv._select(b)
    _press(cv, Qt.Key.Key_Delete)
    assert win.code_panel.combo.count() == 0
    assert win.code_panel.current is None
    assert "btn1Click" not in cv.doc.code              # Rumpf mit aufgeraeumt
    win.close()


# --- Inspector: Abschnitte -------------------------------------------------

def _sichtbare_abschnitte(ins):
    # `isHidden()` statt `isVisible()`: letzteres ist auch dann False, wenn
    # bloss das Fenster nicht angezeigt wurde.
    return [lbl.text() for lbl, _ in ins._sections if not lbl.isHidden()]


def test_inspector_zeigt_nur_abschnitte_mit_inhalt(tmp_path):
    """Ueberschriften blenden sich mit ihren Zeilen aus -- sonst staende bei
    einem Label ein leeres 'Werte' und 'Ereignisse' im Inspector."""
    from gamebasic.formdesigner import FormDoc
    _app()
    doc = FormDoc(title="T")

    schieber = doc.add("slider", 10, 10)
    ins = _Inspector()
    ins.set_control(schieber)
    assert "WERTE" in _sichtbare_abschnitte(ins), "Schieber hat Min/Max/Wert"
    assert "EREIGNISSE" in _sichtbare_abschnitte(ins), "Schieber hat on_change"

    beschriftung = doc.add("label", 10, 40)
    ins.set_control(beschriftung)
    offen = _sichtbare_abschnitte(ins)
    assert "WERTE" not in offen, "Label hat keine Wertebereiche"
    assert "EREIGNISSE" not in offen, "Label hat keine Ereignisse"
    # Was jedes Control hat, bleibt sichtbar.
    assert "ALLGEMEIN" in offen and "POSITION UND GROESSE" in offen


def test_inspector_ohne_auswahl_ist_leer(tmp_path):
    _app()
    ins = _Inspector()
    ins.set_control(None)
    assert _sichtbare_abschnitte(ins) == []


def test_inspector_abschnitte_decken_alle_zeilen_ab(tmp_path):
    """Jede Eigenschaftszeile muss unter einer Ueberschrift stehen -- eine
    Zeile ausserhalb waere fuer den Nutzer nicht zuzuordnen."""
    _app()
    ins = _Inspector()
    zugeordnet = {id(w) for _, widgets in ins._sections for w in widgets}
    fehlend = [lbl for lbl, w in ins._rows if id(w) not in zugeordnet]
    assert not fehlend, f"Zeilen ohne Abschnitt: {fehlend}"


# ---------------------------------------------------------------- Werkzeugleiste

def test_tool_icons_werden_gezeichnet():
    """Jede Symbol-Art liefert ein Bild -- ein leeres QIcon faellt in der
    Leiste nur als Luecke auf, nicht als Fehler."""
    _app()
    from gamebasic.formdesigner_qt import _tool_icon
    for kind in ("new", "open", "save", "undo", "redo", "run", "code"):
        ic = _tool_icon(kind)
        assert not ic.isNull(), kind
        assert not ic.pixmap(26, 26).toImage().allGray() or kind, kind
    _tool_icon("gibt-es-nicht")          # unbekannt -> leeres Bild, kein Absturz


def test_hauptleiste_hat_die_wichtigen_befehle(tmp_path):
    """Neu/Oeffnen/Speichern/Rueckgaengig/Wiederholen/Code/Ausfuehren lagen
    vorher nur im Menue -- besonders Ausfuehren war ohne F5 unauffindbar."""
    _app()
    win = FormDesigner(tmp_path)
    try:
        texte = [a.text() for a in win.main_bar.actions() if a.text()]
        for erwartet in ("Neues Formular", "Speichern", "Ausfuehren"):
            assert erwartet in texte, texte
        # Rueckgaengig/Wiederholen sind DIESELBEN QActions wie im Menue --
        # sonst wuerden sie nicht mit ausgrauen.
        assert win.act_undo in win.main_bar.actions()
        assert win.act_redo in win.main_bar.actions()
        run = [a for a in win.main_bar.actions() if a.text() == "Ausfuehren"][0]
        # Objektname => das gemeinsame Thema faerbt den Knopf gruen.
        assert win.main_bar.widgetForAction(run).objectName() == "RunButton"
    finally:
        win.close()


def test_anordnen_befehle_grauen_aus(tmp_path):
    """Ausrichten braucht 2 Controls, Verteilen 3. Ein grauer Knopf sagt das,
    bevor man klickt -- vorher kam die Absage erst in der Statusleiste."""
    _app()
    win = FormDesigner(tmp_path)
    try:
        doc = win.canvas.doc
        for _ in range(3):
            doc.add("button", 10, 10)

        def aktiv(anzahl):
            win.canvas.selection = list(doc.controls[:anzahl])
            win.canvas.selection_changed.emit(None)
            return sum(1 for a, _ in win._arrange_actions if a.isEnabled())

        gesamt = len(win._arrange_actions)
        verteilen = sum(1 for _, m in win._arrange_actions if m >= 3)
        assert aktiv(0) == 0
        assert aktiv(1) == 0
        assert aktiv(2) == gesamt - verteilen
        assert aktiv(3) == gesamt
    finally:
        win.close()


# ------------------------------------------------- Flaeche um das Formular
# Beide Fehler, die hier passiert sind, waren im CODE unauffaellig und nur im
# gerenderten Bild zu sehen: ein Raster, das im Thema verschwand, und ein
# Schatten ohne Spielraum nach unten. Darum wird hier gemessen statt gelesen.

def _canvas_bild(win, breite=640, hoehe=480):
    """Canvas ohne Event-Schleife in ein Bild rendern (`render`, nicht `grab`):
    kein `show()`, kein `processEvents()` -- im gemeinsamen pytest-Prozess ist
    ungebremstes Pumpen der bekannte Aufhaenger."""
    from PySide6.QtGui import QPixmap
    win.canvas.resize(breite, hoehe)
    pm = QPixmap(breite, hoehe)
    win.canvas.render(pm)
    return pm.toImage()


def _abstand(a, b):
    return abs(a.red() - b.red()) + abs(a.green() - b.green()) + abs(a.blue() - b.blue())


@pytest.mark.parametrize("thema", ["", "modern_light"])
def test_raster_hebt_sich_vom_formular_ab(tmp_path, thema):
    """Das Raster wurde einmal aus Fenster- und RAHMEN-farbe gemischt -- die
    liegen in beiden Themen dicht beieinander, das Raster verschwand komplett.
    Gemischt wird darum zur Schriftfarbe, die per Definition Kontrast hat.

    Gemessen wird im GERENDERTEN Bild, nicht an der Formel: haette der Test
    die Mischung selbst nachgerechnet, waere er gruen geblieben, egal was
    `_paint_grid` tatsaechlich zeichnet -- und genau dort sass der Fehler."""
    _app()
    from gamebasic.formdesigner_qt import PAD, TITLE_H, GRID, _col
    from gamebasic.formdesigner import theme_colors
    win = FormDesigner(tmp_path)
    try:
        win.canvas.doc.theme = thema
        win.canvas.zoom = 1.0
        win.canvas.snap_grid = True
        img = _canvas_bild(win)
        bg = _col(theme_colors(thema)["win_bg"])
        # Erster Rasterpunkt im Inhaltsbereich (siehe _paint_grid).
        punkt = img.pixelColor(PAD + GRID, PAD + TITLE_H + GRID)
        assert _abstand(bg, punkt) >= 55, (thema, bg.name(), punkt.name())
        # ... aber nicht so laut, dass es die Controls uebertoent.
        assert _abstand(bg, punkt) <= 260, (thema, bg.name(), punkt.name())
    finally:
        win.close()


def test_raster_verschwindet_bei_starker_verkleinerung(tmp_path):
    """Unter GRID_MIN_ZOOM liegen die Punkte so dicht, dass sie als Rauschen
    statt als Raster lesen."""
    _app()
    from gamebasic.formdesigner_qt import GRID_MIN_ZOOM
    win = FormDesigner(tmp_path)
    try:
        gezeichnet = []
        win.canvas._paint_grid = lambda *a, **k: gezeichnet.append(1)
        win.canvas.snap_grid = True
        win.canvas.zoom = GRID_MIN_ZOOM
        _canvas_bild(win)
        assert gezeichnet, "bei GRID_MIN_ZOOM soll das Raster noch kommen"
        gezeichnet.clear()
        win.canvas.zoom = GRID_MIN_ZOOM / 2
        _canvas_bild(win)
        assert not gezeichnet, "darunter soll es entfallen"
    finally:
        win.close()


def test_schatten_ist_neben_dem_formular_messbar(tmp_path):
    """Der Schatten war zuerst unsichtbar, weil die Arbeitsflaeche fast
    schwarz war -- ein schwarzer Schatten hatte dort keinen Spielraum mehr
    (gemessene 11 Stufen). Er muss sich deutlich vom Grund abheben."""
    _app()
    from gamebasic.formdesigner_qt import PAD
    win = FormDesigner(tmp_path)
    try:
        win.canvas.zoom = 1.0
        win.canvas.snap_grid = False
        d = win.canvas.doc
        img = _canvas_bild(win)
        y = PAD + d.h // 2                       # halbe Hoehe, rechts daneben
        direkt = img.pixelColor(PAD + d.w + 2, y)      # dicht am Rand: Schatten
        weit = img.pixelColor(PAD + d.w + 60, y)       # weit weg: reiner Grund
        # Der Grund ist bewusst angehoben, damit hier Platz nach unten bleibt.
        assert weit.red() > 30, f"Arbeitsflaeche zu dunkel fuer einen Schatten: {weit.name()}"
        assert _abstand(direkt, weit) >= 25, (direkt.name(), weit.name())
        assert direkt.red() < weit.red(), "Schatten muss DUNKLER sein als der Grund"
    finally:
        win.close()
