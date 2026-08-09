"""Tests fuer die Qt-Schicht des Tilemap-Editors (tilemapeditor_qt.py).

Bisher gab es hier keine Tests -- nur fuer das Qt-freie Datenmodell
(test_tilemapeditor.py). Diese Datei deckt die Review-Funde ab:
1. Layer-Add/Del/Move und Resize verwarfen bisher die GESAMTE Undo-Historie
   statt selbst undoable zu sein (dieselbe Bug-Klasse wie zuvor im
   Sprite-Editor: Crop/Rotate/Flatten waren dort nicht rueckgaengig machbar).
2. Neu/Oeffnen ersetzten das Dokument ohne auf ungespeicherte Aenderungen
   zu pruefen (doc.dirty wird von save_json/load_json bereits korrekt
   gepflegt, wurde aber nie abgefragt).
3. launch() verschluckte einen kaputten initial_file komplett stumm.
"""
import os
from pathlib import Path
from unittest.mock import patch

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

pytest.importorskip("PySide6")

from PySide6.QtWidgets import QApplication, QMessageBox

_ROOT = Path(__file__).resolve().parent.parent


@pytest.fixture(scope="module")
def app():
    return QApplication.instance() or QApplication([])


def _win(app, tmp_path=None):
    from drachenhauch.tilemapeditor_qt import TileMapEditor
    return TileMapEditor(project_root=tmp_path or _ROOT)


# --------------------------------------------------- Struktur-Undo

def test_add_layer_is_undoable(app):
    win = _win(app)
    n0 = len(win.doc.layers)
    win._add_layer()
    assert len(win.doc.layers) == n0 + 1
    win._undo()
    assert len(win.doc.layers) == n0
    win._redo()
    assert len(win.doc.layers) == n0 + 1


def test_add_layer_updates_dirty_title(app):
    """Review-Fund: doc.dirty wurde an vielen Stellen gesetzt, aber nie fuer
    einen Titel-Stern gelesen -- Struktur-Aenderungen liefen ueber
    _sync_layers() (jetzt der zentrale Hook fuer _update_title())."""
    win = _win(app)
    assert "*" not in win.windowTitle()
    win._add_layer()
    assert win.doc.dirty is True
    assert win.windowTitle().endswith("*")


def test_undo_after_structural_change_updates_title(app):
    win = _win(app)
    win._add_layer()
    # So tun, als waere gerade gespeichert worden (dirty=False + Titel neu).
    win.doc.dirty = False
    win._update_title()
    assert "*" not in win.windowTitle()
    win._undo()
    assert win.doc.dirty is True
    assert win.windowTitle().endswith("*")


def test_tile_paint_commit_updates_dirty_title(app):
    """Deckt die dirty-VOR-emit-Umsortierung in _Canvas._commit() ab: der
    Slot im Hauptfenster (_on_committed) ruft _update_title() waehrend des
    synchronen emit() auf -- ohne die Umsortierung waere doc.dirty an der
    Stelle noch False (Signal feuert vor der alten dirty=True-Zeile)."""
    win = _win(app)
    layer = win.doc.layers[0]
    assert "*" not in win.windowTitle()
    win.canvas._before = list(layer.tiles)
    layer.tiles = [1] * len(layer.tiles)   # tatsaechliche Aenderung
    win.canvas._commit(layer)
    assert win.doc.dirty is True
    assert win.windowTitle().endswith("*")


def test_save_clears_dirty_title(app, tmp_path, monkeypatch):
    from PySide6.QtWidgets import QFileDialog
    win = _win(app, tmp_path)
    win._add_layer()
    assert win.windowTitle().endswith("*")

    target = tmp_path / "map.json"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        lambda *a, **k: (str(target), ""))
    win._save()
    assert win.doc.path == str(target)
    assert not win.windowTitle().endswith("*")
    assert target.name in win.windowTitle()


def test_add_object_layer_is_undoable(app):
    win = _win(app)
    n0 = len(win.doc.layers)
    win._add_object_layer()
    assert len(win.doc.layers) == n0 + 1
    win._undo()
    assert len(win.doc.layers) == n0


def test_delete_layer_is_undoable(app):
    win = _win(app)
    win._add_layer()
    n0 = len(win.doc.layers)
    win.canvas.active_layer = 0
    win._del_layer()
    assert len(win.doc.layers) == n0 - 1
    win._undo()
    assert len(win.doc.layers) == n0


