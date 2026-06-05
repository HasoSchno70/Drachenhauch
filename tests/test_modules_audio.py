"""Tests fuer das audio-Modul (Tree-Walker = konsolen-only).

Audio-Wiedergabe/Synthese-Builtins (AUDIO_TONE/NOISE/SFX/PLAY/MUSIC_*) laufen
nur in der nativen Runtime (gbrt) und werfen im Tree-Walker eine klare Meldung
-- abgedeckt durch die gbrt-Golden-Tests, nicht hier. Hier bleibt die *reine*
Synth-Mathematik (gamebasic.synth), die der Builtin und der gbsfx-Export teilen.
"""
import numpy as np
import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError


@pytest.fixture(scope="module", autouse=True)
def _load_audio():
    assert load_module("audio")


# --- Native-only-Gate -----------------------------------------------

def test_audio_tone_native_only(call_builtin):
    """AUDIO_TONE wirft im konsolen-only Tree-Walker eine klare gbrt-Meldung."""
    with pytest.raises(GBRuntimeError, match="nativen Runtime"):
        call_builtin("audio_tone", [440.0, 50])


# --- Reine Synth-Mathematik (gamebasic.synth) -----------------------

def test_synth_stereo_shape_and_channels():
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
    from gamebasic.synth import synthesize
    w = synthesize("square", 440.0, 0.0, 0, 50, 50, sr=44100)
    assert w.shape[0] == int(44100 * 100 / 1000)
    assert abs(w[-1]) < 0.1         # Decay laeuft am Ende auf ~0 aus
    assert np.abs(w).max() <= 1.0
    # Mit Attack-Ramp startet das Signal bei ~0.
    wa = synthesize("square", 440.0, 0.0, 30, 30, 30, sr=44100)
    assert abs(wa[0]) < 0.1
