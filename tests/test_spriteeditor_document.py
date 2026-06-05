"""Tests fuer das extrahierte SpriteDoc-Modell.

Frame und SpriteDoc sind nach dem Refactor in `spriteeditor.document`
herausgezogen worden. Diese Tests sichern die Datenmodell-Logik
(Frame-Operationen, Undo/Redo, Resize) ohne Qt-Window aufzubauen.
"""
import os
import io
from pathlib import Path

import pytest

# Headless: PySide6 importiert sich offline. Auch das document-Modul
# braucht Qt-Imports (QImage/QPixmap), aber kein Display.
os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

from gamebasic.spriteeditor.document import (
    DEFAULT_FRAME_DURATION_MS,
    Frame,
    SpriteDoc,
    pil_to_qpixmap,
)
from PIL import Image


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    """QPixmap braucht eine QApplication-Instanz, sonst haengt
    `QPixmap.fromImage`. Ohne Display ist offscreen-Plattform OK."""
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


@pytest.fixture
def doc():
    return SpriteDoc(16, 16)


# --- Frame-Lifecycle ----------------------------------------------

def test_new_doc_has_one_frame(doc):
    assert len(doc.frames) == 1
    assert doc.current_index == 0
    assert doc.width == 16 and doc.height == 16


def test_add_frame_inserts_after_current(doc):
    new_idx = doc.add_frame()
    assert new_idx == 1
    assert len(doc.frames) == 2
    assert doc.current_index == 1


def test_add_frame_copy_current(doc):
    # Pixel auf erstem Frame setzen, neuen Frame mit Kopie machen,
    # dann sollte der neue Frame denselben Pixel haben.
    doc.current.pixels.putpixel((5, 5), (255, 0, 0, 255))
    doc.add_frame(copy_current=True)
    assert doc.current.pixels.getpixel((5, 5)) == (255, 0, 0, 255)


def test_delete_frame_keeps_at_least_one(doc):
    # Mit nur einem Frame: delete liefert False
    assert doc.delete_frame() is False
    doc.add_frame()
    assert doc.delete_frame() is True
    assert len(doc.frames) == 1


def test_move_frame_reorders(doc):
    doc.add_frame()
    doc.add_frame()  # 3 frames, current_index = 2
    doc.frames[0].pixels.putpixel((0, 0), (255, 0, 0, 255))
    doc.frames[1].pixels.putpixel((0, 0), (0, 255, 0, 255))
    doc.frames[2].pixels.putpixel((0, 0), (0, 0, 255, 255))
    doc.move_frame(-1)  # current rutscht von Index 2 zu 1
    assert doc.current_index == 1
    assert doc.frames[1].pixels.getpixel((0, 0)) == (0, 0, 255, 255)


def test_move_frame_clamps_at_boundary(doc):
    doc.add_frame()  # 2 frames, current = 1
    assert doc.move_frame(5) is False  # ueber das Ende -> False
    assert doc.current_index == 1


# --- Undo/Redo ------------------------------------------------------

def test_undo_returns_false_when_history_empty(doc):
    assert doc.current.undo() is False


def test_snapshot_then_undo_restores(doc):
    doc.current.pixels.putpixel((0, 0), (255, 0, 0, 255))
    doc.current.snapshot()
    doc.current.pixels.putpixel((0, 0), (0, 255, 0, 255))
    assert doc.current.undo() is True
    assert doc.current.pixels.getpixel((0, 0)) == (255, 0, 0, 255)


def test_undo_then_redo_reverses(doc):
    doc.current.snapshot()
    doc.current.pixels.putpixel((0, 0), (255, 0, 0, 255))
    doc.current.undo()
    assert doc.current.redo() is True
    assert doc.current.pixels.getpixel((0, 0)) == (255, 0, 0, 255)


def test_snapshot_clears_redo(doc):
    doc.current.snapshot()
    doc.current.pixels.putpixel((0, 0), (255, 0, 0, 255))
    doc.current.undo()
    assert doc.current.redo_stack  # redo gibt was her
    doc.current.snapshot()  # neue Snapshot loescht den redo-Stack
    assert not doc.current.redo_stack


def test_history_max_length_is_80(doc):
    # 100 Snapshots -> nur 80 werden behalten
    for _ in range(100):
        doc.current.snapshot()
    assert len(doc.current.history) == 80


# --- Resize ---------------------------------------------------------

def test_resize_increases_canvas(doc):
    doc.current.pixels.putpixel((0, 0), (255, 0, 0, 255))
    doc.resize(32, 32)
    assert doc.width == 32 and doc.height == 32
    # Original-Pixel bleibt am gleichen Spot
    assert doc.current.pixels.getpixel((0, 0)) == (255, 0, 0, 255)
    # Neue Pixel sind transparent
    assert doc.current.pixels.getpixel((20, 20)) == (0, 0, 0, 0)


def test_resize_clears_history():
    d = SpriteDoc(16, 16)
    d.current.snapshot()
    assert d.current.history
    d.resize(32, 32)
    assert not d.current.history


# --- Persistenz -----------------------------------------------------

