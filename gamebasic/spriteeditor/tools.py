"""Pixel-Manipulation-Tools fuer den Sprite-Editor.

Tool-Konvention:
- `name`-Attribut: Identifier, der in `app.activate_tool(name)` referenziert
  wird (auch fuer Tastenkuerzel-Mappings und Toolbar-Buttons).
- `needs_snapshot`: ob der Tool-Lifecycle einen Doc-Snapshot fuer Undo
  braucht. `True` (default) -- in `begin()` ruft das Tool selbst
  `app.doc.current.snapshot()`. Tools wie Eyedropper/Magic-Wand setzen
  das auf `False`, weil sie nichts mutieren.
- `begin(app, x, y, button)` / `move(app, x, y)` / `end(app, x, y)`:
  klassischer Drag-Lifecycle. Alle drei sind optional implementierbar;
  Tools die nur Click brauchen, fuellen nur `begin`.

Tools nehmen einen `app`-Parameter, der dem `ToolHost`-Protocol entspricht
(siehe `tool_context.py`). Erwartete Attribute:
  app.doc, app.fg, app.bg, app.brush_size, app.symmetry_mode, app.canvas
  app.in_bounds(x, y), app.mark_dirty(), app.set_fg(c), app.set_bg(c),
  app.activate_tool(name, _silent=False), app.statusBar()
  app.show_selection_context_menu(global_pos)

Aufgespalten aus dem ehemaligen Monolith `spriteeditor_qt.py` -- Imports
laufen in beide Richtungen via Re-Export.
"""
from __future__ import annotations

import random as _random
from typing import TYPE_CHECKING, Iterable

from PIL import Image, ImageDraw
from PySide6.QtCore import Qt

if TYPE_CHECKING:
    # Nur fuer Type-Hints; zur Laufzeit nicht importiert (kein Zirkular).
    from .tool_context import ToolHost  # noqa: F401


# ============================================================
# Tools (Pixel-Manipulation auf SpriteDoc)
# ============================================================

def _bresenham(x0, y0, x1, y1) -> Iterable[tuple[int, int]]:
    dx = abs(x1 - x0); dy = -abs(y1 - y0)
    sx = 1 if x0 < x1 else -1
    sy = 1 if y0 < y1 else -1
    err = dx + dy
    while True:
        yield x0, y0
        if x0 == x1 and y0 == y1:
            return
        e2 = 2 * err
        if e2 >= dy:
            err += dy; x0 += sx
        if e2 <= dx:
            err += dx; y0 += sy


def _brush_offsets(size: int) -> tuple:
    if size <= 1:
        return ((0, 0),)
    half = size // 2
    return tuple((dx, dy)
                 for dy in range(-half, size - half)
                 for dx in range(-half, size - half))


class Tool:
    name = "tool"
    needs_snapshot = True

    def begin(self, app, x, y, button): ...
    def move(self, app, x, y): ...
    def end(self, app, x, y): ...


def _symmetry_points(app, x: int, y: int) -> list[tuple[int, int]]:
    """Liefert alle Spiegel-Positionen fuer einen Stamp-Punkt anhand des
    aktuellen Symmetrie-Modus. 'none' -> nur (x, y), 'x' -> plus
    horizontal gespiegelt, 'y' -> plus vertikal, 'both' -> alle vier."""
    pts = [(x, y)]
    sym = app.symmetry_mode
    if sym == "none":
        return pts
    w, h = app.doc.width, app.doc.height
    if sym in ("x", "both"):
        pts.append((w - 1 - x, y))
    if sym in ("y", "both"):
        pts.append((x, h - 1 - y))
    if sym == "both":
        pts.append((w - 1 - x, h - 1 - y))
    return pts


