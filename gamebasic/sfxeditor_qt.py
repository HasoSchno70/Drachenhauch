"""SFX-Generator fuer GameBasic (`gbsfx` / `gbrun.py --sfx`).

sfxr-Stil-Tool fuer Retro-Soundeffekte: eigener Synthesizer (Waveform +
Pitch-Slide + Hüllkurve + Vibrato), Live-Wellenform-Vorschau, Abspielen, und
Export als **WAV** (per `LOADSOUND` ladbar) oder -- bei einfachen Toenen --
als GB-Code (`AUDIO_TONE`/`AUDIO_NOISE`).

Der Synth ist bewusst eigenstaendig: das `audio`-Modul kann nur konstante
Toene (`AUDIO_TONE`) -- fuer Sweeps/Huellkurven brauchen wir mehr. Das
Ergebnis wird als WAV-Asset exportiert.
"""
from __future__ import annotations

import wave
from pathlib import Path

import numpy as np
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QFont, QKeySequence, QPainter, QPen, QShortcut
from PySide6.QtWidgets import (
    QApplication, QComboBox, QDoubleSpinBox, QFileDialog, QFrame, QGroupBox,
    QHBoxLayout, QLabel, QMainWindow, QPlainTextEdit, QPushButton, QSpinBox,
    QVBoxLayout, QWidget,
)

from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .editor_qt.undo_history import SnapshotUndo
from .editor_qt.preset_bar import PresetBar
from .editor_qt.preset_library import PresetLibrary, default_dir
from .synth import SAMPLE_RATE as _SAMPLE_RATE, WAVEFORMS as _WAVEFORMS
from .synth import synthesize as _synth

# Presets: typische sfxr-Kategorien. Werte sind ein Startpunkt, "Zufall"
# variiert sie. (waveform, base_freq, slide_hz_s, atk, sus, dec, vib_d, vib_s)
_PRESETS = {
    "Pickup/Coin":  ("square", 900,   600,  0,  40, 160, 0.0, 0),
    "Laser/Shoot":  ("saw",    1000, -1400, 0,  30, 150, 0.0, 0),
    "Explosion":    ("noise",  120,  -60,   0,  60, 420, 0.0, 0),
    "Powerup":      ("square", 380,   700,  0,  90, 240, 0.15, 18),
    "Hit/Hurt":     ("square", 500,  -500,  0,  20, 140, 0.0, 0),
    "Jump":         ("square", 420,   560,  0,  50, 130, 0.0, 0),
    "Blip/Select":  ("square", 820,   0,    0,  18, 70,  0.0, 0),
}


def synthesize(p: dict, sr: int = _SAMPLE_RATE) -> np.ndarray:
    """Float-Sample-Array [-1, 1] inkl. Lautstaerke -- nutzt den geteilten
    Synth (`gamebasic.synth`), denselben Code wie der AUDIO_SFX-Builtin.
    Bei stereo_width > 0 ist die Rueckgabe `(n, 2)`."""
    wave = _synth(p["waveform"], p["base_freq"], p["slide"],
                  p["attack"], p["sustain"], p["decay"],
                  p["vib_depth"], p["vib_speed"], p.get("stereo_width", 0.0),
                  sr=sr,
                  duty=p.get("duty", 0.5),
                  pwm_depth=p.get("pwm_depth", 0.0),
                  pwm_speed=p.get("pwm_speed", 0.0),
                  flt_cutoff=p.get("flt_cutoff", 0.0),
                  flt_sweep=p.get("flt_sweep", 0.0),
                  flt_res=p.get("flt_res", 0.0))
    return np.clip(wave * p["volume"], -1.0, 1.0)


def save_wav(path: Path, samples: np.ndarray, sr: int = _SAMPLE_RATE) -> None:
    channels = 2 if samples.ndim == 2 else 1
    int16 = (np.clip(samples, -1.0, 1.0) * 32767.0).astype("<i2")
    with wave.open(str(path), "wb") as w:
        w.setnchannels(channels)
        w.setsampwidth(2)
        w.setframerate(sr)
        w.writeframes(np.ascontiguousarray(int16).tobytes())


def play(samples: np.ndarray, sr: int = _SAMPLE_RATE) -> None:
    """Spielt die Samples ueber sounddevice (best effort -- ohne Audio-Geraet
    oder ohne installiertes sounddevice still). Mono ODER Stereo `(n, 2)`.
    sounddevice nimmt die float32-Arrays direkt (keine int16-Konvertierung noetig)."""
    try:
        import sounddevice as sd
        arr = np.ascontiguousarray(
            np.clip(samples, -1.0, 1.0).astype(np.float32))
        sd.play(arr, sr)
    except Exception:
        pass


