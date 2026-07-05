"""Notenblatt-Editor fuer GameBasic (`gbscore` / `gbrun.py --score`).

Echte Notensatz-Darstellung (5-Linien-System, Violin-/Bassschluessel,
Hilfslinien, Vorzeichen) statt eines Zeilen-Rasters wie im Tracker: Noten per
Klick auf eine Notenzeile setzen, pro Spur GENAU EIN Instrument (kein
Pattern-Zell-Ueberschreiben wie im Tracker -- v1-Vereinfachung), Wiedergabe
ueber den geteilten additiven Mixer (`gamebasic.audio_preview.Mixer`, auch
vom Tracker genutzt), eigenes `*.json`-Format ODER Export/Uebernahme in den
Tracker (`gamebasic.score.convert.to_tracker_song`).

Das Datenmodell + die Tracker-Konvertierung liegen Qt-frei in
`gamebasic.score` (headless getestet).

**V1-Limitationen** (siehe auch CLAUDE.md/docs/score-editor.md): festes
4/4-Metrum, ein Instrument pro Spur, Akkorde werden beim Tracker-Export auf
die hoechste Note reduziert, Vorzeichen werden immer als Kreuz der Stammnote
darunter notiert (nie als B), Achtel/Sechzehntel bekommen nur Faehnchen statt
Balken-Gruppierung, kein Schlagzeug-Spurtyp.
"""
from __future__ import annotations

from pathlib import Path

import numpy as np
from PySide6.QtCore import QPointF, QRectF, QSize, Qt, QTimer
from PySide6.QtGui import QColor, QFont, QKeySequence, QPainter, QPen, QShortcut
from PySide6.QtWidgets import (
    QApplication, QButtonGroup, QCheckBox, QComboBox, QFileDialog, QGroupBox,
    QHBoxLayout, QLabel, QLineEdit, QMainWindow, QMessageBox, QPushButton,
    QScrollArea, QSpinBox, QToolButton, QVBoxLayout, QWidget,
)

from .audio_preview import Mixer
from .editor_qt.theme import COLORS, EDITOR_FONT_FAMILY, global_qss
from .score.convert import to_tracker_song
from .score.document import ScoreDoc

# Diatonische Halbtonversaetze der 7 Stammtoene (C,D,E,F,G,A,B) innerhalb
# einer Oktave -- Notenlinien sind DIATONISCH gleich verteilt (C->D und
# E->F belegen je EINEN Schritt), nicht halbtonlinear. Siehe CLAUDE.md.
NATURAL_MIDI = (0, 2, 4, 5, 7, 9, 11)
# MIDI-Ton der untersten Notenlinie (Linie 1) je Notenschluessel.
CLEF_REF = {"treble": 64, "bass": 43}  # E4 / G2
# MIDI-Ton der MITTLEREN Notenlinie (Linie 3) je Notenschluessel -- Ziel-
# Zentrum fuer den optionalen Oktav-Transpose beim Schluesselwechsel.
CLEF_CENTER = {"treble": 71, "bass": 50}  # B4 / D3

DURATIONS = [
    ("Ganze", 4.0), ("Halbe", 2.0), ("Viertel", 1.0),
    ("Achtel", 0.5), ("Sechzehntel", 0.25),
]

_TRACK_HUES = ("accent", "success", "danger", "warning",
              "kw_ctrl", "kw_decl", "string", "number")


def _track_color(i: int) -> str:
    return COLORS[_TRACK_HUES[i % len(_TRACK_HUES)]]


def _dia_index(midi: int) -> int:
    """Diatonischer Index (streng monoton mit der Tonhoehe) -- gemeinsamer
    Bezugsrahmen fuer beide Notenschluessel. Oktavkonvention wie
    `tracker.song.note_name` (`m // 12 - 1`, MIDI 60 = "C4")."""
    pc = midi % 12
    octave = midi // 12 - 1
    natural_index = (NATURAL_MIDI.index(pc) if pc in NATURAL_MIDI
                      else NATURAL_MIDI.index(pc - 1))
    return octave * 7 + natural_index


def _has_accidental(midi: int) -> bool:
    return midi % 12 not in NATURAL_MIDI


