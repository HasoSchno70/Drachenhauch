"""Tests fuer das vereinte Audio Studio (Tracker + SFX als Tabs).

Headless (offscreen). Prueft, dass beide Editoren eingebettet und ihre
Undo-Shortcuts auf den Tab-Teilbaum beschraenkt sind (sonst "ambiguous
shortcut" bei zwei Strg+Z im selben Fenster), und dass die Tab-Wahl klappt.
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


def _studio(_qapp):
    from pathlib import Path
    from gamebasic.audiostudio_qt import AudioStudio
    return AudioStudio(Path("."))


def test_studio_embeds_both_editors(_qapp):
    from gamebasic.trackereditor_qt import TrackerEditor
    from gamebasic.sfxeditor_qt import SfxGenerator
    st = _studio(_qapp)
    assert st.tabs.count() == 2
    assert isinstance(st.tracker, TrackerEditor)
    assert isinstance(st.sfx, SfxGenerator)
    # beide Editoren sind als Tab-Seiten eingebettet
    assert st.tabs.widget(0) is st.tracker
    assert st.tabs.widget(1) is st.sfx


def test_embedded_undo_shortcuts_are_tab_scoped(_qapp):
    from PySide6.QtGui import QShortcut
    from PySide6.QtCore import Qt
    st = _studio(_qapp)
    for ed in (st.tracker, st.sfx):
        shortcuts = ed.findChildren(QShortcut)
        assert shortcuts, "Editor sollte Undo/Redo-Shortcuts haben"
        for sc in shortcuts:
            assert sc.context() == Qt.ShortcutContext.WidgetWithChildrenShortcut


def test_select_tab_by_name(_qapp):
    st = _studio(_qapp)
    st.select_tab("sfx")
    assert st.tabs.currentIndex() == 1
    st.select_tab("tracker")
    assert st.tabs.currentIndex() == 0
    st.select_tab("sound")          # Alias fuer SFX
    assert st.tabs.currentIndex() == 1
    st.select_tab("music")          # Alias fuer Tracker
    assert st.tabs.currentIndex() == 0


def test_close_event_delegates_to_embedded_tracker_dirty_check(_qapp, monkeypatch):
    """TrackerEditor.closeEvent feuert NICHT, wenn er nur eine Tab-Seite ist
    (kein Top-Level-Fenster) -- AudioStudio.closeEvent muss den Dirty-Check
    deshalb selbst an self.tracker delegieren, sonst geht ungespeicherte
    Tracker-Arbeit beim Schliessen des Studio-Fensters kommentarlos verloren."""
    from PySide6.QtGui import QCloseEvent
    from PySide6.QtWidgets import QMessageBox
    st = _studio(_qapp)
    st.tracker.song.patterns[0].set(0, 0, 60)
    st.tracker._mark(); st.tracker.undo.flush()
    assert st.tracker.dirty is True

    called = []
    monkeypatch.setattr(QMessageBox, "question",
                        lambda *a, **k: called.append(1) or QMessageBox.StandardButton.Cancel)
    ev = QCloseEvent()
    st.closeEvent(ev)
    assert called == [1]
    assert not ev.isAccepted()


def test_close_event_stops_tracker_playback_timer(_qapp):
    """Review-Fund: AudioStudio.closeEvent stoppte nur den Mixer, nicht den
    Tracker-Playback-Timer -- Mixer.play() oeffnet nach stop() transparent
    einen neuen Audio-Stream, wenn der Timer noch einen Tick nachschiebt."""
    from PySide6.QtGui import QCloseEvent
    st = _studio(_qapp)
    st.tracker._toggle_play("pattern")
    assert st.tracker._timer.isActive()

    ev = QCloseEvent()
    st.closeEvent(ev)
    assert ev.isAccepted()
    assert not st.tracker._timer.isActive()


def test_close_event_no_dialog_when_tracker_clean(_qapp):
    from PySide6.QtGui import QCloseEvent
    from PySide6.QtWidgets import QMessageBox
    st = _studio(_qapp)
    assert st.tracker.dirty is False

    def _boom(*a, **k):
        raise AssertionError("QMessageBox.question sollte bei sauberem Tracker nicht aufgerufen werden")
    import unittest.mock
    with unittest.mock.patch.object(QMessageBox, "question", _boom):
        ev = QCloseEvent()
        st.closeEvent(ev)
    assert ev.isAccepted()
