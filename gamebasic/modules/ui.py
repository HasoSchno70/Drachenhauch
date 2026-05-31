"""UI-Modul fuer GameBasic - Immediate-Mode-Komponenten.

Built-ins:
    UI_LABEL(x, y, text$[, color])               ' nur Zeichnen
    UI_BUTTON(id$, x, y, w, h, text$[, bg, fg])  -> BOOLEAN  (Klick im Frame?)
    UI_CHECKBOX(id$, x, y, label$[, default])    -> BOOLEAN  (Toggle-State)
    UI_SLIDER(id$, x, y, w, min, max[, default]) -> FLOAT    (aktueller Wert)
    UI_PROGRESS(x, y, w, h, value, max[, fg, bg])' nur Zeichnen, read-only
    UI_PANEL(x, y, w, h[, title$[, bg]])         ' Container mit optionalem Titel
    UI_TEXTFIELD(id$, x, y, w, h[, placeholder$])-> STRING (aktueller Wert)
    UI_TEXTFIELD_SET(id$, value$)                ' Wert programmatisch setzen
    UI_RADIO(id$, x, y, options[, default_idx])  -> INTEGER (gewaehlter Index)
    UI_TABLE(id$, x, y, w, h, headers, cells
             [, cell_colors[, col_widths[, cell_bg_colors]]])
                                                 -> INTEGER (geklickte Zeile, -1 wenn keine)
    UI_END_FRAME()                               ' am Ende JEDES Frames vor FLIP()
    UI_RESET()                                   ' allen State loeschen

Bedienung:
- Komponenten werden jeden Frame neu aufgerufen (Immediate-Mode).
- State (Checkbox, Slider) wird ueber `id$` indexiert und intern gehalten.
- `UI_END_FRAME()` ist Pflicht am Ende jedes Frames vor `FLIP()` - sonst
  zaehlt eine gehaltene Maustaste als kontinuierlich klickend.

Beispiel:
    IMPORT "ui"
    SCREEN(320, 240, "UI-Demo", 2)
    WHILE NOT QUITREQUESTED()
        CLS(RGB(20, 25, 40))
        UI_LABEL(10, 10, "Mein Spiel")
        IF UI_BUTTON("start", 10, 40, 100, 30, "Start") THEN
            PRINT "Spiel startet"
        END IF
        DIM sound AS BOOLEAN
        sound = UI_CHECKBOX("snd", 10, 80, "Sound an", TRUE)
        DIM vol AS FLOAT
        vol = UI_SLIDER("vol", 10, 100, 200, 0.0, 1.0, 0.7)
        UI_END_FRAME()
        FLIP()
        SLEEP(16)
    WEND
"""
from __future__ import annotations

from ..builtins_registry import graphics_builtin
from ..errors import GBRuntimeError, TypeMismatchError


class _UIState:
    """Persistenter State pro Komponente, indexiert ueber id-String."""
    def __init__(self):
        self.checkbox: dict = {}      # id -> bool
        self.slider: dict = {}        # id -> float
        self.text: dict = {}          # id -> str (TEXTFIELD-Inhalt)
        self.radio: dict = {}         # id -> int (gewaehlter Index)
        # Tabellen: id -> dict mit scroll_x, scroll_y, drag_v, drag_h, drag_off
        self.tables: dict = {}
        self.focused: str | None = None    # id des fokussierten Textfields
        # Maus-Klick-Edge-Detection: war Maustaste im letzten Frame gedrueckt?
        self.was_mouse_down = False
        # Klick auf-und-loslassen muss innerhalb derselben Komponente passieren.
        # Wir merken uns die "id wo Klick begonnen hat" - wird beim Loslassen
        # mit der id wo Klick endet verglichen.
        self.click_origin = None
        # Edge-Detection fuer Tasten (Backspace, Enter, ...) im TEXTFIELD
        self.prev_keys: set = set()
        # Caret-Blink: Frame-Counter fuer den blinkenden Cursor im Textfield
        self.frame_count = 0
        # --- Immediate-Mode-Fenster (UI_WINDOW_BEGIN/END) ---
        self.windows: dict = {}       # id -> {x, y, collapsed}
        self.win_stack: list = []     # gesicherte (offset_x, offset_y, input_blocked)
        self.offset_x = 0             # aktueller Fenster-Offset fuer Widgets
        self.offset_y = 0
        self.input_blocked = False    # True wenn die aktuelle Fenster-Ebene keinen Input hat
        self.drag_win = None          # id des gerade gezogenen Fensters
        self.drag_off = (0, 0)
        self.hover_win = None         # oberstes Fenster unter der Maus (dieser Frame)
        self.active_win = None        # Fenster mit Input (Hover aus dem Vorframe)


_state = _UIState()


# Wichtige SDL-Keycodes fuer TEXTFIELD-Handling
_KEY_BACKSPACE = 8
_KEY_RETURN = 13
_KEY_TAB = 9
_KEY_DELETE = 127


# --- Hilfsfunktionen --------------------------------------------------

def _check_int(v, fn: str, name: str) -> int:
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn}: {name} muss INTEGER sein")
    return v


def _check_num(v, fn: str, name: str) -> float:
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeMismatchError(f"{fn}: {name} muss Zahl sein")
    return float(v)


def _check_str(v, fn: str, name: str) -> str:
    if not isinstance(v, str):
        raise TypeMismatchError(f"{fn}: {name} muss STRING sein")
    return v


def _check_id(v, fn: str) -> str:
    s = _check_str(v, fn, "id")
    if not s:
        raise TypeMismatchError(f"{fn}: id darf nicht leer sein")
    return s


def _check_color(v, fn: str) -> int:
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn}: Farbe muss INTEGER sein")
    if v < 0 or v > 0xFFFFFF:
        raise GBRuntimeError(f"{fn}: Farbe muss 0..0xFFFFFF sein")
    return v