class _StaffView(QWidget):
    """Notensystem EINER Spur: Zeichnen + Klick-zu-Note/Pause."""

    LINE_SPACING = 15.0
    STEP_PX = LINE_SPACING / 2.0
    LEFT_MARGIN = 76.0
    TOP_MARGIN = 84.0
    PIXELS_PER_BEAT = 64.0
    NOTE_R = 6.5
    STEM_LEN = 34.0
    # Taktstriche werden etwas VOR ihrer exakten Beat-Position gezeichnet,
    # damit eine Note genau auf dem Taktanfang (z.B. Beat 4.0) nicht direkt
    # auf der Linie sitzt -- reine Zeichenposition, beeinflusst NICHT die
    # Beat<->X-Zuordnung/Snap-Logik.
    BARLINE_LEAD = 8.0

    def __init__(self, editor: "ScoreEditor", track_index: int, parent=None):
        super().__init__(parent)
        self.editor = editor
        self.track_index = track_index
        self.playhead_beat: float | None = None
        # Vorschau-Cursor: (beat, pitch|None) an der aktuellen Mausposition,
        # damit man vor dem Klick sieht, wo/welche Note gesetzt wuerde --
        # sonst aendert man leicht versehentlich eine bestehende Note.
        self.hover_pos: tuple[float, int | None] | None = None
        self.setMouseTracking(True)
        self.setMinimumHeight(250)

    @property
    def track(self):
        return self.editor.doc.tracks[self.track_index]

    # ---- Geometrie: Tonhoehe <-> Y (Abschnitt 4.1 im Implementierungsplan) --
    def _dia_ref(self) -> int:
        return _dia_index(CLEF_REF[self.track.clef])

    def _y_ref(self) -> float:
        """Y-Koordinate der untersten Notenlinie (Linie 1, step_offset=0)."""
        return self.TOP_MARGIN + 8 * self.STEP_PX

    def _step_offset(self, midi: int) -> int:
        return _dia_index(midi) - self._dia_ref()

    def _y_for_step(self, step_offset: int | float) -> float:
        return self._y_ref() - step_offset * self.STEP_PX

    def _y_for_pitch(self, midi: int) -> float:
        return self._y_for_step(self._step_offset(midi))

    def _pitch_for_y(self, y: float, accidental: int) -> int:
        step_offset = round((self._y_ref() - y) / self.STEP_PX)
        dia = self._dia_ref() + step_offset
        octave, natural_index = divmod(dia, 7)
        natural = (octave + 1) * 12 + NATURAL_MIDI[natural_index]
        return natural + accidental

    @staticmethod
    def _ledger_steps(step_offset: int) -> list[int]:
        """Hilfslinien-Positionen (nur an geraden step_offset-Werten, d.h.
        Linien-Positionen -- niemals an Zwischenraum-Positionen)."""
        steps: list[int] = []
        if step_offset > 8:
            s = 10
            while s <= step_offset:
                steps.append(s)
                s += 2
        elif step_offset < 0:
            s = -2
            while s >= step_offset:
                steps.append(s)
                s -= 2
        return steps

    # ---- Geometrie: Zeit <-> X (Abschnitt 4.2) -----------------------------
    def _x_for_beat(self, beat: float) -> float:
        return self.LEFT_MARGIN + beat * self.PIXELS_PER_BEAT

    def _beat_for_x(self, x: float) -> float:
        return max(0.0, (x - self.LEFT_MARGIN) / self.PIXELS_PER_BEAT)

    def _snap_beat(self, beat_raw: float) -> float:
        """Die aktuell gewaehlte Notendauer IST die Rasterauflösung -- v1
        hat kein davon unabhaengiges Raster (siehe Implementierungsplan)."""
        dur = self.editor.entry_duration_beats()
        if dur <= 0:
            return max(0.0, beat_raw)
        return max(0.0, round(beat_raw / dur) * dur)

    def _content_beats(self) -> float:
        return max(8.0, self.track.length_beats() + 4.0)

    def _content_width(self) -> int:
        return int(self._x_for_beat(self._content_beats())) + 40

    def sizeHint(self) -> QSize:  # noqa: N802
        return QSize(max(800, self._content_width()), 260)

    def minimumSizeHint(self) -> QSize:  # noqa: N802
        return QSize(400, 250)

    # ---- Balken-Gruppierung (Achtel/Sechzehntel statt Einzel-Faehnchen) ----
    def _beam_groups(self) -> list[list]:
        """Liefert Gruppen von >=2 zusammenhaengenden Noten (gleiche Dauer,
        Achtel oder kuerzer, im selben Beat, ohne Luecke) -- die bekommen
        einen gemeinsamen Balken statt je ein Faehnchen. V1-Vereinfachung:
        nur Laeufe GLEICHER Dauer werden gebalkt (kein Mischen von Achtel+
        Sechzehntel in einer Gruppe/keine Partial-Balken)."""
        notes = sorted(
            (n for n in self.track.notes if not n.rest and n.dur_beat <= 0.5),
            key=lambda n: n.start_beat)
        groups: list[list] = []
        current: list = []
        for n in notes:
            if current:
                prev = current[-1]
                same_beat = int(prev.start_beat) == int(n.start_beat)
                contiguous = abs(prev.end_beat - n.start_beat) < 1e-6
                same_dur = abs(prev.dur_beat - n.dur_beat) < 1e-6
                if same_beat and contiguous and same_dur:
                    current.append(n)
                    continue
                if len(current) >= 2:
                    groups.append(current)
                current = []
            current = [n]
        if len(current) >= 2:
            groups.append(current)
        return groups

    def _draw_beam_group(self, p: QPainter, group: list) -> None:
        steps = [self._step_offset(n.pitch) for n in group]
        ys = [self._y_for_step(s) for s in steps]
        xs = [self._x_for_beat(n.start_beat) for n in group]
        stem_up = (sum(steps) / len(steps)) < 4
        color = QColor(_track_color(self.track_index))
        n_beams = 2 if group[0].dur_beat <= 0.25 else 1
        beam_y = (min(ys) - self.STEM_LEN) if stem_up else (max(ys) + self.STEM_LEN)

        p.setPen(QPen(color, 1.4))
        for x, y in zip(xs, ys):
            sx = x + self.NOTE_R if stem_up else x - self.NOTE_R
            p.drawLine(int(sx), int(y), int(sx), int(beam_y))

        sx0 = xs[0] + (self.NOTE_R if stem_up else -self.NOTE_R)
        sx1 = xs[-1] + (self.NOTE_R if stem_up else -self.NOTE_R)
        for b in range(n_beams):
            by = beam_y + b * (5 if stem_up else -5)
            p.setPen(QPen(color, 3.4))
            p.drawLine(int(sx0), int(by), int(sx1), int(by))

    # ---- Rendering ----------------------------------------------------------
    def paintEvent(self, e) -> None:  # noqa: N802
        p = QPainter(self)
        p.fillRect(self.rect(), QColor(COLORS["bg"]))
        w = max(self.width(), self._content_width())
        y_top = self._y_for_step(8)
        y_bot = self._y_for_step(0)
        track_color = QColor(_track_color(self.track_index))

        # Abwechselnde Takt-Schattierung (Kontrast-/Lesbarkeitshilfe, wie die
        # Beat-Schattierung im Tracker-Gitter).
        n_measures = int(self._content_beats() // 4) + 1
        band = QColor(COLORS["bg_alt"])
        for m in range(0, n_measures + 1, 2):
            x0 = int(self._x_for_beat(m * 4))
            x1 = int(self._x_for_beat((m + 1) * 4))
            p.fillRect(x0, int(y_top) - 20, x1 - x0, int(y_bot - y_top) + 40, band)

        p.setPen(QPen(QColor(COLORS["fg_muted"]), 1.3))
        for i in range(5):
            y = self._y_for_step(i * 2)
            p.drawLine(int(self.LEFT_MARGIN - 14), int(y), w, int(y))

        p.setPen(track_color)
        f = QFont(EDITOR_FONT_FAMILY)
        f.setPointSize(36)
        p.setFont(f)
        glyph = "\U0001D11E" if self.track.clef == "treble" else "\U0001D122"
        p.drawText(QRectF(4, y_top - 34, self.LEFT_MARGIN - 10,
                          (y_bot - y_top) + 68),
                   Qt.AlignmentFlag.AlignCenter, glyph)

        # Erster Taktstrich wird bewusst NICHT gezeichnet (sonst kollidiert er
        # mit einer Note direkt am Anfang) -- ab dem 2. Takt jeweils leicht
        # VOR der exakten Beat-Position, damit Noten auf dem Taktanfang nicht
        # direkt auf der Linie sitzen (siehe BARLINE_LEAD).
        p.setPen(QPen(QColor(COLORS["fg_muted"]), 1.3))
        for m in range(1, n_measures + 1):
            x = int(self._x_for_beat(m * 4) - self.BARLINE_LEAD)
            p.drawLine(x, int(y_top) - 6, x, int(y_bot) + 6)

        beam_groups = self._beam_groups()
        beamed_ids = {id(n) for g in beam_groups for n in g}
        for note in self.track.notes:
            self._draw_note(p, note, beamed=id(note) in beamed_ids)
        for group in beam_groups:
            self._draw_beam_group(p, group)

        if self.playhead_beat is not None:
            x = int(self._x_for_beat(self.playhead_beat))
            p.setPen(QPen(QColor(COLORS["danger"]), 2.4))
            p.drawLine(x, int(y_top) - 12, x, int(y_bot) + 12)

        self._draw_hover_preview(p, y_top, y_bot)
        p.end()

    def _draw_hover_preview(self, p: QPainter, y_top: float, y_bot: float) -> None:
        """Vorschau-Cursor an der aktuellen Mausposition -- zeigt VOR dem
        Klick, an welcher Zeit/Tonhoehe die Note (bzw. Pause) landen wuerde,
        halbtransparent damit bestehende Noten nicht verdeckt werden."""
        if self.hover_pos is None:
            return
        beat, pitch = self.hover_pos
        x = self._x_for_beat(beat)
        color = QColor(_track_color(self.track_index))
        color.setAlpha(120)

        p.setPen(QPen(color, 1.2, Qt.PenStyle.DashLine))
        p.drawLine(int(x), int(y_top) - 18, int(x), int(y_bot) + 18)

        if pitch is None:
            y = self._y_for_step(4)
            p.setPen(QPen(color, 2.2))
            p.drawRect(int(x) - 5, int(y) - 3, 10, 6)
            return

        step = self._step_offset(pitch)
        y = self._y_for_step(step)
        for s in self._ledger_steps(step):
            ly = self._y_for_step(s)
            p.setPen(QPen(color, 1.3))
            p.drawLine(int(x - 12), int(ly), int(x + 12), int(ly))
        p.setPen(QPen(color, 1.6))
        p.setBrush(color)
        p.drawEllipse(QPointF(x, y), self.NOTE_R, self.NOTE_R)
        p.setBrush(Qt.BrushStyle.NoBrush)

    def _draw_note(self, p: QPainter, note, beamed: bool = False) -> None:
        x = self._x_for_beat(note.start_beat)
        color = QColor(_track_color(self.track_index))
        if note.rest:
            y = self._y_for_step(4)
            p.setPen(QPen(color, 2.2))
            p.drawRect(int(x) - 5, int(y) - 3, 10, 6)
            return

        step = self._step_offset(note.pitch)
        y = self._y_for_step(step)
        filled = note.dur_beat < 2.0

        ledger_color = color.lighter(115)
        for s in self._ledger_steps(step):
            ly = self._y_for_step(s)
            p.setPen(QPen(ledger_color, 1.4))
            p.drawLine(int(x - 12), int(ly), int(x + 12), int(ly))

        p.setPen(QPen(color, 1.8))
        p.setBrush(color if filled else Qt.BrushStyle.NoBrush)
        p.drawEllipse(QPointF(x, y), self.NOTE_R, self.NOTE_R)
        p.setBrush(Qt.BrushStyle.NoBrush)

        if note.dur_beat < 4.0 and not beamed:
            stem_up = step < 4
            p.setPen(QPen(color, 1.6))
            if stem_up:
                sx = x + self.NOTE_R
                p.drawLine(int(sx), int(y), int(sx), int(y - self.STEM_LEN))
            else:
                sx = x - self.NOTE_R
                p.drawLine(int(sx), int(y), int(sx), int(y + self.STEM_LEN))
            if note.dur_beat <= 0.5:
                n_flags = 2 if note.dur_beat <= 0.25 else 1
                fy = (y - self.STEM_LEN) if stem_up else (y + self.STEM_LEN)
                for k in range(n_flags):
                    off = k * 7 * (1 if stem_up else -1)
                    p.drawLine(int(sx), int(fy + off),
                              int(sx + 9), int(fy + off + 7))

        if _has_accidental(note.pitch):
            p.setPen(QColor(COLORS["warning"]))
            f2 = QFont(EDITOR_FONT_FAMILY)
            f2.setPointSize(14)
            p.setFont(f2)
            p.drawText(QRectF(x - 28, y - 12, 20, 24),
                      Qt.AlignmentFlag.AlignCenter, "♯")

    # ---- Klick-Eingabe --------------------------------------------------
    def _existing_at(self, beat: float):
        for n in self.track.notes:
            if abs(n.start_beat - beat) < 1e-6:
                return n
        return None

    def mousePressEvent(self, e) -> None:  # noqa: N802
        beat = self._snap_beat(self._beat_for_x(e.position().x()))
        if e.button() == Qt.MouseButton.RightButton:
            existing = self._existing_at(beat)
            if existing is not None:
                self.track.remove_note(existing)
                self.editor._mark_dirty()
                self.update()
            return
        if self.editor.entry_rest:
            self._place(beat, rest=True)
        else:
            pitch = self._pitch_for_y(e.position().y(),
                                      self.editor.entry_accidental)
            self._place(beat, pitch=pitch)

    def mouseMoveEvent(self, e) -> None:  # noqa: N802
        beat = self._snap_beat(self._beat_for_x(e.position().x()))
        if self.editor.entry_rest:
            self.hover_pos = (beat, None)
        else:
            pitch = self._pitch_for_y(e.position().y(),
                                      self.editor.entry_accidental)
            self.hover_pos = (beat, pitch)
        self.update()

    def leaveEvent(self, e) -> None:  # noqa: N802
        self.hover_pos = None
        self.update()

    def _place(self, beat: float, pitch: int | None = None,
               rest: bool = False) -> None:
        existing = self._existing_at(beat)
        same = (existing is not None and existing.rest == rest
                and (rest or existing.pitch == pitch))
        if existing is not None:
            self.track.remove_note(existing)
        if not same:
            dur = self.editor.entry_duration_beats()
            self.track.add_note(beat, dur, pitch=pitch, rest=rest)
        self.editor._mark_dirty()
        self.updateGeometry()
        self.update()


class ScoreEditor(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = Path(project_root)
        self.setWindowTitle("GameBasic Notenblatt-Editor")
        self.resize(1150, 780)
        self.doc = ScoreDoc()
        self._mixer = Mixer()
        self._sound_cache: dict = {}
        self.entry_accidental = 0
        self.entry_rest = False
        self._dirty = False
        self._playing = False
        self._play_beat = 0.0
        self._play_timer = QTimer(self)
        self._play_timer.timeout.connect(self._play_tick)
        self._track_rows: list[dict] = []

        central = QWidget()
        self.setCentralWidget(central)
        root = QVBoxLayout(central)
        root.setContentsMargins(10, 8, 10, 8)
        root.setSpacing(7)

        title = QLabel("♪  GameBasic Notenblatt-Editor")
        tf = QFont()
        tf.setBold(True)
        tf.setPointSize(13)
        title.setFont(tf)
        title.setStyleSheet(f"color: {COLORS['accent']}; padding: 2px 0;")
        root.addWidget(title)

        root.addLayout(self._build_toolbar())
        root.addLayout(self._build_track_bar())

        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.tracks_panel = QWidget()
        self.tracks_layout = QVBoxLayout(self.tracks_panel)
        self.tracks_layout.setSpacing(14)
        self.scroll.setWidget(self.tracks_panel)
        root.addWidget(self.scroll, 1)

        self._rebuild_tracks_ui()
        self._build_menu()
        self.status = self.statusBar()
        self._update_info()
        QShortcut(QKeySequence("F11"), self, activated=self._toggle_fullscreen)

    def _toggle_fullscreen(self) -> None:
        if self.isFullScreen():
            self.showMaximized()
        else:
            self.showFullScreen()

    # ---- UI-Aufbau --------------------------------------------------------
    def _build_toolbar(self) -> QHBoxLayout:
        tb = QHBoxLayout()
        tb.setSpacing(8)
        big_font = QFont(EDITOR_FONT_FAMILY)
        big_font.setPointSize(13)

        lbl_dur = QLabel("Dauer:")
        lbl_dur.setFont(big_font)
        tb.addWidget(lbl_dur)
        self.dur_combo = QComboBox()
        self.dur_combo.setFont(big_font)
        self.dur_combo.setMinimumHeight(34)
        for name, beats in DURATIONS:
            self.dur_combo.addItem(name, beats)
        self.dur_combo.setCurrentIndex(2)   # Viertel
        self.dur_combo.currentIndexChanged.connect(lambda _i: self._update_info())
        tb.addWidget(self.dur_combo)
        self.dotted_check = QCheckBox("Punktiert")
        self.dotted_check.setFont(big_font)
        self.dotted_check.toggled.connect(lambda _c: self._update_info())
        tb.addWidget(self.dotted_check)

        tb.addSpacing(12)
        self.acc_group = QButtonGroup(self)
        self.acc_group.setExclusive(True)
        acc_font = QFont(EDITOR_FONT_FAMILY)
        acc_font.setPointSize(20)
        acc_font.setBold(True)
        self.btn_natural = QToolButton()
        self.btn_natural.setText("♮")
        self.btn_natural.setToolTip("Vorzeichen: natuerlich (kein Halbtonversatz)")
        self.btn_sharp = QToolButton()
        self.btn_sharp.setText("♯")
        self.btn_sharp.setToolTip("Vorzeichen: Kreuz (+1 Halbton)")
        self.btn_flat = QToolButton()
        self.btn_flat.setText("♭")
        self.btn_flat.setToolTip("Vorzeichen: B (-1 Halbton)")
        for gid, b in enumerate((self.btn_natural, self.btn_sharp, self.btn_flat)):
            b.setCheckable(True)
            b.setFont(acc_font)
            b.setFixedSize(46, 40)
            self.acc_group.addButton(b, gid)
            tb.addWidget(b)
        self.btn_natural.setChecked(True)
        self.acc_group.idClicked.connect(self._on_accidental_changed)

        tb.addSpacing(12)
        self.rest_check = QCheckBox("Pause")
        self.rest_check.setFont(big_font)
        self.rest_check.toggled.connect(self._on_rest_toggled)
        tb.addWidget(self.rest_check)

        tb.addSpacing(20)
        lbl_bpm = QLabel("Tempo (BPM):")
        lbl_bpm.setFont(big_font)
        tb.addWidget(lbl_bpm)
        self.bpm_spin = QSpinBox()
        self.bpm_spin.setFont(big_font)
        self.bpm_spin.setMinimumHeight(34)
        self.bpm_spin.setRange(40, 300)
        self.bpm_spin.setValue(self.doc.bpm)
        self.bpm_spin.valueChanged.connect(self._on_bpm_changed)
        tb.addWidget(self.bpm_spin)

        tb.addStretch(1)
        self.btn_play = QPushButton("▶ Abspielen")
        self.btn_play.setProperty("accent", True)
        self.btn_play.setFont(big_font)
        self.btn_play.setMinimumHeight(38)
        self.btn_play.setMinimumWidth(140)
        self.btn_play.clicked.connect(self._toggle_play)
        tb.addWidget(self.btn_play)

        b_tracker = QPushButton("In Tracker oeffnen")
        b_tracker.setFont(big_font)
        b_tracker.setMinimumHeight(38)
        b_tracker.setToolTip(
            "Konvertiert das Stueck in ein Tracker-Projekt (ein Kanal pro "
            "Spur + Instrument) und oeffnet es in gbtracker")
        b_tracker.clicked.connect(self._export_to_tracker)
        tb.addWidget(b_tracker)
        return tb

    def _build_track_bar(self) -> QHBoxLayout:
        bar = QHBoxLayout()
        b_add = QPushButton("+ Spur")
        b_add.clicked.connect(self._add_track)
        bar.addWidget(b_add)
        b_remove = QPushButton("- Spur")
        b_remove.clicked.connect(self._remove_last_track)
        bar.addWidget(b_remove)
        bar.addStretch(1)
        return bar

    def _build_menu(self) -> None:
        menu = self.menuBar().addMenu("&Datei")
        menu.addAction("Neu", self._new_doc)
        menu.addAction("Oeffnen...", self._open)
        menu.addAction("Speichern...", self._save)

    def _rebuild_tracks_ui(self) -> None:
        while self.tracks_layout.count():
            item = self.tracks_layout.takeAt(0)
            w = item.widget()
            if w is not None:
                w.deleteLater()
        self._track_rows = []
        from .tracker.presets import preset_names
        names = preset_names()
        for i, track in enumerate(self.doc.tracks):
            box = QGroupBox(track.name)
            box.setStyleSheet(
                f"QGroupBox {{ color: {_track_color(i)}; font-weight: bold; }}")
            v = QVBoxLayout(box)
            header = QHBoxLayout()
            header.addWidget(QLabel("Name:"))
            name_edit = QLineEdit(track.name)
            name_edit.editingFinished.connect(
                lambda idx=i, e=name_edit: self._on_track_name_changed(idx, e))
            header.addWidget(name_edit)
            header.addWidget(QLabel("Schluessel:"))
            clef_combo = QComboBox()
            clef_combo.addItem("Violinschluessel", "treble")
            clef_combo.addItem("Bassschluessel", "bass")
            clef_combo.setCurrentIndex(0 if track.clef == "treble" else 1)
            clef_combo.currentIndexChanged.connect(
                lambda _idx, idx=i, c=clef_combo: self._on_clef_changed(idx, c))
            header.addWidget(clef_combo)
            header.addWidget(QLabel("Instrument:"))
            inst_combo = QComboBox()
            inst_combo.addItems(names)
            if track.instrument.name in names:
                inst_combo.setCurrentIndex(names.index(track.instrument.name))
            inst_combo.currentIndexChanged.connect(
                lambda idx, ti=i: self._on_instrument_changed(ti, idx))
            header.addWidget(inst_combo)
            header.addStretch(1)
            v.addLayout(header)
            staff = _StaffView(self, i)
            v.addWidget(staff)
            self.tracks_layout.addWidget(box)
            self._track_rows.append({
                "box": box, "staff": staff, "name_edit": name_edit,
                "clef_combo": clef_combo, "inst_combo": inst_combo,
            })
        self.tracks_layout.addStretch(1)

    # ---- Toolbar-Zustand ----------------------------------------------------
    def entry_duration_beats(self) -> float:
        base = float(self.dur_combo.currentData())
        return base * 1.5 if self.dotted_check.isChecked() else base

    def _on_accidental_changed(self, gid: int) -> None:
        self.entry_accidental = {0: 0, 1: 1, 2: -1}.get(gid, 0)
        self._update_info()

    def _on_rest_toggled(self, checked: bool) -> None:
        self.entry_rest = checked
        self._update_info()

    def _update_info(self) -> None:
        """Info-Leiste (Statusbar): aktueller Eingabe-Modus + Stueck-Ueberblick
        + kurze Bedienhinweise -- aktualisiert sich bei jeder relevanten
        Aenderung (siehe `_mark_dirty` + Toolbar-Handler)."""
        if not hasattr(self, "status"):
            return
        dur_name = self.dur_combo.currentText()
        dotted = " punktiert" if self.dotted_check.isChecked() else ""
        acc = {0: "natuerlich", 1: "Kreuz (+1)", -1: "B (-1)"}.get(
            self.entry_accidental, "natuerlich")
        mode = "Pause" if self.entry_rest else "Note"
        beats = self.doc.length_beats()
        self.status.showMessage(
            f"Eingabe: {mode} · {dur_name}{dotted} · Vorzeichen {acc}   |   "
            f"{len(self.doc.tracks)} Spur(en) · {beats:g} Beats "
            f"(~{beats / 4.0:.1f} Takte) bei {self.doc.bpm} BPM   |   "
            f"Linksklick: setzen/entfernen · Rechtsklick: entfernen · "
            f"F11: Vollbild")

    def _on_bpm_changed(self, v: int) -> None:
        self.doc.bpm = int(v)
        self._mark_dirty()

    def _on_track_name_changed(self, idx: int, edit: QLineEdit) -> None:
        name = edit.text().strip() or f"Stimme {idx + 1}"
        self.doc.tracks[idx].name = name
        self._track_rows[idx]["box"].setTitle(name)
        self._mark_dirty()

    @staticmethod
    def _octave_shift_for_clef(track, new_clef: str) -> int:
        """Oktav-Versatz (Vielfaches von 12 Halbtoenen), der den Notendurch-
        schnitt der Spur moeglichst nah an die Mittellinie des NEUEN
        Schluessels rueckt -- volle Oktaven, damit Melodie/Intervalle exakt
        erhalten bleiben (nur das Register aendert sich)."""
        pitched = [n.pitch for n in track.notes if not n.rest]
        if not pitched:
            return 0
        avg = sum(pitched) / len(pitched)
        center = CLEF_CENTER.get(new_clef, 71)
        return int(round((center - avg) / 12) * 12)

    def _on_clef_changed(self, idx: int, combo: QComboBox) -> None:
        new_clef = combo.currentData()
        track = self.doc.tracks[idx]
        if new_clef == track.clef:
            return
        shift = self._octave_shift_for_clef(track, new_clef)
        track.clef = new_clef
        if shift != 0:
            n_oct = shift // 12
            reply = QMessageBox.question(
                self, "Noten transponieren?",
                f"Die vorhandenen Noten dieser Spur liegen beim neuen "
                f"Notenschluessel weit ab vom System.\n\n"
                f"Um {n_oct:+d} Oktave(n) verschieben, damit sie wieder "
                f"nahe am System liegen? (Melodie/Intervalle bleiben exakt "
                f"erhalten, nur das Register aendert sich.)",
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
                QMessageBox.StandardButton.Yes)
            if reply == QMessageBox.StandardButton.Yes:
                for n in track.notes:
                    if not n.rest:
                        n.pitch += shift
                self._sound_cache.clear()
        self._track_rows[idx]["staff"].update()
        self._mark_dirty()

    def _on_instrument_changed(self, idx: int, combo_idx: int) -> None:
        from .tracker.presets import factory_instruments
        insts = factory_instruments()
        if 0 <= combo_idx < len(insts):
            self.doc.tracks[idx].instrument = insts[combo_idx]
            self._sound_cache.clear()
            self._mark_dirty()

    def _add_track(self) -> None:
        self.doc.add_track()
        self._rebuild_tracks_ui()
        self._mark_dirty()

    def _remove_last_track(self) -> None:
        if len(self.doc.tracks) > 1:
            self.doc.remove_track(len(self.doc.tracks) - 1)
            self._rebuild_tracks_ui()
            self._mark_dirty()

    def _mark_dirty(self) -> None:
        self._dirty = True
        self._update_info()

    # ---- Wiedergabe ---------------------------------------------------------
    def _toggle_play(self) -> None:
        if self._playing:
            self._stop_play()
        else:
            self._start_play()

    def _start_play(self) -> None:
        self._play_beat = 0.0
        self._playing = True
        self.btn_play.setText("■ Stop")
        interval_ms = max(15, int(60000 / max(1, self.doc.bpm) / 4))
        self._play_timer.setInterval(interval_ms)
        self._play_timer.start()

    def _stop_play(self) -> None:
        self._playing = False
        self._play_timer.stop()
        self.btn_play.setText("▶ Abspielen")
        for row in self._track_rows:
            row["staff"].playhead_beat = None
            row["staff"].update()

    def _play_tick(self) -> None:
        step = 0.25   # eine Sechzehntel pro Tick (ROWS_PER_BEAT-Aequivalent)
        for i, track in enumerate(self.doc.tracks):
            for note in track.notes:
                if not note.rest and abs(note.start_beat - self._play_beat) < step / 2:
                    self._trigger_note(track, note)
            if i < len(self._track_rows):
                self._track_rows[i]["staff"].playhead_beat = self._play_beat
                self._track_rows[i]["staff"].update()
        self._play_beat += step
        if self._play_beat > self.doc.length_beats() + step:
            self._stop_play()

    def _trigger_note(self, track, note) -> None:
        sr = 44100
        seconds = note.dur_beat * 60.0 / max(1, self.doc.bpm)
        n_samples = max(1, int(sr * seconds))
        key = (id(track.instrument), int(note.pitch), n_samples)
        arr = self._sound_cache.get(key)
        if arr is None:
            wave = track.instrument.render_note(note.pitch, n_samples, sr)
            arr = np.clip(wave, -1.0, 1.0).astype(np.float32) * 0.6
            self._sound_cache[key] = arr
        self._mixer.play(arr)

    # ---- Datei --------------------------------------------------------------
    def _new_doc(self) -> None:
        self._stop_play()
        self.doc = ScoreDoc()
        self._sound_cache.clear()
        self.bpm_spin.setValue(self.doc.bpm)
        self._rebuild_tracks_ui()
        self.setWindowTitle("GameBasic Notenblatt-Editor")
        self._dirty = False

    def _open(self) -> None:
        path, _ = QFileDialog.getOpenFileName(
            self, "Notenblatt oeffnen", str(self.project_root),
            "GameBasic-Notenblatt (*.json)")
        if not path:
            return
        try:
            self.doc = ScoreDoc.load_json(path)
        except Exception as exc:  # noqa: BLE001
            QMessageBox.warning(self, "Fehler", f"Konnte nicht laden:\n{exc}")
            return
        self._stop_play()
        self._sound_cache.clear()
        self.bpm_spin.setValue(self.doc.bpm)
        self._rebuild_tracks_ui()
        self.setWindowTitle(f"GameBasic Notenblatt-Editor -- {Path(path).name}")
        self._dirty = False

    def _save(self) -> None:
        path, _ = QFileDialog.getSaveFileName(
            self, "Notenblatt speichern", str(self.project_root),
            "GameBasic-Notenblatt (*.json)")
        if not path:
            return
        if not path.lower().endswith(".json"):
            path += ".json"
        self.doc.save_json(path)
        self.setWindowTitle(f"GameBasic Notenblatt-Editor -- {Path(path).name}")
        self._dirty = False

    # ---- Tracker-Export -------------------------------------------------
    def _export_to_tracker(self) -> None:
        try:
            song, warn = to_tracker_song(self.doc)
        except ValueError as exc:
            QMessageBox.warning(self, "Export nicht moeglich", str(exc))
            return
        if warn:
            shown = "\n".join(warn[:30]) + ("\n..." if len(warn) > 30 else "")
            QMessageBox.information(self, "Hinweise beim Export", shown)
        path, _ = QFileDialog.getSaveFileName(
            self, "Als Tracker-Projekt speichern", str(self.project_root),
            "Tracker-Projekt (*.json)")
        if not path:
            return
        if not path.lower().endswith(".json"):
            path += ".json"
        song.save_json(path)
        gbrun = self.project_root / "gbrun.py"
        try:
            import subprocess
            import sys
            subprocess.Popen([sys.executable, str(gbrun), "--tracker", path])
        except Exception:
            QMessageBox.information(
                self, "Exportiert",
                f"Als Tracker-Projekt gespeichert:\n{path}\n\n"
                "Oeffne es manuell mit gbtracker.")


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance()
    if app is None:
        app = QApplication([])
        app.setStyle("Fusion")
    app.setStyleSheet(global_qss())
    win = ScoreEditor(project_root)
    if initial_file and Path(initial_file).exists():
        try:
            win.doc = ScoreDoc.load_json(str(initial_file))
            win.bpm_spin.setValue(win.doc.bpm)
            win._rebuild_tracks_ui()
        except Exception:
            pass
    win.showMaximized()          # "ordentlich Platz" -- wie Audio Studio
    return app.exec()
