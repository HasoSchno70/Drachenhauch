"""Tests fuer den SoundFont-(.sf2)-Reader.

Baut eine minimale, gueltige SF2-Datei (1 Preset -> 1 Instrument -> 1 Zone ->
1 Sample) und prueft Parsing + Keymap-Instrument-Bau end-to-end.
"""
import struct

import numpy as np

from gamebasic.tracker.sf2 import SoundFont


def _chunk(tag: bytes, data: bytes) -> bytes:
    out = tag + struct.pack("<I", len(data)) + data
    if len(data) & 1:
        out += b"\x00"
    return out


def _list(ltype: bytes, *subs: bytes) -> bytes:
    body = ltype + b"".join(subs)
    return b"LIST" + struct.pack("<I", len(body)) + body


def _name(s: str) -> bytes:
    return s.encode("latin-1")[:20].ljust(20, b"\x00")


def _build_min_sf2(tmp_path):
    # 200 int16-Samples (Rampe)
    samples = (np.linspace(-20000, 20000, 200)).astype("<i2").tobytes()
    sdta = _list(b"sdta", _chunk(b"smpl", samples))

    # phdr: 1 Preset "TestPiano" (bank 0, prog 0, bag 0) + Terminal "EOP"
    phdr = (_name("TestPiano") + struct.pack("<HHHIII", 0, 0, 0, 0, 0, 0)
            + _name("EOP") + struct.pack("<HHHIII", 0, 0, 1, 0, 0, 0))
    # pbag: Zone 0 -> pgen[0..1], Terminal -> 1
    pbag = struct.pack("<HH", 0, 0) + struct.pack("<HH", 1, 0)
    # pgen: instrument-Gen (41) -> inst 0, dann Terminal-Gen
    pgen = struct.pack("<H", 41) + struct.pack("<H", 0) \
        + struct.pack("<H", 0) + struct.pack("<H", 0)
    # inst: 1 Instrument "TestInst" (bag 0) + Terminal "EOI"
    inst = _name("TestInst") + struct.pack("<H", 0) \
        + _name("EOI") + struct.pack("<H", 1)
    # ibag: Zone 0 -> igen[0..2], Terminal -> 2
    ibag = struct.pack("<HH", 0, 0) + struct.pack("<HH", 2, 0)
    # igen: keyRange(43)=0..127, sampleID(53)=0, Terminal
    igen = (struct.pack("<H", 43) + bytes([0, 127])
            + struct.pack("<H", 53) + struct.pack("<H", 0)
            + struct.pack("<H", 0) + struct.pack("<H", 0))
    # shdr: 1 Sample + Terminal "EOS"
    shdr = (_name("TestSample")
            + struct.pack("<IIIII", 0, 200, 50, 150, 22050)
            + struct.pack("<BbHH", 60, 0, 0, 1)
            + _name("EOS") + struct.pack("<IIIII", 0, 0, 0, 0, 0)
            + struct.pack("<BbHH", 0, 0, 0, 0))

    pdta = _list(b"pdta",
                 _chunk(b"phdr", phdr), _chunk(b"pbag", pbag),
                 _chunk(b"pgen", pgen), _chunk(b"inst", inst),
                 _chunk(b"ibag", ibag), _chunk(b"igen", igen),
                 _chunk(b"shdr", shdr))

    body = b"sfbk" + sdta + pdta
    riff = b"RIFF" + struct.pack("<I", len(body)) + body
    p = tmp_path / "test.sf2"
    p.write_bytes(riff)
    return str(p)


def test_sf2_lists_presets(tmp_path):
    sf = SoundFont(_build_min_sf2(tmp_path))
    presets = sf.presets()
    assert presets == [(0, 0, "TestPiano")]


def test_sf2_builds_keymap_instrument(tmp_path):
    sf = SoundFont(_build_min_sf2(tmp_path))
    inst = sf.build_instrument(0, 0)
    assert inst.is_keymap()
    assert inst.name == "TestPiano"
    assert len(inst.zones) == 1
    z = inst.zones[0]
    assert z.lo_key == 0 and z.hi_key == 127
    assert z.root_note == 60
    assert z.sample_rate == 22050
    assert z.samples.size == 200
    assert z.loop_mode == "none"


def test_sf2_instrument_renders(tmp_path):
    sf = SoundFont(_build_min_sf2(tmp_path))
    inst = sf.build_instrument(0, 0)
    out = inst.render_note(60, 4410)         # darf nicht crashen + Inhalt
    assert out.shape == (4410,)
    assert np.max(np.abs(out)) > 0.0


def test_sf2_rejects_non_sf2(tmp_path):
    import pytest
    p = tmp_path / "x.sf2"
    p.write_bytes(b"NOTRIFF.....")
    with pytest.raises(ValueError):
        SoundFont(str(p))
