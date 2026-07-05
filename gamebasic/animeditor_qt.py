"""Animations-FSM-Editor `gbanim` -- PySide6-UI (Unity-Mecanim-Stil).

Ein Knoten-Graph-Editor: **Knoten = States** (an eine Sprite-Animation
gebunden), **Pfeile = Transitions** (mit Bedingungen). Links das Dokument-Panel
(Sprite-Sheet + Parameter), in der Mitte der Graph, rechts der Inspector
(State- bzw. Transition-Eigenschaften). Speichert/laedt `.gbanim` (Runtime-
Format des `animfsm`-Moduls) und startet per F5 eine Live-Vorschau mit `gbrt`
(Parameter-Panel + Sprite, der den aktuellen State spielt).

Datenmodell + Code-Gen liegen Qt-frei in `gamebasic/animeditor/`.
Start: `gbanim [datei.gbanim]` bzw. `gbrun.py --anim`.
"""
from __future__ import annotations

import os
import subprocess
import tempfile
from pathlib import Path

from PySide6.QtCore import Qt, QPoint, QPointF, QRect, QRectF, Signal
from PySide6.QtGui import (
    QAction, QBrush, QColor, QFont, QKeySequence, QPainter, QPen, QPolygonF,
)
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QDockWidget, QScrollArea, QVBoxLayout,
    QHBoxLayout, QFormLayout, QLineEdit, QSpinBox, QDoubleSpinBox, QCheckBox,
    QComboBox, QListWidget, QListWidgetItem, QPushButton, QLabel, QToolBar,
    QFileDialog, QMessageBox, QStackedWidget, QMenu, QInputDialog, QFrame,
)

from .animeditor import (
    ANY_STATE, OPS, PARAM_TYPES, AnimDoc, Condition, History, State, Transition,
    snap,
)
from .animeditor.document import OP_LABELS

try:
    from .editor_qt.theme import global_qss
except Exception:  # pragma: no cover - Theme optional
    def global_qss() -> str:
        return ""


def _find_gbrt():
    root = Path(__file__).resolve().parents[1]
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for v in ("release", "debug"):
        p = root / "rust" / "gb_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


def _editor_qss() -> str:
    """Tailored QSS fuer den gbanim-Editor: Buttons/Toolbuttons mit Rahmen +
    Rundung, farbige Akzente (Cyan/Mint/Magenta aus dem Editor-Theme)."""
    try:
        from .editor_qt.theme import COLORS as C
    except Exception:
        C = {}

    def g(k, d):
        return C.get(k, d)

    bg = g("bg", "#0B1421")
    alt = g("bg_alt", "#121E2C")
    panel = g("bg_panel", "#1A2738")
    panel_hi = g("bg_panel_hi", "#22344A")
    hover = g("bg_hover", "#243549")
    fg = g("fg", "#D5EDF0")
    muted = g("fg_muted", "#7A8E96")
    border = g("border", "#243A4D")
    accent = g("accent", "#2DE0E0")
    accent_text = g("accent_text", "#0B1421")
    accent_soft = g("accent_soft", "#16404C")
    success = g("success", "#2DE89A")
    success_h = g("success_hover", "#5EEDB2")
    danger = g("danger", "#E55A8C")
    return f"""
    QMainWindow, QDockWidget, QScrollArea {{ background: {bg}; }}
    QWidget {{ color: {fg}; }}
    QPushButton {{
        background: {panel_hi}; color: {fg};
        border: 1px solid {border}; border-radius: 7px;
        padding: 5px 12px; font-weight: 600;
    }}
    QPushButton:hover {{ background: {hover}; border: 1px solid {accent}; }}
    QPushButton:pressed {{ background: {accent_soft}; }}
    QToolBar {{ background: {panel}; border: 0; padding: 5px; spacing: 5px; }}
    QToolButton {{
        color: {fg}; background: {panel_hi};
        border: 1px solid {border}; border-radius: 7px;
        padding: 5px 11px; font-weight: 600;
    }}
    QToolButton:hover {{ background: {hover}; border: 1px solid {accent}; }}
    QToolButton:pressed {{ background: {accent_soft}; }}
    QToolButton:checked {{ background: {accent}; color: {accent_text}; border: 1px solid {accent}; }}
    QToolButton#runBtn {{ background: {success}; color: {accent_text}; border: 1px solid {success}; }}
    QToolButton#runBtn:hover {{ background: {success_h}; border: 1px solid {success_h}; }}
    QToolButton#delBtn {{ color: {danger}; border: 1px solid {danger}; }}
    QToolButton#delBtn:hover {{ background: {danger}; color: {accent_text}; }}
    QLineEdit, QSpinBox, QDoubleSpinBox, QComboBox {{
        background: {alt}; color: {fg};
        border: 1px solid {border}; border-radius: 5px; padding: 3px 6px;
        selection-background-color: {accent}; selection-color: {accent_text};
    }}
    QLineEdit:focus, QSpinBox:focus, QDoubleSpinBox:focus, QComboBox:focus {{
        border: 1px solid {accent};
    }}
    QComboBox::drop-down {{ border: 0; width: 18px; }}
    QComboBox QAbstractItemView {{
        background: {alt}; color: {fg};
        selection-background-color: {accent}; selection-color: {accent_text};
        border: 1px solid {border};
    }}
    QListWidget {{ background: {alt}; border: 1px solid {border}; border-radius: 6px; padding: 3px; }}
    QListWidget::item {{ padding: 4px 6px; border-radius: 4px; }}
    QListWidget::item:selected {{ background: {accent_soft}; color: {fg}; }}
    QDockWidget {{ color: {fg}; font-weight: 700; }}
    QDockWidget::title {{
        background: {panel_hi}; padding: 6px 10px;
        border-bottom: 2px solid {accent}; font-weight: 700;
    }}
    QCheckBox {{ color: {fg}; spacing: 6px; }}
    QCheckBox::indicator {{
        width: 16px; height: 16px; border: 1px solid {border};
        border-radius: 4px; background: {alt};
    }}
    QCheckBox::indicator:checked {{ background: {accent}; border: 1px solid {accent}; }}
    QLabel {{ color: {fg}; }}
    QFrame[frameShape="4"] {{ color: {border}; }}
    QStatusBar {{ background: {panel}; color: {muted}; }}
    QMenu {{ background: {panel}; color: {fg}; border: 1px solid {border}; }}
    QMenu::item:selected {{ background: {accent_soft}; }}
    """


