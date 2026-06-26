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
const KEY_TAB: i64 = 9;
// Tastencodes (pygame/SDL) fuer die TextInput-Editierung.
const KEY_DELETE: i64 = 127;
const KEY_LEFT: i64 = 1073741904;
const KEY_RIGHT: i64 = 1073741903;
const KEY_UP: i64 = 1073741906;
const KEY_DOWN: i64 = 1073741905;
const KEY_HOME: i64 = 1073741898;
const KEY_END: i64 = 1073741901;
const KEY_ENTER: i64 = 13;
const K_A: i64 = 97;
const K_C: i64 = 99;
const K_V: i64 = 118;
const K_X: i64 = 120;

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
        ("corner_radius", 0), ("shadow", 0),
    ].iter().map(|(k, v)| (k.to_string(), *v as i32)).collect()
}

/// Metrik-Profil eines Presets. Die "modern_*"-Presets aktivieren runde Ecken +
/// Fenster-Schatten + groessere Titelleiste (-> der professionelle Look); alle
/// anderen bleiben beim flachen Default. Wird in theme_preset() angewandt.
fn preset_metrics(name: &str) -> Option<HashMap<String, i32>> {
    if name.starts_with("modern") {
        let mut m = default_metrics();
        m.insert("corner_radius".into(), 7);
        m.insert("title_h".into(), 30);
        m.insert("pad".into(), 8);
        m.insert("shadow".into(), 16);
        Some(m)
    } else {
        None
    }
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
        // Professionelle Looks (runde Ecken + Schatten via preset_metrics):
        // "modern_dark" passt zum GameBasic-Editor (Anthrazit + Cyan).
        "modern_dark" => Some(m(&[
            ("win_bg", 0x1E2630), ("win_border", 0x33414F), ("title_bg", 0x161D26),
            ("title_bg_focus", 0x223140), ("title_fg", 0xEAF2F8), ("widget_bg", 0x2A3542),
            ("widget_border", 0x3C4A5A), ("text_fg", 0xEAF2F8), ("muted_fg", 0x8493A4),
            ("accent", 0x2BC4E8), ("close_hover", 0xE03B3B)])),
        // "modern_light" -- heller, Windows-11-naher Look (Weiss/Hellgrau + Blau).
        "modern_light" => Some(m(&[
            ("win_bg", 0xFAFBFC), ("win_border", 0xD8DEE6), ("title_bg", 0xEFF2F5),
            ("title_bg_focus", 0xFFFFFF), ("title_fg", 0x1F2733), ("widget_bg", 0xFFFFFF),
            ("widget_border", 0xCBD3DD), ("text_fg", 0x1F2733), ("muted_fg", 0x8A93A0),
            ("accent", 0x2A7DE1), ("close_hover", 0xE03B3B)])),
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
pub enum Kind {
    Button, Label, Checkbox, Slider, TextInput, Panel, Table, Radio, Dropdown,
    Progress, ListBox, Image, Canvas, Separator, GroupBox, TextArea, Spinner, Splitter,
    Toolbar,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Button => "button", Kind::Label => "label", Kind::Checkbox => "checkbox",
            Kind::Slider => "slider", Kind::TextInput => "textinput", Kind::Panel => "panel",
            Kind::Table => "table", Kind::Radio => "radio", Kind::Dropdown => "dropdown",
            Kind::Progress => "progress", Kind::ListBox => "listbox", Kind::Image => "image",
            Kind::Canvas => "canvas", Kind::Separator => "separator", Kind::GroupBox => "groupbox",
            Kind::TextArea => "textarea", Kind::Spinner => "spinner", Kind::Splitter => "splitter",
            Kind::Toolbar => "toolbar",
        }
    }
    fn from_str(s: &str) -> Option<Kind> {
        Some(match s {
            "button" => Kind::Button, "label" => Kind::Label, "checkbox" => Kind::Checkbox,
            "slider" => Kind::Slider, "textinput" => Kind::TextInput, "panel" => Kind::Panel,
            "table" => Kind::Table, "radio" => Kind::Radio, "dropdown" => Kind::Dropdown,
            "progress" => Kind::Progress, "listbox" => Kind::ListBox, "image" => Kind::Image,
            "canvas" => Kind::Canvas, "separator" => Kind::Separator, "groupbox" => Kind::GroupBox,
            "textarea" => Kind::TextArea, "spinner" => Kind::Spinner, "splitter" => Kind::Splitter,
            "toolbar" => Kind::Toolbar,
            _ => return None,
        })
    }
}

const DROPDOWN_ITEM_H: i32 = 22;

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
    // Formular-Widgets: `group` = Radio-Gruppe; `items` = Dropdown-/ListBox-
    // Eintraege; `sel` = ausgewaehlter Index (Dropdown), -1 = keiner.
    group: String,
    items: Vec<String>,
    sel: i32,
    // Styling (Phase 4): `enabled` = interaktiv (sonst ausgegraut); `font` =
    // FONT-Handle (-1 = Default); `font_size` = px (0 = Default-Textgroesse).
    enabled: bool,
    font: i64,
    font_size: i32,
    // Anchoring (Reflow beim Fenster-Resize): Bitmaske L=1,R=2,T=4,B=8
    // (Default L|T = 5 = oben-links fixiert). `bx/by/bw/bh` = Basis-Rechteck,
    // gegen das relativ zur Basis-Fenstergroesse neu gelayoutet wird.
    anchor: u8,
    bx: i32, by: i32, bw: i32, bh: i32,
    // TextInput-Editierzustand (transient, nicht serialisiert): `caret` = Cursor
    // als Zeichen-Index 0..=len; `sel_anchor` = Anker der Selektion (== caret =>
    // keine Auswahl). `scroll` = horizontaler Pixel-Offset (Text laenger als Feld).
    caret: i32,
    sel_anchor: i32,
    scroll: i32,
    // Reiter-Zuordnung: -1 = immer sichtbar, sonst Index des Tab-Pages.
    tab_page: i32,
    // Hover-Hilfetext (Tooltip); leer = keiner. Erscheint nach kurzem Verweilen.
    tooltip: String,
    // Schrittweite (nur Spinner): +/- / Pfeil / Mausrad aendert value um step.
    step: f64,
}

pub struct Window {
    title: String,
    x: i32, y: i32, w: i32, h: i32,
    widgets: Vec<Widget>,
    movable: bool,
    closable: bool,
    visible: bool,
    resizable: bool,                 // am unteren-rechten Griff ziehbar?
    chrome: bool,                    // Titelleiste + Rahmen + Buttons? (aus = randlos,
                                     //   z.B. wenn die Form das OS-Fenster fuellt)
    min_w: i32, min_h: i32,          // Groessen-Grenzen (0 = keine)
    max_w: i32, max_h: i32,
    base_w: i32, base_h: i32,        // Referenzgroesse fuer Anchoring (Layout-Basis)
    close_clicked: bool,
    alive: bool,   // Tombstone -- Fenster-Index bleibt als Handle stabil
    // Menues: `in_bar=true` erscheint in der Menueleiste, sonst Kontextmenue
    // (per Rechtsklick im Fenster gezeigt).
    menus: Vec<Menu>,
    // Scrollbarer Inhalt: ist die Inhaltshoehe (aus den Widgets) groesser als der
    // sichtbare Bereich, scrollt der Inhalt (Mausrad + Scrollbalken).
    scrollable: bool,
    scroll_y: i32,
    // Reiter (Tabs): Labels + aktiver Reiter. Widgets mit `tab_page == active_tab`
    // (oder tab_page == -1 = immer sichtbar) werden gezeigt/bedient.
    tabs: Vec<String>,
    active_tab: i32,
}

struct Menu {
    label: String,
    in_bar: bool,
    items: Vec<MenuItem>,
}

struct MenuItem {
    label: String,
    separator: bool,
    enabled: bool,
    clicked: bool,     // gesetzt im Frame des Klicks (via GUI_CLICKED gelesen)
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
    resize_window: Option<usize>,        // laufender Fenster-Resize (am Griff)
    resize_dx: i32, resize_dy: i32,      // Ecke->Maus-Versatz
    active_slider: Option<(usize, usize)>,
    active_split: Option<(usize, usize)>,    // laufendes Splitter-Drag
    split_off: i32,                          // Griff-Versatz (Maus -> Balkenkante)
    open_dropdown: Option<(usize, usize)>,   // gerade aufgeklapptes Dropdown
    open_menu: Option<(usize, usize)>,       // offenes Menueleisten-Dropdown (win, menu)
    context_open: Option<(usize, usize, i32, i32)>,  // Kontextmenue (win, menu, x, y)
    was_right_down: bool,                    // Rechtsklick-Flankenerkennung
    scroll_drag: Option<usize>,              // Fenster, dessen Inhalts-Scrollbar gezogen wird
    active_table: Option<(usize, usize)>,
    table_press: Option<(usize, usize, i32)>,   // (win, widget, row)
    press_origin: Option<(usize, usize)>,
    was_mouse_down: bool,
    frame_count: i64,
    theme: HashMap<String, i64>,
    metrics: HashMap<String, i32>,
    // Benannte Styles (Stylesheet): Name -> {prop -> wert}. Props: bg/fg/border/
    // accent (Farbe), font (Handle), font_size (px). Via GUI_APPLY_STYLE auf
    // Widgets uebertragen (-> ov-Overrides + font/font_size).
    styles: HashMap<String, HashMap<String, i64>>,
    // In diesem Frame ausgeloeste FUNCREF-Callbacks (Namen). Die VM leert die
    // Liste nach GUI_UPDATE und ruft sie auf -- so kann ein Callback nicht
    // mitten im State-Update die GUI re-entrant veraendern.
    pending: Vec<String>,
    // Tooltip-Dwell: ueber welchem Widget (Fenster, Widget) ruht die Maus und seit
    // welchem frame_count? Mausbewegung/Klick setzt den Timer zurueck; nach
    // TOOLTIP_DELAY ruhenden Frames zeigt draw() den Hilfetext.
    hover_w: Option<(usize, usize)>,
    hover_frame: i64,
    hover_x: i32, hover_y: i32,
}

const WIDGET_SHIFT: i64 = 20;
const WIDGET_MASK: i64 = (1 << WIDGET_SHIFT) - 1;

// Menue-Handles: getaggt oberhalb der Widget-Range, damit sie kollisionsfrei
// von Widget-Handles unterscheidbar sind.
const MENU_FLAG: i64 = 1 << 60;
const ITEM_FLAG: i64 = 1 << 61;
const MENUBAR_H: i32 = 26;        // Hoehe der Menueleiste (unter der Titelleiste)
const MENU_ITEM_H: i32 = 24;      // Hoehe einer Dropdown-Zeile
const TABBAR_H: i32 = 30;         // Hoehe der Reiter-Leiste
const TOOLTIP_DELAY: i64 = 30;    // Ruhe-Frames bis ein Tooltip erscheint (~0.5s @60fps)

fn enc_menu(win: usize, m: usize) -> i64 { MENU_FLAG | ((win as i64) << WIDGET_SHIFT) | m as i64 }
fn dec_menu(h: i64) -> (usize, usize) {
    let x = h & !MENU_FLAG;
    ((x >> WIDGET_SHIFT) as usize, (x & WIDGET_MASK) as usize)
}
fn enc_item(win: usize, m: usize, it: usize) -> i64 {
    ITEM_FLAG | ((win as i64) << 40) | ((m as i64) << WIDGET_SHIFT) | it as i64
}
fn dec_item(h: i64) -> (usize, usize, usize) {
    let x = h & !ITEM_FLAG;
    ((x >> 40) as usize, ((x >> WIDGET_SHIFT) & WIDGET_MASK) as usize, (x & WIDGET_MASK) as usize)
}

