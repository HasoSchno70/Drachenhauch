//! Eigene Formen, Bildhaeute und Animationen (docs/intern/entwurf-gui-formen.md,
//! Stufe 1).
//!
//! Teil von `gui.rs`. Die Stildatei setzt `form`, `bild`/`bildrand` und
//! `animation` (gui_stil.rs liest sie); hier steht, was daraus wird:
//! - **Form** gilt fuer Zeichnen UND Treffer -- eine Quelle (`form_trifft`),
//!   die Ueberfahren, Druecken, Loslassen und GUI_HIT_TEST fragen
//!   (`trifft`). Ein Klick neben dem Kreis trifft nicht.
//! - **Bildhaut**: statt Flaeche und Rand zeichnet ein Knopf ein Bild (mit
//!   `bildrand` als 9-Slice), je Zustand ein eigenes. Die Bilder laedt
//!   `aussehen_pass` beim ersten Gebrauch (dort gibt es Graphics); fuer
//!   `form: bild` haelt es die Deckkraft des Bildes fuer den Treffer.
//! - **Animation**: eine Matrix um die Mitte des Widgets beim Zeichnen
//!   (`anim_matrix`) -- Lage und Treffer bleiben, wo sie sind. Die Uhr laeuft
//!   mit DELTA(); GUI_ANIMATE startet eine eigene, die der Stildatei folgt
//!   dem Zustand (`button:hover { animation: puls }`).

use super::*;

/// Formen (`form`): 0 = keine Angabe (Rechteck).
pub(super) const FORMEN: [&str; 6] = ["", "rechteck", "pille", "kreis", "raute", "bild"];
pub(super) const F_PILLE: u8 = 2;
pub(super) const F_KREIS: u8 = 3;
pub(super) const F_RAUTE: u8 = 4;
pub(super) const F_BILD: u8 = 5;

/// Animationsarten.
pub(super) const ARTEN: [&str; 6] = ["", "puls", "wackeln", "schweben", "huepfen", "drehen"];

/// Eine Animation aus der Stildatei oder GUI_ANIMATE: Art, Dauer einer
/// Runde in ms, Anzahl Runden (0 = ohne Ende).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AnimStil { pub art: u8, pub dauer: i32, pub anzahl: i32 }

/// Was ein Widget an eigenem Aussehen traegt -- nur bei Bedarf angelegt.
#[derive(Clone, Debug, Default)]
pub(super) struct Aussehen {
    /// GUI_SET_SHAPE (0 = die Stildatei entscheidet).
    pub form: u8,
    /// GUI_ANIMATE: die Animation und ihr Beginn (Uhr der gui).
    pub lauf: Option<(AnimStil, f64)>,
    /// Die Animation der Stildatei, die gerade gilt, und seit wann.
    pub stil_lauf: Option<(AnimStil, f64)>,
    /// Gleiten (`gleiten` in der Stildatei): die zuletzt gesehene Lage im
    /// Fenster (x, y, b, h), ob es sichtbar war, und ein laufendes Gleiten
    /// (Startrechteck, Beginn, Dauer in ms).
    pub lage: Option<[f64; 4]>,
    pub sichtbar: bool,
    pub gleit: Option<([f64; 4], f64, f64)>,
}

/// Langsam auslaufen (ease-out, kubisch).
fn auslaufen(p: f64) -> f64 { let q = 1.0 - p.clamp(0.0, 1.0); 1.0 - q * q * q }

/// Wo steht ein gleitendes Widget gerade (Rechteck im Fenster)?
pub(super) fn gleit_rechteck(von: [f64; 4], ziel: [f64; 4], t0: f64, dauer: f64, uhr: f64) -> [f64; 4] {
    let p = auslaufen((uhr - t0) * 1000.0 / dauer.max(1.0));
    [0, 1, 2, 3].map(|k| von[k] + (ziel[k] - von[k]) * p)
}

/// Eine geladene Bildhaut: ihre Einzelbilder (eines bei PNG & Co., alle
/// eines animierten GIFs), deren Groesse und Dauer.
pub(super) struct Haut { pub bilder: Vec<HautBild>, pub b: u32, pub h: u32, pub dauern: Vec<f64> }
/// Ein Einzelbild: Textur und Deckkraft je Pixel (fuer `form: bild`).
pub(super) struct HautBild { pub tex: i64, pub alpha: Vec<u8> }

/// Dauer eines GIF-Einzelbilds in ms; 0 und sehr kurze Angaben zeigen
/// Browser als 100 ms, ebenso hier (sonst flimmerten alte GIFs).
pub(super) fn gif_dauer(ms: i64) -> f64 { if ms <= 10 { 100.0 } else { ms as f64 } }

