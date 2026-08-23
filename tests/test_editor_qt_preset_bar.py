"""Tests fuer das PresetBar-Widget + die Editor-Integration (offscreen)."""
import os
from pathlib import Path

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def test_preset_bar_apply_and_save(tmp_path, monkeypatch):
    from drachenhauch.editor_qt.preset_library import PresetLibrary
    from drachenhauch.editor_qt.preset_bar import PresetBar
    from PySide6.QtWidgets import QInputDialog

    lib = PresetLibrary(tmp_path / "p.json", builtins={"Feuer": {"v": 1}})
    state = {"v": 0}
    bar = PresetBar(lib, get_state=lambda: dict(state),
                    apply_state=lambda s: state.update(s))

    # Combo hat Platzhalter + 1 Builtin
    assert bar.combo.count() == 2
    # Builtin auswaehlen -> apply_state wird gerufen
    i = bar.combo.findData("Feuer")
    bar.combo.setCurrentIndex(i)
    bar._on_selected(i)
    assert state["v"] == 1
    # Builtin ist nicht loeschbar
    assert not bar.btn_del.isEnabled()

    # Aktuellen Zustand als User-Preset speichern (Dialog gemockt)
    state["v"] = 42
    monkeypatch.setattr(QInputDialog, "getText",
                        staticmethod(lambda *a, **k: ("MeinPreset", True)))
    bar._on_save()
    assert lib.get("MeinPreset") == {"v": 42}
    # Jetzt in der Combo + auswaehlbar/loeschbar
    assert bar.combo.findData("MeinPreset") >= 0
    bar.combo.setCurrentIndex(bar.combo.findData("MeinPreset"))
    bar._update_del_button()
    assert bar.btn_del.isEnabled()


def test_particle_editor_has_factory_presets(_qapp):
    try:
        from drachenhauch.particleeditor_qt import ParticleEditor, _FACTORY_PRESETS
        ed = ParticleEditor(Path("."))
    except ImportError as exc:  # pragma: no cover -- Zusatzpaket fehlt
        # ENG halten: ein breites `except` machte aus jedem kaputten
        # Editor einen gruenen Uebersprung statt eines roten Tests.
        pytest.skip(f"Zusatzpaket fehlt: {exc}")
    # Werks-Presets sind in der Bibliothek
    for name in _FACTORY_PRESETS:
        assert name in ed.presets.names()
    # Ein Preset anwenden setzt die Widgets
    ed._apply_state(ed.presets.get("Feuer"))
    assert ed.mode.currentText() == "glow"
    assert ed.fade.isChecked() is True


def test_sfx_editor_has_preset_bar(_qapp):
    try:
        from drachenhauch.sfxeditor_qt import SfxGenerator
        ed = SfxGenerator(Path("."))
    except ImportError as exc:  # pragma: no cover -- Zusatzpaket fehlt
        # ENG halten: ein breites `except` machte aus jedem kaputten
        # Editor einen gruenen Uebersprung statt eines roten Tests.
        pytest.skip(f"Zusatzpaket fehlt: {exc}")
    assert hasattr(ed, "preset_bar")
    # Roundtrip ueber _params/_apply_params (das, was der Bar nutzt)
    p = ed._params()
    p["base_freq"] = 1234.0
    ed._apply_params(p)
    assert ed.base_freq.value() == 1234


def test_sfx_editor_has_factory_presets(_qapp):
    try:
        from drachenhauch.sfxeditor_qt import SfxGenerator, _FACTORY_PRESETS
        ed = SfxGenerator(Path("."))
    except ImportError as exc:  # pragma: no cover -- Zusatzpaket fehlt
        # ENG halten: ein breites `except` machte aus jedem kaputten
        # Editor einen gruenen Uebersprung statt eines roten Tests.
        pytest.skip(f"Zusatzpaket fehlt: {exc}")
    # Werks-Presets sind -- wie beim Partikel-Editor -- in EINER Bibliothek
    # mit den User-Presets vereint (keine separate Sidebar mehr).
    for name in _FACTORY_PRESETS:
        assert name in ed.presets.names()
        assert ed.presets.is_builtin(name)
    ed._apply_params(ed.presets.get("Laser/Shoot"))
    assert ed.waveform.currentText() == "saw"
    assert ed.base_freq.value() == 1000
