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
    """Simuliert einen echten Klick (Press + Release ohne Bewegung) --
    ein reiner Press wuerde bei einer bestehenden Note nur einen Drag
    starten, siehe _StaffView.mousePressEvent/mouseReleaseEvent."""
    from PySide6.QtCore import Qt, QPointF
    from PySide6.QtGui import QMouseEvent
    btn = Qt.MouseButton.LeftButton if button == "left" else Qt.MouseButton.RightButton
    press = QMouseEvent(QMouseEvent.Type.MouseButtonPress, QPointF(x, y),
                        btn, btn, Qt.KeyboardModifier.NoModifier)
    staff.mousePressEvent(press)
    release = QMouseEvent(QMouseEvent.Type.MouseButtonRelease, QPointF(x, y),
                          btn, Qt.MouseButton.NoButton, Qt.KeyboardModifier.NoModifier)
    staff.mouseReleaseEvent(release)


def _drag(staff, x0, y0, x1, y1):
    """Simuliert ein Ziehen: Press an (x0,y0), Move zu (x1,y1), Release."""
    from PySide6.QtCore import Qt, QPointF
    from PySide6.QtGui import QMouseEvent
    press = QMouseEvent(QMouseEvent.Type.MouseButtonPress, QPointF(x0, y0),
                        Qt.MouseButton.LeftButton, Qt.MouseButton.LeftButton,
                        Qt.KeyboardModifier.NoModifier)
    staff.mousePressEvent(press)
    move = QMouseEvent(QMouseEvent.Type.MouseMove, QPointF(x1, y1),
                       Qt.MouseButton.NoButton, Qt.MouseButton.LeftButton,
                       Qt.KeyboardModifier.NoModifier)
    staff.mouseMoveEvent(move)
    release = QMouseEvent(QMouseEvent.Type.MouseButtonRelease, QPointF(x1, y1),
                          Qt.MouseButton.LeftButton, Qt.MouseButton.NoButton,
                          Qt.KeyboardModifier.NoModifier)
    staff.mouseReleaseEvent(release)


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
    ed._on_mode_changed(1)  # Pause-Modus
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


def test_drag_moves_note_to_new_beat_and_pitch():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    assert len(ed.doc.tracks[0].notes) == 1

    _drag(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60),
         staff._x_for_beat(2.0), staff._y_for_pitch(67))

    notes = ed.doc.tracks[0].notes
    assert len(notes) == 1                          # verschoben, nicht dupliziert
    assert abs(notes[0].start_beat - 2.0) < 1e-6
    assert notes[0].pitch == 67


def test_drag_without_movement_still_removes_note():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    x, y = staff._x_for_beat(0.0), staff._y_for_pitch(60)
    _click(staff, x, y)
    assert len(ed.doc.tracks[0].notes) == 1
    _drag(staff, x, y, x, y)                         # Press+Release ohne Bewegung
    assert len(ed.doc.tracks[0].notes) == 0


def test_drag_onto_existing_note_replaces_it():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    _click(staff, staff._x_for_beat(2.0), staff._y_for_pitch(64))
    assert len(ed.doc.tracks[0].notes) == 2

    _drag(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60),
         staff._x_for_beat(2.0), staff._y_for_pitch(67))

    notes = ed.doc.tracks[0].notes
    assert len(notes) == 1                           # Zielnote wurde ersetzt
    assert abs(notes[0].start_beat - 2.0) < 1e-6
    assert notes[0].pitch == 67


def test_drag_rest_only_changes_beat_not_pitch():
    ed = _editor()
    ed._on_mode_changed(1)  # Pause-Modus
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    assert ed.doc.tracks[0].notes[0].rest is True

    ed._on_mode_changed(0)  # zurueck zu Note-Modus -- Ziehen soll trotzdem Pause bleiben
    _drag(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60),
         staff._x_for_beat(3.0), staff._y_for_pitch(72))

    note = ed.doc.tracks[0].notes[0]
    assert note.rest is True
    assert note.pitch is None
    assert abs(note.start_beat - 3.0) < 1e-6


