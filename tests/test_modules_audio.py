"""Tests fuer die reine Synth-Mathematik (`gamebasic.synth`).

Stufe B: Die Audio-Wiedergabe/Synthese-Builtins (AUDIO_TONE/NOISE/SFX/...) laufen
nur nativ in gbrt; der frueher hier getestete Tree-Walker-"nur nativ"-Gate
entfaellt mit dem Tree-Walker (Phase 8). Was bleibt, ist die geteilte Synth-
Mathematik in `gamebasic/synth.py` (von Builtin UND gbsfx-Export genutzt, reines
numpy -- in Phase 8 behalten).
"""
import numpy as np

from gamebasic.synth import synthesize, svf_lowpass


def test_synth_duty_default_unchanged():
    # duty=0.5 (Default) muss bit-genau die alte sign(sin)-Rechteckwelle sein.
    old = synthesize("square", 440.0, 0.0, 0, 100, 0, sr=44100)
    new = synthesize("square", 440.0, 0.0, 0, 100, 0, sr=44100, duty=0.5)
    assert np.array_equal(old, new)


def test_synth_narrow_duty_changes_wave():
    # Schmale Pulsbreite -> ueberwiegend bei -1 (anderer Klang).
    w = synthesize("square", 220.0, 0.0, 0, 100, 0, sr=44100, duty=0.1)
    assert float(np.mean(w)) < -0.3


def test_synth_filter_attenuates_high_freq():
    # 8 kHz durch tiefen Cutoff -> zweite Haelfte deutlich leiser.
    raw = synthesize("saw", 8000.0, 0.0, 0, 100, 0, sr=44100)
    flt = synthesize("saw", 8000.0, 0.0, 0, 100, 0, sr=44100, flt_cutoff=300.0)
    h = len(raw) // 2
    assert np.abs(flt[h:]).max() < np.abs(raw[h:]).max() * 0.6


def test_svf_bypass_when_cutoff_zero():
    sig = np.array([0.3, -0.7, 0.5, -0.2])
    assert np.array_equal(svf_lowpass(sig, 0.0, 0.0, 0.0, 44100), sig)


def test_synth_stereo_shape_and_channels():
    mono = synthesize("saw", 1000, -1400, 0, 30, 150)
    st = synthesize("saw", 1000, -1400, 0, 30, 150, stereo_width=0.6)
    assert mono.ndim == 1
    assert st.ndim == 2 and st.shape[1] == 2
    assert not np.allclose(st[:, 0], st[:, 1])      # Detune -> L != R
    # Noise: L/R dekorreliert
    nst = synthesize("noise", 200, 0, 0, 50, 100, stereo_width=0.5)
    assert not np.allclose(nst[:, 0], nst[:, 1])


def test_synth_matches_envelope_shape():
    w = synthesize("square", 440.0, 0.0, 0, 50, 50, sr=44100)
    assert w.shape[0] == int(44100 * 100 / 1000)
    assert abs(w[-1]) < 0.1         # Decay laeuft am Ende auf ~0 aus
    assert np.abs(w).max() <= 1.0
    # Mit Attack-Ramp startet das Signal bei ~0.
    wa = synthesize("square", 440.0, 0.0, 30, 30, 30, sr=44100)
    assert abs(wa[0]) < 0.1


# --- AUDIO_MUSIC_PLAY/STOP: Argument-Validierung (gbrt-Golden) ---------------
# Die Wiedergabe selbst braucht ein Audio-Geraet (nicht headless testbar);
# die Wrapper-Validierung in vm.rs laeuft aber VOR der Audio-Initialisierung
# und ist damit golden-testbar.