/// Welches Einzelbild gilt zur Uhr (Sekunden)? Alle Bildhaeute laufen
/// mit derselben Uhr der gui, ohne Ende, wie ein GIF im Browser.
pub(super) fn bild_zur_zeit(dauern: &[f64], uhr: f64) -> usize {
    let gesamt: f64 = dauern.iter().sum();
    if dauern.len() < 2 || gesamt <= 0.0 { return 0; }
    let mut t = (uhr * 1000.0).rem_euclid(gesamt);
    for (k, d) in dauern.iter().enumerate() {
        if t < *d { return k; }
        t -= d;
    }
    dauern.len() - 1
}

/// Eine Animation als Text (`puls 700ms immer`) -- fuer den Inspektor.
pub(super) fn anim_text(a: AnimStil) -> String {
    let n = if a.anzahl == 0 { "immer".to_string() } else if a.anzahl == 1 { "einmal".to_string() } else { a.anzahl.to_string() };
    let art = if a.art >= EIGEN { format!("@ablauf{}", a.art - EIGEN + 1) } else { ARTEN.get(a.art as usize).unwrap_or(&"?").to_string() };
    format!("{} {}ms {}", art, a.dauer, n)
}

/// Eigene Ablaeufe (`@ablauf name { ... }`) zaehlen als Art ab hier: Art
/// `EIGEN + k` ist der k-te Ablauf der Stildatei.
pub(super) const EIGEN: u8 = 100;

/// Ein Schluesselbild eines eigenen Ablaufs: wo (0..1) und was dann gilt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Schluessel { pub p: f64, pub x: f64, pub y: f64, pub drehen: f64, pub groesse: f64 }

/// Den Rumpf eines `@ablauf` lesen: `0%: x 0, y -8; 50%: drehen 10, groesse 1.1`.
/// Fehlt 0 % oder 100 %, gilt dort die Ruhe.
pub(super) fn ablauf_lesen(rumpf: &str, kopf_zeile: usize, zeile_bei: impl Fn(usize) -> usize) -> Result<Vec<Schluessel>, String> {
    let mut out: Vec<Schluessel> = Vec::new();
    let mut off = 0usize;
    for decl in rumpf.split(|c| c == ';' || c == '\n') {
        let zeile = zeile_bei(off);
        off += decl.len() + 1;
        let d = decl.trim();
        if d.is_empty() { continue; }
        let (wann, was) = d.split_once(':').ok_or_else(|| format!("Zeile {}: '{}' -- erwartet z.B. 50%: y -8, drehen 10", zeile, d))?;
        let p = wann.trim().strip_suffix('%').and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| (0.0..=100.0).contains(v))
            .ok_or_else(|| format!("Zeile {}: '{}' -- ein Schluesselbild steht bei 0% bis 100%", zeile, wann.trim()))?;
        let mut k = Schluessel { p: p / 100.0, x: 0.0, y: 0.0, drehen: 0.0, groesse: 1.0 };
        for teil in was.split(',') {
            let t: Vec<&str> = teil.split_whitespace().collect();
            if t.is_empty() { continue; }
            let wert = t.get(1).and_then(|v| v.parse::<f64>().ok())
                .ok_or_else(|| format!("Zeile {}: '{}' -- erwartet name zahl (x, y, drehen, groesse)", zeile, teil.trim()))?;
            match t[0].to_lowercase().as_str() {
                "x" => k.x = wert,
                "y" => k.y = wert,
                "drehen" | "rotate" => k.drehen = wert,
                "groesse" | "scale" => k.groesse = wert,
                n => return Err(format!("Zeile {}: '{}' gibt es in einem Ablauf nicht -- x, y, drehen, groesse", zeile, n)),
            }
        }
        out.push(k);
    }
    if out.is_empty() { return Err(format!("Zeile {}: @ablauf ohne Schluesselbilder", kopf_zeile)); }
    out.sort_by(|a, b| a.p.partial_cmp(&b.p).unwrap_or(std::cmp::Ordering::Equal));
    let ruhe = |p: f64| Schluessel { p, x: 0.0, y: 0.0, drehen: 0.0, groesse: 1.0 };
    if out[0].p > 0.0 { out.insert(0, ruhe(0.0)); }
    if out.last().map(|k| k.p < 1.0).unwrap_or(true) { out.push(ruhe(1.0)); }
    Ok(out)
}

/// Ein eigener Ablauf bei p (0..1): zwischen den Schluesselbildern gerade
/// verbunden. (dx, dy, Winkel, Massstab) in logischen Punkten.
pub(super) fn ablauf_verformung(ks: &[Schluessel], p: f64) -> (f64, f64, f64, f64) {
    let p = p.clamp(0.0, 1.0);
    let i = ks.iter().rposition(|k| k.p <= p).unwrap_or(0);
    let a = ks[i];
    let b = *ks.get(i + 1).unwrap_or(&a);
    let t = if b.p > a.p { (p - a.p) / (b.p - a.p) } else { 0.0 };
    let m = |u: f64, v: f64| u + (v - u) * t;
    (m(a.x, b.x), m(a.y, b.y), m(a.drehen, b.drehen), m(a.groesse, b.groesse))
}

