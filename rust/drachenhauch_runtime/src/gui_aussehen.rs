//! Wie Widgets aussehen: Farben und Thema (`wcol`, `GUI_THEME_*`, Metriken),
//! Schrift (`wtext`, `wsize`), Flaechen (`fbox`, Glanz, Fase, Verlauf), eigene
//! Formen, Bildhaeute und Animationen (docs/intern/entwurf-gui-formen.md).
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
pub(super) const ARTEN: [&str; 7] = ["", "puls", "wackeln", "schweben", "huepfen", "drehen", "blinken"];

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

/// Sekunden bis zum naechsten Bild einer Bildfolge (None = nur ein Bild).
pub(super) fn rest_bis_wechsel(dauern: &[f64], uhr: f64) -> Option<f64> {
    let gesamt: f64 = dauern.iter().sum();
    if dauern.len() < 2 || gesamt <= 0.0 { return None; }
    let mut t = (uhr * 1000.0).rem_euclid(gesamt);
    for d in dauern {
        if t < *d { return Some((d - t) / 1000.0); }
        t -= d;
    }
    Some(0.0)
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
pub(super) struct Schluessel { pub p: f64, pub x: f64, pub y: f64, pub drehen: f64, pub groesse: f64,
    /// Deckkraft 0..1 und Farbton (RGB, weiss = keiner) -- ueber eine Schicht
    /// (graphics.rs `Cmd::SchichtAnfang`).
    pub deckkraft: f64, pub farbe: [f64; 3] }

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
        let mut k = Schluessel { p: p / 100.0, x: 0.0, y: 0.0, drehen: 0.0, groesse: 1.0, deckkraft: 1.0, farbe: [255.0; 3] };
        for teil in was.split(',') {
            let t: Vec<&str> = teil.split_whitespace().collect();
            if t.is_empty() { continue; }
            if matches!(t[0].to_lowercase().as_str(), "farbe" | "color") {
                let h = t.get(1).and_then(|v| crate::farbraum::hex_lesen(v))
                    .ok_or_else(|| format!("Zeile {}: '{}' -- erwartet farbe #RGB oder #RRGGBB", zeile, teil.trim()))?;
                k.farbe = [((h >> 16) & 0xFF) as f64, ((h >> 8) & 0xFF) as f64, (h & 0xFF) as f64];
                continue;
            }
            let wert = t.get(1).and_then(|v| v.parse::<f64>().ok())
                .ok_or_else(|| format!("Zeile {}: '{}' -- erwartet name zahl (x, y, drehen, groesse, deckkraft)", zeile, teil.trim()))?;
            match t[0].to_lowercase().as_str() {
                "deckkraft" | "opacity" => k.deckkraft = wert.clamp(0.0, 1.0),
                "x" => k.x = wert,
                "y" => k.y = wert,
                "drehen" | "rotate" => k.drehen = wert,
                "groesse" | "scale" => k.groesse = wert,
                n => return Err(format!("Zeile {}: '{}' gibt es in einem Ablauf nicht -- x, y, drehen, groesse, deckkraft, farbe", zeile, n)),
            }
        }
        out.push(k);
    }
    if out.is_empty() { return Err(format!("Zeile {}: @ablauf ohne Schluesselbilder", kopf_zeile)); }
    out.sort_by(|a, b| a.p.partial_cmp(&b.p).unwrap_or(std::cmp::Ordering::Equal));
    let ruhe = |p: f64| Schluessel { p, x: 0.0, y: 0.0, drehen: 0.0, groesse: 1.0, deckkraft: 1.0, farbe: [255.0; 3] };
    if out[0].p > 0.0 { out.insert(0, ruhe(0.0)); }
    if out.last().map(|k| k.p < 1.0).unwrap_or(true) { out.push(ruhe(1.0)); }
    Ok(out)
}

/// Ein eigener Ablauf bei p (0..1): zwischen den Schluesselbildern gerade
/// verbunden. (dx, dy, Winkel, Massstab) in logischen Punkten.
pub(super) fn ablauf_verformung(ks: &[Schluessel], p: f64) -> (f64, f64, f64, f64) {
    let k = ablauf_bei(ks, p);
    (k.x, k.y, k.drehen, k.groesse)
}
/// Das Schluesselbild bei p, zwischen den Nachbarn gerade verbunden.
pub(super) fn ablauf_bei(ks: &[Schluessel], p: f64) -> Schluessel {
    let p = p.clamp(0.0, 1.0);
    let i = ks.iter().rposition(|k| k.p <= p).unwrap_or(0);
    let a = ks[i];
    let b = *ks.get(i + 1).unwrap_or(&a);
    let t = if b.p > a.p { (p - a.p) / (b.p - a.p) } else { 0.0 };
    let m = |u: f64, v: f64| u + (v - u) * t;
    Schluessel { p, x: m(a.x, b.x), y: m(a.y, b.y), drehen: m(a.drehen, b.drehen), groesse: m(a.groesse, b.groesse),
        deckkraft: m(a.deckkraft, b.deckkraft), farbe: [0, 1, 2].map(|c| m(a.farbe[c], b.farbe[c])) }
}

