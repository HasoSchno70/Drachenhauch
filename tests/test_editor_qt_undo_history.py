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
    except Exception:  # pragma: no cover - PySide6 fehlt
        pytest.skip("PySide6 nicht verfuegbar")
    app = QApplication.instance() or QApplication([])
    yield app


def _make(_qapp, initial=0):
    from gamebasic.editor_qt.undo_history import SnapshotUndo
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
    from gamebasic.editor_qt.undo_history import SnapshotUndo
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
    except Exception as exc:  # pragma: no cover - Audio/Display fehlt
        pytest.skip(f"Editor nicht konstruierbar: {exc}")


def test_particle_editor_undo_redo(_qapp):
    from gamebasic.particleeditor_qt import ParticleEditor
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
    from gamebasic.sfxeditor_qt import SfxGenerator
    ed = _editor(_qapp, SfxGenerator)
    old = ed.base_freq.value()
    ed.base_freq.setValue(old + 123)
    ed.undo.flush()
    ed.undo.undo()
    assert ed.base_freq.value() == old
    ed.undo.redo()
    assert ed.base_freq.value() == old + 123


def test_tracker_editor_undo_redo(_qapp):
    from gamebasic.trackereditor_qt import TrackerEditor
    ed = _editor(_qapp, TrackerEditor)
    ed.grid.setCurrentCell(0, 0)
    ed._set_note(0, 0, 60)
    ed.undo.flush()
    assert ed.song.patterns[ed.cur].data[0][0] == 60
    ed.undo.undo()
    assert ed.song.patterns[ed.cur].data[0][0] is None
    ed.undo.redo()
    assert ed.song.patterns[ed.cur].data[0][0] == 60
