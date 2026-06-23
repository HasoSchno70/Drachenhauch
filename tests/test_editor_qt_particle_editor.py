"""Tests fuer Partikel-Editor-Verbesserungen:
- min/max-Klemmung (Anzeige == Verhalten),
- Burst (einmalige Emission),
- Hintergrund-Umschalter,
- lauffaehige Test-Demo ist gueltiger GB-Code (gegen gbrt --check).
"""
import json
import os
import subprocess
import tempfile
from pathlib import Path

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

pytest.importorskip("PySide6")

_ROOT = Path(__file__).resolve().parent.parent


@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


@pytest.fixture
def editor(app):
    from gamebasic.particleeditor_qt import ParticleEditor
    return ParticleEditor(project_root=_ROOT)


def test_minmax_enforced_on_change(editor):
    """max-Widget zieht sichtbar nach, wenn min darueber gesetzt wird -- sonst
    zeigte die Spinbox einen Wert, den die Sim still mit max(min,max) ignoriert."""
    editor.vx_min.setValue(40)
    editor.vx_max.setValue(10)
    editor._on_change()
    assert editor.vx_max.value() >= editor.vx_min.value()
    # Gilt auch fuer Groesse/Lebensdauer (randint braucht min<=max).
    editor.size_min.setValue(8)
    editor.size_max.setValue(2)
    editor._on_change()
    assert editor.size_max.value() >= editor.size_min.value()


def test_burst_emits_once(editor):
    editor.sys.clear()
    editor.preview.burst(50)
    assert editor.sys.count() == 50


def test_bg_modes_switch_without_error(editor):
    modes = [editor.bg_combo.itemData(i) for i in range(editor.bg_combo.count())]
    assert modes == ["dark", "black", "light", "checker"]
    for i in range(editor.bg_combo.count()):
        editor.bg_combo.setCurrentIndex(i)
    assert editor.preview.bg_mode == "checker"


def _find_gbrt():
    base = _ROOT / "rust" / "gb_runtime" / "target"
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


def test_runnable_demo_is_valid_gb(editor, tmp_path):
    """Die 'In GameBasic testen'-Demo muss gueltiger GB-Code sein."""
    gbrt = _find_gbrt()
    if gbrt is None:
        pytest.skip("gbrt nicht gebaut")
    demo = editor._build_runnable_demo()
    f = tmp_path / "demo.gb"
    f.write_text(demo, encoding="utf-8")
    r = subprocess.run([str(gbrt), "--check", str(f)],
                       capture_output=True, text=True, timeout=30)
    diags = json.loads(r.stdout or "[]")
    errors = [d for d in diags if d.get("severity") != "warning"]
    assert not errors, errors


def test_export_snippet_is_valid_gb(editor, tmp_path):
    """Auch das Export-Snippet (auskommentierter Loop) muss kompilieren."""
    gbrt = _find_gbrt()
    if gbrt is None:
        pytest.skip("gbrt nicht gebaut")
    # _export baut den Code-String; wir greifen die Snippet-Logik indirekt ueber
    # die gleichen Setter ab, indem wir das Export-Dialog oeffnen und den Text
    # lesen. Einfacher: den Code ueber denselben Pfad wie _export erzeugen.
    editor._export()
    # Text aus dem QPlainTextEdit im Export-Dialog ziehen.
    from PySide6.QtWidgets import QPlainTextEdit
    edit = editor._export_dlg.findChild(QPlainTextEdit)
    assert edit is not None
    f = tmp_path / "snippet.gb"
    f.write_text(edit.toPlainText(), encoding="utf-8")
    r = subprocess.run([str(gbrt), "--check", str(f)],
                       capture_output=True, text=True, timeout=30)
    diags = json.loads(r.stdout or "[]")
    errors = [d for d in diags if d.get("severity") != "warning"]
    assert not errors, errors
