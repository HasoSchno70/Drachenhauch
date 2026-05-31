"""Tests fuer das ui-Modul.

UI-Built-ins sind graphics_builtin und brauchen den Graphics-Kontext mit
Maus-Status. Wir mocken den minimal: ein Fake-Graphics-Objekt mit den
Methoden die UI nutzt (mouse_x/y, mouse_button, box, rect, text).
"""
import pytest

from gamebasic.modules import load_module
from gamebasic.modules import ui as ui_mod
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("ui")


class FakeGraphics:
    """Minimaler Graphics-Mock: Maus-/Tastatur-Status setzbar, Drawing-Methoden no-op."""
    def __init__(self):
        self._mx = 0
        self._my = 0
        self._mb = [False, False, False]
        self._typed = ""        # Was pop_text_input() liefert
        self._keys: set = set()  # was keys_pressed() liefert
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
    """Vor jedem Test UI-State leeren - Tests sind voneinander unabhaengig."""
    ui_mod._state.checkbox.clear()
    ui_mod._state.slider.clear()
    ui_mod._state.text.clear()
    ui_mod._state.radio.clear()
    ui_mod._state.tables.clear()
    ui_mod._state.focused = None
    ui_mod._state.was_mouse_down = False
    ui_mod._state.click_origin = None
    ui_mod._state.prev_keys.clear()
    ui_mod._state.frame_count = 0
    ui_mod._state.windows.clear()
    ui_mod._state.win_stack = []
    ui_mod._state.offset_x = 0
    ui_mod._state.offset_y = 0
    ui_mod._state.input_blocked = False
    ui_mod._state.drag_win = None
    ui_mod._state.hover_win = None
    ui_mod._state.active_win = None
    yield


@pytest.fixture
def g():
    return FakeGraphics()


def call_ui(name, g, *args):
    """Direkter Aufruf eines graphics_builtin auf der FakeGraphics-Instanz."""
    from gamebasic.interpreter import GRAPHICS_BUILTINS
    fn = GRAPHICS_BUILTINS[name.lower()]
    return fn(g, list(args))


def end_frame(g):
    call_ui("ui_end_frame", g)


# --- Label -----------------------------------------------------------

def test_label_zeichnet_text(g):
    call_ui("ui_label", g, 10, 20, "Hallo")
    # Label nutzt g.text - ein Aufruf erwartet
    text_calls = [c for c in g.draw_calls if c[0] == "text"]
    assert len(text_calls) == 1
    assert text_calls[0][1][2] == "Hallo"  # 3. Arg ist Text


def test_label_default_color(g):
    call_ui("ui_label", g, 10, 20, "Hi")
    text_args = g.draw_calls[0][1]
    assert text_args[3] == 0xFFFFFF        # Default = weiss


def test_label_custom_color(g):
    call_ui("ui_label", g, 10, 20, "Hi", 0xFF0000)
    text_args = g.draw_calls[0][1]
    assert text_args[3] == 0xFF0000


# --- Button: Klick-Edge ---------------------------------------------

def test_button_kein_klick_ohne_maus(g):
    g._mx, g._my = 50, 50
    g._mb[0] = False
    clicked = call_ui("ui_button", g, "ok", 0, 0, 100, 30, "OK")
    assert clicked is False


def test_button_klick_bei_press_und_release_innerhalb(g):
    g._mx, g._my = 50, 15           # ueber dem Knopf
    # Frame 1: Maustaste runter
    g._mb[0] = True
    clicked = call_ui("ui_button", g, "ok", 0, 0, 100, 30, "OK")
    assert clicked is False         # noch kein Klick (nur Press)
    end_frame(g)
    # Frame 2: Maustaste losgelassen, immer noch ueber dem Knopf
    g._mb[0] = False
    clicked = call_ui("ui_button", g, "ok", 0, 0, 100, 30, "OK")
    assert clicked is True          # Klick komplett
    end_frame(g)


def test_button_kein_klick_bei_press_ausserhalb(g):
    g._mx, g._my = 200, 200         # ausserhalb
    g._mb[0] = True
    clicked = call_ui("ui_button", g, "ok", 0, 0, 100, 30, "OK")
    assert clicked is False


