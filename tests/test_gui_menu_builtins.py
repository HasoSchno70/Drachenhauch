"""Menue-Builtins des gui-Moduls: nur Registrierung.

GUI_MENU/GUI_CONTEXT/GUI_MENU_ITEM/GUI_MENU_SEPARATOR brauchen einen echten
GL-Kontext (Maus/Tastatur/Rendering) -> kein Funktionstest via run_gb. Live
verifiziert ueber examples/129_gui_menu.gb. Hier wird geprueft, dass sie im
eingefrorenen dhrt-Index stehen (sonst Editor-Warnung + Drift-Test rot).
"""
from gamebasic.editor_qt.dhrt_meta import builtin_names_lower


def test_gui_menu_builtins_registered():
    n = builtin_names_lower()
    for name in ("gui_menu", "gui_context", "gui_menu_item", "gui_menu_separator"):
        assert name in n, name
