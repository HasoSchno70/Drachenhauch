"""SoundFont-2-(.sf2)-Reader -- laedt echte Instrumente in Keymap-Instrumente.

Abhaengigkeitsfrei (pure Python + numpy). SF2 ist ein RIFF-Container mit der
"Hydra"-Struktur (phdr/pbag/pgen/inst/ibag/igen/shdr) + den rohen 16-bit-PCM-
Samples ('smpl'). Ein **Preset** (z. B. "Acoustic Grand Piano", Bank/Program)
verweist auf Instrumente, diese auf **Zonen** (Sample + Tastenbereich + Root +
Loop) -- das mappt 1:1 auf unser `Zone`/`Instrument`-Keymap-Modell.

Unterstuetzt der haeufige Fall: Preset -> Instrument(e) -> Zonen mit keyRange
(Gen 43), overridingRootKey (58), sampleID (53), sampleModes/Loop (54).
Velocity-Layer (44) werden ignoriert (zone_for nimmt die erste passende Zone),
Modulatoren werden nicht ausgewertet. Stereo-Samples werden als Mono genommen.
"""
from __future__ import annotations

import struct

import numpy as np

# Generator-Operatoren, die wir brauchen
GEN_KEYRANGE = 43
GEN_SAMPLEID = 53
GEN_SAMPLEMODES = 54
GEN_OVERRIDINGROOTKEY = 58
GEN_INSTRUMENT = 41


class _Rec:
    __slots__ = ("name", "bag", "preset", "bank")

    def __init__(self, name, bag, preset=0, bank=0):
        self.name = name
        self.bag = bag
        self.preset = preset
        self.bank = bank


class _Shdr:
    __slots__ = ("name", "start", "end", "startloop", "endloop",
                 "sample_rate", "pitch", "correction", "link", "stype")

    def __init__(self, t):
        (raw, self.start, self.end, self.startloop, self.endloop,
         self.sample_rate, self.pitch, self.correction,
         self.link, self.stype) = t
        self.name = _cstr(raw)


def _cstr(b: bytes) -> str:
    return b.split(b"\x00", 1)[0].decode("latin-1", "replace").strip()