class _StampTool(Tool):
    """Pencil/Eraser-Basis. `_stamp_color` ueberschreiben.

    Beruecksichtigt den App-Symmetrie-Modus: jeder Pixel wird optional an
    der X-, Y- oder beiden Achsen gespiegelt mitgesetzt.
    """
    def begin(self, app, x, y, button):
        app.doc.current.snapshot()
        self._last = (x, y)
        self._color = self._stamp_color(app, button)
        self._offsets = _brush_offsets(app.brush_size)
        self._stamp(app, x, y)

    def move(self, app, x, y):
        if getattr(self, "_last", None) is None:
            return
        for px, py in _bresenham(self._last[0], self._last[1], x, y):
            self._stamp(app, px, py)
        self._last = (x, y)

    def end(self, app, x, y):
        self._last = None
        app.mark_dirty()

    def _stamp_color(self, app, button):
        return app.fg if button == Qt.LeftButton else app.bg

    def _stamp(self, app, x, y):
        pixels = app.doc.current.pixels
        for cx, cy in _symmetry_points(app, x, y):
            for dx, dy in self._offsets:
                xx, yy = cx + dx, cy + dy
                if app.in_bounds(xx, yy):
                    pixels.putpixel((xx, yy), self._color)
        app.canvas.invalidate_all()


class PencilTool(_StampTool):
    name = "pencil"


class EraserTool(_StampTool):
    name = "eraser"

    def _stamp_color(self, app, button):
        return (0, 0, 0, 0)


class SprayTool(Tool):
    """Spray-Tool: verteilt zufaellig Pixel innerhalb eines Radius
    rund um den Cursor. Der Radius skaliert mit der Brush-Size,
    Density (Anteil der Pixel pro Tick) ist fix.
    """
    name = "spray"

    def begin(self, app, x, y, button):
        app.doc.current.snapshot()
        self._color = app.fg if button == Qt.LeftButton else app.bg
        # Brush-Size 1..4 -> Spray-Radius 2..6
        self._radius = app.brush_size + 2
        # Pixel pro Tick
        self._density = 0.30
        self._spray(app, x, y)

    def move(self, app, x, y):
        self._spray(app, x, y)

    def end(self, app, x, y):
        app.mark_dirty()

    def _spray(self, app, cx, cy):
        import random
        r = self._radius
        # Anzahl gestreuter Pixel pro Tick: prop. zur Flaeche * Density
        n = max(1, int(r * r * self._density))
        pixels = app.doc.current.pixels
        for _ in range(n):
            # Rejection-Sampling fuer Kreis-uniforme Verteilung
            for _try in range(8):
                dx = random.randint(-r, r)
                dy = random.randint(-r, r)
                if dx * dx + dy * dy <= r * r:
                    break
            xx, yy = cx + dx, cy + dy
            # Symmetrie-Modus wie bei Stift/Radierer respektieren.
            for sxx, syy in _symmetry_points(app, xx, yy):
                if app.in_bounds(sxx, syy):
                    pixels.putpixel((sxx, syy), self._color)
        app.canvas.invalidate_all()


class BucketTool(Tool):
    name = "bucket"

    def begin(self, app, x, y, button):
        if not app.in_bounds(x, y):
            return
        app.doc.current.snapshot()
        target = app.doc.current.pixels.getpixel((x, y))
        replacement = app.fg if button == Qt.LeftButton else app.bg
        if target == replacement:
            return
        self._flood(app.doc.current.pixels, x, y, target, replacement)
        app.canvas.invalidate_all()
        app.mark_dirty()

    @staticmethod
    def _flood(img, sx, sy, target, replacement):
        w, h = img.size
        stack = [(sx, sy)]
        px = img.load()
        while stack:
            x, y = stack.pop()
            if x < 0 or x >= w or y < 0 or y >= h:
                continue
            if px[x, y] != target:
                continue
            px[x, y] = replacement
            stack.append((x + 1, y))
            stack.append((x - 1, y))
            stack.append((x, y + 1))
            stack.append((x, y - 1))


