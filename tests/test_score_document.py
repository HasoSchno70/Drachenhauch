"""Tests fuer das Notenblatt-Datenmodell (Qt-frei, gamebasic.score.document)."""
import pytest

from gamebasic.score.document import ScoreDoc, Track, NoteEvent, CLEFS
from gamebasic.tracker.instrument import Instrument


def test_note_event_rest_normalization():
    ev = NoteEvent(0.0, 1.0, pitch=None, rest=False)
    assert ev.rest is True
    assert ev.pitch is None

    ev2 = NoteEvent(0.0, 1.0, pitch=60, rest=True)
    assert ev2.rest is True
    assert ev2.pitch is None

    ev3 = NoteEvent(0.0, 1.0, pitch=60)
    assert ev3.rest is False
    assert ev3.pitch == 60


def test_note_event_end_beat():
    ev = NoteEvent(2.0, 1.5, pitch=60)
    assert ev.end_beat == 3.5


def test_note_event_roundtrip_pitched():
    ev = NoteEvent(1.0, 0.5, pitch=67)
    ev2 = NoteEvent.from_dict(ev.to_dict())
    assert ev2.start_beat == ev.start_beat
    assert ev2.dur_beat == ev.dur_beat
    assert ev2.pitch == ev.pitch
    assert ev2.rest is False


def test_note_event_roundtrip_rest():
    ev = NoteEvent(1.0, 0.5, rest=True)
    ev2 = NoteEvent.from_dict(ev.to_dict())
    assert ev2.rest is True
    assert ev2.pitch is None


def test_track_add_note_keeps_sorted():
    t = Track("T")
    t.add_note(2.0, 1.0, 60)
    t.add_note(0.0, 1.0, 62)
    t.add_note(1.0, 1.0, 64)
    assert [n.start_beat for n in t.notes] == [0.0, 1.0, 2.0]


def test_track_remove_note():
    t = Track("T")
    ev = t.add_note(0.0, 1.0, 60)
    t.remove_note(ev)
    assert t.notes == []
    t.remove_note(ev)  # no-op, kein Fehler


def test_track_length_beats_empty_and_nonempty():
    t = Track("T")
    assert t.length_beats() == 0.0
    t.add_note(0.0, 1.0, 60)
    t.add_note(2.0, 0.5, 64)
    assert t.length_beats() == 2.5


def test_track_default_instrument_is_triangle_synth():
    t = Track("Lead")
    assert t.instrument.kind == "synth"
    assert t.instrument.waveform == "triangle"


def test_track_clef_clamped_to_known_values():
    t = Track("T", clef="weird")
    assert t.clef == "treble"
    for c in CLEFS:
        assert Track("T", clef=c).clef == c


def test_track_roundtrip_with_instrument():
    t = Track("Bass", clef="bass", instrument=Instrument.synth("Bass", "square"))
    t.add_note(0.0, 1.0, 40)
    t2 = Track.from_dict(t.to_dict())
    assert t2.name == "Bass"
    assert t2.clef == "bass"
    assert t2.instrument.kind == "synth"
    assert t2.instrument.waveform == "square"
    assert len(t2.notes) == 1
    assert t2.notes[0].pitch == 40


def test_scoredoc_defaults():
    d = ScoreDoc()
    assert d.bpm == 120
    assert d.time_sig == (4, 4)
    assert len(d.tracks) == 1
    assert d.tracks[0].name == "Stimme 1"


def test_scoredoc_add_track_auto_name():
    d = ScoreDoc()
    idx = d.add_track()
    assert idx == 1
    assert d.tracks[1].name == "Stimme 2"


def test_scoredoc_remove_track_keeps_last():
    d = ScoreDoc()
    d.remove_track(0)
    assert len(d.tracks) == 1  # letzte Spur bleibt


def test_scoredoc_remove_track_normal_case():
    d = ScoreDoc()
    d.add_track("Zweite")
    d.remove_track(0)
    assert len(d.tracks) == 1
    assert d.tracks[0].name == "Zweite"


def test_scoredoc_length_beats():
    d = ScoreDoc()
    d.tracks[0].add_note(0.0, 4.0, 60)
    d.add_track("Bass")
    d.tracks[1].add_note(0.0, 2.0, 40)
    assert d.length_beats() == 4.0


def test_scoredoc_save_load_roundtrip(tmp_path):
    d = ScoreDoc()
    d.bpm = 90
    d.tracks[0].add_note(0.0, 1.0, 60)
    d.add_track("Bass", clef="bass")
    d.tracks[1].add_note(0.5, 0.5, 40)
    path = tmp_path / "song.json"
    d.save_json(str(path))

    d2 = ScoreDoc.load_json(str(path))
    assert d2.bpm == 90
    assert d2.time_sig == (4, 4)
    assert len(d2.tracks) == 2
    assert d2.tracks[1].clef == "bass"
    assert d2.tracks[0].notes[0].pitch == 60
    assert d2.path == str(path)

    saved = d.to_dict()
    assert saved["format"] == "gbscore-song"
    assert saved["version"] == 1


def test_scoredoc_from_dict_is_permissive_missing_fields():
    # Handbearbeitete/formatlose Datei ohne "format"/"tracks" -> laedt trotzdem.
    d = ScoreDoc.from_dict({})
    assert d.bpm == 120
    assert d.time_sig == (4, 4)
    assert len(d.tracks) == 1