def test_button_haelt_nur_einmal_klick(g):
    """Bei gehaltener Maustaste darf der Klick nicht in jedem Frame neu zaehlen."""
    g._mx, g._my = 50, 15
    g._mb[0] = True
    call_ui("ui_button", g, "ok", 0, 0, 100, 30, "OK")
    end_frame(g)
    # Maustaste weiter gedrueckt halten
    clicked = call_ui("ui_button", g, "ok", 0, 0, 100, 30, "OK")
    assert clicked is False
    end_frame(g)
    # Loslassen -> jetzt Klick
    g._mb[0] = False
    clicked = call_ui("ui_button", g, "ok", 0, 0, 100, 30, "OK")
    assert clicked is True


def test_button_kein_klick_wenn_press_in_a_release_in_b(g):
    """Drueck auf Knopf A, Maus zu Knopf B bewegt, losgelassen -> kein Klick."""
    # Press auf A
    g._mx, g._my = 50, 15
    g._mb[0] = True
    call_ui("ui_button", g, "a", 0, 0, 100, 30, "A")
    end_frame(g)
    # Maus auf B bewegen, Maustaste weiter unten
    g._mx, g._my = 200, 15
    call_ui("ui_button", g, "b", 150, 0, 100, 30, "B")
    end_frame(g)
    # Loslassen ueber B
    g._mb[0] = False
    clicked_b = call_ui("ui_button", g, "b", 150, 0, 100, 30, "B")
    assert clicked_b is False       # B war nicht origin


def test_button_id_required(g):
    with pytest.raises(TypeMismatchError, match="id darf nicht leer"):
        call_ui("ui_button", g, "", 0, 0, 100, 30, "OK")


# --- Checkbox -------------------------------------------------------

def test_checkbox_default_false(g):
    g._mx, g._my = 200, 200
    g._mb[0] = False
    state = call_ui("ui_checkbox", g, "snd", 10, 10, "Sound")
    assert state is False


def test_checkbox_default_true(g):
    g._mx, g._my = 200, 200
    state = call_ui("ui_checkbox", g, "snd", 10, 10, "Sound", True)
    assert state is True


def test_checkbox_toggle_bei_klick(g):
    g._mx, g._my = 12, 12          # innerhalb der 14x14 Box bei (10, 10)
    # Frame 1: Press toggelt
    g._mb[0] = True
    state = call_ui("ui_checkbox", g, "snd", 10, 10, "Sound", False)
    assert state is True
    end_frame(g)
    # Frame 2: Maus weiter unten, kein neuer Toggle
    state = call_ui("ui_checkbox", g, "snd", 10, 10, "Sound")
    assert state is True
    end_frame(g)
    # Frame 3: Maus loslassen + neu pressen
    g._mb[0] = False
    call_ui("ui_checkbox", g, "snd", 10, 10, "Sound")
    end_frame(g)
    g._mb[0] = True
    state = call_ui("ui_checkbox", g, "snd", 10, 10, "Sound")
    assert state is False           # zurueckgetoggelt


def test_checkbox_default_nur_beim_ersten_aufruf(g):
    g._mx, g._my = 200, 200
    state = call_ui("ui_checkbox", g, "snd", 10, 10, "Sound", True)
    assert state is True
    # Default beim 2. Aufruf wird ignoriert (State bleibt erhalten)
    state = call_ui("ui_checkbox", g, "snd", 10, 10, "Sound", False)
    assert state is True


# --- Slider ---------------------------------------------------------

def test_slider_default_min(g):
    g._mx, g._my = 200, 200
    val = call_ui("ui_slider", g, "vol", 10, 50, 100, 0.0, 1.0)
    assert val == 0.0


def test_slider_default_initial(g):
    g._mx, g._my = 200, 200
    val = call_ui("ui_slider", g, "vol", 10, 50, 100, 0.0, 1.0, 0.7)
    assert val == 0.7


