"""Tests fuer den Notenblatt-Editor (offscreen, gamebasic.scoreeditor_qt)."""
import os
from pathlib import Path

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _editor():
    try:
        from gamebasic.scoreeditor_qt import ScoreEditor
        return ScoreEditor(Path("."))
    except Exception as exc:  # pragma: no cover
        pytest.skip(f"Editor nicht konstruierbar: {exc}")


def _click(staff, x, y, button="left"):
    from PySide6.QtCore import Qt, QPointF
    from PySide6.QtGui import QMouseEvent
    btn = Qt.MouseButton.LeftButton if button == "left" else Qt.MouseButton.RightButton
    ev = QMouseEvent(QMouseEvent.Type.MouseButtonPress, QPointF(x, y),
                     btn, btn, Qt.KeyboardModifier.NoModifier)
    staff.mousePressEvent(ev)


def _move(staff, x, y):
    from PySide6.QtCore import Qt, QPointF
    from PySide6.QtGui import QMouseEvent
    ev = QMouseEvent(QMouseEvent.Type.MouseMove, QPointF(x, y),
                     Qt.MouseButton.NoButton, Qt.MouseButton.NoButton,
                     Qt.KeyboardModifier.NoModifier)
    staff.mouseMoveEvent(ev)


def test_hover_pos_tracks_mouse_position():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    x = staff._x_for_beat(2.0)
    y = staff._y_for_pitch(67)
    _move(staff, x, y)
    assert staff.hover_pos is not None
    beat, pitch = staff.hover_pos
    assert abs(beat - 2.0) < 1e-6
    assert pitch == 67


def test_hover_pos_shows_rest_when_pause_active():
    ed = _editor()
    ed.rest_check.setChecked(True)
    staff = ed._track_rows[0]["staff"]
    _move(staff, staff._x_for_beat(1.0), staff._y_for_pitch(60))
    beat, pitch = staff.hover_pos
    assert pitch is None
    assert abs(beat - 1.0) < 1e-6


def test_hover_pos_cleared_on_leave():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _move(staff, staff._x_for_beat(1.0), staff._y_for_pitch(60))
    assert staff.hover_pos is not None
    from PySide6.QtCore import QEvent
    staff.leaveEvent(QEvent(QEvent.Type.Leave))
    assert staff.hover_pos is None


def test_editor_constructs_with_one_default_track():
    ed = _editor()
    assert len(ed.doc.tracks) == 1
    assert len(ed._track_rows) == 1


def test_add_and_remove_track_updates_doc_and_ui():
    ed = _editor()
    ed._add_track()
    assert len(ed.doc.tracks) == 2
    assert len(ed._track_rows) == 2

    ed._remove_last_track()
    assert len(ed.doc.tracks) == 1
    assert len(ed._track_rows) == 1

    # Letzte Spur kann nicht entfernt werden.
    ed._remove_last_track()
    assert len(ed.doc.tracks) == 1


def test_click_places_note_at_expected_pitch_and_beat():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    x = staff._x_for_beat(2.0)
    y = staff._y_for_pitch(67)     # G4 -- Linie 2 im Violinschluessel
    _click(staff, x, y)

    assert len(ed.doc.tracks[0].notes) == 1
    note = ed.doc.tracks[0].notes[0]
    assert note.pitch == 67
    assert abs(note.start_beat - 2.0) < 1e-6
    assert note.rest is False


def test_click_same_spot_toggles_note_off():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    x = staff._x_for_beat(0.0)
    y = staff._y_for_pitch(67)
    _click(staff, x, y)
    assert len(ed.doc.tracks[0].notes) == 1
    _click(staff, x, y)             # gleiche Stelle -> entfernt
    assert len(ed.doc.tracks[0].notes) == 0


def test_right_click_removes_note():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    x = staff._x_for_beat(1.0)
    y = staff._y_for_pitch(60)
    _click(staff, x, y)
    assert len(ed.doc.tracks[0].notes) == 1
    _click(staff, x, y, button="right")
    assert len(ed.doc.tracks[0].notes) == 0


def test_rest_toggle_places_rest_instead_of_note():
    ed = _editor()
    ed.rest_check.setChecked(True)
    staff = ed._track_rows[0]["staff"]
    x = staff._x_for_beat(0.0)
    y = staff._y_for_pitch(60)
    _click(staff, x, y)
    assert len(ed.doc.tracks[0].notes) == 1
    assert ed.doc.tracks[0].notes[0].rest is True


