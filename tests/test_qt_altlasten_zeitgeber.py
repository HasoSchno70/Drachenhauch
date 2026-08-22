"""Kein Zeitgeber darf einen Test ueberleben, den die Aufraeum-Fixture nicht sieht.

Hintergrund (gemessen am 2026-08-22): die Testdateien lassen ihre Editor-Fenster
stehen, und `_disarm_leftover_qt_widgets()` in conftest.py nimmt ihnen danach die
Zuendschnur. Es findet die Zeitgeber aber ueber `findChildren(QTimer)` ab den
TOP-LEVEL-FENSTERN -- ein Zeitgeber, dessen Besitzer gar nicht im Objektbaum
eines Fensters haengt, ist fuer diesen Weg unsichtbar und bleibt scharf.

Genau das war bei `SnapshotUndo` der Fall: elternloses `QObject`, sein
`QTimer(self)` also ausserhalb jedes Fensterbaums. Ueber acht Editor-Testdateien
gezaehlt blieben so 2111 Mal scharfe Entprell-Zeitgeber (1 ms und 250 ms) beim
START eines FREMDEN Tests stehen. Der naechste `app.processEvents()` -- gleich
welche Datei ihn ruft -- liess sie feuern und damit `capture()`/`changed` auf
einem laengst verlassenen Editor-Fenster laufen. Das ist der Zuendfunke, den der
sporadische Access-Violation-Absturz der CI brauchte.

Der Test sichert die Ursache ab, nicht den Absturz (der ist ein Rennen und
taugt nicht als Zusicherung): der Entprell-Zeitgeber jedes Editors muss der
Aufraeum-Fixture SICHTBAR sein und nach ihr stillstehen.
"""
import os
from pathlib import Path

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")

pytest.importorskip("PySide6")

@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


def _editoren():
    """(Name, Fabrik) je Editor mit `SnapshotUndo`-Historie."""
    from drachenhauch.particleeditor_qt import ParticleEditor
    from drachenhauch.scoreeditor_qt import ScoreEditor
    from drachenhauch.sfxeditor_qt import SfxGenerator
    from drachenhauch.trackereditor_qt import TrackerEditor
    return [
        ("ParticleEditor", lambda: ParticleEditor(Path("."))),
        ("ScoreEditor", lambda: ScoreEditor(Path("."))),
        ("SfxGenerator", lambda: SfxGenerator(Path("."))),
        ("TrackerEditor", lambda: TrackerEditor(Path("."))),
    ]


@pytest.mark.parametrize("name,fabrik", _editoren(), ids=lambda v: v if isinstance(v, str) else "")
def test_entprell_zeitgeber_ist_fuer_die_aufraeum_fixture_sichtbar(
        app, qt_altlasten_entschaerfen, name, fabrik):
    from PySide6.QtCore import QTimer

    ed = fabrik()
    zeitgeber = ed.undo._timer

    # Der Weg, den `_disarm_leftover_qt_widgets()` geht: ab dem Fenster nach
    # unten. Vor der Korrektur war `SnapshotUndo` elternlos -- der Zeitgeber
    # tauchte hier NICHT auf und blieb deshalb scharf stehen.
    assert zeitgeber in ed.findChildren(QTimer), (
        f"{name}: der Entprell-Zeitgeber der Undo-Historie haengt nicht im "
        f"Objektbaum des Fensters -- die Aufraeum-Fixture kann ihn nicht sehen")

    ed.undo.mark()                       # scharf machen (entprellt)
    assert zeitgeber.isActive()

    qt_altlasten_entschaerfen()
    assert not zeitgeber.isActive(), (
        f"{name}: Zeitgeber laeuft nach dem Aufraeumen weiter und feuert in "
        f"den naechsten Test hinein")
