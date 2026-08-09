"""Tests fuer Sprite-Editor-Fixes aus der 2026-07-26-Review-Runde: Animation-
Vorschau-Fenster-Leck, Undo fuer Animations-Bereiche, Palette-Extraktion aus
dem Composite statt nur der aktiven Ebene, gedrosselte Slider-Commits
(Deckkraft + RGBA), und WA_DeleteOnClose auf allen restlichen Dialogen."""
import os
from pathlib import Path

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

pytest.importorskip("PySide6")

from PySide6.QtCore import Qt


@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


def _window(app):
    from drachenhauch.spriteeditor_qt import SpriteEditorWindow
    return SpriteEditorWindow(project_root=Path("."))


# --- AnimationPreview: Fenster-/Timer-Leck -------------------------------

def test_animation_preview_has_delete_on_close(app):
    from drachenhauch.spriteeditor_qt import AnimationPreview
    win = _window(app)
    dlg = AnimationPreview(win, win.doc.frames, win.doc.width, win.doc.height)
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
    dlg.close()


def test_animation_preview_close_stops_timers(app):
    from drachenhauch.spriteeditor_qt import AnimationPreview
    win = _window(app)
    dlg = AnimationPreview(win, win.doc.frames, win.doc.width, win.doc.height)
    assert dlg._timer.isActive()

    dlg.close()
    assert not dlg._timer.isActive()
    assert not dlg._raise_timer.isActive()


# --- AnimsPanel: Undo fuer Animations-Bereiche ---------------------------

def test_anims_panel_add_is_undoable(app, monkeypatch):
    from drachenhauch.spriteeditor_qt import AnimRangeDialog, Anim
    win = _window(app)
    win.doc.add_frame()
    monkeypatch.setattr(
        AnimRangeDialog, "edit",
        staticmethod(lambda parent, doc, anim: Anim("run", 0, 1, 8)))

    before_seq = win.doc.last_struct_undo_seq()
    win.anims_panel._add()
    assert [a.name for a in win.doc.anims] == ["run"]
    assert win.doc.last_struct_undo_seq() != before_seq

    assert win.doc.undo_struct()
    assert win.doc.anims == []


def test_anims_panel_delete_is_undoable(app):
    from drachenhauch.spriteeditor_qt import Anim
    win = _window(app)
    win.doc.add_frame()
    win.doc.anims = [Anim("run", 0, 1, 8)]
    win.anims_panel.refresh()
    win.anims_panel.list_widget.setCurrentRow(0)

    win.anims_panel._delete()
    assert win.doc.anims == []
    assert win.doc.undo_struct()
    assert [a.name for a in win.doc.anims] == ["run"]


def test_anims_panel_edit_is_undoable(app, monkeypatch):
    from drachenhauch.spriteeditor_qt import AnimRangeDialog, Anim
    win = _window(app)
    win.doc.add_frame()
    win.doc.anims = [Anim("run", 0, 1, 8)]
    win.anims_panel.refresh()
    win.anims_panel.list_widget.setCurrentRow(0)
    monkeypatch.setattr(
        AnimRangeDialog, "edit",
        staticmethod(lambda parent, doc, anim: Anim("walk", 0, 1, 12)))

    win.anims_panel._edit()
    assert [a.name for a in win.doc.anims] == ["walk"]
    assert win.doc.undo_struct()
    assert [a.name for a in win.doc.anims] == ["run"]


# --- Palette-Extraktion: composite() statt nur aktive Ebene ---------------

def test_palette_extract_uses_composite_not_just_active_layer(app):
    win = _window(app)
    f = win.doc.current
    f.add_layer()                                   # jetzt aktiv (Ebene 1, oben)
    f.pixels.putpixel((0, 0), (10, 20, 30, 255))     # nur auf der aktiven Ebene
    f.layers[0].pixels.putpixel((1, 1), (200, 100, 50, 255))  # untere Ebene

    win.action_palette_extract()
    assert (10, 20, 30, 255) in win.palette
    assert (200, 100, 50, 255) in win.palette


