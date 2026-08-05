//! Modul `chart` -- Diagramme: Kuchen/Donut, Balken, Linie/Flaeche, Tacho.
//!
//! Handle-basiert wie `gui`/`particles`: einmal aufbauen und einstellen, dann
//! pro Bild `CHART_DRAW(c)`. Der Zustand (Daten + Stil) liegt in `ChartObj`;
//! gezeichnet wird ueber die vorhandenen Graphics-Primitive (`ring` fuer
//! Kuchenstuecke und Tacho-Boegen, sonst Rechtecke/Linien/Text).
//!
//! **Warum String-Schluessel fuer den Stil?** Es sind ~40 Einstellungen. Als je
//! eigenes Builtin waeren das 40 Eintraege in `builtin_index.json` -- stattdessen
//! vier Setter (`CHART_SET`/`_NUM`/`_COLOR`/`_FLAG`), die den Schluessel auf ein
//! Feld von `Style` abbilden. Ein unbekannter Schluessel ist ein Fehler im
//! Klartext, kein stilles Verschlucken.
//!
//! Winkel durchgehend in Grad, 0 = rechts, wachsend im Uhrzeigersinn (raylib,
//! Bildschirm-y nach unten).

use std::collections::HashMap;

#[cfg(feature = "graphics")]
use crate::graphics::Graphics;

/// Diagrammart. `CHART_NEW` nimmt sie als String entgegen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Pie,
    Bar,
    Line,
    Gauge,
    /// Durchgehende Leiste mit Marker -- der lineare Bruder des Tachos.
    Leiste,
    /// Diskrete Zellen (Lampen), die bis zum Wert leuchten.
    Led,
}

impl Kind {
    pub fn parse(s: &str) -> Option<Kind> {
        match s.to_lowercase().as_str() {
            "kuchen" | "pie" | "donut" => Some(Kind::Pie),
            "balken" | "bar" => Some(Kind::Bar),
            "linie" | "line" | "flaeche" | "area" => Some(Kind::Line),
            "tacho" | "gauge" => Some(Kind::Gauge),
            "leiste" | "balkenanzeige" | "bar_gauge" => Some(Kind::Leiste),
            "led" | "lampen" | "zellen" => Some(Kind::Led),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Pie => "kuchen",
            Kind::Bar => "balken",
            Kind::Line => "linie",
            Kind::Gauge => "tacho",
            Kind::Leiste => "leiste",
            Kind::Led => "led",
        }
    }
}

/// Eine Datenreihe (Balken-/Liniendiagramm: mehrere nebeneinander; Kuchen und
/// Tacho nutzen nur die erste).
pub struct Series {
    pub name: String,
    pub color: i64,
    pub values: Vec<f64>,
    /// Angezeigte Werte -- ziehen bei `animation > 0` den echten hinterher.
    pub shown: Vec<f64>,
}

/// Farbzone auf der Tacho-Skala (roter Bereich & Co.).
pub struct Zone {
    pub from: f64,
    pub to: f64,
    pub color: i64,
    /// Beschriftung, die entlang des Bogens mitgedreht wird ("" = keine).
    pub name: String,
}

/// Alle Aussehens-Einstellungen. Voreinstellungen = Thema "dunkel".
pub struct Style {
    // --- Text (CHART_SET) ---
    pub titel: String,
    pub einheit: String,
    pub achse_x: String,
    pub achse_y: String,
    /// "aus" | "oben" | "unten" | "links" | "rechts"
    pub legende: String,
    /// "aus" | "innen" | "aussen"
    pub werte: String,
    /// "senkrecht" | "waagerecht" (Balken)
    pub ausrichtung: String,
    /// "nadel" | "balken" | "pfeil" (Tacho)
    pub zeigerform: String,
    /// Tacho-Zifferblatt: "ring" | "segmente" | "striche" | "baender"
    pub zifferblatt: String,
    /// Wo der Wert steht: "aus" | "innen" | "pille" | "blase" | "am_zeiger"
    pub wertanzeige: String,

    // --- Zahlen (CHART_SET_NUM) ---
    /// NaN = automatisch aus den Daten.
    pub min: f64,
    pub max: f64,
    /// Anteil des Aussenradius (0..0.95). > 0 macht aus dem Kuchen einen Donut.
    pub innenradius: f64,
    /// Kuchen: Segmente nach aussen versetzt. Balken: Luecke zwischen Balken.
    pub abstand: f64,
    pub ecken: i32,
    pub rahmen_dicke: i32,
    pub polster: i32,
    /// Schrittweite der Wertachse; 0 = automatisch.
    pub gitter: f64,
    pub nachkomma: i32,
    pub titel_groesse: i32,
    pub text_groesse: i32,
    /// FONT-Handle aus LOADFONT; -1 = Standardschrift.
    pub schrift: i64,
    pub start_winkel: f64,
    pub end_winkel: f64,
    pub striche: i32,
    pub unterstriche: i32,
    pub linien_dicke: f64,
    pub punkt_radius: i32,
    /// Sekunden, die der angezeigte Wert dem echten hinterherzieht; 0 = sofort.
    pub animation: f64,
    /// Gleitendes Fenster fuer CHART_PUSH; 0 = unbegrenzt.
    pub fenster: i32,
    /// Versatz des Schlagschattens in Pixeln; 0 = kein Schatten.
    pub schatten: i32,
    /// Weichzeichnung des Schattens in Pixeln (gestaffelte Kopien mit
    /// abnehmender Deckkraft); 0 = harte Kante.
    pub schatten_weich: i32,
    /// Deckkraft ALLER Datenfarben (Balken/Segmente/Linien), 0..1.
    pub deckkraft: f64,
    /// Deckkraft der Flaeche unter einer Linie, 0..1 -- getrennt regelbar,
    /// weil eine Flaeche fast immer durchscheinender sein soll als ihre Linie.
    pub flaeche_deckkraft: f64,

    // --- Farben (CHART_SET_COLOR) ---
    pub c_hintergrund: i64,
    pub c_rahmen: i64,
    pub c_gitter: i64,
    pub c_text: i64,
    pub c_titel: i64,
    pub c_achse: i64,
    pub c_zeiger: i64,
    pub c_flaeche: i64,
    /// Zweite Farbe fuer den Hintergrund-Verlauf (Flag "verlauf").
    pub c_verlauf: i64,
    /// Farbe des Schlagschattens -- mit Alpha (RGBA), sonst wird er zur Wand.
    pub c_schatten: i64,
    /// Zielfarbe des Daten-Verlaufs (Flag "verlauf_daten"). -1 = automatisch
    /// eine abgedunkelte Fassung der jeweiligen Reihenfarbe.
    pub c_verlauf_ende: i64,
    /// Farbverlauf der SKALA (Tacho/Leiste/Lampen) von unten nach oben.
    ///
    /// Das ist bewusst NICHT die Palette: die ist kategorial (acht gut
    /// unterscheidbare Farben fuer acht Reihen) und ergibt interpoliert einen
    /// Regenbogen. Eine Skala braucht einen gerichteten Verlauf, dem man
    /// ansieht, wo "wenig" und wo "viel" ist.
    pub c_skala_von: i64,
    pub c_skala_mitte: i64,
    pub c_skala_bis: i64,

    // --- Schalter (CHART_SET_FLAG) ---
    pub f_rahmen: bool,
    pub f_gitter_x: bool,
    pub f_gitter_y: bool,
    pub f_prozent: bool,
    pub f_flaeche: bool,
    pub f_punkte: bool,
    pub f_glatt: bool,
    pub f_null_linie: bool,
    pub f_stapel: bool,
    pub f_verlauf: bool,
    /// Balken/Segmente/Flaechen als Farbverlauf statt in Vollfarbe.
    pub f_verlauf_daten: bool,
    /// Auch die Daten (Balken, Segmente, Zeiger) werfen Schatten -- nicht nur
    /// das Feld.
    pub f_schatten_daten: bool,
    /// Grosse Zahlen kurz schreiben (1.2M statt 1200000) -- wie NUMFMT$.
    pub f_kurz: bool,
    /// Element unter der Maus hervorheben (aufhellen, herausruecken, wachsen).
    pub f_hover: bool,
    /// Sprechblase mit Name und Wert am Element unter der Maus.
    pub f_tooltip: bool,
    /// Sekunden, die die Hervorhebung zum Ein-/Ausblenden braucht.
    pub hover_tempo: f64,
    /// Wie weit das hervorgehobene Element herausrueckt/waechst (Pixel).
    pub hover_weite: f64,
    /// Wie stark es aufhellt (0 = gar nicht, 1 = deutlich).
    pub hover_glanz: f64,
    /// Anzahl Segmente/Striche beim Zifferblatt "segmente"/"striche".
    pub blatt_teile: i32,
    /// Luecke zwischen den Segmenten in Grad.
    pub blatt_luecke: f64,
    /// Dicke des Skalenbogens als Anteil des Radius (0.05..0.6).
    pub blatt_dicke: f64,
    /// Metallische Fassung um die Scheibe (Breite in Pixeln, 0 = keine).
    pub fassung: i32,

    /// Reihenfolge der Standardfarben fuer Reihen/Segmente ohne eigene Farbe.
    pub palette: Vec<i64>,
}

impl Default for Style {
    fn default() -> Self {
        let mut s = Style {
            titel: String::new(),
            einheit: String::new(),
            achse_x: String::new(),
            achse_y: String::new(),
            legende: "aus".into(),
            werte: "aus".into(),
            ausrichtung: "senkrecht".into(),
            zeigerform: "nadel".into(),
            zifferblatt: "ring".into(),
            wertanzeige: "innen".into(),
            min: f64::NAN,
            max: f64::NAN,
            innenradius: 0.0,
            abstand: 0.0,
            ecken: 6,
            rahmen_dicke: 1,
            polster: 12,
            gitter: 0.0,
            nachkomma: 0,
            titel_groesse: 20,
            text_groesse: 14,
            schrift: -1,
            start_winkel: 135.0,
            end_winkel: 405.0,
            striche: 9,
            unterstriche: 4,
            linien_dicke: 2.0,
            punkt_radius: 3,
            animation: 0.0,
            fenster: 0,
            schatten: 0,
            schatten_weich: 0,
            deckkraft: 1.0,
            flaeche_deckkraft: 0.35,
            c_hintergrund: 0,
            c_rahmen: 0,
            c_gitter: 0,
            c_text: 0,
            c_titel: 0,
            c_achse: 0,
            c_zeiger: 0,
            c_flaeche: 0,
            c_verlauf: 0,
            c_schatten: 0,
            c_verlauf_ende: -1,
            c_skala_von: 0,
            c_skala_mitte: 0,
            c_skala_bis: 0,
            f_rahmen: true,
            f_gitter_x: false,
            f_gitter_y: true,
            f_prozent: false,
            f_flaeche: false,
            f_punkte: false,
            f_glatt: false,
            f_null_linie: true,
            f_stapel: false,
            f_verlauf: false,
            f_verlauf_daten: false,
            f_schatten_daten: false,
            f_kurz: false,
            f_hover: true,
            f_tooltip: false,
            hover_tempo: 0.15,
            hover_weite: 8.0,
            hover_glanz: 0.35,
            blatt_teile: 32,
            blatt_luecke: 2.0,
            blatt_dicke: 0.2,
            fassung: 0,
            palette: Vec::new(),
        };
        apply_theme(&mut s, "dunkel");
        s
    }
}

/// Farbwerte eines Themas -- ein Ort fuer alles Farbige, damit ein Thema nie
/// die Haelfte der Rollen ungesetzt laesst.
pub fn apply_theme(s: &mut Style, name: &str) -> bool {
    // (hintergrund, rahmen, gitter, text, titel, achse, zeiger, flaeche,
    //  verlauf, schatten -- letzterer mit Alpha im obersten Byte)
    #[allow(clippy::type_complexity)]
    let (bg, ra, gi, te, ti, ac, ze, fl, ve, sh, pal): (
        i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, &[i64],
    ) = match name.to_lowercase().as_str() {
        "dunkel" => (
            0x1E1E28, 0x3C3C50, 0x32323C, 0xC8C8D2, 0xFFFFFF, 0x646478, 0xFF3C50, 0x28465A,
            0x0A0A14, 0x8C000000,
            &[0x4FC3F7, 0xFFB74D, 0x81C784, 0xE57373, 0xBA68C8, 0x4DD0E1, 0xFFF176, 0xA1887F],
        ),
        "hell" => (
            0xFAFAFA, 0xC8C8C8, 0xE1E1E1, 0x323232, 0x141414, 0x969696, 0xC81E28, 0xC8DCF0,
            0xFFFFFF, 0x50283C50,
            &[0x1976D2, 0xF57C00, 0x388E3C, 0xD32F2F, 0x7B1FA2, 0x0097A7, 0xFBC02D, 0x5D4037],
        ),
        "neon" => (
            0x0A0014, 0x5A00A0, 0x1E0A3C, 0x00FFC8, 0x00FFC8, 0x7828C8, 0xFF00A0, 0x1E0050,
            0x000000, 0x78FF00A0,
            &[0x00FFC8, 0xFF00A0, 0x00C8FF, 0xFFF000, 0xA000FF, 0x00FF50, 0xFF6400, 0xFF00FF],
        ),
        "pastell" => (
            0xFFF8F0, 0xE0D5C8, 0xEFE6DC, 0x5A5048, 0x3C3630, 0xB4A898, 0xE07070, 0xF0E0D8,
            0xFFFFFF, 0x46B4A090,
            &[0x8FBCD4, 0xF0B49E, 0xA8CBA0, 0xE0A0A8, 0xC0A8D8, 0x90CFC4, 0xE8D08C, 0xB8A898],
        ),
        _ => return false,
    };
    s.c_hintergrund = bg;
    s.c_rahmen = ra;
    s.c_gitter = gi;
    s.c_text = te;
    s.c_titel = ti;
    s.c_achse = ac;
    s.c_zeiger = ze;
    s.c_flaeche = fl;
    s.c_verlauf = ve;
    s.c_schatten = sh;
    s.palette = pal.to_vec();
    // Skalenverlauf: rot -> gelb -> gruen, im jeweiligen Ton des Themas.
    let (sv, sm, sb) = match name.to_lowercase().as_str() {
        "hell" => (0xE53935, 0xFDD835, 0x43A047),
        "neon" => (0xFF0064, 0xFFF000, 0x00FFC8),
        "pastell" => (0xE0A0A8, 0xE8D08C, 0xA8CBA0),
        _ => (0xE5393C, 0xFFC400, 0x66BB6A),
    };
    s.c_skala_von = sv;
    s.c_skala_mitte = sm;
    s.c_skala_bis = sb;
    true
}

// --- Farb-Hilfen -----------------------------------------------------------
//
// GB-Farben sind 0xAARRGGBB; ein Alpha von 0 gilt in gbrt als DECKEND (damit
// alte 24-Bit-Farben unveraendert bleiben). Beim Rechnen muss man das also
// zuerst auf 255 heben, sonst wird aus "deckend" versehentlich "unsichtbar".

/// Alpha-Anteil einer GB-Farbe als 0..255 (0 im obersten Byte = deckend).
fn alpha_of(c: i64) -> u32 {
    let a = ((c as u32) >> 24) & 0xFF;
    if a == 0 {
        255
    } else {
        a
    }
}

/// Deckkraft mit `f` (0..1) multiplizieren. `f >= 1` laesst die Farbe in Ruhe,
/// damit ein unveraenderter Regler nichts umschreibt.
pub fn with_alpha(c: i64, f: f64) -> i64 {
    if f >= 1.0 {
        return c;
    }
    let a = (alpha_of(c) as f64 * f.clamp(0.0, 1.0)).round() as i64;
    // 0 waere "deckend" -- fuer echtes Unsichtbar bleibt 1 die untere Grenze.
    let a = a.clamp(1, 255);
    (a << 24) | (c & 0x00FF_FFFF)
}

