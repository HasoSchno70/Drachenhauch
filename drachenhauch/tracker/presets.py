"""Fertige Instrument-Presets fuer den Tracker (Qt-frei).

Damit man NICHT erst WAV-Samples suchen muss, gibt es eine Bibliothek
spielbarer Klaenge -- synthetisiert aus den Wellenformen plus ADSR-Huellkurve,
Vibrato und einem leichten Detune-Layer (Chorus). Das ergibt deutlich
unterschiedlichere, "instrumentenhafte" Klaenge als die rohen Wellenformen
(Fluegel, E-Piano, Orgel, Streicher, Bass, Glocke ...). Echte Sample-/Keymap-
Instrumente bleiben zusaetzlich moeglich.

`factory_instruments()` liefert frische `Instrument`-Objekte (Kopien), damit
mehrere Songs sie unabhaengig veraendern koennen.
"""
from __future__ import annotations

from .instrument import Instrument

# (Anzeige-Name, waveform, attack, decay, sustain, release, vib_depth,
#  vib_speed, detune_cents)
_PRESETS = [
    ("Fluegel (Piano)",  "triangle",  2, 700, 0.0, 200, 0.0,  0,  6),
    ("E-Piano",          "sine",      2, 450, 0.30, 220, 0.0,  0,  8),
    ("Orgel",            "square",    6,   0, 1.0,  70, 0.06, 6,  5),
    ("Kirchenorgel",     "saw",      14,   0, 1.0, 140, 0.04, 5, 10),
    ("Streicher",        "saw",     140,   0, 1.0, 280, 0.05, 5, 14),
    ("Synth-Pad",        "triangle",220,   0, 0.85,420, 0.03, 4, 18),
    ("Bass",             "square",    2, 140, 0.55, 80,  0.0,  0,  0),
    ("Synth-Bass",       "saw",       2, 120, 0.50, 70,  0.0,  0,  6),
    ("Lead",             "saw",       2,   0, 0.9,  90,  0.05, 6,  4),
    ("Pluck",            "triangle",  1, 220, 0.0, 130,  0.0,  0,  4),
    ("Glocke",           "sine",      1, 900, 0.0, 320,  0.0,  0, 10),
    ("Marimba",          "sine",      1, 260, 0.0, 130,  0.0,  0,  0),
    ("Chip-Square",      "square",    1,   0, 1.0,  20,  0.0,  0,  0),
    ("Chip-Saw",         "saw",       1,   0, 1.0,  20,  0.0,  0,  0),
]

# Schlagzeug-Sounds aus den Wellenformen (perkussiv, kurze Decays).
_DRUMS = [
    ("Kick",   "sine",   0,  90, 0.0, 30, 0.0, 0, 0),
    ("Snare",  "noise",  0, 140, 0.0, 40, 0.0, 0, 0),
    ("HiHat",  "noise",  0,  40, 0.0, 20, 0.0, 0, 0),
    ("Tom",    "sine",   0, 160, 0.0, 40, 0.0, 0, 0),
]


def _make(spec) -> Instrument:
    name, wf, atk, dec, sus, rel, vd, vs, det = spec
    inst = Instrument.synth(name, wf)
    inst.env_attack_ms = atk
    inst.env_decay_ms = dec
    inst.env_sustain = sus
    inst.env_release_ms = rel
    inst.vib_depth = vd
    inst.vib_speed = vs
    inst.detune_cents = det
    return inst


def factory_instruments() -> list:
    """Frische Liste aller Instrument-Presets (Melodie + Drums)."""
    return [_make(s) for s in _PRESETS] + [_make(s) for s in _DRUMS]


def preset_names() -> list:
    return [s[0] for s in _PRESETS] + [s[0] for s in _DRUMS]