class _TwoPointTool(Tool):
    """Linie/Rechteck/Ellipse: live-Preview waehrend des Drags, commit bei
    Release. Der Preview wird ueber app.canvas.set_preview(img) gesetzt."""
    def begin(self, app, x, y, button):
        app.doc.current.snapshot()
        self._start = (x, y)
        self._color = app.fg if button == Qt.LeftButton else app.bg
        # Brush-Size 1..4 wie bei Stift/Radierer: Linien-/Konturbreite. VOR
        # dem ersten _preview setzen (das _draw aufruft).
        self._brush = max(1, getattr(app, "brush_size", 1))
        self._preview(app, x, y)

    def move(self, app, x, y):
        self._preview(app, x, y)

    def end(self, app, x, y):
        app.canvas.set_preview(None)
        self._draw(app.doc.current.pixels, self._start[0], self._start[1],
                   x, y, self._color)
        app.canvas.invalidate_all()
        app.mark_dirty()

    def _preview(self, app, x, y):
        prev = app.doc.current.pixels.copy()
        self._draw(prev, self._start[0], self._start[1], x, y, self._color)
        app.canvas.set_preview(prev)

    def _draw(self, img, x0, y0, x1, y1, color):
        raise NotImplementedError


class LineTool(_TwoPointTool):
    name = "line"

    def _draw(self, img, x0, y0, x1, y1, color):
        px = img.load(); w, h = img.size
        offsets = _brush_offsets(getattr(self, "_brush", 1))
        for x, y in _bresenham(x0, y0, x1, y1):
            for dx, dy in offsets:
                xx, yy = x + dx, y + dy
                if 0 <= xx < w and 0 <= yy < h:
                    px[xx, yy] = color


class RectTool(_TwoPointTool):
    def __init__(self, filled: bool):
        self.filled = filled
        self.name = "rect_fill" if filled else "rect"

    def _draw(self, img, x0, y0, x1, y1, color):
        d = ImageDraw.Draw(img)
        lo_x, hi_x = sorted((x0, x1))
        lo_y, hi_y = sorted((y0, y1))
        if self.filled:
            d.rectangle([lo_x, lo_y, hi_x, hi_y], fill=color)
        else:
            d.rectangle([lo_x, lo_y, hi_x, hi_y], outline=color,
                        width=getattr(self, "_brush", 1))


class EllipseTool(_TwoPointTool):
    def __init__(self, filled: bool):
        self.filled = filled
        self.name = "ellipse_fill" if filled else "ellipse"

    def _draw(self, img, x0, y0, x1, y1, color):
        d = ImageDraw.Draw(img)
        lo_x, hi_x = sorted((x0, x1))
        lo_y, hi_y = sorted((y0, y1))
        if self.filled:
            d.ellipse([lo_x, lo_y, hi_x, hi_y], fill=color)
        else:
            d.ellipse([lo_x, lo_y, hi_x, hi_y], outline=color,
                      width=getattr(self, "_brush", 1))


class EyedropperTool(Tool):
    name = "eyedrop"
    needs_snapshot = False

    def begin(self, app, x, y, button):
        if not app.in_bounds(x, y):
            return
        col = app.doc.current.pixels.getpixel((x, y))
        if button == Qt.LeftButton:
            app.set_fg(col)
        else:
            app.set_bg(col)
        if app._previous_tool_name and app._previous_tool_name != "eyedrop":
            app.activate_tool(app._previous_tool_name, _silent=True)


class MoveTool(Tool):
    name = "move"

    def begin(self, app, x, y, button):
        app.doc.current.snapshot()
        self._start = (x, y)
        self._original = app.doc.current.pixels.copy()

    def move(self, app, x, y):
        if self._start is None:
            return
        dx = x - self._start[0]; dy = y - self._start[1]
        new_img = Image.new("RGBA", self._original.size, (0, 0, 0, 0))
        new_img.paste(self._original, (dx, dy))
        app.doc.current.pixels = new_img
        app.canvas.invalidate_all()

    def end(self, app, x, y):
        self._start = None
        self._original = None
        app.mark_dirty()


