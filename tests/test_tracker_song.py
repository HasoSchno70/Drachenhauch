"""Tests fuer das Tracker-Song-Modell (Qt-frei).

Deckt Pattern/Order-Ops, JSON-Roundtrip und den GB-Code-Export ab
(Flatten + Kompilierbarkeit des erzeugten Programms)."""
import json
import os
import subprocess
from pathlib import Path

import pytest

from gamebasic.tracker import (
    CHANNELS, TONAL, Pattern, Song, midi_to_freq, note_name,
)

_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_GBRT = _find_gbrt()


def _check_compiles(tmp_path, src):
    """Der exportierte GB-Code muss in gbrt kompilieren (`gbrt --check`,
    leere Fehlerliste). Frueher via Python-Compiler (in Phase 8 geloescht)."""
    if _GBRT is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")
    fd = tmp_path / "_track.gb"
    fd.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_GBRT), "--check", str(fd)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    diags = json.loads(r.stdout or "[]")
    errs = [d for d in diags if d.get("severity") == "error"]
    assert errs == [], f"GB-Code kompiliert nicht: {errs}"


# --------------------------------------------------------------- Pattern

def test_new_pattern_empty():
    p = Pattern("P1", 16)
    assert p.rows == 16
    assert len(p.data) == CHANNELS
    assert all(v is None for col in p.data for v in col)


def test_pattern_set_get():
    p = Pattern("P1", 8)
    p.set(0, 3, 60)
    assert p.get(0, 3) == 60


def test_pattern_set_rows_grow_and_shrink():
    p = Pattern("P1", 4)
    p.set(0, 0, 60)
    p.set(1, 3, 62)
    p.set_rows(8)
    assert p.rows == 8
    assert p.get(0, 0) == 60 and p.get(1, 3) == 62
    assert p.get(0, 7) is None
    p.set_rows(2)                      # schrumpfen -> Reihe 3 faellt weg
    assert p.rows == 2
    assert p.get(0, 0) == 60
    assert all(len(col) == 2 for col in p.data)


def test_pattern_copy_is_independent():
    p = Pattern("P1", 4); p.set(0, 0, 60)
    q = p.copy("P2")
    q.set(0, 0, 72)
    assert p.get(0, 0) == 60 and q.get(0, 0) == 72
    assert q.name == "P2"


def test_note_helpers():
    assert note_name(60) == "C4"
    assert round(midi_to_freq(69)) == 440


# --------------------------------------------------------------- Song-Ops

def test_song_defaults():
    s = Song()
    assert s.bpm == 120
    assert len(s.patterns) == 1
    assert s.order == [0]
    assert len(s.waves) == TONAL


def test_add_and_duplicate_pattern():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    i = s.add_pattern("Bridge", 8)
    assert i == 1 and s.patterns[1].rows == 8
    j = s.duplicate_pattern(0)
    assert j == 2 and s.patterns[2].get(0, 0) == 60


def test_remove_pattern_fixes_order():
    s = Song()
    s.add_pattern(); s.add_pattern()      # patterns 0,1,2
    s.order = [0, 1, 2, 1]
    s.remove_pattern(1)                    # pattern 1 weg
    # Verweise auf 1 raus, 2 -> 1
    assert s.order == [0, 1]
    assert len(s.patterns) == 2


def test_remove_last_pattern_noop():
    s = Song()
    s.remove_pattern(0)
    assert len(s.patterns) == 1


def test_remove_pattern_empty_order_falls_back():
    s = Song()
    s.add_pattern()                       # 0,1
    s.order = [1]
    s.remove_pattern(1)
    assert s.order == [0]


def test_order_ops():
    s = Song()
    s.add_pattern()
    s.order_add(1)
    assert s.order == [0, 1]
    j = s.order_move(0, 1)
    assert j == 1 and s.order == [1, 0]
    s.order_remove(0)
    assert s.order == [0]
    s.order_remove(0)                     # letzter bleibt
    assert s.order == [0]


def test_row_ms():
    s = Song(); s.bpm = 120
    assert s.row_ms() == 125              # 60000/120/4


# --------------------------------------------------------------- Flatten

def test_flatten_concatenates_order():
    s = Song()
    s.patterns[0].set_rows(4)
    s.patterns[0].set(0, 0, 69)           # A4 = 440 Hz
    s.patterns[0].set(TONAL, 1, 60)       # Drum-Hit -> 1
    i = s.add_pattern("P2", 2)
    s.patterns[i].set(1, 0, 69)
    s.order = [0, 1, 0]
    total, ch = s.flatten()
    assert total == 4 + 2 + 4
    assert ch[0][0] == 440                # erste Reihe von P0, Kanal 0
    assert ch[TONAL][1] == 1              # Drum-Hit
    assert ch[1][4] == 440               # P2 beginnt bei Reihe 4, Kanal 1
    assert ch[0][6] == 440               # zweite P0-Instanz ab Reihe 6


