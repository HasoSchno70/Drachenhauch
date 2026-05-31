"""Tests fuer die extrahierten Sprite-Editor-Tools.

Wir bauen einen Mock-`app` (Duck-Type) der die Tool-Erwartungen abdeckt:
`doc.current.pixels`, `fg`, `bg`, `brush_size`, `symmetry_mode`, `canvas`
und ein paar Helper-Methoden. Die Canvas-Methoden (`invalidate_all`,
`set_preview`, `set_selection`) sind als no-op-Stubs realisiert -- die
Tools mutieren die PIL-Bilder direkt, das laesst sich vorm Mocking
verifizieren.
"""
import os

import pytest
from PIL import Image

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    """Tools importieren PySide6.Qt, brauchen aber keinen QApplication-
    Mainloop. Wir erzeugen einen, falls noch keiner da ist."""
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


# --- Mock-App -------------------------------------------------------

class _MockCanvas:
    def __init__(self):
        self.invalidate_count = 0
        self.preview = None
        self.selection = None

    def invalidate_all(self):
        self.invalidate_count += 1

    def set_preview(self, pil):
        self.preview = pil

    def set_selection(self, x0, y0, x1, y1):
        self.selection = (x0, y0, x1, y1)

    def get_selection(self):
        if self.selection is None:
            return None
        x0, y0, x1, y1 = self.selection
        return (min(x0, x1), min(y0, y1), max(x0, x1) + 1, max(y0, y1) + 1)


class _MockFrame:
    def __init__(self, w, h):
        self.pixels = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        self.snapshot_count = 0

    def snapshot(self):
        self.snapshot_count += 1


class _MockDoc:
    def __init__(self, w=8, h=8):
        self.width = w
        self.height = h
        self.current = _MockFrame(w, h)


class _MockApp:
    def __init__(self, w=8, h=8):
        self.doc = _MockDoc(w, h)
        self.fg = (255, 0, 0, 255)
        self.bg = (0, 0, 255, 255)
        self.brush_size = 1
        self.symmetry_mode = "none"
        self.canvas = _MockCanvas()
        self.dirty_count = 0
        self._previous_tool_name = None
        self._activated = []

    def in_bounds(self, x, y):
        return 0 <= x < self.doc.width and 0 <= y < self.doc.height

    def mark_dirty(self):
        self.dirty_count += 1

    def set_fg(self, c):
        self.fg = c

    def set_bg(self, c):
        self.bg = c

    def activate_tool(self, name, _silent=False):
        self._activated.append(name)

    def statusBar(self):
        class _SB:
            def showMessage(self, *_a, **_kw):
                pass
        return _SB()

    def show_selection_context_menu(self, _pos):
        pass


# --- Pencil ---------------------------------------------------------

def test_pencil_paints_single_pixel():
    from gamebasic.spriteeditor.tools import PencilTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    tool = PencilTool()
    tool.begin(app, 3, 4, Qt.LeftButton)
    tool.end(app, 3, 4)
    assert app.doc.current.pixels.getpixel((3, 4)) == (255, 0, 0, 255)
    assert app.doc.current.snapshot_count == 1
    assert app.dirty_count == 1


def test_pencil_uses_bg_with_right_button():
    from gamebasic.spriteeditor.tools import PencilTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    tool = PencilTool()
    tool.begin(app, 0, 0, Qt.RightButton)
    tool.end(app, 0, 0)
    assert app.doc.current.pixels.getpixel((0, 0)) == (0, 0, 255, 255)


def test_pencil_drag_paints_line():
    """Move zwischen zwei Punkten zieht eine Bresenham-Linie."""
    from gamebasic.spriteeditor.tools import PencilTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    tool = PencilTool()
    tool.begin(app, 0, 0, Qt.LeftButton)
    tool.move(app, 4, 0)
    tool.end(app, 4, 0)
    # Alle Pixel in (0..4, 0) sollten gefaerbt sein
    for x in range(5):
        assert app.doc.current.pixels.getpixel((x, 0)) == (255, 0, 0, 255)


def test_pencil_respects_bounds():
    from gamebasic.spriteeditor.tools import PencilTool
    from PySide6.QtCore import Qt
    app = _MockApp(w=4, h=4)
    tool = PencilTool()
    tool.begin(app, 100, 100, Qt.LeftButton)
    tool.end(app, 100, 100)
    # Out-of-bounds -- nichts sollte gefaerbt sein
    for x in range(4):
        for y in range(4):
            assert app.doc.current.pixels.getpixel((x, y)) == (0, 0, 0, 0)


# --- Eraser ---------------------------------------------------------