impl Gui {
    pub fn new() -> Gui {
        Gui {
            windows: Vec::new(), z_order: Vec::new(),
            focus_window: None, focus_widget: None,
            drag_window: None, drag_dx: 0, drag_dy: 0,
            resize_window: None, resize_dx: 0, resize_dy: 0,
            active_slider: None, active_split: None, split_off: 0,
            open_dropdown: None, active_table: None, table_press: None, press_origin: None,
            open_menu: None, context_open: None, was_right_down: false,
            scroll_drag: None,
            was_mouse_down: false, frame_count: 0,
            theme: default_theme(), metrics: default_metrics(),
            styles: HashMap::new(),
            pending: Vec::new(),
            hover_w: None, hover_frame: 0, hover_x: 0, hover_y: 0,
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
            movable: true, closable: false, visible: true,
            resizable: false, chrome: true, min_w: 0, min_h: 0, max_w: 0, max_h: 0,
            base_w: w, base_h: h,
            close_clicked: false, alive: true,
            menus: Vec::new(),
            scrollable: false, scroll_y: 0,
            tabs: Vec::new(), active_tab: 0,
        });
        self.z_order.push(idx);
        self.focus_window = Some(idx);
        idx as i64
    }

    fn add_widget(&mut self, win: i64, fn_: &str, mut wdg: Widget) -> Result<i64, String> {
        let wi = win as usize;
        let w = self.windows.get_mut(wi).ok_or_else(|| format!("{}: erwartet GUI_WINDOW", fn_))?;
        wdg.color = *self.theme.get("text_fg").unwrap_or(&0xFFFFFF);
        wdg.bx = wdg.x; wdg.by = wdg.y; wdg.bw = wdg.w; wdg.bh = wdg.h;  // Anchor-Basis
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
            group: String::new(), items: Vec::new(), sel: -1,
            enabled: true, font: -1, font_size: 0,
            anchor: 5, bx: x, by: y, bw: w, bh: h,         // Default: oben-links (L|T)
            caret: 0, sel_anchor: 0, scroll: 0,
            tab_page: -1,
            tooltip: String::new(),
            step: 1.0,
        }
    }

    /// Anchor-Bitmaske -> Edge-String ("lt", "lrtb", ...).
    fn anchor_str(a: u8) -> String {
        let mut s = String::new();
        if a & 1 != 0 { s.push('l'); }
        if a & 2 != 0 { s.push('r'); }
        if a & 4 != 0 { s.push('t'); }
        if a & 8 != 0 { s.push('b'); }
        s
    }
    /// Edge-String -> Bitmaske (leer/ungueltig -> Default L|T = 5).
    fn anchor_mask(s: &str) -> u8 {
        let mut a = 0u8;
        for c in s.chars() {
            match c { 'l' | 'L' => a |= 1, 'r' | 'R' => a |= 2,
                      't' | 'T' => a |= 4, 'b' | 'B' => a |= 8, _ => {} }
        }
        if a == 0 { 5 } else { a }
    }

    /// Widgets eines Fensters relativ zur Basisgroesse neu anordnen (Anchoring).
    fn relayout(&mut self, wi: usize) {
        let (cw, ch, bw, bh) = {
            let w = &self.windows[wi];
            (w.w, w.h, w.base_w, w.base_h)
        };
        if bw <= 0 || bh <= 0 { return; }
        let (dx, dy) = (cw - bw, ch - bh);
        for wd in self.windows[wi].widgets.iter_mut() {
            let a = wd.anchor;
            let (l, r, t, b) = (a & 1 != 0, a & 2 != 0, a & 4 != 0, a & 8 != 0);
            let (mut x, mut w) = (wd.bx, wd.bw);
            if l && r { w = (wd.bw + dx).max(1); }       // beide -> dehnen
            else if r { x = wd.bx + dx; }                // nur rechts -> mitwandern
            else if !l { x = wd.bx + dx / 2; }           // keiner -> zentrieren
            let (mut y, mut h) = (wd.by, wd.bh);
            if t && b { h = (wd.bh + dy).max(1); }
            else if b { y = wd.by + dy; }
            else if !t { y = wd.by + dy / 2; }
            wd.x = x; wd.y = y; wd.w = w; wd.h = h;
        }
    }

    // --- Window-Flags ---
    pub fn window_movable(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_MOVABLE")?.movable = f; Ok(())
    }
    pub fn window_closable(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_CLOSABLE")?.closable = f; Ok(())
    }
    pub fn window_resizable(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_RESIZABLE")?.resizable = f; Ok(())
    }
    /// Fenster-Chrome (Titelleiste/Rahmen/Buttons) an/aus. Aus = randlos, der
    /// Inhalt beginnt oben; gedacht, damit eine Form das OS-Fenster ausfuellt.
    pub fn window_chrome(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_CHROME")?.chrome = f; Ok(())
    }
    pub fn window_min_size(&mut self, h: i64, w: i32, ht: i32) -> Result<(), String> {
        let win = self.win_mut(h, "GUI_WINDOW_SET_MIN_SIZE")?;
        win.min_w = w.max(0); win.min_h = ht.max(0); Ok(())
    }
    pub fn window_max_size(&mut self, h: i64, w: i32, ht: i32) -> Result<(), String> {
        let win = self.win_mut(h, "GUI_WINDOW_SET_MAX_SIZE")?;
        win.max_w = w.max(0); win.max_h = ht.max(0); Ok(())
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
    /// Zahlenfeld mit +/- Tasten (Spinner). Wert via Pfeile/Mausrad/Klick auf die
    /// Schaltflaechen, in Schritten von `step`, geklemmt auf [mn, mx].
    pub fn spinner(&mut self, win: i64, x: i32, y: i32, w: i32, mn: f64, mx: f64,
                   default: f64, step: f64) -> Result<i64, String> {
        if mx <= mn { return Err("GUI_SPINNER: max muss > min sein".into()); }
        let h = (self.m("title_h") + 2).max(26);
        let mut wd = Self::blank(Kind::Spinner, x, y, w.max(48), h);
        wd.min = mn; wd.max = mx; wd.value = default.clamp(mn, mx);
        wd.step = if step != 0.0 { step.abs() } else { 1.0 };
        self.add_widget(win, "GUI_SPINNER", wd)
    }
    /// Verschiebbare Trennlinie (Splitter). orient = "v"/"vertical" (senkrechter
    /// Balken, zieht waagerecht) oder "h"/"horizontal" (waagerechter Balken, zieht
    /// senkrecht). Bei "v" ist x die Startposition (geklemmt auf [mn,mx]) und der
    /// Balken laeuft von y ueber `length` Pixel; bei "h" entsprechend y/x getauscht.
    /// Position via GUI_VALUE lesen (fenster-relativ) und damit zwei Bereiche per
    /// GUI_SET_BOUNDS layouten. group = Orientierung ("v"/"h").
    pub fn splitter(&mut self, win: i64, x: i32, y: i32, length: i32, orient: String,
                    mn: i32, mx: i32) -> Result<i64, String> {
        let o = orient.to_lowercase();
        let vert = matches!(o.as_str(), "v" | "vertical" | "vertikal");
        let horiz = matches!(o.as_str(), "h" | "horizontal");
        if !vert && !horiz {
            return Err("GUI_SPLITTER: orientation muss \"v\"/\"vertical\" oder \"h\"/\"horizontal\" sein".into());
        }
        if mx <= mn { return Err("GUI_SPLITTER: max muss > min sein".into()); }
        const THICK: i32 = 6;
        let len = length.max(8);
        let (rx, ry, rw, rh) = if vert { (x.clamp(mn, mx), y, THICK, len) }
                               else { (x, y.clamp(mn, mx), len, THICK) };
        let mut wd = Self::blank(Kind::Splitter, rx, ry, rw, rh);
        wd.group = if vert { "v".into() } else { "h".into() };
        wd.min = mn as f64; wd.max = mx as f64;
        self.add_widget(win, "GUI_SPLITTER", wd)
    }
    pub fn panel(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, title: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Panel, x, y, w, h); wd.text = title;
        self.add_widget(win, "GUI_PANEL", wd)
    }
    pub fn separator(&mut self, win: i64, x: i32, y: i32, w: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Separator, x, y, w.max(1), 8);
        wd.enabled = false;                      // rein dekorativ, nicht interaktiv
        self.add_widget(win, "GUI_SEPARATOR", wd)
    }
    pub fn groupbox(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, title: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::GroupBox, x, y, w, h); wd.text = title;
        wd.enabled = false;
        self.add_widget(win, "GUI_GROUPBOX", wd)
    }
    pub fn textinput(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, placeholder: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::TextInput, x, y, w, h); wd.placeholder = placeholder;
        self.add_widget(win, "GUI_TEXTINPUT", wd)
    }
    /// Mehrzeiliges Textfeld (TextArea): Enter = neue Zeile, vertikal scrollbar.
    pub fn textarea(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, placeholder: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::TextArea, x, y, w, h); wd.placeholder = placeholder;
        self.add_widget(win, "GUI_TEXTAREA", wd)
    }

    // --- Formular-Widgets (Phase 3): Radio, Dropdown, ProgressBar ---
    pub fn radio(&mut self, win: i64, group: String, label: String, x: i32, y: i32) -> Result<i64, String> {
        let cs = self.m("check_size");
        let mut wd = Self::blank(Kind::Radio, x, y, cs, cs);
        wd.text = label; wd.group = group;
        self.add_widget(win, "GUI_RADIO", wd)
    }
    pub fn progress(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Progress, x, y, w, h);
        wd.min = 0.0; wd.max = 1.0; wd.value = 0.0;
        self.add_widget(win, "GUI_PROGRESS", wd)
    }
    pub fn dropdown(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, items: Vec<String>) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Dropdown, x, y, w, h);
        wd.sel = if items.is_empty() { -1 } else { 0 };
        wd.items = items;
        self.add_widget(win, "GUI_DROPDOWN", wd)
    }
    /// Alle Radios derselben Gruppe (Fenster `wi`) ausser `keep` deselektieren.
    fn select_radio(&mut self, wi: usize, keep: usize) {
        let group = self.windows[wi].widgets[keep].group.clone();
        for (j, w) in self.windows[wi].widgets.iter_mut().enumerate() {
            if w.kind == Kind::Radio && w.group == group && w.alive {
                w.checked = j == keep;
            }
        }
    }
    /// Index (0-basiert in Erstellungsreihenfolge) des gewaehlten Radios der
    /// Gruppe von `h`, oder -1. `h` darf ein beliebiges Radio der Gruppe sein.
    pub fn radio_selected(&self, h: i64) -> Result<i64, String> {
        let w = self.wdg(h, "GUI_RADIO_SELECTED")?;
        if w.kind != Kind::Radio { return Err("GUI_RADIO_SELECTED: Widget ist kein radio".into()); }
        let (wi, _) = Self::dec_widget(h);
        let group = &w.group;
        let mut idx = 0i64;
        for wd in &self.windows[wi].widgets {
            if wd.kind == Kind::Radio && &wd.group == group && wd.alive {
                if wd.checked { return Ok(idx); }
                idx += 1;
            }
        }
        Ok(-1)
    }
    // ListBox: scrollbare, immer sichtbare Auswahlliste (teilt items/sel mit
    // Dropdown; `value` haelt den vertikalen Scroll-Offset in Pixeln).
    pub fn listbox(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, items: Vec<String>) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::ListBox, x, y, w, h);
        wd.sel = -1; wd.items = items;
        self.add_widget(win, "GUI_LISTBOX", wd)
    }
    pub fn image(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, tex: i64) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Image, x, y, w, h);
        wd.sel = tex as i32;   // Textur-Handle (LOADIMAGE) im sel-Feld
        self.add_widget(win, "GUI_IMAGE", wd)
    }
    pub fn set_image(&mut self, h: i64, tex: i64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_IMAGE")?;
        if w.kind != Kind::Image { return Err("GUI_SET_IMAGE: Widget ist kein image".into()); }
        w.sel = tex as i32; Ok(())
    }
    /// Button mit Icon (Textur-Handle aus LOADIMAGE/GENTEX im sel-Feld). Mit Text
    /// = Icon links + Text rechts; ohne Text = flacher, mittiger Icon-Button (fuer
    /// Toolbars). Ansonsten ein ganz normaler Button (GUI_CLICKED/ON_CLICK).
    pub fn icon_button(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32,
                       tex: i64, text: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Button, x, y, w, h);
        wd.sel = tex as i32; wd.text = text;
        self.add_widget(win, "GUI_ICON_BUTTON", wd)
    }
    /// Icon eines Buttons setzen/ersetzen (-1 entfernt es).
    pub fn set_icon(&mut self, h: i64, tex: i64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_ICON")?;
        if w.kind != Kind::Button { return Err("GUI_SET_ICON: Widget ist kein button".into()); }
        w.sel = tex as i32; Ok(())
    }
    /// Werkzeugleiste: flacher Streifen als Hintergrund fuer eine Reihe Icon-
    /// Buttons (dekorativ, nicht interaktiv -- Buttons liegen darueber).
    pub fn toolbar(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Toolbar, x, y, w, h);
        wd.enabled = false;
        self.add_widget(win, "GUI_TOOLBAR", wd)
    }
    pub fn canvas(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let wd = Self::blank(Kind::Canvas, x, y, w, h);
        self.add_widget(win, "GUI_CANVAS", wd)
    }
    /// Absoluter Bildschirm-Bereich (Inhaltsflaeche) eines Canvas, in den der User
    /// nach GUI_DRAW mit normalen Zeichenbefehlen malen kann (selbst clippen).
    pub fn canvas_rect(&self, h: i64) -> Result<(i32, i32, i32, i32), String> {
        let (wi, _) = Self::dec_widget(h);
        let w = self.wdg(h, "GUI_CANVAS_*")?;
        if w.kind != Kind::Canvas { return Err("GUI_CANVAS_*: Widget ist kein canvas".into()); }
        let (ax, ay, ww, wh) = self.abs_rect(wi, w);
        Ok((ax + 1, ay + 1, (ww - 2).max(0), (wh - 2).max(0)))   // Inhalt ohne Rahmen
    }
    // Item-Auswahl: Dropdown UND ListBox (beide nutzen items/sel).
    fn item_widget(&self, h: i64, fn_: &str) -> Result<&Widget, String> {
        let w = self.wdg(h, fn_)?;
        if !matches!(w.kind, Kind::Dropdown | Kind::ListBox) { return Err(format!("{}: Widget ist kein dropdown/listbox", fn_)); }
        Ok(w)
    }
    pub fn dropdown_selected(&self, h: i64) -> Result<i64, String> {
        Ok(self.item_widget(h, "GUI_DROPDOWN_SELECTED")?.sel as i64)
    }
    pub fn dropdown_text(&self, h: i64) -> Result<String, String> {
        let w = self.item_widget(h, "GUI_DROPDOWN_TEXT")?;
        Ok(if w.sel >= 0 && (w.sel as usize) < w.items.len() { w.items[w.sel as usize].clone() } else { String::new() })
    }
    pub fn dropdown_set_selected(&mut self, h: i64, idx: i64) -> Result<(), String> {
        self.item_widget(h, "GUI_DROPDOWN_SET_SELECTED")?;
        let w = self.wdg_mut(h, "GUI_DROPDOWN_SET_SELECTED")?;
        w.sel = if idx >= 0 && (idx as usize) < w.items.len() { idx as i32 } else { -1 };
        Ok(())
    }
    pub fn set_dropdown_items(&mut self, h: i64, items: Vec<String>) -> Result<(), String> {
        self.item_widget(h, "GUI_SET_DROPDOWN")?;
        let w = self.wdg_mut(h, "GUI_SET_DROPDOWN")?;
        w.sel = if items.is_empty() { -1 } else { w.sel.min(items.len() as i32 - 1) };
        w.items = items; w.value = 0.0; Ok(())
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
    pub fn clicked(&self, h: i64) -> Result<bool, String> {
        // Menue-Item-Handle? -> dessen clicked-Flag.
        if h & ITEM_FLAG != 0 {
            let (wi, mi, ii) = dec_item(h);
            return Ok(self.windows.get(wi).and_then(|w| w.menus.get(mi))
                .and_then(|m| m.items.get(ii)).map(|it| it.clicked).unwrap_or(false));
        }
        Ok(self.wdg(h, "GUI_CLICKED")?.clicked)
    }

    // --- Menues -----------------------------------------------------------
    fn add_menu_impl(&mut self, win: i64, in_bar: bool, label: String, fn_: &str) -> Result<i64, String> {
        let wi = win as usize;
        let w = self.windows.get_mut(wi).ok_or_else(|| format!("{}: erwartet GUI_WINDOW", fn_))?;
        let mi = w.menus.len();
        w.menus.push(Menu { label, in_bar, items: Vec::new() });
        Ok(enc_menu(wi, mi))
    }
    /// Top-Level-Menue in der Menueleiste (z.B. "Datei").
    pub fn add_menu(&mut self, win: i64, label: String) -> Result<i64, String> {
        self.add_menu_impl(win, true, label, "GUI_MENU")
    }
    /// Kontextmenue (per Rechtsklick im Fenster). Label wird nicht angezeigt.
    pub fn add_context(&mut self, win: i64) -> Result<i64, String> {
        self.add_menu_impl(win, false, String::new(), "GUI_CONTEXT")
    }
    fn menu_mut(&mut self, h: i64, fn_: &str) -> Result<&mut Menu, String> {
        if h & MENU_FLAG == 0 || h & ITEM_FLAG != 0 { return Err(format!("{}: erwartet GUI_MENU/GUI_CONTEXT", fn_)); }
        let (wi, mi) = dec_menu(h);
        self.windows.get_mut(wi).and_then(|w| w.menus.get_mut(mi))
            .ok_or_else(|| format!("{}: ungueltiges Menue-Handle", fn_))
    }
    /// Eintrag an ein Menue anhaengen -> Item-Handle (fuer GUI_CLICKED).
    pub fn add_menu_item(&mut self, menu: i64, label: String) -> Result<i64, String> {
        let (wi, mi) = dec_menu(menu);
        let m = self.menu_mut(menu, "GUI_MENU_ITEM")?;
        let ii = m.items.len();
        m.items.push(MenuItem { label, separator: false, enabled: true, clicked: false });
        Ok(enc_item(wi, mi, ii))
    }
    /// Trennlinie an ein Menue anhaengen.
    pub fn add_menu_separator(&mut self, menu: i64) -> Result<(), String> {
        let m = self.menu_mut(menu, "GUI_MENU_SEPARATOR")?;
        m.items.push(MenuItem { label: String::new(), separator: true, enabled: false, clicked: false });
        Ok(())
    }
    pub fn hovered(&self, h: i64) -> Result<bool, String> { Ok(self.wdg(h, "GUI_HOVERED")?.hovered) }
    pub fn checked(&self, h: i64) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_CHECKED")?;
        if !matches!(w.kind, Kind::Checkbox | Kind::Radio) { return Err("GUI_CHECKED: Widget ist keine checkbox/radio".into()); }
        Ok(w.checked)
    }
    pub fn value(&self, h: i64) -> Result<f64, String> {
        let w = self.wdg(h, "GUI_VALUE")?;
        match w.kind {
            Kind::Slider | Kind::Progress | Kind::Spinner => Ok(w.value),
            Kind::Splitter => Ok(if w.group == "v" { w.x as f64 } else { w.y as f64 }),
            _ => Err("GUI_VALUE: Widget ist kein slider/progress/spinner/splitter".into()),
        }
    }
    pub fn text(&self, h: i64) -> Result<String, String> { Ok(self.wdg(h, "GUI_TEXT")?.text.clone()) }
    pub fn set_text(&mut self, h: i64, t: String) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_TEXT")?;
        let n = t.chars().count() as i32;
        w.text = t;
        // Caret ans Ende, Selektion/Scroll zuruecksetzen (sonst zeigt das Caret
        // hinter das Ende des nun kuerzeren Textes).
        w.caret = n; w.sel_anchor = n; w.scroll = 0;
        Ok(())
    }
    pub fn set_tooltip(&mut self, h: i64, t: String) -> Result<(), String> {
        self.wdg_mut(h, "GUI_TOOLTIP")?.tooltip = t; Ok(())
    }
    pub fn set_checked(&mut self, h: i64, f: bool) -> Result<(), String> {
        let kind = self.wdg(h, "GUI_SET_CHECKED")?.kind;
        if !matches!(kind, Kind::Checkbox | Kind::Radio) { return Err("GUI_SET_CHECKED: Widget ist keine checkbox/radio".into()); }
        self.wdg_mut(h, "GUI_SET_CHECKED")?.checked = f;
        // Radio: beim Setzen die Gruppen-Geschwister deselektieren.
        if kind == Kind::Radio && f {
            let (wi, i) = Self::dec_widget(h);
            self.select_radio(wi, i);
        }
        Ok(())
    }
    pub fn set_value(&mut self, h: i64, v: f64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_VALUE")?;
        match w.kind {
            Kind::Slider | Kind::Progress | Kind::Spinner => { w.value = v.clamp(w.min, w.max); }
            Kind::Splitter => {
                let c = (v.round() as i32).clamp(w.min as i32, w.max as i32);
                if w.group == "v" { w.x = c; } else { w.y = c; }
            }
            _ => return Err("GUI_SET_VALUE: Widget ist kein slider/progress/spinner/splitter".into()),
        }
        Ok(())
    }
    pub fn on_click(&mut self, h: i64, func: Option<String>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_CLICK")?.on_click = func; Ok(())
    }
    pub fn on_change(&mut self, h: i64, func: Option<String>) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_ON_CHANGE")?;
        if !matches!(w.kind, Kind::Slider | Kind::TextInput | Kind::TextArea | Kind::Checkbox | Kind::Table | Kind::Radio | Kind::Dropdown | Kind::ListBox | Kind::Spinner | Kind::Splitter) {
            return Err("GUI_ON_CHANGE: nur fuer slider, textinput, textarea, checkbox, table, radio, dropdown, listbox, spinner oder splitter".into());
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
    // --- Styling (Phase 4): enabled-Zustand, per-Widget-Font/-Groesse ---
    pub fn set_enabled(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_ENABLED")?.enabled = f;
        if !f {
            let (wi, i) = Self::dec_widget(h);
            if self.focus_widget == Some((wi, i)) { self.focus_widget = None; }
            if self.open_dropdown == Some((wi, i)) { self.open_dropdown = None; }
        }
        Ok(())
    }
    pub fn enabled(&self, h: i64) -> Result<bool, String> { Ok(self.wdg(h, "GUI_ENABLED")?.enabled) }
    pub fn set_font(&mut self, h: i64, font: i64) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_FONT")?.font = font; Ok(())
    }
    pub fn set_font_size(&mut self, h: i64, sz: i64) -> Result<(), String> {
        if sz < 0 { return Err("GUI_SET_FONT_SIZE: Groesse muss >= 0 sein".into()); }
        self.wdg_mut(h, "GUI_SET_FONT_SIZE")?.font_size = sz as i32; Ok(())
    }
    /// Anchoring eines Widgets setzen (Edge-String aus "lrtb", z.B. "lrtb",
    /// "rb"). Die Anchor-Basis ist das aktuelle Rechteck des Widgets.
    pub fn set_anchor(&mut self, h: i64, s: &str) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_ANCHOR")?.anchor = Self::anchor_mask(s); Ok(())
    }
    /// Benannten Style anlegen/erweitern: `prop` in bg/fg/border/accent (Farbe),
    /// font (Handle), font_size (px). Inkrementell aufrufbar.
    pub fn style_set(&mut self, name: String, prop: String, value: i64) -> Result<(), String> {
        if !matches!(prop.as_str(), "bg" | "fg" | "border" | "accent" | "font" | "font_size") {
            return Err("GUI_STYLE_SET: prop muss bg/fg/border/accent/font/font_size sein".into());
        }
        if prop == "font_size" && value < 0 { return Err("GUI_STYLE_SET: font_size muss >= 0 sein".into()); }
        self.styles.entry(name).or_default().insert(prop, value);
        Ok(())
    }
    /// Einen benannten Style auf ein Widget anwenden (ueberschreibt dessen
    /// Farb-Overrides/Font). Wirft, wenn der Style unbekannt ist.
    pub fn apply_style(&mut self, h: i64, name: &str) -> Result<(), String> {
        let props = self.styles.get(name)
            .ok_or_else(|| format!("GUI_APPLY_STYLE: unbekannter Style '{}'", name))?
            .clone();
        let w = self.wdg_mut(h, "GUI_APPLY_STYLE")?;
        for (k, v) in props {
            match k.as_str() {
                "font" => w.font = v,
                "font_size" => w.font_size = v as i32,
                _ => { w.ov.insert(k, v); }   // bg/fg/border/accent
            }
        }
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
        if self.active_split == Some((wi, i)) { self.active_split = None; }
        if self.press_origin == Some((wi, i)) { self.press_origin = None; }
        if self.open_dropdown == Some((wi, i)) { self.open_dropdown = None; }
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
        let (ow, oh) = (win.w, win.h);
        win.x = x; win.y = y; win.w = w.max(0); win.h = ht.max(0);
        let wi = h as usize;
        if self.windows[wi].w != ow || self.windows[wi].h != oh {
            self.relayout(wi);                          // Anchoring nachziehen
        }
        Ok(())
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
        if self.resize_window == Some(wi) { self.resize_window = None; }
        if self.open_dropdown.map(|(w, _)| w) == Some(wi) { self.open_dropdown = None; }
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
        if !w.group.is_empty() { o["group"] = serde_json::json!(w.group); }
        if !w.items.is_empty() { o["items"] = serde_json::json!(w.items); }
        if w.sel != -1 { o["sel"] = serde_json::json!(w.sel); }   // Dropdown/ListBox-Index ODER Image-Textur
        if !w.enabled { o["enabled"] = serde_json::json!(false); }
        if w.font != -1 { o["font"] = serde_json::json!(w.font); }
        if w.font_size != 0 { o["font_size"] = serde_json::json!(w.font_size); }
        if w.anchor != 5 { o["anchor"] = serde_json::json!(Self::anchor_str(w.anchor)); }
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
        w.group = wj["group"].as_str().unwrap_or("").to_string();
        if let Some(its) = wj["items"].as_array() {
            w.items = its.iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
        }
        w.sel = wj["sel"].as_i64().unwrap_or(-1) as i32;
        w.enabled = wj["enabled"].as_bool().unwrap_or(true);
        w.font = wj["font"].as_i64().unwrap_or(-1);
        w.font_size = wj["font_size"].as_i64().unwrap_or(0) as i32;
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
        w.anchor = wj["anchor"].as_str().map(Self::anchor_mask).unwrap_or(5);
        w.bx = w.x; w.by = w.y; w.bw = w.w; w.bh = w.h;   // Anchor-Basis = Design-Rechteck
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
            "resizable": win.resizable, "chrome": win.chrome,
            "min_w": win.min_w, "min_h": win.min_h, "max_w": win.max_w, "max_h": win.max_h,
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
        self.windows[wi].resizable = v["resizable"].as_bool().unwrap_or(false);
        self.windows[wi].chrome = v["chrome"].as_bool().unwrap_or(true);
        self.windows[wi].min_w = gi("min_w", 0); self.windows[wi].min_h = gi("min_h", 0);
        self.windows[wi].max_w = gi("max_w", 0); self.windows[wi].max_h = gi("max_h", 0);
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
        let key = name.to_lowercase();
        match preset(&key) {
            Some(p) => {
                for (k, v) in p { self.theme.insert(k, v); }
                // Passendes Metrik-Profil mitziehen, damit ein Preset ein KOMPLETTER
                // Look ist (modern_* -> runde Ecken + Schatten, klassische -> flach).
                self.metrics = preset_metrics(&key).unwrap_or_else(default_metrics);
                Ok(())
            }
            None => Err(format!("GUI_THEME_PRESET: unbekanntes Preset '{}'", name)),
        }
    }

    /// "Modern-Modus": runde Ecken aktiv -> Renderer zeichnet Schatten, gerundete
    /// Titelleiste, Haekchen-Checkboxen usw. (statt des flachen Retro-Looks).
    fn modern(&self) -> bool { self.m("corner_radius") > 0 }

    // --- Geometrie ---
    fn abs_rect(&self, win: usize, w: &Widget) -> (i32, i32, i32, i32) {
        let toff = self.content_top(win);
        let sy = if self.windows[win].scrollable { self.windows[win].scroll_y } else { 0 };
        let win = &self.windows[win];
        (win.x + w.x, win.y + toff + w.y - sy, w.w, w.h)
    }
    /// Hoehe der Menueleiste dieses Fensters (0, wenn keine Bar-Menues).
    fn menubar_h(&self, win: usize) -> i32 {
        if self.windows[win].menus.iter().any(|m| m.in_bar) { MENUBAR_H } else { 0 }
    }
    /// Oberkante des Inhaltsbereichs (Titelleiste + Menueleiste + Reiter-Leiste).
    fn content_top(&self, win: usize) -> i32 {
        (if self.windows[win].chrome { self.m("title_h") } else { 0 })
            + self.menubar_h(win) + self.tabbar_h(win)
    }
    /// Hoehe der Reiter-Leiste (0, wenn keine Tabs).
    fn tabbar_h(&self, win: usize) -> i32 {
        if self.windows[win].tabs.is_empty() { 0 } else { TABBAR_H }
    }
    /// Ist das Widget aktuell sichtbar/bedienbar? (Tab-Seite beruecksichtigt)
    fn widget_shown(&self, win: usize, w: &Widget) -> bool {
        w.alive && w.visible && (w.tab_page < 0 || w.tab_page == self.windows[win].active_tab)
    }
    /// Layout der Reiter: (page_idx, x_links_abs, x_rechts_abs).
    fn tab_slots(&self, g: &Graphics, wi: usize) -> Vec<(usize, i32, i32)> {
        let win = &self.windows[wi];
        let pad = self.m("pad");
        let mut x = win.x + pad;
        let mut out = Vec::new();
        for (ti, label) in win.tabs.iter().enumerate() {
            let wl = g.text_width(label) + pad * 3;
            out.push((ti, x, x + wl));
            x += wl + 2;
        }
        out
    }
    /// Inhaltshoehe = unterster Widget-Rand (+Rand). Basis fuer den Scrollbereich.
    fn content_height(&self, win: usize) -> i32 {
        let pad = self.m("pad");
        self.windows[win].widgets.iter()
            .filter(|w| self.widget_shown(win, w) && w.kind != Kind::Canvas)
            .map(|w| w.y + w.h).max().unwrap_or(0) + pad * 2
    }
    /// Sichtbare Inhaltshoehe (Fenster minus Titel/Menue/Rand).
    fn view_height(&self, win: usize) -> i32 {
        (self.windows[win].h - self.content_top(win) - 2).max(1)
    }
    /// Maximaler Scroll-Offset (0, wenn Inhalt passt).
    fn max_scroll_y(&self, win: usize) -> i32 {
        if !self.windows[win].scrollable { return 0; }
        (self.content_height(win) - self.view_height(win)).max(0)
    }
    /// Reiter (Tabs) eines Fensters setzen (Labels). Aktiver Reiter -> 0.
    pub fn set_tabs(&mut self, win: i64, labels: Vec<String>) -> Result<(), String> {
        let w = self.windows.get_mut(win as usize).ok_or("GUI_TABS: erwartet GUI_WINDOW")?;
        w.tabs = labels;
        w.active_tab = 0;
        Ok(())
    }
    /// Widget einem Reiter zuordnen (-1 = auf allen Reitern sichtbar).
    pub fn set_widget_tab(&mut self, h: i64, page: i32) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_TAB")?.tab_page = page;
        Ok(())
    }
    pub fn active_tab(&self, win: i64) -> Result<i64, String> {
        self.windows.get(win as usize).map(|w| w.active_tab as i64).ok_or("GUI_ACTIVE_TAB: erwartet GUI_WINDOW".into())
    }
    pub fn set_active_tab(&mut self, win: i64, i: i32) -> Result<(), String> {
        let w = self.windows.get_mut(win as usize).ok_or("GUI_SET_ACTIVE_TAB: erwartet GUI_WINDOW")?;
        if i >= 0 && (i as usize) < w.tabs.len() { w.active_tab = i; }
        Ok(())
    }
    /// Inhalt eines Fensters scrollbar machen (Mausrad + Scrollbalken).
    pub fn window_scrollable(&mut self, win: i64, flag: bool) -> Result<(), String> {
        let w = self.windows.get_mut(win as usize).ok_or("GUI_WINDOW_SCROLLABLE: erwartet GUI_WINDOW")?;
        w.scrollable = flag;
        if !flag { w.scroll_y = 0; }
        Ok(())
    }
    /// Geometrie des Inhalts-Scrollbalkens: (track_x, track_y, breite, track_h,
    /// thumb_y, thumb_h) oder None, wenn nicht noetig/aus.
    fn winscroll_geom(&self, wi: usize) -> Option<(i32, i32, i32, i32, i32, i32)> {
        let content = self.content_height(wi);
        let view = self.view_height(wi);
        if !self.windows[wi].scrollable || content <= view { return None; }
        let w = &self.windows[wi];
        let bw = 10;
        let tx = w.x + w.w - bw - 2;
        let ty = w.y + self.content_top(wi) + 1;
        let th = view - 2;
        let max_s = content - view;
        let thh = ((th * view) / content).clamp(24, th);
        let thy = ty + if max_s > 0 { (w.scroll_y * (th - thh)) / max_s } else { 0 };
        Some((tx, ty, bw, th, thy, thh))
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
    // Styling-aware Farb-/Text-Helfer (beruecksichtigen den enabled-Zustand).
    fn txt_col(&self, w: &Widget) -> i64 {
        if !w.enabled { self.th("muted_fg") } else { self.wcol(w, "fg", "text_fg") }
    }
    fn acc_col(&self, w: &Widget) -> i64 {
        let a = self.wcol(w, "accent", "accent");
        if !w.enabled { shade(a, -70) } else { a }
    }
    /// Text mit per-Widget-Font/-Groesse (sonst unveraendert via g.text).
    fn wtext(&self, g: &mut Graphics, w: &Widget, x: i32, y: i32, s: String, c: i64) {
        if w.font == -1 && w.font_size == 0 {
            g.text(x, y, s, c);
        } else {
            let sz = if w.font_size > 0 { w.font_size } else { g.text_height() };
            g.text_styled(x, y, s, c, w.font, sz);
        }
    }
    /// Gefuellte Box + Rahmen, runde Ecken wenn Metrik `corner_radius` > 0.
    fn fbox(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, fill: i64, border: i64) {
        let rad = self.m("corner_radius");
        if rad > 0 {
            g.round_rect(x1, y1, x2, y2, rad, fill, true);
            g.round_rect(x1, y1, x2, y2, rad, border, false);
        } else {
            g.box_fill(x1, y1, x2, y2, fill);
            g.rect(x1, y1, x2, y2, border);
        }
    }

    // --- Update (ein Frame) ---
    pub fn update(&mut self, g: &mut Graphics) {
        let mx = g.mouse_x() as i32;
        let my = g.mouse_y() as i32;
        let is_down = g.mouse_button(0);
        let just_pressed = is_down && !self.was_mouse_down;
        let just_released = !is_down && self.was_mouse_down;
        let right_down = g.mouse_button(1);
        let right_just = right_down && !self.was_right_down;

        // Transiente Flags ruecksetzen.
        for win in self.windows.iter_mut() {
            for wdg in win.widgets.iter_mut() {
                wdg.clicked = false; wdg.hovered = false;
                if let Some(t) = wdg.tbl.as_mut() { t.hover_row = -1; t.clicked_row = -1; }
            }
            for m in win.menus.iter_mut() {
                for it in m.items.iter_mut() { it.clicked = false; }
            }
        }
        // Menue-Eingabe (Menueleiste/Dropdown/Kontext) VOR den Widgets -- konsumiert
        // den Klick ggf., damit er nicht zusaetzlich ein Widget ausloest.
        let menu_consumed = self.menu_input(mx, my, just_pressed, right_just, g);

        // Inhalts-Scroll: Mausrad ueber scrollbarem Fenster.
        if let Some(top) = self.topmost_at(mx, my) {
            if self.windows[top].scrollable {
                let wheel = g.pop_mouse_wheel();
                if wheel != 0 {
                    let ms = self.max_scroll_y(top);
                    self.windows[top].scroll_y = (self.windows[top].scroll_y - wheel as i32 * 40).clamp(0, ms);
                }
            }
        }
        // Scroll-Offsets klemmen (Inhalt kann durch Resize/Aenderung schrumpfen).
        for wi in 0..self.windows.len() {
            if self.windows[wi].scrollable {
                let ms = self.max_scroll_y(wi);
                self.windows[wi].scroll_y = self.windows[wi].scroll_y.clamp(0, ms);
            }
        }
        // Laufendes Scrollbalken-Drag.
        if let Some(wi) = self.scroll_drag {
            if is_down {
                if let Some((_tx, ty, _bw, th, _thy, thh)) = self.winscroll_geom(wi) {
                    let ms = self.max_scroll_y(wi);
                    let rel = ((my - ty - thh / 2) as f32 / (th - thh).max(1) as f32).clamp(0.0, 1.0);
                    self.windows[wi].scroll_y = (rel * ms as f32) as i32;
                }
            } else { self.scroll_drag = None; }
        }
        // Klick auf den Scrollbalken startet das Drag (konsumiert den Klick).
        let mut scroll_consumed = false;
        if just_pressed && !menu_consumed && self.scroll_drag.is_none() {
            if let Some(top) = self.topmost_at(mx, my) {
                if let Some((tx, ty, bw, th, _thy, _thh)) = self.winscroll_geom(top) {
                    if mx >= tx && mx < tx + bw && my >= ty && my < ty + th {
                        self.scroll_drag = Some(top);
                        scroll_consumed = true;
                    }
                }
            }
        }
        // Klick auf einen Reiter (Tab) wechselt die Seite (konsumiert den Klick).
        let mut tab_consumed = false;
        if just_pressed && !menu_consumed && !scroll_consumed {
            for &wi in self.z_order.clone().iter().rev() {
                if !self.windows[wi].alive || !self.windows[wi].visible || self.tabbar_h(wi) == 0 { continue; }
                let by = self.windows[wi].y
                    + (if self.windows[wi].chrome { self.m("title_h") } else { 0 })
                    + self.menubar_h(wi);
                if my < by || my >= by + TABBAR_H { continue; }
                if let Some((ti, _, _)) = self.tab_slots(g, wi).into_iter().find(|(_, x0, x1)| mx >= *x0 && mx < *x1) {
                    self.windows[wi].active_tab = ti as i32;
                    tab_consumed = true;
                    break;
                }
            }
        }
        // Hover (nur oberstes Fenster); Tabellen aktualisieren Scroll/Hover/Wheel.
        if let Some(top) = self.topmost_at(mx, my) {
            let n = self.windows[top].widgets.len();
            for i in 0..n {
                let (r, kind, active) = {
                    let w = &self.windows[top].widgets[i];
                    (self.abs_rect(top, w), w.kind, self.widget_shown(top, w) && w.enabled)
                };
                if active && Self::in_rect(mx, my, r) {
                    self.windows[top].widgets[i].hovered = true;
                    if kind == Kind::Table { self.table_hover(top, i, mx, my, g); }
                    if kind == Kind::ListBox { self.listbox_wheel(top, i, r.3, g); }
                    if kind == Kind::Spinner { self.spinner_wheel(top, i, g); }
                }
            }
        }
        // Tooltip-Dwell: oberstes Widget mit Hilfetext unter der Maus bestimmen.
        // Bei Wechsel/Bewegung/gedrueckter Maus startet der Verweil-Timer neu.
        let tip_cur = self.topmost_at(mx, my).and_then(|top| {
            let mut found = None;
            for i in 0..self.windows[top].widgets.len() {
                let w = &self.windows[top].widgets[i];
                if w.hovered && !w.tooltip.is_empty() { found = Some((top, i)); }
            }
            found
        });
        if tip_cur != self.hover_w || mx != self.hover_x || my != self.hover_y || is_down {
            self.hover_w = tip_cur;
            self.hover_frame = self.frame_count;
        }
        self.hover_x = mx;
        self.hover_y = my;
        // Laufendes Fenster-Drag.
        if let Some(wi) = self.drag_window {
            if is_down {
                self.windows[wi].x = mx - self.drag_dx;
                self.windows[wi].y = my - self.drag_dy;
            } else { self.drag_window = None; }
        }
        // Laufender Fenster-Resize (unten-rechts).
        if let Some(wi) = self.resize_window {
            if is_down {
                let w = &mut self.windows[wi];
                let mut nw = (mx - w.x + self.resize_dx).max(80);
                let mut nh = (my - w.y + self.resize_dy).max(50);
                if w.min_w > 0 { nw = nw.max(w.min_w); }
                if w.min_h > 0 { nh = nh.max(w.min_h); }
                if w.max_w > 0 { nw = nw.min(w.max_w); }
                if w.max_h > 0 { nh = nh.min(w.max_h); }
                let changed = w.w != nw || w.h != nh;
                w.w = nw; w.h = nh;
                if changed { self.relayout(wi); }       // Anchoring nachziehen
            } else { self.resize_window = None; }
        }
        // Laufendes Slider-Drag.
        if let Some((wi, i)) = self.active_slider {
            if is_down { self.drag_slider(wi, i, mx); } else { self.active_slider = None; }
        }
        // Laufendes Splitter-Drag.
        if let Some((wi, i)) = self.active_split {
            if is_down { self.drag_split(wi, i, mx, my); } else { self.active_split = None; }
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
        // Neuer Druck (entfaellt, wenn ein Menue den Klick verarbeitet hat).
        if just_pressed && !menu_consumed && !scroll_consumed && !tab_consumed && self.drag_window.is_none() && self.resize_window.is_none()
            && self.active_slider.is_none() && self.active_table.is_none() && self.active_split.is_none() {
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
        // Tastatur + Maus fuer das fokussierte Textfeld (Caret/Selektion/Editieren).
        if let Some((wi, i)) = self.focus_widget {
            match self.windows[wi].widgets[i].kind {
                Kind::TextInput => self.edit_textinput(wi, i, g),
                Kind::TextArea => self.edit_textarea(wi, i, g),
                Kind::Spinner => self.spinner_keys(wi, i, g),
                _ => {}
            }
        }
        // Tastatur-Navigation: Tab / Shift+Tab wechselt den Fokus zwischen den
        // TextInputs des aktiven Fensters (in Anlege-Reihenfolge, nur sichtbare).
        if g.key_pressed(KEY_TAB) {
            if let Some(top) = self.focus_window {
                let n = self.windows[top].widgets.len();
                let mut idxs: Vec<usize> = Vec::new();
                for i in 0..n {
                    let w = &self.windows[top].widgets[i];
                    if matches!(w.kind, Kind::TextInput | Kind::TextArea) && self.widget_shown(top, w) && w.enabled { idxs.push(i); }
                }
                if !idxs.is_empty() {
                    let cur = self.focus_widget.filter(|(w, _)| *w == top).map(|(_, i)| i);
                    let pos = cur.and_then(|c| idxs.iter().position(|&i| i == c));
                    let fwd = !g.key_shift();
                    let next = match pos {
                        Some(p) => if fwd { (p + 1) % idxs.len() } else { (p + idxs.len() - 1) % idxs.len() },
                        None => 0,
                    };
                    let ni = idxs[next];
                    self.focus_widget = Some((top, ni));
                    let len = self.windows[top].widgets[ni].text.chars().count() as i32;
                    let w = &mut self.windows[top].widgets[ni];
                    w.caret = len; w.sel_anchor = 0;   // Inhalt markiert (wie ueblich beim Tabben)
                }
            }
        }
        self.was_mouse_down = is_down;
        self.was_right_down = right_down;
        self.frame_count += 1;
    }

    // --- Menue-Eingabe + Hit-Tests -----------------------------------------
    fn popup_width(&self, g: &Graphics, wi: usize, mi: usize) -> i32 {
        let pad = self.m("pad");
        let mut wmax = 80;
        if let Some(m) = self.windows.get(wi).and_then(|w| w.menus.get(mi)) {
            for it in &m.items { if !it.separator { wmax = wmax.max(g.text_width(&it.label) + pad * 4); } }
        }
        wmax
    }
    fn popup_item_at(&self, g: &Graphics, wi: usize, mi: usize, px: i32, py: i32, mx: i32, my: i32) -> Option<usize> {
        let m = self.windows.get(wi).and_then(|w| w.menus.get(mi))?;
        let wmax = self.popup_width(g, wi, mi);
        if mx < px || mx >= px + wmax { return None; }
        for (ii, it) in m.items.iter().enumerate() {
            let iy = py + 2 + ii as i32 * MENU_ITEM_H;
            if my >= iy && my < iy + MENU_ITEM_H {
                if it.separator || !it.enabled { return None; }
                return Some(ii);
            }
        }
        None
    }
    fn bar_menu_at(&self, g: &Graphics, wi: usize, mx: i32, my: i32) -> Option<usize> {
        if self.menubar_h(wi) == 0 { return None; }
        let by = self.windows[wi].y + if self.windows[wi].chrome { self.m("title_h") } else { 0 };
        if my < by || my >= by + MENUBAR_H { return None; }
        self.menubar_slots(g, wi).into_iter().find(|(_, x0, x1)| mx >= *x0 && mx < *x1).map(|(mi, _, _)| mi)
    }
    fn fire_menu_item(&mut self, wi: usize, mi: usize, ii: usize) {
        if let Some(it) = self.windows.get_mut(wi).and_then(|w| w.menus.get_mut(mi)).and_then(|m| m.items.get_mut(ii)) {
            it.clicked = true;
        }
    }
    /// Verarbeitet Menueleisten-/Dropdown-/Kontext-Klicks. Gibt true zurueck,
    /// wenn der Klick konsumiert wurde (dann kein Widget-Press).
    fn menu_input(&mut self, mx: i32, my: i32, left_just: bool, right_just: bool, g: &Graphics) -> bool {
        // 1) Offenes Kontextmenue
        if let Some((wi, mi, cx, cy)) = self.context_open {
            if left_just || right_just {
                let hit = self.popup_item_at(g, wi, mi, cx, cy, mx, my);
                self.context_open = None;
                if let Some(ii) = hit { self.fire_menu_item(wi, mi, ii); }
                return true;
            }
            return false;
        }
        // 2) Offenes Menueleisten-Dropdown
        if let Some((wi, mi)) = self.open_menu {
            if left_just {
                let toff = (if self.windows[wi].chrome { self.m("title_h") } else { 0 }) + MENUBAR_H;
                let py = self.windows[wi].y + toff;
                if let Some((_, x0, _)) = self.menubar_slots(g, wi).into_iter().find(|(m, _, _)| *m == mi) {
                    if let Some(ii) = self.popup_item_at(g, wi, mi, x0, py, mx, my) {
                        self.open_menu = None;
                        self.fire_menu_item(wi, mi, ii);
                        return true;
                    }
                }
                if let Some(nm) = self.bar_menu_at(g, wi, mx, my) {
                    self.open_menu = Some((wi, nm));   // auf anderes Bar-Menue umschalten
                    return true;
                }
                self.open_menu = None;                 // Klick ausserhalb -> schliessen
                return true;
            }
            return false;
        }
        // 3) Kein Menue offen
        if left_just {
            for &wi in self.z_order.clone().iter().rev() {
                if !self.windows[wi].alive || !self.windows[wi].visible { continue; }
                if let Some(nm) = self.bar_menu_at(g, wi, mx, my) {
                    self.open_menu = Some((wi, nm));
                    return true;
                }
            }
        }
        if right_just {
            if let Some(wi) = self.topmost_at(mx, my) {
                if let Some(mi) = self.windows[wi].menus.iter().position(|m| !m.in_bar) {
                    self.context_open = Some((wi, mi, mx, my));
                    return true;
                }
            }
        }
        false
    }

    /// Zeichen-Index, dessen Caret-Position am naechsten an `target_px` liegt
    /// (Pixel ab Textanfang). Misst echte Textbreiten -> funktioniert mit jedem Font.
    fn caret_index_at(g: &Graphics, chars: &[char], target_px: i32) -> i32 {
        if target_px <= 0 { return 0; }
        for n in 1..=chars.len() {
            let prev: String = chars[..n - 1].iter().collect();
            let cur: String = chars[..n].iter().collect();
            let mid = (g.text_width(&prev) + g.text_width(&cur)) / 2;
            if target_px < mid { return (n - 1) as i32; }
        }
        chars.len() as i32
    }

    /// Volles Editieren des fokussierten TextInputs: Zeichen einfuegen, Backspace/
    /// Delete, Pfeile/Home/End (Shift = Selektion), Strg+A/C/V/X, Maus-Klick/-Drag
    /// zum Setzen/Aufziehen der Selektion. Caret/Selektion/Scroll werden mit
    /// GEMESSENEN Textbreiten gefuehrt (passt zu jedem Font).
    fn edit_textinput(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let before = self.windows[wi].widgets[i].text.clone();
        let (ax, _ay, fw, _fh) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let mut chars: Vec<char> = before.chars().collect();
        let mut caret = self.windows[wi].widgets[i].caret.clamp(0, chars.len() as i32);
        let mut anchor = self.windows[wi].widgets[i].sel_anchor.clamp(0, chars.len() as i32);
        let scroll = self.windows[wi].widgets[i].scroll;
        let shift = g.key_shift();
        let ctrl = g.key_ctrl();

        // --- Maus: Klick setzt Caret (rising edge), Ziehen erweitert Selektion ---
        if g.mouse_button(0) {
            let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
            let target = mx - (ax + 5) + scroll;
            let idx = Self::caret_index_at(g, &chars, target);
            if !self.was_mouse_down {
                let r = self.abs_rect(wi, &self.windows[wi].widgets[i]);
                if Self::in_rect(mx, my, r) { caret = idx; anchor = idx; }
            } else {
                caret = idx;   // Drag -> Selektion bis hierher
            }
        }

        // --- Zeichen-Eingabe (nicht bei gedruecktem Strg) ---
        if !ctrl {
            let typed: String = g.pop_text_input().chars()
                .filter(|c| !c.is_control() && *c != '\t').collect();
            if !typed.is_empty() {
                let (lo, hi) = (caret.min(anchor), caret.max(anchor));
                if lo != hi { chars.drain(lo as usize..hi as usize); caret = lo; }
                for (k, ch) in typed.chars().enumerate() { chars.insert(caret as usize + k, ch); }
                caret += typed.chars().count() as i32; anchor = caret;
            }
        } else {
            // Strg+A: alles markieren
            if g.key_pressed(K_A) { anchor = 0; caret = chars.len() as i32; }
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if g.key_pressed(K_C) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
            }
            if g.key_pressed(K_X) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
                chars.drain(lo as usize..hi as usize); caret = lo; anchor = lo;
            }
            if g.key_pressed(K_V) {
                let ins: Vec<char> = g.clipboard_get().chars().filter(|c| !c.is_control()).collect();
                let (lo, hi) = (caret.min(anchor), caret.max(anchor));
                if lo != hi { chars.drain(lo as usize..hi as usize); caret = lo; }
                for (k, ch) in ins.iter().enumerate() { chars.insert(caret as usize + k, *ch); }
                caret += ins.len() as i32; anchor = caret;
            }
        }

        // --- Backspace / Delete ---
        if g.key_pressed(KEY_BACKSPACE) {
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if lo != hi { chars.drain(lo as usize..hi as usize); caret = lo; }
            else if caret > 0 { chars.remove(caret as usize - 1); caret -= 1; }
            anchor = caret;
        }
        if g.key_pressed(KEY_DELETE) {
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if lo != hi { chars.drain(lo as usize..hi as usize); caret = lo; }
            else if (caret as usize) < chars.len() { chars.remove(caret as usize); }
            anchor = caret;
        }

        // --- Navigation (Shift = Selektion erweitern) ---
        let len = chars.len() as i32;
        caret = caret.clamp(0, len); anchor = anchor.clamp(0, len);
        if g.key_pressed(KEY_LEFT) {
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if !shift && lo != hi { caret = lo; } else { caret = (caret - 1).max(0); }
            if !shift { anchor = caret; }
        }
        if g.key_pressed(KEY_RIGHT) {
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if !shift && lo != hi { caret = hi; } else { caret = (caret + 1).min(len); }
            if !shift { anchor = caret; }
        }
        if g.key_pressed(KEY_HOME) { caret = 0; if !shift { anchor = 0; } }
        if g.key_pressed(KEY_END) { caret = len; if !shift { anchor = len; } }

        // --- Horizontalen Scroll so anpassen, dass das Caret sichtbar bleibt ---
        let caret_px = g.text_width(&chars[..caret as usize].iter().collect::<String>());
        let inner = (fw - 10).max(1);
        let mut scroll = scroll;
        if caret_px - scroll > inner { scroll = caret_px - inner; }
        if caret_px - scroll < 0 { scroll = caret_px; }
        if scroll < 0 { scroll = 0; }

        // --- Zurueckschreiben ---
        let new_text: String = chars.iter().collect();
        let w = &mut self.windows[wi].widgets[i];
        w.text = new_text;
        w.caret = caret; w.sel_anchor = anchor; w.scroll = scroll;
        if w.text != before {
            if let Some(f) = w.on_change.clone() { self.pending.push(f); }
        }
    }

    /// Zeilenanfang-Indizes (Char-Position nach jedem '\n', plus 0).
    fn line_starts(chars: &[char]) -> Vec<usize> {
        let mut v = vec![0usize];
        for (i, &c) in chars.iter().enumerate() { if c == '\n' { v.push(i + 1); } }
        v
    }
    /// Zeilenhoehe der TextArea (Schriftgroesse + etwas Durchschuss).
    fn ta_line_h(&self, g: &Graphics) -> i32 { (g.text_height() + 5).max(10) }

    /// Mehrzeiliges Textfeld editieren: Tippen, Enter=Umbruch, Backspace/Delete,
    /// Pfeile (auch hoch/runter), Home/End, Strg+A/C/V/X, Maus-Klick + Ziehen.
    /// Selektion: Maus-Drag, Shift+Navigation, Strg+A. Vertikal scrollend, damit
    /// das Caret sichtbar bleibt.
    fn edit_textarea(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let before = self.windows[wi].widgets[i].text.clone();
        let mut chars: Vec<char> = before.chars().collect();
        let mut caret = self.windows[wi].widgets[i].caret.clamp(0, chars.len() as i32);
        let mut anchor = self.windows[wi].widgets[i].sel_anchor.clamp(0, chars.len() as i32);
        let ctrl = g.key_ctrl();
        let shift = g.key_shift();
        let pad = 5;
        let lh = self.ta_line_h(g);
        let (ax, ay, fw, fh) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let scroll = self.windows[wi].widgets[i].scroll;

        // Maus: Klick (steigende Flanke) setzt Caret+Anker, Ziehen erweitert die
        // Selektion bis zur aktuellen Position.
        if g.mouse_button(0) {
            let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
            let row = (scroll + ((my - ay - pad).max(0) / lh)).max(0);
            let starts = Self::line_starts(&chars);
            let r = (row as usize).min(starts.len().saturating_sub(1));
            let lstart = starts[r];
            let lend = if r + 1 < starts.len() { starts[r + 1] - 1 } else { chars.len() };
            let target = mx - (ax + pad);
            let sub: Vec<char> = chars[lstart..lend].to_vec();
            let off = Self::caret_index_at(g, &sub, target) as usize;
            let idx = (lstart + off) as i32;
            if !self.was_mouse_down {
                if Self::in_rect(mx, my, (ax, ay, fw, fh)) { caret = idx; anchor = idx; }
            } else {
                caret = idx;   // Ziehen -> Selektion bis hierher
            }
        }

        // Zeichen-Eingabe / Strg-Kombis (Selektion wird jeweils ersetzt).
        if !ctrl {
            let typed: String = g.pop_text_input().chars().filter(|c| !c.is_control()).collect();
            if !typed.is_empty() {
                let (lo, hi) = (caret.min(anchor), caret.max(anchor));
                if lo != hi { chars.drain(lo as usize..hi as usize); caret = lo; }
                for (k, ch) in typed.chars().enumerate() { chars.insert(caret as usize + k, ch); }
                caret += typed.chars().count() as i32; anchor = caret;
            }
        } else {
            if g.key_pressed(K_A) { anchor = 0; caret = chars.len() as i32; }
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if g.key_pressed(K_C) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
            }
            if g.key_pressed(K_X) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
                chars.drain(lo as usize..hi as usize); caret = lo; anchor = lo;
            }
            if g.key_pressed(K_V) {
                let ins: Vec<char> = g.clipboard_get().chars().filter(|c| *c == '\n' || !c.is_control()).collect();
                let (lo, hi) = (caret.min(anchor), caret.max(anchor));
                if lo != hi { chars.drain(lo as usize..hi as usize); caret = lo; }
                for (k, ch) in ins.iter().enumerate() { chars.insert(caret as usize + k, *ch); }
                caret += ins.len() as i32; anchor = caret;
            }
        }
        // Enter = Umbruch (ersetzt evtl. Selektion).
        if g.key_pressed(KEY_ENTER) {
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if lo != hi { chars.drain(lo as usize..hi as usize); caret = lo; }
            chars.insert(caret as usize, '\n'); caret += 1; anchor = caret;
        }
        // Backspace / Delete (Selektion hat Vorrang).
        if g.key_pressed(KEY_BACKSPACE) {
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if lo != hi { chars.drain(lo as usize..hi as usize); caret = lo; }
            else if caret > 0 { chars.remove(caret as usize - 1); caret -= 1; }
            anchor = caret;
        }
        if g.key_pressed(KEY_DELETE) {
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if lo != hi { chars.drain(lo as usize..hi as usize); caret = lo; }
            else if (caret as usize) < chars.len() { chars.remove(caret as usize); }
            anchor = caret;
        }

        // Navigation (Shift = Selektion erweitern, sonst Anker = Caret).
        let len = chars.len() as i32;
        caret = caret.clamp(0, len); anchor = anchor.clamp(0, len);
        let starts = Self::line_starts(&chars);
        let row = starts.iter().rposition(|&s| s as i32 <= caret).unwrap_or(0);
        let col = caret - starts[row] as i32;
        if g.key_pressed(KEY_LEFT) {
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if !shift && lo != hi { caret = lo; } else { caret = (caret - 1).max(0); }
            if !shift { anchor = caret; }
        }
        if g.key_pressed(KEY_RIGHT) {
            let (lo, hi) = (caret.min(anchor), caret.max(anchor));
            if !shift && lo != hi { caret = hi; } else { caret = (caret + 1).min(len); }
            if !shift { anchor = caret; }
        }
        if g.key_pressed(KEY_HOME) { caret = starts[row] as i32; if !shift { anchor = caret; } }
        if g.key_pressed(KEY_END) {
            caret = if row + 1 < starts.len() { starts[row + 1] as i32 - 1 } else { len };
            if !shift { anchor = caret; }
        }
        if g.key_pressed(KEY_UP) && row > 0 {
            let ps = starts[row - 1] as i32;
            let pe = starts[row] as i32 - 1;
            caret = (ps + col).min(pe);
            if !shift { anchor = caret; }
        }
        if g.key_pressed(KEY_DOWN) && row + 1 < starts.len() {
            let ns = starts[row + 1] as i32;
            let ne = if row + 2 < starts.len() { starts[row + 2] as i32 - 1 } else { len };
            caret = (ns + col).min(ne);
            if !shift { anchor = caret; }
        }

        // Vertikal scrollen, damit die Caret-Zeile sichtbar bleibt.
        let starts2 = Self::line_starts(&chars);
        let crow = starts2.iter().rposition(|&s| s as i32 <= caret).unwrap_or(0) as i32;
        let view_lines = ((fh - 2 * pad) / lh).max(1);
        let mut scroll = scroll;
        if crow < scroll { scroll = crow; }
        if crow >= scroll + view_lines { scroll = crow - view_lines + 1; }
        let max_scroll = (starts2.len() as i32 - view_lines).max(0);
        scroll = scroll.clamp(0, max_scroll);

        let new_text: String = chars.iter().collect();
        let w = &mut self.windows[wi].widgets[i];
        w.text = new_text; w.caret = caret; w.sel_anchor = anchor; w.scroll = scroll;
        if w.text != before {
            if let Some(f) = w.on_change.clone() { self.pending.push(f); }
        }
    }

    fn listbox_wheel(&mut self, wi: usize, i: usize, h: i32, g: &mut Graphics) {
        let wheel = g.pop_mouse_wheel();
        if wheel == 0 { return; }
        let w = &mut self.windows[wi].widgets[i];
        let max_scroll = (w.items.len() as i32 * DROPDOWN_ITEM_H - h).max(0);
        let nv = (w.value as i32 - wheel as i32 * DROPDOWN_ITEM_H).clamp(0, max_scroll);
        w.value = nv as f64;
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

    // --- Spinner (Zahlenfeld mit +/-) ---
    /// Rechtecke der Auf-/Ab-Schaltflaechen (rechte Spalte) + deren linke Kante.
    fn spinner_btn_rects(ax: i32, ay: i32, w: i32, h: i32)
        -> ((i32, i32, i32, i32), (i32, i32, i32, i32), i32) {
        let bw = 20;
        let bx = ax + w - bw;
        let mid = h / 2;
        ((bx, ay, bw, mid), (bx, ay + mid, bw, h - mid), bx)
    }

    /// Wert um `steps` Schritte aendern (geklemmt); feuert on_change bei Aenderung.
    fn spinner_step(&mut self, wi: usize, i: usize, steps: i32) {
        let w = &mut self.windows[wi].widgets[i];
        let nv = (w.value + steps as f64 * w.step).clamp(w.min, w.max);
        if nv != w.value {
            w.value = nv;
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Mausrad ueber dem Spinner: hoch = +, runter = -.
    fn spinner_wheel(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let wheel = g.pop_mouse_wheel();
        if wheel != 0 { self.spinner_step(wi, i, wheel as i32); }
    }

    /// Tastatur am fokussierten Spinner: Pfeil hoch/runter aendert den Wert.
    fn spinner_keys(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        if g.key_pressed(KEY_UP) { self.spinner_step(wi, i, 1); }
        if g.key_pressed(KEY_DOWN) { self.spinner_step(wi, i, -1); }
    }

    /// Zahl ohne ueberfluessige Nachkommastellen (ganze Zahl -> ohne Komma).
    fn fmt_num(v: f64) -> String {
        if (v.fract()).abs() < 1e-9 { format!("{}", v.round() as i64) }
        else { format!("{:.2}", v) }
    }

    /// Splitter ziehen: neue fenster-relative Position (geklemmt auf [min,max]),
    /// feuert on_change bei Aenderung.
    fn drag_split(&mut self, wi: usize, i: usize, mx: i32, my: i32) {
        let off = self.split_off;
        let (winx, winy) = (self.windows[wi].x, self.windows[wi].y);
        let vert = self.windows[wi].widgets[i].group == "v";
        let w = &mut self.windows[wi].widgets[i];
        let (lo, hi) = (w.min as i32, w.max as i32);
        let nv = if vert { (mx - winx - off).clamp(lo, hi) }
                 else { (my - winy - off).clamp(lo, hi) };
        let cur = if vert { w.x } else { w.y };
        if nv != cur {
            if vert { w.x = nv; } else { w.y = nv; }
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    fn dropdown_popup_rect(&self, wi: usize, idx: usize) -> (i32, i32, i32, i32) {
        let (ax, ay, w, h) = self.abs_rect(wi, &self.windows[wi].widgets[idx]);
        let n = self.windows[wi].widgets[idx].items.len() as i32;
        (ax, ay + h, w, n * DROPDOWN_ITEM_H)
    }

    fn handle_press(&mut self, mx: i32, my: i32) {
        // Offenes Dropdown hat Vorrang: das Popup liegt ueber allem und reicht
        // evtl. ueber den Fensterrand hinaus (topmost_at wuerde es verfehlen).
        if let Some((dw, di)) = self.open_dropdown {
            let valid = self.windows.get(dw).and_then(|w| w.widgets.get(di))
                .map(|x| x.alive && x.visible && x.kind == Kind::Dropdown).unwrap_or(false);
            if valid {
                let (px, py, pw, ph) = self.dropdown_popup_rect(dw, di);
                let (bx, by, bw, bh) = self.abs_rect(dw, &self.windows[dw].widgets[di]);
                if Self::in_rect(mx, my, (px, py, pw, ph)) {
                    let item = (my - py) / DROPDOWN_ITEM_H;
                    let n = self.windows[dw].widgets[di].items.len() as i32;
                    if item >= 0 && item < n {
                        let changed = self.windows[dw].widgets[di].sel != item;
                        self.windows[dw].widgets[di].sel = item;
                        if changed {
                            let f = self.windows[dw].widgets[di].on_change.clone();
                            if let Some(f) = f { self.pending.push(f); }
                        }
                    }
                    self.open_dropdown = None;
                    return;
                }
                self.open_dropdown = None;
                // Klick auf die zugeklappte Box selbst -> nur schliessen (kein Toggle-Reopen).
                if Self::in_rect(mx, my, (bx, by, bw, bh)) { return; }
            } else {
                self.open_dropdown = None;
            }
        }
        let win = match self.topmost_at(mx, my) {
            Some(w) => w,
            None => { self.focus_widget = None; return; }
        };
        self.bring_to_front(win);
        self.focus_window = Some(win);
        let th = self.m("title_h");
        let (wx, wy, ww, wh) = (self.windows[win].x, self.windows[win].y,
                                self.windows[win].w, self.windows[win].h);
        // Resize-Griff unten-rechts?
        let has_chrome = self.windows[win].chrome;
        let grip = 14;
        if has_chrome && self.windows[win].resizable
            && Self::in_rect(mx, my, (wx + ww - grip, wy + wh - grip, grip, grip)) {
            self.resize_window = Some(win);
            self.resize_dx = (wx + ww) - mx;
            self.resize_dy = (wy + wh) - my;
            return;
        }
        // Schliess-Button? (nur mit Chrome)
        if has_chrome && self.windows[win].closable
            && Self::in_rect(mx, my, (wx + ww - th, wy, th, th)) {
            self.windows[win].close_clicked = true;
            self.windows[win].visible = false;
            return;
        }
        // Titelleiste -> Drag? (nur mit Chrome)
        if has_chrome && Self::in_rect(mx, my, (wx, wy, ww, th)) {
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
            let (r, active) = { let w = &self.windows[win].widgets[i]; (self.abs_rect(win, w), self.widget_shown(win, w) && w.enabled) };
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
            Kind::Splitter => {
                self.active_split = Some((win, i));
                let vert = self.windows[win].widgets[i].group == "v";
                let (ax, ay, _w, _h) = self.abs_rect(win, &self.windows[win].widgets[i]);
                self.split_off = if vert { mx - ax } else { my - ay };
            }
            Kind::Spinner => {
                self.focus_widget = Some((win, i));
                let (ax, ay, ww, hh) = self.abs_rect(win, &self.windows[win].widgets[i]);
                let (up, down, _bx) = Self::spinner_btn_rects(ax, ay, ww, hh);
                if Self::in_rect(mx, my, up) { self.spinner_step(win, i, 1); }
                else if Self::in_rect(mx, my, down) { self.spinner_step(win, i, -1); }
            }
            Kind::TextInput => self.focus_widget = Some((win, i)),
            Kind::TextArea => self.focus_widget = Some((win, i)),
            Kind::Table => { self.focus_widget = None; self.table_press(win, i, mx, my); }
            Kind::Radio => {
                self.focus_widget = None;
                let was = self.windows[win].widgets[i].checked;
                self.select_radio(win, i);
                if !was {
                    let w = &self.windows[win].widgets[i];
                    let oc = w.on_click.clone(); let och = w.on_change.clone();
                    if let Some(f) = oc { self.pending.push(f); }
                    if let Some(f) = och { self.pending.push(f); }
                }
            }
            Kind::Dropdown => { self.focus_widget = None; self.open_dropdown = Some((win, i)); }
            Kind::ListBox => {
                self.focus_widget = None;
                let ay = self.abs_rect(win, &self.windows[win].widgets[i]).1;
                let scroll = self.windows[win].widgets[i].value as i32;
                let row = (my - ay + scroll) / DROPDOWN_ITEM_H;
                let n = self.windows[win].widgets[i].items.len() as i32;
                if row >= 0 && row < n && self.windows[win].widgets[i].sel != row {
                    self.windows[win].widgets[i].sel = row;
                    let f = self.windows[win].widgets[i].on_change.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
            _ => self.focus_widget = None,
        }
    }

    // --- Draw ---
    pub fn draw(&self, g: &mut Graphics) {
        for &wi in &self.z_order {
            if self.windows[wi].alive && self.windows[wi].visible { self.draw_window(g, wi); }
        }
        // Kontextmenue ueber ALLEM (nach allen Fenstern).
        if let Some((wi, mi, cx, cy)) = self.context_open {
            if self.windows.get(wi).map(|w| w.alive).unwrap_or(false) {
                self.draw_items_popup(g, wi, mi, cx, cy);
            }
        }
        // Tooltip ganz zuletzt (ueber allem), wenn die Maus lange genug ruht.
        self.draw_tooltip(g);
    }

    /// Zeichnet den Hilfetext des aktuell verweilten Widgets (nahe der Maus),
    /// sobald TOOLTIP_DELAY ruhende Frames vergangen sind. Mehrzeilig (\n),
    /// am Bildschirmrand zurueckgeklemmt.
    fn draw_tooltip(&self, g: &mut Graphics) {
        let (wi, i) = match self.hover_w { Some(p) => p, None => return };
        if self.frame_count - self.hover_frame < TOOLTIP_DELAY { return; }
        let text = match self.windows.get(wi).and_then(|w| w.widgets.get(i)) {
            Some(w) if !w.tooltip.is_empty() => w.tooltip.clone(),
            _ => return,
        };
        let pad = 6;
        let lh = (g.text_height() + 3).max(12);
        let lines: Vec<&str> = text.split('\n').collect();
        let tw = lines.iter().map(|l| g.text_width(l)).max().unwrap_or(0);
        let bw = tw + pad * 2;
        let bh = lines.len() as i32 * lh + pad * 2 - 3;
        // Position: rechts-unter dem Cursor, an den Bildschirmrand geklemmt.
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        let (sw, sh) = (g.screen_width() as i32, g.screen_height() as i32);
        let mut bx = mx + 14;
        let mut by = my + 18;
        if bx + bw > sw { bx = (mx - bw - 6).max(0); }
        if by + bh > sh { by = (my - bh - 6).max(0); }
        let rad = self.m("corner_radius").min(6);
        g.round_rect(bx + 2, by + 3, bx + bw - 1 + 2, by + bh - 1 + 3, rad.max(2), (0x55i64 << 24) as i64, true);
        g.round_rect(bx, by, bx + bw - 1, by + bh - 1, rad, self.th("widget_bg"), true);
        g.round_rect(bx, by, bx + bw - 1, by + bh - 1, rad, self.th("accent"), false);
        for (r, line) in lines.iter().enumerate() {
            g.text(bx + pad, by + pad + r as i32 * lh, line.to_string(), self.th("text_fg"));
        }
    }

    /// Layout der Menueleiste: (menu_idx, x_links_abs, x_rechts_abs) je Bar-Menue.
    fn menubar_slots(&self, g: &Graphics, wi: usize) -> Vec<(usize, i32, i32)> {
        let win = &self.windows[wi];
        let pad = self.m("pad");
        let mut x = win.x + pad;
        let mut out = Vec::new();
        for (mi, m) in win.menus.iter().enumerate() {
            if !m.in_bar { continue; }
            let wlbl = g.text_width(&m.label) + pad * 2;
            out.push((mi, x, x + wlbl));
            x += wlbl;
        }
        out
    }

    /// Zeichnet ein Menue-Popup (Dropdown/Kontext) mit Items ab (px, py).
    fn draw_items_popup(&self, g: &mut Graphics, wi: usize, mi: usize, px: i32, py: i32) {
        let m = match self.windows.get(wi).and_then(|w| w.menus.get(mi)) { Some(m) => m, None => return };
        let pad = self.m("pad");
        let mut wmax = 80;
        for it in &m.items { if !it.separator { wmax = wmax.max(g.text_width(&it.label) + pad * 4); } }
        let h = m.items.len() as i32 * MENU_ITEM_H + 4;
        let rad = self.m("corner_radius").min(6);
        // Schatten + Hintergrund + Rahmen.
        g.round_rect(px + 3, py + 4, px + wmax - 1 + 3, py + h - 1 + 4, rad.max(2), (0x44i64 << 24) as i64, true);
        g.round_rect(px, py, px + wmax - 1, py + h - 1, rad, self.th("widget_bg"), true);
        g.round_rect(px, py, px + wmax - 1, py + h - 1, rad, self.th("win_border"), false);
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        for (ii, it) in m.items.iter().enumerate() {
            let iy = py + 2 + ii as i32 * MENU_ITEM_H;
            if it.separator {
                g.line(px + 6, iy + MENU_ITEM_H / 2, px + wmax - 7, iy + MENU_ITEM_H / 2, self.th("win_border"));
                continue;
            }
            let hov = it.enabled && mx >= px && mx < px + wmax && my >= iy && my < iy + MENU_ITEM_H;
            if hov { g.box_fill(px + 2, iy, px + wmax - 3, iy + MENU_ITEM_H - 1, self.th("title_bg_focus")); }
            let fg = if it.enabled { self.th("text_fg") } else { self.th("muted_fg") };
            g.text(px + pad + 4, iy + (MENU_ITEM_H - 14) / 2, it.label.clone(), fg);
        }
    }

    fn draw_window(&self, g: &mut Graphics, wi: usize) {
        let win = &self.windows[wi];
        let focused = self.focus_window == Some(wi);
        let (x, y, w, h) = (win.x, win.y, win.w, win.h);
        let th = self.m("title_h");
        let pad = self.m("pad");
        let rad = self.m("corner_radius");
        let shadow = self.m("shadow");
        // Fenster-Schatten (modern): weiche, halbtransparente Lagen nach unten/rechts
        // versetzt -> das Fenster "schwebt" ueberm Hintergrund statt aufgeklebt.
        if shadow > 0 {
            for k in 0..3 {
                let off = 3 + k * 3;
                let grow = k * 2;
                let alpha = (0x3A - k * 0x14).max(0x10) as i64;
                let scol = (alpha << 24) as i64;       // schwarz mit Alpha (ARGB)
                g.round_rect(x - grow + off, y + off + 2,
                             x + w - 1 + grow + off, y + h - 1 + grow + off,
                             (rad + grow).max(0), scol, true);
            }
        }
        // Fenster-Hintergrund (rund, wenn corner_radius > 0).
        if rad > 0 { g.round_rect(x, y, x + w - 1, y + h - 1, rad, self.th("win_bg"), true); }
        else { g.box_fill(x, y, x + w - 1, y + h - 1, self.th("win_bg")); }
        let toff = if win.chrome { th } else { 0 };   // Inhalts-Oberkante
        if win.chrome {
            let title_bg = if focused { self.th("title_bg_focus") } else { self.th("title_bg") };
            if rad > 0 {
                // Titelleiste: oben gerundet (folgt der Fensterform), unten gerade.
                g.round_rect(x, y, x + w - 1, y + th - 1, rad, title_bg, true);
                g.box_fill(x, y + th - 1 - rad, x + w - 1, y + th - 1, title_bg);
                g.line(x + 1, y + th, x + w - 2, y + th, self.th("win_border"));  // Trennlinie
                g.round_rect(x, y, x + w - 1, y + h - 1, rad, self.th("win_border"), false);
            } else {
                g.box_fill(x, y, x + w - 1, y + th - 1, title_bg);
                g.rect(x, y, x + w - 1, y + h - 1, self.th("win_border"));
            }
            g.text(x + pad, y + (th - 14) / 2, win.title.clone(), self.th("title_fg"));
            if win.closable {
                let (cx, cy, cw, ch) = (x + w - th, y, th, th);
                let m = (th / 3).clamp(5, 9);          // Rand des X (skaliert mit der Titelhoehe)
                g.line(cx + m, cy + m, cx + cw - m - 1, cy + ch - m - 1, self.th("title_fg"));
                g.line(cx + cw - m - 1, cy + m, cx + m, cy + ch - m - 1, self.th("title_fg"));
            }
            if win.resizable {                   // Resize-Griff unten-rechts (Diagonalen)
                let c = self.th("win_border");
                for o in [2, 6, 10] {
                    g.line(x + w - 2 - o, y + h - 2, x + w - 2, y + h - 2 - o, c);
                }
            }
        }
        // Menueleiste (unter der Titelleiste), falls Bar-Menues vorhanden.
        let mboff = self.menubar_h(wi);
        if mboff > 0 {
            let by = y + toff;
            g.box_fill(x + 1, by, x + w - 2, by + mboff - 1, self.th("title_bg"));
            g.line(x + 1, by + mboff, x + w - 2, by + mboff, self.th("win_border"));
            for (mi, x0, x1) in self.menubar_slots(g, wi) {
                if self.open_menu == Some((wi, mi)) {
                    g.box_fill(x0, by, x1 - 1, by + mboff - 1, self.th("title_bg_focus"));
                }
                g.text(x0 + pad, by + (mboff - 14) / 2, win.menus[mi].label.clone(), self.th("title_fg"));
            }
        }
        // Reiter-Leiste (unter Titel/Menue), falls Tabs vorhanden.
        let tboff = self.tabbar_h(wi);
        if tboff > 0 {
            let by = y + toff + mboff;
            g.box_fill(x + 1, by, x + w - 2, by + tboff - 1, self.th("win_bg"));
            g.line(x + 1, by + tboff - 1, x + w - 2, by + tboff - 1, self.th("win_border"));
            for (ti, x0, x1) in self.tab_slots(g, wi) {
                let active = win.active_tab == ti as i32;
                if active {
                    g.box_fill(x0, by + 3, x1 - 1, by + tboff - 1, self.th("widget_bg"));
                    g.box_fill(x0, by + tboff - 2, x1 - 1, by + tboff - 1, self.th("accent"));  // Akzent-Unterstrich
                }
                let fg = if active { self.th("text_fg") } else { self.th("muted_fg") };
                g.text(x0 + pad, by + (tboff - 14) / 2, win.tabs[ti].clone(), fg);
            }
        }
        let coff = toff + mboff + tboff;   // Inhalts-Oberkante inkl. Menue + Reiter
        // Widgets auf den Fenster-Innenbereich begrenzen: wird das Fenster kleiner
        // gezogen, ragt nichts ueber den Rand/die Titelleiste/Menueleiste hinaus.
        g.push_clip(x + 1, y + coff, (w - 2).max(0), (h - coff - 1).max(0));
        for (i, wdg) in win.widgets.iter().enumerate() {
            if self.widget_shown(wi, wdg) { self.draw_widget(g, wi, i, wdg); }
        }
        g.pop_clip();
        // Inhalts-Scrollbalken (rechts), wenn der Inhalt hoeher als das Fenster ist.
        if let Some((tx, ty, bw, th, thy, thh)) = self.winscroll_geom(wi) {
            g.box_fill(tx, ty, tx + bw - 1, ty + th - 1, shade(self.th("win_bg"), -10));
            g.round_rect(tx + 1, thy, tx + bw - 2, thy + thh - 1, 4, self.th("widget_border"), true);
        }
        // Aufgeklapptes Menueleisten-Dropdown ueber den Widgets.
        if let Some((mw, mi)) = self.open_menu {
            if mw == wi {
                let slot = self.menubar_slots(g, wi).into_iter().find(|(m, _, _)| *m == mi);
                if let Some((_, x0, _)) = slot {
                    self.draw_items_popup(g, wi, mi, x0, y + toff + mboff);
                }
            }
        }
        // Aufgeklapptes Dropdown-Popup ueber allen Widgets dieses Fensters.
        if let Some((dw, di)) = self.open_dropdown {
            if dw == wi && win.widgets.get(di).map(|x| x.alive && x.visible).unwrap_or(false) {
                self.draw_dropdown_popup(g, wi, di);
            }
        }
    }

    fn draw_widget(&self, g: &mut Graphics, wi: usize, idx: usize, wdg: &Widget) {
        let (ax, ay, w, h) = self.abs_rect(wi, wdg);
        let pad = self.m("pad");
        match wdg.kind {
            Kind::Label => {
                let fg = if !wdg.enabled { self.th("muted_fg") }
                         else { wdg.ov.get("fg").copied().unwrap_or(wdg.color) };
                self.wtext(g, wdg, ax, ay, wdg.text.clone(), fg);
            }
            Kind::Panel => {
                self.fbox(g, ax, ay, ax + w - 1, ay + h - 1,
                    self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
                if !wdg.text.is_empty() {
                    g.box_fill(ax, ay, ax + w - 1, ay + 17, self.th("win_border"));
                    self.wtext(g, wdg, ax + 5, ay + 2, wdg.text.clone(), self.wcol(wdg, "fg", "title_fg"));
                }
            }
            Kind::Button => {
                let has_icon = wdg.sel >= 0;
                let icon_only = has_icon && wdg.text.is_empty();
                let pressed = self.press_origin == Some((wi, idx));
                if icon_only {
                    // Flacher Toolbar-Look: Flaeche nur bei Hover/Press.
                    let bgc = self.wcol(wdg, "bg", "widget_bg");
                    let hl = if pressed { Some(shade(bgc, -20)) }
                             else if wdg.hovered { Some(shade(bgc, 24)) } else { None };
                    if let Some(c) = hl {
                        let rad = self.m("corner_radius").min(6);
                        g.round_rect(ax, ay, ax + w - 1, ay + h - 1, rad, c, true);
                    }
                } else {
                    let mut bg = self.wcol(wdg, "bg", "widget_bg");
                    if pressed { bg = shade(bg, -30); } else if wdg.hovered { bg = shade(bg, 30); }
                    self.fbox(g, ax, ay, ax + w - 1, ay + h - 1, bg, self.wcol(wdg, "border", "widget_border"));
                }
                // Icon + optionaler Text.
                let isz = if has_icon {
                    if icon_only { (h - 8).min(w - 8).max(4) } else { (h - 10).max(4) }
                } else { 0 };
                if has_icon {
                    let iy = ay + (h - isz) / 2;
                    let ix = if icon_only { ax + (w - isz) / 2 } else { ax + pad };
                    g.draw_image_rect(wdg.sel as i64, ix, iy, isz, isz);
                }
                if !wdg.text.is_empty() {
                    let tx = if has_icon { ax + pad + isz + 4 } else { ax + pad };
                    self.wtext(g, wdg, tx, ay + (h - 14) / 2, wdg.text.clone(), self.txt_col(wdg));
                }
            }
            Kind::Checkbox => {
                let acc = self.acc_col(wdg);
                let bordc = self.wcol(wdg, "border", "widget_border");
                if self.modern() {
                    // Gerundetes Kaestchen; gefuellt + Haekchen wenn aktiv.
                    if wdg.checked {
                        g.round_rect(ax, ay, ax + w - 1, ay + h - 1, 3, acc, true);
                        let ck = self.th("win_bg");   // Kontrast-Haekchen auf Akzentflaeche
                        let (x1, y1) = (ax + w * 28 / 100, ay + h * 52 / 100);
                        let (x2, y2) = (ax + w * 44 / 100, ay + h * 70 / 100);
                        let (x3, y3) = (ax + w * 74 / 100, ay + h * 30 / 100);
                        g.line(x1, y1, x2, y2, ck);
                        g.line(x2, y2, x3, y3, ck);
                    } else {
                        g.round_rect(ax, ay, ax + w - 1, ay + h - 1, 3, self.wcol(wdg, "bg", "widget_bg"), true);
                        g.round_rect(ax, ay, ax + w - 1, ay + h - 1, 3, if wdg.hovered { acc } else { bordc }, false);
                    }
                } else {
                    g.rect(ax, ay, ax + w - 1, ay + h - 1, bordc);
                    if wdg.hovered { g.rect(ax - 1, ay - 1, ax + w, ay + h, acc); }
                    if wdg.checked { g.box_fill(ax + 3, ay + 3, ax + w - 4, ay + h - 4, acc); }
                }
                self.wtext(g, wdg, ax + w + pad, ay, wdg.text.clone(), self.txt_col(wdg));
            }
            Kind::Slider => {
                let handle_w = self.m("slider_handle_w");
                g.box_fill(ax, ay + h / 2 - 1, ax + w - 1, ay + h / 2 + 1, self.wcol(wdg, "bg", "widget_bg"));
                g.rect(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "border", "widget_border"));
                let span = wdg.max - wdg.min;
                let ratio = if span != 0.0 { (wdg.value - wdg.min) / span } else { 0.0 };
                let hx = ax + (ratio * (w - handle_w) as f64) as i32;
                g.box_fill(hx, ay, hx + handle_w - 1, ay + h - 1, self.acc_col(wdg));
            }
            Kind::TextInput => {
                let focused = self.focus_widget == Some((wi, idx));
                let fg = self.txt_col(wdg);
                let bcol = if focused { self.wcol(wdg, "accent", "accent") } else { self.wcol(wdg, "border", "widget_border") };
                self.fbox(g, ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"), bcol);
                let tx = ax + 5;
                let ty = ay + (h - 14) / 2;
                let scroll = wdg.scroll;
                // Inhalt auf das Feld-Innere clippen (langer Text laeuft nicht raus).
                g.push_clip(ax + 2, ay + 1, (w - 4).max(0), (h - 2).max(0));
                if wdg.text.is_empty() {
                    if !wdg.placeholder.is_empty() && !focused {
                        self.wtext(g, wdg, tx, ty, wdg.placeholder.clone(), self.th("muted_fg"));
                    }
                } else {
                    let chars: Vec<char> = wdg.text.chars().collect();
                    // Selektion-Highlight (halbtransparenter Akzent hinter dem Text).
                    if focused {
                        let lo = wdg.caret.min(wdg.sel_anchor).clamp(0, chars.len() as i32);
                        let hi = wdg.caret.max(wdg.sel_anchor).clamp(0, chars.len() as i32);
                        if lo != hi {
                            let x0 = tx + g.text_width(&chars[..lo as usize].iter().collect::<String>()) - scroll;
                            let x1 = tx + g.text_width(&chars[..hi as usize].iter().collect::<String>()) - scroll;
                            let acc = self.wcol(wdg, "accent", "accent");
                            let selbg = ((0x66i64) << 24) | (acc & 0xFFFFFF);   // ARGB, halbtransparent
                            g.box_fill(x0, ay + 2, x1, ay + h - 3, selbg);
                        }
                    }
                    self.wtext(g, wdg, tx - scroll, ty, wdg.text.clone(), fg);
                }
                // Caret (blinkend) an der gemessenen Position.
                if focused && (self.frame_count / self.m("caret_period").max(1) as i64) % 2 == 0 {
                    let pre: String = wdg.text.chars().take(wdg.caret.max(0) as usize).collect();
                    let cx = tx + g.text_width(&pre) - scroll;
                    g.line(cx, ay + 3, cx, ay + h - 4, fg);
                }
                g.pop_clip();
            }
            Kind::TextArea => {
                let focused = self.focus_widget == Some((wi, idx));
                let fg = self.txt_col(wdg);
                let bcol = if focused { self.wcol(wdg, "accent", "accent") } else { self.wcol(wdg, "border", "widget_border") };
                self.fbox(g, ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"), bcol);
                let pad = 5;
                let lh = self.ta_line_h(g);
                let scroll = wdg.scroll;
                g.push_clip(ax + 2, ay + 2, (w - 4).max(0), (h - 4).max(0));
                if wdg.text.is_empty() && !focused && !wdg.placeholder.is_empty() {
                    self.wtext(g, wdg, ax + pad, ay + pad, wdg.placeholder.clone(), self.th("muted_fg"));
                } else {
                    let chars: Vec<char> = wdg.text.chars().collect();
                    let starts = Self::line_starts(&chars);
                    let lines: Vec<&str> = wdg.text.split('\n').collect();
                    let view_lines = ((h - 2 * pad) / lh).max(1);
                    // Selektion-Highlight pro sichtbarer Zeile (halbtransparenter Akzent).
                    let lo = wdg.caret.min(wdg.sel_anchor).clamp(0, chars.len() as i32);
                    let hi = wdg.caret.max(wdg.sel_anchor).clamp(0, chars.len() as i32);
                    if focused && lo != hi {
                        let acc = self.wcol(wdg, "accent", "accent");
                        let selbg = ((0x66i64) << 24) | (acc & 0xFFFFFF);   // ARGB, halbtransparent
                        for r in 0..view_lines {
                            let li = scroll + r;
                            if li < 0 || li as usize >= starts.len() { continue; }
                            let lstart = starts[li as usize] as i32;
                            let lend = if (li as usize) + 1 < starts.len() {
                                starts[li as usize + 1] as i32 - 1
                            } else { chars.len() as i32 };
                            let a = lo.max(lstart);
                            let b = hi.min(lend);
                            if b < a || (a == b && hi <= lend) { continue; }
                            let x0 = ax + pad + g.text_width(&chars[lstart as usize..a as usize].iter().collect::<String>());
                            let mut x1 = ax + pad + g.text_width(&chars[lstart as usize..b as usize].iter().collect::<String>());
                            if hi > lend { x1 += g.text_width(" ").max(4); }   // Auswahl laeuft weiter
                            let y = ay + pad + r * lh;
                            g.box_fill(x0, y, x1, y + lh - 2, selbg);
                        }
                    }
                    for r in 0..view_lines {
                        let li = scroll + r;
                        if li < 0 || li as usize >= lines.len() { continue; }
                        self.wtext(g, wdg, ax + pad, ay + pad + r * lh, lines[li as usize].to_string(), fg);
                    }
                }
                if focused && (self.frame_count / self.m("caret_period").max(1) as i64) % 2 == 0 {
                    let chars: Vec<char> = wdg.text.chars().collect();
                    let starts = Self::line_starts(&chars);
                    let crow = starts.iter().rposition(|&s| s as i32 <= wdg.caret).unwrap_or(0);
                    let lstart = starts[crow];
                    let cend = (lstart + (wdg.caret - lstart as i32).max(0) as usize).min(chars.len());
                    let prefix: String = chars[lstart..cend].iter().collect();
                    let cx = ax + pad + g.text_width(&prefix);
                    let cy = ay + pad + (crow as i32 - scroll) * lh;
                    if cy >= ay + 2 && cy + lh <= ay + h { g.line(cx, cy, cx, cy + lh - 2, fg); }
                }
                g.pop_clip();
            }
            Kind::Spinner => {
                let focused = self.focus_widget == Some((wi, idx));
                let bcol = if focused { self.wcol(wdg, "accent", "accent") } else { self.wcol(wdg, "border", "widget_border") };
                self.fbox(g, ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"), bcol);
                let (up, down, bx) = Self::spinner_btn_rects(ax, ay, w, h);
                let bordc = self.wcol(wdg, "border", "widget_border");
                // Trennlinien vor und in der Buttonspalte.
                g.line(bx, ay + 1, bx, ay + h - 2, bordc);
                g.line(bx, ay + h / 2, ax + w - 1, ay + h / 2, bordc);
                // Zahlenwert links (auf das Feldinnere geclippt).
                g.push_clip(ax + 2, ay + 1, (bx - ax - 3).max(0), (h - 2).max(0));
                self.wtext(g, wdg, ax + pad, ay + (h - 14) / 2, Self::fmt_num(wdg.value), self.txt_col(wdg));
                g.pop_clip();
                // Hover-Highlight + Pfeil-Dreiecke der beiden Schaltflaechen.
                let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
                let fg = self.txt_col(wdg);
                let hl = shade(self.wcol(wdg, "bg", "widget_bg"), 30);
                if wdg.enabled && Self::in_rect(mx, my, up) {
                    g.box_fill(up.0 + 1, up.1 + 1, up.0 + up.2 - 2, up.1 + up.3 - 1, hl);
                }
                let (ucx, ucy) = (up.0 + up.2 / 2, up.1 + up.3 / 2);
                g.line(ucx - 4, ucy + 2, ucx, ucy - 2, fg);   // ^
                g.line(ucx, ucy - 2, ucx + 4, ucy + 2, fg);
                if wdg.enabled && Self::in_rect(mx, my, down) {
                    g.box_fill(down.0 + 1, down.1, down.0 + down.2 - 2, down.1 + down.3 - 2, hl);
                }
                let (dcx, dcy) = (down.0 + down.2 / 2, down.1 + down.3 / 2);
                g.line(dcx - 4, dcy - 2, dcx, dcy + 2, fg);   // v
                g.line(dcx, dcy + 2, dcx + 4, dcy - 2, fg);
            }
            Kind::Splitter => {
                let vert = wdg.group == "v";
                let active = self.active_split == Some((wi, idx));
                let col = if active || wdg.hovered { self.acc_col(wdg) }
                          else { self.wcol(wdg, "border", "widget_border") };
                g.box_fill(ax, ay, ax + w - 1, ay + h - 1, col);
                // Drei Griff-Punkte mittig laengs des Balkens.
                let dot = self.th("win_bg");
                let (cx, cy) = (ax + w / 2, ay + h / 2);
                for k in -1..=1 {
                    if vert { g.box_fill(cx - 1, cy + k * 5 - 1, cx + 1, cy + k * 5 + 1, dot); }
                    else { g.box_fill(cx + k * 5 - 1, cy - 1, cx + k * 5 + 1, cy + 1, dot); }
                }
            }
            Kind::Radio => {
                let acc = self.acc_col(wdg);
                let (cx, cy, r) = (ax + w / 2, ay + h / 2, (w / 2).max(2));
                g.circle(cx, cy, r, self.wcol(wdg, "border", "widget_border"));   // Ring
                g.circle(cx, cy, (r - 1).max(1), self.th("win_bg"));
                if wdg.hovered { g.circle(cx, cy, r, acc); g.circle(cx, cy, (r - 1).max(1), self.th("win_bg")); }
                if wdg.checked { g.circle(cx, cy, (r - 4).max(1), acc); }   // Punkt
                self.wtext(g, wdg, ax + w + pad, ay, wdg.text.clone(), self.txt_col(wdg));
            }
            Kind::Progress => {
                let acc = self.acc_col(wdg);
                self.fbox(g, ax, ay, ax + w - 1, ay + h - 1,
                    self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
                let span = wdg.max - wdg.min;
                let ratio = if span != 0.0 { ((wdg.value - wdg.min) / span).clamp(0.0, 1.0) } else { 0.0 };
                let fw = (ratio * (w - 2) as f64) as i32;
                if fw > 0 { g.box_fill(ax + 1, ay + 1, ax + fw, ay + h - 2, acc); }
                let pct = format!("{}%", (ratio * 100.0).round() as i32);
                self.wtext(g, wdg, ax + w / 2 - (pct.len() as i32 * 8) / 2, ay + (h - 14) / 2, pct, self.txt_col(wdg));
            }
            Kind::Dropdown => {
                let bg = self.wcol(wdg, "bg", "widget_bg");
                let b = if wdg.hovered { shade(bg, 18) } else { bg };
                self.fbox(g, ax, ay, ax + w - 1, ay + h - 1, b, self.wcol(wdg, "border", "widget_border"));
                let fg = self.txt_col(wdg);
                let txt = if wdg.sel >= 0 && (wdg.sel as usize) < wdg.items.len() {
                    wdg.items[wdg.sel as usize].clone()
                } else { String::new() };
                self.wtext(g, wdg, ax + pad, ay + (h - 14) / 2, txt, fg);
                let (axr, cy) = (ax + w - 14, ay + h / 2);   // ▼
                g.line(axr, cy - 2, axr + 4, cy + 2, fg);
                g.line(axr + 4, cy + 2, axr + 8, cy - 2, fg);
            }
            Kind::ListBox => {
                self.fbox(g, ax, ay, ax + w - 1, ay + h - 1,
                    self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
                let fg = self.txt_col(wdg);
                let acc = self.acc_col(wdg);
                let scroll = wdg.value as i32;
                let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
                g.push_clip(ax + 1, ay + 1, w - 2, h - 2);
                for (k, it) in wdg.items.iter().enumerate() {
                    let iy = ay + k as i32 * DROPDOWN_ITEM_H - scroll;
                    if iy + DROPDOWN_ITEM_H < ay || iy > ay + h { continue; }
                    if k as i32 == wdg.sel {
                        g.box_fill(ax + 1, iy, ax + w - 2, iy + DROPDOWN_ITEM_H - 1, shade(acc, -110));
                    } else if wdg.enabled && mx >= ax && mx < ax + w && my >= iy && my < iy + DROPDOWN_ITEM_H {
                        g.box_fill(ax + 1, iy, ax + w - 2, iy + DROPDOWN_ITEM_H - 1, shade(self.wcol(wdg, "bg", "widget_bg"), 22));
                    }
                    self.wtext(g, wdg, ax + pad, iy + (DROPDOWN_ITEM_H - 14) / 2, it.clone(), fg);
                }
                g.pop_clip();
            }
            Kind::Image => {
                if wdg.sel >= 0 { g.draw_image_rect(wdg.sel as i64, ax, ay, w, h); }
            }
            Kind::Canvas => {
                // Platzhalter-Flaeche; der User malt nach GUI_DRAW mit normalen
                // Befehlen in den per GUI_CANVAS_X/Y/W/H gelieferten Bereich.
                self.fbox(g, ax, ay, ax + w - 1, ay + h - 1,
                    self.wcol(wdg, "bg", "win_bg"), self.wcol(wdg, "border", "widget_border"));
            }
            Kind::Separator => {
                let my = ay + h / 2;
                g.line(ax, my, ax + w - 1, my, self.th("widget_border"));
            }
            Kind::Toolbar => {
                // Flacher Streifen + dezente Unterkante.
                g.box_fill(ax, ay, ax + w - 1, ay + h - 1, shade(self.th("win_bg"), 8));
                g.line(ax, ay + h - 1, ax + w - 1, ay + h - 1, self.th("win_border"));
            }
            Kind::GroupBox => {
                // Rahmen + eingelassener Titel oben-links (ueber dem Rahmen).
                g.rect(ax, ay + 7, ax + w - 1, ay + h - 1, self.th("widget_border"));
                if !wdg.text.is_empty() {
                    let tw = wdg.text.chars().count() as i32 * 8 + 8;
                    g.box_fill(ax + 8, ay, ax + 8 + tw.min(w - 16), ay + 13, self.th("win_bg"));
                    self.wtext(g, wdg, ax + 12, ay + 1, wdg.text.clone(), self.th("title_fg"));
                }
            }
            Kind::Table => self.draw_table(g, wi, idx),
        }
    }

    fn draw_dropdown_popup(&self, g: &mut Graphics, wi: usize, idx: usize) {
        let wdg = &self.windows[wi].widgets[idx];
        if wdg.items.is_empty() { return; }
        let (px, py, pw, ph) = self.dropdown_popup_rect(wi, idx);
        let bg = self.wcol(wdg, "bg", "widget_bg");
        let border = self.wcol(wdg, "border", "widget_border");
        let fg = self.wcol(wdg, "fg", "text_fg");
        let acc = self.wcol(wdg, "accent", "accent");
        let pad = self.m("pad");
        g.box_fill(px, py, px + pw - 1, py + ph - 1, bg);
        g.rect(px, py, px + pw - 1, py + ph - 1, border);
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        for (k, it) in wdg.items.iter().enumerate() {
            let iy = py + k as i32 * DROPDOWN_ITEM_H;
            let hovered = mx >= px && mx < px + pw && my >= iy && my < iy + DROPDOWN_ITEM_H;
            if k as i32 == wdg.sel {
                g.box_fill(px + 1, iy, px + pw - 2, iy + DROPDOWN_ITEM_H - 1, shade(acc, -110));
            } else if hovered {
                g.box_fill(px + 1, iy, px + pw - 2, iy + DROPDOWN_ITEM_H - 1, shade(bg, 22));
            }
            g.text(px + pad, iy + (DROPDOWN_ITEM_H - 14) / 2, it.clone(), fg);
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

    // Tooltip-Setter (ohne Graphics): Default leer, setzen/loeschen, Fehler bei
    // ungueltigem Handle.
    #[test]
    fn tooltip_set_clear_and_invalid() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 200, 200);
        let b = g.button(win, "OK".into(), 10, 10, 60, 24).unwrap();
        let (wi, i) = Gui::dec_widget(b);
        assert_eq!(g.windows[wi].widgets[i].tooltip, "");        // Default leer
        g.set_tooltip(b, "Hilfe".into()).unwrap();
        assert_eq!(g.windows[wi].widgets[i].tooltip, "Hilfe");
        g.set_tooltip(b, String::new()).unwrap();                // "" entfernt
        assert_eq!(g.windows[wi].widgets[i].tooltip, "");
        assert!(g.set_tooltip(999_999, "x".into()).is_err());    // ungueltiges Handle
    }

    // Spinner-Schrittlogik (ohne Graphics): klemmt auf [min,max], feuert
    // on_change nur bei echter Aenderung; max<=min ist ein Fehler.
    #[test]
    fn spinner_step_clamps_and_fires_change() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 300, 200);
        let sp = g.spinner(win, 10, 10, 120, 0.0, 10.0, 5.0, 2.0).unwrap();
        let (wi, i) = Gui::dec_widget(sp);
        g.on_change(sp, Some("changed".into())).unwrap();
        assert_eq!(g.value(sp).unwrap(), 5.0);
        g.spinner_step(wi, i, 1);                 // +2 -> 7
        assert_eq!(g.value(sp).unwrap(), 7.0);
        assert_eq!(g.take_pending(), vec!["changed".to_string()]);
        g.spinner_step(wi, i, 5);                 // +10 -> klemmt auf 10
        assert_eq!(g.value(sp).unwrap(), 10.0);
        g.take_pending();                         // Queue leeren
        g.spinner_step(wi, i, 0);                 // keine Aenderung -> kein Callback
        assert!(g.take_pending().is_empty());
        g.spinner_step(wi, i, -100);              // klemmt auf min 0
        assert_eq!(g.value(sp).unwrap(), 0.0);
        assert!(g.spinner(win, 0, 0, 100, 5.0, 5.0, 5.0, 1.0).is_err()); // max<=min
    }

    // Splitter-Drag (ohne Graphics): Position folgt der Maus (fenster-relativ,
    // via split_off), klemmt auf [min,max], feuert on_change; Validierung.
    #[test]
    fn splitter_drag_clamps_and_fires() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 100, 50, 400, 300);   // Fenster bei (100,50)
        let sp = g.splitter(win, 200, 20, 200, "v".into(), 80, 360).unwrap();
        let (wi, i) = Gui::dec_widget(sp);
        g.on_change(sp, Some("moved".into())).unwrap();
        assert_eq!(g.value(sp).unwrap(), 200.0);                 // Start-x
        g.split_off = 0;
        g.drag_split(wi, i, 350, 0);                             // x = 350-100-0 = 250
        assert_eq!(g.value(sp).unwrap(), 250.0);
        assert_eq!(g.take_pending(), vec!["moved".to_string()]);
        g.drag_split(wi, i, 9999, 0);                            // klemmt auf max 360
        assert_eq!(g.value(sp).unwrap(), 360.0);
        g.take_pending();
        g.drag_split(wi, i, 0, 0);                               // klemmt auf min 80
        assert_eq!(g.value(sp).unwrap(), 80.0);
        g.set_value(sp, 150.0).unwrap();                         // programmatisch
        assert_eq!(g.value(sp).unwrap(), 150.0);
        assert!(g.splitter(win, 0, 0, 100, "x".into(), 0, 100).is_err());   // Orientierung
        assert!(g.splitter(win, 0, 0, 100, "v".into(), 100, 100).is_err()); // max<=min
    }

    // Icon-Button = echter Button mit Textur-Handle in sel; set_icon nur auf
    // Buttons; Toolbar ist dekorativ (nicht interaktiv).
    #[test]
    fn icon_button_and_toolbar() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 200, 200);
        let b = g.icon_button(win, 10, 10, 40, 40, 7, String::new()).unwrap();
        let (wi, i) = Gui::dec_widget(b);
        assert!(matches!(g.windows[wi].widgets[i].kind, Kind::Button));   // echter Button
        assert_eq!(g.windows[wi].widgets[i].sel, 7);                     // Icon-Handle
        g.set_icon(b, 3).unwrap();
        assert_eq!(g.windows[wi].widgets[i].sel, 3);
        g.set_icon(b, -1).unwrap();                                      // entfernt
        assert_eq!(g.windows[wi].widgets[i].sel, -1);
        let sl = g.slider(win, 0, 60, 100, 0.0, 10.0, 5.0).unwrap();
        assert!(g.set_icon(sl, 1).is_err());                            // nur Buttons
        let tb = g.toolbar(win, 0, 0, 200, 32).unwrap();
        let (tw, ti) = Gui::dec_widget(tb);
        assert!(matches!(g.windows[tw].widgets[ti].kind, Kind::Toolbar));
        assert!(!g.windows[tw].widgets[ti].enabled);                     // dekorativ
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
