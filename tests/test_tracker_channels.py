"""Tests fuer konfigurierbare Kanalzahl (Song.channels) + Block-Operationen
(Copy/Paste/Transpose/Interpolate) im Tracker-Datenmodell (Qt-frei).

Frueher war der Tracker fest auf 4 Kanaele (3 tonal + 1 Drum) verdrahtet;
`Song.channels`/`Song.set_channels()` heben das auf 4..32 (MOD..XM-Niveau)
an, der LETZTE Kanal bleibt immer Drum/Noise."""
import numpy as np
import pytest

from gamebasic.tracker import (
    CHANNELS, MAX_CHANNELS, MIN_CHANNELS, TONAL, Pattern, Song,
    block_copy, block_interpolate, block_paste, block_transpose,
)
from gamebasic.tracker.mixer import render_song


# --------------------------------------------------------------- Kanalzahl

def test_default_song_has_classic_4_channels():
    s = Song()
    assert s.channels == CHANNELS == 4
    assert s.tonal == TONAL == 3
    assert len(s.waves) == 3
    assert len(s.channel_inst) == 4
    assert s.patterns[0].channels == 4


def test_song_created_with_more_channels():
    s = Song(channels=8)
    assert s.channels == 8
    assert s.tonal == 7
    assert len(s.waves) == 7
    assert len(s.channel_inst) == 8
    assert s.patterns[0].channels == 8
    # letzter Kanal bleibt Drum/Noise
    assert s.instrument_for_channel(7).waveform == "noise"
    assert s.instrument_for_channel(0).waveform == "square"


def test_channels_clamped_to_range():
    assert Song(channels=1).channels == MIN_CHANNELS
    assert Song(channels=999).channels == MAX_CHANNELS


def test_set_channels_expands_preserving_data():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set(3, 1, 40)     # Drum-Hit auf altem letzten Kanal
    s.set_channels(8)
    assert s.channels == 8
    assert s.tonal == 7
    assert len(s.waves) == 7
    assert len(s.channel_inst) == 8
    pat = s.patterns[0]
    assert pat.channels == 8
    assert pat.data[0][0] == 60          # alte Note erhalten
    assert pat.data[3][1] == 40          # alter Drum-Hit blieb an Index 3
    assert pat.data[7] == [None] * pat.rows   # neuer letzter Kanal ist leer


def test_set_channels_shrinks_truncating():
    s = Song(channels=8)
    s.patterns[0].set(7, 0, 50)
    s.set_channels(4)
    assert s.channels == 4
    assert len(s.patterns[0].data) == 4


def test_set_channels_all_patterns_resized():
    s = Song()
    s.add_pattern()
    s.set_channels(6)
    assert all(p.channels == 6 for p in s.patterns)


def test_add_pattern_uses_song_channel_count():
    s = Song(channels=8)
    idx = s.add_pattern()
    assert s.patterns[idx].channels == 8


def test_duplicate_pattern_keeps_channel_count():
    s = Song(channels=8)
    s.patterns[0].set(2, 0, 64)
    idx = s.duplicate_pattern(0)
    assert s.patterns[idx].channels == 8
    assert s.patterns[idx].data[2][0] == 64


# --------------------------------------------------------------- JSON-Roundtrip

def test_channels_json_roundtrip():
    s = Song(channels=10)
    s.patterns[0].set(5, 2, 70)
    s.waves[4] = "sine"
    d = s.to_dict()
    assert d["channels"] == 10
    s2 = Song.from_dict(d)
    assert s2.channels == 10
    assert s2.tonal == 9
    assert s2.patterns[0].channels == 10
    assert s2.patterns[0].data[5][2] == 70
    assert s2.waves[4] == "sine"


def test_old_file_without_channels_field_defaults_to_4():
    s = Song()
    d = s.to_dict()
    del d["channels"]                    # simuliert eine aeltere Datei
    for p in d["patterns"]:
        del p["channels"]
    s2 = Song.from_dict(d)
    assert s2.channels == CHANNELS == 4


# --------------------------------------------------------------- Flatten/GB-Code/Mixer

def test_flatten_and_gb_code_with_more_channels(tmp_path):
    s = Song(channels=6)
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set(5, 1, 40)          # letzter Kanal = Drum
    total, channels = s.flatten()
    assert len(channels) == 6
    assert channels[0][0] != 0
    assert channels[5][1] == 1           # Drum-Hit als 1 kodiert
    code = s.gb_code()
    assert "DIM trk5[TRK_ROWS]" in code   # letzter Kanal exportiert


def test_render_song_with_more_channels_smoke():
    s = Song(channels=6)
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set(5, 1, 40)
    mix = render_song(s, sr=8000, tail_ms=50)
    assert mix.size > 0
    assert np.isfinite(mix).all()


def test_render_song_stereo_hard_pan_more_channels_smoke():
    s = Song(channels=8)
    for c in range(8):
        s.patterns[0].set(c, 0, 60)
    mix = render_song(s, sr=8000, tail_ms=20, stereo=True, hard_pan=True)
    assert mix.shape[1] == 2
    assert np.isfinite(mix).all()


# --------------------------------------------------------------- Block-Ops

