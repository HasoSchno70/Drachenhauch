"""Chiptune-Tracker fuer GameBasic (`gbtracker` / `gbrun.py --tracker`).

Mehrspuriger Pattern-Editor: **einstellbare Kanalzahl** (4..32, "Kanaele:"-
Spinbox; je eigene Waveform, der LETZTE Kanal ist immer Noise/Drum) +
**mehrere Patterns mit einstellbarer Laenge + Song-Arrangement** (Order:
Reihenfolge, in der Patterns abgespielt werden). Noten per klickbarer
Klaviatur in die Gitter-Zellen setzen, **Block-Auswahl** (Shift-Klick/Ziehen)
fuer Copy/Cut/Paste/Transpose/Interpolate, Pattern ODER ganzen Song abspielen
(nutzt den geteilten Synth `gamebasic.synth`), Projekt als `.json` speichern/
laden und als GB-Code exportieren -- ein frame-basierter Player
(`TRACKER_UPDATE`), der mit `DELTA()` im Game-Loop laeuft.

Das Datenmodell + I/O + GB-Export liegen Qt-frei in `gamebasic.tracker`
(headless getestet).
"""
from __future__ import annotations

from pathlib import Path

import numpy as np
from PySide6.QtCore import Qt, QItemSelectionModel, QTimer, Signal
from PySide6.QtGui import QColor, QFont, QKeySequence, QPainter, QPen, QShortcut
from PySide6.QtWidgets import (
    QApplication, QCheckBox, QComboBox, QDialog, QDialogButtonBox,
    QDoubleSpinBox, QFileDialog, QFormLayout, QFrame, QHBoxLayout, QHeaderView,
    QLabel, QLineEdit, QListWidget, QListWidgetItem, QMainWindow, QMessageBox,
    QPlainTextEdit, QProgressBar, QPushButton, QScrollArea, QSlider, QSpinBox,
    QStyle, QStyledItemDelegate, QTableWidget, QTableWidgetItem,
    QTableWidgetSelectionRange, QVBoxLayout, QWidget,
)

from .audio_preview import Mixer
from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .editor_qt.undo_history import SnapshotUndo
from .synth import synthesize
from .tracker import (
    MAX_CHANNELS, MIN_CHANNELS, NOTE_OFF, SLIDE_MAX, VOL_MAX, WAVEFORMS,
    Song, midi_to_freq, note_name, vol_to_pct,
    FX_NONE, FX_CODES, FX_NAMES,
    block_copy, block_interpolate, block_paste, block_transpose,
)

# Styling fuer das Pattern-Gitter (Tracker-Look: Header-Chrome, Reihen-Nummern,
# Selektion + Playhead). Beat-Hintergruende werden pro Zelle gesetzt.
_TRACKER_GRID_QSS = f"""
QTableWidget {{
    background: {COLORS['bg']};
    border: 1px solid {COLORS['border']};
    selection-background-color: {COLORS['sel']};
    selection-color: {COLORS['fg']};
    outline: 0;
}}
QTableWidget::item {{ padding: 0px; }}
QTableWidget::item:selected {{
    background: {COLORS['sel']};
    color: {COLORS['fg']};
}}
QHeaderView::section {{
    background: {COLORS['bg_panel']};
    color: {COLORS['fg']};
    border: 0px;
    border-right: 1px solid {COLORS['border_soft']};
    border-bottom: 1px solid {COLORS['border']};
    padding: 3px;
    font-weight: bold;
}}
QHeaderView::section:vertical {{
    color: {COLORS['line_no']};
    font-weight: normal;
    padding: 0 8px;
}}
QTableCornerButton::section {{
    background: {COLORS['bg_panel']};
    border: 0px;
}}
"""


def _channel_names(n: int) -> list[str]:
    """Kanal-Labels fuer `n` Kanaele: tonale Kanaele `Ch1..Ch(n-1)`, der
    LETZTE Kanal ist immer "Drum"."""
    return [f"Ch{i + 1}" for i in range(n - 1)] + ["Drum"]


# Eigene Akzentfarbe pro Kanal (zyklisch) -- sonst sieht bei vielen Kanaelen
# alles gleich-cyan aus (Spur-Sounds-Panel, Gitter-Header, Noten-Text,
# VU-Meter, Lautstaerke-Regler). Aus bestehenden Theme-Farben zusammen-
# gestellt (kein neuer Farbwert), damit es zum Rest des Editors passt.
_CHANNEL_HUES = ("accent", "success", "danger", "warning",
                "kw_ctrl", "kw_decl", "string", "number")


def _channel_color(c: int) -> str:
    return COLORS[_CHANNEL_HUES[c % len(_CHANNEL_HUES)]]


def _slider_qss(color: str) -> str:
    """Duenner, farbiger Schieberegler (Griff + gefuellte Seite in der
    Kanalfarbe) -- Fusion-Default-Slider sind sonst recht unauffaellig grau."""
    return f"""
        QSlider::groove:horizontal {{
            height: 4px; background: {COLORS['bg_alt']}; border-radius: 2px;
        }}
        QSlider::sub-page:horizontal {{
            background: {color}; border-radius: 2px;
        }}
        QSlider::handle:horizontal {{
            background: {color}; width: 12px; margin: -5px 0; border-radius: 6px;
        }}
    """


class _WaveformView(QWidget):
    """Zeigt die Sample-Wellenform (Min/Max pro Pixel-Spalte -- schnell auch
    bei langen Samples, kein Downsample-Preprocessing noetig) mit zwei
    ziehbaren vertikalen Markern fuer Loop-Start/-Ende. Reines Vorschau-/
    Eingabe-Widget: haelt selbst keine Loop-Semantik, meldet Aenderungen nur
    ueber `loop_changed` -- der Aufrufer (Dialog) ist die Quelle der Wahrheit
    (Spinboxen bleiben bidirektional synchron)."""

    loop_changed = Signal(int, int)
    _HANDLE_PX = 6

    def __init__(self, samples, loop_start: int, loop_end: int, parent=None):
        super().__init__(parent)
        self.samples = samples if samples is not None else np.zeros(0, dtype=np.float32)
        self.n = max(1, int(self.samples.size))
        self.loop_start = max(0, min(int(loop_start), self.n))
        self.loop_end = max(0, min(int(loop_end), self.n))
        self._drag = None          # "start" | "end" | None
        self.setMinimumHeight(90)
        self.setMouseTracking(True)
        self.setToolTip("Loop-Start (gruen) / Loop-Ende (rot) per Ziehen setzen")

    def set_loop(self, start: int, end: int) -> None:
        """Von aussen (Spinbox-Aenderung) gesetzt -- rein visuell, emittiert
        KEIN loop_changed (sonst Signal-Ping-Pong mit den Spinboxen)."""
        self.loop_start = max(0, min(int(start), self.n))
        self.loop_end = max(0, min(int(end), self.n))
        self.update()

    def _frame_to_x(self, frame: int) -> float:
        return frame / self.n * max(1, self.width())

    def _x_to_frame(self, x: float) -> int:
        return int(round(x / max(1, self.width()) * self.n))

    def _hit(self, x: float) -> str | None:
        if abs(x - self._frame_to_x(self.loop_start)) <= self._HANDLE_PX:
            return "start"
        if abs(x - self._frame_to_x(self.loop_end)) <= self._HANDLE_PX:
            return "end"
        return None

    def paintEvent(self, e) -> None:  # noqa: N802
        p = QPainter(self)
        p.fillRect(self.rect(), QColor(COLORS["bg"]))
        w, h = self.width(), self.height()
        mid = h / 2.0
        if self.samples.size:
            n = self.samples.size
            step = max(1, n // max(1, w))
            p.setPen(QPen(QColor(COLORS["accent"]), 1))
            for x in range(w):
                i0 = int(x / w * n)
                i1 = min(n, i0 + step)
                if i1 <= i0:
                    continue
                seg = self.samples[i0:i1]
                lo, hi = float(seg.min()), float(seg.max())
                p.drawLine(x, int(mid - hi * mid), x, int(mid - lo * mid))
        x0, x1 = self._frame_to_x(self.loop_start), self._frame_to_x(self.loop_end)
        p.fillRect(int(x0), 0, max(1, int(x1 - x0)), h, QColor(COLORS["accent_soft"]))
        p.setPen(QPen(QColor(COLORS["success"]), 2))
        p.drawLine(int(x0), 0, int(x0), h)
        p.setPen(QPen(QColor(COLORS["danger"]), 2))
        p.drawLine(int(x1), 0, int(x1), h)
        p.end()

    def mousePressEvent(self, e) -> None:  # noqa: N802
        self._drag = self._hit(e.position().x())

    def mouseMoveEvent(self, e) -> None:  # noqa: N802
        if self._drag is None:
            return
        frame = max(0, min(self.n, self._x_to_frame(e.position().x())))
        if self._drag == "start":
            self.loop_start = min(frame, self.loop_end)
        else:
            self.loop_end = max(frame, self.loop_start)
        self.update()
        self.loop_changed.emit(self.loop_start, self.loop_end)

    def mouseReleaseEvent(self, e) -> None:  # noqa: N802
        self._drag = None


class _InstrumentDialog(QDialog):
    """Bearbeitet ein Sample-Instrument: Grundton, Loop, ADSR-Huellkurve."""

    def __init__(self, inst, parent=None):
        super().__init__(parent)
        self.inst = inst
        self.setWindowTitle(f"Instrument: {inst.name}")
        form = QFormLayout(self)
        n = inst.samples.size if inst.samples is not None else 0
        form.addRow(QLabel(f"Sample: {n} Samples @ {inst.sample_rate} Hz"))

        self.base = QSpinBox(); self.base.setRange(0, 127)
        self.base.setValue(int(inst.base_note))
        self.base.valueChanged.connect(self._upd_base_label)
        self.base_label = QLabel("")
        brow = QHBoxLayout(); brow.addWidget(self.base); brow.addWidget(self.base_label)
        bw = QWidget(); bw.setLayout(brow)
        form.addRow("Grundton (MIDI):", bw)
        self._upd_base_label()

        self.loop_mode = QComboBox()
        self.loop_mode.addItems(["none", "forward", "pingpong"])
        self.loop_mode.setCurrentText(inst.loop_mode)
        form.addRow("Loop-Modus:", self.loop_mode)
        self.loop_start = QSpinBox(); self.loop_start.setRange(0, max(0, n))
        self.loop_start.setValue(min(int(inst.loop_start), max(0, n)))
        form.addRow("Loop-Start (Sample):", self.loop_start)
        self.loop_end = QSpinBox(); self.loop_end.setRange(0, max(0, n))
        self.loop_end.setValue(min(int(inst.loop_end) or n, max(0, n)))
        form.addRow("Loop-Ende (Sample):", self.loop_end)

        # Wellenform-Vorschau mit ziehbaren Loop-Markern (bidirektional mit
        # den Spinboxen synchron -- ziehen ODER Zahl eintippen, beides geht).
        self.wave_view = _WaveformView(
            inst.samples, self.loop_start.value(), self.loop_end.value())
        form.addRow(self.wave_view)
        self.loop_start.valueChanged.connect(
            lambda v: self.wave_view.set_loop(v, self.loop_end.value()))
        self.loop_end.valueChanged.connect(
            lambda v: self.wave_view.set_loop(self.loop_start.value(), v))
        self.wave_view.loop_changed.connect(self._on_wave_loop_changed)

        self.atk = QSpinBox(); self.atk.setRange(0, 5000)
        self.atk.setValue(int(inst.env_attack_ms))
        form.addRow("Attack (ms):", self.atk)
        self.dec = QSpinBox(); self.dec.setRange(0, 5000)
        self.dec.setValue(int(inst.env_decay_ms))
        form.addRow("Decay (ms):", self.dec)
        self.sus = QDoubleSpinBox(); self.sus.setRange(0.0, 1.0)
        self.sus.setSingleStep(0.05); self.sus.setValue(float(inst.env_sustain))
        form.addRow("Sustain (0..1):", self.sus)
        self.rel = QSpinBox(); self.rel.setRange(0, 5000)
        self.rel.setValue(int(inst.env_release_ms))
        form.addRow("Release (ms):", self.rel)

        prow = QHBoxLayout()
        self.pan_slider = QSlider(Qt.Orientation.Horizontal)
        self.pan_slider.setRange(-100, 100)
        self.pan_slider.setValue(round(float(inst.pan) * 100))
        self.pan_slider.setToolTip("Stereo-Position fuer den WAV-Render "
                                   "(links .. Mitte .. rechts)")
        self.pan_slider.setStyleSheet(_slider_qss(COLORS["accent"]))
        self.pan_slider.valueChanged.connect(self._upd_pan_label)
        prow.addWidget(self.pan_slider, 1)
        self.pan_label = QLabel(""); self.pan_label.setFixedWidth(48)
        prow.addWidget(self.pan_label)
        pw = QWidget(); pw.setLayout(prow)
        form.addRow("Pan (L .. R):", pw)
        self._upd_pan_label(self.pan_slider.value())

        box = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel)
        box.accepted.connect(self.accept)
        box.rejected.connect(self.reject)
        form.addRow(box)

    def _upd_base_label(self) -> None:
        self.base_label.setText(note_name(self.base.value()))

    def _upd_pan_label(self, v: int) -> None:
        if v == 0:
            self.pan_label.setText("Mitte")
        elif v < 0:
            self.pan_label.setText(f"L {-v}%")
        else:
            self.pan_label.setText(f"R {v}%")

    def _on_wave_loop_changed(self, start: int, end: int) -> None:
        """Marker gezogen -> Spinboxen nachziehen (block_signals verhindert
        Signal-Ping-Pong zurueck zur Wellenform-Ansicht)."""
        self.loop_start.blockSignals(True)
        self.loop_end.blockSignals(True)
        self.loop_start.setValue(start)
        self.loop_end.setValue(end)
        self.loop_start.blockSignals(False)
        self.loop_end.blockSignals(False)

    def apply_to(self) -> None:
        i = self.inst
        i.base_note = self.base.value()
        i.loop_mode = self.loop_mode.currentText()
        i.loop_start = self.loop_start.value()
        i.loop_end = self.loop_end.value()
        i.env_attack_ms = self.atk.value()
        i.env_decay_ms = self.dec.value()
        i.env_sustain = self.sus.value()
        i.env_release_ms = self.rel.value()
        i.pan = self.pan_slider.value() / 100.0


