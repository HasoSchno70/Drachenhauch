//! Zeichnen: Fenster, Leisten, Popups und jedes Widget.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Draw ---
    /// Alle Fenster zeichnen. `obenauf` = FALSE laesst die Schicht weg, die
    /// ueber ALLEN Fenstern liegt (offenes Kontextmenue, Tooltip) -- fuer
    /// Programme, die danach noch selbst malen und sie mit `draw_top` wieder
    /// aufsetzen.
    ///
    /// Warum nicht einfach zweimal zeichnen: Tooltip und Kontextmenue haben
    /// einen halbdurchsichtigen Schlagschatten. Zweimal uebereinander ist er
    /// dunkler -- und zwar nur DORT, wo das eigene Zeichnen die erste Fassung
    /// nicht zugedeckt hat. Genau an einer Kante also, an der ein Tooltip oft
    /// liegt, weil er der Maus folgt.
    pub fn draw(&self, g: &mut Graphics, obenauf: bool) {
        for (k, &wi) in self.z_order.iter().enumerate() {
            if self.modal == Some(wi) { self.schleier(g); }
            // Ein Popover kommt in der oberen Schicht (draw_top).
            if self.windows[wi].alive && self.windows[wi].visible && self.windows[wi].pop.is_none() { self.draw_window_sicht(g, wi); }
            // Schwebende Leisten eines Gastfensters liegen ueber seinen
            // angedockten Fenstern: gezeichnet nach dem letzten davon.
            if let Some(h) = self.dock_angedockter_gast(wi) {
                let naechster = self.z_order.get(k + 1).and_then(|&n| self.dock_angedockter_gast(n));
                if naechster != Some(h) { self.dock_oben_nachholen(g, h); }
            }
        }
        if obenauf { self.draw_top(g); }
    }

    /// Layout der Menueleiste: (menu_idx, x_links_abs, x_rechts_abs) je Bar-Menue.
    pub(super) fn menubar_slots(&self, g: &Graphics, wi: usize) -> Vec<(usize, i32, i32)> {
        let win = &self.windows[wi];
        let pad = self.m("pad");
        let mut x = win.x + pad;
        let mut out = Vec::new();
        for (mi, m) in win.menus.iter().enumerate() {
            if !m.in_bar { continue; }
            let wlbl = self.ctext_width(g, &m.label) + pad * 2;
            out.push((mi, x, x + wlbl));
            x += wlbl;
        }
        out
    }

    /// Zeichnet ein Menue-Popup (Dropdown/Kontext/Untermenue) mit Items ab
    /// (px, py): links Haekchen oder Sinnbild, dann der Text, rechts das
    /// Kuerzel gedaempft, ganz rechts ein Pfeil bei einem Untermenue.
    pub(super) fn draw_items_popup(&self, g: &mut Graphics, wi: usize, mi: usize, px: i32, py: i32) {
        let m = match self.windows.get(wi).and_then(|w| w.menus.get(mi)) { Some(m) => m, None => return };
        let pad = self.m("pad");
        let (wmax, links, _wk) = self.popup_layout(g, wi, mi);
        let ih = self.sk(MENU_ITEM_H);
        let h_voll = m.items.len() as i32 * ih + 4;
        // Aufrollen wie die Klappliste: von oben herab, dabei einblenden.
        // Die Eintraege stehen schon an ihrer Stelle, zu sehen ist der obere
        // Teil. Ohne Uebergang (0) steht es sofort ganz da.
        let t = if self.m("uebergang") > 0 { weich(m.auf_t) } else { 1.0 };
        let h = ((h_voll as f64 * t).round() as i32).max(1);
        let rad = self.m("corner_radius").min(6);
        g.push_clip(px, py, wmax + 8, h + 8);
        // Schatten + Hintergrund + Rahmen.
        let schatten = ((0x44 as f64) * t).round().max(1.0) as i64;
        g.round_rect(px + 3, py + 4, px + wmax - 1 + 3, py + h - 1 + 4, rad.max(2), schatten << 24, true);
        g.round_rect(px, py, px + wmax - 1, py + h - 1, rad, deckkraft(self.th("widget_bg"), 0.35 + 0.65 * t), true);
        g.round_rect(px, py, px + wmax - 1, py + h - 1, rad, self.th("win_border"), false);
        g.push_clip(px, py, wmax, h);
        // Der Eintrag, dessen Untermenue offen ist, bleibt hervorgehoben --
        // sonst verliert man beim Hinueberfahren den Faden.
        let offen_sub: Option<i32> = self.sub_chain.iter()
            .find(|(smi, _, _)| m.items.iter().any(|it| it.sub == *smi as i32))
            .map(|(smi, _, _)| *smi as i32);
        // Der Tastatur-Cursor gilt nur im TIEFSTEN offenen Leistenmenue.
        let cursor_hier = self.menu_cursor.filter(|_| self.tiefstes_popup() == Some((wi, mi)));
        for (ii, it) in m.items.iter().enumerate() {
            let iy = py + 2 + ii as i32 * ih;
            if it.separator {
                g.line(px + 6, iy + ih / 2, px + wmax - 7, iy + ih / 2, self.th("win_border"));
                continue;
            }
            // Die Maus blendet weich; Tastatur-Cursor und offenes
            // Untermenue stehen sofort ganz da.
            let per_taste = cursor_hier == Some(ii);
            let fest = per_taste || (it.sub >= 0 && offen_sub == Some(it.sub));
            let a = if fest { 1.0 } else if it.enabled { m.b.anteil(ii as i32) } else { 0.0 };
            if a > 0.0 { g.box_fill(px + 2, iy, px + wmax - 3, iy + ih - 1, deckkraft(self.th("title_bg_focus"), a)); }
            let fg = if it.enabled { self.th("text_fg") } else { self.th("muted_fg") };
            let th14 = self.ctext_height(g);
            let ty = iy + (ih - th14) / 2;
            let tx = px + pad + 4 + links;
            if !it.sym.is_empty() {
                let isz = (ih - 8).min(self.sk(16));
                let c = if it.enabled { mischen(self.th("text_fg"), self.th("muted_fg"), 0.25) } else { self.th("muted_fg") };
                sinnbild(g, &it.sym, px + pad + 2, iy + (ih - isz) / 2, isz, c, !it.enabled);
            } else if it.icon >= 0 {
                let isz = (ih - 6).min(self.sk(18));
                g.draw_image_rect(it.icon, px + pad + 2, iy + (ih - isz) / 2, isz, isz);
            } else if it.haken && it.an {
                // Haekchen aus zwei Strichen -- eine Glyphe haette nicht jede Schrift.
                let cx = px + pad + 5;
                let cy = iy + ih / 2;
                g.line(cx, cy, cx + 3, cy + 3, fg);
                g.line(cx + 1, cy, cx + 4, cy + 3, fg);
                g.line(cx + 3, cy + 3, cx + 9, cy - 4, fg);
                g.line(cx + 4, cy + 3, cx + 10, cy - 4, fg);
            }
            self.ctext(g, tx, ty, it.label.clone(), fg);
            if !it.kuerzel.is_empty() {
                let kw = self.ctext_width(g, &it.kuerzel);
                let kx = px + wmax - pad - 4 - kw - (if it.sub >= 0 { self.sk(16) } else { 0 });
                self.ctext(g, kx, ty, it.kuerzel.clone(), self.th("muted_fg"));
            }
            if it.sub >= 0 {
                let ax = px + wmax - pad - 8;
                let ay = iy + ih / 2;
                for d in 0..4 { g.line(ax + d, ay - 4 + d, ax + d, ay + 4 - d, fg); }
            }
        }
        g.pop_clip();
        g.pop_clip();
    }

    pub(super) fn draw_window(&self, g: &mut Graphics, wi: usize) {
        let win = &self.windows[wi];
        let focused = self.focus_window == Some(wi);
        let (x, y, w, h) = (win.x, win.y, win.w, win.h);
        let th = self.m("title_h");
        let pad = self.m("pad");
        let rad = self.m("corner_radius");
        let shadow = self.m("shadow");
        // Fenster-Schatten (modern): weiche, halbtransparente Lagen nach unten/rechts
        // versetzt -> das Fenster "schwebt" ueberm Hintergrund statt aufgeklebt.
        if shadow > 0 {
            for k in 0..3 {
                let off = 3 + k * 3;
                let grow = k * 2;
                let alpha = (0x3A - k * 0x14).max(0x10) as i64;
                let scol = (alpha << 24) as i64;       // schwarz mit Alpha (ARGB)
                g.round_rect(x - grow + off, y + off + 2,
                             x + w - 1 + grow + off, y + h - 1 + grow + off,
                             (rad + grow).max(0), scol, true);
            }
        }
        // Fenster-Hintergrund (rund, wenn corner_radius > 0).
        if rad > 0 { g.round_rect(x, y, x + w - 1, y + h - 1, rad, self.th("win_bg"), true); }
        else { g.box_fill(x, y, x + w - 1, y + h - 1, self.th("win_bg")); }
        let toff = if win.chrome { th } else { 0 };   // Inhalts-Oberkante
        if win.chrome {
            let title_bg = if focused { self.th("title_bg_focus") } else { self.th("title_bg") };
            if rad > 0 {
                // Titelleiste: oben gerundet (folgt der Fensterform), unten gerade.
                g.round_rect(x, y, x + w - 1, y + th - 1, rad, title_bg, true);
                g.box_fill(x, y + th - 1 - rad, x + w - 1, y + th - 1, title_bg);
                g.line(x + 1, y + th, x + w - 2, y + th, self.th("win_border"));  // Trennlinie
                g.round_rect(x, y, x + w - 1, y + h - 1, rad, self.th("win_border"), false);
            } else {
                g.box_fill(x, y, x + w - 1, y + th - 1, title_bg);
                g.rect(x, y, x + w - 1, y + h - 1, self.th("win_border"));
            }
            let th14 = self.ctext_height(g);
            self.ctext(g, x + pad, y + (th - th14) / 2, win.title.clone(), self.th("title_fg"));
            if win.closable {
                let (cx, cy, cw, ch) = (x + w - th, y, th, th);
                let m = (th / 3).clamp(5, 9);          // Rand des X (skaliert mit der Titelhoehe)
                g.line(cx + m, cy + m, cx + cw - m - 1, cy + ch - m - 1, self.th("title_fg"));
                g.line(cx + cw - m - 1, cy + m, cx + m, cy + ch - m - 1, self.th("title_fg"));
            }
            if win.resizable {                   // Resize-Griff unten-rechts (Diagonalen)
                let c = self.th("win_border");
                for o in [2, 6, 10] {
                    g.line(x + w - 2 - o, y + h - 2, x + w - 2, y + h - 2 - o, c);
                }
            }
        }
        // Menueleiste (unter der Titelleiste), falls Bar-Menues vorhanden.
        let mboff = self.menubar_h(wi);
        if mboff > 0 {
            let by = y + toff;
            g.box_fill(x + 1, by, x + w - 2, by + mboff - 1, self.th("title_bg"));
            g.line(x + 1, by + mboff, x + w - 2, by + mboff, self.th("win_border"));
            for (mi, x0, x1) in self.menubar_slots(g, wi) {
                if self.open_menu == Some((wi, mi)) {
                    g.box_fill(x0, by, x1 - 1, by + mboff - 1, self.th("title_bg_focus"));
                }
                let th14 = self.ctext_height(g);
                self.ctext(g, x0 + pad, by + (mboff - th14) / 2, win.menus[mi].label.clone(), self.th("title_fg"));
            }
        }
        // Reiter-Leiste (unter Titel/Menue), falls Tabs vorhanden.
        let tboff = self.tabbar_h(wi);
        if tboff > 0 {
            let by = y + toff + mboff;
            g.box_fill(x + 1, by, x + w - 2, by + tboff - 1, self.th("win_bg"));
            g.line(x + 1, by + tboff - 1, x + w - 2, by + tboff - 1, self.th("win_border"));
            for (ti, x0, x1) in self.tab_slots(g, wi) {
                let active = win.active_tab == ti as i32;
                if active {
                    g.box_fill(x0, by + 3, x1 - 1, by + tboff - 1, self.th("widget_bg"));
                    g.box_fill(x0, by + tboff - 2, x1 - 1, by + tboff - 1, self.th("accent"));  // Akzent-Unterstrich
                }
                let fg = if active { self.th("text_fg") } else { self.th("muted_fg") };
                let th14 = self.ctext_height(g);
                self.ctext(g, x0 + pad, by + (tboff - th14) / 2, win.tabs[ti].clone(), fg);
                if win.tabs_closable {
                    // Zwei Striche statt eines Zeichens: ein `x` aus der
                    // Schrift sitzt je nach Zeichensatz anders und ist bei
                    // 9 Pixeln kaum als Kreuz zu erkennen.
                    let (kx, ky, k) = self.tab_kreuz_rect(wi, x1, by);
                    let kf = if active { fg } else { self.th("muted_fg") };
                    g.line(kx, ky, kx + k - 1, ky + k - 1, kf);
                    g.line(kx + k - 1, ky, kx, ky + k - 1, kf);
                }
            }
        }
        let coff = toff + mboff + tboff;   // Inhalts-Oberkante inkl. Menue + Reiter
        // Widgets auf den Fenster-Innenbereich begrenzen: wird das Fenster kleiner
        // gezogen, ragt nichts ueber den Rand/die Titelleiste/Menueleiste hinaus.
        g.push_clip(x + 1, y + coff, (w - 2).max(0), (h - coff - 1).max(0));
        self.dock_zeichnen_unten(g, wi);
        for (i, wdg) in win.widgets.iter().enumerate() {
            // Widgets einer schwebenden Leiste kommen spaeter, obenauf.
            if wdg.dock_von >= 0 && win.dock.as_ref().and_then(|d| d.leisten.get(wdg.dock_von as usize)).map(|l| l.schwebt >= 0).unwrap_or(false) { continue; }
            // Kind eines Akkordeons, dessen Abschnitt gerade auf- oder
            // zugeht: auf den sichtbaren Teil beschnitten gezeichnet, damit
            // es hineingleitet statt aufzutauchen. Bedienbar (widget_shown)
            // ist es erst, wenn es ganz zu sehen ist.
            if let Some((cx, cy, cw, ch)) = self.akk_ausschnitt(wi, wdg) {
                if ch > 0 && (wdg.tab_page < 0 || wdg.tab_page == win.active_tab) {
                    g.push_clip(cx, cy, cw, ch);
                    self.draw_widget(g, wi, i, wdg);
                    g.pop_clip();
                }
                continue;
            }
            if !self.widget_shown(wi, wdg) { continue; }
            if wdg.panel_von >= 0 {
                // Kind eines rollenden Panels: auf dessen Innenflaeche beschnitten.
                let p = &win.widgets[wdg.panel_von as usize];
                let (px, py, pw, ph) = self.abs_rect(wi, p);
                let kopf = if p.text.is_empty() { 0 } else { 18 };
                g.push_clip(px + 1, py + kopf + 1, (pw - 2).max(0), (ph - kopf - 2).max(0));
                self.draw_widget(g, wi, i, wdg);
                g.pop_clip();
            } else {
                self.draw_widget(g, wi, i, wdg);
            }
        }
        // Mit angedockten Fenstern kommt die obere Schicht des Docks erst
        // nach ihnen (`draw`), sonst laege eine schwebende Leiste darunter.
        if !self.dock_hat_angedockte(wi) { self.dock_zeichnen_oben(g, wi); }
        g.pop_clip();
        // Inhalts-Scrollbalken (rechts), wenn der Inhalt hoeher als das Fenster ist.
        if let Some((tx, ty, bw, th, thy, thh)) = self.winscroll_geom(wi) {
            g.box_fill(tx, ty, tx + bw - 1, ty + th - 1, shade(self.th("win_bg"), -10));
            g.round_rect(tx + 1, thy, tx + bw - 2, thy + thh - 1, 4, self.th("widget_border"), true);
        }
        // Der Lichtstreif liegt ueber den Widgets, aber unter aufgeklappten
        // Menues und Klapplisten -- die gehoeren nicht zum Fenster.
        self.draw_glanz(g, wi);
        // Aufgeklapptes Leistenmenue und offene Klappliste liegen ueber ALLEN
        // Fenstern (`draw_top`) -- der Klick geht ja auch an sie, egal was
        // darueber liegt. Hier nur bei einem vergroesserten oder
        // zugeschnittenen Fenster, dessen Lage sie teilen.
        let _ = (y, toff, mboff);
        if !self.popups_oben(wi) { self.popups_zeichnen(g, wi); }
    }

    /// Das aufgeklappte Leistenmenue und die offene Klappliste von `wi`.
    pub(super) fn popups_zeichnen(&self, g: &mut Graphics, wi: usize) {
        let win = &self.windows[wi];
        if !(win.alive && win.visible) { return; }
        if let Some((mw, _)) = self.open_menu {
            if mw == wi && self.context_open.is_none() {
                for (pw, pm, px, py) in self.popup_chain(g) { self.draw_items_popup(g, pw, pm, px, py); }
            }
        }
        if let Some((dw, di)) = self.open_dropdown {
            if dw == wi && win.widgets.get(di).map(|x| x.alive && x.visible).unwrap_or(false) {
                self.draw_dropdown_popup(g, wi, di);
            }
        }
    }

    /// Der Lichtstreif eines Fensters (siehe `Glanz`).
    pub(super) fn draw_glanz(&self, g: &mut Graphics, wi: usize) {
        let win = &self.windows[wi];
        let gl = &win.glanz;
        if gl.t < 0.0 || gl.t > gl.dauer || gl.dauer <= 0.0 { return; }
        let (x, y, w, h) = (win.x, win.y, win.w, win.h);
        if w < 4 || h < 4 { return; }
        let p = (gl.t / gl.dauer).clamp(0.0, 1.0);
        // Weich anfahren und auslaufen, und am Anfang und Ende ausblenden --
        // sonst ploppt der Streif am Rand auf.
        let e = p * p * (3.0 - 2.0 * p);
        let huelle = ((p * std::f32::consts::PI).sin() * 1.6).min(1.0);
        let staerke = gl.staerke * huelle;
        if staerke <= 0.004 { return; }
        let farbe = (if gl.farbe < 0 { self.th("accent") } else { gl.farbe }) & 0xFFFFFF;
        let heiss = mischen(farbe, 0xFFFFFF, 0.55) & 0xFFFFFF;
        let senkrecht = gl.richtung >= 2;
        let laenge = (if senkrecht { h } else { w }) as f32;
        let bw = (laenge * gl.breite).max(8.0);
        // Mitte des Streifs: von ganz ausserhalb bis ganz ausserhalb.
        let mut c = -bw + e * (laenge + 2.0 * bw);
        if gl.richtung == 1 || gl.richtung == 3 { c = laenge - c; }
        let alpha = |f: f32| -> i64 { ((f * 255.0).round() as i64).clamp(1, 255) };
        if gl.art == 2 {
            self.draw_streiflicht(g, wi, e, staerke, farbe, heiss);
            return;
        }
        if gl.art == 1 {
            // Die Lampe: erst das Fenster ein wenig abdunkeln (multipliziert,
            // Farben bleiben, nur dunkler), dann den Lichtkegel additiv darueber
            // -- ein Hof in der Farbe, der Kegel warm-weiss, ein heller Kern.
            // Der Kegel zieht im flachen Bogen, wie ein geschwenkter Arm ihn
            // fuehrt; Anfang und Ende liegen ganz ausserhalb.
            let kurz = w.min(h) as f32;
            let ry = (kurz * gl.breite * 1.5).max(12.0);
            let rx = ry * 1.25;
            let (lang, quer) = if senkrecht { (h as f32, w as f32) } else { (w as f32, h as f32) };
            let reich = if senkrecht { ry } else { rx };
            let mut entlang = -reich * 1.4 + e * (lang + 2.8 * reich);
            if gl.richtung == 1 || gl.richtung == 3 { entlang = lang - entlang; }
            let bogen = quer * (0.56 - 0.16 * (p * std::f32::consts::PI).sin());
            let (cx, cy) = if senkrecht { (x + bogen as i32, y + entlang as i32) }
                           else { (x + entlang as i32, y + bogen as i32) };
            let warm = mischen(farbe, 0xFFFFFF, 0.72) & 0xFFFFFF;
            g.push_clip(x + 1, y + 1, (w - 2).max(0), (h - 2).max(0));
            let dunkel = (255.0 - 170.0 * staerke).round().clamp(110.0, 255.0) as i64;
            if dunkel < 255 {
                g.blend_mode(2);
                g.box_fill(x + 1, y + 1, x + w - 2, y + h - 2, (dunkel << 16) | (dunkel << 8) | dunkel);
            }
            g.blend_mode(1);
            g.lichtfleck(cx, cy, (rx * 1.7) as i32, (ry * 1.7) as i32, (alpha(staerke * 0.22) << 24) | farbe);
            g.lichtfleck(cx, cy, rx as i32, ry as i32, (alpha(staerke * 0.55) << 24) | warm);
            g.lichtfleck(cx, cy, (rx * 0.45) as i32, (ry * 0.45) as i32, (alpha(staerke * 0.25) << 24) | heiss);
            if gl.rand {
                // Wo das Licht die Kante streift, leuchtet sie auf.
                let a = (alpha(staerke * 0.9) << 24) | heiss;
                if senkrecht {
                    g.lichtfleck(x + 1, cy, 3, (ry * 1.2) as i32, a);
                    g.lichtfleck(x + w - 2, cy, 3, (ry * 1.2) as i32, a);
                } else {
                    g.lichtfleck(cx, y + 1, (rx * 1.2) as i32, 3, a);
                    g.lichtfleck(cx, y + h - 2, (rx * 1.2) as i32, 3, a);
                }
            }
            g.blend_mode(0);
            g.pop_clip();
            return;
        }
        // Ein Band: zwei Verlaeufe von durchsichtig zur Farbe und zurueck.
        // Alpha 0 hiesse DECKEND, darum beginnt "durchsichtig" bei 1.
        let band = |g: &mut Graphics, mitte: f32, halb: f32, col: i64, a: i64, q0: i32, q1: i32| {
            let halb = halb.max(1.0);
            let voll = (a << 24) | col;
            let leer = (1i64 << 24) | col;
            let v0 = mitte - halb;
            let v1 = mitte + halb;
            if senkrecht {
                let (a0, a1, a2) = (y + v0 as i32, y + mitte as i32, y + v1 as i32);
                g.gradient_rect(q0, a0, q1, a1, leer, voll, true);
                g.gradient_rect(q0, a1, q1, a2, voll, leer, true);
            } else {
                let (a0, a1, a2) = (x + v0 as i32, x + mitte as i32, x + v1 as i32);
                g.gradient_rect(a0, q0, a1, q1, leer, voll, false);
                g.gradient_rect(a1, q0, a2, q1, voll, leer, false);
            }
        };
        let (q0, q1) = if senkrecht { (x + 1, x + w - 2) } else { (y + 1, y + h - 2) };
        g.push_clip(x + 1, y + 1, (w - 2).max(0), (h - 2).max(0));
        g.blend_mode(1);
        // Mehrere Dreiecke uebereinander ergeben zusammen eine glockenfoermige
        // Kurve -- mit zwei oder drei saehe man ihre Knicke als Stufen.
        for (breite, anteil) in [(1.0, 0.10), (0.72, 0.10), (0.5, 0.12), (0.32, 0.14), (0.18, 0.16)] {
            band(g, c, bw * breite, farbe, alpha(staerke * anteil), q0, q1);
        }
        band(g, c, (bw * 0.08).max(3.0), heiss, alpha(staerke * 0.35), q0, q1);   // heisser Kern
        band(g, c, (bw * 0.025).max(1.5), heiss, alpha(staerke * 0.45), q0, q1);  // heisse Linie
        if gl.rand {
            // Der Rahmen leuchtet, wo der Streif ist: je zwei Punkte an den
            // beiden Kanten quer zur Laufrichtung.
            if senkrecht {
                band(g, c, bw * 0.6, heiss, alpha(staerke), x + 1, x + 2);
                band(g, c, bw * 0.6, heiss, alpha(staerke), x + w - 3, x + w - 2);
            } else {
                band(g, c, bw * 0.6, heiss, alpha(staerke), y + 1, y + 2);
                band(g, c, bw * 0.6, heiss, alpha(staerke), y + h - 3, y + h - 2);
            }
        }
        g.blend_mode(0);
        g.pop_clip();
    }

    /// Streiflicht (Art "rahmen"): eine schraege Lichtkante, die ueber das
    /// Fenster zieht, als leuchte jemand flach von der Seite darueber. Hell
    /// wird nur, was Kanten hat: wo die Linie den Fensterrahmen oder den
    /// Rahmen eines Widgets kreuzt, blitzt ein kurzes Stueck auf; der Inhalt
    /// bekommt einen schwachen schraegen Schimmer. `e` ist der Fortschritt
    /// (0..1, weich), `staerke` schon mit der Huelle verrechnet.
    pub(super) fn draw_streiflicht(&self, g: &mut Graphics, wi: usize, e: f32, staerke: f32, farbe: i64, heiss: i64) {
        let win = &self.windows[wi];
        let gl = &win.glanz;
        let (x, y, w, h) = (win.x, win.y, win.w, win.h);
        let senkrecht = gl.richtung >= 2;
        let (lang, quer) = if senkrecht { (h as f32, w as f32) } else { (w as f32, h as f32) };
        // Die Neigung: oben liegt die Linie um `d` weiter vorn als unten.
        let d = quer * 0.55;
        let rand = lang * 0.08 + d;
        let mut c = -rand + e * (lang + 2.0 * rand);
        if gl.richtung == 1 || gl.richtung == 3 { c = lang - c; }
        let vor = if gl.richtung == 1 || gl.richtung == 3 { -1.0 } else { 1.0 };
        // u = entlang der Laufrichtung, v = quer dazu (0 = oben/links).
        let u_bei = |v: f32| c + vor * d * (0.5 - v / quer.max(1.0));
        let punkt = |u: f32, v: f32| -> (i32, i32) {
            if senkrecht { (x + v as i32, y + u as i32) } else { (x + u as i32, y + v as i32) }
        };
        let alpha = |f: f32| -> i64 { ((f * 255.0).round() as i64).clamp(1, 255) };
        // Ein Aufblitzen an einer Kante: laenglich entlang der Kante, ein Hof
        // in der Farbe und ein heisser Kern.
        let blitz = |g: &mut Graphics, u: f32, v: f32, laenge: f32, entlang_u: bool, st: f32| {
            let (px, py) = punkt(u, v);
            let (lx, ly) = if entlang_u != senkrecht { (laenge as i32, 3) } else { (3, laenge as i32) };
            let (hx, hy) = if entlang_u != senkrecht { ((laenge * 1.8) as i32, 7) } else { (7, (laenge * 1.8) as i32) };
            g.lichtfleck(px, py, hx, hy, (alpha(st * 0.35) << 24) | farbe);
            g.lichtfleck(px, py, lx, ly, (alpha(st) << 24) | heiss);
        };
        g.push_clip(x, y, w.max(0), h.max(0));
        g.blend_mode(1);
        // Der Schimmer: Flecke entlang der Linie, die sich zu einem weichen
        // schraegen Band ueberlagern.
        let n = 12;
        for k in 0..n {
            let v = (k as f32 + 0.5) / n as f32 * quer;
            let (px, py) = punkt(u_bei(v), v);
            let (rx, ry) = (quer * 0.10, quer / n as f32 * 1.6);
            let (ax, ay) = if senkrecht { (ry as i32, rx as i32) } else { (rx as i32, ry as i32) };
            g.lichtfleck(px, py, ax, ay, (alpha(staerke * 0.07) << 24) | farbe);
        }
        // Der Fensterrahmen: oben und unten, und die Seiten, wenn die Linie
        // sie gerade kreuzt.
        let lg = (lang * 0.10).max(24.0);
        blitz(g, u_bei(0.0), 1.0, lg, true, staerke);
        blitz(g, u_bei(quer), quer - 2.0, lg, true, staerke);
        for kante in [1.0, lang - 2.0] {
            // u_bei(v) = kante  =>  v = quer * (0.5 - (kante - c) / (vor * d))
            let v = quer * (0.5 - (kante - c) / (vor * d));
            if v >= 0.0 && v <= quer { blitz(g, kante, v, (quer * 0.12).max(20.0), false, staerke); }
        }
        // Die Rahmen der Widgets, etwas leiser.
        for wdg in &win.widgets {
            if !self.widget_shown(wi, wdg) || wdg.w < 30 || wdg.h < 16 { continue; }
            if matches!(wdg.kind, Kind::Label | Kind::Layout | Kind::Separator) { continue; }
            let (ax, ay, aw, ah) = self.abs_rect(wi, wdg);
            // Das Widget in u/v des Fensters.
            let (u0, v0, ul, vl) = if senkrecht { ((ay - y) as f32, (ax - x) as f32, ah as f32, aw as f32) }
                                   else { ((ax - x) as f32, (ay - y) as f32, aw as f32, ah as f32) };
            let klein = (ul * 0.18).clamp(10.0, lg);
            for vk in [v0, v0 + vl - 1.0] {
                let u = u_bei(vk);
                if u >= u0 && u <= u0 + ul { blitz(g, u, vk, klein, true, staerke * 0.7); }
            }
            for uk in [u0, u0 + ul - 1.0] {
                let v = quer * (0.5 - (uk - c) / (vor * d));
                if v >= v0 && v <= v0 + vl { blitz(g, uk, v, (vl * 0.25).clamp(8.0, 40.0), false, staerke * 0.7); }
            }
        }
        g.blend_mode(0);
        g.pop_clip();
    }

    /// Die Eintraege einer Werkzeugleiste. Ein Knopf zeigt seine Flaeche nur,
    /// wenn er etwas zu sagen hat -- unter der Maus, gedrueckt oder
    /// eingeschaltet; sonst steht das Sinnbild frei auf dem Streifen.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_leiste(&self, g: &mut Graphics, wi: usize, idx: usize, wdg: &Widget, ax: i32, ay: i32, w: i32, h: i32) {
        let l = match wdg.leiste.as_ref() { Some(l) if !l.eintraege.is_empty() => l, _ => return };
        let gedrueckt = if self.press_origin == Some((wi, idx)) { l.gedrueckt } else { -1 };
        let rad = self.m("corner_radius").min(self.sk(5));
        let iy = ay + self.sk(3);
        let ih = (h - self.sk(6)).max(self.sk(12));
        let acc = self.th("accent");
        let bg = self.th("win_bg");
        let ruhig = mischen(self.th("text_fg"), self.th("muted_fg"), 0.25);
        g.push_clip(ax, ay, w, h);
        for (k, x0, x1) in self.tb_geom(ax, ay, w, h, wdg) {
            let e = &l.eintraege[k];
            match e.art {
                1 => {
                    let mx = (x0 + x1) / 2;
                    let (y1, y2) = (iy + self.sk(4), iy + ih - self.sk(4));
                    g.line(mx, y1, mx, y2, self.th("win_border"));
                    g.line(mx + 1, y1, mx + 1, y2, shade(bg, 18));
                }
                2 => {}
                _ => {
                    let (bx1, by1, bx2, by2) = (x0, iy, x1 - 1, iy + ih - 1);
                    let hover = l.hover == k as i32 && !e.aus;
                    let druck = gedrueckt == k as i32 && hover;
                    let ht = if e.aus { 0.0 } else { weich(l.hover_t.get(k).copied().unwrap_or(if hover { 1.0 } else { 0.0 })) };
                    if druck {
                        let f = shade(self.th("widget_bg"), -14);
                        g.round_gradient(bx1, by1, bx2, by2, rad, shade(f, -6), shade(f, 4));
                        g.round_rect(bx1, by1, bx2, by2, rad, self.th("widget_border"), false);
                    } else if e.an {
                        // Eingeschaltet: ein Hauch Akzent statt einer Flaeche --
                        // zwei eingeschaltete Knoepfe nebeneinander sollen noch
                        // wie Knoepfe aussehen, nicht wie ein Balken.
                        let a = (0x40 as f64 + 0x20 as f64 * ht).round() as i64;
                        g.round_rect(bx1, by1, bx2, by2, rad, (a << 24) | (acc & 0xFF_FFFF), true);
                        g.round_rect(bx1, by1, bx2, by2, rad, mischen(acc, bg, 0.35), false);
                    } else if ht > 0.0 {
                        // Blendet ein und wieder aus: faehrt man ueber die
                        // Leiste, zieht die Flaeche weich hinter der Maus her.
                        let f = shade(self.th("widget_bg"), 8);
                        g.round_gradient(bx1, by1, bx2, by2, rad, deckkraft(shade(f, 10), ht), deckkraft(shade(f, -6), ht));
                        self.gloss(g, bx1, by1, bx2, by2, rad, 0.6 * ht);
                        g.round_rect(bx1, by1, bx2, by2, rad, deckkraft(self.th("widget_border"), ht), false);
                    }
                    let c = if e.aus { self.th("muted_fg") } else if e.an { acc } else { ruhig };
                    let hat_bild = !e.symbol.is_empty() || e.bild >= 0;
                    let s = if l.symbol > 0 { self.sk(l.symbol) } else { (ih - self.sk(10)).clamp(self.sk(12), self.sk(22)) };
                    let mit_text = !e.text.is_empty() && (l.mit_text || !hat_bild);
                    let v = if druck { 1 } else { 0 };
                    let sx = if mit_text { x0 + (ih - s) / 2 } else { x0 + (x1 - x0 - s) / 2 };
                    let sy = iy + (ih - s) / 2 + v;
                    if !e.symbol.is_empty() {
                        sinnbild(g, &e.symbol, sx + v, sy, s, c, e.aus);
                    } else if e.bild >= 0 {
                        g.draw_image_rect(e.bild, sx + v, sy, s, s);
                    }
                    if mit_text {
                        let tx = if hat_bild { x0 + ih - self.sk(4) } else { x0 + self.sk(5) };
                        let sz = self.wsize(g, wdg);
                        let tc = if e.aus { self.th("muted_fg") } else { self.th("text_fg") };
                        self.wtext(g, wdg, tx + v, iy + (ih - sz) / 2 + v, e.text.clone(), tc);
                    }
                }
            }
        }
        // Der >>-Knopf, wenn nicht alles hineinpasst.
        if let (_, versteckt, Some((cx0, cx1))) = self.tb_layout(ax, ay, w, h, wdg) {
            if !versteckt.is_empty() {
                let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
                let offen = self.leiste_popup == Some((wi, idx));
                if offen || (mx >= cx0 && mx < cx1 && my >= iy && my < iy + ih) {
                    let f = shade(self.th("widget_bg"), if offen { -10 } else { 8 });
                    g.round_rect(cx0, iy, cx1 - 1, iy + ih - 1, rad, f, true);
                    g.round_rect(cx0, iy, cx1 - 1, iy + ih - 1, rad, self.th("widget_border"), false);
                }
                let s = ((cx1 - cx0) * 2 / 3).max(self.sk(8));
                let sy = iy + (ih - s) / 2;
                let sx = cx0 + (cx1 - cx0 - s) / 2;
                sinnbild(g, "rechts", sx - s / 5, sy, s, ruhig, false);
                sinnbild(g, "rechts", sx + s / 5, sy, s, ruhig, false);
            }
        }
        g.pop_clip();
    }

    /// Das Ueberlauf-Menue einer Werkzeugleiste -- wie ein Kontextmenue:
    /// Sinnbild, Name, gesperrte gedaempft, eingeschaltete in Akzentfarbe.
    pub(super) fn draw_leisten_popup(&self, g: &mut Graphics, wi: usize, i: usize) {
        let (px, py, pw, zeilen) = match self.tb_popup_geom(wi, i) { Some(x) => x, None => return };
        let l = match self.windows[wi].widgets[i].leiste.as_ref() { Some(l) => l, None => return };
        let ph = zeilen.last().map(|z| z.2 - py).unwrap_or(0);
        let rad = self.m("corner_radius").min(6);
        g.round_rect(px + 2, py + 3, px + pw + 1, py + ph + 2, rad.max(2), (0x55i64 << 24) as i64, true);
        g.round_rect(px, py, px + pw - 1, py + ph - 1, rad, self.th("widget_bg"), true);
        g.round_rect(px, py, px + pw - 1, py + ph - 1, rad, self.th("widget_border"), false);
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        let acc = self.th("accent");
        let ruhig = mischen(self.th("text_fg"), self.th("muted_fg"), 0.25);
        for (k, y0, y1) in zeilen {
            let e = &l.eintraege[k];
            if e.art == 1 {
                let my_ = (y0 + y1) / 2;
                g.line(px + self.sk(8), my_, px + pw - self.sk(8), my_, self.th("widget_border"));
                continue;
            }
            if !e.aus && mx >= px && mx < px + pw && my >= y0 && my < y1 {
                g.box_fill(px + 1, y0, px + pw - 2, y1 - 1, shade(self.th("widget_bg"), 22));
            }
            let c = if e.aus { self.th("muted_fg") } else if e.an { acc } else { ruhig };
            let s = self.sk(16);
            if !e.symbol.is_empty() {
                sinnbild(g, &e.symbol, px + self.sk(8), y0 + (y1 - y0 - s) / 2, s, c, e.aus);
            } else if e.bild >= 0 {
                g.draw_image_rect(e.bild, px + self.sk(8), y0 + (y1 - y0 - s) / 2, s, s);
            }
            let name = if !e.text.is_empty() { e.text.clone() } else if !e.tip.is_empty() { e.tip.clone() } else { e.symbol.clone() };
            let tc = if e.aus { self.th("muted_fg") } else if e.an { acc } else { self.th("text_fg") };
            self.ctext(g, px + self.sk(32), y0 + (y1 - y0 - self.ctext_height(g)) / 2, name, tc);
        }
    }

    /// Ein einzeiliges Textfeld zeichnen -- das Textfeld selbst und die
    /// bearbeitbare Klappliste (`rechts` = Platz fuer ihren Pfeil).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn zeichne_textfeld(&self, g: &mut Graphics, wi: usize, idx: usize, wdg: &Widget,
                        ax: i32, ay: i32, w: i32, h: i32, rechts: i32) {
        let focused = self.focus_widget == Some((wi, idx));
        let fg = self.txt_col(wdg);
        let bcol = if focused && !self.stil_hat(wdg, "border", stil::FOKUS) { self.wcol(wdg, "accent", "accent") }
                   else { self.wcol(wdg, "border", "widget_border") };
        self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"), bcol);
        // Im Passwortmodus stehen Punkte -- gemessen wird dann auch an
        // den Punkten, sonst saesse die Schreibmarke neben dem Text.
        // Die IME-Vorschau steht an der Schreibmarke mit im Text.
        let (anz_chars, caret_anz) = self.anzeige_mit_vorschau(wdg);
        let anzeige: String = anz_chars.iter().collect();
        let ty = ay + (h - self.wsize(g, wdg)).max(0) / 2;
        // Sinnbilder links/rechts (Suchfeld, GUI_TEXTINPUT_ICON) nehmen Platz.
        let (sl, sr) = self.ti_rand(wdg);
        let inner = (w - 10 - rechts - sl - sr).max(1);
        let tw = self.wtext_width(g, wdg, &anzeige);
        // Ausrichtung nur, solange der Text hineinpasst -- sonst gilt
        // links mit Rollen, damit die Schreibmarke sichtbar bleibt.
        let (tx, scroll) = if wdg.align >= 1 && tw <= inner {
            (if wdg.align == 1 { ax + 5 + sl + (inner - tw) / 2 } else { ax + 5 + sl + inner - tw }, 0)
        } else { (ax + 5 + sl, wdg.scroll) };
        // Inhalt auf das Feld-Innere clippen (langer Text laeuft nicht raus).
        g.push_clip(ax + 2 + sl, ay + 1, (w - 4 - rechts - sl - sr).max(0), (h - 2).max(0));
        if anzeige.is_empty() {
            // Der Platzhalter bleibt, bis getippt wird -- auch mit Fokus. Vorher
            // verschwand er mit dem Fokus, und ein Feld, das beim Start den
            // Fokus hat, zeigte ihn nie (Dogfooding 2026-10-09).
            if !wdg.placeholder.is_empty() {
                self.wtext(g, wdg, tx, ty, wdg.placeholder.clone(), self.leise(self.wcol(wdg, "bg", "win_bg")));
            }
        } else {
            let chars: Vec<char> = anz_chars;
            // Selektion-Highlight (halbtransparenter Akzent hinter dem Text).
            if focused && wdg.vorschau.is_empty() {
                let lo = wdg.caret.min(wdg.sel_anchor).clamp(0, chars.len() as i32);
                let hi = wdg.caret.max(wdg.sel_anchor).clamp(0, chars.len() as i32);
                if lo != hi {
                    for (x0, x1) in self.zeilen_bereiche(g, wdg, &chars, lo as usize, hi as usize) {
                        g.box_fill(tx + x0 - scroll, ay + 2, tx + x1 - scroll, ay + h - 3, self.selection_bg(wdg));
                    }
                }
            }
            self.wtext(g, wdg, tx - scroll, ty, anzeige.clone(), fg);
            self.rs_textfeld_wellen(g, wdg, &chars, tx - scroll, ay + h - self.sk(6));
            // Die IME-Vorschau unterstreichen -- so sieht man, was
            // noch nicht fertig ist.
            if !wdg.vorschau.is_empty() {
                let von = wdg.caret.clamp(0, chars.len() as i32) as usize;
                let bis = (von + wdg.vorschau.chars().count()).min(chars.len());
                for (x0, x1) in self.zeilen_bereiche(g, wdg, &chars, von, bis) {
                    g.line(tx + x0 - scroll, ay + h - 5, tx + x1 - scroll, ay + h - 5, fg);
                }
            }
        }
        // Caret (blinkend) an der gemessenen Position -- in der
        // Vorschau dort, wo die IME ihre Marke hat.
        if focused && self.caret_blink_on() {
            let az: Vec<char> = anzeige.chars().collect();
            let cx = tx + self.zeilen_x(g, wdg, &az, caret_anz.max(0) as usize) - scroll;
            g.line(cx, ay + 3, cx, ay + h - 4, fg);
        }
        g.pop_clip();
        self.feld_symbole_zeichnen(g, wdg, ax, ay, w, h);
    }

    pub(super) fn draw_widget(&self, g: &mut Graphics, wi: usize, idx: usize, wdg: &Widget) {
        // Die Rundung aus der Stildatei gilt fuer jede Flaeche dieses Widgets.
        let vorher = self.rad_jetzt.replace(self.w_radius(wdg));
        let form_vorher = self.form_jetzt.replace(self.w_form(wdg));
        if wdg.stil_auf.is_some() {
            let (ax, ay, w, h) = self.abs_rect(wi, wdg);
            self.schatten_zeichnen(g, wdg, ax, ay, w, h);
        }
        self.draw_widget_innen(g, wi, idx, wdg);
        self.form_jetzt.set(form_vorher);
        self.rad_jetzt.set(vorher);
    }

    pub(super) fn draw_widget_innen(&self, g: &mut Graphics, wi: usize, idx: usize, wdg: &Widget) {
        let (ax, ay, w, h) = self.abs_rect(wi, wdg);
        // Innenabstand: aus der Stildatei, sonst die Metrik -- eine Stelle
        // fuer Knopf, Kaestchen, Klappliste, Drehfeld und Liste.
        let pad = self.w_pad(wdg);
        match wdg.kind {
            Kind::Layout => {
                self.fluss_auswahl_zeichnen(g, wi, wdg);
                // Unsichtbar -- nur mit `rahmen` zum Entwickeln als gestrichelte Linie.
                if wdg.layout.as_ref().map(|l| l.rahmen).unwrap_or(false) {
                    let c = self.th("muted_fg");
                    let s = self.sk(6);
                    let mut x = ax;
                    while x < ax + w { g.line(x, ay, (x + s / 2).min(ax + w - 1), ay, c); g.line(x, ay + h - 1, (x + s / 2).min(ax + w - 1), ay + h - 1, c); x += s; }
                    let mut y = ay;
                    while y < ay + h { g.line(ax, y, ax, (y + s / 2).min(ay + h - 1), c); g.line(ax + w - 1, y, ax + w - 1, (y + s / 2).min(ay + h - 1), c); y += s; }
                }
            }
            Kind::Label => {
                let eigen = if wdg.color < 0 { self.th("text_fg") } else { wdg.color };
                let fg = if !wdg.enabled { self.th("muted_fg") }
                         else if let Some(c) = wdg.ov.get("fg") { *c }
                         else if wdg.color >= 0 { wdg.color }
                         else { self.stil_misch(wdg, "fg", self.stil_farbe(wdg, "fg", stil::BASIS).unwrap_or(eigen)) };
                // Zeilen einzeln: so gilt die Ausrichtung je Zeile, und ein
                // umbrechendes Label zeichnet genau das, was umbruch_layout
                // gemessen hat (samt Kuerzen, Auswahl und Alt-Buchstabe).
                let (pv, ph) = self.w_abstand(wdg).unwrap_or((0, 0));
                self.label_zeichnen(g, wdg, ax + ph, ay + pv, (w - 2 * ph).max(1), fg);
            }
            Kind::Panel => {
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
                    self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
                if !wdg.text.is_empty() {
                    g.box_fill(ax, ay, ax + w - 1, ay + 17, self.th("win_border"));
                    self.wtext(g, wdg, ax + 5, ay + 2, wdg.text.clone(), self.wcol(wdg, "fg", "title_fg"));
                }
                // Rollbalken rechts, wenn der Inhalt hoeher ist als das Panel.
                if let Some((tx, ty, bw, th, thy, thh)) = self.panel_bar_geom(wi, idx) {
                    g.box_fill(tx, ty, tx + bw - 1, ty + th - 1, shade(self.wcol(wdg, "bg", "widget_bg"), -10));
                    g.round_rect(tx + 1, thy, tx + bw - 2, thy + thh - 1, 3, self.wcol(wdg, "border", "widget_border"), true);
                }
            }
            Kind::Button => {
                let has_icon = wdg.sel >= 0 || !wdg.sinnbild.is_empty();
                let icon_only = has_icon && wdg.text.is_empty();
                let pressed = self.press_origin == Some((wi, idx));
                // Uebergaenge: eh = ueberfahren, ed = gedrueckt (beide 0..1).
                let (eh, ed) = if wdg.enabled { (weich(wdg.ueber_t), weich(wdg.druck_t)) } else { (0.0, 0.0) };
                if icon_only {
                    // Flacher Toolbar-Look: Flaeche nur bei Hover/Press --
                    // sie blendet ein, statt aufzutauchen.
                    let bgc = self.wcol(wdg, "bg", "widget_bg");
                    let anteil = eh.max(ed);
                    if anteil > 0.0 {
                        let c = mischen(shade(bgc, 24), shade(bgc, -20), ed);
                        let rad = if wdg.rund { (w.min(h) / 2).max(2) } else { self.m("corner_radius").min(6) };
                        g.round_rect(ax, ay, ax + w - 1, ay + h - 1, rad, deckkraft(c, anteil), true);
                    }
                } else if wdg.rund {
                    // Runder Knopf: der Eckenradius ist die halbe kurze Seite,
                    // damit aus dem Rechteck eine Kapsel bzw. ein Kreis wird.
                    let grund = self.wcol(wdg, "bg", "widget_bg");
                    let bg = mischen(mischen(grund, shade(grund, 30), eh), shade(grund, -30), ed);
                    let r = (w.min(h) / 2).max(2);
                    let gr = self.m_form("gradient");
                    if gr > 0 {
                        g.round_gradient(ax, ay, ax + w - 1, ay + h - 1, r, shade(bg, gr), shade(bg, -gr));
                        self.gloss(g, ax, ay, ax + w - 1, ay + h - 1, r, self.verlauf_anteil(h));
                    } else {
                        g.round_rect(ax, ay, ax + w - 1, ay + h - 1, r, bg, true);
                    }
                    g.round_rect(ax, ay, ax + w - 1, ay + h - 1, r,
                                 self.wcol(wdg, "border", "widget_border"), false);
                } else if matches!(wdg.variante, 5 | 6 | 7) {
                    // Umriss, flach, Verweis: der Knopf hat KEINE eigene Flaeche --
                    // sie tritt hervor, wenn man ihn beruehrt.
                    let acc = self.wcol(wdg, "accent", "accent");
                    let rad = self.m("corner_radius").min(self.sk(6));
                    let anteil = eh.max(ed);
                    match wdg.variante {
                        5 => {
                            if anteil > 0.0 {
                                let a = (0x2C as f64 * eh + (0x50 as f64 - 0x2C as f64 * eh) * ed).round() as i64;
                                if a >= 1 {
                                    g.round_rect(ax, ay, ax + w - 1, ay + h - 1, rad, (a.min(255) << 24) | (acc & 0xFF_FFFF), true);
                                }
                            }
                            let rc = if wdg.enabled { self.wcol(wdg, "border", "accent") } else { self.th("widget_border") };
                            g.round_rect(ax, ay, ax + w - 1, ay + h - 1, rad, rc, false);
                            g.round_rect(ax + 1, ay + 1, ax + w - 2, ay + h - 2, (rad - 1).max(0), rc, false);
                        }
                        6 => {
                            if anteil > 0.0 {
                                let bg = self.wcol(wdg, "bg", "widget_bg");
                                let f = mischen(shade(bg, 16), shade(bg, -12), ed);
                                g.round_rect(ax, ay, ax + w - 1, ay + h - 1, rad, deckkraft(f, anteil), true);
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Gefuellter Knopf: Grundfarbe aus eigener Farbe oder Knopfart,
                    // sonst das Thema. Ueberfahren und Druecken nehmen eigene
                    // Farben (Rollen hover/pressed), sonst hell/dunkel vom Grund.
                    let grund = self.knopf_grund(wdg);
                    let mut bg = grund.unwrap_or_else(|| self.th("widget_bg"));
                    // Gesperrt: gedaempft -- ausser die Stildatei sagt selbst,
                    // wie ein gesperrter Knopf aussieht (:disabled).
                    if !wdg.enabled && grund.is_some() && !self.stil_hat(wdg, "bg", stil::AUS) {
                        bg = mischen(bg, self.th("widget_bg"), 0.65);
                    }
                    // Hover/Druck: eigene Rollen, sonst die Schichten der
                    // Stildatei (button:hover { bg }), sonst hell/dunkel.
                    let bg_h = wdg.ov.get("hover").copied().or_else(|| self.stil_farbe(wdg, "bg", stil::HOVER))
                        .unwrap_or(shade(bg, if grund.is_some() { 18 } else { 30 }));
                    let bg_d = wdg.ov.get("pressed").copied().or_else(|| self.stil_farbe(wdg, "bg", stil::AKTIV))
                        .unwrap_or(shade(bg, -30));
                    // Die Grundfarbe nimmt ihre eigene Dauer, wenn die
                    // Stildatei eine nennt (`uebergang: bg 400`).
                    let (eh_bg, ed_bg) = if wdg.enabled { (Self::rolle_t(wdg, "bg", 0), Self::rolle_t(wdg, "bg", 2)) } else { (0.0, 0.0) };
                    bg = mischen(mischen(bg, bg_h, eh_bg), bg_d, ed_bg);
                    // Der Standard-Knopf traegt den Akzent als Rahmen -- man
                    // soll sehen, was Enter tun wird.
                    let rahmen = if self.windows[wi].default_btn == idx as i32 && wdg.variante != 1 {
                        self.wcol(wdg, "accent", "accent")
                    } else if let (Some(gr), None) = (grund, self.eigen(wdg, "border")) {
                        shade(gr, -40)
                    } else { self.wcol(wdg, "border", "widget_border") };
                    self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, bg, rahmen);
                }
                // Icon + optionaler Text.
                let isz = if has_icon {
                    if icon_only { (h - 8).min(w - 8).max(4) } else { (h - 10).max(4) }
                } else { 0 };
                if has_icon {
                    let iy = ay + (h - isz) / 2;
                    let ix = if icon_only { ax + (w - isz) / 2 } else { ax + pad };
                    if !wdg.sinnbild.is_empty() {
                        let s = if icon_only { isz.min(self.sk(24)) } else { isz.min(self.sk(18)) };
                        let c = if wdg.variante > 0 || self.eigen(wdg, "bg").is_some() || self.eigen(wdg, "fg").is_some() { self.knopf_text(wdg) }
                                else if wdg.enabled { mischen(self.th("text_fg"), self.th("muted_fg"), 0.25) } else { self.th("muted_fg") };
                        sinnbild(g, &wdg.sinnbild, ix + (isz - s) / 2, iy + (isz - s) / 2, s, c, !wdg.enabled);
                    } else {
                        g.draw_image_rect(wdg.sel as i64, ix, iy, isz, isz);
                    }
                }
                if !wdg.text.is_empty() {
                    // Beschriftung mittig -- vorher klebte sie am linken Rand.
                    // Mit Sinnbild bleibt der Platz dafuer links stehen und
                    // der Text zentriert sich im Rest.
                    let sz = self.wsize(g, wdg);
                    let tw = self.wtext_width(g, wdg, &self.anzeige_text(wdg));
                    let links = if has_icon { pad + isz + 4 } else { pad };
                    let frei = (w - links - pad).max(0);
                    let (versatz, beschnitten) = knopf_text_x(frei, tw);
                    let versatz = match wdg.align {
                        0 if !beschnitten => 0,
                        2 if !beschnitten => frei - tw,
                        _ => versatz,
                    };
                    let tx = ax + links + versatz;
                    let ty = ay + (h - sz) / 2;
                    if beschnitten {
                        g.push_clip(ax + links, ay + 1, frei, (h - 2).max(0));
                    }
                    let tc = self.knopf_text(wdg);
                    self.wtext_mn(g, wdg, tx, ty, &wdg.text, tc);
                    if wdg.variante == 7 && wdg.enabled && (wdg.hovered || pressed) {
                        let ul = ty + sz + self.sk(1);
                        g.line(tx, ul, tx + tw.min(frei), ul, tc);
                    }
                    if beschnitten {
                        g.pop_clip();
                    }
                }
            }
            Kind::Checkbox => {
                // Das Kaestchen ist so gross wie die Metrik, nicht wie das
                // Rechteck: das reicht seit dem Automass ueber den Text.
                let cs = self.m("check_size").min(h).max(8);
                let (ay, w, h) = (ay + (h - cs) / 2, cs, cs);
                let acc = self.acc_col(wdg);
                let bordc = self.wcol(wdg, "border", "widget_border");
                if self.modern() {
                    // Gerundetes Kaestchen; gefuellt + Haekchen wenn aktiv.
                    if wdg.checked {
                        if self.m_form("gradient") > 0 {
                            g.round_gradient(ax, ay, ax + w - 1, ay + h - 1, 3,
                                             shade(acc, 26), shade(acc, -22));
                            self.gloss(g, ax, ay, ax + w - 1, ay + h - 1, 3, self.verlauf_anteil(h));
                        } else {
                            g.round_rect(ax, ay, ax + w - 1, ay + h - 1, 3, acc, true);
                        }
                        let ck = self.th("win_bg");   // Kontrast-Haekchen auf Akzentflaeche
                        let (x1, y1) = (ax + w * 28 / 100, ay + h * 52 / 100);
                        let (x2, y2) = (ax + w * 44 / 100, ay + h * 70 / 100);
                        let (x3, y3) = (ax + w * 74 / 100, ay + h * 30 / 100);
                        g.line(x1, y1, x2, y2, ck);
                        g.line(x2, y2, x3, y3, ck);
                    } else {
                        // Leer = versenkte Mulde: man sieht, dass hier etwas
                        // hineingehoert.
                        self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
                                       self.wcol(wdg, "bg", "widget_bg"),
                                       mischen(bordc, acc, if wdg.enabled { weich(wdg.ueber_t) } else { 0.0 }));
                    }
                } else {
                    g.rect(ax, ay, ax + w - 1, ay + h - 1, bordc);
                    if wdg.ueber_t > 0.0 { g.rect(ax - 1, ay - 1, ax + w, ay + h, deckkraft(acc, weich(wdg.ueber_t))); }
                    if wdg.checked { g.box_fill(ax + 3, ay + 3, ax + w - 4, ay + h - 4, acc); }
                }
                self.wtext_mn(g, wdg, ax + w + pad, ay, &wdg.text, self.txt_col(wdg));
            }
            Kind::Slider if wdg.vert => self.draw_vslider(g, wdg, ax, ay, w, h),
            Kind::Slider => {
                let handle_w = self.m("slider_handle_w");
                let span = wdg.max - wdg.min;
                let ratio = if span != 0.0 { (wdg.value - wdg.min) / span } else { 0.0 };
                let acc = self.acc_col(wdg);
                if self.m_form("gradient") > 0 {
                    // Rinne: schmal und versenkt. Der zurueckgelegte Teil wird
                    // in der Akzentfarbe gefuellt -- so ist der Wert auch ohne
                    // Zahl ablesbar.
                    let (ty, tb) = (ay + h / 2 - 3, ay + h / 2 + 3);
                    self.fbox_tief(g, ax, ty, ax + w - 1, tb,
                                   self.wcol(wdg, "bg", "widget_bg"),
                                   self.wcol(wdg, "border", "widget_border"));
                    let bis = ax + (ratio * (w - 1) as f64) as i32;
                    if bis > ax + 2 {
                        g.round_gradient(ax + 1, ty + 1, bis, tb - 1, 3,
                                         shade(acc, 24), shade(acc, -18));
                    }
                    let hx = ax + (ratio * (w - handle_w) as f64) as i32 + handle_w / 2;
                    self.knauf(g, hx, ay + h / 2, (h / 2).max(5), self.griff_farbe(wdg));
                } else {
                    g.box_fill(ax, ay + h / 2 - 1, ax + w - 1, ay + h / 2 + 1, self.wcol(wdg, "bg", "widget_bg"));
                    g.rect(ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "border", "widget_border"));
                    let hx = ax + (ratio * (w - handle_w) as f64) as i32;
                    g.box_fill(hx, ay, hx + handle_w - 1, ay + h - 1, acc);
                }
                self.regler_marken_zeichnen(g, wdg, ax, ay, w, h);
            }
            Kind::Toggle => {
                // Die Pille ist doppelt so breit wie hoch; das Rechteck
                // reicht ueber die Beschriftung.
                let ph = self.m("check_size").max(self.sk(14)).min(h);
                let (ay, w, h) = (ay + (h - ph) / 2, ph * 2, ph);
                let acc = self.acc_col(wdg);
                let aus = self.wcol(wdg, "bg", "widget_bg");
                let r = h / 2;
                // Die Rinne faerbt sich mit dem Zustand ein -- ueber `value`,
                // also gleitend statt springend.
                let mix = weich(wdg.value as f32);
                let bahn = mischen(aus, acc, mix);
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, bahn,
                               self.wcol(wdg, "border", "widget_border"));
                let kx = ax + r + ((w - h) as f64 * mix) as i32;
                self.knauf(g, kx, ay + r, r - 2, self.griff_farbe(wdg));
                if !wdg.text.is_empty() {
                    self.wtext_mn(g, wdg, ax + w + pad, ay + (h - self.wsize(g, wdg)).max(0) / 2, &wdg.text, self.txt_col(wdg));
                }
            }
            Kind::Knob => {
                let acc = self.acc_col(wdg);
                let r = (w.min(h) / 2).max(8);
                let (cx, cy) = (ax + w / 2, ay + h / 2);
                let spanne = wdg.max - wdg.min;
                let anteil = if spanne != 0.0 { ((wdg.value - wdg.min) / spanne).clamp(0.0, 1.0) } else { 0.0 };
                // Skalenbogen: 270 Grad, unten offen -- wie am Mischpult.
                let (von, bis) = (135.0, 405.0);
                let zw = von + (bis - von) * anteil;
                g.ring(cx, cy, r - 3, r, von, bis, self.wcol(wdg, "border", "widget_border"), true);
                if anteil > 0.0 {
                    g.ring(cx, cy, r - 3, r, von, zw, acc, true);
                }
                // Metallkappe + Kerbe, die den Wert zeigt.
                self.knauf(g, cx, cy, r - 6, self.griff_farbe(wdg));
                let rad = zw.to_radians();
                let (i0, i1) = ((r - 14).max(2) as f64, (r - 7).max(3) as f64);
                g.line_thick(
                    cx + (rad.cos() * i0) as i32, cy + (rad.sin() * i0) as i32,
                    cx + (rad.cos() * i1) as i32, cy + (rad.sin() * i1) as i32,
                    2.0, self.txt_col(wdg),
                );
                if !wdg.text.is_empty() {
                    let tw = self.wtext_width(g, wdg, &wdg.text);
                    self.wtext(g, wdg, cx - tw / 2, ay + h + 2, wdg.text.clone(), self.txt_col(wdg));
                }
            }
            Kind::TextInput => self.zeichne_textfeld(g, wi, idx, wdg, ax, ay, w, h, 0),
            Kind::TextArea => {
                let focused = self.focus_widget == Some((wi, idx));
                let fg = self.txt_col(wdg);
                let bcol = if focused { self.wcol(wdg, "accent", "accent") } else { self.wcol(wdg, "border", "widget_border") };
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"), bcol);
                let pad = 5;
                let lh = self.ta_line_h(g, wdg);
                let scroll = wdg.scroll;
                g.push_clip(ax + 2, ay + 2, (w - 4).max(0), (h - 4).max(0));
                // Das Hintergrundbild zuerst: eingepasst, mit etwas Rand,
                // mittig -- unter Marken, Auswahl und Text.
                if wdg.hg_bild >= 0 && wdg.hg_deckkraft > 0 {
                    let (iw, ih) = (g.image_width(wdg.hg_bild).unwrap_or(0) as i32, g.image_height(wdg.hg_bild).unwrap_or(0) as i32);
                    let (bw, bh) = ((w - 4) * 92 / 100, (h - 4) * 92 / 100);
                    if iw > 0 && ih > 0 && bw > 0 && bh > 0 {
                        let f = (bw as f64 / iw as f64).min(bh as f64 / ih as f64);
                        let (dw, dh) = (((iw as f64 * f) as i32).max(1), ((ih as f64 * f) as i32).max(1));
                        g.draw_image_rect_alpha(wdg.hg_bild, ax + (w - dw) / 2, ay + (h - dh) / 2, dw, dh, wdg.hg_deckkraft);
                    }
                }
                // Der Platzhalter bleibt, bis getippt wird (wie im Textfeld);
                // mit Fokus laeuft das Zeichnen danach weiter -- fuer die
                // Schreibmarke und die Nummernspalte.
                let platzhalter = wdg.text.is_empty() && wdg.vorschau.is_empty() && !wdg.placeholder.is_empty();
                if platzhalter {
                    let gut = self.ta_gutter(g, wdg, 1);
                    self.wtext(g, wdg, ax + pad + gut, ay + pad, wdg.placeholder.clone(), self.leise(self.wcol(wdg, "bg", "win_bg")));
                }
                if !(platzhalter && !focused) {
                    let (chars, caret_anz) = self.anzeige_mit_vorschau(wdg);
                    let starts = Self::line_starts(&chars);
                    let view_lines = ((h - 2 * pad) / lh).max(1);
                    // Linke Kante des TEXTES: hinter der Nummernspalte und um
                    // den waagerechten Versatz nach links geschoben. Alles,
                    // was eine Spalte verortet, geht durch diese eine Zahl.
                    let gutter = self.ta_gutter(g, wdg, starts.len());
                    let rows = self.ta_rows(g, wdg, &chars, &starts, self.ta_breite(g, wdg, starts.len()));
                    let tx0 = ax + pad + gutter - wdg.scroll_x;
                    // Marken: ein Farbhauch ueber der ganzen Zeile, unter allem
                    // anderen. Nur die erste sichtbare Zeile eines umgebrochenen
                    // Absatzes traegt ihn, wie die Zeilennummer.
                    // Zeilenfarben: unter den Marken, damit ein Haltepunkt
                    // auf einer heissen Zeile sichtbar bleibt.
                    if !wdg.zeilenfarben.is_empty() {
                        for r in 0..view_lines {
                            let ri = scroll + r;
                            if ri < 0 || ri as usize >= rows.len() { continue; }
                            let (li, _, _) = rows[ri as usize];
                            if let Some(&(_, farbe)) = wdg.zeilenfarben.iter().find(|(z, _)| *z == li) {
                                let y = ay + pad + r * lh;
                                g.box_fill(ax + 2 + gutter, y, ax + w - 3, y + lh - 1, farbe);
                            }
                        }
                    }
                    if !wdg.marken.is_empty() {
                        for r in 0..view_lines {
                            let ri = scroll + r;
                            if ri < 0 || ri as usize >= rows.len() { continue; }
                            let (li, rs, _) = rows[ri as usize];
                            if rs != starts[li] { continue; }
                            if let Some(&(_, farbe)) = wdg.marken.iter().find(|(z, _)| *z == li) {
                                let y = ay + pad + r * lh;
                                g.box_fill(ax + 2 + gutter, y, ax + w - 3, y + lh - 2,
                                           0x38_000000 | (farbe & 0xFF_FFFF));
                            }
                        }
                    }
                    if wdg.aktive_zeile {
                        let crow = Self::ta_row_of(&rows, caret_anz.max(0) as usize) as i32;
                        let r = crow - scroll;
                        if r >= 0 && r < view_lines {
                            let y = ay + pad + r * lh;
                            g.box_fill(ax + 2 + gutter, y, ax + w - 3, y + lh - 2,
                                       shade(self.wcol(wdg, "bg", "win_bg"), 10));
                        }
                    }
                    // Text bleibt rechts der Nummernspalte.
                    g.push_clip(ax + 2 + gutter, ay + 2, (w - 4 - gutter).max(0), (h - 4).max(0));
                    // Einrueckungslinien, unter allem anderen. Nur die erste
                    // Zeile eines umgebrochenen Absatzes traegt sie -- der
                    // Umbruch selbst ist keine Stufe.
                    if wdg.einzugslinien {
                        let breite = wdg.tabbreite.max(1) as usize;
                        let stufe = self.wtext_width(g, wdg, &" ".repeat(breite)).max(1);
                        let farbe = 0x30_000000 | (fg & 0xFF_FFFF);
                        for r in 0..view_lines {
                            let ri = scroll + r;
                            if ri < 0 || ri as usize >= rows.len() { continue; }
                            let (li, rs, _) = rows[ri as usize];
                            if rs != starts[li] { continue; }
                            let tiefe = Self::ta_einzug_tiefe(&chars, &starts, li, breite);
                            let y = ay + pad + r * lh;
                            for k in 1..tiefe {
                                let x = tx0 + k as i32 * stufe;
                                g.line(x, y, x, y + lh - 2, farbe);
                            }
                        }
                    }
                    // Selektion-Highlight pro sichtbarer Zeile (halbtransparenter Akzent).
                    // Jede Marke hat ihre eigene Auswahl -- die fuehrende
                    // zuerst, dann die weiteren.
                    let mut bereiche: Vec<(i32, i32)> = vec![(caret_anz, wdg.sel_anchor)];
                    bereiche.extend(wdg.marken_zusatz.iter().copied());
                    // Auch OHNE Fokus, dann gedaempft: eine Suchleiste nimmt dem
                    // Feld den Fokus und markiert darin den Treffer -- ohne das
                    // saehe man nicht, was [Ersetzen] gleich ersetzt (jeder
                    // Editor zeigt eine inaktive Auswahl so). Nur waehrend einer
                    // Umwandlung der Eingabemethode bleibt sie weg, die Vorschau
                    // hat dort den Platz.
                    if !(focused && !wdg.vorschau.is_empty()) {
                        let selbg = if focused { self.selection_bg(wdg) } else {
                            let acc = self.wcol(wdg, "accent", "accent");
                            (0x30i64 << 24) | (acc & 0xFFFFFF)
                        };
                        for &(mc, ma) in bereiche.iter() {
                            let lo = mc.min(ma).clamp(0, chars.len() as i32);
                            let hi = mc.max(ma).clamp(0, chars.len() as i32);
                            if lo == hi { continue; }
                            for r in 0..view_lines {
                                let ri = scroll + r;
                                if ri < 0 || ri as usize >= rows.len() { continue; }
                                let (_, rs, re) = rows[ri as usize];
                                let (lstart, lend) = (rs as i32, re as i32);
                                let a = lo.max(lstart);
                                let b = hi.min(lend);
                                if b < a || (a == b && hi <= lend) { continue; }
                                let zeile = &chars[lstart as usize..lend as usize];
                                let y = ay + pad + r * lh;
                                let mut stuecke = self.zeilen_bereiche(g, wdg, zeile, (a - lstart) as usize, (b - lstart) as usize);
                                stuecke.retain(|s| s.0 != s.1);
                                // Laeuft die Auswahl ueber das Zeilenende, kommt am
                                // Ende der Zeile ein Stueck dazu.
                                if hi > lend {
                                    let ende = self.zeilen_x(g, wdg, zeile, zeile.len());
                                    stuecke.push((ende, ende + self.wtext_width(g, wdg, " ").max(self.sk(4))));
                                }
                                for (x0, x1) in stuecke { g.box_fill(tx0 + x0, y, tx0 + x1, y + lh - 2, selbg); }
                            }
                        }
                    }
                    let hin_rects = self.ta_hinweis_rects(g, wdg, ax, ay, &chars, &starts, &rows, view_lines);
                    for r in 0..view_lines {
                        let ri = scroll + r;
                        if ri < 0 || ri as usize >= rows.len() { continue; }
                        let (li, rs, re) = rows[ri as usize];
                        let y = ay + pad + r * lh;
                        let zeile: String = chars[rs..re].iter().collect();
                        let geformt = wdg.formatiert && wdg.stile_text == wdg.text
                            && wdg.stile.len() == chars.len()
                            && wdg.stile[rs..re].iter().any(|&b| b != 0);
                        let rtl = if wdg.spans.is_empty() && !geformt { None } else { self.rtl_lage(g, wdg, &chars[rs..re]) };
                        let breite = if wdg.spans.is_empty() && !geformt {
                            self.wtext(g, wdg, tx0, y, zeile.clone(), fg);
                            self.wtext_width(g, wdg, &zeile)
                        } else if let Some(lage) = rtl {
                            // Farben je logischem Zeichen; die Inline-Formate fallen
                            // in einer solchen Zeile weg.
                            let farben = self.zeilen_farben(wdg, rs, re - rs, fg);
                            self.rtl_bunt(g, wdg, tx0, y, &lage, |l| farben[l]);
                            lage.breite()
                        } else {
                            self.zeile_bunt(g, wdg, tx0, y, &zeile, rs, fg, ax + 2 + gutter, ax + w - 2);
                            self.wtext_width(g, wdg, &zeile)
                        };
                        // Zugeklappt: hinter der Kopfzeile steht, wie viele
                        // Zeilen darunter liegen. Ohne dieses Zeichen sieht
                        // eine gefaltete Datei aus wie eine lueckenhafte.
                        let mut ende_x = tx0 + breite;
                        if let Some(&(_, b)) = wdg.gefaltet.iter().find(|&&(v, _)| v == li + 1) {
                            let txt = format!("... {} Zeilen", b - li - 1);
                            let bw = self.wtext_width(g, wdg, &txt) + self.sk(10);
                            let bx = tx0 + breite + self.sk(10);
                            g.box_fill(bx, y + 1, bx + bw, y + lh - 3, shade(self.wcol(wdg, "bg", "win_bg"), 26));
                            self.wtext(g, wdg, bx + self.sk(5), y, txt, self.th("muted_fg"));
                            ende_x = bx + bw;
                        }
                        // Werte hinter der Zeile: nur an ihrer LETZTEN
                        // sichtbaren Reihe (bei Umbruch), mit Abstand, in
                        // gedaempfter Farbe auf einem Hauch Grund -- sie sind
                        // Anzeige, nicht Text.
                        let _ = (ende_x, re);
                        if let Some(&(_, bx, _, bw, _)) = hin_rects.iter().find(|h| h.0 == li && h.2 == y + 1) {
                            if let Some(pos) = wdg.hinweise.iter().position(|(z, _)| *z == li) {
                                let txt = &wdg.hinweise[pos].1;
                                let bg = self.wcol(wdg, "bg", "win_bg");
                                let eigene = wdg.hinweise_farben.get(pos).copied().unwrap_or(-1);
                                let farbe = if eigene >= 0 { eigene } else if wdg.hinweise_farbe >= 0 { wdg.hinweise_farbe } else { self.leise(bg) };
                                g.box_fill(bx, y + 1, bx + bw, y + lh - 3, 0x60_000000 | (shade(bg, 14) & 0xFF_FFFF));
                                self.wtext(g, wdg, bx + self.sk(6), y, txt.clone(), farbe);
                            }
                        }
                    }
                    // Wellenlinien unter Fehlern und Warnungen: je sichtbare
                    // Zeile der Teil, der in sie faellt, als Zickzack knapp
                    // unter der Grundlinie.
                    let rs_wellen = Self::rs_wellen(wdg);
                    if !wdg.wellen.is_empty() || !rs_wellen.is_empty() {
                        let zacke = self.sk(2).max(2);
                        for r in 0..view_lines {
                            let ri = scroll + r;
                            if ri < 0 || ri as usize >= rows.len() { continue; }
                            let (_, rs, re) = rows[ri as usize];
                            let yb = ay + pad + r * lh + lh - self.sk(4);
                            for &(s, l, c) in wdg.wellen.iter().chain(rs_wellen.iter()) {
                                let (s, e) = (s as usize, s as usize + l as usize);
                                if s > re || e < rs || (l > 0 && e <= rs) { continue; }
                                if l > 0 && s >= re && re > rs { continue; }
                                let a = s.max(rs).min(re);
                                let b = e.max(a).min(re);
                                let zeile = &chars[rs..re];
                                let mut stuecke = self.zeilen_bereiche(g, wdg, zeile, a - rs, b - rs);
                                stuecke.retain(|s| s.0 != s.1);
                                if stuecke.is_empty() { let x = self.zeilen_x(g, wdg, zeile, a - rs); stuecke.push((x, x)); }
                                for (x0, x1) in stuecke {
                                    let (x0, mut x1) = (tx0 + x0, tx0 + x1);
                                    // Ohne Laenge (oder am Zeilenende) ein kurzes Stueck.
                                    if x1 < x0 + self.sk(8) { x1 = x0 + self.sk(8); }
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
                            }
                        }
                    }
                    // Farbfelder: ein kleines Quadrat hinter dem Stueck, mit
                    // Rahmen -- ohne ihn verschwaende ein dunkles Feld im
                    // dunklen Grund.
                    for (k, fx, fy, kante) in
                            self.ta_farbfeld_rects(g, wdg, ax, ay, &chars, &rows, starts.len(), view_lines) {
                        let farbe = wdg.farbfelder[k].2;
                        g.box_fill(fx, fy, fx + kante, fy + kante, farbe);
                        g.rect(fx, fy, fx + kante, fy + kante, self.th("widget_border"));
                    }
                    g.pop_clip();
                    // Nummernspalte: eigener Grund, rechtsbuendige Zahlen,
                    // Trennlinie. Rechtsbuendig, damit der Text nicht wandert,
                    // sobald die Datei dreistellig wird.
                    if gutter > 0 {
                        g.box_fill(ax + 2, ay + 2, ax + pad + gutter - self.sk(5), ay + h - 3,
                                   self.wcol(wdg, "bg", "win_bg"));
                        for r in 0..view_lines {
                            let ri = scroll + r;
                            if ri < 0 || ri as usize >= rows.len() { continue; }
                            let (li, rs, _) = rows[ri as usize];
                            // Eine umgebrochene Fortsetzung traegt keine Nummer.
                            if rs != starts[li] { continue; }
                            let nr = (li + 1).to_string();
                            // Marke: ein Punkt am linken Rand der Nummernspalte.
                            if let Some(&(_, farbe)) = wdg.marken.iter().find(|(z, _)| *z == li) {
                                let rr = (lh / 4).max(3);
                                g.circle(ax + 2 + rr + self.sk(3), ay + pad + r * lh + lh / 2 - 1, rr, farbe);
                            }
                            let nx = ax + pad + gutter - self.sk(9) - self.ta_faltbreite(wdg)
                                     - self.wtext_width(g, wdg, &nr);
                            self.wtext(g, wdg, nx, ay + pad + r * lh, nr, self.th("muted_fg"));
                            // Faltpfeil: ausgefuellt nach unten = offen, nach
                            // rechts = zugeklappt. Nur an Kopfzeilen.
                            if let Some((fx, fw)) = self.ta_falt_spalte(g, wdg, ax, starts.len()) {
                                let ln = li + 1;
                                if wdg.faltbar.iter().any(|&(v, _)| v == ln) {
                                    let zu = wdg.gefaltet.iter().any(|&(v, _)| v == ln);
                                    let cy = ay + pad + r * lh + lh / 2 - 1;
                                    let a = (fw / 3).max(3);
                                    let c = self.th("muted_fg");
                                    if zu {
                                        g.triangle(fx + a - 1, cy - a, fx + a - 1, cy + a, fx + a * 2, cy, c);
                                    } else {
                                        g.triangle(fx, cy - a / 2, fx + a * 2, cy - a / 2, fx + a, cy + a, c);
                                    }
                                }
                            }
                        }
                        let gx = ax + pad + gutter - self.sk(4);
                        g.line(gx, ay + 2, gx, ay + h - 3, self.wcol(wdg, "border", "widget_border"));
                    }
                }
                // Die IME-Vorschau unterstreichen, je sichtbarer Zeile ihr Stueck.
                if focused && !wdg.vorschau.is_empty() {
                    let (chars, _) = self.anzeige_mit_vorschau(wdg);
                    let starts = Self::line_starts(&chars);
                    let rows = self.ta_rows(g, wdg, &chars, &starts, self.ta_breite(g, wdg, starts.len()));
                    let von = wdg.caret.clamp(0, chars.len() as i32) as usize;
                    let bis = (von + wdg.vorschau.chars().count()).min(chars.len());
                    let tx0 = ax + pad + self.ta_gutter(g, wdg, starts.len()) - wdg.scroll_x;
                    for (r, &(_, rs, re)) in rows.iter().enumerate() {
                        let (a, b) = (von.max(rs), bis.min(re));
                        if b <= a { continue; }
                        let y = ay + pad + (r as i32 - scroll) * lh + lh - 3;
                        if y < ay + 2 || y > ay + h - 3 { continue; }
                        for (x0, x1) in self.zeilen_bereiche(g, wdg, &chars[rs..re], a - rs, b - rs) {
                            g.line(tx0 + x0, y, tx0 + x1, y, fg);
                        }
                    }
                }
                if focused && self.caret_blink_on() {
                    let (chars, caret_anz) = self.anzeige_mit_vorschau(wdg);
                    let starts = Self::line_starts(&chars);
                    let rows = self.ta_rows(g, wdg, &chars, &starts, self.ta_breite(g, wdg, starts.len()));
                    // Die weiteren Marken zuerst, damit die fuehrende oben
                    // liegt -- sie steht an derselben Stelle wie sonst.
                    for &(mc, _) in wdg.marken_zusatz.iter() {
                        let mc = mc.clamp(0, chars.len() as i32);
                        let mrow = Self::ta_row_of(&rows, mc as usize);
                        let (_, ms, mend) = rows[mrow];
                        let me = (ms + (mc - ms as i32).max(0) as usize).min(chars.len());
                        let mx = ax + pad + self.ta_gutter(g, wdg, starts.len()) - wdg.scroll_x
                                 + self.zeilen_x(g, wdg, &chars[ms..mend], me - ms);
                        let my = ay + pad + (mrow as i32 - scroll) * lh;
                        if my >= ay + 2 && my + lh <= ay + h
                           && mx >= ax + 2 + self.ta_gutter(g, wdg, starts.len()) {
                            g.line(mx, my, mx, my + lh - 2, self.th("accent"));
                        }
                    }
                    let crow = Self::ta_row_of(&rows, caret_anz.max(0) as usize);
                    let (_, lstart, lend) = rows[crow];
                    let cend = (lstart + (caret_anz - lstart as i32).max(0) as usize).min(chars.len());
                    let cx = ax + pad + self.ta_gutter(g, wdg, starts.len()) - wdg.scroll_x
                             + self.zeilen_x(g, wdg, &chars[lstart..lend], cend - lstart);
                    let cy = ay + pad + (crow as i32 - scroll) * lh;
                    // Auch die Schreibmarke bleibt rechts der Nummernspalte --
                    // sonst blinkte sie beim Scrollen nach rechts in den Zahlen.
                    if cy >= ay + 2 && cy + lh <= ay + h
                       && cx >= ax + 2 + self.ta_gutter(g, wdg, starts.len()) {
                        g.line(cx, cy, cx, cy + lh - 2, fg);
                    }
                }
                // Mitlaufende Blockkoepfe, ueber allem: eigener Grund, die
                // Nummer in der Spalte, der Kopf gefaerbt wie im Text, darunter
                // eine Kante mit Schatten -- sonst hielte man ihn fuer eine
                // Zeile, die dort wirklich steht.
                if wdg.kopfzeilen > 0 && !wdg.faltbar.is_empty() && wdg.scroll > 0 {
                    let (chars, _) = self.anzeige_mit_vorschau(wdg);
                    let starts = Self::line_starts(&chars);
                    let rows = self.ta_rows(g, wdg, &chars, &starts, self.ta_breite(g, wdg, starts.len()));
                    let sicht = ((h - 2 * pad) / lh).max(1);
                    let koepfe = Self::ta_koepfe(wdg, &rows, sicht, scroll);
                    if !koepfe.is_empty() {
                        let gutter = self.ta_gutter(g, wdg, starts.len());
                        let bg = self.wcol(wdg, "bg", "win_bg");
                        let grund = mischen(bg, fg, 0.07);
                        let tx0 = ax + pad + gutter - wdg.scroll_x;
                        for (k, &li) in koepfe.iter().enumerate() {
                            let y = ay + pad + k as i32 * lh;
                            let y0 = if k == 0 { ay + 2 } else { y };
                            g.box_fill(ax + 2, y0, ax + w - 3, y + lh - 1, grund);
                            if gutter > 0 {
                                let nr = (li + 1).to_string();
                                let nx = ax + pad + gutter - self.sk(9) - self.ta_faltbreite(wdg)
                                         - self.wtext_width(g, wdg, &nr);
                                self.wtext(g, wdg, nx, y, nr, self.th("muted_fg"));
                            }
                            let rs = starts[li];
                            let re = rows.iter().find(|r| r.1 == rs).map(|r| r.2)
                                .unwrap_or_else(|| starts.get(li + 1).map(|&n| n.saturating_sub(1)).unwrap_or(chars.len()));
                            let zeile: String = chars[rs..re.max(rs)].iter().collect();
                            g.push_clip(ax + 2 + gutter, y0, (w - 4 - gutter).max(0), (y + lh - y0).max(0));
                            if wdg.spans.is_empty() { self.wtext(g, wdg, tx0, y, zeile, fg); }
                            else { self.zeile_bunt(g, wdg, tx0, y, &zeile, rs, fg, ax + 2 + gutter, ax + w - 2); }
                            g.pop_clip();
                        }
                        let yb = ay + pad + koepfe.len() as i32 * lh - 1;
                        g.line(ax + 2, yb, ax + w - 3, yb, self.wcol(wdg, "border", "widget_border"));
                        g.box_fill(ax + 2, yb + 1, ax + w - 3, yb + self.sk(3), 0x28_000000);
                    }
                }
                g.pop_clip();
            }
            Kind::Spinner => {
                let focused = self.focus_widget == Some((wi, idx));
                let bcol = if focused { self.wcol(wdg, "accent", "accent") } else { self.wcol(wdg, "border", "widget_border") };
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, self.wcol(wdg, "bg", "win_bg"), bcol);
                let (up, down, bx) = Self::spinner_btn_rects(ax, ay, w, h);
                let bordc = self.wcol(wdg, "border", "widget_border");
                // Trennlinien vor und in der Buttonspalte.
                g.line(bx, ay + 1, bx, ay + h - 2, bordc);
                g.line(bx, ay + h / 2, ax + w - 1, ay + h / 2, bordc);
                // Zahlenwert links (auf das Feldinnere geclippt) -- oder, solange
                // getippt wird, der Text samt Auswahl und Schreibmarke.
                g.push_clip(ax + 2, ay + 1, (bx - ax - 3).max(0), (h - 2).max(0));
                let ty = ay + (h - self.wsize(g, wdg)).max(0) / 2;
                let tipp = match &self.spin_edit {
                    Some((ewi, ei, chars, c, a, _, _)) if (*ewi, *ei) == (wi, idx) && focused => Some((chars.clone(), *c, *a)),
                    _ => None,
                };
                if let Some((chars, c, a)) = tipp {
                    let text: String = chars.iter().collect();
                    let (lo, hi) = (c.min(a), c.max(a));
                    let bis = |n: i32| -> String { chars.iter().take(n.max(0) as usize).collect() };
                    let (x_lo, x_hi) = (self.wtext_width(g, wdg, &bis(lo)), self.wtext_width(g, wdg, &bis(hi)));
                    if lo != hi {
                        g.box_fill(ax + pad + x_lo, ay + 3, ax + pad + x_hi, ay + h - 4, self.selection_bg(wdg));
                    }
                    self.wtext(g, wdg, ax + pad, ty, text, self.txt_col(wdg));
                    if lo == hi && self.caret_blink_on() {
                        g.line(ax + pad + x_lo, ay + 4, ax + pad + x_lo, ay + h - 5, self.txt_col(wdg));
                    }
                } else {
                    self.wtext(g, wdg, ax + pad, ty, Self::fmt_num(wdg.value), self.txt_col(wdg));
                }
                g.pop_clip();
                // Hover-Highlight + Pfeil-Dreiecke der beiden Schaltflaechen.
                let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
                let fg = self.txt_col(wdg);
                let hl = shade(self.wcol(wdg, "bg", "widget_bg"), 30);
                if wdg.enabled && Self::in_rect(mx, my, up) {
                    g.box_fill(up.0 + 1, up.1 + 1, up.0 + up.2 - 2, up.1 + up.3 - 1, hl);
                }
                let (ucx, ucy) = (up.0 + up.2 / 2, up.1 + up.3 / 2);
                g.line(ucx - 4, ucy + 2, ucx, ucy - 2, fg);   // ^
                g.line(ucx, ucy - 2, ucx + 4, ucy + 2, fg);
                if wdg.enabled && Self::in_rect(mx, my, down) {
                    g.box_fill(down.0 + 1, down.1, down.0 + down.2 - 2, down.1 + down.3 - 2, hl);
                }
                let (dcx, dcy) = (down.0 + down.2 / 2, down.1 + down.3 / 2);
                g.line(dcx - 4, dcy - 2, dcx, dcy + 2, fg);   // v
                g.line(dcx, dcy + 2, dcx + 4, dcy - 2, fg);
            }
            Kind::Splitter => {
                let vert = wdg.group == "v";
                let active = self.active_split == Some((wi, idx));
                let col = if active || wdg.hovered { self.acc_col(wdg) }
                          else { self.wcol(wdg, "border", "widget_border") };
                g.box_fill(ax, ay, ax + w - 1, ay + h - 1, col);
                // Drei Griff-Punkte mittig laengs des Balkens.
                let dot = self.th("win_bg");
                let (cx, cy) = (ax + w / 2, ay + h / 2);
                for k in -1..=1 {
                    if vert { g.box_fill(cx - 1, cy + k * 5 - 1, cx + 1, cy + k * 5 + 1, dot); }
                    else { g.box_fill(cx + k * 5 - 1, cy - 1, cx + k * 5 + 1, cy + 1, dot); }
                }
            }
            Kind::Radio => {
                let cs = self.m("check_size").min(h).max(8);
                let (ay, w, h) = (ay + (h - cs) / 2, cs, cs);
                let acc = self.acc_col(wdg);
                let (cx, cy, r) = (ax + w / 2, ay + h / 2, (w / 2).max(2));
                g.circle(cx, cy, r, self.wcol(wdg, "border", "widget_border"));   // Ring
                g.circle(cx, cy, (r - 1).max(1), self.th("win_bg"));
                if wdg.ueber_t > 0.0 {
                    g.circle(cx, cy, r, mischen(self.wcol(wdg, "border", "widget_border"), acc, weich(wdg.ueber_t)));
                    g.circle(cx, cy, (r - 1).max(1), self.th("win_bg"));
                }
                if wdg.checked { g.circle(cx, cy, (r - 4).max(1), acc); }   // Punkt
                self.wtext_mn(g, wdg, ax + w + pad, ay, &wdg.text, self.txt_col(wdg));
            }
            Kind::Progress if Self::ist_kreisel(wdg) => self.kreisel_zeichnen(g, wdg, ax, ay, w, h),
            Kind::Progress => {
                let acc = self.acc_col(wdg);
                self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
                    self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
                if wdg.unbestimmt {
                    // Laufendes Band: die Zeit treibt es, nicht der Wert --
                    // fuer alles, dessen Dauer man nicht kennt.
                    let bw = (w / 3).max(8);
                    let t = (g.get_time() * 0.6).fract();
                    let x0 = ax + 1 - bw + (t * (w - 2 + bw) as f64) as i32;
                    g.push_clip(ax + 1, ay + 1, (w - 2).max(0), (h - 2).max(0));
                    g.round_gradient(x0, ay + 1, x0 + bw - 1, ay + h - 2, 3, shade(acc, 24), shade(acc, -18));
                    g.pop_clip();
                } else {
                    let span = wdg.max - wdg.min;
                    let ratio = if span != 0.0 { ((wdg.value - wdg.min) / span).clamp(0.0, 1.0) } else { 0.0 };
                    let fw = (ratio * (w - 2) as f64) as i32;
                    if fw > 0 { g.box_fill(ax + 1, ay + 1, ax + fw, ay + h - 2, acc); }
                    let pct = format!("{}%", (ratio * 100.0).round() as i32);
                    let pctw = self.wtext_width(g, wdg, &pct);
                    self.wtext(g, wdg, ax + w / 2 - pctw / 2, ay + (h - self.wsize(g, wdg)).max(0) / 2, pct, self.txt_col(wdg));
                }
            }
            Kind::Dropdown if wdg.frei => {
                let pb = self.sk(DD_PFEIL_B);
                self.zeichne_textfeld(g, wi, idx, wdg, ax, ay, w, h, pb);
                let fg = self.txt_col(wdg);
                g.line(ax + w - pb, ay + 4, ax + w - pb, ay + h - 5, self.wcol(wdg, "border", "widget_border"));
                let (axr, cy) = (ax + w - pb / 2 - 4, ay + h / 2);   // ▼
                g.line(axr, cy - 2, axr + 4, cy + 2, fg);
                g.line(axr + 4, cy + 2, axr + 8, cy - 2, fg);
            }
            Kind::Dropdown => {
                let bg = self.wcol(wdg, "bg", "widget_bg");
                let b = mischen(bg, shade(bg, 18), weich(wdg.ueber_t));
                self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, b, self.wcol(wdg, "border", "widget_border"));
                let fg = self.txt_col(wdg);
                let gewaehlt = wdg.sel >= 0 && (wdg.sel as usize) < wdg.items.len();
                // Nichts gewaehlt: der Platzhalter, gedaempft ("Bitte waehlen").
                let (txt, farbe) = if gewaehlt { (wdg.items[wdg.sel as usize].clone(), fg) }
                                   else { (wdg.placeholder.clone(), self.leise(bg)) };
                self.wtext(g, wdg, ax + pad, ay + (h - self.wsize(g, wdg)).max(0) / 2, txt, farbe);
                let (axr, cy) = (ax + w - 14, ay + h / 2);   // ▼
                g.line(axr, cy - 2, axr + 4, cy + 2, fg);
                g.line(axr + 4, cy + 2, axr + 8, cy - 2, fg);
            }
            Kind::ListBox => {
                self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
                    self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
                let fg = self.txt_col(wdg);
                let acc = self.acc_col(wdg);
                let muted = self.leise(self.wcol(wdg, "bg", "widget_bg"));
                let scroll = wdg.value as i32;
                let ih = self.list_zeile_h(wdg);
                let box_w = self.list_box_w(wdg);
                let l = wdg.list.as_deref();
                let hat_icon = l.map(|l| l.icons.iter().any(|&i| i >= 0)).unwrap_or(false);
                let icon_w = if hat_icon { ih } else { 0 };
                let ansicht = liste_ansicht(&wdg.items, l);
                // Mit Rollbalken enden die Zeilen vor ihm -- sonst laege der
                // Zusatztext rechts unter dem Griff.
                let bar = self.list_bar_geom(wi, idx);
                let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
                let wz = if bar.is_some() { w - self.sk(8) - 2 } else { w };
                g.push_clip(ax + 1, ay + 1, w - 2, h - 2);
                if ansicht.is_empty() {
                    // Leer ist eine Aussage: "keine Treffer" statt einer Flaeche,
                    // die aussieht, als sei die Liste kaputt.
                    if let Some(t) = l.map(|l| l.leer_text.clone()).filter(|t| !t.is_empty()) {
                        self.leer_hinweis(g, wdg, ax, ay + self.sk(10), w, &t, muted);
                    }
                }
                for (r, &k) in ansicht.iter().enumerate() {
                    let it = &wdg.items[k];
                    let iy = ay + r as i32 * ih - scroll;
                    if iy + ih < ay || iy > ay + h { continue; }
                    let kopf = l.map(|l| l.kopf.get(k).copied().unwrap_or(false)).unwrap_or(false);
                    let aus = l.map(|l| l.aus.get(k).copied().unwrap_or(false)).unwrap_or(false);
                    if kopf {
                        // Gruppenkopf: gedaempfte, fette Schrift und eine Linie --
                        // er gliedert, er ist kein Eintrag.
                        let ty = iy + (ih - self.wsize(g, wdg)).max(0) / 2;
                        let kc = mischen(acc, fg, 0.35);
                        let it = &Self::sichtbar_vorn(it, wz);
                        self.wtext(g, wdg, ax + pad, ty, it.clone(), kc);
                        self.wtext(g, wdg, ax + pad + 1, ty, it.clone(), kc);
                        let tw = self.wtext_width(g, wdg, it) + self.sk(8);
                        g.line(ax + pad + tw, iy + ih / 2, ax + w - pad, iy + ih / 2, self.wcol(wdg, "border", "widget_border"));
                        continue;
                    }
                    let gewaehlt = match l {
                        Some(l) if l.multi => l.sel.get(k).copied().unwrap_or(false),
                        _ => k as i32 == wdg.sel,
                    };
                    if gewaehlt {
                        g.box_fill(ax + 1, iy, ax + wz - 2, iy + ih - 1, shade(acc, -110));
                    } else if !aus && wdg.zeile_b.anteil(k as i32) > 0.0 {
                        let a = wdg.zeile_b.anteil(k as i32);
                        g.box_fill(ax + 1, iy, ax + wz - 2, iy + ih - 1, deckkraft(shade(self.wcol(wdg, "bg", "widget_bg"), 22), a));
                    }
                    let mut tx = ax + pad;
                    if box_w > 0 {
                        // Kaestchen wie das Checkbox-Widget: Rahmen, gefuellt mit Haken.
                        let cs = (ih - 8).max(8);
                        let bx = ax + 4;
                        let by = iy + (ih - cs) / 2;
                        let an = l.and_then(|l| l.checks.get(k).copied()).unwrap_or(false);
                        g.rect(bx, by, bx + cs - 1, by + cs - 1, self.wcol(wdg, "border", "widget_border"));
                        if an {
                            g.box_fill(bx + 2, by + 2, bx + cs - 3, by + cs - 3, acc);
                        }
                        tx = ax + box_w;
                    }
                    if let Some(ic) = l.and_then(|l| l.icons.get(k).copied()).filter(|&i| i >= 0) {
                        let isz = ih - 4;
                        g.draw_image_rect(ic, tx, iy + 2, isz, isz);
                    }
                    tx += icon_w + if icon_w > 0 { 2 } else { 0 };
                    // Auf der Auswahl folgt die Schrift dem GRUND: im hellen Thema
                    // stand dunkle Schrift auf dunkelblauer Zeile, und die gewaehlte
                    // Zeile war die einzige, die man nicht lesen konnte.
                    let fg = if gewaehlt { lesbar_auf(shade(acc, -110)) } else { fg };
                    let farbe = if aus { muted } else if gewaehlt { fg } else { l.and_then(|l| l.colors.get(k).copied()).filter(|&c| c >= 0).unwrap_or(fg) };
                    let ty = iy + (ih - self.wsize(g, wdg)).max(0) / 2;
                    // Zusatztext rechtsbuendig; der Haupttext endet vor ihm,
                    // statt darunter weiterzulaufen.
                    let detail = l.and_then(|l| l.details.get(k)).filter(|d| !d.is_empty());
                    let rechts = match detail {
                        Some(d) => {
                            let dw = self.wtext_width(g, wdg, d);
                            let dx = ax + wz - pad - dw;
                            self.wtext(g, wdg, dx, ty, d.clone(), if gewaehlt { mischen(fg, muted, 0.4) } else { muted });
                            dx - self.sk(10)
                        }
                        None => ax + wz - 2,
                    };
                    g.push_clip(tx, iy, (rechts - tx).max(0), ih);
                    let spans = l.and_then(|l| l.spans.get(k)).filter(|s| !s.is_empty());
                    match spans {
                        // Auf der Auswahl und gesperrt: EINE Farbe, die dort
                        // lesbar ist -- bunte Abschnitte auf Dunkelblau waeren
                        // genau das Problem, das `lesbar_auf` loest.
                        Some(sp) if !gewaehlt && !aus => {
                            // Laeufe ueber die Breite des VORSPANNS setzen, nicht
                            // Laufbreiten addieren -- wie im Textbereich, sonst
                            // klebten die Woerter an der Farbgrenze zusammen.
                            let zeichen: Vec<char> = Self::sichtbar_vorn(it, rechts - tx).chars().collect();
                            let mut pos = 0usize;
                            let mut laeufe: Vec<(usize, usize, i64)> = Vec::new();
                            for &(s, n, c) in sp.iter() {
                                let s = s.min(zeichen.len());
                                let e = (s + n).min(zeichen.len());
                                if s < pos || e <= s { continue; }
                                if s > pos { laeufe.push((pos, s, farbe)); }
                                laeufe.push((s, e, c));
                                pos = e;
                            }
                            if pos < zeichen.len() { laeufe.push((pos, zeichen.len(), farbe)); }
                            for (a, e, c) in laeufe {
                                let vor: String = zeichen[..a].iter().collect();
                                let teil: String = zeichen[a..e].iter().collect();
                                let x = tx + self.wtext_width(g, wdg, &vor);
                                self.wtext(g, wdg, x, ty, teil, c);
                            }
                        }
                        _ => {
                            let t = self.mit_auslassung(g, wdg, it, rechts - tx)
                                .unwrap_or_else(|| Self::sichtbar_vorn(it, rechts - tx));
                            self.wtext(g, wdg, tx, ty, t, farbe)
                        }
                    }
                    g.pop_clip();
                }
                // Rollbalken: Rinne und Griff, der Griff hebt sich unter der
                // Maus und beim Ziehen -- er ist ein Bedienelement, kein Zierrat.
                if let Some((tx, ty, bw, th, thy, thh)) = bar {
                    let grund = self.wcol(wdg, "bg", "widget_bg");
                    g.box_fill(tx, ty, tx + bw - 1, ty + th - 1, shade(grund, -10));
                    let aktiv = self.listbar_zug.map(|(a, b, _)| (a, b) == (wi, idx)).unwrap_or(false)
                        || (mx >= tx && mx < tx + bw && my >= thy && my < thy + thh);
                    let gc = if aktiv { mischen(self.wcol(wdg, "border", "widget_border"), acc, 0.5) }
                             else { self.wcol(wdg, "border", "widget_border") };
                    g.round_rect(tx + 1, thy, tx + bw - 2, thy + thh - 1, 3, gc, true);
                }
                // Umbenennen: ein Eingabefeld ueber der Zeile, mit Auswahl und
                // Schreibmarke -- gezeichnet wie ein Textfeld mit Fokus.
                if let (Some(e), Some((ex, ey, ew, eh))) = (l.and_then(|l| l.edit.as_ref()), self.list_edit_rect(wi, idx)) {
                    let bg = shade(self.wcol(wdg, "bg", "widget_bg"), 12);
                    g.box_fill(ex, ey, ex + ew - 1, ey + eh - 1, bg);
                    g.rect(ex, ey, ex + ew - 1, ey + eh - 1, acc);
                    let text: String = e.text.iter().collect();
                    let tx = self.list_text_x(wdg, ax);
                    let ty = ey + (eh - self.wsize(g, wdg)).max(0) / 2;
                    let x_bei = |g: &mut Graphics, n: i32| -> i32 {
                        let vor: String = e.text.iter().take(n.max(0) as usize).collect();
                        tx + self.wtext_width(g, wdg, &vor)
                    };
                    let (xa, xb) = (x_bei(g, e.anker.min(e.caret)), x_bei(g, e.anker.max(e.caret)));
                    let cx = x_bei(g, e.caret);
                    g.push_clip(ex + 1, ey + 1, (ew - 2).max(0), (eh - 2).max(0));
                    if e.anker != e.caret {
                        g.box_fill(xa, ey + 3, xb, ey + eh - 4, deckkraft(acc, 0.45));
                    }
                    self.wtext(g, wdg, tx, ty, text, fg);
                    g.line(cx, ey + 4, cx, ey + eh - 5, fg);
                    g.pop_clip();
                }
                g.pop_clip();
            }
            Kind::Image => {
                if wdg.sel >= 0 { self.draw_bild(g, wdg, ax, ay, w, h); }
            }
            Kind::Card => self.draw_karte(g, wi, idx, ax, ay, w, h),
            Kind::Canvas => {
                // Platzhalter-Flaeche; der User malt nach GUI_DRAW mit normalen
                // Befehlen in den per GUI_CANVAS_X/Y/W/H gelieferten Bereich.
                self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
                    self.wcol(wdg, "bg", "win_bg"), self.wcol(wdg, "border", "widget_border"));
            }
            Kind::Separator => {
                let my = ay + h / 2;
                g.line(ax, my, ax + w - 1, my, self.th("widget_border"));
            }
            Kind::Toolbar => {
                // Streifen: ein ruhiger Verlauf wie eine Flaeche, aber ohne
                // Rahmen ringsum -- nur die Unterkante trennt ihn vom Inhalt.
                let bg = self.th("win_bg");
                let grad = (self.m_form("gradient") as f64 * self.verlauf_anteil(h) * 0.6).round() as i32;
                if grad > 0 {
                    g.round_gradient(ax, ay, ax + w - 1, ay + h - 1, 0, shade(bg, 8 + grad), shade(bg, 4 - grad / 2));
                } else {
                    g.box_fill(ax, ay, ax + w - 1, ay + h - 1, shade(bg, 8));
                }
                g.line(ax, ay + h - 1, ax + w - 1, ay + h - 1, self.th("win_border"));
                self.draw_leiste(g, wi, idx, wdg, ax, ay, w, h);
            }
            Kind::StatusBar => {
                // Streifen mit einer Oberkante; die Felder durch feine Striche
                // getrennt. Ein klickbares Feld hebt sich unter der Maus ab.
                let bg = shade(self.th("win_bg"), 6);
                g.box_fill(ax, ay, ax + w - 1, ay + h - 1, bg);
                g.line(ax, ay, ax + w - 1, ay, self.th("win_border"));
                if let Some(s) = wdg.status.as_ref() {
                    let acc = self.acc_col(wdg);
                    let haupt = self.txt_col(wdg);
                    let neben = mischen(haupt, self.th("muted_fg"), 0.35);
                    let pad = self.sk(8);
                    let sz = self.wsize(g, wdg);
                    for (n, (k, x0, x1)) in self.sb_geom(ax, w, wdg).into_iter().enumerate() {
                        let f = &s.felder[k];
                        let ueber = f.klickbar && wdg.enabled && s.hover == k as i32;
                        if ueber { g.box_fill(x0 + 1, ay + 1, x1 - 2, ay + h - 1, shade(bg, 14)); }
                        if n > 0 { g.line(x0, ay + self.sk(5), x0, ay + h - self.sk(5), self.th("win_border")); }
                        if f.text.is_empty() { continue; }
                        let innen = (x1 - x0 - 2 * pad).max(0);
                        // Passt der Text nicht, endet er mit "..." statt mitten
                        // im Wort abgeschnitten (Dogfooding 2026-10-09: aus
                        // "geaendert" wurde "geaender", ohne dass man es merkt).
                        let text = self.text_kuerzen(g, wdg, &f.text, innen);
                        let tw = self.wtext_width(g, wdg, &text);
                        let tx = match f.ausr { 1 => x0 + pad + ((innen - tw) / 2).max(0), 2 => x0 + pad + (innen - tw).max(0), _ => x0 + pad };
                        let fc = if ueber { acc } else if k == 0 { haupt } else { neben };
                        g.push_clip(x0 + pad / 2, ay, (x1 - x0 - pad).max(0), h);
                        self.wtext(g, wdg, tx, ay + (h - sz).max(0) / 2, text, fc);
                        g.pop_clip();
                    }
                }
            }
            Kind::Breadcrumb => {
                // Teile als flache Knoepfe, dazwischen ein Winkel. Der letzte ist
                // "wo man steht" und steht kraeftiger da.
                if let Some(p) = wdg.pfad.as_ref() {
                    let (geo, punkte) = self.pf_geom(ax, w, wdg);
                    let acc = self.acc_col(wdg);
                    let haupt = self.txt_col(wdg);
                    let neben = mischen(haupt, self.th("muted_fg"), 0.45);
                    let muted = self.th("muted_fg");
                    let sz = self.wsize(g, wdg);
                    let rad = self.m("corner_radius").min(self.sk(5));
                    let ty = ay + (h - sz).max(0) / 2;
                    let trenn = self.sk(16);
                    let ws = self.sk(9);
                    let winkel = |g: &mut Graphics, x: i32| sinnbild(g, "rechts", x + (trenn - ws) / 2, ay + (h - ws) / 2, ws, muted, false);
                    g.push_clip(ax, ay, w, h);
                    if let Some((px0, px1)) = punkte {
                        if p.hover == -2 { g.round_rect(px0, ay + 2, px1 - 1, ay + h - 3, rad, shade(self.th("widget_bg"), 8), true); }
                        let tw = self.wtext_width(g, wdg, "...");
                        self.wtext(g, wdg, px0 + (px1 - px0 - tw) / 2, ty, "...".to_string(), neben);
                        winkel(g, px1);
                    }
                    let letzter = p.texte.len().saturating_sub(1);
                    for (n, &(k, x0, x1)) in geo.iter().enumerate() {
                        let ueber = wdg.enabled && p.hover == k as i32;
                        if ueber {
                            g.round_rect(x0, ay + 2, x1 - 1, ay + h - 3, rad, shade(self.th("widget_bg"), 8), true);
                        }
                        let tx = x0 + self.sk(6);
                        let fc = if ueber { acc } else if k == letzter { haupt } else { neben };
                        self.wtext(g, wdg, tx, ty, p.texte[k].clone(), fc);
                        if k == letzter { self.wtext(g, wdg, tx + 1, ty, p.texte[k].clone(), fc); }
                        if n + 1 < geo.len() { winkel(g, x1); }
                    }
                    g.pop_clip();
                }
            }
            Kind::GroupBox => {
                // Rahmen + eingelassener Titel oben-links (ueber dem Rahmen).
                g.rect(ax, ay + 7, ax + w - 1, ay + h - 1, self.th("widget_border"));
                if !wdg.text.is_empty() {
                    let tw = wdg.text.chars().count() as i32 * 8 + 8;
                    g.box_fill(ax + 8, ay, ax + 8 + tw.min(w - 16), ay + 13, self.th("win_bg"));
                    self.wtext(g, wdg, ax + 12, ay + 1, wdg.text.clone(), self.th("title_fg"));
                }
            }
            Kind::Accordion => {
                let kopf = self.sk(AKK_KOPF_H);
                let border = self.wcol(wdg, "border", "widget_border");
                let grund = self.wcol(wdg, "bg", "win_bg");
                let kopf_bg = self.wcol(wdg, "bg", "widget_bg");
                self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, grund, border);
                let acc = self.acc_col(wdg);
                let fg = self.txt_col(wdg);
                let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
                let fokus_hier = self.focus_widget == Some((wi, idx));
                g.push_clip(ax + 1, ay + 1, (w - 2).max(1), (h - 2).max(1));
                if let Some(a) = wdg.akk.as_deref() {
                    for (s, &(ky, inh)) in a.koepfe.iter().enumerate() {
                        let y0 = ay + ky - a.scroll;
                        if y0 > ay + h || y0 + kopf + inh < ay { continue; }
                        let offen = a.offen.get(s).copied().unwrap_or(false);
                        if inh > 0 {
                            g.box_fill(ax + 1, y0 + kopf, ax + w - 2, y0 + kopf + inh - 1, shade(grund, -6));
                        }
                        let ueber = mx >= ax && mx < ax + w && my >= y0.max(ay) && my < (y0 + kopf).min(ay + h);
                        let bg = if ueber { shade(kopf_bg, 14) } else { kopf_bg };
                        self.fbox(g, ax + 1, y0, ax + w - 2, y0 + kopf - 1, bg, border);
                        // Dreieck: zu zeigt nach rechts, offen nach unten.
                        let (tx, ty, d) = (ax + self.sk(14), y0 + kopf / 2, self.sk(5));
                        // Das Dreieck dreht sich mit dem Aufklappen: von
                        // rechts (zu) um 90 Grad nach unten (offen).
                        let t = weich(a.auf_t.get(s).copied().unwrap_or(if offen { 1.0 } else { 0.0 }));
                        let (sn, cs) = (t * std::f64::consts::FRAC_PI_2).sin_cos();
                        let dreh = |dx: i32, dy: i32| -> (i32, i32) {
                            let (dx, dy) = (dx as f64, dy as f64);
                            (tx + (dx * cs - dy * sn).round() as i32, ty + (dx * sn + dy * cs).round() as i32)
                        };
                        let (p1, p2, p3) = (dreh(-d / 2, -d), dreh(-d / 2, d), dreh(d, 0));
                        g.triangle(p1.0, p1.1, p2.0, p2.1, p3.0, p3.1, mischen(fg, acc, t));
                        let th = self.wsize(g, wdg);
                        self.wtext(g, wdg, ax + self.sk(28), y0 + (kopf - th).max(0) / 2, wdg.items[s].clone(), fg);
                        if fokus_hier && a.fokus == s as i32 {
                            g.rect(ax + 3, y0 + 2, ax + w - 4, y0 + kopf - 3, acc);
                        }
                    }
                    if a.inhalt_h > h {
                        let sw = self.sk(4);
                        let anteil = (h as f64 / a.inhalt_h as f64).min(1.0);
                        let griff = ((h - 4) as f64 * anteil) as i32;
                        let oben = ((h - 4 - griff) as f64 * (a.scroll as f64 / (a.inhalt_h - h) as f64).clamp(0.0, 1.0)) as i32;
                        g.box_fill(ax + w - sw - 2, ay + 2 + oben, ax + w - 3, ay + 2 + oben + griff, shade(fg, -60));
                    }
                }
                g.pop_clip();
            }
            Kind::Wizard => {
                let gm = self.wz_geom(wdg, ax, ay);
                let border = self.wcol(wdg, "border", "widget_border");
                let grund = self.wcol(wdg, "bg", "win_bg");
                let kbg = self.wcol(wdg, "bg", "widget_bg");
                self.fbox_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1, grund, border);
                let acc = self.acc_col(wdg);
                let fg = self.txt_col(wdg);
                let leise = self.leise(grund);
                let n = wdg.items.len() as i32;
                let sel = wdg.sel;
                let rand = self.sk(10);
                let rk = self.sk(11);
                let cy = ay + self.sk(18);
                let klein = (self.wsize(g, wdg) - self.sk(2)).max(8);
                let font = self.wfont(g, wdg);
                if n > 0 {
                    let slot = ((w - 2 * rand) / n).max(1);
                    let mitte = |k: i32| ax + rand + slot * k + slot / 2;
                    for k in 1..n {
                        let c = if k <= sel { acc } else { border };
                        g.line(mitte(k - 1) + rk + 2, cy, mitte(k) - rk - 2, cy, c);
                    }
                    for k in 0..n {
                        let cx = mitte(k);
                        if k < sel {
                            g.circle(cx, cy, rk, acc);
                            // Haken fuer erledigte Schritte.
                            let d = rk / 2;
                            g.line(cx - d, cy, cx - d / 3, cy + d / 2 + 1, lesbar_auf(acc));
                            g.line(cx - d / 3, cy + d / 2 + 1, cx + d, cy - d / 2, lesbar_auf(acc));
                        } else if k == sel {
                            g.circle(cx, cy, rk, acc);
                            let z = (k + 1).to_string();
                            let zb = g.text_width_in(&z, klein, font);
                            g.text_styled_stil(cx - zb / 2, cy - klein / 2, z, lesbar_auf(acc), font, klein, crate::schnitt::FETT);
                        } else {
                            g.circle(cx, cy, rk, kbg);
                            g.circle_outline(cx, cy, rk, border);
                            let z = (k + 1).to_string();
                            let zb = g.text_width_in(&z, klein, font);
                            g.text_styled(cx - zb / 2, cy - klein / 2, z, leise, font, klein);
                        }
                        // Titel darunter, auf die Breite seines Platzes gekuerzt.
                        let mut titel = wdg.items[k as usize].clone();
                        while titel.chars().count() > 1 && g.text_width_in(&titel, klein, font) > slot - self.sk(6) {
                            titel.pop();
                        }
                        let tb = g.text_width_in(&titel, klein, font);
                        let stil = if k == sel { crate::schnitt::FETT } else { 0 };
                        g.text_styled_stil(cx - tb / 2, cy + rk + self.sk(3), titel, if k == sel { fg } else { leise }, font, klein, stil);
                    }
                }
                g.line(ax + 1, ay + gm.kopf_h, ax + w - 2, ay + gm.kopf_h, border);
                g.line(ax + 1, gm.fuss_y, ax + w - 2, gm.fuss_y, border);
                // Knoepfe: Weiter bzw. Fertig im Akzent, gesperrte gedaempft.
                let z = wdg.wz.as_deref();
                let texte = z.map(|z| z.texte.clone()).unwrap_or_else(|| WzState::default().texte);
                let weiter_aus = z.map(|z| z.weiter_aus).unwrap_or(false) || n == 0;
                let letzter = sel + 1 >= n;
                let knoepfe = [
                    (gm.zurueck, texte[0].clone(), false, sel <= 0),
                    (gm.weiter, if letzter { texte[2].clone() } else { texte[1].clone() }, true, weiter_aus),
                    (gm.abbrechen, texte[3].clone(), false, false),
                ];
                for ((kx, ky, kb, kh), text, primaer, aus) in knoepfe {
                    let bg = if primaer && !aus { acc } else { kbg };
                    self.fbox(g, kx, ky, kx + kb - 1, ky + kh - 1, bg, if primaer && !aus { acc } else { border });
                    let tfg = if aus { self.leise(bg) } else if primaer { lesbar_auf(acc) } else { fg };
                    let tb = self.wtext_width(g, wdg, &text);
                    let th = self.wsize(g, wdg);
                    self.wtext(g, wdg, kx + (kb - tb) / 2, ky + (kh - th).max(0) / 2, text, tfg);
                }
            }
            Kind::TabControl => {
                // Koepfe oben, darunter die Flaeche -- die Kinder zeichnet die
                // Schleife danach ganz normal, sie liegen nur auf einer Seite.
                let kopf = self.sk(TC_KOPF_H);
                self.fbox_w(g, wdg.kind, ax, ay + kopf, ax + w - 1, ay + h - 1,
                    self.wcol(wdg, "bg", "win_bg"), self.wcol(wdg, "border", "widget_border"));
                let acc = self.acc_col(wdg);
                g.push_clip(ax, ay, w, kopf);
                let kreuz = if Self::tc_schliessbar(wdg) { self.sk(16) } else { 0 };
                for (i, x0, x1) in self.tc_koepfe(ax, w, wdg) {
                    let aktiv = i as i32 == wdg.sel;
                    let bg = if aktiv { self.wcol(wdg, "bg", "widget_bg") } else { shade(self.th("win_bg"), -10) };
                    g.box_fill(x0, ay + 2, x1 - 1, ay + kopf - 1, bg);
                    g.rect(x0, ay + 2, x1 - 1, ay + kopf - 1, self.wcol(wdg, "border", "widget_border"));
                    if aktiv { g.box_fill(x0, ay + kopf - 3, x1 - 1, ay + kopf - 1, acc); }
                    let fg = if aktiv { self.txt_col(wdg) } else { self.th("muted_fg") };
                    let th = self.wsize(g, wdg);
                    // Ein geschmaelerter Kopf kuerzt seinen Titel (tc_koepfe).
                    let titel = self.text_kuerzen(g, wdg, &wdg.items[i], x1 - x0 - self.sk(20) - kreuz);
                    self.wtext(g, wdg, x0 + self.sk(10), ay + 2 + (kopf - 2 - th).max(0) / 2,
                               titel, fg);
                    if Self::tc_schliessbar(wdg) {
                        // Zwei Striche wie am Reiter des Fensters.
                        let (kx, ky, k) = self.tc_kreuz(x1, ay);
                        g.line(kx, ky, kx + k - 1, ky + k - 1, fg);
                        g.line(kx + k - 1, ky, kx, ky + k - 1, fg);
                    }
                }
                g.pop_clip();
            }
            Kind::RichText => self.draw_richtext(g, wi, idx),
            Kind::TimePicker => {
                let (x0, schritt, n) = self.tp_geom(wdg, ax, w);
                let acc = self.acc_col(wdg);
                let fg = self.txt_col(wdg);
                let fw = schritt - self.sk(10);
                for k in 0..n {
                    let fx = x0 + k as i32 * schritt;
                    self.fbox_tief(g, fx, ay, fx + fw - 1, ay + h - 1,
                                   self.wcol(wdg, "bg", "widget_bg"), self.wcol(wdg, "border", "widget_border"));
                    if wdg.sel == k as i32 {
                        g.rect(fx, ay, fx + fw - 1, ay + h - 1, acc);
                    }
                    let txt = format!("{:02}", wdg.zeit[k]);
                    let tw = self.wtext_width(g, wdg, &txt);
                    let th = self.wsize(g, wdg);
                    self.wtext(g, wdg, fx + (fw - tw) / 2 - self.sk(5), ay + (h - th).max(0) / 2, txt, fg);
                    // Pfeile rechts im Feld: oben hoch, unten runter.
                    let px = fx + fw - self.sk(9);
                    let (o, u) = (ay + h / 4, ay + h * 3 / 4);
                    g.line(px - 3, o + 2, px, o - 2, fg);
                    g.line(px, o - 2, px + 3, o + 2, fg);
                    g.line(px - 3, u - 2, px, u + 2, fg);
                    g.line(px, u + 2, px + 3, u - 2, fg);
                    if k + 1 < n {
                        let cx = fx + fw + self.sk(3);
                        g.box_fill(cx, ay + h / 2 - self.sk(4), cx + 1, ay + h / 2 - self.sk(3), fg);
                        g.box_fill(cx, ay + h / 2 + self.sk(2), cx + 1, ay + h / 2 + self.sk(3), fg);
                    }
                }
            }
            Kind::Table => self.draw_table(g, wi, idx),
            Kind::Tree => self.draw_tree(g, wi, idx),
            Kind::ColorPicker => self.draw_colorpicker(g, wdg, ax, ay, w, h),
            Kind::DatePicker => if wdg.kompakt {
                self.draw_datumsfeld(g, wdg, ax, ay, w, h)
            } else {
                self.draw_datepicker(g, wdg, ax, ay, w, h)
            },
        }
        // Fokus-Ring: EINE Stelle fuer alle Arten, nach dem Zeichnen und damit
        // obenauf. Liegt AUSSERHALB der Widget-Flaeche (2 px Luft), sonst
        // verdeckte er Inhalt -- ein Kaestchen ist nur check_size gross, ein
        // Ring darin waere kaum zu sehen. Ohne sichtbaren Fokus waere die
        // Tab-Navigation wertlos: man wuesste nie, wo man gerade ist.
        // Rollbalken des Textbereichs, ueber dem Text (er liegt im Innenabstand).
        if wdg.kind == Kind::TextArea {
            if let Some((tx, ty, bw, th, thy, thh, _, _)) = self.ta_bar_geom(g, wi, idx) {
                let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
                g.box_fill(tx, ty, tx + bw - 1, ty + th - 1, shade(self.wcol(wdg, "bg", "win_bg"), -8));
                let aktiv = self.tabar_zug.map(|(a, b, _)| (a, b) == (wi, idx)).unwrap_or(false)
                    || (mx >= tx - 2 && mx < tx + bw + 1 && my >= thy && my < thy + thh);
                let rc = self.wcol(wdg, "border", "widget_border");
                g.round_rect(tx, thy, tx + bw - 1, thy + thh - 1, 2, if aktiv { mischen(rc, self.th("accent"), 0.5) } else { rc }, true);
            }
        }
        if wdg.fokus_t > 0.0 && wdg.kind.fokussierbar() {
            // Um eine GROSSE Flaeche (Metrik `verlauf_hoehe`) leiser: dort
            // liegt der Ring als zweiter Rahmen um den ersten, und um ein
            // Code-Feld von 900 Punkten sieht das unruhig aus; die Schreibmarke
            // zeigt den Fokus ohnehin. Ein Fuenftel bleibt immer stehen.
            let leise = self.verlauf_anteil(h);
            g.rect(ax - 3, ay - 3, ax + w + 2, ay + h + 2,
                   deckkraft(self.th("accent"), weich(wdg.fokus_t) * leise));
        }
        // Fehlerrahmen der Formularpruefung: zwei Linien in Rot, direkt am
        // Rand -- der Fokusring liegt weiter aussen, beide bleiben sichtbar.
        if !wdg.fehler.is_empty() {
            let rot = 0xE0_5050;
            g.rect(ax - 1, ay - 1, ax + w, ay + h, rot);
            g.rect(ax, ay, ax + w - 1, ay + h - 1, rot);
        }
    }

    /// Eine Textzeile in Abschnittsfarben zeichnen.
    ///
    /// Der Weg fuehrt ueber eine Farbe JE ZEICHEN und erst danach ueber das
    /// Zusammenfassen gleichfarbiger Laeufe. Direkt ueber die Abschnitte zu
    /// laufen waere kuerzer, aber dann muesste jeder Sonderfall (Abschnitt
    /// ragt ueber die Zeile hinaus, zwei ueberlappen sich, dazwischen klafft
    /// eine Luecke) einzeln bedacht werden -- und genau dort entstehen die
    /// Fehler, bei denen ein Zeichen die Farbe des Nachbarn bekommt.
    ///
    /// `zeilen_start` ist der Zeichen-Index des Zeilenanfangs im GESAMTEN
    /// Text; die Abschnitte zaehlen von dort.
    /// Vorne so viel von `s`, wie in `px` Punkte passen KANN: kein Zeichen ist
    /// schmaler als ein Punkt. Eine Liste zeichnete einen Eintrag sonst ganz
    /// und liess ihn nur vom Clip beschneiden -- eine Ausgabezeile mit 300000
    /// Zeichen kostete so jedes Bild bis zu 72 ms (Stresstest 2026-10-02).
    /// Wie viel Breite der TEXT eines Listeneintrags hat -- dieselbe Rechnung
    /// wie beim Zeichnen (Kaestchen, Sinnbild, Zusatztext rechts, Rollbalken).
    /// Gebraucht fuer den Tooltip mit dem ganzen Text.
    pub(super) fn liste_textplatz(&self, g: &Graphics, wi: usize, idx: usize, k: usize) -> i32 {
        let wdg = &self.windows[wi].widgets[idx];
        let pad = self.m("pad");
        let ih = self.list_zeile_h(wdg);
        let box_w = self.list_box_w(wdg);
        let l = wdg.list.as_deref();
        let hat_icon = l.map(|l| l.icons.iter().any(|&i| i >= 0)).unwrap_or(false);
        let icon_w = if hat_icon { ih } else { 0 };
        let (_, _, w, _) = self.abs_rect(wi, wdg);
        let wz = if self.list_bar_geom(wi, idx).is_some() { w - self.sk(8) - 2 } else { w };
        let tx = if box_w > 0 { box_w } else { pad } + icon_w + if icon_w > 0 { 2 } else { 0 };
        let rechts = match l.and_then(|l| l.details.get(k)).filter(|d| !d.is_empty()) {
            Some(d) => wz - pad - self.wtext_width(g, wdg, d) - self.sk(10),
            None => wz - 2,
        };
        rechts - tx
    }

    /// Text auf eine Breite kuerzen: passt er nicht, endet er mit "..." --
    /// statt mitten im Buchstaben abgeschnitten zu werden.
    pub(super) fn mit_auslassung(&self, g: &Graphics, wdg: &Widget, text: &str, breite: i32) -> Option<String> {
        // Nie mehr Zeichen messen, als in die Zeile passen KOENNEN (keins ist
        // schmaler als ein Punkt) -- ein Eintrag mit einer Million Zeichen
        // kostete sonst je Bild zwanzig Messungen ueber den ganzen Text.
        let cap = breite.max(0) as usize + 8;
        let kopf: String = text.chars().take(cap + 1).collect();
        let zu_lang = kopf.chars().count() > cap;
        if !zu_lang && self.wtext_width(g, wdg, &kopf) <= breite { return None; }
        let z: Vec<char> = kopf.chars().take(cap).collect();
        let (mut lo, mut hi) = (0usize, z.len());
        while lo < hi {
            let m = (lo + hi + 1) / 2;
            let t: String = z[..m].iter().collect::<String>().trim_end().to_string() + "...";
            if self.wtext_width(g, wdg, &t) <= breite { lo = m; } else { hi = m - 1; }
        }
        Some(z[..lo].iter().collect::<String>().trim_end().to_string() + "...")
    }

    pub(super) fn sichtbar_vorn(s: &str, px: i32) -> String {
        let cap = px.max(0) as usize + 8;
        if s.len() <= cap { return s.to_string(); }
        s.chars().take(cap).collect()
    }

    /// Farbe je Zeichen einer Zeile aus den Abschnitten (GUI_TEXTAREA_SPANS).
    pub(super) fn zeilen_farben(&self, wdg: &Widget, zeilen_start: usize, n: usize, grund: i64) -> Vec<i64> {
        let mut farbe = vec![grund; n];
        let ende = zeilen_start + n;
        // Die Abschnitte liegen sortiert -- ab dem ersten, der noch in die
        // Zeile hineinreicht, und nur solange er vor ihrem Ende beginnt.
        let erster = wdg.spans.partition_point(|&(s, l, _)| (s as usize + l as usize) <= zeilen_start);
        for &(s, l, c) in &wdg.spans[erster..] {
            let s = s as usize;
            if s >= ende { break; }
            let von = s.max(zeilen_start) - zeilen_start;
            let bis = ((s + l as usize).min(ende)) - zeilen_start;
            for f in &mut farbe[von..bis] { *f = c; }
        }
        farbe
    }

    pub(super) fn zeile_bunt(&self, g: &mut Graphics, wdg: &Widget, x: i32, y: i32,
                  zeile: &str, zeilen_start: usize, grund: i64, links: i32, rechts: i32) {
        let z: Vec<char> = zeile.chars().collect();
        if z.is_empty() { return; }
        let farbe = self.zeilen_farben(wdg, zeilen_start, z.len(), grund);
        // Inline-Formate: derselbe Lauf-Mechanismus, ein Lauf endet auch am
        // Stilwechsel. Gezeichnet wird der Stil NACHGEBILDET auf der
        // Grundschrift -- ein echter fetter Schnitt waere breiter, und
        // Schreibmarke, Auswahl und Klick messen am ungeformten Text.
        let geformt = wdg.formatiert && wdg.stile_text == wdg.text
            && wdg.stile.len() >= zeilen_start + z.len();
        let stil = |k: usize| -> u8 { if geformt { wdg.stile[zeilen_start + k] } else { 0 } };
        // Gleichfarbige Laeufe zeichnen. Die Position kommt aus der Breite des
        // VORSPANNS (Zeilenanfang bis hierher), nicht aus der Summe der
        // bisherigen Laeufe.
        //
        // Der Unterschied ist nicht theoretisch: eine Textbreite enthaelt den
        // Abstand ZWISCHEN den Zeichen, aber keinen hinter dem letzten. Wer
        // Laufbreiten addiert, verliert an jeder Farbgrenze genau diesen
        // Abstand, und die Woerter kleben aneinander. Ausserdem rechnen
        // Schreibmarke und Auswahl schon ueber den Vorspann -- zwei
        // Konventionen im selben Feld liefen unweigerlich auseinander.
        //
        // Gemessen wird nur, was SICHTBAR ist (`links`..`rechts`): jeder Lauf
        // misst seinen ganzen Vorspann, und ueber alle Laeufe einer langen
        // Zeile war das quadratisch -- eine Zeile mit 11000 Zeichen kostete
        // 850 ms je Bild, eine mit 270000 haengte die IDE (Stresstest
        // 2026-10-02). Der erste sichtbare Lauf kommt per Binaersuche ueber
        // die Laufanfaenge, gezeichnet wird bis zum rechten Rand.
        let mut anfaenge: Vec<usize> = Vec::new();
        let mut a = 0usize;
        while a < z.len() {
            anfaenge.push(a);
            let (c, st) = (farbe[a], stil(a));
            let mut b = a + 1;
            while b < z.len() && farbe[b] == c && stil(b) == st { b += 1; }
            a = b;
        }
        // Mit festem Font: einmal von links, Stueck fuer Stueck aufaddiert
        // (additiv, siehe text_breite_fest) -- eine Zeile mit 270000 Zeichen
        // kostete sonst je sichtbarem Lauf den ganzen Vorspann.
        if let (Some((sz, font)), false) = (self.wtext_fest(g, wdg), geformt) {
            let abstand = g.zeichenabstand() as f64;
            let mut summe = 0f64;
            for k in 0..anfaenge.len() {
                let lauf = anfaenge[k];
                let bis = anfaenge.get(k + 1).copied().unwrap_or(z.len());
                let stueck: String = z[lauf..bis].iter().collect();
                let breite = g.text_breite_fest(&stueck, sz, font).unwrap_or(0.0) as f64;
                // Vorspann = Breite der Laeufe davor + Abstand ZWISCHEN ihnen
                // (k - 1 Mal) -- genau wie wtext_width(z[..lauf]) es misst.
                let px = if k == 0 { x } else { x + summe as i32 };
                if px > rechts { break; }
                if px + breite as i32 + 1 >= links { self.wtext(g, wdg, px, y, stueck, farbe[lauf]); }
                summe = if k == 0 { breite } else { summe + abstand + breite };
            }
            return;
        }
        let px_bei = |g: &Graphics, k: usize| -> i32 {
            if k == 0 { return x; }
            let vorspann: String = z[..k].iter().collect();
            x + self.wtext_width(g, wdg, &vorspann)
        };
        // Erster Lauf, dessen ENDE rechts von `links` liegt.
        let mut erster = 0usize;
        if x < links {
            let (mut lo, mut hi) = (0usize, anfaenge.len());
            while lo < hi {
                let mitte = (lo + hi) / 2;
                let ende = anfaenge.get(mitte + 1).copied().unwrap_or(z.len());
                if px_bei(g, ende) <= links { lo = mitte + 1; } else { hi = mitte; }
            }
            erster = lo;
        }
        for k in erster..anfaenge.len() {
            let lauf = anfaenge[k];
            let bis = anfaenge.get(k + 1).copied().unwrap_or(z.len());
            let c = farbe[lauf];
            let st = stil(lauf);
            let px = px_bei(g, lauf);
            if px > rechts { break; }
            let stueck: String = z[lauf..bis].iter().collect();
            if st == 0 { self.wtext(g, wdg, px, y, stueck, c); }
            else {
                let (sz, basis) = (self.wsize(g, wdg), font_wahl(wdg.font, g.active_font()));
                g.text_nachgebildet(px, y, stueck, c, basis, sz, st);
            }
        }
    }

    pub(super) fn draw_colorpicker(&self, g: &mut Graphics, wdg: &Widget,
                        ax: i32, ay: i32, w: i32, h: i32) {
        let (feld, streifen, deck) = self.cp_geom(wdg, ax, ay, w, h);
        let (fx, fy, fw, fh) = feld;
        let voll = crate::farbraum::hsv_zu_rgb(wdg.hsv[0], 1.0, 1.0);
        // Saettigung waagerecht (weiss -> voller Ton), Helligkeit senkrecht
        // (durchsichtig -> schwarz). Zwei Verlaeufe uebereinander statt Pixel
        // fuer Pixel -- letzteres waere je Bild ein Rechteck pro Punkt.
        g.gradient_rect(fx, fy, fx + fw - 1, fy + fh - 1, 0xFFFFFF, voll, false);
        // Alpha steht im obersten Byte, und 0 bedeutet DECKEND (nicht
        // durchsichtig -- so bleiben die alten 24-bit-Farben deckend). Fuer
        // "fast unsichtbar" ist darum 1 der kleinste Wert, nicht 0. Mit
        // 0x000000 als Startfarbe war das Feld schlicht schwarz.
        g.gradient_rect(fx, fy, fx + fw - 1, fy + fh - 1,
                        0x01_000000, 0xFF_000000, true);
        // Ton-Streifen: sechs Abschnitte durch den Regenbogen.
        let (sx, sy, sw, sh) = streifen;
        let ecken: [i64; 7] = [0xFF0000, 0xFFFF00, 0x00FF00, 0x00FFFF,
                               0x0000FF, 0xFF00FF, 0xFF0000];
        for i in 0..6 {
            let y0 = sy + sh * i as i32 / 6;
            let y1 = sy + sh * (i as i32 + 1) / 6 - 1;
            if y1 >= y0 { g.gradient_rect(sx, y0, sx + sw - 1, y1, ecken[i], ecken[i + 1], true); }
        }
        // Zeiger. Im Feld ein Ring in Gegenfarbe, damit er auf hell UND auf
        // dunkel sichtbar bleibt -- eine feste Farbe verschwindet immer
        // irgendwo.
        let px = fx + (wdg.hsv[1] * (fw - 1) as f32).round() as i32;
        let py = fy + ((1.0 - wdg.hsv[2]) * (fh - 1) as f32).round() as i32;
        let ring = if wdg.hsv[2] > 0.6 { 0x000000 } else { 0xFFFFFF };
        g.circle_outline(px, py, self.sk(5), ring);
        g.circle_outline(px, py, self.sk(4), ring);
        let ty = sy + (wdg.hsv[0] / 360.0 * (sh - 1) as f32).round() as i32;
        g.rect(sx - 2, ty - 2, sx + sw + 1, ty + 2, 0xFFFFFF);
        g.rect(sx - 1, ty - 1, sx + sw, ty + 1, 0x000000);
        // Rahmen, und bei Fokus in Akzentfarbe (der Ring liegt aussen, siehe
        // draw_widget -- hier geht es nur um die Kante des Feldes).
        // Deckkraft-Streifen: Schachbrett als Untergrund (sonst sieht man
        // dem Verlauf nicht an, WO er durchsichtig wird), darueber der
        // Verlauf von unsichtbar nach voll.
        if let Some((dx, dy, dw, dh)) = deck {
            let kar = self.sk(5).max(2);
            let mut y = dy;
            let mut reihe = 0;
            while y < dy + dh {
                let mut x = dx;
                let mut spalte = 0;
                while x < dx + dw {
                    let hell = (reihe + spalte) % 2 == 0;
                    g.box_fill(x, y, (x + kar - 1).min(dx + dw - 1), (y + kar - 1).min(dy + dh - 1),
                               if hell { 0xB0B0B0 } else { 0x707070 });
                    x += kar; spalte += 1;
                }
                y += kar; reihe += 1;
            }
            let voll_farbe = crate::farbraum::hsv_zu_rgb(wdg.hsv[0], wdg.hsv[1], wdg.hsv[2]);
            g.gradient_rect(dx, dy, dx + dw - 1, dy + dh - 1,
                            0x01_000000 | voll_farbe, 0xFF_000000 | voll_farbe, true);
            let ay2 = dy + ((wdg.alpha.clamp(1, 255) - 1) as f32 / 254.0 * (dh - 1) as f32).round() as i32;
            g.rect(dx - 2, ay2 - 2, dx + dw + 1, ay2 + 2, 0xFFFFFF);
            g.rect(dx - 1, ay2 - 1, dx + dw, ay2 + 1, 0x000000);
            self.rahmen(g, wdg, dx, dy, dx + dw - 1, dy + dh - 1);
        }
        self.rahmen(g, wdg, fx, fy, fx + fw - 1, fy + fh - 1);
        self.rahmen(g, wdg, sx, sy, sx + sw - 1, sy + sh - 1);
    }

    pub(super) fn rahmen(&self, g: &mut Graphics, wdg: &Widget, x1: i32, y1: i32, x2: i32, y2: i32) {
        g.rect(x1, y1, x2, y2, self.wcol(wdg, "border", "widget_border"));
    }

    /// Das zugeklappte Datumsfeld: das Datum und rechts ein Kalenderblatt.
    pub(super) fn draw_datumsfeld(&self, g: &mut Graphics, wdg: &Widget,
                       ax: i32, ay: i32, w: i32, h: i32) {
        self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
                         self.wcol(wdg, "bg", "widget_bg"),
                         self.wcol(wdg, "border", "widget_border"));
        let schrift = self.wsize(g, wdg);
        let text = crate::kalender::format(wdg.datum[0], wdg.datum[1], wdg.datum[2]);
        self.wtext(g, wdg, ax + self.sk(8), ay + (h - schrift).max(0) / 2, text, self.txt_col(wdg));
        let s = self.sk(12);
        let (bx, by) = (ax + w - s - self.sk(9), ay + (h - s) / 2);
        let c = self.th("muted_fg");
        g.rect(bx, by + 2, bx + s, by + s, c);
        g.line(bx, by + 5, bx + s, by + 5, c);
        g.line(bx + 3, by, bx + 3, by + 3, c);
        g.line(bx + s - 3, by, bx + s - 3, by + 3, c);
    }

    /// Der aufgeklappte Monat, obenauf: dasselbe Gitter wie der Waehler, mit
    /// dem markierten Tag; Wochenbeginn und Grenzen vom Datumsfeld.
    pub(super) fn draw_datum_popup(&self, g: &mut Graphics) {
        let Some((wi, i, zelle)) = self.datum_auf else { return };
        let Some((px, py, pw, ph)) = self.datum_geom() else { return };
        let quelle = &self.windows[wi].widgets[i];
        let mut tw = Self::blank(Kind::DatePicker, 0, 0, pw, ph);
        tw.datum = self.datum_zeigt;
        if !zelle {
            tw.wochenbeginn = quelle.wochenbeginn;
            tw.datum_min = quelle.datum_min;
            tw.datum_max = quelle.datum_max;
        }
        g.box_fill(px + 3, py + 3, px + pw + 2, py + ph + 2, 0x40i64 << 24);
        self.draw_datepicker(g, &tw, px, py, pw, ph);
    }

    pub(super) fn draw_datepicker(&self, g: &mut Graphics, wdg: &Widget,
                       ax: i32, ay: i32, w: i32, h: i32) {
        let (kopf, zw, zh) = self.dp_geom(w, h);
        self.fbox_tief_w(g, wdg.kind, ax, ay, ax + w - 1, ay + h - 1,
                         self.wcol(wdg, "bg", "widget_bg"),
                         self.wcol(wdg, "border", "widget_border"));
        let fg = self.txt_col(wdg);
        let matt = self.th("muted_fg");
        let (j, m, t) = (wdg.datum[0], wdg.datum[1], wdg.datum[2]);
        // Kopf: Pfeile aussen, Monat und Jahr mittig.
        let schrift = self.wsize(g, wdg);
        let ky = ay + (kopf - schrift).max(0) / 2;
        // Aussen die Jahres-, innen die Monatspfeile. Ohne Jahressprung
        // braeuchte man bis 1985 rund fuenfhundert Klicks.
        self.wtext(g, wdg, ax + self.sk(5), ky, "<<".into(), matt);
        self.wtext(g, wdg, ax + self.sk(26), ky, "<".into(), fg);
        self.wtext(g, wdg, ax + w - self.sk(15), ky, ">".into(), fg);
        self.wtext(g, wdg, ax + w - self.sk(32), ky, ">>".into(), matt);
        let titel = std::format!("{} {}", crate::kalender::MONATSNAMEN[(m - 1) as usize], j);
        let tw = self.wtext_width(g, wdg, &titel);
        self.wtext(g, wdg, ax + (w - tw) / 2, ky, titel, fg);
        // Wochentage.
        let wy = ay + kopf;
        for i in 0..7i32 {
            let name = crate::kalender::WOCHENTAGE[((i + wdg.wochenbeginn) % 7) as usize];
            let cx = ax + zw * i;
            let bw = self.wtext_width(g, wdg, name);
            self.wtext(g, wdg, cx + (zw - bw) / 2, wy, name.to_string(), matt);
        }
        // Gitter. Der Erste steht in der Spalte seines Wochentags; davor und
        // danach stehen die Nachbarmonate matt -- ein leeres Feld sieht nach
        // einem Fehler aus.
        let erster = (crate::kalender::wochentag(j, m, 1) - wdg.wochenbeginn).rem_euclid(7);
        let im_monat = crate::kalender::tage_im_monat(j, m);
        let (vj, vm, _) = crate::kalender::plus_monate(j, m, 1, -1);
        let im_vormonat = crate::kalender::tage_im_monat(vj, vm);
        let gy = ay + kopf + self.sk(18);
        for zelle in 0..42i32 {
            let (sp, ze) = (zelle % 7, zelle / 7);
            let cx = ax + zw * sp;
            let cy = gy + zh * ze;
            let nr = zelle - erster + 1;
            let (zahl, aktiv) = if nr < 1 { (im_vormonat + nr, false) }
                                else if nr > im_monat { (nr - im_monat, false) }
                                else { (nr, true) };
            if aktiv && zahl == t {
                g.box_fill(cx + 1, cy, cx + zw - 2, cy + zh - 2,
                           self.wcol(wdg, "accent", "accent"));
            }
            let txt = zahl.to_string();
            let bw = self.wtext_width(g, wdg, &txt);
            // Ausserhalb der Grenzen: noch matter als ein Nachbarmonatstag,
            // damit man den Unterschied sieht (der eine ist erreichbar, der
            // andere gesperrt).
            let gesperrt = aktiv && !Self::dp_erlaubt(wdg, [j, m, zahl]);
            let farbe = if gesperrt { shade(matt, -30) }
                        else if !aktiv { matt }
                        else if zahl == t { self.th("win_bg") }
                        else { fg };
            self.wtext(g, wdg, cx + (zw - bw) / 2, cy + (zh - schrift).max(0) / 2, txt, farbe);
            // Markierter Tag (GUI_DATE_MARK): ein Punkt unter der Zahl.
            if aktiv {
                if let Some(c) = self.dp_marke(wdg, [j, m, zahl]) {
                    let c = if zahl == t { self.th("win_bg") } else { c };
                    g.circle(cx + zw / 2, cy + zh - self.sk(4), self.sk(2).max(2), c);
                }
            }
        }
    }
}
