"""Tooltip-Builtin: nur Registrierung. Die Hover-/Render-Logik braucht einen
GL-Kontext und wird live ueber examples/133_gui_tooltip.gb abgenommen."""
from drachenhauch.editor_qt.dhrt_meta import builtin_names_lower


def test_tooltip_builtin_registered():
    assert "gui_tooltip" in builtin_names_lower()
