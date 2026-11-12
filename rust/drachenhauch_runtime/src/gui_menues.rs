//! Abfragen und Setzer, Menues, Stil je Widget, Geometrie und Lebenslauf zur Laufzeit, Text- und Formularbefehle.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Polling / Setter ---
    pub fn clicked(&self, h: i64) -> Result<bool, String> {
        // Menue-Item-Handle? -> dessen clicked-Flag.
        if h & ITEM_FLAG != 0 {
            let (wi, mi, ii) = dec_item(h);
            return Ok(self.windows.get(wi).and_then(|w| w.menus.get(mi))
                .and_then(|m| m.items.get(ii)).map(|it| it.clicked).unwrap_or(false));
        }
        Ok(self.wdg(h, "GUI_CLICKED")?.clicked)
    }

    // --- Menues -----------------------------------------------------------
    pub(super) fn add_menu_impl(&mut self, win: i64, in_bar: bool, label: String, fn_: &str) -> Result<i64, String> {
        let wi = win as usize;
        let w = self.windows.get_mut(wi).ok_or_else(|| format!("{}: erwartet GUI_WINDOW", fn_))?;
        let mi = w.menus.len();
        w.menus.push(Menu { label, in_bar, unter: false, items: Vec::new(), auf_t: 0.0, b: Blende::default(), ziel: -1 });
        Ok(enc_menu(wi, mi))
    }
    /// Top-Level-Menue in der Menueleiste (z.B. "Datei").
    pub fn add_menu(&mut self, win: i64, label: String) -> Result<i64, String> {
        self.add_menu_impl(win, true, label, "GUI_MENU")
    }
    /// Kontextmenue (per Rechtsklick im Fenster). Label wird nicht angezeigt.
    pub fn add_context(&mut self, win: i64) -> Result<i64, String> {
        self.add_menu_impl(win, false, String::new(), "GUI_CONTEXT")
    }
    pub(super) fn menu_mut(&mut self, h: i64, fn_: &str) -> Result<&mut Menu, String> {
        if h & MENU_FLAG == 0 || h & ITEM_FLAG != 0 { return Err(format!("{}: erwartet GUI_MENU/GUI_CONTEXT", fn_)); }
        let (wi, mi) = dec_menu(h);
        self.windows.get_mut(wi).and_then(|w| w.menus.get_mut(mi))
            .ok_or_else(|| format!("{}: ungueltiges Menue-Handle", fn_))
    }
    /// Eintrag an ein Menue anhaengen -> Item-Handle (fuer GUI_CLICKED).
    pub fn add_menu_item(&mut self, menu: i64, label: String) -> Result<i64, String> {
        let (wi, mi) = dec_menu(menu);
        let m = self.menu_mut(menu, "GUI_MENU_ITEM")?;
        let ii = m.items.len();
        m.items.push(MenuItem::neu(label, false));
        Ok(enc_item(wi, mi, ii))
    }
    /// Trennlinie an ein Menue anhaengen.
    pub fn add_menu_separator(&mut self, menu: i64) -> Result<(), String> {
        let m = self.menu_mut(menu, "GUI_MENU_SEPARATOR")?;
        m.items.push(MenuItem::neu(String::new(), true));
        Ok(())
    }

    // ---- Menue-Ausbau (Punkt 2, 2026-09-04) ------------------------------

    /// GUI_SUBMENU(menu, label$) -> Menue-Handle. Der Eintrag im Elternmenue
    /// oeffnet es beim Ueberfahren; Eintraege haengt man wie ueberall mit
    /// GUI_MENU_ITEM an. Beliebig tief.
    pub fn add_submenu(&mut self, menu: i64, label: String) -> Result<i64, String> {
        let (wi, mi) = dec_menu(menu);
        self.menu_mut(menu, "GUI_SUBMENU")?;
        let w = &mut self.windows[wi];
        let smi = w.menus.len();
        w.menus.push(Menu { label: label.clone(), in_bar: false, unter: true, items: Vec::new(), auf_t: 0.0, b: Blende::default(), ziel: -1 });
        let mut it = MenuItem::neu(label, false);
        it.sub = smi as i32;
        w.menus[mi].items.push(it);
        Ok(enc_menu(wi, smi))
    }
    pub(super) fn item_mut(&mut self, h: i64, fn_: &str) -> Result<&mut MenuItem, String> {
        if h & ITEM_FLAG == 0 { return Err(format!("{}: erwartet einen Menue-Eintrag (von GUI_MENU_ITEM)", fn_)); }
        let (wi, mi, ii) = dec_item(h);
        self.windows.get_mut(wi).and_then(|w| w.menus.get_mut(mi)).and_then(|m| m.items.get_mut(ii))
            .ok_or_else(|| format!("{}: ungueltiges Eintrags-Handle", fn_))
    }
    /// GUI_MENU_SHORTCUT(item, kuerzel$) -- "" entfernt es. Angezeigt wird der
    /// Text, wie er angegeben wurde; ausgeloest wird ueber die geparste Form.
    pub fn menu_shortcut(&mut self, h: i64, kuerzel: &str) -> Result<(), String> {
        let (mods, code) = if kuerzel.trim().is_empty() { (0, 0) }
                           else { kuerzel_parsen(kuerzel).map_err(|e| format!("GUI_MENU_SHORTCUT: {}", e))? };
        let it = self.item_mut(h, "GUI_MENU_SHORTCUT")?;
        it.kuerzel = kuerzel.trim().to_string();
        it.k_mods = mods; it.k_code = code;
        Ok(())
    }
    /// GUI_MENU_ENABLE(item, an) -- ein gesperrter Eintrag ist grau, nimmt
    /// keinen Klick und kein Kuerzel an.
    pub fn menu_enable(&mut self, h: i64, an: bool) -> Result<(), String> {
        let it = self.item_mut(h, "GUI_MENU_ENABLE")?;
        if it.separator { return Err("GUI_MENU_ENABLE: eine Trennlinie laesst sich nicht schalten".into()); }
        it.enabled = an; Ok(())
    }
    /// GUI_MENU_CHECK(item, an) -- macht den Eintrag zum Haekchen-Eintrag und
    /// setzt den Zustand. Ein Klick (oder das Kuerzel) kippt ihn danach selbst.
    pub fn menu_check(&mut self, h: i64, an: bool) -> Result<(), String> {
        let it = self.item_mut(h, "GUI_MENU_CHECK")?;
        it.haken = true; it.an = an; Ok(())
    }
    /// GUI_MENU_CHECKED(item) -> BOOLEAN
    pub fn menu_checked(&self, h: i64) -> Result<bool, String> {
        if h & ITEM_FLAG == 0 { return Err("GUI_MENU_CHECKED: erwartet einen Menue-Eintrag".into()); }
        let (wi, mi, ii) = dec_item(h);
        self.windows.get(wi).and_then(|w| w.menus.get(mi)).and_then(|m| m.items.get(ii))
            .map(|it| it.an).ok_or_else(|| "GUI_MENU_CHECKED: ungueltiges Eintrags-Handle".into())
    }
    /// GUI_MENU_ICON(item, bild) -- Textur-Handle wie bei GUI_ICON_BUTTON, -1 entfernt.
    pub fn menu_icon(&mut self, h: i64, tex: i64) -> Result<(), String> {
        let it = self.item_mut(h, "GUI_MENU_ICON")?;
        it.icon = tex.max(-1); it.sym.clear(); Ok(())
    }
    /// GUI_MENU_ICON(item, "sichern") -- ein eingebautes Sinnbild, dasselbe
    /// wie in der Werkzeugleiste. So sehen Menue und Leiste gleich aus, ohne
    /// dass das Programm Bilder malen muss.
    pub fn menu_icon_symbol(&mut self, h: i64, name: &str) -> Result<(), String> {
        let n = if name.is_empty() { String::new() } else { sinnbild_pruefen(name, "GUI_MENU_ICON")? };
        let it = self.item_mut(h, "GUI_MENU_ICON")?;
        it.sym = n; it.icon = -1; Ok(())
    }
    /// GUI_MENU_TEXT(item, label$) -- Beschriftung aendern ("Pause" / "Weiter").
    pub fn menu_text(&mut self, h: i64, label: String) -> Result<(), String> {
        let it = self.item_mut(h, "GUI_MENU_TEXT")?;
        it.label = label; Ok(())
    }

    /// Kuerzel pruefen -- jedes Bild. Zuerst im Fenster mit Fokus, dann in
    /// allen anderen sichtbaren Fenstern von oben nach unten: ein Programm
    /// mit Werkzeugleiste links und Inspektor rechts hat EIN Menue, und
    /// Strg+S soll auch dann sichern, wenn der letzte Klick in den Inspektor
    /// ging. Drei Piloten (Form-Designer, Anim-FSM, Notenblatt) mussten sich
    /// vorher nach jedem Knopf den Fokus zurueckholen, sonst war Strg+S stumm
    /// -- und jedes Mal sah es erst der Test. Hat das Fokus-Fenster dasselbe
    /// Kuerzel selbst, gewinnt es (es steht vorn in der Reihe). Ein modales
    /// Fenster laesst nur seine eigenen zu, ein Fenster im Entwurfsmodus
    /// (`GUI_WINDOW_DESIGN`) zaehlt nicht: seine Menues sind Ansicht, nicht
    /// Bedienung. Ohne Strg/Alt gehoert eine Taste dem Textfeld mit Fokus
    /// (ein "Entf" darf dort loeschen, nicht den Menuepunkt ausloesen).
    pub(super) fn kuerzel_pruefen(&mut self, g: &Graphics) {
        let mods = (g.key_ctrl() as u8) | ((g.key_shift() as u8) << 1) | ((g.key_alt() as u8) << 2);
        let fokus_w = self.focus_widget
            .and_then(|(w, i)| self.windows.get(w).and_then(|win| win.widgets.get(i)));
        let kandidaten: Vec<usize> = self.focus_window.into_iter()
            .chain(self.z_order.iter().rev().copied())
            .filter(|&wi| wi < self.windows.len())
            .collect();
        let mut gesehen: Vec<usize> = Vec::new();
        let mut treffer: Option<(usize, usize, usize)> = None;
        'suche: for wi in kandidaten {
            if gesehen.contains(&wi) { continue; }
            gesehen.push(wi);
            let win = &self.windows[wi];
            if !win.alive || !win.visible || win.entwurf { continue; }
            if self.modal.is_some_and(|m| m != wi) { continue; }
            for (mi, m) in win.menus.iter().enumerate() {
                for (ii, it) in m.items.iter().enumerate() {
                    if it.k_code == 0 || !it.enabled || it.separator || it.k_mods != mods { continue; }
                    // Ohne Strg/Alt gehoert die Taste dem Widget mit Fokus, wenn es
                    // sie braucht (Entf loescht im Textfeld ein Zeichen, die
                    // Leertaste kippt ein Kaestchen) -- AUSSER F1..F12: die
                    // erzeugen nie Text, und F5 in einer IDE muss aus dem
                    // Code-Feld heraus starten.
                    if (mods & 5) == 0 && !ist_funktionstaste(it.k_code)
                        && fokus_w.is_some_and(|w| widget_nimmt_taste(w, it.k_code)) { continue; }
                    if kuerzel_taste(g, it.k_code) { treffer = Some((wi, mi, ii)); break 'suche; }
                }
            }
        }
        if let Some((wi, mi, ii)) = treffer {
            self.open_menu = None; self.context_open = None; self.sub_chain.clear();
            // Die Taste ist verbraucht. Ohne diesen Merker taete sie ZWEI
            // Dinge: `Strg+Umschalt+Hoch` feuerte das Kuerzel UND schoebe im
            // Code-Feld die Auswahl eine Zeile hoch -- der Befehl bekaeme
            // dann eine andere Auswahl zu sehen, als der Nutzer markiert hat.
            self.kuerzel_gefeuert = true;
            self.fire_menu_item(wi, mi, ii);
        }
    }

    /// Alle offenen Popups von der Wurzel abwaerts: (Fenster, Menue, x, y).
    pub(super) fn popup_chain(&self, g: &Graphics) -> Vec<(usize, usize, i32, i32)> {
        let mut out = Vec::new();
        if let Some((wi, mi, cx, cy)) = self.context_open {
            // Ein Kontextmenue am Rand weicht aus: ueber der Maus statt
            // darunter, links von ihr statt rechts -- sonst lief es aus dem
            // Bild, und die unteren Eintraege waren nicht zu erreichen.
            let (w, h) = self.popup_size(g, wi, mi);
            let (sb, sh) = self.schirm();
            let px = if cx + w > sb && cx - w >= 0 { cx - w } else { cx.min(sb - w).max(0) };
            let py = if cy + h > sh && cy - h >= 0 { cy - h } else { cy.min(sh - h).max(0) };
            out.push((wi, mi, px, py));
        } else if let Some((wi, mi)) = self.open_menu {
            let toff = (if self.windows[wi].chrome { self.m("title_h") } else { 0 }) + self.sk(MENUBAR_H);
            if let Some((_, x0, _)) = self.menubar_slots(g, wi).into_iter().find(|(m, _, _)| *m == mi) {
                let (w, _) = self.popup_size(g, wi, mi);
                let (sb, _) = self.schirm();
                out.push((wi, mi, x0.min(sb - w).max(0), self.windows[wi].y + toff));
            }
        }
        if let Some(&(wi, _, _, _)) = out.first() {
            for &(smi, px, py) in &self.sub_chain {
                // Ein Untermenue, das rechts nicht passt, klappt LINKS vom
                // Elternmenue auf; unten schiebt es sich hoch.
                let (w, h) = self.popup_size(g, wi, smi);
                let (sb, sh) = self.schirm();
                let eltern_x = out.last().map(|e| e.2).unwrap_or(px);
                let px = if px + w > sb { (eltern_x - w + 4).max(0) } else { px };
                let py = py.min(sh - h).max(0);
                out.push((wi, smi, px, py));
            }
        }
        out
    }
    /// Bildschirmmasse aus dem letzten GUI_UPDATE (vorher: unbegrenzt).
    pub(super) fn schirm(&self) -> (i32, i32) {
        let b = if self.schirm_b > 0 { self.schirm_b } else { i32::MAX / 4 };
        let h = if self.schirm_h > 0 { self.schirm_h } else { i32::MAX / 4 };
        (b, h)
    }

    /// Untermenues folgen der Maus: ueber einem Eintrag mit Untermenue geht
    /// es auf, ueber einem anderen Eintrag derselben Ebene geht die tiefere
    /// Kette zu. Ausserhalb aller Popups aendert sich nichts -- sonst
    /// klappte das Untermenue zu, waehrend man schraeg hinueberfaehrt.
    pub(super) fn untermenues_folgen(&mut self, g: &Graphics, mx: i32, my: i32) {
        if self.open_menu.is_none() && self.context_open.is_none() { self.sub_chain.clear(); return; }
        let chain = self.popup_chain(g);
        for (level, &(wi, mi, px, py)) in chain.iter().enumerate().rev() {
            let (wmax, h) = self.popup_size(g, wi, mi);
            if !(mx >= px && mx < px + wmax && my >= py && my < py + h) { continue; }
            let zeile = ((my - py - 2) / self.sk(MENU_ITEM_H)).max(0) as usize;
            let (sub, iy) = match self.windows[wi].menus[mi].items.get(zeile) {
                Some(it) if !it.separator && it.enabled => (it.sub, py + 2 + zeile as i32 * self.sk(MENU_ITEM_H)),
                _ => return,
            };
            self.sub_chain.truncate(level);
            if sub >= 0 {
                self.sub_chain.push((sub as usize, px + wmax - 4, iy - 2));
            }
            return;
        }
    }
    pub fn hovered(&self, h: i64) -> Result<bool, String> { Ok(self.wdg(h, "GUI_HOVERED")?.hovered) }
    pub fn checked(&self, h: i64) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_CHECKED")?;
        if !matches!(w.kind, Kind::Checkbox | Kind::Radio | Kind::Toggle) { return Err("GUI_CHECKED: Widget ist keine checkbox/radio/toggle".into()); }
        Ok(w.checked)
    }
    pub fn value(&self, h: i64) -> Result<f64, String> {
        let w = self.wdg(h, "GUI_VALUE")?;
        match w.kind {
            Kind::Slider | Kind::Progress | Kind::Spinner | Kind::Knob => Ok(w.value),
            Kind::Splitter => Ok(if w.group == "v" { w.x as f64 } else { w.y as f64 }),
            _ => Err("GUI_VALUE: Widget ist kein slider/progress/spinner/splitter/knob".into()),
        }
    }
    pub fn text(&self, h: i64) -> Result<String, String> {
        let w = self.wdg(h, "GUI_TEXT")?;
        // Eine Klappliste ohne Tippen zeigt ihre Auswahl -- GUI_TEXT liefert
        // dasselbe. Vorher kam der zuletzt mit GUI_SET_TEXT gesetzte Text,
        // auch nachdem der Nutzer etwas anderes gewaehlt hatte
        // (Kuehlschrank-App, Standard-Einheit in den Einstellungen).
        if w.kind == Kind::Dropdown && !w.frei {
            return Ok(if w.sel >= 0 && (w.sel as usize) < w.items.len() { w.items[w.sel as usize].clone() } else { String::new() });
        }
        Ok(w.text.clone())
    }
    pub fn set_text(&mut self, h: i64, t: String) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_TEXT")?;
        let n = t.chars().count() as i32;
        w.text = t;
        // Die Abschnitte gehoerten zum ALTEN Text. Sie stehen zu lassen faerbte
        // den neuen nach den Positionen des alten -- besser sichtbar farblos
        // als sichtbar falsch.
        w.spans.clear();
        w.wellen.clear(); w.hinweise.clear(); w.hinweise_farben.clear();
        // Der Verlauf gehoerte zum alten Text -- ein Strg+Z danach brachte
        // sonst etwas zurueck, das der Nutzer nie getippt hat.
        w.undo.clear(); w.redo.clear();
        // Die Faltung ebenso: sie haengt an Zeilennummern des alten Textes,
        // und die decken sich mit dem neuen nur zufaellig. Die Farbfelder
        // haengen an Zeichenstellen -- dasselbe Argument.
        w.gefaltet.clear(); w.faltbar.clear(); w.farbfelder.clear();
        // Die Stile ebenso -- ein neuer Text ist ungeformt; wer ihn geformt
        // setzen will, nimmt GUI_TEXTAREA_SET_MARKDOWN.
        if w.formatiert { w.stile = vec![0; n as usize]; w.stile_text = w.text.clone(); w.tipp_stil = None; }
        // Caret ans Ende, Selektion/Scroll zuruecksetzen (sonst zeigt das Caret
        // hinter das Ende des nun kuerzeren Textes).
        w.caret = n; w.sel_anchor = n; w.scroll = 0;
        // Eine Klappliste waehlt den Eintrag, der so heisst (Gross/klein egal),
        // sonst keinen -- ohne Tippen gibt es nichts anderes zu zeigen. Vorher
        // stand der Text nur im Widget: die Box zeigte weiter die alte Auswahl.
        if w.kind == Kind::Dropdown { dd_frei_abgleichen(w); }
        // Beim gesetzten Text ist der Text die QUELLE -- sonst setzte
        // GUI_SET_TEXT dort etwas, das nie zu sehen ist.
        // Bei der Statusleiste ist der Text das ERSTE Feld -- ein Programm, das
        // bisher eine Beschriftung als Statuszeile nahm, muss so nichts
        // umschreiben.
        if w.kind == Kind::StatusBar {
            let t = w.text.clone();
            if let Some(f) = w.status.as_mut().and_then(|s| s.felder.first_mut()) { f.text = t; }
        }
        if w.kind == Kind::RichText {
            let t = w.text.clone();
            if let Some(r) = w.rich.as_mut() {
                r.quelle = t; r.scroll = 0; r.stand = (usize::MAX, 0, 0, 0);
                r.hat_auswahl = false; r.ziehen = false;
            }
        }
        Ok(())
    }
    pub fn set_tooltip(&mut self, h: i64, t: String) -> Result<(), String> {
        self.wdg_mut(h, "GUI_TOOLTIP")?.tooltip = t; Ok(())
    }
    /// GUI_TOOLTIP$(w): der mit GUI_TOOLTIP gesetzte Hilfetext.
    pub fn tooltip_text(&self, h: i64) -> Result<String, String> {
        Ok(self.wdg(h, "GUI_TOOLTIP$")?.tooltip.clone())
    }
    pub fn set_checked(&mut self, h: i64, f: bool) -> Result<(), String> {
        let kind = self.wdg(h, "GUI_SET_CHECKED")?.kind;
        if !matches!(kind, Kind::Checkbox | Kind::Radio | Kind::Toggle) { return Err("GUI_SET_CHECKED: Widget ist keine checkbox/radio/toggle".into()); }
        {
            let w = self.wdg_mut(h, "GUI_SET_CHECKED")?;
            w.checked = f;
        }
        // Radio: beim Setzen die Gruppen-Geschwister deselektieren.
        if kind == Kind::Radio && f {
            let (wi, i) = Self::dec_widget(h);
            self.select_radio(wi, i);
        }
        Ok(())
    }
    pub fn set_value(&mut self, h: i64, v: f64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_SET_VALUE")?;
        match w.kind {
            Kind::Slider | Kind::Progress | Kind::Spinner | Kind::Knob => { w.value = v.clamp(w.min, w.max); }
            Kind::Splitter => {
                let c = (v.round() as i32).clamp(w.min as i32, w.max as i32);
                if w.group == "v" { w.x = c; } else { w.y = c; }
            }
            _ => return Err("GUI_SET_VALUE: Widget ist kein slider/progress/spinner/splitter".into()),
        }
        Ok(())
    }
    pub fn on_click(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_CLICK")?.on_click = func; Ok(())
    }

    /// Maus betritt das Widget (einmal je Eintritt, nicht jedes Bild).
    pub fn on_hover(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_HOVER")?.on_hover = func; Ok(())
    }
    /// Maus verlaesst das Widget.
    pub fn on_leave(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_LEAVE")?.on_leave = func; Ok(())
    }
    /// Widget bekommt die Eingabe (Textfeld angeklickt, Tab-Wechsel).
    pub fn on_focus(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_FOCUS")?.on_focus = func; Ok(())
    }
    /// Widget verliert die Eingabe -- der Punkt, an dem man eine Eingabe
    /// pruefen will.
    pub fn on_blur(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        self.wdg_mut(h, "GUI_ON_BLUR")?.on_blur = func; Ok(())
    }
    pub fn on_change(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_ON_CHANGE")?;
        if !matches!(w.kind, Kind::Slider | Kind::TextInput | Kind::TextArea | Kind::Checkbox | Kind::Table | Kind::Radio | Kind::Dropdown | Kind::ListBox | Kind::Spinner | Kind::Splitter | Kind::Tree | Kind::TabControl | Kind::RichText | Kind::TimePicker | Kind::Accordion | Kind::Wizard) {
            return Err("GUI_ON_CHANGE: nur fuer slider, textinput, textarea, checkbox, table, radio, dropdown, listbox, spinner, splitter oder tree".into());
        }
        w.on_change = func; Ok(())
    }
    /// Widget rund zeichnen. Gedacht fuer Knoepfe, die nur ein Sinnbild
    /// tragen (Wiedergabe, Pause, Weiter) -- ein runder Knopf mit Dreieck
    /// darin liest sich sofort als Abspieltaste.
    pub fn set_round(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_ROUND")?.rund = f;
        Ok(())
    }
    /// Grafik fuer eine Widget-Art hinterlegen (9-Slice). `bild` = -1 nimmt
    /// sie wieder weg.
    pub fn set_skin(&mut self, art: String, bild: i64, rand: i32) -> Result<(), String> {
        let a = art.to_lowercase();
        if Kind::from_str(&a).is_none() {
            return Err(format!("GUI_SKIN: unbekannte Widget-Art '{}'", art));
        }
        if bild < 0 {
            self.skins.remove(&a);
        } else {
            self.skins.insert(a, (bild, rand.max(0)));
        }
        Ok(())
    }

    /// Hinterlegte Grafik einer Art (falls vorhanden).
    pub(super) fn skin_of(&self, kind: Kind) -> Option<(i64, i32)> {
        self.skins.get(kind.as_str()).copied()
    }

    pub fn set_color(&mut self, h: i64, role: String, color: i64) -> Result<(), String> {
        if !matches!(role.as_str(), "bg" | "fg" | "border" | "accent" | "hover" | "pressed") {
            return Err("GUI_SET_COLOR: role muss bg/fg/border/accent/hover/pressed sein".into());
        }
        let w = self.wdg_mut(h, "GUI_SET_COLOR")?;
        if color == -1 { w.ov.remove(&role); } else { w.ov.insert(role, color); }
        Ok(())
    }
    // --- Styling (Phase 4): enabled-Zustand, per-Widget-Font/-Groesse ---
    pub fn set_enabled(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_ENABLED")?.enabled = f;
        if !f {
            let (wi, i) = Self::dec_widget(h);
            if self.focus_widget == Some((wi, i)) { self.focus_widget = None; }
            if self.open_dropdown == Some((wi, i)) { self.open_dropdown = None; }
            // Review-Fund: press_origin/active_slider/active_split fehlten --
            // ein Slider mitten im Drag deaktiviert feuerte weiter
            // GUI_ON_CHANGE (drag_slider haelt sich nicht an `enabled`), und
            // ein zwischen Press und Release deaktivierter Button loeste
            // beim Loslassen trotzdem noch GUI_ON_CLICK aus.
            if self.press_origin == Some((wi, i)) { self.press_origin = None; }
            if self.cp_drag.map(|(w, x, _)| (w, x)) == Some((wi, i)) { self.cp_drag = None; }
            if self.active_slider == Some((wi, i)) { self.active_slider = None; }
        if self.drag.as_ref().map(|d| d.quelle) == Some((wi, i)) { self.drag = None; }
        if self.active_knob.map(|(w, k, _, _)| (w, k)) == Some((wi, i)) { self.active_knob = None; }
            if self.active_knob.map(|(w, k, _, _)| (w, k)) == Some((wi, i)) { self.active_knob = None; }
            if self.active_split == Some((wi, i)) { self.active_split = None; }
        }
        Ok(())
    }
    pub fn enabled(&self, h: i64) -> Result<bool, String> { Ok(self.wdg(h, "GUI_ENABLED")?.enabled) }
    pub fn set_font(&mut self, h: i64, font: i64) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_FONT")?.font = font; Ok(())
    }
    /// GUI_SET_FONT_STYLE(wdg, stil$): fett, kursiv, unterstrichen,
    /// durchgestrichen -- verbunden mit `+`. Gilt fuer den Text des Widgets.
    pub fn set_font_style(&mut self, h: i64, stil: &str) -> Result<(), String> {
        let bits = crate::schnitt::stil_parsen(stil).map_err(|e| format!("GUI_SET_FONT_STYLE: {}", e))?;
        self.wdg_mut(h, "GUI_SET_FONT_STYLE")?.stil = bits; Ok(())
    }
    pub fn font_style(&self, h: i64) -> Result<String, String> {
        Ok(crate::schnitt::stil_text(self.wdg(h, "GUI_GET_FONT_STYLE$")?.stil))
    }
    pub fn set_font_size(&mut self, h: i64, sz: i64) -> Result<(), String> {
        if sz < 0 { return Err("GUI_SET_FONT_SIZE: Groesse muss >= 0 sein".into()); }
        self.wdg_mut(h, "GUI_SET_FONT_SIZE")?.font_size = sz as i32; Ok(())
    }
    /// Anchoring eines Widgets setzen (Edge-String aus "lrtb", z.B. "lrtb",
    /// "rb"). Die Anchor-Basis ist das aktuelle Rechteck des Widgets.
    pub fn set_anchor(&mut self, h: i64, s: &str) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_ANCHOR")?.anchor = Self::anchor_mask(s); Ok(())
    }
    /// Benannten Style anlegen/erweitern: `prop` in bg/fg/border/accent (Farbe),
    /// font (Handle), font_size (px). Inkrementell aufrufbar.
    pub fn style_set(&mut self, name: String, prop: String, value: i64) -> Result<(), String> {
        if !matches!(prop.as_str(), "bg" | "fg" | "border" | "accent" | "hover" | "pressed" | "font" | "font_size") {
            return Err("GUI_STYLE_SET: prop muss bg/fg/border/accent/hover/pressed/font/font_size sein".into());
        }
        if prop == "font_size" && value < 0 { return Err("GUI_STYLE_SET: font_size muss >= 0 sein".into()); }
        self.styles.entry(name).or_default().insert(prop, value);
        Ok(())
    }
    /// Einen benannten Style auf ein Widget anwenden (ueberschreibt dessen
    /// Farb-Overrides/Font). Wirft, wenn der Style unbekannt ist.
    pub fn apply_style(&mut self, h: i64, name: &str) -> Result<(), String> {
        let props = self.styles.get(name)
            .ok_or_else(|| format!("GUI_APPLY_STYLE: unbekannter Style '{}'", name))?
            .clone();
        let w = self.wdg_mut(h, "GUI_APPLY_STYLE")?;
        for (k, v) in props {
            match k.as_str() {
                "font" => w.font = v,
                "font_size" => w.font_size = v as i32,
                _ => { w.ov.insert(k, v); }   // bg/fg/border/accent
            }
        }
        Ok(())
    }

    // --- Laufzeit-Manipulation (Geometrie / Lifecycle / Hit-Test) ---
    // Basis fuer dynamische UIs und einen WYSIWYG-Editor. Widget-Koordinaten sind
    // fenster-relativ (wie bei der Konstruktion).
    pub fn set_bounds(&mut self, h: i64, x: i32, y: i32, w: i32, ht: i32) -> Result<(), String> {
        let (x, y, w, ht) = (self.sk(x), self.sk(y), self.sk(w), self.sk(ht));
        let wd = self.wdg_mut(h, "GUI_SET_BOUNDS")?;
        wd.x = x; wd.y = y; wd.w = w.max(0); wd.h = ht.max(0);
        wd.nat_w = wd.w; wd.nat_h = wd.h;       // das ist jetzt die natuerliche Groesse
        wd.auto_w = false; wd.auto_h = false;   // eine gesetzte Groesse ist keine automatische mehr
        Ok(())
    }
    pub fn widget_bounds(&self, h: i64, fn_: &str) -> Result<(i32, i32, i32, i32), String> {
        let w = self.wdg(h, fn_)?;
        Ok((self.unsk(w.x), self.unsk(w.y), self.unsk(w.w), self.unsk(w.h)))
    }
    pub fn destroy(&mut self, h: i64) -> Result<(), String> {
        let (wi, i) = Self::dec_widget(h);
        let wd = self.wdg_mut(h, "GUI_DESTROY")?;
        wd.alive = false;
        // Haengende Interaktions-Referenzen auf dieses Widget loesen.
        // Review-Fund: active_table/table_press/hover_w fehlten hier -- ein
        // GUI_DESTROY waehrend eines Table-Scrollbar-Drags oder mit noch
        // gedrueckter Tabellenzeile liess GUI_TABLE_SELECTED/GUI_ON_CHANGE
        // weiter auf das (tombstoned) Widget wirken, und ein laufendes
        // Scrollbar-Drag konnte weiterhin table_drag() (und dessen
        // clamp-Panic-Risiko) fuer das zerstoerte Widget ausloesen.
        if self.focus_widget == Some((wi, i)) { self.focus_widget = None; }
        if self.active_slider == Some((wi, i)) { self.active_slider = None; }
        if self.drag.as_ref().map(|d| d.quelle) == Some((wi, i)) { self.drag = None; }
        if self.active_split == Some((wi, i)) { self.active_split = None; }
        if self.press_origin == Some((wi, i)) { self.press_origin = None; }
        if self.cp_drag.map(|(w, x, _)| (w, x)) == Some((wi, i)) { self.cp_drag = None; }
        if self.open_dropdown == Some((wi, i)) { self.open_dropdown = None; }
        if self.active_table == Some((wi, i)) { self.active_table = None; }
        if self.table_press.map(|(tw, ti, _)| (tw, ti)) == Some((wi, i)) { self.table_press = None; }
        if self.editing_table == Some((wi, i)) { self.editing_table = None; }
        if let Some(t) = self.windows[wi].widgets[i].tbl.as_mut() { t.head_drag = None; }
        if self.hover_w == Some((wi, i)) { self.hover_w = None; }
        Ok(())
    }
    pub fn set_widget_visible(&mut self, h: i64, f: bool) -> Result<(), String> {
        self.wdg_mut(h, "GUI_SET_VISIBLE")?.visible = f; Ok(())
    }
    pub fn widget_visible(&self, h: i64) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_VISIBLE")?; Ok(w.alive && w.visible)
    }
    /// GUI_SHOWN: ist das Widget jetzt zu sehen? Anders als GUI_VISIBLE zaehlen
    /// das Fenster, die Seite des Fensters und jeder Reiter-/Akkordeon-Behaelter
    /// darueber mit -- eine Zeichenflaeche auf einer verdeckten Reiterseite
    /// soll nicht zeichnen.
    pub fn widget_gezeigt(&self, h: i64) -> Result<bool, String> {
        let mut w = self.wdg(h, "GUI_SHOWN")?;
        let (wi, _) = Self::dec_widget(h);
        let win = &self.windows[wi];
        if !win.alive || !win.visible { return Ok(false); }
        for _ in 0..64 {
            if !self.widget_shown(wi, w) { return Ok(false); }
            if w.tc_von < 0 { return Ok(true); }
            match win.widgets.get(w.tc_von as usize) { Some(tc) => w = tc, None => return Ok(true) }
        }
        Ok(true)
    }
    pub fn kind_name(&self, h: i64) -> Result<&'static str, String> {
        Ok(self.wdg(h, "GUI_KIND")?.kind.as_str())
    }
    pub fn focus(&mut self, h: i64) -> Result<(), String> {
        // GUI_FOCUS(-1): niemand hat den Fokus -- die Tasten gehoeren dem
        // Programm. Vorher musste man ihn auf ein anderes Element legen, und
        // das nahm womoeglich selbst Tasten (eine Klappliste springt beim
        // Tippen). on_blur feuert wie bei jedem Fokuswechsel.
        if h < 0 { self.focus_widget = None; return Ok(()); }
        let (wi, i) = Self::dec_widget(h);
        let wd = self.wdg(h, "GUI_FOCUS")?;
        // Review-Fund: `wdg()` prueft nur, ob der Handle ueberhaupt in den
        // Bounds liegt, NICHT ob das Widget noch lebt -- GUI_FOCUS auf ein
        // getombstontes Handle wurde klaglos akzeptiert und rief
        // bring_to_front() fuer ein bereits ZERSTOERTES Fenster auf, was den
        // Index wieder in z_order einfuegte (den window_destroy gerade erst
        // entfernt hatte).
        if !wd.alive || !self.windows.get(wi).map(|w| w.alive).unwrap_or(false) {
            return Err("GUI_FOCUS: Widget/Fenster wurde bereits zerstoert".into());
        }
        self.focus_widget = Some((wi, i));
        self.focus_window = Some(wi);
        self.bring_to_front(wi);
        Ok(())
    }

    /// Handle des Widgets mit Tastatur-Fokus, oder -1. Gegenstueck zu
    /// GUI_FOCUS -- ohne Abfrage koennte ein Programm nicht anzeigen, worauf
    /// die Tastatur gerade wirkt (Hilfetext zum aktiven Feld, Statuszeile).
    pub fn focused(&self) -> i64 {
        match self.focus_widget {
            Some((wi, i)) => {
                let lebt = self.windows.get(wi)
                    .and_then(|w| w.widgets.get(i).map(|x| x.alive && w.alive))
                    .unwrap_or(false);
                if lebt { Self::enc_widget(wi, i) } else { -1 }
            }
            None => -1,
        }
    }
    /// Oberstes lebendes+sichtbares Widget am Bildschirmpunkt (Z-Order), oder -1.
    /// Liefert ein Widget-Handle (fuer Selektion im WYSIWYG-Editor).
    /// Das oberste sichtbare Fenster an einem Bildschirmpunkt, -1 = keins.
    /// Fuer Programme, die das Mausrad selbst auswerten: liegt ein anderes
    /// Fenster ueber ihrer Flaeche, gehoert das Rad diesem -- sonst rollte
    /// beides zugleich (die IDE blaetterte ihre Kacheln unter dem offenen
    /// Handbuch weiter).
    pub fn window_at(&self, mx: i32, my: i32) -> i64 {
        for &wi in self.z_order.iter().rev() {
            let win = &self.windows[wi];
            if !win.alive || !win.visible { continue; }
            if self.dock_gast_verdeckt(wi, mx, my) { continue; }
            if Self::in_rect(mx, my, self.sicht_rect(wi)) { return wi as i64; }
        }
        -1
    }

    pub fn hit_test(&self, mx: i32, my: i32) -> i64 {
        for &wi in self.z_order.iter().rev() {
            let win = &self.windows[wi];
            if !win.alive || !win.visible { continue; }
            if !Self::in_rect(mx, my, self.sicht_rect(wi)) { continue; }
            if self.dock_gast_verdeckt(wi, mx, my) { continue; }
            let (mx, my) = self.ins_fenster(wi, mx, my);
            // innerhalb des Fensters: spaeter gezeichnete Widgets liegen oben.
            // Review-Fund: pruefte bisher nur alive+visible, nicht die
            // Tab-Seite -- GUI_HIT_TEST fand auf einem getabbten Fenster
            // Widgets einer INAKTIVEN Reiter-Seite, obwohl update()/draw()
            // konsequent widget_shown() (das den Tab zusaetzlich prueft)
            // verwenden. Ein darauf aufbauender WYSIWYG-Editor haette so
            // unsichtbare Controls selektiert.
            for i in (0..win.widgets.len()).rev() {
                let wd = &win.widgets[i];
                if wd.kind != Kind::Layout && self.widget_shown(wi, wd) && Self::in_rect(mx, my, self.abs_rect(wi, wd))
                    && self.im_panel_sichtbar(wi, wd, mx, my) {
                    return Self::enc_widget(wi, i);
                }
            }
            return -1;   // Fenster getroffen, aber kein Widget
        }
        -1
    }
    // Window-Geometrie / Lifecycle / Enumeration.
    pub fn window_set_bounds(&mut self, h: i64, x: i32, y: i32, w: i32, ht: i32) -> Result<(), String> {
        let (x, y, w, ht) = (self.sk(x), self.sk(y), self.sk(w), self.sk(ht));
        let win = self.win_mut(h, "GUI_WINDOW_SET_BOUNDS")?;
        let (ow, oh) = (win.w, win.h);
        win.x = x; win.y = y; win.w = w.max(0); win.h = ht.max(0);
        let wi = h as usize;
        if self.windows[wi].w != ow || self.windows[wi].h != oh {
            self.relayout(wi);                          // Anchoring nachziehen
        }
        Ok(())
    }
    pub fn window_bounds(&self, h: i64) -> Result<(i32, i32, i32, i32), String> {
        let w = self.windows.get(h as usize)
            .ok_or("GUI_WINDOW_GET_*: ungueltiges GUI_WINDOW-Handle")?;
        Ok((self.unsk(w.x), self.unsk(w.y), self.unsk(w.w), self.unsk(w.h)))
    }
    /// Breite und Hoehe des INHALTSbereichs -- unter Titel-, Menue- und
    /// Reiterleiste, dort, wo Widget-Koordinaten zaehlen. Ohne das musste ein
    /// Programm mit Menue die Hoehe der Leiste raten, und die haengt an Thema
    /// und Massstab (gefunden beim Dogfooding 2026-10-09: die Statusleiste
    /// eines Notizzettels sass zu hoch).
    pub fn window_inhalt(&self, h: i64) -> Result<(i32, i32), String> {
        let wi = h as usize;
        let w = self.windows.get(wi)
            .ok_or("GUI_WINDOW_CONTENT_*: ungueltiges GUI_WINDOW-Handle")?;
        Ok((self.unsk(w.w), self.unsk((w.h - self.content_top(wi)).max(0))))
    }
    /// Loest JEDE Interaktions-Referenz auf Fenster `wi` (oder ein Widget
    /// darin), egal ob das Fenster gerade zerstoert, unsichtbar gemacht oder
    /// per Close-Button geschlossen wird.
    ///
    /// Review-Fund: `window_destroy` loeste bisher nur 5 von ~14 solchen
    /// Referenzen (focus_window/drag_window/resize_window/open_dropdown) --
    /// focus_widget/press_origin/active_slider/active_split/active_table/
    /// table_press/scroll_drag/open_menu/context_open/hover_w zeigten
    /// danach weiter auf das (tombstoned, aber nie geloeschte) Fenster/
    /// Widget. Weil Widgets nie wirklich entfernt werden (Tombstone-Pattern),
    /// indizierten diese Referenzen weiter erfolgreich -- `update()` feuerte
    /// dadurch z.B. GUI_ON_CHANGE/GUI_ON_CLICK fuer ein bereits zerstoertes
    /// Fenster, oder ein laufendes Table-Scrollbar-Drag lief gegen ein totes
    /// Widget weiter (und konnte den clamp-Panic in table_drag ausloesen).
    pub(super) fn clear_window_interactions(&mut self, wi: usize) {
        if self.focus_window == Some(wi) { self.focus_window = None; }
        if self.drag_window == Some(wi) { self.drag_window = None; }
        if self.resize_window == Some(wi) { self.resize_window = None; }
        if self.scroll_drag == Some(wi) { self.scroll_drag = None; }
        if self.focus_widget.map(|(w, _)| w) == Some(wi) { self.focus_widget = None; }
        if self.press_origin.map(|(w, _)| w) == Some(wi) { self.press_origin = None; }
        if self.active_slider.map(|(w, _)| w) == Some(wi) { self.active_slider = None; }
        if self.drag.as_ref().map(|d| d.quelle.0) == Some(wi) { self.drag = None; }
        if self.active_knob.map(|(w, _, _, _)| w) == Some(wi) { self.active_knob = None; }
        if self.active_split.map(|(w, _)| w) == Some(wi) { self.active_split = None; }
        if self.active_table.map(|(w, _)| w) == Some(wi) { self.active_table = None; }
        if self.table_press.map(|(w, _, _)| w) == Some(wi) { self.table_press = None; }
        if self.editing_table.map(|(w, _)| w) == Some(wi) { self.editing_table = None; }
        if self.hover_w.map(|(w, _)| w) == Some(wi) { self.hover_w = None; }
        if self.open_dropdown.map(|(w, _)| w) == Some(wi) { self.open_dropdown = None; }
        if self.open_menu.map(|(w, _)| w) == Some(wi) { self.open_menu = None; self.sub_chain.clear(); }
        if self.context_open.map(|(w, _, _, _)| w) == Some(wi) { self.context_open = None; }
    }
    pub fn window_destroy(&mut self, h: i64) -> Result<(), String> {
        let win = self.win_mut(h, "GUI_WINDOW_DESTROY")?;
        win.alive = false; win.visible = false;
        let wi = h as usize;
        // z_order-Entfernung nur beim ECHTEN Destroy -- ein nur unsichtbar
        // gemachtes Fenster (window_visible(false)/Close-Button) bleibt in
        // z_order, sonst wuerde es nach einem erneuten window_visible(true)
        // nie wieder gezeichnet/getroffen (nichts fuegt es sonst wieder ein).
        self.z_order.retain(|&i| i != wi);
        self.clear_window_interactions(wi);
        Ok(())
    }
    pub fn window_widget_count(&self, h: i64) -> Result<i64, String> {
        let w = self.windows.get(h as usize)
            .ok_or("GUI_WINDOW_WIDGET_COUNT: ungueltiges GUI_WINDOW-Handle")?;
        // Review-Fund: pruefte nur die Widgets auf `alive`, nie das FENSTER
        // selbst -- ein zerstoertes Fenster (window_destroy) lieferte
        // weiterhin seine (noch lebenden) Widget-Handles, obwohl to_json()
        // solche Fenster beim Serialisieren bereits konsequent ausfiltert.
        if !w.alive { return Ok(0); }
        Ok(w.widgets.iter().filter(|wd| wd.alive).count() as i64)
    }
    /// Handle des `n`-ten LEBENDEN Widgets von Fenster `h` (Einfuege-Reihenfolge),
    /// oder -1. Fuer Enumeration/Serialisierung.
    pub fn window_widget(&self, h: i64, n: i64) -> Result<i64, String> {
        let wi = h as usize;
        let w = self.windows.get(wi)
            .ok_or("GUI_WINDOW_WIDGET: ungueltiges GUI_WINDOW-Handle")?;
        if !w.alive { return Ok(-1); }
        let mut k = 0i64;
        for (i, wd) in w.widgets.iter().enumerate() {
            if !wd.alive { continue; }
            if k == n { return Ok(Self::enc_widget(wi, i)); }
            k += 1;
        }
        Ok(-1)
    }

    // --- Serialisierung (Layout als JSON; Editor<->Runtime-Kreis) ---
    /// Ein Widget als JSON. Die Groessen werden in den LOGISCHEN Massstab
    /// zurueckgerechnet: eine `.dhform` beschreibt das Layout, nicht die
    /// Anzeige. Sonst waere jede bei Massstab 2 gespeicherte Datei beim
    /// naechsten Laden doppelt so gross -- und wieder, und wieder.
    // ---- Text und Formular (Punkt 1 des gui-Ausbaus, 2026-09-04) ----------

    /// GUI_SET_ALIGN(wdg, "links"|"mitte"|"rechts") -- Beschriftung, Knopf,
    /// Textfeld. Gilt innerhalb der Breite des Widgets.
    pub fn set_align(&mut self, h: i64, wie: &str) -> Result<(), String> {
        let a = parse_align(wie).ok_or_else(|| format!(
            "GUI_SET_ALIGN: '{}' unbekannt -- links, mitte oder rechts", wie))?;
        let w = self.wdg_mut(h, "GUI_SET_ALIGN")?;
        if !matches!(w.kind, Kind::Label | Kind::Button | Kind::TextInput) {
            return Err("GUI_SET_ALIGN: nur Beschriftung, Knopf und Textfeld haben eine Ausrichtung".into());
        }
        w.align = a;
        Ok(())
    }

    /// GUI_SET_WRAP(label, breite) -- bei `breite` Pixeln an Wortgrenzen
    /// umbrechen (0 = aus). Die Hoehe folgt dem Text (siehe umbruch_layout);
    /// GUI_GET_H liefert sie ab dem naechsten GUI_UPDATE.
    pub fn set_wrap(&mut self, h: i64, breite: i32) -> Result<(), String> {
        if breite < 0 { return Err("GUI_SET_WRAP: breite muss >= 0 sein (0 = aus)".into()); }
        let breite = self.sk(breite);
        let w = self.wdg_mut(h, "GUI_SET_WRAP")?;
        if w.kind != Kind::Label {
            return Err("GUI_SET_WRAP: nur eine Beschriftung (GUI_LABEL) kann umbrechen".into());
        }
        w.wrap = breite > 0;
        if breite > 0 { w.w = breite; w.bw = breite; }
        Ok(())
    }

    /// GUI_TEXTINPUT_SET(tf, key$, wert) -- `passwort`, `nur_lesen`,
    /// `maxlaenge`, `zahlen` (0 frei, 1 ganze Zahl, 2 Kommazahl). EIN Setter
    /// statt vier Builtins, wie bei GUI_TEXTAREA_SET und GUI_TABLE_SET.
    pub fn textinput_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_TEXTINPUT_SET")?;
        if w.kind != Kind::TextInput {
            return Err("GUI_TEXTINPUT_SET: das Widget ist kein GUI_TEXTINPUT".into());
        }
        match key.to_lowercase().as_str() {
            "passwort" | "password" => w.passwort = wert != 0.0,
            "nur_lesen" | "readonly" => w.nur_lesen = wert != 0.0,
            "maxlaenge" | "maxlen" => {
                if wert < 0.0 { return Err("GUI_TEXTINPUT_SET: maxlaenge muss >= 0 sein (0 = frei)".into()); }
                w.maxlaenge = wert as i32;
            }
            "zahlen" | "numeric" => {
                if !(0.0..=3.0).contains(&wert) || wert.fract() != 0.0 {
                    return Err("GUI_TEXTINPUT_SET: zahlen ist 0 (frei), 1 (ganze Zahl), 2 (Kommazahl) oder 3 (Geldbetrag in Cent)".into());
                }
                w.zahlen = wert as u8;
            }
            "suche" | "search" => return self.feld_suche_setzen(h, wert != 0.0),
            _ => return Err(format!(
                "GUI_TEXTINPUT_SET: unbekannter Schluessel '{}' -- passwort, nur_lesen, maxlaenge, zahlen, suche", key)),
        }
        Ok(())
    }

    /// GUI_TEXTINPUT_GET(tf, key$): liest, was GUI_TEXTINPUT_SET setzt
    /// (Schalter als 0/1). Ein Test fragt so den Zustand, statt ihn sich
    /// selbst zu merken.
    pub fn textinput_get(&self, h: i64, key: &str) -> Result<f64, String> {
        let w = self.wdg(h, "GUI_TEXTINPUT_GET")?;
        if w.kind != Kind::TextInput {
            return Err("GUI_TEXTINPUT_GET: das Widget ist kein GUI_TEXTINPUT".into());
        }
        let b = |x: bool| if x { 1.0 } else { 0.0 };
        Ok(match key.to_lowercase().as_str() {
            "passwort" | "password" => b(w.passwort),
            "nur_lesen" | "readonly" => b(w.nur_lesen),
            "maxlaenge" | "maxlen" => w.maxlaenge as f64,
            "zahlen" | "numeric" => w.zahlen as f64,
            "suche" | "search" => b(Self::feld_ist_suche(w)),
            _ => return Err(format!(
                "GUI_TEXTINPUT_GET: unbekannter Schluessel '{}' -- passwort, nur_lesen, maxlaenge, zahlen, suche", key)),
        })
    }
    /// GUI_TEXTINPUT_PLACEHOLDER$(tf): der graue Text im leeren Feld.
    pub fn textinput_placeholder(&self, h: i64) -> Result<String, String> {
        let w = self.wdg(h, "GUI_TEXTINPUT_PLACEHOLDER$")?;
        if !matches!(w.kind, Kind::TextInput | Kind::TextArea) {
            return Err("GUI_TEXTINPUT_PLACEHOLDER$: das Widget ist kein GUI_TEXTINPUT oder GUI_TEXTAREA".into());
        }
        Ok(w.placeholder.clone())
    }

    /// GUI_ENTERED(tf) -- wurde in DIESEM Bild Enter im Textfeld gedrueckt?
    pub fn entered(&self, h: i64) -> Result<bool, String> {
        Ok(self.wdg(h, "GUI_ENTERED")?.entered)
    }

    /// GUI_ON_ENTER(tf, handler) -- Rueckruf statt Abfrage.
    pub fn on_enter(&mut self, h: i64, func: Option<Rueckruf>) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_ON_ENTER")?;
        if w.kind != Kind::TextInput {
            return Err("GUI_ON_ENTER: nur ein Textfeld (GUI_TEXTINPUT) meldet Enter".into());
        }
        w.on_enter = func;
        Ok(())
    }

    /// GUI_WINDOW_DEFAULT(win, knopf) / GUI_WINDOW_CANCEL(win, knopf) -- der
    /// Knopf, den Enter bzw. ESC im Fenster ausloest (-1 = keiner). Nur ein
    /// Knopf desselben Fensters kommt in Frage.
    pub fn window_default(&mut self, win: i64, knopf: i64, abbruch: bool) -> Result<(), String> {
        let fn_ = if abbruch { "GUI_WINDOW_CANCEL" } else { "GUI_WINDOW_DEFAULT" };
        let wi = win as usize;
        if win < 0 || wi >= self.windows.len() || !self.windows[wi].alive {
            return Err(format!("{}: ungueltiges Fenster-Handle {}", fn_, win));
        }
        let idx = if knopf < 0 { -1 } else {
            let (kwi, ki) = Self::dec_widget(knopf);
            if kwi != wi {
                return Err(format!("{}: der Knopf gehoert zu einem anderen Fenster", fn_));
            }
            let w = self.windows[wi].widgets.get(ki).filter(|w| w.alive)
                .ok_or_else(|| format!("{}: ungueltiges Widget-Handle {}", fn_, knopf))?;
            if w.kind != Kind::Button {
                return Err(format!("{}: nur ein Knopf (GUI_BUTTON) kann Standard- oder Abbrechen-Knopf sein", fn_));
            }
            ki as i32
        };
        if abbruch { self.windows[wi].cancel_btn = idx; } else { self.windows[wi].default_btn = idx; }
        Ok(())
    }

    /// Einen Knopf per Taste ausloesen -- wie ein Klick, mit denselben
    /// Bedingungen (lebendig, sichtbar, bedienbar).
    pub(super) fn knopf_ausloesen(&mut self, wi: usize, idx: i32) {
        if idx < 0 { return; }
        let i = idx as usize;
        let ok = match self.windows[wi].widgets.get(i) {
            Some(w) => w.alive && w.enabled && w.kind == Kind::Button && self.widget_shown(wi, w),
            None => false,
        };
        if !ok { return; }
        let w = &mut self.windows[wi].widgets[i];
        w.clicked = true;
        let f = w.on_click.clone();
        if let Some(f) = f { self.pending.push(f); }
    }

    /// Umbrechende Beschriftungen: die Hoehe folgt dem Text. Laeuft in
    /// GUI_UPDATE, weil nur dort Graphics (zum Messen) UND Schreibzugriff
    /// zusammenkommen -- das Zeichnen ist `&self`.
    pub(super) fn umbruch_layout(&mut self, g: &Graphics) {
        let mut neu: Vec<(usize, usize, i32)> = Vec::new();
        for wi in 0..self.windows.len() {
            if !self.windows[wi].alive { continue; }
            for i in 0..self.windows[wi].widgets.len() {
                let w = &self.windows[wi].widgets[i];
                if !w.alive || w.kind != Kind::Label || !w.wrap { continue; }
                let (pv, ph) = self.w_abstand(w).unwrap_or((0, 0));
                let n = self.umbrechen(g, w, &w.text, (w.w - 2 * ph).max(1)).len().max(1) as i32;
                let h = n * (self.wsize(g, w) + self.sk(4)) + 2 * pv;
                if h != w.h { neu.push((wi, i, h)); }
            }
        }
        for (wi, i, h) in neu {
            let w = &mut self.windows[wi].widgets[i];
            w.h = h; w.bh = h;
        }
    }

    /// Der Hinweis einer leeren Liste, eines Baums oder einer Tabelle:
    /// umgebrochen auf die Breite, jede Zeile mittig. Vorher stand er in einer
    /// Zeile und lief rechts aus dem Feld (gesehen beim Dogfooding 2026-10-09:
    /// "Noch keine Notiz -- Strg" in einer 260 Punkte breiten Liste).
    pub(super) fn leer_hinweis(&self, g: &mut Graphics, wdg: &Widget, x: i32, y: i32, breite: i32, text: &str, farbe: i64) {
        let rand = self.sk(6);
        let innen = (breite - 2 * rand).max(1);
        let zh = self.wsize(g, wdg) + self.sk(4);
        for (k, z) in self.umbrechen(g, wdg, text, innen).into_iter().enumerate() {
            let tw = self.wtext_width(g, wdg, &z);
            self.wtext(g, wdg, x + ((breite - tw) / 2).max(rand), y + k as i32 * zh, z, farbe);
        }
    }

    /// Text auf `breite` Pixel kuerzen: passt er, unveraendert, sonst so viele
    /// Zeichen, wie mit "..." dahinter passen.
    pub(super) fn text_kuerzen(&self, g: &Graphics, wdg: &Widget, text: &str, breite: i32) -> String {
        if self.wtext_width(g, wdg, text) <= breite { return text.to_string(); }
        let zeichen: Vec<char> = text.chars().collect();
        let mut n = zeichen.len();
        while n > 0 {
            n -= 1;
            let probe: String = zeichen[..n].iter().collect::<String>() + "...";
            if self.wtext_width(g, wdg, &probe) <= breite { return probe; }
        }
        String::new()
    }

    /// Text an Wortgrenzen auf `breite` Pixel umbrechen. Ein Wort, das allein
    /// nicht in eine Zeile passt, wird an der Zeichengrenze geteilt; `\n`
    /// bleibt ein Umbruch. Gemessen wird mit der Schrift des Widgets.
    pub(super) fn umbrechen(&self, g: &Graphics, w: &Widget, text: &str, breite: i32) -> Vec<String> {
        let mut zeilen: Vec<String> = Vec::new();
        for absatz in text.split('\n') {
            let mut zeile = String::new();
            for wort in absatz.split(' ') {
                let mut rest = wort.to_string();
                loop {
                    let probe = if zeile.is_empty() { rest.clone() } else { format!("{} {}", zeile, rest) };
                    if self.wtext_width(g, w, &probe) <= breite { zeile = probe; break; }
                    if !zeile.is_empty() { zeilen.push(std::mem::take(&mut zeile)); continue; }
                    // Leere Zeile und das Wort ist trotzdem zu breit: so viele
                    // Zeichen, wie passen -- mindestens eines, sonst dreht es sich.
                    let mut teil = String::new();
                    let mut n = 0usize;
                    for ch in rest.chars() {
                        let p2 = format!("{}{}", teil, ch);
                        if n > 0 && self.wtext_width(g, w, &p2) > breite { break; }
                        teil = p2; n += 1;
                    }
                    zeilen.push(teil);
                    rest = rest.chars().skip(n).collect();
                    if rest.is_empty() { break; }
                }
            }
            zeilen.push(zeile);
        }
        zeilen
    }

    /// Stand VOR einer Aenderung merken. Anschlaege innerhalb von 0,8 s
    /// bilden EINEN Schritt.
    pub(super) fn undo_merken(w: &mut Widget, before: &str, caret: i32, jetzt: f64) {
        if w.undo.is_empty() || jetzt - w.undo_zeit > 0.8 {
            // Die Stile gehoeren zum Stand VORHER -- abgeglichen sind sie
            // bis zum Ende des letzten Bildes, also genau dann, wenn ihr Text
            // `before` ist. Sonst (ein Weg, der den Text aenderte, ohne die
            // Bits zu kennen) waeren sie zum falschen Text gemerkt.
            let st = if w.formatiert && w.stile_text == before { w.stile.clone() } else { Vec::new() };
            w.undo.push((before.to_string(), caret, st));
            verlauf_kuerzen(&mut w.undo);
        }
        w.undo_zeit = jetzt;
        w.redo.clear();
    }

    /// Strg+Z / Strg+Y im Textfeld oder Textbereich.
    pub(super) fn text_undo(&mut self, wi: usize, i: usize, wieder: bool) {
        let w = &mut self.windows[wi].widgets[i];
        let stand = if wieder { w.redo.pop() } else { w.undo.pop() };
        let Some((t, c, st)) = stand else { return; };
        stile_abgleichen(w);
        let st_jetzt = if w.formatiert { w.stile.clone() } else { Vec::new() };
        let jetzt_stand = (w.text.clone(), w.caret, st_jetzt);
        if wieder { w.undo.push(jetzt_stand); verlauf_kuerzen(&mut w.undo); } else { w.redo.push(jetzt_stand); }
        let n = t.chars().count() as i32;
        w.text = t;
        if w.formatiert {
            w.stile = if st.len() == n as usize { st } else { vec![0; n as usize] };
            w.stile_text = w.text.clone();
            w.tipp_stil = None;
        }
        w.caret = c.clamp(0, n); w.sel_anchor = w.caret;
        w.undo_zeit = -10.0;      // die naechste Eingabe ist ein neuer Schritt
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }

    pub(super) fn menu_items_json(&self, win: &Window, m: &Menu) -> Vec<serde_json::Value> {
        m.items.iter().map(|it| {
            let mut o = serde_json::json!({
                "label": it.label, "separator": it.separator, "enabled": it.enabled,
            });
            if !it.kuerzel.is_empty() { o["shortcut"] = serde_json::json!(it.kuerzel); }
            if !it.sym.is_empty() { o["symbol"] = serde_json::json!(it.sym); }
            if it.haken { o["checkable"] = serde_json::json!(true); o["checked"] = serde_json::json!(it.an); }
            if it.sub >= 0 {
                if let Some(sm) = win.menus.get(it.sub as usize) {
                    o["items"] = serde_json::json!(self.menu_items_json(win, sm));
                }
            }
            o
        }).collect()
    }
    /// Eintraege aus JSON -- ein Eintrag mit `items` wird zum Untermenue, das
    /// dafuer an `win.menus` angehaengt wird (Index im Eintrag).
    pub(super) fn menu_items_aus_json(win: &mut Window, its: &[serde_json::Value]) -> Vec<MenuItem> {
        let mut out = Vec::new();
        for itj in its {
            let sep = itj["separator"].as_bool().unwrap_or(false);
            let mut it = MenuItem::neu(itj["label"].as_str().unwrap_or("").to_string(), sep);
            if !sep { it.enabled = itj["enabled"].as_bool().unwrap_or(true); }
            if let Some(k) = itj["shortcut"].as_str() {
                if let Ok((mods, code)) = kuerzel_parsen(k) { it.kuerzel = k.to_string(); it.k_mods = mods; it.k_code = code; }
            }
            if itj["checkable"].as_bool().unwrap_or(false) {
                it.haken = true; it.an = itj["checked"].as_bool().unwrap_or(false);
            }
            if let Some(s) = itj["symbol"].as_str() {
                if let Ok(n) = sinnbild_pruefen(s, "GUI_LOAD") { it.sym = n; }
            }
            if let Some(sub) = itj["items"].as_array() {
                let smi = win.menus.len();
                win.menus.push(Menu { label: it.label.clone(), in_bar: false, unter: true, items: Vec::new(), auf_t: 0.0, b: Blende::default(), ziel: -1 });
                let sub_items = Self::menu_items_aus_json(win, sub);
                win.menus[smi].items = sub_items;
                it.sub = smi as i32;
            }
            out.push(it);
        }
        out
    }
}