def test_palette_extract_ignores_hidden_active_layer(app):
    win = _window(app)
    f = win.doc.current
    f.add_layer()
    f.pixels.putpixel((0, 0), (10, 20, 30, 255))
    f.layers[f.active_layer].visible = False         # aktive Ebene ausgeblendet
    f.layers[0].pixels.putpixel((1, 1), (200, 100, 50, 255))

    win.action_palette_extract()
    assert (10, 20, 30, 255) not in win.palette
    assert (200, 100, 50, 255) in win.palette


# --- Opacity-Slider: gedrosselter Commit ----------------------------------

def test_opacity_drag_does_not_rebuild_panels_per_tick(app):
    win = _window(app)
    panel = win.layers_panel
    calls = []
    win.mark_dirty = lambda: calls.append(1)

    panel._on_opacity(50)
    panel._on_opacity(60)
    panel._on_opacity(70)
    assert calls == []                       # waehrend des "Drags" kein Rebuild
    assert win.doc.current.layers[0].opacity == 0.70

    panel._on_opacity_committed()
    assert calls == [1]                      # genau EIN Rebuild beim Commit


def test_opacity_commit_via_debounce_timer_covers_keyboard(app):
    """Fallback fuer Tastatur-Bedienung (kein sliderReleased)."""
    win = _window(app)
    panel = win.layers_panel
    calls = []
    win.mark_dirty = lambda: calls.append(1)

    panel._on_opacity(40)
    assert panel._opacity_commit_timer.isActive()
    panel._opacity_commit_timer.timeout.emit()
    assert calls == [1]


# --- RGBA-Farb-Slider: gedrosselter Commit + saubere Historie -------------

def test_color_slider_drag_does_not_pollute_history(app):
    win = _window(app)
    panel = win.color_panel
    win.color_history = []

    panel._on_slider("r", 10)
    panel._on_slider("r", 20)
    panel._on_slider("r", 30)
    assert win.color_history == []           # waehrend des "Drags" keine Historie
    assert win.fg[0] == 30                   # Live-Vorschau aktualisiert sich trotzdem

    panel._commit_slider_color()
    assert len(win.color_history) == 1
    assert win.color_history[0] == win.fg


def test_color_slider_commit_via_debounce_timer(app):
    win = _window(app)
    panel = win.color_panel
    win.color_history = []

    panel._on_slider("g", 99)
    assert panel._slider_commit_timer.isActive()
    panel._slider_commit_timer.timeout.emit()
    assert len(win.color_history) == 1


# --- WA_DeleteOnClose auf den restlichen Dialogen -------------------------

def test_resize_canvas_dialog_has_delete_on_close(app):
    from drachenhauch.spriteeditor_qt import ResizeCanvasDialog
    win = _window(app)
    dlg = ResizeCanvasDialog(win, win.doc.width, win.doc.height)
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_new_sprite_dialog_has_delete_on_close(app):
    from drachenhauch.spriteeditor_qt import NewSpriteDialog
    win = _window(app)
    dlg = NewSpriteDialog(win)
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_color_replace_dialog_has_delete_on_close(app):
    from drachenhauch.spriteeditor_qt import ColorReplaceDialog
    win = _window(app)
    dlg = ColorReplaceDialog(win)
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_sheet_import_dialog_has_delete_on_close(app):
    from drachenhauch.spriteeditor_qt import SheetImportDialog
    from PIL import Image
    win = _window(app)
    img = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    dlg = SheetImportDialog(win, img)
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_anim_range_dialog_has_delete_on_close(app):
    from drachenhauch.spriteeditor_qt import AnimRangeDialog, Anim
    win = _window(app)
    dlg = AnimRangeDialog(win, win.doc, Anim("a", 0, 0, 8))
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