def test_slider_klick_setzt_wert(g):
    g._mx, g._my = 60, 55           # auf der Slider-Mitte
    g._mb[0] = True
    val = call_ui("ui_slider", g, "vol", 10, 50, 100, 0.0, 1.0)
    # rel = (60 - 10) / (100 - 1) = 0.505
    assert 0.4 < val < 0.6


def test_slider_klick_links_setzt_min(g):
    g._mx, g._my = 10, 55           # ganz links
    g._mb[0] = True
    val = call_ui("ui_slider", g, "vol", 10, 50, 100, 0.0, 1.0)
    assert val == 0.0


def test_slider_klick_rechts_setzt_max(g):
    g._mx, g._my = 109, 55          # rechte Kante (Box ist [10, 110), exklusiv)
    g._mb[0] = True
    val = call_ui("ui_slider", g, "vol", 10, 50, 100, 0.0, 1.0)
    assert val == 1.0


def test_slider_max_must_be_greater_than_min(g):
    with pytest.raises(GBRuntimeError, match="max muss > min"):
        call_ui("ui_slider", g, "vol", 10, 50, 100, 1.0, 0.5)


def test_slider_default_in_range_clamped(g):
    g._mx, g._my = 200, 200
    val = call_ui("ui_slider", g, "vol", 10, 50, 100, 0.0, 1.0, 5.0)
    assert val == 1.0               # auf max geclamped


# --- Progress -------------------------------------------------------

def test_progress_zeichnet_rahmen_und_fuellung(g):
    call_ui("ui_progress", g, 0, 0, 100, 10, 50.0, 100.0)
    kinds = [c[0] for c in g.draw_calls]
    # Hintergrund-box, Rahmen-rect, Fuellung-box
    assert kinds.count("box") == 2
    assert kinds.count("rect") == 1


def test_progress_keine_fuellung_bei_null(g):
    call_ui("ui_progress", g, 0, 0, 100, 10, 0.0, 100.0)
    boxes = [c for c in g.draw_calls if c[0] == "box"]
    # Nur Hintergrund-Box, keine Fuellung
    assert len(boxes) == 1


def test_progress_voll_bei_max(g):
    call_ui("ui_progress", g, 0, 0, 100, 10, 200.0, 100.0)
    boxes = [c for c in g.draw_calls if c[0] == "box"]
    # Hintergrund + voller Fuellbalken
    assert len(boxes) == 2


def test_progress_max_muss_positiv(g):
    with pytest.raises(GBRuntimeError, match="max"):
        call_ui("ui_progress", g, 0, 0, 100, 10, 5.0, 0.0)


# --- Panel ----------------------------------------------------------

def test_panel_ohne_titel(g):
    call_ui("ui_panel", g, 10, 10, 200, 100)
    kinds = [c[0] for c in g.draw_calls]
    # 1 Box (BG) + 1 Rect (Rahmen) - kein Titel
    assert kinds.count("box") == 1
    assert kinds.count("rect") == 1
    assert "text" not in kinds


def test_panel_mit_titel(g):
    call_ui("ui_panel", g, 10, 10, 200, 100, "Optionen")
    kinds = [c[0] for c in g.draw_calls]
    text_calls = [c for c in g.draw_calls if c[0] == "text"]
    assert len(text_calls) == 1
    assert text_calls[0][1][2] == "Optionen"


# --- TextField ------------------------------------------------------

def test_textfield_initial_leer(g):
    val = call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    assert val == ""


def test_textfield_klick_fokussiert(g):
    g._mx, g._my = 50, 12
    g._mb[0] = True
    call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    assert ui_mod._state.focused == "name"


def test_textfield_klick_ausserhalb_blurred(g):
    # Erstmal fokussieren (Press + Release im Feld)
    g._mx, g._my = 50, 12
    g._mb[0] = True
    call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    end_frame(g)
    g._mb[0] = False
    call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    end_frame(g)
    assert ui_mod._state.focused == "name"
    # Jetzt NEUER Klick ausserhalb -> Blur
    g._mx, g._my = 500, 500
    g._mb[0] = True
    call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    assert ui_mod._state.focused is None


