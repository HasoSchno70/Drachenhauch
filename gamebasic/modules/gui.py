"""GUI-Modul fuer GameBasic - Retained-Mode-Fenster & Widgets.

Im Gegensatz zum Immediate-Mode-`ui`-Modul sind Fenster und Widgets hier
*persistente Objekte*: einmal angelegt, leben sie weiter. Pro Frame ruft das
Spiel `GUI_UPDATE()` (Maus/Tasten verarbeiten) und `GUI_DRAW()` (alle Fenster
hinten->vorne zeichnen). Events werden per Polling abgefragt
(`IF GUI_CLICKED(btn) THEN ...`).

  IMPORT "gui"
  DIM win AS GUI_WINDOW
  win = GUI_WINDOW("Einstellungen", 120, 80, 320, 220)
  GUI_WINDOW_MOVABLE(win, TRUE) : GUI_WINDOW_CLOSABLE(win, TRUE)
  DIM b_ok AS GUI_WIDGET
  b_ok = GUI_BUTTON(win, "OK", 20, 160, 90, 30)
  DIM chk AS GUI_WIDGET
  chk = GUI_CHECKBOX(win, "Vollbild", 20, 60, FALSE)

  WHILE NOT QUITREQUESTED()
      CLS(&H101828)
      GUI_UPDATE()          ' Maus/Tasten: Hover, Klick, Drag, Fokus, Z-Order
      GUI_DRAW()            ' alle Fenster hinten->vorne
      IF GUI_CLICKED(b_ok) THEN PRINT GUI_CHECKED(chk)
      FLIP() : SLEEP(16)
  WEND

Koordinaten von Widgets sind relativ zum Client-Bereich ihres Fensters
(unterhalb der Titelleiste). Fenster verschieben sich per Drag an der
Titelleiste -- die Widgets folgen automatisch.

Konstruktoren (`GUI_WINDOW`, `GUI_BUTTON`, ...) sind reine Built-ins und
brauchen keinen Graphics-Kontext. Nur `GUI_UPDATE`/`GUI_DRAW` sind
graphics_builtins (Maus-/Zeichen-Zugriff).
"""
from __future__ import annotations

from ..builtins_registry import builtin, graphics_builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type, call_funcref


# --- Layout-Metriken (zur Laufzeit via GUI_METRIC_SET aenderbar) ------
# Groessen-Metriken fliessen beim ANLEGEN eines Widgets in dessen w/h ein
# (check_size, slider_h); am besten vor dem UI-Aufbau setzen. title_h, pad,
# caret_period u.a. werden live gelesen.
METRICS = {
    "title_h": 22,          # Hoehe der Titelleiste
    "slider_h": 14,         # Hoehe eines Sliders (neue Slider)
    "check_size": 16,       # Kantenlaenge der Checkbox (neue Checkboxen)
    "slider_handle_w": 10,  # Breite des Slider-Griffs
    "caret_period": 16,     # Frames pro Caret-Blink-Halbphase
    "pad": 6,               # horizontaler Text-Innenabstand
}

# Tabellen-Layout (Retained-Mode GUI_TABLE). Fixe Metriken -- analog zum
# Immediate-Mode-UI_TABLE, damit beide Tabellen sich gleich anfuehlen.
_TBL_HEADER_H = 22          # Hoehe der fixierten Kopfzeile
_TBL_ROW_H = 20             # Hoehe einer Datenzeile
_TBL_SCROLL_W = 12          # Breite/Hoehe der Scrollbalken
_TBL_PADDING = 6            # horizontaler Text-Innenabstand pro Zelle
_TBL_MIN_COL_W = 40         # Mindest-Spaltenbreite bei Auto-Verteilung

# SDL-Keycodes (wie im ui-Modul)
_KEY_BACKSPACE = 8


# --- Theme (Logo-Cyan-Palette, konsistent zum Editor) ----------------
THEME = {
    "win_bg":        0x18222E,
    "win_border":    0x2E3C50,
    "title_bg":      0x123C50,
    "title_bg_focus": 0x1C7DA0,
    "title_fg":      0xE6F7FF,
    "widget_bg":     0x26323F,
    "widget_border": 0x46586E,
    "text_fg":       0xFFFFFF,
    "muted_fg":      0x7A8AA0,
    "accent":        0x2BC4E8,   # Cyan
    "close_hover":   0xC04848,
}

# Default-Schnappschuesse fuer GUI_RESET (stellt Theme + Metriken wieder her).
_DEFAULT_THEME = dict(THEME)
_DEFAULT_METRICS = dict(METRICS)


# --- Widget/Window-Datentypen ---------------------------------------

class _Widget:
    """Ein Widget innerhalb eines Fensters. `kind` unterscheidet die Art.

    Koordinaten (`x`, `y`) sind relativ zum Client-Bereich des Fensters.
    """
    __slots__ = (
        "kind", "window", "x", "y", "w", "h", "text", "color",
        "value", "min", "max", "checked", "placeholder",
        "clicked", "hovered", "on_click", "on_change", "ov", "tbl",
    )

    def __init__(self, kind, window, x, y, w, h):
        self.kind = kind
        self.window = window
        self.x = x
        self.y = y
        self.w = w
        self.h = h
        self.text = ""
        self.color = THEME["text_fg"]
        self.value = 0.0
        self.min = 0.0
        self.max = 1.0
        self.checked = False
        self.placeholder = ""
        self.clicked = False    # nur im Frame des Loslassens (Button)
        self.hovered = False
        self.on_click = None    # optionale FUNCREF (GUI_ON_CLICK)
        self.on_change = None   # optionale FUNCREF (GUI_ON_CHANGE)
        self.ov = None          # optionale Farb-Overrides {rolle: farbe}
        self.tbl = None         # nur fuer kind=="table": Tabellen-State (dict)

    def abs_rect(self):
        """Absolute Bildschirm-Koordinaten (x, y, w, h)."""
        win = self.window
        return (win.x + self.x, win.y + METRICS["title_h"] + self.y, self.w, self.h)

    def __repr__(self):
        return f"<GUI_WIDGET {self.kind}>"


class _Window:
    __slots__ = (
        "title", "x", "y", "w", "h", "widgets",
        "movable", "closable", "visible", "close_clicked",
    )

    def __init__(self, title, x, y, w, h):
        self.title = title
        self.x = x
        self.y = y
        self.w = w
        self.h = h
        self.widgets = []
        self.movable = True
        self.closable = False
        self.visible = True
        self.close_clicked = False

    def close_button_rect(self):
        """(x, y, w, h) des Schliess-Buttons in der Titelleiste."""
        th = METRICS["title_h"]
        return (self.x + self.w - th, self.y, th, th)

    def title_rect(self):
        return (self.x, self.y, self.w, METRICS["title_h"])

    def __repr__(self):
        return f"<GUI_WINDOW {self.title!r}>"


register_type("gui_window", _Window)
register_type("gui_widget", _Widget)


# --- Manager (Modul-Singleton) --------------------------------------

