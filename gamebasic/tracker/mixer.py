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

from .song import (NOTE_OFF, vol_to_pct, FX_NONE, FX_ARP, FX_VIB, FX_RET,
                   FX_OFF, TICKS_PER_ROW)

SAMPLE_RATE = 44100

# Amiga/Paula-Hard-Panning-Konvention: Kanaele 1+4 links, 2+3 rechts
# (Indizes 0,3 = links; 1,2 = rechts). Nicht ganz hart (0.8) fuer einen
# ertraeglicheren Stereo-Eindruck als das harte +/-1 des echten Amiga. Bei
# mehr als 4 Kanaelen wiederholt sich das Muster (klassische Tracker-
# Konvention fuer >4-Kanal-Module, z.B. FastTracker).
_AMIGA_PAN = (-0.8, 0.8, 0.8, -0.8)


def _pan_gains(pan: float) -> tuple[float, float]:
    """Equal-Power-Panning -1..+1 -> (links, rechts). pan=0 -> beide ~0.707."""
    p = max(-1.0, min(1.0, float(pan)))
    ang = (p + 1.0) * 0.25 * np.pi          # 0..pi/2
    return float(np.cos(ang)), float(np.sin(ang))


def _channel_pan(c: int, inst, hard_pan: bool) -> float:
    """Endgueltiger Pan eines Kanals: Amiga-Basis (falls hard_pan) +
    Instrument-Pan, geklemmt auf -1..+1."""
    base = _AMIGA_PAN[c % len(_AMIGA_PAN)] if hard_pan else 0.0
    return max(-1.0, min(1.0, base + float(getattr(inst, "pan", 0.0))))


def _pitch_remap(buf: np.ndarray, ratio: np.ndarray) -> np.ndarray:
    """Liest `buf` mit einer pro-Sample variierenden Lesegeschwindigkeit
    `ratio` (kumuliert) -- Basis fuer Arpeggio/Vibrato als reines
    Post-Processing der fertig gerenderten Note (instrument-unabhaengig)."""
    n = buf.size
    if n == 0:
        return buf
    pos = np.concatenate(([0.0], np.cumsum(ratio)[:-1]))
    idx = np.arange(n, dtype=np.float64)
    out = np.interp(pos, idx, buf).astype(np.float32)
    out[pos > (n - 1)] = 0.0          # ueber das Ende hinaus = Stille
    return out


