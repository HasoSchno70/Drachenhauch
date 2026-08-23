"""Tests fuer die Snapshot-Undo/Redo-Historie (Partikel/SFX/Tracker).

Deterministisch: statt auf den debounce-Timer zu warten, rufen wir
`flush()`/`undo()` (das selbst flusht). Kern-Logik ohne die schweren
Editor-Fenster -- ein simpler externer State-Dict als Modell.
"""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "hide")


@pytest.fixture(scope="module")
def _qapp():
    try:
        from PySide6.QtWidgets import QApplication
    except ImportError:  # pragma: no cover - PySide6 fehlt
        pytest.skip("PySide6 nicht verfuegbar")
    app = QApplication.instance() or QApplication([])
    yield app


def _make(_qapp, initial=0):
    from drachenhauch.editor_qt.undo_history import SnapshotUndo
    state = {"v": initial}
    hist = SnapshotUndo(lambda: dict(state), lambda s: state.update(s),
                        debounce_ms=0)
    return state, hist


def test_initial_has_no_history(_qapp):
    _state, hist = _make(_qapp)
    assert not hist.can_undo()
    assert not hist.can_redo()


def test_undo_redo_roundtrip(_qapp):
    state, hist = _make(_qapp, 0)
    state["v"] = 5
    hist.mark(); hist.flush()
    assert hist.can_undo()
    hist.undo()
    assert state["v"] == 0
    assert hist.can_redo()
    hist.redo()
    assert state["v"] == 5


def test_multiple_steps(_qapp):
    state, hist = _make(_qapp, 0)
    for v in (1, 2, 3):
        state["v"] = v
        hist.mark(); hist.flush()
    hist.undo(); assert state["v"] == 2
    hist.undo(); assert state["v"] == 1
    hist.undo(); assert state["v"] == 0
    assert not hist.can_undo()
    hist.redo(); assert state["v"] == 1


def test_new_change_clears_redo(_qapp):
    state, hist = _make(_qapp, 0)
    state["v"] = 1; hist.mark(); hist.flush()
    hist.undo(); assert state["v"] == 0
    assert hist.can_redo()
    # Neue Aenderung verwirft den Redo-Stack.
    state["v"] = 9; hist.mark(); hist.flush()
    assert not hist.can_redo()


def test_no_op_change_is_not_recorded(_qapp):
    state, hist = _make(_qapp, 7)
    # Kein echter Unterschied -> kein Undo-Schritt.
    hist.mark(); hist.flush()
    assert not hist.can_undo()


def test_reset_drops_history(_qapp):
    state, hist = _make(_qapp, 0)
    state["v"] = 1; hist.mark(); hist.flush()
    assert hist.can_undo()
    hist.reset()
    assert not hist.can_undo()
    assert not hist.can_redo()


def test_limit_caps_undo_stack(_qapp):
    from drachenhauch.editor_qt.undo_history import SnapshotUndo
    state = {"v": 0}
    hist = SnapshotUndo(lambda: dict(state), lambda s: state.update(s),
                        debounce_ms=0, limit=3)
    for v in range(1, 8):
        state["v"] = v
        hist.mark(); hist.flush()
    # Nur die letzten 3 Schritte bleiben undobar.
    steps = 0
    while hist.can_undo():
        hist.undo()
        steps += 1
        if steps > 10:
            break
    assert steps == 3


# --- Integration: die drei Editoren tatsaechlich konstruieren -----------
# Geschuetzt: fehlt Audio/Display, wird uebersprungen statt zu scheitern.

from pathlib import Path  # noqa: E402


def _editor(_qapp, factory):
    try:
        return factory(Path("."))
    except ImportError as exc:  # pragma: no cover -- Zusatzpaket fehlt
        # ENG halten: ein breites `except` machte aus jedem kaputten
        # Editor einen gruenen Uebersprung statt eines roten Tests.
        pytest.skip(f"Zusatzpaket fehlt: {exc}")


def test_particle_editor_undo_redo(_qapp):
    from drachenhauch.particleeditor_qt import ParticleEditor
    ed = _editor(_qapp, ParticleEditor)
    old = ed.size_min.value()
    ed.size_min.setValue(old + 7)
    ed.undo.flush()
    assert ed.undo.can_undo()
    ed.undo.undo()
    assert ed.size_min.value() == old
    ed.undo.redo()
    assert ed.size_min.value() == old + 7


