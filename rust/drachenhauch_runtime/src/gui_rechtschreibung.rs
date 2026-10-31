//! Rechtschreibung in Textfeld und Textbereich (GUI_SPELL, Paket 3 aus
//! docs/entwurf-gui-wie-gtk.md).
//!
//! Das Feld haelt nur das Handle eines Woerterbuchs (rechtschreibung.rs);
//! geprueft wird eine kurze Pause nach der letzten Aenderung, nicht je
//! Anschlag -- sonst stuende unter jedem halb getippten Wort eine Welle, und
//! ein langer Text kostete bei jedem Zeichen eine ganze Pruefung. Bis dahin
//! wandern die alten Wellen mit dem Text (gemeinsamer Anfang/Ende), die im
//! geaenderten Stueck fallen weg.
//!
//! Ein Rechtsklick auf ein falsches Wort oeffnet ein kleines Menue mit
//! Vorschlaegen und "Ignorieren" -- es geht dem Kontextmenue des Programms
//! vor, weil dort genau das Wort gemeint ist. Umschalt+F10 oeffnet es an der
//! Schreibmarke.
use super::*;
use crate::rechtschreibung as rs;

/// Pause nach der letzten Aenderung, bis geprueft wird (Sekunden).
const RS_PAUSE: f64 = 0.3;
/// Hoechstens so viele Vorschlaege im Menue.
const RS_VORSCHLAEGE: usize = 6;
/// Farbe der Wellen (Alpha 0 = deckend).
const RS_FARBE: i64 = 0xD8_3A3A;
const KEY_F10: i64 = 1073741891;

pub(crate) struct RsZusatz {
    pub h: i64,
    /// Der Text, zu dem `fehler` gehoert.
    stand: String,
    /// (erstes Zeichen, Zeichenzahl) je falschem Wort.
    pub fehler: Vec<(usize, usize)>,
    /// Sekunden bis zur naechsten Pruefung, < 0 = keine faellig.
    faellig: f64,
}

/// Ein Eintrag im Menue: Text und Art (0 Vorschlag, 1 Ignorieren, 2 nur
/// Hinweis).
pub(crate) struct RsMenue {
    wi: usize,
    i: usize,
    start: usize,
    laenge: usize,
    wort: String,
    eintraege: Vec<(String, u8)>,
    x: i32,
    y: i32,
    mark: i32,
}

/// Fehler zum alten Text auf den neuen umrechnen: was vor dem geaenderten
/// Stueck liegt, bleibt, was dahinter liegt, rueckt mit, was es beruehrt,
/// faellt weg.
fn verschieben(alt: &str, neu: &str, fehler: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let a: Vec<char> = alt.chars().collect();
    let n: Vec<char> = neu.chars().collect();
    let mut p = 0;
    while p < a.len() && p < n.len() && a[p] == n[p] { p += 1; }
    let mut s = 0;
    while s < a.len() - p && s < n.len() - p && a[a.len() - 1 - s] == n[n.len() - 1 - s] { s += 1; }
    let alt_ende = a.len() - s;
    // Grenzt die Aenderung direkt an ein Wort, zaehlt sie nur dann zu ihm,
    // wenn dort jetzt ein Wortzeichen steht -- ein Leerzeichen dahinter
    // laesst das Wort, wie es war.
    let wz = |c: Option<&char>| c.map(|c| c.is_alphabetic() || matches!(c, '\'' | '\u{2019}')).unwrap_or(false);
    fehler.iter().filter_map(|&(st, l)| {
        let neu_st = (st + n.len()).saturating_sub(a.len());
        if st + l < p || (st + l == p && !wz(n.get(p))) { Some((st, l)) }
        else if st > alt_ende || (st == alt_ende && (neu_st == 0 || !wz(n.get(neu_st - 1)))) { Some((neu_st, l)) }
        else { None }
    }).collect()
}

