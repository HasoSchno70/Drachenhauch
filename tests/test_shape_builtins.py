"""Tests fuer die neuen Shape-Built-ins:
TRIANGLE, TRIANGLEOUTLINE, POLYGON, POLYGONOUTLINE, ELLIPSE,
ELLIPSEOUTLINE, ARC.

Wir mocken die Graphics-Klasse so, dass die Built-ins die korrekten
Methoden mit den korrekten Parametern aufrufen - kein echtes Pygame.
"""
from unittest.mock import MagicMock

import pytest


@pytest.fixture
def graphics_mock():
    """Stellt einen Mock fuer die Graphics-Instanz bereit, der von
    den graphics_builtins direkt verwendet wird (ueber den Interpreter)."""
    g = MagicMock()
    return g


def _call_builtin(name: str, args: list, graphics):
    """Ruft einen Graphics-Builtin direkt auf, mit injizierter Graphics."""
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    fn = GRAPHICS_BUILTINS[name.lower()]
    return fn(graphics, args)


def test_triangle_calls_graphics_triangle(graphics_mock):
    _call_builtin("TRIANGLE", [10, 20, 30, 40, 50, 60], graphics_mock)
    graphics_mock.triangle.assert_called_once_with(
        10, 20, 30, 40, 50, 60, color=0xFFFFFF
    )


def test_triangle_with_color(graphics_mock):
    _call_builtin("TRIANGLE", [10, 20, 30, 40, 50, 60, 0xFF0000], graphics_mock)
    graphics_mock.triangle.assert_called_once_with(
        10, 20, 30, 40, 50, 60, color=0xFF0000
    )


def test_triangle_outline_with_width(graphics_mock):
    _call_builtin(
        "TRIANGLEOUTLINE", [10, 20, 30, 40, 50, 60, 0xFFFF00, 3],
        graphics_mock,
    )
    graphics_mock.triangle_outline.assert_called_once_with(
        10, 20, 30, 40, 50, 60, color=0xFFFF00, width=3,
    )


def test_polygon_with_array_of_integer(graphics_mock):
    """POLYGON nimmt eine Liste/Array von Punkt-Werten."""
    points = [100, 50, 150, 150, 50, 150]   # Dreieck
    _call_builtin("POLYGON", [points, 0x00FF00], graphics_mock)
    graphics_mock.polygon.assert_called_once_with(points, 0x00FF00)


def test_polygon_default_color(graphics_mock):
    points = [0, 0, 10, 0, 10, 10, 0, 10]   # Quad
    _call_builtin("POLYGON", [points], graphics_mock)
    graphics_mock.polygon.assert_called_once_with(points, 0xFFFFFF)


def test_polygon_outline_with_color_and_width(graphics_mock):
    points = [0, 0, 10, 0, 5, 10]
    _call_builtin("POLYGONOUTLINE", [points, 0xFF0000, 2], graphics_mock)
    graphics_mock.polygon_outline.assert_called_once_with(
        points, color=0xFF0000, width=2,
    )


def test_ellipse_basic(graphics_mock):
    _call_builtin("ELLIPSE", [10, 20, 110, 80, 0x00FFFF], graphics_mock)
    graphics_mock.ellipse.assert_called_once_with(
        10, 20, 110, 80, color=0x00FFFF
    )


def test_ellipse_outline_with_width(graphics_mock):
    _call_builtin(
        "ELLIPSEOUTLINE", [10, 20, 110, 80, 0xFF00FF, 4], graphics_mock,
    )
    graphics_mock.ellipse_outline.assert_called_once_with(
        10, 20, 110, 80, color=0xFF00FF, width=4,
    )


def test_arc_basic(graphics_mock):
    _call_builtin(
        "ARC", [10, 10, 110, 110, 0.0, 3.14159, 0xFFFFFF, 2], graphics_mock,
    )
    graphics_mock.arc.assert_called_once_with(
        10, 10, 110, 110, start_angle=0.0, end_angle=3.14159,
        color=0xFFFFFF, width=2,
    )


def test_arc_default_color_and_width(graphics_mock):
    _call_builtin("ARC", [0, 0, 100, 100, 0.0, 1.5707], graphics_mock)
    graphics_mock.arc.assert_called_once_with(
        0, 0, 100, 100, start_angle=0.0, end_angle=1.5707,
        color=0xFFFFFF, width=1,
    )
