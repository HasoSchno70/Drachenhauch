"""Kamera/Viewport-Modul fuer GameBasic.

Verschiebt und zoomt alle Zeichenbefehle. Du arbeitest in
World-Koordinaten - die Kamera bestimmt, welcher Welt-Ausschnitt auf
dem Bildschirm landet.

Built-ins:
    CAMERA_SET(x, y[, zoom])     ' x,y = Welt-Punkt fuer obere linke Ecke
    CAMERA_RESET()                ' wieder Identitaet (0, 0, 1.0)
    CAMERA_X()  -> FLOAT          ' aktuelle Camera-Position
    CAMERA_Y()  -> FLOAT
    CAMERA_ZOOM() -> FLOAT
    CAMERA_FOLLOW(target_x, target_y, screen_w, screen_h)
                                  ' zentriert Camera auf target
    CAMERA_S2W_X(screen_x) -> FLOAT   ' Screen-Pixel -> Welt-Koordinate
    CAMERA_S2W_Y(screen_y) -> FLOAT

Hinweise:
- TEXT() wird nur translatiert, nicht gezoomt (bleibt scharf). Fuer HUD
  vorher CAMERA_RESET() aufrufen.
- Bilder werden bei zoom != 1 jeden Frame neu skaliert; das kostet
  Performance bei vielen Bildern.
- Kamera ist global pro Graphics-Kontext, nicht pro Bildebene.
"""
from __future__ import annotations

from ..builtins_registry import graphics_builtin
from ..errors import GBRuntimeError, TypeMismatchError


def _check_num(v, fn: str, name: str) -> float:
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn}: {name} muss Zahl sein")
    return float(v)


@graphics_builtin("CAMERA_SET", arity=(2, 3))
def _set(g, *args):
    x = _check_num(args[0], "CAMERA_SET", "x")
    y = _check_num(args[1], "CAMERA_SET", "y")
    zoom = 1.0
    if len(args) == 3:
        zoom = _check_num(args[2], "CAMERA_SET", "zoom")
        if zoom <= 0:
            raise GBRuntimeError("CAMERA_SET: zoom muss > 0 sein")
    g.set_camera(x, y, zoom)
    return None


@graphics_builtin("CAMERA_RESET", arity=0)
def _reset(g):
    g.reset_camera()
    return None


@graphics_builtin("CAMERA_X", arity=0)
def _x(g):
    return g.get_camera()[0]


@graphics_builtin("CAMERA_Y", arity=0)
def _y(g):
    return g.get_camera()[1]


@graphics_builtin("CAMERA_ZOOM", arity=0)
def _zoom(g):
    return g.get_camera()[2]


@graphics_builtin("CAMERA_FOLLOW", arity=4)
def _follow(g, target_x, target_y, screen_w, screen_h):
    """Setzt die Camera so, dass (target_x, target_y) im Bildschirm-Zentrum
    landet. screen_w und screen_h sind die LOGISCHE Aufloesung (was an
    SCREEN uebergeben wurde)."""
    tx = _check_num(target_x, "CAMERA_FOLLOW", "target_x")
    ty = _check_num(target_y, "CAMERA_FOLLOW", "target_y")
    sw = _check_num(screen_w, "CAMERA_FOLLOW", "screen_w")
    sh = _check_num(screen_h, "CAMERA_FOLLOW", "screen_h")
    _, _, zoom = g.get_camera()
    # halbe Bildschirm-Welt-Breite ist (sw / zoom) / 2
    g.set_camera(tx - (sw / zoom) / 2, ty - (sh / zoom) / 2, zoom)
    return None


@graphics_builtin("CAMERA_S2W_X", arity=1)
def _s2w_x(g, sx):
    sx = _check_num(sx, "CAMERA_S2W_X", "sx")
    wx, _ = g._s2w(sx, 0)
    return wx


@graphics_builtin("CAMERA_S2W_Y", arity=1)
def _s2w_y(g, sy):
    sy = _check_num(sy, "CAMERA_S2W_Y", "sy")
    _, wy = g._s2w(0, sy)
    return wy
