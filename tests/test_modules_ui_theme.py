"""Tests fuer das Theming des ui-Moduls: UI_THEME_SET/GET, UI_METRIC_SET/GET,
UI_THEME_PRESET und Reset via UI_RESET. Headless mit FakeGraphics.
"""
import pytest

from gamebasic.modules import load_module
from gamebasic.modules import ui as ui_mod
from gamebasic.errors import GBRuntimeError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("ui")


class FakeGraphics:
    def __init__(self):
        self._mx = -999
        self._my = -999
        self._mb = [False, False, False]
        self._typed = ""
        self._keys = set()
        self._wheel = 0
        self.draw_calls = []

    def mouse_x(self): return self._mx
    def mouse_y(self): return self._my
    def mouse_button(self, n): return self._mb[n]
    def pop_text_input(self):
        s = self._typed
        self._typed = ""
        return s
    def keys_pressed(self): return set(self._keys)
    def pop_mouse_wheel(self):
        v = self._wheel
        self._wheel = 0
        return v
    def push_clip(self, *a): self.draw_calls.append(("push_clip", a))
    def pop_clip(self): self.draw_calls.append(("pop_clip", ()))
    def box(self, *a):  self.draw_calls.append(("box", a))
    def rect(self, *a): self.draw_calls.append(("rect", a))
    def text(self, *a): self.draw_calls.append(("text", a))
    def circle(self, *a): self.draw_calls.append(("circle", a))
    def line(self, *a): self.draw_calls.append(("line", a))


@pytest.fixture(autouse=True)
def reset_state(g_app):
    gr("ui_reset", g_app)
    yield
    gr("ui_reset", g_app)


@pytest.fixture
def g():
    return FakeGraphics()


# Eigene Graphics-Instanz nur fuer den Reset-Fixture (UI_RESET braucht ein g).
@pytest.fixture
def g_app():
    return FakeGraphics()


def gr(name, gg, *args):
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    return GRAPHICS_BUILTINS[name.lower()](gg, list(args))


def _box_colors(gg):
    return [c[1][4] for c in gg.draw_calls if c[0] == "box" and len(c[1]) >= 5]


# --- UI_THEME_SET / GET ---------------------------------------------

def test_ui_theme_get_default(g):
    assert gr("ui_theme_get", g, "accent") == 0x80C0FF


def test_ui_theme_set_button_bg_used(g):
    gr("ui_theme_set", g, "button_bg", 0x010203)
    g._mx, g._my = -999, -999
    gr("ui_button", g, "b", 0, 0, 80, 28, "OK")   # ohne bg-Arg -> Theme-Default
    assert 0x010203 in _box_colors(g)


def test_ui_theme_explicit_arg_overrides_theme(g):
    gr("ui_theme_set", g, "button_bg", 0x010203)
    gr("ui_button", g, "b", 0, 0, 80, 28, "OK", 0xAABBCC)  # explizit
    cols = _box_colors(g)
    assert 0xAABBCC in cols
    assert 0x010203 not in cols


def test_ui_theme_set_invalid_key(g):
    with pytest.raises(GBRuntimeError, match="unbekannter Schluessel"):
        gr("ui_theme_set", g, "nope", 0x123456)


# --- UI_METRIC_SET / GET --------------------------------------------

def test_ui_metric_get_default(g):
    assert gr("ui_metric_get", g, "checkbox_size") == 14


def test_ui_metric_set_checkbox_size(g):
    gr("ui_metric_set", g, "checkbox_size", 22)
    assert gr("ui_metric_get", g, "checkbox_size") == 22
    # Checkbox zeichnet jetzt mit groesserer Box (rect bei x..x+21)
    g._mx, g._my = -999, -999
    gr("ui_checkbox", g, "c", 10, 10, "An", False)
    rects = [c for c in g.draw_calls if c[0] == "rect"]
    assert any(r[1][2] == 10 + 22 - 1 for r in rects)   # x2 == x + size - 1


def test_ui_metric_set_negative_rejected(g):
    with pytest.raises(GBRuntimeError, match=">= 1"):
        gr("ui_metric_set", g, "slider_h", 0)


# --- UI_THEME_PRESET ------------------------------------------------

def test_ui_preset_light(g):
    gr("ui_theme_preset", g, "light")
    assert ui_mod.THEME["text_fg"] == 0x202428
    assert ui_mod.THEME["accent"] == 0x2A7DE1


def test_ui_preset_unknown(g):
    with pytest.raises(GBRuntimeError, match="unbekanntes Preset"):
        gr("ui_theme_preset", g, "neon")


def test_ui_reset_restores_theme_and_metrics(g):
    gr("ui_theme_preset", g, "retro")
    gr("ui_metric_set", g, "checkbox_size", 30)
    gr("ui_reset", g)
    assert gr("ui_theme_get", g, "accent") == 0x80C0FF
    assert gr("ui_metric_get", g, "checkbox_size") == 14