def test_native_roundtrip(tmp_path):
    d = SpriteDoc(8, 8)
    d.current.pixels.putpixel((3, 3), (255, 0, 0, 255))
    d.add_frame()
    d.frames[1].duration_ms = 250
    d.frames[1].pixels.putpixel((4, 4), (0, 255, 0, 255))
    target = tmp_path / "sprite.gbsprite"
    d.save_native(target)
    assert target.exists()
    loaded = SpriteDoc.load_native(target)
    assert loaded.width == 8 and loaded.height == 8
    assert len(loaded.frames) == 2
    assert loaded.frames[1].duration_ms == 250
    assert loaded.frames[0].pixels.getpixel((3, 3)) == (255, 0, 0, 255)
    assert loaded.frames[1].pixels.getpixel((4, 4)) == (0, 255, 0, 255)


def test_native_load_v1_backward_compat(tmp_path):
    """Aeltere .gbsprite-Dateien ohne duration_ms-Feld muessen weiter laden."""
    import base64, json
    img = Image.new("RGBA", (4, 4), (1, 2, 3, 255))
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    v1_data = {
        "version": 1,
        "width": 4,
        "height": 4,
        "frames": [{"data": base64.b64encode(buf.getvalue()).decode("ascii")}],
    }
    target = tmp_path / "old.gbsprite"
    target.write_text(json.dumps(v1_data), encoding="utf-8")
    loaded = SpriteDoc.load_native(target)
    assert loaded.frames[0].duration_ms == DEFAULT_FRAME_DURATION_MS


def test_save_png_single(tmp_path):
    d = SpriteDoc(4, 4)
    d.current.pixels.putpixel((1, 1), (200, 100, 50, 255))
    target = tmp_path / "test.png"
    d.save_png_single(target)
    assert target.exists()
    loaded = Image.open(target)
    assert loaded.mode == "RGBA"
    assert loaded.size == (4, 4)


def test_save_sheet_horizontal(tmp_path):
    d = SpriteDoc(4, 4)
    d.add_frame()
    d.add_frame()  # 3 frames
    target = tmp_path / "sheet.png"
    d.save_sheet_png(target, layout="horizontal")
    sheet = Image.open(target)
    assert sheet.size == (12, 4)  # 3 frames * 4px wide


def test_save_sheet_vertical(tmp_path):
    d = SpriteDoc(4, 4)
    d.add_frame()
    target = tmp_path / "sheet_v.png"
    d.save_sheet_png(target, layout="vertical")
    sheet = Image.open(target)
    assert sheet.size == (4, 8)  # 2 frames * 4px tall


def test_save_sheet_atlas_writes_png_and_json(tmp_path):
    """Atlas-Export: PNG + JSON-Manifest gemeinsam."""
    import json as _json
    d = SpriteDoc(16, 16)
    d.add_frame()
    d.add_frame()        # 3 Frames
    png = tmp_path / "tiles.png"
    j   = tmp_path / "tiles.json"
    manifest = d.save_sheet_atlas(png, j)
    # PNG existiert und hat die richtige Groesse
    assert png.exists()
    sheet = Image.open(png)
    assert sheet.size == (48, 16)  # 3 Frames * 16px wide
    # JSON existiert und ist lesbar
    assert j.exists()
    data = _json.loads(j.read_text(encoding="utf-8"))
    assert data == manifest  # zurueckgegebenes dict == geschriebenes
    # Schema-Check
    assert data["image"] == "tiles.png"
    assert set(data["sprites"].keys()) == {"tiles_0", "tiles_1", "tiles_2"}
    assert data["sprites"]["tiles_0"] == [0, 0, 16, 16]
    assert data["sprites"]["tiles_1"] == [16, 0, 16, 16]
    assert data["sprites"]["tiles_2"] == [32, 0, 16, 16]


def test_save_sheet_atlas_custom_prefix(tmp_path):
    d = SpriteDoc(8, 8)
    d.add_frame()
    png = tmp_path / "anim.png"
    j   = tmp_path / "anim.json"
    d.save_sheet_atlas(png, j, name_prefix="player_walk")
    import json as _json
    data = _json.loads(j.read_text(encoding="utf-8"))
    assert set(data["sprites"].keys()) == {"player_walk_0", "player_walk_1"}


def test_save_sheet_atlas_vertical_layout(tmp_path):
    d = SpriteDoc(8, 8)
    d.add_frame()         # 2 Frames
    png = tmp_path / "v.png"
    j   = tmp_path / "v.json"
    d.save_sheet_atlas(png, j, layout="vertical")
    sheet = Image.open(png)
    assert sheet.size == (8, 16)
    import json as _json
    data = _json.loads(j.read_text(encoding="utf-8"))
    assert data["sprites"]["v_0"] == [0, 0, 8, 8]
    assert data["sprites"]["v_1"] == [0, 8, 8, 8]


