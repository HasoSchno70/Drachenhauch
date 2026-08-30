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
use crate::value::Rueckruf;

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
            ("widget_border", 0x9AA0A8), ("text_fg", 0x202428), ("muted_fg", 0x90969C),
            ("accent", 0x2A7DE1), ("close_hover", 0xC04848)])),
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

/// Zwei Farben mischen (`t` = 0 ganz `a`, 1 ganz `b`). Fuer Uebergaenge, die
/// einem Wert folgen -- etwa die Rinne des Kippschalters, die sich beim
/// Umlegen einfaerbt.
fn mischen(a: i64, b: i64, t: f64) -> i64 {
    let t = t.clamp(0.0, 1.0);
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
            | Kind::Toolbar | Kind::Progress | Kind::Image | Kind::Canvas)
    }

    /// Widgets, die Text entgegennehmen -- dort darf die Leertaste NICHT
    /// aktivieren, sie ist ein Zeichen.
    fn nimmt_text(self) -> bool {
        matches!(self, Kind::TextInput | Kind::TextArea)
    }
}

const DROPDOWN_ITEM_H: i32 = 22;
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
    zebra: bool,
    grid: bool,
    scroll_x: i32, scroll_y: i32,
    drag_v: bool, drag_h: bool, drag_off: i32,
    selected: i32, hover_row: i32, clicked_row: i32,
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
}

