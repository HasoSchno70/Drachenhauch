"""Tests fuer den Software-Mixer + Render-to-File (Qt-frei)."""
import wave

import numpy as np

from gamebasic.tracker import Song, Instrument
from gamebasic.tracker.mixer import (render_song, save_wav, _note_events,
                                     apply_effect)
from gamebasic.tracker.song import FX_ARP, FX_RET, FX_OFF, FX_VIB


def _sample_inst(name="S", freq=440, secs=0.2, sr=44100, base=69):
    t = np.arange(int(sr * secs)) / sr
    return Instrument.from_array(name, np.sin(2 * np.pi * freq * t), sr, base)


def test_note_events_walks_order():
    s = Song()
    s.patterns[0].set_rows(4)
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set(0, 2, 64)
    s.patterns[0].set(1, 1, 67)
    ev = _note_events(s)
    assert ev[0] == [(0, 60, None, None, 0, 0), (2, 64, None, None, 0, 0)]
    assert ev[1] == [(1, 67, None, None, 0, 0)]


def test_render_song_nonsilent_and_length():
    s = Song()
    s.bpm = 120
    s.patterns[0].set_rows(4)
    s.patterns[0].set(0, 0, 60)
    out = render_song(s)
    row_samples = int(44100 * s.row_ms() / 1000.0)
    assert out.shape[0] == 4 * row_samples + int(44100 * 0.8)
    assert np.max(np.abs(out)) > 0.05        # Synth-Note hoerbar


def test_render_song_with_sample_instrument():
    s = Song()
    s.patterns[0].set_rows(4)
    idx = s.add_instrument(_sample_inst())
    s.channel_inst[0] = idx
    s.patterns[0].set(0, 0, 69)              # Grundton -> 1:1
    out = render_song(s)
    assert np.max(np.abs(out)) > 0.05


def test_render_song_note_sustains_until_next():
    s = Song()
    s.patterns[0].set_rows(8)
    # Looping-Sample, das endlos klingt
    inst = _sample_inst(secs=0.05)
    inst.loop_mode = "forward"; inst.loop_start = 50; inst.loop_end = 1000
    idx = s.add_instrument(inst)
    s.channel_inst[0] = idx
    s.patterns[0].set(0, 0, 69)              # eine Note in Reihe 0
    out = render_song(s)
    row_samples = int(44100 * s.row_ms() / 1000.0)
    # Reihe 6 (lange nach dem Anschlag) ist dank Loop noch laut
    seg = out[6 * row_samples:7 * row_samples]
    assert np.max(np.abs(seg)) > 0.2


def test_render_normalizes_on_clip():
    s = Song()
    s.patterns[0].set_rows(2)
    # zwei DC-Samples gleichzeitig -> wuerde > 1 summieren
    dc = Instrument.from_array("DC", np.ones(44100, np.float32), 44100, 69)
    i = s.add_instrument(dc)
    s.channel_inst[0] = i
    s.channel_inst[1] = i
    s.patterns[0].set(0, 0, 69)
    s.patterns[0].set(1, 0, 69)
    out = render_song(s)
    assert np.max(np.abs(out)) <= 1.0001


def test_save_wav_roundtrip(tmp_path):
    s = Song()
    s.patterns[0].set(0, 0, 60)
    out = render_song(s)
    p = tmp_path / "song.wav"
    save_wav(str(p), out)
    with wave.open(str(p), "rb") as w:
        assert w.getnchannels() == 1
        assert w.getsampwidth() == 2
        assert w.getframerate() == 44100
        assert w.getnframes() == len(out)


def test_render_stereo_shape_and_panning():
    # Instrument hart nach rechts gepannt -> rechter Kanal lauter als linker.
    s = Song()
    s.patterns[0].set_rows(4)
    inst = _sample_inst()
    inst.pan = 1.0
    idx = s.add_instrument(inst)
    s.channel_inst[0] = idx
    s.patterns[0].set(0, 0, 69)
    out = render_song(s, stereo=True)
    assert out.ndim == 2 and out.shape[1] == 2
    left = float(np.max(np.abs(out[:, 0])))
    right = float(np.max(np.abs(out[:, 1])))
    assert right > left * 4          # klar rechts


