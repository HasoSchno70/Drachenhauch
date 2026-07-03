"""Tests fuer die Render-Buendelung im Sprite-Editor-Canvas.

Review-Fund: `_render_frame_pixmap()` (composite() + resize() + QPixmap-
Konvertierung) lief frueher SYNCHRON bei jedem `invalidate_all()`/
`set_preview()`-Aufruf -- also bei jedem einzelnen mouseMoveEvent waehrend
eines Drags. Bei grossen Sprites/hohem Zoom spuerbar laggy. Jetzt ueber
einen Zero-Timer gebuendelt: mehrere Aufrufe VOR dem naechsten Event-Loop-
Tick fassen sich zu einem einzigen Render zusammen.
"""
import os
from pathlib import Path

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

pytest.importorskip("PySide6")

from PySide6.QtCore import Qt


@pytest.fixture(scope="module")
def app():
    from PySide6.QtWidgets import QApplication
    return QApplication.instance() or QApplication([])


def _window(app):
    from gamebasic.spriteeditor_qt import SpriteEditorWindow
    return SpriteEditorWindow(project_root=Path("."))


def _pixel(canvas, x, y):
    z = canvas._zoom
    img = canvas._frame_item.pixmap().toImage()
    return img.pixelColor(x * z + z // 2, y * z + z // 2).getRgb()


def test_moves_are_coalesced_until_event_loop_tick(app):
    win = _window(app)
    canvas = win.canvas
    win.activate_tool("pencil")
    win.set_fg((255, 0, 0, 255))

    assert not canvas._render_pending
    win.tool.begin(win, 2, 2, Qt.LeftButton)
    win.tool.move(win, 3, 3)
    win.tool.move(win, 4, 4)
    # Mehrere invalidate_all()-Aufrufe VOR dem Event-Loop-Tick -> genau
    # EIN ausstehender Render, nicht drei.
    assert canvas._render_pending
    # Der zuletzt gemalte Pixel ist noch NICHT sichtbar -- der Render
    # steht noch aus.
    assert _pixel(canvas, 4, 4) == (0, 0, 0, 0)

    app.processEvents()

    assert not canvas._render_pending
    assert _pixel(canvas, 4, 4) == (255, 0, 0, 255)
    win.tool.end(win, 4, 4)


def test_render_reflects_final_state_not_intermediate(app):
    """Zwischenstaende gehen beim Buendeln nicht verloren -- der finale
    Dokument-Zustand wird gerendert, auch wenn dazwischen mehrfach
    invalidate_all() aufgerufen wurde."""
    win = _window(app)
    canvas = win.canvas
    win.activate_tool("pencil")
    win.set_fg((0, 255, 0, 255))

    win.tool.begin(win, 0, 0, Qt.LeftButton)
    for i in range(1, 6):
        win.tool.move(win, i, i)
    win.tool.end(win, 5, 5)

    app.processEvents()
    assert _pixel(canvas, 5, 5) == (0, 255, 0, 255)
    assert _pixel(canvas, 0, 0) == (0, 255, 0, 255)