class _WaveView(QWidget):
    """Zeichnet die aktuelle Wellenform."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setMinimumHeight(140)
        self._samples = np.zeros(1)

    def set_samples(self, s: np.ndarray) -> None:
        # Bei Stereo den linken Kanal zeichnen.
        self._samples = s[:, 0] if s.ndim == 2 else s
        self.update()

    def paintEvent(self, _e):  # noqa: N802
        p = QPainter(self)
        p.fillRect(self.rect(), QColor(COLORS["bg"]))
        w = self.width()
        h = self.height()
        mid = h / 2.0
        p.setPen(QPen(QColor(COLORS["border"]), 1))
        p.drawLine(0, int(mid), w, int(mid))
        s = self._samples
        n = s.shape[0]
        if n < 2 or w < 2:
            return
        p.setPen(QPen(QColor(COLORS["accent"]), 1))
        step = max(1, n // w)
        prev_x = 0
        prev_y = mid - float(s[0]) * mid * 0.95
        for x in range(1, w):
            idx = min(n - 1, int(x / w * n))
            # Peak im Fenster fuer dichtere Wellen.
            seg = s[idx:min(n, idx + step)]
            v = seg[np.argmax(np.abs(seg))] if seg.size else s[idx]
            y = mid - float(v) * mid * 0.95
            p.drawLine(prev_x, int(prev_y), x, int(y))
            prev_x, prev_y = x, y


class SfxGenerator(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = project_root
        self.setWindowTitle("GameBasic SFX-Generator")
        self.resize(720, 640)
        self._counter = 0

        central = QWidget()
        self.setCentralWidget(central)
        root = QVBoxLayout(central)

        title = QLabel("SFX-Generator")
        tf = QFont(); tf.setBold(True); tf.setPointSize(13)
        title.setFont(tf)
        root.addWidget(title)

        # Presets (Werks-Schnellzugriff)
        pre = QHBoxLayout()
        pre.addWidget(QLabel("Preset:"))
        for name in _PRESETS:
            b = QPushButton(name)
            b.clicked.connect(lambda _=False, n=name: self._load_preset(n))
            pre.addWidget(b)
        pre.addStretch(1)
        root.addLayout(pre)

        # Preset-Bibliothek (eigene Sounds speichern/laden)
        self.presets = PresetLibrary(default_dir() / "sfx.json")
        self.preset_bar = PresetBar(
            self.presets, self._params, self._apply_params)
        root.addWidget(self.preset_bar)

        # Wellenform-Vorschau
        self.wave_view = _WaveView()
        root.addWidget(self.wave_view)

        # Parameter
        params = QHBoxLayout()
        col1 = QGroupBox("Ton")
        c1 = QVBoxLayout(col1)
        self.waveform = QComboBox(); self.waveform.addItems(_WAVEFORMS)
        self.waveform.currentTextChanged.connect(self._on_change)
        self._row(c1, "Waveform", self.waveform)
        self.base_freq = self._ispin(c1, "Frequenz (Hz)", 50, 8000, 800)
        self.slide = self._ispin(c1, "Pitch-Slide (Hz/s)", -8000, 8000, 0)
        self.volume = self._dspin(c1, "Lautstaerke", 0.0, 1.0, 0.7, 0.05)
        params.addWidget(col1)

        col2 = QGroupBox("Huellkurve && Vibrato")
        c2 = QVBoxLayout(col2)
        self.attack = self._ispin(c2, "Attack (ms)", 0, 2000, 0)
        self.sustain = self._ispin(c2, "Sustain (ms)", 0, 4000, 40)
        self.decay = self._ispin(c2, "Decay (ms)", 0, 4000, 150)
        self.vib_depth = self._dspin(c2, "Vibrato-Tiefe", 0.0, 0.5, 0.0, 0.01)
        self.vib_speed = self._ispin(c2, "Vibrato-Speed (Hz)", 0, 60, 0)
        self.stereo_width = self._dspin(c2, "Stereo-Breite", 0.0, 1.0, 0.0, 0.05)
        self.pan = self._dspin(c2, "Pan (L -1 .. +1 R)", -1.0, 1.0, 0.0, 0.1)
        params.addWidget(col2)

        # SID-Stil: Pulsbreite/PWM (nur square) + resonanter Tiefpass-Sweep.
        col3 = QGroupBox("SID / Filter")
        c3 = QVBoxLayout(col3)
        self.duty = self._dspin(c3, "Pulsbreite (square)", 0.05, 0.95, 0.5, 0.05)
        self.pwm_depth = self._dspin(c3, "PWM-Tiefe", 0.0, 0.45, 0.0, 0.01)
        self.pwm_speed = self._dspin(c3, "PWM-Speed (Hz)", 0.0, 30.0, 0.0, 0.5)
        self.flt_cutoff = self._ispin(c3, "Filter-Cutoff (Hz, 0=aus)", 0, 12000, 0)
        self.flt_sweep = self._ispin(c3, "Filter-Sweep (Hz/s)", -12000, 12000, 0)
        self.flt_res = self._dspin(c3, "Resonanz", 0.0, 0.95, 0.0, 0.05)
        params.addWidget(col3)
        root.addLayout(params)

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
        b_play = QPushButton("▶ Abspielen")
        b_play.setProperty("accent", True)
        b_play.clicked.connect(self._play)
        btns.addWidget(b_play)
        b_rand = QPushButton("Zufall")
        b_rand.clicked.connect(self._randomize)
        btns.addWidget(b_rand)
        btns.addStretch(1)
        b_wav = QPushButton("WAV exportieren ...")
        b_wav.clicked.connect(self._export_wav)
        btns.addWidget(b_wav)
        b_code = QPushButton("GB-Code")
        b_code.clicked.connect(self._export_code)
        btns.addWidget(b_code)
        root.addLayout(btns)
        root.addStretch(1)

        self._load_preset("Jump")

        # Undo/Redo: Snapshot der Parameter (self._params <-> _apply_params).
        self.undo = SnapshotUndo(self._params, self._apply_params, debounce_ms=250)
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

    def _apply_params(self, p: dict) -> None:
        """Setzt alle Parameter-Widgets aus einem `_params()`-Dict (fuer Undo)."""
        self.waveform.setCurrentText(p["waveform"])
        self.base_freq.setValue(int(p["base_freq"]))
        self.slide.setValue(int(p["slide"]))
        self.volume.setValue(float(p["volume"]))
        self.attack.setValue(int(p["attack"]))
        self.sustain.setValue(int(p["sustain"]))
        self.decay.setValue(int(p["decay"]))
        self.vib_depth.setValue(float(p["vib_depth"]))
        self.vib_speed.setValue(int(p["vib_speed"]))
        self.stereo_width.setValue(float(p["stereo_width"]))
        self.pan.setValue(float(p["pan"]))
        self.duty.setValue(float(p.get("duty", 0.5)))
        self.pwm_depth.setValue(float(p.get("pwm_depth", 0.0)))
        self.pwm_speed.setValue(float(p.get("pwm_speed", 0.0)))
        self.flt_cutoff.setValue(int(p.get("flt_cutoff", 0)))
        self.flt_sweep.setValue(int(p.get("flt_sweep", 0)))
        self.flt_res.setValue(float(p.get("flt_res", 0.0)))
        self._on_change()

    # ---- Helfer
    def _row(self, layout, label, widget):
        r = QHBoxLayout()
        lab = QLabel(label); lab.setFixedWidth(140)
        r.addWidget(lab); r.addWidget(widget, 1)
        layout.addLayout(r)

    def _ispin(self, layout, label, lo, hi, val):
        sp = QSpinBox(); sp.setRange(lo, hi); sp.setValue(int(val))
        sp.valueChanged.connect(self._on_change)
        self._row(layout, label, sp)
        return sp

    def _dspin(self, layout, label, lo, hi, val, step):
        sp = QDoubleSpinBox(); sp.setRange(lo, hi); sp.setSingleStep(step)
        sp.setValue(val)
        sp.valueChanged.connect(self._on_change)
        self._row(layout, label, sp)
        return sp

    # ---- Parameter <-> UI
    def _params(self) -> dict:
        return {
            "waveform": self.waveform.currentText(),
            "base_freq": float(self.base_freq.value()),
            "slide": float(self.slide.value()),
            "volume": float(self.volume.value()),
            "attack": int(self.attack.value()),
            "sustain": int(self.sustain.value()),
            "decay": int(self.decay.value()),
            "vib_depth": float(self.vib_depth.value()),
            "vib_speed": float(self.vib_speed.value()),
            "stereo_width": float(self.stereo_width.value()),
            "pan": float(self.pan.value()),
            "duty": float(self.duty.value()),
            "pwm_depth": float(self.pwm_depth.value()),
            "pwm_speed": float(self.pwm_speed.value()),
            "flt_cutoff": float(self.flt_cutoff.value()),
            "flt_sweep": float(self.flt_sweep.value()),
            "flt_res": float(self.flt_res.value()),
        }

    def _on_change(self, *_a) -> None:
        self.wave_view.set_samples(synthesize(self._params()))
        u = getattr(self, "undo", None)
        if u is not None:
            u.mark()

    def _load_preset(self, name: str) -> None:
        wf, base, slide, atk, sus, dec, vd, vs = _PRESETS[name]
        self.waveform.setCurrentText(wf)
        self.base_freq.setValue(base)
        self.slide.setValue(slide)
        self.attack.setValue(atk)
        self.sustain.setValue(sus)
        self.decay.setValue(dec)
        self.vib_depth.setValue(vd)
        self.vib_speed.setValue(vs)
        self.stereo_width.setValue(0.0)   # Presets sind mono/zentriert
        self.pan.setValue(0.0)
        self.duty.setValue(0.5)           # SID-Parameter auf neutral
        self.pwm_depth.setValue(0.0)
        self.pwm_speed.setValue(0.0)
        self.flt_cutoff.setValue(0)
        self.flt_sweep.setValue(0)
        self.flt_res.setValue(0.0)
        self._on_change()
        self._play()

    def _randomize(self) -> None:
        import random
        self.waveform.setCurrentText(random.choice(_WAVEFORMS))
        self.base_freq.setValue(random.randint(150, 1600))
        self.slide.setValue(random.randint(-1600, 1600))
        self.attack.setValue(random.choice([0, 0, 0, 20, 60]))
        self.sustain.setValue(random.randint(10, 120))
        self.decay.setValue(random.randint(60, 400))
        self.vib_depth.setValue(round(random.choice([0, 0, 0.1, 0.2]), 2))
        self.vib_speed.setValue(random.choice([0, 0, 12, 24]))
        self.stereo_width.setValue(round(random.choice([0, 0, 0.3, 0.6]), 2))
        self._on_change()
        self._play()

    def _play(self) -> None:
        play(synthesize(self._params()))

    # ---- Export
    def _export_wav(self) -> None:
        path, _ = QFileDialog.getSaveFileName(
            self, "WAV speichern",
            str(self.project_root / "assets" / "sfx.wav"),
            "WAV-Dateien (*.wav)")
        if not path:
            return
        if not path.lower().endswith(".wav"):
            path += ".wav"
        save_wav(Path(path), synthesize(self._params()))
        name = Path(path).name
        self._show_code(
            f'DIM snd AS SOUND\n'
            f'snd = LOADSOUND("{name}")\n'
            f'PLAYSOUND(snd)\n', title=f"{name} gespeichert")

    def _export_code(self) -> None:
        p = self._params()
        # AUDIO_SFX erzeugt den Effekt prozedural zur Laufzeit (nativ in gbrt),
        # kein WAV-Asset noetig. Die optionalen Trailing-Args sind positionell
        # -> alle bis zum letzten nicht-Default-Wert anhaengen.
        sfx = (f'AUDIO_SFX("{p["waveform"]}", {p["base_freq"]:g}, '
               f'{p["slide"]:g}, {p["attack"]}, {p["sustain"]}, {p["decay"]}, '
               f'{p["vib_depth"]:g}, {p["vib_speed"]:g}, {p["volume"]:g}')
        # (Wert, Default) in AUDIO_SFX-Argument-Reihenfolge ab Index 9.
        trailing = [
            (p["stereo_width"], 0.0), (p["duty"], 0.5),
            (p["pwm_depth"], 0.0), (p["pwm_speed"], 0.0),
            (p["flt_cutoff"], 0.0), (p["flt_sweep"], 0.0), (p["flt_res"], 0.0),
        ]
        last = -1
        for i, (v, d) in enumerate(trailing):
            if abs(v - d) > 1e-9:
                last = i
        for v, _d in trailing[:last + 1]:
            sfx += f', {v:g}'
        sfx += ")"
        lines = ['IMPORT "audio"', "", "DIM snd AS SOUND", f"snd = {sfx}"]
        pan = p["pan"]
        if abs(pan) > 0.001:
            # Pan zur Laufzeit ueber den Wiedergabe-Channel (AUDIO_PAN).
            left = max(0.0, min(1.0, 1.0 - max(0.0, pan)))
            right = max(0.0, min(1.0, 1.0 + min(0.0, pan)))
            lines += [
                "DIM ch AS AUDIO_CHANNEL",
                "ch = AUDIO_PLAY(snd)",
                f"AUDIO_PAN(ch, {left:g}, {right:g})",
            ]
        else:
            lines.append("PLAYSOUND(snd)")
        self._show_code("\n".join(lines) + "\n", title="GB-Code (AUDIO_SFX)")

    def _show_code(self, code: str, title: str) -> None:
        dlg = QFrame(self, Qt.WindowType.Window)
        dlg.setWindowTitle(title)
        dlg.resize(520, 260)
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
    app = QApplication.instance() or QApplication([])
    app.setStyleSheet(global_qss())
    win = SfxGenerator(project_root)
    win.show()
    return app.exec()