def test_drag_marks_dirty_and_is_undoable():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    ed.undo.flush()

    _drag(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60),
         staff._x_for_beat(1.0), staff._y_for_pitch(62))
    ed.undo.flush()
    assert abs(ed.doc.tracks[0].notes[0].start_beat - 1.0) < 1e-6

    ed.undo.undo()
    assert abs(ed.doc.tracks[0].notes[0].start_beat - 0.0) < 1e-6
    assert ed.doc.tracks[0].notes[0].pitch == 60


def test_rest_toggle_places_rest_instead_of_note():
    ed = _editor()
    ed._on_mode_changed(1)  # Pause-Modus
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


def test_track_name_unchanged_edit_does_not_mark_dirty():
    """Vorher: jedes editingFinished() markierte dirty + Undo-Snapshot, auch
    wenn der Text unveraendert war (z.B. Fokus verloren/zurueckgewonnen ohne
    echten Edit)."""
    ed = _editor()
    edit = ed._track_rows[0]["name_edit"]
    original = ed.doc.tracks[0].name
    assert edit.text() == original

    ed._on_track_name_changed(0, edit)
    assert not ed._dirty
    assert not ed.undo.can_undo()

    edit.setText("Melodie")
    ed._on_track_name_changed(0, edit)
    assert ed.doc.tracks[0].name == "Melodie"
    assert ed._dirty


def test_playback_triggers_mixer_without_crashing():
    ed = _editor()
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    ed._start_play()
    for _ in range(8):
        ed._play_tick()
    ed._stop_play()
    assert not ed._playing


def test_save_reuses_known_path_without_dialog(tmp_path, monkeypatch):
    """Quick-Save: nach dem ersten Speichern/Oeffnen kennt doc.path den Pfad
    schon (ScoreDoc.save_json/load_json setzen ihn) -- ein wiederholtes
    _save() sollte ihn wiederverwenden statt erneut den Save-Dialog zu oeffnen."""
    from PySide6.QtWidgets import QFileDialog

    ed = _editor()
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    save_path = tmp_path / "erststueck.json"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        lambda *a, **k: (str(save_path), ""))
    ed._save()
    assert ed.doc.path == str(save_path)

    ed.doc.tracks[0].add_note(1.0, 1.0, 64)

    def _boom(*a, **k):
        raise AssertionError("Quick-Save haette keinen Save-Dialog oeffnen duerfen")
    monkeypatch.setattr(QFileDialog, "getSaveFileName", _boom)
    ed._save()
    assert not ed.windowTitle().endswith("*")


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
    ed._on_mode_changed(1)  # Pause-Modus
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
    from PySide6.QtWidgets import QFileDialog, QMessageBox

    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    ed.undo.flush()
    assert ed.undo.can_undo()

    # Dokument ist dirty -> _new_doc() fragt nach; "Verwerfen" waehlen.
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.No)
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


def test_title_shows_asterisk_when_dirty_and_clears_on_save(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QFileDialog

    ed = _editor()
    assert "*" not in ed.windowTitle()

    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    assert ed.windowTitle().endswith("*")

    save_path = tmp_path / "stueck.json"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        lambda *a, **k: (str(save_path), ""))
    ed._save()
    assert "*" not in ed.windowTitle()
    assert save_path.name in ed.windowTitle()


def test_confirm_dirty_true_without_dialog_when_clean():
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()

    def _boom(*a, **k):
        raise AssertionError("QMessageBox.question sollte bei sauberem Doc nicht aufgerufen werden")
    import unittest.mock
    with unittest.mock.patch.object(QMessageBox, "question", _boom):
        assert ed._confirm_dirty() is True


def test_confirm_dirty_cancel_blocks(monkeypatch):
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    assert ed._dirty

    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.Cancel)
    assert ed._confirm_dirty() is False
    assert ed._dirty                                # unveraendert


def test_confirm_dirty_discard_returns_true_without_saving(monkeypatch, tmp_path):
    from PySide6.QtWidgets import QFileDialog, QMessageBox

    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))

    calls = []
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        lambda *a, **k: calls.append(1) or (str(tmp_path / "x.json"), ""))
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.No)
    assert ed._confirm_dirty() is True
    assert not calls                                 # kein Speichern-Dialog aufgerufen