def test_flatten_empty_song_min_one_row():
    s = Song()
    total, ch = s.flatten()
    assert total == 16                    # ein leeres 16-Reihen-Pattern
    assert all(v == 0 for v in ch[0])


# --------------------------------------------------------------- JSON

def test_json_roundtrip(tmp_path):
    s = Song()
    s.bpm = 140
    s.waves = ["sine", "triangle", "square"]
    s.patterns[0].set(0, 0, 60)
    s.add_pattern("Chorus", 8)
    s.patterns[1].set(2, 3, 64)
    s.order = [0, 1, 1, 0]
    path = str(tmp_path / "song.json")
    s.save_json(path)

    s2 = Song.load_json(path)
    assert s2.bpm == 140
    assert s2.waves == ["sine", "triangle", "square"]
    assert len(s2.patterns) == 2
    assert s2.patterns[0].get(0, 0) == 60
    assert s2.patterns[1].rows == 8 and s2.patterns[1].get(2, 3) == 64
    assert s2.order == [0, 1, 1, 0]


def test_from_dict_filters_bad_order_indices():
    s = Song.from_dict({"bpm": 100, "patterns": [Pattern("A").to_dict()],
                        "order": [0, 5, -1, 0]})
    assert s.order == [0, 0]              # 5 und -1 fallen raus


# --------------------------------------------------------------- GB-Code

def test_gb_code_compiles(tmp_path):
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set(TONAL, 4, 50)
    s.add_pattern("P2", 8)
    s.patterns[1].set(1, 2, 64)
    s.order = [0, 1, 0]
    _check_compiles(tmp_path, s.gb_code())


def test_gb_code_has_expanded_rows():
    s = Song()
    s.patterns[0].set_rows(4)
    s.add_pattern("P2", 4)
    s.order = [0, 1, 0]                    # 12 Reihen total
    code = s.gb_code()
    assert "CONST TRK_ROWS = 12" in code


# --------------------------------------------------------------- Lautstaerke

def test_set_vol_requires_note():
    from gamebasic.tracker import VOL_MAX
    p = Pattern("P")
    # ohne Note ignoriert set_vol
    p.set_vol(0, 0, 8)
    assert p.get_vol(0, 0) is None
    # mit Note wird gesetzt
    p.set(0, 0, 60)
    p.set_vol(0, 0, 8)
    assert p.get_vol(0, 0) == 8
    # Clamping
    p.set_vol(0, 0, 999)
    assert p.get_vol(0, 0) == VOL_MAX
    # 0/None -> Standard (None)
    p.set_vol(0, 0, 0)
    assert p.get_vol(0, 0) is None


def test_clearing_note_clears_vol():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_vol(0, 0, 10)
    p.set(0, 0, None)                     # Note loeschen
    assert p.get_vol(0, 0) is None


def test_set_rows_keeps_vol():
    p = Pattern("P", 8)
    p.set(0, 2, 60)
    p.set_vol(0, 2, 5)
    p.set_rows(4)                         # 2 < 4 -> bleibt
    assert p.get_vol(0, 2) == 5
    p.set_rows(2)                         # 2 >= 2 -> Reihe 2 faellt weg
    assert p.rows == 2


def test_to_dict_omits_empty_vol():
    p = Pattern("P")
    assert "vol" not in p.to_dict()
    p.set(0, 0, 60)
    p.set_vol(0, 0, 7)
    assert "vol" in p.to_dict()


def test_vol_json_roundtrip(tmp_path):
    s = Song()
    s.patterns[0].set(1, 3, 64)
    s.patterns[0].set_vol(1, 3, 12)
    path = str(tmp_path / "vol.json")
    s.save_json(path)
    s2 = Song.load_json(path)
    assert s2.patterns[0].get_vol(1, 3) == 12


def test_vol_copy_independent():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_vol(0, 0, 9)
    q = p.copy()
    q.set_vol(0, 0, 3)
    assert p.get_vol(0, 0) == 9          # Original unveraendert


def test_vol_to_pct_mapping():
    from gamebasic.tracker import VOL_MAX, vol_to_pct
    assert vol_to_pct(VOL_MAX) == 100
    assert vol_to_pct(1) >= 1            # nie 0 (reserviert fuer Standard)


def test_gb_code_with_volume_compiles(tmp_path):
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_vol(0, 0, 12)
    code = s.gb_code()
    assert "DIM trkV0[TRK_ROWS]" in code
    assert "FUNCTION TRACKER_AMP" in code
    _check_compiles(tmp_path, code)


