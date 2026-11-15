//! Farbige Glyphen: Emoji in Farbe statt als einfarbige Silhouette.
//!
//! Die gui (und TEXT) setzen Emoji ueber die Ausweich-Schriften
//! (graphics.rs `ausweich_datei_fuer`) und die Formung (formung.rs) -- so
//! stimmen Messen und Zeichnen, und Folgen mit Verbinder (ZWJ) und Hautton
//! setzt rustybuzz zu EINER Glyphe zusammen. Bis 2026.49 wurde jede Glyphe
//! als Deckungsmaske gerastert und in der Textfarbe getoent; ein Emoji kam
//! einfarbig. Hier entsteht fuer eine Farbglyphe ein RGBA-Bild:
//!
//! * **COLR** (Segoe UI Emoji unter Windows; v0 Lagen in Farben der CPAL,
//!   v1 dazu Verlaeufe, Transformationen, Ausschnitte und Ebenen): ein
//!   `Painter` fuer ttf-parser, der mit dem Rasterer der Formung zeichnet.
//! * **Bitmaps** (sbix: Apple Color Emoji, CBDT: Noto Color Emoji): das
//!   PNG der Schrift; entpackt und skaliert wird in graphics.rs.
//!
//! Rein (ohne raylib) und darum mit Rust-Tests pruefbar.

use crate::formung::{gesicht_info, Raster};
use ttf_parser::colr::{CompositeMode, GradientExtend, Paint, Painter};
use ttf_parser::{GlyphId, RgbaColor, Transform};

/// Ein farbiges Glyphenbild: RGBA (nicht vormultipliziert), Zeile fuer
/// Zeile; Lage wie `formung::GlyphBild` (links ab Stift, oben ab Grundlinie).
pub struct FarbBild {
    pub b: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
    pub links: f32,
    pub oben: f32,
}

/// Was eine Farbglyphe ist.
pub enum Farbglyphe {
    Bild(FarbBild),
    /// PNG aus sbix/CBDT: Daten, Versatz (Pixel der Strike, x ab Stift,
    /// y ab Grundlinie nach oben bis zur Unterkante) und Massstab auf `px`.
    Png { daten: &'static [u8], x: f32, y: f32, skala: f32 },
}

/// Hat die Schrift fuer diese Glyphe Farbe?
pub fn farbig(pfad: &str, gid: u16) -> bool {
    let Some((face, _, _)) = gesicht_info(pfad) else { return false };
    let g = GlyphId(gid);
    face.is_color_glyph(g) || face.glyph_raster_image(g, 64).is_some_and(|b| b.format == ttf_parser::RasterImageFormat::PNG)
}

/// Die Farbglyphe `gid` der Schrift aus `pfad` in `px` Pixeln (dieselbe
/// Groesse wie `formung::rastern`: Ober- bis Unterlaenge = px).
pub fn rastern(pfad: &str, gid: u16, px: f32) -> Option<Farbglyphe> {
    let (face, em, _) = gesicht_info(pfad)?;
    let g = GlyphId(gid);
    let skala = px / em;
    if face.is_color_glyph(g) {
        // Leinwand: Vorschub mal Ober- bis Unterlaenge, mit Rand -- die
        // Lagen eines Emoji koennen etwas ueber das Geviert hinausragen.
        let vorschub = face.glyph_hor_advance(g).unwrap_or(face.units_per_em()) as f32;
        let rand = (px * 0.15).ceil();
        let x0 = -rand;
        let y1 = (face.ascender() as f32 * skala).ceil() + rand;
        let b = ((vorschub * skala).ceil() + 2.0 * rand).max(1.0) as usize;
        let h = (y1 - (face.descender() as f32 * skala).floor() + rand).max(1.0) as usize;
        if b > 2048 || h > 2048 { return None; }
        let basis = Affin { a: skala, b: 0.0, c: 0.0, d: -skala, e: -x0, f: y1 };
        let mut p = Maler::neu(face, b, h, basis);
        face.paint_color_glyph(g, 0, RgbaColor::new(0, 0, 0, 255), &mut p)?;
        let rgba = p.ergebnis();
        return Some(Farbglyphe::Bild(FarbBild { b, h, rgba, links: x0, oben: y1 }));
    }
    let bild = face.glyph_raster_image(g, px.round().max(1.0) as u16)?;
    if bild.format != ttf_parser::RasterImageFormat::PNG { return None; }
    let s = px / bild.pixels_per_em.max(1) as f32 * (face.units_per_em() as f32 / em);
    Some(Farbglyphe::Png { daten: bild.data, x: bild.x as f32, y: bild.y as f32, skala: s })
}

/// Eine affine Abbildung wie in ttf-parser: x' = a x + c y + e, y' = b x + d y + f.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Affin { a: f32, b: f32, c: f32, d: f32, e: f32, f: f32 }

impl Affin {
    fn von(t: Transform) -> Self { Affin { a: t.a, b: t.b, c: t.c, d: t.d, e: t.e, f: t.f } }
    /// Erst `innen`, dann `self`.
    fn nach(self, innen: Affin) -> Affin {
        Affin {
            a: self.a * innen.a + self.c * innen.b,
            b: self.b * innen.a + self.d * innen.b,
            c: self.a * innen.c + self.c * innen.d,
            d: self.b * innen.c + self.d * innen.d,
            e: self.a * innen.e + self.c * innen.f + self.e,
            f: self.b * innen.e + self.d * innen.f + self.f,
        }
    }
    fn an(&self, x: f32, y: f32) -> (f32, f32) { (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f) }
    fn umgekehrt(&self) -> Option<Affin> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-12 { return None; }
        let (a, b, c, d) = (self.d / det, -self.b / det, -self.c / det, self.a / det);
        Some(Affin { a, b, c, d, e: -(a * self.e + c * self.f), f: -(b * self.e + d * self.f) })
    }
}