def test_confirm_dirty_yes_saves_and_clears_dirty(monkeypatch, tmp_path):
    from PySide6.QtWidgets import QFileDialog, QMessageBox

    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))

    save_path = tmp_path / "y.json"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        lambda *a, **k: (str(save_path), ""))
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.Yes)
    assert ed._confirm_dirty() is True
    assert save_path.exists()
    assert not ed._dirty


def test_close_event_accepts_when_clean():
    from PySide6.QtGui import QCloseEvent

    ed = _editor()
    ev = QCloseEvent()
    ed.closeEvent(ev)
    assert ev.isAccepted()


def test_close_event_ignored_on_cancel(monkeypatch):
    from PySide6.QtGui import QCloseEvent
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))

    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.Cancel)
    ev = QCloseEvent()
    ed.closeEvent(ev)
    assert not ev.isAccepted()


def test_close_event_accepts_on_discard(monkeypatch):
    from PySide6.QtGui import QCloseEvent
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))

    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.No)
    ev = QCloseEvent()
    ed.closeEvent(ev)
    assert ev.isAccepted()


def test_close_event_stops_playback_timer():
    """Vorher (Bug): closeEvent stoppte nur den Mixer, nicht den QTimer --
    ein noch aktiver _play_tick() haette den Mixer nach dem Schliessen
    erneut gestartet (Mixer.play() reconnectet transparent)."""
    from PySide6.QtGui import QCloseEvent

    ed = _editor()
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    ed._start_play()
    assert ed._play_timer.isActive()

    ev = QCloseEvent()
    ed.closeEvent(ev)
    assert ev.isAccepted()
    assert not ed._play_timer.isActive()
    assert not ed._playing


def test_save_shows_warning_and_stays_dirty_on_write_failure(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()
    ed.doc.tracks[0].add_note(0.0, 1.0, 60)
    ed.doc.path = str(tmp_path / "stueck.json")
    ed._dirty = True

    warned = []
    monkeypatch.setattr(
        QMessageBox, "warning",
        lambda *a, **k: warned.append(a) or QMessageBox.StandardButton.Ok)

    def _boom(*_a, **_k):
        raise OSError("Platte voll")
    monkeypatch.setattr(ed.doc, "save_json", _boom)

    assert ed._save() is False
    assert ed._dirty                  # kein falscher "gespeichert"-Zustand
    assert warned                     # User informiert statt Traceback


def test_confirm_dirty_yes_but_save_fails_stays_blocked(monkeypatch, tmp_path):
    from PySide6.QtWidgets import QMessageBox

    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))

    ed.doc.path = str(tmp_path / "z.json")

    def _boom(*_a, **_k):
        raise OSError("nope")
    monkeypatch.setattr(ed.doc, "save_json", _boom)
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.Yes)
    monkeypatch.setattr(QMessageBox, "warning",
                        lambda *a, **k: QMessageBox.StandardButton.Ok)

    assert ed._confirm_dirty() is False
    assert ed._dirty


def test_launch_with_initial_file_resets_undo_history(tmp_path, monkeypatch):
    """Vorher (Bug): launch() liess die Undo-Baseline auf dem leeren
    Default-Dokument stehen -- das erste Strg+Z nach dem Oeffnen einer
    Datei setzte das geladene Stueck auf leer zurueck."""
    from PySide6.QtWidgets import QApplication
    from gamebasic import scoreeditor_qt
    from gamebasic.score.document import ScoreDoc

    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 1.0, 60)
    path = tmp_path / "stueck.json"
    doc.save_json(str(path))

    monkeypatch.setattr(QApplication, "exec", lambda self: 0)
    captured = {}
    orig_init = scoreeditor_qt.ScoreEditor.__init__

    def _capture_init(self, *a, **k):
        orig_init(self, *a, **k)
        captured["win"] = self
    monkeypatch.setattr(scoreeditor_qt.ScoreEditor, "__init__", _capture_init)

    scoreeditor_qt.launch(Path("."), initial_file=path)
    win = captured["win"]

    assert win.doc.tracks[0].notes[0].pitch == 60
    assert not win.undo.can_undo()
    assert not win._dirty
    assert path.name in win.windowTitle()

    win.doc.tracks[0].add_note(1.0, 1.0, 64)
    win._mark_dirty()
    win.undo.flush()
    assert win.undo.can_undo()
    win.undo.undo()
    # Regressionsschutz: zurueck zum GELADENEN Stand (1 Note), nicht zum
    # leeren Default-Dokument.
    assert len(win.doc.tracks[0].notes) == 1
    assert win.doc.tracks[0].notes[0].pitch == 60