def test_save_sheet_atlas_loads_via_atlas_load(tmp_path):
    """Round-trip: Editor schreibt Atlas, GameBasic-Engine laedt ihn.
    Schliesst den Workflow-Loop."""
    pytest.importorskip("pygame")
    d = SpriteDoc(16, 16)
    d.add_frame()  # 2 Frames
    png = tmp_path / "atlas.png"
    j   = tmp_path / "atlas.json"
    d.save_sheet_atlas(png, j, name_prefix="tile")

    from gamebasic.graphics import Graphics
    g = Graphics()
    try:
        atlas = g.load_sprite_atlas(str(j))
        assert sorted(atlas.frames.keys()) == ["tile_0", "tile_1"]
        assert atlas.frames["tile_0"] == (0, 0, 16, 16)
        assert atlas.frames["tile_1"] == (16, 0, 16, 16)
    finally:
        g.shutdown()


def test_save_sheet_atlas_relative_image_path(tmp_path):
    """PNG und JSON im gleichen Verzeichnis -> image-Feld nur der Filename."""
    d = SpriteDoc(4, 4)
    png = tmp_path / "x.png"
    j   = tmp_path / "x.json"
    d.save_sheet_atlas(png, j)
    import json as _json
    data = _json.loads(j.read_text(encoding="utf-8"))
    assert data["image"] == "x.png"
    # Keine Backslashes auch auf Windows
    assert "\\" not in data["image"]


def test_save_sheet_atlas_uses_frame_names(tmp_path):
    """Frames mit eigenem Namen liefern diesen als Sprite-ID, unbenannte
    fallen auf <prefix>_<idx> zurueck."""
    import json as _json
    d = SpriteDoc(16, 16)
    d.add_frame()
    d.add_frame()        # 3 Frames
    d.frames[0].name = "idle"
    d.frames[2].name = "jump"
    png = tmp_path / "hero.png"
    j   = tmp_path / "hero.json"
    d.save_sheet_atlas(png, j)
    data = _json.loads(j.read_text(encoding="utf-8"))
    assert set(data["sprites"].keys()) == {"idle", "hero_1", "jump"}
    assert data["sprites"]["idle"] == [0, 0, 16, 16]
    assert data["sprites"]["jump"] == [32, 0, 16, 16]


def test_save_sheet_atlas_duplicate_names_disambiguated(tmp_path):
    """Doppelte Frame-Namen werden nicht ueberschrieben, sondern eindeutig
    gemacht (sonst gingen Frames im Manifest verloren)."""
    import json as _json
    d = SpriteDoc(8, 8)
    d.add_frame()
    d.frames[0].name = "dup"
    d.frames[1].name = "dup"
    png = tmp_path / "d.png"
    j   = tmp_path / "d.json"
    d.save_sheet_atlas(png, j)
    data = _json.loads(j.read_text(encoding="utf-8"))
    assert len(data["sprites"]) == 2     # beide Frames vertreten
    assert "dup" in data["sprites"]
    assert "dup_1" in data["sprites"]


def test_native_roundtrip_preserves_frame_name(tmp_path):
    """name-Feld ueberlebt save_native -> load_native (Version 3)."""
    d = SpriteDoc(8, 8)
    d.add_frame()
    d.frames[0].name = "walk"
    target = tmp_path / "named.gbsprite"
    d.save_native(target)
    loaded = SpriteDoc.load_native(target)
    assert loaded.frames[0].name == "walk"
    assert loaded.frames[1].name == ""


def test_load_native_version2_defaults_empty_name(tmp_path):
    """Aeltere Version-2-Dateien ohne name-Feld laden mit name=''."""
    import json as _json
    import base64 as _b64
    buf = io.BytesIO()
    Image.new("RGBA", (4, 4), (0, 0, 0, 0)).save(buf, format="PNG")
    legacy = {
        "version": 2,
        "width": 4,
        "height": 4,
        "frames": [{
            "data": _b64.b64encode(buf.getvalue()).decode("ascii"),
            "duration_ms": 100,
        }],
    }
    target = tmp_path / "legacy.gbsprite"
    target.write_text(_json.dumps(legacy), encoding="utf-8")
    loaded = SpriteDoc.load_native(target)
    assert loaded.frames[0].name == ""
    assert loaded.frames[0].duration_ms == 100


def test_load_image_single_frame(tmp_path):
    img = Image.new("RGBA", (8, 8), (0, 0, 0, 255))
    img.putpixel((3, 3), (255, 0, 0, 255))
    src = tmp_path / "in.png"
    img.save(src)
    d = SpriteDoc.load_image(src)
    assert d.width == 8 and d.height == 8
    assert len(d.frames) == 1


def test_load_image_with_frame_size_slices_sheet(tmp_path):
    """8x16 Sheet mit frame_w=8, frame_h=8 -> 2 Frames."""
    img = Image.new("RGBA", (8, 16), (0, 0, 0, 255))
    src = tmp_path / "sheet.png"
    img.save(src)
    d = SpriteDoc.load_image(src, frame_w=8, frame_h=8)
    assert d.width == 8 and d.height == 8
    assert len(d.frames) == 2


# --- Konversion -----------------------------------------------------

def test_pil_to_qpixmap_preserves_size():
    img = Image.new("RGBA", (12, 8), (255, 0, 0, 128))
    pix = pil_to_qpixmap(img)
    assert pix.width() == 12
    assert pix.height() == 8
    assert not pix.isNull()