def _in_box(mx: int, my: int, x: int, y: int, w: int, h: int) -> bool:
    return x <= mx < x + w and y <= my < y + h


def _lighten(color: int, factor: float) -> int:
    r = (color >> 16) & 0xFF
    g = (color >> 8) & 0xFF
    b = color & 0xFF
    r = min(255, int(r + (255 - r) * factor))
    g = min(255, int(g + (255 - g) * factor))
    b = min(255, int(b + (255 - b) * factor))
    return (r << 16) | (g << 8) | b


_OFFSCREEN = -100000


def _mouse(g):
    """Maus-Position fuer Widgets. Liefert Off-Screen-Koordinaten, wenn die
    aktuelle Fenster-Ebene keinen Input besitzt (von einem anderen Fenster
    ueberdeckt) -- so reagiert kein darunterliegendes Widget, ohne dass die
    einzelnen Widget-Funktionen das Fenster-Konzept kennen muessen. Ausserhalb
    von Fenstern ist nichts geblockt und die echte Maus wird geliefert."""
    if _state.input_blocked:
        return _OFFSCREEN, _OFFSCREEN
    return g.mouse_x(), g.mouse_y()


# --- Built-ins -------------------------------------------------------

@graphics_builtin("UI_LABEL", arity=(3, 4))
def _label(g, *args):
    x = _check_int(args[0], "UI_LABEL", "x")
    y = _check_int(args[1], "UI_LABEL", "y")
    x += _state.offset_x
    y += _state.offset_y
    text = _check_str(args[2], "UI_LABEL", "text")
    color = _check_color(args[3], "UI_LABEL") if len(args) == 4 else 0xFFFFFF
    g.text(x, y, text, color)
    return None


