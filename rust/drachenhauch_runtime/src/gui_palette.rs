//! Befehlspalette (docs/module-gui.md, Abschnitt 11).
//!
//! Teil von `gui.rs`. Strg+Umschalt+P oeffnet ueber dem Fenster mit dem
//! Fokus eine Suche ueber alle Eintraege seiner Menueleiste (samt
//! Untermenues) und seiner Werkzeugleisten -- jede Anwendung hat sie, ohne
//! eine Zeile dafuer. Tippen filtert, Hoch/Runter waehlt, Enter oder ein
//! Klick loest aus (`fire_menu_item` bzw. `tb_klick`, derselbe Weg wie
//! Maus und Kuerzel), ESC schliesst.
//!
//! Ein Menuekuerzel geht vor: hat ein Menue des Programms selbst
//! Strg+Umschalt+P (die IDE), feuert dessen Eintrag, und die Palette bleibt
//! zu (`kuerzel_gefeuert`). Solange sie offen ist, hat kein Widget den
//! Fokus -- sonst landete jede Taste zusaetzlich im Textfeld darunter.

use super::*;

/// Wohin ein Eintrag fuehrt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum PalZiel { Menue(usize, usize, usize), Leiste(usize, usize, i32) }

#[derive(Clone, Debug)]
pub(super) struct PalEintrag { pub text: String, pub kuerzel: String, pub ziel: PalZiel }

#[derive(Clone, Debug)]
pub(super) struct Palette {
    pub wi: usize,
    pub suche: String,
    pub wahl: usize,
    pub oben: usize,
    pub alle: Vec<PalEintrag>,
    /// Indizes in `alle`, die zur Suche passen, beste zuerst.
    pub treffer: Vec<usize>,
    /// Der Fokus vor dem Oeffnen -- kommt beim Schliessen zurueck.
    pub fokus_vorher: Option<(usize, usize)>,
}

const PAL_ZEILEN: usize = 10;

/// Wie gut passt `suche` auf `text` (beides klein)? None = gar nicht.
/// Kleiner ist besser: ein zusammenhaengender Treffer schlaegt verstreute
/// Buchstaben, ein Wortanfang einen Treffer mitten im Wort.
pub(super) fn pal_wertung(text: &str, suche: &str) -> Option<i64> {
    if suche.is_empty() { return Some(0); }
    if let Some(p) = text.find(suche) {
        let wortanfang = p == 0 || !text[..p].chars().last().map(|c| c.is_alphanumeric()).unwrap_or(false);
        return Some(if wortanfang { 0 } else { 100 } + p as i64);
    }
    // Buchstaben der Reihe nach, mit Abstand bestraft.
    let mut rest = text.char_indices();
    let mut letzte: Option<usize> = None;
    let mut luecken = 0i64;
    for c in suche.chars() {
        let (p, _) = rest.find(|(_, t)| *t == c)?;
        if let Some(l) = letzte { luecken += (p - l - 1) as i64; }
        letzte = Some(p);
    }
    Some(1000 + luecken)
}

impl Gui {
    /// Die Eintraege eines Fensters: Menueleiste (rekursiv, "Datei > Neu")
    /// und Knoepfe der Werkzeugleisten, die kein Menue schon nennt.
    fn pal_sammeln(&self, wi: usize) -> Vec<PalEintrag> {
        let win = &self.windows[wi];
        let mut out: Vec<PalEintrag> = Vec::new();
        fn sauber(s: &str) -> String { s.replace('&', "").trim().trim_end_matches("...").trim_end_matches('\u{2026}').trim().to_string() }
        fn menue(win: &Window, wi: usize, mi: usize, pfad: &str, tiefe: usize, out: &mut Vec<PalEintrag>) {
            if tiefe > 8 { return; }
            for (ii, it) in win.menus[mi].items.iter().enumerate() {
                if it.separator || !it.enabled { continue; }
                let name = format!("{}{}", pfad, sauber(&it.label));
                if it.sub >= 0 && (it.sub as usize) < win.menus.len() {
                    menue(win, wi, it.sub as usize, &format!("{} \u{203A} ", name), tiefe + 1, out);
                } else {
                    out.push(PalEintrag { text: name, kuerzel: it.kuerzel.clone(), ziel: PalZiel::Menue(wi, mi, ii) });
                }
            }
        }
        for (mi, m) in win.menus.iter().enumerate() {
            if !m.in_bar { continue; }
            menue(win, wi, mi, &format!("{} \u{203A} ", sauber(&m.label)), 0, &mut out);
        }
        let schon: Vec<String> = out.iter().map(|e| e.text.rsplit(" \u{203A} ").next().unwrap_or("").to_lowercase()).collect();
        for (i, w) in win.widgets.iter().enumerate() {
            if !w.alive || w.kind != Kind::Toolbar || !self.widget_shown(wi, w) || !w.enabled { continue; }
            let Some(l) = w.leiste.as_ref() else { continue };
            for (k, e) in l.eintraege.iter().enumerate() {
                if e.art != 0 || e.aus { continue; }
                let name = if !e.text.is_empty() { sauber(&e.text) } else if !e.tip.is_empty() { sauber(&e.tip) } else { e.symbol.clone() };
                if name.is_empty() || schon.contains(&name.to_lowercase()) { continue; }
                out.push(PalEintrag { text: name, kuerzel: String::new(), ziel: PalZiel::Leiste(wi, i, k as i32) });
            }
        }
        out
    }

