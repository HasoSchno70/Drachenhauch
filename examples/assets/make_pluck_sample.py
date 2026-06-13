#!/usr/bin/env python3
"""Erzeugt ein kurzes, frei verwendbares Pluck-/Saiten-Sample (WAV) fuer die
SAMPLE_PLAY-Demo examples/116_sampler.gb.

Karplus-Strong-aehnlicher Zupf-Klang bei ~220 Hz (A3), das in der Demo per
SAMPLE_PLAY ueber die ganze Klaviatur resampled wird (Amiga/Paula-Prinzip:
hoehere Note = schneller abgespielt). Selbst generiert -> gemeinfrei.

Aufruf:  python examples/assets/make_pluck_sample.py
Ergebnis: examples/assets/pluck.wav
"""
import math
import os
import struct
import wave

SR = 22050
BASE_HZ = 220.0          # A3 -- der "Grundton" des Samples (SAMPLE base)
DUR = 0.5

n = int(SR * DUR)
period = int(SR / BASE_HZ)

# Karplus-Strong: Rausch-Burst in einen Ringpuffer, dann gleitender Mittelwert.
import random
random.seed(42)
ring = [random.uniform(-1.0, 1.0) for _ in range(period)]
out = []
idx = 0
for i in range(n):
    cur = ring[idx]
    nxt = ring[(idx + 1) % period]
    val = 0.4 * cur + 0.6 * nxt        # einfacher Tiefpass -> dampft hohe Frequenzen
    # leichte Daempfung ueber die Zeit (Ausklang)
    val *= 0.999
    ring[idx] = val
    out.append(cur)
    idx = (idx + 1) % period

# Global ausklingen lassen (Hüllkurve)
for i in range(n):
    env = min(1.0, i / (SR * 0.005))               # 5ms Attack
    env *= math.exp(-3.0 * i / n)                  # exponentieller Decay
    out[i] *= env

peak = max(1e-6, max(abs(v) for v in out))
out = [v / peak * 0.85 for v in out]

w = wave.open(os.path.join(os.path.dirname(__file__), "pluck.wav"), "w")
w.setnchannels(1)
w.setsampwidth(2)
w.setframerate(SR)
w.writeframes(b"".join(struct.pack("<h", int(max(-1.0, min(1.0, v)) * 32767))
                       for v in out))
w.close()
print(f"pluck.wav: {n} Frames @ {SR} Hz, Grundton {BASE_HZ} Hz")
