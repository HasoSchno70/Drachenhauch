"""Partikel-Editor fuer GameBasic (`gbparticles` / `gbrun.py --particles`).

Standalone-Qt-Tool: alle Parameter des `particles`-Moduls live per Slider/
Spinbox tunen, mit Echtzeit-Vorschau, und das Ergebnis als GB-Code-Snippet
exportieren. Die Vorschau treibt eine echte `_ParticleSystem`-Instanz (das
gleiche Simulationsmodell wie die Engine) per QTimer und rendert sie mit
QPainter -- so entspricht die Vorschau exakt dem spaeteren Verhalten.
"""
from __future__ import annotations

from pathlib import Path

import numpy as np
from PySide6.QtCore import Qt, QTimer, Signal
from PySide6.QtGui import (
    QColor, QFont, QKeySequence, QPainter, QPen, QRadialGradient, QShortcut,
)
from PySide6.QtWidgets import (
    QApplication, QCheckBox, QColorDialog, QComboBox, QDoubleSpinBox, QFrame,
    QGroupBox, QHBoxLayout, QLabel, QMainWindow, QPlainTextEdit, QPushButton,
    QSpinBox, QVBoxLayout, QWidget,
)

from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .editor_qt.undo_history import SnapshotUndo
from .modules.particles import _ParticleSystem

_MODES = ("circle", "pixel", "square", "streak", "glow")


def _compute_colors(sys: _ParticleSystem):
    """Finale Farben pro Partikel (repliziert particles._draw)."""
    lifetimes = np.maximum(sys._lifetimes, 1).astype(np.float32)
    life_t = np.clip(sys._ages.astype(np.float32) / lifetimes, 0.0, 1.0)
    sr = ((sys._colors >> 16) & 0xFF).astype(np.float32)
    sg = ((sys._colors >> 8) & 0xFF).astype(np.float32)
    sb = (sys._colors & 0xFF).astype(np.float32)
    if sys.has_color_end:
        er = (sys.color_end >> 16) & 0xFF
        eg = (sys.color_end >> 8) & 0xFF
        eb = sys.color_end & 0xFF
        inv = 1.0 - life_t
        sr = sr * inv + er * life_t
        sg = sg * inv + eg * life_t
        sb = sb * inv + eb * life_t
    if sys.fade:
        f = 1.0 - life_t
        sr, sg, sb = sr * f, sg * f, sb * f
    return (np.clip(sr, 0, 255).astype(np.int32),
            np.clip(sg, 0, 255).astype(np.int32),
            np.clip(sb, 0, 255).astype(np.int32))


class _Preview(QWidget):
    """Echtzeit-Partikel-Vorschau (QTimer-getrieben)."""

    def __init__(self, sys: _ParticleSystem, parent=None):
        super().__init__(parent)
        self.sys = sys
        self.emit_rate = 6
        self.paused = False
        self.setMinimumSize(420, 420)
        self._timer = QTimer(self)
        self._timer.setInterval(16)            # ~60 fps
        self._timer.timeout.connect(self._tick)
        self._timer.start()

    def _tick(self) -> None:
        if not self.paused:
            self.sys.x = self.width() / 2.0
            self.sys.y = self.height() / 2.0
            self.sys.emit(self.emit_rate)
            self.sys.update(16)
        self.update()

    def paintEvent(self, _event):  # noqa: N802
        p = QPainter(self)
        p.fillRect(self.rect(), QColor(COLORS["bg"]))
        n = self.sys.count()
        if n == 0:
            return
        xs = self.sys._xs
        ys = self.sys._ys
        sizes = self.sys._sizes
        rr, gg, bb = _compute_colors(self.sys)
        mode = self.sys.mode
        if mode == "glow":
            p.setCompositionMode(QPainter.CompositionMode.CompositionMode_Plus)
        p.setRenderHint(QPainter.RenderHint.Antialiasing, mode in ("circle", "glow"))
        for i in range(n):
            x = float(xs[i]); y = float(ys[i])
            s = int(sizes[i]) or 1
            col = QColor(int(rr[i]), int(gg[i]), int(bb[i]))
            if mode == "pixel":
                p.setPen(QPen(col, 2))
                p.drawPoint(int(x), int(y))
            elif mode == "square":
                p.fillRect(int(x - s), int(y - s), s * 2, s * 2, col)
            elif mode == "streak":
                vx = float(self.sys._vxs[i]); vy = float(self.sys._vys[i])
                p.setPen(QPen(col, max(1, s)))
                p.drawLine(int(x), int(y),
                           int(x - vx * 0.04), int(y - vy * 0.04))
            elif mode == "glow":
                grad = QRadialGradient(x, y, s * 2)
                c0 = QColor(col); c1 = QColor(col); c1.setAlpha(0)
                grad.setColorAt(0.0, c0)
                grad.setColorAt(1.0, c1)
                p.setPen(Qt.PenStyle.NoPen)
                p.setBrush(grad)
                p.drawEllipse(int(x - s * 2), int(y - s * 2), s * 4, s * 4)
            else:  # circle
                p.setPen(Qt.PenStyle.NoPen)
                p.setBrush(col)
                p.drawEllipse(int(x - s), int(y - s), s * 2, s * 2)


