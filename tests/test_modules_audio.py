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
