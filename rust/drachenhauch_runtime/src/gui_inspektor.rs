//! Inspektor und ruhige Bewegung (docs/module-gui.md, Abschnitt 9).
//!
//! Teil von `gui.rs`.
//! - **Inspektor** wie die Entwicklerwerkzeuge eines Browsers: an
//!   (`GUI_INSPECTOR(TRUE)`, oder Strg+Umschalt+I, wenn `DHRT_INSPEKTOR=1`
//!   gesetzt ist) umrandet er das Widget unter dem Zeiger; ein Klick waehlt
//!   es -- und erreicht das Widget NICHT -- und eine Tafel zeigt Art, Name,
//!   Klassen, Lage, Zustand, die passenden Regeln der Stildatei mit Zeile
//!   und was je Schicht gilt. Die Tafel liest denselben Text wie
//!   `GUI_INSPECT$` (`inspect`), Treffer ueber `hit_test`.
//! - **Bewegung**: `GUI_SET_MOTION("system"|"an"|"aus")`. Unter "system"
//!   (Vorgabe) folgt die gui der Einstellung "Animationen reduzieren" des
//!   Betriebssystems: dann stehen Animationen still und GIF-Haeute zeigen
//!   ihr erstes Bild. Farbuebergaenge bleiben -- sie bewegen nichts.

use super::*;

/// Farben des Inspektors -- fest, unabhaengig vom Thema des Programms.
const I_UEBER: i64 = 0x00B4FF;
const I_WAHL: i64 = 0xFF8C00;
const I_GRUND: i64 = (0xF0i64 << 24) | 0x1B1F27;
const I_TEXT: i64 = 0xE8ECF2;
const I_LEISE: i64 = 0x8C96A5;

impl Gui {
    // ================================================================ Bewegung
    /// Spielt die gui gerade Bewegung ab? (GUI_MOTION)
    pub fn bewegung_offen(&self) -> bool { !self.ruhig }

    /// In GUI_UPDATE: entscheiden, ob Bewegung laeuft. Das System wird nur
    /// gefragt, wenn sich ueberhaupt etwas bewegen koennte (unter macOS und
    /// Linux startet die Frage ein Programm).
    pub(super) fn bewegung_pass(&mut self) {
        self.ruhig = match self.bewegung {
            1 => false,
            2 => true,
            _ => {
                let braucht = self.haeute.values().any(|h| h.as_ref().map(|h| h.bilder.len() > 1).unwrap_or(false))
                    || self.windows.iter().any(|w| w.widgets.iter().any(|x| x.aussehen.is_some()));
                if braucht { crate::systemaussehen::weniger_bewegung() } else { self.ruhig }
            }
        };
    }

    /// GUI_SET_MOTION(modus$): "system" (Vorgabe), "an", "aus".
    pub fn set_motion(&mut self, modus: &str) -> Result<(), String> {
        self.bewegung = match modus.trim().to_lowercase().as_str() {
            "system" | "" => 0,
            "an" | "on" | "voll" => 1,
            "aus" | "off" | "ruhig" => 2,
            _ => return Err(format!("GUI_SET_MOTION: '{}' -- erwartet \"system\", \"an\" oder \"aus\"", modus)),
        };
        self.ruhig = match self.bewegung { 1 => false, 2 => true, _ => crate::systemaussehen::weniger_bewegung() };
        Ok(())
    }

    // ================================================================ Inspektor
    /// GUI_INSPECTOR(an): den Inspektor zeigen oder schliessen.
    pub fn inspector(&mut self, an: bool) {
        self.insp_an = an;
        if !an { self.insp_wahl = -1; }
    }
    pub fn inspector_on(&self) -> bool { self.insp_an }
    /// GUI_INSPECTOR_SELECTED(): das gewaehlte Widget, -1 = keins.
    pub fn inspector_selected(&self) -> i64 {
        if self.insp_wahl < 0 { return -1; }
        match self.wdg(self.insp_wahl, "") { Ok(w) if w.alive => self.insp_wahl, _ => -1 }
    }

