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


def test_paste_as_frame_inserts_after_current():
    d = SpriteDoc(16, 16)
    d.add_frame()                     # 2 Frames, current = 1
    img = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    img.putpixel((3, 4), (10, 20, 30, 255))
    idx = d.paste_as_frame(img)
    assert idx == 2                   # nach dem aktuellen (1) eingefuegt
    assert len(d.frames) == 3
    assert d.current_index == 2
    assert d.current.pixels.getpixel((3, 4)) == (10, 20, 30, 255)
    assert d.dirty is True


def test_paste_as_frame_crops_oversized():
    d = SpriteDoc(8, 8)
    big = Image.new("RGBA", (20, 20), (5, 5, 5, 255))
    d.paste_as_frame(big)
    # Auf Dokumentgroesse beschnitten
    assert d.current.pixels.size == (8, 8)


def test_paste_as_frame_pads_undersized():
    d = SpriteDoc(16, 16)
    small = Image.new("RGBA", (4, 4), (9, 9, 9, 255))
    d.paste_as_frame(small)
    assert d.current.pixels.size == (16, 16)
    assert d.current.pixels.getpixel((0, 0)) == (9, 9, 9, 255)
    assert d.current.pixels.getpixel((10, 10)) == (0, 0, 0, 0)   # transparent


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


# --- Benannte Animations-Bereiche (V4) --------------------------------------

def _doc_with_anims():
    from gamebasic.spriteeditor.document import Anim
    doc = SpriteDoc(8, 8)
    for _ in range(5):
        doc.add_frame()
    doc.anims = [Anim("walk", 0, 3, 10), Anim("jump", 4, 5, 6)]
    return doc


def test_anims_roundtrip_native(tmp_path):
    doc = _doc_with_anims()
    p = tmp_path / "a.gbsprite"
    doc.save_native(p)
    loaded = SpriteDoc.load_native(p)
    assert [(a.name, a.first, a.last, a.fps) for a in loaded.anims] == \
        [("walk", 0, 3, 10), ("jump", 4, 5, 6)]


def test_anims_backward_compat_v3(tmp_path):
    # Datei ohne anims-Feld (V3) laedt mit leerer Bereichs-Liste.
    doc = SpriteDoc(8, 8)
    p = tmp_path / "old.gbsprite"
    doc.save_native(p)
    import json as _json
    data = _json.loads(p.read_text(encoding="utf-8"))
    data["version"] = 3
    data.pop("anims", None)
    p.write_text(_json.dumps(data), encoding="utf-8")
    assert SpriteDoc.load_native(p).anims == []


def test_anims_shift_on_frame_delete():
    doc = _doc_with_anims()
    doc.select(1)
    doc.delete_frame()          # Frame 1 weg -> walk schrumpft, jump rueckt auf
    assert [(a.name, a.first, a.last) for a in doc.anims] == \
        [("walk", 0, 2), ("jump", 3, 4)]


def test_anims_shift_on_frame_insert():
    doc = _doc_with_anims()
    doc.select(0)
    doc.add_frame()             # Insert bei Index 1 -> alles ab 1 verschiebt
    assert [(a.name, a.first, a.last) for a in doc.anims] == \
        [("walk", 0, 4), ("jump", 5, 6)]


def test_anim_fps_suggestion_from_durations():
    doc = SpriteDoc(8, 8)
    doc.add_frame()
    for f in doc.frames:
        f.duration_ms = 100     # 10 fps
    assert doc.anim_fps_suggestion(0, 1) == 10


def test_generate_gb_snippet_uses_anims_and_compiles(run_gb):
    doc = _doc_with_anims()
    snippet = doc.generate_gb_snippet("hero.png")
    assert 'SPRITE_ADD_ANIM(sp, "walk", 0, 3, 10)' in snippet
    assert 'SPRITE_ADD_ANIM(sp, "jump", 4, 5, 6)' in snippet
    assert 'SPRITE_PLAY(sp, "walk")' in snippet


def test_generate_gb_snippet_fallback_idle_fps_from_durations():
    doc = SpriteDoc(8, 8)
    doc.add_frame()
    for f in doc.frames:
        f.duration_ms = 50      # 20 fps statt hardcoded 8
    snippet = doc.generate_gb_snippet("x.png")
    assert 'SPRITE_ADD_ANIM(sp, "idle", 0, 1, 20)' in snippet


def test_generate_gbanim_loads_in_runtime(run_gb, tmp_path):
    # Integration: die exportierte .gbanim ist direkt ANIM_FSM_LOAD-ladbar.
    import json as _json
    doc = _doc_with_anims()
    (tmp_path / "hero.gbanim").write_text(
        _json.dumps(doc.generate_gbanim(), indent=2), encoding="utf-8")
    src = "\n".join([
        'IMPORT "animfsm"',
        'DIM fsm AS ANIM_FSM',
        'fsm = ANIM_FSM_LOAD("hero.gbanim")',
        'PRINT ANIM_FSM_STATE(fsm)',
    ])
    out = run_gb(src, base=tmp_path)
    assert "walk" in out


