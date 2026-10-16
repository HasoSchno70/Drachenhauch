//! Skalieren und Bildanpassung so, wie Pillow (PIL) es rechnet.
//!
//! raylibs `ImageResize` (stb_image_resize) liefert andere Zahlen als PILs
//! `Image.resize(..., LANCZOS)`, und wer ein Muster aus einem Foto ableitet,
//! bekommt mit einem anderen Filter andere Farben -- Glutfaden vergleicht sich
//! Punkt fuer Punkt mit PySticky, das mit PIL arbeitet. Nachgebaut ist
//! `libImaging/Resample.c` (Gewichte in doppelter Genauigkeit, dann auf
//! 22 Bit Festkomma gerundet, erst waagerecht in ein 8-Bit-Zwischenbild, dann
//! senkrecht) und `ImageEnhance` ueber `Image.blend` (in einfacher
//! Genauigkeit, wie `ImagingBlend` mit seinem `float alpha`).
//!
//! Alles arbeitet auf RGBA-Bytes, Zeile fuer Zeile, und kennt kein Fenster.

/// Die Filter von PIL.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Filter { Box, Bilinear, Hamming, Bikubisch, Lanczos }

impl Filter {
    /// Name -> Filter (deutsch oder englisch). `None` = unbekannt.
    pub fn aus_namen(n: &str) -> Option<Filter> {
        match n.to_lowercase().as_str() {
            "lanczos" => Some(Filter::Lanczos),
            "bikubisch" | "bicubic" => Some(Filter::Bikubisch),
            "box" => Some(Filter::Box),
            "hamming" => Some(Filter::Hamming),
            "linear" | "dreieck" | "triangle" => Some(Filter::Bilinear),
            _ => None,
        }
    }
    fn traeger(self) -> f64 {
        match self {
            Filter::Box => 0.5,
            Filter::Bilinear | Filter::Hamming => 1.0,
            Filter::Bikubisch => 2.0,
            Filter::Lanczos => 3.0,
        }
    }
    fn wert(self, x: f64) -> f64 {
        match self {
            Filter::Box => if x > -0.5 && x <= 0.5 { 1.0 } else { 0.0 },
            Filter::Bilinear => {
                let x = x.abs();
                if x < 1.0 { 1.0 - x } else { 0.0 }
            }
            Filter::Hamming => {
                let x = x.abs();
                if x == 0.0 { return 1.0; }
                if x >= 1.0 { return 0.0; }
                let x = x * std::f64::consts::PI;
                // 0.54f/0.46f sind in Resample.c float-Literale
                x.sin() / x * (0.54f32 as f64 + 0.46f32 as f64 * x.cos())
            }
            Filter::Bikubisch => {
                let a = -0.5;
                let x = x.abs();
                if x < 1.0 { ((a + 2.0) * x - (a + 3.0)) * x * x + 1.0 }
                else if x < 2.0 { (((x - 5.0) * x + 8.0) * x - 4.0) * a }
                else { 0.0 }
            }
            Filter::Lanczos => {
                if (-3.0..3.0).contains(&x) { sinc(x) * sinc(x / 3.0) } else { 0.0 }
            }
        }
    }
}

fn sinc(x: f64) -> f64 {
    if x == 0.0 { return 1.0; }
    let x = x * std::f64::consts::PI;
    x.sin() / x
}

const GENAU: u32 = 32 - 8 - 2; // PRECISION_BITS in Resample.c

/// Gewichte einer Achse: je Zielpunkt (erster Quellpunkt, Anzahl) und die
/// Gewichte als Festkommazahlen; `ksize` Plaetze je Zielpunkt.
struct Gewichte { ksize: usize, grenzen: Vec<(usize, usize)>, k: Vec<i32> }

fn gewichte(in_groesse: usize, aus_groesse: usize, f: Filter) -> Gewichte {
    let (in0, in1) = (0.0f64, in_groesse as f64);
    let massstab = (in1 - in0) / aus_groesse as f64;
    let fmass = if massstab < 1.0 { 1.0 } else { massstab };
    let traeger = f.traeger() * fmass;
    let ksize = (traeger.ceil() as usize) * 2 + 1;
    let mut grenzen = Vec::with_capacity(aus_groesse);
    let mut k = vec![0i32; aus_groesse * ksize];
    let mut roh = vec![0f64; ksize];
    let ss = 1.0 / fmass;
    for xx in 0..aus_groesse {
        let mitte = in0 + (xx as f64 + 0.5) * massstab;
        // (int) in C schneidet Richtung null ab
        let mut xmin = (mitte - traeger + 0.5) as i64;
        if xmin < 0 { xmin = 0; }
        let mut xmax = (mitte + traeger + 0.5) as i64;
        if xmax > in_groesse as i64 { xmax = in_groesse as i64; }
        let xmax = (xmax - xmin).max(0) as usize;
        let xmin = xmin as usize;
        let mut ww = 0.0;
        for x in 0..xmax {
            let w = f.wert((x as f64 + xmin as f64 - mitte + 0.5) * ss);
            roh[x] = w;
            ww += w;
        }
        for x in 0..xmax {
            let w = if ww != 0.0 { roh[x] / ww } else { roh[x] };
            let s = w * (1u32 << GENAU) as f64;
            k[xx * ksize + x] = if w < 0.0 { (-0.5 + s) as i32 } else { (0.5 + s) as i32 };
        }
        grenzen.push((xmin, xmax));
    }
    Gewichte { ksize, grenzen, k }
}

