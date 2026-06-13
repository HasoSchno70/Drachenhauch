"""Tests fuer die reine Synth-Mathematik (`gamebasic.synth`).

Stufe B: Die Audio-Wiedergabe/Synthese-Builtins (AUDIO_TONE/NOISE/SFX/...) laufen
nur nativ in gbrt; der frueher hier getestete Tree-Walker-"nur nativ"-Gate
entfaellt mit dem Tree-Walker (Phase 8). Was bleibt, ist die geteilte Synth-
Mathematik in `gamebasic/synth.py` (von Builtin UND gbsfx-Export genutzt, reines
numpy -- in Phase 8 behalten).
"""
import numpy as np

from gamebasic.synth import synthesize


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
