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
from PySide6.QtCore import Qt, QTimer, Signal
from PySide6.QtGui import QColor, QFont, QKeySequence, QPainter, QPen, QShortcut
from PySide6.QtWidgets import (
    QApplication, QCheckBox, QComboBox, QFileDialog, QFrame, QGridLayout,
    QGroupBox, QHBoxLayout, QLabel, QMainWindow, QPlainTextEdit, QPushButton,
    QVBoxLayout, QWidget,
)

from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .editor_qt.fader import Fader
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
        self.resize(900, 680)
        self._counter = 0

        # Auto-Play: kurz entprellt nach der letzten Regler-Aenderung spielen,
        # damit ein Drag genau einmal (beim Loslassen) hoerbar wird.
        self._autoplay_timer = QTimer(self)
        self._autoplay_timer.setSingleShot(True)
        self._autoplay_timer.setInterval(160)
        self._autoplay_timer.timeout.connect(self._play)

        central = QWidget()
        self.setCentralWidget(central)
        root = QVBoxLayout(central)

        title = QLabel("⚡  SFX-Generator")
        tf = QFont(); tf.setBold(True); tf.setPointSize(14)
        title.setFont(tf)
        title.setStyleSheet(f"color:{COLORS['accent']}; padding:2px 0;")
        root.addWidget(title)

        # --- Transport (Abspielen/Zufall/Export + Undo) ---
        btns = QHBoxLayout()
        b_play = QPushButton("▶  Abspielen")
        b_play.setProperty("accent", True)
        b_play.clicked.connect(self._play)
        btns.addWidget(b_play)
        b_rand = QPushButton("🎲  Zufall")
        b_rand.clicked.connect(self._randomize)
        btns.addWidget(b_rand)
        self.cb_autoplay = QCheckBox("Auto-Play")
        self.cb_autoplay.setChecked(True)
        self.cb_autoplay.setToolTip(
            "Beim Verschieben eines Reglers den Ton automatisch abspielen "
            "(kurz entprellt -- spielt, sobald du loslaesst).")
        btns.addWidget(self.cb_autoplay)
        btns.addStretch(1)
        self.btn_undo = QPushButton("↶")
        self.btn_undo.setToolTip("Rueckgaengig (Strg+Z)")
        self.btn_undo.setFixedWidth(34)
        btns.addWidget(self.btn_undo)
        self.btn_redo = QPushButton("↷")
        self.btn_redo.setToolTip("Wiederholen (Strg+Y)")
        self.btn_redo.setFixedWidth(34)
        btns.addWidget(self.btn_redo)
        b_wav = QPushButton("WAV exportieren ...")
        b_wav.clicked.connect(self._export_wav)
        btns.addWidget(b_wav)
        b_code = QPushButton("GB-Code")
        b_code.clicked.connect(self._export_code)
        btns.addWidget(b_code)
        root.addLayout(btns)

        # Preset-Bibliothek (eigene Sounds speichern/laden)
        self.presets = PresetLibrary(default_dir() / "sfx.json")
        self.preset_bar = PresetBar(
            self.presets, self._params, self._apply_params)
        root.addWidget(self.preset_bar)

        # --- Hauptbereich: Preset-Leiste links | Wellenform + Fader-Bank ---
        main = QHBoxLayout()
        main.setSpacing(10)

        rail = QVBoxLayout(); rail.setSpacing(5)
        rl = QLabel("Presets"); rl.setStyleSheet(
            f"color:{COLORS['fg_muted']}; font-size:11px;")
        rail.addWidget(rl)
        for name in _PRESETS:
            b = QPushButton(name)
            b.clicked.connect(lambda _=False, n=name: self._load_preset(n))
            rail.addWidget(b)
        rail.addStretch(1)
        rail_w = QWidget(); rail_w.setLayout(rail); rail_w.setFixedWidth(132)
        main.addWidget(rail_w)

        right = QVBoxLayout(); right.setSpacing(8)
        self.wave_view = _WaveView()
        self.wave_view.setMinimumHeight(150)
        right.addWidget(self.wave_view)

        grid = QGridLayout(); grid.setSpacing(8)
        cyan, mint = COLORS["accent"], COLORS["success"]
        magenta, amber = COLORS["danger"], "#EF9F27"

        gb1, c1 = self._group("Ton", cyan)
        self.waveform = QComboBox(); self.waveform.addItems(_WAVEFORMS)
        self.waveform.currentTextChanged.connect(self._on_change)
        self._row(c1, "Waveform", self.waveform)
        self.base_freq = self._fader(c1, "Frequenz", 50, 8000, 800, 1, 0, "Hz", cyan)
        self.slide = self._fader(c1, "Pitch-Slide", -8000, 8000, 0, 1, 0, "Hz/s", cyan)
        self.volume = self._fader(c1, "Lautstaerke", 0.0, 1.0, 0.7, 0.01, 2, "", cyan)
        grid.addWidget(gb1, 0, 0)

        gb2, c2 = self._group("Huellkurve", mint)
        self.attack = self._fader(c2, "Attack", 0, 2000, 0, 1, 0, "ms", mint)
        self.sustain = self._fader(c2, "Sustain", 0, 4000, 40, 1, 0, "ms", mint)
        self.decay = self._fader(c2, "Decay", 0, 4000, 150, 1, 0, "ms", mint)
        grid.addWidget(gb2, 0, 1)

        gb3, c3 = self._group("SID / Filter", magenta)
        self.duty = self._fader(c3, "Pulsbreite", 0.05, 0.95, 0.5, 0.01, 2, "", magenta)
        self.pwm_depth = self._fader(c3, "PWM-Tiefe", 0.0, 0.45, 0.0, 0.01, 2, "", magenta)
        self.pwm_speed = self._fader(c3, "PWM-Speed", 0.0, 30.0, 0.0, 0.5, 1, "Hz", magenta)
        self.flt_cutoff = self._fader(c3, "Filter-Cutoff", 0, 12000, 0, 50, 0, "Hz", magenta)
        self.flt_sweep = self._fader(c3, "Filter-Sweep", -12000, 12000, 0, 100, 0, "Hz/s", magenta)
        self.flt_res = self._fader(c3, "Resonanz", 0.0, 0.95, 0.0, 0.05, 2, "", magenta)
        grid.addWidget(gb3, 1, 0)

        gb4, c4 = self._group("Vibrato / Stereo", amber)
        self.vib_depth = self._fader(c4, "Vib-Tiefe", 0.0, 0.5, 0.0, 0.01, 2, "", amber)
        self.vib_speed = self._fader(c4, "Vib-Speed", 0, 60, 0, 1, 0, "Hz", amber)
        self.stereo_width = self._fader(c4, "Stereo-Breite", 0.0, 1.0, 0.0, 0.05, 2, "", amber)
        self.pan = self._fader(c4, "Pan (L..R)", -1.0, 1.0, 0.0, 0.1, 1, "", amber)
        grid.addWidget(gb4, 1, 1)

        right.addLayout(grid)
        right.addStretch(1)
        main.addLayout(right, 1)
        root.addLayout(main, 1)

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
        lab = QLabel(label)
        lab.setStyleSheet(f"color:{COLORS['fg_muted']}; font-size:11px;")
        lab.setFixedWidth(74)
        r.addWidget(lab); r.addWidget(widget, 1)
        layout.addLayout(r)

    def _group(self, title: str, accent: str):
        """Eine farbcodierte Parameter-Karte. Liefert (GroupBox, Layout)."""
        gb = QGroupBox(title)
        gb.setStyleSheet(
            f"QGroupBox::title {{ color:{accent}; "
            f"background-color:{COLORS['bg_alt']}; }}")
        lay = QVBoxLayout(gb); lay.setSpacing(7)
        return gb, lay

    def _fader(self, layout, label, lo, hi, val, step=1.0, decimals=0,
               unit="", accent=None):
        f = Fader(label, lo, hi, val, step, decimals, unit, accent)
        f.valueChanged.connect(self._on_change)
        layout.addWidget(f)
        return f

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
        # Auto-Play (entprellt): startet/erneuert den Timer, solange gezogen wird.
        cb = getattr(self, "cb_autoplay", None)
        if cb is not None and cb.isChecked():
            self._autoplay_timer.start()

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
        # Jede explizite Wiedergabe (Button/Preset/Zufall) bricht ein
        # ausstehendes Auto-Play ab -> kein Doppel-Ton.
        self._autoplay_timer.stop()
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
        # WA_DeleteOnClose: sonst haengt jedes per Export erzeugte Fenster als
        # verstecktes Kind von `self` weiter (Qt raeumt Kind-Widgets nur beim
        # Schliessen des Eltern-Fensters auf) -- wiederholtes Exportieren in
        # einer Sitzung haette so Fenster angesammelt, die nie freigegeben werden.
        dlg.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose, True)
        dlg.show()
        self._code_dlg = dlg


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance() or QApplication([])
    app.setStyleSheet(global_qss())
    win = SfxGenerator(project_root)
    win.show()
    return app.exec()
