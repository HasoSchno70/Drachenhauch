"""Tests fuer command_palette/quick_open (Review-Fund: ein Fehler im
ausgewaehlten Befehl/Callback lief bisher ungeschuetzt durch Qts Signal-
Dispatch statt eine verstaendliche Fehlermeldung zu zeigen)."""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _dummy_parent():
    from PySide6.QtWidgets import QWidget
    w = QWidget()
    w.setGeometry(0, 0, 800, 600)
    return w


def _fake_exec_picks_first(monkeypatch):
    """dlg.exec() blockiert normalerweise auf einer echten Event-Loop --
    hier simulieren wir stattdessen direkt eine Nutzer-Auswahl des ersten
    Eintrags, ohne den Dialog wirklich modal zu zeigen."""
    from drachenhauch.editor_qt import palette as P

    def fake_exec(self):
        self.chosen.emit(self._all_entries[0])
        return 0

    monkeypatch.setattr(P._PickerDialog, "exec", fake_exec)


def test_command_palette_shows_warning_instead_of_crashing(monkeypatch):
    from drachenhauch.editor_qt import palette as P
    from PySide6.QtWidgets import QMessageBox

    _fake_exec_picks_first(monkeypatch)
    warnings = []
    monkeypatch.setattr(QMessageBox, "warning", lambda *a, **kw: warnings.append(a))

    def boom():
        raise RuntimeError("kaputt")

    P.command_palette(_dummy_parent(), [("Test-Befehl", "", boom)])
    assert len(warnings) == 1


def test_command_palette_runs_normal_command_without_warning(monkeypatch):
    from drachenhauch.editor_qt import palette as P
    from PySide6.QtWidgets import QMessageBox

    _fake_exec_picks_first(monkeypatch)
    warnings = []
    monkeypatch.setattr(QMessageBox, "warning", lambda *a, **kw: warnings.append(a))

    ran = []
    P.command_palette(_dummy_parent(), [("Test-Befehl", "", lambda: ran.append(1))])
    assert ran == [1]
    assert warnings == []


def test_quick_open_shows_warning_instead_of_crashing(monkeypatch, tmp_path):
    from drachenhauch.editor_qt import palette as P
    from PySide6.QtWidgets import QMessageBox

    (tmp_path / "a.dh").write_text("PRINT 1\n", encoding="utf-8")
    _fake_exec_picks_first(monkeypatch)
    warnings = []
    monkeypatch.setattr(QMessageBox, "warning", lambda *a, **kw: warnings.append(a))

    def boom(_path):
        raise OSError("Datei nicht lesbar")

    P.quick_open(_dummy_parent(), tmp_path, boom)
    assert len(warnings) == 1
