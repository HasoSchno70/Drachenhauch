"""Display-/Monitor-Builtins: nur Registrierung.

MONITOR_COUNT/WIDTH/HEIGHT/REFRESH/NAME/X/Y, CURRENT_MONITOR,
SET_WINDOW_MONITOR, WINDOW_X/Y, SET_WINDOW_POS sind raylib-Engine-Builtins
(nativ in gbrt -- sie fragen reale Monitor-Hardware/Fensterposition ab und
brauchen daher einen echten Desktop-GL-Kontext; kein Funktionstest via
run_gb). Hier wird geprueft, dass die Builtins im eingefrorenen gbrt-Index
stehen -- sonst warnt der Editor live und der Drift-Test schlaegt an, sobald
ein Beispiel (examples/120_monitors.gb) sie nutzt.
"""
from gamebasic.editor_qt.gbrt_meta import builtin_names_lower


def test_monitor_builtins_registered():
    n = builtin_names_lower()
    for name in ("monitor_count", "current_monitor", "monitor_width",
                 "monitor_height", "monitor_refresh", "monitor_name",
                 "monitor_x", "monitor_y", "set_window_monitor",
                 "window_x", "window_y", "set_window_pos"):
        assert name in n, name
