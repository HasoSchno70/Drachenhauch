"""Z-Layer-Rendering.

Decken ab:
- LAYER_DEFINE + LAYER_USE + LAYER_END Pipeline
- Z-Order: niedriges z -> hinten, hohes z -> vorne
- Auto-Define bei unbekanntem Namen
- Update von z bei Re-Define
- Layer wird auf den Main-Buffer komponiert in FLIP
- Layer wird nach FLIP gecleart (transparent)
- Backwards-Compat: Code ohne Layer-Calls funktioniert wie vorher
"""
import pytest


pygame = pytest.importorskip("pygame")


@pytest.fixture
def graphics():
    """Frisches Graphics + SCREEN; Cleanup nach jedem Test."""
    from gamebasic.graphics import Graphics
    g = Graphics()
    g.screen(64, 64, "test")
    yield g
    g.shutdown()


def _get_pixel(surface, x, y):
    """RGB-Tuple lesen, alpha ignorieren."""
    c = surface.get_at((x, y))
    return (c.r, c.g, c.b)


def test_no_layer_backwards_compat(graphics):
    """Ohne LAYER-Calls geht alles direkt auf den Main-Buffer."""
    graphics.cls(0x000000)
    graphics.box(0, 0, 10, 10, 0xFF0000)
    assert _get_pixel(graphics._main_buffer, 5, 5) == (255, 0, 0)


def test_single_layer_compose(graphics):
    """Ein Layer wird beim FLIP auf den Main-Buffer komponiert."""
    graphics.layer_define("sprites", 10)
    graphics.layer_use("sprites")
    graphics.box(0, 0, 10, 10, 0x00FF00)
    # Vor FLIP: Main-Buffer noch leer
    assert _get_pixel(graphics._main_buffer, 5, 5) == (0, 0, 0)
    graphics.flip()
    # Nach FLIP: Layer wurde komponiert
    assert _get_pixel(graphics._main_buffer, 5, 5) == (0, 255, 0)


def test_layer_z_order(graphics):
    """Layer mit hoeherem z ueberdecken Layer mit niedrigerem z."""
    graphics.layer_define("bg", 0)
    graphics.layer_define("fg", 100)
    graphics.layer_use("bg")
    graphics.box(0, 0, 30, 30, 0xFF0000)   # rot hinten
    graphics.layer_use("fg")
    graphics.box(0, 0, 30, 30, 0x00FF00)   # gruen vorne
    graphics.flip()
    # Gruen gewinnt (vorne)
    assert _get_pixel(graphics._main_buffer, 15, 15) == (0, 255, 0)


def test_layer_z_order_reverse_definition(graphics):
    """Die Definition-Reihenfolge ist egal -- nur z zaehlt."""
    graphics.layer_define("fg", 100)       # zuerst definiert
    graphics.layer_define("bg", 0)         # spaeter definiert, aber niedrigeres z
    graphics.layer_use("bg")
    graphics.box(0, 0, 30, 30, 0xFF0000)
    graphics.layer_use("fg")
    graphics.box(0, 0, 30, 30, 0x00FF00)
    graphics.flip()
    assert _get_pixel(graphics._main_buffer, 15, 15) == (0, 255, 0)


def test_layer_end_returns_to_main(graphics):
    """LAYER_END schaltet zurueck auf den Main-Buffer."""
    graphics.layer_define("sprites", 10)
    graphics.layer_use("sprites")
    graphics.box(0, 0, 10, 10, 0x00FF00)
    graphics.layer_end()
    # Jetzt auf Main-Buffer direkt zeichnen (Koords sind x1,y1,x2,y2)
    graphics.box(20, 20, 30, 30, 0x0000FF)
    graphics.flip()
    # Beides sichtbar
    assert _get_pixel(graphics._main_buffer, 5, 5) == (0, 255, 0)
    assert _get_pixel(graphics._main_buffer, 25, 25) == (0, 0, 255)


def test_layer_cleared_after_flip(graphics):
    """Nach FLIP ist der Layer leer (transparent) -- naechstes Frame
    kann frisch zeichnen, ohne dass alter Inhalt nachhinkt."""
    graphics.layer_define("sprites", 10)
    graphics.layer_use("sprites")
    graphics.box(0, 0, 10, 10, 0xFF0000)
    graphics.flip()
    # Zweites Frame: Main-Buffer leer machen, kein neuer Draw auf den Layer
    graphics._main_buffer.fill((0, 0, 0))
    graphics.flip()
    # Layer war leer, hat nichts beigetragen
    assert _get_pixel(graphics._main_buffer, 5, 5) == (0, 0, 0)


def test_layer_redefine_updates_z(graphics):
    """LAYER_DEFINE mit gleichem Namen aktualisiert nur das z."""
    graphics.layer_define("fg", 5)
    graphics.layer_define("bg", 0)
    # Jetzt fg nach hinten verschieben:
    graphics.layer_define("fg", -1)
    graphics.layer_use("bg")
    graphics.box(0, 0, 30, 30, 0xFF0000)
    graphics.layer_use("fg")
    graphics.box(0, 0, 30, 30, 0x00FF00)
    graphics.flip()
    # bg gewinnt jetzt
    assert _get_pixel(graphics._main_buffer, 15, 15) == (255, 0, 0)


def test_layer_auto_define(graphics):
    """LAYER auf unbekanntem Namen erstellt den Layer mit auto-z (hinter
    allen existierenden)."""
    graphics.layer_define("bg", 0)
    graphics.layer_use("magic")   # auto-define mit z=1 (nach bg)
    graphics.box(0, 0, 30, 30, 0x00FF00)
    graphics.layer_use("bg")
    graphics.box(0, 0, 30, 30, 0xFF0000)
    graphics.flip()
    # magic hat hoeheres z -> gewinnt vor bg
    assert _get_pixel(graphics._main_buffer, 15, 15) == (0, 255, 0)


def test_flip_resets_current_layer(graphics):
    """Wenn der User LAYER_END vergisst, korrigiert FLIP das."""
    graphics.layer_define("sprites", 10)
    graphics.layer_use("sprites")
    graphics.box(0, 0, 10, 10, 0x00FF00)
    # User vergisst LAYER_END
    graphics.flip()
    # Nach FLIP: kein Layer mehr aktiv
    assert graphics._current_layer is None
    assert graphics._buffer is graphics._main_buffer


def test_layer_drawing_isolates(graphics):
    """Draws auf verschiedene Layer beeinflussen sich nicht."""
    graphics.layer_define("a", 0)
    graphics.layer_define("b", 10)
    graphics.layer_use("a")
    graphics.box(0, 0, 20, 20, 0xFF0000)
    graphics.layer_use("b")
    # b nicht ueberschneidend mit a's Region (Koords x1,y1,x2,y2)
    graphics.box(30, 30, 50, 50, 0x00FF00)
    graphics.flip()
    assert _get_pixel(graphics._main_buffer, 5, 5) == (255, 0, 0)
    assert _get_pixel(graphics._main_buffer, 45, 45) == (0, 255, 0)


def test_layer_clear_manual(graphics):
    """LAYER_CLEAR cleart einen Layer manuell zwischen FLIPs."""
    graphics.layer_define("sprites", 10)
    graphics.layer_use("sprites")
    graphics.box(0, 0, 10, 10, 0xFF0000)
    graphics.layer_clear("sprites")
    graphics.flip()
    # Nichts auf dem Main-Buffer
    assert _get_pixel(graphics._main_buffer, 5, 5) == (0, 0, 0)
