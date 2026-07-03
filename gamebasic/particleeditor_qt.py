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
    QApplication, QCheckBox, QColorDialog, QComboBox, QDoubleSpinBox,
    QFileDialog, QFrame, QGroupBox, QHBoxLayout, QLabel, QMainWindow,
    QMessageBox, QPlainTextEdit, QPushButton, QSpinBox, QVBoxLayout, QWidget,
)

from .editor_qt.fader import Fader

from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .editor_qt.undo_history import SnapshotUndo
from .editor_qt.preset_bar import PresetBar
from .editor_qt.preset_library import PresetLibrary, default_dir
from .particle_sim import _ParticleSystem

_MODES = ("circle", "pixel", "square", "streak", "glow")

# Werks-Presets (Parameter-Dicts wie _capture_state). Als Startbibliothek;
# der Nutzer kann eigene dazu speichern.
_FACTORY_PRESETS = {
    "Feuer": {
        "vx_min": -30, "vx_max": 30, "vy_min": -90, "vy_max": -40,
        "gx": 0, "gy": -20, "mode": "glow", "size_min": 3, "size_max": 7,
        "color": 0xFFAA00, "color_end_on": True, "color_end": 0xFF2000,
        "fade": True, "life_min": 400, "life_max": 900, "rate": 8,
    },
    "Rauch": {
        "vx_min": -10, "vx_max": 10, "vy_min": -40, "vy_max": -15,
        "gx": 0, "gy": -5, "mode": "circle", "size_min": 6, "size_max": 14,
        "color": 0x808088, "color_end_on": True, "color_end": 0x303038,
        "fade": True, "life_min": 800, "life_max": 1800, "rate": 4,
    },
    "Funken": {
        "vx_min": -120, "vx_max": 120, "vy_min": -150, "vy_max": -40,
        "gx": 0, "gy": 300, "mode": "streak", "size_min": 1, "size_max": 3,
        "color": 0xFFEE60, "color_end_on": False, "color_end": 0xFF3000,
        "fade": True, "life_min": 300, "life_max": 700, "rate": 10,
    },
    "Explosion": {
        "vx_min": -160, "vx_max": 160, "vy_min": -160, "vy_max": 160,
        "gx": 0, "gy": 120, "mode": "pixel", "size_min": 2, "size_max": 5,
        "color": 0xFF8020, "color_end_on": True, "color_end": 0x802000,
        "fade": True, "life_min": 250, "life_max": 600, "rate": 40,
    },
    "Regen": {
        "vx_min": -5, "vx_max": 5, "vy_min": 120, "vy_max": 200,
        "gx": 0, "gy": 200, "mode": "streak", "size_min": 1, "size_max": 2,
        "color": 0x4090E0, "color_end_on": False, "color_end": 0x102040,
        "fade": False, "life_min": 600, "life_max": 1000, "rate": 12,
    },
}


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

    # Vorschau-Hintergruende -- damit sich Glow/Fade/helle Partikel gegen
    # verschiedene Untergruende beurteilen lassen (nicht nur das dunkle Theme).
    _BG = {
        "dark":  QColor(COLORS["bg"]),
        "black": QColor(0, 0, 0),
        "light": QColor(200, 200, 200),
    }

    def __init__(self, sys: _ParticleSystem, parent=None):
        super().__init__(parent)
        self.sys = sys
        self.emit_rate = 6
        self.paused = False
        self.bg_mode = "dark"
        self._burst_hold = 0      # Frames, in denen die Dauer-Emission aussetzt
        self.setMinimumSize(420, 420)
        self._timer = QTimer(self)
        self._timer.setInterval(16)            # ~60 fps
        self._timer.timeout.connect(self._tick)
        self._timer.start()

    def set_bg_mode(self, mode: str) -> None:
        self.bg_mode = mode
        self.update()

    def burst(self, n: int) -> None:
        """Solo-Burst: bestehende Partikel + Dauer-Emission raus, dann einmalig
        `n` Partikel in der Mitte. Die Dauer-Emission setzt aus, bis der Burst
        ausgelebt hat (Frames aus der max. Lebensdauer + Puffer) -- sonst
        ueberdeckt der laufende Brunnen die Eruption und man sieht nichts."""
        self.sys.x = self.width() / 2.0
        self.sys.y = self.height() / 2.0
        self.sys.clear()
        self.sys.emit(max(1, n))
        self._burst_hold = int(self.sys.lifetime_max / 16) + 30

    def _tick(self) -> None:
        if not self.paused:
            self.sys.x = self.width() / 2.0
            self.sys.y = self.height() / 2.0
            if self._burst_hold > 0:
                self._burst_hold -= 1     # waehrend des Bursts kein Dauer-Nachschub
            else:
                self.sys.emit(self.emit_rate)
            self.sys.update(16)
        self.update()

    def _paint_bg(self, p: QPainter) -> None:
        if self.bg_mode == "checker":
            tile = 16
            a, b = QColor(58, 58, 58), QColor(38, 38, 38)
            p.fillRect(self.rect(), b)
            w, h = self.width(), self.height()
            for ty in range(0, h, tile):
                for tx in range(0, w, tile):
                    if ((tx // tile) + (ty // tile)) % 2 == 0:
                        p.fillRect(tx, ty, tile, tile, a)
        else:
            p.fillRect(self.rect(), self._BG.get(self.bg_mode, self._BG["dark"]))

    def paintEvent(self, _event):  # noqa: N802
        p = QPainter(self)
        self._paint_bg(p)
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
        # Eindeutiger objectName -> die Farb-Stylesheet-Regel wird darauf
        # eingeschraenkt und kann NICHT auf Kind-Widgets (z.B. einen vom Button
        # geoeffneten Dialog) durchschlagen.
        self.setObjectName("gbColorSwatch")
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
        # Auf diesen Button beschraenkte Regel (objectName) -- so faerbt sie
        # NUR den Swatch, nie ein Kind-/Dialogfenster.
        self.setStyleSheet(
            f"QPushButton#gbColorSwatch {{ background-color: {c.name()}; "
            f"border: 1px solid {COLORS['border']}; border-radius: 3px; }}")

    def _pick(self) -> None:
        c = QColor((self._value >> 16) & 0xFF, (self._value >> 8) & 0xFF,
                   self._value & 0xFF)
        # Dialog an das TOP-LEVEL-Fenster haengen, nicht an diesen (gefaerbten)
        # Button -- sonst erbt der Dialog dessen Hintergrundfarbe.
        new = QColorDialog.getColor(c, self.window(), "Farbe waehlen")
        if new.isValid():
            self.set_value((new.red() << 16) | (new.green() << 8) | new.blue())
            self.colorChanged.emit(self._value)


class ParticleEditor(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = project_root
        self.setWindowTitle("GameBasic Partikel-Editor")
        self.resize(1240, 820)
        self.sys = _ParticleSystem(0.0, 0.0)

        central = QWidget()
        self.setCentralWidget(central)
        root = QHBoxLayout(central)

        # Vorschau zuerst anlegen -- _build_controls ruft _on_change(), das
        # die Preview-Emission-Rate setzt.
        self.preview = _Preview(self.sys)

        # --- Steuerung links ---------------------------------------
        controls = QWidget()
        controls.setFixedWidth(440)
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
        title = QLabel("✦  Partikel-Editor")
        tf = QFont(); tf.setBold(True); tf.setPointSize(14)
        title.setFont(tf)
        # Verlaufs-Banner: links in der Akzentfarbe getoent, nach rechts
        # auslaufend.
        title.setStyleSheet(
            f"color:{COLORS['accent']}; padding:6px 10px; border-radius:6px; "
            f"background: qlineargradient(x1:0, y1:0, x2:1, y2:0, "
            f"stop:0 {self._tint(COLORS['accent'], 24)}, "
            f"stop:1 rgba(0,0,0,0));")
        cl.addWidget(title)
        cyan, mint, amber = COLORS["accent"], COLORS["success"], "#EF9F27"

        # Preset-Bibliothek (Werks-Presets + eigene)
        self.presets = PresetLibrary(
            default_dir() / "particles.json", builtins=_FACTORY_PRESETS)
        self.preset_bar = PresetBar(
            self.presets, self._capture_state, self._apply_state)
        cl.addWidget(self.preset_bar)

        # Bewegung
        g_move, ml = self._group("Bewegung", cyan)
        self.vx_min = self._dspin(ml, "vx min", -500, 500, self.sys.vx_min, cyan)
        self.vx_max = self._dspin(ml, "vx max", -500, 500, self.sys.vx_max, cyan)
        self.vy_min = self._dspin(ml, "vy min", -500, 500, self.sys.vy_min, cyan)
        self.vy_max = self._dspin(ml, "vy max", -500, 500, self.sys.vy_max, cyan)
        self.gx = self._dspin(ml, "Gravity x", -1000, 1000, self.sys.gravity_x, cyan)
        self.gy = self._dspin(ml, "Gravity y", -1000, 1000, self.sys.gravity_y, cyan)
        cl.addWidget(g_move)

        # Aussehen
        g_look, ll = self._group("Aussehen", mint)
        self.mode = QComboBox(); self.mode.addItems(_MODES)
        self.mode.currentTextChanged.connect(self._on_change)
        self._row(ll, "Modus", self.mode)
        self.size_min = self._ispin(ll, "Groesse min", 1, 64, self.sys.size_min, mint)
        self.size_max = self._ispin(ll, "Groesse max", 1, 64, self.sys.size_max, mint)
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
        g_emit, el = self._group("Lebenszeit & Emission", amber)
        self.life_min = self._ispin(el, "Lebensdauer min (ms)", 50, 8000,
                                    self.sys.lifetime_min, amber, step=10)
        self.life_max = self._ispin(el, "Lebensdauer max (ms)", 50, 8000,
                                    self.sys.lifetime_max, amber, step=10)
        self.rate = self._ispin(el, "Emission/Frame", 0, 200, 6, amber)
        cl.addWidget(g_emit)

        # Vorschau-Optionen: Hintergrund + Burst
        prev_row = QHBoxLayout()
        bg_lab = QLabel("Hintergrund")
        bg_lab.setStyleSheet(f"color:{COLORS['fg_muted']}; font-size:11px;")
        prev_row.addWidget(bg_lab)
        self.bg_combo = QComboBox()
        # (Anzeige, Modus) -- Modus geht an _Preview.set_bg_mode.
        for label, mode in (("Dunkel", "dark"), ("Schwarz", "black"),
                            ("Hell", "light"), ("Schachbrett", "checker")):
            self.bg_combo.addItem(label, mode)
        self.bg_combo.currentIndexChanged.connect(
            lambda _i: self.preview.set_bg_mode(self.bg_combo.currentData()))
        prev_row.addWidget(self.bg_combo, 1)
        self.btn_burst = QPushButton("Burst")
        self.btn_burst.setObjectName("burstBtn")
        self.btn_burst.setToolTip("Einmalige Emission (fuer Explosion/Funken)")
        # Dezenter, gedaempfter Orange-Verlauf (passt zum ruhigen Accent-Look).
        self.btn_burst.setStyleSheet(
            "QPushButton#burstBtn { background: qlineargradient("
            "x1:0, y1:0, x2:0, y2:1, stop:0 #B5651D, stop:1 #8A3B0A); "
            "color:#FFEFE0; border:0; border-radius:6px; padding:5px 16px; "
            "font-weight:600; } "
            "QPushButton#burstBtn:hover { background: qlineargradient("
            "x1:0, y1:0, x2:0, y2:1, stop:0 #C9762B, stop:1 #9C4A12); }")
        self.btn_burst.clicked.connect(
            lambda: self.preview.burst(max(40, self.rate.value() * 8)))
        prev_row.addWidget(self.btn_burst)
        cl.addLayout(prev_row)

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
        # Nutzt den globalen (jetzt ruhigen) Accent-Button-Stil aus theme.py.
        btn_export.setProperty("accent", True)
        btn_export.clicked.connect(self._export)
        btns.addWidget(btn_export)
        cl.addLayout(btns)

        self._on_change()

    # ---- Widget-Helfer
    def _row(self, layout, label, widget) -> None:
        r = QHBoxLayout()
        lab = QLabel(label)
        lab.setStyleSheet(f"color:{COLORS['fg_muted']}; font-size:11px;")
        lab.setFixedWidth(96)
        r.addWidget(lab)
        r.addWidget(widget, 1)
        layout.addLayout(r)

    @staticmethod
    def _tint(color: str, alpha_pct: int) -> str:
        """`rgba(...)`-String einer Theme-/Akzentfarbe mit gegebener Deckkraft --
        fuer dezente Verlaeufe, die in den Panel-Hintergrund auslaufen."""
        c = QColor(color)
        return f"rgba({c.red()}, {c.green()}, {c.blue()}, {alpha_pct}%)"

    def _group(self, title: str, accent: str):
        """Farbcodierte Parameter-Karte mit dezentem Verlauf: oben in der
        Gruppenfarbe getoent, nach unten in den Panel-/App-Hintergrund
        auslaufend. Liefert (GroupBox, Layout)."""
        gb = QGroupBox(title)
        gb.setStyleSheet(
            f"QGroupBox {{ background: qlineargradient(x1:0, y1:0, x2:0, y2:1, "
            f"stop:0 {self._tint(accent, 16)}, stop:0.4 {COLORS['bg_panel']}, "
            f"stop:1 {COLORS['bg']}); }} "
            f"QGroupBox::title {{ color:{accent}; "
            f"background-color:{COLORS['bg_alt']}; }}")
        lay = QVBoxLayout(gb); lay.setSpacing(7)
        return gb, lay

    def _dspin(self, layout, label, lo, hi, val, accent=None, step=1) -> Fader:
        f = Fader(label, lo, hi, val, step, 0, "", accent)
        f.valueChanged.connect(self._on_change)
        layout.addWidget(f)
        return f

    def _ispin(self, layout, label, lo, hi, val, accent=None, step=1) -> Fader:
        f = Fader(label, lo, hi, int(val), step, 0, "", accent)
        f.valueChanged.connect(self._on_change)
        layout.addWidget(f)
        return f

    # ------------------------------------------------- Sync
    def _enforce_minmax(self) -> None:
        """Haelt jedes max-Widget >= seinem min-Widget. Vorher nutzte die Sim
        still `max(min,max)`, waehrend die Spinbox den ignorierten Wert zeigte
        (Anzeige != Verhalten). Jetzt zieht das max-Widget sichtbar nach.
        blockSignals verhindert Re-Entrancy ueber valueChanged -> _on_change."""
        if getattr(self, "_syncing", False):
            return
        self._syncing = True
        try:
            for lo, hi in ((self.vx_min, self.vx_max), (self.vy_min, self.vy_max),
                           (self.size_min, self.size_max),
                           (self.life_min, self.life_max)):
                if lo.value() > hi.value():
                    hi.blockSignals(True)
                    hi.setValue(lo.value())
                    hi.blockSignals(False)
        finally:
            self._syncing = False

    def _on_change(self, *_a) -> None:
        self._enforce_minmax()
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
        btn_test = QPushButton("In GameBasic testen")
        btn_test.setToolTip("Lauffaehige Demo (Maus = Emitter) im gbrt-Fenster starten")
        btn_test.clicked.connect(self._run_in_gamebasic)
        row.addWidget(btn_test)
        btn_save = QPushButton("In .gb speichern...")
        btn_save.clicked.connect(lambda: self._save_snippet(code))
        row.addWidget(btn_save)
        btn_copy = QPushButton("In Zwischenablage")
        btn_copy.setProperty("accent", True)
        btn_copy.clicked.connect(
            lambda: QApplication.clipboard().setText(code))
        row.addWidget(btn_copy)
        dl.addLayout(row)
        # WA_DeleteOnClose: sonst haengt jedes per "GB-Code exportieren"
        # erzeugte Fenster als verstecktes Kind von `self` weiter (Qt
        # raeumt Kind-Widgets nur beim Schliessen des Eltern-Fensters auf)
        # -- wiederholtes Exportieren in einer Sitzung haette so Fenster
        # angesammelt, die nie wieder freigegeben werden.
        dlg.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose, True)
        dlg.show()
        self._export_dlg = dlg          # Referenz halten, solange offen

    def _save_snippet(self, code: str) -> None:
        path, _ = QFileDialog.getSaveFileName(
            self, "GB-Code speichern",
            str(self.project_root / "partikel.gb"), "GameBasic (*.gb)")
        if not path:
            return
        try:
            Path(path).write_text(code, encoding="utf-8")
        except Exception as exc:
            QMessageBox.critical(self, "Speichern fehlgeschlagen", str(exc))

    def _build_runnable_demo(self) -> str:
        """Lauffaehiges Test-Programm (im Gegensatz zum Export-Snippet, dessen
        Game-Loop auskommentiert ist): Fenster + Loop, Emitter folgt der Maus,
        Klick = Eruption. Mirror von examples/28_particles_visual.gb."""
        s = self.sys
        rate = self.rate.value()
        burst = max(40, rate * 8)
        lines = [
            "' Auto-generiert vom Partikel-Editor.",
            "' Maus bewegen = Emitter, Klick = Eruption, ESC = Ende.",
            'IMPORT "particles"',
            "",
            'SCREEN(640, 480, "Partikel-Test")',
            "",
            "DIM ps AS PARTICLE_SYSTEM",
            "ps = PARTICLE_SYSTEM_NEW(320.0, 240.0)",
            f"PARTICLE_SET_VELOCITY(ps, {s.vx_min:g}, {s.vx_max:g}, "
            f"{s.vy_min:g}, {s.vy_max:g})",
            f"PARTICLE_SET_LIFETIME(ps, {s.lifetime_min}, {s.lifetime_max})",
            f"PARTICLE_SET_GRAVITY(ps, {s.gravity_x:g}, {s.gravity_y:g})",
            f"PARTICLE_SET_SIZE(ps, {s.size_min}, {s.size_max})",
            f"PARTICLE_SET_COLOR(ps, &H{s.color:06X})",
        ]
        if s.has_color_end:
            lines.append(f"PARTICLE_SET_COLOR_END(ps, &H{s.color_end:06X})")
        lines += [
            f"PARTICLE_SET_FADE(ps, {'TRUE' if s.fade else 'FALSE'})",
            f'PARTICLE_SET_MODE(ps, "{s.mode}")',
            "",
            "DIM last_ms AS INTEGER",
            "last_ms = MILLIS()",
            "",
            "WHILE NOT QUITREQUESTED()",
            "    IF KEYPRESSED(27) THEN",
            "        BREAK",
            "    END IF",
            "    DIM now_ms AS INTEGER",
            "    now_ms = MILLIS()",
            "    DIM dt AS INTEGER",
            "    dt = now_ms - last_ms",
            "    last_ms = now_ms",
            "    PARTICLE_SET_POS(ps, MOUSEX() * 1.0, MOUSEY() * 1.0)",
            "    IF MOUSEBUTTON(0) THEN",
            f"        PARTICLE_EMIT(ps, {burst})",
            "    ELSE",
            f"        PARTICLE_EMIT(ps, {rate})",
            "    END IF",
            "    PARTICLE_UPDATE(ps, dt)",
            "    CLS(RGB(0, 0, 30))",
            "    PARTICLE_DRAW(ps)",
            '    TEXT(8, 8, "Maus = Emitter, Klick = Eruption, ESC = Ende", '
            "RGB(200, 200, 200))",
            "    FLIP()",
            "    SLEEP(16)",
            "WEND",
        ]
        return "\n".join(lines)

    def _run_in_gamebasic(self) -> None:
        import sys
        import subprocess
        import tempfile
        gbrun = self.project_root / "gbrun.py"
        if not gbrun.exists():
            QMessageBox.warning(
                self, "gbrun.py fehlt",
                f"gbrun.py nicht gefunden in {self.project_root}.\n"
                f"Der Partikel-Test braucht das CLI, um GB-Programme zu starten.")
            return
        # Temp-Dir bewusst NICHT loeschen -- der gbrun-Subprozess liest die
        # Datei noch beim Start (OS-Cleanup nach Reboot ist OK).
        tmpdir = Path(tempfile.mkdtemp(prefix="gb_particle_test_"))
        gb_path = tmpdir / "_test.gb"
        gb_path.write_text(self._build_runnable_demo(), encoding="utf-8")
        try:
            subprocess.Popen([sys.executable, str(gbrun), str(gb_path)],
                             cwd=str(tmpdir))
        except Exception as exc:
            QMessageBox.critical(self, "Start fehlgeschlagen", str(exc))


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance() or QApplication([])
    app.setStyleSheet(global_qss())
    win = ParticleEditor(project_root)
    win.show()
    return app.exec()
