"""Tests fuer die Sample-Instrument-Integration im Tracker-Editor (offscreen)."""
import os
import wave
from pathlib import Path

import numpy as np
import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _editor():
    try:
        from gamebasic.trackereditor_qt import TrackerEditor
        return TrackerEditor(Path("."))
    except Exception as exc:  # pragma: no cover
        pytest.skip(f"Editor nicht konstruierbar: {exc}")


def _write_wav(path, freq=440, secs=0.1, sr=44100):
    t = np.arange(int(sr * secs)) / sr
    i16 = (np.sin(2 * np.pi * freq * t) * 30000).astype("<i2")
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(sr)
        w.writeframes(i16.tobytes())


def test_load_sample_file_creates_instrument(tmp_path):
    ed = _editor()
    p = tmp_path / "kick.wav"
    _write_wav(p)
    inst = ed._instrument_from_file(str(p))
    assert inst is not None and inst.kind == "sample"
    assert inst.name == "kick"


def test_assign_instrument_to_channel_and_preview(tmp_path):
    ed = _editor()
    p = tmp_path / "lead.wav"
    _write_wav(p)
    inst = ed._instrument_from_file(str(p))
    ed.song.add_instrument(inst)
    ed._refresh_instruments()
    assert ed.inst_combo.count() == 1
    # Auf Kanal 0 zuweisen
    ed.inst_combo.setCurrentIndex(0)
    ed.assign_combo.setCurrentIndex(0)
    ed._assign_instrument()
    assert ed.song.channel_inst[0] == 0
    assert ed.song.instrument_for_channel(0).kind == "sample"
    # Preview rendert ueber das Sample (kein Crash, Sound-Objekt oder None)
    ed._sound(0, 60)            # darf nicht werfen
    # Wieder auf Synth
    ed._unassign_channel()
    assert ed.song.channel_inst[0] is None


def test_remove_instrument_updates_combo(tmp_path):
    ed = _editor()
    p = tmp_path / "s.wav"; _write_wav(p)
    ed.song.add_instrument(ed._instrument_from_file(str(p)))
    ed._refresh_instruments()
    ed.inst_combo.setCurrentIndex(0)
    ed._remove_instrument()
    assert ed.inst_combo.count() == 0
    assert len(ed.song.instruments) == 0


def test_instrument_dialog_applies_loop_and_env(tmp_path):
    from gamebasic.trackereditor_qt import _InstrumentDialog
    ed = _editor()
    p = tmp_path / "pad.wav"; _write_wav(p, secs=0.2)
    inst = ed._instrument_from_file(str(p))
    dlg = _InstrumentDialog(inst)
    dlg.base.setValue(72)
    dlg.loop_mode.setCurrentText("pingpong")
    dlg.loop_start.setValue(100)
    dlg.loop_end.setValue(3000)
    dlg.atk.setValue(20)
    dlg.rel.setValue(40)
    dlg.sus.setValue(0.7)
    dlg.apply_to()
    assert inst.base_note == 72
    assert inst.loop_mode == "pingpong"
    assert inst.loop_start == 100 and inst.loop_end == 3000
    assert inst.has_loop() is True
    assert inst.env_attack_ms == 20 and inst.env_release_ms == 40
    assert abs(inst.env_sustain - 0.7) < 1e-6