# --- Farben / Geometrie -----------------------------------------------------
NODE_W, NODE_H = 138, 50
ANY_W, ANY_H = 120, 36
HIT = 7.0           # Klick-Toleranz auf eine Transition (px)

_BG = QColor(15, 23, 36)            # tiefes Navy (Theme-BG)
_GRID = QColor(28, 42, 58)
_NODE_LOOP = QColor(38, 72, 104)    # loop-States: blau
_NODE_ONCE = QColor(104, 76, 40)    # one-shot-States: bernstein
_NODE_BORDER = QColor(96, 124, 156)
_DEFAULT = QColor(45, 232, 154)     # Mint (Default-State + wait_finished)
_ANY = QColor(150, 100, 190)        # Violett (Any State)
_TEXT = QColor(224, 240, 244)
_SUB = QColor(150, 178, 196)
_ARROW = QColor(94, 200, 210)       # Cyan: bedingungsbasierte Transition
_ARROW_TRIG = QColor(232, 184, 92)  # Amber: trigger-basierte Transition
_ARROW_SEL = QColor(94, 234, 234)   # hell-cyan: ausgewaehlt


class _GraphCanvas(QWidget):
    """Zeichnet den State-Graph und behandelt Auswahl/Verschieben/Verlinken."""

    selection_changed = Signal(object)   # State | Transition | None
    doc_changed = Signal()               # Struktur/Werte geaendert -> dirty + redraw
    before_mutation = Signal()           # vor einer Mutation -> History-Snapshot

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc = AnimDoc()
        self.selected = None             # State | Transition | None
        self.link_mode = False
        self.any_pos = QPoint(40, 40)    # Position der Any-State-Pille (nicht persistiert)
        self.setMinimumSize(1600, 1000)
        self.setMouseTracking(True)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self._drag = None                # ("move", state, dx, dy) | ("any", dx, dy)
        self._link_from = None           # State-Name | "*" waehrend Link-Zieh
        self._link_to_pt = None          # aktueller Mauspunkt beim Verlinken
        self._moved = False

    # ---- Doc-Anbindung ----
    def set_doc(self, doc: AnimDoc):
        self.doc = doc
        self.selected = None
        self.selection_changed.emit(None)
        self.update()

    # ---- Geometrie ----
    def _node_rect(self, name: str) -> QRect:
        if name == ANY_STATE:
            return QRect(self.any_pos.x(), self.any_pos.y(), ANY_W, ANY_H)
        s = self.doc.state_by_name(name)
        if s is None:
            return QRect(0, 0, 0, 0)
        return QRect(s.x, s.y, NODE_W, NODE_H)

    def _node_at(self, pt: QPoint):
        # States zuerst (oben gezeichnet), dann Any-State.
        for s in reversed(self.doc.states):
            if QRect(s.x, s.y, NODE_W, NODE_H).contains(pt):
                return s.name
        if self._node_rect(ANY_STATE).contains(pt):
            return ANY_STATE
        return None

    @staticmethod
    def _edge_point(rect: QRect, toward: QPointF) -> QPointF:
        """Punkt auf dem Rand von `rect` in Richtung `toward` (vom Zentrum aus)."""
        cx, cy = rect.center().x(), rect.center().y()
        dx, dy = toward.x() - cx, toward.y() - cy
        if dx == 0 and dy == 0:
            return QPointF(cx, cy)
        hw, hh = rect.width() / 2.0, rect.height() / 2.0
        sx = hw / abs(dx) if dx != 0 else float("inf")
        sy = hh / abs(dy) if dy != 0 else float("inf")
        s = min(sx, sy)
        return QPointF(cx + dx * s, cy + dy * s)

    def _transition_segment(self, t: Transition):
        """(p1, p2) der gezeichneten Transition -- mit seitlichem Versatz, wenn es
        die Gegenrichtung auch gibt (damit beide Pfeile sichtbar sind)."""
        r_from = self._node_rect(t.from_state)
        r_to = self._node_rect(t.to_state)
        if r_from.isEmpty() or r_to.isEmpty():
            return None
        c_from = QPointF(r_from.center())
        c_to = QPointF(r_to.center())
        p1 = self._edge_point(r_from, c_to)
        p2 = self._edge_point(r_to, c_from)
        # Versatz bei reziproker Transition
        has_reverse = any(
            x is not t and x.from_state == t.to_state and x.to_state == t.from_state
            for x in self.doc.transitions
        )
        if has_reverse:
            dx, dy = p2.x() - p1.x(), p2.y() - p1.y()
            ln = max(1.0, (dx * dx + dy * dy) ** 0.5)
            ox, oy = -dy / ln * 9.0, dx / ln * 9.0
            p1 = QPointF(p1.x() + ox, p1.y() + oy)
            p2 = QPointF(p2.x() + ox, p2.y() + oy)
        return p1, p2

    def _transition_at(self, pt: QPoint):
        p = QPointF(pt)
        for t in self.doc.transitions:
            seg = self._transition_segment(t)
            if seg is None:
                continue
            if _dist_point_seg(p, seg[0], seg[1]) <= HIT:
                return t
        return None

    # ---- Maus ----
    def mousePressEvent(self, ev):
        pt = ev.position().toPoint()
        if ev.button() == Qt.MouseButton.RightButton:
            self._context_menu(pt, ev.globalPosition().toPoint())
            return
        if ev.button() != Qt.MouseButton.LeftButton:
            return
        name = self._node_at(pt)
        if self.link_mode and name is not None:
            self._link_from = name
            self._link_to_pt = pt
            return
        if name is not None:
            r = self._node_rect(name)
            if name == ANY_STATE:
                self._select(None)
                self._drag = ("any", pt.x() - r.x(), pt.y() - r.y())
            else:
                self._select(self.doc.state_by_name(name))
                self._drag = ("move", name, pt.x() - r.x(), pt.y() - r.y())
            self._moved = False
            return
        # Transition?
        t = self._transition_at(pt)
        self._select(t)

    def mouseMoveEvent(self, ev):
        pt = ev.position().toPoint()
        if self._link_from is not None:
            self._link_to_pt = pt
            self.update()
            return
        if self._drag is None:
            return
        if not self._moved:
            self.before_mutation.emit()
            self._moved = True
        kind = self._drag[0]
        if kind == "any":
            self.any_pos = QPoint(snap(pt.x() - self._drag[1]), snap(pt.y() - self._drag[2]))
        else:
            s = self.doc.state_by_name(self._drag[1])
            if s is not None:
                s.x = max(0, snap(pt.x() - self._drag[2]))
                s.y = max(0, snap(pt.y() - self._drag[3]))
        self.update()

    def mouseReleaseEvent(self, ev):
        if self._link_from is not None:
            target = self._node_at(ev.position().toPoint())
            src = self._link_from
            self._link_from = None
            self._link_to_pt = None
            if target is not None and target != ANY_STATE:
                self.before_mutation.emit()
                t = self.doc.add_transition(src, target)
                if t is not None:
                    self._select(t)
                    self.doc_changed.emit()
            self.update()
            return
        if self._drag is not None and self._moved:
            self.doc_changed.emit()
        self._drag = None
        self._moved = False

    def mouseDoubleClickEvent(self, ev):
        pt = ev.position().toPoint()
        if self._node_at(pt) is not None or self._transition_at(pt) is not None:
            return
        self.before_mutation.emit()
        s = self.doc.add_state(snap(pt.x()), snap(pt.y()))
        self._select(s)
        self.doc_changed.emit()

    def keyPressEvent(self, ev):
        if ev.key() in (Qt.Key.Key_Delete, Qt.Key.Key_Backspace):
            self.delete_selected()
        else:
            super().keyPressEvent(ev)

    # ---- Aktionen ----
    def _select(self, obj):
        self.selected = obj
        self.selection_changed.emit(obj)
        self.update()

    def delete_selected(self):
        if isinstance(self.selected, State):
            self.before_mutation.emit()
            self.doc.remove_state(self.selected.name)
            self._select(None)
            self.doc_changed.emit()
        elif isinstance(self.selected, Transition):
            self.before_mutation.emit()
            self.doc.remove_transition(self.selected)
            self._select(None)
            self.doc_changed.emit()

    def _context_menu(self, pt: QPoint, global_pt: QPoint):
        name = self._node_at(pt)
        menu = QMenu(self)
        if name is not None and name != ANY_STATE:
            s = self.doc.state_by_name(name)
            self._select(s)
            menu.addAction("Als Default-State", lambda: self._set_default(name))
            menu.addAction("Umbenennen ...", lambda: self._rename_dialog(name))
            menu.addSeparator()
            menu.addAction("State loeschen", self.delete_selected)
        else:
            t = self._transition_at(pt)
            if t is not None:
                self._select(t)
                menu.addAction("Transition loeschen", self.delete_selected)
            else:
                menu.addAction("State hinzufuegen",
                               lambda: self._add_state_at(pt))
        if not menu.isEmpty():
            menu.exec(global_pt)

    def _add_state_at(self, pt: QPoint):
        self.before_mutation.emit()
        s = self.doc.add_state(snap(pt.x()), snap(pt.y()))
        self._select(s)
        self.doc_changed.emit()

    def _set_default(self, name: str):
        self.before_mutation.emit()
        self.doc.default_state = name
        self.doc_changed.emit()
        self.update()

    def _rename_dialog(self, name: str):
        new, ok = QInputDialog.getText(self, "State umbenennen", "Neuer Name:", text=name)
        if ok and new.strip():
            self.before_mutation.emit()
            if self.doc.rename_state(name, new.strip()):
                self.doc_changed.emit()
                self.selection_changed.emit(self.doc.state_by_name(new.strip()))
            self.update()

    # ---- Zeichnen ----
    def paintEvent(self, _ev):
        qp = QPainter(self)
        qp.setRenderHint(QPainter.RenderHint.Antialiasing, True)
        qp.fillRect(self.rect(), _BG)
        # Raster
        qp.setPen(QPen(_GRID, 1))
        step = 32
        for x in range(0, self.width(), step):
            qp.drawLine(x, 0, x, self.height())
        for y in range(0, self.height(), step):
            qp.drawLine(0, y, self.width(), y)

        # Transitions
        for t in self.doc.transitions:
            self._draw_transition(qp, t)
        # Link-Vorschau
        if self._link_from is not None and self._link_to_pt is not None:
            r = self._node_rect(self._link_from)
            qp.setPen(QPen(_ARROW_SEL, 2, Qt.PenStyle.DashLine))
            qp.drawLine(r.center(), self._link_to_pt)

        # Default-Eingangspfeil
        default = self.doc.effective_default()
        if default:
            r = self._node_rect(default)
            qp.setPen(QPen(_DEFAULT, 2))
            qp.setBrush(QBrush(_DEFAULT))
            start = QPointF(r.left() - 34, r.center().y())
            end = QPointF(r.left() - 2, r.center().y())
            qp.drawLine(start, end)
            _arrow_head(qp, start, end, _DEFAULT)

        # Any-State-Pille
        self._draw_any(qp)
        # State-Knoten
        for s in self.doc.states:
            self._draw_state(qp, s)

    def _draw_any(self, qp: QPainter):
        r = self._node_rect(ANY_STATE)
        sel = self.selected is None and False
        qp.setPen(QPen(_NODE_BORDER, 1.5))
        qp.setBrush(QBrush(_ANY))
        qp.drawRoundedRect(r, 16, 16)
        qp.setPen(_TEXT)
        f = qp.font(); f.setBold(True); qp.setFont(f)
        qp.drawText(r, Qt.AlignmentFlag.AlignCenter, "Any State")

    def _draw_state(self, qp: QPainter, s: State):
        r = QRect(s.x, s.y, NODE_W, NODE_H)
        is_sel = self.selected is s
        is_def = (s.name == self.doc.effective_default())
        base = _NODE_ONCE if not s.loop else _NODE_LOOP
        if is_sel:
            base = base.lighter(135)
        # weicher Vertikal-Gradient fuer etwas Tiefe
        from PySide6.QtGui import QLinearGradient
        grad = QLinearGradient(r.topLeft(), r.bottomLeft())
        grad.setColorAt(0.0, base.lighter(118))
        grad.setColorAt(1.0, base.darker(112))
        qp.setBrush(QBrush(grad))
        border_col = _DEFAULT if is_def else (_ARROW_SEL if is_sel else _NODE_BORDER)
        qp.setPen(QPen(border_col, 2.6 if (is_def or is_sel) else 1.5))
        qp.drawRoundedRect(r, 9, 9)
        # Farbiger Typ-Streifen links (loop = mint, one-shot = amber)
        stripe = _DEFAULT if s.loop else _ARROW_TRIG
        qp.setPen(Qt.PenStyle.NoPen)
        qp.setBrush(QBrush(stripe))
        qp.drawRoundedRect(QRect(s.x + 3, s.y + 6, 5, NODE_H - 12), 2, 2)
        # Name
        qp.setPen(_TEXT)
        f = qp.font(); f.setBold(True); f.setPointSize(10); qp.setFont(f)
        qp.drawText(QRect(s.x + 16, s.y + 4, NODE_W - 22, 20),
                    Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter, s.name)
        # Default-Badge
        if is_def:
            qp.setPen(_DEFAULT)
            f.setPointSize(7); qp.setFont(f)
            qp.drawText(QRect(s.x + 8, s.y + 3, NODE_W - 14, 12),
                        Qt.AlignmentFlag.AlignRight | Qt.AlignmentFlag.AlignVCenter, "DEFAULT")
        # Untertitel: anim + loop/once + frames
        qp.setPen(_SUB)
        f.setBold(False); f.setPointSize(8); qp.setFont(f)
        mode = "loop" if s.loop else "once"
        sub = f"{s.anim_name}  [{s.first}-{s.last}]  {mode}"
        qp.drawText(QRect(s.x + 16, s.y + 24, NODE_W - 22, 18),
                    Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter, sub)

    def _draw_transition(self, qp: QPainter, t: Transition):
        seg = self._transition_segment(t)
        if seg is None:
            return
        p1, p2 = seg
        is_sel = self.selected is t
        if is_sel:
            col = _ARROW_SEL
        elif any(c.op == "trigger" for c in t.conditions):
            col = _ARROW_TRIG                       # Amber: trigger-basiert
        elif t.wait_finished and not t.conditions:
            col = _DEFAULT                          # Mint: rein "wenn fertig"
        else:
            col = _ARROW                            # Cyan: bedingungsbasiert
        qp.setPen(QPen(col, 2.6 if is_sel else 2.0))
        qp.drawLine(p1, p2)
        _arrow_head(qp, p1, p2, col)
        # Label
        label = _transition_label(t)
        if label:
            mid = QPointF((p1.x() + p2.x()) / 2, (p1.y() + p2.y()) / 2)
            qp.setPen(_SUB if not is_sel else _TEXT)
            f = qp.font(); f.setBold(False); f.setPointSize(8); qp.setFont(f)
            qp.drawText(QRectF(mid.x() - 60, mid.y() - 18, 120, 16),
                        Qt.AlignmentFlag.AlignCenter, label)


