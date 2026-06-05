"""Tests fuer das Instrument-Modell (Synth + Sample, Resampling) -- Qt-frei."""
import io
import wave

import numpy as np

from gamebasic.tracker.instrument import (
    Instrument, load_wav_mono, _decode_pcm, midi_to_freq,
)


def _sine(freq, secs, sr=44100):
    t = np.arange(int(sr * secs)) / sr
    return np.sin(2 * np.pi * freq * t).astype(np.float32)


def _write_wav(path, samples, sr=44100, nch=1):
    i16 = (np.clip(samples, -1, 1) * 32767).astype("<i2")
    if nch == 2:
        i16 = np.column_stack([i16, i16]).reshape(-1)
    with wave.open(str(path), "wb") as w:
        w.setnchannels(nch)
        w.setsampwidth(2)
        w.setframerate(sr)
        w.writeframes(i16.tobytes())


# --- Synth-Instrument ----------------------------------------------

def test_synth_render_length_and_nonsilent():
    inst = Instrument.synth("Lead", "square")
    out = inst.render_note(60, 4410)        # 0.1s @ 44100
    assert out.shape == (4410,)
    assert np.max(np.abs(out)) > 0.1        # nicht still


def test_synth_default_kind():
    inst = Instrument.synth("X", "saw")
    assert inst.kind == "synth"
    assert inst.is_sample() is False


# --- WAV laden -----------------------------------------------------

def test_load_wav_mono(tmp_path):
    p = tmp_path / "a.wav"
    _write_wav(p, _sine(440, 0.2))
    samples, sr = load_wav_mono(str(p))
    assert sr == 44100
    assert len(samples) == int(44100 * 0.2)
    assert -1.0 <= float(samples.min()) and float(samples.max()) <= 1.0


def test_load_wav_stereo_downmix(tmp_path):
    p = tmp_path / "s.wav"
    _write_wav(p, _sine(330, 0.1), nch=2)
    samples, sr = load_wav_mono(str(p))
    assert len(samples) == int(44100 * 0.1)   # auf Mono gemittelt


def test_decode_pcm_8bit():
    raw = bytes([128, 255, 0, 128])          # center, max, min, center
    a = _decode_pcm(raw, nch=1, sampwidth=1)
    assert abs(a[0]) < 0.01
    assert a[1] > 0.9 and a[2] < -0.9


# --- Resampling / Tonhoehe -----------------------------------------

def test_sample_render_at_base_note_matches_source_length():
    src = _sine(440, 0.5)                     # 22050 samples
    inst = Instrument.from_array("S", src, 44100, base_note=69)  # A4
    out = inst.render_note(69, 44100)        # base note -> step ~1
    # Ausgabe deckt ~ die ganze Quelle ab (Rest mit Stille auf n_samples)
    nonzero = np.count_nonzero(np.abs(out) > 1e-4)
    assert abs(nonzero - len(src)) < 200


def test_sample_octave_up_halves_consumed_source():
    src = _sine(440, 0.5)                     # 22050 samples
    inst = Instrument.from_array("S", src, 44100, base_note=69)
    out = inst.render_note(81, 44100)        # +12 Halbtoene = Oktave hoeher
    nonzero = np.count_nonzero(np.abs(out) > 1e-4)
    # Doppelte Geschwindigkeit -> halb so viele Ausgabe-Samples mit Inhalt
    assert abs(nonzero - len(src) // 2) < 300


def test_sample_octave_up_doubles_frequency():
    sr = 44100
    src = _sine(440, 1.0, sr)
    inst = Instrument.from_array("S", src, sr, base_note=69)
    out = inst.render_note(81, sr)           # sollte ~880 Hz sein
    seg = out[:8192]
    mag = np.abs(np.fft.rfft(seg))
    peak_hz = np.argmax(mag) * sr / len(seg)
    assert abs(peak_hz - 880) < 30


def test_render_note_truncates_to_n_samples():
    inst = Instrument.from_array("S", _sine(440, 1.0), 44100, 69)
    out = inst.render_note(69, 1000)
    assert out.shape == (1000,)


# --- Serialisierung ------------------------------------------------

def test_synth_dict_roundtrip():
    inst = Instrument.synth("Bass", "triangle")
    inst.default_vol = 9
    d = inst.to_dict()
    assert d["kind"] == "synth" and d["waveform"] == "triangle"
    inst2 = Instrument.from_dict(d)
    assert inst2.waveform == "triangle" and inst2.default_vol == 9


def test_sample_dict_roundtrip():
    src = _sine(220, 0.05)
    inst = Instrument.from_array("Kick", src, 22050, base_note=48)
    d = inst.to_dict()
    assert d["kind"] == "sample" and "samples" in d
    inst2 = Instrument.from_dict(d)
    assert inst2.kind == "sample"
    assert inst2.sample_rate == 22050 and inst2.base_note == 48
    # int16-Roundtrip -> kleine Toleranz
    assert np.allclose(inst2.samples, src, atol=1e-3)


def test_from_wav(tmp_path):
    p = tmp_path / "inst.wav"
    _write_wav(p, _sine(440, 0.1))
    inst = Instrument.from_wav(str(p), base_note=69)
    assert inst.kind == "sample"
    assert inst.name == "inst"
    assert inst.is_sample()
