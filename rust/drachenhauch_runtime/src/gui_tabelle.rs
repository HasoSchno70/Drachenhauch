//! Die Tabelle: Zellen, Zeilen, Gitter, Mehrfachauswahl, Spaltenreihenfolge, Bearbeiten, Sortieren und Filtern.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Tabelle ---
    pub(super) fn tbl_mut(&mut self, h: i64, fn_: &str) -> Result<&mut TableState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Table { return Err(format!("{}: Widget ist keine Tabelle", fn_)); }
        Ok(w.tbl.as_mut().unwrap())
    }
    pub(super) fn tbl_ref(&self, h: i64, fn_: &str) -> Result<&TableState, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::Table { return Err(format!("{}: Widget ist keine Tabelle", fn_)); }
        Ok(w.tbl.as_ref().unwrap())
    }
    pub fn table(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Table, x, y, w, h);
        // Zeilen-/Kopf-/Filterhoehe sind Pixel und muessen mit dem Massstab
        // wachsen. TableState::default() kennt ihn nicht (es ist kein Gui),
        // darum hier -- an der einen Stelle, an der eine Tabelle entsteht.
        wd.tbl = Some(Box::new(TableState {
            selected: -1, hover_row: -1, clicked_row: -1,
            row_h: self.sk(TBL_ROW_H), header_h: self.sk(TBL_HEADER_H),
            filter_h: self.sk(TBL_FILTER_H),
            ..Default::default()
        }));
        self.add_widget(win, "GUI_TABLE", wd)
    }
    pub fn table_set_headers(&mut self, h: i64, headers: Vec<String>) -> Result<(), String> {
        // Keine Laengenpruefung mehr: die Spaltenzahl ergibt sich aus der
        // BREITESTEN Angabe (TableState::n_cols), kuerzere Zeilen sind einfach
        // leer. Vorher war jede Abweichung ein Fehler -- und wer die Zeilen VOR
        // dem Kopf setzte, umging die Pruefung ganz und brachte das Zeichnen
        // zum Absturz. Ueber die eine Quelle kann kein Index danebengreifen.
        let t = self.tbl_mut(h, "GUI_TABLE_HEADERS")?;
        t.headers = headers;
        t.rebuild_view();
        Ok(())
    }
    pub fn table_set_rows(&mut self, h: i64, rows: Vec<Vec<String>>) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_ROWS")?;
        let old_n = t.rows.len();
        t.rows = rows.into_iter()
            .map(|r| r.into_iter().map(Cell::text).collect())
            .collect();
        t.row_fg = vec![-1; t.rows.len()];
        t.row_bg = vec![-1; t.rows.len()];
        if t.selected >= t.rows.len() as i32 { t.selected = -1; }
        // Die Auswahl zeigte sonst auf Zeilen, die es nicht mehr gibt.
        t.sel_rows.retain(|&r| r < t.rows.len() as i32);
        t.sel_anchor = -1;
        // Schrumpft die Tabelle, muss der Scroll zurueck -- sonst bliebe eine
        // weit heruntergescrollte Tabelle bis zur naechsten Mausbewegung leer.
        // Nur beim Schrumpfen, sonst waere ein periodisch aktualisiertes
        // Scoreboard nie scrollbar.
        if t.rows.len() < old_n { t.scroll_y = 0; }
        t.rebuild_view();
        Ok(())
    }
    pub fn table_set_col_widths(&mut self, h: i64, widths: Option<Vec<i32>>) -> Result<(), String> {
        // Spaltenbreiten sind Pixel und muessen mitwachsen -- sonst bliebe die
        // Tabelle bei Massstab 2 als einziges Element in Originalbreite stehen.
        let widths = widths.map(|v| v.into_iter().map(|b| self.sk(b)).collect::<Vec<i32>>());
        let t = self.tbl_mut(h, "GUI_TABLE_COL_WIDTHS")?;
        if let Some(ref cw) = widths {
            let n = t.n_cols();
            if n != 0 && cw.len() != n {
                return Err(format!("GUI_TABLE_COL_WIDTHS: {} Breiten, die Tabelle hat {} Spalten",
                                   cw.len(), n));
            }
        }
        t.col_widths = widths;
        Ok(())
    }

    // --- Zellen einzeln ------------------------------------------------------

    /// Zeile/Spalte pruefen und melden, WELCHE Angabe daneben lag -- ein blosses
    /// "Index ungueltig" laesst einen beide durchsuchen.
    pub(super) fn tbl_rc(t: &TableState, r: i64, c: i64, fn_: &str) -> Result<(usize, usize), String> {
        if r < 0 || r >= t.rows.len() as i64 {
            return Err(format!("{}: Zeile {} gibt es nicht (0..{})",
                               fn_, r, t.rows.len().saturating_sub(1)));
        }
        let n = t.n_cols();
        if c < 0 || c >= n as i64 {
            return Err(format!("{}: Spalte {} gibt es nicht (0..{})",
                               fn_, c, n.saturating_sub(1)));
        }
        Ok((r as usize, c as usize))
    }

    pub fn table_set_cell(&mut self, h: i64, r: i64, c: i64, text: String) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SET_CELL")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_SET_CELL")?;
        t.cell_mut(r, c).unwrap().text = text;
        t.rebuild_view();   // Filter/Sortierung haengen am Text
        Ok(())
    }
    pub fn table_get_cell(&self, h: i64, r: i64, c: i64) -> Result<String, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_GET_CELL")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_GET_CELL")?;
        Ok(t.cell(r, c).map(|x| x.text.clone()).unwrap_or_default())
    }
    /// Farben je Zelle. -1 laesst die jeweilige Farbe beim Thema bzw. bei der
    /// Zeilenfarbe -- so faerbt man nur den Vordergrund, ohne die Flaeche zu
    /// verlieren.
    pub fn table_cell_color(&mut self, h: i64, r: i64, c: i64, fg: i64, bg: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_COLOR")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_COLOR")?;
        let cell = t.cell_mut(r, c).unwrap();
        cell.fg = fg;
        cell.bg = bg;
        Ok(())
    }
    pub fn table_cell_align(&mut self, h: i64, r: i64, c: i64, wie: &str) -> Result<(), String> {
        let a = parse_align(wie).ok_or_else(|| format!(
            "GUI_TABLE_CELL_ALIGN: '{}' unbekannt -- links, mitte oder rechts", wie))?;
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_ALIGN")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_ALIGN")?;
        t.cell_mut(r, c).unwrap().align = a;
        Ok(())
    }
    /// Ganze Zeile faerben -- ein Aufruf statt einer je Spalte.
    pub fn table_row_color(&mut self, h: i64, r: i64, fg: i64, bg: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_ROW_COLOR")?;
        if r < 0 || r >= t.rows.len() as i64 {
            return Err(format!("GUI_TABLE_ROW_COLOR: Zeile {} gibt es nicht (0..{})",
                               r, t.rows.len().saturating_sub(1)));
        }
        let r = r as usize;
        while t.row_fg.len() <= r { t.row_fg.push(-1); }
        while t.row_bg.len() <= r { t.row_bg.push(-1); }
        t.row_fg[r] = fg;
        t.row_bg[r] = bg;
        Ok(())
    }
    pub fn table_cell_kind(&mut self, h: i64, r: i64, c: i64, art: &str) -> Result<(), String> {
        let k = CellKind::parse(art).ok_or_else(|| format!(
            "GUI_TABLE_CELL_KIND: '{}' unbekannt -- text, bild, haken, balken, knopf", art))?;
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_KIND")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_KIND")?;
        t.cell_mut(r, c).unwrap().kind = k;
        Ok(())
    }
    /// Bild in die Zelle. Setzt die Art gleich mit -- sonst muesste man zwei
    /// Aufrufe machen und der erste allein bliebe wirkungslos.
    pub fn table_cell_image(&mut self, h: i64, r: i64, c: i64, bild: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_IMAGE")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_IMAGE")?;
        let cell = t.cell_mut(r, c).unwrap();
        cell.image = bild;
        cell.kind = CellKind::Image;
        Ok(())
    }
    /// Wert einer Haken- oder Balkenzelle (Haken: 0/1, Balken: 0..1).
    pub fn table_cell_value(&mut self, h: i64, r: i64, c: i64, v: f64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_VALUE")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_VALUE")?;
        t.cell_mut(r, c).unwrap().value = v;
        Ok(())
    }
    pub fn table_get_value(&self, h: i64, r: i64, c: i64) -> Result<f64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_GET_VALUE")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_GET_VALUE")?;
        Ok(t.cell(r, c).map(|x| x.value).unwrap_or(0.0))
    }
    pub fn table_col_align(&mut self, h: i64, c: i64, wie: &str) -> Result<(), String> {
        let a = parse_align(wie).ok_or_else(|| format!(
            "GUI_TABLE_COL_ALIGN: '{}' unbekannt -- links, mitte oder rechts", wie))?;
        let t = self.tbl_mut(h, "GUI_TABLE_COL_ALIGN")?;
        if c < 0 { return Err("GUI_TABLE_COL_ALIGN: Spalte darf nicht negativ sein".into()); }
        let c = c as usize;
        while t.col_align.len() <= c { t.col_align.push(0); }
        t.col_align[c] = a;
        Ok(())
    }

    // --- Zeilen anfuegen / entfernen ----------------------------------------

    pub fn table_add_row(&mut self, h: i64, zellen: Vec<String>) -> Result<i64, String> {
        let t = self.tbl_mut(h, "GUI_TABLE_ADD_ROW")?;
        t.rows.push(zellen.into_iter().map(Cell::text).collect());
        t.row_fg.push(-1);
        t.row_bg.push(-1);
        t.zeile_angehaengt();
        Ok(t.rows.len() as i64 - 1)
    }
    pub fn table_remove_row(&mut self, h: i64, r: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_REMOVE_ROW")?;
        if r < 0 || r >= t.rows.len() as i64 {
            return Err(format!("GUI_TABLE_REMOVE_ROW: Zeile {} gibt es nicht (0..{})",
                               r, t.rows.len().saturating_sub(1)));
        }
        let r = r as usize;
        if t.baum {
            // Die Kinder der entfernten Zeile ruecken zu deren Eltern auf,
            // alle Nummern dahinter um eins nach vorn.
            t.baum_sync();
            let neu_e = t.eltern[r];
            t.eltern.remove(r);
            t.offen.remove(r);
            for e in t.eltern.iter_mut() {
                if *e == r as i32 { *e = if neu_e > r as i32 { neu_e - 1 } else { neu_e }; }
                else if *e > r as i32 { *e -= 1; }
            }
        }
        t.rows.remove(r);
        if r < t.row_fg.len() { t.row_fg.remove(r); }
        if r < t.row_bg.len() { t.row_bg.remove(r); }
        // Sonst zeigt die Auswahl danach auf eine ANDERE Zeile als vorher --
        // alle Indizes hinter der geloeschten ruecken auf.
        if t.selected == r as i32 { t.selected = -1; }
        else if t.selected > r as i32 { t.selected -= 1; }
        t.sel_rows.retain(|&x| x != r as i32);
        for x in t.sel_rows.iter_mut() { if *x > r as i32 { *x -= 1; } }
        if t.sel_anchor == r as i32 { t.sel_anchor = -1; }
        else if t.sel_anchor > r as i32 { t.sel_anchor -= 1; }
        t.rebuild_view();
        Ok(())
    }
    pub fn table_clear(&mut self, h: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CLEAR")?;
        t.rows.clear();
        t.eltern.clear(); t.offen.clear();
        t.row_fg.clear();
        t.row_bg.clear();
        t.sel_rows.clear();
        t.sel_anchor = -1;
        t.selected = -1;
        t.hover_row = -1;
        t.scroll_x = 0;
        t.scroll_y = 0;
        t.view.clear();
        Ok(())
    }

    // --- Tabellenweite Einstellungen ----------------------------------------

    /// Ein Setter statt einem Builtin je Schalter -- wie bei `chart`. Ein
    /// unbekannter Schluessel zaehlt die gueltigen auf, statt still nichts zu tun.
    pub fn table_set_opt(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        // Massstab vor dem Zugriff auf die Tabelle merken: `sk` liest `self`,
        // und `tbl_mut` haelt es bis zum Ende veraenderlich in der Hand.
        let ms = self.scale;
        let skal = |v: i32| if ms == 1.0 { v } else { ((v as f64 * ms).round() as i32).max(1) };
        let t = self.tbl_mut(h, "GUI_TABLE_SET")?;
        let n = wert as i32;
        match key.to_lowercase().as_str() {
            // Untergrenze: eine Zeilenhoehe von 0 waere eine Division durch null
            // im Treffertest, eine negative ein Absturz beim Zeichnen.
            "zeilenhoehe" | "row_height" => t.row_h = skal(n.clamp(8, 400)),
            "kopfhoehe" | "header_height" => t.header_h = skal(n.clamp(0, 200)),
            "zebra" => t.zebra = n != 0,
            "gitter" | "grid" => t.grid = n != 0,
            "filterzeile" | "filter_row" => { t.filter_row = n != 0; if n == 0 { t.filter_focus = -1; } }
            "sortierbar" | "sortable" => t.sortable = n != 0,
            "spalten_ziehbar" | "resizable_cols" => t.resizable_cols = n != 0,
            "feste_spalten" | "frozen" => t.frozen = n.max(0),
            "spalten_verschiebbar" | "reorderable" => t.reorderable = n != 0,
            "baum" | "tree" => { t.baum = n != 0; t.rebuild_view(); }
            "zellmodus" | "cells" => {
                t.zellmodus = n != 0;
                if !t.zellmodus { t.ber_r = -1; t.ber_c = -1; }
            }
            "zeilen_anhaengen" | "append_rows" => t.zeilen_anhaengen = n != 0,
            "mehrfachauswahl" | "multi_select" => {
                t.multi = n != 0;
                // Beim Einschalten die bisherige Einzelauswahl uebernehmen,
                // sonst waere sie nach dem Umschalten ploetzlich weg.
                if t.multi {
                    t.sel_rows.clear();
                    if t.selected >= 0 { t.sel_rows.push(t.selected); }
                }
            }
            _ => return Err(format!(
                "GUI_TABLE_SET: '{}' unbekannt -- zeilenhoehe, kopfhoehe, zebra, gitter, \
filterzeile, sortierbar, spalten_ziehbar, feste_spalten, spalten_verschiebbar, mehrfachauswahl, \
zellmodus, zeilen_anhaengen", key)),
        }
        Ok(())
    }

    // --- Gitter (Stufe 30) ---------------------------------------------------

    /// Eine Tabelle, die als Gitter anfaengt: Zellmodus, jede Spalte
    /// bearbeitbar, `zeilen` leere Zeilen.
    #[allow(clippy::too_many_arguments)]
    pub fn grid(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32,
                headers: Vec<String>, zeilen: i64) -> Result<i64, String> {
        if !(0..=100_000).contains(&zeilen) {
            return Err(format!("GUI_GRID: Zeilenzahl {} ausserhalb 0..100000", zeilen));
        }
        let t = self.table(win, x, y, w, h)?;
        let n = headers.len();
        let ts = self.tbl_mut(t, "GUI_GRID")?;
        ts.headers = headers;
        ts.zellmodus = true;
        ts.col_edit = vec![true; n];
        for _ in 0..zeilen {
            ts.rows.push((0..n).map(|_| Cell::text(String::new())).collect());
            ts.row_fg.push(-1);
            ts.row_bg.push(-1);
        }
        ts.rebuild_view();
        Ok(t)
    }
    pub fn table_current(&self, h: i64, spalte: bool) -> Result<i64, String> {
        let t = self.tbl_ref(h, if spalte { "GUI_TABLE_CURRENT_COL" } else { "GUI_TABLE_CURRENT_ROW" })?;
        Ok(if spalte { t.cur_c } else { t.cur_r } as i64)
    }
    pub(super) fn zelle_pruefen(t: &TableState, r: i64, c: i64, fn_: &str) -> Result<(), String> {
        if r < 0 || r >= t.rows.len() as i64 || c < 0 || c >= t.n_cols() as i64 {
            return Err(format!("{}: Zelle ({}, {}) gibt es nicht -- {} Zeilen, {} Spalten",
                               fn_, r, c, t.rows.len(), t.n_cols()));
        }
        Ok(())
    }
    pub fn table_set_current(&mut self, h: i64, r: i64, c: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SET_CURRENT")?;
        if r == -1 && c == -1 {
            t.cur_r = -1; t.cur_c = -1; t.ber_r = -1; t.ber_c = -1;
            return Ok(());
        }
        Self::zelle_pruefen(t, r, c, "GUI_TABLE_SET_CURRENT")?;
        t.cur_r = r as i32; t.cur_c = c as i32;
        t.ber_r = t.cur_r; t.ber_c = t.cur_c;
        t.selected = t.cur_r;
        t.sicht_holen = true;
        Ok(())
    }
    /// (Anker-Zeile, Anker-Spalte, Zeile, Spalte) -- alles Datenangaben.
    pub fn table_range(&self, h: i64) -> Result<(i64, i64, i64, i64), String> {
        let t = self.tbl_ref(h, "GUI_TABLE_RANGE")?;
        if t.cur_r < 0 { return Ok((-1, -1, -1, -1)); }
        let (ar, ac) = if t.ber_r >= 0 { (t.ber_r, t.ber_c) } else { (t.cur_r, t.cur_c) };
        Ok((ar as i64, ac as i64, t.cur_r as i64, t.cur_c as i64))
    }
    pub fn table_select_range(&mut self, h: i64, ar: i64, ac: i64, r: i64, c: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SELECT_RANGE")?;
        Self::zelle_pruefen(t, ar, ac, "GUI_TABLE_SELECT_RANGE")?;
        Self::zelle_pruefen(t, r, c, "GUI_TABLE_SELECT_RANGE")?;
        t.ber_r = ar as i32; t.ber_c = ac as i32;
        t.cur_r = r as i32; t.cur_c = c as i32;
        t.selected = t.cur_r;
        t.sicht_holen = true;
        Ok(())
    }
    pub fn table_copy(&self, h: i64) -> Result<String, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_COPY$")?.bereich_text())
    }
    pub fn table_paste(&mut self, h: i64, text: &str) -> Result<i64, String> {
        let t = self.tbl_mut(h, "GUI_TABLE_PASTE")?;
        if t.cur_r < 0 || t.view_of(t.cur_r) < 0 {
            // Ohne aktuelle Zelle ab oben links -- auch in ein leeres Gitter.
            t.cur_r = t.data_row(0);
            t.cur_c = t.order().first().map(|&c| c as i32).unwrap_or(-1);
        }
        Ok(t.einfuegen_text(text))
    }
    pub fn table_col_type(&mut self, h: i64, c: i64, art: &str) -> Result<(), String> {
        let typ = match art.to_lowercase().as_str() {
            "text" => 0,
            "ganz" | "ganzzahl" | "int" | "integer" => 1,
            "zahl" | "float" | "number" => 2,
            "auswahl" | "liste" | "choice" => 3,
            _ => return Err(format!("GUI_TABLE_COL_TYPE: '{}' unbekannt -- text, ganz, zahl, auswahl", art)),
        };
        let t = self.tbl_mut(h, "GUI_TABLE_COL_TYPE")?;
        if !(0..1000).contains(&c) { return Err(format!("GUI_TABLE_COL_TYPE: Spalte {} ausserhalb 0..999", c)); }
        let c = c as usize;
        if t.col_typ.len() <= c { t.col_typ.resize(c + 1, 0); }
        t.col_typ[c] = typ;
        Ok(())
    }
    pub fn table_col_choices(&mut self, h: i64, c: i64, eintraege: Vec<String>) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_COL_CHOICES")?;
        if !(0..1000).contains(&c) { return Err(format!("GUI_TABLE_COL_CHOICES: Spalte {} ausserhalb 0..999", c)); }
        let c = c as usize;
        if t.col_wahl.len() <= c { t.col_wahl.resize(c + 1, Vec::new()); }
        t.col_wahl[c] = eintraege;
        // Eine Liste ohne Art waere wirkungslos -- sie macht die Spalte zur Auswahl.
        if t.col_typ.len() <= c { t.col_typ.resize(c + 1, 0); }
        t.col_typ[c] = 3;
        Ok(())
    }

    /// Die aktuelle Zelle ins Bild rollen.
    pub(super) fn tabelle_zeigen(&mut self, wi: usize, i: usize) {
        let gm = self.table_geom(wi, i);
        let t = match self.windows[wi].widgets[i].tbl.as_mut() { Some(t) => t, None => return };
        t.sicht_holen = false;
        let vi = t.view_of(t.cur_r);
        if vi >= 0 && gm.row_h > 0 {
            let y = vi * gm.row_h;
            if y < t.scroll_y { t.scroll_y = y; }
            if y + gm.row_h > t.scroll_y + gm.body_h { t.scroll_y = y + gm.row_h - gm.body_h; }
            t.scroll_y = t.scroll_y.clamp(0, gm.max_scroll_y.max(0));
        }
        let p = t.pos_of(t.cur_c);
        if p >= 0 && (p as usize) >= gm.frozen && (p as usize) < gm.col_widths.len() {
            let x: i32 = gm.col_widths[..p as usize].iter().sum();
            let w = gm.col_widths[p as usize];
            if x - t.scroll_x < gm.frozen_w { t.scroll_x = x - gm.frozen_w; }
            if x + w - t.scroll_x > gm.body_w { t.scroll_x = x + w - gm.body_w; }
            t.scroll_x = t.scroll_x.clamp(0, gm.max_scroll_x.max(0));
        }
    }
    /// Je Bild: Tabellen, deren aktuelle Zelle ein Aufruf ohne Geometrie
    /// gesetzt hat, rollen sie jetzt ins Bild.
    pub(super) fn tabellen_pass(&mut self) {
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                if self.windows[wi].widgets[i].tbl.as_ref().map(|t| t.sicht_holen).unwrap_or(false) {
                    self.tabelle_zeigen(wi, i);
                }
            }
        }
    }
    pub fn table_get_opt(&self, h: i64, key: &str) -> Result<f64, String> {
        let ms = self.scale;
        let entskal = |v: i32| if ms == 1.0 { v } else { (v as f64 / ms).round() as i32 };
        let t = self.tbl_ref(h, "GUI_TABLE_GET")?;
        Ok(match key.to_lowercase().as_str() {
            "zeilenhoehe" | "row_height" => entskal(t.row_h) as f64,
            "kopfhoehe" | "header_height" => entskal(t.header_h) as f64,
            "zebra" => if t.zebra { 1.0 } else { 0.0 },
            "gitter" | "grid" => if t.grid { 1.0 } else { 0.0 },
            "filterzeile" | "filter_row" => if t.filter_row { 1.0 } else { 0.0 },
            "sortierbar" | "sortable" => if t.sortable { 1.0 } else { 0.0 },
            "spalten_ziehbar" | "resizable_cols" => if t.resizable_cols { 1.0 } else { 0.0 },
            "feste_spalten" | "frozen" => t.frozen as f64,
            "spalten_verschiebbar" | "reorderable" => if t.reorderable { 1.0 } else { 0.0 },
            "mehrfachauswahl" | "multi_select" => if t.multi { 1.0 } else { 0.0 },
            "zellmodus" | "cells" => if t.zellmodus { 1.0 } else { 0.0 },
            "baum" | "tree" => if t.baum { 1.0 } else { 0.0 },
            "zeilen_anhaengen" | "append_rows" => if t.zeilen_anhaengen { 1.0 } else { 0.0 },
            "spalten" | "columns" => t.n_cols() as f64,
            _ => return Err(format!(
                "GUI_TABLE_GET: '{}' unbekannt -- zeilenhoehe, kopfhoehe, zebra, gitter, \
filterzeile, sortierbar, spalten_ziehbar, feste_spalten, spalten_verschiebbar, mehrfachauswahl, \
zellmodus, zeilen_anhaengen, spalten", key)),
        })
    }

    // --- Mehrfachauswahl -----------------------------------------------------

    // --- Spalten-Reihenfolge -------------------------------------------------

    /// Spalte von einer Anzeige-Position auf eine andere schieben.
    pub fn table_move_col(&mut self, h: i64, von: i64, nach: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_MOVE_COL")?;
        let n = t.n_cols() as i64;
        if von < 0 || von >= n || nach < 0 || nach >= n {
            return Err(format!("GUI_TABLE_MOVE_COL: Position ausserhalb 0..{}", n.saturating_sub(1)));
        }
        t.move_col(von as usize, nach as usize);
        Ok(())
    }
    /// Welche DATENspalte steht an der Anzeige-Position `pos`?
    pub fn table_col_at(&self, h: i64, pos: i64) -> Result<i64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_COL_AT")?;
        Ok(t.order().get(pos.max(0) as usize).map(|&c| c as i64).unwrap_or(-1))
    }
    /// An welcher Anzeige-Position steht die DATENspalte `c`?
    pub fn table_col_pos(&self, h: i64, c: i64) -> Result<i64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_COL_POS")?;
        Ok(t.order().iter().position(|&x| x as i64 == c).map(|p| p as i64).unwrap_or(-1))
    }
    /// Reihenfolge zuruecksetzen.
    pub fn table_reset_cols(&mut self, h: i64) -> Result<(), String> {
        self.tbl_mut(h, "GUI_TABLE_RESET_COLS")?.col_order.clear();
        Ok(())
    }

    pub fn table_sel_count(&self, h: i64) -> Result<i64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_SEL_COUNT")?;
        Ok(if t.multi { t.sel_rows.len() as i64 }
           else if t.selected >= 0 { 1 } else { 0 })
    }
    /// i-te gewaehlte DATENzeile, in der Reihenfolge des Anklickens.
    pub fn table_sel_row(&self, h: i64, i: i64) -> Result<i64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_SEL_ROW")?;
        if !t.multi { return Ok(if i == 0 { t.selected as i64 } else { -1 }); }
        Ok(t.sel_rows.get(i.max(0) as usize).copied().unwrap_or(-1) as i64)
    }
    pub fn table_is_selected(&self, h: i64, r: i64) -> Result<bool, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_IS_SELECTED")?.ist_gewaehlt(r as i32))
    }
    /// Zeile zur Auswahl hinzunehmen bzw. herausnehmen (programmgesteuert).
    pub fn table_select(&mut self, h: i64, r: i64, an: bool) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SELECT")?;
        if r < 0 || r >= t.rows.len() as i64 {
            return Err(format!("GUI_TABLE_SELECT: Zeile {} gibt es nicht (0..{})",
                               r, t.rows.len().saturating_sub(1)));
        }
        let r = r as i32;
        let drin = t.sel_rows.contains(&r);
        if an != drin { t.auswahl_umschalten(r); }
        if an { t.selected = r; }
        Ok(())
    }
    pub fn table_clear_selection(&mut self, h: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CLEAR_SELECTION")?;
        t.auswahl_setzen(-1);
        Ok(())
    }

    // --- Zellen bearbeiten ---------------------------------------------------

    /// Spalte fuer die Bearbeitung freigeben. Ohne Freigabe passiert beim
    /// Doppelklick nichts -- eine Tabelle, in der man versehentlich ueberall
    /// hineinschreiben kann, waere schlimmer als eine ohne Bearbeitung.
    pub fn table_col_edit(&mut self, h: i64, c: i64, an: bool) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_COL_EDIT")?;
        if c < 0 { return Err("GUI_TABLE_COL_EDIT: Spalte darf nicht negativ sein".into()); }
        let c = c as usize;
        while t.col_edit.len() <= c { t.col_edit.push(false); }
        t.col_edit[c] = an;
        Ok(())
    }
    /// Zeile der Zelle, die gerade bearbeitet wird (-1 = keine).
    pub fn table_editing_row(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_EDITING_ROW")?.edit_row as i64)
    }
    pub fn table_editing_col(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_EDITING_COL")?.edit_col as i64)
    }

    // --- Sortieren und Filtern ----------------------------------------------

    /// Programmgesteuert sortieren. Spalte < 0 hebt die Sortierung auf.
    pub fn table_sort(&mut self, h: i64, col: i64, desc: bool) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SORT")?;
        let n = t.n_cols() as i64;
        t.sort_col = if col >= 0 && col < n { col as i32 } else { -1 };
        t.sort_desc = desc;
        t.rebuild_view();
        Ok(())
    }
    pub fn table_sort_col(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_SORT_COL")?.sort_col as i64)
    }
    pub fn table_sort_desc(&self, h: i64) -> Result<bool, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_SORT_DESC")?.sort_desc)
    }
    /// Filter je Spalte setzen (leerer Text = kein Filter).
    pub fn table_placeholder(&mut self, h: i64, text: String) -> Result<(), String> {
        self.tbl_mut(h, "GUI_TABLE_PLACEHOLDER")?.leer_text = text;
        Ok(())
    }
    pub fn table_filter(&mut self, h: i64, col: i64, text: String) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_FILTER")?;
        if col < 0 { return Err("GUI_TABLE_FILTER: Spalte darf nicht negativ sein".into()); }
        let c = col as usize;
        while t.filters.len() <= c { t.filters.push(String::new()); }
        t.filters[c] = text;
        t.rebuild_view();
        t.scroll_y = 0;
        Ok(())
    }
    pub fn table_get_filter(&self, h: i64, col: i64) -> Result<String, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_GET_FILTER")?;
        Ok(t.filters.get(col.max(0) as usize).cloned().unwrap_or_default())
    }
    /// Wieviele Zeilen sind nach Filter/Sortierung sichtbar?
    pub fn table_view_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_VIEW_COUNT")?.view.len() as i64)
    }
    /// Ansichtszeile -> Datenzeile. Damit laesst sich die Tabelle in der
    /// SICHTBAREN Reihenfolge durchgehen, ohne dass die Daten umgestellt
    /// werden muessten.
    pub fn table_view_row(&self, h: i64, i: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_VIEW_ROW")?.data_row(i as i32) as i64)
    }

    pub fn table_selected(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_SELECTED")?.selected as i64)
    }
    pub fn table_set_selected(&mut self, h: i64, row: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_SET_SELECTED")?;
        let n = t.rows.len() as i64;
        let r = if row >= 0 && row < n { row as i32 } else { -1 };
        // Ueber auswahl_setzen, damit die Mehrfachauswahl nicht daneben
        // stehenbleibt und beide Angaben auseinander laufen.
        t.auswahl_setzen(r);
        Ok(())
    }
    pub fn table_clicked(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_CLICKED")?.clicked_row as i64)
    }
    pub fn table_clicked_col(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_CLICKED_COL")?.clicked_col as i64)
    }
    pub fn table_row_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.tbl_ref(h, "GUI_TABLE_ROW_COUNT")?.rows.len() as i64)
    }
}
