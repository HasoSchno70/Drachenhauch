//! Volle Textformung (gui-Luecke Punkt 2): Schriften, deren Glyphen erst
//! eine Formung ergibt -- indische und suedostasiatische Schriften
//! (Devanagari, Bengali, Tamil, Thai, Khmer, Myanmar, Tibetisch ...) mit
//! Konjunkten, vorgestellten Vokalzeichen und gestapelten Zeichen -- und auf
//! Wunsch die Ligaturen einer Schrift (`FONT_LIGATURES`).
//!
//! raylib zeichnet Glyphe fuer Glyphe nach Codepunkt aus einem Atlas; die
//! Glyphen einer Formung haben aber oft gar keinen Codepunkt (ein Konjunkt
//! ist eine eigene Glyphe der Schrift). Darum ein zweiter Weg nach
//! Glyphennummer: `rustybuzz` formt, `ttf-parser` liefert die Umrisse, ein
//! kleiner Flaechen-Rasterer (Kantenglaettung ueber aufsummierte
//! Flaechenanteile) macht daraus Glyphenbilder, die `graphics.rs` als
//! Texturen zwischenspeichert. Messen und Zeichnen gehen durch dieselbe
//! Formung (`formen`), damit Schreibmarke und Text nicht auseinanderlaufen.
//!
//! Ein Lauf ohne solche Zeichen (und ohne eingeschaltete Ligaturen) geht wie
//! bisher durch raylib -- kein Umweg fuer die gewoehnlichen Faelle.
//! Arabisch formt `schreibrichtung.rs` weiter ueber Darstellungsformen.

use std::cell::RefCell;
use std::collections::HashMap;
// ttf-parser ueber rustybuzz: so baut das Modul auch mit dem Feature pdf
// allein (die Rust-Tests der CI laufen ohne Grafik).
use rustybuzz::ttf_parser;

/// Braucht dieser Text die Formung? (Zeichen aus einer Schrift, die ohne
/// sie falsch dastuende.)
pub fn braucht(s: &str) -> bool {
    !s.is_ascii() && s.chars().any(komplex)
}

fn komplex(c: char) -> bool {
    matches!(c as u32,
        0x0900..=0x0DFF      // Devanagari bis Sinhala
        | 0x0E00..=0x0EFF    // Thai, Lao
        | 0x0F00..=0x0FFF    // Tibetisch
        | 0x1000..=0x109F    // Myanmar
        | 0x1780..=0x17FF    // Khmer
        | 0x1A20..=0x1AAF    // Tai Tham
        | 0x1B00..=0x1B7F    // Balinesisch
        | 0xA8E0..=0xA8FF    // Devanagari erweitert
        | 0xA9E0..=0xA9FF    // Myanmar erweitert B
        | 0xAA60..=0xAA7F    // Myanmar erweitert A
        | 0x11000..=0x1137F) // Brahmi bis Grantha
}

/// Die Schrift eines Handles (LOADFONT) und ob ihre Ligaturen an sind.
#[derive(Clone, Default)]
struct Eintrag { pfad: String, ligaturen: bool }

/// Eine geladene Schrift: die Datei (fuer immer im Speicher -- Schriften
/// sind wenige), ihr Gesicht fuer die Formung und die Masse.
struct Gesicht {
    face: rustybuzz::Face<'static>,
    /// Ober- bis Unterlaenge in Einheiten der Schrift: raylib setzt eine
    /// Schrift so, dass diese Hoehe der Groesse entspricht
    /// (stbtt_ScaleForPixelHeight) -- dieselbe Rechnung hier, sonst stuende
    /// geformter Text groesser oder kleiner neben dem anderen.
    em: f32,
    oben: f32,
}

thread_local! {
    static HANDLES: RefCell<HashMap<i64, Eintrag>> = RefCell::new(HashMap::new());
    static GESICHTER: RefCell<HashMap<String, Option<&'static Gesicht>>> = RefCell::new(HashMap::new());
    static GEFORMT: RefCell<HashMap<(String, String, u32), std::rc::Rc<Geformt>>> = RefCell::new(HashMap::new());
}

/// LOADFONT hat Handle `h` aus `pfad` geladen (eine Datei oder eine
/// eingebaute Schrift).
pub fn schrift_merken(h: i64, pfad: &str) {
    HANDLES.with(|m| { m.borrow_mut().insert(h, Eintrag { pfad: pfad.to_string(), ligaturen: false }); });
}

/// FONT_LIGATURES(font, an).
pub fn ligaturen_setzen(h: i64, an: bool) -> bool {
    HANDLES.with(|m| match m.borrow_mut().get_mut(&h) { Some(e) => { e.ligaturen = an; true } None => false })
}

