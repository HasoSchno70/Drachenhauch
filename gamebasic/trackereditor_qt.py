"""Chiptune-Tracker fuer GameBasic (`gbtracker` / `gbrun.py --tracker`).

Mehrspuriger Pattern-Editor: 3 Ton-Kanaele (je eigene Waveform) + 1 Noise-
Kanal (Drums), 16 Reihen pro Pattern. Noten per klickbarer Klaviatur in die
Gitter-Zellen setzen, abspielen (nutzt den geteilten Synth `gamebasic.synth`),
und als GB-Code exportieren -- ein frame-basierter Player (`TRACKER_UPDATE`),
der mit `DELTA()` im Game-Loop laeuft.
"""
from __future__ import annotations

from pathlib import Path

import numpy as np
from PySide6.QtCore import Qt, QTimer, Signal
from PySide6.QtGui import QColor, QFont, QPainter, QPen
from PySide6.QtWidgets import (
    QApplication, QComboBox, QFrame, QHBoxLayout, QHeaderView, QLabel,
    QMainWindow, QPlainTextEdit, QPushButton, QSpinBox, QTableWidget,
    QTableWidgetItem, QVBoxLayout, QWidget,
)

from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .synth import synthesize

_NOTE_NAMES = ("C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B")
_WAVEFORMS = ("square", "saw", "sine", "triangle")
_ROWS = 16
_TONAL = 3                 # Kanaele 0..2 tonal, Kanal 3 = Noise/Drum
_CHANNELS = _TONAL + 1


def midi_to_freq(m: int) -> float:
    return 440.0 * (2.0 ** ((m - 69) / 12.0))


def note_name(m: int) -> str:
    return f"{_NOTE_NAMES[m % 12]}{m // 12 - 1}"


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