def test_render_hard_pan_amiga_splits_channels():
    # Amiga-Konvention: Kanal 0 sitzt links. Eine Note nur auf Kanal 0 mit
    # hard_pan -> linker Kanal deutlich lauter als der rechte.
    s = Song()
    s.patterns[0].set_rows(4)
    idx = s.add_instrument(_sample_inst())
    s.channel_inst[0] = idx
    s.patterns[0].set(0, 0, 69)
    out = render_song(s, stereo=True, hard_pan=True)
    left = float(np.max(np.abs(out[:, 0])))
    right = float(np.max(np.abs(out[:, 1])))
    assert left > right * 2          # Kanal 0 klar links


def test_save_wav_stereo_roundtrip(tmp_path):
    s = Song()
    s.patterns[0].set(0, 0, 60)
    out = render_song(s, stereo=True)
    p = tmp_path / "song_stereo.wav"
    save_wav(str(p), out)
    with wave.open(str(p), "rb") as w:
        assert w.getnchannels() == 2
        assert w.getnframes() == out.shape[0]


def test_sample_slide_changes_pitch():
    # Ein gleitendes Sample (slide > 0) endet hoeher als ohne Slide:
    # die mittlere Nulldurchgangsrate steigt. Wir pruefen, dass Slide
    # ueberhaupt eine andere Wellenform erzeugt (vorher fuer Samples ignoriert).
    inst = _sample_inst(freq=200, secs=1.0)
    n = 22050
    flat = inst.render_note(69, n, 44100, slide=0)
    glide = inst.render_note(69, n, 44100, slide=12)   # +1 Oktave ueber die Note
    assert not np.allclose(flat, glide)
    # Zweite Haelfte des Glides hat mehr Nulldurchgaenge (hoeher) als das Flat.
    def zc(a):
        h = a[len(a)//2:]
        return int(np.sum(np.abs(np.diff(np.sign(h))) > 0))
    assert zc(glide) > zc(flat)


# --- Effekt-Spalte (Arpeggio/Vibrato/Retrigger/Sample-Offset) ---------------

def _tone(freq=440, secs=0.5, sr=44100):
    t = np.arange(int(sr * secs)) / sr
    return np.sin(2 * np.pi * freq * t).astype(np.float32)


def test_apply_effect_none_is_identity():
    buf = _tone()
    assert np.array_equal(apply_effect(buf, 0, 0, 44100, 125), buf)


def test_arpeggio_raises_pitch_in_some_ticks():
    # Arp +12/+7: spaetere Ticks lesen schneller -> mehr Nulldurchgaenge als
    # das unbearbeitete Original ueber dasselbe Fenster.
    buf = _tone(220, 0.6)
    arp = apply_effect(buf, FX_ARP, (12 << 4) | 7, 44100, 125)
    def zc(a):
        return int(np.sum(np.abs(np.diff(np.sign(a))) > 0))
    assert zc(arp) > zc(buf)
    assert arp.shape == buf.shape


def test_retrigger_tiles_head():
    buf = _tone(440, 0.5)
    ret = apply_effect(buf, FX_RET, 1, 44100, 125)   # alle 1 Tick neu
    assert ret.shape == buf.shape
    # Der Anfang wiederholt sich: 2. Tick-Block gleicht dem 1.
    tick = int(44100 * 125 / 1000 / 6)
    assert np.allclose(ret[:tick], ret[tick:2 * tick])


def test_sample_offset_shifts_forward():
    buf = _tone(440, 0.2)
    off = apply_effect(buf, FX_OFF, 4, 44100, 125)   # 4*512 Frames spaeter
    assert off.shape == buf.shape
    assert np.allclose(off[:len(buf) - 4 * 512], buf[4 * 512:])


def test_vibrato_modulates_but_keeps_length():
    buf = _tone(440, 0.4)
    vib = apply_effect(buf, FX_VIB, (6 << 4) | 8, 44100, 125)
    assert vib.shape == buf.shape
    assert not np.allclose(vib, buf)


def test_render_song_with_arpeggio_nonsilent():
    s = Song()
    s.patterns[0].set_rows(4)
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_fx(0, 0, FX_ARP, (4 << 4) | 7)
    out = render_song(s)
    assert np.max(np.abs(out)) > 0.05
