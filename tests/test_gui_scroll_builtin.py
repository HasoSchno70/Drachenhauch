"""GUI_WINDOW_SCROLLABLE: nur Registrierung (Scroll = GL/Maus -> live verifiziert
ueber examples/130_gui_scroll.gb)."""
from gamebasic.editor_qt.dhrt_meta import builtin_names_lower


def test_window_scrollable_registered():
    assert "gui_window_scrollable" in builtin_names_lower()