def test_gb_code_without_volume_has_no_amp_helper():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    code = s.gb_code()
    assert "TRACKER_AMP" not in code     # ohne Lautstaerke kein Helfer/Overhead


# --------------------------------------------------------------- Pitch-Slide

def test_set_slide_requires_note():
    from gamebasic.tracker import SLIDE_MAX
    p = Pattern("P")
    p.set_slide(0, 0, 3)
    assert p.get_slide(0, 0) is None     # ohne Note ignoriert
    p.set(0, 0, 60)
    p.set_slide(0, 0, 3)
    assert p.get_slide(0, 0) == 3
    p.set_slide(0, 0, 99)                # Clamping
    assert p.get_slide(0, 0) == SLIDE_MAX
    p.set_slide(0, 0, -99)
    assert p.get_slide(0, 0) == -SLIDE_MAX
    p.set_slide(0, 0, 0)                 # 0 -> None
    assert p.get_slide(0, 0) is None


def test_clearing_note_clears_slide():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_slide(0, 0, 4)
    p.set(0, 0, None)
    assert p.get_slide(0, 0) is None


def test_slide_hz_per_s_direction():
    from gamebasic.tracker import slide_hz_per_s, midi_to_freq
    f = midi_to_freq(60)
    assert slide_hz_per_s(f, 2, 125) > 0     # aufwaerts -> positiv
    assert slide_hz_per_s(f, -2, 125) < 0    # abwaerts -> negativ
    assert slide_hz_per_s(f, 0, 125) == 0


def test_slide_json_roundtrip(tmp_path):
    s = Song()
    s.patterns[0].set(0, 1, 60)
    s.patterns[0].set_slide(0, 1, -5)
    path = str(tmp_path / "slide.json")
    s.save_json(path)
    s2 = Song.load_json(path)
    assert s2.patterns[0].get_slide(0, 1) == -5


def test_gb_code_with_slide_uses_sfx(tmp_path):
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_slide(0, 0, 2)
    s.patterns[0].set(0, 1, 62)          # ohne Slide -> AUDIO_TONE
    code = s.gb_code()
    assert "DIM trkSl0[TRK_ROWS]" in code
    assert "AUDIO_SFX" in code
    assert "AUDIO_TONE" in code          # Nicht-Slide-Note nutzt weiter TONE
    _check_compiles(tmp_path, code)


def test_gb_code_without_slide_has_no_sfx():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    assert "AUDIO_SFX" not in s.gb_code()


# --------------------------------------------------------------- Effekt-Spalte

def test_set_fx_requires_note():
    from gamebasic.tracker.song import FX_ARP, FX_NONE
    p = Pattern("P")
    p.set_fx(0, 0, FX_ARP, 0x47)
    assert p.get_fx(0, 0) == (FX_NONE, 0)    # ohne Note ignoriert
    p.set(0, 0, 60)
    p.set_fx(0, 0, FX_ARP, 0x47)
    assert p.get_fx(0, 0) == (FX_ARP, 0x47)
    p.set_fx(0, 0, FX_NONE)                  # loeschen
    assert p.get_fx(0, 0) == (FX_NONE, 0)


def test_set_fx_clamps_param():
    from gamebasic.tracker.song import FX_RET
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_fx(0, 0, FX_RET, 999)
    assert p.get_fx(0, 0) == (FX_RET, 255)


def test_clearing_note_clears_fx():
    from gamebasic.tracker.song import FX_VIB, FX_NONE
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_fx(0, 0, FX_VIB, 0x68)
    p.set(0, 0, None)
    assert p.get_fx(0, 0) == (FX_NONE, 0)


def test_fx_json_roundtrip(tmp_path):
    from gamebasic.tracker.song import FX_ARP
    s = Song()
    s.patterns[0].set(2, 5, 64)
    s.patterns[0].set_fx(2, 5, FX_ARP, 0x37)
    path = str(tmp_path / "fx.json")
    s.save_json(path)
    s2 = Song.load_json(path)
    assert s2.patterns[0].get_fx(2, 5) == (FX_ARP, 0x37)


def test_fx_copy_independent():
    from gamebasic.tracker.song import FX_ARP, FX_RET
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_fx(0, 0, FX_ARP, 0x47)
    q = p.copy()
    q.set_fx(0, 0, FX_RET, 3)
    assert p.get_fx(0, 0) == (FX_ARP, 0x47)  # Original unveraendert


def test_to_dict_omits_empty_fx():
    from gamebasic.tracker.song import FX_ARP
    p = Pattern("P")
    assert "fx" not in p.to_dict()
    p.set(0, 0, 60)
    p.set_fx(0, 0, FX_ARP, 0x47)
    assert "fx" in p.to_dict() and "fxp" in p.to_dict()


