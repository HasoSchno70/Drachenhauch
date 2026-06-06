"""Game-Loop- + Shader-Builtins: nur Registrierung.

DELTA/FPS/SETFPS/SET_FULLSCREEN/SETWINDOWTITLE/SAVESCREENSHOT + SHADER_* sind
raylib-Engine-Builtins (nativ in gbrt -- Timing/Fenster/GPU brauchen einen echten
Screen, daher kein Funktionstest via run_gb). Hier nur, dass die Builtins bekannt
sind (eingefrorener gbrt-Metadaten-Index). Frueher prueften wir zusaetzlich die
Python-Dispatch-Delegation an einen Graphics-Stub -- diese Python-Schicht wird in
Phase 8 entfernt; das native Verhalten deckt die Beispiel-/Screenshot-Verifikation.
"""
from gamebasic.editor_qt.gbrt_meta import builtin_names_lower


def test_gameloop_builtins_registered():
    n = builtin_names_lower()
    for name in ("delta", "fps", "setfps", "set_fullscreen",
                 "setwindowtitle", "savescreenshot"):
        assert name in n, name


def test_shader_builtins_registered():
    n = builtin_names_lower()
    for name in ("shader_load", "shader_set", "shader_set2", "shader_set3", "postfx"):
        assert name in n, name
