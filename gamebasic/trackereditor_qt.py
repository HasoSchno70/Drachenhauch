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
    QApplication, QComboBox, QDialog, QDialogButtonBox, QDoubleSpinBox,
    QFileDialog, QFormLayout, QFrame, QHBoxLayout, QHeaderView, QLabel,
    QListWidget, QListWidgetItem, QMainWindow, QMessageBox, QPlainTextEdit,
    QPushButton, QSpinBox, QTableWidget, QTableWidgetItem, QVBoxLayout,
    QWidget,
)

from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .editor_qt.undo_history import SnapshotUndo
from .synth import synthesize
from .tracker import (
    CHANNELS, SLIDE_MAX, TONAL, VOL_MAX, WAVEFORMS, Song, midi_to_freq,
    note_name, vol_to_pct,
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

        box = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel)
        box.accepted.connect(self.accept)
        box.rejected.connect(self.reject)
        form.addRow(box)

    def _upd_base_label(self) -> None:
        self.base_label.setText(note_name(self.base.value()))

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
        self.resize(1120, 840)
        self.song = Song()
        self.cur = 0                   # aktueller Pattern-Index
        self._sound_cache: dict = {}
        self._play_mode = None         # None | "pattern" | "song"
        self._play_row = 0
        self._play_order_pos = 0
        self._playhead = -1            # aktuell hervorgehobene Wiedergabe-Reihe

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
        top.addStretch(1)
        b_wav = QPushButton("♪ Audio (WAV)...")
        b_wav.setToolTip("Song mit allen Instrumenten als WAV rendern "
                         "(im Spiel via PLAYMUSIC)")
        b_wav.clicked.connect(self._export_audio)
        top.addWidget(b_wav)
        b_code = QPushButton("GB-Code"); b_code.clicked.connect(self._export)
        top.addWidget(b_code)
        root.addLayout(top)

        # --- Sound pro Spur (Keyboard-Klang je Kanal) ---
        srow = QHBoxLayout()
        lab = QLabel("Spur-Sounds:")
        lab.setStyleSheet(f"color: {COLORS['accent']}; font-weight: bold;")
        srow.addWidget(lab)
        self.sound_combos = []
        ch_names = ["Ch1", "Ch2", "Ch3", "Drum"]
        for c in range(CHANNELS):
            srow.addWidget(QLabel(ch_names[c] + ":"))
            cb = QComboBox(); cb.setMinimumWidth(150)
            cb.currentIndexChanged.connect(
                lambda idx, ch=c: self._on_sound_changed(ch, idx))
            srow.addWidget(cb)
            self.sound_combos.append(cb)
        srow.addStretch(1)
        root.addLayout(srow)

        # --- Instrument-Bibliothek (eigene Sounds verwalten) ---
        irow = QHBoxLayout()
        irow.addWidget(QLabel("Bibliothek:"))
        self.inst_combo = QComboBox()
        self.inst_combo.setMinimumWidth(170)
        irow.addWidget(self.inst_combo)
        b_load = QPushButton("+ Sample (WAV)...")
        b_load.setToolTip("Eigene Aufnahme als Instrument laden")
        b_load.clicked.connect(self._load_sample)
        irow.addWidget(b_load)
        b_keymap = QPushButton("+ Keymap...")
        b_keymap.setToolTip("Mehrere Samples ueber die Klaviatur verteilen "
                            "(Multisample / Drumkit)")
        b_keymap.clicked.connect(self._edit_keymap)
        irow.addWidget(b_keymap)
        b_iedit = QPushButton("Bearbeiten...")
        b_iedit.setToolTip("Grundton, Loop-Punkte, ADSR-Huellkurve")
        b_iedit.clicked.connect(self._edit_instrument)
        irow.addWidget(b_iedit)
        b_idel = QPushButton("Loeschen")
        b_idel.clicked.connect(self._remove_instrument)
        irow.addWidget(b_idel)
        irow.addStretch(1)
        root.addLayout(irow)

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
        self.grid.verticalHeader().setDefaultSectionSize(26)
        self.grid.verticalHeader().setSectionResizeMode(
            QHeaderView.ResizeMode.Fixed)
        self.grid.horizontalHeader().setSectionResizeMode(
            QHeaderView.ResizeMode.Stretch)
        self.grid.horizontalHeader().setFixedHeight(40)
        self.grid.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self.grid.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectItems)
        self.grid.setSelectionMode(QTableWidget.SelectionMode.SingleSelection)
        self.grid.setShowGrid(False)
        gfont = QFont(EDITOR_FONT_FAMILY, 12); gfont.setBold(True)
        self.grid.setFont(gfont)
        self.grid.setStyleSheet(_TRACKER_GRID_QSS)
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
        self.cur = min(self.cur, len(self.song.patterns) - 1)
        self._refresh_instruments()
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
                self._style_cell(it, c, r, pat.data[c][r])
        self.grid.blockSignals(False)
        self._update_channel_headers()
        self._sync_vol_spin()

    # Beat-Hintergruende (alle 4 Reihen heller, alle 16 betont) + Farb-Kodierung
    # (Toene cyan, Drums magenta, leere Zellen gedaempft).
    def _style_cell(self, item, c: int, r: int, note) -> None:
        if r % 16 == 0:
            bg = QColor(COLORS["bg_panel"])
        elif r % 4 == 0:
            bg = QColor(COLORS["bg_alt"])
        else:
            bg = QColor(COLORS["bg"])
        item.setBackground(bg)
        if note is None:
            item.setForeground(QColor(COLORS["line_no"]))
        elif c == TONAL:
            item.setForeground(QColor(COLORS["danger"]))
        else:
            item.setForeground(QColor(COLORS["accent"]))

    def _update_channel_headers(self) -> None:
        """Spalten-Header zeigen Kanal + zugewiesenes Instrument/Wellenform."""
        names = ["Ch1", "Ch2", "Ch3", "Drum"]
        labels = []
        for c in range(CHANNELS):
            idx = self.song.channel_inst[c]
            if idx is not None and 0 <= idx < len(self.song.instruments):
                sub = self.song.instruments[idx].name
            elif c == TONAL:
                sub = "noise"
            else:
                sub = self.song.waves[c]
            labels.append(f"{names[c]}\n{sub}")
        self.grid.setHorizontalHeaderLabels(labels)

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
        if hasattr(self, "grid"):
            self._update_channel_headers()

    def _rebuild_sound_combos(self) -> None:
        """Pro-Spur-Sound-Dropdowns aus dem Instrument-Pool fuellen + die
        aktuelle Zuweisung spiegeln."""
        for c in range(CHANNELS):
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
        if 0 <= idx < len(self.song.instruments) and 0 <= c < CHANNELS:
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
            if c < CHANNELS and nm in by_name:
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
        (headless), sonst pygame-Dekodierung (OGG/float-WAV/...). None bei
        Fehler."""
        from .tracker.instrument import load_wav_mono
        try:
            return load_wav_mono(path)
        except Exception:
            pass
        try:
            import os
            os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "hide")
            import pygame
            if pygame.mixer.get_init() is None:
                pygame.mixer.init(frequency=44100, size=-16, channels=2)
            snd = pygame.mixer.Sound(path)
            arr = pygame.sndarray.array(snd)
            if arr.ndim == 2:
                arr = arr.mean(axis=1)
            return (arr.astype(np.float32) / 32768.0,
                    pygame.mixer.get_init()[0] or 44100)
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
        it = self.grid.item(row, ci)
        it.setText(self._cell_text(ci, pat.data[ci][row], pat.vol[ci][row],
                                   pat.slide[ci][row]))
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
    def _row_samples(self) -> int:
        return max(1, int(44100 * self.song.row_ms() / 1000.0))

    def _render_sound(self, inst, midi: int, n_samples: int, slide: int = 0):
        """Pygame-Sound der Note ueber das Instrument (gecacht). n_samples =
        Klanglaenge -> Noten klingen fuer ihre Dauer."""
        key = (id(inst), int(midi), int(n_samples), int(slide or 0))
        snd = self._sound_cache.get(key)
        if snd is None:
            snd = self._make_sound(
                inst.render_note(midi, n_samples, 44100, slide or 0))
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
                   slide: int = 0, n_rows: int | None = None) -> None:
        """Spielt eine Note. `n_rows` = Notenlaenge in Reihen (None = kurze
        Vorhoer-Laenge); so klingen Noten waehrend der Wiedergabe so lange,
        bis die naechste Note kommt."""
        inst = self.song.instrument_for_channel(ci)
        row_s = self._row_samples()
        if n_rows is None:
            n = max(int(44100 * 0.6), row_s * 2)      # Vorhoeren
        else:
            n = max(row_s, row_s * int(n_rows))
        snd = self._render_sound(inst, midi, n, slide or 0)
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
        veraendern). row < 0 = Highlight entfernen."""
        old = self._playhead
        self._playhead = row
        pat = self.song.patterns[self.cur]
        for rr in {old, row}:
            if rr is None or not (0 <= rr < self.grid.rowCount()):
                continue
            for c in range(CHANNELS):
                it = self.grid.item(rr, c)
                if it is None:
                    continue
                if rr == row:
                    it.setBackground(QColor(COLORS["accent_soft"]))
                else:
                    note = pat.data[c][rr] if rr < pat.rows else None
                    self._style_cell(it, c, rr, note)
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

    def _play_columns(self, pat, row: int) -> None:
        for c in range(CHANNELS):
            n = pat.data[c][row]
            if n is not None:
                self._play_note(c, n, pat.vol[c][row], pat.slide[c][row] or 0,
                                self._note_len_rows(pat, c, row))

    # ============================================== Datei
    def _new_song(self) -> None:
        self.song = Song()
        self._install_factory_presets()
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
        self._install_factory_presets()    # alte Songs ohne Instrumente
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

    def _export_audio(self) -> None:
        """Rendert den ganzen Song (inkl. Sample-Instrumente/Loop/ADSR) offline
        zu einer WAV-Datei -- im Spiel via PLAYMUSIC/LOADSOUND abspielbar."""
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
                mix = render_song(self.song)
                save_wav(path, mix)
            finally:
                QApplication.restoreOverrideCursor()
        except Exception as exc:  # noqa: BLE001
            QMessageBox.warning(self, "Fehler",
                                f"Audio-Export fehlgeschlagen:\n{exc}")
            return
        secs = len(mix) / 44100.0
        QMessageBox.information(
            self, "Audio exportiert",
            f"{Path(path).name} ({secs:.1f}s) gerendert.\n\n"
            f"Im Spiel abspielen:  PLAYMUSIC(\"{Path(path).name}\")")

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
