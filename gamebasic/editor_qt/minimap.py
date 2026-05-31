"""Minimap rechts neben dem Editor.

Pro Buffer-Zeile zeichnen wir eine kurze farbige Linie (Indent-
Verschoben, Laenge ~ Anzahl Nicht-Leerzeichen). Darueber liegt das
Viewport-Rechteck, das den aktuell sichtbaren Editor-Bereich zeigt.
Click + Drag scrollt den Editor: ein y-Pixel auf der Minimap entspricht
einer Buffer-Zeile, geteilt durch das Skalierungsverhaeltnis.

Performance: bei jedem `textChanged`-Schub wird ein 150-ms-Debounce
ausgeloest, dann wird das gesamte Pixmap neu gemalt. Fuer Dateien mit
mehreren tausend Zeilen ist das immer noch fluessig genug; ein
inkrementeller Re-Paint waere die Optimierung der Wahl, aber nicht
hier in Phase 3 noetig.
"""
from __future__ import annotations

from PySide6.QtCore import QRect, Qt, QTimer
from PySide6.QtGui import QColor, QMouseEvent, QPainter, QPen
from PySide6.QtWidgets import QPlainTextEdit, QWidget

from .theme import COLORS, theme_signals


# Skalierung: 1 Buffer-Zeile = `LINE_HEIGHT_PX` Minimap-Pixel.
LINE_HEIGHT_PX = 2
# Maximale Pixel-Breite eines Strichs (Code-Zeilen werden auf diese Breite
# normalisiert).
MAX_LINE_WIDTH = 90
MINIMAP_WIDTH = 110


class Minimap(QWidget):
    """Schmales Widget rechts vom Editor; Pixmap-basiert."""

    def __init__(self, editor: QPlainTextEdit, parent: QWidget | None = None):
        super().__init__(parent)
        self.editor = editor
        self.setFixedWidth(MINIMAP_WIDTH)
        self.setMouseTracking(True)
        self.setCursor(Qt.CursorShape.OpenHandCursor)

        self._lines: list[tuple[int, int]] = []  # (indent_chars, content_chars)
        self._dragging = False

        # Debounce: bei textChanged 150 ms warten, dann re-render.
        self._refresh_timer = QTimer(self)
        self._refresh_timer.setSingleShot(True)
        self._refresh_timer.setInterval(150)
        self._refresh_timer.timeout.connect(self._rebuild_lines)

        editor.textChanged.connect(self._refresh_timer.start)
        editor.verticalScrollBar().valueChanged.connect(self.update)
        theme_signals.changed.connect(self._on_theme_changed)

        self._rebuild_lines()

    def _on_theme_changed(self, _name: str) -> None:
        self.update()

    def _rebuild_lines(self) -> None:
        text = self.editor.toPlainText()
        self._lines = []
        for raw in text.split("\n"):
            indent = len(raw) - len(raw.lstrip(" \t"))
            content = len(raw.rstrip())
            self._lines.append((indent, content))
        self.update()

    # ----------------------------------------------------- Painting
    def paintEvent(self, event):  # noqa: N802
        painter = QPainter(self)
        c = COLORS
        painter.fillRect(self.rect(), QColor(c["bg_alt"]))

        if not self._lines:
            return

        total = len(self._lines)
        view_h = self.height()
        # Wie viel Buffer passt vertikal in die Minimap?
        max_lines_visible = max(1, view_h // LINE_HEIGHT_PX)

        # Wenn Buffer kleiner als sichtbar: keine Skalierung.
        # Sonst skalieren -- Schritt zwischen 0 und total.
        if total <= max_lines_visible:
            offset_first = 0
            scale = 1.0
        else:
            # Editor-Scroll-Position ueberblicken: erst-sichtbare Zeile aus
            # dem Editor's first-visible-block.
            first_visible = self.editor.firstVisibleBlock().blockNumber()
            # Minimap wird mit-scrollen: setze offset_first so, dass die
            # aktuell sichtbare Region vertikal zentriert sichtbar ist.
            offset_first = max(0, min(total - max_lines_visible, first_visible - max_lines_visible // 4))
            scale = 1.0

        line_color = QColor(c["fg"])
        line_color.setAlpha(140)
        painter.setPen(line_color)

        y = 0
        for i in range(offset_first, min(total, offset_first + max_lines_visible)):
            indent, content = self._lines[i]
            if content > 0:
                # Linke Position: 4 px + indent-skaliert
                x_start = 4 + min(indent, 40)
                x_end = min(MINIMAP_WIDTH - 6, x_start + min(content - indent, MAX_LINE_WIDTH))
                painter.drawLine(x_start, y + 1, x_end, y + 1)
            y += int(LINE_HEIGHT_PX * scale)

        # Viewport-Rechteck: aktuell sichtbarer Bereich
        first_visible = self.editor.firstVisibleBlock().blockNumber()
        last_visible = first_visible + self._approx_visible_block_count()
        rect_top = max(0, (first_visible - offset_first) * LINE_HEIGHT_PX)
        rect_bot = max(rect_top + 4, (last_visible - offset_first) * LINE_HEIGHT_PX)
        rect = QRect(0, int(rect_top), MINIMAP_WIDTH, int(rect_bot - rect_top))
        overlay = QColor(c["accent"])
        overlay.setAlpha(48)
        painter.fillRect(rect, overlay)
        painter.setPen(QPen(QColor(c["accent"]), 1))
        painter.drawRect(rect.adjusted(0, 0, -1, -1))

        # Cache offset fuer Click-Konvertierung.
        self._offset_first = offset_first

    def _approx_visible_block_count(self) -> int:
        """Zaehl der sichtbaren Bloecke im Editor."""
        block = self.editor.firstVisibleBlock()
        viewport_h = self.editor.viewport().height()
        block_height = self.editor.blockBoundingRect(block).height() or 14
        return max(1, int(viewport_h / block_height))

    # ----------------------------------------------------- Maus
    def mousePressEvent(self, event: QMouseEvent):  # noqa: N802
        if event.button() != Qt.MouseButton.LeftButton:
            return
        self._dragging = True
        self.setCursor(Qt.CursorShape.ClosedHandCursor)
        self._scroll_to(event.position().y())

    def mouseMoveEvent(self, event: QMouseEvent):  # noqa: N802
        if self._dragging:
            self._scroll_to(event.position().y())

    def mouseReleaseEvent(self, event: QMouseEvent):  # noqa: N802
        self._dragging = False
        self.setCursor(Qt.CursorShape.OpenHandCursor)

    def _scroll_to(self, y: float) -> None:
        # Der Y-Wert auf der Minimap entspricht "offset_first + y/LINE_HEIGHT".
        offset_first = getattr(self, "_offset_first", 0)
        target_block = max(0, int(offset_first + y / LINE_HEIGHT_PX))
        # Ziel-Block in der Mitte des Editors landen lassen.
        target_block = max(0, target_block - self._approx_visible_block_count() // 2)
        sb = self.editor.verticalScrollBar()
        # Block-Index ist nicht exakt = Scrollwert (wegen wrap), aber
        # `setValue` mit Linien-Index ist gut genug fuer NoWrap-Layouts.
        sb.setValue(min(sb.maximum(), target_block))