def test_textfield_nimmt_getippte_zeichen_auf(g):
    # Fokussieren via Klick
    g._mx, g._my = 50, 12
    g._mb[0] = True
    call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    end_frame(g)
    g._mb[0] = False
    # Tippen
    g._typed = "Anna"
    val = call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    assert val == "Anna"


def test_textfield_unfokussiert_keine_eingabe(g):
    """Nicht-fokussierte Felder ignorieren Tipp-Eingaben."""
    g._typed = "X"
    val = call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    assert val == ""


def test_textfield_backspace_loescht_letztes_zeichen(g):
    # Fokussieren + Text setzen
    ui_mod._state.text["name"] = "ABC"
    ui_mod._state.focused = "name"
    # Backspace via Edge: aktuelles Frame hat key 8 gedrueckt, prev_keys leer
    g._keys = {8}
    val = call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    assert val == "AB"


def test_textfield_backspace_keine_repeat_im_gleichen_frame(g):
    """Wenn Backspace im prev_keys schon war (gehalten), nicht erneut loeschen."""
    ui_mod._state.text["name"] = "ABC"
    ui_mod._state.focused = "name"
    ui_mod._state.prev_keys = {8}
    g._keys = {8}    # weiter gehalten
    val = call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    assert val == "ABC"      # unveraendert - gehalten zaehlt nicht als Edge


def test_textfield_set_setzt_wert(g):
    call_ui("ui_textfield_set", g, "name", "Vorgabe")
    val = call_ui("ui_textfield", g, "name", 0, 0, 200, 24)
    assert val == "Vorgabe"


# --- Radio ----------------------------------------------------------

def _str_array(items):
    """Helper: baut ein _GBArray of string fuer UI_RADIO."""
    from gamebasic.interpreter import _GBArray
    arr = _GBArray("string", [len(items)], lambda: "")
    for i, v in enumerate(items):
        arr.values[i] = v
    return arr


def test_radio_default_zero(g):
    idx = call_ui("ui_radio", g, "diff", 0, 0, _str_array(["Easy", "Mid", "Hard"]))
    assert idx == 0


def test_radio_default_param(g):
    idx = call_ui("ui_radio", g, "diff", 0, 0, _str_array(["A", "B", "C"]), 2)
    assert idx == 2


def test_radio_klick_aendert_auswahl(g):
    options = _str_array(["A", "B", "C"])
    # Erste Initialisierung
    call_ui("ui_radio", g, "diff", 0, 0, options)
    end_frame(g)
    # Klick auf zweite Zeile (row_h=18, also y=18..36)
    g._mx, g._my = 50, 25
    g._mb[0] = True
    idx = call_ui("ui_radio", g, "diff", 0, 0, options)
    assert idx == 1


def test_radio_leeres_array_returnt_minus_eins(g):
    idx = call_ui("ui_radio", g, "diff", 0, 0, _str_array([]))
    assert idx == -1


def test_radio_options_typecheck(g):
    with pytest.raises(TypeMismatchError, match="ARRAY OF STRING"):
        call_ui("ui_radio", g, "diff", 0, 0, "kein array")


# --- Tabellen -------------------------------------------------------

def _str_array_2d(rows):
    """rows ist list[list[str]]; baut _GBArray mit dims=(len(rows), len(rows[0]))."""
    from gamebasic.interpreter import _GBArray
    n_rows = len(rows)
    n_cols = len(rows[0]) if rows else 0
    arr = _GBArray("string", [n_rows, n_cols], lambda: "")
    for r in range(n_rows):
        for c in range(n_cols):
            arr.values[r * n_cols + c] = rows[r][c]
    return arr


def _int_array_2d(rows):
    from gamebasic.interpreter import _GBArray
    n_rows = len(rows)
    n_cols = len(rows[0]) if rows else 0
    arr = _GBArray("integer", [n_rows, n_cols], lambda: 0)
    for r in range(n_rows):
        for c in range(n_cols):
            arr.values[r * n_cols + c] = rows[r][c]
    return arr


