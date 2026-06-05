"""Chiptune-Tracker fuer GameBasic (`gbtracker` / `gbrun.py --tracker`).

Mehrspuriger Pattern-Editor: 3 Ton-Kanaele (je eigene Waveform) + 1 Noise-
Kanal (Drums). **Mehrere Patterns mit einstellbarer Laenge + Song-Arrangement**
(Order: Reihenfolge, in der Patterns abgespielt werden). Noten per klickbarer
Klaviatur in die Gitter-Zellen setzen, Pattern ODER ganzen Song abspielen
(nutzt den geteilten Synth `gamebasic.synth`), Projekt als `.json` speichern/
laden und als GB-Code exportieren -- ein frame-basierter Player
(`TRACKER_UPDATE`), der mit `DELTA()` im Game-Loop laeuft.

Das Datenmodell + I/O + GB-Export liegen Qt-frei in `gamebasic.tracker`
(headless getestet).
"""
from __future__ import annotations

from pathlib import Path

import numpy as np
from PySide6.QtCore import Qt, QTimer, Signal
from PySide6.QtGui import QColor, QFont, QKeySequence, QPainter, QPen, QShortcut
from PySide6.QtWidgets import (
    QApplication, QComboBox, QFileDialog, QFrame, QHBoxLayout, QHeaderView,
    QLabel, QListWidget, QListWidgetItem, QMainWindow, QMessageBox,
    QPlainTextEdit, QPushButton, QSpinBox, QTableWidget, QTableWidgetItem,
    QVBoxLayout, QWidget,
)

