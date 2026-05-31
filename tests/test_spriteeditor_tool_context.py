"""Tests fuer das Tool-Context-Protocol.

Validiert dass die Mock-App aus den Tool-Tests UND die echte
SpriteEditorWindow-Klasse alle vom Protocol verlangten Attribute /
Methoden anbieten. `runtime_checkable` erlaubt isinstance(...)-Checks
auf Protocols -- aber Vorsicht: das prueft nur die Existenz der Member-
Namen, nicht ihre Signaturen.
"""
import os

import pytest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


def _make_mock_app():
    """Aequivalent zur _MockApp in test_spriteeditor_tools.py -- minimaler
    Tool-Host fuer Tool-Aufrufe ohne Qt-Window."""
    from PIL import Image

    class _Canvas:
        def invalidate_all(self): pass
        def set_preview(self, pil): pass
        def set_selection(self, x0, y0, x1, y1): pass
        def get_selection(self): return None

    class _Frame:
        def __init__(self, w, h):
            self.pixels = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        def snapshot(self): pass

    class _Doc:
        def __init__(self, w=8, h=8):
            self.width = w
            self.height = h
            self.current = _Frame(w, h)

    class _StatusBar:
        def showMessage(self, *_a, **_kw): pass

    class _MockApp:
        def __init__(self):
            self.doc = _Doc()
            self.fg = (255, 0, 0, 255)
            self.bg = (0, 0, 255, 255)
            self.brush_size = 1
            self.symmetry_mode = "none"
            self.canvas = _Canvas()
            self._previous_tool_name = None
        def in_bounds(self, x, y):
            return 0 <= x < 8 and 0 <= y < 8
        def mark_dirty(self): pass
        def set_fg(self, c): self.fg = c
        def set_bg(self, c): self.bg = c
        def activate_tool(self, name, _silent=False): pass
        def statusBar(self): return _StatusBar()
        def show_selection_context_menu(self, _pos): pass

    return _MockApp()


# --- Protocol-Konformitaet ----------------------------------------

def test_mock_app_satisfies_protocol():
    """isinstance gegen runtime_checkable Protocol prueft Methoden-Existenz."""
    from gamebasic.spriteeditor.tool_context import ToolHost
    app = _make_mock_app()
    assert isinstance(app, ToolHost)


def test_real_window_satisfies_protocol():
    """Die echte SpriteEditorWindow-Klasse muss das Protocol erfuellen --
    sonst wuerden Tools darauf nicht laufen."""
    from gamebasic.spriteeditor_qt import SpriteEditorWindow
    from gamebasic.spriteeditor.tool_context import ToolHost
    from pathlib import Path
    w = SpriteEditorWindow(Path("."))
    assert isinstance(w, ToolHost)


def test_doc_protocol():
    """Sub-Protocol fuer Doc -- SpriteDoc muss `width`/`height`/`current`
    haben."""
    from gamebasic.spriteeditor.document import SpriteDoc
    from gamebasic.spriteeditor.tool_context import _DocProto
    doc = SpriteDoc(16, 16)
    assert isinstance(doc, _DocProto)


def test_canvas_protocol():
    """Canvas muss invalidate_all/set_preview/set_selection/get_selection
    bieten."""
    from gamebasic.spriteeditor_qt import SpriteCanvas, SpriteEditorWindow
    from gamebasic.spriteeditor.tool_context import _CanvasProto
    from pathlib import Path
    w = SpriteEditorWindow(Path("."))
    assert isinstance(w.canvas, _CanvasProto)


# --- Negativ-Test: unvollstaendige Mock ist KEIN ToolHost ------------

def test_incomplete_mock_does_not_satisfy_protocol():
    """Ein Mock ohne `mark_dirty` faellt durch."""
    from gamebasic.spriteeditor.tool_context import ToolHost

    class IncompleteApp:
        doc = None
        canvas = None
        fg = None
        bg = None
        brush_size = 1
        symmetry_mode = "none"
        _previous_tool_name = None
        # FEHLEND: in_bounds, mark_dirty, set_fg, set_bg, activate_tool, ...

    assert not isinstance(IncompleteApp(), ToolHost)