    /// Das Fenster, fuer das die Palette gilt: der Dialog, wenn einer modal
    /// ist, sonst das mit dem Fokus, sonst das oberste.
    fn pal_fenster(&self) -> Option<usize> {
        let ok = |wi: usize| self.windows.get(wi).map(|w| w.alive && w.visible && !w.entwurf).unwrap_or(false);
        if let Some(m) = self.modal { return if ok(m) { Some(m) } else { None }; }
        self.focus_window.filter(|&w| ok(w)).or_else(|| self.z_order.iter().rev().copied().find(|&w| ok(w)))
    }

    fn pal_filtern(&mut self) {
        let Some(p) = self.palette.as_mut() else { return };
        let suche = p.suche.to_lowercase();
        let mut t: Vec<(i64, usize)> = p.alle.iter().enumerate()
            .filter_map(|(k, e)| pal_wertung(&e.text.to_lowercase(), &suche).map(|w| (w, k))).collect();
        t.sort();
        p.treffer = t.into_iter().map(|(_, k)| k).collect();
        p.wahl = 0;
        p.oben = 0;
    }

    /// GUI_PALETTE(an): oeffnen (fuer das Fenster mit dem Fokus) oder schliessen.
    pub fn palette_oeffnen(&mut self, an: bool) {
        if !an {
            if let Some(p) = self.palette.take() { self.focus_widget = p.fokus_vorher; }
            return;
        }
        if self.palette.is_some() { return; }
        let Some(wi) = self.pal_fenster() else { return };
        let alle = self.pal_sammeln(wi);
        self.palette = Some(Palette { wi, suche: String::new(), wahl: 0, oben: 0, alle, treffer: Vec::new(), fokus_vorher: self.focus_widget });
        self.focus_widget = None;
        self.open_menu = None; self.context_open = None; self.sub_chain.clear();
        self.pal_filtern();
    }
    pub fn palette_offen(&self) -> bool { self.palette.is_some() }
    /// GUI_PALETTE_KEY(an): Strg+Umschalt+P an oder aus (Vorgabe an).
    pub fn palette_taste(&mut self, an: bool) { self.pal_taste = an; }
    /// GUI_PALETTE_FILTER(text$): die Suche setzen.
    pub fn palette_filter(&mut self, text: &str) -> Result<(), String> {
        let p = self.palette.as_mut().ok_or("GUI_PALETTE_FILTER: die Palette ist nicht offen -- erst GUI_PALETTE(TRUE)")?;
        p.suche = text.to_string();
        self.pal_filtern();
        Ok(())
    }
    /// GUI_PALETTE_ITEMS$(): die passenden Eintraege, je Zeile der Text und,
    /// wenn es eins gibt, TAB und Kuerzel; die gewaehlte beginnt mit `>`.
    pub fn palette_items(&self) -> String {
        let Some(p) = self.palette.as_ref() else { return String::new() };
        p.treffer.iter().enumerate().map(|(n, &k)| {
            let e = &p.alle[k];
            let k = if e.kuerzel.is_empty() { String::new() } else { format!("\t{}", e.kuerzel) };
            format!("{}{}{}", if n == p.wahl { ">" } else { "" }, e.text, k)
        }).collect::<Vec<_>>().join("\n")
    }

    /// Den gewaehlten Eintrag ausloesen und schliessen.
    fn pal_ausloesen(&mut self, n: usize) {
        let ziel = self.palette.as_ref().and_then(|p| p.treffer.get(n).map(|&k| p.alle[k].ziel));
        self.palette_oeffnen(false);
        match ziel {
            Some(PalZiel::Menue(wi, mi, ii)) => {
                let gueltig = self.windows.get(wi).and_then(|w| w.menus.get(mi)).and_then(|m| m.items.get(ii)).map(|it| it.enabled).unwrap_or(false);
                if gueltig { self.fire_menu_item(wi, mi, ii); }
            }
            Some(PalZiel::Leiste(wi, i, k)) => {
                if self.windows.get(wi).and_then(|w| w.widgets.get(i)).is_some() { self.tb_klick(wi, i, k); }
            }
            None => {}
        }
    }