def test_duration_selection_changes_snap_grid():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]

    ed.dur_combo.setCurrentIndex(2)          # Viertel (1.0 Beat)
    assert ed.entry_duration_beats() == 1.0
    x_quarter = staff._x_for_beat(0.6)       # nahe an Beat 0.6
    y = staff._y_for_pitch(60)
    _click(staff, x_quarter, y)
    assert abs(ed.doc.tracks[0].notes[0].start_beat - 1.0) < 1e-6

    ed.doc.tracks[0].notes.clear()
    ed.dur_combo.setCurrentIndex(4)          # Sechzehntel (0.25 Beat)
    assert ed.entry_duration_beats() == 0.25
    _click(staff, x_quarter, y)
    assert abs(ed.doc.tracks[0].notes[0].start_beat - 0.5) < 1e-6


def test_accidental_toggle_shifts_pitch():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    x = staff._x_for_beat(0.0)
    y = staff._y_for_pitch(60)               # natuerliches C4

    ed._on_accidental_changed(1)             # Kreuz (Sharp)
    _click(staff, x, y)
    assert ed.doc.tracks[0].notes[0].pitch == 61


def test_instrument_and_clef_change_reflected_in_doc():
    ed = _editor()
    from gamebasic.tracker.presets import preset_names
    names = preset_names()
    ed._on_instrument_changed(0, names.index("Bass"))
    assert ed.doc.tracks[0].instrument.name == "Bass"

    ed._on_clef_changed(0, ed._track_rows[0]["clef_combo"])
    # setCurrentIndex nicht simuliert -- direkter Aufruf mit dem Test-Combo
    ed._track_rows[0]["clef_combo"].setCurrentIndex(1)
    ed._on_clef_changed(0, ed._track_rows[0]["clef_combo"])
    assert ed.doc.tracks[0].clef == "bass"


def test_playback_triggers_mixer_without_crashing():
    ed = _editor()
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    ed._start_play()
    for _ in range(8):
        ed._play_tick()
    ed._stop_play()
    assert not ed._playing


def test_save_and_load_roundtrip(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QFileDialog

    ed = _editor()
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    save_path = tmp_path / "mystueck.json"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        lambda *a, **k: (str(save_path), ""))
    ed._save()
    assert save_path.exists()

    ed2 = _editor()
    monkeypatch.setattr(QFileDialog, "getOpenFileName",
                        lambda *a, **k: (str(save_path), ""))
    ed2._open()
    assert ed2.doc.tracks[0].notes[0].pitch == 60


def test_octave_shift_for_clef_computes_whole_octaves():
    from gamebasic.scoreeditor_qt import ScoreEditor

    ed = _editor()
    track = ed.doc.tracks[0]
    for p in (60, 62, 64, 65, 67, 69, 71):     # C4..B4, Mittel ~65.4
        track.add_note(float(len(track.notes)), 1.0, p)

    shift = ScoreEditor._octave_shift_for_clef(track, "bass")
    assert shift % 12 == 0
    assert shift == -12                          # eine Oktave tiefer


def test_octave_shift_for_clef_zero_without_pitched_notes():
    from gamebasic.scoreeditor_qt import ScoreEditor

    ed = _editor()
    track = ed.doc.tracks[0]
    track.add_note(0.0, 1.0, rest=True)
    assert ScoreEditor._octave_shift_for_clef(track, "bass") == 0


def test_clef_change_same_clef_is_noop(monkeypatch):
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    called = []
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: called.append(1) or QMessageBox.StandardButton.Yes)
    combo = ed._track_rows[0]["clef_combo"]
    combo.setCurrentIndex(0)                     # bereits treble (Default)
    ed._on_clef_changed(0, combo)
    assert not called
    assert ed.doc.tracks[0].notes[0].pitch == 60


def test_clef_change_without_notes_skips_dialog(monkeypatch):
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()
    called = []
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: called.append(1) or QMessageBox.StandardButton.Yes)
    combo = ed._track_rows[0]["clef_combo"]
    combo.setCurrentIndex(1)                     # -> bass, keine Noten vorhanden
    ed._on_clef_changed(0, combo)
    assert not called
    assert ed.doc.tracks[0].clef == "bass"


def test_clef_change_accepts_transpose(monkeypatch):
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()
    track = ed.doc.tracks[0]
    for p in (60, 62, 64, 65, 67, 69, 71):
        track.add_note(float(len(track.notes)), 1.0, p)
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.Yes)

    combo = ed._track_rows[0]["clef_combo"]
    combo.setCurrentIndex(1)                     # -> bass
    ed._on_clef_changed(0, combo)

    assert track.clef == "bass"
    assert [n.pitch for n in track.notes] == [48, 50, 52, 53, 55, 57, 59]


def test_clef_change_declines_transpose_keeps_pitches(monkeypatch):
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()
    track = ed.doc.tracks[0]
    for p in (60, 62, 64, 65, 67, 69, 71):
        track.add_note(float(len(track.notes)), 1.0, p)
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.No)

    combo = ed._track_rows[0]["clef_combo"]
    combo.setCurrentIndex(1)                     # -> bass
    ed._on_clef_changed(0, combo)

    assert track.clef == "bass"                  # Schluessel wechselt trotzdem
    assert [n.pitch for n in track.notes] == [60, 62, 64, 65, 67, 69, 71]


