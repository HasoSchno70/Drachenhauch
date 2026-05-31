"""Tests fuer das gui-Modul (Retained-Mode-Fenster & Widgets).

Wie beim ui-Modul brauchen GUI_UPDATE/GUI_DRAW einen Graphics-Kontext mit
Maus-/Tastatur-Status. Wir mocken ihn mit FakeGraphics. Konstruktoren und
Polling-Getter sind reine Built-ins (kein g). Geprueft werden State und
Hit-Test headless -- die Grafik selbst ist nicht bit-kritisch.
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
        self._mx = 0
        self._my = 0
        self._mb = [False, False, False]
        self._typed = ""
        self._keys: set = set()
        self._wheel = 0          # was pop_mouse_wheel() liefert
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
    def pop_clip(self):      self.draw_calls.append(("pop_clip", ()))
    def box(self, *a):    self.draw_calls.append(("box", a))
    def rect(self, *a):   self.draw_calls.append(("rect", a))
    def text(self, *a):   self.draw_calls.append(("text", a))
    def circle(self, *a): self.draw_calls.append(("circle", a))
    def line(self, *a):   self.draw_calls.append(("line", a))


@pytest.fixture(autouse=True)
def reset_state():
    gui_mod._mgr.reset()
    yield


@pytest.fixture
def g():
    return FakeGraphics()


# --- Aufruf-Helfer ---------------------------------------------------

def b(name, *args):
    from gamebasic.interpreter import BUILTINS
    return BUILTINS[name.lower()](list(args))


def gr(name, g, *args):
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    return GRAPHICS_BUILTINS[name.lower()](g, list(args))


def _press(g, x, y):
    """Maus an (x,y) druecken + UPDATE."""
    g._mx, g._my = x, y
    g._mb[0] = True
    gr("gui_update", g)


def _release(g, x, y):
    g._mx, g._my = x, y
    g._mb[0] = False
    gr("gui_update", g)


# --- Registrierung / Typen ------------------------------------------

def test_external_types_registriert():
    from gamebasic.modules import EXTERNAL_TYPES
    assert EXTERNAL_TYPES.get("gui_window") is gui_mod._Window
    assert EXTERNAL_TYPES.get("gui_widget") is gui_mod._Widget


def test_window_anlegen():
    win = b("gui_window", "Titel", 100, 100, 200, 150)
    assert isinstance(win, gui_mod._Window)
    assert win in gui_mod._mgr.windows
    assert win.title == "Titel"


def test_window_typecheck():
    with pytest.raises(TypeMismatchError, match="titel"):
        b("gui_window", 123, 0, 0, 100, 100)


def test_button_an_fenster():
    win = b("gui_window", "T", 0, 0, 200, 200)
    btn = b("gui_button", win, "OK", 10, 10, 80, 30)
    assert isinstance(btn, gui_mod._Widget)
    assert btn.kind == "button"
    assert btn in win.widgets


def test_button_braucht_window():
    with pytest.raises(TypeMismatchError, match="GUI_WINDOW"):
        b("gui_button", "kein fenster", "OK", 0, 0, 10, 10)


# --- Button-Klick-Edge ----------------------------------------------

def test_button_klick_press_und_release(g):
    win = b("gui_window", "T", 100, 100, 200, 150)
    btn = b("gui_button", win, "OK", 20, 50, 90, 30)
    # abs Button: (120, 172, 90, 30) -> Mitte ~ (165, 187)
    _press(g, 165, 187)
    assert b("gui_clicked", btn) is False     # nur Press
    _release(g, 165, 187)
    assert b("gui_clicked", btn) is True       # Release ueber demselben Button


def test_button_kein_klick_release_woanders(g):
    win = b("gui_window", "T", 100, 100, 300, 150)
    btn = b("gui_button", win, "OK", 20, 50, 90, 30)
    _press(g, 165, 187)
    _release(g, 350, 200)                       # woanders losgelassen
    assert b("gui_clicked", btn) is False


def test_button_klick_nur_einen_frame(g):
    win = b("gui_window", "T", 100, 100, 200, 150)
    btn = b("gui_button", win, "OK", 20, 50, 90, 30)
    _press(g, 165, 187)
    _release(g, 165, 187)
    assert b("gui_clicked", btn) is True
    gr("gui_update", g)                          # naechster Frame
    assert b("gui_clicked", btn) is False


# --- Checkbox -------------------------------------------------------

def test_checkbox_default(g):
    win = b("gui_window", "T", 0, 0, 200, 200)
    chk = b("gui_checkbox", win, "An", 10, 10, True)
    assert b("gui_checked", chk) is True


def test_checkbox_toggle_bei_press(g):
    win = b("gui_window", "T", 100, 100, 200, 200)
    chk = b("gui_checkbox", win, "An", 20, 20, False)
    # abs Box: (120, 142, 16, 16) -> Mitte (128, 150)
    _press(g, 128, 150)
    assert b("gui_checked", chk) is True
    _release(g, 128, 150)
    # erneut druecken toggelt zurueck
    _press(g, 128, 150)
    assert b("gui_checked", chk) is False


def test_checked_nur_auf_checkbox(g):
    win = b("gui_window", "T", 0, 0, 200, 200)
    btn = b("gui_button", win, "OK", 0, 0, 10, 10)
    with pytest.raises(GBRuntimeError, match="checkbox"):
        b("gui_checked", btn)


# --- Slider ---------------------------------------------------------

def test_slider_default_und_clamp(g):
    win = b("gui_window", "T", 0, 0, 300, 200)
    s = b("gui_slider", win, 10, 10, 200, 0.0, 1.0, 5.0)
    assert b("gui_value", s) == 1.0            # auf max geklemmt


def test_slider_press_setzt_wert(g):
    win = b("gui_window", "T", 100, 100, 300, 200)
    s = b("gui_slider", win, 20, 20, 200, 0.0, 1.0)
    # abs links: x = 100+20 = 120 -> value=min
    _press(g, 120, 145)
    assert b("gui_value", s) == 0.0
    # ganz rechts: x = 120 + (200-1) = 319 -> value=max
    g._mx = 319
    gr("gui_update", g)
    assert b("gui_value", s) == 1.0


def test_slider_max_groesser_min():
    win = b("gui_window", "T", 0, 0, 300, 200)
    with pytest.raises(GBRuntimeError, match="max muss > min"):
        b("gui_slider", win, 10, 10, 200, 1.0, 0.5)


# --- Z-Order --------------------------------------------------------

def test_klick_bringt_fenster_nach_vorne(g):
    a = b("gui_window", "A", 0, 0, 100, 100)
    c = b("gui_window", "C", 200, 0, 100, 100)
    assert gui_mod._mgr.windows[-1] is c       # zuletzt angelegt = vorne
    _press(g, 50, 50)                           # Klick in Fenster A
    assert gui_mod._mgr.windows[-1] is a        # A jetzt vorne
    assert gui_mod._mgr.focus_window is a


# --- Window-Drag ----------------------------------------------------

def test_window_drag(g):
    win = b("gui_window", "T", 100, 100, 200, 150)
    # Press auf Titelleiste (y in 100..122)
    _press(g, 150, 110)
    assert gui_mod._mgr.drag_window is win
    # Maus bewegen (weiter gedrueckt)
    g._mx, g._my = 160, 130
    gr("gui_update", g)
    assert win.x == 110 and win.y == 120        # +10 / +20
    # Loslassen beendet Drag
    _release(g, 160, 130)
    assert gui_mod._mgr.drag_window is None


def test_window_nicht_movable(g):
    win = b("gui_window", "T", 100, 100, 200, 150)
    b("gui_window_movable", win, False)
    _press(g, 150, 110)
    assert gui_mod._mgr.drag_window is None


# --- Close-Button ---------------------------------------------------

def test_window_close(g):
    win = b("gui_window", "T", 100, 100, 200, 150)
    b("gui_window_closable", win, True)
    assert b("gui_window_closed", win) is False
    # Close-Button: (100+200-22, 100, 22, 22) = (278,100,22,22)
    _press(g, 285, 110)
    assert b("gui_window_closed", win) is True
    assert win.visible is False


def test_window_visible_reset_close(g):
    win = b("gui_window", "T", 0, 0, 200, 150)
    b("gui_window_closable", win, True)
    win.close_clicked = True
    win.visible = False
    b("gui_window_visible", win, True)
    assert b("gui_window_closed", win) is False  # wieder sichtbar -> Flag weg


# --- TextInput ------------------------------------------------------

def test_textinput_fokus_und_eingabe(g):
    win = b("gui_window", "T", 100, 100, 300, 150)
    tf = b("gui_textinput", win, 20, 20, 160, 24, "tippen")
    # abs: (120, 142, 160, 24) -> Mitte (200, 154)
    _press(g, 200, 154)
    assert gui_mod._mgr.focus_widget is tf
    # Tippen im naechsten Frame
    g._mb[0] = False
    g._typed = "Hallo"
    gr("gui_update", g)
    assert b("gui_text", tf) == "Hallo"


def test_textinput_backspace(g):
    win = b("gui_window", "T", 0, 0, 300, 150)
    tf = b("gui_textinput", win, 20, 20, 160, 24)
    b("gui_set_text", tf, "ABC")
    gui_mod._mgr.focus_widget = tf
    g._keys = {8}                                # Backspace gedrueckt (Edge)
    gr("gui_update", g)
    assert b("gui_text", tf) == "AB"


def test_textinput_klick_ausserhalb_blurred(g):
    win = b("gui_window", "T", 100, 100, 300, 150)
    tf = b("gui_textinput", win, 20, 20, 160, 24)
    _press(g, 200, 154)
    assert gui_mod._mgr.focus_widget is tf
    _release(g, 200, 154)
    _press(g, 500, 500)                          # ins Leere
    assert gui_mod._mgr.focus_widget is None


# --- Label / Panel / Setter -----------------------------------------

def test_label_farbe():
    win = b("gui_window", "T", 0, 0, 200, 200)
    lbl = b("gui_label", win, "Hi", 5, 5, 0xFF0000)
    assert lbl.color == 0xFF0000


def test_set_text():
    win = b("gui_window", "T", 0, 0, 200, 200)
    lbl = b("gui_label", win, "alt", 5, 5)
    b("gui_set_text", lbl, "neu")
    assert b("gui_text", lbl) == "neu"


def test_panel_optional_titel():
    win = b("gui_window", "T", 0, 0, 200, 200)
    p = b("gui_panel", win, 5, 5, 100, 80, "Box")
    assert p.kind == "panel" and p.text == "Box"


# --- Draw / Reset ---------------------------------------------------

def test_draw_zeichnet(g):
    win = b("gui_window", "T", 10, 10, 200, 150)
    b("gui_button", win, "OK", 20, 20, 80, 30)
    gr("gui_draw", g)
    kinds = {c[0] for c in g.draw_calls}
    assert "box" in kinds and "text" in kinds


def test_draw_unsichtbar_nicht(g):
    win = b("gui_window", "T", 10, 10, 200, 150)
    b("gui_window_visible", win, False)
    gr("gui_draw", g)
    assert g.draw_calls == []


def test_reset_loescht_alles():
    b("gui_window", "T", 0, 0, 200, 200)
    b("gui_reset")
    assert gui_mod._mgr.windows == []


# --- GUI_ON_CLICK (FUNCREF-Callbacks, Phase 3) ----------------------

class FakeEngine:
    """Minimaler Engine-Stub: zeichnet gb_call_function-Aufrufe auf."""
    def __init__(self):
        self.calls = []

    def gb_call_function(self, name, args):
        self.calls.append((name, list(args)))
        return None


def _funcref(name):
    from gamebasic.interpreter import _FuncRef
    return _FuncRef(name)


def test_on_click_typecheck():
    win = b("gui_window", "T", 0, 0, 200, 200)
    btn = b("gui_button", win, "OK", 0, 0, 80, 30)
    with pytest.raises(TypeMismatchError, match="FUNCREF"):
        b("gui_on_click", btn, 42)


def test_on_click_button_dispatch(g):
    eng = FakeEngine()
    g._gb_engine = eng
    win = b("gui_window", "T", 100, 100, 200, 150)
    btn = b("gui_button", win, "OK", 20, 50, 90, 30)
    b("gui_on_click", btn, _funcref("handler"))
    # Klick: Press + Release ueber dem Button (abs Mitte ~165,187)
    _press(g, 165, 187)
    assert eng.calls == []                      # noch kein Dispatch (nur Press)
    _release(g, 165, 187)
    assert eng.calls == [("handler", [])]       # Callback genau einmal


def test_on_click_checkbox_dispatch(g):
    eng = FakeEngine()
    g._gb_engine = eng
    win = b("gui_window", "T", 100, 100, 200, 200)
    chk = b("gui_checkbox", win, "An", 20, 20, False)
    b("gui_on_click", chk, _funcref("toggled"))
    _press(g, 128, 150)                          # Toggle on press
    assert eng.calls == [("toggled", [])]


def test_on_click_no_dispatch_without_handler(g):
    eng = FakeEngine()
    g._gb_engine = eng
    win = b("gui_window", "T", 100, 100, 200, 150)
    btn = b("gui_button", win, "OK", 20, 50, 90, 30)
    _press(g, 165, 187)
    _release(g, 165, 187)
    assert eng.calls == []                       # kein Handler gesetzt
    assert b("gui_clicked", btn) is True         # Polling funktioniert weiter


def test_on_click_remove_with_nil(g):
    eng = FakeEngine()
    g._gb_engine = eng
    win = b("gui_window", "T", 100, 100, 200, 150)
    btn = b("gui_button", win, "OK", 20, 50, 90, 30)
    b("gui_on_click", btn, _funcref("h"))
    b("gui_on_click", btn, None)                 # Callback entfernen
    _press(g, 165, 187)
    _release(g, 165, 187)
    assert eng.calls == []


# --- GUI_ON_CHANGE --------------------------------------------------

def test_on_change_slider_dispatch(g):
    eng = FakeEngine()
    g._gb_engine = eng
    win = b("gui_window", "T", 100, 100, 300, 200)
    s = b("gui_slider", win, 20, 20, 200, 0.0, 1.0)
    b("gui_on_change", s, _funcref("moved"))
    # Press auf Slider links -> value 0.0 (== Default, keine Aenderung) ...
    _press(g, 120, 145)
    eng.calls.clear()
    # ... dann nach rechts ziehen -> Wert aendert sich -> Callback
    g._mx = 280
    gr("gui_update", g)
    assert eng.calls == [("moved", [])]


def test_on_change_textinput_dispatch(g):
    eng = FakeEngine()
    g._gb_engine = eng
    win = b("gui_window", "T", 100, 100, 300, 150)
    tf = b("gui_textinput", win, 20, 20, 160, 24)
    b("gui_on_change", tf, _funcref("typed"))
    _press(g, 200, 154)                          # fokussieren
    _release(g, 200, 154)
    eng.calls.clear()
    g._typed = "x"
    gr("gui_update", g)
    assert eng.calls == [("typed", [])]


def test_on_change_checkbox_dispatch(g):
    eng = FakeEngine()
    g._gb_engine = eng
    win = b("gui_window", "T", 100, 100, 200, 200)
    chk = b("gui_checkbox", win, "An", 20, 20, False)
    b("gui_on_change", chk, _funcref("ch"))
    _press(g, 128, 150)                          # Toggle
    assert eng.calls == [("ch", [])]


def test_on_change_rejects_button(g):
    win = b("gui_window", "T", 0, 0, 200, 200)
    btn = b("gui_button", win, "OK", 0, 0, 80, 30)
    with pytest.raises(GBRuntimeError, match="slider"):
        b("gui_on_change", btn, _funcref("x"))


def test_on_change_no_dispatch_without_value_change(g):
    eng = FakeEngine()
    g._gb_engine = eng
    win = b("gui_window", "T", 100, 100, 300, 200)
    s = b("gui_slider", win, 20, 20, 200, 0.0, 1.0, 0.0)
    b("gui_on_change", s, _funcref("moved"))
    _press(g, 120, 145)                          # value bleibt 0.0
    assert eng.calls == []


# --- GUI_TABLE (Retained-Mode-Tabelle) ------------------------------

def _sarr1(items):
    from gamebasic.interpreter import _GBArray
    a = _GBArray("string", [len(items)], lambda: "")
    for i, v in enumerate(items):
        a.values[i] = v
    return a


def _sarr2(rows):
    from gamebasic.interpreter import _GBArray
    nr = len(rows)
    nc = len(rows[0]) if rows else 0
    a = _GBArray("string", [nr, nc], lambda: "")
    for r in range(nr):
        for c in range(nc):
            a.values[r * nc + c] = rows[r][c]
    return a


def _iarr1(items):
    from gamebasic.interpreter import _GBArray
    a = _GBArray("integer", [len(items)], lambda: 0)
    for i, v in enumerate(items):
        a.values[i] = v
    return a


def _mk_table(g, rows=("Anna 100", "Bert 75", "Cilly 40"),
              wx=100, wy=100, ww=300, wh=260, tx=10, ty=10, tw=260, th=140):
    """Hilfe: Fenster + GUI_TABLE mit 2-spaltigen Daten. Liefert (win, tbl)."""
    win = b("gui_window", "T", wx, wy, ww, wh)
    cells = [r.split(" ") for r in rows]
    tbl = b("gui_table", win, tx, ty, tw, th,
            _sarr1(["Name", "HP"]), _sarr2(cells))
    return win, tbl


def test_table_construct_with_data(g):
    win, tbl = _mk_table(g)
    assert isinstance(tbl, gui_mod._Widget)
    assert tbl.kind == "table"
    assert tbl in win.widgets
    assert b("gui_table_row_count", tbl) == 3
    assert b("gui_table_selected", tbl) == -1


def test_table_construct_empty_then_set():
    win = b("gui_window", "T", 0, 0, 300, 200)
    tbl = b("gui_table", win, 10, 10, 260, 140)
    assert b("gui_table_row_count", tbl) == 0
    b("gui_table_headers", tbl, _sarr1(["A", "B"]))
    b("gui_table_rows", tbl, _sarr2([["1", "2"], ["3", "4"]]))
    assert b("gui_table_row_count", tbl) == 2


def test_table_six_args_rejected():
    win = b("gui_window", "T", 0, 0, 300, 200)
    with pytest.raises(GBRuntimeError, match="headers UND cells"):
        b("gui_table", win, 10, 10, 260, 140, _sarr1(["A"]))


def test_table_rows_col_mismatch():
    win = b("gui_window", "T", 0, 0, 300, 200)
    tbl = b("gui_table", win, 10, 10, 260, 140)
    b("gui_table_headers", tbl, _sarr1(["A", "B"]))
    with pytest.raises(GBRuntimeError, match="Spalten"):
        b("gui_table_rows", tbl, _sarr2([["nur eine"]]))


def test_table_needs_table_widget():
    win = b("gui_window", "T", 0, 0, 200, 200)
    btn = b("gui_button", win, "OK", 0, 0, 80, 30)
    with pytest.raises(GBRuntimeError, match="table"):
        b("gui_table_selected", btn)


def test_table_click_selects_row(g):
    # Fenster (100,100); title_h=22; Tabelle x=10,y=10 -> abs (110,132,260,140)
    # body_y = 132+22 = 154; Zeile 1 = 154+20 = 174..194, Mitte ~184
    win, tbl = _mk_table(g)
    _press(g, 150, 184)
    assert b("gui_table_selected", tbl) == -1     # nur Press -> noch keine Wahl
    _release(g, 150, 184)
    assert b("gui_table_selected", tbl) == 1
    assert b("gui_table_clicked", tbl) == 1


def test_table_clicked_only_one_frame(g):
    win, tbl = _mk_table(g)
    _press(g, 150, 184)
    _release(g, 150, 184)
    assert b("gui_table_clicked", tbl) == 1
    gr("gui_update", g)                            # naechster Frame
    assert b("gui_table_clicked", tbl) == -1
    assert b("gui_table_selected", tbl) == 1       # Selektion bleibt


def test_table_no_select_press_a_release_b(g):
    win, tbl = _mk_table(g)
    _press(g, 150, 184)                            # Zeile 1
    _release(g, 150, 164)                          # Zeile 0 (154..174)
    assert b("gui_table_selected", tbl) == -1


def test_table_set_selected_and_clamp(g):
    win, tbl = _mk_table(g)
    b("gui_table_set_selected", tbl, 2)
    assert b("gui_table_selected", tbl) == 2
    b("gui_table_set_selected", tbl, 99)           # out of range -> -1
    assert b("gui_table_selected", tbl) == -1
    b("gui_table_set_selected", tbl, 1)
    b("gui_table_set_selected", tbl, -1)           # deselektieren
    assert b("gui_table_selected", tbl) == -1


def test_table_selected_reset_when_rows_shrink(g):
    win, tbl = _mk_table(g)
    b("gui_table_set_selected", tbl, 2)
    b("gui_table_rows", tbl, _sarr2([["x", "1"]]))  # nur noch 1 Zeile
    assert b("gui_table_selected", tbl) == -1


def test_table_on_change_on_select(g):
    eng = FakeEngine()
    g._gb_engine = eng
    win, tbl = _mk_table(g)
    b("gui_on_change", tbl, _funcref("sel"))
    _press(g, 150, 184)
    assert eng.calls == []                         # noch nichts (nur Press)
    _release(g, 150, 184)
    assert eng.calls == [("sel", [])]


def test_table_wheel_scrolls(g):
    # 10 Zeilen -> Body laeuft ueber -> vertikale Scrollbar, Mausrad scrollt
    win, tbl = _mk_table(g, rows=tuple(f"R{i} {i}" for i in range(10)))
    g._mx, g._my = 150, 200                        # ueber dem Body
    g._wheel = -2                                  # nach unten scrollen
    gr("gui_update", g)
    assert tbl.tbl["scroll_y"] == 40               # 2 * row_h(20)


def test_table_col_widths_explicit(g):
    win = b("gui_window", "T", 0, 0, 300, 200)
    tbl = b("gui_table", win, 10, 10, 260, 140,
            _sarr1(["A", "B"]), _sarr2([["1", "2"]]))
    b("gui_table_col_widths", tbl, _iarr1([60, 120]))
    assert tbl.tbl["col_widths"] == [60, 120]
    b("gui_table_col_widths", tbl, None)           # zurueck auf Auto
    assert tbl.tbl["col_widths"] is None


def test_table_col_widths_count_mismatch(g):
    win = b("gui_window", "T", 0, 0, 300, 200)
    tbl = b("gui_table", win, 10, 10, 260, 140,
            _sarr1(["A", "B"]), _sarr2([["1", "2"]]))
    with pytest.raises(GBRuntimeError, match="Breiten"):
        b("gui_table_col_widths", tbl, _iarr1([60]))


def test_table_draw_balanced_clip(g):
    win, tbl = _mk_table(g)
    gr("gui_draw", g)
    kinds = {c[0] for c in g.draw_calls}
    assert "box" in kinds and "text" in kinds
    pushes = sum(1 for c in g.draw_calls if c[0] == "push_clip")
    pops = sum(1 for c in g.draw_calls if c[0] == "pop_clip")
    assert pushes == pops and pushes > 0
