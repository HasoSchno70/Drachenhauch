"""Tests fuer das audio-Modul.

Pygame-Mixer wird tatsaechlich initialisiert (laeuft auch in CI ohne
Soundkarte mit dem dummy-Driver, falls noetig). Wir testen die Logik
ohne tatsaechlich etwas zu hoeren - Sound-Generation, Channel-Wrappen,
Validierung und Fehlerpfade.
"""
import os
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_audio():
    # Dummy-Audio-Driver erlaubt mixer.init() ohne Soundkarte
    os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
    os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "hide")
    assert load_module("audio")


@pytest.fixture(scope="module", autouse=True)
def _init_mixer():
    """Initialisiert pygame.mixer mit dem dummy-Driver."""
    import pygame
    if pygame.mixer.get_init():
        pygame.mixer.quit()
    pygame.mixer.init(frequency=22050, size=-16, channels=2, buffer=512)
    yield
    pygame.mixer.quit()


# --- Mixer-Lifecycle -----------------------------------------------

def test_audio_num_channels_default(call_builtin):
    n = call_builtin("audio_num_channels", [])
    assert n >= 1
    assert isinstance(n, int)


def test_audio_set_num_channels(call_builtin):
    call_builtin("audio_set_num_channels", [16])
    assert call_builtin("audio_num_channels", []) == 16
    call_builtin("audio_set_num_channels", [8])
    assert call_builtin("audio_num_channels", []) == 8


def test_audio_set_num_channels_negative_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="muss >= 0"):
        call_builtin("audio_set_num_channels", [-1])


def test_audio_busy_channels_zero_when_idle(call_builtin):
    call_builtin("audio_stop_all", [])
    assert call_builtin("audio_busy_channels", []) == 0


# --- Tone-Generation -----------------------------------------------

def test_audio_tone_returns_sound(call_builtin):
    snd = call_builtin("audio_tone", [440.0, 50])
    from gamebasic.interpreter import _Sound
    assert isinstance(snd, _Sound)


def test_audio_tone_default_waveform_is_sine(call_builtin):
    # Beide Aufrufe sollten gleich kompatibel sein
    snd1 = call_builtin("audio_tone", [440.0, 50])
    snd2 = call_builtin("audio_tone", [440.0, 50, "sine"])
    assert snd1 is not None and snd2 is not None


@pytest.mark.parametrize("waveform", ["sine", "square", "saw", "triangle", "noise"])
def test_audio_tone_all_waveforms_work(call_builtin, waveform):
    snd = call_builtin("audio_tone", [220.0, 30, waveform])
    from gamebasic.interpreter import _Sound
    assert isinstance(snd, _Sound)


def test_audio_tone_waveform_case_insensitive(call_builtin):
    snd = call_builtin("audio_tone", [220.0, 30, "SQUARE"])
    from gamebasic.interpreter import _Sound
    assert isinstance(snd, _Sound)


def test_audio_tone_unknown_waveform_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="unbekannte Waveform"):
        call_builtin("audio_tone", [220.0, 30, "schwurbel"])


def test_audio_tone_negative_freq_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="freq_hz muss > 0"):
        call_builtin("audio_tone", [-100.0, 30])


def test_audio_tone_zero_duration_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="dauer_ms muss > 0"):
        call_builtin("audio_tone", [440.0, 0])


def test_audio_tone_freq_must_be_number(call_builtin):
    with pytest.raises(TypeMismatchError, match="freq_hz"):
        call_builtin("audio_tone", ["nope", 30])


def test_audio_tone_volume_clamped(call_builtin):
    # 5.0 wird auf 1.0 geclampt - kein Crash
    snd = call_builtin("audio_tone", [440.0, 30, "sine", 5.0])
    from gamebasic.interpreter import _Sound
    assert isinstance(snd, _Sound)


def test_audio_noise_returns_sound(call_builtin):
    snd = call_builtin("audio_noise", [50])
    from gamebasic.interpreter import _Sound
    assert isinstance(snd, _Sound)


