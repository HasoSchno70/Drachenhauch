"""TextArea + modale Dialoge: nur Registrierung. Live verifiziert ueber
examples/132_gui_textarea.dh (Rendering/Maus/Tastatur bzw. native Dialoge)."""
from drachenhauch.editor_qt.dhrt_meta import builtin_names_lower


def test_textarea_and_dialog_builtins_registered():
    n = builtin_names_lower()
    for name in ("gui_textarea", "gui_message", "gui_confirm"):
        assert name in n, name
