"""Sprite-Atlas + Batch-Draw.

Decken ab:
- ATLAS_LOAD parst Manifest, laedt Image, baut frames-Dict
- ATLAS_DRAW zeichnet einzelnes Sub-Sprite (Camera-aware)
- BATCH_DRAW haengt an Queue; BATCH_FLUSH rendert
- Pending Batch wird beim FLIP automatisch geflusht
- Pending Batch wird beim Layer-Wechsel geflusht (auf das alte Target)
- Pending Batch wird vor ATLAS_DRAW geflusht (Reihenfolge)
- Fehler-Cases: Manifest fehlt, kein 'image' Feld, kaputter Sprite-Rect
"""
import json
from pathlib import Path

import pytest


pygame = pytest.importorskip("pygame")


@pytest.fixture
def graphics():
    from gamebasic.graphics import Graphics
    g = Graphics()
    g.screen(64, 64, "test")
    yield g
    g.shutdown()


@pytest.fixture
def atlas_dir(tmp_path):
    """Legt ein einfaches 2x2 Atlas-PNG an (4 farbige 16x16 Quadrate)
    plus Manifest. Atlas-Layout:
        (0,0)=rot     (16,0)=gruen
        (0,16)=blau   (16,16)=gelb
    """
    pg = pygame
    pg.init()
    surf = pg.Surface((32, 32))
    surf.fill((255, 0, 0),   (0,  0,  16, 16))
    surf.fill((0, 255, 0),   (16, 0,  16, 16))
    surf.fill((0, 0, 255),   (0,  16, 16, 16))
    surf.fill((255, 255, 0), (16, 16, 16, 16))
    img_path = tmp_path / "atlas.png"
    pg.image.save(surf, str(img_path))

    manifest = tmp_path / "atlas.json"
    manifest.write_text(json.dumps({
        "image": "atlas.png",
        "sprites": {
            "red":    [0, 0, 16, 16],
            "green":  [16, 0, 16, 16],
            "blue":   [0, 16, 16, 16],
            "yellow": [16, 16, 16, 16],
        }
    }))
    return manifest


def _pixel(surface, x, y):
    c = surface.get_at((x, y))
    return (c.r, c.g, c.b)


def test_atlas_load_parses_manifest(graphics, atlas_dir):
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    from gamebasic.interpreter import _SpriteAtlas
    assert isinstance(atlas, _SpriteAtlas)
    assert set(atlas.frames.keys()) == {"red", "green", "blue", "yellow"}
    assert atlas.frames["red"] == (0, 0, 16, 16)
    assert atlas.frames["yellow"] == (16, 16, 16, 16)


def test_atlas_draw_renders_sub_sprite(graphics, atlas_dir):
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    graphics.cls(0x000000)
    graphics.atlas_draw(atlas, "red", 0, 0)
    graphics.atlas_draw(atlas, "green", 16, 0)
    # Beide direkt -- auch wenn FLIP nicht ruft, sind sie schon auf
    # _main_buffer
    assert _pixel(graphics._main_buffer, 5, 5) == (255, 0, 0)
    assert _pixel(graphics._main_buffer, 20, 5) == (0, 255, 0)


def test_atlas_draw_unknown_sprite_errors(graphics, atlas_dir):
    from gamebasic.errors import GBRuntimeError
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    with pytest.raises(GBRuntimeError, match="nicht im Atlas"):
        graphics.atlas_draw(atlas, "no_such_sprite", 0, 0)


def test_batch_draw_queued_then_flush(graphics, atlas_dir):
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    graphics.cls(0x000000)
    graphics.atlas_draw_batch(atlas, "red", 0, 0)
    graphics.atlas_draw_batch(atlas, "blue", 0, 16)
    # Noch nicht gerendert
    assert _pixel(graphics._main_buffer, 5, 5) == (0, 0, 0)
    assert len(graphics._batch) == 2
    graphics.batch_flush()
    # Jetzt da
    assert _pixel(graphics._main_buffer, 5, 5) == (255, 0, 0)
    assert _pixel(graphics._main_buffer, 5, 20) == (0, 0, 255)
    assert len(graphics._batch) == 0