# --- Struktur-Undo (Frame-Ops, Resize) ---------------------------------------

def test_struct_undo_restores_deleted_frame():
    from gamebasic.spriteeditor.document import Anim
    doc = SpriteDoc(8, 8)
    doc.add_frame()
    doc.frames[1].pixels.putpixel((2, 2), (255, 0, 0, 255))
    doc.frames[1].name = "kopf"
    doc.anims = [Anim("a", 0, 1, 8)]
    doc.select(1)
    assert doc.delete_frame()
    # Bereich schrumpft aufs verbleibende Frame (statt zu verschwinden)
    assert len(doc.frames) == 1
    assert [(a.name, a.first, a.last) for a in doc.anims] == [("a", 0, 0)]
    assert doc.undo_struct()
    assert len(doc.frames) == 2
    assert doc.frames[1].pixels.getpixel((2, 2)) == (255, 0, 0, 255)
    assert doc.frames[1].name == "kopf"
    assert [(a.name, a.first, a.last) for a in doc.anims] == [("a", 0, 1)]
    # Redo loescht wieder
    assert doc.redo_struct()
    assert len(doc.frames) == 1


def test_struct_undo_restores_resize():
    doc = SpriteDoc(8, 8)
    doc.current.pixels.putpixel((7, 7), (0, 255, 0, 255))
    doc.resize(16, 16)
    assert (doc.width, doc.height) == (16, 16)
    assert doc.undo_struct()
    assert (doc.width, doc.height) == (8, 8)
    assert doc.current.pixels.getpixel((7, 7)) == (0, 255, 0, 255)


def test_struct_undo_restores_move():
    doc = SpriteDoc(8, 8)
    doc.add_frame()
    doc.frames[0].name = "a"
    doc.frames[1].name = "b"
    doc.select(0)
    assert doc.move_frame(+1)
    assert [f.name for f in doc.frames] == ["b", "a"]
    assert doc.undo_struct()
    assert [f.name for f in doc.frames] == ["a", "b"]


def test_unified_sequence_pixel_vs_struct():
    # Juengste Aktion gewinnt: nach Pixel-Strich ist dessen Sequenz hoeher
    # als die des aelteren Struktur-Eintrags -- und umgekehrt.
    doc = SpriteDoc(8, 8)
    doc.add_frame()                      # Struktur-Eintrag
    assert doc.last_struct_undo_seq() > doc.current.last_undo_seq()
    doc.current.snapshot()               # Pixel-Strich danach
    assert doc.current.last_undo_seq() > doc.last_struct_undo_seq()


# --- Onion-Skin-Helfer ---------------------------------------------

def test_onion_indices_single_frame_empty():
    from gamebasic.spriteeditor.document import onion_indices
    assert onion_indices(0, 1, 1) == []
    assert onion_indices(0, 5, 0) == []


def test_onion_indices_basic_prev_next():
    from gamebasic.spriteeditor.document import onion_indices
    out = onion_indices(2, 5, 1)
    assert out == [(1, "blue", 1.0), (3, "red", 1.0)]


def test_onion_indices_two_frames_dedup():
    # Bei 2 Frames ist vorher == nachher -> nur EIN Eintrag (blau).
    from gamebasic.spriteeditor.document import onion_indices
    out = onion_indices(0, 2, 1)
    assert out == [(1, "blue", 1.0)]


def test_onion_indices_depth_falloff_and_no_current():
    from gamebasic.spriteeditor.document import onion_indices
    out = onion_indices(3, 8, 3)
    idxs = [e[0] for e in out]
    assert 3 not in idxs                      # aktuelles Frame nie dabei
    assert set(idxs) == {0, 1, 2, 4, 5, 6}
    # Distanz 2/3 blasser als Distanz 1
    by_idx = {i: f for (i, _m, f) in out}
    assert by_idx[2] == 1.0 and by_idx[4] == 1.0
    assert by_idx[1] < 1.0 and by_idx[0] < by_idx[1]


def test_onion_indices_depth_wraps_without_duplicates():
    # depth groesser als Frame-Anzahl: jeder Frame hoechstens einmal.
    from gamebasic.spriteeditor.document import onion_indices
    out = onion_indices(0, 3, 3)
    idxs = [e[0] for e in out]
    assert sorted(idxs) == [1, 2]


def test_onion_tinted_alpha_and_channels():
    from gamebasic.spriteeditor.document import onion_tinted
    img = Image.new("RGBA", (2, 2), (200, 100, 100, 255))
    blue = onion_tinted(img, "blue", 0.5)
    r, g, b, a = blue.getpixel((0, 0))
    assert a == 127                  # Alpha halbiert
    assert r == 100 and g == 70      # Rot/Gruen gedimmt
    assert b == 100                  # Blau unveraendert
    red = onion_tinted(img, "red", 0.5)
    r2, g2, b2, a2 = red.getpixel((0, 0))
    assert b2 == 50 and r2 == 200    # Blau gedimmt, Rot bleibt
    # Original unveraendert
    assert img.getpixel((0, 0)) == (200, 100, 100, 255)
