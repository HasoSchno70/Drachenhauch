"""Tests fuer Ungespeicherte-Aenderungen-Schutz + Quick-Save im Tracker-Editor
(offscreen). Siehe auch tests/test_audiostudio.py fuer den eingebetteten Fall
(AudioStudio.closeEvent, weil TrackerEditor.closeEvent als Tab-Seite nicht
feuert)."""
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


@pytest.fixture(autouse=True)
def _no_blocking_close_dialog(monkeypatch):
    """Ohne Mock wuerde `QMessageBox.question()` in Tests, die `.close()`
    nach einer dirty-machenden Aktion aufrufen, aber das Dialog-Verhalten
    selbst nicht pruefen wollen, unter offscreen-Qt auf ein nie kommendes
    Nutzer-Event warten. Default: "Verwerfen" (Discard) -- die einzelnen
    Dialog-spezifischen Tests unten ueberschreiben das selbst."""
    from PySide6.QtWidgets import QMessageBox
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.No)


def _editor():
    from gamebasic.trackereditor_qt import TrackerEditor
    return TrackerEditor(Path("."))


def test_editing_marks_dirty():
    ed = _editor()
    assert ed.dirty is False
    ed.song.patterns[0].set(0, 0, 60)
    ed._mark()
    ed.undo.flush()
    assert ed.dirty is True
    assert ed.windowTitle().endswith("*")


def test_save_as_then_quick_save_reuses_path(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QFileDialog
    ed = _editor()
    ed.song.patterns[0].set(0, 0, 60)
    ed._mark(); ed.undo.flush()
    assert ed.dirty is True

    target = tmp_path / "song.json"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        lambda *a, **k: (str(target), ""))
    assert ed._save_as() is True
    assert ed.path == target
    assert ed.dirty is False
    assert not ed.windowTitle().endswith("*")
    assert target.exists()

    # Zweite Aenderung + Quick-Save (Strg+S) -- KEIN Dialog diesmal.
    ed.song.patterns[0].set(0, 1, 64)
    ed._mark(); ed.undo.flush()
    assert ed.dirty is True

    def _boom(*a, **k):
        raise AssertionError("Quick-Save haette keinen Save-Dialog oeffnen duerfen")
    monkeypatch.setattr(QFileDialog, "getSaveFileName", _boom)
    assert ed._save() is True
    assert ed.dirty is False


def test_save_without_path_falls_back_to_save_as(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QFileDialog
    ed = _editor()
    assert ed.path is None
    target = tmp_path / "new.json"
    monkeypatch.setattr(QFileDialog, "getSaveFileName",
                        lambda *a, **k: (str(target), ""))
    assert ed._save() is True
    assert ed.path == target


def test_open_sets_path_and_clears_dirty(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QFileDialog
    from gamebasic.tracker import Song
    src = tmp_path / "existing.json"
    Song().save_json(str(src))

    ed = _editor()
    ed.song.patterns[0].set(0, 0, 60)
    ed._mark(); ed.undo.flush()
    assert ed.dirty is True

    monkeypatch.setattr(QFileDialog, "getOpenFileName",
                        lambda *a, **k: (str(src), ""))
    ed._open()
    assert ed.path == src
    assert ed.dirty is False


def test_close_event_prompts_when_dirty(monkeypatch):
    from PySide6.QtGui import QCloseEvent
    from PySide6.QtWidgets import QMessageBox
    ed = _editor()
    ed.song.patterns[0].set(0, 0, 60)
    ed._mark(); ed.undo.flush()
    assert ed.dirty is True

    called = []
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: called.append(1) or QMessageBox.StandardButton.Cancel)
    ev = QCloseEvent()
    ed.closeEvent(ev)
    assert called == [1]
    assert not ev.isAccepted()


def test_close_event_no_dialog_when_clean():
    from PySide6.QtGui import QCloseEvent
    from PySide6.QtWidgets import QMessageBox
    ed = _editor()
    assert ed.dirty is False

    def _boom(*a, **k):
        raise AssertionError("QMessageBox.question sollte bei sauberem Doc nicht aufgerufen werden")
    import unittest.mock
    with unittest.mock.patch.object(QMessageBox, "question", _boom):
        ev = QCloseEvent()
        ed.closeEvent(ev)
    assert ev.isAccepted()


def test_new_song_discards_dirty_only_after_confirmation(monkeypatch):
    from PySide6.QtWidgets import QMessageBox
    ed = _editor()
    ed.song.patterns[0].set(0, 0, 60)
    ed._mark(); ed.undo.flush()
    assert ed.dirty is True

    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: QMessageBox.StandardButton.No)   # verwerfen
    ed._new_song()
    assert ed.song.patterns[0].get(0, 0) is None
    assert ed.dirty is False


