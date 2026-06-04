"""Geteilte Synthesizer-Logik (sfxr-Stil) -- Qt-frei, pure numpy.

Single-Source fuer den SFX-Generator (`sfxeditor_qt.py`) UND den
`AUDIO_SFX`-Builtin (`modules/audio.py`). Die native Runtime `gbrt`
repliziert dieselbe Mathematik in Rust (`rust/gb_runtime/src/audio.rs`),
damit der Effekt in beiden Pfaden gleich klingt.

`synthesize(...)` liefert ein Float-Array in [-1, 1] mit angewandter
Attack/Sustain/Decay-Huellkurve, OHNE Lautstaerke (die wird beim Bau des
Sounds bzw. beim Abspielen aufmultipliziert).
"""
from __future__ import annotations

import numpy as np

SAMPLE_RATE = 44100
WAVEFORMS = ("square", "saw", "sine", "triangle", "noise")


def synthesize(waveform: str, base_freq: float, slide: float,
               attack_ms: int, sustain_ms: int, decay_ms: int,
               vib_depth: float = 0.0, vib_speed: float = 0.0,
               sr: int = SAMPLE_RATE) -> np.ndarray:
    """sfxr-Stil-Synthese mit Pitch-Slide (Phasen-Integration) + Vibrato +
    ADSR-Huellkurve. Rueckgabe: Float-Array [-1, 1], OHNE Volume."""
    wf = waveform.lower()
    total_ms = max(1, int(attack_ms) + int(sustain_ms) + int(decay_ms))
    n = max(1, int(sr * total_ms / 1000.0))
    t = np.arange(n, dtype=np.float64) / sr
    freq = base_freq + slide * t
    if vib_depth > 0 and vib_speed > 0:
        freq = freq * (1.0 + vib_depth * np.sin(2.0 * np.pi * vib_speed * t))
    freq = np.clip(freq, 20.0, sr / 2.0)
    if wf == "noise":
        wave = np.random.uniform(-1.0, 1.0, n)
    else:
        # Phasen-Integration, damit der Pitch-Slide sauber gleitet.
        phase = 2.0 * np.pi * np.cumsum(freq) / sr
        ph = phase / (2.0 * np.pi)
        if wf == "sine":
            wave = np.sin(phase)
        elif wf == "square":
            wave = np.where(np.sin(phase) >= 0, 1.0, -1.0)
        elif wf == "saw":
            wave = 2.0 * (ph - np.floor(0.5 + ph))
        elif wf == "triangle":
            wave = 2.0 * np.abs(2.0 * (ph - np.floor(0.5 + ph))) - 1.0
        else:
            raise ValueError(f"unbekannte Waveform '{waveform}'")
    na = int(n * int(attack_ms) / total_ms)
    nd = int(n * int(decay_ms) / total_ms)
    env = np.ones(n)
    if na > 0:
        env[:na] = np.linspace(0.0, 1.0, na)
    if nd > 0:
        env[-nd:] = np.linspace(1.0, 0.0, nd)
    return np.clip(wave * env, -1.0, 1.0)