    /// Strg+Umschalt+I (mit DHRT_INSPEKTOR=1) schaltet um, ESC schliesst --
    /// beide gelten als verbraucht.
    pub(super) fn inspektor_tasten(&mut self, g: &Graphics) {
        if self.insp_taste && g.key_ctrl() && g.key_shift() && g.key_pressed(73) {
            self.inspector(!self.insp_an);
            self.kuerzel_gefeuert = true;
        } else if self.insp_an && g.key_pressed(27) {   // 27 = ESC
            self.inspector(false);
            self.kuerzel_gefeuert = true;
        }
    }

    /// Ein Druck bei offenem Inspektor: waehlt das Widget darunter (auf der
    /// Tafel: nichts). TRUE = verbraucht, kein Widget sieht ihn.
    pub(super) fn inspektor_druck(&mut self, g: &Graphics, mx: i32, my: i32) -> bool {
        if !self.insp_an { return false; }
        let f = self.flaeche;
        if self.insp_flaeche() == f && Self::in_rect(mx, my, self.insp_tafel(g).0) { return true; }
        self.insp_wahl = self.hit_test_auf(Some(f), mx, my);
        true
    }

    /// Auf welcher Flaeche steht die Tafel? Wo das gewaehlte Widget lebt,
    /// sonst im Hauptfenster.
    fn insp_flaeche(&self) -> u32 {
        let wahl = self.inspector_selected();
        if wahl < 0 { return 0; }
        self.popup_flaeche(Self::dec_widget(wahl).0)
    }
    /// Der Bereich, in dem die Tafel steht: der Schirm, in einem echten
    /// Fenster dessen Rechteck.
    fn insp_bereich(&self, g: &Graphics, f: u32) -> (i32, i32, i32, i32) {
        if f != 0 {
            if let Some(win) = self.windows.iter().find(|w| w.alive && w.nativ == f) { return (win.x, win.y, win.w, win.h); }
        }
        (0, 0, g.screen_width() as i32, g.screen_height() as i32)
    }

    /// GUI_INSPECT$(widget): was der Inspektor ueber ein Widget weiss.
    pub fn inspect(&self, h: i64) -> Result<String, String> {
        let f = "GUI_INSPECT$";
        let w = self.wdg(h, f)?;
        let (wi, i) = Self::dec_widget(h);
        let mut z: Vec<String> = Vec::new();
        let mut kopf = w.kind.as_str().to_string();
        if !w.name.is_empty() { kopf += &format!(" #{}", w.name); }
        let mut kl = w.klassen.clone();
        if w.kind == Kind::Button {
            if let Some(a) = KNOPF_ARTEN.get(w.variante as usize) { kl.push(a.to_string()); }
        }
        for k in &kl { kopf += &format!(" .{}", k); }
        z.push(kopf);
        let win = &self.windows[wi];
        let fenster = if win.name.is_empty() { format!("\"{}\"", win.title) } else { format!("#{}", win.name) };
        // Die Lage, wie das Programm sie setzt (GUI_SET_BOUNDS, logisch).
        z.push(format!("fenster {}  lage {} {} {} {}", fenster, self.unsk(w.x), self.unsk(w.y), self.unsk(w.w), self.unsk(w.h)));
        let mut zust: Vec<&str> = Vec::new();
        if !self.widget_shown(wi, w) { zust.push("verborgen"); }
        if !w.enabled { zust.push("disabled"); }
        if w.hovered { zust.push("hover"); }
        if self.focus_widget == Some((wi, i)) { zust.push("focus"); }
        if self.press_origin == Some((wi, i)) { zust.push("active"); }
        if w.checked { zust.push("checked"); }
        z.push(format!("zustand {}", if zust.is_empty() { "normal".to_string() } else { zust.join(" ") }));
        if w.kind == Kind::Button {
            let form = aussehen::FORMEN.get(self.w_form_art(w) as usize).copied().filter(|f| !f.is_empty()).unwrap_or("rechteck");
            let anim = w.aussehen.as_ref().and_then(|a| a.lauf.or(a.stil_lauf)).map(|(a, _)| aussehen::anim_text(a));
            z.push(format!("form {}{}{}", form,
                anim.map(|a| format!("  animation {}", a)).unwrap_or_default(),
                if self.ruhig { "  (bewegung aus)" } else { "" }));
        }
        match self.stil.as_ref() {
            None => z.push("stildatei: keine".into()),
            Some(datei) => {
                let kette = if datei.mit_vorfahren { Self::ketten(win).into_iter().nth(i).unwrap_or_default() } else { Vec::new() };
                let regeln = datei.passend_kurz(w.kind.as_str(), &kl, &w.name, w.checked, &kette, &self.stil_lage(wi));
                z.push(format!("stildatei {}{}", self.stil_quelle,
                    if regeln.is_empty() { " -- keine Regel passt".to_string() } else { String::new() }));
                for (zeile, sel, _) in regeln { z.push(format!("  zeile {}: {}", zeile, sel)); }
                if let Some(auf) = w.stil_auf.as_ref() {
                    for (k, s) in auf.iter().enumerate() {
                        let t = s.beschreiben();
                        if !t.is_empty() { z.push(format!("  {}: {}", stil::SCHICHT_NAMEN[k], t)); }
                    }
                }
                if !self.stil_fehler.is_empty() { z.push(format!("fehler beim neu laden: {}", self.stil_fehler)); }
            }
        }
        // Barrierefreiheit (gui_pruefen.rs): was an diesem Widget auffaellt.
        for b in self.a11y_befunde(wi, i, &self.a11y_ziele(wi)) { z.push(format!("barrierefreiheit: {}", b)); }
        Ok(z.join("\n"))
    }

