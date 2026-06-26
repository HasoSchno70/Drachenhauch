"""Icon-Button/Toolbar-Builtins: nur Registrierung. Render/Klick brauchen einen
GL-Kontext und werden live ueber examples/136_gui_iconbutton.gb abgenommen; die
Modell-Logik (sel-Handle, set_icon, Toolbar) deckt ein Rust-Unit-Test ab."""
from gamebasic.editor_qt.gbrt_meta import builtin_names_lower


def test_iconbutton_builtins_registered():
    n = builtin_names_lower()
    for name in ("gui_icon_button", "gui_set_icon", "gui_toolbar"):
        assert name in n, name
