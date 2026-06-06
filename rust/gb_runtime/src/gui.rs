//! Retained-Mode-GUI (Modul `gui`) -- nativer Port von
//! `gamebasic/modules/gui.py`. Persistente Fenster/Widgets; pro Frame
//! `GUI_UPDATE` (Maus/Tasten) + `GUI_DRAW` (zeichnen). Events per Polling.
//!
//! Stand: vollstaendig portiert -- Window + Button/Label/Checkbox/Slider/
//! TextInput/Panel/Table, Drag/Z-Order/Fokus/Close, programmierbares Theme,
//! Polling (`GUI_CLICKED`/`CHECKED`/`VALUE`/`TEXT`) UND FUNCREF-Callbacks
//! (`GUI_ON_CLICK`/`GUI_ON_CHANGE`): ausgeloeste Handler sammelt `update()` in
//! `pending`; die VM leert die Queue nach `GUI_UPDATE` und ruft sie auf.
//! `GUI_TABLE` mit fixierter Kopfzeile, V/H-Scroll (Mausrad + Scrollbalken),
//! Hover-/Selektions-Highlight; Layout aus einer Quelle (`table_geom`).
//! Handles sind INTEGER: Window = Index, Widget = (win<<20)|idx.
#![cfg(feature = "graphics")]

use std::collections::HashMap;

use crate::graphics::Graphics;

const KEY_BACKSPACE: i64 = 8;

// Tabellen-Layout (analog zum Immediate-Mode-UI_TABLE).
const TBL_HEADER_H: i32 = 22;
const TBL_ROW_H: i32 = 20;
const TBL_SCROLL_W: i32 = 12;
const TBL_PADDING: i32 = 6;
const TBL_MIN_COL_W: i32 = 40;

fn default_theme() -> HashMap<String, i64> {
    [
        ("win_bg", 0x18222E), ("win_border", 0x2E3C50), ("title_bg", 0x123C50),
        ("title_bg_focus", 0x1C7DA0), ("title_fg", 0xE6F7FF), ("widget_bg", 0x26323F),
        ("widget_border", 0x46586E), ("text_fg", 0xFFFFFF), ("muted_fg", 0x7A8AA0),
        ("accent", 0x2BC4E8), ("close_hover", 0xC04848),
    ].iter().map(|(k, v)| (k.to_string(), *v as i64)).collect()
}

fn default_metrics() -> HashMap<String, i32> {
    [
        ("title_h", 22), ("slider_h", 14), ("check_size", 16),
        ("slider_handle_w", 10), ("caret_period", 16), ("pad", 6),
    ].iter().map(|(k, v)| (k.to_string(), *v as i32)).collect()
}

fn preset(name: &str) -> Option<HashMap<String, i64>> {
    let m = |pairs: &[(&str, i64)]| -> HashMap<String, i64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    };
    match name {
        "dark" => Some(default_theme()),
        "light" => Some(m(&[
            ("win_bg", 0xF4F6F9), ("win_border", 0xA8AEB6), ("title_bg", 0xD0D5DC),
            ("title_bg_focus", 0x2A7DE1), ("title_fg", 0x202428), ("widget_bg", 0xD8DCE2),
            ("widget_border", 0x9AA0A8), ("text_fg", 0x202428), ("muted_fg", 0x90969C),
            ("accent", 0x2A7DE1), ("close_hover", 0xC04848)])),
        "retro" => Some(m(&[
            ("win_bg", 0x020802), ("win_border", 0x1F8C3C), ("title_bg", 0x0A2A0A),
            ("title_bg_focus", 0x0F4F1F), ("title_fg", 0x33FF66), ("widget_bg", 0x0A1A0A),
            ("widget_border", 0x1F8C3C), ("text_fg", 0x33FF66), ("muted_fg", 0x1F8C3C),
            ("accent", 0x33FF66), ("close_hover", 0xC04848)])),
        "contrast" => Some(m(&[
            ("win_bg", 0x000000), ("win_border", 0xFFD400), ("title_bg", 0x202000),
            ("title_bg_focus", 0x4F4F00), ("title_fg", 0xFFFFFF), ("widget_bg", 0x000000),
            ("widget_border", 0xFFD400), ("text_fg", 0xFFFFFF), ("muted_fg", 0xAAAAAA),
            ("accent", 0xFFD400), ("close_hover", 0xFF4040)])),
        _ => None,
    }
}

fn shade(color: i64, delta: i32) -> i64 {
    let r = (((color >> 16) & 0xFF) as i32 + delta).clamp(0, 255);
    let g = (((color >> 8) & 0xFF) as i32 + delta).clamp(0, 255);
    let b = ((color & 0xFF) as i32 + delta).clamp(0, 255);
    ((r << 16) | (g << 8) | b) as i64
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind { Button, Label, Checkbox, Slider, TextInput, Panel, Table }

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Button => "button", Kind::Label => "label", Kind::Checkbox => "checkbox",
            Kind::Slider => "slider", Kind::TextInput => "textinput", Kind::Panel => "panel",
            Kind::Table => "table",
        }
    }
    fn from_str(s: &str) -> Option<Kind> {
        Some(match s {
            "button" => Kind::Button, "label" => Kind::Label, "checkbox" => Kind::Checkbox,
            "slider" => Kind::Slider, "textinput" => Kind::TextInput, "panel" => Kind::Panel,
            "table" => Kind::Table, _ => return None,
        })
    }
}

#[derive(Default)]
pub struct TableState {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    col_widths: Option<Vec<i32>>,
    scroll_x: i32, scroll_y: i32,
    drag_v: bool, drag_h: bool, drag_off: i32,
    selected: i32, hover_row: i32, clicked_row: i32,
}

pub struct Widget {
    kind: Kind,
    x: i32, y: i32, w: i32, h: i32,
    text: String,
    color: i64,
    value: f64, min: f64, max: f64,
    checked: bool,
    placeholder: String,
    clicked: bool,
    hovered: bool,
    on_click: Option<String>,
    on_change: Option<String>,
    ov: HashMap<String, i64>,
    tbl: Option<Box<TableState>>,   // nur fuer Kind::Table
    // Laufzeit-Lifecycle (Tombstone -- Indizes/Handles bleiben stabil): `alive`
    // = nicht zerstoert, `visible` = wird gezeichnet + interaktiv.
    alive: bool,
    visible: bool,
}

pub struct Window {
    title: String,
    x: i32, y: i32, w: i32, h: i32,
    widgets: Vec<Widget>,
    movable: bool,
    closable: bool,
    visible: bool,
    close_clicked: bool,
    alive: bool,   // Tombstone -- Fenster-Index bleibt als Handle stabil
}

// Aufgeloestes Tabellen-Layout (einzige Wahrheit fuer Hit-Test + Zeichnen).
struct TGeom {
    ax: i32, ay: i32, w: i32, h: i32,
    n_cols: usize, n_rows: usize, col_widths: Vec<i32>,
    body_x: i32, body_y: i32, body_w: i32, body_h: i32,
    body_w_raw: i32, total_w: i32, total_h: i32,
    need_v: bool, need_h: bool, max_scroll_y: i32, max_scroll_x: i32,
}

