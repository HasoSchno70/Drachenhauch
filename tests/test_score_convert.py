"""Tests fuer die Score->Tracker-Konvertierung (Qt-frei, gamebasic.score.convert)."""
import pytest

from gamebasic.score.document import ScoreDoc
from gamebasic.score.convert import to_tracker_song, _beat_to_row, _row_exact
from gamebasic.tracker.song import MAX_CHANNELS, MIN_CHANNELS, NOTE_OFF


@pytest.mark.parametrize("beat,row", [
    (0.25, 1), (0.5, 2), (1.0, 4), (1.5, 6), (2.0, 8), (4.0, 16),
])
def test_beat_to_row_exact_cases(beat, row):
    assert _beat_to_row(beat) == row
    assert _row_exact(beat) is True


def test_beat_to_row_rounds_inexact_and_flags():
    beat = 1.0 / 3.0             # triolen-artig, kein 1/16-Vielfaches
    row = _beat_to_row(beat)
    assert row == round(beat * 4)
    assert _row_exact(beat) is False


def test_single_track_short_piece():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 1.0, 60)
    song, warnings = to_tracker_song(doc)

    assert song.channels == MIN_CHANNELS       # 1 Spur+Drum < 4 -> geklemmt
    assert len(song.patterns) == 1
    assert song.order == [0]
    assert song.patterns[0].rows == 4          # 1 Beat = 4 Zeilen
    assert song.patterns[0].data[0] == [60, None, None, None]
    assert len(song.instruments) == 1
    assert song.channel_inst[0] == 0
    assert warnings == []


def test_multi_track_instrument_pool_assignment():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 1.0, 60)
    for i in range(4):
        idx = doc.add_track(f"Spur{i+2}")
        doc.tracks[idx].add_note(0.0, 1.0, 40 + i)

    song, warnings = to_tracker_song(doc)

    assert song.channels == 6                  # 5 Spuren + Drum, nicht geklemmt
    assert len(song.instruments) == 5
    assert song.channel_inst[:5] == [0, 1, 2, 3, 4]
    assert song.channel_inst[5] is None          # Drum-Kanal unbelegt
    assert warnings == []


def test_channel_count_over_max_raises():
    doc = ScoreDoc()
    for _ in range(MAX_CHANNELS - 1):            # 31 weitere -> 32 Spuren total
        doc.add_track()
    with pytest.raises(ValueError):
        to_tracker_song(doc)


def test_long_piece_forces_pattern_split_and_truncates_at_boundary():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 20.0, 60)         # 20 Beats = 80 Zeilen > 64
    song, warnings = to_tracker_song(doc)

    assert len(song.patterns) == 2
    assert song.order == [0, 1]
    assert song.patterns[0].rows == 64
    assert song.patterns[1].rows == 16
    assert song.patterns[0].data[0][0] == 60
    assert any("Pattern-Grenze" in w for w in warnings)


def test_note_exactly_on_chunk_boundary_lands_in_next_pattern():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 16.0, 60)         # endet exakt an Zeile 64
    doc.tracks[0].add_note(16.0, 1.0, 67)         # startet exakt an Zeile 64
    song, warnings = to_tracker_song(doc)

    assert len(song.patterns) == 2
    # Erste Note endet GENAU an der Grenze -> keine Kuerzungs-Warnung fuer sie.
    assert not any("Pattern-Grenze" in w for w in warnings)
    assert song.patterns[0].data[0][0] == 60
    assert song.patterns[1].data[0][0] == 67      # lokale Zeile 0 im 2. Pattern


def test_chord_reduces_to_highest_note():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 1.0, 60)
    doc.tracks[0].add_note(0.0, 1.0, 67)          # Akkord: gleicher Start-Beat
    song, warnings = to_tracker_song(doc)

    assert song.patterns[0].data[0][0] == 67       # hoechste Note gewinnt
    assert any("Akkord" in w for w in warnings)


def test_note_off_inserted_for_gap_before_next_note():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 1.0, 60)           # Zeile 0..3
    doc.tracks[0].add_note(2.0, 1.0, 64)           # Zeile 8, Luecke dazwischen
    song, warnings = to_tracker_song(doc)

    data = song.patterns[0].data[0]
    assert data[0] == 60
    assert data[4] == NOTE_OFF                     # Note endet bei Zeile 4 -> Cut
    assert data[8] == 64


def test_legato_note_not_left_with_dangling_note_off():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 1.0, 60)           # Zeile 0..3
    doc.tracks[0].add_note(1.0, 1.0, 64)           # Zeile 4, direkt anschliessend
    song, warnings = to_tracker_song(doc)

    data = song.patterns[0].data[0]
    assert data[0] == 60
    assert data[4] == 64                            # keine NOTE_OFF-Ueberreste


def test_staccato_note_gets_earlier_note_off():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 4.0, 60, staccato=True)   # Ganze Note, staccato
    song, warnings = to_tracker_song(doc)

    data = song.patterns[0].data[0]
    assert data[0] == 60
    assert data[8] == NOTE_OFF          # Haelfte von 16 Zeilen -> Zeile 8
    assert all(v is None for v in data[1:8])


def test_non_staccato_note_keeps_full_length():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 4.0, 60, staccato=False)
    song, warnings = to_tracker_song(doc)
    data = song.patterns[0].data[0]
    assert data[0] == 60
    assert all(v is None for v in data[1:])       # kein vorzeitiges NOTE_OFF


def test_staccato_very_short_note_keeps_at_least_one_row():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 0.25, 60, staccato=True)  # 1 Zeile -- kann nicht kuerzer
    song, warnings = to_tracker_song(doc)
    data = song.patterns[0].data[0]
    assert data[0] == 60
    # Zeile 1 ist bereits die naechste Note-Slot-Grenze -- kein Crash/keine
    # Verkuerzung unter eine Zeile.
    assert not any("Pattern-Grenze" in w for w in warnings)


def test_to_tracker_song_never_mutates_doc():
    doc = ScoreDoc()
    doc.tracks[0].add_note(0.0, 1.0, 60)
    doc.add_track("Bass", clef="bass")
    doc.tracks[1].add_note(0.5, 0.5, 40)
    before = doc.to_dict()

    to_tracker_song(doc)

    assert doc.to_dict() == before
