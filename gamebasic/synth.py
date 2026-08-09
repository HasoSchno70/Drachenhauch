"""Geteilte Synthesizer-Logik (sfxr-Stil) -- Qt-frei, pure numpy.

Single-Source fuer den SFX-Generator (`sfxeditor_qt.py`) UND den
`AUDIO_SFX`-Builtin (`modules/audio.py`). Die native Runtime `dhrt`
repliziert dieselbe Mathematik in Rust (`rust/drachenhauch_runtime/src/audio.rs`),
damit der Effekt in beiden Pfaden gleich klingt.

`synthesize(...)` liefert ein Float-Array in [-1, 1] mit angewandter
Attack/Sustain/Decay-Huellkurve, OHNE Lautstaerke (die wird beim Bau des
Sounds bzw. beim Abspielen aufmultipliziert).
"""
from __future__ import annotations

import numpy as np

SAMPLE_RATE = 44100
WAVEFORMS = ("square", "saw", "sine", "triangle", "noise")


def svf_lowpass(wave: np.ndarray, cutoff: float, sweep: float,
                res: float, sr: int) -> np.ndarray:
    """Resonanter Tiefpass (Chamberlin State-Variable-Filter) mit ueber die
    Zeit gleitender Grenzfrequenz -- der SID/Acid-Sweep.

    `cutoff` Hz (Startfrequenz), `sweep` Hz/s (linear ueber die Note),
    `res` 0..0.95 (Resonanz; hoeher = staerkere Betonung an der Grenze).
    cutoff <= 0 -> Bypass. Identisch in Rust (`audio.rs::svf_lowpass`)
    gehalten, damit Editor-Vorschau und Engine gleich klingen.
    """
    n = wave.size
    if cutoff <= 0.0 or n == 0:
        return wave
    q = 1.0 - min(0.95, max(0.0, res))        # Daempfung: kleiner = mehr Resonanz
    nyq = sr * 0.5
    low = 0.0
    band = 0.0
    out = np.empty(n, dtype=np.float64)
    for i in range(n):
        fc = cutoff + sweep * (i / sr)
        fc = min(nyq * 0.99, max(10.0, fc))
        f = 2.0 * np.sin(np.pi * fc / sr)
        low += f * band
        high = float(wave[i]) - low - q * band
        band += f * high
        out[i] = low
    return np.clip(out, -1.0, 1.0)


def _mono(wf: str, base_freq: float, slide: float, n: int, na: int, nd: int,
          vib_depth: float, vib_speed: float, sr: int,
          duty: float = 0.5, pwm_depth: float = 0.0, pwm_speed: float = 0.0,
          flt_cutoff: float = 0.0, flt_sweep: float = 0.0,
          flt_res: float = 0.0) -> np.ndarray:
    """Ein Mono-Kanal [-1,1] mit Pitch-Slide + Vibrato + ADSR (ohne Volume).

    SID-Erweiterungen (alle Defaults = unveraendertes Verhalten):
    `duty`/`pwm_*` = Pulsbreite + ihre Modulation fuer `square`;
    `flt_*` = resonanter Tiefpass-Sweep ueber den ganzen Ton."""
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
            # Pulsbreite: ph mod 1 < duty. duty=0.5 == sign(sin) (alt).
            frac = ph - np.floor(ph)
            duty_t: float | np.ndarray = float(duty)
            if pwm_depth > 0 and pwm_speed > 0:
                duty_t = duty + pwm_depth * np.sin(2.0 * np.pi * pwm_speed * t)
            duty_t = np.clip(duty_t, 0.05, 0.95)
            wave = np.where(frac < duty_t, 1.0, -1.0)
        elif wf == "saw":
            wave = 2.0 * (ph - np.floor(0.5 + ph))
        elif wf == "triangle":
            wave = 2.0 * np.abs(2.0 * (ph - np.floor(0.5 + ph))) - 1.0
        else:
            raise ValueError(f"unbekannte Waveform '{wf}'")
    if flt_cutoff > 0:
        wave = svf_lowpass(wave, flt_cutoff, flt_sweep, flt_res, sr)
    env = np.ones(n)
    if na > 0:
        env[:na] = np.linspace(0.0, 1.0, na)
    if nd > 0:
        env[-nd:] = np.linspace(1.0, 0.0, nd)
    return np.clip(wave * env, -1.0, 1.0)


def synthesize(waveform: str, base_freq: float, slide: float,
               attack_ms: int, sustain_ms: int, decay_ms: int,
               vib_depth: float = 0.0, vib_speed: float = 0.0,
               stereo_width: float = 0.0, sr: int = SAMPLE_RATE,
               duty: float = 0.5, pwm_depth: float = 0.0,
               pwm_speed: float = 0.0, flt_cutoff: float = 0.0,
               flt_sweep: float = 0.0, flt_res: float = 0.0) -> np.ndarray:
    """sfxr-Stil-Synthese. Rueckgabe: Float [-1, 1], OHNE Volume.

    `stereo_width` in (0, 1] erzeugt einen STEREO-Effekt (Form `(n, 2)`):
    der rechte Kanal wird leicht verstimmt (Detune, "breiter"); bei `noise`
    ist er unabhaengig (dekorreliert = sehr breit). 0 -> Mono `(n,)`.

    SID-Parameter (`duty`/`pwm_*` Pulsbreite+Modulation fuer square,
    `flt_*` resonanter Tiefpass-Sweep) -- Defaults reproduzieren exakt den
    bisherigen Klang."""
    wf = waveform.lower()
    total_ms = max(1, int(attack_ms) + int(sustain_ms) + int(decay_ms))
    n = max(1, int(sr * total_ms / 1000.0))
    na = int(n * int(attack_ms) / total_ms)
    nd = int(n * int(decay_ms) / total_ms)
    extra = (duty, pwm_depth, pwm_speed, flt_cutoff, flt_sweep, flt_res)
    left = _mono(wf, base_freq, slide, n, na, nd, vib_depth, vib_speed, sr,
                 *extra)
    if stereo_width <= 0:
        return left
    w = min(1.0, stereo_width)
    if wf == "noise":
        right = _mono(wf, base_freq, slide, n, na, nd, vib_depth, vib_speed,
                      sr, *extra)
    else:
        detune = 1.0 + 0.04 * w           # bis 4% Verstimmung = Chorus-Breite
        right = _mono(wf, base_freq * detune, slide * detune, n, na, nd,
                      vib_depth, vib_speed, sr, *extra)
    return np.column_stack([left, right])
