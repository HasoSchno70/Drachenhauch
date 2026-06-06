"""Golden-Tests fuer GUI Phase 4b: benannte Styles (Stylesheet), ListBox,
Image, Canvas. Headless via run_gb (State/Serialisierung/Geometrie); die
visuelle Wirkung + Klick/Scroll brauchen SCREEN -> manuell verifiziert.
"""
import pytest

from gamebasic.errors import GameBasicError

_W = ('IMPORT "gui"\n'
      'DIM win AS GUI_WINDOW\nwin = GUI_WINDOW("W", 50, 40, 320, 260)\n')


# ------------------------------------------------------------- Named styles
def test_apply_style(run_gb):
    out = run_gb(_W +
        'GUI_STYLE_SET("primary", "bg", 100)\n'
        'GUI_STYLE_SET("primary", "font_size", 18)\n'
        'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "OK", 10, 10, 80, 24)\n'
        'GUI_APPLY_STYLE(b, "primary")\n'
        'PRINT GUI_KIND(b)\n')   # kein Crash; ov/font_size landen am Widget
    assert out.strip() == "button"


def test_style_unknown_raises(run_gb):
    with pytest.raises(GameBasicError, match="unbekannter Style"):
        run_gb(_W +
            'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "OK", 0, 0, 50, 20)\n'
            'GUI_APPLY_STYLE(b, "nope")\n')


def test_style_bad_prop_raises(run_gb):
    with pytest.raises(GameBasicError, match="GUI_STYLE_SET"):
        run_gb(_W + 'GUI_STYLE_SET("s", "blarg", 1)\n')


# ------------------------------------------------------------- ListBox
def test_listbox_select(run_gb):
    out = run_gb(_W +
        'DIM it[3] AS STRING\nit[0]="A" : it[1]="B" : it[2]="C"\n'
        'DIM lb AS GUI_WIDGET\nlb = GUI_LISTBOX(win, 10, 30, 140, 88, it)\n'
        'PRINT GUI_LISTBOX_SELECTED(lb)\n'          # -1 (nichts gewaehlt)
        'GUI_LISTBOX_SET_SELECTED(lb, 2)\n'
        'PRINT f"{GUI_LISTBOX_SELECTED(lb)} {GUI_LISTBOX_TEXT(lb)}"\n')
    assert out.splitlines() == ["-1", "2 C"]


def test_listbox_set_items(run_gb):
    out = run_gb(_W +
        'DIM it[1] AS STRING\nit[0]="x"\n'
        'DIM lb AS GUI_WIDGET\nlb = GUI_LISTBOX(win, 10, 30, 140, 88, it)\n'
        'DIM it2[2] AS STRING\nit2[0]="p" : it2[1]="q"\n'
        'GUI_SET_LISTBOX(lb, it2)\nGUI_LISTBOX_SET_SELECTED(lb, 1)\n'
        'PRINT GUI_LISTBOX_TEXT(lb)\n')
    assert out.strip() == "q"


def test_listbox_roundtrip(run_gb):
    out = run_gb(_W +
        'DIM it[3] AS STRING\nit[0]="A" : it[1]="B" : it[2]="C"\n'
        'DIM lb AS GUI_WIDGET\nlb = GUI_LISTBOX(win, 10, 30, 140, 88, it)\n'
        'GUI_LISTBOX_SET_SELECTED(lb, 1)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_LISTBOX_TEXT(GUI_WINDOW_WIDGET(w2, 0))\n')
    assert out.strip() == "B"


# ------------------------------------------------------------- Image
def test_image_kind_and_roundtrip(run_gb):
    # Textur-Handle wird gespeichert (ohne SCREEN nicht gezeichnet).
    out = run_gb(_W +
        'DIM im AS GUI_WIDGET\nim = GUI_IMAGE(win, 10, 10, 64, 64, 3)\n'
        'PRINT GUI_KIND(im)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(w2, 0))\n')
    assert out.splitlines() == ["image", "image"]


# ------------------------------------------------------------- Canvas
def test_canvas_rect(run_gb):
    # Fenster (50,40), title_h=22, Canvas-Widget (10,130) 200x100.
    # abs = (60,192,200,100); Inhalt (Rahmen-inset) = (61,193,198,98).
    out = run_gb(_W +
        'DIM cv AS GUI_WIDGET\ncv = GUI_CANVAS(win, 10, 130, 200, 100)\n'
        'PRINT f"{GUI_CANVAS_X(cv)},{GUI_CANVAS_Y(cv)},{GUI_CANVAS_W(cv)},{GUI_CANVAS_H(cv)}"\n')
    assert out.strip() == "61,193,198,98"


def test_canvas_rect_follows_window(run_gb):
    # Nach Fenster-Verschiebung liefert GUI_CANVAS_* aktualisierte Koordinaten.
    out = run_gb(_W +
        'DIM cv AS GUI_WIDGET\ncv = GUI_CANVAS(win, 10, 130, 200, 100)\n'
        'GUI_WINDOW_SET_BOUNDS(win, 0, 0, 320, 260)\n'
        'PRINT f"{GUI_CANVAS_X(cv)},{GUI_CANVAS_Y(cv)}"\n')   # (0+10+1, 0+22+130+1)
    assert out.strip() == "11,153"


def test_canvas_wrong_kind_raises(run_gb):
    with pytest.raises(GameBasicError, match="kein canvas"):
        run_gb(_W +
            'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "x", 0, 0, 50, 20)\n'
            'PRINT GUI_CANVAS_X(b)\n')