/// `bild: datei.png [anzahl [Nfps]]` lesen -- mit Anzahl ist es eine
/// Bildfolge (Atlas: die Bilder nebeneinander), gespeichert als
/// `datei|anzahl|fps`. Ein Pfad mit Leerzeichen steht in Anfuehrungszeichen.
pub(super) fn bild_lesen(wert: &str, zeile: usize) -> Result<Option<String>, String> {
    let w = wert.trim();
    let (pfad, rest) = if let Some(r) = w.strip_prefix('"') {
        match r.find('"') { Some(e) => (r[..e].to_string(), r[e + 1..].trim().to_string()), None => (r.to_string(), String::new()) }
    } else {
        let t: Vec<&str> = w.split_whitespace().collect();
        let zahl = |x: &str| x.trim_end_matches("fps").parse::<u32>().is_ok();
        let n = t.iter().rev().take(2).take_while(|x| zahl(x)).count();
        (t[..t.len() - n].join(" "), t[t.len() - n..].join(" "))
    };
    let pfad = pfad.trim().trim_matches('\'').to_string();
    if pfad.is_empty() || pfad == "keins" || pfad == "none" { return Ok(None); }
    if rest.is_empty() { return Ok(Some(pfad)); }
    let teile: Vec<&str> = rest.split_whitespace().collect();
    let anzahl = teile[0].parse::<u32>().ok().filter(|n| (1..=256).contains(n))
        .ok_or_else(|| format!("Zeile {}: bild -- Anzahl der Bilder '{}' muss 1..256 sein", zeile, teile[0]))?;
    let fps = match teile.get(1) {
        Some(f) => f.trim_end_matches("fps").parse::<u32>().ok().filter(|n| (1..=120).contains(n))
            .ok_or_else(|| format!("Zeile {}: bild -- '{}' muss 1fps..120fps sein", zeile, f))?,
        None => 12,
    };
    Ok(Some(format!("{}|{}|{}", pfad, anzahl, fps)))
}

/// `animation: art [dauer] [einmal|N]` lesen; `eigene` sind die Namen der
/// `@ablauf`-Bloecke der Stildatei.
pub(super) fn anim_lesen(wert: &str, zeile: usize, eigene: &[String]) -> Result<AnimStil, String> {
    let mut art = 0u8;
    let mut dauer = 0i32;
    let mut anzahl = 0i32;
    for t in wert.split_whitespace() {
        let tl = t.to_lowercase();
        if let Some(a) = ARTEN.iter().position(|x| !x.is_empty() && *x == tl) { art = a as u8; continue; }
        if let Some(k) = eigene.iter().position(|x| *x == tl).filter(|&k| k < 100) { art = EIGEN + k as u8; continue; }
        if tl == "einmal" || tl == "once" { anzahl = 1; continue; }
        if tl == "immer" || tl == "infinite" { anzahl = 0; continue; }
        if let Some(ms) = tl.strip_suffix("ms") {
            dauer = ms.parse::<i32>().map_err(|_| format!("Zeile {}: animation: '{}' ist keine Dauer", zeile, t))?;
            continue;
        }
        if let Ok(n) = tl.parse::<i32>() { anzahl = n.max(0); continue; }
        if tl == "keine" || tl == "none" { art = 0; continue; }
        return Err(format!("Zeile {}: animation: '{}' unbekannt -- puls, wackeln, schweben, huepfen, drehen, ein @ablauf der Stildatei, Dauer in ms, einmal oder eine Anzahl", zeile, t));
    }
    if dauer == 0 { dauer = vorgabe_dauer(art); }
    if !(50..=20000).contains(&dauer) { return Err(format!("Zeile {}: animation: Dauer {} ms ausserhalb 50..20000", zeile, dauer)); }
    Ok(AnimStil { art, dauer, anzahl })
}

/// Ist eine endliche Animation (Beginn `t0`) zur Uhr `uhr` vorbei? EINE
/// Stelle fuer Zeichnen, GUI_ANIMATING und das Aufraeumen.
pub(super) fn vorbei(a: AnimStil, t0: f64, uhr: f64) -> bool {
    a.anzahl > 0 && (uhr - t0) * 1000.0 >= (a.dauer as f64) * (a.anzahl as f64)
}

fn vorgabe_dauer(art: u8) -> i32 { match art { 2 => 400, 4 => 600, 5 => 1200, a if a >= EIGEN => 1000, _ => 900 } }

