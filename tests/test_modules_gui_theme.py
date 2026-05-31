"""Tests fuer das Theming des gui-Moduls: GUI_THEME_SET/GET, GUI_METRIC_SET/GET,
GUI_SET_COLOR (Per-Widget-Override) sowie das Zuruecksetzen via GUI_RESET.

Self-contained (eigene FakeGraphics + Helfer), damit unabhaengig vom
Haupt-Testfile.
"""
import pytest

from gamebasic.modules import load_module
from gamebasic.modules import gui as gui_mod
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("gui")


class FakeGraphics:
    def __init__(self):
        self._mx = -999
        self._my = -999
        self._mb = [False, False, False]
        self._typed = ""
        self._keys = set()
        self.draw_calls = []

    def mouse_x(self): return self._mx
    def mouse_y(self): return self._my
    def mouse_button(self, n): return self._mb[n]
    def pop_text_input(self):
        s = self._typed
        self._typed = ""
        return s
    def keys_pressed(self): return set(self._keys)
    def box(self, *a):  self.draw_calls.append(("box", a))
    def rect(self, *a): self.draw_calls.append(("rect", a))
    def text(self, *a): self.draw_calls.append(("text", a))
    def circle(self, *a): self.draw_calls.append(("circle", a))
    def line(self, *a): self.draw_calls.append(("line", a))


@pytest.fixture(autouse=True)
def reset_state():
    gui_mod._mgr.reset()
    yield
    gui_mod._mgr.reset()


@pytest.fixture
def g():
    return FakeGraphics()


def b(name, *args):
    from gamebasic.interpreter import BUILTINS
    return BUILTINS[name.lower()](list(args))


def gr(name, gg, *args):
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    return GRAPHICS_BUILTINS[name.lower()](gg, list(args))


def _box_colors(gg):
    return [c[1][4] for c in gg.draw_calls if c[0] == "box" and len(c[1]) >= 5]


# --- GUI_THEME_SET / GET --------------------------------------------

def test_theme_get_default():
    assert b("gui_theme_get", "accent") == 0x2BC4E8


def test_theme_set_changes_palette():
    b("gui_theme_set", "accent", 0xFF8800)
    assert b("gui_theme_get", "accent") == 0xFF8800
    assert gui_mod.THEME["accent"] == 0xFF8800


def test_theme_set_win_bg_used_in_draw(g):
    b("gui_theme_set", "win_bg", 0x010203)
    b("gui_window", "T", 10, 10, 200, 150)
    gr("gui_draw", g)
    assert 0x010203 in _box_colors(g)      # Fenster-Korpus nutzt win_bg


def test_theme_set_invalid_key():
    with pytest.raises(GBRuntimeError, match="unbekannter Schluessel"):
        b("gui_theme_set", "gibtsnicht", 0x123456)


def test_theme_set_invalid_color():
    with pytest.raises(GBRuntimeError, match="Farbe"):
        b("gui_theme_set", "accent", 0x1000000)   # > 0xFFFFFF


def test_theme_short_accent_still_works():
    b("gui_theme", 0x123456)
    assert b("gui_theme_get", "accent") == 0x123456


# --- GUI_METRIC_SET / GET -------------------------------------------

def test_metric_get_default():
    assert b("gui_metric_get", "title_h") == 22
    assert b("gui_metric_get", "check_size") == 16


def test_metric_set_title_h_shifts_abs_rect():
    b("gui_metric_set", "title_h", 40)
    win = b("gui_window", "T", 100, 100, 200, 150)
    btn = b("gui_button", win, "OK", 10, 10, 80, 30)
    # abs_y = win.y + title_h + widget.y = 100 + 40 + 10 = 150
    assert btn.abs_rect()[1] == 150


def test_metric_set_check_size_affects_new_checkbox():
    b("gui_metric_set", "check_size", 24)
    win = b("gui_window", "T", 0, 0, 200, 200)
    chk = b("gui_checkbox", win, "An", 10, 10, False)
    assert chk.w == 24 and chk.h == 24


def test_metric_set_invalid_key():
    with pytest.raises(GBRuntimeError, match="unbekannter Schluessel"):
        b("gui_metric_set", "nope", 5)


def test_metric_set_negative_rejected():
    with pytest.raises(GBRuntimeError, match=">= 0"):
        b("gui_metric_set", "pad", -3)


# --- GUI_SET_COLOR (Per-Widget) -------------------------------------

def test_set_color_button_bg(g):
    win = b("gui_window", "T", 0, 0, 200, 200)
    btn = b("gui_button", win, "OK", 0, 0, 80, 30)
    b("gui_set_color", btn, "bg", 0x445566)
    gr("gui_draw", g)
    assert 0x445566 in _box_colors(g)


def test_set_color_only_affects_target(g):
    win = b("gui_window", "T", 0, 0, 200, 200)
    b1 = b("gui_button", win, "A", 0, 0, 80, 30)
    b2 = b("gui_button", win, "B", 0, 60, 80, 30)
    b("gui_set_color", b1, "bg", 0x445566)
    gr("gui_draw", g)
    cols = _box_colors(g)
    assert 0x445566 in cols                 # b1 override
    assert gui_mod.THEME["widget_bg"] in cols  # b2 weiter Theme-Default


def test_set_color_invalid_role():
    win = b("gui_window", "T", 0, 0, 200, 200)
    btn = b("gui_button", win, "OK", 0, 0, 80, 30)
    with pytest.raises(GBRuntimeError, match="role"):
        b("gui_set_color", btn, "hintergrund", 0x112233)


def test_set_color_clear_with_minus1(g):
    win = b("gui_window", "T", 0, 0, 200, 200)
    btn = b("gui_button", win, "OK", 0, 0, 80, 30)
    b("gui_set_color", btn, "bg", 0x445566)
    b("gui_set_color", btn, "bg", -1)        # Override entfernen
    gr("gui_draw", g)
    cols = _box_colors(g)
    assert 0x445566 not in cols
    assert gui_mod.THEME["widget_bg"] in cols


def test_set_color_label_fg_override(g):
    win = b("gui_window", "T", 0, 0, 200, 200)
    lbl = b("gui_label", win, "Hi", 5, 5, 0x00FF00)
    b("gui_set_color", lbl, "fg", 0xFF0000)   # Override schlaegt GUI_LABEL-Farbe
    gr("gui_draw", g)
    text_colors = [c[1][3] for c in g.draw_calls if c[0] == "text" and len(c[1]) >= 4]
    assert 0xFF0000 in text_colors
    assert 0x00FF00 not in text_colors


# --- GUI_RESET stellt Theme + Metriken wieder her -------------------

def test_reset_restores_theme_and_metrics():
    b("gui_theme_set", "accent", 0xABCDEF)
    b("gui_metric_set", "title_h", 50)
    b("gui_reset")
    assert b("gui_theme_get", "accent") == 0x2BC4E8
    assert b("gui_metric_get", "title_h") == 22


# --- Presets --------------------------------------------------------

def test_gui_preset_light_changes_palette():
    b("gui_theme_preset", "light")
    assert gui_mod.THEME["win_bg"] == 0xF4F6F9
    assert gui_mod.THEME["accent"] == 0x2A7DE1


def test_gui_preset_unknown_rejected():
    with pytest.raises(GBRuntimeError, match="unbekanntes Preset"):
        b("gui_theme_preset", "neon")


def test_gui_preset_reset_back_to_dark():
    b("gui_theme_preset", "retro")
    b("gui_reset")
    assert gui_mod.THEME["accent"] == 0x2BC4E8     # Default-Cyan wieder da