/// Zwei Farben mischen (`t` = 0 ganz `a`, 1 ganz `b`). Das Alpha von `a`
/// bleibt erhalten -- gemischt wird nur die Farbe, nicht die Durchsichtigkeit.
pub fn mix_rgb(a: i64, b: i64, t: f64) -> i64 {
    let t = t.clamp(0.0, 1.0);
    let (va, vb) = (a as u32, b as u32);
    let ch = |sh: u32| -> i64 {
        let (ca, cb) = (((va >> sh) & 0xFF) as f64, ((vb >> sh) & 0xFF) as f64);
        ((ca + (cb - ca) * t).round() as i64).clamp(0, 255)
    };
    (((va >> 24) & 0xFF) as i64) << 24 | (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

/// Helligkeit skalieren (`f` < 1 dunkelt ab, > 1 hellt auf), Alpha bleibt.
pub fn scale_rgb(c: i64, f: f64) -> i64 {
    let v = c as u32;
    let ch = |sh: u32| -> i64 {
        ((((v >> sh) & 0xFF) as f64 * f).round() as i64).clamp(0, 255)
    };
    let a = ((v >> 24) & 0xFF) as i64;
    (a << 24) | (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

pub struct ChartObj {
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// Kategorien (Balken/Linie: Achsenbeschriftung; Kuchen: Segmentnamen).
    pub labels: Vec<String>,
    /// Farbe je Kategorie beim Kuchen; -1 = aus der Palette.
    pub label_colors: Vec<i64>,
    pub series: Vec<Series>,
    pub zones: Vec<Zone>,
    pub style: Style,

    // --- Interaktion (von CHART_DRAW gefuellt, ueber CHART_HOVER/... lesbar) ---
    /// Punkt unter der Maus, -1 = keiner.
    pub hover: i32,
    /// Reihe unter der Maus, -1 = keine (nur Balken/Linie unterscheiden Reihen).
    pub hover_serie: i32,
    /// In DIESEM Bild angeklickter Punkt, -1 = keiner.
    pub geklickt: i32,
    pub geklickt_serie: i32,
    /// Hervorhebungsstaerke je Punkt (0..1) -- zieht weich nach, damit das
    /// Ein- und Ausblenden nicht springt. Laenge folgt der Punktzahl.
    pub glanz: Vec<f64>,
}

impl ChartObj {
    pub fn new(kind: Kind, x: i32, y: i32, w: i32, h: i32) -> ChartObj {
        let mut c = ChartObj {
            kind,
            x,
            y,
            w: w.max(1),
            h: h.max(1),
            labels: Vec::new(),
            label_colors: Vec::new(),
            series: Vec::new(),
            zones: Vec::new(),
            style: Style::default(),
            hover: -1,
            hover_serie: -1,
            geklickt: -1,
            geklickt_serie: -1,
            glanz: Vec::new(),
        };
        // Der Tacho hat genau EINEN Wert -- die Reihe gleich anlegen, damit
        // CHART_VALUE ohne vorheriges CHART_SERIES funktioniert.
        if matches!(kind, Kind::Leiste | Kind::Led) {
            // `ausrichtung` steht per Vorgabe auf "senkrecht" -- richtig fuer
            // Balkendiagramme (Balken stehen), aber eine Leiste liegt. Ohne
            // das baute sie ungefragt hochkant.
            c.style.ausrichtung = "waagerecht".into();
            // Eine Leiste braucht keine Teilstriche, solange man sie nicht
            // bestellt; der Tacho dagegen lebt von seiner Skala.
            c.style.striche = 0;
        }
        if matches!(kind, Kind::Gauge | Kind::Leiste | Kind::Led) {
            c.add_series("", -1);
            c.series[0].values.push(0.0);
            c.series[0].shown.push(0.0);
            c.style.max = 100.0;
            c.style.min = 0.0;
        }
        c
    }

    pub fn add_series(&mut self, name: &str, color: i64) -> i64 {
        let n = self.labels.len();
        self.series.push(Series {
            name: name.to_string(),
            color,
            values: vec![0.0; n],
            shown: vec![0.0; n],
        });
        (self.series.len() - 1) as i64
    }

    /// Kategorie anlegen (Kuchen: ein Segment) und ihren Wert in Reihe 0 setzen.
    pub fn add_point(&mut self, label: &str, value: f64, color: i64) -> i64 {
        // Die Reihe MUSS vor der Kategorie entstehen: `add_series` fuellt die
        // neue Reihe mit einem Nullwert je bereits vorhandener Kategorie auf.
        // Andersherum zaehlte die gerade erst angelegte Kategorie schon mit,
        // die Reihe begann mit einem Leerwert -- und danach lagen ALLE Werte
        // um eine Stelle neben ihrer Beschriftung.
        if self.series.is_empty() {
            self.add_series("", -1);
        }
        self.labels.push(label.to_string());
        self.label_colors.push(color);
        for (i, s) in self.series.iter_mut().enumerate() {
            // Alle Reihen auf gleiche Laenge halten -- nur Reihe 0 bekommt den
            // uebergebenen Wert, die uebrigen eine 0 als Platzhalter.
            s.values.push(if i == 0 { value } else { 0.0 });
            s.shown.push(if i == 0 { value } else { 0.0 });
        }
        (self.labels.len() - 1) as i64
    }

    pub fn set_point(&mut self, serie: usize, idx: usize, value: f64) -> Result<(), String> {
        let s = self
            .series
            .get_mut(serie)
            .ok_or_else(|| format!("CHART_SET_POINT: Reihe {} gibt es nicht", serie))?;
        let slot = s
            .values
            .get_mut(idx)
            .ok_or_else(|| format!("CHART_SET_POINT: Punkt {} gibt es nicht", idx))?;
        *slot = value;
        Ok(())
    }

    /// Werte einer Reihe komplett ersetzen. Kuerzere/laengere Listen passen die
    /// Kategorienzahl an, damit Achse und Daten nicht auseinanderlaufen.
    pub fn set_data(&mut self, serie: usize, values: &[f64]) -> Result<(), String> {
        if serie >= self.series.len() {
            return Err(format!("CHART_DATA: Reihe {} gibt es nicht", serie));
        }
        while self.labels.len() < values.len() {
            self.labels.push(String::new());
            self.label_colors.push(-1);
        }
        self.series[serie].values = values.to_vec();
        self.series[serie].shown = values.to_vec();
        let n = self.labels.len();
        for s in &mut self.series {
            s.values.resize(n, 0.0);
            s.shown.resize(n, 0.0);
        }
        Ok(())
    }

    /// Wert hinten anhaengen (Live-Kurven). Mit `fenster > 0` faellt vorne der
    /// aelteste Wert heraus.
    pub fn push(&mut self, serie: usize, value: f64) -> Result<(), String> {
        if serie >= self.series.len() {
            return Err(format!("CHART_PUSH: Reihe {} gibt es nicht", serie));
        }
        self.series[serie].values.push(value);
        self.series[serie].shown.push(value);
        let n = self.series[serie].values.len();
        while self.labels.len() < n {
            self.labels.push(String::new());
            self.label_colors.push(-1);
        }
        let fenster = self.style.fenster;
        if fenster > 0 {
            let f = fenster as usize;
            for s in &mut self.series {
                if s.values.len() > f {
                    s.values.drain(..s.values.len() - f);
                }
                if s.shown.len() > f {
                    s.shown.drain(..s.shown.len() - f);
                }
            }
            if self.labels.len() > f {
                let d = self.labels.len() - f;
                self.labels.drain(..d);
                self.label_colors.drain(..d);
            }
        } else {
            let m = self.labels.len();
            for s in &mut self.series {
                s.values.resize(m, 0.0);
                s.shown.resize(m, 0.0);
            }
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.labels.clear();
        self.label_colors.clear();
        for s in &mut self.series {
            s.values.clear();
            s.shown.clear();
        }
        if matches!(self.kind, Kind::Gauge | Kind::Leiste | Kind::Led) {
            for s in &mut self.series {
                s.values.push(0.0);
                s.shown.push(0.0);
            }
        }
    }

    /// Angezeigte Werte an die echten heranfuehren (`animation` = Sekunden bis
    /// die Luecke praktisch geschlossen ist). Exponentielle Annaeherung, damit
    /// die Bewegung von der Bildrate unabhaengig bleibt.
    pub fn advance(&mut self, dt: f64) {
        let a = self.style.animation;
        for s in &mut self.series {
            s.shown.resize(s.values.len(), 0.0);
            if a <= 0.0 {
                s.shown.copy_from_slice(&s.values);
                continue;
            }
            let k = 1.0 - (-dt / (a / 3.0).max(1e-6)).exp();
            for (shown, &ziel) in s.shown.iter_mut().zip(s.values.iter()) {
                *shown += (ziel - *shown) * k.clamp(0.0, 1.0);
                if (ziel - *shown).abs() < 1e-9 {
                    *shown = ziel;
                }
            }
        }
    }

    /// Die Werte, die tatsaechlich gezeichnet werden.
    ///
    /// MIT Animation sind das die nachziehenden `shown` (die `CHART_UPDATE`
    /// fortschreibt), OHNE Animation direkt die echten `values`. Sonst waere
    /// `CHART_UPDATE` auch dann Pflicht, wenn gar keine Animation bestellt
    /// wurde -- und wer es vergisst, saehe ein Diagramm, das ewig auf Null
    /// steht, ohne jede Fehlermeldung.
    pub fn anzeige<'s>(&self, s: &'s Series) -> &'s [f64] {
        if self.style.animation > 0.0 {
            &s.shown
        } else {
            &s.values
        }
    }

    pub fn color_of(&self, serie: usize) -> i64 {
        let c = self.series.get(serie).map(|s| s.color).unwrap_or(-1);
        if c >= 0 {
            return c;
        }
        palette_at(&self.style.palette, serie)
    }

    fn slice_color(&self, idx: usize) -> i64 {
        match self.label_colors.get(idx) {
            Some(&c) if c >= 0 => c,
            _ => palette_at(&self.style.palette, idx),
        }
    }

    /// Wertebereich der Achse: feste Vorgaben schlagen die Daten; wo nichts
    /// vorgegeben ist, wird aus den Daten abgeleitet (die Null immer
    /// eingeschlossen, sonst luegen Balkenlaengen).
    fn range(&self) -> (f64, f64) {
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        if self.style.f_stapel && self.kind == Kind::Bar {
            for i in 0..self.labels.len() {
                let mut pos = 0.0;
                let mut neg = 0.0;
                for s in &self.series {
                    let v = *self.anzeige(s).get(i).unwrap_or(&0.0);
                    if v >= 0.0 {
                        pos += v;
                    } else {
                        neg += v;
                    }
                }
                lo = lo.min(neg);
                hi = hi.max(pos);
            }
        } else {
            for s in &self.series {
                for &v in self.anzeige(s) {
                    if v.is_finite() {
                        lo = lo.min(v);
                        hi = hi.max(v);
                    }
                }
            }
        }
        if !lo.is_finite() || !hi.is_finite() {
            lo = 0.0;
            hi = 1.0;
        }
        lo = lo.min(0.0);
        hi = hi.max(0.0);
        if self.style.min.is_finite() {
            lo = self.style.min;
        }
        if self.style.max.is_finite() {
            hi = self.style.max;
        }
        if (hi - lo).abs() < 1e-12 {
            hi = lo + 1.0;
        }
        (lo, hi)
    }

    /// (Startwinkel, Spanne) je Segment. Leere Segmente (Wert <= 0) bekommen
    /// eine Spanne von 0 und behalten trotzdem ihren Platz in der Liste,
    /// damit der Index zur Beschriftung passt.
    pub fn kuchen_stuecke(&self) -> Vec<(f64, f64)> {
        let werte: Vec<f64> = self
            .series
            .first()
            .map(|s| self.anzeige(s).iter().map(|v| v.max(0.0)).collect())
            .unwrap_or_default();
        let summe: f64 = werte.iter().sum();
        let mut out = Vec::with_capacity(werte.len());
        let mut w = -90.0f64; // bei 12 Uhr beginnen
        for &v in &werte {
            let spanne = if summe > 0.0 { v / summe * 360.0 } else { 0.0 };
            out.push((w, spanne));
            w += spanne;
        }
        out
    }

    /// Beschriftung des Elements unter der Maus ("" wenn keins).
    pub fn hover_label(&self) -> String {
        if self.hover < 0 {
            return String::new();
        }
        self.labels.get(self.hover as usize).cloned().unwrap_or_default()
    }

    /// Wert des Elements unter der Maus (0 wenn keins).
    pub fn hover_value(&self) -> f64 {
        if self.hover < 0 {
            return 0.0;
        }
        let si = self.hover_serie.max(0) as usize;
        self.series
            .get(si)
            .and_then(|s| s.values.get(self.hover as usize))
            .copied()
            .unwrap_or(0.0)
    }

    /// Zahl so beschriften, wie der Stil es vorgibt (Nachkommastellen, kurze
    /// Schreibweise, Einheit).
    pub fn fmt_value(&self, v: f64) -> String {
        let n = self.style.nachkomma.clamp(0, 9);
        let s = if self.style.f_kurz {
            crate::builtins::numfmt(v, n as i64)
        } else {
            format!("{:.*}", n as usize, v)
        };
        format!("{}{}", s, self.style.einheit)
    }
}

fn palette_at(pal: &[i64], i: usize) -> i64 {
    if pal.is_empty() {
        0x4FC3F7
    } else {
        pal[i % pal.len()]
    }
}

/// Welches Kuchenstueck liegt bei Winkel `w` (Grad)?  -1 = keins.
///
/// Die Segmente beginnen bei -90 Grad (12 Uhr), `atan2` liefert aber
/// -180..180 -- der Winkel muss also erst in den Bereich der Segmentliste
/// gedreht werden, sonst trifft die Maus links oben ins Leere.
pub fn stueck_bei_winkel(stuecke: &[(f64, f64)], w: f64) -> i32 {
    let mut w = w;
    while w < -90.0 {
        w += 360.0;
    }
    while w >= 270.0 {
        w -= 360.0;
    }
    for (i, (start, spanne)) in stuecke.iter().enumerate() {
        if *spanne > 0.0 && w >= *start && w < start + spanne {
            return i as i32;
        }
    }
    -1
}

/// Automatische Gitter-Schrittweite: die groesste "runde" Zahl (1/2/5 * 10^n),
/// die hoechstens `ziel` Linien ergibt.
fn nice_step(span: f64, ziel: i32) -> f64 {
    if !(span.is_finite() && span > 0.0) {
        return 1.0;
    }
    let roh = span / ziel.max(1) as f64;
    let exp = roh.log10().floor();
    let basis = 10f64.powf(exp);
    for m in [1.0, 2.0, 2.5, 5.0, 10.0] {
        if basis * m >= roh {
            return basis * m;
        }
    }
    basis * 10.0
}

// ---------------------------------------------------------------------------
// Zeichnen (nur mit Grafik-Feature -- das Datenmodell oben laeuft auch
// konsolen-only, damit ein Programm ein Diagramm aufbauen kann, ohne dass
// gleich ein Fenster noetig waere).
// ---------------------------------------------------------------------------

#[cfg(feature = "graphics")]
struct Area {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

#[cfg(feature = "graphics")]
impl Area {
    fn w(&self) -> i32 {
        self.x1 - self.x0
    }
    fn h(&self) -> i32 {
        self.y1 - self.y0
    }
}

#[cfg(feature = "graphics")]
impl ChartObj {
    /// Schlagschatten unter ein Rechteck legen. `schatten` = Versatz,
    /// `schatten_weich` = Anzahl gestaffelter Kopien; jede traegt nur einen
    /// Bruchteil der Deckkraft, aufsummiert ergibt das den weichen Rand.
    /// (raylib hat keinen Weichzeichner fuer Formen -- gestaffelte Kopien sind
    /// die Naeherung, die ohne Render-Target auskommt.)
    fn schatten_rrect(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, r: i32) {
        let st = &self.style;
        if st.schatten <= 0 {
            return;
        }
        let o = st.schatten;
        let lagen = st.schatten_weich.max(0);
        if lagen == 0 {
            g.round_rect(x1 + o, y1 + o, x2 + o, y2 + o, r, st.c_schatten, true);
            return;
        }
        let anteil = 1.0 / (lagen + 1) as f64;
        let c = with_alpha(st.c_schatten, anteil);
        for k in (0..=lagen).rev() {
            g.round_rect(x1 + o - k, y1 + o - k, x2 + o + k, y2 + o + k, r + k, c, true);
        }
    }

    /// Datenfarbe einer Reihe inklusive des globalen Deckkraft-Reglers.
    fn data_color(&self, serie: usize) -> i64 {
        with_alpha(self.color_of(serie), self.style.deckkraft)
    }

    /// Zielfarbe eines Daten-Verlaufs: entweder fest gesetzt oder eine
    /// abgedunkelte Fassung der Ausgangsfarbe (Alpha bleibt erhalten).
    fn verlauf_ende(&self, von: i64) -> i64 {
        if self.style.c_verlauf_ende >= 0 {
            with_alpha(self.style.c_verlauf_ende, self.style.deckkraft)
        } else {
            scale_rgb(von, 0.45)
        }
    }

    /// Gefuellte Flaeche mit runden Ecken -- mit Flag "verlauf_daten" als
    /// senkrechter Verlauf. Der Verlauf sitzt um die Eckenrundung eingerueckt
    /// (GradientRect ist rechteckig), der Rand bleibt in der Ausgangsfarbe.
    #[allow(clippy::too_many_arguments)]
    fn fuellung(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, ecken: i32, farbe: i64) {
        g.round_rect(x1, y1, x2, y2, ecken, farbe, true);
        if self.style.f_verlauf_daten {
            let e = ecken.max(0);
            if x2 - x1 > 2 * e && y2 - y1 > 2 * e {
                g.gradient_rect(x1 + e, y1 + e, x2 - e, y2 - e, farbe, self.verlauf_ende(farbe), true);
            }
        }
    }

    // --- Geometrie -------------------------------------------------------
    //
    // EINE Quelle je Diagrammart. Treffertest und Zeichnen fragen dieselbe
    // Funktion; sonst laufen sie auseinander und die Maus trifft neben dem,
    // was man sieht (derselbe Fehler wie frueher beim Tacho-Schatten).

    /// Zeichenflaeche nach Abzug von Polster, Titel und Legende.
    fn inhalt(&self, g: &Graphics) -> Area {
        let st = &self.style;
        let mut a = Area {
            x0: self.x + st.polster,
            y0: self.y + st.polster,
            x1: self.x + self.w - st.polster,
            y1: self.y + self.h - st.polster,
        };
        if !st.titel.is_empty() {
            a.y0 += st.titel_groesse + 8;
        }
        self.legende_abzug(g, &mut a);
        a
    }

    /// Mittelpunkt und Radien des Kuchens.
    fn kuchen_geom(&self, a: &Area) -> (i32, i32, i32, i32) {
        let st = &self.style;
        let (cx, cy) = (a.x0 + a.w() / 2, a.y0 + a.h() / 2);
        // Platz fuer das Herausruecken freihalten -- beim Hover kommt
        // `hover_weite` noch obendrauf, sonst stiesse das Segment an den Rand.
        let rand = st.abstand.max(0.0) + if st.f_hover { st.hover_weite } else { 0.0 };
        let r = (a.w().min(a.h()) / 2 - rand as i32).max(4);
        let ri = (r as f64 * st.innenradius.clamp(0.0, 0.95)) as i32;
        (cx, cy, r, ri)
    }

    /// Alles, was Balken- und Liniendiagramm zum Platzieren brauchen.
    fn achsen_geom(&self, g: &Graphics, a: &Area, waagerecht: bool) -> (Area, f64, f64) {
        let (lo, hi) = self.range();
        (self.axis_area(g, a, lo, hi, waagerecht), lo, hi)
    }

    /// Balken-Aufteilung: Fachbreite, Luecke, Balkenbreite.
    fn balken_geom(&self, plot: &Area, waagerecht: bool) -> (f64, f64, f64) {
        let st = &self.style;
        let n = self.labels.len().max(1);
        let laenge = if waagerecht { plot.h() } else { plot.w() };
        let fach = laenge as f64 / n as f64;
        let luecke = st.abstand.max(0.0).min(fach / 2.0);
        let reihen = if st.f_stapel { 1 } else { self.series.len().max(1) };
        let breite = ((fach - luecke) / reihen as f64).max(1.0);
        (fach, luecke, breite)
    }

    // --- Treffertest -----------------------------------------------------

    /// Punkt (und ggf. Reihe) unter der Maus. -1 = nichts getroffen.
    fn treffer(&self, g: &Graphics, a: &Area) -> (i32, i32) {
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        // Ausserhalb des Feldes gar nicht erst suchen.
        if mx < self.x || mx > self.x + self.w || my < self.y || my > self.y + self.h {
            return (-1, -1);
        }
        match self.kind {
            Kind::Pie => {
                let (cx, cy, r, ri) = self.kuchen_geom(a);
                let (dx, dy) = ((mx - cx) as f64, (my - cy) as f64);
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > r as f64 || dist < ri as f64 {
                    return (-1, -1);
                }
                let w = dy.atan2(dx).to_degrees();
                match stueck_bei_winkel(&self.kuchen_stuecke(), w) {
                    -1 => (-1, -1),
                    i => (i, 0),
                }
            }
            Kind::Bar => {
                let waagerecht = self.style.ausrichtung.eq_ignore_ascii_case("waagerecht");
                let (plot, lo, hi) = self.achsen_geom(g, a, waagerecht);
                let (fach, luecke, breite) = self.balken_geom(&plot, waagerecht);
                let quer = if waagerecht { my - plot.y0 } else { mx - plot.x0 };
                if quer < 0 {
                    return (-1, -1);
                }
                let i = (quer as f64 / fach).floor() as usize;
                if i >= self.labels.len() {
                    return (-1, -1);
                }
                // Innerhalb des Fachs: welcher Balken (bei mehreren Reihen)?
                let rest = quer as f64 - fach * i as f64 - luecke / 2.0;
                if self.style.f_stapel {
                    // Gestapelt: ein Balken, die Reihe ergibt sich aus der
                    // Position entlang der Wertachse.
                    if rest < 0.0 || rest > breite {
                        return (-1, -1);
                    }
                    let mut unten = 0.0f64;
                    for (si, s) in self.series.iter().enumerate() {
                        let v = *self.anzeige(s).get(i).unwrap_or(&0.0);
                        let p1 = self.val_pos(&plot, unten, lo, hi, waagerecht);
                        let p2 = self.val_pos(&plot, unten + v, lo, hi, waagerecht);
                        let laengs = if waagerecht { mx } else { my };
                        if laengs >= p1.min(p2) && laengs <= p1.max(p2) {
                            return (i as i32, si as i32);
                        }
                        unten += v;
                    }
                    (-1, -1)
                } else {
                    let si = (rest / breite).floor() as i32;
                    if si < 0 || si as usize >= self.series.len() {
                        return (-1, -1);
                    }
                    // Nur treffen, wo der Balken auch wirklich ist.
                    let v = *self
                        .anzeige(&self.series[si as usize])
                        .get(i)
                        .unwrap_or(&0.0);
                    let p1 = self.val_pos(&plot, 0.0, lo, hi, waagerecht);
                    let p2 = self.val_pos(&plot, v, lo, hi, waagerecht);
                    let laengs = if waagerecht { mx } else { my };
                    if laengs >= p1.min(p2) && laengs <= p1.max(p2) {
                        (i as i32, si)
                    } else {
                        (-1, -1)
                    }
                }
            }
            Kind::Line => {
                let n = self.labels.len();
                if n < 2 {
                    return (-1, -1);
                }
                let (plot, lo, hi) = self.achsen_geom(g, a, false);
                if mx < plot.x0 || mx > plot.x1 {
                    return (-1, -1);
                }
                let dx = plot.w() as f64 / (n - 1) as f64;
                // Naechstliegender Stuetzpunkt in x -- eine Linie trifft man
                // nicht auf den Pixel genau, also faengt die ganze Spalte.
                let i = (((mx - plot.x0) as f64 / dx).round() as usize).min(n - 1);
                // Von den Reihen die, deren Punkt senkrecht am naechsten liegt.
                let mut beste = (-1i32, i32::MAX);
                for (si, s) in self.series.iter().enumerate() {
                    let v = *self.anzeige(s).get(i).unwrap_or(&0.0);
                    let py = self.val_pos(&plot, v, lo, hi, false);
                    let d = (py - my).abs();
                    if d < beste.1 {
                        beste = (si as i32, d);
                    }
                }
                if beste.0 >= 0 {
                    (i as i32, beste.0)
                } else {
                    (-1, -1)
                }
            }
            // Die Leiste hat einen Wert -- das ganze Feld ist das Element.
            Kind::Leiste => (0, 0),
            // Bei den Lampen ist jede Zelle einzeln ansprechbar.
            Kind::Led => {
                let (zellen, _, waagerecht) = self.led_geom();
                let (pos, len) = if waagerecht {
                    (mx - a.x0, a.w())
                } else {
                    (a.y1 - my, a.h())
                };
                if pos < 0 || pos >= len || len <= 0 {
                    return (-1, -1);
                }
                let i = (pos as f64 / (len as f64 / zellen as f64)).floor() as i32;
                (i.clamp(0, zellen - 1), 0)
            }
            // Der Tacho hat nur einen Wert -- die ganze Scheibe ist das Element.
            Kind::Gauge => {
                let (cx, cy) = (a.x0 + a.w() / 2, a.y0 + a.h() / 2);
                let r = (a.w().min(a.h()) / 2 - 2).max(6);
                let (dx, dy) = ((mx - cx) as f64, (my - cy) as f64);
                if (dx * dx + dy * dy).sqrt() <= r as f64 {
                    (0, 0)
                } else {
                    (-1, -1)
                }
            }
        }
    }

    /// Hervorhebung an das Ziel heranfuehren (weiches Ein-/Ausblenden).
    fn glanz_fortschreiben(&mut self, dt: f64) {
        let n = self.labels.len().max(1);
        self.glanz.resize(n, 0.0);
        let tempo = self.style.hover_tempo;
        let hover = self.hover;
        for (i, g) in self.glanz.iter_mut().enumerate() {
            let ziel = if i as i32 == hover { 1.0 } else { 0.0 };
            if tempo <= 0.0 {
                *g = ziel;
                continue;
            }
            let k = (1.0 - (-dt / (tempo / 3.0).max(1e-6)).exp()).clamp(0.0, 1.0);
            *g += (ziel - *g) * k;
            if (ziel - *g).abs() < 1e-4 {
                *g = ziel;
            }
        }
    }

    /// Hervorhebungsstaerke eines Punktes (0 wenn Hover aus ist).
    fn glanz_von(&self, i: usize) -> f64 {
        if !self.style.f_hover {
            return 0.0;
        }
        self.glanz.get(i).copied().unwrap_or(0.0)
    }

    /// Farbe eines hervorgehobenen Elements.
    ///
    /// Gemischt wird gegen WEISS, nicht per RGB-Skalierung: bei gesaettigten
    /// Farben verschiebt das Skalieren den Farbton, weil der groesste Kanal
    /// schon bei 255 klemmt und nur die kleineren mitwachsen -- ein
    /// hervorgehobenes Orange wurde so sichtbar gelb und sah aus wie ein
    /// anderer Eintrag der Palette.
    fn hell(&self, farbe: i64, glanz: f64) -> i64 {
        if glanz <= 0.0 {
            return farbe;
        }
        mix_rgb(farbe, 0xFFFFFF, self.style.hover_glanz * glanz)
    }

    /// Text der Sprechblase: Reihe, Beschriftung und Wert -- je nachdem, was
    /// es gibt. Beim Kuchen zusaetzlich der Anteil, weil genau das dort die
    /// interessante Zahl ist.
    fn tooltip_text(&self) -> String {
        let wert = self.hover_value();
        let mut kopf = String::new();
        if self.kind != Kind::Pie {
            if let Some(s) = self.series.get(self.hover_serie.max(0) as usize) {
                if !s.name.is_empty() {
                    kopf = s.name.clone();
                }
            }
        }
        let label = self.hover_label();
        if !label.is_empty() {
            if kopf.is_empty() {
                kopf = label;
            } else {
                kopf = format!("{} - {}", kopf, label);
            }
        }
        let mut wert_txt = self.fmt_value(wert);
        if self.kind == Kind::Pie {
            let summe: f64 = self
                .series
                .first()
                .map(|s| self.anzeige(s).iter().map(|v| v.max(0.0)).sum())
                .unwrap_or(0.0);
            if summe > 0.0 {
                wert_txt = format!("{} ({:.0}%)", wert_txt, wert / summe * 100.0);
            }
        }
        if kopf.is_empty() {
            wert_txt
        } else {
            format!("{}: {}", kopf, wert_txt)
        }
    }

    /// Sprechblase an der Maus. Sie weicht nach innen aus, wenn sie sonst
    /// ueber den Rand des Feldes hinausstuende -- eine Blase, die halb im
    /// Nichts haengt, ist schlechter als eine leicht versetzte.
    fn tooltip(&self, g: &mut Graphics) {
        let st = &self.style;
        let text = self.tooltip_text();
        if text.is_empty() {
            return;
        }
        let sz = st.text_groesse;
        let tw = g.text_width_at(&text, sz);
        let (bw, bh) = (tw + 16, sz + 12);
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        let mut x = mx + 14;
        let mut y = my - bh - 8;
        if x + bw > self.x + self.w {
            x = mx - bw - 14;
        }
        if y < self.y {
            y = my + 16;
        }
        // Dunkler Kasten mit heller Schrift -- liest sich auf jedem Thema.
        g.round_rect(x + 2, y + 3, x + bw + 2, y + bh + 3, 5, 0x64000000, true);
        g.round_rect(x, y, x + bw, y + bh, 5, 0x1E1E1E, true);
        g.round_rect(x, y, x + bw, y + bh, 5, 0x646464, false);
        g.text_styled(x + 8, y + 6, text, 0xF0F0F0, st.schrift, sz);
    }

    pub fn draw(&mut self, g: &mut Graphics) {
        // Maus auswerten, BEVOR gezeichnet wird -- die Hervorhebung soll im
        // selben Bild wirken, in dem sie erkannt wurde.
        let flaeche = self.inhalt(g);
        let (h, hs) = if self.style.f_hover || self.style.f_tooltip {
            self.treffer(g, &flaeche)
        } else {
            (-1, -1)
        };
        self.hover = h;
        self.hover_serie = hs;
        if g.mouse_hit(0) {
            self.geklickt = h;
            self.geklickt_serie = hs;
        } else {
            self.geklickt = -1;
            self.geklickt_serie = -1;
        }
        self.glanz_fortschreiben(g.delta());
        self.zeichnen(g);
        if self.style.f_tooltip && self.hover >= 0 {
            self.tooltip(g);
        }
    }

    fn zeichnen(&self, g: &mut Graphics) {
        let st = &self.style;
        // Schatten zuerst, damit er unter allem liegt.
        self.schatten_rrect(g, self.x, self.y, self.x + self.w, self.y + self.h, st.ecken);
        if st.f_verlauf {
            // Ein Verlauf kann nicht rund sein (GradientRect ist rechteckig) --
            // das runde Feld darunter fuellt die Ecken, der Verlauf legt sich
            // leicht eingerueckt darueber.
            g.round_rect(self.x, self.y, self.x + self.w, self.y + self.h, st.ecken, st.c_hintergrund, true);
            let e = st.ecken.max(0);
            g.gradient_rect(
                self.x + e,
                self.y + e,
                self.x + self.w - e,
                self.y + self.h - e,
                st.c_hintergrund,
                st.c_verlauf,
                true,
            );
        } else {
            g.round_rect(self.x, self.y, self.x + self.w, self.y + self.h, st.ecken, st.c_hintergrund, true);
        }
        if st.f_rahmen {
            for i in 0..st.rahmen_dicke.max(1) {
                g.round_rect(
                    self.x + i,
                    self.y + i,
                    self.x + self.w - i,
                    self.y + self.h - i,
                    (st.ecken - i).max(0),
                    st.c_rahmen,
                    false,
                );
            }
        }

        if !st.titel.is_empty() {
            let tw = g.text_width_at(&st.titel, st.titel_groesse);
            g.text_styled(
                self.x + (self.w - tw) / 2,
                self.y + st.polster,
                st.titel.clone(),
                st.c_titel,
                st.schrift,
                st.titel_groesse,
            );
        }
        self.draw_legend(g);
        let a = self.inhalt(g);

        if a.w() < 8 || a.h() < 8 {
            return; // Zu klein zum Zeichnen -- lieber nichts als Muell.
        }
        match self.kind {
            Kind::Pie => self.draw_pie(g, &a),
            Kind::Bar => self.draw_bar(g, &a),
            Kind::Line => self.draw_line(g, &a),
            Kind::Gauge => self.draw_gauge(g, &a),
            Kind::Leiste => self.draw_leiste(g, &a),
            Kind::Led => self.draw_led(g, &a),
        }
    }

    /// Legendeneintraege (Name + Farbe). Kuchen beschriftet Segmente, alles
    /// andere die Reihen. Namenlose Eintraege fallen weg.
    fn legende_eintraege(&self) -> Vec<(String, i64)> {
        let roh: Vec<(String, i64)> = if self.kind == Kind::Pie {
            self.labels
                .iter()
                .enumerate()
                .map(|(i, l)| (l.clone(), self.slice_color(i)))
                .collect()
        } else {
            self.series
                .iter()
                .enumerate()
                .map(|(i, s)| (s.name.clone(), self.color_of(i)))
                .collect()
        };
        roh.into_iter().filter(|(n, _)| !n.is_empty()).collect()
    }

    /// Platz, den die Legende der Zeichenflaeche wegnimmt. Rein rechnend --
    /// `inhalt()` (Treffertest) und `draw_legend` (Zeichnen) teilen sie sich,
    /// damit die Maus nicht neben dem trifft, was zu sehen ist.
    fn legende_abzug(&self, g: &Graphics, a: &mut Area) {
        let st = &self.style;
        let pos = st.legende.to_lowercase();
        if pos == "aus" {
            return;
        }
        let eintraege = self.legende_eintraege();
        if eintraege.is_empty() {
            return;
        }
        let sz = st.text_groesse;
        let kasten = (sz * 3) / 4;
        let zeile = sz + 6;
        match pos.as_str() {
            "links" | "rechts" => {
                let breite = eintraege
                    .iter()
                    .map(|(n, _)| g.text_width_at(n, sz))
                    .max()
                    .unwrap_or(0)
                    + kasten
                    + 10;
                if pos == "links" {
                    a.x0 += breite + 6;
                } else {
                    a.x1 -= breite + 6;
                }
            }
            _ => {
                if pos == "oben" {
                    a.y0 += zeile + 6;
                } else {
                    a.y1 -= zeile + 6;
                }
            }
        }
    }

    /// Legende an einer der vier Kanten zeichnen.
    fn draw_legend(&self, g: &mut Graphics) {
        let st = &self.style;
        let pos = st.legende.to_lowercase();
        if pos == "aus" {
            return;
        }
        let eintraege = self.legende_eintraege();
        if eintraege.is_empty() {
            return;
        }
        // Flaeche VOR dem Legenden-Abzug -- dort wird sie hingezeichnet.
        let mut a = Area {
            x0: self.x + st.polster,
            y0: self.y + st.polster,
            x1: self.x + self.w - st.polster,
            y1: self.y + self.h - st.polster,
        };
        if !st.titel.is_empty() {
            a.y0 += st.titel_groesse + 8;
        }
        let a = &mut a;
        let sz = st.text_groesse;
        let kasten = (sz * 3) / 4;
        let zeile = sz + 6;
        match pos.as_str() {
            "links" | "rechts" => {
                let breite = eintraege
                    .iter()
                    .map(|(n, _)| g.text_width_at(n, sz))
                    .max()
                    .unwrap_or(0)
                    + kasten
                    + 10;
                let x = if pos == "links" { a.x0 } else { a.x1 - breite };
                let mut y = a.y0 + ((a.h() - eintraege.len() as i32 * zeile) / 2).max(0);
                for (name, c) in &eintraege {
                    g.round_rect(x, y + 2, x + kasten, y + 2 + kasten, 2, *c, true);
                    g.text_styled(x + kasten + 6, y, name.clone(), st.c_text, st.schrift, sz);
                    y += zeile;
                }
                if pos == "links" {
                    a.x0 += breite + 6;
                } else {
                    a.x1 -= breite + 6;
                }
            }
            _ => {
                let gesamt: i32 = eintraege
                    .iter()
                    .map(|(n, _)| g.text_width_at(n, sz) + kasten + 16)
                    .sum();
                let y = if pos == "oben" { a.y0 } else { a.y1 - zeile };
                let mut x = a.x0 + ((a.w() - gesamt) / 2).max(0);
                for (name, c) in &eintraege {
                    g.round_rect(x, y + 2, x + kasten, y + 2 + kasten, 2, *c, true);
                    g.text_styled(x + kasten + 6, y, name.clone(), st.c_text, st.schrift, sz);
                    x += g.text_width_at(name, sz) + kasten + 16;
                }
                if pos == "oben" {
                    a.y0 += zeile + 6;
                } else {
                    a.y1 -= zeile + 6;
                }
            }
        }
    }

    fn draw_pie(&self, g: &mut Graphics, a: &Area) {
        let st = &self.style;
        let werte: Vec<f64> = self
            .series
            .first()
            .map(|s| self.anzeige(s).iter().map(|v| v.max(0.0)).collect())
            .unwrap_or_default();
        let summe: f64 = werte.iter().sum();
        if summe <= 0.0 {
            return;
        }
        let (cx, cy, r, ri) = self.kuchen_geom(a);
        let stuecke = self.kuchen_stuecke();

        for (i, &v) in werte.iter().enumerate() {
            let (winkel, spanne) = stuecke[i];
            if spanne <= 0.0 {
                continue;
            }
            let mitte = (winkel + spanne / 2.0).to_radians();
            let glanz = self.glanz_von(i);
            // "abstand" schiebt das Segment aus der Mitte heraus (Tortenstueck
            // herausgezogen); die Hervorhebung legt beim Ueberfahren noch
            // `hover_weite` drauf -- daher der Rand in kuchen_geom().
            let raus = st.abstand + st.hover_weite * glanz;
            let vx = cx + (mitte.cos() * raus) as i32;
            let vy = cy + (mitte.sin() * raus) as i32;
            let ra = r + (st.hover_weite * 0.35 * glanz) as i32;
            let farbe = self.hell(with_alpha(self.slice_color(i), st.deckkraft), glanz);
            if st.f_schatten_daten && st.schatten > 0 {
                g.ring(vx + st.schatten, vy + st.schatten, ri, ra,
                       winkel, winkel + spanne, st.c_schatten, true);
            }
            g.ring(vx, vy, ri, ra, winkel, winkel + spanne, farbe, true);
            if st.f_verlauf_daten {
                // Ein echter Radialverlauf ist mit `ring` nicht zu haben --
                // ein abgedunkeltes Band auf der Innenhaelfte gibt dem
                // Segment aber dieselbe Tiefenwirkung.
                let band = ri + (ra - ri) / 2;
                g.ring(vx, vy, ri, band, winkel, winkel + spanne,
                       with_alpha(self.verlauf_ende(farbe), 0.55), true);
            }

            if st.werte != "aus" {
                let text = if st.f_prozent {
                    format!("{:.*}%", st.nachkomma.clamp(0, 9) as usize, v / summe * 100.0)
                } else {
                    self.fmt_value(v)
                };
                let (lr, farbe) = if st.werte == "innen" {
                    (((ra + ri) / 2) as f64, st.c_text)
                } else {
                    (ra as f64 + 14.0, st.c_text)
                };
                let tx = vx + (mitte.cos() * lr) as i32 - g.text_width_at(&text, st.text_groesse) / 2;
                let ty = vy + (mitte.sin() * lr) as i32 - st.text_groesse / 2;
                g.text_styled(tx, ty, text, farbe, st.schrift, st.text_groesse);
            }
        }
    }

    fn draw_bar(&self, g: &mut Graphics, a: &Area) {
        let st = &self.style;
        let n = self.labels.len();
        if n == 0 || self.series.is_empty() {
            return;
        }
        let waagerecht = st.ausrichtung.eq_ignore_ascii_case("waagerecht");
        let (lo, hi) = self.range();
        let plot = self.axis_area(g, a, lo, hi, waagerecht);
        if plot.w() < 4 || plot.h() < 4 {
            return;
        }
        self.draw_grid(g, &plot, lo, hi, waagerecht);

        let (fach, luecke, breite) = self.balken_geom(&plot, waagerecht);

        for i in 0..n {
            let basis = fach * i as f64 + luecke / 2.0;
            if st.f_stapel {
                let (mut stapel_pos, mut stapel_neg) = (0.0f64, 0.0f64);
                for (si, s) in self.series.iter().enumerate() {
                    let v = *self.anzeige(s).get(i).unwrap_or(&0.0);
                    let von = if v >= 0.0 { stapel_pos } else { stapel_neg };
                    let bis = von + v;
                    if v >= 0.0 {
                        stapel_pos = bis;
                    } else {
                        stapel_neg = bis;
                    }
                    let f = self.hell(self.data_color(si), self.glanz_von(i));
                    self.bar_rect(g, &plot, basis, breite, von, bis, lo, hi, f, waagerecht);
                }
            } else {
                for (si, s) in self.series.iter().enumerate() {
                    let v = *self.anzeige(s).get(i).unwrap_or(&0.0);
                    let off = basis + breite * si as f64;
                    let f = self.hell(self.data_color(si), self.glanz_von(i));
                    self.bar_rect(g, &plot, off, breite, 0.0, v, lo, hi, f, waagerecht);
                    // Werte nur bei EINER Reihe anschreiben -- bei mehreren
                    // stehen die Balken so dicht, dass die Zahlen ineinander
                    // laufen wuerden. Dort tut es die Achse.
                    if st.werte != "aus" && self.series.len() == 1 {
                        let t = self.fmt_value(v);
                        let tw = g.text_width_at(&t, st.text_groesse);
                        let (tx, ty) = if waagerecht {
                            (
                                self.val_pos(&plot, v, lo, hi, true) + 4,
                                plot.y0 + (off + breite / 2.0) as i32 - st.text_groesse / 2,
                            )
                        } else {
                            (
                                plot.x0 + (off + breite / 2.0) as i32 - tw / 2,
                                self.val_pos(&plot, v, lo, hi, false) - st.text_groesse - 3,
                            )
                        };
                        g.text_styled(tx, ty, t, st.c_text, st.schrift, st.text_groesse);
                    }
                }
            }
            // Kategoriebeschriftung
            if let Some(l) = self.labels.get(i) {
                if !l.is_empty() {
                    let mitte = basis + (fach - luecke) / 2.0;
                    if waagerecht {
                        let tw = g.text_width_at(l, st.text_groesse);
                        g.text_styled(
                            plot.x0 - tw - 6,
                            plot.y0 + mitte as i32 - st.text_groesse / 2,
                            l.clone(),
                            st.c_text,
                            st.schrift,
                            st.text_groesse,
                        );
                    } else {
                        let tw = g.text_width_at(l, st.text_groesse);
                        g.text_styled(
                            plot.x0 + mitte as i32 - tw / 2,
                            plot.y1 + 5,
                            l.clone(),
                            st.c_text,
                            st.schrift,
                            st.text_groesse,
                        );
                    }
                }
            }
        }
        self.draw_axis_titles(g, a, &plot);
    }

    #[allow(clippy::too_many_arguments)]
    fn bar_rect(
        &self,
        g: &mut Graphics,
        plot: &Area,
        off: f64,
        breite: f64,
        von: f64,
        bis: f64,
        lo: f64,
        hi: f64,
        farbe: i64,
        waagerecht: bool,
    ) {
        let st = &self.style;
        let p1 = self.val_pos(plot, von, lo, hi, waagerecht);
        let p2 = self.val_pos(plot, bis, lo, hi, waagerecht);
        let (a1, a2) = (p1.min(p2), p1.max(p2));
        if a2 - a1 < 1 {
            return; // Nullwert -- kein Ein-Pixel-Strich, der Daten vortaeuscht.
        }
        let e = st.ecken.min((breite / 2.0) as i32).max(0);
        let (bx1, by1, bx2, by2) = if waagerecht {
            (a1, plot.y0 + off as i32, a2, plot.y0 + (off + breite) as i32)
        } else {
            (plot.x0 + off as i32, a1, plot.x0 + (off + breite) as i32, a2)
        };
        if st.f_schatten_daten {
            self.schatten_rrect(g, bx1, by1, bx2, by2, e);
        }
        self.fuellung(g, bx1, by1, bx2, by2, e, farbe);
    }

    fn draw_line(&self, g: &mut Graphics, a: &Area) {
        let st = &self.style;
        let n = self.labels.len();
        if n < 2 || self.series.is_empty() {
            return;
        }
        let (plot, lo, hi) = self.achsen_geom(g, a, false);
        if plot.w() < 4 || plot.h() < 4 {
            return;
        }
        self.draw_grid(g, &plot, lo, hi, false);

        let dx = plot.w() as f64 / (n - 1) as f64;
        for (si, s) in self.series.iter().enumerate() {
            let farbe = self.data_color(si);
            let xs: Vec<i32> = (0..n).map(|i| plot.x0 + (dx * i as f64) as i32).collect();
            let ys: Vec<i32> = (0..n)
                .map(|i| self.val_pos(&plot, *self.anzeige(s).get(i).unwrap_or(&0.0), lo, hi, false))
                .collect();

            if st.f_flaeche {
                let basis = self.val_pos(&plot, 0.0f64.clamp(lo, hi), lo, hi, false);
                let oben = with_alpha(st.c_flaeche, st.flaeche_deckkraft);
                if st.f_verlauf_daten {
                    // Senkrechte Streifen zwischen Kurve und Grundlinie, jeder
                    // von der Flaechenfarbe ins Durchsichtige. GradientRect ist
                    // rechteckig -- schmale Streifen sind die Naeherung, die den
                    // Kurvenverlauf trotzdem nachzeichnet.
                    let unten = with_alpha(st.c_flaeche, st.flaeche_deckkraft * 0.06);
                    const STREIFEN: i32 = 2;
                    let mut x = xs[0];
                    while x < xs[n - 1] {
                        // Punkt links von x suchen und linear interpolieren.
                        let t = (x - plot.x0) as f64 / dx;
                        let i = (t.floor() as usize).min(n - 2);
                        let f = (t - i as f64).clamp(0.0, 1.0);
                        let y = ys[i] as f64 + (ys[i + 1] - ys[i]) as f64 * f;
                        let (y1, y2) = ((y as i32).min(basis), (y as i32).max(basis));
                        if y2 > y1 {
                            g.gradient_rect(x, y1, (x + STREIFEN).min(xs[n - 1]), y2, oben, unten, true);
                        }
                        x += STREIFEN;
                    }
                } else {
                    let mut flach: Vec<i32> = Vec::with_capacity(n * 2 + 4);
                    for i in 0..n {
                        flach.push(xs[i]);
                        flach.push(ys[i]);
                    }
                    flach.push(xs[n - 1]);
                    flach.push(basis);
                    flach.push(xs[0]);
                    flach.push(basis);
                    let _ = g.polygon(&flach, oben, true);
                }
            }
            if st.f_glatt {
                g.spline(&xs, &ys, st.linien_dicke, farbe);
            } else {
                for i in 1..n {
                    g.line_thick(xs[i - 1], ys[i - 1], xs[i], ys[i], st.linien_dicke, farbe);
                }
            }
            for i in 0..n {
                let glanz = if self.hover_serie == si as i32 { self.glanz_von(i) } else { 0.0 };
                if st.f_punkte {
                    let r = st.punkt_radius.max(1) + (st.hover_weite * 0.4 * glanz) as i32;
                    g.circle(xs[i], ys[i], r, self.hell(farbe, glanz));
                } else if glanz > 0.05 {
                    // Ohne dauerhafte Punkte trotzdem einen zeigen, solange die
                    // Maus darauf steht -- sonst bliebe unklar, worauf sich die
                    // Sprechblase bezieht.
                    let r = (2.0 + st.hover_weite * 0.4 * glanz) as i32;
                    g.circle(xs[i], ys[i], r.max(2), self.hell(farbe, glanz));
                }
            }
        }
        // Kategoriebeschriftung: nur so viele, wie ohne Ueberlappung passen.
        let schritt = self.label_step(g, n, plot.w());
        for i in (0..n).step_by(schritt) {
            if let Some(l) = self.labels.get(i) {
                if !l.is_empty() {
                    let tw = g.text_width_at(l, st.text_groesse);
                    g.text_styled(
                        plot.x0 + (dx * i as f64) as i32 - tw / 2,
                        plot.y1 + 5,
                        l.clone(),
                        st.c_text,
                        st.schrift,
                        st.text_groesse,
                    );
                }
            }
        }
        self.draw_axis_titles(g, a, &plot);
    }

    /// Anzahl Zellen, Luecke in Pixeln und Ausrichtung der LED-Anzeige.
    fn led_geom(&self) -> (i32, i32, bool) {
        let st = &self.style;
        (
            st.blatt_teile.max(2),
            st.blatt_luecke.max(0.0) as i32,
            !st.ausrichtung.eq_ignore_ascii_case("senkrecht"),
        )
    }

    /// Farbe an der Stelle `anteil` (0..1) der Skala.
    ///
    /// Mit Farbzonen gewinnt die Zone, in die der Wert faellt -- dieselbe
    /// Angabe bestimmt damit die Farbe von Tacho, Leiste und Lampen. Ohne
    /// Zonen wird die Palette als Verlauf durchfahren.
    fn skala_farbe(&self, anteil: f64, lo: f64, hi: f64) -> i64 {
        let wert = lo + (hi - lo) * anteil.clamp(0.0, 1.0);
        for z in &self.zones {
            if wert >= z.from.min(z.to) && wert <= z.from.max(z.to) {
                return z.color;
            }
        }
        let st = &self.style;
        let a = anteil.clamp(0.0, 1.0);
        if a < 0.5 {
            mix_rgb(st.c_skala_von, st.c_skala_mitte, a * 2.0)
        } else {
            mix_rgb(st.c_skala_mitte, st.c_skala_bis, (a - 0.5) * 2.0)
        }
    }

    /// Wert und Grenzen einer einwertigen Anzeige (Tacho/Leiste/Lampen).
    fn einzel_bereich(&self) -> (f64, f64, f64) {
        let st = &self.style;
        let lo = if st.min.is_finite() { st.min } else { 0.0 };
        let hi = if st.max.is_finite() { st.max } else { 100.0 };
        let hi = if (hi - lo).abs() < 1e-12 { lo + 1.0 } else { hi };
        let wert = self
            .series
            .first()
            .and_then(|s| self.anzeige(s).first())
            .copied()
            .unwrap_or(0.0);
        (wert, lo, hi)
    }

    /// Wert als Kapsel bzw. Blase mit Zipfel -- geteilt von Leiste und Lampen.
    ///
    /// `von_oben` = die Blase sitzt UEBER der Fundstelle und zeigt nach unten
    /// (waagerechte Anzeigen). Sonst sitzt sie rechts daneben und zeigt nach
    /// links -- bei einer senkrechten Leiste laege sie oben sonst auf der
    /// Leiste selbst und verdeckte genau den Bereich, um den es geht.
    fn marker(&self, g: &mut Graphics, mx: i32, my: i32, text: &str, farbe: i64, von_oben: bool) {
        let st = &self.style;
        let sz = st.text_groesse + 4;
        let tw = g.text_width_at(text, sz);
        let (bw, bh) = (tw + 20, sz + 12);
        // "blase" ist der dunkle Kasten, alles andere die Kapsel in Skalenfarbe.
        let (kasten, schrift) = if st.wertanzeige == "blase" {
            (0x1E1E1E, 0xF0F0F0)
        } else {
            (farbe, 0xFFFFFF)
        };
        let ecke = if st.wertanzeige == "blase" { 6 } else { bh / 2 };
        let sp = 7;
        let (x, y) = if von_oben {
            let x = (mx - bw / 2).clamp(self.x + 2, self.x + self.w - bw - 2);
            let y = (my - bh - 9).max(self.y + 2);
            g.triangle(mx - sp, y + bh, mx + sp, y + bh, mx, y + bh + 9, kasten);
            (x, y)
        } else {
            let x = (mx + 9).min(self.x + self.w - bw - 2);
            let y = (my - bh / 2).clamp(self.y + 2, self.y + self.h - bh - 2);
            g.triangle(x, my - sp, x, my + sp, x - 9, my, kasten);
            (x, y)
        };
        g.round_rect(x, y, x + bw, y + bh, ecke, kasten, true);
        g.text_styled(x + 10, y + 6, text.to_string(), schrift, st.schrift, sz);
    }

    /// Durchgehende Leiste mit Marker -- der lineare Bruder des Tachos.
    fn draw_leiste(&self, g: &mut Graphics, a: &Area) {
        let st = &self.style;
        let (wert, lo, hi) = self.einzel_bereich();
        let anteil = ((wert - lo) / (hi - lo)).clamp(0.0, 1.0);
        let waagerecht = !st.ausrichtung.eq_ignore_ascii_case("senkrecht");

        // Platz fuer den Marker freihalten, sonst stiesse er oben an.
        let marker_h = if st.wertanzeige == "aus" { 0 } else { st.text_groesse + 28 };
        let b = if waagerecht {
            Area { x0: a.x0, y0: a.y0 + marker_h, x1: a.x1, y1: a.y1 }
        } else {
            Area { x0: a.x0, y0: a.y0, x1: a.x1 - marker_h, y1: a.y1 }
        };
        let quer = if waagerecht { b.h() } else { b.w() };
        let dick = (quer as f64 * st.blatt_dicke.max(0.05) * 2.0).clamp(6.0, 400.0) as i32;
        let dick = dick.min(quer.max(6));
        let (x0, y0, x1, y1) = if waagerecht {
            let m = b.y0 + (b.h() - dick) / 2;
            (b.x0, m, b.x1, m + dick)
        } else {
            let m = b.x0 + (b.w() - dick) / 2;
            (m, b.y0, m + dick, b.y1)
        };
        let ecke = dick / 2;
        if st.schatten > 0 {
            self.schatten_rrect(g, x0, y0, x1, y1, ecke);
        }
        g.round_rect(x0, y0, x1, y1, ecke, st.c_gitter, true);

        // Verlauf in schmalen Streifen: so kommt jede Farbe aus derselben
        // Quelle (`skala_farbe`), egal ob sie aus Zonen oder Palette stammt.
        let laenge = if waagerecht { x1 - x0 } else { y1 - y0 };
        // `zeigerform`="balken" fuellt nur bis zum Wert, sonst die ganze Skala.
        let bis = if st.zeigerform.eq_ignore_ascii_case("balken") {
            (laenge as f64 * anteil) as i32
        } else {
            laenge
        };
        const S: i32 = 3;
        let mut k = 0;
        while k < bis {
            let f = k as f64 / laenge.max(1) as f64;
            let c = with_alpha(self.skala_farbe(f, lo, hi), st.deckkraft);
            let e = (k + S).min(bis);
            if waagerecht {
                g.box_fill(x0 + k, y0, x0 + e, y1, c);
            } else {
                g.box_fill(x0, y1 - e, x1, y1 - k, c);
            }
            k += S;
        }
        g.round_rect(x0, y0, x1, y1, ecke, st.c_rahmen, false);

        if st.striche >= 2 {
            for i in 0..=st.striche {
                let f = i as f64 / st.striche as f64;
                let pos = (laenge as f64 * f) as i32;
                if waagerecht {
                    g.line(x0 + pos, y0 + dick / 4, x0 + pos, y1 - dick / 4, st.c_achse);
                } else {
                    g.line(x0 + dick / 4, y1 - pos, x1 - dick / 4, y1 - pos, st.c_achse);
                }
            }
        }

        if st.wertanzeige != "aus" {
            let pos = (laenge as f64 * anteil) as i32;
            let farbe = self.skala_farbe(anteil, lo, hi);
            let text = self.fmt_value(wert);
            if waagerecht {
                self.marker(g, x0 + pos, y0, &text, farbe, true);
            } else {
                self.marker(g, x1, y1 - pos, &text, farbe, false);
            }
        }
    }

    /// Diskrete Zellen, die bis zum Wert leuchten.
    fn draw_led(&self, g: &mut Graphics, a: &Area) {
        let st = &self.style;
        let (wert, lo, hi) = self.einzel_bereich();
        let anteil = ((wert - lo) / (hi - lo)).clamp(0.0, 1.0);
        let (zellen, luecke, waagerecht) = self.led_geom();

        let marker_h = if st.wertanzeige == "aus" { 0 } else { st.text_groesse + 28 };
        let b = if waagerecht {
            Area { x0: a.x0, y0: a.y0 + marker_h, x1: a.x1, y1: a.y1 }
        } else {
            Area { x0: a.x0, y0: a.y0, x1: a.x1 - marker_h, y1: a.y1 }
        };
        let laenge = if waagerecht { b.w() } else { b.h() };
        let quer = if waagerecht { b.h() } else { b.w() };
        let dick = (quer as f64 * st.blatt_dicke.max(0.05) * 2.0).clamp(6.0, 400.0) as i32;
        let dick = dick.min(quer.max(6));
        let fach = laenge as f64 / zellen as f64;
        let zelle = (fach - luecke as f64).max(2.0) as i32;
        // Aufrunden, damit ein Wert knapp ueber Null schon die erste Lampe
        // zuendet -- sonst wirkt die Anzeige bei kleinen Werten tot.
        let an = (anteil * zellen as f64).ceil() as i32;

        for i in 0..zellen {
            let f = if zellen > 1 { i as f64 / (zellen - 1) as f64 } else { 0.0 };
            let grund = self.skala_farbe(f, lo, hi);
            let c = if i < an {
                self.hell(with_alpha(grund, st.deckkraft), self.glanz_von(i as usize))
            } else {
                // Aus, aber nicht unsichtbar: stark abgedunkelte Eigenfarbe,
                // damit die Skala auch im Ruhezustand ablesbar bleibt.
                mix_rgb(grund, st.c_hintergrund, 0.82)
            };
            let pos = (fach * i as f64) as i32;
            let (x0, y0, x1, y1) = if waagerecht {
                let m = b.y0 + (b.h() - dick) / 2;
                (b.x0 + pos, m, b.x0 + pos + zelle, m + dick)
            } else {
                let m = b.x0 + (b.w() - dick) / 2;
                (m, b.y1 - pos - zelle, m + dick, b.y1 - pos)
            };
            let ecke = zelle.min(dick) / 3;
            if st.schatten > 0 && i < an {
                self.schatten_rrect(g, x0, y0, x1, y1, ecke);
            }
            self.fuellung(g, x0, y0, x1, y1, ecke, c);
        }

        if st.wertanzeige != "aus" {
            let pos = (laenge as f64 * anteil) as i32;
            let farbe = self.skala_farbe(anteil, lo, hi);
            let text = self.fmt_value(wert);
            let rand = (quer - dick) / 2;
            if waagerecht {
                self.marker(g, b.x0 + pos, b.y0 + rand, &text, farbe, true);
            } else {
                self.marker(g, b.x0 + b.w() / 2 + dick / 2, b.y1 - pos, &text, farbe, false);
            }
        }
    }

    fn draw_gauge(&self, g: &mut Graphics, a: &Area) {
        let st = &self.style;
        let wert = self
            .series
            .first()
            .and_then(|s| self.anzeige(s).first())
            .copied()
            .unwrap_or(0.0);
        let lo = if st.min.is_finite() { st.min } else { 0.0 };
        let hi = if st.max.is_finite() { st.max } else { 100.0 };
        let hi = if (hi - lo).abs() < 1e-12 { lo + 1.0 } else { hi };
        let (von, bis) = (st.start_winkel, st.end_winkel);
        let spanne = bis - von;

        let (cx, cy) = (a.x0 + a.w() / 2, a.y0 + a.h() / 2);
        let r = (a.w().min(a.h()) / 2 - 2).max(6);
        let dicke = (r / 5).max(4);
        let ri = r - dicke;

        // Fassung: metallischer Ring um die Scheibe (skeuomorpher Stil).
        // Mehrere Ringe mit wechselnder Helligkeit ergeben den Metall-Eindruck,
        // ohne dass es dafuer einen Verlauf entlang eines Kreises braeuchte
        // (den kann `ring` nicht).
        if st.fassung > 0 {
            let f = st.fassung;
            for k in 0..f {
                let anteil = k as f64 / f.max(1) as f64;
                // hell -> dunkel -> hell: wirkt wie eine gewoelbte Kante
                let hell = 1.0 - (anteil - 0.35).abs() * 1.6;
                let c = mix_rgb(st.c_rahmen, 0xFFFFFF, hell.clamp(0.0, 1.0) * 0.55);
                g.ring(cx, cy, r + k, r + k + 1, 0.0, 360.0, c, true);
            }
        }

        // Skalenbogen als Untergrund, dann die Farbzonen darueber.
        //
        // Beim Balken-Zeiger fuellt der Fortschritt denselben Ring wie die
        // Zonen. Damit beides ablesbar bleibt, liegen die Zonen dort
        // durchgehend als schmaler Aussenrand -- durchgehend deshalb, weil
        // sie sonst an der Fortschrittskante ihre Breite wechseln wuerden.
        let balken_zeiger = st.zeigerform.eq_ignore_ascii_case("balken");
        let zone_ri = if balken_zeiger { r - (r - ri) * 2 / 5 } else { ri };
        // Ein Bogenstueck in der gewaehlten Zifferblatt-Bauart zeichnen.
        // Untergrund UND Farbzonen laufen hier durch -- sonst waere der
        // Untergrund segmentiert und die Zone ein durchgehender Ring.
        let bogen = |g: &mut Graphics, r_in: i32, r_out: i32, w1: f64, w2: f64, farbe: i64| {
            match st.zifferblatt.as_str() {
                "segmente" | "striche" => {
                    let teile = st.blatt_teile.max(2);
                    let schritt = (bis - von) / teile as f64;
                    let luecke = st.blatt_luecke.min(schritt * 0.9);
                    // Nur die Teile zeichnen, die in [w1,w2] liegen -- so faerbt
                    // eine Zone genau ihre Segmente ein.
                    for k in 0..teile {
                        let a1 = von + schritt * k as f64;
                        let a2 = a1 + schritt - luecke;
                        let mitte = (a1 + a2) / 2.0;
                        if mitte < w1 - 1e-9 || mitte >= w2 {
                            continue;
                        }
                        // "striche" ist die duenne Fassung von "segmente".
                        let (si, so) = if st.zifferblatt == "striche" {
                            (r_out - (r_out - r_in) / 3, r_out)
                        } else {
                            (r_in, r_out)
                        };
                        g.ring(cx, cy, si, so, a1, a2, farbe, true);
                    }
                }
                "baender" => {
                    // Volle Sektoren bis zur Mitte, mit kleiner Luecke.
                    let l = st.blatt_luecke.min((w2 - w1).abs() * 0.4);
                    g.ring(cx, cy, 0, r_out, w1, (w2 - l).max(w1), farbe, true);
                }
                _ => g.ring(cx, cy, r_in, r_out, w1, w2, farbe, true),
            }
        };
        let zonen = |g: &mut Graphics| {
            for z in &self.zones {
                let (zf, zt) = (z.from.min(z.to), z.from.max(z.to));
                let f = ((zf - lo) / (hi - lo)).clamp(0.0, 1.0);
                let t = ((zt - lo) / (hi - lo)).clamp(0.0, 1.0);
                if t > f {
                    bogen(g, zone_ri, r, von + spanne * f, von + spanne * t, z.color);
                }
            }
        };
        bogen(g, ri, r, von, bis, st.c_gitter);
        zonen(g);

        // Zonen-Beschriftung entlang des Bogens (POOR / NORMAL / GUT ...).
        for z in &self.zones {
            if z.name.is_empty() {
                continue;
            }
            let (zf, zt) = (z.from.min(z.to), z.from.max(z.to));
            let f = ((zf - lo) / (hi - lo)).clamp(0.0, 1.0);
            let tt = ((zt - lo) / (hi - lo)).clamp(0.0, 1.0);
            if tt <= f {
                continue;
            }
            let mitte = von + spanne * (f + tt) / 2.0;
            let rad = mitte.to_radians();
            // Bei "baender" liegt die Schrift weiter innen (der Sektor geht
            // bis zur Mitte), sonst mittig im Ring.
            let lr = if st.zifferblatt == "baender" {
                r as f64 * 0.7
            } else {
                (ri + r) as f64 / 2.0
            };
            // Tangential ausrichten: +90 Grad zur Radialen. In der unteren
            // Haelfte kaeme der Text auf dem Kopf an -> dort umdrehen.
            let mut dreh = mitte + 90.0;
            let norm = ((mitte % 360.0) + 360.0) % 360.0;
            if norm > 90.0 && norm < 270.0 {
                dreh += 180.0;
            }
            g.text_rot(
                cx + (rad.cos() * lr) as i32,
                cy + (rad.sin() * lr) as i32,
                z.name.clone(),
                dreh as f32,
                st.text_groesse as f32 / g.text_height().max(1) as f32,
                st.c_text,
            );
        }

        // Striche (lang) + Unterstriche (kurz) samt Beschriftung.
        let haupt = st.striche.max(0);
        if haupt >= 2 {
            let unter = st.unterstriche.max(0);
            for i in 0..haupt {
                let t = i as f64 / (haupt - 1) as f64;
                let w = (von + spanne * t).to_radians();
                let (c, s) = (w.cos(), w.sin());
                g.line_thick(
                    cx + (c * (ri - 2) as f64) as i32,
                    cy + (s * (ri - 2) as f64) as i32,
                    cx + (c * (ri - dicke) as f64) as i32,
                    cy + (s * (ri - dicke) as f64) as i32,
                    2.0,
                    st.c_achse,
                );
                let beschriftung = self.fmt_value(lo + (hi - lo) * t);
                let lr = (ri - dicke - 10) as f64;
                let tw = g.text_width_at(&beschriftung, st.text_groesse);
                g.text_styled(
                    cx + (c * lr) as i32 - tw / 2,
                    cy + (s * lr) as i32 - st.text_groesse / 2,
                    beschriftung,
                    st.c_text,
                    st.schrift,
                    st.text_groesse,
                );
                // Unterstriche zwischen diesem und dem naechsten Hauptstrich.
                if i + 1 < haupt {
                    for k in 1..=unter {
                        let tt = t + (k as f64 / (unter + 1) as f64) / (haupt - 1) as f64;
                        let w = (von + spanne * tt).to_radians();
                        let (c, s) = (w.cos(), w.sin());
                        g.line_thick(
                            cx + (c * (ri - 2) as f64) as i32,
                            cy + (s * (ri - 2) as f64) as i32,
                            cx + (c * (ri - dicke / 2) as f64) as i32,
                            cy + (s * (ri - dicke / 2) as f64) as i32,
                            1.0,
                            st.c_achse,
                        );
                    }
                }
            }
        }

        let t = ((wert - lo) / (hi - lo)).clamp(0.0, 1.0);
        let zw = von + spanne * t;
        let zeiger = with_alpha(st.c_zeiger, st.deckkraft);

        // Zeiger UND sein Schatten kommen aus derselben Routine, nur mit
        // Versatz und Farbe unterschiedlich. Vorher zeichnete der Schatten
        // immer eine Nadel-Linie -- bei Zeigerform "balken"/"pfeil" lag also
        // ein Strich auf dem Blatt, den es gar nicht gab; ausserdem begann er
        // am Drehpunkt, waehrend die echte Nadel ein Stueck dahinter anfaengt.
        let zeiger_zeichnen = |g: &mut Graphics, dx: i32, dy: i32, farbe: i64| {
            let (cx, cy) = (cx + dx, cy + dy);
            let w = zw.to_radians();
            let laenge = (ri - dicke / 2) as f64;
            match st.zeigerform.to_lowercase().as_str() {
                "balken" => {
                    // Kein Zeiger, sondern der zurueckgelegte Teil des Bogens.
                    if t > 0.0 {
                        g.ring(cx, cy, ri, r, von, zw, farbe, true);
                    }
                }
                "pfeil" => {
                    let (sx, sy) = (cx + (w.cos() * laenge) as i32, cy + (w.sin() * laenge) as i32);
                    // Basis-Ecken gut 130 Grad neben der Spitze -> schlanker Pfeil.
                    let (q1, q2) = (w + 2.4, w - 2.4);
                    let br = (r / 12).max(3) as f64;
                    let flach = [
                        sx, sy,
                        cx + (q1.cos() * br) as i32, cy + (q1.sin() * br) as i32,
                        cx + (q2.cos() * br) as i32, cy + (q2.sin() * br) as i32,
                    ];
                    let _ = g.polygon(&flach, farbe, true);
                }
                _ => {
                    // Kurzes Gegenstueck hinter der Achse -- so sitzt die Nadel
                    // optisch auf dem Drehpunkt statt daran zu kleben.
                    g.line_thick(
                        cx - (w.cos() * (r / 8) as f64) as i32,
                        cy - (w.sin() * (r / 8) as f64) as i32,
                        cx + (w.cos() * laenge) as i32,
                        cy + (w.sin() * laenge) as i32,
                        (st.linien_dicke * 1.5).max(2.0),
                        farbe,
                    );
                }
            }
            // Die Nabe gehoert zum Zeiger -- sonst schwebt der Schatten am
            // Drehpunkt ohne Anschluss.
            if st.zeigerform.to_lowercase() != "balken" {
                g.circle(cx, cy, (r / 12).max(3), farbe);
            }
        };

        if st.f_schatten_daten && st.schatten > 0 {
            let o = st.schatten;
            let lagen = st.schatten_weich.max(0);
            if lagen == 0 {
                zeiger_zeichnen(g, o, o, st.c_schatten);
            } else {
                // Weich wie das Feld: gestaffelte Kopien mit Bruchteil-Deckkraft.
                let c = with_alpha(st.c_schatten, 1.0 / (lagen + 1) as f64);
                for k in 0..=lagen {
                    zeiger_zeichnen(g, o + k - lagen / 2, o + k - lagen / 2, c);
                }
            }
        }
        zeiger_zeichnen(g, 0, 0, zeiger);
        if balken_zeiger {
            // Der Fortschrittsbogen hat die Zonen gerade ueberdeckt -- sie
            // kommen darum noch einmal obenauf (gleiche Breite wie oben).
            zonen(g);
        } else {
            g.circle(cx, cy, (r / 24).max(1), st.c_hintergrund);
        }

        // --- Wertanzeige -------------------------------------------------
        // Der Tacho haengt allein an `wertanzeige`. `werte` bleibt fuer die
        // Beschriftung einzelner Datenpunkte bei Kuchen/Balken zustaendig --
        // beides zu koppeln hiesse, zwei Schalter fuer dieselbe Sache zu haben,
        // und dann zeigt der Tacho nichts, weil der andere noch auf "aus" steht.
        if st.wertanzeige != "aus" {
            let text = self.fmt_value(wert);
            let sz = st.text_groesse * 2;
            let tw = g.text_width_at(&text, sz);
            match st.wertanzeige.as_str() {
                "pille" => {
                    // Abgerundete Kapsel in der Farbe der getroffenen Zone --
                    // so sagt schon die Farbe, wie der Wert einzuordnen ist.
                    let farbe = self
                        .zones
                        .iter()
                        .find(|z| wert >= z.from.min(z.to) && wert <= z.from.max(z.to))
                        .map(|z| z.color)
                        .unwrap_or(st.c_zeiger);
                    let (bw, bh) = (tw + 24, sz + 14);
                    let (x, y) = (cx - bw / 2, cy + (r * 3) / 5 - bh / 2);
                    g.round_rect(x, y, x + bw, y + bh, bh / 2, farbe, true);
                    g.text_styled(x + 12, y + 7, text, 0xFFFFFF, st.schrift, sz);
                }
                "blase" => {
                    // Dunkler Kasten mit Zipfel nach oben (wie die Tooltips).
                    let (bw, bh) = (tw + 24, sz + 14);
                    let (x, y) = (cx - bw / 2, cy + (r * 3) / 5 - bh / 2);
                    g.round_rect(x, y, x + bw, y + bh, 6, 0x1E1E1E, true);
                    let sp = 7;
                    g.triangle(cx - sp, y, cx + sp, y, cx, y - 9, 0x1E1E1E);
                    g.text_styled(x + 12, y + 7, text, 0xF0F0F0, st.schrift, sz);
                }
                "am_zeiger" => {
                    // Kapsel an der Zeigerspitze -- sie wandert mit dem Wert.
                    let w = zw.to_radians();
                    let lr = (ri - dicke / 2) as f64;
                    let (px, py) = (cx + (w.cos() * lr) as i32, cy + (w.sin() * lr) as i32);
                    let s2 = st.text_groesse;
                    let tw2 = g.text_width_at(&text, s2);
                    let (bw, bh) = (tw2 + 18, s2 + 10);
                    let x = (px - bw / 2).clamp(self.x + 2, self.x + self.w - bw - 2);
                    let y = (py - bh / 2).clamp(self.y + 2, self.y + self.h - bh - 2);
                    let farbe = self
                        .zones
                        .iter()
                        .find(|z| wert >= z.from.min(z.to) && wert <= z.from.max(z.to))
                        .map(|z| z.color)
                        .unwrap_or(st.c_zeiger);
                    g.round_rect(x, y, x + bw, y + bh, bh / 2, farbe, true);
                    g.text_styled(x + 9, y + 5, text, 0xFFFFFF, st.schrift, s2);
                }
                // "innen": schlicht unter der Mitte.
                _ => {
                    g.text_styled(cx - tw / 2, cy + (r * 3) / 5, text, st.c_text, st.schrift, sz);
                }
            }
        }
    }

    // --- gemeinsame Achsen-Hilfen -----------------------------------------

    /// Zeichenflaeche nach Abzug des Platzes fuer die Achsenbeschriftung.
    fn axis_area(&self, g: &Graphics, a: &Area, lo: f64, hi: f64, waagerecht: bool) -> Area {
        let st = &self.style;
        let breiteste = {
            let s1 = self.fmt_value(lo);
            let s2 = self.fmt_value(hi);
            g.text_width_at(&s1, st.text_groesse).max(g.text_width_at(&s2, st.text_groesse))
        };
        let mut r = Area { x0: a.x0, y0: a.y0, x1: a.x1, y1: a.y1 };
        // Halbe Zeilenhoehe oben, damit die oberste Achsenzahl nicht abgeschnitten wird.
        r.y0 += st.text_groesse / 2;
        if waagerecht {
            let kat = self
                .labels
                .iter()
                .map(|l| g.text_width_at(l, st.text_groesse))
                .max()
                .unwrap_or(0);
            r.x0 += kat + 8;
            r.y1 -= st.text_groesse + 6;
        } else {
            r.x0 += breiteste + 8;
            r.y1 -= st.text_groesse + 6;
        }
        if !st.achse_x.is_empty() {
            r.y1 -= st.text_groesse + 4;
        }
        if !st.achse_y.is_empty() {
            r.x0 += st.text_groesse + 4;
        }
        r
    }

    /// Bildschirmposition eines Wertes auf der Wertachse.
    fn val_pos(&self, plot: &Area, v: f64, lo: f64, hi: f64, waagerecht: bool) -> i32 {
        let t = ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
        if waagerecht {
            plot.x0 + (plot.w() as f64 * t) as i32
        } else {
            plot.y1 - (plot.h() as f64 * t) as i32
        }
    }

    fn draw_grid(&self, g: &mut Graphics, plot: &Area, lo: f64, hi: f64, waagerecht: bool) {
        let st = &self.style;
        let schritt = if st.gitter > 0.0 { st.gitter } else { nice_step(hi - lo, 5) };
        if schritt <= 0.0 {
            return;
        }
        let quer = if waagerecht { st.f_gitter_x } else { st.f_gitter_y };
        let mut v = (lo / schritt).ceil() * schritt;
        let mut wache = 0;
        while v <= hi + 1e-9 && wache < 200 {
            wache += 1;
            let p = self.val_pos(plot, v, lo, hi, waagerecht);
            let ist_null = v.abs() < schritt * 1e-6;
            let farbe = if ist_null && st.f_null_linie { st.c_achse } else { st.c_gitter };
            if quer || ist_null {
                if waagerecht {
                    g.line(p, plot.y0, p, plot.y1, farbe);
                } else {
                    g.line(plot.x0, p, plot.x1, p, farbe);
                }
            }
            // Achsenzahl
            let t = self.fmt_value(v);
            if waagerecht {
                let tw = g.text_width_at(&t, st.text_groesse);
                g.text_styled(p - tw / 2, plot.y1 + 5, t, st.c_text, st.schrift, st.text_groesse);
            } else {
                let tw = g.text_width_at(&t, st.text_groesse);
                g.text_styled(
                    plot.x0 - tw - 6,
                    p - st.text_groesse / 2,
                    t,
                    st.c_text,
                    st.schrift,
                    st.text_groesse,
                );
            }
            v += schritt;
        }
        g.line(plot.x0, plot.y0, plot.x0, plot.y1, st.c_achse);
        g.line(plot.x0, plot.y1, plot.x1, plot.y1, st.c_achse);
    }

    fn draw_axis_titles(&self, g: &mut Graphics, a: &Area, plot: &Area) {
        let st = &self.style;
        if !st.achse_x.is_empty() {
            let tw = g.text_width_at(&st.achse_x, st.text_groesse);
            g.text_styled(
                plot.x0 + (plot.w() - tw) / 2,
                a.y1 - st.text_groesse,
                st.achse_x.clone(),
                st.c_text,
                st.schrift,
                st.text_groesse,
            );
        }
        if !st.achse_y.is_empty() {
            // Gedreht an der linken Kante -- waagerecht braeuchte es zu viel Platz.
            g.text_rot(
                a.x0 + st.text_groesse,
                plot.y0 + plot.h() / 2 + g.text_width_at(&st.achse_y, st.text_groesse) / 2,
                st.achse_y.clone(),
                -90.0,
                st.text_groesse as f32 / g.text_height().max(1) as f32,
                st.c_text,
            );
        }
    }

    /// Wie viele Kategoriebeschriftungen dargestellt werden koennen, ohne dass
    /// sie ineinanderlaufen (bei Live-Kurven mit 200 Punkten sonst Brei).
    fn label_step(&self, g: &Graphics, n: usize, breite: i32) -> usize {
        let noetig = self
            .labels
            .iter()
            .map(|l| g.text_width_at(l, self.style.text_groesse) + 10)
            .max()
            .unwrap_or(1)
            .max(1);
        let passen = (breite / noetig).max(1) as usize;
        (n.div_ceil(passen)).max(1)
    }
}

// ---------------------------------------------------------------------------
// Schluessel-Tabellen fuer die vier Setter. Ein Ort je Setter -- die Fehler-
// meldung listet daraus die gueltigen Namen auf.
// ---------------------------------------------------------------------------

pub const KEYS_STR: &[&str] = &[
    "titel", "einheit", "achse_x", "achse_y", "legende", "werte", "ausrichtung", "zeigerform",
    "zifferblatt", "wertanzeige",
];
pub const KEYS_NUM: &[&str] = &[
    "min", "max", "innenradius", "abstand", "ecken", "rahmen_dicke", "polster", "gitter",
    "nachkomma", "titel_groesse", "text_groesse", "schrift", "start_winkel", "end_winkel",
    "striche", "unterstriche", "linien_dicke", "punkt_radius", "animation", "fenster", "schatten",
    "schatten_weich", "deckkraft", "flaeche_deckkraft",
    "hover_tempo", "hover_weite", "hover_glanz",
    "blatt_teile", "blatt_luecke", "blatt_dicke", "fassung",
];
pub const KEYS_COLOR: &[&str] = &[
    "hintergrund", "rahmen", "gitter", "text", "titel", "achse", "zeiger", "flaeche", "verlauf",
    "schatten", "verlauf_ende", "skala_von", "skala_mitte", "skala_bis",
];
pub const KEYS_FLAG: &[&str] = &[
    "rahmen", "gitter_x", "gitter_y", "prozent", "flaeche", "punkte", "glatt", "null_linie",
    "stapel", "verlauf", "verlauf_daten", "schatten_daten", "kurz",
    "hover", "tooltip",
];

fn unbekannt(fn_: &str, key: &str, gueltig: &[&str]) -> String {
    format!("{}: unbekannte Eigenschaft '{}' (gueltig: {})", fn_, key, gueltig.join(", "))
}

impl ChartObj {
    pub fn set_str(&mut self, key: &str, v: &str) -> Result<(), String> {
        let k = key.to_lowercase();
        let s = &mut self.style;
        match k.as_str() {
            "titel" => s.titel = v.to_string(),
            "einheit" => s.einheit = v.to_string(),
            "achse_x" => s.achse_x = v.to_string(),
            "achse_y" => s.achse_y = v.to_string(),
            "legende" => {
                let vv = v.to_lowercase();
                if !["aus", "oben", "unten", "links", "rechts"].contains(&vv.as_str()) {
                    return Err(format!(
                        "CHART_SET: legende erwartet aus/oben/unten/links/rechts, nicht '{}'",
                        v
                    ));
                }
                s.legende = vv;
            }
            "werte" => {
                let vv = v.to_lowercase();
                if !["aus", "innen", "aussen"].contains(&vv.as_str()) {
                    return Err(format!("CHART_SET: werte erwartet aus/innen/aussen, nicht '{}'", v));
                }
                s.werte = vv;
            }
            "ausrichtung" => {
                let vv = v.to_lowercase();
                if !["senkrecht", "waagerecht"].contains(&vv.as_str()) {
                    return Err(format!(
                        "CHART_SET: ausrichtung erwartet senkrecht/waagerecht, nicht '{}'",
                        v
                    ));
                }
                s.ausrichtung = vv;
            }
            "zeigerform" => {
                let vv = v.to_lowercase();
                if !["nadel", "balken", "pfeil"].contains(&vv.as_str()) {
                    return Err(format!(
                        "CHART_SET: zeigerform erwartet nadel/balken/pfeil, nicht '{}'",
                        v
                    ));
                }
                s.zeigerform = vv;
            }
            "zifferblatt" => {
                let vv = v.to_lowercase();
                if !["ring", "segmente", "striche", "baender"].contains(&vv.as_str()) {
                    return Err(format!(
                        "CHART_SET: zifferblatt erwartet ring/segmente/striche/baender, nicht '{}'", v));
                }
                s.zifferblatt = vv;
            }
            "wertanzeige" => {
                let vv = v.to_lowercase();
                if !["aus", "innen", "pille", "blase", "am_zeiger"].contains(&vv.as_str()) {
                    return Err(format!(
                        "CHART_SET: wertanzeige erwartet aus/innen/pille/blase/am_zeiger, nicht '{}'", v));
                }
                s.wertanzeige = vv;
            }
            _ => return Err(unbekannt("CHART_SET", key, KEYS_STR)),
        }
        Ok(())
    }

    pub fn set_num(&mut self, key: &str, v: f64) -> Result<(), String> {
        let k = key.to_lowercase();
        let s = &mut self.style;
        match k.as_str() {
            "min" => s.min = v,
            "max" => s.max = v,
            "innenradius" => s.innenradius = v.clamp(0.0, 0.95),
            "abstand" => s.abstand = v.max(0.0),
            "ecken" => s.ecken = v.max(0.0) as i32,
            "rahmen_dicke" => s.rahmen_dicke = v.clamp(0.0, 20.0) as i32,
            "polster" => s.polster = v.clamp(0.0, 500.0) as i32,
            "gitter" => s.gitter = v.max(0.0),
            "nachkomma" => s.nachkomma = v.clamp(0.0, 9.0) as i32,
            "titel_groesse" => s.titel_groesse = v.clamp(4.0, 200.0) as i32,
            "text_groesse" => s.text_groesse = v.clamp(4.0, 200.0) as i32,
            "schrift" => s.schrift = v as i64,
            "start_winkel" => s.start_winkel = v,
            "end_winkel" => s.end_winkel = v,
            "striche" => s.striche = v.clamp(0.0, 100.0) as i32,
            "unterstriche" => s.unterstriche = v.clamp(0.0, 50.0) as i32,
            "linien_dicke" => s.linien_dicke = v.clamp(0.1, 50.0),
            "punkt_radius" => s.punkt_radius = v.clamp(0.0, 100.0) as i32,
            "animation" => s.animation = v.max(0.0),
            "fenster" => s.fenster = v.max(0.0) as i32,
            "schatten" => s.schatten = v.clamp(0.0, 50.0) as i32,
            "schatten_weich" => s.schatten_weich = v.clamp(0.0, 30.0) as i32,
            "deckkraft" => s.deckkraft = v.clamp(0.0, 1.0),
            "flaeche_deckkraft" => s.flaeche_deckkraft = v.clamp(0.0, 1.0),
            "hover_tempo" => s.hover_tempo = v.clamp(0.0, 5.0),
            "hover_weite" => s.hover_weite = v.clamp(0.0, 100.0),
            "hover_glanz" => s.hover_glanz = v.clamp(0.0, 1.0),
            "blatt_teile" => s.blatt_teile = v.clamp(2.0, 200.0) as i32,
            "blatt_luecke" => s.blatt_luecke = v.clamp(0.0, 30.0),
            "blatt_dicke" => s.blatt_dicke = v.clamp(0.05, 0.6),
            "fassung" => s.fassung = v.clamp(0.0, 60.0) as i32,
            _ => return Err(unbekannt("CHART_SET_NUM", key, KEYS_NUM)),
        }
        Ok(())
    }

    pub fn set_color(&mut self, key: &str, v: i64) -> Result<(), String> {
        let k = key.to_lowercase();
        let s = &mut self.style;
        match k.as_str() {
            "hintergrund" => s.c_hintergrund = v,
            "rahmen" => s.c_rahmen = v,
            "gitter" => s.c_gitter = v,
            "text" => s.c_text = v,
            "titel" => s.c_titel = v,
            "achse" => s.c_achse = v,
            "zeiger" => s.c_zeiger = v,
            "flaeche" => s.c_flaeche = v,
            "verlauf" => s.c_verlauf = v,
            "schatten" => s.c_schatten = v,
            "verlauf_ende" => s.c_verlauf_ende = v,
            "skala_von" => s.c_skala_von = v,
            "skala_mitte" => s.c_skala_mitte = v,
            "skala_bis" => s.c_skala_bis = v,
            _ => return Err(unbekannt("CHART_SET_COLOR", key, KEYS_COLOR)),
        }
        Ok(())
    }

    pub fn set_flag(&mut self, key: &str, v: bool) -> Result<(), String> {
        let k = key.to_lowercase();
        let s = &mut self.style;
        match k.as_str() {
            "rahmen" => s.f_rahmen = v,
            "gitter_x" => s.f_gitter_x = v,
            "gitter_y" => s.f_gitter_y = v,
            "prozent" => s.f_prozent = v,
            "flaeche" => s.f_flaeche = v,
            "punkte" => s.f_punkte = v,
            "glatt" => s.f_glatt = v,
            "null_linie" => s.f_null_linie = v,
            "stapel" => s.f_stapel = v,
            "verlauf" => s.f_verlauf = v,
            "verlauf_daten" => s.f_verlauf_daten = v,
            "schatten_daten" => s.f_schatten_daten = v,
            "kurz" => s.f_kurz = v,
            "hover" => s.f_hover = v,
            "tooltip" => s.f_tooltip = v,
            _ => return Err(unbekannt("CHART_SET_FLAG", key, KEYS_FLAG)),
        }
        Ok(())
    }

    /// Eigene Farbreihenfolge fuer Reihen/Segmente ohne gesetzte Farbe.
    pub fn set_palette(&mut self, farben: &[i64]) {
        if !farben.is_empty() {
            self.style.palette = farben.to_vec();
        }
    }
}

impl ChartObj {
    /// Nur fuer Tests: `hell()` liegt im Grafik-Zweig, die Farbmathematik
    /// soll aber auch ohne Fenster pruefbar sein.
    #[cfg(test)]
    pub fn hell_test(&self, farbe: i64, glanz: f64) -> i64 {
        mix_rgb(farbe, 0xFFFFFF, self.style.hover_glanz * glanz)
    }

    /// Nur fuer Tests -- gleiche Begruendung wie `hell_test`.
    #[cfg(test)]
    pub fn skala_farbe_test(&self, anteil: f64, lo: f64, hi: f64) -> i64 {
        let wert = lo + (hi - lo) * anteil.clamp(0.0, 1.0);
        for z in &self.zones {
            if wert >= z.from.min(z.to) && wert <= z.from.max(z.to) {
                return z.color;
            }
        }
        let s = &self.style;
        let a = anteil.clamp(0.0, 1.0);
        if a < 0.5 {
            mix_rgb(s.c_skala_von, s.c_skala_mitte, a * 2.0)
        } else {
            mix_rgb(s.c_skala_mitte, s.c_skala_bis, (a - 0.5) * 2.0)
        }
    }
}

/// Vorgefertigte Farbwelten (fuer die Fehlermeldung von CHART_THEME).
pub const THEMES: &[&str] = &["dunkel", "hell", "neon", "pastell"];

/// Statistik-Kennzahlen ueber eine Reihe -- die typischen Fragen an einen
/// Datensatz, ohne dass das GB-Programm selbst schleifen muss.
pub fn stats(values: &[f64]) -> HashMap<&'static str, f64> {
    let mut m = HashMap::new();
    let gueltig: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let n = gueltig.len();
    m.insert("anzahl", n as f64);
    if n == 0 {
        for k in ["summe", "mittel", "min", "max"] {
            m.insert(k, 0.0);
        }
        return m;
    }
    let summe: f64 = gueltig.iter().sum();
    m.insert("summe", summe);
    m.insert("mittel", summe / n as f64);
    m.insert("min", gueltig.iter().copied().fold(f64::INFINITY, f64::min));
    m.insert("max", gueltig.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_parse_akzeptiert_deutsch_und_englisch() {
        assert_eq!(Kind::parse("Kuchen"), Some(Kind::Pie));
        assert_eq!(Kind::parse("PIE"), Some(Kind::Pie));
        assert_eq!(Kind::parse("tacho"), Some(Kind::Gauge));
        assert_eq!(Kind::parse("balken"), Some(Kind::Bar));
        assert_eq!(Kind::parse("torte"), None);
    }

    #[test]
    fn add_point_haelt_alle_reihen_gleich_lang() {
        let mut c = ChartObj::new(Kind::Bar, 0, 0, 100, 100);
        c.add_series("a", -1);
        c.add_series("b", -1);
        c.add_point("Mo", 5.0, -1);
        c.add_point("Di", 7.0, -1);
        assert_eq!(c.labels.len(), 2);
        for s in &c.series {
            assert_eq!(s.values.len(), 2, "Reihe {} laeuft aus dem Takt", s.name);
        }
        // Nur die erste Reihe bekommt den Wert aus ADD.
        assert_eq!(c.series[0].values, vec![5.0, 7.0]);
        assert_eq!(c.series[1].values, vec![0.0, 0.0]);
    }

    #[test]
    fn add_point_ohne_vorherige_series_verschiebt_nichts() {
        // Regression: CHART_ADD legte die Kategorie an, BEVOR die implizite
        // erste Reihe entstand. Die Reihe wurde dann mit einem Nullwert fuer
        // die schon gezaehlte Kategorie vorbelegt -- jeder Wert sass danach
        // eine Stelle neben seiner Beschriftung.
        let mut c = ChartObj::new(Kind::Pie, 0, 0, 100, 100);
        c.add_point("Holz", 45.0, -1);
        c.add_point("Stein", 30.0, -1);
        assert_eq!(c.labels, vec!["Holz", "Stein"]);
        assert_eq!(c.series[0].values, vec![45.0, 30.0]);
        assert_eq!(c.series[0].values.len(), c.labels.len());
    }

    #[test]
    fn push_mit_fenster_laesst_vorne_herausfallen() {
        let mut c = ChartObj::new(Kind::Line, 0, 0, 100, 100);
        c.add_series("s", -1);
        c.set_num("fenster", 3.0).unwrap();
        for i in 0..6 {
            c.push(0, i as f64).unwrap();
        }
        assert_eq!(c.series[0].values, vec![3.0, 4.0, 5.0]);
        assert_eq!(c.labels.len(), 3, "Kategorien muessen mitwandern");
    }

    #[test]
    fn range_schliesst_die_null_ein() {
        let mut c = ChartObj::new(Kind::Bar, 0, 0, 100, 100);
        c.add_series("s", -1);
        c.set_data(0, &[5.0, 8.0, 6.0]).unwrap();
        let (lo, hi) = c.range();
        assert_eq!(lo, 0.0, "sonst luegen die Balkenlaengen");
        assert_eq!(hi, 8.0);
    }

    #[test]
    fn range_stapelt_wenn_gestapelt_gezeichnet_wird() {
        let mut c = ChartObj::new(Kind::Bar, 0, 0, 100, 100);
        c.add_series("a", -1);
        c.add_series("b", -1);
        c.set_data(0, &[3.0, 4.0]).unwrap();
        c.set_data(1, &[5.0, 1.0]).unwrap();
        c.set_flag("stapel", true).unwrap();
        let (_, hi) = c.range();
        assert_eq!(hi, 8.0, "gestapelt zaehlt die Summe, nicht das Maximum");
    }

    #[test]
    fn feste_grenzen_schlagen_die_daten() {
        let mut c = ChartObj::new(Kind::Bar, 0, 0, 100, 100);
        c.add_series("s", -1);
        c.set_data(0, &[5.0]).unwrap();
        c.set_num("max", 20.0).unwrap();
        let (_, hi) = c.range();
        assert_eq!(hi, 20.0);
    }

    #[test]
    fn animation_zieht_nach_und_kommt_an() {
        let mut c = ChartObj::new(Kind::Gauge, 0, 0, 100, 100);
        c.set_num("animation", 0.3).unwrap();
        c.set_point(0, 0, 100.0).unwrap();
        c.advance(0.016);
        let zwischen = c.series[0].shown[0];
        assert!(zwischen > 0.0 && zwischen < 100.0, "sprang sofort: {}", zwischen);
        for _ in 0..600 {
            c.advance(0.016);
        }
        assert!((c.series[0].shown[0] - 100.0).abs() < 1e-6, "kam nie an");
    }

    #[test]
    fn ohne_animation_ist_der_wert_sofort_da() {
        let mut c = ChartObj::new(Kind::Gauge, 0, 0, 100, 100);
        c.set_point(0, 0, 42.0).unwrap();
        c.advance(0.016);
        assert_eq!(c.series[0].shown[0], 42.0);
    }

    #[test]
    fn unbekannte_eigenschaft_nennt_die_gueltigen() {
        let mut c = ChartObj::new(Kind::Pie, 0, 0, 100, 100);
        let e = c.set_num("innen_radius", 0.5).unwrap_err();
        assert!(e.contains("innen_radius"), "{}", e);
        assert!(e.contains("innenradius"), "Fehler muss den richtigen Namen zeigen: {}", e);
    }

    #[test]
    fn set_str_prueft_die_erlaubten_werte() {
        let mut c = ChartObj::new(Kind::Bar, 0, 0, 100, 100);
        assert!(c.set_str("legende", "oben").is_ok());
        assert!(c.set_str("legende", "schraeg").is_err());
        assert!(c.set_str("ausrichtung", "waagerecht").is_ok());
    }

    #[test]
    fn nice_step_liefert_runde_schritte() {
        assert_eq!(nice_step(100.0, 5), 20.0);
        assert_eq!(nice_step(10.0, 5), 2.0);
        assert_eq!(nice_step(1.0, 5), 0.2);
        assert_eq!(nice_step(0.0, 5), 1.0, "Spanne 0 darf nicht endlos schleifen");
    }

    #[test]
    fn theme_setzt_alle_farbrollen() {
        let mut s = Style::default();
        assert!(apply_theme(&mut s, "neon"));
        assert!(!apply_theme(&mut s, "gibtsnicht"));
        assert!(!s.palette.is_empty());
        // Kein Thema darf eine Rolle auf dem Wert eines anderen Themas stehen lassen.
        let mut hell = Style::default();
        apply_theme(&mut hell, "hell");
        assert_ne!(hell.c_hintergrund, s.c_hintergrund);
    }

    #[test]
    fn tacho_hat_von_anfang_an_einen_wert() {
        let c = ChartObj::new(Kind::Gauge, 0, 0, 100, 100);
        assert_eq!(c.series.len(), 1, "CHART_VALUE muss ohne CHART_SERIES gehen");
        assert_eq!(c.series[0].values.len(), 1);
    }

    #[test]
    fn fmt_value_haengt_die_einheit_an() {
        let mut c = ChartObj::new(Kind::Gauge, 0, 0, 100, 100);
        c.set_str("einheit", " km/h").unwrap();
        c.set_num("nachkomma", 1.0).unwrap();
        assert_eq!(c.fmt_value(42.25), "42.2 km/h");
        c.set_flag("kurz", true).unwrap();
        c.set_num("nachkomma", 2.0).unwrap();
        assert_eq!(c.fmt_value(1_500_000.0), "1.50M km/h");
    }

    #[test]
    fn ohne_animation_zeichnet_der_wert_auch_ohne_chart_update() {
        // Der Fallstrick: gezeichnet wurden frueher immer die nachziehenden
        // `shown`-Werte, die NUR CHART_UPDATE fortschreibt. Wer keine
        // Animation bestellt hatte, sah darum ein Diagramm, das stumm auf
        // Null stehenblieb. Ohne Animation muessen die echten Werte gelten.
        let mut c = ChartObj::new(Kind::Gauge, 0, 0, 100, 100);
        c.set_num("max", 8000.0).unwrap();
        c.set_point(0, 0, 5800.0).unwrap();
        // KEIN advance() -- genau wie ein Programm ohne CHART_UPDATE.
        assert_eq!(c.anzeige(&c.series[0])[0], 5800.0);

        // Mit Animation gilt weiterhin der nachziehende Wert.
        c.set_num("animation", 0.5).unwrap();
        assert_eq!(c.anzeige(&c.series[0])[0], 0.0, "Animation muss nachziehen");
        c.advance(10.0);
        assert!((c.anzeige(&c.series[0])[0] - 5800.0).abs() < 1e-6);
    }

    #[test]
    fn range_sieht_die_werte_auch_ohne_chart_update() {
        // Gleiche Falle eine Ebene tiefer: die Achse skalierte auf 0..1,
        // weil sie nur die `shown` sah.
        let mut c = ChartObj::new(Kind::Bar, 0, 0, 100, 100);
        c.add_series("s", -1);
        c.add_point("a", 0.0, -1);
        c.set_point(0, 0, 42.0).unwrap();
        let (_, hi) = c.range();
        assert_eq!(hi, 42.0, "Achse skalierte an den echten Daten vorbei");
    }

    #[test]
    fn with_alpha_behandelt_die_null_als_deckend() {
        // Eine 24-Bit-Farbe hat Alpha-Byte 0 -- das heisst in gbrt DECKEND.
        // Wer daraus naiv rechnet, macht aus "voll sichtbar" versehentlich
        // "unsichtbar"; halbe Deckkraft muss also 0x80 ergeben, nicht 0x00.
        assert_eq!(with_alpha(0xFF8800, 0.5), 0x80FF8800u32 as i64);
        // Volle Deckkraft laesst die Farbe unangetastet (auch das Alpha-Byte).
        assert_eq!(with_alpha(0xFF8800, 1.0), 0xFF8800);
        // Ein bereits halbdurchsichtiger Wert wird weiter abgesenkt, nicht ersetzt.
        assert_eq!(alpha_of(with_alpha(0x80FF8800u32 as i64, 0.5)), 0x40);
        // 0 als Deckkraft darf nicht auf das Alpha-Byte 0 fallen -- das waere
        // wieder "deckend". Untere Grenze ist 1.
        assert_eq!(alpha_of(with_alpha(0xFF8800, 0.0)), 1);
    }

    #[test]
    fn leiste_und_lampen_liegen_waagerecht() {
        // Regression: `ausrichtung` steht per Vorgabe auf "senkrecht" --
        // richtig fuer Balkendiagramme (Balken stehen), aber eine Leiste
        // liegt. Ohne die Ausnahme baute sie ungefragt hochkant.
        for art in [Kind::Leiste, Kind::Led] {
            let c = ChartObj::new(art, 0, 0, 200, 60);
            assert_eq!(c.style.ausrichtung, "waagerecht", "{:?}", art);
        }
        // Balken bleiben senkrecht.
        assert_eq!(ChartObj::new(Kind::Bar, 0, 0, 100, 100).style.ausrichtung, "senkrecht");
    }

    #[test]
    fn einwertige_arten_bringen_ihre_reihe_mit() {
        for art in [Kind::Gauge, Kind::Leiste, Kind::Led] {
            let c = ChartObj::new(art, 0, 0, 100, 100);
            assert_eq!(c.series.len(), 1, "{:?}", art);
            assert_eq!(c.series[0].values.len(), 1, "{:?}: CHART_VALUE muss sofort gehen", art);
        }
    }

    #[test]
    fn skala_farbe_verlaeuft_gerichtet_statt_kategorial() {
        // Die Palette ist kategorial (acht gut unterscheidbare Farben fuer
        // acht Reihen); interpoliert ergibt sie einen Regenbogen. Eine Skala
        // braucht einen gerichteten Verlauf -- daher eine eigene Farbrolle.
        let c = ChartObj::new(Kind::Leiste, 0, 0, 200, 60);
        let unten = c.skala_farbe_test(0.0, 0.0, 100.0);
        let oben = c.skala_farbe_test(1.0, 0.0, 100.0);
        assert_eq!(unten, c.style.c_skala_von);
        assert_eq!(oben, c.style.c_skala_bis);
        // Rot unten, Gruen oben: die Reihenfolge der Kanaele kippt.
        assert!((unten >> 16) & 0xFF > (unten >> 8) & 0xFF, "unten nicht rot");
        assert!((oben >> 8) & 0xFF > (oben >> 16) & 0xFF, "oben nicht gruen");
        // Die Mitte liegt dazwischen, nicht auf einem der Enden.
        let mitte = c.skala_farbe_test(0.5, 0.0, 100.0);
        assert_eq!(mitte, c.style.c_skala_mitte);
    }

    #[test]
    fn skala_farbe_folgt_den_zonen_wenn_es_welche_gibt() {
        let mut c = ChartObj::new(Kind::Led, 0, 0, 200, 60);
        c.zones.push(Zone { from: 0.0, to: 50.0, color: 0x112233, name: String::new() });
        c.zones.push(Zone { from: 50.0, to: 100.0, color: 0x445566, name: String::new() });
        assert_eq!(c.skala_farbe_test(0.1, 0.0, 100.0), 0x112233);
        assert_eq!(c.skala_farbe_test(0.9, 0.0, 100.0), 0x445566);
    }

    #[test]
    fn jedes_thema_bringt_einen_skalenverlauf_mit() {
        for name in THEMES {
            let mut s = Style::default();
            assert!(apply_theme(&mut s, name));
            assert_ne!(s.c_skala_von, s.c_skala_bis, "Thema {} hat keinen Verlauf", name);
        }
    }

    #[test]
    fn zifferblatt_und_wertanzeige_pruefen_ihre_werte() {
        let mut c = ChartObj::new(Kind::Gauge, 0, 0, 100, 100);
        for b in ["ring", "segmente", "striche", "baender"] {
            assert!(c.set_str("zifferblatt", b).is_ok(), "{}", b);
        }
        for w in ["aus", "innen", "pille", "blase", "am_zeiger"] {
            assert!(c.set_str("wertanzeige", w).is_ok(), "{}", w);
        }
        let e = c.set_str("zifferblatt", "kringel").unwrap_err();
        assert!(e.contains("kringel") && e.contains("segmente"), "{}", e);
    }

    #[test]
    fn tacho_zeigt_seinen_wert_ohne_zutun() {
        // Regression: die Wertanzeige haing zusaetzlich am Schalter `werte`,
        // der per Vorgabe "aus" ist -- der Tacho blieb dadurch stumm, obwohl
        // `wertanzeige` auf "innen" stand. Zwei Schalter fuer dieselbe Sache.
        let c = ChartObj::new(Kind::Gauge, 0, 0, 100, 100);
        assert_eq!(c.style.wertanzeige, "innen");
        assert_eq!(c.style.werte, "aus", "`werte` darf den Tacho nicht mitschalten");
    }

    #[test]
    fn zonen_koennen_beschriftet_werden() {
        let mut c = ChartObj::new(Kind::Gauge, 0, 0, 100, 100);
        c.zones.push(Zone { from: 0.0, to: 50.0, color: 0xFF0000, name: "SCHLECHT".into() });
        c.zones.push(Zone { from: 50.0, to: 100.0, color: 0x00FF00, name: String::new() });
        assert_eq!(c.zones[0].name, "SCHLECHT");
        assert!(c.zones[1].name.is_empty(), "ohne Namen bleibt sie unbeschriftet");
    }

    #[test]
    fn kuchen_stuecke_summieren_sich_auf_360() {
        let mut c = ChartObj::new(Kind::Pie, 0, 0, 100, 100);
        c.add_point("a", 45.0, -1);
        c.add_point("b", 30.0, -1);
        c.add_point("c", 25.0, -1);
        let s = c.kuchen_stuecke();
        assert_eq!(s.len(), 3);
        assert!((s[0].0 - (-90.0)).abs() < 1e-9, "startet nicht bei 12 Uhr");
        let summe: f64 = s.iter().map(|(_, sp)| sp).sum();
        assert!((summe - 360.0).abs() < 1e-9, "Summe {}", summe);
        // Luecken darf es nicht geben: jedes Stueck beginnt, wo das vorige endet.
        for i in 1..s.len() {
            assert!((s[i].0 - (s[i - 1].0 + s[i - 1].1)).abs() < 1e-9);
        }
    }

    #[test]
    fn stueck_bei_winkel_trifft_rundherum() {
        // 4 gleiche Viertel ab 12 Uhr im Uhrzeigersinn.
        let s = vec![(-90.0, 90.0), (0.0, 90.0), (90.0, 90.0), (180.0, 90.0)];
        assert_eq!(stueck_bei_winkel(&s, -45.0), 0);   // oben rechts
        assert_eq!(stueck_bei_winkel(&s, 45.0), 1);    // unten rechts
        assert_eq!(stueck_bei_winkel(&s, 135.0), 2);   // unten links
        // atan2 liefert fuer "oben links" -135 -- ohne Umrechnung faende man
        // dort nichts, obwohl das vierte Viertel genau da liegt.
        assert_eq!(stueck_bei_winkel(&s, -135.0), 3);
        assert_eq!(stueck_bei_winkel(&s, 225.0), 3, "gleicher Punkt, andere Schreibweise");
    }

    #[test]
    fn stueck_bei_winkel_ueberspringt_leere_segmente() {
        let s = vec![(-90.0, 0.0), (-90.0, 360.0)];
        assert_eq!(stueck_bei_winkel(&s, 0.0), 1, "Nullsegment darf nicht treffen");
        assert_eq!(stueck_bei_winkel(&[], 0.0), -1);
    }

    #[test]
    fn hover_getter_sind_ohne_zeichnen_leer() {
        let c = ChartObj::new(Kind::Pie, 0, 0, 100, 100);
        assert_eq!(c.hover, -1);
        assert_eq!(c.hover_label(), "");
        assert_eq!(c.hover_value(), 0.0);
        assert_eq!(c.geklickt, -1);
    }

    #[test]
    fn hervorhebung_verschiebt_den_farbton_nicht() {
        // Regression: `scale_rgb` machte aus hervorgehobenem Orange ein Gelb,
        // weil Rot schon bei 255 klemmte und nur Gruen mitwuchs. Das sah aus
        // wie ein ANDERER Eintrag der Palette. Gegen Weiss mischen haelt die
        // Reihenfolge der Kanaele -- und damit den Farbton -- ein.
        let orange = 0xFFA500;
        let mut c = ChartObj::new(Kind::Pie, 0, 0, 100, 100);
        c.set_num("hover_glanz", 0.35).unwrap();
        let hell = c.hell_test(orange, 1.0);
        let (r, gr, b) = ((hell >> 16) & 0xFF, (hell >> 8) & 0xFF, hell & 0xFF);
        assert!(r > gr && gr > b, "Farbton gekippt: #{:06X}", hell);
        assert!(hell != orange, "gar nicht aufgehellt");
        // Zum Vergleich: das alte Verfahren kippte die Reihenfolge.
        let alt = scale_rgb(orange, 1.35);
        assert_eq!((alt >> 16) & 0xFF, 255);
        assert!((alt >> 8) & 0xFF > 0xA5, "alte Fassung hellte Gruen staerker auf");
    }

    #[test]
    fn mix_rgb_haelt_das_alpha_der_ersten_farbe() {
        assert_eq!(mix_rgb(0x000000, 0xFFFFFF, 0.5), 0x808080);
        assert_eq!(mix_rgb(0x80000000u32 as i64, 0xFFFFFF, 0.0), 0x80000000u32 as i64);
        // Alpha von b wird NICHT uebernommen.
        assert_eq!(mix_rgb(0x40FF0000u32 as i64, 0xFF00FF00u32 as i64, 1.0), 0x4000FF00u32 as i64);
    }

    #[test]
    fn scale_rgb_dunkelt_ab_und_laesst_alpha_stehen() {
        assert_eq!(scale_rgb(0x808080, 0.5), 0x404040);
        // Alpha bleibt erhalten ...
        assert_eq!(scale_rgb(0x80808080u32 as i64, 0.5), 0x80404040u32 as i64);
        // ... und kein Kanal laeuft beim Aufhellen ueber.
        assert_eq!(scale_rgb(0xC0C0C0, 4.0), 0xFFFFFF);
    }

    #[test]
    fn verlauf_ende_leitet_sich_aus_der_farbe_ab() {
        let mut c = ChartObj::new(Kind::Bar, 0, 0, 100, 100);
        // Ohne feste Zielfarbe: dunklere Fassung der Ausgangsfarbe.
        assert_eq!(c.style.c_verlauf_ende, -1);
        assert_eq!(scale_rgb(0x808080, 0.45), 0x3A3A3A);
        // Mit fester Zielfarbe gewinnt diese.
        c.set_color("verlauf_ende", 0x123456).unwrap();
        assert_eq!(c.style.c_verlauf_ende, 0x123456);
    }

    #[test]
    fn jedes_thema_setzt_eine_schattenfarbe_mit_alpha() {
        for name in THEMES {
            let mut s = Style::default();
            assert!(apply_theme(&mut s, name));
            let a = alpha_of(s.c_schatten);
            assert!(a < 255, "Thema {} wirft einen voll deckenden Schatten (Alpha {})", name, a);
            assert!(a > 0, "Thema {}: Schatten unsichtbar", name);
        }
    }

    #[test]
    fn neue_stellschrauben_stehen_in_den_schluesseltabellen() {
        // Sonst kennt der Setter sie, aber die Fehlermeldung verschweigt sie.
        let mut c = ChartObj::new(Kind::Bar, 0, 0, 100, 100);
        for k in ["schatten_weich", "deckkraft", "flaeche_deckkraft"] {
            assert!(c.set_num(k, 0.5).is_ok(), "{} nicht setzbar", k);
            assert!(KEYS_NUM.contains(&k), "{} fehlt in KEYS_NUM", k);
        }
        for k in ["schatten", "verlauf_ende"] {
            assert!(c.set_color(k, 0x112233).is_ok(), "{} nicht setzbar", k);
            assert!(KEYS_COLOR.contains(&k), "{} fehlt in KEYS_COLOR", k);
        }
        for k in ["verlauf_daten", "schatten_daten"] {
            assert!(c.set_flag(k, true).is_ok(), "{} nicht setzbar", k);
            assert!(KEYS_FLAG.contains(&k), "{} fehlt in KEYS_FLAG", k);
        }
    }

    #[test]
    fn deckkraft_wird_geklemmt() {
        let mut c = ChartObj::new(Kind::Bar, 0, 0, 100, 100);
        c.set_num("deckkraft", 5.0).unwrap();
        assert_eq!(c.style.deckkraft, 1.0);
        c.set_num("deckkraft", -1.0).unwrap();
        assert_eq!(c.style.deckkraft, 0.0);
    }

    #[test]
    fn stats_ueber_eine_reihe() {
        let s = stats(&[1.0, 2.0, 3.0, f64::NAN]);
        assert_eq!(s["anzahl"], 3.0, "NAN darf nicht mitzaehlen");
        assert_eq!(s["summe"], 6.0);
        assert_eq!(s["mittel"], 2.0);
        assert_eq!(s["min"], 1.0);
        assert_eq!(s["max"], 3.0);
        let leer = stats(&[]);
        assert_eq!(leer["anzahl"], 0.0);
        assert_eq!(leer["mittel"], 0.0, "leere Reihe darf nicht NaN liefern");
    }
}
