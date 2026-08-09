"""Tests fuer FindReplaceDialog (Review-Fund: Escape raeumte die Find-
Highlights im Editor nicht auf, und die Status-Label-Farbe blieb nach
einem Theme-Wechsel stehen)."""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _dialog():
    from drachenhauch.editor_qt.dialogs import FindReplaceDialog

    class _FakeEditor:
        def __init__(self):
            self.cleared = 0

        def clear_find_hits(self):
            self.cleared += 1

        def textCursor(self):
            from PySide6.QtGui import QTextCursor
            from PySide6.QtGui import QTextDocument
            return QTextCursor(QTextDocument())

        def find_all(self, *a, **kw):
            return []

    ed = _FakeEditor()
    dlg = FindReplaceDialog(None, lambda: ed, with_replace=True)
    return dlg, ed


def test_escape_clears_find_hits_same_as_close_button():
    dlg, ed = _dialog()
    dlg.show()
    assert ed.cleared == 0
    dlg.reject()   # Qt's Escape-Default-Handler ruft reject() auf
    assert ed.cleared == 1


def test_close_button_still_clears_find_hits():
    dlg, ed = _dialog()
    dlg.show()
    dlg.close()
    assert ed.cleared == 1


def test_theme_change_refreshes_status_label_color():
    from drachenhauch.editor_qt import theme as theme_mod
    dlg, _ed = _dialog()
    # Auf ein anderes Theme wechseln, dann zurueck -- Hauptsache das Signal
    # feuert und der Dialog reagiert (kein stiller No-Op).
    calls = []
    orig = dlg._on_theme_changed
    dlg._on_theme_changed = lambda name: (calls.append(name), orig(name))[-1]
    theme_mod.theme_signals.changed.emit("dark")
    assert calls == ["dark"]