# --- Geometrie-Helfer -------------------------------------------------------
def _dist_point_seg(p: QPointF, a: QPointF, b: QPointF) -> float:
    ax, ay, bx, by = a.x(), a.y(), b.x(), b.y()
    dx, dy = bx - ax, by - ay
    if dx == 0 and dy == 0:
        return ((p.x() - ax) ** 2 + (p.y() - ay) ** 2) ** 0.5
    tt = ((p.x() - ax) * dx + (p.y() - ay) * dy) / (dx * dx + dy * dy)
    tt = max(0.0, min(1.0, tt))
    cx, cy = ax + tt * dx, ay + tt * dy
    return ((p.x() - cx) ** 2 + (p.y() - cy) ** 2) ** 0.5


def _arrow_head(qp: QPainter, a: QPointF, b: QPointF, col: QColor):
    import math
    ang = math.atan2(b.y() - a.y(), b.x() - a.x())
    size = 10.0
    p1 = QPointF(b.x() - size * math.cos(ang - 0.4), b.y() - size * math.sin(ang - 0.4))
    p2 = QPointF(b.x() - size * math.cos(ang + 0.4), b.y() - size * math.sin(ang + 0.4))
    qp.setBrush(QBrush(col))
    qp.setPen(QPen(col, 1))
    qp.drawPolygon(QPolygonF([b, p1, p2]))


