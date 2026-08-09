"""Tests fuer per-Note-Instrument-Ueberschreiben (Pattern.inst / Song.
instrument_for_cell) -- der Architektur-Wechsel von "Instrument pro Kanal"
(channel_inst, fest) auf "Instrument pro Note" (wie echte Tracker: XM/IT/
Renoise/ProTracker binden das Instrument an die einzelne Note, ein Kanal ist
nur ein Stimmen-Slot). Bei uns bleibt die Instrument-Nummer pro Note OPTIONAL
-- fehlt sie, greift weiterhin die Kanal-Standard-Zuweisung (channel_inst)."""
import numpy as np
import pytest

from drachenhauch.tracker import Instrument, Pattern, Song
from drachenhauch.tracker.mixer import _note_events, render_song


def _flat_inst(name: str, level: float, n: int = 4000, sr: int = 8000) -> Instrument:
    """Sample-Instrument mit konstantem Pegel -- macht per-Note-Instrument-
    Ueberschreiben im Mixer-Output messbar unterscheidbar."""
    return Instrument.from_array(name, np.full(n, level, dtype=np.float32), sr, 69)


# --------------------------------------------------------------- Pattern.inst

def test_set_inst_requires_a_note():
    p = Pattern("P")
    p.set_inst(0, 0, 2)             # keine Note an der Stelle -> no-op
    assert p.get_inst(0, 0) is None


def test_set_inst_on_existing_note():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_inst(0, 0, 3)
    assert p.get_inst(0, 0) == 3


def test_set_inst_negative_clears_to_none():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_inst(0, 0, 3)
    p.set_inst(0, 0, -1)
    assert p.get_inst(0, 0) is None


def test_clearing_note_clears_inst():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_inst(0, 0, 2)
    p.set(0, 0, None)
    assert p.get_inst(0, 0) is None


def test_note_off_clears_inst_too():
    from drachenhauch.tracker import NOTE_OFF
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_inst(0, 0, 2)
    p.set(0, 0, NOTE_OFF)
    assert p.get_inst(0, 0) is None


def test_set_rows_preserves_inst():
    p = Pattern("P", rows=8)
    p.set(0, 0, 60)
    p.set_inst(0, 0, 1)
    p.set_rows(16)
    assert p.get_inst(0, 0) == 1


def test_set_channels_preserves_inst():
    p = Pattern("P", rows=8, channels=4)
    p.set(0, 0, 60)
    p.set_inst(0, 0, 1)
    p.set_channels(8)
    assert p.get_inst(0, 0) == 1
    assert p.inst[7] == [None] * p.rows


def test_clear_resets_inst():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_inst(0, 0, 2)
    p.clear()
    assert p.get_inst(0, 0) is None


def test_copy_duplicates_inst_independently():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_inst(0, 0, 2)
    q = p.copy()
    q.set_inst(0, 0, 5)
    assert p.get_inst(0, 0) == 2
    assert q.get_inst(0, 0) == 5


def test_inst_json_roundtrip_only_written_if_set():
    p = Pattern("P")
    assert "inst" not in p.to_dict()
    p.set(0, 0, 60)
    p.set_inst(0, 0, 3)
    d = p.to_dict()
    assert "inst" in d
    p2 = Pattern.from_dict(d, channels=p.channels)
    assert p2.get_inst(0, 0) == 3


# --------------------------------------------------------------- Song-Aufloesung

def test_instrument_for_cell_falls_back_to_channel_default():
    s = Song()
    a = _flat_inst("A", 0.5)
    ia = s.add_instrument(a)
    s.channel_inst[0] = ia
    s.patterns[0].set(0, 0, 60)     # kein per-Note-Override
    resolved = s.instrument_for_cell(s.patterns[0], 0, 0)
    assert resolved is a