def test_music_play_stop_fade_validation(run_gb):
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_MUSIC_PLAY(-1, -5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_MUSIC_STOP(-1)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_MUSIC_PLAY: fade_in_ms muss >= 0 sein" in out
    assert "AUDIO_MUSIC_STOP: fade_out_ms muss >= 0 sein" in out


def test_play_stop_fade_validation(run_gb):
    # AUDIO_PLAY(sound[, loops[, volume[, fade_in_ms]]]) / AUDIO_STOP(ch[, fade_out_ms])
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PLAY(0, -1, 1.0, -5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_STOP(0, -1)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_PLAY: fade_in_ms muss >= 0 sein" in out
    assert "AUDIO_STOP: fade_out_ms muss >= 0 sein" in out


def test_pan_slide_validation(run_gb):
    # AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms) -- dauer_ms wird im Wrapper
    # (vor der Audio-Initialisierung) geprueft -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PAN_SLIDE(0, 0.0, 1.0, 0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    assert "AUDIO_PAN_SLIDE: dauer_ms muss > 0 sein" in run_gb(src)


def test_pitch_validation(run_gb):
    # faktor <= 0 wird im Wrapper (vor der Audio-Initialisierung) geprueft.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_PITCH(0, 0.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_MUSIC_PITCH(-1.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_PITCH: faktor muss > 0 sein" in out
    assert "AUDIO_MUSIC_PITCH: faktor muss > 0 sein" in out


def test_sample_type_compiles(run_gb):
    # SAMPLE ist ein externer Typ des audio-Moduls. DIM ... AS SAMPLE
    # initialisiert KEIN Audio-Geraet -> headless golden-testbar (verifiziert
    # die Typ-Verdrahtung preprocess MODULE_TYPES + Compiler).
    src = '\n'.join([
        'IMPORT "audio"',
        'DIM s AS SAMPLE',
        'PRINT "sample-typ ok"',
    ])
    assert "sample-typ ok" in run_gb(src)


def test_lofi_validation(run_gb):
    # AUDIO_LOFI(an[, bits[, cutoff_hz]]) -- Argument-Pruefung laeuft VOR der
    # Audio-Initialisierung -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_LOFI(TRUE, 99)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_LOFI(TRUE, 8, -1.0)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_LOFI: bits muss 1..16 sein" in out
    assert "AUDIO_LOFI: cutoff_hz muss >= 0 sein" in out


def test_bus_volume_validation(run_gb):
    # AUDIO_BUS_VOLUME/GET: unbekannter Bus wird VOR der Audio-Init geprueft
    # -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_BUS_VOLUME("foo", 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    PRINT STR$(AUDIO_BUS_GET_VOLUME("bar"))',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_BUS_VOLUME: unbekannter Bus 'foo' (sfx, music, master)" in out
    assert "AUDIO_BUS_GET_VOLUME: unbekannter Bus 'bar' (sfx, music, master)" in out


def test_bus_effects_validation(run_gb):
    # AUDIO_FILTER/REVERB/DELAY: unbekannter Bus wird VOR der Audio-Init
    # geprueft -> headless golden-testbar.
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_FILTER("nope", 1000, 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_REVERB("nope", 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_DELAY("nope", 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_FILTER: unbekannter Bus 'nope' (sfx, music, master)" in out
    assert "AUDIO_REVERB: unbekannter Bus 'nope' (sfx, music, master)" in out
    assert "AUDIO_DELAY: unbekannter Bus 'nope' (sfx, music, master)" in out


def test_bus_dynamics_eq_validation(run_gb):
    # AUDIO_DISTORTION/COMPRESSOR/EQ: unbekannter Bus -> klare Meldung vor
    # der Audio-Init (golden-testbar).
    src = '\n'.join([
        'IMPORT "audio"',
        'TRY',
        '    AUDIO_DISTORTION("nope", 0.5)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_COMPRESSOR("nope", -18, 4)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
        'TRY',
        '    AUDIO_EQ("nope", 100, 3)',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    out = run_gb(src)
    assert "AUDIO_DISTORTION: unbekannter Bus 'nope' (sfx, music, master)" in out
    assert "AUDIO_COMPRESSOR: unbekannter Bus 'nope' (sfx, music, master)" in out
    assert "AUDIO_EQ: unbekannter Bus 'nope' (sfx, music, master)" in out


def test_unloadsound_is_wired(run_gb):
    # UNLOADSOUND(s) gibt einen Sound-Puffer frei. Der Arg-Check (gi) laeuft
    # VOR der Audio-Initialisierung -> headless golden-testbar und beweist,
    # dass das Builtin verdrahtet ist (sonst "unbekannter Builtin").
    src = '\n'.join([
        'TRY',
        '    UNLOADSOUND()',
        'CATCH e',
        '    PRINT e',
        'END TRY',
    ])
    assert "UNLOADSOUND: fehlendes Argument 1" in run_gb(src)
