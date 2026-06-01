"""Game-Loop-Grundlagen (DELTA/FPS/SETFPS/SET_FULLSCREEN/SETWINDOWTITLE/
SAVESCREENSHOT) -- raylib/pygame-Engine-Builtins.

Timing/Fenster sind nicht deterministisch und brauchen einen echten Screen,
daher kein Funktionstest hier -- nur dass die Builtins registriert sind und
ueber einen Graphics-Stub die richtigen Methoden ansprechen.
"""
from gamebasic.interpreter import GRAPHICS_BUILTINS


def test_gameloop_builtins_registered():
    for name in ("delta", "fps", "setfps", "set_fullscreen",
                 "setwindowtitle", "savescreenshot"):
        assert name in GRAPHICS_BUILTINS, name


def test_delta_fps_delegate_to_graphics():
    calls = {}

    class _G:
        def delta(self): return 0.016
        def fps(self): return 60
        def set_target_fps(self, n): calls["fps"] = n
        def set_window_title(self, s): calls["title"] = s
        def save_screenshot(self, p): calls["shot"] = p

    g = _G()
    assert GRAPHICS_BUILTINS["delta"](g, []) == 0.016
    assert GRAPHICS_BUILTINS["fps"](g, []) == 60
    GRAPHICS_BUILTINS["setfps"](g, [120])
    GRAPHICS_BUILTINS["setwindowtitle"](g, ["hi"])
    GRAPHICS_BUILTINS["savescreenshot"](g, ["a.png"])
    assert calls == {"fps": 120, "title": "hi", "shot": "a.png"}