    /// Tasten: nach den Menuekuerzeln, vor den Widgets.
    pub(super) fn palette_pass(&mut self, g: &mut Graphics) {
        if self.palette.is_none() {
            if self.pal_taste && !self.kuerzel_gefeuert && g.key_ctrl() && g.key_shift() && !g.key_alt() && g.key_pressed(80) {
                self.palette_oeffnen(true);
                if self.palette.is_some() {
                    let _ = g.pop_text_input();
                    self.kuerzel_gefeuert = true;
                }
            }
            return;
        }
        // Ist ihr Fenster weg (geschlossen, versteckt), geht sie mit.
        let wi = self.palette.as_ref().map(|p| p.wi).unwrap_or(0);
        if !self.windows.get(wi).map(|w| w.alive && w.visible).unwrap_or(false) { self.palette_oeffnen(false); return; }
        // Offen: die Tasten gehoeren ihr (den Fokus hat `palette_oeffnen`
        // schon genommen).
        self.kuerzel_gefeuert = true;
        if g.key_pressed(KEY_ESC) { self.palette_oeffnen(false); return; }
        if g.key_pressed(KEY_ENTER) {
            let n = self.palette.as_ref().map(|p| p.wahl).unwrap_or(0);
            self.pal_ausloesen(n);
            return;
        }
        let getippt: String = g.pop_text_input().chars().filter(|c| !c.is_control()).collect();
        let zurueck = g.key_repeat(KEY_BACKSPACE);
        let p = self.palette.as_mut().unwrap();
        let mut neu_filtern = false;
        if !getippt.is_empty() { p.suche.push_str(&getippt); neu_filtern = true; }
        if zurueck && p.suche.pop().is_some() { neu_filtern = true; }
        if neu_filtern { self.pal_filtern(); return; }
        let n = p.treffer.len();
        if n == 0 { return; }
        if g.key_repeat(KEY_DOWN) { p.wahl = (p.wahl + 1) % n; }
        if g.key_repeat(KEY_UP) { p.wahl = (p.wahl + n - 1) % n; }
        if g.key_repeat(KEY_PAGEDOWN) { p.wahl = (p.wahl + PAL_ZEILEN).min(n - 1); }
        if g.key_repeat(KEY_PAGEUP) { p.wahl = p.wahl.saturating_sub(PAL_ZEILEN); }
        if p.wahl < p.oben { p.oben = p.wahl; }
        if p.wahl >= p.oben + PAL_ZEILEN { p.oben = p.wahl + 1 - PAL_ZEILEN; }
    }

    /// Lage: (Kasten, Suchzeile-Hoehe, Zeilenhoehe) -- dieselbe Quelle fuer
    /// Zeichnen und Druck.
    fn pal_lage(&self, g: &Graphics) -> Option<((i32, i32, i32, i32), i32, i32)> {
        let p = self.palette.as_ref()?;
        let sb = g.screen_width() as i32;
        let sh = g.screen_height() as i32;
        let b = self.sk(560).min(sb - self.sk(24)).max(self.sk(200));
        let zh = self.ctext_height(g) + self.sk(10);
        let kopf = zh + self.sk(12);
        let zeilen = p.treffer.len().min(PAL_ZEILEN).max(1) as i32;
        let h = kopf + zeilen * zh + self.sk(8);
        let x = (sb - b) / 2;
        let y = (sh / 8).min(self.sk(80)).max(self.sk(8));
        Some(((x, y, b, h), kopf, zh))
    }

    /// Ein Druck bei offener Palette: auf einem Eintrag loest er aus,
    /// daneben schliesst er sie. Immer verbraucht.
    pub(super) fn palette_druck(&mut self, g: &Graphics, mx: i32, my: i32) -> bool {
        let Some(((x, y, b, h), kopf, zh)) = self.pal_lage(g) else { return false };
        if !Self::in_rect(mx, my, (x, y, b, h)) { self.palette_oeffnen(false); return true; }
        if my >= y + kopf {
            let zeile = ((my - y - kopf) / zh.max(1)) as usize;
            let oben = self.palette.as_ref().map(|p| p.oben).unwrap_or(0);
            let n = self.palette.as_ref().map(|p| p.treffer.len()).unwrap_or(0);
            if zeile < PAL_ZEILEN && oben + zeile < n { self.pal_ausloesen(oben + zeile); }
        }
        true
    }

