"""Golden-Tests fuer die GUI-Laufzeit-Manipulation (Phase 1): Geometrie,
Lifecycle (Destroy/Visible), Hit-Test, Enumeration, Introspektion.

Konstruktoren/Getter sind reine Builtins (kein SCREEN noetig) -> headless via
run_gb testbar. GUI_UPDATE/GUI_DRAW (grafisch) werden hier NICHT aufgerufen.
"""


def _gb(body: str) -> str:
    return ('IMPORT "gui"\n'
            'DIM win AS GUI_WINDOW\n'
            'win = GUI_WINDOW("T", 100, 80, 300, 200)\n' + body)


def test_kind(run_gb):
    out = run_gb(_gb(
        'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "OK", 10, 10, 80, 24)\n'
        'DIM c AS GUI_WIDGET\nc = GUI_CHECKBOX(win, "x", 10, 50)\n'
        'PRINT GUI_KIND(b)\nPRINT GUI_KIND(c)\n'))
    assert out.splitlines() == ["button", "checkbox"]


def test_bounds_get_set(run_gb):
    out = run_gb(_gb(
        'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "OK", 20, 40, 80, 24)\n'
        'PRINT f"{GUI_GET_X(b)},{GUI_GET_Y(b)},{GUI_GET_W(b)},{GUI_GET_H(b)}"\n'
        'GUI_SET_BOUNDS(b, 50, 60, 120, 30)\n'
        'PRINT f"{GUI_GET_X(b)},{GUI_GET_Y(b)},{GUI_GET_W(b)},{GUI_GET_H(b)}"\n'))
    assert out.splitlines() == ["20,40,80,24", "50,60,120,30"]


def test_window_bounds(run_gb):
    out = run_gb(_gb(
        'PRINT f"{GUI_WINDOW_GET_X(win)},{GUI_WINDOW_GET_Y(win)},{GUI_WINDOW_GET_W(win)},{GUI_WINDOW_GET_H(win)}"\n'
        'GUI_WINDOW_SET_BOUNDS(win, 0, 0, 640, 480)\n'
        'PRINT f"{GUI_WINDOW_GET_W(win)},{GUI_WINDOW_GET_H(win)}"\n'))
    assert out.splitlines() == ["100,80,300,200", "640,480"]


def test_destroy_and_count(run_gb):
    out = run_gb(_gb(
        'DIM a AS GUI_WIDGET\nDIM b AS GUI_WIDGET\n'
        'a = GUI_BUTTON(win, "A", 10, 10, 60, 24)\n'
        'b = GUI_BUTTON(win, "B", 10, 50, 60, 24)\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(win)\n'
        'GUI_DESTROY(a)\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(win)\n'))
    assert out.splitlines() == ["2", "1"]


def test_enumerate_skips_destroyed(run_gb):
    # window_widget liefert lebende Widgets in Reihenfolge; nach Destroy von a
    # bleibt b an Index 0 der Enumeration -- Handle stabil (kein Shift).
    out = run_gb(_gb(
        'DIM a AS GUI_WIDGET\nDIM b AS GUI_WIDGET\n'
        'a = GUI_BUTTON(win, "A", 10, 10, 60, 24)\n'
        'b = GUI_LABEL(win, "B", 10, 50)\n'
        'GUI_DESTROY(a)\n'
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(win, 0))\n'
        'PRINT GUI_WINDOW_WIDGET(win, 0) = b\n'
        'PRINT GUI_WINDOW_WIDGET(win, 1)\n'))   # -1 (kein zweites lebendes)
    assert out.splitlines() == ["label", "TRUE", "-1"]


def test_visible_toggle(run_gb):
    out = run_gb(_gb(
        'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "OK", 10, 10, 60, 24)\n'
        'PRINT GUI_VISIBLE(b)\n'
        'GUI_SET_VISIBLE(b, FALSE)\n'
        'PRINT GUI_VISIBLE(b)\n'))
    assert out.splitlines() == ["TRUE", "FALSE"]


def test_hit_test(run_gb):
    # Button fenster-relativ (50,60) 120x30; Fenster (100,80), title_h=22.
    # abs-Rect: x[150,270), y[162,192).
    out = run_gb(_gb(
        'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "OK", 50, 60, 120, 30)\n'
        'PRINT GUI_HIT_TEST(160, 175) = b\n'   # innerhalb
        'PRINT GUI_HIT_TEST(5, 5)\n'           # ausserhalb -> -1
        'GUI_DESTROY(b)\n'
        'PRINT GUI_HIT_TEST(160, 175)\n'))     # zerstoert -> -1
    assert out.splitlines() == ["TRUE", "-1", "-1"]


def test_hit_test_invisible_skipped(run_gb):
    out = run_gb(_gb(
        'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "OK", 50, 60, 120, 30)\n'
        'GUI_SET_VISIBLE(b, FALSE)\n'
        'PRINT GUI_HIT_TEST(160, 175)\n'))     # unsichtbar -> -1
    assert out.strip() == "-1"


def test_window_destroy(run_gb):
    out = run_gb(_gb(
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_WINDOW("T2", 0, 0, 100, 100)\n'
        'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(w2, "X", 10, 10, 60, 24)\n'
        'GUI_WINDOW_DESTROY(w2)\n'
        # Hit-Test im zerstoerten Fenster -> -1 (Fenster aus z_order entfernt)
        'PRINT GUI_HIT_TEST(20, 40)\n'))
    assert out.strip() == "-1"