def test_sfx_editor_undo_redo(_qapp):
    from drachenhauch.sfxeditor_qt import SfxGenerator
    ed = _editor(_qapp, SfxGenerator)
    old = ed.base_freq.value()
    ed.base_freq.setValue(old + 123)
    ed.undo.flush()
    ed.undo.undo()
    assert ed.base_freq.value() == old
    ed.undo.redo()
    assert ed.base_freq.value() == old + 123


def test_sfx_autoplay_debounce_scheduling(_qapp):
    # Bei aktivem Auto-Play startet ein Regler-Wechsel den (entprellten)
    # Wiedergabe-Timer; ausgeschaltet bleibt er aus. Eine explizite Wiedergabe
    # bricht ein ausstehendes Auto-Play ab.
    from drachenhauch.sfxeditor_qt import SfxGenerator
    ed = _editor(_qapp, SfxGenerator)
    ed.cb_autoplay.setChecked(True)
    ed._autoplay_timer.stop()
    ed.base_freq.setValue(ed.base_freq.value() + 100)
    assert ed._autoplay_timer.isActive()
    ed._play()                              # explizit -> bricht ab
    assert not ed._autoplay_timer.isActive()
    ed.cb_autoplay.setChecked(False)
    ed._autoplay_timer.stop()
    ed.base_freq.setValue(ed.base_freq.value() + 100)
    assert not ed._autoplay_timer.isActive()


def test_tracker_editor_undo_redo(_qapp):
    from drachenhauch.trackereditor_qt import TrackerEditor
    ed = _editor(_qapp, TrackerEditor)
    ed.grid.setCurrentCell(0, 0)
    ed._set_note(0, 0, 60)
    ed.undo.flush()
    assert ed.song.patterns[ed.cur].data[0][0] == 60
    ed.undo.undo()
    assert ed.song.patterns[ed.cur].data[0][0] is None
    ed.undo.redo()
    assert ed.song.patterns[ed.cur].data[0][0] == 60


def test_tracker_volume_column_ui(_qapp):
    """Volume-Spinbox setzt die Noten-Lautstaerke der gewaehlten Zelle und
    spiegelt sie beim Selektieren zurueck."""
    from drachenhauch.trackereditor_qt import TrackerEditor
    ed = _editor(_qapp, TrackerEditor)
    ed.grid.setCurrentCell(0, 0)
    ed._set_note(0, 0, 60)
    # Lautstaerke ueber den Spinbox setzen
    ed.vol_spin.setValue(9)
    assert ed.song.patterns[ed.cur].vol[0][0] == 9
    # Zelltext zeigt die Lautstaerke
    assert "v9" in ed.grid.item(0, 0).text()
    # Auf leere Zelle wechseln -> Spinbox zeigt Standard (0)
    ed.grid.setCurrentCell(1, 0)
    ed._sync_vol_spin()
    assert ed.vol_spin.value() == 0
    # Zurueck zur Note -> Spinbox spiegelt 9
    ed.grid.setCurrentCell(0, 0)
    ed._sync_vol_spin()
    assert ed.vol_spin.value() == 9
    # 0 = Standard -> Lautstaerke wieder None
    ed.vol_spin.setValue(0)
    assert ed.song.patterns[ed.cur].vol[0][0] is None


def test_tracker_slide_column_ui(_qapp):
    """Slide-Spinbox setzt den Pitch-Slide der gewaehlten Note (nur tonale
    Kanaele), spiegelt zurueck und zeigt ihn im Zelltext."""
    from drachenhauch.trackereditor_qt import TrackerEditor
    from drachenhauch.tracker import TONAL
    ed = _editor(_qapp, TrackerEditor)
    ed.grid.setCurrentCell(0, 0)
    ed._set_note(0, 0, 60)
    ed.slide_spin.setValue(3)
    assert ed.song.patterns[ed.cur].slide[0][0] == 3
    assert "s+3" in ed.grid.item(0, 0).text()
    # 0 -> kein Slide
    ed.slide_spin.setValue(0)
    assert ed.song.patterns[ed.cur].slide[0][0] is None
    # Drum-Kanal (TONAL): Slide ist deaktiviert + wird nicht gesetzt
    ed.grid.setCurrentCell(0, TONAL)
    ed._set_note(0, TONAL, 60)
    ed._sync_vol_spin()
    assert ed.slide_spin.isEnabled() is False