fn clip8(v: i64) -> u8 { (v >> GENAU).clamp(0, 255) as u8 }

/// RGBA -> vormultipliziert (PILs RGBA -> RGBa).
fn vormultiplizieren(px: &mut [u8]) {
    for p in px.chunks_exact_mut(4) {
        let a = p[3] as u32;
        for c in &mut p[..3] {
            let t = *c as u32 * a + 128;
            *c = (((t >> 8) + t) >> 8) as u8;
        }
    }
}

/// zurueck (PILs RGBa -> RGBA).
fn entmultiplizieren(px: &mut [u8]) {
    for p in px.chunks_exact_mut(4) {
        let a = p[3] as u32;
        if a == 255 || a == 0 { continue; }
        for c in &mut p[..3] {
            *c = ((255 * *c as u32) / a).min(255) as u8;
        }
    }
}

/// Ein RGBA-Bild auf `nb` x `nh` bringen wie PIL `Image.resize(size, filter)`.
/// Mit Deckkraft wird wie bei PIL vormultipliziert gerechnet (sonst bluten
/// die Farben durchsichtiger Punkte in die Nachbarn).
pub fn skalieren(px: &[u8], b: usize, h: usize, nb: usize, nh: usize, f: Filter) -> Vec<u8> {
    assert_eq!(px.len(), b * h * 4);
    if nb == b && nh == h { return px.to_vec(); }
    let mit_alpha = px.chunks_exact(4).any(|p| p[3] != 255);
    let mut quelle = px.to_vec();
    if mit_alpha { vormultiplizieren(&mut quelle); }
    let gw = gewichte(b, nb, f);
    let gs = gewichte(h, nh, f);
    let waagerecht = nb != b;
    let senkrecht = nh != h;
    let (erste, letzte) = (gs.grenzen[0].0, gs.grenzen[nh - 1].0 + gs.grenzen[nh - 1].1);
    // waagerecht (nur die Zeilen, die die senkrechte Runde braucht)
    let (zw, zw_b, zw_h, versatz) = if waagerecht {
        let zh = letzte - erste;
        let mut aus = vec![0u8; nb * zh * 4];
        for yy in 0..zh {
            let zeile = &quelle[(yy + erste) * b * 4..(yy + erste + 1) * b * 4];
            for xx in 0..nb {
                let (xmin, xmax) = gw.grenzen[xx];
                let k = &gw.k[xx * gw.ksize..];
                for c in 0..4 {
                    let mut ss: i64 = 1 << (GENAU - 1);
                    for x in 0..xmax {
                        ss += zeile[(x + xmin) * 4 + c] as i64 * k[x] as i64;
                    }
                    aus[(yy * nb + xx) * 4 + c] = clip8(ss);
                }
            }
        }
        (aus, nb, zh, erste)
    } else {
        (quelle, b, h, 0)
    };
    let mut ergebnis = if senkrecht {
        let mut aus = vec![0u8; zw_b * nh * 4];
        for yy in 0..nh {
            let (ymin, ymax) = gs.grenzen[yy];
            let ymin = ymin - versatz;
            let k = &gs.k[yy * gs.ksize..];
            for xx in 0..zw_b {
                for c in 0..4 {
                    let mut ss: i64 = 1 << (GENAU - 1);
                    for y in 0..ymax {
                        ss += zw[((y + ymin) * zw_b + xx) * 4 + c] as i64 * k[y] as i64;
                    }
                    aus[(yy * zw_b + xx) * 4 + c] = clip8(ss);
                }
            }
        }
        aus
    } else {
        let _ = zw_h;
        zw
    };
    if mit_alpha { entmultiplizieren(&mut ergebnis); }
    ergebnis
}

/// Die Anpassungen von `ImageEnhance`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anpassung { Helligkeit, Kontrast, Saettigung }

impl Anpassung {
    pub fn aus_namen(n: &str) -> Option<Anpassung> {
        match n.to_lowercase().as_str() {
            "helligkeit" | "brightness" => Some(Anpassung::Helligkeit),
            "kontrast" | "contrast" => Some(Anpassung::Kontrast),
            "saettigung" | "sättigung" | "farbe" | "color" | "saturation" => Some(Anpassung::Saettigung),
            _ => None,
        }
    }
}

/// Grauwert wie PILs `convert("L")` (ITU-R 601-2, Festkomma).
pub fn grau(r: u8, g: u8, b: u8) -> u8 {
    ((r as u32 * 19595 + g as u32 * 38470 + b as u32 * 7471 + 0x8000) >> 16) as u8
}