    /// Lage der Tafel und ihre Zeilen -- dieselbe Quelle fuer Zeichnen und
    /// Druck. Ohne Wahl ein kurzer Hinweis.
    fn insp_tafel(&self, g: &Graphics) -> ((i32, i32, i32, i32), Vec<String>) {
        let wahl = self.inspector_selected();
        let mut zeilen = vec!["Inspektor -- Klick waehlt, ESC schliesst".to_string()];
        match self.inspect(wahl) {
            Ok(t) if wahl >= 0 => zeilen.extend(t.lines().map(|s| s.to_string())),
            _ => zeilen.push("(kein Widget gewaehlt)".into()),
        }
        let pad = self.sk(8);
        let lh = self.ctext_height(g) + self.sk(3);
        let (bx, by, sb, _) = self.insp_bereich(g, self.insp_flaeche());
        // Hoechstens die halbe Breite; laengere Zeilen brechen an ", " und
        // doppelten Leerzeichen um, der Rest eingerueckt.
        let platz = (sb / 2).max(self.sk(240)) - pad * 2;
        let zeilen: Vec<String> = zeilen.into_iter().flat_map(|z| self.insp_umbrechen(g, &z, platz)).collect();
        let breite = zeilen.iter().map(|z| self.ctext_width(g, z)).max().unwrap_or(0).min(platz) + pad * 2;
        let hoehe = zeilen.len() as i32 * lh + pad * 2;
        // Rechts oben -- steht das gewaehlte Widget rechts, dann links.
        let rechts = if wahl >= 0 {
            let (wi, _) = Self::dec_widget(wahl);
            let (x, _, b, _) = self.abs_rect(wi, &self.windows[wi].widgets[Self::dec_widget(wahl).1]);
            x + b / 2 < bx + sb / 2
        } else { true };
        let m = self.sk(8);
        let x = if rechts { bx + sb - breite - m } else { bx + m };
        ((x, by + m, breite, hoehe), zeilen)
    }