def test_move_layer_is_undoable(app):
    win = _win(app)
    win._add_layer()
    win.doc.layers[0].name = "A"
    win.doc.layers[1].name = "B"
    win.canvas.active_layer = 0
    win._move_layer(+1)
    assert [l.name for l in win.doc.layers] == ["B", "A"]
    win._undo()
    assert [l.name for l in win.doc.layers] == ["A", "B"]


def test_resize_map_is_undoable(app, monkeypatch):
    win = _win(app)
    w0, h0 = win.doc.width, win.doc.height
    import drachenhauch.tilemapeditor_qt as tm
    monkeypatch.setattr(tm, "_ask_map_params",
                        lambda *a, **k: (w0 + 3, h0 + 2, win.doc.tile_w, win.doc.tile_h))
    win._resize_map()
    assert (win.doc.width, win.doc.height) == (w0 + 3, h0 + 2)
    win._undo()
    assert (win.doc.width, win.doc.height) == (w0, h0)
    win._redo()
    assert (win.doc.width, win.doc.height) == (w0 + 3, h0 + 2)


def test_load_tileset_is_undoable(app, tmp_path):
    """Review-Fund: Tileset hinzufuegen/entfernen war -- wie zuvor Layer-
    Add/Del/Move/Resize -- nicht ueber die Undo-Historie abgesichert."""
    win = _win(app, tmp_path)
    a = tmp_path / "a.png"; a.write_bytes(b"\x89PNG\r\n")
    from PySide6.QtGui import QPixmap
    n0 = len(win.doc.tilesets)

    before = win._snapshot_doc()
    idx = win.doc.add_tileset(str(a), 32, 32)
    win.tileset_pixmaps.append(QPixmap(2, 2))
    win._push_doc_undo(before)
    assert len(win.doc.tilesets) == n0 + 1
    assert len(win.tileset_pixmaps) == n0 + 1

    win._undo()
    assert len(win.doc.tilesets) == n0
    assert len(win.tileset_pixmaps) == n0   # parallele Liste bleibt synchron

    win._redo()
    assert len(win.doc.tilesets) == n0 + 1
    assert len(win.tileset_pixmaps) == n0 + 1


def test_remove_tileset_is_undoable(app, tmp_path):
    win = _win(app, tmp_path)
    a = tmp_path / "a.png"; a.write_bytes(b"\x89PNG\r\n")
    b = tmp_path / "b.png"; b.write_bytes(b"\x89PNG\r\n")
    win.doc.add_tileset(str(a), 64, 32)
    win.doc.add_tileset(str(b), 32, 32)
    win.tileset_pixmaps = [None, None]
    win.doc.active_tileset = 1

    win._remove_tileset()
    assert len(win.doc.tilesets) == 1
    win._undo()
    assert len(win.doc.tilesets) == 2
    assert [t.firstgid for t in win.doc.tilesets] == [1, 9]
    win._redo()
    assert len(win.doc.tilesets) == 1


def test_older_tile_edit_survives_layer_structural_undo(app):
    """Ein aelterer Tiles-Undo-Eintrag (layer_idx-basiert) bleibt gueltig,
    solange Undo strikt in Stack-Reihenfolge laeuft -- das strukturelle
    "doc"-Undo direkt davor stellt die Layer-Liste in genau den Zustand
    zurueck, in dem der aeltere Eintrag urspruenglich erzeugt wurde."""
    from drachenhauch.tilemap.document import TileLayer
    win = _win(app)
    layer0 = win.doc.layers[0]
    assert isinstance(layer0, TileLayer)
    before_tiles = list(layer0.tiles)
    layer0.set(0, 0, 5)
    win._on_committed(0, before_tiles, list(layer0.tiles))

    win._add_layer()   # struktureller Undo-Eintrag oben auf dem Stack

    win._undo()   # macht add_layer rueckgaengig
    win._undo()   # macht die Tile-Aenderung rueckgaengig
    assert win.doc.layers[0].tiles[0] == 0
    win._redo()
    assert win.doc.layers[0].tiles[0] == 5


# --------------------------------------------------- Ungespeicherte Aenderungen

def test_confirm_discard_changes_clean_doc_no_prompt(app):
    win = _win(app)
    assert win.doc.dirty is False
    assert win._confirm_discard_changes() is True


def test_confirm_discard_changes_dirty_respects_answer(app):
    win = _win(app)
    win.doc.dirty = True
    with patch.object(QMessageBox, "question",
                      return_value=QMessageBox.StandardButton.No):
        assert win._confirm_discard_changes() is False
    with patch.object(QMessageBox, "question",
                      return_value=QMessageBox.StandardButton.Yes):
        assert win._confirm_discard_changes() is True


