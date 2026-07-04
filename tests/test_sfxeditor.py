"""Tests fuer den SFX-Generator (`gbsfx`): Synth-Kern + Editor-Exportpfade.

Bisher gab es keine dedizierten Tests -- `synth.py` ist geteilte Mathematik
mit dem `AUDIO_SFX`-Rust-Builtin (Parity-kritisch) und `sfxeditor_qt.py` nur
indirekt ueber `test_audiostudio.py` abgesichert. Reine Funktionen
(`synthesize`/`save_wav`) sind Qt-frei testbar; die GUI-Tests laufen
offscreen wie die uebrigen Editor-Tests.
"""
import os
import wave
from pathlib import Path

import numpy as np
import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

from gamebasic.sfxeditor_qt import _FACTORY_PRESETS, save_wav, synthesize
from gamebasic.synth import WAVEFORMS


# ---------------------------------------------------------------- synthesize
@pytest.mark.parametrize("wf", WAVEFORMS)
def test_synthesize_mono_bounds(wf):
    p = dict(_FACTORY_PRESETS["Jump"])
    p["waveform"] = wf
    s = synthesize(p)
    assert s.ndim == 1
    assert s.size > 0
    assert np.all(np.abs(s) <= 1.0 + 1e-9)


def test_synthesize_stereo_shape():
    p = dict(_FACTORY_PRESETS["Jump"])
    p["stereo_width"] = 0.5
    s = synthesize(p)
    assert s.ndim == 2
    assert s.shape[1] == 2


def test_synthesize_respects_samplerate():
    p = dict(_FACTORY_PRESETS["Jump"])
    full = synthesize(p, sr=44100)
    half = synthesize(p, sr=22050)
    # Gleiche Dauer, halbe Samplerate -> ~halb so viele Samples.
    assert half.size < full.size


def test_synthesize_zero_duration_does_not_crash():
    p = dict(_FACTORY_PRESETS["Jump"])
    p["attack"] = p["sustain"] = p["decay"] = 0
    s = synthesize(p)
    assert s.size >= 1


def test_synthesize_volume_scales_amplitude():
    p = dict(_FACTORY_PRESETS["Blip/Select"])
    p["volume"] = 1.0
    loud = synthesize(p)
    p["volume"] = 0.2
    quiet = synthesize(p)
    assert np.max(np.abs(loud)) > np.max(np.abs(quiet))


# --------------------------------------------------------------------- WAV
def test_save_wav_16bit_roundtrip(tmp_path):
    p = dict(_FACTORY_PRESETS["Jump"])
    samples = synthesize(p)
    path = tmp_path / "out.wav"
    save_wav(path, samples, sr=44100, bit_depth=16)
    with wave.open(str(path), "rb") as w:
        assert w.getsampwidth() == 2
        assert w.getnchannels() == 1
        assert w.getframerate() == 44100
        assert w.getnframes() == samples.size


def test_save_wav_8bit_roundtrip(tmp_path):
    p = dict(_FACTORY_PRESETS["Jump"])
    samples = synthesize(p)
    path = tmp_path / "out8.wav"
    save_wav(path, samples, sr=22050, bit_depth=8)
    with wave.open(str(path), "rb") as w:
        assert w.getsampwidth() == 1
        assert w.getframerate() == 22050
        raw = np.frombuffer(w.readframes(w.getnframes()), dtype=np.uint8)
        assert raw.size == samples.size


def test_save_wav_8bit_encoding_is_unsigned_centered(tmp_path):
    """8-bit PCM ist laut WAV-Spezifikation UNSIGNED (0..255, Mitte=128) --
    anders als 16-bit (signed). round(x*127+128): -1.0->1, 0.0->128, 1.0->255."""
    path = tmp_path / "raw8.wav"
    save_wav(path, np.array([-1.0, 0.0, 1.0]), sr=8000, bit_depth=8)
    with wave.open(str(path), "rb") as w:
        raw = np.frombuffer(w.readframes(w.getnframes()), dtype=np.uint8)
    assert list(raw) == [1, 128, 255]


def test_save_wav_stereo(tmp_path):
    p = dict(_FACTORY_PRESETS["Jump"])
    p["stereo_width"] = 0.8
    samples = synthesize(p)
    path = tmp_path / "stereo.wav"
    save_wav(path, samples)
    with wave.open(str(path), "rb") as w:
        assert w.getnchannels() == 2


# --------------------------------------------------------- SfxGenerator-GUI
@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


@pytest.fixture
def editor(app):
    from gamebasic.sfxeditor_qt import SfxGenerator
    return SfxGenerator(Path("."))


def test_export_code_omits_default_trailing_args(editor, monkeypatch):
    """Ohne SID/Filter/Pan-Abweichung vom Default bleibt AUDIO_SFX kurz --
    keine unnoetigen Trailing-Args."""
    captured = {}
    monkeypatch.setattr(
        editor, "_show_code",
        lambda code, title: captured.update(code=code, title=title))
    editor._apply_params(dict(_FACTORY_PRESETS["Jump"]))
    editor._export_code()
    assert "AUDIO_SFX(" in captured["code"]
    assert captured["code"].count(",") == 8  # 9 Pflicht-Args -> 8 Kommas
    assert "AUDIO_PAN" not in captured["code"]
    assert "PLAYSOUND(snd)" in captured["code"]


def test_export_code_includes_filter_trailing_args(editor, monkeypatch):
    captured = {}
    monkeypatch.setattr(
        editor, "_show_code",
        lambda code, title: captured.update(code=code, title=title))
    p = dict(_FACTORY_PRESETS["Jump"])
    p["flt_cutoff"] = 2000.0
    editor._apply_params(p)
    editor._export_code()
    # flt_cutoff ist der vorletzte Trailing-Slot -> alles bis flt_sweep muss
    # mit-angehaengt sein (positionelle Argumente).
    assert captured["code"].count(",") == 8 + 5  # stereo/duty/pwm*2/cutoff/sweep


def test_export_code_adds_audio_pan_when_panned(editor, monkeypatch):
    captured = {}
    monkeypatch.setattr(
        editor, "_show_code",
        lambda code, title: captured.update(code=code, title=title))
    p = dict(_FACTORY_PRESETS["Jump"])
    p["pan"] = -0.5
    editor._apply_params(p)
    editor._export_code()
    assert "AUDIO_PAN" in captured["code"]


def test_randomize_resets_filter_pwm_pan(editor):
    """Regressionstest: Zufall liess frueher pan/duty/pwm_*/flt_* unveraendert
    -- ein zuvor manuell gesetzter Filter faerbte dadurch jeden Zufalls-Sound
    weiter ein."""
    editor.pan.setValue(0.9)
    editor.flt_cutoff.setValue(4000)
    editor.pwm_depth.setValue(0.3)
    editor._randomize()
    assert editor.pan.value() == 0.0
    assert editor.flt_cutoff.value() == 0
    assert editor.pwm_depth.value() == 0.0
    assert editor.duty.value() == 0.5


def test_wave_view_flags_clipping(editor):
    editor.wave_view.set_samples(np.array([0.0, 1.0, -1.0, 0.5]))
    assert editor.wave_view._clipped is True
    editor.wave_view.set_samples(np.array([0.0, 0.5, -0.5, 0.2]))
    assert editor.wave_view._clipped is False