def _transition_label(t: Transition) -> str:
    parts = []
    if t.wait_finished:
        parts.append("fertig")
    for c in t.conditions:
        if c.op in ("is_true", "is_false", "trigger"):
            parts.append(f"{c.param} {OP_LABELS.get(c.op, c.op)}")
        else:
            v = int(c.value) if float(c.value).is_integer() else c.value
            parts.append(f"{c.param}{OP_LABELS.get(c.op, c.op)}{v}")
    if not parts:
        return ""
    if len(parts) == 1:
        return parts[0]
    return parts[0] + f"  +{len(parts) - 1}"


# ============================================================ Inspector-Seiten
class _StateInspector(QWidget):
    changed = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self._loading = False
        self._state: State | None = None
        self._doc: AnimDoc | None = None
        form = QFormLayout(self)
        self.name = QLineEdit()
        self.anim = QLineEdit()
        self.loop = QCheckBox("Animation loopt (sonst one-shot)")
        self.first = QSpinBox(); self.first.setRange(0, 9999)
        self.last = QSpinBox(); self.last.setRange(0, 9999)
        self.fps = QDoubleSpinBox(); self.fps.setRange(0.0, 240.0); self.fps.setDecimals(1)
        self.is_default = QCheckBox("Default-State")
        form.addRow("Name", self.name)
        form.addRow("Animation", self.anim)
        form.addRow("", self.loop)
        form.addRow("Frame von", self.first)
        form.addRow("Frame bis", self.last)
        form.addRow("FPS", self.fps)
        form.addRow("", self.is_default)
        self.name.editingFinished.connect(self._apply_name)
        for w in (self.anim,):
            w.editingFinished.connect(self._apply)
        self.loop.toggled.connect(self._apply)
        for w in (self.first, self.last):
            w.valueChanged.connect(self._apply)
        self.fps.valueChanged.connect(self._apply)
        self.is_default.toggled.connect(self._apply_default)

    def set_state(self, doc: AnimDoc, s: State | None):
        self._doc = doc
        self._state = s
        if s is None:
            return
        self._loading = True
        self.name.setText(s.name)
        self.anim.setText(s.anim)
        self.loop.setChecked(s.loop)
        self.first.setValue(s.first)
        self.last.setValue(s.last)
        self.fps.setValue(s.fps)
        self.is_default.setChecked(doc.effective_default() == s.name)
        self._loading = False

    def _apply(self):
        if self._loading or self._state is None:
            return
        s = self._state
        s.anim = self.anim.text().strip()
        s.loop = self.loop.isChecked()
        s.first = self.first.value()
        s.last = self.last.value()
        s.fps = self.fps.value()
        self.changed.emit()

    def _apply_name(self):
        if self._loading or self._state is None or self._doc is None:
            return
        new = self.name.text().strip()
        if new and new != self._state.name:
            if not self._doc.rename_state(self._state.name, new):
                self.name.setText(self._state.name)   # abgelehnt -> zuruecksetzen
        self.changed.emit()

    def _apply_default(self):
        if self._loading or self._state is None or self._doc is None:
            return
        if self.is_default.isChecked():
            self._doc.default_state = self._state.name
            self.changed.emit()