def test_new_map_prompts_when_dirty(app, monkeypatch):
    import drachenhauch.tilemapeditor_qt as tm
    win = _win(app)
    win.doc.dirty = True
    monkeypatch.setattr(tm, "_ask_map_params",
                        lambda *a, **k: pytest.fail("_ask_map_params sollte nicht laufen"))
    with patch.object(QMessageBox, "question",
                      return_value=QMessageBox.StandardButton.No):
        win._new_map()   # bricht vor _ask_map_params ab


def test_close_event_ignored_when_dirty_and_declined(app):
    win = _win(app)
    win.doc.dirty = True

    class _FakeEvent:
        def __init__(self):
            self.accepted = None
        def accept(self): self.accepted = True
        def ignore(self): self.accepted = False

    ev = _FakeEvent()
    with patch.object(QMessageBox, "question",
                      return_value=QMessageBox.StandardButton.No):
        win.closeEvent(ev)
    assert ev.accepted is False


# --------------------------------------------------- launch() Fehleranzeige

def test_launch_reports_broken_initial_file(app, tmp_path, monkeypatch):
    import drachenhauch.tilemapeditor_qt as tm
    bad = tmp_path / "broken.json"
    bad.write_text("{not valid json", encoding="utf-8")

    warnings = []
    monkeypatch.setattr(QMessageBox, "warning",
                        lambda *a, **k: warnings.append(a))
    # exec() wuerde blockieren -- Fenster fuer den Test nicht anzeigen/laufen lassen.
    monkeypatch.setattr(tm.QApplication, "exec", lambda self: 0)
    monkeypatch.setattr(tm.TileMapEditor, "show", lambda self: None)

    tm.launch(tmp_path, initial_file=bad)
    assert warnings, "keine Warnung bei kaputtem initial_file gezeigt"


# --------------------------------------------------- Tileset entfernen

def test_remove_tileset_ui_refuses_when_in_use(app, tmp_path):
    """Review-Fund: _remove_tileset() duplizierte frueher die
    "in Benutzung"-Pruefung inline statt das Modell zu fragen -- jetzt
    nutzt sie doc.tileset_in_use()/remove_tileset()'s eigene Ablehnung."""
    win = _win(app, tmp_path)
    a = tmp_path / "a.png"; a.write_bytes(b"\x89PNG\r\n")
    b = tmp_path / "b.png"; b.write_bytes(b"\x89PNG\r\n")
    win.doc.add_tileset(str(a), 64, 32)
    win.doc.add_tileset(str(b), 32, 32)
    win.tileset_pixmaps = [None, None]
    win.doc.active_tileset = 1
    win.doc.layers[0].set(0, 0, win.doc.tilesets[1].firstgid)

    with patch.object(QMessageBox, "information") as info:
        win._remove_tileset()
        assert info.called
    assert len(win.doc.tilesets) == 2   # nicht entfernt

    win.doc.layers[0].set(0, 0, 0)   # nicht mehr in Benutzung
    win._remove_tileset()
    assert len(win.doc.tilesets) == 1


# --------------------------------------------------- Tile-Eigenschaften

def test_edit_props_is_undoable(app, tmp_path):
    """Review-Fund: _edit_props() war der EINZIGE Struktur-Mutations-
    Aufrufer in dieser Datei, der keinen Undo-Snapshot anlegte -- Ctrl+Z
    konnte eine Tile-Eigenschaften-Aenderung ueberhaupt nicht rueckgaengig
    machen."""
    from drachenhauch.tilemapeditor_qt import _PropDialog
    win = _win(app, tmp_path)
    a = tmp_path / "a.png"; a.write_bytes(b"\x89PNG\r\n")
    win.doc.add_tileset(str(a), 64, 32)
    win.sel_local = 0
    assert win.doc.properties_of(0) == {}

    from PySide6.QtWidgets import QDialog

    def _fake_exec(self):
        self._add_row("solid", "bool", "true")
        return QDialog.DialogCode.Accepted

    with patch.object(_PropDialog, "exec", _fake_exec):
        win._edit_props()
    assert win.doc.properties_of(0) == {"solid": True}
    assert win.doc.dirty is True

    win._undo()
    assert win.doc.properties_of(0) == {}