# --- closeEvent muss laufende Wiedergabe VOR dem Mixer stoppen -----------
# (Review-Fund: Mixer.play() oeffnet nach stop() transparent einen neuen
# Stream, wenn der Timer noch einen Tick nachschiebt.)

def test_close_event_stops_playback_timer():
    from PySide6.QtGui import QCloseEvent
    ed = _editor()
    ed._toggle_play("pattern")
    assert ed._timer.isActive()

    ev = QCloseEvent()
    ed.closeEvent(ev)
    assert ev.isAccepted()
    assert not ed._timer.isActive()
    assert not ed._vu_timer.isActive()


# --- _new_song()/_open() stoppen laufende Wiedergabe + leeren den --------
# --- Sound-Cache (Review-Fund) --------------------------------------------

def test_new_song_stops_playback_and_clears_sound_cache():
    ed = _editor()
    ed._toggle_play("pattern")
    assert ed._timer.isActive()
    ed._sound_cache["stale"] = "x"

    ed._new_song()
    assert not ed._timer.isActive()
    assert ed._sound_cache == {}


def test_open_stops_playback(tmp_path, monkeypatch):
    from PySide6.QtWidgets import QFileDialog
    from gamebasic.tracker import Song
    src = tmp_path / "existing.json"
    Song().save_json(str(src))

    ed = _editor()
    ed._toggle_play("pattern")
    assert ed._timer.isActive()

    monkeypatch.setattr(QFileDialog, "getOpenFileName",
                        lambda *a, **k: (str(src), ""))
    ed._open()
    assert not ed._timer.isActive()


# --- BPM-Aenderung waehrend laufender Wiedergabe aktualisiert das ---------
# --- Timer-Intervall live (Review-Fund) -----------------------------------

def test_bpm_change_updates_running_timer_interval():
    ed = _editor()
    ed._toggle_play("pattern")
    old_interval = ed._timer.interval()

    ed.bpm.setValue(ed.song.bpm * 2)     # deutlich schneller
    assert ed._timer.interval() == ed.song.row_ms()
    assert ed._timer.interval() != old_interval


def test_bpm_change_without_playback_does_not_touch_timer():
    ed = _editor()
    assert not ed._timer.isActive()
    ed.bpm.setValue(200)
    assert not ed._timer.isActive()      # kein Crash/Start durch die BPM-Aenderung


# --- GB-Code-Export warnt bei Effekt-Spalte/Per-Note-Instrument (Review- --
# --- Fund: der Export ignoriert beides komplett, vorher ohne Hinweis) -----

def test_export_warns_when_song_uses_fx(monkeypatch):
    from PySide6.QtWidgets import QMessageBox
    from gamebasic.tracker.song import FX_ARP
    ed = _editor()
    ed.song.patterns[0].set(0, 0, 60)
    ed.song.patterns[0].set_fx(0, 0, FX_ARP, 0x37)

    calls = []
    monkeypatch.setattr(QMessageBox, "information",
                        lambda *a, **k: calls.append(1))
    ed._export()
    assert calls == [1]


def test_export_no_warning_for_plain_song(monkeypatch):
    from PySide6.QtWidgets import QMessageBox
    ed = _editor()
    ed.song.patterns[0].set(0, 0, 60)

    def _boom(*a, **k):
        raise AssertionError("QMessageBox.information sollte hier nicht aufgerufen werden")
    monkeypatch.setattr(QMessageBox, "information", _boom)
    ed._export()   # kein Crash, kein Warn-Dialog
