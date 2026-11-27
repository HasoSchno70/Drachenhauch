//! Layouts (Zeilen, Spalten, Raster, Formular): Mindestmasse und Verteilen. Thema und Metriken stehen in gui_aussehen.rs, die `.dhform` in gui_form.rs.
//!
//! Teil von `gui.rs`: dort stehen die Typen, hier die Methoden dieses Themas.

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
        if w.kind != Kind::Layout { return Err(format!("{}: das Widget ist kein GUI_LAYOUT", fn_)); }
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
        match super::einstellungen::schluessel("GUI_LAYOUT_SET", key, super::einstellungen::LAYOUT)? {
            "abstand" => { if wert < 0.0 { return Err("GUI_LAYOUT_SET: abstand muss >= 0 sein".into()); } l.abstand = wert as i32; }
            "rand" => { if wert < 0.0 { return Err("GUI_LAYOUT_SET: rand muss >= 0 sein".into()); } l.rand = wert as i32; }
            "ausrichtung" => {
                if !(0.0..=2.0).contains(&wert) { return Err("GUI_LAYOUT_SET: ausrichtung ist 0 (Anfang), 1 (Mitte) oder 2 (Ende)".into()); }
                l.ausrichtung = wert as u8;
            }
            "dehnen" => l.dehnen = wert != 0.0,
            "rahmen" => l.rahmen = wert != 0.0,
            "auswahl" => { l.auswahl = wert != 0.0; if !l.auswahl { l.gewaehlt = -1; } }
            _ => unreachable!("Tafel und Arme laufen auseinander"),
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
    ///
    /// Mit Grafik wird vorher gemessen wie in GUI_UPDATE: Beschriftungen
    /// und Felder "nach Inhalt" kennen ihre Breite erst mit der Schrift.
    /// Bis 2026-11 lieferte die Frage direkt nach dem Aufbau (vor dem
    /// ersten GUI_UPDATE) die Schaetzung -- 272x160 statt 403x240 fuer das
    /// Formular aus Kapitel 2 von "Fenster, Formulare, Daten", und
    /// WINDOW_MIN_SIZE liess das Fenster zu klein ziehen.
    /// GUI_TEXT_WIDTH(widget, text): so breit, wie DIESES Widget `text`
    /// zeichnet (seine Schrift, Groesse, Stil, der Massstab) -- logisch, wie
    /// alle Lagen nach aussen. Ohne das schaetzte ein Programm Spalten- und
    /// Knopfbreiten an der Zeichenzahl (das Studio: "Ganze Za" im
    /// Spalten-Gitter, Buch "Fenster, Formulare, Daten", Kapitel 13).
    pub fn text_breite_messen(&self, g: Option<&Graphics>, h: i64, s: &str) -> Result<i64, String> {
        let w = self.wdg(h, "GUI_TEXT_WIDTH")?;
        let g = g.ok_or("GUI_TEXT_WIDTH: braucht ein Fenster (SCREEN)")?;
        Ok(self.unsk(self.wtext_width(g, w, s)) as i64)
    }
    pub fn layout_min_messen(&mut self, g: Option<&Graphics>, h: i64) -> Result<(i64, i64), String> {
        self.wdg(h, "GUI_LAYOUT_MIN_W")?;
        if let Some(g) = g { self.layout_pass(g); }
        self.layout_min(h)
    }
    pub fn layout_min(&self, h: i64) -> Result<(i64, i64), String> {
        let (wi, i) = Self::dec_widget(h);
        let w = self.wdg(h, "GUI_LAYOUT_MIN_W")?;
        if w.kind != Kind::Layout { return Err("GUI_LAYOUT_MIN_W: das Widget ist kein GUI_LAYOUT".into()); }
        let (mw, mh) = self.min_mass(wi, i, 0);
        Ok((self.unsk(mw) as i64, self.unsk(mh) as i64))
    }
    /// GUI_GET_MIN_W / GUI_GET_MIN_H: die mit GUI_SET_MIN_SIZE gesetzte
    /// Mindestgroesse (logisch), 0 = keine.
    pub fn get_min_size(&self, h: i64, f: &str) -> Result<(i64, i64), String> {
        let w = self.wdg(h, f)?;
        Ok((self.unsk(w.min_w) as i64, self.unsk(w.min_h) as i64))
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

}