pub struct Gui {
    windows: Vec<Window>,        // stabile Indizes (Handles!)
    z_order: Vec<usize>,         // Zeichen-/Hit-Reihenfolge (umordbar)
    focus_window: Option<usize>,
    focus_widget: Option<(usize, usize)>,
    drag_window: Option<usize>,
    drag_dx: i32, drag_dy: i32,
    active_slider: Option<(usize, usize)>,
    active_table: Option<(usize, usize)>,
    table_press: Option<(usize, usize, i32)>,   // (win, widget, row)
    press_origin: Option<(usize, usize)>,
    was_mouse_down: bool,
    prev_backspace: bool,
    frame_count: i64,
    theme: HashMap<String, i64>,
    metrics: HashMap<String, i32>,
    // In diesem Frame ausgeloeste FUNCREF-Callbacks (Namen). Die VM leert die
    // Liste nach GUI_UPDATE und ruft sie auf -- so kann ein Callback nicht
    // mitten im State-Update die GUI re-entrant veraendern.
    pending: Vec<String>,
}

const WIDGET_SHIFT: i64 = 20;
const WIDGET_MASK: i64 = (1 << WIDGET_SHIFT) - 1;

impl Gui {
    pub fn new() -> Gui {
        Gui {
            windows: Vec::new(), z_order: Vec::new(),
            focus_window: None, focus_widget: None,
            drag_window: None, drag_dx: 0, drag_dy: 0,
            active_slider: None, active_table: None, table_press: None, press_origin: None,
            was_mouse_down: false, prev_backspace: false, frame_count: 0,
            theme: default_theme(), metrics: default_metrics(),
            pending: Vec::new(),
        }
    }

    /// Entnimmt die in diesem Frame ausgeloesten Callback-Namen (FIFO).
    pub fn take_pending(&mut self) -> Vec<String> { std::mem::take(&mut self.pending) }

    pub fn reset(&mut self) {
        let theme = default_theme();
        let metrics = default_metrics();
        *self = Gui::new();
        self.theme = theme;
        self.metrics = metrics;
    }

    fn m(&self, k: &str) -> i32 { *self.metrics.get(k).unwrap_or(&0) }
    fn th(&self, k: &str) -> i64 { *self.theme.get(k).unwrap_or(&0) }

    // --- Handles ---
    fn enc_widget(win: usize, idx: usize) -> i64 { ((win as i64) << WIDGET_SHIFT) | idx as i64 }
    fn dec_widget(h: i64) -> (usize, usize) { ((h >> WIDGET_SHIFT) as usize, (h & WIDGET_MASK) as usize) }

    fn win_mut(&mut self, h: i64, fn_: &str) -> Result<&mut Window, String> {
        self.windows.get_mut(h as usize).ok_or_else(|| format!("{}: ungueltiges GUI_WINDOW-Handle", fn_))
    }
    fn wdg(&self, h: i64, fn_: &str) -> Result<&Widget, String> {
        let (w, i) = Self::dec_widget(h);
        self.windows.get(w).and_then(|win| win.widgets.get(i))
            .ok_or_else(|| format!("{}: ungueltiges GUI_WIDGET-Handle", fn_))
    }
    fn wdg_mut(&mut self, h: i64, fn_: &str) -> Result<&mut Widget, String> {
        let (w, i) = Self::dec_widget(h);
        self.windows.get_mut(w).and_then(|win| win.widgets.get_mut(i))
            .ok_or_else(|| format!("{}: ungueltiges GUI_WIDGET-Handle", fn_))
    }

    // --- Konstruktion ---
    pub fn new_window(&mut self, title: String, x: i32, y: i32, w: i32, h: i32) -> i64 {
        let idx = self.windows.len();
        self.windows.push(Window {
            title, x, y, w, h, widgets: Vec::new(),
            movable: true, closable: false, visible: true, close_clicked: false, alive: true,
        });
        self.z_order.push(idx);
        self.focus_window = Some(idx);
        idx as i64
    }

    fn add_widget(&mut self, win: i64, fn_: &str, mut wdg: Widget) -> Result<i64, String> {
        let wi = win as usize;
        let w = self.windows.get_mut(wi).ok_or_else(|| format!("{}: erwartet GUI_WINDOW", fn_))?;
        wdg.color = *self.theme.get("text_fg").unwrap_or(&0xFFFFFF);
        let idx = w.widgets.len();
        w.widgets.push(wdg);
        Ok(Self::enc_widget(wi, idx))
    }

    fn blank(kind: Kind, x: i32, y: i32, w: i32, h: i32) -> Widget {
        Widget {
            kind, x, y, w, h, text: String::new(), color: 0xFFFFFF,
            value: 0.0, min: 0.0, max: 1.0, checked: false,
            placeholder: String::new(), clicked: false, hovered: false,
            on_click: None, on_change: None, ov: HashMap::new(), tbl: None,
            alive: true, visible: true,
        }
    }

    // --- Window-Flags ---
    pub fn window_movable(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_MOVABLE")?.movable = f; Ok(())
    }
    pub fn window_closable(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_CLOSABLE")?.closable = f; Ok(())
    }
    pub fn window_visible(&mut self, h: i64, f: bool) -> Result<(), String> {
        let w = self.win_mut(h, "GUI_WINDOW_VISIBLE")?;
        w.visible = f; if f { w.close_clicked = false; } Ok(())
    }
    pub fn window_closed(&self, h: i64) -> Result<bool, String> {
        Ok(self.windows.get(h as usize)
            .ok_or("GUI_WINDOW_CLOSED: ungueltiges GUI_WINDOW-Handle")?.close_clicked)
    }

