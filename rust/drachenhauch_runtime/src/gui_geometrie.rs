//! Geometrie: die eine Quelle fuer Lage und Groesse, die Zeichnen, Treffertest und Tastatur fragen.
//!
//! Teil von `gui.rs`: dort stehen die Typen, hier die Methoden dieses Themas.

use super::*;

impl Gui {
    // --- Geometrie ---
    pub(super) fn abs_rect(&self, win: usize, w: &Widget) -> (i32, i32, i32, i32) {
        let toff = self.content_top(win);
        let sy = if self.windows[win].scrollable { self.windows[win].scroll_y } else { 0 };
        // Kind eines rollenden Panels: dessen Blick-Versatz kommt dazu.
        let sp = if w.panel_von >= 0 {
            self.windows[win].widgets.get(w.panel_von as usize)
                .and_then(|p| p.panel.as_ref()).map(|p| p.scroll).unwrap_or(0)
        } else { 0 };
        let win = &self.windows[win];
        // Rechts nach links: gespiegelt an der Breite des Fensters -- EINE
        // Stelle, die Zeichnen, Treffer und Tastatur gemeinsam fragen; das
        // Programm sieht weiter seine eigenen Lagen (GUI_GET_X).
        let x = if win.rtl { win.w - w.x - w.w } else { w.x };
        (win.x + x, win.y + toff + w.y - sy - sp, w.w, w.h)
    }
    /// Steckt das Widget in einem rollenden Panel, zaehlt es nur innerhalb
    /// von dessen Flaeche -- was herausgerollt ist, ist weder zu sehen noch
    /// zu treffen.
    pub(super) fn im_panel_sichtbar(&self, win: usize, w: &Widget, mx: i32, my: i32) -> bool {
        // Liegt eine schwebende Leiste darueber, gehoert der Punkt ihr.
        if !self.dock_frei(win, w, mx, my) { return false; }
        if w.panel_von < 0 { return true; }
        match self.windows[win].widgets.get(w.panel_von as usize) {
            Some(p) => Self::in_rect(mx, my, self.abs_rect(win, p)),
            None => true,
        }
    }
    /// Hoehe der Menueleiste dieses Fensters (0, wenn keine Bar-Menues).
    pub(super) fn menubar_h(&self, win: usize) -> i32 {
        if self.windows[win].menus.iter().any(|m| m.in_bar) { self.sk(MENUBAR_H) } else { 0 }
    }
    /// Oberkante des Inhaltsbereichs (Titelleiste + Menueleiste + Reiter-Leiste).
    pub(super) fn content_top(&self, win: usize) -> i32 {
        (if self.windows[win].chrome { self.m("title_h") } else { 0 })
            + self.menubar_h(win) + self.tabbar_h(win)
    }
    /// Hoehe des Titelstreifens eines Panels (0 ohne Titel) -- EINE Quelle
    /// fuer Zeichnen, Beschnitt, Rollbalken und die Lage der Kinder. Bis
    /// 2026.53 stand dort fuenfmal eine rohe 18: bei GUI_SCALE(2) lief der
    /// Titel ueber den Streifen.
    pub(super) fn panel_kopf(&self, w: &Widget) -> i32 {
        if w.text.is_empty() { 0 } else { self.sk(18) }
    }
    /// Hat in diesem Bild schon jemand die Taste genommen, die sonst der
    /// Standard- oder Abbrechen-Knopf bekaeme (eine Liste, eine Klappliste,
    /// ein Kuerzel)? Die eine Frage fuer die Knoepfe -- bis 2026.53 fragten
    /// manche Stellen nur einen der beiden Merker.
    pub(super) fn taste_fuer_knoepfe_genommen(&self) -> bool {
        self.liste_taste || self.kuerzel_gefeuert
    }
    /// Oberkante der Reiter-Leiste eines Fensters (unter Titel und
    /// Menueleiste), absolut. EINE Quelle fuer Zeichnen, Treffer und
    /// Bildschirmleser -- bis 2026.53 stand die Summe fuenfmal da.
    pub(super) fn tabbar_y(&self, win: usize) -> i32 {
        let w = &self.windows[win];
        w.y + (if w.chrome { self.m("title_h") } else { 0 }) + self.menubar_h(win)
    }
    /// Hoehe der Reiter-Leiste (0, wenn keine Tabs).
    pub(super) fn tabbar_h(&self, win: usize) -> i32 {
        if self.windows[win].tabs.is_empty() { 0 } else { self.sk(TABBAR_H) }
    }
    /// Ist das Widget aktuell sichtbar/bedienbar? (Tab-Seite beruecksichtigt)
    pub(super) fn widget_shown(&self, win: usize, w: &Widget) -> bool {
        if !(w.alive && w.visible) { return false; }
        if w.tab_page >= 0 && w.tab_page != self.windows[win].active_tab { return false; }
        // Leiste eines Docks: nur, wenn ihr Inhalt zu sehen ist (vorderer
        // Reiter, Bereich an, nicht ausgeblendet).
        if w.dock_von >= 0 {
            if let Some(d) = self.windows[win].dock.as_ref() {
                if !d.leisten.get(w.dock_von as usize).map(|l| l.sichtbar).unwrap_or(true) { return false; }
            }
        }
        // Stufe 3b: Widgets einer abgerissenen Leiste stehen in ihrer
        // Teilflaeche, nicht im Gastfenster -- und durch die Teilflaeche
        // gesehen gibt es nur sie.
        if let Some(d) = self.windows[win].dock.as_ref() {
            let gruppe = if w.dock_von >= 0 { d.leisten.get(w.dock_von as usize).map(|l| l.schwebt).unwrap_or(-1) } else { -1 };
            match self.leihe.as_ref().filter(|l| l.wi == win) {
                Some(l) => if gruppe != l.k as i32 { return false; },
                None => if gruppe >= 0 && d.teilflaeche(gruppe as usize) { return false; },
            }
        }
        // Reiter IM Fenster: nur die vordere Seite ist da -- fuer das
        // Zeichnen wie fuer jeden Treffertest, die beide hier fragen.
        if w.tc_von >= 0 {
            match self.windows[win].widgets.get(w.tc_von as usize) {
                // Akkordeon: der Abschnitt muss offen sein, und das Kind muss
                // ganz im sichtbaren Teil liegen -- halb herausgerollt waere es
                // halb ueber dem Nachbarn oder dem Rand.
                // Waehrend des Auf- und Zuklappens zaehlt der Teil des
                // Abschnitts, der schon (noch) zu sehen ist: ein Kind
                // erscheint, sobald es ganz hineinpasst.
                Some(tc) if tc.kind == Kind::Accordion => {
                    let s = w.tc_seite.max(0) as usize;
                    let unten = match tc.akk.as_ref().and_then(|a| a.koepfe.get(s).map(|&(ky, inh)| (ky, inh, a.scroll))) {
                        Some((ky, inh, sc)) if inh > 0 => tc.y + ky + self.sk(AKK_KOPF_H) + inh - sc,
                        _ => return false,
                    };
                    return tc.alive && tc.visible && w.y >= tc.y && w.y + w.h <= (tc.y + tc.h).min(unten);
                }
                Some(tc) => return tc.sel == w.tc_seite && tc.alive && tc.visible,
                None => return true,
            }
        }
        true
    }
    /// Layout der Reiter: (page_idx, x_links_abs, x_rechts_abs).
    pub(super) fn tab_slots(&self, g: &Graphics, wi: usize) -> Vec<(usize, i32, i32)> {
        let win = &self.windows[wi];
        let pad = self.m("pad");
        let kreuz = self.tab_kreuz_w(wi);
        let mut x = win.x + pad;
        let mut out = Vec::new();
        for (ti, label) in win.tabs.iter().enumerate() {
            let wl = self.ctext_width(g, label) + pad * 3 + kreuz;
            out.push((ti, x, x + wl));
            x += wl + 2;
        }
        out
    }

