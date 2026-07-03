"""Tilemap-/Level-Editor fuer GameBasic (`gbtilemap` / `gbrun.py --tilemap`).

Atlas-Tiles auf ein Gitter malen, mehrere Layer, Per-Tile-Properties
(`solid`, `damage`, ...), Speichern/Laden als Tiled-JSON (das `TILED_LOAD`
direkt einliest) plus GB-Code-Export eines fertigen Renderers. Schliesst den
Kreis mit dem Sprite-Atlas-Export des Sprite-Editors.

Aufbau:
  - links:  Tileset-Palette (Tile waehlen)
  - mitte:  Karten-Canvas (malen mit Pencil/Eraser/Fill/Rect/Pipette)
  - rechts: Layer-Liste (anlegen/loeschen/sortieren/Sichtbarkeit)

Das Datenmodell + die Tiled-JSON-Serialisierung liegen Qt-frei in
`gamebasic.tilemap.document` (headless getestet).
"""
from __future__ import annotations

import copy
import os
from pathlib import Path

from PySide6.QtCore import QPoint, QRect, QSize, Qt, Signal
from PySide6.QtGui import (
    QAction, QActionGroup, QColor, QFont, QKeySequence, QPainter, QPen, QPixmap,
)
from PySide6.QtWidgets import (
    QAbstractItemView, QApplication, QComboBox, QDialog, QDialogButtonBox,
    QDockWidget, QFileDialog, QFormLayout, QHBoxLayout, QInputDialog, QLabel,
    QLineEdit, QListWidget, QListWidgetItem, QMainWindow, QMessageBox,
    QPlainTextEdit, QPushButton, QScrollArea, QSpinBox, QTableWidget,
    QTableWidgetItem, QToolBar, QToolButton, QVBoxLayout, QWidget,
)

from .editor_qt.icons import icons
from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .tilemap.document import (
    PROP_TYPES, MapObject, ObjectLayer, TileLayer, TileMapDoc, coerce_prop,
)

# Werkzeuge
TOOL_PENCIL, TOOL_ERASER, TOOL_FILL, TOOL_RECT, TOOL_PICK, TOOL_SELECT = range(6)