class MagicWandTool(Tool):
    """Auswahl per Klick auf eine zusammenhaengende, gleichfarbige Region.

    Floodet wie der Eimer, sammelt alle Pixel-Koordinaten und setzt das
    Bounding-Rect als Auswahl. Vereinfachung gegenueber einer echten
    Pixel-Maske (was klassische Editoren machen) -- das Bounding-Rect
    deckt den haeufigsten Use-Case (gleichfarbige Region selektieren um
    sie z.B. via Color-Replace zu tauschen) ab und nutzt den existing
    Auswahl-Code.
    """
    name = "magic_wand"
    needs_snapshot = False

    def begin(self, app, x, y, button):
        if not app.in_bounds(x, y):
            return
        target = app.doc.current.pixels.getpixel((x, y))
        pixels = self._collect(app.doc.current.pixels, x, y, target)
        if not pixels:
            return
        xs = [p[0] for p in pixels]
        ys = [p[1] for p in pixels]
        x0, x1 = min(xs), max(xs)
        y0, y1 = min(ys), max(ys)
        app.canvas.set_selection(x0, y0, x1, y1)
        # Hint
        n = len(pixels)
        w = x1 - x0 + 1; h = y1 - y0 + 1
        rgba = target
        app.statusBar().showMessage(
            f"Magic Wand: {n} Pixel der Farbe "
            f"({rgba[0]},{rgba[1]},{rgba[2]},{rgba[3]}) selektiert "
            f"-- Bounding-Rect {w}x{h}",
            5000,
        )

    def move(self, app, x, y):
        pass

    def end(self, app, x, y):
        pass

    @staticmethod
    def _collect(img: Image.Image, sx: int, sy: int,
                 target: tuple) -> list[tuple[int, int]]:
        """4-connectivity Flood, sammelt alle gleichfarbigen Pixel."""
        w, h = img.size
        px = img.load()
        if px[sx, sy] != target:
            return []
        seen = set()
        stack = [(sx, sy)]
        out = []
        while stack:
            x, y = stack.pop()
            if (x, y) in seen:
                continue
            if x < 0 or x >= w or y < 0 or y >= h:
                continue
            if px[x, y] != target:
                continue
            seen.add((x, y))
            out.append((x, y))
            stack.append((x + 1, y))
            stack.append((x - 1, y))
            stack.append((x, y + 1))
            stack.append((x, y - 1))
        return out


def polygon_mask(size: tuple[int, int], points: list[tuple[int, int]]) -> Image.Image:
    """Baut eine 'L'-Maske (255 = selektiert) aus einem Freiform-Polygon.

    Punkte ausserhalb des Bildes sind erlaubt (werden geclippt). Bei
    weniger als 3 Punkten ist die Maske leer.
    """
    mask = Image.new("L", size, 0)
    if len(points) >= 3:
        d = ImageDraw.Draw(mask)
        d.polygon(points, fill=255, outline=255)
    return mask