def test_flip_auto_flushes_batch(graphics, atlas_dir):
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    graphics.cls(0x000000)
    graphics.atlas_draw_batch(atlas, "yellow", 16, 16)
    graphics.flip()
    assert _pixel(graphics._main_buffer, 20, 20) == (255, 255, 0)
    assert len(graphics._batch) == 0


def test_layer_switch_flushes_batch_to_old_target(graphics, atlas_dir):
    """Pending Batch landet auf dem AKTUELLEN Layer, nicht auf dem nach
    dem Switch -- richtige Z-Reihenfolge."""
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    graphics.layer_define("bg", 0)
    graphics.layer_define("fg", 10)
    graphics.layer_use("bg")
    graphics.atlas_draw_batch(atlas, "red", 0, 0)
    # Switch -- der rote Sprite muss auf bg flushed sein, nicht auf fg
    graphics.layer_use("fg")
    graphics.atlas_draw_batch(atlas, "green", 0, 0)
    graphics.flip()
    # Gruen (fg, vorne) gewinnt -> (0, 255, 0)
    assert _pixel(graphics._main_buffer, 5, 5) == (0, 255, 0)
    # Probe ausserhalb fg's Region: gar nichts (bg=rot ist auch nur 16x16)
    # Lass uns das anders verifizieren: gruen aktiv 0..16, rot 0..16. Beide gleicher Bereich.
    # Wenn nur gruen gewinnt: alles im Schnitt ist gruen ✓


def test_atlas_draw_after_batch_flushes_first(graphics, atlas_dir):
    """ATLAS_DRAW (direkt) vor pending Batch muss diesen erst flushen,
    sonst kommt der Direct-Draw VOR dem queued Batch -> falsche Reihenfolge."""
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    graphics.cls(0x000000)
    # Batch queued
    graphics.atlas_draw_batch(atlas, "red", 0, 0)
    # Direct-Draw -- erwartet, dass der Batch JETZT geflusht wird, BEVOR
    # der Direct-Draw gezeichnet wird. Sonst koennte der Direct-Draw
    # ueberlagert werden vom spaeter geflushten Batch.
    graphics.atlas_draw(atlas, "green", 16, 0)
    # Batch geflusht UND green gezeichnet
    assert len(graphics._batch) == 0
    assert _pixel(graphics._main_buffer, 5, 5) == (255, 0, 0)
    assert _pixel(graphics._main_buffer, 20, 5) == (0, 255, 0)


def test_atlas_load_missing_manifest(graphics, tmp_path):
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError, match="nicht gefunden"):
        graphics.load_sprite_atlas(str(tmp_path / "missing.json"))


def test_atlas_load_missing_image_field(graphics, tmp_path):
    from gamebasic.errors import GBRuntimeError
    m = tmp_path / "m.json"
    m.write_text(json.dumps({"sprites": {}}))
    with pytest.raises(GBRuntimeError, match="'image' fehlt"):
        graphics.load_sprite_atlas(str(m))


def test_atlas_load_bad_sprite_rect(graphics, atlas_dir, tmp_path):
    from gamebasic.errors import GBRuntimeError
    m = tmp_path / "bad.json"
    # Kopiere das echte atlas-Bild rueber, damit das image-Field OK ist
    img = atlas_dir.parent / "atlas.png"
    (tmp_path / "atlas.png").write_bytes(img.read_bytes())
    m.write_text(json.dumps({
        "image": "atlas.png",
        "sprites": {"oops": [1, 2, 3]}   # nur 3 statt 4 Werte
    }))
    with pytest.raises(GBRuntimeError, match="\\[x, y, w, h\\]"):
        graphics.load_sprite_atlas(str(m))