/// Die Verformung einer Animation zur Phase `p` (0..1 einer Runde) und
/// dem Anteil `rest` (1 am Anfang, 0 am Ende einer endlichen Animation):
/// (dx, dy, Winkel in Grad, Massstab) in logischen Punkten.
pub(super) fn verformung(art: u8, p: f64, rest: f64) -> (f64, f64, f64, f64) {
    use std::f64::consts::PI;
    let s = (2.0 * PI * p).sin();
    match art {
        1 => (0.0, 0.0, 0.0, 1.0 + 0.06 * 0.5 * (1.0 - (2.0 * PI * p).cos())),
        2 => (0.0, 0.0, 7.0 * s * rest.max(0.0), 1.0),
        3 => (0.0, -4.0 * s, 0.0, 1.0),
        4 => (0.0, -9.0 * (PI * p).sin().abs(), 0.0, 1.0),
        5 => (0.0, 0.0, 360.0 * p, 1.0),
        _ => (0.0, 0.0, 0.0, 1.0),
    }
}

impl Gui {
    // ================================================================ Abfragen
    /// Die Form eines Widgets: eigene (GUI_SET_SHAPE), sonst die Stildatei.
    pub(super) fn w_form_art(&self, w: &Widget) -> u8 {
        if let Some(f) = w.aussehen.as_ref().map(|a| a.form).filter(|&f| f != 0) { return f; }
        w.stil_auf.as_ref().and_then(|s| s[stil::BASIS].form).unwrap_or(0)
    }

    /// Die Bildhaut fuer den Zustand, in dem das Widget gerade ist: (Pfad,
    /// Rand). Gesperrt vor gedrueckt vor ueberfahren vor Grund.
    pub(super) fn w_bild(&self, w: &Widget, gedrueckt: bool) -> Option<(String, i32)> {
        let s = w.stil_auf.as_ref()?;
        let schicht = |k: usize| s[k].bild.as_ref().map(|b| (b.clone(), s[k].bildrand.or(s[stil::BASIS].bildrand).unwrap_or(0)));
        if !w.enabled { if let Some(b) = schicht(stil::AUS) { return Some(b); } }
        if gedrueckt { if let Some(b) = schicht(stil::AKTIV) { return Some(b); } }
        if w.hovered || w.ueber_t > 0.5 { if let Some(b) = schicht(stil::HOVER) { return Some(b); } }
        schicht(stil::BASIS)
    }

    /// Die Animation der Stildatei fuer den jetzigen Zustand.
    fn w_stil_anim(&self, w: &Widget, gedrueckt: bool) -> Option<AnimStil> {
        let s = w.stil_auf.as_ref()?;
        let a = |k: usize| s[k].animation.filter(|a| a.art != 0);
        if !w.enabled { return a(stil::AUS); }
        if gedrueckt { if let Some(x) = a(stil::AKTIV) { return Some(x); } }
        if w.hovered { if let Some(x) = a(stil::HOVER) { return Some(x); } }
        a(stil::BASIS)
    }

    /// Trifft (mx, my) das Widget -- sein Rechteck UND seine Form? Die eine
    /// Stelle fuer Ueberfahren, Druecken, Loslassen und GUI_HIT_TEST.
    pub(super) fn trifft(&self, wi: usize, w: &Widget, mx: i32, my: i32) -> bool {
        let r = self.abs_rect(wi, w);
        Self::in_rect(mx, my, r) && self.form_trifft(w, r, mx, my)
    }

    /// Liegt der Punkt in der Form (das Rechteck ist schon geprueft)?
    pub(super) fn form_trifft(&self, w: &Widget, r: (i32, i32, i32, i32), mx: i32, my: i32) -> bool {
        if w.kind != Kind::Button { return true; }
        let (x, y, b, h) = r;
        if b <= 0 || h <= 0 { return false; }
        let (px, py) = ((mx - x) as f64 + 0.5, (my - y) as f64 + 0.5);
        let (cx, cy) = (b as f64 / 2.0, h as f64 / 2.0);
        match self.w_form_art(w) {
            F_PILLE => {
                let rr = cx.min(cy);
                let qx = (px - cx).abs() - (cx - rr);
                let qy = (py - cy).abs() - (cy - rr);
                qx <= 0.0 || qy <= 0.0 || qx * qx + qy * qy <= rr * rr
            }
            F_KREIS => {
                let rr = cx.min(cy);
                (px - cx).powi(2) + (py - cy).powi(2) <= rr * rr
            }
            F_RAUTE => (px - cx).abs() / cx + (py - cy).abs() / cy <= 1.0,
            F_BILD => {
                let Some((pfad, _)) = self.w_bild(w, false) else { return true };
                let Some(Some(haut)) = self.haeute.get(&pfad) else { return true };
                let bild = &haut.bilder[bild_zur_zeit(&haut.dauern, self.haut_uhr())];
                let ix = ((px / b as f64) * haut.b as f64).floor().clamp(0.0, haut.b as f64 - 1.0) as usize;
                let iy = ((py / h as f64) * haut.h as f64).floor().clamp(0.0, haut.h as f64 - 1.0) as usize;
                bild.alpha.get(iy * haut.b as usize + ix).map(|&a| a >= 32).unwrap_or(true)
            }
            _ => true,
        }
    }

