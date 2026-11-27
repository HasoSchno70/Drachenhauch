//! Klangthemen und Funken (docs/module-gui.md, Abschnitt 10).
//!
//! Teil von `gui.rs`.
//! - **Klang**: `klang: klick` in der Stildatei -- in einer Regel ohne
//!   Zustand beim Klick, mit `:hover` beim Hineinfahren, `:active` beim
//!   Druecken, `:focus` beim Fokus. Eingebaute Klaenge (`KLAENGE`) oder eine
//!   Datei. Die gui spielt nicht selbst: sie legt die Wuensche in
//!   `klang_warte`, die VM spielt sie nach GUI_UPDATE (vm_gui.rs, dort ist
//!   das Audio). `GUI_SOUND_LOG$` nennt, was gewuenscht wurde -- auch ohne
//!   Soundkarte, damit es sich pruefen laesst.
//! - **Funken**: `effekt: funken|konfetti|sterne` (dieselben Anlaesse) oder
//!   `GUI_BURST`. Kleine Teilchen, gezeichnet ganz oben (draw_top), bewegt
//!   mit der Uhr der gui. Ohne Bewegung (GUI_SET_MOTION, "Animationen
//!   reduzieren") gibt es keine.

use super::*;

/// Eingebaute Klaenge der gui.
pub(super) const KLAENGE: [&str; 6] = ["klick", "tick", "ping", "plopp", "fehler", "erfolg"];
/// Effekte: 0 = keiner.
pub(super) const EFFEKTE: [&str; 4] = ["", "funken", "konfetti", "sterne"];

/// `klang: name [NN%]` lesen: (Name oder Pfad, Lautstaerke in Prozent).
pub(super) fn klang_lesen(wert: &str, zeile: usize) -> Result<(String, i32), String> {
    let mut name = String::new();
    let mut laut = 100;
    for t in wert.split_whitespace() {
        if let Some(p) = t.strip_suffix('%') {
            laut = p.parse::<i32>().ok().filter(|v| (0..=100).contains(v))
                .ok_or_else(|| format!("Zeile {}: klang -- Lautstaerke '{}' muss 0%..100% sein", zeile, t))?;
        } else if name.is_empty() {
            name = t.trim_matches('"').trim_matches('\'').to_string();
        } else {
            return Err(format!("Zeile {}: klang '{}' -- erwartet name [lautstaerke%]", zeile, wert.trim()));
        }
    }
    if name.is_empty() || name == "keiner" || name == "none" { return Ok((String::new(), laut)); }
    let ist_datei = name.contains('.') || name.contains('/') || name.contains('\\');
    if !ist_datei && !KLAENGE.contains(&name.to_lowercase().as_str()) {
        return Err(format!("Zeile {}: klang '{}' unbekannt -- {} oder eine Datei (.wav, .ogg)", zeile, name, KLAENGE.join(", ")));
    }
    Ok((if ist_datei { name } else { name.to_lowercase() }, laut))
}

/// `effekt: art` lesen.
pub(super) fn effekt_lesen(wert: &str, zeile: usize) -> Result<u8, String> {
    let w = wert.trim().trim_matches('"').to_lowercase();
    if w.is_empty() || w == "keiner" || w == "none" { return Ok(0); }
    EFFEKTE.iter().position(|e| !e.is_empty() && *e == w).map(|n| n as u8)
        .ok_or_else(|| format!("Zeile {}: effekt '{}' unbekannt -- funken, konfetti, sterne", zeile, wert.trim()))
}

/// Ein Teilchen: Lage und Geschwindigkeit in Bildpunkten, Leben in
/// Sekunden, Art (1 Funke, 2 Konfetti, 3 Stern).
#[derive(Clone, Debug)]
pub(super) struct Funke {
    x: f64, y: f64, vx: f64, vy: f64,
    alter: f64, leben: f64,
    farbe: i64, groesse: f64, art: u8, dreh: f64, dreh_v: f64,
}

/// Ein kleiner, fester Zufall (xorshift) -- dieselbe Folge in jedem Lauf,
/// damit Bilder in Pruefungen gleich bleiben.
fn zufall(s: &mut u64) -> f64 {
    let mut x = *s;
    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
    *s = x;
    (x >> 11) as f64 / (1u64 << 53) as f64
}