impl Default for TableState {
    fn default() -> Self {
        TableState {
            headers: vec![], rows: vec![], col_widths: None, col_align: vec![],
            row_fg: vec![], row_bg: vec![],
            row_h: TBL_ROW_H, header_h: TBL_HEADER_H,
            zebra: false, grid: true,
            scroll_x: 0, scroll_y: 0, drag_v: false, drag_h: false, drag_off: 0,
            selected: -1, hover_row: -1, clicked_row: -1, hover_col: -1, clicked_col: -1,
            frozen: 0, col_order: vec![], reorderable: false, head_drag: None,
            multi: false, sel_rows: vec![], sel_anchor: -1,
            edit_row: -1, edit_col: -1, edit_text: String::new(), edit_caret: 0,
            edit_anchor: 0, edit_scroll: 0, edit_maus_sperre: false,
            col_edit: vec![],
            drag_col: -1, drag_col_w: 0, drag_col_x: 0, resizable_cols: true,
            sort_col: -1, sort_desc: false, sortable: true,
            filters: vec![], filter_row: false, filter_focus: -1,
            view: vec![],
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
        self.header_h + if self.filter_row { TBL_FILTER_H } else { 0 }
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
}

// Baum-Zustand (Kind::Tree): flache Knotenliste, Knoten-id == Index (stabil bis
// GUI_TREE_CLEAR). `parent` = -1 fuer Wurzelknoten. Sichtbarkeit ergibt sich aus
// den `expanded`-Flags der Vorfahren (siehe tree_visible).
#[derive(Default)]
pub struct TreeState {
    nodes: Vec<TreeNode>,
    selected: i32,   // Knoten-id oder -1
    hover: i32,      // gehoverte Knoten-id oder -1
    scroll: i32,     // vertikaler Pixel-Offset
}

#[derive(Default)]
struct TreeNode {
    label: String,
    parent: i32,
    level: i32,
    expanded: bool,
    has_children: bool,
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
    ov: HashMap<String, i64>,
    tbl: Option<Box<TableState>>,   // nur fuer Kind::Table
    tree: Option<Box<TreeState>>,   // nur fuer Kind::Tree
    // Laufzeit-Lifecycle (Tombstone -- Indizes/Handles bleiben stabil): `alive`
    // = nicht zerstoert, `visible` = wird gezeichnet + interaktiv.
    alive: bool,
    visible: bool,
    /// Rund statt eckig zeichnen (Transport-Knoepfe, Werkzeug-Kreise).
    rund: bool,
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
    windows: Vec<Window>,        // stabile Indizes (Handles!)
    z_order: Vec<usize>,         // Zeichen-/Hit-Reihenfolge (umordbar)
    focus_window: Option<usize>,
    focus_widget: Option<(usize, usize)>,
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
    open_menu: Option<(usize, usize)>,       // offenes Menueleisten-Dropdown (win, menu)
    context_open: Option<(usize, usize, i32, i32)>,  // Kontextmenue (win, menu, x, y)
    was_right_down: bool,                    // Rechtsklick-Flankenerkennung
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
    pending: Vec<Rueckruf>,
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
            skins: HashMap::new(),
            active_slider: None,
            active_knob: None, active_split: None, split_off: 0,
            open_dropdown: None, active_table: None, table_press: None, press_origin: None,
            editing_table: None, last_click: None, dbl_click: false,
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
    pub fn take_pending(&mut self) -> Vec<Rueckruf> { std::mem::take(&mut self.pending) }

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
        // `color` wird hier BEWUSST nicht aus dem Thema gefuellt. Frueher wurde
        // die Textfarbe beim Anlegen kopiert und damit eingefroren: wer danach
        // das Thema wechselte, hatte Beschriftungen in der Farbe des alten --
        // im hellen Thema also weisse Schrift auf weissem Grund. -1 heisst
        // "dem Thema folgen" und wird erst beim Zeichnen aufgeloest.
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
            on_click: None, on_change: None, ov: HashMap::new(), tbl: None, tree: None,
            on_hover: None, on_leave: None, on_focus: None, on_blur: None,
            was_hovered: false, was_focused: false,
            alive: true, visible: true, rund: false,
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
        let cs = self.m("check_size");
        let mut wd = Self::blank(Kind::Checkbox, x, y, cs, cs); wd.text = label; wd.checked = default;
        self.add_widget(win, "GUI_CHECKBOX", wd)
    }
    /// Kippschalter (An/Aus-Pille). Zustand liegt wie beim Kaestchen in
    /// `checked`, also lesbar mit GUI_CHECKED und setzbar mit GUI_SET_CHECKED.
    /// `value` fuehrt die Schiebe-Animation (0 = aus, 1 = an).
    pub fn toggle(&mut self, win: i64, label: String, x: i32, y: i32, default: bool) -> Result<i64, String> {
        let h = self.m("check_size").max(14);
        let mut wd = Self::blank(Kind::Toggle, x, y, h * 2, h);
        wd.text = label;
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
        // Keine Laengenpruefung mehr: die Spaltenzahl ergibt sich aus der
        // BREITESTEN Angabe (TableState::n_cols), kuerzere Zeilen sind einfach
        // leer. Vorher war jede Abweichung ein Fehler -- und wer die Zeilen VOR
        // dem Kopf setzte, umging die Pruefung ganz und brachte das Zeichnen
        // zum Absturz. Ueber die eine Quelle kann kein Index danebengreifen.
        let t = self.tbl_mut(h, "GUI_TABLE_HEADERS")?;
        t.headers = headers;
        t.rebuild_view();
        Ok(())
    }
    pub fn table_set_rows(&mut self, h: i64, rows: Vec<Vec<String>>) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_ROWS")?;
        let old_n = t.rows.len();
        t.rows = rows.into_iter()
            .map(|r| r.into_iter().map(Cell::text).collect())
            .collect();
        t.row_fg = vec![-1; t.rows.len()];
        t.row_bg = vec![-1; t.rows.len()];
        if t.selected >= t.rows.len() as i32 { t.selected = -1; }
        // Die Auswahl zeigte sonst auf Zeilen, die es nicht mehr gibt.
        t.sel_rows.retain(|&r| r < t.rows.len() as i32);
        t.sel_anchor = -1;
        // Schrumpft die Tabelle, muss der Scroll zurueck -- sonst bliebe eine
        // weit heruntergescrollte Tabelle bis zur naechsten Mausbewegung leer.
        // Nur beim Schrumpfen, sonst waere ein periodisch aktualisiertes
        // Scoreboard nie scrollbar.
        if t.rows.len() < old_n { t.scroll_y = 0; }
        t.rebuild_view();
        Ok(())
    }
    pub fn table_set_col_widths(&mut self, h: i64, widths: Option<Vec<i32>>) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_COL_WIDTHS")?;
        if let Some(ref cw) = widths {
            let n = t.n_cols();
            if n != 0 && cw.len() != n {
                return Err(format!("GUI_TABLE_COL_WIDTHS: {} Breiten, die Tabelle hat {} Spalten",
                                   cw.len(), n));
            }
        }
        t.col_widths = widths;
        Ok(())
    }

    // --- Zellen einzeln ------------------------------------------------------

    /// Zeile/Spalte pruefen und melden, WELCHE Angabe daneben lag -- ein blosses
    /// "Index ungueltig" laesst einen beide durchsuchen.
    fn tbl_rc(t: &TableState, r: i64, c: i64, fn_: &str) -> Result<(usize, usize), String> {
        if r < 0 || r >= t.rows.len() as i64 {
            return Err(format!("{}: Zeile {} gibt es nicht (0..{})",
                               fn_, r, t.rows.len().saturating_sub(1)));
        }
        let n = t.n_cols();
        if c < 0 || c >= n as i64 {
            return Err(format!("{}: Spalte {} gibt es nicht (0..{})",
                               fn_, c, n.saturating_sub(1)));
        }
        Ok((r as usize, c as usize))
    }

    pub fn table_set_cell(&mut self, h: i64, r: i64, c: i64, text: String) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SET_CELL")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_SET_CELL")?;
        t.cell_mut(r, c).unwrap().text = text;
        t.rebuild_view();   // Filter/Sortierung haengen am Text
        Ok(())
    }
    pub fn table_get_cell(&self, h: i64, r: i64, c: i64) -> Result<String, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_GET_CELL")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_GET_CELL")?;
        Ok(t.cell(r, c).map(|x| x.text.clone()).unwrap_or_default())
    }
    /// Farben je Zelle. -1 laesst die jeweilige Farbe beim Thema bzw. bei der
    /// Zeilenfarbe -- so faerbt man nur den Vordergrund, ohne die Flaeche zu
    /// verlieren.
    pub fn table_cell_color(&mut self, h: i64, r: i64, c: i64, fg: i64, bg: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_COLOR")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_COLOR")?;
        let cell = t.cell_mut(r, c).unwrap();
        cell.fg = fg;
        cell.bg = bg;
        Ok(())
    }
    pub fn table_cell_align(&mut self, h: i64, r: i64, c: i64, wie: &str) -> Result<(), String> {
        let a = parse_align(wie).ok_or_else(|| format!(
            "GUI_TABLE_CELL_ALIGN: '{}' unbekannt -- links, mitte oder rechts", wie))?;
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_ALIGN")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_ALIGN")?;
        t.cell_mut(r, c).unwrap().align = a;
        Ok(())
    }
    /// Ganze Zeile faerben -- ein Aufruf statt einer je Spalte.
    pub fn table_row_color(&mut self, h: i64, r: i64, fg: i64, bg: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_ROW_COLOR")?;
        if r < 0 || r >= t.rows.len() as i64 {
            return Err(format!("GUI_TABLE_ROW_COLOR: Zeile {} gibt es nicht (0..{})",
                               r, t.rows.len().saturating_sub(1)));
        }
        let r = r as usize;
        while t.row_fg.len() <= r { t.row_fg.push(-1); }
        while t.row_bg.len() <= r { t.row_bg.push(-1); }
        t.row_fg[r] = fg;
        t.row_bg[r] = bg;
        Ok(())
    }
    pub fn table_cell_kind(&mut self, h: i64, r: i64, c: i64, art: &str) -> Result<(), String> {
        let k = CellKind::parse(art).ok_or_else(|| format!(
            "GUI_TABLE_CELL_KIND: '{}' unbekannt -- text, bild, haken, balken, knopf", art))?;
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_KIND")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_KIND")?;
        t.cell_mut(r, c).unwrap().kind = k;
        Ok(())
    }
    /// Bild in die Zelle. Setzt die Art gleich mit -- sonst muesste man zwei
    /// Aufrufe machen und der erste allein bliebe wirkungslos.
    pub fn table_cell_image(&mut self, h: i64, r: i64, c: i64, bild: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_IMAGE")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_IMAGE")?;
        let cell = t.cell_mut(r, c).unwrap();
        cell.image = bild;
        cell.kind = CellKind::Image;
        Ok(())
    }
    /// Wert einer Haken- oder Balkenzelle (Haken: 0/1, Balken: 0..1).
    pub fn table_cell_value(&mut self, h: i64, r: i64, c: i64, v: f64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_VALUE")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_VALUE")?;
        t.cell_mut(r, c).unwrap().value = v;
        Ok(())
    }
    pub fn table_get_value(&self, h: i64, r: i64, c: i64) -> Result<f64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_GET_VALUE")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_GET_VALUE")?;
        Ok(t.cell(r, c).map(|x| x.value).unwrap_or(0.0))
    }
    pub fn table_col_align(&mut self, h: i64, c: i64, wie: &str) -> Result<(), String> {
        let a = parse_align(wie).ok_or_else(|| format!(
            "GUI_TABLE_COL_ALIGN: '{}' unbekannt -- links, mitte oder rechts", wie))?;
        let t = self.tbl_mut(h, "GUI_TABLE_COL_ALIGN")?;
        if c < 0 { return Err("GUI_TABLE_COL_ALIGN: Spalte darf nicht negativ sein".into()); }
        let c = c as usize;
        while t.col_align.len() <= c { t.col_align.push(0); }
        t.col_align[c] = a;
        Ok(())
    }

    // --- Zeilen anfuegen / entfernen ----------------------------------------

    pub fn table_add_row(&mut self, h: i64, zellen: Vec<String>) -> Result<i64, String> {
        let t = self.tbl_mut(h, "GUI_TABLE_ADD_ROW")?;
        t.rows.push(zellen.into_iter().map(Cell::text).collect());
        t.row_fg.push(-1);
        t.row_bg.push(-1);
        t.rebuild_view();
        Ok(t.rows.len() as i64 - 1)
    }
    pub fn table_remove_row(&mut self, h: i64, r: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_REMOVE_ROW")?;
        if r < 0 || r >= t.rows.len() as i64 {
            return Err(format!("GUI_TABLE_REMOVE_ROW: Zeile {} gibt es nicht (0..{})",
                               r, t.rows.len().saturating_sub(1)));
        }
        let r = r as usize;
        t.rows.remove(r);
        if r < t.row_fg.len() { t.row_fg.remove(r); }
        if r < t.row_bg.len() { t.row_bg.remove(r); }
        // Sonst zeigt die Auswahl danach auf eine ANDERE Zeile als vorher --
        // alle Indizes hinter der geloeschten ruecken auf.
        if t.selected == r as i32 { t.selected = -1; }
        else if t.selected > r as i32 { t.selected -= 1; }
        t.sel_rows.retain(|&x| x != r as i32);
        for x in t.sel_rows.iter_mut() { if *x > r as i32 { *x -= 1; } }
        if t.sel_anchor == r as i32 { t.sel_anchor = -1; }
        else if t.sel_anchor > r as i32 { t.sel_anchor -= 1; }
        t.rebuild_view();
        Ok(())
    }
    pub fn table_clear(&mut self, h: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CLEAR")?;
        t.rows.clear();
        t.row_fg.clear();
        t.row_bg.clear();
        t.sel_rows.clear();
        t.sel_anchor = -1;
        t.selected = -1;
        t.hover_row = -1;
        t.scroll_x = 0;
        t.scroll_y = 0;
        t.view.clear();
        Ok(())
    }

    // --- Tabellenweite Einstellungen ----------------------------------------

    /// Ein Setter statt einem Builtin je Schalter -- wie bei `chart`. Ein
    /// unbekannter Schluessel zaehlt die gueltigen auf, statt still nichts zu tun.
    pub fn table_set_opt(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SET")?;
        let n = wert as i32;
        match key.to_lowercase().as_str() {
            // Untergrenze: eine Zeilenhoehe von 0 waere eine Division durch null
            // im Treffertest, eine negative ein Absturz beim Zeichnen.
            "zeilenhoehe" | "row_height" => t.row_h = n.clamp(8, 400),
            "kopfhoehe" | "header_height" => t.header_h = n.clamp(0, 200),
            "zebra" => t.zebra = n != 0,
            "gitter" | "grid" => t.grid = n != 0,
            "filterzeile" | "filter_row" => { t.filter_row = n != 0; if n == 0 { t.filter_focus = -1; } }
            "sortierbar" | "sortable" => t.sortable = n != 0,
            "spalten_ziehbar" | "resizable_cols" => t.resizable_cols = n != 0,
            "feste_spalten" | "frozen" => t.frozen = n.max(0),
            "spalten_verschiebbar" | "reorderable" => t.reorderable = n != 0,
            "mehrfachauswahl" | "multi_select" => {
                t.multi = n != 0;
                // Beim Einschalten die bisherige Einzelauswahl uebernehmen,
                // sonst waere sie nach dem Umschalten ploetzlich weg.
                if t.multi {
                    t.sel_rows.clear();
                    if t.selected >= 0 { t.sel_rows.push(t.selected); }
                }
            }
            _ => return Err(format!(
                "GUI_TABLE_SET: '{}' unbekannt -- zeilenhoehe, kopfhoehe, zebra, gitter, \
filterzeile, sortierbar, spalten_ziehbar, feste_spalten, spalten_verschiebbar, mehrfachauswahl", key)),
        }
        Ok(())
    }
    pub fn table_get_opt(&self, h: i64, key: &str) -> Result<f64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_GET")?;
        Ok(match key.to_lowercase().as_str() {
            "zeilenhoehe" | "row_height" => t.row_h as f64,
            "kopfhoehe" | "header_height" => t.header_h as f64,
            "zebra" => if t.zebra { 1.0 } else { 0.0 },
            "gitter" | "grid" => if t.grid { 1.0 } else { 0.0 },
            "filterzeile" | "filter_row" => if t.filter_row { 1.0 } else { 0.0 },
            "sortierbar" | "sortable" => if t.sortable { 1.0 } else { 0.0 },
            "spalten_ziehbar" | "resizable_cols" => if t.resizable_cols { 1.0 } else { 0.0 },
            "feste_spalten" | "frozen" => t.frozen as f64,
            "spalten_verschiebbar" | "reorderable" => if t.reorderable { 1.0 } else { 0.0 },
            "mehrfachauswahl" | "multi_select" => if t.multi { 1.0 } else { 0.0 },
            "spalten" | "columns" => t.n_cols() as f64,
            _ => return Err(format!(
                "GUI_TABLE_GET: '{}' unbekannt -- zeilenhoehe, kopfhoehe, zebra, gitter, \
filterzeile, sortierbar, spalten_ziehbar, feste_spalten, spalten_verschiebbar, mehrfachauswahl, spalten", key)),
        })
    }

    // --- Mehrfachauswahl -----------------------------------------------------

    // --- Spalten-Reihenfolge -------------------------------------------------

    /// Spalte von einer Anzeige-Position auf eine andere schieben.
    pub fn table_move_col(&mut self, h: i64, von: i64, nach: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_MOVE_COL")?;
        let n = t.n_cols() as i64;
        if von < 0 || von >= n || nach < 0 || nach >= n {
            return Err(format!("GUI_TABLE_MOVE_COL: Position ausserhalb 0..{}", n.saturating_sub(1)));
        }
        t.move_col(von as usize, nach as usize);
        Ok(())
    }
    /// Welche DATENspalte steht an der Anzeige-Position `pos`?
    pub fn table_col_at(&self, h: i64, pos: i64) -> Result<i64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_COL_AT")?;
        Ok(t.order().get(pos.max(0) as usize).map(|&c| c as i64).unwrap_or(-1))
    }
    /// An welcher Anzeige-Position steht die DATENspalte `c`?
    pub fn table_col_pos(&self, h: i64, c: i64) -> Result<i64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_COL_POS")?;
        Ok(t.order().iter().position(|&x| x as i64 == c).map(|p| p as i64).unwrap_or(-1))
    }
    /// Reihenfolge zuruecksetzen.
    pub fn table_reset_cols(&mut self, h: i64) -> Result<(), String> {
        self.tbl_mut(h, "GUI_TABLE_RESET_COLS")?.col_order.clear();
        Ok(())
    }

    pub fn table_sel_count(&self, h: i64) -> Result<i64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_SEL_COUNT")?;
        Ok(if t.multi { t.sel_rows.len() as i64 }
           else if t.selected >= 0 { 1 } else { 0 })
    }
    /// i-te gewaehlte DATENzeile, in der Reihenfolge des Anklickens.
    pub fn table_sel_row(&self, h: i64, i: i64) -> Result<i64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_SEL_ROW")?;
        if !t.multi { return Ok(if i == 0 { t.selected as i64 } else { -1 }); }
        Ok(t.sel_rows.get(i.max(0) as usize).copied().unwrap_or(-1) as i64)
    }
    pub fn table_is_selected(&self, h: i64, r: i64) -> Result<bool, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_IS_SELECTED")?.ist_gewaehlt(r as i32))
    }
    /// Zeile zur Auswahl hinzunehmen bzw. herausnehmen (programmgesteuert).
    pub fn table_select(&mut self, h: i64, r: i64, an: bool) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SELECT")?;
        if r < 0 || r >= t.rows.len() as i64 {
            return Err(format!("GUI_TABLE_SELECT: Zeile {} gibt es nicht (0..{})",
                               r, t.rows.len().saturating_sub(1)));
        }
        let r = r as i32;
        let drin = t.sel_rows.contains(&r);
        if an != drin { t.auswahl_umschalten(r); }
        if an { t.selected = r; }
        Ok(())
    }
    pub fn table_clear_selection(&mut self, h: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CLEAR_SELECTION")?;
        t.auswahl_setzen(-1);
        Ok(())
    }

    // --- Zellen bearbeiten ---------------------------------------------------

    /// Spalte fuer die Bearbeitung freigeben. Ohne Freigabe passiert beim
    /// Doppelklick nichts -- eine Tabelle, in der man versehentlich ueberall
    /// hineinschreiben kann, waere schlimmer als eine ohne Bearbeitung.
    pub fn table_col_edit(&mut self, h: i64, c: i64, an: bool) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_COL_EDIT")?;
        if c < 0 { return Err("GUI_TABLE_COL_EDIT: Spalte darf nicht negativ sein".into()); }
        let c = c as usize;
        while t.col_edit.len() <= c { t.col_edit.push(false); }
        t.col_edit[c] = an;
        Ok(())
    }
    /// Zeile der Zelle, die gerade bearbeitet wird (-1 = keine).
    pub fn table_editing_row(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_EDITING_ROW")?.edit_row as i64)
    }
    pub fn table_editing_col(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_EDITING_COL")?.edit_col as i64)
    }

    // --- Sortieren und Filtern ----------------------------------------------

    /// Programmgesteuert sortieren. Spalte < 0 hebt die Sortierung auf.
    pub fn table_sort(&mut self, h: i64, col: i64, desc: bool) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SORT")?;
        let n = t.n_cols() as i64;
        t.sort_col = if col >= 0 && col < n { col as i32 } else { -1 };
        t.sort_desc = desc;
        t.rebuild_view();
        Ok(())
    }
    pub fn table_sort_col(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_SORT_COL")?.sort_col as i64)
    }
    pub fn table_sort_desc(&self, h: i64) -> Result<bool, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_SORT_DESC")?.sort_desc)
    }
    /// Filter je Spalte setzen (leerer Text = kein Filter).
    pub fn table_filter(&mut self, h: i64, col: i64, text: String) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_FILTER")?;
        if col < 0 { return Err("GUI_TABLE_FILTER: Spalte darf nicht negativ sein".into()); }
        let c = col as usize;
        while t.filters.len() <= c { t.filters.push(String::new()); }
        t.filters[c] = text;
        t.rebuild_view();
        t.scroll_y = 0;
        Ok(())
    }
    pub fn table_get_filter(&self, h: i64, col: i64) -> Result<String, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_GET_FILTER")?;
        Ok(t.filters.get(col.max(0) as usize).cloned().unwrap_or_default())
    }
    /// Wieviele Zeilen sind nach Filter/Sortierung sichtbar?
    pub fn table_view_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_VIEW_COUNT")?.view.len() as i64)
    }
    /// Ansichtszeile -> Datenzeile. Damit laesst sich die Tabelle in der
    /// SICHTBAREN Reihenfolge durchgehen, ohne dass die Daten umgestellt
    /// werden muessten.
    pub fn table_view_row(&self, h: i64, i: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_VIEW_ROW")?.data_row(i as i32) as i64)
    }

    pub fn table_selected(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_SELECTED")?.selected as i64)
    }
    pub fn table_set_selected(&mut self, h: i64, row: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SET_SELECTED")?;
        let n = t.rows.len() as i64;
        let r = if row >= 0 && row < n { row as i32 } else { -1 };
        // Ueber auswahl_setzen, damit die Mehrfachauswahl nicht daneben
        // stehenbleibt und beide Angaben auseinander laufen.
        t.auswahl_setzen(r);
        Ok(())
    }
    pub fn table_clicked(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_CLICKED")?.clicked_row as i64)
    }
    pub fn table_clicked_col(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_CLICKED_COL")?.clicked_col as i64)
    }
    pub fn table_row_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_ROW_COUNT")?.rows.len() as i64)
    }

    // --- Tree (Baum-Ansicht) ------------------------------------------------
    fn tree_mut(&mut self, h: i64, fn_: &str) -> Result<&mut TreeState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Tree { return Err(format!("{}: Widget ist kein Baum", fn_)); }
        Ok(w.tree.as_mut().unwrap())
    }
    fn tree_ref(&self, h: i64, fn_: &str) -> Result<&TreeState, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::Tree { return Err(format!("{}: Widget ist kein Baum", fn_)); }
        Ok(w.tree.as_ref().unwrap())
    }
    pub fn tree(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Tree, x, y, w, h);
        wd.tree = Some(Box::new(TreeState { selected: -1, hover: -1, ..Default::default() }));
        self.add_widget(win, "GUI_TREE", wd)
    }
    /// Knoten anhaengen. parent = -1 (Wurzel) oder eine bestehende Knoten-id.
    /// Liefert die neue Knoten-id (== Index, stabil bis GUI_TREE_CLEAR).
    pub fn tree_add(&mut self, h: i64, parent: i64, label: String) -> Result<i64, String> {
        let t = self.tree_mut(h, "GUI_TREE_ADD")?;
        let n = t.nodes.len() as i64;
        if parent < -1 || parent >= n {
            return Err(format!("GUI_TREE_ADD: ungueltiger parent {}", parent));
        }
        let level = if parent < 0 { 0 } else { t.nodes[parent as usize].level + 1 };
        if parent >= 0 { t.nodes[parent as usize].has_children = true; }
        t.nodes.push(TreeNode { label, parent: parent as i32, level, expanded: false, has_children: false });
        Ok((t.nodes.len() - 1) as i64)
    }
    pub fn tree_clear(&mut self, h: i64) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_CLEAR")?;
        t.nodes.clear(); t.selected = -1; t.hover = -1; t.scroll = 0; Ok(())
    }
    pub fn tree_selected(&self, h: i64) -> Result<i64, String> {
        Ok(self.tree_ref(h, "GUI_TREE_SELECTED")?.selected as i64)
    }
    pub fn tree_set_selected(&mut self, h: i64, node: i64) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_SET_SELECTED")?;
        let n = t.nodes.len() as i64;
        t.selected = if node >= 0 && node < n { node as i32 } else { -1 };
        Ok(())
    }
    pub fn tree_label(&self, h: i64, node: i64) -> Result<String, String> {
        let t = self.tree_ref(h, "GUI_TREE_LABEL")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_LABEL: ungueltige Knoten-id {}", node));
        }
        Ok(t.nodes[node as usize].label.clone())
    }
    pub fn tree_expand(&mut self, h: i64, node: i64, flag: bool) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_EXPAND")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_EXPAND: ungueltige Knoten-id {}", node));
        }
        t.nodes[node as usize].expanded = flag; Ok(())
    }

    /// Sichtbare Knoten (id-Liste) in Anzeigereihenfolge: Vorfahren-expanded.
    fn tree_visible(t: &TreeState) -> Vec<usize> {
        let mut out = Vec::new();
        Self::tree_collect(t, -1, &mut out);
        out
    }
    fn tree_collect(t: &TreeState, parent: i32, out: &mut Vec<usize>) {
        for i in 0..t.nodes.len() {
            if t.nodes[i].parent == parent {
                out.push(i);
                if t.nodes[i].expanded && t.nodes[i].has_children {
                    Self::tree_collect(t, i as i32, out);
                }
            }
        }
    }

    fn tree_hover(&mut self, wi: usize, idx: usize, my: i32, g: &mut Graphics) {
        let (_ax, ay, _w, h) = self.abs_rect(wi, &self.windows[wi].widgets[idx]);
        let vis = Self::tree_visible(self.windows[wi].widgets[idx].tree.as_ref().unwrap());
        let max_scroll = (vis.len() as i32 * TREE_ROW_H - (h - 2)).max(0);
        let wheel = g.pop_mouse_wheel();
        let t = self.windows[wi].widgets[idx].tree.as_mut().unwrap();
        t.scroll = t.scroll.clamp(0, max_scroll);
        if wheel != 0 { t.scroll = (t.scroll - wheel as i32 * TREE_ROW_H).clamp(0, max_scroll); }
        let row = (my - (ay + 1) + t.scroll) / TREE_ROW_H;
        t.hover = if row >= 0 && (row as usize) < vis.len() { vis[row as usize] as i32 } else { -1 };
    }

    fn tree_press(&mut self, wi: usize, idx: usize, mx: i32, my: i32) {
        let (ax, ay, _w, _h) = self.abs_rect(wi, &self.windows[wi].widgets[idx]);
        let t = self.windows[wi].widgets[idx].tree.as_ref().unwrap();
        let vis = Self::tree_visible(t);
        let row = (my - (ay + 1) + t.scroll) / TREE_ROW_H;
        if row < 0 || row as usize >= vis.len() { return; }
        let ni = vis[row as usize];
        let level = t.nodes[ni].level;
        let has_children = t.nodes[ni].has_children;
        let toggle_x = ax + 4 + level * TREE_INDENT;
        let on_toggle = has_children && mx >= toggle_x && mx < toggle_x + TREE_TOGGLE_W;
        if on_toggle {
            let t = self.windows[wi].widgets[idx].tree.as_mut().unwrap();
            let e = t.nodes[ni].expanded; t.nodes[ni].expanded = !e;
        } else {
            let changed = {
                let t = self.windows[wi].widgets[idx].tree.as_mut().unwrap();
                if t.selected != ni as i32 { t.selected = ni as i32; true } else { false }
            };
            if changed {
                let f = self.windows[wi].widgets[idx].on_change.clone();
                if let Some(f) = f { self.pending.push(f); }
            }
        }
    }

    fn table_geom(&self, wi: usize, idx: usize) -> TGeom {
        let w = &self.windows[wi].widgets[idx];
        let (ax, ay, ww, hh) = self.abs_rect(wi, w);
        let t = w.tbl.as_ref().unwrap();
        let n_cols = t.n_cols();
        // SICHTBARE Zeilen -- Filter und Sortierung bestimmen, was gezeichnet
        // und getroffen wird. Der Rest der Geometrie kennt keinen Unterschied.
        let n_rows = t.view.len();
        let row_h = t.row_h;
        let header_h = t.head_total();
        // Mehr feste Spalten als vorhanden waeren ein leerer Scrollbereich --
        // deckeln statt darauf zu vertrauen, dass niemand es versucht.
        let frozen = (t.frozen.max(0) as usize).min(n_cols);
        let order = t.order();
        // Breiten haengen an der DATENspalte, gebraucht werden sie in
        // ANZEIGE-Reihenfolge -- hier einmal umgeordnet, damit alles weitere
        // schlicht ueber die Position laufen kann.
        let roh: Vec<i32> = match &t.col_widths {
            Some(cw) if cw.len() == n_cols => cw.clone(),
            _ => {
                let avail = ww - TBL_SCROLL_W - 2;
                let per = if n_cols > 0 { (avail / n_cols as i32).max(TBL_MIN_COL_W) } else { 0 };
                vec![per; n_cols]
            }
        };
        let col_widths: Vec<i32> = order.iter().map(|&c| roh[c]).collect();
        let body_x = ax + 1;
        let body_y = ay + header_h;
        let body_w_raw = ww - 2;
        let body_h_raw = hh - header_h - 1;
        let total_w: i32 = col_widths.iter().sum();
        let total_h = n_rows as i32 * row_h;
        let mut need_v = total_h > body_h_raw;
        let need_h = total_w > body_w_raw - if need_v { TBL_SCROLL_W } else { 0 };
        if need_h && total_h > body_h_raw - TBL_SCROLL_W { need_v = true; }
        let body_w = body_w_raw - if need_v { TBL_SCROLL_W } else { 0 };
        let body_h = body_h_raw - if need_h { TBL_SCROLL_W } else { 0 };
        let frozen_w: i32 = col_widths.iter().take(frozen).sum();
        TGeom {
            ax, ay, w: ww, h: hh, n_cols, n_rows, col_widths, row_h, header_h,
            scroll_x: t.scroll_x, frozen, frozen_w, order,
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
            t.scroll_y = (t.scroll_y - wheel as i32 * gm.row_h).clamp(0, gm.max_scroll_y);
        }
        if !t.drag_v && !t.drag_h && over {
            let hv = (my - gm.body_y + t.scroll_y) / gm.row_h;
            // Nach aussen ist die Datenzeile gemeint -- eine sortierte Tabelle
            // wuerde sonst beim Hovern die falsche Zeile hervorheben.
            t.hover_row = if hv >= 0 && hv < gm.n_rows as i32 { t.data_row(hv) } else { -1 };
        }
    }

    fn table_press(&mut self, wi: usize, idx: usize, mx: i32, my: i32) {
        // Klick im laufenden Eingabefeld gehoert dem Editor -- sonst wuerde er
        // hier zusaetzlich als Zeilen-Klick zaehlen.
        if self.editing_table == Some((wi, idx)) {
            if let Some(r) = self.edit_cell_rect(wi, idx) {
                if Self::in_rect(mx, my, r) { return; }
            }
        }
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
        // Klick in den Kopfbereich: sortieren bzw. Filterzelle fokussieren.
        // Kommt VOR dem Zeilen-Treffer, sonst waere der Kopf tot.
        let (titel_h, filter_an) = {
            let t = self.windows[wi].widgets[idx].tbl.as_ref().unwrap();
            (t.header_h, t.filter_row)
        };
        if my >= gm.ay && my < gm.ay + titel_h {
            // Zuerst pruefen, ob die Maus auf einer SPALTENKANTE liegt --
            // sonst wuerde jeder Griff nach der Kante die Spalte sortieren.
            let kante = Self::edge_at(&gm, mx);
            if kante >= 0 {
                let breite = gm.col_widths[kante as usize];
                // `edge_at` liefert eine ANZEIGE-Position, die Breiten haengen
                // aber an der Datenspalte -- ohne diese Umrechnung zoege man
                // nach dem Umsortieren die falsche Spalte breit.
                let daten_c = gm.order[kante as usize] as i32;
                let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
                if t.resizable_cols {
                    t.drag_col = daten_c;
                    t.drag_col_w = breite;
                    t.drag_col_x = mx;
                    t.filter_focus = -1;
                    // Beim ersten Ziehen die berechneten Breiten festhalten,
                    // sonst haette die Tabelle gar keine eigenen Werte zum
                    // Aendern und spraenge beim Loslassen zurueck.
                    // Die berechneten Breiten festhalten -- in DATEN-Reihenfolge,
                    // denn so werden sie gespeichert.
                    if t.col_widths.is_none() {
                        let mut roh = vec![0; gm.n_cols];
                        for (p, &c) in gm.order.iter().enumerate() { roh[c] = gm.col_widths[p]; }
                        t.col_widths = Some(roh);
                    }
                    self.active_table = Some((wi, idx));
                    return;
                }
            }
            let pos = Self::pos_at(&gm, mx);
            let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
            t.filter_focus = -1;
            if pos >= 0 {
                // NICHT sofort sortieren: erst beim Loslassen steht fest, ob
                // die Geste ein Klick war oder ein Zug. Sonst wuerde jedes
                // Verschieben nebenbei auch noch sortieren.
                t.head_drag = Some((pos as usize, mx, false));
                self.active_table = Some((wi, idx));
            }
            return;
        }
        if filter_an && my >= gm.ay + titel_h && my < gm.ay + titel_h + TBL_FILTER_H {
            let c = Self::col_at(&gm, mx);
            let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
            t.filter_focus = c;
            // Das Tabellen-Widget bekommt den Tastaturfokus, damit die
            // Eingabe hier landet und nicht in einem Textfeld daneben.
            if c >= 0 { self.focus_widget = Some((wi, idx)); }
            return;
        }
        let hr = self.windows[wi].widgets[idx].tbl.as_ref().unwrap().hover_row;
        if hr >= 0 {
            // Spalte aus der x-Lage bestimmen -- ohne sie liesse sich ein
            // Knopf in Spalte 3 nicht von einem Haken in Spalte 1 trennen.
            let hc = Self::col_at(&gm, mx);
            self.windows[wi].widgets[idx].tbl.as_mut().unwrap().hover_col = hc;
            // Doppelklick auf eine bearbeitbare Textzelle -> Bearbeiten.
            if self.dbl_click && hc >= 0 {
                self.table_begin_edit(wi, idx, hr, hc);
                return;
            }
            self.table_press = Some((wi, idx, hr));
        }
    }

    /// Bearbeitung starten. Laeuft nur auf TEXT-Zellen einer freigegebenen
    /// Spalte: ein Haken schaltet ohnehin per Klick um, ein Balken oder Bild
    /// hat keinen sinnvollen Text zum Tippen.
    fn table_begin_edit(&mut self, wi: usize, idx: usize, r: i32, c: i32) {
        let t = match self.windows[wi].widgets[idx].tbl.as_mut() { Some(t) => t, None => return };
        if r < 0 || c < 0 || r as usize >= t.rows.len() { return; }
        if !t.col_editable(c as usize) { return; }
        let art = t.cell(r as usize, c as usize).map(|x| x.kind).unwrap_or(CellKind::Text);
        if art != CellKind::Text { return; }
        t.edit_text = t.cell(r as usize, c as usize).map(|x| x.text.clone()).unwrap_or_default();
        t.edit_caret = t.edit_text.chars().count() as i32;
        // Beim Oeffnen alles markiert -- so ersetzt das erste Tippen den
        // bisherigen Inhalt, statt ihn zu verlaengern. Genau das erwartet man,
        // wenn man eine Zelle zum Ueberschreiben aufmacht.
        t.edit_anchor = 0;
        t.edit_scroll = 0;
        t.edit_maus_sperre = true;
        t.edit_row = r;
        t.edit_col = c;
        t.selected = r;
        self.editing_table = Some((wi, idx));
        // Die Tabelle braucht den Tastaturfokus, sonst landen die Zeichen im
        // Textfeld daneben.
        self.focus_widget = Some((wi, idx));
    }

    /// Bearbeitung beenden. `uebernehmen` = Text in die Zelle schreiben.
    fn table_end_edit(&mut self, wi: usize, idx: usize, uebernehmen: bool) {
        let mut geaendert = false;
        {
            let t = match self.windows[wi].widgets[idx].tbl.as_mut() { Some(t) => t, None => return };
            if t.edit_row < 0 { return; }
            let (r, c) = (t.edit_row as usize, t.edit_col as usize);
            if uebernehmen {
                let neu = std::mem::take(&mut t.edit_text);
                if let Some(cell) = t.cell_mut(r, c) {
                    if cell.text != neu { cell.text = neu; geaendert = true; }
                }
            }
            t.edit_row = -1;
            t.edit_col = -1;
            t.edit_text.clear();
            t.edit_caret = 0;
            t.edit_anchor = 0;
            t.edit_scroll = 0;
            // Erst JETZT neu sortieren/filtern. Waehrend des Tippens waere die
            // Zeile bei jedem Zeichen weggesprungen.
            if geaendert { t.rebuild_view(); }
        }
        self.editing_table = None;
        if geaendert {
            let f = self.windows[wi].widgets[idx].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Linke Bildschirmkante der Spalte `c`. Feste Spalten scrollen nicht mit.
    /// EINE Quelle fuer Treffertest und Zeichnen.
    fn col_x(gm: &TGeom, c: usize) -> i32 {
        let bis = c.min(gm.col_widths.len());
        let mut x = gm.body_x + gm.col_widths[..bis].iter().sum::<i32>();
        if c >= gm.frozen { x -= gm.scroll_x; }
        x
    }

    /// Bereich, in dem die Spalte `c` sichtbar sein darf. Ohne dieses Clipping
    /// schoebe sich eine gescrollte Spalte unter den festen Block.
    fn col_clip(gm: &TGeom, c: usize) -> (i32, i32) {
        if c < gm.frozen {
            (gm.body_x, gm.frozen_w)
        } else {
            (gm.body_x + gm.frozen_w, (gm.body_w - gm.frozen_w).max(0))
        }
    }

    /// Liegt `mx` auf der rechten Kante einer Spalte? Liefert deren Index.
    /// Der Fangbereich ist bewusst breiter als ein Pixel -- eine Kante, die
    /// man auf den Punkt genau treffen muss, findet niemand.
    fn edge_at(gm: &TGeom, mx: i32) -> i32 {
        for c in 0..gm.col_widths.len() {
            let rechts = Self::col_x(gm, c) + gm.col_widths[c];
            if (mx - rechts).abs() <= TBL_EDGE_GRAB { return c as i32; }
        }
        -1
    }

    /// Bildschirm-x -> Spaltenindex (-1 daneben). Rechnet mit derselben
    /// Geometrie wie das Zeichnen, damit Treffer und Anzeige nicht auseinander
    /// laufen -- derselbe Grund wie bei `chart`.
    /// Bildschirm-x -> ANZEIGE-Position (-1 daneben).
    fn pos_at(gm: &TGeom, mx: i32) -> i32 {
        for p in 0..gm.col_widths.len() {
            let (kx, kw) = Self::col_clip(gm, p);
            // Nur treffen, was in seinem Bereich auch SICHTBAR ist -- sonst
            // truefe man eine Spalte, die unter dem festen Block liegt.
            if mx < kx || mx >= kx + kw { continue; }
            let x = Self::col_x(gm, p);
            if mx >= x && mx < x + gm.col_widths[p] { return p as i32; }
        }
        -1
    }

    /// Bildschirm-x -> DATENspalte. Alles nach aussen spricht von Datenspalten;
    /// nur Zeichnen und Treffertest rechnen in Positionen.
    fn col_at(gm: &TGeom, mx: i32) -> i32 {
        let p = Self::pos_at(gm, mx);
        if p < 0 { return -1; }
        gm.order.get(p as usize).map(|&c| c as i32).unwrap_or(-1)
    }

    /// Tastatur fuer die Tabelle. Eine laufende Zellbearbeitung hat Vorrang --
    /// sonst landeten die Zeichen in einem Filterfeld, das vorher den Fokus
    /// hatte.
    fn table_keys(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let bearbeitet = self.windows[wi].widgets[i].tbl.as_ref()
            .map(|t| t.edit_row >= 0).unwrap_or(false);
        if bearbeitet { self.table_edit_keys(wi, i, g); }
        else { self.table_filter_keys(wi, i, g); }
    }

    /// Tippen in die Zelle, die gerade bearbeitet wird.
    ///
    /// Schreibmarke, Pfeile, Pos1/Ende, Entfernen, Einfuegen aus der
    /// Zwischenablage. **Keine Textauswahl** -- die volle Textfeld-Logik hier
    /// zu wiederholen waere ein zweiter Ort, an dem dieselben Fehler entstehen
    /// koennen; fuer eine Tabellenzelle reicht das hier.
    /// Bildschirm-Rechteck der Zelle, die gerade bearbeitet wird.
    ///
    /// Aus `table_geom` gerechnet statt beim Zeichnen gemerkt: das Zeichnen
    /// laeuft ueber `&self` und koennte gar nichts merken -- und eine zweite,
    /// nachgefuehrte Quelle waere genau die Sorte Doppelung, die zwischen
    /// Treffer und Anzeige auseinander laeuft.
    fn edit_cell_rect(&self, wi: usize, idx: usize) -> Option<(i32, i32, i32, i32)> {
        let gm = self.table_geom(wi, idx);
        let t = self.windows[wi].widgets[idx].tbl.as_ref()?;
        if t.edit_row < 0 { return None; }
        let pos = gm.order.iter().position(|&c| c as i32 == t.edit_col)?;
        let vi = t.view_of(t.edit_row);
        if vi < 0 { return None; }
        let x = Self::col_x(&gm, pos);
        let y = gm.body_y + vi * gm.row_h - t.scroll_y;
        Some((x, y, gm.col_widths[pos], gm.row_h))
    }

    fn table_edit_keys(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let ctrl = g.key_ctrl();
        let shift = g.key_shift();
        // Enter/ESC zuerst -- danach ist die Bearbeitung vorbei und alles
        // Weitere waere Arbeit an einem Zustand, den es nicht mehr gibt.
        if g.key_pressed(KEY_ENTER) { self.table_end_edit(wi, i, true); return; }
        if g.key_pressed(27) { self.table_end_edit(wi, i, false); return; }

        let rect = self.edit_cell_rect(wi, i);
        let (rx, rw) = rect.map(|(x, _, w, _)| (x, w)).unwrap_or((0, 0));

        let t = match self.windows[wi].widgets[i].tbl.as_mut() { Some(t) => t, None => return };
        let mut chars: Vec<char> = t.edit_text.chars().collect();
        let mut caret = t.edit_caret.clamp(0, chars.len() as i32);
        let mut anchor = t.edit_anchor.clamp(0, chars.len() as i32);
        let scroll = t.edit_scroll;

        // Der oeffnende Doppelklick zaehlt nicht als Setzen der Marke -- erst
        // wenn die Taste einmal losgelassen wurde, hoert der Editor auf die
        // Maus.
        if t.edit_maus_sperre {
            if !g.mouse_button(0) { t.edit_maus_sperre = false; }
        }
        let maus_an = !t.edit_maus_sperre;

        // Maus im Eingabefeld: Klick setzt die Marke, Ziehen markiert. Das
        // Feld liegt IN der Tabelle, deshalb hier und nicht im allgemeinen
        // Tabellen-Treffertest -- der wuerde den Klick als Zeilenauswahl
        // deuten und die Bearbeitung beenden.
        if maus_an && g.mouse_button(0) {
            let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
            let ziel = mx - (rx + 5) + scroll;
            let idx = Self::caret_index_at(g, &chars, ziel);
            if !self.was_mouse_down {
                let drin = rect.map(|r| Self::in_rect(mx, my, r)).unwrap_or(false);
                if drin { caret = idx; anchor = idx; }
            } else {
                caret = idx;
            }
        }

        Self::einzeiler_tasten(g, &mut chars, &mut caret, &mut anchor, ctrl, shift);

        // Versatz nachfuehren, damit die Schreibmarke im Feld bleibt.
        let vor: String = chars[..caret.clamp(0, chars.len() as i32) as usize].iter().collect();
        let marke_px = g.text_width(&vor);
        let innen = (rw - 10).max(1);
        let mut scroll = scroll;
        if marke_px - scroll > innen { scroll = marke_px - innen; }
        if marke_px - scroll < 0 { scroll = marke_px; }
        if scroll < 0 { scroll = 0; }

        let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
        t.edit_text = chars.iter().collect();
        t.edit_caret = caret.clamp(0, chars.len() as i32);
        t.edit_anchor = anchor.clamp(0, chars.len() as i32);
        t.edit_scroll = scroll;
    }

    /// Tippen in die fokussierte Filterzelle. Bewusst schlicht: Zeichen
    /// anhaengen, Rueckschritt loeschen, Escape/Enter beendet die Eingabe.
    /// Kein Cursor, keine Auswahl -- ein Filter ist ein Suchfeld, kein Editor,
    /// und die volle Textfeld-Logik hier zu wiederholen waere ein zweiter Ort,
    /// an dem dieselben Fehler entstehen koennen.
    fn table_filter_keys(&mut self, wi: usize, idx: usize, g: &mut Graphics) {
        let c = {
            let t = match self.windows[wi].widgets[idx].tbl.as_ref() { Some(t) => t, None => return };
            if !t.filter_row { return; }
            t.filter_focus
        };
        if c < 0 { return; }
        let mut geaendert = false;
        let eingabe: Vec<char> = g.pop_text_input().chars().collect();
        let rueck = g.key_repeat(KEY_BACKSPACE);
        let raus = g.key_pressed(27) || g.key_pressed(KEY_ENTER);   // 27 = ESC
        {
            let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
            while t.filters.len() <= c as usize { t.filters.push(String::new()); }
            let f = &mut t.filters[c as usize];
            for ch in eingabe {
                if !ch.is_control() { f.push(ch); geaendert = true; }
            }
            if rueck && !f.is_empty() { f.pop(); geaendert = true; }
            if raus { t.filter_focus = -1; }
        }
        if geaendert {
            let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
            t.rebuild_view();
            // Nach dem Filtern kann die alte Scroll-Position weit hinter dem
            // Ende liegen -- dann saehe man eine leere Tabelle.
            t.scroll_y = 0;
            let f = self.windows[wi].widgets[idx].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    fn table_drag(&mut self, wi: usize, idx: usize, mx: i32, my: i32) {
        let gm = self.table_geom(wi, idx);
        let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
        if let Some((pos, x0, bewegt)) = t.head_drag {
            // Erst ab einer Mindeststrecke gilt es als Zug -- ein Klick
            // wackelt fast immer um ein, zwei Punkte.
            if !bewegt && (mx - x0).abs() < HEAD_DRAG_MIN { return; }
            if !t.reorderable { return; }
            let mut pos = pos;
            t.head_drag = Some((pos, x0, true));
            // Ueber die Mitte des Nachbarn hinaus -> tauschen. Kein Warten auf
            // das Loslassen: man soll SEHEN, wo die Spalte landet.
            let mitte_links = if pos > 0 { Self::col_x(&gm, pos - 1) + gm.col_widths[pos - 1] / 2 } else { i32::MIN };
            let mitte_rechts = if pos + 1 < gm.n_cols { Self::col_x(&gm, pos + 1) + gm.col_widths[pos + 1] / 2 } else { i32::MAX };
            if mx < mitte_links {
                t.move_col(pos, pos - 1);
                pos -= 1;
                t.head_drag = Some((pos, x0, true));
            } else if mx > mitte_rechts {
                t.move_col(pos, pos + 1);
                pos += 1;
                t.head_drag = Some((pos, x0, true));
            }
            return;
        }
        if t.drag_col >= 0 {
            let c = t.drag_col as usize;
            let neu = (t.drag_col_w + (mx - t.drag_col_x)).max(TBL_MIN_COL_W);
            if let Some(cw) = t.col_widths.as_mut() {
                if c < cw.len() { cw[c] = neu; }
            }
            return;
        }
        if t.drag_v && gm.max_scroll_y > 0 {
            let track = gm.body_h;
            let handle = Self::handle_height(track, gm.body_h, gm.total_h);
            // Review-Fund: `handle` hat eine feste Untergrenze (handle_height
            // clamped auf min. 16px), `track` (die Widget-Hoehe abzueglich
            // Kopfzeile) nicht -- ein sehr kleines/geschrumpftes Table-Widget
            // (z.B. per GUI_SET_BOUNDS) liess `track - handle` negativ werden,
            // und `i32::clamp(0, negativ)` paniked hart ("assertion failed:
            // min <= max"). `.max(0)` haelt die Invariante unabhaengig von
            // der Widget-Groesse.
            let hi = (track - handle).max(0);
            let new_y = ((my - gm.body_y) - t.drag_off).clamp(0, hi);
            let pos = new_y as f64 / hi.max(1) as f64;
            t.scroll_y = (pos * gm.max_scroll_y as f64) as i32;
        } else if t.drag_h && gm.max_scroll_x > 0 {
            let track = gm.body_w;
            let handle = Self::handle_height(track, gm.body_w, gm.total_w);
            let hi = (track - handle).max(0);
            let new_x = ((mx - gm.body_x) - t.drag_off).clamp(0, hi);
            let pos = new_x as f64 / hi.max(1) as f64;
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
        if !matches!(w.kind, Kind::Checkbox | Kind::Radio | Kind::Toggle) { return Err("GUI_CHECKED: Widget ist keine checkbox/radio/toggle".into()); }
        Ok(w.checked)
    }
    pub fn value(&self, h: i64) -> Result<f64, String> {
        let w = self.wdg(h, "GUI_VALUE")?;
        match w.kind {
            Kind::Slider | Kind::Progress | Kind::Spinner | Kind::Knob => Ok(w.value),
            Kind::Splitter => Ok(if w.group == "v" { w.x as f64 } else { w.y as f64 }),
            _ => Err("GUI_VALUE: Widget ist kein slider/progress/spinner/splitter/knob".into()),
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
        if !matches!(kind, Kind::Checkbox | Kind::Radio | Kind::Toggle) { return Err("GUI_SET_CHECKED: Widget ist keine checkbox/radio/toggle".into()); }
        {
            let w = self.wdg_mut(h, "GUI_SET_CHECKED")?;
            w.checked = f;
        }
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
            Kind::Slider | Kind::Progress | Kind::Spinner | Kind::Knob => { w.value = v.clamp(w.min, w.max); }
            Kind::Splitter => {
                let c = (v.round() as i32).clamp(w.min as i32, w.max as i32);
                if w.group == "v" { w.x = c; } else { w.y = c; }
            }
            _ => return Err("GUI_SET_VALUE: Widget ist kein slider/progress/spinner/splitter".into()),
        }
        Ok(())
    }
    pub fn on_click(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_CLICK")?.on_click = func; Ok(())
    }

    /// Maus betritt das Widget (einmal je Eintritt, nicht jedes Bild).
    pub fn on_hover(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_HOVER")?.on_hover = func; Ok(())
    }
    /// Maus verlaesst das Widget.
    pub fn on_leave(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_LEAVE")?.on_leave = func; Ok(())
    }
    /// Widget bekommt die Eingabe (Textfeld angeklickt, Tab-Wechsel).
    pub fn on_focus(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_FOCUS")?.on_focus = func; Ok(())
    }
    /// Widget verliert die Eingabe -- der Punkt, an dem man eine Eingabe
    /// pruefen will.
    pub fn on_blur(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_BLUR")?.on_blur = func; Ok(())
    }
    pub fn on_change(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_ON_CHANGE")?;
        if !matches!(w.kind, Kind::Slider | Kind::TextInput | Kind::TextArea | Kind::Checkbox | Kind::Table | Kind::Radio | Kind::Dropdown | Kind::ListBox | Kind::Spinner | Kind::Splitter | Kind::Tree) {
            return Err("GUI_ON_CHANGE: nur fuer slider, textinput, textarea, checkbox, table, radio, dropdown, listbox, spinner, splitter oder tree".into());
        }
        w.on_change = func; Ok(())
    }
    /// Widget rund zeichnen. Gedacht fuer Knoepfe, die nur ein Sinnbild
    /// tragen (Wiedergabe, Pause, Weiter) -- ein runder Knopf mit Dreieck
    /// darin liest sich sofort als Abspieltaste.
    pub fn set_round(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_ROUND")?.rund = f;
        Ok(())
    }
    /// Grafik fuer eine Widget-Art hinterlegen (9-Slice). `bild` = -1 nimmt
    /// sie wieder weg.
    pub fn set_skin(&mut self, art: String, bild: i64, rand: i32) -> Result<(), String> {
        let a = art.to_lowercase();
        if Kind::from_str(&a).is_none() {
            return Err(format!("GUI_SKIN: unbekannte Widget-Art '{}'", art));
        }
        if bild < 0 {
            self.skins.remove(&a);
        } else {
            self.skins.insert(a, (bild, rand.max(0)));
        }
        Ok(())
    }

    /// Hinterlegte Grafik einer Art (falls vorhanden).
    fn skin_of(&self, kind: Kind) -> Option<(i64, i32)> {
        self.skins.get(kind.as_str()).copied()
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
            // Review-Fund: press_origin/active_slider/active_split fehlten --
            // ein Slider mitten im Drag deaktiviert feuerte weiter
            // GUI_ON_CHANGE (drag_slider haelt sich nicht an `enabled`), und
            // ein zwischen Press und Release deaktivierter Button loeste
            // beim Loslassen trotzdem noch GUI_ON_CLICK aus.
            if self.press_origin == Some((wi, i)) { self.press_origin = None; }
            if self.active_slider == Some((wi, i)) { self.active_slider = None; }
        if self.active_knob.map(|(w, k, _, _)| (w, k)) == Some((wi, i)) { self.active_knob = None; }
            if self.active_knob.map(|(w, k, _, _)| (w, k)) == Some((wi, i)) { self.active_knob = None; }
            if self.active_split == Some((wi, i)) { self.active_split = None; }
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
        // Review-Fund: active_table/table_press/hover_w fehlten hier -- ein
        // GUI_DESTROY waehrend eines Table-Scrollbar-Drags oder mit noch
        // gedrueckter Tabellenzeile liess GUI_TABLE_SELECTED/GUI_ON_CHANGE
        // weiter auf das (tombstoned) Widget wirken, und ein laufendes
        // Scrollbar-Drag konnte weiterhin table_drag() (und dessen
        // clamp-Panic-Risiko) fuer das zerstoerte Widget ausloesen.
        if self.focus_widget == Some((wi, i)) { self.focus_widget = None; }
        if self.active_slider == Some((wi, i)) { self.active_slider = None; }
        if self.active_split == Some((wi, i)) { self.active_split = None; }
        if self.press_origin == Some((wi, i)) { self.press_origin = None; }
        if self.open_dropdown == Some((wi, i)) { self.open_dropdown = None; }
        if self.active_table == Some((wi, i)) { self.active_table = None; }
        if self.table_press.map(|(tw, ti, _)| (tw, ti)) == Some((wi, i)) { self.table_press = None; }
        if self.editing_table == Some((wi, i)) { self.editing_table = None; }
        if let Some(t) = self.windows[wi].widgets[i].tbl.as_mut() { t.head_drag = None; }
        if self.hover_w == Some((wi, i)) { self.hover_w = None; }
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
        let wd = self.wdg(h, "GUI_FOCUS")?;
        // Review-Fund: `wdg()` prueft nur, ob der Handle ueberhaupt in den
        // Bounds liegt, NICHT ob das Widget noch lebt -- GUI_FOCUS auf ein
        // getombstontes Handle wurde klaglos akzeptiert und rief
        // bring_to_front() fuer ein bereits ZERSTOERTES Fenster auf, was den
        // Index wieder in z_order einfuegte (den window_destroy gerade erst
        // entfernt hatte).
        if !wd.alive || !self.windows.get(wi).map(|w| w.alive).unwrap_or(false) {
            return Err("GUI_FOCUS: Widget/Fenster wurde bereits zerstoert".into());
        }
        self.focus_widget = Some((wi, i));
        self.focus_window = Some(wi);
        self.bring_to_front(wi);
        Ok(())
    }

    /// Handle des Widgets mit Tastatur-Fokus, oder -1. Gegenstueck zu
    /// GUI_FOCUS -- ohne Abfrage koennte ein Programm nicht anzeigen, worauf
    /// die Tastatur gerade wirkt (Hilfetext zum aktiven Feld, Statuszeile).
    pub fn focused(&self) -> i64 {
        match self.focus_widget {
            Some((wi, i)) => {
                let lebt = self.windows.get(wi)
                    .and_then(|w| w.widgets.get(i).map(|x| x.alive && w.alive))
                    .unwrap_or(false);
                if lebt { Self::enc_widget(wi, i) } else { -1 }
            }
            None => -1,
        }
    }
    /// Oberstes lebendes+sichtbares Widget am Bildschirmpunkt (Z-Order), oder -1.
    /// Liefert ein Widget-Handle (fuer Selektion im WYSIWYG-Editor).
    pub fn hit_test(&self, mx: i32, my: i32) -> i64 {
        for &wi in self.z_order.iter().rev() {
            let win = &self.windows[wi];
            if !win.alive || !win.visible { continue; }
            if !Self::in_rect(mx, my, (win.x, win.y, win.w, win.h)) { continue; }
            // innerhalb des Fensters: spaeter gezeichnete Widgets liegen oben.
            // Review-Fund: pruefte bisher nur alive+visible, nicht die
            // Tab-Seite -- GUI_HIT_TEST fand auf einem getabbten Fenster
            // Widgets einer INAKTIVEN Reiter-Seite, obwohl update()/draw()
            // konsequent widget_shown() (das den Tab zusaetzlich prueft)
            // verwenden. Ein darauf aufbauender WYSIWYG-Editor haette so
            // unsichtbare Controls selektiert.
            for i in (0..win.widgets.len()).rev() {
                let wd = &win.widgets[i];
                if self.widget_shown(wi, wd) && Self::in_rect(mx, my, self.abs_rect(wi, wd)) {
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
    /// Loest JEDE Interaktions-Referenz auf Fenster `wi` (oder ein Widget
    /// darin), egal ob das Fenster gerade zerstoert, unsichtbar gemacht oder
    /// per Close-Button geschlossen wird.
    ///
    /// Review-Fund: `window_destroy` loeste bisher nur 5 von ~14 solchen
    /// Referenzen (focus_window/drag_window/resize_window/open_dropdown) --
    /// focus_widget/press_origin/active_slider/active_split/active_table/
    /// table_press/scroll_drag/open_menu/context_open/hover_w zeigten
    /// danach weiter auf das (tombstoned, aber nie geloeschte) Fenster/
    /// Widget. Weil Widgets nie wirklich entfernt werden (Tombstone-Pattern),
    /// indizierten diese Referenzen weiter erfolgreich -- `update()` feuerte
    /// dadurch z.B. GUI_ON_CHANGE/GUI_ON_CLICK fuer ein bereits zerstoertes
    /// Fenster, oder ein laufendes Table-Scrollbar-Drag lief gegen ein totes
    /// Widget weiter (und konnte den clamp-Panic in table_drag ausloesen).
    fn clear_window_interactions(&mut self, wi: usize) {
        if self.focus_window == Some(wi) { self.focus_window = None; }
        if self.drag_window == Some(wi) { self.drag_window = None; }
        if self.resize_window == Some(wi) { self.resize_window = None; }
        if self.scroll_drag == Some(wi) { self.scroll_drag = None; }
        if self.focus_widget.map(|(w, _)| w) == Some(wi) { self.focus_widget = None; }
        if self.press_origin.map(|(w, _)| w) == Some(wi) { self.press_origin = None; }
        if self.active_slider.map(|(w, _)| w) == Some(wi) { self.active_slider = None; }
        if self.active_knob.map(|(w, _, _, _)| w) == Some(wi) { self.active_knob = None; }
        if self.active_split.map(|(w, _)| w) == Some(wi) { self.active_split = None; }
        if self.active_table.map(|(w, _)| w) == Some(wi) { self.active_table = None; }
        if self.table_press.map(|(w, _, _)| w) == Some(wi) { self.table_press = None; }
        if self.editing_table.map(|(w, _)| w) == Some(wi) { self.editing_table = None; }
        if self.hover_w.map(|(w, _)| w) == Some(wi) { self.hover_w = None; }
        if self.open_dropdown.map(|(w, _)| w) == Some(wi) { self.open_dropdown = None; }
        if self.open_menu.map(|(w, _)| w) == Some(wi) { self.open_menu = None; }
        if self.context_open.map(|(w, _, _, _)| w) == Some(wi) { self.context_open = None; }
    }
    pub fn window_destroy(&mut self, h: i64) -> Result<(), String> {
        let win = self.win_mut(h, "GUI_WINDOW_DESTROY")?;
        win.alive = false; win.visible = false;
        let wi = h as usize;
        // z_order-Entfernung nur beim ECHTEN Destroy -- ein nur unsichtbar
        // gemachtes Fenster (window_visible(false)/Close-Button) bleibt in
        // z_order, sonst wuerde es nach einem erneuten window_visible(true)
        // nie wieder gezeichnet/getroffen (nichts fuegt es sonst wieder ein).
        self.z_order.retain(|&i| i != wi);
        self.clear_window_interactions(wi);
        Ok(())
    }
    pub fn window_widget_count(&self, h: i64) -> Result<i64, String> {
        let w = self.windows.get(h as usize)
            .ok_or("GUI_WINDOW_WIDGET_COUNT: ungueltiges GUI_WINDOW-Handle")?;
        // Review-Fund: pruefte nur die Widgets auf `alive`, nie das FENSTER
        // selbst -- ein zerstoertes Fenster (window_destroy) lieferte
        // weiterhin seine (noch lebenden) Widget-Handles, obwohl to_json()
        // solche Fenster beim Serialisieren bereits konsequent ausfiltert.
        if !w.alive { return Ok(0); }
        Ok(w.widgets.iter().filter(|wd| wd.alive).count() as i64)
    }
    /// Handle des `n`-ten LEBENDEN Widgets von Fenster `h` (Einfuege-Reihenfolge),
    /// oder -1. Fuer Enumeration/Serialisierung.
    pub fn window_widget(&self, h: i64, n: i64) -> Result<i64, String> {
        let wi = h as usize;
        let w = self.windows.get(wi)
            .ok_or("GUI_WINDOW_WIDGET: ungueltiges GUI_WINDOW-Handle")?;
        if !w.alive { return Ok(-1); }
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
        // Nur UNGEBUNDENE Handler wandern in die Datei. Eine an eine Instanz
        // gebundene Methode laesst sich nicht wiederherstellen -- das Objekt
        // gibt es beim Laden nicht. Ihren blossen Namen zu schreiben waere
        // eine Luege: er wuerde beim Laden als freie Funktion gedeutet.
        if let Some(f) = &w.on_click {
            if !f.ist_gebunden() { o["on_click"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_hover {
            if !f.ist_gebunden() { o["on_hover"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_leave {
            if !f.ist_gebunden() { o["on_leave"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_focus {
            if !f.ist_gebunden() { o["on_focus"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_blur {
            if !f.ist_gebunden() { o["on_blur"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_change {
            if !f.ist_gebunden() { o["on_change"] = serde_json::json!(&*f.name); }
        }
        if !w.ov.is_empty() { o["ov"] = serde_json::json!(w.ov); }
        if !w.group.is_empty() { o["group"] = serde_json::json!(w.group); }
        if !w.items.is_empty() { o["items"] = serde_json::json!(w.items); }
        if w.sel != -1 { o["sel"] = serde_json::json!(w.sel); }   // Dropdown/ListBox-Index ODER Image-Textur
        if !w.enabled { o["enabled"] = serde_json::json!(false); }
        if w.font != -1 { o["font"] = serde_json::json!(w.font); }
        if w.font_size != 0 { o["font_size"] = serde_json::json!(w.font_size); }
        if w.anchor != 5 { o["anchor"] = serde_json::json!(Self::anchor_str(w.anchor)); }
        if let Some(t) = &w.tbl {
            // Eine schlichte Textzelle wird als STRING geschrieben, nur eine
            // mit eigener Farbe/Art/Bild als Objekt. Damit bleibt das
            // .dhform-Format fuer gewoehnliche Tabellen unveraendert (alte
            // Dateien und der Form-Designer lesen es weiter), ohne dass die
            // neuen Angaben beim Speichern verloren gehen.
            let rows: Vec<serde_json::Value> = t.rows.iter().map(|row| {
                serde_json::Value::Array(row.iter().map(|c| {
                    if c.fg < 0 && c.bg < 0 && c.align < 0
                        && c.kind == CellKind::Text && c.image < 0 && c.value == 0.0 {
                        serde_json::json!(c.text)
                    } else {
                        let mut o = serde_json::Map::new();
                        o.insert("text".into(), serde_json::json!(c.text));
                        if c.fg >= 0 { o.insert("fg".into(), serde_json::json!(c.fg)); }
                        if c.bg >= 0 { o.insert("bg".into(), serde_json::json!(c.bg)); }
                        if c.align >= 0 { o.insert("align".into(), serde_json::json!(c.align)); }
                        if c.kind != CellKind::Text {
                            o.insert("kind".into(), serde_json::json!(c.kind.name()));
                        }
                        if c.image >= 0 { o.insert("image".into(), serde_json::json!(c.image)); }
                        if c.value != 0.0 { o.insert("value".into(), serde_json::json!(c.value)); }
                        serde_json::Value::Object(o)
                    }
                }).collect())
            }).collect();
            let mut tj = serde_json::json!({
                "headers": t.headers, "rows": rows,
                "col_widths": t.col_widths, "selected": t.selected,
            });
            // Nur schreiben, was vom Standard abweicht -- sonst waere jede
            // gespeicherte Form voller Vorgabewerte.
            if t.row_h != TBL_ROW_H { tj["row_h"] = serde_json::json!(t.row_h); }
            if t.header_h != TBL_HEADER_H { tj["header_h"] = serde_json::json!(t.header_h); }
            if t.zebra { tj["zebra"] = serde_json::json!(true); }
            if !t.grid { tj["grid"] = serde_json::json!(false); }
            if !t.col_align.is_empty() { tj["col_align"] = serde_json::json!(t.col_align); }
            if !t.col_order.is_empty() { tj["col_order"] = serde_json::json!(t.col_order); }
            if t.frozen > 0 { tj["frozen"] = serde_json::json!(t.frozen); }
            // Die Schalter gehoerten bisher nicht ins .dhform -- ohne sie
            // koennte ein Formular-Designer sie zwar setzen, beim Laden waeren
            // sie aber wieder weg. Nur schreiben, was vom Standard abweicht.
            if t.filter_row { tj["filter_row"] = serde_json::json!(true); }
            if !t.sortable { tj["sortable"] = serde_json::json!(false); }
            if !t.resizable_cols { tj["resizable_cols"] = serde_json::json!(false); }
            if t.reorderable { tj["reorderable"] = serde_json::json!(true); }
            if t.multi { tj["multi"] = serde_json::json!(true); }
            if t.col_edit.iter().any(|&x| x) { tj["col_edit"] = serde_json::json!(t.col_edit); }
            if t.row_fg.iter().any(|&x| x >= 0) { tj["row_fg"] = serde_json::json!(t.row_fg); }
            if t.row_bg.iter().any(|&x| x >= 0) { tj["row_bg"] = serde_json::json!(t.row_bg); }
            o["table"] = tj;
        }
        if let Some(t) = &w.tree {
            o["tree"] = serde_json::json!({
                "nodes": t.nodes.iter().map(|n| serde_json::json!({
                    "label": n.label, "parent": n.parent, "level": n.level,
                    "expanded": n.expanded, "has_children": n.has_children,
                })).collect::<Vec<_>>(),
                "selected": t.selected,
            });
        }
        if w.tab_page != -1 { o["tab_page"] = serde_json::json!(w.tab_page); }
        o
    }
    fn widget_from_json(wj: &serde_json::Value) -> Result<Widget, String> {
        let ks = wj["kind"].as_str().ok_or("GUI_LOAD: Widget ohne 'kind'")?;
        let kind = Kind::from_str(ks).ok_or_else(|| format!("GUI_LOAD: unbekannter Widget-Typ '{}'", ks))?;
        let gi = |k: &str, d: i64| wj[k].as_i64().unwrap_or(d) as i32;
        let mut w = Self::blank(kind, gi("x", 0), gi("y", 0), gi("w", 0), gi("h", 0));
        w.text = wj["text"].as_str().unwrap_or("").to_string();
        w.color = wj["color"].as_i64().unwrap_or(-1);   // -1 = dem Thema folgen
        w.value = wj["value"].as_f64().unwrap_or(0.0);
        w.min = wj["min"].as_f64().unwrap_or(0.0);
        w.max = wj["max"].as_f64().unwrap_or(1.0);
        // Review-Fund: `slider()`/`spinner()` (Konstruktoren) lehnen
        // `max <= min` ab, aber GUI_LOAD baut Widgets direkt aus JSON ohne
        // diese Pruefung -- ein geladenes `{"min":10,"max":0}` liess jeden
        // spaeteren `v.clamp(w.min, w.max)` (GUI_SET_VALUE, Spinner-Schritt)
        // mit "min <= max"-Panic abstuerzen. Reihenfolge einfach herstellen
        // statt hart abzulehnen -- ein geladenes Widget soll nutzbar bleiben.
        if w.min > w.max { std::mem::swap(&mut w.min, &mut w.max); }
        w.checked = wj["checked"].as_bool().unwrap_or(false);
        w.placeholder = wj["placeholder"].as_str().unwrap_or("").to_string();
        w.visible = wj["visible"].as_bool().unwrap_or(true);
        w.on_click = wj["on_click"].as_str().map(Rueckruf::benannt);
        w.on_hover = wj["on_hover"].as_str().map(Rueckruf::benannt);
        w.on_leave = wj["on_leave"].as_str().map(Rueckruf::benannt);
        w.on_focus = wj["on_focus"].as_str().map(Rueckruf::benannt);
        w.on_blur = wj["on_blur"].as_str().map(Rueckruf::benannt);
        w.on_change = wj["on_change"].as_str().map(Rueckruf::benannt);
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
                    // Review-Fund: `filter_map(|x| x.as_str())` liess jede
                    // NICHT-String-Zelle (Zahl, Bool, null) STILLSCHWEIGEND
                    // aus der Zeile verschwinden -- ein `.dhform`/gespeichertes
                    // GUI_SAVE mit einer numerischen Tabellenzelle lud eine zu
                    // kurze Zeile, `draw_table` paniked dann beim naechsten
                    // GUI_DRAW. Jede Zelle wird jetzt zu einem String
                    // konvertiert statt verworfen -- Zeilenlaenge bleibt erhalten.
                    fn cell_to_string(x: &serde_json::Value) -> String {
                        match x {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        }
                    }
                    // Zelle kann ein blosser Wert sein (altes Format, und der
                    // Normalfall) ODER ein Objekt mit Farben/Art/Bild.
                    fn to_cell(x: &serde_json::Value) -> Cell {
                        if let Some(o) = x.as_object() {
                            let mut c = Cell::text(o.get("text").map(cell_to_string).unwrap_or_default());
                            if let Some(v) = o.get("fg").and_then(|v| v.as_i64()) { c.fg = v; }
                            if let Some(v) = o.get("bg").and_then(|v| v.as_i64()) { c.bg = v; }
                            if let Some(v) = o.get("align").and_then(|v| v.as_i64()) { c.align = v as i8; }
                            if let Some(v) = o.get("kind").and_then(|v| v.as_str()) {
                                if let Some(k) = CellKind::parse(v) { c.kind = k; }
                            }
                            if let Some(v) = o.get("image").and_then(|v| v.as_i64()) { c.image = v; }
                            if let Some(v) = o.get("value").and_then(|v| v.as_f64()) { c.value = v; }
                            c
                        } else {
                            Cell::text(cell_to_string(x))
                        }
                    }
                    ts.rows = rs.iter().map(|row| row.as_array()
                        .map(|r| r.iter().map(to_cell).collect())
                        .unwrap_or_default()).collect();
                    ts.row_fg = vec![-1; ts.rows.len()];
                    ts.row_bg = vec![-1; ts.rows.len()];
                }
                if let Some(v) = tj["row_h"].as_i64() { ts.row_h = (v as i32).clamp(8, 400); }
                if let Some(v) = tj["header_h"].as_i64() { ts.header_h = (v as i32).clamp(0, 200); }
                if let Some(v) = tj["zebra"].as_bool() { ts.zebra = v; }
                if let Some(v) = tj["grid"].as_bool() { ts.grid = v; }
                if let Some(a) = tj["col_align"].as_array() {
                    ts.col_align = a.iter().filter_map(|x| x.as_i64().map(|n| n as i8)).collect();
                }
                if let Some(a) = tj["col_order"].as_array() {
                    ts.col_order = a.iter().filter_map(|x| x.as_u64().map(|n| n as usize)).collect();
                }
                if let Some(v) = tj["frozen"].as_i64() { ts.frozen = v as i32; }
                if let Some(v) = tj["filter_row"].as_bool() { ts.filter_row = v; }
                if let Some(v) = tj["sortable"].as_bool() { ts.sortable = v; }
                if let Some(v) = tj["resizable_cols"].as_bool() { ts.resizable_cols = v; }
                if let Some(v) = tj["reorderable"].as_bool() { ts.reorderable = v; }
                if let Some(v) = tj["multi"].as_bool() { ts.multi = v; }
                if let Some(a) = tj["col_edit"].as_array() {
                    ts.col_edit = a.iter().map(|x| x.as_bool().unwrap_or(false)).collect();
                }
                for (schluessel, ziel) in [("row_fg", true), ("row_bg", false)] {
                    if let Some(a) = tj[schluessel].as_array() {
                        let v: Vec<i64> = a.iter().map(|x| x.as_i64().unwrap_or(-1)).collect();
                        if ziel { ts.row_fg = v; } else { ts.row_bg = v; }
                    }
                }
                // Nach dem Laden muessen die Zeilenfarb-Listen zur Zeilenzahl
                // passen -- sonst greift row_fg[r] beim Zeichnen daneben.
                ts.row_fg.resize(ts.rows.len(), -1);
                ts.row_bg.resize(ts.rows.len(), -1);
                if let Some(cw) = tj["col_widths"].as_array() {
                    ts.col_widths = Some(cw.iter().filter_map(|x| x.as_i64().map(|n| n as i32)).collect());
                }
                ts.selected = tj["selected"].as_i64().unwrap_or(-1) as i32;
            }
            ts.hover_row = -1; ts.clicked_row = -1;
            w.tbl = Some(Box::new(ts));
        }
        if kind == Kind::Tree {
            let mut ts = TreeState::default();
            if let Some(tj) = wj.get("tree") {
                if let Some(ns) = tj["nodes"].as_array() {
                    ts.nodes = ns.iter().map(|nj| TreeNode {
                        label: nj["label"].as_str().unwrap_or("").to_string(),
                        parent: nj["parent"].as_i64().unwrap_or(-1) as i32,
                        level: nj["level"].as_i64().unwrap_or(0) as i32,
                        expanded: nj["expanded"].as_bool().unwrap_or(false),
                        has_children: nj["has_children"].as_bool().unwrap_or(false),
                    }).collect();
                }
                ts.selected = tj["selected"].as_i64().unwrap_or(-1) as i32;
            }
            ts.hover = -1;
            w.tree = Some(Box::new(ts));
        }
        w.tab_page = wj["tab_page"].as_i64().unwrap_or(-1) as i32;
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
        let menus: Vec<serde_json::Value> = win.menus.iter().map(|m| serde_json::json!({
            "label": m.label, "in_bar": m.in_bar,
            "items": m.items.iter().map(|it| serde_json::json!({
                "label": it.label, "separator": it.separator, "enabled": it.enabled,
            })).collect::<Vec<_>>(),
        })).collect();
        let obj = serde_json::json!({
            "title": win.title, "x": win.x, "y": win.y, "w": win.w, "h": win.h,
            "movable": win.movable, "closable": win.closable, "visible": win.visible,
            "resizable": win.resizable, "chrome": win.chrome,
            "min_w": win.min_w, "min_h": win.min_h, "max_w": win.max_w, "max_h": win.max_h,
            "widgets": widgets,
            "menus": menus, "tabs": win.tabs, "active_tab": win.active_tab,
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
        if let Some(ms) = v["menus"].as_array() {
            self.windows[wi].menus = ms.iter().map(|mj| Menu {
                label: mj["label"].as_str().unwrap_or("").to_string(),
                in_bar: mj["in_bar"].as_bool().unwrap_or(true),
                items: mj["items"].as_array().map(|its| its.iter().map(|itj| MenuItem {
                    label: itj["label"].as_str().unwrap_or("").to_string(),
                    separator: itj["separator"].as_bool().unwrap_or(false),
                    enabled: itj["enabled"].as_bool().unwrap_or(true),
                    clicked: false,
                }).collect()).unwrap_or_default(),
            }).collect();
        }
        if let Some(ts) = v["tabs"].as_array() {
            self.windows[wi].tabs = ts.iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
        }
        self.windows[wi].active_tab = v["active_tab"].as_i64().unwrap_or(0) as i32;
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
        // Review-Fund: `th` (Scrollbar-Spurlaenge) hat keine Untergrenze --
        // ein kleines/schmales scrollbares Fenster (z.B. `GUI_WINDOW_SET_
        // BOUNDS win, x, y, w, 40`) liess `th < 24` werden, und
        // `.clamp(24, th)` paniked hart ("assertion failed: min <= max"),
        // reproduzierbar in JEDEM GUI_DRAW eines solchen Fensters. `lo`
        // haelt sich selbst unter 24, wenn die Spur kuerzer ist.
        let th = (view - 2).max(0);
        let max_s = content - view;
        let lo = th.min(24);
        let thh = if th > 0 { ((th * view) / content.max(1)).clamp(lo, th) } else { 0 };
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
    /// Halbtransparente Selektions-Hintergrundfarbe (ARGB) hinter markiertem
    /// Text -- gemeinsam fuer TextInput/TextArea (frueher an beiden Stellen
    /// dupliziert).
    fn selection_bg(&self, wdg: &Widget) -> i64 {
        let acc = self.wcol(wdg, "accent", "accent");
        ((0x66i64) << 24) | (acc & 0xFFFFFF)
    }
    /// Ob der Text-Caret in diesem Frame sichtbar ist (Blink-Takt) --
    /// gemeinsam fuer TextInput/TextArea.
    fn caret_blink_on(&self) -> bool {
        (self.frame_count / self.m("caret_period").max(1) as i64) % 2 == 0
    }
    /// Text mit per-Widget-Font/-Groesse (sonst unveraendert via g.text).
    /// Schriftgroesse dieses Widgets in Pixeln.
    fn wsize(&self, g: &Graphics, w: &Widget) -> i32 {
        if w.font_size > 0 { w.font_size } else { g.text_height() }
    }

    /// Breite eines Textes so, wie `wtext` ihn zeichnen WIRD.
    ///
    /// Die Fallunterscheidung muss die von `wtext` genau spiegeln: ohne
    /// eigene Schrift und Groesse zeichnet es mit der aktiven Schrift, sonst
    /// mit dem Handle des Widgets (-1 = Standardschrift, NICHT die aktive).
    /// Weicht das Messen davon ab, sitzt zentrierter Text schief und der
    /// Beschnitt greift an der falschen Stelle.
    fn wtext_width(&self, g: &Graphics, w: &Widget, s: &str) -> i32 {
        if w.font == -1 && w.font_size == 0 {
            g.text_width(s)
        } else {
            g.text_width_in(s, self.wsize(g, w), self.wfont(g, w))
        }
    }

    /// Schrift, mit der DIESES Widget zeichnet.
    ///
    /// Ohne eigene Schrift gilt die global gesetzte -- NICHT die eingebaute
    /// Standardschrift. Vorher wechselte `GUI_SET_FONT_SIZE` allein still
    /// die Schriftart auf die Pixelschrift, weil das Zeichnen dann mit dem
    /// Handle -1 lief. Wer eine GROESSE setzt, meint aber die Groesse, nicht
    /// eine andere Schrift.
    fn wfont(&self, g: &Graphics, w: &Widget) -> i64 {
        font_wahl(w.font, g.active_font())
    }

    fn wtext(&self, g: &mut Graphics, w: &Widget, x: i32, y: i32, s: String, c: i64) {
        if w.font == -1 && w.font_size == 0 {
            g.text(x, y, s, c);
        } else {
            let (sz, font) = (self.wsize(g, w), self.wfont(g, w));
            g.text_styled(x, y, s, c, font, sz);
        }
    }
    /// Gefuellte Box + Rahmen, runde Ecken wenn Metrik `corner_radius` > 0.
    /// Gefuellte Flaeche mit Rand -- die gemeinsame Grundform fast aller
    /// Widgets. Mit den Plastik-Metriken (`gradient`/`gloss`/`bevel`) wird
    /// daraus eine gewoelbte Flaeche; sind sie 0, bleibt alles flach wie
    /// bisher.
    fn fbox(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, fill: i64, border: i64) {
        self.flaeche(g, x1, y1, x2, y2, fill, border, false);
    }

    /// Wie `fbox`, beruecksichtigt aber eine fuer diese Widget-Art
    /// hinterlegte Grafik.
    #[allow(clippy::too_many_arguments)]
    fn fbox_w(&self, g: &mut Graphics, kind: Kind, x1: i32, y1: i32, x2: i32, y2: i32,
              fill: i64, border: i64) {
        if self.skin(g, kind, x1, y1, x2, y2) { return; }
        self.flaeche(g, x1, y1, x2, y2, fill, border, false);
    }

    #[allow(clippy::too_many_arguments)]
    fn fbox_tief_w(&self, g: &mut Graphics, kind: Kind, x1: i32, y1: i32, x2: i32, y2: i32,
                   fill: i64, border: i64) {
        if self.skin(g, kind, x1, y1, x2, y2) { return; }
        self.flaeche(g, x1, y1, x2, y2, fill, border, true);
    }

    /// Hinterlegte Grafik zeichnen, falls es eine gibt. Liefert TRUE, wenn
    /// sie die gezeichnete Flaeche ersetzt hat.
    fn skin(&self, g: &mut Graphics, kind: Kind, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        match self.skin_of(kind) {
            Some((bild, rand)) => {
                let _ = g.nine_slice(bild, x1, y1, x2 - x1 + 1, y2 - y1 + 1, rand);
                true
            }
            None => false,
        }
    }

    /// Wie `fbox`, aber VERSENKT: der Verlauf laeuft andersherum und statt
    /// einer Glanzkante liegt ein Schatten unter dem oberen Rand. Das ist der
    /// Unterschied zwischen einem Knopf, den man drueckt, und einem Feld, in
    /// das man hineinschreibt -- ohne ihn sieht ein Eingabefeld aus wie ein
    /// Knopf, und die Oberflaeche verliert ihre Aussage.
    fn fbox_tief(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, fill: i64, border: i64) {
        self.flaeche(g, x1, y1, x2, y2, fill, border, true);
    }

    #[allow(clippy::too_many_arguments)]
    fn flaeche(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32,
               fill: i64, border: i64, tief: bool) {
        let rad = self.m("corner_radius");
        let grad = self.m("gradient");
        if grad > 0 {
            // Erhaben: hell oben, dunkel unten. Versenkt: genau umgekehrt --
            // so faellt das Licht scheinbar weiter von oben ein, die Flaeche
            // liegt aber tiefer.
            let (oben, unten) = if tief {
                (shade(fill, -grad), shade(fill, grad / 2))
            } else {
                (shade(fill, grad), shade(fill, -grad))
            };
            g.round_gradient(x1, y1, x2, y2, rad, oben, unten);
            if !tief {
                self.gloss(g, x1, y1, x2, y2, rad);
            }
        } else if rad > 0 {
            g.round_rect(x1, y1, x2, y2, rad, fill, true);
        } else {
            g.box_fill(x1, y1, x2, y2, fill);
        }
        self.bevel(g, x1, y1, x2, y2, rad, tief);
        if rad > 0 {
            g.round_rect(x1, y1, x2, y2, rad, border, false);
        } else {
            g.rect(x1, y1, x2, y2, border);
        }
    }

    /// Glanzkante: halbdurchsichtiges Weiss ueber der oberen Haelfte, nach
    /// unten ausblendend. Erst das laesst eine Flaeche gewoelbt wirken -- ein
    /// blosser Verlauf sieht weiterhin flach aus.
    fn gloss(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, rad: i32) {
        let s = self.m("gloss");
        let h = y2 - y1;
        if s <= 0 || h < 4 {
            return;
        }
        let a = ((s.clamp(0, 100) as f64 / 100.0) * 255.0) as i64;
        // Bei RUNDEN Formen (Radius >= halbe Hoehe: Kreis oder Kapsel) muss
        // der Glanz die ganze Hoehe einnehmen. Nur die obere Haelfte zu
        // nehmen ginge schief, weil `round_gradient` seinen Radius auf die
        // halbe Hoehe DIESER Flaeche deckelt -- aus dem Kreis wuerde eine
        // flache Kapsel, die seitlich ueber die Form hinausragt. Genau das
        // waren die eckigen Schatten an runden Knoepfen und der Klecks auf
        // den Drehreglern.
        let unten = gloss_unten(y1, y2, rad);
        // Alpha 1 = praktisch unsichtbar (0 hiesse in dhrt DECKEND).
        g.round_gradient(x1, y1, x2, unten, rad,
                         (a << 24) | 0xFFFFFF, 0x01FF_FFFFu32 as i64);
    }

    /// Grundfarbe fuer Griffe (Schieber-Knauf, Kippschalter-Knopf,
    /// Drehregler-Kappe).
    ///
    /// NICHT einfach die Widget-Farbe: die liegt per Definition nahe am
    /// Untergrund, und ein weisser Griff auf weissem Feld ist unsichtbar --
    /// im dunklen Thema faellt das nicht auf, weil dort der Verlauf traegt.
    /// Gemischt wird Richtung Textfarbe, denn die kontrastiert im JEDEM Thema
    /// mit dem Untergrund. So hebt sich der Griff hell wie dunkel ab, ohne
    /// dass ein Thema eine eigene Farbe dafuer angeben muss.
    fn griff_farbe(&self, w: &Widget) -> i64 {
        mischen(self.wcol(w, "bg", "widget_bg"), self.th("text_fg"), 0.28)
    }

    /// Metallischer Rundgriff (Schieber-Knauf, Kippschalter-Knopf,
    /// Drehregler-Kappe). Gebaut aus konzentrischen Ringen mit wechselnder
    /// Helligkeit -- einen Verlauf ENTLANG eines Kreises kann `ring` nicht,
    /// gestaffelte Ringe sind die Naeherung, die ohne Textur auskommt.
    fn knauf(&self, g: &mut Graphics, cx: i32, cy: i32, r: i32, grund: i64) {
        let r = r.max(2);
        if self.m("gradient") <= 0 {
            g.circle(cx, cy, r, grund);
            return;
        }
        // Schatten darunter, dann die Kappe von hell (oben) nach dunkel.
        g.circle(cx, cy + 1, r, 0x50000000);
        g.round_gradient(cx - r, cy - r, cx + r, cy + r, r,
                         shade(grund, 34), shade(grund, -28));
        // Lichtreflex ueber die GANZE Kugel, nicht als abgesetzte Kappe:
        // ein kleineres rundes Rechteck bekaeme von `round_gradient` einen
        // auf seine halbe Hoehe gedeckelten Radius und saehe als flacher
        // Klecks auf der Kugel aus.
        g.round_gradient(cx - r, cy - r, cx + r, cy + r, r,
                         0x4EFFFFFF, 0x01FF_FFFFu32 as i64);
        // Kontur. Auf hellem Untergrund traegt der Verlauf allein nicht --
        // eine weisse Woelbung auf Weiss hat kaum Kontrast, und der Griff
        // verschwaende ohne diese Kante.
        g.circle_outline(cx, cy, r, self.th("widget_border"));
    }

    /// Fase: eine helle Linie oben, eine dunkle unten. Die schmalste Zutat
    /// mit der groessten Wirkung -- sie trennt die Flaeche sichtbar vom
    /// Untergrund.
    fn bevel(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, rad: i32, tief: bool) {
        if self.m("bevel") <= 0 || x2 - x1 < 4 {
            return;
        }
        let ein = rad.max(1);
        let (oben, unten) = if tief {
            // Versenkt: Schatten oben, Lichtkante unten.
            (0x60000000u32 as i64, 0x38FFFFFFu32 as i64)
        } else {
            (0x50FFFFFFu32 as i64, 0x40000000u32 as i64)
        };
        g.line(x1 + ein, y1 + 1, x2 - ein, y1 + 1, oben);
        g.line(x1 + ein, y2 - 1, x2 - ein, y2 - 1, unten);
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
                if let Some(t) = wdg.tree.as_mut() { t.hover = -1; }
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
                    if kind == Kind::Tree { self.tree_hover(top, i, my, g); }
                }
            }
        }
        // --- Flanken: betreten/verlassen, Fokus bekommen/verlieren ---------
        //
        // Erst HIER, weil `hovered` fuer dieses Bild feststeht und der Fokus
        // gesetzt ist. Verglichen wird gegen den Zustand des VORIGEN Bildes --
        // `hovered` allein wuerde jedes Bild feuern, solange die Maus
        // stehenbleibt. Die Handler wandern wie ueberall in `pending` und
        // werden nach `update()` von der VM aufgerufen, damit ein Handler die
        // GUI nicht mitten im Zustands-Update umbauen kann.
        let fokus = self.focus_widget;
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                let w = &mut self.windows[wi].widgets[i];
                let ist_fokus = fokus == Some((wi, i));
                let (h_jetzt, h_vorher) = (w.hovered, w.was_hovered);
                let (f_jetzt, f_vorher) = (ist_fokus, w.was_focused);
                w.was_hovered = h_jetzt;
                w.was_focused = f_jetzt;
                let mut feuern: Vec<Rueckruf> = Vec::new();
                if h_jetzt && !h_vorher {
                    if let Some(f) = w.on_hover.clone() { feuern.push(f); }
                } else if !h_jetzt && h_vorher {
                    if let Some(f) = w.on_leave.clone() { feuern.push(f); }
                }
                if f_jetzt && !f_vorher {
                    if let Some(f) = w.on_focus.clone() { feuern.push(f); }
                } else if !f_jetzt && f_vorher {
                    if let Some(f) = w.on_blur.clone() { feuern.push(f); }
                }
                self.pending.extend(feuern);
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
        // Laufendes Drehregler-Ziehen: senkrecht, ausgehend vom Wert beim
        // Anfassen. 140 Pixel entsprechen dem ganzen Bereich -- weit genug
        // fuer feines Einstellen, kurz genug zum schnellen Durchfahren.
        if let Some((wi, i, y0, v0)) = self.active_knob {
            if is_down {
                let w = &mut self.windows[wi].widgets[i];
                let spanne = w.max - w.min;
                let neu = (v0 + (y0 - my) as f64 / 140.0 * spanne).clamp(w.min, w.max);
                if (neu - w.value).abs() > f64::EPSILON {
                    w.value = neu;
                    let och = w.on_change.clone();
                    if let Some(f) = och { self.pending.push(f); }
                }
            } else {
                self.active_knob = None;
            }
        }
        // Kippschalter: `value` zieht dem Zustand weich nach, damit der Knopf
        // hinuebergleitet statt zu springen.
        for win in self.windows.iter_mut() {
            for wdg in win.widgets.iter_mut() {
                if wdg.kind == Kind::Toggle {
                    let ziel = if wdg.checked { 1.0 } else { 0.0 };
                    wdg.value += (ziel - wdg.value) * 0.28;
                    if (ziel - wdg.value).abs() < 0.005 { wdg.value = ziel; }
                }
            }
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
                if let Some(t) = self.windows[wi].widgets[i].tbl.as_mut() {
                    t.drag_v = false; t.drag_h = false;
                    t.drag_col = -1;      // Spaltenbreiten-Ziehen endet hier
                    // Kopf-Geste auswerten: ohne Bewegung war es ein
                    // Sortier-Klick, mit Bewegung wurde schon verschoben.
                    if let Some((pos, _, bewegt)) = t.head_drag.take() {
                        if !bewegt && t.sortable {
                            let o = t.order();
                            if let Some(&c) = o.get(pos) {
                                let c = c as i32;
                                if t.sort_col == c { t.sort_desc = !t.sort_desc; }
                                else { t.sort_col = c; t.sort_desc = false; }
                                t.rebuild_view();
                            }
                        }
                    }
                }
                self.active_table = None;
            }
        }
        // Neuer Druck (entfaellt, wenn ein Menue den Klick verarbeitet hat).
        if just_pressed && !menu_consumed && !scroll_consumed && !tab_consumed && self.drag_window.is_none() && self.resize_window.is_none()
            && self.active_slider.is_none() && self.active_table.is_none() && self.active_split.is_none() {
            // Doppelklick: zweiter Druck innerhalb DBL_TIME an fast derselben
            // Stelle. Die Ortspruefung muss sein -- sonst gaelten zwei rasche
            // Klicks auf verschiedene Zeilen als Doppelklick.
            let jetzt = g.get_time();
            self.dbl_click = match self.last_click {
                Some((t0, x0, y0)) =>
                    jetzt - t0 <= DBL_TIME && (mx - x0).abs() <= DBL_DIST && (my - y0).abs() <= DBL_DIST,
                None => false,
            };
            // Nach einem erkannten Doppelklick von vorn zaehlen, sonst waere
            // ein dritter Klick gleich wieder einer.
            self.last_click = if self.dbl_click { None } else { Some((jetzt, mx, my)) };
            self.handle_press(mx, my);
            self.dbl_click = false;
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
            let (ctrl, shift) = (g.key_ctrl(), g.key_shift());
            if let Some((wi, i, row)) = self.table_press.take() {
                let hr = self.windows[wi].widgets[i].tbl.as_ref().map(|t| t.hover_row).unwrap_or(-1);
                if row >= 0 && hr == row {
                    let w = &mut self.windows[wi].widgets[i];
                    let mut haken = false;
                    if let Some(t) = w.tbl.as_mut() {
                        // Strg schaltet eine Zeile um, Umschalt waehlt den
                        // Bereich vom Anker -- sonst ersetzt der Klick die
                        // Auswahl. Genau so kennt man es von jeder Liste.
                        if t.multi && ctrl { t.auswahl_umschalten(row); }
                        else if t.multi && shift { t.auswahl_bereich(row); }
                        else { t.auswahl_setzen(row); }
                        t.clicked_row = row;
                        t.clicked_col = t.hover_col;
                        // Eine Hakenzelle schaltet beim Klick selbst um --
                        // sonst muesste jedes Programm das nachbauen, und die
                        // Zelle saehe bedienbar aus, ohne es zu sein.
                        let (r, c) = (row as usize, t.hover_col);
                        if c >= 0 {
                            if let Some(cell) = t.cell(r, c as usize) {
                                if cell.kind == CellKind::Check { haken = true; }
                            }
                        }
                        if haken {
                            let cell = t.cell_mut(r, t.hover_col as usize).unwrap();
                            cell.value = if cell.value >= 0.5 { 0.0 } else { 1.0 };
                        }
                    }
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
                Kind::Table => self.table_keys(wi, i, g),
                k => self.widget_keys(wi, i, k, g),
            }
        }
        // Tastatur-Navigation: Tab / Shift+Tab wechselt den Fokus zwischen ALLEN
        // bedienbaren Widgets des aktiven Fensters (Anlege-Reihenfolge, nur
        // sichtbare + eingeschaltete). Frueher liefen nur die Textfelder mit --
        // Knopf, Kaestchen und Klappliste waren ohne Maus nicht erreichbar.
        if g.key_pressed(KEY_TAB) {
            if let Some(top) = self.focus_window {
                let n = self.windows[top].widgets.len();
                let mut idxs: Vec<usize> = Vec::new();
                for i in 0..n {
                    let w = &self.windows[top].widgets[i];
                    if w.kind.fokussierbar() && self.widget_shown(top, w) && w.enabled { idxs.push(i); }
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
                    self.open_dropdown = None;   // beim Weitertabben nichts offen stehen lassen
                    self.focus_widget = Some((top, ni));
                    // Nur ein Textfeld bekommt Caret + Markierung -- bei einem
                    // Knopf waeren beide bedeutungslos.
                    if self.windows[top].widgets[ni].kind.nimmt_text() {
                        let len = self.windows[top].widgets[ni].text.chars().count() as i32;
                        let w = &mut self.windows[top].widgets[ni];
                        w.caret = len; w.sel_anchor = 0;   // Inhalt markiert (wie ueblich beim Tabben)
                    }
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

        Self::einzeiler_tasten(g, &mut chars, &mut caret, &mut anchor, ctrl, shift);

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

    /// Tastatur eines EINZEILIGEN Textfelds: Tippen, Strg+A/C/V/X,
    /// Ruecktaste/Entf, Pfeile, Pos1/Ende -- jeweils mit Textauswahl
    /// (Umschalt erweitert, Tippen ersetzt das Markierte).
    ///
    /// Herausgeloest, damit das Textfeld-Widget UND der Zell-Editor der Tabelle
    /// dieselbe Logik benutzen. Vorher hatte die Zelle eine eigene, aermere
    /// Fassung ohne Auswahl -- zwei Orte fuer dieselbe Sache, an denen dieselben
    /// Fehler zweimal entstehen koennen.
    ///
    /// `chars`/`caret`/`anchor` werden an Ort und Stelle geaendert; wer sie
    /// haelt (Widget oder Zelle) entscheidet der Aufrufer.
    fn einzeiler_tasten(g: &mut Graphics, chars: &mut Vec<char>,
                        caret: &mut i32, anchor: &mut i32, ctrl: bool, shift: bool) {
        // --- Zeichen-Eingabe (nicht bei gedruecktem Strg) ---
        if !ctrl {
            let typed: String = g.pop_text_input().chars()
                .filter(|c| !c.is_control() && *c != '\t').collect();
            if !typed.is_empty() {
                let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
                if lo != hi { chars.drain(lo as usize..hi as usize); *caret = lo; }
                for (k, ch) in typed.chars().enumerate() { chars.insert(*caret as usize + k, ch); }
                *caret += typed.chars().count() as i32; *anchor = *caret;
            }
        } else {
            // Strg+A: alles markieren
            if g.key_pressed(K_A) { *anchor = 0; *caret = chars.len() as i32; }
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if g.key_pressed(K_C) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
            }
            if g.key_pressed(K_X) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
                chars.drain(lo as usize..hi as usize); *caret = lo; *anchor = lo;
            }
            if g.key_pressed(K_V) {
                let ins: Vec<char> = g.clipboard_get().chars().filter(|c| !c.is_control()).collect();
                let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
                if lo != hi { chars.drain(lo as usize..hi as usize); *caret = lo; }
                for (k, ch) in ins.iter().enumerate() { chars.insert(*caret as usize + k, *ch); }
                *caret += ins.len() as i32; *anchor = *caret;
            }
        }

        // --- Ruecktaste / Entf ---
        if g.key_pressed(KEY_BACKSPACE) {
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if lo != hi { chars.drain(lo as usize..hi as usize); *caret = lo; }
            else if *caret > 0 { chars.remove(*caret as usize - 1); *caret -= 1; }
            *anchor = *caret;
        }
        if g.key_pressed(KEY_DELETE) {
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if lo != hi { chars.drain(lo as usize..hi as usize); *caret = lo; }
            else if (*caret as usize) < chars.len() { chars.remove(*caret as usize); }
            *anchor = *caret;
        }

        // --- Navigation (Umschalt = Auswahl erweitern) ---
        let len = chars.len() as i32;
        *caret = (*caret).clamp(0, len); *anchor = (*anchor).clamp(0, len);
        if g.key_pressed(KEY_LEFT) {
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if !shift && lo != hi { *caret = lo; } else { *caret = (*caret - 1).max(0); }
            if !shift { *anchor = *caret; }
        }
        if g.key_pressed(KEY_RIGHT) {
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if !shift && lo != hi { *caret = hi; } else { *caret = (*caret + 1).min(len); }
            if !shift { *anchor = *caret; }
        }
        if g.key_pressed(KEY_HOME) { *caret = 0; if !shift { *anchor = 0; } }
        if g.key_pressed(KEY_END) { *caret = len; if !shift { *anchor = len; } }
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

    /// Wert eines Regler-artigen Widgets setzen (geklemmt) und bei Aenderung
    /// on_change feuern. Eine Stelle fuer Maus- und Tastatur-Weg.
    fn set_value_fire(&mut self, wi: usize, i: usize, roh: f64) {
        let w = &mut self.windows[wi].widgets[i];
        let nv = roh.clamp(w.min, w.max);
        if (nv - w.value).abs() > f64::EPSILON {
            w.value = nv;
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Auswahl in Liste/Klappliste um `d` Zeilen bewegen (am Rand stehenbleiben,
    /// nicht umlaufen -- beim Halten der Taste kreiste man sonst ewig) und bei
    /// Aenderung on_change feuern.
    fn liste_bewegen(&mut self, wi: usize, i: usize, d: i32) {
        let n = self.windows[wi].widgets[i].items.len() as i32;
        if n == 0 { return; }
        let w = &mut self.windows[wi].widgets[i];
        let neu = (w.sel + d).clamp(0, n - 1);
        if neu != w.sel {
            w.sel = neu;
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Die gewaehlte Zeile einer ListBox in den sichtbaren Bereich scrollen --
    /// ohne das wandert die Auswahl beim Tippen aus dem Bild heraus.
    fn liste_sichtbar(&mut self, wi: usize, i: usize) {
        let h = self.windows[wi].widgets[i].h;
        let w = &mut self.windows[wi].widgets[i];
        if w.sel < 0 { return; }
        let oben = w.sel * DROPDOWN_ITEM_H;
        let unten = oben + DROPDOWN_ITEM_H;
        let sicht = w.value as i32;
        let neu = if oben < sicht { oben }
                  else if unten > sicht + h { unten - h }
                  else { sicht };
        w.value = neu.max(0) as f64;
    }

    fn tree_setze_offen(&mut self, wi: usize, i: usize, node: i32, offen: bool) {
        if node < 0 { return; }
        if let Some(t) = self.windows[wi].widgets[i].tree.as_mut() {
            if let Some(n) = t.nodes.get_mut(node as usize) { n.expanded = offen; }
        }
    }

    /// Baum-Auswahl setzen, on_change feuern, Knoten ins Bild scrollen.
    fn tree_setze_auswahl(&mut self, wi: usize, i: usize, node: i32) {
        let geaendert = match self.windows[wi].widgets[i].tree.as_mut() {
            Some(t) if t.selected != node => { t.selected = node; true }
            _ => false,
        };
        if geaendert {
            let f = self.windows[wi].widgets[i].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
            self.tree_sichtbar(wi, i);
        }
    }

    fn tree_sichtbar(&mut self, wi: usize, i: usize) {
        let h = self.windows[wi].widgets[i].h;
        if let Some(t) = self.windows[wi].widgets[i].tree.as_mut() {
            let vis = Self::tree_visible(t);
            if let Some(p) = vis.iter().position(|&n| n as i32 == t.selected) {
                let oben = p as i32 * TREE_ROW_H;
                let unten = oben + TREE_ROW_H;
                if oben < t.scroll { t.scroll = oben; }
                else if unten > t.scroll + h { t.scroll = unten - h; }
                if t.scroll < 0 { t.scroll = 0; }
            }
        }
    }

    /// Tastatur am fokussierten Widget, das KEIN Textfeld ist.
    ///
    /// Leertaste/Enter loesen aus (Knopf, Kaestchen, Schalter, Radio), die
    /// Pfeile verstellen Werte (Regler, Drehknopf, Trenner) bzw. bewegen die
    /// Auswahl (Liste, Klappliste, Baum). Damit ist jede Art, die man anklicken
    /// kann, auch ohne Maus bedienbar -- vorher endete die Tastaturbedienung
    /// bei den Textfeldern, ein Formular liess sich also nicht abschicken, ohne
    /// zur Maus zu greifen.
    fn widget_keys(&mut self, wi: usize, i: usize, kind: Kind, g: &mut Graphics) {
        let ausloesen = g.key_pressed(KEY_SPACE) || g.key_pressed(KEY_ENTER);
        let (auf, ab) = (g.key_pressed(KEY_UP), g.key_pressed(KEY_DOWN));
        let (links, rechts) = (g.key_pressed(KEY_LEFT), g.key_pressed(KEY_RIGHT));
        match kind {
            Kind::Button => {
                if ausloesen {
                    let w = &mut self.windows[wi].widgets[i];
                    w.clicked = true;
                    let f = w.on_click.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
            Kind::Checkbox | Kind::Toggle => {
                if ausloesen {
                    let w = &mut self.windows[wi].widgets[i];
                    w.checked = !w.checked;
                    let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                    if let Some(f) = oc { self.pending.push(f); }
                    if let Some(f) = och { self.pending.push(f); }
                }
            }
            Kind::Radio => {
                if ausloesen && !self.windows[wi].widgets[i].checked {
                    self.select_radio(wi, i);
                    let w = &self.windows[wi].widgets[i];
                    let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                    if let Some(f) = oc { self.pending.push(f); }
                    if let Some(f) = och { self.pending.push(f); }
                }
            }
            Kind::Slider | Kind::Knob => {
                // Ein Zwanzigstel des Bereichs je Druck: von Anschlag zu Anschlag
                // sind es immer 20 Anschlaege, egal wie gross der Bereich ist.
                // Pos1/Ende springen an die Enden.
                let (lo, hi, v) = { let w = &self.windows[wi].widgets[i]; (w.min, w.max, w.value) };
                let schritt = (hi - lo) / 20.0;
                if rechts || auf { self.set_value_fire(wi, i, v + schritt); }
                if links || ab { self.set_value_fire(wi, i, v - schritt); }
                if g.key_pressed(KEY_HOME) { self.set_value_fire(wi, i, lo); }
                if g.key_pressed(KEY_END) { self.set_value_fire(wi, i, hi); }
            }
            Kind::Splitter => {
                let vert = self.windows[wi].widgets[i].group == "v";
                let d = if vert { rechts as i32 - links as i32 } else { ab as i32 - auf as i32 } * 8;
                if d != 0 {
                    let w = &mut self.windows[wi].widgets[i];
                    let (lo, hi) = (w.min as i32, w.max as i32);
                    let cur = if vert { w.x } else { w.y };
                    let nv = (cur + d).clamp(lo, hi);
                    if nv != cur {
                        if vert { w.x = nv; } else { w.y = nv; }
                        let f = w.on_change.clone();
                        if let Some(f) = f { self.pending.push(f); }
                    }
                }
            }
            Kind::Dropdown => {
                // Zu: Leertaste/Enter/Ab klappt auf. Offen: Pfeile waehlen,
                // Leertaste/Enter uebernimmt, ESC schliesst.
                if self.open_dropdown == Some((wi, i)) {
                    if ausloesen || g.key_pressed(KEY_ESC) { self.open_dropdown = None; }
                    else {
                        let d = ab as i32 - auf as i32;
                        if d != 0 { self.liste_bewegen(wi, i, d); }
                    }
                } else if ausloesen || ab {
                    self.open_dropdown = Some((wi, i));
                }
            }
            Kind::ListBox => {
                let d = ab as i32 - auf as i32;
                if d != 0 { self.liste_bewegen(wi, i, d); self.liste_sichtbar(wi, i); }
            }
            Kind::Tree => {
                let (vis, sel, hat_kinder, offen, eltern) = {
                    let t = match self.windows[wi].widgets[i].tree.as_ref() {
                        Some(t) => t, None => return,
                    };
                    let sel = t.selected;
                    let (hk, of, pa) = match t.nodes.get(sel.max(0) as usize) {
                        Some(n) if sel >= 0 => (n.has_children, n.expanded, n.parent),
                        _ => (false, false, -1),
                    };
                    (Self::tree_visible(t), sel, hk, of, pa)
                };
                if vis.is_empty() { return; }
                let pos = vis.iter().position(|&n| n as i32 == sel);
                let mut ziel: Option<i32> = None;
                let d = ab as i32 - auf as i32;
                if d != 0 {
                    let np = match pos {
                        Some(p) => (p as i32 + d).clamp(0, vis.len() as i32 - 1) as usize,
                        None => 0,
                    };
                    ziel = Some(vis[np] as i32);
                } else if rechts {
                    // Zu -> aufklappen, offen -> ins erste Kind. So kennt man es
                    // von jedem Dateibaum.
                    if hat_kinder && !offen { self.tree_setze_offen(wi, i, sel, true); }
                    else if let Some(p) = pos { if p + 1 < vis.len() { ziel = Some(vis[p + 1] as i32); } }
                } else if links {
                    if hat_kinder && offen { self.tree_setze_offen(wi, i, sel, false); }
                    else if eltern >= 0 { ziel = Some(eltern); }
                } else if ausloesen && hat_kinder {
                    self.tree_setze_offen(wi, i, sel, !offen);
                }
                if let Some(z) = ziel { self.tree_setze_auswahl(wi, i, z); }
            }
            _ => {}
        }
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
        // Ein Klick irgendwohin uebernimmt eine laufende Zellbearbeitung.
        // Ohne das bliebe der Editor offen stehen, waehrend die Tastatur
        // laengst woanders hinhoert -- ein Feld, das aussieht als koennte man
        // hineintippen, aber nichts mehr annimmt. Uebernehmen (nicht
        // verwerfen), weil Wegklicken ueberall sonst auch bestaetigt;
        // zuruecknehmen geht mit ESC.
        //
        // AUSSER man klickt IN das Eingabefeld selbst -- dort setzt der Klick
        // die Schreibmarke bzw. beginnt eine Markierung. Ohne diese Ausnahme
        // koennte man im eigenen Text nicht klicken, ohne ihn zu schliessen.
        if let Some((ew, ei)) = self.editing_table {
            let drin = self.edit_cell_rect(ew, ei)
                .map(|r| Self::in_rect(mx, my, r)).unwrap_or(false);
            if !drin { self.table_end_edit(ew, ei, true); }
        }
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
            // Review-Fund: derselbe haengende-Fokus-Bug wie GUI_WINDOW_VISIBLE
            // FALSE -- der Close-Button rief das bisher gar nicht auf.
            self.clear_window_interactions(win);
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
        // Fokus folgt dem Klick -- an EINER Stelle fuer alle Arten, damit ein
        // spaeteres Tab dort weiterlaeuft, wo die Maus zuletzt war. Deko faengt
        // keinen Fokus (fokussierbar() ist die eine Quelle).
        let kind = self.windows[win].widgets[i].kind;
        self.focus_widget = if kind.fokussierbar() { Some((win, i)) } else { None };
        match kind {
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
            Kind::Toggle => {
                let w = &mut self.windows[win].widgets[i];
                w.checked = !w.checked;
                let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                if let Some(f) = oc { self.pending.push(f); }
                if let Some(f) = och { self.pending.push(f); }
            }
            Kind::Knob => { self.active_knob = Some((win, i, my, self.windows[win].widgets[i].value)); }
            Kind::Splitter => {
                self.active_split = Some((win, i));
                let vert = self.windows[win].widgets[i].group == "v";
                let (ax, ay, _w, _h) = self.abs_rect(win, &self.windows[win].widgets[i]);
                self.split_off = if vert { mx - ax } else { my - ay };
            }
            Kind::Spinner => {
                let (ax, ay, ww, hh) = self.abs_rect(win, &self.windows[win].widgets[i]);
                let (up, down, _bx) = Self::spinner_btn_rects(ax, ay, ww, hh);
                if Self::in_rect(mx, my, up) { self.spinner_step(win, i, 1); }
                else if Self::in_rect(mx, my, down) { self.spinner_step(win, i, -1); }
            }
            Kind::TextInput | Kind::TextArea => {}   // Caret setzt die Editier-Routine
            Kind::Table => self.table_press(win, i, mx, my),
            Kind::Tree => self.tree_press(win, i, mx, my),
            Kind::Radio => {
                let was = self.windows[win].widgets[i].checked;
                self.select_radio(win, i);
                if !was {
                    let w = &self.windows[win].widgets[i];
                    let oc = w.on_click.clone(); let och = w.on_change.clone();
                    if let Some(f) = oc { self.pending.push(f); }
                    if let Some(f) = och { self.pending.push(f); }
                }
            }
            Kind::Dropdown => self.open_dropdown = Some((win, i)),
            Kind::ListBox => {
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
            _ => {}
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
                let eigen = if wdg.color < 0 { self.th("text_fg") } else { wdg.color };
                let fg = if !wdg.enabled { self.th("muted_fg") }
                         else { wdg.ov.get("fg").copied().unwrap_or(eigen) };
                self.wtext(g, wdg, ax, ay, wdg.text.clone(), fg);
            }
            Kind::Panel => {
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
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
                        let rad = if wdg.rund { (w.min(h) / 2).max(2) } else { self.m("corner_radius").min(6) };
                        g.round_rect(ax, ay, ax + w - 1, ay + h - 1, rad, c, true);
                    }
                } else if wdg.rund {
                    // Runder Knopf: der Eckenradius ist die halbe kurze Seite,
                    // damit aus dem Rechteck eine Kapsel bzw. ein Kreis wird.
                    let mut bg = self.wcol(wdg, "bg", "widget_bg");
                    if pressed { bg = shade(bg, -30); } else if wdg.hovered { bg = shade(bg, 30); }
                    let r = (w.min(h) / 2).max(2);
                    let gr = self.m("gradient");
                    if gr > 0 {
                        g.round_gradient(ax, ay, ax + w - 1, ay + h - 1, r, shade(bg, gr), shade(bg, -gr));
                        self.gloss(g, ax, ay, ax + w - 1, ay + h - 1, r);
                    } else {
                        g.round_rect(ax, ay, ax + w - 1, ay + h - 1, r, bg, true);
                    }
                    g.round_rect(ax, ay, ax + w - 1, ay + h - 1, r,
                                 self.wcol(wdg, "border", "widget_border"), false);
                } else {
                    let mut bg = self.wcol(wdg, "bg", "widget_bg");
                    if pressed { bg = shade(bg, -30); } else if wdg.hovered { bg = shade(bg, 30); }
                    self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, bg, self.wcol(wdg, "border", "widget_border"));
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
                    // Beschriftung mittig -- vorher klebte sie am linken Rand.
                    // Mit Sinnbild bleibt der Platz dafuer links stehen und
                    // der Text zentriert sich im Rest.
                    let sz = self.wsize(g, wdg);
                    let tw = self.wtext_width(g, wdg, &wdg.text);
                    let links = if has_icon { pad + isz + 4 } else { pad };
                    let frei = (w - links - pad).max(0);
                    let (versatz, beschnitten) = knopf_text_x(frei, tw);
                    let tx = ax + links + versatz;
                    let ty = ay + (h - sz) / 2;
                    if beschnitten {
                        g.push_clip(ax + links, ay + 1, frei, (h - 2).max(0));
                    }
                    self.wtext(g, wdg, tx, ty, wdg.text.clone(), self.txt_col(wdg));
                    if beschnitten {
                        g.pop_clip();
                    }
                }
            }
            Kind::Checkbox => {
                let acc = self.acc_col(wdg);
                let bordc = self.wcol(wdg, "border", "widget_border");
                if self.modern() {
                    // Gerundetes Kaestchen; gefuellt + Haekchen wenn aktiv.
                    if wdg.checked {
                        if self.m("gradient") > 0 {
                            g.round_gradient(ax, ay, ax + w - 1, ay + h - 1, 3,
                                             shade(acc, 26), shade(acc, -22));
                            self.gloss(g, ax, ay, ax + w - 1, ay + h - 1, 3);
                        } else {
                            g.round_rect(ax, ay, ax + w - 1, ay + h - 1, 3, acc, true);
                        }
                        let ck = self.th("win_bg");   // Kontrast-Haekchen auf Akzentflaeche
                        let (x1, y1) = (ax + w * 28 / 100, ay + h * 52 / 100);
                        let (x2, y2) = (ax + w * 44 / 100, ay + h * 70 / 100);
                        let (x3, y3) = (ax + w * 74 / 100, ay + h * 30 / 100);
                        g.line(x1, y1, x2, y2, ck);
                        g.line(x2, y2, x3, y3, ck);
                    } else {
                        // Leer = versenkte Mulde: man sieht, dass hier etwas
                        // hineingehoert.
                        self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
                                       self.wcol(wdg, "bg", "widget_bg"),
                                       if wdg.hovered { acc } else { bordc });
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
                let span = wdg.max - wdg.min;
                let ratio = if span != 0.0 { (wdg.value - wdg.min) / span } else { 0.0 };
                let acc = self.acc_col(wdg);
                if self.m("gradient") > 0 {
                    // Rinne: schmal und versenkt. Der zurueckgelegte Teil wird
                    // in der Akzentfarbe gefuellt -- so ist der Wert auch ohne
                    // Zahl ablesbar.
                    let (ty, tb) = (ay + h / 2 - 3, ay + h / 2 + 3);
                    self.fbox_tief(g, ax, ty, ax + w - 1, tb,
                                   self.wcol(wdg, "bg", "widget_bg"),
                                   self.wcol(wdg, "border", "widget_border"));
                    let bis = ax + (ratio * (w - 1) as f64) as i32;
                    if bis > ax + 2 {
                        g.round_gradient(ax + 1, ty + 1, bis, tb - 1, 3,
                                         shade(acc, 24), shade(acc, -18));
                    }
                    let hx = ax + (ratio * (w - handle_w) as f64) as i32 + handle_w / 2;
                    self.knauf(g, hx, ay + h / 2, (h / 2).max(5), self.griff_farbe(wdg));
                } else {
                    g.box_fill(ax, ay + h / 2 - 1, ax + w - 1, ay + h / 2 + 1, self.wcol(wdg, "bg", "widget_bg"));
                    g.rect(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "border", "widget_border"));
                    let hx = ax + (ratio * (w - handle_w) as f64) as i32;
                    g.box_fill(hx, ay, hx + handle_w - 1, ay + h - 1, acc);
                }
            }
            Kind::Toggle => {
                let acc = self.acc_col(wdg);
                let aus = self.wcol(wdg, "bg", "widget_bg");
                let r = h / 2;
                // Die Rinne faerbt sich mit dem Zustand ein -- ueber `value`,
                // also gleitend statt springend.
                let mix = wdg.value.clamp(0.0, 1.0);
                let bahn = mischen(aus, acc, mix);
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, bahn,
                               self.wcol(wdg, "border", "widget_border"));
                let kx = ax + r + ((w - h) as f64 * mix) as i32;
                self.knauf(g, kx, ay + r, r - 2, self.griff_farbe(wdg));
                if !wdg.text.is_empty() {
                    self.wtext(g, wdg, ax + w + pad, ay + (h - 14) / 2, wdg.text.clone(), self.txt_col(wdg));
                }
            }
            Kind::Knob => {
                let acc = self.acc_col(wdg);
                let r = (w.min(h) / 2).max(8);
                let (cx, cy) = (ax + w / 2, ay + h / 2);
                let spanne = wdg.max - wdg.min;
                let anteil = if spanne != 0.0 { ((wdg.value - wdg.min) / spanne).clamp(0.0, 1.0) } else { 0.0 };
                // Skalenbogen: 270 Grad, unten offen -- wie am Mischpult.
                let (von, bis) = (135.0, 405.0);
                let zw = von + (bis - von) * anteil;
                g.ring(cx, cy, r - 3, r, von, bis, self.wcol(wdg, "border", "widget_border"), true);
                if anteil > 0.0 {
                    g.ring(cx, cy, r - 3, r, von, zw, acc, true);
                }
                // Metallkappe + Kerbe, die den Wert zeigt.
                self.knauf(g, cx, cy, r - 6, self.griff_farbe(wdg));
                let rad = zw.to_radians();
                let (i0, i1) = ((r - 14).max(2) as f64, (r - 7).max(3) as f64);
                g.line_thick(
                    cx + (rad.cos() * i0) as i32, cy + (rad.sin() * i0) as i32,
                    cx + (rad.cos() * i1) as i32, cy + (rad.sin() * i1) as i32,
                    2.0, self.txt_col(wdg),
                );
                if !wdg.text.is_empty() {
                    let tw = g.text_width(&wdg.text);
                    self.wtext(g, wdg, cx - tw / 2, ay + h + 2, wdg.text.clone(), self.txt_col(wdg));
                }
            }
            Kind::TextInput => {
                let focused = self.focus_widget == Some((wi, idx));
                let fg = self.txt_col(wdg);
                let bcol = if focused { self.wcol(wdg, "accent", "accent") } else { self.wcol(wdg, "border", "widget_border") };
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"), bcol);
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
                            g.box_fill(x0, ay + 2, x1, ay + h - 3, self.selection_bg(wdg));
                        }
                    }
                    self.wtext(g, wdg, tx - scroll, ty, wdg.text.clone(), fg);
                }
                // Caret (blinkend) an der gemessenen Position.
                if focused && self.caret_blink_on() {
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
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"), bcol);
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
                        let selbg = self.selection_bg(wdg);
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
                if focused && self.caret_blink_on() {
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
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"), bcol);
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
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
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
                self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, b, self.wcol(wdg, "border", "widget_border"));
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
                self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
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
                self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
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
            Kind::Tree => self.draw_tree(g, wi, idx),
        }
        // Fokus-Ring: EINE Stelle fuer alle Arten, nach dem Zeichnen und damit
        // obenauf. Liegt AUSSERHALB der Widget-Flaeche (2 px Luft), sonst
        // verdeckte er Inhalt -- ein Kaestchen ist nur check_size gross, ein
        // Ring darin waere kaum zu sehen. Ohne sichtbaren Fokus waere die
        // Tab-Navigation wertlos: man wuesste nie, wo man gerade ist.
        if self.focus_widget == Some((wi, idx)) && wdg.kind.fokussierbar() {
            g.rect(ax - 3, ay - 3, ax + w + 2, ay + h + 2, self.th("accent"));
        }
    }

    fn draw_tree(&self, g: &mut Graphics, wi: usize, idx: usize) {
        let wdg = &self.windows[wi].widgets[idx];
        let (ax, ay, w, h) = self.abs_rect(wi, wdg);
        self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
            self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
        let t = wdg.tree.as_ref().unwrap();
        let vis = Self::tree_visible(t);
        let fg = self.txt_col(wdg);
        let acc = self.acc_col(wdg);
        let scroll = t.scroll;
        g.push_clip(ax + 1, ay + 1, w - 2, h - 2);
        for (r, &ni) in vis.iter().enumerate() {
            let ry = ay + 1 + r as i32 * TREE_ROW_H - scroll;
            if ry + TREE_ROW_H < ay || ry > ay + h { continue; }
            let node = &t.nodes[ni];
            let indent = node.level * TREE_INDENT;
            if ni as i32 == t.selected {
                g.box_fill(ax + 1, ry, ax + w - 2, ry + TREE_ROW_H - 1, shade(acc, -110));
            } else if ni as i32 == t.hover {
                g.box_fill(ax + 1, ry, ax + w - 2, ry + TREE_ROW_H - 1, shade(self.wcol(wdg, "bg", "widget_bg"), 18));
            }
            // Auf-/Zuklapp-Dreieck (nur bei Kindknoten).
            let tx = ax + 4 + indent;
            let cy = ry + TREE_ROW_H / 2;
            if node.has_children {
                let cx = tx + TREE_TOGGLE_W / 2;
                if node.expanded {
                    g.line(cx - 4, cy - 2, cx, cy + 2, fg);    // v
                    g.line(cx, cy + 2, cx + 4, cy - 2, fg);
                } else {
                    g.line(cx - 2, cy - 4, cx + 2, cy, fg);    // >
                    g.line(cx + 2, cy, cx - 2, cy + 4, fg);
                }
            }
            let lx = tx + TREE_TOGGLE_W + 2;
            self.wtext(g, wdg, lx, ry + (TREE_ROW_H - 14) / 2, node.label.clone(), fg);
        }
        g.pop_clip();
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

    /// Flaechenfarbe einer Zelle: eigene Farbe > Zeilenfarbe > Zebra > nichts.
    /// -1 heisst jeweils "nicht gesetzt" und reicht an die naechste Stufe
    /// weiter -- so faerbt eine Zeilenfarbe alle Zellen, eine einzelne Zelle
    /// kann aber ausscheren.
    fn cell_bg(t: &TableState, r: usize, c: usize, zebra_bg: i64) -> i64 {
        if let Some(cell) = t.cell(r, c) {
            if cell.bg >= 0 { return cell.bg; }
        }
        if let Some(&rb) = t.row_bg.get(r) {
            if rb >= 0 { return rb; }
        }
        zebra_bg
    }
    fn cell_fg(t: &TableState, r: usize, c: usize, thema: i64) -> i64 {
        if let Some(cell) = t.cell(r, c) {
            if cell.fg >= 0 { return cell.fg; }
        }
        if let Some(&rf) = t.row_fg.get(r) {
            if rf >= 0 { return rf; }
        }
        thema
    }

    /// x-Lage von Text in einer Zelle nach Ausrichtung. Getrennt, weil Kopf-
    /// und Datenzellen dieselbe Regel brauchen.
    fn align_x(links: i32, breite: i32, textbreite: i32, align: i8) -> i32 {
        match align {
            1 => links + (breite - textbreite) / 2,
            2 => links + breite - textbreite - TBL_PADDING,
            _ => links + TBL_PADDING,
        }
        .max(links + 2)
    }

    /// Kleines Dreieck als Sortier-Anzeige. Ohne sie sieht man der Tabelle
    /// nicht an, WONACH sie gerade sortiert ist.
    fn sort_arrow(g: &mut Graphics, cx: i32, cy: i32, desc: bool, c: i64) {
        // Aufsteigend zeigt der Pfeil NACH OBEN, absteigend nach unten -- so
        // herum kennt man es von jeder Tabelle. (Erst herum: die Spitze zeigte
        // genau falsch, was im Bild sofort auffiel.)
        let (a, b) = if desc { (3, -2) } else { (-3, 2) };
        g.line(cx - 4, cy + b, cx, cy + a + b, c);
        g.line(cx, cy + a + b, cx + 4, cy + b, c);
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
        let zebra_bg = shade(bg, 10);
        let grid_c = shade(bg, 14);
        let schrift = self.wsize(g, wdg);

        // Aussenrahmen + Kopfzeile.
        g.box_fill(ax, ay, ax + w - 1, ay + h - 1, bg);
        g.rect(ax, ay, ax + w - 1, ay + h - 1, border);
        if gm.header_h > 0 {
            let titel_h = t.header_h;
            g.box_fill(ax + 1, ay + 1, ax + w - 2, ay + gm.header_h - 1, self.th("title_bg"));
            g.line(ax, ay + gm.header_h - 1, ax + w - 1, ay + gm.header_h - 1, border);

            // Kopf-Zellen. Feste Spalten stehen still, der Rest scrollt --
            // beides ueber `col_x`, damit Kopf und Daten nie versetzt sind.
            g.push_clip(ax + 1, ay + 1, gm.body_w_raw, gm.header_h - 2);
            for p in 0..gm.n_cols {
                let c = gm.order[p];                 // Anzeige-Position -> Datenspalte
                let cx = Self::col_x(&gm, p);
                let cw = gm.col_widths[p];
                let (kx, kw) = Self::col_clip(&gm, p);
                if cx + cw > kx && cx < kx + kw {
                    g.push_clip(kx, ay + 1, kw, gm.header_h - 2);
                    // Die Spalte, die gerade gezogen wird, hebt sich ab --
                    // sonst sieht man beim Verschieben nicht, was man haelt.
                    if let Some((dp, _, true)) = t.head_drag {
                        if dp == p {
                            g.box_fill(cx, ay + 1, cx + cw - 1, ay + titel_h - 2,
                                       (90i64 << 24) | (accent & 0xFFFFFF));
                        }
                    }
                    let s = t.header(c).to_string();
                    let al = *t.col_align.get(c).unwrap_or(&0);
                    // Platz fuer den Sortierpfeil freihalten, sonst schreibt
                    // ein langer Titel darueber.
                    let pfeil_platz = if t.sort_col == c as i32 { 14 } else { 0 };
                    let tw = self.wtext_width(g, wdg, &s);
                    let tx = Self::align_x(cx, cw - pfeil_platz, tw, al);
                    let ty = ay + (titel_h - schrift).max(0) / 2;
                    self.wtext(g, wdg, tx, ty, s, title_fg);
                    if t.sort_col == c as i32 {
                        Self::sort_arrow(g, cx + cw - 11, ay + titel_h / 2, t.sort_desc, title_fg);
                    }
                    if p < gm.n_cols - 1 {
                        g.line(cx + cw, ay + 1, cx + cw, ay + gm.header_h - 2, border);
                    }
                    g.pop_clip();
                }
            }
            // Filterzeile: je Spalte ein kleines Eingabefeld.
            if t.filter_row {
                let fy = ay + titel_h;
                for p in 0..gm.n_cols {
                    let c = gm.order[p];
                    let cx = Self::col_x(&gm, p);
                    let cw = gm.col_widths[p];
                    let (kx, kw) = Self::col_clip(&gm, p);
                    if cx + cw > kx && cx < kx + kw {
                        g.push_clip(kx, fy, kw, TBL_FILTER_H);
                        let fokus = t.filter_focus == c as i32;
                        self.fbox_tief(g, cx + 2, fy + 2, cx + cw - 3, fy + TBL_FILTER_H - 3,
                                       self.th("field_bg"),
                                       if fokus { accent } else { self.th("field_border") });
                        let leer = t.filters.get(c).map(|s| s.is_empty()).unwrap_or(true);
                        let s = if leer { "Filter ...".to_string() }
                                else { t.filters[c].clone() };
                        let farbe = if leer { shade(title_fg, -60) } else { fg };
                        g.push_clip(cx + 4, fy + 2, (cw - 8).max(1), TBL_FILTER_H - 4);
                        self.wtext(g, wdg, cx + 6, fy + (TBL_FILTER_H - schrift).max(0) / 2,
                                   s, farbe);
                        g.pop_clip();
                        g.pop_clip();
                    }
                }
            }
            g.pop_clip();
            // Kante des festen Blocks. NUR wenn wirklich seitwaerts gescrollt
            // ist -- bei stehender Tabelle erklaert sie nichts und waere eine
            // willkuerliche Markierung. Und im Rahmenton, nicht im Akzent: der
            // bedeutet ueberall sonst "ausgewaehlt/aktiv", hier hiesse er
            // "ab hier scrollt es" -- zwei Dinge in einer Farbe.
            if gm.frozen > 0 && t.scroll_x > 0 {
                let fx = body_x + gm.frozen_w;
                g.line(fx, ay + 1, fx, ay + gm.header_h - 2, shade(border, 40));
            }
        }

        // Body.
        g.push_clip(body_x, body_y, body_w, body_h);
        let first = (t.scroll_y / gm.row_h).max(0);
        let last = ((t.scroll_y + body_h) / gm.row_h + 1).min(gm.n_rows as i32);
        for r in first..last {
            // r = Ansichtszeile (Position auf dem Schirm), ri = Datenzeile.
            // Farben, Inhalte und die Auswahl haengen an den DATEN, die Lage
            // an der Ansicht -- werden die verwechselt, faerbt eine sortierte
            // Tabelle die falschen Zeilen.
            let ri = match t.view.get(r as usize) { Some(&x) => x, None => continue };
            let row_y = body_y + r * gm.row_h - t.scroll_y;
            // Zebra nach der ANSICHTSzeile, nicht nach der Datenzeile --
            // sonst waeren die Streifen nach dem Filtern unregelmaessig.
            let zebra = if t.zebra && r % 2 == 1 { zebra_bg } else { -1 };

            // Zellflaechen zuerst, danach Auswahl/Hover DARUEBER -- sonst
            // deckte eine eigene Zellfarbe die Auswahl zu und man saehe nicht
            // mehr, welche Zeile gewaehlt ist.
            for p in 0..gm.n_cols {
                let c = gm.order[p];
                let cx = Self::col_x(&gm, p);
                let cw = gm.col_widths[p];
                let (kx, kw) = Self::col_clip(&gm, p);
                if cx + cw > kx && cx < kx + kw {
                    let cb = Self::cell_bg(t, ri, c, zebra);
                    if cb >= 0 {
                        g.push_clip(kx, row_y, kw, gm.row_h);
                        g.box_fill(cx, row_y, cx + cw - 1, row_y + gm.row_h - 1, cb);
                        g.pop_clip();
                    }
                }
            }
            // Halbdurchsichtig ueber die Zellflaechen: eine eigene Zellfarbe
            // bleibt sichtbar, die Auswahl aber auch. Alpha steckt im
            // hoechsten Byte (0xAARRGGBB).
            if t.ist_gewaehlt(ri as i32) {
                g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + gm.row_h - 1,
                           (150i64 << 24) | (sel_bg & 0xFFFFFF));
            } else if ri as i32 == t.hover_row {
                g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + gm.row_h - 1,
                           (110i64 << 24) | (hover_bg & 0xFFFFFF));
            }

            // Inhalte.
            for p in 0..gm.n_cols {
                let c = gm.order[p];
                let cx = Self::col_x(&gm, p);
                let cw = gm.col_widths[p];
                let (kx, kw) = Self::col_clip(&gm, p);
                let clip_x = (cx + 1).max(kx);
                let clip_w = (cw - 2).min((kx + kw) - clip_x);
                if clip_w > 0 {
                    g.push_clip(clip_x, row_y, clip_w, gm.row_h);
                    self.draw_cell(g, wdg, t, ri, c, cx, row_y, cw, gm.row_h, fg, accent, border);
                    g.pop_clip();
                }
                if t.grid && p < gm.n_cols - 1 && cx + cw > kx && cx + cw < kx + kw {
                    g.line(cx + cw, row_y, cx + cw, row_y + gm.row_h - 1, grid_c);
                }
            }
            if t.grid {
                g.line(body_x, row_y + gm.row_h - 1, body_x + body_w - 1, row_y + gm.row_h - 1, grid_c);
            }
        }
        if gm.frozen > 0 && t.scroll_x > 0 {
            let fx = body_x + gm.frozen_w;
            g.line(fx, body_y, fx, body_y + body_h - 1, shade(border, 40));
        }
        g.pop_clip();

        // Senkrechte Bildlaufleiste.
        if gm.need_v {
            let sb_x = ax + gm.body_w_raw - TBL_SCROLL_W + 1;
            let track = body_h;
            let handle = Self::handle_height(track, body_h, gm.total_h);
            let ratio = if gm.max_scroll_y > 0 { t.scroll_y as f64 / gm.max_scroll_y as f64 } else { 0.0 };
            let hy = body_y + ((track - handle) as f64 * ratio) as i32;
            g.box_fill(sb_x, body_y, sb_x + TBL_SCROLL_W - 1, body_y + track - 1, shade(bg, -8));
            g.box_fill(sb_x + 2, hy, sb_x + TBL_SCROLL_W - 3, hy + handle - 1,
                       if t.drag_v { accent } else { border });
        }
        // Waagerechte Bildlaufleiste.
        if gm.need_h {
            let hs_y = ay + h - TBL_SCROLL_W - 1;
            let track = body_w;
            let handle = Self::handle_height(track, body_w, gm.total_w);
            let ratio = if gm.max_scroll_x > 0 { t.scroll_x as f64 / gm.max_scroll_x as f64 } else { 0.0 };
            let hx = body_x + ((track - handle) as f64 * ratio) as i32;
            g.box_fill(body_x, hs_y, body_x + track - 1, hs_y + TBL_SCROLL_W - 1, shade(bg, -8));
            g.box_fill(hx, hs_y + 2, hx + handle - 1, hs_y + TBL_SCROLL_W - 3,
                       if t.drag_h { accent } else { border });
        }
    }

    /// Eine einzelne Zelle zeichnen. Getrennt, weil sonst fuenf Zellarten in
    /// der ohnehin langen Zeichenschleife stehen wuerden.
    #[allow(clippy::too_many_arguments)]
    fn draw_cell(&self, g: &mut Graphics, wdg: &Widget, t: &TableState,
                 r: usize, c: usize, cx: i32, cy: i32, cw: i32, ch: i32,
                 fg_thema: i64, accent: i64, border: i64) {
        let leer = Cell::text(String::new());
        let cell = t.cell(r, c).unwrap_or(&leer);
        let fgc = Self::cell_fg(t, r, c, fg_thema);
        let al = t.align_of(r, c);
        let schrift = self.wsize(g, wdg);

        // Wird DIESE Zelle gerade bearbeitet? Dann ein Eingabefeld statt Text.
        if t.edit_row == r as i32 && t.edit_col == c as i32 {
            self.fbox_tief(g, cx + 1, cy + 2, cx + cw - 2, cy + ch - 3,
                           self.th("field_bg"), accent);
            let alle: Vec<char> = t.edit_text.chars().collect();
            let tx = cx + 5 - t.edit_scroll;
            let ty = cy + (ch - schrift).max(0) / 2;
            g.push_clip(cx + 3, cy + 2, (cw - 6).max(1), ch - 4);

            // Markierter Bereich als Flaeche HINTER dem Text -- ohne sie sieht
            // man nicht, was das naechste Zeichen ersetzen wuerde.
            let (lo, hi) = (t.edit_caret.min(t.edit_anchor).clamp(0, alle.len() as i32),
                            t.edit_caret.max(t.edit_anchor).clamp(0, alle.len() as i32));
            if lo != hi {
                let vor: String = alle[..lo as usize].iter().collect();
                let mitte: String = alle[lo as usize..hi as usize].iter().collect();
                let x1 = tx + self.wtext_width(g, wdg, &vor);
                let x2 = x1 + self.wtext_width(g, wdg, &mitte);
                g.box_fill(x1, cy + 4, x2 - 1, cy + ch - 5,
                           (140i64 << 24) | (accent & 0xFFFFFF));
            }
            self.wtext(g, wdg, tx, ty, t.edit_text.clone(), fg_thema);
            // Schreibmarke -- ohne sie sieht man nicht, wo man tippt.
            let bis: String = alle[..t.edit_caret.clamp(0, alle.len() as i32) as usize].iter().collect();
            let vx = tx + self.wtext_width(g, wdg, &bis);
            g.line(vx, cy + 4, vx, cy + ch - 5, accent);
            g.pop_clip();
            return;
        }

        match cell.kind {
            CellKind::Text => {
                let tw = self.wtext_width(g, wdg, &cell.text);
                let tx = Self::align_x(cx, cw, tw, al);
                self.wtext(g, wdg, tx, cy + (ch - schrift).max(0) / 2, cell.text.clone(), fgc);
            }
            CellKind::Image => {
                if cell.image >= 0 {
                    // In die Zelle einpassen, Seitenverhaeltnis behalten --
                    // gestreckte Bilder in einer Tabelle sehen kaputt aus.
                    let iw = g.image_width(cell.image).unwrap_or(0);
                    let ih = g.image_height(cell.image).unwrap_or(0);
                    if iw > 0 && ih > 0 {
                        let rand = 2;
                        let max_w = (cw - 2 * rand).max(1);
                        let max_h = (ch - 2 * rand).max(1);
                        let s = (max_w as f64 / iw as f64).min(max_h as f64 / ih as f64).min(4.0);
                        let dw = ((iw as f64 * s) as i32).max(1);
                        let dh = ((ih as f64 * s) as i32).max(1);
                        let dx = match al { 1 => cx + (cw - dw) / 2, 2 => cx + cw - dw - rand, _ => cx + rand };
                        g.draw_image_rect(cell.image, dx, cy + (ch - dh) / 2, dw, dh);
                    }
                }
            }
            CellKind::Check => {
                let b = (ch - 8).clamp(10, 18);
                let bx = match al { 1 => cx + (cw - b) / 2, 2 => cx + cw - b - TBL_PADDING, _ => cx + TBL_PADDING };
                let by = cy + (ch - b) / 2;
                g.box_fill(bx, by, bx + b - 1, by + b - 1, self.th("field_bg"));
                g.rect(bx, by, bx + b - 1, by + b - 1, border);
                if cell.value >= 0.5 {
                    // Haken als zwei Striche -- ein gefuelltes Kaestchen
                    // liesse sich von einer eingefaerbten Zelle nicht trennen.
                    let m = b / 2;
                    g.line(bx + 3, by + m, bx + m - 1, by + b - 4, accent);
                    g.line(bx + m - 1, by + b - 4, bx + b - 3, by + 3, accent);
                }
            }
            CellKind::Bar => {
                let rand = 3;
                let bw = (cw - 2 * rand).max(1);
                let bh = (ch - 2 * rand).clamp(6, 22);
                let bx = cx + rand;
                let by = cy + (ch - bh) / 2;
                let v = cell.value.clamp(0.0, 1.0);
                g.box_fill(bx, by, bx + bw - 1, by + bh - 1, self.th("field_bg"));
                if v > 0.0 {
                    let fw = ((bw - 2) as f64 * v) as i32;
                    if fw > 0 {
                        let farbe = if cell.fg >= 0 { cell.fg } else { accent };
                        g.box_fill(bx + 1, by + 1, bx + fw, by + bh - 2, farbe);
                    }
                }
                g.rect(bx, by, bx + bw - 1, by + bh - 1, border);
                if !cell.text.is_empty() {
                    let tw = self.wtext_width(g, wdg, &cell.text);
                    self.wtext(g, wdg, bx + (bw - tw) / 2, by + (bh - schrift).max(0) / 2,
                               cell.text.clone(), fg_thema);
                }
            }
            CellKind::Button => {
                let rand = 3;
                let bx = cx + rand;
                let by = cy + rand;
                let bw = (cw - 2 * rand).max(1);
                let bh = (ch - 2 * rand).max(1);
                self.fbox(g, bx, by, bx + bw - 1, by + bh - 1,
                          self.th("widget_bg"), border);
                let tw = self.wtext_width(g, wdg, &cell.text);
                self.wtext(g, wdg, bx + (bw - tw) / 2, by + (bh - schrift).max(0) / 2,
                           cell.text.clone(), fgc);
            }
        }
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
        assert!(!g.windows[tw].widgets[ti].enabled);                     // dekorativ
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
