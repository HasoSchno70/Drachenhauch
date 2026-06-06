"""Golden-Tests fuer das GUI-Styling (Phase 4): enabled-Zustand, per-Widget-
Font/-Groesse, corner_radius-Metrik. Headless (State/Serialisierung); die
visuelle Wirkung (Ausgrauen, runde Ecken, Font-Rendering) braucht SCREEN ->
manuell verifiziert.
"""
import pytest

from gamebasic.errors import GameBasicError

_W = ('IMPORT "gui"\n'
      'DIM win AS GUI_WINDOW\nwin = GUI_WINDOW("S", 0, 0, 300, 200)\n'
      'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "OK", 20, 40, 120, 30)\n')


def test_enabled_toggle(run_gb):
    out = run_gb(_W +
        'PRINT GUI_ENABLED(b)\n'
        'GUI_SET_ENABLED(b, FALSE)\n'
        'PRINT GUI_ENABLED(b)\n')
    assert out.splitlines() == ["TRUE", "FALSE"]


def test_disabled_still_hit_testable(run_gb):
    # Fuer den Editor: deaktivierte Widgets bleiben selektierbar (GUI_HIT_TEST),
    # nur die Interaktion (Klick/Hover) ist unterbunden.
    out = run_gb(_W +
        'GUI_SET_ENABLED(b, FALSE)\n'
        'PRINT GUI_HIT_TEST(60, 77) = b\n')   # Mitte des Buttons (abs: y=40+22+15)
    assert out.strip() == "TRUE"


def test_font_setters_and_roundtrip(run_gb):
    out = run_gb(_W +
        'GUI_SET_ENABLED(b, FALSE)\n'
        'GUI_SET_FONT_SIZE(b, 24)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_ENABLED(GUI_WINDOW_WIDGET(w2, 0))\n')
    assert out.strip() == "FALSE"


def test_font_size_negative_raises(run_gb):
    with pytest.raises(GameBasicError, match="GUI_SET_FONT_SIZE"):
        run_gb(_W + 'GUI_SET_FONT_SIZE(b, -5)\n')


def test_corner_radius_metric(run_gb):
    out = run_gb('IMPORT "gui"\n'
        'PRINT GUI_METRIC_GET("corner_radius")\n'   # Default 0
        'GUI_METRIC_SET("corner_radius", 8)\n'
        'PRINT GUI_METRIC_GET("corner_radius")\n')
    assert out.splitlines() == ["0", "8"]


def test_set_font_accepts_handle(run_gb):
    # GUI_SET_FONT akzeptiert ein (auch -1=Default) Handle ohne Fehler.
    out = run_gb(_W + 'GUI_SET_FONT(b, -1)\nPRINT GUI_KIND(b)\n')
    assert out.strip() == "button"
