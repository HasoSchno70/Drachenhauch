"""Partikel-Editor fuer Drachenhauch (`dhparticles` / `dhrun.py --particles`).

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
    QColor, QFont, QKeySequence, QPainter, QPen, QShortcut,
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

# Vorschau-Frame-Zeit (ms) -- treibt Timer-Intervall, Sim-dt UND den
# Burst-Hold; an einer Stelle benannt statt als Literal 16 an fuenf Stellen
# dupliziert (Review-Fund: Re-Timing haette sonst leicht eine Stelle vergessen).
_FRAME_MS = 16

# Trail-Laenge im "streak"-Modus (Vorschau UND Laufzeit multiplizieren die
# Geschwindigkeit mit demselben Faktor) -- muss synchron zu
# `rust/drachenhauch_runtime/src/vm.rs`s `particle_draw` (Zeile mit `* 0.04`) bleiben.
_STREAK_TRAIL_FACTOR = 0.04

# Burst-Groesse: min. Partikelzahl bzw. Vielfaches der Dauerrate (Burst-
# Button UND generierte Test-Demo nutzen dieselbe Formel).
_BURST_MIN = 40
_BURST_RATE_MULTIPLIER = 8

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
    """Finale Farben (+ Alpha) pro Partikel -- repliziert `particle_color()` in
    `rust/drachenhauch_runtime/src/vm.rs` (der eigentliche Laufzeit-Pfad; das fruehere
    Python-Modul `particles.py`, auf das diese Funktion urspruenglich verwies,
    ist seit Phase 8 entfernt).

    Review-Fund: FADE senkte hier frueher die RGB-Kanaele Richtung Schwarz ab,
    waehrend die Laufzeit laengst auf Alpha-Fade umgestellt wurde (funktioniert
    auch additiv/auf hellem Untergrund) -- die Vorschau zeigte auf einem
    hellen Hintergrund erloeschende Partikel faelschlich schwarz statt
    transparent. Jetzt identisch zu `vm.rs::particle_color`."""
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
        aa = np.clip(np.round((1.0 - life_t) * 255.0), 1, 255)
    else:
        aa = np.full_like(life_t, 255.0)
    return (np.clip(sr, 0, 255).astype(np.int32),
            np.clip(sg, 0, 255).astype(np.int32),
            np.clip(sb, 0, 255).astype(np.int32),
            aa.astype(np.int32))


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
        self._timer.setInterval(_FRAME_MS)     # ~60 fps
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
        self._burst_hold = int(self.sys.lifetime_max / _FRAME_MS) + 30

    def _tick(self) -> None:
        if self.paused:
            return
        self.sys.x = self.width() / 2.0
        self.sys.y = self.height() / 2.0
        if self._burst_hold > 0:
            self._burst_hold -= 1     # waehrend des Bursts kein Dauer-Nachschub
        else:
            self.sys.emit(self.emit_rate)
        self.sys.update(_FRAME_MS)
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
        rr, gg, bb, aa = _compute_colors(self.sys)
        mode = self.sys.mode
        # Review-Fund: die Laufzeit (vm.rs::particle_draw) kennt kein echtes
        # additives Glow -- ihr eigener Kommentar sagt es explizit ("glow
        # additiv ist im Recording-Modell nicht direkt machbar -> Kreis-
        # Approx."), "glow" faellt dort in denselben Match-Arm wie "circle"
        # (ein deckender Kreis, keine additive Ueberblendung, keine
        # Groessen-Aufweitung). Die Vorschau zeichnete "glow" zuvor als
        # 4x-grosses additives Radialverlauf-Bloom -- optisch komplett anders
        # als das, was im Spiel tatsaechlich erscheint. Jetzt identisch: nur
        # "circle" behandelt beide Modi.
        p.setRenderHint(QPainter.RenderHint.Antialiasing, mode not in ("pixel", "square", "streak"))
        for i in range(n):
            x = float(xs[i]); y = float(ys[i])
            s = int(sizes[i]) or 1
            col = QColor(int(rr[i]), int(gg[i]), int(bb[i]), int(aa[i]))
            if mode == "pixel":
                # g.plot() zeichnet exakt 1 Pixel -- die Vorschau malte zuvor
                # per QPen-Breite 2 einen 2x2-Block.
                p.fillRect(int(x), int(y), 1, 1, col)
            elif mode == "square":
                # g.box_fill(x-sz,...,x+sz,...) ist EINSCHLIESSLICH beider
                # Kanten -> Breite/Hoehe = 2*sz+1 (nicht 2*sz).
                p.fillRect(int(x - s), int(y - s), s * 2 + 1, s * 2 + 1, col)
            elif mode == "streak":
                # g.line() (raylib draw_line) ist immer 1px -- die "Groesse"
                # beeinflusst im Spiel NUR die (hier weiterhin nachgebildete)
                # Trail-Laenge, nie die Strichstaerke.
                vx = float(self.sys._vxs[i]); vy = float(self.sys._vys[i])
                p.setPen(QPen(col, 1))
                p.drawLine(int(x), int(y),
                           int(x - vx * _STREAK_TRAIL_FACTOR),
                           int(y - vy * _STREAK_TRAIL_FACTOR))
            else:  # circle & glow (siehe Kommentar oben)
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
        self.setWindowTitle("Drachenhauch Partikel-Editor")
        self.resize(1240, 820)
        self.sys = _ParticleSystem(0.0, 0.0)
        self._syncing = False

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
                                 debounce_ms=250, parent=self)
        self.undo.changed.connect(self._update_undo_buttons)
        self.btn_undo.clicked.connect(self.undo.undo)
        self.btn_redo.clicked.connect(self.undo.redo)
        QShortcut(QKeySequence.StandardKey.Undo, self, activated=self.undo.undo)
        # Review-Fund: NUR den Standard-Key registrieren -- auf Windows IST
        # QKeySequence::Redo bereits Ctrl+Y. Ein zusaetzliches, separates
        # QShortcut("Ctrl+Y") auf demselben Fenster/Kontext macht daraus eine
        # AMBIGUE Sequenz -> Qt feuert activatedAmbiguously() statt
        # activated(), Ctrl+Y wurde also gar nicht ausgefuehrt (nur der
        # Toolbar-Button funktionierte, was den Bug verdeckte).
        QShortcut(QKeySequence.StandardKey.Redo, self, activated=self.undo.redo)
        self._update_undo_buttons()

    def closeEvent(self, event) -> None:  # noqa: N802
        # Preview-QTimer liefe sonst nach dem Schliessen unbegrenzt weiter --
        # besonders im launch()-Pfad, der eine bestehende QApplication
        # wiederverwendet (dort ist das Fenster nicht WA_DeleteOnClose,
        # Schliessen wuerde es nur verstecken). Kein Dokumentmodell hier
        # (Presets + Export sind die einzige Persistenz) -> kein Dirty-Check
        # noetig, analog zum SFX-Generator (audiostudio_qt.py).
        self.preview._timer.stop()
        event.accept()

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
        """Setzt alle Parameter-Widgets aus einem Snapshot (Undo ODER ein
        User-Preset aus `~/.drachenhauch/presets/particles.json`).

        Review-Fund: `PresetLibrary.load()` validiert das JSON nicht gegen
        ein Schema -- ein handbearbeitetes oder von einer aelteren/neueren
        Editor-Version stammendes Preset mit fehlendem Key liess diese
        Methode zuvor mit einem KeyError innerhalb eines Qt-Slots abstuerzen.
        Jetzt behaelt ein fehlender Key einfach den aktuellen Widget-Wert.
        Ein `mode`-Wert ausserhalb von `_MODES` wurde zuvor STILL verworfen
        (setCurrentText() ist auf einer nicht-editierbaren Combo ein No-Op
        fuer unbekannte Strings) -- der alte Modus blieb unbemerkt aktiv und
        wurde beim naechsten Undo-Snapshot als "der geladene Zustand"
        festgeschrieben. Jetzt explizit geprueft, gleiches Verhalten
        (behalte aktuellen Modus), aber ohne den stillen Seiteneffekt."""
        g = s.get
        self.vx_min.setValue(g("vx_min", self.vx_min.value()))
        self.vx_max.setValue(g("vx_max", self.vx_max.value()))
        self.vy_min.setValue(g("vy_min", self.vy_min.value()))
        self.vy_max.setValue(g("vy_max", self.vy_max.value()))
        self.gx.setValue(g("gx", self.gx.value()))
        self.gy.setValue(g("gy", self.gy.value()))
        mode = g("mode", self.mode.currentText())
        if mode in _MODES:
            self.mode.setCurrentText(mode)
        self.size_min.setValue(g("size_min", self.size_min.value()))
        self.size_max.setValue(g("size_max", self.size_max.value()))
        self.color.set_value(g("color", self.color.value()))
        self.color_end_on.setChecked(g("color_end_on", self.color_end_on.isChecked()))
        self.color_end.set_value(g("color_end", self.color_end.value()))
        self.fade.setChecked(g("fade", self.fade.isChecked()))
        self.life_min.setValue(g("life_min", self.life_min.value()))
        self.life_max.setValue(g("life_max", self.life_max.value()))
        self.rate.setValue(g("rate", self.rate.value()))
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
        # Modus-Liste kommt aus _Preview._BG selbst (+ "checker", das dort
        # nicht als Farbe gefuehrt wird, weil es prozedural gezeichnet wird)
        # -- vorher war diese Liste eine dritte, separate Kopie der Modi und
        # ein Tippfehler haette hier still auf den Dunkel-Hintergrund
        # zurueckgefallen (_BG.get(..., _BG["dark"])), statt sichtbar zu
        # brechen (Review-Fund).
        bg_labels = {"dark": "Dunkel", "black": "Schwarz", "light": "Hell",
                     "checker": "Schachbrett"}
        for mode in (*_Preview._BG.keys(), "checker"):
            self.bg_combo.addItem(bg_labels[mode], mode)
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
            lambda: self.preview.burst(self._burst_count()))
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
        btn_clear.clicked.connect(self._on_clear)
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
        if self._syncing:
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
        # _enforce_minmax() laeuft zuerst und garantiert bereits hi >= lo auf
        # den Widgets selbst -- die max(lo, hi)-Klemmung hier war ein zweiter,
        # redundanter Mechanismus fuer dieselbe Invariante (Review-Fund).
        self._enforce_minmax()
        s = self.sys
        s.vx_min = self.vx_min.value()
        s.vx_max = self.vx_max.value()
        s.vy_min = self.vy_min.value()
        s.vy_max = self.vy_max.value()
        s.gravity_x = self.gx.value()
        s.gravity_y = self.gy.value()
        s.mode = self.mode.currentText()
        s.size_min = self.size_min.value()
        s.size_max = self.size_max.value()
        s.color = self.color.value()
        s.has_color_end = self.color_end_on.isChecked()
        s.color_end = self.color_end.value()
        s.fade = self.fade.isChecked()
        s.lifetime_min = self.life_min.value()
        s.lifetime_max = self.life_max.value()
        self.preview.emit_rate = self.rate.value()
        u = getattr(self, "undo", None)
        if u is not None:
            u.mark()

    def _burst_count(self) -> int:
        """Burst-Groesse aus der Dauerrate -- von Burst-Button UND der
        generierten Test-Demo geteilt (waren zuvor zwei Kopien derselben
        Formel, Review-Fund)."""
        return max(_BURST_MIN, self.rate.value() * _BURST_RATE_MULTIPLIER)

    def _on_clear(self) -> None:
        """Review-Fund: 'Leeren' waehrend eines aktiven Bursts loeschte die
        Partikel, liess aber `_burst_hold` (bis zu ~530 Frames / ~8.5s bei
        maximaler Lebensdauer) unangetastet -- die Dauer-Emission blieb also
        weiter ausgesetzt und die Vorschau wirkte fuer mehrere Sekunden
        eingefroren, ohne jeden Hinweis darauf."""
        self.sys.clear()
        self.preview._burst_hold = 0

    # ------------------------------------------------- Export
    def _particle_set_lines(self) -> list[str]:
        """PARTICLE_SET_*-Zeilen aus dem aktuellen Zustand -- von `_export()`
        UND `_build_runnable_demo()` geteilt (waren zuvor zwei Kopien
        derselben neun Zeilen; ein neuer Parameter liess sich so leicht in
        einem der beiden Pfade vergessen, Review-Fund)."""
        s = self.sys
        lines = [
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
        return lines

    def _export(self) -> None:
        lines = [
            'IMPORT "particles"',
            "",
            "DIM ps AS PARTICLE_SYSTEM",
            "ps = PARTICLE_SYSTEM_NEW(160, 120)",
            *self._particle_set_lines(),
            "",
            "' --- im Game-Loop ---",
            f"' PARTICLE_EMIT(ps, {self.rate.value()})",
            f"' PARTICLE_UPDATE(ps, {_FRAME_MS})",
            "' PARTICLE_DRAW(ps)",
        ]
        code = "\n".join(lines)

        dlg = QFrame(self, Qt.WindowType.Window)
        dlg.setWindowTitle("GB-Code")
        dlg.resize(560, 420)
        # Review-Fund: dieses Fenster blieb bisher NICHT-modal, waehrend
        # "Kopieren"/"Speichern" den `code`-String vom OEFFNEN des Dialogs
        # einfrieren -- tunte man danach weiter an den Reglern, gaben
        # Anzeige/Kopieren/Speichern und der (live neu bauende) "In
        # Drachenhauch testen"-Button drei VERSCHIEDENE Ergebnisse. WindowModal
        # blockiert waehrend der Dialog offen ist genau das Eltern-Fenster
        # (`self`) -- der eingefrorene Snapshot kann dadurch gar nicht mehr
        # veralten, ohne den funktionierenden Test-Button anzutasten.
        dlg.setWindowModality(Qt.WindowModality.WindowModal)
        dl = QVBoxLayout(dlg)
        edit = QPlainTextEdit()
        edit.setPlainText(code)
        edit.setFont(QFont(EDITOR_FONT_FAMILY, 10))
        edit.setReadOnly(True)
        dl.addWidget(edit)
        row = QHBoxLayout()
        row.addStretch(1)
        btn_test = QPushButton("In Drachenhauch testen")
        btn_test.setToolTip("Lauffaehige Demo (Maus = Emitter) im dhrt-Fenster starten")
        btn_test.clicked.connect(self._run_in_drachenhauch)
        row.addWidget(btn_test)
        btn_save = QPushButton("In .dh speichern...")
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
        # Review-Fund: `self._export_dlg` blieb nach der Zerstoerung ein
        # dangelnder Verweis auf ein geloeschtes C++-Objekt -- jede spaetere
        # Beruehrung haette ein RuntimeError geworfen. Gleiches Muster wie
        # main_window.py's WA_DeleteOnClose-Referenzen.
        dlg.destroyed.connect(lambda: setattr(self, "_export_dlg", None))
        dlg.show()
        self._export_dlg = dlg          # Referenz halten, solange offen

    def _save_snippet(self, code: str) -> None:
        path, _ = QFileDialog.getSaveFileName(
            self, "GB-Code speichern",
            str(self.project_root / "partikel.dh"), "Drachenhauch (*.dh)")
        if not path:
            return
        try:
            Path(path).write_text(code, encoding="utf-8")
        except Exception as exc:
            QMessageBox.critical(self, "Speichern fehlgeschlagen", str(exc))

    def _build_runnable_demo(self) -> str:
        """Lauffaehiges Test-Programm (im Gegensatz zum Export-Snippet, dessen
        Game-Loop auskommentiert ist): Fenster + Loop, Emitter folgt der Maus,
        Klick = Eruption. Mirror von examples/28_particles_visual.dh."""
        rate = self.rate.value()
        burst = self._burst_count()
        lines = [
            "' Auto-generiert vom Partikel-Editor.",
            "' Maus bewegen = Emitter, Klick = Eruption, ESC = Ende.",
            'IMPORT "particles"',
            "",
            'SCREEN(640, 480, "Partikel-Test")',
            "",
            "DIM ps AS PARTICLE_SYSTEM",
            "ps = PARTICLE_SYSTEM_NEW(320.0, 240.0)",
            *self._particle_set_lines(),
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
            f"    SLEEP({_FRAME_MS})",
            "WEND",
        ]
        return "\n".join(lines)

    def _run_in_drachenhauch(self) -> None:
        import sys
        import subprocess
        import tempfile
        dhrun = self.project_root / "dhrun.py"
        if not dhrun.exists():
            QMessageBox.warning(
                self, "dhrun.py fehlt",
                f"dhrun.py nicht gefunden in {self.project_root}.\n"
                f"Der Partikel-Test braucht das CLI, um GB-Programme zu starten.")
            return
        # Temp-Dir bewusst NICHT loeschen -- der dhrun-Subprozess liest die
        # Datei noch beim Start (OS-Cleanup nach Reboot ist OK).
        tmpdir = Path(tempfile.mkdtemp(prefix="gb_particle_test_"))
        dh_path = tmpdir / "_test.dh"
        try:
            # Review-Fund: das Schreiben lag zuvor AUSSERHALB des try-Blocks
            # -- ein OSError (voller Datentraeger, keine Schreibrechte etc.)
            # entkam als unbehandelter Traceback, waehrend der Popen-Aufruf
            # direkt darunter bereits sauber abgefangen wurde.
            dh_path.write_text(self._build_runnable_demo(), encoding="utf-8")
            # QProcess statt Popen -- kein Konsolenfenster, Fehler kommen im
            # Editor an (siehe editor_qt/vorschau_start.py).
            from .editor_qt.vorschau_start import starte_vorschau
            self._vorschau = starte_vorschau(
                self, [sys.executable, str(dhrun), str(dh_path)], tmpdir,
                titel="Partikel-Test")
        except Exception as exc:
            QMessageBox.critical(self, "Start fehlgeschlagen", str(exc))


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    # `initial_file` wird bewusst ignoriert: anders als Score-/Tracker-/
    # Tilemap-Editor hat der Partikel-Editor kein Dateimodell zum Oeffnen --
    # Zustand kommt nur aus der Preset-Bibliothek oder GB-Code-Export. Der
    # Parameter existiert nur fuer eine einheitliche launch()-Signatur ueber
    # alle Begleit-Editoren hinweg (dhrun.py ruft ihn immer mit `None` auf).
    app = QApplication.instance()
    if app is None:
        # Fusion-Style NUR bei frischer QApplication erzwingen -- siehe
        # trackereditor_qt.launch() fuer die volle Begruendung.
        app = QApplication([])
        app.setStyle("Fusion")
    app.setStyleSheet(global_qss())
    win = ParticleEditor(project_root)
    win.show()
    return app.exec()
