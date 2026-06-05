"""Instrumente fuer den Tracker: Synth-Wellenform ODER gesampeltes Audio.

Qt-frei + nur numpy/stdlib -> headless testbar. Ein **Instrument** ist
entweder ein klassischer Synth-Klang (eine der Wellenformen) oder ein
**Sample** (geladene WAV-Aufnahme). Ein Sample wird ueber die Klaviatur
gespielt, indem es per Resampling in der Geschwindigkeit/Tonhoehe verschoben
wird (`render_note`) -- genau das Kernprinzip professioneller Tracker
(MOD/XM/IT): EIN Sample, in Echtzeit auf jede Note umgerechnet.

Spaetere Stufen ergaenzen hier Loop-Punkte, Envelopes und Feinstimmung.
"""
from __future__ import annotations

import base64
import io
import wave
from dataclasses import dataclass, field

import numpy as np

SAMPLE_RATE = 44100
DEFAULT_BASE_NOTE = 60          # MIDI C4 -- Tonhoehe, bei der das Sample 1:1 spielt


def midi_to_freq(m: float) -> float:
    return 440.0 * (2.0 ** ((m - 69) / 12.0))


# --------------------------------------------------------------- WAV-I/O

def load_wav_mono(path: str) -> tuple[np.ndarray, int]:
    """Laedt eine PCM-WAV-Datei als Mono-Float-Array [-1, 1] + Sample-Rate.

    Unterstuetzt 8/16/32-bit PCM, Mono oder Stereo (Stereo wird gemittelt).
    Wirft ValueError bei nicht unterstuetzten Formaten (z. B. float-WAV)."""
    with wave.open(str(path), "rb") as w:
        nch = w.getnchannels()
        sw = w.getsampwidth()
        sr = w.getframerate()
        raw = w.readframes(w.getnframes())
    return _decode_pcm(raw, nch, sw), sr


def _decode_pcm(raw: bytes, nch: int, sampwidth: int) -> np.ndarray:
    if sampwidth == 1:                       # 8-bit unsigned (0..255)
        a = np.frombuffer(raw, dtype=np.uint8).astype(np.float32)
        a = (a - 128.0) / 128.0
    elif sampwidth == 2:                     # 16-bit signed
        a = np.frombuffer(raw, dtype="<i2").astype(np.float32) / 32768.0
    elif sampwidth == 4:                     # 32-bit signed
        a = np.frombuffer(raw, dtype="<i4").astype(np.float32) / 2147483648.0
    else:
        raise ValueError(f"WAV-Sampleweite {sampwidth*8}-bit nicht unterstuetzt")
    if nch > 1:
        a = a.reshape(-1, nch).mean(axis=1)
    return np.clip(a, -1.0, 1.0).astype(np.float32)


def _samples_to_b64(samples: np.ndarray) -> str:
    """Mono-Float [-1,1] -> base64(int16-PCM) fuer kompakte JSON-Ablage."""
    i16 = (np.clip(samples, -1.0, 1.0) * 32767.0).astype("<i2")
    return base64.b64encode(i16.tobytes()).decode("ascii")


def _b64_to_samples(s: str) -> np.ndarray:
    raw = base64.b64decode(s)
    return (np.frombuffer(raw, dtype="<i2").astype(np.float32) / 32768.0)


# --------------------------------------------------------------- Instrument