def _int_array_1d(items):
    from gamebasic.interpreter import _GBArray
    arr = _GBArray("integer", [len(items)], lambda: 0)
    for i, v in enumerate(items):
        arr.values[i] = v
    return arr


def test_table_basic_rendering(g):
    headers = _str_array(["Name", "HP"])
    cells = _str_array_2d([
        ["Anna", "100"],
        ["Bert", "75"],
    ])
    rc = call_ui("ui_table", g, "tbl", 0, 0, 200, 100, headers, cells)
    assert rc == -1
    # Zeichnungen passieren
    assert any(c[0] == "text" for c in g.draw_calls)


def test_table_returns_minus_one_without_click(g):
    headers = _str_array(["A"])
    cells = _str_array_2d([["x"], ["y"]])
    rc = call_ui("ui_table", g, "tbl", 0, 0, 100, 100, headers, cells)
    assert rc == -1


def test_table_click_returns_row_index(g):
    headers = _str_array(["Name"])
    cells = _str_array_2d([
        ["Anna"], ["Bert"], ["Cilly"],
    ])
    # Header H = 22, Row H = 20. Body startet bei y=22.
    # Zeile 1 (Bert) liegt bei y=22 + 20 = 42..62, Mitte ~52
    g._mx, g._my = 50, 52

    # Frame 1: Press
    g._mb[0] = True
    rc = call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    assert rc == -1
    end_frame(g)

    # Frame 2: Release ueber derselben Zeile -> Klick
    g._mb[0] = False
    rc = call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    assert rc == 1


def test_table_no_click_if_press_in_a_release_in_b(g):
    headers = _str_array(["Name"])
    cells = _str_array_2d([
        ["Anna"], ["Bert"], ["Cilly"],
    ])
    # Press auf Zeile 0 (y~32)
    g._mx, g._my = 50, 32
    g._mb[0] = True
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    end_frame(g)
    # Release auf Zeile 1 (y~52) -> kein Klick
    g._my = 52
    g._mb[0] = False
    rc = call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    assert rc == -1


def test_table_mouse_wheel_scrolls_vertically(g):
    headers = _str_array(["A"])
    # 30 Zeilen -> Content-Hoehe 30*20 = 600, Body-Hoehe 100-22-12 = 66
    rows = [[str(i)] for i in range(30)]
    cells = _str_array_2d(rows)

    # Maus ueber der Tabelle
    g._mx, g._my = 50, 50
    # 3 Notches nach unten scrollen (negativ)
    g._wheel = -3
    call_ui("ui_table", g, "tbl", 0, 0, 100, 100, headers, cells)

    st = ui_mod._state.tables["tbl"]
    # 3 * 20 px = 60 px nach unten
    assert st["scroll_y"] == 60


def test_table_wheel_clamped_to_max(g):
    headers = _str_array(["A"])
    rows = [[str(i)] for i in range(5)]   # passt komplett rein
    cells = _str_array_2d(rows)
    g._mx, g._my = 50, 50
    g._wheel = -100
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    st = ui_mod._state.tables["tbl"]
    assert st["scroll_y"] == 0   # nichts zu scrollen


def test_table_cell_color_used(g):
    headers = _str_array(["A", "B"])
    cells = _str_array_2d([
        ["x", "y"],
        ["a", "b"],
    ])
    colors = _int_array_2d([
        [0xFF0000, 0x00FF00],
        [0x0000FF, 0xFFFFFF],
    ])
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells, colors)
    text_calls = [c for c in g.draw_calls if c[0] == "text"]
    # Mindestens ein Text-Aufruf mit roter Farbe (Anna's "x")
    colors_used = {tc[1][3] for tc in text_calls}
    assert 0xFF0000 in colors_used
    assert 0x00FF00 in colors_used


def test_table_col_widths_explicit(g):
    headers = _str_array(["A", "B"])
    cells = _str_array_2d([["x", "y"]])
    widths = _int_array_1d([60, 80])
    rc = call_ui("ui_table", g, "tbl", 0, 0, 200, 200,
                 headers, cells, None, widths)
    assert rc == -1