class _KeymapDialog(QDialog):
    """Verteilt Samples ueber die Klaviatur (Multisample / Drumkit). Jede Zone
    = ein Sample fuer einen Tastenbereich [Lo, Hi], unverschoben bei Root."""

    def __init__(self, name: str, zones, load_fn, parent=None):
        super().__init__(parent)
        from .tracker.instrument import Zone
        self._Zone = Zone
        self._load_fn = load_fn
        self.setWindowTitle("Keymap-Instrument")
        self.resize(560, 380)
        v = QVBoxLayout(self)

        nrow = QHBoxLayout()
        nrow.addWidget(QLabel("Name:"))
        self.name_edit = QComboBox(); self.name_edit.setEditable(True)
        self.name_edit.setCurrentText(name or "Keymap")
        nrow.addWidget(self.name_edit, 1)
        v.addLayout(nrow)

        self.table = QTableWidget(0, 4)
        self.table.setHorizontalHeaderLabels(["Sample", "Lo", "Hi", "Root"])
        self.table.horizontalHeader().setSectionResizeMode(
            QHeaderView.ResizeMode.Stretch)
        v.addWidget(self.table, 1)
        # vorhandene Zonen uebernehmen (Kopien der Sample-Arrays)
        self._zones = [self._Zone(
            samples=np.array(z.samples, copy=True), sample_rate=z.sample_rate,
            root_note=z.root_note, lo_key=z.lo_key, hi_key=z.hi_key,
            loop_mode=z.loop_mode, loop_start=z.loop_start,
            loop_end=z.loop_end, name=z.name) for z in (zones or [])]
        self._rebuild_table()

        brow = QHBoxLayout()
        b_add = QPushButton("Sample hinzufuegen...")
        b_add.clicked.connect(self._add_sample)
        brow.addWidget(b_add)
        b_del = QPushButton("Zone entfernen")
        b_del.clicked.connect(self._remove_zone)
        brow.addWidget(b_del)
        b_drum = QPushButton("Auto-Drumkit (ab C2)")
        b_drum.setToolTip("Jedem Sample EINE Taste zuweisen (ab MIDI 36)")
        b_drum.clicked.connect(self._auto_drumkit)
        brow.addWidget(b_drum)
        brow.addStretch(1)
        v.addLayout(brow)

        box = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel)
        box.accepted.connect(self._commit_table)
        box.accepted.connect(self.accept)
        box.rejected.connect(self.reject)
        v.addWidget(box)

    def _rebuild_table(self) -> None:
        self.table.setRowCount(len(self._zones))
        for r, z in enumerate(self._zones):
            it = QTableWidgetItem(f"{z.name} ({z.samples.size})")
            it.setFlags(it.flags() & ~Qt.ItemFlag.ItemIsEditable)
            self.table.setItem(r, 0, it)
            for col, val in ((1, z.lo_key), (2, z.hi_key), (3, z.root_note)):
                sp = QSpinBox(); sp.setRange(0, 127); sp.setValue(int(val))
                self.table.setCellWidget(r, col, sp)

    def _commit_table(self) -> None:
        for r, z in enumerate(self._zones):
            z.lo_key = self.table.cellWidget(r, 1).value()
            z.hi_key = self.table.cellWidget(r, 2).value()
            z.root_note = self.table.cellWidget(r, 3).value()
            if z.hi_key < z.lo_key:
                z.hi_key = z.lo_key

    def _add_sample(self) -> None:
        path, _ = QFileDialog.getOpenFileName(
            self, "Sample fuer Zone laden", "",
            "Audio (*.wav *.ogg *.flac)")
        if not path:
            return
        res = self._load_fn(path)
        if res is None:
            QMessageBox.warning(self, "Fehler", "Datei nicht ladbar.")
            return
        samples, sr = res
        from pathlib import Path
        self._commit_table()                 # aktuelle Edits sichern
        self._zones.append(self._Zone(
            samples=np.asarray(samples, dtype=np.float32), sample_rate=sr,
            root_note=60, lo_key=0, hi_key=127,
            name=Path(path).stem or "zone"))
        self._rebuild_table()

    def _remove_zone(self) -> None:
        r = self.table.currentRow()
        if 0 <= r < len(self._zones):
            self._commit_table()
            del self._zones[r]
            self._rebuild_table()

    def _auto_drumkit(self) -> None:
        self._commit_table()
        for i, z in enumerate(self._zones):
            key = min(127, 36 + i)
            z.lo_key = z.hi_key = z.root_note = key
        self._rebuild_table()

    def get_name(self) -> str:
        return self.name_edit.currentText().strip() or "Keymap"

    def get_zones(self):
        return self._zones


class _Sf2PresetDialog(QDialog):
    """Waehlt ein Preset aus einer SoundFont-Datei (mit Suchfeld)."""

    def __init__(self, presets, parent=None):
        super().__init__(parent)
        self.setWindowTitle("SoundFont-Preset waehlen")
        self.resize(440, 500)
        self._presets = presets
        v = QVBoxLayout(self)
        self.search = QLineEdit()
        self.search.setPlaceholderText("Suchen (z. B. Piano, Strings, Bass)...")
        self.search.textChanged.connect(self._fill)
        v.addWidget(self.search)
        self.list = QListWidget()
        self.list.itemDoubleClicked.connect(lambda *_: self.accept())
        v.addWidget(self.list, 1)
        self._fill("")
        box = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel)
        box.accepted.connect(self.accept)
        box.rejected.connect(self.reject)
        v.addWidget(box)

    def _fill(self, flt: str) -> None:
        flt = (flt or "").lower()
        self.list.clear()
        for bank, prog, name in self._presets:
            label = f"{bank:03d}:{prog:03d}  {name}"
            if flt in label.lower():
                it = QListWidgetItem(label)
                it.setData(Qt.ItemDataRole.UserRole, (bank, prog))
                self.list.addItem(it)
        if self.list.count():
            self.list.setCurrentRow(0)

    def selected(self):
        it = self.list.currentItem()
        return it.data(Qt.ItemDataRole.UserRole) if it else None


class _Piano(QWidget):
    """Klickbare Klaviatur (2 Oktaven). Emittiert die MIDI-Note."""

    note_clicked = Signal(int)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.base_midi = 60            # C4
        self.setMinimumHeight(72)
        self.setMinimumWidth(420)

    def set_base(self, midi: int) -> None:
        self.base_midi = midi
        self.update()

    # weisse Tasten = Halbton-Offsets ohne Schwarze
    _WHITE = (0, 2, 4, 5, 7, 9, 11)
    _BLACK = {0: 1, 1: 3, 3: 6, 4: 8, 5: 10}   # weiss-index -> schwarz-offset

    def _white_count(self) -> int:
        return 14                       # 2 Oktaven

    def paintEvent(self, _e):  # noqa: N802
        p = QPainter(self)
        w = self.width()
        h = self.height()
        nwhite = self._white_count()
        kw = w / nwhite
        # weisse Tasten
        for i in range(nwhite):
            x = i * kw
            p.setPen(QPen(QColor(COLORS["border"])))
            p.setBrush(QColor("#EEEEEE"))
            p.drawRect(int(x), 0, int(kw) - 1, h - 1)
        # schwarze Tasten
        for i in range(nwhite):
            octi = i % 7
            if octi in self._BLACK:
                x = (i + 1) * kw - kw * 0.3
                p.setBrush(QColor("#202020"))
                p.setPen(Qt.PenStyle.NoPen)
                p.drawRect(int(x), 0, int(kw * 0.6), int(h * 0.6))

    def _midi_at(self, px: float, py: float) -> int:
        nwhite = self._white_count()
        kw = self.width() / nwhite
        wi = int(px / kw)
        wi = max(0, min(nwhite - 1, wi))
        octave = wi // 7
        # schwarze Taste getroffen? (oberer Bereich)
        if py < self.height() * 0.6:
            octi = wi % 7
            # pruefe ob im rechten Rand-Bereich eine schwarze Taste liegt
            frac = (px - wi * kw) / kw
            if octi in self._BLACK and frac > 0.7:
                return self.base_midi + octave * 12 + self._BLACK[octi]
            if octi > 0 and (octi - 1) in self._BLACK and frac < 0.3:
                return self.base_midi + octave * 12 + self._BLACK[octi - 1]
        return self.base_midi + octave * 12 + self._WHITE[wi % 7]

    def mousePressEvent(self, e):  # noqa: N802
        m = self._midi_at(e.position().x(), e.position().y())
        self.note_clicked.emit(m)


