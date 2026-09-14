//! Software-Raster fuer die Zeichenbefehle auf einem IMAGE: gefuelltes Vieleck,
//! dicke Linie mit runden Gelenken, Rechteck, Rahmen, abgerundetes Rechteck,
//! Ellipse, Ring und Verlauf.
//!
//! **Geschrieben wird UEBER, nicht gemischt** -- ein Punkt bekommt genau die
//! Farbe samt Deckkraft, die der Befehl nennt. So zeichnet auch PILs
//! `ImageDraw` auf ein RGBA-Bild, und genau das braucht ein Kachel-Generator:
//! ein Loch in eine Scheibe stanzt man mit Deckkraft 0. Wer mischen will,
//! zeichnet in ein eigenes Bild und legt es mit `IMAGE_DRAW_IMAGE` darueber.
//!
//! **Koordinaten sind Kommazahlen, ein Punkt sitzt auf seiner ganzzahligen
//! Stelle** (Punkt (3, 4) hat seine Mitte bei x = 3, y = 4). Rechteck- und
//! Ellipsengrenzen gelten einschliesslich -- wie bei PIL, dessen Kacheln hier
//! nachgebaut werden. Keine Kantenglaettung: wer weiche Kanten will, zeichnet
//! groesser und skaliert herunter.
//!
//! Das Modul kennt raylib nicht: es arbeitet auf einem RGBA-Byte-Feld und ist
//! darum fuer sich pruefbar.

pub type Farbe = [u8; 4];

pub struct Leinwand<'a> {
    px: &'a mut [u8],
    w: i32,
    h: i32,
}

impl<'a> Leinwand<'a> {
    /// `px` muss `w * h * 4` Bytes lang sein (R, G, B, A je Punkt).
    pub fn neu(px: &'a mut [u8], w: i32, h: i32) -> Option<Self> {
        if w <= 0 || h <= 0 || px.len() < (w as usize) * (h as usize) * 4 {
            return None;
        }
        Some(Leinwand { px, w, h })
    }