class LassoTool(Tool):
    """Freiform-Auswahl (Lasso) mit echter Pixel-Maske.

    Ziehen zeichnet einen Freiform-Pfad (Live-Vorschau); beim Loslassen
    wird er zum Polygon geschlossen und als Maske selektiert -- Auswahl-
    Operationen (Cut/Copy/Fuellen/Spiegeln/Loeschen) wirken dann nur auf
    die maskierten Pixel. Klick IN eine bestehende Lasso-Auswahl greift
    nur die maskierten Pixel und verschiebt sie (Float-Move wie beim
    Auswahl-Tool, ein Undo-Schritt); die Maske wandert mit.
    """
    name = "lasso"
    needs_snapshot = False

    PATH_COLOR = (255, 216, 0, 255)   # wie die Marquee-Farbe

    def __init__(self):
        self._pts: list[tuple[int, int]] | None = None
        # Float-Move-Zustand
        self._moving = False
        self._float_img = None     # maskierte Pixel (RGBA, bbox-gross)
        self._float_mask = None    # Masken-Ausschnitt (L, bbox-gross)
        self._float_pos = (0, 0)
        self._grab_off = (0, 0)
        self._base_img = None      # Frame ohne die maskierten Pixel

    def begin(self, app, x, y, button):
        mask = app.canvas.get_selection_mask()
        if (mask is not None and 0 <= x < mask.width and 0 <= y < mask.height
                and mask.getpixel((x, y))):
            self._begin_move(app, mask, x, y)
            return
        self._pts = [(x, y)]
        self._preview_path(app)

    def move(self, app, x, y):
        if self._moving:
            self._float_pos = (x - self._grab_off[0], y - self._grab_off[1])
            self._show_float(app)
            return
        if self._pts is None:
            return
        if self._pts[-1] != (x, y):
            self._pts.append((x, y))
            self._preview_path(app)

    def end(self, app, x, y):
        if self._moving:
            self._end_move(app)
            return
        if self._pts is None:
            return
        pts, self._pts = self._pts, None
        app.canvas.set_preview(None)
        mask = polygon_mask((app.doc.width, app.doc.height), pts)
        if mask.getbbox() is None:
            app.statusBar().showMessage(
                "Lasso: Bereich umfahren (mind. 3 Punkte), "
                "Loslassen schliesst das Polygon", 3000)
            return
        app.canvas.set_selection_mask(mask)
        from PySide6.QtGui import QCursor
        app.show_selection_context_menu(QCursor.pos())

    # --- Pfad-Vorschau waehrend des Aufziehens --------------------------

    def _preview_path(self, app):
        prev = app.doc.current.pixels.copy()
        px = prev.load()
        w, h = prev.size
        pts = self._pts
        for i in range(len(pts) - 1):
            for lx, ly in _bresenham(*pts[i], *pts[i + 1]):
                if 0 <= lx < w and 0 <= ly < h:
                    px[lx, ly] = self.PATH_COLOR
        if pts:
            lx, ly = pts[0]
            if 0 <= lx < w and 0 <= ly < h:
                px[lx, ly] = self.PATH_COLOR
        app.canvas.set_preview(prev)

    # --- Float-Move der maskierten Pixel --------------------------------

    def _begin_move(self, app, mask, x, y):
        x0, y0, x1, y1 = mask.getbbox()
        f = app.doc.current
        f.snapshot()
        self._float_mask = mask.crop((x0, y0, x1, y1))
        region = f.pixels.crop((x0, y0, x1, y1))
        float_img = Image.new("RGBA", region.size, (0, 0, 0, 0))
        float_img.paste(region, (0, 0), self._float_mask)
        self._float_img = float_img
        base = f.pixels.copy()
        base.paste((0, 0, 0, 0), (0, 0), mask)
        self._base_img = base
        self._float_pos = (x0, y0)
        self._grab_off = (x - x0, y - y0)
        self._moving = True
        self._show_float(app)

    def _show_float(self, app):
        prev = self._base_img.copy()
        prev.paste(self._float_img, self._float_pos, self._float_img)
        app.canvas.set_preview(prev)

    def _moved_mask(self, app) -> Image.Image:
        moved = Image.new("L", (app.doc.width, app.doc.height), 0)
        moved.paste(self._float_mask, self._float_pos)
        return moved

    def _end_move(self, app):
        result = self._base_img
        result.paste(self._float_img, self._float_pos, self._float_img)
        app.doc.current.pixels = result
        moved = self._moved_mask(app)
        app.canvas.set_preview(None)
        self._moving = False
        self._float_img = None
        self._float_mask = None
        self._base_img = None
        if moved.getbbox() is not None:
            app.canvas.set_selection_mask(moved)
        else:
            app.canvas.clear_selection()
        app.canvas.invalidate_all()
        app.mark_dirty()


