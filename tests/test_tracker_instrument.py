"""Tests fuer das Instrument-Modell (Synth + Sample, Resampling) -- Qt-frei."""
import io
import wave

import numpy as np

from drachenhauch.tracker.instrument import (
    Instrument, Zone, load_wav_mono, _decode_pcm, midi_to_freq,
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


def test_pan_dict_roundtrip():
    # Pan (-1..+1) ueberlebt to_dict/from_dict; Default 0.0; alte Dateien
    # ohne "pan" laden zentriert.
    for kind_inst in (Instrument.synth("L", "saw"),
                      Instrument.from_array("S", _sine(220, 0.02), 44100)):
        kind_inst.pan = -0.5
        back = Instrument.from_dict(kind_inst.to_dict())
        assert abs(back.pan - (-0.5)) < 1e-6
    legacy = Instrument.synth("X", "square").to_dict()
    legacy.pop("pan", None)
    assert Instrument.from_dict(legacy).pan == 0.0


def test_from_wav(tmp_path):
    p = tmp_path / "inst.wav"
    _write_wav(p, _sine(440, 0.1))
    inst = Instrument.from_wav(str(p), base_note=69)
    assert inst.kind == "sample"
    assert inst.name == "inst"
    assert inst.is_sample()


# --- Loop ----------------------------------------------------------

def test_no_loop_short_sample_has_trailing_silence():
    src = _sine(440, 0.05)                    # 2205 samples
    inst = Instrument.from_array("S", src, 44100, 69)
    out = inst.render_note(69, 44100)         # 1s angefordert
    assert out.shape == (44100,)
    assert np.max(np.abs(out[-1000:])) < 1e-3  # Ende still (kein Loop)


def test_forward_loop_fills_full_length():
    src = _sine(440, 0.05)
    inst = Instrument.from_array("S", src, 44100, 69)
    inst.loop_mode = "forward"
    inst.loop_start = 100
    inst.loop_end = 2000
    out = inst.render_note(69, 44100)
    # Mit Loop ist auch das Ende noch laut (Sustain durch Schleife)
    assert np.max(np.abs(out[20000:21000])) > 0.3


def test_pingpong_loop_fills_full_length():
    src = _sine(440, 0.05)
    inst = Instrument.from_array("S", src, 44100, 69)
    inst.loop_mode = "pingpong"
    inst.loop_start = 100
    inst.loop_end = 2000
    out = inst.render_note(69, 44100)
    assert np.max(np.abs(out[30000:31000])) > 0.3


def test_has_loop_validation():
    inst = Instrument.from_array("S", _sine(440, 0.05), 44100, 69)
    assert inst.has_loop() is False           # mode none
    inst.loop_mode = "forward"; inst.loop_start = 0; inst.loop_end = 0
    assert inst.has_loop() is False           # end <= start
    inst.loop_end = 1000
    assert inst.has_loop() is True


def test_has_loop_rejects_end_past_sample_length():
    """Review-Fund: has_loop() pruefte vorher nur `loop_end > loop_start`,
    nicht gegen die tatsaechliche Sample-Laenge -- anders als der
    render-Pfad (_resample's eigenes `loop_ok`), der das schon korrekt tat.
    Ein `has_loop()`-Aufrufer haette also faelschlich "ja" gehoert, obwohl
    render_note() dieselbe Region intern als ungueltig verwirft."""
    src = _sine(440, 0.05)          # 2205 Samples
    inst = Instrument.from_array("S", src, 44100, 69)
    inst.loop_mode = "forward"
    inst.loop_start = 100
    inst.loop_end = src.size + 500  # deutlich hinter dem Sample-Ende
    assert inst.has_loop() is False


# --- Envelope ------------------------------------------------------

def test_envelope_attack_starts_quiet_release_ends_quiet():
    src = np.ones(44100, dtype=np.float32)    # DC -> Envelope direkt sichtbar
    inst = Instrument.from_array("S", src, 44100, 69)
    inst.env_attack_ms = 50
    inst.env_release_ms = 50
    out = inst.render_note(69, 44100)
    assert abs(out[0]) < 0.05                  # Attack startet bei 0
    assert abs(out[-1]) < 0.05                  # Release endet bei 0
    assert out[22050] > 0.8                     # Mitte = Sustain (~1)


def test_envelope_passthrough_default_unchanged():
    src = np.full(1000, 0.5, np.float32)
    inst = Instrument.from_array("S", src, 44100, 69)
    out = inst.render_note(69, 1000)
    # Default-Envelope formt nichts (bis auf den 2ms-Anti-Click am Ende)
    assert np.allclose(out[:900], 0.5, atol=1e-3)


def test_envelope_short_note_with_long_release_is_not_silent():
    """Regression: `_adsr_env` klemmte die Release-Phase frueher nur gegen
    die Gesamtlaenge `n`, nicht gegen `n - na` (Ende der Attack-Phase). Bei
    einer Note, die kuerzer ist als Attack+Decay+Release zusammen (z.B. das
    "Fluegel (Piano)"-Preset: attack=2ms, decay=700ms, sustain=0, release=
    200ms, bei einer Note von nur ~125ms), ueberlappte die Release-Rampe die
    Attack-Phase und `start_lvl` griff auf einen Wert ~0 zu -- die GESAMTE
    Huellkurve wurde faelschlich zu einer Rampe von ~0 nach 0 (komplett
    stumme Note trotz gueltigem Waveform-Signal). Traf reale Presets bei
    dichten Patterns (viele kurze, aufeinanderfolgende Noten)."""
    src = np.ones(44100, dtype=np.float32)    # DC -> Huellkurve direkt sichtbar
    inst = Instrument.from_array("S", src, 44100, 69)
    inst.env_attack_ms = 2
    inst.env_decay_ms = 700
    inst.env_sustain = 0.0
    inst.env_release_ms = 200
    for n_samples in (500, 1000, 2000, 5512, 8000, 22050):
        out = inst.render_note(69, n_samples)
        assert np.max(np.abs(out)) > 0.1, (
            f"Note mit n_samples={n_samples} war stumm (ADSR-Regression)")


def test_envelope_release_never_overlaps_attack_phase():
    """Direkter Test von `_adsr_env`: die Release-Rampe darf nie in die
    Attack-Phase hineinreichen (sonst liest `start_lvl` einen falschen
    Wert und zerstoert die ganze Huellkurve, siehe Bugfix-Kommentar)."""
    from drachenhauch.tracker.instrument import _adsr_env
    env = _adsr_env(n=5512, attack_ms=2, decay_ms=700, sustain=0.0,
                    release_ms=200, sr=44100)
    assert env[0] == 0.0                       # Attack startet bei 0 (unveraendert)
    assert np.max(env) > 0.5                    # Huellkurve baut sich echt auf


# --- Keymap (Multisample / Drumkit) --------------------------------

def _zone(freq, lo, hi, root, secs=0.05, sr=44100):
    t = np.arange(int(sr * secs)) / sr
    return Zone(samples=np.sin(2 * np.pi * freq * t).astype(np.float32),
                sample_rate=sr, root_note=root, lo_key=lo, hi_key=hi,
                name=f"z{freq}")


def test_zone_covers_and_distance():
    z = _zone(440, 60, 67, 64)
    assert z.covers(64) and z.covers(60) and z.covers(67)
    assert not z.covers(59) and not z.covers(68)
    assert z.distance(64) == 0
    assert z.distance(57) == 3 and z.distance(70) == 3


def test_keymap_picks_zone_by_key():
    # Drumkit: drei verschiedene Frequenzen auf drei Tastenbereiche
    inst = Instrument.keymap("Kit", [
        _zone(200, 36, 47, 36),     # tiefer Bereich
        _zone(800, 48, 59, 48),     # mittlerer Bereich
        _zone(1500, 60, 71, 60),    # hoher Bereich
    ])
    assert inst.is_keymap()
    # Note in jedem Bereich am ROOT -> ~ Frequenz der jeweiligen Zone
    for note, freq in [(36, 200), (48, 800), (60, 1500)]:
        out = inst.render_note(note, 44100)
        seg = out[:8192]
        peak = np.argmax(np.abs(np.fft.rfft(seg))) * 44100 / len(seg)
        assert abs(peak - freq) < 40


def test_keymap_resamples_within_zone():
    inst = Instrument.keymap("M", [_zone(440, 60, 72, 60)])  # root 60
    out = inst.render_note(72, 44100)      # +12 -> Oktave hoeher = ~880
    seg = out[:8192]
    peak = np.argmax(np.abs(np.fft.rfft(seg))) * 44100 / len(seg)
    assert abs(peak - 880) < 40


def test_keymap_out_of_range_uses_nearest():
    inst = Instrument.keymap("M", [_zone(440, 60, 67, 64)])
    out = inst.render_note(40, 1000)       # weit unterhalb -> naechste Zone
    assert out.shape == (1000,)
    assert np.max(np.abs(out)) > 0.0       # klingt trotzdem


def test_keymap_dict_roundtrip():
    inst = Instrument.keymap("Kit", [
        _zone(200, 36, 47, 36), _zone(800, 48, 59, 48)])
    inst.env_attack_ms = 5
    d = inst.to_dict()
    assert d["kind"] == "keymap" and len(d["zones"]) == 2
    inst2 = Instrument.from_dict(d)
    assert inst2.is_keymap() and len(inst2.zones) == 2
    assert inst2.zones[0].lo_key == 36 and inst2.zones[1].root_note == 48
    assert inst2.env_attack_ms == 5


# --- Synth-Presets / Klangformung --------------------------------

def test_factory_instruments_list():
    from drachenhauch.tracker.presets import factory_instruments, preset_names
    insts = factory_instruments()
    assert len(insts) == len(preset_names())
    names = [i.name for i in insts]
    assert "Fluegel (Piano)" in names and "Orgel" in names and "Kick" in names
    assert all(i.kind == "synth" for i in insts)


def test_preset_has_envelope_shaping():
    from drachenhauch.tracker.presets import factory_instruments
    piano = next(i for i in factory_instruments() if i.name.startswith("Fluegel"))
    # Piano dekayt (sustain 0) -> Ende leiser als Anfang
    out = piano.render_note(60, 44100)
    assert np.max(np.abs(out[:2000])) > np.max(np.abs(out[-2000:]))


def test_organ_sustains():
    from drachenhauch.tracker.presets import factory_instruments
    organ = next(i for i in factory_instruments() if i.name == "Orgel")
    out = organ.render_note(60, 44100)
    # Orgel haelt -> Mitte etwa so laut wie frueh
    assert np.max(np.abs(out[20000:21000])) > 0.3


def test_synth_slide_in_render():
    inst = Instrument.synth("L", "saw")
    base = inst.render_note(60, 8192)
    slid = inst.render_note(60, 44100, slide=12)   # eine Oktave hoch ueber n
    # Mit Slide steigt die Frequenz -> spaeterer Abschnitt hat hoehere Peak-Hz
    def peak(seg):
        return np.argmax(np.abs(np.fft.rfft(seg))) * 44100 / len(seg)
    assert peak(slid[35000:43192]) > peak(slid[:8192]) + 30


def test_synth_detune_vib_roundtrip():
    inst = Instrument.synth("Pad", "saw")
    inst.env_attack_ms = 100; inst.env_sustain = 0.8
    inst.vib_depth = 0.05; inst.vib_speed = 5; inst.detune_cents = 14
    inst2 = Instrument.from_dict(inst.to_dict())
    assert inst2.waveform == "saw" and inst2.env_attack_ms == 100
    assert abs(inst2.vib_depth - 0.05) < 1e-6 and inst2.vib_speed == 5
    assert abs(inst2.detune_cents - 14) < 1e-6


def test_loop_env_dict_roundtrip():
    inst = Instrument.from_array("S", _sine(220, 0.05), 44100, 60)
    inst.loop_mode = "pingpong"; inst.loop_start = 50; inst.loop_end = 900
    inst.env_attack_ms = 10; inst.env_decay_ms = 20
    inst.env_sustain = 0.6; inst.env_release_ms = 30
    inst2 = Instrument.from_dict(inst.to_dict())
    assert inst2.loop_mode == "pingpong"
    assert inst2.loop_start == 50 and inst2.loop_end == 900
    assert inst2.env_attack_ms == 10 and inst2.env_decay_ms == 20
    assert abs(inst2.env_sustain - 0.6) < 1e-6 and inst2.env_release_ms == 30