from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .editor_qt.undo_history import SnapshotUndo
from .synth import synthesize
from .tracker import (
    CHANNELS, SLIDE_MAX, TONAL, VOL_MAX, WAVEFORMS, Song, midi_to_freq,
    note_name, vol_to_pct,
)


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
        self.project_root = Path(project_root)
        self.setWindowTitle("GameBasic Tracker")
        self.resize(760, 760)
        self.song = Song()
        self.cur = 0                   # aktueller Pattern-Index
        self._sound_cache: dict = {}
        self._play_mode = None         # None | "pattern" | "song"
        self._play_row = 0
        self._play_order_pos = 0

        central = QWidget()
        self.setCentralWidget(central)
        root = QVBoxLayout(central)

        title = QLabel("Tracker")
        tf = QFont(); tf.setBold(True); tf.setPointSize(13)
        title.setFont(tf)
        root.addWidget(title)

        # --- Transport ---
        top = QHBoxLayout()
        self.btn_play = QPushButton("▶ Pattern")
        self.btn_play.setProperty("accent", True)
        self.btn_play.clicked.connect(lambda: self._toggle_play("pattern"))
        top.addWidget(self.btn_play)
        self.btn_song = QPushButton("▶ Song")
        self.btn_song.clicked.connect(lambda: self._toggle_play("song"))
        top.addWidget(self.btn_song)
        top.addWidget(QLabel("BPM:"))
        self.bpm = QSpinBox(); self.bpm.setRange(40, 300); self.bpm.setValue(self.song.bpm)
        self.bpm.valueChanged.connect(self._on_bpm)
        top.addWidget(self.bpm)
        self.wave_combos = []
        for ci in range(TONAL):
            top.addWidget(QLabel(f"Ch{ci + 1}:"))
            cb = QComboBox(); cb.addItems(WAVEFORMS)
            cb.setCurrentText(self.song.waves[ci])
            cb.currentTextChanged.connect(lambda v, i=ci: self._set_wave(i, v))
            top.addWidget(cb)
            self.wave_combos.append(cb)
        top.addStretch(1)
        b_code = QPushButton("GB-Code"); b_code.clicked.connect(self._export)
        top.addWidget(b_code)
        root.addLayout(top)

        # --- Datei + Pattern-Verwaltung ---
        prow = QHBoxLayout()
        b_new = QPushButton("Neu"); b_new.clicked.connect(self._new_song)
        b_open = QPushButton("Oeffnen"); b_open.clicked.connect(self._open)
        b_save = QPushButton("Speichern"); b_save.clicked.connect(self._save)
        for b in (b_new, b_open, b_save):
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
        b_padd = QPushButton("+ Pattern"); b_padd.clicked.connect(self._add_pattern)
        b_pdup = QPushButton("Duplizieren"); b_pdup.clicked.connect(self._dup_pattern)
        b_pdel = QPushButton("Loeschen"); b_pdel.clicked.connect(self._del_pattern)
        b_clear = QPushButton("Leeren"); b_clear.clicked.connect(self._clear)
        for b in (b_padd, b_pdup, b_pdel, b_clear):
            prow.addWidget(b)
        prow.addStretch(1)
        root.addLayout(prow)

        # --- Pattern-Gitter ---
        self.grid = QTableWidget(self.song.patterns[0].rows, CHANNELS)
        self.grid.setHorizontalHeaderLabels(["Ch1", "Ch2", "Ch3", "Drum"])
        self.grid.verticalHeader().setDefaultSectionSize(22)
        self.grid.horizontalHeader().setSectionResizeMode(
            QHeaderView.ResizeMode.Stretch)
        self.grid.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self.grid.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectItems)
        self.grid.setSelectionMode(QTableWidget.SelectionMode.SingleSelection)
        self.grid.setFont(QFont(EDITOR_FONT_FAMILY, 10))
        self.grid.itemSelectionChanged.connect(self._audition_selected)
        self.grid.itemSelectionChanged.connect(self._sync_vol_spin)
        root.addWidget(self.grid, 1)

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
        krow.addWidget(QLabel("  (Zelle waehlen, dann Taste klicken; Entf loescht)"))
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
        self.btn_undo.clicked.connect(self.undo.undo)
        self.btn_redo.clicked.connect(self.undo.redo)
        QShortcut(QKeySequence.StandardKey.Undo, self, activated=self.undo.undo)
        QShortcut(QKeySequence.StandardKey.Redo, self, activated=self.undo.redo)
        QShortcut(QKeySequence("Ctrl+Y"), self, activated=self.undo.redo)
        self._update_undo_buttons()

    def _mark(self) -> None:
        u = getattr(self, "undo", None)
        if u is not None:
            u.mark()

    def _update_undo_buttons(self) -> None:
        self.btn_undo.setEnabled(self.undo.can_undo())
        self.btn_redo.setEnabled(self.undo.can_redo())

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
        for ci, cb in enumerate(self.wave_combos):
            cb.blockSignals(True); cb.setCurrentText(self.song.waves[ci]); cb.blockSignals(False)
        self.cur = min(self.cur, len(self.song.patterns) - 1)
        self._reload_pattern_combo()
        self._reload_order()
        self._load_pattern(self.cur)

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
        self.grid.setVerticalHeaderLabels([f"{r:02d}" for r in range(pat.rows)])
        for r in range(pat.rows):
            for c in range(CHANNELS):
                it = QTableWidgetItem(
                    self._cell_text(c, pat.data[c][r], pat.vol[c][r],
                                    pat.slide[c][r]))
                it.setTextAlignment(Qt.AlignmentFlag.AlignCenter)
                self.grid.setItem(r, c, it)
        self.grid.blockSignals(False)
        self._sync_vol_spin()

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
        self._mark()

    def _cell_text(self, ci: int, note, vol=None, slide=None) -> str:
        if note is None:
            return "···"
        base = "  X" if ci == TONAL else note_name(note)
        if vol:
            base += f" v{vol}"
        if slide:
            base += f" s{slide:+d}"
        return base

    def _cell_refresh(self, row: int, ci: int) -> None:
        pat = self.song.patterns[self.cur]
        self.grid.item(row, ci).setText(
            self._cell_text(ci, pat.data[ci][row], pat.vol[ci][row],
                            pat.slide[ci][row]))

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
        if pat.data[c][r] is None or c == TONAL:   # nur tonale Noten
            return
        pat.set_slide(c, r, s)               # 0 -> None
        self._cell_refresh(r, c)
        self._mark()

    def _sync_vol_spin(self) -> None:
        """Spiegelt die Effekte der gewaehlten Zelle in die Spinboxen."""
        self.vol_spin.blockSignals(True)
        self.slide_spin.blockSignals(True)
        if self._has_sel():
            r, c = self._sel()
            pat = self.song.patterns[self.cur]
            self.vol_spin.setValue(pat.vol[c][r] or 0)
            self.slide_spin.setValue(pat.slide[c][r] or 0)
            self.slide_spin.setEnabled(c != TONAL)
        else:
            self.vol_spin.setValue(0)
            self.slide_spin.setValue(0)
        self.vol_spin.blockSignals(False)
        self.slide_spin.blockSignals(False)

    def _on_piano(self, midi: int) -> None:
        ch = self._sel_channel() if self._has_sel() else 0
        self._play_note(ch, midi)
        if self._has_sel():
            r, c = self._sel()
            self._set_note(r, c, midi)
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
            if n is not None:
                self._play_note(c, n, pat.vol[c][r], pat.slide[c][r] or 0)

    def keyPressEvent(self, e):  # noqa: N802
        if e.key() in (Qt.Key.Key_Delete, Qt.Key.Key_Backspace) and self._has_sel():
            r, c = self._sel()
            self._set_note(r, c, None)
            return
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
    def _sound(self, ci: int, midi: int, slide: int = 0):
        wf = "noise" if ci == TONAL else self.song.waves[ci]
        key = (ci, wf, midi, slide)
        snd = self._sound_cache.get(key)
        if snd is None:
            freq = 220.0 if ci == TONAL else midi_to_freq(midi)
            dec = 120 if ci == TONAL else 220
            slide_hz = 0.0
            if slide and ci != TONAL:
                # Halbtoene -> Hz/s ueber die Reihen-Dauer (wie der Export).
                target = freq * (2.0 ** (slide / 12.0))
                row_s = max(0.001, self.song.row_ms() / 1000.0)
                slide_hz = (target - freq) / row_s
            wave = synthesize(wf, freq, slide_hz, 4, 40, dec)
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

    def _play_note(self, ci: int, midi: int, vol: int | None = None,
                   slide: int = 0) -> None:
        snd = self._sound(ci, midi, slide or 0)
        if snd is not None:
            try:
                ch = snd.play()
                if ch is not None and vol:   # vol = Lautstaerke 1..15
                    ch.set_volume(min(1.0, max(0.0, vol_to_pct(vol) / 100.0)))
            except Exception:
                pass

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
        self.btn_play.setText("■ Stop" if mode == "pattern" else "▶ Pattern")
        self.btn_song.setText("■ Stop" if mode == "song" else "▶ Song")

    def _stop_play(self) -> None:
        self._timer.stop()
        self.btn_play.setText("▶ Pattern")
        self.btn_song.setText("▶ Song")

    def _tick(self) -> None:
        if self._play_mode == "pattern":
            pat = self.song.patterns[self.cur]
            self._play_row %= pat.rows
            self.grid.selectRow(self._play_row)
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
            self.grid.selectRow(self._play_row)
            self._play_columns(pat, self._play_row)
            self._play_row += 1
            if self._play_row >= pat.rows:
                self._play_row = 0
                self._play_order_pos = (self._play_order_pos + 1) % len(order)

    def _play_columns(self, pat, row: int) -> None:
        for c in range(CHANNELS):
            n = pat.data[c][row]
            if n is not None:
                self._play_note(c, n, pat.vol[c][row], pat.slide[c][row] or 0)

    # ============================================== Datei
    def _new_song(self) -> None:
        self.song = Song()
        self.cur = 0
        self._reload_all()
        self.undo.reset()      # frisches Dokument -> Historie verwerfen
        self.setWindowTitle("GameBasic Tracker")

    def _open(self) -> None:
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
        self.cur = 0
        self._sound_cache.clear()
        self._reload_all()
        self.undo.reset()      # geladenes Dokument -> Historie verwerfen
        self.setWindowTitle(f"GameBasic Tracker -- {Path(path).name}")

    def _save(self) -> None:
        path, _ = QFileDialog.getSaveFileName(
            self, "Tracker-Projekt speichern", str(self.project_root),
            "Tracker-Projekt (*.json)")
        if not path:
            return
        if not path.lower().endswith(".json"):
            path += ".json"
        try:
            self.song.save_json(path)
        except Exception as exc:  # noqa: BLE001
            QMessageBox.warning(self, "Fehler", f"Konnte nicht speichern:\n{exc}")
            return
        self.setWindowTitle(f"GameBasic Tracker -- {Path(path).name}")

    # ============================================== Export
    def _export(self) -> None:
        self._show_code(self.song.gb_code())

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
    if initial_file and Path(initial_file).exists():
        try:
            win.song = Song.load_json(str(initial_file))
            win.cur = 0
            win._reload_all()
        except Exception:
            pass
    win.show()
    return app.exec()