class _ColorButton(QPushButton):
    """Button, der seine Farbe als Swatch zeigt; Klick oeffnet den Picker."""

    colorChanged = Signal(int)

    def __init__(self, value: int, parent=None):
        super().__init__(parent)
        self._value = value & 0xFFFFFF
        self.setFixedSize(54, 24)
        self.clicked.connect(self._pick)
        self._refresh()

    def value(self) -> int:
        return self._value

    def set_value(self, v: int) -> None:
        self._value = v & 0xFFFFFF
        self._refresh()

    def _refresh(self) -> None:
        c = QColor((self._value >> 16) & 0xFF, (self._value >> 8) & 0xFF,
                   self._value & 0xFF)
        self.setStyleSheet(
            f"background-color: {c.name()}; border: 1px solid {COLORS['border']}; "
            f"border-radius: 3px;")

    def _pick(self) -> None:
        c = QColor((self._value >> 16) & 0xFF, (self._value >> 8) & 0xFF,
                   self._value & 0xFF)
        new = QColorDialog.getColor(c, self, "Farbe waehlen")
        if new.isValid():
            self.set_value((new.red() << 16) | (new.green() << 8) | new.blue())
            self.colorChanged.emit(self._value)


class ParticleEditor(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = project_root
        self.setWindowTitle("GameBasic Partikel-Editor")
        self.resize(960, 660)
        self.sys = _ParticleSystem(0.0, 0.0)

        central = QWidget()
        self.setCentralWidget(central)
        root = QHBoxLayout(central)

        # Vorschau zuerst anlegen -- _build_controls ruft _on_change(), das
        # die Preview-Emission-Rate setzt.
        self.preview = _Preview(self.sys)

        # --- Steuerung links ---------------------------------------
        controls = QWidget()
        controls.setFixedWidth(380)
        cl = QVBoxLayout(controls)
        cl.setSpacing(8)
        self._build_controls(cl)
        cl.addStretch(1)
        root.addWidget(controls)

        # --- Vorschau rechts ---------------------------------------
        root.addWidget(self.preview, 1)

        # Undo/Redo ueber Snapshots aller Parameter-Widgets.
        self.undo = SnapshotUndo(self._capture_state, self._apply_state,
                                 debounce_ms=250)
        self.undo.changed.connect(self._update_undo_buttons)
        self.btn_undo.clicked.connect(self.undo.undo)
        self.btn_redo.clicked.connect(self.undo.redo)
        QShortcut(QKeySequence.StandardKey.Undo, self, activated=self.undo.undo)
        QShortcut(QKeySequence.StandardKey.Redo, self, activated=self.undo.redo)
        QShortcut(QKeySequence("Ctrl+Y"), self, activated=self.undo.redo)
        self._update_undo_buttons()

    def _update_undo_buttons(self) -> None:
        self.btn_undo.setEnabled(self.undo.can_undo())
        self.btn_redo.setEnabled(self.undo.can_redo())

    def _capture_state(self) -> dict:
        return {
            "vx_min": self.vx_min.value(), "vx_max": self.vx_max.value(),
            "vy_min": self.vy_min.value(), "vy_max": self.vy_max.value(),
            "gx": self.gx.value(), "gy": self.gy.value(),
            "mode": self.mode.currentText(),
            "size_min": self.size_min.value(), "size_max": self.size_max.value(),
            "color": self.color.value(),
            "color_end_on": self.color_end_on.isChecked(),
            "color_end": self.color_end.value(),
            "fade": self.fade.isChecked(),
            "life_min": self.life_min.value(), "life_max": self.life_max.value(),
            "rate": self.rate.value(),
        }

    def _apply_state(self, s: dict) -> None:
        self.vx_min.setValue(s["vx_min"]); self.vx_max.setValue(s["vx_max"])
        self.vy_min.setValue(s["vy_min"]); self.vy_max.setValue(s["vy_max"])
        self.gx.setValue(s["gx"]); self.gy.setValue(s["gy"])
        self.mode.setCurrentText(s["mode"])
        self.size_min.setValue(s["size_min"]); self.size_max.setValue(s["size_max"])
        self.color.set_value(s["color"])
        self.color_end_on.setChecked(s["color_end_on"])
        self.color_end.set_value(s["color_end"])
        self.fade.setChecked(s["fade"])
        self.life_min.setValue(s["life_min"]); self.life_max.setValue(s["life_max"])
        self.rate.setValue(s["rate"])
        self._on_change()

    # ------------------------------------------------- Controls
    def _build_controls(self, cl: QVBoxLayout) -> None:
        title = QLabel("Partikel-Editor")
        tf = QFont(); tf.setBold(True); tf.setPointSize(13)
        title.setFont(tf)
        cl.addWidget(title)

        # Bewegung
        g_move = QGroupBox("Bewegung")
        ml = QVBoxLayout(g_move)
        self.vx_min = self._dspin(ml, "vx min", -500, 500, self.sys.vx_min)
        self.vx_max = self._dspin(ml, "vx max", -500, 500, self.sys.vx_max)
        self.vy_min = self._dspin(ml, "vy min", -500, 500, self.sys.vy_min)
        self.vy_max = self._dspin(ml, "vy max", -500, 500, self.sys.vy_max)
        self.gx = self._dspin(ml, "Gravity x", -1000, 1000, self.sys.gravity_x)
        self.gy = self._dspin(ml, "Gravity y", -1000, 1000, self.sys.gravity_y)
        cl.addWidget(g_move)

        # Aussehen
        g_look = QGroupBox("Aussehen")
        ll = QVBoxLayout(g_look)
        self.mode = QComboBox(); self.mode.addItems(_MODES)
        self.mode.currentTextChanged.connect(self._on_change)
        self._row(ll, "Modus", self.mode)
        self.size_min = self._ispin(ll, "Groesse min", 1, 64, self.sys.size_min)
        self.size_max = self._ispin(ll, "Groesse max", 1, 64, self.sys.size_max)
        self.color = _ColorButton(self.sys.color)
        self.color.colorChanged.connect(self._on_change)
        self._row(ll, "Farbe", self.color)
        self.color_end_on = QCheckBox("Farbverlauf zu")
        self.color_end_on.toggled.connect(self._on_change)
        self.color_end = _ColorButton(0xFF3000)
        self.color_end.colorChanged.connect(self._on_change)
        row = QHBoxLayout()
        row.addWidget(self.color_end_on)
        row.addWidget(self.color_end)
        row.addStretch(1)
        ll.addLayout(row)
        self.fade = QCheckBox("Fade (am Ende abdunkeln)")
        self.fade.setChecked(self.sys.fade)
        self.fade.toggled.connect(self._on_change)
        ll.addWidget(self.fade)
        cl.addWidget(g_look)

        # Lebenszeit & Emission
        g_emit = QGroupBox("Lebenszeit & Emission")
        el = QVBoxLayout(g_emit)
        self.life_min = self._ispin(el, "Lebensdauer min (ms)", 50, 8000,
                                    self.sys.lifetime_min)
        self.life_max = self._ispin(el, "Lebensdauer max (ms)", 50, 8000,
                                    self.sys.lifetime_max)
        self.rate = self._ispin(el, "Emission/Frame", 0, 200, 6)
        cl.addWidget(g_emit)

        # Buttons
        btns = QHBoxLayout()
        self.btn_undo = QPushButton("↶")
        self.btn_undo.setToolTip("Rueckgaengig (Strg+Z)")
        self.btn_undo.setFixedWidth(34)
        btns.addWidget(self.btn_undo)
        self.btn_redo = QPushButton("↷")
        self.btn_redo.setToolTip("Wiederholen (Strg+Y)")
        self.btn_redo.setFixedWidth(34)
        btns.addWidget(self.btn_redo)
        self.btn_pause = QPushButton("Pause")
        self.btn_pause.setCheckable(True)
        self.btn_pause.toggled.connect(
            lambda on: setattr(self.preview, "paused", on))
        btns.addWidget(self.btn_pause)
        btn_clear = QPushButton("Leeren")
        btn_clear.clicked.connect(self.sys.clear)
        btns.addWidget(btn_clear)
        btn_export = QPushButton("GB-Code exportieren")
        btn_export.setProperty("accent", True)
        btn_export.clicked.connect(self._export)
        btns.addWidget(btn_export)
        cl.addLayout(btns)

        self._on_change()

    # ---- Widget-Helfer
    def _row(self, layout, label, widget) -> None:
        r = QHBoxLayout()
        lab = QLabel(label)
        lab.setFixedWidth(130)
        r.addWidget(lab)
        r.addWidget(widget, 1)
        layout.addLayout(r)

    def _dspin(self, layout, label, lo, hi, val) -> QDoubleSpinBox:
        sp = QDoubleSpinBox()
        sp.setRange(lo, hi)
        sp.setValue(val)
        sp.setSingleStep(5.0)
        sp.valueChanged.connect(self._on_change)
        self._row(layout, label, sp)
        return sp

    def _ispin(self, layout, label, lo, hi, val) -> QSpinBox:
        sp = QSpinBox()
        sp.setRange(lo, hi)
        sp.setValue(int(val))
        sp.valueChanged.connect(self._on_change)
        self._row(layout, label, sp)
        return sp

    # ------------------------------------------------- Sync
    def _on_change(self, *_a) -> None:
        s = self.sys
        s.vx_min = self.vx_min.value()
        s.vx_max = max(self.vx_min.value(), self.vx_max.value())
        s.vy_min = self.vy_min.value()
        s.vy_max = max(self.vy_min.value(), self.vy_max.value())
        s.gravity_x = self.gx.value()
        s.gravity_y = self.gy.value()
        s.mode = self.mode.currentText()
        s.size_min = self.size_min.value()
        s.size_max = max(self.size_min.value(), self.size_max.value())
        s.color = self.color.value()
        s.has_color_end = self.color_end_on.isChecked()
        s.color_end = self.color_end.value()
        s.fade = self.fade.isChecked()
        s.lifetime_min = self.life_min.value()
        s.lifetime_max = max(self.life_min.value(), self.life_max.value())
        self.preview.emit_rate = self.rate.value()
        u = getattr(self, "undo", None)
        if u is not None:
            u.mark()

    # ------------------------------------------------- Export
    def _export(self) -> None:
        s = self.sys
        lines = [
            'IMPORT "particles"',
            "",
            "DIM ps AS PARTICLE_SYSTEM",
            "ps = PARTICLE_SYSTEM_NEW(160, 120)",
            f"PARTICLE_SET_VELOCITY(ps, {s.vx_min:g}, {s.vx_max:g}, "
            f"{s.vy_min:g}, {s.vy_max:g})",
            f"PARTICLE_SET_LIFETIME(ps, {s.lifetime_min}, {s.lifetime_max})",
            f"PARTICLE_SET_GRAVITY(ps, {s.gravity_x:g}, {s.gravity_y:g})",
            f"PARTICLE_SET_SIZE(ps, {s.size_min}, {s.size_max})",
            f"PARTICLE_SET_COLOR(ps, &H{s.color:06X})",
        ]
        if s.has_color_end:
            lines.append(f"PARTICLE_SET_COLOR_END(ps, &H{s.color_end:06X})")
        lines.append(f"PARTICLE_SET_FADE(ps, {'TRUE' if s.fade else 'FALSE'})")
        lines.append(f'PARTICLE_SET_MODE(ps, "{s.mode}")')
        lines += [
            "",
            "' --- im Game-Loop ---",
            f"' PARTICLE_EMIT(ps, {self.rate.value()})",
            "' PARTICLE_UPDATE(ps, 16)",
            "' PARTICLE_DRAW(ps)",
        ]
        code = "\n".join(lines)

        dlg = QFrame(self, Qt.WindowType.Window)
        dlg.setWindowTitle("GB-Code")
        dlg.resize(560, 420)
        dl = QVBoxLayout(dlg)
        edit = QPlainTextEdit()
        edit.setPlainText(code)
        edit.setFont(QFont(EDITOR_FONT_FAMILY, 10))
        edit.setReadOnly(True)
        dl.addWidget(edit)
        row = QHBoxLayout()
        row.addStretch(1)
        btn_copy = QPushButton("In Zwischenablage")
        btn_copy.setProperty("accent", True)
        btn_copy.clicked.connect(
            lambda: QApplication.clipboard().setText(code))
        row.addWidget(btn_copy)
        dl.addLayout(row)
        dlg.show()
        self._export_dlg = dlg          # Referenz halten


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance() or QApplication([])
    app.setStyleSheet(global_qss())
    win = ParticleEditor(project_root)
    win.show()
    return app.exec()
