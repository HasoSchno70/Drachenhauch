"""Regression: der Farb-Swatch im Partikel-Editor darf seine Farbe NICHT auf
ein von ihm geoeffnetes Dialogfenster durchschlagen lassen.

Ursache des Bugs: unqualifizierte `background-color`-Stylesheet-Regel am
QPushButton + Dialog an den Button geparentet -> der Dialog erbte die Farbe.
Fix: objectName-eingeschraenkte Regel + Dialog ans Top-Level-Fenster.
"""
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

pytest.importorskip("PySide6")


@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


def _red_pixel_fraction(widget, app):
    widget.show()
    app.processEvents()
    img = widget.grab().toImage()
    total = 0
    red = 0
    for y in range(0, img.height(), 6):
        for x in range(0, img.width(), 6):
            total += 1
            p = img.pixelColor(x, y)
            if p.red() > 180 and p.green() < 90 and p.blue() < 90:
                red += 1
    return red / max(1, total)


def test_swatch_stylesheet_is_scoped(app):
    from drachenhauch.particleeditor_qt import _ColorButton
    b = _ColorButton(0xFF0000)
    assert b.objectName() == "gbColorSwatch"
    # Regel ist auf den Button eingeschraenkt (kein unqualifiziertes
    # background-color, das auf Kinder durchschlaegt).
    assert "QPushButton#gbColorSwatch" in b.styleSheet()


def test_child_dialog_does_not_inherit_swatch_color(app):
    from PySide6.QtWidgets import QColorDialog
    from PySide6.QtGui import QColor
    from drachenhauch.particleeditor_qt import _ColorButton

    b = _ColorButton(0xFF0000)            # knallrot
    # Selbst wenn ein Dialog an den Button geparentet wird, darf er NICHT
    # rot werden (Regel ist objectName-skopiert).
    dlg = QColorDialog(QColor("white"), b)
    dlg.setOption(QColorDialog.ColorDialogOption.DontUseNativeDialog, True)
    frac = _red_pixel_fraction(dlg, app)
    dlg.close()
    assert frac < 0.2, f"Dialog-Hintergrund zu rot (Anteil {frac:.2f}) -- Farbe schlaegt durch"
