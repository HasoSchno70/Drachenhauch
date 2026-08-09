"""Tests fuer das Fader-Widget (Slider mit QSpinBox-kompatibler API)."""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module")
def _qapp():
    try:
        from PySide6.QtWidgets import QApplication
    except Exception:  # pragma: no cover
        pytest.skip("PySide6 nicht verfuegbar")
    app = QApplication.instance() or QApplication([])
    yield app


def test_int_fader_roundtrip(_qapp):
    from drachenhauch.editor_qt.fader import Fader
    f = Fader("Freq", 50, 8000, 800, step=1, decimals=0, unit="Hz")
    assert f.value() == 800 and isinstance(f.value(), int)
    f.setValue(1234)
    assert f.value() == 1234


def test_float_fader_rounds_to_step(_qapp):
    from drachenhauch.editor_qt.fader import Fader
    f = Fader("Vol", 0.0, 1.0, 0.7, step=0.01, decimals=2)
    assert abs(f.value() - 0.7) < 1e-9
    f.setValue(0.33)
    assert abs(f.value() - 0.33) < 1e-9


def test_fader_clamps_out_of_range(_qapp):
    from drachenhauch.editor_qt.fader import Fader
    f = Fader("X", 0, 10, 5)
    f.setValue(999)
    assert f.value() == 10
    f.setValue(-50)
    assert f.value() == 0


def test_fader_emits_on_setvalue(_qapp):
    from drachenhauch.editor_qt.fader import Fader
    f = Fader("X", 0, 100, 0)
    seen = []
    f.valueChanged.connect(lambda v: seen.append(v))
    f.setValue(42)
    assert seen and abs(seen[-1] - 42) < 1e-9
