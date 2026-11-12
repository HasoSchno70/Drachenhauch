//! Formular-Widgets: Radio, Dropdown, Fortschritt, Statusleiste, Pfadleiste, Listen samt Ausbau, Knopfarten.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Formular-Widgets (Phase 3): Radio, Dropdown, ProgressBar ---
    pub fn radio(&mut self, win: i64, group: String, label: String, x: i32, y: i32) -> Result<i64, String> {
        let cs = self.m_roh("check_size");
        let mut wd = Self::blank(Kind::Radio, x, y, cs, cs);
        wd.text = label; wd.group = group;
        wd.auto_w = true;                        // wie beim Kaestchen
        self.add_widget(win, "GUI_RADIO", wd)
    }
    pub fn progress(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Progress, x, y, w, h);
        wd.min = 0.0; wd.max = 1.0; wd.value = 0.0;
        self.add_widget(win, "GUI_PROGRESS", wd)
    }
    pub fn dropdown(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, items: Vec<String>) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Dropdown, x, y, w, h);
        wd.sel = if items.is_empty() { -1 } else { 0 };
        wd.items = items;
        self.add_widget(win, "GUI_DROPDOWN", wd)
    }
    /// Alle Radios derselben Gruppe (Fenster `wi`) ausser `keep` deselektieren.
    pub(super) fn select_radio(&mut self, wi: usize, keep: usize) {
        let group = self.windows[wi].widgets[keep].group.clone();
        for (j, w) in self.windows[wi].widgets.iter_mut().enumerate() {
            if w.kind == Kind::Radio && w.group == group && w.alive {
                w.checked = j == keep;
            }
        }
    }
    /// Index (0-basiert in Erstellungsreihenfolge) des gewaehlten Radios der
    /// Gruppe von `h`, oder -1. `h` darf ein beliebiges Radio der Gruppe sein.
    pub fn radio_selected(&self, h: i64) -> Result<i64, String> {
        let w = self.wdg(h, "GUI_RADIO_SELECTED")?;
        if w.kind != Kind::Radio { return Err("GUI_RADIO_SELECTED: Widget ist kein radio".into()); }
        let (wi, _) = Self::dec_widget(h);
        let group = &w.group;
        let mut idx = 0i64;
        for wd in &self.windows[wi].widgets {
            if wd.kind == Kind::Radio && &wd.group == group && wd.alive {
                if wd.checked { return Ok(idx); }
                idx += 1;
            }
        }
        Ok(-1)
    }
    // ListBox: scrollbare, immer sichtbare Auswahlliste (teilt items/sel mit
    // Dropdown; `value` haelt den vertikalen Scroll-Offset in Pixeln).
    pub fn listbox(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, items: Vec<String>) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::ListBox, x, y, w, h);
        wd.sel = -1; wd.items = items;
        self.add_widget(win, "GUI_LISTBOX", wd)
    }
    pub fn image(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, tex: i64) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Image, x, y, w, h);
        wd.sel = tex as i32;   // Textur-Handle (LOADIMAGE) im sel-Feld
        self.add_widget(win, "GUI_IMAGE", wd)
    }
    pub fn set_image(&mut self, h: i64, tex: i64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_IMAGE")?;
        if !matches!(w.kind, Kind::Image | Kind::Card) { return Err("GUI_SET_IMAGE: Widget ist kein image und keine Kachel".into()); }
        w.sel = tex as i32; Ok(())
    }
    /// Kachel (GUI_CARD): Bild oben, Titel, Beschreibung darunter -- und das
    /// GANZE ist der Knopf. Vorher baute man das aus Bild, Knopf und
    /// Beschriftung zusammen, und nur der schmale Knopf nahm den Klick an.
    /// Bild-Handle im sel-Feld wie beim Bild, Titel im text, die
    /// Beschreibung im placeholder-Feld (das eine Kachel sonst nicht braucht
    /// und das die .dhform schon schreibt).
    #[allow(clippy::too_many_arguments)]
    pub fn card(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, tex: i64,
                titel: String, text: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Card, x, y, w, h);
        wd.sel = tex as i32; wd.text = titel; wd.placeholder = text;
        wd.bildmodus = 2;   // fuellen: ein Bildschirmfoto soll die Flaeche decken
        self.add_widget(win, "GUI_CARD", wd)
    }
    pub fn card_set_text(&mut self, h: i64, text: String) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_CARD_SET_TEXT")?;
        if w.kind != Kind::Card { return Err("GUI_CARD_SET_TEXT: Widget ist keine Kachel (GUI_CARD)".into()); }
        w.placeholder = text; Ok(())
    }
    pub fn card_text(&self, h: i64) -> Result<String, String> {
        let w = self.wdg(h, "GUI_CARD_TEXT$")?;
        if w.kind != Kind::Card { return Err("GUI_CARD_TEXT$: Widget ist keine Kachel (GUI_CARD)".into()); }
        Ok(w.placeholder.clone())
    }
    /// Button mit Icon (Textur-Handle aus LOADIMAGE/GENTEX im sel-Feld). Mit Text
    /// = Icon links + Text rechts; ohne Text = flacher, mittiger Icon-Button (fuer
    /// Toolbars). Ansonsten ein ganz normaler Button (GUI_CLICKED/ON_CLICK).
    pub fn icon_button(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32,
                       tex: i64, text: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Button, x, y, w, h);
        wd.sel = tex as i32; wd.text = text;
        self.add_widget(win, "GUI_ICON_BUTTON", wd)
    }
    /// Icon eines Buttons setzen/ersetzen (-1 entfernt es).
    pub fn set_icon(&mut self, h: i64, tex: i64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_ICON")?;
        if w.kind != Kind::Button { return Err("GUI_SET_ICON: Widget ist kein button".into()); }
        w.sel = tex as i32; w.sinnbild.clear(); Ok(())
    }
    /// Eingebautes Sinnbild auf einen Knopf -- `""` nimmt es wieder weg.
    pub fn set_icon_symbol(&mut self, h: i64, name: &str) -> Result<(), String> {
        let n = if name.is_empty() { String::new() } else { sinnbild_pruefen(name, "GUI_SET_ICON")? };
        let w = self.wdg_mut(h, "GUI_SET_ICON")?;
        if w.kind != Kind::Button { return Err("GUI_SET_ICON: Widget ist kein button".into()); }
        w.sinnbild = n; Ok(())
    }
    /// Werkzeugleiste: ohne Eintraege ein flacher Streifen (so war sie immer),
    /// mit Eintraegen (GUI_TOOLBAR_ADD) eine Leiste, die ihre Knoepfe selbst
    /// verteilt, zeichnet und meldet.
    pub fn toolbar(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Toolbar, x, y, w, h);
        wd.enabled = false;
        self.add_widget(win, "GUI_TOOLBAR", wd)
    }
    pub(super) fn tb_mut(&mut self, h: i64, fn_: &str) -> Result<&mut LeisteState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Toolbar { return Err(format!("{}: Widget ist keine Werkzeugleiste (GUI_TOOLBAR)", fn_)); }
        Ok(w.leiste.get_or_insert_with(|| Box::new(LeisteState::default())))
    }
    pub(super) fn tb_ref(&self, h: i64, fn_: &str) -> Result<Option<&LeisteState>, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::Toolbar { return Err(format!("{}: Widget ist keine Werkzeugleiste (GUI_TOOLBAR)", fn_)); }
        Ok(w.leiste.as_deref())
    }
    pub(super) fn tb_eintrag(&mut self, h: i64, i: i64, fn_: &str) -> Result<&mut LeistenEintrag, String> {
        let l = self.tb_mut(h, fn_)?;
        let n = l.eintraege.len();
        l.eintraege.get_mut(i.max(0) as usize).filter(|_| i >= 0)
            .ok_or_else(|| format!("{}: Eintrag {} gibt es nicht ({} Eintraege)", fn_, i, n))
    }
    pub(super) fn tb_anhaengen(&mut self, h: i64, e: LeistenEintrag, fn_: &str) -> Result<i64, String> {
        let l = self.tb_mut(h, fn_)?;
        l.eintraege.push(e);
        let n = l.eintraege.len() as i64 - 1;
        // Mit Eintraegen nimmt die Leiste Klicks an -- ohne bleibt sie Deko,
        // damit Knoepfe, die ein altes Programm DARAUF legt, weiter treffen.
        self.wdg_mut(h, fn_)?.enabled = true;
        Ok(n)
    }
    /// Einen Knopf anhaengen -- `bild` ist ein eingebautes Sinnbild (Name)
    /// oder ein Textur-Handle (-1 = keins). Liefert die Nummer des Eintrags.
    pub fn toolbar_add(&mut self, h: i64, symbol: Option<String>, bild: i64, tip: String, text: String) -> Result<i64, String> {
        let symbol = match symbol {
            Some(s) if !s.is_empty() => sinnbild_pruefen(&s, "GUI_TOOLBAR_ADD")?,
            _ => String::new(),
        };
        let e = LeistenEintrag { art: 0, symbol, bild: bild.max(-1), tip, text, aus: false, kippbar: false, an: false, text_b: 0 };
        self.tb_anhaengen(h, e, "GUI_TOOLBAR_ADD")
    }
    pub(super) fn tb_leer(art: u8) -> LeistenEintrag {
        LeistenEintrag { art, symbol: String::new(), bild: -1, tip: String::new(), text: String::new(), aus: false, kippbar: false, an: false, text_b: 0 }
    }
    pub fn toolbar_separator(&mut self, h: i64) -> Result<i64, String> {
        self.tb_anhaengen(h, Self::tb_leer(1), "GUI_TOOLBAR_SEPARATOR")
    }
    pub fn toolbar_spacer(&mut self, h: i64) -> Result<i64, String> {
        self.tb_anhaengen(h, Self::tb_leer(2), "GUI_TOOLBAR_SPACER")
    }
    pub fn toolbar_clicked(&self, h: i64) -> Result<i64, String> {
        Ok(self.tb_ref(h, "GUI_TOOLBAR_CLICKED")?.map(|l| l.geklickt as i64).unwrap_or(-1))
    }
    pub fn toolbar_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.tb_ref(h, "GUI_TOOLBAR_COUNT")?.map(|l| l.eintraege.len() as i64).unwrap_or(0))
    }
    pub fn toolbar_clear(&mut self, h: i64) -> Result<(), String> {
        let l = self.tb_mut(h, "GUI_TOOLBAR_CLEAR")?;
        l.eintraege.clear(); l.hover = -1; l.gedrueckt = -1; l.geklickt = -1;
        self.wdg_mut(h, "GUI_TOOLBAR_CLEAR")?.enabled = false;
        Ok(())
    }
    pub fn toolbar_enable(&mut self, h: i64, i: i64, an: bool) -> Result<(), String> {
        self.tb_eintrag(h, i, "GUI_TOOLBAR_ENABLE")?.aus = !an; Ok(())
    }
    pub fn toolbar_enabled(&mut self, h: i64, i: i64) -> Result<bool, String> {
        Ok(!self.tb_eintrag(h, i, "GUI_TOOLBAR_ENABLED")?.aus)
    }
    pub fn toolbar_checkable(&mut self, h: i64, i: i64, an: bool) -> Result<(), String> {
        let e = self.tb_eintrag(h, i, "GUI_TOOLBAR_CHECKABLE")?;
        if e.art != 0 { return Err(format!("GUI_TOOLBAR_CHECKABLE: Eintrag {} ist kein Knopf", i)); }
        e.kippbar = an; if !an { e.an = false; } Ok(())
    }
    pub fn toolbar_set_checked(&mut self, h: i64, i: i64, an: bool) -> Result<(), String> {
        let e = self.tb_eintrag(h, i, "GUI_TOOLBAR_SET_CHECKED")?;
        if !e.kippbar { return Err(format!("GUI_TOOLBAR_SET_CHECKED: Eintrag {} ist nicht kippbar (erst GUI_TOOLBAR_CHECKABLE)", i)); }
        e.an = an; Ok(())
    }
    pub fn toolbar_checked(&mut self, h: i64, i: i64) -> Result<bool, String> {
        Ok(self.tb_eintrag(h, i, "GUI_TOOLBAR_CHECKED")?.an)
    }
    pub fn toolbar_set_icon(&mut self, h: i64, i: i64, symbol: Option<String>, bild: i64) -> Result<(), String> {
        let symbol = match symbol {
            Some(s) if !s.is_empty() => sinnbild_pruefen(&s, "GUI_TOOLBAR_SET_ICON")?,
            _ => String::new(),
        };
        let e = self.tb_eintrag(h, i, "GUI_TOOLBAR_SET_ICON")?;
        e.symbol = symbol; e.bild = if e.symbol.is_empty() { bild.max(-1) } else { -1 }; Ok(())
    }
    pub fn toolbar_set_tip(&mut self, h: i64, i: i64, tip: String) -> Result<(), String> {
        self.tb_eintrag(h, i, "GUI_TOOLBAR_SET_TIP")?.tip = tip; Ok(())
    }
    pub fn toolbar_set_text(&mut self, h: i64, i: i64, text: String) -> Result<(), String> {
        self.tb_eintrag(h, i, "GUI_TOOLBAR_SET_TEXT")?.text = text; Ok(())
    }
    /// Einstellungen: `beschriftung` (0/1, Text neben dem Sinnbild),
    /// `symbolgroesse` (Punkte, 0 = aus der Hoehe).
    pub fn toolbar_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let l = self.tb_mut(h, "GUI_TOOLBAR_SET")?;
        match key.to_lowercase().as_str() {
            "beschriftung" | "text" => l.mit_text = wert != 0.0,
            "symbolgroesse" | "symbol" => l.symbol = (wert as i32).clamp(0, 64),
            _ => return Err(format!("GUI_TOOLBAR_SET: unbekannte Einstellung '{}' -- bekannt sind: beschriftung, symbolgroesse", key)),
        }
        Ok(())
    }
    /// GUI_TOOLBAR_GET(tb, key$): liest, was GUI_TOOLBAR_SET setzt.
    pub fn toolbar_get(&self, h: i64, key: &str) -> Result<f64, String> {
        let l = self.tb_ref(h, "GUI_TOOLBAR_GET")?;
        let (mit_text, symbol) = l.map(|l| (l.mit_text, l.symbol)).unwrap_or((false, 0));
        Ok(match key.to_lowercase().as_str() {
            "beschriftung" | "text" => if mit_text { 1.0 } else { 0.0 },
            "symbolgroesse" | "symbol" => symbol as f64,
            _ => return Err(format!("GUI_TOOLBAR_GET: unbekannte Einstellung '{}' -- bekannt sind: beschriftung, symbolgroesse", key)),
        })
    }
    /// Lage eines Eintrags (x, Breite) in logischen Koordinaten, wie GUI_GET_X.
    pub fn toolbar_item_rect(&self, h: i64, i: i64) -> Result<(i64, i64), String> {
        let w = self.wdg(h, "GUI_TOOLBAR_ITEM_X")?;
        if w.kind != Kind::Toolbar { return Err("GUI_TOOLBAR_ITEM_X: Widget ist keine Werkzeugleiste (GUI_TOOLBAR)".into()); }
        let geo = self.tb_geom(w.x, w.y, w.w, w.h, w);
        match geo.iter().find(|&&(k, _, _)| k as i64 == i) {
            Some(&(_, x0, x1)) => Ok((self.unsk(x0) as i64, self.unsk(x1 - x0) as i64)),
            None => Err(format!("GUI_TOOLBAR_ITEM_X: Eintrag {} gibt es nicht", i)),
        }
    }

    /// Die SICHTBAREN Eintraege einer Werkzeugleiste (siehe `tb_layout`).
    pub(super) fn tb_geom(&self, ax: i32, ay: i32, w: i32, h: i32, wdg: &Widget) -> Vec<(usize, i32, i32)> {
        self.tb_layout(ax, ay, w, h, wdg).0
    }
    /// Wie viele Eintraege gerade im Ueberlauf-Menue stehen (0 = alles passt).
    pub fn toolbar_overflow(&self, h: i64) -> Result<i64, String> {
        let w = self.wdg(h, "GUI_TOOLBAR_OVERFLOW")?;
        if w.kind != Kind::Toolbar { return Err("GUI_TOOLBAR_OVERFLOW: Widget ist keine Werkzeugleiste (GUI_TOOLBAR)".into()); }
        let (wi, i) = Self::dec_widget(h);
        let (ax, ay, bw, bh) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        Ok(self.tb_layout(ax, ay, bw, bh, w).1.iter().filter(|&&k| w.leiste.as_ref().map(|l| l.eintraege[k].art == 0).unwrap_or(false)).count() as i64)
    }
    /// Liegt (mx, my) auf dem >>-Knopf einer Werkzeugleiste?
    pub(super) fn tb_chevron_hit(&self, wi: usize, i: usize, mx: i32, my: i32) -> bool {
        let w = &self.windows[wi].widgets[i];
        let (ax, ay, bw, bh) = self.abs_rect(wi, w);
        match self.tb_layout(ax, ay, bw, bh, w).2 {
            Some((x0, x1)) => mx >= x0 && mx < x1 && my >= ay && my < ay + bh,
            None => false,
        }
    }
    /// Das Ueberlauf-Menue: (x, y, Breite, Zeilen als (Eintrag, y oben, y unten)).
    /// Die Breite wird wie die Knoepfe an der Zeichenzahl geschaetzt -- der
    /// Treffertest hat keine Grafik.
    pub(super) fn tb_popup_geom(&self, wi: usize, i: usize) -> Option<(i32, i32, i32, Vec<(usize, i32, i32)>)> {
        let w = self.windows.get(wi)?.widgets.get(i)?;
        if !w.alive || !self.widget_shown(wi, w) { return None; }
        let l = w.leiste.as_ref()?;
        let (ax, ay, bw, bh) = self.abs_rect(wi, w);
        let (_, versteckt, chev) = self.tb_layout(ax, ay, bw, bh, w);
        let (_, cx1) = chev?;
        let zeile = self.sk(26);
        let mut breite = self.sk(160);
        let mut y = ay + bh;
        let mut zeilen = Vec::new();
        for &k in &versteckt {
            let e = &l.eintraege[k];
            if e.art == 2 { continue; }
            let hoch = if e.art == 1 { self.sk(9) } else { zeile };
            let name = if !e.text.is_empty() { &e.text } else if !e.tip.is_empty() { &e.tip } else { &e.symbol };
            breite = breite.max(self.sk(8) * name.chars().count() as i32 + self.sk(48));
            zeilen.push((k, y, y + hoch));
            y += hoch;
        }
        if zeilen.is_empty() { return None; }
        let px = (cx1 - breite).max(0);
        Some((px, ay + bh, breite, zeilen))
    }

    /// Die Eintraege einer Werkzeugleiste: (Eintrag, x links, x rechts),
    /// absolut, dazu die Eintraege, die NICHT hineinpassen, und der >>-Knopf.
    /// EINE Quelle fuer Zeichnen, Treffertest, Tooltip und Barrierefreiheit.
    /// Passt alles, bleibt die Liste der versteckten leer und es gibt keinen
    /// Knopf. Beschriftungen zaehlen, sobald `leisten_pass` sie gemessen hat.
    pub(super) fn tb_layout(&self, ax: i32, _ay: i32, w: i32, h: i32, wdg: &Widget) -> (Vec<(usize, i32, i32)>, Vec<usize>, Option<(i32, i32)>) {
        let l = match wdg.leiste.as_ref() { Some(l) => l, None => return (Vec::new(), Vec::new(), None) };
        let rand = self.sk(4);
        let luft = self.sk(2);
        let knopf = (h - self.sk(6)).max(self.sk(12));
        let breite = |e: &LeistenEintrag| -> i32 {
            match e.art {
                1 => self.sk(11),
                2 => 0,
                _ => {
                    let hat_bild = !e.symbol.is_empty() || e.bild >= 0;
                    // Ein Knopf ohne Bild zeigt seinen Text immer -- sonst
                    // stuende dort eine leere Flaeche.
                    if !e.text.is_empty() && (l.mit_text || !hat_bild) {
                        // Gemessen, sobald ein GUI_UPDATE gelaufen ist; vorher
                        // (etwa fuer GUI_TOOLBAR_ITEM_X gleich nach dem Anlegen)
                        // grosszuegig geschaetzt.
                        let tw = if e.text_b > 0 { e.text_b } else { self.sk(8) * e.text.chars().count() as i32 };
                        let tb = tw + self.sk(10);
                        if hat_bild { knopf + tb - self.sk(4) } else { tb.max(knopf) }
                    } else { knopf }
                }
            }
        };
        // Luft steht ZWISCHEN Eintraegen, nicht hinter dem letzten -- sonst
        // endete ein rechtsbuendiger Knopf zwei Punkte vor dem Rand.
        let fest: i32 = l.eintraege.iter().map(|e| breite(e) + luft).sum::<i32>() - if l.eintraege.is_empty() { 0 } else { luft };
        let luecken = l.eintraege.iter().filter(|e| e.art == 2).count() as i32;
        if fest <= w - 2 * rand {
            let rest = (w - 2 * rand - fest).max(0);
            let mut x = ax + rand;
            let mut out = Vec::with_capacity(l.eintraege.len());
            let mut verteilt = 0;
            let mut nr = 0;
            for (k, e) in l.eintraege.iter().enumerate() {
                let mut b = breite(e);
                if e.art == 2 && luecken > 0 {
                    nr += 1;
                    b = if nr == luecken { rest - verteilt } else { rest / luecken };
                    verteilt += b;
                }
                out.push((k, x, x + b));
                x += b + luft;
            }
            return (out, Vec::new(), None);
        }
        // Zu eng: von vorn so viele, wie neben dem >>-Knopf Platz haben; alles
        // danach wandert ins Menue. Luecken fallen weg -- es ist ohnehin kein
        // Platz uebrig, den sie verteilen koennten.
        let chev = knopf.min(self.sk(22)).max(self.sk(16));
        let grenze = ax + w - rand - chev - luft;
        let mut x = ax + rand;
        let mut out = Vec::new();
        let mut versteckt = Vec::new();
        let mut voll = false;
        for (k, e) in l.eintraege.iter().enumerate() {
            if e.art == 2 { continue; }
            let b = breite(e);
            if !voll && x + b <= grenze {
                out.push((k, x, x + b));
                x += b + luft;
            } else {
                voll = true;
                versteckt.push(k);
            }
        }
        // Ein Trenner am Ende der sichtbaren Reihe oder am Anfang des Menues
        // saehe aus wie ein Fehler.
        while out.last().map(|&(k, _, _)| l.eintraege[k].art == 1).unwrap_or(false) { out.pop(); }
        while versteckt.first().map(|&k| l.eintraege[k].art == 1).unwrap_or(false) { versteckt.remove(0); }
        let cx = ax + w - rand - chev;
        (out, versteckt, Some((cx, cx + chev)))
    }

    // --- Statusleiste (GUI_STATUSBAR) ---------------------------------------
    pub fn statusbar(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, text: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::StatusBar, x, y, w, h);
        wd.text = text.clone();
        wd.status = Some(Box::new(StatusState {
            felder: vec![StatusFeld { text, tip: String::new(), breite: 0, ausr: 0, klickbar: false }],
            ..Default::default()
        }));
        self.add_widget(win, "GUI_STATUSBAR", wd)
    }
    pub(super) fn sb_mut(&mut self, h: i64, fn_: &str) -> Result<&mut Widget, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::StatusBar { return Err(format!("{}: Widget ist keine Statusleiste (GUI_STATUSBAR)", fn_)); }
        if w.status.is_none() { w.status = Some(Box::new(StatusState::default())); }
        Ok(w)
    }
    pub(super) fn sb_feld(&mut self, h: i64, i: i64, fn_: &str) -> Result<&mut StatusFeld, String> {
        let w = self.sb_mut(h, fn_)?;
        let s = w.status.as_mut().unwrap();
        let n = s.felder.len();
        s.felder.get_mut(i.max(0) as usize).filter(|_| i >= 0)
            .ok_or_else(|| format!("{}: Feld {} gibt es nicht ({} Felder)", fn_, i, n))
    }
    pub(super) fn ausrichtung_lesen(a: &str, fn_: &str) -> Result<u8, String> {
        Ok(match a.trim().to_lowercase().as_str() {
            "" | "links" | "left" => 0,
            "mitte" | "center" => 1,
            "rechts" | "right" => 2,
            _ => return Err(format!("{}: Ausrichtung '{}' gibt es nicht -- links, mitte, rechts", fn_, a)),
        })
    }
    /// Ein Feld anhaengen: `breite` in Punkten, 0 = teilt sich den Rest.
    pub fn statusbar_add(&mut self, h: i64, text: String, breite: i64, ausr: &str) -> Result<i64, String> {
        let a = Self::ausrichtung_lesen(ausr, "GUI_STATUSBAR_ADD")?;
        if breite < 0 { return Err("GUI_STATUSBAR_ADD: Breite muss >= 0 sein (0 = teilt sich den Rest)".into()); }
        let s = self.sb_mut(h, "GUI_STATUSBAR_ADD")?.status.as_mut().unwrap();
        s.felder.push(StatusFeld { text, tip: String::new(), breite: breite as i32, ausr: a, klickbar: false });
        Ok(s.felder.len() as i64 - 1)
    }
    pub fn statusbar_set(&mut self, h: i64, i: i64, text: String) -> Result<(), String> {
        self.sb_feld(h, i, "GUI_STATUSBAR_SET")?.text = text.clone();
        if i == 0 { self.wdg_mut(h, "GUI_STATUSBAR_SET")?.text = text; }
        Ok(())
    }
    pub fn statusbar_text(&mut self, h: i64, i: i64) -> Result<String, String> {
        Ok(self.sb_feld(h, i, "GUI_STATUSBAR_TEXT")?.text.clone())
    }
    pub fn statusbar_tip(&mut self, h: i64, i: i64, tip: String) -> Result<(), String> {
        self.sb_feld(h, i, "GUI_STATUSBAR_TIP")?.tip = tip; Ok(())
    }
    /// GUI_STATUSBAR_TIP$(sb, feld): der Hilfetext eines Feldes.
    pub fn statusbar_tip_text(&mut self, h: i64, i: i64) -> Result<String, String> {
        Ok(self.sb_feld(h, i, "GUI_STATUSBAR_TIP$")?.tip.clone())
    }
    pub fn statusbar_clickable(&mut self, h: i64, i: i64, an: bool) -> Result<(), String> {
        self.sb_feld(h, i, "GUI_STATUSBAR_CLICKABLE")?.klickbar = an; Ok(())
    }
    pub fn statusbar_clicked(&mut self, h: i64) -> Result<i64, String> {
        Ok(self.sb_mut(h, "GUI_STATUSBAR_CLICKED")?.status.as_ref().unwrap().geklickt as i64)
    }
    pub fn statusbar_count(&mut self, h: i64) -> Result<i64, String> {
        Ok(self.sb_mut(h, "GUI_STATUSBAR_COUNT")?.status.as_ref().unwrap().felder.len() as i64)
    }
    pub fn statusbar_field_rect(&self, h: i64, i: i64) -> Result<(i64, i64), String> {
        let w = self.wdg(h, "GUI_STATUSBAR_FIELD_X")?;
        if w.kind != Kind::StatusBar { return Err("GUI_STATUSBAR_FIELD_X: Widget ist keine Statusleiste (GUI_STATUSBAR)".into()); }
        match self.sb_geom(w.x, w.w, w).iter().find(|&&(k, _, _)| k as i64 == i) {
            Some(&(_, x0, x1)) => Ok((self.unsk(x0) as i64, self.unsk(x1 - x0) as i64)),
            None => Err(format!("GUI_STATUSBAR_FIELD_X: Feld {} gibt es nicht", i)),
        }
    }
    /// Die Felder: (Feld, x links, x rechts). Feste Breiten zuerst, der Rest
    /// wird unter den Feldern mit Breite 0 geteilt.
    pub(super) fn sb_geom(&self, ax: i32, w: i32, wdg: &Widget) -> Vec<(usize, i32, i32)> {
        let s = match wdg.status.as_ref() { Some(s) => s, None => return Vec::new() };
        let fest: i32 = s.felder.iter().map(|f| self.sk(f.breite)).sum();
        let flex = s.felder.iter().filter(|f| f.breite == 0).count() as i32;
        let rest = (w - fest).max(0);
        let mut x = ax;
        let mut nr = 0;
        let mut verteilt = 0;
        s.felder.iter().enumerate().map(|(k, f)| {
            let b = if f.breite > 0 { self.sk(f.breite) } else {
                nr += 1;
                let b = if nr == flex { rest - verteilt } else { rest / flex.max(1) };
                verteilt += b;
                b
            };
            let r = (k, x, x + b);
            x += b;
            r
        }).collect()
    }
    pub(super) fn sb_treffer(&self, wi: usize, i: usize, mx: i32) -> i32 {
        let w = &self.windows[wi].widgets[i];
        let (ax, _, bw, _) = self.abs_rect(wi, w);
        self.sb_geom(ax, bw, w).into_iter().find(|&(_, x0, x1)| mx >= x0 && mx < x1).map(|(k, _, _)| k as i32).unwrap_or(-1)
    }

    // --- Pfadleiste (GUI_BREADCRUMB) ----------------------------------------
    pub fn breadcrumb(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Breadcrumb, x, y, w, h);
        wd.pfad = Some(Box::new(PfadState::default()));
        self.add_widget(win, "GUI_BREADCRUMB", wd)
    }
    pub(super) fn pf_mut(&mut self, h: i64, fn_: &str) -> Result<&mut PfadState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Breadcrumb { return Err(format!("{}: Widget ist keine Pfadleiste (GUI_BREADCRUMB)", fn_)); }
        Ok(w.pfad.get_or_insert_with(|| Box::new(PfadState::default())))
    }
    pub(super) fn pf_idx(p: &PfadState, i: i64, fn_: &str) -> Result<usize, String> {
        if i < 0 || i as usize >= p.texte.len() { return Err(format!("{}: Teil {} gibt es nicht ({} Teile)", fn_, i, p.texte.len())); }
        Ok(i as usize)
    }
    pub fn breadcrumb_add(&mut self, h: i64, text: String, wert: String) -> Result<i64, String> {
        let p = self.pf_mut(h, "GUI_BREADCRUMB_ADD")?;
        p.texte.push(text); p.daten.push(wert); p.breiten.push(0);
        Ok(p.texte.len() as i64 - 1)
    }
    pub fn breadcrumb_set(&mut self, h: i64, texte: Vec<String>) -> Result<(), String> {
        let p = self.pf_mut(h, "GUI_BREADCRUMB_SET")?;
        // Ein unveraenderter Pfad bleibt, wie er ist -- sonst ginge je Bild
        // die Messung verloren, und ein Programm setzt ihn gern je Bild.
        if p.texte == texte { return Ok(()); }
        let n = texte.len();
        p.texte = texte; p.daten = vec![String::new(); n]; p.breiten = vec![0; n];
        Ok(())
    }
    pub fn breadcrumb_clear(&mut self, h: i64) -> Result<(), String> {
        let p = self.pf_mut(h, "GUI_BREADCRUMB_CLEAR")?;
        p.texte.clear(); p.daten.clear(); p.breiten.clear(); Ok(())
    }
    pub fn breadcrumb_count(&mut self, h: i64) -> Result<i64, String> {
        Ok(self.pf_mut(h, "GUI_BREADCRUMB_COUNT")?.texte.len() as i64)
    }
    pub fn breadcrumb_clicked(&mut self, h: i64) -> Result<i64, String> {
        Ok(self.pf_mut(h, "GUI_BREADCRUMB_CLICKED")?.geklickt as i64)
    }
    pub fn breadcrumb_text(&mut self, h: i64, i: i64) -> Result<String, String> {
        let p = self.pf_mut(h, "GUI_BREADCRUMB_TEXT")?;
        Ok(p.texte[Self::pf_idx(p, i, "GUI_BREADCRUMB_TEXT")?].clone())
    }
    pub fn breadcrumb_data(&mut self, h: i64, i: i64) -> Result<String, String> {
        let p = self.pf_mut(h, "GUI_BREADCRUMB_DATA")?;
        Ok(p.daten[Self::pf_idx(p, i, "GUI_BREADCRUMB_DATA")?].clone())
    }
    /// Der erste Teil, der zu sehen ist -- davor steht das "...".
    pub fn breadcrumb_first_visible(&self, h: i64) -> Result<i64, String> {
        let w = self.wdg(h, "GUI_BREADCRUMB_FIRST_VISIBLE")?;
        if w.kind != Kind::Breadcrumb { return Err("GUI_BREADCRUMB_FIRST_VISIBLE: Widget ist keine Pfadleiste (GUI_BREADCRUMB)".into()); }
        let (geo, _) = self.pf_geom(w.x, w.w, w);
        Ok(geo.first().map(|&(k, _, _)| k as i64).unwrap_or(0))
    }
    /// Die sichtbaren Teile (Teil, x links, x rechts) und das "..." davor,
    /// falls die vorderen nicht hineinpassen.
    pub(super) fn pf_geom(&self, ax: i32, w: i32, wdg: &Widget) -> (Vec<(usize, i32, i32)>, Option<(i32, i32)>) {
        let p = match wdg.pfad.as_ref() { Some(p) => p, None => return (Vec::new(), None) };
        let n = p.texte.len();
        if n == 0 { return (Vec::new(), None); }
        let rand = self.sk(2);
        let innen = self.sk(12);
        let trenn = self.sk(16);
        let breite = |k: usize| -> i32 {
            let t = p.breiten.get(k).copied().filter(|&b| b > 0)
                .unwrap_or_else(|| self.sk(8) * p.texte[k].chars().count() as i32);
            t + innen
        };
        let gesamt: i32 = (0..n).map(breite).sum::<i32>() + trenn * (n as i32 - 1);
        let mut erster = 0;
        let mut punkte = None;
        if gesamt > w - 2 * rand {
            // Vom LETZTEN Teil rueckwaerts, solange es neben das "..." passt;
            // der letzte steht immer da, auch wenn er selbst zu breit ist.
            let pw = self.sk(24);
            let platz = w - 2 * rand - pw - trenn;
            erster = n - 1;
            let mut belegt = breite(n - 1);
            while erster > 0 {
                let dazu = breite(erster - 1) + trenn;
                if belegt + dazu > platz { break; }
                belegt += dazu;
                erster -= 1;
            }
            if erster > 0 { punkte = Some((ax + rand, ax + rand + pw)); }
        }
        let mut x = match punkte { Some((_, x1)) => x1 + trenn, None => ax + rand };
        let mut out = Vec::new();
        for k in erster..n {
            let b = breite(k);
            out.push((k, x, x + b));
            x += b + trenn;
        }
        (out, punkte)
    }
    /// Ein Teil der Pfadleiste ist geklickt worden (Maus und Hilfsprogramm).
    pub(super) fn pf_klick(&mut self, wi: usize, i: usize, k: i32) {
        let w = &mut self.windows[wi].widgets[i];
        let n = w.pfad.as_ref().map(|p| p.texte.len()).unwrap_or(0);
        if k < 0 || k as usize >= n { return; }
        if let Some(p) = w.pfad.as_mut() { p.geklickt = k; }
        w.clicked = true;
        let f = w.on_click.clone();
        if let Some(f) = f { self.pending.push(f); }
    }
    /// Welcher Teil liegt unter der Maus? -2 = das "...", -1 = keiner.
    pub(super) fn pf_treffer(&self, wi: usize, i: usize, mx: i32) -> i32 {
        let w = &self.windows[wi].widgets[i];
        let (ax, _, bw, _) = self.abs_rect(wi, w);
        let (geo, punkte) = self.pf_geom(ax, bw, w);
        if let Some((x0, x1)) = punkte { if mx >= x0 && mx < x1 { return -2; } }
        geo.into_iter().find(|&(_, x0, x1)| mx >= x0 && mx < x1).map(|(k, _, _)| k as i32).unwrap_or(-1)
    }
    /// Welcher KNOPF liegt unter der Maus? -1 = keiner (Trenner, Luecke, daneben).
    pub(super) fn tb_treffer(&self, wi: usize, i: usize, mx: i32, my: i32) -> i32 {
        let w = &self.windows[wi].widgets[i];
        let (ax, ay, bw, bh) = self.abs_rect(wi, w);
        if my < ay || my >= ay + bh { return -1; }
        let l = match w.leiste.as_ref() { Some(l) => l, None => return -1 };
        self.tb_geom(ax, ay, bw, bh, w).into_iter()
            .find(|&(k, x0, x1)| mx >= x0 && mx < x1 && l.eintraege[k].art == 0)
            .map(|(k, _, _)| k as i32).unwrap_or(-1)
    }
    /// Ein Knopf ist geklickt worden -- auf demselben Weg fuer Maus und
    /// Hilfsprogramm. Gesperrte Knoepfe tun nichts.
    pub(super) fn tb_klick(&mut self, wi: usize, i: usize, k: i32) {
        let w = &mut self.windows[wi].widgets[i];
        let ok = match w.leiste.as_mut().and_then(|l| l.eintraege.get_mut(k.max(0) as usize)) {
            Some(e) if k >= 0 && e.art == 0 && !e.aus => { if e.kippbar { e.an = !e.an; } true }
            _ => false,
        };
        if !ok { return; }
        if let Some(l) = w.leiste.as_mut() { l.geklickt = k; }
        w.clicked = true;
        let f = w.on_click.clone();
        if let Some(f) = f { self.pending.push(f); }
    }
    pub fn canvas(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let wd = Self::blank(Kind::Canvas, x, y, w, h);
        self.add_widget(win, "GUI_CANVAS", wd)
    }
    /// Absoluter Bildschirm-Bereich (Inhaltsflaeche) eines Canvas, in den der User
    /// nach GUI_DRAW mit normalen Zeichenbefehlen malen kann (selbst clippen).
    pub fn canvas_rect(&self, h: i64) -> Result<(i32, i32, i32, i32), String> {
        let (wi, _) = Self::dec_widget(h);
        let w = self.wdg(h, "GUI_CANVAS_*")?;
        if w.kind != Kind::Canvas { return Err("GUI_CANVAS_*: Widget ist kein canvas".into()); }
        let (ax, ay, ww, wh) = self.abs_rect(wi, w);
        Ok((ax + 1, ay + 1, (ww - 2).max(0), (wh - 2).max(0)))   // Inhalt ohne Rahmen
    }
    // Item-Auswahl: Dropdown UND ListBox (beide nutzen items/sel).
    pub(super) fn item_widget(&self, h: i64, fn_: &str) -> Result<&Widget, String> {
        let w = self.wdg(h, fn_)?;
        if !matches!(w.kind, Kind::Dropdown | Kind::ListBox) { return Err(format!("{}: Widget ist kein dropdown/listbox", fn_)); }
        Ok(w)
    }
    pub fn dropdown_selected(&self, h: i64) -> Result<i64, String> {
        Ok(self.item_widget(h, "GUI_DROPDOWN_SELECTED")?.sel as i64)
    }
    pub fn dropdown_text(&self, h: i64) -> Result<String, String> {
        let w = self.item_widget(h, "GUI_DROPDOWN_TEXT")?;
        if w.kind == Kind::Dropdown && w.frei { return Ok(w.text.clone()); }
        Ok(if w.sel >= 0 && (w.sel as usize) < w.items.len() { w.items[w.sel as usize].clone() } else { String::new() })
    }
    pub fn dropdown_set_selected(&mut self, h: i64, idx: i64) -> Result<(), String> {
        self.item_widget(h, "GUI_DROPDOWN_SET_SELECTED")?;
        let w = self.wdg_mut(h, "GUI_DROPDOWN_SET_SELECTED")?;
        w.sel = if idx >= 0 && (idx as usize) < w.items.len() { idx as i32 } else { -1 };
        dd_frei_text_setzen(w);
        Ok(())
    }
    pub fn set_dropdown_items(&mut self, h: i64, items: Vec<String>) -> Result<(), String> {
        self.item_widget(h, "GUI_SET_DROPDOWN")?;
        let w = self.wdg_mut(h, "GUI_SET_DROPDOWN")?;
        w.sel = if items.is_empty() { -1 } else { w.sel.min(items.len() as i32 - 1) };
        // Symbole, Haken, Farben und Auswahl gehoerten zu den ALTEN Eintraegen.
        if let Some(l) = w.list.as_mut() {
            let n = items.len();
            l.icons = vec![-1; n]; l.checks = vec![false; n]; l.colors = vec![-1; n]; l.sel = vec![false; n];
            l.daten = vec![String::new(); n]; l.details = vec![String::new(); n]; l.tips = vec![String::new(); n];
            l.aus = vec![false; n]; l.kopf = vec![false; n];
            l.anker = -1;
        }
        w.items = items; w.value = 0.0;
        // Bearbeitbar: der Text bleibt, was er war -- ob er jetzt ein Eintrag
        // ist, entscheiden die NEUEN Eintraege.
        if w.kind == Kind::Dropdown && w.frei { dd_frei_abgleichen(w); }
        Ok(())
    }

    // ---- Listen-Ausbau (Punkt 3, 2026-09-04) ------------------------------

    /// Den Zusatzzustand einer ListBox holen (bei Bedarf anlegen).
    pub(super) fn list_mut(&mut self, h: i64, fn_: &str) -> Result<&mut ListState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::ListBox { return Err(format!("{}: das Widget ist keine Liste (GUI_LISTBOX)", fn_)); }
        let n = w.items.len();
        let l = w.list.get_or_insert_with(|| Box::new(ListState { anker: -1, ..Default::default() }));
        l.sync(n);
        Ok(l)
    }
    pub(super) fn list_idx(w: &Widget, i: i64, fn_: &str) -> Result<usize, String> {
        if i < 0 || i as usize >= w.items.len() {
            return Err(format!("{}: Eintrag {} gibt es nicht (die Liste hat {})", fn_, i, w.items.len()));
        }
        Ok(i as usize)
    }
    /// GUI_LISTBOX_ADD(lb, text$[, pos]) -> Index. Auch fuer Klapplisten.
    pub fn list_add(&mut self, h: i64, text: String, pos: i64) -> Result<i64, String> {
        self.item_widget(h, "GUI_LISTBOX_ADD")?;
        let w = self.wdg_mut(h, "GUI_LISTBOX_ADD")?;
        let n = w.items.len();
        let p = if pos < 0 || pos as usize > n { n } else { pos as usize };
        w.items.insert(p, text);
        if let Some(l) = w.list.as_mut() { l.sync(n); l.einfuegen(p); }
        // Eine Auswahl HINTER der Einfuegestelle rueckt mit -- sie meint
        // denselben Eintrag wie vorher.
        if w.sel >= p as i32 { w.sel += 1; }
        Ok(p as i64)
    }
    /// GUI_LISTBOX_REMOVE(lb, i)
    pub fn list_remove(&mut self, h: i64, i: i64) -> Result<(), String> {
        self.item_widget(h, "GUI_LISTBOX_REMOVE")?;
        let w = self.wdg_mut(h, "GUI_LISTBOX_REMOVE")?;
        let p = Self::list_idx(w, i, "GUI_LISTBOX_REMOVE")?;
        let n = w.items.len();
        w.items.remove(p);
        if let Some(l) = w.list.as_mut() { l.sync(n); l.entfernen(p); if l.anker >= p as i32 { l.anker -= 1; } }
        if w.sel == p as i32 { w.sel = -1; } else if w.sel > p as i32 { w.sel -= 1; }
        Ok(())
    }
    /// GUI_LISTBOX_CLEAR(lb)
    pub fn list_clear(&mut self, h: i64) -> Result<(), String> {
        self.set_dropdown_items(h, Vec::new())
    }
    /// GUI_DROPDOWN_COUNT / GUI_DROPDOWN_ITEM: die Eintraege einer
    /// Klappliste lesen. Die Liste-Befehle konnten es schon, aber unter dem
    /// Namen der Klappliste suchte man vergeblich (gefunden mit der
    /// Kuehlschrank-App: welche Vorschlaege stehen gerade darin?).
    pub fn dropdown_count(&self, h: i64) -> Result<i64, String> {
        let w = self.wdg(h, "GUI_DROPDOWN_COUNT")?;
        if w.kind != Kind::Dropdown { return Err("GUI_DROPDOWN_COUNT: Widget ist keine Klappliste".into()); }
        Ok(w.items.len() as i64)
    }
    pub fn dropdown_item(&self, h: i64, i: i64) -> Result<String, String> {
        let w = self.wdg(h, "GUI_DROPDOWN_ITEM")?;
        if w.kind != Kind::Dropdown { return Err("GUI_DROPDOWN_ITEM: Widget ist keine Klappliste".into()); }
        Ok(w.items[Self::list_idx(w, i, "GUI_DROPDOWN_ITEM")?].clone())
    }
    /// GUI_LISTBOX_COUNT(lb)
    pub fn list_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.item_widget(h, "GUI_LISTBOX_COUNT")?.items.len() as i64)
    }
    /// GUI_LISTBOX_ITEM(lb, i) -> Text
    pub fn list_item(&self, h: i64, i: i64) -> Result<String, String> {
        let w = self.item_widget(h, "GUI_LISTBOX_ITEM")?;
        Ok(w.items[Self::list_idx(w, i, "GUI_LISTBOX_ITEM")?].clone())
    }
    /// GUI_LISTBOX_SET_ITEM(lb, i, text$)
    pub fn list_set_item(&mut self, h: i64, i: i64, text: String) -> Result<(), String> {
        self.item_widget(h, "GUI_LISTBOX_SET_ITEM")?;
        let w = self.wdg_mut(h, "GUI_LISTBOX_SET_ITEM")?;
        let p = Self::list_idx(w, i, "GUI_LISTBOX_SET_ITEM")?;
        w.items[p] = text; Ok(())
    }
    /// GUI_LISTBOX_MOVE(lb, von, nach) -- herausnehmen und einsetzen, nicht
    /// tauschen (wie TILED_MOVE_LAYER: ein Sprung ueber mehrere Plaetze ist
    /// sonst etwas anderes). Symbol, Haken, Farbe und Auswahl reisen mit.
    pub fn list_move(&mut self, h: i64, von: i64, nach: i64) -> Result<(), String> {
        self.item_widget(h, "GUI_LISTBOX_MOVE")?;
        let w = self.wdg_mut(h, "GUI_LISTBOX_MOVE")?;
        let a = Self::list_idx(w, von, "GUI_LISTBOX_MOVE")?;
        let b = Self::list_idx(w, nach, "GUI_LISTBOX_MOVE")?;
        if a == b { return Ok(()); }
        let t = w.items.remove(a); w.items.insert(b, t);
        if let Some(l) = w.list.as_mut() {
            l.sync(w.items.len());
            l.tragen(a, b);
        }
        if w.sel == a as i32 { w.sel = b as i32; }
        else if a < b && w.sel > a as i32 && w.sel <= b as i32 { w.sel -= 1; }
        else if a > b && w.sel >= b as i32 && w.sel < a as i32 { w.sel += 1; }
        Ok(())
    }
    /// GUI_LISTBOX_SET(lb, key$, wert): `mehrfachauswahl`, `kaestchen`.
    pub fn list_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let l = self.list_mut(h, "GUI_LISTBOX_SET")?;
        match key.to_lowercase().as_str() {
            "mehrfachauswahl" | "multi_select" | "multi" => { l.multi = wert != 0.0; if !l.multi { for x in l.sel.iter_mut() { *x = false; } } }
            "kaestchen" | "checkboxes" => l.kaestchen = wert != 0.0,
            "bearbeitbar" | "editable" => { l.bearbeitbar = wert != 0.0; if !l.bearbeitbar { l.edit = None; } }
            _ => return Err(format!("GUI_LISTBOX_SET: unbekannter Schluessel '{}' -- mehrfachauswahl, kaestchen, bearbeitbar", key)),
        }
        Ok(())
    }
    pub fn list_icon(&mut self, h: i64, i: i64, tex: i64) -> Result<(), String> {
        let p = Self::list_idx(self.wdg(h, "GUI_LISTBOX_ICON")?, i, "GUI_LISTBOX_ICON")?;
        self.list_mut(h, "GUI_LISTBOX_ICON")?.icons[p] = tex.max(-1); Ok(())
    }
    pub fn list_color(&mut self, h: i64, i: i64, farbe: i64) -> Result<(), String> {
        let p = Self::list_idx(self.wdg(h, "GUI_LISTBOX_COLOR")?, i, "GUI_LISTBOX_COLOR")?;
        self.list_mut(h, "GUI_LISTBOX_COLOR")?.colors[p] = farbe.max(-1); Ok(())
    }
    pub fn list_checked(&self, h: i64, i: i64) -> Result<bool, String> {
        let w = self.item_widget(h, "GUI_LISTBOX_CHECKED")?;
        let p = Self::list_idx(w, i, "GUI_LISTBOX_CHECKED")?;
        Ok(w.list.as_ref().and_then(|l| l.checks.get(p).copied()).unwrap_or(false))
    }
    pub fn list_set_checked(&mut self, h: i64, i: i64, an: bool) -> Result<(), String> {
        let p = Self::list_idx(self.wdg(h, "GUI_LISTBOX_SET_CHECKED")?, i, "GUI_LISTBOX_SET_CHECKED")?;
        self.list_mut(h, "GUI_LISTBOX_SET_CHECKED")?.checks[p] = an; Ok(())
    }
    /// GUI_LISTBOX_IS_SELECTED(lb, i) -- in der Mehrfachauswahl die Menge,
    /// sonst die eine gewaehlte Zeile.
    pub fn list_is_selected(&self, h: i64, i: i64) -> Result<bool, String> {
        let w = self.item_widget(h, "GUI_LISTBOX_IS_SELECTED")?;
        let p = Self::list_idx(w, i, "GUI_LISTBOX_IS_SELECTED")?;
        Ok(match w.list.as_ref() {
            Some(l) if l.multi => l.sel.get(p).copied().unwrap_or(false),
            _ => w.sel == p as i32,
        })
    }
    pub fn list_select(&mut self, h: i64, i: i64, an: bool) -> Result<(), String> {
        let p = Self::list_idx(self.wdg(h, "GUI_LISTBOX_SELECT")?, i, "GUI_LISTBOX_SELECT")?;
        let multi = self.list_mut(h, "GUI_LISTBOX_SELECT")?.multi;
        let w = self.wdg_mut(h, "GUI_LISTBOX_SELECT")?;
        if multi {
            let l = w.list.as_mut().unwrap();
            l.sel[p] = an;
            if an { w.sel = p as i32; l.anker = p as i32; }
        } else {
            w.sel = if an { p as i32 } else if w.sel == p as i32 { -1 } else { w.sel };
        }
        Ok(())
    }
    pub fn list_sel_count(&self, h: i64) -> Result<i64, String> {
        let w = self.item_widget(h, "GUI_LISTBOX_SEL_COUNT")?;
        Ok(match w.list.as_ref() {
            Some(l) if l.multi => l.sel.iter().filter(|&&x| x).count() as i64,
            _ => if w.sel >= 0 { 1 } else { 0 },
        })
    }
    /// GUI_LISTBOX_SEL_ROW(lb, k) -- der k-te gewaehlte Eintrag (0-basiert),
    /// in Listenreihenfolge; -1 wenn es ihn nicht gibt.
    pub fn list_sel_row(&self, h: i64, k: i64) -> Result<i64, String> {
        let w = self.item_widget(h, "GUI_LISTBOX_SEL_ROW")?;
        Ok(match w.list.as_ref() {
            Some(l) if l.multi => l.sel.iter().enumerate().filter(|(_, &x)| x).nth(k.max(0) as usize)
                                    .map(|(i, _)| i as i64).unwrap_or(-1),
            _ => if k == 0 { w.sel as i64 } else { -1 },
        })
    }
    pub fn list_clear_selection(&mut self, h: i64) -> Result<(), String> {
        self.item_widget(h, "GUI_LISTBOX_CLEAR_SELECTION")?;
        let w = self.wdg_mut(h, "GUI_LISTBOX_CLEAR_SELECTION")?;
        w.sel = -1;
        if let Some(l) = w.list.as_mut() { for x in l.sel.iter_mut() { *x = false; } l.anker = -1; }
        Ok(())
    }
    /// GUI_DOUBLE_CLICKED(lb) -- ein Bild lang, wie GUI_CLICKED.
    pub fn double_clicked(&self, h: i64) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_DOUBLE_CLICKED")?;
        match w.kind {
            Kind::ListBox => Ok(w.list.as_ref().map(|l| l.doppel).unwrap_or(false)),
            Kind::Table => Ok(w.tbl.as_ref().map(|t| t.doppel).unwrap_or(false)),
            Kind::Tree => Ok(w.tree.as_ref().map(|t| t.doppel).unwrap_or(false)),
            _ => Err("GUI_DOUBLE_CLICKED: nur fuer Listen, Tabellen und Baeume (GUI_LISTBOX, GUI_TABLE, GUI_TREE)".into()),
        }
    }
    /// Zeilenhoehe einer Liste: 22 Punkte, mit eigener Schriftgroesse
    /// (GUI_SET_FONT_SIZE) mindestens Schrift + 6 -- sonst stiessen groessere
    /// Buchstaben oben und unten an. Ohne eigene Groesse bleibt es, wie es war.
    pub(super) fn list_zeile_h(&self, w: &Widget) -> i32 {
        // Mit Innenabstand aus der Stildatei: Schrift plus der Abstand oben
        // und unten -- genau, ohne Mindestmass, wie beim Knopf.
        if let Some((v, _)) = self.w_abstand(w) {
            let schrift = match self.w_groesse(w) { 0 => DROPDOWN_ITEM_H - 6, n => n };
            return (self.sk(schrift) + 2 * v).max(self.sk(8));
        }
        if w.font_size > 0 { self.sk(DROPDOWN_ITEM_H.max(w.font_size + 6)) } else { self.sk(DROPDOWN_ITEM_H) }
    }
    /// Breite der Kaestchen-Spalte einer Liste (0 ohne Kaestchen).
    pub(super) fn list_box_w(&self, w: &Widget) -> i32 {
        if w.list.as_ref().map(|l| l.kaestchen).unwrap_or(false) { self.sk(DROPDOWN_ITEM_H) } else { 0 }
    }

    // ---- Stufe 27: was eine Liste komfortabel macht -----------------------

    pub(super) fn list_ref(&self, h: i64, fn_: &str) -> Result<&Widget, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::ListBox { return Err(format!("{}: das Widget ist keine Liste (GUI_LISTBOX)", fn_)); }
        Ok(w)
    }
    /// Liste ODER Klappliste -- die Werte je Eintrag gelten fuer beide.
    pub(super) fn werte_ref(&self, h: i64, fn_: &str) -> Result<&Widget, String> {
        let w = self.wdg(h, fn_)?;
        if !matches!(w.kind, Kind::ListBox | Kind::Dropdown) {
            return Err(format!("{}: das Widget ist keine Liste und keine Klappliste", fn_));
        }
        Ok(w)
    }
    /// Unsichtbarer Wert je Eintrag -- die ID einer Datenbankzeile, ein Pfad,
    /// ein Steuersatz. Der TEXT ist fuer Menschen und aendert sich; der Wert
    /// bleibt. Auch fuer Klapplisten: eine gebundene Klappliste mit Werten
    /// bindet den WERT des gewaehlten Eintrags statt seiner Nummer.
    pub fn list_set_data(&mut self, h: i64, i: i64, wert: String) -> Result<(), String> {
        let p = Self::list_idx(self.werte_ref(h, "GUI_LISTBOX_SET_DATA")?, i, "GUI_LISTBOX_SET_DATA")?;
        let w = self.wdg_mut(h, "GUI_LISTBOX_SET_DATA")?;
        let n = w.items.len();
        let l = w.list.get_or_insert_with(|| Box::new(ListState { anker: -1, ..Default::default() }));
        l.sync(n);
        l.daten[p] = wert;
        Ok(())
    }
    pub fn list_data(&self, h: i64, i: i64) -> Result<String, String> {
        let w = self.werte_ref(h, "GUI_LISTBOX_DATA")?;
        let p = Self::list_idx(w, i, "GUI_LISTBOX_DATA")?;
        Ok(w.list.as_ref().and_then(|l| l.daten.get(p).cloned()).unwrap_or_default())
    }
    /// Der erste Eintrag mit diesem Wert, -1 = keiner.
    pub fn list_find_data(&self, h: i64, wert: &str) -> Result<i64, String> {
        let w = self.werte_ref(h, "GUI_LISTBOX_FIND_DATA")?;
        Ok(w.list.as_ref().and_then(|l| l.daten.iter().position(|d| d == wert)).map(|p| p as i64).unwrap_or(-1))
    }
    /// GUI_LISTBOX_SPANS: Abschnitte eines Eintrags einfaerben (Zeichen
    /// ab 0). Leere Felder nehmen die Farben wieder weg. Die Abschnitte
    /// gehoeren zum EINTRAG und wandern beim Sortieren, Verschieben und
    /// Loeschen mit wie alles andere an ihm.
    pub fn list_spans(&mut self, h: i64, i: i64, starts: Vec<i64>, laengen: Vec<i64>,
                      farben: Vec<i64>) -> Result<(), String> {
        let f = "GUI_LISTBOX_SPANS";
        if starts.len() != laengen.len() || starts.len() != farben.len() {
            return Err(format!("{}: starts, laengen und farben muessen gleich lang sein ({}, {}, {})",
                f, starts.len(), laengen.len(), farben.len()));
        }
        let p = Self::list_idx(self.list_ref(h, f)?, i, f)?;
        let mut v: Vec<(usize, usize, i64)> = starts.iter().zip(laengen.iter()).zip(farben.iter())
            .filter(|((&s, &l), _)| s >= 0 && l > 0)
            .map(|((&s, &l), &c)| (s as usize, l as usize, c)).collect();
        v.sort_by_key(|x| x.0);
        self.list_mut(h, f)?.spans[p] = v;
        Ok(())
    }
    pub fn list_detail(&mut self, h: i64, i: i64, text: String) -> Result<(), String> {
        let p = Self::list_idx(self.list_ref(h, "GUI_LISTBOX_DETAIL")?, i, "GUI_LISTBOX_DETAIL")?;
        self.list_mut(h, "GUI_LISTBOX_DETAIL")?.details[p] = text; Ok(())
    }
    /// GUI_LISTBOX_ITEM_RECT: wo ein Eintrag auf dem Bildschirm steht (x, y, b, h),
    /// in Bildschirmpunkten wie GUI_HIT_TEST -- aus derselben Rechnung wie
    /// Zeichnen und Klick (Ansicht nach Filter, Rollstand, Zeilenhoehe). Ein
    /// weggefilterter Eintrag liefert (0, 0, 0, 0); ein herausgerollter eine
    /// Lage ausserhalb der Liste. Gebraucht fuer Pruefungen mit echten Klicks
    /// -- die Zeilenhoehe haengt an Schrift und Stildatei.
    pub fn list_item_rect(&self, h: i64, k: i64) -> Result<(i32, i32, i32, i32), String> {
        let w = self.wdg(h, "GUI_LISTBOX_ITEM_RECT")?;
        if !matches!(w.kind, Kind::ListBox) {
            return Err("GUI_LISTBOX_ITEM_RECT: Widget ist keine Liste".into());
        }
        if k < 0 || k as usize >= w.items.len() {
            return Err(format!("GUI_LISTBOX_ITEM_RECT: Eintrag {} ausserhalb [0..{}]", k, w.items.len() as i64 - 1));
        }
        let (wi, _) = Self::dec_widget(h);
        let pos = match liste_ansicht(&w.items, w.list.as_deref()).iter().position(|&x| x == k as usize) {
            Some(p) => p as i32,
            None => return Ok((0, 0, 0, 0)),
        };
        let (ax, ay, aw, _) = self.abs_rect(wi, w);
        let ih = self.list_zeile_h(w);
        Ok((ax, ay + pos * ih - w.value as i32, aw, ih))
    }
    pub fn list_tip(&mut self, h: i64, i: i64, text: String) -> Result<(), String> {
        let p = Self::list_idx(self.list_ref(h, "GUI_LISTBOX_TIP")?, i, "GUI_LISTBOX_TIP")?;
        self.list_mut(h, "GUI_LISTBOX_TIP")?.tips[p] = text; Ok(())
    }
    /// Einen Eintrag sperren: er bleibt sichtbar, laesst sich aber weder
    /// anklicken noch mit der Tastatur erreichen. Ist er gerade gewaehlt, faellt
    /// die Wahl weg -- sonst meldete GUI_LISTBOX_SELECTED etwas, das man nicht
    /// waehlen darf.
    pub fn list_enable(&mut self, h: i64, i: i64, an: bool) -> Result<(), String> {
        let p = Self::list_idx(self.list_ref(h, "GUI_LISTBOX_ENABLE")?, i, "GUI_LISTBOX_ENABLE")?;
        let l = self.list_mut(h, "GUI_LISTBOX_ENABLE")?;
        l.aus[p] = !an;
        if !an { l.sel[p] = false; }
        let w = self.wdg_mut(h, "GUI_LISTBOX_ENABLE")?;
        if !an && w.sel == p as i32 { w.sel = -1; }
        Ok(())
    }
    pub fn list_enabled(&self, h: i64, i: i64) -> Result<bool, String> {
        let w = self.list_ref(h, "GUI_LISTBOX_ENABLED")?;
        let p = Self::list_idx(w, i, "GUI_LISTBOX_ENABLED")?;
        Ok(w.list.as_ref().map(|l| !l.aus.get(p).copied().unwrap_or(false)).unwrap_or(true))
    }
    /// Einen Eintrag zum Gruppenkopf machen: er gliedert die Liste, ist nicht
    /// waehlbar, und Sortieren arbeitet INNERHALB der Gruppen.
    pub fn list_header(&mut self, h: i64, i: i64, an: bool) -> Result<(), String> {
        let p = Self::list_idx(self.list_ref(h, "GUI_LISTBOX_HEADER")?, i, "GUI_LISTBOX_HEADER")?;
        let l = self.list_mut(h, "GUI_LISTBOX_HEADER")?;
        l.kopf[p] = an;
        if an { l.sel[p] = false; }
        let w = self.wdg_mut(h, "GUI_LISTBOX_HEADER")?;
        if an && w.sel == p as i32 { w.sel = -1; }
        Ok(())
    }
    /// Filter: nur Eintraege, deren Text (oder Zusatztext) den Teiltext
    /// enthaelt, ohne Ruecksicht auf Gross/klein. Die Eintraege bleiben, wo
    /// sie sind -- alle Nummern nach aussen sind weiter DATENzeilen.
    pub fn list_filter(&mut self, h: i64, text: String) -> Result<(), String> {
        let l = self.list_mut(h, "GUI_LISTBOX_FILTER")?;
        if l.filter == text { return Ok(()); }
        l.filter = text;
        self.wdg_mut(h, "GUI_LISTBOX_FILTER")?.value = 0.0;
        Ok(())
    }
    pub fn list_get_filter(&self, h: i64) -> Result<String, String> {
        Ok(self.list_ref(h, "GUI_LISTBOX_GET_FILTER")?.list.as_ref().map(|l| l.filter.clone()).unwrap_or_default())
    }
    pub fn list_view_count(&self, h: i64) -> Result<i64, String> {
        let w = self.list_ref(h, "GUI_LISTBOX_VIEW_COUNT")?;
        Ok(liste_ansicht(&w.items, w.list.as_deref()).len() as i64)
    }
    /// Die k-te SICHTBARE Zeile als Datenzeile, -1 wenn es sie nicht gibt.
    pub fn list_view_row(&self, h: i64, k: i64) -> Result<i64, String> {
        let w = self.list_ref(h, "GUI_LISTBOX_VIEW_ROW")?;
        Ok(liste_ansicht(&w.items, w.list.as_deref()).get(k.max(0) as usize).filter(|_| k >= 0).map(|&x| x as i64).unwrap_or(-1))
    }
    pub fn list_placeholder(&mut self, h: i64, text: String) -> Result<(), String> {
        self.list_mut(h, "GUI_LISTBOX_PLACEHOLDER")?.leer_text = text; Ok(())
    }
    /// Sortieren -- natuerlich ("Datei 9" vor "Datei 10"), und INNERHALB der
    /// Gruppen, wenn es Gruppenkoepfe gibt: sonst risse Sortieren die Koepfe
    /// von ihren Eintraegen weg. Alles, was an einem Eintrag haengt, reist mit,
    /// und die Auswahl meint danach denselben Eintrag.
    pub fn list_sort(&mut self, h: i64, absteigend: bool) -> Result<(), String> {
        self.list_mut(h, "GUI_LISTBOX_SORT")?;
        let w = self.wdg_mut(h, "GUI_LISTBOX_SORT")?;
        let n = w.items.len();
        let kopf: Vec<bool> = w.list.as_ref().map(|l| l.kopf.clone()).unwrap_or_else(|| vec![false; n]);
        let mut ord: Vec<usize> = Vec::with_capacity(n);
        let mut k = 0;
        while k < n {
            if kopf[k] { ord.push(k); k += 1; continue; }
            let start = k;
            while k < n && !kopf[k] { k += 1; }
            let mut gruppe: Vec<usize> = (start..k).collect();
            gruppe.sort_by(|&a, &b| {
                let o = natuerlich_vergleichen(&w.items[a], &w.items[b]);
                if absteigend { o.reverse() } else { o }
            });
            ord.extend(gruppe);
        }
        let alt_items = w.items.clone();
        w.items = ord.iter().map(|&k| alt_items[k].clone()).collect();
        if let Some(l) = w.list.as_mut() { l.umordnen(&ord); l.anker = -1; }
        if w.sel >= 0 { w.sel = ord.iter().position(|&k| k as i32 == w.sel).map(|p| p as i32).unwrap_or(-1); }
        Ok(())
    }
    /// Der naechste Eintrag ab `ab` (einschliesslich), dessen Text den
    /// Teiltext enthaelt -- ohne Ruecksicht auf Gross/klein; -1 = keiner.
    pub fn list_find(&self, h: i64, text: &str, ab: i64) -> Result<i64, String> {
        let w = self.list_ref(h, "GUI_LISTBOX_FIND")?;
        let t = text.to_lowercase();
        Ok(w.items.iter().enumerate().skip(ab.max(0) as usize)
            .find(|(_, s)| s.to_lowercase().contains(&t)).map(|(k, _)| k as i64).unwrap_or(-1))
    }
    /// So rollen, dass der Eintrag zu sehen ist (liegt er im Filter verborgen,
    /// passiert nichts).
    pub fn list_scroll_to(&mut self, h: i64, i: i64) -> Result<(), String> {
        let (wi, idx) = Self::dec_widget(h);
        let p = Self::list_idx(self.list_ref(h, "GUI_LISTBOX_SCROLL_TO")?, i, "GUI_LISTBOX_SCROLL_TO")? as i32;
        self.liste_rollen_zu(wi, idx, p);
        Ok(())
    }
    /// Wie weit eine Liste bzw. ein Baum rollen kann (0 = alles passt) --
    /// dieselbe Rechnung wie ihr Rad.
    pub(super) fn list_max_scroll(&self, wi: usize, i: usize) -> i32 {
        let w = &self.windows[wi].widgets[i];
        if w.kind == Kind::Tree {
            let n = w.tree.as_ref().map(|t| Self::tree_visible(t).len()).unwrap_or(0) as i32;
            return (n * self.sk(TREE_ROW_H) - (w.h - 2)).max(0);
        }
        let n = liste_ansicht(&w.items, w.list.as_deref()).len() as i32;
        (n * self.list_zeile_h(w) - w.h).max(0)
    }
    /// Rollstand: die Liste fuehrt ihn in `value`, der Baum in `tree.scroll`.
    pub(super) fn roll_ist(w: &Widget) -> f64 {
        match w.tree.as_ref() {
            Some(t) if w.kind == Kind::Tree => t.scroll as f64,
            // Der Textbereich rollt in ZEILEN (`scroll` = erste sichtbare).
            _ if w.kind == Kind::TextArea => w.scroll as f64,
            _ => w.value,
        }
    }
    pub(super) fn roll_setze(w: &mut Widget, v: f64) {
        if w.kind == Kind::Tree {
            if let Some(t) = w.tree.as_mut() { t.scroll = v.round() as i32; }
        } else if w.kind == Kind::TextArea {
            w.scroll = v.round() as i32;
        } else {
            w.value = v;
        }
    }
    /// Rollbalken einer Liste: (x, y, Breite, Hoehe der Rinne, Griff-y,
    /// Griff-Hoehe) -- nur, wenn mehr Eintraege da sind, als hineinpassen.
    /// EINE Quelle fuer Zeichnen, Klick und Ziehen, wie beim Panel.
    pub(super) fn list_bar_geom(&self, wi: usize, i: usize) -> Option<(i32, i32, i32, i32, i32, i32)> {
        let w = &self.windows[wi].widgets[i];
        if !matches!(w.kind, Kind::ListBox | Kind::Tree) || !self.widget_shown(wi, w) { return None; }
        let max = self.list_max_scroll(wi, i);
        if max <= 0 { return None; }
        let (ax, ay, aw, ah) = self.abs_rect(wi, w);
        let bw = self.sk(8);
        let (tx, ty, th) = (ax + aw - 1 - bw, ay + 1, (ah - 2).max(1));
        let inhalt = max + w.h;
        let thh = ((th as i64 * w.h as i64 / inhalt.max(1) as i64) as i32).clamp(self.sk(16).min(th), th);
        let thy = ty + ((th - thh) as f64 * (Self::roll_ist(w) / max as f64).clamp(0.0, 1.0)) as i32;
        Some((tx, ty, bw, th, thy, thh))
    }
    pub(super) fn list_bar_at(&self, wi: usize, i: usize, mx: i32, my: i32) -> bool {
        self.list_bar_geom(wi, i).map(|(tx, ty, bw, th, _, _)| Self::in_rect(mx, my, (tx, ty, bw, th))).unwrap_or(false)
    }
    /// Rechtsklick auf eine Zeile waehlt sie -- ein Kontextmenue soll sich auf
    /// das beziehen, worauf man zeigt. In einer Mehrfachauswahl bleibt eine
    /// bestehende Auswahl stehen, wenn die Zeile dazugehoert (so wirkt
    /// "Loeschen" im Kontextmenue auf alle gewaehlten).
    pub(super) fn liste_rechtsklick(&mut self, wi: usize, i: usize, k: i32) {
        if k < 0 { return; }
        let w = &mut self.windows[wi].widgets[i];
        if !w.enabled { return; }
        let n = w.items.len();
        if k as usize >= n { return; }
        if w.list.as_ref().map(|l| !l.waehlbar(k as usize)).unwrap_or(false) { return; }
        let multi = w.list.as_ref().map(|l| l.multi).unwrap_or(false);
        let mut geaendert = false;
        if multi {
            let l = w.list.as_mut().unwrap();
            l.sync(n);
            if !l.sel[k as usize] {
                for x in l.sel.iter_mut() { *x = false; }
                l.sel[k as usize] = true;
                l.anker = k;
                geaendert = true;
            }
            if w.sel != k { w.sel = k; }
        } else if w.sel != k {
            w.sel = k;
            geaendert = true;
        }
        if geaendert {
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }
    /// Rechtsklick auf eine Tabellenzeile waehlt sie -- wie bei der Liste; eine
    /// Mehrfachauswahl bleibt, wenn die Zeile dazugehoert.
    pub(super) fn tabelle_rechtsklick(&mut self, wi: usize, i: usize) {
        let w = &mut self.windows[wi].widgets[i];
        if !w.enabled { return; }
        let t = match w.tbl.as_mut() { Some(t) => t, None => return };
        let r = t.hover_row;
        if r < 0 || t.ist_gewaehlt(r) { return; }
        t.auswahl_setzen(r);
        t.cur_r = r;
        if t.zellmodus && t.hover_col >= 0 { t.cur_c = t.hover_col; t.ber_r = r; t.ber_c = t.cur_c; }
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }
    /// Umbenennen beginnen: der ganze Text ist markiert, das erste Tippen
    /// ersetzt ihn -- wie im Dateimanager.
    pub(super) fn liste_edit_start(&mut self, wi: usize, i: usize, k: usize) {
        self.liste_rollen_zu(wi, i, k as i32);
        let w = &mut self.windows[wi].widgets[i];
        if k >= w.items.len() { return; }
        let text: Vec<char> = w.items[k].chars().collect();
        let n = w.items.len();
        let l = w.list.get_or_insert_with(|| Box::new(ListState { anker: -1, ..Default::default() }));
        l.sync(n);
        let len = text.len() as i32;
        l.edit = Some(ListEdit { k, text, caret: len, anker: 0 });
    }
    /// Umbenennen beenden: uebernehmen (Enter, Klick woanders, Fokus weg)
    /// oder verwerfen (ESC). Ein unveraenderter Text ist kein Ereignis.
    pub(super) fn liste_edit_ende(&mut self, wi: usize, i: usize, uebernehmen: bool) {
        let w = &mut self.windows[wi].widgets[i];
        let e = match w.list.as_mut().and_then(|l| l.edit.take()) { Some(e) => e, None => return };
        if !uebernehmen || e.k >= w.items.len() { return; }
        let neu: String = e.text.iter().collect();
        if neu == w.items[e.k] { return; }
        w.items[e.k] = neu;
        if let Some(l) = w.list.as_mut() { l.bearbeitet = Some(e.k); }
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }
    /// Wo das Eingabefeld beim Umbenennen steht (Bildschirm) -- aus der Lage
    /// der Zeile in der ANSICHT, wie Zeichnen und Klick sie rechnen.
    pub(super) fn list_edit_rect(&self, wi: usize, i: usize) -> Option<(i32, i32, i32, i32)> {
        let w = &self.windows[wi].widgets[i];
        let e = w.list.as_ref()?.edit.as_ref()?;
        let pos = liste_ansicht(&w.items, w.list.as_deref()).iter().position(|&x| x == e.k)? as i32;
        let (ax, ay, aw, _) = self.abs_rect(wi, w);
        let ih = self.list_zeile_h(w);
        let bar = if self.list_bar_geom(wi, i).is_some() { self.sk(8) + 2 } else { 0 };
        // Das Feld deckt die Zeile (bei Kaestchen erst dahinter); der Text
        // darin steht, wo der Zeilentext stand (`list_text_x`), damit er
        // beim Oeffnen nicht springt.
        let box_w = self.list_box_w(w);
        let x0 = if box_w > 0 { ax + box_w - self.sk(3) } else { ax + 1 };
        Some((x0, ay + pos * ih - w.value as i32, (ax + aw - 2 - bar - x0).max(8), ih))
    }
    /// Wo der Text einer Listenzeile beginnt: hinter Kaestchen und Sinnbild.
    pub(super) fn list_text_x(&self, w: &Widget, ax: i32) -> i32 {
        let box_w = self.list_box_w(w);
        let ih = self.list_zeile_h(w);
        let hat_icon = w.list.as_ref().map(|l| l.icons.iter().any(|&i| i >= 0)).unwrap_or(false);
        let mut tx = if box_w > 0 { ax + box_w } else { ax + self.w_pad(w) };
        if hat_icon { tx += ih + 2; }
        tx
    }
    /// Rechtsklick auf einen Knoten waehlt ihn -- wie bei Liste und Tabelle;
    /// eine Mehrfachauswahl bleibt, wenn der Knoten dazugehoert.
    pub(super) fn baum_rechtsklick(&mut self, wi: usize, i: usize) {
        let (k, behalten) = match self.windows[wi].widgets[i].tree.as_ref() {
            Some(t) if self.windows[wi].widgets[i].enabled => {
                let k = t.hover;
                (k, k >= 0 && t.multi && t.sel.get(k as usize).copied().unwrap_or(false))
            }
            _ => return,
        };
        if k < 0 || behalten { return; }
        self.tree_setze_auswahl(wi, i, k);
    }
    /// Knoten umbenennen: dieselbe Arbeitskopie wie bei der Liste.
    pub(super) fn baum_edit_start(&mut self, wi: usize, i: usize, k: usize) {
        let t = match self.windows[wi].widgets[i].tree.as_mut() { Some(t) => t, None => return };
        if k >= t.nodes.len() || t.datei.is_some() { return; }
        // Nur ein SICHTBARER Knoten laesst sich bearbeiten -- er ist vorher
        // gewaehlt worden oder wird es jetzt.
        if !Self::tree_visible(t).contains(&k) { return; }
        let text: Vec<char> = t.nodes[k].label.chars().collect();
        let len = text.len() as i32;
        t.edit = Some(ListEdit { k, text, caret: len, anker: 0 });
        t.selected = k as i32;
        self.tree_sichtbar(wi, i);
    }
    pub(super) fn baum_edit_ende(&mut self, wi: usize, i: usize, uebernehmen: bool) {
        let w = &mut self.windows[wi].widgets[i];
        let t = match w.tree.as_mut() { Some(t) => t, None => return };
        let e = match t.edit.take() { Some(e) => e, None => return };
        if !uebernehmen || e.k >= t.nodes.len() { return; }
        let neu: String = e.text.iter().collect();
        if neu == t.nodes[e.k].label { return; }
        t.nodes[e.k].label = neu;
        t.bearbeitet = Some(e.k);
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }
    /// Wo der Name eines Knotens beginnt: hinter Kaestchen, Einzug, Dreieck
    /// und Sinnbild -- dieselbe Rechnung wie das Zeichnen.
    pub(super) fn baum_text_x(&self, t: &TreeState, ax: i32, k: usize) -> i32 {
        let level = t.nodes[k].level;
        let mut x = ax + 4 + self.tree_kast_w(t) + level * self.sk(TREE_INDENT) + self.sk(TREE_TOGGLE_W) + 2;
        if t.nodes.iter().any(|n| n.icon >= 0 || !n.sym.is_empty()) { x += self.sk(TREE_ROW_H) - 4 + 3; }
        x
    }
    pub(super) fn baum_edit_rect(&self, wi: usize, i: usize) -> Option<(i32, i32, i32, i32)> {
        let w = &self.windows[wi].widgets[i];
        let t = w.tree.as_ref()?;
        let e = t.edit.as_ref()?;
        let pos = Self::tree_visible(t).iter().position(|&x| x == e.k)? as i32;
        let (ax, ay, aw, _) = self.abs_rect(wi, w);
        let rh = self.sk(TREE_ROW_H);
        let bar = if self.list_bar_geom(wi, i).is_some() { self.sk(8) + 2 } else { 0 };
        let x0 = self.baum_text_x(t, ax, e.k) - self.sk(3);
        Some((x0, ay + 1 + pos * rh - t.scroll, (ax + aw - 2 - bar - x0).max(8), rh))
    }
    /// Umbenennen-Feld von Liste ODER Baum -- fuer die Stellen, die beides
    /// gleich behandeln (Klick daneben, Fokus weg).
    pub(super) fn edit_rect_any(&self, wi: usize, i: usize) -> Option<(i32, i32, i32, i32)> {
        self.list_edit_rect(wi, i).or_else(|| self.baum_edit_rect(wi, i))
    }
    pub(super) fn edit_ende_any(&mut self, wi: usize, i: usize, uebernehmen: bool) {
        self.liste_edit_ende(wi, i, uebernehmen);
        self.baum_edit_ende(wi, i, uebernehmen);
    }
    pub(super) fn wird_umbenannt(w: &Widget) -> bool {
        w.list.as_ref().map(|l| l.edit.is_some()).unwrap_or(false)
            || w.tree.as_ref().map(|t| t.edit.is_some()).unwrap_or(false)
    }
    /// Tippen springt (Baum): wie bei der Liste, ueber die SICHTBAREN Knoten.
    pub(super) fn baum_tippen(&mut self, wi: usize, i: usize, vis: &[usize], g: &mut Graphics) -> bool {
        let mut s = g.pop_text_input();
        s.retain(|c| !c.is_control() && c != ' ');
        if s.is_empty() && !g.key_ctrl() {
            for c in (b'a'..=b'z').chain(b'0'..=b'9') {
                if g.key_pressed(c as i64) { s.push(c as char); }
            }
        }
        if s.is_empty() || vis.is_empty() { return false; }
        let jetzt = g.get_time();
        let ziel = {
            let t = self.windows[wi].widgets[i].tree.as_mut().unwrap();
            if jetzt - t.tipp_zeit > 1.0 { t.tipp.clear(); }
            t.tipp_zeit = jetzt;
            t.tipp.push_str(&s.to_lowercase());
            let tipp = t.tipp.clone();
            let erster = tipp.chars().next().unwrap();
            let wiederholt = tipp.chars().all(|c| c == erster);
            let such: String = if wiederholt { erster.to_string() } else { tipp };
            let sel = t.selected;
            let pos = vis.iter().position(|&k| k as i32 == sel);
            let start = match pos { Some(p) if wiederholt => p + 1, Some(p) => p, None => 0 };
            (0..vis.len()).map(|d| vis[(start + d) % vis.len()])
                .find(|&k| t.nodes[k].label.to_lowercase().starts_with(&such))
        };
        if let Some(k) = ziel { self.tree_setze_auswahl(wi, i, k as i32); }
        true
    }
    // --- GUI_TREE_EDIT / EDITED / EDITING / PLACEHOLDER ---
    pub fn tree_edit(&mut self, h: i64, node: i64) -> Result<(), String> {
        let f = "GUI_TREE_EDIT";
        let (wi, idx) = Self::dec_widget(h);
        let t = self.tree_ref(h, f)?;
        if t.datei.is_some() { return Err(format!("{}: ein Dateibaum benennt keine Dateien um -- das Programm tut es (RENAME) und der Baum zeigt es", f)); }
        if node < 0 || node as usize >= t.nodes.len() { return Err(format!("{}: Knoten {} gibt es nicht", f, node)); }
        if !Self::tree_visible(t).contains(&(node as usize)) { return Err(format!("{}: Knoten {} ist zugeklappt -- erst GUI_TREE_EXPAND auf seine Eltern", f, node)); }
        self.baum_edit_start(wi, idx, node as usize);
        Ok(())
    }
    pub fn tree_edited(&self, h: i64) -> Result<i64, String> {
        Ok(self.tree_ref(h, "GUI_TREE_EDITED")?.bearbeitet.map(|k| k as i64).unwrap_or(-1))
    }
    pub fn tree_editing(&self, h: i64) -> Result<i64, String> {
        Ok(self.tree_ref(h, "GUI_TREE_EDITING")?.edit.as_ref().map(|e| e.k as i64).unwrap_or(-1))
    }
    pub fn tree_placeholder(&mut self, h: i64, text: String) -> Result<(), String> {
        self.tree_mut(h, "GUI_TREE_PLACEHOLDER")?.leer_text = text;
        Ok(())
    }
    // --- GUI_LISTBOX_EDIT / EDITED / EDITING ---
    pub fn list_edit(&mut self, h: i64, i: i64) -> Result<(), String> {
        let (wi, idx) = Self::dec_widget(h);
        let p = Self::list_idx(self.list_ref(h, "GUI_LISTBOX_EDIT")?, i, "GUI_LISTBOX_EDIT")?;
        self.liste_edit_start(wi, idx, p);
        Ok(())
    }
    pub fn list_edited(&self, h: i64) -> Result<i64, String> {
        let w = self.list_ref(h, "GUI_LISTBOX_EDITED")?;
        Ok(w.list.as_ref().and_then(|l| l.bearbeitet).map(|k| k as i64).unwrap_or(-1))
    }
    pub fn list_editing(&self, h: i64) -> Result<i64, String> {
        let w = self.list_ref(h, "GUI_LISTBOX_EDITING")?;
        Ok(w.list.as_ref().and_then(|l| l.edit.as_ref()).map(|e| e.k as i64).unwrap_or(-1))
    }

    /// Welche Datenzeile liegt auf Hoehe `my`? -1 = keine.
    pub(super) fn liste_zeile(&self, wi: usize, i: usize, my: i32) -> i32 {
        let w = &self.windows[wi].widgets[i];
        let (_, ay, _, _) = self.abs_rect(wi, w);
        let r = (my - ay + w.value as i32).div_euclid(self.list_zeile_h(w));
        let ansicht = liste_ansicht(&w.items, w.list.as_deref());
        if r < 0 || r as usize >= ansicht.len() { return -1; }
        ansicht[r as usize] as i32
    }

    // --- Knopfarten ---
    /// GUI_BUTTON_VARIANT(knopf, art$): standard, primaer, erfolg, warnung,
    /// gefahr, umriss, flach, link. Die Farben kommen aus dem Thema (primaer =
    /// Akzent) bzw. aus einer festen, lesbaren Reihe -- und die TEXTfarbe wird
    /// danach gewaehlt, wie hell der Grund ist.
    pub fn set_button_variant(&mut self, h: i64, art: &str) -> Result<(), String> {
        let a = art.trim().to_lowercase();
        let k = KNOPF_ARTEN.iter().position(|x| *x == a).ok_or_else(|| format!(
            "GUI_BUTTON_VARIANT: Knopfart '{}' gibt es nicht -- bekannt sind: {}", art, KNOPF_ARTEN.join(", ")))?;
        let w = self.wdg_mut(h, "GUI_BUTTON_VARIANT")?;
        if w.kind != Kind::Button { return Err("GUI_BUTTON_VARIANT: Widget ist kein Knopf".into()); }
        w.variante = k as u8; Ok(())
    }
    pub fn button_variant(&self, h: i64) -> Result<String, String> {
        let w = self.wdg(h, "GUI_BUTTON_VARIANT$")?;
        Ok(KNOPF_ARTEN[w.variante.min(KNOPF_ARTEN.len() as u8 - 1) as usize].to_string())
    }
    /// Grundfarbe eines gefuellten Knopfes -- eigene Farbe vor Knopfart; None =
    /// die Farbe des Themas.
    pub(super) fn knopf_grund(&self, w: &Widget) -> Option<i64> {
        if !w.enabled && !w.ov.contains_key("bg") {
            if let Some(c) = self.stil_farbe(w, "bg", stil::AUS) { return Some(c); }
        }
        if let Some(c) = self.eigen(w, "bg") { return Some(c); }
        match w.variante {
            1 => Some(self.wcol(w, "accent", "accent")),
            2 => Some(0x2E9E5B),
            3 => Some(0xD98E1F),
            4 => Some(0xC94A3F),
            _ => None,
        }
    }
    /// Textfarbe eines Knopfes: eigene vor lesbarer vor Thema.
    pub(super) fn knopf_text(&self, w: &Widget) -> i64 {
        if !w.enabled { return self.stil_farbe(w, "fg", stil::AUS).unwrap_or_else(|| self.th("muted_fg")); }
        if let Some(c) = self.eigen(w, "fg") { return self.stil_misch(w, "fg", c); }
        let c = match w.variante {
            5 | 7 => self.wcol(w, "accent", "accent"),
            6 => self.th("text_fg"),
            _ => match self.knopf_grund(w) { Some(bg) => lesbar_auf(bg), None => self.th("text_fg") },
        };
        // Eine Hover-Regel fuer die Schrift greift auch ohne eigene Grundfarbe.
        self.stil_misch(w, "fg", c)
    }
}