class TrackerEditor(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = project_root
        self.setWindowTitle("GameBasic Tracker")
        self.resize(720, 660)
        # Pattern: pro Kanal eine Liste von _ROWS (MIDI-Note oder None).
        self.pattern = [[None] * _ROWS for _ in range(_CHANNELS)]
        self.wave = ["square", "saw", "triangle"]      # Kanal 0..2
        self._sound_cache: dict = {}
        self._play_row = 0

        central = QWidget()
        self.setCentralWidget(central)
        root = QVBoxLayout(central)

        title = QLabel("Tracker")
        tf = QFont(); tf.setBold(True); tf.setPointSize(13)
        title.setFont(tf)
        root.addWidget(title)

        # Transport
        top = QHBoxLayout()
        self.btn_play = QPushButton("▶ Abspielen")
        self.btn_play.setProperty("accent", True)
        self.btn_play.clicked.connect(self._toggle_play)
        top.addWidget(self.btn_play)
        top.addWidget(QLabel("BPM:"))
        self.bpm = QSpinBox(); self.bpm.setRange(40, 300); self.bpm.setValue(120)
        top.addWidget(self.bpm)
        # Waveform pro Ton-Kanal
        self.wave_combos = []
        for ci in range(_TONAL):
            top.addWidget(QLabel(f"Ch{ci + 1}:"))
            cb = QComboBox(); cb.addItems(_WAVEFORMS)
            cb.setCurrentText(self.wave[ci])
            cb.currentTextChanged.connect(lambda v, i=ci: self._set_wave(i, v))
            top.addWidget(cb)
            self.wave_combos.append(cb)
        top.addStretch(1)
        b_clear = QPushButton("Leeren"); b_clear.clicked.connect(self._clear)
        top.addWidget(b_clear)
        b_code = QPushButton("GB-Code"); b_code.clicked.connect(self._export)
        top.addWidget(b_code)
        root.addLayout(top)

        # Pattern-Gitter
        self.grid = QTableWidget(_ROWS, _CHANNELS)
        self.grid.setHorizontalHeaderLabels(["Ch1", "Ch2", "Ch3", "Drum"])
        self.grid.verticalHeader().setDefaultSectionSize(22)
        self.grid.setVerticalHeaderLabels([f"{r:02d}" for r in range(_ROWS)])
        self.grid.horizontalHeader().setSectionResizeMode(
            QHeaderView.ResizeMode.Stretch)
        self.grid.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self.grid.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectItems)
        self.grid.setSelectionMode(QTableWidget.SelectionMode.SingleSelection)
        self.grid.setFont(QFont(EDITOR_FONT_FAMILY, 10))
        for r in range(_ROWS):
            for c in range(_CHANNELS):
                it = QTableWidgetItem("···")
                it.setTextAlignment(Qt.AlignmentFlag.AlignCenter)
                self.grid.setItem(r, c, it)
        self.grid.itemSelectionChanged.connect(self._audition_selected)
        root.addWidget(self.grid, 1)

        # Klaviatur + Oktave
        prow = QHBoxLayout()
        prow.addWidget(QLabel("Oktave:"))
        self.octave = QSpinBox(); self.octave.setRange(2, 6); self.octave.setValue(4)
        self.octave.valueChanged.connect(
            lambda v: self.piano.set_base(12 * (v + 1)))
        prow.addWidget(self.octave)
        prow.addWidget(QLabel("  (Zelle waehlen, dann Taste klicken; Entf loescht)"))
        prow.addStretch(1)
        root.addLayout(prow)
        self.piano = _Piano()
        self.piano.set_base(12 * (self.octave.value() + 1))
        self.piano.note_clicked.connect(self._on_piano)
        root.addWidget(self.piano)

        # Playback-Timer
        self._timer = QTimer(self)
        self._timer.timeout.connect(self._tick)

    # ---------------------------------------------------- Bearbeiten
    def _set_wave(self, ci: int, v: str) -> None:
        self.wave[ci] = v
        self._sound_cache.clear()

    def _cell_text(self, ci: int, note) -> str:
        if note is None:
            return "···"
        return "  X" if ci == _TONAL else note_name(note)

    def _set_note(self, row: int, ci: int, note) -> None:
        self.pattern[ci][row] = note
        self.grid.item(row, ci).setText(self._cell_text(ci, note))

    def _on_piano(self, midi: int) -> None:
        self._play_note(self._sel_channel() if self._has_sel() else 0, midi)
        if self._has_sel():
            r, c = self._sel()
            self._set_note(r, c, midi)
            # automatisch eine Reihe weiter
            if r + 1 < _ROWS:
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
            n = self.pattern[c][r]
            if n is not None:
                self._play_note(c, n)

    def keyPressEvent(self, e):  # noqa: N802
        if e.key() in (Qt.Key.Key_Delete, Qt.Key.Key_Backspace) and self._has_sel():
            r, c = self._sel()
            self._set_note(r, c, None)
            return
        super().keyPressEvent(e)

    def _clear(self) -> None:
        for c in range(_CHANNELS):
            for r in range(_ROWS):
                self._set_note(r, c, None)

    # ---------------------------------------------------- Sound
    def _sound(self, ci: int, midi: int):
        wf = "noise" if ci == _TONAL else self.wave[ci]
        key = (ci, wf, midi)
        snd = self._sound_cache.get(key)
        if snd is None:
            freq = 220.0 if ci == _TONAL else midi_to_freq(midi)
            dec = 120 if ci == _TONAL else 220
            wave = synthesize(wf, freq, 0.0, 4, 40, dec)
            snd = self._make_sound(wave)
            self._sound_cache[key] = snd
        return snd

    @staticmethod
    def _make_sound(wave: np.ndarray, sr: int = 44100):
        try:
            import os
            os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "hide")
            import pygame
            if pygame.mixer.get_init() is None:
                pygame.mixer.init(frequency=sr, size=-16, channels=2)
            int16 = (np.clip(wave, -1, 1) * 0.6 * 32767).astype(np.int16)
            if int16.ndim == 1:
                int16 = np.column_stack((int16, int16))
            return pygame.sndarray.make_sound(np.ascontiguousarray(int16))
        except Exception:
            return None

    def _play_note(self, ci: int, midi: int) -> None:
        snd = self._sound(ci, midi)
        if snd is not None:
            try:
                snd.play()
            except Exception:
                pass

    # ---------------------------------------------------- Playback
    def _toggle_play(self) -> None:
        if self._timer.isActive():
            self._timer.stop()
            self.btn_play.setText("▶ Abspielen")
            self.grid.setRangeSelected(
                self.grid.selectedRanges()[0], False) if self.grid.selectedRanges() else None
        else:
            self._play_row = 0
            self._timer.setInterval(int(60000 / self.bpm.value() / 4))
            self._timer.start()
            self.btn_play.setText("■ Stop")

    def _tick(self) -> None:
        r = self._play_row
        self.grid.selectRow(r)
        for c in range(_CHANNELS):
            n = self.pattern[c][r]
            if n is not None:
                self._play_note(c, n)
        self._play_row = (r + 1) % _ROWS

    # ---------------------------------------------------- Export
    def _export(self) -> None:
        rowms = int(60000 / self.bpm.value() / 4)
        lines = ['IMPORT "audio"', "",
                 f"' === Tracker-Song (GameBasic-Tracker) -- {_CHANNELS} Kanaele, "
                 f"{_ROWS} Reihen, BPM {self.bpm.value()} ===",
                 f"CONST TRK_ROWS = {_ROWS}",
                 f"CONST TRK_ROWMS = {rowms}", ""]
        # Frequenz-Arrays pro Kanal: DIM mit Groesse (Default 0), dann nur die
        # belegten Zellen setzen (Drum: 1 = Hit, Ton: Frequenz in Hz).
        for c in range(_CHANNELS):
            lines.append(f"DIM trk{c}[TRK_ROWS] AS INTEGER")
            for r in range(_ROWS):
                n = self.pattern[c][r]
                if n is None:
                    continue
                val = 1 if c == _TONAL else int(round(midi_to_freq(n)))
                lines.append(f"trk{c}[{r}] = {val}")
        lines += [
            "",
            "' --- Player-Status + Update (im Game-Loop aufrufen) ---",
            "DIM trkRow AS INTEGER",
            "DIM trkAcc AS FLOAT",
            "trkRow = 0",
            "trkAcc = 0.0",
            "",
            "SUB TRACKER_PLAY_ROW(r AS INTEGER)",
        ]
        for c in range(_TONAL):
            lines.append(
                f"    IF trk{c}[r] > 0 THEN PLAYSOUND("
                f'AUDIO_TONE(trk{c}[r], TRK_ROWMS, "{self.wave[c]}", 0.5))')
        lines.append(
            f"    IF trk{_TONAL}[r] > 0 THEN PLAYSOUND(AUDIO_NOISE(120, 0.5))")
        lines += [
            "END SUB",
            "",
            "SUB TRACKER_UPDATE(dt_ms AS FLOAT)",
            "    trkAcc = trkAcc + dt_ms",
            "    WHILE trkAcc >= TRK_ROWMS",
            "        trkAcc = trkAcc - TRK_ROWMS",
            "        TRACKER_PLAY_ROW(trkRow)",
            "        trkRow = (trkRow + 1) MOD TRK_ROWS",
            "    WEND",
            "END SUB",
            "",
            "' Im Game-Loop:  TRACKER_UPDATE(DELTA() * 1000.0)",
        ]
        self._show_code("\n".join(lines) + "\n")

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
    app = QApplication.instance() or QApplication([])
    app.setStyleSheet(global_qss())
    win = TrackerEditor(project_root)
    win.show()
    return app.exec()