    // ================================================================ je Bild
    /// In GUI_UPDATE: die Uhr weiterstellen, Bildhaeute laden, die
    /// Animation der Stildatei je Widget nachfuehren und abgelaufene
    /// beenden.
    pub(super) fn aussehen_pass(&mut self, g: &mut Graphics) {
        self.uhr += g.delta().clamp(0.0, 0.25);
        let uhr = self.uhr;
        // Bildhaeute: jeder Pfad, den eine Regel nennt, einmal laden.
        if self.stil.is_some() {
            let mut fehlt: Vec<String> = Vec::new();
            for win in &self.windows {
                for w in &win.widgets {
                    let Some(s) = w.stil_auf.as_ref() else { continue };
                    for schicht in s.iter() {
                        if let Some(p) = schicht.bild.as_ref() {
                            if !self.haeute.contains_key(p) && !fehlt.contains(p) { fehlt.push(p.clone()); }
                        }
                    }
                }
            }
            for p in fehlt {
                let haut = Self::haut_laden(g, &p);
                self.haeute.insert(p, haut);
            }
        }
        let gedrueckt = self.press_origin;
        // Sichtbar je Widget -- vorab, die Schleife unten haelt die Fenster
        // veraenderlich.
        let gezeigt: Vec<Vec<bool>> = self.windows.iter().enumerate()
            .map(|(wi, win)| win.widgets.iter().map(|w| w.alive && self.widget_shown(wi, w)).collect()).collect();
        let ruhig = self.ruhig;
        for (wi, win) in self.windows.iter_mut().enumerate() {
            for (i, w) in win.widgets.iter_mut().enumerate() {
                if w.aussehen.is_none() && w.stil_auf.is_none() { continue; }
                // Gleiten: hat sich die Lage geaendert, gleitet das Bild vom
                // alten Rechteck (oder von da, wo es gerade gleitet) zum neuen;
                // wird es sichtbar, waechst es aus 92 % auf.
                let ms = w.stil_auf.as_ref().and_then(|s| s[stil::BASIS].gleiten).unwrap_or(0);
                if ms > 0 || w.aussehen.as_ref().map(|a| a.lage.is_some()).unwrap_or(false) {
                    let jetzt_lage = [w.x as f64, w.y as f64, w.w as f64, w.h as f64];
                    let zu_sehen = gezeigt[wi][i];
                    let au = w.aussehen.get_or_insert_with(Default::default);
                    if ms <= 0 || ruhig {
                        au.gleit = None;
                    } else if let Some(alt) = au.lage {
                        let bewegt = (0..4).any(|k| (alt[k] - jetzt_lage[k]).abs() > 0.5);
                        if bewegt {
                            let von = match au.gleit { Some((v, t0, d)) => gleit_rechteck(v, alt, t0, d, uhr), None => alt };
                            au.gleit = Some((von, uhr, ms as f64));
                        } else if zu_sehen && !au.sichtbar {
                            let [x, y, b, h] = jetzt_lage;
                            au.gleit = Some(([x + b * 0.04, y + h * 0.04, b * 0.92, h * 0.92], uhr, ms as f64));
                        }
                    }
                    if let Some((_, t0, d)) = au.gleit { if (uhr - t0) * 1000.0 >= d { au.gleit = None; } }
                    au.lage = Some(jetzt_lage);
                    au.sichtbar = zu_sehen;
                }
                let druck = gedrueckt == Some((wi, i));
                let jetzt = {
                    let s = w.stil_auf.as_ref();
                    let a = |k: usize| s.and_then(|s| s[k].animation).filter(|a| a.art != 0);
                    if !w.enabled { a(stil::AUS) }
                    else { (if druck { a(stil::AKTIV) } else { None }).or_else(|| if w.hovered { a(stil::HOVER) } else { None }).or_else(|| a(stil::BASIS)) }
                };
                let alt = w.aussehen.as_ref().and_then(|a| a.stil_lauf.map(|l| l.0));
                if jetzt != alt {
                    let au = w.aussehen.get_or_insert_with(Default::default);
                    au.stil_lauf = jetzt.map(|a| (a, uhr));
                }
                if let Some(au) = w.aussehen.as_mut() {
                    if let Some((a, t0)) = au.lauf {
                        if vorbei(a, t0, uhr) { au.lauf = None; }
                    }
                }
            }
        }
    }

