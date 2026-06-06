"""Golden-Tests fuer die GUI-Serialisierung (Phase 2): GUI_TO_JSON/FROM_JSON
(String) + GUI_SAVE/LOAD (Datei). Headless via run_gb (kein SCREEN noetig)."""
import pytest

from gamebasic.errors import GameBasicError


_BUILD = (
    'IMPORT "gui"\n'
    'DIM win AS GUI_WINDOW\n'
    'win = GUI_WINDOW("Login", 100, 80, 300, 200)\n'
    'GUI_WINDOW_CLOSABLE(win, TRUE)\n'
    'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "Anmelden", 20, 120, 120, 30)\n'
    'DIM c AS GUI_WIDGET\nc = GUI_CHECKBOX(win, "Merken", 20, 80)\n'
    'GUI_SET_CHECKED(c, TRUE)\n'
    'DIM s AS GUI_WIDGET\ns = GUI_SLIDER(win, 20, 160, 200, 0, 100)\n'
    'GUI_SET_VALUE(s, 42)\n'
)


def test_string_roundtrip_structure(run_gb):
    out = run_gb(_BUILD +
        'DIM js AS STRING\njs = GUI_TO_JSON(win)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(js)\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(w2)\n'
        'PRINT f"{GUI_WINDOW_GET_X(w2)},{GUI_WINDOW_GET_W(w2)}"\n'
        'DIM w0 AS GUI_WIDGET\nw0 = GUI_WINDOW_WIDGET(w2, 0)\n'
        'PRINT f"{GUI_KIND(w0)} {GUI_GET_X(w0)},{GUI_GET_Y(w0)},{GUI_GET_W(w0)},{GUI_GET_H(w0)}"\n')
    assert out.splitlines() == ["3", "100,300", "button 20,120,120,30"]


def test_roundtrip_preserves_state(run_gb):
    # Checkbox-Zustand + Slider-Wert muessen erhalten bleiben.
    out = run_gb(_BUILD +
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_CHECKED(GUI_WINDOW_WIDGET(w2, 1))\n'
        'PRINT GUI_VALUE(GUI_WINDOW_WIDGET(w2, 2))\n')
    assert out.splitlines() == ["TRUE", "42.0"]


def test_roundtrip_skips_destroyed(run_gb):
    # Zerstoerte Widgets landen NICHT im JSON.
    out = run_gb(_BUILD +
        'GUI_DESTROY(b)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(w2)\n'              # 2 statt 3
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(w2, 0))\n')      # checkbox (button weg)
    assert out.splitlines() == ["2", "checkbox"]


def test_table_roundtrip(run_gb):
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM win AS GUI_WINDOW\nwin = GUI_WINDOW("T", 0, 0, 300, 200)\n'
        'DIM hdr[2] AS STRING\nhdr[0]="ID" : hdr[1]="Name"\n'
        'DIM cells[2, 2] AS STRING\n'
        'cells[0,0]="1" : cells[0,1]="Anna"\n'
        'cells[1,0]="2" : cells[1,1]="Bert"\n'
        'DIM t AS GUI_WIDGET\n'
        't = GUI_TABLE(win, 10, 10, 280, 150, hdr, cells)\n'
        'GUI_TABLE_SET_SELECTED(t, 1)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'DIM t2 AS GUI_WIDGET\nt2 = GUI_WINDOW_WIDGET(w2, 0)\n'
        'PRINT GUI_KIND(t2)\n'
        'PRINT GUI_TABLE_ROW_COUNT(t2)\n'
        'PRINT GUI_TABLE_SELECTED(t2)\n')
    assert out.splitlines() == ["table", "2", "1"]


def test_file_roundtrip(run_gb, tmp_path):
    out = run_gb(_BUILD +
        'GUI_SAVE(win, "layout.json")\n'
        'DIM w3 AS GUI_WINDOW\nw3 = GUI_LOAD("layout.json")\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(w3)\n'
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(w3, 0))\n',
        base=tmp_path)
    assert out.splitlines() == ["3", "button"]
    assert (tmp_path / "layout.json").exists()


def test_from_json_invalid_raises(run_gb):
    with pytest.raises(GameBasicError, match="ungueltiges JSON"):
        run_gb('IMPORT "gui"\nDIM w AS GUI_WINDOW\nw = GUI_FROM_JSON("{nope")\n')


def test_load_missing_file_raises(run_gb):
    with pytest.raises(GameBasicError, match="GUI_LOAD"):
        run_gb('IMPORT "gui"\nDIM w AS GUI_WINDOW\nw = GUI_LOAD("nope_xyz.json")\n')