    fn setzen(&mut self, x: i32, y: i32, c: Farbe) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return;
        }
        let i = ((y as usize) * (self.w as usize) + x as usize) * 4;
        self.px[i..i + 4].copy_from_slice(&c);
    }

    /// Ein Bild (RGBA-Bytes, `sw` x `sh`) ueber diese Leinwand legen:
    /// Ausschnitt `q = (qx, qy, qb, qh)` der Quelle an `(x, y)`, mit `tint`
    /// multipliziert. Echtes "ueber" (Porter-Duff) -- auch wenn das ZIEL
    /// halbdurchsichtig ist. raylibs `ImageDraw` mischt dort falsch: zwei
    /// Punkte mit Deckkraft 60 in Orange ergaben Rot 0x01.
    pub fn bild_ueber(&mut self, src: &[u8], sw: i32, sh: i32,
                      q: (i32, i32, i32, i32), x: i32, y: i32, tint: Farbe) {
        let (qx, qy, qb, qh) = q;
        if src.len() < (sw.max(0) as usize) * (sh.max(0) as usize) * 4 {
            return;
        }
        for dy in 0..qh {
            let (ty, sy) = (y + dy, qy + dy);
            if ty < 0 || ty >= self.h || sy < 0 || sy >= sh { continue; }
            for dx in 0..qb {
                let (tx, sx) = (x + dx, qx + dx);
                if tx < 0 || tx >= self.w || sx < 0 || sx >= sw { continue; }
                let si = ((sy * sw + sx) * 4) as usize;
                let di = ((ty * self.w + tx) * 4) as usize;
                let sa = src[si + 3] as f32 * tint[3] as f32 / (255.0 * 255.0);
                if sa <= 0.0 { continue; }
                let da = self.px[di + 3] as f32 / 255.0;
                let oa = sa + da * (1.0 - sa);
                for k in 0..3 {
                    let sc = src[si + k] as f32 * tint[k] as f32 / 255.0;
                    let dc = self.px[di + k] as f32;
                    let c = (sc * sa + dc * da * (1.0 - sa)) / oa;
                    self.px[di + k] = c.round().clamp(0.0, 255.0) as u8;
                }
                self.px[di + 3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    /// Alle Punkte einer Zeile mit `x0 <= x <= x1`.
    fn spanne(&mut self, y: i32, x0: f32, x1: f32, c: Farbe) {
        if y < 0 || y >= self.h || x1 < x0 {
            return;
        }
        let a = (x0.ceil() as i32).max(0);
        let b = (x1.floor() as i32).min(self.w - 1);
        for x in a..=b {
            self.setzen(x, y, c);
        }
    }

    /// Gefuelltes Rechteck, Grenzen einschliesslich.
    pub fn rechteck(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, c: Farbe) {
        let (x0, x1) = (x0.min(x1), x0.max(x1));
        let (y0, y1) = (y0.min(y1), y0.max(y1));
        let a = (y0.ceil() as i32).max(0);
        let b = (y1.floor() as i32).min(self.h - 1);
        for y in a..=b {
            self.spanne(y, x0, x1, c);
        }
    }

    /// Rahmen der Breite `breite`, nach INNEN gezeichnet (wie PILs `outline`).
    pub fn rahmen(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, breite: f32, c: Farbe) {
        let (x0, x1) = (x0.min(x1), x0.max(x1));
        let (y0, y1) = (y0.min(y1), y0.max(y1));
        let b = breite.max(1.0);
        if x1 - x0 < 2.0 * b || y1 - y0 < 2.0 * b {
            self.rechteck(x0, y0, x1, y1, c);
            return;
        }
        self.rechteck(x0, y0, x1, y0 + b - 1.0, c);
        self.rechteck(x0, y1 - b + 1.0, x1, y1, c);
        self.rechteck(x0, y0 + b, x0 + b - 1.0, y1 - b, c);
        self.rechteck(x1 - b + 1.0, y0 + b, x1, y1 - b, c);
    }

    /// Ellipse in der Box `(cx - rx, cy - ry) .. (cx + rx, cy + ry)`, gefuellt.
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Farbe) {
        if rx <= 0.0 || ry <= 0.0 {
            if rx >= 0.0 && ry >= 0.0 {
                self.setzen(cx.round() as i32, cy.round() as i32, c);
            }
            return;
        }
        let a = ((cy - ry).ceil() as i32).max(0);
        let b = ((cy + ry).floor() as i32).min(self.h - 1);
        for y in a..=b {
            let dy = (y as f32 - cy) / ry;
            let q = 1.0 - dy * dy;
            if q < 0.0 {
                continue;
            }
            let dx = rx * q.sqrt();
            self.spanne(y, cx - dx, cx + dx, c);
        }
    }

    /// Kreisring: alles mit `r - breite < Abstand <= r` (nach innen, wie PIL).
    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, breite: f32, c: Farbe) {
        if r <= 0.0 {
            return;
        }
        let innen = (r - breite.max(1.0)).max(0.0);
        let a = ((cy - r).ceil() as i32).max(0);
        let b = ((cy + r).floor() as i32).min(self.h - 1);
        for y in a..=b {
            let dy = y as f32 - cy;
            let aussen2 = r * r - dy * dy;
            if aussen2 < 0.0 {
                continue;
            }
            let ax = aussen2.sqrt();
            let innen2 = innen * innen - dy * dy;
            if innen <= 0.0 || innen2 <= 0.0 {
                self.spanne(y, cx - ax, cx + ax, c);
            } else {
                let ix = innen2.sqrt();
                // offene Innengrenze: ein Punkt genau auf dem inneren Kreis gehoert nicht dazu
                self.spanne(y, cx - ax, (cx - ix).next_down_f32(), c);
                self.spanne(y, (cx + ix).next_up_f32(), cx + ax, c);
            }
        }
    }

    /// Abgerundetes Rechteck mit Eckradius `r`, gefuellt.
    pub fn rund_rechteck(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, c: Farbe) {
        let (x0, x1) = (x0.min(x1), x0.max(x1));
        let (y0, y1) = (y0.min(y1), y0.max(y1));
        let r = r.max(0.0).min((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
        if r <= 0.0 {
            self.rechteck(x0, y0, x1, y1, c);
            return;
        }
        let a = (y0.ceil() as i32).max(0);
        let b = (y1.floor() as i32).min(self.h - 1);
        for y in a..=b {
            let yf = y as f32;
            // Abstand in die Ecke: oben und unten wird die Zeile eingerueckt
            let d = if yf < y0 + r {
                y0 + r - yf
            } else if yf > y1 - r {
                yf - (y1 - r)
            } else {
                0.0
            };
            let einzug = if d > 0.0 { r - (r * r - d * d).max(0.0).sqrt() } else { 0.0 };
            self.spanne(y, x0 + einzug, x1 - einzug, c);
        }
    }

    /// Gefuelltes Vieleck (gerade-ungerade-Regel), Kanten an Punktmitten.
    pub fn vieleck(&mut self, pkt: &[(f32, f32)], c: Farbe) {
        if pkt.len() < 3 {
            if pkt.len() == 2 {
                self.linie(pkt, 1.0, c);
            } else if pkt.len() == 1 {
                self.setzen(pkt[0].0.round() as i32, pkt[0].1.round() as i32, c);
            }
            return;
        }
        let ymin = pkt.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let ymax = pkt.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
        let a = (ymin.ceil() as i32).max(0);
        let b = (ymax.floor() as i32).min(self.h - 1);
        let mut schnitte: Vec<f32> = Vec::with_capacity(pkt.len());
        for y in a..=b {
            let yf = y as f32;
            schnitte.clear();
            for i in 0..pkt.len() {
                let (xa, ya) = pkt[i];
                let (xb, yb) = pkt[(i + 1) % pkt.len()];
                if ya == yb {
                    continue;
                }
                // halboffen, damit ein Eckpunkt nicht doppelt zaehlt
                let (lo, hi) = if ya < yb { (ya, yb) } else { (yb, ya) };
                if yf < lo || yf >= hi {
                    continue;
                }
                schnitte.push(xa + (yf - ya) * (xb - xa) / (yb - ya));
            }
            schnitte.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
            for paar in schnitte.chunks(2) {
                if paar.len() == 2 {
                    self.spanne(y, paar[0], paar[1], c);
                }
            }
        }
        // waagerechte Kanten ganz oben/unten haetten sonst keinen Punkt
        for i in 0..pkt.len() {
            let (xa, ya) = pkt[i];
            let (xb, yb) = pkt[(i + 1) % pkt.len()];
            if ya == yb && ya.fract() == 0.0 {
                self.spanne(ya as i32, xa.min(xb), xa.max(xb), c);
            }
        }
    }

    /// Linienzug der Breite `breite`. Jedes Stueck ist ein Viereck, an den
    /// Gelenken sitzt eine Scheibe -- runde Stoesse wie PILs `joint="curve"`.
    pub fn linie(&mut self, pkt: &[(f32, f32)], breite: f32, c: Farbe) {
        if pkt.is_empty() {
            return;
        }
        let halb = breite.max(1.0) / 2.0;
        if pkt.len() == 1 {
            self.ellipse(pkt[0].0, pkt[0].1, halb, halb, c);
            return;
        }
        for i in 0..pkt.len() - 1 {
            let (xa, ya) = pkt[i];
            let (xb, yb) = pkt[i + 1];
            let (dx, dy) = (xb - xa, yb - ya);
            let len = (dx * dx + dy * dy).sqrt();
            if len == 0.0 {
                continue;
            }
            if breite <= 1.0 {
                // duenne Linie: ein Punkt je Schritt, damit nichts abreisst
                let n = len.ceil() as i32;
                for k in 0..=n {
                    let t = k as f32 / n.max(1) as f32;
                    self.setzen((xa + dx * t).round() as i32, (ya + dy * t).round() as i32, c);
                }
                continue;
            }
            let (nx, ny) = (-dy / len * halb, dx / len * halb);
            self.vieleck(&[(xa + nx, ya + ny), (xb + nx, yb + ny), (xb - nx, yb - ny), (xa - nx, ya - ny)], c);
            if i > 0 {
                self.ellipse(xa, ya, halb, halb, c);
            }
        }
    }

    /// Verlauf in einem Rechteck, zeilen- (senkrecht) oder spaltenweise, von
    /// `c1` nach `c2` -- Farbe UND Deckkraft werden gemischt.
    pub fn verlauf(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, c1: Farbe, c2: Farbe, senkrecht: bool) {
        let mischen = |t: f32| -> Farbe {
            let t = t.clamp(0.0, 1.0);
            let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
            [m(c1[0], c2[0]), m(c1[1], c2[1]), m(c1[2], c2[2]), m(c1[3], c2[3])]
        };
        if senkrecht {
            let hoehe = (y1 - y0).max(1.0);
            for y in (y0 as i32)..=(y1 as i32) {
                let c = mischen((y as f32 - y0) / hoehe);
                self.spanne(y, x0, x1, c);
            }
        } else {
            let breite = (x1 - x0).max(1.0);
            for x in (x0 as i32)..=(x1 as i32) {
                let c = mischen((x as f32 - x0) / breite);
                self.rechteck(x as f32, y0, x as f32, y1, c);
            }
        }
    }
}

trait NaechsteZahl {
    fn next_up_f32(self) -> f32;
    fn next_down_f32(self) -> f32;
}

impl NaechsteZahl for f32 {
    fn next_up_f32(self) -> f32 {
        self + f32::EPSILON * self.abs().max(1.0)
    }
    fn next_down_f32(self) -> f32 {
        self - f32::EPSILON * self.abs().max(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROT: Farbe = [255, 0, 0, 255];
    const LEER: Farbe = [0, 0, 0, 0];

    fn feld(w: i32, h: i32) -> Vec<u8> {
        vec![0u8; (w * h * 4) as usize]
    }

    #[test]
    fn ueber_halbdurchsichtigem_ziel_behaelt_die_farbe() {
        // Orange 60 ueber Orange 59: Farbe bleibt, Deckkraft 60 + 59 * (1 - 60/255).
        let mut px = vec![255u8, 146, 34, 59];
        let src = [255u8, 150, 40, 60];
        let mut l = Leinwand::neu(&mut px, 1, 1).unwrap();
        l.bild_ueber(&src, 1, 1, (0, 0, 1, 1), 0, 0, [255, 255, 255, 255]);
        assert_eq!(px[3], 105);
        assert!(px[0] >= 254 && (146..=150).contains(&px[1]) && (34..=40).contains(&px[2]), "{:?}", px);
        // Deckende Quelle ersetzt, leeres Ziel nimmt die Quelle, Tint multipliziert.
        let mut px = vec![0u8, 0, 255, 255, 0, 0, 0, 0];
        let src = [0u8, 255, 0, 255, 200, 100, 50, 128];
        let mut l = Leinwand::neu(&mut px, 2, 1).unwrap();
        l.bild_ueber(&src, 2, 1, (0, 0, 2, 1), 0, 0, [255, 255, 255, 255]);
        assert_eq!(&px[..4], &[0, 255, 0, 255]);
        assert_eq!(&px[4..], &[200, 100, 50, 128]);
        let mut px = vec![0u8; 4];
        let mut l = Leinwand::neu(&mut px, 1, 1).unwrap();
        l.bild_ueber(&[200, 200, 200, 255], 1, 1, (0, 0, 1, 1), 0, 0, [255, 0, 0, 255]);
        assert_eq!(px, vec![200, 0, 0, 255]);
    }

    fn punkt(px: &[u8], w: i32, x: i32, y: i32) -> Farbe {
        let i = ((y * w + x) * 4) as usize;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    }

    fn gefuellt(px: &[u8]) -> usize {
        px.chunks(4).filter(|p| p[3] != 0).count()
    }

    #[test]
    fn rechteck_schliesst_beide_grenzen_ein() {
        let mut px = feld(10, 10);
        Leinwand::neu(&mut px, 10, 10).unwrap().rechteck(2.0, 3.0, 4.0, 5.0, ROT);
        assert_eq!(gefuellt(&px), 9);
        assert_eq!(punkt(&px, 10, 2, 3), ROT);
        assert_eq!(punkt(&px, 10, 4, 5), ROT);
        assert_eq!(punkt(&px, 10, 5, 5), LEER);
    }

    #[test]
    fn ueber_den_rand_hinaus_ist_kein_absturz() {
        let mut px = feld(8, 8);
        let mut l = Leinwand::neu(&mut px, 8, 8).unwrap();
        l.rechteck(-50.0, -50.0, 50.0, 50.0, ROT);
        l.ellipse(4.0, 4.0, 100.0, 100.0, ROT);
        l.vieleck(&[(-10.0, -10.0), (20.0, 0.0), (0.0, 30.0)], ROT);
        l.linie(&[(-5.0, 4.0), (15.0, 4.0)], 3.0, ROT);
        assert_eq!(gefuellt(&px), 64);
    }

    #[test]
    fn es_wird_ueberschrieben_nicht_gemischt() {
        // Das Loch in einer Scheibe: Deckkraft 0 muss als 0 im Bild landen.
        let mut px = feld(16, 16);
        let mut l = Leinwand::neu(&mut px, 16, 16).unwrap();
        l.ellipse(8.0, 8.0, 7.0, 7.0, ROT);
        l.ellipse(8.0, 8.0, 3.0, 3.0, LEER);
        assert_eq!(punkt(&px, 16, 8, 8), LEER);
        assert_eq!(punkt(&px, 16, 8, 2), ROT);
        let halb = [0, 0, 255, 128];
        Leinwand::neu(&mut px, 16, 16).unwrap().rechteck(0.0, 0.0, 15.0, 15.0, halb);
        assert_eq!(punkt(&px, 16, 8, 8), halb);
    }

    #[test]
    fn ellipse_ist_symmetrisch_und_hat_die_richtige_flaeche() {
        let mut px = feld(64, 64);
        Leinwand::neu(&mut px, 64, 64).unwrap().ellipse(32.0, 32.0, 20.0, 20.0, ROT);
        let n = gefuellt(&px) as f32;
        let soll = std::f32::consts::PI * 20.0 * 20.0;
        assert!((n - soll).abs() / soll < 0.03, "{} gegen {}", n, soll);
        for (x, y) in [(12, 32), (52, 32), (32, 12), (32, 52)] {
            assert_eq!(punkt(&px, 64, x, y), ROT, "Rand bei {},{}", x, y);
        }
        assert_eq!(punkt(&px, 64, 11, 32), LEER);
    }

    #[test]
    fn ring_ist_innen_leer() {
        let mut px = feld(64, 64);
        Leinwand::neu(&mut px, 64, 64).unwrap().ring(32.0, 32.0, 20.0, 4.0, ROT);
        assert_eq!(punkt(&px, 64, 32, 32), LEER);
        assert_eq!(punkt(&px, 64, 32, 13), ROT);
        assert_eq!(punkt(&px, 64, 32, 18), LEER);
        let n = gefuellt(&px) as f32;
        let soll = std::f32::consts::PI * (20.0 * 20.0 - 16.0 * 16.0);
        assert!((n - soll).abs() / soll < 0.08, "{} gegen {}", n, soll);
    }

    #[test]
    fn vieleck_dreieck_und_konkave_form() {
        let mut px = feld(40, 40);
        Leinwand::neu(&mut px, 40, 40).unwrap().vieleck(&[(0.0, 0.0), (30.0, 0.0), (0.0, 30.0)], ROT);
        let n = gefuellt(&px) as f32;
        assert!((n - 465.0).abs() < 40.0, "{}", n);
        // konkav: ein V -- die Kerbe bleibt frei
        let mut px = feld(40, 40);
        Leinwand::neu(&mut px, 40, 40).unwrap().vieleck(
            &[(0.0, 0.0), (20.0, 20.0), (40.0, 0.0), (40.0, 30.0), (0.0, 30.0)], ROT);
        assert_eq!(punkt(&px, 40, 20, 5), LEER);
        assert_eq!(punkt(&px, 40, 20, 25), ROT);
    }

    #[test]
    fn dicke_linie_hat_ihre_breite_und_runde_gelenke() {
        let mut px = feld(40, 40);
        Leinwand::neu(&mut px, 40, 40).unwrap().linie(&[(5.0, 20.0), (35.0, 20.0)], 6.0, ROT);
        assert_eq!(punkt(&px, 40, 20, 17), ROT);
        assert_eq!(punkt(&px, 40, 20, 23), ROT);
        assert_eq!(punkt(&px, 40, 20, 24), LEER);
        let mut px = feld(40, 40);
        Leinwand::neu(&mut px, 40, 40).unwrap().linie(&[(5.0, 5.0), (20.0, 20.0), (35.0, 5.0)], 6.0, ROT);
        assert_eq!(punkt(&px, 40, 20, 22), ROT, "am Gelenk darf keine Kerbe sein");
    }

    #[test]
    fn duenne_linie_reisst_nicht_ab() {
        let mut px = feld(20, 20);
        Leinwand::neu(&mut px, 20, 20).unwrap().linie(&[(0.0, 0.0), (19.0, 7.0)], 1.0, ROT);
        for x in 0..20 {
            assert!((0..20).any(|y| punkt(&px, 20, x, y) == ROT), "Spalte {} leer", x);
        }
    }

    #[test]
    fn abgerundetes_rechteck_hat_leere_ecken() {
        let mut px = feld(40, 40);
        Leinwand::neu(&mut px, 40, 40).unwrap().rund_rechteck(0.0, 0.0, 39.0, 39.0, 10.0, ROT);
        assert_eq!(punkt(&px, 40, 0, 0), LEER);
        assert_eq!(punkt(&px, 40, 39, 39), LEER);
        assert_eq!(punkt(&px, 40, 20, 0), ROT);
        assert_eq!(punkt(&px, 40, 0, 20), ROT);
        assert_eq!(punkt(&px, 40, 20, 20), ROT);
    }

    #[test]
    fn rahmen_ist_innen_leer_und_nach_innen_gezeichnet() {
        let mut px = feld(20, 20);
        Leinwand::neu(&mut px, 20, 20).unwrap().rahmen(2.0, 2.0, 17.0, 17.0, 2.0, ROT);
        assert_eq!(punkt(&px, 20, 2, 10), ROT);
        assert_eq!(punkt(&px, 20, 3, 10), ROT);
        assert_eq!(punkt(&px, 20, 4, 10), LEER);
        assert_eq!(punkt(&px, 20, 1, 10), LEER);
        assert_eq!(punkt(&px, 20, 17, 17), ROT);
    }

    #[test]
    fn verlauf_beginnt_und_endet_bei_den_farben() {
        let mut px = feld(10, 11);
        let oben = [0, 0, 0, 255];
        let unten = [200, 100, 50, 55];
        Leinwand::neu(&mut px, 10, 11).unwrap().verlauf(0.0, 0.0, 9.0, 10.0, oben, unten, true);
        assert_eq!(punkt(&px, 10, 4, 0), oben);
        assert_eq!(punkt(&px, 10, 4, 10), unten);
        assert_eq!(punkt(&px, 10, 4, 5), [100, 50, 25, 155]);
        let mut px = feld(11, 4);
        Leinwand::neu(&mut px, 11, 4).unwrap().verlauf(0.0, 0.0, 10.0, 3.0, oben, unten, false);
        assert_eq!(punkt(&px, 11, 0, 2), oben);
        assert_eq!(punkt(&px, 11, 10, 2), unten);
    }

    #[test]
    fn zu_kleines_feld_wird_abgelehnt() {
        let mut px = vec![0u8; 10];
        assert!(Leinwand::neu(&mut px, 4, 4).is_none());
        assert!(Leinwand::neu(&mut px, 0, 4).is_none());
    }
}