def test_beam_groups_join_same_duration_runs_within_a_beat():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    track = ed.doc.tracks[0]
    track.add_note(0.0, 0.5, 60)
    track.add_note(0.5, 0.5, 62)          # Beat 0: 2 Achtel -> eine Gruppe
    track.add_note(1.0, 0.25, 64)
    track.add_note(1.25, 0.25, 65)
    track.add_note(1.5, 0.25, 67)
    track.add_note(1.75, 0.25, 69)        # Beat 1: 4 Sechzehntel -> eine Gruppe
    track.add_note(3.0, 0.5, 72)          # alleinstehendes Achtel -> keine Gruppe

    groups = staff._beam_groups()
    pitch_groups = sorted(([n.pitch for n in g] for g in groups))
    assert pitch_groups == [[60, 62], [64, 65, 67, 69]]


def test_beam_groups_break_across_beats_and_rests():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    track = ed.doc.tracks[0]
    track.add_note(0.5, 0.5, 60)           # startet nicht auf Beat-Grenze
    track.add_note(1.0, 0.5, 62)           # anderer Beat -> keine Verbindung
    groups = staff._beam_groups()
    assert groups == []                     # beide bleiben Einzelnoten


def test_status_bar_reflects_entry_state_and_song_summary():
    ed = _editor()
    ed.dur_combo.setCurrentIndex(3)          # Achtel
    ed._on_accidental_changed(1)             # Kreuz
    ed.rest_check.setChecked(True)
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    ed._mark_dirty()
    msg = ed.status.currentMessage()
    assert "Achtel" in msg
    assert "Kreuz" in msg
    assert "Pause" in msg
    assert "1 Spur" in msg


def test_undo_redo_note_placement():
    ed = _editor()
    assert not ed.undo.can_undo()
    assert not ed.btn_undo.isEnabled()

    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    ed.undo.flush()
    assert len(ed.doc.tracks[0].notes) == 1
    assert ed.undo.can_undo()
    assert ed.btn_undo.isEnabled()

    ed.undo.undo()
    assert len(ed.doc.tracks[0].notes) == 0     # Note weg
    assert ed.undo.can_redo()
    assert ed.btn_redo.isEnabled()

    ed.undo.redo()
    assert len(ed.doc.tracks[0].notes) == 1
    assert ed.doc.tracks[0].notes[0].pitch == 60


def test_undo_multi_step_and_redo_cleared_by_new_edit():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    ed.undo.flush()
    staff = ed._track_rows[0]["staff"]           # nach Restore neu holen
    _click(staff, staff._x_for_beat(1.0), staff._y_for_pitch(62))
    ed.undo.flush()
    assert len(ed.doc.tracks[0].notes) == 2

    ed.undo.undo()
    assert len(ed.doc.tracks[0].notes) == 1
    ed.undo.undo()
    assert len(ed.doc.tracks[0].notes) == 0
    assert not ed.undo.can_undo()

    ed.undo.redo()
    assert len(ed.doc.tracks[0].notes) == 1
    ed.undo.redo()
    assert len(ed.doc.tracks[0].notes) == 2
    assert not ed.undo.can_redo()

    # Ein neuer Edit nach einem Undo verwirft den Redo-Stack.
    ed.undo.undo()
    assert len(ed.doc.tracks[0].notes) == 1
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(3.0), staff._y_for_pitch(64))
    ed.undo.flush()
    assert not ed.undo.can_redo()


def test_new_doc_and_open_reset_undo_history(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QFileDialog

    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    ed.undo.flush()
    assert ed.undo.can_undo()

    ed._new_doc()
    assert not ed.undo.can_undo()

    path = tmp_path / "song.json"
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    ed.doc.save_json(str(path))
    ed._new_doc()
    monkeypatch.setattr(QFileDialog, "getOpenFileName",
                        lambda *a, **k: (str(path), ""))
    ed._open()
    assert not ed.undo.can_undo()


def test_export_to_tracker_writes_valid_tracker_song(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QFileDialog
    from gamebasic.tracker.song import Song

    ed = _editor()
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    out_path = tmp_path / "exported.json"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        lambda *a, **k: (str(out_path), ""))

    calls = []
    monkeypatch.setattr(
        "subprocess.Popen", lambda cmd, **k: calls.append(cmd))

    ed._export_to_tracker()

    assert out_path.exists()
    song = Song.load_json(str(out_path))
    assert song.patterns[0].data[0][0] == 60
    assert len(calls) == 1
    assert "--tracker" in calls[0]