def test_edit_props_deleting_all_rows_marks_dirty(app, tmp_path):
    """Review-Fund: apply_to_doc() leerte tile_properties/-types direkt
    (bypass von set_property()), OHNE dirty zu setzen, wenn danach KEINE
    Zeile mehr in der Tabelle stand -- die Apply-Schleife ruft set_property()
    dann kein einziges Mal auf. `_edit_props()` markiert jetzt explizit
    selbst dirty, unabhaengig vom Tabelleninhalt."""
    from drachenhauch.tilemapeditor_qt import _PropDialog
    win = _win(app, tmp_path)
    a = tmp_path / "a.png"; a.write_bytes(b"\x89PNG\r\n")
    win.doc.add_tileset(str(a), 64, 32)
    win.sel_local = 0
    win.doc.set_property(0, "solid", "true", "bool")
    win.doc.dirty = False   # so tun, als waere gerade gespeichert worden

    from PySide6.QtWidgets import QDialog

    def _fake_exec(self):
        while self.table.rowCount():
            self.table.removeRow(0)   # alle Zeilen loeschen
        return QDialog.DialogCode.Accepted

    with patch.object(_PropDialog, "exec", _fake_exec):
        win._edit_props()
    assert win.doc.properties_of(0) == {}
    assert win.doc.dirty is True
    assert win.windowTitle().endswith("*")


# --------------------------------------------------- Auswahl/Clipboard

def test_set_doc_resets_selection_and_clipboard(app, tmp_path):
    """Review-Fund: eine kopierte Tile-Auswahl (Clipboard) und eine laufende
    Selektion ueberlebten frueher einen Dokument-Wechsel (Neu/Oeffnen) --
    ein Paste danach haette GIDs aus dem ALTEN Dokument in das neue
    gestempelt, die dort auf ein evtl. gar nicht existierendes Tileset
    zeigen."""
    from drachenhauch.tilemap.document import TileMapDoc
    win = _win(app, tmp_path)
    win.canvas.tile_clipboard = [[5, 6], [7, 8]]
    win.canvas._sel_rect = (0, 0, 1, 1)
    win.canvas.selected_obj = object()

    win.canvas.set_doc(TileMapDoc(4, 4, 16, 16), None)
    assert win.canvas.tile_clipboard is None
    assert win.canvas._sel_rect is None
    assert win.canvas.selected_obj is None


def test_doc_undo_restore_resets_selection_and_clipboard(app):
    """Gleicher Schutz beim Wiederherstellen eines "doc"-Struktur-Snapshots
    (z.B. Resize rueckgaengig) -- Auswahl/Clipboard koennten sonst auf
    Koordinaten/Tilesets zeigen, die im wiederhergestellten Zustand nicht
    mehr gueltig sind."""
    import drachenhauch.tilemapeditor_qt as tm
    win = _win(app)
    w0, h0 = win.doc.width, win.doc.height

    before = win._snapshot_doc()
    win.doc.resize(w0 + 2, h0 + 2)
    win._push_doc_undo(before)

    win.canvas.tile_clipboard = [[1]]
    win.canvas._sel_rect = (0, 0, 0, 0)

    win._undo()
    assert win.canvas.tile_clipboard is None
    assert win.canvas._sel_rect is None


# --------------------------------------------------- WA_DeleteOnClose

def test_prop_dialog_has_delete_on_close(app, tmp_path):
    from drachenhauch.tilemapeditor_qt import _PropDialog
    from PySide6.QtCore import Qt
    win = _win(app, tmp_path)
    dlg = _PropDialog(win.doc, 0, win)
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_object_dialog_has_delete_on_close(app):
    from drachenhauch.tilemapeditor_qt import _ObjectDialog
    from drachenhauch.tilemap.document import MapObject
    from PySide6.QtCore import Qt
    win = _win(app)
    dlg = _ObjectDialog(MapObject(0, 0, name="spawn"), win)
    assert dlg.testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_map_params_dialog_has_delete_on_close(app, monkeypatch):
    from drachenhauch.tilemapeditor_qt import _ask_map_params
    from PySide6.QtWidgets import QDialog
    from PySide6.QtCore import Qt
    win = _win(app)
    captured = {}
    def _capture_exec(self):
        captured["dlg"] = self
        return QDialog.DialogCode.Rejected
    monkeypatch.setattr(QDialog, "exec", _capture_exec)
    _ask_map_params(win)
    assert captured["dlg"].testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)


def test_export_code_dialog_has_delete_on_close(app, monkeypatch, tmp_path):
    from PySide6.QtWidgets import QDialog
    from PySide6.QtCore import Qt
    win = _win(app, tmp_path)
    captured = {}
    def _capture_exec(self):
        captured["dlg"] = self
    monkeypatch.setattr(QDialog, "exec", _capture_exec)
    win._export_code()
    assert captured["dlg"].testAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