class _TransitionInspector(QWidget):
    changed = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self._loading = False
        self._t: Transition | None = None
        self._doc: AnimDoc | None = None
        lay = QVBoxLayout(self)
        self.head = QLabel("—")
        self.head.setWordWrap(True)
        f = self.head.font(); f.setBold(True); self.head.setFont(f)
        lay.addWidget(self.head)
        self.wait = QCheckBox("Erst wechseln, wenn Animation fertig (wait_finished)")
        self.wait.toggled.connect(self._apply_wait)
        lay.addWidget(self.wait)
        lay.addWidget(_hline())
        lay.addWidget(QLabel("Bedingungen (UND-verknuepft):"))
        self.cond_box = QVBoxLayout()
        cont = QWidget(); cont.setLayout(self.cond_box)
        lay.addWidget(cont)
        add = QPushButton("+ Bedingung")
        add.clicked.connect(self._add_cond)
        lay.addWidget(add)
        lay.addStretch(1)

    def set_transition(self, doc: AnimDoc, t: Transition | None):
        self._doc = doc
        self._t = t
        if t is None:
            return
        self._loading = True
        src = "Any State" if t.from_state == ANY_STATE else t.from_state
        self.head.setText(f"{src}  →  {t.to_state}")
        self.wait.setChecked(t.wait_finished)
        self._rebuild_conds()
        self._loading = False

    def _rebuild_conds(self):
        while self.cond_box.count():
            item = self.cond_box.takeAt(0)
            w = item.widget()
            if w is not None:
                w.deleteLater()
        if self._t is None or self._doc is None:
            return
        pnames = [p.name for p in self._doc.params]
        for c in list(self._t.conditions):
            self.cond_box.addWidget(_CondRow(self._doc, c, pnames, self))

    def _add_cond(self):
        if self._t is None or self._doc is None:
            return
        if not self._doc.params:
            QMessageBox.information(self, "Kein Parameter",
                                   "Erst links einen Parameter anlegen.")
            return
        self.changed.emit()                     # History-Snapshot
        p = self._doc.params[0]
        op = "trigger" if p.ptype == "trigger" else ("is_true" if p.ptype == "bool" else "gt")
        self._t.conditions.append(Condition(p.name, op, 0.0))
        self._rebuild_conds()
        self.changed.emit()

    def remove_cond(self, c: Condition):
        if self._t is None:
            return
        self.changed.emit()
        self._t.conditions = [x for x in self._t.conditions if x is not c]
        self._rebuild_conds()
        self.changed.emit()

    def _apply_wait(self):
        if self._loading or self._t is None:
            return
        self._t.wait_finished = self.wait.isChecked()
        self.changed.emit()


