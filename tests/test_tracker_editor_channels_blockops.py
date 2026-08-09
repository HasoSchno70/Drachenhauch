"""Tests fuer die Kanalzahl-UI + Block-Operationen (Copy/Cut/Paste/Transpose/
Interpolate) im Tracker-Editor (offscreen)."""
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


def _select(ed, c0, r0, c1=None, r1=None):
    from PySide6.QtCore import QItemSelectionModel
    from PySide6.QtWidgets import QTableWidgetSelectionRange
    c1 = c0 if c1 is None else c1
    r1 = r0 if r1 is None else r1
    top, bottom = sorted((r0, r1))
    left, right = sorted((c0, c1))
    ed.grid.clearSelection()
    ed.grid.setRangeSelected(
        QTableWidgetSelectionRange(top, left, bottom, right), True)
    # NoUpdate: setCurrentCell() wuerde die Rechteck-Auswahl sonst zusammenklappen.
    ed.grid.setCurrentCell(top, left, QItemSelectionModel.SelectionFlag.NoUpdate)


# --------------------------------------------------------------- Kanalzahl-UI

def test_default_editor_has_4_channel_strips():
    ed = _editor()
    assert ed.song.channels == 4
    assert len(ed.sound_combos) == 4
    assert ed.grid.columnCount() == 4


def test_channels_spin_changes_song_and_rebuilds_grid():
    ed = _editor()
    ed.channels_spin.setValue(8)
    assert ed.song.channels == 8
    assert ed.grid.columnCount() == 8
    assert len(ed.sound_combos) == 8
    assert len(ed.mute_btns) == 8
    assert len(ed.vu_meters) == 8


def test_channels_spin_reflects_loaded_song(tmp_path):
    from drachenhauch.tracker import Song
    ed = _editor()
    s = Song(channels=10)
    s.patterns[0].set(0, 0, 60)
    path = tmp_path / "wide.json"
    s.save_json(str(path))
    ed._restore_song(s.to_dict())
    assert ed.song.channels == 10
    assert ed.channels_spin.value() == 10
    assert ed.grid.columnCount() == 10
    assert len(ed.sound_combos) == 10


def test_channel_headers_last_is_drum_after_resize():
    ed = _editor()
    ed.channels_spin.setValue(6)
    labels = [ed.grid.horizontalHeaderItem(c).text() for c in range(6)]
    assert labels[-1].startswith("Drum")
    assert labels[0].startswith("Ch1")


# --- Mute/Solo ueberleben Undo/Redo + Kanalzahl-Aenderung (Review-Fund: ---
# --- _rebuild_channel_strips() setzte sie vorher stillschweigend zurueck) -

def test_mute_solo_survive_undo_triggered_rebuild():
    ed = _editor()
    ed.mute_btns[1].setChecked(True)
    ed.solo_btns[2].setChecked(True)
    assert ed._muted == [False, True, False, False]
    assert ed._solo == [False, False, True, False]

    # _restore_song() -> _reload_all() -> _rebuild_channel_strips(), wie
    # bei jedem Undo/Redo (Song-Inhalt hier bewusst unveraendert simuliert).
    ed._restore_song(ed.song.to_dict())

    assert ed._muted == [False, True, False, False]
    assert ed._solo == [False, False, True, False]
    assert ed.mute_btns[1].isChecked() is True
    assert ed.solo_btns[2].isChecked() is True


def test_mute_solo_extend_with_false_on_more_channels():
    ed = _editor()
    ed.mute_btns[0].setChecked(True)
    ed.channels_spin.setValue(8)
    assert ed._muted == [True, False, False, False, False, False, False, False]
    assert ed.mute_btns[0].isChecked() is True


def test_mute_solo_truncate_on_fewer_channels():
    ed = _editor()
    ed.channels_spin.setValue(8)
    ed.mute_btns[6].setChecked(True)
    ed.channels_spin.setValue(4)
    assert ed._muted == [False, False, False, False]   # Kanal 6 existiert nicht mehr


# --------------------------------------------------------------- Block-Ops

def test_block_copy_paste_via_editor_methods():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    pat.set(1, 0, 64)
    _select(ed, 0, 0, 1, 0)
    ed._block_copy()
    assert ed._block_clip is not None
    _select(ed, 0, 4)
    ed._block_paste()
    assert pat.data[0][4] == 60
    assert pat.data[1][4] == 64


