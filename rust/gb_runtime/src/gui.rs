//! Retained-Mode-GUI (Modul `gui`) -- nativer Port von
//! `gamebasic/modules/gui.py`. Persistente Fenster/Widgets; pro Frame
//! `GUI_UPDATE` (Maus/Tasten) + `GUI_DRAW` (zeichnen). Events per Polling.
//!
//! Stand: Kern portiert -- Window + Button/Label/Checkbox/Slider/TextInput/
//! Panel, Drag/Z-Order/Fokus/Close, programmierbares Theme, Polling
//! (`GUI_CLICKED`/`CHECKED`/`VALUE`/`TEXT`) UND FUNCREF-Callbacks
//! (`GUI_ON_CLICK`/`GUI_ON_CHANGE`): ausgeloeste Handler sammelt `update()` in
//! `pending`; die VM leert die Queue nach `GUI_UPDATE` und ruft sie auf.
//! **Noch nicht nativ:** `GUI_TABLE` (eigener Pass). Handles sind INTEGER:
//! Window = Index, Widget = (win<<20)|idx.
#![cfg(feature = "graphics")]

use std::collections::HashMap;

use crate::graphics::Graphics;

const KEY_BACKSPACE: i64 = 8;

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
pub enum Kind { Button, Label, Checkbox, Slider, TextInput, Panel }

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
}

pub struct Window {
    title: String,
    x: i32, y: i32, w: i32, h: i32,
    widgets: Vec<Widget>,
    movable: bool,
    closable: bool,
    visible: bool,
    close_clicked: bool,
}

pub struct Gui {
    windows: Vec<Window>,        // stabile Indizes (Handles!)
    z_order: Vec<usize>,         // Zeichen-/Hit-Reihenfolge (umordbar)
    focus_window: Option<usize>,
    focus_widget: Option<(usize, usize)>,
    drag_window: Option<usize>,
    drag_dx: i32, drag_dy: i32,
    active_slider: Option<(usize, usize)>,
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
            active_slider: None, press_origin: None,
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
            movable: true, closable: false, visible: true, close_clicked: false,
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
            on_click: None, on_change: None, ov: HashMap::new(),
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
        if !matches!(w.kind, Kind::Slider | Kind::TextInput | Kind::Checkbox) {
            return Err("GUI_ON_CHANGE: nur fuer slider, textinput oder checkbox".into());
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
            if w.visible && Self::in_rect(mx, my, (w.x, w.y, w.w, w.h)) { return Some(wi); }
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
            for wdg in win.widgets.iter_mut() { wdg.clicked = false; wdg.hovered = false; }
        }
        // Hover (nur oberstes Fenster).
        if let Some(top) = self.topmost_at(mx, my) {
            let n = self.windows[top].widgets.len();
            for i in 0..n {
                let r = { let w = &self.windows[top].widgets[i]; self.abs_rect(top, w) };
                if Self::in_rect(mx, my, r) { self.windows[top].widgets[i].hovered = true; }
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
        // Neuer Druck.
        if just_pressed && self.drag_window.is_none() && self.active_slider.is_none() {
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
            let r = { let w = &self.windows[win].widgets[i]; self.abs_rect(win, w) };
            if Self::in_rect(mx, my, r) { hit = Some(i); break; }
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
            _ => self.focus_widget = None,
        }
    }

    // --- Draw ---
    pub fn draw(&self, g: &mut Graphics) {
        for &wi in &self.z_order {
            if self.windows[wi].visible { self.draw_window(g, wi); }
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
        for (i, wdg) in win.widgets.iter().enumerate() { self.draw_widget(g, wi, i, wdg); }
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
}