const FUNKEN_FARBEN: [i64; 4] = [0xFFD54A, 0xFF9F1C, 0xFFF3C4, 0xFF6B3D];
const KONFETTI_FARBEN: [i64; 6] = [0xE74C3C, 0xF1C40F, 0x2ECC71, 0x3498DB, 0x9B59B6, 0xFF7AB6];
const STERN_FARBEN: [i64; 3] = [0xFFFFFF, 0xFFE9A8, 0xBDE4FF];

impl Gui {
    // ================================================================ Anlaesse
    /// Nach GUI_UPDATE: welches Widget hat gerade einen Anlass (Klick,
    /// Hineinfahren, Druck, Fokus), und will seine Stildatei dazu einen Klang
    /// oder einen Effekt?
    pub(super) fn effekte_anlass_pass(&mut self) {
        let druck = self.press_origin;
        let fokus = self.focus_widget;
        let mut jetzt_ueber: Vec<(usize, usize)> = Vec::new();
        let mut wuensche: Vec<(usize, usize, usize)> = Vec::new();
        for (wi, win) in self.windows.iter().enumerate() {
            if !win.alive { continue; }
            for (i, w) in win.widgets.iter().enumerate() {
                if w.hovered { jetzt_ueber.push((wi, i)); }
                let Some(s) = w.stil_auf.as_ref() else { continue };
                if !s.iter().any(|x| x.klang.is_some() || x.effekt.is_some()) { continue; }
                if w.clicked { wuensche.push((wi, i, stil::BASIS)); }
                if w.hovered && !self.vorher_ueber.contains(&(wi, i)) { wuensche.push((wi, i, stil::HOVER)); }
                if druck == Some((wi, i)) && self.vorher_druck != druck { wuensche.push((wi, i, stil::AKTIV)); }
                if fokus == Some((wi, i)) && self.vorher_fokus != fokus { wuensche.push((wi, i, stil::FOKUS)); }
            }
        }
        self.vorher_ueber = jetzt_ueber;
        self.vorher_druck = druck;
        self.vorher_fokus = fokus;
        for (wi, i, schicht) in wuensche {
            let (klang, effekt) = {
                let s = &self.windows[wi].widgets[i].stil_auf.as_ref().unwrap()[schicht];
                (s.klang.clone(), s.effekt)
            };
            if let Some((name, laut)) = klang.filter(|k| !k.0.is_empty()) { self.klang_wuenschen(name, laut); }
            if let Some(e) = effekt.filter(|&e| e > 0) { self.funken_aus(wi, i, e, 0); }
        }
    }

    // ================================================================ Klang
    fn klang_wuenschen(&mut self, name: String, laut: i32) {
        if !self.klang_an { return; }
        self.klang_log.push(name.clone());
        if self.klang_log.len() > 200 { self.klang_log.remove(0); }
        self.klang_warte.push((name, laut));
    }
    /// Fuer die VM: was nach diesem GUI_UPDATE zu spielen ist.
    pub fn klaenge_abholen(&mut self) -> Vec<(String, i32)> { std::mem::take(&mut self.klang_warte) }

    /// GUI_SOUND(name$ [, lautstaerke%]): einen Klang der gui spielen.
    pub fn sound(&mut self, name: &str, laut: i64) -> Result<(), String> {
        let (n, _) = klang_lesen(name, 0).map_err(|e| format!("GUI_SOUND: {}", e.trim_start_matches("Zeile 0: ")))?;
        if n.is_empty() { return Ok(()); }
        // 0..100 % wie die Lautstaerke 0..1 aller Ton-Befehle (bis 2026.53 0..200,
        // ueber 100 ohne Wirkung).
        if !(0..=100).contains(&laut) { return Err(format!("GUI_SOUND: Lautstaerke {} muss 0..100 sein", laut)); }
        self.klang_wuenschen(n, laut as i32);
        Ok(())
    }
    /// GUI_SET_SOUNDS(an) / GUI_SOUNDS().
    pub fn set_sounds(&mut self, an: bool) { self.klang_an = an; if !an { self.klang_warte.clear(); } }
    pub fn sounds(&self) -> bool { self.klang_an }
    /// GUI_SOUND_LOG$(): die gewuenschten Klaenge seit dem letzten Aufruf,
    /// durch Komma getrennt.
    pub fn sound_log(&mut self) -> String { std::mem::take(&mut self.klang_log).join(",") }