impl Gui {
    /// GUI_SPELL(wdg, woerterbuch): prueft das Feld mit diesem Woerterbuch,
    /// -1 schaltet ab.
    pub fn spell(&mut self, h: i64, buch: i64) -> Result<(), String> {
        let f = "GUI_SPELL";
        let k = self.wdg(h, f)?.kind;
        if !matches!(k, Kind::TextInput | Kind::TextArea) {
            return Err(format!("{}: nur fuer Textfeld und Textbereich, nicht fuer {}", f, k.as_str()));
        }
        if buch < 0 { self.wdg_mut(h, f)?.rs = None; return Ok(()); }
        rs::sprache(buch).map_err(|e| e.replace("SPELL_LANGUAGE$", f))?;
        let w = self.wdg_mut(h, f)?;
        w.rs = Some(Box::new(RsZusatz { h: buch, stand: w.text.clone(), fehler: Vec::new(), faellig: 0.0 }));
        Ok(())
    }

    /// GUI_SPELL_COUNT(wdg): wie viele Woerter gerade als falsch markiert sind.
    pub fn spell_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.wdg(h, "GUI_SPELL_COUNT")?.rs.as_ref().map(|z| z.fehler.len() as i64).unwrap_or(0))
    }

    /// Je Bild am Ende von GUI_UPDATE: Wellen nachziehen, nach der Pause
    /// pruefen.
    pub(super) fn rs_pass(&mut self, dt: f64) {
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                let w = &mut self.windows[wi].widgets[i];
                if !w.alive { continue; }
                let passwort = w.passwort;
                let Some(z) = w.rs.as_mut() else { continue };
                if passwort { z.fehler.clear(); continue; }
                if w.text != z.stand {
                    z.fehler = verschieben(&z.stand, &w.text, &z.fehler);
                    z.stand = w.text.clone();
                    z.faellig = RS_PAUSE;
                    continue;
                }
                if z.faellig < 0.0 { continue; }
                z.faellig -= dt;
                if z.faellig > 0.0 { continue; }
                z.faellig = -1.0;
                match rs::fehler(z.h, &w.text) {
                    Ok(f) => z.fehler = f,
                    // Das Woerterbuch ist freigegeben -- das Feld prueft nicht mehr.
                    Err(_) => w.rs = None,
                }
            }
        }
        if let Some(m) = self.rs_menue.as_ref() {
            let weg = self.windows.get(m.wi).and_then(|w| w.widgets.get(m.i))
                .map(|w| !w.alive || w.rs.is_none() || !w.text.chars().skip(m.start).take(m.laenge).eq(m.wort.chars()))
                .unwrap_or(true);
            if weg { self.rs_menue = None; }
        }
    }

    /// Zeichen unter der Maus in einem pruefenden Feld: (Fenster, Widget, Stelle).
    fn rs_stelle_bei(&self, g: &Graphics, mx: i32, my: i32) -> Option<(usize, usize, i32)> {
        let wi = self.topmost_at(mx, my)?;
        if self.windows[wi].entwurf || self.modal.is_some_and(|m| m != wi) { return None; }
        for i in (0..self.windows[wi].widgets.len()).rev() {
            let w = &self.windows[wi].widgets[i];
            if !w.alive || !self.widget_shown(wi, w) || !Self::in_rect(mx, my, self.abs_rect(wi, w))
                || !self.im_panel_sichtbar(wi, w, mx, my) { continue; }
            if w.rs.is_none() || !w.enabled || w.passwort { return None; }
            return match w.kind {
                Kind::TextArea => self.ta_index_at(g, wi, i, mx, my).map(|k| (wi, i, k)),
                Kind::TextInput => {
                    let ms = self.mass(g, w);
                    let (ax, _, _, _) = self.abs_rect(wi, w);
                    let chars: Vec<char> = w.text.chars().collect();
                    let k = Self::caret_index_at(g, &chars, mx - (ax + 5 + self.ti_rand(w).0) + w.scroll, ms);
                    Some((wi, i, k))
                }
                _ => None,
            };
        }
        None
    }

    /// Das Menue fuer das falsche Wort an Stelle `k` oeffnen. false = dort
    /// steht keins.
    fn rs_oeffnen(&mut self, wi: usize, i: usize, k: i32, x: i32, y: i32) -> bool {
        let w = &self.windows[wi].widgets[i];
        let Some(z) = w.rs.as_ref() else { return false };
        let k = k.max(0) as usize;
        let Some(&(start, laenge)) = z.fehler.iter().find(|&&(s, l)| k >= s && k <= s + l) else { return false };
        let wort: String = w.text.chars().skip(start).take(laenge).collect();
        let mut eintraege: Vec<(String, u8)> = rs::vorschlaege(z.h, &wort, RS_VORSCHLAEGE).unwrap_or_default()
            .into_iter().map(|v| (v, 0)).collect();
        if eintraege.is_empty() { eintraege.push(("(keine Vorschläge)".into(), 2)); }
        eintraege.push((format!("„{}“ ignorieren", wort), 1));
        let mark = eintraege.iter().position(|e| e.1 != 2).unwrap_or(0) as i32;
        self.rs_menue = Some(RsMenue { wi, i, start, laenge, wort, eintraege, x, y, mark });
        self.focus_window = Some(wi);
        self.focus_widget = Some((wi, i));
        true
    }

    /// Lage des Menues: (x, y, Breite, Hoehe, Zeilenhoehe). Am Bildschirmrand
    /// weicht es aus wie ein Kontextmenue.
    fn rs_rect(&self, g: &Graphics) -> Option<(i32, i32, i32, i32, i32)> {
        let m = self.rs_menue.as_ref()?;
        let ih = self.sk(24);
        let pad = self.sk(10);
        let b = m.eintraege.iter().map(|e| self.ctext_width(g, &e.0)).max().unwrap_or(0) + 2 * pad + 2;
        let h = m.eintraege.len() as i32 * ih + 2 + self.sk(5);
        let mut x = m.x;
        let mut y = m.y;
        if self.schirm_b > 0 && x + b > self.schirm_b { x = (m.x - b).max(0); }
        if self.schirm_h > 0 && y + h > self.schirm_h { y = (m.y - h).max(0); }
        Some((x, y, b, h, ih))
    }

    /// Eintrag `k` an der Stelle y (der Trenner vor "Ignorieren" ist kein Eintrag).
    fn rs_zeile_y(&self, y: i32, ih: i32, k: usize, n: usize) -> i32 {
        y + 1 + k as i32 * ih + if k + 1 == n { self.sk(5) } else { 0 }
    }

    /// Ein neuer Druck: ins Menue (waehlt) oder daneben (schliesst); ein
    /// Rechtsklick auf ein falsches Wort oeffnet es. true = verbraucht.
    pub(super) fn rs_druck(&mut self, g: &Graphics, mx: i32, my: i32, links: bool, rechts: bool) -> bool {
        if !links && !rechts { return false; }
        if let Some((x, y, b, h, ih)) = self.rs_rect(g) {
            if Self::in_rect(mx, my, (x, y, b, h)) {
                if links {
                    let n = self.rs_menue.as_ref().unwrap().eintraege.len();
                    if let Some(k) = (0..n).find(|&k| { let zy = self.rs_zeile_y(y, ih, k, n); my >= zy && my < zy + ih }) {
                        self.rs_waehlen(g, k);
                    }
                }
                return true;
            }
            self.rs_menue = None;
            if links { return true; }
        }
        if rechts {
            if let Some((wi, i, k)) = self.rs_stelle_bei(g, mx, my) {
                if self.rs_oeffnen(wi, i, k, mx, my) {
                    let w = &mut self.windows[wi].widgets[i];
                    w.caret = k; w.sel_anchor = k;
                    w.marken_zusatz.clear();
                    return true;
                }
            }
        }
        false
    }

    /// Tasten, solange das Menue offen ist (Pfeile, Enter, ESC) -- sie
    /// gehoeren dann ihm. Ohne Menue: Umschalt+F10 in einem pruefenden Feld
    /// oeffnet es an der Schreibmarke. true = die Tasten sind verbraucht.
    pub(super) fn rs_tasten(&mut self, g: &Graphics) -> bool {
        if self.rs_menue.is_none() {
            let Some((wi, i)) = self.focus_widget else { return false };
            if !g.key_shift() || !g.key_pressed(KEY_F10) { return false; }
            let Some(w) = self.windows.get(wi).and_then(|w| w.widgets.get(i)) else { return false };
            if w.rs.is_none() { return false; }
            let (ax, ay, _, fh) = self.abs_rect(wi, w);
            let k = w.caret;
            return self.rs_oeffnen(wi, i, k, ax + self.sk(8), ay + fh.min(self.sk(28)));
        }
        let m = self.rs_menue.as_mut().unwrap();
        let n = m.eintraege.len() as i32;
        let waehlbar = |m: &RsMenue, k: i32| m.eintraege.get(k as usize).map(|e| e.1 != 2).unwrap_or(false);
        if g.key_pressed(KEY_DOWN) {
            let mut k = m.mark;
            for _ in 0..n { k = (k + 1).rem_euclid(n); if waehlbar(m, k) { break; } }
            m.mark = k;
        }
        if g.key_pressed(KEY_UP) {
            let mut k = m.mark;
            for _ in 0..n { k = (k - 1).rem_euclid(n); if waehlbar(m, k) { break; } }
            m.mark = k;
        }
        // Die Tasten gehoeren dem Menue: der Aufrufer sperrt damit auch Enter
        // und ESC fuer den Standard- und den Abbrechen-Knopf.
        if g.key_pressed(KEY_ENTER) {
            let k = m.mark as usize;
            self.rs_waehlen(g, k);
        } else if g.key_pressed(KEY_ESC) {
            self.rs_menue = None;
        }
        true
    }

    /// Eintrag `k` ausfuehren: Vorschlag einsetzen oder das Wort ignorieren.
    fn rs_waehlen(&mut self, g: &Graphics, k: usize) {
        let Some(m) = self.rs_menue.take() else { return };
        let Some((text, art)) = m.eintraege.get(k) else { return };
        match art {
            0 => self.rs_ersetzen(m.wi, m.i, m.start, m.laenge, text, g.get_time()),
            1 => {
                let h = self.windows[m.wi].widgets[m.i].rs.as_ref().map(|z| z.h).unwrap_or(-1);
                if rs::ignorieren(h, &m.wort).is_ok() {
                    // Jedes Feld mit demselben Woerterbuch sieht das gleich.
                    for win in self.windows.iter_mut() {
                        for w in win.widgets.iter_mut() {
                            if let Some(z) = w.rs.as_mut().filter(|z| z.h == h) { z.faellig = 0.0; }
                        }
                    }
                }
            }
            _ => self.rs_menue = Some(m),
        }
    }

    /// Ein Wort ersetzen -- ein eigener Schritt fuer Strg+Z, mit on_change.
    fn rs_ersetzen(&mut self, wi: usize, i: usize, start: usize, laenge: usize, neu: &str, jetzt: f64) {
        let w = &mut self.windows[wi].widgets[i];
        if w.nur_lesen { return; }
        let before = w.text.clone();
        let mut chars: Vec<char> = before.chars().collect();
        if start + laenge > chars.len() { return; }
        let ein: Vec<char> = neu.chars().collect();
        if w.maxlaenge > 0 && (chars.len() - laenge + ein.len()) as i32 > w.maxlaenge { return; }
        let caret0 = w.caret;
        chars.splice(start..start + laenge, ein.iter().copied());
        w.undo_zeit = -10.0;
        Self::undo_merken(w, &before, caret0, jetzt);
        w.text = chars.iter().collect();
        w.caret = (start + ein.len()) as i32;
        w.sel_anchor = w.caret;
        if w.kind == Kind::TextArea {
            stile_abgleichen(w);
            let nachher = w.text.clone();
            Self::falten_nachziehen(w, &before, &nachher, caret0);
            w.spans.clear();
            w.wellen.clear();
        }
        if let Some(f) = w.on_change.clone() { self.pending.push(f); }
    }

    /// Die Wellen eines pruefenden Felds, im Format von `wellen`.
    pub(super) fn rs_wellen(wdg: &Widget) -> Vec<(u32, u32, i64)> {
        wdg.rs.as_ref().map(|z| z.fehler.iter().map(|&(s, l)| (s as u32, l as u32, RS_FARBE)).collect()).unwrap_or_default()
    }

    /// Wellen im Textfeld (aus `zeichne_textfeld`, im Clip des Felds).
    pub(super) fn rs_textfeld_wellen(&self, g: &mut Graphics, wdg: &Widget, chars: &[char], tx: i32, yb: i32) {
        let Some(z) = wdg.rs.as_ref() else { return };
        if wdg.passwort || !wdg.vorschau.is_empty() { return; }
        let zacke = self.sk(2).max(2);
        for &(s, l) in &z.fehler {
            if s + l > chars.len() { continue; }
            let x0 = tx + self.wtext_width(g, wdg, &chars[..s].iter().collect::<String>());
            let x1 = tx + self.wtext_width(g, wdg, &chars[..s + l].iter().collect::<String>());
            zickzack(g, x0, x1, yb, zacke, RS_FARBE);
        }
    }

    /// Das Menue (aus `draw_top`).
    pub(super) fn rs_zeichnen(&self, g: &mut Graphics) {
        let Some((x, y, b, h, ih)) = self.rs_rect(g) else { return };
        let m = self.rs_menue.as_ref().unwrap();
        let bg = self.th("widget_bg");
        let fg = self.th("text_fg");
        let acc = self.th("accent");
        let border = self.th("widget_border");
        g.round_rect(x + 1, y + 3, x + b, y + h + 2, 3, (0x40i64) << 24, true);
        g.box_fill(x, y, x + b - 1, y + h - 1, bg);
        g.rect(x, y, x + b - 1, y + h - 1, border);
        let th = self.ctext_height(g);
        let pad = self.sk(10);
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        let n = m.eintraege.len();
        for (k, (text, art)) in m.eintraege.iter().enumerate() {
            let zy = self.rs_zeile_y(y, ih, k, n);
            if k + 1 == n {
                let ly = zy - self.sk(3);
                g.line(x + pad / 2, ly, x + b - pad / 2, ly, border);
            }
            let ueber = *art != 2 && mx >= x && mx < x + b && my >= zy && my < zy + ih;
            let farbe = if *art == 2 { self.leise(bg) }
                else if k as i32 == m.mark { g.box_fill(x + 1, zy, x + b - 2, zy + ih - 1, shade(acc, -110)); lesbar_auf(shade(acc, -110)) }
                else { if ueber { g.box_fill(x + 1, zy, x + b - 2, zy + ih - 1, shade(bg, 22)); } fg };
            self.ctext(g, x + pad, zy + (ih - th) / 2, text.clone(), farbe);
        }
    }
}