    /// Eine Zeile der Tafel auf `platz` Pixel umbrechen.
    fn insp_umbrechen(&self, g: &Graphics, z: &str, platz: i32) -> Vec<String> {
        if self.ctext_width(g, z) <= platz { return vec![z.to_string()]; }
        let einzug: String = z.chars().take_while(|c| *c == ' ').collect();
        let fort = format!("{}    ", einzug);
        // Stuecke samt Trenner, damit nichts verloren geht.
        let mut stuecke: Vec<String> = Vec::new();
        let mut rest = z;
        while let Some(p) = [rest.find(", "), rest.find("  ")].into_iter().flatten().filter(|&p| p > 0).min() {
            stuecke.push(rest[..p + 2].to_string());
            rest = &rest[p + 2..];
        }
        stuecke.push(rest.to_string());
        let mut out: Vec<String> = Vec::new();
        let mut zeile = String::new();
        for st in stuecke {
            let probe = format!("{}{}", zeile, st);
            if !zeile.trim().is_empty() && self.ctext_width(g, probe.trim_end()) > platz {
                out.push(zeile.trim_end().to_string());
                zeile = format!("{}{}", fort, st.trim_start());
            } else {
                zeile = probe;
            }
        }
        if !zeile.trim().is_empty() { out.push(zeile.trim_end().to_string()); }
        out
    }

    /// Zeichnen, ganz oben auf der Flaeche `f` (0 = Hauptfenster): Rahmen um
    /// das Widget unter dem Zeiger und um das gewaehlte, wenn sie hier leben;
    /// die Tafel dort, wo das gewaehlte lebt.
    pub(super) fn inspektor_zeichnen(&self, g: &mut Graphics, f: u32) {
        if !self.insp_an { return; }
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        let ((tx, ty, tb, th), zeilen) = self.insp_tafel(g);
        let tafel_hier = self.insp_flaeche() == f;
        // Unter dem Zeiger: duenner Rahmen und ein Schild mit Art und Groesse.
        let unter = if tafel_hier && Self::in_rect(mx, my, (tx, ty, tb, th)) { -1 } else { self.hit_test_auf(Some(f), mx, my) };
        if unter >= 0 && unter != self.insp_wahl {
            let (wi, i) = Self::dec_widget(unter);
            let w = &self.windows[wi].widgets[i];
            let (x, y, b, h) = self.abs_rect(wi, w);
            g.rect(x - 1, y - 1, x + b, y + h, I_UEBER);
            let mut schild = w.kind.as_str().to_string();
            if !w.name.is_empty() { schild += &format!(" #{}", w.name); }
            schild += &format!("  {}x{}", self.unsk(b), self.unsk(h));
            let sw = self.ctext_width(g, &schild) + self.sk(8);
            let sh = self.ctext_height(g) + self.sk(4);
            let sy = if y - sh - 2 >= 0 { y - sh - 2 } else { y + h + 2 };
            g.box_fill(x, sy, x + sw, sy + sh, I_UEBER);
            self.ctext(g, x + self.sk(4), sy + self.sk(2), schild, 0x0A0D12);
        }
        let wahl = self.inspector_selected();
        if wahl >= 0 && tafel_hier {
            let (wi, i) = Self::dec_widget(wahl);
            let (x, y, b, h) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
            g.rect(x - 1, y - 1, x + b, y + h, I_WAHL);
            g.rect(x - 2, y - 2, x + b + 1, y + h + 1, I_WAHL);
        }
        if !tafel_hier { return; }
        // Die Tafel.
        let rad = self.sk(6);
        g.round_rect(tx + 2, ty + 3, tx + tb + 1, ty + th + 2, rad, 0x50i64 << 24, true);
        g.round_rect(tx, ty, tx + tb - 1, ty + th - 1, rad, I_GRUND, true);
        g.round_rect(tx, ty, tx + tb - 1, ty + th - 1, rad, I_WAHL, false);
        let pad = self.sk(8);
        let lh = self.ctext_height(g) + self.sk(3);
        g.push_clip(tx, ty, tb, th);
        for (k, z) in zeilen.into_iter().enumerate() {
            let farbe = if k == 0 { I_WAHL } else if z.starts_with("  ") { I_TEXT } else { I_LEISE };
            let farbe = if k == 1 { I_TEXT } else { farbe };
            self.ctext(g, tx + pad, ty + pad + k as i32 * lh, z, farbe);
        }
        g.pop_clip();
    }
}