# ===================================================================
#  Tileset-Palette
# ===================================================================
class _Palette(QWidget):
    """Zeigt das Tileset, Gitter + Auswahl-Highlight. Klick waehlt ein Tile."""

    tile_selected = Signal(int)            # lokale Tile-ID

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc: TileMapDoc | None = None
        self.pixmap: QPixmap | None = None
        self.zoom = 2
        self.sel = 0
        self.setMouseTracking(True)

    def set_doc(self, doc: TileMapDoc, pixmap: QPixmap | None) -> None:
        self.doc = doc
        self.pixmap = pixmap
        self._update_size()
        self.update()

    def set_zoom(self, z: int) -> None:
        self.zoom = max(1, min(8, z))
        self._update_size()
        self.update()

    def _update_size(self) -> None:
        if self.pixmap and not self.pixmap.isNull():
            self.setMinimumSize(self.pixmap.width() * self.zoom,
                                self.pixmap.height() * self.zoom)
        else:
            self.setMinimumSize(200, 120)

    def _tile_at(self, pos: QPoint) -> int:
        if not self.doc or self.doc.columns <= 0:
            return -1
        tw = self.doc.tile_w * self.zoom
        th = self.doc.tile_h * self.zoom
        cx = pos.x() // tw
        cy = pos.y() // th
        if cx < 0 or cy < 0 or cx >= self.doc.columns:
            return -1
        lid = cy * self.doc.columns + cx
        if lid >= self.doc.tile_count:
            return -1
        return lid

    def mousePressEvent(self, e):  # noqa: N802
        lid = self._tile_at(e.position().toPoint())
        if lid >= 0:
            self.sel = lid
            self.tile_selected.emit(lid)
            self.update()

    def paintEvent(self, _e):  # noqa: N802
        p = QPainter(self)
        p.fillRect(self.rect(), QColor(COLORS["bg"]))
        if not self.pixmap or self.pixmap.isNull() or not self.doc:
            p.setPen(QColor(COLORS["fg"]))
            p.drawText(self.rect(), Qt.AlignmentFlag.AlignCenter,
                       "Kein Tileset geladen\n(Werkzeugleiste: Tileset)")
            return
        z = self.zoom
        scaled = QRect(0, 0, self.pixmap.width() * z, self.pixmap.height() * z)
        p.drawPixmap(scaled, self.pixmap)
        # Gitter
        tw = self.doc.tile_w * z
        th = self.doc.tile_h * z
        p.setPen(QPen(QColor(0, 0, 0, 90)))
        for cx in range(self.doc.columns + 1):
            p.drawLine(cx * tw, 0, cx * tw, scaled.height())
        rows = self.doc.tile_count // max(1, self.doc.columns)
        for cy in range(rows + 1):
            p.drawLine(0, cy * th, scaled.width(), cy * th)
        # Auswahl
        if self.doc.columns > 0:
            sx = (self.sel % self.doc.columns) * tw
            sy = (self.sel // self.doc.columns) * th
            p.setPen(QPen(QColor(COLORS["accent"]), 2))
            p.drawRect(sx + 1, sy + 1, tw - 2, th - 2)


# ===================================================================
#  Karten-Canvas
# ===================================================================
class _Canvas(QWidget):
    """Rendert alle sichtbaren Layer + Gitter, malt mit dem aktiven Werkzeug."""

    # (layer_idx, before_tiles, after_tiles) -- fuer Undo
    committed = Signal(int, object, object)
    # (layer_idx, before_objects, after_objects) -- fuer Object-Layer-Undo
    obj_committed = Signal(int, object, object)
    obj_edit_requested = Signal(object)    # MapObject zum Bearbeiten
    obj_selected = Signal(object)          # MapObject oder None
    cell_hovered = Signal(int, int)
    picked = Signal(int)                   # Pipette -> lokale Tile-ID

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc: TileMapDoc | None = None
        self.pixmaps: list = []            # QPixmap pro Tileset (parallel)
        self.zoom = 2
        self.tool = TOOL_PENCIL
        self.active_layer = 0
        self.current_gid = 1               # zu malendes gid (lokale-ID + 1)
        self.show_grid = True
        self.dim_others = True
        self._painting = False
        self._erase = False
        self._before: list | None = None
        self._rect_start: tuple | None = None
        self._rect_cur: tuple | None = None
        # Select-Tool: rechteckige Tile-Auswahl + Clipboard (2D-GID-Block)
        self._sel_start: tuple | None = None
        self._sel_cur: tuple | None = None
        self._sel_rect: tuple | None = None      # (x0,y0,x1,y1) inklusiv
        self.tile_clipboard: list[list[int]] | None = None
        self._hover: tuple = (0, 0)
        # Object-Layer-Interaktion
        self.selected_obj: MapObject | None = None
        self._obj_before: list | None = None   # Snapshot fuer Undo
        self._obj_moving = False
        self._obj_move_off = (0.0, 0.0)
        self._obj_drag_start: tuple | None = None   # (px,py) doc-Pixel
        self._obj_drag_cur: tuple | None = None
        self.setMouseTracking(True)

    def set_doc(self, doc: TileMapDoc, pixmaps=None) -> None:
        self.doc = doc
        # pixmaps: Liste (parallel zu doc.tilesets). Rueckwaerts-kompatibel:
        # ein einzelnes QPixmap wird als Ein-Element-Liste behandelt.
        if pixmaps is None:
            self.pixmaps = []
        elif isinstance(pixmaps, list):
            self.pixmaps = pixmaps
        else:
            self.pixmaps = [pixmaps]
        self.active_layer = 0
        self._update_size()
        self.update()

    def set_zoom(self, z: int) -> None:
        self.zoom = max(1, min(8, z))
        self._update_size()
        self.update()

    def _update_size(self) -> None:
        if self.doc:
            self.setFixedSize(self.doc.width * self.doc.tile_w * self.zoom,
                              self.doc.height * self.doc.tile_h * self.zoom)

    # ---------------------------------------------------- Maus
    def _cell(self, pos: QPoint) -> tuple[int, int]:
        if not self.doc:
            return (-1, -1)
        cw = self.doc.tile_w * self.zoom
        ch = self.doc.tile_h * self.zoom
        return (pos.x() // cw, pos.y() // ch)

    # ---------------------------------------------------- Object-Layer
    def _active_obj_layer(self):
        """Der aktive Layer, falls er ein Object-Layer ist -- sonst None."""
        if self.doc and 0 <= self.active_layer < len(self.doc.layers):
            l = self.doc.layers[self.active_layer]
            if isinstance(l, ObjectLayer):
                return l
        return None

    def _doc_px(self, pos: QPoint) -> tuple[float, float]:
        """Bildschirm-Pixel -> Doc-Pixel (Tiled-Koordinaten)."""
        return (pos.x() / self.zoom, pos.y() / self.zoom)

    def _snap_point(self, px: float, py: float) -> tuple[float, float]:
        """Punkt auf die Mitte der getroffenen Zelle snappen."""
        tw, th = self.doc.tile_w, self.doc.tile_h
        cx = int(px // tw); cy = int(py // th)
        return (cx * tw + tw / 2.0, cy * th + th / 2.0)

    def _select_obj(self, obj) -> None:
        self.selected_obj = obj
        self.obj_selected.emit(obj)
        self.update()

    def _obj_press(self, pos: QPoint, right: bool) -> None:
        layer = self._active_obj_layer()
        px, py = self._doc_px(pos)
        hit = layer.object_at(px, py)
        if right:                                  # Rechtsklick = loeschen
            if hit is not None:
                self._obj_before = copy.deepcopy(layer.objects)
                layer.remove(hit)
                self._select_obj(None)
                self._obj_commit(layer)
            return
        if hit is not None:                        # bestehendes Objekt: waehlen + ziehen
            self._select_obj(hit)
            self._obj_before = copy.deepcopy(layer.objects)
            self._obj_moving = True
            self._obj_move_off = (px - hit.x, py - hit.y)
            return
        # leere Stelle: neues Objekt aufziehen
        self._obj_drag_start = (px, py)
        self._obj_drag_cur = (px, py)
        self.update()

    def _obj_move(self, pos: QPoint) -> None:
        layer = self._active_obj_layer()
        px, py = self._doc_px(pos)
        if self._obj_moving and self.selected_obj is not None:
            o = self.selected_obj
            nx, ny = px - self._obj_move_off[0], py - self._obj_move_off[1]
            if o.is_point():
                o.x, o.y = self._snap_point(nx, ny)
            else:
                tw, th = self.doc.tile_w, self.doc.tile_h
                o.x = round(nx / tw) * tw
                o.y = round(ny / th) * th
            self.update()
        elif self._obj_drag_start is not None:
            self._obj_drag_cur = (px, py)
            self.update()

    def _obj_release(self) -> None:
        layer = self._active_obj_layer()
        if self._obj_moving:
            self._obj_moving = False
            self._obj_commit(layer)
            return
        if self._obj_drag_start is None:
            return
        x0, y0 = self._obj_drag_start
        x1, y1 = self._obj_drag_cur or self._obj_drag_start
        self._obj_drag_start = self._obj_drag_cur = None
        tw, th = self.doc.tile_w, self.doc.tile_h
        self._obj_before = copy.deepcopy(layer.objects)
        if abs(x1 - x0) < tw / 2 and abs(y1 - y0) < th / 2:
            # Punkt-Objekt (Spawn-Marker)
            sx, sy = self._snap_point(x0, y0)
            obj = MapObject(sx, sy, name="", otype="")
        else:
            # Rechteck-Objekt (Zone/Trigger) -- auf Zellen gerundet
            cx0, cx1 = sorted((int(x0 // tw), int(x1 // tw)))
            cy0, cy1 = sorted((int(y0 // th), int(y1 // th)))
            obj = MapObject(cx0 * tw, cy0 * th,
                            (cx1 - cx0 + 1) * tw, (cy1 - cy0 + 1) * th)
        layer.add(obj)
        self._select_obj(obj)
        self._obj_commit(layer)
        self.obj_edit_requested.emit(obj)          # gleich benennen lassen

    def _obj_commit(self, layer) -> None:
        if self._obj_before is not None:
            after = copy.deepcopy(layer.objects)
            self.obj_committed.emit(self.active_layer, self._obj_before, after)
            self.doc.dirty = True
        self._obj_before = None

    def delete_selected(self) -> None:
        """Loescht das selektierte Objekt (Entf-Taste vom Editor) -- oder
        leert die Tile-Auswahl, wenn das Select-Tool aktiv ist."""
        if self._active_obj_layer() is None:
            self.delete_selection()
            return
        layer = self._active_obj_layer()
        if layer is None or self.selected_obj is None:
            return
        if self.selected_obj in layer.objects:
            self._obj_before = copy.deepcopy(layer.objects)
            layer.remove(self.selected_obj)
            self._select_obj(None)
            self._obj_commit(layer)

    def mouseDoubleClickEvent(self, e):  # noqa: N802
        if self._active_obj_layer() is None:
            return
        layer = self._active_obj_layer()
        px, py = self._doc_px(e.position().toPoint())
        hit = layer.object_at(px, py)
        if hit is not None:
            self._select_obj(hit)
            self.obj_edit_requested.emit(hit)

    def mousePressEvent(self, e):  # noqa: N802
        if not self.doc:
            return
        if self._active_obj_layer() is not None:
            self._obj_press(e.position().toPoint(),
                            e.button() == Qt.MouseButton.RightButton)
            return
        cx, cy = self._cell(e.position().toPoint())
        self._erase = (e.button() == Qt.MouseButton.RightButton)
        layer = self.doc.layers[self.active_layer]
        if self.tool == TOOL_PICK and not self._erase:
            g = layer.get(cx, cy)
            if g > 0:
                self.picked.emit(g - 1)
            return
        if self.tool == TOOL_SELECT and not self._erase:
            self._sel_start = (cx, cy)
            self._sel_cur = (cx, cy)
            self._sel_rect = None
            self.update()
            return
        self._before = list(layer.tiles)
        if self.tool == TOOL_FILL and not self._erase:
            self.doc.flood_fill(self.active_layer, cx, cy,
                                self.current_gid)
            self._commit(layer)
            self.update()
            return
        if self.tool == TOOL_RECT and not self._erase:
            self._rect_start = (cx, cy)
            self._rect_cur = (cx, cy)
            self.update()
            return
        # Pencil / Eraser / Rechtsklick-Loeschen
        self._painting = True
        self._paint_cell(cx, cy)
        self.update()

    def mouseMoveEvent(self, e):  # noqa: N802
        if not self.doc:
            return
        if self._active_obj_layer() is not None:
            self._obj_move(e.position().toPoint())
            return
        cx, cy = self._cell(e.position().toPoint())
        self._hover = (cx, cy)
        self.cell_hovered.emit(cx, cy)
        if self._sel_start is not None:
            self._sel_cur = (cx, cy)
            self.update()
        elif self._rect_start is not None:
            self._rect_cur = (cx, cy)
            self.update()
        elif self._painting:
            self._paint_cell(cx, cy)
            self.update()

    def mouseReleaseEvent(self, e):  # noqa: N802
        if not self.doc:
            return
        if self._active_obj_layer() is not None:
            self._obj_release()
            return
        layer = self.doc.layers[self.active_layer]
        if self._sel_start is not None:
            x0, y0 = self._sel_start
            x1, y1 = self._sel_cur or self._sel_start
            self._sel_rect = (min(x0, x1), min(y0, y1),
                              max(x0, x1), max(y0, y1))
            self._sel_start = self._sel_cur = None
            self.update()
            return
        if self._rect_start is not None:
            x0, y0 = self._rect_start
            x1, y1 = self._rect_cur or self._rect_start
            gid = 0 if self._erase else self.current_gid
            for yy in range(min(y0, y1), max(y0, y1) + 1):
                for xx in range(min(x0, x1), max(x0, x1) + 1):
                    layer.set(xx, yy, gid)
            self._rect_start = self._rect_cur = None
            self._commit(layer)
            self.update()
        elif self._painting:
            self._painting = False
            self._commit(layer)

    def _paint_cell(self, cx: int, cy: int) -> None:
        layer = self.doc.layers[self.active_layer]
        gid = 0 if (self._erase or self.tool == TOOL_ERASER) else self.current_gid
        layer.set(cx, cy, gid)

    def _commit(self, layer) -> None:
        if self._before is not None and self._before != layer.tiles:
            self.committed.emit(self.active_layer, self._before, list(layer.tiles))
            self.doc.dirty = True
        self._before = None

    # ---------------------------------------------------- Select-Clipboard
    def _tile_layer(self):
        """Aktiver Layer, falls Tile-Layer -- sonst None."""
        if self.doc and 0 <= self.active_layer < len(self.doc.layers):
            l = self.doc.layers[self.active_layer]
            if isinstance(l, TileLayer):
                return l
        return None

    def has_selection(self) -> bool:
        return self._sel_rect is not None and self._tile_layer() is not None

    def clear_selection(self) -> None:
        if self._sel_rect is not None or self._sel_start is not None:
            self._sel_rect = self._sel_start = self._sel_cur = None
            self.update()

    def copy_selection(self) -> bool:
        """Kopiert die Auswahl als 2D-GID-Block ins Clipboard."""
        if not self.has_selection():
            return False
        x0, y0, x1, y1 = self._sel_rect
        self.tile_clipboard = self.doc.get_region(
            self.active_layer, x0, y0, x1 - x0 + 1, y1 - y0 + 1)
        return True

    def cut_selection(self) -> bool:
        """Kopiert + leert die Auswahl (eine Undo-Operation)."""
        if not self.copy_selection():
            return False
        layer = self._tile_layer()
        x0, y0, x1, y1 = self._sel_rect
        self._before = list(layer.tiles)
        self.doc.clear_region(self.active_layer, x0, y0,
                              x1 - x0 + 1, y1 - y0 + 1)
        self._commit(layer)
        self.update()
        return True

    def delete_selection(self) -> bool:
        """Leert die Auswahl (ohne ins Clipboard zu kopieren)."""
        layer = self._tile_layer()
        if not self.has_selection() or layer is None:
            return False
        x0, y0, x1, y1 = self._sel_rect
        self._before = list(layer.tiles)
        changed = self.doc.clear_region(self.active_layer, x0, y0,
                                        x1 - x0 + 1, y1 - y0 + 1)
        self._commit(layer)
        self.update()
        return changed

    def paste_clipboard(self) -> bool:
        """Stempelt das Clipboard mit der oberen-linken Ecke an der aktuellen
        Hover-Zelle (bzw. Auswahl-Ursprung)."""
        layer = self._tile_layer()
        if not self.tile_clipboard or layer is None:
            return False
        if self._sel_rect is not None:
            tx, ty = self._sel_rect[0], self._sel_rect[1]
        else:
            tx, ty = self._hover
        self._before = list(layer.tiles)
        self.doc.stamp_region(self.active_layer, tx, ty, self.tile_clipboard)
        self._commit(layer)
        self.update()
        return True

    # ---------------------------------------------------- Zeichnen
    def paintEvent(self, _e):  # noqa: N802
        p = QPainter(self)
        if not self.doc:
            return
        cw = self.doc.tile_w * self.zoom
        ch = self.doc.tile_h * self.zoom
        # Schachbrett-Hintergrund (zeigt Transparenz)
        c1, c2 = QColor(40, 40, 48), QColor(48, 48, 58)
        for ty in range(self.doc.height):
            for tx in range(self.doc.width):
                col = c1 if (tx + ty) % 2 == 0 else c2
                p.fillRect(tx * cw, ty * ch, cw, ch, col)
        # Layer
        have_ts = (any(px is not None and not px.isNull() for px in self.pixmaps)
                   and len(self.doc.tilesets) > 0)
        for li, layer in enumerate(self.doc.layers):
            if not layer.visible:
                continue
            if self.dim_others and li != self.active_layer:
                p.setOpacity(0.40)
            else:
                p.setOpacity(1.0)
            if isinstance(layer, ObjectLayer):
                self._draw_objects(p, layer)
            elif have_ts:
                self._draw_layer_tiles(p, layer, cw, ch)
            else:
                self._draw_layer_ids(p, layer, cw, ch)
        p.setOpacity(1.0)
        # Rechteck-Vorschau
        if self._rect_start is not None and self._rect_cur is not None:
            x0, y0 = self._rect_start
            x1, y1 = self._rect_cur
            r = QRect(min(x0, x1) * cw, min(y0, y1) * ch,
                      (abs(x1 - x0) + 1) * cw, (abs(y1 - y0) + 1) * ch)
            p.setPen(QPen(QColor(COLORS["accent"]), 2))
            p.drawRect(r)
        # Auswahl (Select-Tool): gestrichelter Rahmen, live beim Ziehen
        sel = self._sel_rect
        if self._sel_start is not None and self._sel_cur is not None:
            sx0, sy0 = self._sel_start
            sx1, sy1 = self._sel_cur
            sel = (min(sx0, sx1), min(sy0, sy1), max(sx0, sx1), max(sy0, sy1))
        if sel is not None:
            x0, y0, x1, y1 = sel
            r = QRect(x0 * cw, y0 * ch, (x1 - x0 + 1) * cw, (y1 - y0 + 1) * ch)
            pen = QPen(QColor(COLORS["accent"]), 2, Qt.PenStyle.DashLine)
            p.setPen(pen)
            p.drawRect(r)
        # Objekt-Aufzieh-Vorschau
        if self._obj_drag_start is not None and self._obj_drag_cur is not None:
            x0, y0 = self._obj_drag_start
            x1, y1 = self._obj_drag_cur
            r = QRect(int(min(x0, x1) * self.zoom), int(min(y0, y1) * self.zoom),
                      int(abs(x1 - x0) * self.zoom), int(abs(y1 - y0) * self.zoom))
            p.setPen(QPen(QColor(COLORS["accent"]), 1, Qt.PenStyle.DashLine))
            p.drawRect(r)
        # Gitter
        if self.show_grid:
            p.setPen(QPen(QColor(0, 0, 0, 70)))
            for tx in range(self.doc.width + 1):
                p.drawLine(tx * cw, 0, tx * cw, self.doc.height * ch)
            for ty in range(self.doc.height + 1):
                p.drawLine(0, ty * ch, self.doc.width * cw, ty * ch)

    def _draw_layer_tiles(self, p, layer, cw, ch) -> None:
        doc = self.doc
        for ty in range(layer.height):
            for tx in range(layer.width):
                g = layer.tiles[ty * layer.width + tx]
                if g <= 0:
                    continue
                tsi, lid = doc.gid_to_tileset(g)
                if tsi < 0 or tsi >= len(self.pixmaps):
                    continue
                pix = self.pixmaps[tsi]
                if pix is None or pix.isNull():
                    continue
                sx, sy, sw, sh = doc.tilesets[tsi].tile_src_rect(lid)
                p.drawPixmap(QRect(tx * cw, ty * ch, cw, ch),
                             pix, QRect(sx, sy, sw, sh))

    def _draw_layer_ids(self, p, layer, cw, ch) -> None:
        """Fallback ohne Tileset-Bild: gid als Zahl + Farbflaeche."""
        p.setPen(QColor(COLORS["fg"]))
        f = QFont(EDITOR_FONT_FAMILY, max(6, ch // 3))
        p.setFont(f)
        for ty in range(layer.height):
            for tx in range(layer.width):
                g = layer.tiles[ty * layer.width + tx]
                if g <= 0:
                    continue
                hue = (g * 47) % 360
                p.fillRect(tx * cw + 1, ty * ch + 1, cw - 2, ch - 2,
                           QColor.fromHsv(hue, 120, 150))
                p.drawText(QRect(tx * cw, ty * ch, cw, ch),
                           Qt.AlignmentFlag.AlignCenter, str(g))

    def _draw_objects(self, p, layer) -> None:
        """Zeichnet die Objekte eines Object-Layers: Rechtecke als Rahmen,
        Punkte als Marker, je mit Name/Typ-Label; Selektion hervorgehoben."""
        z = self.zoom
        accent = QColor(COLORS["accent"])
        fill = QColor(accent.red(), accent.green(), accent.blue(), 50)
        f = QFont(EDITOR_FONT_FAMILY, max(7, int(self.doc.tile_h * z / 3)))
        p.setFont(f)
        for obj in layer.objects:
            sel = obj is self.selected_obj
            pen = QPen(QColor("#ffd84d") if sel else accent, 2 if sel else 1)
            if obj.is_point():
                cx, cy = int(obj.x * z), int(obj.y * z)
                rad = max(4, int(self.doc.tile_w * z / 5))
                p.setPen(pen)
                p.setBrush(fill)
                p.drawEllipse(QPoint(cx, cy), rad, rad)
                p.drawLine(cx - rad, cy, cx + rad, cy)
                p.drawLine(cx, cy - rad, cx, cy + rad)
                label = obj.name or obj.type
                if label:
                    p.setPen(QColor("#ffffff"))
                    p.drawText(cx + rad + 2, cy + 4, label)
            else:
                r = QRect(int(obj.x * z), int(obj.y * z),
                          int(obj.width * z), int(obj.height * z))
                p.setPen(pen)
                p.fillRect(r, fill)
                p.drawRect(r)
                label = obj.name or obj.type
                if label:
                    p.setPen(QColor("#ffffff"))
                    p.drawText(r.adjusted(3, 1, -2, -2),
                               Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft,
                               label)
        p.setBrush(Qt.BrushStyle.NoBrush)


# ===================================================================
#  Tile-Eigenschaften-Dialog
# ===================================================================
class _PropDialog(QDialog):
    def __init__(self, doc: TileMapDoc, local_id: int, parent=None):
        super().__init__(parent)
        self.doc = doc
        self.local_id = local_id
        self.setWindowTitle(f"Eigenschaften -- Tile #{local_id} (gid {local_id + 1})")
        self.resize(420, 320)
        lay = QVBoxLayout(self)
        lay.addWidget(QLabel(
            "Per-Tile-Properties (z.B. solid=bool, damage=int). "
            "Das Kollisionssystem liest sie via TILED_TILE_PROP_*."))
        self.table = QTableWidget(0, 3)
        self.table.setHorizontalHeaderLabels(["Name", "Typ", "Wert"])
        self.table.horizontalHeader().setStretchLastSection(True)
        lay.addWidget(self.table, 1)
        # bestehende laden
        props = doc.properties_of(local_id)
        types = doc.tile_property_types.get(local_id, {})
        for k, v in props.items():
            self._add_row(k, types.get(k, "string"), v)

        row = QHBoxLayout()
        b_add = QPushButton("+ Zeile"); b_add.clicked.connect(lambda: self._add_row())
        b_del = QPushButton("- Zeile"); b_del.clicked.connect(self._del_row)
        row.addWidget(b_add); row.addWidget(b_del); row.addStretch(1)
        lay.addLayout(row)
        bb = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok |
                              QDialogButtonBox.StandardButton.Cancel)
        bb.accepted.connect(self.accept)
        bb.rejected.connect(self.reject)
        lay.addWidget(bb)

    def _add_row(self, name="", ptype="string", value="") -> None:
        r = self.table.rowCount()
        self.table.insertRow(r)
        self.table.setItem(r, 0, QTableWidgetItem(str(name)))
        cb = QComboBox(); cb.addItems(PROP_TYPES)
        cb.setCurrentText(ptype if ptype in PROP_TYPES else "string")
        self.table.setCellWidget(r, 1, cb)
        self.table.setItem(r, 2, QTableWidgetItem("" if value is None else str(value)))

    def _del_row(self) -> None:
        r = self.table.currentRow()
        if r >= 0:
            self.table.removeRow(r)

    def apply_to_doc(self) -> None:
        # alte komplett ersetzen
        self.doc.tile_properties[self.local_id] = {}
        self.doc.tile_property_types[self.local_id] = {}
        for r in range(self.table.rowCount()):
            name_item = self.table.item(r, 0)
            name = name_item.text().strip() if name_item else ""
            if not name:
                continue
            ptype = self.table.cellWidget(r, 1).currentText()
            val_item = self.table.item(r, 2)
            val = val_item.text() if val_item else ""
            self.doc.set_property(self.local_id, name, val, ptype)


# ===================================================================
#  Objekt-Eigenschaften-Dialog (Object-Layer)
# ===================================================================
class _ObjectDialog(QDialog):
    """Bearbeitet Name, Typ (Tiled `class`) und Properties eines Map-Objekts."""

    def __init__(self, obj: MapObject, parent=None):
        super().__init__(parent)
        self.obj = obj
        kind = "Punkt / Spawn" if obj.is_point() else "Rechteck / Zone"
        self.setWindowTitle(f"Objekt bearbeiten ({kind})")
        self.resize(420, 360)
        lay = QVBoxLayout(self)
        form = QFormLayout()
        self.name_edit = QLineEdit(obj.name)
        self.type_edit = QLineEdit(obj.type)
        form.addRow("Name:", self.name_edit)
        form.addRow("Typ (class):", self.type_edit)
        lay.addLayout(form)
        lay.addWidget(QLabel(
            "Properties (z.B. hp=int, active=bool). Das Spiel liest sie via "
            "TILED_OBJECT_PROP_*."))
        self.table = QTableWidget(0, 3)
        self.table.setHorizontalHeaderLabels(["Name", "Typ", "Wert"])
        self.table.horizontalHeader().setStretchLastSection(True)
        lay.addWidget(self.table, 1)
        for k, v in obj.properties.items():
            self._add_row(k, obj.property_types.get(k, "string"), v)

        row = QHBoxLayout()
        b_add = QPushButton("+ Zeile"); b_add.clicked.connect(lambda: self._add_row())
        b_del = QPushButton("- Zeile"); b_del.clicked.connect(self._del_row)
        row.addWidget(b_add); row.addWidget(b_del); row.addStretch(1)
        lay.addLayout(row)
        bb = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok |
                              QDialogButtonBox.StandardButton.Cancel)
        bb.accepted.connect(self.accept)
        bb.rejected.connect(self.reject)
        lay.addWidget(bb)
        self.name_edit.setFocus()

    def _add_row(self, name="", ptype="string", value="") -> None:
        r = self.table.rowCount()
        self.table.insertRow(r)
        self.table.setItem(r, 0, QTableWidgetItem(str(name)))
        cb = QComboBox(); cb.addItems(PROP_TYPES)
        cb.setCurrentText(ptype if ptype in PROP_TYPES else "string")
        self.table.setCellWidget(r, 1, cb)
        self.table.setItem(r, 2, QTableWidgetItem("" if value is None else str(value)))

    def _del_row(self) -> None:
        r = self.table.currentRow()
        if r >= 0:
            self.table.removeRow(r)

    def apply_to_obj(self) -> None:
        self.obj.name = self.name_edit.text().strip()
        self.obj.type = self.type_edit.text().strip()
        self.obj.properties = {}
        self.obj.property_types = {}
        for r in range(self.table.rowCount()):
            name_item = self.table.item(r, 0)
            name = name_item.text().strip() if name_item else ""
            if not name:
                continue
            ptype = self.table.cellWidget(r, 1).currentText()
            val_item = self.table.item(r, 2)
            val = val_item.text() if val_item else ""
            self.obj.set_property(name, val, ptype)


# ===================================================================
#  Neue-Karte-Dialog
# ===================================================================
def _ask_map_params(parent, w=20, h=15, tw=16, th=16):
    dlg = QDialog(parent)
    dlg.setWindowTitle("Neue Karte")
    form = QFormLayout(dlg)
    sw = QSpinBox(); sw.setRange(1, 1000); sw.setValue(w)
    sh = QSpinBox(); sh.setRange(1, 1000); sh.setValue(h)
    stw = QSpinBox(); stw.setRange(1, 256); stw.setValue(tw)
    sth = QSpinBox(); sth.setRange(1, 256); sth.setValue(th)
    form.addRow("Breite (Tiles):", sw)
    form.addRow("Hoehe (Tiles):", sh)
    form.addRow("Tile-Breite (px):", stw)
    form.addRow("Tile-Hoehe (px):", sth)
    bb = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok |
                          QDialogButtonBox.StandardButton.Cancel)
    bb.accepted.connect(dlg.accept); bb.rejected.connect(dlg.reject)
    form.addRow(bb)
    if dlg.exec() != QDialog.DialogCode.Accepted:
        return None
    return (sw.value(), sh.value(), stw.value(), sth.value())


# ===================================================================
#  Hauptfenster
# ===================================================================
class TileMapEditor(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = Path(project_root)
        self.setWindowTitle("GameBasic Tilemap-Editor")
        self.resize(1360, 880)
        self.doc = TileMapDoc()
        self.tileset_pixmaps: list = []     # QPixmap pro Tileset (parallel)
        self.sel_local = 0
        self.undo_stack: list = []
        self.redo_stack: list = []

        self._build_ui()
        self._sync_layers()
        self._refresh_views()

    # ---------------------------------------------------- UI-Aufbau
    def _build_ui(self) -> None:
        self._make_toolbar()
        self._make_menu()

        # Mitte: Canvas in ScrollArea
        self.canvas = _Canvas()
        self.canvas.committed.connect(self._on_committed)
        self.canvas.obj_committed.connect(self._on_obj_committed)
        self.canvas.obj_edit_requested.connect(self._edit_object)
        self.canvas.obj_selected.connect(self._on_obj_selected)
        self.canvas.cell_hovered.connect(self._on_hover)
        self.canvas.picked.connect(self._on_picked)
        self.scroll = QScrollArea()
        self.scroll.setWidget(self.canvas)
        self.scroll.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.setCentralWidget(self.scroll)

        # Links: Palette
        left = QWidget(); lv = QVBoxLayout(left)
        lv.setContentsMargins(4, 4, 4, 4)
        # Tileset-Auswahl (Multi-Tileset)
        ts_row = QHBoxLayout()
        ts_row.addWidget(QLabel("Tileset:"))
        self.ts_combo = QComboBox()
        self.ts_combo.currentIndexChanged.connect(self._on_tileset_changed)
        ts_row.addWidget(self.ts_combo, 1)
        b_ts_add = QToolButton(); b_ts_add.setText("+")
        b_ts_add.setToolTip("Tileset hinzufuegen")
        b_ts_add.clicked.connect(self._load_tileset)
        ts_row.addWidget(b_ts_add)
        b_ts_del = QToolButton(); b_ts_del.setText("−")
        b_ts_del.setToolTip("Aktives Tileset entfernen (nur wenn unbenutzt)")
        b_ts_del.clicked.connect(self._remove_tileset)
        ts_row.addWidget(b_ts_del)
        lv.addLayout(ts_row)
        self.palette = _Palette()
        self.palette.tile_selected.connect(self._on_palette_select)
        pscroll = QScrollArea(); pscroll.setWidget(self.palette)
        pscroll.setWidgetResizable(False)
        pscroll.setMinimumWidth(220)
        lv.addWidget(pscroll, 1)
        b_props = QPushButton("Tile-Eigenschaften ...")
        b_props.clicked.connect(self._edit_props)
        lv.addWidget(b_props)
        self.sel_label = QLabel("Tile: 0")
        lv.addWidget(self.sel_label)
        self._dock("Palette", left, Qt.DockWidgetArea.LeftDockWidgetArea)

        # Rechts: Layer
        right = QWidget(); rv = QVBoxLayout(right)
        rv.setContentsMargins(4, 4, 4, 4)
        rv.addWidget(QLabel("Layer (oben = vorne)"))
        self.layer_list = QListWidget()
        self.layer_list.setSelectionMode(
            QAbstractItemView.SelectionMode.SingleSelection)
        self.layer_list.currentRowChanged.connect(self._on_layer_row)
        self.layer_list.itemChanged.connect(self._on_layer_item_changed)
        self.layer_list.itemDoubleClicked.connect(self._rename_layer)
        rv.addWidget(self.layer_list, 1)
        btns = QHBoxLayout()
        for txt, fn, tip in (
            ("+", self._add_layer, "Tile-Layer hinzufuegen"),
            ("+◇", self._add_object_layer, "Object-Layer hinzufuegen "
                "(Spawn-Punkte / Trigger / Zonen)"),
            ("-", self._del_layer, "Layer loeschen"),
            ("▲", lambda: self._move_layer(-1), "nach vorne"),
            ("▼", lambda: self._move_layer(1), "nach hinten"),
        ):
            b = QPushButton(txt); b.setFixedWidth(40); b.setToolTip(tip)
            b.clicked.connect(fn)
            btns.addWidget(b)
        btns.addStretch(1)
        rv.addLayout(btns)
        self.obj_hint = QLabel("")
        self.obj_hint.setWordWrap(True)
        self.obj_hint.setStyleSheet(f"color: {COLORS['fg_muted']};")
        rv.addWidget(self.obj_hint)
        self._dock("Layer", right, Qt.DockWidgetArea.RightDockWidgetArea)

        # Entf loescht das selektierte Objekt (Object-Layer) bzw. die
        # Tile-Auswahl (Select-Tool).
        from PySide6.QtGui import QShortcut
        sc_del = QShortcut(QKeySequence(Qt.Key.Key_Delete), self.canvas)
        sc_del.activated.connect(self.canvas.delete_selected)
        # Auswahl-Clipboard (Select-Tool): Kopieren/Ausschneiden/Einfuegen.
        for keyseq, fn in (
            (QKeySequence.StandardKey.Copy, self.canvas.copy_selection),
            (QKeySequence.StandardKey.Cut, self.canvas.cut_selection),
            (QKeySequence.StandardKey.Paste, self.canvas.paste_clipboard),
        ):
            QShortcut(keyseq, self.canvas).activated.connect(fn)

        self.status = self.statusBar()
        self._update_status()

    def _dock(self, title, widget, area):
        d = QDockWidget(title, self)
        d.setWidget(widget)
        d.setFeatures(QDockWidget.DockWidgetFeature.DockWidgetMovable |
                      QDockWidget.DockWidgetFeature.DockWidgetFloatable)
        self.addDockWidget(area, d)
        return d

    def _make_toolbar(self) -> None:
        tb = QToolBar("Haupt"); tb.setMovable(False)
        tb.setIconSize(QSize(20, 20))
        self.addToolBar(tb)

        def act(icon, text, slot, shortcut=None):
            a = QAction(icons.get(icon) if icon else icons.get("new"), text, self)
            if shortcut:
                a.setShortcut(QKeySequence(shortcut))
            a.triggered.connect(slot)
            tb.addAction(a)
            return a

        act("new", "Neu", self._new_map, "Ctrl+N")
        act("open", "Oeffnen", self._open, "Ctrl+O")
        act("save", "Speichern", self._save, "Ctrl+S")
        act("save_as", "Speichern unter", self._save_as, "Ctrl+Shift+S")
        tb.addSeparator()
        act("sprite_editor", "Tileset laden", self._load_tileset)
        tb.addSeparator()

        # Werkzeuge (checkable)
        grp = QActionGroup(self); grp.setExclusive(True)
        self._tool_actions = {}
        for tool, label, sc in (
            (TOOL_PENCIL, "Stift", "B"), (TOOL_ERASER, "Radierer", "E"),
            (TOOL_FILL, "Fuellen", "G"), (TOOL_RECT, "Rechteck", "R"),
            (TOOL_PICK, "Pipette", "I"), (TOOL_SELECT, "Auswahl", "S"),
        ):
            a = QAction(label, self); a.setCheckable(True)
            a.setShortcut(QKeySequence(sc))
            a.triggered.connect(lambda _c, t=tool: self._set_tool(t))
            grp.addAction(a); tb.addAction(a)
            self._tool_actions[tool] = a
        self._tool_actions[TOOL_PENCIL].setChecked(True)
        tb.addSeparator()

        a_grid = QAction("Gitter", self); a_grid.setCheckable(True)
        a_grid.setChecked(True)
        a_grid.triggered.connect(self._toggle_grid)
        tb.addAction(a_grid)
        a_dim = QAction("Andere abblenden", self); a_dim.setCheckable(True)
        a_dim.setChecked(True)
        a_dim.triggered.connect(self._toggle_dim)
        tb.addAction(a_dim)
        tb.addSeparator()
        act("unfold", "Zoom +", lambda: self._zoom(1), "Ctrl++")
        act("fold", "Zoom -", lambda: self._zoom(-1), "Ctrl+-")
        tb.addSeparator()
        act("bench", "GB-Code", self._export_code)

    def _menu_action(self, menu, text, slot, shortcut=None) -> QAction:
        a = QAction(text, self)
        if shortcut:
            a.setShortcut(QKeySequence(shortcut))
        a.triggered.connect(slot)
        menu.addAction(a)
        return a

    def _make_menu(self) -> None:
        m = self.menuBar()
        # Shortcuts fuer Datei-Aktionen liegen schon auf den Toolbar-Actions
        # -> hier ohne, sonst meldet Qt mehrdeutige Shortcuts.
        mf = m.addMenu("&Datei")
        self._menu_action(mf, "Neu", self._new_map)
        self._menu_action(mf, "Oeffnen ...", self._open)
        self._menu_action(mf, "Speichern", self._save)
        self._menu_action(mf, "Speichern unter ...", self._save_as)
        mf.addSeparator()
        self._menu_action(mf, "Tileset laden ...", self._load_tileset)
        self._menu_action(mf, "GB-Code exportieren ...", self._export_code)
        mf.addSeparator()
        self._menu_action(mf, "Schliessen", self.close)

        me = m.addMenu("&Bearbeiten")
        self._menu_action(me, "Rueckgaengig", self._undo, "Ctrl+Z")
        self._menu_action(me, "Wiederholen", self._redo, "Ctrl+Y")

        mm = m.addMenu("&Karte")
        self._menu_action(mm, "Groesse aendern ...", self._resize_map)

    # ---------------------------------------------------- Tileset
    def _load_tileset(self) -> None:
        start = str(self.project_root)
        path, _ = QFileDialog.getOpenFileName(
            self, "Tileset-Bild laden", start, "Bilder (*.png *.bmp *.jpg)")
        if not path:
            return
        pix = QPixmap(path)
        if pix.isNull():
            QMessageBox.warning(self, "Fehler", "Bild konnte nicht geladen werden.")
            return
        # Hinzufuegen als weiteres Tileset (das erste legt Tileset 0 an).
        idx = self.doc.add_tileset(path, pix.width(), pix.height())
        if idx >= len(self.tileset_pixmaps):
            self.tileset_pixmaps.extend(
                [None] * (idx + 1 - len(self.tileset_pixmaps)))
        self.tileset_pixmaps[idx] = pix
        self.sel_local = 0
        self._refresh_views()
        self._update_status()

    def _remove_tileset(self) -> None:
        idx = self.doc.active_tileset
        if self.doc.active_ts is None:
            return
        # Nur entfernen, wenn keine platzierten Tiles dieses Tileset nutzen
        # (sonst wuerden GIDs ungueltig) -- `tileset_in_use` ist jetzt die
        # EINE Quelle dieser Pruefung; remove_tileset() selbst wuerde bei
        # Benutzung ohnehin ablehnen (frueher verliess sich das Modell
        # allein auf diese UI-Pruefung).
        if self.doc.tileset_in_use(idx):
            QMessageBox.information(
                self, "Tileset in Benutzung",
                "Dieses Tileset wird noch von platzierten Tiles genutzt "
                "und kann nicht entfernt werden.")
            return
        if not self.doc.remove_tileset(idx):
            return
        if 0 <= idx < len(self.tileset_pixmaps):
            del self.tileset_pixmaps[idx]
        self.sel_local = 0
        self._refresh_views()
        self._update_status()

    def _on_tileset_changed(self, idx: int) -> None:
        if 0 <= idx < len(self.doc.tilesets) and idx != self.doc.active_tileset:
            self.doc.active_tileset = idx
            self.sel_local = 0
            self._refresh_views()
            self._update_status()

    # ---------------------------------------------------- Datei
    def _confirm_discard_changes(self) -> bool:
        """Fragt bei ungespeicherten Aenderungen nach, bevor sie verworfen
        werden (Neu/Oeffnen/Schliessen). Frueher ersetzten _new_map()/_open()
        self.doc kommentarlos -- ungespeicherte Arbeit ging stillschweigend
        verloren, obwohl doc.dirty (von save_json/load_json korrekt
        zurueckgesetzt) das bereits zuverlaessig haette verhindern koennen."""
        if not self.doc.dirty:
            return True
        ans = QMessageBox.question(
            self, "Ungespeicherte Aenderungen",
            "Die aktuelle Map hat ungespeicherte Aenderungen.\n\n"
            "Trotzdem fortfahren und verwerfen?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
            QMessageBox.StandardButton.No)
        return ans == QMessageBox.StandardButton.Yes

    def closeEvent(self, event) -> None:  # noqa: N802
        if self._confirm_discard_changes():
            event.accept()
        else:
            event.ignore()

    def _new_map(self) -> None:
        if not self._confirm_discard_changes():
            return
        params = _ask_map_params(self, self.doc.width, self.doc.height,
                                 self.doc.tile_w, self.doc.tile_h)
        if not params:
            return
        w, h, tw, th = params
        self.doc = TileMapDoc(w, h, tw, th)
        self.tileset_pixmaps = []
        self.sel_local = 0
        self.undo_stack.clear(); self.redo_stack.clear()
        self._sync_layers()
        self._refresh_views()
        self._update_status()

    def _load_tileset_pixmaps(self) -> None:
        """Laedt fuer jedes Tileset des Dokuments das Bild (parallele Liste)."""
        self.tileset_pixmaps = []
        for ts in self.doc.tilesets:
            pix = None
            if ts.image_abs and os.path.exists(ts.image_abs):
                p = QPixmap(ts.image_abs)
                if not p.isNull():
                    pix = p
            self.tileset_pixmaps.append(pix)

    def _open(self) -> None:
        if not self._confirm_discard_changes():
            return
        path, _ = QFileDialog.getOpenFileName(
            self, "Tiled-Map oeffnen", str(self.project_root),
            "Tiled-Map (*.json *.tmj)")
        if not path:
            return
        try:
            self.doc = TileMapDoc.load_json(path)
        except Exception as exc:
            QMessageBox.warning(self, "Fehler", f"Konnte nicht laden:\n{exc}")
            return
        # Tileset-Bilder laden (alle Tilesets)
        self._load_tileset_pixmaps()
        self.sel_local = 0
        self.undo_stack.clear(); self.redo_stack.clear()
        self._sync_layers()
        self._refresh_views()
        self._update_status()

    def _save(self) -> None:
        if not self.doc.path:
            self._save_as()
            return
        self._do_save(self.doc.path)

    def _save_as(self) -> None:
        path, _ = QFileDialog.getSaveFileName(
            self, "Tiled-Map speichern", str(self.project_root),
            "Tiled-Map (*.json)")
        if not path:
            return
        if not path.lower().endswith((".json", ".tmj")):
            path += ".json"
        self._do_save(path)

    def _do_save(self, path: str) -> None:
        try:
            self.doc.save_json(path)
        except Exception as exc:
            QMessageBox.warning(self, "Fehler", f"Konnte nicht speichern:\n{exc}")
            return
        self.setWindowTitle(f"GameBasic Tilemap-Editor -- {Path(path).name}")
        self.status.showMessage(f"Gespeichert: {path}", 4000)

    def _export_code(self) -> None:
        code = self.doc.gb_code(self.doc.path or None)
        dlg = QDialog(self); dlg.setWindowTitle("GB-Code (Tilemap-Renderer)")
        dlg.resize(680, 560)
        dl = QVBoxLayout(dlg)
        dl.addWidget(QLabel(
            "Selbststaendiger Renderer. Map als .json speichern + Tileset-Bild "
            "daneben legen, dann ausfuehren."))
        edit = QPlainTextEdit(); edit.setPlainText(code); edit.setReadOnly(True)
        edit.setFont(QFont(EDITOR_FONT_FAMILY, 10))
        dl.addWidget(edit, 1)
        row = QHBoxLayout(); row.addStretch(1)
        b = QPushButton("In Zwischenablage"); b.setProperty("accent", True)
        b.clicked.connect(lambda: QApplication.clipboard().setText(code))
        row.addWidget(b)
        dl.addLayout(row)
        dlg.exec()

    # ---------------------------------------------------- Werkzeuge/Ansicht
    def _set_tool(self, tool: int) -> None:
        self.canvas.tool = tool
        if tool != TOOL_SELECT:
            self.canvas.clear_selection()

    def _toggle_grid(self, on: bool) -> None:
        self.canvas.show_grid = on
        self.canvas.update()

    def _toggle_dim(self, on: bool) -> None:
        self.canvas.dim_others = on
        self.canvas.update()

    def _zoom(self, delta: int) -> None:
        self.canvas.set_zoom(self.canvas.zoom + delta)

    # ---------------------------------------------------- Palette/Auswahl
    def _on_palette_select(self, lid: int) -> None:
        self.sel_local = lid
        gid = self.doc.local_to_gid(self.doc.active_tileset, lid)
        self.canvas.current_gid = gid
        self.sel_label.setText(f"Tile: {lid} (gid {gid})")

    def _on_picked(self, lid: int) -> None:
        # Pipette liefert eine gid; aufs richtige Tileset umschalten.
        gid = lid + 1
        tsi, local = self.doc.gid_to_tileset(gid)
        if tsi >= 0 and tsi != self.doc.active_tileset:
            self.doc.active_tileset = tsi
            self._refresh_views()
        elif tsi < 0:
            local = lid
        self.palette.sel = local
        self.palette.update()
        self._on_palette_select(local)

    def _edit_props(self) -> None:
        if self.doc.columns <= 0:
            QMessageBox.information(self, "Kein Tileset",
                                   "Erst ein Tileset laden.")
            return
        dlg = _PropDialog(self.doc, self.sel_local, self)
        if dlg.exec() == QDialog.DialogCode.Accepted:
            dlg.apply_to_doc()

    # ---------------------------------------------------- Layer
    def _sync_layers(self) -> None:
        """Layer-Liste aus dem Doc neu aufbauen (oberster = vorderster)."""
        self.layer_list.blockSignals(True)
        self.layer_list.clear()
        for li in reversed(range(len(self.doc.layers))):
            l = self.doc.layers[li]
            label = f"◇ {l.name}" if isinstance(l, ObjectLayer) else l.name
            it = QListWidgetItem(label)
            it.setData(Qt.ItemDataRole.UserRole, li)
            it.setFlags(it.flags() | Qt.ItemFlag.ItemIsUserCheckable)
            it.setCheckState(Qt.CheckState.Checked if l.visible
                             else Qt.CheckState.Unchecked)
            self.layer_list.addItem(it)
        # aktive Zeile = aktiver Layer
        self._select_layer_row(self.canvas.active_layer)
        self.layer_list.blockSignals(False)

    def _select_layer_row(self, layer_idx: int) -> None:
        for row in range(self.layer_list.count()):
            if self.layer_list.item(row).data(Qt.ItemDataRole.UserRole) == layer_idx:
                self.layer_list.setCurrentRow(row)
                return

    def _on_layer_row(self, row: int) -> None:
        if row < 0:
            return
        li = self.layer_list.item(row).data(Qt.ItemDataRole.UserRole)
        self.canvas.active_layer = li
        self._update_obj_hint()
        self.canvas.update()

    def _update_obj_hint(self) -> None:
        li = self.canvas.active_layer
        is_obj = (0 <= li < len(self.doc.layers)
                  and isinstance(self.doc.layers[li], ObjectLayer))
        if is_obj:
            self.obj_hint.setText(
                "Object-Layer aktiv: Klick = Punkt (Spawn), Ziehen = Rechteck "
                "(Zone); Klick auf Objekt = waehlen/ziehen, Doppelklick = "
                "bearbeiten, Entf / Rechtsklick = loeschen.")
        else:
            self.obj_hint.setText("")

    def _on_layer_item_changed(self, item: QListWidgetItem) -> None:
        li = item.data(Qt.ItemDataRole.UserRole)
        if li is None or li >= len(self.doc.layers):
            return
        self.doc.layers[li].visible = (item.checkState() == Qt.CheckState.Checked)
        self.doc.dirty = True
        self.canvas.update()

    def _rename_layer(self, item: QListWidgetItem) -> None:
        li = item.data(Qt.ItemDataRole.UserRole)
        name, ok = QInputDialog.getText(self, "Layer umbenennen", "Name:",
                                        text=self.doc.layers[li].name)
        if ok and name.strip():
            self.doc.layers[li].name = name.strip()
            self.doc.dirty = True
            self._sync_layers()

    def _add_layer(self) -> None:
        before = self._snapshot_doc()
        idx = self.doc.add_layer()
        self.canvas.active_layer = idx
        self._push_doc_undo(before)
        self._sync_layers()
        self.canvas.update()

    def _add_object_layer(self) -> None:
        before = self._snapshot_doc()
        idx = self.doc.add_object_layer()
        self.canvas.active_layer = idx
        self.canvas.selected_obj = None
        self._push_doc_undo(before)
        self._sync_layers()
        self._update_obj_hint()
        self.canvas.update()

    # ---------------------------------------------------- Objekte
    def _edit_object(self, obj) -> None:
        """Bearbeitet Name/Typ/Properties eines Objekts (Doppelklick/Neuanlage)."""
        if obj is None:
            return
        li = self.canvas.active_layer
        layer = self.doc.layers[li] if 0 <= li < len(self.doc.layers) else None
        if not isinstance(layer, ObjectLayer) or obj not in layer.objects:
            return
        before = copy.deepcopy(layer.objects)
        dlg = _ObjectDialog(obj, self)
        if dlg.exec() == QDialog.DialogCode.Accepted:
            dlg.apply_to_obj()
            self._on_obj_committed(li, before, copy.deepcopy(layer.objects))
            self.doc.dirty = True
            self.canvas.update()
            self._on_obj_selected(obj)

    def _on_obj_selected(self, obj) -> None:
        if obj is None:
            return
        kind = "Punkt" if obj.is_point() else "Rechteck"
        nm = obj.name or "(ohne Name)"
        self.status.showMessage(
            f"{kind}: {nm}  typ='{obj.type}'  @ ({obj.x:.0f}, {obj.y:.0f})"
            + ("" if obj.is_point() else f"  {obj.width:.0f}x{obj.height:.0f}"))

    def _del_layer(self) -> None:
        if len(self.doc.layers) <= 1:
            return
        before = self._snapshot_doc()
        self.doc.remove_layer(self.canvas.active_layer)
        self.canvas.active_layer = min(self.canvas.active_layer,
                                       len(self.doc.layers) - 1)
        self._push_doc_undo(before)
        self._sync_layers()
        self.canvas.update()

    def _move_layer(self, delta: int) -> None:
        before = self._snapshot_doc()
        new = self.doc.move_layer(self.canvas.active_layer, delta)
        self.canvas.active_layer = new
        self._push_doc_undo(before)
        self._sync_layers()
        self.canvas.update()

    # ---------------------------------------------------- Karte
    def _resize_map(self) -> None:
        params = _ask_map_params(self, self.doc.width, self.doc.height,
                                 self.doc.tile_w, self.doc.tile_h)
        if not params:
            return
        w, h, tw, th = params
        before = self._snapshot_doc()
        retile = (tw != self.doc.tile_w or th != self.doc.tile_h)
        self.doc.tile_w, self.doc.tile_h = tw, th
        self.doc.resize(w, h)
        if retile:
            # Tile-Groesse aller Tilesets angleichen + Spalten/Anzahl neu.
            for i, ts in enumerate(self.doc.tilesets):
                ts.tile_w, ts.tile_h = tw, th
                pix = (self.tileset_pixmaps[i]
                       if i < len(self.tileset_pixmaps) else None)
                if pix is not None:
                    ts.set_image(ts.image_abs or ts.image,
                                 pix.width(), pix.height())
            self.doc.recompute_firstgids()
        self._push_doc_undo(before)
        self._refresh_views()
        self._update_status()

    # ---------------------------------------------------- Undo/Redo
    # Eintraege: ("tiles", li, before, after) / ("objects", li, before, after)
    # / ("doc", None, before, after) -- letzteres ein voller Struktur-Snapshot
    # (Layer-Liste + Geometrie + Tilesets) fuer Layer-Add/Del/Move und Resize.
    # Frueher wurde bei diesen strukturellen Operationen die GESAMTE Undo-
    # Historie verworfen (undo_stack.clear()) -- die Ops selbst waren dadurch
    # NICHT rueckgaengig machbar (dieselbe Bug-Klasse wie zuvor im Sprite-
    # Editor: Crop-to-Content/Rotate/Flatten waren dort ebenfalls nicht
    # undoable). Der volle Snapshot macht sie undoable UND haelt die
    # layer_idx-Referenzen aelterer Tiles/Objects-Eintraege gueltig, solange
    # Undo/Redo strikt in Stack-Reihenfolge angewendet wird (das "doc"-Undo
    # direkt davor stellt die Layer-Liste wieder in den Zustand her, in dem
    # der aeltere Eintrag urspruenglich gueltig war).
    def _snapshot_doc(self) -> dict:
        return {
            "width": self.doc.width, "height": self.doc.height,
            "tile_w": self.doc.tile_w, "tile_h": self.doc.tile_h,
            "layers": copy.deepcopy(self.doc.layers),
            "tilesets": copy.deepcopy(self.doc.tilesets),
        }

    def _push_doc_undo(self, before: dict) -> None:
        self.undo_stack.append(("doc", None, before, self._snapshot_doc()))
        self.redo_stack.clear()

    def _on_committed(self, layer_idx, before, after) -> None:
        self.undo_stack.append(("tiles", layer_idx, before, after))
        self.redo_stack.clear()

    def _on_obj_committed(self, layer_idx, before, after) -> None:
        self.undo_stack.append(("objects", layer_idx, before, after))
        self.redo_stack.clear()

    def _restore_undo(self, entry, *, use_before: bool) -> None:
        kind, li, before, after = entry
        snap = before if use_before else after
        if kind == "doc":
            self.doc.width = snap["width"]; self.doc.height = snap["height"]
            self.doc.tile_w = snap["tile_w"]; self.doc.tile_h = snap["tile_h"]
            self.doc.layers = copy.deepcopy(snap["layers"])
            self.doc.tilesets = copy.deepcopy(snap["tilesets"])
            self.canvas.active_layer = min(self.canvas.active_layer,
                                           len(self.doc.layers) - 1)
            self.canvas.selected_obj = None
            self.canvas.obj_selected.emit(None)
            self.doc.dirty = True
            self._sync_layers()
            self._refresh_views()
            self.canvas.update()
            return
        if li >= len(self.doc.layers):
            return
        layer = self.doc.layers[li]
        if kind == "tiles" and isinstance(layer, TileLayer):
            layer.tiles = list(snap)
        elif kind == "objects" and isinstance(layer, ObjectLayer):
            layer.objects = copy.deepcopy(snap)
            self.canvas.selected_obj = None
            self.canvas.obj_selected.emit(None)
        self.doc.dirty = True
        self.canvas.update()

    def _undo(self) -> None:
        if not self.undo_stack:
            return
        entry = self.undo_stack.pop()
        self._restore_undo(entry, use_before=True)
        self.redo_stack.append(entry)

    def _redo(self) -> None:
        if not self.redo_stack:
            return
        entry = self.redo_stack.pop()
        self._restore_undo(entry, use_before=False)
        self.undo_stack.append(entry)

    # ---------------------------------------------------- Views/Status
    def _refresh_views(self) -> None:
        # Pixmap-Liste auf die Tileset-Liste laengen-angleichen.
        while len(self.tileset_pixmaps) < len(self.doc.tilesets):
            self.tileset_pixmaps.append(None)
        del self.tileset_pixmaps[len(self.doc.tilesets):]
        ai = self.doc.active_tileset
        active_pix = (self.tileset_pixmaps[ai]
                      if 0 <= ai < len(self.tileset_pixmaps) else None)
        self.canvas.set_doc(self.doc, self.tileset_pixmaps)
        self.palette.set_doc(self.doc, active_pix)
        self.canvas.current_gid = self.doc.local_to_gid(ai, self.sel_local)
        self._refresh_tileset_combo()
        self._sync_layers()

    def _refresh_tileset_combo(self) -> None:
        self.ts_combo.blockSignals(True)
        self.ts_combo.clear()
        for i, ts in enumerate(self.doc.tilesets):
            self.ts_combo.addItem(f"{i}: {ts.name}")
        if self.doc.tilesets:
            self.ts_combo.setCurrentIndex(
                min(self.doc.active_tileset, len(self.doc.tilesets) - 1))
        self.ts_combo.blockSignals(False)

    def _on_hover(self, cx: int, cy: int) -> None:
        if 0 <= cx < self.doc.width and 0 <= cy < self.doc.height:
            self.status.showMessage(f"Tile ({cx}, {cy})")

    def _update_status(self) -> None:
        n = len(self.doc.tilesets)
        if n == 0:
            ts = "kein Tileset"
        else:
            tiles = sum(t.tile_count for t in self.doc.tilesets)
            ts = f"{n} Tileset(s), {tiles} Tiles"
        self.status.showMessage(
            f"Karte {self.doc.width}x{self.doc.height} @ "
            f"{self.doc.tile_w}x{self.doc.tile_h}px  |  {ts}")


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance() or QApplication([])
    app.setStyleSheet(global_qss())
    win = TileMapEditor(project_root)
    if initial_file and Path(initial_file).exists():
        try:
            win.doc = TileMapDoc.load_json(str(initial_file))
            win._load_tileset_pixmaps()
            win._refresh_views(); win._update_status()
        except Exception as exc:
            # Frueher komplett stumm -- bei kaputtem/falsch formatiertem
            # initial_file oeffnete sich das Fenster einfach mit einer
            # leeren Map, ohne dass der User je erfuhr, dass sein File
            # nicht geladen wurde.
            QMessageBox.warning(
                win, "Fehler",
                f"Konnte '{initial_file}' nicht laden:\n{exc}")
    win.show()
    return app.exec()