def apply_effect(buf: np.ndarray, fx: int, fxp: int, sr: int,
                 row_ms: int) -> np.ndarray:
    """Wendet einen Tracker-Effekt als Post-Processing auf die gerenderte
    Mono-Note `buf` an (instrument-unabhaengig). Reine numpy-Funktion.

    - FX_ARP: Tonhoehe springt im Tick-Takt zwischen Note / +x / +y Halbtoenen
      (x,y = fxp-Nibbles) -- der klassische Akkord-Arpeggio.
    - FX_VIB: Tonhoehe pendelt sinusfoermig (Speed = x Hz, Tiefe = y*0.125 HT).
    - FX_RET: schlaegt den Notenkopf alle `ticks` Ticks neu an.
    - FX_OFF: startet `fxp*512` Frames spaeter (Sample-Offset).
    """
    n = buf.size
    if n == 0 or fx == FX_NONE:
        return buf
    tick = max(1, int(sr * row_ms / 1000.0 / TICKS_PER_ROW))
    if fx == FX_ARP:
        x, y = (fxp >> 4) & 0xF, fxp & 0xF
        offsets = (0, x, y)
        semis = np.empty(n, dtype=np.float64)
        for i in range(0, n, tick):
            semis[i:i + tick] = offsets[(i // tick) % 3]
        return _pitch_remap(buf, 2.0 ** (semis / 12.0))
    if fx == FX_VIB:
        speed = float((fxp >> 4) & 0xF)               # Hz
        depth = float(fxp & 0xF) * 0.125              # Halbtoene
        if speed <= 0 or depth <= 0:
            return buf
        t = np.arange(n, dtype=np.float64) / sr
        semis = depth * np.sin(2.0 * np.pi * speed * t)
        return _pitch_remap(buf, 2.0 ** (semis / 12.0))
    if fx == FX_RET:
        ticks = max(1, fxp)
        seg = max(1, ticks * tick)
        head = buf[:seg]
        reps = int(np.ceil(n / seg))
        return np.tile(head, reps)[:n].astype(np.float32)
    if fx == FX_OFF:
        off = min(n, fxp * 512)
        out = np.zeros(n, dtype=np.float32)
        out[:n - off] = buf[off:]
        return out
    return buf


def _note_events(song):
    """Pro Kanal eine Liste von (global_row, midi, vol, slide, fx, fxp) fuer
    jede gesetzte Note -- die flache Timeline aus der Order."""
    events = {c: [] for c in range(song.channels)}
    i = 0
    for p in (song.order or [0]):
        if not (0 <= p < len(song.patterns)):
            continue
        pat = song.patterns[p]
        for r in range(pat.rows):
            for c in range(song.channels):
                note = pat.data[c][r]
                if note is not None:
                    fx, fxp = pat.get_fx(c, r)
                    events[c].append(
                        (i + r, int(note), pat.vol[c][r], pat.slide[c][r],
                         fx, fxp))
        i += pat.rows
    return events


def render_song(song, sr: int = SAMPLE_RATE, tail_ms: int = 800,
                stereo: bool = False, hard_pan: bool = False) -> np.ndarray:
    """Rendert den ganzen Song zu Float-Audio [-1, 1].

    Jede Note klingt bis zur naechsten Note desselben Kanals (Sustain ueber
    leere Reihen) und folgt ihrem Pitch-Slide (jetzt auch fuer Sample-/Keymap-
    Instrumente, nicht nur Synth). `tail_ms` = zusaetzliche Stille am Ende,
    damit ein langes Sample/Release ausklingen kann. Bei Uebersteuerung wird
    der gesamte Mix normalisiert.

    `stereo=True` liefert ein `(n, 2)`-Array (L/R), pro Kanal nach
    Instrument-Pan verteilt; `hard_pan=True` legt zusaetzlich die Amiga-
    Konvention (Kanal 1+4 links, 2+3 rechts) als Basis darunter. Mono
    (Default) bleibt unveraendert ein 1D-Array.
    """
    row_samples = max(1, int(sr * song.row_ms() / 1000.0))
    total_rows, _ = song.flatten()
    tail = int(sr * max(0, tail_ms) / 1000.0)
    total = max(1, total_rows * row_samples + tail)
    mix = np.zeros((total, 2) if stereo else total, dtype=np.float32)

    events = _note_events(song)
    for c in range(song.channels):
        inst = song.instrument_for_channel(c)
        gl, gr = _pan_gains(_channel_pan(c, inst, hard_pan)) if stereo else (1.0, 1.0)
        evs = events[c]
        for k, (start_row, midi, volc, slidec, fx, fxp) in enumerate(evs):
            if midi == NOTE_OFF:
                # Note-Off selbst erzeugt keinen Klang -- es diente schon als
                # `end_row` fuer das VORHERIGE Event (das dadurch bereits
                # abgeschnitten wurde). Einfach ueberspringen.
                continue
            end_row = evs[k + 1][0] if k + 1 < len(evs) else total_rows
            n = (end_row - start_row) * row_samples
            if n <= 0:
                continue
            # Sample/Loop darf bis zum Tail nachklingen.
            n_render = n + tail if k + 1 >= len(evs) else n
            note = inst.render_note(midi, n_render, sr, slide=(slidec or 0))
            if fx != FX_NONE:
                note = apply_effect(note, fx, fxp, sr, song.row_ms())
            amp = (vol_to_pct(volc) / 100.0) if volc else (inst.default_vol / 15.0)
            start = start_row * row_samples
            seg = note[:max(0, total - start)]
            if not seg.size:
                continue
            if stereo:
                mix[start:start + seg.size, 0] += seg * (amp * gl)
                mix[start:start + seg.size, 1] += seg * (amp * gr)
            else:
                mix[start:start + seg.size] += seg * amp

    peak = float(np.max(np.abs(mix))) if mix.size else 0.0
    if peak > 1.0:
        mix /= peak
    return mix


def save_wav(path: str, samples: np.ndarray, sr: int = SAMPLE_RATE) -> None:
    """Schreibt Float-Audio [-1, 1] als 16-bit-PCM-WAV. Mono (1D) ODER
    Stereo (`(n, 2)`) -- die Kanalzahl wird aus der Form abgeleitet."""
    arr = np.asarray(samples)
    channels = 2 if arr.ndim == 2 else 1
    i16 = (np.clip(arr, -1.0, 1.0) * 32767.0).astype("<i2")
    with wave.open(str(path), "wb") as w:
        w.setnchannels(channels)
        w.setsampwidth(2)
        w.setframerate(int(sr))
        w.writeframes(np.ascontiguousarray(i16).tobytes())