    /// Eine Bildhaut laden: ein GIF mit allen Einzelbildern und ihren
    /// Dauern (IMAGE_LOAD_GIF/IMAGE_GIF_DELAYS), alles andere als ein Bild.
    /// None = nicht lesbar (der Knopf zeichnet sich dann wie ohne Haut).
    fn haut_laden(g: &mut Graphics, pfad: &str) -> Option<Haut> {
        // Bildfolge aus einem Atlas (`bild: figur.png 8 12fps`, gespeichert als
        // `figur.png|8|12`): die Bilder stehen nebeneinander.
        let teile: Vec<&str> = pfad.split('|').collect();
        if teile.len() == 3 {
            let (n, fps) = (teile[1].parse::<u32>().ok()?, teile[2].parse::<u32>().ok()?);
            let tex = g.load_texture(teile[0]).ok()?;
            let (px, b, h) = g.image_rgba(tex, "bild").ok()?;
            let fb = b / n.max(1);
            if fb == 0 { return None; }
            let mut bilder = Vec::with_capacity(n as usize);
            for k in 0..n {
                let mut stueck = vec![0u8; (fb * h * 4) as usize];
                for z in 0..h {
                    let q = ((z * b + k * fb) * 4) as usize;
                    stueck[(z * fb * 4) as usize..((z + 1) * fb * 4) as usize].copy_from_slice(&px[q..q + (fb * 4) as usize]);
                }
                let t = g.textur_aus_rgba(&stueck, fb, h).ok()?;
                bilder.push(HautBild { tex: t, alpha: stueck.chunks(4).map(|c| c[3]).collect() });
            }
            return Some(Haut { bilder, b: fb, h, dauern: vec![1000.0 / fps.max(1) as f64; n as usize] });
        }
        let texturen = if pfad.to_ascii_lowercase().ends_with(".gif") {
            g.image_load_gif(pfad).ok()?
        } else {
            vec![g.load_texture(pfad).ok()?]
        };
        let mut bilder = Vec::with_capacity(texturen.len());
        let (mut b, mut h) = (0, 0);
        for tex in texturen {
            let (px, bb, hh) = g.image_rgba(tex, "bild").ok()?;
            (b, h) = (bb, hh);
            bilder.push(HautBild { tex, alpha: px.chunks(4).map(|c| c[3]).collect() });
        }
        if bilder.is_empty() { return None; }
        let mut dauern = vec![100.0; bilder.len()];
        if bilder.len() > 1 {
            let aufgeloest = crate::builtins::resolve_asset_path(pfad);
            if let Ok(d) = std::fs::read(&aufgeloest).map_err(|e| e.to_string()).and_then(|d| crate::gifschreiber::dauern(&d)) {
                for (k, ms) in d.into_iter().take(dauern.len()).enumerate() { dauern[k] = gif_dauer(ms); }
            }
        }
        Some(Haut { bilder, b, h, dauern })
    }