    // --- Widget-Konstruktoren ---
    pub fn button(&mut self, win: i64, text: String, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Button, x, y, w, h); wd.text = text;
        self.add_widget(win, "GUI_BUTTON", wd)
    }
    pub fn label(&mut self, win: i64, text: String, x: i32, y: i32, color: Option<i64>) -> Result<i64, String> {
        let w = (text.chars().count() as i32 * 8).max(1);
        let mut wd = Self::blank(Kind::Label, x, y, w, 16); wd.text = text;
        let h = self.add_widget(win, "GUI_LABEL", wd)?;
        if let Some(c) = color { let (wi, i) = Self::dec_widget(h); self.windows[wi].widgets[i].color = c; }
        Ok(h)
    }
    pub fn checkbox(&mut self, win: i64, label: String, x: i32, y: i32, default: bool) -> Result<i64, String> {
        let cs = self.m("check_size");
        let mut wd = Self::blank(Kind::Checkbox, x, y, cs, cs); wd.text = label; wd.checked = default;
        self.add_widget(win, "GUI_CHECKBOX", wd)
    }
    pub fn slider(&mut self, win: i64, x: i32, y: i32, w: i32, mn: f64, mx: f64, default: f64) -> Result<i64, String> {
        if mx <= mn { return Err("GUI_SLIDER: max muss > min sein".into()); }
        let sh = self.m("slider_h");
        let mut wd = Self::blank(Kind::Slider, x, y, w, sh);
        wd.min = mn; wd.max = mx; wd.value = default.clamp(mn, mx);
        self.add_widget(win, "GUI_SLIDER", wd)
    }
    pub fn panel(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, title: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Panel, x, y, w, h); wd.text = title;
        self.add_widget(win, "GUI_PANEL", wd)
    }
    pub fn textinput(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, placeholder: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::TextInput, x, y, w, h); wd.placeholder = placeholder;
        self.add_widget(win, "GUI_TEXTINPUT", wd)
    }

    // --- Tabelle ---
    fn tbl_mut(&mut self, h: i64, fn_: &str) -> Result<&mut TableState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Table { return Err(format!("{}: Widget ist keine Tabelle", fn_)); }
        Ok(w.tbl.as_mut().unwrap())
    }
    fn tbl_ref(&self, h: i64, fn_: &str) -> Result<&TableState, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::Table { return Err(format!("{}: Widget ist keine Tabelle", fn_)); }
        Ok(w.tbl.as_ref().unwrap())
    }
    pub fn table(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Table, x, y, w, h);
        wd.tbl = Some(Box::new(TableState { selected: -1, hover_row: -1, clicked_row: -1, ..Default::default() }));
        self.add_widget(win, "GUI_TABLE", wd)
    }
    pub fn table_set_headers(&mut self, h: i64, headers: Vec<String>) -> Result<(), String> {
        self.tbl_mut(h, "GUI_TABLE_HEADERS")?.headers = headers; Ok(())
    }
    pub fn table_set_rows(&mut self, h: i64, rows: Vec<Vec<String>>) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_ROWS")?;
        let n_cols = t.headers.len();
        if !rows.is_empty() && n_cols != 0 && rows[0].len() != n_cols {
            return Err(format!("GUI_TABLE_ROWS: Zeilen haben {} Spalten, Header hat {}", rows[0].len(), n_cols));
        }
        t.rows = rows;
        if t.selected >= t.rows.len() as i32 { t.selected = -1; }
        Ok(())
    }
    pub fn table_set_col_widths(&mut self, h: i64, widths: Option<Vec<i32>>) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_COL_WIDTHS")?;
        if let Some(ref cw) = widths {
            let n = t.headers.len();
            if n != 0 && cw.len() != n {
                return Err(format!("GUI_TABLE_COL_WIDTHS: {} Breiten, Header hat {} Spalten", cw.len(), n));
            }
        }
        t.col_widths = widths; Ok(())
    }
    pub fn table_selected(&self, h: i64) -> Result<i64, String> { Ok(self.tbl_ref(h, "GUI_TABLE_SELECTED")?.selected as i64) }
    pub fn table_set_selected(&mut self, h: i64, row: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SET_SELECTED")?;
        let n = t.rows.len() as i64;
        t.selected = if row >= 0 && row < n { row as i32 } else { -1 };
        Ok(())
    }
    pub fn table_clicked(&self, h: i64) -> Result<i64, String> { Ok(self.tbl_ref(h, "GUI_TABLE_CLICKED")?.clicked_row as i64) }
    pub fn table_row_count(&self, h: i64) -> Result<i64, String> { Ok(self.tbl_ref(h, "GUI_TABLE_ROW_COUNT")?.rows.len() as i64) }

    fn table_geom(&self, wi: usize, idx: usize) -> TGeom {
        let w = &self.windows[wi].widgets[idx];
        let (ax, ay, ww, hh) = self.abs_rect(wi, w);
        let t = w.tbl.as_ref().unwrap();
        let n_cols = t.headers.len();
        let n_rows = t.rows.len();
        let col_widths: Vec<i32> = match &t.col_widths {
            Some(cw) if cw.len() == n_cols => cw.clone(),
            _ => {
                let avail = ww - TBL_SCROLL_W - 2;
                let per = if n_cols > 0 { (avail / n_cols as i32).max(TBL_MIN_COL_W) } else { 0 };
                vec![per; n_cols]
            }
        };
        let body_x = ax + 1;
        let body_y = ay + TBL_HEADER_H;
        let body_w_raw = ww - 2;
        let body_h_raw = hh - TBL_HEADER_H - 1;
        let total_w: i32 = col_widths.iter().sum();
        let total_h = n_rows as i32 * TBL_ROW_H;
        let mut need_v = total_h > body_h_raw;
        let need_h = total_w > body_w_raw - if need_v { TBL_SCROLL_W } else { 0 };
        if need_h && total_h > body_h_raw - TBL_SCROLL_W { need_v = true; }
        let body_w = body_w_raw - if need_v { TBL_SCROLL_W } else { 0 };
        let body_h = body_h_raw - if need_h { TBL_SCROLL_W } else { 0 };
        TGeom {
            ax, ay, w: ww, h: hh, n_cols, n_rows, col_widths,
            body_x, body_y, body_w, body_h, body_w_raw, total_w, total_h,
            need_v, need_h,
            max_scroll_y: (total_h - body_h).max(0), max_scroll_x: (total_w - body_w).max(0),
        }
    }

    fn handle_height(track: i32, visible: i32, total: i32) -> i32 {
        if total > 0 { ((track as f64 * (visible as f64 / total as f64)) as i32).max(16) } else { track }
    }

    fn table_hover(&mut self, wi: usize, idx: usize, mx: i32, my: i32, g: &mut Graphics) {
        let gm = self.table_geom(wi, idx);
        let wheel = g.pop_mouse_wheel();
        let over = Self::in_rect(mx, my, (gm.body_x, gm.body_y, gm.body_w, gm.body_h));
        let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
        t.scroll_y = t.scroll_y.clamp(0, gm.max_scroll_y);
        t.scroll_x = t.scroll_x.clamp(0, gm.max_scroll_x);
        if wheel != 0 && over {
            t.scroll_y = (t.scroll_y - wheel as i32 * TBL_ROW_H).clamp(0, gm.max_scroll_y);
        }
        if !t.drag_v && !t.drag_h && over {
            let hr = (my - gm.body_y + t.scroll_y) / TBL_ROW_H;
            t.hover_row = if hr >= 0 && hr < gm.n_rows as i32 { hr } else { -1 };
        }
    }

    fn table_press(&mut self, wi: usize, idx: usize, mx: i32, my: i32) {
        let gm = self.table_geom(wi, idx);
        if gm.need_v {
            let sb_x = gm.ax + gm.body_w_raw - TBL_SCROLL_W + 1;
            if Self::in_rect(mx, my, (sb_x, gm.body_y, TBL_SCROLL_W, gm.body_h)) {
                let off = Self::handle_height(gm.body_h, gm.body_h, gm.total_h) / 2;
                { let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap(); t.drag_v = true; t.drag_off = off; }
                self.active_table = Some((wi, idx));
                self.table_drag(wi, idx, mx, my);
                return;
            }
        }
        if gm.need_h {
            let hs_y = gm.ay + gm.h - TBL_SCROLL_W - 1;
            if Self::in_rect(mx, my, (gm.body_x, hs_y, gm.body_w, TBL_SCROLL_W)) {
                let off = Self::handle_height(gm.body_w, gm.body_w, gm.total_w) / 2;
                { let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap(); t.drag_h = true; t.drag_off = off; }
                self.active_table = Some((wi, idx));
                self.table_drag(wi, idx, mx, my);
                return;
            }
        }
        let hr = self.windows[wi].widgets[idx].tbl.as_ref().unwrap().hover_row;
        if hr >= 0 { self.table_press = Some((wi, idx, hr)); }
    }

    fn table_drag(&mut self, wi: usize, idx: usize, mx: i32, my: i32) {
        let gm = self.table_geom(wi, idx);
        let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
        if t.drag_v && gm.max_scroll_y > 0 {
            let track = gm.body_h;
            let handle = Self::handle_height(track, gm.body_h, gm.total_h);
            let new_y = ((my - gm.body_y) - t.drag_off).clamp(0, track - handle);
            let pos = new_y as f64 / (track - handle).max(1) as f64;
            t.scroll_y = (pos * gm.max_scroll_y as f64) as i32;
        } else if t.drag_h && gm.max_scroll_x > 0 {
            let track = gm.body_w;
            let handle = Self::handle_height(track, gm.body_w, gm.total_w);
            let new_x = ((mx - gm.body_x) - t.drag_off).clamp(0, track - handle);
            let pos = new_x as f64 / (track - handle).max(1) as f64;
            t.scroll_x = (pos * gm.max_scroll_x as f64) as i32;
        }
    }

    // --- Polling / Setter ---
    pub fn clicked(&self, h: i64) -> Result<bool, String> { Ok(self.wdg(h, "GUI_CLICKED")?.clicked) }
    pub fn hovered(&self, h: i64) -> Result<bool, String> { Ok(self.wdg(h, "GUI_HOVERED")?.hovered) }
    pub fn checked(&self, h: i64) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_CHECKED")?;
        if w.kind != Kind::Checkbox { return Err("GUI_CHECKED: Widget ist keine checkbox".into()); }
        Ok(w.checked)
    }
    pub fn value(&self, h: i64) -> Result<f64, String> {
        let w = self.wdg(h, "GUI_VALUE")?;
        if w.kind != Kind::Slider { return Err("GUI_VALUE: Widget ist kein slider".into()); }
        Ok(w.value)
    }
    pub fn text(&self, h: i64) -> Result<String, String> { Ok(self.wdg(h, "GUI_TEXT")?.text.clone()) }
    pub fn set_text(&mut self, h: i64, t: String) -> Result<(), String> { self.wdg_mut(h, "GUI_SET_TEXT")?.text = t; Ok(()) }
    pub fn set_checked(&mut self, h: i64, f: bool) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_CHECKED")?;
        if w.kind != Kind::Checkbox { return Err("GUI_SET_CHECKED: Widget ist keine checkbox".into()); }
        w.checked = f; Ok(())
    }
    pub fn set_value(&mut self, h: i64, v: f64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_VALUE")?;
        if w.kind != Kind::Slider { return Err("GUI_SET_VALUE: Widget ist kein slider".into()); }
        w.value = v.clamp(w.min, w.max); Ok(())
    }
    pub fn on_click(&mut self, h: i64, func: Option<String>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_CLICK")?.on_click = func; Ok(())
    }
    pub fn on_change(&mut self, h: i64, func: Option<String>) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_ON_CHANGE")?;
        if !matches!(w.kind, Kind::Slider | Kind::TextInput | Kind::Checkbox | Kind::Table) {
            return Err("GUI_ON_CHANGE: nur fuer slider, textinput, checkbox oder table".into());
        }
        w.on_change = func; Ok(())
    }
    pub fn set_color(&mut self, h: i64, role: String, color: i64) -> Result<(), String> {
        if !matches!(role.as_str(), "bg" | "fg" | "border" | "accent") {
            return Err("GUI_SET_COLOR: role muss bg/fg/border/accent sein".into());
        }
        let w = self.wdg_mut(h, "GUI_SET_COLOR")?;
        if color == -1 { w.ov.remove(&role); } else { w.ov.insert(role, color); }
        Ok(())
    }

    // --- Laufzeit-Manipulation (Geometrie / Lifecycle / Hit-Test) ---
    // Basis fuer dynamische UIs und einen WYSIWYG-Editor. Widget-Koordinaten sind
    // fenster-relativ (wie bei der Konstruktion).
    pub fn set_bounds(&mut self, h: i64, x: i32, y: i32, w: i32, ht: i32) -> Result<(), String> {
        let wd = self.wdg_mut(h, "GUI_SET_BOUNDS")?;
        wd.x = x; wd.y = y; wd.w = w.max(0); wd.h = ht.max(0); Ok(())
    }
    pub fn widget_bounds(&self, h: i64, fn_: &str) -> Result<(i32, i32, i32, i32), String> {
        let w = self.wdg(h, fn_)?; Ok((w.x, w.y, w.w, w.h))
    }
    pub fn destroy(&mut self, h: i64) -> Result<(), String> {
        let (wi, i) = Self::dec_widget(h);
        let wd = self.wdg_mut(h, "GUI_DESTROY")?;
        wd.alive = false;
        // Haengende Interaktions-Referenzen auf dieses Widget loesen.
        if self.focus_widget == Some((wi, i)) { self.focus_widget = None; }
        if self.active_slider == Some((wi, i)) { self.active_slider = None; }
        if self.press_origin == Some((wi, i)) { self.press_origin = None; }
        Ok(())
    }
    pub fn set_widget_visible(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_VISIBLE")?.visible = f; Ok(())
    }
    pub fn widget_visible(&self, h: i64) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_VISIBLE")?; Ok(w.alive && w.visible)
    }
    pub fn kind_name(&self, h: i64) -> Result<&'static str, String> {
        Ok(self.wdg(h, "GUI_KIND")?.kind.as_str())
    }
    pub fn focus(&mut self, h: i64) -> Result<(), String> {
        let (wi, i) = Self::dec_widget(h);
        self.wdg(h, "GUI_FOCUS")?;          // Handle validieren
        self.focus_widget = Some((wi, i));
        self.focus_window = Some(wi);
        self.bring_to_front(wi);
        Ok(())
    }
    /// Oberstes lebendes+sichtbares Widget am Bildschirmpunkt (Z-Order), oder -1.
    /// Liefert ein Widget-Handle (fuer Selektion im WYSIWYG-Editor).
    pub fn hit_test(&self, mx: i32, my: i32) -> i64 {
        for &wi in self.z_order.iter().rev() {
            let win = &self.windows[wi];
            if !win.alive || !win.visible { continue; }
            if !Self::in_rect(mx, my, (win.x, win.y, win.w, win.h)) { continue; }
            // innerhalb des Fensters: spaeter gezeichnete Widgets liegen oben.
            for i in (0..win.widgets.len()).rev() {
                let wd = &win.widgets[i];
                if wd.alive && wd.visible && Self::in_rect(mx, my, self.abs_rect(wi, wd)) {
                    return Self::enc_widget(wi, i);
                }
            }
            return -1;   // Fenster getroffen, aber kein Widget
        }
        -1
    }
    // Window-Geometrie / Lifecycle / Enumeration.
    pub fn window_set_bounds(&mut self, h: i64, x: i32, y: i32, w: i32, ht: i32) -> Result<(), String> {
        let win = self.win_mut(h, "GUI_WINDOW_SET_BOUNDS")?;
        win.x = x; win.y = y; win.w = w.max(0); win.h = ht.max(0); Ok(())
    }
    pub fn window_bounds(&self, h: i64) -> Result<(i32, i32, i32, i32), String> {
        let w = self.windows.get(h as usize)
            .ok_or("GUI_WINDOW_GET_*: ungueltiges GUI_WINDOW-Handle")?;
        Ok((w.x, w.y, w.w, w.h))
    }
    pub fn window_destroy(&mut self, h: i64) -> Result<(), String> {
        let win = self.win_mut(h, "GUI_WINDOW_DESTROY")?;
        win.alive = false; win.visible = false;
        let wi = h as usize;
        self.z_order.retain(|&i| i != wi);
        if self.focus_window == Some(wi) { self.focus_window = None; }
        if self.drag_window == Some(wi) { self.drag_window = None; }
        Ok(())
    }
    pub fn window_widget_count(&self, h: i64) -> Result<i64, String> {
        let w = self.windows.get(h as usize)
            .ok_or("GUI_WINDOW_WIDGET_COUNT: ungueltiges GUI_WINDOW-Handle")?;
        Ok(w.widgets.iter().filter(|wd| wd.alive).count() as i64)
    }
    /// Handle des `n`-ten LEBENDEN Widgets von Fenster `h` (Einfuege-Reihenfolge),
    /// oder -1. Fuer Enumeration/Serialisierung.
    pub fn window_widget(&self, h: i64, n: i64) -> Result<i64, String> {
        let wi = h as usize;
        let w = self.windows.get(wi)
            .ok_or("GUI_WINDOW_WIDGET: ungueltiges GUI_WINDOW-Handle")?;
        let mut k = 0i64;
        for (i, wd) in w.widgets.iter().enumerate() {
            if !wd.alive { continue; }
            if k == n { return Ok(Self::enc_widget(wi, i)); }
            k += 1;
        }
        Ok(-1)
    }

    // --- Serialisierung (Layout als JSON; Editor<->Runtime-Kreis) ---
    fn widget_json(w: &Widget) -> serde_json::Value {
        let mut o = serde_json::json!({
            "kind": w.kind.as_str(),
            "x": w.x, "y": w.y, "w": w.w, "h": w.h,
            "text": w.text, "color": w.color,
            "value": w.value, "min": w.min, "max": w.max,
            "checked": w.checked, "placeholder": w.placeholder, "visible": w.visible,
        });
        if let Some(f) = &w.on_click { o["on_click"] = serde_json::json!(f); }
        if let Some(f) = &w.on_change { o["on_change"] = serde_json::json!(f); }
        if !w.ov.is_empty() { o["ov"] = serde_json::json!(w.ov); }
        if let Some(t) = &w.tbl {
            o["table"] = serde_json::json!({
                "headers": t.headers, "rows": t.rows,
                "col_widths": t.col_widths, "selected": t.selected,
            });
        }
        o
    }
    fn widget_from_json(wj: &serde_json::Value) -> Result<Widget, String> {
        let ks = wj["kind"].as_str().ok_or("GUI_LOAD: Widget ohne 'kind'")?;
        let kind = Kind::from_str(ks).ok_or_else(|| format!("GUI_LOAD: unbekannter Widget-Typ '{}'", ks))?;
        let gi = |k: &str, d: i64| wj[k].as_i64().unwrap_or(d) as i32;
        let mut w = Self::blank(kind, gi("x", 0), gi("y", 0), gi("w", 0), gi("h", 0));
        w.text = wj["text"].as_str().unwrap_or("").to_string();
        w.color = wj["color"].as_i64().unwrap_or(0xFFFFFF);
        w.value = wj["value"].as_f64().unwrap_or(0.0);
        w.min = wj["min"].as_f64().unwrap_or(0.0);
        w.max = wj["max"].as_f64().unwrap_or(1.0);
        w.checked = wj["checked"].as_bool().unwrap_or(false);
        w.placeholder = wj["placeholder"].as_str().unwrap_or("").to_string();
        w.visible = wj["visible"].as_bool().unwrap_or(true);
        w.on_click = wj["on_click"].as_str().map(|s| s.to_string());
        w.on_change = wj["on_change"].as_str().map(|s| s.to_string());
        if let Some(ov) = wj["ov"].as_object() {
            for (k, val) in ov { if let Some(c) = val.as_i64() { w.ov.insert(k.clone(), c); } }
        }
        if kind == Kind::Table {
            let mut ts = TableState::default();
            if let Some(tj) = wj.get("table") {
                if let Some(hs) = tj["headers"].as_array() {
                    ts.headers = hs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
                }
                if let Some(rs) = tj["rows"].as_array() {
                    ts.rows = rs.iter().map(|row| row.as_array()
                        .map(|r| r.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                        .unwrap_or_default()).collect();
                }
                if let Some(cw) = tj["col_widths"].as_array() {
                    ts.col_widths = Some(cw.iter().filter_map(|x| x.as_i64().map(|n| n as i32)).collect());
                }
                ts.selected = tj["selected"].as_i64().unwrap_or(-1) as i32;
            }
            ts.hover_row = -1; ts.clicked_row = -1;
            w.tbl = Some(Box::new(ts));
        }
        Ok(w)
    }
    /// Ein Fenster (inkl. lebender Widgets) als JSON-String (GUI_SAVE/TO_JSON).
    pub fn to_json(&self, h: i64) -> Result<String, String> {
        let win = self.windows.get(h as usize)
            .filter(|w| w.alive)
            .ok_or("GUI_SAVE/GUI_TO_JSON: ungueltiges GUI_WINDOW-Handle")?;
        let widgets: Vec<serde_json::Value> =
            win.widgets.iter().filter(|w| w.alive).map(Self::widget_json).collect();
        let obj = serde_json::json!({
            "title": win.title, "x": win.x, "y": win.y, "w": win.w, "h": win.h,
            "movable": win.movable, "closable": win.closable, "visible": win.visible,
            "widgets": widgets,
        });
        serde_json::to_string_pretty(&obj).map_err(|e| format!("GUI_SAVE: {}", e))
    }
    /// Aus JSON ein neues Fenster bauen, Handle zurueck (GUI_LOAD/FROM_JSON).
    pub fn from_json(&mut self, s: &str) -> Result<i64, String> {
        let v: serde_json::Value = serde_json::from_str(s)
            .map_err(|e| format!("GUI_LOAD: ungueltiges JSON: {}", e))?;
        let gi = |k: &str, d: i64| v[k].as_i64().unwrap_or(d) as i32;
        let title = v["title"].as_str().unwrap_or("").to_string();
        let h = self.new_window(title, gi("x", 0), gi("y", 0), gi("w", 200), gi("h", 150));
        let wi = h as usize;
        self.windows[wi].movable = v["movable"].as_bool().unwrap_or(true);
        self.windows[wi].closable = v["closable"].as_bool().unwrap_or(false);
        self.windows[wi].visible = v["visible"].as_bool().unwrap_or(true);
        if let Some(arr) = v["widgets"].as_array() {
            for wj in arr {
                let wdg = Self::widget_from_json(wj)?;
                self.windows[wi].widgets.push(wdg);
            }
        }
        Ok(h)
    }

    // --- Theme / Metriken ---
    pub fn theme_accent(&mut self, c: i64) { self.theme.insert("accent".into(), c); }
    pub fn theme_set(&mut self, key: String, c: i64) -> Result<(), String> {
        if !self.theme.contains_key(&key) { return Err(format!("GUI_THEME_SET: unbekannter Schluessel '{}'", key)); }
        self.theme.insert(key, c); Ok(())
    }
    pub fn theme_get(&self, key: &str) -> Result<i64, String> {
        self.theme.get(key).copied().ok_or_else(|| format!("GUI_THEME_GET: unbekannter Schluessel '{}'", key))
    }
    pub fn metric_set(&mut self, key: String, v: i32) -> Result<(), String> {
        if !self.metrics.contains_key(&key) { return Err(format!("GUI_METRIC_SET: unbekannter Schluessel '{}'", key)); }
        if v < 0 { return Err("GUI_METRIC_SET: Wert muss >= 0 sein".into()); }
        self.metrics.insert(key, v); Ok(())
    }
    pub fn metric_get(&self, key: &str) -> Result<i32, String> {
        self.metrics.get(key).copied().ok_or_else(|| format!("GUI_METRIC_GET: unbekannter Schluessel '{}'", key))
    }
    pub fn theme_preset(&mut self, name: &str) -> Result<(), String> {
        match preset(&name.to_lowercase()) {
            Some(p) => { for (k, v) in p { self.theme.insert(k, v); } Ok(()) }
            None => Err(format!("GUI_THEME_PRESET: unbekanntes Preset '{}'", name)),
        }
    }

    // --- Geometrie ---
    fn abs_rect(&self, win: usize, w: &Widget) -> (i32, i32, i32, i32) {
        let win = &self.windows[win];
        (win.x + w.x, win.y + self.m("title_h") + w.y, w.w, w.h)
    }
    fn in_rect(mx: i32, my: i32, r: (i32, i32, i32, i32)) -> bool {
        mx >= r.0 && mx < r.0 + r.2 && my >= r.1 && my < r.1 + r.3
    }
    fn topmost_at(&self, mx: i32, my: i32) -> Option<usize> {
        for &wi in self.z_order.iter().rev() {
            let w = &self.windows[wi];
            if w.alive && w.visible && Self::in_rect(mx, my, (w.x, w.y, w.w, w.h)) { return Some(wi); }
        }
        None
    }
    fn bring_to_front(&mut self, wi: usize) {
        if self.z_order.last() == Some(&wi) { return; }
        self.z_order.retain(|&i| i != wi);
        self.z_order.push(wi);
    }

    fn wcol(&self, w: &Widget, role: &str, theme_key: &str) -> i64 {
        if let Some(c) = w.ov.get(role) { *c } else { self.th(theme_key) }
    }

    // --- Update (ein Frame) ---
    pub fn update(&mut self, g: &mut Graphics) {
        let mx = g.mouse_x() as i32;
        let my = g.mouse_y() as i32;
        let is_down = g.mouse_button(0);
        let just_pressed = is_down && !self.was_mouse_down;
        let just_released = !is_down && self.was_mouse_down;

        // Transiente Flags ruecksetzen.
        for win in self.windows.iter_mut() {
            for wdg in win.widgets.iter_mut() {
                wdg.clicked = false; wdg.hovered = false;
                if let Some(t) = wdg.tbl.as_mut() { t.hover_row = -1; t.clicked_row = -1; }
            }
        }
        // Hover (nur oberstes Fenster); Tabellen aktualisieren Scroll/Hover/Wheel.
        if let Some(top) = self.topmost_at(mx, my) {
            let n = self.windows[top].widgets.len();
            for i in 0..n {
                let (r, is_table, active) = {
                    let w = &self.windows[top].widgets[i];
                    (self.abs_rect(top, w), w.kind == Kind::Table, w.alive && w.visible)
                };
                if active && Self::in_rect(mx, my, r) {
                    self.windows[top].widgets[i].hovered = true;
                    if is_table { self.table_hover(top, i, mx, my, g); }
                }
            }
        }
        // Laufendes Fenster-Drag.
        if let Some(wi) = self.drag_window {
            if is_down {
                self.windows[wi].x = mx - self.drag_dx;
                self.windows[wi].y = my - self.drag_dy;
            } else { self.drag_window = None; }
        }
        // Laufendes Slider-Drag.
        if let Some((wi, i)) = self.active_slider {
            if is_down { self.drag_slider(wi, i, mx); } else { self.active_slider = None; }
        }
        // Laufendes Tabellen-Scrollbar-Drag.
        if let Some((wi, i)) = self.active_table {
            if is_down {
                self.table_drag(wi, i, mx, my);
            } else {
                if let Some(t) = self.windows[wi].widgets[i].tbl.as_mut() { t.drag_v = false; t.drag_h = false; }
                self.active_table = None;
            }
        }
        // Neuer Druck.
        if just_pressed && self.drag_window.is_none() && self.active_slider.is_none()
            && self.active_table.is_none() {
            self.handle_press(mx, my);
        }
        // Loslassen -> Button-Klick bestaetigen.
        if just_released {
            if let Some((wi, i)) = self.press_origin {
                let r = { let w = &self.windows[wi].widgets[i]; self.abs_rect(wi, w) };
                let w = &mut self.windows[wi].widgets[i];
                let fire = if w.kind == Kind::Button && Self::in_rect(mx, my, r) {
                    w.clicked = true; w.on_click.clone()
                } else { None };
                if let Some(f) = fire { self.pending.push(f); }
            }
            self.press_origin = None;
            // Tabellen-Zeilen-Klick bestaetigen (Selektion + on_change).
            if let Some((wi, i, row)) = self.table_press.take() {
                let hr = self.windows[wi].widgets[i].tbl.as_ref().map(|t| t.hover_row).unwrap_or(-1);
                if row >= 0 && hr == row {
                    let w = &mut self.windows[wi].widgets[i];
                    if let Some(t) = w.tbl.as_mut() { t.selected = row; t.clicked_row = row; }
                    let f = w.on_change.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
        }
        // Tastatur fuer fokussiertes TextInput (nur Backspace + Zeichen).
        if let Some((wi, i)) = self.focus_widget {
            if self.windows[wi].widgets[i].kind == Kind::TextInput {
                let before = self.windows[wi].widgets[i].text.clone();
                let typed = g.pop_text_input();
                if !typed.is_empty() {
                    let t: String = typed.chars().filter(|c| *c != '\t').collect();
                    self.windows[wi].widgets[i].text.push_str(&t);
                }
                let bs = g.key_down(KEY_BACKSPACE);
                if bs && !self.prev_backspace {
                    self.windows[wi].widgets[i].text.pop();
                }
                self.prev_backspace = bs;
                if self.windows[wi].widgets[i].text != before {
                    let f = self.windows[wi].widgets[i].on_change.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
        }
        self.was_mouse_down = is_down;
        self.frame_count += 1;
    }

    fn drag_slider(&mut self, wi: usize, i: usize, mx: i32) {
        let ax = self.windows[wi].x + self.windows[wi].widgets[i].x;
        let w = &mut self.windows[wi].widgets[i];
        let rel = ((mx - ax) as f64 / (w.w - 1).max(1) as f64).clamp(0.0, 1.0);
        let new_val = w.min + rel * (w.max - w.min);
        if new_val != w.value {
            w.value = new_val;
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    fn handle_press(&mut self, mx: i32, my: i32) {
        let win = match self.topmost_at(mx, my) {
            Some(w) => w,
            None => { self.focus_widget = None; return; }
        };
        self.bring_to_front(win);
        self.focus_window = Some(win);
        let th = self.m("title_h");
        let (wx, wy, ww) = (self.windows[win].x, self.windows[win].y, self.windows[win].w);
        // Schliess-Button?
        if self.windows[win].closable && Self::in_rect(mx, my, (wx + ww - th, wy, th, th)) {
            self.windows[win].close_clicked = true;
            self.windows[win].visible = false;
            return;
        }
        // Titelleiste -> Drag?
        if Self::in_rect(mx, my, (wx, wy, ww, th)) {
            if self.windows[win].movable {
                self.drag_window = Some(win);
                self.drag_dx = mx - wx;
                self.drag_dy = my - wy;
            }
            return;
        }
        // Widget unter der Maus.
        let mut hit = None;
        let n = self.windows[win].widgets.len();
        for i in 0..n {
            let (r, active) = { let w = &self.windows[win].widgets[i]; (self.abs_rect(win, w), w.alive && w.visible) };
            if active && Self::in_rect(mx, my, r) { hit = Some(i); break; }
        }
        let i = match hit { Some(i) => i, None => { self.focus_widget = None; return; } };
        match self.windows[win].widgets[i].kind {
            Kind::Button => self.press_origin = Some((win, i)),
            Kind::Checkbox => {
                let w = &mut self.windows[win].widgets[i];
                w.checked = !w.checked;
                let oc = w.on_click.clone();
                let och = w.on_change.clone();
                if let Some(f) = oc { self.pending.push(f); }
                if let Some(f) = och { self.pending.push(f); }
            }
            Kind::Slider => { self.active_slider = Some((win, i)); self.drag_slider(win, i, mx); }
            Kind::TextInput => self.focus_widget = Some((win, i)),
            Kind::Table => { self.focus_widget = None; self.table_press(win, i, mx, my); }
            _ => self.focus_widget = None,
        }
    }

    // --- Draw ---
    pub fn draw(&self, g: &mut Graphics) {
        for &wi in &self.z_order {
            if self.windows[wi].alive && self.windows[wi].visible { self.draw_window(g, wi); }
        }
    }

    fn draw_window(&self, g: &mut Graphics, wi: usize) {
        let win = &self.windows[wi];
        let focused = self.focus_window == Some(wi);
        let (x, y, w, h) = (win.x, win.y, win.w, win.h);
        let th = self.m("title_h");
        let pad = self.m("pad");
        g.box_fill(x, y, x + w - 1, y + h - 1, self.th("win_bg"));
        g.rect(x, y, x + w - 1, y + h - 1, self.th("win_border"));
        let title_bg = if focused { self.th("title_bg_focus") } else { self.th("title_bg") };
        g.box_fill(x, y, x + w - 1, y + th - 1, title_bg);
        g.text(x + pad, y + 4, win.title.clone(), self.th("title_fg"));
        if win.closable {
            let (cx, cy, cw, ch) = (x + w - th, y, th, th);
            g.line(cx + 6, cy + 6, cx + cw - 7, cy + ch - 7, self.th("title_fg"));
            g.line(cx + cw - 7, cy + 6, cx + 6, cy + ch - 7, self.th("title_fg"));
        }
        for (i, wdg) in win.widgets.iter().enumerate() {
            if wdg.alive && wdg.visible { self.draw_widget(g, wi, i, wdg); }
        }
    }

    fn draw_widget(&self, g: &mut Graphics, wi: usize, idx: usize, wdg: &Widget) {
        let (ax, ay, w, h) = self.abs_rect(wi, wdg);
        let pad = self.m("pad");
        match wdg.kind {
            Kind::Label => {
                let fg = wdg.ov.get("fg").copied().unwrap_or(wdg.color);
                g.text(ax, ay, wdg.text.clone(), fg);
            }
            Kind::Panel => {
                g.box_fill(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "widget_bg"));
                g.rect(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "border", "widget_border"));
                if !wdg.text.is_empty() {
                    g.box_fill(ax, ay, ax + w - 1, ay + 17, self.th("win_border"));
                    g.text(ax + 5, ay + 2, wdg.text.clone(), self.wcol(wdg, "fg", "title_fg"));
                }
            }
            Kind::Button => {
                let mut bg = self.wcol(wdg, "bg", "widget_bg");
                if self.press_origin == Some((wi, idx)) { bg = shade(bg, -30); }
                else if wdg.hovered { bg = shade(bg, 30); }
                g.box_fill(ax, ay, ax + w - 1, ay + h - 1, bg);
                g.rect(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "border", "widget_border"));
                g.text(ax + pad, ay + (h - 14) / 2, wdg.text.clone(), self.wcol(wdg, "fg", "text_fg"));
            }
            Kind::Checkbox => {
                let acc = self.wcol(wdg, "accent", "accent");
                g.rect(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "border", "widget_border"));
                if wdg.hovered { g.rect(ax - 1, ay - 1, ax + w, ay + h, acc); }
                if wdg.checked { g.box_fill(ax + 3, ay + 3, ax + w - 4, ay + h - 4, acc); }
                g.text(ax + w + pad, ay, wdg.text.clone(), self.wcol(wdg, "fg", "text_fg"));
            }
            Kind::Slider => {
                let handle_w = self.m("slider_handle_w");
                g.box_fill(ax, ay + h / 2 - 1, ax + w - 1, ay + h / 2 + 1, self.wcol(wdg, "bg", "widget_bg"));
                g.rect(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "border", "widget_border"));
                let span = wdg.max - wdg.min;
                let ratio = if span != 0.0 { (wdg.value - wdg.min) / span } else { 0.0 };
                let hx = ax + (ratio * (w - handle_w) as f64) as i32;
                g.box_fill(hx, ay, hx + handle_w - 1, ay + h - 1, self.wcol(wdg, "accent", "accent"));
            }
            Kind::TextInput => {
                let focused = self.focus_widget == Some((wi, idx));
                let fg = self.wcol(wdg, "fg", "text_fg");
                g.box_fill(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"));
                let bcol = if focused { self.wcol(wdg, "accent", "accent") } else { self.wcol(wdg, "border", "widget_border") };
                g.rect(ax, ay, ax + w - 1, ay + h - 1, bcol);
                if !wdg.text.is_empty() {
                    g.text(ax + 5, ay + (h - 14) / 2, wdg.text.clone(), fg);
                } else if !wdg.placeholder.is_empty() && !focused {
                    g.text(ax + 5, ay + (h - 14) / 2, wdg.placeholder.clone(), self.th("muted_fg"));
                }
                if focused && (self.frame_count / self.m("caret_period").max(1) as i64) % 2 == 0 {
                    let cx = (ax + 5 + wdg.text.chars().count() as i32 * 8).min(ax + w - 3);
                    g.line(cx, ay + 3, cx, ay + h - 4, fg);
                }
            }
            Kind::Table => self.draw_table(g, wi, idx),
        }
    }

    fn draw_table(&self, g: &mut Graphics, wi: usize, idx: usize) {
        let gm = self.table_geom(wi, idx);
        let wdg = &self.windows[wi].widgets[idx];
        let t = wdg.tbl.as_ref().unwrap();
        let (ax, ay, w, h) = (gm.ax, gm.ay, gm.w, gm.h);
        let (body_x, body_y, body_w, body_h) = (gm.body_x, gm.body_y, gm.body_w, gm.body_h);

        let bg = self.wcol(wdg, "bg", "widget_bg");
        let border = self.wcol(wdg, "border", "widget_border");
        let fg = self.wcol(wdg, "fg", "text_fg");
        let title_fg = self.wcol(wdg, "fg", "title_fg");
        let accent = self.wcol(wdg, "accent", "accent");
        let sel_bg = shade(accent, -110);
        let hover_bg = shade(bg, 22);

        // Aussenrahmen + Kopfzeile.
        g.box_fill(ax, ay, ax + w - 1, ay + h - 1, bg);
        g.rect(ax, ay, ax + w - 1, ay + h - 1, border);
        g.box_fill(ax + 1, ay + 1, ax + w - 2, ay + TBL_HEADER_H - 1, self.th("title_bg"));
        g.line(ax, ay + TBL_HEADER_H - 1, ax + w - 1, ay + TBL_HEADER_H - 1, border);

        // Kopf-Zellen (horizontal mitscrollend, auf Header-Breite geclippt).
        g.push_clip(ax + 1, ay + 1, gm.body_w_raw, TBL_HEADER_H - 2);
        let mut cx = body_x - t.scroll_x;
        for c in 0..gm.n_cols {
            let cw = gm.col_widths[c];
            if cx + cw > ax + 1 && cx < ax + 1 + gm.body_w_raw {
                g.text(cx + TBL_PADDING, ay + 4, t.headers[c].clone(), title_fg);
                if c < gm.n_cols - 1 { g.line(cx + cw, ay + 1, cx + cw, ay + TBL_HEADER_H - 2, border); }
            }
            cx += cw;
        }
        g.pop_clip();

        // Body.
        g.push_clip(body_x, body_y, body_w, body_h);
        let first = (t.scroll_y / TBL_ROW_H).max(0);
        let last = ((t.scroll_y + body_h) / TBL_ROW_H + 1).min(gm.n_rows as i32);
        for r in first..last {
            let row_y = body_y + r * TBL_ROW_H - t.scroll_y;
            if r == t.selected { g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + TBL_ROW_H - 1, sel_bg); }
            if r == t.hover_row { g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + TBL_ROW_H - 1, hover_bg); }
            let mut cx = body_x - t.scroll_x;
            let row = &t.rows[r as usize];
            for c in 0..gm.n_cols {
                let cw = gm.col_widths[c];
                if cx + cw > body_x && cx < body_x + body_w {
                    let clip_x = (cx + 1).max(body_x);
                    let clip_w = (cw - 2).min((body_x + body_w) - clip_x);
                    g.push_clip(clip_x, row_y, clip_w, TBL_ROW_H);
                    g.text(cx + TBL_PADDING, row_y + 3, row[c].clone(), fg);
                    g.pop_clip();
                }
                cx += cw;
            }
            g.line(body_x, row_y + TBL_ROW_H - 1, body_x + body_w - 1, row_y + TBL_ROW_H - 1, shade(bg, 14));
        }
        g.pop_clip();

        // Vertikale Scrollbar.
        if gm.need_v {
            let sb_x = ax + gm.body_w_raw - TBL_SCROLL_W + 1;
            let track = body_h;
            let handle = Self::handle_height(track, body_h, gm.total_h);
            let ratio = if gm.max_scroll_y > 0 { t.scroll_y as f64 / gm.max_scroll_y as f64 } else { 0.0 };
            let hy = body_y + ((track - handle) as f64 * ratio) as i32;
            g.box_fill(sb_x, body_y, sb_x + TBL_SCROLL_W - 1, body_y + track - 1, shade(bg, -8));
            g.box_fill(sb_x + 2, hy, sb_x + TBL_SCROLL_W - 3, hy + handle - 1, if t.drag_v { accent } else { border });
        }
        // Horizontale Scrollbar.
        if gm.need_h {
            let hs_y = ay + h - TBL_SCROLL_W - 1;
            let track = body_w;
            let handle = Self::handle_height(track, body_w, gm.total_w);
            let ratio = if gm.max_scroll_x > 0 { t.scroll_x as f64 / gm.max_scroll_x as f64 } else { 0.0 };
            let hx = body_x + ((track - handle) as f64 * ratio) as i32;
            g.box_fill(body_x, hs_y, body_x + track - 1, hs_y + TBL_SCROLL_W - 1, shade(bg, -8));
            g.box_fill(hx, hs_y + 2, hx + handle - 1, hs_y + TBL_SCROLL_W - 3, if t.drag_h { accent } else { border });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Checkbox-Klick (Press-basiert, ohne Graphics) muss togglen UND die
    // Callbacks in der Reihenfolge on_click, on_change in die pending-Queue
    // legen -- identisch zur Python-Referenz (_handle_press).
    #[test]
    fn checkbox_press_queues_callbacks_and_toggles() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 200, 200);
        let cb = g.checkbox(win, "X".into(), 10, 10, false).unwrap();
        g.on_click(cb, Some("clicked".into())).unwrap();
        g.on_change(cb, Some("toggled".into())).unwrap();
        // abs Checkbox-Rect: (10, 22+10, 16, 16) -> Mitte (18, 40).
        g.handle_press(18, 40);
        assert_eq!(g.take_pending(), vec!["clicked".to_string(), "toggled".to_string()]);
        assert!(g.checked(cb).unwrap());
        // Zweite Abfrage liefert leere Queue (FIFO geleert).
        assert!(g.take_pending().is_empty());
    }

    // Klick ins Leere (kein Fenster) loescht den Fokus und feuert nichts.
    #[test]
    fn press_empty_clears_focus_no_callback() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 100, 100);
        let _b = g.button(win, "OK".into(), 10, 10, 50, 20).unwrap();
        g.handle_press(400, 400); // weit weg
        assert!(g.take_pending().is_empty());
    }

    fn rows(n: usize) -> Vec<Vec<String>> {
        (0..n).map(|i| vec![format!("r{}", i), format!("{}", i)]).collect()
    }

    // Tabellen-Layout: 14 Zeilen ueberlaufen -> vertikale Scrollbar, korrekte
    // Body-Hoehe + Scroll-Maximum.
    #[test]
    fn table_geom_overflow_needs_vscroll() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 360, 250);
        let tbl = g.table(win, 14, 12, 332, 170).unwrap();
        g.table_set_headers(tbl, vec!["Name".into(), "Lvl".into()]).unwrap();
        g.table_set_rows(tbl, rows(14)).unwrap();
        let gm = g.table_geom(0, 0);
        assert_eq!(gm.n_rows, 14);
        assert!(gm.need_v && !gm.need_h);
        assert_eq!(gm.body_h, 170 - TBL_HEADER_H - 1);          // 147
        assert_eq!(gm.total_h, 14 * TBL_ROW_H);                  // 280
        assert_eq!(gm.max_scroll_y, gm.total_h - gm.body_h);     // 133
    }

    // Press auf eine Body-Zeile (kein Scrollbar) merkt den Zeilen-Klick vor;
    // Selektion folgt beim Release ueber derselben Zeile.
    #[test]
    fn table_row_press_then_release_selects() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 360, 250);
        let tbl = g.table(win, 14, 12, 332, 170).unwrap();
        g.table_set_headers(tbl, vec!["Name".into(), "Lvl".into()]).unwrap();
        g.table_set_rows(tbl, rows(14)).unwrap();
        // Hover-Zeile setzen (sonst von table_hover/Graphics abhaengig).
        g.windows[0].widgets[0].tbl.as_mut().unwrap().hover_row = 3;
        // Body-Koordinate (nicht Titel, nicht Scrollbar).
        g.handle_press(100, 90);
        assert_eq!(g.table_press, Some((0, 0, 3)));
        // Release ueber derselben Zeile -> Selektion + clicked_row.
        g.was_mouse_down = true;            // simulate prior press frame
        // Manuelles Release-Pendant (update braucht Graphics): Logik spiegeln.
        if let Some((wi, i, row)) = g.table_press.take() {
            let hr = g.windows[wi].widgets[i].tbl.as_ref().unwrap().hover_row;
            if row >= 0 && hr == row {
                let t = g.windows[wi].widgets[i].tbl.as_mut().unwrap();
                t.selected = row; t.clicked_row = row;
            }
        }
        assert_eq!(g.table_selected(tbl).unwrap(), 3);
        assert_eq!(g.table_clicked(tbl).unwrap(), 3);
    }
}