/// Eine Wellenlinie von x0 bis x1 knapp unter der Grundlinie yb.
pub(super) fn zickzack(g: &mut Graphics, x0: i32, x1: i32, yb: i32, zacke: i32, c: i64) {
    let mut x = x0;
    let mut oben = true;
    while x < x1 {
        let nx = (x + zacke).min(x1);
        let (y0, y1) = if oben { (yb, yb + zacke) } else { (yb + zacke, yb) };
        g.line(x, y0, nx, y1, c);
        x = nx;
        oben = !oben;
    }
}

#[cfg(test)]
mod tests {
    use super::verschieben;

    #[test]
    fn wellen_wandern_mit_dem_text() {
        // "Ein Hasu im Gartn": Fehler Hasu (4,4), Gartn (12,5).
        let f = vec![(4, 4), (12, 5)];
        // Vorn eingefuegt: beide ruecken.
        assert_eq!(verschieben("Ein Hasu im Gartn", "Da Ein Hasu im Gartn", &f), vec![(7, 4), (15, 5)]);
        // Im ersten Wort getippt: es faellt weg, das zweite rueckt.
        assert_eq!(verschieben("Ein Hasu im Gartn", "Ein Haus im Gartn", &f), vec![(12, 5)]);
        // Hinten angehaengt: das letzte Wort waechst, also faellt es weg.
        assert_eq!(verschieben("Ein Hasu im Gartn", "Ein Hasu im Gartne", &f), vec![(4, 4)]);
        // Ein Leerzeichen dahinter laesst es stehen.
        assert_eq!(verschieben("Ein Hasu im Gartn", "Ein Hasu im Gartn ", &f), vec![(4, 4), (12, 5)]);
    }
}
