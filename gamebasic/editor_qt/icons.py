"""Theme-reaktive Vektor-Icons fuer die Toolbar.

Statt Qt-Standard-Icons (SP_FileIcon etc., die im Dark-Theme oft viel
zu dunkel rendern) malen wir die Symbole selbst auf einem QPixmap mit
QPainter. Stroke/Fill-Farben kommen aus der aktiven Palette -- bei
Theme-Wechsel cachen wir die Icons frisch.

Verwendung:
    icon_provider.set_theme_changed_listener()  # einmal in MainWindow
    btn.setIcon(icon_provider.get("save"))

Verfuegbare Schluessel:
    new, open, save, save_as, run, stop, bench, find, replace,
    goto, settings, fold, unfold, info, theme, close, refresh.
"""
from __future__ import annotations

from PySide6.QtCore import QObject, QPoint, QPointF, QRect, QRectF, Qt, Signal
from PySide6.QtGui import (
    QBrush, QColor, QIcon, QPainter, QPainterPath, QPen, QPixmap, QPolygonF,
)

from .theme import COLORS, theme_signals


# Bevorzugte Icon-Pixelgroesse. Toolbar-IconSize ist meist 22-24px;
# wir rendern in 24x24 mit DPI=2 fuer scharfe Edges auf HiDPI.
ICON_SIZE = 24


def _new_pixmap() -> QPixmap:
    pix = QPixmap(ICON_SIZE * 2, ICON_SIZE * 2)
    pix.setDevicePixelRatio(2.0)
    pix.fill(Qt.GlobalColor.transparent)
    return pix


def _painter(pix: QPixmap) -> QPainter:
    p = QPainter(pix)
    p.setRenderHint(QPainter.RenderHint.Antialiasing, True)
    return p


def _stroke(width: float = 1.6, color_key: str = "fg") -> QPen:
    pen = QPen(QColor(COLORS[color_key]))
    pen.setWidthF(width)
    pen.setCapStyle(Qt.PenCapStyle.RoundCap)
    pen.setJoinStyle(Qt.PenJoinStyle.RoundJoin)
    return pen


# ---------- Einzelne Icon-Maler --------------------------------------

def _icon_new() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.6))
    # Doc-Rect mit gefaltetem rechten oberen Eck
    path = QPainterPath()
    path.moveTo(6, 4)
    path.lineTo(15, 4)
    path.lineTo(20, 9)
    path.lineTo(20, 20)
    path.lineTo(6, 20)
    path.closeSubpath()
    p.drawPath(path)
    # Faltlinie
    p.drawLine(15, 4, 15, 9)
    p.drawLine(15, 9, 20, 9)
    # Plus-Symbol unten
    p.setPen(_stroke(1.6, "accent"))
    p.drawLine(13, 14, 13, 18)
    p.drawLine(11, 16, 15, 16)
    p.end()
    return pix


def _icon_open() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.6))
    # Hinterer Folder-Body
    p.drawRoundedRect(QRectF(3, 7, 18, 13), 1.5, 1.5)
    # Tab oben
    p.drawLine(3, 7, 9, 7)
    p.drawLine(9, 7, 11, 5)
    p.drawLine(11, 5, 17, 5)
    p.drawLine(17, 5, 17, 7)
    p.end()
    return pix


def _icon_save() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.6))
    # Floppy outline
    p.drawRoundedRect(QRectF(4, 4, 16, 16), 1.5, 1.5)
    # Slot oben
    p.setBrush(QBrush(QColor(COLORS["fg"])))
    p.drawRect(QRectF(7, 4, 10, 4))
    # Label-Bereich unten
    p.setBrush(QBrush(QColor(COLORS["bg"])))
    p.drawRect(QRectF(7, 13, 10, 7))
    p.end()
    return pix


def _icon_save_as() -> QPixmap:
    pix = _icon_save()
    p = _painter(pix)
    # Pfeil rechts unten -- "exportieren"
    p.setPen(_stroke(1.6, "accent"))
    p.setBrush(QBrush(QColor(COLORS["accent"])))
    poly = QPolygonF([QPointF(17, 22), QPointF(22, 18), QPointF(20, 18),
                       QPointF(20, 14), QPointF(14, 14), QPointF(14, 18),
                       QPointF(12, 18)])
    # Vereinfacht: einfacher Pfeil nach rechts unten
    p.setPen(_stroke(1.6, "accent"))
    p.drawLine(20, 16, 22, 16)
    p.drawLine(22, 16, 22, 22)
    p.drawLine(22, 22, 16, 22)
    p.drawLine(22, 22, 18, 18)
    p.end()
    return pix


def _icon_run() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(Qt.PenStyle.NoPen)
    p.setBrush(QBrush(QColor(COLORS["success"])))
    poly = QPolygonF([QPointF(7, 5), QPointF(7, 19), QPointF(19, 12)])
    p.drawPolygon(poly)
    p.end()
    return pix