    /// Wie viel Platz das Kreuz am Reiter braucht (0, wenn es keins gibt).
    pub(super) fn tab_kreuz_w(&self, wi: usize) -> i32 {
        if self.windows[wi].tabs_closable { self.sk(18) } else { 0 }
    }

    /// Das Rechteck des Kreuzes eines Reiters: (x, y, kante).
    ///
    /// EINE Quelle fuer Zeichnen und Treffertest -- liefen sie auseinander,
    /// schloesse ein Klick neben dem Kreuz einen Reiter (oder gar keiner).
    /// Skaliert wie die Reiterleiste selbst (bis 2026.53 beide unskaliert --
    /// bei GUI_SCALE(2) lief die Schrift der Reiter ueber ihren Streifen).
    pub(super) fn tab_kreuz_rect(&self, wi: usize, x1: i32, by: i32) -> (i32, i32, i32) {
        let k = self.sk(9);
        let w = self.tab_kreuz_w(wi);
        (x1 - w + (w - k) / 2, by + (self.sk(TABBAR_H) - k) / 2, k)
    }
    /// Inhaltshoehe = unterster Widget-Rand (+Rand). Basis fuer den Scrollbereich.
    pub(super) fn content_height(&self, win: usize) -> i32 {
        let pad = self.m("pad");
        self.windows[win].widgets.iter()
            .filter(|w| self.widget_shown(win, w) && w.kind != Kind::Canvas && w.panel_von < 0)
            .map(|w| w.y + w.h).max().unwrap_or(0) + pad * 2
    }
    /// Sichtbare Inhaltshoehe (Fenster minus Titel/Menue/Rand).
    pub(super) fn view_height(&self, win: usize) -> i32 {
        (self.windows[win].h - self.content_top(win) - 2).max(1)
    }
    /// Maximaler Scroll-Offset (0, wenn Inhalt passt).
    pub(super) fn max_scroll_y(&self, win: usize) -> i32 {
        if !self.windows[win].scrollable { return 0; }
        (self.content_height(win) - self.view_height(win)).max(0)
    }
    /// Reiter (Tabs) eines Fensters setzen (Labels). Aktiver Reiter -> 0.
    pub fn set_tabs(&mut self, win: i64, labels: Vec<String>) -> Result<(), String> {
        let w = self.windows.get_mut(win as usize).ok_or("GUI_TABS: erwartet GUI_WINDOW")?;
        w.tabs = labels;
        w.active_tab = 0;
        Ok(())
    }
    /// Widget einem Reiter zuordnen (-1 = auf allen Reitern sichtbar).
    pub fn set_widget_tab(&mut self, h: i64, page: i32) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_TAB")?.tab_page = page;
        Ok(())
    }
    /// Ist das Fenster gerade sichtbar (GUI_WINDOW_SHOWN)?
    ///
    /// Das Gegenstueck zu `GUI_WINDOW_VISIBLE`, das nur setzt. Ohne den
    /// Getter muss ein Programm sich merken, was es selbst gesetzt hat --
    /// und liegt daneben, sobald der Nutzer das Fenster ueber sein Kreuz
    /// schliesst. Ein zerstoertes Fenster ist nicht sichtbar.
    pub fn window_shown(&self, win: i64) -> Result<bool, String> {
        self.windows.get(win as usize).map(|w| w.alive && w.visible)
            .ok_or("GUI_WINDOW_SHOWN: erwartet GUI_WINDOW".into())
    }
    pub fn active_tab(&self, win: i64) -> Result<i64, String> {
        self.windows.get(win as usize).map(|w| w.active_tab as i64).ok_or("GUI_ACTIVE_TAB: erwartet GUI_WINDOW".into())
    }
    /// Reiter mit Kreuz (GUI_TABS_CLOSABLE).
    ///
    /// Die Laufzeit schliesst NICHTS -- sie meldet nur, welches Kreuz
    /// getroffen wurde. Was ein geschlossener Reiter bedeutet, weiss nur das
    /// Programm: an ihm haengen seine Widgets, und vielleicht will es vorher
    /// fragen, ob Ungesichertes verloren gehen darf.
    pub fn tabs_closable(&mut self, win: i64, an: bool) -> Result<(), String> {
        let w = self.windows.get_mut(win as usize).ok_or("GUI_TABS_CLOSABLE: erwartet GUI_WINDOW")?;
        w.tabs_closable = an;
        Ok(())
    }

    /// Welcher Reiter in diesem Bild geschlossen werden soll (-1 = keiner).
    ///
    /// Transient wie `GUI_CLICKED`: ein Klick ist ein Ereignis. Getroffen
    /// wird ueber das Kreuz oder die mittlere Maustaste.
    /// GUI_CONTEXT_TABS(menu): das Kontextmenue gehoert der Reiterleiste
    /// seines Fensters und oeffnet beim Rechtsklick auf einen Reiter.
    pub fn context_tabs(&mut self, menu: i64) -> Result<(), String> {
        let m = self.menu_mut(menu, "GUI_CONTEXT_TABS")?;
        if m.in_bar || m.unter { return Err("GUI_CONTEXT_TABS: erwartet ein Menue aus GUI_CONTEXT".into()); }
        m.ziel = -2;
        Ok(())
    }
    /// GUI_TAB_CONTEXT(win): auf welchem Reiter das Kontextmenue der
    /// Reiterleiste zuletzt aufging (-1 = noch nie).
    pub fn tab_context(&self, win: i64) -> Result<i64, String> {
        self.windows.get(win as usize).map(|w| w.tab_kontext as i64)
            .ok_or("GUI_TAB_CONTEXT: erwartet GUI_WINDOW".into())
    }
    pub fn tab_closed(&self, win: i64) -> Result<i64, String> {
        self.windows.get(win as usize).map(|w| w.tab_zu as i64)
            .ok_or("GUI_TAB_CLOSED: erwartet GUI_WINDOW".into())
    }

    pub fn set_active_tab(&mut self, win: i64, i: i32) -> Result<(), String> {
        let w = self.windows.get_mut(win as usize).ok_or("GUI_SET_ACTIVE_TAB: erwartet GUI_WINDOW")?;
        if i >= 0 && (i as usize) < w.tabs.len() { w.active_tab = i; }
        Ok(())
    }
    /// Inhalt eines Fensters scrollbar machen (Mausrad + Scrollbalken).
    pub fn window_scrollable(&mut self, win: i64, flag: bool) -> Result<(), String> {
        let w = self.windows.get_mut(win as usize).ok_or("GUI_WINDOW_SCROLLABLE: erwartet GUI_WINDOW")?;
        w.scrollable = flag;
        if !flag { w.scroll_y = 0; }
        Ok(())
    }
    /// Geometrie des Inhalts-Scrollbalkens: (track_x, track_y, breite, track_h,
    /// thumb_y, thumb_h) oder None, wenn nicht noetig/aus.
    pub(super) fn winscroll_geom(&self, wi: usize) -> Option<(i32, i32, i32, i32, i32, i32)> {
        let content = self.content_height(wi);
        let view = self.view_height(wi);
        if !self.windows[wi].scrollable || content <= view || view <= 2 { return None; }
        let w = &self.windows[wi];
        // Rechts im Fenster, gespiegelt links -- wie jeder Rollbalken
        // (`rollbalken_lage`).
        Some(self.rollbalken_lage((w.x, w.y + self.content_top(wi), w.w - 1, view), w.rtl, view, content, w.scroll_y as f64))
    }
    pub(super) fn in_rect(mx: i32, my: i32, r: (i32, i32, i32, i32)) -> bool {
        mx >= r.0 && mx < r.0 + r.2 && my >= r.1 && my < r.1 + r.3
    }
    pub(super) fn topmost_at(&self, mx: i32, my: i32) -> Option<usize> {
        for &wi in self.z_order.iter().rev() {
            let w = &self.windows[wi];
            // Ein angedocktes Fenster unter einer schwebenden Leiste seines
            // Gastfensters: der Punkt gehoert der Leiste.
            // Nur Fenster der Oberflaeche, durch die die gui gerade schaut
            // (Weg E): ein Punkt im Hauptfenster trifft kein natives.
            if self.flaeche_von(wi) != self.flaeche { continue; }
            if w.alive && w.visible && Self::in_rect(mx, my, self.sicht_rect(wi))
                && !self.dock_gast_verdeckt(wi, mx, my) { return Some(wi); }
        }
        None
    }
    /// Die Hintergrundfenster ganz nach hinten, in ihrer Reihenfolge.
    pub(super) fn hinten_ordnen(&mut self) {
        let (mut hinten, vorn): (Vec<usize>, Vec<usize>) = self.z_order.iter()
            .partition(|&&i| self.windows.get(i).map(|w| w.hinten).unwrap_or(false));
        hinten.extend(vorn);
        self.z_order = hinten;
    }
    pub(super) fn bring_to_front(&mut self, wi: usize) {
        // Ein angedocktes Fenster bringt sein Gastfenster nach vorn -- es
        // selbst steht immer direkt darueber (dock_ordnen).
        let h = self.windows[wi].dock_host;
        let wi = if h >= 0 && self.windows.get(h as usize).map(|w| w.alive).unwrap_or(false) { h as usize } else { wi };
        // Ein Hintergrundfenster bleibt hinten (Fokus bekommt es trotzdem).
        if self.windows[wi].hinten { return; }
        if self.z_order.last() == Some(&wi) && !self.windows.iter().any(|w| w.dock_host == wi as i32) { return; }
        self.z_order.retain(|&i| i != wi);
        self.z_order.push(wi);
        self.dock_ordnen();
        self.pops_nach_vorn();
    }

}
