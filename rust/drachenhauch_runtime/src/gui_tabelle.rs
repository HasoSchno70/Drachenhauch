//! Die Tabelle: Geometrie (`table_geom`, die eine Quelle), Zellen, Zeilen, Gitter, Mehrfachauswahl, Spaltenreihenfolge, Bedienung, Bearbeiten, Sortieren und Filtern.
//!
//! Teil von `gui.rs`: dort stehen die Typen, hier die Methoden dieses Themas.

use super::*;

impl Gui {
    // --- Tabelle ---
    pub(super) fn tbl_mut(&mut self, h: i64, fn_: &str) -> Result<&mut TableState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Table { return Err(format!("{}: das Widget ist kein GUI_TABLE", fn_)); }
        Ok(w.tbl.as_mut().unwrap())
    }
    pub(super) fn tbl_ref(&self, h: i64, fn_: &str) -> Result<&TableState, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::Table { return Err(format!("{}: das Widget ist kein GUI_TABLE", fn_)); }
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
        // Eigene Zeilen beenden den virtuellen Modus.
        let old_n = t.zeilen_zahl();
        t.virt = None;
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

    /// GUI_TABLE_GET_COL_WIDTHS(tbl): die Breiten, die gerade gelten -- die
    /// eigenen oder die verteilten, in Reihenfolge der DATENspalten und in
    /// logischen Punkten (wie GUI_TABLE_COL_WIDTHS sie nimmt).
    pub fn table_get_col_widths(&self, h: i64) -> Result<Vec<i64>, String> {
        self.tbl_ref(h, "GUI_TABLE_GET_COL_WIDTHS")?;
        let (wi, i) = Self::dec_widget(h);
        let gm = self.table_geom(wi, i);
        let mut roh = vec![0i64; gm.n_cols];
        for (p, &c) in gm.order.iter().enumerate() {
            if c < roh.len() && p < gm.col_widths.len() { roh[c] = self.unsk(gm.col_widths[p]) as i64; }
        }
        Ok(roh)
    }

    // --- Zellen einzeln ------------------------------------------------------

    /// Zeile/Spalte pruefen und melden, WELCHE Angabe daneben lag -- ein blosses
    /// "Index ungueltig" laesst einen beide durchsuchen.
    pub(super) fn tbl_rc(t: &TableState, r: i64, c: i64, fn_: &str) -> Result<(usize, usize), String> {
        if r < 0 || r >= t.zeilen_zahl() as i64 {
            return Err(format!("{}: Zeile {} gibt es nicht (0..{})",
                               fn_, r, t.zeilen_zahl().saturating_sub(1)));
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
        t.cell_mut(r, c).unwrap().text = text.into();
        t.zelle_geaendert(r, c);   // Filter/Sortierung haengen am Text
        Ok(())
    }
    pub fn table_get_cell(&self, h: i64, r: i64, c: i64) -> Result<String, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_GET_CELL")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_GET_CELL")?;
        Ok(t.cell(r, c).map(|x| x.text.to_string()).unwrap_or_default())
    }
    /// Farben je Zelle. -1 laesst die jeweilige Farbe beim Thema bzw. bei der
    /// Zeilenfarbe -- so faerbt man nur den Vordergrund, ohne die Flaeche zu
    /// verlieren.
    pub fn table_cell_color(&mut self, h: i64, r: i64, c: i64, fg: i64, bg: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_COLOR")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_COLOR")?;
        let cell = t.cell_mut(r, c).unwrap();
        cell.set_fg(fg);
        cell.set_bg(bg);
        Ok(())
    }
    /// GUI_TABLE_GET_CELL_COLOR(tbl, row, column, role$) -- die Farbe, die
    /// GUI_TABLE_CELL_COLOR der Zelle gegeben hat (`fg`/`bg`), -1 = keine.
    /// Fehlte bis 2026-11: ein Test konnte die Faerbung einer Zelle nur am
    /// Bild nachmessen (Buch "Fenster, Formulare, Daten", Kapitel 3).
    pub fn table_get_cell_color(&self, h: i64, r: i64, c: i64, rolle: &str) -> Result<i64, String> {
        const F: &str = "GUI_TABLE_GET_CELL_COLOR";
        let t = self.tbl_ref(h, F)?;
        let (r, c) = Self::tbl_rc(t, r, c, F)?;
        let zelle = t.cell(r, c);
        match farb_rolle(rolle, F)? {
            true => Ok(zelle.map(|z| z.fg()).unwrap_or(-1)),
            false => Ok(zelle.map(|z| z.bg()).unwrap_or(-1)),
        }
    }
    /// GUI_TABLE_GET_ROW_COLOR(tbl, row, role$) -- die Farbe aus
    /// GUI_TABLE_ROW_COLOR (`fg`/`bg`), -1 = keine.
    pub fn table_get_row_color(&self, h: i64, r: i64, rolle: &str) -> Result<i64, String> {
        const F: &str = "GUI_TABLE_GET_ROW_COLOR";
        let t = self.tbl_ref(h, F)?;
        if r < 0 || r >= t.zeilen_zahl() as i64 {
            return Err(format!("{}: Zeile {} gibt es nicht (0..{})", F, r, t.zeilen_zahl().saturating_sub(1)));
        }
        let r = r as usize;
        let farben = if farb_rolle(rolle, F)? { &t.row_fg } else { &t.row_bg };
        Ok(farben.get(r).copied().unwrap_or(-1))
    }
    pub fn table_cell_align(&mut self, h: i64, r: i64, c: i64, wie: &str) -> Result<(), String> {
        let a = parse_align(wie).ok_or_else(|| format!(
            "GUI_TABLE_CELL_ALIGN: '{}' unbekannt -- links, mitte oder rechts", wie))?;
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_ALIGN")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_ALIGN")?;
        t.cell_mut(r, c).unwrap().set_align(a);
        Ok(())
    }
    /// Ganze Zeile faerben -- ein Aufruf statt einer je Spalte.
    pub fn table_row_color(&mut self, h: i64, r: i64, fg: i64, bg: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_ROW_COLOR")?;
        if t.virt.is_some() {
            return Err("GUI_TABLE_ROW_COLOR: nicht in einer virtuellen Tabelle -- GUI_TABLE_CELL_COLOR je gelieferter Zelle".into());
        }
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
        t.cell_mut(r, c).unwrap().set_kind(k);
        Ok(())
    }
    /// Bild in die Zelle. Setzt die Art gleich mit -- sonst muesste man zwei
    /// Aufrufe machen und der erste allein bliebe wirkungslos.
    pub fn table_cell_image(&mut self, h: i64, r: i64, c: i64, bild: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_IMAGE")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_IMAGE")?;
        let cell = t.cell_mut(r, c).unwrap();
        cell.set_image(bild);
        cell.set_kind(CellKind::Image);
        Ok(())
    }
    /// Wert einer Haken- oder Balkenzelle (Haken: 0/1, Balken: 0..1).
    pub fn table_cell_value(&mut self, h: i64, r: i64, c: i64, v: f64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_CELL_VALUE")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_CELL_VALUE")?;
        t.cell_mut(r, c).unwrap().set_value(v);
        Ok(())
    }
    pub fn table_get_value(&self, h: i64, r: i64, c: i64) -> Result<f64, String> {
        let t = self.tbl_ref(h, "GUI_TABLE_GET_VALUE")?;
        let (r, c) = Self::tbl_rc(t, r, c, "GUI_TABLE_GET_VALUE")?;
        Ok(t.cell(r, c).map(|x| x.value()).unwrap_or(0.0))
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
        if t.virt.is_some() { return Err("GUI_TABLE_ADD_ROW: die Tabelle ist virtuell -- Zeilenzahl mit GUI_TABLE_VIRTUAL, Inhalt mit GUI_TABLE_SUPPLY".into()); }
        t.rows.push(zellen.into_iter().map(Cell::text).collect());
        t.row_fg.push(-1);
        t.row_bg.push(-1);
        t.zeile_angehaengt();
        Ok(t.rows.len() as i64 - 1)
    }
    pub fn table_remove_row(&mut self, h: i64, r: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_REMOVE_ROW")?;
        if t.virt.is_some() { return Err("GUI_TABLE_REMOVE_ROW: die Tabelle ist virtuell -- Zeilenzahl mit GUI_TABLE_VIRTUAL".into()); }
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
        t.virt = None;
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
        match super::einstellungen::schluessel("GUI_TABLE_SET", key, super::einstellungen::TABLE)? {
            // Untergrenze: eine Zeilenhoehe von 0 waere eine Division durch null
            // im Treffertest, eine negative ein Absturz beim Zeichnen.
            "zeilenhoehe" => t.row_h = skal(n.clamp(8, 400)),
            "kopfhoehe" => t.header_h = skal(n.clamp(0, 200)),
            "zebra" => t.zebra = n != 0,
            "gitter" => t.grid = n != 0,
            "filterzeile" => { t.filter_row = n != 0; if n == 0 { t.filter_focus = -1; } }
            "sortierbar" => t.sortable = n != 0,
            "spalten_ziehbar" => t.resizable_cols = n != 0,
            "feste_spalten" => t.frozen = n.max(0),
            "spalten_verschiebbar" => t.reorderable = n != 0,
            "baum" => { t.baum = n != 0; t.rebuild_view(); }
            "zellmodus" => {
                t.zellmodus = n != 0;
                if !t.zellmodus { t.ber_r = -1; t.ber_c = -1; }
            }
            "zeilen_anhaengen" => t.zeilen_anhaengen = n != 0,
            "spalten" => return Err("GUI_TABLE_SET: spalten laesst sich nur abfragen -- Spalten setzt GUI_TABLE_SET_HEADERS".into()),
            "mehrfachauswahl" => {
                t.multi = n != 0;
                // Beim Einschalten die bisherige Einzelauswahl uebernehmen,
                // sonst waere sie nach dem Umschalten ploetzlich weg.
                if t.multi {
                    t.sel_rows.clear();
                    if t.selected >= 0 { t.sel_rows.push(t.selected); }
                }
            }
            _ => unreachable!("Tafel und Arme laufen auseinander"),
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
        if r < 0 || r >= t.zeilen_zahl() as i64 || c < 0 || c >= t.n_cols() as i64 {
            return Err(format!("{}: Zelle ({}, {}) gibt es nicht -- {} Zeilen, {} Spalten",
                               fn_, r, c, t.zeilen_zahl(), t.n_cols()));
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
            "datum" | "date" => 4,
            _ => return Err(format!("GUI_TABLE_COL_TYPE: '{}' unbekannt -- text, ganz, zahl, auswahl, datum", art)),
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
        Ok(match super::einstellungen::schluessel("GUI_TABLE_GET", key, super::einstellungen::TABLE)? {
            "zeilenhoehe" => entskal(t.row_h) as f64,
            "kopfhoehe" => entskal(t.header_h) as f64,
            "zebra" => if t.zebra { 1.0 } else { 0.0 },
            "gitter" => if t.grid { 1.0 } else { 0.0 },
            "filterzeile" => if t.filter_row { 1.0 } else { 0.0 },
            "sortierbar" => if t.sortable { 1.0 } else { 0.0 },
            "spalten_ziehbar" => if t.resizable_cols { 1.0 } else { 0.0 },
            "feste_spalten" => t.frozen as f64,
            "spalten_verschiebbar" => if t.reorderable { 1.0 } else { 0.0 },
            "mehrfachauswahl" => if t.multi { 1.0 } else { 0.0 },
            "zellmodus" => if t.zellmodus { 1.0 } else { 0.0 },
            "baum" => if t.baum { 1.0 } else { 0.0 },
            "zeilen_anhaengen" => if t.zeilen_anhaengen { 1.0 } else { 0.0 },
            "spalten" => t.n_cols() as f64,
            _ => unreachable!("Tafel und Arme laufen auseinander"),
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
        if r < 0 || r >= t.zeilen_zahl() as i64 {
            return Err(format!("GUI_TABLE_SELECT: Zeile {} gibt es nicht (0..{})",
                               r, t.zeilen_zahl().saturating_sub(1)));
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
        Ok(self.tbl_ref(h, "GUI_TABLE_VIEW_COUNT")?.ansicht_len() as i64)
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
        let n = t.zeilen_zahl() as i64;
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
        Ok(self.tbl_ref(h, "GUI_TABLE_ROW_COUNT")?.zeilen_zahl() as i64)
    }

    /// GUI_TABLE_ROW_RECT(tb, row): wo die Datenzeile `row` gerade steht
    /// (x, y, Breite, Hoehe in Bildschirmpunkten wie GUI_HIT_TEST, wie die
    /// Liste -- fuer Pruefungen mit echten Klicks und Bildern) -- aus derselben
    /// Geometrie, mit der gezeichnet und getroffen wird (`table_geom`), samt
    /// Sortierung, Filter und Rollen. (0, 0, 0, 0), wenn ein Filter sie
    /// ausblendet. Wie GUI_LISTBOX_ITEM_RECT und GUI_TREE_ITEM_RECT.
    pub fn table_row_rect(&self, h: i64, row: i64, fn_: &str) -> Result<(i32, i32, i32, i32), String> {
        let t = self.tbl_ref(h, fn_)?;
        let n = t.zeilen_zahl() as i64;
        if row < 0 || row >= n {
            return Err(format!("{}: Zeile {} ausserhalb [0..{}]", fn_, row, n - 1));
        }
        let ansicht = match (0..t.ansicht_len()).find(|&r| t.ansicht(r) == Some(row as usize)) {
            Some(r) => r as i32,
            None => return Ok((0, 0, 0, 0)),
        };
        let (wi, idx) = Self::dec_widget(h);
        let gm = self.table_geom(wi, idx);
        let y = gm.body_y + ansicht * gm.row_h - t.scroll_y;
        Ok((gm.body_x, y, gm.body_w, gm.row_h))
    }

    // --- Virtueller Modus ----------------------------------------------------

    /// GUI_TABLE_VIRTUAL(tbl, zeilen): die Tabelle zeigt `zeilen` Zeilen,
    /// deren Inhalt das Programm auf Nachfrage liefert (GUI_TABLE_NEED_FROM,
    /// GUI_TABLE_SUPPLY). Nochmals gerufen: neue Zeilenzahl (etwa nach einem
    /// anderen Filter), alles Gelieferte gilt als veraltet. Eigene Zeilen
    /// (GUI_TABLE_ROWS, GUI_TABLE_CLEAR) beenden den Modus.
    pub fn table_virtual(&mut self, h: i64, zeilen: i64) -> Result<(), String> {
        let t = self.tbl_mut(h, "GUI_TABLE_VIRTUAL")?;
        // Die Bildlauf-Rechnung der Tabelle spricht Pixel in i32.
        let grenze = (i32::MAX / 2) as i64 / t.row_h.max(1) as i64;
        if zeilen < 0 || zeilen > grenze {
            return Err(format!("GUI_TABLE_VIRTUAL: Zeilenzahl {} ausserhalb 0..{}", zeilen, grenze));
        }
        let n = zeilen as usize;
        t.rows.clear();
        t.view.clear();
        t.row_fg.clear();
        t.row_bg.clear();
        t.baum = false;
        t.virt = Some(Box::new(Virtuell { n, cache: Default::default() }));
        if t.selected >= n as i32 { t.selected = -1; }
        t.sel_rows.retain(|&r| (r as usize) < n);
        if t.cur_r >= n as i32 { t.cur_r = -1; t.ber_r = -1; }
        if t.edit_row >= 0 { t.edit_row = -1; t.edit_col = -1; }
        let max = (n as i64 * t.row_h as i64).min(i32::MAX as i64) as i32;
        if t.scroll_y > max { t.scroll_y = 0; }
        Ok(())
    }

    /// Welche Zeilen braucht die virtuelle Tabelle gerade? (erste, Anzahl):
    /// die erste fehlende im sichtbaren Bereich samt einer Seite davor und
    /// danach, und wie viele ab dort am Stueck fehlen (hoechstens 1000).
    /// (-1, 0), wenn nichts fehlt oder die Tabelle nicht virtuell ist.
    /// Eine Zeile, die ein Aufruf ohne Geometrie ins Bild geholt hat
    /// (GUI_TABLE_SET_CURRENT), zaehlt schon -- so auch vor dem ersten Bild.
    pub fn table_need(&mut self, h: i64) -> Result<(i64, i64), String> {
        let (wi, i) = Self::dec_widget(h);
        if self.tbl_ref(h, "GUI_TABLE_NEED_FROM")?.virt.is_none() { return Ok((-1, 0)); }
        if self.windows[wi].widgets[i].tbl.as_ref().map(|t| t.sicht_holen).unwrap_or(false) { self.tabelle_zeigen(wi, i); }
        let (von, bis) = self.virtuell_bereich(wi, i);
        let t = self.tbl_ref(h, "GUI_TABLE_NEED_FROM")?;
        let Some(v) = t.virt.as_ref() else { return Ok((-1, 0)) };
        let Some(erste) = (von..bis).find(|r| !v.cache.contains_key(r)) else { return Ok((-1, 0)) };
        let n = (erste..bis).take_while(|r| !v.cache.contains_key(r)).take(1000).count();
        Ok((erste as i64, n as i64))
    }

    /// Sichtbarer Bereich einer virtuellen Tabelle samt einer Seite davor
    /// und danach: [von, bis) in Zeilen.
    fn virtuell_bereich(&self, wi: usize, i: usize) -> (usize, usize) {
        let gm = self.table_geom(wi, i);
        let Some(t) = self.windows[wi].widgets[i].tbl.as_ref() else { return (0, 0) };
        let n = t.ansicht_len();
        let rh = gm.row_h.max(1);
        let erste = (t.scroll_y / rh).max(0) as usize;
        let seite = ((gm.body_h / rh) + 1).max(1) as usize;
        (erste.saturating_sub(seite).min(n), (erste + 2 * seite).min(n))
    }

    /// GUI_TABLE_SUPPLY(tbl, ab, zellen): Zeilen ab `ab` liefern -- ein 2D
    /// ARRAY OF STRING (mehrere Zeilen) oder ein ARRAY OF STRING (eine).
    /// Weit vom sichtbaren Bereich gelieferte Zeilen vergisst die Tabelle
    /// wieder, sobald sie mehr als einige Seiten haelt.
    pub fn table_supply(&mut self, h: i64, ab: i64, zeilen: Vec<Vec<String>>) -> Result<(), String> {
        let (wi, i) = Self::dec_widget(h);
        let n = match self.tbl_ref(h, "GUI_TABLE_SUPPLY")?.virt.as_ref() {
            Some(v) => v.n,
            None => return Err("GUI_TABLE_SUPPLY: die Tabelle ist nicht virtuell (GUI_TABLE_VIRTUAL)".into()),
        };
        if ab < 0 || ab as usize > n {
            return Err(format!("GUI_TABLE_SUPPLY: Zeile {} gibt es nicht (0..{})", ab, n.saturating_sub(1)));
        }
        let (von, bis) = self.virtuell_bereich(wi, i);
        let seite = (bis - von).max(1);
        let t = self.tbl_mut(h, "GUI_TABLE_SUPPLY")?;
        let Some(v) = t.virt.as_deref_mut() else { return Ok(()) };
        for (k, z) in zeilen.into_iter().enumerate() {
            let r = ab as usize + k;
            if r >= n { break; }
            v.cache.insert(r, z.into_iter().map(Cell::text).collect());
        }
        // Vergessen, was weit weg liegt -- erst ab einem Vorrat, damit Hin-
        // und Herrollen nicht staendig neu fragt.
        let halten = (8 * seite).max(2000);
        if v.cache.len() > halten {
            let (a, b) = (von.saturating_sub(2 * seite), bis + 2 * seite);
            v.cache.retain(|&r, _| r >= a && r < b);
        }
        Ok(())
    }

    pub(super) fn table_geom(&self, wi: usize, idx: usize) -> TGeom {
        let w = &self.windows[wi].widgets[idx];
        let (ax, ay, ww, hh) = self.abs_rect(wi, w);
        let t = w.tbl.as_ref().unwrap();
        let n_cols = t.n_cols();
        // SICHTBARE Zeilen -- Filter und Sortierung bestimmen, was gezeichnet
        // und getroffen wird. Der Rest der Geometrie kennt keinen Unterschied.
        let n_rows = t.ansicht_len();
        let row_h = t.row_h;
        let header_h = t.head_total();
        // Mehr feste Spalten als vorhanden waeren ein leerer Scrollbereich --
        // deckeln statt darauf zu vertrauen, dass niemand es versucht.
        let frozen = (t.frozen.max(0) as usize).min(n_cols);
        let order = t.order();
        // Breiten haengen an der DATENspalte, gebraucht werden sie in
        // ANZEIGE-Reihenfolge -- hier einmal umgeordnet, damit alles weitere
        // schlicht ueber die Position laufen kann.
        let roh: Vec<i32> = match &t.col_widths {
            Some(cw) if cw.len() == n_cols => cw.clone(),
            _ => {
                let avail = ww - self.rollbalken_w() - 2;
                let per = if n_cols > 0 { (avail / n_cols as i32).max(self.sk(TBL_MIN_COL_W)) } else { 0 };
                vec![per; n_cols]
            }
        };
        let col_widths: Vec<i32> = order.iter().map(|&c| roh[c]).collect();
        let body_x_ltr = ax + 1;
        let body_y = ay + header_h;
        let body_w_raw = ww - 2;
        let body_h_raw = hh - header_h - 1;
        let total_w: i32 = col_widths.iter().sum();
        let total_h = n_rows as i32 * row_h;
        let mut need_v = total_h > body_h_raw;
        let need_h = total_w > body_w_raw - if need_v { self.rollbalken_w() } else { 0 };
        if need_h && total_h > body_h_raw - self.rollbalken_w() { need_v = true; }
        let body_w = body_w_raw - if need_v { self.rollbalken_w() } else { 0 };
        let body_h = body_h_raw - if need_h { self.rollbalken_w() } else { 0 };
        // Gespiegelt steht der senkrechte Balken links, der Rumpf rechts davon.
        let rtl = self.windows[wi].rtl;
        let body_x = if rtl && need_v { body_x_ltr + self.rollbalken_w() } else { body_x_ltr };
        let vbar_x = if rtl { ax + 1 } else { ax + body_w_raw - self.rollbalken_w() + 1 };
        let frozen_w: i32 = col_widths.iter().take(frozen).sum();
        TGeom {
            ax, ay, w: ww, h: hh, rtl, vbar_x, n_cols, n_rows, col_widths, row_h, header_h,
            scroll_x: t.scroll_x, frozen, frozen_w, order,
            body_x, body_y, body_w, body_h, body_w_raw, total_w, total_h,
            need_v, need_h,
            max_scroll_y: (total_h - body_h).max(0), max_scroll_x: (total_w - body_w).max(0),
        }
    }

    pub(super) fn handle_height(track: i32, visible: i32, total: i32) -> i32 {
        if total > 0 { ((track as f64 * (visible as f64 / total as f64)) as i32).max(16) } else { track }
    }

    pub(super) fn table_hover(&mut self, wi: usize, idx: usize, mx: i32, my: i32, g: &mut Graphics) {
        let gm = self.table_geom(wi, idx);
        let wheel = Self::rad(g);
        let over = Self::in_rect(mx, my, (gm.body_x, gm.body_y, gm.body_w, gm.body_h));
        let weich_an = self.m("uebergang") > 0;
        {
            // Weiches Rollen wie bei der Liste: Radschritte addieren sich aufs
            // Ziel, `uebergaenge` zieht nach.
            let w = &mut self.windows[wi].widgets[idx];
            let t = w.tbl.as_mut().unwrap();
            t.scroll_y = t.scroll_y.clamp(0, gm.max_scroll_y);
            t.scroll_x = t.scroll_x.clamp(0, gm.max_scroll_x);
            if wheel != 0 && over {
                let basis = w.rad_ziel.unwrap_or(t.scroll_y as f64);
                let ziel = (basis as i32 - wheel as i32 * gm.row_h).clamp(0, gm.max_scroll_y);
                if weich_an { w.rad_ziel = Some(ziel as f64); w.rad_letzt = t.scroll_y as f64; }
                else { t.scroll_y = ziel; }
            }
        }
        let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
        if !t.drag_v && !t.drag_h && over {
            let hv = (my - gm.body_y + t.scroll_y) / gm.row_h;
            // Nach aussen ist die Datenzeile gemeint -- eine sortierte Tabelle
            // wuerde sonst beim Hovern die falsche Zeile hervorheben.
            t.hover_row = if hv >= 0 && hv < gm.n_rows as i32 { t.data_row(hv) } else { -1 };
        }
        // Bereich mit der Maus aufziehen: die aktuelle Zelle folgt, der Anker
        // bleibt, wo der Druck war.
        if t.zell_zug {
            if g.mouse_button(0) {
                let c = Self::col_at(&gm, mx);
                if over && t.hover_row >= 0 && c >= 0 && t.edit_row < 0 {
                    t.cur_r = t.hover_row;
                    t.cur_c = c;
                    t.selected = t.cur_r;
                }
            } else {
                t.zell_zug = false;
            }
        }
    }

    pub(super) fn table_press(&mut self, wi: usize, idx: usize, mx: i32, my: i32) {
        // Klick im laufenden Eingabefeld gehoert dem Editor -- sonst wuerde er
        // hier zusaetzlich als Zeilen-Klick zaehlen.
        if self.editing_table == Some((wi, idx)) {
            if let Some(r) = self.edit_cell_rect(wi, idx) {
                if Self::in_rect(mx, my, r) { return; }
            }
        }
        let gm = self.table_geom(wi, idx);
        if gm.need_v {
            let sb_x = gm.vbar_x;
            if Self::in_rect(mx, my, (sb_x, gm.body_y, self.rollbalken_w(), gm.body_h)) {
                let off = Self::handle_height(gm.body_h, gm.body_h, gm.total_h) / 2;
                { let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap(); t.drag_v = true; t.drag_off = off; }
                self.active_table = Some((wi, idx));
                self.table_drag(wi, idx, mx, my);
                return;
            }
        }
        if gm.need_h {
            let hs_y = gm.ay + gm.h - self.rollbalken_w() - 1;
            if Self::in_rect(mx, my, (gm.body_x, hs_y, gm.body_w, self.rollbalken_w())) {
                let off = Self::handle_height(gm.body_w, gm.body_w, gm.total_w) / 2;
                { let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap(); t.drag_h = true; t.drag_off = off; }
                self.active_table = Some((wi, idx));
                self.table_drag(wi, idx, mx, my);
                return;
            }
        }
        // Klick in den Kopfbereich: sortieren bzw. Filterzelle fokussieren.
        // Kommt VOR dem Zeilen-Treffer, sonst waere der Kopf tot.
        let (titel_h, filter_an) = {
            let t = self.windows[wi].widgets[idx].tbl.as_ref().unwrap();
            (t.header_h, t.filter_row)
        };
        if my >= gm.ay && my < gm.ay + titel_h {
            // Zuerst pruefen, ob die Maus auf einer SPALTENKANTE liegt --
            // sonst wuerde jeder Griff nach der Kante die Spalte sortieren.
            let kante = Self::edge_at(&gm, mx, self.sk(TBL_EDGE_GRAB));
            if kante >= 0 {
                let breite = gm.col_widths[kante as usize];
                // `edge_at` liefert eine ANZEIGE-Position, die Breiten haengen
                // aber an der Datenspalte -- ohne diese Umrechnung zoege man
                // nach dem Umsortieren die falsche Spalte breit.
                let daten_c = gm.order[kante as usize] as i32;
                let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
                if t.resizable_cols {
                    t.drag_col = daten_c;
                    t.drag_col_w = breite;
                    t.drag_col_x = mx;
                    t.filter_focus = -1;
                    // Beim ersten Ziehen die berechneten Breiten festhalten,
                    // sonst haette die Tabelle gar keine eigenen Werte zum
                    // Aendern und spraenge beim Loslassen zurueck.
                    // Die berechneten Breiten festhalten -- in DATEN-Reihenfolge,
                    // denn so werden sie gespeichert.
                    if t.col_widths.is_none() {
                        let mut roh = vec![0; gm.n_cols];
                        for (p, &c) in gm.order.iter().enumerate() { roh[c] = gm.col_widths[p]; }
                        t.col_widths = Some(roh);
                    }
                    self.active_table = Some((wi, idx));
                    return;
                }
            }
            let pos = Self::pos_at(&gm, mx);
            let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
            t.filter_focus = -1;
            if pos >= 0 {
                // NICHT sofort sortieren: erst beim Loslassen steht fest, ob
                // die Geste ein Klick war oder ein Zug. Sonst wuerde jedes
                // Verschieben nebenbei auch noch sortieren.
                t.head_drag = Some((pos as usize, mx, false));
                self.active_table = Some((wi, idx));
            }
            return;
        }
        if filter_an && my >= gm.ay + titel_h && my < gm.ay + titel_h + self.sk(TBL_FILTER_H) {
            let c = Self::col_at(&gm, mx);
            let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
            t.filter_focus = c;
            // Das Tabellen-Widget bekommt den Tastaturfokus, damit die
            // Eingabe hier landet und nicht in einem Textfeld daneben.
            if c >= 0 { self.focus_widget = Some((wi, idx)); }
            return;
        }
        let hr = self.windows[wi].widgets[idx].tbl.as_ref().unwrap().hover_row;
        if hr >= 0 {
            // Baum: ein Klick auf das Dreieck (oder den Einzug davor) klappt
            // die Zeile um, statt sie zu waehlen.
            let umklappen = {
                let t = self.windows[wi].widgets[idx].tbl.as_ref().unwrap();
                if t.baum && t.hat_kinder.get(hr as usize).copied().unwrap_or(false) && Self::pos_at(&gm, mx) == 0 {
                    let (_, o) = self.tt_einzug(t, hr as usize);
                    mx < Self::col_x(&gm, 0) + o
                } else { false }
            };
            if umklappen {
                let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
                t.offen[hr as usize] = !t.offen[hr as usize];
                t.rebuild_view();
                return;
            }
            // Spalte aus der x-Lage bestimmen -- ohne sie liesse sich ein
            // Knopf in Spalte 3 nicht von einem Haken in Spalte 1 trennen.
            let hc = Self::col_at(&gm, mx);
            self.windows[wi].widgets[idx].tbl.as_mut().unwrap().hover_col = hc;
            // Im Zellmodus setzt der Druck die aktuelle Zelle -- mit Umschalt
            // bleibt der Anker stehen und der Bereich waechst.
            let (_, shift) = self.tasten_mod;
            {
                let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
                if t.zellmodus && hc >= 0 {
                    t.cur_r = hr;
                    t.cur_c = hc;
                    if !shift || t.ber_r < 0 { t.ber_r = hr; t.ber_c = hc; }
                    t.zell_zug = true;
                }
            }
            // Doppelklick auf eine bearbeitbare Textzelle -> Bearbeiten.
            // Sonst ist er "oeffnen" wie bei der Liste: die Zeile wird
            // gewaehlt, und GUI_DOUBLE_CLICKED meldet es ein Bild lang.
            // Nicht auf einem Knopf oder Haken in der Zelle: zweimal schnell
            // "+1" sind zwei Klicks auf den Knopf, kein "oeffnen" der Zeile
            // (gefunden mit der Kuehlschrank-App -- es ging der Dialog auf).
            let bedienzelle = hc >= 0 && self.windows[wi].widgets[idx].tbl.as_ref().unwrap()
                .cell(hr as usize, hc as usize)
                .map(|z| matches!(z.kind(), CellKind::Button | CellKind::Check)).unwrap_or(false);
            if self.dbl_click && hc >= 0 && !bedienzelle {
                self.table_begin_edit(wi, idx, hr, hc);
                let bearbeitet = self.windows[wi].widgets[idx].tbl.as_ref().map(|t| t.edit_row >= 0).unwrap_or(false);
                if bearbeitet { return; }
                self.windows[wi].widgets[idx].tbl.as_mut().unwrap().doppel = true;
            }
            self.table_press = Some((wi, idx, hr));
        }
    }

    /// Bearbeitung starten. Laeuft nur auf TEXT-Zellen einer freigegebenen
    /// Spalte: ein Haken schaltet ohnehin per Klick um, ein Balken oder Bild
    /// hat keinen sinnvollen Text zum Tippen.
    pub(super) fn table_begin_edit(&mut self, wi: usize, idx: usize, r: i32, c: i32) {
        let t = match self.windows[wi].widgets[idx].tbl.as_mut() { Some(t) => t, None => return };
        if r < 0 || c < 0 || r as usize >= t.zeilen_zahl() { return; }
        if !t.col_editable(c as usize) { return; }
        let art = t.cell(r as usize, c as usize).map(|x| x.kind()).unwrap_or(CellKind::Text);
        if art != CellKind::Text { return; }
        t.edit_text = t.cell(r as usize, c as usize).map(|x| x.text.to_string()).unwrap_or_default();
        // Eine Auswahlspalte bekommt statt des Eingabefelds ihre Liste; ohne
        // Eintraege gibt es nichts zu waehlen.
        if t.col_typ_of(c as usize) == 3 {
            let w = t.col_wahl.get(c as usize).cloned().unwrap_or_default();
            if w.is_empty() { t.edit_text.clear(); return; }
            t.wahl_pos = w.iter().position(|x| *x == t.edit_text).map(|p| p as i32).unwrap_or(0);
        }
        t.edit_caret = t.edit_text.chars().count() as i32;
        // Beim Oeffnen alles markiert -- so ersetzt das erste Tippen den
        // bisherigen Inhalt, statt ihn zu verlaengern. Genau das erwartet man,
        // wenn man eine Zelle zum Ueberschreiben aufmacht.
        t.edit_anchor = 0;
        t.edit_scroll = 0;
        t.edit_maus_sperre = true;
        t.edit_row = r;
        t.edit_col = c;
        t.selected = r;
        self.editing_table = Some((wi, idx));
        // Die Tabelle braucht den Tastaturfokus, sonst landen die Zeichen im
        // Textfeld daneben.
        self.focus_widget = Some((wi, idx));
        // Eine Datumsspalte klappt dazu den Monat auf (getippt werden darf
        // trotzdem); er beginnt beim Datum der Zelle, sonst heute.
        let datum = self.windows[wi].widgets[idx].tbl.as_ref()
            .filter(|t| t.col_typ_of(c as usize) == 4)
            .map(|t| crate::kalender::parse(&t.edit_text));
        if let Some(d) = datum {
            self.datum_zeigt = match d {
                Some((j, m, t)) => [j, m, t],
                None => {
                    let (j, m, t, _, _, _) = crate::builtins::local_datetime();
                    [j as i32, m as i32, t as i32]
                }
            };
            self.open_dropdown = None;
            self.datum_auf = Some((wi, idx, true));
        }
    }

    /// Bearbeitung beenden. `uebernehmen` = Text in die Zelle schreiben.
    pub(super) fn table_end_edit(&mut self, wi: usize, idx: usize, uebernehmen: bool) {
        if self.datum_auf == Some((wi, idx, true)) { self.datum_auf = None; }
        let mut geaendert = false;
        {
            let t = match self.windows[wi].widgets[idx].tbl.as_mut() { Some(t) => t, None => return };
            if t.edit_row < 0 { return; }
            let (r, c) = (t.edit_row as usize, t.edit_col as usize);
            if uebernehmen {
                let neu = std::mem::take(&mut t.edit_text);
                // Eine Zahlenspalte laesst beim Tippen Zwischenstaende wie "-"
                // zu; uebernommen wird nur, was wirklich passt.
                if t.wert_ok(c, &neu) {
                    let neu = if t.col_typ_of(c) == 0 { neu } else { neu.trim().to_string() };
                    if let Some(cell) = t.cell_mut(r, c) {
                        if cell.text != neu { cell.text = neu.into(); geaendert = true; }
                    }
                }
            }
            t.wahl_pos = -1;
            t.edit_row = -1;
            t.edit_col = -1;
            t.edit_text.clear();
            t.edit_caret = 0;
            t.edit_anchor = 0;
            t.edit_scroll = 0;
            // Erst JETZT neu sortieren/filtern. Waehrend des Tippens waere die
            // Zeile bei jedem Zeichen weggesprungen.
            if geaendert { t.zelle_geaendert(r, c); }
        }
        self.editing_table = None;
        if geaendert {
            let f = self.windows[wi].widgets[idx].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Linke Bildschirmkante der Spalte `c`. Feste Spalten scrollen nicht mit.
    /// EINE Quelle fuer Treffertest und Zeichnen.
    pub(super) fn col_x(gm: &TGeom, c: usize) -> i32 {
        let bis = c.min(gm.col_widths.len());
        let mut x = gm.body_x + gm.col_widths[..bis].iter().sum::<i32>();
        if c >= gm.frozen { x -= gm.scroll_x; }
        if gm.rtl { x = Self::tbl_spiegel(gm, x, gm.col_widths.get(c).copied().unwrap_or(0)); }
        x
    }

    /// Linke Kante eines Stuecks der Breite `b`, das links bei `x` laege --
    /// gespiegelt um die Mitte des Rumpfs.
    pub(super) fn tbl_spiegel(gm: &TGeom, x: i32, b: i32) -> i32 {
        2 * gm.body_x + gm.body_w - x - b
    }

    /// Bereich, in dem die Spalte `c` sichtbar sein darf. Ohne dieses Clipping
    /// schoebe sich eine gescrollte Spalte unter den festen Block.
    pub(super) fn col_clip(gm: &TGeom, c: usize) -> (i32, i32) {
        let (x, b) = if c < gm.frozen {
            (gm.body_x, gm.frozen_w)
        } else {
            (gm.body_x + gm.frozen_w, (gm.body_w - gm.frozen_w).max(0))
        };
        if gm.rtl { (Self::tbl_spiegel(gm, x, b), b) } else { (x, b) }
    }

    /// Liegt `mx` auf der rechten Kante einer Spalte? Liefert deren Index.
    /// Der Fangbereich ist bewusst breiter als ein Pixel -- eine Kante, die
    /// man auf den Punkt genau treffen muss, findet niemand.
    pub(super) fn edge_at(gm: &TGeom, mx: i32, grab: i32) -> i32 {
        for c in 0..gm.col_widths.len() {
            // Die Kante liegt am ENDE der Spalte -- gespiegelt links.
            let rechts = if gm.rtl { Self::col_x(gm, c) } else { Self::col_x(gm, c) + gm.col_widths[c] };
            if (mx - rechts).abs() <= grab { return c as i32; }
        }
        -1
    }

    /// Bildschirm-x -> Spaltenindex (-1 daneben). Rechnet mit derselben
    /// Geometrie wie das Zeichnen, damit Treffer und Anzeige nicht auseinander
    /// laufen -- derselbe Grund wie bei `chart`.
    /// Bildschirm-x -> ANZEIGE-Position (-1 daneben).
    pub(super) fn pos_at(gm: &TGeom, mx: i32) -> i32 {
        for p in 0..gm.col_widths.len() {
            let (kx, kw) = Self::col_clip(gm, p);
            // Nur treffen, was in seinem Bereich auch SICHTBAR ist -- sonst
            // truefe man eine Spalte, die unter dem festen Block liegt.
            if mx < kx || mx >= kx + kw { continue; }
            let x = Self::col_x(gm, p);
            if mx >= x && mx < x + gm.col_widths[p] { return p as i32; }
        }
        -1
    }

    /// Bildschirm-x -> DATENspalte. Alles nach aussen spricht von Datenspalten;
    /// nur Zeichnen und Treffertest rechnen in Positionen.
    pub(super) fn col_at(gm: &TGeom, mx: i32) -> i32 {
        let p = Self::pos_at(gm, mx);
        if p < 0 { return -1; }
        gm.order.get(p as usize).map(|&c| c as i32).unwrap_or(-1)
    }

    /// Tastatur fuer die Tabelle. Eine laufende Zellbearbeitung hat Vorrang --
    /// sonst landeten die Zeichen in einem Filterfeld, das vorher den Fokus
    /// hatte.
    pub(super) fn table_keys(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let (bearbeitet, wahl, filter) = self.windows[wi].widgets[i].tbl.as_ref()
            .map(|t| (t.edit_row >= 0, t.edit_row >= 0 && t.col_typ_of(t.edit_col.max(0) as usize) == 3,
                      t.filter_row && t.filter_focus >= 0))
            .unwrap_or((false, false, false));
        if wahl { self.table_wahl_keys(wi, i, g); }
        else if bearbeitet { self.table_edit_keys(wi, i, g); }
        else if filter { self.table_filter_keys(wi, i, g); }
        else { self.table_nav_keys(wi, i, g); }
    }

    /// Weiterruecken nach dem Bestaetigen: `dv` Zeilen, `dp` Zellen (am
    /// Zeilenende in die naechste Zeile). Unter der letzten Zeile haengt
    /// `anhaengen` eine neue an, wenn die Tabelle es erlaubt.
    pub(super) fn table_schritt(&mut self, wi: usize, i: usize, dv: i32, dp: i32, anhaengen: bool) {
        let t = match self.windows[wi].widgets[i].tbl.as_mut() { Some(t) => t, None => return };
        let order = t.order();
        let (nv, np) = (t.ansicht_len() as i32, order.len() as i32);
        if nv == 0 || np == 0 { return; }
        let mut vi = t.view_of(t.cur_r).max(0);
        let mut p = t.pos_of(t.cur_c).max(0);
        if dp > 0 { p += 1; if p >= np { p = 0; vi += 1; } }
        if dp < 0 { p -= 1; if p < 0 { p = np - 1; vi -= 1; } }
        vi += dv;
        if vi < 0 { vi = 0; p = 0; }
        if vi >= nv {
            if anhaengen && t.zeilen_anhaengen {
                let r = t.zeile_anhaengen();
                t.rebuild_view();
                vi = t.view_of(r as i32);
                if vi < 0 { return; }
            } else {
                vi = nv - 1;
                if dp != 0 { p = np - 1; }
            }
        }
        let r = t.data_row(vi);
        t.cur_r = r;
        t.cur_c = order[p as usize] as i32;
        t.ber_r = t.cur_r;
        t.ber_c = t.cur_c;
        t.selected = r;
        t.sicht_holen = true;
    }

    /// Tastatur der Tabelle, wenn nichts bearbeitet wird. Ohne Zellmodus
    /// bewegen Pfeile, Bild und Pos1/Ende die gewaehlte Zeile -- das fehlte
    /// bis Stufe 30 ganz: eine Tabelle mit Fokus nahm keine Taste an.
    /// Tippen springt (Zeilenmodus): Buchstaben waehlen die naechste Zeile,
    /// deren Text in der SORTIERspalte so anfaengt -- ohne Sortierung in der
    /// ersten sichtbaren Spalte. Dieselbe Regel wie bei der Liste: was
    /// innerhalb einer Sekunde getippt wird, ist ein Wort, dieselbe Taste
    /// mehrmals laeuft durch alle mit diesem Anfang. TRUE = eine Taste kam an.
    pub(super) fn tabelle_tippen(&mut self, wi: usize, i: usize, gm: &TGeom, g: &mut Graphics) -> bool {
        let s = super::bausteine::getippt(g);
        if s.is_empty() || gm.order.is_empty() { return false; }
        let jetzt = g.uhr_jetzt();
        let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
        let (such, wiederholt) = super::bausteine::tipp_merken(&mut t.tipp, &mut t.tipp_zeit, jetzt, &s);
        let spalte = if t.sort_col >= 0 { t.sort_col as usize } else { gm.order[0] };
        let n = t.ansicht_len();
        if n == 0 { return true; }
        let pos = t.view_of(t.selected);
        let start = super::bausteine::tipp_start(if pos >= 0 { Some(pos as usize) } else { None }, wiederholt);
        // Virtuell sucht es nur in den gelieferten Zeilen (die anderen kennt
        // die Tabelle nicht).
        let treffer = (0..n).map(|d| (start + d) % n).take(if t.virt.is_some() { 5000 } else { n }).find(|&v| {
            let Some(r) = t.ansicht(v) else { return false };
            t.cell(r, spalte).map(|c| c.text.to_lowercase().starts_with(&such)).unwrap_or(false)
        });
        if let Some(v) = treffer {
            let r = t.ansicht(v).unwrap_or(0) as i32;
            if r != t.selected {
                t.auswahl_setzen(r);
                t.cur_r = r;
                self.tabelle_zeigen(wi, i);
                let f = self.windows[wi].widgets[i].on_change.clone();
                if let Some(f) = f { self.pending.push(f); }
            }
        }
        true
    }

    pub(super) fn table_nav_keys(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        if self.kuerzel_gefeuert { return; }
        let gm = self.table_geom(wi, i);
        let seite = (gm.body_h / gm.row_h.max(1)).max(1);
        let (ctrl, shift) = (g.key_ctrl(), g.key_shift());
        let (nv, np) = (gm.n_rows as i32, gm.n_cols as i32);
        let zm = self.windows[wi].widgets[i].tbl.as_ref().map(|t| t.zellmodus).unwrap_or(false);

        if !zm {
            if nv == 0 { return; }
            // Enter "oeffnet" die gewaehlte Zeile -- der Doppelklick der
            // Tastatur, wie bei der Liste.
            let gewaehlt = self.windows[wi].widgets[i].tbl.as_ref().map(|t| t.view_of(t.selected) >= 0).unwrap_or(false);
            if g.key_pressed(KEY_ENTER) && gewaehlt {
                self.windows[wi].widgets[i].tbl.as_mut().unwrap().doppel = true;
                return;
            }
            // F2 bearbeitet die erste freigegebene Textspalte der gewaehlten
            // Zeile (in Anzeige-Reihenfolge) -- ohne Doppelklick, ohne Maus.
            if g.key_pressed(KEY_F2) && gewaehlt {
                let (r, spalte) = {
                    let t = self.windows[wi].widgets[i].tbl.as_ref().unwrap();
                    let r = t.selected;
                    let c = gm.order.iter().copied().find(|&c| t.col_editable(c)
                        && t.col_typ_of(c) != 3
                        && t.cell(r as usize, c).map(|x| x.kind()).unwrap_or(CellKind::Text) == CellKind::Text);
                    (r, c)
                };
                if let Some(c) = spalte { self.table_begin_edit(wi, i, r, c as i32); self.tabelle_zeigen(wi, i); }
                return;
            }
            if self.tabelle_tippen(wi, i, &gm, g) { return; }
            let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
            let vi = t.view_of(t.selected);
            let neu = if g.key_repeat(KEY_DOWN) { vi + 1 }
                else if g.key_repeat(KEY_UP) { if vi < 0 { 0 } else { vi - 1 } }
                else if g.key_repeat(KEY_PAGEDOWN) { vi + seite }
                else if g.key_repeat(KEY_PAGEUP) { (vi - seite).max(0) }
                else if g.key_pressed(KEY_HOME) { 0 }
                else if g.key_pressed(KEY_END) { nv - 1 }
                else if t.baum && t.selected >= 0 && (g.key_repeat(KEY_RIGHT) || g.key_repeat(KEY_LEFT)) {
                    // Rechts klappt auf bzw. geht zum ersten Kind, links klappt
                    // zu bzw. geht zu den Eltern -- wie in jedem Dateibaum.
                    let r = t.selected as usize;
                    let rechts = g.key_repeat(KEY_RIGHT);
                    let kinder = t.hat_kinder.get(r).copied().unwrap_or(false);
                    let auf = t.offen.get(r).copied().unwrap_or(false);
                    if kinder && rechts != auf {
                        t.offen[r] = rechts;
                        t.rebuild_view();
                        return;
                    }
                    if rechts && kinder { vi + 1 }
                    else if !rechts && t.eltern.get(r).copied().unwrap_or(-1) >= 0 { t.view_of(t.eltern[r]) }
                    else { return }
                }
                else { return };
            let neu = neu.clamp(0, nv - 1);
            if neu == vi { return; }
            let r = t.data_row(neu);
            t.auswahl_setzen(r);
            t.cur_r = r;
            self.tabelle_zeigen(wi, i);
            let f = self.windows[wi].widgets[i].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
            return;
        }
        if np == 0 { return; }

        // --- Bewegen -------------------------------------------------------
        let mut ziel: Option<(i32, i32, bool)> = None;   // (Ansichtszeile, Position, Bereich erweitern)
        {
            let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
            let hatte = t.view_of(t.cur_r) >= 0 && t.pos_of(t.cur_c) >= 0;
            let vc = t.view_of(t.cur_r).max(0);
            let pc = t.pos_of(t.cur_c).max(0);
            if nv > 0 {
                if ctrl && g.key_pressed(K_A) {
                    t.ber_r = t.data_row(0);
                    t.ber_c = gm.order[0] as i32;
                    ziel = Some((nv - 1, np - 1, true));
                }
                else if g.key_repeat(KEY_DOWN) { ziel = Some((if hatte { vc + 1 } else { 0 }, pc, shift)); }
                else if g.key_repeat(KEY_UP) { ziel = Some((vc - 1, pc, shift)); }
                else if g.key_repeat(KEY_RIGHT) { ziel = Some((vc, if hatte { pc + 1 } else { 0 }, shift)); }
                else if g.key_repeat(KEY_LEFT) { ziel = Some((vc, pc - 1, shift)); }
                else if g.key_repeat(KEY_PAGEDOWN) { ziel = Some((vc + seite, pc, shift)); }
                else if g.key_repeat(KEY_PAGEUP) { ziel = Some((vc - seite, pc, shift)); }
                else if g.key_pressed(KEY_HOME) { ziel = Some((if ctrl { 0 } else { vc }, 0, shift)); }
                else if g.key_pressed(KEY_END) { ziel = Some((if ctrl { nv - 1 } else { vc }, np - 1, shift)); }
            }
        }
        if nv > 0 && !ctrl && g.key_repeat(KEY_TAB) {
            let hatte = self.windows[wi].widgets[i].tbl.as_ref().map(|t| t.view_of(t.cur_r) >= 0).unwrap_or(false);
            if hatte { self.table_schritt(wi, i, 0, if shift { -1 } else { 1 }, false); }
            else { ziel = Some((0, 0, false)); }
            if hatte { self.tabelle_zeigen(wi, i); return; }
        }
        if let Some((v, p, erweitern)) = ziel {
            let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
            let (v, p) = (v.clamp(0, nv - 1), p.clamp(0, np - 1));
            t.cur_r = t.data_row(v);
            t.cur_c = gm.order[p as usize] as i32;
            if !erweitern || t.ber_r < 0 { t.ber_r = t.cur_r; t.ber_c = t.cur_c; }
            t.selected = t.cur_r;
            self.tabelle_zeigen(wi, i);
            return;
        }

        // --- Zwischenablage -------------------------------------------------
        if ctrl && (g.key_pressed(K_C) || g.key_pressed(K_X)) {
            let text = self.windows[wi].widgets[i].tbl.as_ref().map(|t| t.bereich_text()).unwrap_or_default();
            if !text.is_empty() { g.clipboard_set(&text); }
            if g.key_pressed(K_X) {
                let n = self.windows[wi].widgets[i].tbl.as_mut().map(|t| t.bereich_leeren()).unwrap_or(0);
                if n > 0 { let f = self.windows[wi].widgets[i].on_change.clone(); if let Some(f) = f { self.pending.push(f); } }
            }
            return;
        }
        if ctrl && g.key_pressed(K_V) {
            let text = g.clipboard_get();
            let n = {
                let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
                if t.view_of(t.cur_r) < 0 {
                    t.cur_r = t.data_row(0);
                    t.cur_c = gm.order[0] as i32;
                }
                t.einfuegen_text(&text)
            };
            if n > 0 { let f = self.windows[wi].widgets[i].on_change.clone(); if let Some(f) = f { self.pending.push(f); } }
            return;
        }
        if nv == 0 { return; }

        // --- Zelle bearbeiten, leeren, Haken kippen ---------------------------
        let (r, c, art, darf, typ) = {
            let t = self.windows[wi].widgets[i].tbl.as_ref().unwrap();
            if t.view_of(t.cur_r) < 0 || t.cur_c < 0 { return; }
            let (r, c) = (t.cur_r as usize, t.cur_c as usize);
            (r, c, t.cell(r, c).map(|x| x.kind()).unwrap_or(CellKind::Text), t.col_editable(c), t.col_typ_of(c))
        };
        if g.key_pressed(KEY_ENTER) || g.key_pressed(KEY_F2) {
            if darf && art == CellKind::Text { self.table_begin_edit(wi, i, r as i32, c as i32); }
            else if g.key_pressed(KEY_ENTER) { self.table_schritt(wi, i, 1, 0, false); self.tabelle_zeigen(wi, i); }
            return;
        }
        if g.key_pressed(KEY_DELETE) || g.key_pressed(KEY_BACKSPACE) {
            let n = self.windows[wi].widgets[i].tbl.as_mut().map(|t| t.bereich_leeren()).unwrap_or(0);
            if n > 0 { let f = self.windows[wi].widgets[i].on_change.clone(); if let Some(f) = f { self.pending.push(f); } }
            return;
        }
        if art == CellKind::Check && g.key_pressed(KEY_SPACE) {
            let _ = g.pop_text_input();
            {
                let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
                if let Some(x) = t.cell_mut(r, c) { x.set_value(if x.value() >= 0.5 { 0.0 } else { 1.0 }); }
            }
            let f = self.windows[wi].widgets[i].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
            return;
        }
        // Tippen beginnt die Bearbeitung und ERSETZT den Inhalt -- wie in jeder
        // Tabellenkalkulation. Die Wiedergabe einer Aufnahme fuellt nur Tasten,
        // darum ersatzweise Buchstaben und Ziffern von dort.
        let mut s = g.pop_text_input();
        s.retain(|ch| !ch.is_control());
        if s.is_empty() && !ctrl {
            for k in (b'a'..=b'z').chain(b'0'..=b'9').chain([b'-', b',', b'.']) {
                if g.key_pressed(k as i64) { s.push(k as char); }
            }
        }
        if s.is_empty() || ctrl || !darf || art != CellKind::Text { return; }
        if typ == 3 {
            self.table_begin_edit(wi, i, r as i32, c as i32);
            self.table_wahl_springen(wi, i, &s);
            return;
        }
        if (typ == 1 || typ == 2) && !zahl_erlaubt(&s, typ) { return; }
        self.table_begin_edit(wi, i, r as i32, c as i32);
        if let Some(t) = self.windows[wi].widgets[i].tbl.as_mut() {
            if t.edit_row >= 0 {
                t.edit_text = s;
                t.edit_caret = t.edit_text.chars().count() as i32;
                t.edit_anchor = t.edit_caret;
                t.edit_maus_sperre = false;
            }
        }
    }

    /// In der offenen Auswahlliste zum naechsten Eintrag mit diesem Anfang.
    pub(super) fn table_wahl_springen(&mut self, wi: usize, i: usize, s: &str) {
        let t = match self.windows[wi].widgets[i].tbl.as_mut() { Some(t) => t, None => return };
        if t.edit_row < 0 { return; }
        let w = t.col_wahl.get(t.edit_col as usize).cloned().unwrap_or_default();
        let n = w.len();
        let ch = match s.chars().next() { Some(c) => c.to_lowercase().to_string(), None => return };
        if n == 0 { return; }
        let ab = (t.wahl_pos.max(0) as usize + 1) % n;
        if let Some(k) = (0..n).map(|d| (ab + d) % n).find(|&k| w[k].to_lowercase().starts_with(&ch)) {
            t.wahl_pos = k as i32;
            t.edit_text = w[k].clone();
        }
    }

    /// Tasten der offenen Auswahlliste.
    pub(super) fn table_wahl_keys(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let (ctrl, shift) = (g.key_ctrl(), g.key_shift());
        let (zm, w) = {
            let t = match self.windows[wi].widgets[i].tbl.as_ref() { Some(t) => t, None => return };
            (t.zellmodus, t.col_wahl.get(t.edit_col.max(0) as usize).cloned().unwrap_or_default())
        };
        let n = w.len() as i32;
        if n == 0 || g.key_pressed(KEY_ESC) { self.table_end_edit(wi, i, false); return; }
        {
            let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
            let mut pos = t.wahl_pos.clamp(0, n - 1);
            if g.key_repeat(KEY_DOWN) { pos = (pos + 1).min(n - 1); }
            if g.key_repeat(KEY_UP) { pos = (pos - 1).max(0); }
            if g.key_pressed(KEY_HOME) { pos = 0; }
            if g.key_pressed(KEY_END) { pos = n - 1; }
            t.wahl_pos = pos;
            t.edit_text = w[pos as usize].clone();
        }
        let mut s = g.pop_text_input();
        s.retain(|ch| !ch.is_control() && ch != ' ');
        if s.is_empty() && !ctrl {
            for k in (b'a'..=b'z').chain(b'0'..=b'9') {
                if g.key_pressed(k as i64) { s.push(k as char); }
            }
        }
        if !s.is_empty() { self.table_wahl_springen(wi, i, &s); }
        let enter = g.key_pressed(KEY_ENTER);
        let tab = zm && !ctrl && g.key_pressed(KEY_TAB);
        if enter || tab {
            self.table_end_edit(wi, i, true);
            if zm {
                if enter { self.table_schritt(wi, i, 1, 0, true); }
                else { self.table_schritt(wi, i, 0, if shift { -1 } else { 1 }, false); }
            }
        }
    }

    /// Rechteck der offenen Auswahlliste unter der Zelle: (x, y, b, h, erster
    /// gezeigter Eintrag, gezeigte Zeilen, Zeilenhoehe). EINE Quelle fuer
    /// Zeichnen und Klick.
    pub(super) fn wahl_geom(&self, wi: usize, i: usize) -> Option<(i32, i32, i32, i32, usize, usize, i32)> {
        let t = self.windows.get(wi)?.widgets.get(i)?.tbl.as_ref()?;
        if t.edit_row < 0 || t.col_typ_of(t.edit_col.max(0) as usize) != 3 { return None; }
        let w = t.col_wahl.get(t.edit_col as usize)?;
        if w.is_empty() { return None; }
        let (x, y, cw, ch) = self.edit_cell_rect(wi, i)?;
        let zh = ch.max(self.sk(18));
        let zeigen = w.len().min(8);
        let erster = (t.wahl_pos.max(0) as usize + 1).saturating_sub(zeigen).min(w.len() - zeigen);
        Some((x, y + ch, cw.max(self.sk(90)), zh * zeigen as i32 + 2, erster, zeigen, zh))
    }

    pub(super) fn draw_wahl_popup(&self, g: &mut Graphics, wi: usize, i: usize) {
        let (px, py, pw, ph, erster, zeigen, zh) = match self.wahl_geom(wi, i) { Some(x) => x, None => return };
        let wdg = &self.windows[wi].widgets[i];
        let t = wdg.tbl.as_ref().unwrap();
        let w = &t.col_wahl[t.edit_col as usize];
        let acc = self.th("accent");
        g.box_fill(px + 3, py + 3, px + pw + 2, py + ph + 2, 0x40i64 << 24);
        g.box_fill(px, py, px + pw - 1, py + ph - 1, self.th("widget_bg"));
        g.rect(px, py, px + pw - 1, py + ph - 1, self.th("widget_border"));
        let schrift = self.wsize(g, wdg);
        for k in 0..zeigen {
            let idx = erster + k;
            let y = py + 1 + k as i32 * zh;
            if idx as i32 == t.wahl_pos {
                g.box_fill(px + 1, y, px + pw - 2, y + zh - 1, (150i64 << 24) | (shade(acc, -110) & 0xFFFFFF));
            }
            self.wtext(g, wdg, px + self.sk(TBL_PADDING), y + (zh - schrift).max(0) / 2,
                       w[idx].clone(), self.th("text_fg"));
        }
    }

    /// Tippen in die Zelle, die gerade bearbeitet wird.
    ///
    /// Schreibmarke, Pfeile, Pos1/Ende, Entfernen, Einfuegen aus der
    /// Zwischenablage. **Keine Textauswahl** -- die volle Textfeld-Logik hier
    /// zu wiederholen waere ein zweiter Ort, an dem dieselben Fehler entstehen
    /// koennen; fuer eine Tabellenzelle reicht das hier.
    /// Bildschirm-Rechteck der Zelle, die gerade bearbeitet wird.
    ///
    /// Aus `table_geom` gerechnet statt beim Zeichnen gemerkt: das Zeichnen
    /// laeuft ueber `&self` und koennte gar nichts merken -- und eine zweite,
    /// nachgefuehrte Quelle waere genau die Sorte Doppelung, die zwischen
    /// Treffer und Anzeige auseinander laeuft.
    pub(super) fn edit_cell_rect(&self, wi: usize, idx: usize) -> Option<(i32, i32, i32, i32)> {
        let gm = self.table_geom(wi, idx);
        let t = self.windows[wi].widgets[idx].tbl.as_ref()?;
        if t.edit_row < 0 { return None; }
        let pos = gm.order.iter().position(|&c| c as i32 == t.edit_col)?;
        let vi = t.view_of(t.edit_row);
        if vi < 0 { return None; }
        let x = Self::col_x(&gm, pos);
        let y = gm.body_y + vi * gm.row_h - t.scroll_y;
        Some((x, y, gm.col_widths[pos], gm.row_h))
    }

    pub(super) fn table_edit_keys(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let ms = self.mass(g, &self.windows[wi].widgets[i]);
        let ctrl = g.key_ctrl();
        let shift = g.key_shift();
        // Enter/ESC zuerst -- danach ist die Bearbeitung vorbei und alles
        // Weitere waere Arbeit an einem Zustand, den es nicht mehr gibt.
        // Im Zellmodus rueckt Enter eine Zeile tiefer und Tab eine Zelle
        // weiter, wie in jeder Tabellenkalkulation.
        let (zm, typ) = self.windows[wi].widgets[i].tbl.as_ref()
            .map(|t| (t.zellmodus, t.col_typ_of(t.edit_col.max(0) as usize))).unwrap_or((false, 0));
        if g.key_pressed(KEY_ENTER) {
            self.table_end_edit(wi, i, true);
            if zm { self.table_schritt(wi, i, 1, 0, true); }
            return;
        }
        if g.key_pressed(27) { self.table_end_edit(wi, i, false); return; }
        if zm && !ctrl && g.key_pressed(KEY_TAB) {
            self.table_end_edit(wi, i, true);
            self.table_schritt(wi, i, 0, if shift { -1 } else { 1 }, false);
            return;
        }

        let rect = self.edit_cell_rect(wi, i);
        let (rx, rw) = rect.map(|(x, _, w, _)| (x, w)).unwrap_or((0, 0));

        let t = match self.windows[wi].widgets[i].tbl.as_mut() { Some(t) => t, None => return };
        let mut chars: Vec<char> = t.edit_text.chars().collect();
        let mut caret = t.edit_caret.clamp(0, chars.len() as i32);
        let mut anchor = t.edit_anchor.clamp(0, chars.len() as i32);
        let scroll = t.edit_scroll;

        // Der oeffnende Doppelklick zaehlt nicht als Setzen der Marke -- erst
        // wenn die Taste einmal losgelassen wurde, hoert der Editor auf die
        // Maus.
        if t.edit_maus_sperre {
            if !g.mouse_button(0) { t.edit_maus_sperre = false; }
        }
        let maus_an = !t.edit_maus_sperre;

        // Maus im Eingabefeld: Klick setzt die Marke, Ziehen markiert. Das
        // Feld liegt IN der Tabelle, deshalb hier und nicht im allgemeinen
        // Tabellen-Treffertest -- der wuerde den Klick als Zeilenauswahl
        // deuten und die Bearbeitung beenden.
        if maus_an && g.mouse_button(0) {
            let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
            let ziel = mx - (rx + 5) + scroll;
            let idx = Self::caret_index_at(g, &chars, ziel, ms);
            if !self.was_mouse_down {
                let drin = rect.map(|r| Self::in_rect(mx, my, r)).unwrap_or(false);
                if drin { caret = idx; anchor = idx; }
            } else {
                caret = idx;
            }
        }

        let (alt, alt_caret, alt_anchor) = (chars.clone(), caret, anchor);
        Self::einzeiler_tasten(g, &mut chars, &mut caret, &mut anchor, ctrl, shift,
                               self.kuerzel_gefeuert);
        // Zahlenspalten nehmen nur, was eine Zahl werden kann -- dieselbe
        // Regel wie im Textfeld mit `zahlen`.
        if (typ == 1 || typ == 2) && !zahl_erlaubt(&chars.iter().collect::<String>(), typ) {
            chars = alt; caret = alt_caret; anchor = alt_anchor;
        }

        // Versatz nachfuehren, damit die Schreibmarke im Feld bleibt.
        let vor: String = chars[..caret.clamp(0, chars.len() as i32) as usize].iter().collect();
        let marke_px = ms.breite(g, &vor);
        let innen = (rw - 10).max(1);
        let mut scroll = scroll;
        if marke_px - scroll > innen { scroll = marke_px - innen; }
        if marke_px - scroll < 0 { scroll = marke_px; }
        if scroll < 0 { scroll = 0; }

        let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
        t.edit_text = chars.iter().collect();
        t.edit_caret = caret.clamp(0, chars.len() as i32);
        t.edit_anchor = anchor.clamp(0, chars.len() as i32);
        t.edit_scroll = scroll;
    }

    /// Tippen in die fokussierte Filterzelle. Bewusst schlicht: Zeichen
    /// anhaengen, Rueckschritt loeschen, Escape/Enter beendet die Eingabe.
    /// Kein Cursor, keine Auswahl -- ein Filter ist ein Suchfeld, kein Editor,
    /// und die volle Textfeld-Logik hier zu wiederholen waere ein zweiter Ort,
    /// an dem dieselben Fehler entstehen koennen.
    pub(super) fn table_filter_keys(&mut self, wi: usize, idx: usize, g: &mut Graphics) {
        let c = {
            let t = match self.windows[wi].widgets[idx].tbl.as_ref() { Some(t) => t, None => return };
            if !t.filter_row { return; }
            t.filter_focus
        };
        if c < 0 { return; }
        let mut geaendert = false;
        let eingabe: Vec<char> = g.pop_text_input().chars().collect();
        let rueck = g.key_repeat(KEY_BACKSPACE);
        let raus = g.key_pressed(27) || g.key_pressed(KEY_ENTER);   // 27 = ESC
        {
            let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
            while t.filters.len() <= c as usize { t.filters.push(String::new()); }
            let f = &mut t.filters[c as usize];
            for ch in eingabe {
                if !ch.is_control() { f.push(ch); geaendert = true; }
            }
            if rueck && !f.is_empty() { f.pop(); geaendert = true; }
            if raus { t.filter_focus = -1; }
        }
        if geaendert {
            let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
            t.rebuild_view();
            // Nach dem Filtern kann die alte Scroll-Position weit hinter dem
            // Ende liegen -- dann saehe man eine leere Tabelle.
            t.scroll_y = 0;
            let f = self.windows[wi].widgets[idx].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    pub(super) fn table_drag(&mut self, wi: usize, idx: usize, mx: i32, my: i32) {
        let gm = self.table_geom(wi, idx);
        // Beide Laengen vor dem mut-Zugriff auf die Tabelle.
        let (zug_min, spalte_min) = (self.sk(HEAD_DRAG_MIN), self.sk(TBL_MIN_COL_W));
        let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap();
        if let Some((pos, x0, bewegt)) = t.head_drag {
            // Erst ab einer Mindeststrecke gilt es als Zug -- ein Klick
            // wackelt fast immer um ein, zwei Punkte.
            if !bewegt && (mx - x0).abs() < zug_min { return; }
            if !t.reorderable { return; }
            let mut pos = pos;
            t.head_drag = Some((pos, x0, true));
            // Ueber die Mitte des Nachbarn hinaus -> tauschen. Kein Warten auf
            // das Loslassen: man soll SEHEN, wo die Spalte landet.
            let mitte = |p: usize| Self::col_x(&gm, p) + gm.col_widths[p] / 2;
            // Davor heisst links, gespiegelt rechts.
            let vor = |m: i32| if gm.rtl { mx > m } else { mx < m };
            let mitte_links = if pos > 0 { Some(mitte(pos - 1)) } else { None };
            let mitte_rechts = if pos + 1 < gm.n_cols { Some(mitte(pos + 1)) } else { None };
            if mitte_links.is_some_and(vor) {
                t.move_col(pos, pos - 1);
                pos -= 1;
                t.head_drag = Some((pos, x0, true));
            } else if mitte_rechts.is_some_and(|m| !vor(m) && m != mx) {
                t.move_col(pos, pos + 1);
                pos += 1;
                t.head_drag = Some((pos, x0, true));
            }
            return;
        }
        if t.drag_col >= 0 {
            let c = t.drag_col as usize;
            let zug = if gm.rtl { t.drag_col_x - mx } else { mx - t.drag_col_x };
            let neu = (t.drag_col_w + zug).max(spalte_min);
            if let Some(cw) = t.col_widths.as_mut() {
                if c < cw.len() { cw[c] = neu; }
            }
            return;
        }
        if t.drag_v && gm.max_scroll_y > 0 {
            let track = gm.body_h;
            let handle = Self::handle_height(track, gm.body_h, gm.total_h);
            // Review-Fund: `handle` hat eine feste Untergrenze (handle_height
            // clamped auf min. 16px), `track` (die Widget-Hoehe abzueglich
            // Kopfzeile) nicht -- ein sehr kleines/geschrumpftes Table-Widget
            // (z.B. per GUI_SET_BOUNDS) liess `track - handle` negativ werden,
            // und `i32::clamp(0, negativ)` paniked hart ("assertion failed:
            // min <= max"). `.max(0)` haelt die Invariante unabhaengig von
            // der Widget-Groesse.
            let hi = (track - handle).max(0);
            let new_y = ((my - gm.body_y) - t.drag_off).clamp(0, hi);
            let pos = new_y as f64 / hi.max(1) as f64;
            t.scroll_y = (pos * gm.max_scroll_y as f64) as i32;
        } else if t.drag_h && gm.max_scroll_x > 0 {
            let track = gm.body_w;
            let handle = Self::handle_height(track, gm.body_w, gm.total_w);
            let hi = (track - handle).max(0);
            let new_x = ((mx - gm.body_x) - t.drag_off).clamp(0, hi);
            let pos = new_x as f64 / hi.max(1) as f64;
            t.scroll_x = (pos * gm.max_scroll_x as f64) as i32;
        }
    }
}

/// `fg` -> true (Schrift), `bg` -> false (Flaeche) -- wie bei GUI_GET_COLOR.
fn farb_rolle(rolle: &str, f: &str) -> Result<bool, String> {
    match rolle.to_ascii_lowercase().as_str() {
        "fg" => Ok(true),
        "bg" => Ok(false),
        _ => Err(format!("{}: '{}' unbekannt -- fg (Schrift) oder bg (Flaeche)", f, rolle)),
    }
}