def _icon_stop() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(Qt.PenStyle.NoPen)
    p.setBrush(QBrush(QColor(COLORS["danger"])))
    p.drawRoundedRect(QRectF(6, 6, 12, 12), 1.5, 1.5)
    p.end()
    return pix


def _icon_bench() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.8, "accent"))
    # Lightning Bolt
    poly = QPolygonF([
        QPointF(13, 3), QPointF(6, 13), QPointF(11, 13),
        QPointF(9, 21), QPointF(18, 10), QPointF(13, 10),
    ])
    p.setBrush(QBrush(QColor(COLORS["accent"])))
    p.drawPolygon(poly)
    p.end()
    return pix


def _icon_find() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.7))
    # Lupen-Kreis
    p.drawEllipse(QPointF(10, 10), 5, 5)
    # Stiel
    p.drawLine(14, 14, 19, 19)
    p.end()
    return pix


def _icon_replace() -> QPixmap:
    pix = _icon_find()
    p = _painter(pix)
    p.setPen(_stroke(1.5, "accent"))
    # Zwei kleine Pfeile (Ersetzen-Symbol) rechts oben
    p.drawLine(16, 4, 21, 4)
    p.drawLine(21, 4, 19, 6)
    p.drawLine(21, 4, 19, 2)
    p.drawLine(21, 9, 16, 9)
    p.drawLine(16, 9, 18, 7)
    p.drawLine(16, 9, 18, 11)
    p.end()
    return pix


def _icon_close() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.8, "fg_muted"))
    p.drawLine(7, 7, 17, 17)
    p.drawLine(17, 7, 7, 17)
    p.end()
    return pix


def _icon_info() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.6, "accent"))
    p.drawEllipse(QPointF(12, 12), 8, 8)
    p.setBrush(QBrush(QColor(COLORS["accent"])))
    p.drawEllipse(QPointF(12, 8), 1.2, 1.2)
    p.setPen(_stroke(1.6, "accent"))
    p.drawLine(12, 11, 12, 17)
    p.end()
    return pix


def _icon_theme_dark() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(Qt.PenStyle.NoPen)
    p.setBrush(QBrush(QColor(COLORS["fg"])))
    # Sichel: Vollkreis links, Halbkreis-Substraktion rechts
    path = QPainterPath()
    path.addEllipse(QPointF(11, 12), 8, 8)
    cut = QPainterPath()
    cut.addEllipse(QPointF(15, 10), 7, 7)
    p.drawPath(path.subtracted(cut))
    p.end()
    return pix


def _icon_theme_light() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.7, "fg"))
    p.setBrush(QBrush(QColor(COLORS["fg"])))
    p.drawEllipse(QPointF(12, 12), 4, 4)
    # 8 Strahlen
    import math
    for i in range(8):
        a = i * math.pi / 4
        cx = math.cos(a); sy = math.sin(a)
        p.drawLine(int(12 + cx * 7), int(12 + sy * 7),
                   int(12 + cx * 10), int(12 + sy * 10))
    p.end()
    return pix


def _icon_fold() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.6, "accent"))
    # Aufgehender Pfeil + Strich darunter (collapse-up)
    p.drawLine(8, 9, 12, 5)
    p.drawLine(12, 5, 16, 9)
    p.drawLine(6, 14, 18, 14)
    p.drawLine(8, 19, 16, 19)
    p.end()
    return pix


def _icon_unfold() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.6, "accent"))
    # Pfeil nach unten + Striche darueber/darunter (expand)
    p.drawLine(6, 5, 18, 5)
    p.drawLine(8, 10, 16, 10)
    p.drawLine(8, 16, 12, 20)
    p.drawLine(12, 20, 16, 16)
    p.end()
    return pix


def _icon_files() -> QPixmap:
    """ActivityBar: Files-View. Stilisierter Ordner mit Dokument-Ecke."""
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.6))
    # Hintergrund: Ordner (offen)
    p.drawRoundedRect(QRectF(3, 8, 18, 12), 1.5, 1.5)
    p.drawLine(3, 8, 9, 8)
    p.drawLine(9, 8, 11, 6)
    p.drawLine(11, 6, 17, 6)
    p.drawLine(17, 6, 17, 8)
    # Dokument-Ueberlappung in Akzent
    p.setPen(_stroke(1.4, "accent"))
    p.setBrush(QBrush(QColor(COLORS["bg"])))
    path = QPainterPath()
    path.moveTo(8, 12)
    path.lineTo(15, 12)
    path.lineTo(18, 15)
    path.lineTo(18, 19)
    path.lineTo(8, 19)
    path.closeSubpath()
    p.drawPath(path)
    p.drawLine(15, 12, 15, 15)
    p.drawLine(15, 15, 18, 15)
    p.end()
    return pix