class _CondRow(QWidget):
    def __init__(self, doc: AnimDoc, cond: Condition, pnames: list[str],
                 owner: _TransitionInspector):
        super().__init__(owner)
        self._doc = doc
        self._c = cond
        self._owner = owner
        row = QHBoxLayout(self)
        row.setContentsMargins(0, 0, 0, 0)
        self.param = QComboBox(); self.param.addItems(pnames)
        if cond.param in pnames:
            self.param.setCurrentText(cond.param)
        self.op = QComboBox()
        for o in OPS:
            self.op.addItem(OP_LABELS.get(o, o), o)
        idx = self.op.findData(cond.op)
        if idx >= 0:
            self.op.setCurrentIndex(idx)
        self.value = QDoubleSpinBox(); self.value.setRange(-1e6, 1e6); self.value.setDecimals(2)
        self.value.setValue(cond.value)
        rm = QPushButton("✕"); rm.setFixedWidth(26)
        row.addWidget(self.param, 2)
        row.addWidget(self.op, 2)
        row.addWidget(self.value, 2)
        row.addWidget(rm)
        self.param.currentTextChanged.connect(self._apply)
        self.op.currentIndexChanged.connect(self._apply)
        self.value.valueChanged.connect(self._apply)
        rm.clicked.connect(lambda: owner.remove_cond(self._c))
        self._sync_value_enabled()

    def _sync_value_enabled(self):
        op = self.op.currentData()
        self.value.setEnabled(op in ("gt", "lt", "ge", "le", "eq", "ne"))

    def _apply(self):
        self._c.param = self.param.currentText()
        self._c.op = self.op.currentData()
        self._c.value = self.value.value()
        self._sync_value_enabled()
        self._owner.changed.emit()


def _hline() -> QFrame:
    ln = QFrame(); ln.setFrameShape(QFrame.Shape.HLine); ln.setFrameShadow(QFrame.Shadow.Sunken)
    return ln


# ============================================================ Parameter-Panel
class _ParamPanel(QWidget):
    changed = Signal()
    before_mutation = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self._doc: AnimDoc | None = None
        self._loading = False
        lay = QVBoxLayout(self)
        lay.addWidget(QLabel("Parameter"))
        self.list = QListWidget()
        self.list.currentRowChanged.connect(self._on_select)
        lay.addWidget(self.list, 1)
        btns = QHBoxLayout()
        add = QPushButton("+"); rm = QPushButton("−")
        add.clicked.connect(self._add); rm.clicked.connect(self._remove)
        btns.addWidget(add); btns.addWidget(rm)
        lay.addLayout(btns)
        # Editor
        form = QFormLayout()
        self.name = QLineEdit()
        self.ptype = QComboBox(); self.ptype.addItems(PARAM_TYPES)
        self.default = QDoubleSpinBox(); self.default.setRange(-1e6, 1e6); self.default.setDecimals(2)
        form.addRow("Name", self.name)
        form.addRow("Typ", self.ptype)
        form.addRow("Default", self.default)
        lay.addLayout(form)
        self.name.editingFinished.connect(self._apply_name)
        self.ptype.currentTextChanged.connect(self._apply)
        self.default.valueChanged.connect(self._apply)

    def set_doc(self, doc: AnimDoc):
        self._doc = doc
        self._reload()

    def _reload(self):
        self._loading = True
        self.list.clear()
        if self._doc is not None:
            for p in self._doc.params:
                self.list.addItem(f"{p.name}  ({p.ptype})")
        self._loading = False
        self._on_select(self.list.currentRow())

    def _cur(self):
        i = self.list.currentRow()
        if self._doc is None or i < 0 or i >= len(self._doc.params):
            return None
        return self._doc.params[i]

    def _on_select(self, _row):
        p = self._cur()
        en = p is not None
        self.name.setEnabled(en); self.ptype.setEnabled(en); self.default.setEnabled(en)
        if p is None:
            return
        self._loading = True
        self.name.setText(p.name)
        self.ptype.setCurrentText(p.ptype)
        self.default.setValue(p.default)
        self.default.setEnabled(p.ptype != "trigger")
        self._loading = False

    def _add(self):
        if self._doc is None:
            return
        self.before_mutation.emit()
        self._doc.add_param("param", "float")
        self._reload()
        self.list.setCurrentRow(self.list.count() - 1)
        self.changed.emit()

    def _remove(self):
        p = self._cur()
        if p is None:
            return
        self.before_mutation.emit()
        self._doc.remove_param(p.name)
        self._reload()
        self.changed.emit()

    def _apply_name(self):
        if self._loading:
            return
        p = self._cur()
        if p is None:
            return
        new = self.name.text().strip()
        if new and new != p.name:
            self.before_mutation.emit()
            if not self._doc.rename_param(p.name, new):
                self.name.setText(p.name)
                return
            self._reload_keep_row()
            self.changed.emit()

    def _apply(self):
        if self._loading:
            return
        p = self._cur()
        if p is None:
            return
        self.before_mutation.emit()
        p.ptype = self.ptype.currentText()
        p.default = self.default.value()
        self.default.setEnabled(p.ptype != "trigger")
        self._reload_keep_row()
        self.changed.emit()

    def _reload_keep_row(self):
        row = self.list.currentRow()
        self._reload()
        if 0 <= row < self.list.count():
            self.list.setCurrentRow(row)


# ============================================================ Dokument-Panel
class _DocPanel(QWidget):
    changed = Signal()
    before_mutation = Signal()

    def __init__(self, project_root: Path, parent=None):
        super().__init__(parent)
        self._doc: AnimDoc | None = None
        self._loading = False
        self._root = project_root
        lay = QVBoxLayout(self)
        form = QFormLayout()
        self.sheet = QLineEdit()
        browse = QPushButton("…"); browse.setFixedWidth(30)
        browse.clicked.connect(self._browse_sheet)
        srow = QHBoxLayout(); srow.addWidget(self.sheet); srow.addWidget(browse)
        sw = QWidget(); sw.setLayout(srow)
        self.fw = QSpinBox(); self.fw.setRange(1, 4096); self.fw.setValue(16)
        self.fh = QSpinBox(); self.fh.setRange(1, 4096); self.fh.setValue(16)
        self.scale = QDoubleSpinBox(); self.scale.setRange(0.1, 32.0); self.scale.setValue(4.0)
        form.addRow("Sprite-Sheet", sw)
        form.addRow("Frame-Breite", self.fw)
        form.addRow("Frame-Hoehe", self.fh)
        form.addRow("Vorschau-Skala", self.scale)
        lay.addLayout(form)
        self.sheet.editingFinished.connect(self._apply)
        for w in (self.fw, self.fh):
            w.valueChanged.connect(self._apply)
        self.scale.valueChanged.connect(self._apply)

    def set_doc(self, doc: AnimDoc):
        self._doc = doc
        self._loading = True
        self.sheet.setText(doc.sheet)
        self.fw.setValue(doc.frame_w)
        self.fh.setValue(doc.frame_h)
        self.scale.setValue(doc.scale)
        self._loading = False

    def _browse_sheet(self):
        fn, _ = QFileDialog.getOpenFileName(self, "Sprite-Sheet waehlen",
                                            str(self._root), "Bilder (*.png *.bmp *.jpg)")
        if fn:
            # relativ zum Projekt, wenn moeglich
            try:
                rel = os.path.relpath(fn, str(self._root))
                fn = rel.replace("\\", "/")
            except ValueError:
                pass
            self.sheet.setText(fn)
            self._apply()

    def _apply(self):
        if self._loading or self._doc is None:
            return
        self.before_mutation.emit()
        self._doc.sheet = self.sheet.text().strip()
        self._doc.frame_w = self.fw.value()
        self._doc.frame_h = self.fh.value()
        self._doc.scale = self.scale.value()
        self.changed.emit()


