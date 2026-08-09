"""Splitter-Builtin: nur Registrierung. Die Drag-Logik braucht einen GL-Kontext
und wird live ueber examples/135_gui_splitter.dh abgenommen; die Positions-/
Klemm-Logik deckt ein Rust-Unit-Test ab."""
from drachenhauch.editor_qt.dhrt_meta import builtin_names_lower


def test_splitter_builtin_registered():
    assert "gui_splitter" in builtin_names_lower()
