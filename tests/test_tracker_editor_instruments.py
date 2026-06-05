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
