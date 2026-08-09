"""Spinner-Builtin: nur Registrierung. Die Klick-/Mausrad-/Tastatur-Logik
braucht einen GL-Kontext und wird live ueber examples/134_gui_spinner.gb
abgenommen; die Schrittlogik deckt ein Rust-Unit-Test ab."""
from drachenhauch.editor_qt.dhrt_meta import builtin_names_lower


def test_spinner_builtin_registered():
    assert "gui_spinner" in builtin_names_lower()