def test_add_remove_track_clears_sound_cache():
    ed = _editor()
    ed._sound_cache["stale"] = "x"
    ed._add_track()
    assert ed._sound_cache == {}

    ed._sound_cache["stale2"] = "y"
    ed._remove_last_track()
    assert ed._sound_cache == {}


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


# ---- Staccato/Fingersatz/Bindebogen (Notationszusaetze) --------------------

def test_staccato_mode_toggles_existing_note():
    ed = _editor()
    ed._on_mode_changed(4)                       # Staccato-Modus
    staff = ed._track_rows[0]["staff"]
    x, y = staff._x_for_beat(0.0), staff._y_for_pitch(60)
    _click(staff, x, y)                          # Note existiert noch nicht -> nichts passiert
    assert ed.doc.tracks[0].notes == []

    ed._on_mode_changed(0)
    _click(staff, x, y)                          # jetzt eine Note setzen
    ed._on_mode_changed(4)
    _click(staff, x, y)
    assert ed.doc.tracks[0].notes[0].staccato is True
    _click(staff, x, y)
    assert ed.doc.tracks[0].notes[0].staccato is False


def test_staccato_mode_ignores_rests():
    ed = _editor()
    ed._on_mode_changed(1)                        # Pause setzen
    staff = ed._track_rows[0]["staff"]
    x, y = staff._x_for_beat(0.0), staff._y_for_pitch(60)
    _click(staff, x, y)
    assert ed.doc.tracks[0].notes[0].rest is True

    ed._on_mode_changed(4)                        # Staccato-Modus
    _click(staff, x, y)
    assert ed.doc.tracks[0].notes[0].staccato is False   # Pause bleibt unveraendert


def test_fingering_mode_assigns_and_clears():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    x, y = staff._x_for_beat(0.0), staff._y_for_pitch(60)
    _click(staff, x, y)

    ed._on_mode_changed(3)                        # Fingersatz-Modus
    ed.fingering_spin.setValue(3)
    _click(staff, x, y)
    assert ed.doc.tracks[0].notes[0].fingering == 3

    _click(staff, x, y)                           # gleiche Zahl nochmal -> entfernt
    assert ed.doc.tracks[0].notes[0].fingering is None

    ed.fingering_spin.setValue(5)
    _click(staff, x, y)
    ed.fingering_spin.setValue(2)
    _click(staff, x, y)                           # andere Zahl -> ersetzt, nicht entfernt
    assert ed.doc.tracks[0].notes[0].fingering == 2


def test_slur_mode_two_clicks_create_slur():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    _click(staff, staff._x_for_beat(1.0), staff._y_for_pitch(64))

    ed._on_mode_changed(2)                        # Bindebogen-Modus
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    assert staff._slur_anchor_beat is not None
    _click(staff, staff._x_for_beat(1.0), staff._y_for_pitch(64))
    assert staff._slur_anchor_beat is None
    assert ed.doc.tracks[0].slurs == [(0.0, 1.0)]


def test_slur_mode_same_note_twice_cancels_anchor():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    ed._on_mode_changed(2)
    x, y = staff._x_for_beat(0.0), staff._y_for_pitch(60)
    _click(staff, x, y)
    assert staff._slur_anchor_beat is not None
    _click(staff, x, y)
    assert staff._slur_anchor_beat is None
    assert ed.doc.tracks[0].slurs == []


