#!/usr/bin/env python3
"""Erzeugt ein kleines, frei verwendbares ProTracker-MOD (M.K., 4 Kanaele)
fuer die Modul-Player-Demo examples/115_modplayer.gb.

Selbst generiert -> gemeinfrei. Vier Kanaele im Amiga-Stil:
  Kanal 1  BASS    (Square, tief)
  Kanal 2  CHORD   (Square-Akkordton)
  Kanal 3  LEAD    (Square-Melodie)
  Kanal 4  PERC    (kurzes Noise-Sample)

Aufruf:  python examples/assets/make_demo_mod.py
Ergebnis: examples/assets/demo.mod
"""
import os
import struct

# ----- Samples (signed 8-bit) -------------------------------------------
def square(length, hi=70, duty=0.5):
    period = max(2, length)
    return bytes(((hi if (i % period) < period * duty else -hi) & 0xFF)
                 for i in range(length))

def noise(length):
    # deterministisches "Rauschen" (LCG) fuer Percussion
    s = 0x1234
    out = bytearray()
    for _ in range(length):
        s = (s * 1103515245 + 12345) & 0x7FFFFFFF
        out.append(((s >> 16) % 160 - 80) & 0xFF)
    return bytes(out)

# Instrument-Liste: (name, sampledata, loop?)
samples = [
    (b"bass",  square(64, 78, 0.5), True),
    (b"chord", square(48, 55, 0.25), True),    # duenner (Pulsbreite 25%)
    (b"lead",  square(32, 64, 0.5), True),
    (b"perc",  noise(600), False),
]

# ----- Perioden (ProTracker, finetune 0) --------------------------------
PER = {
    "C2": 428, "D2": 381, "E2": 339, "F2": 320, "G2": 285, "A2": 254, "B2": 226,
    "C3": 214, "D3": 190, "E3": 170, "F3": 160, "G3": 143, "A3": 127, "B3": 113,
    "C1": 856, "E1": 678, "G1": 570, "A1": 508, "F1": 720, "D1": 762,
}

# ----- Song: 2 Patterns, in der Order 0,1,0,1 ---------------------------
# je 64 Reihen. Akkordfolge Am - F - C - G (8 Reihen je Akkord).
# (note_or_None, sample_no) pro Zelle.
def build_pattern(roots_bass, chord_notes, lead_notes):
    rows = [[None]*4 for _ in range(64)]
    for blk in range(8):                      # 8 Bloecke je 8 Reihen
        base = blk * 8
        # Bass: Grundton auf Reihe 0 und 4 des Blocks
        rows[base + 0][0] = (roots_bass[blk], 1)
        rows[base + 4][0] = (roots_bass[blk], 1)
        # Chord: Akkordton auf 0 und 2 und 4 und 6
        for r in (0, 2, 4, 6):
            rows[base + r][1] = (chord_notes[blk], 2)
        # Lead: kleine Melodie, jede 2. Reihe
        for k, r in enumerate((0, 2, 4, 6)):
            rows[base + r][2] = (lead_notes[(blk + k) % len(lead_notes)], 3)
        # Perc: jede Reihe 0/4 ein Hit, plus offbeats
        rows[base + 0][3] = ("C3", 4)
        rows[base + 4][3] = ("C3", 4)
        rows[base + 2][3] = ("C3", 4)
        rows[base + 6][3] = ("C3", 4)
    return rows

pat0 = build_pattern(
    roots_bass=["A1","A1","F1","F1","C2","C2","G1","G1"],
    chord_notes=["A2","A2","F2","F2","C3","C3","G2","G2"],
    lead_notes=["A3","C3","E3","C3","A3","G3"],
)
pat1 = build_pattern(
    roots_bass=["A1","A1","F1","F1","C2","C2","E1","E1"],
    chord_notes=["A2","C3","F2","A2","C3","E3","G2","B2"],
    lead_notes=["E3","D3","C3","B2","A2","C3"],
)
patterns = [pat0, pat1]
order = [0, 1, 0, 1]

# ----- MOD zusammenbauen -------------------------------------------------
def cell_bytes(cell):
    if cell is None:
        return bytes(4)
    note, sample_no = cell
    period = PER[note]
    b0 = (sample_no & 0xF0) | ((period >> 8) & 0x0F)
    b1 = period & 0xFF
    b2 = ((sample_no & 0x0F) << 4) | 0x0
    b3 = 0
    return bytes([b0, b1, b2, b3])

out = bytearray()
out += b"GB DEMO MODULE".ljust(20, b"\0")

for i in range(31):
    if i < len(samples):
        name, data, loop = samples[i]
        words = len(data) // 2
        rep_point = 0
        rep_len = words if loop else 1
        out += name.ljust(22, b"\0")
        out += struct.pack(">H", words)
        out += bytes([0, 64])                  # finetune 0, volume 64
        out += struct.pack(">H", rep_point)
        out += struct.pack(">H", rep_len)
    else:
        out += b"".ljust(22, b"\0")
        out += struct.pack(">H", 0)
        out += bytes([0, 0])
        out += struct.pack(">H", 0)
        out += struct.pack(">H", 1)

out += bytes([len(order), 127])
order_tab = bytearray(128)
for i, p in enumerate(order):
    order_tab[i] = p
out += order_tab
out += b"M.K."

for pat in patterns:
    for row in pat:
        for ch in range(4):
            out += cell_bytes(row[ch])

for _, data, _ in samples:
    if len(data) % 2:                          # gerade Laenge (words)
        data += b"\0"
    out += data

dst = os.path.join(os.path.dirname(__file__), "demo.mod")
with open(dst, "wb") as f:
    f.write(out)
print(f"demo.mod geschrieben: {len(out)} Bytes, {len(patterns)} Patterns, "
      f"Order {order}")