/// Sind die Ligaturen der Schrift `h` an?
pub fn ligaturen(h: i64) -> bool {
    HANDLES.with(|m| m.borrow().get(&h).map(|e| e.ligaturen).unwrap_or(false))
}

/// Die Datei hinter Handle `h` (None: die eingebaute Pixelschrift oder
/// unbekannt).
pub fn pfad_von(h: i64) -> Option<String> {
    HANDLES.with(|m| m.borrow().get(&h).map(|e| e.pfad.clone()))
}

/// Fuer farbglyphe.rs: das Gesicht einer Schrift (ttf-parser), ihre Hoehe
/// von Ober- bis Unterlaenge und die Oberlaenge (Schrift-Einheiten).
pub(crate) fn gesicht_info(pfad: &str) -> Option<(&'static ttf_parser::Face<'static>, f32, f32)> {
    let g = gesicht(pfad)?;
    Some((&g.face, g.em, g.oben))
}

fn gesicht(pfad: &str) -> Option<&'static Gesicht> {
    if let Some(g) = GESICHTER.with(|m| m.borrow().get(pfad).copied()) { return g; }
    let g = laden(pfad);
    GESICHTER.with(|m| { m.borrow_mut().insert(pfad.to_string(), g); });
    g
}

fn laden(pfad: &str) -> Option<&'static Gesicht> {
    // "datei.ttc#2": die dritte Schrift einer Sammlung.
    let (datei, index) = match pfad.rsplit_once('#') {
        Some((d, i)) if i.chars().all(|c| c.is_ascii_digit()) && !i.is_empty() => (d, i.parse().unwrap_or(0)),
        _ => (pfad, 0u32),
    };
    #[cfg(feature = "graphics")]
    if let Some(name) = datei.strip_prefix(crate::graphics::EINGEBAUT_VORSATZ) {
        return gesicht_aus(crate::graphics::eingebaute_schrift(name)?, index);
    }
    gesicht_aus(Box::leak(std::fs::read(datei).ok()?.into_boxed_slice()), index)
}

fn gesicht_aus(bytes: &'static [u8], index: u32) -> Option<&'static Gesicht> {
    let face = rustybuzz::Face::from_slice(bytes, index)?;
    let em = (face.ascender() as f32 - face.descender() as f32).max(1.0);
    if em <= 0.0 { return None; }
    let oben = face.ascender() as f32;
    Some(Box::leak(Box::new(Gesicht { face, em, oben })))
}

/// Ein geformter Lauf: Glyphen mit Lage (Pixel ab dem Anfang des Laufs, y
/// nach unten) und die ganze Vorschubbreite.
pub struct Geformt {
    pub glyphen: Vec<(u16, f32, f32)>,
    pub breite: f32,
    /// Abstand vom oberen Rand der Zeile bis zur Grundlinie.
    pub grundlinie: f32,
}

/// `text` mit der Schrift aus `pfad` in `px` Pixeln formen. None, wenn die
/// Datei sich nicht als Schrift lesen laesst.
pub fn formen(pfad: &str, text: &str, px: f32) -> Option<std::rc::Rc<Geformt>> {
    let schluessel = (pfad.to_string(), text.to_string(), (px * 16.0).round() as u32);
    if let Some(g) = GEFORMT.with(|m| m.borrow().get(&schluessel).cloned()) { return Some(g); }
    let g = gesicht(pfad)?;
    let skala = px / g.em;
    let mut puffer = rustybuzz::UnicodeBuffer::new();
    puffer.push_str(text);
    puffer.guess_segment_properties();
    let erg = rustybuzz::shape(&g.face, &[], puffer);
    let mut x = 0.0f32;
    let mut glyphen = Vec::with_capacity(erg.len());
    for (info, pos) in erg.glyph_infos().iter().zip(erg.glyph_positions()) {
        glyphen.push((info.glyph_id as u16, x + pos.x_offset as f32 * skala, -(pos.y_offset as f32) * skala));
        x += pos.x_advance as f32 * skala;
    }
    let geformt = std::rc::Rc::new(Geformt { glyphen, breite: x, grundlinie: g.oben * skala });
    GEFORMT.with(|m| {
        let mut m = m.borrow_mut();
        // Ein Zwischenspeicher, kein Gedaechtnis: wer laufend neue Texte
        // zeichnet (eine Uhr), soll ihn nicht ins Unendliche fuellen.
        if m.len() > 4096 { m.clear(); }
        m.insert(schluessel, geformt.clone());
    });
    Some(geformt)
}

/// Ein gerastertes Glyphenbild: Deckkraft je Pixel (Zeile fuer Zeile),
/// und wo es relativ zum Stift steht (links: ab der Stiftlage nach rechts,
/// oben: ab der Grundlinie nach oben).
pub struct GlyphBild {
    pub b: usize,
    pub h: usize,
    pub deckung: Vec<u8>,
    pub links: f32,
    pub oben: f32,
}

/// Die Glyphe `gid` der Schrift aus `pfad` in `px` Pixeln rastern. None
/// fuer eine leere Glyphe (Leerzeichen) oder eine unlesbare Schrift.
pub fn rastern(pfad: &str, gid: u16, px: f32) -> Option<GlyphBild> {
    let g = gesicht(pfad)?;
    let skala = px / g.em;
    let bbox = g.face.glyph_bounding_box(ttf_parser::GlyphId(gid))?;
    let rand = 1.0f32;
    let x0 = (bbox.x_min as f32 * skala).floor() - rand;
    let y1 = (bbox.y_max as f32 * skala).ceil() + rand;
    let b = ((bbox.x_max as f32 * skala).ceil() + rand - x0).max(1.0) as usize;
    let h = (y1 - (bbox.y_min as f32 * skala).floor() + rand).max(1.0) as usize;
    if b > 4096 || h > 4096 { return None; }
    let mut r = Raster::neu(b, h);
    {
        let mut zeichner = Umriss { r: &mut r, skala, x0, y1, start: (0.0, 0.0), stift: (0.0, 0.0) };
        g.face.outline_glyph(ttf_parser::GlyphId(gid), &mut zeichner)?;
        zeichner.schliessen();
    }
    Some(GlyphBild { b, h, deckung: r.ergebnis(), links: x0, oben: y1 })
}

/// Ein Umriss aus ttf-parser, auf Pixel gebracht und in Linien zerlegt.
struct Umriss<'a> {
    r: &'a mut Raster,
    skala: f32,
    x0: f32,
    y1: f32,
    start: (f32, f32),
    stift: (f32, f32),
}

