"""GameBasic Sprite-Editor (PySide6).

Pixel-Art-Editor mit Alpha-Kanal und Frame-Animation. Diese Version nutzt
Qt/PySide6 statt Tk -- gibt scharfere Icons, smoothes Zoomen via
QGraphicsView und native Toolbar/Statusbar/Dialoge.

Output ist immer PNG mit Alpha-Kanal -- direkt von LOADIMAGE / DRAWIMAGE
und vom sprite-Modul (SPRITE_NEW) ladbar. Multi-Frame-Sprites werden
als horizontaler Sheet exportiert.

Datei-Formate:
- .png  -- Single-Frame oder Multi-Frame als horizontaler Sheet
- .gbsprite -- natives Format (JSON + Base64-RGBA pro Frame)

Tools:
    Stift (B), Radierer (E), Eimer (G), Linie (L),
    Rechteck (R), Ellipse (O), Pipette (I), Auswahl (M),
    Verschieben (V).
Linke Maustaste = Vordergrundfarbe, Rechte = Hintergrundfarbe.

Tastenkuerzel:
    Strg+N Neu          Strg+S Speichern
    Strg+O Oeffnen      Strg+Shift+S Speichern unter
    Strg+E Sheet-Export Strg+Z Undo  Strg+Y Redo
    Strg+H Flip H       Strg+J Flip V
    Strg+. Rotate CW    Strg+, Rotate CCW
    Strg+P Animation-Preview
    1..4   Brush-Groesse  X Farben tauschen
    +/-/0  Zoom         F2..F9 Frame waehlen
"""
from __future__ import annotations

import base64
import io
import json
import sys
from collections import deque
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, Iterable

try:
    from PIL import Image, ImageDraw
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "Pillow ist nicht installiert.\n"
        "Im .venv installieren:\n"
        "  .venv\\Scripts\\python.exe -m pip install Pillow"
    ) from exc

try:
    from PySide6.QtCore import (
        Qt, QPoint, QPointF, QRect, QRectF, QSize, QTimer, Signal,
        QFileSystemWatcher,
    )
    from PySide6.QtGui import (
        QAction, QBrush, QColor, QCursor, QIcon, QImage, QKeySequence,
        QPainter, QPen, QPixmap, QPolygon, QPolygonF, QShortcut,
        QStandardItemModel, QStandardItem, QTransform,
    )
    from PySide6.QtWidgets import (
        QApplication, QButtonGroup, QCheckBox, QColorDialog, QComboBox,
        QDialog, QDialogButtonBox, QDockWidget, QFileDialog, QFrame,
        QGraphicsScene, QGraphicsView, QGridLayout, QGroupBox, QHBoxLayout,
        QInputDialog, QLabel, QLineEdit, QListView, QListWidget,
        QListWidgetItem, QMainWindow, QMenu, QMenuBar, QMessageBox,
        QPushButton, QSizePolicy, QSlider, QSpinBox, QStatusBar, QStyle,
        QToolBar, QToolButton, QVBoxLayout, QWidget,
    )
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "PySide6 ist nicht installiert.\n"
        "Im .venv installieren:\n"
        "  .venv\\Scripts\\python.exe -m pip install PySide6"
    ) from exc


# ============================================================
# Konstanten / Theme
# ============================================================

# Dunkles Theme -- abgestimmt auf das GameBasic-Logo (cyan/mint auf navy).
# Wird als globales Stylesheet auf das QApplication-Objekt gesetzt;
# einzelne Widgets ueberschreiben nur Spezialfaelle.
# Pixel-Editor-spezifische Farben (grid/selection/checker) bleiben
# bewusst getrennt von der UI-Palette.
COLORS = {
    "bg":           "#0B1421",
    "bg_alt":       "#121E2C",
    "bg_panel":     "#1A2738",
    "bg_hover":     "#243549",
    "fg":           "#D5EDF0",
    "fg_muted":     "#7A8E96",
    "border":       "#243A4D",
    "accent":       "#2DE0E0",
    "accent_hover": "#5EEAEA",
    "success":      "#2DE89A",
    "danger":       "#E55A8C",
    # Pixel-Editor-Canvas-Farben -- konstrastieren bewusst gegen das UI-Theme
    "grid":         "#1F3346",
    "grid_strong":  "#3A5566",
    "selection":    "#FFCC00",
    "checker_a":    "#1A2738",
    "checker_b":    "#243549",
}

GLOBAL_QSS = f"""
QMainWindow, QDialog, QDockWidget {{
    background-color: {COLORS["bg"]};
    color: {COLORS["fg"]};
}}
QMenuBar {{
    background-color: {COLORS["bg_panel"]};
    color: {COLORS["fg"]};
    border-bottom: 1px solid {COLORS["border"]};
}}
QMenuBar::item:selected {{
    background-color: {COLORS["bg_hover"]};
}}
QMenu {{
    background-color: {COLORS["bg_panel"]};
    color: {COLORS["fg"]};
    border: 1px solid {COLORS["border"]};
}}
QMenu::item:selected {{
    background-color: {COLORS["accent"]};
}}
QMenu::separator {{
    height: 1px;
    background: {COLORS["border"]};
    margin: 4px 8px;
}}
QToolBar {{
    background-color: {COLORS["bg_panel"]};
    border: none;
    spacing: 4px;
    padding: 4px;
}}
QToolBar::separator {{
    background: {COLORS["border"]};
    width: 1px;
    margin: 4px 6px;
}}
QToolButton {{
    background-color: transparent;
    color: {COLORS["fg"]};
    border: 1px solid transparent;
    border-radius: 4px;
    padding: 4px 6px;
}}
QToolButton:hover {{
    background-color: {COLORS["bg_hover"]};
    border: 1px solid {COLORS["border"]};
}}
QToolButton:checked {{
    background-color: {COLORS["accent"]};
    border: 1px solid {COLORS["accent_hover"]};
}}
QPushButton {{
    background-color: {COLORS["bg_hover"]};
    color: {COLORS["fg"]};
    border: 1px solid {COLORS["border"]};
    border-radius: 4px;
    padding: 6px 14px;
}}
QPushButton:hover {{
    background-color: {COLORS["accent_hover"]};
    border-color: {COLORS["accent"]};
}}
QPushButton:pressed {{
    background-color: {COLORS["accent"]};
}}
QPushButton#primary {{
    background-color: {COLORS["accent"]};
    color: white;
    border-color: {COLORS["accent_hover"]};
}}
QPushButton#primary:hover {{
    background-color: {COLORS["accent_hover"]};
}}
QPushButton#success {{
    background-color: {COLORS["success"]};
    color: white;
    border-color: #22C55E;
}}
QPushButton#success:hover {{
    background-color: #22C55E;
}}
QStatusBar {{
    background-color: {COLORS["bg_panel"]};
    color: {COLORS["fg_muted"]};
    border-top: 1px solid {COLORS["border"]};
}}
QStatusBar::item {{ border: 0; }}
QDockWidget {{
    color: {COLORS["fg_muted"]};
    titlebar-close-icon: none;
    titlebar-normal-icon: none;
}}
QDockWidget::title {{
    background: {COLORS["bg_panel"]};
    padding: 6px 10px;
    border-bottom: 1px solid {COLORS["border"]};
    text-align: left;
    font-weight: bold;
    text-transform: uppercase;
    font-size: 11px;
    letter-spacing: 1px;
}}
QLabel {{
    color: {COLORS["fg"]};
    background: transparent;
}}
QLabel#sectionHeader {{
    color: {COLORS["fg_muted"]};
    font-weight: bold;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    padding: 6px 0 2px 0;
}}
QSlider::groove:horizontal {{
    border: none;
    background: {COLORS["bg_alt"]};
    height: 6px;
    border-radius: 3px;
}}
QSlider::sub-page:horizontal {{
    background: {COLORS["accent"]};
    border-radius: 3px;
}}
QSlider::add-page:horizontal {{
    background: {COLORS["bg_alt"]};
    border-radius: 3px;
}}
QSlider::handle:horizontal {{
    background: {COLORS["fg"]};
    border: 1px solid {COLORS["border"]};
    width: 12px;
    height: 12px;
    margin: -4px 0;
    border-radius: 6px;
}}
QSlider::handle:horizontal:hover {{
    background: {COLORS["accent_hover"]};
}}
QLineEdit, QSpinBox, QComboBox {{
    background-color: {COLORS["bg_alt"]};
    color: {COLORS["fg"]};
    border: 1px solid {COLORS["border"]};
    border-radius: 4px;
    padding: 4px 6px;
    selection-background-color: {COLORS["accent"]};
}}
QLineEdit:focus, QSpinBox:focus, QComboBox:focus {{
    border: 1px solid {COLORS["accent"]};
}}
QListWidget {{
    background-color: {COLORS["bg_alt"]};
    color: {COLORS["fg"]};
    border: 1px solid {COLORS["border"]};
    border-radius: 4px;
    outline: none;
}}
QListWidget::item {{
    padding: 4px;
    border-bottom: 1px solid {COLORS["border"]};
}}
QListWidget::item:selected {{
    background-color: {COLORS["accent"]};
    color: white;
}}
QCheckBox {{
    color: {COLORS["fg"]};
    spacing: 8px;
}}
QCheckBox::indicator {{
    width: 14px;
    height: 14px;
    border: 1px solid {COLORS["border"]};
    border-radius: 3px;
    background: {COLORS["bg_alt"]};
}}
QCheckBox::indicator:checked {{
    background: {COLORS["accent"]};
    border-color: {COLORS["accent_hover"]};
}}
QGroupBox {{
    border: 1px solid {COLORS["border"]};
    border-radius: 4px;
    margin-top: 14px;
    padding-top: 10px;
    color: {COLORS["fg_muted"]};
    font-weight: bold;
}}
QGroupBox::title {{
    subcontrol-origin: margin;
    left: 10px;
    padding: 0 6px;
}}
QScrollBar:vertical {{
    background: {COLORS["bg_alt"]};
    width: 10px;
    border: none;
}}
QScrollBar::handle:vertical {{
    background: {COLORS["bg_hover"]};
    border-radius: 5px;
    min-height: 20px;
}}
QScrollBar::handle:vertical:hover {{
    background: {COLORS["accent"]};
}}
QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {{
    height: 0;
}}
QScrollBar:horizontal {{
    background: {COLORS["bg_alt"]};
    height: 10px;
    border: none;
}}
QScrollBar::handle:horizontal {{
    background: {COLORS["bg_hover"]};
    border-radius: 5px;
    min-width: 20px;
}}
QScrollBar::handle:horizontal:hover {{
    background: {COLORS["accent"]};
}}
QScrollBar::add-line:horizontal, QScrollBar::sub-line:horizontal {{
    width: 0;
}}
"""


DEFAULT_PALETTE = [
    (0, 0, 0, 255),         (255, 255, 255, 255),
    (155, 173, 183, 255),   (68, 137, 26, 255),
    (224, 60, 40, 255),     (37, 113, 121, 255),
    (158, 41, 47, 255),     (224, 142, 0, 255),
    (255, 209, 102, 255),   (240, 89, 65, 255),
    (180, 41, 60, 255),     (108, 31, 49, 255),
    (255, 156, 110, 255),   (220, 90, 70, 255),
    (140, 50, 40, 255),     (80, 30, 30, 255),
    (90, 200, 250, 255),    (50, 130, 220, 255),
    (40, 80, 170, 255),     (30, 50, 100, 255),
    (150, 230, 160, 255),   (60, 180, 100, 255),
    (30, 110, 60, 255),     (20, 60, 40, 255),
    (240, 240, 240, 255),   (200, 200, 200, 255),
    (140, 140, 140, 255),   (90, 90, 90, 255),
    (40, 40, 40, 255),      (255, 0, 255, 255),
    (255, 255, 0, 255),     (0, 255, 255, 255),
]

DEFAULT_DOC_SIZE = (16, 16)
ZOOM_LEVELS = [1, 2, 3, 4, 6, 8, 12, 16, 20, 24, 32, 40, 48]
DEFAULT_ZOOM_INDEX = 7


# ============================================================
# Datenmodell + PIL/Qt-Konversion -- siehe spriteeditor.document
# ============================================================
#
# Frame, SpriteDoc, DEFAULT_FRAME_DURATION_MS und pil_to_qpixmap leben
# in einem eigenen Modul. Hier nur Re-Exporte fuer Code, der noch via
# `from gamebasic.spriteeditor_qt import SpriteDoc` importiert.

from .spriteeditor.document import (  # noqa: E402,F401  -- intentional re-export
    DEFAULT_FRAME_DURATION_MS,
    Anim,
    Frame,
    SpriteDoc,
    pil_to_qpixmap,
)


# ============================================================
# Tool-Icons + Toolbar-Action-Icons -- siehe spriteeditor.icons
# ============================================================
#
# Beide Icon-Generatoren wohnen in einem eigenen Modul, weil sie ~540
# Zeilen QPainter-Code sind und keinerlei Editor-State brauchen.

from .spriteeditor.icons import (  # noqa: E402,F401  -- re-export
    make_tool_icon,
    make_action_icon,
)


# ============================================================
# Tools (Pixel-Manipulation auf SpriteDoc) -- siehe spriteeditor.tools
# ============================================================
#
# Tool-Klassen + Helpers leben in einem eigenen Modul. Hier nur
# Re-Exporte, damit Bestands-Code (`from spriteeditor_qt import PencilTool`)
# weiter laeuft.

from .spriteeditor.tools import (  # noqa: E402,F401  -- re-export
    Tool,
    PencilTool, EraserTool, SprayTool, BucketTool,
    LineTool, RectTool, EllipseTool,
    EyedropperTool, MoveTool, MagicWandTool, SelectTool,
    _bresenham, _brush_offsets, _symmetry_points,
)


# ============================================================
# Pixel-Canvas (QGraphicsView)
# ============================================================

