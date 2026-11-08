//! Textbereich-Befehle fuer Editoren (Weg C): Auswahl, Marken, Faltung, Suche, Bearbeiten.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Textbereich-Befehle fuer Editoren (Weg C) --------------------------
    // Zeilen und Spalten zaehlen ab 1, in ZEICHEN (wie GUI_TEXTAREA_VIEW und
    // die Fehlermeldungen von --check). Alle Rechnungen gehen ueber den
    // Zeichen-Index `caret`, denselben, den das Editieren fuehrt.

    pub(super) fn ta_wdg(&self, h: i64, fn_: &str) -> Result<&Widget, String> {
        let wd = self.wdg(h, fn_)?;
        if wd.kind != Kind::TextArea { return Err(format!("{}: das Widget ist kein GUI_TEXTAREA", fn_)); }
        Ok(wd)
    }

    /// Zeichen-Index -> (Zeile, Spalte), beide ab 1.
    pub(super) fn ta_zeile_spalte(chars: &[char], idx: usize) -> (i64, i64) {
        let idx = idx.min(chars.len());
        let starts = Self::line_starts(chars);
        let z = starts.iter().rposition(|&s| s <= idx).unwrap_or(0);
        (z as i64 + 1, (idx - starts[z]) as i64 + 1)
    }

    /// (Zeile, Spalte) ab 1 -> Zeichen-Index, an die Grenzen geklemmt.
    pub(super) fn ta_index(chars: &[char], zeile: i64, spalte: i64) -> usize {
        let starts = Self::line_starts(chars);
        let z = (zeile.max(1) as usize - 1).min(starts.len() - 1);
        let ende = if z + 1 < starts.len() { starts[z + 1] - 1 } else { chars.len() };
        (starts[z] + (spalte.max(1) as usize - 1)).min(ende)
    }

    /// Zeichen und Zeilenanfaenge des Texts -- aus `ta_zerlegung`, wenn der
    /// Text sich seit der letzten Frage nicht geaendert hat.
    pub(super) fn ta_zerlegt(&self, h: i64, text: &str) -> std::rc::Rc<(Vec<char>, Vec<usize>)> {
        let mut c = self.ta_zerlegung.borrow_mut();
        if let Some((t, z)) = c.get(&h) {
            if t == text { return z.clone(); }
        }
        let chars: Vec<char> = text.chars().collect();
        let starts = Self::line_starts(&chars);
        let z = std::rc::Rc::new((chars, starts));
        c.insert(h, (text.to_string(), z.clone()));
        z
    }

    /// GUI_TEXTAREA_CURSOR: Zeile und Spalte der Schreibmarke.
    pub fn textarea_cursor(&self, h: i64) -> Result<(i64, i64), String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_CURSOR")?;
        let z = self.ta_zerlegt(h, &wd.text);
        let (chars, starts) = (&z.0, &z.1);
        let idx = (wd.caret.max(0) as usize).min(chars.len());
        // Binaer statt `rposition`: die Zeilenanfaenge sind aufsteigend.
        let zl = starts.partition_point(|&s| s <= idx).saturating_sub(1);
        Ok((zl as i64 + 1, (idx - starts[zl]) as i64 + 1))
    }

    /// GUI_TEXTAREA_POS_AT: welches Zeichen liegt unter dem Bildschirmpunkt
    /// (x, y)? (zeile, spalte) ab 1 -- (0, 0) neben dem Text: ausserhalb des
    /// Feldes, in der Nummernspalte, unter der letzten Zeile oder rechts
    /// hinter dem Zeilenende. Das Letzte ist Absicht: wer die Maus ueber den
    /// leeren Rest einer Zeile haelt, zeigt auf nichts, und ein Tooltip zum
    /// letzten Wort dort waere eine Antwort auf eine Frage, die keiner
    /// gestellt hat. Dieselbe Rechnung wie der Klick (Zeilenhoehe, Scroll,
    /// Nummernspalte, waagerechter Versatz, Umbruch und Faltung ueber
    /// `ta_rows`) -- liefe sie auseinander, zeigte der Tooltip das Wort
    /// neben dem, auf das ein Klick die Marke setzt.
    pub fn textarea_pos_at(&self, g: &Graphics, h: i64, x: i64, y: i64) -> Result<(i64, i64), String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_POS_AT")?;
        let (wi, _) = Self::dec_widget(h);
        let (ax, ay, fw, fh) = self.abs_rect(wi, wd);
        let (mx, my) = (x as i32, y as i32);
        if !Self::in_rect(mx, my, (ax, ay, fw, fh)) { return Ok((0, 0)); }
        let pad = 5;
        if my < ay + pad { return Ok((0, 0)); }
        let lh = self.ta_line_h(g, wd);
        let z = self.ta_zerlegt(h, &wd.text);
        let (chars, starts) = (&z.0, &z.1);
        let rows = self.ta_rows(g, wd, chars, starts, self.ta_breite(g, wd, starts.len()));
        // Unter einem angehefteten Blockkopf ist der Text verdeckt.
        let sicht = ((fh - 2 * pad) / lh.max(1)).max(1);
        if my < ay + pad + Self::ta_koepfe(wd, &rows, sicht, wd.scroll).len() as i32 * lh { return Ok((0, 0)); }
        let row = wd.scroll + (my - ay - pad) / lh;
        if row < 0 || row as usize >= rows.len() { return Ok((0, 0)); }
        let (rli, lstart, lend) = rows[row as usize];
        let gut = self.ta_gutter(g, wd, starts.len());
        if mx < ax + pad + gut { return Ok((0, 0)); }
        let target = mx - (ax + pad + gut - wd.scroll_x);
        let ms = self.mass(g, wd);
        let sub = &chars[lstart..lend];
        if let Some(lage) = Self::rtl_lage_mit(g, sub, |s| ms.breite(g, s)) {
            return Ok(match lage.zeichen_bei(target) {
                Some(l) => (rli as i64 + 1, ((lstart + l) - starts[rli] + 1) as i64),
                None => (0, 0),
            });
        }
        // Das erste n, dessen Vorspann breiter ist als der Abstand -- binaer,
        // wie in caret_index_at (die Breite waechst mit n).
        let breite = |n: usize| -> i32 { let t: String = sub[..n].iter().collect(); ms.breite(g, &t) };
        let (mut lo, mut hi) = (1usize, sub.len() + 1);
        while lo < hi {
            let n = (lo + hi) / 2;
            if target < breite(n) { hi = n; } else { lo = n + 1; }
        }
        if lo <= sub.len() {
            let spalte = (lstart + lo - 1) - starts[rli] + 1;
            return Ok((rli as i64 + 1, spalte as i64));
        }
        Ok((0, 0))
    }

    /// GUI_TEXTAREA_SELECTION_RANGE: Anfang und Ende der Auswahl als
    /// (z1, s1, z2, s2), ab 1, geordnet -- ohne Auswahl steht die Marke an
    /// beiden Enden. Der Text allein (SELECTION$) reicht einem Editor nicht:
    /// wer ganze Zeilen einrueckt oder auskommentiert, muss wissen, WELCHE.
    pub fn textarea_selection_range(&self, h: i64) -> Result<(i64, i64, i64, i64), String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_SELECTION_RANGE")?;
        let chars: Vec<char> = wd.text.chars().collect();
        let n = chars.len() as i32;
        let (lo, hi) = (wd.caret.clamp(0, n).min(wd.sel_anchor.clamp(0, n)), wd.caret.clamp(0, n).max(wd.sel_anchor.clamp(0, n)));
        let (z1, s1) = Self::ta_zeile_spalte(&chars, lo as usize);
        let (z2, s2) = Self::ta_zeile_spalte(&chars, hi as usize);
        Ok((z1, s1, z2, s2))
    }

    /// GUI_TEXTAREA_SELECTION$: der markierte Text (leer ohne Auswahl).
    pub fn textarea_selection(&self, h: i64) -> Result<String, String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_SELECTION$")?;
        let chars: Vec<char> = wd.text.chars().collect();
        let n = chars.len() as i32;
        let (lo, hi) = (wd.caret.clamp(0, n).min(wd.sel_anchor.clamp(0, n)), wd.caret.clamp(0, n).max(wd.sel_anchor.clamp(0, n)));
        Ok(chars[lo as usize..hi as usize].iter().collect())
    }

    /// Den Ausschnitt so legen, dass die Marke zu sehen ist -- mittig, damit
    /// nach einem Sprung Umgebung da ist; die Editierschleife klemmt danach.
    pub(super) fn ta_marke_zeigen(&mut self, g: &Graphics, h: i64) -> Result<(), String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_GOTO")?;
        let chars: Vec<char> = wd.text.chars().collect();
        let starts = Self::line_starts(&chars);
        let lh = self.ta_line_h(g, wd);
        let sicht = ((wd.h - 2 * 5) / lh).max(1);
        let rows = self.ta_rows(g, wd, &chars, &starts, self.ta_breite(g, wd, starts.len()));
        let crow = Self::ta_row_of(&rows, wd.caret.max(0) as usize) as i32;
        let max_scroll = (rows.len() as i32 - sicht).max(0);
        let scroll = (crow - sicht / 2).clamp(0, max_scroll);
        let w = self.wdg_mut(h, "GUI_TEXTAREA_GOTO")?;
        w.scroll = scroll;
        w.scroll_x = 0;
        Ok(())
    }

    /// GUI_TEXTAREA_GOTO(ta, zeile[, spalte]): Marke setzen, Auswahl und
    /// weitere Marken aufheben, Ausschnitt nachziehen.
    pub fn textarea_goto(&mut self, g: &Graphics, h: i64, zeile: i64, spalte: i64) -> Result<(), String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_GOTO")?;
        let chars: Vec<char> = wd.text.chars().collect();
        let idx = Self::ta_index(&chars, zeile, spalte) as i32;
        let w = self.wdg_mut(h, "GUI_TEXTAREA_GOTO")?;
        w.caret = idx; w.sel_anchor = idx; w.marken_zusatz.clear();
        // Ein Sprung in einen zugeklappten Block klappt ihn auf -- sonst
        // spraenge die Suche an eine Stelle, die man nicht sieht.
        Self::falte_am_caret_oeffnen(w);
        self.ta_marke_zeigen(g, h)
    }

    /// GUI_TEXTAREA_SCROLL(ta, zeile): den Ausschnitt so rollen, dass die
    /// (logische) Zeile oben steht -- OHNE die Marke zu bewegen, wie das
    /// Mausrad. Geklemmt, sodass unten kein leerer Platz entsteht; eine Zeile
    /// in einem zugeklappten Block nimmt die Kopfzeile.
    pub fn textarea_scroll(&mut self, g: &Graphics, h: i64, zeile: i64) -> Result<(), String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_SCROLL")?;
        let chars: Vec<char> = wd.text.chars().collect();
        let starts = Self::line_starts(&chars);
        let lh = self.ta_line_h(g, wd).max(1);
        let sicht = ((wd.h - 2 * 5) / lh).max(1);
        let rows = self.ta_rows(g, wd, &chars, &starts, self.ta_breite(g, wd, starts.len()));
        let ziel = (zeile - 1).max(0) as usize;
        // Die letzte Reihe, die zu einer Zeile <= ziel gehoert -- so trifft
        // eine verborgene Zeile die sichtbare Kopfzeile ihres Blocks.
        let mut row = 0usize;
        for (k, r) in rows.iter().enumerate() {
            if r.0 <= ziel && (k == 0 || rows[k - 1].0 != r.0) { row = k; }
            if r.0 > ziel { break; }
        }
        let max = (rows.len() as i32 - sicht).max(0);
        let w = self.wdg_mut(h, "GUI_TEXTAREA_SCROLL")?;
        w.scroll = (row as i32).clamp(0, max);
        w.rad_ziel = None;
        w.rad_stand = Some((w.caret, w.sel_anchor, w.text.chars().count()));
        Ok(())
    }

    /// GUI_TEXTAREA_SELECT(ta, z1, s1, z2, s2): Bereich markieren, Marke am Ende.
    pub fn textarea_select(&mut self, g: &Graphics, h: i64, z1: i64, s1: i64, z2: i64, s2: i64) -> Result<(), String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_SELECT")?;
        let chars: Vec<char> = wd.text.chars().collect();
        let a = Self::ta_index(&chars, z1, s1) as i32;
        let b = Self::ta_index(&chars, z2, s2) as i32;
        let w = self.wdg_mut(h, "GUI_TEXTAREA_SELECT")?;
        w.sel_anchor = a; w.caret = b; w.marken_zusatz.clear();
        Self::falte_am_caret_oeffnen(w);
        self.ta_marke_zeigen(g, h)
    }

    /// GUI_TEXTAREA_INSERT(ta, text$): ersetzt die Auswahl bzw. fuegt an der
    /// Marke ein -- als eigener Undo-Schritt, mit on_change wie beim Tippen.
    pub fn textarea_insert(&mut self, h: i64, text: &str, jetzt: f64) -> Result<(), String> {
        self.ta_wdg(h, "GUI_TEXTAREA_INSERT")?;
        let w = self.wdg_mut(h, "GUI_TEXTAREA_INSERT")?;
        let before = w.text.clone();
        let mut chars: Vec<char> = before.chars().collect();
        let n = chars.len() as i32;
        let (lo, hi) = (w.caret.clamp(0, n).min(w.sel_anchor.clamp(0, n)), w.caret.clamp(0, n).max(w.sel_anchor.clamp(0, n)));
        let caret0 = w.caret;
        chars.drain(lo as usize..hi as usize);
        let neu: Vec<char> = text.chars().filter(|c| *c == '\n' || *c == '\t' || !c.is_control()).collect();
        for (k, ch) in neu.iter().enumerate() { chars.insert(lo as usize + k, *ch); }
        // Ein eingefuegter Block ist ein eigener Schritt, kein Teil des Tippens davor.
        w.undo_zeit = -10.0;
        Self::undo_merken(w, &before, caret0, jetzt);
        w.text = chars.iter().collect();
        w.caret = lo + neu.len() as i32; w.sel_anchor = w.caret;
        stile_abgleichen(w);
        let nachher = w.text.clone();
        Self::falten_nachziehen(w, &before, &nachher, caret0);
        Self::falte_am_caret_oeffnen(w);
        w.spans.clear();
        w.wellen.clear();
        if let Some(f) = w.on_change.clone() { self.pending.push(f); }
        Ok(())
    }

    /// GUI_TEXTAREA_FIND(ta, text$[, ab_zeile[, ab_spalte[, genau]]]) ->
    /// (Zeile, Spalte) des naechsten Treffers ab der Stelle, sonst (-1, -1).
    /// Ohne `genau` ohne Ruecksicht auf Gross/Klein. Kein Umlauf: wer am
    /// Ende nichts findet, sucht ab 1,1 noch einmal.
    pub fn textarea_find(&self, h: i64, gesucht: &str, ab_zeile: i64, ab_spalte: i64, genau: bool) -> Result<(i64, i64), String> {
        let wd = self.ta_wdg(h, "GUI_TEXTAREA_FIND")?;
        if gesucht.is_empty() { return Ok((-1, -1)); }
        let chars: Vec<char> = wd.text.chars().collect();
        let ab = Self::ta_index(&chars, ab_zeile, ab_spalte);
        let (heu, nadel): (Vec<char>, Vec<char>) = if genau {
            (chars.clone(), gesucht.chars().collect())
        } else {
            (chars.iter().flat_map(|c| c.to_lowercase()).collect(), gesucht.chars().flat_map(|c| c.to_lowercase()).collect())
        };
        if nadel.len() > heu.len() { return Ok((-1, -1)); }
        for i in ab..=heu.len() - nadel.len() {
            if heu[i..i + nadel.len()] == nadel[..] { return Ok(Self::ta_zeile_spalte(&chars, i)); }
        }
        Ok((-1, -1))
    }

    /// Mehrzeiliges Textfeld editieren: Tippen, Enter=Umbruch, Backspace/Delete,
    /// Pfeile (auch hoch/runter), Home/End, Strg+A/C/V/X, Maus-Klick + Ziehen.
    /// Selektion: Maus-Drag, Shift+Navigation, Strg+A. Vertikal scrollend, damit
    /// das Caret sichtbar bleibt.
    pub(super) fn edit_textarea(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let ms = self.mass(g, &self.windows[wi].widgets[i]);
        let before = self.windows[wi].widgets[i].text.clone();
        let mut chars: Vec<char> = before.chars().collect();
        let mut caret = self.windows[wi].widgets[i].caret.clamp(0, chars.len() as i32);
        let mut anchor = self.windows[wi].widgets[i].sel_anchor.clamp(0, chars.len() as i32);
        let caret0 = caret;
        let ctrl = g.key_ctrl();
        let shift = g.key_shift();
        let alt = g.key_alt();
        // Hat ein Menue-Kuerzel dieselbe Taste schon verbraucht? Dann bewegt
        // sie hier NICHTS mehr (siehe kuerzel_pruefen).
        let kuerzel_weg = self.kuerzel_gefeuert;
        // Weitere Schreibmarken: LEER heisst eine, und dann laeuft alles wie
        // zuvor. ESC raeumt sie weg -- ohne den Ausweg saehe man nur, dass
        // ploetzlich an mehreren Stellen etwas passiert.
        let mut zusatz: Vec<(i32, i32)> = self.windows[wi].widgets[i].marken_zusatz.clone();
        zusatz.retain(|&(c, a)| c >= 0 && (c as usize) <= chars.len() && a >= 0);
        if g.key_pressed(KEY_ESC) { zusatz.clear(); }
        if ctrl && !kuerzel_weg && (g.key_pressed(K_Z) || g.key_pressed(K_Y)) {
            let ziel = self.windows[wi].widgets[i].teil_von.unwrap_or(i);
            self.text_undo(wi, ziel, g.key_pressed(K_Y) || shift);
            self.teil_pass(g.get_time());
            return;
        }
        // Inline-Formate: Strg+B fett, Strg+I kursiv, Strg+U unterstrichen --
        // auf die Auswahl, ohne Auswahl fuer das, was als Naechstes kommt.
        // Hat ein Menue-Kuerzel die Taste schon genommen, bleibt es bei ihm.
        if self.windows[wi].widgets[i].formatiert && ctrl && !alt && !kuerzel_weg && !shift {
            use crate::schnitt::{FETT, KURSIV, UNTER};
            let bit = if g.key_pressed(K_B) { FETT } else if g.key_pressed(K_I) { KURSIV }
                      else if g.key_pressed(K_U) { UNTER } else { 0 };
            if bit != 0 {
                let jetzt = g.get_time();
                self.stil_anwenden(wi, i, bit, None, jetzt);
                return;
            }
        }
        {
            let w = &mut self.windows[wi].widgets[i];
            if w.formatiert { stile_abgleichen(w); }
        }
        let pad = 5;
        let lh = self.ta_line_h(g, &self.windows[wi].widgets[i]);
        let (ax, ay, fw, fh) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let scroll = self.windows[wi].widgets[i].scroll;

        // Maus: Klick (steigende Flanke) setzt Caret+Anker, Ziehen erweitert die
        // Selektion bis zur aktuellen Position.
        if !g.mouse_button(0) {
            self.windows[wi].widgets[i].farbfeld_zug = false;
            self.windows[wi].widgets[i].wort_zug = false;
            self.windows[wi].widgets[i].spalten_start = (-1, -1);
        }
        // Ein Druck, der nicht IN diesem Feld beginnt -- daneben oder auf
        // einem Fenster, das darueber liegt --, gehoert ihm nicht: das Feld
        // hat den Fokus (eine Vorschlagsliste laesst ihn bewusst im Code),
        // und ohne diese Sperre setzte ein Klick auf die Liste ueber dem
        // Code auch die Marke darunter, und ein Zug an ihrer Titelleiste
        // markierte Text. Der Zug wird wie beim Farbfeld ganz gesperrt.
        if g.mouse_button(0) && !self.was_mouse_down {
            let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
            if !Self::in_rect(mx, my, (ax, ay, fw, fh)) || self.topmost_at(mx, my) != Some(wi) {
                self.windows[wi].widgets[i].farbfeld_zug = true;
            } else {
                self.zug_feld = Some((wi, i));
            }
        } else if g.mouse_button(0) && self.zug_feld != Some((wi, i)) {
            // Die Taste war schon unten, als das Feld den Fokus bekam.
            self.windows[wi].widgets[i].farbfeld_zug = true;
        }
        if g.mouse_button(0) && !self.windows[wi].widgets[i].farbfeld_zug {
            let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
            let row = (scroll + ((my - ay - pad).max(0) / lh)).max(0);
            let starts = Self::line_starts(&chars);
            let wref = &self.windows[wi].widgets[i];
            let rows = self.ta_rows(g, wref, &chars, &starts, self.ta_breite(g, wref, starts.len()));
            let r = (row as usize).min(rows.len().saturating_sub(1));
            let (rli, lstart, lend) = rows[r];
            // Klick auf ein Farbfeld meldet sich und laesst die Marke stehen
            // -- man will die Farbe aendern, nicht dort schreiben.
            if !self.was_mouse_down && Self::in_rect(mx, my, (ax, ay, fw, fh))
               && !self.windows[wi].widgets[i].farbfelder.is_empty() {
                let sicht = ((fh - 2 * pad) / lh).max(1);
                let treffer = self.ta_farbfeld_rects(g, wref, ax, ay, &chars, &rows,
                                                     starts.len(), sicht)
                    .into_iter()
                    .find(|&(_, fx, fy, kante)| Self::in_rect(mx, my, (fx, fy, kante, kante)));
                if let Some((k, _, _, _)) = treffer {
                    self.windows[wi].widgets[i].farbfeld_klick = k as i32;
                    self.windows[wi].widgets[i].farbfeld_zug = true;
                    return;
                }
            }
            // Klick auf einen Hinweis hinter dem Zeilenende: gemeldet, die
            // Marke bleibt (wie beim Farbfeld).
            if !self.was_mouse_down && wref.hinweise_klickbar && !wref.hinweise.is_empty()
               && Self::in_rect(mx, my, (ax, ay, fw, fh)) {
                let sicht = ((fh - 2 * pad) / lh).max(1);
                let koepfe = Self::ta_koepfe(wref, &rows, sicht, scroll).len() as i32;
                let treffer = self.ta_hinweis_rects(g, wref, ax, ay, &chars, &starts, &rows, sicht)
                    .into_iter()
                    .find(|&(_, hx, hy, hb, hh)| hy >= ay + pad + koepfe * lh
                          && Self::in_rect(mx, my, (hx, hy, hb, hh)));
                if let Some((li, _, _, _, _)) = treffer {
                    self.windows[wi].widgets[i].hinweis_klick = li as i64 + 1;
                    self.windows[wi].widgets[i].farbfeld_zug = true;
                    return;
                }
            }
            // Klick auf einen angehefteten Blockkopf springt dorthin -- die
            // Zeile darunter ist verdeckt, und der Kopf ist, was man sieht.
            if !self.was_mouse_down && Self::in_rect(mx, my, (ax, ay, fw, fh)) {
                let sicht = ((fh - 2 * pad) / lh).max(1);
                let koepfe = Self::ta_koepfe(wref, &rows, sicht, scroll);
                if !koepfe.is_empty() && my < ay + pad + koepfe.len() as i32 * lh {
                    let k = ((my - ay - pad).max(0) / lh) as usize;
                    let li = koepfe[k.min(koepfe.len() - 1)];
                    let c = starts[li] as i32;
                    let zrow = Self::ta_row_of(&rows, c as usize) as i32;
                    let w = &mut self.windows[wi].widgets[i];
                    w.caret = c; w.sel_anchor = c; w.marken_zusatz.clear();
                    w.scroll = zrow; w.rad_ziel = None;
                    w.rad_stand = Some((c, c, chars.len()));
                    w.farbfeld_zug = true;
                    return;
                }
            }
            // Klick auf den Faltpfeil klappt um, statt die Marke zu setzen --
            // sonst spraenge sie bei jedem Zuklappen mit.
            if !self.was_mouse_down && Self::in_rect(mx, my, (ax, ay, fw, fh)) {
                if let Some((fx, fbw)) = self.ta_falt_spalte(g, wref, ax, starts.len()) {
                    if mx >= fx && mx < fx + fbw
                       && wref.faltbar.iter().any(|&(v, _)| v == rli + 1) {
                        let h = Self::enc_widget(wi, i);
                        let _ = self.textarea_fold(h, rli as i64 + 1, None);
                        return;
                    }
                }
            }
            // Klick in die Nummernspalte: gemeldet (GUI_TEXTAREA_GUTTER_CLICKED),
            // die Marke bleibt, wo sie ist -- wer einen Haltepunkt setzt, will
            // nicht an den Zeilenanfang springen.
            // Der ganze Zug gehoert dazu (`farbfeld_zug` sperrt ihn wie beim
            // Farbfeld), sonst zoege das naechste Bild -- die Taste ist noch
            // unten -- die Marke doch noch an den Rand.
            if !self.was_mouse_down && !alt
               && self.ta_rand_zeile(g, wi, i, mx, my).is_some() {
                self.windows[wi].widgets[i].farbfeld_zug = true;
                return;
            }
            // Dieselbe Rechnung wie beim Zeichnen: hinter der Nummernspalte,
            // um den waagerechten Versatz verschoben. Liefe der Treffertest
            // hier auseinander, landete die Schreibmarke neben dem Klick.
            let gut = self.ta_gutter(g, &self.windows[wi].widgets[i],
                                     starts.len());
            let sx = self.windows[wi].widgets[i].scroll_x;
            let target = mx - (ax + pad + gut - sx);
            let sub: Vec<char> = chars[lstart..lend].to_vec();
            let off = Self::caret_index_at(g, &sub, target, ms) as usize;
            let idx = (lstart + off) as i32;
            if !self.was_mouse_down {
                if Self::in_rect(mx, my, (ax, ay, fw, fh)) {
                    // Alt+Klick legt eine WEITERE Marke, statt die eine zu
                    // versetzen -- dieselbe Geste wie in jedem Editor, und
                    // ohne Alt bleibt alles beim Alten.
                    if alt && idx != caret && !zusatz.iter().any(|&(c, _)| c == idx) {
                        zusatz.push((idx, idx));
                    } else if !alt {
                        // Ein gewoehnlicher Klick meint EINE Stelle: die
                        // weiteren Marken gehen weg (wie beim Rechtsklick
                        // und bei Strg+A) -- sonst landete das naechste
                        // Tippen auch dort, wo man nicht mehr hinsieht.
                        caret = idx; anchor = idx; zusatz.clear();
                        // Doppelklick waehlt das Wort, Dreifachklick die Zeile
                        // (samt Umbruch) -- wie in jedem Editor. Gezaehlt wird
                        // an DIESER Stelle, in 0,4 s.
                        let jetzt = g.get_time();
                        let w = &mut self.windows[wi].widgets[i];
                        let n = if jetzt - w.klick_zeit < 0.4 && (idx - w.klick_idx).abs() <= 2 { w.klick_n % 3 + 1 } else { 1 };
                        w.klick_n = n; w.klick_zeit = jetzt; w.klick_idx = idx;
                        if n == 2 {
                            let (a, e) = Self::wort_um(&chars, idx as usize);
                            anchor = a as i32; caret = e as i32;
                            w.wort_zug = true;
                        } else if n == 3 {
                            anchor = starts[rli] as i32;
                            caret = starts.get(rli + 1).copied().unwrap_or(chars.len()) as i32;
                            w.wort_zug = true;
                        }
                    }
                    // Wo ein Alt-Zug anfaengt: ein Klick ohne Bewegung bleibt
                    // die weitere Marke von oben, ein ZUG macht daraus eine
                    // Spaltenauswahl.
                    if alt {
                        self.windows[wi].widgets[i].spalten_start =
                            (rli as i32, idx - starts[rli] as i32);
                    }
                }
            } else if !alt && !self.windows[wi].widgets[i].wort_zug {
                caret = idx;   // Ziehen -> Selektion bis hierher
            } else if self.windows[wi].widgets[i].spalten_start.0 >= 0 {
                // Alt-Zug: ein RECHTECK statt eines Laufs. Jede Zeile bekommt
                // ihre eigene Marke samt Auswahl -- getippt wird in allen.
                let (z0, c0) = self.windows[wi].widgets[i].spalten_start;
                let marken = Self::spalten_marken(&chars, &starts, z0, c0,
                                                  rli as i32, idx - starts[rli] as i32);
                if !marken.is_empty() {
                    caret = marken[0].0;
                    anchor = marken[0].1;
                    zusatz = marken[1..].to_vec();
                }
            }
        }

        // Zeichen-Eingabe / Strg-Kombis (Selektion wird jeweils ersetzt).
        // Alle Marken in EINER Liste: die fuehrende zuerst. Jede Aenderung
        // laeuft ueber `an_marken` -- mit einer Marke ist das genau der Weg
        // von vorher, mit mehreren derselbe an jeder Stelle.
        let mut marken: Vec<(i32, i32)> = vec![(caret, anchor)];
        marken.extend(zusatz.iter().copied());
        // Wurde in diesem Bild Text an den Marken eingesetzt? Danach wird
        // geprueft, ob die Zeile nun ausrueckt.
        let mut geschrieben = false;
        // Ein eingefuegter BLOCK (mehrere Zeilen): das Ausruecken unten gilt
        // ihm nicht -- dessen letzte Zeile ist nicht getippt, und wenn sie
        // zufaellig `NEXT` oder `END IF` ist, verlor sie sonst ihre
        // Einrueckung. Ein einzelnes eingefuegtes Wort zaehlt weiter wie
        // getippt (so pruefen es die Tests, die ueber Strg+V "tippen").
        let mut eingefuegt = false;
        if !ctrl {
            let typed: String = g.pop_text_input().chars().filter(|c| !c.is_control()).collect();
            if !typed.is_empty() {
                let paare = self.windows[wi].widgets[i].paare.clone();
                if paare.is_empty() {
                    Self::an_marken(&mut chars, &mut marken, |_, c, a| {
                        ((c.min(a)) as usize, (c.max(a)) as usize, typed.clone())
                    });
                } else {
                    for ch in typed.chars() { Self::paar_tippen(&mut chars, &mut marken, ch, &paare); }
                }
                geschrieben = true;
            }
        } else if !kuerzel_weg {
            // Hat ein Menue-Kuerzel dieselbe Taste schon verbraucht, gehoert
            // sie ihm allein -- `Strg+Umschalt+V` als Kuerzel fuegte sonst
            // NEBENBEI die Zwischenablage ein, und zwar unsichtbar, weil der
            // Befehl daneben sein eigenes Fenster aufmacht.
            //
            // Alles markieren raeumt die weiteren Marken weg: eine Auswahl
            // ueber ALLES und daneben noch drei Marken ergaebe kein Bild,
            // das jemand im Kopf haette.
            if g.key_pressed(K_A) {
                marken.truncate(1);
                marken[0] = (chars.len() as i32, 0);
            }
            let (lo, hi) = (marken[0].0.min(marken[0].1), marken[0].0.max(marken[0].1));
            // Kopieren und Ausschneiden nehmen die FUEHRENDE Auswahl -- was
            // mehrere Stuecke in der Zwischenablage bedeuten sollen, ist
            // ausserhalb dieses Programms nicht ausgemacht.
            if g.key_pressed(K_C) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
            }
            if g.key_pressed(K_X) && lo != hi {
                g.clipboard_set(&chars[lo as usize..hi as usize].iter().collect::<String>());
                let mut nur = vec![marken[0]];
                Self::an_marken(&mut chars, &mut nur, |_, c, a| {
                    ((c.min(a)) as usize, (c.max(a)) as usize, String::new())
                });
                marken[0] = nur[0];
            }
            if g.key_pressed(K_V) {
                let ins: String = g.clipboard_get().chars().filter(|c| *c == '\n' || !c.is_control()).collect();
                Self::an_marken(&mut chars, &mut marken, |_, c, a| {
                    ((c.min(a)) as usize, (c.max(a)) as usize, ins.clone())
                });
                geschrieben = true;
                eingefuegt = ins.contains('\n');
            }
        }
        // Ausruecken: steht in der Zeile jetzt NUR ein Wort aus der
        // `aus`-Liste (END, NEXT, ELSE ...), rueckt sie eine Stufe zurueck.
        // Ohne das waere das Einruecken die halbe Sache -- das schliessende
        // Wort bliebe unter dem Rumpf stehen.
        //
        // Die Bedingung ist "die Zeile IST das Wort", nicht "faengt damit
        // an": so rueckt sie genau einmal aus, und was danach noch dazukommt
        // (`END IF`) aendert nichts mehr. Ein eingefuegter Block trifft sie
        // aus demselben Grund nicht -- und ein eingefuegter Block (Strg+V
        // mit mehreren Zeilen) bleibt ganz in Ruhe.
        if geschrieben && !eingefuegt {
            let aus = self.windows[wi].widgets[i].einzug_aus.clone();
            let breite = self.windows[wi].widgets[i].tabbreite.max(1) as usize;
            if !aus.is_empty() {
                // Nicht ueber `an_marken`: dort setzt jede Aenderung die Marke
                // ANS ENDE des Eingesetzten -- hier faellt aber etwas VOR ihr
                // weg, und sie muss mitrutschen statt an den Zeilenanfang zu
                // springen. (Genau das hat der Test gezeigt: die naechste
                // Eingabe landete dann vor der Einrueckung.)
                //
                // Die betroffenen Zeilen einmal einsammeln (zwei Marken in
                // derselben Zeile ruecken sie nur EINmal aus) und von hinten
                // nach vorn abarbeiten -- dann bleiben die Stellen davor gueltig.
                let mut zeilen: Vec<usize> = marken.iter()
                    .map(|m| Self::zeile_um(&chars, m.0.clamp(0, chars.len() as i32) as usize).0)
                    .collect();
                zeilen.sort_unstable();
                zeilen.dedup();
                zeilen.reverse();
                for von in zeilen {
                    let (_, bis) = Self::zeile_um(&chars, von);
                    let zeile: String = chars[von..bis].iter().collect();
                    let text = zeile.trim().to_uppercase();
                    let vorlauf = zeile.chars().take_while(|c| *c == ' ' || *c == '\t').count();
                    if vorlauf < breite || !aus.iter().any(|k| *k == text) { continue; }
                    chars.drain(von..von + breite);
                    for m in marken.iter_mut() {
                        if (m.0 as usize) >= von + breite { m.0 -= breite as i32; }
                        if (m.1 as usize) >= von + breite { m.1 -= breite as i32; }
                    }
                }
            }
        }
        // Tabulator rueckt ein, statt das Bedienelement zu wechseln --
        // aber nur, wenn das Feld ausdruecklich danach verlangt hat
        // (GUI_TEXTAREA_SET "tab_fuegt_ein"). Sonst bliebe man in einem
        // Formular im Textfeld haengen und kaeme nicht mehr heraus.
        // Gehoert der Tabulator ganz dem Aufrufer, MELDET das Feld ihn nur.
        // Dann faellt beides weg: das Nachsehen nach Abkuerzungen und das
        // Einruecken -- der Aufrufer macht es selbst, in SEINER Reihenfolge.
        let tab_ein = self.windows[wi].widgets[i].tab_fuegt_ein
                      && !self.windows[wi].widgets[i].tab_meldet;
        if self.windows[wi].widgets[i].tab_meldet && g.key_pressed(KEY_TAB) {
            self.windows[wi].widgets[i].tab_treffer = true;
        }
        // Steht links der fuehrenden Marke eine bekannte Abkuerzung, MELDET
        // das Feld sie und rueckt NICHT ein (GUI_TEXTAREA_ABBREV). Der
        // Aufrufer setzt dann sein Geruest an die Stelle.
        let mut abk = false;
        if tab_ein && g.key_pressed(KEY_TAB)
           && !self.windows[wi].widgets[i].abkuerzungen.is_empty() {
            let lo = marken[0].0.min(marken[0].1).clamp(0, chars.len() as i32) as usize;
            let anfang = chars[..lo].iter()
                .rposition(|c| !(c.is_alphanumeric() || *c == '_'))
                .map(|p| p + 1).unwrap_or(0);
            let wort: String = chars[anfang..lo].iter().collect::<String>().to_lowercase();
            if !wort.is_empty() {
                if let Some(k) = self.windows[wi].widgets[i].abkuerzungen.iter()
                    .position(|a| a.to_lowercase() == wort) {
                    self.windows[wi].widgets[i].abk_treffer = k as i32;
                    abk = true;
                }
            }
        }
        if tab_ein && !abk && g.key_pressed(KEY_TAB) {
            let breite = self.windows[wi].widgets[i].tabbreite.max(1);
            // Bis zur naechsten Spalte auffuellen, nicht stur `breite`
            // Leerzeichen: sonst steht die Einrueckung schief, sobald man
            // mitten in der Zeile tabbt.
            Self::an_marken(&mut chars, &mut marken, |ch, c, a| {
                let lo = c.min(a);
                let st = Self::line_starts(ch);
                let zeilen_start = st.iter().rposition(|&s| s as i32 <= lo).map(|r| st[r]).unwrap_or(0);
                let spalte = lo - zeilen_start as i32;
                let n = breite - (spalte % breite);
                (lo as usize, (c.max(a)) as usize, " ".repeat(n.max(1) as usize))
            });
        }
        // Enter = Umbruch (ersetzt evtl. Selektion), mit der Einrueckung
        // der laufenden Zeile, wenn das Feld danach verlangt.
        if g.key_pressed(KEY_ENTER) {
            let wref = Self::einzug_regeln(&self.windows[wi].widgets[i]);
            // Das Geruest wird VOR dem Umbruch entschieden -- danach steht die
            // Marke schon in der neuen Zeile, und die oeffnende waere nur noch
            // rueckwaerts zu finden. Nur bei EINER Marke und ohne Auswahl:
            // fuenf Abschluesse auf einmal will niemand, und bei einer Auswahl
            // ist die Zeile nach dem Umbruch eine andere.
            let geruest = if marken.len() == 1 && marken[0].0 == marken[0].1 {
                Self::geruest_schluss(&wref, &chars, marken[0].0.clamp(0, chars.len() as i32) as usize)
            } else { None };
            Self::an_marken(&mut chars, &mut marken, |ch, c, a| {
                let lo = c.min(a);
                let ein = Self::neuer_einzug(&wref, ch, lo as usize);
                (lo as usize, (c.max(a)) as usize, format!("\n{}", ein))
            });
            if let Some(zeile) = geruest {
                // Hinter der Marke einsetzen, ohne sie zu bewegen: die Marke
                // gehoert in die leere Zeile DAZWISCHEN, dort wird getippt.
                let p = marken[0].0.clamp(0, chars.len() as i32) as usize;
                let ein: Vec<char> = format!("\n{}", zeile).chars().collect();
                chars.splice(p..p, ein);
            }
        }
        // Backspace / Delete (Selektion hat Vorrang).
        if g.key_pressed(KEY_BACKSPACE) {
            // Zwischen einem leeren Paar nimmt die Ruecktaste beide -- das
            // schliessende kam ungefragt, es soll auch ungefragt gehen.
            let paare = self.windows[wi].widgets[i].paare.clone();
            Self::an_marken(&mut chars, &mut marken, |zs, c, a| {
                let (lo, hi) = (c.min(a), c.max(a));
                let leer_paar = lo > 0 && paare.iter().any(|p| {
                    zs.get(lo as usize - 1) == Some(&p.0) && zs.get(lo as usize) == Some(&p.1)
                });
                if lo != hi { (lo as usize, hi as usize, String::new()) }
                else if leer_paar { (lo as usize - 1, lo as usize + 1, String::new()) }
                else if lo > 0 { (lo as usize - 1, lo as usize, String::new()) }
                else { (0, 0, String::new()) }
            });
        }
        if g.key_pressed(KEY_DELETE) {
            let len0 = chars.len();
            Self::an_marken(&mut chars, &mut marken, |_, c, a| {
                let (lo, hi) = (c.min(a), c.max(a));
                if lo != hi { (lo as usize, hi as usize, String::new()) }
                else if (lo as usize) < len0 { (lo as usize, lo as usize + 1, String::new()) }
                else { (lo as usize, lo as usize, String::new()) }
            });
        }

        // Navigation (Shift = Selektion erweitern, sonst Anker = Caret).
        caret = marken[0].0; anchor = marken[0].1;
        zusatz = marken[1..].to_vec();
        let len = chars.len() as i32;
        caret = caret.clamp(0, len); anchor = anchor.clamp(0, len);
        let starts = Self::line_starts(&chars);
        let rows = {
            let wref = &self.windows[wi].widgets[i];
            self.ta_rows(g, wref, &chars, &starts, self.ta_breite(g, wref, starts.len()))
        };
        // Die Pfeile bewegen JEDE Marke -- blieben die weiteren stehen,
        // liefen sie beim ersten Tastendruck auseinander.
        // Pos1/Ende und Pfeile bewegen sich in SICHTBAREN Zeilen -- bei
        // Umbruch also innerhalb der umgebrochenen, wie in jedem Editor.
        let mut alle: Vec<(i32, i32)> = vec![(caret, anchor)];
        alle.extend(zusatz.iter().copied());
        for m in alle.iter_mut() {
            let (mut c, mut a) = (m.0.clamp(0, len), m.1.clamp(0, len));
            let row = Self::ta_row_of(&rows, c.max(0) as usize);
            let col = c - rows[row].1 as i32;
            let (k_zurueck, k_vor) = if richtung::zeile_rtl(&chars[rows[row].1..rows[row].2]) { (KEY_RIGHT, KEY_LEFT) } else { (KEY_LEFT, KEY_RIGHT) };
            if g.key_pressed(k_zurueck) {
                let (lo, hi) = (c.min(a), c.max(a));
                if !shift && lo != hi { c = lo; } else { c = (c - 1).max(0); }
                if !shift { a = c; }
            }
            if g.key_pressed(k_vor) {
                let (lo, hi) = (c.min(a), c.max(a));
                if !shift && lo != hi { c = hi; } else { c = (c + 1).min(len); }
                if !shift { a = c; }
            }
            // Mit Strg an den Anfang bzw. das Ende des ganzen Textes -- ohne
            // das blieb Strg+Ende stehen und Strg+Pos1 ging nur in die Zeile.
            if g.key_pressed(KEY_HOME) {
                c = if ctrl { 0 } else { rows[row].1 as i32 };
                if !shift { a = c; }
            }
            if g.key_pressed(KEY_END) {
                c = if ctrl { len } else { rows[row].2 as i32 };
                if !shift { a = c; }
            }
            if g.key_pressed(KEY_UP) && row > 0 && !kuerzel_weg {
                let (_, ps, pe) = rows[row - 1];
                c = (ps as i32 + col).min(pe as i32);
                if !shift { a = c; }
            }
            if g.key_pressed(KEY_DOWN) && row + 1 < rows.len() && !kuerzel_weg {
                let (_, ns, ne) = rows[row + 1];
                c = (ns as i32 + col).min(ne as i32);
                if !shift { a = c; }
            }
            *m = (c, a);
        }
        caret = alle[0].0; anchor = alle[0].1;
        // Zwei Marken an derselben Stelle sind eine -- ohne das Zusammen-
        // legen tippte man dort doppelt, sobald zwei aufeinandertreffen.
        zusatz = alle[1..].to_vec();
        zusatz.sort();
        zusatz.dedup_by_key(|m| m.0);
        zusatz.retain(|&(c, _)| c != caret);

        // Vertikal scrollen, damit die Caret-Zeile sichtbar bleibt.
        let starts2 = Self::line_starts(&chars);
        let rows2 = {
            let wref = &self.windows[wi].widgets[i];
            self.ta_rows(g, wref, &chars, &starts2, self.ta_breite(g, wref, starts2.len()))
        };
        let crow = Self::ta_row_of(&rows2, caret.max(0) as usize) as i32;
        let view_lines = ((fh - 2 * pad) / lh).max(1);
        let mut scroll = scroll;
        let gerollt = caret == caret0
            && self.windows[wi].widgets[i].rad_stand == Some((caret, anchor, chars.len()));
        if !gerollt {
            self.windows[wi].widgets[i].rad_stand = None;
            if crow < scroll { scroll = crow; }
            if crow >= scroll + view_lines { scroll = crow - view_lines + 1; }
            // Unter einem angehefteten Kopf waere die Marke unsichtbar --
            // dann rollt es zurueck, bis sie frei steht (die Koepfe werden
            // dabei weniger).
            let wref = &self.windows[wi].widgets[i];
            while scroll > 0 && crow < scroll + Self::ta_koepfe(wref, &rows2, view_lines, scroll).len() as i32 {
                scroll -= 1;
            }
        }
        let max_scroll = (rows2.len() as i32 - view_lines).max(0);
        scroll = scroll.clamp(0, max_scroll);

        // Waagerecht mitziehen, damit die Schreibmarke im Feld bleibt. Ohne
        // das endet eine lange Zeile am Rand und man tippt blind weiter --
        // in einem Code-Feld der haeufigste Fall ueberhaupt. Mit Umbruch
        // gibt es nichts mitzuziehen.
        let zeilen_start = rows2[crow.max(0) as usize].1;
        let vorspann: String = chars[zeilen_start..(caret as usize).min(chars.len())].iter().collect();
        let marke_px = self.wtext_width(g, &self.windows[wi].widgets[i], &vorspann);
        let gut = self.ta_gutter(g, &self.windows[wi].widgets[i], starts2.len());
        let sicht = (fw - 2 * pad - gut).max(1);
        let mut sx = self.windows[wi].widgets[i].scroll_x;
        if !gerollt {
            if marke_px - sx > sicht - self.sk(8) { sx = marke_px - sicht + self.sk(8); }
            if marke_px - sx < 0 { sx = marke_px; }
        }
        sx = sx.max(0);
        if self.windows[wi].widgets[i].umbruch { sx = 0; }

        let new_text: String = chars.iter().collect();
        let jetzt = g.get_time();
        let w = &mut self.windows[wi].widgets[i];
        if new_text != before {
            Self::undo_merken(w, &before, caret0, jetzt);
            Self::falten_nachziehen(w, &before, &new_text, caret0);
        }
        let geaendert = new_text != before;
        w.text = new_text; w.caret = caret; w.sel_anchor = anchor; w.scroll = scroll;
        w.scroll_x = sx;
        w.marken_zusatz = zusatz;
        if w.formatiert {
            stile_abgleichen(w);
            // Der Tippstil gilt fuer das naechste Getippte -- ist es getippt
            // oder wandert die Marke weg, gilt wieder der Stil davor.
            if geaendert || caret != caret0 { w.tipp_stil = None; }
        }
        // Links/Rechts und Backspace koennen die Marke in verborgenen Text
        // tragen -- dort waere jede weitere Taste unsichtbar wirksam.
        Self::falte_am_caret_oeffnen(w);
        if w.text != before {
            if let Some(f) = w.on_change.clone() { self.pending.push(f); }
        }
    }

    /// Das Mausrad fuer die gui. Mit gedrueckter Strg-Taste gehoert es dem
    /// PROGRAMM -- Strg+Rad heisst in jedem Editor "Zoom", und wenn dabei
    /// auch noch die Liste oder das Code-Feld unter der Maus rollte, sprang
    /// der Inhalt bei jedem Schritt weg. `MOUSEWHEEL_Y` liefert den Wert
    /// weiter, das Programm fragt dazu `KEYPRESSED(KEY_LCTRL)`.
    pub(super) fn rad(g: &Graphics) -> i64 {
        if g.key_ctrl() { 0 } else { g.pop_mouse_wheel() }
    }

    /// Mausrad ueber einem Textbereich: drei Zeilen je Schritt. Bis
    /// 2026-09-18 rollte er gar nicht -- nur Marke und Tasten bewegten den
    /// Ausschnitt, und in einer IDE ist das das Code-Feld.
    pub(super) fn textarea_wheel(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let wheel = Self::rad(g);
        if wheel == 0 { return; }
        let (zeilen, sicht) = {
            let wd = &self.windows[wi].widgets[i];
            let chars: Vec<char> = wd.text.chars().collect();
            let starts = Self::line_starts(&chars);
            let lh = self.ta_line_h(g, wd);
            let n = self.ta_rows(g, wd, &chars, &starts, self.ta_breite(g, wd, starts.len())).len() as i32;
            (n, ((wd.h - 2 * 5) / lh).max(1))
        };
        let max = (zeilen - sicht).max(0);
        let weich_an = self.m("uebergang") > 0;
        let w = &mut self.windows[wi].widgets[i];
        // Weich wie Liste und Baum: Radschritte addieren sich aufs ZIEL,
        // `uebergaenge` zieht Zeile um Zeile nach.
        let basis = w.rad_ziel.unwrap_or(w.scroll as f64);
        let ziel = (basis.round() as i32 - wheel as i32 * 3).clamp(0, max);
        if weich_an { w.rad_ziel = Some(ziel as f64); w.rad_letzt = w.scroll as f64; }
        else { w.scroll = ziel; }
        w.rad_stand = Some((w.caret, w.sel_anchor, w.text.chars().count()));
    }
    /// Rollbalken des Textbereichs: (x, y, Breite, Rinnenhoehe, Griff-y,
    /// Griff-Hoehe, Zeilen, sichtbare Zeilen) -- nur, wenn der Text nicht
    /// hineinpasst. Er liegt im rechten Innenabstand, ueber keinem Zeichen.
    /// EINE Quelle fuer Zeichnen, Klick und Ziehen.
    pub(super) fn ta_bar_geom(&self, g: &Graphics, wi: usize, i: usize) -> Option<(i32, i32, i32, i32, i32, i32, i32, i32)> {
        let wd = &self.windows[wi].widgets[i];
        if wd.kind != Kind::TextArea || !self.widget_shown(wi, wd) { return None; }
        let (ax, ay, w, h) = self.abs_rect(wi, wd);
        let lh = self.ta_line_h(g, wd).max(1);
        let chars: Vec<char> = wd.text.chars().collect();
        let starts = Self::line_starts(&chars);
        let n = self.ta_rows(g, wd, &chars, &starts, self.ta_breite(g, wd, starts.len())).len() as i32;
        let sicht = ((h - 2 * 5) / lh).max(1);
        if n <= sicht { return None; }
        let bw = 5;
        let (tx, ty, th) = (ax + w - 2 - bw, ay + 2, (h - 4).max(1));
        let thh = ((th as i64 * sicht as i64 / n.max(1) as i64) as i32).clamp(12.min(th), th);
        let max = (n - sicht).max(1);
        let thy = ty + ((th - thh) as i64 * wd.scroll.clamp(0, max) as i64 / max as i64) as i32;
        Some((tx, ty, bw, th, thy, thh, n, sicht))
    }
    /// Zeichenstelle unter der Maus (fuer den Rechtsklick) -- dieselbe Rechnung
    /// wie ein Klick in `edit_textarea`. None ausserhalb und in der
    /// Nummernspalte.
    pub(super) fn ta_index_at(&self, g: &Graphics, wi: usize, i: usize, mx: i32, my: i32) -> Option<i32> {
        let wd = &self.windows[wi].widgets[i];
        let (ax, ay, fw, fh) = self.abs_rect(wi, wd);
        if !Self::in_rect(mx, my, (ax, ay, fw, fh)) { return None; }
        let pad = 5;
        let lh = self.ta_line_h(g, wd).max(1);
        let chars: Vec<char> = wd.text.chars().collect();
        let starts = Self::line_starts(&chars);
        let rows = self.ta_rows(g, wd, &chars, &starts, self.ta_breite(g, wd, starts.len()));
        if rows.is_empty() { return Some(0); }
        // Unter einem angehefteten Kopf liegt verdeckter Text -- die Maus
        // zeigt auf den Kopf, nicht auf ihn.
        let sicht = ((fh - 2 * pad) / lh).max(1);
        if my < ay + pad + Self::ta_koepfe(wd, &rows, sicht, wd.scroll).len() as i32 * lh { return None; }
        let row = (wd.scroll + ((my - ay - pad).max(0) / lh)).max(0) as usize;
        let (_, lstart, lend) = rows[row.min(rows.len() - 1)];
        let gut = self.ta_gutter(g, wd, starts.len());
        if mx < ax + pad + gut { return None; }
        let target = mx - (ax + pad + gut - wd.scroll_x);
        let ms = self.mass(g, wd);
        let sub: Vec<char> = chars[lstart..lend].to_vec();
        Some((lstart + Self::caret_index_at(g, &sub, target, ms) as usize) as i32)
    }
    /// Rechtsklick setzt die Schreibmarke -- ein Kontextmenue ("Ausschneiden",
    /// "Zur Definition") soll sich auf die Stelle beziehen, auf die man zeigt.
    /// Trifft er die bestehende Auswahl, bleibt sie: dann ist sie gemeint.
    /// Die Nummernspalte gehoert weiter dem Haltepunkt (GUTTER_CLICKED).
    /// Stil-Bits auf die Auswahl anwenden. `an` = None schaltet um (Strg+B):
    /// tragen ALLE gewaehlten Zeichen das Bit schon, geht es weg, sonst kommt
    /// es auf alle. Ohne Auswahl wird es der Stil fuer das naechste Getippte.
    /// Eine Aenderung an der Auswahl ist ein eigener Verlaufsschritt.
    pub(super) fn stil_anwenden(&mut self, wi: usize, i: usize, bits: u8, an: Option<bool>, jetzt: f64) {
        let w = &mut self.windows[wi].widgets[i];
        stile_abgleichen(w);
        let n = w.stile.len() as i32;
        let (lo, hi) = (w.caret.clamp(0, n).min(w.sel_anchor.clamp(0, n)) as usize,
                        w.caret.clamp(0, n).max(w.sel_anchor.clamp(0, n)) as usize);
        if lo == hi {
            let jetzt_stil = w.tipp_stil.unwrap_or_else(|| stil_vor(w, lo));
            let setzen = an.unwrap_or(jetzt_stil & bits != bits);
            w.tipp_stil = Some(if setzen { jetzt_stil | bits } else { jetzt_stil & !bits });
            return;
        }
        let setzen = an.unwrap_or_else(|| !w.stile[lo..hi].iter().all(|&b| b & bits == bits));
        let neu: Vec<u8> = w.stile[lo..hi].iter().map(|&b| if setzen { b | bits } else { b & !bits }).collect();
        if neu[..] == w.stile[lo..hi] { return; }
        w.undo_zeit = -10.0;
        let t = w.text.clone();
        Self::undo_merken(w, &t, w.caret, jetzt);
        w.undo_zeit = -10.0;      // Tippen danach ist ein neuer Schritt
        w.stile[lo..hi].copy_from_slice(&neu);
        if let Some(f) = w.on_change.clone() { self.pending.push(f); }
    }

    pub(super) fn ta_formatiert(&mut self, h: i64, name: &str) -> Result<&mut Widget, String> {
        self.ta_wdg(h, name)?;
        let w = self.wdg_mut(h, name)?;
        if !w.formatiert {
            return Err(format!("{}: der Textbereich ist nicht formatiert -- erst GUI_TEXTAREA_SET(ta, \"formatiert\", 1)", name));
        }
        stile_abgleichen(w);
        Ok(w)
    }

    /// GUI_TEXTAREA_PAIRS(ta, paare$): je zwei Zeichen ein Paar, oeffnend
    /// und schliessend (`"()[]{}\"\""`); leer schaltet es ab.
    pub fn textarea_pairs(&mut self, h: i64, paare: &str) -> Result<(), String> {
        self.ta_wdg(h, "GUI_TEXTAREA_PAIRS")?;
        let z: Vec<char> = paare.chars().collect();
        if z.len() % 2 != 0 {
            return Err("GUI_TEXTAREA_PAIRS: erwartet je zwei Zeichen, oeffnend und schliessend (z. B. \"()[]\")".into());
        }
        let w = self.wdg_mut(h, "GUI_TEXTAREA_PAIRS")?;
        w.paare = z.chunks(2).map(|p| (p[0], p[1])).collect();
        Ok(())
    }

    /// Ein getipptes Zeichen an allen Marken, mit Klammerpaaren: ein
    /// oeffnendes setzt das schliessende gleich dahinter (um eine Auswahl
    /// herum: umschliessen), ein schliessendes vor einem gleichen tritt nur
    /// darueber. Anfuehrungszeichen (oeffnend = schliessend) paaren sich
    /// nicht hinter einem Wortzeichen und nicht mitten in einer
    /// Zeichenkette -- sonst kaeme bei `"Hallo"` am Ende ein drittes.
    /// Gepaart wird nur vor Leerraum, Zeilenende, Satzzeichen oder einem
    /// schliessenden Zeichen: vor einem Wort will man es selten.
    pub(super) fn paar_tippen(chars: &mut Vec<char>, marken: &mut [(i32, i32)], ch: char,
                   paare: &[(char, char)]) {
        let wort = |x: Option<char>| x.map_or(false, |c| c.is_alphanumeric() || c == '_' || c == '$');
        let mut reihe: Vec<usize> = (0..marken.len()).collect();
        reihe.sort_by_key(|&i| std::cmp::Reverse(marken[i].0.min(marken[i].1)));
        for i in reihe {
            let (c, a) = marken[i];
            let n = chars.len() as i32;
            let (lo, hi) = (c.min(a).clamp(0, n) as usize, c.max(a).clamp(0, n) as usize);
            let danach = chars.get(hi).copied();
            let davor = if lo > 0 { Some(chars[lo - 1]) } else { None };
            let zu = paare.iter().find(|p| p.0 == ch).map(|p| p.1);
            let schliesst = paare.iter().any(|p| p.1 == ch);
            if lo == hi && schliesst && danach == Some(ch) && (zu.is_none() || zu == Some(ch)) {
                marken[i] = ((lo + 1) as i32, (lo + 1) as i32);
                continue;
            }
            if let Some(zu) = zu {
                if lo != hi {
                    chars.insert(hi, zu);
                    chars.insert(lo, ch);
                    marken[i] = if c >= a { ((hi + 1) as i32, (lo + 1) as i32) }
                                else { ((lo + 1) as i32, (hi + 1) as i32) };
                    continue;
                }
                let frei = danach.map_or(true, |d| d.is_whitespace() || ",;:".contains(d)
                                         || paare.iter().any(|p| p.1 == d && p.0 != p.1));
                let im_text = zu == ch && {
                    let anfang = chars[..lo].iter().rposition(|x| *x == '\n').map_or(0, |k| k + 1);
                    chars[anfang..lo].iter().filter(|x| **x == ch).count() % 2 == 1
                };
                if frei && !(zu == ch && (wort(davor) || im_text)) {
                    chars.splice(lo..lo, [ch, zu]);
                    marken[i] = ((lo + 1) as i32, (lo + 1) as i32);
                    continue;
                }
            }
            chars.splice(lo..hi, [ch]);
            marken[i] = ((lo + 1) as i32, (lo + 1) as i32);
        }
    }

    /// GUI_TEXTAREA_SHARE(ansicht, von): `ansicht` zeigt ab jetzt denselben
    /// Text wie `von` (beide Textbereiche im selben Fenster); -1 loest die
    /// Verbindung. Jede Seite behaelt ihre Marke, ihren Ausschnitt und ihre
    /// Faltung -- geteilt wird nur der Text und der Verlauf.
    pub fn textarea_share(&mut self, h: i64, von: i64) -> Result<(), String> {
        self.ta_wdg(h, "GUI_TEXTAREA_SHARE")?;
        let (wi, i) = Self::dec_widget(h);
        if von < 0 {
            let w = self.wdg_mut(h, "GUI_TEXTAREA_SHARE")?;
            w.teil_von = None;
            w.teil_stand.clear();
            return Ok(());
        }
        self.ta_wdg(von, "GUI_TEXTAREA_SHARE")?;
        let (wv, iv) = Self::dec_widget(von);
        if wv != wi {
            return Err("GUI_TEXTAREA_SHARE: beide Textbereiche muessen im selben Fenster liegen".into());
        }
        if iv == i {
            return Err("GUI_TEXTAREA_SHARE: ein Textbereich kann keine Ansicht auf sich selbst sein".into());
        }
        if self.windows[wi].widgets[iv].teil_von.is_some() {
            return Err("GUI_TEXTAREA_SHARE: `von` ist selbst eine Ansicht -- nimm dessen Besitzer".into());
        }
        let (text, caret) = {
            let a = &self.windows[wi].widgets[iv];
            (a.text.clone(), a.caret)
        };
        let w = &mut self.windows[wi].widgets[i];
        w.teil_von = Some(iv);
        w.teil_stand = text.clone();
        w.text = text;
        w.caret = caret.clamp(0, w.text.chars().count() as i32);
        w.sel_anchor = w.caret;
        w.undo.clear();
        w.redo.clear();
        w.spans.clear();
        w.wellen.clear();
        w.gefaltet.clear();
        w.faltbar.clear();
        Ok(())
    }

    /// Eine Marke hinter eine Aenderung nachziehen: vor dem geaenderten
    /// Stueck bleibt sie, dahinter rueckt sie um die Laengendifferenz, mitten
    /// darin landet sie an seinem Ende.
    pub(super) fn marke_nachziehen(alt: &[char], neu: &[char], m: i32) -> i32 {
        let mut a = 0;
        while a < alt.len() && a < neu.len() && alt[a] == neu[a] { a += 1; }
        let mut e = 0;
        while e < alt.len() - a && e < neu.len() - a && alt[alt.len() - 1 - e] == neu[neu.len() - 1 - e] { e += 1; }
        let m = m.clamp(0, alt.len() as i32) as usize;
        let r = if m <= a { m } else if m >= alt.len() - e { m + neu.len() - alt.len() }
                else { neu.len() - e };
        r as i32
    }

    /// Geteilte Textbereiche abgleichen: hat die Ansicht sich seit dem
    /// letzten Abgleich geaendert, geht ihr Text an den Besitzer (samt
    /// Verlaufsschritt dort); sonst, wenn der Besitzer sich geaendert hat
    /// (Tippen, GUI_SET_TEXT, Rueckgaengig), an die Ansicht.
    pub(super) fn teil_pass(&mut self, jetzt: f64) {
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                let Some(iv) = self.windows[wi].widgets[i].teil_von else { continue; };
                if iv >= self.windows[wi].widgets.len() || self.windows[wi].widgets[iv].kind != Kind::TextArea {
                    self.windows[wi].widgets[i].teil_von = None;
                    continue;
                }
                let stand = self.windows[wi].widgets[i].teil_stand.clone();
                let eigen = self.windows[wi].widgets[i].text.clone();
                let besitzer = self.windows[wi].widgets[iv].text.clone();
                if eigen != stand {
                    let alt: Vec<char> = besitzer.chars().collect();
                    let neu: Vec<char> = eigen.chars().collect();
                    let a = &mut self.windows[wi].widgets[iv];
                    let c = Self::marke_nachziehen(&alt, &neu, a.caret);
                    let s = Self::marke_nachziehen(&alt, &neu, a.sel_anchor);
                    let ac = a.caret;
                    Self::undo_merken(a, &besitzer, ac, jetzt);
                    a.text = eigen.clone();
                    a.caret = c;
                    a.sel_anchor = s;
                    if a.formatiert { stile_abgleichen(a); }
                    let f = a.on_change.clone();
                    if let Some(f) = f { self.pending.push(f); }
                    let w = &mut self.windows[wi].widgets[i];
                    w.teil_stand = eigen;
                    w.undo.clear();
                    w.redo.clear();
                } else if besitzer != stand {
                    let alt: Vec<char> = eigen.chars().collect();
                    let neu: Vec<char> = besitzer.chars().collect();
                    let w = &mut self.windows[wi].widgets[i];
                    w.caret = Self::marke_nachziehen(&alt, &neu, w.caret);
                    w.sel_anchor = Self::marke_nachziehen(&alt, &neu, w.sel_anchor);
                    w.text = besitzer.clone();
                    w.teil_stand = besitzer;
                    w.marken_zusatz.clear();
                    if w.formatiert { stile_abgleichen(w); }
                }
            }
        }
    }

    /// GUI_TEXTAREA_STYLE(ta, stil$, an)
    pub fn textarea_style(&mut self, h: i64, bits: u8, an: bool, jetzt: f64) -> Result<(), String> {
        self.ta_formatiert(h, "GUI_TEXTAREA_STYLE")?;
        let (wi, i) = Self::dec_widget(h);
        self.stil_anwenden(wi, i, bits, Some(an), jetzt);
        Ok(())
    }

    /// GUI_TEXTAREA_GET_STYLE$(ta): was fuer die Auswahl GEMEINSAM gilt,
    /// ohne Auswahl der Stil, mit dem das naechste Getippte kaeme.
    pub fn textarea_get_style(&mut self, h: i64) -> Result<String, String> {
        let w = self.ta_formatiert(h, "GUI_TEXTAREA_GET_STYLE$")?;
        let n = w.stile.len() as i32;
        let (lo, hi) = (w.caret.clamp(0, n).min(w.sel_anchor.clamp(0, n)) as usize,
                        w.caret.clamp(0, n).max(w.sel_anchor.clamp(0, n)) as usize);
        let bits = if lo == hi { w.tipp_stil.unwrap_or_else(|| stil_vor(w, lo)) }
                   else { w.stile[lo..hi].iter().fold(0xFF, |a, &b| a & b) };
        Ok(crate::schnitt::stil_text(bits))
    }

    /// GUI_TEXTAREA_MARKDOWN$(ta)
    pub fn textarea_markdown(&mut self, h: i64) -> Result<String, String> {
        let w = self.ta_formatiert(h, "GUI_TEXTAREA_MARKDOWN$")?;
        let chars: Vec<char> = w.text.chars().collect();
        Ok(markdown_aus(&chars, &w.stile))
    }

    /// GUI_TEXTAREA_SET_MARKDOWN(ta, md$): wie GUI_SET_TEXT, nur geformt --
    /// und schaltet die Formate ein, falls sie es noch nicht sind.
    pub fn textarea_set_markdown(&mut self, h: i64, md: &str) -> Result<(), String> {
        self.ta_wdg(h, "GUI_TEXTAREA_SET_MARKDOWN")?;
        let (t, st) = markdown_ein(md);
        self.set_text(h, t)?;
        let w = self.wdg_mut(h, "GUI_TEXTAREA_SET_MARKDOWN")?;
        w.formatiert = true;
        w.stile = st; w.stile_text = w.text.clone(); w.tipp_stil = None;
        Ok(())
    }

    /// Rechtsklick ins Textfeld: die Marke wandert an die Stelle -- ausser
    /// er trifft die Auswahl, die ein Kontextmenue gleich kopieren soll.
    pub(super) fn ti_rechtsklick(&mut self, g: &Graphics, wi: usize, i: usize, mx: i32) {
        if !self.windows[wi].widgets[i].enabled { return; }
        let ms = self.mass(g, &self.windows[wi].widgets[i]);
        let (ax, _, _, _) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let w = &self.windows[wi].widgets[i];
        let chars: Vec<char> = if w.passwort { vec!['\u{2022}'; w.text.chars().count()] }
                               else { w.text.chars().collect() };
        let sl = self.ti_rand(w).0;
        let idx = Self::caret_index_at(g, &chars, mx - (ax + 5 + sl) + w.scroll, ms);
        let w = &mut self.windows[wi].widgets[i];
        let (lo, hi) = (w.caret.min(w.sel_anchor), w.caret.max(w.sel_anchor));
        if !(lo != hi && idx >= lo && idx <= hi) { w.caret = idx; w.sel_anchor = idx; }
        self.focus_widget = Some((wi, i));
    }

    pub(super) fn ta_rechtsklick(&mut self, g: &Graphics, wi: usize, i: usize, mx: i32, my: i32) {
        if !self.windows[wi].widgets[i].enabled { return; }
        if self.ta_rand_zeile(g, wi, i, mx, my).is_some() { return; }
        let idx = match self.ta_index_at(g, wi, i, mx, my) { Some(k) => k, None => return };
        let w = &mut self.windows[wi].widgets[i];
        let (lo, hi) = (w.caret.min(w.sel_anchor), w.caret.max(w.sel_anchor));
        if lo != hi && idx >= lo && idx <= hi { self.focus_widget = Some((wi, i)); return; }
        w.caret = idx;
        w.sel_anchor = idx;
        w.marken_zusatz.clear();
        self.focus_widget = Some((wi, i));
    }

    pub(super) fn listbox_wheel(&mut self, wi: usize, i: usize, h: i32, g: &mut Graphics) {
        let wheel = Self::rad(g);
        if wheel == 0 { return; }
        let zeile_h = self.list_zeile_h(&self.windows[wi].widgets[i]);   // vor dem mut-Zugriff
        let weich_an = self.m("uebergang") > 0;
        let w = &mut self.windows[wi].widgets[i];
        let zeilen = liste_ansicht(&w.items, w.list.as_deref()).len() as i32;
        let max_scroll = (zeilen * zeile_h - h).max(0);
        // Mehrere Raeder hintereinander addieren sich auf das ZIEL, nicht
        // auf den halb gerollten Stand -- sonst verschluckte schnelles Drehen
        // Zeilen.
        let basis = w.rad_ziel.unwrap_or(w.value);
        let nv = (basis as i32 - wheel as i32 * zeile_h).clamp(0, max_scroll) as f64;
        if weich_an {
            w.rad_ziel = Some(nv);
            w.rad_letzt = w.value;
        } else {
            w.value = nv;
        }
    }

    /// Mausrad ueber gesetztem Text.
    pub(super) fn richtext_wheel(&mut self, wi: usize, i: usize, h: i32, g: &mut Graphics) {
        let wheel = Self::rad(g);
        if wheel == 0 { return; }
        let schritt = self.sk(48);
        let r = self.windows[wi].widgets[i].rich.as_mut().unwrap();
        let max = (r.inhalt_h - h).max(0);
        r.scroll = (r.scroll - wheel as i32 * schritt).clamp(0, max);
    }

    pub(super) fn drag_slider(&mut self, wi: usize, i: usize, mx: i32, my: i32) {
        let (ax, ay, _, _) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let w = &mut self.windows[wi].widgets[i];
        // Senkrecht waechst der Wert nach OBEN -- wie an jedem Mischpult.
        let rel = if w.vert { 1.0 - ((my - ay) as f64 / (w.h - 1).max(1) as f64).clamp(0.0, 1.0) }
                  else { ((mx - ax) as f64 / (w.w - 1).max(1) as f64).clamp(0.0, 1.0) };
        let new_val = Self::regler_einrasten(w, w.min + rel * (w.max - w.min));
        if new_val != w.value {
            w.value = new_val;
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }
}
