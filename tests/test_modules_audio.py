"""Tests fuer die reine Synth-Mathematik (`drachenhauch.synth`).

Stufe B: Die Audio-Wiedergabe/Synthese-Builtins (AUDIO_TONE/NOISE/SFX/...) laufen
nur nativ in dhrt; der frueher hier getestete Tree-Walker-"nur nativ"-Gate
entfaellt mit dem Tree-Walker (Phase 8). Was bleibt, ist die geteilte Synth-
Mathematik in `drachenhauch/synth.py` (von Builtin UND dhsfx-Export genutzt, reines
numpy -- in Phase 8 behalten).

Alle Faelle, die dhrt laufen lassen, liegen als Pruefsammlung in
`tests/pruef/modules_audio.dhtest` (dhrt test); hier bleibt nur die
Python-Mathematik, die dhrt gar nicht benutzt.
"""
import numpy as np

from drachenhauch.synth import synthesize, svf_lowpass


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
