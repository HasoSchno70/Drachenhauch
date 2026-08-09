"""Golden-Tests fuer die GUI-Formular-Widgets (Phase 3): ProgressBar, Radio
(Gruppe/Mutual-Exclusion), Dropdown. Headless via run_gb (State/Getter/Setter +
Serialisierung). Klick-Interaktion (Popup) braucht SCREEN -> manuell verifiziert.
"""
import pytest

from drachenhauch.errors import DrachenhauchError

_W = ('IMPORT "gui"\n'
      'DIM win AS GUI_WINDOW\nwin = GUI_WINDOW("F", 0, 0, 300, 260)\n')


# ------------------------------------------------------------- ProgressBar
def test_progress_value(run_gb):
    out = run_gb(_W +
        'DIM p AS GUI_WIDGET\np = GUI_PROGRESS(win, 10, 10, 200, 18)\n'
        'GUI_SET_VALUE(p, 0.65)\n'
        'PRINT GUI_VALUE(p)\nPRINT GUI_KIND(p)\n')
    assert out.splitlines() == ["0.65", "progress"]


def test_progress_value_clamped(run_gb):
    out = run_gb(_W +
        'DIM p AS GUI_WIDGET\np = GUI_PROGRESS(win, 10, 10, 200, 18)\n'
        'GUI_SET_VALUE(p, 5.0)\nPRINT GUI_VALUE(p)\n')   # auf max=1 geklemmt
    assert out.strip() == "1.0"


# ------------------------------------------------------------- Radio
def test_radio_mutual_exclusion(run_gb):
    out = run_gb(_W +
        'DIM r1 AS GUI_WIDGET\nDIM r2 AS GUI_WIDGET\nDIM r3 AS GUI_WIDGET\n'
        'r1 = GUI_RADIO(win, "g", "A", 10, 10)\n'
        'r2 = GUI_RADIO(win, "g", "B", 10, 40)\n'
        'r3 = GUI_RADIO(win, "g", "C", 10, 70)\n'
        'GUI_SET_CHECKED(r2, TRUE)\n'
        'PRINT f"{GUI_CHECKED(r1)} {GUI_CHECKED(r2)} {GUI_CHECKED(r3)}"\n'
        'PRINT GUI_RADIO_SELECTED(r1)\n'
        'GUI_SET_CHECKED(r3, TRUE)\n'          # r2 muss abgewaehlt werden
        'PRINT f"{GUI_CHECKED(r2)} {GUI_CHECKED(r3)}"\n'
        'PRINT GUI_RADIO_SELECTED(r1)\n')
    assert out.splitlines() == ["FALSE TRUE FALSE", "1", "FALSE TRUE", "2"]


def test_radio_groups_are_independent(run_gb):
    out = run_gb(_W +
        'DIM a1 AS GUI_WIDGET\nDIM b1 AS GUI_WIDGET\n'
        'a1 = GUI_RADIO(win, "ga", "A1", 10, 10)\n'
        'b1 = GUI_RADIO(win, "gb", "B1", 10, 40)\n'
        'GUI_SET_CHECKED(a1, TRUE)\nGUI_SET_CHECKED(b1, TRUE)\n'
        # beide bleiben gewaehlt -- verschiedene Gruppen
        'PRINT f"{GUI_CHECKED(a1)} {GUI_CHECKED(b1)}"\n')
    assert out.strip() == "TRUE TRUE"


def test_radio_selected_none(run_gb):
    out = run_gb(_W +
        'DIM r AS GUI_WIDGET\nr = GUI_RADIO(win, "g", "A", 10, 10)\n'
        'PRINT GUI_RADIO_SELECTED(r)\n')   # nichts gewaehlt -> -1
    assert out.strip() == "-1"


# ------------------------------------------------------------- Dropdown
def test_dropdown_select(run_gb):
    out = run_gb(_W +
        'DIM it[3] AS STRING\nit[0]="Rot" : it[1]="Gruen" : it[2]="Blau"\n'
        'DIM d AS GUI_WIDGET\nd = GUI_DROPDOWN(win, 10, 10, 160, 24, it)\n'
        'PRINT f"{GUI_DROPDOWN_SELECTED(d)} {GUI_DROPDOWN_TEXT(d)}"\n'
        'GUI_DROPDOWN_SET_SELECTED(d, 2)\n'
        'PRINT f"{GUI_DROPDOWN_SELECTED(d)} {GUI_DROPDOWN_TEXT(d)}"\n')
    assert out.splitlines() == ["0 Rot", "2 Blau"]


def test_dropdown_set_items(run_gb):
    out = run_gb(_W +
        'DIM it[2] AS STRING\nit[0]="x" : it[1]="y"\n'
        'DIM d AS GUI_WIDGET\nd = GUI_DROPDOWN(win, 10, 10, 160, 24, it)\n'
        'DIM it2[3] AS STRING\nit2[0]="a" : it2[1]="b" : it2[2]="c"\n'
        'GUI_SET_DROPDOWN(d, it2)\n'
        'GUI_DROPDOWN_SET_SELECTED(d, 2)\n'
        'PRINT GUI_DROPDOWN_TEXT(d)\n')
    assert out.strip() == "c"


# ------------------------------------------------------------- Serialisierung
def test_form_widgets_roundtrip(run_gb):
    out = run_gb(_W +
        'DIM p AS GUI_WIDGET\np = GUI_PROGRESS(win, 10, 10, 200, 18)\nGUI_SET_VALUE(p, 0.4)\n'
        'DIM r AS GUI_WIDGET\nr = GUI_RADIO(win, "g", "A", 10, 40)\nGUI_SET_CHECKED(r, TRUE)\n'
        'DIM it[2] AS STRING\nit[0]="x" : it[1]="y"\n'
        'DIM d AS GUI_WIDGET\nd = GUI_DROPDOWN(win, 10, 70, 160, 24, it)\nGUI_DROPDOWN_SET_SELECTED(d, 1)\n'
        'DIM w2 AS GUI_WINDOW\nw2 = GUI_FROM_JSON(GUI_TO_JSON(win))\n'
        'PRINT GUI_VALUE(GUI_WINDOW_WIDGET(w2, 0))\n'
        'PRINT GUI_CHECKED(GUI_WINDOW_WIDGET(w2, 1))\n'
        'PRINT GUI_DROPDOWN_TEXT(GUI_WINDOW_WIDGET(w2, 2))\n')
    assert out.splitlines() == ["0.4", "TRUE", "y"]


# ------------------------------------------------------------- Fehler
def test_wrong_kind_raises(run_gb):
    with pytest.raises(DrachenhauchError, match="kein dropdown"):
        run_gb(_W +
            'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(win, "x", 0, 0, 50, 20)\n'
            'PRINT GUI_DROPDOWN_SELECTED(b)\n')
