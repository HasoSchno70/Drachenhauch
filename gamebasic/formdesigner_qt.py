"""Form-Designer (WYSIWYG, Xojo-Stil) -- PySide6-UI.

Eigenstaendiges Qt-Programm: links die Control-Palette, in der Mitte die
Design-Flaeche (Controls platzieren/selektieren/verschieben/loeschen), rechts
der Inspector (Eigenschaften + Events). Speichert/laedt `.gbform` (Runtime-
Format) und kann das Formular per "Run" mit `gbrt` starten (laedt das Layout +
generierte Event-Handler-Stubs -- der Xojo-Lauf).

Datenmodell + Code-Generierung liegen Qt-frei in `gamebasic/formdesigner/`.
Start: `gbform [datei.gbform]` bzw. `gbrun.py --form`.
"""
from __future__ import annotations

import subprocess
import tempfile
from pathlib import Path

from PySide6.QtCore import Qt, QRect, QRectF, QPoint, QSize, QMimeData, Signal
from PySide6.QtGui import (
    QAction, QColor, QPainter, QPen, QBrush, QKeySequence, QFont, QPixmap, QIcon,
)
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QListWidget, QListWidgetItem, QDockWidget,
    QScrollArea, QFormLayout, QLineEdit, QSpinBox, QCheckBox, QPlainTextEdit,
    QFileDialog, QMessageBox, QLabel, QVBoxLayout, QHBoxLayout, QDoubleSpinBox,
    QComboBox, QAbstractItemView, QMenu, QStackedWidget, QToolBar, QPushButton,
    QColorDialog,
)

from .formdesigner import (
    FormDoc, FormProject, Control, History, PALETTE, palette_spec, GRID, HANDLES,
    snap, resize_rect,
)

try:
    from .editor_qt.theme import global_qss
except Exception:  # pragma: no cover - Theme optional
    def global_qss() -> str:
        return ""

try:
    from .editor_qt.highlighter import GBHighlighter
except Exception:  # pragma: no cover - Highlighter optional
    GBHighlighter = None

PAD = 24          # Rand um das Fenster auf der Canvas
TITLE_H = 22      # Titelleisten-Hoehe (wie im gui-Modul)
HANDLE = 8        # Kantenlaenge eines Resize-Griffs (px)
RULER = 18        # Breite/Hoehe der Lineale am Rand

# Resize-Griff -> Maus-Cursor (diagonal/horizontal/vertikal)
_HANDLE_CURSORS = {
    "nw": Qt.CursorShape.SizeFDiagCursor, "se": Qt.CursorShape.SizeFDiagCursor,
    "ne": Qt.CursorShape.SizeBDiagCursor, "sw": Qt.CursorShape.SizeBDiagCursor,
    "n": Qt.CursorShape.SizeVerCursor, "s": Qt.CursorShape.SizeVerCursor,
    "e": Qt.CursorShape.SizeHorCursor, "w": Qt.CursorShape.SizeHorCursor,
}


# EINE Quelle fuer die gbrt-Suche (`editor_qt/gbrt_locate.py`): die frueher hier
# stehende Kopie kannte nur den Dev-Baum und meldete in der installierten IDE
# "Runtime nicht gebaut", obwohl `gbrt.exe` neben `GameBasic.exe` liegt. Als
# Alias importiert bleibt der Modul-Name `_find_gbrt` fuer Tests patchbar.
from .editor_qt.gbrt_locate import find_gbrt as _find_gbrt


def _col(i: int) -> QColor:
    return QColor((i >> 16) & 0xFF, (i >> 8) & 0xFF, i & 0xFF)


def _progress_frac(c) -> float:
    """Fuellstand eines ProgressBar-Controls -- exakt wie die Laufzeit
    (`gui.rs`, `Kind::Progress`): Anteil an [min, max]. Die Canvas las `value`
    vorher roh als 0..1, ein Balken mit max=100/value=25 sah dadurch randvoll
    aus und lief zur Laufzeit dann bei 25 %."""
    span = c.max - c.min
    if span == 0:
        return 0.0
    return min(1.0, max(0.0, (c.value - c.min) / span))


# MIME-Typ fuer Drag&Drop eines Palette-Eintrags (Control-Art).
_CONTROL_MIME = "application/x-gbcontrol-kind"

_PAL_BG = QColor(34, 46, 58)
_PAL_FG = QColor(222, 232, 240)
_PAL_ACCENT = QColor(43, 196, 232)
_PAL_BORDER = QColor(78, 100, 122)


def _paint_glyph(qp: QPainter, kind: str, r: QRect):
    """Mini-Vorschau eines Controls in `r` -- fuer Palette-Icon + Drag-Bild."""
    qp.setRenderHint(QPainter.RenderHint.Antialiasing, True)
    qp.setFont(QFont("Segoe UI", 7))
    cy = r.center().y()
    qp.setPen(QPen(_PAL_BORDER, 1))
    if kind == "button":
        qp.setBrush(QColor(52, 68, 86)); qp.drawRoundedRect(r, 4, 4)
        qp.setPen(_PAL_FG); qp.drawText(r, Qt.AlignmentFlag.AlignCenter, "Button")
    elif kind == "label":
        qp.setPen(_PAL_FG); qp.drawText(r, Qt.AlignmentFlag.AlignCenter, "Label")
    elif kind in ("checkbox", "radio"):
        box = QRect(r.left() + 2, cy - 6, 12, 12)
        qp.setBrush(QColor(24, 32, 42))
        if kind == "checkbox":
            qp.drawRect(box)
            qp.setPen(QPen(_PAL_ACCENT, 2))
            qp.drawLine(box.left() + 2, cy, box.center().x(), box.bottom() - 2)
            qp.drawLine(box.center().x(), box.bottom() - 2, box.right() - 1, box.top() + 1)
        else:
            qp.drawEllipse(box)
            qp.setBrush(_PAL_ACCENT); qp.setPen(Qt.PenStyle.NoPen)
            qp.drawEllipse(box.adjusted(3, 3, -3, -3))
        qp.setPen(_PAL_FG)
        qp.drawText(QRect(box.right() + 5, r.top(), r.right() - box.right() - 5, r.height()),
                    Qt.AlignmentFlag.AlignVCenter, kind.capitalize())
    elif kind == "slider":
        qp.setPen(QPen(_PAL_BORDER, 2)); qp.drawLine(r.left() + 4, cy, r.right() - 4, cy)
        qp.setBrush(_PAL_ACCENT); qp.setPen(Qt.PenStyle.NoPen)
        qp.drawEllipse(QRect(r.center().x() - 5, cy - 5, 10, 10))
    elif kind == "textinput":
        qp.setBrush(QColor(20, 26, 34)); qp.drawRect(r)
        qp.setPen(QPen(_PAL_ACCENT, 1)); qp.drawLine(r.left() + 5, r.top() + 4, r.left() + 5, r.bottom() - 4)
        qp.setPen(QColor(150, 162, 176))
        qp.drawText(r.adjusted(10, 0, 0, 0), Qt.AlignmentFlag.AlignVCenter, "Text…")
    elif kind == "dropdown":
        qp.setBrush(QColor(52, 68, 86)); qp.drawRect(r)
        qp.setPen(_PAL_FG); qp.drawText(r.adjusted(5, 0, -16, 0), Qt.AlignmentFlag.AlignVCenter, "Auswahl")
        qp.drawText(QRect(r.right() - 14, r.top(), 12, r.height()), Qt.AlignmentFlag.AlignCenter, "▾")
    elif kind == "listbox":
        qp.setBrush(QColor(30, 40, 52)); qp.drawRect(r)
        qp.setPen(QColor(120, 134, 150))
        for i in range(3):
            yy = r.top() + 6 + i * 8
            qp.drawLine(r.left() + 5, yy, r.right() - 6, yy)
    elif kind == "progress":
        qp.setBrush(QColor(30, 40, 52)); qp.drawRect(r)
        qp.setBrush(_PAL_ACCENT); qp.setPen(Qt.PenStyle.NoPen)
        qp.drawRect(QRect(r.left() + 1, r.top() + 1, int(r.width() * 0.6), r.height() - 2))
    elif kind == "image":
        qp.setBrush(QColor(40, 44, 52)); qp.drawRect(r)
        qp.setPen(QPen(_PAL_ACCENT, 1))
        qp.drawLine(r.left() + 3, r.bottom() - 4, r.center().x() - 2, cy)
        qp.drawLine(r.center().x() - 2, cy, r.right() - 4, r.bottom() - 4)
        qp.setBrush(QColor(240, 220, 120)); qp.setPen(Qt.PenStyle.NoPen)
        qp.drawEllipse(QRect(r.right() - 14, r.top() + 4, 6, 6))
    elif kind == "canvas":
        qp.setBrush(QColor(14, 20, 28)); qp.drawRect(r)
        qp.setPen(QPen(_PAL_BORDER, 1, Qt.PenStyle.DashLine))
        qp.drawLine(r.topLeft(), r.bottomRight())
    elif kind == "panel":
        qp.setBrush(QColor(30, 40, 52)); qp.drawRect(r)
        qp.fillRect(QRect(r.left(), r.top(), r.width(), 8), QColor(46, 60, 76))
    elif kind == "separator":
        qp.setPen(QPen(_PAL_BORDER, 2)); qp.drawLine(r.left() + 2, cy, r.right() - 2, cy)
    elif kind == "groupbox":
        qp.setBrush(Qt.BrushStyle.NoBrush); qp.setPen(QPen(_PAL_BORDER, 1))
        qp.drawRect(QRect(r.left() + 2, r.top() + 4, r.width() - 4, r.height() - 6))
        qp.fillRect(QRect(r.left() + 6, r.top() + 1, 22, 6), _PAL_BG)
        qp.setPen(_PAL_FG); qp.drawText(QRect(r.left() + 7, r.top(), 24, 8), Qt.AlignmentFlag.AlignVCenter, "Grp")
    else:
        qp.setBrush(QColor(40, 52, 66)); qp.drawRect(r)
        qp.setPen(_PAL_FG); qp.drawText(r, Qt.AlignmentFlag.AlignCenter, kind[:6])


def _palette_icon(kind: str, w: int = 72, h: int = 40) -> QIcon:
    pm = QPixmap(w, h)
    pm.fill(QColor(0, 0, 0, 0))
    qp = QPainter(pm)
    _paint_glyph(qp, kind, QRect(3, 5, w - 6, h - 10))
    qp.end()
    return QIcon(pm)