# --------------------------------------------------------------- Instrumente

def _sample_inst(name="Smp"):
    import numpy as np
    from gamebasic.tracker.instrument import Instrument
    t = np.arange(2205) / 44100.0
    return Instrument.from_array(name, np.sin(2 * np.pi * 440 * t), 44100, 69)


def test_instrument_for_channel_default_is_synth():
    s = Song()
    inst = s.instrument_for_channel(0)
    assert inst.kind == "synth" and inst.waveform == "square"
    assert s.instrument_for_channel(TONAL).waveform == "noise"


def test_instrument_for_channel_returns_stable_object_when_unassigned():
    """Regression: instrument_for_channel() baute frueher bei JEDEM Aufruf
    ein neues Instrument.synth()-Objekt fuer einen Kanal ohne explizite
    Zuweisung -- trackereditor_qt._render_sound()s id(inst)-basierter
    Sound-Cache traf dadurch nie (jede Note wurde neu synthetisiert, auch
    identische Wiederholungen). Jetzt wird das fluechtige Synth pro
    (Kanal, Wellenform) wiederverwendet -- id() bleibt ueber Aufrufe stabil."""
    s = Song()
    a = s.instrument_for_channel(0)
    b = s.instrument_for_channel(0)
    assert a is b

    # Wellenform-Wechsel -> bewusst ein ANDERES (neues) Instrument, keine
    # veraltete Instanz der alten Wellenform.
    s.waves[0] = "saw"
    c = s.instrument_for_channel(0)
    assert c is not a
    assert c.waveform == "saw"

    # Nach dem Zuweisen eines echten Instruments hat die Identitaets-Frage
    # keine Bedeutung mehr (kommt direkt aus dem Pool, nicht aus dem Cache).
    idx = s.add_instrument(_sample_inst("Kick"))
    s.channel_inst[0] = idx
    assert s.instrument_for_channel(0) is s.instruments[idx]


def test_assign_instrument_to_channel():
    s = Song()
    idx = s.add_instrument(_sample_inst("Kick"))
    s.channel_inst[0] = idx
    assert s.instrument_for_channel(0).kind == "sample"
    assert s.instrument_for_channel(0).name == "Kick"


def test_remove_instrument_fixes_assignments():
    s = Song()
    a = s.add_instrument(_sample_inst("A"))
    b = s.add_instrument(_sample_inst("B"))
    s.channel_inst[0] = a
    s.channel_inst[1] = b
    assert s.remove_instrument(a) is True
    assert s.channel_inst[0] is None        # geloeschtes -> None
    assert s.channel_inst[1] == 0           # nachgerueckt


def test_instruments_json_roundtrip(tmp_path):
    s = Song()
    idx = s.add_instrument(_sample_inst("Lead"))
    s.channel_inst[2] = idx
    path = str(tmp_path / "inst.json")
    s.save_json(path)
    s2 = Song.load_json(path)
    assert len(s2.instruments) == 1
    assert s2.instruments[0].kind == "sample"
    assert s2.channel_inst[2] == 0


def test_song_without_instruments_omits_field():
    s = Song()
    assert "instruments" not in s.to_dict()


def test_gb_code_sample_channel_commented(tmp_path):
    s = Song()
    idx = s.add_instrument(_sample_inst("Smp"))
    s.channel_inst[0] = idx
    s.patterns[0].set(0, 0, 60)
    code = s.gb_code()
    assert "Sample-Instrument" in code      # Hinweis-Kommentar
    # Synth-Kanaele + Drum bleiben gueltig -> kompiliert weiter
    _check_compiles(tmp_path, code)


# --- has_gb_code_fidelity_gaps() (Review-Fund: GB-Code-Export ignoriert ---
# --- Effekt-Spalte + Per-Note-Instrument komplett, ohne Warnung) ----------

def test_no_fidelity_gaps_on_plain_song():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    assert s.has_gb_code_fidelity_gaps() is False


def test_fidelity_gap_detected_for_fx():
    from gamebasic.tracker.song import FX_ARP
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_fx(0, 0, FX_ARP, 0x37)
    assert s.has_gb_code_fidelity_gaps() is True


def test_fidelity_gap_detected_for_per_note_instrument():
    s = Song()
    idx = s.add_instrument(_sample_inst("Lead"))
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_inst(0, 0, idx)
    assert s.has_gb_code_fidelity_gaps() is True


def test_fidelity_gap_ignores_fx_none():
    from gamebasic.tracker.song import FX_NONE
    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_fx(0, 0, FX_NONE)
    assert s.has_gb_code_fidelity_gaps() is False
