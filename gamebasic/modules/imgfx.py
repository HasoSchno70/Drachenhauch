"""Bild-Effekte fuer GameBasic.

Built-ins (alle geben ein NEUES Bild zurueck, das Original bleibt erhalten):

    IMAGE_SCALE(img, w, h)            -> IMAGE   ' neue Groesse
    IMAGE_ROTATE(img, grad)           -> IMAGE   ' Drehung in Grad
    IMAGE_FLIP(img, flipX, flipY)     -> IMAGE   ' Spiegelung
    IMAGE_TINT(img, color)            -> IMAGE   ' RGB-Multiplikation
    IMAGE_COPY(img)                   -> IMAGE   ' tiefer Klon

Fuer den Tint wird der Pixel-Wert mit (R/255, G/255, B/255) multipliziert.
Weiss (0xFFFFFF) laesst das Bild unveraendert; Schwarz (0x000000) macht es
komplett schwarz; Rot (0xFF0000) erhaelt nur den Rot-Kanal.

Pygame muss verfuegbar sein - z.B. nach LOADIMAGE oder SCREEN.
"""
from __future__ import annotations

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from ..interpreter import _Image


def _check_image(v, fn: str) -> _Image:
    if v is None:
        raise GBRuntimeError(f"{fn}: Bild ist NIL")
    if not isinstance(v, _Image):
        raise TypeMismatchError(f"{fn} erwartet IMAGE, erhalten {type(v).__name__}")
    return v


def _check_int(v, fn: str, name: str) -> int:
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn}: {name} muss INTEGER sein")
    return v


def _import_pygame():
    try:
        import pygame
        return pygame
    except ImportError as exc:
        raise GBRuntimeError(
            "imgfx benoetigt pygame. Mit LOADIMAGE oder SCREEN ist es bereits "
            "initialisiert; sonst: 'py -m pip install pygame'."
        ) from exc


@builtin("IMAGE_SCALE", arity=3)
def _scale(img, w, h):
    img = _check_image(img, "IMAGE_SCALE")
    w = _check_int(w, "IMAGE_SCALE", "Breite")
    h = _check_int(h, "IMAGE_SCALE", "Hoehe")
    if w <= 0 or h <= 0:
        raise GBRuntimeError("IMAGE_SCALE: w und h muessen > 0 sein")
    pygame = _import_pygame()
    return _Image(pygame.transform.scale(img.surface, (w, h)),
                  img.path + "@scaled")


@builtin("IMAGE_ROTATE", arity=2, types=("any", "num"))
def _rotate(img, degrees):
    img = _check_image(img, "IMAGE_ROTATE")
    pygame = _import_pygame()
    return _Image(pygame.transform.rotate(img.surface, degrees),
                  img.path + "@rotated")


@builtin("IMAGE_FLIP", arity=3, types=("any", "bool", "bool"))
def _flip(img, flip_x, flip_y):
    img = _check_image(img, "IMAGE_FLIP")
    pygame = _import_pygame()
    return _Image(pygame.transform.flip(img.surface, flip_x, flip_y),
                  img.path + "@flipped")


@builtin("IMAGE_TINT", arity=2, types=("any", "int"))
def _tint(img, color):
    img = _check_image(img, "IMAGE_TINT")
    if color < 0 or color > 0xFFFFFF:
        raise GBRuntimeError("IMAGE_TINT: Farbe muss 0..0xFFFFFF sein")
    pygame = _import_pygame()
    r = (color >> 16) & 0xFF
    g = (color >> 8) & 0xFF
    b = color & 0xFF
    new_surf = img.surface.copy()
    # BLEND_RGBA_MULT multipliziert Pixel kanalweise mit dem Fuell-Wert.
    # Alpha 255 laesst den Alpha-Kanal des Originals unveraendert.
    new_surf.fill((r, g, b, 255), special_flags=pygame.BLEND_RGBA_MULT)
    return _Image(new_surf, img.path + "@tinted")


@builtin("IMAGE_COPY", arity=1)
def _copy(img):
    img = _check_image(img, "IMAGE_COPY")
    return _Image(img.surface.copy(), img.path + "@copy")
