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
        self.selection_mask = None

    def invalidate_all(self):
        self.invalidate_count += 1

    def set_preview(self, pil):
        self.preview = pil

    def set_selection(self, x0, y0, x1, y1):
        self.selection = (x0, y0, x1, y1)
        self.selection_mask = None

    def get_selection(self):
        if self.selection is None:
            return None
        x0, y0, x1, y1 = self.selection
        return (min(x0, x1), min(y0, y1), max(x0, x1) + 1, max(y0, y1) + 1)

    def set_selection_mask(self, mask):
        bbox = mask.getbbox()
        if bbox is None:
            self.clear_selection()
            return
        x0, y0, x1, y1 = bbox
        self.selection_mask = mask
        self.selection = (x0, y0, x1 - 1, y1 - 1)

    def get_selection_mask(self):
        return self.selection_mask

    def clear_selection(self):
        self.selection = None
        self.selection_mask = None


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


def test_line_tool_respects_brush_size():
    """Brush-Size 3 -> die Linie ist 3 Pixel dick (quadratischer Stempel)."""
    from gamebasic.spriteeditor.tools import LineTool
    from PySide6.QtCore import Qt
    app = _MockApp(w=12, h=12)
    app.brush_size = 3
    tool = LineTool()
    tool.begin(app, 3, 5, Qt.LeftButton)
    tool.end(app, 7, 5)
    px = app.doc.current.pixels
    # Um die Mittel-Linie (y=5) herum sind y=4 und y=6 mitgefaerbt.
    for y in (4, 5, 6):
        assert px.getpixel((5, y)) == (255, 0, 0, 255)


def test_rect_tool_outline_brush_width():
    """Brush-Size 2 -> die Kontur ist 2 Pixel breit (nach innen gezeichnet)."""
    from gamebasic.spriteeditor.tools import RectTool
    from PySide6.QtCore import Qt
    app = _MockApp(w=12, h=12)
    app.brush_size = 2
    tool = RectTool(filled=False)
    tool.begin(app, 2, 2, Qt.LeftButton)
    tool.end(app, 9, 9)
    px = app.doc.current.pixels
    # Aeusserer Rand + eine Stelle nach innen sind gesetzt (Breite 2).
    assert px.getpixel((2, 2)) == (255, 0, 0, 255)
    assert px.getpixel((3, 3)) == (255, 0, 0, 255)
    # Tief im Inneren bleibt leer.
    assert px.getpixel((5, 5)) == (0, 0, 0, 0)


def test_spray_respects_symmetry():
    """Im X-Symmetrie-Modus hat jedes gespruehte Pixel sein Spiegelbild."""
    import random
    from gamebasic.spriteeditor.tools import SprayTool
    from PySide6.QtCore import Qt
    random.seed(1234)
    app = _MockApp(w=9, h=9)
    app.symmetry_mode = "x"
    app.brush_size = 4
    tool = SprayTool()
    tool.begin(app, 4, 4, Qt.LeftButton)
    for _ in range(10):
        tool.move(app, 4, 4)
    tool.end(app, 4, 4)
    px = app.doc.current.pixels
    painted = [(x, y) for y in range(9) for x in range(9)
               if px.getpixel((x, y))[3] != 0]
    assert painted, "Spray hat nichts gemalt"
    # Invariante: zu jedem Pixel ist das X-Spiegelbild ebenfalls gesetzt.
    for x, y in painted:
        assert px.getpixel((9 - 1 - x, y))[3] != 0


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


# --- SelectTool: Auswahl-Inhalt verschieben ---------------------------------

def _select_host(w=8, h=8):
    """Mock-App mit stateful Canvas (Selection + Preview) fuer Float-Move."""
    from PIL import Image

    class _Canvas:
        def __init__(self):
            self.selection = None
            self.preview = None
        def invalidate_all(self): pass
        def set_preview(self, pil): self.preview = pil
        def set_selection(self, x0, y0, x1, y1):
            self.selection = (x0, y0, x1, y1)
        def get_selection(self):
            if not self.selection:
                return None
            x0, y0, x1, y1 = self.selection
            return (min(x0, x1), min(y0, y1), max(x0, x1) + 1, max(y0, y1) + 1)

    class _Frame:
        def __init__(self):
            self.pixels = Image.new("RGBA", (w, h), (0, 0, 0, 0))
            self.snapshots = 0
        def snapshot(self): self.snapshots += 1

    class _Doc:
        def __init__(self):
            self.width, self.height = w, h
            self.current = _Frame()

    class _SB:
        def showMessage(self, *a, **k): pass

    class _App:
        def __init__(self):
            self.doc = _Doc()
            self.canvas = _Canvas()
        def mark_dirty(self): pass
        def statusBar(self): return _SB()
        def show_selection_context_menu(self, _p): pass

    return _App()


def test_select_drag_inside_moves_content():
    from gamebasic.spriteeditor.tools import SelectTool
    app = _select_host()
    px = app.doc.current.pixels
    px.putpixel((1, 1), (255, 0, 0, 255))
    px.putpixel((2, 2), (0, 255, 0, 255))
    app.canvas.set_selection(1, 1, 2, 2)     # Auswahl ueber beide Pixel

    tool = SelectTool()
    tool.begin(app, 1, 1, None)              # Klick IN die Auswahl -> greifen
    assert app.doc.current.snapshots == 1    # undoable
    tool.move(app, 4, 3)                     # um (+3, +2) ziehen
    assert app.canvas.preview is not None    # Live-Float sichtbar
    tool.end(app, 4, 3)

    out = app.doc.current.pixels
    assert out.getpixel((4, 3)) == (255, 0, 0, 255)
    assert out.getpixel((5, 4)) == (0, 255, 0, 255)
    assert out.getpixel((1, 1)) == (0, 0, 0, 0)      # Quelle geraeumt
    assert app.canvas.preview is None                # Preview abgebaut
    # Auswahl-Rechteck ist mitgewandert
    assert app.canvas.get_selection() == (4, 3, 6, 5)


