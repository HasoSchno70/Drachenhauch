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
    # Ein paar verschachtelte .gb-Dateien anlegen, damit der Baum Tiefe hat.
    (tmp_path / "a").mkdir()
    (tmp_path / "a" / "b").mkdir()
    (tmp_path / "top.gb").write_text("PRINT 1", encoding="utf-8")
    (tmp_path / "a" / "mid.gb").write_text("PRINT 2", encoding="utf-8")
    (tmp_path / "a" / "b" / "deep.gb").write_text("PRINT 3", encoding="utf-8")

    from gamebasic.editor_qt.file_browser import FileBrowser
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