/// Deckkraft der eingebauten Arten (nur `blinken` hat eine): 1 -> 0.35 -> 1.
pub(super) fn deckkraft_bei(art: u8, p: f64) -> f64 {
    if art == 6 { 0.35 + 0.65 * 0.5 * (1.0 + (2.0 * std::f64::consts::PI * p).cos()) } else { 1.0 }
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

fn vorgabe_dauer(art: u8) -> i32 { match art { 2 => 400, 4 => 600, 5 => 1200, 6 => 1000, a if a >= EIGEN => 1000, _ => 900 } }

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

    /// Die Haut des Griffs eines Reglers fuer den jetzigen Zustand (wie
    /// `w_bild`).
    pub(super) fn w_griff(&self, w: &Widget, gedrueckt: bool) -> Option<String> {
        let s = w.stil_auf.as_ref()?;
        let schicht = |k: usize| s[k].griff.clone();
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
        self.uhr += self.takt_dt.clamp(0.0, 0.25);
        let uhr = self.uhr;
        // Bildhaeute: jeder Pfad, den eine Regel nennt, einmal laden.
        if self.stil.is_some() {
            let mut fehlt: Vec<String> = Vec::new();
            for win in &self.windows {
                for w in &win.widgets {
                    let Some(s) = w.stil_auf.as_ref() else { continue };
                    for schicht in s.iter() {
                        for p in [schicht.bild.as_ref(), schicht.griff.as_ref()].into_iter().flatten() {
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
        let mut laeuft = false;
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
                    // Was sichtbar laeuft, braucht das naechste Bild sofort.
                    if gezeigt[wi][i] && (au.gleit.is_some() || (!ruhig && (au.lauf.is_some() || au.stil_lauf.is_some()))) {
                        laeuft = true;
                    }
                }
            }
        }
        if laeuft { self.bild_in(0.0); }
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
    /// Deckkraft und Farbton der laufenden Animation (Ablauf mit
    /// `deckkraft`/`farbe`, oder `blinken`); None = voll und ohne Ton.
    pub(super) fn anim_deck(&self, wi: usize, idx: usize, w: &Widget) -> Option<(u8, i64)> {
        if self.ruhig { return None; }
        let au = w.aussehen.as_ref()?;
        let druck = self.press_origin == Some((wi, idx));
        let (a, t0) = au.lauf.or_else(|| au.stil_lauf.filter(|l| Some(l.0) == self.w_stil_anim(w, druck)))
            .filter(|&(a, t0)| !vorbei(a, t0, self.uhr))?;
        let p = ((self.uhr - t0) * 1000.0 / a.dauer as f64).fract();
        let (deck, farbe) = if a.art >= EIGEN {
            let ks = &self.stil.as_ref()?.ablaeufe.get((a.art - EIGEN) as usize)?.1;
            let k = ablauf_bei(ks, p);
            (k.deckkraft, k.farbe)
        } else { (deckkraft_bei(a.art, p), [255.0; 3]) };
        let a8 = (deck * 255.0).round().clamp(0.0, 255.0) as u8;
        let ton = ((farbe[0].round() as i64) << 16) | ((farbe[1].round() as i64) << 8) | (farbe[2].round() as i64);
        if a8 == 255 && ton == 0xFFFFFF { None } else { Some((a8, ton)) }
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
        if !self.ruhig { if let Some(r) = rest_bis_wechsel(&haut.dauern, self.uhr) { self.bild_in(r); } }
        if rand > 0 { g.nine_slice(tex, x, y, b, h, rand).is_ok() } else { g.draw_image_rect(tex, x, y, b, h); true }
    }

    // ================================================================ Befehle
    /// GUI_SET_SHAPE(knopf, form$): rechteck, pille, kreis, raute, bild --
    /// fuer Zeichnen und Treffer; "" = wieder die Stildatei.
    pub fn set_shape(&mut self, h: i64, form: &str) -> Result<(), String> {
        let f = super::gui_wort(form);
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

    pub(super) fn wcol(&self, w: &Widget, role: &str, theme_key: &str) -> i64 {
        // GUI_SET_COLOR, sonst die Stildatei (samt Zustaenden), sonst das Thema.
        if let Some(c) = w.ov.get(role) { return *c; }
        let basis = self.eigen(w, role).unwrap_or_else(|| self.th(theme_key));
        self.stil_misch(w, role, basis)
    }

    // Styling-aware Farb-/Text-Helfer (beruecksichtigen den enabled-Zustand).
    /// Die gedaempfte Farbe fuer Hinweise (Platzhalter, Zusatztext, leere
    /// Liste) -- aber so weit zur Textfarbe hin geschoben, dass sie auf
    /// DIESEM Grund mindestens 4,5:1 Kontrast hat (WCAG AA fuer Text). Das
    /// Grau des Themas passt zu seiner eigenen Flaeche; auf einer getoenten
    /// oder dunkleren war es kaum noch zu lesen.
    pub(super) fn leise(&self, bg: i64) -> i64 {
        let (m, t) = (self.th("muted_fg") & 0xFF_FFFF, self.th("text_fg") & 0xFF_FFFF);
        let bg = bg & 0xFF_FFFF;
        let mut a = 0.0;
        while a < 1.0 {
            let c = mischen(m, t, a);
            if kontrast(c, bg) >= 4.5 { return c; }
            a += 0.05;
        }
        t
    }

    pub(super) fn txt_col(&self, w: &Widget) -> i64 {
        if !w.enabled { self.stil_farbe(w, "fg", stil::AUS).unwrap_or_else(|| self.th("muted_fg")) }
        else { self.wcol(w, "fg", "text_fg") }
    }

    pub(super) fn acc_col(&self, w: &Widget) -> i64 {
        let a = self.wcol(w, "accent", "accent");
        if !w.enabled { shade(a, -70) } else { a }
    }

    /// Halbtransparente Selektions-Hintergrundfarbe (ARGB) hinter markiertem
    /// Text -- gemeinsam fuer TextInput/TextArea (frueher an beiden Stellen
    /// dupliziert).
    pub(super) fn selection_bg(&self, wdg: &Widget) -> i64 {
        let acc = self.wcol(wdg, "accent", "accent");
        ((0x66i64) << 24) | (acc & 0xFFFFFF)
    }

    /// Ob der Text-Caret in diesem Frame sichtbar ist (Blink-Takt) --
    /// gemeinsam fuer TextInput/TextArea.
    ///
    /// `caret_period` zaehlt in Sechzigstelsekunden (frueher Bilder bei 60
    /// Bildern je Sekunde); die Marke meldet, wann sie umschlaegt.
    pub(super) fn caret_blink_on(&self) -> bool {
        let p = self.m("caret_period").max(1) as f64 / 60.0;
        let phase = (self.takt / p + 1e-6).floor();
        self.bild_in((phase + 1.0) * p - self.takt);
        (phase as i64) % 2 == 0
    }

    /// Text mit per-Widget-Font/-Groesse (sonst unveraendert via g.text).
    /// Schriftgroesse dieses Widgets in Pixeln.
    pub(super) fn wsize(&self, g: &Graphics, w: &Widget) -> i32 {
        let n = self.w_groesse(w);
        self.sk(if n > 0 { n } else { g.text_height() })
    }

    /// Breite eines Textes so, wie `wtext` ihn zeichnen WIRD.
    ///
    /// Die Fallunterscheidung muss die von `wtext` genau spiegeln: ohne
    /// eigene Schrift und Groesse zeichnet es mit der aktiven Schrift, sonst
    /// mit dem Handle des Widgets (-1 = Standardschrift, NICHT die aktive).
    /// Weicht das Messen davon ab, sitzt zentrierter Text schief und der
    /// Beschnitt greift an der falschen Stelle.
    pub(super) fn wtext_width(&self, g: &Graphics, w: &Widget, s: &str) -> i32 {
        let st = self.w_stil(w);
        if st != 0 {
            let basis = font_wahl(w.font, g.active_font());
            return g.text_width_stil(s, self.wsize(g, w), basis, st);
        }
        if w.font == -1 && self.w_groesse(w) == 0 && self.scale == 1.0 {
            g.text_width(s)
        } else {
            g.text_width_in(s, self.wsize(g, w), self.wfont(g, w))
        }
    }

    /// Groesse und Font, mit denen `wtext_width` dieses Widget misst -- aber
    /// nur, wenn der Font fest ist (geladen, ohne Stil): dann laesst sich eine
    /// Zeile stueckweise vermessen (`Graphics::text_breite_fest`).
    pub(super) fn wtext_fest(&self, g: &Graphics, w: &Widget) -> Option<(i32, i64)> {
        if self.w_stil(w) != 0 { return None; }
        let (size, font) = if w.font == -1 && self.w_groesse(w) == 0 && self.scale == 1.0 {
            (g.text_height(), g.active_font())
        } else {
            (self.wsize(g, w), self.wfont(g, w))
        };
        if font < 0 { None } else { Some((size, font)) }
    }

    /// Schrift, mit der DIESES Widget zeichnet.
    ///
    /// Ohne eigene Schrift gilt die global gesetzte -- NICHT die eingebaute
    /// Standardschrift. Vorher wechselte `GUI_SET_FONT_SIZE` allein still
    /// die Schriftart auf die Pixelschrift, weil das Zeichnen dann mit dem
    /// Handle -1 lief. Wer eine GROESSE setzt, meint aber die Groesse, nicht
    /// eine andere Schrift.
    pub(super) fn wfont(&self, g: &Graphics, w: &Widget) -> i64 {
        // Mit Stil der geladene Schnitt -- so messen auch die Editier-Helfer
        // (Schreibmarke, Auswahl) mit der Schrift, die dasteht.
        g.schnitt_von(font_wahl(w.font, g.active_font()), self.w_stil(w))
    }

    /// Text der Fenster-Chrome: Titel, Menueleiste, Reiter, Popup-Eintrag,
    /// Hilfetext. Anders als `wtext` gehoert er zu keinem Widget, kann also
    /// keine eigene Schrift haben -- aber er muss mit dem Massstab wachsen.
    ///
    /// Messen und Zeichnen laufen bewusst durch DASSELBE Paar (`ctext` /
    /// `ctext_width`): waehlte das eine die aktive Groesse und das andere die
    /// skalierte, saessen Menuebreiten und Reiter-Kaesten neben ihrem Text.
    pub(super) fn ctext(&self, g: &mut Graphics, x: i32, y: i32, s: String, c: i64) {
        if self.scale == 1.0 { g.text(x, y, s, c); }
        else {
            let (sz, font) = (self.ctext_height(g), g.active_font());
            g.text_styled(x, y, s, c, font, sz);
        }
    }

    pub(super) fn ctext_width(&self, g: &Graphics, s: &str) -> i32 {
        if self.scale == 1.0 { g.text_width(s) }
        else { g.text_width_in(s, self.ctext_height(g), g.active_font()) }
    }

    pub(super) fn ctext_height(&self, g: &Graphics) -> i32 { self.sk(g.text_height()) }

    /// Groesse + Schrift eines Widgets als Buendel -- genau die, mit der
    /// `wtext` zeichnet. Die Editier-Helfer sind statisch und kommen an
    /// `self` nicht heran; ohne dieses Buendel muessten sie mit der aktiven
    /// Groesse messen, und Schreibmarke wie Auswahl saessen bei Massstab != 1
    /// neben dem Text.
    pub(super) fn mass(&self, g: &Graphics, w: &Widget) -> Mass {
        Mass { size: self.wsize(g, w), font: self.wfont(g, w) }
    }

    pub(super) fn wtext(&self, g: &mut Graphics, w: &Widget, x: i32, y: i32, s: String, c: i64) {
        let st = self.w_stil(w);
        if st != 0 {
            let (sz, basis) = (self.wsize(g, w), font_wahl(w.font, g.active_font()));
            g.text_styled_stil(x, y, s, c, basis, sz, st);
            return;
        }
        if w.font == -1 && self.w_groesse(w) == 0 && self.scale == 1.0 {
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
    pub(super) fn fbox(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, fill: i64, border: i64) {
        self.flaeche(g, x1, y1, x2, y2, fill, border, false);
    }

    /// Wie `fbox`, beruecksichtigt aber eine fuer diese Widget-Art
    /// hinterlegte Grafik.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn fbox_w(&self, g: &mut Graphics, kind: Kind, x1: i32, y1: i32, x2: i32, y2: i32,
              fill: i64, border: i64) {
        if self.skin(g, kind, x1, y1, x2, y2) { return; }
        self.flaeche(g, x1, y1, x2, y2, fill, border, false);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn fbox_tief_w(&self, g: &mut Graphics, kind: Kind, x1: i32, y1: i32, x2: i32, y2: i32,
                   fill: i64, border: i64) {
        if self.skin(g, kind, x1, y1, x2, y2) { return; }
        self.flaeche(g, x1, y1, x2, y2, fill, border, true);
    }

    /// Hinterlegte Grafik zeichnen, falls es eine gibt. Liefert TRUE, wenn
    /// sie die gezeichnete Flaeche ersetzt hat.
    pub(super) fn skin(&self, g: &mut Graphics, kind: Kind, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
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
    pub(super) fn fbox_tief(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, fill: i64, border: i64) {
        self.flaeche(g, x1, y1, x2, y2, fill, border, true);
    }

    /// Wie stark Verlauf und Glanz auf DIESER Hoehe noch sein sollen.
    ///
    /// `verlauf_hoehe` = 0 heisst: ueberall gleich stark (so war es immer).
    /// Sonst nimmt die Staerke ab, je hoeher die Flaeche ist -- ein Knopf
    /// woelbt sich weiter, eine grosse Liste wird matt. Ein Rest bleibt
    /// stehen, sonst saehe eine hohe Flaeche gar nicht mehr nach Material
    /// aus.
    pub(super) fn verlauf_anteil(&self, hoehe: i32) -> f64 {
        let vh = self.m("verlauf_hoehe");
        if vh <= 0 || hoehe <= vh { return 1.0; }
        (vh as f64 / hoehe as f64).max(0.2)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn flaeche(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32,
               fill: i64, border: i64, tief: bool) {
        let rad = self.rad_jetzt.get().unwrap_or_else(|| self.m("corner_radius"));
        let anteil = self.verlauf_anteil(y2 - y1);
        let grad = (self.m_form("gradient") as f64 * anteil).round() as i32;
        // Ein Farbverlauf aus der Stildatei ersetzt die Grundfarbe; Glanz,
        // Fase und Rand kommen wie sonst darueber.
        let bahn = self.bahn_jetzt.borrow().clone();
        if let Some((farben, winkel, phase, zyklisch)) = bahn {
            g.farbbahn(x1, y1, x2, y2, rad, &farben, winkel, phase, zyklisch);
            if grad > 0 && !tief { self.gloss(g, x1, y1, x2, y2, rad, anteil); }
        } else if grad > 0 {
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
                self.gloss(g, x1, y1, x2, y2, rad, anteil);
            }
        } else if rad > 0 {
            g.round_rect(x1, y1, x2, y2, rad, fill, true);
        } else {
            g.box_fill(x1, y1, x2, y2, fill);
        }
        // Drachenhauch-Nebel (Stildatei `nebel`): ueber der Fuellung, unter
        // Fase, Rand und allem, was das Widget darauf schreibt.
        if let Some((staerke, farbe)) = self.nebel_jetzt.get() {
            g.nebel(x1, y1, x2, y2, rad, farbe, staerke, self.nebel_uhr);
        }
        self.bevel(g, x1, y1, x2, y2, rad, tief, anteil);
        // Randbreite aus der Stildatei: 0 = kein Rand, mehr = Ringe nach innen.
        let breite = self.form_jetzt.get().rand.unwrap_or(1);
        for k in 0..breite.min((x2 - x1).min(y2 - y1) / 2) {
            if rad > 0 {
                g.round_rect(x1 + k, y1 + k, x2 - k, y2 - k, (rad - k).max(0), border, false);
            } else {
                g.rect(x1 + k, y1 + k, x2 - k, y2 - k, border);
            }
        }
    }

    /// Glanzkante: halbdurchsichtiges Weiss ueber der oberen Haelfte, nach
    /// unten ausblendend. Erst das laesst eine Flaeche gewoelbt wirken -- ein
    /// blosser Verlauf sieht weiterhin flach aus.
    pub(super) fn gloss(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, rad: i32,
             anteil: f64) {
        let s = (self.m_form("gloss") as f64 * anteil).round() as i32;
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
    pub(super) fn griff_farbe(&self, w: &Widget) -> i64 {
        mischen(self.wcol(w, "bg", "widget_bg"), self.th("text_fg"), 0.28)
    }

    /// Metallischer Rundgriff (Schieber-Knauf, Kippschalter-Knopf,
    /// Drehregler-Kappe). Gebaut aus konzentrischen Ringen mit wechselnder
    /// Helligkeit -- einen Verlauf ENTLANG eines Kreises kann `ring` nicht,
    /// gestaffelte Ringe sind die Naeherung, die ohne Textur auskommt.
    pub(super) fn knauf(&self, g: &mut Graphics, cx: i32, cy: i32, r: i32, grund: i64) {
        let r = r.max(2);
        if self.m_form("gradient") <= 0 {
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
    ///
    /// Auf einer HOHEN Flaeche (Metrik `verlauf_hoehe`) wird sie leiser, mit
    /// dem Quadrat des Anteils, den auch Verlauf und Glanz bekommen: eine
    /// helle Linie in voller Staerke ueber einer ganzen Liste ist die Kante
    /// eines Knopfs aus Windows 95 -- auf einem Knopf richtig, auf einer
    /// Leiste von 200 Punkten zu laut. Ohne die Metrik bleibt sie, wie sie war.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn bevel(&self, g: &mut Graphics, x1: i32, y1: i32, x2: i32, y2: i32, rad: i32, tief: bool,
             anteil: f64) {
        if self.m_form("bevel") <= 0 || x2 - x1 < 4 {
            return;
        }
        let ein = rad.max(1);
        // Deckkraft 0 hiesse in dhrt DECKEND -- leiser wird es bis 1.
        let leiser = |a: i64, rgb: i64| -> i64 {
            let a = ((a as f64) * anteil * anteil).round().max(1.0) as i64;
            (a << 24) | rgb
        };
        let (oben, unten) = if tief {
            // Versenkt: Schatten oben, Lichtkante unten.
            (leiser(0x60, 0x000000), leiser(0x38, 0xFFFFFF))
        } else {
            (leiser(0x50, 0xFFFFFF), leiser(0x40, 0x000000))
        };
        g.line(x1 + ein, y1 + 1, x2 - ein, y1 + 1, oben);
        g.line(x1 + ein, y2 - 1, x2 - ein, y2 - 1, unten);
    }

    /// Die Uebergaenge eines Bildes nachfuehren (Metrik `uebergang` in ms).
    ///
    /// Gerechnet wird mit `delta()` statt mit Bildern: bei 144 Hz liefe eine
    /// Bildzahl mehr als doppelt so schnell. Headless ist `delta` fest 1/60,
    /// ein Test sieht also bei jedem Lauf dieselbe Stufe.
    ///
    /// **Hinein geht es nur beim Ueberfahren weich.** Druecken und Fokus
    /// erscheinen SOFORT und blenden nur aus: wer klickt oder mit Tab
    /// weitergeht, will die Antwort in diesem Bild sehen, nicht in dreien.
    pub(super) fn uebergaenge(&mut self, g: &Graphics) {
        // Verliert eine Liste beim Umbenennen den Fokus (Tab, GUI_FOCUS),
        // gilt das wie ein Klick woanders: uebernehmen.
        let fokus_jetzt = self.focus_widget;
        let mut offen_edit = Vec::new();
        for (wi, win) in self.windows.iter().enumerate() {
            for (i, wdg) in win.widgets.iter().enumerate() {
                if Self::wird_umbenannt(wdg) && fokus_jetzt != Some((wi, i)) {
                    offen_edit.push((wi, i));
                }
            }
        }
        for (wi, i) in offen_edit { self.edit_ende_any(wi, i, true); }
        // Formatierte Textbereiche: ein Text, der auf einem Weg ohne Stile
        // kam (Eingabemethode, Bindung), bekommt sie hier nachgezogen.
        for win in self.windows.iter_mut() {
            for w in win.widgets.iter_mut() {
                if w.formatiert { stile_abgleichen(w); }
                if w.kind == Kind::TextArea { marken_abgleichen(w); }
            }
        }
        let dt = self.takt_dt;
        // Lichtstreifen der Fenster weiterschieben.
        for win in self.windows.iter_mut() { win.glanz.weiter(dt as f32); }
        if self.windows.iter().any(|w| w.alive && w.visible && w.glanz.t >= 0.0) { self.bild_in(0.0); }
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        let dauer = self.m("uebergang").max(0) as f32;
        let schritt = if dauer > 0.0 { (dt as f32 * 1000.0 / dauer).clamp(0.0, 1.0) } else { 1.0 };
        let (druck, fokus) = (self.press_origin, self.focus_widget);
        // Klappliste: geht nur AUF weich; zu ist sie sofort weg.
        if self.open_dropdown.is_none() || self.open_dropdown != self.dd_auf_von {
            self.dd_auf_t = 0.0;
            self.dd_auf_von = self.open_dropdown;
            // Frisch geoeffnet: die Markierung steht auf der Auswahl, und die
            // Auswahl steht in der MITTE -- man sieht ihre Nachbarn nach oben
            // und unten, statt sie am Rand zu suchen.
            if let Some((wi, i)) = self.open_dropdown {
                self.dd_mark = self.windows[wi].widgets[i].sel;
                let rows = self.dd_zeilen(wi, i);
                let n = self.windows[wi].widgets[i].items.len() as i32;
                self.dd_scroll = (self.dd_mark - rows / 2).clamp(0, (n - rows).max(0));
            }
        }
        if self.open_dropdown.is_some() { naeher(&mut self.dd_auf_t, 1.0, schritt); }
        // Welcher Eintrag der offenen Klappliste liegt unter der Maus?
        let dd_hover: Option<(usize, usize, i32)> = self.open_dropdown.map(|(wi, i)| {
            let (px, py, pw, ph) = self.dropdown_popup_rect(wi, i);
            let k = if mx >= px && mx < px + pw && my >= py && my < py + ph {
                self.dd_scroll + (my - py) / self.sk(DROPDOWN_ITEM_H)
            } else { -1 };
            (wi, i, k)
        });
        // Offene Menue-Popups (Wurzel und Untermenues) und ihr Eintrag unter
        // der Maus. Gerechnet vor der Schleife, die die Menues veraendert.
        let offen: Vec<(usize, usize, i32)> = self.popup_chain(g).into_iter()
            .map(|(wi, mi, px, py)| (wi, mi, self.popup_item_at(g, wi, mi, px, py, mx, my).map(|k| k as i32).unwrap_or(-1)))
            .collect();
        for (wi, win) in self.windows.iter_mut().enumerate() {
            for (mi, m) in win.menus.iter_mut().enumerate() {
                match offen.iter().find(|&&(w, i, _)| w == wi && i == mi) {
                    Some(&(_, _, k)) => { naeher(&mut m.auf_t, 1.0, schritt); m.b.schritt(k, schritt); }
                    None => { m.auf_t = 0.0; m.b = Blende::default(); }
                }
            }
        }
        for (wi, win) in self.windows.iter_mut().enumerate() {
            for (i, wdg) in win.widgets.iter_mut().enumerate() {
                // Eine eigene Dauer aus der Stildatei (`uebergang`).
                let schritt = match Self::w_uebergang(wdg) {
                    Some(d) if d > 0 => (dt as f32 * 1000.0 / d as f32).clamp(0.0, 1.0),
                    Some(_) => 1.0,
                    None => schritt,
                };
                // Der Stand VOR diesem Schritt: damit beginnen die Rollen, wenn
                // sie neu angelegt werden -- mit dem Stand danach waere eine
                // Rolle mit eigener, langer Dauer im ersten Bild schon am Ziel.
                let vorher = (wdg.ueber_t, wdg.fokus_t, wdg.druck_t);
                naeher(&mut wdg.ueber_t, if wdg.hovered && wdg.enabled { 1.0 } else { 0.0 }, schritt);
                if druck == Some((wi, i)) { wdg.druck_t = 1.0; } else { naeher(&mut wdg.druck_t, 0.0, schritt); }
                if fokus == Some((wi, i)) { wdg.fokus_t = 1.0; } else { naeher(&mut wdg.fokus_t, 0.0, schritt); }
                // Eigene Dauern je Farbrolle: dieselben drei Ziele, jede Rolle
                // mit ihrer Dauer (ohne eigene die des Widgets).
                match Self::w_uebergang_je(wdg) {
                    Some(je) => {
                        let (u, f, d) = vorher;
                        let ziel_u = if wdg.hovered && wdg.enabled { 1.0 } else { 0.0 };
                        let rt = wdg.rollen_t.get_or_insert_with(|| Box::new([[u, f, d]; 4]));
                        for k in 0..4 {
                            let s = match je[k] {
                                Some(ms) if ms > 0 => (dt as f32 * 1000.0 / ms as f32).clamp(0.0, 1.0),
                                Some(_) => 1.0,
                                None => schritt,
                            };
                            naeher(&mut rt[k][0], ziel_u, s);
                            if fokus == Some((wi, i)) { rt[k][1] = 1.0; } else { naeher(&mut rt[k][1], 0.0, s); }
                            if druck == Some((wi, i)) { rt[k][2] = 1.0; } else { naeher(&mut rt[k][2], 0.0, s); }
                        }
                    }
                    None => wdg.rollen_t = None,
                }
                // Kippschalter: `value` gleitet dem Zustand nach, gezeichnet
                // ueber `weich()`, damit der Knopf anfaehrt und abbremst.
                if wdg.kind == Kind::Toggle {
                    let ziel = if wdg.checked { 1.0 } else { 0.0 };
                    let mut v = wdg.value as f32;
                    naeher(&mut v, ziel, schritt);
                    wdg.value = v as f64;
                }
                // Die Zeile unter der Maus, je Art aus ihrer eigenen Quelle.
                let zeile = match wdg.kind {
                    Kind::ListBox => wdg.zeile_jetzt,
                    Kind::Table => wdg.tbl.as_ref().filter(|t| !t.zellmodus).map(|t| t.hover_row).unwrap_or(-1),
                    Kind::Tree => wdg.tree.as_ref().map(|t| t.hover).unwrap_or(-1),
                    Kind::Dropdown => match dd_hover { Some((w, j, k)) if w == wi && j == i => k, _ => -1 },
                    _ => -1,
                };
                let zeile = if wdg.enabled { zeile } else { -1 };
                wdg.zeile_b.schritt(zeile, schritt);
                // Weiches Rollen: dem Ziel nachziehen, schnell los und
                // weich aus. Hat jemand anderes gerollt, gibt das Rad nach.
                if let Some(z) = wdg.rad_ziel {
                    // Der Rollstand liegt je Art woanders: Liste in `value`,
                    // Tabelle in `scroll_y` (ganze Punkte -- der Rest wird
                    // in `rad_letzt` mitgefuehrt, sonst bliebe sie am Ende
                    // einen Punkt vor dem Ziel stehen).
                    let ist = match wdg.tbl.as_ref() { Some(t) => t.scroll_y as f64, None => Self::roll_ist(wdg) };
                    if (ist - wdg.rad_letzt.round()).abs() > 0.5 && (ist - wdg.rad_letzt).abs() > 0.5 {
                        wdg.rad_ziel = None;
                    } else {
                        let k = (schritt as f64 * 3.0).min(1.0);
                        let mut neu = wdg.rad_letzt + (z - wdg.rad_letzt) * k;
                        if (z - neu).abs() < 0.5 { neu = z; wdg.rad_ziel = None; }
                        wdg.rad_letzt = neu;
                        match wdg.tbl.as_mut() { Some(t) => t.scroll_y = neu.round() as i32, None => Self::roll_setze(wdg, neu) }
                    }
                }
                if let Some(t) = wdg.tree.as_mut() {
                    let n = t.nodes.len();
                    t.auf_t.truncate(n);
                    for k in 0..n {
                        let ziel = if t.nodes[k].expanded && t.nodes[k].has_children { 1.0 } else { 0.0 };
                        if k >= t.auf_t.len() { t.auf_t.push(ziel); } else { naeher(&mut t.auf_t[k], ziel, schritt); }
                    }
                }
                if let Some(l) = wdg.leiste.as_mut() {
                    l.hover_t.resize(l.eintraege.len(), 0.0);
                    for (k, t) in l.hover_t.iter_mut().enumerate() {
                        let ziel = if l.hover == k as i32 && !l.eintraege[k].aus { 1.0 } else { 0.0 };
                        naeher(t, ziel, schritt);
                    }
                }
            }
        }
    }

    // --- Theme / Metriken ---
    pub fn theme_accent(&mut self, c: i64) { self.theme.insert("accent".into(), c); }

    pub fn theme_set(&mut self, key: String, c: i64) -> Result<(), String> {
        if !self.theme.contains_key(&key) { return Err(unbekannt("GUI_THEME_SET", &key, self.theme.keys())); }
        self.theme.insert(key, c); Ok(())
    }

    pub fn theme_get(&self, key: &str) -> Result<i64, String> {
        self.theme.get(key).copied().ok_or_else(|| unbekannt("GUI_THEME_GET", key, self.theme.keys()))
    }

    pub fn metric_set(&mut self, key: String, v: i32) -> Result<(), String> {
        if !self.metrics.contains_key(&key) { return Err(unbekannt("GUI_METRIC_SET", &key, self.metrics.keys())); }
        if v < 0 { return Err("GUI_METRIC_SET: Wert muss >= 0 sein".into()); }
        self.metrics.insert(key, v); Ok(())
    }

    pub fn metric_get(&self, key: &str) -> Result<i32, String> {
        self.metrics.get(key).copied().ok_or_else(|| unbekannt("GUI_METRIC_GET", key, self.metrics.keys()))
    }

    pub fn theme_preset(&mut self, name: &str) -> Result<(), String> {
        let mut key = name.to_lowercase();
        // "system": hell oder dunkel wie das Betriebssystem, mit dessen
        // Akzentfarbe (systemaussehen.rs); unbekannt = dunkel.
        let system = key == "system";
        let aussehen = if system { Some(crate::systemaussehen::jetzt()) } else { None };
        if let Some(a) = aussehen {
            key = if a.dunkel == Some(false) { "modern_light".into() } else { "modern_dark".into() };
        }
        let ergebnis = self.theme_preset_name(&key, name);
        if let Some(farbe) = aussehen.and_then(|a| a.akzent) { self.theme.insert("accent".into(), farbe); }
        ergebnis
    }

    fn theme_preset_name(&mut self, key: &str, name: &str) -> Result<(), String> {
        let key = key.to_string();
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
    pub(super) fn modern(&self) -> bool { self.m("corner_radius") > 0 }
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
        assert!(ablauf_lesen("50%: schwanz 1", 3, |_| 3).unwrap_err().contains("x, y, drehen, groesse"));
        assert!(ablauf_lesen("50%: farbe 1", 3, |_| 3).unwrap_err().contains("farbe #RGB oder #RRGGBB"));
        let k = ablauf_bei(&ablauf_lesen("100%: deckkraft 0, farbe #0000FF", 1, |_| 1).unwrap(), 0.5);
        assert_eq!((k.deckkraft, k.farbe), (0.5, [127.5, 127.5, 255.0]));
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
