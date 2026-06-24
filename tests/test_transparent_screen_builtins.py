"""Transparente Fenster / Overlay: nur Registrierung.

SCREEN_TRANSPARENT, WINDOW_UNDECORATED, WINDOW_TOPMOST sind raylib-Engine-
Builtins (Fenster-Erzeugungs-Flag bzw. Window-State -- brauchen einen echten
Desktop). Kein Funktionstest via run_gb; live verifiziert (examples/123_overlay.gb
+ 124_glass_window.gb). Hier wird geprueft, dass sie im eingefrorenen gbrt-Index
stehen -- sonst warnt der Editor live und der Drift-Test schlaegt an, sobald ein
Beispiel sie nutzt.
"""
from gamebasic.editor_qt.gbrt_meta import builtin_names_lower


def test_transparent_window_builtins_registered():
    n = builtin_names_lower()
    for name in ("screen_transparent", "window_undecorated", "window_topmost"):
        assert name in n, name