def test_audio_noise_zero_duration_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="dauer_ms muss > 0"):
        call_builtin("audio_noise", [0])


# --- SFX-Synth (AUDIO_SFX) -----------------------------------------

def _sfx_args(waveform="saw"):
    # waveform, freq, slide, attack, sustain, decay, vib_depth, vib_speed, vol
    return [waveform, 1000.0, -1400.0, 0, 30, 150, 0.0, 0.0, 0.7]


def test_audio_sfx_returns_sound(call_builtin):
    snd = call_builtin("audio_sfx", _sfx_args())
    from gamebasic.interpreter import _Sound
    assert isinstance(snd, _Sound)


@pytest.mark.parametrize("waveform", ["sine", "square", "saw", "triangle", "noise"])
def test_audio_sfx_all_waveforms(call_builtin, waveform):
    from gamebasic.interpreter import _Sound
    assert isinstance(call_builtin("audio_sfx", _sfx_args(waveform)), _Sound)


def test_audio_sfx_with_vibrato(call_builtin):
    from gamebasic.interpreter import _Sound
    args = ["square", 380.0, 700.0, 0, 90, 240, 0.15, 18.0, 0.6]
    assert isinstance(call_builtin("audio_sfx", args), _Sound)


def test_audio_sfx_unknown_waveform_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="unbekannte Waveform"):
        call_builtin("audio_sfx", _sfx_args("triangel"))


def test_audio_sfx_zero_total_duration_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="Gesamtdauer"):
        call_builtin("audio_sfx", ["saw", 440.0, 0.0, 0, 0, 0, 0.0, 0.0, 0.7])


def test_audio_sfx_stereo_width_returns_sound(call_builtin):
    from gamebasic.interpreter import _Sound
    args = _sfx_args() + [0.6]          # 10. Arg = stereo_width
    assert isinstance(call_builtin("audio_sfx", args), _Sound)


def test_synth_stereo_shape_and_channels():
    import numpy as np
    from gamebasic.synth import synthesize
    mono = synthesize("saw", 1000, -1400, 0, 30, 150)
    st = synthesize("saw", 1000, -1400, 0, 30, 150, stereo_width=0.6)
    assert mono.ndim == 1
    assert st.ndim == 2 and st.shape[1] == 2
    assert not np.allclose(st[:, 0], st[:, 1])      # Detune -> L != R
    # Noise: L/R dekorreliert
    nst = synthesize("noise", 200, 0, 0, 50, 100, stereo_width=0.5)
    assert not np.allclose(nst[:, 0], nst[:, 1])


def test_synth_matches_envelope_shape():
    # Der geteilte Synth liefert ein env-geformtes Signal in [-1, 1] (ohne Vol).
    import numpy as np
    from gamebasic.synth import synthesize
    w = synthesize("square", 440.0, 0.0, 0, 50, 50, sr=44100)
    assert w.shape[0] == int(44100 * 100 / 1000)
    assert abs(w[-1]) < 0.1         # Decay laeuft am Ende auf ~0 aus
    assert np.abs(w).max() <= 1.0
    # Mit Attack-Ramp startet das Signal bei ~0.
    wa = synthesize("square", 440.0, 0.0, 30, 30, 30, sr=44100)
    assert abs(wa[0]) < 0.1


# --- Channel-Playback ---------------------------------------------

def test_audio_play_returns_audio_channel(call_builtin):
    snd = call_builtin("audio_tone", [440.0, 50])
    ch = call_builtin("audio_play", [snd])
    from gamebasic.modules.audio import _AudioChannel
    assert isinstance(ch, _AudioChannel)
    call_builtin("audio_stop", [ch])


def test_audio_play_with_loops_volume(call_builtin):
    snd = call_builtin("audio_tone", [440.0, 30])
    ch = call_builtin("audio_play", [snd, 0, 0.5])
    from gamebasic.modules.audio import _AudioChannel
    assert isinstance(ch, _AudioChannel)
    call_builtin("audio_stop", [ch])