    // ================================================================ Funken
    /// GUI_BURST(wdg, art$ [, anzahl]).
    pub fn burst(&mut self, h: i64, art: &str, anzahl: Option<i64>) -> Result<(), String> {
        let f = "GUI_BURST";
        self.wdg(h, f)?;
        let e = effekt_lesen(art, 0).map_err(|e| format!("{}: {}", f, e.trim_start_matches("Zeile 0: ")))?;
        if e == 0 { return Ok(()); }
        let n = anzahl.unwrap_or(0);
        if !(0..=500).contains(&n) { return Err(format!("{}: anzahl {} muss 1..500 sein (0 = Vorgabe)", f, n)); }
        let (wi, i) = Self::dec_widget(h);
        self.funken_aus(wi, i, e, n as usize);
        Ok(())
    }
    /// GUI_BURSTING(): wie viele Teilchen gerade fliegen.
    pub fn bursting(&self) -> i64 { self.funken.len() as i64 }

    /// Teilchen an einem Widget entstehen lassen (0 = Vorgabe-Anzahl).
    fn funken_aus(&mut self, wi: usize, i: usize, art: u8, anzahl: usize) {
        if self.ruhig { return; }
        let (x, y, b, h) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let (cx, cy) = (x as f64 + b as f64 / 2.0, y as f64 + h as f64 / 2.0);
        let s = self.scale.max(0.5);
        let n = if anzahl > 0 { anzahl } else { match art { 1 => 26, 2 => 44, _ => 14 } };
        let mut z = self.funken_saat;
        for k in 0..n {
            let r = |z: &mut u64| zufall(z);
            let teil = match art {
                1 => {
                    let w = r(&mut z) * std::f64::consts::TAU;
                    let v = (140.0 + 160.0 * r(&mut z)) * s;
                    Funke { x: cx, y: cy, vx: w.cos() * v, vy: w.sin() * v - 60.0 * s, alter: 0.0,
                        leben: 0.35 + 0.3 * r(&mut z), farbe: FUNKEN_FARBEN[k % 4], groesse: (2.0 + r(&mut z) * 1.5) * s,
                        art: 1, dreh: 0.0, dreh_v: 0.0 }
                }
                2 => {
                    let w = -std::f64::consts::FRAC_PI_2 + (r(&mut z) - 0.5) * 1.9;
                    let v = (260.0 + 220.0 * r(&mut z)) * s;
                    Funke { x: x as f64 + b as f64 * r(&mut z), y: y as f64, vx: w.cos() * v, vy: w.sin() * v, alter: 0.0,
                        leben: 1.2 + 0.6 * r(&mut z), farbe: KONFETTI_FARBEN[k % 6], groesse: (3.0 + r(&mut z) * 2.0) * s,
                        art: 2, dreh: r(&mut z) * 6.0, dreh_v: 6.0 + 10.0 * r(&mut z) }
                }
                _ => {
                    let w = (k as f64 / n as f64) * std::f64::consts::TAU + r(&mut z) * 0.4;
                    let v = (70.0 + 50.0 * r(&mut z)) * s;
                    Funke { x: cx + w.cos() * b as f64 * 0.3, y: cy + w.sin() * h as f64 * 0.3, vx: w.cos() * v, vy: w.sin() * v,
                        alter: 0.0, leben: 0.6 + 0.3 * r(&mut z), farbe: STERN_FARBEN[k % 3], groesse: (3.0 + r(&mut z) * 3.0) * s,
                        art: 3, dreh: r(&mut z) * 3.0, dreh_v: 3.0 }
                }
            };
            self.funken.push(teil);
        }
        self.funken_saat = z;
        // Eine Obergrenze, damit ein Dauerfeuer nicht alles verlangsamt.
        let zu_viel = self.funken.len().saturating_sub(2000);
        if zu_viel > 0 { self.funken.drain(..zu_viel); }
    }