def test_block_cut_clears_source():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    _select(ed, 0, 0)
    ed._block_cut()
    assert pat.data[0][0] is None
    _select(ed, 0, 2)
    ed._block_paste()
    assert pat.data[0][2] == 60


def test_block_transpose_via_editor_method():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    _select(ed, 0, 0)
    ed._block_transpose(12)
    assert pat.data[0][0] == 72


def test_block_transpose_skips_drum_channel_via_editor():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    drum = ed.song.tonal
    pat.set(drum, 0, 40)
    pat.set(0, 0, 60)
    _select(ed, 0, 0, drum, 0)
    ed._block_transpose(5)
    assert pat.data[0][0] == 65
    assert pat.data[drum][0] == 40   # Drum-Kanal unangetastet


def test_block_interpolate_via_editor_method():
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    pat.set(0, 4, 68)
    _select(ed, 0, 0, 0, 4)
    ed._block_interpolate()
    assert pat.data[0][2] == 64


def test_ctrl_i_keyboard_shortcut_interpolates():
    from PySide6.QtCore import Qt
    from PySide6.QtTest import QTest
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    pat.set(0, 4, 68)
    _select(ed, 0, 0, 0, 4)
    QTest.keyClick(ed, Qt.Key.Key_I, Qt.KeyboardModifier.ControlModifier)
    assert pat.data[0][2] == 64


def test_delete_key_clears_whole_block():
    from PySide6.QtCore import Qt
    from PySide6.QtTest import QTest
    ed = _editor()
    pat = ed.song.patterns[ed.cur]
    pat.set(0, 0, 60)
    pat.set(1, 0, 62)
    _select(ed, 0, 0, 1, 0)
    QTest.keyClick(ed, Qt.Key.Key_Delete)
    assert pat.data[0][0] is None
    assert pat.data[1][0] is None


# --------------------------------------------------------------- Kanal-Fader + Farben

def test_channel_strip_has_volume_slider_per_channel():
    ed = _editor()
    assert len(ed.vol_sliders) == 4
    for sl in ed.vol_sliders:
        assert sl.value() == 100        # Default channel_vol = 1.0


def test_moving_volume_slider_updates_song_and_label():
    ed = _editor()
    ed.vol_sliders[1].setValue(60)
    assert ed.song.channel_vol[1] == pytest.approx(0.6)
    assert "60" in ed.vol_labels[1].text()


def test_channels_spin_rebuilds_sliders_with_correct_count():
    ed = _editor()
    ed.channels_spin.setValue(8)
    assert len(ed.vol_sliders) == 8
    assert len(ed.vol_labels) == 8


def test_loaded_song_channel_vol_reflected_in_sliders(tmp_path):
    from drachenhauch.tracker import Song
    ed = _editor()
    s = Song()
    s.channel_vol[2] = 0.3
    ed._restore_song(s.to_dict())
    assert ed.vol_sliders[2].value() == pytest.approx(30, abs=1)


def test_channels_get_distinct_colors():
    from drachenhauch.trackereditor_qt import _channel_color
    colors = {_channel_color(c) for c in range(8)}
    assert len(colors) == 8             # 8 Kanaele, 8 eigene Paletten-Farben


def test_channel_header_uses_channel_color():
    from PySide6.QtGui import QColor
    from drachenhauch.trackereditor_qt import _channel_color
    ed = _editor()
    item0 = ed.grid.horizontalHeaderItem(0)
    item1 = ed.grid.horizontalHeaderItem(1)
    assert item0.foreground().color() == QColor(_channel_color(0))
    assert item1.foreground().color() == QColor(_channel_color(1))
    assert item0.foreground().color() != item1.foreground().color()


def test_cell_delegate_note_color_is_channel_specific():
    from drachenhauch.trackereditor_qt import _CellDelegate, _channel_color
    c0 = _CellDelegate._token_color(0, "C4", channel=0)
    c1 = _CellDelegate._token_color(0, "C4", channel=1)
    assert c0 == _channel_color(0)
    assert c1 == _channel_color(1)
    assert c0 != c1


def test_cell_delegate_drum_and_off_colors_unaffected_by_channel():
    from drachenhauch.trackereditor_qt import _CellDelegate
    assert _CellDelegate._token_color(0, "X", channel=3) == \
           _CellDelegate._token_color(0, "X", channel=5)
    assert _CellDelegate._token_color(0, "OFF", channel=3) == \
           _CellDelegate._token_color(0, "OFF", channel=5)
