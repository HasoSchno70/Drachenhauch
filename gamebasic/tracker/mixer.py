"""Software-Mixer + Render-to-File fuer den Tracker (Qt-frei, numpy).

Mischt den ganzen Song offline zu einem Mono-Float-Stream: jede Note wird
ueber ihr Kanal-Instrument gerendert (`Instrument.render_note` -- inkl.
Resampling, Loop und ADSR) und an ihre Zeitposition gemischt. Eine Note
klingt bis zur naechsten Note desselben Kanals (klassisches Tracker-Sustain).

Damit lassen sich Songs mit Sample-Instrumenten als Audiodatei exportieren
(`render_song` -> `save_wav`) und im Spiel via `PLAYMUSIC` abspielen --
voellig unabhaengig von den Engine-Audio-Grenzen (kein Runtime-Resampling
noetig, weil hier offline gemischt wird).
"""
from __future__ import annotations

import wave

import numpy as np

from .song import CHANNELS, vol_to_pct

SAMPLE_RATE = 44100


def _note_events(song):
    """Pro Kanal eine Liste von (global_row, midi, vol_cell, slide_cell) fuer
    jede gesetzte Note -- die flache Timeline aus der Order."""
    events = {c: [] for c in range(CHANNELS)}
    i = 0
    for p in (song.order or [0]):
        if not (0 <= p < len(song.patterns)):
            continue
        pat = song.patterns[p]
        for r in range(pat.rows):
            for c in range(CHANNELS):
                note = pat.data[c][r]
                if note is not None:
                    events[c].append(
                        (i + r, int(note), pat.vol[c][r], pat.slide[c][r]))
        i += pat.rows
    return events


def render_song(song, sr: int = SAMPLE_RATE, tail_ms: int = 800) -> np.ndarray:
    """Rendert den ganzen Song zu einem Mono-Float-Array [-1, 1].

    Jede Note klingt bis zur naechsten Note desselben Kanals (Sustain ueber
    leere Reihen). `tail_ms` = zusaetzliche Stille am Ende, damit ein langes
    Sample/Release am Schluss ausklingen kann. Bei Uebersteuerung wird der
    gesamte Mix normalisiert.
    """
    row_samples = max(1, int(sr * song.row_ms() / 1000.0))
    total_rows, _ = song.flatten()
    tail = int(sr * max(0, tail_ms) / 1000.0)
    total = total_rows * row_samples + tail
    mix = np.zeros(max(1, total), dtype=np.float32)

    events = _note_events(song)
    for c in range(CHANNELS):
        inst = song.instrument_for_channel(c)
        evs = events[c]
        for k, (start_row, midi, volc, slidec) in enumerate(evs):
            end_row = evs[k + 1][0] if k + 1 < len(evs) else total_rows
            n = (end_row - start_row) * row_samples
            if n <= 0:
                continue
            # Sample/Loop darf bis zum Tail nachklingen.
            n_render = n + tail if k + 1 >= len(evs) else n
            # Slide gilt nur fuer Synth (render_note ignoriert ihn sonst).
            note = inst.render_note(midi, n_render, sr, slide=(slidec or 0))
            amp = (vol_to_pct(volc) / 100.0) if volc else (inst.default_vol / 15.0)
            start = start_row * row_samples
            seg = note[:max(0, mix.size - start)]
            if seg.size:
                mix[start:start + seg.size] += seg * amp

    peak = float(np.max(np.abs(mix))) if mix.size else 0.0
    if peak > 1.0:
        mix /= peak
    return mix


def save_wav(path: str, samples: np.ndarray, sr: int = SAMPLE_RATE) -> None:
    """Schreibt ein Mono-Float-Array [-1, 1] als 16-bit-PCM-WAV."""
    i16 = (np.clip(samples, -1.0, 1.0) * 32767.0).astype("<i2")
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(int(sr))
        w.writeframes(i16.tobytes())