    pub(super) fn palette_zeichnen(&self, g: &mut Graphics) {
        let Some(((x, y, b, h), kopf, zh)) = self.pal_lage(g) else { return };
        let p = self.palette.as_ref().unwrap();
        let rad = self.m("corner_radius").min(8).max(2);
        let grund = self.th("widget_bg");
        let rand = self.th("win_border");
        let text = self.th("text_fg");
        let leise = self.th("muted_fg");
        let akzent = self.th("accent");
        g.round_rect(x + 3, y + 5, x + b + 2, y + h + 4, rad, 0x60i64 << 24, true);
        g.round_rect(x, y, x + b - 1, y + h - 1, rad, grund, true);
        g.round_rect(x, y, x + b - 1, y + h - 1, rad, rand, false);
        // Suchzeile mit Lupe, Text und Schreibmarke.
        let pad = self.sk(10);
        let th = self.ctext_height(g);
        let sy = y + (kopf - th) / 2;
        let s = self.sk(14);
        sinnbild(g, "suchen", x + pad, y + (kopf - s) / 2, s, leise, false);
        let tx = x + pad + s + self.sk(8);
        if p.suche.is_empty() {
            self.ctext(g, tx, sy, "Befehl suchen ...".to_string(), leise);
            g.box_fill(tx, sy, tx + 1, sy + th, akzent);
        } else {
            let tw = self.ctext_width(g, &p.suche);
            self.ctext(g, tx, sy, p.suche.clone(), text);
            g.box_fill(tx + tw + 1, sy, tx + tw + 2, sy + th, akzent);
        }
        g.line(x + 1, y + kopf - 1, x + b - 2, y + kopf - 1, rand);
        if p.treffer.is_empty() {
            self.ctext(g, x + pad, y + kopf + (zh - th) / 2, "Kein passender Befehl".to_string(), leise);
            return;
        }
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        for (z, &k) in p.treffer.iter().enumerate().skip(p.oben).take(PAL_ZEILEN) {
            let ry = y + kopf + (z - p.oben) as i32 * zh;
            let e = &p.alle[k];
            let gewaehlt = z == p.wahl;
            let ueber = Self::in_rect(mx, my, (x, ry, b, zh));
            if gewaehlt || ueber {
                let c = if gewaehlt { mischen(grund, akzent, 0.28) } else { mischen(grund, akzent, 0.12) };
                g.box_fill(x + 3, ry, x + b - 4, ry + zh - 1, c);
            }
            let ky = ry + (zh - th) / 2;
            let kw = if e.kuerzel.is_empty() { 0 } else { self.ctext_width(g, &e.kuerzel) + self.sk(12) };
            let platz = (b - pad * 2 - kw).max(self.sk(40));
            let mut t = e.text.clone();
            while t.chars().count() > 3 && self.ctext_width(g, &t) > platz { t.pop(); t = format!("{}\u{2026}", t.trim_end_matches('\u{2026}')); }
            self.ctext(g, x + pad, ky, t, text);
            if !e.kuerzel.is_empty() {
                let kx = x + b - pad - self.ctext_width(g, &e.kuerzel);
                self.ctext(g, kx, ky, e.kuerzel.clone(), leise);
            }
        }
        if p.treffer.len() > PAL_ZEILEN {
            let gesamt = p.treffer.len() as f64;
            let hb = (PAL_ZEILEN as i32 * zh) as f64;
            let dy = (p.oben as f64 / gesamt * hb) as i32;
            let dh = ((PAL_ZEILEN as f64 / gesamt * hb) as i32).max(self.sk(12));
            g.box_fill(x + b - self.sk(5), y + kopf + dy, x + b - self.sk(3), y + kopf + dy + dh, leise);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wertung_bevorzugt_wortanfang_und_zusammenhang() {
        let a = pal_wertung("datei \u{203a} speichern", "spei").unwrap();
        let b = pal_wertung("datei \u{203a} alles speichern", "spei").unwrap();
        let c = pal_wertung("datei \u{203a} umspeichern", "spei").unwrap();
        // "dsp": d aus "datei", s und p aus "speichern" -- nur verstreut.
        let d = pal_wertung("datei \u{203a} speichern", "dsp").unwrap();
        assert!(a < c && b < c, "Wortanfang vor Wortmitte: {} {} {}", a, b, c);
        assert!(c < d, "zusammenhaengend vor verstreut");
        assert_eq!(pal_wertung("bearbeiten \u{203a} kopieren", "xyz"), None);
        assert_eq!(pal_wertung("egal", ""), Some(0));
    }
}