/// Ein Umriss aus ttf-parser, mit einer Abbildung auf Pixel gebracht.
struct Umriss<'r> { r: &'r mut Raster, m: Affin, start: (f32, f32), stift: (f32, f32) }

impl Umriss<'_> {
    fn schliessen(&mut self) {
        if self.stift != self.start { self.r.linie(self.stift, self.start); }
        self.stift = self.start;
    }
}

impl ttf_parser::OutlineBuilder for Umriss<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.schliessen();
        let p = self.m.an(x, y);
        self.start = p;
        self.stift = p;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.m.an(x, y);
        self.r.linie(self.stift, p);
        self.stift = p;
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (a, b, c) = (self.stift, self.m.an(x1, y1), self.m.an(x, y));
        for i in 1..=12 {
            let t = i as f32 / 12.0;
            let u = 1.0 - t;
            let q = (u * u * a.0 + 2.0 * u * t * b.0 + t * t * c.0, u * u * a.1 + 2.0 * u * t * b.1 + t * t * c.1);
            self.r.linie(self.stift, q);
            self.stift = q;
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (a, b, c, d) = (self.stift, self.m.an(x1, y1), self.m.an(x2, y2), self.m.an(x, y));
        for i in 1..=16 {
            let t = i as f32 / 16.0;
            let u = 1.0 - t;
            let k = |p0: f32, p1: f32, p2: f32, p3: f32| u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3;
            let q = (k(a.0, b.0, c.0, d.0), k(a.1, b.1, c.1, d.1));
            self.r.linie(self.stift, q);
            self.stift = q;
        }
    }
    fn close(&mut self) { self.schliessen(); }
}

/// Der Maler: Abbildungen, Ausschnitte (Deckung 0..1 je Pixel) und Ebenen
/// (vormultipliziertes RGBA, f32).
struct Maler<'f> {
    face: &'f ttf_parser::Face<'f>,
    b: usize,
    h: usize,
    basis: Affin,
    abbildungen: Vec<Affin>,
    ausschnitte: Vec<Vec<f32>>,
    umriss: Option<Vec<f32>>,
    ebenen: Vec<(Vec<f32>, CompositeMode)>,
}

