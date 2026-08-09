"""Tests fuer `_companion_import_error` (Review-Fund: 5 `_open_*_editor`-
Methoden wiederholten denselben try/except-SystemExit/except-Exception-Block
mit nur leicht unterschiedlichem Anzeigetext -- jetzt in einem gemeinsamen
Helfer konsolidiert)."""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


@pytest.fixture
def win(tmp_path, monkeypatch):
    from drachenhauch.editor_qt import main_window as mw
    monkeypatch.setattr(mw, "save_settings", lambda *_a, **_kw: None)
    monkeypatch.setattr(mw, "clear_autosaves", lambda *_a, **_kw: None)
    return mw.DrachenhauchEditor(tmp_path)


def test_system_exit_shows_not_available_message(win, monkeypatch):
    from PySide6.QtWidgets import QMessageBox
    calls = []
    monkeypatch.setattr(QMessageBox, "warning",
                         lambda *a: calls.append(a) or None)
    win._companion_import_error("Sprite-Editor", SystemExit("kein PySide6"))
    assert len(calls) == 1
    assert "nicht verfuegbar" in calls[0][1]
    assert "kein PySide6" in calls[0][2]


def test_generic_exception_shows_error_with_hint(win, monkeypatch):
    from PySide6.QtWidgets import QMessageBox
    calls = []
    monkeypatch.setattr(QMessageBox, "warning",
                         lambda *a: calls.append(a) or None)
    win._companion_import_error("Partikel-Editor", ImportError("numpy fehlt"),
                                 "Braucht 'PySide6' und 'numpy'.")
    assert len(calls) == 1
    assert "Partikel-Editor-Fehler" in calls[0][1]
    assert "ImportError: numpy fehlt" in calls[0][2]
    assert "Braucht 'PySide6' und 'numpy'." in calls[0][2]


def test_generic_exception_without_hint_omits_trailing_blank_section(win, monkeypatch):
    from PySide6.QtWidgets import QMessageBox
    calls = []
    monkeypatch.setattr(QMessageBox, "warning",
                         lambda *a: calls.append(a) or None)
    win._companion_import_error("Sprite-Editor", RuntimeError("boom"))
    assert calls[0][2] == "Konnte Sprite-Editor nicht laden:\nRuntimeError: boom"
