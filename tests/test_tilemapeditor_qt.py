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
    from gamebasic.tilemapeditor_qt import TileMapEditor
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
    import gamebasic.tilemapeditor_qt as tm
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
    from gamebasic.tilemap.document import TileLayer
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
    import gamebasic.tilemapeditor_qt as tm
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
    import gamebasic.tilemapeditor_qt as tm
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
