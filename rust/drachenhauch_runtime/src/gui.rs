//! Retained-Mode-GUI (Modul `gui`) -- nativer Port von
//! `drachenhauch/modules/gui.py`. Persistente Fenster/Widgets; pro Frame
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
use crate::value::{Rueckruf, Value};

#[path = "gui_dock.rs"]
mod dock;
#[path = "gui_ueber.rs"]
mod ueber;
#[path = "gui_stil.rs"]
mod stil;
#[path = "gui_bausteine.rs"]
mod bausteine;
#[path = "gui_rechtschreibung.rs"]
mod rechtschreibung;
#[path = "gui_richtung.rs"]
mod richtung;
// Die Methoden von `Gui` nach Themen; hier bleiben die Typen, die
// Konstruktion, Fenster-Flags und die Widget-Konstruktoren.
#[path = "gui_widgets.rs"]
mod widgets;
#[path = "gui_tabelle.rs"]
mod tabelle;
#[path = "gui_baeume.rs"]
mod baeume;
#[path = "gui_richtext.rs"]
mod richtext;
#[path = "gui_menues.rs"]
mod menues;
#[path = "gui_layout.rs"]
mod layout;
#[path = "gui_geometrie.rs"]
mod geometrie;
#[path = "gui_eingabe.rs"]
mod eingabe;
#[path = "gui_textbereich.rs"]
mod textbereich;
#[path = "gui_bedienung.rs"]
mod bedienung;
#[path = "gui_zeichnen.rs"]
mod zeichnen;
#[path = "gui_zeichnen_teile.rs"]
mod zeichnen_teile;
#[path = "gui_bindung.rs"]
mod bindung;
#[path = "gui_a11y.rs"]
mod a11y;

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
const KEY_SPACE: i64 = 32;
const KEY_ESC: i64 = 27;
const KEY_PAGEUP: i64 = 1073741899;
const KEY_PAGEDOWN: i64 = 1073741902;
const KEY_F2: i64 = 1073741883;
const KEY_F4: i64 = 1073741885;
const K_A: i64 = 97;
const K_B: i64 = 98;
const K_I: i64 = 105;
const K_U: i64 = 117;
const K_C: i64 = 99;
const K_V: i64 = 118;
const K_X: i64 = 120;
const K_Y: i64 = 121;
const K_Z: i64 = 122;

// Tabellen-Layout (analog zum Immediate-Mode-UI_TABLE).
const TBL_HEADER_H: i32 = 22;
const TBL_ROW_H: i32 = 20;
const TBL_SCROLL_W: i32 = 12;
const TBL_PADDING: i32 = 6;
const TBL_MIN_COL_W: i32 = 40;
const TBL_FILTER_H: i32 = 22;   // Hoehe der Filterzeile im Kopf
const TBL_EDGE_GRAB: i32 = 4;   // Fangbereich der Spaltenkante (px)
const DBL_TIME: f64 = 0.4;      // Doppelklick-Fenster (Sekunden)
const DBL_DIST: i32 = 4;        // erlaubter Versatz zwischen beiden Klicks
const HEAD_DRAG_MIN: i32 = 5;   // ab hier gilt eine Kopf-Geste als Zug

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
        // Plastik-Metriken: 0 = flach (bisheriges Verhalten).
        // gradient: Helligkeits-Abstand oben/unten in Farbstufen.
        // gloss:    Staerke der Glanzkante ueber der oberen Haelfte, 0..100.
        // bevel:    1 = helle Linie oben, dunkle unten (die Fase).
        ("gradient", 0), ("gloss", 0), ("bevel", 0),
        // verlauf_hoehe: ab dieser Hoehe klingt der Verlauf aus (0 = nie).
        // Ein Knopf von 28 Pixeln darf sich woelben; eine Liste von 400
        // bekommt mit derselben Staerke einen Schlagschatten ueber die halbe
        // Flaeche -- dieselbe Zahl sieht auf zwei Groessen verschieden aus.
        ("verlauf_hoehe", 0),
        // uebergang: Dauer in ms, mit der Ueberfahren, Druecken, Fokus und
        // Kippschalter weich nachlaufen (0 = springen). Das Endbild ist
        // dasselbe -- nur der Weg dorthin blendet.
        ("uebergang", 120),
    ].iter().map(|(k, v)| (k.to_string(), *v as i32)).collect()
}

/// Metrik-Profil eines Presets. Die "modern_*"-Presets aktivieren runde Ecken +
/// Fenster-Schatten + groessere Titelleiste (-> der professionelle Look); alle
/// anderen bleiben beim flachen Default. Wird in theme_preset() angewandt.
fn preset_metrics(name: &str) -> Option<HashMap<String, i32>> {
    // "glas_*": runde Ecken + Verlauf + Glanzkante + Fase -- der plastische
    // Look. Alles andere bleibt beim flachen Default, damit bestehende
    // Programme sich nicht veraendern.
    if name.starts_with("glas") {
        let mut m = default_metrics();
        m.insert("corner_radius".into(), 5);
        m.insert("title_h".into(), 30);
        m.insert("pad".into(), 8);
        m.insert("shadow".into(), 18);
        m.insert("gradient".into(), 16);
        m.insert("gloss".into(), 26);
        m.insert("bevel".into(), 1);
        m.insert("slider_h".into(), 16);
        return Some(m);
    }
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
            // Gedaempfter Text und Akzent nachgezogen (2026-09-06): 0x90969C
            // lag bei 2,76:1 auf dem Fensterhintergrund, unter WCAG AA (4,5:1);
            // 0x545B63 hat 6,3:1, auf Widget-Grund 4,9:1. Der Akzent von
            // 3,8:1 auf 5,6:1 (0x1F62B5) -- er ist Fokusring und Haekchen.
            ("widget_border", 0x9AA0A8), ("text_fg", 0x202428), ("muted_fg", 0x545B63),
            ("accent", 0x1F62B5), ("close_hover", 0xC04848)])),
        // Dunkles Glas: Anthrazit-Blau mit Cyan-Akzent -- der Look aus dem
        // Vorlagenbild (Verlaeufe + Glanz kommen aus dem Metrik-Profil).
        "glas_dunkel" | "glas_dark" => Some(m(&[
            ("win_bg", 0x232A33), ("win_border", 0x151A21), ("title_bg", 0x2C343F),
            ("title_bg_focus", 0x1C6E96), ("title_fg", 0xEAF2F8), ("widget_bg", 0x39424F),
            ("widget_border", 0x161B22), ("text_fg", 0xEAF2F8), ("muted_fg", 0x8B97A6),
            ("accent", 0x2FA8D8), ("close_hover", 0xC04848)])),
        // Helles Glas: Weiss/Silber mit demselben Blau.
        // Helles Glas: Weiss/Silber mit demselben Blau.
        //
        // Fensterflaeche bewusst deutlich GRAUER als die Widgets. Vorher lagen
        // beide bei ~0xF0 und die Knoepfe verschwanden schlicht im Untergrund;
        // im Dunkeln faellt das nicht auf, weil dort der Verlauf traegt, aber
        // auf Weiss hat ein weisser Verlauf fast keinen Kontrast. Die Raender
        // sind aus demselben Grund kraeftiger als im dunklen Thema.
        "glas_hell" | "glas_light" => Some(m(&[
            ("win_bg", 0xE2E8EF), ("win_border", 0x6E7C8C), ("title_bg", 0xD2DAE3),
            ("title_bg_focus", 0x2A8FD0), ("title_fg", 0x1E2530), ("widget_bg", 0xFBFCFE),
            ("widget_border", 0x63707F), ("text_fg", 0x1E2530), ("muted_fg", 0x66707C),
            ("accent", 0x2A8FD0), ("close_hover", 0xC04848)])),
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
        // "modern_dark" passt zum Drachenhauch-Editor (Anthrazit + Cyan).
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

/// Ein Uebergangswert 0..1 als weiche Kurve (smoothstep): langsam los, langsam
/// an. Linear gezeichnet wirkt ein Einblenden mechanisch -- der Anfang springt.
fn weich(t: f32) -> f64 {
    let t = t.clamp(0.0, 1.0) as f64;
    t * t * (3.0 - 2.0 * t)
}

/// Einen Uebergangswert um einen Schritt an sein Ziel heranfuehren.
fn naeher(t: &mut f32, ziel: f32, schritt: f32) {
    if *t < ziel { *t = (*t + schritt).min(ziel); } else { *t = (*t - schritt).max(ziel); }
}

/// Ein Zeilen-Uebergang: die Zeile unter der Maus blendet ein, die vorige
/// gleichzeitig aus. Zwei Plaetze reichen -- faehrt die Maus schnell ueber
/// viele Zeilen, blendet jeweils nur die zuletzt verlassene aus; das sieht man
/// nicht, und eine Liste mit tausend Zeilen braucht keine tausend Werte.
#[derive(Clone, Copy)]
struct Blende { akt: i32, akt_t: f32, alt: i32, alt_t: f32 }
impl Default for Blende {
    fn default() -> Self { Blende { akt: -1, akt_t: 0.0, alt: -1, alt_t: 0.0 } }
}
impl Blende {
    fn schritt(&mut self, jetzt: i32, s: f32) {
        if jetzt != self.akt {
            let (a, at) = (self.akt, self.akt_t);
            // Zurueck auf die Zeile, die gerade ausblendet: dort weiter,
            // statt bei 0 -- sonst flackert sie beim Hin und Her.
            self.akt_t = if jetzt >= 0 && jetzt == self.alt { self.alt_t } else { 0.0 };
            self.akt = jetzt;
            self.alt = a; self.alt_t = at;
        }
        if self.akt >= 0 { self.akt_t = (self.akt_t + s).min(1.0); }
        if self.alt >= 0 {
            self.alt_t = (self.alt_t - s).max(0.0);
            if self.alt_t <= 0.0 { self.alt = -1; }
        }
    }
    fn anteil(&self, k: i32) -> f64 {
        if k < 0 { return 0.0; }
        if k == self.akt { weich(self.akt_t) } else if k == self.alt { weich(self.alt_t) } else { 0.0 }
    }
}

/// Eine Farbe mit anteiliger Deckkraft (1 = wie sie ist, 0 = weg).
///
/// Alpha 0 heisst in dhrt DECKEND -- darum wird eine Farbe ohne Alpha als 255
/// gelesen, und ganz ausgeblendet wird 1 statt 0 geschrieben (sonst stuende
/// genau die unsichtbare Stufe als volle Farbe da).
fn deckkraft(c: i64, anteil: f64) -> i64 {
    let a0 = ((c >> 24) & 0xFF) as f64;
    let a0 = if a0 == 0.0 { 255.0 } else { a0 };
    let a = (a0 * anteil.clamp(0.0, 1.0)).round();
    if a >= 255.0 { return c & 0xFF_FFFF; }
    ((a.max(1.0) as i64) << 24) | (c & 0xFF_FFFF)
}

/// Zwei Farben mischen (`t` = 0 ganz `a`, 1 ganz `b`). Fuer Uebergaenge, die
/// einem Wert folgen -- etwa die Rinne des Kippschalters, die sich beim
/// Umlegen einfaerbt.
fn mischen(a: i64, b: i64, t: f64) -> i64 {
    // An den Enden unveraendert (samt Deckkraft): ein Uebergang, der gerade
    // ruht, darf eine halbdurchsichtige Farbe nicht deckend machen.
    if t <= 0.0 { return a; }
    if t >= 1.0 { return b; }
    let ch = |sh: u32| -> i64 {
        let (ca, cb) = (((a >> sh) & 0xFF) as f64, ((b >> sh) & 0xFF) as f64);
        ((ca + (cb - ca) * t).round() as i64).clamp(0, 255)
    };
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

/// Welche Schrift gilt: die eigene des Widgets, sonst die global aktive.
///
/// -1 am Widget heisst "keine eigene" und muss auf die AKTIVE fallen, nicht
/// auf die eingebaute Standardschrift. Sonst wechselt `GUI_SET_FONT_SIZE`
/// allein still die Schriftart -- wer eine Groesse setzt, meint aber die
/// Groesse.
fn font_wahl(eigen: i64, aktiv: i64) -> i64 {
    if eigen >= 0 { eigen } else { aktiv }
}

/// Waagerechte Lage der Knopf-Beschriftung im freien Bereich.
///
/// Liefert (Versatz, muss_beschnitten_werden). Passt der Text, sitzt er
/// mittig. Passt er NICHT, beginnt er am linken Rand und wird abgeschnitten
/// -- so bleibt die Form des Knopfes die Grenze, wie man es von Oberflaechen
/// kennt. Ihn stattdessen zentriert ueberstehen zu lassen wuerde ihn an
/// BEIDEN Seiten anschneiden und damit auch den Anfang unlesbar machen.
fn knopf_text_x(frei: i32, tw: i32) -> (i32, bool) {
    if tw > frei { (0, true) } else { ((frei - tw) / 2, false) }
}

/// Untere Kante der Glanzflaeche.
///
/// Bei RUNDEN Formen (Radius >= halbe Hoehe: Kreis oder Kapsel) nimmt der
/// Glanz die ganze Hoehe ein, sonst nur die obere Haelfte. Der Grund ist
/// `round_gradient`: es deckelt seinen Eckenradius auf die halbe Hoehe der
/// Flaeche, die es zeichnet. Eine halbhohe Glanzflaeche auf einem Kreis
/// bekaeme dadurch einen zu kleinen Radius und wuerde als flache Kapsel
/// seitlich ueber die runde Form hinausragen.
fn gloss_unten(y1: i32, y2: i32, rad: i32) -> i32 {
    if rad * 2 >= y2 - y1 { y2 } else { y1 + (y2 - y1) / 2 }
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
    Toggle, Knob,
    Toolbar, Tree,
    ColorPicker, DatePicker,
    Layout, TabControl, RichText, TimePicker, Accordion, Wizard,
    StatusBar, Breadcrumb, Card,
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
            Kind::Toolbar => "toolbar", Kind::Tree => "tree",
            Kind::Toggle => "toggle", Kind::Knob => "knob",
            Kind::ColorPicker => "colorpicker", Kind::DatePicker => "datepicker",
            Kind::Layout => "layout", Kind::TabControl => "tabcontrol",
            Kind::Accordion => "accordion", Kind::Wizard => "wizard",
            Kind::RichText => "richtext", Kind::TimePicker => "timepicker",
            Kind::StatusBar => "statusbar", Kind::Breadcrumb => "breadcrumb",
            Kind::Card => "card",
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
            "toolbar" => Kind::Toolbar, "tree" => Kind::Tree,
            "toggle" => Kind::Toggle, "knob" => Kind::Knob,
            "colorpicker" => Kind::ColorPicker, "datepicker" => Kind::DatePicker,
            "layout" => Kind::Layout, "tabcontrol" => Kind::TabControl,
            "accordion" => Kind::Accordion, "wizard" => Kind::Wizard,
            "richtext" => Kind::RichText, "timepicker" => Kind::TimePicker,
            "statusbar" => Kind::StatusBar, "breadcrumb" => Kind::Breadcrumb,
            "card" => Kind::Card,
            _ => return None,
        })
    }

    /// Kann dieses Widget den Tastatur-Fokus bekommen? Genau die Arten, die man
    /// auch mit der Maus bedient -- reine Deko (Label, Panel, Trennlinie,
    /// Gruppe, Werkzeugleiste), Anzeigen (Fortschritt, Bild) und die freie
    /// Zeichenflaeche bleiben aussen vor. EINE Quelle: der Tab-Zyklus, der
    /// Klick-Fokus und der Fokus-Ring fragen alle hier.
    fn fokussierbar(self) -> bool {
        !matches!(self,
            Kind::Label | Kind::Panel | Kind::Separator | Kind::GroupBox
            | Kind::Toolbar | Kind::Progress | Kind::Image | Kind::Canvas | Kind::Layout
            | Kind::StatusBar | Kind::Breadcrumb)
    }

    /// Widgets, die Text entgegennehmen -- dort darf die Leertaste NICHT
    /// aktivieren, sie ist ein Zeichen.
    fn nimmt_text(self) -> bool {
        matches!(self, Kind::TextInput | Kind::TextArea)
    }
}

/// Nimmt dieses Widget getippten Text? Wie `Kind::nimmt_text`, dazu die
/// bearbeitbare Klappliste -- dort gehoert eine Taste ohne Strg/Alt dem Feld,
/// nicht einem Menue-Kuerzel.
fn nimmt_text_w(w: &Widget) -> bool {
    w.kind.nimmt_text() || (w.kind == Kind::Dropdown && w.frei)
}

/// Braucht das Widget mit Fokus diese Taste (ohne Strg/Alt) selbst? Dann
/// gehoert sie ihm und nicht einem Menue-Kuerzel. Bis 2026.28 galt das nur
/// fuer Textfelder -- ein Kaestchen mit Fokus kippte bei der Leertaste UND
/// das Kuerzel "Leertaste" feuerte (gefunden beim Sequencer, Musik-Runde).
/// Leertaste und Enter loesen jedes bedienbare Widget aus, Pfeile und
/// Pos1/Ende/Bild bewegen darin, und wo Tippen springt (Listen, Baeume,
/// Tabellen, Klapplisten), gehoeren ihm auch Buchstaben und Ziffern.
fn widget_nimmt_taste(w: &Widget, code: i64) -> bool {
    if nimmt_text_w(w) { return true; }
    const AUSLOESEN: [i64; 3] = [32, 13, 1073741912];          // Leertaste, Enter
    const BEWEGEN: [i64; 8] = [1073741903, 1073741904, 1073741905, 1073741906,
                               1073741898, 1073741901, 1073741899, 1073741902];
    let k = w.kind;
    // Kaestchen, Schalter und Radio nehmen nur die Leertaste -- Enter gehoert
    // dort dem Standard-Knopf (wie unter Windows, macOS, Qt und im Web).
    if code == 32 && matches!(k, Kind::Checkbox | Kind::Radio | Kind::Toggle) { return true; }
    if AUSLOESEN.contains(&code) {
        return matches!(k, Kind::Button | Kind::Card
            | Kind::Dropdown | Kind::ListBox | Kind::Tree | Kind::Table | Kind::Accordion
            | Kind::TabControl | Kind::DatePicker | Kind::ColorPicker | Kind::TimePicker);
    }
    if BEWEGEN.contains(&code) {
        return matches!(k, Kind::Slider | Kind::Knob | Kind::Spinner | Kind::Radio | Kind::Dropdown
            | Kind::ListBox | Kind::Tree | Kind::Table | Kind::Accordion | Kind::TabControl
            | Kind::DatePicker | Kind::TimePicker | Kind::ColorPicker | Kind::RichText | Kind::Splitter);
    }
    // Wo Tippen springt, gehoeren auch Buchstaben und Ziffern dem Widget
    matches!(k, Kind::ListBox | Kind::Tree | Kind::Table | Kind::Dropdown)
        && ((97..=122).contains(&code) || (48..=57).contains(&code))
}

/// Bearbeitbare Klappliste: `sel` folgt dem Text -- der Eintrag, der genau so
/// heisst (Gross/klein egal), sonst -1.
fn dd_frei_abgleichen(w: &mut Widget) {
    let t = w.text.to_lowercase();
    w.sel = w.items.iter().position(|x| x.to_lowercase() == t).map(|k| k as i32).unwrap_or(-1);
}

/// Bearbeitbare Klappliste: den gewaehlten Eintrag ins Feld schreiben,
/// Schreibmarke ans Ende.
fn dd_frei_text_setzen(w: &mut Widget) {
    if !w.frei || w.sel < 0 || w.sel as usize >= w.items.len() { return; }
    w.text = w.items[w.sel as usize].clone();
    let n = w.text.chars().count() as i32;
    w.caret = n; w.sel_anchor = n; w.scroll = 0;
}

const DROPDOWN_ITEM_H: i32 = 22;
/// Breite des Pfeils einer bearbeitbaren Klappliste -- dort klappt ein Klick
/// auf, links davon setzt er die Schreibmarke.
const DD_PFEIL_B: i32 = 22;
/// Hoechstens so viele Eintraege zeigt eine offene Klappliste auf einmal; der
/// Rest rollt. Vorher war das Popup so hoch wie die Liste lang -- hundert
/// Eintraege ragten weit ueber den Bildschirm hinaus.
const DD_MAX_ZEILEN: i32 = 10;
const TC_KOPF_H: i32 = 26;        // Hoehe der Reiterkoepfe im Fenster
const AKK_KOPF_H: i32 = 30;       // Hoehe eines Akkordeon-Kopfes
const WZ_KOPF_H: i32 = 52;        // Schrittanzeige des Assistenten
const WZ_FUSS_H: i32 = 48;        // Knopfreihe des Assistenten
const TREE_ROW_H: i32 = 22;       // Hoehe einer Baum-Zeile
const TREE_INDENT: i32 = 16;      // Einrueckung pro Ebene
const TREE_TOGGLE_W: i32 = 16;    // Breite der Auf-/Zuklapp-Flaeche

/// Was in einer Zelle steckt. `Text` ist der Normalfall; die uebrigen Arten
/// zeichnen ein kleines Bedienelement in die Zelle (Stufe 2).
#[derive(Clone, Copy, PartialEq, Default)]
pub enum CellKind {
    #[default]
    Text,
    /// Bild, in die Zelle eingepasst (Seitenverhaeltnis bleibt).
    Image,
    /// Ankreuzfeld -- Klick schaltet um.
    Check,
    /// Fortschrittsbalken, `value` 0..1.
    Bar,
    /// Knopf -- Klick meldet sich ueber GUI_TABLE_CLICKED_COL.
    Button,
}

impl CellKind {
    fn parse(s: &str) -> Option<CellKind> {
        Some(match s.to_lowercase().as_str() {
            "text" => CellKind::Text,
            "bild" | "image" => CellKind::Image,
            "haken" | "check" | "checkbox" => CellKind::Check,
            "balken" | "bar" | "progress" => CellKind::Bar,
            "knopf" | "button" => CellKind::Button,
            _ => return None,
        })
    }
    fn name(self) -> &'static str {
        match self {
            CellKind::Text => "text", CellKind::Image => "bild",
            CellKind::Check => "haken", CellKind::Bar => "balken",
            CellKind::Button => "knopf",
        }
    }
}

/// Ausrichtung: 0 links, 1 mitte, 2 rechts. Eigener Wert je Zelle mit -1 =
/// "nimm die Spalte" -- so setzt man eine Zahlenspalte EINMAL rechtsbuendig,
/// kann aber einzelne Zellen ausscheren lassen.
/// Zahlenfilter des Textfelds: 1 = ganze Zahl (mit Vorzeichen), 2 = Kommazahl
/// (Punkt ODER Komma, hoechstens eines). Zwischenstaende wie "" und "-" sind
/// erlaubt -- sonst liesse sich eine negative Zahl gar nicht tippen.
/// Stil der Zeichen VOR einer Stelle -- damit schreibt man weiter, wie man
/// aufgehoert hat. Am Anfang einer Zeile gilt das Zeichen dahinter.
fn stil_vor(w: &Widget, k: usize) -> u8 {
    let chars: Vec<char> = w.text.chars().collect();
    if k > 0 && k <= w.stile.len() && chars.get(k - 1) != Some(&'\n') { return w.stile[k - 1]; }
    match chars.get(k) { Some(&c) if c != '\n' => w.stile.get(k).copied().unwrap_or(0), _ => 0 }
}

/// Marken (0-basierte Zeilen) von `alt` auf `neu` umrechnen. Gemeinsamer
/// Anfang und gemeinsames Ende grenzen die Aenderung ein: eine Zeile, die
/// HINTER ihr beginnt, rueckt um die Laengendifferenz mit (wer am Zeilenanfang
/// Enter drueckt, schiebt sie also eine Zeile tiefer), eine davor bleibt, und
/// eine, die IN der Aenderung begann, wird ueber die ZEILEN abgeglichen:
/// hat die Aenderung vorher und nachher gleich viele Zeilen (Umbenennen
/// ueber die ganze Datei), bleibt sie auf ihrer Zeile; sonst sucht eine
/// gemeinsame Teilfolge der Zeilen ihre neue Stelle, und erst eine Zeile,
/// die es nicht mehr gibt, landet am Anfang der Aenderung. Vorher fiel
/// JEDE Marke im Bereich auf den Anfang -- ein Umbau, der den ganzen Text
/// ersetzt, zog so einen Haltepunkt aus Zeile 10 auf Zeile 2.
fn zeilen_nachziehen(alt: &str, neu: &str, zeilen: &mut [usize]) {
    let a_ch: Vec<char> = alt.chars().collect();
    let n_ch: Vec<char> = neu.chars().collect();
    let mut a = 0;
    while a < a_ch.len() && a < n_ch.len() && a_ch[a] == n_ch[a] { a += 1; }
    let mut e = 0;
    while e < a_ch.len() - a && e < n_ch.len() - a
        && a_ch[a_ch.len() - 1 - e] == n_ch[n_ch.len() - 1 - e] { e += 1; }
    let ende_alt = a_ch.len() - e;
    let delta = n_ch.len() as i64 - a_ch.len() as i64;
    let mut starts = vec![0usize];
    for (i, &c) in a_ch.iter().enumerate() { if c == '\n' { starts.push(i + 1); } }
    let zeile_neu = |off: usize| n_ch[..off.min(n_ch.len())].iter().filter(|&&c| c == '\n').count();
    // Alles hinter der Aenderung rueckt um dieselbe Zahl Zeilen: Umbrueche im
    // neuen Stueck minus Umbrueche im alten. EINMAL gerechnet -- je Marke von
    // vorn gezaehlt kostete ein Tastendruck bei 2000 Hinweisen (Aufrufzaehler
    // der IDE) in einer Datei mit 20000 Zeilen rund 10 ms.
    let umbrueche = |ch: &[char]| ch.iter().filter(|&&c| c == '\n').count() as i64;
    let zeilen_delta = umbrueche(&n_ch[a..n_ch.len() - e]) - umbrueche(&a_ch[a..ende_alt]);
    for z in zeilen.iter_mut() {
        let o = match starts.get(*z) { Some(&o) => o, None => continue };
        if o >= ende_alt && o > a || (o == ende_alt && o == a && delta > 0) {
            *z = (*z as i64 + zeilen_delta).max(0) as usize;
        } else if o <= a {
            // davor: bleibt
        } else {
            *z = in_der_aenderung(&a_ch, &n_ch, a, ende_alt, n_ch.len() - e, *z)
                .unwrap_or_else(|| zeile_neu(a));
        }
    }
}

/// Neue Zeile einer Marke, deren Zeile in der Aenderung `a..ende_alt` (im
/// neuen Text `a..ende_neu`) lag; `None` = die Zeile gibt es nicht mehr.
fn in_der_aenderung(a_ch: &[char], n_ch: &[char], a: usize, ende_alt: usize,
                    ende_neu: usize, z: usize) -> Option<usize> {
    let zeile_von = |ch: &[char], off: usize| ch[..off.min(ch.len())].iter().filter(|&&c| c == '\n').count();
    let erste = zeile_von(a_ch, a);
    let letzte_alt = zeile_von(a_ch, ende_alt);
    let letzte_neu = zeile_von(n_ch, ende_neu);
    if z < erste || z > letzte_alt { return None; }
    // Gleich viele Zeilen: jede Zeile wurde an ihrem Platz geaendert.
    if letzte_alt == letzte_neu { return Some(z); }
    let zeilen = |ch: &[char], von: usize, bis: usize| -> Vec<String> {
        let ganz: String = ch.iter().collect();
        ganz.split('\n').skip(von).take(bis - von + 1).map(|t| t.trim().to_string()).collect()
    };
    let alt = zeilen(a_ch, erste, letzte_alt);
    let neu = zeilen(n_ch, erste, letzte_neu);
    let (n, m) = (alt.len(), neu.len());
    if n * m > 4_000_000 { return None; }
    // Laengste gemeinsame Teilfolge, von hinten gerechnet.
    let mut t = vec![0u32; (n + 1) * (m + 1)];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            t[i * (m + 1) + j] = if alt[i] == neu[j] { t[(i + 1) * (m + 1) + j + 1] + 1 }
                else { t[(i + 1) * (m + 1) + j].max(t[i * (m + 1) + j + 1]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if alt[i] == neu[j] {
            if erste + i == z { return Some(erste + j); }
            i += 1; j += 1;
        } else if t[(i + 1) * (m + 1) + j] >= t[i * (m + 1) + j + 1] { i += 1; } else { j += 1; }
    }
    None
}

/// Marken eines Textbereichs an seinen jetzigen Text anpassen (siehe
/// `marken`). Billig, wenn sich nichts geaendert hat.
/// Deckkraft einer Zeilenfarbe: 0 im obersten Byte wuerde den Text
/// verdecken (0 = deckend), also dann ein Viertel.
fn zeilenfarbe_deckkraft(c: i64) -> i64 {
    if (c >> 24) & 0xFF == 0 { 0x40_000000 | (c & 0xFF_FFFF) } else { c & 0xFFFF_FFFF }
}

/// Hoechstens so viele Schritte im Rueckgaengig-Verlauf ...
const VERLAUF_SCHRITTE: usize = 100;
/// ... und hoechstens so viele Bytes darin. Jeder Schritt haelt den GANZEN
/// Text vorher; 100 Schritte an einer Datei mit 2,7 MB waren 270 MB, an
/// einer mit 30 MB drei Gigabyte (Stresstest 2026-10-02). Der juengste
/// Schritt bleibt immer, auch wenn er allein groesser ist -- sonst liesse
/// sich gerade das grosse Einfuegen nicht zuruecknehmen.
const VERLAUF_BYTES: usize = 64 * 1024 * 1024;

/// Die aeltesten Schritte weg, bis Zahl und Groesse passen.
fn verlauf_kuerzen(v: &mut Vec<(String, i32, Vec<u8>)>) {
    let mut summe: usize = v.iter().map(|s| s.0.len() + s.2.len()).sum();
    let mut weg = 0;
    while v.len() - weg > 1 && (v.len() - weg > VERLAUF_SCHRITTE || summe > VERLAUF_BYTES) {
        summe -= v[weg].0.len() + v[weg].2.len();
        weg += 1;
    }
    if weg > 0 { v.drain(..weg); }
}

#[cfg(test)]
mod verlauf_tests {
    use super::{verlauf_kuerzen, VERLAUF_BYTES, VERLAUF_SCHRITTE};

    #[test]
    fn kleine_texte_behalten_hundert_schritte() {
        let mut v: Vec<(String, i32, Vec<u8>)> = Vec::new();
        for i in 0..150 { v.push((format!("t{}", i), i, Vec::new())); verlauf_kuerzen(&mut v); }
        assert_eq!(v.len(), VERLAUF_SCHRITTE);
        assert_eq!(v[0].0, "t50");
        assert_eq!(v.last().unwrap().0, "t149");
    }

    #[test]
    fn grosse_texte_halten_das_budget() {
        let gross = "x".repeat(10 * 1024 * 1024);
        let mut v: Vec<(String, i32, Vec<u8>)> = Vec::new();
        for i in 0..20 { v.push((gross.clone(), i, Vec::new())); verlauf_kuerzen(&mut v); }
        let summe: usize = v.iter().map(|s| s.0.len()).sum();
        assert!(summe <= VERLAUF_BYTES);
        assert_eq!(v.len(), VERLAUF_BYTES / gross.len());
        // Die juengsten bleiben.
        assert_eq!(v.last().unwrap().1, 19);
    }

    #[test]
    fn ein_riesiger_schritt_bleibt_allein() {
        let riesig = "y".repeat(VERLAUF_BYTES + 1);
        let mut v: Vec<(String, i32, Vec<u8>)> = vec![("a".into(), 0, Vec::new())];
        v.push((riesig, 1, Vec::new()));
        verlauf_kuerzen(&mut v);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].1, 1);
    }
}

#[cfg(test)]
mod zeilenfarben_tests {
    use super::zeilenfarbe_deckkraft;
    #[test]
    fn ohne_deckkraft_ein_viertel() {
        assert_eq!(zeilenfarbe_deckkraft(0xFF3030), 0x40FF3030);
        assert_eq!(zeilenfarbe_deckkraft(0x80FF3030), 0x80FF3030);
    }
}

fn marken_abgleichen(w: &mut Widget) {
    // Die Werte hinter den Zeilen (GUI_TEXTAREA_HINTS) wandern genauso mit.
    if !w.hinweise.is_empty() && w.hinweise_text != w.text {
        let mut z: Vec<usize> = w.hinweise.iter().map(|m| m.0).collect();
        zeilen_nachziehen(&w.hinweise_text, &w.text, &mut z);
        for (m, n) in w.hinweise.iter_mut().zip(z) { m.0 = n; }
        w.hinweise_text = w.text.clone();
    }
    if !w.zeilenfarben.is_empty() && w.zeilenfarben_text != w.text {
        let mut z: Vec<usize> = w.zeilenfarben.iter().map(|m| m.0).collect();
        zeilen_nachziehen(&w.zeilenfarben_text, &w.text, &mut z);
        for (m, n) in w.zeilenfarben.iter_mut().zip(z) { m.0 = n; }
        w.zeilenfarben_text = w.text.clone();
    }
    if w.marken.is_empty() || w.marken_text == w.text { return; }
    let mut z: Vec<usize> = w.marken.iter().map(|m| m.0).collect();
    zeilen_nachziehen(&w.marken_text, &w.text, &mut z);
    for (m, n) in w.marken.iter_mut().zip(z) { m.0 = n; }
    w.marken_text = w.text.clone();
}

/// Stile an einen geaenderten Text anpassen: gemeinsamer Anfang und
/// gemeinsames Ende behalten ihre Bits, das Neue dazwischen bekommt den
/// Tippstil bzw. den Stil davor. Das deckt Tippen, Loeschen, Einfuegen und
/// Ersetzen ab, ohne dass jeder dieser Wege die Bits kennen muss.
fn stile_abgleichen(w: &mut Widget) {
    if !w.formatiert { return; }
    let neu: Vec<char> = w.text.chars().collect();
    if w.stile_text == w.text && w.stile.len() == neu.len() { return; }
    let alt: Vec<char> = w.stile_text.chars().collect();
    if w.stile.len() != alt.len() { w.stile = vec![0; alt.len()]; }
    let mut a = 0;
    while a < alt.len() && a < neu.len() && alt[a] == neu[a] { a += 1; }
    let mut e = 0;
    while e < alt.len() - a && e < neu.len() - a && alt[alt.len() - 1 - e] == neu[neu.len() - 1 - e] { e += 1; }
    let neu_st = w.tipp_stil.unwrap_or_else(|| {
        if a > 0 && alt[a - 1] != '\n' { w.stile[a - 1] }
        else if a < alt.len() - e && alt[a] != '\n' { w.stile[a] }
        else if a > 0 { w.stile[a - 1] } else { 0 }
    });
    let mut st = Vec::with_capacity(neu.len());
    st.extend_from_slice(&w.stile[..a]);
    st.extend(std::iter::repeat(neu_st).take(neu.len() - a - e));
    st.extend_from_slice(&w.stile[alt.len() - e..]);
    w.stile = st;
    w.stile_text = w.text.clone();
}

/// Text + Stile -> Markdown: `**fett**`, `*kursiv*`, `***beides***`,
/// `~~durch~~`, `<u>unter</u>`. Geschrieben werden nur WECHSEL -- was
/// endet, dann was anfaengt; fett und kursiv zusammen in EINEM Zeichen,
/// sonst stuenden `**` und `*` nebeneinander und laesen sich als `***`.
/// Zeichen mit eigener Bedeutung bekommen einen Rueckstrich.
pub(crate) fn markdown_aus(chars: &[char], stile: &[u8]) -> String {
    use crate::schnitt::{DURCH, FETT, KURSIV, UNTER};
    fn fk(bits: u8) -> &'static str {
        match bits & (FETT | KURSIV) { x if x == FETT | KURSIV => "***", FETT => "**", KURSIV => "*", _ => "" }
    }
    let mut out = String::new();
    let mut akt = 0u8;
    let wechsel = |out: &mut String, akt: u8, neu: u8| {
        let aus = akt & !neu;
        let ein = neu & !akt;
        // Zuerst schliessen, in umgekehrter Reihenfolge des Oeffnens.
        out.push_str(fk(aus));
        if aus & DURCH != 0 { out.push_str("~~"); }
        if aus & UNTER != 0 { out.push_str("</u>"); }
        if ein & UNTER != 0 { out.push_str("<u>"); }
        if ein & DURCH != 0 { out.push_str("~~"); }
        out.push_str(fk(ein));
    };
    for (k, &c) in chars.iter().enumerate() {
        let st = stile.get(k).copied().unwrap_or(0);
        if st != akt { wechsel(&mut out, akt, st); akt = st; }
        match c {
            '\\' | '*' | '~' => { out.push('\\'); out.push(c); }
            '<' => {
                let rest: String = chars[k..chars.len().min(k + 4)].iter().collect();
                if rest.starts_with("<u>") || rest.starts_with("</u>") { out.push('\\'); }
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    wechsel(&mut out, akt, 0);
    out
}

/// Markdown -> Text + Stile. Die Marken SCHALTEN um (`**` kippt fett,
/// `***` fett und kursiv, `~~` durch), `<u>` schaltet ein, `</u>` aus; ein
/// Rueckstrich nimmt das naechste Zeichen woertlich. Ein einzelnes `~` ist
/// ein Zeichen. So liest es alles, was `markdown_aus` schreibt, und das
/// uebliche Markdown von Hand ebenso.
pub(crate) fn markdown_ein(md: &str) -> (String, Vec<u8>) {
    use crate::schnitt::{DURCH, FETT, KURSIV, UNTER};
    let z: Vec<char> = md.chars().collect();
    let (mut text, mut st) = (String::new(), Vec::new());
    let mut akt = 0u8;
    let mut k = 0;
    while k < z.len() {
        let c = z[k];
        if c == '\\' && k + 1 < z.len() {
            text.push(z[k + 1]); st.push(akt); k += 2; continue;
        }
        if c == '*' {
            let mut n = 0;
            while k + n < z.len() && z[k + n] == '*' { n += 1; }
            k += n;
            while n > 0 {
                let m = n.min(3);
                akt ^= match m { 3 => FETT | KURSIV, 2 => FETT, _ => KURSIV };
                n -= m;
            }
            continue;
        }
        if c == '~' && z.get(k + 1) == Some(&'~') { akt ^= DURCH; k += 2; continue; }
        if c == '<' {
            let rest: String = z[k..z.len().min(k + 4)].iter().collect();
            if rest.starts_with("<u>") { akt |= UNTER; k += 3; continue; }
            if rest.starts_with("</u>") { akt &= !UNTER; k += 4; continue; }
        }
        text.push(c); st.push(akt); k += 1;
    }
    (text, st)
}

fn zahl_erlaubt(text: &str, zahlen: u8) -> bool {
    if zahlen == 0 { return true; }
    let mut trenner = 0;
    for (i, ch) in text.chars().enumerate() {
        if ch.is_ascii_digit() { continue; }
        if ch == '-' && i == 0 { continue; }
        if zahlen >= 2 && (ch == '.' || ch == ',') { trenner += 1; if trenner > 1 { return false; } continue; }
        return false;
    }
    true
}

fn parse_align(s: &str) -> Option<i8> {
    Some(match s.to_lowercase().as_str() {
        "links" | "left" => 0,
        "mitte" | "zentriert" | "center" => 1,
        "rechts" | "right" => 2,
        _ => return None,
    })
}

#[derive(Clone, Default)]
pub struct Cell {
    text: String,
    /// -1 = Farbe des Themas (nicht eingefroren -- ein Themenwechsel wirkt).
    fg: i64,
    /// -1 = keine eigene Flaeche (Zeilenfarbe bzw. Zebra schlaegt durch).
    bg: i64,
    kind: CellKind,
    /// Bild-Handle fuer `CellKind::Image`, sonst -1.
    image: i64,
    /// Haken (0/1) bzw. Fortschritt (0..1).
    value: f64,
    /// -1 = Ausrichtung der Spalte uebernehmen.
    align: i8,
}

impl Cell {
    fn text(s: String) -> Cell {
        Cell { text: s, fg: -1, bg: -1, kind: CellKind::Text, image: -1, value: 0.0, align: -1 }
    }
}

pub struct TableState {
    headers: Vec<String>,
    rows: Vec<Vec<Cell>>,
    col_widths: Option<Vec<i32>>,
    /// Ausrichtung je Spalte (0/1/2). Kuerzer als die Spaltenzahl = Rest links.
    col_align: Vec<i8>,
    /// Eigene Farben je Zeile (-1 = keine). Liegt getrennt von den Zellen,
    /// damit "ganze Zeile rot" ein Aufruf bleibt und nicht n Aufrufe.
    row_fg: Vec<i64>,
    row_bg: Vec<i64>,
    row_h: i32,
    header_h: i32,
    // Hoehe der Filterzeile. Ein FELD und keine Konstante, weil `head_total`
    // auf TableState sitzt und den Anzeige-Massstab nicht kennt -- gesetzt
    // wird sie beim Anlegen der Tabelle (wie row_h/header_h).
    filter_h: i32,
    zebra: bool,
    grid: bool,
    scroll_x: i32, scroll_y: i32,
    drag_v: bool, drag_h: bool, drag_off: i32,
    selected: i32, hover_row: i32, clicked_row: i32,
    /// Doppelklick (oder Enter im Zeilenmodus) in DIESEM Bild -- "oeffnen",
    /// wie bei der Liste; GUI_DOUBLE_CLICKED fragt es.
    doppel: bool,
    /// Hinweis, wenn keine Zeile zu sehen ist ("Keine Treffer").
    leer_text: String,
    /// Tipp-Puffer fuer "tippen springt" (Zeilenmodus).
    tipp: String,
    tipp_zeit: f64,
    /// Spalte unter der Maus beim Druecken.
    hover_col: i32,
    /// Spalte des letzten Klicks (-1 = keine) -- fuer Knopf-Zellen.
    clicked_col: i32,

    // --- Sortierung und Filter ---------------------------------------------
    /// Sortierspalte (-1 = unsortiert) und Richtung.
    sort_col: i32,
    sort_desc: bool,
    /// Sortieren per Kopfklick erlauben.
    sortable: bool,
    /// Filtertext je Spalte (leer = kein Filter).
    filters: Vec<String>,
    /// Filterzeile unter der Kopfzeile zeigen.
    filter_row: bool,
    /// Welche Filterzelle die Tastatur bekommt (-1 = keine).
    filter_focus: i32,
    /// Wieviele Spalten von links bleiben beim Waagerecht-Scrollen stehen.
    frozen: i32,

    // --- Spalten-Reihenfolge -------------------------------------------------
    /// Anzeige-Position -> DATENspalte. Leer = unveraendert.
    ///
    /// Wie bei den Zeilen wird auch hier nichts umgestellt: `GUI_TABLE_GET_CELL`
    /// und alle anderen Angaben meinen weiter die DATENspalte. Wuerde man die
    /// Daten wirklich umsortieren, zeigte jede Spaltennummer, die ein Programm
    /// sich gemerkt hat, nach dem ersten Zug auf etwas anderes.
    col_order: Vec<usize>,
    /// Spalten mit der Maus umsortieren erlauben. Per Vorgabe AUS: ein
    /// versehentlicher Zug am Kopf wuerde sonst die Anordnung zerwuerfeln, und
    /// zurueck geht es nur von Hand.
    reorderable: bool,
    /// Laufender Kopf-Zug: (Anzeige-Position, Start-x, schon bewegt?).
    /// Erst beim LOSLASSEN entscheidet sich, ob es ein Sortier-Klick war oder
    /// ein Verschieben -- sonst wuerde jeder Zug auch noch sortieren.
    head_drag: Option<(usize, i32, bool)>,

    // --- Mehrfachauswahl -----------------------------------------------------
    /// Erlaubt (sonst verhaelt sich die Tabelle wie bisher: eine Zeile).
    multi: bool,
    /// Alle gewaehlten DATENzeilen. `selected` bleibt daneben die zuletzt
    /// angeklickte -- damit funktioniert vorhandener Code unveraendert weiter,
    /// der nur GUI_TABLE_SELECTED kennt.
    sel_rows: Vec<i32>,
    /// Ankerzeile fuer die Umschalt-Auswahl (Bereich von hier bis zum Klick).
    sel_anchor: i32,

    // --- Zelle bearbeiten ---------------------------------------------------
    /// Welche Zelle gerade bearbeitet wird (-1 = keine).
    edit_row: i32,
    edit_col: i32,
    /// Arbeitskopie waehrend der Bearbeitung. Die Zelle selbst wird erst beim
    /// Bestaetigen geschrieben -- sonst koennte Escape nichts mehr zuruecknehmen,
    /// und jede Taste wuerde Sortierung und Filter neu anwerfen (die Zeile
    /// spraenge einem beim Tippen unter dem Finger weg).
    edit_text: String,
    edit_caret: i32,
    /// Anker der Textauswahl. Zusammen mit `edit_caret` die markierte Stelle;
    /// beide gleich = nichts markiert.
    edit_anchor: i32,
    /// Waagerechter Versatz, damit die Schreibmarke sichtbar bleibt.
    edit_scroll: i32,
    /// Maus ignorieren, solange der OEFFNENDE Doppelklick noch gedrueckt ist.
    ///
    /// Ohne das las der Editor denselben Klick, der ihn geoeffnet hat, sofort
    /// als "Marke hierhin setzen" -- und hob damit die Rundum-Markierung auf,
    /// die das Ueberschreiben erst moeglich macht. Ergebnis: die Ruecktaste
    /// loeschte EIN Zeichen statt des Inhalts.
    edit_maus_sperre: bool,
    /// Welche Spalten bearbeitet werden duerfen. Kuerzer als die Spaltenzahl
    /// = der Rest ist gesperrt.
    col_edit: Vec<bool>,

    /// Spalte, deren rechte Kante gerade gezogen wird (-1 = keine).
    drag_col: i32,
    /// Breite dieser Spalte beim Anfassen + Maus-x -- daraus ergibt sich die
    /// neue Breite. Ohne den Startwert wuerde die Spalte beim Anfassen an den
    /// Mauszeiger springen.
    drag_col_w: i32,
    drag_col_x: i32,
    /// Spaltenbreiten mit der Maus aendern erlauben.
    resizable_cols: bool,
    /// Sichtbare Zeilen als Verweis auf `rows` -- das Ergebnis von Filter und
    /// Sortierung. Die Daten selbst werden NIE umgestellt: sonst zeigten
    /// Zeilennummern, die das Programm sich gemerkt hat, ploetzlich auf etwas
    /// anderes. Alle Zeilenangaben nach aussen bleiben Datenzeilen.
    view: Vec<usize>,

    // --- Gitter (Stufe 30) ---------------------------------------------------
    /// Zellmodus: eine AKTUELLE ZELLE statt einer gewaehlten Zeile, Bereich
    /// mit Umschalt, Tippen bearbeitet, Strg+C/V als Tabulator-Text. Per
    /// Vorgabe aus -- eine Tabelle, die Zeilen zeigt, bleibt, was sie war.
    zellmodus: bool,
    /// Aktuelle Zelle als DATENzeile/-spalte (-1 = keine) und der Anker des
    /// Bereichs. Der Bereich ist das Rechteck zwischen beiden in der
    /// SICHTBAREN Reihenfolge.
    cur_r: i32, cur_c: i32,
    ber_r: i32, ber_c: i32,
    /// Maus zieht gerade einen Bereich auf.
    zell_zug: bool,
    /// Enter unter der letzten Zeile und Einfuegen ueber das Ende hinaus
    /// haengen Zeilen an.
    zeilen_anhaengen: bool,
    /// Spaltenart fuer die Eingabe: 0 Text, 1 ganze Zahl, 2 Zahl, 3 Auswahl.
    col_typ: Vec<u8>,
    /// Eintraege je Auswahlspalte.
    col_wahl: Vec<Vec<String>>,
    /// Markierter Eintrag der offenen Auswahlliste (-1 = keine offen).
    wahl_pos: i32,
    /// Die aktuelle Zelle soll beim naechsten Durchgang ins Bild gerollt
    /// werden (gesetzt von Stellen, an denen die Geometrie nicht greifbar ist).
    sicht_holen: bool,

    // --- Baum mit Spalten -----------------------------------------------------
    /// Die Zeilen bilden einen Baum: je Datenzeile ihre Elternzeile (-1 =
    /// oben) und ob sie aufgeklappt ist. Die Daten bleiben eine flache Liste
    /// -- alle Nummern nach aussen sind weiter DATENzeilen, der Baum steckt
    /// nur in `view`, wie Sortieren und Filtern auch.
    baum: bool,
    eltern: Vec<i32>,
    offen: Vec<bool>,
    /// Je Datenzeile, bei jedem `rebuild_view` gerechnet: Tiefe und ob sie
    /// Kinder hat -- fuer Einzug, Dreieck und Treffertest.
    ebene: Vec<i32>,
    hat_kinder: Vec<bool>,
    /// So viele Elternangaben (von vorn) sind schon geprueft. Gueltig
    /// halten sie Anhaengen, Entfernen und GUI_TREETABLE_SET_PARENT selbst;
    /// neu geprueft wird nur, was dazukam -- jedes Mal alle zu pruefen machte
    /// das Fuellen einer Baumtabelle quadratisch.
    eltern_geprueft: usize,
}

impl Default for TableState {
    fn default() -> Self {
        TableState {
            headers: vec![], rows: vec![], col_widths: None, col_align: vec![],
            row_fg: vec![], row_bg: vec![],
            row_h: TBL_ROW_H, header_h: TBL_HEADER_H, filter_h: TBL_FILTER_H,
            zebra: false, grid: true,
            scroll_x: 0, scroll_y: 0, drag_v: false, drag_h: false, drag_off: 0,
            selected: -1, hover_row: -1, clicked_row: -1, hover_col: -1, clicked_col: -1,
            doppel: false, leer_text: String::new(), tipp: String::new(), tipp_zeit: 0.0,
            frozen: 0, col_order: vec![], reorderable: false, head_drag: None,
            multi: false, sel_rows: vec![], sel_anchor: -1,
            edit_row: -1, edit_col: -1, edit_text: String::new(), edit_caret: 0,
            edit_anchor: 0, edit_scroll: 0, edit_maus_sperre: false,
            col_edit: vec![],
            drag_col: -1, drag_col_w: 0, drag_col_x: 0, resizable_cols: true,
            sort_col: -1, sort_desc: false, sortable: true,
            filters: vec![], filter_row: false, filter_focus: -1,
            view: vec![],
            zellmodus: false, cur_r: -1, cur_c: -1, ber_r: -1, ber_c: -1, zell_zug: false,
            zeilen_anhaengen: false, col_typ: vec![], col_wahl: vec![], wahl_pos: -1,
            sicht_holen: false,
            baum: false, eltern: vec![], offen: vec![], ebene: vec![], hat_kinder: vec![],
            eltern_geprueft: 0,
        }
    }
}

impl TableState {
    /// Spaltenzahl = die BREITESTE Angabe. Damit ist die Spaltenzahl wirklich
    /// frei: Zeilen duerfen kuerzer sein als der Kopf (fehlende Zellen sind
    /// leer) und laenger (der Kopf bekommt dann leere Titel). Vorher war jede
    /// Abweichung ein Fehler -- und wer die Zeilen VOR dem Kopf setzte, konnte
    /// die Pruefung ganz umgehen und brachte `draw_table` zum Absturz.
    /// Ueber diese eine Quelle kann kein Index mehr danebengreifen.
    fn n_cols(&self) -> usize {
        self.rows.iter().map(|r| r.len()).chain(std::iter::once(self.headers.len())).max().unwrap_or(0)
    }
    fn cell(&self, r: usize, c: usize) -> Option<&Cell> {
        self.rows.get(r).and_then(|row| row.get(c))
    }
    /// Zelle zum Schreiben, notfalls bis dorthin auffuellen.
    fn cell_mut(&mut self, r: usize, c: usize) -> Option<&mut Cell> {
        let row = self.rows.get_mut(r)?;
        while row.len() <= c { row.push(Cell::text(String::new())); }
        row.get_mut(c)
    }
    fn header(&self, c: usize) -> &str {
        self.headers.get(c).map(|s| s.as_str()).unwrap_or("")
    }
    /// Gesamthoehe des Kopfbereichs: Titelzeile plus (wenn sichtbar) die
    /// Filterzeile. An EINER Stelle, damit Treffertest und Zeichnen nicht
    /// auseinander laufen.
    fn head_total(&self) -> i32 {
        self.header_h + if self.filter_row { self.filter_h } else { 0 }
    }

    /// Passt eine Zeile auf alle gesetzten Filter? Teilstring, ohne Ruecksicht
    /// auf Gross-/Kleinschreibung -- ein Filter, der exakte Treffer verlangt,
    /// waere zum Suchen unbrauchbar.
    fn passt(&self, r: usize) -> bool {
        for (c, f) in self.filters.iter().enumerate() {
            if f.is_empty() { continue; }
            let txt = self.cell(r, c).map(|x| x.text.as_str()).unwrap_or("");
            if !txt.to_lowercase().contains(&f.to_lowercase()) { return false; }
        }
        true
    }

    /// `view` neu bauen: erst filtern, dann sortieren.
    ///
    /// Wird nach JEDER Aenderung an Daten, Filter oder Sortierung gerufen --
    /// lieber einmal zu viel als eine Tabelle, die veraltete Zeilen zeigt.
    /// Kein heisser Pfad: nur wenn sich wirklich etwas geaendert hat.
    ///
    /// Sortiert wird ZAHLENWEISE, wenn beide Zellen als Zahl lesbar sind,
    /// sonst textweise. Ohne das stuende "10" vor "9" -- in einer
    /// Punktespalte die haeufigste Enttaeuschung an einer Tabelle.
    fn rebuild_view(&mut self) {
        if self.baum { return self.rebuild_baum(); }
        let mut v: Vec<usize> = (0..self.rows.len()).filter(|&r| self.passt(r)).collect();
        if self.sort_col >= 0 {
            let c = self.sort_col as usize;
            let desc = self.sort_desc;
            let rows = &self.rows;                 // nur lesend -- `v` ist lokal
            let txt = |r: usize| -> &str {
                rows.get(r).and_then(|row| row.get(c)).map(|x| x.text.as_str()).unwrap_or("")
            };
            v.sort_by(|&a, &b| {
                let (sa, sb) = (txt(a), txt(b));
                let ord = match (sa.trim().parse::<f64>(), sb.trim().parse::<f64>()) {
                    (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
                    _ => sa.to_lowercase().cmp(&sb.to_lowercase()),
                };
                if desc { ord.reverse() } else { ord }
            });
        }
        self.view = v;
    }

    /// Vergleich zweier Datenzeilen nach der Sortierspalte -- dieselbe Regel
    /// wie in `rebuild_view` (Zahlen als Zahlen), hier fuer Geschwister im Baum.
    fn vergleiche(&self, a: usize, b: usize) -> std::cmp::Ordering {
        let c = self.sort_col.max(0) as usize;
        let txt = |r: usize| -> &str {
            self.rows.get(r).and_then(|row| row.get(c)).map(|x| x.text.as_str()).unwrap_or("")
        };
        let (sa, sb) = (txt(a), txt(b));
        let ord = match (sa.trim().parse::<f64>(), sb.trim().parse::<f64>()) {
            (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
            _ => sa.to_lowercase().cmp(&sb.to_lowercase()),
        };
        if self.sort_desc { ord.reverse() } else { ord }
    }

    /// Die zuletzt angehaengte Zeile in die Ansicht nehmen, ohne sie ganz
    /// neu zu bauen, wo das Ergebnis feststeht: flach (unsortiert ans Ende,
    /// sortiert per Binaersuche an ihre Stelle -- beides nur, wenn sie zum
    /// Filter passt), im Baum ohne Filter und Sortierung eine
    /// neue Wurzel (ans Ende) oder ein Kind unter einem ZUgeklappten Knoten
    /// (unsichtbar, nur das Dreieck erscheint). Sonst der volle Neubau. Jede
    /// Zeile neu zu bauen machte das Fuellen quadratisch: 120000 Zeilen in
    /// eine Baumtabelle dauerten 38 s, die Variablenliste des Debuggers hielt
    /// die IDE bei einem grossen Programm 2 s fest (Stresstest 2026-10-02).
    fn zeile_angehaengt(&mut self) {
        let r = self.rows.len() - 1;
        if self.sort_col >= 0 {
            if self.baum { return self.rebuild_view(); }
            // Sortiert: per Binaersuche an ihre Stelle -- hinter alle
            // gleichen, wie die stabile Sortierung im Neubau sie stellt.
            if !self.passt(r) { return; }
            let pos = self.view.partition_point(|&x| self.vergleiche(x, r) != std::cmp::Ordering::Greater);
            self.view.insert(pos, r);
            return;
        }
        if !self.baum {
            if self.passt(r) { self.view.push(r); }
            return;
        }
        if self.filters.iter().any(|f| !f.is_empty()) { return self.rebuild_baum(); }
        self.baum_sync();
        // Nur wenn die Baum-Angaben zum Stand vor der Zeile passen.
        if self.hat_kinder.len() != r || self.ebene.len() != r { return self.rebuild_baum(); }
        let e = self.eltern[r];
        if e < 0 {
            self.view.push(r);
        } else if !self.offen[e as usize] {
            self.hat_kinder[e as usize] = true;
        } else {
            return self.rebuild_baum();
        }
        self.ebene.push(0);
        self.hat_kinder.push(false);
    }

    /// Baumzustand so lang wie die Daten halten. Neue Zeilen haengen oben
    /// und sind zu; eine Elternangabe, die ins Leere zeigt, gilt als oben.
    fn baum_sync(&mut self) {
        let n = self.rows.len();
        // Kuerzer geworden: alles pruefen (geladen ist der Stand frisch, 0).
        if self.eltern.len() > n || self.eltern_geprueft > n { self.eltern_geprueft = 0; }
        self.eltern.resize(n, -1);
        self.offen.resize(n, false);
        for r in self.eltern_geprueft..n {
            let e = self.eltern[r];
            if e >= n as i32 || e == r as i32 { self.eltern[r] = -1; }
        }
        self.eltern_geprueft = n;
    }

    /// `view` als Baum: Tiefensuche von den Wurzeln, Kinder nur unter
    /// aufgeklappten Eltern. Mit Filter zaehlt eine Zeile, wenn sie selbst
    /// oder einer ihrer Nachkommen passt -- und dann ist der Weg dorthin
    /// offen, sonst saehe man den Treffer nicht. Sortiert werden Geschwister
    /// untereinander; ein Kind verliesse sonst seinen Elternteil.
    fn rebuild_baum(&mut self) {
        self.baum_sync();
        let n = self.rows.len();
        let mut kinder: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut wurzeln: Vec<usize> = Vec::new();
        for r in 0..n {
            match self.eltern[r] { e if e >= 0 => kinder[e as usize].push(r), _ => wurzeln.push(r) }
        }
        if self.sort_col >= 0 {
            wurzeln.sort_by(|&a, &b| self.vergleiche(a, b));
            for k in kinder.iter_mut() { let mut kk = std::mem::take(k); kk.sort_by(|&a, &b| self.vergleiche(a, b)); *k = kk; }
        }
        let filter = self.filters.iter().any(|f| !f.is_empty());
        // Treffer von unten nach oben: eine Zeile zaehlt, wenn sie passt oder
        // ein Nachkomme. Ein Kreis in den Elternangaben endet an `besucht`.
        let mut zaehlt = vec![false; n];
        if filter {
            fn tief(r: usize, kinder: &[Vec<usize>], t: &TableState, zaehlt: &mut [bool], besucht: &mut [bool]) -> bool {
                if besucht[r] { return zaehlt[r]; }
                besucht[r] = true;
                let mut ja = t.passt(r);
                for &k in &kinder[r] { if tief(k, kinder, t, zaehlt, besucht) { ja = true; } }
                zaehlt[r] = ja;
                ja
            }
            let mut besucht = vec![false; n];
            for r in 0..n { tief(r, &kinder, self, &mut zaehlt, &mut besucht); }
        }
        let mut ebene = vec![0i32; n];
        let mut v: Vec<usize> = Vec::new();
        let mut gesehen = vec![false; n];
        let mut stapel: Vec<(usize, i32)> = wurzeln.iter().rev().map(|&r| (r, 0)).collect();
        while let Some((r, e)) = stapel.pop() {
            if gesehen[r] { continue; }
            gesehen[r] = true;
            if filter && !zaehlt[r] { continue; }
            ebene[r] = e;
            v.push(r);
            if self.offen[r] || filter {
                for &k in kinder[r].iter().rev() { stapel.push((k, e + 1)); }
            }
        }
        self.hat_kinder = kinder.iter().map(|k| !k.is_empty()).collect();
        self.ebene = ebene;
        self.view = v;
    }

    /// Ansichtszeile -> Datenzeile. Alle Angaben nach aussen sind DATENzeilen;
    /// nur das Zeichnen und der Treffertest arbeiten mit der Ansicht.
    fn data_row(&self, view_i: i32) -> i32 {
        if view_i < 0 { return -1; }
        self.view.get(view_i as usize).map(|&r| r as i32).unwrap_or(-1)
    }
    /// Datenzeile -> Ansichtszeile (-1 wenn wegefiltert).
    fn view_of(&self, data_r: i32) -> i32 {
        if data_r < 0 { return -1; }
        self.view.iter().position(|&r| r as i32 == data_r).map(|i| i as i32).unwrap_or(-1)
    }

    /// Gueltige Reihenfolge: gespeicherte Wuensche, bereinigt und ergaenzt.
    ///
    /// Kommen Spalten dazu oder fallen weg, muss die Liste trotzdem JEDE Spalte
    /// genau einmal enthalten -- sonst faellt eine beim Zeichnen unter den
    /// Tisch oder taucht doppelt auf. Darum wird sie hier bei jedem Abruf
    /// gesaeubert statt an zwoelf Stellen nachgefuehrt.
    fn order(&self) -> Vec<usize> {
        let n = self.n_cols();
        let mut out: Vec<usize> = Vec::with_capacity(n);
        for &c in &self.col_order {
            if c < n && !out.contains(&c) { out.push(c); }
        }
        for c in 0..n {
            if !out.contains(&c) { out.push(c); }
        }
        out
    }
    /// Spalte an eine andere Anzeige-Position schieben.
    fn move_col(&mut self, von: usize, nach: usize) {
        let mut o = self.order();
        if von >= o.len() || nach >= o.len() { return; }
        let c = o.remove(von);
        o.insert(nach, c);
        self.col_order = o;
    }

    fn ist_gewaehlt(&self, r: i32) -> bool {
        // Ohne Mehrfachauswahl zaehlt nur `selected` -- sonst muesste jeder
        // Setter zwei Stellen pflegen und sie koennten auseinander laufen.
        if !self.multi { return self.selected == r; }
        self.sel_rows.contains(&r)
    }
    fn auswahl_setzen(&mut self, r: i32) {
        self.selected = r;
        self.sel_rows.clear();
        if r >= 0 { self.sel_rows.push(r); }
        self.sel_anchor = r;
    }
    fn auswahl_umschalten(&mut self, r: i32) {
        if let Some(i) = self.sel_rows.iter().position(|&x| x == r) {
            self.sel_rows.remove(i);
            // Die "zuletzt angeklickte" muss mitwandern, sonst zeigte
            // GUI_TABLE_SELECTED auf eine abgewaehlte Zeile.
            if self.selected == r { self.selected = *self.sel_rows.last().unwrap_or(&-1); }
        } else {
            self.sel_rows.push(r);
            self.selected = r;
        }
        self.sel_anchor = r;
    }
    /// Bereich vom Anker bis `r` -- in der SICHTBAREN Reihenfolge. Man waehlt
    /// mit Umschalt das aus, was dazwischen zu SEHEN ist; ueber die Datenzeilen
    /// zu gehen wuerde bei sortierter Tabelle etwas voellig anderes greifen.
    fn auswahl_bereich(&mut self, r: i32) {
        let a = if self.sel_anchor >= 0 { self.sel_anchor } else { r };
        let (ia, ib) = (self.view_of(a), self.view_of(r));
        if ia < 0 || ib < 0 { self.auswahl_setzen(r); return; }
        let (von, bis) = (ia.min(ib), ia.max(ib));
        self.sel_rows.clear();
        for i in von..=bis {
            if let Some(&d) = self.view.get(i as usize) { self.sel_rows.push(d as i32); }
        }
        self.selected = r;
    }

    fn col_editable(&self, c: usize) -> bool {
        *self.col_edit.get(c).unwrap_or(&false)
    }

    fn align_of(&self, r: usize, c: usize) -> i8 {
        let ca = *self.col_align.get(c).unwrap_or(&0);
        match self.cell(r, c) {
            Some(cell) if cell.align >= 0 => cell.align,
            _ => ca,
        }
    }

    // --- Gitter --------------------------------------------------------------

    fn col_typ_of(&self, c: usize) -> u8 {
        *self.col_typ.get(c).unwrap_or(&0)
    }
    /// Anzeige-Position der Datenspalte `c` (-1 = keine).
    fn pos_of(&self, c: i32) -> i32 {
        if c < 0 { return -1; }
        self.order().iter().position(|&x| x as i32 == c).map(|p| p as i32).unwrap_or(-1)
    }
    /// Passt `s` in die Spalte `c`? Leer passt immer -- eine Zelle zu leeren
    /// ist keine falsche Eingabe.
    fn wert_ok(&self, c: usize, s: &str) -> bool {
        let s = s.trim();
        if s.is_empty() { return true; }
        match self.col_typ_of(c) {
            1 => s.parse::<i64>().is_ok(),
            2 => s.replace(',', ".").parse::<f64>().is_ok(),
            3 => self.col_wahl.get(c).map(|w| w.iter().any(|x| x == s)).unwrap_or(false),
            4 => crate::kalender::parse(s).is_some(),
            _ => true,
        }
    }
    /// Der Zellbereich als (Ansichtszeile von, bis, Position von, bis). Ein
    /// Rechteck ist er nur in der SICHTBAREN Reihenfolge -- ueber Datenzeilen
    /// waere er bei sortierter Tabelle ein Flickenteppich.
    fn bereich(&self) -> Option<(usize, usize, usize, usize)> {
        let (vc, pc) = (self.view_of(self.cur_r), self.pos_of(self.cur_c));
        if vc < 0 || pc < 0 { return None; }
        let (mut va, mut pa) = (self.view_of(self.ber_r), self.pos_of(self.ber_c));
        if va < 0 || pa < 0 { va = vc; pa = pc; }
        Some((va.min(vc) as usize, va.max(vc) as usize, pa.min(pc) as usize, pa.max(pc) as usize))
    }
    /// Eine Zelle als Text fuer die Zwischenablage. Tabulator und Umbruch im
    /// Inhalt werden zu Leerzeichen -- sie sind dort die Trenner.
    fn zell_text(&self, r: usize, c: usize) -> String {
        let s = match self.cell(r, c) {
            Some(x) if x.kind == CellKind::Check => (if x.value >= 0.5 { "1" } else { "0" }).to_string(),
            Some(x) if x.kind == CellKind::Bar => zahl_kurz(x.value),
            Some(x) => x.text.clone(),
            None => String::new(),
        };
        s.replace(['\t', '\n', '\r'], " ")
    }
    /// Der Bereich als Text: Zellen durch Tabulator, Zeilen durch Umbruch --
    /// das Format, das jede Tabellenkalkulation beim Einfuegen versteht.
    fn bereich_text(&self) -> String {
        let (v1, v2, p1, p2) = match self.bereich() { Some(b) => b, None => return String::new() };
        let order = self.order();
        let mut zeilen = Vec::new();
        for vi in v1..=v2 {
            let r = match self.view.get(vi) { Some(&r) => r, None => continue };
            let z: Vec<String> = (p1..=p2).map(|p| self.zell_text(r, order[p])).collect();
            zeilen.push(z.join("\t"));
        }
        zeilen.join("\n")
    }
    /// Einen Text in eine Zelle schreiben, nach den Regeln der Spalte. Gesperrte
    /// Spalten und unpassende Werte werden UEBERGANGEN, nicht halb geschrieben;
    /// eine Hakenzelle nimmt 1/0/ja/nein. Liefert, ob geschrieben wurde.
    fn zelle_schreiben(&mut self, r: usize, c: usize, s: &str) -> bool {
        if !self.col_editable(c) || r >= self.rows.len() { return false; }
        let kind = self.cell(r, c).map(|x| x.kind).unwrap_or(CellKind::Text);
        match kind {
            CellKind::Check => {
                let v = match s.trim().to_lowercase().as_str() {
                    "1" | "ja" | "true" | "wahr" | "x" => 1.0,
                    "0" | "nein" | "false" | "falsch" | "" => 0.0,
                    _ => return false,
                };
                if let Some(x) = self.cell_mut(r, c) { x.value = v; }
                true
            }
            CellKind::Text => {
                let s = s.trim_end_matches('\r');
                if !self.wert_ok(c, s) { return false; }
                let neu = if self.col_typ_of(c) == 0 { s.to_string() } else { s.trim().to_string() };
                if let Some(x) = self.cell_mut(r, c) { x.text = neu; }
                true
            }
            _ => false,
        }
    }
    /// Neue leere Zeile anhaengen; liefert ihre Datenzeile.
    fn zeile_anhaengen(&mut self) -> usize {
        self.rows.push(Vec::new());
        self.row_fg.push(-1);
        self.row_bg.push(-1);
        self.rows.len() - 1
    }
    /// Tabulator-Text ab der aktuellen Zelle einfuegen. Liefert die Zahl der
    /// geschriebenen Zellen; der Bereich zeigt danach das Eingefuegte.
    fn einfuegen_text(&mut self, text: &str) -> i64 {
        let order = self.order();
        if order.is_empty() { return 0; }
        let vc = self.view_of(self.cur_r).max(0) as usize;
        let pc = self.pos_of(self.cur_c).max(0) as usize;
        let mut zeilen: Vec<&str> = text.split('\n').collect();
        if zeilen.last().map(|z| z.trim_end_matches('\r').is_empty()).unwrap_or(false) { zeilen.pop(); }
        let mut ziele: Vec<usize> = Vec::new();
        for i in 0..zeilen.len() {
            if let Some(&r) = self.view.get(vc + i) { ziele.push(r); }
            else if self.zeilen_anhaengen { ziele.push(self.zeile_anhaengen()); }
            else { break; }
        }
        let mut n = 0;
        let mut breite = 1;
        for (i, &r) in ziele.iter().enumerate() {
            let zellen: Vec<&str> = zeilen[i].trim_end_matches('\r').split('\t').collect();
            breite = breite.max(zellen.len());
            for (j, s) in zellen.iter().enumerate() {
                if let Some(&c) = order.get(pc + j) {
                    if self.zelle_schreiben(r, c, s) { n += 1; }
                }
            }
        }
        if let (Some(&erste), Some(&letzte)) = (ziele.first(), ziele.last()) {
            self.ber_r = erste as i32;
            self.ber_c = order[pc] as i32;
            self.cur_r = letzte as i32;
            self.cur_c = order[(pc + breite - 1).min(order.len() - 1)] as i32;
            self.selected = self.cur_r;
        }
        self.rebuild_view();
        self.sicht_holen = true;
        n
    }
    /// Alle bearbeitbaren Zellen des Bereichs leeren.
    fn bereich_leeren(&mut self) -> i64 {
        let (v1, v2, p1, p2) = match self.bereich() { Some(b) => b, None => return 0 };
        let order = self.order();
        let rows: Vec<usize> = (v1..=v2).filter_map(|v| self.view.get(v).copied()).collect();
        let mut n = 0;
        for r in rows {
            for p in p1..=p2 {
                if self.zelle_schreiben(r, order[p], "") { n += 1; }
            }
        }
        if n > 0 { self.rebuild_view(); }
        n
    }
}

// Baum-Zustand (Kind::Tree): flache Knotenliste, Knoten-id == Index (stabil bis
// GUI_TREE_CLEAR). `parent` = -1 fuer Wurzelknoten. Sichtbarkeit ergibt sich aus
// den `expanded`-Flags der Vorfahren (siehe tree_visible).
#[derive(Default)]
pub struct TreeState {
    nodes: Vec<TreeNode>,
    selected: i32,   // Knoten-id oder -1 (zuletzt angeklickt)
    hover: i32,      // gehoverte Knoten-id oder -1
    scroll: i32,     // vertikaler Pixel-Offset
    sel: Vec<bool>,  // Mehrfachauswahl je Knoten (nur mit `multi`)
    multi: bool,
    anker: i32,      // Knoten-id fuer Umschalt+Klick
    kaestchen: bool,      // Haken-Spalte (wie bei der Liste)
    checks: Vec<bool>,    // Haken je Knoten (nur mit `kaestchen`)
    datei: Option<Box<DateiBaum>>,   // nur beim Dateibaum (GUI_FILETREE)
    /// Wie weit jeder Knoten aufgeklappt ist (0..1), weich nachlaufend. Nur
    /// fuers ZEICHNEN -- Treffertest, Tastatur und Rollen rechnen mit
    /// `expanded`. Fehlt ein Eintrag, gilt der Zustand selbst.
    auf_t: Vec<f32>,
    /// Knoten lassen sich mit F2 umbenennen (GUI_TREE_SET "bearbeitbar") --
    /// nicht beim Dateibaum: dort ist der Name der Name einer Datei.
    bearbeitbar: bool,
    /// Der Knoten, der gerade umbenannt wird (Arbeitskopie wie bei der Liste).
    edit: Option<ListEdit>,
    /// Ereignisse dieses Bildes: umbenannter Knoten, Doppelklick/Enter.
    bearbeitet: Option<usize>,
    doppel: bool,
    /// Hinweis, wenn der Baum leer ist.
    leer_text: String,
    /// Tipp-Puffer fuer "tippen springt".
    tipp: String,
    tipp_zeit: f64,
}

impl TreeState {
    /// Die Auswahl-Liste auf die Knotenzahl bringen. Knoten kommen nur ueber
    /// GUI_TREE_ADD dazu, verschwinden nur mit GUI_TREE_CLEAR -- ein
    /// Nachziehen an jedem Abruf kostet also nichts und kann nie zu kurz sein.
    /// Beim DATEIbaum wird die Liste dagegen neu gebaut, sobald sich ein Ordner
    /// oeffnet; dort haengen Auswahl und Haken am WEG, nicht an der Nummer.
    fn sync(&mut self) {
        let n = self.nodes.len();
        self.sel.resize(n, false);
        self.checks.resize(n, false);
    }
}

#[derive(Default)]
struct TreeNode {
    label: String,
    parent: i32,
    level: i32,
    expanded: bool,
    has_children: bool,
    icon: i64,    // Textur-Handle, -1 = keins
    color: i64,   // Textfarbe, -1 = Thema
    /// Eingebautes Sinnbild (GUI_TREE_ICON mit Namen, GUI_FILETREE_ICON),
    /// leer = keins. Gezeichnet in der Farbe des Knotens.
    sym: String,
}

/// Der DATEIbaum (GUI_FILETREE): ein Baum, der seine Knoten selbst aus dem
/// Dateisystem holt. Gelesen wird nur, was zu sehen ist -- die Wurzel und jeder
/// AUFGEKLAPPTE Ordner; ein Projekt mit einem `target`-Ordner darin kostet
/// damit nichts, solange niemand hineinsieht.
///
/// **Knoten-Nummern sind hier fluechtig.** Die Liste wird neu gebaut, sobald
/// sich ein Ordner oeffnet oder die Platte neu gelesen wird -- Auswahl, Haken
/// und aufgeklappte Aeste haengen deshalb am WEG (relativ zur Wurzel, immer mit
/// `/`), nicht an der Nummer. Nach aussen spricht dieses Widget darum auch nur
/// ueber Wege.
#[derive(Default)]
struct DateiBaum {
    wurzel: String,
    muster: Vec<String>,      // Dateien: "*.dh" usw.; leer = alle
    ausnahmen: Vec<String>,   // uebergangene Namen (Muster), Ordner wie Dateien
    ordner_zuerst: bool,
    verborgene: bool,         // Namen, die mit "." anfangen
    nur_ordner: bool,
    offen: Vec<String>,       // aufgeklappte Ordner (Wege)
    gehakt: Vec<String>,      // angehakte Wege (mit `kaestchen`) -- ueberleben das Zuklappen
    pfade: Vec<String>,       // je Knoten sein Weg
    ordner: Vec<bool>,        // je Knoten: Ordner?
    aktiviert: String,        // Doppelklick auf eine Datei -- ein Bild lang
    bild_ordner: i64,         // Sinnbilder, -1 = keins
    bild_datei: i64,
    klick_klappt: bool,       // ein Klick auf einen Ordner klappt ihn auf/zu
    auffrischen: f64,         // Sekunden zwischen zwei Blicken auf die Platte, 0 = nie
    naechste: f64,            // Zeitpunkt des naechsten Blicks
    /// Textfarbe je Muster (GUI_FILETREE_COLOR), in der Reihenfolge, in der
    /// sie gesetzt wurden -- das erste passende gilt. "/" meint Ordner.
    farben: Vec<(String, i64)>,
    /// Sinnbild je Muster (GUI_FILETREE_ICON), dieselbe Regel wie die Farben.
    symbole: Vec<(String, String)>,
    /// Dateien eines Ordners nach ihrer ART ordnen (GUI_FILETREE_SET
    /// "nach_art"): in der Reihenfolge, in der die Muster der Sinnbilder
    /// (ersatzweise der Farben) gesetzt wurden, erst darin nach dem Namen.
    nach_art: bool,
}

impl DateiBaum {
    /// Weg -> Knoten-Nummer (-1, wenn er gerade nicht sichtbar ist).
    fn knoten(&self, pfad: &str) -> i32 {
        self.pfade.iter().position(|p| p == pfad).map(|i| i as i32).unwrap_or(-1)
    }
    fn uebergangen(&self, name: &str) -> bool {
        if !self.verborgene && name.starts_with('.') { return true; }
        self.ausnahmen.iter().any(|m| crate::builtins::passt_muster(name, m))
    }
    fn passt_datei(&self, name: &str) -> bool {
        self.muster.is_empty() || self.muster.iter().any(|m| crate::builtins::passt_muster(name, m))
    }
    /// Die Farbe eines Eintrags: "/" trifft Ordner, jedes andere Muster nur
    /// Dateien (ein Ordner namens "bilder.png" ist kein Bild). -1 = Thema.
    fn farbe_fuer(&self, name: &str, ist_ordner: bool) -> i64 {
        self.farben.iter()
            .find(|(m, _)| if m == "/" { ist_ordner } else { !ist_ordner && crate::builtins::passt_muster(name, m) })
            .map(|(_, f)| *f).unwrap_or(-1)
    }
    /// Rang einer Datei fuer "nach_art": die Stelle des ersten passenden
    /// Musters, was keins trifft, kommt zuletzt. Ordner haben keinen Rang.
    fn art_rang(&self, name: &str) -> usize {
        let muster: Vec<&String> = if self.symbole.is_empty() {
            self.farben.iter().map(|(m, _)| m).collect()
        } else {
            self.symbole.iter().map(|(m, _)| m).collect()
        };
        muster.iter().filter(|m| m.as_str() != "/")
            .position(|m| crate::builtins::passt_muster(name, m))
            .unwrap_or(usize::MAX)
    }
    /// Das Sinnbild eines Eintrags nach derselben Regel. Ein aufgeklappter
    /// Ordner mit dem Sinnbild "ordner" zeigt "ordner_offen" -- man sieht
    /// dem Baum an, was offen ist, auch ohne auf das Dreieck zu schauen.
    fn symbol_fuer(&self, name: &str, ist_ordner: bool, offen: bool) -> String {
        let s = self.symbole.iter()
            .find(|(m, _)| if m == "/" { ist_ordner } else { !ist_ordner && crate::builtins::passt_muster(name, m) })
            .map(|(_, s)| s.clone()).unwrap_or_default();
        if offen && s == "ordner" { "ordner_offen".to_string() } else { s }
    }
}

/// Die Knotenliste neu aus dem Dateisystem bauen. Auswahl kommt ueber den WEG
/// zurueck (was zugeklappt wurde, faellt aus der Auswahl -- wie in jedem
/// Dateimanager), die Haken stehen ohnehin in `gehakt` und ueberleben es.
fn dateibaum_neu(t: &mut TreeState) {
    let Some(mut d) = t.datei.take() else { return };
    let wahl = if t.selected >= 0 { d.pfade.get(t.selected as usize).cloned() } else { None };
    let sel_wege: Vec<String> = d.pfade.iter().enumerate()
        .filter(|(i, _)| t.sel.get(*i).copied().unwrap_or(false))
        .map(|(_, p)| p.clone()).collect();
    // Der Oeffnungsgrad haengt am WEG, nicht an der Nummer: ein eben
    // aufgeklappter Ordner behaelt seine 0 und klappt weich auf. (Zu geht
    // ein Dateibaum sofort -- seine Kinder sind dann gar nicht mehr gelesen.)
    let alt_t: HashMap<String, f32> = d.pfade.iter().zip(t.auf_t.iter())
        .map(|(p, &v)| (p.clone(), v)).collect();
    let (mut nodes, mut pfade, mut ordner) = (Vec::new(), Vec::new(), Vec::new());
    dateibaum_sammeln(&d, "", -1, 0, &mut nodes, &mut pfade, &mut ordner);
    t.auf_t = pfade.iter().zip(nodes.iter())
        .map(|(p, n): (&String, &TreeNode)| alt_t.get(p).copied().unwrap_or(if n.expanded { 1.0 } else { 0.0 }))
        .collect();
    d.pfade = pfade; d.ordner = ordner;
    t.nodes = nodes;
    t.sel = vec![false; t.nodes.len()];
    t.checks = d.pfade.iter().map(|p| d.gehakt.iter().any(|g| g == p)).collect();
    t.selected = wahl.as_deref().map(|w| d.knoten(w)).unwrap_or(-1);
    for w in sel_wege { let k = d.knoten(&w); if k >= 0 { t.sel[k as usize] = true; } }
    t.anker = t.selected;
    t.datei = Some(d);
}

/// Einen Ordner lesen und (rekursiv fuer aufgeklappte) Knoten anhaengen.
/// Ein Ordner, den das System nicht hergibt, liefert nichts -- eine Meldung
/// waere hier falsch: ein Baum zeichnet jedes Bild neu, und ein gesperrter
/// Ordner unterwegs wuerde ihn zum Abbruch bringen statt ihn stumm zu lassen.
fn dateibaum_sammeln(d: &DateiBaum, rel: &str, eltern: i32, ebene: i32,
                     nodes: &mut Vec<TreeNode>, pfade: &mut Vec<String>, ordner: &mut Vec<bool>) {
    let mut voll = std::path::PathBuf::from(&d.wurzel);
    if !rel.is_empty() { for t in rel.split('/') { voll.push(t); } }
    let Ok(eintraege) = std::fs::read_dir(&voll) else { return };
    let mut liste: Vec<(String, bool)> = Vec::new();
    for e in eintraege.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if d.uebergangen(&name) { continue; }
        let ist_ordner = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !ist_ordner && (d.nur_ordner || !d.passt_datei(&name)) { continue; }
        liste.push((name, ist_ordner));
    }
    liste.sort_by(|a, b| {
        if d.ordner_zuerst && a.1 != b.1 { return b.1.cmp(&a.1); }
        if d.nach_art && !a.1 && !b.1 {
            let (ra, rb) = (d.art_rang(&a.0), d.art_rang(&b.0));
            if ra != rb { return ra.cmp(&rb); }
        }
        a.0.to_lowercase().cmp(&b.0.to_lowercase()).then(a.0.cmp(&b.0))
    });
    for (name, ist_ordner) in liste {
        let weg = if rel.is_empty() { name.clone() } else { format!("{}/{}", rel, name) };
        let farbe = d.farbe_fuer(&name, ist_ordner);
        let auf = ist_ordner && d.offen.iter().any(|o| o == &weg);
        let sym = d.symbol_fuer(&name, ist_ordner, auf);
        if eltern >= 0 { nodes[eltern as usize].has_children = true; }
        nodes.push(TreeNode {
            label: name, parent: eltern, level: ebene, expanded: auf,
            // JEDER Ordner bekommt sein Dreieck, auch ein leerer: was darin
            // liegt, weiss man erst, wenn man hineinsieht -- und einmal
            // vorsorglich hineinzusehen ist genau das, was der Baum vermeidet.
            has_children: ist_ordner,
            icon: if ist_ordner { d.bild_ordner } else { d.bild_datei },
            color: farbe,
            sym,
        });
        pfade.push(weg.clone());
        ordner.push(ist_ordner);
        if auf {
            let k = (nodes.len() - 1) as i32;
            dateibaum_sammeln(d, &weg, k, ebene + 1, nodes, pfade, ordner);
        }
    }
}

/// Eine Pruefregel an einem Formularfeld (GUI_RULE). `a`/`b` sind Zahlen
/// (Bereich, Laenge), `text` ein Muster, `meldung` der Satz fuer den Nutzer
/// (leer = Vorgabe der Regel).
#[derive(Clone, Debug)]
pub struct Regel {
    art: RegelArt,
    a: f64,
    b: f64,
    text: String,
    meldung: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RegelArt { Pflicht, Zahl, Ganz, Bereich, Laenge, Email, Datum, Muster }

impl RegelArt {
    fn from_str(s: &str) -> Option<RegelArt> {
        Some(match s.to_ascii_lowercase().as_str() {
            "pflicht" | "required" => RegelArt::Pflicht,
            "zahl" | "number" => RegelArt::Zahl,
            "ganz" | "integer" => RegelArt::Ganz,
            "bereich" | "range" => RegelArt::Bereich,
            "laenge" | "länge" | "length" => RegelArt::Laenge,
            "email" | "e-mail" => RegelArt::Email,
            "datum" | "date" => RegelArt::Datum,
            "muster" | "pattern" | "regex" => RegelArt::Muster,
            _ => return None,
        })
    }
    fn as_str(self) -> &'static str {
        match self {
            RegelArt::Pflicht => "pflicht", RegelArt::Zahl => "zahl", RegelArt::Ganz => "ganz",
            RegelArt::Bereich => "bereich", RegelArt::Laenge => "laenge", RegelArt::Email => "email",
            RegelArt::Datum => "datum", RegelArt::Muster => "muster",
        }
    }
}

/// Zahl aus einem Formularfeld: Komma oder Punkt als Dezimaltrenner, Leerraum
/// aussen herum egal. None, wenn es keine Zahl ist.
fn zahl_lesen(text: &str) -> Option<f64> {
    let t = text.trim().replace(',', ".");
    if t.is_empty() { return None; }
    t.parse::<f64>().ok().filter(|x| x.is_finite())
}

/// Ein Betrag in Cent, wie ein Geldfeld ihn zeigt: 4800 -> "48,00",
/// -5 -> "-0,05".
fn cent_text(c: i64) -> String {
    let v = if c < 0 { "-" } else { "" };
    format!("{}{},{:02}", v, c.unsigned_abs() / 100, c.unsigned_abs() % 100)
}

/// Die Regel fuer Zahlenfelder mit fuehrender Null: "080331" ist eine
/// Postleitzahl oder Nummer und bleibt Text (als INTEGER verloere sie die
/// Null) -- "0,99" und "0" sind dagegen Zahlen. Bis 2026.30 galt die Regel
/// fuer alles, was mit 0 anfing und laenger war: ein Preis "0,99" ging als
/// Text in die Datenbank.
fn nummer_mit_null(text: &str) -> bool {
    let t = text.trim();
    let mut z = t.chars();
    z.next() == Some('0') && z.next().map_or(false, |c| c.is_ascii_digit())
}

fn zahl_kurz(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 { format!("{}", x as i64) } else { format!("{}", x) }
}

/// Eine Regel gegen einen Wert pruefen. `text` ist der Feldinhalt (bei
/// Klappliste/Kaestchen leer, dort zaehlt `gewaehlt`). Liefert die Meldung
/// oder None, wenn die Regel erfuellt ist. Reine Funktion -- mit Rust-Tests.
fn regel_pruefen(r: &Regel, text: &str, gewaehlt: bool) -> Option<String> {
    let t = text.trim();
    let melde = |vorgabe: String| Some(if r.meldung.is_empty() { vorgabe } else { r.meldung.clone() });
    match r.art {
        RegelArt::Pflicht => {
            if t.is_empty() && !gewaehlt { melde("Pflichtfeld".to_string()) } else { None }
        }
        // Alle weiteren Regeln lassen ein LEERES Feld durch -- ob es leer sein
        // darf, sagt allein `pflicht`. Sonst hiesse "Bereich 1..10" zugleich
        // "muss ausgefuellt sein", und das sieht man der Regel nicht an.
        _ if t.is_empty() => None,
        RegelArt::Zahl => {
            if zahl_lesen(t).is_none() { melde("Bitte eine Zahl eingeben".to_string()) } else { None }
        }
        RegelArt::Ganz => {
            match zahl_lesen(t) {
                Some(x) if x.fract() == 0.0 => None,
                _ => melde("Bitte eine ganze Zahl eingeben".to_string()),
            }
        }
        RegelArt::Bereich => {
            match zahl_lesen(t) {
                Some(x) if x >= r.a && x <= r.b => None,
                _ => melde(format!("Bitte einen Wert von {} bis {} eingeben", zahl_kurz(r.a), zahl_kurz(r.b))),
            }
        }
        RegelArt::Laenge => {
            let n = t.chars().count() as f64;
            if n >= r.a && n <= r.b { None }
            else { melde(format!("Bitte {} bis {} Zeichen eingeben", zahl_kurz(r.a), zahl_kurz(r.b))) }
        }
        RegelArt::Email => {
            let ok = match t.find('@') {
                Some(p) if p > 0 => {
                    let rest = &t[p + 1..];
                    rest.contains('.') && !rest.starts_with('.') && !rest.ends_with('.')
                        && !rest.contains('@') && !t.contains(char::is_whitespace)
                }
                _ => false,
            };
            if ok { None } else { melde("Bitte eine gueltige E-Mail-Adresse eingeben".to_string()) }
        }
        RegelArt::Datum => {
            if crate::kalender::parse(t).is_some() { None }
            else { melde("Bitte ein Datum als JJJJ-MM-TT eingeben".to_string()) }
        }
        RegelArt::Muster => {
            match regex::Regex::new(&format!("^(?:{})$", r.text)) {
                Ok(re) if re.is_match(t) => None,
                Ok(_) => melde("Die Eingabe hat nicht das erwartete Format".to_string()),
                Err(e) => Some(format!("GUI_RULE: ungueltiges Muster '{}': {}", r.text, e)),
            }
        }
    }
}

/// Ein Panel, das seine Kinder rollt: die Kinder behalten ihre Lage im
/// Fenster, `scroll` ist nur ein Blick-Versatz (abs_rect zieht ihn ab).
/// Dadurch muss beim Rollen nichts verschoben werden, und ein Layout im
/// Panel rechnet weiter mit denselben Koordinaten.
#[derive(Default)]
struct PanelState {
    kinder: Vec<usize>,   // Widget-Indizes im selben Fenster
    scroll: i32,
    inhalt_h: i32,        // unterster Kindrand + Rand, je Bild gemessen
}

/// Reiter INNERHALB eines Fensters (Kind::TabControl). Die Beschriftungen
/// stehen in `Widget::items`, die aktive Seite in `Widget::sel`; hier liegt
/// nur, welches Kind auf welche Seite gehoert.
///
/// Die Kinder BEHALTEN ihre Lage im Fenster -- eine Seite blendet sie nur
/// ein oder aus (`widget_shown`), genau wie die Reiter des Fensters es mit
/// `tab_page` tun. Nichts wird verschoben, ein Layout auf einer Seite
/// rechnet also weiter mit denselben Koordinaten.
#[derive(Default)]
struct TabCtlState {
    kinder: Vec<(usize, i32)>,   // (Widget-Index, Seite)
    /// Reiter mit Kreuz (`GUI_TABCONTROL_SET "schliessbar"`). Wie bei den
    /// Reitern des Fensters schliesst die Laufzeit NICHTS -- sie meldet nur
    /// (`zu`, ein Bild lang), welcher Reiter zu soll.
    schliessbar: bool,
    zu: Option<usize>,
    /// Gemessene Breite jeder Beschriftung (Schirmpunkte, `leisten_pass`);
    /// leer oder 0 = noch nicht gemessen, dann gilt die Schaetzung.
    breiten: Vec<i32>,
}

/// Akkordeon (Kind::Accordion): Abschnitte untereinander, jeder mit einem
/// Kopf, der ihn auf- und zuklappt. Die Titel stehen in `Widget::items`,
/// welches Kind in welchem Abschnitt liegt in `Widget::tabctl` -- dieselbe
/// Buchfuehrung wie beim Reiterwerk.
///
/// Anders als dort WANDERN die Kinder: klappt ein Abschnitt oben auf, rutscht
/// alles darunter nach unten. Darum steht hier je Kind seine Lage IM
/// Abschnitt (`lage`), und `akk_pass` setzt die Fensterlage je Bild daraus.
#[derive(Default)]
struct AkkState {
    offen: Vec<bool>,
    /// (Widget-Index, x, y im Abschnitt) -- die Lage beim Hineinlegen.
    lage: Vec<(usize, i32, i32)>,
    /// Eigene Inhaltshoehe je Abschnitt, 0 = nach den Kindern.
    hoehen: Vec<i32>,
    /// Mehrere Abschnitte zugleich offen (sonst schliesst Oeffnen die anderen).
    mehrere: bool,
    scroll: i32,
    /// Je Bild gerechnet: (Kopf-y ohne Scroll, Inhaltshoehe) je Abschnitt.
    koepfe: Vec<(i32, i32)>,
    inhalt_h: i32,
    /// Kopf mit Tastaturfokus.
    fokus: i32,
    /// Ereignis dieses Bildes: der umgeschaltete Abschnitt (-1 = keiner).
    umgeschaltet: i32,
    /// Wie weit jeder Abschnitt offen ist (0..1), weich nachlaufend
    /// (`uebergaenge`). Fehlt ein Eintrag, gilt der Zustand selbst -- so
    /// steht ein beim Aufbau geoeffneter Abschnitt sofort offen da, statt
    /// beim ersten Bild aufzuklappen.
    auf_t: Vec<f32>,
}

/// Assistent (Kind::Wizard): Schritte nacheinander, oben die Schrittanzeige,
/// unten Zurueck/Weiter/Fertig und Abbrechen. Titel in `Widget::items`, der
/// aktuelle Schritt in `Widget::sel`, die Kinder je Schritt in
/// `Widget::tabctl` -- wie beim Reiterwerk behalten sie ihre Lage.
struct WzState {
    /// Vor Weiter/Fertig die Regeln (GUI_RULE) der Felder dieses Schritts
    /// pruefen -- ein falsches Feld haelt den Assistenten an.
    pruefen: bool,
    /// Weiter gesperrt (vom Programm, etwa bis eine Auswahl getroffen ist).
    weiter_aus: bool,
    /// Beschriftungen: Zurueck, Weiter, Fertig, Abbrechen.
    texte: [String; 4],
    /// Ereignis dieses Bildes: 1 Schritt gewechselt, 2 fertig, 3 abgebrochen.
    ereignis: u8,
}
impl Default for WzState {
    fn default() -> Self {
        WzState { pruefen: false, weiter_aus: false, ereignis: 0,
                  texte: ["Zurück".into(), "Weiter".into(), "Fertig".into(), "Abbrechen".into()] }
    }
}

/// Lage der Teile eines Assistenten, EINE Quelle fuer Zeichnen, Klick und
/// Bildschirmleser. Knoepfe als (x, y, b, h), absolut.
struct WzGeom {
    kopf_h: i32,
    fuss_y: i32,
    zurueck: (i32, i32, i32, i32),
    weiter: (i32, i32, i32, i32),
    abbrechen: (i32, i32, i32, i32),
}

/// Ein Eintrag der Werkzeugleiste (Kind::Toolbar).
#[derive(Clone)]
struct LeistenEintrag {
    art: u8,          // 0 Knopf, 1 Trenner, 2 Luecke (schiebt den Rest nach rechts)
    symbol: String,   // eingebautes Sinnbild, "" = keins
    bild: i64,        // Textur-Handle, -1 = keins
    tip: String,
    text: String,
    aus: bool,        // gesperrt
    kippbar: bool,
    an: bool,
    /// Gemessene Breite der Beschriftung (0 = noch nicht gemessen). Gemessen
    /// wird in GUI_UPDATE (`leisten_pass`), weil der Treffertest keine Grafik
    /// hat -- geschaetzt lief ein "Sichern" in den naechsten Knopf hinein,
    /// und das hat erst das gerenderte BILD gezeigt.
    text_b: i32,
}

/// Werkzeugleiste mit EIGENEN Eintraegen. Bis Stand 25 war `GUI_TOOLBAR` nur
/// ein Streifen, auf den das Programm Knoepfe legte und von Hand verschob --
/// mit Bildern, die fuer 16 Punkte gemalt waren und krumm auf 20 gestreckt
/// wurden, und einem Text-`|` als Trenner. Jetzt verteilt die Leiste ihre
/// Eintraege selbst, zeichnet eingebaute Sinnbilder in IHRER Groesse (als
/// Striche, nicht als gestrecktes Pixelbild) und meldet, welcher geklickt
/// wurde. Ohne Eintraege bleibt sie der Streifen, der sie war.
struct LeisteState {
    eintraege: Vec<LeistenEintrag>,
    hover: i32,       // Eintrag unter der Maus (je Bild neu)
    gedrueckt: i32,   // Eintrag, auf dem die Taste herunterging
    geklickt: i32,    // Eintrag, der in DIESEM Bild geklickt wurde (transient)
    mit_text: bool,   // Beschriftung neben dem Sinnbild
    symbol: i32,      // Sinnbildgroesse in Punkten, 0 = aus der Hoehe
    /// Ueberfahren je Eintrag, weich nachlaufend (siehe `uebergaenge`).
    hover_t: Vec<f32>,
}

/// Ein Feld der Statusleiste.
#[derive(Clone)]
struct StatusFeld {
    text: String,
    tip: String,
    breite: i32,      // logische Punkte, 0 = teilt sich den Rest
    ausr: u8,         // 0 links, 1 mitte, 2 rechts
    klickbar: bool,
}

/// Statusleiste mit Feldern (Kind::StatusBar). Bis Stand 27 schrieb ein
/// Programm alles in EINE Beschriftung -- die Meldung "Gesichert." und die
/// Stelle der Schreibmarke ueberschrieben sich gegenseitig, und die IDE
/// verlor jede Meldung beim naechsten Pfeil.
struct StatusState {
    felder: Vec<StatusFeld>,
    hover: i32,
    geklickt: i32,
}

impl Default for StatusState {
    fn default() -> Self { StatusState { felder: Vec::new(), hover: -1, geklickt: -1 } }
}

/// Pfadleiste (Brotkrumen, Kind::Breadcrumb): anklickbare Teile mit einem
/// unsichtbaren Wert je Teil. Passt nicht alles hinein, fallen die VORDEREN
/// Teile unter ein "..." -- das Ende eines Pfades ist das, wo man steht.
struct PfadState {
    texte: Vec<String>,
    daten: Vec<String>,
    breiten: Vec<i32>,  // gemessen in GUI_UPDATE (0 = noch nicht gemessen)
    hover: i32,         // Teil unter der Maus, -2 = das "...", -1 = nichts
    geklickt: i32,
}

impl Default for PfadState {
    fn default() -> Self { PfadState { texte: Vec::new(), daten: Vec::new(), breiten: Vec::new(), hover: -1, geklickt: -1 } }
}

impl Default for LeisteState {
    fn default() -> Self {
        LeisteState { eintraege: Vec::new(), hover: -1, gedrueckt: -1, geklickt: -1, mit_text: false, symbol: 0, hover_t: Vec::new() }
    }
}

/// Die eingebauten Sinnbilder. EINE Liste fuer Pruefung, Fehlermeldung und
/// Doku -- ein Name, den `sinnbild` nicht zeichnet, darf hier nicht stehen.
pub const SINNBILDER: &[&str] = &[
    "neu", "datei", "oeffnen", "sichern", "drucken", "export", "import",
    "rueckgaengig", "wiederholen", "ausschneiden", "kopieren", "einfuegen",
    "suchen", "start", "stopp", "pause", "debug", "haltepunkt", "pruefen",
    "profil", "handbuch", "einstellungen", "werkzeug", "umbruch", "info",
    "warnung", "plus", "minus", "schliessen", "links", "rechts", "hoch",
    "runter", "menue", "aktualisieren",
    // Malwerkzeuge und Editoren (2026-10-29)
    "stift", "radierer", "fuellen", "linie", "rechteck", "ellipse",
    "pipette", "auswahl", "lasso", "zauberstab", "verschieben", "spruehen",
    "wuerfel", "raster", "auge", "auge_zu", "spiegeln", "drehen",
    // Dateiarten fuer Baeume und Listen (2026-11-07)
    "ordner", "ordner_offen", "datei_code", "datei_test", "datei_bild",
    "datei_daten", "datei_text", "datei_ton", "datei_form",
];

fn sinnbild_pruefen(name: &str, fn_: &str) -> Result<String, String> {
    let n = name.trim().to_lowercase();
    if SINNBILDER.contains(&n.as_str()) { return Ok(n); }
    Err(format!("{}: Sinnbild '{}' gibt es nicht -- bekannt sind: {}", fn_, name, SINNBILDER.join(", ")))
}

/// Ein eingebautes Sinnbild zeichnen, in einem Quadrat der Seite `s` bei
/// (x, y). Gerechnet wird auf einem 16er-Raster, gezogen wird mit Strichen
/// in der ECHTEN Groesse -- so bleibt ein Sinnbild bei 14 wie bei 28 Punkten
/// scharf, und alle haben dieselbe Strichstaerke. `c` ist die Farbe; Starten,
/// Stoppen und Haltepunkt tragen ihre eigene, solange `gesperrt` nicht gilt.
fn sinnbild(g: &mut Graphics, name: &str, x: i32, y: i32, s: i32, c: i64, gesperrt: bool) {
    let f = s as f64 / 16.0;
    let px = |v: f64| x + (v * f).round() as i32;
    let py = |v: f64| y + (v * f).round() as i32;
    let t = (s as f64 / 10.0).max(1.4);
    let gruen = if gesperrt { c } else { 0x4CC38A };
    let rot = if gesperrt { c } else { 0xE0605A };
    let zug = |g: &mut Graphics, pts: &[(f64, f64)], w: f64, c: i64| {
        for p in pts.windows(2) {
            g.line_thick(px(p[0].0), py(p[0].1), px(p[1].0), py(p[1].1), w, c);
        }
    };
    let kreis = |g: &mut Graphics, cx: f64, cy: f64, r: f64, w: f64, c: i64| {
        let ro = (r * f).round() as i32;
        let ri = (ro - w.round() as i32).max(0);
        g.ring(px(cx), py(cy), ri, ro, 0.0, 360.0, c, true);
    };
    let bogen = |g: &mut Graphics, cx: f64, cy: f64, r: f64, von: f64, bis: f64, w: f64, c: i64| {
        let ro = (r * f).round() as i32;
        let ri = (ro - w.round() as i32).max(0);
        g.ring(px(cx), py(cy), ri, ro, von, bis, c, true);
    };
    // Eine gefuellte Flaeche, halb durchsichtig in der Strichfarbe -- gibt
    // Ordner und Fenstern Koerper, ohne sie zu Kloetzen zu machen.
    let hauch = deckkraft(c, 0.28);
    // Ein Blatt mit Eselsohr, der Grund der Dateiarten.
    let blatt = |g: &mut Graphics| {
        zug(g, &[(3.5, 1.5), (9.5, 1.5), (12.5, 4.5), (12.5, 14.5), (3.5, 14.5), (3.5, 1.5)], t, c);
        zug(g, &[(9.5, 1.5), (9.5, 4.5), (12.5, 4.5)], t, c);
    };
    match name {
        "ordner" => {
            g.box_fill(px(1.5), py(4.5), px(14.5), py(13.5), hauch);
            zug(g, &[(1.5, 4.5), (1.5, 2.5), (6.0, 2.5), (7.5, 4.5)], t, c);
            zug(g, &[(1.5, 4.5), (14.5, 4.5), (14.5, 13.5), (1.5, 13.5), (1.5, 4.5)], t, c);
        }
        "ordner_offen" => {
            zug(g, &[(1.5, 13.5), (1.5, 2.5), (6.0, 2.5), (7.5, 4.5), (13.0, 4.5), (13.0, 7.0)], t, c);
            g.triangle(px(1.5), py(13.5), px(4.0), py(7.0), px(15.0), py(7.0), hauch);
            g.triangle(px(1.5), py(13.5), px(15.0), py(7.0), px(12.5), py(13.5), hauch);
            zug(g, &[(1.5, 13.5), (4.0, 7.0), (15.0, 7.0), (12.5, 13.5), (1.5, 13.5)], t, c);
        }
        "datei_code" => {
            blatt(g);
            zug(g, &[(6.5, 7.5), (5.0, 9.75), (6.5, 12.0)], t, c);
            zug(g, &[(9.5, 7.5), (11.0, 9.75), (9.5, 12.0)], t, c);
        }
        "datei_test" => {
            blatt(g);
            zug(g, &[(5.0, 9.5), (7.0, 11.5), (11.0, 7.0)], t * 1.15, c);
        }
        "datei_text" => {
            blatt(g);
            zug(g, &[(5.5, 7.0), (10.5, 7.0)], t, c);
            zug(g, &[(5.5, 9.5), (10.5, 9.5)], t, c);
            zug(g, &[(5.5, 12.0), (9.0, 12.0)], t, c);
        }
        "datei_bild" => {
            zug(g, &[(1.5, 2.5), (14.5, 2.5), (14.5, 13.5), (1.5, 13.5), (1.5, 2.5)], t, c);
            g.triangle(px(2.5), py(12.5), px(6.5), py(7.5), px(10.5), py(12.5), hauch);
            zug(g, &[(2.5, 12.5), (6.5, 7.5), (9.0, 10.5), (11.0, 8.5), (13.5, 12.5)], t, c);
            kreis(g, 11.0, 5.5, 1.6, 1.6 * f + 1.0, c);
        }
        "datei_daten" => {
            g.box_fill(px(1.5), py(2.5), px(14.5), py(6.0), hauch);
            zug(g, &[(1.5, 2.5), (14.5, 2.5), (14.5, 13.5), (1.5, 13.5), (1.5, 2.5)], t, c);
            zug(g, &[(1.5, 6.0), (14.5, 6.0)], t, c);
            zug(g, &[(1.5, 9.75), (14.5, 9.75)], t, c);
            zug(g, &[(6.0, 2.5), (6.0, 13.5)], t, c);
        }
        "datei_ton" => {
            kreis(g, 5.0, 12.0, 2.4, 2.4 * f + 1.0, c);
            zug(g, &[(7.2, 12.0), (7.2, 2.0)], t, c);
            zug(g, &[(7.2, 2.0), (12.0, 4.0), (12.0, 7.0)], t, c);
        }
        "datei_form" => {
            g.box_fill(px(1.5), py(2.5), px(14.5), py(5.5), hauch);
            zug(g, &[(1.5, 2.5), (14.5, 2.5), (14.5, 13.5), (1.5, 13.5), (1.5, 2.5)], t, c);
            zug(g, &[(1.5, 5.5), (14.5, 5.5)], t, c);
            zug(g, &[(4.0, 8.5), (12.0, 8.5)], t, c);
            zug(g, &[(8.0, 11.0), (12.0, 11.0)], t, c);
        }
        "neu" | "datei" => {
            zug(g, &[(3.5, 1.5), (9.5, 1.5), (12.5, 4.5), (12.5, 14.5), (3.5, 14.5), (3.5, 1.5)], t, c);
            zug(g, &[(9.5, 1.5), (9.5, 4.5), (12.5, 4.5)], t, c);
            if name == "neu" {
                zug(g, &[(8.0, 7.0), (8.0, 12.0)], t, c);
                zug(g, &[(5.5, 9.5), (10.5, 9.5)], t, c);
            } else {
                zug(g, &[(5.5, 8.0), (10.5, 8.0)], t, c);
                zug(g, &[(5.5, 11.0), (10.5, 11.0)], t, c);
            }
        }
        "oeffnen" => {
            zug(g, &[(1.5, 13.5), (1.5, 3.0), (6.0, 3.0), (7.5, 4.5), (13.0, 4.5), (13.0, 6.5)], t, c);
            zug(g, &[(1.5, 13.5), (4.0, 7.0), (15.0, 7.0), (12.5, 13.5), (1.5, 13.5)], t, c);
        }
        "sichern" => {
            zug(g, &[(2.0, 2.0), (11.5, 2.0), (14.0, 4.5), (14.0, 14.0), (2.0, 14.0), (2.0, 2.0)], t, c);
            zug(g, &[(5.0, 2.0), (5.0, 6.0), (10.5, 6.0), (10.5, 2.0)], t, c);
            zug(g, &[(5.0, 14.0), (5.0, 10.0), (11.0, 10.0), (11.0, 14.0)], t, c);
        }
        "drucken" => {
            zug(g, &[(4.5, 5.5), (4.5, 1.5), (11.5, 1.5), (11.5, 5.5)], t, c);
            zug(g, &[(4.5, 11.5), (1.5, 11.5), (1.5, 5.5), (14.5, 5.5), (14.5, 11.5), (11.5, 11.5)], t, c);
            zug(g, &[(4.5, 9.0), (4.5, 14.5), (11.5, 14.5), (11.5, 9.0), (4.5, 9.0)], t, c);
        }
        "export" | "import" => {
            zug(g, &[(2.0, 9.5), (2.0, 14.0), (14.0, 14.0), (14.0, 9.5)], t, c);
            zug(g, &[(8.0, 2.0), (8.0, 10.5)], t, c);
            if name == "export" {
                zug(g, &[(4.5, 5.5), (8.0, 2.0), (11.5, 5.5)], t, c);
            } else {
                zug(g, &[(4.5, 7.0), (8.0, 10.5), (11.5, 7.0)], t, c);
            }
        }
        "rueckgaengig" | "wiederholen" => {
            let links = name == "rueckgaengig";
            // Bogen ueber oben; der Pfeil sitzt am freien Ende.
            bogen(g, 8.0, 9.5, 5.5, 180.0, 405.0, t, c);
            if links {
                g.triangle(px(0.5), py(8.0), px(6.0), py(8.0), px(3.0), py(12.5), c);
            } else {
                g.triangle(px(10.0), py(8.0), px(15.5), py(8.0), px(13.0), py(12.5), c);
            }
            let _ = links;
        }
        "ausschneiden" => {
            kreis(g, 4.5, 12.0, 2.6, t, c);
            kreis(g, 11.5, 12.0, 2.6, t, c);
            zug(g, &[(5.8, 9.8), (11.5, 1.5)], t, c);
            zug(g, &[(10.2, 9.8), (4.5, 1.5)], t, c);
        }
        "kopieren" => {
            zug(g, &[(5.5, 5.5), (14.0, 5.5), (14.0, 14.5), (5.5, 14.5), (5.5, 5.5)], t, c);
            zug(g, &[(2.0, 10.5), (2.0, 1.5), (10.5, 1.5)], t, c);
        }
        "einfuegen" => {
            zug(g, &[(5.5, 3.0), (2.5, 3.0), (2.5, 14.5), (13.5, 14.5), (13.5, 3.0), (10.5, 3.0)], t, c);
            g.round_rect(px(5.5), py(1.0), px(10.5), py(5.0), (f * 1.0).round() as i32, c, true);
        }
        "suchen" => {
            kreis(g, 6.5, 6.5, 5.0, t, c);
            zug(g, &[(10.2, 10.2), (14.5, 14.5)], t * 1.4, c);
        }
        "start" => g.triangle(px(4.0), py(2.0), px(4.0), py(14.0), px(14.0), py(8.0), gruen),
        "stopp" => g.round_rect(px(3.0), py(3.0), px(13.0), py(13.0), (f * 1.5).round() as i32, rot, true),
        "pause" => {
            g.round_rect(px(3.5), py(2.5), px(6.5), py(13.5), 1, c, true);
            g.round_rect(px(9.5), py(2.5), px(12.5), py(13.5), 1, c, true);
        }
        "debug" => {
            g.circle(px(8.0), py(9.5), (4.2 * f).round() as i32, c);
            g.circle(px(8.0), py(4.2), (2.3 * f).round() as i32, c);
            zug(g, &[(1.5, 6.0), (4.5, 8.0)], t, c);
            zug(g, &[(14.5, 6.0), (11.5, 8.0)], t, c);
            zug(g, &[(1.0, 10.5), (4.0, 10.5)], t, c);
            zug(g, &[(15.0, 10.5), (12.0, 10.5)], t, c);
            zug(g, &[(1.5, 15.0), (4.5, 12.5)], t, c);
            zug(g, &[(14.5, 15.0), (11.5, 12.5)], t, c);
        }
        "haltepunkt" => g.circle(px(8.0), py(8.0), (5.5 * f).round() as i32, rot),
        "pruefen" => zug(g, &[(2.5, 8.5), (6.5, 12.5), (13.5, 3.5)], t * 1.2, gruen),
        "profil" => {
            g.box_fill(px(2.0), py(9.0), px(4.5), py(14.0), c);
            g.box_fill(px(6.75), py(5.0), px(9.25), py(14.0), c);
            g.box_fill(px(11.5), py(2.0), px(14.0), py(14.0), c);
        }
        "handbuch" => {
            zug(g, &[(8.0, 4.5), (8.0, 14.0)], t, c);
            zug(g, &[(8.0, 4.5), (4.5, 3.0), (1.5, 3.0), (1.5, 12.5), (4.5, 12.5), (8.0, 14.0)], t, c);
            zug(g, &[(8.0, 4.5), (11.5, 3.0), (14.5, 3.0), (14.5, 12.5), (11.5, 12.5), (8.0, 14.0)], t, c);
        }
        "einstellungen" => {
            for k in 0..8 {
                let a = k as f64 * std::f64::consts::FRAC_PI_4;
                let (ca, sa) = (a.cos(), a.sin());
                zug(g, &[(8.0 + ca * 4.0, 8.0 + sa * 4.0), (8.0 + ca * 7.0, 8.0 + sa * 7.0)], t * 1.3, c);
            }
            kreis(g, 8.0, 8.0, 5.0, (2.6 * f).max(t), c);
        }
        "werkzeug" => {
            zug(g, &[(3.0, 13.0), (9.0, 7.0)], t * 1.6, c);
            bogen(g, 10.5, 5.5, 4.0, 90.0, 360.0, t, c);
        }
        "umbruch" => {
            zug(g, &[(1.5, 3.0), (14.5, 3.0)], t, c);
            zug(g, &[(1.5, 8.0), (12.0, 8.0)], t, c);
            bogen(g, 12.0, 10.5, 2.5, 270.0, 450.0, t, c);
            zug(g, &[(12.0, 13.0), (7.5, 13.0)], t, c);
            g.triangle(px(5.0), py(13.0), px(8.0), py(10.5), px(8.0), py(15.5), c);
            zug(g, &[(1.5, 13.0), (3.5, 13.0)], t, c);
        }
        "info" => {
            kreis(g, 8.0, 8.0, 7.0, t, c);
            zug(g, &[(8.0, 7.0), (8.0, 12.0)], t * 1.2, c);
            g.circle(px(8.0), py(4.3), (1.1 * f).round().max(1.0) as i32, c);
        }
        "warnung" => {
            zug(g, &[(8.0, 1.5), (15.0, 14.5), (1.0, 14.5), (8.0, 1.5)], t, c);
            zug(g, &[(8.0, 6.0), (8.0, 10.0)], t * 1.2, c);
            g.circle(px(8.0), py(12.3), (1.0 * f).round().max(1.0) as i32, c);
        }
        "plus" => { zug(g, &[(8.0, 2.5), (8.0, 13.5)], t * 1.2, c); zug(g, &[(2.5, 8.0), (13.5, 8.0)], t * 1.2, c); }
        "minus" => zug(g, &[(2.5, 8.0), (13.5, 8.0)], t * 1.2, c),
        "schliessen" => { zug(g, &[(3.0, 3.0), (13.0, 13.0)], t * 1.2, c); zug(g, &[(13.0, 3.0), (3.0, 13.0)], t * 1.2, c); }
        "links" => zug(g, &[(10.5, 2.5), (5.0, 8.0), (10.5, 13.5)], t * 1.2, c),
        "rechts" => zug(g, &[(5.5, 2.5), (11.0, 8.0), (5.5, 13.5)], t * 1.2, c),
        "hoch" => zug(g, &[(2.5, 10.5), (8.0, 5.0), (13.5, 10.5)], t * 1.2, c),
        "runter" => zug(g, &[(2.5, 5.5), (8.0, 11.0), (13.5, 5.5)], t * 1.2, c),
        "menue" => {
            zug(g, &[(2.0, 3.5), (14.0, 3.5)], t * 1.2, c);
            zug(g, &[(2.0, 8.0), (14.0, 8.0)], t * 1.2, c);
            zug(g, &[(2.0, 12.5), (14.0, 12.5)], t * 1.2, c);
        }
        "aktualisieren" => {
            bogen(g, 8.0, 8.0, 6.0, 0.0, 290.0, t, c);
            g.triangle(px(10.0), py(0.5), px(15.5), py(3.0), px(11.0), py(7.0), c);
        }
        // Ein Bleistift schraeg von unten links nach oben rechts.
        "stift" => {
            zug(g, &[(2.0, 14.0), (2.5, 10.5), (11.5, 1.5), (14.5, 4.5), (5.5, 13.5), (2.0, 14.0)], t, c);
            zug(g, &[(9.5, 3.5), (12.5, 6.5)], t, c);
            zug(g, &[(2.5, 10.5), (5.5, 13.5)], t, c);
        }
        // Ein Radiergummi: schraeger Quader, die Spitze gefuellt.
        "radierer" => {
            zug(g, &[(1.5, 10.5), (9.0, 3.0), (14.5, 8.5), (8.0, 15.0), (4.5, 15.0), (1.5, 12.0), (1.5, 10.5)], t, c);
            g.triangle(px(1.5), py(11.0), px(5.5), py(7.0), px(9.5), py(11.0), c);
            g.triangle(px(1.5), py(11.0), px(9.5), py(11.0), px(5.5), py(15.0), c);
            zug(g, &[(6.0, 15.0), (15.0, 15.0)], t, c);
        }
        // Ein gekippter Eimer, aus dem Farbe laeuft.
        "fuellen" => {
            zug(g, &[(6.5, 1.5), (13.0, 8.0), (8.0, 13.0), (1.5, 6.5), (6.5, 1.5)], t, c);
            zug(g, &[(1.5, 6.5), (13.0, 8.0)], t, c);
            g.circle(px(14.0), py(12.5), (1.8 * f).round().max(1.0) as i32, c);
            zug(g, &[(13.0, 8.0), (14.0, 11.0)], t, c);
        }
        "linie" => {
            zug(g, &[(2.5, 13.5), (13.5, 2.5)], t * 1.2, c);
            g.circle(px(2.5), py(13.5), (1.7 * f).round().max(1.0) as i32, c);
            g.circle(px(13.5), py(2.5), (1.7 * f).round().max(1.0) as i32, c);
        }
        "rechteck" => zug(g, &[(2.0, 3.5), (14.0, 3.5), (14.0, 12.5), (2.0, 12.5), (2.0, 3.5)], t, c),
        "ellipse" => {
            let mut p = Vec::with_capacity(33);
            for k in 0..=32 {
                let a = k as f64 / 32.0 * std::f64::consts::TAU;
                p.push((8.0 + 6.5 * a.cos(), 8.0 + 5.0 * a.sin()));
            }
            zug(g, &p, t, c);
        }
        // Eine Pipette: Gummikappe oben rechts, Roehre nach unten links.
        "pipette" => {
            g.circle(px(12.0), py(4.0), (2.6 * f).round() as i32, c);
            zug(g, &[(8.5, 4.5), (11.5, 7.5)], t * 1.3, c);
            zug(g, &[(10.0, 6.0), (3.0, 13.0), (2.0, 14.5), (3.5, 13.5)], t, c);
        }
        // Auswahl: gestricheltes Rechteck.
        "auswahl" => {
            for (a, b) in [((2.0, 2.5), (5.0, 2.5)), ((7.0, 2.5), (9.0, 2.5)), ((11.0, 2.5), (14.0, 2.5)),
                           ((2.0, 13.5), (5.0, 13.5)), ((7.0, 13.5), (9.0, 13.5)), ((11.0, 13.5), (14.0, 13.5)),
                           ((2.0, 2.5), (2.0, 5.5)), ((2.0, 7.0), (2.0, 9.0)), ((2.0, 10.5), (2.0, 13.5)),
                           ((14.0, 2.5), (14.0, 5.5)), ((14.0, 7.0), (14.0, 9.0)), ((14.0, 10.5), (14.0, 13.5))] {
                zug(g, &[a, b], t, c);
            }
        }
        // Lasso: eine Schlinge mit Schwanz.
        "lasso" => {
            let mut p = Vec::with_capacity(28);
            for k in 0..=26 {
                let a = (k as f64 / 26.0) * 5.4 + 1.6;
                p.push((8.5 + 6.0 * a.cos(), 6.0 + 4.0 * a.sin()));
            }
            zug(g, &p, t, c);
            zug(g, &[(4.6, 9.0), (4.0, 12.0), (6.0, 15.0)], t, c);
        }
        // Zauberstab mit Funkeln an der Spitze.
        "zauberstab" => {
            zug(g, &[(2.0, 14.0), (10.0, 6.0)], t * 1.4, c);
            zug(g, &[(12.5, 1.0), (12.5, 6.0)], t, c);
            zug(g, &[(10.0, 3.5), (15.0, 3.5)], t, c);
            zug(g, &[(14.5, 7.0), (15.5, 8.0)], t, c);
            zug(g, &[(8.5, 1.0), (9.5, 2.0)], t, c);
        }
        // Verschieben: Kreuz mit vier Pfeilspitzen.
        "verschieben" => {
            zug(g, &[(8.0, 1.5), (8.0, 14.5)], t, c);
            zug(g, &[(1.5, 8.0), (14.5, 8.0)], t, c);
            g.triangle(px(8.0), py(0.5), px(5.5), py(3.5), px(10.5), py(3.5), c);
            g.triangle(px(8.0), py(15.5), px(10.5), py(12.5), px(5.5), py(12.5), c);
            g.triangle(px(0.5), py(8.0), px(3.5), py(10.5), px(3.5), py(5.5), c);
            g.triangle(px(15.5), py(8.0), px(12.5), py(5.5), px(12.5), py(10.5), c);
        }
        // Spruehdose: Dose links, Spruehpunkte rechts oben.
        "spruehen" => {
            zug(g, &[(2.0, 6.5), (7.5, 6.5), (7.5, 15.0), (2.0, 15.0), (2.0, 6.5)], t, c);
            zug(g, &[(3.5, 6.5), (3.5, 4.0), (6.0, 4.0), (6.0, 6.5)], t, c);
            for (x, y) in [(10.0, 2.0), (13.0, 3.0), (11.0, 5.0), (14.5, 6.0), (12.0, 8.0), (15.0, 9.5), (10.0, 9.0)] {
                g.circle(px(x), py(y), (0.9 * f).round().max(1.0) as i32, c);
            }
        }
        // Ein Wuerfel von oben mit fuenf Augen.
        "wuerfel" => {
            g.round_rect(px(1.5), py(1.5), px(14.5), py(14.5), (f * 2.5).round() as i32, c, false);
            for (x, y) in [(5.0, 5.0), (11.0, 5.0), (8.0, 8.0), (5.0, 11.0), (11.0, 11.0)] {
                g.circle(px(x), py(y), (1.3 * f).round().max(1.0) as i32, c);
            }
        }
        "raster" => {
            zug(g, &[(1.5, 1.5), (14.5, 1.5), (14.5, 14.5), (1.5, 14.5), (1.5, 1.5)], t, c);
            zug(g, &[(5.8, 1.5), (5.8, 14.5)], t, c);
            zug(g, &[(10.2, 1.5), (10.2, 14.5)], t, c);
            zug(g, &[(1.5, 5.8), (14.5, 5.8)], t, c);
            zug(g, &[(1.5, 10.2), (14.5, 10.2)], t, c);
        }
        "auge" | "auge_zu" => {
            let mut p = Vec::with_capacity(34);
            for k in 0..=16 {
                let a = k as f64 / 16.0 * std::f64::consts::PI;
                p.push((8.0 - 7.0 * a.cos(), 8.0 - 4.5 * a.sin()));
            }
            for k in 0..=16 {
                let a = k as f64 / 16.0 * std::f64::consts::PI;
                p.push((8.0 + 7.0 * a.cos(), 8.0 + 4.5 * a.sin()));
            }
            zug(g, &p, t, c);
            g.circle(px(8.0), py(8.0), (2.2 * f).round().max(1.0) as i32, c);
            if name == "auge_zu" {
                zug(g, &[(2.0, 14.0), (14.0, 2.0)], t * 1.2, c);
            }
        }
        // Spiegeln: zwei Dreiecke an einer gestrichelten Achse.
        "spiegeln" => {
            zug(g, &[(1.5, 13.0), (6.5, 3.0), (6.5, 13.0), (1.5, 13.0)], t, c);
            g.triangle(px(9.5), py(3.0), px(9.5), py(13.0), px(14.5), py(13.0), c);
            for y in [1.0, 5.0, 9.0, 13.0] {
                zug(g, &[(8.0, y), (8.0, y + 2.0)], t, c);
            }
        }
        // Drehen: Dreiviertelkreis mit Pfeil im Uhrzeigersinn.
        "drehen" => {
            bogen(g, 8.0, 8.5, 6.0, 270.0, 540.0, t, c);
            g.triangle(px(7.0), py(0.0), px(7.0), py(5.5), px(11.5), py(2.5), c);
        }
        _ => g.circle(px(8.0), py(8.0), (3.0 * f).round() as i32, c),
    }
}

/// Gesetzter Text (Kind::RichText): ein Widget, das Markdown SETZT statt es
/// nur anzuzeigen -- Ueberschriften, Absaetze mit Umbruch, Aufzaehlungen,
/// Codebloecke, Tabellen mit echten Spalten, Zitate, Linien und Verweise.
///
/// **Gesetzt wird in GUI_UPDATE, nicht beim Zeichnen.** Nur dort kommen
/// Graphics (zum Messen) und der Schreibzugriff zusammen, und nur so kostet
/// ein Dokument mit zweitausend Zeilen seinen Satz EINMAL statt in jedem
/// Bild. Neu gesetzt wird, wenn sich Quelle, Breite, Schriftgroesse oder
/// Massstab geaendert haben -- `stand` haelt genau das fest.
#[derive(Default)]
struct RichState {
    quelle: String,
    zeilen: Vec<RtZeile>,
    inhalt_h: i32,
    stand: (usize, i32, i32, i32),   // (Quelllaenge, Breite, Basisgroesse, Massstab*100)
    scroll: i32,
    basis: i32,          // 0 = Groesse des Widgets/Themas
    code_font: i64,      // -1 = dieselbe Schrift wie der Fliesstext
    ziele: Vec<String>,  // Verweise, von den Laeufen ueber ihren Index benannt
    geklickt: String,    // in diesem Bild angeklickter Verweis (transient)
    // --- Stufe 29: Auswahl und Suche ---------------------------------------
    // Eine Stelle ist (Zeile, Zeichen im ZEILENTEXT) -- der Zeilentext ist
    // die Laeufe einer Zeile, mit Leerzeichen, wo zwischen ihnen eine Luecke
    // steht (`rt_zeilentext`). Damit findet die Suche auch zwei Woerter, und
    // Kopieren liefert lesbaren Text statt aneinandergeklebter Woerter.
    anker: (usize, usize),
    marke: (usize, usize),
    hat_auswahl: bool,
    ziehen: bool,         // Maustaste haengt, die Marke folgt ihr
    anker_neu: bool,      // der Anker ist beim naechsten Durchgang zu messen
    druck: (i32, i32),    // Druckpunkt relativ zum INHALT (mit Scroll)
    doppel: bool,         // der Druck war ein Doppelklick -> ein Wort waehlen
    nadel: String,        // GUI_RICHTEXT_MARK_ALL, klein geschrieben; leer = aus
    // --- Ueberschriften: (Stufe, Titel, y im Satz) -- fuer ein
    // Inhaltsverzeichnis und den Sprung dorthin.
    ueberschriften: Vec<(u8, String, i32)>,
    satz_aktuell: bool,        // der Satz gehoert zur jetzigen Quelle
    ziel_ue: Option<usize>,    // Sprung, der auf den naechsten Satz wartet
    ziel_suche: Option<(String, i64)>, // GUI_RICHTEXT_FIND vor dem Satz
    // Farben der Codebloecke je Art (Kommentar, Text, Zahl, Schluessel,
    // Name, Operator); None = die Vorgabe passend zum Grund.
    code_farben: [Option<i64>; 6],
    // --- Knoepfe an Codebloecken (GUI_RICHTEXT_CODE_BUTTONS) ---------------
    code_knoepfe: Vec<String>,
    /// Je Block: (y des Knopfstreifens, Knopfhoehe, Code, [(x, Breite)] je Knopf).
    codebloecke: Vec<(i32, i32, String, Vec<(i32, i32)>)>,
    code_aktion: String,   // in diesem Bild gedrueckter Knopf (transient)
    code_text: String,     // der Code des Blocks dazu
}

/// Eine gesetzte Zeile. `grund` faerbt die ganze Zeile (Codeblock), `linie`
/// ist ein waagerechter Strich (`---`), `balken` der senkrechte Strich eines
/// Zitats.
#[derive(Default, Clone)]
struct RtZeile {
    y: i32,
    h: i32,
    /// Grund unter der Zeile: 0 keiner, 1 Codeblock, 2 Akzentband (unter
    /// der Hauptueberschrift), 3 Tabellenkopf, 4 Tabellenzeile (Zebra),
    /// 5 Zitat. `bx0..bx1` = Breite (bx1 = 0: die ganze Zeile).
    band: u8,
    bx0: i32,
    bx1: i32,
    linie: bool,
    balken: i32,      // x des Zitatstrichs, -1 = keiner
    laeufe: Vec<RtLauf>,
}

/// Ein Stueck Text mit eigener Gestalt. `rolle`: 0 Text, 1 gedaempft,
/// 2 Akzent (Verweis, Aufzaehlungspunkt), 3 Code im Text, 4..6 Ueberschrift
/// der Stufe 1..3, 8 Codeblock (Farbe nach `art`). `link` = Index in
/// `ziele`, -1 = keiner. `art` = Abschnittsart im Codeblock (1 Kommentar,
/// 2 Text, 3 Zahl, 4 Schluessel, 5 Name, 6 Operator), 0 = Grundfarbe.
#[derive(Default, Clone)]
struct RtLauf {
    x: i32,
    breite: i32,
    text: String,
    groesse: i32,
    rolle: u8,
    /// Bits aus `schnitt`.
    stil: u8,
    code: bool,
    link: i32,
    art: u8,
    /// Versatz nach unten (Code im Fliesstext ist kleiner und sitzt tiefer,
    /// damit die Grundlinie mit dem Text davor stimmt).
    dy: i32,
}

/// Code im Fliesstext etwas kleiner: eine dicktengleiche Schrift wirkt bei
/// gleicher Pixelgroesse deutlich groesser als die Textschrift daneben.
fn rt_code_gr(gr: i32, code: bool) -> (i32, i32) {
    if !code { return (gr, 0); }
    let k = (gr * 86 / 100).max(8);
    (k, (gr - k) * 7 / 10)
}

/// Ein Wort mit seiner Gestalt -- die Zwischenform zwischen Auszeichnung und
/// Satz. Umgebrochen wird an Wortgrenzen, also ist das Wort die Einheit.
#[derive(Clone)]
struct RtWort {
    text: String,
    /// Bits aus `schnitt`.
    stil: u8,
    code: bool,
    rolle: u8,
    link: i32,
    /// Kein Leerzeichen davor -- `**fett**bar` ist EIN Wort in zwei
    /// Gestalten, und eine Luecke dazwischen saehe aus wie ein Tippfehler.
    kleben: bool,
}

/// Eine Zeile Markdown in Woerter mit Gestalt zerlegen: `**fett**`,
/// `*kursiv*`, `~~durchgestrichen~~`, `<u>unterstrichen</u>`,
/// `` `code` `` und `[Text](Ziel)`.
///
/// Fett und kursiv nehmen den echten Schnitt der Schrift, wenn es ihn als
/// Datei gibt (`Graphics::schnitt`), sonst zeichnet die Laufzeit Fett
/// doppelt und Kursiv geneigt -- sichtbar ist die Auszeichnung in jedem Fall.
/// Ueberschrift-Stufe einer (links beschnittenen) Zeile: 1..3, 0 = keine.
/// Zur ersten gesetzten Zeile unter `ab` rollen, die `nadel` (klein) enthaelt.
fn rt_finden(r: &mut RichState, nadel: &str, ab: i64, hoehe: i32) -> Option<i32> {
    let y = r.zeilen.iter().find(|z| {
        z.y > ab as i32 && z.laeufe.iter().any(|l| l.text.to_lowercase().contains(nadel))
    }).map(|z| z.y)?;
    let max = (r.inhalt_h - hoehe).max(0);
    r.scroll = (y - 20).clamp(0, max);
    Some(y)
}

fn rt_stufe(t: &str) -> (u8, &str) {
    if let Some(r) = t.strip_prefix("### ") { (3, r) }
    else if let Some(r) = t.strip_prefix("## ") { (2, r) }
    else if let Some(r) = t.strip_prefix("# ") { (1, r) }
    else { (0, t) }
}

/// Der Titel einer Ueberschrift ohne Auszeichnung (`**`, `` ` ``, Verweise).
fn rt_titel(rest: &str) -> String {
    let mut ziele = Vec::new();
    let mut s = String::new();
    for w in rt_inline(rest, 0, &mut ziele) {
        if !s.is_empty() && !w.kleben { s.push(' '); }
        s.push_str(&w.text);
    }
    s
}

/// Die Ueberschriften einer Quelle, ohne zu setzen -- in derselben
/// Reihenfolge, in der `rt_setzen` sie findet (Codebloecke zaehlen nicht).
fn rt_ueberschriften_quelle(quelle: &str) -> Vec<(u8, String)> {
    let mut raus = Vec::new();
    let mut im_code = false;
    for z in quelle.lines() {
        let t = z.trim_end().trim_start();
        if t.starts_with("```") { im_code = !im_code; continue; }
        if im_code { continue; }
        let (stufe, rest) = rt_stufe(t);
        if stufe > 0 { raus.push((stufe, rt_titel(rest))); }
    }
    raus
}

fn rt_inline(z: &str, grund_rolle: u8, ziele: &mut Vec<String>) -> Vec<RtWort> {
    let zeichen: Vec<char> = z.chars().collect();
    let mut raus: Vec<RtWort> = Vec::new();
    let (mut fett, mut kursiv, mut code) = (false, false, false);
    let (mut unter, mut durch) = (false, false);
    let mut link = -1i32;
    let mut puffer = String::new();
    // Ob der zuletzt verarbeitete Text mit einer Luecke endete. Das
    // Leerzeichen VOR einer Auszeichnung steht am Ende des vorigen Puffers
    // und faellt beim Zerlegen weg -- ohne diesen Merker wuerde aus
    // `Ein **fetter** Anfang` ein `Einfetter Anfang`.
    let mut letzte_luecke = true;
    let mut i = 0usize;
    // Was sich angesammelt hat, als Woerter anhaengen.
    macro_rules! spuelen {
        ($raus:expr, $puffer:expr) => {
            if !$puffer.is_empty() {
                let rolle = if link >= 0 { 2 } else if code { 3 } else { grund_rolle };
                let stil = (if fett { crate::schnitt::FETT } else { 0 })
                    | (if kursiv { crate::schnitt::KURSIV } else { 0 })
                    | (if unter { crate::schnitt::UNTER } else { 0 })
                    | (if durch { crate::schnitt::DURCH } else { 0 });
                let anhaengen = !$puffer.starts_with(' ') && !$raus.is_empty() && !letzte_luecke;
                letzte_luecke = $puffer.ends_with(' ');
                let mut erstes = true;
                for w in $puffer.split(' ') {
                    if w.is_empty() { continue; }
                    $raus.push(RtWort { text: w.to_string(), stil, code, rolle, link,
                                        kleben: erstes && anhaengen });
                    erstes = false;
                }
                $puffer.clear();
            }
        };
    }
    while i < zeichen.len() {
        let rest: String = zeichen[i..].iter().collect();
        if rest.starts_with("**") && !code {
            spuelen!(raus, puffer); fett = !fett; i += 2; continue;
        }
        if rest.starts_with("~~") && !code {
            spuelen!(raus, puffer); durch = !durch; i += 2; continue;
        }
        if !code && (rest.starts_with("<u>") || rest.starts_with("<U>")) {
            spuelen!(raus, puffer); unter = true; i += 3; continue;
        }
        if !code && (rest.starts_with("</u>") || rest.starts_with("</U>")) {
            spuelen!(raus, puffer); unter = false; i += 4; continue;
        }
        if (zeichen[i] == '*' || zeichen[i] == '_') && !code {
            // Ein `_` MITTEN im Wort ist ein Namensteil (`gui_update`), keine
            // Auszeichnung -- sonst faerbte jeder Bezeichner den halben Absatz.
            let am_rand = i == 0 || zeichen[i - 1] == ' ' || zeichen[i - 1] == '(' || zeichen[i - 1] == '*';
            let schliesst = kursiv;
            if am_rand || schliesst {
                spuelen!(raus, puffer); kursiv = !kursiv; i += 1; continue;
            }
        }
        if zeichen[i] == '`' {
            spuelen!(raus, puffer); code = !code; i += 1; continue;
        }
        if zeichen[i] == '[' && !code {
            // [Text](Ziel) -- nur, wenn beides vollstaendig dasteht.
            if let Some(zu) = zeichen[i..].iter().position(|&c| c == ']') {
                if zeichen.get(i + zu + 1) == Some(&'(') {
                    if let Some(ende) = zeichen[i + zu + 2..].iter().position(|&c| c == ')') {
                        spuelen!(raus, puffer);
                        let text: String = zeichen[i + 1..i + zu].iter().collect();
                        let ziel: String = zeichen[i + zu + 2..i + zu + 2 + ende].iter().collect();
                        ziele.push(ziel);
                        link = ziele.len() as i32 - 1;
                        puffer.push_str(&text);
                        spuelen!(raus, puffer);
                        link = -1;
                        i += zu + 2 + ende + 1;
                        continue;
                    }
                }
            }
        }
        puffer.push(zeichen[i]);
        i += 1;
    }
    spuelen!(raus, puffer);
    raus
}

/// Ein laufender Zug: beginnt mit dem Druck auf ein ziehbares Widget, wird
/// nach 5 px Bewegung `aktiv` -- vorher ist es ein Klick.
struct DragState {
    quelle: (usize, usize),
    index: i32,           // Listenzeile / Baumknoten, -1 = das Widget selbst
    text: String,
    x0: i32, y0: i32,
    aktiv: bool,
}

/// Die Ablage dieses Bildes -- ein Ereignis, kein Zustand.
struct DropInfo {
    ziel: (usize, usize),
    quelle: (usize, usize),
    index_quelle: i32,
    index_ziel: i32,
    text: String,
}

fn bildmodus_str(m: u8) -> &'static str {
    match m { 1 => "einpassen", 2 => "fuellen", 3 => "mitte", 4 => "kacheln", _ => "strecken" }
}
fn bildmodus_parsen(s: &str) -> Option<u8> {
    match s.to_ascii_lowercase().as_str() {
        "strecken" | "stretch" => Some(0),
        "einpassen" | "fit" => Some(1),
        "fuellen" | "füllen" | "cover" | "fill" => Some(2),
        "mitte" | "center" | "zentrieren" => Some(3),
        "kacheln" | "tile" => Some(4),
        _ => None,
    }
}

/// Wie in einem Textfeld gemessen wird: dieselbe Groesse und Schrift, mit
/// der auch gezeichnet wird. Eine Quelle fuer beides.
#[derive(Clone, Copy)]
struct Mass { size: i32, font: i64 }

impl Mass {
    fn breite(&self, g: &Graphics, s: &str) -> i32 { g.text_width_in(s, self.size, self.font) }
}

/// Was eine Liste ueber ihre Eintraege hinaus weiss -- nur angelegt, wenn
/// ein Programm eines davon benutzt; eine schlichte Liste bleibt, was sie war.
/// Alle Felder je Eintrag laufen ueber `sync` mit `items` gleich lang.
#[derive(Default)]
struct ListState {
    icons: Vec<i64>,      // Textur-Handle je Eintrag, -1 = keins
    checks: Vec<bool>,    // Haken je Eintrag (nur mit `kaestchen`)
    colors: Vec<i64>,     // Textfarbe je Eintrag, -1 = Thema
    sel: Vec<bool>,       // Mehrfachauswahl je Eintrag
    // --- Stufe 27: was eine Liste komfortabel macht -------------------------
    daten: Vec<String>,   // unsichtbarer Wert je Eintrag (eine ID, ein Pfad)
    details: Vec<String>, // Zusatztext rechts, gedaempft (Kuerzel, Zeilennummer)
    tips: Vec<String>,    // Tooltip je Eintrag
    aus: Vec<bool>,       // gesperrt: sichtbar, aber nicht waehlbar
    kopf: Vec<bool>,      // Gruppenkopf: nicht waehlbar, gliedert die Liste
    /// Farbabschnitte je Eintrag (Zeichen-Anfang, Zeichen-Laenge, Farbe) --
    /// "function" in der Farbe der Schluesselwoerter, der Name in seiner.
    spans: Vec<Vec<(usize, usize, i64)>>,
    filter: String,       // Teiltext; leer = alles sichtbar
    leer_text: String,    // Hinweis, wenn nichts zu sehen ist
    hover: i32,           // Eintrag unter der Maus (je Bild neu, fuer den Tooltip)
    tipp: String,         // Tipp-Puffer fuer "tippen springt"
    tipp_zeit: f64,
    multi: bool,
    kaestchen: bool,
    anker: i32,           // Anker fuer Umschalt+Klick
    doppel: bool,         // Doppelklick in DIESEM Bild (wie `clicked`)
    /// Eintraege lassen sich mit F2 umbenennen (GUI_LISTBOX_SET "bearbeitbar").
    bearbeitbar: bool,
    /// Der Eintrag, der gerade umbenannt wird -- die Arbeitskopie; die Liste
    /// bekommt den Text erst beim Uebernehmen (ESC nimmt ihn so zurueck).
    edit: Option<ListEdit>,
    /// Ereignis dieses Bildes: umbenannter Eintrag (GUI_LISTBOX_EDITED).
    bearbeitet: Option<usize>,
}

/// Arbeitskopie beim Umbenennen eines Listeneintrags.
struct ListEdit {
    k: usize,
    text: Vec<char>,
    caret: i32,
    anker: i32,
}

impl ListState {
    fn sync(&mut self, n: usize) {
        self.icons.resize(n, -1);
        self.checks.resize(n, false);
        self.colors.resize(n, -1);
        self.sel.resize(n, false);
        self.daten.resize(n, String::new());
        self.details.resize(n, String::new());
        self.tips.resize(n, String::new());
        self.aus.resize(n, false);
        self.kopf.resize(n, false);
        self.spans.resize(n, Vec::new());
    }
    fn einfuegen(&mut self, pos: usize) {
        self.icons.insert(pos, -1);
        self.checks.insert(pos, false);
        self.colors.insert(pos, -1);
        self.sel.insert(pos, false);
        self.daten.insert(pos, String::new());
        self.details.insert(pos, String::new());
        self.tips.insert(pos, String::new());
        self.aus.insert(pos, false);
        self.kopf.insert(pos, false);
        self.spans.insert(pos, Vec::new());
    }
    fn entfernen(&mut self, pos: usize) {
        self.icons.remove(pos);
        self.checks.remove(pos);
        self.colors.remove(pos);
        self.sel.remove(pos);
        self.daten.remove(pos);
        self.details.remove(pos);
        self.tips.remove(pos);
        self.aus.remove(pos);
        self.kopf.remove(pos);
        self.spans.remove(pos);
    }
    /// Einen Eintrag von `a` nach `b` tragen -- alles, was an ihm haengt, reist mit.
    fn tragen(&mut self, a: usize, b: usize) {
        fn zug<T>(v: &mut Vec<T>, a: usize, b: usize) { let x = v.remove(a); v.insert(b, x); }
        zug(&mut self.icons, a, b); zug(&mut self.checks, a, b); zug(&mut self.colors, a, b);
        zug(&mut self.sel, a, b); zug(&mut self.daten, a, b); zug(&mut self.details, a, b);
        zug(&mut self.tips, a, b); zug(&mut self.aus, a, b); zug(&mut self.kopf, a, b);
        zug(&mut self.spans, a, b);
    }
    /// Nach einer Umordnung: jeder Vektor in der Reihenfolge `ord` (alte Plaetze).
    fn umordnen(&mut self, ord: &[usize]) {
        fn nach<T: Clone>(v: &mut Vec<T>, ord: &[usize]) { *v = ord.iter().map(|&k| v[k].clone()).collect(); }
        nach(&mut self.icons, ord); nach(&mut self.checks, ord); nach(&mut self.colors, ord);
        nach(&mut self.sel, ord); nach(&mut self.daten, ord); nach(&mut self.details, ord);
        nach(&mut self.tips, ord); nach(&mut self.aus, ord); nach(&mut self.kopf, ord);
        nach(&mut self.spans, ord);
    }
    fn waehlbar(&self, k: usize) -> bool {
        !self.aus.get(k).copied().unwrap_or(false) && !self.kopf.get(k).copied().unwrap_or(false)
    }
    fn ist_leer(&self) -> bool {
        !self.multi && !self.kaestchen && self.icons.iter().all(|&i| i < 0)
            && self.colors.iter().all(|&c| c < 0) && !self.checks.iter().any(|&c| c)
            && self.daten.iter().all(|s| s.is_empty()) && self.details.iter().all(|s| s.is_empty())
            && self.tips.iter().all(|s| s.is_empty()) && !self.aus.iter().any(|&x| x)
            && !self.kopf.iter().any(|&x| x) && self.leer_text.is_empty()
            && self.spans.iter().all(|s| s.is_empty()) && !self.bearbeitbar
    }
}

/// Die SICHTBAREN Eintraege einer Liste, in Anzeigereihenfolge (Datenzeilen).
/// EINE Quelle fuer Zeichnen, Klick, Tastatur, Rollen und Bildschirmleser --
/// wie `view` bei der Tabelle. Ein Gruppenkopf bleibt stehen, solange in seiner
/// Gruppe etwas zum Filter passt; sonst stuende er ueber einer leeren Gruppe.
fn liste_ansicht(items: &[String], l: Option<&ListState>) -> Vec<usize> {
    let filter = match l { Some(l) if !l.filter.is_empty() => l.filter.to_lowercase(), _ => return (0..items.len()).collect() };
    let l = l.unwrap();
    let passt = |k: usize| items[k].to_lowercase().contains(&filter)
        || l.details.get(k).map(|d| d.to_lowercase().contains(&filter)).unwrap_or(false);
    let mut out = Vec::new();
    let mut kopf: Option<usize> = None;
    for k in 0..items.len() {
        if l.kopf.get(k).copied().unwrap_or(false) { kopf = Some(k); continue; }
        if passt(k) {
            if let Some(kp) = kopf.take() { out.push(kp); }
            out.push(k);
        }
    }
    out
}

/// Zwei Eintraege vergleichen wie ein Mensch: Zahlen nach Wert ("Datei 9" vor
/// "Datei 10"), sonst ohne Ruecksicht auf Gross/klein.
fn natuerlich_vergleichen(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (a, b) = (a.to_lowercase(), b.to_lowercase());
    let (mut ia, mut ib) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (ia.peek().copied(), ib.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let mut na = String::new(); while let Some(c) = ia.peek().copied().filter(|c| c.is_ascii_digit()) { na.push(c); ia.next(); }
                let mut nb = String::new(); while let Some(c) = ib.peek().copied().filter(|c| c.is_ascii_digit()) { nb.push(c); ib.next(); }
                let (za, zb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
                let o = za.len().cmp(&zb.len()).then_with(|| za.cmp(zb));
                if o != Ordering::Equal { return o; }
            }
            (Some(x), Some(y)) => {
                if x != y { return x.cmp(&y); }
                ia.next(); ib.next();
            }
        }
    }
}

/// Lesbare Schrift auf einem Grund: dunkel auf hellem, weiss auf dunklem.
/// Gewichtet wie das Auge (Gruen zaehlt am meisten) -- ein gelber Knopf mit
/// weisser Schrift ist sonst unlesbar, obwohl sein Rotanteil niedrig ist.
/// Relative Leuchtdichte nach WCAG (0 = schwarz, 1 = weiss).
fn leuchtdichte(c: i64) -> f64 {
    let k = |v: i64| { let s = v as f64 / 255.0; if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) } };
    0.2126 * k((c >> 16) & 0xFF) + 0.7152 * k((c >> 8) & 0xFF) + 0.0722 * k(c & 0xFF)
}
/// Kontrastverhaeltnis zweier Farben (1..21).
fn kontrast(a: i64, b: i64) -> f64 {
    let (la, lb) = (leuchtdichte(a), leuchtdichte(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn lesbar_auf(bg: i64) -> i64 {
    let (r, g, b) = (((bg >> 16) & 0xFF) as f64, ((bg >> 8) & 0xFF) as f64, (bg & 0xFF) as f64);
    if 0.299 * r + 0.587 * g + 0.114 * b > 160.0 { 0x1C2026 } else { 0xFFFFFF }
}

/// Knopfarten (GUI_BUTTON_VARIANT). Index = Wert im Widget-Feld `variante`.
pub const KNOPF_ARTEN: &[&str] = &["standard", "primaer", "erfolg", "warnung", "gefahr", "umriss", "flach", "link"];

/// Ein Layout-Behaelter: unsichtbar, verteilt seine Kinder in jedem
/// GUI_UPDATE neu. Kinder sind Widget-Indizes desselben Fensters; -1 ist ein
/// Leerraum (GUI_LAYOUT_SPACER). Masse (abstand, rand) liegen UNskaliert --
/// der Massstab kommt erst im Durchlauf dazu, wie bei allen Laengen.
#[derive(Default, Clone)]
struct LayoutState {
    art: u8,                 // 0 zeile, 1 spalte, 2 raster
    spalten: i32,            // nur raster
    kinder: Vec<(i32, i32)>, // (Widget-Index oder -1, Gewicht)
    abstand: i32,
    rand: i32,
    ausrichtung: u8,         // quer zur Richtung: 0 Anfang, 1 Mitte, 2 Ende
    dehnen: bool,            // quer dehnen
    rahmen: bool,            // zum Entwickeln sichtbar machen
    /// Nur Fliesslayout (art 3): ein Kind per Klick waehlbar, und welches.
    auswahl: bool,
    gewaehlt: i32,
}

/// Die Einrueckungs-Regeln eines Textbereichs, aus dem Widget gezogen.
struct EinzugRegeln {
    auto_einzug: bool,
    einzug_anfang: Vec<String>,
    einzug_ende: Vec<String>,
    einzug_aus: Vec<String>,
    schluss_oeffner: Vec<String>,
    schluss_texte: Vec<String>,
    tabbreite: i32,
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
    on_click: Option<Rueckruf>,
    on_change: Option<Rueckruf>,
    /// GUI_CHANGED: einmal gefragt, wird das Widget beobachtet -- GUI_UPDATE
    /// merkt sich zu Beginn seinen Wert und vergleicht am Ende. So zaehlt nur,
    /// was die BEDIENUNG geaendert hat; ein Wert, den das Programm zwischen
    /// zwei Bildern setzt, ist keine Aenderung.
    beobachtet: bool,
    stand_vorher: String,
    geaendert: bool,
    /// Maus betritt / verlaesst das Widget, Fokus kommt / geht.
    on_hover: Option<Rueckruf>,
    on_leave: Option<Rueckruf>,
    on_focus: Option<Rueckruf>,
    on_blur: Option<Rueckruf>,
    /// Hover-Zustand des VORIGEN Bildes -- nur daraus laesst sich die FLANKE
    /// bestimmen. `hovered` allein wuerde jedes Bild feuern, solange die Maus
    /// stehenbleibt.
    was_hovered: bool,
    was_focused: bool,
    /// Uebergaenge 0..1, je Bild in `update` nachgefuehrt: Ueberfahren,
    /// Druecken, Fokus. Gezeichnet wird ueber `weich()` -- die Flags
    /// `hovered`/`press_origin`/`focus_widget` bleiben die Wahrheit fuer
    /// alles, was ENTSCHEIDET; diese drei nur fuer das, was man sieht.
    ueber_t: f32,
    druck_t: f32,
    fokus_t: f32,
    /// Nur mit eigenen Dauern je Farbrolle (Stildatei `uebergang: bg 400`):
    /// je Rolle (bg, fg, border, accent) Ueberfahren, Fokus, Druck -- wie
    /// die drei oben, nur mit der Dauer der Rolle nachgefuehrt.
    rollen_t: Option<Box<[[f32; 3]; 4]>>,
    /// Die ZEILE unter der Maus, weich: Liste, Tabelle, Baum und die offene
    /// Klappliste (dort ihr Eintrag).
    zeile_b: Blende,
    /// Zeile unter der Maus fuer die Blende, je Bild neu (eine schlichte
    /// Liste hat keinen ListState, der sie tragen koennte).
    zeile_jetzt: i32,
    /// Der GANZE Text der Listenzeile unter der Maus, wenn er in der Zeile
    /// gekuerzt dasteht ("..."); sonst leer. Gesetzt in GUI_UPDATE, gezeigt
    /// als Tooltip, wenn der Eintrag keinen eigenen hat.
    ganzer_text: String,
    /// Weiches Rollen mit dem Rad: das Ziel, und der Stand, den das Rollen
    /// zuletzt selbst geschrieben hat -- steht dort etwas anderes, hat jemand
    /// anderes gerollt (Pfeiltaste, SCROLL_TO), und das Rad gibt nach.
    rad_ziel: Option<f64>,
    rad_letzt: f64,
    ov: HashMap<String, i64>,
    tbl: Option<Box<TableState>>,   // nur fuer Kind::Table
    tree: Option<Box<TreeState>>,   // nur fuer Kind::Tree
    list: Option<Box<ListState>>,   // nur fuer Kind::ListBox, bei Bedarf
    layout: Option<Box<LayoutState>>, // nur fuer Kind::Layout
    /// Nur Kind::Panel, sobald es Kinder rollt (GUI_PANEL_ADD).
    panel: Option<Box<PanelState>>,
    /// Panel, in dem dieses Widget steckt (Widget-Index, -1 = keins). Wird
    /// je Bild aus den Panel-Kindern gesetzt -- auch durch Layouts hindurch,
    /// damit ein Behaelter im Panel seine Kinder mitnimmt.
    panel_von: i32,
    /// Reiter IM Fenster (Kind::TabControl): die Kinder je Seite. Am KIND
    /// stehen `tc_von` (Widget-Index des Reiterwerks, -1 = keins) und
    /// `tc_seite` -- je Bild gesetzt, wie beim rollenden Panel.
    tabctl: Option<Box<TabCtlState>>,
    tc_von: i32,
    tc_seite: i32,
    /// Leiste eines Docks, zu der das Widget gehoert (GUI_DOCK_ADD), -1 = keine.
    dock_von: i32,
    /// Nur Kind::Accordion bzw. Kind::Wizard.
    akk: Option<Box<AkkState>>,
    wz: Option<Box<WzState>>,
    /// Werkzeugleiste mit eigenen Eintraegen (Kind::Toolbar).
    leiste: Option<Box<LeisteState>>,
    /// Statusleiste (Kind::StatusBar) und Pfadleiste (Kind::Breadcrumb).
    status: Option<Box<StatusState>>,
    pfad: Option<Box<PfadState>>,
    /// Eingebautes Sinnbild eines Knopfes ("" = keins; sonst gewinnt es
    /// vor dem Textur-Handle in `sel`).
    sinnbild: String,
    /// Gesetzter Text (Kind::RichText).
    rich: Option<Box<RichState>>,
    vert: bool,          // Slider: senkrecht (GUI_VSLIDER), Wert waechst nach oben
    unbestimmt: bool,    // Progress: laufendes Band statt Wert
    bildmodus: u8,       // Image: 0 strecken, 1 einpassen, 2 fuellen, 3 mitte, 4 kacheln
    /// Mindestmasse (GUI_SET_MIN_SIZE): ein gewichtetes Kind im Behaelter
    /// faellt nie darunter, ein verankertes Widget beim Schrumpfen auch nicht.
    min_w: i32, min_h: i32,
    /// Natuerliche Groesse: was das Programm gesetzt oder das Automass
    /// gemessen hat -- NICHT, was ein Behaelter oder Anker daraus gemacht
    /// hat. Nur daraus laesst sich ein Mindestmass ableiten; `w`/`h` tragen
    /// nach dem ersten Layout-Durchlauf die gedehnte Groesse.
    nat_w: i32, nat_h: i32,
    /// Formularpruefung: Regeln (GUI_RULE), aktuelle Meldung (leer = ok) und
    /// eine Beschriftung, die die Meldung anzeigt (Widget-Index, -1 = keine).
    regeln: Vec<Regel>,
    fehler: String,
    fehler_label: i32,
    /// TextArea: an Wortgrenzen auf die Breite umbrechen (GUI_TEXTAREA_SET
    /// "umbruch"); dann gibt es keinen waagerechten Versatz.
    umbruch: bool,
    /// Datenbindung (GUI_BIND): Schluessel (= Spaltenname) und Formularname.
    bind: String,
    form: String,
    ziehbar: bool,       // Quelle fuer Ziehen (GUI_DRAGGABLE)
    ablage: bool,        // nimmt Abgelegtes an (GUI_DROP_TARGET)
    abgelegt: bool,      // in DIESEM Bild etwas abgelegt bekommen (transient, wie `clicked`)
    // Automass: Breite/Hoehe folgen dem Inhalt (0 beim Anlegen oder GUI_AUTOSIZE),
    // gemessen im naechsten GUI_UPDATE -- nur dort gibt es Graphics.
    auto_w: bool,
    auto_h: bool,
    // Laufzeit-Lifecycle (Tombstone -- Indizes/Handles bleiben stabil): `alive`
    // = nicht zerstoert, `visible` = wird gezeichnet + interaktiv.
    alive: bool,
    visible: bool,
    /// Rund statt eckig zeichnen (Transport-Knoepfe, Werkzeug-Kreise).
    rund: bool,
    /// Knopfart (Index in KNOPF_ARTEN): 0 = standard, 1 = primaer, ...
    variante: u8,
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
    /// Schriftstil (Bits aus `schnitt`): fett/kursiv waehlen den Schnitt
    /// der Schrift, unterstrichen/durchgestrichen sind Linien.
    stil: u8,
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
    /// Eingabemethoden: die unfertige Umwandlung (Vorschau) und die
    /// Schreibmarke darin -- steht an `caret`, unterstrichen, bis die IME
    /// ein Ergebnis liefert. Transient, nie in der Datei.
    vorschau: String,
    vorschau_marke: i32,
    /// Tab-Reihenfolge (GUI_SET_TAB_INDEX): Widgets mit Index > 0 kommen
    /// zuerst, aufsteigend; der Rest folgt in der Reihenfolge des Anlegens.
    tab_index: i32,
    // Schrittweite (nur Spinner): +/- / Pfeil / Mausrad aendert value um step.
    step: f64,
    // Farbige Abschnitte (nur TextArea): (start, laenge, farbe) in ZEICHEN,
    // aufsteigend nach start. Leer = alles in der Grundfarbe. Gesetzt wird das
    // von aussen (GUI_TEXTAREA_SPANS) -- welche Farbe ein Kommentar hat, ist
    // eine Frage des Themas, nicht der Laufzeit.
    spans: Vec<(u32, u32, i64)>,
    // Marken je ZEILE (0-basiert, Farbe): Haltepunkt, angehaltene Zeile,
    // Fehler -- ein Punkt in der Nummernspalte und ein Farbhauch ueber der
    // Zeile. GUI_SET_TEXT laesst sie stehen, das Programm setzt sie neu
    // (GUI_TEXTAREA_MARKS). Wird im Feld BEARBEITET, wandern sie mit ihrer
    // Zeile (`marken_abgleichen`) -- bis 2026-09-27 blieb ein Haltepunkt auf
    // seiner NUMMER stehen, und eine eingefuegte Zeile darueber legte ihn
    // still auf die Zeile davor. `marken_text` ist der Text, zu dem sie passen.
    marken: Vec<(usize, i64)>,
    marken_text: String,
    // Zeilenfarben (GUI_TEXTAREA_LINE_COLORS, seit 2026-09-29): ein Band
    // hinter der ganzen Zeile in der Farbe samt Deckkraft des Aufrufers --
    // fuer Wertungen je Zeile wie ein Profil ("wie heiss ist diese Zeile").
    // Getrennt von den Marken, die Haltepunkten und Lesezeichen gehoeren;
    // sie wandern wie diese mit ihrer Zeile (`marken_abgleichen`).
    zeilenfarben: Vec<(usize, i64)>,
    zeilenfarben_text: String,
    // Nur TextArea: was aus einem Textfeld ein Code-Feld macht.
    // `scroll` ist dort die erste SICHTBARE ZEILE, `scroll_x` der waagerechte
    // Versatz in Pixeln -- ohne den waeren lange Zeilen einfach abgeschnitten.
    scroll_x: i32,
    zeilennummern: bool,
    aktive_zeile: bool,
    tab_fuegt_ein: bool,
    tabbreite: i32,
    // Faltung (nur TextArea), 1-basierte Zeilen.
    //
    // `faltbar` sind die Bloecke, die sich falten LASSEN -- welche das sind,
    // weiss nur der Aufrufer (die IDE fragt CODE_SYMBOLS$); die Laufzeit
    // kennt keine Sprache. `gefaltet` sind die Kopfzeilen, die gerade
    // zugeklappt sind; verborgen ist dann `von+1 ..= bis`.
    faltbar: Vec<(usize, usize)>,
    gefaltet: Vec<(usize, usize)>,
    // Weitere Schreibmarken (nur TextArea), je (caret, anker) in ZEICHEN.
    // LEER heisst: genau eine Marke, und dann laeuft alles wie zuvor --
    // die eine ist `caret`/`sel_anchor` und bleibt es. Alt+Klick legt eine
    // dazu, ESC raeumt sie weg.
    marken_zusatz: Vec<(i32, i32)>,
    // Einrueckung beim Zeilenumbruch (nur TextArea). `auto_einzug` uebernimmt
    // die Einrueckung der laufenden Zeile; die drei Wortlisten machen daraus
    // eine, die die SPRACHE kennt -- welche Woerter das sind, weiss nur der
    // Aufrufer. Alle Woerter in Grossbuchstaben.
    auto_einzug: bool,
    einzug_anfang: Vec<String>,   // Zeile faengt damit an -> eine Stufe mehr
    einzug_ende: Vec<String>,     // Zeile endet damit -> eine Stufe mehr
    einzug_aus: Vec<String>,      // getippt -> die Zeile selbst eine zurueck
    // Das mitwachsende Geruest (GUI_TEXTAREA_CLOSE_WORDS): zwei gleich lange
    // Listen, je ein oeffnendes Wort und die Zeile, die es schliesst
    // (`IF` -> `END IF`, `FOR` -> `NEXT`). Leer = aus, und dann ist Enter,
    // was es vorher war.
    schluss_oeffner: Vec<String>,
    schluss_texte: Vec<String>,
    // Farbfelder (nur TextArea): (start, laenge, farbe) in ZEICHEN wie die
    // Abschnitte. Gezeichnet wird ein kleines Quadrat HINTER dem Stueck --
    // welche Stelle im Text eine Farbe MEINT, weiss nur der Aufrufer.
    farbfelder: Vec<(u32, u32, i64)>,
    // Wellenlinien (nur TextArea): (Start, Laenge, Farbe) in Zeichen, sortiert
    // nach Start -- Fehler und Warnungen DORT, wo sie im Text stehen, statt nur
    // als Marke am Rand. Laenge 0 = ein kurzes Stueck an der Stelle.
    wellen: Vec<(u32, u32, i64)>,
    /// Text hinter dem Ende einer Zeile (GUI_TEXTAREA_HINTS), etwa die Werte
    /// im Debugger-Halt: (Zeile ab 0, Text). Folgt seiner Zeile wie eine
    /// Marke; `hinweise_farbe` -1 = gedaempft.
    hinweise: Vec<(usize, String)>,
    hinweise_text: String,
    hinweise_farbe: i64,
    /// Eine Farbe je Hinweis, gleich lang wie `hinweise` (leer = alle in
    /// `hinweise_farbe`); -1 im Eintrag = gedaempft. Seit 2026-09-30: die IDE
    /// zeigt Fehler rot und Aufrufzaehler gedaempft auf demselben Weg.
    hinweise_farben: Vec<i64>,
    /// Nimmt ein Hinweis Klicks an (GUI_TEXTAREA_SET "hinweise_klickbar")?
    /// Dann Hand als Zeiger, die Marke bleibt stehen, und
    /// GUI_TEXTAREA_HINT_CLICKED nennt die Zeile (ab 1) ein Bild lang.
    hinweise_klickbar: bool,
    hinweis_klick: i64,
    // Welches davon in diesem Bild angeklickt wurde (-1 = keins). Transient
    // wie `clicked`: ein Klick ist ein Ereignis, kein Zustand.
    farbfeld_klick: i32,
    // Klick in die Nummernspalte (nur TextArea): (Zeile ab 1, Taste 0 links /
    // 1 rechts), (0, -1) = keiner. Transient wie `farbfeld_klick` -- was ein
    // Klick dort bedeutet (Haltepunkt, Lesezeichen), weiss nur der Aufrufer.
    rand_klick: (i32, i32),
    // Der laufende Zug begann auf einem Farbfeld (oder in der Nummernspalte):
    // dann zieht er auch keine
    // Auswahl. Ohne das setzte das naechste Bild -- die Taste ist noch
    // unten -- die Marke doch noch ans Feld.
    farbfeld_zug: bool,
    /// Eigene Zeigerform des Programms fuer dieses Widget (GUI_SET_CURSOR),
    /// None = die gui entscheidet.
    zeiger: Option<&'static str>,
    // Mit dem Rad gerollt (nur TextArea): Marke, Anker und Zeichenzahl zu
    // dem Zeitpunkt. Solange sich keins davon aendert, zieht die
    // Editierschleife den Ausschnitt NICHT zur Marke zurueck -- sonst
    // sprang ein gerollter Code nach einem Bild wieder zurueck.
    rad_stand: Option<(i32, i32, usize)>,
    /// Mehrfachklick im Textbereich: wie oft hintereinander (1..3), wann und
    /// wo -- zweimal waehlt das Wort, dreimal die Zeile.
    klick_n: u8,
    klick_zeit: f64,
    klick_idx: i32,
    /// Die Taste haengt noch vom Doppel-/Dreifachklick: der Zug darf die
    /// Wort- bzw. Zeilenauswahl nicht auf einen Punkt zusammenziehen.
    wort_zug: bool,
    /// Inline-Formate im Textbereich (`GUI_TEXTAREA_SET "formatiert"`):
    /// je Zeichen die Stil-Bits aus `schnitt` -- fett, kursiv,
    /// unterstrichen, durchgestrichen. `stile_text` ist der Text, zu dem
    /// die Bits gehoeren: aendert sich der Text auf einem Weg, der die Bits
    /// nicht kennt (Tippen, Einfuegen, GUI_TEXTAREA_INSERT, Eingabemethode),
    /// gleicht `stile_abgleichen` sie ueber gemeinsamen Anfang und gemeinsames
    /// Ende nach. `tipp_stil` ist der Stil fuer das naechste Getippte, wenn
    /// Strg+B ohne Auswahl kam -- er gilt, bis die Marke woanders hinwandert.
    formatiert: bool,
    /// Klammerpaare, die das Feld beim Tippen selbst schliesst
    /// (GUI_TEXTAREA_PAIRS) -- leer = aus.
    paare: Vec<(char, char)>,
    /// Zweite Ansicht auf einen anderen Textbereich (GUI_TEXTAREA_SHARE):
    /// dessen Nummer im selben Fenster und der zuletzt abgeglichene Text.
    /// Der Verlauf liegt nur beim Besitzer -- Strg+Z in der Ansicht nimmt
    /// dort zurueck, sonst haette jede Seite ihre eigene Geschichte.
    teil_von: Option<usize>,
    teil_stand: String,
    stile: Vec<u8>,
    stile_text: String,
    tipp_stil: Option<u8>,
    // Abkuerzungen (nur TextArea): Woerter, die der Tabulator MELDET statt
    // einzuruecken. Was an ihre Stelle kommt, weiss nur der Aufrufer -- die
    // Laufzeit kennt weder Schnipsel noch Sprache.
    abkuerzungen: Vec<String>,
    // Einrueckungslinien (nur TextArea): ein feiner senkrechter Strich je
    // Stufe, unter dem Text.
    einzugslinien: bool,
    // Mitlaufende Blockkoepfe (nur TextArea): ist die Kopfzeile eines
    // faltbaren Blocks oben hinausgerollt, der Block aber noch im Bild,
    // steht sie oben angeheftet -- hoechstens so viele. 0 = aus.
    kopfzeilen: i32,
    // Bild schwach im Hintergrund (nur TextArea), eingepasst in den
    // Innenbereich; -1 = keins. Deckkraft 0..255.
    hg_bild: i64,
    hg_deckkraft: u8,
    // Wo ein Alt-Zug begonnen hat (logische Zeile, Spalte), (-1, -1) = keiner.
    // Daraus wird beim Ziehen die SPALTENauswahl gebaut.
    spalten_start: (i32, i32),
    // Welche davon in diesem Bild getroffen wurde (-1 = keine). Transient
    // wie `clicked`: ein Tastendruck ist ein Ereignis, kein Zustand.
    abk_treffer: i32,
    // Der Tabulator gehoert ganz dem Aufrufer (GUI_TEXTAREA_SET "tab_meldet"):
    // das Feld rueckt dann nicht ein und sieht auch nicht nach Abkuerzungen,
    // es MELDET nur. Wer im selben Feld drei Bedeutungen fuer den Tabulator
    // hat (Platzhalter, Schnipsel, Einruecken), kann sie sonst nicht der
    // Reihe nach abfragen -- die Laufzeit entschiede die ersten zwei selbst.
    tab_meldet: bool,
    tab_treffer: bool,          // transient wie abk_treffer
    // Nur ColorPicker: Farbton (Grad), Saettigung, Hellwert.
    //
    // HSV und nicht RGB, weil der Farbton bei Schwarz und die Saettigung bei
    // Grau UNBESTIMMT sind. Wer nur die RGB-Farbe behaelt und beides beim
    // Zeichnen zurueckrechnet, verliert sie genau dort: der Zeiger springt
    // beim Herunterziehen auf Schwarz nach links, und beim Hochziehen kommt
    // Rot statt der Farbe zurueck, die man gewaehlt hatte.
    hsv: [f32; 3],
    // Nur ColorPicker: Deckkraft 1..255, und ob der Streifen dafuer sichtbar
    // ist. 0 gibt es NICHT -- die Laufzeit liest ein oberstes Byte von 0 als
    // "deckend" (damit die alten 24-bit-Farben deckend bleiben). Fuer
    // "praktisch unsichtbar" ist 1 der kleinste Wert.
    alpha: i32,
    alpha_an: bool,
    // Nur DatePicker: Jahr, Monat (1..12), Tag.
    datum: [i32; 3],
    /// Uhrzeit (Kind::TimePicker): Stunde, Minute, Sekunde -- und ob die
    /// Sekunden ueberhaupt dastehen.
    zeit: [i32; 3],
    zeit_sek: bool,
    // Grenzen (leer = keine) und erster Tag der Woche (0 = Montag).
    datum_min: Option<[i32; 3]>,
    datum_max: Option<[i32; 3]>,
    wochenbeginn: i32,
    /// Datumsfeld (GUI_DATEFIELD): ein Feld mit dem Datum, das den Monat zum
    /// Waehlen aufklappt -- statt des Monatsgitters im Fenster (wie QDateEdit
    /// mit Kalender).
    kompakt: bool,
    // Ausrichtung des Textes: -1 = Vorgabe der Art (Beschriftung und Textfeld
    // links, Knopf mittig), 0 links, 1 mitte, 2 rechts.
    align: i8,
    // Nur Label: bei `w` an Wortgrenzen umbrechen; die Hoehe folgt dem Text.
    wrap: bool,
    // Nur TextInput: Punkte statt Zeichen, kein Schreiben, Hoechstlaenge
    // (0 = frei), Zahlenfilter (0 frei, 1 ganze Zahl, 2 Kommazahl).
    passwort: bool,
    nur_lesen: bool,
    maxlaenge: i32,
    zahlen: u8,
    // Nur Klappliste: in das Feld darf getippt werden (Combobox). Der Text
    // steht dann in `text`, `sel` folgt ihm (der Eintrag, der genau so
    // heisst, sonst -1).
    frei: bool,
    // Enter im Textfeld -- GENAU EIN BILD lang, wie `clicked`.
    entered: bool,
    on_enter: Option<Rueckruf>,
    // Rueckgaengig beim Tippen (Textfeld und Textbereich): der Stand VOR einer
    // Aenderung samt Schreibmarke. Anschlaege innerhalb von 0,8 s sind EIN
    // Schritt -- sonst naehme Strg+Z ein Zeichen statt eines Wortes zurueck.
    undo: Vec<(String, i32, Vec<u8>)>,
    redo: Vec<(String, i32, Vec<u8>)>,
    undo_zeit: f64,
    /// Nur TextInput: Sinnbilder, Suchfeld, Vorschlaege (gui_ueber.rs) --
    /// nur bei Bedarf angelegt.
    feld: Option<Box<ueber::FeldZusatz>>,
    /// Name und Klassen fuer die Stildatei (GUI_SET_NAME, GUI_ADD_CLASS),
    /// klein geschrieben.
    name: String,
    klassen: Vec<String>,
    /// Was die Stildatei fuer dieses Widget sagt (je Schicht) und wofuer
    /// das aufgeloest wurde: (Stand der Stildatei, checked, enabled,
    /// Knopfart, Hash der Kette darueber). None = neu aufloesen.
    stil_auf: Option<Box<stil::StilAufgeloest>>,
    stil_key: Option<(u64, bool, bool, u8, u64)>,
    /// Zusaetze aus Paket 2 (gui_bausteine.rs) -- nur bei Bedarf angelegt.
    bau: Option<Box<bausteine::Bauzusatz>>,
    /// Rechtschreibung (GUI_SPELL).
    rs: Option<Box<rechtschreibung::RsZusatz>>,
}

/// Datenbank-Buchfuehrung eines Formulars (siehe `Window::form_db`).
#[derive(Default, Clone)]
struct FormDb {
    form: String,
    spalte: String,
    stand: Option<i64>,
    konflikt: String,
}

pub struct Window {
    title: String,
    /// Name fuer `window#name` in der Stildatei (GUI_WINDOW_SET_NAME).
    name: String,
    x: i32, y: i32, w: i32, h: i32,
    widgets: Vec<Widget>,
    movable: bool,
    closable: bool,
    visible: bool,
    resizable: bool,                 // am unteren-rechten Griff ziehbar?
    /// GUI_VALIDATE_LIVE: die Regeln eines Feldes laufen, sobald der Fokus
    /// es verlaesst -- nicht bei jedem Anschlag (eine halb getippte Adresse
    /// ist keine falsche).
    pruefung_live: bool,
    /// Stand der gebundenen Werte beim letzten Setzen/Leeren/Bereinigen:
    /// (Formular, Schluessel, Text). Daraus antwortet GUI_FORM_CHANGED.
    form_stand: Vec<(String, String, String)>,
    /// Je Formular, das mit der Datenbank spricht: Versionsspalte
    /// (GUI_FORM_VERSION), der beim Laden gelesene Stand und was das letzte
    /// GUI_FORM_SAVE verhindert hat (GUI_FORM_CONFLICT$).
    form_db: Vec<FormDb>,
    /// Entwurfsmodus (GUI_WINDOW_DESIGN): das Fenster wird gezeichnet, aber
    /// seine Widgets bekommen KEINE Eingabe -- kein Hover, kein Klick, kein
    /// Fokus, keine Rueckrufe. Ein Form-Designer legt so echte Widgets auf
    /// die Flaeche und verwaltet die Maus selbst (GUI_HIT_TEST geht weiter).
    entwurf: bool,
    /// Vergroesserung beim Zeichnen (GUI_WINDOW_ZOOM), nur im Entwurfsmodus:
    /// das Fenster bekommt dort keine Eingabe, also muss nur GUI_HIT_TEST
    /// (und wo ein Fenster liegt) die Mauslage zurueckrechnen.
    zoom: f32,
    /// Zuschnitt auf dem Schirm (GUI_WINDOW_CLIP), nur im Entwurfsmodus: was
    /// draussen liegt, wird nicht gezeichnet und trifft keinen Klick -- das
    /// Studio haelt so ein grosses Entwurfsfenster in seiner Mitte, statt es
    /// ueber Inspektor und Ausgabe zu legen.
    zuschnitt: Option<(i32, i32, i32, i32)>,
    chrome: bool,                    // Titelleiste + Rahmen + Buttons? (aus = randlos,
                                     //   z.B. wenn die Form das OS-Fenster fuellt)
    min_w: i32, min_h: i32,          // Groessen-Grenzen (0 = keine)
    max_w: i32, max_h: i32,
    base_w: i32, base_h: i32,        // Referenzgroesse fuer Anchoring (Layout-Basis)
    close_clicked: bool,
    alive: bool,   // Tombstone -- Fenster-Index bleibt als Handle stabil
    // Dialog (GUI_MESSAGE/GUI_CONFIRM): `dlg` markiert das Fenster als Dialog,
    // `answer` traegt die Antwort GENAU EIN BILD lang -- wie `clicked` bei den
    // Widgets, und aus demselben Grund: eine Antwort ist ein Ereignis, kein
    // Zustand. Wer sie behalten will, schreibt sie sich in eine Variable.
    dlg: bool,
    answer: i32,   // 0 = keine, 1 = OK/Ja, 2 = Abbrechen/Nein
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
    // Reiter mit Kreuz (GUI_TABS_CLOSABLE). Geschlossen wird NICHT von der
    // Laufzeit -- sie meldet nur, welches Kreuz getroffen wurde
    // (`tab_zu`, transient). Was ein geschlossener Reiter bedeutet, weiss
    // nur das Programm: es haengt sein Widget daran, und vielleicht will es
    // vorher fragen.
    tabs_closable: bool,
    tab_zu: i32,
    /// Der Reiter, auf dem zuletzt das Kontextmenue der Reiterleiste
    /// (GUI_CONTEXT_TABS) aufging, -1 = keiner.
    tab_kontext: i32,
    // Standard-Knopf (Enter) und Abbrechen-Knopf (ESC) -- Widget-Index im
    // Fenster oder -1. Was jedes Formular braucht und was sonst jedes
    // Programm selbst mit KEYHIT nachbaute, ohne den Fokus zu beachten.
    default_btn: i32,
    cancel_btn: i32,
    /// Lichtstreif ueber das Fenster (GUI_WINDOW_GLOW) -- "ein neuer
    /// Drachenhauch zieht durch".
    glanz: Glanz,
    /// Dock dieses Fensters (GUI_DOCK, gui_dock.rs).
    dock: Option<Box<dock::DockState>>,
    /// Steckt das Fenster selbst in einem Dock (GUI_DOCK_WINDOW): das
    /// Gastfenster, sonst -1.
    dock_host: i32,
    /// Ist das Fenster ein Popover (GUI_POPOVER): woran er haengt.
    pop: Option<Box<ueber::PopAnker>>,
    /// Meldungen unten im Fenster (GUI_TOAST), die neueste zuletzt.
    toasts: Vec<ueber::Toast>,
    /// Die Leiste oben im Inhalt (GUI_INFOBAR).
    info: Option<Box<ueber::InfoLeiste>>,
    /// Weg E: die Oberflaeche des Fensters -- 0 = im Hauptfenster, sonst
    /// ein echtes OS-Fenster (graphics.rs `nativ_*`). Ein natives steht
    /// auf seiner Oberflaeche bei (0, 0) und ohne eigene Titelleiste (die
    /// gibt das System); `nativ_vorher` merkt Lage und Rahmen fuer den
    /// Rueckweg (GUI_WINDOW_NATIVE ..., FALSE).
    nativ: u32,
    nativ_vorher: Option<(i32, i32, bool)>,
    /// Als abgerissene Dock-Leiste (Stufe 3) traegt das Fenster Titel,
    /// Kreuz und Griff der Leiste; vorher: (schliessbar, groesse aenderbar,
    /// Titel), zurueck beim Andocken.
    dock_raus_vorher: Option<(bool, bool, String)>,
}

/// Groesster Zeitschritt des Lichtstreifs je Bild (Sekunden), siehe
/// `Glanz::weiter`.
const GLANZ_SCHRITT_MAX: f32 = 0.05;

/// Ein Lichtstreif, der ueber ein Fenster zieht: ein weicher Schein, ein
/// hellerer Kern und eine heisse Linie, additiv gemischt, dazu ein Aufleuchten
/// des Rahmens, wo der Streif gerade ist. Die Zeit laeuft mit `g.delta()`
/// (ohne Fenster fest 1/60 s), ein Bild zeigt also immer dieselbe Stelle.
#[derive(Clone)]
struct Glanz {
    /// -1 = die Akzentfarbe des Themas.
    farbe: i64,
    /// Sekunden fuer einen Durchgang.
    dauer: f32,
    /// Sekunden zwischen zwei Durchgaengen; 0 = nur einmal.
    pause: f32,
    /// 0 nach rechts, 1 nach links, 2 nach unten, 3 nach oben.
    richtung: u8,
    /// Breite des Streifs als Anteil der Fensterbreite (bzw. -hoehe).
    breite: f32,
    /// Helligkeit 0..1.
    staerke: f32,
    /// Leuchtet der Rahmen mit?
    rand: bool,
    /// 0 = Streif (Baender quer ueber das Fenster), 1 = Lampe (das Fenster
    /// dunkelt ein wenig ab, ein weicher Lichtkegel zieht im Bogen darueber
    /// -- wie eine Taschenlampe, die jemand darueber schwenkt), 2 = Rahmen
    /// (Streiflicht: eine schraege Lichtkante zieht darueber, und jede
    /// Kante, die sie kreuzt -- Fensterrahmen, Rahmen der Widgets --, blitzt
    /// auf; der Inhalt bekommt nur einen Hauch).
    art: u8,
    /// Sekunden seit Beginn des Durchgangs; < 0 = aus.
    t: f32,
}


impl Glanz {
    /// Ein Bild weiter. Hoechstens `GLANZ_SCHRITT_MAX` je Bild: haengt ein
    /// Bild (das Programm prueft gerade eine grosse Datei), bleibt der Streif
    /// kurz stehen und zieht dann weiter -- mit der vollen Zeit sprang er um
    /// ein Viertel seines Wegs, und das sah aus wie ein Stottern (IDE-Start
    /// mit offener Datei, 614 ms in einem Bild).
    fn weiter(&mut self, dt: f32) {
        if self.t < 0.0 { return; }
        self.t += dt.clamp(0.0, GLANZ_SCHRITT_MAX);
        if self.t > self.dauer {
            if self.pause > 0.0 {
                if self.t > self.dauer + self.pause { self.t = 0.0; }
            } else {
                self.t = -1.0;
            }
        }
    }

    fn neu() -> Glanz {
        Glanz { farbe: 0xFFB050, dauer: 1.2, pause: 0.0, richtung: 0,
                breite: 0.35, staerke: 0.55, rand: true, art: 0, t: -1.0 }
    }
}

const GLANZ_SCHLUESSEL: &str = "farbe, dauer, pause, richtung, breite, staerke, rand, art";

struct Menu {
    label: String,
    in_bar: bool,
    // Untermenue eines anderen Menues: nie Kontextmenue, nie in der Leiste.
    unter: bool,
    items: Vec<MenuItem>,
    /// Aufrollen (0..1) und der Eintrag unter der Maus, weich -- nur solange
    /// das Popup offen ist; zu setzt beides zurueck.
    auf_t: f32,
    b: Blende,
    /// Kontextmenue eines Widgets (GUI_CONTEXT_WIDGET): oeffnet nur beim
    /// Rechtsklick auf dieses Widget (Index im Fenster), -1 = ueberall im
    /// Fenster, wo kein gebundenes trifft, -2 = auf einem Reiter der
    /// Reiterleiste (GUI_CONTEXT_TABS).
    ziel: i64,
}

struct MenuItem {
    label: String,
    separator: bool,
    enabled: bool,
    clicked: bool,     // gesetzt im Frame des Klicks (via GUI_CLICKED gelesen)
    // Tastenkuerzel: Anzeige ("Strg+S") und geparste Form -- Bits 1 Strg,
    // 2 Umschalt, 4 Alt; `k_code` ist der Tastencode der gui-Ebene, 0 = keins.
    kuerzel: String,
    k_mods: u8,
    k_code: i64,
    // Haekchen-Eintrag: `haken` = ist einer, `an` = Zustand. Ein Klick kippt.
    haken: bool,
    an: bool,
    // Sinnbild (Textur-Handle) links vom Text, -1 = keins.
    icon: i64,
    // Eingebautes Sinnbild ("" = keins) -- gewinnt vor `icon`.
    sym: String,
    // Untermenue: Index in `Window::menus`, -1 = keins.
    sub: i32,
}

impl MenuItem {
    fn neu(label: String, separator: bool) -> MenuItem {
        MenuItem { label, separator, enabled: !separator, clicked: false,
                   kuerzel: String::new(), k_mods: 0, k_code: 0,
                   haken: false, an: false, icon: -1, sym: String::new(), sub: -1 }
    }
}

/// Tastenkuerzel lesen: "Strg+S", "Strg+Umschalt+O", "F5", "Entf", "Alt+Enter".
/// Deutsche und englische Namen, Gross-/Kleinschreibung egal. Liefert
/// (Modifier-Bits, gui-Tastencode) -- derselbe Code-Raum wie KEY_*.
/// F1..F12 (Codes aus `vm::DEFAULT_KEYS`, zusammenhaengend).
fn ist_funktionstaste(code: i64) -> bool {
    (1073741882..=1073741893).contains(&code)
}

/// Plus und Minus sind im Kuerzel KEINE einzelne Taste: raylib benennt die
/// Tasten nach ihrer Lage im US-Layout, und das "+" einer deutschen Tastatur
/// liegt dort, wo die US-Tastatur "]" hat, ihr "-" dort, wo "/" liegt.
/// `Strg+Plus` trifft darum jede der Tasten, die auf einer der beiden
/// Belegungen "+" bedeutet, dazu den Ziffernblock -- sonst zoomte Strg+Plus
/// nur auf dem Ziffernblock oder nur auf einer Belegung.
const K_PLUS: i64 = -43;
const K_MINUS: i64 = -45;

/// Unbekannter Schluessel bei Thema/Metrik: die gueltigen nennen, sortiert --
/// wie bei den uebrigen *_SET (bis 2026.27 sagten diese vier nur "unbekannt").
fn unbekannt<'a>(befehl: &str, key: &str, bekannt: impl Iterator<Item = &'a String>) -> String {
    let mut namen: Vec<&str> = bekannt.map(|s| s.as_str()).collect();
    namen.sort_unstable();
    format!("{}: unbekannter Schluessel '{}' (bekannt: {})", befehl, key, namen.join(", "))
}

fn kuerzel_taste(g: &Graphics, code: i64) -> bool {
    match code {
        K_PLUS => [1073741911, 61, 93].iter().any(|&c| g.key_pressed(c)),
        K_MINUS => [1073741910, 45, 47].iter().any(|&c| g.key_pressed(c)),
        c => g.key_pressed(c),
    }
}

fn kuerzel_parsen(s: &str) -> Result<(u8, i64), String> {
    let mut mods = 0u8;
    let mut code = 0i64;
    // "Strg++" -- das Plus als Taste steht hinter dem Trenner-Plus.
    let (s_teile, plus_am_ende) = match s.trim_end().strip_suffix("++") {
        Some(rest) => (rest.to_string(), true),
        None => (s.to_string(), false),
    };
    if plus_am_ende { code = K_PLUS; }
    for teil in s_teile.split('+') {
        let t = teil.trim().to_lowercase();
        if t.is_empty() { continue; }
        match t.as_str() {
            "strg" | "ctrl" | "steuerung" | "control" => { mods |= 1; continue; }
            "umschalt" | "shift" => { mods |= 2; continue; }
            "alt" => { mods |= 4; continue; }
            _ => {}
        }
        if code != 0 {
            return Err(format!("Tastenkuerzel '{}': mehr als eine Taste", s));
        }
        let ch: Vec<char> = t.chars().collect();
        // Punkt und Komma liegen auf der deutschen Tastatur dort, wo raylib
        // (nach US-Lage benannt) sie sucht -- anders als Plus und Minus.
        if ch.len() == 1 && (ch[0].is_ascii_lowercase() || ch[0].is_ascii_digit() || ch[0] == '.' || ch[0] == ',') {
            code = ch[0] as i64;
            continue;
        }
        let name = match t.as_str() {
            "entf" | "delete" | "del" => "key_delete",
            "einfg" | "insert" | "ins" => "key_insert",
            "pos1" | "home" => "key_home",
            "ende" | "end" => "key_end",
            "bild auf" | "bild-auf" | "bildauf" | "pageup" | "bild hoch" => "key_pageup",
            "bild ab" | "bild-ab" | "bildab" | "pagedown" | "bild runter" => "key_pagedown",
            "leer" | "leertaste" | "space" => "key_space",
            "enter" | "eingabe" | "return" => "key_enter",
            "esc" | "escape" => "key_escape",
            "tab" | "tabulator" => "key_tab",
            "rueck" | "ruecktaste" | "backspace" => "key_backspace",
            "links" | "left" => "key_left",
            "rechts" | "right" => "key_right",
            "hoch" | "auf" | "up" => "key_up",
            "runter" | "ab" | "down" => "key_down",
            "punkt" | "period" => { code = 46; continue; }
            "komma" | "comma" => { code = 44; continue; }
            "plus" => { code = K_PLUS; continue; }
            "minus" | "-" => { code = K_MINUS; continue; }
            _ => {
                if t.len() >= 2 && t.starts_with('f') && t[1..].chars().all(|c| c.is_ascii_digit()) {
                    // F1..F12 ueber die Tabelle unten
                    ""
                } else {
                    return Err(format!(
                        "Tastenkuerzel '{}': Taste '{}' unbekannt -- Buchstabe, Ziffer, F1..F12, Entf, Einfg, \
                         Pos1, Ende, Bild auf/ab, Leer, Enter, Esc, Tab, Rueck, Links/Rechts/Hoch/Runter, Plus, Minus, Punkt, Komma",
                        s, teil.trim()));
                }
            }
        };
        let key_name = if name.is_empty() { format!("key_{}", t) } else { name.to_string() };
        code = crate::vm::DEFAULT_KEYS.iter().find(|(n, _)| *n == key_name).map(|(_, v)| *v)
            .ok_or_else(|| format!("Tastenkuerzel '{}': Taste '{}' unbekannt", s, teil.trim()))?;
    }
    if code == 0 {
        return Err(format!("Tastenkuerzel '{}': keine Taste angegeben", s));
    }
    Ok((mods, code))
}

// Aufgeloestes Tabellen-Layout (einzige Wahrheit fuer Hit-Test + Zeichnen).
struct TGeom {
    ax: i32, ay: i32, w: i32, h: i32,
    n_cols: usize, n_rows: usize, col_widths: Vec<i32>,
    /// Zeilen- und Kopfhoehe sind einstellbar (Bilder in Zellen brauchen
    /// Platz) -- sie stehen hier, damit Treffertest und Zeichnen dieselbe
    /// Quelle benutzen.
    row_h: i32, header_h: i32,
    /// Waagerechter Scroll-Stand, aus dem der Treffertest die Spalte
    /// bestimmt. Ohne ihn haette `col_at` bei gescrollter Tabelle auf die
    /// falsche Spalte gezeigt.
    scroll_x: i32,
    /// Anzeige-Position -> DATENspalte. `col_widths` steht in ANZEIGE-Reihenfolge;
    /// wer an die Daten will, geht ueber diese Abbildung. Beides zusammen an
    /// einer Stelle, damit Zeichnen und Treffertest nie verschiedene
    /// Reihenfolgen benutzen.
    order: Vec<usize>,
    /// Anzahl fester Spalten und ihre Gesamtbreite. Alles, was Spalten
    /// verortet, geht ueber `col_x`/`col_clip` -- eine Quelle fuer Treffertest
    /// UND Zeichnen, sonst waere ein fester Block der sicherste Weg, beide
    /// auseinander laufen zu lassen.
    frozen: usize,
    frozen_w: i32,
    body_x: i32, body_y: i32, body_w: i32, body_h: i32,
    body_w_raw: i32, total_w: i32, total_h: i32,
    need_v: bool, need_h: bool, max_scroll_y: i32, max_scroll_x: i32,
}

pub struct Gui {
    /// GUI_FONT: die eigene Schrift (Handle, Groesse) der gui. `None` = wie
    /// bisher die AKTIVE Schrift des Programms. Gesetzt, tauscht die VM sie
    /// fuer jeden gui-Befehl ein -- ein Programm, das zwischendurch mit einer
    /// anderen Schrift zeichnet (Notensatz, Symbole), verdirbt damit nicht
    /// mehr die Oberflaeche.
    pub schrift: Option<(i64, i32)>,
    windows: Vec<Window>,        // stabile Indizes (Handles!)
    // Zeichen und Zeilenanfaenge eines Textbereichs, je Handle, solange sein
    // Text derselbe ist (verglichen wird der Text selbst -- es gibt viele
    // Wege, ihn zu aendern, und ein Zaehler muesste an jedem stehen). Ein
    // Programm fragt GUI_TEXTAREA_CURSOR und _VIEW mehrmals je Bild; bei
    // 100000 Zeilen kostete jede Frage 2 ms fuer das Zerlegen, der
    // Vergleich kostet einen Bruchteil davon (Stresstest 2026-10-02).
    ta_zerlegung: std::cell::RefCell<HashMap<i64, (String, std::rc::Rc<(Vec<char>, Vec<usize>)>)>>,
    z_order: Vec<usize>,         // Zeichen-/Hit-Reihenfolge (umordbar)
    /// Die Oberflaeche, durch die gui gerade schaut (Weg E): waehrend
    /// GUI_UPDATE die unter dem Zeiger. Nur ihre Fenster sind treffbar.
    flaeche: u32,
    /// Welche Oberflaeche hatte im letzten Bild den Tastaturfokus des Systems?
    fokus_flaeche: u32,
    /// Ein natives Fenster, das den Fokus des Systems bekommen soll (ein
    /// Klick ins Hauptfenster, waehrend es modal ist) -- im naechsten
    /// Abgleich, der die Grafik hat.
    nativ_fokus_wunsch: Option<u32>,
    focus_window: Option<usize>,
    focus_widget: Option<(usize, usize)>,
    // Offener modaler Dialog. Solange einer steht, bekommt NUR er Eingaben --
    // das ist der ganze Sinn von "modal": die Frage muss beantwortet werden,
    // bevor daneben weitergeklickt wird.
    modal: Option<usize>,
    // Was vorher modal war: ein Dialog ueber einem modalen Fenster (Frage im
    // Detailfenster) gibt bei seiner Antwort das Fenster darunter wieder
    // frei -- vorher fiel die Modalitaet ganz weg, und das Fenster verlor mit
    // ihr Enter und ESC (Standard- und Abbrechen-Knopf).
    modal_vorher: Vec<usize>,
    // Der Tastaturfokus, den ein modales Fenster beim Aufgehen vorfand: es
    // nimmt ihn einem Widget in einem anderen Fenster weg und gibt ihn beim
    // Gehen zurueck. Vorher blieb er auf der Tabelle dahinter, und Enter
    // oeffnete dort per "Doppelklick" das naechste Fenster, statt den
    // Standard-Knopf des modalen zu druecken (Kuehlschrank-App, Rezepte).
    modal_fokus: Vec<(usize, Option<(usize, usize)>)>,
    // Gezogener Farbwaehler: (Fenster, Widget, Zone). Zone ist 0 = Farbfeld,
    // 1 = Ton-Streifen, 2 = Deckkraft-Streifen. Gemerkt wird sie, weil man
    // beim Suchen einer Farbe fast immer ueber den Rand hinauszieht -- ohne
    // das endete jede Geste an der naechsten Kante, und beim Ueberfahren der
    // Grenze spraenge die Farbe in die andere Bedeutung.
    cp_drag: Option<(usize, usize, u8)>,
    drag_window: Option<usize>,
    drag_dx: i32, drag_dy: i32,
    resize_window: Option<usize>,        // laufender Fenster-Resize (am Griff)
    resize_dx: i32, resize_dy: i32,      // Ecke->Maus-Versatz
    /// Eigene Grafik je Widget-Art: Name -> (IMAGE-Handle, Randbreite).
    /// Wo eine hinterlegt ist, ersetzt sie die gezeichnete Flaeche; alles
    /// andere (Text, Haekchen, Griffe) bleibt.
    skins: HashMap<String, (i64, i32)>,
    active_slider: Option<(usize, usize)>,
    /// Laufendes Drehregler-Ziehen: (Fenster, Widget, Maus-y beim Griff,
    /// Wert beim Griff). Der Startwert wird gemerkt, damit der Regler nicht
    /// wegspringt, wenn man ihn irgendwo auf der Kappe anfasst.
    active_knob: Option<(usize, usize, i32, f64)>,
    active_split: Option<(usize, usize)>,    // laufendes Splitter-Drag
    split_off: i32,                          // Griff-Versatz (Maus -> Balkenkante)
    open_dropdown: Option<(usize, usize)>,   // gerade aufgeklapptes Dropdown
    /// Aufgeklappter Monat: (Fenster, Widget, Zelle). Zelle = FALSE: ein
    /// Datumsfeld; TRUE: die Zelle einer Datumsspalte, die gerade bearbeitet
    /// wird (das Widget ist die Tabelle).
    datum_auf: Option<(usize, usize, bool)>,
    /// Der Tag, der im aufgeklappten Monat markiert ist (noch nicht gewaehlt).
    datum_zeigt: [i32; 3],
    // Bearbeitbare Klappliste: steht die Markierung, weil jemand mit den
    // Pfeilen dorthin ging? Nur dann nimmt Enter den Eintrag -- ein beim
    // Tippen VORGESCHLAGENER Eintrag ersetzte sonst, was man getippt hat.
    dd_pfeil: bool,
    /// Wie weit die offene Klappliste schon aufgeklappt ist (0..1), und fuer
    /// welche -- wechselt die Liste, faengt es bei 0 an.
    dd_auf_t: f32,
    dd_auf_von: Option<(usize, usize)>,
    /// Offene Klappliste: der Eintrag, den die Tastatur markiert (Enter
    /// uebernimmt ihn, ESC verwirft), die erste sichtbare Zeile, und der
    /// Tipp-Puffer fuer "tippen springt".
    dd_mark: i32,
    dd_scroll: i32,
    dd_tipp: String,
    dd_tipp_zeit: f64,
    /// Bildschirmhoehe aus dem letzten GUI_UPDATE -- ob eine Klappliste nach
    /// unten Platz hat, fragt ihre Geometrie, und die kennt kein `g`.
    schirm_h: i32,
    schirm_b: i32,
    /// Rollbalken einer Liste wird gezogen: (Fenster, Liste, Griff-Versatz).
    listbar_zug: Option<(usize, usize, i32)>,
    /// Dasselbe fuer den Rollbalken eines Textbereichs.
    tabar_zug: Option<(usize, usize, i32)>,
    /// Enter oder ESC hat in DIESEM Bild ein Umbenennen beendet -- dann
    /// gehoert die Taste nicht mehr dem Standard-/Abbrechen-Knopf, obwohl
    /// die Liste danach nicht mehr bearbeitet wird.
    liste_taste: bool,
    /// Ein Drehfeld, in das gerade getippt wird: (Fenster, Widget, Zeichen,
    /// Schreibmarke, Anker, der Wert, zu dem der Text zuletzt passte, der Wert
    /// vor dem Tippen). Der Wert gilt schon, wenn der Text eine Zahl im Bereich
    /// ist; ganz uebernommen (geklemmt) wird beim Verlassen, ESC holt den Wert
    /// von vorher zurueck. Weicht der Wert ab -- Pfeil, Rad, GUI_SET_VALUE --,
    /// wird der Text neu aus ihm gebaut.
    spin_edit: Option<(usize, usize, Vec<char>, i32, i32, f64, f64)>,
    open_menu: Option<(usize, usize)>,       // offenes Menueleisten-Dropdown (win, menu)
    context_open: Option<(usize, usize, i32, i32)>,  // Kontextmenue (win, menu, x, y)
    /// Das Widget (Fenster, Index), dessen gebundenes Kontextmenue IN
    /// DIESEM BILD aufging -- es bekommt seinen Rechtsklick trotzdem (ein
    /// Textbereich setzt die Marke, bevor ein Befehl aus dem Menue sie liest).
    kontext_neu: Option<(usize, usize)>,
    /// Offenes Menue mit Rechtschreib-Vorschlaegen.
    rs_menue: Option<rechtschreibung::RsMenue>,
    // Strg/Umschalt zum Zeitpunkt des Drucks -- handle_press hat kein `g`.
    tasten_mod: (bool, bool),
    // Hat in diesem Bild ein Menue-Kuerzel gefeuert? Dann gehoert die Taste
    // dem Menue -- und NICHT zusaetzlich der Navigation im Textfeld.
    kuerzel_gefeuert: bool,
    // Offene Untermenues unter dem offenen Wurzelmenue: (Menue-Index, x, y)
    // je Ebene. Leer, wenn keine Kette offen ist.
    sub_chain: Vec<(usize, i32, i32)>,
    was_right_down: bool,                    // Rechtsklick-Flankenerkennung
    was_mitte_down: bool,                    // Mittelklick-Flankenerkennung (Reiter zu)
    scroll_drag: Option<usize>,              // Fenster, dessen Inhalts-Scrollbar gezogen wird
    active_table: Option<(usize, usize)>,
    table_press: Option<(usize, usize, i32)>,   // (win, widget, row)
    /// Tabelle mit laufender Zellbearbeitung. Ohne diesen Merker muesste man
    /// alle Fenster durchsuchen, um einen Klick woanders zu bemerken -- und
    /// ein vergessener Editor bliebe verwaist offen stehen.
    editing_table: Option<(usize, usize)>,
    /// Zeitpunkt und Ort des letzten Drucks -- daraus wird der Doppelklick
    /// erkannt. Zeit statt Bildzaehler, damit die Erkennung nicht von der
    /// Bildrate abhaengt.
    last_click: Option<(f64, i32, i32)>,
    /// Gilt der gerade laufende Druck als Doppelklick? Wird in `update` (dort
    /// gibt es die Zeit) gesetzt und von `handle_press` gelesen.
    dbl_click: bool,
    press_origin: Option<(usize, usize)>,
    /// Ziehen: begonnen beim Druck auf ein ziehbares Widget, aktiv ab 5 px.
    drag: Option<DragState>,
    /// Ablage dieses Bildes (transient).
    drop: Option<DropInfo>,
    /// Cursorformen ueber Widgets (GUI_CURSORS, Vorgabe an). Die gui meldet je
    /// Bild einen Wunsch an Graphics, gesetzt wird beim FLIP.
    cursors: bool,
    was_mouse_down: bool,
    /// Das Text-Widget, in dem der laufende Druck BEGANN. Nur dort markiert
    /// Ziehen. Bekommt ein Feld den Fokus erst, waehrend die Taste schon
    /// unten ist (Klick in eine Liste, das Programm setzt den Fokus in den
    /// Text), hielt es das Halten fuer seinen eigenen Zug und markierte bis
    /// zur Maus -- gefunden beim Dogfooding 2026-10-09: die ganze Notiz war
    /// markiert, und das erste Tippen haette sie ersetzt.
    zug_feld: Option<(usize, usize)>,
    frame_count: i64,
    theme: HashMap<String, i64>,
    metrics: HashMap<String, i32>,
    /// Die Stildatei (GUI_STYLESHEET, gui_stil.rs) und ihr Stand -- jede
    /// neue zaehlt hoch, damit die Widgets neu aufloesen.
    stil: Option<Box<stil::Stildatei>>,
    stil_stand: u64,
    /// Eckenrundung des Widgets, das gerade gezeichnet wird (aus der
    /// Stildatei), None = die Metrik. Gesetzt in `draw_widget`, gelesen in
    /// `flaeche` -- so gilt sie fuer jede Flaeche des Widgets.
    rad_jetzt: std::cell::Cell<Option<i32>>,
    /// Verlauf, Glanz, Fase und Rand des Widgets, das gerade gezeichnet wird.
    form_jetzt: std::cell::Cell<stil::FormStil>,
    /// Alt-Buchstaben (GUI_MNEMONICS): `&` im Text markiert sie.
    mnemonics: bool,
    /// Markierbare Beschriftung: die, in der gerade gezogen wird, und die,
    /// deren Auswahl Strg+C kopiert.
    label_zug: Option<(usize, usize)>,
    label_wahl: Option<(usize, usize)>,
    /// Groessengruppen (GUI_SIZE_GROUP): das groesste Mass je Gruppe.
    gruppen: HashMap<String, (i32, i32)>,
    /// Offene Schriftwahlen (GUI_FONT_DIALOG).
    schriftwahl: Vec<bausteine::SchriftWahl>,
    // Benannte Styles (Stylesheet): Name -> {prop -> wert}. Props: bg/fg/border/
    // accent (Farbe), font (Handle), font_size (px). Via GUI_APPLY_STYLE auf
    // Widgets uebertragen (-> ov-Overrides + font/font_size).
    styles: HashMap<String, HashMap<String, i64>>,
    // In diesem Frame ausgeloeste FUNCREF-Callbacks (Namen). Die VM leert die
    // Liste nach GUI_UPDATE und ruft sie auf -- so kann ein Callback nicht
    // mitten im State-Update die GUI re-entrant veraendern.
    pending: Vec<Rueckruf>,
    // Tooltip-Dwell: ueber welchem Widget (Fenster, Widget) ruht die Maus und seit
    // welchem frame_count? Mausbewegung/Klick setzt den Timer zurueck; nach
    // TOOLTIP_DELAY ruhenden Frames zeigt draw() den Hilfetext.
    hover_w: Option<(usize, usize)>,
    hover_frame: i64,
    /// Knopf einer Werkzeugleiste unter der Maus (-1 = keiner) -- ein Wechsel
    /// startet den Tooltip-Verweil neu, wie ein Wechsel des Widgets.
    hover_teil: i32,
    /// Offenes Ueberlauf-Menue einer Werkzeugleiste (Fenster, Widget).
    leiste_popup: Option<(usize, usize)>,
    hover_x: i32, hover_y: i32,
    // Anzeige-Massstab (GUI_SCALE). Multipliziert JEDE Laengenangabe, die in
    // die GUI hineingeht: Fenster-/Widget-Geometrie beim Anlegen, die
    // Chrome-Metriken und die Schriftgroesse. Wirkt nur beim ANLEGEN -- was
    // schon steht, wird nicht nachtraeglich umgerechnet (darum lehnt
    // `set_scale` einen Wechsel ab, solange Fenster existieren).
    scale: f64,
    // --- Bedienung ohne Maus, zweiter Teil (docs/intern/entwurf-barrierefreiheit.md, Weg A)
    /// Per Tastatur gefuehrter Eintrag im tiefsten offenen Leistenmenue
    /// (Index in `items`); None = die Maus fuehrt.
    menu_cursor: Option<usize>,
    /// Alt ist gedrueckt und bisher OHNE andere Taste -- beim Loslassen
    /// oeffnet es das Menue (wie ueberall unter Windows). Ein Kuerzel mit Alt
    /// loescht den Merker.
    alt_allein: bool,
    /// Tooltip am Widget, das per TASTATUR den Fokus bekam, und seit welchem
    /// Bild -- ohne das stand der Hilfetext nur der Maus zur Verfuegung.
    tip_fokus: Option<(usize, usize)>,
    tip_fokus_frame: i64,
    // --- Barrierefreiheit (Weg C): Ansage und ob ein Hilfsprogramm zuhoert
    ansage: String,
    ansage_dringend: bool,
    ansage_nr: u32,
    /// Von der VM je Bild gesetzt: hat ein Bildschirmleser (o.ae.) den Baum
    /// angefordert? GUI_SCREENREADER() liest es.
    pub screenreader: bool,
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
            schrift: None,
            windows: Vec::new(), z_order: Vec::new(), flaeche: 0, fokus_flaeche: 0, nativ_fokus_wunsch: None,
            ta_zerlegung: std::cell::RefCell::new(HashMap::new()),
            focus_window: None, focus_widget: None, modal: None, modal_vorher: Vec::new(), modal_fokus: Vec::new(), cp_drag: None,
            drag_window: None, drag_dx: 0, drag_dy: 0,
            resize_window: None, resize_dx: 0, resize_dy: 0,
            skins: HashMap::new(),
            active_slider: None,
            active_knob: None, active_split: None, split_off: 0,
            open_dropdown: None, datum_auf: None, datum_zeigt: [2000, 1, 1], dd_auf_t: 0.0, dd_auf_von: None, dd_mark: -1, dd_pfeil: false, dd_scroll: 0, dd_tipp: String::new(), dd_tipp_zeit: 0.0, schirm_h: 0, schirm_b: 0, listbar_zug: None, tabar_zug: None, liste_taste: false, spin_edit: None, active_table: None, table_press: None, press_origin: None,
            drag: None, drop: None, cursors: true,
            editing_table: None, last_click: None, dbl_click: false,
            open_menu: None, context_open: None, kontext_neu: None, rs_menue: None, sub_chain: Vec::new(), tasten_mod: (false, false),
            kuerzel_gefeuert: false, was_right_down: false, was_mitte_down: false,
            scroll_drag: None,
            was_mouse_down: false, zug_feld: None, frame_count: 0,
            theme: default_theme(), metrics: default_metrics(),
            stil: None, stil_stand: 0, rad_jetzt: std::cell::Cell::new(None), form_jetzt: std::cell::Cell::new(stil::FormStil::default()),
            mnemonics: false, label_zug: None, label_wahl: None, gruppen: HashMap::new(), schriftwahl: Vec::new(),
            styles: HashMap::new(),
            pending: Vec::new(),
            hover_w: None, hover_frame: 0, hover_teil: -1, leiste_popup: None, hover_x: 0, hover_y: 0,
            scale: 1.0,
            menu_cursor: None, alt_allein: false, tip_fokus: None, tip_fokus_frame: 0,
            ansage: String::new(), ansage_dringend: false, ansage_nr: 0, screenreader: false,
        }
    }

    /// Entnimmt die in diesem Frame ausgeloesten Callback-Namen (FIFO).
    pub fn take_pending(&mut self) -> Vec<Rueckruf> { std::mem::take(&mut self.pending) }

    pub fn reset(&mut self) {
        let theme = default_theme();
        let metrics = default_metrics();
        let scale = self.scale;
        *self = Gui::new();
        self.theme = theme;
        self.metrics = metrics;
        self.scale = scale;   // globale Einstellung, wie Thema/Metriken
    }

    /// Eine Laenge auf den Anzeige-Massstab bringen. Jede feste Pixelzahl im
    /// Modul laeuft hier durch -- eine einzige Stelle, an der aus 20 px bei
    /// Massstab 2 vierzig werden. Mindestens 1, sonst verschwaende eine
    /// Trennlinie bei kleinem Massstab ganz.
    fn sk(&self, n: i32) -> i32 {
        if self.scale == 1.0 { return n; }
        let v = (n as f64 * self.scale).round() as i32;
        if n > 0 { v.max(1) } else { v }
    }

    /// Metriken, die PIXEL sind und darum mitskalieren. Der Rest der
    /// Metrik-Tabelle sind Staerken und Zaehler (`gradient` = Helligkeits-
    /// Abstand, `gloss` = 0..100, `bevel` = Schalter, `caret_period` =
    /// Bilder) -- die zu skalieren waere schlicht falsch.
    const M_PIXEL: [&'static str; 7] = [
        "title_h", "slider_h", "check_size", "slider_handle_w",
        "pad", "corner_radius", "shadow",
    ];

    fn m(&self, k: &str) -> i32 {
        let roh = self.m_roh(k);
        if Self::M_PIXEL.contains(&k) { self.sk(roh) } else { roh }
    }
    /// Metrik OHNE Massstab. Nur dort richtig, wo der Wert anschliessend noch
    /// durch `add_widget` laeuft -- der skaliert selbst, und zweimal waere
    /// zweimal zu viel.
    fn m_roh(&self, k: &str) -> i32 { *self.metrics.get(k).unwrap_or(&0) }
    /// Eine Plastik-Metrik fuer das Widget, das gerade gezeichnet wird: die
    /// Stildatei (`verlauf`, `glanz`, `fase`) vor der Metrik.
    fn m_form(&self, k: &str) -> i32 {
        let f = self.form_jetzt.get();
        let eigen = match k { "gradient" => f.verlauf, "gloss" => f.glanz, "bevel" => f.fase, _ => None };
        eigen.unwrap_or_else(|| self.m(k))
    }
    fn th(&self, k: &str) -> i64 { *self.theme.get(k).unwrap_or(&0) }

    /// Anzeige-Massstab setzen. Muss VOR dem ersten Fenster kommen: schon
    /// angelegte Widgets nachtraeglich umzurechnen ginge nur naeherungsweise
    /// (jede Runde neue Rundungsfehler), und eine halb skalierte Oberflaeche
    /// waere schlimmer als eine klare Absage.
    pub fn set_scale(&mut self, f: f64) -> Result<(), String> {
        if !(0.5..=4.0).contains(&f) {
            return Err("GUI_SCALE: Faktor muss zwischen 0.5 und 4.0 liegen".into());
        }
        if !self.windows.is_empty() {
            return Err("GUI_SCALE: erst den Massstab setzen, dann Fenster und Widgets anlegen (bereits angelegte werden nicht umgerechnet)".into());
        }
        self.scale = f;
        Ok(())
    }
    pub fn get_scale(&self) -> f64 { self.scale }

    /// Eine Laenge aus dem Massstab zurueckrechnen. Gegenstueck zu `sk`: was
    /// nach aussen geht (Getter, gespeicherte Form) ist wieder in der Groesse,
    /// in der das Programm sie hineingegeben hat. Ohne das waere
    /// `GUI_SET_BOUNDS(w, GUI_GET_X(w) + 10, ...)` bei Massstab 2 ein Sprung
    /// um das Doppelte, und eine gespeicherte `.dhform` wuechse mit jedem
    /// Speichern weiter.
    fn unsk(&self, n: i32) -> i32 {
        if self.scale == 1.0 { return n; }
        (n as f64 / self.scale).round() as i32
    }

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
        let (x, y, w, h) = (self.sk(x), self.sk(y), self.sk(w), self.sk(h));
        let idx = self.windows.len();
        self.windows.push(Window {
            title, name: String::new(), x, y, w, h, widgets: Vec::new(),
            movable: true, closable: false, visible: true, entwurf: false, zoom: 1.0, zuschnitt: None,
            resizable: false, pruefung_live: false, form_stand: Vec::new(), form_db: Vec::new(), chrome: true, min_w: 0, min_h: 0, max_w: 0, max_h: 0,
            base_w: w, base_h: h,
            close_clicked: false, alive: true, dlg: false, answer: 0,
            menus: Vec::new(),
            scrollable: false, scroll_y: 0,
            tabs: Vec::new(), active_tab: 0, tabs_closable: false, tab_zu: -1, tab_kontext: -1,
            default_btn: -1, cancel_btn: -1,
            glanz: Glanz::neu(),
            dock: None, dock_host: -1,
            pop: None, toasts: Vec::new(), info: None,
            nativ: 0, nativ_vorher: None, dock_raus_vorher: None,
        });
        self.z_order.push(idx);
        self.focus_window = Some(idx);
        idx as i64
    }

    /// Modales Meldungs-/Rueckfragefenster.
    ///
    /// `frage = false` gibt einen "OK"-Knopf (Meldung), `true` gibt "Ja"/"Nein"
    /// (Rueckfrage). Das Fenster wird auf dem Bildschirm zentriert und passt
    /// sich dem Text an; `\n` im Text trennt Zeilen.
    ///
    /// Es ist BEWUSST nicht blockierend. `gui` ist ein Polling-Toolkit -- ein
    /// blockierender Dialog muesste mitten im Bild des Aufrufers eine eigene
    /// Zeichenschleife drehen und wuerde dessen Layer-/Render-Ziel-Zustand
    /// mitten im Frame uebernehmen. Stattdessen liegt die Antwort ein Bild
    /// lang in `GUI_ANSWER` an, genau wie ein Klick in `GUI_CLICKED`.
    pub fn dialog(&mut self, g: &Graphics, titel: String, text: String, frage: bool)
                  -> Result<i64, String> {
        let knoepfe: Vec<String> = if frage { vec!["Ja".into(), "Nein".into()] } else { vec!["OK".into()] };
        self.dialog_frei(g, titel, text, knoepfe, None)
    }

    /// Der Dialog mit BELIEBIGEN Knoepfen (`GUI_DIALOG(t, x, "Speichern|Verwerfen|Abbrechen")`)
    /// und wahlweise einem Eingabefeld (`GUI_PROMPT`). Antwort = Nummer des
    /// Knopfs, 1-basiert. Enter drueckt den ERSTEN, ESC und das Schliessen-
    /// Kreuz den LETZTEN -- die Konvention jedes Systemdialogs: Abbrechen
    /// steht rechts. Bei nur einem Knopf ist der beides.
    pub fn dialog_frei(&mut self, g: &Graphics, titel: String, text: String,
                       knoepfe: Vec<String>, eingabe: Option<String>) -> Result<i64, String> {
        if knoepfe.is_empty() || knoepfe.len() > 5 {
            return Err("GUI_DIALOG: ein bis fuenf Knoepfe, getrennt mit |".into());
        }
        if knoepfe.iter().any(|k| k.trim().is_empty()) {
            return Err("GUI_DIALOG: ein Knopf ohne Beschriftung".into());
        }
        // In LOGISCHEN Einheiten rechnen -- `new_window`/`add_widget`
        // skalieren selbst, und zweimal waere zweimal zu viel.
        let zeilen: Vec<String> = text.split('\n').map(|z| z.to_string()).collect();
        let zh = self.unsk(self.ctext_height(g)) + 6;
        let pad = 16;
        // Knopfbreite nach der laengsten Beschriftung -- "Verwerfen" passt
        // nicht in 84 Pixel.
        let bw = knoepfe.iter().map(|k| self.unsk(self.ctext_width(g, k)) + 24).max().unwrap_or(84).max(84);
        let n = knoepfe.len() as i32;
        let gesamt = n * bw + (n - 1) * 8;
        let breiteste = zeilen.iter()
            .map(|z| self.unsk(self.ctext_width(g, z)))
            .max().unwrap_or(0)
            .max(self.unsk(self.ctext_width(g, &titel)) + 30)
            .max(gesamt);
        let ww = (breiteste + 2 * pad + 8).clamp(220, 640);
        let feld_h = if eingabe.is_some() { 28 + 10 } else { 0 };
        // Widget-Koordinaten sind RELATIV zum Inhaltsbereich, die Fensterhoehe
        // ist es nicht -- die Titelleiste kommt oben drauf. Ohne diesen
        // Zuschlag rutschte die Knopfreihe unter den unteren Rand.
        let inhalt_h = 14 + zeilen.len() as i32 * zh + feld_h + 12 + 26 + 12;
        let wh = self.m_roh("title_h") + inhalt_h;
        let (sw, sh) = (self.unsk(g.screen_width() as i32), self.unsk(g.screen_height() as i32));
        let h = self.new_window(titel, (sw - ww) / 2, (sh - wh) / 2, ww, wh);
        let wi = h as usize;
        for (i, z) in zeilen.iter().enumerate() {
            self.label(h, z.clone(), pad, 14 + i as i32 * zh, None)?;
        }
        let mut feld: Option<usize> = None;
        if let Some(vorgabe) = eingabe {
            let fy = 14 + zeilen.len() as i32 * zh + 4;
            let fh = self.textinput(h, pad, fy, ww - 2 * pad, 28, String::new())?;
            self.set_text(fh, vorgabe)?;
            // Alles markiert: das erste Tippen ersetzt die Vorgabe.
            let (fwi, fi) = Self::dec_widget(fh);
            self.windows[fwi].widgets[fi].sel_anchor = 0;
            feld = Some(fi);
        }
        // Knopfreihe unten rechts -- wie in jedem Systemdialog.
        let by = inhalt_h - 12 - 26;
        let bx0 = ww - pad - gesamt;
        let erster = self.windows[wi].widgets.len();
        for (k, kn) in knoepfe.iter().enumerate() {
            self.button(h, kn.clone(), bx0 + k as i32 * (bw + 8), by, bw, 26)?;
        }
        let letzter = self.windows[wi].widgets.len() - 1;
        {
            let win = &mut self.windows[wi];
            win.dlg = true;
            win.movable = true;
            win.closable = n > 1;   // Schliessen = der letzte Knopf; eine Meldung hat nur OK
            win.default_btn = erster as i32;
            win.cancel_btn = letzter as i32;
        }
        self.modal_setzen(wi);
        self.bring_to_front(wi);
        self.focus_window = Some(wi);
        // Fokus auf das Eingabefeld, sonst auf den ersten Knopf: der Dialog
        // soll sofort mit der Tastatur zu bedienen sein.
        self.focus_widget = Some((wi, feld.unwrap_or(erster)));
        Ok(h)
    }

    /// GUI_DIALOG_TEXT(dlg) -- was in das Eingabefeld eines GUI_PROMPT getippt
    /// wurde. Auch nach der Antwort lesbar: das Fenster bleibt als Tombstone.
    pub fn dialog_text(&self, h: i64) -> Result<String, String> {
        let w = self.windows.get(h as usize).filter(|_| h >= 0)
            .ok_or("GUI_DIALOG_TEXT: ungueltiges GUI_WINDOW-Handle")?;
        if !w.dlg { return Err("GUI_DIALOG_TEXT: das Fenster ist kein Dialog (GUI_PROMPT)".into()); }
        w.widgets.iter().find(|x| x.kind == Kind::TextInput).map(|x| x.text.clone())
            .ok_or_else(|| "GUI_DIALOG_TEXT: dieser Dialog hat kein Eingabefeld (nur GUI_PROMPT)".into())
    }

    /// GUI_WINDOW_MODAL(win, an) -- ein EIGENES Fenster modal schalten: alles
    /// andere nimmt keine Eingabe an und liegt hinter dem Schleier, bis das
    /// Fenster wieder freigegeben, ausgeblendet oder zerstoert wird.
    pub fn window_modal(&mut self, win: i64, an: bool) -> Result<(), String> {
        let wi = win as usize;
        if win < 0 || wi >= self.windows.len() || !self.windows[wi].alive {
            return Err(format!("GUI_WINDOW_MODAL: ungueltiges Fenster-Handle {}", win));
        }
        if an {
            if self.windows[wi].dlg { return Err("GUI_WINDOW_MODAL: ein Dialog ist schon modal".into()); }
            self.modal_setzen(wi);
            self.bring_to_front(wi);
            self.focus_window = Some(wi);
        } else if self.modal == Some(wi) {
            self.modal_loesen();
        } else {
            self.modal_vorher.retain(|&m| m != wi);
        }
        Ok(())
    }

    /// Ein Fenster wird modal; was es vorher war, kommt auf den Stapel.
    fn modal_setzen(&mut self, wi: usize) {
        if let Some(m) = self.modal {
            if m != wi { self.modal_vorher.push(m); }
        }
        self.modal_vorher.retain(|&m| m != wi);
        self.modal = Some(wi);
        if !self.modal_fokus.iter().any(|(m, _)| *m == wi) {
            self.modal_fokus.push((wi, self.focus_widget));
        }
        if self.focus_widget.map_or(false, |(w, _)| w != wi) { self.focus_widget = None; }
    }

    /// Das modale Fenster geht: das vorige, das noch steht, ist es wieder,
    /// und der Fokus, den das gehende vorfand, kommt zurueck.
    fn modal_loesen(&mut self) {
        let geht = self.modal.take();
        while let Some(m) = self.modal_vorher.pop() {
            if self.windows.get(m).map(|w| w.alive && w.visible).unwrap_or(false) {
                self.modal = Some(m);
                self.focus_window = Some(m);
                break;
            }
        }
        if let Some(g) = geht {
            if let Some(k) = self.modal_fokus.iter().rposition(|(m, _)| *m == g) {
                let (_, vorher) = self.modal_fokus.remove(k);
                let lebt = vorher.map_or(false, |(w, i)| self.windows.get(w)
                    .map_or(false, |win| win.alive && win.visible && win.widgets.get(i).map_or(false, |x| x.alive && x.visible)));
                let erlaubt = vorher.map_or(false, |(w, _)| self.modal.map_or(true, |m| m == w));
                if self.focus_widget.map_or(true, |(w, _)| w == g || !self.windows.get(w).map_or(false, |x| x.alive)) {
                    self.focus_widget = if lebt && erlaubt { vorher } else { None };
                }
            }
        }
    }

    /// Antwort eines Dialogs: 0 = noch offen, 1 = OK/Ja, 2 = Abbrechen/Nein.
    /// Gilt GENAU EIN BILD -- wie `GUI_CLICKED`.
    ///
    /// Ein NEGATIVER Handle ist "gerade kein Dialog" und liefert 0. Ohne
    /// diesen Ausweg gaebe es keinen freien Wert dafuer: Fenster-Handles
    /// zaehlen ab 0, und ausgerechnet die 0 ist das ERSTE Fenster des
    /// Programms -- ein `frage = 0` als "keiner" zeigte also auf das
    /// Hauptfenster. Merkt man sich den offenen Dialog in einer Variablen,
    /// setzt man sie auf -1, wenn keiner offen ist.
    pub fn answer(&self, h: i64) -> Result<i64, String> {
        if h < 0 { return Ok(0); }
        let w = self.windows.get(h as usize)
            .ok_or("GUI_ANSWER: ungueltiges GUI_WINDOW-Handle")?;
        if !w.dlg { return Err("GUI_ANSWER: das Fenster ist kein Dialog (GUI_DIALOG/GUI_PROMPT)".into()); }
        Ok(w.answer as i64)
    }

    /// Steht gerade ein modaler Dialog? Fuer Programme, die ESC selbst
    /// belegen und nicht beenden wollen, waehrend eine Frage offen ist.
    pub fn modal_offen(&self) -> bool { self.modal.is_some() }

    /// Farbwaehler: Saettigungs-/Helligkeitsfeld links, Farbton-Streifen
    /// rechts. Der gewaehlte Wert kommt mit `GUI_PICKED_COLOR` heraus.
    pub fn colorpicker(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32)
                       -> Result<i64, String> {
        let mut wd = Self::blank(Kind::ColorPicker, x, y, w.max(60), h.max(50));
        wd.hsv = [30.0, 0.8, 1.0];
        self.add_widget(win, "GUI_COLORPICKER", wd)
    }

    /// Datumswaehler: Monatsgitter mit Blaetterpfeilen. Der Wert kommt als
    /// `YYYY-MM-DD` heraus -- dasselbe Format wie `DATE$()`.
    pub fn datepicker(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32)
                      -> Result<i64, String> {
        let mut wd = Self::blank(Kind::DatePicker, x, y, w.max(140), h.max(120));
        let (j, m, t, _, _, _) = crate::builtins::local_datetime();
        wd.datum = [j as i32, m as i32, t as i32];
        self.add_widget(win, "GUI_DATEPICKER", wd)
    }

    /// Datumsfeld: zeigt das Datum und klappt beim Klick (Leertaste, F4,
    /// Alt+Ab) den Monat auf -- ein Tag uebernimmt und klappt zu, ESC klappt
    /// nur zu, die Pfeile verstellen es auch zugeklappt. Sonst ein
    /// GUI_DATEPICKER: GUI_DATE, GUI_SET_DATE, GUI_DATE_RANGE gelten.
    pub fn datefield(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32)
                     -> Result<i64, String> {
        let mut wd = Self::blank(Kind::DatePicker, x, y, w.max(90), h.max(20));
        let (j, m, t, _, _, _) = crate::builtins::local_datetime();
        wd.datum = [j as i32, m as i32, t as i32];
        wd.kompakt = true;
        self.add_widget(win, "GUI_DATEFIELD", wd)
    }

    pub fn picked_color(&self, h: i64) -> Result<i64, String> {
        let w = self.wdg(h, "GUI_PICKED_COLOR")?;
        if w.kind != Kind::ColorPicker {
            return Err("GUI_PICKED_COLOR: das Widget ist kein GUI_COLORPICKER".into());
        }
        let rgb = crate::farbraum::hsv_zu_rgb(w.hsv[0], w.hsv[1], w.hsv[2]);
        // Ohne Alpha-Streifen bleibt es bei 0xRRGGBB -- so, wie es Programme
        // bisher bekommen haben.
        Ok(if w.alpha_an { (w.alpha.clamp(1, 255) as i64) << 24 | rgb } else { rgb })
    }

    pub fn set_picked_color(&mut self, h: i64, farbe: i64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_PICKED_COLOR")?;
        if w.kind != Kind::ColorPicker {
            return Err("GUI_SET_PICKED_COLOR: das Widget ist kein GUI_COLORPICKER".into());
        }
        let (ht, s, v) = crate::farbraum::rgb_zu_hsv(farbe & 0xFF_FFFF);
        // Bei Grau/Schwarz laesst die Umrechnung den Ton offen -- dann den
        // bisherigen behalten, sonst springt der Zeiger bei jedem Setzen.
        let ton = if s <= f32::EPSILON || v <= f32::EPSILON { w.hsv[0] } else { ht };
        w.hsv = [ton, s, v];
        let a8 = (farbe >> 24) & 0xFF;
        // Oberstes Byte 0 heisst deckend -- also 255, nicht 0.
        if w.alpha_an { w.alpha = if a8 == 0 { 255 } else { a8 as i32 }; }
        Ok(())
    }

    /// Einstellungen des Farbwaehlers (GUI_COLORPICKER_SET).
    pub fn colorpicker_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_COLORPICKER_SET")?;
        if w.kind != Kind::ColorPicker {
            return Err("GUI_COLORPICKER_SET: das Widget ist kein GUI_COLORPICKER".into());
        }
        match key.to_lowercase().as_str() {
            "alpha" | "deckkraft" => w.alpha_an = wert as i32 != 0,
            other => return Err(std::format!(
                "GUI_COLORPICKER_SET: '{}' unbekannt -- moeglich ist alpha", other)),
        }
        Ok(())
    }

    /// Einstellungen des Datumswaehlers (GUI_DATEPICKER_SET).
    pub fn datepicker_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_DATEPICKER_SET")?;
        if w.kind != Kind::DatePicker {
            return Err("GUI_DATEPICKER_SET: das Widget ist kein GUI_DATEPICKER".into());
        }
        match key.to_lowercase().as_str() {
            // 0 = Montag ... 6 = Sonntag. In Deutschland beginnt die Woche am
            // Montag, in den USA am Sonntag -- eine feste Wahl waere fuer die
            // eine Haelfte der Welt falsch.
            "wochenbeginn" | "week_start" => w.wochenbeginn = (wert as i32).clamp(0, 6),
            other => return Err(std::format!(
                "GUI_DATEPICKER_SET: '{}' unbekannt -- moeglich ist wochenbeginn", other)),
        }
        Ok(())
    }

    /// Erlaubter Bereich des Datumswaehlers (GUI_DATE_RANGE). Ein leerer Text
    /// hebt die jeweilige Grenze auf.
    ///
    /// Ein gesetztes Datum ausserhalb wird sofort hereingezogen -- sonst
    /// stuende im Feld ein Wert, den der Waehler selbst nicht mehr zulaesst.
    pub fn date_range(&mut self, h: i64, von: &str, bis: &str) -> Result<(), String> {
        let lies = |t: &str, was: &str| -> Result<Option<[i32; 3]>, String> {
            if t.trim().is_empty() { return Ok(None); }
            crate::kalender::parse(t).map(|(j, m, d)| Some([j, m, d]))
                .ok_or_else(|| std::format!(
                    "GUI_DATE_RANGE: {} '{}' ist kein Datum -- erwartet JJJJ-MM-TT", was, t))
        };
        let (a, b) = (lies(von, "von")?, lies(bis, "bis")?);
        if let (Some(x), Some(y)) = (a, b) {
            if x > y {
                return Err("GUI_DATE_RANGE: 'von' liegt nach 'bis'".into());
            }
        }
        let w = self.wdg_mut(h, "GUI_DATE_RANGE")?;
        if w.kind != Kind::DatePicker {
            return Err("GUI_DATE_RANGE: das Widget ist kein GUI_DATEPICKER".into());
        }
        w.datum_min = a;
        w.datum_max = b;
        if let Some(x) = a { if w.datum < x { w.datum = x; } }
        if let Some(y) = b { if w.datum > y { w.datum = y; } }
        Ok(())
    }

    /// Liegt das Datum im erlaubten Bereich?
    fn dp_erlaubt(wd: &Widget, d: [i32; 3]) -> bool {
        if let Some(x) = wd.datum_min { if d < x { return false; } }
        if let Some(y) = wd.datum_max { if d > y { return false; } }
        true
    }

    pub fn date(&self, h: i64) -> Result<String, String> {
        let w = self.wdg(h, "GUI_DATE")?;
        if w.kind != Kind::DatePicker {
            return Err("GUI_DATE: das Widget ist kein GUI_DATEPICKER".into());
        }
        Ok(crate::kalender::format(w.datum[0], w.datum[1], w.datum[2]))
    }

    pub fn set_date(&mut self, h: i64, datum: &str) -> Result<(), String> {
        let (j, m, t) = crate::kalender::parse(datum).ok_or_else(|| format!(
            "GUI_SET_DATE: '{}' ist kein Datum -- erwartet JJJJ-MM-TT (wie DATE$())", datum))?;
        let w = self.wdg_mut(h, "GUI_SET_DATE")?;
        if w.kind != Kind::DatePicker {
            return Err("GUI_SET_DATE: das Widget ist kein GUI_DATEPICKER".into());
        }
        w.datum = [j, m, t];
        Ok(())
    }

    /// Farbige Abschnitte eines Textfelds setzen (GUI_TEXTAREA_SPANS).
    ///
    /// `starts`/`laengen` zaehlen ZEICHEN, nicht Bytes -- sonst laege alles
    /// hinter dem ersten Umlaut daneben. Ueberlappende oder unsortierte
    /// Angaben sind erlaubt; gezeichnet wird der ZULETZT genannte, so kann ein
    /// Programm eine Grundfaerbung mit Einzelfaellen ueberschreiben.
    pub fn textarea_spans(&mut self, h: i64, starts: Vec<i64>, laengen: Vec<i64>,
                          farben: Vec<i64>) -> Result<(), String> {
        if starts.len() != laengen.len() || starts.len() != farben.len() {
            return Err(format!(
                "GUI_TEXTAREA_SPANS: die drei Listen muessen gleich lang sein \
                 ({} Starts, {} Laengen, {} Farben)",
                starts.len(), laengen.len(), farben.len()));
        }
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_SPANS")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_SPANS: das Widget ist kein GUI_TEXTAREA".into());
        }
        wd.spans = starts.iter().zip(&laengen).zip(&farben)
            .filter(|((_, &l), _)| l > 0)
            .map(|((&s, &l), &c)| (s.max(0) as u32, l as u32, c))
            .collect();
        wd.spans.sort_by_key(|&(s, _, _)| s);
        Ok(())
    }

    /// Wellenlinien unter Zeichen-Abschnitten (ersetzt alle bisherigen; leere
    /// Listen loeschen). Gezeichnet wird unter dem Text, gemessen wie die
    /// Farbabschnitte ueber die Breite des Vorspanns -- sonst saesse die Linie
    /// hinter dem ersten breiten Zeichen daneben.
    pub fn textarea_squiggles(&mut self, h: i64, starts: Vec<i64>, laengen: Vec<i64>,
                              farben: Vec<i64>) -> Result<(), String> {
        if starts.len() != laengen.len() || starts.len() != farben.len() {
            return Err(format!(
                "GUI_TEXTAREA_SQUIGGLES: die drei Listen muessen gleich lang sein \
                 ({} Starts, {} Laengen, {} Farben)",
                starts.len(), laengen.len(), farben.len()));
        }
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_SQUIGGLES")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_SQUIGGLES: das Widget ist kein GUI_TEXTAREA".into());
        }
        wd.wellen = starts.iter().zip(&laengen).zip(&farben)
            .filter(|((&s, &l), _)| s >= 0 && l >= 0)
            .map(|((&s, &l), &c)| (s as u32, l as u32, c))
            .collect();
        wd.wellen.sort_by_key(|&(s, _, _)| s);
        Ok(())
    }

    /// GUI_TEXTAREA_HINTS(ta, zeilen, texte [, farbe]): Text hinter dem Ende
    /// der Zeilen (ab 1), ersetzt alle bisherigen; leere Listen loeschen.
    /// GUI_TEXTAREA_HINT$(ta, zeile): der Hinweis hinter einer Zeile (ab 1),
    /// "" ohne. Die Zeilen wandern beim Tippen mit -- gefragt wird der
    /// Stand nach dem Nachziehen, so wie er gezeichnet wird.
    pub fn textarea_hint(&mut self, h: i64, zeile: i64) -> Result<String, String> {
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_HINT$")?;
        if wd.kind != Kind::TextArea { return Err("GUI_TEXTAREA_HINT$: das Widget ist kein GUI_TEXTAREA".into()); }
        marken_abgleichen(wd);
        if zeile < 1 { return Ok(String::new()); }
        Ok(wd.hinweise.iter().find(|m| m.0 as i64 == zeile - 1).map(|m| m.1.clone()).unwrap_or_default())
    }
    /// GUI_TEXTAREA_HINT_COUNT(ta): wie viele Zeilen einen Hinweis tragen.
    pub fn textarea_hint_count(&self, h: i64) -> Result<i64, String> {
        let wd = self.wdg(h, "GUI_TEXTAREA_HINT_COUNT")?;
        if wd.kind != Kind::TextArea { return Err("GUI_TEXTAREA_HINT_COUNT: das Widget ist kein GUI_TEXTAREA".into()); }
        Ok(wd.hinweise.len() as i64)
    }
    pub fn textarea_hints(&mut self, h: i64, zeilen: Vec<i64>, texte: Vec<String>, farbe: i64, farben: Vec<i64>) -> Result<(), String> {
        if zeilen.len() != texte.len() {
            return Err(format!(
                "GUI_TEXTAREA_HINTS: beide Listen muessen gleich lang sein ({} Zeilen, {} Texte)",
                zeilen.len(), texte.len()));
        }
        if !farben.is_empty() && farben.len() != zeilen.len() {
            return Err(format!(
                "GUI_TEXTAREA_HINTS: je Hinweis eine Farbe ({} Zeilen, {} Farben)",
                zeilen.len(), farben.len()));
        }
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_HINTS")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_HINTS: das Widget ist kein GUI_TEXTAREA".into());
        }
        let mut hin = Vec::new();
        let mut fa = Vec::new();
        for (i, (&z, t)) in zeilen.iter().zip(texte).enumerate() {
            if z < 1 || t.is_empty() { continue; }
            hin.push(((z - 1) as usize, t.replace('\n', " ")));
            if !farben.is_empty() { fa.push(farben[i]); }
        }
        wd.hinweise = hin;
        wd.hinweise_farben = fa;
        wd.hinweise_text = wd.text.clone();
        wd.hinweise_farbe = farbe;
        Ok(())
    }

    /// Marken je Zeile setzen (ersetzt alle bisherigen; leere Listen loeschen).
    pub fn textarea_marks(&mut self, h: i64, zeilen: Vec<i64>, farben: Vec<i64>) -> Result<(), String> {
        if zeilen.len() != farben.len() {
            return Err(format!(
                "GUI_TEXTAREA_MARKS: beide Listen muessen gleich lang sein ({} Zeilen, {} Farben)",
                zeilen.len(), farben.len()));
        }
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_MARKS")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_MARKS: das Widget ist kein GUI_TEXTAREA".into());
        }
        wd.marken = zeilen.iter().zip(&farben)
            .filter(|(&z, _)| z >= 1)
            .map(|(&z, &c)| ((z - 1) as usize, c))
            .collect();
        wd.marken_text = wd.text.clone();
        Ok(())
    }

    /// Zeilenfarben setzen (ersetzt alle; leere Listen loeschen). Die Farbe
    /// traegt ihre Deckkraft im obersten Byte; 0 (= deckend in der uebrigen
    /// gui) hiesse hier, den Text zu verdecken -- dann gilt 0x40.
    pub fn textarea_line_colors(&mut self, h: i64, zeilen: Vec<i64>, farben: Vec<i64>) -> Result<(), String> {
        if zeilen.len() != farben.len() {
            return Err(format!(
                "GUI_TEXTAREA_LINE_COLORS: beide Listen muessen gleich lang sein ({} Zeilen, {} Farben)",
                zeilen.len(), farben.len()));
        }
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_LINE_COLORS")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_LINE_COLORS: das Widget ist kein GUI_TEXTAREA".into());
        }
        wd.zeilenfarben = zeilen.iter().zip(&farben)
            .filter(|(&z, _)| z >= 1)
            .map(|(&z, &c)| ((z - 1) as usize, zeilenfarbe_deckkraft(c)))
            .collect();
        wd.zeilenfarben_text = wd.text.clone();
        Ok(())
    }

    /// Die Zeilen der Zeilenfarben, wie sie JETZT stehen, in der Reihenfolge
    /// des Setzens (GUI_TEXTAREA_LINE_COLORS_GET) -- wer zu jeder Zeile noch
    /// etwas anderes zeigt (einen Hinweis), findet sie so nach dem Tippen.
    pub fn textarea_line_colors_get(&mut self, h: i64) -> Result<Vec<i64>, String> {
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_LINE_COLORS_GET")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_LINE_COLORS_GET: das Widget ist kein GUI_TEXTAREA".into());
        }
        marken_abgleichen(wd);
        Ok(wd.zeilenfarben.iter().map(|&(z, _)| z as i64 + 1).collect())
    }

    /// Die Zeilen der Marken, wie sie JETZT stehen (GUI_TEXTAREA_MARKS_GET):
    /// 1-basiert, in der Reihenfolge, in der sie gesetzt wurden. Hat der
    /// Nutzer darueber Zeilen eingefuegt oder geloescht, sind sie mitgewandert
    /// -- so erfaehrt ein Programm, wohin sein Haltepunkt gerutscht ist.
    pub fn textarea_marks_get(&mut self, h: i64) -> Result<Vec<i64>, String> {
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_MARKS_GET")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_MARKS_GET: das Widget ist kein GUI_TEXTAREA".into());
        }
        marken_abgleichen(wd);
        Ok(wd.marken.iter().map(|&(z, _)| z as i64 + 1).collect())
    }

    /// Welche Bloecke sich falten lassen (GUI_TEXTAREA_FOLDABLE).
    ///
    /// Zwei gleich lange Listen mit 1-basierten Zeilen: Kopfzeile und letzte
    /// Zeile des Blocks. Welche Zeilen einen Block bilden, weiss nur der
    /// Aufrufer -- die IDE fragt dafuer `CODE_SYMBOLS$`; die Laufzeit kennt
    /// hier keine Sprache und zaehlt keine Einrueckung. Ein neuer Satz
    /// ersetzt den alten; was zugeklappt ist, bleibt es (sonst spraenge nach
    /// jeder Aenderung alles wieder auf).
    pub fn textarea_foldable(&mut self, h: i64, von: Vec<i64>, bis: Vec<i64>) -> Result<(), String> {
        if von.len() != bis.len() {
            return Err(format!(
                "GUI_TEXTAREA_FOLDABLE: beide Listen muessen gleich lang sein ({} Anfaenge, {} Enden)",
                von.len(), bis.len()));
        }
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_FOLDABLE")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_FOLDABLE: das Widget ist kein GUI_TEXTAREA".into());
        }
        wd.faltbar = von.iter().zip(&bis)
            .filter(|(&v, &b)| v >= 1 && b > v)
            .map(|(&v, &b)| (v as usize, b as usize))
            .collect();
        wd.faltbar.sort();
        Ok(())
    }

    /// Einen Block zuklappen oder aufklappen (GUI_TEXTAREA_FOLD).
    ///
    /// `zeile` ist irgendeine Zeile des Blocks -- gefaltet wird der ENGSTE
    /// faltbare Block, der sie enthaelt. Ohne `an` wird umgeschaltet.
    /// Liefert TRUE, wenn danach zugeklappt ist.
    pub fn textarea_fold(&mut self, h: i64, zeile: i64, an: Option<bool>) -> Result<bool, String> {
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_FOLD")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_FOLD: das Widget ist kein GUI_TEXTAREA".into());
        }
        let z = zeile.max(1) as usize;
        // Der engste Block gewinnt: eine Methode in einer Klasse soll sich
        // einzeln zuklappen lassen, ohne die ganze Klasse mitzunehmen.
        let Some(&(v, b)) = wd.faltbar.iter()
            .filter(|&&(v, b)| z >= v && z <= b)
            .min_by_key(|&&(v, b)| b - v) else { return Ok(false); };
        let vorhanden = wd.gefaltet.iter().position(|&(gv, _)| gv == v);
        let zu = an.unwrap_or(vorhanden.is_none());
        match (zu, vorhanden) {
            (true, None) => {
                wd.gefaltet.push((v, b));
                // Steht die Marke im Block, waere sie danach unsichtbar --
                // sie kommt auf die Kopfzeile.
                let chars: Vec<char> = wd.text.chars().collect();
                let starts = Self::line_starts(&chars);
                if Self::zeile_verborgen(wd, starts.iter()
                        .rposition(|&s| s <= wd.caret.max(0) as usize).unwrap_or(0) + 1) {
                    let c = starts.get(v - 1).copied().unwrap_or(0) as i32;
                    wd.caret = c; wd.sel_anchor = c;
                }
            }
            (false, Some(k)) => { wd.gefaltet.remove(k); }
            _ => {}
        }
        Ok(zu)
    }

    /// Alles auf- oder zuklappen (GUI_TEXTAREA_FOLD_ALL).
    pub fn textarea_fold_all(&mut self, h: i64, zu: bool) -> Result<i64, String> {
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_FOLD_ALL")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_FOLD_ALL: das Widget ist kein GUI_TEXTAREA".into());
        }
        if !zu { wd.gefaltet.clear(); return Ok(0); }
        // Nur die AEUSSEREN Bloecke: ein innerer waere unter dem aeusseren
        // ohnehin verborgen und stuende beim Aufklappen ueberraschend zu.
        let mut aussen: Vec<(usize, usize)> = Vec::new();
        for &(v, b) in wd.faltbar.iter() {
            if !aussen.iter().any(|&(av, ab)| v > av && v <= ab) { aussen.push((v, b)); }
        }
        wd.gefaltet = aussen;
        Self::falte_am_caret_oeffnen(wd);
        Ok(wd.gefaltet.len() as i64)
    }

    /// Ist die Zeile in einem zugeklappten Block verborgen (GUI_TEXTAREA_FOLDED)?
    pub fn textarea_folded(&self, h: i64, zeile: i64) -> Result<bool, String> {
        let wd = self.wdg(h, "GUI_TEXTAREA_FOLDED")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_FOLDED: das Widget ist kein GUI_TEXTAREA".into());
        }
        Ok(Self::zeile_verborgen(wd, zeile.max(1) as usize))
    }

    /// Die Kopfzeilen der zugeklappten Bloecke (GUI_TEXTAREA_FOLDS).
    pub fn textarea_folds(&self, h: i64) -> Result<Vec<i64>, String> {
        let wd = self.wdg(h, "GUI_TEXTAREA_FOLDS")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_FOLDS: das Widget ist kein GUI_TEXTAREA".into());
        }
        let mut z: Vec<i64> = wd.gefaltet.iter().map(|&(v, _)| v as i64).collect();
        z.sort();
        Ok(z)
    }

    /// Farbfelder im Text (GUI_TEXTAREA_SWATCHES).
    ///
    /// Drei gleich lange Listen: Anfang und Laenge des Stueckes in ZEICHEN
    /// (wie bei den Abschnitten) und die Farbe. Gezeichnet wird ein kleines
    /// Quadrat HINTER dem Stueck -- der Text bleibt, wie er ist; nichts
    /// wird verdeckt oder verschoben.
    ///
    /// **Welche Stelle im Text eine Farbe meint, weiss nur der Aufrufer.**
    /// Die IDE sucht dafuer `&H`-Literale; ein anderes Programm faende sie
    /// woanders. Dieselbe Aufteilung wie bei Faltung und Einrueckung.
    pub fn textarea_swatches(&mut self, h: i64, starts: Vec<i64>, laengen: Vec<i64>,
                             farben: Vec<i64>) -> Result<(), String> {
        if starts.len() != laengen.len() || starts.len() != farben.len() {
            return Err(format!(
                "GUI_TEXTAREA_SWATCHES: die drei Listen muessen gleich lang sein \
                 ({} Starts, {} Laengen, {} Farben)",
                starts.len(), laengen.len(), farben.len()));
        }
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_SWATCHES")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_SWATCHES: das Widget ist kein GUI_TEXTAREA".into());
        }
        wd.farbfelder = starts.iter().zip(&laengen).zip(&farben)
            .filter(|((_, &l), _)| l > 0)
            .map(|((&s, &l), &c)| (s.max(0) as u32, l as u32, c))
            .collect();
        Ok(())
    }

    /// Welches Farbfeld wurde in diesem Bild angeklickt (GUI_TEXTAREA_SWATCH_CLICKED)?
    ///
    /// Die Nummer in der Liste von `GUI_TEXTAREA_SWATCHES`, -1 = keins.
    /// Gilt genau ein Bild lang, wie `GUI_CLICKED`.
    pub fn textarea_swatch_clicked(&self, h: i64) -> Result<i64, String> {
        Ok(self.ta_wdg(h, "GUI_TEXTAREA_SWATCH_CLICKED")?.farbfeld_klick as i64)
    }

    /// Welcher Hinweis wurde in diesem Bild angeklickt (GUI_TEXTAREA_HINT_CLICKED)?
    /// Die Zeile (ab 1), 0 = keiner. Nur mit "hinweise_klickbar".
    pub fn textarea_hint_clicked(&self, h: i64) -> Result<i64, String> {
        Ok(self.ta_wdg(h, "GUI_TEXTAREA_HINT_CLICKED")?.hinweis_klick)
    }

    /// Welche Zeile wurde in diesem Bild in der Nummernspalte angeklickt
    /// (GUI_TEXTAREA_GUTTER_CLICKED)? `taste` 0 = links, 1 = rechts.
    ///
    /// Die Zeile zaehlt ab 1 (logische Zeile, wie GUI_TEXTAREA_CURSOR), 0 =
    /// kein Klick dieser Taste. Gilt genau ein Bild lang, wie `GUI_CLICKED`.
    pub fn textarea_gutter_clicked(&self, h: i64, taste: i64) -> Result<i64, String> {
        let (z, t) = self.ta_wdg(h, "GUI_TEXTAREA_GUTTER_CLICKED")?.rand_klick;
        Ok(if t as i64 == taste { z as i64 } else { 0 })
    }

    /// Die logische Zeile (ab 1) unter einem Punkt der Nummernspalte, `None`
    /// ausserhalb -- auch auf dem Faltpfeil (der klappt, siehe edit_textarea)
    /// und unter der letzten Zeile. Dieselbe Zeilenrechnung wie ein Klick in
    /// den Text (`ta_rows`, `scroll`), sonst saesse ein Haltepunkt eine Zeile
    /// daneben, sobald gefaltet oder umbrochen ist.
    fn ta_rand_zeile(&self, g: &Graphics, wi: usize, i: usize, mx: i32, my: i32) -> Option<i32> {
        let wd = &self.windows[wi].widgets[i];
        let chars: Vec<char> = wd.text.chars().collect();
        let starts = Self::line_starts(&chars);
        let gut = self.ta_gutter(g, wd, starts.len());
        if gut == 0 { return None; }
        let (ax, ay, _, fh) = self.abs_rect(wi, wd);
        let pad = 5;
        if mx < ax || mx >= ax + pad + gut - self.sk(4) { return None; }
        if my < ay + pad || my >= ay + fh - pad { return None; }
        if let Some((fx, fbw)) = self.ta_falt_spalte(g, wd, ax, starts.len()) {
            if mx >= fx && mx < fx + fbw { return None; }
        }
        let lh = self.ta_line_h(g, wd);
        let rows = self.ta_rows(g, wd, &chars, &starts, self.ta_breite(g, wd, starts.len()));
        // Im angehefteten Kopf: der Klick meint den Kopf, nicht die Zeile
        // darunter (die ist verdeckt) -- er springt, siehe edit_textarea.
        let sicht = ((fh - 2 * pad) / lh).max(1);
        if my < ay + pad + Self::ta_koepfe(wd, &rows, sicht, wd.scroll).len() as i32 * lh { return None; }
        let row = wd.scroll + (my - ay - pad) / lh;
        if row < 0 || row as usize >= rows.len() { return None; }
        Some(rows[row as usize].0 as i32 + 1)
    }

    /// Abkuerzungen, die der Tabulator meldet (GUI_TEXTAREA_ABBREV).
    ///
    /// Steht links der Schreibmarke eines dieser Woerter, rueckt der
    /// Tabulator nicht ein, sondern meldet die Nummer ueber
    /// `GUI_TEXTAREA_ABBREV_HIT` -- genau ein Bild lang. Gross- und
    /// Kleinschreibung zaehlen nicht.
    ///
    /// Gilt nur in einem Feld, das den Tabulator ohnehin hat
    /// (`GUI_TEXTAREA_SET(ta, "tab_fuegt_ein", 1)`) -- sonst gehoert die
    /// Taste dem Fokus-Wechsel, und ein Wort ohne Treffer waere eine tote
    /// Taste.
    pub fn textarea_abbrev(&mut self, h: i64, woerter: Vec<String>) -> Result<(), String> {
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_ABBREV")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_ABBREV: das Widget ist kein GUI_TEXTAREA".into());
        }
        wd.abkuerzungen = woerter.into_iter().filter(|w| !w.is_empty()).collect();
        Ok(())
    }

    /// Welche Abkuerzung der Tabulator in diesem Bild getroffen hat, -1 = keine.
    pub fn textarea_abbrev_hit(&self, h: i64) -> Result<i64, String> {
        Ok(self.ta_wdg(h, "GUI_TEXTAREA_ABBREV_HIT")?.abk_treffer as i64)
    }

    /// Wurde der Tabulator in diesem Bild gedrueckt (GUI_TEXTAREA_TAB_HIT)?
    ///
    /// Nur mit `GUI_TEXTAREA_SET(ta, "tab_meldet", 1)`; ohne den Schalter ist
    /// es immer FALSE, und der Tabulator tut, was er vorher tat.
    pub fn textarea_tab_hit(&self, h: i64) -> Result<bool, String> {
        Ok(self.ta_wdg(h, "GUI_TEXTAREA_TAB_HIT")?.tab_treffer)
    }

    /// Woerter, die die Einrueckung steuern (GUI_TEXTAREA_INDENT_WORDS).
    ///
    /// Drei Listen, alle ohne Ruecksicht auf Gross/Klein:
    /// `anfang` -- die Zeile faengt damit an, die naechste rueckt ein
    /// (`SUB`, `FOR`, `WHILE`); `ende` -- die Zeile endet damit
    /// (`THEN`; `IF x THEN y = 1` endet nicht darauf und rueckt darum nicht
    /// ein); `aus` -- wird dieses Wort allein in eine Zeile getippt, rueckt
    /// sie selbst eine Stufe zurueck (`END`, `NEXT`, `ELSE`).
    ///
    /// Zwei Listen fuer "mehr" und nicht eine: `SUB` oeffnet am ANFANG,
    /// `THEN` am ENDE -- mit einer Liste ruckte auch ein `END SUB` die
    /// naechste Zeile ein. Welche Woerter das sind, weiss nur der Aufrufer;
    /// die Laufzeit kennt hier keine Sprache.
    pub fn textarea_indent_words(&mut self, h: i64, anfang: Vec<String>, ende: Vec<String>,
                                 aus: Vec<String>) -> Result<(), String> {
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_INDENT_WORDS")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_INDENT_WORDS: das Widget ist kein GUI_TEXTAREA".into());
        }
        let gross = |v: Vec<String>| v.into_iter().map(|s| s.trim().to_uppercase())
                                      .filter(|s| !s.is_empty()).collect();
        wd.einzug_anfang = gross(anfang);
        wd.einzug_ende = gross(ende);
        wd.einzug_aus = gross(aus);
        Ok(())
    }

    /// Das mitwachsende Geruest (GUI_TEXTAREA_CLOSE_WORDS).
    ///
    /// Zwei gleich lange Listen: `oeffner` sind Woerter, mit denen eine Zeile
    /// anfaengt, `schluesse` die Zeile, die der Umbruch darunter setzt
    /// (`IF` -> `END IF`, `FOR` -> `NEXT`). Der Abschluss kommt UNTER die
    /// neue Zeile, die Marke bleibt dazwischen stehen.
    ///
    /// Welche Woerter das sind, weiss wieder nur der Aufrufer -- wie bei
    /// GUI_TEXTAREA_INDENT_WORDS kennt die Laufzeit hier keine Sprache. Die
    /// Gross-/Kleinschreibung des Abschlusses bleibt, wie sie hereinkommt:
    /// sie wird GESCHRIEBEN und nicht verglichen.
    ///
    /// Zwei leere Listen schalten es ab.
    pub fn textarea_close_words(&mut self, h: i64, oeffner: Vec<String>,
                                schluesse: Vec<String>) -> Result<(), String> {
        if oeffner.len() != schluesse.len() {
            return Err("GUI_TEXTAREA_CLOSE_WORDS: beide Listen muessen gleich lang sein".into());
        }
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_CLOSE_WORDS")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_CLOSE_WORDS: das Widget ist kein GUI_TEXTAREA".into());
        }
        wd.schluss_oeffner = oeffner.iter().map(|s| s.trim().to_uppercase()).collect();
        wd.schluss_texte = schluesse;
        Ok(())
    }

    /// Eine weitere Schreibmarke setzen (GUI_TEXTAREA_ADD_CARET).
    ///
    /// Tippen, Enter, Ruecktaste, Entf, Tabulator und Einfuegen wirken dann
    /// an JEDER Marke; die Pfeile bewegen alle. Zwei Marken an derselben
    /// Stelle werden zu einer. Liefert die Zahl der Marken danach.
    pub fn textarea_add_caret(&mut self, h: i64, zeile: i64, spalte: i64) -> Result<i64, String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_ADD_CARET")?;
        let chars: Vec<char> = wd.text.chars().collect();
        let idx = Self::ta_index(&chars, zeile, spalte) as i32;
        let w = self.wdg_mut(h, "GUI_TEXTAREA_ADD_CARET")?;
        if idx != w.caret && !w.marken_zusatz.iter().any(|&(c, _)| c == idx) {
            w.marken_zusatz.push((idx, idx));
        }
        Ok(w.marken_zusatz.len() as i64 + 1)
    }

    /// Wie viele Schreibmarken das Feld gerade hat (GUI_TEXTAREA_CARETS).
    pub fn textarea_carets(&self, h: i64) -> Result<i64, String> {
        Ok(self.ta_wdg(h, "GUI_TEXTAREA_CARETS")?.marken_zusatz.len() as i64 + 1)
    }

    /// Zurueck auf eine Schreibmarke (GUI_TEXTAREA_CLEAR_CARETS).
    pub fn textarea_clear_carets(&mut self, h: i64) -> Result<(), String> {
        self.ta_wdg(h, "GUI_TEXTAREA_CLEAR_CARETS")?;
        self.wdg_mut(h, "GUI_TEXTAREA_CLEAR_CARETS")?.marken_zusatz.clear();
        Ok(())
    }

    /// Einstellungen eines Textbereichs (GUI_TEXTAREA_SET).
    ///
    /// EIN Setter mit Schluesselwort statt vier Builtins -- dasselbe Muster
    /// wie bei `chart` und der Tabelle. Ein unbekannter Schluessel zaehlt die
    /// gueltigen auf, statt still nichts zu tun.
    /// Ein Bild schwach hinter den Text legen (GUI_TEXTAREA_BACKGROUND).
    /// Es fuellt den Innenbereich eingepasst und rollt NICHT mit -- es ist
    /// Grund, kein Inhalt. `bild` < 0 nimmt es weg.
    pub fn textarea_background(&mut self, h: i64, bild: i64, deckkraft: i64) -> Result<(), String> {
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_BACKGROUND")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_BACKGROUND: erwartet einen Textbereich".into());
        }
        wd.hg_bild = if bild < 0 { -1 } else { bild };
        wd.hg_deckkraft = deckkraft.clamp(0, 255) as u8;
        Ok(())
    }
    pub fn textarea_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_SET")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_SET: das Widget ist kein GUI_TEXTAREA".into());
        }
        let n = wert as i32;
        match key.to_lowercase().as_str() {
            "zeilennummern" => wd.zeilennummern = n != 0,
            "aktive_zeile" => wd.aktive_zeile = n != 0,
            // Ohne diesen Schalter waere ein Code-Feld nicht benutzbar: TAB
            // wechselt sonst das Bedienelement, und man kann nicht einruecken.
            // Aus per Vorgabe, damit die Tastatur-Navigation ueberall sonst
            // bleibt, wie sie ist.
            "tab_fuegt_ein" => wd.tab_fuegt_ein = n != 0,
            "tabbreite" => wd.tabbreite = n.clamp(1, 16),
            // Umbruch an Wortgrenzen -- fuer Notizen und Briefe, nicht fuer
            // Code. Mit Umbruch gibt es keinen waagerechten Versatz mehr.
            "umbruch" | "wrap" => { wd.umbruch = n != 0; if wd.umbruch { wd.scroll_x = 0; } }
            // Inline-Formate: an = alles ungeformt, aus = die Formate fallen weg.
            "formatiert" | "formatted" => {
                wd.formatiert = n != 0;
                wd.stile = if wd.formatiert { vec![0; wd.text.chars().count()] } else { Vec::new() };
                wd.stile_text = if wd.formatiert { wd.text.clone() } else { String::new() };
                wd.tipp_stil = None;
            }
            // Eine neue Zeile faengt mit der Einrueckung der alten an. Das
            // allein ist sprachfrei; die Woerter, die eine Stufe mehr oder
            // weniger bedeuten, kommen ueber GUI_TEXTAREA_INDENT_WORDS.
            "auto_einzug" | "auto_indent" => wd.auto_einzug = n != 0,
            // Ein Strich je Einrueckungsstufe. Die Breite einer Stufe haengt
            // an `tabbreite` -- die Laufzeit misst sie an Leerzeichen, statt
            // sie zu raten.
            "einzugslinien" | "indent_guides" => wd.einzugslinien = n != 0,
            // Der Tabulator gehoert ganz dem Aufrufer: das Feld rueckt nicht
            // ein, sieht nicht nach Abkuerzungen, sondern MELDET ihn nur
            // (GUI_TEXTAREA_TAB_HIT). Fuer Felder, in denen der Tabulator
            // mehrere Bedeutungen hat und der Aufrufer die Reihenfolge kennt.
            "tab_meldet" | "tab_reports" => wd.tab_meldet = n != 0,
            // Mitlaufende Blockkoepfe, hoechstens n Zeilen. Welche Zeilen
            // einen Block bilden, sagt GUI_TEXTAREA_FOLDABLE -- dieselben
            // Bloecke, die sich falten lassen.
            "kopfzeilen" | "sticky_headers" => wd.kopfzeilen = n.clamp(0, 10) as i32,
            // Hinweise hinter dem Zeilenende nehmen Klicks an
            // (GUI_TEXTAREA_HINT_CLICKED) -- etwa "3 Aufrufe" hinter einer SUB.
            "hinweise_klickbar" | "hints_clickable" => wd.hinweise_klickbar = n != 0,
            other => return Err(format!(
                "GUI_TEXTAREA_SET: '{}' unbekannt -- moeglich sind zeilennummern, \
                 aktive_zeile, tab_fuegt_ein, tabbreite, umbruch, auto_einzug, \
                 einzugslinien, tab_meldet, kopfzeilen, hinweise_klickbar", other)),
        }
        Ok(())
    }

    /// Aufteilung des Farbwaehlers: (Feld, Ton-Streifen).
    ///
    /// EINE Quelle -- Zeichnen und Treffertest fragen beide hier. Getrennt
    /// gerechnet waere der Zeiger irgendwann neben dem Klick.
    fn cp_geom(&self, wdg: &Widget, ax: i32, ay: i32, w: i32, h: i32)
               -> ((i32, i32, i32, i32), (i32, i32, i32, i32), Option<(i32, i32, i32, i32)>) {
        let streifen = self.sk(18);
        let luecke = self.sk(6);
        let spalten = if wdg.alpha_an { 2 } else { 1 };
        let feld_w = (w - spalten * (streifen + luecke)).max(10);
        let ton = (ax + feld_w + luecke, ay, streifen, h);
        let deck = if wdg.alpha_an {
            Some((ton.0 + streifen + luecke, ay, streifen, h))
        } else { None };
        ((ax, ay, feld_w, h), ton, deck)
    }

    /// Aufteilung des Datumswaehlers: (Kopfhoehe, Zellbreite, Zellhoehe).
    ///
    /// Unter dem Kopf steht die Wochentagszeile, darunter SECHS Wochen --
    /// fest sechs und nicht "so viele wie noetig", damit das Gitter beim
    /// Blaettern nicht in der Hoehe springt.
    fn dp_geom(&self, w: i32, h: i32) -> (i32, i32, i32) {
        let kopf = self.sk(26);
        let zell_w = (w / 7).max(1);
        let zell_h = ((h - kopf - self.sk(18)) / 6).max(1);
        (kopf, zell_w, zell_h)
    }

    /// Platz fuer die Faltpfeile am rechten Rand der Nummernspalte (0 = keine).
    ///
    /// Sie kosten nur Platz, wenn es ueberhaupt faltbare Bloecke gibt --
    /// ein gewoehnlicher Textbereich sieht aus wie zuvor.
    fn ta_faltbreite(&self, wdg: &Widget) -> i32 {
        if wdg.faltbar.is_empty() { 0 } else { self.sk(11) }
    }

    /// Waagerechte Lage der Faltpfeile: (linke Kante, Breite).
    ///
    /// EINE Quelle fuer Zeichnen und Klick -- liefen sie auseinander, klappte
    /// ein Klick neben dem Pfeil den Block zu.
    fn ta_falt_spalte(&self, g: &Graphics, wdg: &Widget, ax: i32, zeilen: usize) -> Option<(i32, i32)> {
        let fw = self.ta_faltbreite(wdg);
        if fw == 0 { return None; }
        let gutter = self.ta_gutter(g, wdg, zeilen);
        if gutter == 0 { return None; }
        Some((ax + 5 + gutter - self.sk(4) - fw, fw))
    }

    /// Breite der Nummernspalte (0 = aus).
    ///
    /// EINE Quelle: Zeichnen, Treffertest und Schreibmarke fragen alle hier.
    /// Die Zeilenzahl kommt von aussen, weil die Aufrufer sie ohnehin schon
    /// ausgerechnet haben -- sie hier nochmal aus dem Text zu zaehlen waere
    /// bei einer grossen Datei ein Durchlauf pro Bild und Aufruf.
    fn ta_gutter(&self, g: &Graphics, wdg: &Widget, zeilen: usize) -> i32 {
        if !wdg.zeilennummern { return 0; }
        let stellen = zeilen.max(1).to_string().len();
        let muster = "9".repeat(stellen.max(2));
        self.wtext_width(g, wdg, &muster) + self.sk(12) + self.ta_faltbreite(wdg)
    }

    fn add_widget(&mut self, win: i64, fn_: &str, mut wdg: Widget) -> Result<i64, String> {
        let wi = win as usize;
        // Massstab VOR dem veraenderlichen Zugriff auf `windows` -- `sk` liest
        // `self`, und beides gleichzeitig laesst der Compiler nicht zu.
        let (sx, sy, sw, sh) = (self.sk(wdg.x), self.sk(wdg.y), self.sk(wdg.w), self.sk(wdg.h));
        let w = self.windows.get_mut(wi).ok_or_else(|| format!("{}: erwartet GUI_WINDOW", fn_))?;
        // `color` wird hier BEWUSST nicht aus dem Thema gefuellt. Frueher wurde
        // die Textfarbe beim Anlegen kopiert und damit eingefroren: wer danach
        // das Thema wechselte, hatte Beschriftungen in der Farbe des alten --
        // im hellen Thema also weisse Schrift auf weissem Grund. -1 heisst
        // "dem Thema folgen" und wird erst beim Zeichnen aufgeloest.
        // Massstab: hier, weil JEDES Widget durch diesen Trichter laeuft --
        // auch die aus `GUI_LOAD`/`GUI_FROM_JSON`. Eine im Designer bei
        // Massstab 1 gebaute Form wird damit einfach groesser gezeichnet.
        wdg.x = sx; wdg.y = sy; wdg.w = sw; wdg.h = sh;
        // Breite oder Hoehe 0 heisst "nach Inhalt" -- gemessen im ersten GUI_UPDATE.
        if sw <= 0 { wdg.auto_w = true; }
        if sh <= 0 { wdg.auto_h = true; }
        wdg.bx = wdg.x; wdg.by = wdg.y; wdg.bw = wdg.w; wdg.bh = wdg.h;  // Anchor-Basis
        let idx = w.widgets.len();
        w.widgets.push(wdg);
        Ok(Self::enc_widget(wi, idx))
    }

    fn blank(kind: Kind, x: i32, y: i32, w: i32, h: i32) -> Widget {
        Widget {
            kind, x, y, w, h, text: String::new(), color: -1,   // -1 = dem Thema folgen
            value: 0.0, min: 0.0, max: 1.0, checked: false,
            placeholder: String::new(), clicked: false, hovered: false,
            on_click: None, on_change: None, beobachtet: false, stand_vorher: String::new(), geaendert: false, ov: HashMap::new(), tbl: None, tree: None, list: None,
            layout: None, auto_w: false, auto_h: false,
            panel: None, panel_von: -1, tabctl: None, tc_von: -1, tc_seite: -1, dock_von: -1, akk: None, wz: None,
            leiste: None, status: None, pfad: None, sinnbild: String::new(),
            rich: None, vert: false, unbestimmt: false, bildmodus: 0,
            ziehbar: false, ablage: false, abgelegt: false,
            umbruch: false, bind: String::new(), form: String::new(),
            min_w: 0, min_h: 0, nat_w: w, nat_h: h, regeln: Vec::new(), fehler: String::new(), fehler_label: -1,
            on_hover: None, on_leave: None, on_focus: None, on_blur: None,
            was_hovered: false, was_focused: false,
            ueber_t: 0.0, druck_t: 0.0, fokus_t: 0.0, rollen_t: None, zeile_b: Blende::default(), zeile_jetzt: -1, ganzer_text: String::new(), rad_ziel: None, rad_letzt: 0.0,
            alive: true, visible: true, rund: false, variante: 0,
            group: String::new(), items: Vec::new(), sel: -1,
            enabled: true, font: -1, font_size: 0, stil: 0,
            anchor: 5, bx: x, by: y, bw: w, bh: h,         // Default: oben-links (L|T)
            caret: 0, sel_anchor: 0, scroll: 0,
            tab_page: -1,
            tooltip: String::new(),
            vorschau: String::new(), vorschau_marke: 0,
            tab_index: 0,
            spans: Vec::new(),
            marken: Vec::new(), marken_text: String::new(),
            zeilenfarben: Vec::new(), zeilenfarben_text: String::new(),
            scroll_x: 0, zeilennummern: false, aktive_zeile: false,
            tab_fuegt_ein: false, tabbreite: 4,
            faltbar: Vec::new(), gefaltet: Vec::new(), marken_zusatz: Vec::new(),
            auto_einzug: false, einzug_anfang: Vec::new(),
            einzug_ende: Vec::new(), einzug_aus: Vec::new(),
            schluss_oeffner: Vec::new(), schluss_texte: Vec::new(),
            farbfelder: Vec::new(), wellen: Vec::new(), hinweise: Vec::new(), hinweise_text: String::new(), hinweise_farbe: -1, hinweise_farben: Vec::new(), hinweise_klickbar: false, hinweis_klick: 0, farbfeld_klick: -1, rand_klick: (0, -1), farbfeld_zug: false, zeiger: None, rad_stand: None, klick_n: 0, klick_zeit: -10.0, klick_idx: -1, wort_zug: false,
            formatiert: false, paare: Vec::new(), teil_von: None, teil_stand: String::new(), stile: Vec::new(), stile_text: String::new(), tipp_stil: None,
            abkuerzungen: Vec::new(), abk_treffer: -1, einzugslinien: false, kopfzeilen: 0, hg_bild: -1, hg_deckkraft: 0,
            tab_meldet: false, tab_treffer: false,
            spalten_start: (-1, -1),
            hsv: [0.0, 1.0, 1.0], alpha: 255, alpha_an: false,
            datum: [2000, 1, 1], datum_min: None, datum_max: None, wochenbeginn: 0, kompakt: false,
            zeit: [12, 0, 0], zeit_sek: false,
            step: 1.0,
            align: -1, wrap: false, passwort: false, nur_lesen: false, maxlaenge: 0, zahlen: 0, frei: false,
            entered: false, on_enter: None, undo: Vec::new(), redo: Vec::new(), undo_zeit: -10.0,
            feld: None,
            name: String::new(), klassen: Vec::new(), stil_auf: None, stil_key: None,
            bau: None,
            rs: None,
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
            // Mindestmass gilt auch hier: ein gedehntes Widget schrumpft
            // nicht unter das, was sein Inhalt braucht.
            if wd.min_w > 0 { w = w.max(wd.min_w); }
            if wd.min_h > 0 { h = h.max(wd.min_h); }
            wd.x = x; wd.y = y; wd.w = w; wd.h = h;
        }
    }

    // --- Window-Flags ---
    pub fn window_movable(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_MOVABLE")?.movable = f; Ok(())
    }
    /// GUI_WINDOW_TITLE(win, titel$): die Titelleiste nachtraeglich beschriften
    /// -- ein Fenster, das den Namen des Dokuments traegt, muss ihn wechseln
    /// koennen, ohne neu gebaut zu werden.
    pub fn window_title(&mut self, h: i64, titel: &str) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_TITLE")?.title = titel.to_string(); Ok(())
    }

    /// GUI_WINDOW_GLOW(win [, farbe [, dauer_ms]]): einen Durchgang starten.
    /// Farbe und Dauer bleiben fuer die naechsten gesetzt. Laeuft schon
    /// einer, beginnt er von vorn.
    pub fn window_glow(&mut self, h: i64, farbe: Option<i64>, dauer_ms: Option<f64>) -> Result<(), String> {
        let w = self.win_mut(h, "GUI_WINDOW_GLOW")?;
        if let Some(f) = farbe { w.glanz.farbe = f; }
        if let Some(d) = dauer_ms {
            if !(d > 0.0) { return Err("GUI_WINDOW_GLOW: die Dauer muss groesser als 0 ms sein".into()); }
            w.glanz.dauer = (d / 1000.0) as f32;
        }
        w.glanz.t = 0.0;
        Ok(())
    }

    pub fn window_glow_stop(&mut self, h: i64) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_GLOW_STOP")?.glanz.t = -1.0; Ok(())
    }

    pub fn window_glowing(&mut self, h: i64) -> Result<bool, String> {
        Ok(self.win_mut(h, "GUI_WINDOW_GLOWING")?.glanz.t >= 0.0)
    }

    /// GUI_WINDOW_GLOW_SET(win, schluessel$, wert). `zahl` ist der Wert als
    /// Zahl (None, wenn er keine ist), `text` als Text.
    pub fn window_glow_set(&mut self, h: i64, key: &str, zahl: Option<f64>, text: &str) -> Result<(), String> {
        let n = "GUI_WINDOW_GLOW_SET";
        let w = self.win_mut(h, n)?;
        let brauche = |z: Option<f64>| z.ok_or_else(|| format!("{}: '{}' erwartet eine Zahl", n, key));
        match key.to_lowercase().as_str() {
            "farbe" | "color" => w.glanz.farbe = brauche(zahl)? as i64,
            "dauer" | "duration" => {
                let d = brauche(zahl)?;
                if !(d > 0.0) { return Err(format!("{}: die Dauer muss groesser als 0 ms sein", n)); }
                w.glanz.dauer = (d / 1000.0) as f32;
            }
            "pause" | "wiederholen" | "repeat" => w.glanz.pause = (brauche(zahl)?.max(0.0) / 1000.0) as f32,
            "breite" | "width" => w.glanz.breite = brauche(zahl)?.clamp(0.05, 2.0) as f32,
            "staerke" | "strength" | "intensity" => w.glanz.staerke = brauche(zahl)?.clamp(0.0, 1.0) as f32,
            "rand" | "border" => w.glanz.rand = brauche(zahl)? != 0.0,
            "art" | "kind" | "style" => {
                w.glanz.art = match (zahl, text.to_lowercase().as_str()) {
                    (_, "streif" | "stripe" | "band") => 0,
                    (_, "lampe" | "taschenlampe" | "licht" | "lamp" | "flashlight" | "spot") => 1,
                    (_, "rahmen" | "kante" | "streiflicht" | "rim" | "edge") => 2,
                    (Some(z), _) if z == 0.0 || z == 1.0 || z == 2.0 => z as u8,
                    _ => return Err(format!("{}: art ist streif, lampe oder rahmen (nicht '{}')", n, text)),
                };
            }
            "richtung" | "direction" => {
                let r = match (zahl, text.to_lowercase().as_str()) {
                    (_, "rechts" | "right") => 0,
                    (_, "links" | "left") => 1,
                    (_, "unten" | "down") => 2,
                    (_, "oben" | "up") => 3,
                    (Some(z), _) if (0.0..=3.0).contains(&z) => z as u8,
                    _ => return Err(format!("{}: richtung ist rechts, links, unten oder oben (nicht '{}')", n, text)),
                };
                w.glanz.richtung = r;
            }
            _ => return Err(format!("{}: unbekannter Schluessel '{}' (bekannt: {})", n, key, GLANZ_SCHLUESSEL)),
        }
        Ok(())
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
    /// GUI_WINDOW_ZOOM(win, faktor): ein Fenster im Entwurfsmodus
    /// vergroessert zeichnen (0.25..4) -- fuer einen Form-Designer, dessen
    /// Flaeche sonst so klein ist wie die Form. Die Lage (x, y) bleibt, die
    /// Groesse waechst; Geometrie nach aussen bleibt UNvergroessert,
    /// GUI_HIT_TEST rechnet die Maus zurueck.
    pub fn window_zoom(&mut self, h: i64, f: f64) -> Result<(), String> {
        if !(0.25..=4.0).contains(&f) { return Err("GUI_WINDOW_ZOOM: der Faktor geht von 0.25 bis 4".into()); }
        let w = self.win_mut(h, "GUI_WINDOW_ZOOM")?;
        if !w.entwurf { return Err("GUI_WINDOW_ZOOM: nur fuer ein Fenster im Entwurfsmodus (GUI_WINDOW_DESIGN) -- es nimmt sonst Eingaben an, und die Maus traefe daneben".into()); }
        w.zoom = f as f32;
        Ok(())
    }
    pub fn window_get_zoom(&self, h: i64) -> Result<f64, String> {
        let w = self.windows.get(h as usize).ok_or("GUI_WINDOW_GET_ZOOM: ungueltiges GUI_WINDOW-Handle")?;
        Ok(w.zoom as f64)
    }
    /// Der Faktor, mit dem ein Fenster gerade gezeichnet wird (1 ohne Zoom
    /// oder ausserhalb des Entwurfsmodus).
    fn zoom_von(&self, wi: usize) -> f32 {
        let w = &self.windows[wi];
        if w.entwurf { w.zoom } else { 1.0 }
    }
    /// Das Rechteck, das ein Fenster auf dem Schirm einnimmt.
    fn sicht_rect(&self, wi: usize) -> (i32, i32, i32, i32) {
        let w = &self.windows[wi];
        let z = self.zoom_von(wi);
        let r = if z == 1.0 { (w.x, w.y, w.w, w.h) }
                else { (w.x, w.y, (w.w as f32 * z).round() as i32, (w.h as f32 * z).round() as i32) };
        match self.zuschnitt_von(wi) {
            Some(c) => Self::schnitt(r, c),
            None => r,
        }
    }
    fn zuschnitt_von(&self, wi: usize) -> Option<(i32, i32, i32, i32)> {
        let w = &self.windows[wi];
        if w.entwurf { w.zuschnitt } else { None }
    }
    /// Kommen das aufgeklappte Leistenmenue und die offene Klappliste dieses
    /// Fensters in die obere Schicht (`draw_top`)? Ja, ausser das Fenster
    /// wird vergroessert oder zugeschnitten gezeichnet -- dann stehen sie in
    /// dessen Lage und bleiben beim Fenster.
    fn popups_oben(&self, wi: usize) -> bool {
        // Ein natives Fenster hat seine eigene Flaeche: Menues und
        // Klapplisten gehoeren hinein, nicht ueber das Hauptfenster.
        self.zoom_von(wi) == 1.0 && self.zuschnitt_von(wi).is_none() && self.windows[wi].nativ == 0
    }
    /// Ein natives Fenster (Weg E) in sein Render-Ziel zeichnen, mit dem
    /// Zeiger in seiner Sicht; der Hilfetext gehoert hinein, wenn er zu
    /// einem seiner Widgets gehoert.
    pub(super) fn nativ_zeichnen(&self, g: &mut Graphics, wi: usize) {
        let id = self.windows[wi].nativ;
        if id == 0 || !g.nativ_beginnen(id) { return; }
        g.sicht_setzen(Some(id));
        self.draw_window_sicht(g, wi);
        self.draw_top_flaeche(g, id);
        g.sicht_setzen(Some(0));
        g.nativ_beenden();
    }
    /// Ein Druck auf einer anderen Oberflaeche schliesst die Popups eines
    /// Fensters, das nicht auf ihr steht -- wie ein Klick daneben. Sonst
    /// verglichen die Treffertests der Popups Koordinaten zweier Flaechen.
    pub(super) fn fremde_popups_zu(&mut self) {
        let f = self.flaeche;
        let fremd = |w: &Vec<Window>, wi: usize| w.get(wi).map(|x| x.nativ != f).unwrap_or(true);
        if self.context_open.map(|c| fremd(&self.windows, c.0)).unwrap_or(false) { self.context_open = None; self.sub_chain.clear(); }
        if self.open_menu.map(|m| fremd(&self.windows, m.0)).unwrap_or(false) { self.open_menu = None; }
        if self.open_dropdown.map(|d| fremd(&self.windows, d.0)).unwrap_or(false) { self.open_dropdown = None; }
        if self.datum_auf.map(|d| fremd(&self.windows, d.0)).unwrap_or(false) { self.datum_auf = None; }
        if self.leiste_popup.map(|l| fremd(&self.windows, l.0)).unwrap_or(false) { self.leiste_popup = None; }
        if self.rs_fenster().map(|wi| fremd(&self.windows, wi)).unwrap_or(false) { self.rs_menue = None; }
    }
    /// Zu welchem Fenster gehoert der Hilfetext, der gerade dran ist?
    pub(super) fn tip_fenster(&self) -> Option<usize> {
        self.hover_w.map(|(wi, _)| wi).or(self.tip_fokus.map(|(wi, _)| wi))
    }

    /// GUI_WINDOW_NATIVE(win, an): das Fenster als echtes OS-Fenster
    /// (Weg E) oder zurueck ins Hauptfenster. TRUE, wenn es danach ein
    /// natives ist -- FALSE heisst bei `an` der Rueckfall: im Browser, ohne
    /// Bildschirm, unter macOS (noch ungeprueft) oder wenn das System das
    /// Fenster ablehnt, bleibt es ein gui-Fenster wie bisher.
    pub fn window_native(&mut self, g: Option<&mut Graphics>, h: i64, an: bool) -> Result<bool, String> {
        let wi = h as usize;
        let w = self.windows.get(wi).filter(|w| w.alive)
            .ok_or_else(|| "GUI_WINDOW_NATIVE: ungueltiges GUI_WINDOW-Handle".to_string())?;
        if an == (w.nativ != 0) { return Ok(an); }
        let Some(g) = g else { return Ok(false) };
        if an {
            if w.dock_host >= 0 || w.pop.is_some() { return Ok(false); }
            let (fx, fy) = g.fenster_lage();
            let Some(id) = g.nativ_anlegen(&w.title, w.w, w.h, fx + w.x, fy + w.y, w.resizable, true) else { return Ok(false) };
            let w = &mut self.windows[wi];
            w.nativ_vorher = Some((w.x, w.y, w.chrome));
            w.nativ = id;
            w.x = 0; w.y = 0; w.chrome = false;
            self.clear_window_interactions(wi);
            Ok(true)
        } else {
            let id = w.nativ;
            g.nativ_schliessen(id);
            let w = &mut self.windows[wi];
            w.nativ = 0;
            if let Some((x, y, c)) = w.nativ_vorher.take() { w.x = x; w.y = y; w.chrome = c; }
            self.clear_window_interactions(wi);
            Ok(false)
        }
    }
    /// GUI_WINDOW_IS_NATIVE(win): ist es gerade ein echtes OS-Fenster?
    pub fn window_is_native(&self, h: i64) -> Result<bool, String> {
        self.windows.get(h as usize).map(|w| w.alive && w.nativ != 0)
            .ok_or_else(|| "GUI_WINDOW_IS_NATIVE: ungueltiges GUI_WINDOW-Handle".to_string())
    }
    /// GUI_WINDOW_NATIVE_SCALE(win): um wie viel das native Fenster auf
    /// seinem Schirm groesser gezeichnet wird als im Hauptfenster (Dichte
    /// seines Schirms durch die des Hauptfensters). 1.0 im Hauptfenster.
    pub fn window_native_scale(&self, g: Option<&Graphics>, h: i64) -> Result<f64, String> {
        let w = self.windows.get(h as usize).filter(|w| w.alive)
            .ok_or_else(|| "GUI_WINDOW_NATIVE_SCALE: ungueltiges GUI_WINDOW-Handle".to_string())?;
        Ok(match (g, w.nativ) { (Some(g), id) if id != 0 => g.nativ_massstab(id) as f64, _ => 1.0 })
    }
    /// Einmal je Bild, vor der Bedienung: was das System an den nativen
    /// Fenstern getan hat (Kreuz, Groesse, Fokus), und was das Programm
    /// geaendert hat (Titel, sichtbar, Lage, Groesse), abgleichen.
    pub(super) fn nativ_abgleich(&mut self, g: &mut Graphics) {
        // Native Fenster, zu denen kein lebendes gui-Fenster mehr gehoert
        // (GUI_WINDOW_DESTROY, GUI_RESET), schliessen.
        let lebend: Vec<u32> = self.windows.iter().filter(|w| w.alive && w.nativ != 0).map(|w| w.nativ).collect();
        for id in g.nativ_ids() { if !lebend.contains(&id) { g.nativ_schliessen(id); } }
        for wi in 0..self.windows.len() {
            let id = self.windows[wi].nativ;
            if id == 0 { continue; }
            if !self.windows[wi].alive { self.windows[wi].nativ = 0; continue; }
            if g.nativ_schliessen_gewuenscht(id) && self.windows[wi].closable {
                // Wie ein Klick auf das Kreuz der gui (gui_bedienung.rs).
                self.windows[wi].close_clicked = true;
                self.windows[wi].visible = false;
                self.clear_window_interactions(wi);
            }
            if let Some((b, h)) = g.nativ_groesse_neu(id) {
                let w = &mut self.windows[wi];
                if (w.w, w.h) != (b, h) { w.w = b; w.h = h; self.relayout(wi); }
            }
            let w = &mut self.windows[wi];
            // Vom Programm verschoben (GUI_WINDOW_SET_BOUNDS): das OS-Fenster
            // folgt, das gui-Fenster bleibt auf seiner Flaeche bei (0, 0).
            if w.x != 0 || w.y != 0 {
                let (fx, fy) = g.fenster_lage();
                g.nativ_lage_setzen(id, fx + w.x, fy + w.y);
                w.x = 0; w.y = 0;
            }
            // Eine abgerissene Dock-Leiste zeichnet ihren Kopf selbst (das
            // System gibt ihr keinen Rahmen).
            if w.dock_raus_vorher.is_none() { w.chrome = false; }
            // Modal (Stufe 2): das Hauptfenster wird Besitzer -- der Dialog
            // bleibt darueber; beim Modalwerden bekommt er den Fokus.
            let modal = self.modal == Some(wi);
            g.nativ_groesse(id, w.w, w.h, true);
            g.nativ_besitzer(id, modal);
            g.nativ_titel(id, &w.title);
            g.nativ_sichtbar(id, w.visible);
        }
        // Bekommt ein natives Fenster den Fokus des Systems, bekommt sein
        // gui-Fenster den der gui -- sonst landete Getipptes im Fenster,
        // das die gui zuletzt fokussiert hatte.
        if let Some(id) = self.nativ_fokus_wunsch.take() { g.nativ_fokussieren(id); }
        let fokus = g.fokus_flaeche();
        if fokus != self.fokus_flaeche {
            self.fokus_flaeche = fokus;
            let ziel = if fokus != 0 {
                self.windows.iter().position(|w| w.alive && w.nativ == fokus)
            } else {
                self.z_order.iter().rev().copied().find(|&i| self.windows[i].alive && self.windows[i].visible && self.windows[i].nativ == 0)
            };
            if let Some(wi) = ziel { self.focus_window = Some(wi); }
        }
    }
    /// Die Schnittmenge zweier Rechtecke (x, y, b, h); leer = Breite/Hoehe 0.
    fn schnitt(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> (i32, i32, i32, i32) {
        let x = a.0.max(b.0);
        let y = a.1.max(b.1);
        let x2 = (a.0 + a.2).min(b.0 + b.2);
        let y2 = (a.1 + a.3).min(b.1 + b.3);
        (x, y, (x2 - x).max(0), (y2 - y).max(0))
    }
    /// GUI_WINDOW_CLIP(win, x, y, b, h): nur dieser Teil des Schirms zeigt das
    /// Fenster (logische Punkte wie GUI_WINDOW_SET_BOUNDS); b oder h <= 0 hebt
    /// den Zuschnitt auf. Wie GUI_WINDOW_ZOOM nur im Entwurfsmodus -- ein
    /// Fenster, das Eingaben nimmt, bekaeme sonst Tasten fuer Felder, die
    /// niemand sieht.
    pub fn window_clip(&mut self, h: i64, x: i32, y: i32, b: i32, ht: i32) -> Result<(), String> {
        let r = (self.sk(x), self.sk(y), self.sk(b), self.sk(ht));
        let w = self.win_mut(h, "GUI_WINDOW_CLIP")?;
        if !w.entwurf { return Err("GUI_WINDOW_CLIP: nur fuer ein Fenster im Entwurfsmodus (GUI_WINDOW_DESIGN)".into()); }
        w.zuschnitt = if b > 0 && ht > 0 { Some(r) } else { None };
        Ok(())
    }
    /// Ein Schirmpunkt in die (unvergroesserten) Koordinaten des Fensters.
    fn ins_fenster(&self, wi: usize, mx: i32, my: i32) -> (i32, i32) {
        let z = self.zoom_von(wi);
        if z == 1.0 { return (mx, my); }
        let w = &self.windows[wi];
        (w.x + ((mx - w.x) as f32 / z).floor() as i32, w.y + ((my - w.y) as f32 / z).floor() as i32)
    }
    /// Ein Fenster zeichnen -- vergroessert ueber die Zoom-Flaeche, wenn es
    /// einen Faktor hat.
    fn draw_window_sicht(&self, g: &mut Graphics, wi: usize) {
        let z = self.zoom_von(wi);
        let clip = self.zuschnitt_von(wi);
        if z != 1.0 && g.zoom_beginnen() {
            self.draw_window(g, wi);
            let w = &self.windows[wi];
            // Zugeschnitten wird erst beim Uebertragen auf den Schirm.
            g.zoom_beenden(w.x, w.y, w.w, w.h, z, clip);
        } else {
            if let Some((x, y, b, h)) = clip { g.push_clip(x, y, b, h); }
            self.draw_window(g, wi);
            if clip.is_some() { g.pop_clip(); }
        }
    }

    pub fn window_design(&mut self, h: i64, f: bool) -> Result<(), String> {
        let wi = h as usize;
        self.win_mut(h, "GUI_WINDOW_DESIGN")?.entwurf = f;
        // Ein Fokus, der schon drin sitzt, bliebe sonst haengen.
        if f { self.clear_window_interactions(wi); }
        Ok(())
    }
    pub fn window_min_size(&mut self, h: i64, w: i32, ht: i32) -> Result<(), String> {
        let (w, ht) = (self.sk(w), self.sk(ht));
        let win = self.win_mut(h, "GUI_WINDOW_SET_MIN_SIZE")?;
        win.min_w = w.max(0); win.min_h = ht.max(0); Ok(())
    }
    pub fn window_max_size(&mut self, h: i64, w: i32, ht: i32) -> Result<(), String> {
        let (w, ht) = (self.sk(w), self.sk(ht));
        let win = self.win_mut(h, "GUI_WINDOW_SET_MAX_SIZE")?;
        win.max_w = w.max(0); win.max_h = ht.max(0); Ok(())
    }
    /// GUI_WINDOW_FRONT: ein Fenster nach vorn holen, OHNE ihm den Fokus zu
    /// geben -- fuer Vorschlagslisten und Tooltips, die beim Tippen ueber dem
    /// Code liegen sollen, waehrend die Tastatur im Code bleibt. Vorher kam
    /// ein Fenster nur ueber einen Klick oder GUI_FOCUS nach vorn; per
    /// GUI_WINDOW_VISIBLE eingeblendet lag es HINTER dem Fenster, das zuletzt
    /// angeklickt war, und war unsichtbar (die Vervollstaendigung der IDE).
    pub fn window_front(&mut self, h: i64) -> Result<(), String> {
        self.win_mut(h, "GUI_WINDOW_FRONT")?;
        self.bring_to_front(h as usize);
        Ok(())
    }

    pub fn window_visible(&mut self, h: i64, f: bool) -> Result<(), String> {
        let w = self.win_mut(h, "GUI_WINDOW_VISIBLE")?;
        w.visible = f; if f { w.close_clicked = false; }
        // Review-Fund: ein unsichtbares Fenster behielt bisher den
        // Tastatur-Fokus (und jede andere Interaktions-Referenz) -- `update()`
        // routete Tippen/Backspace/GUI_ON_CHANGE weiter in ein Fenster, das
        // der Nutzer gar nicht mehr sieht.
        if !f {
            let wi = h as usize;
            self.clear_window_interactions(wi);
        }
        Ok(())
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
        let cs = self.m_roh("check_size");
        let mut wd = Self::blank(Kind::Checkbox, x, y, cs, cs); wd.text = label; wd.checked = default;
        // Das Rechteck umfasst Kaestchen UND Beschriftung (Breite wird im
        // naechsten GUI_UPDATE gemessen): ein Klick auf den Text trifft, und
        // ein Layout-Behaelter reserviert den ganzen Platz. Gezeichnet wird
        // das Kaestchen unabhaengig von der Breite (siehe draw_widget).
        wd.auto_w = true;
        self.add_widget(win, "GUI_CHECKBOX", wd)
    }
    /// Kippschalter (An/Aus-Pille). Zustand liegt wie beim Kaestchen in
    /// `checked`, also lesbar mit GUI_CHECKED und setzbar mit GUI_SET_CHECKED.
    /// `value` fuehrt die Schiebe-Animation (0 = aus, 1 = an).
    pub fn toggle(&mut self, win: i64, label: String, x: i32, y: i32, default: bool) -> Result<i64, String> {
        let h = self.m_roh("check_size").max(14);
        let mut wd = Self::blank(Kind::Toggle, x, y, h * 2, h);
        wd.text = label;
        wd.auto_w = true;                        // wie beim Kaestchen
        wd.checked = default;
        wd.value = if default { 1.0 } else { 0.0 };
        self.add_widget(win, "GUI_TOGGLE", wd)
    }

    /// Drehregler. Wird durch senkrechtes Ziehen verstellt (nach oben = mehr) --
    /// das ist die Bedienung, die man von Mischpult-Oberflaechen kennt und die
    /// ohne Kreisbewegung der Maus auskommt.
    pub fn knob(&mut self, win: i64, x: i32, y: i32, groesse: i32,
                mn: f64, mx: f64, default: f64) -> Result<i64, String> {
        if mx <= mn { return Err("GUI_KNOB: max muss > min sein".into()); }
        let s = groesse.max(16);
        let mut wd = Self::blank(Kind::Knob, x, y, s, s);
        wd.min = mn; wd.max = mx; wd.value = default.clamp(mn, mx);
        self.add_widget(win, "GUI_KNOB", wd)
    }

    pub fn slider(&mut self, win: i64, x: i32, y: i32, w: i32, mn: f64, mx: f64, default: f64) -> Result<i64, String> {
        if mx <= mn { return Err("GUI_SLIDER: max muss > min sein".into()); }
        let sh = self.m_roh("slider_h");
        let mut wd = Self::blank(Kind::Slider, x, y, w, sh);
        wd.min = mn; wd.max = mx; wd.value = default.clamp(mn, mx);
        self.add_widget(win, "GUI_SLIDER", wd)
    }
    /// Zahlenfeld mit +/- Tasten (Spinner). Wert via Pfeile/Mausrad/Klick auf die
    /// Schaltflaechen, in Schritten von `step`, geklemmt auf [mn, mx].
    pub fn spinner(&mut self, win: i64, x: i32, y: i32, w: i32, mn: f64, mx: f64,
                   default: f64, step: f64) -> Result<i64, String> {
        if mx <= mn { return Err("GUI_SPINNER: max muss > min sein".into()); }
        let h = (self.m_roh("title_h") + 2).max(26);
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
        wd.min = self.sk(mn) as f64; wd.max = self.sk(mx) as f64;
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

}

#[cfg(test)]
mod geldfeld_tests {
    use super::{cent_text, nummer_mit_null, zahl_erlaubt};

    #[test]
    fn cent_als_anzeige() {
        assert_eq!(cent_text(4800), "48,00");
        assert_eq!(cent_text(5), "0,05");
        assert_eq!(cent_text(-1999), "-19,99");
        assert_eq!(cent_text(0), "0,00");
    }

    #[test]
    fn fuehrende_null_nur_vor_ziffern() {
        assert!(nummer_mit_null("080331"));
        assert!(nummer_mit_null(" 01 "));
        assert!(!nummer_mit_null("0,99"));
        assert!(!nummer_mit_null("0.5"));
        assert!(!nummer_mit_null("0"));
        assert!(!nummer_mit_null("80331"));
    }

    #[test]
    fn geldfeld_nimmt_komma_wie_kommazahl() {
        assert!(zahl_erlaubt("12,50", 3));
        assert!(!zahl_erlaubt("12,5,0", 3));
        assert!(!zahl_erlaubt("12,50", 1));
    }
}

#[cfg(test)]
mod kuerzel_tests {
    use super::kuerzel_parsen;

    #[test]
    fn buchstabe_mit_strg() {
        assert_eq!(kuerzel_parsen("Strg+S").unwrap(), (1, 's' as i64));
        assert_eq!(kuerzel_parsen("ctrl+shift+o").unwrap(), (3, 'o' as i64));
        assert_eq!(kuerzel_parsen("Alt+Enter").unwrap(), (4, 13));
    }

    #[test]
    fn benannte_tasten_deutsch_und_englisch() {
        assert_eq!(kuerzel_parsen("F5").unwrap(), kuerzel_parsen("f5").unwrap());
        assert_eq!(kuerzel_parsen("Entf").unwrap(), kuerzel_parsen("Delete").unwrap());
        assert_eq!(kuerzel_parsen("Entf").unwrap().1, 127);
        assert_eq!(kuerzel_parsen("Bild ab").unwrap(), kuerzel_parsen("PageDown").unwrap());
        assert_eq!(kuerzel_parsen("Strg+Pos1").unwrap().0, 1);
    }

    #[test]
    fn fehler_im_klartext() {
        assert!(kuerzel_parsen("Strg+").unwrap_err().contains("keine Taste"));
        assert!(kuerzel_parsen("Strg+Q+W").unwrap_err().contains("mehr als eine Taste"));
        assert!(kuerzel_parsen("Hyper").unwrap_err().contains("unbekannt"));
    }

    #[test]
    fn plus_und_minus() {
        use super::{K_MINUS, K_PLUS};
        assert_eq!(kuerzel_parsen("Strg+Plus").unwrap(), (1, K_PLUS));
        assert_eq!(kuerzel_parsen("Strg++").unwrap(), (1, K_PLUS));
        assert_eq!(kuerzel_parsen("Strg+Minus").unwrap(), (1, K_MINUS));
        assert_eq!(kuerzel_parsen("Strg+-").unwrap(), (1, K_MINUS));
        assert_eq!(kuerzel_parsen("Strg+0").unwrap(), (1, '0' as i64));
        assert!(kuerzel_parsen("Strg+Plus+Minus").unwrap_err().contains("mehr als eine Taste"));
    }

    #[test]
    fn punkt_und_komma() {
        assert_eq!(kuerzel_parsen("Strg+.").unwrap(), (1, 46));
        assert_eq!(kuerzel_parsen("Strg+Punkt").unwrap(), (1, 46));
        assert_eq!(kuerzel_parsen("Strg+,").unwrap(), (1, 44));
        assert_eq!(kuerzel_parsen("Strg+Komma").unwrap(), (1, 44));
    }
}

#[cfg(test)]
mod zahlenfilter_tests {
    use super::zahl_erlaubt;

    #[test]
    fn frei_nimmt_alles() { assert!(zahl_erlaubt("ab-1.2x", 0)); }

    #[test]
    fn ganze_zahl() {
        assert!(zahl_erlaubt("", 1));
        assert!(zahl_erlaubt("-", 1), "Zwischenstand beim Tippen einer negativen Zahl");
        assert!(zahl_erlaubt("-42", 1));
        assert!(!zahl_erlaubt("4-2", 1));
        assert!(!zahl_erlaubt("4.2", 1));
        assert!(!zahl_erlaubt("12a", 1));
    }

    #[test]
    fn kommazahl() {
        assert!(zahl_erlaubt("3.14", 2));
        assert!(zahl_erlaubt("3,14", 2), "deutsche Tastatur, deutsches Komma");
        assert!(zahl_erlaubt("-0.5", 2));
        assert!(!zahl_erlaubt("3.1.4", 2));
        assert!(!zahl_erlaubt("3,1.4", 2));
        assert!(!zahl_erlaubt("x", 2));
    }
}

#[cfg(test)]
mod label_tests {
    use super::{Gui, Kind};

    #[test]
    fn beschriftung_ohne_farbe_folgt_dem_thema() {
        // Regression: die Textfarbe wurde beim Anlegen aus dem Thema kopiert
        // und eingefroren. Ein spaeterer Themenwechsel liess Beschriftungen
        // in der Farbe des ALTEN Themas stehen -- im hellen Thema also weisse
        // Schrift auf weissem Grund.
        let mut gui = Gui::new();
        gui.theme_preset("glas_dunkel").unwrap();
        let w = gui.new_window("W".into(), 0, 0, 200, 100);
        let h = gui.label(w, "Test".into(), 10, 10, None).unwrap();
        let (wi, i) = Gui::dec_widget(h);
        assert_eq!(gui.windows[wi].widgets[i].color, -1, "Farbe wurde eingefroren");
        assert!(gui.windows[wi].widgets[i].kind == Kind::Label);

        // Mit ausdruecklicher Farbe bleibt diese erhalten.
        let h2 = gui.label(w, "Fest".into(), 10, 30, Some(0xFF0000)).unwrap();
        let (wi2, i2) = Gui::dec_widget(h2);
        assert_eq!(gui.windows[wi2].widgets[i2].color, 0xFF0000);
    }
}

#[cfg(test)]
mod gloss_tests {
    use super::{font_wahl, gloss_unten, knopf_text_x};

    #[test]
    fn eigene_groesse_behaelt_die_globale_schrift() {
        // Regression: nur die Groesse zu setzen liess das Zeichnen mit dem
        // Handle -1 laufen und damit auf die eingebaute Pixelschrift kippen.
        let aktiv = 3;
        assert_eq!(font_wahl(-1, aktiv), aktiv, "Groesse hat die Schrift gewechselt");
        // Eine eigene Schrift gewinnt.
        assert_eq!(font_wahl(7, aktiv), 7);
        // Ohne global gesetzte Schrift bleibt es die Standardschrift.
        assert_eq!(font_wahl(-1, -1), -1);
    }

    #[test]
    fn knopf_text_sitzt_mittig_und_wird_sonst_beschnitten() {
        // Passt: mittig, kein Beschnitt.
        assert_eq!(knopf_text_x(100, 40), (30, false));
        assert_eq!(knopf_text_x(100, 100), (0, false));
        // Passt nicht: am linken Rand beginnen und abschneiden. Zentriert
        // ueberstehen zu lassen wuerde auch den ANFANG anschneiden.
        assert_eq!(knopf_text_x(60, 200), (0, true));
        assert_eq!(knopf_text_x(0, 10), (0, true));
    }

    #[test]
    fn runde_formen_bekommen_den_glanz_ueber_die_ganze_hoehe() {
        // Kreis (Radius = halbe Hoehe) und Kapsel: ganze Hoehe.
        assert_eq!(gloss_unten(0, 60, 30), 60);
        assert_eq!(gloss_unten(10, 70, 40), 70);
        // Normaler Knopf: nur die obere Haelfte.
        assert_eq!(gloss_unten(0, 30, 5), 15);
        assert_eq!(gloss_unten(100, 140, 6), 120);
        // Genau an der Grenze zaehlt schon als rund.
        assert_eq!(gloss_unten(0, 40, 20), 40);
        assert_eq!(gloss_unten(0, 40, 19), 20);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auszeichnungen_als_stil() {
        use crate::schnitt::{DURCH, FETT, KURSIV, UNTER};
        let mut ziele = Vec::new();
        let w = rt_inline("a ***b*** c ~~d~~ <u>e</u> *f* **g**", 0, &mut ziele);
        let stil = |t: &str| w.iter().find(|x| x.text == t).map(|x| x.stil).unwrap();
        assert_eq!(stil("a"), 0);
        assert_eq!(stil("b"), FETT | KURSIV);
        assert_eq!(stil("c"), 0);
        assert_eq!(stil("d"), DURCH);
        assert_eq!(stil("e"), UNTER);
        assert_eq!(stil("f"), KURSIV);
        assert_eq!(stil("g"), FETT);
        // Kursiv ist nicht mehr gedaempft: dieselbe Rolle wie der Grund.
        assert!(w.iter().all(|x| x.rolle == 0));
        // `~~` und `<u>` im Code bleiben Text.
        let c = rt_inline("`~~x~~ <u>`", 0, &mut ziele);
        assert!(c.iter().all(|x| x.stil == 0 && x.code));
        assert_eq!(c.iter().map(|x| x.text.clone()).collect::<Vec<_>>().join(" "), "~~x~~ <u>");
    }

    #[test]
    fn kontrast_nach_wcag() {
        assert!((kontrast(0x000000, 0xFFFFFF) - 21.0).abs() < 0.01);
        assert!((kontrast(0xFFFFFF, 0xFFFFFF) - 1.0).abs() < 0.001);
        // Das Grau des dunklen Glas-Themas auf seinem Widget-Grund war zu
        // schwach -- genau dafuer gibt es `leise`.
        assert!(kontrast(0x8B97A6, 0x39424F) < 4.5);
        let mut gui = Gui::new();
        gui.theme_preset("glas_dunkel").unwrap();
        for bg in [0x39424F, 0x375453, 0x232A33, 0x2E2A3C] {
            let c = gui.leise(bg);
            assert!(kontrast(c, bg) >= 4.5, "{:06X} auf {:06X}", c, bg);
        }
        gui.theme_preset("glas_hell").unwrap();
        for bg in [0xFBFCFE, 0xDAEDE5, 0xE2E8EF] {
            let c = gui.leise(bg);
            assert!(kontrast(c, bg) >= 4.5, "{:06X} auf {:06X}", c, bg);
        }
    }

    // Checkbox-Klick (Press-basiert, ohne Graphics) muss togglen UND die
    /// Nur die Namen der ausgeloesten Rueckrufe -- die Tests hier pruefen
    /// Reihenfolge und Ausloesung, nicht die Bindung an ein Objekt.
    fn namen(cbs: Vec<Rueckruf>) -> Vec<String> {
        cbs.iter().map(|c| c.name.to_string()).collect()
    }

    // Callbacks in der Reihenfolge on_click, on_change in die pending-Queue
    // legen -- identisch zur Python-Referenz (_handle_press).
    #[test]
    fn checkbox_press_queues_callbacks_and_toggles() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 200, 200);
        let cb = g.checkbox(win, "X".into(), 10, 10, false).unwrap();
        g.on_click(cb, Some(Rueckruf::benannt("clicked"))).unwrap();
        g.on_change(cb, Some(Rueckruf::benannt("toggled"))).unwrap();
        // abs Checkbox-Rect: (10, 22+10, 16, 16) -> Mitte (18, 40).
        g.handle_press(18, 40);
        assert_eq!(namen(g.take_pending()), vec!["clicked", "toggled"]);
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

    // Helfer fuer die Regel-Tests (kein Test -- das #[test] davor stand
    // falsch und liess `cargo test --features graphics` nicht uebersetzen).
    fn regel(art: RegelArt, a: f64, b: f64, text: &str) -> Regel {
        Regel { art, a, b, text: text.to_string(), meldung: String::new() }
    }

    #[test]
    fn regeln_pruefen_wie_ein_formular() {
        let p = regel(RegelArt::Pflicht, 0.0, 0.0, "");
        assert_eq!(regel_pruefen(&p, "  ", false).as_deref(), Some("Pflichtfeld"));
        assert!(regel_pruefen(&p, "x", false).is_none());
        assert!(regel_pruefen(&p, "", true).is_none(), "Kaestchen/Klappliste: gewaehlt zaehlt");
        // Leer laesst jede andere Regel durch -- ob leer sein darf, sagt pflicht.
        let z = regel(RegelArt::Zahl, 0.0, 0.0, "");
        assert!(regel_pruefen(&z, "", false).is_none());
        assert!(regel_pruefen(&z, "12,5", false).is_none());
        assert!(regel_pruefen(&z, "12.5", false).is_none());
        assert!(regel_pruefen(&z, "zwoelf", false).is_some());
        let g = regel(RegelArt::Ganz, 0.0, 0.0, "");
        assert!(regel_pruefen(&g, "7", false).is_none() && regel_pruefen(&g, "7,5", false).is_some());
        let b = regel(RegelArt::Bereich, 1.0, 10.0, "");
        assert!(regel_pruefen(&b, "10", false).is_none());
        assert_eq!(regel_pruefen(&b, "11", false).as_deref(), Some("Bitte einen Wert von 1 bis 10 eingeben"));
        let l = regel(RegelArt::Laenge, 2.0, 4.0, "");
        assert!(regel_pruefen(&l, "ab", false).is_none() && regel_pruefen(&l, "abcde", false).is_some());
        assert!(regel_pruefen(&l, "\u{e4}\u{f6}", false).is_none(), "Zeichen, nicht Bytes");
        let e = regel(RegelArt::Email, 0.0, 0.0, "");
        assert!(regel_pruefen(&e, "a@b.de", false).is_none());
        for schlecht in ["ab.de", "@b.de", "a@b", "a@.de", "a@b.", "a b@c.de", "a@b@c.de"] {
            assert!(regel_pruefen(&e, schlecht, false).is_some(), "{}", schlecht);
        }
        let d = regel(RegelArt::Datum, 0.0, 0.0, "");
        assert!(regel_pruefen(&d, "2026-02-28", false).is_none());
        assert!(regel_pruefen(&d, "2026-02-30", false).is_some() && regel_pruefen(&d, "28.02.2026", false).is_some());
        let m = regel(RegelArt::Muster, 0.0, 0.0, "[0-9]{5}");
        assert!(regel_pruefen(&m, "12345", false).is_none() && regel_pruefen(&m, "1234", false).is_some());
        assert!(regel_pruefen(&m, "123456", false).is_some(), "das Muster gilt fuer die GANZE Eingabe");
        let mut eigen = regel(RegelArt::Pflicht, 0.0, 0.0, "");
        eigen.meldung = "Der Name fehlt.".into();
        assert_eq!(regel_pruefen(&eigen, "", false).as_deref(), Some("Der Name fehlt."));
    }

    #[test]
    fn ime_ergebnis_und_vorschau() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 300, 200);
        let tf = g.textinput(win, 10, 10, 200, 26, String::new()).unwrap();
        let (wi, i) = Gui::dec_widget(tf);
        // Ohne Fokus: nichts ist aktiv, ein Ergebnis verpufft.
        assert!(!g.ime_feld_aktiv());
        g.ime_ergebnis("日本", 1.0);
        assert_eq!(g.windows[wi].widgets[i].text, "");
        g.focus(tf).unwrap();
        assert!(g.ime_feld_aktiv());
        // Vorschau haengt am Feld mit Fokus, die Anzeige traegt sie an der Marke.
        g.set_text(tf, "ab".into()).unwrap();          // Marke am Ende
        g.ime_vorschau(Some(("にほ".into(), 1)));
        assert_eq!(g.windows[wi].widgets[i].vorschau, "にほ");
        let (anz, marke) = g.anzeige_mit_vorschau(&g.windows[wi].widgets[i]);
        assert_eq!(anz.iter().collect::<String>(), "abにほ");
        assert_eq!(marke, 3);                          // hinter dem ersten Vorschauzeichen
        // Ergebnis: eingefuegt, Vorschau weg, Marke dahinter, Rueckgaengig moeglich.
        g.ime_ergebnis("日本", 2.0);
        let w = &g.windows[wi].widgets[i];
        assert_eq!(w.text, "ab日本");
        assert_eq!(w.caret, 4);
        assert_eq!(w.vorschau, "");
        assert_eq!(w.undo.last().map(|u| u.0.as_str()), Some("ab"));
        // Hoechstlaenge und Zahlenfilter gelten auch fuer die IME.
        g.textinput_set(tf, "maxlaenge", 5.0).unwrap();
        g.ime_ergebnis("語です", 3.0);
        assert_eq!(g.windows[wi].widgets[i].text, "ab日本語");
        g.ime_vorschau(None);
        assert_eq!(g.windows[wi].widgets[i].vorschau, "");
        // Ein gesperrtes Feld ist kein aktives.
        g.textinput_set(tf, "nur_lesen", 1.0).unwrap();
        assert!(!g.ime_feld_aktiv());
    }

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
        g.on_change(sp, Some(Rueckruf::benannt("changed"))).unwrap();
        assert_eq!(g.value(sp).unwrap(), 5.0);
        g.spinner_step(wi, i, 1);                 // +2 -> 7
        assert_eq!(g.value(sp).unwrap(), 7.0);
        assert_eq!(namen(g.take_pending()), vec!["changed"]);
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
        g.on_change(sp, Some(Rueckruf::benannt("moved"))).unwrap();
        assert_eq!(g.value(sp).unwrap(), 200.0);                 // Start-x
        g.split_off = 0;
        g.drag_split(wi, i, 350, 0);                             // x = 350-100-0 = 250
        assert_eq!(g.value(sp).unwrap(), 250.0);
        assert_eq!(namen(g.take_pending()), vec!["moved"]);
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
        assert!(!g.windows[tw].widgets[ti].enabled);                     // ohne Eintraege dekorativ
    }

    // Werkzeugleiste mit Eintraegen: Geometrie, Klick, Kippen, Sperren,
    // Speichern -- ohne Grafik ueber tb_geom/tb_klick.
    #[test]
    fn werkzeugleiste_mit_eintraegen() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 400, 200);
        let tb = g.toolbar(win, 0, 0, 300, 34).unwrap();
        let (tw, ti) = Gui::dec_widget(tb);
        assert_eq!(g.toolbar_add(tb, Some("neu".into()), -1, "Neu".into(), String::new()).unwrap(), 0);
        assert_eq!(g.toolbar_separator(tb).unwrap(), 1);
        assert_eq!(g.toolbar_add(tb, Some("START".into()), -1, "Start".into(), String::new()).unwrap(), 2);
        assert_eq!(g.toolbar_spacer(tb).unwrap(), 3);
        assert_eq!(g.toolbar_add(tb, Some("einstellungen".into()), -1, String::new(), String::new()).unwrap(), 4);
        assert!(g.windows[tw].widgets[ti].enabled);
        assert!(g.toolbar_add(tb, Some("gibtsnicht".into()), -1, String::new(), String::new()).unwrap_err().contains("bekannt sind"));
        // Geometrie: Knoepfe sind so breit wie hoch (h - 6), die Luecke nimmt
        // den Rest, der letzte Knopf endet am rechten Rand (minus 4).
        let geo = { let w = &g.windows[tw].widgets[ti]; g.tb_geom(0, 0, 300, 34, w) };
        assert_eq!((geo[0].1, geo[0].2), (4, 32));
        assert_eq!(geo[1].2 - geo[1].1, 11);
        assert_eq!(geo[4].2, 300 - 4);
        // Klick, Kippen, Sperren.
        g.tb_klick(tw, ti, 2);
        assert_eq!(g.toolbar_clicked(tb).unwrap(), 2);
        g.tb_klick(tw, ti, 1);                                  // Trenner: nichts
        assert_eq!(g.toolbar_clicked(tb).unwrap(), 2);
        g.toolbar_checkable(tb, 0, true).unwrap();
        g.tb_klick(tw, ti, 0);
        assert!(g.toolbar_checked(tb, 0).unwrap());
        g.toolbar_enable(tb, 4, false).unwrap();
        g.windows[tw].widgets[ti].leiste.as_mut().unwrap().geklickt = -1;
        g.tb_klick(tw, ti, 4);
        assert_eq!(g.toolbar_clicked(tb).unwrap(), -1);
        assert!(g.toolbar_set_checked(tb, 2, true).is_err());   // nicht kippbar
        // Rundweg ueber JSON.
        let j = g.widget_json(&g.windows[tw].widgets[ti]);
        let w2 = Gui::widget_from_json(&j).unwrap();
        let l2 = w2.leiste.as_ref().unwrap();
        assert_eq!(l2.eintraege.len(), 5);
        assert_eq!(l2.eintraege[2].symbol, "start");
        assert!(l2.eintraege[0].an && l2.eintraege[4].aus);
        assert_eq!(l2.eintraege[3].art, 2);
    }

    // Stufe 28: Statusleiste, Pfadleiste, Ueberlauf der Werkzeugleiste --
    // ohne Grafik ueber die Geometrie-Quellen.
    #[test]
    fn statusleiste_pfadleiste_ueberlauf() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 600, 200);
        // Statusleiste: feste Felder zuerst, der Rest geht an Breite 0.
        let sb = g.statusbar(win, 0, 0, 500, 24, "Bereit.".into()).unwrap();
        assert_eq!(g.statusbar_add(sb, "Zeile 1".into(), 120, "rechts").unwrap(), 1);
        assert_eq!(g.statusbar_add(sb, "".into(), 0, "").unwrap(), 2);
        assert!(g.statusbar_add(sb, "".into(), 0, "oben").is_err());
        let (sw, si) = Gui::dec_widget(sb);
        let geo = { let w = &g.windows[sw].widgets[si]; g.sb_geom(0, 500, w) };
        assert_eq!(geo, vec![(0, 0, 190), (1, 190, 310), (2, 310, 500)]);
        g.set_text(sb, "Gesichert.".into()).unwrap();
        assert_eq!(g.statusbar_text(sb, 0).unwrap(), "Gesichert.");
        // Pfadleiste: passt alles, kein "..."; zu eng, fallen die vorderen weg.
        let pf = g.breadcrumb(win, 0, 40, 400, 24).unwrap();
        g.breadcrumb_set(pf, vec!["projekt".into(), "quelle".into(), "spiel.dh".into()]).unwrap();
        let (pw, pi) = Gui::dec_widget(pf);
        g.windows[pw].widgets[pi].pfad.as_mut().unwrap().breiten = vec![60, 60, 60];
        let (geo, punkte) = { let w = &g.windows[pw].widgets[pi]; g.pf_geom(0, 400, w) };
        assert!(punkte.is_none());
        assert_eq!(geo.len(), 3);
        let (geo, punkte) = { let w = &g.windows[pw].widgets[pi]; g.pf_geom(0, 190, w) };
        assert!(punkte.is_some());
        assert_eq!(geo.last().unwrap().0, 2);
        assert!(geo[0].0 >= 1);
        // Werkzeugleiste: zu eng -> versteckte Eintraege und ein >>-Knopf.
        let tb = g.toolbar(win, 0, 80, 120, 34).unwrap();
        for s in ["neu", "oeffnen", "sichern", "start", "stopp"] {
            g.toolbar_add(tb, Some(s.into()), -1, s.into(), String::new()).unwrap();
        }
        let (tw, ti) = Gui::dec_widget(tb);
        let (sicht, versteckt, chev) = { let w = &g.windows[tw].widgets[ti]; g.tb_layout(0, 80, 120, 34, w) };
        assert!(chev.is_some());
        assert_eq!(sicht.len() + versteckt.len(), 5);
        assert!(!versteckt.is_empty());
        let (_, versteckt, chev) = { let w = &g.windows[tw].widgets[ti]; g.tb_layout(0, 80, 600, 34, w) };
        assert!(versteckt.is_empty() && chev.is_none());
    }

    // Jedes Sinnbild der Liste hat eine eigene Zeichnung -- ein Name, der in
    // den Rueckfall faellt, waere ein Punkt statt eines Bildes.
    #[test]
    fn jedes_sinnbild_ist_gezeichnet() {
        let quelle = include_str!("gui.rs");
        let start = quelle.find("fn sinnbild(g: &mut Graphics").unwrap();
        let rumpf = &quelle[start..start + quelle[start..].find("\n}\n").unwrap()];
        for n in SINNBILDER {
            assert!(rumpf.contains(&format!("\"{}\"", n)), "Sinnbild {} hat keinen Zweig", n);
        }
    }

    // Tree-Modell (ohne Graphics): Knoten anlegen, Sichtbarkeit folgt expanded,
    // Auswahl/Label, Fehlerfaelle, CLEAR.
    #[test]
    fn tree_model_and_visibility() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 300, 300);
        let tr = g.tree(win, 0, 0, 200, 200).unwrap();
        let root = g.tree_add(tr, -1, "Wurzel".into()).unwrap();
        let a = g.tree_add(tr, root, "A".into()).unwrap();
        let _b = g.tree_add(tr, root, "B".into()).unwrap();
        let a1 = g.tree_add(tr, a, "A.1".into()).unwrap();
        assert_eq!((root, a, a1), (0, 1, 3));
        let vis = |g: &Gui| {
            let (wi, i) = Gui::dec_widget(tr);
            Gui::tree_visible(g.windows[wi].widgets[i].tree.as_ref().unwrap())
        };
        assert_eq!(vis(&g), vec![0]);                 // alles zu -> nur Wurzel
        g.tree_expand(tr, root, true).unwrap();
        assert_eq!(vis(&g), vec![0, 1, 2]);           // Wurzel, A, B (A noch zu)
        g.tree_expand(tr, a, true).unwrap();
        assert_eq!(vis(&g), vec![0, 1, 3, 2]);        // Wurzel, A, A.1, B
        g.tree_set_selected(tr, a1).unwrap();
        assert_eq!(g.tree_selected(tr).unwrap(), a1);
        assert_eq!(g.tree_label(tr, a1).unwrap(), "A.1");
        assert!(g.tree_add(tr, 99, "x".into()).is_err());   // ungueltiger parent
        assert!(g.tree_label(tr, 99).is_err());
        g.tree_clear(tr).unwrap();
        assert_eq!(g.tree_selected(tr).unwrap(), -1);
        assert_eq!(vis(&g), Vec::<usize>::new());
    }

    fn rows(n: usize) -> Vec<Vec<String>> {
        (0..n).map(|i| vec![format!("r{}", i), format!("{}", i)]).collect()
    }

    // Tabellen-Layout: 14 Zeilen ueberlaufen -> vertikale Scrollbar, korrekte
    // Body-Hoehe + Scroll-Maximum.
    #[test]
    fn gitter_einfuegen_und_kopieren() {
        let mut g = Gui::new();
        let win = g.new_window("T".into(), 0, 0, 360, 250);
        let t = g.grid(win, 0, 0, 300, 200, vec!["A".into(), "B".into()], 1).unwrap();
        g.table_col_type(t, 1, "zahl").unwrap();
        g.table_set_current(t, 0, 0).unwrap();
        // Ohne Anhaengen endet das Einfuegen an der letzten Zeile.
        assert_eq!(g.table_paste(t, "x\t1,5\ny\t2\n").unwrap(), 2);
        assert_eq!(g.table_row_count(t).unwrap(), 1);
        g.table_set_opt(t, "zeilen_anhaengen", 1.0).unwrap();
        g.table_set_current(t, 0, 0).unwrap();
        // "q" passt nicht in die Zahlenspalte und wird uebergangen.
        assert_eq!(g.table_paste(t, "p\tq\r\nr\t2").unwrap(), 3);
        assert_eq!(g.table_row_count(t).unwrap(), 2);
        assert_eq!(g.table_range(t).unwrap(), (0, 0, 1, 1));
        assert_eq!(g.table_copy(t).unwrap(), "p\t1,5\nr\t2");
        let n = g.windows[0].widgets[0].tbl.as_mut().unwrap().bereich_leeren();
        assert_eq!(n, 4);
        assert_eq!(g.table_get_cell(t, 1, 1).unwrap(), "");
        assert!(g.table_set_current(t, 5, 0).is_err());
        assert!(g.table_col_type(t, 0, "quatsch").is_err());
    }

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


#[cfg(test)]
mod formate_tests {
    use super::{markdown_aus, markdown_ein};
    use crate::schnitt::{DURCH, FETT, KURSIV, UNTER};

    fn rund(text: &str, st: &[u8]) {
        let z: Vec<char> = text.chars().collect();
        let md = markdown_aus(&z, st);
        let (t, s) = markdown_ein(&md);
        assert_eq!(t, text, "Text ueber {md:?}");
        assert_eq!(s, st, "Stile ueber {md:?}");
    }

    #[test]
    fn markdown_schreibt_nur_wechsel() {
        let z: Vec<char> = "ab cd".chars().collect();
        assert_eq!(markdown_aus(&z, &[FETT, FETT, 0, KURSIV, KURSIV]), "**ab** *cd*");
        assert_eq!(markdown_aus(&z, &[FETT | KURSIV; 5]), "***ab cd***");
        assert_eq!(markdown_aus(&z, &[UNTER, UNTER, 0, DURCH, DURCH]), "<u>ab</u> ~~cd~~");
    }

    #[test]
    fn markdown_rundweg() {
        rund("ab cd", &[FETT, FETT | KURSIV, KURSIV, 0, FETT]);
        rund("x", &[FETT | KURSIV | UNTER | DURCH]);
        rund("a*b~c\\d<u>e", &[0, FETT, 0, 0, 0, 0, 0, UNTER, 0, 0, 0]);
        rund("zwei\nzeilen", &[FETT; 11]);
        rund("", &[]);
        // fett endet, kursiv beginnt: ** und * stehen nebeneinander.
        rund("ab", &[FETT, KURSIV]);
        rund("abc", &[FETT | KURSIV, FETT, KURSIV]);
    }

    #[test]
    fn markdown_von_hand() {
        let (t, s) = markdown_ein("ein **fettes** und *schiefes* ~Wort~");
        assert_eq!(t, "ein fettes und schiefes ~Wort~");
        assert_eq!(s[4], FETT);
        assert_eq!(s[15], KURSIV);
        assert_eq!(s[3], 0);
    }
}

#[cfg(test)]
mod tabelle_anhaengen_tests {
    use super::{Cell, TableState};

    // Nach jedem Anhaengen muss die Ansicht dieselbe sein wie nach dem
    // vollen Neubau -- flach und als Baum, mit und ohne Filter/Sortierung,
    // mit auf- und zugeklappten Knoten.
    #[test]
    fn anhaengen_wie_neu_gebaut() {
        let mut zufall: u64 = 99;
        let mut naechste = |n: usize| { zufall = zufall.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((zufall >> 33) as usize) % n.max(1) };
        for runde in 0..60 {
            let mut t = TableState::default();
            t.baum = runde % 2 == 0;
            if runde % 5 == 1 { t.filters = vec!["a".to_string()]; }
            if runde % 7 == 3 { t.sort_col = 0; t.sort_desc = runde % 3 == 0; }
            for _ in 0..80 {
                let n = t.rows.len();
                let text = ["a1", "b2", "ab", "c"][naechste(4)].to_string();
                t.rows.push(vec![Cell::text(text)]);
                t.row_fg.push(-1);
                t.row_bg.push(-1);
                if t.baum {
                    let e = if n > 0 && naechste(3) > 0 { naechste(n) as i32 } else { -1 };
                    t.eltern.push(e);
                    t.offen.push(false);
                }
                t.zeile_angehaengt();
                // ab und zu einen Knoten auf- oder zuklappen (voller Neubau)
                if t.baum && n > 0 && naechste(5) == 0 {
                    let k = naechste(n + 1);
                    t.offen[k] = !t.offen[k];
                    t.rebuild_view();
                }
                let mut neu = TableState::default();
                neu.rows = t.rows.clone();
                neu.baum = t.baum;
                neu.filters = t.filters.clone();
                neu.sort_col = t.sort_col;
                neu.sort_desc = t.sort_desc;
                neu.eltern = t.eltern.clone();
                neu.offen = t.offen.clone();
                neu.rebuild_view();
                assert_eq!(t.view, neu.view, "runde {}", runde);
                if t.baum {
                    assert_eq!(t.hat_kinder, neu.hat_kinder, "runde {}", runde);
                    for &r in &t.view { assert_eq!(t.ebene[r], neu.ebene[r], "runde {}", runde); }
                }
            }
        }
    }
}

#[cfg(test)]
mod marken_tests {
    use super::zeilen_nachziehen;

    fn nach(alt: &str, neu: &str, z: &[usize]) -> Vec<usize> {
        let mut v = z.to_vec();
        zeilen_nachziehen(alt, neu, &mut v);
        v
    }

    #[test]
    fn marken_wandern_mit_ihrer_zeile() {
        let alt = "a\nb\nc\nd";
        // Zeile darueber eingefuegt: alles ab dort eine tiefer.
        assert_eq!(nach(alt, "a\nNEU\nb\nc\nd", &[0, 1, 3]), vec![0, 2, 4]);
        // Enter am Anfang der markierten Zeile: sie rutscht mit.
        assert_eq!(nach(alt, "a\n\nb\nc\nd", &[1]), vec![2]);
        // In der Zeile getippt: bleibt.
        assert_eq!(nach(alt, "a\nbx\nc\nd", &[1]), vec![1]);
        assert_eq!(nach(alt, "a\nxb\nc\nd", &[1]), vec![1]);
        // Zeile darueber geloescht: eine hoeher.
        assert_eq!(nach(alt, "b\nc\nd", &[2]), vec![1]);
        // Die markierte Zeile selbst geloescht: landet an der Stelle.
        assert_eq!(nach(alt, "a\nc\nd", &[1, 2]), vec![1, 1]);
        // Ruecktaste am Zeilenanfang (zusammengefuegt): eine hoeher.
        assert_eq!(nach(alt, "a\nbc\nd", &[2]), vec![1]);
        // Unveraendert.
        assert_eq!(nach(alt, alt, &[3]), vec![3]);
    }

    #[test]
    fn marken_hinter_der_aenderung_wie_von_vorn_gezaehlt() {
        // Die Verschiebung wird einmal gerechnet statt je Marke von vorn
        // gezaehlt -- an vielen zufaelligen Aenderungen muss sie dasselbe
        // ergeben wie das Zaehlen von vorn (die Fassung bis 2026-10-02).
        let mut zufall: u64 = 12345;
        let mut naechste = |n: usize| { zufall = zufall.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((zufall >> 33) as usize) % n.max(1) };
        let stuecke = ["ab", "\n", "x\ny", "\n\n", "", "zz"];
        for _ in 0..500 {
            let mut alt = String::new();
            for _ in 0..(5 + naechste(20)) { alt.push_str(stuecke[naechste(stuecke.len())]); alt.push('\n'); }
            let a: Vec<char> = alt.chars().collect();
            let von = naechste(a.len());
            let bis = von + naechste(a.len() - von + 1);
            let mut neu: String = a[..von].iter().collect();
            neu.push_str(stuecke[naechste(stuecke.len())]);
            neu.extend(a[bis..].iter());
            let zahl = alt.matches('\n').count() + 1;
            let marken: Vec<usize> = (0..zahl).collect();
            let ist = nach(&alt, &neu, &marken);
            // Von vorn gezaehlt fuer jede Marke hinter der Aenderung.
            let n_ch: Vec<char> = neu.chars().collect();
            let mut p = 0;
            while p < a.len() && p < n_ch.len() && a[p] == n_ch[p] { p += 1; }
            let mut e = 0;
            while e < a.len() - p && e < n_ch.len() - p && a[a.len() - 1 - e] == n_ch[n_ch.len() - 1 - e] { e += 1; }
            let ende_alt = a.len() - e;
            let delta = n_ch.len() as i64 - a.len() as i64;
            let mut starts = vec![0usize];
            for (i, &c) in a.iter().enumerate() { if c == '\n' { starts.push(i + 1); } }
            for (z, &o) in starts.iter().enumerate() {
                if o >= ende_alt && o > p || (o == ende_alt && o == p && delta > 0) {
                    let off = ((o as i64 + delta).max(0) as usize).min(n_ch.len());
                    let erwartet = n_ch[..off].iter().filter(|&&c| c == '\n').count();
                    assert_eq!(ist[z], erwartet, "alt {:?} neu {:?} zeile {}", alt, neu, z);
                }
            }
        }
    }

    #[test]
    fn marken_in_einer_grossen_aenderung() {
        // Umbenennen ueber die ganze Datei: Zeile 1 und 4 aendern sich, der
        // Bereich dazwischen gilt als geaendert. Die Marke auf Zeile 3
        // bleibt (vorher fiel sie auf Zeile 1).
        let alt = "kopf\nf quadrat\nx\ny = quadrat(1)\nfuss";
        let neu = "kopf\nf hoch2\nx\ny = hoch2(1)\nfuss";
        assert_eq!(nach(alt, neu, &[3, 2, 4]), vec![3, 2, 4]);
        // Zeilen kommen dazu UND aendern sich: die gemeinsame Teilfolge
        // findet die markierte Zeile wieder.
        let alt = "a\nquadrat\nb\nc\nquadrat\nd";
        let neu = "a\nhoch2\nNEU\nb\nc\nhoch2\nd";
        assert_eq!(nach(alt, neu, &[3]), vec![4]);
        // Die markierte Zeile gibt es nicht mehr: Anfang der Aenderung.
        let alt = "a\nx1\nweg\nx2\nz";
        let neu = "a\ny1\ny2\nneu\nmehr\nz";
        assert_eq!(nach(alt, neu, &[2]), vec![1]);
    }
}

#[cfg(test)]
mod glanz_tests {
    use super::*;

    #[test]
    fn ein_haengendes_bild_laesst_den_streif_nicht_springen() {
        let mut g = Glanz::neu();
        g.dauer = 2.2;
        g.t = 0.0;
        // gleichmaessig: 1/60 s je Bild
        g.weiter(1.0 / 60.0);
        assert!((g.t - 1.0 / 60.0).abs() < 1e-6);
        // ein Bild haengt 614 ms -- der Streif geht nur 50 ms weiter
        g.weiter(0.614);
        assert!((g.t - (1.0 / 60.0 + GLANZ_SCHRITT_MAX)).abs() < 1e-6);
        // Gegenprobe: ohne Grenze waere er um mehr als ein Viertel gesprungen
        assert!(0.614 / g.dauer > 0.25);
        // eine negative Zeit schiebt nicht zurueck
        let vorher = g.t;
        g.weiter(-1.0);
        assert_eq!(g.t, vorher);
        // am Ende ist er vorbei
        for _ in 0..200 { g.weiter(1.0 / 60.0); }
        assert!(g.t < 0.0);
    }
}
