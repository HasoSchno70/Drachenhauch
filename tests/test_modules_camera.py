"""Tests fuer die Kamera-Mathematik (auf der behaltenen `Graphics`-Klasse).

Die Kamera-Built-ins sind graphics_builtin (nativ in dhrt); die zugrunde liegende
Mathematik (set/get/reset_camera, _w2s/_s2w/_scale_size) lebt in `graphics.py`
(Stufe A behalten) und wird hier direkt getestet -- unabhaengig vom Builtin-Layer.
"""
import pytest

from gamebasic.graphics import Graphics
from gamebasic.errors import GBRuntimeError


@pytest.fixture
def g():
    return Graphics()


def test_default_camera_is_identity(g):
    cx, cy, cz = g.get_camera()
    assert (cx, cy, cz) == (0.0, 0.0, 1.0)


def test_set_camera(g):
    g.set_camera(100, 200, 2.0)
    assert g.get_camera() == (100.0, 200.0, 2.0)


def test_zero_zoom_raises(g):
    with pytest.raises(GBRuntimeError, match="zoom muss > 0"):
        g.set_camera(0, 0, 0)


def test_negative_zoom_raises(g):
    with pytest.raises(GBRuntimeError, match="zoom muss > 0"):
        g.set_camera(0, 0, -1)


def test_reset(g):
    g.set_camera(100, 200, 2.0)
    g.reset_camera()
    assert g.get_camera() == (0.0, 0.0, 1.0)


def test_w2s_identity(g):
    assert g._w2s(50, 50) == (50, 50)


def test_w2s_translation(g):
    g.set_camera(100, 200, 1.0)
    assert g._w2s(150, 250) == (50, 50)
    assert g._w2s(100, 200) == (0, 0)


def test_w2s_zoom(g):
    g.set_camera(0, 0, 2.0)
    assert g._w2s(10, 20) == (20, 40)


def test_w2s_translation_and_zoom(g):
    g.set_camera(100, 100, 2.0)
    # World (110, 110) -> (110-100)*2 = (20, 20)
    assert g._w2s(110, 110) == (20, 20)


def test_s2w_inverse_of_w2s(g):
    g.set_camera(50, 75, 2.0)
    sx, sy = g._w2s(123.5, 456.5)
    wx, wy = g._s2w(sx, sy)
    # Round-trip leidet unter int-Cast in _w2s, aber sollte nahe sein
    assert abs(wx - 123.5) < 1.0
    assert abs(wy - 456.5) < 1.0


def test_scale_size(g):
    g.set_camera(0, 0, 2.0)
    assert g._scale_size(10) == 20
    g.set_camera(0, 0, 0.5)
    assert g._scale_size(10) == 5
