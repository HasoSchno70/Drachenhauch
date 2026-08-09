"""Test fuer den Explorer (FileBrowser): Alles aus-/einklappen per Knopf."""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

from pathlib import Path

import pytest

pytest.importorskip("PySide6")


@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


def _count_expanded(tree):
    n = 0

    def walk(item):
        nonlocal n
        for i in range(item.childCount()):
            child = item.child(i)
            if child.isExpanded():
                n += 1
            walk(child)

    walk(tree.invisibleRootItem())
    return n


def test_expand_collapse_all(app, tmp_path):
    # Ein paar verschachtelte .dh-Dateien anlegen, damit der Baum Tiefe hat.
    (tmp_path / "a").mkdir()
    (tmp_path / "a" / "b").mkdir()
    (tmp_path / "top.dh").write_text("PRINT 1", encoding="utf-8")
    (tmp_path / "a" / "mid.dh").write_text("PRINT 2", encoding="utf-8")
    (tmp_path / "a" / "b" / "deep.dh").write_text("PRINT 3", encoding="utf-8")

    from drachenhauch.editor_qt.file_browser import FileBrowser
    fb = FileBrowser(tmp_path)
    app.processEvents()

    # Knoepfe vorhanden + beschriftet
    assert fb.expand_btn.toolTip() == "Alles ausklappen"
    assert fb.collapse_btn.toolTip() == "Alles einklappen"
    assert not fb.expand_btn.icon().isNull()
    assert not fb.collapse_btn.icon().isNull()

    fb.collapse_btn.click()
    app.processEvents()
    assert _count_expanded(fb.tree) == 0

    fb.expand_btn.click()
    app.processEvents()
    assert _count_expanded(fb.tree) > 0

    fb.deleteLater()


def _find_by_key(tree, key):
    from PySide6.QtCore import Qt
    found = [None]

    def walk(item):
        for i in range(item.childCount()):
            ch = item.child(i)
            if ch.data(0, Qt.ItemDataRole.UserRole) == key:
                found[0] = ch
            walk(ch)

    walk(tree.invisibleRootItem())
    return found[0]


def test_refresh_preserves_expand_state(app, tmp_path):
    # Verschachtelte .dh-Dateien -> Baum mit Tiefe.
    (tmp_path / "a" / "b").mkdir(parents=True)
    (tmp_path / "a" / "mid.dh").write_text("PRINT 2", encoding="utf-8")
    (tmp_path / "a" / "b" / "deep.dh").write_text("PRINT 3", encoding="utf-8")

    from drachenhauch.editor_qt.file_browser import FileBrowser
    fb = FileBrowser(tmp_path)
    app.processEvents()

    # Gradient: Button hat einen expliziten Verlaufs-Stil (qlineargradient).
    assert "qlineargradient" in fb.refresh_btn.styleSheet()

    # Sauberer Ausgangszustand, dann gezielt Sektion + Ordner "a" oeffnen,
    # "a/b" aber ZU lassen.
    fb.collapse_btn.click()
    app.processEvents()
    sec = _find_by_key(fb.tree, ("section", "files"))
    a = _find_by_key(fb.tree, ("dir", tmp_path / "a"))
    assert sec is not None and a is not None
    sec.setExpanded(True)
    a.setExpanded(True)

    before = fb._collect_expanded()
    assert ("dir", tmp_path / "a") in before
    assert ("dir", tmp_path / "a" / "b") not in before

    # Klick auf den Button (nicht nur refresh() direkt) -- muss funktionieren
    # und darf den Auf-/Zu-Zustand NICHT veraendern.
    fb.refresh_btn.click()
    app.processEvents()
    after = fb._collect_expanded()
    assert after == before
    assert ("dir", tmp_path / "a") in after          # offen geblieben
    assert ("dir", tmp_path / "a" / "b") not in after  # zu geblieben

    fb.deleteLater()


def test_watcher_registers_project_root_and_gb_dirs(app, tmp_path):
    """Review-Fund: der Baum synchronisierte sich bisher NUR ueber manuelles
    "Aktualisieren" -- externe Aenderungen (anderes Programm, git checkout,
    zweiter Editor-Prozess) blieben bis zum naechsten Klick unsichtbar."""
    (tmp_path / "sub").mkdir()
    (tmp_path / "sub" / "x.dh").write_text("PRINT 1", encoding="utf-8")

    from drachenhauch.editor_qt.file_browser import FileBrowser
    fb = FileBrowser(tmp_path)
    app.processEvents()

    watched = set(fb._watcher.directories())
    assert str(tmp_path) in watched
    assert str(tmp_path / "sub") in watched

    fb.deleteLater()


def test_external_file_change_triggers_debounced_refresh(app, tmp_path, monkeypatch):
    """Eine externe Aenderung (hier simuliert per direktem Signal-Emit, da
    echte Dateisystem-Events auf manchen CI-Dateisystemen unzuverlaessig/
    verzoegert sind) muss -- nach dem Debounce -- refresh() ausloesen."""
    from drachenhauch.editor_qt.file_browser import FileBrowser
    fb = FileBrowser(tmp_path)
    app.processEvents()

    calls = []
    monkeypatch.setattr(fb, "refresh", lambda: calls.append(1))

    fb._on_fs_changed(str(tmp_path))
    assert calls == []   # noch nicht sofort

    # Debounce-Timer manuell ablaufen lassen, statt real 300ms zu warten.
    fb._watch_debounce.stop()
    fb._watch_debounce.timeout.emit()
    assert calls == [1]

    fb.deleteLater()
