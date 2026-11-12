//! Ein Bild Eingabe (`update`): Maus, Menues, Treffertests, Texteingabe.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Update (ein Frame) ---
    /// Was GUI_CHANGED vergleicht: der Wert wie fuer die Bindung, dazu
    /// Auswahl, Zahl und Haken (eine Liste, eine Tabelle haben keinen Text).
    pub(super) fn abdruck(&self, w: &Widget) -> String {
        let tab = w.tbl.as_ref().map(|t| t.selected).unwrap_or(-1);
        // Der Baum haelt seine Auswahl im eigenen Zustand -- ohne sie hier
        // meldete GUI_CHANGED bei einem Baum nie, dass ein Knoten gewaehlt wurde.
        let baum = w.tree.as_ref().map(|t| t.selected).unwrap_or(-1);
        // Beim Kippschalter ist `value` die LAGE des Knopfs, die nach einem
        // Wechsel einige Bilder lang nachgleitet -- auch nach GUI_SET_CHECKED
        // aus dem Programm. Sie zaehlte als Bedienung, GUI_CHANGED meldete
        // dann jedes Bild (gefunden am Inspektor des Studios). Der Zustand
        // steht in `checked`.
        let wert = if w.kind == Kind::Toggle { 0.0 } else { w.value };
        format!("{}|{}|{}|{}|{}|{}", self.wert_text(w), w.sel, wert, w.checked, tab, baum)
    }

    /// GUI_CHANGED(wdg): hat die Bedienung das Widget im letzten GUI_UPDATE
    /// geaendert? Der erste Aufruf meldet FALSE und beginnt zu beobachten.
    pub fn changed(&mut self, h: i64) -> Result<bool, String> {
        let w = self.wdg_mut(h, "GUI_CHANGED")?;
        if !w.beobachtet { w.beobachtet = true; return Ok(false); }
        Ok(w.geaendert)
    }

    pub fn update(&mut self, g: &mut Graphics) {
        // GUI_CHANGED: den Stand vor der Bedienung merken
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                if self.windows[wi].widgets[i].beobachtet {
                    let a = self.abdruck(&self.windows[wi].widgets[i]);
                    let w = &mut self.windows[wi].widgets[i];
                    w.stand_vorher = a;
                    w.geaendert = false;
                }
            }
        }
        // Weg E: erst abgleichen, was an den nativen Fenstern geschah, dann
        // durch die Oberflaeche unter dem Zeiger schauen.
        self.nativ_abgleich(g);
        self.flaeche = g.zeiger_flaeche();
        g.sicht_setzen(Some(self.flaeche));
        self.update_innen(g);
        g.sicht_setzen(None);
        self.flaeche = 0;
        // Das Dock rechnet NACH der Bedienung: ein Klick auf einen Reiter
        // oder ein Zug an der Trennlinie steht so schon in GUI_DOCK_RECT und
        // GUI_DOCK_CHANGED, wenn das Programm nach GUI_UPDATE fragt.
        self.dock_pass(g);
        self.stil_pass();
        self.rs_pass(g.delta());
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                if self.windows[wi].widgets[i].beobachtet {
                    let a = self.abdruck(&self.windows[wi].widgets[i]);
                    let w = &mut self.windows[wi].widgets[i];
                    w.geaendert = a != w.stand_vorher;
                }
            }
        }
    }

    pub(super) fn update_innen(&mut self, g: &mut Graphics) {
        let mx = g.mouse_x() as i32;
        let my = g.mouse_y() as i32;
        let is_down = g.mouse_button(0);
        let just_pressed = is_down && !self.was_mouse_down;
        let just_released = !is_down && self.was_mouse_down;
        let right_down = g.mouse_button(1);
        let right_just = right_down && !self.was_right_down;
        // Mittlere Taste: nur die Reiter benutzen sie (Reiter zu).
        let mitte_just = g.mouse_button(2) && !self.was_mitte_down;
        self.was_mitte_down = g.mouse_button(2);
        // Weg E: ein Druck auf einer anderen Oberflaeche schliesst die
        // offenen Popups der Fenster, die nicht auf ihr stehen.
        if just_pressed || right_just { self.fremde_popups_zu(); }

        // Transiente Flags ruecksetzen -- die Dialog-Antwort gehoert dazu:
        // sie ist ein Ereignis, kein Zustand (siehe Window::answer).
        for win in self.windows.iter_mut() {
            win.answer = 0;
            win.tab_zu = -1;
            if let Some(d) = win.dock.as_deref_mut() { d.geaendert = false; d.geschlossen.clear(); }
            for wdg in win.widgets.iter_mut() {
                wdg.clicked = false; wdg.hovered = false; wdg.entered = false; wdg.abgelegt = false;
                if let Some(l) = wdg.leiste.as_mut() { l.hover = -1; l.geklickt = -1; }
                if let Some(l) = wdg.list.as_mut() { l.hover = -1; l.bearbeitet = None; }
                wdg.zeile_jetzt = -1;
                if let Some(s) = wdg.status.as_mut() { s.hover = -1; s.geklickt = -1; }
                if let Some(p) = wdg.pfad.as_mut() { p.hover = -1; p.geklickt = -1; }
                if let Some(tc) = wdg.tabctl.as_mut() { tc.zu = None; }
                wdg.farbfeld_klick = -1; wdg.hinweis_klick = 0; wdg.rand_klick = (0, -1); wdg.abk_treffer = -1; wdg.tab_treffer = false;
                if let Some(l) = wdg.list.as_mut() { l.doppel = false; }
                if let Some(t) = wdg.tbl.as_mut() { t.hover_row = -1; t.clicked_row = -1; t.doppel = false; }
                if let Some(t) = wdg.tree.as_mut() {
                    t.hover = -1; t.doppel = false; t.bearbeitet = None;
                    if let Some(d) = t.datei.as_mut() { d.aktiviert.clear(); }
                }
                if let Some(r) = wdg.rich.as_mut() { r.geklickt.clear(); r.code_aktion.clear(); }
                if let Some(a) = wdg.akk.as_mut() { a.umgeschaltet = -1; }
                if let Some(z) = wdg.wz.as_mut() { z.ereignis = 0; }
            }
            for m in win.menus.iter_mut() {
                for it in m.items.iter_mut() { it.clicked = false; }
            }
        }
        self.drop = None;
        self.liste_taste = false;
        self.schirm_h = g.screen_height() as i32;
        self.schirm_b = g.screen_width() as i32;
        // Popover, Toasts, Infoleisten, Suchfelder, Vorschlaege (gui_ueber.rs):
        // erst legen, dann bekommen sie den Druck -- was dort landet oder
        // einen Popover schliesst, erreicht kein Widget mehr.
        self.ueber_zuruecksetzen();
        self.ueber_pass(g, mx, my);
        let ueber_weg = (just_pressed || right_just) && self.ueber_druck(mx, my, just_pressed);
        // Das Rechtschreib-Menue (gui_rechtschreibung.rs): ein Rechtsklick auf
        // ein falsches Wort geht dem Kontextmenue des Programms vor.
        let ueber_weg = ueber_weg || self.rs_druck(g, mx, my, just_pressed, right_just);
        let just_pressed = just_pressed && !ueber_weg;
        let right_just = right_just && !ueber_weg;
        if just_pressed { self.fluss_druck(mx, my); }
        self.schriftwahl_pass(g);
        // Rad ueber der offenen Klappliste rollt SIE -- vor allem anderen, denn
        // sie liegt ueber allem, und das Rad gibt es nur einmal.
        if let Some((dw, di)) = self.open_dropdown {
            if dw < self.windows.len() && di < self.windows[dw].widgets.len() {
                let (px, py, pw, ph) = self.dropdown_popup_rect(dw, di);
                if Self::in_rect(mx, my, (px, py, pw, ph)) {
                    let rad = Self::rad(g);
                    if rad != 0 {
                        let n = self.windows[dw].widgets[di].items.len() as i32;
                        let rows = self.dd_zeilen(dw, di);
                        self.dd_scroll = (self.dd_scroll - rad as i32).clamp(0, (n - rows).max(0));
                    }
                }
            }
        }
        self.dateibaeume_auffrischen(g.get_time());
        self.schnitte_laden(g);
        self.rt_pass(g);
        self.umbruch_layout(g);
        self.akk_pass(g.delta());
        self.layout_pass(g);
        self.panel_pass();
        self.tabctl_pass();
        self.leisten_pass(g);
        // Menue-Eingabe (Menueleiste/Dropdown/Kontext) VOR den Widgets -- konsumiert
        // den Klick ggf., damit er nicht zusaetzlich ein Widget ausloest.
        // Waehrend eines modalen Dialogs gelten nur SEINE Menues (bis 2026.47
        // waren alle gesperrt, auch das Kontextmenue im Dialog selbst --
        // gefunden beim Bau der echten Fenster, Weg E Stufe 2).
        self.kontext_neu = None;
        let menu_consumed = self.menu_input(mx, my, just_pressed, right_just, g);
        self.untermenues_folgen(g, mx, my);
        self.kuerzel_gefeuert = false;
        if !menu_consumed { self.kuerzel_pruefen(g); self.mn_pruefen(g); }

        // Inhalts-Scroll: Mausrad ueber scrollbarem Fenster.
        if let Some(top) = self.topmost_at(mx, my) {
            if self.windows[top].scrollable {
                let wheel = Self::rad(g);
                if wheel != 0 {
                    let ms = self.max_scroll_y(top);
                    self.windows[top].scroll_y = (self.windows[top].scroll_y - wheel as i32 * 40).clamp(0, ms);
                }
            }
        }
        // Scroll-Offsets klemmen (Inhalt kann durch Resize/Aenderung schrumpfen).
        for wi in 0..self.windows.len() {
            if self.windows[wi].scrollable {
                let ms = self.max_scroll_y(wi);
                self.windows[wi].scroll_y = self.windows[wi].scroll_y.clamp(0, ms);
            }
        }
        // Laufendes Scrollbalken-Drag.
        if let Some(wi) = self.scroll_drag {
            if is_down {
                if let Some((_tx, ty, _bw, th, _thy, thh)) = self.winscroll_geom(wi) {
                    let ms = self.max_scroll_y(wi);
                    let rel = ((my - ty - thh / 2) as f32 / (th - thh).max(1) as f32).clamp(0.0, 1.0);
                    self.windows[wi].scroll_y = (rel * ms as f32) as i32;
                }
            } else { self.scroll_drag = None; }
        }
        // Klick auf den Scrollbalken startet das Drag (konsumiert den Klick).
        let mut scroll_consumed = false;
        if just_pressed && !menu_consumed && self.scroll_drag.is_none() {
            if let Some(top) = self.topmost_at(mx, my) {
                if let Some((tx, ty, bw, th, _thy, _thh)) = self.winscroll_geom(top) {
                    if mx >= tx && mx < tx + bw && my >= ty && my < ty + th {
                        self.scroll_drag = Some(top);
                        scroll_consumed = true;
                    }
                }
            }
        }
        // Mittlere Taste auf einem Reiter: zu. So kennt man es aus jedem
        // Browser, und es ist der schnellste Weg, wenn das Kreuz klein ist.
        if mitte_just {
            for &wi in self.z_order.clone().iter().rev() {
                if !self.windows[wi].alive || !self.windows[wi].visible { continue; }
                if !self.windows[wi].tabs_closable || self.tabbar_h(wi) == 0 { continue; }
                let by = self.windows[wi].y
                    + (if self.windows[wi].chrome { self.m("title_h") } else { 0 })
                    + self.menubar_h(wi);
                if my < by || my >= by + TABBAR_H { continue; }
                if let Some((ti, _, _)) = self.tab_slots(g, wi).into_iter().find(|(_, x0, x1)| mx >= *x0 && mx < *x1) {
                    self.windows[wi].tab_zu = ti as i32;
                    break;
                }
            }
        }
        // Mittlere Taste auf dem Kopf eines schliessbaren Reiterwerks: zu.
        if mitte_just {
            if let Some(top) = self.topmost_at(mx, my) {
                let kopf = self.sk(TC_KOPF_H);
                let mut treffer = None;
                for (i, w) in self.windows[top].widgets.iter().enumerate() {
                    if !Self::tc_schliessbar(w) || !self.widget_shown(top, w) { continue; }
                    let (ax, ay, bw, _) = self.abs_rect(top, w);
                    if my < ay || my >= ay + kopf { continue; }
                    if let Some((k, _, _)) = self.tc_koepfe(ax, bw, w).into_iter().find(|&(_, x0, x1)| mx >= x0 && mx < x1) {
                        treffer = Some((i, k));
                    }
                }
                if let Some((i, k)) = treffer {
                    if let Some(st) = self.windows[top].widgets[i].tabctl.as_mut() { st.zu = Some(k); }
                }
            }
        }
        // Klick auf einen Reiter (Tab) wechselt die Seite (konsumiert den Klick).
        let mut tab_consumed = false;
        if just_pressed && !menu_consumed && !scroll_consumed {
            for &wi in self.z_order.clone().iter().rev() {
                if !self.windows[wi].alive || !self.windows[wi].visible || self.tabbar_h(wi) == 0 { continue; }
                let by = self.windows[wi].y
                    + (if self.windows[wi].chrome { self.m("title_h") } else { 0 })
                    + self.menubar_h(wi);
                if my < by || my >= by + TABBAR_H { continue; }
                if let Some((ti, _, x1)) = self.tab_slots(g, wi).into_iter().find(|(_, x0, x1)| mx >= *x0 && mx < *x1) {
                    // Das Kreuz schliesst, statt umzuschalten -- und die
                    // MITTLERE Taste tut es von ueberall auf dem Reiter aus
                    // (so kennt man es aus jedem Browser).
                    let (kx, ky, k) = self.tab_kreuz_rect(wi, x1, by);
                    let aufs_kreuz = self.windows[wi].tabs_closable
                        && mx >= kx - 3 && mx < kx + k + 3 && my >= ky - 3 && my < ky + k + 3;
                    if aufs_kreuz {
                        self.windows[wi].tab_zu = ti as i32;
                    } else {
                        self.windows[wi].active_tab = ti as i32;
                    }
                    tab_consumed = true;
                    break;
                }
            }
        }
        // Hover (nur oberstes Fenster); Tabellen aktualisieren Scroll/Hover/Wheel.
        if let Some(top) = self.topmost_at(mx, my).filter(|&t| !self.windows[t].entwurf) {
            let n = self.windows[top].widgets.len();
            for i in 0..n {
                let (r, kind, active) = {
                    let w = &self.windows[top].widgets[i];
                    (self.abs_rect(top, w), w.kind, self.widget_shown(top, w) && w.enabled && w.kind != Kind::Layout
                        && self.im_panel_sichtbar(top, w, mx, my))
                };
                if active && Self::in_rect(mx, my, r) {
                    self.windows[top].widgets[i].hovered = true;
                    if kind == Kind::Table {
                        self.table_hover(top, i, mx, my, g);
                        if right_just { self.tabelle_rechtsklick(top, i); }
                    }
                    if kind == Kind::ListBox {
                        self.listbox_wheel(top, i, r.3, g);
                        // Ueber dem Rollbalken liegt keine Zeile.
                        let k = if self.list_bar_at(top, i, mx, my) { -1 } else { self.liste_zeile(top, i, my) };
                        if let Some(l) = self.windows[top].widgets[i].list.as_mut() { l.hover = k; }
                        self.windows[top].widgets[i].zeile_jetzt = k;
                        let ganz = if k >= 0 {
                            let platz = self.liste_textplatz(g, top, i, k as usize);
                            let wdg = &self.windows[top].widgets[i];
                            let it = wdg.items.get(k as usize).cloned().unwrap_or_default();
                            if self.mit_auslassung(g, wdg, &it, platz).is_some() { it } else { String::new() }
                        } else { String::new() };
                        self.windows[top].widgets[i].ganzer_text = ganz;
                        if right_just { self.liste_rechtsklick(top, i, k); }
                    }
                    if kind == Kind::RichText { self.richtext_wheel(top, i, r.3, g); }
                    let eigenes_menue = self.kontext_neu == Some((top, i));
                    if kind == Kind::TextInput && right_just && (!menu_consumed || eigenes_menue) {
                        self.ti_rechtsklick(g, top, i, mx);
                    }
                    if kind == Kind::TextArea {
                        self.textarea_wheel(top, i, g);
                        // Rollbalken: greifen (auf dem Griff) oder hinspringen
                        // (in der Rinne) -- vor dem Editieren, das denselben
                        // Druck sonst als Klick in den Text naehme.
                        if just_pressed && !menu_consumed {
                            if let Some((tx, ty, bw, th, thy, thh, n, sicht)) = self.ta_bar_geom(g, top, i) {
                                if Self::in_rect(mx, my, (tx - 2, ty, bw + 3, th)) {
                                    let griff = if my >= thy && my < thy + thh { my - thy } else {
                                        let max = (n - sicht).max(0);
                                        let rel = ((my - ty - thh / 2) as f64 / (th - thh).max(1) as f64).clamp(0.0, 1.0);
                                        let w = &mut self.windows[top].widgets[i];
                                        w.scroll = (rel * max as f64).round() as i32;
                                        w.rad_ziel = None;
                                        w.rad_stand = Some((w.caret, w.sel_anchor, w.text.chars().count()));
                                        thh / 2
                                    };
                                    self.tabar_zug = Some((top, i, griff));
                                    self.windows[top].widgets[i].farbfeld_zug = true;
                                }
                            }
                        }
                        if right_just && (!menu_consumed || eigenes_menue) && self.tabar_zug.is_none() {
                            self.ta_rechtsklick(g, top, i, mx, my);
                        }
                        if !menu_consumed && !tab_consumed && !scroll_consumed
                           && (just_pressed || right_just) {
                            let taste = if just_pressed { 0 } else { 1 };
                            if let Some(z) = self.ta_rand_zeile(g, top, i, mx, my) {
                                self.windows[top].widgets[i].rand_klick = (z, taste);
                            }
                        }
                    }
                    if kind == Kind::Spinner { self.spinner_wheel(top, i, g); }
                    if kind == Kind::Tree {
                        self.tree_hover(top, i, my, g);
                        if self.list_bar_at(top, i, mx, my) {
                            if let Some(t) = self.windows[top].widgets[i].tree.as_mut() { t.hover = -1; }
                        }
                        if right_just { self.baum_rechtsklick(top, i); }
                    }
                    if kind == Kind::StatusBar {
                        let k = self.sb_treffer(top, i, mx);
                        if let Some(s) = self.windows[top].widgets[i].status.as_mut() { s.hover = k; }
                    }
                    if kind == Kind::Breadcrumb {
                        let k = self.pf_treffer(top, i, mx);
                        if let Some(p) = self.windows[top].widgets[i].pfad.as_mut() { p.hover = k; }
                    }
                    if kind == Kind::Toolbar {
                        let k = self.tb_treffer(top, i, mx, my);
                        if let Some(l) = self.windows[top].widgets[i].leiste.as_mut() { l.hover = k; }
                    }
                }
            }
            // Mausrad ueber einem rollenden Panel -- NACH den Kindern, damit
            // eine Liste darin das Rad zuerst bekommt (pop_mouse_wheel
            // liefert es nur einmal).
            for i in 0..n {
                let (r, rollt) = {
                    let w = &self.windows[top].widgets[i];
                    (self.abs_rect(top, w), w.panel.is_some() && self.widget_shown(top, w))
                };
                if rollt && Self::in_rect(mx, my, r) {
                    let wheel = Self::rad(g);
                    if wheel != 0 { self.panel_scroll_um(top, i, -(wheel as i32) * 40); }
                }
                let akk = { let w = &self.windows[top].widgets[i]; w.akk.is_some() && self.widget_shown(top, w) };
                if akk && Self::in_rect(mx, my, r) {
                    let wheel = Self::rad(g);
                    if wheel != 0 {
                        let schritt = self.sk(40);
                        let (hh, a) = { let w = &mut self.windows[top].widgets[i]; (w.h, w.akk.as_mut().unwrap()) };
                        a.scroll = (a.scroll - wheel as i32 * schritt).clamp(0, (a.inhalt_h - hh).max(0));
                    }
                }
            }
        }
        // --- Flanken: betreten/verlassen, Fokus bekommen/verlieren ---------
        //
        // Erst HIER, weil `hovered` fuer dieses Bild feststeht und der Fokus
        // gesetzt ist. Verglichen wird gegen den Zustand des VORIGEN Bildes --
        // `hovered` allein wuerde jedes Bild feuern, solange die Maus
        // stehenbleibt. Die Handler wandern wie ueberall in `pending` und
        // werden nach `update()` von der VM aufgerufen, damit ein Handler die
        // GUI nicht mitten im Zustands-Update umbauen kann.
        let fokus = self.focus_widget;
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                let w = &mut self.windows[wi].widgets[i];
                let ist_fokus = fokus == Some((wi, i));
                let (h_jetzt, h_vorher) = (w.hovered, w.was_hovered);
                let (f_jetzt, f_vorher) = (ist_fokus, w.was_focused);
                w.was_hovered = h_jetzt;
                w.was_focused = f_jetzt;
                let mut feuern: Vec<Rueckruf> = Vec::new();
                if h_jetzt && !h_vorher {
                    if let Some(f) = w.on_hover.clone() { feuern.push(f); }
                } else if !h_jetzt && h_vorher {
                    if let Some(f) = w.on_leave.clone() { feuern.push(f); }
                }
                if f_jetzt && !f_vorher {
                    if let Some(f) = w.on_focus.clone() { feuern.push(f); }
                } else if !f_jetzt && f_vorher {
                    if let Some(f) = w.on_blur.clone() { feuern.push(f); }
                }
                let verlassen = !f_jetzt && f_vorher && !w.regeln.is_empty();
                self.pending.extend(feuern);
                if verlassen && self.windows[wi].pruefung_live { self.widget_pruefen(wi, i); }
            }
        }

        self.cursor_pass(g, mx, my);
        // Tooltip-Dwell: oberstes Widget mit Hilfetext unter der Maus bestimmen.
        // Bei Wechsel/Bewegung/gedrueckter Maus startet der Verweil-Timer neu.
        // Liegt ein modales Fenster obenauf, hat alles dahinter keinen Tipp:
        // es nimmt keine Klicks an, und ein Tooltip ueber dem Schleier
        // sah aus, als liesse es sich noch bedienen.
        let modal = self.modal;
        let tip_cur = self.topmost_at(mx, my).filter(|&top| modal.map_or(true, |m| m == top)).and_then(|top| {
            let mut found = None;
            for i in 0..self.windows[top].widgets.len() {
                let w = &self.windows[top].widgets[i];
                let leisten_tip = w.leiste.as_ref()
                    .and_then(|l| l.eintraege.get(l.hover.max(0) as usize).filter(|_| l.hover >= 0))
                    .map(|e| !e.tip.is_empty()).unwrap_or(false);
                let listen_tip = w.list.as_ref()
                    .and_then(|l| l.tips.get(l.hover.max(0) as usize).filter(|_| l.hover >= 0))
                    .map(|t| !t.is_empty()).unwrap_or(false);
                let status_tip = w.status.as_ref()
                    .and_then(|s| s.felder.get(s.hover.max(0) as usize).filter(|_| s.hover >= 0))
                    .map(|f| !f.tip.is_empty()).unwrap_or(false);
                let pfad_tip = w.pfad.as_ref().map(|p| p.hover == -2).unwrap_or(false);
                let ganz_tip = w.hovered && !w.ganzer_text.is_empty();
                if w.hovered && (!w.tooltip.is_empty() || !w.fehler.is_empty() || leisten_tip || listen_tip || status_tip || pfad_tip || ganz_tip) { found = Some((top, i)); }
            }
            found
        });
        // Innerhalb einer Werkzeugleiste zaehlt ein anderer KNOPF als ein
        // anderes Widget -- sonst stuende der Tipp des vorigen Knopfes weiter
        // da, sobald man einmal verweilt hat.
        let teil = tip_cur.and_then(|(wi, i)| {
            let w = &self.windows[wi].widgets[i];
            w.leiste.as_ref().map(|l| l.hover).or_else(|| w.list.as_ref().map(|l| l.hover))
                .or_else(|| w.status.as_ref().map(|s| s.hover)).or_else(|| w.pfad.as_ref().map(|p| p.hover))
                // Eine schlichte Liste hat keinen ListState -- ihre Zeile
                // unter der Maus zaehlt trotzdem als eigener Teil.
                .or_else(|| (w.kind == Kind::ListBox).then_some(w.zeile_jetzt))
        }).unwrap_or(-1);
        if teil != self.hover_teil {
            self.hover_teil = teil;
            self.hover_frame = self.frame_count;
        }
        if mx != self.hover_x || my != self.hover_y || is_down {
            // Die Maus uebernimmt: der Tastatur-Tooltip und der
            // Tastatur-Menuecursor treten zurueck.
            self.tip_fokus = None;
            self.menu_cursor = None;
        }
        if tip_cur != self.hover_w || mx != self.hover_x || my != self.hover_y || is_down {
            self.hover_w = tip_cur;
            self.hover_frame = self.frame_count;
        }
        self.hover_x = mx;
        self.hover_y = my;
        // Laufendes Fenster-Drag.
        if let Some(wi) = self.drag_window {
            if is_down {
                self.windows[wi].x = mx - self.drag_dx;
                self.windows[wi].y = my - self.drag_dy;
            } else { self.drag_window = None; }
        }
        // Laufender Fenster-Resize (unten-rechts).
        if let Some(wi) = self.resize_window {
            if is_down {
                let w = &mut self.windows[wi];
                let mut nw = (mx - w.x + self.resize_dx).max(80);
                let mut nh = (my - w.y + self.resize_dy).max(50);
                if w.min_w > 0 { nw = nw.max(w.min_w); }
                if w.min_h > 0 { nh = nh.max(w.min_h); }
                if w.max_w > 0 { nw = nw.min(w.max_w); }
                if w.max_h > 0 { nh = nh.min(w.max_h); }
                let changed = w.w != nw || w.h != nh;
                w.w = nw; w.h = nh;
                if changed { self.relayout(wi); }       // Anchoring nachziehen
            } else { self.resize_window = None; }
        }
        // Laufendes Slider-Drag.
        if let Some((wi, i)) = self.active_slider {
            if is_down { self.drag_slider(wi, i, mx, my); } else { self.active_slider = None; }
        }
        // Ziehen: ab 5 px Bewegung wird aus dem Druck ein Zug; beim
        // Loslassen ueber einer Ablage wird abgelegt (ein Bild lang
        // abfragbar), sonst verpufft es still.
        let zieh_ende = match self.drag.as_mut() {
            Some(d) if is_down => {
                if !d.aktiv && ((mx - d.x0).abs() >= 5 || (my - d.y0).abs() >= 5) { d.aktiv = true; }
                false
            }
            Some(_) => true,
            None => false,
        };
        if zieh_ende {
            if let Some(d) = self.drag.take() {
                if d.aktiv { self.ablegen(d, mx, my); }
            }
        }
        // Laufendes Drehregler-Ziehen: senkrecht, ausgehend vom Wert beim
        // Anfassen. 140 Pixel entsprechen dem ganzen Bereich -- weit genug
        // fuer feines Einstellen, kurz genug zum schnellen Durchfahren.
        if let Some((wi, i, y0, v0)) = self.active_knob {
            if is_down {
                let w = &mut self.windows[wi].widgets[i];
                let spanne = w.max - w.min;
                let neu = (v0 + (y0 - my) as f64 / 140.0 * spanne).clamp(w.min, w.max);
                if (neu - w.value).abs() > f64::EPSILON {
                    w.value = neu;
                    let och = w.on_change.clone();
                    if let Some(f) = och { self.pending.push(f); }
                }
            } else {
                self.active_knob = None;
            }
        }
        self.uebergaenge(g);
        self.dock_zug_pass(is_down, mx, my);
        // Rollbalken einer Liste ziehen -- auch wenn die Maus die Rinne
        // verlaesst, wie bei jedem Rollbalken.
        if let Some((wi, i, griff)) = self.listbar_zug {
            if is_down {
                if let Some((_, ty, _, th, _, thh)) = self.list_bar_geom(wi, i) {
                    let max = self.list_max_scroll(wi, i);
                    let rel = ((my - griff - ty) as f64 / (th - thh).max(1) as f64).clamp(0.0, 1.0);
                    let w = &mut self.windows[wi].widgets[i];
                    Self::roll_setze(w, (rel * max as f64).round());
                    w.rad_ziel = None;
                }
            } else { self.listbar_zug = None; }
        }
        if let Some((wi, i, griff)) = self.tabar_zug {
            if is_down {
                if let Some((_, ty, _, th, _, thh, n, sicht)) = self.ta_bar_geom(g, wi, i) {
                    let max = (n - sicht).max(0);
                    let rel = ((my - griff - ty) as f64 / (th - thh).max(1) as f64).clamp(0.0, 1.0);
                    let w = &mut self.windows[wi].widgets[i];
                    w.scroll = (rel * max as f64).round() as i32;
                    w.rad_ziel = None;
                    // Die Schreibmarke zieht die Ansicht sonst zu sich zurueck.
                    w.rad_stand = Some((w.caret, w.sel_anchor, w.text.chars().count()));
                }
            } else { self.tabar_zug = None; }
        }
        // Laufendes Splitter-Drag.
        if let Some((wi, i)) = self.active_split {
            if is_down { self.drag_split(wi, i, mx, my); } else { self.active_split = None; }
        }
        // Laufendes Tabellen-Scrollbar-Drag.
        if let Some((wi, i)) = self.active_table {
            if is_down {
                self.table_drag(wi, i, mx, my);
            } else {
                if let Some(t) = self.windows[wi].widgets[i].tbl.as_mut() {
                    t.drag_v = false; t.drag_h = false;
                    t.drag_col = -1;      // Spaltenbreiten-Ziehen endet hier
                    // Kopf-Geste auswerten: ohne Bewegung war es ein
                    // Sortier-Klick, mit Bewegung wurde schon verschoben.
                    if let Some((pos, _, bewegt)) = t.head_drag.take() {
                        if !bewegt && t.sortable {
                            let o = t.order();
                            if let Some(&c) = o.get(pos) {
                                let c = c as i32;
                                if t.sort_col == c { t.sort_desc = !t.sort_desc; }
                                else { t.sort_col = c; t.sort_desc = false; }
                                t.rebuild_view();
                            }
                        }
                    }
                }
                self.active_table = None;
            }
        }
        // Neuer Druck (entfaellt, wenn ein Menue den Klick verarbeitet hat).
        if just_pressed && !menu_consumed && !scroll_consumed && !tab_consumed && self.drag_window.is_none() && self.resize_window.is_none()
            && self.active_slider.is_none() && self.active_table.is_none() && self.active_split.is_none() {
            // Doppelklick: zweiter Druck innerhalb DBL_TIME an fast derselben
            // Stelle. Die Ortspruefung muss sein -- sonst gaelten zwei rasche
            // Klicks auf verschiedene Zeilen als Doppelklick.
            let jetzt = g.get_time();
            self.dbl_click = match self.last_click {
                Some((t0, x0, y0)) =>
                    jetzt - t0 <= DBL_TIME && (mx - x0).abs() <= self.sk(DBL_DIST) && (my - y0).abs() <= self.sk(DBL_DIST),
                None => false,
            };
            // Nach einem erkannten Doppelklick von vorn zaehlen, sonst waere
            // ein dritter Klick gleich wieder einer.
            self.last_click = if self.dbl_click { None } else { Some((jetzt, mx, my)) };
            self.tasten_mod = (g.key_ctrl(), g.key_shift());
            self.handle_press(mx, my);
            self.dbl_click = false;
        }
        self.rt_auswahl_pass(g, is_down, mx, my);
        self.label_pass(g, mx, my, just_pressed, is_down);
        self.tabellen_pass();
        // Farbwaehler weiterziehen, auch ausserhalb seiner Flaeche.
        if let Some((cw, ci, _)) = self.cp_drag {
            if is_down { self.cp_zieh(cw, ci, mx, my); } else { self.cp_drag = None; }
        }
        // Loslassen -> Button-Klick bestaetigen.
        if just_released {
            if let Some((wi, i)) = self.press_origin.filter(|&(wi, i)| self.windows[wi].widgets[i].kind == Kind::Toolbar) {
                // Geklickt ist, wenn die Taste auf DEMSELBEN Knopf wieder
                // hochgeht -- wie bei jedem Knopf laesst sich ein Druck durch
                // Wegziehen zuruecknehmen.
                let k = self.tb_treffer(wi, i, mx, my);
                let d = self.windows[wi].widgets[i].leiste.as_ref().map(|l| l.gedrueckt).unwrap_or(-1);
                if let Some(l) = self.windows[wi].widgets[i].leiste.as_mut() { l.gedrueckt = -1; }
                if k >= 0 && k == d { self.tb_klick(wi, i, k); }
                self.press_origin = None;
            }
            if let Some((wi, i)) = self.press_origin {
                let r = { let w = &self.windows[wi].widgets[i]; self.abs_rect(wi, w) };
                let w = &mut self.windows[wi].widgets[i];
                let fire = if matches!(w.kind, Kind::Button | Kind::Card) && Self::in_rect(mx, my, r) {
                    w.clicked = true; w.on_click.clone()
                } else { None };
                if let Some(f) = fire { self.pending.push(f); }
            }
            self.press_origin = None;
            // Tabellen-Zeilen-Klick bestaetigen (Selektion + on_change).
            let (ctrl, shift) = (g.key_ctrl(), g.key_shift());
            if let Some((wi, i, row)) = self.table_press.take() {
                let hr = self.windows[wi].widgets[i].tbl.as_ref().map(|t| t.hover_row).unwrap_or(-1);
                if row >= 0 && hr == row {
                    let w = &mut self.windows[wi].widgets[i];
                    let mut haken = false;
                    if let Some(t) = w.tbl.as_mut() {
                        // Strg schaltet eine Zeile um, Umschalt waehlt den
                        // Bereich vom Anker -- sonst ersetzt der Klick die
                        // Auswahl. Genau so kennt man es von jeder Liste.
                        if t.multi && ctrl { t.auswahl_umschalten(row); }
                        else if t.multi && shift { t.auswahl_bereich(row); }
                        else { t.auswahl_setzen(row); }
                        t.clicked_row = row;
                        t.clicked_col = t.hover_col;
                        // Eine Hakenzelle schaltet beim Klick selbst um --
                        // sonst muesste jedes Programm das nachbauen, und die
                        // Zelle saehe bedienbar aus, ohne es zu sein.
                        let (r, c) = (row as usize, t.hover_col);
                        if c >= 0 {
                            if let Some(cell) = t.cell(r, c as usize) {
                                if cell.kind == CellKind::Check { haken = true; }
                            }
                        }
                        if haken {
                            let cell = t.cell_mut(r, t.hover_col as usize).unwrap();
                            cell.value = if cell.value >= 0.5 { 0.0 } else { 1.0 };
                        }
                    }
                    let f = w.on_change.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
        }
        // Menue per Tastatur (Alt / F10, Pfeile, Enter, ESC). Solange ein
        // Leistenmenue offen ist, gehoeren die Tasten dem Menue -- sonst
        // tippte man in ein Textfeld, waehrend das Menue aufsteht.
        let menue_hat_tasten = self.menue_tasten(g) || self.rs_tasten(g);
        if !menue_hat_tasten {
        // Ein Drehfeld, in das getippt wurde, gibt den Fokus ab: der Text gilt.
        if let Some((ewi, ei, ..)) = self.spin_edit {
            if self.focus_widget != Some((ewi, ei)) { self.spin_fertig(); }
        }
        // Tastatur + Maus fuer das fokussierte Textfeld (Caret/Selektion/Editieren).
        if let Some((wi, i)) = self.focus_widget {
            let frei_dd = self.windows[wi].widgets[i].frei;
            match self.windows[wi].widgets[i].kind {
                Kind::TextInput => if !self.feld_tasten(wi, i, g) { self.edit_textinput(wi, i, g) },
                Kind::Dropdown if frei_dd => self.edit_combo(wi, i, g),
                Kind::TextArea => self.edit_textarea(wi, i, g),
                Kind::Spinner => self.spinner_keys(wi, i, g),
                Kind::Table => self.table_keys(wi, i, g),
                k => self.widget_keys(wi, i, k, g),
            }
        }
        // Geteilte Textbereiche gleich nach dem Tippen abgleichen -- wer nach
        // GUI_UPDATE den Text des Besitzers liest (Sichern!), bekommt so den
        // Stand dieses Bildes, nicht den des vorigen.
        self.teil_pass(g.get_time());
        // Tastatur-Navigation: Tab / Shift+Tab wechselt den Fokus zwischen ALLEN
        // bedienbaren Widgets des aktiven Fensters (Anlege-Reihenfolge, nur
        // sichtbare + eingeschaltete). Frueher liefen nur die Textfelder mit --
        // Knopf, Kaestchen und Klappliste waren ohne Maus nicht erreichbar.
        // Ein Code-Feld verlangt den Tabulator fuer sich (siehe
        // edit_textarea) -- dann darf er hier NICHT zusaetzlich den Fokus
        // weiterschalten, sonst tut eine Taste zwei Dinge. Das gilt auch,
        // wenn es ihn nur MELDET (`tab_meldet`): gemeldet UND den Fokus
        // weiter waere derselbe Fehler.
        // Ein Gitter im Zellmodus nimmt den Tabulator ebenfalls (Zelle weiter);
        // hinaus kommt man mit Strg+Tab.
        let strg = g.key_ctrl();
        let tab_belegt = self.focus_widget
            .and_then(|(w, i)| self.windows.get(w).and_then(|win| win.widgets.get(i)))
            .map(|w| (w.kind == Kind::TextArea && (w.tab_fuegt_ein || w.tab_meldet))
                || (w.kind == Kind::Table && !strg && w.tbl.as_ref().map(|t| t.zellmodus).unwrap_or(false)))
            .unwrap_or(false);
        if g.key_pressed(KEY_TAB) && !tab_belegt {
            if let Some(top) = self.focus_window {
                let idxs = self.tab_reihenfolge(top);
                if !idxs.is_empty() {
                    let cur = self.focus_widget.filter(|(w, _)| *w == top).map(|(_, i)| i);
                    let pos = cur.and_then(|c| idxs.iter().position(|&i| i == c));
                    let fwd = !g.key_shift();
                    let next = match pos {
                        Some(p) => if fwd { (p + 1) % idxs.len() } else { (p + idxs.len() - 1) % idxs.len() },
                        None => 0,
                    };
                    let ni = idxs[next];
                    self.open_dropdown = None;   // beim Weitertabben nichts offen stehen lassen
                    self.focus_widget = Some((top, ni));
                    // Der Hilfetext erscheint nach kurzem Verweilen auch hier
                    // -- bisher stand er nur der Maus zur Verfuegung.
                    self.tip_fokus = Some((top, ni));
                    self.tip_fokus_frame = self.frame_count;
                    // Nur ein Textfeld bekommt Caret + Markierung -- bei einem
                    // Knopf waeren beide bedeutungslos.
                    if nimmt_text_w(&self.windows[top].widgets[ni]) {
                        let len = self.windows[top].widgets[ni].text.chars().count() as i32;
                        let w = &mut self.windows[top].widgets[ni];
                        w.caret = len; w.sel_anchor = 0;   // Inhalt markiert (wie ueblich beim Tabben)
                    }
                }
            }
        }
        // ESC schliesst zuerst einen offenen Popover -- er liegt obenauf und
        // ist das, was man zuletzt geoeffnet hat.
        if g.key_pressed(KEY_ESC) && !self.liste_taste && self.open_dropdown.is_none() && self.pop_esc() {
            self.liste_taste = true;
        }
        // Standard- und Abbrechen-Knopf: Enter loest den einen aus, ESC den
        // anderen -- aber nur, wenn die Taste nicht schon einem Widget gehoert.
        // Ein Knopf oder Kaestchen mit Fokus nimmt Enter selbst (widget_keys),
        // ein Textbereich macht daraus einen Umbruch, eine Zelle in
        // Bearbeitung ihr Ende. Aus einem Textfeld heraus ist Enter dagegen
        // genau das: das Formular abschicken.
        if let Some(top) = self.focus_window {
            if top < self.windows.len() && self.windows[top].alive && self.windows[top].visible
                && self.modal.map_or(true, |m| m == top) && self.editing_table.is_none() {
                let fok = self.focus_widget.filter(|(w, _)| *w == top)
                    .and_then(|(_, i)| self.windows[top].widgets.get(i).map(|w| w.kind));
                let fok_frei = self.focus_widget.filter(|(w, _)| *w == top)
                    .and_then(|(_, i)| self.windows[top].widgets.get(i))
                    .map(|w| w.kind == Kind::Dropdown && w.frei).unwrap_or(false);
                // Ein Zahlenfeld uebernimmt Enter und gibt es weiter (spinner_keys)
                // -- vorher blieb es dort haengen, ein Dialog mit Zahl liess sich
                // nicht mit Enter bestaetigen (Kuehlschrank-App, MHD setzen).
                // Ebenso ein Kaestchen, ein Schalter, ein Radio: sie schalten mit der
                // Leertaste, Enter schickt das Formular ab. Vorher schaltete Enter
                // das Kaestchen zurueck, das man eben angeklickt hatte, und der
                // Dialog liess sich nicht mit Enter speichern (Kuehlschrank-App,
                // Einstellungen).
                // Ein zugeklapptes Datumsfeld ebenso (wie QDateEdit): Enter gilt
                // dem Formular, aufgeklappt waehlt es den Tag.
                let fok_datum = self.focus_widget.filter(|(w, _)| *w == top)
                    .and_then(|(_, i)| self.windows[top].widgets.get(i))
                    .map(|w| w.kind == Kind::DatePicker && w.kompakt).unwrap_or(false)
                    && self.datum_auf.is_none();
                let enter_frei = matches!(fok, None | Some(Kind::TextInput) | Some(Kind::Spinner)
                    | Some(Kind::Checkbox) | Some(Kind::Toggle) | Some(Kind::Radio)) || fok_frei || fok_datum;
                // Beim Umbenennen in einer Liste gehoeren Enter und ESC dem Feld.
                let liste_edit = self.focus_widget.filter(|(w, _)| *w == top)
                    .and_then(|(_, i)| self.windows[top].widgets.get(i))
                    .map(Self::wird_umbenannt)
                    .unwrap_or(false);
                let liste_edit = liste_edit || self.liste_taste;
                let enter_frei = enter_frei && !liste_edit;
                if enter_frei && g.key_pressed(KEY_ENTER) {
                    let k = self.windows[top].default_btn;
                    self.knopf_ausloesen(top, k);
                }
                if g.key_pressed(KEY_ESC) && self.open_dropdown.is_none() && !liste_edit {
                    let k = self.windows[top].cancel_btn;
                    self.knopf_ausloesen(top, k);
                }
            }
        }
        }   // !menue_hat_tasten
        self.dialog_auswerten();
        self.was_mouse_down = is_down;
        if !is_down { self.zug_feld = None; }
        self.was_right_down = right_down;
        self.frame_count += 1;
    }

    /// Hat der offene Dialog in diesem Bild eine Antwort bekommen? Dann
    /// Antwort setzen, Fenster abraeumen, Modalitaet aufheben.
    ///
    /// Das Fenster wird ZERSTOERT, nicht nur versteckt: sonst sammelten sich
    /// bei einem Programm, das oft nachfragt, immer mehr tote Dialoge im
    /// z_order an. Der Handle bleibt als Tombstone gueltig, `GUI_ANSWER`
    /// liest die Antwort also noch in diesem Bild.
    pub(super) fn dialog_auswerten(&mut self) {
        let wi = match self.modal { Some(wi) => wi, None => return };
        // Ein per GUI_WINDOW_MODAL geschaltetes EIGENES Fenster gibt die
        // Modalitaet frei, sobald es weg oder ausgeblendet ist -- sonst
        // saesse das Programm hinter einem Schleier ohne Fenster.
        if !self.windows.get(wi).map(|w| w.alive && w.visible).unwrap_or(false) {
            self.modal_loesen();
            return;
        }
        if !self.windows[wi].dlg { return; }
        // Antwort = Nummer des Knopfs (1-basiert); Schliessen = der letzte.
        let n_knoepfe = self.windows[wi].widgets.iter().filter(|w| w.kind == Kind::Button).count() as i32;
        let mut antwort = 0;
        if self.windows[wi].close_clicked { antwort = n_knoepfe; }
        let mut k = 0;
        for wdg in self.windows[wi].widgets.iter() {
            if wdg.kind == Kind::Button {
                k += 1;
                if wdg.clicked { antwort = k; }
            }
        }
        if antwort == 0 { return; }
        self.modal_loesen();
        // Wie window_destroy: aus der z-Reihenfolge nehmen und JEDE haengende
        // Interaktions-Referenz loesen.
        self.z_order.retain(|&i| i != wi);
        self.clear_window_interactions(wi);
        let win = &mut self.windows[wi];
        win.alive = false;
        win.visible = false;
        win.answer = antwort;
    }

    // --- Menue-Eingabe + Hit-Tests -----------------------------------------
    /// Layout eines Popups aus EINER Quelle fuer Treffertest und Zeichnen:
    /// (Breite, linker Einzug fuer Haekchen/Sinnbild, Breite der Kuerzelspalte).
    pub(super) fn popup_layout(&self, g: &Graphics, wi: usize, mi: usize) -> (i32, i32, i32) {
        let pad = self.m("pad");
        let mut wlbl = 60;
        let mut wk = 0;
        let mut links = 0;
        let mut pfeil = 0;
        if let Some(m) = self.windows.get(wi).and_then(|w| w.menus.get(mi)) {
            for it in &m.items {
                if it.separator { continue; }
                wlbl = wlbl.max(self.ctext_width(g, &it.label));
                if !it.kuerzel.is_empty() { wk = wk.max(self.ctext_width(g, &it.kuerzel)); }
                if it.haken || it.icon >= 0 || !it.sym.is_empty() { links = self.sk(22); }
                if it.sub >= 0 { pfeil = self.sk(16); }
            }
        }
        let wmax = pad * 4 + links + wlbl + (if wk > 0 { self.sk(24) + wk } else { 0 }) + pfeil;
        (wmax.max(80), links, wk)
    }
    pub(super) fn popup_size(&self, g: &Graphics, wi: usize, mi: usize) -> (i32, i32) {
        let n = self.windows.get(wi).and_then(|w| w.menus.get(mi)).map(|m| m.items.len()).unwrap_or(0);
        (self.popup_layout(g, wi, mi).0, n as i32 * self.sk(MENU_ITEM_H) + 4)
    }
    pub(super) fn popup_width(&self, g: &Graphics, wi: usize, mi: usize) -> i32 { self.popup_layout(g, wi, mi).0 }
    pub(super) fn popup_item_at(&self, g: &Graphics, wi: usize, mi: usize, px: i32, py: i32, mx: i32, my: i32) -> Option<usize> {
        let m = self.windows.get(wi).and_then(|w| w.menus.get(mi))?;
        let wmax = self.popup_width(g, wi, mi);
        if mx < px || mx >= px + wmax { return None; }
        for (ii, it) in m.items.iter().enumerate() {
            let iy = py + 2 + ii as i32 * self.sk(MENU_ITEM_H);
            if my >= iy && my < iy + self.sk(MENU_ITEM_H) {
                if it.separator || !it.enabled { return None; }
                return Some(ii);
            }
        }
        None
    }
    pub(super) fn bar_menu_at(&self, g: &Graphics, wi: usize, mx: i32, my: i32) -> Option<usize> {
        if self.menubar_h(wi) == 0 { return None; }
        let by = self.windows[wi].y + if self.windows[wi].chrome { self.m("title_h") } else { 0 };
        if my < by || my >= by + self.sk(MENUBAR_H) { return None; }
        self.menubar_slots(g, wi).into_iter().find(|(_, x0, x1)| mx >= *x0 && mx < *x1).map(|(mi, _, _)| mi)
    }
    pub(super) fn fire_menu_item(&mut self, wi: usize, mi: usize, ii: usize) {
        if let Some(it) = self.windows.get_mut(wi).and_then(|w| w.menus.get_mut(mi)).and_then(|m| m.items.get_mut(ii)) {
            it.clicked = true;
            if it.haken { it.an = !it.an; }
        }
    }
    /// Verarbeitet Menueleisten-/Dropdown-/Kontext-Klicks. Gibt true zurueck,
    /// wenn der Klick konsumiert wurde (dann kein Widget-Press).
    pub(super) fn menu_input(&mut self, mx: i32, my: i32, left_just: bool, right_just: bool, g: &Graphics) -> bool {
        // 1) + 2) Ein Wurzelmenue ist offen (Kontext oder Leiste), womoeglich
        // mit Untermenues darunter. Der Klick wird von der TIEFSTEN Ebene aus
        // gesucht -- ein Untermenue liegt ueber seinem Elternmenue.
        if self.context_open.is_some() || self.open_menu.is_some() {
            if !(left_just || right_just) { return false; }
            let chain = self.popup_chain(g);
            let ist_kontext = self.context_open.is_some();
            for &(wi, mi, px, py) in chain.iter().rev() {
                let (wmax, h) = self.popup_size(g, wi, mi);
                if !(mx >= px && mx < px + wmax && my >= py && my < py + h) { continue; }
                if let Some(ii) = self.popup_item_at(g, wi, mi, px, py, mx, my) {
                    if self.windows[wi].menus[mi].items[ii].sub >= 0 {
                        return true;          // Untermenue-Kopf: bleibt offen
                    }
                    self.open_menu = None; self.context_open = None; self.sub_chain.clear();
                    self.fire_menu_item(wi, mi, ii);
                }
                return true;                  // Trennlinie/gesperrt: Klick verpufft, Menue bleibt
            }
            // Klick neben allen Popups.
            self.sub_chain.clear();
            if ist_kontext {
                self.context_open = None;
                return true;
            }
            let (wi, _) = self.open_menu.unwrap();
            if left_just {
                if let Some(nm) = self.bar_menu_at(g, wi, mx, my) {
                    self.open_menu = Some((wi, nm));   // auf anderes Bar-Menue umschalten
                    return true;
                }
            }
            self.open_menu = None;                     // Klick ausserhalb -> schliessen
            return true;
        }
        // 3) Kein Menue offen
        if left_just {
            for &wi in self.z_order.clone().iter().rev() {
                if !self.windows[wi].alive || !self.windows[wi].visible { continue; }
                // Nur Fenster der Flaeche unter dem Zeiger (Weg E: ein
                // natives steht bei 0,0 -- sonst traefe ein Klick oben im
                // Hauptfenster seine Menueleiste) und, waehrend eines
                // modalen Dialogs, nur dieser.
                if self.windows[wi].nativ != self.flaeche { continue; }
                if self.modal.is_some_and(|m| m != wi) { continue; }
                if let Some(nm) = self.bar_menu_at(g, wi, mx, my) {
                    self.open_menu = Some((wi, nm));
                    return true;
                }
            }
        }
        if right_just {
            if let Some(wi) = self.topmost_at(mx, my).filter(|&wi| self.modal.is_none_or(|m| m == wi)) {
                // Ein an ein Widget gebundenes Kontextmenue gewinnt, wenn die
                // Maus auf seinem Widget steht; sonst das des Fensters. Das
                // Widget waehlt dabei trotzdem seine Zeile (Liste, Baum,
                // Tabelle): der Rechtsklick erreicht es im selben Bild.
                let mut wahl: Option<usize> = None;
                // Auf einem Reiter: das Menue der Reiterleiste, und es merkt
                // sich, welcher Reiter gemeint war.
                if let Some(mi) = self.windows[wi].menus.iter().position(|m| !m.in_bar && !m.unter && m.ziel == -2) {
                    let by = self.windows[wi].y
                        + (if self.windows[wi].chrome { self.m("title_h") } else { 0 })
                        + self.menubar_h(wi);
                    if self.tabbar_h(wi) > 0 && my >= by && my < by + TABBAR_H {
                        if let Some((ti, _, _)) = self.tab_slots(g, wi).into_iter().find(|(_, x0, x1)| mx >= *x0 && mx < *x1) {
                            self.windows[wi].tab_kontext = ti as i32;
                            self.context_open = Some((wi, mi, mx, my));
                            self.menu_cursor = None;
                            return true;
                        }
                    }
                }
                for (mi, m) in self.windows[wi].menus.iter().enumerate() {
                    if m.in_bar || m.unter || m.ziel < 0 { continue; }
                    let Some(w) = self.windows[wi].widgets.get(m.ziel as usize) else { continue; };
                    if !w.alive || !self.widget_shown(wi, w) { continue; }
                    // Die Nummernspalte eines Textbereichs gehoert ihrem eigenen
                    // Rechtsklick (GUI_TEXTAREA_GUTTER_CLICKED, bei einer IDE die
                    // Bedingung eines Haltepunkts) -- dort kein Menue.
                    if w.kind == Kind::TextArea && self.ta_rand_zeile(g, wi, m.ziel as usize, mx, my).is_some() { continue; }
                    if Self::in_rect(mx, my, self.abs_rect(wi, w)) {
                        self.kontext_neu = Some((wi, m.ziel as usize));
                        wahl = Some(mi);
                        break;
                    }
                }
                if wahl.is_none() {
                    wahl = self.windows[wi].menus.iter().position(|m| !m.in_bar && !m.unter && m.ziel == -1);
                }
                if let Some(mi) = wahl {
                    self.context_open = Some((wi, mi, mx, my));
                    self.menu_cursor = None;
                    return true;
                }
            }
        }
        false
    }

    /// GUI_CONTEXT_WIDGET(menu, wdg): das Kontextmenue gehoert zu einem
    /// Widget desselben Fensters und oeffnet nur beim Rechtsklick darauf.
    /// wdg = -1 bindet es wieder ans ganze Fenster.
    pub fn context_widget(&mut self, menu: i64, wdg: i64) -> Result<(), String> {
        let (wi, _) = dec_menu(menu);
        let ziel = if wdg < 0 { -1 } else {
            let (ww, i) = Self::dec_widget(wdg);
            self.wdg(wdg, "GUI_CONTEXT_WIDGET")?;
            if ww != wi { return Err("GUI_CONTEXT_WIDGET: Menue und Widget liegen in verschiedenen Fenstern".into()); }
            i as i64
        };
        let m = self.menu_mut(menu, "GUI_CONTEXT_WIDGET")?;
        if m.in_bar || m.unter { return Err("GUI_CONTEXT_WIDGET: erwartet ein Menue aus GUI_CONTEXT".into()); }
        m.ziel = ziel;
        Ok(())
    }

    /// Zeichen-Index, dessen Caret-Position am naechsten an `target_px` liegt
    /// (Pixel ab Textanfang). Misst echte Textbreiten -> funktioniert mit jedem Font.
    /// Das Wort um eine Zeichenstelle: Buchstaben, Ziffern, `_` und `$`
    /// (Drachenhauch-Namen wie `name$`). Steht dort kein Wortzeichen, gilt
    /// das eine Zeichen -- ein Doppelklick auf eine Klammer waehlt sie.
    pub(super) fn wort_um(chars: &[char], idx: usize) -> (usize, usize) {
        let ist = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
        if chars.is_empty() { return (0, 0); }
        let mut k = idx.min(chars.len());
        // Auf dem Ende eines Wortes (Marke dahinter): das Wort davor.
        if (k == chars.len() || !ist(chars[k])) && k > 0 && ist(chars[k - 1]) { k -= 1; }
        if k >= chars.len() || !ist(chars[k]) {
            let e = (k + 1).min(chars.len());
            return (k.min(chars.len()), e);
        }
        let mut a = k;
        while a > 0 && ist(chars[a - 1]) { a -= 1; }
        let mut e = k;
        while e < chars.len() && ist(chars[e]) { e += 1; }
        (a, e)
    }
    pub(super) fn caret_index_at(g: &Graphics, chars: &[char], target_px: i32, ms: Mass) -> i32 {
        // Eine Zeile mit Text von rechts nach links steht anders da, als sie
        // gespeichert ist -- der Klick trifft dann ein Anzeige-Zeichen.
        if let Some(l) = Self::rtl_lage_mit(g, chars, |s| ms.breite(g, s)) { return l.index(target_px) as i32; }
        if target_px <= 0 { return 0; }
        // Gesucht ist das erste n, bei dem der Punkt vor der Mitte des n-ten
        // Zeichens liegt. Die Mitte waechst mit n -- also binaer statt Zeichen
        // fuer Zeichen mit jeweils neu gemessenem Vorspann: das war quadratisch,
        // ein Klick in Spalte 20000 einer langen Zeile mass 400 Millionen
        // Zeichen (Stresstest 2026-10-02).
        let breite = |n: usize| -> i32 { let t: String = chars[..n].iter().collect(); ms.breite(g, &t) };
        let (mut lo, mut hi) = (1usize, chars.len() + 1);
        while lo < hi {
            let n = (lo + hi) / 2;
            let mid = (breite(n - 1) + breite(n)) / 2;
            if target_px < mid { hi = n; } else { lo = n + 1; }
        }
        (lo - 1) as i32
    }

    /// Volles Editieren des fokussierten TextInputs: Zeichen einfuegen, Backspace/
    /// Delete, Pfeile/Home/End (Shift = Selektion), Strg+A/C/V/X, Maus-Klick/-Drag
    /// zum Setzen/Aufziehen der Selektion. Caret/Selektion/Scroll werden mit
    /// GEMESSENEN Textbreiten gefuehrt (passt zu jedem Font).
    /// Bearbeitbare Klappliste mit Fokus: getippt wird wie in einem Textfeld
    /// (dieselbe Routine), die Liste schlaegt vor. Pfeil ab klappt auf, offen
    /// waehlen die Pfeile, Enter nimmt den mit den Pfeilen gewaehlten Eintrag
    /// -- sonst bleibt, was getippt ist. ESC klappt nur zu.
    pub(super) fn edit_combo(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let offen = self.open_dropdown == Some((wi, i));
        let n = self.windows[wi].widgets[i].items.len() as i32;
        if offen {
            if g.key_pressed(KEY_ESC) { self.open_dropdown = None; self.liste_taste = true; return; }
            let rows = self.dd_zeilen(wi, i);
            let m = self.dd_mark;
            let neu = if g.key_pressed(KEY_DOWN) { Some(m + 1) }
                else if g.key_pressed(KEY_UP) { Some(if m < 0 { 0 } else { m - 1 }) }
                else if g.key_pressed(KEY_PAGEDOWN) { Some(m + rows) }
                else if g.key_pressed(KEY_PAGEUP) { Some(m - rows) }
                else { None };
            if let Some(k) = neu {
                if n > 0 {
                    self.dd_mark = k.clamp(0, n - 1);
                    self.dd_pfeil = true;
                    self.dd_mark_zeigen(wi, i);
                }
                return;
            }
            if g.key_pressed(KEY_ENTER) {
                // Enter gehoert hier der Liste, nicht dem Standard-Knopf.
                self.liste_taste = true;
                self.open_dropdown = None;
                if self.dd_pfeil && self.dd_mark >= 0 {
                    let k = self.dd_mark;
                    self.dd_waehle(wi, i, k);
                    return;
                }
            }
        } else if g.key_pressed(KEY_DOWN) && n > 0 {
            self.open_dropdown = Some((wi, i));
            self.dd_pfeil = false;
            return;
        }
        let vorher = self.windows[wi].widgets[i].text.clone();
        self.edit_textinput(wi, i, g);
        let w = &mut self.windows[wi].widgets[i];
        if w.text == vorher { return; }
        dd_frei_abgleichen(w);
        // Vorschlag: der erste Eintrag, der so anfaengt. Nur markiert --
        // uebernommen wird er erst mit den Pfeilen.
        let t = w.text.to_lowercase();
        let treffer = if t.is_empty() { None } else { w.items.iter().position(|x| x.to_lowercase().starts_with(&t)) };
        match treffer {
            Some(k) => {
                self.open_dropdown = Some((wi, i));
                // Selbst als "frisch geoeffnet" gemerkt -- sonst setzte das
                // naechste Bild die Markierung zurueck auf die Auswahl.
                self.dd_auf_von = self.open_dropdown;
                self.dd_mark = k as i32;
                self.dd_pfeil = false;
                self.dd_mark_zeigen(wi, i);
            }
            None => { if self.open_dropdown == Some((wi, i)) { self.open_dropdown = None; } }
        }
    }

    pub(super) fn edit_textinput(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let ms = self.mass(g, &self.windows[wi].widgets[i]);
        let before = self.windows[wi].widgets[i].text.clone();
        let (ax, _ay, fw, _fh) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let (nur_lesen, maxl, zahlen, passwort) = {
            let w = &self.windows[wi].widgets[i];
            (w.nur_lesen, w.maxlaenge, w.zahlen, w.passwort)
        };
        let mut chars: Vec<char> = before.chars().collect();
        let mut caret = self.windows[wi].widgets[i].caret.clamp(0, chars.len() as i32);
        let mut anchor = self.windows[wi].widgets[i].sel_anchor.clamp(0, chars.len() as i32);
        let (caret0, anchor0) = (caret, anchor);
        let scroll = self.windows[wi].widgets[i].scroll;
        let shift = g.key_shift();
        let ctrl = g.key_ctrl();

        // Rueckgaengig/Wiederholen vor allem anderen -- der Schritt ersetzt
        // die Eingabe dieses Bildes.
        if ctrl && (g.key_pressed(K_Z) || g.key_pressed(K_Y)) {
            if !nur_lesen { self.text_undo(wi, i, g.key_pressed(K_Y) || shift); }
            return;
        }
        // Passwortpunkte sind anders breit als die Zeichen -- Treffertest
        // und Rollen rechnen mit dem, was zu SEHEN ist.
        let sichtbar = |cs: &[char]| -> Vec<char> {
            if passwort { vec!['\u{2022}'; cs.len()] } else { cs.to_vec() }
        };

        // --- Maus: Klick setzt Caret (rising edge), Ziehen erweitert Selektion ---
        // Doppelklick waehlt das Wort, Dreifachklick alles -- wie im
        // Textbereich, gezaehlt an DIESER Stelle in 0,4 s. Ein Passwortfeld
        // kennt keine Woerter (sie verrieten, wo ein Leerzeichen steht): dort
        // waehlt schon der Doppelklick alles.
        if !g.mouse_button(0) { self.windows[wi].widgets[i].wort_zug = false; }
        if g.mouse_button(0) {
            let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
            let (sl, sr) = self.ti_rand(&self.windows[wi].widgets[i]);
            let target = mx - (ax + 5 + sl) + scroll;
            let idx = Self::caret_index_at(g, &sichtbar(&chars), target, ms);
            if !self.was_mouse_down {
                // Ueber einem Sinnbild setzt der Druck keine Marke.
                let (rx, ry, rw, rh) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
                if Self::in_rect(mx, my, (rx + sl, ry, (rw - sl - sr).max(0), rh)) {
                    self.zug_feld = Some((wi, i));
                    caret = idx; anchor = idx;
                    let jetzt = g.get_time();
                    let w = &mut self.windows[wi].widgets[i];
                    let n = if jetzt - w.klick_zeit < 0.4 && (idx - w.klick_idx).abs() <= 2 { w.klick_n % 3 + 1 } else { 1 };
                    w.klick_n = n; w.klick_zeit = jetzt; w.klick_idx = idx;
                    if n == 3 || (n == 2 && passwort) {
                        anchor = 0; caret = chars.len() as i32;
                        w.wort_zug = true;
                    } else if n == 2 {
                        let (a, e) = Self::wort_um(&chars, idx as usize);
                        anchor = a as i32; caret = e as i32;
                        w.wort_zug = true;
                    }
                }
            } else if !self.windows[wi].widgets[i].wort_zug && self.zug_feld == Some((wi, i)) {
                caret = idx;   // Drag -> Selektion bis hierher (nur ein Zug, der hier begann)
            }
        }

        Self::einzeiler_tasten(g, &mut chars, &mut caret, &mut anchor, ctrl, shift,
                               self.kuerzel_gefeuert);
        // Hat ein Menue-Kuerzel die Taste genommen (Strg+Alt+Enter), ist es
        // kein Abschicken des Feldes -- sonst taete eine Taste zwei Dinge.
        let enter = g.key_pressed(KEY_ENTER) && !self.kuerzel_gefeuert;

        // --- Grenzen des Feldes ---
        // Ein gesperrtes Feld nimmt nichts an (Schreibmarke und Kopieren
        // gehen weiter), eine Hoechstlaenge schneidet ab -- auch beim
        // Einfuegen --, der Zahlenfilter lehnt den ganzen Schritt ab.
        let mut neu: String = chars.iter().collect();
        if neu != before {
            let mut zurueck = nur_lesen;
            if !zurueck && maxl > 0 && chars.len() as i32 > maxl {
                chars.truncate(maxl as usize);
                neu = chars.iter().collect();
                caret = caret.min(maxl); anchor = anchor.min(maxl);
            }
            if !zurueck && !zahl_erlaubt(&neu, zahlen) { zurueck = true; }
            if zurueck {
                chars = before.chars().collect();
                neu = before.clone();
                caret = caret0.min(chars.len() as i32); anchor = anchor0.min(chars.len() as i32);
            }
        }

        // --- Horizontalen Scroll so anpassen, dass das Caret sichtbar bleibt ---
        let sicht = sichtbar(&chars);
        let caret_px = match Self::rtl_lage_mit(g, &sicht, |s| ms.breite(g, s)) {
            Some(l) => l.marke(caret as usize),
            None => ms.breite(g, &sicht[..caret as usize].iter().collect::<String>()),
        };
        let pfeil = if self.windows[wi].widgets[i].frei { self.sk(DD_PFEIL_B) } else { 0 };
        let (sl, sr) = self.ti_rand(&self.windows[wi].widgets[i]);
        let inner = (fw - 10 - pfeil - sl - sr).max(1);
        let mut scroll = scroll;
        if caret_px - scroll > inner { scroll = caret_px - inner; }
        if caret_px - scroll < 0 { scroll = caret_px; }
        if scroll < 0 { scroll = 0; }

        // --- Zurueckschreiben ---
        let geaendert = neu != before;
        let jetzt = g.get_time();
        let w = &mut self.windows[wi].widgets[i];
        if geaendert { Self::undo_merken(w, &before, caret0, jetzt); }
        w.text = neu;
        w.caret = caret; w.sel_anchor = anchor; w.scroll = scroll;
        if geaendert {
            if let Some(f) = w.on_change.clone() { self.pending.push(f); }
            if self.windows[wi].widgets[i].kind == Kind::TextInput { self.feld_getippt(wi, i); }
        }
        if enter {
            let w = &mut self.windows[wi].widgets[i];
            w.entered = true;
            let f = w.on_enter.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Tastatur eines EINZEILIGEN Textfelds: Tippen, Strg+A/C/V/X,
    /// Ruecktaste/Entf, Pfeile, Pos1/Ende -- jeweils mit Textauswahl
    /// (Umschalt erweitert, Tippen ersetzt das Markierte).
    ///
    /// Herausgeloest, damit das Textfeld-Widget UND der Zell-Editor der Tabelle
    /// dieselbe Logik benutzen. Vorher hatte die Zelle eine eigene, aermere
    /// Fassung ohne Auswahl -- zwei Orte fuer dieselbe Sache, an denen dieselben
    /// Fehler zweimal entstehen koennen.
    ///
    /// `chars`/`caret`/`anchor` werden an Ort und Stelle geaendert; wer sie
    /// haelt (Widget oder Zelle) entscheidet der Aufrufer.
    pub(super) fn einzeiler_tasten(g: &mut Graphics, chars: &mut Vec<char>,
                        caret: &mut i32, anchor: &mut i32, ctrl: bool, shift: bool,
                        kuerzel_weg: bool) {
        // --- Zeichen-Eingabe (nicht bei gedruecktem Strg) ---
        if !ctrl {
            let typed: String = g.pop_text_input().chars()
                .filter(|c| !c.is_control() && *c != '\t').collect();
            if !typed.is_empty() {
                let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
                if lo != hi { chars.drain(lo as usize..hi as usize); *caret = lo; }
                for (k, ch) in typed.chars().enumerate() { chars.insert(*caret as usize + k, ch); }
                *caret += typed.chars().count() as i32; *anchor = *caret;
            }
        } else if !kuerzel_weg {
            // Hat ein Menue-Kuerzel dieselbe Taste schon verbraucht, gehoert
            // sie ihm allein (siehe edit_textarea).
            // Strg+A: alles markieren
            if g.key_pressed(K_A) { *anchor = 0; *caret = chars.len() as i32; }
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if g.key_pressed(K_C) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
            }
            if g.key_pressed(K_X) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
                chars.drain(lo as usize..hi as usize); *caret = lo; *anchor = lo;
            }
            if g.key_pressed(K_V) {
                let ins: Vec<char> = g.clipboard_get().chars().filter(|c| !c.is_control()).collect();
                let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
                if lo != hi { chars.drain(lo as usize..hi as usize); *caret = lo; }
                for (k, ch) in ins.iter().enumerate() { chars.insert(*caret as usize + k, *ch); }
                *caret += ins.len() as i32; *anchor = *caret;
            }
        }

        // --- Ruecktaste / Entf ---
        if g.key_pressed(KEY_BACKSPACE) {
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if lo != hi { chars.drain(lo as usize..hi as usize); *caret = lo; }
            else if *caret > 0 { chars.remove(*caret as usize - 1); *caret -= 1; }
            *anchor = *caret;
        }
        if g.key_pressed(KEY_DELETE) {
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if lo != hi { chars.drain(lo as usize..hi as usize); *caret = lo; }
            else if (*caret as usize) < chars.len() { chars.remove(*caret as usize); }
            *anchor = *caret;
        }

        // --- Navigation (Umschalt = Auswahl erweitern) ---
        let len = chars.len() as i32;
        *caret = (*caret).clamp(0, len); *anchor = (*anchor).clamp(0, len);
        // Steht die Zeile rechts nach links, geht "links" im Text weiter.
        let (k_zurueck, k_vor) = if richtung::zeile_rtl(chars) { (KEY_RIGHT, KEY_LEFT) } else { (KEY_LEFT, KEY_RIGHT) };
        if g.key_pressed(k_zurueck) {
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if !shift && lo != hi { *caret = lo; } else { *caret = (*caret - 1).max(0); }
            if !shift { *anchor = *caret; }
        }
        if g.key_pressed(k_vor) {
            let (lo, hi) = ((*caret).min(*anchor), (*caret).max(*anchor));
            if !shift && lo != hi { *caret = hi; } else { *caret = (*caret + 1).min(len); }
            if !shift { *anchor = *caret; }
        }
        if g.key_pressed(KEY_HOME) { *caret = 0; if !shift { *anchor = 0; } }
        if g.key_pressed(KEY_END) { *caret = len; if !shift { *anchor = len; } }
    }

    /// Zeilenanfang-Indizes (Char-Position nach jedem '\n', plus 0).
    pub(super) fn line_starts(chars: &[char]) -> Vec<usize> {
        let mut v = vec![0usize];
        for (i, &c) in chars.iter().enumerate() { if c == '\n' { v.push(i + 1); } }
        v
    }
    /// Zeilenhoehe der TextArea (Schriftgroesse + etwas Durchschuss).
    /// Zeilenhoehe eines Textbereichs -- aus SEINER Schriftgroesse, nicht aus
    /// der aktiven. Mit der aktiven stimmten Zeilen und Text nicht mehr,
    /// sobald ein Feld eine eigene Groesse hatte (gezoomtes Code-Feld,
    /// groessere Oberflaechenschrift): die Zeilen ueberlappten oder klafften.
    pub(super) fn ta_line_h(&self, g: &Graphics, wdg: &Widget) -> i32 {
        let gr = if wdg.font_size > 0 { self.sk(wdg.font_size) } else { self.ctext_height(g) };
        (gr + self.sk(5)).max(self.sk(10))
    }

    /// Breite, auf die ein Textbereich umbricht (0 = kein Umbruch).
    pub(super) fn ta_breite(&self, g: &Graphics, wdg: &Widget, zeilen: usize) -> i32 {
        if !wdg.umbruch { return 0; }
        let gutter = self.ta_gutter(g, wdg, zeilen);
        (wdg.w - 2 * 5 - gutter - self.sk(8)).max(self.sk(20))
    }

    /// Was ein Textbereich ueber Einrueckung weiss -- herausgeloest, weil der
    /// Enter-Zweig es in einem Verschluss braucht und dort kein Widget mehr
    /// ausleihen kann.
    pub(super) fn einzug_regeln(w: &Widget) -> EinzugRegeln {
        EinzugRegeln {
            auto_einzug: w.auto_einzug,
            einzug_anfang: w.einzug_anfang.clone(),
            einzug_ende: w.einzug_ende.clone(),
            einzug_aus: w.einzug_aus.clone(),
            schluss_oeffner: w.schluss_oeffner.clone(),
            schluss_texte: w.schluss_texte.clone(),
            tabbreite: w.tabbreite,
        }
    }

    /// Anfang und Ende der Zeile, in der `pos` steht (Zeichen-Indizes).
    pub(super) fn zeile_um(chars: &[char], pos: usize) -> (usize, usize) {
        let pos = pos.min(chars.len());
        let von = chars[..pos].iter().rposition(|&c| c == '\n').map(|i| i + 1).unwrap_or(0);
        let bis = chars[pos..].iter().position(|&c| c == '\n').map(|i| pos + i).unwrap_or(chars.len());
        (von, bis)
    }

    /// Der Einzug, mit dem eine neue Zeile hinter `pos` anfangen soll.
    ///
    /// Grundlage ist die Einrueckung der laufenden Zeile -- das allein ist
    /// sprachfrei und schon der halbe Nutzen. Eine Stufe mehr gibt es, wenn
    /// die Zeile mit einem der `einzug_anfang`-Woerter beginnt oder auf eines
    /// der `einzug_ende`-Woerter endet. Zwei Listen und nicht eine: `SUB`
    /// oeffnet am ANFANG, `THEN` am ENDE -- mit einer Liste ruckte auch ein
    /// `END SUB` die naechste Zeile ein.
    pub(super) fn neuer_einzug(w: &EinzugRegeln, chars: &[char], pos: usize) -> String {
        if !w.auto_einzug { return String::new(); }
        let (von, _) = Self::zeile_um(chars, pos);
        let mut einzug = String::new();
        for &c in &chars[von..pos.min(chars.len())] {
            if c == ' ' || c == '\t' { einzug.push(c); } else { break; }
        }
        let bis_marke: String = chars[von..pos.min(chars.len())].iter().collect();
        if Self::oeffnet(w, &bis_marke.trim().to_uppercase()) {
            einzug.push_str(&" ".repeat(w.tabbreite.max(1) as usize));
        }
        einzug
    }

    /// Faengt `s` mit dem Wort `w` an? (Wort heisst: danach kommt kein
    /// Namenszeichen mehr -- `ENDE` faengt nicht mit `END` an.)
    pub(super) fn wort_am_anfang(s: &str, w: &str) -> bool {
        s == w || (s.len() > w.len() && s.starts_with(w)
                   && !s[w.len()..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_'))
    }

    /// Endet `s` auf dem Wort `w`?
    pub(super) fn wort_am_ende(s: &str, w: &str) -> bool {
        s == w || (s.len() > w.len() && s.ends_with(w)
                   && !s[..s.len() - w.len()].ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_'))
    }

    /// Oeffnet diese Zeile (GROSS, getrimmt) einen Block?
    ///
    /// Dieselbe Frage fuer die Einrueckung und fuer das Geruest -- getrennt
    /// beantwortet liefen beide auseinander, und `IF x THEN y = 1` bekaeme
    /// ein `END IF`, obwohl es nicht einmal einrueckt.
    pub(super) fn oeffnet(w: &EinzugRegeln, text: &str) -> bool {
        if text.is_empty() { return false; }
        w.einzug_anfang.iter().any(|k| Self::wort_am_anfang(text, k))
            || w.einzug_ende.iter().any(|k| Self::wort_am_ende(text, k))
    }

    /// Die Zeile, die den eben geoeffneten Block schliesst -- oder None.
    ///
    /// Gerufen VOR dem Umbruch, mit der Marke noch in der oeffnenden Zeile.
    /// Drei Bedingungen, und jede hat ihren Grund:
    /// * die Zeile oeffnet ueberhaupt einen Block (`oeffnet`) -- sonst
    ///   bekaeme die einzeilige Form `IF x THEN y = 1` ein `END IF`;
    /// * hinter der Marke steht nichts mehr -- sonst landete der Rest der
    ///   Zeile HINTER dem Abschluss;
    /// * der Block ist nicht schon geschlossen (`bereits_geschlossen`) --
    ///   sonst saete jedes Enter am Ende einer alten `SUB`-Zeile ein zweites
    ///   `END SUB`.
    pub(super) fn geruest_schluss(w: &EinzugRegeln, chars: &[char], pos: usize) -> Option<String> {
        if w.schluss_oeffner.is_empty() { return None; }
        let (von, bis) = Self::zeile_um(chars, pos);
        if chars[pos.min(bis)..bis].iter().any(|c| !c.is_whitespace()) { return None; }
        let zeile: String = chars[von..bis].iter().collect();
        let text = zeile.trim().to_uppercase();
        if !Self::oeffnet(w, &text) { return None; }
        // Das laengste passende Wort gewinnt: `DO WHILE` vor `DO`.
        let mut treffer: Option<usize> = None;
        for (k, o) in w.schluss_oeffner.iter().enumerate() {
            if !Self::wort_am_anfang(&text, o) { continue; }
            if treffer.map_or(true, |t| o.len() > w.schluss_oeffner[t].len()) { treffer = Some(k); }
        }
        let k = treffer?;
        let schluss = w.schluss_texte[k].clone();
        let einzug: String = zeile.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
        if Self::bereits_geschlossen(w, chars, bis, einzug.chars().count(), &schluss.to_uppercase()) {
            return None;
        }
        Some(einzug + &schluss)
    }

    /// Steht der Abschluss schon unter der Zeile?
    ///
    /// Gegangen wird ab `ab` (Ende der oeffnenden Zeile) nach unten: tiefer
    /// eingerueckte Zeilen sind der Rumpf, eine Zeile aus der `aus`-Liste auf
    /// gleicher Hoehe gehoert noch dazu (`ELSE`, `CASE`), alles andere auf
    /// gleicher oder geringerer Hoehe beendet die Suche. Bei krumm
    /// eingerueckten Dateien faellt die Antwort auf "nicht geschlossen" --
    /// ein Abschluss zu viel ist leichter zu sehen als einer zu wenig.
    pub(super) fn bereits_geschlossen(w: &EinzugRegeln, chars: &[char], ab: usize,
                           einzug: usize, schluss: &str) -> bool {
        let mut i = ab;
        let mut gesehen = 0;
        while i < chars.len() && gesehen < 500 {
            i += 1;                       // ueber den Umbruch
            if i >= chars.len() { break; }
            let (von, bis) = Self::zeile_um(chars, i);
            i = bis;
            gesehen += 1;
            let zeile: String = chars[von..bis].iter().collect();
            let text = zeile.trim().to_uppercase();
            if text.is_empty() { continue; }
            let tief = zeile.chars().take_while(|c| *c == ' ' || *c == '\t').count();
            if tief > einzug { continue; }
            if tief == einzug {
                if Self::wort_am_anfang(&text, schluss) { return true; }
                if w.einzug_aus.iter().any(|k| text == *k) { continue; }
            }
            return false;
        }
        false
    }

    /// Alle Schreibmarken als (caret, anker), die erste ist die fuehrende.
    pub(super) fn marken(w: &Widget) -> Vec<(i32, i32)> {
        let mut m = vec![(w.caret, w.sel_anchor)];
        m.extend(w.marken_zusatz.iter().copied());
        m
    }

    /// Dieselbe Aenderung an JEDER Marke.
    ///
    /// `f` sagt fuer eine Marke, welcher Bereich weicht und was dafuer
    /// hineinkommt -- Tippen, Enter, Ruecktaste, Entf und Einfuegen sind alle
    /// von dieser Form. Gearbeitet wird von HINTEN nach vorn: dann bleiben
    /// die Stellen der noch offenen Marken gueltig, und es braucht keine
    /// Buchfuehrung ueber Verschiebungen.
    ///
    /// Mit einer einzigen Marke ist das genau der Weg von vorher.
    pub(super) fn an_marken(chars: &mut Vec<char>, marken: &mut [(i32, i32)],
                 f: impl Fn(&[char], i32, i32) -> (usize, usize, String)) {
        let mut reihe: Vec<usize> = (0..marken.len()).collect();
        reihe.sort_by_key(|&i| std::cmp::Reverse(marken[i].0.min(marken[i].1)));
        for i in reihe {
            let (c, a) = marken[i];
            let (von, bis, ein) = f(chars, c, a);
            let von = von.min(chars.len());
            let bis = bis.clamp(von, chars.len());
            let n = ein.chars().count();
            chars.splice(von..bis, ein.chars());
            let neu = (von + n) as i32;
            marken[i] = (neu, neu);
        }
    }

    /// Die Rechtecke der Farbfelder im sichtbaren Ausschnitt, je mit ihrer
    /// Nummer: (index, x, y, kante).
    ///
    /// Sie stehen am ENDE ihrer Zeile, nicht direkt hinter dem Stueck: dort
    /// laege das Quadrat auf dem naechsten Zeichen (bei `CLS(&HFF8800)` genau
    /// auf der Klammer). Mehrere in einer Zeile reihen sich auf, in der
    /// Reihenfolge des Textes.
    ///
    /// EINE Quelle fuer Zeichnen und Treffertest -- liefen sie auseinander,
    /// oeffnete ein Klick neben dem Feld den Farbwaehler (oder gar keiner).
    /// Die Rechtecke der Hinweise hinter dem Zeilenende (GUI_TEXTAREA_HINTS),
    /// je (Zeile ab 0, x, y, b, h) -- EINE Quelle fuer Zeichnen, Klick und
    /// Zeiger. Ein Hinweis steht an der LETZTEN sichtbaren Reihe seiner Zeile
    /// (bei Umbruch), hinter dem Text und hinter der Faltmarke.
    pub(super) fn ta_hinweis_rects(&self, g: &Graphics, wdg: &Widget, ax: i32, ay: i32, chars: &[char],
                        starts: &[usize], rows: &[(usize, usize, usize)], sicht: i32)
                        -> Vec<(usize, i32, i32, i32, i32)> {
        let mut v = Vec::new();
        if wdg.hinweise.is_empty() { return v; }
        let pad = 5;
        let lh = self.ta_line_h(g, wdg);
        let gutter = self.ta_gutter(g, wdg, starts.len());
        let tx0 = ax + pad + gutter - wdg.scroll_x;
        for r in 0..sicht {
            let ri = wdg.scroll + r;
            if ri < 0 || ri as usize >= rows.len() { continue; }
            let (li, rs, re) = rows[ri as usize];
            let letzte = starts.get(li + 1).map(|&n| re + 1 >= n).unwrap_or(true);
            if !letzte { continue; }
            let Some((_, txt)) = wdg.hinweise.iter().find(|(z, _)| *z == li) else { continue };
            let zeile: String = chars[rs..re].iter().collect();
            let breite = self.wtext_width(g, wdg, &zeile);
            let mut ende_x = tx0 + breite;
            if let Some(&(_, b)) = wdg.gefaltet.iter().find(|&&(v, _)| v == li + 1) {
                let t = format!("... {} Zeilen", b - li - 1);
                ende_x = tx0 + breite + self.sk(10) + self.wtext_width(g, wdg, &t) + self.sk(10);
            }
            let bx = ende_x + self.sk(18);
            let bw = self.wtext_width(g, wdg, txt) + self.sk(12);
            v.push((li, bx, ay + pad + r * lh + 1, bw, lh - 4));
        }
        v
    }

    pub(super) fn ta_farbfeld_rects(&self, g: &Graphics, wdg: &Widget, ax: i32, ay: i32,
                         chars: &[char], rows: &[(usize, usize, usize)],
                         starts_len: usize, view_lines: i32)
                         -> Vec<(usize, i32, i32, i32)> {
        let mut raus = Vec::new();
        if wdg.farbfelder.is_empty() { return raus; }
        let pad = 5;
        let lh = self.ta_line_h(g, wdg);
        let gutter = self.ta_gutter(g, wdg, starts_len);
        let tx0 = ax + pad + gutter - wdg.scroll_x;
        let kante = (lh - self.sk(6)).max(self.sk(6));
        let luecke = self.sk(4);
        // Je sichtbarer Zeile: wie viele Felder stehen dort schon?
        let mut belegt: std::collections::HashMap<usize, i32> = Default::default();
        for (k, &(s, l, _)) in wdg.farbfelder.iter().enumerate() {
            let ende = (s as usize + l as usize).min(chars.len());
            // In welcher sichtbaren Zeile endet das Stueck?
            let Some(r) = rows.iter().position(|&(_, von, bis)| ende >= von && ende <= bis)
                else { continue };
            let zeile = r as i32 - wdg.scroll;
            if zeile < 0 || zeile >= view_lines { continue; }
            let nummer = belegt.entry(r).or_insert(0);
            let ganze: String = chars[rows[r].1..rows[r].2].iter().collect();
            let x = tx0 + self.wtext_width(g, wdg, &ganze) + luecke * 2
                    + *nummer * (kante + luecke);
            *nummer += 1;
            let y = ay + pad + zeile * lh + (lh - kante) / 2;
            raus.push((k, x, y, kante));
        }
        raus
    }

    /// Eine Spaltenauswahl als Marken: je Zeile eine, mit ihrer eigenen
    /// Auswahl von Spalte `ca` bis `cb`. Zeilen, die nicht so weit reichen,
    /// bekommen eine Marke an ihrem Ende -- sonst faellt beim Tippen mitten
    /// im Block eine Zeile heraus, und das sieht man erst hinterher.
    ///
    /// Gerechnet wird in ZEICHEN je logischer Zeile. Fuer ein Code-Feld mit
    /// fester Schrittweite ist das genau die Spalte; bei einer proportionalen
    /// Schrift waere eine Spalte ohnehin keine Laenge.
    pub(super) fn spalten_marken(chars: &[char], starts: &[usize], z0: i32, c0: i32, z1: i32, c1: i32)
                      -> Vec<(i32, i32)> {
        let (za, zb) = (z0.min(z1).max(0), z0.max(z1));
        let (ca, cb) = (c0.min(c1).max(0), c0.max(c1));
        let mut raus = Vec::new();
        for z in za..=zb {
            let zi = z as usize;
            if zi >= starts.len() { break; }
            let von = starts[zi];
            let bis = if zi + 1 < starts.len() { starts[zi + 1].saturating_sub(1) } else { chars.len() };
            let laenge = bis.saturating_sub(von) as i32;
            let a = von as i32 + ca.min(laenge);
            let b = von as i32 + cb.min(laenge);
            raus.push((b, a));   // (Marke, Anker): die Marke steht rechts
        }
        raus
    }

    /// Spaltenauswahl setzen (GUI_TEXTAREA_SELECT_COLUMNS), Zeilen und Spalten
    /// ab 1. Die erste Zeile traegt die fuehrende Marke, die uebrigen kommen
    /// als weitere dazu -- getippt wird danach in allen zugleich.
    pub fn textarea_select_columns(&mut self, h: i64, z1: i64, s1: i64, z2: i64, s2: i64)
                                   -> Result<i64, String> {
        let w = self.ta_wdg(h, "GUI_TEXTAREA_SELECT_COLUMNS")?;
        let chars: Vec<char> = w.text.chars().collect();
        let starts = Self::line_starts(&chars);
        let marken = Self::spalten_marken(&chars, &starts,
            (z1 - 1) as i32, (s1 - 1) as i32, (z2 - 1) as i32, (s2 - 1) as i32);
        if marken.is_empty() { return Err("GUI_TEXTAREA_SELECT_COLUMNS: leerer Bereich".into()); }
        let wd = self.wdg_mut(h, "GUI_TEXTAREA_SELECT_COLUMNS")?;
        wd.caret = marken[0].0;
        wd.sel_anchor = marken[0].1;
        wd.marken_zusatz = marken[1..].to_vec();
        Ok(marken.len() as i64)
    }

    /// Wie viele Einrueckungsstufen die (0-basierte) logische Zeile traegt.
    ///
    /// Eine LEERE Zeile nimmt die kleinere Tiefe ihrer beiden nicht-leeren
    /// Nachbarn -- sonst risse die Linie in jedem Absatz auf, und gerade dort
    /// will man sie sehen. Ein Tabulator zaehlt als eine volle Stufe.
    pub(super) fn ta_einzug_tiefe(chars: &[char], starts: &[usize], li: usize, breite: usize) -> usize {
        let stufen = |i: usize| -> Option<usize> {
            let von = starts[i];
            let bis = if i + 1 < starts.len() { starts[i + 1].saturating_sub(1) } else { chars.len() };
            let mut n = 0usize;
            for k in von..bis.min(chars.len()) {
                match chars[k] {
                    ' ' => n += 1,
                    '\t' => n += breite,
                    _ => return Some(n / breite),
                }
            }
            None   // nur Leerraum: die Zeile zaehlt als leer
        };
        if let Some(n) = stufen(li) { return n; }
        let mut vor = 0;
        for i in (0..li).rev() { if let Some(n) = stufen(i) { vor = n; break; } }
        let mut nach = 0;
        for i in li + 1..starts.len() { if let Some(n) = stufen(i) { nach = n; break; } }
        vor.min(nach)
    }

    /// Liegt die (1-basierte) Zeile in einem zugeklappten Block?
    ///
    /// Die Kopfzeile selbst bleibt sichtbar -- verborgen ist `von+1 ..= bis`.
    pub(super) fn zeile_verborgen(wdg: &Widget, zeile: usize) -> bool {
        wdg.gefaltet.iter().any(|&(v, b)| zeile > v && zeile <= b)
    }

    /// Der zugeklappte Block, in dem `zeile` liegt (Kopfzeile ausgenommen).
    pub(super) fn falte_um(wdg: &Widget, zeile: usize) -> Option<usize> {
        wdg.gefaltet.iter().position(|&(v, b)| zeile > v && zeile <= b)
    }

    /// Steht die Schreibmarke in verborgenem Text, klappt der Block auf.
    ///
    /// Pfeile und Klicks laufen ueber `ta_rows` und koennen gar nicht
    /// hineinkommen; Links/Rechts, Backspace und `GUI_TEXTAREA_GOTO`
    /// koennen es. Aufklappen statt Ausweichen, weil ein Sprung ueber den
    /// Block hinweg bei einem Suchtreffer das Falsche waere -- man will die
    /// Fundstelle sehen.
    pub(super) fn falte_am_caret_oeffnen(wdg: &mut Widget) {
        loop {
            let chars: Vec<char> = wdg.text.chars().collect();
            let starts = Self::line_starts(&chars);
            let c = wdg.caret.clamp(0, chars.len() as i32) as usize;
            let zeile = starts.iter().rposition(|&s| s <= c).unwrap_or(0) + 1;
            match Self::falte_um(wdg, zeile) {
                Some(k) => { wdg.gefaltet.remove(k); }
                None => break,
            }
        }
    }

    /// Die Faltungen einer Aenderung nachziehen.
    ///
    /// Eine Falte haengt an ZEILENNUMMERN, und eine Aenderung darueber
    /// verschiebt sie. Wer im Block selbst tippt, klappt ihn auf (dorthin
    /// kommt die Marke ohnehin nur aufgeklappt) -- das ist billiger als eine
    /// Buchfuehrung, die bei jedem Sonderfall daneben liegt.
    pub(super) fn falten_nachziehen(w: &mut Widget, vorher: &str, nachher: &str, caret_vorher: i32) {
        if w.gefaltet.is_empty() { return; }
        let alt = vorher.split('\n').count() as i64;
        let neu = nachher.split('\n').count() as i64;
        let delta = neu - alt;
        let zeile = vorher[..vorher.char_indices().nth(caret_vorher.max(0) as usize)
                            .map(|(b, _)| b).unwrap_or(vorher.len())]
                    .split('\n').count();
        let gesamt = neu as usize;
        w.gefaltet.retain_mut(|(v, b)| {
            if zeile < *v {
                let nv = *v as i64 + delta;
                let nb = *b as i64 + delta;
                if nv < 1 || nb <= nv { return false; }
                *v = nv as usize; *b = nb as usize;
            } else if zeile <= *b {
                return false;   // im Block getippt -> aufgeklappt
            }
            *b = (*b).min(gesamt);
            *b > *v
        });
    }

    /// Sichtbare Zeilen eines Textbereichs: ohne Umbruch eine je logischer
    /// Zeile; mit Umbruch an Wortgrenzen auf `breite` gebrochen -- ein Wort,
    /// das allein nicht passt, wird im Zeichen gebrochen. Jede Zeile als
    /// (logische Zeile, von, bis) in ZEICHEN, `bis` ohne den Umbruch.
    ///
    /// EINE Quelle fuer Zeichnen, Klick, Pfeile, Schreibmarke und Scroll --
    /// liefe eine davon anders, saesse die Marke neben dem Text.
    pub(super) fn ta_rows(&self, g: &Graphics, wdg: &Widget, chars: &[char], starts: &[usize], breite: i32) -> Vec<(usize, usize, usize)> {
        let mut rows = Vec::new();
        for (li, &ls) in starts.iter().enumerate() {
            // Zugeklappt: die Zeilen INNERHALB eines Blocks fallen hier weg,
            // und damit fuer alle auf einmal -- Zeichnen, Klick, Pfeile,
            // Schreibmarke und Scroll fragen dieselbe Liste. Die Kopfzeile
            // bleibt stehen und traegt beim Zeichnen die Sprechblase.
            if Self::zeile_verborgen(wdg, li + 1) { continue; }
            let le = if li + 1 < starts.len() { starts[li + 1] - 1 } else { chars.len() };
            if breite <= 0 || le <= ls { rows.push((li, ls, le)); continue; }
            let mut von = ls;
            while von < le {
                let mut bis = von;
                let mut k = von;
                // Wort fuer Wort anhaengen, solange es passt; gemessen wird
                // OHNE die Leerzeichen dahinter, die aber zur Zeile gehoeren.
                while k < le {
                    let mut e = k;
                    while e < le && chars[e] != ' ' { e += 1; }
                    let wort_ende = e;
                    while e < le && chars[e] == ' ' { e += 1; }
                    let stueck: String = chars[von..wort_ende].iter().collect();
                    if self.wtext_width(g, wdg, &stueck) > breite { break; }
                    bis = e;
                    k = e;
                }
                if bis == von {
                    // Kein ganzes Wort passt: zeichenweise, mindestens eins.
                    bis = von + 1;
                    while bis < le {
                        let stueck: String = chars[von..=bis].iter().collect();
                        if self.wtext_width(g, wdg, &stueck) > breite { break; }
                        bis += 1;
                    }
                }
                rows.push((li, von, bis));
                von = bis;
            }
        }
        rows
    }

    /// Zeile, in der die Schreibmarke steht: die letzte, deren Anfang nicht
    /// hinter ihr liegt -- am Umbruch gehoert die Marke damit zur NEUEN Zeile.
    /// Die mitlaufenden Blockkoepfe (GUI_TEXTAREA_SET "kopfzeilen"): die
    /// Kopfzeilen (logisch, ab 0) der faltbaren Bloecke, deren Kopf oben
    /// hinausgerollt ist, die aber in der ersten Zeile UNTER den Koepfen
    /// noch weitergehen -- aussen zuerst. Eine Quelle fuer Zeichnen, Klick,
    /// Nummernspalte und Mitziehen der Marke. Die Koepfe verdecken selbst
    /// Zeilen, darum wird wiederholt, bis sich die Liste nicht mehr aendert.
    /// Mindestens eine Zeile Code bleibt frei.
    pub(super) fn ta_koepfe(wdg: &Widget, rows: &[(usize, usize, usize)], sicht: i32, scroll: i32) -> Vec<usize> {
        if wdg.kopfzeilen <= 0 || wdg.faltbar.is_empty() || scroll <= 0 { return Vec::new(); }
        let n = (wdg.kopfzeilen as usize).min((sicht.max(1) as usize).saturating_sub(1));
        let mut liste: Vec<usize> = Vec::new();
        for _ in 0..=n {
            let ri = scroll as usize + liste.len();
            let Some(&(li, _, _)) = rows.get(ri) else { break };
            let oben = li + 1;
            let mut neu: Vec<usize> = wdg.faltbar.iter()
                .filter(|&&(v, b)| v < oben && b >= oben && !Self::zeile_verborgen(wdg, v))
                .map(|&(v, _)| v - 1).collect();
            neu.truncate(n);
            if neu == liste { break; }
            liste = neu;
        }
        liste
    }

    pub(super) fn ta_row_of(rows: &[(usize, usize, usize)], caret: usize) -> usize {
        rows.iter().rposition(|&(_, von, _)| von <= caret).unwrap_or(0)
    }

    /// Welcher Ausschnitt ist gerade zu sehen? (GUI_TEXTAREA_VIEW)
    ///
    /// Liefert `(erste_zeile, zeilen, start_zeichen, laenge_zeichen)` -- die
    /// ersten beiden in ZEILEN, die letzten beiden in ZEICHEN.
    ///
    /// Wozu: Wer eine grosse Datei bei jedem Tastendruck GANZ einfaerbt,
    /// zahlt mit der Dateigroesse (gemessen: 3000 Zeilen ~26 ms, 30000 Zeilen
    /// ~270 ms je Anschlag). Mit diesem Ausschnitt faerbt ein Programm nur,
    /// was man sieht -- und die Kosten haengen nur noch von der Fenstergroesse
    /// ab. Dass das RICHTIG ist und nicht bloss schneller, liegt an der
    /// Sprache: Kommentare und Zeichenketten enden in Drachenhauch an der
    /// Zeile, ein Ausschnitt an Zeilengrenzen kann also nicht in der Mitte
    /// eines Gebildes anfangen.
    ///
    /// Die Zeichen-Angaben zaehlen wie ueberall ZEICHEN, nicht Bytes.
    pub fn textarea_view(&self, g: &Graphics, h: i64) -> Result<(i64, i64, i64, i64), String> {
        let wd = self.wdg(h, "GUI_TEXTAREA_VIEW")?;
        if wd.kind != Kind::TextArea {
            return Err("GUI_TEXTAREA_VIEW: das Widget ist kein GUI_TEXTAREA".into());
        }
        let z = self.ta_zerlegt(h, &wd.text);
        let (chars, starts) = (&z.0, &z.1);
        // Dieselbe Rechnung wie beim Zeichnen (pad = 5, ta_line_h) -- liefe
        // sie auseinander, faerbte das Programm einen anderen Ausschnitt, als
        // zu sehen ist.
        let lh = self.ta_line_h(g, wd);
        let sicht = ((wd.h - 2 * 5) / lh).max(1);
        let rows = self.ta_rows(g, wd, &chars, &starts, self.ta_breite(g, wd, starts.len()));
        let erste_row = wd.scroll.clamp(0, (rows.len() as i32 - 1).max(0)) as usize;
        let letzte_row = (erste_row + sicht as usize).min(rows.len());
        // Nach aussen zaehlen weiter LOGISCHE Zeilen -- ein Programm, das den
        // Ausschnitt einfaerbt, kennt keine umgebrochenen.
        let erste = rows[erste_row].0;
        let letzte = rows[letzte_row - 1].0 + 1;
        // Ohne Umbruch bleibt `zeilen` die Zahl der sichtbaren PLAETZE (so
        // war es immer, und Programme rechnen damit); mit Umbruch ist es die
        // Zahl der logischen Zeilen, die im Ausschnitt liegen.
        let zeilen = if wd.umbruch { (letzte - erste).max(1) } else { sicht as usize };
        let von = starts[erste];
        let bis = if letzte < starts.len() { starts[letzte].saturating_sub(1) } else { chars.len() };
        Ok((erste as i64, zeilen as i64, von as i64, bis.saturating_sub(von) as i64))
    }
}