class _GuiManager:
    def __init__(self):
        self.windows = []          # Z-Order: [0]=hinten, [-1]=vorne
        self.focus_window = None
        self.focus_widget = None   # fokussiertes TextInput
        self.drag_window = None     # gerade gezogenes Fenster
        self.drag_dx = 0
        self.drag_dy = 0
        self.active_slider = None    # gerade gezogener Slider
        self.active_table = None      # Tabelle mit laufendem Scrollbar-Drag
        self.table_press = None       # (table_widget, row) eines begonnenen Zeilen-Klicks
        self.press_origin = None     # Widget, auf dem ein Klick begann (Button)
        self._pending = []           # in diesem Frame ausgeloeste Callback-FuncRefs
        self.was_mouse_down = False
        self.prev_keys = set()
        self.frame_count = 0

    # -- Hit-Test-Helfer ---------------------------------------------
    @staticmethod
    def _in(mx, my, rect):
        x, y, w, h = rect
        return x <= mx < x + w and y <= my < y + h

    def _topmost_window_at(self, mx, my):
        for win in reversed(self.windows):
            if win.visible and self._in(mx, my, (win.x, win.y, win.w, win.h)):
                return win
        return None

    def _bring_to_front(self, win):
        if self.windows and self.windows[-1] is win:
            return
        self.windows.remove(win)
        self.windows.append(win)

    # -- Update (ein Frame) ------------------------------------------
    def update(self, g):
        mx, my = g.mouse_x(), g.mouse_y()
        is_down = bool(g.mouse_button(0))
        just_pressed = is_down and not self.was_mouse_down
        just_released = (not is_down) and self.was_mouse_down

        # Transiente Flags je Frame zuruecksetzen
        for win in self.windows:
            for wdg in win.widgets:
                wdg.clicked = False
                wdg.hovered = False
                if wdg.kind == "table":
                    wdg.tbl["hover_row"] = -1
                    wdg.tbl["clicked_row"] = -1

        # Hover (nur oberstes Fenster unter der Maus)
        top = self._topmost_window_at(mx, my)
        if top is not None:
            for wdg in top.widgets:
                if self._in(mx, my, wdg.abs_rect()):
                    wdg.hovered = True
                    if wdg.kind == "table":
                        self._table_hover(g, wdg, mx, my)

        # Laufendes Fenster-Drag
        if self.drag_window is not None:
            if is_down:
                self.drag_window.x = mx - self.drag_dx
                self.drag_window.y = my - self.drag_dy
            else:
                self.drag_window = None

        # Laufendes Slider-Drag
        if self.active_slider is not None:
            if is_down:
                self._drag_slider(self.active_slider, mx)
            else:
                self.active_slider = None

        # Laufendes Tabellen-Scrollbar-Drag
        if self.active_table is not None:
            if is_down:
                self._table_drag(self.active_table, mx, my)
            else:
                t = self.active_table.tbl
                t["drag_v"] = t["drag_h"] = False
                self.active_table = None

        # Neuer Druck: Hit-Test
        if (just_pressed and self.drag_window is None
                and self.active_slider is None and self.active_table is None):
            self._handle_press(mx, my)

        # Loslassen: Button-Klick bzw. Tabellen-Zeilen-Klick bestaetigen
        if just_released:
            if self.press_origin is not None:
                ox, oy, ow, oh = self.press_origin.abs_rect()
                if (self.press_origin.kind == "button"
                        and self._in(mx, my, (ox, oy, ow, oh))):
                    self.press_origin.clicked = True
                    if self.press_origin.on_click is not None:
                        self._pending.append(self.press_origin.on_click)
            self.press_origin = None
            if self.table_press is not None:
                wdg, row = self.table_press
                if row >= 0 and wdg.tbl["hover_row"] == row:
                    wdg.tbl["selected"] = row
                    wdg.tbl["clicked_row"] = row
                    if wdg.on_change is not None:
                        self._pending.append(wdg.on_change)
                self.table_press = None

        # Tastatur fuer fokussiertes TextInput
        if self.focus_widget is not None and self.focus_widget.kind == "textinput":
            self._handle_text_input(g, self.focus_widget)

        # Frame-State sichern (Edge-Detection)
        self.was_mouse_down = is_down
        self.prev_keys = g.keys_pressed()
        self.frame_count += 1

        # Ausgeloeste Callbacks zuletzt dispatchen (FUNCREF-Bruecke). Erst die
        # Queue leeren, dann aufrufen -> waehrend eines Callbacks ausgeloeste
        # Events landen erst im naechsten Frame, keine Re-Entrancy-Schleife.
        if self._pending:
            pending = self._pending
            self._pending = []
            for fr in pending:
                call_funcref(g, fr, [])

    def _handle_press(self, mx, my):
        win = self._topmost_window_at(mx, my)
        if win is None:
            self.focus_widget = None      # Klick ins Leere -> Blur
            return
        self._bring_to_front(win)
        self.focus_window = win

        # Schliess-Button?
        if win.closable and self._in(mx, my, win.close_button_rect()):
            win.close_clicked = True
            win.visible = False
            return

        # Titelleisten-Drag?
        if self._in(mx, my, win.title_rect()):
            if win.movable:
                self.drag_window = win
                self.drag_dx = mx - win.x
                self.drag_dy = my - win.y
            return

        # Widget unter der Maus
        hit = None
        for wdg in win.widgets:
            if self._in(mx, my, wdg.abs_rect()):
                hit = wdg
                break

        if hit is None:
            self.focus_widget = None
            return

        if hit.kind == "button":
            self.press_origin = hit
        elif hit.kind == "checkbox":
            hit.checked = not hit.checked
            if hit.on_click is not None:
                self._pending.append(hit.on_click)
            if hit.on_change is not None:
                self._pending.append(hit.on_change)
        elif hit.kind == "slider":
            self.active_slider = hit
            self._drag_slider(hit, mx)
        elif hit.kind == "textinput":
            self.focus_widget = hit
        elif hit.kind == "table":
            self.focus_widget = None
            self._table_press(hit, mx, my)
        else:
            # Label/Panel: nicht interaktiv -> Fokus loeschen
            self.focus_widget = None

    def _drag_slider(self, wdg, mx):
        ax = wdg.window.x + wdg.x
        rel = (mx - ax) / max(1, wdg.w - 1)
        rel = max(0.0, min(1.0, rel))
        new_val = wdg.min + rel * (wdg.max - wdg.min)
        if new_val != wdg.value:
            wdg.value = new_val
            if wdg.on_change is not None:
                self._pending.append(wdg.on_change)

    def _handle_text_input(self, g, wdg):
        old = wdg.text
        typed = g.pop_text_input()
        if typed:
            wdg.text += typed.replace("\t", "")
        cur_keys = g.keys_pressed()
        just = cur_keys - self.prev_keys
        if _KEY_BACKSPACE in just and wdg.text:
            wdg.text = wdg.text[:-1]
        if wdg.text != old and wdg.on_change is not None:
            self._pending.append(wdg.on_change)

    # -- Tabelle: Interaktion ----------------------------------------
    def _table_hover(self, g, wdg, mx, my):
        """Hover-Zeile + Scroll-Clamp + Mausrad fuer eine Tabelle aktualisieren.
        Wird pro Frame fuer die Tabelle unter dem Cursor (oberstes Fenster)
        gerufen."""
        t = wdg.tbl
        gm = _table_geom(wdg)
        # Scroll an aktuelle Content-Groesse anpassen (Daten koennen schrumpfen)
        t["scroll_y"] = min(max(0, t["scroll_y"]), gm["max_scroll_y"])
        t["scroll_x"] = min(max(0, t["scroll_x"]), gm["max_scroll_x"])
        # Mausrad scrollt vertikal, solange ueber dem Body
        wheel = g.pop_mouse_wheel() if hasattr(g, "pop_mouse_wheel") else 0
        if wheel and self._in(mx, my, (gm["body_x"], gm["body_y"],
                                       gm["body_w"], gm["body_h"])):
            t["scroll_y"] = max(0, min(gm["max_scroll_y"],
                                       t["scroll_y"] - wheel * _TBL_ROW_H))
        # Hover-Zeile (nur ueber dem Body, nicht ueber Scrollbalken)
        if (not t["drag_v"] and not t["drag_h"]
                and self._in(mx, my, (gm["body_x"], gm["body_y"],
                                      gm["body_w"], gm["body_h"]))):
            hr = (my - gm["body_y"] + t["scroll_y"]) // _TBL_ROW_H
            t["hover_row"] = hr if 0 <= hr < gm["n_rows"] else -1

    def _table_press(self, wdg, mx, my):
        """Press auf eine Tabelle: Scrollbar-Griff -> Drag starten, sonst
        Zeilen-Klick vormerken (bestaetigt beim Loslassen ueber derselben
        Zeile)."""
        t = wdg.tbl
        gm = _table_geom(wdg)
        # Vertikale Scrollbar?
        if gm["need_v"]:
            sb_x = gm["ax"] + gm["body_w_raw"] - _TBL_SCROLL_W + 1
            if self._in(mx, my, (sb_x, gm["body_y"], _TBL_SCROLL_W, gm["body_h"])):
                t["drag_v"] = True
                self.active_table = wdg
                t["drag_off"] = self._table_grab_off(
                    gm, "v", my, t["scroll_y"])
                self._table_drag(wdg, mx, my)
                return
        # Horizontale Scrollbar?
        if gm["need_h"]:
            hs_y = gm["ay"] + gm["h"] - _TBL_SCROLL_W - 1
            if self._in(mx, my, (gm["body_x"], hs_y, gm["body_w"], _TBL_SCROLL_W)):
                t["drag_h"] = True
                self.active_table = wdg
                t["drag_off"] = self._table_grab_off(
                    gm, "h", mx, t["scroll_x"])
                self._table_drag(wdg, mx, my)
                return
        # Sonst: Zeilen-Klick vormerken (Selektion beim Release)
        if t["hover_row"] >= 0:
            self.table_press = (wdg, t["hover_row"])

    @staticmethod
    def _table_grab_off(gm, axis, m, scroll):
        """Greif-Offset innerhalb des Scrollbar-Griffs (Maus minus Griff-Start).
        Liefert die halbe Griff-Groesse als Naeherung (Track-Klick zentriert
        den Griff) -- der genaue Wert wird beim Drag ohnehin nachgefuehrt."""
        if axis == "v":
            track = gm["body_h"]
            handle = max(16, int(track * (gm["body_h"] / gm["total_h"]))) if gm["total_h"] else track
        else:
            track = gm["body_w"]
            handle = max(16, int(track * (gm["body_w"] / gm["total_w"]))) if gm["total_w"] else track
        return handle // 2

    def _table_drag(self, wdg, mx, my):
        """Scrollbar-Drag: Maus-Position -> scroll_x/scroll_y."""
        t = wdg.tbl
        gm = _table_geom(wdg)
        if t["drag_v"] and gm["max_scroll_y"]:
            track = gm["body_h"]
            handle = max(16, int(track * (gm["body_h"] / gm["total_h"])))
            new_y = (my - gm["body_y"]) - t["drag_off"]
            new_y = max(0, min(track - handle, new_y))
            pos = new_y / max(1, track - handle)
            t["scroll_y"] = int(pos * gm["max_scroll_y"])
        elif t["drag_h"] and gm["max_scroll_x"]:
            track = gm["body_w"]
            handle = max(16, int(track * (gm["body_w"] / gm["total_w"])))
            new_x = (mx - gm["body_x"]) - t["drag_off"]
            new_x = max(0, min(track - handle, new_x))
            pos = new_x / max(1, track - handle)
            t["scroll_x"] = int(pos * gm["max_scroll_x"])

    # -- Draw --------------------------------------------------------
    def draw(self, g):
        for win in self.windows:        # hinten -> vorne
            if win.visible:
                self._draw_window(g, win)

    def _draw_window(self, g, win):
        focused = (win is self.focus_window)
        x, y, w, h = win.x, win.y, win.w, win.h
        th = METRICS["title_h"]
        pad = METRICS["pad"]
        # Korpus
        g.box(x, y, x + w - 1, y + h - 1, THEME["win_bg"])
        g.rect(x, y, x + w - 1, y + h - 1, THEME["win_border"])
        # Titelleiste
        title_bg = THEME["title_bg_focus"] if focused else THEME["title_bg"]
        g.box(x, y, x + w - 1, y + th - 1, title_bg)
        g.text(x + pad, y + 4, win.title, THEME["title_fg"])
        # Schliess-Button
        if win.closable:
            cx, cy, cw, ch = win.close_button_rect()
            g.line(cx + 6, cy + 6, cx + cw - 7, cy + ch - 7, THEME["title_fg"])
            g.line(cx + cw - 7, cy + 6, cx + 6, cy + ch - 7, THEME["title_fg"])
        # Widgets
        for wdg in win.widgets:
            self._draw_widget(g, wdg)

    def _draw_widget(self, g, wdg):
        ax, ay, w, h = wdg.abs_rect()
        kind = wdg.kind
        pad = METRICS["pad"]
        if kind == "label":
            # Label-fg: GUI_SET_COLOR("fg") vor GUI_LABEL-Farbe (wdg.color).
            fg = wdg.ov["fg"] if (wdg.ov is not None and "fg" in wdg.ov) else wdg.color
            g.text(ax, ay, wdg.text, fg)
        elif kind == "panel":
            g.box(ax, ay, ax + w - 1, ay + h - 1, _wcol(wdg, "bg", "widget_bg"))
            g.rect(ax, ay, ax + w - 1, ay + h - 1, _wcol(wdg, "border", "widget_border"))
            if wdg.text:
                g.box(ax, ay, ax + w - 1, ay + 17, THEME["win_border"])
                g.text(ax + 5, ay + 2, wdg.text, _wcol(wdg, "fg", "title_fg"))
        elif kind == "button":
            bg = _wcol(wdg, "bg", "widget_bg")
            if self.press_origin is wdg:
                bg = _shade(bg, -30)
            elif wdg.hovered:
                bg = _shade(bg, 30)
            g.box(ax, ay, ax + w - 1, ay + h - 1, bg)
            g.rect(ax, ay, ax + w - 1, ay + h - 1, _wcol(wdg, "border", "widget_border"))
            g.text(ax + pad, ay + (h - 14) // 2, wdg.text, _wcol(wdg, "fg", "text_fg"))
        elif kind == "checkbox":
            acc = _wcol(wdg, "accent", "accent")
            g.rect(ax, ay, ax + w - 1, ay + h - 1, _wcol(wdg, "border", "widget_border"))
            if wdg.hovered:
                g.rect(ax - 1, ay - 1, ax + w, ay + h, acc)
            if wdg.checked:
                g.box(ax + 3, ay + 3, ax + w - 4, ay + h - 4, acc)
            g.text(ax + w + pad, ay, wdg.text, _wcol(wdg, "fg", "text_fg"))
        elif kind == "slider":
            handle_w = METRICS["slider_handle_w"]
            g.box(ax, ay + h // 2 - 1, ax + w - 1, ay + h // 2 + 1,
                  _wcol(wdg, "bg", "widget_bg"))
            g.rect(ax, ay, ax + w - 1, ay + h - 1, _wcol(wdg, "border", "widget_border"))
            span = wdg.max - wdg.min
            ratio = (wdg.value - wdg.min) / span if span else 0.0
            hx = ax + int(ratio * (w - handle_w))
            g.box(hx, ay, hx + handle_w - 1, ay + h - 1, _wcol(wdg, "accent", "accent"))
        elif kind == "textinput":
            focused = (wdg is self.focus_widget)
            fg = _wcol(wdg, "fg", "text_fg")
            g.box(ax, ay, ax + w - 1, ay + h - 1, _wcol(wdg, "bg", "win_bg"))
            g.rect(ax, ay, ax + w - 1, ay + h - 1,
                   _wcol(wdg, "accent", "accent") if focused
                   else _wcol(wdg, "border", "widget_border"))
            if wdg.text:
                g.text(ax + 5, ay + (h - 14) // 2, wdg.text, fg)
            elif wdg.placeholder and not focused:
                g.text(ax + 5, ay + (h - 14) // 2, wdg.placeholder,
                       THEME["muted_fg"])
            if focused and (self.frame_count // METRICS["caret_period"]) % 2 == 0:
                cx = min(ax + 5 + len(wdg.text) * 8, ax + w - 3)
                g.line(cx, ay + 3, cx, ay + h - 4, fg)
        elif kind == "table":
            self._draw_table(g, wdg)

    def _draw_table(self, g, wdg):
        """Tabelle zeichnen: fixierte Kopfzeile + scrollbarer Body, theme-aware.
        Clipping via push_clip/pop_clip; selektierte/hovered Zeile farbig."""
        t = wdg.tbl
        gm = _table_geom(wdg)
        ax, ay, w, h = gm["ax"], gm["ay"], gm["w"], gm["h"]
        n_cols, n_rows = gm["n_cols"], gm["n_rows"]
        col_widths = gm["col_widths"]
        body_x, body_y = gm["body_x"], gm["body_y"]
        body_w, body_h = gm["body_w"], gm["body_h"]

        bg = _wcol(wdg, "bg", "widget_bg")
        border = _wcol(wdg, "border", "widget_border")
        fg = _wcol(wdg, "fg", "text_fg")
        accent = _wcol(wdg, "accent", "accent")
        sel_bg = _shade(accent, -110)
        hover_bg = _shade(bg, 22)

        # Aussenrahmen + Korpus
        g.box(ax, ay, ax + w - 1, ay + h - 1, bg)
        g.rect(ax, ay, ax + w - 1, ay + h - 1, border)
        # Kopfzeile
        g.box(ax + 1, ay + 1, ax + w - 2, ay + _TBL_HEADER_H - 1,
              THEME["title_bg"])
        g.line(ax, ay + _TBL_HEADER_H - 1, ax + w - 1, ay + _TBL_HEADER_H - 1, border)

        # Kopf-Zellen (horizontal mitscrollen, auf Header-Breite geclippt)
        g.push_clip(ax + 1, ay + 1, gm["body_w_raw"], _TBL_HEADER_H - 2)
        cx = body_x - t["scroll_x"]
        for c in range(n_cols):
            if cx + col_widths[c] > ax + 1 and cx < ax + 1 + gm["body_w_raw"]:
                g.text(cx + _TBL_PADDING, ay + 4, t["headers"][c],
                       _wcol(wdg, "fg", "title_fg"))
                if c < n_cols - 1:
                    tx = cx + col_widths[c]
                    g.line(tx, ay + 1, tx, ay + _TBL_HEADER_H - 2, border)
            cx += col_widths[c]
        g.pop_clip()

        # Body
        g.push_clip(body_x, body_y, body_w, body_h)
        first = max(0, t["scroll_y"] // _TBL_ROW_H)
        last = min(n_rows, (t["scroll_y"] + body_h) // _TBL_ROW_H + 1)
        for r in range(first, last):
            row_y = body_y + r * _TBL_ROW_H - t["scroll_y"]
            if r == t["selected"]:
                g.box(body_x, row_y, body_x + body_w - 1,
                      row_y + _TBL_ROW_H - 1, sel_bg)
            if r == t["hover_row"]:
                g.box(body_x, row_y, body_x + body_w - 1,
                      row_y + _TBL_ROW_H - 1, hover_bg)
            cx = body_x - t["scroll_x"]
            row = t["rows"][r]
            for c in range(n_cols):
                cw = col_widths[c]
                if cx + cw > body_x and cx < body_x + body_w:
                    g.push_clip(
                        max(body_x, cx + 1), row_y,
                        min(cw - 2, (body_x + body_w) - max(body_x, cx + 1)),
                        _TBL_ROW_H)
                    g.text(cx + _TBL_PADDING, row_y + 3, row[c], fg)
                    g.pop_clip()
                cx += cw
            g.line(body_x, row_y + _TBL_ROW_H - 1,
                   body_x + body_w - 1, row_y + _TBL_ROW_H - 1, _shade(bg, 14))
        g.pop_clip()

        # Vertikale Scrollbar
        if gm["need_v"]:
            sb_x = ax + gm["body_w_raw"] - _TBL_SCROLL_W + 1
            track = body_h
            handle = max(16, int(track * (body_h / gm["total_h"])))
            ratio = t["scroll_y"] / gm["max_scroll_y"] if gm["max_scroll_y"] else 0
            hy = body_y + int((track - handle) * ratio)
            g.box(sb_x, body_y, sb_x + _TBL_SCROLL_W - 1, body_y + track - 1,
                  _shade(bg, -8))
            g.box(sb_x + 2, hy, sb_x + _TBL_SCROLL_W - 3, hy + handle - 1,
                  accent if t["drag_v"] else border)
        # Horizontale Scrollbar
        if gm["need_h"]:
            hs_y = ay + h - _TBL_SCROLL_W - 1
            track = body_w
            handle = max(16, int(track * (body_w / gm["total_w"])))
            ratio = t["scroll_x"] / gm["max_scroll_x"] if gm["max_scroll_x"] else 0
            hx = body_x + int((track - handle) * ratio)
            g.box(body_x, hs_y, body_x + track - 1, hs_y + _TBL_SCROLL_W - 1,
                  _shade(bg, -8))
            g.box(hx, hs_y + 2, hx + handle - 1, hs_y + _TBL_SCROLL_W - 3,
                  accent if t["drag_h"] else border)

    def reset(self):
        self.windows.clear()
        self.focus_window = None
        self.focus_widget = None
        self.drag_window = None
        self.active_slider = None
        self.active_table = None
        self.table_press = None
        self.press_origin = None
        self._pending = []
        self.was_mouse_down = False
        self.prev_keys = set()
        self.frame_count = 0
        # Theme + Metriken auf Defaults zuruecksetzen (frischer Start).
        THEME.clear()
        THEME.update(_DEFAULT_THEME)
        METRICS.clear()
        METRICS.update(_DEFAULT_METRICS)


_mgr = _GuiManager()


def _wcol(wdg, role, theme_key):
    """Farbe fuer eine Widget-Rolle: Per-Widget-Override (GUI_SET_COLOR) vor
    Theme-Default. `role` in {bg, fg, border, accent}."""
    ov = wdg.ov
    if ov is not None and role in ov:
        return ov[role]
    return THEME[theme_key]


def _shade(color, delta):
    """Hellt (delta>0) oder dunkelt (delta<0) eine RGB-Farbe ab."""
    r = max(0, min(255, ((color >> 16) & 0xFF) + delta))
    g = max(0, min(255, ((color >> 8) & 0xFF) + delta))
    b = max(0, min(255, (color & 0xFF) + delta))
    return (r << 16) | (g << 8) | b


def _table_geom(wdg):
    """Berechnet das Tabellen-Layout aus aktuellem State + Daten. Einzige
    Wahrheit fuer Hit-Test (update) und Zeichnen (draw) -- so koennen die
    beiden nicht auseinanderdriften.

    Liefert ein Dict mit Body-Rechteck, aufgeloesten Spaltenbreiten,
    Scrollbalken-Bedarf und Scroll-Maxima.
    """
    t = wdg.tbl
    ax, ay, w, h = wdg.abs_rect()
    n_cols = len(t["headers"])
    n_rows = len(t["rows"])
    cw = t["col_widths"]
    if cw and len(cw) == n_cols:
        col_widths = list(cw)
    else:
        avail = w - _TBL_SCROLL_W - 2
        per = max(_TBL_MIN_COL_W, avail // n_cols) if n_cols else 0
        col_widths = [per] * n_cols
    body_x = ax + 1
    body_y = ay + _TBL_HEADER_H
    body_w_raw = w - 2
    body_h_raw = h - _TBL_HEADER_H - 1
    total_w = sum(col_widths)
    total_h = n_rows * _TBL_ROW_H
    need_v = total_h > body_h_raw
    need_h = total_w > body_w_raw - (_TBL_SCROLL_W if need_v else 0)
    if need_h and total_h > body_h_raw - _TBL_SCROLL_W:
        need_v = True
    body_w = body_w_raw - (_TBL_SCROLL_W if need_v else 0)
    body_h = body_h_raw - (_TBL_SCROLL_W if need_h else 0)
    return {
        "ax": ax, "ay": ay, "w": w, "h": h,
        "n_cols": n_cols, "n_rows": n_rows, "col_widths": col_widths,
        "body_x": body_x, "body_y": body_y, "body_w": body_w, "body_h": body_h,
        "body_w_raw": body_w_raw, "body_h_raw": body_h_raw,
        "total_w": total_w, "total_h": total_h,
        "need_v": need_v, "need_h": need_h,
        "max_scroll_y": max(0, total_h - body_h),
        "max_scroll_x": max(0, total_w - body_w),
    }


def _str_array_1d(v, fn, name):
    """Konvertiert ein 1D ARRAY OF STRING zu einer Python-Liste."""
    from ..interpreter import _GBArray
    if (not isinstance(v, _GBArray) or v.element_type != "string"
            or len(v.dims) != 1):
        raise TypeMismatchError(f"{fn}: {name} muss 1D ARRAY OF STRING sein")
    return [v.values[i] for i in range(v.dims[0])]


def _str_array_2d(v, fn, name):
    """Konvertiert ein 2D ARRAY OF STRING zu list[list[str]]."""
    from ..interpreter import _GBArray
    if (not isinstance(v, _GBArray) or v.element_type != "string"
            or len(v.dims) != 2):
        raise TypeMismatchError(f"{fn}: {name} muss 2D ARRAY OF STRING sein")
    r, c = v.dims
    return [[v.values[i * c + j] for j in range(c)] for i in range(r)]


def _int_array_1d(v, fn, name):
    """Konvertiert ein 1D ARRAY OF INTEGER zu einer Python-Liste."""
    from ..interpreter import _GBArray
    if (not isinstance(v, _GBArray) or v.element_type != "integer"
            or len(v.dims) != 1):
        raise TypeMismatchError(f"{fn}: {name} muss 1D ARRAY OF INTEGER sein")
    return [int(v.values[i]) for i in range(v.dims[0])]


# --- Typ-Pruef-Helfer -----------------------------------------------

def _check_int(v, fn, name):
    if type(v) is not int:
        raise TypeMismatchError(f"{fn}: {name} muss INTEGER sein")
    return v


def _check_num(v, fn, name):
    t = type(v)
    if t is not int and t is not float:
        raise TypeMismatchError(f"{fn}: {name} muss Zahl sein")
    return float(v)


def _check_str(v, fn, name):
    if type(v) is not str:
        raise TypeMismatchError(f"{fn}: {name} muss STRING sein")
    return v


def _check_bool(v, fn, name):
    if type(v) is not bool:
        raise TypeMismatchError(f"{fn}: {name} muss BOOLEAN sein")
    return v


def _check_color(v, fn):
    if type(v) is not int or v < 0 or v > 0xFFFFFF:
        raise GBRuntimeError(f"{fn}: Farbe muss 0..0xFFFFFF sein")
    return v


def _win(v, fn):
    if not isinstance(v, _Window):
        raise TypeMismatchError(f"{fn}: erwartet GUI_WINDOW, erhalten "
                                f"{type(v).__name__}")
    return v


def _widget(v, fn, kind=None):
    if not isinstance(v, _Widget):
        raise TypeMismatchError(f"{fn}: erwartet GUI_WIDGET, erhalten "
                                f"{type(v).__name__}")
    if kind is not None and v.kind != kind:
        raise GBRuntimeError(f"{fn}: Widget ist kein {kind} (sondern {v.kind})")
    return v


# --- Konstruktion ----------------------------------------------------

@builtin("GUI_WINDOW", arity=5)
def _b_window(title, x, y, w, h):
    _check_str(title, "GUI_WINDOW", "titel")
    _check_int(x, "GUI_WINDOW", "x")
    _check_int(y, "GUI_WINDOW", "y")
    _check_int(w, "GUI_WINDOW", "w")
    _check_int(h, "GUI_WINDOW", "h")
    win = _Window(title, x, y, w, h)
    _mgr.windows.append(win)
    _mgr.focus_window = win
    return win


@builtin("GUI_WINDOW_MOVABLE", arity=2)
def _b_win_movable(win, flag):
    _win(win, "GUI_WINDOW_MOVABLE").movable = _check_bool(
        flag, "GUI_WINDOW_MOVABLE", "flag")
    return None


@builtin("GUI_WINDOW_CLOSABLE", arity=2)
def _b_win_closable(win, flag):
    _win(win, "GUI_WINDOW_CLOSABLE").closable = _check_bool(
        flag, "GUI_WINDOW_CLOSABLE", "flag")
    return None


@builtin("GUI_WINDOW_VISIBLE", arity=2)
def _b_win_visible(win, flag):
    w = _win(win, "GUI_WINDOW_VISIBLE")
    w.visible = _check_bool(flag, "GUI_WINDOW_VISIBLE", "flag")
    if w.visible:
        w.close_clicked = False
    return None


@builtin("GUI_WINDOW_CLOSED", arity=1)
def _b_win_closed(win):
    return _win(win, "GUI_WINDOW_CLOSED").close_clicked


def _add_widget(win, fn, kind, x, y, w, h):
    wdg = _Widget(kind, win, x, y, w, h)
    win.widgets.append(wdg)
    return wdg


@builtin("GUI_BUTTON", arity=6)
def _b_button(win, text, x, y, w, h):
    _win(win, "GUI_BUTTON")
    _check_str(text, "GUI_BUTTON", "text")
    for nm, v in (("x", x), ("y", y), ("w", w), ("h", h)):
        _check_int(v, "GUI_BUTTON", nm)
    wdg = _add_widget(win, "GUI_BUTTON", "button", x, y, w, h)
    wdg.text = text
    return wdg


@builtin("GUI_LABEL", arity=(4, 5))
def _b_label(*args):
    win = _win(args[0], "GUI_LABEL")
    text = _check_str(args[1], "GUI_LABEL", "text")
    x = _check_int(args[2], "GUI_LABEL", "x")
    y = _check_int(args[3], "GUI_LABEL", "y")
    color = _check_color(args[4], "GUI_LABEL") if len(args) == 5 else THEME["text_fg"]
    wdg = _add_widget(win, "GUI_LABEL", "label", x, y, max(1, len(text) * 8), 16)
    wdg.text = text
    wdg.color = color
    return wdg


@builtin("GUI_CHECKBOX", arity=(4, 5))
def _b_checkbox(*args):
    win = _win(args[0], "GUI_CHECKBOX")
    label = _check_str(args[1], "GUI_CHECKBOX", "label")
    x = _check_int(args[2], "GUI_CHECKBOX", "x")
    y = _check_int(args[3], "GUI_CHECKBOX", "y")
    default = _check_bool(args[4], "GUI_CHECKBOX", "default") if len(args) == 5 else False
    cs = METRICS["check_size"]
    wdg = _add_widget(win, "GUI_CHECKBOX", "checkbox", x, y, cs, cs)
    wdg.text = label
    wdg.checked = default
    return wdg


@builtin("GUI_SLIDER", arity=(6, 7))
def _b_slider(*args):
    win = _win(args[0], "GUI_SLIDER")
    x = _check_int(args[1], "GUI_SLIDER", "x")
    y = _check_int(args[2], "GUI_SLIDER", "y")
    w = _check_int(args[3], "GUI_SLIDER", "w")
    mn = _check_num(args[4], "GUI_SLIDER", "min")
    mx = _check_num(args[5], "GUI_SLIDER", "max")
    if mx <= mn:
        raise GBRuntimeError("GUI_SLIDER: max muss > min sein")
    default = _check_num(args[6], "GUI_SLIDER", "default") if len(args) == 7 else mn
    wdg = _add_widget(win, "GUI_SLIDER", "slider", x, y, w, METRICS["slider_h"])
    wdg.min = mn
    wdg.max = mx
    wdg.value = max(mn, min(mx, default))
    return wdg


@builtin("GUI_PANEL", arity=(5, 6))
def _b_panel(*args):
    win = _win(args[0], "GUI_PANEL")
    x = _check_int(args[1], "GUI_PANEL", "x")
    y = _check_int(args[2], "GUI_PANEL", "y")
    w = _check_int(args[3], "GUI_PANEL", "w")
    h = _check_int(args[4], "GUI_PANEL", "h")
    title = _check_str(args[5], "GUI_PANEL", "titel") if len(args) == 6 else ""
    wdg = _add_widget(win, "GUI_PANEL", "panel", x, y, w, h)
    wdg.text = title
    return wdg


@builtin("GUI_TEXTINPUT", arity=(5, 6))
def _b_textinput(*args):
    win = _win(args[0], "GUI_TEXTINPUT")
    x = _check_int(args[1], "GUI_TEXTINPUT", "x")
    y = _check_int(args[2], "GUI_TEXTINPUT", "y")
    w = _check_int(args[3], "GUI_TEXTINPUT", "w")
    h = _check_int(args[4], "GUI_TEXTINPUT", "h")
    placeholder = _check_str(args[5], "GUI_TEXTINPUT", "platzhalter") if len(args) == 6 else ""
    wdg = _add_widget(win, "GUI_TEXTINPUT", "textinput", x, y, w, h)
    wdg.placeholder = placeholder
    return wdg


# --- Tabelle (Retained-Mode) ----------------------------------------

def _new_table_state():
    return {
        "headers": [], "rows": [], "col_widths": None,
        "scroll_x": 0, "scroll_y": 0,
        "drag_v": False, "drag_h": False, "drag_off": 0,
        "selected": -1, "hover_row": -1, "clicked_row": -1,
    }


def _table_set_rows(t, rows, fn):
    """Setzt die Datenzeilen + passt Selektion an die neue Zeilenzahl an."""
    n_cols = len(t["headers"])
    if rows and n_cols and len(rows[0]) != n_cols:
        raise GBRuntimeError(
            f"{fn}: Zeilen haben {len(rows[0])} Spalten, "
            f"Header hat {n_cols}")
    t["rows"] = rows
    if t["selected"] >= len(rows):
        t["selected"] = -1


@builtin("GUI_TABLE", arity=(5, 7))
def _b_table(*args):
    """Persistente Tabelle als Widget. Optional koennen headers (1D ARRAY OF
    STRING) und cells (2D ARRAY OF STRING) gleich uebergeben werden -- sonst
    spaeter via GUI_TABLE_HEADERS / GUI_TABLE_ROWS setzen.

    Fixierte Kopfzeile, vertikal + horizontal scrollbarer Body (Mausrad +
    Scrollbalken-Drag), Hover-Highlight, persistente Zeilen-Selektion. Klick
    auf eine Zeile selektiert sie (Polling via GUI_TABLE_CLICKED /
    GUI_TABLE_SELECTED, optional GUI_ON_CHANGE-Callback)."""
    win = _win(args[0], "GUI_TABLE")
    x = _check_int(args[1], "GUI_TABLE", "x")
    y = _check_int(args[2], "GUI_TABLE", "y")
    w = _check_int(args[3], "GUI_TABLE", "w")
    h = _check_int(args[4], "GUI_TABLE", "h")
    wdg = _add_widget(win, "GUI_TABLE", "table", x, y, w, h)
    wdg.tbl = _new_table_state()
    if len(args) == 7:
        wdg.tbl["headers"] = _str_array_1d(args[5], "GUI_TABLE", "headers")
        _table_set_rows(wdg.tbl,
                        _str_array_2d(args[6], "GUI_TABLE", "cells"), "GUI_TABLE")
    elif len(args) == 6:
        raise GBRuntimeError(
            "GUI_TABLE: entweder ohne Daten oder mit headers UND cells aufrufen")
    return wdg


@builtin("GUI_TABLE_HEADERS", arity=2)
def _b_table_headers(wdg, headers):
    """Setzt die Spalten-Kopfzeilen (1D ARRAY OF STRING)."""
    w = _widget(wdg, "GUI_TABLE_HEADERS", "table")
    w.tbl["headers"] = _str_array_1d(headers, "GUI_TABLE_HEADERS", "headers")
    return None


@builtin("GUI_TABLE_ROWS", arity=2)
def _b_table_rows(wdg, cells):
    """Setzt die Datenzeilen (2D ARRAY OF STRING). Spaltenzahl muss zu den
    Headern passen (falls gesetzt). Selektion wird angepasst, wenn sie
    ausserhalb der neuen Zeilenzahl liegt."""
    w = _widget(wdg, "GUI_TABLE_ROWS", "table")
    _table_set_rows(w.tbl, _str_array_2d(cells, "GUI_TABLE_ROWS", "cells"),
                    "GUI_TABLE_ROWS")
    return None


@builtin("GUI_TABLE_COL_WIDTHS", arity=2)
def _b_table_col_widths(wdg, widths):
    """Setzt explizite Spaltenbreiten (1D ARRAY OF INTEGER). Laenge muss der
    Header-Anzahl entsprechen. NIL/leer -> gleichmaessige Auto-Verteilung."""
    w = _widget(wdg, "GUI_TABLE_COL_WIDTHS", "table")
    if widths is None:
        w.tbl["col_widths"] = None
        return None
    cols = _int_array_1d(widths, "GUI_TABLE_COL_WIDTHS", "widths")
    n_cols = len(w.tbl["headers"])
    if n_cols and len(cols) != n_cols:
        raise GBRuntimeError(
            f"GUI_TABLE_COL_WIDTHS: {len(cols)} Breiten, Header hat {n_cols} Spalten")
    w.tbl["col_widths"] = cols
    return None


@builtin("GUI_TABLE_SELECTED", arity=1)
def _b_table_selected(wdg):
    """Liefert den Index der selektierten Zeile (-1 = keine)."""
    return _widget(wdg, "GUI_TABLE_SELECTED", "table").tbl["selected"]


@builtin("GUI_TABLE_SET_SELECTED", arity=2)
def _b_table_set_selected(wdg, row):
    """Selektiert eine Zeile programmatisch. -1 = Selektion aufheben; Werte
    ausserhalb der Zeilenzahl werden auf -1 gesetzt."""
    w = _widget(wdg, "GUI_TABLE_SET_SELECTED", "table")
    r = _check_int(row, "GUI_TABLE_SET_SELECTED", "row")
    w.tbl["selected"] = r if 0 <= r < len(w.tbl["rows"]) else -1
    return None


@builtin("GUI_TABLE_CLICKED", arity=1)
def _b_table_clicked(wdg):
    """Liefert den Index der in DIESEM Frame angeklickten Zeile (-1 = keine).
    Nur im Frame des Klicks gesetzt -- fuers Polling pro Frame abfragen."""
    return _widget(wdg, "GUI_TABLE_CLICKED", "table").tbl["clicked_row"]


@builtin("GUI_TABLE_ROW_COUNT", arity=1)
def _b_table_row_count(wdg):
    """Liefert die Anzahl der Datenzeilen."""
    return len(_widget(wdg, "GUI_TABLE_ROW_COUNT", "table").tbl["rows"])


# --- Frame: Update + Draw (graphics_builtins) -----------------------

@graphics_builtin("GUI_UPDATE", arity=0)
def _g_update(g):
    _mgr.update(g)
    return None


@graphics_builtin("GUI_DRAW", arity=0)
def _g_draw(g):
    _mgr.draw(g)
    return None


# --- Polling / Zustand ----------------------------------------------

@builtin("GUI_CLICKED", arity=1)
def _b_clicked(wdg):
    return _widget(wdg, "GUI_CLICKED").clicked


@builtin("GUI_HOVERED", arity=1)
def _b_hovered(wdg):
    return _widget(wdg, "GUI_HOVERED").hovered


@builtin("GUI_CHECKED", arity=1)
def _b_checked(wdg):
    return _widget(wdg, "GUI_CHECKED", "checkbox").checked


@builtin("GUI_VALUE", arity=1)
def _b_value(wdg):
    return _widget(wdg, "GUI_VALUE", "slider").value


@builtin("GUI_TEXT", arity=1)
def _b_text(wdg):
    return _widget(wdg, "GUI_TEXT").text


@builtin("GUI_SET_TEXT", arity=2)
def _b_set_text(wdg, text):
    w = _widget(wdg, "GUI_SET_TEXT")
    w.text = _check_str(text, "GUI_SET_TEXT", "text")
    return None


@builtin("GUI_SET_CHECKED", arity=2)
def _b_set_checked(wdg, flag):
    w = _widget(wdg, "GUI_SET_CHECKED", "checkbox")
    w.checked = _check_bool(flag, "GUI_SET_CHECKED", "flag")
    return None


@builtin("GUI_SET_VALUE", arity=2)
def _b_set_value(wdg, value):
    w = _widget(wdg, "GUI_SET_VALUE", "slider")
    v = _check_num(value, "GUI_SET_VALUE", "value")
    w.value = max(w.min, min(w.max, v))
    return None


@builtin("GUI_ON_CLICK", arity=2)
def _b_on_click(wdg, handler):
    """Registriert eine FUNCREF, die bei Klick auf das Widget aufgerufen wird
    (Button-Klick bzw. Checkbox-Toggle). Der Handler ist parameterlos und wird
    am Frame-Ende in GUI_UPDATE() aufgerufen. NIL als handler entfernt den
    Callback. Polling (GUI_CLICKED) und Callback schliessen sich nicht aus."""
    from ..interpreter import _FuncRef
    w = _widget(wdg, "GUI_ON_CLICK")
    if handler is None:
        w.on_click = None
        return None
    if not isinstance(handler, _FuncRef):
        raise TypeMismatchError("GUI_ON_CLICK: handler muss FUNCREF sein")
    w.on_click = handler
    return None


@builtin("GUI_ON_CHANGE", arity=2)
def _b_on_change(wdg, handler):
    """Registriert eine FUNCREF, die aufgerufen wird, wenn sich der Wert des
    Widgets aendert: Slider-Wert (waehrend des Ziehens), TextInput-Text (beim
    Tippen/Loeschen), Checkbox-Zustand (beim Toggle) oder Tabellen-Selektion
    (Klick auf eine Zeile). Parameterloser Handler, am Frame-Ende von
    GUI_UPDATE() aufgerufen; NIL entfernt ihn. Stand drin abfragen via
    GUI_VALUE/GUI_TEXT/GUI_CHECKED/GUI_TABLE_SELECTED."""
    from ..interpreter import _FuncRef
    w = _widget(wdg, "GUI_ON_CHANGE")
    if w.kind not in ("slider", "textinput", "checkbox", "table"):
        raise GBRuntimeError(
            "GUI_ON_CHANGE: nur fuer slider, textinput, checkbox oder table")
    if handler is None:
        w.on_change = None
        return None
    if not isinstance(handler, _FuncRef):
        raise TypeMismatchError("GUI_ON_CHANGE: handler muss FUNCREF sein")
    w.on_change = handler
    return None


# --- Theme / Metriken / Per-Widget-Farben ---------------------------

_COLOR_ROLES = ("bg", "fg", "border", "accent")


@builtin("GUI_THEME", arity=1)
def _b_theme(accent):
    """Setzt die Akzentfarbe (Kurzform fuer GUI_THEME_SET("accent", ...))."""
    THEME["accent"] = _check_color(accent, "GUI_THEME")
    return None


@builtin("GUI_THEME_SET", arity=2)
def _b_theme_set(key, color):
    """Setzt eine einzelne Theme-Farbe global. Gueltige Schluessel:
    win_bg, win_border, title_bg, title_bg_focus, title_fg, widget_bg,
    widget_border, text_fg, muted_fg, accent, close_hover."""
    k = _check_str(key, "GUI_THEME_SET", "key")
    if k not in THEME:
        raise GBRuntimeError(
            f"GUI_THEME_SET: unbekannter Schluessel '{k}' "
            f"(gueltig: {', '.join(sorted(THEME))})")
    THEME[k] = _check_color(color, "GUI_THEME_SET")
    return None


@builtin("GUI_THEME_GET", arity=1)
def _b_theme_get(key):
    """Liefert die aktuelle Theme-Farbe zu einem Schluessel (INTEGER)."""
    k = _check_str(key, "GUI_THEME_GET", "key")
    if k not in THEME:
        raise GBRuntimeError(f"GUI_THEME_GET: unbekannter Schluessel '{k}'")
    return THEME[k]


@builtin("GUI_METRIC_SET", arity=2)
def _b_metric_set(key, value):
    """Setzt eine Layout-Metrik global. Gueltige Schluessel: title_h,
    slider_h, check_size, slider_handle_w, caret_period, pad. Groessen
    (check_size, slider_h) wirken nur auf NEU angelegte Widgets -- am besten
    vor dem UI-Aufbau setzen."""
    k = _check_str(key, "GUI_METRIC_SET", "key")
    if k not in METRICS:
        raise GBRuntimeError(
            f"GUI_METRIC_SET: unbekannter Schluessel '{k}' "
            f"(gueltig: {', '.join(sorted(METRICS))})")
    v = _check_int(value, "GUI_METRIC_SET", "value")
    if v < 0:
        raise GBRuntimeError("GUI_METRIC_SET: Wert muss >= 0 sein")
    METRICS[k] = v
    return None


@builtin("GUI_METRIC_GET", arity=1)
def _b_metric_get(key):
    """Liefert den aktuellen Wert einer Layout-Metrik (INTEGER)."""
    k = _check_str(key, "GUI_METRIC_GET", "key")
    if k not in METRICS:
        raise GBRuntimeError(f"GUI_METRIC_GET: unbekannter Schluessel '{k}'")
    return METRICS[k]


@builtin("GUI_SET_COLOR", arity=3)
def _b_set_color(wdg, role, color):
    """Ueberschreibt eine Farbe fuer EIN Widget (vor dem Theme-Default).
    `role`: "bg", "fg", "border" oder "accent". Farbe -1 entfernt den
    Override wieder (Widget nutzt dann wieder das Theme)."""
    w = _widget(wdg, "GUI_SET_COLOR")
    r = _check_str(role, "GUI_SET_COLOR", "role")
    if r not in _COLOR_ROLES:
        raise GBRuntimeError(
            f"GUI_SET_COLOR: role muss eine von {_COLOR_ROLES} sein")
    if type(color) is not int:
        raise TypeMismatchError("GUI_SET_COLOR: Farbe muss INTEGER sein")
    if color == -1:
        if w.ov is not None:
            w.ov.pop(r, None)
        return None
    _check_color(color, "GUI_SET_COLOR")
    if w.ov is None:
        w.ov = {}
    w.ov[r] = color
    return None


@builtin("GUI_RESET", arity=0)
def _b_reset():
    """Loescht alle Fenster/Widgets UND setzt Theme + Metriken auf Defaults."""
    _mgr.reset()
    return None


# --- Preset-Farbschemata --------------------------------------------

_GUI_PRESETS = {
    "dark": dict(_DEFAULT_THEME),
    "light": {
        "win_bg": 0xF4F6F9, "win_border": 0xA8AEB6, "title_bg": 0xD0D5DC,
        "title_bg_focus": 0x2A7DE1, "title_fg": 0x202428, "widget_bg": 0xD8DCE2,
        "widget_border": 0x9AA0A8, "text_fg": 0x202428, "muted_fg": 0x90969C,
        "accent": 0x2A7DE1, "close_hover": 0xC04848,
    },
    "retro": {
        "win_bg": 0x020802, "win_border": 0x1F8C3C, "title_bg": 0x0A2A0A,
        "title_bg_focus": 0x0F4F1F, "title_fg": 0x33FF66, "widget_bg": 0x0A1A0A,
        "widget_border": 0x1F8C3C, "text_fg": 0x33FF66, "muted_fg": 0x1F8C3C,
        "accent": 0x33FF66, "close_hover": 0xC04848,
    },
    "contrast": {
        "win_bg": 0x000000, "win_border": 0xFFD400, "title_bg": 0x202000,
        "title_bg_focus": 0x4F4F00, "title_fg": 0xFFFFFF, "widget_bg": 0x000000,
        "widget_border": 0xFFD400, "text_fg": 0xFFFFFF, "muted_fg": 0xAAAAAA,
        "accent": 0xFFD400, "close_hover": 0xFF4040,
    },
}


@builtin("GUI_THEME_PRESET", arity=1)
def _b_theme_preset(name):
    """Aktiviert ein fertiges Farbschema: "dark" (Default), "light", "retro",
    "contrast"."""
    n = _check_str(name, "GUI_THEME_PRESET", "name").lower()
    preset = _GUI_PRESETS.get(n)
    if preset is None:
        raise GBRuntimeError(
            f"GUI_THEME_PRESET: unbekanntes Preset '{n}' "
            f"(gueltig: {', '.join(sorted(_GUI_PRESETS))})")
    THEME.update(preset)
    return None
