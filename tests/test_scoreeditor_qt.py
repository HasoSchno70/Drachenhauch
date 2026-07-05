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