def test_block_copy_paste_roundtrip():
    p = Pattern("P", rows=8)
    p.set(0, 0, 60)
    p.set_vol(0, 0, 10)
    p.set(1, 1, 64)
    cells = block_copy(p, 0, 0, 1, 1)
    assert len(cells) == 2 and len(cells[0]) == 2
    n_c, n_r = block_paste(p, cells, 2, 4)
    assert (n_c, n_r) == (2, 2)
    assert p.data[2][4] == 60
    assert p.vol[2][4] == 10
    assert p.data[3][5] == 64


def test_block_paste_clips_at_pattern_edge():
    p = Pattern("P", rows=4)
    src = Pattern("S", rows=4)
    src.set(0, 0, 60)
    src.set(0, 1, 61)
    src.set(0, 2, 62)
    cells = block_copy(src, 0, 0, 0, 2)
    block_paste(p, cells, 0, 2)          # Zeile 4 (0+2+2) liegt ausserhalb
    assert p.data[0][2] == 60
    assert p.data[0][3] == 61
    assert p.rows == 4                   # kein IndexError, sauber geclippt


def test_block_transpose_shifts_notes():
    p = Pattern("P", rows=4)
    p.set(0, 0, 60)
    p.set(0, 1, 64)
    block_transpose(p, 0, 0, 0, 1, 12)
    assert p.data[0][0] == 72
    assert p.data[0][1] == 76


def test_block_transpose_clamps_to_midi_range():
    p = Pattern("P", rows=2)
    p.set(0, 0, 120)
    block_transpose(p, 0, 0, 0, 0, 20)
    assert p.data[0][0] == 127


def test_block_transpose_skips_drum_channel():
    p = Pattern("P", rows=2, channels=4)
    p.set(3, 0, 40)
    block_transpose(p, 0, 0, 3, 0, 12, skip_channel=3)
    assert p.data[3][0] == 40            # unveraendert


def test_block_interpolate_ramps_notes():
    p = Pattern("P", rows=9)
    p.set(0, 0, 60)
    p.set(0, 8, 72)
    block_interpolate(p, 0, 0, 0, 8)
    assert p.data[0][0] == 60
    assert p.data[0][8] == 72
    assert p.data[0][4] == 66             # Mitte linear interpoliert
    assert p.data[0][2] == 63


def test_block_interpolate_leaves_existing_notes_untouched():
    p = Pattern("P", rows=5)
    p.set(0, 0, 60)
    p.set(0, 2, 999)          # absichtlich "falscher" Zwischenwert
    p.set(0, 4, 64)
    block_interpolate(p, 0, 0, 0, 4)
    assert p.data[0][2] == 999


def test_block_interpolate_needs_two_notes():
    p = Pattern("P", rows=4)
    p.set(0, 0, 60)
    block_interpolate(p, 0, 0, 0, 3)
    assert p.data[0][1] is None and p.data[0][2] is None and p.data[0][3] is None


# --------------------------------------------------------------- Kanal-Fader
# (Song.channel_vol -- Mixer-Lautstaerke pro Kanal, wie das Default-Volume in
# XM/IT: separat von der Noten-Lautstaerke (vol-Spalte) und vom Instrument.)

def test_channel_vol_defaults_to_full():
    s = Song()
    assert s.channel_vol == [1.0] * s.channels


def test_set_channels_extends_channel_vol_with_full_default():
    s = Song()
    s.channel_vol[0] = 0.4
    s.set_channels(8)
    assert len(s.channel_vol) == 8
    assert s.channel_vol[0] == 0.4
    assert s.channel_vol[4:] == [1.0] * 4


def test_set_channels_shrinks_channel_vol():
    s = Song(channels=8)
    s.channel_vol[7] = 0.2
    s.set_channels(4)
    assert len(s.channel_vol) == 4


def test_channel_vol_json_roundtrip_only_written_if_nondefault():
    s = Song()
    d = s.to_dict()
    assert "channel_vol" not in d          # alles auf 1.0 -> schlank bleiben
    s.channel_vol[1] = 0.5
    d = s.to_dict()
    assert d["channel_vol"][1] == 0.5
    s2 = Song.from_dict(d)
    assert s2.channel_vol[1] == 0.5
    assert s2.channel_vol[0] == 1.0


def test_channel_vol_old_file_without_field_defaults_full():
    s = Song()
    d = s.to_dict()
    s2 = Song.from_dict(d)                 # kein "channel_vol"-Feld im Dict
    assert s2.channel_vol == [1.0] * s2.channels


def test_render_song_channel_vol_scales_output():
    s_full = Song()
    s_full.patterns[0].set(0, 0, 60)
    mix_full = render_song(s_full, sr=8000, tail_ms=0)

    s_half = Song()
    s_half.patterns[0].set(0, 0, 60)
    s_half.channel_vol[0] = 0.5
    mix_half = render_song(s_half, sr=8000, tail_ms=0)

    assert np.abs(mix_half).max() == pytest.approx(np.abs(mix_full).max() * 0.5, rel=0.05)


def test_gb_code_bakes_channel_vol_into_amp():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.channel_vol[0] = 0.5
    code = s.gb_code()
    assert "* 0.5" in code


def test_gb_code_omits_channel_vol_multiplier_when_full():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    code = s.gb_code()
    assert "* 1.0" not in code