impl<'f> Maler<'f> {
    fn neu(face: &'f ttf_parser::Face<'f>, b: usize, h: usize, basis: Affin) -> Self {
        Maler { face, b, h, basis, abbildungen: Vec::new(), ausschnitte: Vec::new(), umriss: None,
                ebenen: vec![(vec![0.0; b * h * 4], CompositeMode::SourceOver)] }
    }
    /// Schrift-Einheiten -> Pixel, samt aller gestapelten Abbildungen.
    fn jetzt(&self) -> Affin {
        self.abbildungen.iter().fold(self.basis, |m, t| m.nach(*t))
    }
    fn maske_von(&self, malen: impl FnOnce(&mut Umriss)) -> Vec<f32> {
        let mut r = Raster::neu(self.b, self.h);
        {
            let mut u = Umriss { r: &mut r, m: self.jetzt(), start: (0.0, 0.0), stift: (0.0, 0.0) };
            malen(&mut u);
            u.schliessen();
        }
        r.deckung()
    }
    /// Wo darf gemalt werden? Alle Ausschnitte und der offene Umriss.
    fn deckung(&self, i: usize) -> f32 {
        let mut d = self.umriss.as_ref().map(|u| u[i]).unwrap_or(1.0);
        for a in &self.ausschnitte { d = d.min(a[i]); }
        d
    }
    fn ergebnis(mut self) -> Vec<u8> {
        while self.ebenen.len() > 1 { self.pop_layer(); }
        let e = &self.ebenen[0].0;
        let mut out = vec![0u8; self.b * self.h * 4];
        for i in 0..self.b * self.h {
            let a = e[i * 4 + 3];
            if a <= 0.0 { continue; }
            for k in 0..3 { out[i * 4 + k] = ((e[i * 4 + k] / a).clamp(0.0, 1.0) * 255.0 + 0.5) as u8; }
            out[i * 4 + 3] = (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
        out
    }
}

/// Farbe eines Verlaufs bei `t` (Halte nach Lage sortiert, dazwischen linear).
fn farbe_bei(halte: &[(f32, [f32; 4])], t: f32) -> [f32; 4] {
    if halte.is_empty() { return [0.0; 4]; }
    if t <= halte[0].0 { return halte[0].1; }
    for w in halte.windows(2) {
        let ((t0, c0), (t1, c1)) = (w[0], w[1]);
        if t <= t1 {
            let s = if t1 > t0 { (t - t0) / (t1 - t0) } else { 1.0 };
            return [0, 1, 2, 3].map(|k| c0[k] + (c1[k] - c0[k]) * s);
        }
    }
    halte[halte.len() - 1].1
}

fn ausdehnen(t: f32, e: GradientExtend) -> f32 {
    match e {
        GradientExtend::Pad => t.clamp(0.0, 1.0),
        GradientExtend::Repeat => t - t.floor(),
        GradientExtend::Reflect => { let m = t.rem_euclid(2.0); if m > 1.0 { 2.0 - m } else { m } }
    }
}

fn rgba(c: RgbaColor) -> [f32; 4] {
    [c.red as f32 / 255.0, c.green as f32 / 255.0, c.blue as f32 / 255.0, c.alpha as f32 / 255.0]
}

impl<'a, 'f> Painter<'a> for Maler<'f> {
    fn outline_glyph(&mut self, glyph_id: GlyphId) {
        let face = self.face;
        self.umriss = Some(self.maske_von(|u| { face.outline_glyph(glyph_id, u); }));
    }
    fn paint(&mut self, paint: Paint<'a>) {
        // Wer bei einem Pixel welche Farbe hat -- Verlaeufe im Raum der
        // Schrift, also zurueck durch die jetzige Abbildung.
        let zurueck = self.jetzt().umgekehrt();
        let halte = |stops: &mut dyn Iterator<Item = ttf_parser::colr::ColorStop>| -> Vec<(f32, [f32; 4])> {
            let mut v: Vec<(f32, [f32; 4])> = stops.map(|s| (s.stop_offset, rgba(s.color))).collect();
            v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            v
        };
        let farbe: Box<dyn Fn(f32, f32) -> [f32; 4]> = match paint {
            Paint::Solid(c) => { let c = rgba(c); Box::new(move |_, _| c) }
            Paint::LinearGradient(g) => {
                let h = halte(&mut g.stops(0, &[]));
                // Die Richtung: P0 -> P1, projiziert auf die Senkrechte zu P0P2.
                let (p0, p1, p2) = ((g.x0, g.y0), (g.x1, g.y1), (g.x2, g.y2));
                let d = (p2.0 - p0.0, p2.1 - p0.1);
                let q = (-d.1, d.0);
                let qq = q.0 * q.0 + q.1 * q.1;
                let p3 = if qq > 1e-9 {
                    let s = ((p1.0 - p0.0) * q.0 + (p1.1 - p0.1) * q.1) / qq;
                    (p0.0 + q.0 * s, p0.1 + q.1 * s)
                } else { p1 };
                let v = (p3.0 - p0.0, p3.1 - p0.1);
                let vv = (v.0 * v.0 + v.1 * v.1).max(1e-9);
                let ext = g.extend;
                Box::new(move |x, y| farbe_bei(&h, ausdehnen(((x - p0.0) * v.0 + (y - p0.1) * v.1) / vv, ext)))
            }
            Paint::RadialGradient(g) => {
                let h = halte(&mut g.stops(0, &[]));
                let (c0, r0, c1, r1, ext) = ((g.x0, g.y0), g.r0, (g.x1, g.y1), g.r1, g.extend);
                Box::new(move |x, y| {
                    // Der groesste t, fuer den der Punkt auf dem Kreis C(t) liegt.
                    let cd = (c1.0 - c0.0, c1.1 - c0.1);
                    let pd = (x - c0.0, y - c0.1);
                    let dr = r1 - r0;
                    let a = cd.0 * cd.0 + cd.1 * cd.1 - dr * dr;
                    let b = pd.0 * cd.0 + pd.1 * cd.1 + r0 * dr;
                    let c = pd.0 * pd.0 + pd.1 * pd.1 - r0 * r0;
                    let t = if a.abs() < 1e-6 { if b.abs() < 1e-9 { 0.0 } else { c / (2.0 * b) } } else {
                        let disk = b * b - a * c;
                        if disk < 0.0 { return [0.0; 4]; }
                        let w = disk.sqrt();
                        let (t1, t2) = ((b + w) / a, (b - w) / a);
                        if r0 + t1.max(t2) * dr >= 0.0 { t1.max(t2) } else { t1.min(t2) }
                    };
                    farbe_bei(&h, ausdehnen(t, ext))
                })
            }
            Paint::SweepGradient(g) => {
                let h = halte(&mut g.stops(0, &[]));
                // Winkel in Einheiten von 180 Grad (F2DOT14), gegen den Uhrzeigersinn.
                let (cx, cy, a0, a1, ext) = (g.center_x, g.center_y, g.start_angle * 180.0, g.end_angle * 180.0, g.extend);
                Box::new(move |x, y| {
                    let w = (y - cy).atan2(x - cx).to_degrees().rem_euclid(360.0);
                    let t = if (a1 - a0).abs() < 1e-6 { 0.0 } else { (w - a0) / (a1 - a0) };
                    farbe_bei(&h, ausdehnen(t, ext))
                })
            }
        };
        let (b, h) = (self.b, self.h);
        for py in 0..h {
            for px in 0..b {
                let i = py * b + px;
                let dk = self.deckung(i);
                if dk <= 0.0 { continue; }
                let (fx, fy) = match zurueck { Some(m) => m.an(px as f32 + 0.5, py as f32 + 0.5), None => (0.0, 0.0) };
                let c = farbe(fx, fy);
                let a = c[3] * dk;
                if a <= 0.0 { continue; }
                let e = &mut self.ebenen.last_mut().unwrap().0;
                let rest = 1.0 - a;
                for k in 0..3 { e[i * 4 + k] = c[k] * a + e[i * 4 + k] * rest; }
                e[i * 4 + 3] = a + e[i * 4 + 3] * rest;
            }
        }
        // v0 malt ohne Ausschnitt: der Umriss gilt nur fuer diesen Anstrich.
        self.umriss = None;
    }
    fn push_clip(&mut self) {
        let m = self.umriss.take().unwrap_or_else(|| vec![1.0; self.b * self.h]);
        self.ausschnitte.push(m);
    }
    fn push_clip_box(&mut self, k: ttf_parser::colr::ClipBox) {
        let m = self.maske_von(|u| {
            use ttf_parser::OutlineBuilder;
            u.move_to(k.x_min, k.y_min);
            u.line_to(k.x_max, k.y_min);
            u.line_to(k.x_max, k.y_max);
            u.line_to(k.x_min, k.y_max);
            u.close();
        });
        self.ausschnitte.push(m);
    }
    fn pop_clip(&mut self) { self.ausschnitte.pop(); }
    fn push_layer(&mut self, mode: CompositeMode) {
        self.ebenen.push((vec![0.0; self.b * self.h * 4], mode));
    }
    fn pop_layer(&mut self) {
        if self.ebenen.len() < 2 { return; }
        let (oben, mode) = self.ebenen.pop().unwrap();
        let unten = &mut self.ebenen.last_mut().unwrap().0;
        for i in 0..self.b * self.h {
            let s = [oben[i * 4], oben[i * 4 + 1], oben[i * 4 + 2], oben[i * 4 + 3]];
            let d = [unten[i * 4], unten[i * 4 + 1], unten[i * 4 + 2], unten[i * 4 + 3]];
            let r = match mode {
                CompositeMode::Clear => [0.0; 4],
                CompositeMode::Source => s,
                CompositeMode::Destination => d,
                CompositeMode::DestinationOver => [0, 1, 2, 3].map(|k| d[k] + s[k] * (1.0 - d[3])),
                CompositeMode::SourceIn => [0, 1, 2, 3].map(|k| s[k] * d[3]),
                CompositeMode::DestinationIn => [0, 1, 2, 3].map(|k| d[k] * s[3]),
                CompositeMode::SourceOut => [0, 1, 2, 3].map(|k| s[k] * (1.0 - d[3])),
                CompositeMode::DestinationOut => [0, 1, 2, 3].map(|k| d[k] * (1.0 - s[3])),
                CompositeMode::SourceAtop => [0, 1, 2, 3].map(|k| if k == 3 { d[3] } else { s[k] * d[3] + d[k] * (1.0 - s[3]) }),
                CompositeMode::DestinationAtop => [0, 1, 2, 3].map(|k| if k == 3 { s[3] } else { d[k] * s[3] + s[k] * (1.0 - d[3]) }),
                CompositeMode::Xor => [0, 1, 2, 3].map(|k| s[k] * (1.0 - d[3]) + d[k] * (1.0 - s[3])),
                CompositeMode::Plus => [0, 1, 2, 3].map(|k| (s[k] + d[k]).min(1.0)),
                // Die Mischmodi (Multiply, Screen ...) als Darueberlegen --
                // naeherungsweise, Emoji brauchen sie kaum.
                _ => [0, 1, 2, 3].map(|k| s[k] + d[k] * (1.0 - s[3])),
            };
            unten[i * 4..i * 4 + 4].copy_from_slice(&r);
        }
    }
    fn push_transform(&mut self, t: Transform) { self.abbildungen.push(Affin::von(t)); }
    fn pop_transform(&mut self) { self.abbildungen.pop(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbildungen_werden_richtig_verkettet_und_umgekehrt() {
        let s = Affin { a: 2.0, b: 0.0, c: 0.0, d: -2.0, e: 5.0, f: 10.0 };
        let t = Affin { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 3.0, f: 4.0 };
        let m = s.nach(t);
        assert_eq!(m.an(1.0, 1.0), s.an(4.0, 5.0));
        let u = m.umgekehrt().unwrap();
        let (x, y) = u.an(m.an(7.0, -2.0).0, m.an(7.0, -2.0).1);
        assert!((x - 7.0).abs() < 1e-4 && (y + 2.0).abs() < 1e-4);
    }

    #[test]
    fn verlauf_zwischen_den_halten() {
        let h = [(0.0, [1.0, 0.0, 0.0, 1.0]), (1.0, [0.0, 0.0, 1.0, 1.0])];
        assert_eq!(farbe_bei(&h, 0.5), [0.5, 0.0, 0.5, 1.0]);
        assert_eq!(farbe_bei(&h, -1.0), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(ausdehnen(1.25, GradientExtend::Repeat), 0.25);
        assert_eq!(ausdehnen(1.25, GradientExtend::Reflect), 0.75);
        assert_eq!(ausdehnen(1.25, GradientExtend::Pad), 1.0);
    }

    /// Mit der Emoji-Schrift von Windows: ein Smiley ist farbig und hat
    /// Gelb, wo eine einfarbige Silhouette nur eine Farbe haette.
    #[test]
    fn segoe_emoji_kommt_in_farbe() {
        let pfad = "C:/Windows/Fonts/seguiemj.ttf";
        if !std::path::Path::new(pfad).exists() { return; }
        let Some((face, _, _)) = gesicht_info(pfad) else { return };
        let Some(gid) = face.glyph_index('\u{1F600}') else { return };
        assert!(farbig(pfad, gid.0));
        let Some(Farbglyphe::Bild(b)) = rastern(pfad, gid.0, 48.0) else { panic!("kein Bild") };
        let mut farben = std::collections::HashSet::new();
        let mut gelb = 0;
        for p in b.rgba.chunks(4) {
            if p[3] > 200 {
                farben.insert((p[0] / 32, p[1] / 32, p[2] / 32));
                if p[0] > 180 && p[1] > 140 && p[2] < 120 { gelb += 1; }
            }
        }
        assert!(farben.len() >= 3, "nur {} Farben", farben.len());
        assert!(gelb > 50, "kaum Gelb: {}", gelb);
    }
}