class SelectTool(Tool):
    """Rechteck-Auswahl -- und Verschieben des Auswahl-INHALTS:
    Klick IN eine bestehende Auswahl greift die Pixel (Float), Ziehen
    verschiebt sie (Live-Preview), Loslassen setzt sie ab. Klick
    ausserhalb zieht wie gehabt eine neue Auswahl auf. Das Greifen
    macht einen Pixel-Snapshot -> die ganze Verschiebung ist EIN Undo."""
    name = "select"
    needs_snapshot = False

    def __init__(self):
        self._start = None
        # Float-Move-Zustand (Klick in bestehende Auswahl)
        self._moving = False
        self._float_img = None    # ausgeschnittene Pixel (RGBA)
        self._float_pos = (0, 0)  # aktuelle Top-Left-Position des Floats
        self._grab_off = (0, 0)   # Cursor-Offset im Float
        self._base_img = None     # Frame ohne den Ausschnitt

    @staticmethod
    def _clamp(app, x, y):
        cx = max(0, min(app.doc.width - 1, x))
        cy = max(0, min(app.doc.height - 1, y))
        return cx, cy

    def begin(self, app, x, y, button):
        sel = app.canvas.get_selection()
        if sel and sel[0] <= x < sel[2] and sel[1] <= y < sel[3]:
            self._begin_move(app, sel, x, y)
            return
        x, y = self._clamp(app, x, y)
        self._start = (x, y)
        app.canvas.set_selection(x, y, x, y)

    def _begin_move(self, app, sel, x, y):
        x0, y0, x1, y1 = sel
        f = app.doc.current
        f.snapshot()
        self._float_img = f.pixels.crop((x0, y0, x1, y1)).copy()
        base = f.pixels.copy()
        clear = Image.new("RGBA", (x1 - x0, y1 - y0), (0, 0, 0, 0))
        base.paste(clear, (x0, y0))
        self._base_img = base
        self._float_pos = (x0, y0)
        self._grab_off = (x - x0, y - y0)
        self._moving = True
        self._show_float(app)

    def _show_float(self, app):
        prev = self._base_img.copy()
        # paste mit Alpha-Maske: clippt automatisch an den Raendern
        # (auch bei negativen Koordinaten).
        prev.paste(self._float_img, self._float_pos, self._float_img)
        app.canvas.set_preview(prev)
        fx, fy = self._float_pos
        fw, fh = self._float_img.width, self._float_img.height
        # Auswahl-Rechteck mitfuehren (auf Canvas geclampt -- sichtbarer Teil).
        cx0, cy0 = self._clamp(app, fx, fy)
        cx1, cy1 = self._clamp(app, fx + fw - 1, fy + fh - 1)
        app.canvas.set_selection(cx0, cy0, cx1, cy1)

    def move(self, app, x, y):
        if self._moving:
            self._float_pos = (x - self._grab_off[0], y - self._grab_off[1])
            self._show_float(app)
            return
        if self._start is None:
            return
        x, y = self._clamp(app, x, y)
        app.canvas.set_selection(self._start[0], self._start[1], x, y)

    def end(self, app, x, y):
        if self._moving:
            result = self._base_img
            result.paste(self._float_img, self._float_pos, self._float_img)
            app.doc.current.pixels = result
            app.canvas.set_preview(None)
            self._moving = False
            self._float_img = None
            self._base_img = None
            app.canvas.invalidate_all()
            app.mark_dirty()
            return
        if self._start is None:
            return
        x, y = self._clamp(app, x, y)
        app.canvas.set_selection(self._start[0], self._start[1], x, y)
        self._start = None
        sel = app.canvas.get_selection()
        if not sel:
            return
        w = sel[2] - sel[0]; h = sel[3] - sel[1]
        if w >= 2 or h >= 2:
            # Echte Range-Auswahl: Kontextmenue an Cursor-Position oeffnen,
            # damit der User direkt sieht was er machen kann.
            from PySide6.QtGui import QCursor
            app.show_selection_context_menu(QCursor.pos())
        else:
            # Single-Pixel-Klick: nur ein dezenter Hinweis, kein Popup
            app.statusBar().showMessage(
                "Auswahl 1x1 -- ziehen fuer einen Bereich (in der Auswahl "
                "ziehen = Inhalt verschieben), Esc zum Aufheben",
                3000,
            )