class SoundFont:
    """Geparste SF2-Datei. `presets()` listet die Presets, `build_instrument`
    baut aus einem Preset ein Keymap-`Instrument`."""

    def __init__(self, path: str):
        self.path = str(path)
        self._smpl_off = 0
        self._smpl_size = 0
        self._parse()

    # ---------------------------------------------------- RIFF / Chunks
    def _parse(self) -> None:
        with open(self.path, "rb") as f:
            tag = f.read(4)
            if tag != b"RIFF":
                raise ValueError("Keine RIFF/SF2-Datei")
            f.read(4)                       # Gesamtgroesse
            if f.read(4) != b"sfbk":
                raise ValueError("Kein SoundFont (sfbk)")
            pdta = b""
            while True:
                hdr = f.read(8)
                if len(hdr) < 8:
                    break
                cid, size = struct.unpack("<4sI", hdr)
                if cid == b"LIST":
                    ltype = f.read(4)
                    end = f.tell() + size - 4
                    if ltype == b"sdta":
                        while f.tell() < end:
                            sid, ssize = struct.unpack("<4sI", f.read(8))
                            if sid == b"smpl":
                                self._smpl_off = f.tell()
                                self._smpl_size = ssize
                            f.seek(ssize + (ssize & 1), 1)
                    elif ltype == b"pdta":
                        pdta = f.read(size - 4)
                    else:
                        f.seek(size - 4, 1)
                    if size & 1:
                        f.seek(1, 1)
                else:
                    f.seek(size + (size & 1), 1)
        if not pdta:
            raise ValueError("pdta-Block fehlt")
        self._parse_pdta(pdta)

    def _sub(self, data: bytes):
        """Iteriert die Sub-Chunks eines pdta-Blocks -> {tag: bytes}."""
        out = {}
        i = 0
        while i + 8 <= len(data):
            cid, size = struct.unpack_from("<4sI", data, i)
            i += 8
            out[cid.decode("ascii", "replace")] = data[i:i + size]
            i += size + (size & 1)
        return out

    def _parse_pdta(self, data: bytes) -> None:
        s = self._sub(data)
        # phdr: name(20s), preset(H), bank(H), bagNdx(H), library/genre/morph
        self.phdr = []
        for r in _chunks(s["phdr"], 38):
            preset, bank, bag = struct.unpack_from("<HHH", r, 20)
            self.phdr.append(_Rec(_cstr(r[:20]), bag, preset, bank))
        self.pbag = [struct.unpack_from("<HH", r, 0)[0]
                     for r in _chunks(s["pbag"], 4)]
        self.pgen = [struct.unpack_from("<H2s", r, 0)
                     for r in _chunks(s["pgen"], 4)]
        self.inst = []
        for r in _chunks(s["inst"], 22):
            bag, = struct.unpack_from("<H", r, 20)
            self.inst.append(_Rec(_cstr(r[:20]), bag))
        self.ibag = [struct.unpack_from("<HH", r, 0)[0]
                     for r in _chunks(s["ibag"], 4)]
        self.igen = [struct.unpack_from("<H2s", r, 0)
                     for r in _chunks(s["igen"], 4)]
        self.shdr = [_Shdr(struct.unpack_from("<20sIIIIIBbHH", r, 0))
                     for r in _chunks(s["shdr"], 46)]

    # ---------------------------------------------------- Public
    def presets(self):
        """Liste (bank, program, name) -- ohne den Terminal-Eintrag (EOP),
        sortiert nach (bank, program)."""
        out = []
        for i in range(len(self.phdr) - 1):
            p = self.phdr[i]
            out.append((p.bank, p.preset, p.name))
        out.sort(key=lambda t: (t[0], t[1]))
        return out

    def _find_phdr(self, bank: int, program: int) -> int:
        for i in range(len(self.phdr) - 1):
            if self.phdr[i].bank == bank and self.phdr[i].preset == program:
                return i
        raise ValueError(f"Preset bank {bank} program {program} nicht gefunden")

    def read_samples(self, start: int, end: int) -> np.ndarray:
        n = max(0, end - start)
        if n == 0:
            return np.zeros(0, dtype=np.float32)
        with open(self.path, "rb") as f:
            f.seek(self._smpl_off + start * 2)
            raw = f.read(n * 2)
        return (np.frombuffer(raw, dtype="<i2").astype(np.float32) / 32768.0)

    def build_instrument(self, bank: int, program: int):
        """Baut ein Keymap-`Instrument` aus dem Preset (bank, program)."""
        from .instrument import Instrument, Zone
        pi = self._find_phdr(bank, program)
        zones = []
        for pz in range(self.phdr[pi].bag, self.phdr[pi + 1].bag):
            pgens = _gens(self.pgen, self.pbag[pz], self.pbag[pz + 1])
            if GEN_INSTRUMENT not in pgens:
                continue                    # globale Preset-Zone
            inst_idx = _u16(pgens[GEN_INSTRUMENT])
            p_lo, p_hi = _keyrange(pgens, 0, 127)
            if not (0 <= inst_idx < len(self.inst) - 1):
                continue
            for iz in range(self.inst[inst_idx].bag,
                            self.inst[inst_idx + 1].bag):
                igens = _gens(self.igen, self.ibag[iz], self.ibag[iz + 1])
                if GEN_SAMPLEID not in igens:
                    continue                # globale Instrument-Zone
                sid = _u16(igens[GEN_SAMPLEID])
                if not (0 <= sid < len(self.shdr) - 1):
                    continue
                sh = self.shdr[sid]
                lo, hi = _keyrange(igens, 0, 127)
                lo = max(lo, p_lo); hi = min(hi, p_hi)
                if lo > hi:
                    continue
                root = (_u16(igens[GEN_OVERRIDINGROOTKEY])
                        if GEN_OVERRIDINGROOTKEY in igens else sh.pitch)
                modes = (_u16(igens[GEN_SAMPLEMODES])
                         if GEN_SAMPLEMODES in igens else 0)
                samples = self.read_samples(sh.start, sh.end)
                zones.append(Zone(
                    samples=samples, sample_rate=sh.sample_rate or 44100,
                    root_note=int(root), lo_key=int(lo), hi_key=int(hi),
                    loop_mode="forward" if (modes & 1) else "none",
                    loop_start=max(0, sh.startloop - sh.start),
                    loop_end=max(0, sh.endloop - sh.start),
                    name=sh.name))
        name = self.phdr[pi].name or f"SF2 {bank}:{program}"
        return Instrument.keymap(name, zones)


def _chunks(data: bytes, size: int):
    for i in range(0, len(data) - size + 1, size):
        yield data[i:i + size]


def _gens(genlist, lo_ndx: int, hi_ndx: int) -> dict:
    """Generatoren einer Zone (Index-Bereich) -> {oper: amount_bytes}."""
    out = {}
    for k in range(lo_ndx, min(hi_ndx, len(genlist))):
        oper, amount = genlist[k]
        out[oper] = amount
    return out


def _u16(b: bytes) -> int:
    return struct.unpack("<H", b)[0]


def _keyrange(gens: dict, default_lo: int, default_hi: int):
    if GEN_KEYRANGE in gens:
        a = gens[GEN_KEYRANGE]
        return a[0], a[1]
    return default_lo, default_hi