def test_table_typecheck_headers_must_be_1d(g):
    cells = _str_array_2d([["x", "y"]])
    with pytest.raises(TypeMismatchError):
        call_ui("ui_table", g, "tbl", 0, 0, 200, 200,
                "kein array", cells)


def test_table_typecheck_cells_cols_match_headers(g):
    headers = _str_array(["A", "B", "C"])      # 3 Spalten
    cells = _str_array_2d([["x", "y"]])        # nur 2 Spalten -> Fehler
    with pytest.raises(GBRuntimeError, match="Spalten"):
        call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)


def test_table_typecheck_color_dims_match_cells(g):
    headers = _str_array(["A"])
    cells = _str_array_2d([["x"], ["y"]])      # 2 Zeilen
    colors = _int_array_2d([[0xFF0000]])       # nur 1 Zeile -> Fehler
    with pytest.raises(GBRuntimeError, match=r"\[2, 1\]"):
        call_ui("ui_table", g, "tbl", 0, 0, 200, 200,
                headers, cells, colors)


def test_table_empty_headers_rejected(g):
    headers = _str_array([])
    cells = _str_array_2d([])
    with pytest.raises(GBRuntimeError, match="leer"):
        call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)


def test_table_cell_bg_colors_drawn(g):
    """Cell-bg mit RGB != -1 erzeugt eine box pro markierter Zelle."""
    headers = _str_array(["A", "B"])
    cells = _str_array_2d([["x", "y"]])
    bgs = _int_array_2d([[0xFF0000, -1]])    # erste Zelle rot, zweite leer
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200,
            headers, cells, None, None, bgs)
    boxes = [c for c in g.draw_calls if c[0] == "box"]
    # Mindestens eine Box mit der RGB-Farbe (Cell-bg)
    bg_colors_used = {b[1][4] for b in boxes if len(b[1]) >= 5}
    assert 0xFF0000 in bg_colors_used


def test_table_cell_bg_minus_one_means_no_bg(g):
    """Cell-bg = -1 zeichnet keine bg-Box, nur die regulaere row-bg."""
    headers = _str_array(["A"])
    cells = _str_array_2d([["x"]])
    bgs = _int_array_2d([[-1]])   # alle Zellen ohne bg
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200,
            headers, cells, None, None, bgs)
    boxes = [c for c in g.draw_calls if c[0] == "box"]
    bg_colors = [b[1][4] for b in boxes if len(b[1]) >= 5]
    # Keine -1-Farbe sollte als RGB-Box rauskommen
    assert -1 not in bg_colors


def test_table_cell_bg_dim_check(g):
    """cell_bg_colors muss [rows, cols] matchen."""
    headers = _str_array(["A"])
    cells = _str_array_2d([["x"], ["y"]])
    bgs = _int_array_2d([[0xFF0000]])    # nur 1 Zeile
    with pytest.raises(GBRuntimeError, match=r"\[2, 1\]"):
        call_ui("ui_table", g, "tbl", 0, 0, 200, 200,
                headers, cells, None, None, bgs)


def test_table_clip_stack_balanced(g):
    """Jeder push_clip muss einen pop_clip-Partner haben - sonst geht das
    Body-Clipping nach UI_TABLE fuer nachfolgende Aufrufe verloren."""
    headers = _str_array(["A", "B"])
    cells = _str_array_2d([
        ["x", "y"], ["a", "b"], ["c", "d"],
    ])
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    pushes = sum(1 for c in g.draw_calls if c[0] == "push_clip")
    pops = sum(1 for c in g.draw_calls if c[0] == "pop_clip")
    assert pushes == pops, f"{pushes} push vs {pops} pop - Stack unausgeglichen"


# --- UI_TABLE: Selektion + Header-Klick -----------------------------

def test_table_click_sets_selection(g):
    headers = _str_array(["Name"])
    cells = _str_array_2d([["Anna"], ["Bert"], ["Cilly"]])
    assert call_ui("ui_table_selected", g, "tbl") == -1   # noch nie gezeichnet
    # Zeile 1 (Bert): body bei y=22, Zeile 1 = 42..62, Mitte ~52
    g._mx, g._my = 50, 52
    g._mb[0] = True
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    end_frame(g)
    g._mb[0] = False
    rc = call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    assert rc == 1
    assert call_ui("ui_table_selected", g, "tbl") == 1