def _arrange_icon(kind: str, sz: int = 22) -> QIcon:
    """Kleines Icon fuer einen Anordnen-Befehl (Balken + Hilfslinie)."""
    pm = QPixmap(sz, sz); pm.fill(QColor(0, 0, 0, 0))
    qp = QPainter(pm)
    bar = QColor(150, 200, 220); guide = QColor(43, 196, 232)
    qp.setPen(Qt.PenStyle.NoPen); qp.setBrush(bar)
    def hbar(x, y, w): qp.drawRect(x, y, w, 3)
    def vbar(x, y, h): qp.drawRect(x, y, 3, h)
    def gv(x): qp.setPen(QPen(guide, 1)); qp.drawLine(x, 2, x, sz - 2); qp.setPen(Qt.PenStyle.NoPen)
    def gh(y): qp.setPen(QPen(guide, 1)); qp.drawLine(2, y, sz - 2, y); qp.setPen(Qt.PenStyle.NoPen)
    if kind == "left":
        gv(3); hbar(4, 4, 14); hbar(4, 10, 8); hbar(4, 16, 12)
    elif kind == "right":
        gv(sz - 3); hbar(sz - 18, 4, 14); hbar(sz - 12, 10, 8); hbar(sz - 16, 16, 12)
    elif kind == "center_h":
        c = sz // 2; gv(c)
        for w, y in ((14, 4), (8, 10), (12, 16)): hbar(c - w // 2, y, w)
    elif kind == "top":
        gh(3); vbar(4, 4, 14); vbar(10, 4, 8); vbar(16, 4, 12)
    elif kind == "bottom":
        gh(sz - 3); vbar(4, sz - 18, 14); vbar(10, sz - 12, 8); vbar(16, sz - 16, 12)
    elif kind == "center_v":
        c = sz // 2; gh(c)
        for h, x in ((14, 4), (8, 10), (12, 16)): vbar(x, c - h // 2, h)
    elif kind == "dist_h":
        vbar(3, 5, 12); vbar(sz // 2 - 1, 5, 12); vbar(sz - 4, 5, 12)
    elif kind == "dist_v":
        hbar(5, 3, 12); hbar(5, sz // 2 - 1, 12); hbar(5, sz - 4, 12)
    elif kind == "same_w":
        qp.drawRect(4, 5, 14, 4); qp.drawRect(4, 13, 14, 4)
    elif kind == "same_h":
        qp.drawRect(5, 4, 4, 14); qp.drawRect(13, 4, 4, 14)
    elif kind == "same_both":
        qp.setBrush(Qt.BrushStyle.NoBrush); qp.setPen(QPen(bar, 2)); qp.drawRect(4, 4, 14, 14)
    qp.end()
    return QIcon(pm)


class _PaletteList(QListWidget):
    """Palette mit grafischen Control-Icons; Eintraege sind per Drag&Drop auf die
    Canvas ziehbar (MIME `_CONTROL_MIME` = Control-Art)."""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setIconSize(QSize(72, 40))
        self.setSpacing(3)
        self.setDragEnabled(True)
        self.setDragDropMode(QAbstractItemView.DragDropMode.DragOnly)
        self.setSelectionMode(QAbstractItemView.SelectionMode.SingleSelection)

    def mimeData(self, items):
        md = QMimeData()
        if items:
            kind = items[0].data(Qt.ItemDataRole.UserRole)
            md.setData(_CONTROL_MIME, str(kind).encode("utf-8"))
            md.setText(str(kind))
        return md


class _Canvas(QWidget):
    """Zeichnet das Formular + Controls und behandelt Platzieren/Selektieren/Ziehen."""
    selection_changed = Signal(object)   # Control | None
    doc_changed = Signal()
    doc_replaced = Signal(object)        # FormDoc (komplett ersetzt: set_doc/Undo)
    handler_requested = Signal(object)   # Control (Doppelklick -> Code-Editor)
    context_menu = Signal(object)        # QPoint (global) -- Rechtsklick auf Control
    zoom_changed = Signal(float)         # neue Zoom-Stufe
    form_resized = Signal()              # Formular per Griff in der Groesse geaendert

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc = FormDoc()
        self.selected: Control | None = None     # primaeres Control (Inspector/Griffe)
        self.selection: list[Control] = []       # vollstaendige Mehrfach-Selektion
        self.place_kind: str | None = None      # aus der Palette "scharf geschaltet"
        self.snap_grid = True                    # Snap-to-Grid aktiv?
        # commit_history(pre_snapshot): vom Fenster gesetzt, legt einen
        # Undo-Checkpoint an. None = kein Undo (z.B. Standalone-Canvas).
        self.commit_history = None
        self._drag = False
        self._drag_off = QPoint(0, 0)
        self._resize_handle: str | None = None   # aktiver Resize-Griff beim Ziehen
        self._pending: dict | None = None        # Pre-Gesten-Snapshot (Drag/Resize)
        self._nudge_active = False               # laufende Pfeiltasten-Verschiebung?
        self.zoom = 1.0                          # Zoom-Faktor der Design-Flaeche
        self.show_rulers = True                  # Lineale am Rand?
        self._guides_v: list[int] = []           # aktive vertikale Ausrichtlinien (ctrl-x)
        self._guides_h: list[int] = []           # aktive horizontale Ausrichtlinien (ctrl-y)
        self._multi = False                      # Gruppen-Drag (mehrere) aktiv?
        self._drag_origin = (0, 0)               # Maus-Start (ctrl-Raum) fuer Gruppen-Drag
        self._drag_starts: list = []             # [(control, x0, y0)] beim Gruppen-Drag
        self._band = False                       # Auswahlrahmen (Rubber-Band) aktiv?
        self._band_start = (0, 0)
        self._band_now = (0, 0)
        self._band_additive = False
        self._form_resize: str | None = None     # Formular-Resize-Griff (e/s/se)
        self.setMinimumSize(640, 480)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setMouseTracking(True)              # Hover-Cursor ueber den Griffen
        self.setAcceptDrops(True)                # Drag&Drop aus der Palette

    def set_doc(self, doc: FormDoc):
        self.doc = doc
        self.selected = None
        self.selection = []
        # Alle Gesten-Flags loeschen: das Dokument kann mitten im Ziehen
        # getauscht werden (Strg+Z, Formularwechsel). Blieben sie stehen,
        # verschob die naechste Mausbewegung das neue Dokument.
        self._pending = None
        self._band = False
        self._drag = False
        self._multi = False
        self._drag_starts = []
        self._resize_handle = None
        self._form_resize = None
        self.place_kind = None
        self._guides_v = []; self._guides_h = []
        self.selection_changed.emit(None)
        self.doc_replaced.emit(doc)
        self._resize_to_doc()
        self.update()

    def _resize_to_doc(self):
        z = self.zoom
        self.setMinimumSize(int((self.doc.w + 2 * PAD + 40) * z),
                            int((self.doc.h + 2 * PAD + 40) * z))

    def set_zoom(self, z: float):
        z = max(0.25, min(4.0, z))
        if abs(z - self.zoom) < 1e-6:
            return
        self.zoom = z
        self._resize_to_doc()
        self.update()
        self.zoom_changed.emit(z)

    def wheelEvent(self, ev):
        if ev.modifiers() & Qt.KeyboardModifier.ControlModifier:
            self.set_zoom(self.zoom * (1.1 if ev.angleDelta().y() > 0 else 1 / 1.1))
            ev.accept()
        else:
            super().wheelEvent(ev)

    # -- Koordinaten --
    # Widget-Pixel -> Zeichen-Raum (durch den Zoom geteilt); Zeichen-Raum ->
    # Control-relativ (Fenster-Inhalt). Mausereignisse erst durch _to_draw.
    def _to_draw(self, pt) -> QPoint:
        z = self.zoom or 1.0
        return QPoint(int(pt.x() / z), int(pt.y() / z))

    def _to_ctrl(self, p: QPoint) -> tuple[int, int]:
        return (p.x() - PAD, p.y() - PAD - TITLE_H)

    def _snap(self, v: int) -> int:
        return snap(v) if self.snap_grid else int(v)

    _ALIGN_THRESH = 6        # Fangabstand (Zeichen-Pixel) fuer Ausrichtlinien

    def _align_axis(self, lo: float, size: float, targets: set) -> tuple:
        """Eine Achse fangen: `lo`/`size` = Kante+Laenge des gezogenen Controls.
        Liefert (neue_lo | None, guide-Linie | None) -- gefangen wird die Kante
        (Anfang/Mitte/Ende) mit der kleinsten Distanz zu einem Ziel."""
        best = None
        for off in (0.0, size / 2.0, size):       # left/center/right bzw. top/mid/bottom
            edge = lo + off
            for t in targets:
                d = t - edge
                if abs(d) <= self._ALIGN_THRESH and (best is None or abs(d) < abs(best[0])):
                    best = (d, t)
        if best is None:
            return None, None
        return lo + best[0], best[1]

    def _x_targets(self, c: Control) -> set:
        xs = {0.0, self.doc.w / 2.0, float(self.doc.w)}
        for o in self.doc.controls:
            if o is not c:
                xs.update((float(o.x), o.x + o.w / 2.0, float(o.x + o.w)))
        return xs

    def _y_targets(self, c: Control) -> set:
        bottom = self.doc.h - TITLE_H
        ys = {0.0, bottom / 2.0, float(bottom)}
        for o in self.doc.controls:
            if o is not c:
                ys.update((float(o.y), o.y + o.h / 2.0, float(o.y + o.h)))
        return ys

    def _move_to(self, px: int, py: int):
        """Selektiertes Control nach (px, py) (Ziel-Ecke) verschieben -- mit
        Ausrichtungs-Fang an andere Controls/Formularraender, sonst Raster-Fang.
        Setzt die aktiven Hilfslinien (`_guides_*`)."""
        c = self.selected
        px = max(0, px); py = max(0, py)
        nx, gx = self._align_axis(px, c.w, self._x_targets(c))
        ny, gy = self._align_axis(py, c.h, self._y_targets(c))
        c.x = int(nx) if nx is not None else self._snap(px)
        c.y = int(ny) if ny is not None else self._snap(py)
        # Im Formular halten: ein weit nach rechts/unten gezogenes Control lag
        # sonst ausserhalb der Zeichenflaeche -- unsichtbar, nicht anklickbar,
        # in keiner Liste, und so auch ins .gbform gespeichert. Zurueckholen
        # ging nur per Undo oder Hand-Edit im JSON.
        c.x = min(max(0, c.x), self._max_x(c))
        c.y = min(max(0, c.y), self._max_y(c))
        self._guides_v = [int(gx)] if gx is not None else []
        self._guides_h = [int(gy)] if gy is not None else []

    def _clear_guides(self):
        if self._guides_v or self._guides_h:
            self._guides_v = []; self._guides_h = []
            self.update()

    def _handle_points(self, c: Control) -> dict[str, QPoint]:
        """Screen-Mittelpunkte der 8 Resize-Griffe des Controls."""
        x = PAD + c.x
        y = PAD + TITLE_H + c.y
        mx, my = x + c.w // 2, y + c.h // 2
        rx, by = x + c.w, y + c.h
        return {
            "nw": QPoint(x, y), "n": QPoint(mx, y), "ne": QPoint(rx, y),
            "e": QPoint(rx, my), "se": QPoint(rx, by), "s": QPoint(mx, by),
            "sw": QPoint(x, by), "w": QPoint(x, my),
        }

    def _handle_tol(self, c: Control | None = None) -> float:
        """Trefferradius eines Resize-Griffs im Zeichen-Raum.

        `/zoom`, damit der Griff auf dem BILDSCHIRM immer gleich gross ist (bei
        0,25x war er sonst 2 px gross und praktisch nicht treffbar). Zusaetzlich
        nie mehr als ein Viertel des Controls: die 8 Zonen ueberdeckten ein
        16x16-Control (Checkbox/Radio = Palette-Default) sonst zu 100 %, es gab
        keinen Pixel mehr zum Verschieben -- jeder Zieh-Versuch hat die Checkbox
        stattdessen auf 8x8 geschrumpft."""
        t = (HANDLE / 2.0) / (self.zoom or 1.0)
        if c is not None:
            t = min(t, c.w / 4.0, c.h / 4.0)
        return max(2.0, t)

    def _handle_at(self, p: QPoint) -> str | None:
        """Welcher Resize-Griff des selektierten Controls liegt unter `p`?"""
        c = self.selected
        if c is None:
            return None
        tol = self._handle_tol(c)
        # Innenbereich gehoert immer dem Verschieben (durch die Viertel-Grenze
        # oben bleibt er mindestens halb so gross wie das Control).
        x, y = PAD + c.x, PAD + TITLE_H + c.y
        if (x + tol < p.x() < x + c.w - tol) and (y + tol < p.y() < y + c.h - tol):
            return None
        for name, hp in self._handle_points(c).items():
            if abs(p.x() - hp.x()) <= tol and abs(p.y() - hp.y()) <= tol:
                return name
        return None

    def _form_handle_points(self) -> dict[str, QPoint]:
        """Resize-Griffe des Formulars (rechts/unten/Ecke), Zeichen-Raum."""
        w, h = self.doc.w, self.doc.h
        return {
            "e": QPoint(PAD + w, PAD + h // 2),
            "s": QPoint(PAD + w // 2, PAD + h),
            "se": QPoint(PAD + w, PAD + h),
        }

    def _form_handle_at(self, p: QPoint) -> str | None:
        if self.selection:                       # nur wenn das Fenster "selektiert" ist
            return None
        tol = self._handle_tol()
        for name, hp in self._form_handle_points().items():
            if abs(p.x() - hp.x()) <= tol and abs(p.y() - hp.y()) <= tol:
                return name
        return None

    def _clamp_fw(self, v: int) -> int:
        v = max(120, int(v))
        if self.doc.min_w:
            v = max(v, self.doc.min_w)
        if self.doc.max_w:
            v = min(v, self.doc.max_w)
        return v

    def _clamp_fh(self, v: int) -> int:
        v = max(80, int(v))
        if self.doc.min_h:
            v = max(v, self.doc.min_h)
        if self.doc.max_h:
            v = min(v, self.doc.max_h)
        return v

    def paintEvent(self, _ev):
        qp = QPainter(self)
        qp.fillRect(self.rect(), QColor(18, 22, 28))   # Hintergrund (Widget-Raum)
        qp.save()
        qp.scale(self.zoom, self.zoom)                 # ab hier alles im Zeichen-Raum
        d = self.doc
        win = QRect(PAD, PAD, d.w, d.h)
        qp.fillRect(win, QColor(24, 34, 46))
        qp.setPen(QPen(QColor(46, 88, 110), 1))
        qp.drawRect(win)
        if self.snap_grid:
            self._paint_grid(qp, d)
        # Titelleiste
        qp.fillRect(QRect(PAD, PAD, d.w, TITLE_H), QColor(18, 90, 120))
        qp.setPen(QColor(230, 247, 255))
        qp.drawText(PAD + 6, PAD + 15, d.title)
        # Controls
        for c in d.controls:
            self._paint_control(qp, c)
        self._paint_guides(qp, d)
        if not self.selection:
            self._paint_form_handles(qp, d)
        self._paint_selection(qp)
        if self._band:
            self._paint_band(qp)
        qp.restore()                                   # zurueck in den Widget-Raum
        if self.show_rulers:
            self._paint_rulers(qp)

    def _paint_rulers(self, qp: QPainter):
        """Lineale am oberen + linken Rand (Widget-Raum). Zeigen Formular-
        Koordinaten (Inhalt: x ab Form-Links, y ab unter der Titelleiste),
        skaliert mit dem Zoom; markieren den Bereich der Selektion cyan."""
        z = self.zoom
        # Die Lineale liegen im Widget-Raum und schrumpfen daher NICHT mit dem
        # Zoom -- der Formularrand (PAD) schon. Ab etwa 0,7x deckten sie sonst
        # den Anfang des Formulars zu: Controls bis x=48 waren bei 0,25x
        # unsichtbar (aber weiter anklickbar), man suchte ein "verschwundenes"
        # Control. Darum die Leiste auf den verfuegbaren Rand begrenzen.
        R = max(6, min(RULER, int(PAD * z)))
        bg = QColor(28, 34, 42); tickc = QColor(96, 112, 128); txt = QColor(150, 165, 180)
        ox = PAD * z                       # Widget-x von Form-Inhalt-x = 0
        oy = (PAD + TITLE_H) * z            # Widget-y von Form-Inhalt-y = 0
        W, H = self.width(), self.height()
        qp.fillRect(0, 0, W, R, bg)
        qp.fillRect(0, 0, R, H, bg)
        qp.fillRect(0, 0, R, R, QColor(20, 26, 32))
        # Selektions-Bereich hervorheben
        hi = QColor(43, 196, 232, 70)
        for c in self.selection:
            qp.fillRect(int(ox + c.x * z), 0, max(1, int(c.w * z)), R, hi)
            qp.fillRect(0, int(oy + c.y * z), R, max(1, int(c.h * z)), hi)
        step = 50
        qp.setFont(QFont("Segoe UI", 6))
        # Horizontal
        i = 0
        while True:
            wx = ox + i * z
            if wx > W:
                break
            if wx >= R:
                qp.setPen(QPen(tickc, 1)); qp.drawLine(int(wx), R - 5, int(wx), R)
                qp.setPen(txt); qp.drawText(int(wx) + 2, R - 6, str(i))
            i += step
        # Vertikal
        j = 0
        while True:
            wy = oy + j * z
            if wy > H:
                break
            if wy >= R:
                qp.setPen(QPen(tickc, 1)); qp.drawLine(R - 5, int(wy), R, int(wy))
                qp.save(); qp.translate(R - 6, int(wy) - 2); qp.rotate(-90)
                qp.setPen(txt); qp.drawText(0, 0, str(j)); qp.restore()
            j += step

    def _paint_form_handles(self, qp: QPainter, d: FormDoc):
        """Das Formular ist 'selektiert' (kein Control): Accent-Rahmen + 3 Griffe."""
        qp.setPen(QPen(QColor(43, 196, 232), 1, Qt.PenStyle.DashLine))
        qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRect(QRect(PAD - 1, PAD - 1, d.w + 2, d.h + 2))
        qp.setPen(QPen(QColor(12, 18, 24), 1))
        qp.setBrush(QBrush(QColor(43, 196, 232)))
        s = HANDLE / (self.zoom or 1.0)      # auf dem Bildschirm konstant gross
        for hp in self._form_handle_points().values():
            qp.drawRect(QRectF(hp.x() - s / 2, hp.y() - s / 2, s, s))

    def _paint_selection(self, qp: QPainter):
        """Mehrfach-Selektion: alle bekommen einen Rahmen, das primaere Control
        zusaetzlich die 8 Resize-Griffe (Einzel-Resize)."""
        if len(self.selection) == 1:
            self._paint_handles(qp, self.selection[0])
            return
        qp.setPen(QPen(QColor(43, 196, 232), 1, Qt.PenStyle.DashLine))
        qp.setBrush(Qt.BrushStyle.NoBrush)
        for c in self.selection:
            x = PAD + c.x; y = PAD + TITLE_H + c.y
            qp.drawRect(QRect(x - 1, y - 1, c.w + 2, c.h + 2))

    def _paint_band(self, qp: QPainter):
        x0, y0 = self._band_start
        x1, y1 = self._band_now
        r = QRect(PAD + min(x0, x1), PAD + TITLE_H + min(y0, y1), abs(x1 - x0), abs(y1 - y0))
        qp.setPen(QPen(QColor(43, 196, 232), 1, Qt.PenStyle.DashLine))
        qp.setBrush(QColor(43, 196, 232, 40))
        qp.drawRect(r)

    def _paint_guides(self, qp: QPainter, d: FormDoc):
        """Ausrichtungs-Hilfslinien waehrend des Ziehens (pink, ueber dem Form)."""
        if not (self._guides_v or self._guides_h):
            return
        qp.setPen(QPen(QColor(255, 92, 162), 1, Qt.PenStyle.DashLine))
        for gx in self._guides_v:
            X = PAD + gx
            qp.drawLine(X, PAD, X, PAD + d.h)
        for gy in self._guides_h:
            Y = PAD + TITLE_H + gy
            qp.drawLine(PAD, Y, PAD + d.w, Y)

    def _paint_grid(self, qp: QPainter, d: FormDoc):
        """Dezente Raster-Punkte im Fenster-Inhaltsbereich (unter der Titelleiste)."""
        qp.setPen(QPen(QColor(60, 78, 96), 1))
        x0 = PAD
        y0 = PAD + TITLE_H
        gx = GRID
        while gx <= d.w:
            gy = GRID
            while gy <= d.h - TITLE_H:
                qp.drawPoint(x0 + gx, y0 + gy)
                gy += GRID
            gx += GRID

    def _paint_control(self, qp: QPainter, c: Control):
        """Rendert ein Control moeglichst so, wie es zur Laufzeit (gui-Modul,
        Cyan-Theme) aussieht: gefuellte Flaechen, Haken/Knopf/Fortschritt,
        Dropdown-Pfeil, ListBox-Eintraege, disabled gedimmt, unsichtbar getoent."""
        x = PAD + c.x
        y = PAD + TITLE_H + c.y
        w, h = max(c.w, 4), max(c.h, 4)
        r = QRect(x, y, w, h)
        k = c.kind
        en = c.enabled
        fg = QColor(228, 238, 246) if en else QColor(120, 132, 146)
        accent = QColor(43, 196, 232) if en else QColor(74, 112, 128)
        border = QColor(78, 104, 128)
        al = Qt.AlignmentFlag
        font = QFont("Segoe UI")
        font.setPixelSize(max(7, c.font_size)) if c.font_size else font.setPointSize(8)
        qp.setFont(font)
        qp.setRenderHint(QPainter.RenderHint.Antialiasing, True)

        if k == "button":
            qp.setPen(QPen(border, 1)); qp.setBrush(QColor(46, 62, 80) if en else QColor(34, 44, 56))
            qp.drawRoundedRect(r, 5, 5)
            qp.setPen(fg); qp.drawText(r, al.AlignCenter, c.text or "Button")
        elif k == "label":
            col = _col(c.color) if (en and c.color != 0xFFFFFF) else fg
            qp.setPen(col); qp.drawText(r, al.AlignVCenter | al.AlignLeft, c.text or "Label")
        elif k in ("checkbox", "radio"):
            bs = min(h, 16)
            box = QRect(x, y + (h - bs) // 2, bs, bs)
            qp.setPen(QPen(border, 1)); qp.setBrush(QColor(22, 30, 40))
            if k == "checkbox":
                qp.drawRoundedRect(box, 3, 3)
                if c.checked:
                    qp.setPen(QPen(accent, 2))
                    qp.drawLine(box.left() + 3, box.center().y(), box.center().x(), box.bottom() - 3)
                    qp.drawLine(box.center().x(), box.bottom() - 3, box.right() - 2, box.top() + 2)
            else:
                qp.drawEllipse(box)
                if c.checked:
                    qp.setBrush(accent); qp.setPen(Qt.PenStyle.NoPen)
                    qp.drawEllipse(box.adjusted(4, 4, -4, -4))
            qp.setPen(fg)
            qp.drawText(QRect(box.right() + 6, y, w - bs - 6, h), al.AlignVCenter, c.text or k.capitalize())
        elif k == "slider":
            midy = y + h // 2
            frac = (c.value - c.min) / (c.max - c.min) if c.max > c.min else 0.0
            frac = min(1.0, max(0.0, frac))
            kx = int(x + 5 + frac * (w - 10))
            qp.setPen(QPen(border, 3)); qp.drawLine(x + 5, midy, x + w - 5, midy)
            qp.setPen(QPen(accent, 3)); qp.drawLine(x + 5, midy, kx, midy)
            qp.setBrush(accent); qp.setPen(Qt.PenStyle.NoPen)
            qp.drawEllipse(QPoint(kx, midy), 6, 6)
        elif k == "textinput":
            qp.setPen(QPen(border, 1)); qp.setBrush(QColor(18, 24, 32)); qp.drawRect(r)
            if c.text:
                qp.setPen(fg); txt = c.text
            else:
                qp.setPen(QColor(120, 132, 146)); txt = c.placeholder or ""
            qp.drawText(r.adjusted(6, 0, -4, 0), al.AlignVCenter, txt)
        elif k == "dropdown":
            qp.setPen(QPen(border, 1)); qp.setBrush(QColor(46, 62, 80) if en else QColor(34, 44, 56)); qp.drawRect(r)
            sel = c.items[c.sel] if 0 <= c.sel < len(c.items) else ""
            qp.setPen(fg); qp.drawText(r.adjusted(6, 0, -18, 0), al.AlignVCenter, sel)
            qp.drawText(QRect(x + w - 16, y, 14, h), al.AlignCenter, "▾")
        elif k == "listbox":
            qp.setPen(QPen(border, 1)); qp.setBrush(QColor(24, 32, 42)); qp.drawRect(r)
            qp.save(); qp.setClipRect(r)
            lh = 15
            for i, it in enumerate(c.items):
                iy = y + 2 + i * lh
                if iy >= y + h:
                    break
                if i == c.sel:
                    qp.fillRect(QRect(x + 1, iy, w - 2, lh), QColor(28, 84, 112))
                qp.setPen(fg); qp.drawText(QRect(x + 5, iy, w - 8, lh), al.AlignVCenter, str(it))
            qp.restore()
        elif k == "progress":
            qp.setPen(QPen(border, 1)); qp.setBrush(QColor(24, 32, 42)); qp.drawRect(r)
            frac = _progress_frac(c)
            fillw = int((w - 2) * frac)
            if fillw > 0:
                qp.fillRect(QRect(x + 1, y + 1, fillw, h - 2), accent)
        elif k == "panel":
            qp.setPen(QPen(border, 1)); qp.setBrush(QColor(28, 38, 50)); qp.drawRect(r)
            qp.fillRect(QRect(x, y, w, 16), QColor(40, 56, 72))
            qp.setPen(fg); qp.drawText(QRect(x + 5, y, w - 8, 16), al.AlignVCenter, c.text or "")
        elif k == "separator":
            my = y + h // 2
            qp.setPen(QPen(border, 1)); qp.drawLine(x, my, x + w - 1, my)
        elif k == "groupbox":
            qp.setPen(QPen(border, 1)); qp.setBrush(Qt.BrushStyle.NoBrush)
            qp.drawRect(QRect(x, y + 7, w - 1, h - 8))
            if c.text:
                qp.fillRect(QRect(x + 8, y, min(w - 16, len(c.text) * 7 + 8), 13), QColor(24, 34, 46))
                qp.setPen(fg); qp.drawText(x + 12, y + 11, c.text)
        elif k == "image":
            qp.setPen(QPen(border, 1)); qp.setBrush(QColor(40, 44, 52)); qp.drawRect(r)
            qp.setPen(QPen(accent, 1))
            qp.drawLine(x + 4, y + h - 5, x + w // 2 - 2, y + h // 2)
            qp.drawLine(x + w // 2 - 2, y + h // 2, x + w - 5, y + h - 5)
            qp.setBrush(QColor(240, 220, 120)); qp.setPen(Qt.PenStyle.NoPen)
            qp.drawEllipse(QRect(x + w - 16, y + 6, 7, 7))
        elif k == "canvas":
            qp.setBrush(QColor(14, 20, 28)); qp.setPen(QPen(border, 1, Qt.PenStyle.DashLine)); qp.drawRect(r)
            qp.setPen(QColor(120, 134, 150)); qp.drawText(r, al.AlignCenter, "Canvas")
        else:
            qp.setPen(QPen(border, 1)); qp.setBrush(QColor(40, 52, 66)); qp.drawRect(r)
            qp.setPen(fg); qp.drawText(r, al.AlignCenter, c.text or k)

        if not c.visible:                       # unsichtbares Control angedeutet toenen
            qp.fillRect(r, QColor(18, 22, 28, 150))

    def _paint_handles(self, qp: QPainter, c: Control):
        x = PAD + c.x
        y = PAD + TITLE_H + c.y
        # Selektionsrahmen
        qp.setPen(QPen(QColor(43, 196, 232), 1, Qt.PenStyle.DashLine))
        qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRect(QRect(x - 1, y - 1, c.w + 2, c.h + 2))
        # 8 Resize-Griffe -- so gross wie ihre Trefferzone (siehe _handle_tol)
        qp.setPen(QPen(QColor(12, 18, 24), 1))
        qp.setBrush(QBrush(QColor(43, 196, 232)))
        s = 2 * self._handle_tol(c)
        for hp in self._handle_points(c).values():
            qp.drawRect(QRectF(hp.x() - s / 2, hp.y() - s / 2, s, s))

    # -- Maus --
    def _finish_gesture(self):
        """Laufende Geste beenden: Undo-Checkpoint setzen (falls sich wirklich
        etwas geaendert hat) und ALLE Gesten-Flags loeschen.

        Wird nicht nur beim Loslassen gerufen, sondern auch, wenn sich die Maus
        ohne gedrueckte Taste bewegt. Sonst ueberlebte der Zustand jede
        Unterbrechung, die das Release verschluckt (modales Kontextmenue,
        Fokusverlust) oder das Dokument tauscht (Strg+Z waehrend des Ziehens) --
        danach verschob schon blosses Hovern das Control, ohne Undo."""
        active = (self._drag or self._multi or self._band
                  or self._resize_handle is not None or self._form_resize is not None
                  or self._pending is not None)
        if not active:
            return False
        if self._band:
            self._finish_band()
            self._band = False
        if self._pending is not None:
            if self.commit_history and self._pending != self.doc.to_dict():
                self.commit_history(self._pending)
            self._pending = None
        self._drag = False
        self._multi = False
        self._drag_starts = []
        self._resize_handle = None
        self._form_resize = None
        self._clear_guides()
        return True

    def mousePressEvent(self, ev):
        # Nur die linke Taste bedient Platzieren/Selektieren/Ziehen. Vorher
        # platzierte auch ein Rechtsklick ein scharf geschaltetes Control und
        # startete unsichtbare Auswahlrahmen; ausserdem ueberschrieb eine
        # zweite Taste mitten im Ziehen den Pre-Gesten-Snapshot, womit die
        # ganze Verschiebung nicht mehr rueckgaengig zu machen war.
        if ev.button() != Qt.MouseButton.LeftButton:
            return
        self._nudge_active = False
        p = self._to_draw(ev.position())
        cx, cy = self._to_ctrl(p)
        ctrl = bool(ev.modifiers() & Qt.KeyboardModifier.ControlModifier)
        if self.place_kind:
            self._place_control(self.place_kind, cx, cy)
            self.place_kind = None
            return
        # Resize-Griff -- nur bei genau einem selektierten Control
        if len(self.selection) == 1:
            handle = self._handle_at(p)
            if handle is not None:
                self._resize_handle = handle
                self._pending = self.doc.to_dict()           # Pre-Resize-Snapshot
                return
        # Formular-Resize-Griff (wenn nichts selektiert = Fenster aktiv)
        fh = self._form_handle_at(p)
        if fh is not None:
            self._form_resize = fh
            self._pending = self.doc.to_dict()
            return
        hit = self.doc.control_at(cx, cy)
        if hit is None:                                       # leerer Bereich -> Rubber-Band
            if not ctrl:
                self._select(None)
            self._band = True
            self._band_additive = ctrl
            self._band_start = (cx, cy)
            self._band_now = (cx, cy)
            self.update()
            return
        if ctrl:                                              # Strg+Klick: Toggle
            self._toggle_select(hit)
            self.update()
            return
        if not self._in_selection(hit):                       # neues Einzel-Ziel
            self._select(hit)
        else:                                                 # Teil der Gruppe -> primary
            self.selected = hit
            self.selection_changed.emit(hit)
        self._begin_drag(cx, cy)
        self.update()

    def _begin_drag(self, cx: int, cy: int):
        self._drag = True
        self._pending = self.doc.to_dict()                   # Move = eine Geste = 1 Undo
        if len(self.selection) > 1:
            self._multi = True
            self._drag_origin = (cx, cy)
            self._drag_starts = [(c, c.x, c.y) for c in self.selection]
        else:
            self._multi = False
            self._drag_off = QPoint(cx - self.selected.x, cy - self.selected.y)

    def _multi_move(self, cx: int, cy: int):
        dx = self._snap(cx - self._drag_origin[0])
        dy = self._snap(cy - self._drag_origin[1])
        # Das Delta begrenzen, NICHT jedes Control einzeln: bei einem
        # Einzel-Clamp liefen die Controls am Rand auf, waehrend die anderen
        # weiterwanderten -- eine ausgerichtete Knopfreihe wurde beim Schieben
        # an den linken Rand still zusammengedrueckt.
        dx = self._clamp_delta(dx, [(sx, self._max_x(c)) for c, sx, _ in self._drag_starts])
        dy = self._clamp_delta(dy, [(sy, self._max_y(c)) for c, _, sy in self._drag_starts])
        for c, sx, sy in self._drag_starts:
            c.x = sx + dx; c.y = sy + dy

    @staticmethod
    def _clamp_delta(d: int, starts: list) -> int:
        """Verschiebe-Delta so begrenzen, dass alle Controls im Formular
        bleiben. `starts` = [(startwert, maximum)]. Passt die Gruppe nicht
        vollstaendig hinein, wird nur nach links/oben geklemmt -- lieber ein
        Ueberstand als eine verzerrte Gruppe."""
        lo = -min(s for s, _ in starts)
        hi = min(m - s for s, m in starts)
        return max(lo, min(d, hi)) if hi >= lo else max(lo, d)

    def _max_x(self, c: Control) -> int:
        return max(0, self.doc.w - c.w)

    def _max_y(self, c: Control) -> int:
        return max(0, self.doc.h - TITLE_H - c.h)

    def mouseMoveEvent(self, ev):
        p = self._to_draw(ev.position())
        cx, cy = self._to_ctrl(p)
        if not (ev.buttons() & Qt.MouseButton.LeftButton):
            # Ohne gedrueckte Taste darf keine Geste mehr laufen -- sonst
            # verschiebt/resized blosses Hovern (siehe _finish_gesture).
            if self._finish_gesture():
                self.update()
        if self._band:
            self._band_now = (cx, cy)
            self.update()
            return
        if self._form_resize is not None:
            if "e" in self._form_resize:
                self.doc.w = self._clamp_fw(self._snap(p.x() - PAD))
            if "s" in self._form_resize:
                self.doc.h = self._clamp_fh(self._snap(p.y() - PAD))
            self._resize_to_doc()
            self.form_resized.emit()
            self.doc_changed.emit()
            self.update()
            return
        if self._resize_handle is not None and self.selected is not None:
            c = self.selected
            nx, ny = self._snap(cx), self._snap(cy)
            rx, ry, rw, rh = resize_rect(c.x, c.y, c.w, c.h, self._resize_handle, nx, ny)
            # Der Resize-Pfad klemmte als einziger NICHT auf >= 0: ein Zug am
            # NW-/N-/W-Griff ueber die obere/linke Formularkante hinaus schrieb
            # `x: -56` ins .gbform, zur Laufzeit ragte das Widget aus dem
            # Fenster und wurde weggeclippt. Kante festhalten, Groesse kuerzen.
            if rx < 0:
                rw += rx; rx = 0
            if ry < 0:
                rh += ry; ry = 0
            c.x, c.y, c.w, c.h = rx, ry, max(1, rw), max(1, rh)
            self.selection_changed.emit(c)               # Inspector live aktualisieren
            self.doc_changed.emit()
            self.update()
            return
        if self._drag and self.selected is not None:
            if self._multi:
                self._multi_move(cx, cy)
            else:
                self._move_to(cx - self._drag_off.x(), cy - self._drag_off.y())
            self.selection_changed.emit(self.selected)   # Inspector live aktualisieren
            self.doc_changed.emit()
            self.update()
            return
        # Kein Knopf gedrueckt: Hover-Cursor ueber Resize-Griffen
        if not (ev.buttons() & Qt.MouseButton.LeftButton):
            handle = self._handle_at(p) if len(self.selection) == 1 else None
            if handle is None:
                handle = self._form_handle_at(p)
            self.setCursor(_HANDLE_CURSORS[handle] if handle else Qt.CursorShape.ArrowCursor)

    def mouseReleaseEvent(self, _ev):
        # Gesten-Ende: Rubber-Band auswerten, dann nur EIN Undo-Checkpoint --
        # und auch nur, falls sich wirklich etwas geaendert hat.
        self._finish_gesture()
        self.update()

    def _finish_band(self):
        x0, y0 = self._band_start
        x1, y1 = self._band_now
        rx0, rx1 = sorted((x0, x1))
        ry0, ry1 = sorted((y0, y1))
        hits = [c for c in self.doc.controls
                if c.x < rx1 and c.x + c.w > rx0 and c.y < ry1 and c.y + c.h > ry0]
        if hits:
            self._select_many(hits, additive=self._band_additive)

    def mouseDoubleClickEvent(self, ev):
        if ev.button() != Qt.MouseButton.LeftButton:
            return
        cx, cy = self._to_ctrl(self._to_draw(ev.position()))
        hit = self.doc.control_at(cx, cy)
        if hit is not None:
            # Geste des ersten Klicks sauber abschliessen statt den Snapshot zu
            # verwerfen -- eine Bewegung dazwischen waere sonst nicht undobar.
            self._finish_gesture()
            self._focus_on(hit)         # Mehrfach-Auswahl nicht wegwerfen
            self.update()
            self.handler_requested.emit(hit)

    def _place_control(self, kind: str, cx: int, cy: int):
        """Ein neues Control an (cx, cy) platzieren (von Klick-Platzieren UND
        Drag&Drop genutzt). Setzt einen Undo-Checkpoint + selektiert es."""
        pre = self.doc.to_dict()
        c = self.doc.add(kind, max(self._snap(cx), 0), max(self._snap(cy), 0))
        # Auch ein Klick neben das Formular soll ein erreichbares Control geben.
        c.x = min(c.x, self._max_x(c)); c.y = min(c.y, self._max_y(c))
        if self.commit_history:
            self.commit_history(pre)
        self._select(c)
        self.doc_changed.emit()
        self.update()
        return c

    # -- Drag&Drop aus der Palette --
    def dragEnterEvent(self, ev):
        if ev.mimeData().hasFormat(_CONTROL_MIME):
            ev.acceptProposedAction()

    def dragMoveEvent(self, ev):
        if ev.mimeData().hasFormat(_CONTROL_MIME):
            ev.acceptProposedAction()

    def dropEvent(self, ev):
        md = ev.mimeData()
        if not md.hasFormat(_CONTROL_MIME):
            return
        kind = bytes(md.data(_CONTROL_MIME)).decode("utf-8")
        cx, cy = self._to_ctrl(self._to_draw(ev.position()))
        self.place_kind = None
        self._place_control(kind, cx, cy)
        ev.acceptProposedAction()
        self.setFocus()

    _ARROWS = {
        Qt.Key.Key_Left: (-1, 0), Qt.Key.Key_Right: (1, 0),
        Qt.Key.Key_Up: (0, -1), Qt.Key.Key_Down: (0, 1),
    }

    def keyPressEvent(self, ev):
        if ev.key() in (Qt.Key.Key_Delete, Qt.Key.Key_Backspace) and self.selection:
            # Ein laufender Drag darf hier nicht weiterlaufen -- sein Release
            # haette sonst einen zweiten, leeren Undo-Schritt erzeugt.
            self._drag = False; self._multi = False; self._pending = None
            self._resize_handle = None; self._form_resize = None
            pre = self.doc.to_dict()                         # Undo: Zustand vor dem Loeschen
            for c in list(self.selection):
                self.doc.remove(c)
            if self.commit_history:
                self.commit_history(pre)
            self._select(None)
            self.doc_changed.emit()
            self.update()
        elif ev.key() in self._ARROWS and self.selection:
            dx, dy = self._ARROWS[ev.key()]
            step = GRID if (ev.modifiers() & Qt.KeyboardModifier.ShiftModifier) else 1
            self._nudge(dx * step, dy * step)
        else:
            super().keyPressEvent(ev)

    def _nudge(self, dx: int, dy: int):
        """Selektion per Pfeiltaste verschieben (alle selektierten Controls).
        Eine Tastenfolge (bis Klick/Selektionswechsel) = EIN Undo-Schritt."""
        if not self._nudge_active:
            if self.commit_history:
                self.commit_history(self.doc.to_dict())      # Pre-Nudge-Snapshot
            self._nudge_active = True
        for c in self.selection:
            c.x = min(max(0, c.x + dx), self._max_x(c))
            c.y = min(max(0, c.y + dy), self._max_y(c))
        self.selection_changed.emit(self.selected)
        self.doc_changed.emit()
        self.update()

    def contextMenuEvent(self, ev):
        cx, cy = self._to_ctrl(self._to_draw(ev.pos()))
        hit = self.doc.control_at(cx, cy)
        if hit is not None:
            self._focus_on(hit)
            self.update()
            self.context_menu.emit(ev.globalPos())

    def _in_selection(self, c: Control) -> bool:
        """Identitaets- statt Wertvergleich -- `Control` ist eine dataclass mit
        generiertem `__eq__`, `in` traefe also jedes feldgleiche Control."""
        return any(x is c for x in self.selection)

    def _focus_on(self, c: Control):
        """`c` zum primaeren Control machen. Gehoert es schon zur Auswahl,
        bleibt die Gruppe bestehen -- Rechtsklick/Doppelklick auf ein
        Gruppenmitglied warf sie vorher weg, weshalb das Kontextmenue-Loeschen
        von fuenf markierten Controls nur eines erwischte."""
        if self._in_selection(c):
            self.selected = c
            self._nudge_active = False
            self.selection_changed.emit(c)
        else:
            self._select(c)

    def _select(self, c: Control | None):
        self.selected = c
        self.selection = [c] if c is not None else []
        self._nudge_active = False           # neue Selektion beendet Nudge-Sitzung
        self.selection_changed.emit(c)

    def _toggle_select(self, c: Control):
        """Strg+Klick: Control zur Mehrfach-Selektion hinzu/heraus."""
        if self._in_selection(c):
            self.selection = [x for x in self.selection if x is not c]
            self.selected = self.selection[-1] if self.selection else None
        else:
            self.selection.append(c)
            self.selected = c
        self._nudge_active = False
        self.selection_changed.emit(self.selected)

    def _select_many(self, controls: list, additive: bool = False):
        if additive:
            for c in controls:
                if not self._in_selection(c):
                    self.selection.append(c)
        else:
            self.selection = list(controls)
        self.selected = self.selection[-1] if self.selection else None
        self._nudge_active = False
        self.selection_changed.emit(self.selected)


class _Inspector(QWidget):
    """Eigenschaften + Events des gewaehlten Controls editieren."""
    changed = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self._c: Control | None = None
        self._loading = False
        self._form = QFormLayout(self)
        self.name = QLineEdit(); self.text = QLineEdit()
        self.sx = QSpinBox(); self.sy = QSpinBox(); self.sw = QSpinBox(); self.sh = QSpinBox()
        for s in (self.sx, self.sy, self.sw, self.sh):
            s.setRange(0, 32000)         # 4000 klemmte grosse Controls still ab
        # Mindestens 1x1: bei Breite oder Hoehe 0 trifft `control_at` das
        # Control nicht mehr -- es war weder selektierbar noch sichtbar und
        # nur per Undo zurueckzuholen.
        self.sw.setMinimum(1); self.sh.setMinimum(1)
        self.enabled = QCheckBox("aktiviert")
        self.checked = QCheckBox("angehakt")
        self.on_click = QLineEdit(); self.on_change = QLineEdit()
        self.items = QPlainTextEdit(); self.items.setMaximumHeight(90)
        self.vmin = QDoubleSpinBox(); self.vmax = QDoubleSpinBox(); self.vval = QDoubleSpinBox()
        for s in (self.vmin, self.vmax, self.vval):
            s.setRange(-1e6, 1e6)
        # Farbe (Swatch-Button statt Hex) + Schriftgroesse
        self.color_btn = QPushButton(); self.color_btn.setFixedHeight(22)
        self.color_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.color_btn.clicked.connect(self._pick_color)
        self.sfont = QSpinBox(); self.sfont.setRange(0, 96)
        self.sfont.setToolTip("Schriftgroesse in px (0 = Standard)")
        # Anker (Reflow beim Fenster-Resize): an welchen Kanten klebt das Control?
        self.a_l = QCheckBox("L"); self.a_r = QCheckBox("R")
        self.a_t = QCheckBox("O"); self.a_b = QCheckBox("U")
        for cb, tip in ((self.a_l, "Links"), (self.a_r, "Rechts"),
                        (self.a_t, "Oben"), (self.a_b, "Unten")):
            cb.setToolTip(f"Anker {tip}")
        self._anchor_box = QWidget()
        _ah = QHBoxLayout(self._anchor_box); _ah.setContentsMargins(0, 0, 0, 0)
        for cb in (self.a_l, self.a_r, self.a_t, self.a_b):
            _ah.addWidget(cb)
        self._rows = []
        self._add("Name", self.name)
        self._add("Text", self.text)
        self._add("Farbe", self.color_btn)
        self._add("Schriftgroesse", self.sfont)
        self._add("X", self.sx); self._add("Y", self.sy)
        self._add("Breite", self.sw); self._add("Hoehe", self.sh)
        self._add("Anker", self._anchor_box)
        self._add("on_click", self.on_click)
        self._add("on_change", self.on_change)
        self._add("Items (1/Zeile)", self.items)
        self._add("Min", self.vmin); self._add("Max", self.vmax); self._add("Wert", self.vval)
        self._add("", self.enabled)
        self._add("", self.checked)
        # Signale
        self.name.editingFinished.connect(self._apply)
        self.text.editingFinished.connect(self._apply)
        for s in (self.sx, self.sy, self.sw, self.sh, self.vmin, self.vmax, self.vval):
            s.valueChanged.connect(self._apply)
        self.on_click.editingFinished.connect(self._apply)
        self.on_change.editingFinished.connect(self._apply)
        self.items.textChanged.connect(self._apply)
        self.enabled.toggled.connect(self._apply)
        self.checked.toggled.connect(self._apply)
        self.sfont.valueChanged.connect(self._apply)
        for cb in (self.a_l, self.a_r, self.a_t, self.a_b):
            cb.toggled.connect(self._apply)
        self.set_control(None)

    def _update_color_btn(self):
        if self._c is not None:
            c = _col(self._c.color)
            self.color_btn.setText(f"#{self._c.color & 0xFFFFFF:06X}")
            fg = "#000" if (c.red() + c.green() + c.blue()) > 360 else "#fff"
            self.color_btn.setStyleSheet(
                f"background:{c.name()}; color:{fg}; border:1px solid #555;")

    def _pick_color(self):
        if self._c is None:
            return
        col = QColorDialog.getColor(_col(self._c.color), self, "Farbe waehlen")
        if col.isValid():
            self._c.color = (col.red() << 16) | (col.green() << 8) | col.blue()
            self._update_color_btn()
            self.changed.emit()

    def _add(self, label, widget):
        self._form.addRow(label, widget)
        self._rows.append((label, widget))

    def _show(self, widget, on: bool):
        widget.setVisible(on)
        lbl = self._form.labelForField(widget)
        if lbl is not None:
            lbl.setVisible(on)

    def set_control(self, c: Control | None):
        self._c = c
        self._loading = True
        if c is None:
            for _, w in self._rows:
                self._show(w, False)
            self._loading = False
            return
        sp = palette_spec(c.kind)
        self.name.setText(c.name); self.text.setText(c.text)
        self.sx.setValue(c.x); self.sy.setValue(c.y); self.sw.setValue(c.w); self.sh.setValue(c.h)
        self.on_click.setText(c.on_click); self.on_change.setText(c.on_change)
        self.items.setPlainText("\n".join(c.items))
        self.vmin.setValue(c.min); self.vmax.setValue(c.max); self.vval.setValue(c.value)
        self.enabled.setChecked(c.enabled); self.checked.setChecked(c.checked)
        self.sfont.setValue(c.font_size); self._update_color_btn()
        a = c.anchor or "lt"
        self.a_l.setChecked("l" in a); self.a_r.setChecked("r" in a)
        self.a_t.setChecked("t" in a); self.a_b.setChecked("b" in a)
        has_text = bool(sp and sp.has_text)
        has_items = bool(sp and sp.has_items)
        events = sp.events if sp else ()
        is_range = c.kind in ("slider", "progress")
        is_check = c.kind in ("checkbox", "radio")
        for w in (self.name, self.color_btn, self.sfont, self.sx, self.sy,
                  self.sw, self.sh, self._anchor_box, self.enabled):
            self._show(w, True)
        self._show(self.text, has_text)
        self._show(self.items, has_items)
        self._show(self.on_click, "on_click" in events)
        self._show(self.on_change, "on_change" in events)
        self._show(self.vmin, is_range); self._show(self.vmax, is_range); self._show(self.vval, is_range)
        self._show(self.checked, is_check)
        self._loading = False

    def _apply(self):
        if self._loading or self._c is None:
            return
        c = self._c
        c.name = self.name.text().strip()
        c.text = self.text.text()
        c.x, c.y, c.w, c.h = self.sx.value(), self.sy.value(), self.sw.value(), self.sh.value()
        c.on_click = self.on_click.text().strip()
        c.on_change = self.on_change.text().strip()
        items = [ln for ln in self.items.toPlainText().splitlines() if ln != ""]
        c.items = items
        if c.kind == "dropdown" and items and c.sel < 0:
            c.sel = 0
        c.min, c.max, c.value = self.vmin.value(), self.vmax.value(), self.vval.value()
        c.enabled = self.enabled.isChecked()
        c.checked = self.checked.isChecked()
        c.font_size = self.sfont.value()
        a = ("l" if self.a_l.isChecked() else "") + ("r" if self.a_r.isChecked() else "") \
            + ("t" if self.a_t.isChecked() else "") + ("b" if self.a_b.isChecked() else "")
        c.anchor = a or "lt"
        self.changed.emit()


class _WindowInspector(QWidget):
    """Eigenschaften des Formulars selbst (wie Xojos Fenster-Inspector). Wird
    angezeigt, wenn KEIN Control selektiert ist."""
    changed = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc: FormDoc | None = None
        self._loading = False
        f = QFormLayout(self)
        self.title = QLineEdit()
        self.sw = QSpinBox(); self.sh = QSpinBox()
        self.minw = QSpinBox(); self.minh = QSpinBox()
        self.maxw = QSpinBox(); self.maxh = QSpinBox()
        for s in (self.sw, self.sh, self.minw, self.minh, self.maxw, self.maxh):
            s.setRange(0, 32000)
        self.sw.setMinimum(40); self.sh.setMinimum(40)
        self.movable = QCheckBox("beweglich")
        self.closable = QCheckBox("schliessbar")
        self.resizable = QCheckBox("groessenveraenderbar")
        self.visible = QCheckBox("sichtbar")
        f.addRow("Titel", self.title)
        f.addRow("Breite", self.sw); f.addRow("Hoehe", self.sh)
        f.addRow("Min. Breite", self.minw); f.addRow("Min. Hoehe", self.minh)
        f.addRow("Max. Breite", self.maxw); f.addRow("Max. Hoehe", self.maxh)
        f.addRow("", self.movable); f.addRow("", self.closable)
        f.addRow("", self.resizable); f.addRow("", self.visible)
        self.title.editingFinished.connect(self._apply)
        for s in (self.sw, self.sh, self.minw, self.minh, self.maxw, self.maxh):
            s.valueChanged.connect(self._apply)
        for c in (self.movable, self.closable, self.resizable, self.visible):
            c.toggled.connect(self._apply)

    def set_doc(self, doc: FormDoc):
        self.doc = doc
        self._loading = True
        if doc is not None:
            self.title.setText(doc.title)
            self.sw.setValue(doc.w); self.sh.setValue(doc.h)
            self.minw.setValue(doc.min_w); self.minh.setValue(doc.min_h)
            self.maxw.setValue(doc.max_w); self.maxh.setValue(doc.max_h)
            self.movable.setChecked(doc.movable); self.closable.setChecked(doc.closable)
            self.resizable.setChecked(doc.resizable); self.visible.setChecked(doc.visible)
        self._loading = False

    def _apply(self):
        if self._loading or self.doc is None:
            return
        d = self.doc
        d.title = self.title.text()
        d.w, d.h = self.sw.value(), self.sh.value()
        d.min_w, d.min_h = self.minw.value(), self.minh.value()
        d.max_w, d.max_h = self.maxw.value(), self.maxh.value()
        d.movable, d.closable = self.movable.isChecked(), self.closable.isChecked()
        d.resizable, d.visible = self.resizable.isChecked(), self.visible.isChecked()
        self.changed.emit()


class _CodePanel(QWidget):
    """Integrierter Code-Editor: pro Event-Handler ein GameBasic-Body. Die
    Combo listet die Handler des Formulars, der Editor zeigt/aendert den Body
    des gewaehlten (gespeichert in `doc.code[name]`)."""
    edited = Signal()             # Body geaendert
    session_started = Signal()    # ein Handler wurde frisch geladen (Undo-Basis)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc: FormDoc | None = None
        self.current: str | None = None
        self._loading = False

        lay = QVBoxLayout(self)
        top = QHBoxLayout()
        top.addWidget(QLabel("Handler:"))
        self.combo = QComboBox(); self.combo.setMinimumWidth(200)
        top.addWidget(self.combo, 1)
        lay.addLayout(top)
        self.sig = QLabel("")
        self.sig.setStyleSheet("color:#5fb6d6;")
        lay.addWidget(self.sig)
        self.editor = QPlainTextEdit()
        self.editor.setLineWrapMode(QPlainTextEdit.LineWrapMode.NoWrap)
        f = QFont("Consolas"); f.setStyleHint(QFont.StyleHint.Monospace); f.setPointSize(10)
        self.editor.setFont(f)
        self.editor.setTabStopDistance(4 * self.editor.fontMetrics().horizontalAdvance(" "))
        self._hl = None
        if GBHighlighter is not None:
            self._hl = GBHighlighter(self.editor.document())
        lay.addWidget(self.editor, 1)

        self.combo.currentIndexChanged.connect(self._on_combo)
        self.editor.textChanged.connect(self._on_text)
        self._show_empty()

    def set_doc(self, doc: FormDoc):
        self.doc = doc
        self.current = None
        self.refresh()

    def refresh(self):
        """Combo neu aus den Handler-Namen des Formulars fuellen (Auswahl halten)."""
        names = self.doc.handler_names() if self.doc else []
        self._loading = True
        self.combo.clear()
        self.combo.addItems(names)
        self._loading = False
        if self.current in names:
            self.show_handler(self.current)
        elif names:
            self.show_handler(names[0])
        else:
            self._show_empty()

    def show_handler(self, name: str):
        if not self.doc or name not in self.doc.handler_names():
            return
        self._loading = True
        idx = self.combo.findText(name)
        if idx >= 0:
            self.combo.setCurrentIndex(idx)
        self.current = name
        self.sig.setText(f"SUB {name}()      …      END SUB")
        self.editor.setReadOnly(False)
        self.editor.setPlainText(self.doc.code.get(name, ""))
        self._loading = False
        self.session_started.emit()

    def focus_editor(self):
        self.editor.setFocus()

    def detach_highlighter(self):
        """Highlighter vom Dokument loesen -- verhindert einen Use-after-free
        beim Interpreter-Shutdown (QSyntaxHighlighter ueberlebt sonst die
        Teardown-Race von Dokument + QApplication)."""
        if self._hl is not None:
            self._hl.setDocument(None)
            self._hl = None

    def _show_empty(self):
        self._loading = True
        self.current = None
        self.editor.clear()
        self.editor.setReadOnly(True)
        self.sig.setText("(kein Handler — Doppelklick auf ein Control)")
        self._loading = False

    def _on_combo(self):
        if self._loading or not self.doc:
            return
        name = self.combo.currentText()
        if name:
            self.show_handler(name)

    def _on_text(self):
        if self._loading or not self.doc or self.current is None:
            return
        self.doc.code[self.current] = self.editor.toPlainText()
        self.edited.emit()


class _OpenForm:
    """Ein im Designer geoeffnetes Formular mit eigenem Zustand: Dokument, Pfad,
    Undo-Historie und Dirty-Flag. (Undo ist pro Formular -- nicht uebergreifend.)"""
    def __init__(self, doc: FormDoc, path: Path | None = None):
        self.doc = doc
        self.path = path
        self.history = History()
        self.dirty = False
        # Stand beim letzten Speichern/Laden -- damit ein Undo zurueck auf die
        # Datei den Stern wieder loescht, statt ihn fuer immer stehen zu lassen.
        self.saved: dict | None = doc.to_dict() if path is not None else None


class FormDesigner(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = Path(project_root)
        # Multi-Form-Zustand
        self.forms: list[_OpenForm] = []
        self.active_index: int = -1
        self.project = FormProject()
        self.project_path: Path | None = None
        self._main_form: _OpenForm | None = None   # Startformular (Objekt, nicht Pfad)
        self.unresolved: list[str] = []            # beim Laden fehlende .gbform
        self._suppress_row = False
        self.setWindowTitle("GameBasic Form-Designer")
        self.resize(1500, 950)

        self.canvas = _Canvas()
        scroll = QScrollArea(); scroll.setWidget(self.canvas); scroll.setWidgetResizable(False)
        self.setCentralWidget(scroll)

        # Formular-Liste (Multi-Form-Navigator)
        self.form_list = QListWidget()
        self.form_list.currentRowChanged.connect(self._on_form_row)
        self._dock("Formulare", self.form_list, Qt.DockWidgetArea.LeftDockWidgetArea)

        # Palette (grafische Icons + Drag&Drop)
        self.palette = _PaletteList()
        for sp in PALETTE:
            it = QListWidgetItem(_palette_icon(sp.kind), sp.label)
            it.setData(Qt.ItemDataRole.UserRole, sp.kind)
            it.setToolTip(f"{sp.label} — ziehen oder klicken+platzieren")
            self.palette.addItem(it)
        self.palette.itemClicked.connect(self._arm_place)
        pdock = self._dock("Controls", self.palette, Qt.DockWidgetArea.LeftDockWidgetArea)
        pdock.setMinimumWidth(180)

        # Inspector -- Stack: Control-Eigenschaften ODER Fenster-Eigenschaften
        self.inspector = _Inspector()
        self.win_inspector = _WindowInspector()
        self._insp_stack = QStackedWidget()
        self._insp_stack.addWidget(self.inspector)       # 0: Control selektiert
        self._insp_stack.addWidget(self.win_inspector)   # 1: Fenster (nichts selektiert)
        self._dock("Inspector", self._insp_stack, Qt.DockWidgetArea.RightDockWidgetArea)

        # Code-Editor (integriert)
        self.code_panel = _CodePanel()
        self.code_dock = self._dock("Code", self.code_panel, Qt.DockWidgetArea.BottomDockWidgetArea)

        # Undo/Redo-Edit-Sessions (transient, pro Selektion/Handler)
        self.canvas.commit_history = self._commit_history
        self._insp_baseline: dict | None = None
        self._insp_dirty = False
        self._code_baseline: dict | None = None
        self._code_dirty = False
        self._clip: dict | None = None            # Clipboard fuer Kopieren/Einfuegen

        self.canvas.selection_changed.connect(self.inspector.set_control)
        self.canvas.selection_changed.connect(self._on_selection_changed)
        self.canvas.selection_changed.connect(self._update_status)
        self.canvas.doc_changed.connect(self._mark_dirty)
        self.canvas.doc_replaced.connect(self.code_panel.set_doc)
        self.canvas.form_resized.connect(self._on_form_resized)
        self.canvas.handler_requested.connect(self._open_handler)
        self.canvas.context_menu.connect(self._show_context_menu)
        self.inspector.changed.connect(self._on_inspector_changed)
        self.win_inspector.changed.connect(self._on_window_changed)
        self.code_panel.session_started.connect(self._on_code_session)
        self.code_panel.edited.connect(self._on_code_edited)

        self._status = QLabel("")                 # Live-Anzeige der Selektion
        self.statusBar().addPermanentWidget(self._status)
        self._zoom_lbl = QLabel("100 %")
        self.statusBar().addPermanentWidget(self._zoom_lbl)
        self.canvas.zoom_changed.connect(
            lambda z: self._zoom_lbl.setText(f"{round(z * 100)} %"))

        self._build_menu()
        self._build_arrange_toolbar()
        self._add_open_form(FormDoc())     # ein leeres Start-Formular

    # -- Ungespeichert-Schutz ------------------------------------------------
    def _confirm_dirty(self) -> bool:
        """True = fortfahren. Fragt fuer ALLE ungespeicherten Formulare, nicht
        nur das aktive -- vorher gingen beim Schliessen des Fensters oder beim
        Oeffnen eines Projekts saemtliche Aenderungen kommentarlos verloren
        (samt ihrer Undo-Historien)."""
        dirty = [of for of in self.forms if of.dirty]
        if not dirty:
            return True
        names = ", ".join(of.path.name if of.path else "(unbenannt)" for of in dirty)
        r = QMessageBox.question(
            self, "Ungespeicherte Aenderungen",
            f"Nicht gespeichert: {names}\n\nAenderungen speichern?",
            QMessageBox.StandardButton.Save | QMessageBox.StandardButton.Discard
            | QMessageBox.StandardButton.Cancel)
        if r == QMessageBox.StandardButton.Cancel:
            return False
        if r == QMessageBox.StandardButton.Save:
            return self.save_all()
        return True

    def closeEvent(self, ev):
        if not self._confirm_dirty():
            ev.ignore()
            return
        self.code_panel.detach_highlighter()
        super().closeEvent(ev)

    def _dock(self, title, widget, area):
        d = QDockWidget(title, self)
        d.setWidget(widget)
        self.addDockWidget(area, d)
        return d

    def _build_menu(self):
        m = self.menuBar().addMenu("&Datei")
        def act(name, shortcut, fn, menu=m):
            a = QAction(name, self)
            if shortcut:
                a.setShortcut(QKeySequence(shortcut))
            a.triggered.connect(fn); menu.addAction(a); return a
        act("Neues Formular", "Ctrl+N", self.new_form)
        act("Formular oeffnen...", "Ctrl+O", self.open_form)
        act("Formular schliessen", "Ctrl+W", self.close_form)
        m.addSeparator()
        act("Speichern", "Ctrl+S", self.save_form)
        act("Speichern unter...", "Ctrl+Shift+S", self.save_form_as)
        act("Alle speichern", "Ctrl+Alt+S", self.save_all)
        m.addSeparator()
        act("Projekt oeffnen...", None, self.open_project)
        act("Projekt speichern...", None, self.save_project)
        act("Als Startformular setzen", None, self.set_main_form)
        m.addSeparator()
        act("GB-Code exportieren...", None, self.export_gb_code)
        act("Ausfuehren (gbrt)", "F5", self.run_form)

        e = self.menuBar().addMenu("&Bearbeiten")
        self.act_undo = act("Rueckgaengig", "Ctrl+Z", self.undo, menu=e)
        self.act_redo = act("Wiederholen", "Ctrl+Y", self.redo, menu=e)
        # Zweites, uebliches Redo-Kuerzel (Strg+Umschalt+Z)
        redo2 = QAction(self)
        redo2.setShortcut(QKeySequence("Ctrl+Shift+Z"))
        redo2.triggered.connect(self.redo)
        self.addAction(redo2)
        e.addSeparator()
        # Edit-Ops: Shortcut NUR wenn die Canvas fokussiert ist -- sonst wuerde
        # z.B. Strg+C/V die Textbearbeitung im Code-/Inspector-Panel kapern.
        def edit_act(label, key, fn):
            a = QAction(label, self)
            a.setShortcut(QKeySequence(key))
            a.setShortcutContext(Qt.ShortcutContext.WidgetWithChildrenShortcut)
            a.triggered.connect(fn)
            e.addAction(a); self.canvas.addAction(a)
            return a
        edit_act("Duplizieren", "Ctrl+D", self.duplicate_selected)
        edit_act("Kopieren", "Ctrl+C", self.copy_selected)
        edit_act("Einfuegen", "Ctrl+V", self.paste_clip)
        act("Loeschen", None, self.delete_selected, menu=e)   # Del macht die Canvas
        e.addSeparator()
        edit_act("Nach vorne", "Ctrl+]", self.raise_selected)
        edit_act("Nach hinten", "Ctrl+[", self.lower_selected)

        v = self.menuBar().addMenu("&Ansicht")
        self.act_snap = QAction("Am Raster ausrichten", self, checkable=True)
        self.act_snap.setChecked(self.canvas.snap_grid)
        self.act_snap.setShortcut(QKeySequence("Ctrl+G"))
        self.act_snap.toggled.connect(self._toggle_snap)
        v.addAction(self.act_snap)
        self.act_rulers = QAction("Lineale", self, checkable=True)
        self.act_rulers.setChecked(self.canvas.show_rulers)
        self.act_rulers.toggled.connect(self._toggle_rulers)
        v.addAction(self.act_rulers)
        v.addSeparator()
        act("Vergroessern", "Ctrl+=", lambda: self.canvas.set_zoom(self.canvas.zoom * 1.25), menu=v)
        act("Verkleinern", "Ctrl+-", lambda: self.canvas.set_zoom(self.canvas.zoom / 1.25), menu=v)
        act("Zoom 100%", "Ctrl+0", lambda: self.canvas.set_zoom(1.0), menu=v)

        # Anordnen (Mehrfach-Auswahl): Ausrichten / Gleiche Groesse / Verteilen
        ar = self.menuBar().addMenu("&Anordnen")
        for label, edge in (("Linksbuendig", "left"), ("Rechtsbuendig", "right"),
                            ("Oben buendig", "top"), ("Unten buendig", "bottom"),
                            ("Zentriert horizontal", "center_h"),
                            ("Zentriert vertikal", "center_v")):
            act(label, None, (lambda e: lambda: self._align(e))(edge), menu=ar)
        ar.addSeparator()
        act("Gleiche Breite", None, lambda: self._same_size("w"), menu=ar)
        act("Gleiche Hoehe", None, lambda: self._same_size("h"), menu=ar)
        act("Gleiche Groesse", None, lambda: self._same_size("both"), menu=ar)
        ar.addSeparator()
        act("Horizontal verteilen", None, lambda: self._distribute("h"), menu=ar)
        act("Vertikal verteilen", None, lambda: self._distribute("v"), menu=ar)

    def _build_arrange_toolbar(self):
        """Anordnen-Befehle als grafische Toolbar (Xojo-Stil), schneller als das Menue."""
        tb = QToolBar("Anordnen")
        tb.setIconSize(QSize(22, 22))
        self.addToolBar(tb)
        self.arrange_bar = tb
        groups = [
            [("left", "Linksbuendig", lambda: self._align("left")),
             ("center_h", "Zentriert horizontal", lambda: self._align("center_h")),
             ("right", "Rechtsbuendig", lambda: self._align("right")),
             ("top", "Oben buendig", lambda: self._align("top")),
             ("center_v", "Zentriert vertikal", lambda: self._align("center_v")),
             ("bottom", "Unten buendig", lambda: self._align("bottom"))],
            [("same_w", "Gleiche Breite", lambda: self._same_size("w")),
             ("same_h", "Gleiche Hoehe", lambda: self._same_size("h")),
             ("same_both", "Gleiche Groesse", lambda: self._same_size("both"))],
            [("dist_h", "Horizontal verteilen", lambda: self._distribute("h")),
             ("dist_v", "Vertikal verteilen", lambda: self._distribute("v"))],
        ]
        for gi, group in enumerate(groups):
            if gi:
                tb.addSeparator()
            for kind, tip, fn in group:
                a = QAction(_arrange_icon(kind), tip, self)
                a.setToolTip(tip)
                a.triggered.connect(fn)
                tb.addAction(a)

    def _toggle_snap(self, on: bool):
        self.canvas.snap_grid = on
        self.canvas.update()

    def _toggle_rulers(self, on: bool):
        self.canvas.show_rulers = on
        self.canvas.update()

    # -- Anordnen (auf die Mehrfach-Auswahl, je mit Undo-Checkpoint) --
    def _arrange(self, op, min_n: int = 2):
        sel = list(self.canvas.selection)
        if len(sel) < min_n:
            self.statusBar().showMessage(f"Mindestens {min_n} Controls auswaehlen.", 2500)
            return
        pre = self.canvas.doc.to_dict()
        op(sel)
        self._commit_history(pre)
        self.canvas.update()
        self._mark_dirty()

    def _align(self, edge: str):
        self._arrange(lambda s: self.canvas.doc.align(s, edge))

    def _same_size(self, dim: str):
        # Referenz = primaeres (zuletzt geklicktes) Control
        self._arrange(lambda s: self.canvas.doc.same_size(s, self.canvas.selected, dim))

    def _distribute(self, axis: str):
        self._arrange(lambda s: self.canvas.doc.distribute(s, axis), min_n=3)

    # -- Aktive Form + Navigator ------------------------------------------
    @property
    def active(self) -> _OpenForm:
        return self.forms[self.active_index]

    @property
    def history(self) -> History:
        return self.active.history

    @property
    def path(self) -> Path | None:
        return self.active.path if 0 <= self.active_index < len(self.forms) else None

    @path.setter
    def path(self, v):
        self.active.path = v

    def _add_open_form(self, doc: FormDoc, path: Path | None = None, switch: bool = True):
        of = _OpenForm(doc, path)
        self.forms.append(of)
        if switch:
            self._switch_to(len(self.forms) - 1)
        else:
            self._refresh_form_list()
        return of

    def _switch_to(self, index: int):
        if not (0 <= index < len(self.forms)):
            return
        self.active_index = index
        self._insp_baseline = None; self._insp_dirty = False
        self._code_baseline = None; self._code_dirty = False
        self.canvas.set_doc(self.active.doc)   # doc_replaced -> Code-Panel
        self._refresh_history_actions()
        self._refresh_form_list()
        self._update_title()

    def _set_active_doc(self, doc: FormDoc):
        """Aktives Dokument ersetzen (Undo/Redo) -- haelt forms + canvas synchron."""
        self.active.doc = doc
        self.canvas.set_doc(doc)

    def _refresh_form_list(self):
        self._suppress_row = True
        self.form_list.clear()
        for of in self.forms:
            title = of.doc.title or "(Formular)"
            star = " *" if of.dirty else ""
            # Primaer das gemerkte Objekt, sonst der Manifest-Pfad. Der reine
            # String-Vergleich verfehlte ein von Hand mit Backslashes
            # geschriebenes Manifest und liess die Krone verschwinden.
            crown = "★ " if (of is self._main_form or
                             (self._main_form is None and of.path and self.project.main
                              and self._rel(of.path) == self.project.main)) else ""
            self.form_list.addItem(f"{crown}{title}{star}")
        if 0 <= self.active_index < len(self.forms):
            self.form_list.setCurrentRow(self.active_index)
        self._suppress_row = False

    def _on_form_row(self, row: int):
        if self._suppress_row or row < 0 or row == self.active_index:
            return
        self._switch_to(row)

    def _rel(self, p: Path) -> str:
        """Pfad relativ zum Projekt-Verzeichnis (fuer das `.gbproj`-Manifest).

        `os.path.relpath` statt `Path.relative_to`: letzteres kann nicht nach
        oben laufen, und der Fallback schrieb dann den BLOSSEN DATEINAMEN ins
        Manifest -- eine Form in `../shared/` war beim naechsten Oeffnen des
        Projekts spurlos verschwunden. Auf einem anderen Laufwerk (relpath
        wirft) bleibt der Absolutpfad stehen, der wenigstens auffindbar ist."""
        if self.project_path is not None:
            try:
                return Path(os.path.relpath(Path(p), self.project_path.parent)).as_posix()
            except ValueError:
                return Path(p).as_posix()
        return Path(p).name

    # -- Undo/Redo --
    def _commit_history(self, snapshot: dict):
        """Vom Canvas gerufen (Platzieren/Loeschen/Drag/Resize): Checkpoint setzen."""
        self.history.push(snapshot)
        self._refresh_history_actions()

    def _on_selection_changed(self, c):
        """Neue Selektion -> Inspector-Stack umschalten (Control vs. Fenster) +
        Basis fuer eine evtl. folgende Inspector-Edit-Session."""
        self._insp_baseline = self.canvas.doc.to_dict()
        self._insp_dirty = False
        if c is None:                       # nichts selektiert -> Fenster-Inspector
            self.win_inspector.set_doc(self.canvas.doc)
            self._insp_stack.setCurrentWidget(self.win_inspector)
        else:
            self._insp_stack.setCurrentWidget(self.inspector)

    def _on_window_changed(self):
        """Fenster-Inspector-Aenderung: Edits einer Sitzung = EIN Undo-Schritt."""
        if not self._insp_dirty and self._insp_baseline is not None:
            self.history.push(self._insp_baseline)
            self._insp_dirty = True
            self._refresh_history_actions()
        self.canvas._resize_to_doc()
        self.canvas.update()
        self._refresh_form_list()           # Titel evtl. geaendert
        self._mark_dirty()

    def _on_form_resized(self):
        """Formular per Griff vergroessert/verkleinert -> Fenster-Inspector live."""
        self.win_inspector.set_doc(self.canvas.doc)
        self._mark_dirty()

    def _on_inspector_changed(self):
        """Inspector-Aenderung. Alle Edits einer Selektion = EIN Undo-Schritt."""
        if not self._insp_dirty and self._insp_baseline is not None:
            self.history.push(self._insp_baseline)
            self._insp_dirty = True
            self._refresh_history_actions()
        self.canvas.update()
        self.code_panel.refresh()      # evtl. umbenannte Handler in die Combo uebernehmen
        self._mark_dirty()

    def _refresh_history_actions(self):
        self.act_undo.setEnabled(self.history.can_undo)
        self.act_redo.setEnabled(self.history.can_redo)

    def undo(self):
        if not self.history.can_undo:
            return
        prev = self.history.undo(self.canvas.doc.to_dict())
        self._set_active_doc(FormDoc.from_dict(prev))
        self._refresh_history_actions()
        self._resync_dirty()

    def redo(self):
        if not self.history.can_redo:
            return
        nxt = self.history.redo(self.canvas.doc.to_dict())
        self._set_active_doc(FormDoc.from_dict(nxt))
        self._refresh_history_actions()
        self._resync_dirty()

    # -- Edit-Ops (Control-bezogen, je mit Undo-Checkpoint) --
    def _control_op(self, mutate, select=None):
        """`mutate(doc, c)` mit Undo-Checkpoint ausfuehren. `select`: None = nichts
        aendern, 'result' = Rueckgabe selektieren, 'none' = abwaehlen."""
        c = self.canvas.selected
        if c is None:
            return
        pre = self.canvas.doc.to_dict()
        res = mutate(self.canvas.doc, c)
        self._commit_history(pre)
        if select == "result":
            self.canvas._select(res)
        elif select == "none":
            self.canvas._select(None)
        self.canvas.update()
        self._mark_dirty()

    def duplicate_selected(self):
        self._control_op(lambda d, c: d.duplicate(c), select="result")

    def delete_selected(self):
        sel = list(self.canvas.selection)
        if not sel:
            return
        pre = self.canvas.doc.to_dict()
        for c in sel:
            self.canvas.doc.remove(c)
        self._commit_history(pre)
        self.canvas._select(None)
        self.canvas.update()
        self._mark_dirty()

    def raise_selected(self):
        self._control_op(lambda d, c: d.to_front(c))

    def lower_selected(self):
        self._control_op(lambda d, c: d.to_back(c))

    def copy_selected(self):
        c = self.canvas.selected
        if c is not None:
            self._clip = c.to_dict()
            self.statusBar().showMessage(f"Kopiert: {c.name}", 2000)

    def paste_clip(self):
        if not self._clip:
            return
        pre = self.canvas.doc.to_dict()
        nc = self.canvas.doc.clone_from_dict(self._clip)
        self._commit_history(pre)
        self.canvas._select(nc)
        self.canvas.update()
        self._mark_dirty()

    def _show_context_menu(self, gpos):
        if self.canvas.selected is None:
            return
        menu = QMenu(self)
        menu.addAction("Duplizieren", self.duplicate_selected)
        menu.addAction("Kopieren", self.copy_selected)
        menu.addAction("Loeschen", self.delete_selected)
        menu.addSeparator()
        menu.addAction("Nach vorne", self.raise_selected)
        menu.addAction("Nach hinten", self.lower_selected)
        menu.exec(gpos)

    def _update_status(self, c):
        n = len(self.canvas.selection)
        if n > 1:
            self._status.setText(f"{n} Controls ausgewaehlt")
        elif c is not None:
            self._status.setText(f"{c.name}    x={c.x} y={c.y}    {c.w}×{c.h}")
        else:
            self._status.setText("")

    # -- Code-Editor --
    def _open_handler(self, c: Control):
        """Doppelklick auf ein Control: Handler anlegen/anspringen + fokussieren."""
        if self.canvas.doc.primary_event(c) is None:
            self.statusBar().showMessage(f"{c.kind}: kein Event-Handler moeglich.", 3000)
            return
        pre = self.canvas.doc.to_dict()
        name = self.canvas.doc.ensure_handler(c)
        if pre != self.canvas.doc.to_dict():
            self._commit_history(pre)             # Handler-Erzeugung = Undo-Schritt
        self.code_panel.refresh()
        self.code_panel.show_handler(name)
        self.inspector.set_control(c)             # neuer Handler-Name im Inspector
        self.canvas.update()
        self.code_dock.show(); self.code_dock.raise_()
        self.code_panel.focus_editor()
        self._mark_dirty()

    def _on_code_session(self):
        self._code_baseline = self.canvas.doc.to_dict()
        self._code_dirty = False

    def _on_code_edited(self):
        """Code-Edits einer Handler-Sitzung = EIN Undo-Schritt."""
        if not self._code_dirty and self._code_baseline is not None:
            self.history.push(self._code_baseline)
            self._code_dirty = True
            self._refresh_history_actions()
        self._mark_dirty()

    # -- Aktionen --
    def _mark_dirty(self):
        was = self.active.dirty
        self.active.dirty = True
        self._update_title()
        if not was:
            self._refresh_form_list()    # nur bei Uebergang -> Stern erscheint

    def _resync_dirty(self):
        """Dirty gegen den gespeicherten Stand neu bestimmen (nach Undo/Redo).
        Ein Undo bis zurueck zur Datei liess den Stern sonst fuer immer
        stehen -- er wurde bedeutungslos und verdeckte echte Aenderungen."""
        of = self.active
        of.dirty = of.saved is None or self.canvas.doc.to_dict() != of.saved
        self._update_title()
        self._refresh_form_list()        # Navigator lief nach Undo sonst nach

    def _update_title(self):
        of = self.active
        name = of.path.name if of.path else "(unbenannt)"
        star = "*" if of.dirty else ""
        proj = f"  [{self.project_path.name}]" if self.project_path else ""
        self.setWindowTitle(f"GameBasic Form-Designer{proj}  --  {name}{star}")

    def new_form(self):
        self._add_open_form(FormDoc())

    def open_form(self):
        fn, _ = QFileDialog.getOpenFileName(self, "Formular oeffnen", str(self.project_root),
                                            "GameBasic-Form (*.gbform);;Alle (*.*)")
        if not fn:
            return
        # Schon offen? Dann dorthin wechseln statt einen zweiten Puffer
        # anzulegen -- sonst arbeitet man abwechselnd in beiden und der
        # zuletzt gespeicherte ueberschreibt den anderen ersatzlos.
        target = Path(fn).resolve()
        for i, of in enumerate(self.forms):
            if of.path is not None and Path(of.path).resolve() == target:
                self._switch_to(i)
                self.statusBar().showMessage(f"{target.name} ist bereits geoeffnet.", 3000)
                return
        try:
            self._add_open_form(FormDoc.load(fn), Path(fn))
        except Exception as e:  # noqa: BLE001
            QMessageBox.critical(self, "Fehler", f"Konnte nicht laden:\n{e}")

    def close_form(self):
        if self.active.dirty:
            r = QMessageBox.question(
                self, "Formular schliessen",
                "Das Formular hat ungespeicherte Aenderungen.",
                QMessageBox.StandardButton.Save | QMessageBox.StandardButton.Discard
                | QMessageBox.StandardButton.Cancel)
            if r == QMessageBox.StandardButton.Cancel:
                return
            # Vorher gab es nur Ja/Nein -- kein Weg, beim Schliessen zu sichern.
            if r == QMessageBox.StandardButton.Save and not self.save_form():
                return
        i = self.active_index
        if self.forms[i] is self._main_form:
            self._main_form = None
        del self.forms[i]
        if not self.forms:
            self._add_open_form(FormDoc())        # nie ganz leer
        else:
            self._switch_to(min(i, len(self.forms) - 1))

    def _write(self, fn: str, write) -> bool:
        """Schreibvorgang mit Fehlerdialog. Vorher hatte KEIN Speicherpfad eine
        Fehlerbehandlung: auf ein volles/schreibgeschuetztes Ziel oder einen
        getrennten Netzpfad flog ein roher Traceback, im UI passierte nichts --
        der Nutzer hielt die Datei fuer gespeichert."""
        try:
            write()
            return True
        except OSError as e:
            QMessageBox.critical(self, "Speichern fehlgeschlagen",
                                 f"{fn}\n\n{e}")
            return False

    def save_form(self):
        if self.path is None:
            return self.save_form_as()
        if not self._write(str(self.path), lambda: self.canvas.doc.save(str(self.path))):
            return False
        self._mark_saved()
        return True

    def save_form_as(self):
        fn, _ = QFileDialog.getSaveFileName(self, "Formular speichern", str(self.project_root),
                                            "GameBasic-Form (*.gbform)")
        if not fn:
            return False
        if not fn.endswith(".gbform"):
            fn += ".gbform"
        # Pfad ERST nach erfolgreichem Schreiben uebernehmen -- sonst zeigte der
        # Titel nach einem Fehlschlag eine Datei an, die nie existiert hat, und
        # ein spaeteres Strg+S schrieb still an denselben kaputten Ort.
        if not self._write(fn, lambda: self.canvas.doc.save(fn)):
            return False
        self.path = Path(fn)
        self._mark_saved()
        return True

    def _mark_saved(self):
        """Formular als gespeichert markieren + den Stand merken, damit ein
        Undo zurueck zur Datei den Stern wieder verschwinden laesst."""
        of = self.active
        of.dirty = False
        of.saved = self.canvas.doc.to_dict()
        self._update_title(); self._refresh_form_list()

    def save_all(self) -> bool:
        start = self.active_index
        ok = True
        for i in range(len(self.forms)):
            if self.forms[i].dirty or self.forms[i].path is None:
                self._switch_to(i)
                if not self.save_form():
                    ok = False
                    break                          # Abbruch im Speichern-Dialog
        self._switch_to(min(start, len(self.forms) - 1))
        if not ok:
            self.statusBar().showMessage("Speichern abgebrochen -- nicht alle "
                                         "Formulare wurden gesichert.", 5000)
        return ok

    def set_main_form(self):
        if self.path is None:
            self.statusBar().showMessage("Formular zuerst speichern.", 3000)
            return
        # Das Formular selbst merken, nicht den relativen Pfad: solange kein
        # Projektpfad feststeht, liefert `_rel` nur den Dateinamen. Nach dem
        # Speichern in ein Unterverzeichnis passte der nicht mehr zu den
        # Manifest-Eintraegen und `main` fiel still auf forms[0] zurueck.
        self._main_form = self.active
        self.project.main = self._rel(self.path)
        self._refresh_form_list()
        self.statusBar().showMessage(f"Startformular: {self.path.name}", 3000)

    # -- Projekt (.gbproj) --
    def open_project(self):
        fn, _ = QFileDialog.getOpenFileName(self, "Projekt oeffnen", str(self.project_root),
                                            "GameBasic-Projekt (*.gbproj)")
        if fn:
            self.load_project_file(fn)

    def load_project_file(self, fn: str):
        if not self._confirm_dirty():        # sonst waren alle offenen Formulare weg
            return
        try:
            proj = FormProject.load(fn)
        except Exception as e:  # noqa: BLE001
            QMessageBox.critical(self, "Fehler", f"Projekt nicht ladbar:\n{e}")
            return
        base = Path(fn).parent
        self.project = proj
        self.project_path = Path(fn)
        self.forms = []
        self.active_index = -1
        # Nicht ladbare Formulare merken statt sie stumm zu schlucken: sie
        # bleiben im Manifest (siehe save_project) und der Nutzer erfaehrt davon.
        self.unresolved: list[str] = []
        main_idx = 0
        for rel in proj.forms:
            try:
                doc = FormDoc.load(str(base / rel))
            except Exception:  # noqa: BLE001
                self.unresolved.append(rel)
                continue
            of = self._add_open_form(doc, base / rel, switch=False)
            if rel == proj.main:
                main_idx = len(self.forms) - 1
                self._main_form = of
        if not self.forms:
            self._add_open_form(FormDoc(), switch=False)
        self._switch_to(min(main_idx, len(self.forms) - 1))
        if self.unresolved:
            QMessageBox.warning(
                self, "Formulare fehlen",
                "Diese Formulare konnten nicht geladen werden und bleiben "
                "unveraendert im Projekt:\n\n" + "\n".join(self.unresolved))

    def save_project(self):
        # Erst sicherstellen, dass jede Form einen Pfad hat (+ gespeichert ist).
        start = self.active_index
        for i in range(len(self.forms)):
            if self.forms[i].path is None:
                self._switch_to(i)
                if not self.save_form_as():
                    self._switch_to(start)
                    return
        if self.project_path is None:
            fn, _ = QFileDialog.getSaveFileName(self, "Projekt speichern", str(self.project_root),
                                                "GameBasic-Projekt (*.gbproj)")
            if not fn:
                self._switch_to(start)
                return
            if not fn.endswith(".gbproj"):
                fn += ".gbproj"
            self.project_path = Path(fn)
        # Alle Forms speichern + Manifest aufbauen (relativ zum Projektpfad).
        self.project.forms = []
        main_rel = ""
        for of in self.forms:
            if not self._write(str(of.path), lambda of=of: of.doc.save(str(of.path))):
                self._switch_to(start)
                return
            rel = self._rel(of.path)
            self.project.add(rel)
            if of is getattr(self, "_main_form", None):
                main_rel = rel
        # Beim Laden uebersprungene Formulare bleiben im Manifest -- vorher hat
        # der Neuaufbau aus den OFFENEN Formularen sie dauerhaft geloescht,
        # obwohl die Datei nur kurzzeitig nicht erreichbar war.
        for rel in getattr(self, "unresolved", []):
            self.project.add(rel)
        if main_rel:
            self.project.main = main_rel
        if self.project.main not in self.project.forms:
            self.project.main = self.project.forms[0] if self.project.forms else ""
        if not self._write(str(self.project_path),
                           lambda: self.project.save(str(self.project_path))):
            self._switch_to(start)
            return
        for of in self.forms:                       # erst jetzt als sauber markieren
            of.dirty = False
            of.saved = of.doc.to_dict()
        self._switch_to(min(start, len(self.forms) - 1))
        self.statusBar().showMessage(f"Projekt gespeichert: {self.project_path.name}", 3000)

    def export_gb_code(self):
        """Aktives Formular als eigenstaendiges GameBasic-Programm (explizite
        GUI_*-Konstruktion, kein GUI_LOAD) speichern."""
        default = str(self.path.with_suffix(".gb")) if self.path else str(self.project_root)
        fn, _ = QFileDialog.getSaveFileName(self, "GB-Code exportieren", default,
                                            "GameBasic (*.gb)")
        if not fn:
            return
        if not fn.endswith(".gb"):
            fn += ".gb"
        code = self.canvas.doc.generate_gb_code(screen_title=self.canvas.doc.title)
        Path(fn).write_text(code, encoding="utf-8")
        self.statusBar().showMessage(f"GB-Code exportiert: {Path(fn).name}", 4000)

    def run_form(self):
        gbrt = _find_gbrt(self.project_root)
        if gbrt is None:
            QMessageBox.warning(self, "gbrt fehlt", "Native Runtime nicht gebaut:\n"
                                "python rust/build_runtime.py")
            return
        tmp = Path(tempfile.mkdtemp(prefix="gbform_"))
        form_path = tmp / "form.gbform"
        self.canvas.doc.save(str(form_path))
        runner = self.canvas.doc.generate_runner("form.gbform",
                                                 screen_title=self.canvas.doc.title)
        gb = tmp / "run.gb"
        gb.write_text(runner, encoding="utf-8")
        # Semaphor rund um die Prozess-ERSTELLUNG: schuetzt gegen gleichzeitig
        # startende `gbrt`-Subprozesse aus anderen Editor-Threads (siehe
        # gbrt_locate.gbrt_spawn_semaphore-Kommentar fuer den verifizierten
        # Windows-Crash).
        from .editor_qt.gbrt_locate import gbrt_spawn_semaphore
        with gbrt_spawn_semaphore:
            subprocess.Popen([str(gbrt), "run", str(gb)], cwd=str(tmp))

    def _arm_place(self, item: QListWidgetItem):
        self.canvas.place_kind = item.data(Qt.ItemDataRole.UserRole)
        self.statusBar().showMessage(f"Platzieren: {item.text()} -- auf die Flaeche klicken", 4000)


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance()
    if app is None:
        # Fusion-Style NUR bei frischer QApplication erzwingen -- siehe
        # trackereditor_qt.launch() fuer die volle Begruendung.
        app = QApplication([])
        app.setStyle("Fusion")
    app.setStyleSheet(global_qss())
    win = FormDesigner(project_root)
    if initial_file:
        open_initial(win, Path(initial_file))
    win.show()
    return app.exec()


def open_initial(win: FormDesigner, p: Path) -> bool:
    """Start-Argument oeffnen (`.gbform` oder `.gbproj`). True bei Erfolg.
    Eigene Funktion, damit der Zweig ohne `app.exec()` testbar ist."""
    try:
        if not p.is_file():
            raise OSError("Datei nicht gefunden" if not p.exists()
                          else "ist ein Verzeichnis")
        # `.lower()`: auf Windows ist `Projekt.GBPROJ` dieselbe Datei. Der
        # case-sensitive Vergleich schickte sie in den Formular-Zweig, wo das
        # Manifest als leeres Formular durchging -- das naechste Strg+S hat
        # dann die Projektdatei ueberschrieben.
        if p.suffix.lower() == ".gbproj":
            win.load_project_file(str(p))
        else:
            # leeres Start-Formular durch die geladene Datei ersetzen
            doc = FormDoc.load(str(p))
            of = win.forms[0]
            of.doc = doc; of.path = p; of.saved = doc.to_dict()
            win._switch_to(0)
        return True
    except Exception as e:  # noqa: BLE001
        # Vorher still verschluckt: ein Tippfehler im Dateinamen oeffnete
        # kommentarlos ein leeres Formular, der Nutzer baute es neu.
        QMessageBox.warning(win, "Nicht ladbar", f"{p}\n\n{e}")
        return False
