"""Tree-View-Builtins: nur Registrierung. Render/Klick brauchen einen GL-Kontext
und werden live ueber examples/137_gui_tree.dh abgenommen; das Baum-Modell
(Hierarchie, Sichtbarkeit, Auswahl) deckt ein Rust-Unit-Test ab."""
from drachenhauch.editor_qt.dhrt_meta import builtin_names_lower


def test_tree_builtins_registered():
    n = builtin_names_lower()
    for name in ("gui_tree", "gui_tree_add", "gui_tree_clear", "gui_tree_selected",
                 "gui_tree_set_selected", "gui_tree_label", "gui_tree_expand"):
        assert name in n, name
