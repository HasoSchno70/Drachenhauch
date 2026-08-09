"""Tests fuer die "Instr:"-Dropdown-UI (per-Note-Instrument-Ueberschreiben)
im Tracker-Editor (offscreen)."""
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
        from drachenhauch.trackereditor_qt import TrackerEditor
        return TrackerEditor(Path("."))
    except Exception as exc:  # pragma: no cover
        pytest.skip(f"Editor nicht konstruierbar: {exc}")


def _select(ed, c, r):
    ed.grid.setCurrentCell(r, c)


def test_inst_cell_combo_lists_dash_plus_pool():
    ed = _editor()
    names = [ed.inst_cell_combo.itemText(i)
             for i in range(ed.inst_cell_combo.count())]
    assert names[0] == "—"
    assert ed.inst_cell_combo.count() == 1 + len(ed.song.instruments)
    assert ed.inst_cell_combo.itemData(0) is None


def test_selecting_cell_instrument_sets_pattern_override():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    _select(ed, 0, 0)
    # Index 2 = drittes Kombobox-Element (Index 0 ist "—", 1 ist Instrument 0).
    ed.inst_cell_combo.setCurrentIndex(2)
    assert pat.get_inst(0, 0) == 1


def test_selecting_dash_clears_override():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    pat.set_inst(0, 0, 1)
    _select(ed, 0, 0)
    ed.inst_cell_combo.setCurrentIndex(0)
    assert pat.get_inst(0, 0) is None


def test_cell_inst_combo_syncs_when_selecting_different_cells():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    pat.set_inst(0, 0, 2)
    pat.set(0, 1, 64)               # kein Override
    _select(ed, 0, 0)
    assert ed.inst_cell_combo.currentData() == 2
    _select(ed, 0, 1)
    assert ed.inst_cell_combo.currentData() is None


def test_cell_text_shows_instrument_tag():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    pat.set_inst(0, 0, 3)
    txt = ed._cell_text(0, 60, inst=3)
    assert "i3" in txt


def test_cell_text_no_tag_when_no_override():
    from drachenhauch.tracker import note_name
    ed = _editor()
    txt = ed._cell_text(0, 60, inst=None)
    assert txt.strip() == note_name(60)
    assert not any(tok.startswith("i") and tok[1:].isdigit()
                   for tok in txt.split())


def test_removing_instrument_updates_cell_combo_and_pattern():
    ed = _editor()
    n_before = ed.inst_cell_combo.count()
    ed.inst_combo.setCurrentIndex(ed.inst_combo.count() - 1)
    ed._remove_instrument()
    assert ed.inst_cell_combo.count() == n_before - 1


def test_playback_resolves_per_note_instrument_override(monkeypatch):
    """`_play_columns` muss das per-Note-Instrument auflösen (nicht den
    Kanal-Standard) -- per Spy auf `_render_sound` verifiziert."""
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    other_idx = 1 if len(ed.song.instruments) > 1 else ed.song.add_instrument(
        ed.song.instruments[0])
    pat.set_inst(0, 0, other_idx)

    seen = []
    orig = ed._render_sound

    def spy(inst, midi, n_samples, slide=0):
        seen.append(inst)
        return orig(inst, midi, n_samples, slide)

    monkeypatch.setattr(ed, "_render_sound", spy)
    ed._play_columns(pat, 0)
    assert seen and seen[0] is ed.song.instruments[other_idx]


def test_audition_selected_resolves_per_note_instrument_override(monkeypatch):
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    other_idx = 1 if len(ed.song.instruments) > 1 else ed.song.add_instrument(
        ed.song.instruments[0])
    pat.set_inst(0, 0, other_idx)
    _select(ed, 0, 0)

    seen = []
    orig = ed._render_sound

    def spy(inst, midi, n_samples, slide=0):
        seen.append(inst)
        return orig(inst, midi, n_samples, slide)

    monkeypatch.setattr(ed, "_render_sound", spy)
    ed._audition_selected()
    assert seen and seen[0] is ed.song.instruments[other_idx]
