"""Test fuer den 'Ueber Drachenhauch'-Dialog: Autor + Copyright + aktuelle
Beschreibung (keine veralteten Ausfuehrungspfade mehr)."""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

pytest.importorskip("PySide6")


@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


def test_about_dialog_author_and_description(app):
    from PySide6.QtWidgets import QLabel
    from drachenhauch.editor_qt.dialogs import AboutDialog

    dlg = AboutDialog(None)
    texts = [w.text() for w in dlg.findChildren(QLabel)]
    blob = "\n".join(texts)

    assert "Hans Schnorrenberger" in blob
    assert "(C) 2026" in blob
    # Versionsnummer aus drachenhauch.__version__ wird angezeigt.
    import drachenhauch
    assert f"Version {drachenhauch.__version__}" in blob
    assert "0.1.0" not in blob
    # Beschreibung ist aktuell -- die alten drei VM-Pfade sind weg.
    assert "Cython" not in blob
    assert "Tree-Walker" not in blob
    assert "Python-VM" not in blob
    # Neue Runtime erwaehnt.
    assert "dhrt" in blob
    dlg.deleteLater()
