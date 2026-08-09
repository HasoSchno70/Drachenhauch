"""Reiter-Builtins (Tabs): nur Registrierung. Live verifiziert ueber
examples/131_gui_tabs.gb."""
from drachenhauch.editor_qt.dhrt_meta import builtin_names_lower


def test_gui_tab_builtins_registered():
    n = builtin_names_lower()
    for name in ("gui_tabs", "gui_set_tab", "gui_active_tab", "gui_set_active_tab"):
        assert name in n, name