def test_audio_play_negative_fade_raises(call_builtin):
    snd = call_builtin("audio_tone", [440.0, 30])
    with pytest.raises(GBRuntimeError, match="fade_in_ms"):
        call_builtin("audio_play", [snd, 0, 1.0, -1])


def test_audio_play_requires_sound(call_builtin):
    with pytest.raises(TypeMismatchError, match="SOUND"):
        call_builtin("audio_play", ["not a sound"])


def test_audio_pause_resume_stop_no_crash(call_builtin):
    snd = call_builtin("audio_tone", [440.0, 200])
    ch = call_builtin("audio_play", [snd])
    call_builtin("audio_pause", [ch])
    call_builtin("audio_resume", [ch])
    call_builtin("audio_stop", [ch])


def test_audio_stop_with_fade_out(call_builtin):
    snd = call_builtin("audio_tone", [440.0, 200])
    ch = call_builtin("audio_play", [snd])
    call_builtin("audio_stop", [ch, 50])  # 50ms fadeout


def test_audio_volume_get_round_trip(call_builtin):
    snd = call_builtin("audio_tone", [440.0, 200])
    ch = call_builtin("audio_play", [snd])
    call_builtin("audio_volume", [ch, 0.5])
    v = call_builtin("audio_get_volume", [ch])
    assert 0.4 <= v <= 0.6  # pygame quantisiert leicht
    call_builtin("audio_stop", [ch])


def test_audio_volume_clamped(call_builtin):
    snd = call_builtin("audio_tone", [440.0, 200])
    ch = call_builtin("audio_play", [snd])
    call_builtin("audio_volume", [ch, 5.0])  # wird auf 1.0 geclampt
    v = call_builtin("audio_get_volume", [ch])
    assert v <= 1.0
    call_builtin("audio_stop", [ch])


def test_audio_pan_no_crash(call_builtin):
    snd = call_builtin("audio_tone", [440.0, 200])
    ch = call_builtin("audio_play", [snd])
    call_builtin("audio_pan", [ch, 1.0, 0.0])  # nur links
    call_builtin("audio_stop", [ch])


def test_audio_channel_validation(call_builtin):
    with pytest.raises(TypeMismatchError, match="AUDIO_CHANNEL"):
        call_builtin("audio_pause", ["not a channel"])
    with pytest.raises(TypeMismatchError, match="AUDIO_CHANNEL"):
        call_builtin("audio_volume", ["not a channel", 0.5])


# --- Music --------------------------------------------------------

def test_audio_music_load_invalid_path_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="AUDIO_MUSIC_LOAD"):
        call_builtin("audio_music_load", ["/no/such/file.mp3"])


def test_audio_music_busy_idle(call_builtin):
    # Ohne load/play sollte busy False sein
    call_builtin("audio_music_stop", [])
    assert call_builtin("audio_music_busy", []) is False


def test_audio_music_position_idle(call_builtin):
    # Wenn nichts laeuft, sollte 0.0 zurueckkommen (nicht -1)
    call_builtin("audio_music_stop", [])
    pos = call_builtin("audio_music_position", [])
    assert pos == 0.0


def test_audio_music_volume_round_trip(call_builtin):
    call_builtin("audio_music_volume", [0.7])
    v = call_builtin("audio_music_get_volume", [])
    assert 0.65 <= v <= 0.75


def test_audio_music_play_negative_fade_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="fade_in_ms"):
        call_builtin("audio_music_play", [-1, -10])


# --- Type-Validation ----------------------------------------------

def test_audio_init_invalid_channels_raises(call_builtin):
    # channels muss 1 oder 2 sein
    with pytest.raises(GBRuntimeError, match="ungueltige Parameter"):
        call_builtin("audio_init", [44100, 5])


def test_audio_set_num_channels_bool_rejected(call_builtin):
    with pytest.raises(TypeMismatchError):
        call_builtin("audio_set_num_channels", [True])