def _icon_outline() -> QPixmap:
    """ActivityBar: Outline-View. Hierarchische Punktliste."""
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.4))
    p.setBrush(QBrush(QColor(COLORS["accent"])))
    # 4 Eintraege, 3 davon eingerueckt zur Andeutung von Hierarchie
    levels = [(5, 6), (8, 11), (8, 16), (5, 21)]
    for x, y in levels:
        # Bullet
        p.drawEllipse(QPointF(x, y), 1.6, 1.6)
        # Linie nach rechts
        p.setPen(_stroke(1.4))
        p.drawLine(x + 4, y, 20, y)
        p.setBrush(QBrush(QColor(COLORS["accent"])))
    p.end()
    return pix


def _icon_builtins() -> QPixmap:
    """ActivityBar: Built-ins. Mathematisches Lambda fuer "Funktionen"."""
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(2.0, "accent"))
    # Lambda-Form: Diagonale + zweiter Strich am Knick
    # /\
    # /  \
    p.drawLine(7, 19, 13, 5)     # rechte Diagonale
    p.drawLine(13, 5, 17, 19)    # ... weiter nach rechts unten
    p.drawLine(11, 11, 7, 5)     # linke Diagonale (kurz)
    p.end()
    return pix


def _icon_sprite_editor() -> QPixmap:
    """Pixel-Grid -- erkennt man als Sprite/Pixel-Art-Editor."""
    pix = _new_pixmap()
    p = _painter(pix)
    p.setRenderHint(QPainter.RenderHint.Antialiasing, False)
    # 4x4 Pixel-Quadrat, abwechselnde Farben aus dem Logo-Theme.
    cell_size = 4
    origin_x = 4
    origin_y = 4
    palette = [
        QColor(COLORS["accent"]),       # cyan
        QColor(COLORS["success"]),      # mint
        QColor(COLORS["danger"]),       # magenta
        QColor(COLORS["fg"]),           # weiss
    ]
    p.setPen(Qt.PenStyle.NoPen)
    for ry in range(4):
        for rx in range(4):
            p.setBrush(palette[(rx + ry * 2) % 4])
            p.drawRect(
                origin_x + rx * cell_size,
                origin_y + ry * cell_size,
                cell_size - 1, cell_size - 1,
            )
    # Dezenter Rahmen
    p.setPen(_stroke(1.0, "border"))
    p.setBrush(Qt.BrushStyle.NoBrush)
    p.drawRect(origin_x - 1, origin_y - 1, cell_size * 4, cell_size * 4)
    p.end()
    return pix


def _icon_settings() -> QPixmap:
    pix = _new_pixmap()
    p = _painter(pix)
    p.setPen(_stroke(1.6))
    # 6-Zaehnige Zahnrad-Vereinfachung: Kreis + 6 kleine Striche radial
    p.setBrush(Qt.BrushStyle.NoBrush)
    p.drawEllipse(QPointF(12, 12), 4, 4)
    import math
    for i in range(6):
        a = i * math.pi / 3
        cx = math.cos(a); sy = math.sin(a)
        p.drawLine(int(12 + cx * 7), int(12 + sy * 7),
                   int(12 + cx * 10), int(12 + sy * 10))
    p.end()
    return pix


_BUILDERS = {
    "new":      _icon_new,
    "open":     _icon_open,
    "save":     _icon_save,
    "save_as":  _icon_save_as,
    "run":      _icon_run,
    "stop":     _icon_stop,
    "bench":    _icon_bench,
    "find":     _icon_find,
    "replace":  _icon_replace,
    "close":    _icon_close,
    "info":     _icon_info,
    "theme_dark":  _icon_theme_dark,
    "theme_light": _icon_theme_light,
    "fold":     _icon_fold,
    "unfold":   _icon_unfold,
    "settings": _icon_settings,
    "files":    _icon_files,
    "outline":  _icon_outline,
    "builtins": _icon_builtins,
    "sprite_editor": _icon_sprite_editor,
}


class IconProvider(QObject):
    """Liefert (cached) Icons. Bei Theme-Wechsel cleart der Cache."""

    refreshed = Signal()

    def __init__(self):
        super().__init__()
        self._cache: dict[str, QIcon] = {}
        theme_signals.changed.connect(self._on_theme_changed)

    def get(self, key: str) -> QIcon:
        if key in self._cache:
            return self._cache[key]
        builder = _BUILDERS.get(key)
        if builder is None:
            return QIcon()
        ic = QIcon(builder())
        self._cache[key] = ic
        return ic

    def _on_theme_changed(self, _name: str) -> None:
        self._cache.clear()
        self.refreshed.emit()


# Singleton -- alle Konsumenten holen ihre Icons hier.
icons = IconProvider()
