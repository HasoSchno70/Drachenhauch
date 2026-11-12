//! Geometrie: die eine Quelle fuer Lage und Groesse, die Zeichnen, Treffertest und Tastatur fragen.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

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
        (win.x + w.x, win.y + toff + w.y - sy - sp, w.w, w.h)
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
    /// Hoehe der Reiter-Leiste (0, wenn keine Tabs).
    pub(super) fn tabbar_h(&self, win: usize) -> i32 {
        if self.windows[win].tabs.is_empty() { 0 } else { TABBAR_H }
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
        if self.windows[wi].tabs_closable { 18 } else { 0 }
    }

    /// Das Rechteck des Kreuzes eines Reiters: (x, y, kante).
    ///
    /// EINE Quelle fuer Zeichnen und Treffertest -- liefen sie auseinander,
    /// schloesse ein Klick neben dem Kreuz einen Reiter (oder gar keiner).
    /// Unskaliert wie die Reiterleiste selbst (TABBAR_H).
    pub(super) fn tab_kreuz_rect(&self, wi: usize, x1: i32, by: i32) -> (i32, i32, i32) {
        let k = 9;
        let w = self.tab_kreuz_w(wi);
        (x1 - w + (w - k) / 2, by + (TABBAR_H - k) / 2, k)
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
        if !self.windows[wi].scrollable || content <= view { return None; }
        let w = &self.windows[wi];
        let bw = 10;
        let tx = w.x + w.w - bw - 2;
        let ty = w.y + self.content_top(wi) + 1;
        // Review-Fund: `th` (Scrollbar-Spurlaenge) hat keine Untergrenze --
        // ein kleines/schmales scrollbares Fenster (z.B. `GUI_WINDOW_SET_
        // BOUNDS win, x, y, w, 40`) liess `th < 24` werden, und
        // `.clamp(24, th)` paniked hart ("assertion failed: min <= max"),
        // reproduzierbar in JEDEM GUI_DRAW eines solchen Fensters. `lo`
        // haelt sich selbst unter 24, wenn die Spur kuerzer ist.
        let th = (view - 2).max(0);
        let max_s = content - view;
        let lo = th.min(24);
        let thh = if th > 0 { ((th * view) / content.max(1)).clamp(lo, th) } else { 0 };
        let thy = ty + if max_s > 0 { (w.scroll_y * (th - thh)) / max_s } else { 0 };
        Some((tx, ty, bw, th, thy, thh))
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
            if w.nativ != self.flaeche { continue; }
            if w.alive && w.visible && Self::in_rect(mx, my, self.sicht_rect(wi))
                && !self.dock_gast_verdeckt(wi, mx, my) { return Some(wi); }
        }
        None
    }
    pub(super) fn bring_to_front(&mut self, wi: usize) {
        // Ein angedocktes Fenster bringt sein Gastfenster nach vorn -- es
        // selbst steht immer direkt darueber (dock_ordnen).
        let h = self.windows[wi].dock_host;
        let wi = if h >= 0 && self.windows.get(h as usize).map(|w| w.alive).unwrap_or(false) { h as usize } else { wi };
        if self.z_order.last() == Some(&wi) && !self.windows.iter().any(|w| w.dock_host == wi as i32) { return; }
        self.z_order.retain(|&i| i != wi);
        self.z_order.push(wi);
        self.dock_ordnen();
        self.pops_nach_vorn();
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
    pub(super) fn caret_blink_on(&self) -> bool {
        (self.frame_count / self.m("caret_period").max(1) as i64) % 2 == 0
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
        if grad > 0 {
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
        let dt = g.delta();
        // Lichtstreifen der Fenster weiterschieben.
        for win in self.windows.iter_mut() { win.glanz.weiter(dt as f32); }
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
}
