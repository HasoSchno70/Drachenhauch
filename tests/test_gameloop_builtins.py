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


def test_shader_builtins_registered_and_degrade():
    for name in ("shader_load", "shader_set", "shader_set2", "shader_set3", "postfx"):
        assert name in GRAPHICS_BUILTINS, name

    seen = {}

    class _G:
        def load_shader(self, src): seen["src"] = src; return -1
        def shader_set_float(self, h, n, v): seen["f"] = (h, n, v)
        def shader_set_vec2(self, h, n, x, y): seen["v2"] = (h, n, x, y)
        def shader_set_vec3(self, h, n, x, y, z): seen["v3"] = (h, n, x, y, z)
        def set_postfx(self, h): seen["post"] = h

    g = _G()
    # pygame-Pfad: SHADER_LOAD liefert -1 (kein GLSL), Rest delegiert.
    assert GRAPHICS_BUILTINS["shader_load"](g, ["crt.fs"]) == -1
    GRAPHICS_BUILTINS["shader_set"](g, [0, "time", 1.5])
    GRAPHICS_BUILTINS["shader_set2"](g, [0, "resolution", 1280, 720])
    GRAPHICS_BUILTINS["shader_set3"](g, [0, "c", 1, 2, 3])
    GRAPHICS_BUILTINS["postfx"](g, [-1])
    assert seen["src"] == "crt.fs"
    assert seen["f"] == (0, "time", 1.5)
    assert seen["v2"] == (0, "resolution", 1280.0, 720.0)
    assert seen["post"] == -1
