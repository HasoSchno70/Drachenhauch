//! Layouts (Zeilen, Spalten, Raster, Formular) und Theme/Metriken.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // ---- Layout (Punkt 5 des gui-Ausbaus, 2026-09-04) ---------------------

    pub(super) fn layout_art(art: &str) -> Result<(u8, i32), String> {
        let a = art.trim().to_lowercase();
        if let Some(rest) = a.strip_prefix("raster:").or_else(|| a.strip_prefix("grid:")) {
            let n: i32 = rest.trim().parse().map_err(|_| format!("GUI_LAYOUT: '{}' -- nach raster: gehoert die Spaltenzahl", art))?;
            if !(1..=32).contains(&n) { return Err("GUI_LAYOUT: 1 bis 32 Spalten".into()); }
            return Ok((2, n));
        }
        match a.as_str() {
            "zeile" | "row" | "waagerecht" | "horizontal" => Ok((0, 0)),
            "spalte" | "column" | "senkrecht" | "vertikal" | "vertical" => Ok((1, 0)),
            "fluss" | "flow" => Ok((3, 0)),
            _ => Err(format!("GUI_LAYOUT: '{}' ist keine Art -- zeile, spalte, fluss oder raster:N", art)),
        }
    }
    /// GUI_LAYOUT(win, art$, x, y, w, h) -> GUI_WIDGET
    pub fn layout_new(&mut self, win: i64, art: &str, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let (a, n) = Self::layout_art(art)?;
        let mut wd = Self::blank(Kind::Layout, x, y, w, h);
        wd.layout = Some(Box::new(LayoutState { art: a, spalten: n, abstand: 6, rand: 0,
                                                ausrichtung: 0, dehnen: true, rahmen: false, kinder: Vec::new(),
                                                auswahl: false, gewaehlt: -1 }));
        self.add_widget(win, "GUI_LAYOUT", wd)
    }
    pub(super) fn layout_mut(&mut self, h: i64, fn_: &str) -> Result<&mut LayoutState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Layout { return Err(format!("{}: das Widget ist kein Layout (GUI_LAYOUT)", fn_)); }
        w.layout.as_deref_mut().ok_or_else(|| format!("{}: Layout ohne Zustand", fn_))
    }
    /// Enthaelt Layout `li` (direkt oder tiefer) den Index `ziel`?
    pub(super) fn layout_enthaelt(&self, wi: usize, li: usize, ziel: usize) -> bool {
        let Some(l) = self.windows[wi].widgets.get(li).and_then(|w| w.layout.as_ref()) else { return false };
        for &(k, _) in &l.kinder {
            if k < 0 { continue; }
            if k as usize == ziel { return true; }
            if self.layout_enthaelt(wi, k as usize, ziel) { return true; }
        }
        false
    }
    /// GUI_LAYOUT_ADD(layout, wdg[, gewicht]) -- Gewicht 0 = eigene Groesse,
    /// ab 1 = Anteil am Restplatz. Ein Widget steckt in hoechstens EINEM
    /// Behaelter; ein Behaelter darf nicht sich selbst oder einen Vorfahren
    /// aufnehmen (das liefe in der Verteilung im Kreis).
    pub fn layout_add(&mut self, layout: i64, wdg: i64, gewicht: i64) -> Result<(), String> {
        if gewicht < 0 { return Err("GUI_LAYOUT_ADD: gewicht muss >= 0 sein".into()); }
        let (lwi, li) = Self::dec_widget(layout);
        self.layout_mut(layout, "GUI_LAYOUT_ADD")?;
        let (wwi, ki) = Self::dec_widget(wdg);
        self.wdg(wdg, "GUI_LAYOUT_ADD")?;
        if wwi != lwi { return Err("GUI_LAYOUT_ADD: das Widget gehoert zu einem anderen Fenster".into()); }
        if ki == li { return Err("GUI_LAYOUT_ADD: ein Layout kann sich nicht selbst aufnehmen".into()); }
        if self.layout_enthaelt(lwi, ki, li) {
            return Err("GUI_LAYOUT_ADD: das Widget ist ein Behaelter, in dem dieses Layout schon steckt".into());
        }
        // Aus jedem anderen Behaelter des Fensters loesen.
        for w in self.windows[lwi].widgets.iter_mut() {
            if let Some(l) = w.layout.as_mut() { l.kinder.retain(|&(k, _)| k != ki as i32); }
        }
        let l = self.layout_mut(layout, "GUI_LAYOUT_ADD")?;
        l.kinder.push((ki as i32, gewicht as i32));
        Ok(())
    }
    /// GUI_LAYOUT_SPACER(layout[, gewicht]) -- Leerraum, der Platz nimmt.
    pub fn layout_spacer(&mut self, layout: i64, gewicht: i64) -> Result<(), String> {
        if gewicht < 0 { return Err("GUI_LAYOUT_SPACER: gewicht muss >= 0 sein".into()); }
        self.layout_mut(layout, "GUI_LAYOUT_SPACER")?.kinder.push((-1, gewicht as i32));
        Ok(())
    }
    /// GUI_LAYOUT_REMOVE(layout, wdg) -- das Widget bleibt, wo es zuletzt lag.
    pub fn layout_remove(&mut self, layout: i64, wdg: i64) -> Result<(), String> {
        let (_, ki) = Self::dec_widget(wdg);
        self.layout_mut(layout, "GUI_LAYOUT_REMOVE")?.kinder.retain(|&(k, _)| k != ki as i32);
        Ok(())
    }
    /// GUI_LAYOUT_SET(layout, key$, wert): abstand, rand, ausrichtung, dehnen, rahmen.
    pub fn layout_set(&mut self, layout: i64, key: &str, wert: f64) -> Result<(), String> {
        let l = self.layout_mut(layout, "GUI_LAYOUT_SET")?;
        match key.to_lowercase().as_str() {
            "abstand" | "gap" | "spacing" => { if wert < 0.0 { return Err("GUI_LAYOUT_SET: abstand muss >= 0 sein".into()); } l.abstand = wert as i32; }
            "rand" | "padding" => { if wert < 0.0 { return Err("GUI_LAYOUT_SET: rand muss >= 0 sein".into()); } l.rand = wert as i32; }
            "ausrichtung" | "align" => {
                if !(0.0..=2.0).contains(&wert) { return Err("GUI_LAYOUT_SET: ausrichtung ist 0 (Anfang), 1 (Mitte) oder 2 (Ende)".into()); }
                l.ausrichtung = wert as u8;
            }
            "dehnen" | "stretch" => l.dehnen = wert != 0.0,
            "rahmen" | "frame" => l.rahmen = wert != 0.0,
            "auswahl" | "selection" => { l.auswahl = wert != 0.0; if !l.auswahl { l.gewaehlt = -1; } }
            _ => return Err(format!("GUI_LAYOUT_SET: unbekannter Schluessel '{}' -- abstand, rand, ausrichtung, dehnen, rahmen, auswahl", key)),
        }
        Ok(())
    }

    /// Mass eines Widgets nach seinem Inhalt (skalierte Pixel).
    pub(super) fn inhalt_mass(&self, g: &Graphics, w: &Widget) -> (i32, i32) {
        let m = self.inhalt_mass_roh(g, w);
        self.gruppe_anwenden(w, m)
    }
    /// Mass nach Inhalt ohne Groessengruppe.
    pub(super) fn inhalt_mass_roh(&self, g: &Graphics, w: &Widget) -> (i32, i32) {
        let sz = self.wsize(g, w);
        let text = self.anzeige_text(w);
        let tw = self.wtext_width(g, w, &text);
        match w.kind {
            Kind::Label => {
                let zeilen: Vec<&str> = text.split('\n').collect();
                let bw = zeilen.iter().map(|z| self.wtext_width(g, w, z)).max().unwrap_or(0);
                let (pv, ph) = self.w_abstand(w).unwrap_or((0, 0));
                (bw.max(1) + 2 * ph, (zeilen.len().max(1) as i32) * (sz + self.sk(4)) + 2 * pv)
            }
            Kind::Card => (self.sk(200), self.sk(190)),
            Kind::Button => {
                let icon = if w.sel >= 0 { self.sk(22) } else { 0 };
                match self.w_abstand(w) {
                    // Mit Innenabstand: genau der Text samt Abstand, ohne Mindestmass.
                    Some((v, h)) => (tw + icon + 2 * h, sz + 2 * v),
                    None => ((tw + icon + self.sk(24)).max(self.sk(28)), (sz + self.sk(12)).max(self.sk(26))),
                }
            }
            Kind::Checkbox | Kind::Radio => {
                let cs = self.m("check_size");
                (cs + self.w_abstand(w).map(|a| a.1).unwrap_or(self.sk(8)) + tw, cs.max(sz))
            }
            Kind::Toggle => {
                let ph = self.m("check_size").max(self.sk(14));
                (ph * 2 + self.w_abstand(w).map(|a| a.1).unwrap_or(self.sk(8)) + tw, ph.max(sz))
            }
            Kind::TextInput | Kind::Dropdown | Kind::Spinner => (if w.w > 0 { w.w } else { self.sk(160) },
                sz + 2 * self.w_abstand(w).map(|a| a.0).unwrap_or(self.sk(6))),
            _ => (if w.w > 0 { w.w } else { self.sk(100) }, if w.h > 0 { w.h } else { self.sk(24) }),
        }
    }
    /// GUI_AUTOSIZE(wdg) -- sofort messen, wenn Graphics da ist.
    pub fn autosize(&mut self, g: Option<&Graphics>, h: i64) -> Result<(), String> {
        let (wi, i) = Self::dec_widget(h);
        self.wdg(h, "GUI_AUTOSIZE")?;
        match g {
            Some(g) => {
                let (nw, nh) = self.inhalt_mass(g, &self.windows[wi].widgets[i]);
                let w = &mut self.windows[wi].widgets[i];
                w.w = nw; w.h = nh; w.bw = nw; w.bh = nh; w.nat_w = nw; w.nat_h = nh;
                w.auto_w = false; w.auto_h = false;
            }
            None => { let w = &mut self.windows[wi].widgets[i]; w.auto_w = true; w.auto_h = true; }
        }
        Ok(())
    }

    /// Jedes Bild: erst das Automass, dann jeder Wurzel-Behaelter (einer,
    /// der in keinem anderen steckt) von oben nach unten.
    pub(super) fn layout_pass(&mut self, g: &Graphics) {
        self.gruppen_pass(g);
        for wi in 0..self.windows.len() {
            if !self.windows[wi].alive { continue; }
            let n = self.windows[wi].widgets.len();
            for i in 0..n {
                let w = &self.windows[wi].widgets[i];
                if !w.alive || !(w.auto_w || w.auto_h) { continue; }
                let (nw, nh) = self.inhalt_mass(g, w);
                let (aw, ah) = (w.auto_w, w.auto_h);
                let w = &mut self.windows[wi].widgets[i];
                if aw { w.w = nw; w.bw = nw; }
                if ah { w.h = nh; w.bh = nh; }
            }
            let mut ist_kind = vec![false; n];
            for i in 0..n {
                if let Some(l) = self.windows[wi].widgets[i].layout.as_ref() {
                    for &(k, _) in &l.kinder { if k >= 0 && (k as usize) < n { ist_kind[k as usize] = true; } }
                }
            }
            for i in 0..n {
                let w = &self.windows[wi].widgets[i];
                if w.alive && w.kind == Kind::Layout && !ist_kind[i] { self.layout_anwenden(g, wi, i, 0); }
            }
        }
    }

    /// Mindestmass eines Widgets: das gesetzte (GUI_SET_MIN_SIZE) -- und bei
    /// einem Behaelter zusaetzlich das, was sein Inhalt mindestens braucht
    /// (feste Kinder in voller Groesse, gewichtete mit ihrem Mindestmass,
    /// Abstaende und Rand dazu). Rekursiv, damit ein Behaelter im Behaelter
    /// sein Mass nach oben reicht.
    pub(super) fn min_mass(&self, wi: usize, k: usize, tiefe: u32) -> (i32, i32) {
        let w = match self.windows[wi].widgets.get(k) { Some(w) => w, None => return (0, 0) };
        let (mut mw, mut mh) = (w.min_w, w.min_h);
        if tiefe > 16 { return (mw, mh); }
        if let Some(l) = w.layout.as_ref() {
            let rand = self.sk(l.rand);
            let ab = self.sk(l.abstand);
            let n = self.windows[wi].widgets.len();
            let kinder: Vec<(i32, i32)> = l.kinder.iter().copied()
                .filter(|&(c, _)| c < 0 || ((c as usize) < n && self.windows[wi].widgets[c as usize].alive))
                .collect();
            if kinder.is_empty() { return (mw, mh); }
            match l.art {
                0 | 1 => {
                    let zeile = l.art == 0;
                    let mut laengs = ab * (kinder.len() as i32 - 1);
                    let mut quer = 0;
                    for &(c, gew) in &kinder {
                        if c < 0 { continue; }
                        let (cw, ch) = self.min_mass(wi, c as usize, tiefe + 1);
                        let wd = &self.windows[wi].widgets[c as usize];
                        let (eigen_l, eigen_q) = if zeile { (wd.nat_w, wd.nat_h) } else { (wd.nat_h, wd.nat_w) };
                        let (min_l, min_q) = if zeile { (cw, ch) } else { (ch, cw) };
                        laengs += if gew > 0 { min_l } else { eigen_l.max(min_l) };
                        // Quer zaehlt die eigene Groesse auch bei `dehnen`:
                        // ein auf 5 px gestauchter Knopf ist kein Knopf mehr.
                        quer = quer.max(eigen_q.max(min_q));
                    }
                    let (lw, lh) = if zeile { (laengs, quer) } else { (quer, laengs) };
                    mw = mw.max(lw + 2 * rand);
                    mh = mh.max(lh + 2 * rand);
                }
                _ => {
                    // Raster: breiteste Zelle mal Spalten, Zeilenhoehen summiert.
                    let sp = l.spalten.max(1) as usize;
                    let mut zelle_w = 0;
                    let mut hoehe = 0;
                    let mut zeilen = 0;
                    let mut zeile_h = 0;
                    for (i, &(c, _)) in kinder.iter().enumerate() {
                        if i % sp == 0 && i > 0 { hoehe += zeile_h; zeilen += 1; zeile_h = 0; }
                        if c < 0 { continue; }
                        let (cw, ch) = self.min_mass(wi, c as usize, tiefe + 1);
                        let wd = &self.windows[wi].widgets[c as usize];
                        zelle_w = zelle_w.max(cw.max(wd.nat_w));
                        zeile_h = zeile_h.max(wd.nat_h.max(ch));
                    }
                    hoehe += zeile_h;
                    zeilen += 1;
                    mw = mw.max(zelle_w * sp as i32 + ab * (sp as i32 - 1) + 2 * rand);
                    mh = mh.max(hoehe + ab * (zeilen - 1) + 2 * rand);
                }
            }
        }
        (mw, mh)
    }
    /// GUI_LAYOUT_MIN_W / _H: was ein Behaelter mindestens braucht --
    /// gedacht fuer WINDOW_MIN_SIZE / GUI_WINDOW_SET_MIN_SIZE.
    pub fn layout_min(&self, h: i64) -> Result<(i64, i64), String> {
        let (wi, i) = Self::dec_widget(h);
        let w = self.wdg(h, "GUI_LAYOUT_MIN_W")?;
        if w.kind != Kind::Layout { return Err("GUI_LAYOUT_MIN_W: Widget ist kein Layout-Behaelter".into()); }
        let (mw, mh) = self.min_mass(wi, i, 0);
        Ok((self.unsk(mw) as i64, self.unsk(mh) as i64))
    }
    pub fn set_min_size(&mut self, h: i64, mw: i32, mh: i32) -> Result<(), String> {
        let (mw, mh) = (self.sk(mw.max(0)), self.sk(mh.max(0)));
        let w = self.wdg_mut(h, "GUI_SET_MIN_SIZE")?;
        w.min_w = mw; w.min_h = mh;
        Ok(())
    }

    pub(super) fn layout_anwenden(&mut self, g: &Graphics, wi: usize, li: usize, tiefe: u32) {
        if tiefe > 16 { return; }   // gegen Kreise aus einer von Hand gebauten Datei
        let (l, lx, ly, lw, lh) = {
            let w = &self.windows[wi].widgets[li];
            match w.layout.as_ref() { Some(l) => ((**l).clone(), w.x, w.y, w.w, w.h), None => return }
        };
        let rand = self.sk(l.rand);
        let ab = self.sk(l.abstand);
        let (ix, iy, iw, ih) = (lx + rand, ly + rand, (lw - 2 * rand).max(0), (lh - 2 * rand).max(0));
        let n = self.windows[wi].widgets.len();
        let kinder: Vec<(i32, i32)> = l.kinder.iter().copied()
            .filter(|&(k, _)| k < 0 || ((k as usize) < n && self.windows[wi].widgets[k as usize].alive))
            .collect();
        if kinder.is_empty() { return; }
        // Kinder mit Automass zuerst messen -- ein Knopf "nach Inhalt" muss
        // seine Breite kennen, bevor der Restplatz verteilt wird.
        for &(k, _) in &kinder {
            if k < 0 { continue; }
            let w = &self.windows[wi].widgets[k as usize];
            // Beschriftung, Kaestchen, Radio und Kippschalter haben keine
            // eigene Groesse, nur ihren Inhalt -- im Behaelter werden sie
            // immer exakt gemessen.
            let inhaltlich = matches!(w.kind, Kind::Label | Kind::Checkbox | Kind::Radio | Kind::Toggle);
            if w.auto_w || w.auto_h || inhaltlich {
                let (nw, nh) = self.inhalt_mass(g, w);
                let (aw, ah) = (w.auto_w || inhaltlich, w.auto_h);
                let w = &mut self.windows[wi].widgets[k as usize];
                if aw { w.w = nw; w.nat_w = nw; }
                if ah { w.h = nh; w.nat_h = nh; }
            }
        }
        let mut setzen: Vec<(usize, i32, i32, i32, i32)> = Vec::new();
        match l.art {
            0 | 1 => {
                let zeile = l.art == 0;
                let laenge = if zeile { iw } else { ih };
                let mut fest = ab * (kinder.len() as i32 - 1);
                let mut summe_g = 0;
                for &(k, gew) in &kinder {
                    if gew > 0 { summe_g += gew; continue; }
                    if k >= 0 {
                        let (mw, mh) = self.min_mass(wi, k as usize, 0);
                        let w = &self.windows[wi].widgets[k as usize];
                        fest += if zeile { w.w.max(mw) } else { w.h.max(mh) };
                    }
                }
                let rest = (laenge - fest).max(0);
                let mut pos = if zeile { ix } else { iy };
                let mut vergeben = 0;
                let mut gi = 0;
                for &(k, gew) in &kinder {
                    let anteil = if gew > 0 {
                        gi += 1;
                        // Der letzte Gewichtete bekommt den Rundungsrest.
                        if gi == kinder.iter().filter(|(_, g2)| *g2 > 0).count() { rest - vergeben }
                        else { let a = rest * gew / summe_g.max(1); vergeben += a; a }
                    } else { -1 };
                    if k < 0 { pos += anteil.max(0) + ab; continue; }
                    let (ow, oh) = { let w = &self.windows[wi].widgets[k as usize]; (w.w, w.h) };
                    // Mindestmass: ein gewichtetes Kind faellt nie darunter --
                    // dann laeuft die Zeile lieber ueber, als dass ein Knopf
                    // zu einem Strich wird. Ein Behaelter als Kind bringt das
                    // Mass seines Inhalts mit.
                    let (mw, mh) = self.min_mass(wi, k as usize, 0);
                    let (nw, nh, nx, ny);
                    if zeile {
                        nw = if anteil >= 0 { anteil.max(mw) } else { ow.max(mw) };
                        nh = if l.dehnen { ih.max(mh) } else { oh.max(mh).min(ih.max(mh)) };
                        nx = pos;
                        ny = iy + match l.ausrichtung { 1 => (ih - nh) / 2, 2 => ih - nh, _ => 0 };
                        pos += nw + ab;
                    } else {
                        nh = if anteil >= 0 { anteil.max(mh) } else { oh.max(mh) };
                        nw = if l.dehnen { iw.max(mw) } else { ow.max(mw).min(iw.max(mw)) };
                        ny = pos;
                        nx = ix + match l.ausrichtung { 1 => (iw - nw) / 2, 2 => iw - nw, _ => 0 };
                        pos += nh + ab;
                    }
                    setzen.push((k as usize, nx, ny, nw.max(1), nh.max(1)));
                }
            }
            3 => setzen.extend(self.fluss_legen(wi, &kinder, ix, iy, iw, ab, l.ausrichtung)),
            _ => {
                let sp = l.spalten.max(1);
                let zw = ((iw - ab * (sp - 1)) / sp).max(1);
                let mut y = iy;
                let mut zeile_kinder: Vec<(i32, i32)> = Vec::new();
                let alle: Vec<(i32, i32)> = kinder.clone();
                let mut idx = 0;
                while idx < alle.len() {
                    zeile_kinder.clear();
                    for _ in 0..sp { if idx < alle.len() { zeile_kinder.push(alle[idx]); idx += 1; } }
                    // Zeilenhoehe = hoechstes Kind der Zeile.
                    let zh = zeile_kinder.iter().filter(|&&(k, _)| k >= 0)
                        .map(|&(k, _)| self.windows[wi].widgets[k as usize].h).max().unwrap_or(0).max(1);
                    for (c, &(k, _)) in zeile_kinder.iter().enumerate() {
                        if k < 0 { continue; }
                        let (ow, oh) = { let w = &self.windows[wi].widgets[k as usize]; (w.w, w.h) };
                        let x = ix + c as i32 * (zw + ab);
                        let nw = if l.dehnen { zw } else { ow.min(zw) };
                        let nh = oh.min(zh).max(1);
                        let ny = y + match l.ausrichtung { 1 => (zh - nh) / 2, 2 => zh - nh, _ => 0 };
                        setzen.push((k as usize, x, ny, nw.max(1), nh));
                    }
                    y += zh + ab;
                }
            }
        }
        let mut unter: Vec<usize> = Vec::new();
        for (k, x, y, w, h) in setzen {
            let wd = &mut self.windows[wi].widgets[k];
            wd.x = x; wd.y = y; wd.w = w; wd.h = h;
            wd.bx = x; wd.by = y; wd.bw = w; wd.bh = h;
            if wd.kind == Kind::Layout { unter.push(k); }
        }
        for k in unter { self.layout_anwenden(g, wi, k, tiefe + 1); }
    }

    pub(super) fn widget_json(&self, w: &Widget) -> serde_json::Value {
        let mut o = serde_json::json!({
            "kind": w.kind.as_str(),
            "x": self.unsk(w.x), "y": self.unsk(w.y),
            "w": self.unsk(w.w), "h": self.unsk(w.h),
            "text": w.text, "color": w.color,
            "value": w.value, "min": w.min, "max": w.max,
            "checked": w.checked, "placeholder": w.placeholder, "visible": w.visible,
        });
        // Nur UNGEBUNDENE Handler wandern in die Datei. Eine an eine Instanz
        // gebundene Methode laesst sich nicht wiederherstellen -- das Objekt
        // gibt es beim Laden nicht. Ihren blossen Namen zu schreiben waere
        // eine Luege: er wuerde beim Laden als freie Funktion gedeutet.
        if let Some(f) = &w.on_click {
            if !f.ist_gebunden() { o["on_click"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_hover {
            if !f.ist_gebunden() { o["on_hover"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_leave {
            if !f.ist_gebunden() { o["on_leave"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_focus {
            if !f.ist_gebunden() { o["on_focus"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_blur {
            if !f.ist_gebunden() { o["on_blur"] = serde_json::json!(&*f.name); }
        }
        if let Some(f) = &w.on_change {
            if !f.ist_gebunden() { o["on_change"] = serde_json::json!(&*f.name); }
        }
        if !w.ov.is_empty() { o["ov"] = serde_json::json!(w.ov); }
        if !w.group.is_empty() { o["group"] = serde_json::json!(w.group); }
        if !w.items.is_empty() { o["items"] = serde_json::json!(w.items); }
        if w.sel != -1 { o["sel"] = serde_json::json!(w.sel); }   // Dropdown/ListBox-Index ODER Image-Textur
        if !w.enabled { o["enabled"] = serde_json::json!(false); }
        if w.font != -1 { o["font"] = serde_json::json!(w.font); }
        if w.font_size != 0 { o["font_size"] = serde_json::json!(w.font_size); }
        if w.stil != 0 { o["font_style"] = serde_json::json!(crate::schnitt::stil_text(w.stil)); }
        if w.kind == Kind::ColorPicker {
            o["color_value"] = serde_json::json!(
                std::format!("#{:06X}", crate::farbraum::hsv_zu_rgb(w.hsv[0], w.hsv[1], w.hsv[2])));
        }
        if w.kind == Kind::DatePicker {
            o["date"] = serde_json::json!(crate::kalender::format(w.datum[0], w.datum[1], w.datum[2]));
            if w.kompakt { o["kompakt"] = serde_json::json!(true); }
        }
        if w.anchor != 5 { o["anchor"] = serde_json::json!(Self::anchor_str(w.anchor)); }
        if let Some(l) = w.list.as_ref() {
            // Symbole sind Textur-Handles und bleiben draussen (wie bei GUI_IMAGE).
            if !l.ist_leer() {
                let mut lo = serde_json::Map::new();
                if l.multi { lo.insert("multi".into(), serde_json::json!(true)); }
                if l.kaestchen { lo.insert("kaestchen".into(), serde_json::json!(true)); }
                if l.bearbeitbar { lo.insert("bearbeitbar".into(), serde_json::json!(true)); }
                if l.checks.iter().any(|&c| c) { lo.insert("checked".into(), serde_json::json!(l.checks)); }
                if l.colors.iter().any(|&c| c >= 0) { lo.insert("colors".into(), serde_json::json!(l.colors)); }
                if l.multi && l.sel.iter().any(|&x| x) { lo.insert("selected".into(), serde_json::json!(l.sel)); }
                if l.daten.iter().any(|s| !s.is_empty()) { lo.insert("daten".into(), serde_json::json!(l.daten)); }
                if l.details.iter().any(|s| !s.is_empty()) { lo.insert("details".into(), serde_json::json!(l.details)); }
                if l.tips.iter().any(|s| !s.is_empty()) { lo.insert("tips".into(), serde_json::json!(l.tips)); }
                if l.aus.iter().any(|&x| x) { lo.insert("gesperrt".into(), serde_json::json!(l.aus)); }
                if l.kopf.iter().any(|&x| x) { lo.insert("koepfe".into(), serde_json::json!(l.kopf)); }
                if !l.leer_text.is_empty() { lo.insert("leer_text".into(), serde_json::json!(l.leer_text)); }
                o["list"] = serde_json::Value::Object(lo);
            }
        }
        if let Some(l) = w.layout.as_ref() {
            let art = match l.art { 0 => "zeile", 1 => "spalte", 3 => "fluss", _ => "raster" };
            let mut lo = serde_json::json!({ "art": art, "kinder": l.kinder.iter().map(|&(k, g)| serde_json::json!([k, g])).collect::<Vec<_>>() });
            if l.art == 2 { lo["spalten"] = serde_json::json!(l.spalten); }
            if l.abstand != 6 { lo["abstand"] = serde_json::json!(l.abstand); }
            if l.rand != 0 { lo["rand"] = serde_json::json!(l.rand); }
            if l.ausrichtung != 0 { lo["ausrichtung"] = serde_json::json!(l.ausrichtung); }
            if !l.dehnen { lo["dehnen"] = serde_json::json!(false); }
            if l.rahmen { lo["rahmen"] = serde_json::json!(true); }
            if l.auswahl { lo["auswahl"] = serde_json::json!(true); }
            o["layout"] = lo;
        }
        if w.auto_w { o["auto_w"] = serde_json::json!(true); }
        if w.auto_h { o["auto_h"] = serde_json::json!(true); }
        if w.vert { o["vertical"] = serde_json::json!(true); }
        if w.unbestimmt { o["indeterminate"] = serde_json::json!(true); }
        if w.bildmodus != 0 { o["mode"] = serde_json::json!(bildmodus_str(w.bildmodus)); }
        if w.ziehbar { o["draggable"] = serde_json::json!(true); }
        if w.umbruch { o["wrap_text"] = serde_json::json!(true); }
        // Geformter Text geht als Markdown hinaus -- lesbar und von jedem
        // Markdown-Betrachter anzeigbar; `text` bleibt der schlichte Text.
        if w.formatiert {
            o["formatiert"] = serde_json::json!(true);
            let chars: Vec<char> = w.text.chars().collect();
            let st = if w.stile_text == w.text && w.stile.len() == chars.len() { w.stile.clone() } else { vec![0; chars.len()] };
            o["markdown"] = serde_json::json!(markdown_aus(&chars, &st));
        }
        if !w.bind.is_empty() {
            o["bind"] = serde_json::json!(w.bind);
            if !w.form.is_empty() { o["form"] = serde_json::json!(w.form); }
        }
        if w.min_w > 0 || w.min_h > 0 {
            o["min_w"] = serde_json::json!(self.unsk(w.min_w));
            o["min_h"] = serde_json::json!(self.unsk(w.min_h));
        }
        if !w.regeln.is_empty() {
            o["rules"] = serde_json::json!(w.regeln.iter().map(|r| {
                let mut rj = serde_json::json!({ "art": r.art.as_str() });
                if matches!(r.art, RegelArt::Bereich | RegelArt::Laenge) { rj["a"] = serde_json::json!(r.a); rj["b"] = serde_json::json!(r.b); }
                if r.art == RegelArt::Muster { rj["text"] = serde_json::json!(r.text); }
                if !r.meldung.is_empty() { rj["meldung"] = serde_json::json!(r.meldung); }
                rj
            }).collect::<Vec<_>>());
        }
        if w.fehler_label >= 0 { o["error_label"] = serde_json::json!(w.fehler_label); }
        if w.ablage { o["drop_target"] = serde_json::json!(true); }
        if let Some(p) = w.panel.as_ref() {
            o["panel"] = serde_json::json!({ "kinder": p.kinder, "scroll": self.unsk(p.scroll) });
        }
        if w.align >= 0 {
            o["align"] = serde_json::json!(match w.align { 1 => "mitte", 2 => "rechts", _ => "links" });
        }
        if w.wrap { o["wrap"] = serde_json::json!(true); }
        if w.passwort { o["passwort"] = serde_json::json!(true); }
        if w.nur_lesen { o["nur_lesen"] = serde_json::json!(true); }
        if w.frei { o["bearbeitbar"] = serde_json::json!(true); }
        if w.maxlaenge > 0 { o["maxlaenge"] = serde_json::json!(w.maxlaenge); }
        if w.zahlen != 0 { o["zahlen"] = serde_json::json!(w.zahlen); }
        if !w.name.is_empty() { o["name"] = serde_json::json!(w.name); }
        bausteine::bau_json(w, &mut o);
        if !w.klassen.is_empty() { o["klassen"] = serde_json::json!(w.klassen); }
        ueber::feld_json(w, &mut o);
        // Der Tooltip fehlte in der Datei -- ein im Designer gesetzter war
        // nach dem Laden weg.
        if !w.tooltip.is_empty() { o["tooltip"] = serde_json::json!(w.tooltip); }
        if w.tab_index > 0 { o["tab_index"] = serde_json::json!(w.tab_index); }
        if let Some(f) = &w.on_enter {
            if !f.ist_gebunden() { o["on_enter"] = serde_json::json!(&*f.name); }
        }
        if let Some(t) = &w.tbl {
            // Eine schlichte Textzelle wird als STRING geschrieben, nur eine
            // mit eigener Farbe/Art/Bild als Objekt. Damit bleibt das
            // .dhform-Format fuer gewoehnliche Tabellen unveraendert (alte
            // Dateien und der Form-Designer lesen es weiter), ohne dass die
            // neuen Angaben beim Speichern verloren gehen.
            let rows: Vec<serde_json::Value> = t.rows.iter().map(|row| {
                serde_json::Value::Array(row.iter().map(|c| {
                    if c.fg < 0 && c.bg < 0 && c.align < 0
                        && c.kind == CellKind::Text && c.image < 0 && c.value == 0.0 {
                        serde_json::json!(c.text)
                    } else {
                        let mut o = serde_json::Map::new();
                        o.insert("text".into(), serde_json::json!(c.text));
                        if c.fg >= 0 { o.insert("fg".into(), serde_json::json!(c.fg)); }
                        if c.bg >= 0 { o.insert("bg".into(), serde_json::json!(c.bg)); }
                        if c.align >= 0 { o.insert("align".into(), serde_json::json!(c.align)); }
                        if c.kind != CellKind::Text {
                            o.insert("kind".into(), serde_json::json!(c.kind.name()));
                        }
                        if c.image >= 0 { o.insert("image".into(), serde_json::json!(c.image)); }
                        if c.value != 0.0 { o.insert("value".into(), serde_json::json!(c.value)); }
                        serde_json::Value::Object(o)
                    }
                }).collect())
            }).collect();
            let mut tj = serde_json::json!({
                "headers": t.headers, "rows": rows,
                "col_widths": t.col_widths, "selected": t.selected,
            });
            // Nur schreiben, was vom Standard abweicht -- sonst waere jede
            // gespeicherte Form voller Vorgabewerte.
            if t.row_h != self.sk(TBL_ROW_H) { tj["row_h"] = serde_json::json!(self.unsk(t.row_h)); }
            if t.header_h != self.sk(TBL_HEADER_H) { tj["header_h"] = serde_json::json!(self.unsk(t.header_h)); }
            if t.zebra { tj["zebra"] = serde_json::json!(true); }
            if !t.grid { tj["grid"] = serde_json::json!(false); }
            if !t.col_align.is_empty() { tj["col_align"] = serde_json::json!(t.col_align); }
            if !t.col_order.is_empty() { tj["col_order"] = serde_json::json!(t.col_order); }
            if t.frozen > 0 { tj["frozen"] = serde_json::json!(t.frozen); }
            // Die Schalter gehoerten bisher nicht ins .dhform -- ohne sie
            // koennte ein Formular-Designer sie zwar setzen, beim Laden waeren
            // sie aber wieder weg. Nur schreiben, was vom Standard abweicht.
            if t.filter_row { tj["filter_row"] = serde_json::json!(true); }
            if !t.sortable { tj["sortable"] = serde_json::json!(false); }
            if !t.resizable_cols { tj["resizable_cols"] = serde_json::json!(false); }
            if t.reorderable { tj["reorderable"] = serde_json::json!(true); }
            if t.multi { tj["multi"] = serde_json::json!(true); }
            if t.col_edit.iter().any(|&x| x) { tj["col_edit"] = serde_json::json!(t.col_edit); }
            if t.zellmodus { tj["zellmodus"] = serde_json::json!(true); }
            if !t.leer_text.is_empty() { tj["leer_text"] = serde_json::json!(t.leer_text); }
            if t.baum {
                tj["baum"] = serde_json::json!(true);
                if t.eltern.iter().any(|&e| e >= 0) { tj["eltern"] = serde_json::json!(t.eltern); }
                if t.offen.iter().any(|&o| o) { tj["offen"] = serde_json::json!(t.offen); }
            }
            if t.zeilen_anhaengen { tj["zeilen_anhaengen"] = serde_json::json!(true); }
            if t.col_typ.iter().any(|&x| x != 0) {
                let namen: Vec<&str> = t.col_typ.iter()
                    .map(|&x| match x { 1 => "ganz", 2 => "zahl", 3 => "auswahl", 4 => "datum", _ => "text" }).collect();
                tj["col_type"] = serde_json::json!(namen);
            }
            if t.col_wahl.iter().any(|w| !w.is_empty()) { tj["col_choices"] = serde_json::json!(t.col_wahl); }
            if t.row_fg.iter().any(|&x| x >= 0) { tj["row_fg"] = serde_json::json!(t.row_fg); }
            if t.row_bg.iter().any(|&x| x >= 0) { tj["row_bg"] = serde_json::json!(t.row_bg); }
            o["table"] = tj;
        }
        if let Some(t) = &w.tree {
            o["tree"] = serde_json::json!({
                "nodes": t.nodes.iter().map(|n| {
                    let mut o = serde_json::json!({
                        "label": n.label, "parent": n.parent, "level": n.level,
                        "expanded": n.expanded, "has_children": n.has_children,
                    });
                    // Farbe und Sinnbild nur, wenn gesetzt -- alte Dateien
                    // bleiben so, wie sie waren.
                    if n.color >= 0 { o["color"] = serde_json::json!(n.color); }
                    if !n.sym.is_empty() { o["sym"] = serde_json::json!(n.sym); }
                    o
                }).collect::<Vec<_>>(),
                "selected": t.selected,
                "multi": t.multi,
            });
            if t.kaestchen { o["tree"]["kaestchen"] = serde_json::json!(true); }
            if t.bearbeitbar { o["tree"]["bearbeitbar"] = serde_json::json!(true); }
            if !t.leer_text.is_empty() { o["tree"]["leer_text"] = serde_json::json!(t.leer_text); }
            // Ein Dateibaum speichert seine EINSTELLUNGEN, nicht seine Knoten:
            // die stehen auf der Platte, und wer die Datei woanders oeffnet,
            // haette sonst eine Liste von Namen, die es dort nicht gibt.
            if let Some(d) = t.datei.as_ref() {
                o["tree"]["nodes"] = serde_json::json!([]);
                o["filetree"] = serde_json::json!({
                    "wurzel": d.wurzel, "muster": d.muster, "ausnahmen": d.ausnahmen,
                    "ordner_zuerst": d.ordner_zuerst, "verborgene": d.verborgene,
                    "nur_ordner": d.nur_ordner, "offen": d.offen,
                });
                if !d.farben.is_empty() {
                    o["filetree"]["farben"] = serde_json::json!(d.farben.iter()
                        .map(|(m, f)| serde_json::json!([m, f])).collect::<Vec<_>>());
                }
                if !d.symbole.is_empty() {
                    o["filetree"]["symbole"] = serde_json::json!(d.symbole.iter()
                        .map(|(m, s)| serde_json::json!([m, s])).collect::<Vec<_>>());
                }
                if d.nach_art { o["filetree"]["nach_art"] = serde_json::json!(true); }
            }
        }
        if w.kind == Kind::TimePicker {
            o["time"] = serde_json::json!(format!("{:02}:{:02}:{:02}", w.zeit[0], w.zeit[1], w.zeit[2]));
            if w.zeit_sek { o["seconds"] = serde_json::json!(true); }
        }
        if let Some(r) = &w.rich {
            // Die QUELLE gehoert in die Datei, nicht der Satz: der haengt an
            // Breite, Schrift und Massstab und entsteht beim Laden neu.
            let mut rj = serde_json::json!({ "quelle": r.quelle });
            if r.basis > 0 { rj["groesse"] = serde_json::json!(r.basis); }
            if !r.code_knoepfe.is_empty() { rj["code_knoepfe"] = serde_json::json!(r.code_knoepfe.join("|")); }
            o["rich"] = rj;
        }
        if !w.sinnbild.is_empty() { o["symbol"] = serde_json::json!(w.sinnbild); }
        if w.variante > 0 { o["variant"] = serde_json::json!(KNOPF_ARTEN[w.variante as usize]); }
        if let Some(s) = &w.status {
            o["status"] = serde_json::json!({
                "felder": s.felder.iter().map(|f| {
                    let mut fj = serde_json::json!({ "text": f.text, "breite": f.breite });
                    if !f.tip.is_empty() { fj["tip"] = serde_json::json!(f.tip); }
                    if f.ausr > 0 { fj["ausrichtung"] = serde_json::json!(["links", "mitte", "rechts"][f.ausr as usize]); }
                    if f.klickbar { fj["klickbar"] = serde_json::json!(true); }
                    fj
                }).collect::<Vec<_>>(),
            });
        }
        if let Some(p) = &w.pfad {
            o["pfad"] = serde_json::json!({ "texte": p.texte, "daten": p.daten });
        }
        if let Some(l) = &w.leiste {
            // Textur-Handles gehoeren NICHT in die Datei (sie gelten nur in
            // diesem Lauf) -- ein Eintrag mit Bild steht darum ohne Bild da.
            o["leiste"] = serde_json::json!({
                "eintraege": l.eintraege.iter().map(|e| {
                    let mut ej = serde_json::json!({ "art": match e.art { 1 => "trenner", 2 => "luecke", _ => "knopf" } });
                    if !e.symbol.is_empty() { ej["symbol"] = serde_json::json!(e.symbol); }
                    if !e.tip.is_empty() { ej["tip"] = serde_json::json!(e.tip); }
                    if !e.text.is_empty() { ej["text"] = serde_json::json!(e.text); }
                    if e.aus { ej["gesperrt"] = serde_json::json!(true); }
                    if e.kippbar { ej["kippbar"] = serde_json::json!(true); ej["an"] = serde_json::json!(e.an); }
                    ej
                }).collect::<Vec<_>>(),
                "beschriftung": l.mit_text,
                "symbolgroesse": l.symbol,
            });
        }
        if let Some(a) = &w.akk {
            o["akkordeon"] = serde_json::json!({
                "offen": a.offen, "hoehen": a.hoehen, "mehrere": a.mehrere,
                "lage": a.lage.iter().map(|&(k, x, y)| serde_json::json!([k, x, y])).collect::<Vec<_>>(),
            });
        }
        if let Some(z) = &w.wz {
            o["assistent"] = serde_json::json!({ "pruefen": z.pruefen, "texte": z.texte });
        }
        if let Some(st) = &w.tabctl {
            o["tabctl"] = serde_json::json!({
                "kinder": st.kinder.iter().map(|&(k, s)| serde_json::json!([k, s])).collect::<Vec<_>>(),
                "seite": w.sel,
            });
            if st.schliessbar { o["tabctl"]["schliessbar"] = serde_json::json!(true); }
        }
        if w.tab_page != -1 { o["tab_page"] = serde_json::json!(w.tab_page); }
        o
    }
    pub(super) fn widget_from_json(wj: &serde_json::Value) -> Result<Widget, String> {
        let ks = wj["kind"].as_str().ok_or("GUI_LOAD: Widget ohne 'kind'")?;
        let kind = Kind::from_str(ks).ok_or_else(|| format!("GUI_LOAD: unbekannter Widget-Typ '{}'", ks))?;
        let gi = |k: &str, d: i64| wj[k].as_i64().unwrap_or(d) as i32;
        let mut w = Self::blank(kind, gi("x", 0), gi("y", 0), gi("w", 0), gi("h", 0));
        w.text = wj["text"].as_str().unwrap_or("").to_string();
        w.color = wj["color"].as_i64().unwrap_or(-1);   // -1 = dem Thema folgen
        w.value = wj["value"].as_f64().unwrap_or(0.0);
        w.min = wj["min"].as_f64().unwrap_or(0.0);
        w.max = wj["max"].as_f64().unwrap_or(1.0);
        // Review-Fund: `slider()`/`spinner()` (Konstruktoren) lehnen
        // `max <= min` ab, aber GUI_LOAD baut Widgets direkt aus JSON ohne
        // diese Pruefung -- ein geladenes `{"min":10,"max":0}` liess jeden
        // spaeteren `v.clamp(w.min, w.max)` (GUI_SET_VALUE, Spinner-Schritt)
        // mit "min <= max"-Panic abstuerzen. Reihenfolge einfach herstellen
        // statt hart abzulehnen -- ein geladenes Widget soll nutzbar bleiben.
        if w.min > w.max { std::mem::swap(&mut w.min, &mut w.max); }
        w.checked = wj["checked"].as_bool().unwrap_or(false);
        w.placeholder = wj["placeholder"].as_str().unwrap_or("").to_string();
        w.visible = wj["visible"].as_bool().unwrap_or(true);
        w.on_click = wj["on_click"].as_str().map(Rueckruf::benannt);
        w.on_hover = wj["on_hover"].as_str().map(Rueckruf::benannt);
        w.on_leave = wj["on_leave"].as_str().map(Rueckruf::benannt);
        w.on_focus = wj["on_focus"].as_str().map(Rueckruf::benannt);
        w.on_blur = wj["on_blur"].as_str().map(Rueckruf::benannt);
        w.on_change = wj["on_change"].as_str().map(Rueckruf::benannt);
        if let Some(ov) = wj["ov"].as_object() {
            for (k, val) in ov { if let Some(c) = val.as_i64() { w.ov.insert(k.clone(), c); } }
        }
        w.group = wj["group"].as_str().unwrap_or("").to_string();
        if let Some(its) = wj["items"].as_array() {
            w.items = its.iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
        }
        w.sel = wj["sel"].as_i64().unwrap_or(-1) as i32;
        w.enabled = wj["enabled"].as_bool().unwrap_or(true);
        w.font = wj["font"].as_i64().unwrap_or(-1);
        w.font_size = wj["font_size"].as_i64().unwrap_or(0) as i32;
        // Ein krummer Stil in einer Datei bricht das Laden nicht ab.
        w.stil = wj["font_style"].as_str().and_then(|s| crate::schnitt::stil_parsen(s).ok()).unwrap_or(0);
        if let Some(a) = wj["align"].as_str() { if let Some(v) = parse_align(a) { w.align = v; } }
        w.wrap = wj["wrap"].as_bool().unwrap_or(false);
        w.passwort = wj["passwort"].as_bool().unwrap_or(false);
        w.nur_lesen = wj["nur_lesen"].as_bool().unwrap_or(false);
        w.frei = w.kind == Kind::Dropdown && wj["bearbeitbar"].as_bool().unwrap_or(false);
        w.maxlaenge = wj["maxlaenge"].as_i64().unwrap_or(0).max(0) as i32;
        w.zahlen = wj["zahlen"].as_i64().unwrap_or(0).clamp(0, 3) as u8;
        // Name und Klassen fuer die Stildatei; ein Name, der dort nicht
        // stehen koennte (der Form-Designer erlaubt mehr), zaehlt nicht.
        w.name = wj["name"].as_str().map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'))
            .unwrap_or_default();
        bausteine::bau_aus_json(&mut w, wj);
        w.klassen = wj["klassen"].as_array().map(|a| a.iter().filter_map(|x| x.as_str())
            .map(|s| s.to_lowercase()).collect()).unwrap_or_default();
        if w.kind == Kind::TextInput { ueber::feld_aus_json(&mut w, wj); }
        w.tooltip = wj["tooltip"].as_str().unwrap_or("").to_string();
        w.tab_index = wj["tab_index"].as_i64().unwrap_or(0).max(0) as i32;
        w.on_enter = wj["on_enter"].as_str().map(Rueckruf::benannt);
        w.auto_w = wj["auto_w"].as_bool().unwrap_or(false);
        w.auto_h = wj["auto_h"].as_bool().unwrap_or(false);
        w.vert = kind == Kind::Slider && wj["vertical"].as_bool().unwrap_or(false);
        w.unbestimmt = kind == Kind::Progress && wj["indeterminate"].as_bool().unwrap_or(false);
        if let Some(m) = wj["mode"].as_str() { w.bildmodus = bildmodus_parsen(m).unwrap_or(0); }
        w.ziehbar = wj["draggable"].as_bool().unwrap_or(false);
        w.umbruch = kind == Kind::TextArea && wj["wrap_text"].as_bool().unwrap_or(false);
        if kind == Kind::TextArea && wj["formatiert"].as_bool().unwrap_or(false) {
            w.formatiert = true;
            if let Some(md) = wj["markdown"].as_str() {
                let (t, st) = markdown_ein(md);
                w.text = t; w.stile = st;
            } else { w.stile = vec![0; w.text.chars().count()]; }
            w.stile_text = w.text.clone();
        }
        w.bind = wj["bind"].as_str().unwrap_or("").to_string();
        w.form = wj["form"].as_str().unwrap_or("").to_string();
        w.min_w = wj["min_w"].as_i64().unwrap_or(0).max(0) as i32;
        w.min_h = wj["min_h"].as_i64().unwrap_or(0).max(0) as i32;
        if let Some(rs) = wj["rules"].as_array() {
            for rj in rs {
                let art = match rj["art"].as_str().and_then(RegelArt::from_str) { Some(a) => a, None => continue };
                w.regeln.push(Regel {
                    art,
                    a: rj["a"].as_f64().unwrap_or(0.0), b: rj["b"].as_f64().unwrap_or(0.0),
                    text: rj["text"].as_str().unwrap_or("").to_string(),
                    meldung: rj["meldung"].as_str().unwrap_or("").to_string(),
                });
            }
        }
        w.fehler_label = wj["error_label"].as_i64().unwrap_or(-1) as i32;
        w.ablage = wj["drop_target"].as_bool().unwrap_or(false);
        if kind == Kind::Panel {
            if let Some(pj) = wj.get("panel").and_then(|v| v.as_object()) {
                let kinder: Vec<usize> = pj.get("kinder").and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|x| x.as_i64()).filter(|&x| x >= 0).map(|x| x as usize).collect())
                    .unwrap_or_default();
                w.panel = Some(Box::new(PanelState { kinder, scroll: pj.get("scroll").and_then(|v| v.as_i64()).unwrap_or(0) as i32, inhalt_h: 0 }));
            }
        }
        if kind == Kind::Layout {
            let lj = &wj["layout"];
            let art = match lj["art"].as_str().unwrap_or("spalte") { "zeile" => 0, "raster" => 2, "fluss" => 3, _ => 1 };
            let kinder: Vec<(i32, i32)> = lj["kinder"].as_array().map(|a| a.iter().filter_map(|p| {
                let arr = p.as_array()?;
                Some((arr.first()?.as_i64()? as i32, arr.get(1).and_then(|g| g.as_i64()).unwrap_or(0) as i32))
            }).collect()).unwrap_or_default();
            w.layout = Some(Box::new(LayoutState {
                art, spalten: lj["spalten"].as_i64().unwrap_or(2).clamp(1, 32) as i32, kinder,
                abstand: lj["abstand"].as_i64().unwrap_or(6).max(0) as i32,
                rand: lj["rand"].as_i64().unwrap_or(0).max(0) as i32,
                ausrichtung: lj["ausrichtung"].as_i64().unwrap_or(0).clamp(0, 2) as u8,
                dehnen: lj["dehnen"].as_bool().unwrap_or(true),
                rahmen: lj["rahmen"].as_bool().unwrap_or(false),
                auswahl: lj["auswahl"].as_bool().unwrap_or(false),
                gewaehlt: -1,
            }));
        }
        if let Some(lj) = wj.get("list").and_then(|v| v.as_object()) {
            let n = w.items.len();
            let mut l = ListState { anker: -1, ..Default::default() };
            l.sync(n);
            l.multi = lj.get("multi").and_then(|v| v.as_bool()).unwrap_or(false);
            l.kaestchen = lj.get("kaestchen").and_then(|v| v.as_bool()).unwrap_or(false);
            l.bearbeitbar = lj.get("bearbeitbar").and_then(|v| v.as_bool()).unwrap_or(false);
            if let Some(a) = lj.get("checked").and_then(|v| v.as_array()) {
                for (k, v) in a.iter().enumerate().take(n) { l.checks[k] = v.as_bool().unwrap_or(false); }
            }
            if let Some(a) = lj.get("colors").and_then(|v| v.as_array()) {
                for (k, v) in a.iter().enumerate().take(n) { l.colors[k] = v.as_i64().unwrap_or(-1); }
            }
            if let Some(a) = lj.get("selected").and_then(|v| v.as_array()) {
                for (k, v) in a.iter().enumerate().take(n) { l.sel[k] = v.as_bool().unwrap_or(false); }
            }
            let texte = |name: &str, ziel: &mut Vec<String>| {
                if let Some(a) = lj.get(name).and_then(|v| v.as_array()) {
                    for (k, v) in a.iter().enumerate().take(n) { ziel[k] = v.as_str().unwrap_or("").to_string(); }
                }
            };
            texte("daten", &mut l.daten);
            texte("details", &mut l.details);
            texte("tips", &mut l.tips);
            let schalter = |name: &str, ziel: &mut Vec<bool>| {
                if let Some(a) = lj.get(name).and_then(|v| v.as_array()) {
                    for (k, v) in a.iter().enumerate().take(n) { ziel[k] = v.as_bool().unwrap_or(false); }
                }
            };
            schalter("gesperrt", &mut l.aus);
            schalter("koepfe", &mut l.kopf);
            l.leer_text = lj.get("leer_text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            w.list = Some(Box::new(l));
        }
        // Farbe als "#RRGGBB" -- lesbar in der Datei und nicht als Zahl, die
        // niemand entziffert. Unlesbares wird still uebergangen: eine von Hand
        // bearbeitete Form soll nicht am Farbfeld scheitern.
        if let Some(t) = wj["color_value"].as_str() {
            if let Ok(v) = i64::from_str_radix(t.trim_start_matches('#'), 16) {
                let (h, sa, va) = crate::farbraum::rgb_zu_hsv(v & 0xFF_FFFF);
                w.hsv = [h, sa, va];
            }
        }
        if let Some(t) = wj["date"].as_str() {
            if let Some((j, m, d)) = crate::kalender::parse(t) { w.datum = [j, m, d]; }
        }
        if kind == Kind::DatePicker && wj["kompakt"].as_bool() == Some(true) { w.kompakt = true; }
        if kind == Kind::Table {
            let mut ts = TableState::default();
            if let Some(tj) = wj.get("table") {
                if let Some(hs) = tj["headers"].as_array() {
                    ts.headers = hs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
                }
                if let Some(rs) = tj["rows"].as_array() {
                    // Review-Fund: `filter_map(|x| x.as_str())` liess jede
                    // NICHT-String-Zelle (Zahl, Bool, null) STILLSCHWEIGEND
                    // aus der Zeile verschwinden -- ein `.dhform`/gespeichertes
                    // GUI_SAVE mit einer numerischen Tabellenzelle lud eine zu
                    // kurze Zeile, `draw_table` paniked dann beim naechsten
                    // GUI_DRAW. Jede Zelle wird jetzt zu einem String
                    // konvertiert statt verworfen -- Zeilenlaenge bleibt erhalten.
                    fn cell_to_string(x: &serde_json::Value) -> String {
                        match x {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        }
                    }
                    // Zelle kann ein blosser Wert sein (altes Format, und der
                    // Normalfall) ODER ein Objekt mit Farben/Art/Bild.
                    fn to_cell(x: &serde_json::Value) -> Cell {
                        if let Some(o) = x.as_object() {
                            let mut c = Cell::text(o.get("text").map(cell_to_string).unwrap_or_default());
                            if let Some(v) = o.get("fg").and_then(|v| v.as_i64()) { c.fg = v; }
                            if let Some(v) = o.get("bg").and_then(|v| v.as_i64()) { c.bg = v; }
                            if let Some(v) = o.get("align").and_then(|v| v.as_i64()) { c.align = v as i8; }
                            if let Some(v) = o.get("kind").and_then(|v| v.as_str()) {
                                if let Some(k) = CellKind::parse(v) { c.kind = k; }
                            }
                            if let Some(v) = o.get("image").and_then(|v| v.as_i64()) { c.image = v; }
                            if let Some(v) = o.get("value").and_then(|v| v.as_f64()) { c.value = v; }
                            c
                        } else {
                            Cell::text(cell_to_string(x))
                        }
                    }
                    ts.rows = rs.iter().map(|row| row.as_array()
                        .map(|r| r.iter().map(to_cell).collect())
                        .unwrap_or_default()).collect();
                    ts.row_fg = vec![-1; ts.rows.len()];
                    ts.row_bg = vec![-1; ts.rows.len()];
                }
                if let Some(v) = tj["row_h"].as_i64() { ts.row_h = (v as i32).clamp(8, 400); }
                if let Some(v) = tj["header_h"].as_i64() { ts.header_h = (v as i32).clamp(0, 200); }
                if let Some(v) = tj["zebra"].as_bool() { ts.zebra = v; }
                if let Some(v) = tj["grid"].as_bool() { ts.grid = v; }
                if let Some(a) = tj["col_align"].as_array() {
                    ts.col_align = a.iter().filter_map(|x| x.as_i64().map(|n| n as i8)).collect();
                }
                if let Some(a) = tj["col_order"].as_array() {
                    ts.col_order = a.iter().filter_map(|x| x.as_u64().map(|n| n as usize)).collect();
                }
                if let Some(v) = tj["frozen"].as_i64() { ts.frozen = v as i32; }
                if let Some(v) = tj["filter_row"].as_bool() { ts.filter_row = v; }
                if let Some(v) = tj["sortable"].as_bool() { ts.sortable = v; }
                if let Some(v) = tj["resizable_cols"].as_bool() { ts.resizable_cols = v; }
                if let Some(v) = tj["reorderable"].as_bool() { ts.reorderable = v; }
                if let Some(v) = tj["multi"].as_bool() { ts.multi = v; }
                if let Some(a) = tj["col_edit"].as_array() {
                    ts.col_edit = a.iter().map(|x| x.as_bool().unwrap_or(false)).collect();
                }
                if let Some(v) = tj["zellmodus"].as_bool() { ts.zellmodus = v; }
                if let Some(v) = tj["leer_text"].as_str() { ts.leer_text = v.to_string(); }
                if let Some(v) = tj["baum"].as_bool() { ts.baum = v; }
                if let Some(a) = tj["eltern"].as_array() { ts.eltern = a.iter().map(|x| x.as_i64().unwrap_or(-1) as i32).collect(); }
                if let Some(a) = tj["offen"].as_array() { ts.offen = a.iter().map(|x| x.as_bool().unwrap_or(false)).collect(); }
                if let Some(v) = tj["zeilen_anhaengen"].as_bool() { ts.zeilen_anhaengen = v; }
                if let Some(a) = tj["col_type"].as_array() {
                    ts.col_typ = a.iter().map(|x| match x.as_str().unwrap_or("") {
                        "ganz" => 1, "zahl" => 2, "auswahl" => 3, "datum" => 4, _ => 0 }).collect();
                }
                if let Some(a) = tj["col_choices"].as_array() {
                    ts.col_wahl = a.iter().map(|w| w.as_array().map(|w| w.iter()
                        .filter_map(|s| s.as_str().map(String::from)).collect()).unwrap_or_default()).collect();
                }
                for (schluessel, ziel) in [("row_fg", true), ("row_bg", false)] {
                    if let Some(a) = tj[schluessel].as_array() {
                        let v: Vec<i64> = a.iter().map(|x| x.as_i64().unwrap_or(-1)).collect();
                        if ziel { ts.row_fg = v; } else { ts.row_bg = v; }
                    }
                }
                // Nach dem Laden muessen die Zeilenfarb-Listen zur Zeilenzahl
                // passen -- sonst greift row_fg[r] beim Zeichnen daneben.
                ts.row_fg.resize(ts.rows.len(), -1);
                ts.row_bg.resize(ts.rows.len(), -1);
                if let Some(cw) = tj["col_widths"].as_array() {
                    ts.col_widths = Some(cw.iter().filter_map(|x| x.as_i64().map(|n| n as i32)).collect());
                }
                ts.selected = tj["selected"].as_i64().unwrap_or(-1) as i32;
            }
            ts.hover_row = -1; ts.clicked_row = -1;
            // Die Ansicht aus den geladenen Zeilen bauen. Fehlte bis
            // 2026-09-21: eine Tabelle aus einer .dhform zeigte KEINE Zeile,
            // bis irgendetwas Sortieren oder Filtern anstiess.
            ts.rebuild_view();
            w.tbl = Some(Box::new(ts));
        }
        if kind == Kind::Tree {
            let mut ts = TreeState::default();
            if let Some(tj) = wj.get("tree") {
                if let Some(ns) = tj["nodes"].as_array() {
                    ts.nodes = ns.iter().map(|nj| TreeNode {
                        label: nj["label"].as_str().unwrap_or("").to_string(),
                        parent: nj["parent"].as_i64().unwrap_or(-1) as i32,
                        level: nj["level"].as_i64().unwrap_or(0) as i32,
                        expanded: nj["expanded"].as_bool().unwrap_or(false),
                        has_children: nj["has_children"].as_bool().unwrap_or(false),
                        icon: -1,   // Textur-Handles stehen nicht in der Datei
                        color: nj["color"].as_i64().unwrap_or(-1),
                        sym: nj["sym"].as_str().filter(|s| SINNBILDER.contains(s)).unwrap_or("").to_string(),
                    }).collect();
                }
                ts.selected = tj["selected"].as_i64().unwrap_or(-1) as i32;
                ts.multi = tj["multi"].as_bool().unwrap_or(false);
                ts.kaestchen = tj["kaestchen"].as_bool().unwrap_or(false);
                ts.bearbeitbar = tj["bearbeitbar"].as_bool().unwrap_or(false);
                ts.leer_text = tj["leer_text"].as_str().unwrap_or("").to_string();
            }
            if let Some(fj) = wj.get("filetree") {
                let liste = |k: &str| fj[k].as_array().map(|a| a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>()).unwrap_or_default();
                ts.datei = Some(Box::new(DateiBaum {
                    wurzel: fj["wurzel"].as_str().unwrap_or("").to_string(),
                    muster: liste("muster"), ausnahmen: liste("ausnahmen"), offen: liste("offen"),
                    ordner_zuerst: fj["ordner_zuerst"].as_bool().unwrap_or(true),
                    verborgene: fj["verborgene"].as_bool().unwrap_or(false),
                    nur_ordner: fj["nur_ordner"].as_bool().unwrap_or(false),
                    bild_ordner: -1, bild_datei: -1,
                    farben: fj["farben"].as_array().map(|a| a.iter().filter_map(|p| {
                        Some((p.get(0)?.as_str()?.to_string(), p.get(1)?.as_i64()?))
                    }).collect()).unwrap_or_default(),
                    symbole: fj["symbole"].as_array().map(|a| a.iter().filter_map(|p| {
                        let s = p.get(1)?.as_str()?;
                        if !SINNBILDER.contains(&s) { return None; }
                        Some((p.get(0)?.as_str()?.to_string(), s.to_string()))
                    }).collect()).unwrap_or_default(),
                    nach_art: fj["nach_art"].as_bool().unwrap_or(false),
                    ..Default::default()
                }));
                ts.selected = -1;
                dateibaum_neu(&mut ts);
            }
            ts.hover = -1; ts.anker = -1; ts.sync();
            w.tree = Some(Box::new(ts));
        }
        if kind == Kind::TimePicker {
            w.zeit_sek = wj["seconds"].as_bool().unwrap_or(false);
            if let Some(t) = wj["time"].as_str() {
                let teile: Vec<i32> = t.split(':').filter_map(|x| x.parse().ok()).collect();
                for (k, v) in teile.iter().take(3).enumerate() { w.zeit[k] = (*v).clamp(0, 59); }
                w.zeit[0] = w.zeit[0].clamp(0, 23);
            }
        }
        if kind == Kind::RichText {
            let rj = wj.get("rich");
            w.rich = Some(Box::new(RichState {
                quelle: rj.and_then(|r| r["quelle"].as_str()).unwrap_or("").to_string(),
                basis: rj.and_then(|r| r["groesse"].as_i64()).unwrap_or(0) as i32,
                code_font: -1,
                code_knoepfe: rj.and_then(|r| r["code_knoepfe"].as_str()).unwrap_or("")
                    .split('|').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect(),
                ..Default::default()
            }));
        }
        if let Some(v) = wj["variant"].as_str() {
            if kind == Kind::Button {
                if let Some(k) = KNOPF_ARTEN.iter().position(|x| *x == v) { w.variante = k as u8; }
            }
        }
        if let Some(s) = wj["symbol"].as_str() {
            if kind == Kind::Button { if let Ok(n) = sinnbild_pruefen(s, "GUI_LOAD") { w.sinnbild = n; } }
        }
        if kind == Kind::StatusBar {
            let mut s = StatusState::default();
            for fj in wj.get("status").and_then(|v| v["felder"].as_array()).map(|a| a.as_slice()).unwrap_or(&[]) {
                s.felder.push(StatusFeld {
                    text: fj["text"].as_str().unwrap_or("").to_string(),
                    tip: fj["tip"].as_str().unwrap_or("").to_string(),
                    breite: fj["breite"].as_i64().unwrap_or(0).max(0) as i32,
                    ausr: Self::ausrichtung_lesen(fj["ausrichtung"].as_str().unwrap_or(""), "GUI_LOAD").unwrap_or(0),
                    klickbar: fj["klickbar"].as_bool().unwrap_or(false),
                });
            }
            if s.felder.is_empty() {
                s.felder.push(StatusFeld { text: w.text.clone(), tip: String::new(), breite: 0, ausr: 0, klickbar: false });
            }
            w.status = Some(Box::new(s));
        }
        if kind == Kind::Breadcrumb {
            let mut p = PfadState::default();
            if let Some(pj) = wj.get("pfad") {
                p.texte = pj["texte"].as_array().map(|a| a.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default();
                let n = p.texte.len();
                p.daten = pj["daten"].as_array().map(|a| a.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default();
                p.daten.resize(n, String::new());
                p.breiten = vec![0; n];
            }
            w.pfad = Some(Box::new(p));
        }
        if kind == Kind::Toolbar {
            if let Some(lj) = wj.get("leiste") {
                let mut l = LeisteState::default();
                for ej in lj["eintraege"].as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
                    let art = match ej["art"].as_str().unwrap_or("knopf") { "trenner" => 1, "luecke" => 2, _ => 0 };
                    let mut e = Self::tb_leer(art);
                    if let Some(s) = ej["symbol"].as_str() { if let Ok(n) = sinnbild_pruefen(s, "GUI_LOAD") { e.symbol = n; } }
                    e.tip = ej["tip"].as_str().unwrap_or("").to_string();
                    e.text = ej["text"].as_str().unwrap_or("").to_string();
                    e.aus = ej["gesperrt"].as_bool().unwrap_or(false);
                    e.kippbar = ej["kippbar"].as_bool().unwrap_or(false);
                    e.an = e.kippbar && ej["an"].as_bool().unwrap_or(false);
                    l.eintraege.push(e);
                }
                l.mit_text = lj["beschriftung"].as_bool().unwrap_or(false);
                l.symbol = lj["symbolgroesse"].as_i64().unwrap_or(0) as i32;
                if !l.eintraege.is_empty() { w.enabled = true; }
                w.leiste = Some(Box::new(l));
            }
        }
        if kind == Kind::Accordion {
            let mut a = AkkState { umgeschaltet: -1, ..Default::default() };
            if let Some(aj) = wj.get("akkordeon") {
                if let Some(o) = aj["offen"].as_array() { a.offen = o.iter().map(|x| x.as_bool().unwrap_or(false)).collect(); }
                if let Some(o) = aj["hoehen"].as_array() { a.hoehen = o.iter().map(|x| x.as_i64().unwrap_or(0) as i32).collect(); }
                a.mehrere = aj["mehrere"].as_bool().unwrap_or(false);
                if let Some(l) = aj["lage"].as_array() {
                    a.lage = l.iter().filter_map(|e| {
                        let e = e.as_array()?;
                        Some((e.first()?.as_i64()? as usize, e.get(1)?.as_i64()? as i32, e.get(2)?.as_i64()? as i32))
                    }).collect();
                }
            }
            let n = w.items.len();
            a.offen.resize(n, false);
            a.hoehen.resize(n, 0);
            w.akk = Some(Box::new(a));
        }
        if kind == Kind::Wizard {
            let mut z = WzState::default();
            if let Some(zj) = wj.get("assistent") {
                z.pruefen = zj["pruefen"].as_bool().unwrap_or(false);
                if let Some(tx) = zj["texte"].as_array() {
                    for (k, s) in tx.iter().take(4).enumerate() { if let Some(s) = s.as_str() { z.texte[k] = s.to_string(); } }
                }
            }
            w.wz = Some(Box::new(z));
        }
        if matches!(kind, Kind::TabControl | Kind::Accordion | Kind::Wizard) {
            let mut st = TabCtlState::default();
            if let Some(tj) = wj.get("tabctl") {
                if let Some(ks) = tj["kinder"].as_array() {
                    st.kinder = ks.iter().filter_map(|e| {
                        let a = e.as_array()?;
                        Some((a.first()?.as_i64()? as usize, a.get(1)?.as_i64()? as i32))
                    }).collect();
                }
                w.sel = tj["seite"].as_i64().unwrap_or(0) as i32;
                st.schliessbar = tj["schliessbar"].as_bool().unwrap_or(false);
            }
            w.tabctl = Some(Box::new(st));
        }
        w.tab_page = wj["tab_page"].as_i64().unwrap_or(-1) as i32;
        w.anchor = wj["anchor"].as_str().map(Self::anchor_mask).unwrap_or(5);
        w.bx = w.x; w.by = w.y; w.bw = w.w; w.bh = w.h;   // Anchor-Basis = Design-Rechteck
        Ok(w)
    }
    /// Ein Fenster (inkl. lebender Widgets) als JSON-String (GUI_SAVE/TO_JSON).
    pub fn to_json(&self, h: i64) -> Result<String, String> {
        let win = self.windows.get(h as usize)
            .filter(|w| w.alive)
            .ok_or("GUI_SAVE/GUI_TO_JSON: ungueltiges GUI_WINDOW-Handle")?;
        let widgets: Vec<serde_json::Value> =
            win.widgets.iter().filter(|w| w.alive).map(|w| self.widget_json(w)).collect();
        // Untermenues werden VERSCHACHTELT geschrieben (`items` am Eintrag),
        // nicht als Index -- so bleibt die Datei lesbar und von Hand aenderbar.
        let menus: Vec<serde_json::Value> = win.menus.iter().filter(|m| !m.unter).map(|m| serde_json::json!({
            "label": m.label, "in_bar": m.in_bar,
            "items": self.menu_items_json(win, m),
        })).collect();
        let obj = serde_json::json!({
            "title": win.title,
            "x": self.unsk(win.x), "y": self.unsk(win.y),
            "w": self.unsk(win.w), "h": self.unsk(win.h),
            "movable": win.movable, "closable": win.closable, "visible": win.visible,
            "resizable": win.resizable, "chrome": win.chrome,
            "min_w": self.unsk(win.min_w), "min_h": self.unsk(win.min_h),
            "max_w": self.unsk(win.max_w), "max_h": self.unsk(win.max_h),
            "widgets": widgets,
            "menus": menus, "tabs": win.tabs, "active_tab": win.active_tab,
            "tabs_closable": win.tabs_closable,
            "default_button": win.default_btn, "cancel_button": win.cancel_btn,
        });
        // Nur mit Namen -- sonst bliebe jede alte .dhform beim Sichern nicht gleich.
        let mut obj = obj;
        if !win.name.is_empty() { obj["name"] = serde_json::json!(win.name); }
        serde_json::to_string_pretty(&obj).map_err(|e| format!("GUI_SAVE: {}", e))
    }
    /// Aus JSON ein neues Fenster bauen, Handle zurueck (GUI_LOAD/FROM_JSON).
    pub fn from_json(&mut self, s: &str) -> Result<i64, String> {
        let v: serde_json::Value = serde_json::from_str(s)
            .map_err(|e| format!("GUI_LOAD: ungueltiges JSON: {}", e))?;
        let gi = |k: &str, d: i64| v[k].as_i64().unwrap_or(d) as i32;
        let title = v["title"].as_str().unwrap_or("").to_string();
        let h = self.new_window(title, gi("x", 0), gi("y", 0), gi("w", 200), gi("h", 150));
        let wi = h as usize;
        self.windows[wi].movable = v["movable"].as_bool().unwrap_or(true);
        self.windows[wi].name = v["name"].as_str().unwrap_or("").to_lowercase();
        self.windows[wi].closable = v["closable"].as_bool().unwrap_or(false);
        self.windows[wi].visible = v["visible"].as_bool().unwrap_or(true);
        self.windows[wi].resizable = v["resizable"].as_bool().unwrap_or(false);
        self.windows[wi].chrome = v["chrome"].as_bool().unwrap_or(true);
        self.windows[wi].min_w = self.sk(gi("min_w", 0)); self.windows[wi].min_h = self.sk(gi("min_h", 0));
        self.windows[wi].max_w = self.sk(gi("max_w", 0)); self.windows[wi].max_h = self.sk(gi("max_h", 0));
        if let Some(arr) = v["widgets"].as_array() {
            for wj in arr {
                let mut wdg = Self::widget_from_json(wj)?;
                // Dieser Weg geht NICHT durch `add_widget` -- der Massstab muss
                // hier eigens angewandt werden, sonst laege eine geladene Form
                // als einzige in Originalgroesse im vergroesserten Fenster.
                wdg.x = self.sk(wdg.x); wdg.y = self.sk(wdg.y);
                wdg.w = self.sk(wdg.w); wdg.h = self.sk(wdg.h);
                wdg.bx = wdg.x; wdg.by = wdg.y; wdg.bw = wdg.w; wdg.bh = wdg.h;
                if let Some(t) = wdg.tbl.as_mut() {
                    t.row_h = self.sk(t.row_h); t.header_h = self.sk(t.header_h);
                    t.filter_h = self.sk(t.filter_h);
                    if let Some(cw) = t.col_widths.as_mut() {
                        for b in cw.iter_mut() { *b = self.sk(*b); }
                    }
                }
                self.windows[wi].widgets.push(wdg);
            }
        }
        if let Some(ms) = v["menus"].as_array() {
            for mj in ms {
                let mi = self.windows[wi].menus.len();
                self.windows[wi].menus.push(Menu {
                    label: mj["label"].as_str().unwrap_or("").to_string(),
                    in_bar: mj["in_bar"].as_bool().unwrap_or(true),
                    unter: false, items: Vec::new(), auf_t: 0.0, b: Blende::default(), ziel: -1,
                });
                let items = match mj["items"].as_array() {
                    Some(its) => Self::menu_items_aus_json(&mut self.windows[wi], its),
                    None => Vec::new(),
                };
                self.windows[wi].menus[mi].items = items;
            }
        }
        if let Some(ts) = v["tabs"].as_array() {
            self.windows[wi].tabs = ts.iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
        }
        self.windows[wi].active_tab = v["active_tab"].as_i64().unwrap_or(0) as i32;
        self.windows[wi].tabs_closable = v["tabs_closable"].as_bool().unwrap_or(false);
        let n = self.windows[wi].widgets.len() as i64;
        let knopf = |k: &str| -> i32 {
            let i = v[k].as_i64().unwrap_or(-1);
            if i >= 0 && i < n { i as i32 } else { -1 }
        };
        self.windows[wi].default_btn = knopf("default_button");
        self.windows[wi].cancel_btn = knopf("cancel_button");
        Ok(h)
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
        let key = name.to_lowercase();
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