@dataclass
class Instrument:
    name: str = "Instrument"
    kind: str = "synth"              # "synth" | "sample"
    waveform: str = "square"        # bei kind == "synth"
    # Sample-Felder (kind == "sample"):
    samples: np.ndarray | None = field(default=None, repr=False)
    sample_rate: int = SAMPLE_RATE
    base_note: int = DEFAULT_BASE_NOTE
    default_vol: int = 15           # 1..15 (Tracker-Lautstaerke-Skala)

    # ---- Konstruktoren
    @classmethod
    def synth(cls, name: str, waveform: str) -> "Instrument":
        return cls(name=name, kind="synth", waveform=waveform)

    @classmethod
    def from_array(cls, name: str, samples: np.ndarray, sample_rate: int,
                   base_note: int = DEFAULT_BASE_NOTE) -> "Instrument":
        arr = np.asarray(samples, dtype=np.float32).reshape(-1)
        return cls(name=name, kind="sample", samples=arr,
                   sample_rate=int(sample_rate), base_note=int(base_note))

    @classmethod
    def from_wav(cls, path: str, name: str | None = None,
                 base_note: int = DEFAULT_BASE_NOTE) -> "Instrument":
        samples, sr = load_wav_mono(path)
        from pathlib import Path
        nm = name or Path(path).stem or "Sample"
        return cls.from_array(nm, samples, sr, base_note)

    # ---- Rendern
    def is_sample(self) -> bool:
        return self.kind == "sample" and self.samples is not None \
            and self.samples.size > 0

    def render_note(self, midi: int, n_samples: int,
                    sr: int = SAMPLE_RATE) -> np.ndarray:
        """Liefert ein Mono-Float-Array [-1, 1] der Note `midi`, hoechstens
        `n_samples` lang. Synth: erzeugt die Wellenform; Sample: resampelt das
        Quell-Audio in die Zieltonhoehe (laeuft aus, wenn das Sample kuerzer
        ist -- Loops kommen in einer spaeteren Stufe)."""
        n_samples = max(0, int(n_samples))
        if not self.is_sample():
            return _render_synth(self.waveform, midi, n_samples, sr)
        return _resample(self.samples, self.sample_rate, self.base_note,
                         midi, n_samples, sr)

    # ---- Serialisierung
    def to_dict(self) -> dict:
        d = {"name": self.name, "kind": self.kind,
             "default_vol": int(self.default_vol)}
        if self.kind == "sample" and self.samples is not None:
            d["sample_rate"] = int(self.sample_rate)
            d["base_note"] = int(self.base_note)
            d["samples"] = _samples_to_b64(self.samples)
        else:
            d["waveform"] = self.waveform
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Instrument":
        kind = str(d.get("kind", "synth"))
        name = str(d.get("name", "Instrument"))
        dv = int(d.get("default_vol", 15))
        if kind == "sample" and d.get("samples"):
            inst = cls.from_array(
                name, _b64_to_samples(d["samples"]),
                int(d.get("sample_rate", SAMPLE_RATE)),
                int(d.get("base_note", DEFAULT_BASE_NOTE)))
            inst.default_vol = dv
            return inst
        inst = cls.synth(name, str(d.get("waveform", "square")))
        inst.default_vol = dv
        return inst


def _render_synth(waveform: str, midi: int, n_samples: int, sr: int) -> np.ndarray:
    """Synth-Note ueber den geteilten Synth (kleine ADSR gegen Klicks)."""
    if n_samples <= 0:
        return np.zeros(0, dtype=np.float32)
    from ..synth import synthesize
    total_ms = max(1, int(round(n_samples / sr * 1000.0)))
    atk = min(4, total_ms)
    dec = min(8, max(0, total_ms - atk))
    sus = max(0, total_ms - atk - dec)
    wave_arr = synthesize(waveform, midi_to_freq(midi), 0.0,
                          atk, sus, dec, sr=sr)
    out = np.asarray(wave_arr, dtype=np.float32).reshape(-1)
    if out.size >= n_samples:
        return out[:n_samples]
    return np.concatenate([out, np.zeros(n_samples - out.size, np.float32)])


def _resample(samples: np.ndarray, src_sr: int, base_note: int,
              midi: int, n_samples: int, sr: int) -> np.ndarray:
    """Lineares Resampling eines Samples auf die Zieltonhoehe.

    Schrittweite pro Ausgabe-Sample durchs Quell-Audio:
        step = (src_sr / sr) * 2^((midi - base_note) / 12)
    -> hoehere Note = groesserer Schritt = schnelleres/hoeheres Abspielen.
    """
    src = np.asarray(samples, dtype=np.float32).reshape(-1)
    if src.size == 0 or n_samples <= 0:
        return np.zeros(max(0, n_samples), dtype=np.float32)
    step = (src_sr / float(sr)) * (2.0 ** ((midi - base_note) / 12.0))
    if step <= 0:
        return np.zeros(n_samples, dtype=np.float32)
    # Nur so viele Ausgabe-Samples, wie das Quell-Audio hergibt.
    avail = int(np.floor((src.size - 1) / step)) + 1
    out_len = min(n_samples, max(0, avail))
    if out_len <= 0:
        return np.zeros(n_samples, dtype=np.float32)
    pos = np.arange(out_len, dtype=np.float64) * step
    idx = np.arange(src.size, dtype=np.float64)
    out = np.interp(pos, idx, src).astype(np.float32)
    if out.size < n_samples:                 # Rest mit Stille (kein Loop)
        out = np.concatenate([out, np.zeros(n_samples - out.size, np.float32)])
    return out