impl Umriss<'_> {
    fn p(&self, x: f32, y: f32) -> (f32, f32) { (x * self.skala - self.x0, self.y1 - y * self.skala) }
    fn schliessen(&mut self) {
        if self.stift != self.start { self.r.linie(self.stift, self.start); }
        self.stift = self.start;
    }
}

impl ttf_parser::OutlineBuilder for Umriss<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.schliessen();
        let p = self.p(x, y);
        self.start = p;
        self.stift = p;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.r.linie(self.stift, p);
        self.stift = p;
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (a, b, c) = (self.stift, self.p(x1, y1), self.p(x, y));
        let n = schritte(a, c, b, b);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let q = (u * u * a.0 + 2.0 * u * t * b.0 + t * t * c.0, u * u * a.1 + 2.0 * u * t * b.1 + t * t * c.1);
            self.r.linie(self.stift, q);
            self.stift = q;
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (a, b, c, d) = (self.stift, self.p(x1, y1), self.p(x2, y2), self.p(x, y));
        let n = schritte(a, d, b, c);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let k = |p0: f32, p1: f32, p2: f32, p3: f32| u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3;
            let q = (k(a.0, b.0, c.0, d.0), k(a.1, b.1, c.1, d.1));
            self.r.linie(self.stift, q);
            self.stift = q;
        }
    }
    fn close(&mut self) { self.schliessen(); }
}

/// Wie fein eine Kurve zerlegt wird: nach ihrer Laenge, mindestens zwei.
fn schritte(a: (f32, f32), d: (f32, f32), b: (f32, f32), c: (f32, f32)) -> usize {
    let l = |p: (f32, f32), q: (f32, f32)| ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2)).sqrt();
    let laenge = l(a, b) + l(b, c) + l(c, d);
    ((laenge / 2.0).ceil() as usize).clamp(2, 64)
}

/// Flaechen-Rasterer: jede Linie traegt ihren Flaechenanteil je Pixel ein,
/// eine laufende Summe ueber die Zeile ergibt die Deckung (nonzero).
pub(crate) struct Raster { b: usize, h: usize, a: Vec<f32> }

impl Raster {
    pub(crate) fn neu(b: usize, h: usize) -> Self { Raster { b, h, a: vec![0.0; b * h + 4] } }

    /// Die Deckung je Pixel als 0..1 (fuer farbglyphe.rs).
    pub(crate) fn deckung(&self) -> Vec<f32> {
        let mut summe = 0.0f32;
        (0..self.b * self.h).map(|i| { summe += self.a[i]; summe.abs().min(1.0) }).collect()
    }

    fn add(&mut self, i: isize, v: f32) {
        if i >= 0 { if let Some(x) = self.a.get_mut(i as usize) { *x += v; } }
    }