def test_select_drag_outside_starts_new_selection():
    from gamebasic.spriteeditor.tools import SelectTool
    app = _select_host()
    app.canvas.set_selection(1, 1, 2, 2)
    tool = SelectTool()
    tool.begin(app, 5, 5, None)              # Klick AUSSERHALB -> neue Auswahl
    tool.move(app, 6, 6)
    tool.end(app, 6, 6)
    assert app.canvas.get_selection() == (5, 5, 7, 7)
    assert app.doc.current.snapshots == 0    # kein Pixel-Edit


def test_select_move_clips_at_canvas_edge():
    from gamebasic.spriteeditor.tools import SelectTool
    app = _select_host()
    app.doc.current.pixels.putpixel((1, 1), (255, 0, 0, 255))
    app.canvas.set_selection(1, 1, 1, 1)
    tool = SelectTool()
    tool.begin(app, 1, 1, None)
    tool.move(app, -1, 1)                    # ueber den linken Rand schieben
    tool.end(app, -1, 1)
    out = app.doc.current.pixels
    assert out.getpixel((1, 1)) == (0, 0, 0, 0)      # weg von der Quelle
    # Pixel ist links rausgeschoben -> nirgendwo mehr sichtbar, kein Crash
    assert all(out.getpixel((x, 1)) == (0, 0, 0, 0) for x in range(8))


# --- Lasso ----------------------------------------------------------

def test_polygon_mask_triangle():
    from gamebasic.spriteeditor.tools import polygon_mask
    mask = polygon_mask((8, 8), [(0, 0), (7, 0), (0, 7)])
    assert mask.getpixel((1, 1)) == 255       # innen
    assert mask.getpixel((7, 7)) == 0         # ausserhalb des Dreiecks
    assert mask.getbbox() is not None


def test_polygon_mask_too_few_points_empty():
    from gamebasic.spriteeditor.tools import polygon_mask
    assert polygon_mask((8, 8), [(0, 0), (3, 3)]).getbbox() is None


def test_lasso_drag_sets_mask_selection():
    from gamebasic.spriteeditor.tools import LassoTool
    from PySide6.QtCore import Qt
    app = _MockApp()
    tool = LassoTool()
    tool.begin(app, 1, 1, Qt.LeftButton)
    tool.move(app, 6, 1)
    tool.move(app, 6, 6)
    tool.move(app, 1, 6)
    tool.end(app, 1, 6)
    mask = app.canvas.get_selection_mask()
    assert mask is not None
    assert mask.getpixel((3, 3)) == 255
    assert mask.getpixel((0, 0)) == 0
    # Preview ist nach dem Loslassen weg
    assert app.canvas.preview is None
    # Bounding-Box als Rect-Selection gesetzt
    assert app.canvas.get_selection() == (1, 1, 7, 7)


def test_lasso_move_inside_moves_masked_pixels_only():
    from gamebasic.spriteeditor.tools import LassoTool, polygon_mask
    from PySide6.QtCore import Qt
    app = _MockApp()
    px = app.doc.current.pixels
    # Inhalt: maskierter Pixel (2,2) rot, unmaskierter (0,0) gruen
    px.putpixel((2, 2), (255, 0, 0, 255))
    px.putpixel((0, 0), (0, 255, 0, 255))
    app.canvas.set_selection_mask(polygon_mask((8, 8), [(1, 1), (4, 1), (4, 4), (1, 4)]))
    tool = LassoTool()
    # Klick auf maskierten Pixel -> Float-Move um (+2, +2)
    tool.begin(app, 2, 2, Qt.LeftButton)
    tool.move(app, 4, 4)
    tool.end(app, 4, 4)
    out = app.doc.current.pixels
    assert out.getpixel((4, 4)) == (255, 0, 0, 255)   # mit verschoben
    assert out.getpixel((2, 2)) == (0, 0, 0, 0)       # Quelle geraeumt
    assert out.getpixel((0, 0)) == (0, 255, 0, 255)   # unmaskiert bleibt
    # Maske ist mitgewandert
    mask = app.canvas.get_selection_mask()
    assert mask is not None
    assert mask.getpixel((4, 4)) == 255
    assert mask.getpixel((1, 1)) == 0
    # Ein Undo-Schritt (Snapshot beim Greifen)
    assert app.doc.current.snapshot_count == 1


def test_lasso_click_outside_mask_starts_new_path():
    from gamebasic.spriteeditor.tools import LassoTool, polygon_mask
    from PySide6.QtCore import Qt
    app = _MockApp()
    app.canvas.set_selection_mask(polygon_mask((8, 8), [(5, 5), (7, 5), (7, 7), (5, 7)]))
    tool = LassoTool()
    tool.begin(app, 0, 0, Qt.LeftButton)       # ausserhalb der Maske
    tool.move(app, 3, 0)
    tool.move(app, 0, 3)
    tool.end(app, 0, 3)
    mask = app.canvas.get_selection_mask()
    assert mask is not None
    assert mask.getpixel((1, 1)) == 255        # neue Auswahl
    assert mask.getpixel((6, 6)) == 0          # alte ist ersetzt