# Slide-Suffix-Farbe (Amber -- kein Theme-Key, bewusst warm gegen Cyan/Mint).
_SLIDE_COLOR = "#EF9F27"


class _CellDelegate(QStyledItemDelegate):
    """Zeichnet die Pattern-Zellen im Renoise-Stil: Beat-Zeilen hinterlegt,
    Wiedergabe-Reihe (playhead) betont, und der Zelltext farbcodiert nach
    Feld -- Note=eigene Kanalfarbe (Drum=Magenta, Note-Off=gedaempft),
    v=Mint, s=Amber, FX=Magenta."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.playhead = -1

    def paint(self, painter, option, index):  # noqa: N802
        row = index.row()
        rect = option.rect
        # --- Hintergrund: Auswahl > Playhead > Beat-Zeilen ---
        if option.state & QStyle.StateFlag.State_Selected:
            painter.fillRect(rect, QColor(COLORS["sel"]))
        elif row == self.playhead:
            painter.fillRect(rect, QColor(COLORS["accent_soft"]))
        elif row % 16 == 0:
            painter.fillRect(rect, QColor(COLORS["bg_panel"]))
        elif row % 4 == 0:
            painter.fillRect(rect, QColor(COLORS["bg_alt"]))

        text = (index.data() or "").strip()
        painter.save()
        painter.setFont(option.font)
        fm = painter.fontMetrics()
        if not text or text == "···":
            painter.setPen(QColor(COLORS["line_no"]))
            painter.drawText(rect, Qt.AlignmentFlag.AlignCenter, "···")
            painter.restore()
            return
        tokens = text.split()
        widths = [fm.horizontalAdvance(t + " ") for t in tokens]
        total = sum(widths) - (fm.horizontalAdvance(" ") if tokens else 0)
        cx = rect.x() + (rect.width() - total) / 2.0
        base = rect.y() + rect.height() / 2.0 + fm.ascent() / 2.0 - 1
        channel = index.column()
        for i, t in enumerate(tokens):
            painter.setPen(QColor(self._token_color(i, t, channel)))
            painter.drawText(int(cx), int(base), t)
            cx += widths[i]
        painter.restore()

    @staticmethod
    def _token_color(idx: int, tok: str, channel: int = 0) -> str:
        if idx == 0:                       # Note, Drum-Hit oder Note-Off
            if tok == "OFF":
                return COLORS["fg_muted"]
            return COLORS["danger"] if tok == "X" else _channel_color(channel)
        if tok.startswith("v"):
            return COLORS["success"]       # Lautstaerke
        if tok.startswith("s"):
            return _SLIDE_COLOR            # Slide
        if tok.startswith("i") and tok[1:].isdigit():
            return COLORS["info"]          # Per-Note-Instrument-Ueberschreiben
        return COLORS["danger"]            # Effekt (Arp/Vib/Ret/Off)


class TrackerEditor(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = Path(project_root)
        self.setWindowTitle("GameBasic Tracker")
        self.resize(1120, 840)
        self.song = Song()
        self.path: Path | None = None  # aktueller Speicherort (fuer Quick-Save)
        self.dirty = False
        self.cur = 0                   # aktueller Pattern-Index
        self._sound_cache: dict = {}
        self._mixer = Mixer()          # additiver Mixer (gamebasic.audio_preview)
        self._play_mode = None         # None | "pattern" | "song"
        self._play_row = 0
        self._play_order_pos = 0
        self._playhead = -1            # aktuell hervorgehobene Wiedergabe-Reihe
        self._block_clip = None        # Zellen-Liste aus block_copy() (Strg+C)

        central = QWidget()
        self.setCentralWidget(central)
        root = QVBoxLayout(central)
        root.setContentsMargins(10, 8, 10, 8)
        root.setSpacing(7)

        title = QLabel("♪  GameBasic Tracker")
        tf = QFont(); tf.setBold(True); tf.setPointSize(13)
        title.setFont(tf)
        title.setStyleSheet(f"color: {COLORS['accent']}; padding: 2px 0;")
        root.addWidget(title)

        # Instrument-Pool mit Presets befuellen + sinnvolle Kanal-Sounds.
        self._install_factory_presets()

        # --- Transport ---
        top = QHBoxLayout()
        self.btn_play = QPushButton("▶ Pattern")
        self.btn_play.setProperty("accent", True)
        self.btn_play.clicked.connect(lambda: self._toggle_play("pattern"))
        top.addWidget(self.btn_play)
        self.btn_song = QPushButton("▶ Song")
        self.btn_song.clicked.connect(lambda: self._toggle_play("song"))
        top.addWidget(self.btn_song)
        self.btn_stop = QPushButton("■ Stop")
        self.btn_stop.clicked.connect(self._stop_play)
        top.addWidget(self.btn_stop)
        top.addSpacing(12)
        top.addWidget(QLabel("Tempo (BPM):"))
        self.bpm = QSpinBox(); self.bpm.setRange(40, 300); self.bpm.setValue(self.song.bpm)
        self.bpm.valueChanged.connect(self._on_bpm)
        top.addWidget(self.bpm)
        top.addSpacing(12)
        top.addWidget(QLabel("Kanaele:"))
        self.channels_spin = QSpinBox()
        self.channels_spin.setRange(MIN_CHANNELS, MAX_CHANNELS)
        self.channels_spin.setValue(self.song.channels)
        self.channels_spin.setToolTip(
            "Kanalzahl des Songs (letzter Kanal bleibt immer Drum/Noise)")
        self.channels_spin.valueChanged.connect(self._on_channels_changed)
        top.addWidget(self.channels_spin)
        top.addStretch(1)
        b_wav = QPushButton("♪ Audio (WAV)...")
        b_wav.setToolTip("Song mit allen Instrumenten als WAV rendern "
                         "(im Spiel via PLAYMUSIC)")
        b_wav.clicked.connect(self._export_audio)
        top.addWidget(b_wav)
        b_code = QPushButton("GB-Code"); b_code.clicked.connect(self._export)
        top.addWidget(b_code)
        root.addLayout(top)

        # --- Linkes Instrument-Panel (Spur-Sounds + Bibliothek, Renoise-Stil) ---
        side = QVBoxLayout(); side.setSpacing(6)
        sl = QLabel("Spur-Sounds")
        sl.setStyleSheet(f"color: {COLORS['accent']}; font-weight: bold;")
        side.addWidget(sl)
        # Kanal-Streifen (Name/Mute/Solo/Sound-Dropdown/VU) leben in einem
        # eigenen scrollbaren Container -- `_rebuild_channel_strips()` baut
        # ihn neu auf (Kanalzahl ist jetzt variabel, siehe self.song.channels).
        self.strip_scroll = QScrollArea()
        self.strip_scroll.setWidgetResizable(True)
        self.strip_scroll.setFrameShape(QFrame.Shape.NoFrame)
        self.strip_container = QWidget()
        self.strip_layout = QVBoxLayout(self.strip_container)
        self.strip_layout.setContentsMargins(0, 0, 0, 0)
        self.strip_layout.setSpacing(6)
        self.strip_scroll.setWidget(self.strip_container)
        side.addWidget(self.strip_scroll, 1)
        self.sound_combos: list = []
        self.mute_btns: list = []
        self.solo_btns: list = []
        self.vol_sliders: list = []
        self.vol_labels: list = []
        self.vu_meters: list = []
        self._muted: list = []
        self._solo: list = []
        self.vu_level: list = []

        # VU-Decay: laeuft nur waehrend der Wiedergabe (40 ms).
        self._vu_timer = QTimer(self)
        self._vu_timer.setInterval(40)
        self._vu_timer.timeout.connect(self._vu_decay)

        _sep = QFrame(); _sep.setFrameShape(QFrame.Shape.HLine)
        _sep.setStyleSheet(f"color: {COLORS['border']};")
        side.addSpacing(4); side.addWidget(_sep); side.addSpacing(2)

        bl = QLabel("Bibliothek")
        bl.setStyleSheet(f"color: {COLORS['accent']}; font-weight: bold;")
        side.addWidget(bl)
        self.inst_combo = QComboBox()
        self.inst_combo.setMinimumWidth(150)
        side.addWidget(self.inst_combo)
        b_load = QPushButton("+ Sample (WAV)...")
        b_load.setToolTip("Eigene Aufnahme als Instrument laden")
        b_load.clicked.connect(self._load_sample)
        side.addWidget(b_load)
        b_keymap = QPushButton("+ Keymap...")
        b_keymap.setToolTip("Mehrere Samples ueber die Klaviatur verteilen "
                            "(Multisample / Drumkit)")
        b_keymap.clicked.connect(self._edit_keymap)
        side.addWidget(b_keymap)
        b_sf2 = QPushButton("+ SoundFont (.sf2)...")
        b_sf2.setToolTip("Echtes Instrument aus einer SoundFont-Datei laden "
                         "(General MIDI / Hersteller-Sounds)")
        b_sf2.clicked.connect(self._load_soundfont)
        side.addWidget(b_sf2)
        ibtns = QHBoxLayout()
        b_iedit = QPushButton("Bearbeiten...")
        b_iedit.setToolTip("Grundton, Loop-Punkte, ADSR-Huellkurve")
        b_iedit.clicked.connect(self._edit_instrument)
        ibtns.addWidget(b_iedit)
        b_idel = QPushButton("Loeschen")
        b_idel.clicked.connect(self._remove_instrument)
        ibtns.addWidget(b_idel)
        side.addLayout(ibtns)
        side.addStretch(1)
        side_w = QWidget(); side_w.setLayout(side); side_w.setFixedWidth(250)

        # --- Pattern-/Zellen-Steuerung (ueber dem Gitter) ---
        prow = QHBoxLayout()
        b_new = QPushButton("Neu"); b_new.clicked.connect(self._new_song)
        b_open = QPushButton("Oeffnen"); b_open.clicked.connect(self._open)
        b_save = QPushButton("Speichern"); b_save.clicked.connect(self._save)
        b_save_as = QPushButton("Speichern unter...")
        b_save_as.clicked.connect(self._save_as)
        for b in (b_new, b_open, b_save, b_save_as):
            prow.addWidget(b)
        prow.addSpacing(8)
        self.btn_undo = QPushButton("↶"); self.btn_undo.setFixedWidth(34)
        self.btn_undo.setToolTip("Rueckgaengig (Strg+Z)")
        self.btn_redo = QPushButton("↷"); self.btn_redo.setFixedWidth(34)
        self.btn_redo.setToolTip("Wiederholen (Strg+Y)")
        prow.addWidget(self.btn_undo)
        prow.addWidget(self.btn_redo)
        prow.addSpacing(16)
        prow.addWidget(QLabel("Pattern:"))
        self.pattern_combo = QComboBox()
        self.pattern_combo.currentIndexChanged.connect(self._on_pattern_select)
        prow.addWidget(self.pattern_combo)
        prow.addWidget(QLabel("Reihen:"))
        self.rows_spin = QSpinBox(); self.rows_spin.setRange(1, 64)
        self.rows_spin.valueChanged.connect(self._on_rows)
        prow.addWidget(self.rows_spin)
        prow.addWidget(QLabel("Vol:"))
        self.vol_spin = QSpinBox(); self.vol_spin.setRange(0, VOL_MAX)
        self.vol_spin.setSpecialValueText("–")   # 0 = Standard-Lautstaerke
        self.vol_spin.setToolTip(
            "Lautstaerke der gewaehlten Note (1..15, – = Standard)")
        self.vol_spin.valueChanged.connect(self._on_vol_changed)
        prow.addWidget(self.vol_spin)
        prow.addWidget(QLabel("Slide:"))
        self.slide_spin = QSpinBox()
        self.slide_spin.setRange(-SLIDE_MAX, SLIDE_MAX)
        self.slide_spin.setToolTip(
            "Pitch-Slide der gewaehlten Note in Halbtoenen ueber die Reihe "
            "(0 = kein Slide); nur Ton-Kanaele")
        self.slide_spin.valueChanged.connect(self._on_slide_changed)
        prow.addWidget(self.slide_spin)
        prow.addWidget(QLabel("FX:"))
        self.fx_combo = QComboBox()
        for code in FX_CODES:
            self.fx_combo.addItem(FX_NAMES[code], code)
        self.fx_combo.setToolTip(
            "Effekt der gewaehlten Note: Arp/Vib/Ret/Off (im WAV-Render).")
        self.fx_combo.currentIndexChanged.connect(self._on_fx_changed)
        prow.addWidget(self.fx_combo)
        self.fxp_spin = QSpinBox(); self.fxp_spin.setRange(0, 255)
        self.fxp_spin.setToolTip(
            "Effekt-Parameter (Byte). Arp/Vib: zwei Nibbles x|y "
            "(z.B. 71 = 0x47 -> +4/+7 HT); Ret: Ticks; Off: x*512 Frames.")
        self.fxp_spin.valueChanged.connect(self._on_fxp_changed)
        prow.addWidget(self.fxp_spin)
        prow.addWidget(QLabel("Instr:"))
        self.inst_cell_combo = QComboBox()
        self.inst_cell_combo.setMinimumWidth(130)
        self.inst_cell_combo.setToolTip(
            "Instrument NUR fuer diese Note (— = Kanal-Standard aus "
            "Spur-Sounds) -- wie echte Tracker: eine Note kann von der "
            "Spur-Zuweisung abweichen, z.B. ein anderer Drum-Hit im Fill.")
        self.inst_cell_combo.currentIndexChanged.connect(self._on_cell_inst_changed)
        prow.addWidget(self.inst_cell_combo)
        b_padd = QPushButton("+ Pattern"); b_padd.clicked.connect(self._add_pattern)
        b_pdup = QPushButton("Duplizieren"); b_pdup.clicked.connect(self._dup_pattern)
        b_pdel = QPushButton("Loeschen"); b_pdel.clicked.connect(self._del_pattern)
        b_clear = QPushButton("Leeren"); b_clear.clicked.connect(self._clear)
        for b in (b_padd, b_pdup, b_pdel, b_clear):
            prow.addWidget(b)
        prow.addStretch(1)

        # --- Pattern-Gitter ---
        self.grid = QTableWidget(self.song.patterns[0].rows, self.song.channels)
        self.grid.setHorizontalHeaderLabels(_channel_names(self.song.channels))
        self.grid.verticalHeader().setDefaultSectionSize(30)
        self.grid.verticalHeader().setSectionResizeMode(
            QHeaderView.ResizeMode.Fixed)
        self.grid.horizontalHeader().setSectionResizeMode(
            QHeaderView.ResizeMode.Stretch)
        self.grid.horizontalHeader().setFixedHeight(40)
        self.grid.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self.grid.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectItems)
        # ContiguousSelection statt SingleSelection: erlaubt rechteckige
        # Mehrfach-Auswahl (Shift-Klick/Ziehen) fuer Block-Copy/Transpose/
        # Interpolate, ohne die Einzelzell-Semantik (currentRow/currentColumn
        # via _sel()) zu aendern -- ein Einzelklick bleibt ein 1x1-Block.
        self.grid.setSelectionMode(QTableWidget.SelectionMode.ContiguousSelection)
        self.grid.setShowGrid(False)
        gfont = QFont(EDITOR_FONT_FAMILY, 13); gfont.setBold(True)
        self.grid.setFont(gfont)
        self.grid.setStyleSheet(_TRACKER_GRID_QSS)
        self._delegate = _CellDelegate(self.grid)
        self.grid.setItemDelegate(self._delegate)
        self.grid.itemSelectionChanged.connect(self._audition_selected)
        self.grid.itemSelectionChanged.connect(self._sync_vol_spin)

        # Panel links, Steuerung + Gitter rechts (Renoise-Layout).
        rightcol = QVBoxLayout(); rightcol.setSpacing(6)
        rightcol.addLayout(prow)
        rightcol.addWidget(self.grid, 1)
        rightw = QWidget(); rightw.setLayout(rightcol)
        main = QHBoxLayout(); main.setSpacing(10)
        main.addWidget(side_w)
        main.addWidget(rightw, 1)
        root.addLayout(main, 1)

        # --- Song-Arrangement (Order) ---
        arow = QHBoxLayout()
        arow.addWidget(QLabel("Song:"))
        self.order_list = QListWidget()
        self.order_list.setFlow(QListWidget.Flow.LeftToRight)
        self.order_list.setFixedHeight(40)
        self.order_list.setWrapping(False)
        self.order_list.itemDoubleClicked.connect(self._order_jump)
        arow.addWidget(self.order_list, 1)
        b_oadd = QPushButton("+ akt."); b_oadd.clicked.connect(self._order_add)
        b_odel = QPushButton("entf."); b_odel.clicked.connect(self._order_remove)
        b_ol = QPushButton("◀"); b_ol.setFixedWidth(32)
        b_ol.clicked.connect(lambda: self._order_move(-1))
        b_or = QPushButton("▶"); b_or.setFixedWidth(32)
        b_or.clicked.connect(lambda: self._order_move(1))
        for b in (b_oadd, b_odel, b_ol, b_or):
            arow.addWidget(b)
        root.addLayout(arow)

        # --- Klaviatur ---
        krow = QHBoxLayout()
        krow.addWidget(QLabel("Oktave:"))
        self.octave = QSpinBox(); self.octave.setRange(2, 6); self.octave.setValue(4)
        self.octave.valueChanged.connect(
            lambda v: self.piano.set_base(12 * (v + 1)))
        krow.addWidget(self.octave)
        krow.addSpacing(8)
        b_off = QPushButton("◼ Note Aus")
        b_off.setToolTip(
            "Note-Off in die gewaehlte Zelle setzen (Taste 0) -- schneidet "
            "eine klingende Note VOR der naechsten Note ab, statt sie bis "
            "dahin durchklingen zu lassen")
        b_off.clicked.connect(self._set_note_off)
        krow.addWidget(b_off)
        krow.addWidget(QLabel(
            "  (Zelle waehlen, dann Taste klicken; Entf loescht -- Block: "
            "Shift-Klick/Ziehen, Strg+C/X/V Kopieren/Schneiden/Einfuegen, "
            "Strg+Pfeil (+Shift=Oktave) Transponieren, Strg+I Interpolieren)"))
        krow.addStretch(1)
        root.addLayout(krow)
        self.piano = _Piano()
        self.piano.set_base(12 * (self.octave.value() + 1))
        self.piano.note_clicked.connect(self._on_piano)
        root.addWidget(self.piano)

        self._timer = QTimer(self)
        self._timer.timeout.connect(self._tick)

        self._reload_all()

        # Undo/Redo ueber Snapshots des Song-Modells (to_dict/from_dict).
        # capture als Lambda (self.song wird bei Restore neu zugewiesen) +
        # deepcopy, weil Pattern.to_dict() die Live-`data`-Liste referenziert
        # -- ohne Kopie wuerde der Snapshot mit dem Modell mutieren.
        import copy as _copy
        self.undo = SnapshotUndo(
            lambda: _copy.deepcopy(self.song.to_dict()), self._restore_song,
            debounce_ms=1)
        self.undo.changed.connect(self._update_undo_buttons)
        self.undo.changed.connect(self._mark_dirty)
        self.btn_undo.clicked.connect(self.undo.undo)
        self.btn_redo.clicked.connect(self.undo.redo)
        QShortcut(QKeySequence.StandardKey.Undo, self, activated=self.undo.undo)
        QShortcut(QKeySequence.StandardKey.Redo, self, activated=self.undo.redo)
        QShortcut(QKeySequence("Ctrl+Y"), self, activated=self.undo.redo)
        QShortcut(QKeySequence.StandardKey.Save, self, activated=self._save)
        QShortcut(QKeySequence("Ctrl+Shift+S"), self, activated=self._save_as)
        self._update_undo_buttons()
        self._update_title()

    def _mark(self) -> None:
        u = getattr(self, "undo", None)
        if u is not None:
            u.mark()

    def _update_undo_buttons(self) -> None:
        self.btn_undo.setEnabled(self.undo.can_undo())
        self.btn_redo.setEnabled(self.undo.can_redo())

    # ---- Ungespeicherte-Aenderungen-Schutz ----
    def _mark_dirty(self) -> None:
        # undo.changed feuert auch bei Undo/Redo (nicht nur bei neuen
        # Aenderungen) -- fuer Dirty-Zwecke reicht das (gleiches simples
        # Bool-Flag-Muster wie bei den anderen Begleit-Editoren, keine
        # "zurueck auf exakt den gespeicherten Stand"-Erkennung).
        self.dirty = True
        self._update_title()

    def _update_title(self) -> None:
        base = f"GameBasic Tracker -- {self.path.name}" if self.path else "GameBasic Tracker"
        self.setWindowTitle(base + ("*" if self.dirty else ""))

    def _confirm_dirty(self) -> bool:
        """Fragt bei ungespeicherten Aenderungen nach (Speichern/Verwerfen/
        Abbrechen) -- liefert True, wenn fortgefahren werden darf."""
        if not self.dirty:
            return True
        ans = QMessageBox.question(
            self, "Aenderungen speichern?",
            "Das Tracker-Projekt hat ungespeicherte Aenderungen.\n"
            "Vor dem Fortfahren speichern?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
            | QMessageBox.StandardButton.Cancel,
            QMessageBox.StandardButton.Cancel)
        if ans == QMessageBox.StandardButton.Cancel:
            return False
        if ans == QMessageBox.StandardButton.Yes:
            return self._save()        # False, wenn der Speichern-Dialog abgebrochen wurde
        return True

    def closeEvent(self, event) -> None:  # noqa: N802
        if self._confirm_dirty():
            event.accept()
        else:
            event.ignore()

    def _restore_song(self, snap: dict) -> None:
        """Setzt das Song-Modell aus einem to_dict()-Snapshot (fuer Undo)."""
        self.song = Song.from_dict(snap)
        self._sound_cache.clear()
        self.cur = min(self.cur, len(self.song.patterns) - 1)
        self._reload_all()

    # ============================================== Song/Pattern-Sync
    def _reload_all(self) -> None:
        """Komplettes UI aus self.song neu aufbauen."""
        self.bpm.blockSignals(True); self.bpm.setValue(self.song.bpm); self.bpm.blockSignals(False)
        self.channels_spin.blockSignals(True)
        self.channels_spin.setValue(self.song.channels)
        self.channels_spin.blockSignals(False)
        self._rebuild_channel_strips()
        self.cur = min(self.cur, len(self.song.patterns) - 1)
        self._refresh_instruments()
        self._reload_pattern_combo()
        self._reload_order()
        self._load_pattern(self.cur)

    def _on_channels_changed(self, v: int) -> None:
        if v == self.song.channels:
            return
        self.song.set_channels(v)
        self._sound_cache.clear()
        self._reload_all()
        self._mark()

    def _make_channel_strip(self, c: int, name: str) -> QWidget:
        """Baut EINEN Kanal-Streifen (Name/Mute/Solo/Sound-Dropdown/Lautstaerke-
        Regler/VU) und haengt seine Widgets an die sound_combos/mute_btns/
        solo_btns/vol_sliders/vol_labels/vu_meters-Listen an (Reihenfolge =
        Kanal-Index). Jeder Kanal bekommt seine eigene Akzentfarbe
        (`_channel_color`), damit sich viele Kanaele optisch unterscheiden."""
        col = _channel_color(c)
        w = QWidget()
        v = QVBoxLayout(w); v.setContentsMargins(0, 0, 0, 0); v.setSpacing(2)
        r = QHBoxLayout()
        cl = QLabel(name); cl.setFixedWidth(34)
        cl.setStyleSheet(f"color: {col}; font-weight: bold;")
        r.addWidget(cl)
        mb = QPushButton("M"); mb.setCheckable(True); mb.setFixedWidth(24)
        mb.setToolTip("Spur stummschalten (nur Vorhoeren)")
        mb.toggled.connect(lambda on, ch=c: self._on_mute(ch, on))
        r.addWidget(mb)
        sb = QPushButton("S"); sb.setCheckable(True); sb.setFixedWidth(24)
        sb.setToolTip("Solo -- nur Solo-Spuren klingen (nur Vorhoeren)")
        sb.toggled.connect(lambda on, ch=c: self._on_solo(ch, on))
        r.addWidget(sb)
        cb = QComboBox(); cb.setMinimumWidth(120)
        cb.currentIndexChanged.connect(
            lambda idx, ch=c: self._on_sound_changed(ch, idx))
        r.addWidget(cb, 1)
        v.addLayout(r)
        # Mixer-Lautstaerke-Regler (echter Schieberegler statt Spinbox/Combo --
        # wie der Kanal-Fader in Renoise/FastTracker/OpenMPT). Wirkt im
        # Vorhoeren, WAV-Render UND GB-Code-Export (Song.channel_vol).
        vr = QHBoxLayout(); vr.setSpacing(4)
        vol_slider = QSlider(Qt.Orientation.Horizontal)
        vol_slider.setRange(0, 100)
        vol0 = self.song.channel_vol[c] if c < len(self.song.channel_vol) else 1.0
        vol_slider.setValue(round(vol0 * 100))
        vol_slider.setFixedHeight(14)
        vol_slider.setToolTip("Kanal-Lautstaerke (Mixer-Fader)")
        vol_slider.setStyleSheet(_slider_qss(col))
        vol_slider.valueChanged.connect(
            lambda val, ch=c: self._on_channel_vol_changed(ch, val))
        vr.addWidget(vol_slider, 1)
        vol_label = QLabel(f"{round(vol0 * 100)}%"); vol_label.setFixedWidth(32)
        vol_label.setStyleSheet(f"color: {COLORS['fg_muted']}; font-size: 9px;")
        vr.addWidget(vol_label)
        v.addLayout(vr)
        vu = QProgressBar(); vu.setRange(0, 100); vu.setValue(0)
        vu.setTextVisible(False); vu.setFixedHeight(5)
        vu.setStyleSheet(
            f"QProgressBar {{ border: none; background: {COLORS['bg_alt']}; "
            f"border-radius: 2px; }} "
            f"QProgressBar::chunk {{ background: {col}; border-radius: 2px; }}")
        v.addWidget(vu)
        self.sound_combos.append(cb)
        self.mute_btns.append(mb)
        self.solo_btns.append(sb)
        self.vol_sliders.append(vol_slider)
        self.vol_labels.append(vol_label)
        self.vu_meters.append(vu)
        return w

    def _on_channel_vol_changed(self, c: int, val: int) -> None:
        if c < len(self.song.channel_vol):
            self.song.channel_vol[c] = val / 100.0
        if c < len(self.vol_labels):
            self.vol_labels[c].setText(f"{val}%")
        self._sound_cache.clear()
        self._mark()

    def _rebuild_channel_strips(self) -> None:
        """Baut die Spur-Sounds-Streifen komplett aus `self.song.channels`
        neu auf -- noetig beim Laden/Erzeugen eines Songs UND nach einer
        Kanalzahl-Aenderung (`_on_channels_changed`)."""
        while self.strip_layout.count():
            item = self.strip_layout.takeAt(0)
            w = item.widget()
            if w is not None:
                w.deleteLater()
        self.sound_combos = []
        self.mute_btns = []
        self.solo_btns = []
        self.vol_sliders = []
        self.vol_labels = []
        self.vu_meters = []
        n = self.song.channels
        self._muted = [False] * n
        self._solo = [False] * n
        self.vu_level = [0.0] * n
        for c, name in enumerate(_channel_names(n)):
            self.strip_layout.addWidget(self._make_channel_strip(c, name))
        self.strip_layout.addStretch(1)
        self._rebuild_sound_combos()

    def _reload_pattern_combo(self) -> None:
        self.pattern_combo.blockSignals(True)
        self.pattern_combo.clear()
        for i, p in enumerate(self.song.patterns):
            self.pattern_combo.addItem(f"{i}: {p.name}")
        self.pattern_combo.setCurrentIndex(self.cur)
        self.pattern_combo.blockSignals(False)

    def _load_pattern(self, idx: int) -> None:
        """Gitter + Reihen-Spinbox aus Pattern `idx` fuellen."""
        self.cur = idx
        pat = self.song.patterns[idx]
        self.rows_spin.blockSignals(True); self.rows_spin.setValue(pat.rows); self.rows_spin.blockSignals(False)
        self.grid.blockSignals(True)
        self.grid.setRowCount(pat.rows)
        self.grid.setColumnCount(pat.channels)
        self.grid.setVerticalHeaderLabels([f"{r:02d}" for r in range(pat.rows)])
        for r in range(pat.rows):
            for c in range(pat.channels):
                fx, fxp = pat.get_fx(c, r)
                it = QTableWidgetItem(
                    self._cell_text(c, pat.data[c][r], pat.vol[c][r],
                                    pat.slide[c][r], fx, fxp, pat.get_inst(c, r)))
                it.setTextAlignment(Qt.AlignmentFlag.AlignCenter)
                self.grid.setItem(r, c, it)
                self._style_cell(it, c, r, pat.data[c][r])
        self.grid.blockSignals(False)
        self._update_channel_headers()
        self._sync_vol_spin()

    # Visuelles (Beat-Zeilen, Playhead, Farb-Kodierung) uebernimmt jetzt der
    # _CellDelegate beim Zeichnen -- diese Methode bleibt als No-op-Hook fuer
    # die bestehenden Aufrufer (Zell-Refresh/Pattern-Laden).
    def _style_cell(self, item, c: int, r: int, note) -> None:
        return

    def _update_channel_headers(self) -> None:
        """Spalten-Header zeigen Kanal + zugewiesenes Instrument/Wellenform,
        in der jeweiligen Kanalfarbe (`_channel_color`) statt einheitlich."""
        names = _channel_names(self.song.channels)
        for c in range(self.song.channels):
            idx = self.song.channel_inst[c]
            if idx is not None and 0 <= idx < len(self.song.instruments):
                sub = self.song.instruments[idx].name
            elif c == self.song.tonal:
                sub = "noise"
            else:
                sub = self.song.waves[c]
            item = QTableWidgetItem(f"{names[c]}\n{sub}")
            item.setForeground(QColor(_channel_color(c)))
            self.grid.setHorizontalHeaderItem(c, item)

    def _reload_order(self) -> None:
        self.order_list.clear()
        for p in self.song.order:
            name = self.song.patterns[p].name if p < len(self.song.patterns) else "?"
            self.order_list.addItem(QListWidgetItem(f"{p}:{name}"))

    # ============================================== Bearbeiten
    def _on_bpm(self, v: int) -> None:
        self.song.bpm = v
        self._mark()

    def _set_wave(self, ci: int, v: str) -> None:
        self.song.waves[ci] = v
        self._sound_cache.clear()
        self._update_channel_headers()
        self._mark()

    # ============================================== Instrumente
    def _refresh_instruments(self) -> None:
        self.inst_combo.blockSignals(True)
        self.inst_combo.clear()
        for i, ins in enumerate(self.song.instruments):
            tag = {"sample": "♪", "keymap": "▦"}.get(ins.kind, "~")
            self.inst_combo.addItem(f"{i}: {tag} {ins.name}")
        self.inst_combo.blockSignals(False)
        self._rebuild_sound_combos()
        self._rebuild_inst_cell_combo()
        if hasattr(self, "grid"):
            self._update_channel_headers()

    def _rebuild_inst_cell_combo(self) -> None:
        """Fuellt das "Instr:"-Dropdown (per-Note-Instrument-Ueberschreiben)
        aus dem Instrument-Pool -- "—" (Index None) + alle Instrumente."""
        cb = self.inst_cell_combo
        cb.blockSignals(True)
        cb.clear()
        cb.addItem("—", None)
        for i, ins in enumerate(self.song.instruments):
            tag = {"sample": "♪", "keymap": "▦"}.get(ins.kind, "~")
            cb.addItem(f"{i}: {tag} {ins.name}", i)
        cb.blockSignals(False)

    def _rebuild_sound_combos(self) -> None:
        """Pro-Spur-Sound-Dropdowns aus dem Instrument-Pool fuellen + die
        aktuelle Zuweisung spiegeln."""
        for c in range(len(self.sound_combos)):
            cb = self.sound_combos[c]
            cb.blockSignals(True)
            cb.clear()
            for ins in self.song.instruments:
                cb.addItem(ins.name)
            idx = self.song.channel_inst[c]
            if idx is not None and 0 <= idx < cb.count():
                cb.setCurrentIndex(idx)
            cb.blockSignals(False)

    def _on_sound_changed(self, c: int, idx: int) -> None:
        if 0 <= idx < len(self.song.instruments) and 0 <= c < self.song.channels:
            self.song.channel_inst[c] = idx
            self._sound_cache.clear()
            self._update_channel_headers()
            self._mark()

    def _install_factory_presets(self) -> None:
        """Befuellt einen leeren Pool mit den Instrument-Presets und weist
        jedem Kanal einen sinnvollen Default-Sound zu."""
        if self.song.instruments:
            return
        from .tracker.presets import factory_instruments
        self.song.instruments = factory_instruments()
        by_name = {ins.name: i for i, ins in enumerate(self.song.instruments)}
        defaults = ["Fluegel (Piano)", "Streicher", "Synth-Bass", "Kick"]
        for c, nm in enumerate(defaults):
            if c < self.song.channels and nm in by_name:
                self.song.channel_inst[c] = by_name[nm]

    def _load_sample(self) -> None:
        path, _ = QFileDialog.getOpenFileName(
            self, "Sample laden (WAV/OGG)", str(self.project_root),
            "Audio (*.wav *.ogg *.flac)")
        if not path:
            return
        inst = self._instrument_from_file(path)
        if inst is None:
            QMessageBox.warning(self, "Fehler",
                                "Audiodatei konnte nicht geladen werden.")
            return
        self.song.add_instrument(inst)
        self._sound_cache.clear()
        self._refresh_instruments()
        self.inst_combo.setCurrentIndex(len(self.song.instruments) - 1)
        self._mark()

    def _load_samples_from_file(self, path: str):
        """(samples_float, sample_rate) aus einer Audiodatei -- erst stdlib-WAV
        (headless), sonst soundfile-Dekodierung (OGG/FLAC/float-WAV/...). None
        bei Fehler."""
        from .tracker.instrument import load_wav_mono
        try:
            return load_wav_mono(path)
        except Exception:
            pass
        try:
            import soundfile as sf
            data, sr = sf.read(path, dtype="float32", always_2d=False)
            if data.ndim == 2:
                data = data.mean(axis=1)
            return data.astype(np.float32), int(sr)
        except Exception:
            return None

    def _instrument_from_file(self, path: str):
        """Laedt eine Audiodatei als (Einzel-)Sample-Instrument."""
        from .tracker.instrument import Instrument
        from pathlib import Path
        res = self._load_samples_from_file(path)
        if res is None:
            return None
        samples, sr = res
        return Instrument.from_array(Path(path).stem or "Sample", samples, sr, 60)

    def _remove_instrument(self) -> None:
        idx = self.inst_combo.currentIndex()
        if 0 <= idx < len(self.song.instruments):
            self.song.remove_instrument(idx)
            self._sound_cache.clear()
            self._refresh_instruments()
            self._mark()

    def _load_soundfont(self) -> None:
        """Laedt ein echtes Instrument aus einer SoundFont-(.sf2)-Datei
        (General MIDI / Hersteller-Sounds) als Keymap-Instrument."""
        path, _ = QFileDialog.getOpenFileName(
            self, "SoundFont laden (.sf2)", str(self.project_root),
            "SoundFont (*.sf2)")
        if not path:
            return
        try:
            from .tracker.sf2 import SoundFont
            sf = SoundFont(path)
            presets = sf.presets()
        except Exception as exc:  # noqa: BLE001
            QMessageBox.warning(self, "Fehler",
                                f"SoundFont nicht lesbar:\n{exc}")
            return
        if not presets:
            QMessageBox.information(self, "Leer", "Keine Presets gefunden.")
            return
        dlg = _Sf2PresetDialog(presets, self)
        if dlg.exec() != QDialog.DialogCode.Accepted:
            return
        sel = dlg.selected()
        if sel is None:
            return
        bank, prog = sel
        try:
            QApplication.setOverrideCursor(Qt.CursorShape.WaitCursor)
            try:
                inst = sf.build_instrument(bank, prog)
            finally:
                QApplication.restoreOverrideCursor()
        except Exception as exc:  # noqa: BLE001
            QMessageBox.warning(self, "Fehler",
                                f"Preset nicht ladbar:\n{exc}")
            return
        if not inst.zones:
            QMessageBox.information(
                self, "Leer", "Dieses Preset hat keine spielbaren Zonen.")
            return
        self.song.add_instrument(inst)
        self._sound_cache.clear()
        self._refresh_instruments()
        self.inst_combo.setCurrentIndex(len(self.song.instruments) - 1)
        self._mark()

    def _edit_keymap(self) -> None:
        """Erstellt/bearbeitet ein Keymap-Instrument (Samples ueber die Tasten
        verteilt). Ist das gewaehlte Instrument bereits ein Keymap, wird es
        bearbeitet; sonst entsteht ein neues."""
        from .tracker.instrument import Instrument
        idx = self.inst_combo.currentIndex()
        editing = (0 <= idx < len(self.song.instruments)
                   and self.song.instruments[idx].kind == "keymap")
        cur = self.song.instruments[idx] if editing else None
        dlg = _KeymapDialog(cur.name if cur else "Keymap",
                            cur.zones if cur else [],
                            self._load_samples_from_file, self)
        if dlg.exec() != QDialog.DialogCode.Accepted:
            return
        zones = dlg.get_zones()
        if not zones:
            QMessageBox.information(self, "Keymap leer",
                                    "Mindestens ein Sample hinzufuegen.")
            return
        inst = Instrument.keymap(dlg.get_name(), zones)
        if editing:
            # ADSR/default_vol vom alten Instrument uebernehmen
            inst.default_vol = cur.default_vol
            inst.env_attack_ms = cur.env_attack_ms
            inst.env_decay_ms = cur.env_decay_ms
            inst.env_sustain = cur.env_sustain
            inst.env_release_ms = cur.env_release_ms
            self.song.instruments[idx] = inst
        else:
            self.song.add_instrument(inst)
        self._sound_cache.clear()
        self._refresh_instruments()
        self._mark()

    def _edit_instrument(self) -> None:
        idx = self.inst_combo.currentIndex()
        if not (0 <= idx < len(self.song.instruments)):
            return
        inst = self.song.instruments[idx]
        if inst.kind != "sample":
            QMessageBox.information(
                self, "Synth-Instrument",
                "Grundton/Loop/ADSR gibt es nur fuer Sample-Instrumente.")
            return
        dlg = _InstrumentDialog(inst, self)
        if dlg.exec() == QDialog.DialogCode.Accepted:
            dlg.apply_to()
            self._sound_cache.clear()
            self._mark()

    def _cell_text(self, ci: int, note, vol=None, slide=None,
                   fx=FX_NONE, fxp=0, inst=None) -> str:
        if note is None:
            return "···"
        if note == NOTE_OFF:
            return " OFF"          # traegt nie vol/slide/fx/inst (Pattern.set())
        base = "  X" if ci == self.song.tonal else note_name(note)
        if vol:
            base += f" v{vol}"
        if slide:
            base += f" s{slide:+d}"
        if fx and fx != FX_NONE:
            base += f" {FX_NAMES.get(fx, '?')}{fxp:02X}"
        if inst is not None:
            base += f" i{inst}"
        return base

    def _cell_refresh(self, row: int, ci: int) -> None:
        pat = self.song.patterns[self.cur]
        it = self.grid.item(row, ci)
        fx, fxp = pat.get_fx(ci, row)
        it.setText(self._cell_text(ci, pat.data[ci][row], pat.vol[ci][row],
                                   pat.slide[ci][row], fx, fxp,
                                   pat.get_inst(ci, row)))
        self._style_cell(it, ci, row, pat.data[ci][row])

    def _set_note(self, row: int, ci: int, note) -> None:
        pat = self.song.patterns[self.cur]
        pat.set(ci, row, note)               # loescht ggf. auch die Effekte
        self._cell_refresh(row, ci)
        self._sync_vol_spin()
        self._mark()

    def _on_vol_changed(self, v: int) -> None:
        if not self._has_sel():
            return
        r, c = self._sel()
        pat = self.song.patterns[self.cur]
        if pat.data[c][r] is None:           # nur Noten haben Lautstaerke
            return
        pat.set_vol(c, r, v)                 # 0 -> None (Standard)
        self._cell_refresh(r, c)
        self._mark()

    def _on_slide_changed(self, s: int) -> None:
        if not self._has_sel():
            return
        r, c = self._sel()
        pat = self.song.patterns[self.cur]
        if pat.data[c][r] is None or c == self.song.tonal:   # nur tonale Noten
            return
        pat.set_slide(c, r, s)               # 0 -> None
        self._cell_refresh(r, c)
        self._mark()

    def _on_fx_changed(self, _idx: int) -> None:
        if not self._has_sel():
            return
        r, c = self._sel()
        pat = self.song.patterns[self.cur]
        if pat.data[c][r] is None:           # nur Noten tragen Effekte
            return
        code = self.fx_combo.currentData()
        pat.set_fx(c, r, code, self.fxp_spin.value())
        self._cell_refresh(r, c)
        self._mark()

    def _on_fxp_changed(self, _v: int) -> None:
        if not self._has_sel():
            return
        r, c = self._sel()
        pat = self.song.patterns[self.cur]
        code = self.fx_combo.currentData()
        if pat.data[c][r] is None or code == FX_NONE:
            return
        pat.set_fx(c, r, code, self.fxp_spin.value())
        self._cell_refresh(r, c)
        self._mark()

    def _on_cell_inst_changed(self, _idx: int) -> None:
        if not self._has_sel():
            return
        r, c = self._sel()
        pat = self.song.patterns[self.cur]
        if pat.data[c][r] is None:
            return
        pat.set_inst(c, r, self.inst_cell_combo.currentData())
        self._cell_refresh(r, c)
        self._sound_cache.clear()
        self._mark()

    def _sync_vol_spin(self) -> None:
        """Spiegelt die Effekte der gewaehlten Zelle in die Spinboxen."""
        self.vol_spin.blockSignals(True)
        self.slide_spin.blockSignals(True)
        self.fx_combo.blockSignals(True)
        self.fxp_spin.blockSignals(True)
        self.inst_cell_combo.blockSignals(True)
        if self._has_sel():
            r, c = self._sel()
            pat = self.song.patterns[self.cur]
            self.vol_spin.setValue(pat.vol[c][r] or 0)
            self.slide_spin.setValue(pat.slide[c][r] or 0)
            self.slide_spin.setEnabled(c != self.song.tonal)
            fx, fxp = pat.get_fx(c, r)
            self.fx_combo.setCurrentIndex(
                FX_CODES.index(fx) if fx in FX_CODES else 0)
            self.fxp_spin.setValue(fxp)
            found = self.inst_cell_combo.findData(pat.get_inst(c, r))
            self.inst_cell_combo.setCurrentIndex(found if found >= 0 else 0)
        else:
            self.vol_spin.setValue(0)
            self.slide_spin.setValue(0)
            self.fx_combo.setCurrentIndex(0)
            self.fxp_spin.setValue(0)
            self.inst_cell_combo.setCurrentIndex(0)
        self.vol_spin.blockSignals(False)
        self.slide_spin.blockSignals(False)
        self.fx_combo.blockSignals(False)
        self.fxp_spin.blockSignals(False)
        self.inst_cell_combo.blockSignals(False)

    def _on_piano(self, midi: int) -> None:
        ch = self._sel_channel() if self._has_sel() else 0
        self._play_note(ch, midi)
        if self._has_sel():
            r, c = self._sel()
            self._set_note(r, c, midi)
            if r + 1 < self.song.patterns[self.cur].rows:
                self.grid.setCurrentCell(r + 1, c)

    def _set_note_off(self) -> None:
        """Setzt Note-Off in die gewaehlte Zelle -- schneidet eine klingende
        Note VOR der naechsten Note im Kanal ab (klassisches Tracker-
        Konzept). Traegt nie Lautstaerke/Slide/Effekt (Pattern.set())."""
        if not self._has_sel():
            return
        r, c = self._sel()
        self._set_note(r, c, NOTE_OFF)
        if r + 1 < self.song.patterns[self.cur].rows:
            self.grid.setCurrentCell(r + 1, c)

    def _has_sel(self) -> bool:
        return self.grid.currentRow() >= 0 and self.grid.currentColumn() >= 0

    def _sel(self):
        return self.grid.currentRow(), self.grid.currentColumn()

    def _sel_channel(self) -> int:
        return self.grid.currentColumn()

    def _audition_selected(self) -> None:
        if self._has_sel():
            r, c = self._sel()
            pat = self.song.patterns[self.cur]
            n = pat.data[c][r]
            if n is not None and n != NOTE_OFF:
                inst = self.song.instrument_for_cell(pat, c, r)
                self._play_note(c, n, pat.vol[c][r], pat.slide[c][r] or 0, inst=inst)

    # ---- Block-Auswahl (Copy/Cut/Paste/Transpose/Interpolate) ----
    def _selection_rect(self):
        """(c0, r0, c1, r1) des aktuell markierten Rechtecks (inklusiv),
        oder None ohne Auswahl. `ContiguousSelection` liefert bei einem
        einzelnen Klick ein 1x1-Rechteck -- Einzelzell-Operationen bleiben
        also ein Spezialfall dieser Block-Rechtecke."""
        ranges = self.grid.selectedRanges()
        if ranges:
            rg = ranges[0]
            return rg.leftColumn(), rg.topRow(), rg.rightColumn(), rg.bottomRow()
        if self._has_sel():
            r, c = self._sel()
            return c, r, c, r
        return None

    def _reload_and_select(self, c0: int, r0: int, c1: int, r1: int) -> None:
        """Laedt das Gitter neu (nach einer Block-Op) und stellt die
        Rechteck-Auswahl wieder her -- sonst wuerde z.B. wiederholtes
        Strg+Pfeil-Transponieren nach dem ersten Tastendruck auf eine
        Einzelzelle zurueckfallen."""
        self._load_pattern(self.cur)
        top, bottom = sorted((r0, r1))
        left, right = sorted((c0, c1))
        self.grid.setRangeSelected(
            QTableWidgetSelectionRange(top, left, bottom, right), True)
        # NoUpdate: setCurrentCell() wuerde in ContiguousSelection sonst die
        # gerade gesetzte Rechteck-Auswahl auf die eine Zelle zusammenklappen.
        self.grid.setCurrentCell(top, left, QItemSelectionModel.SelectionFlag.NoUpdate)

    def _block_clear(self, c0: int, r0: int, c1: int, r1: int) -> None:
        pat = self.song.patterns[self.cur]
        for c in range(min(c0, c1), max(c0, c1) + 1):
            for r in range(min(r0, r1), max(r0, r1) + 1):
                pat.set(c, r, None)

    def _block_copy(self) -> None:
        rect = self._selection_rect()
        if rect is None:
            return
        c0, r0, c1, r1 = rect
        self._block_clip = block_copy(self.song.patterns[self.cur], c0, r0, c1, r1)

    def _block_cut(self) -> None:
        rect = self._selection_rect()
        if rect is None:
            return
        self._block_copy()
        c0, r0, c1, r1 = rect
        self._block_clear(c0, r0, c1, r1)
        self._reload_and_select(c0, r0, c1, r1)
        self._mark()

    def _block_paste(self) -> None:
        if not self._block_clip:
            return
        rect = self._selection_rect()
        c0, r0 = (rect[0], rect[1]) if rect else (0, 0)
        pat = self.song.patterns[self.cur]
        n_c, n_r = block_paste(pat, self._block_clip, c0, r0)
        if n_c and n_r:
            self._reload_and_select(c0, r0, c0 + n_c - 1, r0 + n_r - 1)
            self._mark()

    def _block_transpose(self, semitones: int) -> None:
        rect = self._selection_rect()
        if rect is None:
            return
        c0, r0, c1, r1 = rect
        block_transpose(self.song.patterns[self.cur], c0, r0, c1, r1, semitones,
                        skip_channel=self.song.tonal)
        self._reload_and_select(c0, r0, c1, r1)
        self._mark()

    def _block_interpolate(self) -> None:
        rect = self._selection_rect()
        if rect is None:
            return
        c0, r0, c1, r1 = rect
        block_interpolate(self.song.patterns[self.cur], c0, r0, c1, r1,
                          skip_channel=self.song.tonal)
        self._reload_and_select(c0, r0, c1, r1)
        self._mark()

    def keyPressEvent(self, e):  # noqa: N802
        mods = e.modifiers()
        ctrl = bool(mods & Qt.KeyboardModifier.ControlModifier)
        if ctrl and e.key() == Qt.Key.Key_C:
            self._block_copy(); return
        if ctrl and e.key() == Qt.Key.Key_X:
            self._block_cut(); return
        if ctrl and e.key() == Qt.Key.Key_V:
            self._block_paste(); return
        if ctrl and e.key() == Qt.Key.Key_I:
            self._block_interpolate(); return
        if ctrl and e.key() == Qt.Key.Key_Up:
            step = 12 if mods & Qt.KeyboardModifier.ShiftModifier else 1
            self._block_transpose(step); return
        if ctrl and e.key() == Qt.Key.Key_Down:
            step = 12 if mods & Qt.KeyboardModifier.ShiftModifier else 1
            self._block_transpose(-step); return
        if e.key() in (Qt.Key.Key_Delete, Qt.Key.Key_Backspace):
            rect = self._selection_rect()
            if rect is not None:
                self._block_clear(*rect)
                self._reload_and_select(*rect)
                self._mark()
            return
        if not ctrl and e.key() == Qt.Key.Key_0:
            self._set_note_off(); return
        super().keyPressEvent(e)

    def _clear(self) -> None:
        self.song.patterns[self.cur].clear()
        self._load_pattern(self.cur)
        self._mark()

    # ============================================== Pattern-Verwaltung
    def _on_pattern_select(self, idx: int) -> None:
        if 0 <= idx < len(self.song.patterns):
            self._load_pattern(idx)

    def _on_rows(self, v: int) -> None:
        self.song.patterns[self.cur].set_rows(v)
        self._load_pattern(self.cur)
        self._mark()

    def _add_pattern(self) -> None:
        idx = self.song.add_pattern(rows=self.song.patterns[self.cur].rows)
        self.cur = idx
        self._reload_pattern_combo()
        self._load_pattern(idx)
        self._mark()

    def _dup_pattern(self) -> None:
        idx = self.song.duplicate_pattern(self.cur)
        self.cur = idx
        self._reload_pattern_combo()
        self._load_pattern(idx)
        self._mark()

    def _del_pattern(self) -> None:
        if len(self.song.patterns) <= 1:
            return
        self.song.remove_pattern(self.cur)
        self.cur = min(self.cur, len(self.song.patterns) - 1)
        self._reload_pattern_combo()
        self._reload_order()
        self._load_pattern(self.cur)
        self._mark()

    # ============================================== Order/Arrangement
    def _order_add(self) -> None:
        self.song.order_add(self.cur)
        self._reload_order()
        self._mark()

    def _order_remove(self) -> None:
        pos = self.order_list.currentRow()
        if pos >= 0:
            self.song.order_remove(pos)
            self._reload_order()
            self._mark()

    def _order_move(self, delta: int) -> None:
        pos = self.order_list.currentRow()
        if pos >= 0:
            new = self.song.order_move(pos, delta)
            self._reload_order()
            self.order_list.setCurrentRow(new)
            self._mark()

    def _order_jump(self, item: QListWidgetItem) -> None:
        pos = self.order_list.row(item)
        if 0 <= pos < len(self.song.order):
            idx = self.song.order[pos]
            self.cur = idx
            self._reload_pattern_combo()
            self._load_pattern(idx)

    # ============================================== Sound
    def _row_samples(self) -> int:
        return max(1, int(44100 * self.song.row_ms() / 1000.0))

    def _render_sound(self, inst, midi: int, n_samples: int, slide: int = 0):
        """Float-Sample-Array der Note ueber das Instrument (gecacht). n_samples
        = Klanglaenge -> Noten klingen fuer ihre Dauer. 0.6 vorgemischt (Headroom),
        auf [-1, 1] geclippt; Wiedergabe via sounddevice in `_play_array`."""
        key = (id(inst), int(midi), int(n_samples), int(slide or 0))
        arr = self._sound_cache.get(key)
        if arr is None:
            wave = inst.render_note(midi, n_samples, 44100, slide or 0)
            arr = np.clip(wave, -1.0, 1.0).astype(np.float32) * 0.6
            self._sound_cache[key] = arr
        return arr

    def _play_array(self, arr: np.ndarray, sr: int = 44100, vol: int | None = None):
        """Spielt ein float32-Array ueber den additiven Mixer (best effort --
        still ohne Audio-Geraet/Lib). vol = Lautstaerke 1..15 wird beim
        Abspielen eingemischt. `sr` wird nicht mehr durchgereicht -- der Mixer
        laeuft an einer festen Samplerate (44100, wie `_render_sound`)."""
        out = arr
        if vol:
            out = arr * min(1.0, max(0.0, vol_to_pct(vol) / 100.0))
        self._mixer.play(out)

    def _play_note(self, ci: int, midi: int, vol: int | None = None,
                   slide: int = 0, n_rows: int | None = None, inst=None) -> None:
        """Spielt eine Note. `n_rows` = Notenlaenge in Reihen (None = kurze
        Vorhoer-Laenge); so klingen Noten waehrend der Wiedergabe so lange,
        bis die naechste Note kommt. `inst` = explizit aufgeloestes Instrument
        (per-Note-Ueberschreiben via `Song.instrument_for_cell`) -- ohne
        Vorgabe faellt es auf den Kanal-Standard zurueck (z.B. beim reinen
        Klaviatur-Vorhoeren ohne platzierte Note)."""
        inst = inst if inst is not None else self.song.instrument_for_channel(ci)
        row_s = self._row_samples()
        if n_rows is None:
            n = max(int(44100 * 0.6), row_s * 2)      # Vorhoeren
        else:
            n = max(row_s, row_s * int(n_rows))
        arr = self._render_sound(inst, midi, n, slide or 0)
        if arr is not None:
            chvol = (self.song.channel_vol[ci]
                     if 0 <= ci < len(self.song.channel_vol) else 1.0)
            self._play_array(arr * chvol, 44100, vol)
            # VU-Pegel der Spur setzen (Peak x Lautstaerke x Kanal-Fader).
            if arr.size and 0 <= ci < len(self.vu_level):
                vf = (vol_to_pct(vol) / 100.0) if vol else 1.0
                peak = float(np.max(np.abs(arr))) * vf * chvol
                self.vu_level[ci] = max(self.vu_level[ci], min(1.0, peak))
                self.vu_meters[ci].setValue(int(self.vu_level[ci] * 100))

    # ============================================== Playback
    def _toggle_play(self, mode: str) -> None:
        if self._timer.isActive():
            self._stop_play()
            if self._play_mode == mode:
                self._play_mode = None
                return
        self._play_mode = mode
        self._play_row = 0
        self._play_order_pos = 0
        if mode == "song" and self.song.order:
            self._load_pattern(self.song.order[0])
            self._reload_pattern_combo()
        self._timer.setInterval(self.song.row_ms())
        self._timer.start()
        self._vu_timer.start()
        self.btn_play.setText("■ Stop" if mode == "pattern" else "▶ Pattern")
        self.btn_song.setText("■ Stop" if mode == "song" else "▶ Song")

    def _stop_play(self) -> None:
        self._timer.stop()
        self._vu_timer.stop()
        self._reset_vu()
        self._set_playhead(-1)         # Playhead-Highlight entfernen
        self.btn_play.setText("▶ Pattern")
        self.btn_song.setText("▶ Song")

    def _tick(self) -> None:
        if self._play_mode == "pattern":
            pat = self.song.patterns[self.cur]
            self._play_row %= pat.rows
            self._set_playhead(self._play_row)
            self._play_columns(pat, self._play_row)
            self._play_row = (self._play_row + 1) % pat.rows
        elif self._play_mode == "song":
            order = self.song.order or [0]
            self._play_order_pos %= len(order)
            p_idx = order[self._play_order_pos]
            if p_idx != self.cur:
                self.cur = p_idx
                self._reload_pattern_combo()
                self._load_pattern(p_idx)
            pat = self.song.patterns[p_idx]
            self._play_row %= pat.rows
            self._set_playhead(self._play_row)
            self._play_columns(pat, self._play_row)
            self._play_row += 1
            if self._play_row >= pat.rows:
                self._play_row = 0
                self._play_order_pos = (self._play_order_pos + 1) % len(order)

    def _set_playhead(self, row: int) -> None:
        """Hebt die laufende Wiedergabe-Reihe hervor (ohne die User-Auswahl zu
        veraendern). row < 0 = Highlight entfernen. Das Zeichnen uebernimmt der
        Zell-Delegate -- hier nur den Wert setzen + neu zeichnen."""
        self._playhead = row
        self._delegate.playhead = row
        self.grid.viewport().update()
        if 0 <= row < self.grid.rowCount():
            self.grid.scrollToItem(self.grid.item(row, 0))

    @staticmethod
    def _note_len_rows(pat, c: int, r: int) -> int:
        """Reihen, bis die naechste Note auf Kanal `c` kommt (sonst bis zum
        Pattern-Ende) -> Klanglaenge dieser Note."""
        for rr in range(r + 1, pat.rows):
            if pat.data[c][rr] is not None:
                return rr - r
        return pat.rows - r

    # ---- Mute / Solo (nur Live-Vorhoeren; WAV-Render rendert alle Spuren) ----
    def _on_mute(self, c: int, on: bool) -> None:
        self._muted[c] = bool(on)

    def _on_solo(self, c: int, on: bool) -> None:
        self._solo[c] = bool(on)

    def _audible(self, c: int) -> bool:
        if self._muted[c]:
            return False
        if any(self._solo):
            return self._solo[c]
        return True

    def _vu_decay(self) -> None:
        """Laesst die VU-Meter sanft abklingen (waehrend der Wiedergabe)."""
        for c in range(len(self.vu_level)):
            self.vu_level[c] *= 0.82
            self.vu_meters[c].setValue(int(self.vu_level[c] * 100))

    def _reset_vu(self) -> None:
        for c in range(len(self.vu_level)):
            self.vu_level[c] = 0.0
            self.vu_meters[c].setValue(0)

    def _play_columns(self, pat, row: int) -> None:
        for c in range(pat.channels):
            n = pat.data[c][row]
            if n is not None and n != NOTE_OFF and self._audible(c):
                inst = self.song.instrument_for_cell(pat, c, row)
                self._play_note(c, n, pat.vol[c][row], pat.slide[c][row] or 0,
                                self._note_len_rows(pat, c, row), inst=inst)

    # ============================================== Datei
    def _new_song(self) -> None:
        if not self._confirm_dirty():
            return
        self.song = Song()
        self.path = None
        self._install_factory_presets()
        self.cur = 0
        self._reload_all()
        self.undo.reset()      # frisches Dokument -> Historie verwerfen
        self.dirty = False
        self._update_title()

    def _open(self) -> None:
        if not self._confirm_dirty():
            return
        path, _ = QFileDialog.getOpenFileName(
            self, "Tracker-Projekt oeffnen", str(self.project_root),
            "Tracker-Projekt (*.json)")
        if not path:
            return
        try:
            self.song = Song.load_json(path)
        except Exception as exc:  # noqa: BLE001
            QMessageBox.warning(self, "Fehler", f"Konnte nicht laden:\n{exc}")
            return
        self.path = Path(path)
        self.cur = 0
        self._sound_cache.clear()
        self._install_factory_presets()    # alte Songs ohne Instrumente
        self._reload_all()
        self.undo.reset()      # geladenes Dokument -> Historie verwerfen
        self.dirty = False
        self._update_title()

    def _save(self) -> bool:
        """Quick-Save: schreibt auf `self.path`, wenn schon bekannt (aus
        Oeffnen/vorigem Speichern) -- sonst wie `_save_as()` ein Dialog.
        Frueher IMMER ein Dialog, auch beim wiederholten Strg+S auf eine
        bereits benannte Datei."""
        if self.path is None:
            return self._save_as()
        try:
            self.song.save_json(str(self.path))
        except Exception as exc:  # noqa: BLE001
            QMessageBox.warning(self, "Fehler", f"Konnte nicht speichern:\n{exc}")
            return False
        self.dirty = False
        self._update_title()
        return True

    def _save_as(self) -> bool:
        default = str(self.path) if self.path else str(self.project_root)
        path, _ = QFileDialog.getSaveFileName(
            self, "Tracker-Projekt speichern", default,
            "Tracker-Projekt (*.json)")
        if not path:
            return False
        if not path.lower().endswith(".json"):
            path += ".json"
        self.path = Path(path)
        return self._save()

    # ============================================== Export
    def _export(self) -> None:
        self._show_code(self.song.gb_code())

    def _export_audio(self) -> None:
        """Rendert den ganzen Song (inkl. Sample-Instrumente/Loop/ADSR/Pitch-
        Slide) offline zu einer WAV-Datei -- im Spiel via PLAYMUSIC/LOADSOUND
        abspielbar. Optional Stereo (Instrument-Pan) + Amiga-Hard-Panning."""
        stereo, hard_pan = self._ask_render_options()
        if stereo is None:
            return
        path, _ = QFileDialog.getSaveFileName(
            self, "Song als Audio rendern", str(self.project_root),
            "WAV-Audio (*.wav)")
        if not path:
            return
        if not path.lower().endswith(".wav"):
            path += ".wav"
        try:
            from .tracker.mixer import render_song, save_wav
            QApplication.setOverrideCursor(Qt.CursorShape.WaitCursor)
            try:
                mix = render_song(self.song, stereo=stereo, hard_pan=hard_pan)
                save_wav(path, mix)
            finally:
                QApplication.restoreOverrideCursor()
        except Exception as exc:  # noqa: BLE001
            QMessageBox.warning(self, "Fehler",
                                f"Audio-Export fehlgeschlagen:\n{exc}")
            return
        secs = mix.shape[0] / 44100.0
        kind = ("Stereo" + (" / Amiga-Hard-Pan" if hard_pan else "")) if stereo else "Mono"
        QMessageBox.information(
            self, "Audio exportiert",
            f"{Path(path).name} ({secs:.1f}s, {kind}) gerendert.\n\n"
            f"Im Spiel abspielen:  PLAYMUSIC(\"{Path(path).name}\")")

    def _ask_render_options(self):
        """Kleiner Dialog vor dem WAV-Render: Stereo + Amiga-Hard-Pan.
        Liefert (stereo, hard_pan) oder (None, None) bei Abbruch."""
        dlg = QDialog(self)
        dlg.setWindowTitle("Audio-Render-Optionen")
        lay = QVBoxLayout(dlg)
        cb_stereo = QCheckBox("Stereo (Instrument-Pan auswerten)")
        cb_stereo.setChecked(True)
        cb_hard = QCheckBox("Amiga-Hard-Panning (Kanal 1+4 links, 2+3 rechts)")
        cb_hard.setChecked(False)
        cb_hard.setEnabled(True)
        cb_stereo.toggled.connect(cb_hard.setEnabled)
        lay.addWidget(cb_stereo)
        lay.addWidget(cb_hard)
        bb = QDialogButtonBox(QDialogButtonBox.StandardButton.Ok
                              | QDialogButtonBox.StandardButton.Cancel)
        bb.accepted.connect(dlg.accept)
        bb.rejected.connect(dlg.reject)
        lay.addWidget(bb)
        if dlg.exec() != QDialog.DialogCode.Accepted:
            return None, None
        stereo = cb_stereo.isChecked()
        return stereo, (cb_hard.isChecked() and stereo)

    def _show_code(self, code: str) -> None:
        dlg = QFrame(self, Qt.WindowType.Window)
        dlg.setWindowTitle("GB-Code (Tracker)")
        dlg.resize(620, 520)
        dl = QVBoxLayout(dlg)
        edit = QPlainTextEdit(); edit.setPlainText(code); edit.setReadOnly(True)
        edit.setFont(QFont(EDITOR_FONT_FAMILY, 10))
        dl.addWidget(edit)
        row = QHBoxLayout(); row.addStretch(1)
        b = QPushButton("In Zwischenablage"); b.setProperty("accent", True)
        b.clicked.connect(lambda: QApplication.clipboard().setText(code))
        row.addWidget(b)
        dl.addLayout(row)
        dlg.show()
        self._code_dlg = dlg


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance()
    if app is None:
        # Fusion-Style NUR bei frischer QApplication erzwingen (nicht auf
        # einer schon laufenden Instanz -- z.B. im Test-Prozess, wo bereits
        # viele Widgets anderer Fenster existieren; ein Style-Wechsel dort
        # riskiert Qt-interne Abstuerze). Der native "windowsvista"-Style
        # haelt sich fuer Chrome/Buttons/Fensterhintergrund nur teilweise an
        # QSS -- ohne Fusion sieht der Editor trotz global_qss() spartanisch/
        # grau aus (nur Widgets mit eigenem lokalem setStyleSheet(), z.B. das
        # Pattern-Gitter, waeren dann korrekt dunkel). Gleiches Muster wie
        # editor_qt/__init__.py:_ensure_app.
        app = QApplication([])
        app.setStyle("Fusion")
    app.setStyleSheet(global_qss())
    win = TrackerEditor(project_root)
    if initial_file and Path(initial_file).exists():
        try:
            win.song = Song.load_json(str(initial_file))
            win.path = Path(initial_file)
            win.cur = 0
            win._reload_all()
            win.undo.reset()
            win.dirty = False
            win._update_title()
        except Exception:
            pass
    win.show()
    return app.exec()