    pub(crate) fn linie(&mut self, p0: (f32, f32), p1: (f32, f32)) {
        if (p0.1 - p1.1).abs() < 1e-6 { return; }
        let (dir, p0, p1) = if p0.1 < p1.1 { (1.0f32, p0, p1) } else { (-1.0, p1, p0) };
        let dxdy = (p1.0 - p0.0) / (p1.1 - p0.1);
        let mut x = p0.0;
        let y0 = p0.1.max(0.0);
        if p0.1 < 0.0 { x -= p0.1 * dxdy; }
        let bis = (p1.1.ceil() as usize).min(self.h);
        let w = self.b as isize;
        for y in (y0 as usize)..bis {
            let zeile = (y * self.b) as isize;
            let dy = ((y + 1) as f32).min(p1.1) - (y as f32).max(p0.1);
            let xnext = x + dxdy * dy;
            let d = dy * dir;
            let (xa, xb) = if x < xnext { (x, xnext) } else { (xnext, x) };
            let xa = xa.max(0.0);
            let xb = xb.min(w as f32);
            let xaf = xa.floor();
            let xai = xaf as isize;
            let xbc = xb.ceil();
            let xbi = xbc as isize;
            if xbi <= xai + 1 {
                let xmf = 0.5 * (x + xnext) - xaf;
                self.add(zeile + xai, d - d * xmf);
                self.add(zeile + xai + 1, d * xmf);
            } else {
                let s = 1.0 / (xb - xa);
                let x0f = xa - xaf;
                let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
                let x1f = xb - xbc + 1.0;
                let am = 0.5 * s * x1f * x1f;
                self.add(zeile + xai, d * a0);
                if xbi == xai + 2 {
                    self.add(zeile + xai + 1, d * (1.0 - a0 - am));
                } else {
                    let a1 = s * (1.5 - x0f);
                    self.add(zeile + xai + 1, d * (a1 - a0));
                    for xi in (xai + 2)..(xbi - 1) { self.add(zeile + xi, d * s); }
                    let a2 = a1 + (xbi - xai - 3) as f32 * s;
                    self.add(zeile + xbi - 1, d * (1.0 - a2 - am));
                }
                self.add(zeile + xbi, d * am);
            }
            x = xnext;
        }
    }

    fn ergebnis(&self) -> Vec<u8> {
        let mut summe = 0.0f32;
        let mut out = vec![0u8; self.b * self.h];
        for (i, o) in out.iter_mut().enumerate() {
            summe += self.a[i];
            *o = (summe.abs().min(1.0) * 255.0 + 0.5) as u8;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braucht_nur_komplexe_schriften() {
        assert!(!braucht("Hallo Welt"));
        assert!(!braucht("Grüße, ¿qué tal?"));
        assert!(!braucht("مرحبا"));            // Arabisch: schreibrichtung.rs
        assert!(braucht("नमस्ते"));
        assert!(braucht("ok สวัสดี"));
        assert!(braucht("ខ្មែរ"));
    }

    #[test]
    fn ein_quadrat_deckt_genau_seine_flaeche() {
        // Ein Quadrat von (2,2) bis (6,6) in einem 8x8-Raster: innen voll,
        // aussen leer, die Summe ist seine Flaeche.
        let mut r = Raster::neu(8, 8);
        for (a, b) in [((2.0, 2.0), (6.0, 2.0)), ((6.0, 2.0), (6.0, 6.0)), ((6.0, 6.0), (2.0, 6.0)), ((2.0, 6.0), (2.0, 2.0))] {
            r.linie(a, b);
        }
        let d = r.ergebnis();
        assert_eq!(d[3 * 8 + 3], 255);
        assert_eq!(d[0], 0);
        assert_eq!(d[7 * 8 + 7], 0);
        let flaeche: f32 = d.iter().map(|&v| v as f32 / 255.0).sum();
        assert!((flaeche - 16.0).abs() < 0.1, "{}", flaeche);
    }

    #[test]
    fn halbe_pixel_halb_gedeckt() {
        // Ein Rechteck von x 1.5 bis 3.5: die Randpixel halb.
        let mut r = Raster::neu(5, 2);
        for (a, b) in [((1.5, 0.0), (3.5, 0.0)), ((3.5, 0.0), (3.5, 2.0)), ((3.5, 2.0), (1.5, 2.0)), ((1.5, 2.0), (1.5, 0.0))] {
            r.linie(a, b);
        }
        let d = r.ergebnis();
        assert!((d[1] as i32 - 128).abs() <= 1, "{:?}", d);
        assert_eq!(d[2], 255);
        assert!((d[3] as i32 - 128).abs() <= 1, "{:?}", d);
    }
}