def test_instrument_for_cell_prefers_per_note_override():
    s = Song()
    a = _flat_inst("A", 0.5)
    b = _flat_inst("B", 0.2)
    ia = s.add_instrument(a)
    ib = s.add_instrument(b)
    s.channel_inst[0] = ia
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_inst(0, 0, ib)
    resolved = s.instrument_for_cell(s.patterns[0], 0, 0)
    assert resolved is b


def test_instrument_for_cell_ignores_out_of_range_override():
    s = Song()
    a = _flat_inst("A", 0.5)
    ia = s.add_instrument(a)
    s.channel_inst[0] = ia
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].inst[0][0] = 999   # direkt manipuliert, ausser Bereich
    resolved = s.instrument_for_cell(s.patterns[0], 0, 0)
    assert resolved is a


def test_remove_instrument_clears_matching_per_note_overrides():
    s = Song()
    a = _flat_inst("A", 0.5)
    b = _flat_inst("B", 0.2)
    ia = s.add_instrument(a)
    ib = s.add_instrument(b)
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_inst(0, 0, ib)
    s.remove_instrument(ib)
    assert s.patterns[0].get_inst(0, 0) is None


def test_remove_instrument_shifts_higher_per_note_overrides():
    s = Song()
    a = _flat_inst("A", 0.5)
    b = _flat_inst("B", 0.2)
    c = _flat_inst("C", 0.8)
    s.add_instrument(a); s.add_instrument(b); s.add_instrument(c)
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_inst(0, 0, 2)     # zeigt auf c
    s.remove_instrument(0)              # a entfernen -> b=0, c=1
    assert s.patterns[0].get_inst(0, 0) == 1
    assert s.instruments[s.patterns[0].get_inst(0, 0)] is c


def test_song_json_roundtrip_preserves_per_note_instrument():
    s = Song()
    a = _flat_inst("A", 0.5)
    b = _flat_inst("B", 0.2)
    s.add_instrument(a)
    ib = s.add_instrument(b)
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_inst(0, 0, ib)
    s2 = Song.from_dict(s.to_dict())
    assert s2.patterns[0].get_inst(0, 0) == ib
    assert s2.instrument_for_cell(s2.patterns[0], 0, 0).name == "B"


def test_from_dict_clamps_inst_when_pool_smaller():
    s = Song()
    a = _flat_inst("A", 0.5)
    ia = s.add_instrument(a)
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_inst(0, 0, ia)
    d = s.to_dict()
    d["instruments"] = []           # Pool "verschwindet" -- kaputte/alte Datei
    del d["channel_inst"]
    s2 = Song.from_dict(d)
    assert s2.patterns[0].get_inst(0, 0) is None


# --------------------------------------------------------------- Mixer

def test_render_song_uses_per_note_instrument_override():
    s = Song()
    a = _flat_inst("A", 0.9, n=20000)
    b = _flat_inst("B", 0.2, n=20000)
    ia = s.add_instrument(a)
    ib = s.add_instrument(b)
    s.channel_inst[0] = ia
    s.patterns[0].set_rows(4)
    s.patterns[0].set(0, 0, 69)               # Kanal-Standard (A, laut)
    s.patterns[0].set(0, 2, 69)
    s.patterns[0].set_inst(0, 2, ib)           # Override auf B (leise)

    mix = render_song(s, sr=8000, tail_ms=0)
    row_samples = int(8000 * s.row_ms() / 1000.0)
    probe_a = row_samples // 2                 # innerhalb der ersten Note (A)
    probe_b = row_samples * 2 + row_samples // 2  # innerhalb der zweiten (B)

    assert abs(mix[probe_a]) == pytest.approx(0.9, abs=0.05)
    assert abs(mix[probe_b]) == pytest.approx(0.2, abs=0.05)


def test_note_events_carries_per_note_inst_index():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_inst(0, 0, 5)
    ev = _note_events(s)
    assert ev[0][0][-1] == 5


def test_note_events_inst_none_when_unset():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    ev = _note_events(s)
    assert ev[0][0][-1] is None