def test_table_selection_survives_frames(g):
    headers = _str_array(["A"])
    cells = _str_array_2d([["x"], ["y"]])
    # set_selected vor dem ersten Zeichnen ist no-op (id unbekannt)
    call_ui("ui_table_set_selected", g, "tbl", 0)
    assert call_ui("ui_table_selected", g, "tbl") == -1
    # zeichnen, dann programmatisch selektieren
    call_ui("ui_table", g, "tbl", 0, 0, 100, 100, headers, cells)
    call_ui("ui_table_set_selected", g, "tbl", 1)
    assert call_ui("ui_table_selected", g, "tbl") == 1
    # weiterer Frame ohne Klick -> Selektion bleibt
    end_frame(g)
    call_ui("ui_table", g, "tbl", 0, 0, 100, 100, headers, cells)
    assert call_ui("ui_table_selected", g, "tbl") == 1


def test_table_set_selected_negative_is_deselect(g):
    headers = _str_array(["A"])
    cells = _str_array_2d([["x"], ["y"]])
    call_ui("ui_table", g, "tbl", 0, 0, 100, 100, headers, cells)
    call_ui("ui_table_set_selected", g, "tbl", 1)
    call_ui("ui_table_set_selected", g, "tbl", -5)
    assert call_ui("ui_table_selected", g, "tbl") == -1


def test_table_selection_reset_on_shrink(g):
    headers = _str_array(["A"])
    call_ui("ui_table", g, "tbl", 0, 0, 100, 100, headers,
            _str_array_2d([["x"], ["y"], ["z"]]))
    call_ui("ui_table_set_selected", g, "tbl", 2)
    end_frame(g)
    # naechster Frame nur 1 Zeile -> Selektion ungueltig -> -1
    call_ui("ui_table", g, "tbl", 0, 0, 100, 100, headers,
            _str_array_2d([["x"]]))
    assert call_ui("ui_table_selected", g, "tbl") == -1


def test_table_header_click_returns_col(g):
    headers = _str_array(["Name", "HP"])
    cells = _str_array_2d([["Anna", "100"], ["Bert", "75"]])
    # Header-Band y=0..22; Spalte 1 liegt rechts (auto ~93px breit)
    g._mx, g._my = 150, 10
    g._mb[0] = True
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    assert call_ui("ui_table_header_click", g, "tbl") == -1   # nur Press
    end_frame(g)
    g._mb[0] = False
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    assert call_ui("ui_table_header_click", g, "tbl") == 1
    # Klick auf Zeile setzt KEINE Header-Spalte
    assert call_ui("ui_table_selected", g, "tbl") == -1


def test_table_header_click_only_one_frame(g):
    headers = _str_array(["Name", "HP"])
    cells = _str_array_2d([["Anna", "100"]])
    g._mx, g._my = 30, 10
    g._mb[0] = True
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    end_frame(g)
    g._mb[0] = False
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    assert call_ui("ui_table_header_click", g, "tbl") == 0
    end_frame(g)
    # naechster Frame ohne neuen Klick -> wieder -1
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    assert call_ui("ui_table_header_click", g, "tbl") == -1


def test_table_header_click_press_a_release_b(g):
    headers = _str_array(["Name", "HP"])
    cells = _str_array_2d([["Anna", "100"]])
    g._mx, g._my = 30, 10           # Press auf Spalte 0
    g._mb[0] = True
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    end_frame(g)
    g._mx, g._my = 150, 10          # Release auf Spalte 1 -> kein Klick
    g._mb[0] = False
    call_ui("ui_table", g, "tbl", 0, 0, 200, 200, headers, cells)
    assert call_ui("ui_table_header_click", g, "tbl") == -1


# --- Reset ----------------------------------------------------------