# ============================================================ Hauptfenster
class AnimEditor(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = project_root
        self.path: Path | None = None
        self.dirty = False
        self.history = History()

        self.canvas = _GraphCanvas()
        scroll = QScrollArea(); scroll.setWidget(self.canvas); scroll.setWidgetResizable(False)
        self.setCentralWidget(scroll)

        # Links: Dokument + Parameter
        self.doc_panel = _DocPanel(project_root)
        self.param_panel = _ParamPanel()
        left = QWidget(); ll = QVBoxLayout(left)
        ll.addWidget(self.doc_panel)
        ll.addWidget(_hline())
        ll.addWidget(self.param_panel, 1)
        left.setMinimumWidth(230)
        self.param_panel.list.setMaximumHeight(180)
        ldock = QDockWidget("Dokument", self)
        ldock.setWidget(left)
        self.addDockWidget(Qt.DockWidgetArea.LeftDockWidgetArea, ldock)
        self._ldock = ldock

        # Rechts: Inspector (gestapelt)
        self.state_insp = _StateInspector()
        self.trans_insp = _TransitionInspector()
        self.empty_insp = QLabel("Knoten doppelklicken = State.\n"
                                 "Link-Modus + ziehen = Transition.\n"
                                 "Klick auf Pfeil = Transition bearbeiten.")
        self.empty_insp.setAlignment(Qt.AlignmentFlag.AlignTop)
        self.empty_insp.setWordWrap(True)
        self.empty_insp.setContentsMargins(10, 10, 10, 10)
        self.insp_stack = QStackedWidget()
        self.insp_stack.addWidget(self.empty_insp)   # 0
        self.insp_stack.addWidget(self.state_insp)   # 1
        self.insp_stack.addWidget(self.trans_insp)   # 2
        self.insp_stack.setMinimumWidth(250)
        rdock = QDockWidget("Inspector", self)
        rdock.setWidget(self.insp_stack)
        self.addDockWidget(Qt.DockWidgetArea.RightDockWidgetArea, rdock)
        self._rdock = rdock

        self._build_toolbar()
        self._wire()
        self._set_doc(AnimDoc())
        self.setStyleSheet(_editor_qss())
        self.resize(1440, 900)
        # Docks schmal halten, damit der Graph den Hauptplatz bekommt.
        self.resizeDocks([self._ldock, self._rdock], [250, 290],
                         Qt.Orientation.Horizontal)
        self._update_title()

    # ---- Verdrahtung ----
    def _wire(self):
        self.canvas.selection_changed.connect(self._on_selection)
        self.canvas.doc_changed.connect(self._on_doc_changed)
        self.canvas.before_mutation.connect(self._snapshot)
        for panel in (self.doc_panel, self.param_panel):
            panel.changed.connect(self._on_doc_changed)
            panel.before_mutation.connect(self._snapshot)
        self.state_insp.changed.connect(self._on_inspector_changed)
        self.trans_insp.changed.connect(self._on_inspector_changed)

    def _build_toolbar(self):
        tb = QToolBar("Werkzeuge"); tb.setMovable(False)
        from PySide6.QtCore import QSize
        tb.setIconSize(QSize(18, 18))
        self.addToolBar(tb)

        def act(text, slot, shortcut=None, obj=None):
            a = QAction(text, self)
            if shortcut:
                a.setShortcut(QKeySequence(shortcut))
            a.triggered.connect(slot)
            tb.addAction(a)
            if obj:
                w = tb.widgetForAction(a)
                if w is not None:
                    w.setObjectName(obj)
            return a

        act("📄  Neu", self.new_doc, "Ctrl+N")
        act("📂  Öffnen", self.open_doc, "Ctrl+O")
        act("💾  Speichern", self.save_doc, "Ctrl+S")
        act("Speichern unter", self.save_doc_as, "Ctrl+Shift+S")
        tb.addSeparator()
        act("➕  State", self.add_state_center)
        self.link_act = QAction("🔗  Link-Modus", self); self.link_act.setCheckable(True)
        self.link_act.setShortcut(QKeySequence("L"))
        self.link_act.toggled.connect(self._toggle_link)
        self.link_act.setToolTip("Transition ziehen: von einem Knoten auf einen anderen (Taste L)")
        tb.addAction(self.link_act)
        act("🗑  Löschen", self.canvas.delete_selected, "Del", obj="delBtn")
        tb.addSeparator()
        act("↶  Undo", self.undo, "Ctrl+Z")
        act("↷  Redo", self.redo, "Ctrl+Y")
        tb.addSeparator()
        act("▶  Vorschau (F5)", self.run_preview, "F5", obj="runBtn")

    # ---- Doc-Lifecycle ----
    def _set_doc(self, doc: AnimDoc):
        self.canvas.set_doc(doc)
        self.doc_panel.set_doc(doc)
        self.param_panel.set_doc(doc)
        self.history.clear()
        self.insp_stack.setCurrentIndex(0)

    def _snapshot(self):
        self.history.push(self.canvas.doc.to_dict())

    def _restore(self, snap_dict: dict):
        doc = AnimDoc.from_dict(snap_dict)
        self.canvas.doc = doc
        self.canvas.selected = None
        self.doc_panel.set_doc(doc)
        self.param_panel.set_doc(doc)
        self.insp_stack.setCurrentIndex(0)
        self.canvas.update()
        self._mark_dirty()

    def undo(self):
        if self.history.can_undo:
            self._restore(self.history.undo(self.canvas.doc.to_dict()))

    def redo(self):
        if self.history.can_redo:
            self._restore(self.history.redo(self.canvas.doc.to_dict()))

    # ---- Signale ----
    def _on_selection(self, obj):
        if isinstance(obj, State):
            self.state_insp.set_state(self.canvas.doc, obj)
            self.insp_stack.setCurrentIndex(1)
        elif isinstance(obj, Transition):
            self.trans_insp.set_transition(self.canvas.doc, obj)
            self.insp_stack.setCurrentIndex(2)
        else:
            self.insp_stack.setCurrentIndex(0)

    def _on_doc_changed(self):
        self._mark_dirty()
        self.param_panel._reload_keep_row()
        self.canvas.update()

    def _on_inspector_changed(self):
        self._snapshot()
        self._mark_dirty()
        self.canvas.update()

    def _toggle_link(self, on: bool):
        self.canvas.link_mode = on
        self.statusBar().showMessage(
            "Link-Modus: von einem Knoten auf einen anderen ziehen = Transition." if on
            else "", 4000)

    def add_state_center(self):
        self._snapshot()
        s = self.canvas.doc.add_state(snap(200), snap(160))
        self.canvas._select(s)
        self._on_doc_changed()

    # ---- Dirty/Titel ----
    def _mark_dirty(self):
        self.dirty = True
        self._update_title()

    def _update_title(self):
        name = self.path.name if self.path else "unbenannt.gbanim"
        star = "*" if self.dirty else ""
        self.setWindowTitle(f"gbanim — {name}{star}")

    # ---- Datei ----
    def new_doc(self):
        self.path = None
        self.dirty = False
        self._set_doc(AnimDoc())
        self._update_title()

    def open_doc(self):
        fn, _ = QFileDialog.getOpenFileName(self, "Anim-FSM oeffnen",
                                            str(self.project_root), "Anim-FSM (*.gbanim)")
        if not fn:
            return
        try:
            doc = AnimDoc.load(fn)
        except Exception as e:  # pragma: no cover - UI-Fehlerpfad
            QMessageBox.warning(self, "Fehler", f"Konnte nicht laden:\n{e}")
            return
        self.path = Path(fn)
        self.dirty = False
        self._set_doc(doc)
        self._update_title()

    def save_doc(self) -> bool:
        if self.path is None:
            return self.save_doc_as()
        self.canvas.doc.save(str(self.path))
        self.dirty = False
        self._update_title()
        self.statusBar().showMessage(f"Gespeichert: {self.path.name}", 3000)
        return True

    def save_doc_as(self) -> bool:
        default = str(self.path) if self.path else str(self.project_root / "anim.gbanim")
        fn, _ = QFileDialog.getSaveFileName(self, "Speichern unter", default,
                                            "Anim-FSM (*.gbanim)")
        if not fn:
            return False
        if not fn.endswith(".gbanim"):
            fn += ".gbanim"
        self.path = Path(fn)
        return self.save_doc()

    def run_preview(self):
        if not self.canvas.doc.states:
            QMessageBox.information(self, "Leer", "Erst mindestens einen State anlegen.")
            return
        gbrt = _find_gbrt()
        if gbrt is None:
            QMessageBox.warning(self, "gbrt fehlt",
                                "Native Runtime nicht gebaut:\npython rust/build_runtime.py")
            return
        tmp = Path(tempfile.mkdtemp(prefix="gbanim_"))
        # Sheet-Pfad relativ zum Projekt -> in den Temp-Ordner spiegeln, damit
        # der Runner (chdir ins Temp) das Bild findet.
        doc = AnimDoc.from_dict(self.canvas.doc.to_dict())
        if doc.sheet:
            src_img = (self.project_root / doc.sheet)
            if src_img.exists():
                dst = tmp / Path(doc.sheet).name
                try:
                    dst.write_bytes(src_img.read_bytes())
                    doc.sheet = dst.name
                except OSError:
                    doc.sheet = ""
            else:
                doc.sheet = ""
        doc.save(str(tmp / "anim.gbanim"))
        runner = doc.generate_runner("anim.gbanim",
                                     title=(self.path.stem if self.path else "gbanim"))
        (tmp / "run.gb").write_text(runner, encoding="utf-8")
        subprocess.Popen([str(gbrt), "run", str(tmp / "run.gb")], cwd=str(tmp))


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance()
    if app is None:
        # Fusion-Style NUR bei frischer QApplication erzwingen -- siehe
        # trackereditor_qt.launch() fuer die volle Begruendung.
        app = QApplication([])
        app.setStyle("Fusion")
    app.setStyleSheet(global_qss())
    win = AnimEditor(project_root)
    # Ohne Datei: die mitgelieferte Beispiel-FSM laden, damit man sofort sieht,
    # wie ein fertiger Graph aussieht (Strg+N für ein leeres Projekt).
    if initial_file is None:
        demo = Path(project_root) / "examples" / "anim_demo.gbanim"
        if demo.exists():
            initial_file = demo
    if initial_file and Path(initial_file).exists():
        try:
            win.path = Path(initial_file)
            win.dirty = False
            win._set_doc(AnimDoc.load(str(initial_file)))
            win._update_title()
        except Exception:
            pass
    win.show()
    return app.exec()