class SpriteCanvas(QGraphicsView):
    """Zeichen-Canvas mit Schachbrett-Hintergrund, Frame-Pixmap, Onion-Skin,
    Preview-Layer (fuer Linien-Drag), Grid und Selection-Rechteck.

    Qt-Vorteile gegenueber Tk:
    - Smooth Mausrad-Zoom mit Anchor unter dem Cursor
    - Native Pan mit Mittelmaustaste
    - Selection-Rechteck als QGraphicsRectItem (kein delete/redraw)
    """

    def __init__(self, app: "SpriteEditorWindow"):
        super().__init__()
        self.app = app
        self._scene = QGraphicsScene(self)
        self.setScene(self._scene)
        self.setRenderHint(QPainter.Antialiasing, False)
        self.setRenderHint(QPainter.SmoothPixmapTransform, False)
        self.setTransformationAnchor(QGraphicsView.AnchorUnderMouse)
        self.setResizeAnchor(QGraphicsView.AnchorViewCenter)
        self.setDragMode(QGraphicsView.NoDrag)
        self.setBackgroundBrush(QBrush(QColor(COLORS["bg"])))
        self.setHorizontalScrollBarPolicy(Qt.ScrollBarAsNeeded)
        self.setVerticalScrollBarPolicy(Qt.ScrollBarAsNeeded)
        self.setMouseTracking(True)
        self.setFrameShape(QFrame.NoFrame)

        self._show_grid = True
        self._show_onion = False
        self._show_tile_preview = False
        self._zoom = ZOOM_LEVELS[DEFAULT_ZOOM_INDEX]
        self._zoom_index = DEFAULT_ZOOM_INDEX

        self._frame_item = None       # QGraphicsPixmapItem fuer Frame
        self._onion_prev_item = None  # voriges Frame, blau-getoent
        self._onion_next_item = None  # naechstes Frame, rot-getoent
        self._preview_item = None
        self._checker_item = None
        self._selection_items: list = []   # zwei Items: schwarz + gelb (Marching-Ants)
        self._grid_items: list = []
        self._tile_replica_items: list = []
        self._preview_pil: Optional[Image.Image] = None
        self._selection: Optional[tuple[int, int, int, int]] = None
        # Marching-Ants-Animation: laeuft nur waehrend eine Auswahl aktiv ist
        self._marquee_phase = 0
        self._marquee_timer = QTimer(self)
        self._marquee_timer.setInterval(120)
        self._marquee_timer.timeout.connect(self._advance_marquee)

        # Mouse-State
        self._panning = False
        self._pan_last = QPoint()

        self.rebuild_scene()

    # --- Zoom ---

    def zoom_index(self) -> int:
        return self._zoom_index

    def set_zoom_index(self, idx: int):
        idx = max(0, min(len(ZOOM_LEVELS) - 1, idx))
        self._zoom_index = idx
        self._zoom = ZOOM_LEVELS[idx]
        self._apply_transform()

    def zoom_in(self):  self.set_zoom_index(self._zoom_index + 1)
    def zoom_out(self): self.set_zoom_index(self._zoom_index - 1)
    def zoom_reset(self): self.set_zoom_index(DEFAULT_ZOOM_INDEX)

    def _apply_transform(self):
        # Wir benutzen NICHT QGraphicsView::scale, sondern halten die
        # Scene-Pixmaps in nativer Aufloesung (1 Pixmap-Pixel = 1 Sprite-Pixel
        # * _zoom) und lassen den View bei 1.0. Vorteil: Nearest-Neighbor
        # ist sicher (kein Smoothing) und Selection-/Grid-Items sind
        # in Bildschirm-Koordinaten.
        self.rebuild_scene()
        self.app.update_status()

    # --- Public API fuer Tools ---

    def invalidate_all(self):
        """Frame-Pixmap nach Pixel-Aenderung neu rendern."""
        self._render_frame_pixmap()
        if self._show_tile_preview:
            for it in self._tile_replica_items:
                self._scene.removeItem(it)
            self._tile_replica_items = []
            self._render_tile_replicas()

    def set_preview(self, pil: Optional[Image.Image]):
        self._preview_pil = pil
        self._render_frame_pixmap()

    def set_selection(self, x0, y0, x1, y1):
        self._selection = (x0, y0, x1, y1)
        self._render_selection()
        if not self._marquee_timer.isActive():
            self._marquee_timer.start()

    def clear_selection(self):
        self._selection = None
        self._marquee_timer.stop()
        self._render_selection()

    def _advance_marquee(self):
        # Defensiv: wenn die Selection irgendwie weggenommen wurde, ohne
        # dass `clear_selection()` lief, stoppen wir uns selbst. Vermeidet
        # einen still im Hintergrund tickenden Timer + die nutzlosen
        # _render_selection-Aufrufe.
        if self._selection is None:
            self._marquee_timer.stop()
            return
        self._marquee_phase = (self._marquee_phase + 1) % 8
        self._render_selection()

    def get_selection(self) -> Optional[tuple[int, int, int, int]]:
        if not self._selection:
            return None
        x0, y0, x1, y1 = self._selection
        return (min(x0, x1), min(y0, y1), max(x0, x1) + 1, max(y0, y1) + 1)

    def set_show_grid(self, on: bool):
        self._show_grid = bool(on)
        self._render_grid()

    def set_show_onion(self, on: bool):
        self._show_onion = bool(on)
        self._render_onion()

    def set_tile_preview(self, on: bool):
        """Toggle 3x3-Kachel-Vorschau. Im Tile-Modus wird das Sprite 9-mal
        (3 horizontal x 3 vertikal) gerendert, das mittlere Tile ist der
        echte Bearbeitungs-Bereich."""
        self._show_tile_preview = bool(on)
        self.rebuild_scene()

    # --- Scene-Aufbau ---

    def rebuild_scene(self):
        self._scene.clear()
        self._frame_item = None
        self._onion_prev_item = None
        self._onion_next_item = None
        self._preview_item = None
        self._checker_item = None
        self._selection_items = []
        self._tile_replica_items = []
        self._grid_items = []
        self._render_checker()
        self._render_onion()
        self._render_frame_pixmap()
        self._render_tile_replicas()
        self._render_grid()
        self._render_selection()
        # Scene-Rect: im Tile-Preview 3x3, sonst 1x1 (mit dem Sprite bei (0,0))
        doc = self.app.doc; z = self._zoom
        if self._show_tile_preview:
            self._scene.setSceneRect(-doc.width * z, -doc.height * z,
                                     doc.width * z * 3, doc.height * z * 3)
        else:
            self._scene.setSceneRect(0, 0, doc.width * z, doc.height * z)

    def _render_checker(self):
        doc = self.app.doc
        z = self._zoom
        w = doc.width * z; h = doc.height * z
        pix = QPixmap(w, h)
        p = QPainter(pix)
        p.setRenderHint(QPainter.Antialiasing, False)
        a = QColor(COLORS["checker_a"])
        b = QColor(COLORS["checker_b"])
        for cy in range(doc.height):
            for cx in range(doc.width):
                color = a if (cx + cy) % 2 == 0 else b
                p.fillRect(cx * z, cy * z, z, z, color)
        p.end()
        if self._checker_item is None:
            self._checker_item = self._scene.addPixmap(pix)
            self._checker_item.setZValue(0)
        else:
            self._checker_item.setPixmap(pix)

    def _render_onion(self):
        # Vorheriges + naechstes Frame als Onion-Skin -- klassische
        # Animation-Hilfe. Vorher = blauer Stich, nachher = roter Stich,
        # damit die Bewegungs-Richtung sofort klar ist.
        for attr in ("_onion_prev_item", "_onion_next_item"):
            it = getattr(self, attr, None)
            if it is not None:
                self._scene.removeItem(it)
                setattr(self, attr, None)
        if not self._show_onion or len(self.app.doc.frames) <= 1:
            return
        doc = self.app.doc
        z = self._zoom
        prev_idx = (doc.current_index - 1) % len(doc.frames)
        next_idx = (doc.current_index + 1) % len(doc.frames)

        def _tinted(idx, mode):
            """Farbiges, halbtransparentes Onion-Bild. mode in {'blue','red'}."""
            src = doc.frames[idx].pixels
            r, g, b, a = src.split()
            a = a.point(lambda v: int(v * 0.45))
            if mode == "blue":
                # Rot/Gruen zurueck, Blau bleibt -> kuehler Stich
                r = r.point(lambda v: int(v * 0.5))
                g = g.point(lambda v: int(v * 0.7))
            else:  # red
                b = b.point(lambda v: int(v * 0.5))
                g = g.point(lambda v: int(v * 0.7))
            tinted = Image.merge("RGBA", (r, g, b, a))
            scaled = tinted.resize((doc.width * z, doc.height * z),
                                    Image.NEAREST)
            return pil_to_qpixmap(scaled)

        prev_pix = _tinted(prev_idx, "blue")
        self._onion_prev_item = self._scene.addPixmap(prev_pix)
        self._onion_prev_item.setZValue(1)

        # Nur wenn das naechste Frame anders als das vorherige ist (bei
        # 2 Frames sind sie identisch) -- sonst doppeltes Rendering vermeiden.
        if next_idx != prev_idx:
            next_pix = _tinted(next_idx, "red")
            self._onion_next_item = self._scene.addPixmap(next_pix)
            self._onion_next_item.setZValue(1)

    def _render_frame_pixmap(self):
        doc = self.app.doc
        shown = self._preview_pil if self._preview_pil is not None else doc.current.pixels
        z = self._zoom
        scaled = shown.resize((doc.width * z, doc.height * z), Image.NEAREST)
        pix = pil_to_qpixmap(scaled)
        if self._frame_item is None:
            self._frame_item = self._scene.addPixmap(pix)
            self._frame_item.setZValue(2)
        else:
            self._frame_item.setPixmap(pix)

    def _render_tile_replicas(self):
        """Im Tile-Preview-Modus: zeichnet 8 transparente Replicas um den
        zentralen Sprite-Bereich. Das echte Sprite bleibt bei Scene (0,0),
        die Replicas sind Pixmap-Items an den 8 Nachbar-Positionen.
        Werden bei jedem invalidate_all neu gerendert -- das ist
        ausreichend schnell, weil die Pixmap eh schon im Speicher ist.
        """
        if not self._show_tile_preview:
            return
        doc = self.app.doc; z = self._zoom
        shown = self._preview_pil if self._preview_pil is not None else doc.current.pixels
        scaled = shown.resize((doc.width * z, doc.height * z), Image.NEAREST)
        pix = pil_to_qpixmap(scaled)
        for ox in (-doc.width * z, 0, doc.width * z):
            for oy in (-doc.height * z, 0, doc.height * z):
                if ox == 0 and oy == 0:
                    continue   # Mittel-Tile = self._frame_item
                item = self._scene.addPixmap(pix)
                item.setPos(ox, oy)
                item.setZValue(2)
                item.setOpacity(0.55)   # leicht abgedunkelt -- klar erkennbar als Vorschau
                self._tile_replica_items.append(item)

    def _render_grid(self):
        for it in self._grid_items:
            self._scene.removeItem(it)
        self._grid_items = []
        if not self._show_grid:
            return
        z = self._zoom
        if z < 6:
            return
        doc = self.app.doc
        thin = QPen(QColor(COLORS["grid"]), 0)
        thick = QPen(QColor(COLORS["grid_strong"]), 0)
        for gx in range(0, doc.width + 1):
            x = gx * z
            pen = thick if gx % 8 == 0 else thin
            line = self._scene.addLine(x, 0, x, doc.height * z, pen)
            line.setZValue(3)
            self._grid_items.append(line)
        for gy in range(0, doc.height + 1):
            y = gy * z
            pen = thick if gy % 8 == 0 else thin
            line = self._scene.addLine(0, y, doc.width * z, y, pen)
            line.setZValue(3)
            self._grid_items.append(line)

    def _render_selection(self):
        # Alte Marquee-Items entfernen
        for it in self._selection_items:
            self._scene.removeItem(it)
        self._selection_items = []
        if self._selection is None:
            return
        x0, y0, x1, y1 = self._selection
        lo_x = min(x0, x1); hi_x = max(x0, x1) + 1
        lo_y = min(y0, y1); hi_y = max(y0, y1) + 1
        z = self._zoom
        rect = QRectF(lo_x * z, lo_y * z, (hi_x - lo_x) * z, (hi_y - lo_y) * z)

        # Klassisches Marching-Ants-Pattern: schwarze 1-Pixel-Outline
        # darunter, gelbe gestrichelte Linie darueber mit animiertem
        # Dash-Offset. Die schwarze Linie sorgt fuer Kontrast auf hellen
        # wie dunklen Pixeln.
        pen_black = QPen(QColor(0, 0, 0), 1)
        item_black = self._scene.addRect(rect, pen_black)
        item_black.setZValue(10)
        self._selection_items.append(item_black)

        pen_yellow = QPen(QColor(COLORS["selection"]), 1, Qt.CustomDashLine)
        pen_yellow.setDashPattern([4, 4])
        pen_yellow.setDashOffset(self._marquee_phase)
        item_yellow = self._scene.addRect(rect, pen_yellow)
        item_yellow.setZValue(11)
        self._selection_items.append(item_yellow)

    # --- Mouse-Handling ---

    def _scene_to_pixel(self, scene_pos: QPointF) -> Optional[tuple[int, int]]:
        z = self._zoom
        x = int(scene_pos.x() // z)
        y = int(scene_pos.y() // z)
        # Im Tile-Preview-Modus: Klicks auf einen der Replica-Tiles werden
        # auf das echte Sprite mod (w, h) gemappt, damit der User in
        # beliebigem Tile arbeiten kann.
        if self._show_tile_preview:
            doc = self.app.doc
            x = x % doc.width
            y = y % doc.height
        return (x, y)

    def _clamp_to_sprite(self, x: int, y: int) -> tuple[int, int]:
        """Klemmt Koordinaten auf den Sprite-Bereich. Wird waehrend Drag
        verwendet, damit das Werkzeug am Rand weitermalt statt zu
        verschwinden, wenn der Cursor das Sprite verlaesst."""
        return (max(0, min(self.app.doc.width - 1, x)),
                max(0, min(self.app.doc.height - 1, y)))

    def mousePressEvent(self, event):
        if event.button() == Qt.MiddleButton:
            self._panning = True
            self._pan_last = event.pos()
            self.viewport().setCursor(Qt.ClosedHandCursor)
            event.accept()
            return
        scene_pos = self.mapToScene(event.pos())
        coord = self._scene_to_pixel(scene_pos)

        # Rechtsklick auf einen Pixel innerhalb der aktiven Auswahl ->
        # Kontextmenue. Das vermeidet das umstaendliche "M druecken,
        # neuen Marquee ziehen, Hotkey verwenden" -- klassisches OS-
        # Verhalten fuer aktive Selektionen.
        if (event.button() == Qt.RightButton and coord is not None
                and self._selection is not None):
            sel = self.get_selection()
            if (sel and sel[0] <= coord[0] < sel[2]
                    and sel[1] <= coord[1] < sel[3]):
                self.app.show_selection_context_menu(
                    event.globalPosition().toPoint())
                event.accept()
                return

        if coord is None:
            return
        # Werkzeuge nur starten, wenn der erste Klick im Sprite-Bereich liegt.
        # Sonst landen Selection-Marquees, Linien-Anker etc. ausserhalb des
        # Canvas und der User wundert sich ueber "geisterhafte" Rechtecke
        # neben dem eigentlichen Sprite.
        if not self.app.in_bounds(coord[0], coord[1]):
            return
        self._dragging = True
        self.app.tool.begin(self.app, coord[0], coord[1], event.button())

    def mouseMoveEvent(self, event):
        if self._panning:
            delta = event.pos() - self._pan_last
            self._pan_last = event.pos()
            h = self.horizontalScrollBar()
            v = self.verticalScrollBar()
            h.setValue(h.value() - delta.x())
            v.setValue(v.value() - delta.y())
            event.accept()
            return
        scene_pos = self.mapToScene(event.pos())
        coord = self._scene_to_pixel(scene_pos)
        # Statusleiste zeigt nur "echte" Hover-Pixel im Sprite-Bereich
        if coord is not None and self.app.in_bounds(coord[0], coord[1]):
            self.app.set_hover_pixel(coord)
        else:
            self.app.set_hover_pixel(None)
        # Waehrend eines Drags clampen wir die Koordinaten auf den
        # Sprite-Bereich -- so malt der Stift am Rand weiter und die
        # Selection-Marquee bleibt im Bild.
        if (event.buttons() & (Qt.LeftButton | Qt.RightButton)
                and getattr(self, "_dragging", False) and coord is not None):
            cx, cy = self._clamp_to_sprite(coord[0], coord[1])
            self.app.tool.move(self.app, cx, cy)

    def mouseReleaseEvent(self, event):
        if event.button() == Qt.MiddleButton and self._panning:
            self._panning = False
            self.viewport().setCursor(Qt.ArrowCursor)
            event.accept()
            return
        if not getattr(self, "_dragging", False):
            return
        self._dragging = False
        scene_pos = self.mapToScene(event.pos())
        coord = self._scene_to_pixel(scene_pos)
        if coord is None:
            return
        cx, cy = self._clamp_to_sprite(coord[0], coord[1])
        self.app.tool.end(self.app, cx, cy)

    def wheelEvent(self, event):
        # Mausrad ohne Modifier = Zoom (kein Pan, da das Sprite eh klein
        # ist und Zoom intuitiver). Mit Strg = horizontaler Scroll.
        if event.modifiers() & Qt.ControlModifier:
            super().wheelEvent(event)
            return
        if event.angleDelta().y() > 0:
            self.zoom_in()
        else:
            self.zoom_out()
        event.accept()

    def leaveEvent(self, event):
        self.app.set_hover_pixel(None)
        super().leaveEvent(event)


# ============================================================
# Color-Panel (FG/BG-Indikator + Slider + Palette)
# ============================================================

class _ColorBox(QFrame):
    """Anklickbares Quadrat das eine RGBA-Farbe anzeigt (mit Schachbrett
    fuer Alpha < 255)."""
    clicked = Signal()

    def __init__(self, size: int = 60):
        super().__init__()
        self._size = size
        self._color = (0, 0, 0, 255)
        self.setFixedSize(size, size)
        self.setCursor(Qt.PointingHandCursor)
        self.setFrameShape(QFrame.Box)
        self.setLineWidth(1)
        self.setStyleSheet(f"border: 1px solid {COLORS['border']};")

    def set_color(self, col):
        self._color = col
        self.update()

    def color(self):
        return self._color

    def mousePressEvent(self, event):
        if event.button() == Qt.LeftButton:
            self.clicked.emit()
        super().mousePressEvent(event)

    def paintEvent(self, event):
        super().paintEvent(event)
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing, False)
        # Schachbrett
        a = QColor(COLORS["checker_a"])
        b = QColor(COLORS["checker_b"])
        s = self._size
        for yy in range(0, s, 8):
            for xx in range(0, s, 8):
                color = a if (xx // 8 + yy // 8) % 2 == 0 else b
                p.fillRect(xx, yy, 8, 8, color)
        # Farbe
        r, g, bcol, alpha = self._color
        p.fillRect(2, 2, s - 4, s - 4, QColor(r, g, bcol, alpha))


class ColorPanel(QWidget):
    PAL_COLS = 8
    SWATCH = 26

    fg_changed = Signal(tuple)
    bg_changed = Signal(tuple)

    def __init__(self, app: "SpriteEditorWindow"):
        super().__init__()
        self.app = app
        self._suspend_signals = False

        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(8)

        # FG/BG-Indikator
        ind_row = QHBoxLayout()
        ind_row.setSpacing(4)
        ind_layout = QGridLayout()
        ind_layout.setContentsMargins(0, 0, 0, 0)
        ind_layout.setSpacing(0)
        # Two boxes, overlapping like classic Photoshop indicator
        self.fg_box = _ColorBox(60)
        self.bg_box = _ColorBox(60)
        # Container fuer ueberlappende Boxen
        ind_container = QFrame()
        ind_container.setFixedSize(96, 84)
        self.fg_box.setParent(ind_container); self.fg_box.move(0, 0)
        self.bg_box.setParent(ind_container); self.bg_box.move(32, 24)
        # FG vor BG anzeigen (raise)
        self.fg_box.raise_()
        # Labels
        fg_label = QLabel("FG", ind_container)
        fg_label.move(2, 44)
        fg_label.setStyleSheet(f"color: {COLORS['fg']}; font-weight: bold; "
                               f"font-size: 9pt; background: transparent;")
        bg_label = QLabel("BG", ind_container)
        bg_label.move(72, 70)
        bg_label.setStyleSheet(f"color: {COLORS['fg_muted']}; font-weight: bold; "
                               f"font-size: 9pt; background: transparent;")

        self.fg_box.clicked.connect(self._pick_fg)
        self.bg_box.clicked.connect(self._pick_bg)
        ind_row.addWidget(ind_container)

        # Swap-Button mit Icon
        swap_btn = QToolButton()
        swap_btn.setIcon(make_action_icon("swap", 22))
        swap_btn.setIconSize(QSize(22, 22))
        swap_btn.setToolTip("Farben tauschen (X)")
        swap_btn.setFixedSize(34, 34)
        swap_btn.clicked.connect(self.app.swap_colors)
        ind_row.addWidget(swap_btn, alignment=Qt.AlignVCenter)
        ind_row.addStretch()
        layout.addLayout(ind_row)

        # RGBA-Slider
        sliders_box = QGridLayout()
        sliders_box.setContentsMargins(0, 8, 0, 0)
        sliders_box.setHorizontalSpacing(6)
        sliders_box.setVerticalSpacing(4)
        self.sliders = {}
        self.value_labels = {}
        for row, (key, label) in enumerate([("r", "R"), ("g", "G"),
                                             ("b", "B"), ("a", "A")]):
            lbl = QLabel(label)
            lbl.setFixedWidth(14)
            lbl.setStyleSheet(f"color: {COLORS['fg']}; "
                              f"font-family: Consolas; font-weight: bold;")
            sliders_box.addWidget(lbl, row, 0)
            sl = QSlider(Qt.Horizontal)
            sl.setRange(0, 255)
            sl.valueChanged.connect(lambda v, k=key: self._on_slider(k, v))
            sliders_box.addWidget(sl, row, 1)
            v_lbl = QLabel("0")
            v_lbl.setFixedWidth(28)
            v_lbl.setStyleSheet(f"color: {COLORS['fg_muted']}; "
                                f"font-family: Consolas;")
            sliders_box.addWidget(v_lbl, row, 2)
            self.sliders[key] = sl
            self.value_labels[key] = v_lbl
        layout.addLayout(sliders_box)

        # Hex
        hex_row = QHBoxLayout()
        hex_lbl = QLabel("HEX  #")
        hex_lbl.setStyleSheet(f"color: {COLORS['fg_muted']}; "
                              f"font-family: Consolas; font-weight: bold;")
        hex_row.addWidget(hex_lbl)
        self.hex_edit = QLineEdit()
        self.hex_edit.setMaximumWidth(120)
        self.hex_edit.setPlaceholderText("rrggbb[aa]")
        self.hex_edit.editingFinished.connect(self._on_hex)
        hex_row.addWidget(self.hex_edit)
        hex_row.addStretch()
        layout.addLayout(hex_row)

        # Palette
        pal_lbl = QLabel("PALETTE")
        pal_lbl.setObjectName("sectionHeader")
        layout.addWidget(pal_lbl)

        pal_grid = QGridLayout()
        pal_grid.setSpacing(2)
        pal_grid.setContentsMargins(0, 0, 0, 0)
        for i, col in enumerate(self.app.palette):
            r, c = divmod(i, self.PAL_COLS)
            sw = _PaletteSwatch(col, self.SWATCH)
            sw.left_clicked.connect(lambda cc=col: self.app.set_fg(cc))
            sw.right_clicked.connect(lambda cc=col: self.app.set_bg(cc))
            pal_grid.addWidget(sw, r, c)
        layout.addLayout(pal_grid)

        # Recent-Colors (gefuellt sich auf, sobald der User Farben waehlt)
        self.recent_lbl = QLabel("ZULETZT VERWENDET")
        self.recent_lbl.setObjectName("sectionHeader")
        layout.addWidget(self.recent_lbl)
        self.recent_container = QWidget()
        self.recent_grid = QGridLayout(self.recent_container)
        self.recent_grid.setSpacing(2)
        self.recent_grid.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(self.recent_container)

        layout.addStretch()
        self.refresh()

    def _refresh_recent(self):
        # Bestehende Recent-Swatches entfernen
        while self.recent_grid.count():
            it = self.recent_grid.takeAt(0)
            w = it.widget()
            if w is not None:
                w.deleteLater()
        history = self.app.color_history
        if not history:
            self.recent_lbl.setVisible(False)
            self.recent_container.setVisible(False)
            return
        self.recent_lbl.setVisible(True)
        self.recent_container.setVisible(True)
        # 8 Swatches pro Reihe, 2 Reihen max -> 16 Stueck
        for i, col in enumerate(history):
            r, c = divmod(i, self.PAL_COLS)
            sw = _PaletteSwatch(col, self.SWATCH - 6)  # leicht kleiner als Palette
            sw.left_clicked.connect(lambda cc=col: self.app.set_fg(cc))
            sw.right_clicked.connect(lambda cc=col: self.app.set_bg(cc))
            self.recent_grid.addWidget(sw, r, c)

    def refresh(self):
        self.fg_box.set_color(self.app.fg)
        self.bg_box.set_color(self.app.bg)
        self._suspend_signals = True
        try:
            r, g, b, a = self.app.fg
            self.sliders["r"].setValue(r); self.value_labels["r"].setText(str(r))
            self.sliders["g"].setValue(g); self.value_labels["g"].setText(str(g))
            self.sliders["b"].setValue(b); self.value_labels["b"].setText(str(b))
            self.sliders["a"].setValue(a); self.value_labels["a"].setText(str(a))
            hexs = f"{r:02x}{g:02x}{b:02x}"
            if a != 255:
                hexs += f"{a:02x}"
            self.hex_edit.setText(hexs)
        finally:
            self._suspend_signals = False
        # Color-History updaten -- nur wenn das Widget schon gebaut ist
        if hasattr(self, "recent_grid"):
            self._refresh_recent()

    def _on_slider(self, key, val):
        if self._suspend_signals:
            return
        r, g, b, a = self.app.fg
        if   key == "r": r = val
        elif key == "g": g = val
        elif key == "b": b = val
        elif key == "a": a = val
        self.value_labels[key].setText(str(val))
        self.app.set_fg((r, g, b, a))

    def _on_hex(self):
        s = self.hex_edit.text().strip().lstrip("#")
        if len(s) not in (6, 8):
            self.refresh(); return
        try:
            r = int(s[0:2], 16); g = int(s[2:4], 16); b = int(s[4:6], 16)
            a = int(s[6:8], 16) if len(s) == 8 else 255
        except ValueError:
            self.refresh(); return
        self.app.set_fg((r, g, b, a))

    def _pick_fg(self):
        col = self.app.fg
        qc = QColorDialog.getColor(QColor(col[0], col[1], col[2], col[3]),
                                   self, "Vordergrundfarbe",
                                   QColorDialog.ShowAlphaChannel)
        if qc.isValid():
            self.app.set_fg((qc.red(), qc.green(), qc.blue(), qc.alpha()))

    def _pick_bg(self):
        col = self.app.bg
        qc = QColorDialog.getColor(QColor(col[0], col[1], col[2], col[3]),
                                   self, "Hintergrundfarbe",
                                   QColorDialog.ShowAlphaChannel)
        if qc.isValid():
            self.app.set_bg((qc.red(), qc.green(), qc.blue(), qc.alpha()))


class _PaletteSwatch(QFrame):
    left_clicked = Signal()
    right_clicked = Signal()

    def __init__(self, color, size):
        super().__init__()
        self._color = color
        self._size = size
        self.setFixedSize(size, size)
        self.setCursor(Qt.PointingHandCursor)

    def mousePressEvent(self, event):
        if event.button() == Qt.LeftButton:
            self.left_clicked.emit()
        elif event.button() == Qt.RightButton:
            self.right_clicked.emit()
        super().mousePressEvent(event)

    def paintEvent(self, event):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing, False)
        a = QColor(COLORS["checker_a"]); b = QColor(COLORS["checker_b"])
        s = self._size
        for yy in range(0, s, 4):
            for xx in range(0, s, 4):
                c = a if (xx // 4 + yy // 4) % 2 == 0 else b
                p.fillRect(xx, yy, 4, 4, c)
        r, g, bcol, alpha = self._color
        p.fillRect(0, 0, s, s, QColor(r, g, bcol, alpha))
        p.setPen(QPen(QColor(COLORS["border"]), 1))
        p.drawRect(0, 0, s - 1, s - 1)


# ============================================================
# Frames-Panel (Liste mit Thumbnails)
# ============================================================

class AssetBrowser(QWidget):
    """Scrollbarer Asset-Browser fuer das Projekt: zeigt alle .png und
    .gbsprite-Dateien aus den ueblichen Verzeichnissen
    (`examples/assets/`, `examples/`, `assets/`) als Thumbnails an.

    Doppelklick laedt das Asset im Editor. Refresh-Button rescannt das
    Dateisystem.
    """
    THUMB = 48

    SCAN_DIRS = (
        ("examples", "assets"),
        ("examples",),
        ("assets",),
    )

    def __init__(self, app: "SpriteEditorWindow"):
        super().__init__()
        self.app = app
        layout = QVBoxLayout(self)
        layout.setContentsMargins(8, 8, 8, 8)
        layout.setSpacing(6)

        # Header + Refresh-Button
        head = QHBoxLayout()
        title = QLabel("Doppelklick laedt das Asset")
        title.setStyleSheet(f"color: {COLORS['fg_muted']}; "
                             f"font-size: 10px;")
        head.addWidget(title)
        head.addStretch()
        refresh_btn = QToolButton()
        refresh_btn.setText("⟳")
        refresh_btn.setToolTip("Ordner neu scannen")
        refresh_btn.clicked.connect(self.refresh)
        head.addWidget(refresh_btn)
        layout.addLayout(head)

        self.path_lbl = QLabel("")
        self.path_lbl.setStyleSheet(f"color: {COLORS['fg_muted']}; "
                                     f"font-family: Consolas; font-size: 9px;")
        self.path_lbl.setWordWrap(True)
        layout.addWidget(self.path_lbl)

        from PySide6.QtWidgets import QListView
        self.list_widget = QListWidget()
        self.list_widget.setIconSize(QSize(self.THUMB, self.THUMB))
        self.list_widget.setViewMode(QListView.IconMode)
        self.list_widget.setMovement(QListView.Static)
        self.list_widget.setResizeMode(QListView.Adjust)
        self.list_widget.setUniformItemSizes(True)
        self.list_widget.setSpacing(6)
        self.list_widget.setSelectionMode(QListWidget.SingleSelection)
        self.list_widget.itemDoubleClicked.connect(self._on_double_click)
        layout.addWidget(self.list_widget, 1)

        self.refresh()

    def _scan(self) -> list[Path]:
        """Sammelt alle PNG/.gbsprite-Dateien aus den Standard-Asset-
        Verzeichnissen (rekursiv). Dedupliziert nach absolutem Pfad,
        sortiert nach Datei-Name."""
        root = self.app.project_root
        seen = set()
        out = []
        for parts in self.SCAN_DIRS:
            d = root.joinpath(*parts)
            if not d.is_dir():
                continue
            for ext in ("*.png", "*.gbsprite"):
                for p in d.rglob(ext):
                    if not p.is_file():
                        continue
                    rp = p.resolve()
                    if rp in seen:
                        continue
                    seen.add(rp)
                    out.append(p)
        return sorted(out, key=lambda p: (str(p.parent), p.name.lower()))

    def refresh(self):
        self.list_widget.clear()
        assets = self._scan()
        if not assets:
            item = QListWidgetItem("(keine Assets gefunden)")
            item.setFlags(Qt.NoItemFlags)
            self.list_widget.addItem(item)
            self.path_lbl.setText("")
            return
        # Status-Zeile zeigt, welche Verzeichnisse durchsucht wurden
        scanned = []
        for parts in self.SCAN_DIRS:
            d = self.app.project_root.joinpath(*parts)
            if d.is_dir():
                scanned.append("/".join(parts))
        self.path_lbl.setText(f"{len(assets)} Assets aus {', '.join(scanned)}")

        for path in assets:
            try:
                icon = QIcon(pil_to_qpixmap(self._make_thumb(path)))
            except Exception:
                icon = QIcon()
            # Display-Name: relativer Pfad ab project_root, falls kuerzer
            try:
                disp = str(path.relative_to(self.app.project_root))
            except ValueError:
                disp = path.name
            # Kuerzen wenn zu lang
            if len(disp) > 28:
                disp = "..." + disp[-25:]
            item = QListWidgetItem(icon, disp)
            item.setData(Qt.UserRole, str(path))
            item.setToolTip(str(path))
            self.list_widget.addItem(item)

    def _make_thumb(self, path: Path) -> Image.Image:
        # Erstes Frame als Vorschau (.gbsprite hat mehrere Frames)
        if path.suffix.lower() == ".gbsprite":
            doc = SpriteDoc.load_native(path)
            img = doc.frames[0].pixels
        else:
            img = Image.open(path).convert("RGBA")
        ts = self.THUMB
        ratio = min(ts / max(1, img.width), ts / max(1, img.height))
        new_w = max(1, int(img.width * ratio))
        new_h = max(1, int(img.height * ratio))
        small = img.resize((new_w, new_h), Image.NEAREST)
        # Schachbrett-Hintergrund
        bg = Image.new("RGBA", (ts, ts), (45, 45, 48, 255))
        d = ImageDraw.Draw(bg)
        for yy in range(0, ts, 4):
            for xx in range(0, ts, 4):
                if (xx // 4 + yy // 4) % 2 == 0:
                    d.rectangle([xx, yy, xx + 4, yy + 4],
                                 fill=(60, 60, 65, 255))
        bg.alpha_composite(small, ((ts - new_w) // 2, (ts - new_h) // 2))
        return bg

    def _on_double_click(self, item: QListWidgetItem):
        s = item.data(Qt.UserRole)
        if not s:
            return
        path = Path(s)
        if not path.exists():
            QMessageBox.warning(self, "Datei nicht gefunden", str(path))
            self.refresh()
            return
        if not self.app._confirm_dirty():
            return
        self.app._load_path(path)


class FramesPanel(QWidget):
    THUMB = 64

    def __init__(self, app: "SpriteEditorWindow"):
        super().__init__()
        self.app = app
        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(8)

        # Buttons-Reihe
        btn_row = QHBoxLayout()
        btn_row.setSpacing(4)
        for icon_name, tip, slot in [
            ("frame_add",  "Neues Frame",        lambda: self.app.action_frame_add(False)),
            ("frame_dup",  "Frame duplizieren",  lambda: self.app.action_frame_add(True)),
            ("frame_del",  "Frame loeschen",     self.app.action_frame_delete),
        ]:
            b = QToolButton()
            b.setIcon(make_action_icon(icon_name, 20))
            b.setIconSize(QSize(20, 20))
            b.setToolTip(tip)
            b.clicked.connect(slot)
            btn_row.addWidget(b)

        # Up/Down via Pfeil-Symbole (Standard-Style-Icons reichen)
        up_btn = QToolButton(); up_btn.setText("▲")
        up_btn.setToolTip("Frame nach oben")
        up_btn.clicked.connect(lambda: self.app.action_frame_move(-1))
        btn_row.addWidget(up_btn)
        dn_btn = QToolButton(); dn_btn.setText("▼")
        dn_btn.setToolTip("Frame nach unten")
        dn_btn.clicked.connect(lambda: self.app.action_frame_move(+1))
        btn_row.addWidget(dn_btn)

        # Animation-Preview Button
        play_btn = QToolButton()
        play_btn.setIcon(make_action_icon("play", 20))
        play_btn.setIconSize(QSize(20, 20))
        play_btn.setToolTip("Animation-Vorschau (Strg+P)")
        play_btn.clicked.connect(self.app.action_play_animation)
        btn_row.addWidget(play_btn)

        btn_row.addStretch()
        layout.addLayout(btn_row)

        # Frame-Liste mit DnD-Reorder
        from PySide6.QtWidgets import QAbstractItemView
        self.list_widget = QListWidget()
        self.list_widget.setIconSize(QSize(self.THUMB, self.THUMB))
        # Internal-Move = User darf Items innerhalb der Liste umsortieren.
        # Qt verschiebt das Item visuell selbst; rowsMoved-Signal triggert
        # unseren Sync auf doc.frames.
        self.list_widget.setDragDropMode(QAbstractItemView.InternalMove)
        self.list_widget.setDefaultDropAction(Qt.MoveAction)
        self.list_widget.setSelectionMode(QListWidget.SingleSelection)
        self.list_widget.itemSelectionChanged.connect(self._on_list_select)
        self.list_widget.model().rowsMoved.connect(self._on_rows_moved)
        # Rechtsklick-Kontextmenue auf einem Frame
        self.list_widget.setContextMenuPolicy(Qt.CustomContextMenu)
        self.list_widget.customContextMenuRequested.connect(
            self._show_frame_context_menu)
        layout.addWidget(self.list_widget, 1)

        # Onion-Skin
        self.onion_check = QCheckBox("Onion-Skin")
        self.onion_check.toggled.connect(self.app.set_onion)
        layout.addWidget(self.onion_check)

    def refresh(self):
        # Anims-Panel mitziehen: Frame-Ops verschieben Bereiche (document.py).
        if hasattr(self.app, "anims_panel"):
            self.app.anims_panel.refresh()
        # Komplettes Rebuild: bei <50 Frames vernachlaessigbar.
        self.list_widget.blockSignals(True)
        self.list_widget.clear()
        for i, frame in enumerate(self.app.doc.frames):
            icon = self._make_thumb_icon(frame.pixels)
            label = f"#{i:02d}  {frame.name}" if frame.name else f"#{i:02d}"
            item = QListWidgetItem(icon, label)
            item.setData(Qt.UserRole, i)
            self.list_widget.addItem(item)
        self.list_widget.setCurrentRow(self.app.doc.current_index)
        self.list_widget.blockSignals(False)

    def _on_list_select(self):
        row = self.list_widget.currentRow()
        if row < 0:
            return
        if row != self.app.doc.current_index:
            self.app.action_frame_select(row)

    def _on_rows_moved(self, _parent, start, _end, _dest_parent, dst):
        """User hat ein Frame-Thumbnail per Drag verschoben.

        Qt's rowsMoved-Konvention: dst ist der Insertions-Index VOR der
        Entfernung des Source-Items. Ergo: bei Bewegung nach unten muss
        dst um 1 dekrementiert werden, um die echte neue Position zu
        bekommen.
        """
        src = start
        if dst > src:
            dst -= 1
        if src == dst:
            return
        frames = self.app.doc.frames
        if not (0 <= src < len(frames) and 0 <= dst < len(frames)):
            return
        f = frames.pop(src)
        frames.insert(dst, f)
        # current_index nachziehen
        cur = self.app.doc.current_index
        if cur == src:
            new_cur = dst
        elif src < cur <= dst:
            new_cur = cur - 1
        elif dst <= cur < src:
            new_cur = cur + 1
        else:
            new_cur = cur
        self.app.doc.current_index = new_cur
        self.app.doc.dirty = True
        # Item-Texte (#00, #01...) neu nummerieren -- Namen mitfuehren
        for i in range(self.list_widget.count()):
            it = self.list_widget.item(i)
            if it is not None:
                nm = frames[i].name if i < len(frames) else ""
                it.setText(f"#{i:02d}  {nm}" if nm else f"#{i:02d}")
        # Currently selected row syncen ohne erneut _on_list_select zu
        # triggern (sonst doppelter action_frame_select-Call).
        self.list_widget.blockSignals(True)
        self.list_widget.setCurrentRow(new_cur)
        self.list_widget.blockSignals(False)
        self.app.canvas.invalidate_all()
        self.app.update_status()

    def _show_frame_context_menu(self, pos):
        item = self.list_widget.itemAt(pos)
        if item is None:
            return
        idx = self.list_widget.row(item)

        menu = QMenu(self)
        jump = QAction("Hierher springen", self)
        jump.triggered.connect(lambda: self.app.action_frame_select(idx))
        menu.addAction(jump)
        menu.addSeparator()

        rename = QAction("Umbenennen...", self)
        rename.triggered.connect(lambda: self._rename_at(idx))
        menu.addAction(rename)

        dup = QAction("Frame duplizieren", self)
        dup.triggered.connect(lambda: self._dup_at(idx))
        menu.addAction(dup)

        paste_frame = QAction("Aus Zwischenablage als Frame", self)
        paste_frame.triggered.connect(self.app.action_paste_as_frame)
        menu.addAction(paste_frame)

        before = QAction("Davor leeres Frame einfuegen", self)
        before.triggered.connect(lambda: self._insert_blank(idx, after=False))
        menu.addAction(before)

        after = QAction("Danach leeres Frame einfuegen", self)
        after.triggered.connect(lambda: self._insert_blank(idx, after=True))
        menu.addAction(after)

        menu.addSeparator()

        cur_dur = self.app.doc.frames[idx].duration_ms
        dur_act = QAction(f"Dauer setzen... ({cur_dur} ms)", self)
        dur_act.triggered.connect(lambda: self._set_duration_at(idx))
        menu.addAction(dur_act)

        dur_all = QAction("Dauer fuer alle Frames setzen...", self)
        dur_all.triggered.connect(self._set_duration_all)
        menu.addAction(dur_all)

        menu.addSeparator()

        del_act = QAction("Frame loeschen", self)
        del_act.setEnabled(len(self.app.doc.frames) > 1)
        del_act.triggered.connect(lambda: self._delete_at(idx))
        menu.addAction(del_act)

        menu.exec(self.list_widget.viewport().mapToGlobal(pos))

    def _rename_at(self, idx: int):
        """Setzt/loescht den Namen eines Frames -> wird im Atlas-Export als
        Sprite-ID genutzt (statt <prefix>_<idx>)."""
        cur = self.app.doc.frames[idx].name
        val, ok = QInputDialog.getText(
            self, "Frame umbenennen",
            f"Frame {idx} -- Name (leer = nummeriert):", text=cur)
        if not ok:
            return
        self.app.doc.frames[idx].name = val.strip()
        self.app.doc.dirty = True
        self.refresh()
        self.app.update_status()

    def _set_duration_at(self, idx: int):
        cur = self.app.doc.frames[idx].duration_ms
        val, ok = QInputDialog.getInt(
            self, "Frame-Dauer",
            f"Frame {idx} -- Dauer in ms:", cur, 20, 5000, 5)
        if not ok:
            return
        self.app.doc.frames[idx].duration_ms = val
        self.app.doc.dirty = True
        self.app.update_status()

    def _set_duration_all(self):
        # Default = aktueller Wert des current Frames
        cur = self.app.doc.current.duration_ms
        val, ok = QInputDialog.getInt(
            self, "Frame-Dauer",
            f"Dauer in ms fuer alle {len(self.app.doc.frames)} Frames:",
            cur, 20, 5000, 5)
        if not ok:
            return
        for f in self.app.doc.frames:
            f.duration_ms = val
        self.app.doc.dirty = True
        self.app.update_status()

    def _dup_at(self, idx: int):
        """Dupliziert Frame an idx -- bewegt zuerst den current_index dorthin
        und ruft dann action_frame_add(copy_current=True), damit der
        existierende Code-Pfad genutzt wird."""
        self.app.doc.current_index = idx
        self.app.action_frame_add(True)

    def _insert_blank(self, idx: int, after: bool):
        """Fuegt ein leeres Frame vor/nach idx ein."""
        target = idx + 1 if after else idx
        new_img = Image.new("RGBA",
                             (self.app.doc.width, self.app.doc.height),
                             (0, 0, 0, 0))
        self.app.doc.frames.insert(target, Frame(pixels=new_img))
        # current_index nachpflegen
        if self.app.doc.current_index >= target:
            self.app.doc.current_index += 1
        # Auf das neue Frame springen, damit der User es direkt bearbeitet
        self.app.doc.current_index = target
        self.app.doc.dirty = True
        self.refresh()
        self.app.canvas.invalidate_all()
        self.app.update_status()

    def _delete_at(self, idx: int):
        """Loescht Frame an idx -- nutzt action_frame_delete nach dem
        Setzen des current_index."""
        if len(self.app.doc.frames) <= 1:
            return
        self.app.doc.current_index = idx
        self.app.action_frame_delete()

    def _make_thumb_icon(self, src: Image.Image) -> QIcon:
        ts = self.THUMB
        ratio = min(ts / src.width, ts / src.height)
        new_w = max(1, int(src.width * ratio))
        new_h = max(1, int(src.height * ratio))
        small = src.resize((new_w, new_h), Image.NEAREST)

        bg = Image.new("RGBA", (ts, ts), (45, 45, 48, 255))
        d = ImageDraw.Draw(bg)
        for yy in range(0, ts, 4):
            for xx in range(0, ts, 4):
                if (xx // 4 + yy // 4) % 2 == 0:
                    d.rectangle([xx, yy, xx + 4, yy + 4], fill=(60, 60, 65, 255))
        bg.alpha_composite(small, ((ts - new_w) // 2, (ts - new_h) // 2))
        return QIcon(pil_to_qpixmap(bg))


# ============================================================
# Animation-Preview
# ============================================================

class AnimRangeDialog(QDialog):
    """Einen Anim-Bereich anlegen/bearbeiten: Name, first..last, FPS.
    'FPS aus Dauern' uebernimmt den Vorschlag aus den echten Frame-Dauern."""

    def __init__(self, parent, doc: SpriteDoc, anim: Anim):
        super().__init__(parent)
        self.setWindowTitle("Animations-Bereich")
        self._doc = doc
        n = len(doc.frames)
        lay = QVBoxLayout(self)

        row1 = QHBoxLayout()
        row1.addWidget(QLabel("Name:"))
        self.name_edit = QLineEdit(anim.name)
        self.name_edit.setPlaceholderText("z.B. walk")
        row1.addWidget(self.name_edit, 1)
        lay.addLayout(row1)

        row2 = QHBoxLayout()
        row2.addWidget(QLabel("Frames:"))
        self.first_spin = QSpinBox(); self.first_spin.setRange(0, n - 1)
        self.first_spin.setValue(max(0, min(anim.first, n - 1)))
        row2.addWidget(self.first_spin)
        row2.addWidget(QLabel("bis"))
        self.last_spin = QSpinBox(); self.last_spin.setRange(0, n - 1)
        self.last_spin.setValue(max(0, min(anim.last, n - 1)))
        row2.addWidget(self.last_spin)
        row2.addStretch()
        lay.addLayout(row2)

        row3 = QHBoxLayout()
        row3.addWidget(QLabel("FPS:"))
        self.fps_spin = QSpinBox(); self.fps_spin.setRange(1, 60)
        self.fps_spin.setValue(max(1, min(60, anim.fps)))
        row3.addWidget(self.fps_spin)
        from_dur = QPushButton("aus Frame-Dauern")
        from_dur.setToolTip("FPS-Vorschlag aus den Dauern der Frames im Bereich")
        from_dur.clicked.connect(self._fps_from_durations)
        row3.addWidget(from_dur)
        row3.addStretch()
        lay.addLayout(row3)

        buttons = QDialogButtonBox(QDialogButtonBox.Ok | QDialogButtonBox.Cancel)
        buttons.accepted.connect(self.accept)
        buttons.rejected.connect(self.reject)
        lay.addWidget(buttons)

    def _fps_from_durations(self):
        self.fps_spin.setValue(self._doc.anim_fps_suggestion(
            self.first_spin.value(), self.last_spin.value()))

    @staticmethod
    def edit(parent, doc: SpriteDoc, anim: Anim) -> Optional[Anim]:
        dlg = AnimRangeDialog(parent, doc, anim)
        if dlg.exec() != QDialog.Accepted:
            return None
        name = dlg.name_edit.text().strip()
        if not name:
            return None
        first = dlg.first_spin.value()
        last = dlg.last_spin.value()
        if last < first:
            first, last = last, first
        return Anim(name=name, first=first, last=last, fps=dlg.fps_spin.value())


class AnimsPanel(QWidget):
    """Benannte Animations-Bereiche (das SPRITE_ADD_ANIM-Aequivalent des
    Editors). Die Bereiche leben in doc.anims (persistiert in .gbsprite v4)
    und speisen GB-Code-Export, .gbanim-Export und den Sprite-Test."""

    def __init__(self, app: "SpriteEditorWindow"):
        super().__init__()
        self.app = app
        layout = QVBoxLayout(self)
        layout.setContentsMargins(6, 6, 6, 6)

        btn_row = QHBoxLayout()
        for text, tip, slot in (
            ("+", "Animation hinzufuegen (Bereich = aktuelles Frame)", self._add),
            ("✎", "Animation bearbeiten (auch Doppelklick)", self._edit),
            ("−", "Animation loeschen", self._delete),
        ):
            b = QToolButton(); b.setText(text); b.setToolTip(tip)
            b.clicked.connect(slot)
            btn_row.addWidget(b)
        btn_row.addStretch()
        layout.addLayout(btn_row)

        self.list_widget = QListWidget()
        self.list_widget.itemDoubleClicked.connect(lambda _i: self._edit())
        layout.addWidget(self.list_widget, 1)

        hint = QLabel("Bereiche landen im GB-Code-\nund .gbanim-Export.")
        hint.setStyleSheet("color: #7a8190; font-size: 8pt;")
        layout.addWidget(hint)

    def refresh(self):
        self.list_widget.clear()
        for a in self.app.doc.anims:
            self.list_widget.addItem(f"{a.name}    {a.first}\u2013{a.last}    {a.fps} fps")

    def _selected(self) -> int:
        row = self.list_widget.currentRow()
        return row if 0 <= row < len(self.app.doc.anims) else -1

    def _add(self):
        doc = self.app.doc
        cur = doc.current_index
        a = AnimRangeDialog.edit(self, doc,
                                 Anim("", cur, cur, doc.anim_fps_suggestion(cur, cur)))
        if a is not None:
            doc.anims.append(a)
            doc.dirty = True
            self.refresh()
            self.app.update_status()

    def _edit(self):
        i = self._selected()
        if i < 0:
            return
        doc = self.app.doc
        a = AnimRangeDialog.edit(self, doc, doc.anims[i])
        if a is not None:
            doc.anims[i] = a
            doc.dirty = True
            self.refresh()
            self.app.update_status()

    def _delete(self):
        i = self._selected()
        if i < 0:
            return
        del self.app.doc.anims[i]
        self.app.doc.dirty = True
        self.refresh()
        self.app.update_status()


class AnimationPreview(QDialog):
    def __init__(self, parent, frames: list[Frame], sprite_w: int, sprite_h: int):
        super().__init__(parent)
        self.setWindowTitle("Animation-Vorschau")
        self.setModal(False)
        self._frames = frames
        self._sw = sprite_w; self._sh = sprite_h
        self._idx = 0
        self._playing = True
        self._fps = 8
        # Wenn die Frames unterschiedliche Dauern haben, ist es sinnvoll
        # diese zu respektieren; sonst nutzt der Slider den globalen FPS.
        durations = {f.duration_ms for f in frames}
        self._respect_frame_duration = len(durations) > 1

        # Skalierung so dass das Bild ca. 256px gross ist
        target = 256
        self._scale = max(2, target // max(sprite_w, sprite_h))
        cw = sprite_w * self._scale; ch = sprite_h * self._scale

        layout = QVBoxLayout(self)
        layout.setContentsMargins(20, 20, 20, 20)

        self.label = QLabel()
        self.label.setFixedSize(cw, ch)
        self.label.setStyleSheet(
            f"background: {COLORS['bg']}; "
            f"border: 1px solid {COLORS['border']};"
        )
        self.label.setAlignment(Qt.AlignCenter)
        layout.addWidget(self.label, alignment=Qt.AlignCenter)

        self.idx_label = QLabel("")
        self.idx_label.setAlignment(Qt.AlignCenter)
        self.idx_label.setStyleSheet(
            f"color: {COLORS['fg_muted']}; font-family: Consolas;"
        )
        layout.addWidget(self.idx_label)

        # FPS-Slider
        fps_row = QHBoxLayout()
        fps_lbl = QLabel("FPS")
        fps_lbl.setFixedWidth(30)
        fps_row.addWidget(fps_lbl)
        self.fps_slider = QSlider(Qt.Horizontal)
        self.fps_slider.setRange(1, 30)
        self.fps_slider.setValue(self._fps)
        self.fps_slider.valueChanged.connect(self._on_fps)
        fps_row.addWidget(self.fps_slider)
        self.fps_value = QLabel(f"{self._fps}")
        self.fps_value.setFixedWidth(30)
        self.fps_value.setStyleSheet(f"color: {COLORS['fg_muted']}; "
                                      f"font-family: Consolas;")
        fps_row.addWidget(self.fps_value)
        layout.addLayout(fps_row)

        # Buttons
        btn_row = QHBoxLayout()
        btn_row.addStretch()
        self.btn_play = QPushButton("Pause")
        self.btn_play.setObjectName("primary")
        self.btn_play.clicked.connect(self._toggle_play)
        btn_row.addWidget(self.btn_play)
        self.btn_back = QPushButton("◀")
        self.btn_back.clicked.connect(self._step_back)
        btn_row.addWidget(self.btn_back)
        self.btn_fwd = QPushButton("▶")
        self.btn_fwd.clicked.connect(self._step_forward)
        btn_row.addWidget(self.btn_fwd)
        btn_row.addStretch()
        layout.addLayout(btn_row)

        # Hotkeys
        QShortcut(QKeySequence("Space"), self, activated=self._toggle_play)
        QShortcut(QKeySequence("Escape"), self, activated=self.close)

        # Timer
        self._timer = QTimer(self)
        self._timer.timeout.connect(self._tick)
        self._render()
        self._restart_timer()
        # Sicherstellen dass das Fenster nach vorn kommt -- ohne diesen
        # Tick erscheint der Dialog auf Windows oft hinter dem Editor.
        QTimer.singleShot(50, self._raise_to_front)

    def _raise_to_front(self):
        try:
            self.raise_()
            self.activateWindow()
        except Exception:
            pass

    def _restart_timer(self):
        # Bei FPS-Override: gleiche Dauer fuer alle. Sonst die Frame-
        # spezifische duration_ms (mit Speed-Multiplier wenn fps != default).
        # Pragmatisch hier: FPS-Slider ist immer global.
        delay = max(20, int(1000 / self._fps))
        self._timer.start(delay)

    def _on_fps(self, v):
        self._fps = max(1, int(v))
        self.fps_value.setText(str(self._fps))
        self._restart_timer()

    def _tick(self):
        if not self._playing or len(self._frames) <= 1:
            return
        self._idx = (self._idx + 1) % len(self._frames)
        self._render()
        # Bei pro-Frame-Dauer: timer fuer NAECHSTES Frame neu setzen
        # (aktueller Tick hat das Frame ja schon gerendert)
        if self._respect_frame_duration:
            next_dur = self._frames[self._idx].duration_ms
            self._timer.setInterval(max(20, int(next_dur)))

    def _toggle_play(self):
        self._playing = not self._playing
        self.btn_play.setText("Pause" if self._playing else "Play")
        self.btn_play.setObjectName("primary" if self._playing else "success")
        # Stylesheet neu anwenden
        self.btn_play.style().polish(self.btn_play)

    def _step_back(self):
        self._playing = False
        self.btn_play.setText("Play")
        self._idx = (self._idx - 1) % len(self._frames)
        self._render()

    def _step_forward(self):
        self._playing = False
        self.btn_play.setText("Play")
        self._idx = (self._idx + 1) % len(self._frames)
        self._render()

    def _render(self):
        frame = self._frames[self._idx]
        scaled = frame.pixels.resize(
            (self._sw * self._scale, self._sh * self._scale), Image.NEAREST,
        )
        # Schachbrett-Hintergrund
        bg = Image.new("RGBA", scaled.size, (0, 0, 0, 0))
        d = ImageDraw.Draw(bg)
        for yy in range(0, bg.height, 8):
            for xx in range(0, bg.width, 8):
                if (xx // 8 + yy // 8) % 2 == 0:
                    d.rectangle([xx, yy, xx + 8, yy + 8],
                                fill=(42, 42, 42, 255))
                else:
                    d.rectangle([xx, yy, xx + 8, yy + 8],
                                fill=(58, 58, 58, 255))
        bg.alpha_composite(scaled)
        self.label.setPixmap(pil_to_qpixmap(bg))
        self.idx_label.setText(f"Frame {self._idx + 1} / {len(self._frames)}")


# ============================================================
# Main Window
# ============================================================

class SpriteEditorWindow(QMainWindow):
    """Haupt-Fenster des Sprite-Editors (PySide6)."""

    TOOLS = [
        ("pencil",      "Stift (B)",                   "B"),
        ("eraser",      "Radierer (E)",                "E"),
        ("spray",       "Spray (Y)",                   "Y"),
        ("bucket",      "Eimer (G)",                   "G"),
        ("line",        "Linie (L)",                   "L"),
        ("rect",        "Rechteck (R)",                "R"),
        ("rect_fill",   "Rechteck gefuellt (Shift+R)", "Shift+R"),
        ("ellipse",     "Ellipse (O)",                 "O"),
        ("ellipse_fill","Ellipse gefuellt (Shift+O)",  "Shift+O"),
        ("eyedrop",     "Pipette (I)",                 "I"),
        ("select",      "Auswahl (M)",                 "M"),
        ("magic_wand",  "Magic Wand (W)",              "W"),
        ("move",        "Verschieben (V)",             "V"),
    ]

    def __init__(self, project_root: Path, initial_file: Optional[Path] = None):
        super().__init__()
        self.project_root = project_root

        # State
        self.doc = SpriteDoc(*DEFAULT_DOC_SIZE)
        self.fg = (0, 0, 0, 255)
        self.bg = (255, 255, 255, 255)
        self.palette = list(DEFAULT_PALETTE)
        self.tool_name = "pencil"
        self._previous_tool_name: Optional[str] = None
        self.brush_size = 1
        self.symmetry_mode = "none"   # "none" | "x" | "y" | "both"
        self._clipboard_pil: Optional[Image.Image] = None
        self.color_history: list[tuple] = []   # zuletzt verwendete Farben
        self.COLOR_HISTORY_MAX = 16
        self._tools = {
            "pencil":       PencilTool(),
            "eraser":       EraserTool(),
            "spray":        SprayTool(),
            "bucket":       BucketTool(),
            "line":         LineTool(),
            "rect":         RectTool(filled=False),
            "rect_fill":    RectTool(filled=True),
            "ellipse":      EllipseTool(filled=False),
            "ellipse_fill": EllipseTool(filled=True),
            "eyedrop":      EyedropperTool(),
            "select":       SelectTool(),
            "magic_wand":   MagicWandTool(),
            "move":         MoveTool(),
        }

        self.setWindowTitle("GameBasic - Sprite-Editor")
        self.resize(1480, 920)
        self.setAcceptDrops(True)   # PNG / .gbsprite per Drag-and-Drop oeffnen
        self._set_window_icon()

        self._build_actions()
        self._build_menu()
        self._build_top_toolbar()
        self._build_left_toolbar()
        self._build_central()
        self._build_dock_panels()
        self._build_statusbar()

        # File-Watcher: erkennt externe Aenderungen an der aktuellen
        # Datei (z.B. von einem anderen Tool, git pull). Wir warnen den
        # User dann und bieten Reload an.
        self._fs_watcher = QFileSystemWatcher(self)
        self._fs_watcher.fileChanged.connect(self._on_file_changed)
        self._suppress_watcher_until = 0.0   # Timestamp; eigene Saves nicht als externe Aenderung melden

        # Auto-Backup-Timer: alle 60s einen Snapshot schreiben, wenn
        # doc.dirty. Schuetzt vor Crashes/Stromausfall ohne dass der
        # User explizit speichern muss.
        self._backup_timer = QTimer(self)
        self._backup_timer.setInterval(60_000)
        self._backup_timer.timeout.connect(self._auto_backup)
        self._backup_timer.start()
        # Beim Start nach einem Auto-Backup ohne entsprechende Original-
        # Datei suchen -- z.B. nach einem Crash mit nicht-gespeichertem
        # Sprite. Falls vorhanden, dem User Restore anbieten.
        QTimer.singleShot(300, self._maybe_offer_restore)

        if initial_file is not None and Path(initial_file).exists():
            self._load_path(Path(initial_file))

        self.update_tool_buttons()
        self.color_panel.refresh()
        self.frames_panel.refresh()
        self.update_status()
        self.canvas.rebuild_scene()

    @property
    def tool(self) -> Tool:
        return self._tools[self.tool_name]

    # --- Recent files ---
    # Persistiert in einem JSON im Home-Verzeichnis, damit die Liste
    # zwischen verschiedenen Projekt-Roots gleich bleibt.

    _RECENT_MAX = 10

    @staticmethod
    def _recent_path() -> Path:
        try:
            d = Path.home() / ".gamebasic"
            d.mkdir(exist_ok=True)
            return d / "recent_sprites.json"
        except Exception:
            return Path.home() / ".gamebasic_sprites_recent.json"

    @classmethod
    def _load_recent(cls) -> list[str]:
        p = cls._recent_path()
        if not p.exists():
            return []
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
            if isinstance(data, list):
                return [str(x) for x in data][:cls._RECENT_MAX]
        except Exception:
            pass
        return []

    @classmethod
    def _save_recent(cls, paths: list[str]):
        try:
            cls._recent_path().write_text(
                json.dumps(paths, indent=2), encoding="utf-8")
        except Exception:
            pass

    def _add_recent(self, path: Path):
        s = str(path.resolve())
        paths = self._load_recent()
        if s in paths:
            paths.remove(s)
        paths.insert(0, s)
        paths = paths[:self._RECENT_MAX]
        self._save_recent(paths)
        self._refresh_recent_menu()

    def _refresh_recent_menu(self):
        if not hasattr(self, "menu_recent"):
            return
        self.menu_recent.clear()
        paths = [p for p in self._load_recent() if Path(p).exists()]
        if not paths:
            a = QAction("(noch keine)", self)
            a.setEnabled(False)
            self.menu_recent.addAction(a)
            return
        for s in paths:
            p = Path(s)
            a = QAction(p.name, self)
            a.setToolTip(str(p))
            a.triggered.connect(lambda _ck=False, pth=p: self._open_recent(pth))
            self.menu_recent.addAction(a)
        self.menu_recent.addSeparator()
        clear_act = QAction("Liste leeren", self)
        clear_act.triggered.connect(self._clear_recent)
        self.menu_recent.addAction(clear_act)

    def _open_recent(self, path: Path):
        if not path.exists():
            QMessageBox.information(
                self, "Datei nicht gefunden",
                f"{path}\nwird aus der Liste entfernt.")
            paths = [p for p in self._load_recent() if p != str(path.resolve())]
            self._save_recent(paths)
            self._refresh_recent_menu()
            return
        if not self._confirm_dirty():
            return
        self._load_path(path)

    def _clear_recent(self):
        self._save_recent([])
        self._refresh_recent_menu()

    def _set_window_icon(self):
        # Logo aus dem branding-Modul, falls vorhanden
        try:
            from . import branding
            if branding.is_available():
                pil = Image.open(branding._logo_path()).convert("RGBA")
                # Smart-Square-Crop wie in branding.py
                pil = branding._smart_square_crop(pil)
                pil = pil.resize((64, 64), Image.LANCZOS)
                self.setWindowIcon(QIcon(pil_to_qpixmap(pil)))
        except Exception:
            pass

    # --- Actions / Menu / Toolbars ---

    def _build_actions(self):
        """Zentrale Definition aller QAction-Objekte (mit Hotkeys), wird
        vom Menu, der Toolbar UND den Tastenkuerzeln gemeinsam genutzt --
        ein QAction kann an mehreren Stellen verwendet werden."""
        def A(text, shortcut=None, slot=None, icon=None, checkable=False):
            act = QAction(text, self)
            if shortcut:
                act.setShortcut(QKeySequence(shortcut))
            if icon:
                act.setIcon(icon)
            if slot:
                act.triggered.connect(slot)
            if checkable:
                act.setCheckable(True)
            return act

        self.act_new      = A("Neu...",            "Ctrl+N", self.action_new,        make_action_icon("new"))
        self.act_open     = A("Oeffnen...",        "Ctrl+O", self.action_open,       make_action_icon("open"))
        self.act_save     = A("Speichern",         "Ctrl+S", self.action_save,       make_action_icon("save"))
        self.act_save_as  = A("Speichern unter...","Ctrl+Shift+S", self.action_save_as)
        self.act_export   = A("Sheet-PNG exportieren", "Ctrl+E", self.action_export_sheet, make_action_icon("export"))
        self.act_export_atlas = A("Sprite-Atlas exportieren (PNG + JSON)...",
                                   "Ctrl+Shift+E", self.action_export_atlas)
        self.act_export_gbanim = A("Animations-FSM exportieren (.gbanim)...",
                                    None, self.action_export_gbanim)
        self.act_export_gif = A("Animation als GIF...", "Ctrl+G", self.action_export_gif)
        self.act_export_frame = A("Frame als PNG exportieren", None, self.action_export_frame_png)
        self.act_reload   = A("Von Disk neu laden", "F5", self.action_reload_from_disk)
        self.act_close    = A("Schliessen", "Ctrl+W", self.close)

        self.act_undo     = A("Rueckgaengig", "Ctrl+Z", self.action_undo, make_action_icon("undo"))
        self.act_redo     = A("Wiederholen",  "Ctrl+Y", self.action_redo, make_action_icon("redo"))
        self.act_redo2    = A("Wiederholen2", "Ctrl+Shift+Z", self.action_redo)
        self.addAction(self.act_redo2)
        self.act_cut      = A("Auswahl ausschneiden", "Ctrl+X", self.action_cut, make_action_icon("cut"))
        self.act_copy     = A("Auswahl kopieren",     "Ctrl+C", self.action_copy, make_action_icon("copy"))
        self.act_paste    = A("Einfuegen",            "Ctrl+V", self.action_paste, make_action_icon("paste"))
        self.act_paste_frame = A("Als neues Frame einfuegen", "Ctrl+Shift+V", self.action_paste_as_frame, make_action_icon("paste"))
        self.act_clear    = A("Frame leeren",        None, self.action_clear_frame)
        self.act_del_sel  = A("Auswahl loeschen",    "Delete", self.action_delete_selection)
        self.act_clear_sel= A("Auswahl aufheben",    "Escape", self.action_clear_selection)
        self.act_resize   = A("Canvas-Groesse aendern...", None, self.action_resize_canvas)
        self.act_crop     = A("Auf Inhalt zuschneiden", None, self.action_crop_to_content)
        # Kein Hotkey -- 'R' ist Rect-Tool, Ctrl+R wuerde irritieren
        self.act_color_replace = A("Farbe ersetzen...", None, self.action_color_replace)

        self.act_flip_h   = A("Horizontal spiegeln", "Ctrl+H", self.action_flip_horizontal, make_action_icon("flip_h"))
        self.act_flip_v   = A("Vertikal spiegeln",   "Ctrl+J", self.action_flip_vertical,   make_action_icon("flip_v"))
        self.act_rot_cw   = A("90 Grad rechts",      "Ctrl+.", self.action_rotate_cw,       make_action_icon("rotate_cw"))
        self.act_rot_ccw  = A("90 Grad links",       "Ctrl+,", self.action_rotate_ccw,      make_action_icon("rotate_ccw"))

        self.act_frame_new = A("Neues Frame",        None, lambda: self.action_frame_add(False))
        self.act_frame_dup = A("Frame duplizieren",  None, lambda: self.action_frame_add(True))
        self.act_frame_del = A("Frame loeschen",     None, self.action_frame_delete)
        self.act_play      = A("Animation-Preview",  "Ctrl+P", self.action_play_animation, make_action_icon("play"))
        self.act_onion     = A("Onion-Skin (vorher blau, nachher rot)",
                                 None,
                                 lambda: self.set_onion(not self.canvas._show_onion),
                                 checkable=True)
        self.act_reverse   = A("Frames umkehren", None, self.action_reverse_frames)
        self.act_pingpong  = A("Ping-Pong (anfuegen)", None, self.action_pingpong_frames)
        self.act_flatten_current = A("Auf aktuelles Frame reduzieren",
                                       None, self.action_flatten_to_current)
        self.act_flatten_composite = A("Frames zusammenfuegen (Composite)",
                                         None, self.action_flatten_composite)
        self.act_palette_import = A("Palette laden (.gpl)...",
                                     None, self.action_palette_import)
        self.act_palette_export = A("Palette speichern (.gpl)...",
                                     None, self.action_palette_export)
        self.act_palette_extract = A("Palette aus Sprite extrahieren",
                                      None, self.action_palette_extract)

        self.act_zoom_in  = A("Zoom +", "Ctrl++", self.canvas_zoom_in_safe, make_action_icon("zoom_in"))
        self.act_zoom_out = A("Zoom -", "Ctrl+-", self.canvas_zoom_out_safe, make_action_icon("zoom_out"))
        self.act_zoom_reset = A("Zoom 100%", "Ctrl+0", self.canvas_zoom_reset_safe)
        self.act_grid     = A("Grid umschalten", None, self.canvas_toggle_grid_safe, checkable=True)
        self.act_grid.setChecked(True)
        self.act_tile_preview = A("Tile-Vorschau (3x3)", "T", self.toggle_tile_preview,
                                  make_action_icon("tile"), checkable=True)
        self.act_sym_x = A("Symmetrie X", "Ctrl+Shift+X", self.toggle_symmetry_x,
                           make_action_icon("sym_x"), checkable=True)
        self.act_sym_y = A("Symmetrie Y", "Ctrl+Shift+Y", self.toggle_symmetry_y,
                           make_action_icon("sym_y"), checkable=True)
        self.act_copy_code = A("GB-Code kopieren", "Ctrl+Shift+C",
                               self.action_copy_gb_code, make_action_icon("code"))
        self.act_test_sprite = A("Sprite testen (in GameBasic)", "F12",
                                  self.action_test_sprite,
                                  make_action_icon("test_sprite"))

        # Plus-Taste auch ohne Strg fuer Zoom
        QShortcut(QKeySequence(Qt.Key_Plus), self, activated=self.canvas_zoom_in_safe)
        QShortcut(QKeySequence(Qt.Key_Minus), self, activated=self.canvas_zoom_out_safe)
        QShortcut(QKeySequence(Qt.Key_0), self, activated=self.canvas_zoom_reset_safe)
        # X = swap
        QShortcut(QKeySequence("X"), self, activated=self.swap_colors)
        # Brush-Size 1..4
        for n in range(1, 5):
            QShortcut(QKeySequence(str(n)), self,
                      activated=lambda s=n: self.set_brush_size(s))
        # Frame F2..F9 = Frame 0..7
        for i in range(8):
            QShortcut(QKeySequence(f"F{i + 2}"), self,
                      activated=lambda idx=i: self.action_frame_select(idx))

        # Tool-Hotkeys: pro Tool ein QShortcut
        tool_hotkeys = {
            "pencil": "B", "eraser": "E", "spray": "Y", "bucket": "G",
            "line": "L", "rect": "R", "rect_fill": "Shift+R",
            "ellipse": "O", "ellipse_fill": "Shift+O",
            "eyedrop": "I", "select": "M", "magic_wand": "W",
            "move": "V",
        }
        for name, hk in tool_hotkeys.items():
            QShortcut(QKeySequence(hk), self,
                      activated=lambda n=name: self.activate_tool(n))

    def canvas_zoom_in_safe(self):  self.canvas.zoom_in()
    def canvas_zoom_out_safe(self): self.canvas.zoom_out()
    def canvas_zoom_reset_safe(self): self.canvas.zoom_reset()
    def canvas_toggle_grid_safe(self):
        self.canvas.set_show_grid(not self.canvas._show_grid)
        self.act_grid.setChecked(self.canvas._show_grid)

    def _build_menu(self):
        bar = self.menuBar()
        m_file = bar.addMenu("&Datei")
        m_file.addAction(self.act_new)
        m_file.addAction(self.act_open)
        self.menu_recent = m_file.addMenu("Zuletzt geoeffnet")
        self._refresh_recent_menu()
        m_file.addSeparator()
        m_file.addAction(self.act_save)
        m_file.addAction(self.act_save_as)
        m_file.addSeparator()
        m_file.addAction(self.act_export)
        m_file.addAction(self.act_export_atlas)
        m_file.addAction(self.act_export_gif)
        m_file.addAction(self.act_export_gbanim)
        m_file.addAction(self.act_export_frame)
        m_file.addSeparator()
        m_file.addAction(self.act_test_sprite)
        m_file.addAction(self.act_copy_code)
        # Palette-Submenu
        m_palette = m_file.addMenu("Palette")
        m_palette.addAction(self.act_palette_import)
        m_palette.addAction(self.act_palette_export)
        m_palette.addSeparator()
        m_palette.addAction(self.act_palette_extract)
        m_file.addSeparator()
        m_file.addAction(self.act_reload)
        m_file.addSeparator()
        m_file.addAction(self.act_close)

        m_edit = bar.addMenu("&Bearbeiten")
        m_edit.addAction(self.act_undo)
        m_edit.addAction(self.act_redo)
        m_edit.addSeparator()
        m_edit.addAction(self.act_cut)
        m_edit.addAction(self.act_copy)
        m_edit.addAction(self.act_paste)
        m_edit.addAction(self.act_paste_frame)
        m_edit.addSeparator()
        m_edit.addAction(self.act_clear)
        m_edit.addAction(self.act_del_sel)
        m_edit.addAction(self.act_clear_sel)
        m_edit.addSeparator()
        m_edit.addAction(self.act_flip_h)
        m_edit.addAction(self.act_flip_v)
        m_edit.addAction(self.act_rot_cw)
        m_edit.addAction(self.act_rot_ccw)
        m_edit.addSeparator()
        m_edit.addAction(self.act_sym_x)
        m_edit.addAction(self.act_sym_y)
        m_edit.addSeparator()
        m_edit.addAction(self.act_color_replace)
        m_edit.addAction(self.act_crop)
        m_edit.addAction(self.act_resize)

        m_frame = bar.addMenu("&Frame")
        m_frame.addAction(self.act_frame_new)
        m_frame.addAction(self.act_frame_dup)
        m_frame.addAction(self.act_frame_del)
        m_frame.addSeparator()
        m_frame.addAction(self.act_reverse)
        m_frame.addAction(self.act_pingpong)
        m_frame.addSeparator()
        m_frame.addAction(self.act_flatten_current)
        m_frame.addAction(self.act_flatten_composite)
        m_frame.addSeparator()
        m_frame.addAction(self.act_play)
        m_frame.addAction(self.act_onion)

        m_view = bar.addMenu("&Ansicht")
        m_view.addAction(self.act_zoom_in)
        m_view.addAction(self.act_zoom_out)
        m_view.addAction(self.act_zoom_reset)
        m_view.addSeparator()
        m_view.addAction(self.act_grid)
        m_view.addAction(self.act_tile_preview)
        # Reservieren fuer den Asset-Browser-Toggle, der erst nach
        # _build_dock_panels existiert (Dock-Toggle-Action haengt vom
        # Dock-Widget ab).
        self._view_menu = m_view

        m_help = bar.addMenu("&Hilfe")
        m_help.addAction(QAction("Sprite-Statistik...", self,
                                  triggered=self.show_statistics))
        m_help.addSeparator()
        m_help.addAction(QAction("Tastenkuerzel...", self,
                                  triggered=self.show_shortcuts))
        m_help.addAction(QAction("Ueber...", self,
                                  triggered=self.show_about))

    def _build_top_toolbar(self):
        tb = QToolBar("Datei")
        tb.setMovable(False)
        tb.setIconSize(QSize(22, 22))
        tb.setToolButtonStyle(Qt.ToolButtonTextBesideIcon)
        self.addToolBar(Qt.TopToolBarArea, tb)
        tb.addAction(self.act_new)
        tb.addAction(self.act_open)
        tb.addAction(self.act_save)
        tb.addSeparator()
        tb.addAction(self.act_export)
        tb.addAction(self.act_test_sprite)
        tb.addAction(self.act_copy_code)
        tb.addSeparator()
        tb.addAction(self.act_undo)
        tb.addAction(self.act_redo)
        tb.addSeparator()
        tb.addAction(self.act_cut)
        tb.addAction(self.act_copy)
        tb.addAction(self.act_paste)
        tb.addSeparator()
        tb.addAction(self.act_flip_h)
        tb.addAction(self.act_flip_v)
        tb.addAction(self.act_rot_cw)
        tb.addAction(self.act_rot_ccw)
        tb.addSeparator()
        tb.addAction(self.act_sym_x)
        tb.addAction(self.act_sym_y)
        tb.addAction(self.act_tile_preview)
        tb.addSeparator()
        tb.addAction(self.act_zoom_out)
        # Zoom-Label
        self.zoom_label = QLabel(" 100% ")
        self.zoom_label.setStyleSheet(f"color: {COLORS['fg']}; "
                                       f"font-family: Consolas; "
                                       f"min-width: 60px;")
        self.zoom_label.setAlignment(Qt.AlignCenter)
        tb.addWidget(self.zoom_label)
        tb.addAction(self.act_zoom_in)

    def _build_left_toolbar(self):
        tb = QToolBar("Werkzeuge")
        tb.setMovable(False)
        tb.setOrientation(Qt.Vertical)
        tb.setIconSize(QSize(28, 28))
        tb.setToolButtonStyle(Qt.ToolButtonIconOnly)
        self.addToolBar(Qt.LeftToolBarArea, tb)

        self._tool_actions = {}
        self._tool_group = QActionGroup(self) if False else None
        # Manuelle Mutex-Logik (QActionGroup setzt checked, aber wir wollen
        # die Hotkeys auch ohne ActionGroup, daher separat)
        for name, tip, hk in self.TOOLS:
            act = QAction(make_tool_icon(name, 28), tip, self)
            act.setCheckable(True)
            act.setToolTip(f"{tip}")
            act.triggered.connect(lambda _checked=False, n=name: self.activate_tool(n))
            tb.addAction(act)
            self._tool_actions[name] = act

    def _build_central(self):
        self.canvas = SpriteCanvas(self)
        self.setCentralWidget(self.canvas)

    def _build_dock_panels(self):
        # Color-Panel rechts
        self.color_panel = ColorPanel(self)
        color_dock = QDockWidget("Farbe", self)
        color_dock.setWidget(self.color_panel)
        color_dock.setFeatures(QDockWidget.DockWidgetMovable
                                | QDockWidget.DockWidgetFloatable)
        color_dock.setMinimumWidth(260)
        self.addDockWidget(Qt.RightDockWidgetArea, color_dock)

        # Frames-Panel rechts (unter Color)
        self.frames_panel = FramesPanel(self)
        frames_dock = QDockWidget("Frames", self)
        frames_dock.setWidget(self.frames_panel)
        frames_dock.setFeatures(QDockWidget.DockWidgetMovable
                                 | QDockWidget.DockWidgetFloatable
                                 | QDockWidget.DockWidgetClosable)
        frames_dock.setMinimumWidth(220)
        self.addDockWidget(Qt.RightDockWidgetArea, frames_dock)

        # Animations-Panel rechts (unter Frames): benannte Bereiche
        self.anims_panel = AnimsPanel(self)
        anims_dock = QDockWidget("Animationen", self)
        anims_dock.setWidget(self.anims_panel)
        anims_dock.setFeatures(QDockWidget.DockWidgetMovable
                                | QDockWidget.DockWidgetFloatable
                                | QDockWidget.DockWidgetClosable)
        anims_dock.setMinimumWidth(220)
        self.addDockWidget(Qt.RightDockWidgetArea, anims_dock)

        # Asset-Browser-Dock unten (default versteckt -- via Ansicht-Menue
        # einblenden). Zeigt PNGs/.gbsprite aus den Standard-Asset-
        # Verzeichnissen des Projekts an.
        self.asset_browser = AssetBrowser(self)
        self.asset_dock = QDockWidget("Asset-Browser", self)
        self.asset_dock.setWidget(self.asset_browser)
        self.asset_dock.setFeatures(QDockWidget.DockWidgetMovable
                                     | QDockWidget.DockWidgetFloatable
                                     | QDockWidget.DockWidgetClosable)
        self.asset_dock.setMinimumHeight(140)
        self.addDockWidget(Qt.BottomDockWidgetArea, self.asset_dock)
        self.asset_dock.hide()   # default off, der User toggled bei Bedarf

        # Asset-Browser-Toggle ans Ansicht-Menue anhaengen.
        # toggleViewAction() reflektiert show/hide automatisch.
        if hasattr(self, "_view_menu"):
            self._view_menu.addSeparator()
            asset_toggle = self.asset_dock.toggleViewAction()
            asset_toggle.setText("Asset-Browser")
            asset_toggle.setShortcut(QKeySequence("Ctrl+B"))
            self._view_menu.addAction(asset_toggle)

    def _build_statusbar(self):
        sb = QStatusBar()
        sb.setSizeGripEnabled(False)
        self.setStatusBar(sb)
        # Farb-Quadrat (sichtbar nur wenn der Cursor ueber einem Sprite-
        # Pixel haengt). Klein gehalten damit es zur Statusbar-Hoehe passt.
        self.status_color = _ColorBox(18)
        self.status_color.set_color((0, 0, 0, 0))
        self.status_color.setVisible(False)
        sb.addWidget(self.status_color)
        self.status_pixel = QLabel("")
        self.status_pixel.setStyleSheet(f"color: {COLORS['fg_muted']}; "
                                         f"font-family: Consolas;")
        self.status_doc = QLabel("")
        self.status_doc.setStyleSheet(f"color: {COLORS['fg_muted']}; "
                                       f"font-family: Consolas;")
        sb.addWidget(self.status_pixel, 1)
        sb.addPermanentWidget(self.status_doc)

    # --- Status / Tool / Color ---

    def update_status(self):
        n = len(self.doc.frames)
        path = str(self.doc.filepath) if self.doc.filepath else "(ungespeichert)"
        sym_short = {"none": "-", "x": "X", "y": "Y", "both": "X+Y"}[self.symmetry_mode]
        self.status_doc.setText(
            f"{self.doc.width}x{self.doc.height}  Frames: {n}  "
            f"#{self.doc.current_index:02d}  Brush: {self.brush_size}  "
            f"Sym: {sym_short}  {path}"
        )
        self.zoom_label.setText(f" {self.canvas._zoom * 100}% ")
        self._update_title()

    def _update_title(self):
        name = self.doc.filepath.name if self.doc.filepath else "neu"
        dirty = "*" if self.doc.dirty else ""
        self.setWindowTitle(f"GameBasic - Sprite-Editor - {dirty}{name}")

    def set_hover_pixel(self, coord):
        if coord is None:
            self.status_pixel.setText("")
            self.status_color.setVisible(False)
            return
        x, y = coord
        if not self.in_bounds(x, y):
            self.status_pixel.setText("")
            self.status_color.setVisible(False)
            return
        col = self.doc.current.pixels.getpixel((x, y))
        self.status_pixel.setText(
            f"x={x:3d} y={y:3d}  RGBA=({col[0]:3d},{col[1]:3d},{col[2]:3d},{col[3]:3d})"
        )
        self.status_color.set_color(col)
        self.status_color.setVisible(True)

    def in_bounds(self, x, y):
        return 0 <= x < self.doc.width and 0 <= y < self.doc.height

    def mark_dirty(self):
        self.doc.dirty = True
        self.frames_panel.refresh()
        self._update_title()

    def update_tool_buttons(self):
        for name, act in self._tool_actions.items():
            act.setChecked(name == self.tool_name)

    def activate_tool(self, name, _silent=False):
        if name not in self._tools:
            return
        prev = self.tool_name
        if not _silent:
            self._previous_tool_name = prev
        self.tool_name = name
        # Wechsel WEG von einem Selektions-Tool (Auswahl-Marquee oder
        # Magic Wand) zu einem aktiven Mal-Tool loest die Auswahl auf.
        # Wechsel zwischen den Selektions-Tools bleibt erhalten -- so
        # kann der User Magic Wand klicken und mit Pfeiltasten oder
        # Cut/Copy weitermachen, oder mit Auswahl-Tool die Bbox
        # nachjustieren.
        selection_tools = {"select", "magic_wand"}
        if (prev in selection_tools and name not in selection_tools
                and not _silent):
            self.canvas.clear_selection()
        self.update_tool_buttons()
        cursor_map = {
            "eyedrop": Qt.CrossCursor,
            "move":    Qt.SizeAllCursor,
        }
        self.canvas.viewport().setCursor(cursor_map.get(name, Qt.CrossCursor))

    def set_fg(self, col):
        col = self._normalize(col)
        # Color-History pflegen: gleiche Farbe nicht doppelt eintragen,
        # neue Farbe vorne einreihen, max-Limit.
        if not self.color_history or self.color_history[0] != col:
            try:
                self.color_history.remove(col)
            except ValueError:
                pass
            self.color_history.insert(0, col)
            self.color_history = self.color_history[:self.COLOR_HISTORY_MAX]
        self.fg = col
        self.color_panel.refresh()

    def set_bg(self, col):
        self.bg = self._normalize(col)
        self.color_panel.refresh()

    def swap_colors(self):
        self.fg, self.bg = self.bg, self.fg
        self.color_panel.refresh()

    @staticmethod
    def _normalize(col):
        if len(col) == 3:
            return (int(col[0]), int(col[1]), int(col[2]), 255)
        return (int(col[0]), int(col[1]), int(col[2]), int(col[3]))

    def set_brush_size(self, size):
        size = max(1, min(4, int(size)))
        if size == self.brush_size:
            return
        self.brush_size = size
        self.update_status()

    def set_onion(self, on: bool):
        self.canvas.set_show_onion(on)
        self.frames_panel.onion_check.setChecked(on)
        self.act_onion.setChecked(on)

    # --- Symmetrie / Tile-Preview ---

    def set_symmetry_mode(self, mode: str):
        if mode not in ("none", "x", "y", "both"):
            return
        self.symmetry_mode = mode
        # Toolbar-Buttons synchronisieren
        self.act_sym_x.setChecked(mode in ("x", "both"))
        self.act_sym_y.setChecked(mode in ("y", "both"))
        msg = {
            "none": "Symmetrie aus",
            "x":    "Symmetrie X (vertikale Achse)",
            "y":    "Symmetrie Y (horizontale Achse)",
            "both": "Symmetrie X+Y",
        }[mode]
        self.statusBar().showMessage(msg, 1500)
        self.update_status()

    def toggle_symmetry_x(self):
        cur = self.symmetry_mode
        new = {"none": "x", "x": "none", "y": "both", "both": "y"}[cur]
        self.set_symmetry_mode(new)

    def toggle_symmetry_y(self):
        cur = self.symmetry_mode
        new = {"none": "y", "y": "none", "x": "both", "both": "x"}[cur]
        self.set_symmetry_mode(new)

    def toggle_tile_preview(self):
        on = not self.canvas._show_tile_preview
        self.canvas.set_tile_preview(on)
        self.act_tile_preview.setChecked(on)
        self.statusBar().showMessage(
            "Tile-Vorschau an (3x3-Grid)" if on else "Tile-Vorschau aus", 1500
        )

    # --- Clipboard / GB-Code ---

    def action_copy(self):
        sel = self.canvas.get_selection()
        if not sel:
            self.statusBar().showMessage(
                "Keine Auswahl -- mit M (Auswahl-Tool) markieren", 2000)
            return
        x0, y0, x1, y1 = sel
        self._clipboard_pil = self.doc.current.pixels.crop(
            (x0, y0, x1, y1)).copy()
        # Auch ins System-Clipboard, falls der User das Bild woanders
        # einfuegen will (Discord, Aseprite, ...).
        try:
            QApplication.clipboard().setPixmap(pil_to_qpixmap(self._clipboard_pil))
        except Exception:
            pass
        w = x1 - x0; h = y1 - y0
        self.statusBar().showMessage(
            f"Auswahl kopiert ({w}x{h})", 1500)

    def action_cut(self):
        sel = self.canvas.get_selection()
        if not sel:
            self.statusBar().showMessage(
                "Keine Auswahl -- mit M (Auswahl-Tool) markieren", 2000)
            return
        self.action_copy()
        self.doc.current.snapshot()
        x0, y0, x1, y1 = sel
        d = ImageDraw.Draw(self.doc.current.pixels)
        d.rectangle([x0, y0, x1 - 1, y1 - 1], fill=(0, 0, 0, 0))
        # Auswahl nach Cut aufloesen -- die Pixel sind weg, der schwebende
        # Marquee waere irrefuehrend. Bei Copy bleibt sie bestehen.
        self.canvas.clear_selection()
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    def action_paste(self):
        if self._clipboard_pil is None:
            self.statusBar().showMessage(
                "Zwischenablage leer -- erst Strg+C", 2000)
            return
        self.doc.current.snapshot()
        sel = self.canvas.get_selection()
        if sel:
            dest = (sel[0], sel[1])
        else:
            dest = (0, 0)
        self.doc.current.pixels.alpha_composite(
            self._clipboard_pil, dest=dest)
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    def _system_clipboard_pil(self) -> Optional[Image.Image]:
        """Liefert das System-Clipboard-Bild als PIL.Image (oder None).
        Best-effort -- erlaubt Paste-as-Frame von Bildern aus anderen
        Programmen (Aseprite, Browser, ...)."""
        try:
            qimg = QApplication.clipboard().image()
        except Exception:
            return None
        if qimg is None or qimg.isNull():
            return None
        try:
            qimg = qimg.convertToFormat(QImage.Format_RGBA8888)
            w, h = qimg.width(), qimg.height()
            bpl = qimg.bytesPerLine()
            raw = bytes(qimg.constBits())[:bpl * h]
            if bpl == w * 4:
                return Image.frombytes("RGBA", (w, h), raw)
            # Zeilen-Padding entfernen
            rows = [raw[r * bpl: r * bpl + w * 4] for r in range(h)]
            return Image.frombytes("RGBA", (w, h), b"".join(rows))
        except Exception:
            return None

    def action_paste_as_frame(self):
        """Fuegt den Zwischenablage-Inhalt als NEUES Frame ein (nach dem
        aktuellen) -- intern kopierte Auswahl bevorzugt, sonst System-
        Clipboard-Bild."""
        img = self._clipboard_pil or self._system_clipboard_pil()
        if img is None:
            self.statusBar().showMessage(
                "Zwischenablage leer -- erst Strg+C oder Bild kopieren", 2000)
            return
        idx = self.doc.paste_as_frame(img)
        self.canvas.invalidate_all()
        self.canvas._render_onion()
        self.frames_panel.refresh()
        self.mark_dirty()
        self.update_status()
        self.statusBar().showMessage(
            f"Als neues Frame #{idx:02d} eingefuegt", 1500)

    def action_test_sprite(self):
        """Generiert ein temporaeres GB-Test-Programm das das Sprite
        anzeigt (animiert wenn Multi-Frame), und startet es via gbrun.py
        als Subprozess. Der User sieht das Sprite live im
        nativen gbrt-Fenster wie es im Spiel aussehen wuerde.
        """
        import sys, subprocess, tempfile
        if not self.doc.frames:
            return
        gbrun = self.project_root / "gbrun.py"
        if not gbrun.exists():
            QMessageBox.warning(
                self, "gbrun.py fehlt",
                f"gbrun.py nicht gefunden in {self.project_root}.\n"
                f"Sprite-Test braucht das CLI um GB-Programme zu starten.")
            return
        # Temp-Verzeichnis -- bewusst NICHT loeschen, der gbrun-Subprozess
        # liest die Dateien noch beim Start. OS-Cleanup nach Reboot ist OK.
        tmpdir = Path(tempfile.mkdtemp(prefix="gb_sprite_test_"))
        n = len(self.doc.frames)
        if n > 1:
            sprite_file = tmpdir / "test_sheet.png"
            try:
                self.doc.save_sheet_png(sprite_file)
            except Exception as exc:
                QMessageBox.critical(self, "Test-Vorbereitung fehlgeschlagen", str(exc))
                return
        else:
            sprite_file = tmpdir / "test_sprite.png"
            # save_png_single setzt filepath/dirty -- Pre-State sichern
            orig_path = self.doc.filepath
            orig_dirty = self.doc.dirty
            try:
                self.doc.save_png_single(sprite_file)
            except Exception as exc:
                QMessageBox.critical(self, "Test-Vorbereitung fehlgeschlagen", str(exc))
                return
            finally:
                self.doc.filepath = orig_path
                self.doc.dirty = orig_dirty
                self._update_title()

        gb_path = tmpdir / "_test.gb"
        gb_path.write_text(
            self._build_test_gb_code(sprite_file.name, n),
            encoding="utf-8",
        )
        cmd = [sys.executable, str(gbrun), str(gb_path)]
        try:
            subprocess.Popen(cmd, cwd=str(tmpdir))
        except Exception as exc:
            QMessageBox.critical(self, "Start fehlgeschlagen", str(exc))
            return
        self.statusBar().showMessage(
            f"Sprite-Test gestartet: {n} Frame(s) in {tmpdir.name}", 4000)

    def _build_test_gb_code(self, sprite_filename: str, n_frames: int) -> str:
        """Baut den GB-Source-Code fuer den Sprite-Test.

        Multi-Frame: SPRITE-Modul mit looped Animation, plus zwei
        Begleit-Sprites die auf einer Sinuskurve schwingen -- so sieht
        man die Animation in Bewegung.
        Single-Frame: einfaches DrawImage mit Hintergrund-Pattern.
        """
        w, h = self.doc.width, self.doc.height
        cx = max(160 - w // 2, 20)
        cy = max(120 - h // 2, 20)
        a0 = self.doc._effective_anims()[0]
        if n_frames > 1:
            return (
                f"' Auto-generiert vom Sprite-Editor.\n"
                f"' Schliesst sich mit ESC.\n"
                f'IMPORT "sprite"\n'
                f'\n'
                f'Screen(320, 240, "Sprite-Test ({w}x{h}, {n_frames} Frames)")\n'
                f'\n'
                f'DIM sheet AS IMAGE\n'
                f'sheet = LoadImage("{sprite_filename}")\n'
                f'\n'
                f'DIM sp AS SPRITE\n'
                f'sp = SPRITE_NEW(sheet, {w}, {h})\n'
                f'SPRITE_ADD_ANIM(sp, "{a0.name}", {a0.first}, {a0.last}, {a0.fps})\n'
                f'SPRITE_PLAY(sp, "{a0.name}")\n'
                f'SPRITE_SET_POS(sp, {cx}, {cy})\n'
                f'\n'
                f'DIM sp_l AS SPRITE\n'
                f'sp_l = SPRITE_NEW(sheet, {w}, {h})\n'
                f'SPRITE_ADD_ANIM(sp_l, "{a0.name}", {a0.first}, {a0.last}, {max(1, a0.fps - 2)})\n'
                f'SPRITE_PLAY(sp_l, "{a0.name}")\n'
                f'\n'
                f'DIM sp_r AS SPRITE\n'
                f'sp_r = SPRITE_NEW(sheet, {w}, {h})\n'
                f'SPRITE_ADD_ANIM(sp_r, "{a0.name}", {a0.first}, {a0.last}, {min(60, a0.fps + 2)})\n'
                f'SPRITE_PLAY(sp_r, "{a0.name}")\n'
                f'\n'
                f'DIM running AS BOOLEAN\n'
                f'DIM t AS INTEGER\n'
                f'running = TRUE\n'
                f't = 0\n'
                f'\n'
                f'WHILE running\n'
                f'    Cls(RGB(28, 28, 48))\n'
                f'    \n'
                f"    ' Hintergrund-Raster\n"
                f'    DIM gx AS INTEGER\n'
                f'    DIM gy AS INTEGER\n'
                f'    FOR gy = 0 TO 6\n'
                f'        FOR gx = 0 TO 9\n'
                f'            Box(gx * 32, gy * 32, gx * 32 + 31, gy * 32 + 31, RGB(38, 38, 60))\n'
                f'        NEXT gx\n'
                f'    NEXT gy\n'
                f'    \n'
                f'    Text(8, 8, "Sprite-Test - {w}x{h} - {n_frames} Frames", WHITE)\n'
                f'    Text(8, 24, "ESC = Ende", RGB(160, 200, 255))\n'
                f'    \n'
                f"    ' Begleit-Sprites auf Sinus\n"
                f'    DIM ang AS FLOAT\n'
                f'    ang = t / 30.0\n'
                f'    SPRITE_SET_POS(sp_l, 80 + INT(20 * SIN(ang)),       120 + INT(30 * COS(ang)))\n'
                f'    SPRITE_SET_POS(sp_r, 224 + INT(20 * SIN(ang + 1.0)), 120 + INT(30 * COS(ang + 1.0)))\n'
                f'    \n'
                f'    SPRITE_UPDATE(sp,   16)\n'
                f'    SPRITE_UPDATE(sp_l, 16)\n'
                f'    SPRITE_UPDATE(sp_r, 16)\n'
                f'    SPRITE_DRAW(sp_l)\n'
                f'    SPRITE_DRAW(sp_r)\n'
                f'    SPRITE_DRAW(sp)\n'
                f'    \n'
                f'    Flip()\n'
                f'    t = t + 1\n'
                f'    IF KEYPRESSED(KEY_ESCAPE) THEN running = FALSE\n'
                f'WEND\n'
            )
        # Single-Frame
        return (
            f"' Auto-generiert vom Sprite-Editor.\n"
            f"' Schliesst sich mit ESC.\n"
            f'Screen(320, 240, "Sprite-Test ({w}x{h})")\n'
            f'\n'
            f'DIM img AS IMAGE\n'
            f'img = LoadImage("{sprite_filename}")\n'
            f'\n'
            f'DIM running AS BOOLEAN\n'
            f'running = TRUE\n'
            f'WHILE running\n'
            f'    Cls(RGB(28, 28, 48))\n'
            f'    \n'
            f'    DIM gx AS INTEGER\n'
            f'    DIM gy AS INTEGER\n'
            f'    FOR gy = 0 TO 6\n'
            f'        FOR gx = 0 TO 9\n'
            f'            Box(gx * 32, gy * 32, gx * 32 + 31, gy * 32 + 31, RGB(38, 38, 60))\n'
            f'        NEXT gx\n'
            f'    NEXT gy\n'
            f'    \n'
            f'    Text(8, 8, "Sprite-Test - {w}x{h}", WHITE)\n'
            f'    Text(8, 24, "ESC = Ende", RGB(160, 200, 255))\n'
            f'    \n'
            f'    DrawImage(img, {cx}, {cy})\n'
            f'    Flip()\n'
            f'    IF KEYPRESSED(KEY_ESCAPE) THEN running = FALSE\n'
            f'WEND\n'
        )

    def action_copy_gb_code(self):
        """Legt den passenden GameBasic-Snippet (LOADIMAGE/SPRITE_NEW/...)
        in die Zwischenablage, mit aktuellem Sprite-Namen + Frame-Anzahl
        + Sprite-Groesse vorausgefuellt."""
        n = len(self.doc.frames)
        if self.doc.filepath is not None:
            if self.doc.filepath.suffix.lower() == ".gbsprite":
                fname = self.doc.filepath.with_suffix(".png").name
            else:
                fname = self.doc.filepath.name
        else:
            fname = "sprite.png"

        if n > 1:
            # Echte Anim-Bereiche (oder \"idle\" ueber alles mit FPS aus den
            # tatsaechlichen Frame-Dauern) -- generiert im Datenmodell.
            snippet = self.doc.generate_gb_snippet(fname)
        else:
            snippet = (
                f'DIM img AS IMAGE\n'
                f'img = LoadImage("{fname}")\n'
                f'\n'
                f"' Zeichnen:\n"
                f'DrawImage(img, 100, 100)\n'
            )
        QApplication.clipboard().setText(snippet)
        self.statusBar().showMessage(
            f"GameBasic-Code in Zwischenablage ({n} Frame{'s' if n > 1 else ''})",
            2500)

    # --- File-Aktionen ---

    def _confirm_dirty(self) -> bool:
        if not self.doc.dirty:
            return True
        ans = QMessageBox.question(
            self, "Aenderungen speichern?",
            "Das Sprite hat ungesicherte Aenderungen.\n"
            "Vor dem naechsten Schritt speichern?",
            QMessageBox.Yes | QMessageBox.No | QMessageBox.Cancel,
        )
        if ans == QMessageBox.Cancel:
            return False
        if ans == QMessageBox.Yes:
            return self.action_save()
        return True

    def action_new(self):
        if not self._confirm_dirty():
            return
        dlg = NewSpriteDialog(self)
        if dlg.exec() == QDialog.Accepted and dlg.result_size is not None:
            w, h = dlg.result_size
            self.doc = SpriteDoc(w, h)
            self.canvas.rebuild_scene()
            self.frames_panel.refresh()
            self.update_status()

    def action_open(self):
        if not self._confirm_dirty():
            return
        path, _ = QFileDialog.getOpenFileName(
            self, "Sprite oeffnen", str(self.project_root),
            "Alle Sprite-Formate (*.gbsprite *.png);;"
            "GameBasic-Sprite (*.gbsprite);;"
            "PNG-Bild (*.png)",
        )
        if not path:
            return
        self._load_path(Path(path))

    def _load_path(self, path: Path):
        try:
            if path.suffix.lower() == ".gbsprite":
                self.doc = SpriteDoc.load_native(path)
            else:
                img = Image.open(path)
                iw, ih = img.size
                fw, fh = iw, ih
                if iw > 64 or (iw % ih == 0 and iw // ih > 1):
                    # Sheet-Import-Dialog mit Live-Vorschau
                    img_rgba = img.convert("RGBA")
                    dlg = SheetImportDialog(self, img_rgba)
                    if dlg.exec() != QDialog.Accepted:
                        return
                    fw, fh = dlg.result_size
                self.doc = SpriteDoc.load_image(path, fw, fh)
        except Exception as exc:
            QMessageBox.critical(self, "Laden fehlgeschlagen", str(exc))
            return
        self._add_recent(path)
        self._watch_current_file()
        self.canvas.rebuild_scene()
        self.frames_panel.refresh()
        self.color_panel.refresh()
        self.update_status()

    def action_save(self) -> bool:
        if self.doc.filepath is None:
            return self.action_save_as()
        path = self.doc.filepath
        # Watcher fuer 2 Sek. unterdruecken -- damit unser eigener Save
        # nicht als externe Aenderung gemeldet wird.
        import time
        self._suppress_watcher_until = time.time() + 2.0
        try:
            if path.suffix.lower() == ".gbsprite":
                self.doc.save_native(path)
            elif len(self.doc.frames) > 1:
                self.doc.save_sheet_png(path)
                self.doc.dirty = False
            else:
                self.doc.save_png_single(path)
        except Exception as exc:
            QMessageBox.critical(self, "Speichern fehlgeschlagen", str(exc))
            return False
        if self.doc.filepath is not None:
            self._add_recent(self.doc.filepath)
        self._watch_current_file()
        # Backup ist jetzt obsolet
        self._delete_backup()
        self.update_status()
        return True

    def action_save_as(self) -> bool:
        ext = "gbsprite" if len(self.doc.frames) > 1 else "png"
        path, _ = QFileDialog.getSaveFileName(
            self, "Sprite speichern unter",
            str(self.project_root / f"sprite.{ext}"),
            "GameBasic-Sprite (*.gbsprite);;PNG-Bild (*.png)",
        )
        if not path:
            return False
        self.doc.filepath = Path(path)
        return self.action_save()

    def action_export_sheet(self):
        path, _ = QFileDialog.getSaveFileName(
            self, "Sheet als PNG exportieren",
            str(self.project_root / f"sheet_{self.doc.width}x{self.doc.height}.png"),
            "PNG-Bild (*.png)",
        )
        if not path:
            return
        try:
            self.doc.save_sheet_png(Path(path), layout="horizontal")
        except Exception as exc:
            QMessageBox.critical(self, "Export fehlgeschlagen", str(exc))
            return
        n = len(self.doc.frames)
        QMessageBox.information(
            self, "Sheet exportiert",
            f"{n} Frames als horizontaler Sheet gespeichert:\n{path}\n\n"
            f"In GameBasic laden:\n"
            f"   IMPORT \"sprite\"\n"
            f"   img = LOADIMAGE(\"{Path(path).name}\")\n"
            f"   sp = SPRITE_NEW(img, {self.doc.width}, {self.doc.height})\n"
            f"   SPRITE_ADD_ANIM(sp, \"idle\", 0, {n - 1}, 8)\n"
            f"   SPRITE_PLAY(sp, \"idle\")"
        )

    def action_export_atlas(self):
        """Exportiert das Sheet als SPRITE_ATLAS: PNG + JSON-Manifest.
        Schliesst den Workflow-Loop zum `SPRITE_ATLAS`-Typ in GameBasic --
        nach dem Export kann das User-Programm `ATLAS_LOAD("xyz.json")`
        aufrufen und ueber benannte Sub-Sprites (`tiles_0`, `tiles_1`,
        ...) zugreifen.

        File-Picker nimmt den PNG-Pfad entgegen; das JSON wird daneben
        mit gleicher Basename + `.json` geschrieben. Sprite-Namen sind
        `<png_basename>_<idx>`.
        """
        n = len(self.doc.frames)
        if n < 1:
            return
        path, _ = QFileDialog.getSaveFileName(
            self, "Sprite-Atlas exportieren (PNG + JSON)",
            str(self.project_root / f"atlas_{self.doc.width}x{self.doc.height}.png"),
            "PNG-Bild (*.png)",
        )
        if not path:
            return
        png_path = Path(path)
        json_path = png_path.with_suffix(".json")
        try:
            self.doc.save_sheet_atlas(png_path, json_path, layout="horizontal")
        except Exception as exc:
            QMessageBox.critical(self, "Atlas-Export fehlgeschlagen", str(exc))
            return
        prefix = png_path.stem
        QMessageBox.information(
            self, "Atlas exportiert",
            f"{n} Frames als Sprite-Atlas gespeichert:\n"
            f"  {png_path.name}\n  {json_path.name}\n\n"
            f"In GameBasic laden:\n"
            f"   DIM atlas AS SPRITE_ATLAS\n"
            f"   atlas = ATLAS_LOAD(\"{json_path.name}\")\n"
            f"   ATLAS_DRAW(atlas, \"{prefix}_0\", x, y)\n"
            f"   ' oder batched fuer viele Sprites:\n"
            f"   BATCH_DRAW(atlas, \"{prefix}_0\", x, y)\n"
            f"   BATCH_FLUSH()"
        )

    def action_export_gif(self):
        """Animation als GIF exportieren -- mit transparentem Hintergrund.

        Falls die Frames unterschiedliche Dauern haben, fragt der Dialog,
        ob diese verwendet werden sollen oder ob eine einheitliche FPS
        gilt.
        """
        n = len(self.doc.frames)
        if n < 1:
            return
        # Nutzen die Frames unterschiedliche Dauer?
        durations = {f.duration_ms for f in self.doc.frames}
        fps: Optional[int] = None
        if n > 1:
            if len(durations) > 1:
                # Mehrere Dauern -> User entscheidet
                ans = QMessageBox.question(
                    self, "GIF-Export",
                    "Die Frames haben unterschiedliche Dauern. "
                    "Pro-Frame-Dauer beibehalten?\n\n"
                    "Ja  = jede Frame-Dauer einzeln verwenden\n"
                    "Nein = einheitliche FPS abfragen",
                )
                if ans == QMessageBox.Yes:
                    fps = None
                else:
                    val, ok = QInputDialog.getInt(
                        self, "GIF-Export",
                        "Einheitliche Frame-Rate (FPS):", 8, 1, 30)
                    if not ok:
                        return
                    fps = val
            else:
                # Alle gleich -> direkt nutzen, FPS aus duration_ms ableiten
                fps = None
        path, _ = QFileDialog.getSaveFileName(
            self, "Animation als GIF",
            str(self.project_root / "animation.gif"),
            "GIF (*.gif)",
        )
        if not path:
            return
        try:
            self.doc.save_animated_gif(Path(path), fps=fps, loop=0)
        except Exception as exc:
            QMessageBox.critical(self, "Export fehlgeschlagen", str(exc))
            return
        scope = f"einheitlich {fps} fps" if fps else "pro-Frame-Dauer"
        self.statusBar().showMessage(
            f"GIF exportiert: {n} Frames ({scope}) -> {Path(path).name}",
            4000,
        )

    def action_export_gbanim(self):
        """Animations-FSM-Vorlage (.gbanim) aus den Anim-Bereichen schreiben:
        ein State pro Bereich, erster = default. Direkt ANIM_FSM_LOAD-ladbar;
        Transitions/Parameter ergaenzt man in gbanim."""
        default_name = "sprite.gbanim"
        if self.doc.filepath is not None:
            default_name = self.doc.filepath.with_suffix(".gbanim").name
        path, _ = QFileDialog.getSaveFileName(
            self, "Animations-FSM exportieren (.gbanim)", default_name,
            "GameBasic-Animation (*.gbanim)")
        if not path:
            return
        try:
            Path(path).write_text(
                json.dumps(self.doc.generate_gbanim(), indent=2),
                encoding="utf-8")
        except Exception as exc:
            QMessageBox.critical(self, "Export fehlgeschlagen", str(exc))
            return
        n = len(self.doc.anims) or 1
        self.statusBar().showMessage(
            f".gbanim exportiert ({n} State{'s' if n != 1 else ''}): {path}", 3000)

    def action_export_frame_png(self):
        path, _ = QFileDialog.getSaveFileName(
            self, "Aktuelles Frame als PNG",
            str(self.project_root / f"frame_{self.doc.current_index:02d}.png"),
            "PNG-Bild (*.png)",
        )
        if not path:
            return
        try:
            self.doc.current.pixels.save(path, format="PNG")
        except Exception as exc:
            QMessageBox.critical(self, "Export fehlgeschlagen", str(exc))

    # --- Edit ---

    def action_undo(self):
        if self.doc.current.undo():
            self.canvas.invalidate_all()
            self.frames_panel.refresh()
            self.mark_dirty()

    def action_redo(self):
        if self.doc.current.redo():
            self.canvas.invalidate_all()
            self.frames_panel.refresh()
            self.mark_dirty()

    def action_clear_frame(self):
        f = self.doc.current
        f.snapshot()
        f.pixels = Image.new("RGBA", (self.doc.width, self.doc.height),
                             (0, 0, 0, 0))
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    def action_delete_selection(self):
        sel = self.canvas.get_selection()
        if not sel:
            return
        self.doc.current.snapshot()
        x0, y0, x1, y1 = sel
        d = ImageDraw.Draw(self.doc.current.pixels)
        d.rectangle([x0, y0, x1 - 1, y1 - 1], fill=(0, 0, 0, 0))
        # Auswahl nach Delete aufloesen
        self.canvas.clear_selection()
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    # --- Palette-Verwaltung ---

    @staticmethod
    def _parse_gpl(path: Path) -> list[tuple]:
        """Liest eine GIMP-Palette (.gpl). Format:
            GIMP Palette
            Name: ...
            Columns: ...
            #
            R G B  Name
            ...
        """
        palette = []
        text = path.read_text(encoding="utf-8", errors="ignore")
        for line in text.splitlines():
            s = line.strip()
            if not s or s.startswith("#"):
                continue
            if s.startswith(("GIMP", "Name:", "Columns:")):
                continue
            parts = s.split(maxsplit=3)
            if len(parts) < 3:
                continue
            try:
                r, g, b = int(parts[0]), int(parts[1]), int(parts[2])
                palette.append((r, g, b, 255))
            except ValueError:
                continue
        return palette

    @staticmethod
    def _write_gpl(path: Path, palette: list[tuple],
                   name: str = "GameBasic Palette"):
        lines = ["GIMP Palette",
                 f"Name: {name}",
                 "Columns: 8",
                 "#"]
        for col in palette:
            r, g, b = int(col[0]), int(col[1]), int(col[2])
            lines.append(f"{r:3d} {g:3d} {b:3d}\tColor")
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def action_palette_import(self):
        path, _ = QFileDialog.getOpenFileName(
            self, "Palette laden",
            str(self.project_root),
            "GIMP-Palette (*.gpl);;Alle Dateien (*.*)",
        )
        if not path:
            return
        try:
            new_palette = self._parse_gpl(Path(path))
        except Exception as exc:
            QMessageBox.critical(self, "Laden fehlgeschlagen", str(exc))
            return
        if not new_palette:
            QMessageBox.warning(self, "Leere Palette",
                                 "Keine Farben in der Datei gefunden.")
            return
        # Bestehende Palette ersetzen, max 32 Farben (so viel wie das
        # Color-Panel in der Default-Layout zeigt).
        self.palette = new_palette[:32]
        # Color-Panel komplett neu bauen damit die Palette-Swatches
        # die neuen Farben zeigen
        self.color_panel.refresh()
        # Gewollte UX: das Panel-Layout ist hardcodiert auf 32 Slots,
        # daher kann ich nicht einfach swap'en -- ein voller Rebuild
        # waere zuviel. Pragmatisch: zeige Hinweis dass User die Palette
        # benutzen kann via Klick (auch wenn die Swatch-Positionen nicht
        # neu erstellt werden -- in der Praxis ueberschreiben wir die
        # bestehenden Farben).
        # Tatsaechlich: Color-Panel haelt seine eigene swatch-Liste, ich
        # rebuild ihn:
        self._rebuild_color_panel()
        self.statusBar().showMessage(
            f"Palette geladen: {len(self.palette)} Farben aus {Path(path).name}",
            3000,
        )

    def action_palette_export(self):
        path, _ = QFileDialog.getSaveFileName(
            self, "Palette speichern",
            str(self.project_root / "palette.gpl"),
            "GIMP-Palette (*.gpl)",
        )
        if not path:
            return
        try:
            self._write_gpl(Path(path), self.palette, name=Path(path).stem)
        except Exception as exc:
            QMessageBox.critical(self, "Speichern fehlgeschlagen", str(exc))
            return
        self.statusBar().showMessage(
            f"Palette gespeichert: {Path(path).name}", 3000)

    def action_palette_extract(self):
        """Extrahiert die haeufigsten Farben aus dem aktuellen Sprite
        und ersetzt die Palette damit. Top-32 nach Pixel-Anzahl,
        transparente Pixel ignorieren."""
        from collections import Counter
        cnt = Counter()
        for f in self.doc.frames:
            for col in f.pixels.getdata():
                if col[3] >= 128:
                    # Auf RGB reduzieren (Alpha hier nicht relevant fuer
                    # Palette -- sonst haetten wir oft die selbe Farbe
                    # mit verschiedenen Alphas mehrfach drin).
                    cnt[(col[0], col[1], col[2], 255)] += 1
        if not cnt:
            QMessageBox.information(
                self, "Palette extrahieren",
                "Sprite enthaelt nur transparente Pixel.")
            return
        top = [c for c, _ in cnt.most_common(32)]
        self.palette = top
        self._rebuild_color_panel()
        self.statusBar().showMessage(
            f"Palette aus Sprite: {len(top)} Top-Farben uebernommen",
            3000,
        )

    def _rebuild_color_panel(self):
        """Color-Panel neu bauen damit die Palette-Swatches die neuen
        Farben anzeigen. Pragmatischer als manuelles Update -- das Panel
        ist klein, die Performance reicht."""
        # Position im Layout merken: das Panel haengt in einem Dock
        old_panel = self.color_panel
        dock = old_panel.parent()
        # Neuen Panel bauen
        self.color_panel = ColorPanel(self)
        if isinstance(dock, QDockWidget):
            dock.setWidget(self.color_panel)
        old_panel.deleteLater()
        self.color_panel.refresh()

    def action_color_replace(self):
        """Oeffnet den Color-Replace-Dialog: ersetzt eine Farbe durch eine
        andere im aktuellen Frame, optional in der Auswahl oder ueber alle
        Frames."""
        dlg = ColorReplaceDialog(self)
        if dlg.exec() != QDialog.Accepted:
            return
        src = dlg.source_color()
        dst = dlg.target_color()
        if src == dst:
            return
        only_selection = dlg.only_in_selection()
        all_frames = dlg.all_frames()
        sel = self.canvas.get_selection() if only_selection else None
        # Snapshot des aktuellen Frames vor allem (alle Frames werden NICHT
        # in den Undo-Stack des aktuellen Frames gepackt -- das waere falsch.
        # Wir snapshotten pro Frame.)
        total = 0
        target_frames = self.doc.frames if all_frames else [self.doc.current]
        for f in target_frames:
            f.snapshot()
            px = f.pixels.load()
            if sel:
                x0, y0, x1, y1 = sel
            else:
                x0, y0, x1, y1 = 0, 0, self.doc.width, self.doc.height
            for y in range(y0, y1):
                for x in range(x0, x1):
                    if px[x, y] == src:
                        px[x, y] = dst
                        total += 1
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()
        scope = "alle Frames" if all_frames else "aktuelles Frame"
        sel_txt = " (nur Auswahl)" if only_selection else ""
        self.statusBar().showMessage(
            f"{total} Pixel ersetzt in {scope}{sel_txt}", 3000)

    def action_fill_selection_fg(self):
        """Fuellt den Auswahl-Bereich mit der aktuellen Vordergrundfarbe."""
        sel = self.canvas.get_selection()
        if not sel:
            return
        self.doc.current.snapshot()
        x0, y0, x1, y1 = sel
        d = ImageDraw.Draw(self.doc.current.pixels)
        d.rectangle([x0, y0, x1 - 1, y1 - 1], fill=self.fg)
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    def action_flip_selection_h(self):
        """Spiegelt nur den Inhalt der Auswahl horizontal."""
        sel = self.canvas.get_selection()
        if not sel:
            return
        self.doc.current.snapshot()
        x0, y0, x1, y1 = sel
        cropped = self.doc.current.pixels.crop((x0, y0, x1, y1))
        flipped = cropped.transpose(Image.FLIP_LEFT_RIGHT)
        d = ImageDraw.Draw(self.doc.current.pixels)
        d.rectangle([x0, y0, x1 - 1, y1 - 1], fill=(0, 0, 0, 0))
        self.doc.current.pixels.alpha_composite(flipped, dest=(x0, y0))
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    def action_flip_selection_v(self):
        """Spiegelt nur den Inhalt der Auswahl vertikal."""
        sel = self.canvas.get_selection()
        if not sel:
            return
        self.doc.current.snapshot()
        x0, y0, x1, y1 = sel
        cropped = self.doc.current.pixels.crop((x0, y0, x1, y1))
        flipped = cropped.transpose(Image.FLIP_TOP_BOTTOM)
        d = ImageDraw.Draw(self.doc.current.pixels)
        d.rectangle([x0, y0, x1 - 1, y1 - 1], fill=(0, 0, 0, 0))
        self.doc.current.pixels.alpha_composite(flipped, dest=(x0, y0))
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    def show_selection_context_menu(self, global_pos):
        """Oeffnet das Kontextmenue fuer den aktuellen Auswahl-Bereich.

        Wird von zwei Stellen aufgerufen:
        - automatisch nach SelectTool.end (User hat einen Marquee gezogen)
        - bei Rechtsklick auf einen Pixel innerhalb der bestehenden Auswahl

        Single-Pixel-Auswahlen (1x1) zeigen das Menue nicht -- die sind
        meist Versehen und der Popup waere nervig.
        """
        sel = self.canvas.get_selection()
        if not sel:
            return
        w = sel[2] - sel[0]; h = sel[3] - sel[1]

        menu = QMenu(self)
        # Header (deaktiviert, nur als Info)
        header = QAction(f"Auswahl  {w}x{h}  bei ({sel[0]}, {sel[1]})", self)
        header.setEnabled(False)
        menu.addAction(header)
        menu.addSeparator()

        menu.addAction(self.act_cut)
        menu.addAction(self.act_copy)
        menu.addAction(self.act_paste)
        menu.addSeparator()

        fill_act = QAction("Mit Vordergrundfarbe fuellen", self)
        fill_act.triggered.connect(self.action_fill_selection_fg)
        menu.addAction(fill_act)

        flip_h_act = QAction("Auswahl horizontal spiegeln", self)
        flip_h_act.triggered.connect(self.action_flip_selection_h)
        menu.addAction(flip_h_act)

        flip_v_act = QAction("Auswahl vertikal spiegeln", self)
        flip_v_act.triggered.connect(self.action_flip_selection_v)
        menu.addAction(flip_v_act)

        menu.addSeparator()
        menu.addAction(self.act_del_sel)    # Pixel im Bereich loeschen
        menu.addAction(self.act_clear_sel)  # nur Marquee aufheben

        menu.exec(global_pos)

    def action_clear_selection(self):
        self.canvas.clear_selection()

    def action_crop_to_content(self):
        """Schneidet das Sprite auf den Bounding-Box aller nicht-transparenten
        Pixel ueber alle Frames zu. Pillows getbbox() rechnet das pro Frame
        aus -- wir vereinen die Boxen, damit nichts an Frames mit anderem
        Layout abgeschnitten wird.
        """
        min_x = self.doc.width; max_x = -1
        min_y = self.doc.height; max_y = -1
        for f in self.doc.frames:
            bbox = f.pixels.getbbox()  # (x0, y0, x1_exc, y1_exc) oder None
            if bbox is None:
                continue
            x0, y0, x1, y1 = bbox
            if x0 < min_x: min_x = x0
            if y0 < min_y: min_y = y0
            if x1 - 1 > max_x: max_x = x1 - 1
            if y1 - 1 > max_y: max_y = y1 - 1
        if max_x < 0:
            QMessageBox.information(
                self, "Crop", "Sprite enthaelt nur transparente Pixel.")
            return
        new_w = max_x - min_x + 1
        new_h = max_y - min_y + 1
        if (new_w, new_h) == (self.doc.width, self.doc.height) \
                and (min_x, min_y) == (0, 0):
            self.statusBar().showMessage(
                "Keine transparenten Raender -- nichts zu croppen", 2500)
            return
        ans = QMessageBox.question(
            self, "Crop to Content",
            f"Sprite zuschneiden auf {new_w}x{new_h} "
            f"(von {self.doc.width}x{self.doc.height})?\n\n"
            f"Alle Frames werden um ({min_x}, {min_y}) verschoben "
            f"und der Undo-Verlauf der Frames geht dabei verloren.",
        )
        if ans != QMessageBox.Yes:
            return
        for f in self.doc.frames:
            f.pixels = f.pixels.crop((min_x, min_y, max_x + 1, max_y + 1)).copy()
            f.history.clear()
            f.redo_stack.clear()
        self.doc.width = new_w
        self.doc.height = new_h
        self.doc.dirty = True
        self.canvas.rebuild_scene()
        self.frames_panel.refresh()
        self.update_status()
        self.statusBar().showMessage(
            f"Zugeschnitten auf {new_w}x{new_h}", 2500)

    def action_resize_canvas(self):
        dlg = ResizeCanvasDialog(self, self.doc.width, self.doc.height)
        if dlg.exec() != QDialog.Accepted or dlg.result_size is None:
            return
        w, h = dlg.result_size
        self.doc.resize(w, h)
        self.canvas.rebuild_scene()
        self.frames_panel.refresh()
        self.update_status()

    # --- Transformationen ---

    def action_flip_horizontal(self):
        f = self.doc.current
        f.snapshot()
        f.pixels = f.pixels.transpose(Image.FLIP_LEFT_RIGHT)
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    def action_flip_vertical(self):
        f = self.doc.current
        f.snapshot()
        f.pixels = f.pixels.transpose(Image.FLIP_TOP_BOTTOM)
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    def action_rotate_cw(self):
        if self.doc.width != self.doc.height:
            if QMessageBox.question(self, "Rotation",
                f"Sprite ist {self.doc.width}x{self.doc.height}. "
                f"Drehung tauscht alle Frames auf "
                f"{self.doc.height}x{self.doc.width}. Fortfahren?"
            ) != QMessageBox.Yes:
                return
            self._rotate_all(Image.ROTATE_270)
            self.doc.width, self.doc.height = self.doc.height, self.doc.width
            self.canvas.rebuild_scene()
            self.frames_panel.refresh()
            self.update_status()
        else:
            f = self.doc.current
            f.snapshot()
            f.pixels = f.pixels.transpose(Image.ROTATE_270)
            self.canvas.invalidate_all()
            self.frames_panel.refresh()
            self.mark_dirty()

    def action_rotate_ccw(self):
        if self.doc.width != self.doc.height:
            if QMessageBox.question(self, "Rotation",
                f"Sprite ist {self.doc.width}x{self.doc.height}. "
                f"Drehung tauscht alle Frames auf "
                f"{self.doc.height}x{self.doc.width}. Fortfahren?"
            ) != QMessageBox.Yes:
                return
            self._rotate_all(Image.ROTATE_90)
            self.doc.width, self.doc.height = self.doc.height, self.doc.width
            self.canvas.rebuild_scene()
            self.frames_panel.refresh()
            self.update_status()
        else:
            f = self.doc.current
            f.snapshot()
            f.pixels = f.pixels.transpose(Image.ROTATE_90)
            self.canvas.invalidate_all()
            self.frames_panel.refresh()
            self.mark_dirty()

    def _rotate_all(self, op):
        for f in self.doc.frames:
            f.pixels = f.pixels.transpose(op)
            f.history.clear()
            f.redo_stack.clear()
        self.doc.dirty = True

    # --- Frames ---

    def action_frame_select(self, idx: int):
        if 0 <= idx < len(self.doc.frames):
            self.doc.select(idx)
            self.canvas.invalidate_all()
            self.frames_panel.refresh()
            self.update_status()

    def action_frame_add(self, copy_current: bool):
        self.doc.add_frame(copy_current=copy_current)
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.update_status()

    def action_frame_delete(self):
        if not self.doc.delete_frame():
            QMessageBox.information(self, "Letztes Frame",
                                    "Mindestens ein Frame muss bleiben.")
            return
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.update_status()

    def action_frame_move(self, delta: int):
        if self.doc.move_frame(delta):
            self.frames_panel.refresh()
            self.update_status()

    def action_reverse_frames(self):
        """Kehrt die Reihenfolge aller Frames um."""
        n = len(self.doc.frames)
        if n <= 1:
            return
        self.doc.frames.reverse()
        # current_index nachpflegen
        self.doc.current_index = n - 1 - self.doc.current_index
        self.doc.dirty = True
        self.canvas.invalidate_all()
        self.canvas._render_onion()
        self.frames_panel.refresh()
        self.update_status()
        self.statusBar().showMessage("Frames umgekehrt", 1500)

    def action_flatten_to_current(self):
        """Behaelt nur den aktuellen Frame, loescht alle anderen.

        Use-Case: User hat versehentlich aus einem Sheet 4 Frames
        importiert, wollte aber nur einen.
        """
        if len(self.doc.frames) <= 1:
            return
        ans = QMessageBox.question(
            self, "Auf aktuelles Frame reduzieren",
            f"Alle {len(self.doc.frames) - 1} anderen Frames werden geloescht.\n"
            f"Nur Frame #{self.doc.current_index:02d} bleibt erhalten.\n\n"
            f"Fortfahren?",
        )
        if ans != QMessageBox.Yes:
            return
        kept = self.doc.frames[self.doc.current_index]
        self.doc.frames = [kept]
        self.doc.current_index = 0
        self.doc.dirty = True
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.update_status()

    def action_flatten_composite(self):
        """Fuehrt alle Frames per Alpha-Composite zusammen -- einer ueber
        dem anderen, in Frame-Reihenfolge. Ergibt 1 Frame mit allen
        Pixeln uebereinandergelegt.
        """
        if len(self.doc.frames) <= 1:
            return
        ans = QMessageBox.question(
            self, "Frames zusammenfuegen",
            f"Alle {len(self.doc.frames)} Frames werden per Alpha-Composite "
            f"in einen einzigen Frame gemerged.\n\n"
            f"Fortfahren?",
        )
        if ans != QMessageBox.Yes:
            return
        merged = Image.new("RGBA",
                            (self.doc.width, self.doc.height),
                            (0, 0, 0, 0))
        for f in self.doc.frames:
            merged.alpha_composite(f.pixels)
        self.doc.frames = [Frame(pixels=merged)]
        self.doc.current_index = 0
        self.doc.dirty = True
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.update_status()

    def action_pingpong_frames(self):
        """Erzeugt einen Ping-Pong-Loop: haengt frames[1..n-2] in
        umgekehrter Reihenfolge an die Liste an. Aus [A,B,C,D] wird
        [A,B,C,D,C,B] -- Endlosschleife schwingt vor und zurueck.
        """
        n = len(self.doc.frames)
        if n <= 2:
            self.statusBar().showMessage(
                "Ping-Pong braucht mindestens 3 Frames", 2500)
            return
        # frames[1..n-2] in umgekehrter Reihenfolge an Ende anhaengen,
        # mit Pixel-Kopie damit die Original-Frames nicht aliased werden
        # (sonst wuerde Bearbeiten von 'B' auch das angehaengte 'B' aendern).
        new_tail = []
        for f in reversed(self.doc.frames[1:-1]):
            new_tail.append(Frame(pixels=f.pixels.copy(),
                                   duration_ms=f.duration_ms))
        self.doc.frames.extend(new_tail)
        self.doc.dirty = True
        self.frames_panel.refresh()
        self.update_status()
        self.statusBar().showMessage(
            f"Ping-Pong: {len(self.doc.frames)} Frames "
            f"({n} original + {len(new_tail)} reversed)", 3000)

    def action_play_animation(self):
        if not self.doc.frames:
            return
        dlg = AnimationPreview(self, self.doc.frames,
                               self.doc.width, self.doc.height)
        dlg.show()

    # --- Hilfe ---

    def show_statistics(self):
        """Zeigt einen Dialog mit Pixel-/Farb-Statistik des aktuellen
        Sprites: pro Frame und ueber alle Frames aggregiert."""
        from collections import Counter

        n_frames = len(self.doc.frames)
        total_pixels = self.doc.width * self.doc.height * n_frames

        all_colors = Counter()
        opaque_total = 0
        for f in self.doc.frames:
            for col in f.pixels.getdata():
                all_colors[col] += 1
                if col[3] >= 128:
                    opaque_total += 1

        unique_total = len(all_colors)
        # Dominanteste opake Farbe (transparente ignorieren)
        dom_color = None
        dom_count = 0
        for col, cnt in all_colors.most_common():
            if col[3] >= 128:
                dom_color = col
                dom_count = cnt
                break

        # Pro Frame: Pixel-Anzahl + eindeutige Farben
        per_frame = []
        for i, f in enumerate(self.doc.frames):
            colors = Counter(f.pixels.getdata())
            opaque = sum(c for col, c in colors.items() if col[3] >= 128)
            per_frame.append((i, opaque, len([col for col in colors
                                                if col[3] >= 128])))

        transparent_pct = 100.0 * (1.0 - opaque_total / max(1, total_pixels))

        lines = [
            f"<b>Sprite-Statistik</b>",
            f"",
            f"<b>Gesamt</b>",
            f"&nbsp;&nbsp;Groesse: {self.doc.width} x {self.doc.height} px",
            f"&nbsp;&nbsp;Frames: {n_frames}",
            f"&nbsp;&nbsp;Gesamt-Pixel: {total_pixels}",
            f"&nbsp;&nbsp;Davon opak: {opaque_total} "
            f"({100.0 - transparent_pct:.1f}%)",
            f"&nbsp;&nbsp;Davon transparent: {total_pixels - opaque_total} "
            f"({transparent_pct:.1f}%)",
            f"&nbsp;&nbsp;Eindeutige Farben (inkl. Alpha-Stufen): {unique_total}",
        ]
        if dom_color:
            r, g, b, a = dom_color
            lines.append(
                f"&nbsp;&nbsp;Dominanteste Farbe: "
                f"<span style='font-family: Consolas;'>"
                f"#{r:02x}{g:02x}{b:02x}{a:02x}</span> "
                f"({dom_count} Pixel)")

        if n_frames > 1:
            lines += [
                f"",
                f"<b>Pro Frame</b>",
            ]
            for i, opaque, uniq in per_frame:
                lines.append(
                    f"&nbsp;&nbsp;#{i:02d}: {opaque} opake Pixel, "
                    f"{uniq} eindeutige Farben")

        text = "<br>".join(lines)
        QMessageBox.information(self, "Sprite-Statistik", text)

    def show_shortcuts(self):
        text = (
            "Tools:\n"
            "  B   Stift          E   Radierer\n"
            "  G   Eimer          L   Linie\n"
            "  R   Rechteck       Shift+R Rechteck gefuellt\n"
            "  O   Ellipse        Shift+O Ellipse gefuellt\n"
            "  I   Pipette        M   Auswahl\n"
            "  V   Verschieben    X   Farben tauschen\n\n"
            "Zoom & View:\n"
            "  +/-  Zoom in/out   0   100% reset\n"
            "  Mausrad           Zoom unter Cursor\n"
            "  T                 Tile-Vorschau (3x3)\n\n"
            "Datei:\n"
            "  Strg+N Neu         Strg+O Oeffnen\n"
            "  Strg+S Speichern   Strg+Shift+S Speichern unter\n"
            "  Strg+E Sheet-PNG exportieren\n"
            "  Strg+Shift+C  GameBasic-Code in Zwischenablage\n\n"
            "Bearbeiten:\n"
            "  Strg+Z Undo        Strg+Y Redo\n"
            "  Strg+X Cut         Strg+C Copy   Strg+V Paste\n"
            "  Strg+H Flip H      Strg+J Flip V\n"
            "  Strg+. 90 rechts   Strg+, 90 links\n"
            "  Strg+Shift+X  Symmetrie X umschalten\n"
            "  Strg+Shift+Y  Symmetrie Y umschalten\n"
            "  Entf   Auswahl loeschen   Esc Auswahl aufheben\n\n"
            "Brush:\n"
            "  1..4   Pixel-Groesse fuer Stift/Radierer\n\n"
            "Frames:\n"
            "  Strg+P Animation-Preview\n"
            "  F2..F9 Frame 0..7 schnell waehlen"
        )
        QMessageBox.information(self, "Tastenkuerzel", text)

    def show_about(self):
        QMessageBox.about(
            self, "Ueber GameBasic Sprite-Editor",
            "GameBasic Sprite-Editor (PySide6)\n\n"
            "Pixel-Art-Editor mit Alpha-Kanal und Frame-Animation.\n"
            "Output: PNG (Alpha) und .gbsprite (natives Format).\n\n"
            "PNG-Sheets sind direkt mit dem sprite-Modul verwendbar:\n"
            "    img = LOADIMAGE(\"sheet.png\")\n"
            "    sp  = SPRITE_NEW(img, frame_w, frame_h)"
        )

    # --- Window-Events ---

    def keyPressEvent(self, event):
        # Pfeiltasten verschieben den Auswahl-Inhalt um einen Pixel.
        # Damit sie sonst nichts ueberschreiben, reagieren sie nur wenn
        # gerade eine Auswahl aktiv ist.
        if (self.canvas.get_selection() is not None
                and event.key() in (Qt.Key_Left, Qt.Key_Right,
                                     Qt.Key_Up, Qt.Key_Down)):
            dx = -1 if event.key() == Qt.Key_Left else (
                  1 if event.key() == Qt.Key_Right else 0)
            dy = -1 if event.key() == Qt.Key_Up else (
                  1 if event.key() == Qt.Key_Down else 0)
            self._move_selection_content(dx, dy)
            event.accept()
            return
        super().keyPressEvent(event)

    def _move_selection_content(self, dx: int, dy: int):
        """Verschiebt den Inhalt der aktuellen Auswahl um (dx, dy) Pixel
        und folgt der Auswahl mit. Nutzt PIL.Image.crop + alpha_composite
        statt putpixel-Schleifen -- viel schneller bei groesseren
        Selektionen."""
        sel = self.canvas.get_selection()
        if not sel:
            return
        x0, y0, x1, y1 = sel  # x1, y1 exklusiv
        sw, sh = x1 - x0, y1 - y0
        # Neue Position auf Sprite-Bounds clampen
        nx0 = max(0, min(self.doc.width - sw, x0 + dx))
        ny0 = max(0, min(self.doc.height - sh, y0 + dy))
        if (nx0, ny0) == (x0, y0):
            return  # am Rand, keine Bewegung
        self.doc.current.snapshot()
        cropped = self.doc.current.pixels.crop((x0, y0, x1, y1)).copy()
        d = ImageDraw.Draw(self.doc.current.pixels)
        d.rectangle([x0, y0, x1 - 1, y1 - 1], fill=(0, 0, 0, 0))
        self.doc.current.pixels.alpha_composite(cropped, dest=(nx0, ny0))
        self.canvas.set_selection(nx0, ny0, nx0 + sw - 1, ny0 + sh - 1)
        self.canvas.invalidate_all()
        self.frames_panel.refresh()
        self.mark_dirty()

    def closeEvent(self, event):
        if self._confirm_dirty():
            # Sauberes Beenden: Backup-Datei wegputzen
            self._delete_backup()
            event.accept()
        else:
            event.ignore()

    # --- File-Watcher / Reload-from-Disk ---

    def _watch_current_file(self):
        """Hebt den File-Watch fuer alle bisherigen Pfade auf und setzt
        ihn auf den aktuellen filepath neu. Nach jedem Load oder Save
        aufrufen."""
        try:
            old = self._fs_watcher.files()
            if old:
                self._fs_watcher.removePaths(old)
        except Exception:
            pass
        if self.doc.filepath is not None and self.doc.filepath.exists():
            try:
                self._fs_watcher.addPath(str(self.doc.filepath))
            except Exception:
                pass

    def _on_file_changed(self, path: str):
        """Wird vom QFileSystemWatcher gefeuert, wenn die Datei extern
        geaendert wurde. Eigene Save-Operationen unterdruecken wir per
        Zeit-Cooldown -- sonst flackert die Warnung nach jedem Save."""
        import time
        if time.time() < self._suppress_watcher_until:
            # Re-Add: manche Editoren schreiben atomic (rename) und
            # invalidieren den Watcher dabei.
            if path not in self._fs_watcher.files():
                try:
                    self._fs_watcher.addPath(path)
                except Exception:
                    pass
            return
        # User entscheiden lassen
        if self.doc.dirty:
            ans = QMessageBox.question(
                self, "Datei extern geaendert",
                f"{Path(path).name} wurde ausserhalb des Editors geaendert.\n"
                f"Du hast aber ungespeicherte Aenderungen.\n\n"
                f"Externe Version laden (eigene verwerfen)?",
            )
        else:
            ans = QMessageBox.question(
                self, "Datei extern geaendert",
                f"{Path(path).name} wurde ausserhalb des Editors geaendert.\n\n"
                f"Neu laden?",
            )
        if ans == QMessageBox.Yes:
            self.action_reload_from_disk()
        else:
            # Watcher neu setzen (atomic-saves zerstoeren ihn)
            self._watch_current_file()

    def action_reload_from_disk(self):
        """Laedt die aktuelle Datei neu von der Festplatte. Nuetzlich
        wenn ein anderes Tool sie geaendert hat oder nach einem
        externen git pull."""
        if self.doc.filepath is None:
            self.statusBar().showMessage(
                "Reload geht nur fuer gespeicherte Dateien", 2000)
            return
        if not self.doc.filepath.exists():
            QMessageBox.warning(self, "Reload",
                                 f"Datei nicht mehr vorhanden: {self.doc.filepath}")
            return
        path = self.doc.filepath
        # _load_path setzt self.doc neu, ruft refresh
        self._load_path(path)
        self.statusBar().showMessage(f"Neu geladen: {path.name}", 2500)

    # --- Auto-Backup ---
    # Alle 60s wird ein Snapshot in eine '.bak'-Datei geschrieben, wenn
    # das Doc dirty ist. Bei sauberem Schliessen oder erfolgreichem
    # Speichern wird die Backup-Datei wieder geloescht.

    @staticmethod
    def _autobackup_orphan_path() -> Path:
        """Pfad fuer ein noch-nicht-gespeichertes Sprite (kein filepath).
        Liegt im Home-Verzeichnis, damit es persistent ueber Sessions ist."""
        d = Path.home() / ".gamebasic"
        try:
            d.mkdir(exist_ok=True)
        except Exception:
            pass
        return d / "autobackup.gbsprite"

    def _backup_path_for_doc(self) -> Optional[Path]:
        if self.doc.filepath is not None:
            # Sidecar-File neben dem Original
            return self.doc.filepath.with_suffix(
                self.doc.filepath.suffix + ".bak")
        return self._autobackup_orphan_path()

    def _auto_backup(self):
        if not self.doc.dirty:
            return
        path = self._backup_path_for_doc()
        if path is None:
            return
        # save_native ueberschreibt self.doc.filepath / dirty -- daher
        # den Pre-State sichern und nach dem Schreiben wiederherstellen,
        # damit das Backup transparent ist.
        orig_path = self.doc.filepath
        try:
            self.doc.save_native(path)
        except Exception:
            return
        finally:
            self.doc.filepath = orig_path
            self.doc.dirty = True

    def _delete_backup(self):
        path = self._backup_path_for_doc()
        if path is not None and path.exists():
            try:
                path.unlink()
            except Exception:
                pass

    def _maybe_offer_restore(self):
        """Beim Start: pruefen ob ein Orphan-Auto-Backup existiert (z.B.
        nach Crash). Wenn ja, dem User Restore anbieten."""
        bak = self._autobackup_orphan_path()
        if not bak.exists():
            return
        # Pragmatisch: nur anbieten wenn der Backup juenger als 30 Tage
        # ist (alte Backups sind selten erwuenscht).
        try:
            import time
            age_days = (time.time() - bak.stat().st_mtime) / 86400
        except Exception:
            age_days = 0
        if age_days > 30:
            return
        ans = QMessageBox.question(
            self, "Auto-Backup gefunden",
            f"Es existiert ein nicht-gespeichertes Auto-Backup\n({bak})\n\n"
            f"Wiederherstellen?",
        )
        if ans == QMessageBox.Yes:
            try:
                self.doc = SpriteDoc.load_native(bak)
                # Filepath zuruecksetzen -- der User soll es bewusst
                # speichern, sonst bleibt's ein "ungespeichert"-Doc.
                self.doc.filepath = None
                self.doc.dirty = True
                self.canvas.rebuild_scene()
                self.frames_panel.refresh()
                self.color_panel.refresh()
                self.update_status()
                self.statusBar().showMessage(
                    "Auto-Backup wiederhergestellt -- bitte speichern.",
                    5000)
            except Exception as exc:
                QMessageBox.warning(
                    self, "Restore fehlgeschlagen",
                    f"Konnte Backup nicht laden:\n{exc}")
        else:
            # User lehnt ab -> Backup loeschen
            try:
                bak.unlink()
            except Exception:
                pass

    # --- Drag-and-Drop von Sprite-Dateien ---

    @staticmethod
    def _is_supported_drop(path: Path) -> bool:
        return path.suffix.lower() in (".png", ".gbsprite")

    def dragEnterEvent(self, event):
        md = event.mimeData()
        if md.hasUrls() and any(
            self._is_supported_drop(Path(u.toLocalFile()))
            for u in md.urls()
            if u.toLocalFile()
        ):
            event.acceptProposedAction()
        else:
            event.ignore()

    def dragMoveEvent(self, event):
        # Sonst flackert der Cursor zwischen "akzeptiert" und "verweigert"
        if event.mimeData().hasUrls():
            event.acceptProposedAction()

    def dropEvent(self, event):
        for url in event.mimeData().urls():
            local = url.toLocalFile()
            if not local:
                continue
            path = Path(local)
            if not self._is_supported_drop(path):
                continue
            if not self._confirm_dirty():
                event.ignore()
                return
            self._load_path(path)
            event.acceptProposedAction()
            return
        event.ignore()


# QActionGroup separat importieren (vermeidet Symbol-Konflikt im Try/Except oben)
from PySide6.QtGui import QActionGroup  # noqa: E402


# ============================================================
# Sub-Dialoge
# ============================================================

class ResizeCanvasDialog(QDialog):
    """Canvas-Resize mit Presets, Spinboxes und Vorschau-Hinweis.
    Ersetzt die alte Texteingabe -- wesentlich angenehmer fuer den
    haeufigen "Sprite verdoppeln" oder "Padding hinzufuegen"-Workflow.

    Pixel-Daten werden bei Resize an der Top-Left-Ecke verankert.
    Vergroessern -> rechts/unten transparent gepadded; Verkleinern ->
    rechts/unten abgeschnitten.
    """
    PRESETS = [
        ("8 x 8",      8,   8),
        ("16 x 16",    16,  16),
        ("24 x 24",    24,  24),
        ("32 x 32",    32,  32),
        ("48 x 48",    48,  48),
        ("64 x 64",    64,  64),
        ("96 x 96",    96,  96),
        ("128 x 128",  128, 128),
        ("16 x 32 (Charakter)",   16, 32),
        ("32 x 16 (Schiff)",      32, 16),
        ("64 x 32 (Banner)",      64, 32),
        ("256 x 256",  256, 256),
    ]

    def __init__(self, parent, current_w: int, current_h: int):
        super().__init__(parent)
        self.setWindowTitle("Canvas-Groesse")
        self.setMinimumWidth(380)
        self.result_size: Optional[tuple[int, int]] = None
        self._cur_w = current_w
        self._cur_h = current_h

        layout = QVBoxLayout(self)
        layout.setContentsMargins(20, 20, 20, 20)
        layout.setSpacing(10)

        layout.addWidget(QLabel(
            f"<b>Aktuell:</b> {current_w} x {current_h} px"
        ))

        # Spinboxes
        sb_row = QHBoxLayout()
        sb_row.addWidget(QLabel("Breite:"))
        self.spin_w = QSpinBox()
        self.spin_w.setRange(1, 1024)
        self.spin_w.setValue(current_w)
        sb_row.addWidget(self.spin_w)
        sb_row.addSpacing(16)
        sb_row.addWidget(QLabel("Hoehe:"))
        self.spin_h = QSpinBox()
        self.spin_h.setRange(1, 1024)
        self.spin_h.setValue(current_h)
        sb_row.addWidget(self.spin_h)
        sb_row.addStretch()
        layout.addLayout(sb_row)

        # Presets
        layout.addSpacing(8)
        layout.addWidget(QLabel("<b>Presets</b>"))
        presets_grid = QGridLayout()
        presets_grid.setSpacing(4)
        for i, (label, w, h) in enumerate(self.PRESETS):
            r, c = divmod(i, 4)
            btn = QPushButton(label)
            btn.clicked.connect(lambda _ck=False, ww=w, hh=h: self._apply_preset(ww, hh))
            presets_grid.addWidget(btn, r, c)
        layout.addLayout(presets_grid)

        # Hinweis
        layout.addSpacing(6)
        self.info = QLabel(self._info_text())
        self.info.setStyleSheet(f"color: {COLORS['fg_muted']}; "
                                 f"font-family: Consolas;")
        self.info.setWordWrap(True)
        layout.addWidget(self.info)
        self.spin_w.valueChanged.connect(lambda _v: self._update_info())
        self.spin_h.valueChanged.connect(lambda _v: self._update_info())

        # OK/Cancel
        layout.addSpacing(8)
        btn_row = QHBoxLayout()
        btn_row.addStretch()
        cancel = QPushButton("Abbrechen")
        cancel.clicked.connect(self.reject)
        btn_row.addWidget(cancel)
        ok = QPushButton("Aendern")
        ok.setObjectName("primary")
        ok.setDefault(True)
        ok.clicked.connect(self._accept)
        btn_row.addWidget(ok)
        layout.addLayout(btn_row)

    def _apply_preset(self, w: int, h: int):
        self.spin_w.setValue(w)
        self.spin_h.setValue(h)

    def _update_info(self):
        self.info.setText(self._info_text())

    def _info_text(self) -> str:
        new_w = self.spin_w.value()
        new_h = self.spin_h.value()
        if new_w == self._cur_w and new_h == self._cur_h:
            return "Keine Aenderung."
        dx = new_w - self._cur_w
        dy = new_h - self._cur_h
        msg_parts = []
        if dx > 0:
            msg_parts.append(f"+{dx}px rechts")
        elif dx < 0:
            msg_parts.append(f"{dx}px rechts (abgeschnitten)")
        if dy > 0:
            msg_parts.append(f"+{dy}px unten")
        elif dy < 0:
            msg_parts.append(f"{dy}px unten (abgeschnitten)")
        return ("Pixel-Anker oben links. " + ", ".join(msg_parts) +
                ".\nUndo-Verlauf der Frames wird geloescht.")

    def _accept(self):
        w = self.spin_w.value()
        h = self.spin_h.value()
        if (w, h) == (self._cur_w, self._cur_h):
            self.reject()
            return
        self.result_size = (w, h)
        self.accept()


class SheetImportDialog(QDialog):
    """Sheet-Import mit Live-Vorschau: zeigt das geladene PNG mit einem
    rote-Linien-Grid drueber, das die zu erkennenden Frame-Grenzen
    darstellt. User kann frame_w/frame_h anpassen und sieht sofort, ob
    das Sheet korrekt zerlegt wird.

    "Single Frame"-Knopf laedt das Bild als ein einziges Frame ohne
    Zerlegung -- klassischer Use-Case fuer ein bereits zugeschnittenes
    PNG.
    """

    def __init__(self, parent, image: Image.Image):
        super().__init__(parent)
        self.setWindowTitle("Sheet-Import")
        self.setMinimumWidth(540)
        self._image = image
        self._iw, self._ih = image.size
        self.result_size: Optional[tuple[int, int]] = None
        self.single_frame = False

        layout = QVBoxLayout(self)
        layout.setContentsMargins(20, 20, 20, 20)
        layout.setSpacing(10)

        info = QLabel(f"<b>Bildgroesse:</b> {self._iw} x {self._ih} px")
        layout.addWidget(info)

        # Heuristik: wenn iw % ih == 0 und iw // ih > 1, ist's ein
        # horizontaler Sheet mit ih als Frame-Hoehe.
        if self._iw % self._ih == 0 and self._iw // self._ih > 1:
            sg_w, sg_h = self._ih, self._ih
        else:
            sg_w = sg_h = min(self._iw, self._ih)
        sg_w = max(1, min(sg_w, self._iw))
        sg_h = max(1, min(sg_h, self._ih))

        # Spinboxes
        sb_row = QHBoxLayout()
        sb_row.addWidget(QLabel("Frame-Breite:"))
        self.spin_w = QSpinBox()
        self.spin_w.setRange(1, self._iw)
        self.spin_w.setValue(sg_w)
        self.spin_w.valueChanged.connect(self._update_preview)
        sb_row.addWidget(self.spin_w)
        sb_row.addSpacing(16)
        sb_row.addWidget(QLabel("Frame-Hoehe:"))
        self.spin_h = QSpinBox()
        self.spin_h.setRange(1, self._ih)
        self.spin_h.setValue(sg_h)
        self.spin_h.valueChanged.connect(self._update_preview)
        sb_row.addWidget(self.spin_h)
        sb_row.addStretch()
        layout.addLayout(sb_row)

        self.info_count = QLabel("")
        self.info_count.setStyleSheet(f"color: {COLORS['fg_muted']}; "
                                       f"font-family: Consolas;")
        layout.addWidget(self.info_count)

        # Vorschau (Schachbrett + skalierte Bild-Pixmap + rotes Grid)
        self.preview = QLabel()
        self.preview.setAlignment(Qt.AlignCenter)
        self.preview.setMinimumHeight(280)
        self.preview.setStyleSheet(
            f"background: {COLORS['bg']}; "
            f"border: 1px solid {COLORS['border']};"
        )
        layout.addWidget(self.preview, 1)

        # Buttons
        btn_row = QHBoxLayout()
        btn_row.addStretch()
        cancel = QPushButton("Abbrechen")
        cancel.clicked.connect(self.reject)
        btn_row.addWidget(cancel)
        single = QPushButton("Als Single-Frame laden")
        single.clicked.connect(self._pick_single)
        btn_row.addWidget(single)
        ok = QPushButton("Als Sheet zerlegen")
        ok.setObjectName("primary")
        ok.setDefault(True)
        ok.clicked.connect(self._pick_sheet)
        btn_row.addWidget(ok)
        layout.addLayout(btn_row)

        self._update_preview()

    def _update_preview(self):
        fw = self.spin_w.value()
        fh = self.spin_h.value()
        cols = max(1, self._iw // fw)
        rows = max(1, self._ih // fh)
        n = cols * rows

        # Konsistenz-Hinweis: passt fw/fh exakt ohne Rest?
        rest = ""
        if self._iw % fw or self._ih % fh:
            rest = "  (PNG wird auf gerade Frames-Grenze beschnitten!)"
        self.info_count.setText(
            f"-> {cols} x {rows} = {n} Frames a {fw}x{fh}{rest}")

        # Skaliere Bild auf max ~480 breit, ~280 hoch (Vorschau-Bereich)
        max_w, max_h = 480, 260
        ratio = min(max_w / self._iw, max_h / self._ih, 8.0)
        # Nur ganze Pixel-Skalierung zulassen wenn sehr klein, sonst
        # Float ratio fuer smoothes Resize
        scaled_w = max(1, int(self._iw * ratio))
        scaled_h = max(1, int(self._ih * ratio))
        scaled = self._image.resize((scaled_w, scaled_h), Image.NEAREST)
        # Schachbrett-Hintergrund
        bg = Image.new("RGBA", (scaled_w, scaled_h), (0, 0, 0, 0))
        d = ImageDraw.Draw(bg)
        for yy in range(0, scaled_h, 8):
            for xx in range(0, scaled_w, 8):
                if (xx // 8 + yy // 8) % 2 == 0:
                    d.rectangle([xx, yy, xx + 8, yy + 8],
                                fill=(42, 42, 42, 255))
                else:
                    d.rectangle([xx, yy, xx + 8, yy + 8],
                                fill=(58, 58, 58, 255))
        bg.alpha_composite(scaled)
        # Rotes Grid drueber
        d2 = ImageDraw.Draw(bg)
        for c in range(1, cols):
            x = int(c * fw * ratio)
            d2.line([(x, 0), (x, scaled_h)], fill=(255, 80, 80, 220), width=1)
        for r in range(1, rows):
            y = int(r * fh * ratio)
            d2.line([(0, y), (scaled_w, y)], fill=(255, 80, 80, 220), width=1)
        # Aussenrand
        d2.rectangle([0, 0, scaled_w - 1, scaled_h - 1],
                     outline=(255, 80, 80, 200), width=1)
        self.preview.setPixmap(pil_to_qpixmap(bg))

    def _pick_sheet(self):
        self.result_size = (self.spin_w.value(), self.spin_h.value())
        self.single_frame = False
        self.accept()

    def _pick_single(self):
        self.result_size = (self._iw, self._ih)
        self.single_frame = True
        self.accept()


class ColorReplaceDialog(QDialog):
    """Dialog: Quell-Farbe waehlen, Ziel-Farbe waehlen, Scope-Optionen.

    Source-Default = aktuelle Vordergrundfarbe der App. User kann beide
    anklicken um den Color-Picker zu oeffnen.
    """

    def __init__(self, parent: "SpriteEditorWindow"):
        super().__init__(parent)
        self.setWindowTitle("Farbe ersetzen")
        self.setMinimumWidth(360)
        self._src = parent.fg
        # Default-Ziel: leicht abgedunkelte Variante der Quelle, damit's
        # sichtbar ist.
        self._dst = (max(0, parent.fg[0] - 60),
                     max(0, parent.fg[1] - 60),
                     max(0, parent.fg[2] - 60),
                     parent.fg[3])

        layout = QVBoxLayout(self)
        layout.setContentsMargins(20, 20, 20, 20)
        layout.setSpacing(12)

        layout.addWidget(QLabel("<b>Quelle</b>  (anklicken zum Aendern)"))
        self.src_box = _ColorBox(54)
        self.src_box.set_color(self._src)
        self.src_box.clicked.connect(self._pick_src)
        src_row = QHBoxLayout()
        src_row.addWidget(self.src_box)
        src_row.addStretch()
        layout.addLayout(src_row)

        layout.addWidget(QLabel("<b>Ziel</b>"))
        self.dst_box = _ColorBox(54)
        self.dst_box.set_color(self._dst)
        self.dst_box.clicked.connect(self._pick_dst)
        dst_row = QHBoxLayout()
        dst_row.addWidget(self.dst_box)
        dst_row.addStretch()
        layout.addLayout(dst_row)

        layout.addSpacing(8)
        self.cb_selection = QCheckBox("Nur in der aktuellen Auswahl")
        sel = parent.canvas.get_selection()
        self.cb_selection.setEnabled(sel is not None)
        if sel is None:
            self.cb_selection.setToolTip(
                "Keine Auswahl aktiv -- mit M markieren, um diese Option "
                "zu nutzen")
        layout.addWidget(self.cb_selection)

        self.cb_all_frames = QCheckBox(
            f"In allen {len(parent.doc.frames)} Frames anwenden")
        self.cb_all_frames.setEnabled(len(parent.doc.frames) > 1)
        layout.addWidget(self.cb_all_frames)

        # OK / Cancel
        layout.addSpacing(8)
        btn_row = QHBoxLayout()
        btn_row.addStretch()
        cancel = QPushButton("Abbrechen")
        cancel.clicked.connect(self.reject)
        btn_row.addWidget(cancel)
        ok = QPushButton("Ersetzen")
        ok.setObjectName("primary")
        ok.clicked.connect(self.accept)
        ok.setDefault(True)
        btn_row.addWidget(ok)
        layout.addLayout(btn_row)

    def _pick_src(self):
        c = self._src
        qc = QColorDialog.getColor(QColor(c[0], c[1], c[2], c[3]),
                                   self, "Quellfarbe",
                                   QColorDialog.ShowAlphaChannel)
        if qc.isValid():
            self._src = (qc.red(), qc.green(), qc.blue(), qc.alpha())
            self.src_box.set_color(self._src)

    def _pick_dst(self):
        c = self._dst
        qc = QColorDialog.getColor(QColor(c[0], c[1], c[2], c[3]),
                                   self, "Zielfarbe",
                                   QColorDialog.ShowAlphaChannel)
        if qc.isValid():
            self._dst = (qc.red(), qc.green(), qc.blue(), qc.alpha())
            self.dst_box.set_color(self._dst)

    def source_color(self): return self._src
    def target_color(self): return self._dst
    def only_in_selection(self): return self.cb_selection.isChecked()
    def all_frames(self): return self.cb_all_frames.isChecked()


class NewSpriteDialog(QDialog):
    PRESETS = [
        ("Icon",    8,  8),
        ("Sprite",  16, 16),
        ("Avatar",  32, 32),
        ("Tile",    32, 32),
        ("Charakter", 64, 64),
    ]

    def __init__(self, parent):
        super().__init__(parent)
        self.setWindowTitle("Neues Sprite")
        self.setMinimumWidth(340)
        self.result_size: Optional[tuple[int, int]] = None

        layout = QVBoxLayout(self)
        layout.setContentsMargins(20, 20, 20, 20)
        layout.setSpacing(8)

        layout.addWidget(QLabel("<b>Vorlage waehlen:</b>"))
        for name, w, h in self.PRESETS:
            btn = QPushButton(f"{name}  ({w}x{h})")
            btn.clicked.connect(lambda _ck=False, ww=w, hh=h: self._pick(ww, hh))
            layout.addWidget(btn)

        layout.addSpacing(12)
        layout.addWidget(QLabel("<b>Custom (WxH):</b>"))
        row = QHBoxLayout()
        self.entry = QLineEdit()
        self.entry.setPlaceholderText("z.B. 24x24")
        self.entry.returnPressed.connect(self._pick_custom)
        row.addWidget(self.entry)
        ok_btn = QPushButton("OK")
        ok_btn.setObjectName("primary")
        ok_btn.clicked.connect(self._pick_custom)
        row.addWidget(ok_btn)
        layout.addLayout(row)

        layout.addSpacing(12)
        cancel = QPushButton("Abbrechen")
        cancel.clicked.connect(self.reject)
        layout.addWidget(cancel)

    def _pick(self, w, h):
        self.result_size = (w, h)
        self.accept()

    def _pick_custom(self):
        s = self.entry.text().strip().lower()
        try:
            w, h = map(int, s.split("x"))
        except ValueError:
            QMessageBox.warning(self, "Ungueltig", "Format: WxH, z.B. 16x16")
            return
        if not (1 <= w <= 1024 and 1 <= h <= 1024):
            QMessageBox.warning(self, "Ungueltig", "Bereich: 1..1024")
            return
        self.result_size = (w, h)
        self.accept()


# ============================================================
# Entry-Points
# ============================================================

_app_instance: Optional[QApplication] = None


def _ensure_app() -> QApplication:
    """Liefert die QApplication-Singleton, erstellt sie falls noetig."""
    global _app_instance
    app = QApplication.instance()
    if app is None:
        app = QApplication(sys.argv)
        app.setStyle("Fusion")
        app.setStyleSheet(GLOBAL_QSS)
        _app_instance = app
    return app


def launch(project_root: Path, initial_file: Optional[Path] = None) -> int:
    """Startet den Sprite-Editor als eigenstaendige App. Liefert Exit-Code."""
    app = _ensure_app()
    win = SpriteEditorWindow(project_root, initial_file)
    win.show()
    return app.exec()