def test_reset_loescht_state(g):
    call_ui("ui_checkbox", g, "snd", 10, 10, "Sound", True)
    call_ui("ui_slider", g, "vol", 10, 50, 100, 0.0, 1.0, 0.5)
    call_ui("ui_textfield_set", g, "name", "X")
    headers = _str_array(["A"])
    cells = _str_array_2d([["x"]])
    call_ui("ui_table", g, "tbl", 0, 0, 100, 100, headers, cells)
    call_ui("ui_reset", g)
    assert ui_mod._state.checkbox == {}
    assert ui_mod._state.slider == {}
    assert ui_mod._state.text == {}
    assert ui_mod._state.radio == {}
    assert ui_mod._state.tables == {}


# --- Immediate-Mode-Fenster (UI_WINDOW_BEGIN/END, Phase 4) ----------

def test_window_begin_offsets_and_end_restores(g):
    g._mx, g._my = -50, -50            # Maus weit weg -> keine Interaktion
    open_ = call_ui("ui_window_begin", g, "w", "Titel", 50, 40, 200, 150)
    assert open_ is True
    assert ui_mod._state.offset_x == 50
    assert ui_mod._state.offset_y == 60        # 40 + TITLE_H (20)
    call_ui("ui_window_end", g)
    assert ui_mod._state.offset_x == 0
    assert ui_mod._state.offset_y == 0


def test_window_child_button_uses_offset(g):
    # Fenster (50,40); Button rel (10,10,80,24) -> abs (60,70,80,24), Mitte (100,82)
    g._mx, g._my = 100, 82
    g._mb[0] = True
    call_ui("ui_window_begin", g, "w", "T", 50, 40, 200, 150)
    c1 = call_ui("ui_button", g, "b", 10, 10, 80, 24, "OK")
    call_ui("ui_window_end", g)
    assert c1 is False                          # nur Press
    end_frame(g)
    g._mb[0] = False
    call_ui("ui_window_begin", g, "w", "T", 50, 40, 200, 150)
    c2 = call_ui("ui_button", g, "b", 10, 10, 80, 24, "OK")
    call_ui("ui_window_end", g)
    assert c2 is True                           # Release ueber offset-korrektem Button


def test_window_collapse_toggle(g):
    # Collapse-Btn: (wx+4, wy+(20-12)//2)=(54,44), 12x12 -> Mitte (60,50)
    g._mx, g._my = 60, 50
    g._mb[0] = True
    open1 = call_ui("ui_window_begin", g, "w", "T", 50, 40, 200, 150)
    call_ui("ui_window_end", g)
    assert ui_mod._state.windows["w"]["collapsed"] is True
    assert open1 is False                       # sofort eingeklappt -> Body skippen
    end_frame(g)
    g._mb[0] = False
    open2 = call_ui("ui_window_begin", g, "w", "T", 50, 40, 200, 150)
    call_ui("ui_window_end", g)
    assert open2 is False


def test_window_drag(g):
    # Titelleiste (nicht Collapse-Btn): (120,50) in (50,40,200,20)
    g._mx, g._my = 120, 50
    g._mb[0] = True
    call_ui("ui_window_begin", g, "w", "T", 50, 40, 200, 150)
    call_ui("ui_window_end", g)
    assert ui_mod._state.drag_win == "w"
    end_frame(g)                                # was_mouse_down=True, active_win="w"
    g._mx, g._my = 140, 70                      # +20, +20
    call_ui("ui_window_begin", g, "w", "T", 50, 40, 200, 150)
    call_ui("ui_window_end", g)
    assert ui_mod._state.windows["w"]["x"] == 70
    assert ui_mod._state.windows["w"]["y"] == 60


def test_window_input_gating_blocks_child(g):
    ui_mod._state.active_win = "other"          # ein anderes Fenster ist input-aktiv
    g._mx, g._my = 100, 82
    g._mb[0] = True
    call_ui("ui_window_begin", g, "w", "T", 50, 40, 200, 150)
    call_ui("ui_button", g, "b", 10, 10, 80, 24, "OK")
    call_ui("ui_window_end", g)
    assert ui_mod._state.click_origin is None   # Press geblockt -> kein Klick-Origin