/// `Image.blend(entartet, bild, faktor)` -- in einfacher Genauigkeit.
fn mischen(entartet: u8, bild: u8, alpha: f32) -> u8 {
    let t = entartet as i32 as f32 + alpha * (bild as i32 - entartet as i32) as f32;
    if (0.0..=1.0).contains(&alpha) {
        t as u8
    } else if t <= 0.0 { 0 } else if t >= 255.0 { 255 } else { t as u8 }
}

/// Ein RGBA-Bild anpassen wie `ImageEnhance.<Art>(bild).enhance(faktor)`.
/// 1.0 laesst das Bild, wie es ist; die Deckkraft bleibt immer unberuehrt.
pub fn anpassen(px: &[u8], art: Anpassung, faktor: f64) -> Vec<u8> {
    let alpha = faktor as f32;
    let mut aus = px.to_vec();
    let mittel = if art == Anpassung::Kontrast {
        // ImageStat.mean ueber die Grauwerte, dann int(mean + 0.5)
        let n = (px.len() / 4).max(1) as f64;
        let summe: u64 = px.chunks_exact(4).map(|p| grau(p[0], p[1], p[2]) as u64).sum();
        (summe as f64 / n + 0.5) as u8
    } else { 0 };
    for p in aus.chunks_exact_mut(4) {
        let e = match art {
            Anpassung::Helligkeit => [0u8; 3],
            Anpassung::Kontrast => [mittel; 3],
            Anpassung::Saettigung => { let l = grau(p[0], p[1], p[2]); [l; 3] }
        };
        for c in 0..3 { p[c] = mischen(e[c], p[c], alpha); }
    }
    aus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grauverlauf(b: usize, h: usize) -> Vec<u8> {
        let mut v = Vec::with_capacity(b * h * 4);
        for y in 0..h { for x in 0..b {
            let w = ((x * 255) / (b - 1).max(1)) as u8;
            v.extend_from_slice(&[w, (y * 30 % 256) as u8, 255 - w, 255]);
        } }
        v
    }

    #[test]
    fn gleiche_groesse_ist_eine_kopie() {
        let v = grauverlauf(5, 4);
        assert_eq!(skalieren(&v, 5, 4, 5, 4, Filter::Lanczos), v);
    }

    #[test]
    fn einfarbig_bleibt_einfarbig() {
        let v: Vec<u8> = std::iter::repeat([200u8, 10, 99, 255]).take(37 * 23).flatten().collect();
        for f in [Filter::Lanczos, Filter::Bikubisch, Filter::Box, Filter::Hamming, Filter::Bilinear] {
            let s = skalieren(&v, 37, 23, 11, 7, f);
            assert!(s.chunks_exact(4).all(|p| p == [200, 10, 99, 255]), "{:?}", f);
            let s = skalieren(&v, 37, 23, 80, 50, f);
            assert!(s.chunks_exact(4).all(|p| p == [200, 10, 99, 255]), "{:?}", f);
        }
    }

    #[test]
    fn gewichte_summieren_auf_eins() {
        for (i, o) in [(100, 7), (7, 100), (640, 50), (3, 1)] {
            let g = gewichte(i, o, Filter::Lanczos);
            for xx in 0..o {
                let s: i64 = g.k[xx * g.ksize..xx * g.ksize + g.grenzen[xx].1].iter().map(|&k| k as i64).sum();
                assert!((s - (1 << GENAU)).abs() <= g.grenzen[xx].1 as i64, "{} {} {}", i, o, s);
            }
        }
    }

    #[test]
    fn werte_wie_pillow() {
        // Pillow 12.1.1: Image.new RGB 4x1 [(0,0,0),(255,255,255),(0,0,0),(255,255,255)]
        // .resize((3,1), LANCZOS) -> [(91,91,91),(128,128,128),(164,164,164)]
        let mut v = Vec::new();
        for i in 0..4 { let w = if i % 2 == 1 { 255 } else { 0 }; v.extend_from_slice(&[w, w, w, 255]); }
        let s = skalieren(&v, 4, 1, 3, 1, Filter::Lanczos);
        let r: Vec<u8> = s.chunks_exact(4).map(|p| p[0]).collect();
        assert_eq!(r, vec![91, 128, 164]);
    }

    #[test]
    fn grau_wie_pillow() {
        assert_eq!(grau(255, 0, 0), 76);
        assert_eq!(grau(0, 255, 0), 150);
        assert_eq!(grau(0, 0, 255), 29);
        assert_eq!(grau(255, 255, 255), 255);
    }

    #[test]
    fn anpassung_eins_laesst_das_bild() {
        let v = grauverlauf(9, 3);
        for a in [Anpassung::Helligkeit, Anpassung::Kontrast, Anpassung::Saettigung] {
            assert_eq!(anpassen(&v, a, 1.0), v);
        }
        // Saettigung 0 = Grauwert in allen drei Kanaelen
        let g = anpassen(&[255, 0, 0, 255], Anpassung::Saettigung, 0.0);
        assert_eq!(g, vec![76, 76, 76, 255]);
        // Helligkeit 2 klemmt
        assert_eq!(anpassen(&[200, 100, 0, 7], Anpassung::Helligkeit, 2.0), vec![255, 200, 0, 7]);
    }
}