def test_eraser_clears_pixel():
    from gamebasic.spriteeditor.tools import EraserTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    app.doc.current.pixels.putpixel((2, 2), (100, 100, 100, 255))
    tool = EraserTool()
    tool.begin(app, 2, 2, Qt.LeftButton)
    tool.end(app, 2, 2)
    assert app.doc.current.pixels.getpixel((2, 2)) == (0, 0, 0, 0)


# --- Bucket ---------------------------------------------------------

def test_bucket_fills_connected_region():
    from gamebasic.spriteeditor.tools import BucketTool
    from PySide6.QtCore import Qt
    app = _MockApp(w=4, h=4)
    # Zweite Spalte abtrennen damit nur die linke Haelfte gefuellt wird
    for y in range(4):
        app.doc.current.pixels.putpixel((1, y), (50, 50, 50, 255))
    tool = BucketTool()
    tool.begin(app, 0, 0, Qt.LeftButton)
    # Linke Spalte (x=0) gefuellt
    for y in range(4):
        assert app.doc.current.pixels.getpixel((0, y)) == (255, 0, 0, 255)
    # Trenner unveraendert
    for y in range(4):
        assert app.doc.current.pixels.getpixel((1, y)) == (50, 50, 50, 255)
    # Rechte Haelfte unveraendert (transparent)
    for y in range(4):
        for x in range(2, 4):
            assert app.doc.current.pixels.getpixel((x, y)) == (0, 0, 0, 0)


def test_bucket_no_change_if_target_is_replacement():
    """Klick auf ein Pixel, das schon die Ziel-Farbe hat: snapshot trotzdem,
    aber kein Pixel-Update (early-return). Das ist konsistent mit
    klassischem Verhalten."""
    from gamebasic.spriteeditor.tools import BucketTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    app.doc.current.pixels.putpixel((1, 1), (255, 0, 0, 255))  # = fg
    tool = BucketTool()
    tool.begin(app, 1, 1, Qt.LeftButton)
    # Snapshot wurde genommen, aber Bild unveraendert
    assert app.doc.current.pixels.getpixel((1, 1)) == (255, 0, 0, 255)


# --- Line / Rect / Ellipse -----------------------------------------

def test_line_tool_draws_diagonal():
    from gamebasic.spriteeditor.tools import LineTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    tool = LineTool()
    tool.begin(app, 0, 0, Qt.LeftButton)
    tool.end(app, 4, 4)
    # Diagonal -- alle (i, i) sollten gefaerbt sein
    for i in range(5):
        assert app.doc.current.pixels.getpixel((i, i)) == (255, 0, 0, 255)


def test_rect_tool_outline():
    from gamebasic.spriteeditor.tools import RectTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    tool = RectTool(filled=False)
    tool.begin(app, 1, 1, Qt.LeftButton)
    tool.end(app, 3, 3)
    # Rand ist gesetzt
    assert app.doc.current.pixels.getpixel((1, 1)) == (255, 0, 0, 255)
    assert app.doc.current.pixels.getpixel((3, 3)) == (255, 0, 0, 255)
    # Mitte unverand. (Outline-Rect)
    assert app.doc.current.pixels.getpixel((2, 2)) == (0, 0, 0, 0)


def test_rect_tool_filled():
    from gamebasic.spriteeditor.tools import RectTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    tool = RectTool(filled=True)
    tool.begin(app, 1, 1, Qt.LeftButton)
    tool.end(app, 3, 3)
    # Mitte UND Raender gesetzt
    assert app.doc.current.pixels.getpixel((2, 2)) == (255, 0, 0, 255)


def test_two_point_tool_preview_during_drag():
    """Live-Preview waehrend des Drags veraendert nicht den echten Buffer."""
    from gamebasic.spriteeditor.tools import LineTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    tool = LineTool()
    tool.begin(app, 0, 0, Qt.LeftButton)
    tool.move(app, 5, 5)
    # Vor end(): echter Buffer noch leer, Preview ist gesetzt
    assert app.doc.current.pixels.getpixel((3, 3)) == (0, 0, 0, 0)
    assert app.canvas.preview is not None


# --- Eyedropper ----------------------------------------------------

def test_eyedropper_picks_color():
    from gamebasic.spriteeditor.tools import EyedropperTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    app.doc.current.pixels.putpixel((2, 2), (10, 20, 30, 255))
    tool = EyedropperTool()
    tool.begin(app, 2, 2, Qt.LeftButton)
    assert app.fg == (10, 20, 30, 255)


def test_eyedropper_does_not_snapshot():
    """Eyedropper hat needs_snapshot=False -- darf nichts mutieren."""
    from gamebasic.spriteeditor.tools import EyedropperTool
    assert EyedropperTool.needs_snapshot is False


# --- Magic Wand ----------------------------------------------------