@graphics_builtin("UI_BUTTON", arity=(6, 8))
def _button(g, *args):
    """Zeichnet einen Knopf, gibt TRUE zurueck im Frame des Loslassens
    bei vorigem Druecken auf demselben Knopf."""
    id_str = _check_id(args[0], "UI_BUTTON")
    x = _check_int(args[1], "UI_BUTTON", "x")
    y = _check_int(args[2], "UI_BUTTON", "y")
    x += _state.offset_x
    y += _state.offset_y
    w = _check_int(args[3], "UI_BUTTON", "w")
    h = _check_int(args[4], "UI_BUTTON", "h")
    text = _check_str(args[5], "UI_BUTTON", "text")
    bg_color = _check_color(args[6], "UI_BUTTON") if len(args) >= 7 else 0x40445C
    fg_color = _check_color(args[7], "UI_BUTTON") if len(args) >= 8 else 0xFFFFFF

    mx, my = _mouse(g)
    is_down = g.mouse_button(0)
    hovered = _in_box(mx, my, x, y, w, h)
    pressed = hovered and is_down

    # Klick-Edge: gerade gedrueckt -> origin merken
    if hovered and is_down and not _state.was_mouse_down:
        _state.click_origin = id_str
    # Loslassen ueber demselben Knopf, wo der Klick begonnen hat -> "geklickt"
    clicked = (
        hovered and not is_down and _state.was_mouse_down
        and _state.click_origin == id_str
    )

    # Zeichnen: heller bei Hover, dunkler bei Press
    if pressed:
        bg = _lighten(bg_color, -0.2) if False else bg_color
        # Pressed-Look: einfach dunkler
        r = (bg_color >> 16) & 0xFF
        gr = (bg_color >> 8) & 0xFF
        b = bg_color & 0xFF
        bg = (max(0, r - 40) << 16) | (max(0, gr - 40) << 8) | max(0, b - 40)
    elif hovered:
        bg = _lighten(bg_color, 0.25)
    else:
        bg = bg_color
    g.box(x, y, x + w - 1, y + h - 1, bg)
    g.rect(x, y, x + w - 1, y + h - 1, fg_color)
    # Text grob mittig (5px Padding ist dick fuer 20px-Schrift)
    g.text(x + 6, y + (h - 14) // 2, text, fg_color)
    return clicked


@graphics_builtin("UI_CHECKBOX", arity=(4, 5))
def _checkbox(g, *args):
    """Toggle-Checkbox. Default-Wert wird nur beim ersten Aufruf einer id
    angewandt (sonst bleibt der vom User getoggelte State erhalten)."""
    id_str = _check_id(args[0], "UI_CHECKBOX")
    x = _check_int(args[1], "UI_CHECKBOX", "x")
    y = _check_int(args[2], "UI_CHECKBOX", "y")
    x += _state.offset_x
    y += _state.offset_y
    label = _check_str(args[3], "UI_CHECKBOX", "label")

    # Initial-Wert nur beim ersten Aufruf der id setzen
    if id_str not in _state.checkbox:
        if len(args) == 5:
            initial = args[4]
            if not isinstance(initial, bool):
                raise TypeMismatchError("UI_CHECKBOX: default muss BOOLEAN sein")
            _state.checkbox[id_str] = initial
        else:
            _state.checkbox[id_str] = False

    box_size = 14
    mx, my = _mouse(g)
    is_down = g.mouse_button(0)
    hovered = _in_box(mx, my, x, y, box_size, box_size)

    # Edge-detection: Toggle nur beim Druecken (nicht beim Loslassen)
    if hovered and is_down and not _state.was_mouse_down:
        _state.checkbox[id_str] = not _state.checkbox[id_str]

    # Zeichnen: Rahmen, Fuellung wenn angekreuzt, Label rechts
    border = 0xC0C0C0
    fill = 0x80C0FF
    g.rect(x, y, x + box_size - 1, y + box_size - 1, border)
    if hovered:
        g.rect(x - 1, y - 1, x + box_size, y + box_size, fill)
    if _state.checkbox[id_str]:
        g.box(x + 3, y + 3, x + box_size - 4, y + box_size - 4, fill)
    g.text(x + box_size + 5, y, label, 0xFFFFFF)
    return _state.checkbox[id_str]


@graphics_builtin("UI_SLIDER", arity=(6, 7))
def _slider(g, *args):
    """Horizontaler Slider. Gibt aktuellen Wert (FLOAT) zwischen min und max
    zurueck. Wert wird beim Klicken/Ziehen geaendert."""
    id_str = _check_id(args[0], "UI_SLIDER")
    x = _check_int(args[1], "UI_SLIDER", "x")
    y = _check_int(args[2], "UI_SLIDER", "y")
    x += _state.offset_x
    y += _state.offset_y
    w = _check_int(args[3], "UI_SLIDER", "w")
    min_val = _check_num(args[4], "UI_SLIDER", "min")
    max_val = _check_num(args[5], "UI_SLIDER", "max")

    if max_val <= min_val:
        raise GBRuntimeError("UI_SLIDER: max muss > min sein")

    if id_str not in _state.slider:
        if len(args) == 7:
            initial = _check_num(args[6], "UI_SLIDER", "default")
            _state.slider[id_str] = max(min_val, min(max_val, initial))
        else:
            _state.slider[id_str] = min_val

    h = 14
    handle_w = 10
    mx, my = _mouse(g)
    is_down = g.mouse_button(0)
    in_box = _in_box(mx, my, x, y, w, h)

    if in_box and is_down:
        rel = (mx - x) / max(1, w - 1)
        rel = max(0.0, min(1.0, rel))
        _state.slider[id_str] = min_val + rel * (max_val - min_val)

    # Zeichnen: Track + Handle
    track_color = 0x404060
    border = 0xFFFFFF
    handle_color = 0x80C0FF
    g.box(x, y + h // 2 - 1, x + w - 1, y + h // 2 + 1, track_color)
    g.rect(x, y, x + w - 1, y + h - 1, border)
    handle_pos = (_state.slider[id_str] - min_val) / (max_val - min_val)
    handle_x = x + int(handle_pos * (w - handle_w))
    g.box(handle_x, y, handle_x + handle_w - 1, y + h - 1, handle_color)
    return _state.slider[id_str]


@graphics_builtin("UI_PROGRESS", arity=(6, 8))
def _progress(g, *args):
    """Read-only Fortschrittsbalken (HP-Bar, Loading, ...).

    Keine id, kein State - rein visuell. value wird auf [0, max] geklemmt.
    Bei value <= 0 wird die Fuell-Box weggelassen, bei value >= max
    durchgezogen.
    """
    x = _check_int(args[0], "UI_PROGRESS", "x")
    y = _check_int(args[1], "UI_PROGRESS", "y")
    x += _state.offset_x
    y += _state.offset_y
    w = _check_int(args[2], "UI_PROGRESS", "w")
    h = _check_int(args[3], "UI_PROGRESS", "h")
    value = _check_num(args[4], "UI_PROGRESS", "value")
    max_val = _check_num(args[5], "UI_PROGRESS", "max")
    fg = _check_color(args[6], "UI_PROGRESS") if len(args) >= 7 else 0x4CAF50
    bg = _check_color(args[7], "UI_PROGRESS") if len(args) >= 8 else 0x303040

    if max_val <= 0:
        raise GBRuntimeError("UI_PROGRESS: max muss > 0 sein")
    if w < 2 or h < 2:
        return None

    ratio = max(0.0, min(1.0, value / max_val))
    fill_w = int((w - 2) * ratio)

    # Hintergrund + Rahmen
    g.box(x, y, x + w - 1, y + h - 1, bg)
    g.rect(x, y, x + w - 1, y + h - 1, 0xFFFFFF)
    # Fuellbalken (1 px Padding)
    if fill_w > 0:
        g.box(x + 1, y + 1, x + 1 + fill_w - 1, y + h - 2, fg)
    return None


@graphics_builtin("UI_PANEL", arity=(4, 6))
def _panel(g, *args):
    """Visueller Container - Hintergrund, Rahmen, optional Titel oben.

    Keine id, kein State. Das Spiel zeichnet seine eigenen Komponenten
    oben drauf - UI_PANEL ist nur Deko.
    """
    x = _check_int(args[0], "UI_PANEL", "x")
    y = _check_int(args[1], "UI_PANEL", "y")
    x += _state.offset_x
    y += _state.offset_y
    w = _check_int(args[2], "UI_PANEL", "w")
    h = _check_int(args[3], "UI_PANEL", "h")
    title = _check_str(args[4], "UI_PANEL", "title") if len(args) >= 5 else ""
    bg = _check_color(args[5], "UI_PANEL") if len(args) >= 6 else 0x252840

    # Hintergrund + Rahmen
    g.box(x, y, x + w - 1, y + h - 1, bg)
    g.rect(x, y, x + w - 1, y + h - 1, 0x60607A)
    # Titel-Bar wenn Titel gesetzt
    if title:
        title_h = 18
        g.box(x, y, x + w - 1, y + title_h - 1, 0x383C5C)
        g.rect(x, y, x + w - 1, y + title_h - 1, 0x60607A)
        g.text(x + 6, y + 2, title, 0xFFFFFF)
    return None


@graphics_builtin("UI_TEXTFIELD", arity=(5, 6))
def _textfield(g, *args):
    """Text-Input. Klick fokussiert das Feld; getippte Zeichen kommen rein,
    Backspace loescht, Enter tut nichts (es ist kein Multiline-Editor).

    Returns: aktuellen Text als STRING.
    """
    id_str = _check_id(args[0], "UI_TEXTFIELD")
    x = _check_int(args[1], "UI_TEXTFIELD", "x")
    y = _check_int(args[2], "UI_TEXTFIELD", "y")
    x += _state.offset_x
    y += _state.offset_y
    w = _check_int(args[3], "UI_TEXTFIELD", "w")
    h = _check_int(args[4], "UI_TEXTFIELD", "h")
    placeholder = (
        _check_str(args[5], "UI_TEXTFIELD", "placeholder") if len(args) >= 6
        else ""
    )

    if id_str not in _state.text:
        _state.text[id_str] = ""

    mx, my = _mouse(g)
    is_down = g.mouse_button(0)
    hovered = _in_box(mx, my, x, y, w, h)

    # Klick fokussiert / blurred
    if is_down and not _state.was_mouse_down:
        if hovered:
            _state.focused = id_str
        elif _state.focused == id_str:
            _state.focused = None

    # Wenn fokussiert: Tipp-Eingaben aufnehmen
    if _state.focused == id_str:
        typed = g.pop_text_input()
        if typed:
            # Filter: kein Tab als sichtbares Zeichen ins Feld
            typed = typed.replace("\t", "")
            _state.text[id_str] += typed
        # Backspace via Edge-Detection
        cur_keys = g.keys_pressed()
        just_pressed = cur_keys - _state.prev_keys
        if _KEY_BACKSPACE in just_pressed and _state.text[id_str]:
            _state.text[id_str] = _state.text[id_str][:-1]

    # Zeichnen
    border = 0xFFFFFF if _state.focused == id_str else 0x808088
    bg = 0x1A1C2A
    g.box(x, y, x + w - 1, y + h - 1, bg)
    g.rect(x, y, x + w - 1, y + h - 1, border)

    text = _state.text[id_str]
    if text:
        g.text(x + 5, y + (h - 14) // 2, text, 0xFFFFFF)
    elif placeholder and _state.focused != id_str:
        g.text(x + 5, y + (h - 14) // 2, placeholder, 0x707080)

    # Blinkender Caret nur wenn fokussiert (32-Frame-Periode)
    if _state.focused == id_str and (_state.frame_count // 16) % 2 == 0:
        # Caret rechts vom Text, grob: x + 5 + 8*len(text). 8px ist eine
        # Schaetzung fuer die Default-Schrift; gut genug fuer einen Cursor.
        cx = x + 5 + len(text) * 8
        cx = min(cx, x + w - 3)
        g.line(cx, y + 3, cx, y + h - 4, 0xFFFFFF)

    return _state.text[id_str]


@graphics_builtin("UI_TEXTFIELD_SET", arity=2)
def _textfield_set(g, *args):
    """Setzt programmatisch den Wert eines Textfields. Nuetzlich um beim
    Bearbeiten eines Eintrags den existierenden Wert vorzubelegen."""
    id_str = _check_id(args[0], "UI_TEXTFIELD_SET")
    value = _check_str(args[1], "UI_TEXTFIELD_SET", "value")
    _state.text[id_str] = value
    return None


@graphics_builtin("UI_RADIO", arity=(4, 5))
def _radio(g, *args):
    """Radio-Button-Gruppe: vertikale Liste von Optionen, eine ausgewaehlt.

    options muss ein ARRAY OF STRING sein.  Klick auf eine Option
    waehlt sie. Returns: Index der gewaehlten Option (0-basiert).
    """
    from ..interpreter import _GBArray   # lazy, vermeidet Zirkular-Import

    id_str = _check_id(args[0], "UI_RADIO")
    x = _check_int(args[1], "UI_RADIO", "x")
    y = _check_int(args[2], "UI_RADIO", "y")
    x += _state.offset_x
    y += _state.offset_y
    options = args[3]
    if not isinstance(options, _GBArray) or options.element_type != "string":
        raise TypeMismatchError("UI_RADIO: options muss ARRAY OF STRING sein")
    n = options.total_size()
    if n == 0:
        return -1

    if id_str not in _state.radio:
        default = _check_int(args[4], "UI_RADIO", "default") if len(args) >= 5 else 0
        if default < 0 or default >= n:
            default = 0
        _state.radio[id_str] = default

    mx, my = _mouse(g)
    is_down = g.mouse_button(0)
    row_h = 18
    radius = 5

    for i in range(n):
        cy = y + i * row_h + row_h // 2
        cx = x + radius + 2
        # Rahmen-Kreis
        g.circle(cx, cy, radius, 0x1A1C2A)
        # Cycle: Edge-Detect-Klick
        in_row = _in_box(mx, my, x, y + i * row_h, 200, row_h)
        if in_row and is_down and not _state.was_mouse_down:
            _state.radio[id_str] = i
        # Aussenring
        if i == _state.radio[id_str]:
            g.circle(cx, cy, radius, 0x80C0FF)
            g.circle(cx, cy, radius - 2, 0xFFFFFF)
        else:
            g.rect(cx - radius, cy - radius, cx + radius, cy + radius, 0xC0C0C0)
        # Label rechts vom Kreis
        g.text(x + 2 * radius + 8, y + i * row_h + 2,
               options.values[i], 0xFFFFFF)

    return _state.radio[id_str]


def _check_str_array_1d(v, fn: str, name: str):
    """Pruef-Helper: 1D ARRAY OF STRING."""
    from ..interpreter import _GBArray
    if not isinstance(v, _GBArray) or v.element_type != "string":
        raise TypeMismatchError(f"{fn}: {name} muss ARRAY OF STRING sein")
    if len(v.dims) != 1:
        raise TypeMismatchError(
            f"{fn}: {name} muss 1-dimensional sein, hat {len(v.dims)} Dimensionen"
        )
    return v


def _check_str_array_2d(v, fn: str, name: str, expected_cols: int):
    from ..interpreter import _GBArray
    if not isinstance(v, _GBArray) or v.element_type != "string":
        raise TypeMismatchError(f"{fn}: {name} muss 2D ARRAY OF STRING sein")
    if len(v.dims) != 2:
        raise TypeMismatchError(
            f"{fn}: {name} muss 2-dimensional sein, hat {len(v.dims)} Dimensionen"
        )
    if v.dims[1] != expected_cols:
        raise GBRuntimeError(
            f"{fn}: {name} hat {v.dims[1]} Spalten, erwartet {expected_cols}"
        )
    return v


def _check_int_array_2d(v, fn: str, name: str, expected_rows: int, expected_cols: int):
    from ..interpreter import _GBArray
    if not isinstance(v, _GBArray) or v.element_type != "integer":
        raise TypeMismatchError(f"{fn}: {name} muss 2D ARRAY OF INTEGER sein")
    if len(v.dims) != 2:
        raise TypeMismatchError(f"{fn}: {name} muss 2-dimensional sein")
    if v.dims != (expected_rows, expected_cols):
        raise GBRuntimeError(
            f"{fn}: {name} muss [{expected_rows}, {expected_cols}] sein, "
            f"ist {list(v.dims)}"
        )
    return v


def _check_int_array_1d(v, fn: str, name: str, expected_len: int):
    from ..interpreter import _GBArray
    if not isinstance(v, _GBArray) or v.element_type != "integer":
        raise TypeMismatchError(f"{fn}: {name} muss ARRAY OF INTEGER sein")
    if len(v.dims) != 1:
        raise TypeMismatchError(f"{fn}: {name} muss 1-dimensional sein")
    if v.dims[0] != expected_len:
        raise GBRuntimeError(
            f"{fn}: {name} hat {v.dims[0]} Eintraege, erwartet {expected_len}"
        )
    return v


# Konstanten fuer Tabellen-Layout
_TBL_HEADER_H = 22
_TBL_ROW_H = 20
_TBL_SCROLL_W = 12          # Breite der Scrollbar
_TBL_PADDING = 6            # Innenabstand pro Zelle


@graphics_builtin("UI_TABLE", arity=(7, 10))
def _table(g, *args):
    """Tabelle mit fixiertem Header + scrollbarem Body.

    Zellen koennen pro Zelle in Vorder- (text) und Hintergrund-Farbe
    eingefaerbt werden. Vertikale + horizontale Scrollbalken werden
    eingeblendet wenn Content groesser als sichtbar. Mausrad scrollt
    vertikal. Klick+Drag auf einer Scrollbar verschiebt.

    cell_bg_colors: Wert -1 in einer Zelle = "kein Hintergrund zeichnen"
    (Standardverhalten); andere Werte sind RGB.

    Returns: Index der in diesem Frame geklickten Daten-Zeile (-1 wenn nichts).
    """
    id_str = _check_id(args[0], "UI_TABLE")
    x = _check_int(args[1], "UI_TABLE", "x")
    y = _check_int(args[2], "UI_TABLE", "y")
    x += _state.offset_x
    y += _state.offset_y
    w = _check_int(args[3], "UI_TABLE", "w")
    h = _check_int(args[4], "UI_TABLE", "h")
    headers = _check_str_array_1d(args[5], "UI_TABLE", "headers")
    n_cols = headers.dims[0]
    if n_cols == 0:
        raise GBRuntimeError("UI_TABLE: headers darf nicht leer sein")

    cells = _check_str_array_2d(args[6], "UI_TABLE", "cells", n_cols)
    n_rows = cells.dims[0]

    cell_colors = None
    if len(args) >= 8 and args[7] is not None:
        cell_colors = _check_int_array_2d(
            args[7], "UI_TABLE", "cell_colors", n_rows, n_cols,
        )

    # Spaltenbreiten - explizit oder gleichmaessig verteilt
    if len(args) >= 9 and args[8] is not None:
        col_widths_arr = _check_int_array_1d(
            args[8], "UI_TABLE", "col_widths", n_cols,
        )
        col_widths = [int(col_widths_arr.values[i]) for i in range(n_cols)]
    else:
        # Default: gleichmaessige Verteilung der sichtbaren Breite ohne ScrollBar
        avail = w - _TBL_SCROLL_W - 2  # 2 px Rahmen
        per = max(40, avail // n_cols)
        col_widths = [per] * n_cols

    cell_bg_colors = None
    if len(args) >= 10 and args[9] is not None:
        cell_bg_colors = _check_int_array_2d(
            args[9], "UI_TABLE", "cell_bg_colors", n_rows, n_cols,
        )

    # State initialisieren
    if id_str not in _state.tables:
        _state.tables[id_str] = {
            "scroll_x": 0, "scroll_y": 0,
            "drag_v": False, "drag_h": False,
            "drag_off": 0,
        }
    st = _state.tables[id_str]

    # Geometrie
    body_x = x + 1
    body_y = y + _TBL_HEADER_H
    body_w_raw = w - 2
    body_h_raw = h - _TBL_HEADER_H - 1

    total_w = sum(col_widths)
    total_h = n_rows * _TBL_ROW_H

    # Brauchen wir Scrollbalken?
    need_v = total_h > body_h_raw
    need_h = total_w > body_w_raw - (_TBL_SCROLL_W if need_v else 0)
    # Wenn beide gebraucht werden, korrigiert es sich gegenseitig
    if need_h and total_h > body_h_raw - _TBL_SCROLL_W:
        need_v = True
    body_w = body_w_raw - (_TBL_SCROLL_W if need_v else 0)
    body_h = body_h_raw - (_TBL_SCROLL_W if need_h else 0)

    # Scroll-Range anpassen wenn ueberlaufen oder Content geschrumpft ist
    max_scroll_y = max(0, total_h - body_h)
    max_scroll_x = max(0, total_w - body_w)
    st["scroll_y"] = min(max(0, st["scroll_y"]), max_scroll_y)
    st["scroll_x"] = min(max(0, st["scroll_x"]), max_scroll_x)

    mx, my = _mouse(g)
    is_down = g.mouse_button(0)
    just_pressed = is_down and not _state.was_mouse_down
    just_released = (not is_down) and _state.was_mouse_down

    # Mausrad: nur wenn ueber der Tabelle
    over_table = _in_box(mx, my, x, y, w, h)
    if over_table:
        wheel = g.pop_mouse_wheel()
        if wheel != 0:
            st["scroll_y"] = max(0, min(max_scroll_y,
                                         st["scroll_y"] - wheel * _TBL_ROW_H))

    # --- Vertikale Scrollbar Drag ---------------------------------
    if need_v:
        sb_x = x + body_w_raw - _TBL_SCROLL_W + 1
        sb_y = body_y
        sb_h = body_h
        # Handle-Hoehe = sichtbares Verhaeltnis * Scrollbar-Hoehe (min 16)
        handle_h = max(16, int(sb_h * (body_h / total_h)))
        # Handle-Position aus scroll_y berechnen
        scroll_ratio = st["scroll_y"] / max_scroll_y if max_scroll_y else 0
        handle_y = sb_y + int((sb_h - handle_h) * scroll_ratio)

        if just_pressed and _in_box(mx, my, sb_x, sb_y, _TBL_SCROLL_W, sb_h):
            if _in_box(mx, my, sb_x, handle_y, _TBL_SCROLL_W, handle_h):
                st["drag_v"] = True
                st["drag_off"] = my - handle_y
            else:
                # Track-Klick: Handle dorthin springen lassen
                handle_y = max(sb_y, min(sb_y + sb_h - handle_h, my - handle_h // 2))
                st["drag_v"] = True
                st["drag_off"] = handle_h // 2
                # scroll_y entsprechend
                track_pos = (handle_y - sb_y) / max(1, sb_h - handle_h)
                st["scroll_y"] = int(track_pos * max_scroll_y)
        if st["drag_v"] and is_down:
            new_handle_y = my - st["drag_off"]
            new_handle_y = max(sb_y, min(sb_y + sb_h - handle_h, new_handle_y))
            track_pos = (new_handle_y - sb_y) / max(1, sb_h - handle_h)
            st["scroll_y"] = int(track_pos * max_scroll_y)
        if just_released:
            st["drag_v"] = False
    else:
        st["drag_v"] = False

    # --- Horizontale Scrollbar Drag -------------------------------
    if need_h:
        hs_x = body_x
        hs_y = y + body_h_raw - _TBL_SCROLL_W + _TBL_HEADER_H + 1
        # Korrektur: hs_y ist relativ zum Tabellen-y
        hs_y = y + h - _TBL_SCROLL_W - 1
        hs_w = body_w
        handle_w = max(16, int(hs_w * (body_w / total_w)))
        scroll_ratio = st["scroll_x"] / max_scroll_x if max_scroll_x else 0
        handle_x = hs_x + int((hs_w - handle_w) * scroll_ratio)

        if just_pressed and _in_box(mx, my, hs_x, hs_y, hs_w, _TBL_SCROLL_W):
            if _in_box(mx, my, handle_x, hs_y, handle_w, _TBL_SCROLL_W):
                st["drag_h"] = True
                st["drag_off"] = mx - handle_x
            else:
                handle_x = max(hs_x, min(hs_x + hs_w - handle_w, mx - handle_w // 2))
                st["drag_h"] = True
                st["drag_off"] = handle_w // 2
                track_pos = (handle_x - hs_x) / max(1, hs_w - handle_w)
                st["scroll_x"] = int(track_pos * max_scroll_x)
        if st["drag_h"] and is_down:
            new_handle_x = mx - st["drag_off"]
            new_handle_x = max(hs_x, min(hs_x + hs_w - handle_w, new_handle_x))
            track_pos = (new_handle_x - hs_x) / max(1, hs_w - handle_w)
            st["scroll_x"] = int(track_pos * max_scroll_x)
        if just_released:
            st["drag_h"] = False
    else:
        st["drag_h"] = False

    # --- Zeichnen --------------------------------------------------
    # Aussenrahmen + BG
    g.box(x, y, x + w - 1, y + h - 1, 0x1A1C2A)
    g.rect(x, y, x + w - 1, y + h - 1, 0x60607A)

    # Header-Bar
    g.box(x + 1, y + 1, x + w - 2, y + _TBL_HEADER_H - 1, 0x383C5C)
    g.rect(x, y, x + w - 1, y + _TBL_HEADER_H - 1, 0x60607A)

    # Header-Zellen mit Spalten-Trennern - Clip auf Header-Breite
    g.push_clip(x + 1, y + 1, body_w_raw, _TBL_HEADER_H - 2)
    cx = body_x - st["scroll_x"]
    for c in range(n_cols):
        if cx + col_widths[c] > x + 1 and cx < x + 1 + body_w_raw:
            g.text(cx + _TBL_PADDING, y + 4,
                   headers.values[c], 0xFFFFFF)
            # Spalten-Trenner
            if c < n_cols - 1:
                tx = cx + col_widths[c]
                g.line(tx, y + 1, tx, y + _TBL_HEADER_H - 2, 0x60607A)
        cx += col_widths[c]
    g.pop_clip()

    # Body - Clipping auf den sichtbaren Bereich
    g.push_clip(body_x, body_y, body_w, body_h)

    # Hover-Erkennung: welche Zeile?
    hover_row = -1
    if (over_table and _in_box(mx, my, body_x, body_y, body_w, body_h)
            and not st["drag_v"] and not st["drag_h"]):
        hover_row = (my - body_y + st["scroll_y"]) // _TBL_ROW_H
        if hover_row < 0 or hover_row >= n_rows:
            hover_row = -1

    # Zeilen zeichnen - nur die sichtbaren
    first_visible = max(0, st["scroll_y"] // _TBL_ROW_H)
    last_visible = min(n_rows, (st["scroll_y"] + body_h) // _TBL_ROW_H + 1)
    for r in range(first_visible, last_visible):
        row_y = body_y + r * _TBL_ROW_H - st["scroll_y"]

        # Pass 1: Cell-Hintergruende (wenn fuer einzelne Zellen gesetzt)
        if cell_bg_colors is not None:
            cx = body_x - st["scroll_x"]
            for c in range(n_cols):
                cell_w = col_widths[c]
                if cx + cell_w > body_x and cx < body_x + body_w:
                    bg = int(cell_bg_colors.values[r * n_cols + c])
                    if bg >= 0:
                        g.box(cx, row_y, cx + cell_w - 1,
                              row_y + _TBL_ROW_H - 1, bg)
                cx += cell_w

        # Pass 2: Hover-Highlight ueberschreibt cell-bgs auf der hovered-Zeile
        if r == hover_row:
            g.box(body_x, row_y, body_x + body_w - 1,
                  row_y + _TBL_ROW_H - 1, 0x2A2E4A)

        # Pass 3: Zell-Text
        cx = body_x - st["scroll_x"]
        for c in range(n_cols):
            cell_w = col_widths[c]
            if cx + cell_w > body_x and cx < body_x + body_w:
                # Cell-Clipping fuer langen Text - dank echtem Clip-Stack
                # bleibt der aeussere Body-Clip nach pop_clip wirksam.
                g.push_clip(
                    max(body_x, cx + 1), row_y,
                    min(cell_w - 2, (body_x + body_w) - max(body_x, cx + 1)),
                    _TBL_ROW_H,
                )
                color = 0xFFFFFF
                if cell_colors is not None:
                    color = int(cell_colors.values[r * n_cols + c])
                txt = cells.values[r * n_cols + c]
                g.text(cx + _TBL_PADDING, row_y + 3, txt, color)
                g.pop_clip()
            cx += cell_w
        # Zeilen-Trenner
        g.line(body_x, row_y + _TBL_ROW_H - 1,
               body_x + body_w - 1, row_y + _TBL_ROW_H - 1, 0x2A2E4A)

    g.pop_clip()

    # Vertikale Scrollbar zeichnen
    if need_v:
        sb_x = x + body_w_raw - _TBL_SCROLL_W + 1
        sb_y = body_y
        sb_h = body_h
        handle_h = max(16, int(sb_h * (body_h / total_h)))
        scroll_ratio = st["scroll_y"] / max_scroll_y if max_scroll_y else 0
        handle_y = sb_y + int((sb_h - handle_h) * scroll_ratio)
        # Track
        g.box(sb_x, sb_y, sb_x + _TBL_SCROLL_W - 1,
              sb_y + sb_h - 1, 0x252840)
        # Handle
        handle_color = 0x80C0FF if st["drag_v"] else 0x60607A
        g.box(sb_x + 2, handle_y, sb_x + _TBL_SCROLL_W - 3,
              handle_y + handle_h - 1, handle_color)

    # Horizontale Scrollbar
    if need_h:
        hs_x = body_x
        hs_y = y + h - _TBL_SCROLL_W - 1
        hs_w = body_w
        handle_w = max(16, int(hs_w * (body_w / total_w)))
        scroll_ratio = st["scroll_x"] / max_scroll_x if max_scroll_x else 0
        handle_x = hs_x + int((hs_w - handle_w) * scroll_ratio)
        g.box(hs_x, hs_y, hs_x + hs_w - 1,
              hs_y + _TBL_SCROLL_W - 1, 0x252840)
        handle_color = 0x80C0FF if st["drag_h"] else 0x60607A
        g.box(handle_x, hs_y + 2, handle_x + handle_w - 1,
              hs_y + _TBL_SCROLL_W - 3, handle_color)

    # --- Klick-Erkennung auf Daten-Zeile ----------------------------
    clicked_row = -1
    if (over_table and just_pressed and hover_row >= 0
            and not _in_box(mx, my, x + body_w_raw - _TBL_SCROLL_W + 1,
                            body_y, _TBL_SCROLL_W, body_h)):
        # Origin merken - bestaetigt beim Release
        _state.click_origin = f"{id_str}#row#{hover_row}"
    if (over_table and just_released
            and isinstance(_state.click_origin, str)
            and _state.click_origin.startswith(f"{id_str}#row#")
            and hover_row >= 0):
        try:
            origin_row = int(_state.click_origin.split("#")[2])
            if origin_row == hover_row:
                clicked_row = hover_row
        except (ValueError, IndexError):
            pass

    return clicked_row


# --- Immediate-Mode-Fenster -----------------------------------------

_WIN_TITLE_H = 20
_WIN_COLLAPSE = 12       # Kantenlaenge des Einklapp-Buttons


@graphics_builtin("UI_WINDOW_BEGIN", arity=6)
def _window_begin(g, *args):
    """Beginnt ein verschiebbares Immediate-Mode-Fenster. Alle Widgets bis zum
    passenden UI_WINDOW_END() werden fenster-relativ gezeichnet (Koordinaten-
    Offset). Titelleiste: ziehen = verschieben, Klick auf den Pfeil = ein-/
    ausklappen. Rueckgabe FALSE wenn eingeklappt -> Body via `IF ... THEN`
    ueberspringen; UI_WINDOW_END() trotzdem immer aufrufen.

    Z-Order: Fenster werden in Aufrufreihenfolge gezeichnet (spaeter = oben);
    Maus-Input geht an das oberste Fenster unter dem Cursor.
    """
    id_str = _check_id(args[0], "UI_WINDOW_BEGIN")
    title = _check_str(args[1], "UI_WINDOW_BEGIN", "title")
    x = _check_int(args[2], "UI_WINDOW_BEGIN", "x")
    y = _check_int(args[3], "UI_WINDOW_BEGIN", "y")
    w = _check_int(args[4], "UI_WINDOW_BEGIN", "w")
    h = _check_int(args[5], "UI_WINDOW_BEGIN", "h")

    st = _state.windows.get(id_str)
    if st is None:
        st = {"x": x, "y": y, "collapsed": False}
        _state.windows[id_str] = st
    wx, wy = st["x"], st["y"]

    mx, my = g.mouse_x(), g.mouse_y()      # Fenster-Mgmt nutzt immer die echte Maus
    is_down = g.mouse_button(0)
    just_pressed = is_down and not _state.was_mouse_down

    # Besitzt dieses Fenster den Input? (oberstes gehovertes Fenster aus Vorframe)
    owns = (_state.active_win is None or _state.active_win == id_str)
    full_h = _WIN_TITLE_H if st["collapsed"] else h
    if _in_box(mx, my, wx, wy, w, full_h):
        _state.hover_win = id_str          # letzter Schreiber = oberstes Fenster

    cb_x = wx + 4
    cb_y = wy + (_WIN_TITLE_H - _WIN_COLLAPSE) // 2
    if owns and just_pressed:
        if _in_box(mx, my, cb_x, cb_y, _WIN_COLLAPSE, _WIN_COLLAPSE):
            st["collapsed"] = not st["collapsed"]
        elif _in_box(mx, my, wx, wy, w, _WIN_TITLE_H):
            _state.drag_win = id_str
            _state.drag_off = (mx - wx, my - wy)
    if _state.drag_win == id_str and is_down:
        wx = mx - _state.drag_off[0]
        wy = my - _state.drag_off[1]
        st["x"], st["y"] = wx, wy

    # Zeichnen: Korpus (nur wenn offen) + Titelleiste + Einklapp-Pfeil + Titel
    draw_h = _WIN_TITLE_H if st["collapsed"] else h
    g.box(wx, wy, wx + w - 1, wy + draw_h - 1, 0x1A1C2A)
    g.rect(wx, wy, wx + w - 1, wy + draw_h - 1, 0x60607A)
    g.box(wx, wy, wx + w - 1, wy + _WIN_TITLE_H - 1,
          0x2A5C72 if owns else 0x383C5C)
    g.rect(wx, wy, wx + w - 1, wy + _WIN_TITLE_H - 1, 0x60607A)
    g.text(cb_x, cb_y - 2, "+" if st["collapsed"] else "-", 0xFFFFFF)
    g.text(wx + _WIN_COLLAPSE + 8, wy + 3, title, 0xFFFFFF)

    # Offset + Input-Block pushen (Widgets bis UI_WINDOW_END sind relativ)
    _state.win_stack.append((_state.offset_x, _state.offset_y, _state.input_blocked))
    _state.offset_x = wx
    _state.offset_y = wy + _WIN_TITLE_H
    _state.input_blocked = not owns
    return not st["collapsed"]


@graphics_builtin("UI_WINDOW_END", arity=0)
def _window_end(g):
    """Schliesst das aktuelle UI_WINDOW_BEGIN -- stellt Offset + Input-Zustand
    der darueberliegenden Ebene wieder her."""
    if _state.win_stack:
        ox, oy, blk = _state.win_stack.pop()
        _state.offset_x = ox
        _state.offset_y = oy
        _state.input_blocked = blk
    return None


@graphics_builtin("UI_END_FRAME", arity=0)
def _end_frame(g):
    """Muss am Ende jedes Frames vor FLIP() aufgerufen werden.

    Speichert den aktuellen Maustaste-Status, damit Klick-Edge-Detection
    in der naechsten Iteration funktioniert. Ohne diesen Aufruf wuerde ein
    gehaltener Mausklick als kontinuierlich-klickend zaehlen.
    Speichert ebenso den aktuellen Tastatur-State fuer Edge-Detection
    in TEXTFIELD (Backspace, Enter, ...).
    """
    is_down = bool(g.mouse_button(0))
    if not is_down and _state.was_mouse_down:
        # Maustaste losgelassen -> click_origin / Fenster-Drag zuruecksetzen
        _state.click_origin = None
        _state.drag_win = None
    _state.was_mouse_down = is_down
    _state.prev_keys = g.keys_pressed()
    _state.frame_count += 1
    # Fenster-Z-Order: das in DIESEM Frame oberste Fenster unter der Maus wird
    # im NAECHSTEN Frame input-aktiv (deferred Hit-Test wie in Dear ImGui).
    _state.active_win = _state.hover_win
    _state.hover_win = None
    # Offset-/Block-Stack defensiv zuruecksetzen, falls BEGIN/END unbalanciert war.
    _state.offset_x = 0
    _state.offset_y = 0
    _state.input_blocked = False
    _state.win_stack = []
    return None


@graphics_builtin("UI_RESET", arity=0)
def _reset(g):
    """Setzt allen UI-State zurueck (Checkboxen, Slider, Textfields, Radios,
    Klick-State). Sinnvoll bei Spiel-Restart oder Wechsel des Menue-Screens.
    """
    _state.checkbox.clear()
    _state.slider.clear()
    _state.text.clear()
    _state.radio.clear()
    _state.tables.clear()
    _state.windows.clear()
    _state.win_stack = []
    _state.offset_x = 0
    _state.offset_y = 0
    _state.input_blocked = False
    _state.drag_win = None
    _state.hover_win = None
    _state.active_win = None
    _state.focused = None
    _state.was_mouse_down = False
    _state.click_origin = None
    _state.prev_keys.clear()
    _state.frame_count = 0
    return None
