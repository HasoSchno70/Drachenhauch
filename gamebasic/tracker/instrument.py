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

LOOP_MODES = ("none", "forward", "pingpong")


@dataclass
class Zone:
    """Eine Tasten-Zone eines Keymap-Instruments: ein Sample fuer einen
    MIDI-Notenbereich [lo_key, hi_key], unverschoben bei `root_note`.

    Drumkit = je Zone eine einzelne Taste (lo==hi==root, kein Resampling);
    Multisample = ein Sample pro Tastenbereich, innerhalb resampelt."""
    samples: np.ndarray = field(default_factory=lambda: np.zeros(0, np.float32),
                                repr=False)
    sample_rate: int = SAMPLE_RATE
    root_note: int = DEFAULT_BASE_NOTE
    lo_key: int = 0
    hi_key: int = 127
    loop_mode: str = "none"
    loop_start: int = 0
    loop_end: int = 0
    name: str = ""

    def covers(self, midi: int) -> bool:
        return self.lo_key <= midi <= self.hi_key

    def distance(self, midi: int) -> int:
        if midi < self.lo_key:
            return self.lo_key - midi
        if midi > self.hi_key:
            return midi - self.hi_key
        return 0

    def to_dict(self) -> dict:
        return {"name": self.name, "sample_rate": int(self.sample_rate),
                "root_note": int(self.root_note),
                "lo_key": int(self.lo_key), "hi_key": int(self.hi_key),
                "loop_mode": self.loop_mode,
                "loop_start": int(self.loop_start),
                "loop_end": int(self.loop_end),
                "samples": _samples_to_b64(self.samples)}

    @classmethod
    def from_dict(cls, d: dict) -> "Zone":
        lm = str(d.get("loop_mode", "none"))
        return cls(
            samples=_b64_to_samples(d.get("samples", "")),
            sample_rate=int(d.get("sample_rate", SAMPLE_RATE)),
            root_note=int(d.get("root_note", DEFAULT_BASE_NOTE)),
            lo_key=int(d.get("lo_key", 0)), hi_key=int(d.get("hi_key", 127)),
            loop_mode=lm if lm in LOOP_MODES else "none",
            loop_start=int(d.get("loop_start", 0)),
            loop_end=int(d.get("loop_end", 0)),
            name=str(d.get("name", "")))


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
    # Loop (Sample-Indizes; loop_mode == "none" -> kein Loop):
    loop_mode: str = "none"         # "none" | "forward" | "pingpong"
    loop_start: int = 0
    loop_end: int = 0
    # ADSR-Lautstaerke-Huellkurve (ueber die Notendauer). Defaults =
    # passthrough (formt nichts), damit Synth-Klang unveraendert bleibt.
    env_attack_ms: int = 0
    env_decay_ms: int = 0
    env_sustain: float = 1.0        # 0..1 Pegel nach dem Decay
    env_release_ms: int = 0
    # Keymap (kind == "keymap"): Samples ueber Tasten-Zonen verteilt
    # (Multisample / Drumkit). ADSR oben gilt fuer alle Zonen.
    zones: list = field(default_factory=list, repr=False)

    # ---- Konstruktoren
    @classmethod
    def synth(cls, name: str, waveform: str) -> "Instrument":
        return cls(name=name, kind="synth", waveform=waveform)

    @classmethod
    def keymap(cls, name: str, zones: list | None = None) -> "Instrument":
        return cls(name=name, kind="keymap", zones=list(zones or []))

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

    def is_keymap(self) -> bool:
        return self.kind == "keymap" and len(self.zones) > 0

    def has_loop(self) -> bool:
        return (self.loop_mode in ("forward", "pingpong")
                and self.loop_end > self.loop_start >= 0)

    def zone_for(self, midi: int):
        """Die Zone, die `midi` abdeckt -- sonst die naechstgelegene (damit
        Noten ausserhalb aller Bereiche trotzdem klingen). None ohne Zonen."""
        if not self.zones:
            return None
        for z in self.zones:
            if z.covers(midi):
                return z
        return min(self.zones, key=lambda z: z.distance(midi))

    def render_note(self, midi: int, n_samples: int,
                    sr: int = SAMPLE_RATE) -> np.ndarray:
        """Liefert ein Mono-Float-Array [-1, 1] der Note `midi`, `n_samples`
        lang. Synth: Wellenform; Sample: resampelt das Quell-Audio (mit Loop);
        Keymap: waehlt die Tasten-Zone und resampelt deren Sample. ADSR +
        Anti-Click werden in allen Faellen angewandt."""
        n_samples = max(0, int(n_samples))
        if self.is_keymap():
            z = self.zone_for(midi)
            if z is None or z.samples.size == 0:
                out = np.zeros(n_samples, dtype=np.float32)
            else:
                lm = (z.loop_mode if z.loop_mode in ("forward", "pingpong")
                      and z.loop_end > z.loop_start >= 0 else "none")
                out = _resample(z.samples, z.sample_rate, z.root_note,
                                midi, n_samples, sr,
                                lm, int(z.loop_start), int(z.loop_end))
        elif not self.is_sample():
            out = _render_synth(self.waveform, midi, n_samples, sr)
        else:
            lm = self.loop_mode if self.has_loop() else "none"
            out = _resample(self.samples, self.sample_rate, self.base_note,
                            midi, n_samples, sr,
                            lm, int(self.loop_start), int(self.loop_end))
        out = self._apply_envelope(out, sr)
        return _anti_click(out, sr)

    def _apply_envelope(self, out: np.ndarray, sr: int) -> np.ndarray:
        if out.size == 0:
            return out
        if (self.env_attack_ms <= 0 and self.env_decay_ms <= 0
                and self.env_release_ms <= 0 and self.env_sustain >= 1.0):
            return out                       # passthrough
        env = _adsr_env(out.size, self.env_attack_ms, self.env_decay_ms,
                        float(self.env_sustain), self.env_release_ms, sr)
        return (out * env).astype(np.float32)

    # ---- Serialisierung
    def _env_dict(self) -> dict:
        return {"loop_mode": self.loop_mode,
                "loop_start": int(self.loop_start),
                "loop_end": int(self.loop_end),
                "env_attack_ms": int(self.env_attack_ms),
                "env_decay_ms": int(self.env_decay_ms),
                "env_sustain": float(self.env_sustain),
                "env_release_ms": int(self.env_release_ms)}

    def _apply_env_from_dict(self, d: dict) -> None:
        lm = str(d.get("loop_mode", "none"))
        self.loop_mode = lm if lm in LOOP_MODES else "none"
        self.loop_start = int(d.get("loop_start", 0))
        self.loop_end = int(d.get("loop_end", 0))
        self.env_attack_ms = int(d.get("env_attack_ms", 0))
        self.env_decay_ms = int(d.get("env_decay_ms", 0))
        self.env_sustain = float(d.get("env_sustain", 1.0))
        self.env_release_ms = int(d.get("env_release_ms", 0))

    def to_dict(self) -> dict:
        d = {"name": self.name, "kind": self.kind,
             "default_vol": int(self.default_vol)}
        if self.kind == "keymap":
            d["zones"] = [z.to_dict() for z in self.zones]
            d.update(self._env_dict())       # ADSR gilt fuer alle Zonen
            # Loop steckt pro Zone -> nur ADSR-Teil ist hier relevant
        elif self.kind == "sample" and self.samples is not None:
            d["sample_rate"] = int(self.sample_rate)
            d["base_note"] = int(self.base_note)
            d["samples"] = _samples_to_b64(self.samples)
            d.update(self._env_dict())
        else:
            d["waveform"] = self.waveform
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Instrument":
        kind = str(d.get("kind", "synth"))
        name = str(d.get("name", "Instrument"))
        dv = int(d.get("default_vol", 15))
        if kind == "keymap":
            inst = cls.keymap(name, [Zone.from_dict(z)
                                     for z in (d.get("zones") or [])])
            inst.default_vol = dv
            inst._apply_env_from_dict(d)
            return inst
        if kind == "sample" and d.get("samples"):
            inst = cls.from_array(
                name, _b64_to_samples(d["samples"]),
                int(d.get("sample_rate", SAMPLE_RATE)),
                int(d.get("base_note", DEFAULT_BASE_NOTE)))
            inst.default_vol = dv
            inst._apply_env_from_dict(d)
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
              midi: int, n_samples: int, sr: int,
              loop_mode: str = "none",
              loop_start: int = 0, loop_end: int = 0) -> np.ndarray:
    """Lineares Resampling eines Samples auf die Zieltonhoehe, mit optionalem
    Loop (forward / pingpong).

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
    idx_axis = np.arange(src.size, dtype=np.float64)
    loop_ok = (loop_mode in ("forward", "pingpong")
               and 0 <= loop_start < loop_end <= src.size)
    if not loop_ok:
        # Einmal abspielen, Rest mit Stille (kein Loop).
        avail = int(np.floor((src.size - 1) / step)) + 1
        out_len = min(n_samples, max(0, avail))
        pos = np.arange(out_len, dtype=np.float64) * step
        out = np.interp(pos, idx_axis, src).astype(np.float32)
        if out.size < n_samples:
            out = np.concatenate(
                [out, np.zeros(n_samples - out.size, np.float32)])
        return out
    # Mit Loop: virtueller, monoton steigender Playhead `pos`, in eine
    # Quell-Position innerhalb des Loops zurueckgefaltet.
    pos = np.arange(n_samples, dtype=np.float64) * step
    le = float(loop_end)
    ls = float(loop_start)
    loop_len = le - ls
    src_pos = pos.copy()
    after = pos >= le
    rel = pos[after] - le
    if loop_mode == "forward":
        src_pos[after] = ls + np.mod(rel, loop_len)
    else:  # pingpong: Dreieck zwischen ls und le
        tri = np.mod(rel, 2.0 * loop_len)
        up = tri <= loop_len
        folded = np.empty_like(tri)
        folded[up] = le - tri[up]
        folded[~up] = ls + (tri[~up] - loop_len)
        src_pos[after] = folded
    return np.interp(src_pos, idx_axis, src).astype(np.float32)


def _adsr_env(n: int, attack_ms: int, decay_ms: int, sustain: float,
              release_ms: int, sr: int) -> np.ndarray:
    """ADSR-Huellkurve der Laenge n (Attack->Decay->Sustain->Release am Ende)."""
    sustain = max(0.0, min(1.0, sustain))
    env = np.full(n, sustain, dtype=np.float64)   # Sustain-Pegel als Basis
    na = min(n, int(sr * max(0, attack_ms) / 1000))
    nd = min(n - na, int(sr * max(0, decay_ms) / 1000))
    nr = min(n, int(sr * max(0, release_ms) / 1000))
    if na > 0:
        env[:na] = np.linspace(0.0, 1.0, na)
    if nd > 0:
        env[na:na + nd] = np.linspace(1.0, sustain, nd)
    if nr > 0:                                # Release ueberschreibt das Ende
        start_lvl = float(env[-nr])
        env[-nr:] = np.linspace(start_lvl, 0.0, nr)
    return env


def _anti_click(out: np.ndarray, sr: int, fade_ms: float = 2.0) -> np.ndarray:
    """Kurze lineare Ausblendung am Ende gegen Klicks bei hartem Abschnitt."""
    if out.size == 0:
        return out
    nf = min(out.size, int(sr * fade_ms / 1000))
    if nf > 1:
        out = out.copy()
        out[-nf:] *= np.linspace(1.0, 0.0, nf).astype(np.float32)
    return out