    /// Je Bild bewegen (dt = DELTA der gui).
    pub(super) fn funken_pass(&mut self, dt: f64) {
        if self.funken.is_empty() { return; }
        if self.ruhig { self.funken.clear(); return; }
        let s = self.scale.max(0.5);
        for f in self.funken.iter_mut() {
            f.alter += dt;
            let (schwere, bremse) = match f.art { 1 => (420.0, 1.6), 2 => (520.0, 1.2), _ => (0.0, 2.2) };
            f.vy += schwere * s * dt;
            let k = (1.0 - bremse * dt).max(0.0);
            f.vx *= k;
            if f.art != 2 { f.vy *= k; } else { f.vy = f.vy.min(170.0 * s); }
            f.x += f.vx * dt;
            f.y += f.vy * dt;
            f.dreh += f.dreh_v * dt;
        }
        self.funken.retain(|f| f.alter < f.leben);
    }

    /// Zeichnen, ganz oben auf der Hauptflaeche.
    pub(super) fn funken_zeichnen(&self, g: &mut Graphics) {
        for f in &self.funken {
            let rest = 1.0 - f.alter / f.leben;
            let a = if f.art == 2 { (rest * 3.0).min(1.0) } else { rest };
            let c = deckkraft(f.farbe, a);
            let (x, y) = (f.x.round() as i32, f.y.round() as i32);
            match f.art {
                1 => {
                    // Ein Strich entlang der Bewegung -- ein Funke zieht einen Schweif.
                    let l = 0.035;
                    let (x0, y0) = ((f.x - f.vx * l).round() as i32, (f.y - f.vy * l).round() as i32);
                    g.line(x0, y0, x, y, c);
                    let r = (f.groesse * rest).round().max(1.0) as i32;
                    g.circle(x, y, r, c);
                }
                2 => {
                    // Ein Schnipsel, das sich dreht: seine Breite schwankt.
                    let bw = (f.groesse * f.dreh.cos().abs()).max(1.0);
                    let bh = f.groesse * 1.7;
                    g.box_fill((f.x - bw / 2.0) as i32, (f.y - bh / 2.0) as i32, (f.x + bw / 2.0) as i32, (f.y + bh / 2.0) as i32, c);
                }
                _ => {
                    // Ein Stern aus zwei Kreuzen, das schraege kuerzer.
                    let r = (f.groesse * (0.6 + 0.4 * (f.dreh * 2.0).sin().abs())).max(1.5);
                    let ri = r as i32;
                    let rk = (r * 0.55) as i32;
                    g.line(x - ri, y, x + ri, y, c);
                    g.line(x, y - ri, x, y + ri, c);
                    g.line(x - rk, y - rk, x + rk, y + rk, c);
                    g.line(x - rk, y + rk, x + rk, y - rk, c);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn klang_und_effekt_lesen() {
        assert_eq!(klang_lesen("klick", 1).unwrap(), ("klick".to_string(), 100));
        assert_eq!(klang_lesen("Ping 40%", 1).unwrap(), ("ping".to_string(), 40));
        assert_eq!(klang_lesen("toene/tusch.wav", 1).unwrap(), ("toene/tusch.wav".to_string(), 100));
        assert_eq!(klang_lesen("keiner", 1).unwrap().0, "");
        assert!(klang_lesen("quietsch", 3).unwrap_err().contains("Zeile 3"));
        assert!(klang_lesen("klick 300%", 1).is_err());
        assert_eq!(effekt_lesen("Konfetti", 1).unwrap(), 2);
        assert_eq!(effekt_lesen("keiner", 1).unwrap(), 0);
        assert!(effekt_lesen("feuerwerk", 2).is_err());
    }

    #[test]
    fn zufall_bleibt_im_bereich_und_wiederholt_sich() {
        let (mut a, mut b) = (0x9E37_79B9_7F4A_7C15u64, 0x9E37_79B9_7F4A_7C15u64);
        for _ in 0..1000 {
            let x = zufall(&mut a);
            assert!((0.0..1.0).contains(&x));
            assert_eq!(x, zufall(&mut b));
        }
    }
}
