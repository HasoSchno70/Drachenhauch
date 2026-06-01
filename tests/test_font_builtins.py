"""Tests fuer die TTF-Font-Builtins: LOADFONT, SETFONT, TEXT_SPACING.

Die Dispatch-Tests mocken die Graphics-Instanz (wie test_shape_builtins) und
pruefen, dass die Built-ins die richtigen Methoden mit den richtigen Argumenten
aufrufen sowie Typ-/Arity-Validierung greift. Ein Integrationstest treibt die
echte Graphics-Font-Logik headless (SDL-dummy) mit einem System-Font; er wird
uebersprungen, wenn kein TTF gefunden wird.
"""
import os
from pathlib import Path
from unittest.mock import MagicMock

import pytest

from gamebasic.errors import GBRuntimeError, TypeMismatchError


def _call_builtin(name: str, args: list, graphics):
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    fn = GRAPHICS_BUILTINS[name.lower()]
    return fn(graphics, args)


@pytest.fixture
def g():
    return MagicMock()


# --- Dispatch -------------------------------------------------------

def test_loadfont_dispatch_returns_handle(g):
    g.load_font.return_value = 3
    r = _call_builtin("LOADFONT", ["font.ttf", 32], g)
    g.load_font.assert_called_once_with("font.ttf", 32)
    assert r == 3


def test_loadfont_intish_size_converted(g):
    """intish-Spec konvertiert FLOAT-Groessen zu INTEGER."""
    _call_builtin("LOADFONT", ["font.ttf", 32.0], g)
    g.load_font.assert_called_once_with("font.ttf", 32)


def test_setfont_dispatch(g):
    _call_builtin("SETFONT", [2], g)
    g.set_font.assert_called_once_with(2)


def test_setfont_reset_minus_one(g):
    _call_builtin("SETFONT", [-1], g)
    g.set_font.assert_called_once_with(-1)


def test_text_spacing_dispatch(g):
    _call_builtin("TEXT_SPACING", [6], g)
    g.text_spacing.assert_called_once_with(6)


# --- Validierung ----------------------------------------------------

def test_loadfont_requires_string_path(g):
    with pytest.raises(TypeMismatchError):
        _call_builtin("LOADFONT", [123, 32], g)


def test_setfont_requires_strict_int(g):
    with pytest.raises(TypeMismatchError):
        _call_builtin("SETFONT", ["x"], g)


def test_setfont_rejects_float(g):
    # "int"-Spec ist strikt -> Float wird abgelehnt.
    with pytest.raises(TypeMismatchError):
        _call_builtin("SETFONT", [1.0], g)


def test_loadfont_wrong_arity(g):
    with pytest.raises(GBRuntimeError):
        _call_builtin("LOADFONT", ["font.ttf"], g)


# --- Integration (echter Font, headless) ----------------------------

_FONT_CANDIDATES = [
    "C:/Windows/Fonts/arial.ttf",
    "C:/Windows/Fonts/segoeui.ttf",
    "/Library/Fonts/Arial.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
]


def _find_font():
    for p in _FONT_CANDIDATES:
        if Path(p).exists():
            return p
    return None


@pytest.fixture
def headless_graphics():
    os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
    os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
    os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "hide")
    from gamebasic.graphics import Graphics
    gfx = Graphics()
    gfx.screen(200, 100, "test")
    yield gfx
    gfx.shutdown()


def test_font_roundtrip_headless(headless_graphics):
    font_path = _find_font()
    if font_path is None:
        pytest.skip("kein System-TTF gefunden")
    gfx = headless_graphics

    # Default-Breite messen.
    gfx.text_size(20)
    default_w = gfx.text_width("Hallo")

    # TTF laden + aktivieren.
    h = gfx.load_font(font_path, 32)
    assert h == 0
    gfx.set_font(h)
    ttf_w = gfx.text_width("Hallo")
    assert ttf_w > 0

    # TEXT_SIZE skaliert die aktive Schrift -> groesser = breiter.
    gfx.text_size(40)
    assert gfx.text_width("Hallo") > ttf_w

    # Zuruecksetzen auf Default.
    gfx.set_font(-1)
    gfx.text_size(20)
    assert gfx.text_width("Hallo") == default_w


def test_load_font_bad_path_raises(headless_graphics):
    with pytest.raises(GBRuntimeError):
        headless_graphics.load_font("does/not/exist.ttf", 24)


def test_set_font_invalid_handle_raises(headless_graphics):
    with pytest.raises(GBRuntimeError):
        headless_graphics.set_font(99)


def test_load_font_size_bounds(headless_graphics):
    with pytest.raises(GBRuntimeError):
        headless_graphics.load_font("x.ttf", 2)   # < 4