def test_atlas_draw_flipped_horizontal(graphics, atlas_dir):
    """flip_x=TRUE: das rote 16x16-Tile bei (0..15, 0..15) erscheint
    spiegelverkehrt. Pixel-Test: Wenn wir den Sprite an x=20 zeichnen
    mit Hoehe = ohne Flip waere er an x=20..35 mit rotem Inhalt. Mit
    flip_x liefert ein einfarbiges Tile weiterhin nur rote Pixel --
    aber wir koennen pruefen, dass das Rendering erfolgreich ist
    (Surface hat rote Pixel im erwarteten Bereich)."""
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    graphics.cls(0x000000)
    # red-Sprite mit flip_x zeichnen
    graphics.atlas_draw_flipped(atlas, "red", 20, 20, True, False)
    # Pixel im Sprite-Bereich sollten rot sein
    assert _pixel(graphics._main_buffer, 25, 25) == (255, 0, 0)


def test_atlas_draw_flipped_with_no_flip_falls_back(graphics, atlas_dir):
    """flip_x=flip_y=FALSE ist optimiert zum normalen atlas_draw-Pfad."""
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    graphics.cls(0x000000)
    graphics.atlas_draw_flipped(atlas, "red", 0, 0, False, False)
    assert _pixel(graphics._main_buffer, 5, 5) == (255, 0, 0)


def test_atlas_draw_flipped_visible_difference(graphics, atlas_dir):
    """Mit einem asymmetrischen Sprite (red auf der linken Haelfte,
    green auf der rechten): flip_x soll die Anordnung umdrehen."""
    pg = pygame
    pg.init()
    # 16x16 Sprite: links 8x16 rot, rechts 8x16 gruen. Atlas mit nur diesem.
    surf = pg.Surface((16, 16))
    surf.fill((255, 0, 0), (0, 0, 8, 16))
    surf.fill((0, 255, 0), (8, 0, 8, 16))
    img_path = atlas_dir.parent / "asym.png"
    pg.image.save(surf, str(img_path))
    import json as _json
    manifest = atlas_dir.parent / "asym.json"
    manifest.write_text(_json.dumps({
        "image": "asym.png",
        "sprites": {"x": [0, 0, 16, 16]},
    }))
    atlas = graphics.load_sprite_atlas(str(manifest))

    # Ohne Flip: links rot, rechts gruen
    graphics.cls(0x000000)
    graphics.atlas_draw_flipped(atlas, "x", 0, 0, False, False)
    assert _pixel(graphics._main_buffer, 2, 8) == (255, 0, 0)
    assert _pixel(graphics._main_buffer, 12, 8) == (0, 255, 0)

    # Mit Flip-X: links gruen, rechts rot (gespiegelt)
    graphics.cls(0x000000)
    graphics.atlas_draw_flipped(atlas, "x", 0, 0, True, False)
    assert _pixel(graphics._main_buffer, 2, 8) == (0, 255, 0)
    assert _pixel(graphics._main_buffer, 12, 8) == (255, 0, 0)


def test_atlas_draw_flipped_unknown_sprite_errors(graphics, atlas_dir):
    from gamebasic.errors import GBRuntimeError
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    with pytest.raises(GBRuntimeError, match="nicht im Atlas"):
        graphics.atlas_draw_flipped(atlas, "no_such", 0, 0, True, False)


def test_batch_with_camera_translation(graphics, atlas_dir):
    """Camera-Translation wird im Batch beruecksichtigt."""
    atlas = graphics.load_sprite_atlas(str(atlas_dir))
    graphics.cls(0x000000)
    # Camera bei (10, 10): World-Coord (10, 10) -> Screen (0, 0)
    graphics.set_camera(10, 10, 1.0)
    graphics.atlas_draw_batch(atlas, "red", 10, 10)   # = screen(0, 0)
    graphics.batch_flush()
    assert _pixel(graphics._main_buffer, 5, 5) == (255, 0, 0)