def test_slur_mode_right_click_removes_slur_not_note():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    _click(staff, staff._x_for_beat(1.0), staff._y_for_pitch(64))
    ed._on_mode_changed(2)
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    _click(staff, staff._x_for_beat(1.0), staff._y_for_pitch(64))
    assert ed.doc.tracks[0].slurs == [(0.0, 1.0)]

    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60), button="right")
    assert ed.doc.tracks[0].slurs == []
    assert len(ed.doc.tracks[0].notes) == 2       # Noten bleiben erhalten


def test_dragging_note_relocates_its_slur():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    _click(staff, staff._x_for_beat(1.0), staff._y_for_pitch(64))
    ed._on_mode_changed(2)
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    _click(staff, staff._x_for_beat(1.0), staff._y_for_pitch(64))
    assert ed.doc.tracks[0].slurs == [(0.0, 1.0)]

    ed._on_mode_changed(0)                        # zurueck zu Note-Modus zum Ziehen
    _drag(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60),
         staff._x_for_beat(3.0), staff._y_for_pitch(72))
    assert ed.doc.tracks[0].slurs == [(1.0, 3.0)]


def test_mode_change_cancels_pending_slur_anchor():
    ed = _editor()
    staff = ed._track_rows[0]["staff"]
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    ed._on_mode_changed(2)
    _click(staff, staff._x_for_beat(0.0), staff._y_for_pitch(60))
    assert staff._slur_anchor_beat is not None

    ed._on_mode_changed(0)                        # Moduswechsel bricht ab
    assert staff._slur_anchor_beat is None


def test_trigger_note_shortens_staccato_playback(monkeypatch):
    ed = _editor()
    track = ed.doc.tracks[0]
    played = []
    monkeypatch.setattr(ed._mixer, "play", lambda arr: played.append(arr))

    normal = track.add_note(0.0, 2.0, 60)
    ed._trigger_note(track, normal)
    n_normal = played[-1].size

    stacc = track.add_note(2.0, 2.0, 60, staccato=True)
    ed._trigger_note(track, stacc)
    n_staccato = played[-1].size

    assert n_staccato < n_normal


def test_trigger_note_staccato_respects_minimum_duration(monkeypatch):
    """Vorher (Bug): die Live-Vorschau kannte STACCATO_MIN_ROWS (die
    Mindestdauer-Garantie aus convert.py) nicht -- eine sehr kurze
    Staccato-Note klang live hoerbar kuerzer als beim Tracker-Export."""
    from gamebasic.score.convert import ROWS_PER_BEAT, STACCATO_MIN_ROWS

    ed = _editor()
    track = ed.doc.tracks[0]
    played = []
    monkeypatch.setattr(ed._mixer, "play", lambda arr: played.append(arr))

    note = track.add_note(0.0, 0.1, 60, staccato=True)   # dur*factor < Mindestdauer
    ed._trigger_note(track, note)
    n_samples = played[-1].size

    min_dur_beat = STACCATO_MIN_ROWS / ROWS_PER_BEAT
    expected_seconds = min_dur_beat * 60.0 / ed.doc.bpm
    naive_seconds = note.dur_beat * 0.5 * 60.0 / ed.doc.bpm    # alte Formel ohne Floor
    assert expected_seconds > naive_seconds
    sr = 44100
    assert n_samples == max(1, int(sr * expected_seconds))


def test_staff_renders_without_crashing_with_all_annotations():
    ed = _editor()
    track = ed.doc.tracks[0]
    track.add_note(0.0, 1.0, 60, staccato=True)
    track.add_note(1.0, 1.0, 64, fingering=2)
    track.add_note(2.0, 1.0, 67, staccato=True, fingering=5)
    track.add_slur(0.0, 2.0)
    staff = ed._track_rows[0]["staff"]
    staff.resize(400, 260)
    staff._slur_anchor_beat = 1.0     # auch den Anker-Highlight-Pfad ueben
    pix = staff.grab()                # loest paintEvent aus
    assert pix.width() > 0 and pix.height() > 0
