"""Tests fuer den additiven Software-Mixer (Qt-frei, gamebasic.audio_preview).

Betrifft ein ANDERES Modul als tests/test_tracker_mixer.py (das den
WAV-Render-Mixer gamebasic/tracker/mixer.py testet) -- rein additiv."""
import numpy as np

from gamebasic.audio_preview import Mixer


def test_play_ignores_none_and_empty():
    m = Mixer()
    m.play(None)
    m.play(np.zeros(0, dtype=np.float32))
    assert m._voices == []


def test_play_without_sounddevice_does_not_crash(monkeypatch):
    import builtins
    real_import = builtins.__import__

    def fake_import(name, *args, **kwargs):
        if name == "sounddevice":
            raise ImportError("kein Audio-Geraet in Testumgebung")
        return real_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", fake_import)
    m = Mixer()
    m.play(np.ones(10, dtype=np.float32))
    assert m._stream is None


def test_callback_mixes_overlapping_voices_additively():
    m = Mixer()
    a = np.ones(10, dtype=np.float32) * 0.5
    b = np.ones(6, dtype=np.float32) * 0.3
    with m._lock:
        m._voices.append((a, 0))
        m._voices.append((b, 0))

    out1 = np.zeros((4, 1), dtype=np.float32)
    m._callback(out1, 4, None, None)
    assert np.allclose(out1.ravel(), 0.8)

    out2 = np.zeros((4, 1), dtype=np.float32)
    m._callback(out2, 4, None, None)
    # b ist nach Sample 6 zu Ende -- die letzten zwei Samples dieses Blocks
    # kommen nur noch von a.
    assert np.allclose(out2.ravel(), [0.8, 0.8, 0.5, 0.5])

    out3 = np.zeros((4, 1), dtype=np.float32)
    m._callback(out3, 4, None, None)
    # a ist nach Sample 10 zu Ende -- ab hier Stille, keine Stimmen mehr.
    assert np.allclose(out3.ravel(), [0.5, 0.5, 0.0, 0.0])
    assert m._voices == []


def test_callback_clips_to_valid_range():
    m = Mixer()
    loud = np.ones(4, dtype=np.float32)
    with m._lock:
        m._voices.append((loud, 0))
        m._voices.append((loud, 0))
        m._voices.append((loud, 0))  # 3x 1.0 addiert > 1.0 -> muss geclippt werden

    out = np.zeros((4, 1), dtype=np.float32)
    m._callback(out, 4, None, None)
    assert np.max(out) <= 1.0
    assert np.min(out) >= -1.0
