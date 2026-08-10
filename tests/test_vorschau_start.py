"""Vorschau-Laeufe: kein Konsolenfenster, aber Fehler kommen trotzdem an.

Der Weg ueber `QProcess` ist nur dann besser als ein simples
`CREATE_NO_WINDOW`, wenn ein Absturz des Vorschau-Programms weiterhin
gemeldet wird -- vorher war die Konsole die einzige Stelle dafuer. Genau das
pruefen diese Tests.

`waitForFinished` treibt die Prozess-Ereignisse selbst und feuert dabei
`finished` -- so braucht es keine Event-Loop-Schleife (siehe CLAUDE.md zu
`processEvents` in der Testsuite).
"""
from __future__ import annotations

import sys

import pytest

pytest.importorskip("PySide6")

from PySide6.QtWidgets import QApplication, QMessageBox  # noqa: E402

from drachenhauch.editor_qt.vorschau_start import starte_vorschau  # noqa: E402


def _app():
    return QApplication.instance() or QApplication([])


@pytest.fixture
def gemeldet(monkeypatch):
    """Faengt die Meldungen ab, die der Editor zeigen wuerde."""
    raus = []
    monkeypatch.setattr(QMessageBox, "warning",
                        staticmethod(lambda p, t, m, *a, **k: raus.append((t, m))))
    monkeypatch.setattr(QMessageBox, "critical",
                        staticmethod(lambda p, t, m, *a, **k: raus.append((t, m))))
    return raus


def test_absturz_wird_im_editor_gemeldet(gemeldet, tmp_path):
    """Ohne diese Meldung waere das Unterdruecken der Konsole ein Rueckschritt."""
    _app()
    v = starte_vorschau(None, [sys.executable, "-c",
                               "import sys; print('so ging es schief'); sys.exit(3)"],
                        tmp_path, titel="Test-Vorschau")
    v.proc.waitForFinished(15000)
    assert gemeldet, "ein fehlgeschlagener Lauf blieb stumm"
    titel, text = gemeldet[0]
    assert "Test-Vorschau" in titel
    assert "so ging es schief" in text, "die Ausgabe des Programms fehlt in der Meldung"


def test_erfolgreicher_lauf_meldet_nichts(gemeldet, tmp_path):
    _app()
    v = starte_vorschau(None, [sys.executable, "-c", "print('alles gut')"],
                        tmp_path, titel="Test-Vorschau")
    v.proc.waitForFinished(15000)
    assert not gemeldet


def test_stoppen_beendet_wirklich_und_meldet_nichts(gemeldet, tmp_path):
    """Der Stop-Knopf muss den Lauf beenden -- und das ist kein Fehler.

    Die Zeitgrenze ist Absicht: mit `QProcess::terminate()` lief der Prozess
    auf Windows WEITER (terminate schickt dort nur WM_CLOSE an Fenster), der
    Test hing bis zum Timeout. Ein grosszuegiges `waitForFinished` allein
    haette das verdeckt, deshalb wird hier auch die Dauer geprueft.
    """
    import time
    _app()
    v = starte_vorschau(None, [sys.executable, "-c", "import time; time.sleep(30)"],
                        tmp_path, titel="Test-Vorschau")
    assert v.laeuft()
    t0 = time.time()
    v.stoppen()
    assert v.proc.waitForFinished(15000), "der Lauf liess sich nicht beenden"
    assert time.time() - t0 < 5, "Stoppen dauerte viel zu lange (terminate statt kill?)"
    assert not gemeldet


def test_start_ohne_konsolenfenster():
    """QProcess setzt CREATE_NO_WINDOW selbst -- gemessen, nicht geglaubt.

    Der Nachweis steckt in der Messung dieser Sitzung (aus einem
    `pythonw`-Elternprozess: subprocess 3 Konsolenfenster, QProcess 0). Hier
    bleibt festgehalten, dass der Start-Weg wirklich QProcess ist -- ein
    Rueckbau auf `subprocess.Popen` faellt damit auf.
    """
    from PySide6.QtCore import QProcess
    _app()
    v = starte_vorschau(None, [sys.executable, "-c", "pass"], None, titel="x")
    assert isinstance(v.proc, QProcess)
    v.proc.waitForFinished(15000)