def test_magic_wand_selects_bounding_rect():
    from gamebasic.spriteeditor.tools import MagicWandTool
    from PySide6.QtCore import Qt
    app = _MockApp(w=8, h=8)
    # 3x3-Block aus gleicher Farbe
    for x in range(2, 5):
        for y in range(3, 6):
            app.doc.current.pixels.putpixel((x, y), (100, 0, 0, 255))
    tool = MagicWandTool()
    tool.begin(app, 3, 4, Qt.LeftButton)
    sel = app.canvas.selection
    assert sel == (2, 3, 4, 5)


# --- Symmetrie-Modus -----------------------------------------------

def test_pencil_with_x_symmetry():
    from gamebasic.spriteeditor.tools import PencilTool
    from PySide6.QtCore import Qt
    app = _MockApp(w=8, h=8)
    app.symmetry_mode = "x"
    tool = PencilTool()
    tool.begin(app, 1, 4, Qt.LeftButton)
    tool.end(app, 1, 4)
    # Original an (1, 4)
    assert app.doc.current.pixels.getpixel((1, 4)) == (255, 0, 0, 255)
    # Spiegel an (8-1-1, 4) = (6, 4)
    assert app.doc.current.pixels.getpixel((6, 4)) == (255, 0, 0, 255)


def test_pencil_with_both_symmetry():
    from gamebasic.spriteeditor.tools import PencilTool
    from PySide6.QtCore import Qt
    app = _MockApp(w=8, h=8)
    app.symmetry_mode = "both"
    tool = PencilTool()
    tool.begin(app, 1, 1, Qt.LeftButton)
    tool.end(app, 1, 1)
    # Vier Spiegel-Punkte
    assert app.doc.current.pixels.getpixel((1, 1)) == (255, 0, 0, 255)
    assert app.doc.current.pixels.getpixel((6, 1)) == (255, 0, 0, 255)
    assert app.doc.current.pixels.getpixel((1, 6)) == (255, 0, 0, 255)
    assert app.doc.current.pixels.getpixel((6, 6)) == (255, 0, 0, 255)


# --- Helpers -------------------------------------------------------

def test_bresenham_horizontal():
    from gamebasic.spriteeditor.tools import _bresenham
    pts = list(_bresenham(0, 0, 4, 0))
    assert pts == [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]


def test_bresenham_diagonal():
    from gamebasic.spriteeditor.tools import _bresenham
    pts = list(_bresenham(0, 0, 3, 3))
    assert pts == [(0, 0), (1, 1), (2, 2), (3, 3)]


def test_brush_offsets_size_1_is_single_pixel():
    from gamebasic.spriteeditor.tools import _brush_offsets
    offs = _brush_offsets(1)
    assert (0, 0) in offs


def test_brush_offsets_size_grows():
    from gamebasic.spriteeditor.tools import _brush_offsets
    s1 = _brush_offsets(1)
    s4 = _brush_offsets(4)
    assert len(s4) > len(s1)


def test_symmetry_points_none():
    from gamebasic.spriteeditor.tools import _symmetry_points
    app = _MockApp(w=8, h=8)
    app.symmetry_mode = "none"
    assert _symmetry_points(app, 2, 3) == [(2, 3)]


def test_symmetry_points_both():
    from gamebasic.spriteeditor.tools import _symmetry_points
    app = _MockApp(w=8, h=8)
    app.symmetry_mode = "both"
    pts = sorted(_symmetry_points(app, 1, 2))
    assert pts == [(1, 2), (1, 5), (6, 2), (6, 5)]


# --- Tool-Protocol-Vertraege --------------------------------------

def test_tool_base_class_has_required_attrs():
    from gamebasic.spriteeditor.tools import Tool
    assert hasattr(Tool, "name")
    assert hasattr(Tool, "needs_snapshot")
    assert hasattr(Tool, "begin")
    assert hasattr(Tool, "move")
    assert hasattr(Tool, "end")


def test_all_tools_inherit_from_base():
    from gamebasic.spriteeditor.tools import (
        Tool, PencilTool, EraserTool, BucketTool, LineTool, RectTool,
        EllipseTool, EyedropperTool, MoveTool, MagicWandTool, SelectTool,
        SprayTool,
    )
    for cls in [PencilTool, EraserTool, BucketTool, LineTool, RectTool,
                EllipseTool, EyedropperTool, MoveTool, MagicWandTool,
                SelectTool, SprayTool]:
        # MoveTool/RectTool/EllipseTool wollen einen Construktor-Arg haben
        # bei filled, also nicht direkt instanziieren -- nur Subclass-Check.
        assert issubclass(cls, Tool), f"{cls.__name__} ist keine Tool-Subclass"