    /// Die Matrix fuer das Zeichnen eines Widgets: (dx, dy, Winkel,
    /// Massstab x, Massstab y) in Bildpunkten -- Animation und Gleiten
    /// zusammen; None = nichts laeuft.
    pub(super) fn anim_matrix(&self, wi: usize, idx: usize, w: &Widget) -> Option<(f32, f32, f32, f32, f32)> {
        if self.ruhig { return None; }
        let au = w.aussehen.as_ref()?;
        let druck = self.press_origin == Some((wi, idx));
        let anim = au.lauf.or_else(|| au.stil_lauf.filter(|l| Some(l.0) == self.w_stil_anim(w, druck)))
            .filter(|&(a, t0)| !vorbei(a, t0, self.uhr))
            .map(|(a, t0)| {
                let vergangen = (self.uhr - t0) * 1000.0;
                let gesamt = (a.dauer as f64) * (a.anzahl as f64);
                let p = (vergangen / a.dauer as f64).fract();
                let rest = if a.anzahl > 0 { 1.0 - vergangen / gesamt } else { 1.0 };
                let (dx, dy, ang, sc) = if a.art >= EIGEN {
                    // Eigener Ablauf der Stildatei -- steht er nicht (mehr) da, ruht es.
                    self.stil.as_ref().and_then(|d| d.ablaeufe.get((a.art - EIGEN) as usize))
                        .map(|(_, ks)| ablauf_verformung(ks, p)).unwrap_or((0.0, 0.0, 0.0, 1.0))
                } else { verformung(a.art, p, rest) };
                (self.sk_f(dx), self.sk_f(dy), ang as f32, sc as f32)
            });
        let gleit = au.gleit.filter(|&(_, t0, d)| (self.uhr - t0) * 1000.0 < d).map(|(von, t0, d)| {
            let ziel = [w.x as f64, w.y as f64, w.w as f64, w.h as f64];
            let r = gleit_rechteck(von, ziel, t0, d, self.uhr);
            let dx = (r[0] + r[2] / 2.0) - (ziel[0] + ziel[2] / 2.0);
            let dy = (r[1] + r[3] / 2.0) - (ziel[1] + ziel[3] / 2.0);
            let sx = if ziel[2] > 0.0 { r[2] / ziel[2] } else { 1.0 };
            let sy = if ziel[3] > 0.0 { r[3] / ziel[3] } else { 1.0 };
            (dx as f32, dy as f32, sx as f32, sy as f32)
        });
        match (anim, gleit) {
            (None, None) => None,
            (a, g) => {
                let (adx, ady, ang, sc) = a.unwrap_or((0.0, 0.0, 0.0, 1.0));
                let (gdx, gdy, gsx, gsy) = g.unwrap_or((0.0, 0.0, 1.0, 1.0));
                Some((adx + gdx, ady + gdy, ang, sc * gsx, sc * gsy))
            }
        }
    }
    /// GUI_GLIDING(wdg): gleitet das Widget gerade?
    pub fn gliding(&self, h: i64) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_GLIDING")?;
        Ok(w.aussehen.as_ref().and_then(|a| a.gleit).map(|(_, t0, d)| (self.uhr - t0) * 1000.0 < d).unwrap_or(false))
    }
    fn sk_f(&self, v: f64) -> f32 { (v * self.scale) as f32 }

    /// Laeuft gerade eine Animation (eigene oder aus der Stildatei)?
    fn animiert(&self, wi: usize, idx: usize, w: &Widget) -> bool { self.anim_matrix(wi, idx, w).is_some() }

    /// Die Uhr fuer GIF-Haeute: ohne Bewegung steht sie auf dem ersten Bild.
    fn haut_uhr(&self) -> f64 { if self.ruhig { 0.0 } else { self.uhr } }

    /// Eine Bildhaut zeichnen; FALSE, wenn das Bild (noch) fehlt.
    pub(super) fn haut_zeichnen(&self, g: &mut Graphics, pfad: &str, rand: i32, x: i32, y: i32, b: i32, h: i32) -> bool {
        let Some(Some(haut)) = self.haeute.get(pfad) else { return false };
        let tex = haut.bilder[bild_zur_zeit(&haut.dauern, self.haut_uhr())].tex;
        if rand > 0 { g.nine_slice(tex, x, y, b, h, rand).is_ok() } else { g.draw_image_rect(tex, x, y, b, h); true }
    }

    // ================================================================ Befehle
    /// GUI_SET_SHAPE(knopf, form$): rechteck, pille, kreis, raute, bild --
    /// fuer Zeichnen und Treffer; "" = wieder die Stildatei.
    pub fn set_shape(&mut self, h: i64, form: &str) -> Result<(), String> {
        let f = form.trim().to_lowercase();
        let n = FORMEN.iter().position(|x| *x == f)
            .ok_or_else(|| format!("GUI_SET_SHAPE: Form '{}' unbekannt -- rechteck, pille, kreis, raute, bild oder \"\"", form))?;
        let w = self.wdg_mut(h, "GUI_SET_SHAPE")?;
        if w.kind != Kind::Button { return Err("GUI_SET_SHAPE: nur fuer Knoepfe (GUI_BUTTON)".into()); }
        w.aussehen.get_or_insert_with(Default::default).form = n as u8;
        Ok(())
    }
    /// GUI_GET_SHAPE$(knopf): die Form, die gilt (eigene oder Stildatei).
    pub fn get_shape(&self, h: i64) -> Result<String, String> {
        let w = self.wdg(h, "GUI_GET_SHAPE$")?;
        let f = self.w_form_art(w) as usize;
        Ok(if f == 0 { "rechteck".into() } else { FORMEN[f].to_string() })
    }
    /// GUI_ANIMATE(wdg, art$ [, dauer_ms [, anzahl]]): eine Animation
    /// starten (anzahl 0 = ohne Ende, Vorgabe 1); art "" haelt an.
    pub fn animate(&mut self, h: i64, art: &str, dauer: Option<i64>, anzahl: Option<i64>) -> Result<(), String> {
        let a = art.trim().to_lowercase();
        let uhr = self.uhr;
        if a.is_empty() {
            if let Some(au) = self.wdg_mut(h, "GUI_ANIMATE")?.aussehen.as_mut() { au.lauf = None; }
            return Ok(());
        }
        let eigene: Vec<String> = self.stil.as_ref().map(|d| d.ablaeufe.iter().map(|(n, _)| n.clone()).collect()).unwrap_or_default();
        let n = ARTEN.iter().position(|x| !x.is_empty() && *x == a).map(|n| n as u8)
            .or_else(|| eigene.iter().position(|x| *x == a).filter(|&k| k < 100).map(|k| EIGEN + k as u8))
            .ok_or_else(|| format!("GUI_ANIMATE: '{}' unbekannt -- puls, wackeln, schweben, huepfen, drehen, ein @ablauf der Stildatei oder \"\"", art))?;
        let w = self.wdg_mut(h, "GUI_ANIMATE")?;
        let dauer = dauer.map(|d| d as i32).unwrap_or_else(|| vorgabe_dauer(n));
        if !(50..=20000).contains(&dauer) { return Err(format!("GUI_ANIMATE: Dauer {} ms ausserhalb 50..20000", dauer)); }
        let anzahl = anzahl.unwrap_or(1).clamp(0, 1_000_000) as i32;
        w.aussehen.get_or_insert_with(Default::default).lauf = Some((AnimStil { art: n, dauer, anzahl }, uhr));
        Ok(())
    }
    /// GUI_ANIMATING(wdg): laeuft gerade eine Animation?
    pub fn animating(&self, h: i64) -> Result<bool, String> {
        let (wi, i) = Self::dec_widget(h);
        let w = self.wdg(h, "GUI_ANIMATING")?;
        Ok(self.animiert(wi, i, w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_lesen() {
        assert_eq!(anim_lesen("puls", 1, &[]).unwrap(), AnimStil { art: 1, dauer: 900, anzahl: 0 });
        assert_eq!(anim_lesen("wackeln 300ms einmal", 1, &[]).unwrap(), AnimStil { art: 2, dauer: 300, anzahl: 1 });
        assert_eq!(anim_lesen("drehen 2000ms 3", 1, &[]).unwrap(), AnimStil { art: 5, dauer: 2000, anzahl: 3 });
        assert!(anim_lesen("tanzen", 1, &[]).is_err());
        assert!(anim_lesen("puls 10ms", 1, &[]).is_err());
    }

    #[test]
    fn gif_bilder_nach_der_uhr() {
        let d = [100.0, 200.0, 100.0];
        assert_eq!(bild_zur_zeit(&d, 0.0), 0);
        assert_eq!(bild_zur_zeit(&d, 0.099), 0);
        assert_eq!(bild_zur_zeit(&d, 0.1), 1);
        assert_eq!(bild_zur_zeit(&d, 0.299), 1);
        assert_eq!(bild_zur_zeit(&d, 0.35), 2);
        assert_eq!(bild_zur_zeit(&d, 0.45), 0); // von vorn
        assert_eq!(bild_zur_zeit(&[100.0], 5.0), 0);
        assert_eq!(gif_dauer(0), 100.0);
        assert_eq!(gif_dauer(40), 40.0);
    }

    #[test]
    fn ablaeufe_und_bildfolgen_lesen() {
        let ks = ablauf_lesen("50%: y -10, drehen 20\n100%: groesse 2", 1, |_| 1).unwrap();
        assert_eq!(ks.len(), 3, "0 % kommt als Ruhe dazu");
        assert_eq!(ablauf_verformung(&ks, 0.0), (0.0, 0.0, 0.0, 1.0));
        assert_eq!(ablauf_verformung(&ks, 0.5), (0.0, -10.0, 20.0, 1.0));
        assert_eq!(ablauf_verformung(&ks, 0.25), (0.0, -5.0, 10.0, 1.0));
        assert_eq!(ablauf_verformung(&ks, 1.0), (0.0, 0.0, 0.0, 2.0));
        assert!(ablauf_lesen("120%: x 1", 3, |_| 3).unwrap_err().contains("0% bis 100%"));
        assert!(ablauf_lesen("50%: farbe 1", 3, |_| 3).unwrap_err().contains("x, y, drehen, groesse"));
        assert_eq!(bild_lesen("figur.png 8 12fps", 1).unwrap(), Some("figur.png|8|12".to_string()));
        assert_eq!(bild_lesen("figur.png 4", 1).unwrap(), Some("figur.png|4|12".to_string()));
        assert_eq!(bild_lesen("\"mein bild.png\" 2 6fps", 1).unwrap(), Some("mein bild.png|2|6".to_string()));
        assert_eq!(bild_lesen("knopf.png", 1).unwrap(), Some("knopf.png".to_string()));
        assert_eq!(bild_lesen("keins", 1).unwrap(), None);
        assert!(bild_lesen("figur.png 0", 1).is_err());
        let a = anim_lesen("tanz 800ms", 1, &["tanz".to_string()]).unwrap();
        assert_eq!((a.art, a.dauer), (EIGEN, 800));
    }

    #[test]
    fn gleiten_laeuft_aus() {
        let von = [0.0, 0.0, 100.0, 20.0];
        let ziel = [50.0, 10.0, 200.0, 20.0];
        assert_eq!(gleit_rechteck(von, ziel, 1.0, 200.0, 1.0), von);
        assert_eq!(gleit_rechteck(von, ziel, 1.0, 200.0, 1.2), ziel);
        let halb = gleit_rechteck(von, ziel, 1.0, 200.0, 1.1);
        assert!(halb[0] > 25.0 && halb[0] < 50.0, "laeuft aus: nach der halben Zeit mehr als die Haelfte, {}", halb[0]);
    }

    #[test]
    fn verformungen_ruhen_am_anfang() {
        for art in 1..=5u8 {
            let (dx, dy, ang, sc) = verformung(art, 0.0, 1.0);
            assert!(dx.abs() < 1e-9 && dy.abs() < 1e-9 && ang.abs() < 1e-9 && (sc - 1.0).abs() < 1e-9, "art {}", art);
        }
        // Mitte einer Runde: puls am groessten, schweben oben ist eine Viertelrunde.
        assert!((verformung(1, 0.5, 1.0).3 - 1.06).abs() < 1e-9);
        assert!((verformung(3, 0.25, 1.0).1 + 4.0).abs() < 1e-9);
    }
}
