//! Baeume und Behaelter: Tree, Dateibaum, Reiter, Akkordeon, Assistent, Baum mit Spalten, Uhrzeit.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Tree (Baum-Ansicht) ------------------------------------------------
    pub(super) fn tree_mut(&mut self, h: i64, fn_: &str) -> Result<&mut TreeState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Tree { return Err(format!("{}: Widget ist kein Baum", fn_)); }
        Ok(w.tree.as_mut().unwrap())
    }
    pub(super) fn tree_ref(&self, h: i64, fn_: &str) -> Result<&TreeState, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::Tree { return Err(format!("{}: Widget ist kein Baum", fn_)); }
        Ok(w.tree.as_ref().unwrap())
    }
    pub fn tree(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Tree, x, y, w, h);
        wd.tree = Some(Box::new(TreeState { selected: -1, hover: -1, anker: -1, ..Default::default() }));
        self.add_widget(win, "GUI_TREE", wd)
    }
    /// Knoten anhaengen. parent = -1 (Wurzel) oder eine bestehende Knoten-id.
    /// Liefert die neue Knoten-id (== Index, stabil bis GUI_TREE_CLEAR).
    pub fn tree_add(&mut self, h: i64, parent: i64, label: String) -> Result<i64, String> {
        let t = self.tree_mut(h, "GUI_TREE_ADD")?;
        let n = t.nodes.len() as i64;
        if parent < -1 || parent >= n {
            return Err(format!("GUI_TREE_ADD: ungueltiger parent {}", parent));
        }
        let level = if parent < 0 { 0 } else { t.nodes[parent as usize].level + 1 };
        if parent >= 0 { t.nodes[parent as usize].has_children = true; }
        t.nodes.push(TreeNode { label, parent: parent as i32, level, expanded: false, has_children: false, icon: -1, color: -1, sym: String::new() });
        t.sync();
        Ok((t.nodes.len() - 1) as i64)
    }
    pub fn tree_clear(&mut self, h: i64) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_CLEAR")?;
        t.nodes.clear(); t.selected = -1; t.hover = -1; t.scroll = 0;
        t.sel.clear(); t.anker = -1; t.auf_t.clear(); Ok(())
    }
    pub fn tree_selected(&self, h: i64) -> Result<i64, String> {
        Ok(self.tree_ref(h, "GUI_TREE_SELECTED")?.selected as i64)
    }
    pub fn tree_set_selected(&mut self, h: i64, node: i64) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_SET_SELECTED")?;
        let n = t.nodes.len() as i64;
        t.selected = if node >= 0 && node < n { node as i32 } else { -1 };
        // Mit Mehrfachauswahl ist ein gesetzter Knoten die GANZE Auswahl --
        // sonst zeigte der Baum eine Zeile hervorgehoben, die keiner Abfrage
        // gehoert.
        if t.multi {
            t.sync();
            for s in t.sel.iter_mut() { *s = false; }
            if t.selected >= 0 { t.sel[t.selected as usize] = true; t.anker = t.selected; }
        }
        Ok(())
    }
    /// Einstellungen am Baum (GUI_TREE_SET). Bisher nur `mehrfachauswahl`.
    pub fn tree_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_SET")?;
        match key.to_lowercase().as_str() {
            "mehrfachauswahl" | "multi" => {
                t.multi = wert != 0.0;
                t.sync();
                // Beim Einschalten zaehlt die bisherige Auswahl mit, sonst
                // stuende der Baum nach dem Umschalten scheinbar leer da.
                if t.multi && t.selected >= 0 && !t.sel.iter().any(|&s| s) {
                    let k = t.selected as usize;
                    if k < t.sel.len() { t.sel[k] = true; t.anker = t.selected; }
                }
            }
            "kaestchen" | "checkboxes" => { t.kaestchen = wert != 0.0; t.sync(); }
            "bearbeitbar" | "editable" => { t.bearbeitbar = wert != 0.0; if !t.bearbeitbar { t.edit = None; } }
            _ => return Err(format!(
                "GUI_TREE_SET: unbekannte Einstellung '{}' (gueltig: mehrfachauswahl, kaestchen, bearbeitbar)", key)),
        }
        Ok(())
    }
    /// Wie viele Knoten sind ausgewaehlt (GUI_TREE_SEL_COUNT)?
    pub fn tree_sel_count(&self, h: i64) -> Result<i64, String> {
        let t = self.tree_ref(h, "GUI_TREE_SEL_COUNT")?;
        if !t.multi { return Ok(if t.selected >= 0 { 1 } else { 0 }); }
        Ok(t.sel.iter().filter(|&&s| s).count() as i64)
    }
    /// Die i-te ausgewaehlte Knoten-id (GUI_TREE_SEL_NODE), in Knoten-Reihenfolge.
    pub fn tree_sel_node(&self, h: i64, i: i64) -> Result<i64, String> {
        let t = self.tree_ref(h, "GUI_TREE_SEL_NODE")?;
        if !t.multi { return Ok(if i == 0 { t.selected as i64 } else { -1 }); }
        Ok(t.sel.iter().enumerate().filter(|(_, &s)| s).map(|(k, _)| k as i64)
            .nth(i.max(0) as usize).unwrap_or(-1))
    }
    pub fn tree_is_selected(&self, h: i64, node: i64) -> Result<bool, String> {
        let t = self.tree_ref(h, "GUI_TREE_IS_SELECTED")?;
        if !t.multi { return Ok(node >= 0 && node as i32 == t.selected); }
        Ok(t.sel.get(node.max(-1) as usize).copied().unwrap_or(false) && node >= 0)
    }
    pub fn tree_select(&mut self, h: i64, node: i64, an: bool) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_SELECT")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_SELECT: ungueltige Knoten-id {}", node));
        }
        t.sync();
        t.sel[node as usize] = an;
        if an { t.selected = node as i32; t.anker = node as i32; }
        Ok(())
    }
    pub fn tree_clear_selection(&mut self, h: i64) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_CLEAR_SELECTION")?;
        for s in t.sel.iter_mut() { *s = false; }
        t.selected = -1; t.anker = -1; Ok(())
    }
    pub fn tree_label(&self, h: i64, node: i64) -> Result<String, String> {
        let t = self.tree_ref(h, "GUI_TREE_LABEL")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_LABEL: ungueltige Knoten-id {}", node));
        }
        Ok(t.nodes[node as usize].label.clone())
    }
    pub fn tree_expand(&mut self, h: i64, node: i64, flag: bool) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_EXPAND")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_EXPAND: ungueltige Knoten-id {}", node));
        }
        t.nodes[node as usize].expanded = flag;
        // Beim Dateibaum entscheidet die Liste der offenen Wege, was gelesen
        // wird -- das Flag am Knoten allein waere beim naechsten Neuaufbau weg.
        if t.datei.is_some() {
            let weg = t.datei.as_ref().unwrap().pfade.get(node as usize).cloned();
            if let Some(w) = weg { Self::ft_offen_setzen(t, &w, flag); dateibaum_neu(t); }
        }
        Ok(())
    }
    pub fn tree_checked(&self, h: i64, node: i64) -> Result<bool, String> {
        let t = self.tree_ref(h, "GUI_TREE_CHECKED")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_CHECKED: ungueltige Knoten-id {}", node));
        }
        Ok(t.checks.get(node as usize).copied().unwrap_or(false))
    }
    pub fn tree_set_checked(&mut self, h: i64, node: i64, an: bool) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_SET_CHECKED")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_SET_CHECKED: ungueltige Knoten-id {}", node));
        }
        t.sync();
        t.checks[node as usize] = an;
        Self::ft_haken_merken(t, node as usize, an);
        Ok(())
    }

    // --- Dateibaum (GUI_FILETREE) -------------------------------------------
    /// Ein Baum, der seine Knoten selbst von der Platte holt. Er benutzt die
    /// Darstellung und den Treffertest des Baums -- neu ist nur, WOHER die
    /// Knoten kommen und dass nach aussen Wege stehen, wo sonst Nummern stehen.
    pub fn filetree(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, wurzel: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Tree, x, y, w, h);
        let d = DateiBaum {
            wurzel: Self::ft_weg_normal(&wurzel),
            ordner_zuerst: true,
            bild_ordner: -1, bild_datei: -1,
            ..Default::default()
        };
        let mut ts = TreeState { selected: -1, hover: -1, anker: -1, ..Default::default() };
        ts.datei = Some(Box::new(d));
        dateibaum_neu(&mut ts);
        wd.tree = Some(Box::new(ts));
        self.add_widget(win, "GUI_FILETREE", wd)
    }
    pub(super) fn ft_mut(&mut self, h: i64, fn_: &str) -> Result<&mut TreeState, String> {
        let t = self.tree_mut(h, fn_)?;
        if t.datei.is_none() { return Err(format!("{}: Widget ist kein Dateibaum (GUI_FILETREE)", fn_)); }
        Ok(t)
    }
    pub(super) fn ft_ref(&self, h: i64, fn_: &str) -> Result<(&TreeState, &DateiBaum), String> {
        let t = self.tree_ref(h, fn_)?;
        match t.datei.as_deref() {
            Some(d) => Ok((t, d)),
            None => Err(format!("{}: Widget ist kein Dateibaum (GUI_FILETREE)", fn_)),
        }
    }
    /// Rueckstriche zu Schraegstrichen, kein Trennzeichen am Ende -- in EINER
    /// Schreibweise laesst sich vergleichen, in zweien zeigt derselbe Ordner
    /// zweimal auf verschiedene Aeste.
    pub(super) fn ft_weg_normal(p: &str) -> String {
        let s = p.replace('\\', "/");
        let s = s.trim_end_matches('/').to_string();
        if s.is_empty() { p.replace('\\', "/") } else { s }
    }
    /// Ein Weg von aussen: absolut (unterhalb der Wurzel) oder schon relativ.
    pub(super) fn ft_rel(d: &DateiBaum, pfad: &str) -> String {
        let p = Self::ft_weg_normal(pfad);
        let w = format!("{}/", d.wurzel);
        if p.len() > w.len() && p[..w.len()].eq_ignore_ascii_case(&w) { p[w.len()..].to_string() }
        else if p.eq_ignore_ascii_case(&d.wurzel) { String::new() }
        else { p }
    }
    /// Ein Weg nach aussen: immer der volle, in der Schreibweise des Systems.
    pub(super) fn ft_abs(d: &DateiBaum, rel: &str) -> String {
        if rel.is_empty() { return d.wurzel.clone(); }
        let mut p = std::path::PathBuf::from(&d.wurzel);
        for t in rel.split('/') { p.push(t); }
        p.to_string_lossy().to_string()
    }
    pub(super) fn ft_offen_setzen(t: &mut TreeState, weg: &str, an: bool) {
        let Some(d) = t.datei.as_mut() else { return };
        let da = d.offen.iter().position(|o| o == weg);
        match (an, da) {
            (true, None) => d.offen.push(weg.to_string()),
            (false, Some(i)) => { d.offen.remove(i); }
            _ => {}
        }
    }
    pub(super) fn ft_haken_merken(t: &mut TreeState, node: usize, an: bool) {
        let Some(d) = t.datei.as_mut() else { return };
        let Some(weg) = d.pfade.get(node).cloned() else { return };
        let da = d.gehakt.iter().position(|g| g == &weg);
        match (an, da) {
            (true, None) => d.gehakt.push(weg),
            (false, Some(i)) => { d.gehakt.remove(i); }
            _ => {}
        }
    }
    /// GUI_FILETREE_SET(ft, key$, wert).
    pub fn filetree_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_SET")?;
        let an = wert != 0.0;
        match key.to_lowercase().as_str() {
            "ordner_zuerst" | "dirs_first" => t.datei.as_mut().unwrap().ordner_zuerst = an,
            "verborgene" | "hidden" => t.datei.as_mut().unwrap().verborgene = an,
            "nur_ordner" | "dirs_only" => t.datei.as_mut().unwrap().nur_ordner = an,
            // Wie im Datei-Bereich einer Entwicklungsumgebung: ein Klick auf
            // einen Ordner klappt ihn um. Ohne das tut ein Klick auf eine
            // Ordnerzeile scheinbar nichts -- nur das schmale Dreieck wirkt.
            "klick_klappt" | "click_expands" => t.datei.as_mut().unwrap().klick_klappt = an,
            "nach_art" | "by_type" => t.datei.as_mut().unwrap().nach_art = an,
            "mehrfachauswahl" | "multi" => { t.multi = an; t.sync(); }
            "kaestchen" | "checkboxes" => t.kaestchen = an,
            // In MILLISEKUNDEN, wie ueberall sonst -- innen in Sekunden, weil
            // die Uhr der Grafik so zaehlt.
            "auffrischen" | "refresh" => {
                t.datei.as_mut().unwrap().auffrischen = (wert.max(0.0)) / 1000.0;
                t.datei.as_mut().unwrap().naechste = 0.0;
            }
            _ => return Err(format!("GUI_FILETREE_SET: unbekannte Einstellung '{}' (gueltig: \
                ordner_zuerst, verborgene, nur_ordner, nach_art, klick_klappt, mehrfachauswahl, kaestchen, auffrischen)", key)),
        }
        dateibaum_neu(t);
        Ok(())
    }
    /// Welche Dateien der Baum zeigt: Muster mit `;` getrennt, leer = alle.
    pub fn filetree_filter(&mut self, h: i64, muster: &str) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_FILTER")?;
        t.datei.as_mut().unwrap().muster = Self::ft_liste(muster);
        dateibaum_neu(t); Ok(())
    }
    /// Was der Baum ueberspringt -- Namen oder Muster, mit `;` getrennt, fuer
    /// Ordner UND Dateien (`target;__pycache__;_*`).
    pub fn filetree_skip(&mut self, h: i64, namen: &str) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_SKIP")?;
        t.datei.as_mut().unwrap().ausnahmen = Self::ft_liste(namen);
        dateibaum_neu(t); Ok(())
    }
    pub(super) fn ft_liste(s: &str) -> Vec<String> {
        s.split(';').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
    }
    pub fn filetree_set_root(&mut self, h: i64, pfad: String) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_SET_ROOT")?;
        let d = t.datei.as_mut().unwrap();
        // Eine neue Wurzel ist ein neuer Baum: offene Aeste und Haken zeigen
        // auf Wege, die es dort nicht gibt.
        d.wurzel = Self::ft_weg_normal(&pfad);
        d.offen.clear(); d.gehakt.clear();
        t.selected = -1; t.anker = -1; t.scroll = 0;
        dateibaum_neu(t); Ok(())
    }
    pub fn filetree_root(&self, h: i64) -> Result<String, String> {
        Ok(self.ft_ref(h, "GUI_FILETREE_ROOT")?.1.wurzel.clone())
    }
    pub fn filetree_refresh(&mut self, h: i64) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_REFRESH")?;
        dateibaum_neu(t); Ok(())
    }
    pub fn filetree_selected(&self, h: i64) -> Result<String, String> {
        let (t, d) = self.ft_ref(h, "GUI_FILETREE_SELECTED")?;
        if t.selected < 0 { return Ok(String::new()); }
        Ok(d.pfade.get(t.selected as usize).map(|p| Self::ft_abs(d, p)).unwrap_or_default())
    }
    /// Ein Doppelklick auf eine Datei -- ein Bild lang, wie GUI_CLICKED.
    pub fn filetree_activated(&self, h: i64) -> Result<String, String> {
        let (_, d) = self.ft_ref(h, "GUI_FILETREE_ACTIVATED")?;
        if d.aktiviert.is_empty() { return Ok(String::new()); }
        Ok(Self::ft_abs(d, &d.aktiviert))
    }
    /// Leerer Weg = die Auswahl.
    pub fn filetree_is_dir(&self, h: i64, pfad: &str) -> Result<bool, String> {
        let (t, d) = self.ft_ref(h, "GUI_FILETREE_IS_DIR")?;
        let k = if pfad.is_empty() { t.selected } else { d.knoten(&Self::ft_rel(d, pfad)) };
        if k < 0 { return Ok(false); }
        Ok(d.ordner.get(k as usize).copied().unwrap_or(false))
    }
    pub fn filetree_select(&mut self, h: i64, pfad: &str) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_SELECT")?;
        let rel = Self::ft_rel(t.datei.as_ref().unwrap(), pfad);
        if rel.is_empty() {
            t.selected = -1; t.anker = -1;
            for s in t.sel.iter_mut() { *s = false; }
            return Ok(());
        }
        // Die Vorfahren aufklappen: auf einen Weg zeigen, der zugeklappt ist,
        // hiesse sonst, eine Auswahl zu setzen, die niemand sieht.
        let teile: Vec<&str> = rel.split('/').collect();
        let mut bisher = String::new();
        for t2 in teile.iter().take(teile.len().saturating_sub(1)) {
            if !bisher.is_empty() { bisher.push('/'); }
            bisher.push_str(t2);
            Self::ft_offen_setzen(t, &bisher, true);
        }
        dateibaum_neu(t);
        let k = t.datei.as_ref().unwrap().knoten(&rel);
        t.selected = k; t.anker = k;
        for s in t.sel.iter_mut() { *s = false; }
        if k >= 0 { t.sync(); t.sel[k as usize] = true; }
        // Ins Bild rollen: eine Auswahl, die unter dem Rand liegt, sieht
        // niemand. Steht sie schon im Bild, bleibt der Ausschnitt.
        if k >= 0 {
            let (wi, idx) = Self::dec_widget(h);
            let zh = self.sk(TREE_ROW_H);
            let hoehe = (self.abs_rect(wi, &self.windows[wi].widgets[idx]).3 - 2).max(zh);
            let w = &mut self.windows[wi].widgets[idx];
            w.rad_ziel = None;
            let t = w.tree.as_mut().unwrap();
            let vis = Self::tree_visible(t);
            if let Some(r) = vis.iter().position(|&x| x as i32 == k) {
                let y = r as i32 * zh;
                if y < t.scroll || y + zh > t.scroll + hoehe {
                    let max = (vis.len() as i32 * zh - hoehe).max(0);
                    t.scroll = (y - (hoehe - zh) / 2).clamp(0, max);
                }
            }
        }
        Ok(())
    }
    pub fn filetree_expand(&mut self, h: i64, pfad: &str, an: bool) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_EXPAND")?;
        let rel = Self::ft_rel(t.datei.as_ref().unwrap(), pfad);
        Self::ft_offen_setzen(t, &rel, an);
        dateibaum_neu(t); Ok(())
    }
    /// Sichtbare Knoten -- in der Reihenfolge, in der sie dastehen.
    pub fn filetree_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.ft_ref(h, "GUI_FILETREE_COUNT")?.1.pfade.len() as i64)
    }
    pub fn filetree_path(&self, h: i64, i: i64) -> Result<String, String> {
        let (_, d) = self.ft_ref(h, "GUI_FILETREE_PATH")?;
        Ok(d.pfade.get(i.max(-1) as usize).filter(|_| i >= 0).map(|p| Self::ft_abs(d, p)).unwrap_or_default())
    }
    pub fn filetree_sel_count(&self, h: i64) -> Result<i64, String> {
        let (t, _) = self.ft_ref(h, "GUI_FILETREE_SEL_COUNT")?;
        if !t.multi { return Ok(if t.selected >= 0 { 1 } else { 0 }); }
        Ok(t.sel.iter().filter(|&&s| s).count() as i64)
    }
    pub fn filetree_sel_path(&self, h: i64, i: i64) -> Result<String, String> {
        let (t, d) = self.ft_ref(h, "GUI_FILETREE_SEL_PATH")?;
        let k = if !t.multi { if i == 0 { t.selected as i64 } else { -1 } }
                else { t.sel.iter().enumerate().filter(|(_, &s)| s).map(|(k, _)| k as i64)
                        .nth(i.max(0) as usize).unwrap_or(-1) };
        if k < 0 { return Ok(String::new()); }
        Ok(d.pfade.get(k as usize).map(|p| Self::ft_abs(d, p)).unwrap_or_default())
    }
    pub fn filetree_checked(&self, h: i64, pfad: &str) -> Result<bool, String> {
        let (_, d) = self.ft_ref(h, "GUI_FILETREE_CHECKED")?;
        let rel = Self::ft_rel(d, pfad);
        Ok(d.gehakt.iter().any(|g| g == &rel))
    }
    pub fn filetree_set_checked(&mut self, h: i64, pfad: &str, an: bool) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_SET_CHECKED")?;
        let rel = Self::ft_rel(t.datei.as_ref().unwrap(), pfad);
        let d = t.datei.as_mut().unwrap();
        let da = d.gehakt.iter().position(|g| g == &rel);
        match (an, da) {
            (true, None) => d.gehakt.push(rel),
            (false, Some(i)) => { d.gehakt.remove(i); }
            _ => {}
        }
        dateibaum_neu(t); Ok(())
    }
    /// Angehakt zaehlt AUCH, was gerade zugeklappt ist -- ein Haken ist eine
    /// Entscheidung, und die verschwindet nicht, weil man einen Ast zumacht.
    pub fn filetree_checked_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.ft_ref(h, "GUI_FILETREE_CHECKED_COUNT")?.1.gehakt.len() as i64)
    }
    pub fn filetree_checked_path(&self, h: i64, i: i64) -> Result<String, String> {
        let (_, d) = self.ft_ref(h, "GUI_FILETREE_CHECKED_PATH")?;
        Ok(d.gehakt.get(i.max(-1) as usize).filter(|_| i >= 0).map(|p| Self::ft_abs(d, p)).unwrap_or_default())
    }
    /// Textfarbe je Dateiart: `muster` wie beim Filter ("*.dh;*.dhform"),
    /// "/" fuer Ordner; -1 nimmt die Farbe wieder weg. Ein Muster, das schon
    /// eine Farbe hat, bekommt die neue an derselben Stelle -- die Reihenfolge
    /// bleibt, und das erste passende gilt.
    pub fn filetree_color(&mut self, h: i64, muster: &str, farbe: i64) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_COLOR")?;
        let liste = Self::ft_liste(muster);
        if liste.is_empty() {
            return Err("GUI_FILETREE_COLOR: kein Muster -- so: \"*.dh\", \"*.json;*.ini\" oder \"/\" fuer Ordner".to_string());
        }
        let d = t.datei.as_mut().unwrap();
        for m in liste {
            match d.farben.iter().position(|(x, _)| *x == m) {
                Some(k) if farbe < 0 => { d.farben.remove(k); }
                Some(k) => d.farben[k].1 = farbe,
                None if farbe >= 0 => d.farben.push((m, farbe)),
                None => {}
            }
        }
        dateibaum_neu(t); Ok(())
    }

    /// Ein eingebautes Sinnbild je Dateiart, nach derselben Regel wie
    /// GUI_FILETREE_COLOR ("/" = Ordner, das erste passende gilt). "" nimmt
    /// es weg. Ein Bild aus GUI_FILETREE_ICONS geht vor.
    pub fn filetree_icon(&mut self, h: i64, muster: &str, name: &str) -> Result<(), String> {
        let sym = if name.trim().is_empty() { String::new() } else { sinnbild_pruefen(name, "GUI_FILETREE_ICON")? };
        let t = self.ft_mut(h, "GUI_FILETREE_ICON")?;
        let liste = Self::ft_liste(muster);
        if liste.is_empty() {
            return Err("GUI_FILETREE_ICON: kein Muster -- so: \"*.dh\", \"*.json;*.ini\" oder \"/\" fuer Ordner".to_string());
        }
        let d = t.datei.as_mut().unwrap();
        for m in liste {
            match d.symbole.iter().position(|(x, _)| *x == m) {
                Some(k) if sym.is_empty() => { d.symbole.remove(k); }
                Some(k) => d.symbole[k].1 = sym.clone(),
                None if !sym.is_empty() => d.symbole.push((m, sym.clone())),
                None => {}
            }
        }
        dateibaum_neu(t); Ok(())
    }

    pub fn filetree_icons(&mut self, h: i64, ordner: i64, datei: i64) -> Result<(), String> {
        let t = self.ft_mut(h, "GUI_FILETREE_ICONS")?;
        let d = t.datei.as_mut().unwrap();
        d.bild_ordner = ordner.max(-1); d.bild_datei = datei.max(-1);
        dateibaum_neu(t); Ok(())
    }

    // --- Reiter im Fenster (GUI_TABCONTROL) ---------------------------------
    /// Bis Stand 24 gab es Reiter nur AM FENSTER (GUI_TABS): ein Einstellungs-
    /// kasten mit zwei Karteikarten in einer Ecke des Fensters liess sich
    /// damit nicht bauen. Das Reiterwerk ist ein Widget wie jedes andere; die
    /// Kinder bekommen ihre Seite zugewiesen und werden sonst genauso
    /// angelegt und verortet wie ohne Reiter.
    pub fn tabcontrol(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::TabControl, x, y, w, h);
        wd.tabctl = Some(Box::new(TabCtlState::default()));
        wd.sel = 0;
        self.add_widget(win, "GUI_TABCONTROL", wd)
    }
    pub(super) fn tc_mut(&mut self, h: i64, fn_: &str) -> Result<&mut Widget, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::TabControl { return Err(format!("{}: Widget ist kein Reiterwerk (GUI_TABCONTROL)", fn_)); }
        Ok(w)
    }
    pub(super) fn tc_ref(&self, h: i64, fn_: &str) -> Result<&Widget, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::TabControl { return Err(format!("{}: Widget ist kein Reiterwerk (GUI_TABCONTROL)", fn_)); }
        Ok(w)
    }
    /// Eine Seite anhaengen -- liefert ihre Nummer.
    pub fn tabcontrol_add(&mut self, h: i64, titel: String) -> Result<i64, String> {
        let w = self.tc_mut(h, "GUI_TABCONTROL_ADD")?;
        w.items.push(titel);
        if w.sel < 0 { w.sel = 0; }
        Ok((w.items.len() - 1) as i64)
    }
    pub fn tabcontrol_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.tc_ref(h, "GUI_TABCONTROL_COUNT")?.items.len() as i64)
    }
    pub fn tabcontrol_page(&self, h: i64) -> Result<i64, String> {
        Ok(self.tc_ref(h, "GUI_TABCONTROL_PAGE")?.sel as i64)
    }
    pub fn tabcontrol_set_page(&mut self, h: i64, seite: i64) -> Result<(), String> {
        let w = self.tc_mut(h, "GUI_TABCONTROL_SET_PAGE")?;
        if seite < 0 || seite >= w.items.len() as i64 {
            return Err(format!("GUI_TABCONTROL_SET_PAGE: Seite {} gibt es nicht", seite));
        }
        w.sel = seite as i32; Ok(())
    }
    pub fn tabcontrol_title(&self, h: i64, i: i64) -> Result<String, String> {
        let w = self.tc_ref(h, "GUI_TABCONTROL_TITLE")?;
        Ok(w.items.get(i.max(-1) as usize).filter(|_| i >= 0).cloned().unwrap_or_default())
    }
    pub fn tabcontrol_set_title(&mut self, h: i64, i: i64, titel: String) -> Result<(), String> {
        let w = self.tc_mut(h, "GUI_TABCONTROL_SET_TITLE")?;
        if i < 0 || i >= w.items.len() as i64 {
            return Err(format!("GUI_TABCONTROL_SET_TITLE: Seite {} gibt es nicht", i));
        }
        w.items[i as usize] = titel; Ok(())
    }
    /// Ein Widget auf eine Seite legen. Es behaelt seine Lage im Fenster --
    /// gezeigt wird es, solange seine Seite vorn ist.
    pub fn tabcontrol_add_widget(&mut self, h: i64, kind: i64, seite: i64) -> Result<(), String> {
        let (tw, ti) = Self::dec_widget(h);
        let (kw, ki) = Self::dec_widget(kind);
        self.tc_ref(h, "GUI_TABCONTROL_ADD_WIDGET")?;
        self.wdg(kind, "GUI_TABCONTROL_ADD_WIDGET")?;
        if tw != kw { return Err("GUI_TABCONTROL_ADD_WIDGET: Widget gehoert in ein anderes Fenster".into()); }
        if ti == ki { return Err("GUI_TABCONTROL_ADD_WIDGET: das Reiterwerk kann nicht sein eigenes Kind sein".into()); }
        let n = self.tc_ref(h, "GUI_TABCONTROL_ADD_WIDGET")?.items.len() as i64;
        if seite < 0 || seite >= n {
            return Err(format!("GUI_TABCONTROL_ADD_WIDGET: Seite {} gibt es nicht (erst GUI_TABCONTROL_ADD)", seite));
        }
        let w = self.tc_mut(h, "GUI_TABCONTROL_ADD_WIDGET")?;
        let st = w.tabctl.get_or_insert_with(|| Box::new(TabCtlState::default()));
        st.kinder.retain(|&(k, _)| k != ki);
        st.kinder.push((ki, seite as i32));
        Ok(())
    }
    /// Eine Seite entfernen. Die Kinder darauf gehoeren danach zu keiner Seite
    /// mehr (sie sind wieder immer sichtbar) -- sie zu ZERSTOEREN waere eine
    /// Entscheidung, die dem Programm gehoert.
    pub fn tabcontrol_remove(&mut self, h: i64, i: i64) -> Result<(), String> {
        let w = self.tc_mut(h, "GUI_TABCONTROL_REMOVE")?;
        if i < 0 || i >= w.items.len() as i64 {
            return Err(format!("GUI_TABCONTROL_REMOVE: Seite {} gibt es nicht", i));
        }
        w.items.remove(i as usize);
        if let Some(st) = w.tabctl.as_mut() {
            st.kinder.retain(|&(_, s)| s != i as i32);
            for (_, s) in st.kinder.iter_mut() { if *s > i as i32 { *s -= 1; } }
        }
        let n = w.items.len() as i32;
        w.sel = w.sel.min(n - 1).max(if n > 0 { 0 } else { -1 });
        Ok(())
    }

    // --- Akkordeon (GUI_ACCORDION) --------------------------------------------
    pub fn accordion(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Accordion, x, y, w, h);
        wd.tabctl = Some(Box::new(TabCtlState::default()));
        wd.akk = Some(Box::new(AkkState { fokus: 0, umgeschaltet: -1, ..Default::default() }));
        self.add_widget(win, "GUI_ACCORDION", wd)
    }
    pub(super) fn akk_mut(&mut self, h: i64, fn_: &str) -> Result<&mut Widget, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Accordion { return Err(format!("{}: Widget ist kein Akkordeon (GUI_ACCORDION)", fn_)); }
        Ok(w)
    }
    pub(super) fn akk_ref(&self, h: i64, fn_: &str) -> Result<&Widget, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::Accordion { return Err(format!("{}: Widget ist kein Akkordeon (GUI_ACCORDION)", fn_)); }
        Ok(w)
    }
    pub(super) fn akk_nr(w: &Widget, i: i64, fn_: &str) -> Result<usize, String> {
        if i < 0 || i >= w.items.len() as i64 {
            return Err(format!("{}: Abschnitt {} gibt es nicht (erst GUI_ACCORDION_ADD)", fn_, i));
        }
        Ok(i as usize)
    }
    pub fn accordion_add(&mut self, h: i64, titel: String) -> Result<i64, String> {
        let w = self.akk_mut(h, "GUI_ACCORDION_ADD")?;
        w.items.push(titel);
        let a = w.akk.get_or_insert_with(Default::default);
        a.offen.push(false);
        a.hoehen.push(0);
        Ok((w.items.len() - 1) as i64)
    }
    pub fn accordion_count(&self, h: i64) -> Result<i64, String> {
        Ok(self.akk_ref(h, "GUI_ACCORDION_COUNT")?.items.len() as i64)
    }
    pub fn accordion_title(&self, h: i64, i: i64) -> Result<String, String> {
        let w = self.akk_ref(h, "GUI_ACCORDION_TITLE$")?;
        Ok(w.items[Self::akk_nr(w, i, "GUI_ACCORDION_TITLE$")?].clone())
    }
    pub fn accordion_set_title(&mut self, h: i64, i: i64, titel: String) -> Result<(), String> {
        let w = self.akk_mut(h, "GUI_ACCORDION_SET_TITLE")?;
        let k = Self::akk_nr(w, i, "GUI_ACCORDION_SET_TITLE")?;
        w.items[k] = titel; Ok(())
    }
    /// Ein Widget in einen Abschnitt legen. Seine Lage zaehlt ab da als Lage
    /// IM Abschnitt -- (10, 8) heisst 10 rechts, 8 unter dem Kopf.
    pub fn accordion_add_widget(&mut self, h: i64, kind: i64, nr: i64) -> Result<(), String> {
        let f = "GUI_ACCORDION_ADD_WIDGET";
        let (tw, ti) = Self::dec_widget(h);
        let (kw, ki) = Self::dec_widget(kind);
        let (kx, ky) = { let k = self.wdg(kind, f)?; (k.x, k.y) };
        { let w = self.akk_ref(h, f)?; Self::akk_nr(w, nr, f)?; }
        if tw != kw { return Err(format!("{}: Widget gehoert in ein anderes Fenster", f)); }
        if ti == ki { return Err(format!("{}: das Akkordeon kann nicht sein eigenes Kind sein", f)); }
        if ki < ti { return Err(format!("{}: das Kind muss NACH dem Akkordeon angelegt sein, sonst zeichnet es darunter", f)); }
        let w = self.akk_mut(h, f)?;
        let st = w.tabctl.get_or_insert_with(Default::default);
        st.kinder.retain(|&(k, _)| k != ki);
        st.kinder.push((ki, nr as i32));
        let a = w.akk.get_or_insert_with(Default::default);
        a.lage.retain(|&(k, _, _)| k != ki);
        a.lage.push((ki, kx, ky));
        Ok(())
    }
    /// Auf- oder zuklappen. Ohne `mehrere` schliesst Oeffnen die anderen.
    pub fn accordion_open(&mut self, h: i64, i: i64, an: bool) -> Result<(), String> {
        let w = self.akk_mut(h, "GUI_ACCORDION_OPEN")?;
        let k = Self::akk_nr(w, i, "GUI_ACCORDION_OPEN")?;
        Self::akk_setzen(w, k, an);
        Ok(())
    }
    pub(super) fn akk_setzen(w: &mut Widget, k: usize, an: bool) {
        let n = w.items.len();
        let a = w.akk.get_or_insert_with(Default::default);
        a.offen.resize(n, false);
        if an && !a.mehrere { for o in a.offen.iter_mut() { *o = false; } }
        if k < n { a.offen[k] = an; }
    }
    pub fn accordion_is_open(&self, h: i64, i: i64) -> Result<bool, String> {
        let w = self.akk_ref(h, "GUI_ACCORDION_IS_OPEN")?;
        let k = Self::akk_nr(w, i, "GUI_ACCORDION_IS_OPEN")?;
        Ok(w.akk.as_ref().and_then(|a| a.offen.get(k).copied()).unwrap_or(false))
    }
    pub fn accordion_toggled(&self, h: i64) -> Result<i64, String> {
        Ok(self.akk_ref(h, "GUI_ACCORDION_TOGGLED")?.akk.as_ref().map(|a| a.umgeschaltet as i64).unwrap_or(-1))
    }
    pub fn accordion_set(&mut self, h: i64, key: &str, v: i64) -> Result<(), String> {
        let w = self.akk_mut(h, "GUI_ACCORDION_SET")?;
        let a = w.akk.get_or_insert_with(Default::default);
        match key {
            "mehrere" | "multiple" => {
                a.mehrere = v != 0;
                // Aus "mehrere" zurueck zu einem: nur der erste offene bleibt.
                if !a.mehrere {
                    let mut gesehen = false;
                    for o in a.offen.iter_mut() { if *o { if gesehen { *o = false; } gesehen = true; } }
                }
            }
            _ => return Err(format!("GUI_ACCORDION_SET: unbekannter Schluessel '{}' (erlaubt: mehrere)", key)),
        }
        Ok(())
    }
    /// Eigene Inhaltshoehe eines Abschnitts; 0 = nach den Kindern.
    pub fn accordion_set_height(&mut self, h: i64, i: i64, px: i64) -> Result<(), String> {
        let f = "GUI_ACCORDION_SET_HEIGHT";
        if px < 0 { return Err(format!("{}: Hoehe muss >= 0 sein", f)); }
        let s = self.sk(px as i32);
        let w = self.akk_mut(h, f)?;
        let k = Self::akk_nr(w, i, f)?;
        let n = w.items.len();
        let a = w.akk.get_or_insert_with(Default::default);
        a.hoehen.resize(n, 0);
        a.hoehen[k] = s;
        Ok(())
    }
    /// Umschalten wie ein Klick auf den Kopf: mit Ereignis und on_change.
    pub(super) fn akk_umschalten(&mut self, wi: usize, i: usize, k: usize) {
        let w = &mut self.windows[wi].widgets[i];
        let an = !w.akk.as_ref().and_then(|a| a.offen.get(k).copied()).unwrap_or(false);
        Self::akk_setzen(w, k, an);
        if let Some(a) = w.akk.as_mut() { a.umgeschaltet = k as i32; a.fokus = k as i32; }
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }

    /// Je Bild, VOR dem Layout: Koepfe verorten, die Kinder an ihre Stelle im
    /// Abschnitt setzen. Ein Behaelter unter den Kindern verteilt danach
    /// seine eigenen -- darum vor `layout_pass`.
    ///
    /// Hier laeuft auch der Oeffnungsgrad (`auf_t`) weiter, nicht erst in
    /// `uebergaenge` -- sonst stuende die Lage ein Bild hinter dem Zustand,
    /// und mit `uebergang` 0 zeigte ein GUI_UPDATE nach GUI_ACCORDION_OPEN
    /// noch den alten Stand.
    pub(super) fn akk_pass(&mut self, dt: f64) {
        let kopf = self.sk(AKK_KOPF_H);
        let rand = self.sk(8);
        let dauer = self.m("uebergang").max(0) as f32;
        let schritt = if dauer > 0.0 { (dt as f32 * 1000.0 / dauer).clamp(0.0, 1.0) } else { 1.0 };
        for win in self.windows.iter_mut() {
            for wdg in win.widgets.iter_mut() {
                if let Some(a) = wdg.akk.as_mut() {
                    let n = a.offen.len();
                    for s in 0..n {
                        let ziel = if a.offen[s] { 1.0 } else { 0.0 };
                        if s >= a.auf_t.len() { a.auf_t.push(ziel); } else { naeher(&mut a.auf_t[s], ziel, schritt); }
                    }
                    a.auf_t.truncate(n);
                }
            }
        }
        for wi in 0..self.windows.len() {
            for p in 0..self.windows[wi].widgets.len() {
                if self.windows[wi].widgets[p].kind != Kind::Accordion { continue; }
                let (ax, ay, ah, n) = { let w = &self.windows[wi].widgets[p]; (w.x, w.y, w.h, w.items.len()) };
                let (lage, kinder, offen, hoehen, scroll, auf_t) = {
                    let w = &self.windows[wi].widgets[p];
                    let a = match w.akk.as_ref() { Some(a) => a, None => continue };
                    (a.lage.clone(), w.tabctl.as_ref().map(|s| s.kinder.clone()).unwrap_or_default(),
                     a.offen.clone(), a.hoehen.clone(), a.scroll, a.auf_t.clone())
                };
                let seite_von = |k: usize| kinder.iter().find(|&&(c, _)| c == k).map(|&(_, s)| s).unwrap_or(-1);
                let mut koepfe = Vec::with_capacity(n);
                let mut y = 0;
                for s in 0..n {
                    let kopf_y = y;
                    y += kopf;
                    let auf = offen.get(s).copied().unwrap_or(false);
                    let eigen = hoehen.get(s).copied().unwrap_or(0);
                    // Gemessen wird IMMER die volle Hoehe -- auch beim
                    // Zuklappen muss die Flaeche wissen, wovon sie schrumpft.
                    let mut inhalt = 0;
                    for &(k, dx, dy) in &lage {
                        if seite_von(k) != s as i32 { continue; }
                        let kh = self.windows[wi].widgets.get(k).map(|c| c.h).unwrap_or(0);
                        inhalt = inhalt.max(dy + kh + rand);
                        if let Some(c) = self.windows[wi].widgets.get_mut(k) {
                            c.x = ax + dx;
                            c.y = ay + y + dy - scroll;
                        }
                    }
                    let voll = if eigen > 0 { eigen } else { inhalt };
                    let t = auf_t.get(s).copied().unwrap_or(if auf { 1.0 } else { 0.0 });
                    let inhalt = (voll as f64 * weich(t)).round() as i32;
                    koepfe.push((kopf_y, inhalt));
                    y += inhalt;
                }
                let a = self.windows[wi].widgets[p].akk.as_mut().unwrap();
                a.koepfe = koepfe;
                a.inhalt_h = y;
                a.scroll = a.scroll.clamp(0, (y - ah).max(0));
            }
        }
    }
    /// Der sichtbare Teil des Abschnitts, in dem ein Akkordeon-Kind liegt,
    /// als Bildschirm-Rechteck (x, y, b, h) -- `None`, wenn das Widget kein
    /// Kind eines Akkordeons ist. Hoehe 0 = der Abschnitt ist zu. Eine
    /// Quelle fuer das Beschneiden beim Zeichnen; `widget_shown` rechnet
    /// dieselbe Unterkante fuer die Frage "ganz zu sehen?".
    pub(super) fn akk_ausschnitt(&self, wi: usize, w: &Widget) -> Option<(i32, i32, i32, i32)> {
        if w.tc_von < 0 || !(w.alive && w.visible) { return None; }
        let tc = self.windows[wi].widgets.get(w.tc_von as usize)?;
        if tc.kind != Kind::Accordion { return None; }
        let a = tc.akk.as_ref()?;
        if !self.widget_shown(wi, tc) { return Some((0, 0, 0, 0)); }
        let (ax, ay, aw, ah) = self.abs_rect(wi, tc);
        let (ky, inh) = a.koepfe.get(w.tc_seite.max(0) as usize).copied().unwrap_or((0, 0));
        let oben = (ay + ky + self.sk(AKK_KOPF_H) - a.scroll).max(ay + 1);
        let unten = (ay + ky + self.sk(AKK_KOPF_H) + inh - a.scroll).min(ay + ah - 1);
        Some((ax + 1, oben, (aw - 2).max(0), (unten - oben).max(0)))
    }
    /// Welcher Kopf liegt unter y (absolut)? Dieselbe Rechnung wie das Zeichnen.
    pub(super) fn akk_kopf_unter(&self, w: &Widget, ay: i32, my: i32) -> i32 {
        let a = match w.akk.as_ref() { Some(a) => a, None => return -1 };
        let kopf = self.sk(AKK_KOPF_H);
        for (s, &(ky, _)) in a.koepfe.iter().enumerate() {
            let y0 = ay + ky - a.scroll;
            if my >= y0 && my < y0 + kopf && my >= ay && my < ay + w.h { return s as i32; }
        }
        -1
    }

    // --- Assistent (GUI_WIZARD) -----------------------------------------------
    pub fn wizard(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::Wizard, x, y, w, h);
        wd.tabctl = Some(Box::new(TabCtlState::default()));
        wd.wz = Some(Box::new(WzState::default()));
        wd.sel = 0;
        self.add_widget(win, "GUI_WIZARD", wd)
    }
    pub(super) fn wz_mut(&mut self, h: i64, fn_: &str) -> Result<&mut Widget, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::Wizard { return Err(format!("{}: Widget ist kein Assistent (GUI_WIZARD)", fn_)); }
        Ok(w)
    }
    pub(super) fn wz_ref(&self, h: i64, fn_: &str) -> Result<&Widget, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::Wizard { return Err(format!("{}: Widget ist kein Assistent (GUI_WIZARD)", fn_)); }
        Ok(w)
    }
    pub fn wizard_add(&mut self, h: i64, titel: String) -> Result<i64, String> {
        let w = self.wz_mut(h, "GUI_WIZARD_ADD")?;
        w.items.push(titel);
        if w.sel < 0 { w.sel = 0; }
        Ok((w.items.len() - 1) as i64)
    }
    pub fn wizard_count(&self, h: i64) -> Result<i64, String> { Ok(self.wz_ref(h, "GUI_WIZARD_COUNT")?.items.len() as i64) }
    pub fn wizard_step(&self, h: i64) -> Result<i64, String> { Ok(self.wz_ref(h, "GUI_WIZARD_STEP")?.sel as i64) }
    pub fn wizard_set_step(&mut self, h: i64, k: i64) -> Result<(), String> {
        let w = self.wz_mut(h, "GUI_WIZARD_SET_STEP")?;
        if k < 0 || k >= w.items.len() as i64 {
            return Err(format!("GUI_WIZARD_SET_STEP: Schritt {} gibt es nicht", k));
        }
        w.sel = k as i32; Ok(())
    }
    /// Ein Widget auf einen Schritt legen. Es behaelt seine Lage im Fenster --
    /// gezeigt wird es, solange sein Schritt dran ist (wie beim Reiterwerk).
    pub fn wizard_add_widget(&mut self, h: i64, kind: i64, schritt: i64) -> Result<(), String> {
        let f = "GUI_WIZARD_ADD_WIDGET";
        let (tw, ti) = Self::dec_widget(h);
        let (kw, ki) = Self::dec_widget(kind);
        self.wdg(kind, f)?;
        let n = self.wz_ref(h, f)?.items.len() as i64;
        if tw != kw { return Err(format!("{}: Widget gehoert in ein anderes Fenster", f)); }
        if ti == ki { return Err(format!("{}: der Assistent kann nicht sein eigenes Kind sein", f)); }
        if schritt < 0 || schritt >= n { return Err(format!("{}: Schritt {} gibt es nicht (erst GUI_WIZARD_ADD)", f, schritt)); }
        let w = self.wz_mut(h, f)?;
        let st = w.tabctl.get_or_insert_with(Default::default);
        st.kinder.retain(|&(k, _)| k != ki);
        st.kinder.push((ki, schritt as i32));
        Ok(())
    }
    pub(super) fn wz_ereignis(&self, h: i64, fn_: &str) -> Result<u8, String> {
        Ok(self.wz_ref(h, fn_)?.wz.as_ref().map(|z| z.ereignis).unwrap_or(0))
    }
    pub fn wizard_changed(&self, h: i64) -> Result<bool, String> { Ok(self.wz_ereignis(h, "GUI_WIZARD_CHANGED")? == 1) }
    pub fn wizard_finished(&self, h: i64) -> Result<bool, String> { Ok(self.wz_ereignis(h, "GUI_WIZARD_FINISHED")? == 2) }
    pub fn wizard_cancelled(&self, h: i64) -> Result<bool, String> { Ok(self.wz_ereignis(h, "GUI_WIZARD_CANCELLED")? == 3) }
    pub fn wizard_enable_next(&mut self, h: i64, an: bool) -> Result<(), String> {
        let w = self.wz_mut(h, "GUI_WIZARD_ENABLE_NEXT")?;
        w.wz.get_or_insert_with(Default::default).weiter_aus = !an; Ok(())
    }
    pub fn wizard_set(&mut self, h: i64, key: &str, v: i64) -> Result<(), String> {
        let w = self.wz_mut(h, "GUI_WIZARD_SET")?;
        let z = w.wz.get_or_insert_with(Default::default);
        match key {
            "pruefen" | "validate" => z.pruefen = v != 0,
            _ => return Err(format!("GUI_WIZARD_SET: unbekannter Schluessel '{}' (erlaubt: pruefen)", key)),
        }
        Ok(())
    }
    pub fn wizard_labels(&mut self, h: i64, t: [String; 4]) -> Result<(), String> {
        let w = self.wz_mut(h, "GUI_WIZARD_LABELS")?;
        w.wz.get_or_insert_with(Default::default).texte = t; Ok(())
    }
    /// Weiter wie der Knopf: mit Pruefung der Felder, am letzten Schritt
    /// "fertig". Liefert, ob es weiterging.
    pub fn wizard_next(&mut self, h: i64) -> Result<bool, String> {
        self.wz_ref(h, "GUI_WIZARD_NEXT")?;
        let (wi, i) = Self::dec_widget(h);
        Ok(self.wz_weiter(wi, i))
    }
    pub fn wizard_back(&mut self, h: i64) -> Result<bool, String> {
        self.wz_ref(h, "GUI_WIZARD_BACK")?;
        let (wi, i) = Self::dec_widget(h);
        Ok(self.wz_zurueck(wi, i))
    }
    pub(super) fn wz_geom(&self, w: &Widget, ax: i32, ay: i32) -> WzGeom {
        let kopf_h = self.sk(WZ_KOPF_H);
        let fuss_h = self.sk(WZ_FUSS_H);
        let fuss_y = ay + w.h - fuss_h;
        let rand = self.sk(10);
        // Drei Knoepfe und zwei Luecken muessen in die Breite passen -- sonst
        // laege Abbrechen in einem schmalen Assistenten ueber Zurueck.
        let kb = self.sk(112).min(((w.w - 2 * rand - 2 * self.sk(8)) / 3).max(self.sk(40)));
        let kh = self.sk(30);
        let ky = fuss_y + (fuss_h - kh) / 2;
        let weiter = (ax + w.w - rand - kb, ky, kb, kh);
        let zurueck = (weiter.0 - self.sk(8) - kb, ky, kb, kh);
        let abbrechen = (ax + rand, ky, kb, kh);
        WzGeom { kopf_h, fuss_y, zurueck, weiter, abbrechen }
    }
    pub(super) fn wz_weiter(&mut self, wi: usize, i: usize) -> bool {
        let (sel, n, pruefen, aus) = {
            let w = &self.windows[wi].widgets[i];
            let z = w.wz.as_ref();
            (w.sel, w.items.len() as i32, z.map(|z| z.pruefen).unwrap_or(false), z.map(|z| z.weiter_aus).unwrap_or(false))
        };
        if aus || n == 0 { return false; }
        if pruefen {
            // Nur die Felder DIESES Schritts -- ein Fehler auf einem spaeteren
            // haette hier noch keinen Ausweg.
            let mut erstes: Option<usize> = None;
            for k in 0..self.windows[wi].widgets.len() {
                let (gehoert, pruefbar) = {
                    let c = &self.windows[wi].widgets[k];
                    (c.tc_von == i as i32 && c.tc_seite == sel, !c.regeln.is_empty() && c.alive && c.visible && c.enabled)
                };
                if !gehoert || !pruefbar { continue; }
                if !self.widget_pruefen(wi, k).is_empty() && erstes.is_none() { erstes = Some(k); }
            }
            if let Some(k) = erstes {
                if self.windows[wi].widgets[k].kind.fokussierbar() {
                    self.focus_widget = Some((wi, k)); self.focus_window = Some(wi);
                }
                return false;
            }
        }
        let w = &mut self.windows[wi].widgets[i];
        if sel + 1 >= n {
            if let Some(z) = w.wz.as_mut() { z.ereignis = 2; }
            let f = w.on_click.clone();
            if let Some(f) = f { self.pending.push(f); }
        } else {
            w.sel = sel + 1;
            if let Some(z) = w.wz.as_mut() { z.ereignis = 1; }
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
        true
    }
    pub(super) fn wz_zurueck(&mut self, wi: usize, i: usize) -> bool {
        let w = &mut self.windows[wi].widgets[i];
        if w.sel <= 0 { return false; }
        w.sel -= 1;
        if let Some(z) = w.wz.as_mut() { z.ereignis = 1; }
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
        true
    }
    pub(super) fn wz_abbrechen(&mut self, wi: usize, i: usize) {
        if let Some(z) = self.windows[wi].widgets[i].wz.as_mut() { z.ereignis = 3; }
    }
    /// Knopf k (0 zurueck, 1 weiter/fertig, 2 abbrechen) -- fuer Klick und
    /// Bildschirmleser derselbe Weg.
    pub(super) fn wz_knopf(&mut self, wi: usize, i: usize, k: usize) {
        match k { 0 => { self.wz_zurueck(wi, i); } 1 => { self.wz_weiter(wi, i); } _ => self.wz_abbrechen(wi, i) }
    }

    // --- Baum mit Spalten (GUI_TREETABLE) ----------------------------------------
    pub fn treetable(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let hd = self.table(win, x, y, w, h)?;
        let t = self.tbl_mut(hd, "GUI_TREETABLE")?;
        t.baum = true;
        t.rebuild_view();
        Ok(hd)
    }
    pub(super) fn tt_mut(&mut self, h: i64, fn_: &str) -> Result<&mut TableState, String> {
        let t = self.tbl_mut(h, fn_)?;
        if !t.baum { return Err(format!("{}: Tabelle ist kein Baum (GUI_TREETABLE oder GUI_TABLE_SET(t, \"baum\", 1))", fn_)); }
        t.baum_sync();
        Ok(t)
    }
    pub(super) fn tt_zeile(t: &TableState, r: i64, fn_: &str) -> Result<usize, String> {
        if r < 0 || r >= t.rows.len() as i64 { return Err(format!("{}: Zeile {} gibt es nicht", fn_, r)); }
        Ok(r as usize)
    }
    /// Eine Zeile unter `eltern` anhaengen (-1 = oben) -> ihre Datenzeile.
    pub fn treetable_add(&mut self, h: i64, eltern: i64, zellen: Vec<String>) -> Result<i64, String> {
        let f = "GUI_TREETABLE_ADD";
        let t = self.tt_mut(h, f)?;
        if eltern >= 0 { Self::tt_zeile(t, eltern, f)?; }
        t.rows.push(zellen.into_iter().map(Cell::text).collect());
        t.eltern.push(if eltern >= 0 { eltern as i32 } else { -1 });
        t.offen.push(false);
        t.row_fg.push(-1);
        t.row_bg.push(-1);
        t.zeile_angehaengt();
        Ok((t.rows.len() - 1) as i64)
    }
    pub fn treetable_expand(&mut self, h: i64, r: i64, an: bool) -> Result<(), String> {
        let f = "GUI_TREETABLE_EXPAND";
        let t = self.tt_mut(h, f)?;
        let r = Self::tt_zeile(t, r, f)?;
        t.offen[r] = an;
        t.rebuild_view();
        Ok(())
    }
    pub fn treetable_expand_all(&mut self, h: i64, an: bool) -> Result<(), String> {
        let t = self.tt_mut(h, "GUI_TREETABLE_EXPAND_ALL")?;
        for o in t.offen.iter_mut() { *o = an; }
        t.rebuild_view();
        Ok(())
    }
    pub fn treetable_expanded(&mut self, h: i64, r: i64) -> Result<bool, String> {
        let f = "GUI_TREETABLE_EXPANDED";
        let t = self.tt_mut(h, f)?;
        Ok(t.offen[Self::tt_zeile(t, r, f)?])
    }
    pub fn treetable_parent(&mut self, h: i64, r: i64) -> Result<i64, String> {
        let f = "GUI_TREETABLE_PARENT";
        let t = self.tt_mut(h, f)?;
        Ok(t.eltern[Self::tt_zeile(t, r, f)?] as i64)
    }
    pub fn treetable_level(&mut self, h: i64, r: i64) -> Result<i64, String> {
        let f = "GUI_TREETABLE_LEVEL";
        let t = self.tt_mut(h, f)?;
        let mut r = Self::tt_zeile(t, r, f)?;
        let mut n = 0;
        while t.eltern[r] >= 0 && n <= t.rows.len() { r = t.eltern[r] as usize; n += 1; }
        Ok(n as i64)
    }
    /// Eine Zeile unter andere Eltern haengen -- ein Kreis ist ein Fehler.
    pub fn treetable_set_parent(&mut self, h: i64, r: i64, eltern: i64) -> Result<(), String> {
        let f = "GUI_TREETABLE_SET_PARENT";
        let t = self.tt_mut(h, f)?;
        let r = Self::tt_zeile(t, r, f)?;
        if eltern >= 0 {
            let mut e = Self::tt_zeile(t, eltern, f)?;
            let mut n = 0;
            loop {
                if e == r { return Err(format!("{}: Zeile {} kann nicht unter sich selbst oder ihren Nachkommen haengen", f, r)); }
                if t.eltern[e] < 0 || n > t.rows.len() { break; }
                e = t.eltern[e] as usize; n += 1;
            }
        }
        t.eltern[r] = if eltern >= 0 { eltern as i32 } else { -1 };
        t.rebuild_view();
        Ok(())
    }
    /// Einzug der ersten sichtbaren Spalte einer Baumzeile: Tiefe plus Platz
    /// fuer das Dreieck. EINE Stelle fuer Zeichnen und Treffertest.
    pub(super) fn tt_einzug(&self, t: &TableState, r: usize) -> (i32, i32) {
        let stufe = self.sk(16);
        let e = t.ebene.get(r).copied().unwrap_or(0);
        (e * stufe, e * stufe + stufe)
    }

    /// Liegt (mx, my) auf einer Flaeche, die den KINDERN gehoert? Akkordeon
    /// und Assistent liegen vor ihren Kindern, und der erste Treffer gewinnt
    /// -- ohne diese Ausnahme schluckten sie jeden Klick in ein Feld darin.
    /// Sie nehmen nur auf ihren eigenen Teilen an: Koepfe und Rollbalken
    /// bzw. Schrittanzeige und Knopfreihe.
    pub(super) fn luft_fuer_kinder(&self, win: usize, i: usize, mx: i32, my: i32) -> bool {
        let w = &self.windows[win].widgets[i];
        let (ax, ay, aw, _) = self.abs_rect(win, w);
        match w.kind {
            Kind::Accordion => {
                let balken = w.akk.as_ref().map(|a| a.inhalt_h > w.h).unwrap_or(false) && mx >= ax + aw - self.sk(8);
                !balken && self.akk_kopf_unter(w, ay, my) < 0
            }
            Kind::Wizard => {
                let gm = self.wz_geom(w, ax, ay);
                my >= ay + gm.kopf_h && my < gm.fuss_y
            }
            // Unter den Koepfen gehoert die Flaeche den Kindern der Seite --
            // das Reiterwerk liegt in der Reihenfolge VOR ihnen und schluckte
            // sonst jeden Klick auf einen Knopf oder ein Feld darauf.
            Kind::TabControl => my >= ay + self.sk(TC_KOPF_H),
            _ => false,
        }
    }

    /// Die Koepfe eines Reiterwerks: (Seite, x links, x rechts), absolut.
    ///
    /// EINE Quelle fuer Zeichnen und Treffertest. Die Breite wird an der
    /// ZEICHENZAHL geschaetzt, nicht gemessen -- der Treffertest laeuft in
    /// `handle_press`, und dort gibt es keine Grafik. Eine zweite, genauere
    /// Rechnung beim Zeichnen waere der sicherste Weg, Klick und Beschriftung
    /// auseinander laufen zu lassen.
    pub(super) fn tc_koepfe(&self, ax: i32, wdg: &Widget) -> Vec<(usize, i32, i32)> {
        let mut x = ax;
        let mut out = Vec::new();
        let kreuz = if Self::tc_schliessbar(wdg) { self.sk(16) } else { 0 };
        for (i, t) in wdg.items.iter().enumerate() {
            let b = self.sk(8) * t.chars().count() as i32 + self.sk(20) + kreuz;
            out.push((i, x, x + b));
            x += b + 2;
        }
        out
    }
    /// Je Bild: die Beschriftungen der Werkzeugleisten messen. Erst messen,
    /// dann schreiben -- das Messen braucht `&self`.
    pub(super) fn leisten_pass(&mut self, g: &Graphics) {
        let mut messung: Vec<(usize, usize, usize, i32)> = Vec::new();
        let mut pfade: Vec<(usize, usize, Vec<i32>)> = Vec::new();
        for (wi, win) in self.windows.iter().enumerate() {
            if !win.alive { continue; }
            for (i, w) in win.widgets.iter().enumerate() {
                if let Some(p) = w.pfad.as_ref() {
                    pfade.push((wi, i, p.texte.iter().map(|t| self.wtext_width(g, w, t)).collect()));
                }
                if let Some(l) = w.leiste.as_ref() {
                    for (k, e) in l.eintraege.iter().enumerate() {
                        if !e.text.is_empty() {
                            messung.push((wi, i, k, self.wtext_width(g, w, &e.text)));
                        }
                    }
                }
            }
        }
        for (wi, i, b) in pfade {
            if let Some(p) = self.windows[wi].widgets[i].pfad.as_mut() { p.breiten = b; }
        }
        for (wi, i, k, b) in messung {
            if let Some(e) = self.windows[wi].widgets[i].leiste.as_mut().and_then(|l| l.eintraege.get_mut(k)) {
                e.text_b = b;
            }
        }
    }
    /// Je Bild: welches Kind auf welcher Seite liegt (wie panel_pass).
    pub(super) fn tc_schliessbar(wdg: &Widget) -> bool {
        wdg.kind == Kind::TabControl && wdg.tabctl.as_ref().map(|s| s.schliessbar).unwrap_or(false)
    }
    /// Das Kreuz eines Kopfes, der bei `x1` endet: (x, y, kante). EINE Quelle
    /// fuer Zeichnen und Treffertest, wie `tab_kreuz_rect` am Fenster.
    pub(super) fn tc_kreuz(&self, x1: i32, ay: i32) -> (i32, i32, i32) {
        let k = self.sk(9);
        (x1 - self.sk(8) - k, ay + 2 + (self.sk(TC_KOPF_H) - 2 - k) / 2, k)
    }
    /// GUI_TABCONTROL_SET(tc, schluessel$, wert): bisher nur "schliessbar".
    pub fn tabcontrol_set(&mut self, h: i64, schluessel: &str, wert: bool) -> Result<(), String> {
        let w = self.tc_mut(h, "GUI_TABCONTROL_SET")?;
        let st = w.tabctl.get_or_insert_with(Default::default);
        match schluessel.to_lowercase().as_str() {
            "schliessbar" | "closable" => { st.schliessbar = wert; Ok(()) }
            _ => Err(format!("GUI_TABCONTROL_SET: unbekannter Schluessel '{}' (bekannt: schliessbar)", schluessel)),
        }
    }
    /// GUI_TABCONTROL_CLOSED(tc): welche Seite in diesem Bild zu soll
    /// (-1 = keine). Kreuz oder mittlere Maustaste.
    pub fn tabcontrol_closed(&self, h: i64) -> Result<i64, String> {
        let w = self.tc_ref(h, "GUI_TABCONTROL_CLOSED")?;
        Ok(w.tabctl.as_ref().and_then(|s| s.zu).map(|k| k as i64).unwrap_or(-1))
    }
    pub(super) fn tabctl_pass(&mut self) {
        for wi in 0..self.windows.len() {
            let n = self.windows[wi].widgets.len();
            for i in 0..n { self.windows[wi].widgets[i].tc_von = -1; self.windows[wi].widgets[i].tc_seite = -1; }
            for p in 0..n {
                let kinder = match self.windows[wi].widgets[p].tabctl.as_ref() {
                    Some(st) => st.kinder.clone(),
                    None => continue,
                };
                for (k, seite) in kinder {
                    // Durch Behaelter hindurch, damit ein Layout auf einer
                    // Seite seine Kinder mitnimmt -- dieselbe Sammlung wie
                    // beim rollenden Panel.
                    let mut menge = Vec::new();
                    self.panel_sammeln(wi, k, &mut menge, 0);
                    for &m in &menge {
                        if m == p { continue; }
                        let w = &mut self.windows[wi].widgets[m];
                        w.tc_von = p as i32; w.tc_seite = seite;
                    }
                }
            }
        }
    }

    /// Die Zeilen, wie sie GEZEICHNET werden: (Knoten, Hoehenanteil 0..1).
    /// Der Anteil ist das Produkt der Oeffnungsgrade aller Vorfahren -- beim
    /// Aufklappen wachsen die Kinder aus ihrer Elternzeile heraus, beim
    /// Zuklappen schrumpfen sie hinein. Ohne laufenden Uebergang ist das genau
    /// `tree_visible` mit lauter 1.
    pub(super) fn tree_zeilen_weich(t: &TreeState) -> Vec<(usize, f64)> {
        let mut out = Vec::new();
        Self::tree_sammeln_weich(t, -1, 1.0, &mut out);
        out
    }
    pub(super) fn tree_sammeln_weich(t: &TreeState, parent: i32, faktor: f64, out: &mut Vec<(usize, f64)>) {
        for i in 0..t.nodes.len() {
            if t.nodes[i].parent != parent { continue; }
            out.push((i, faktor));
            let n = &t.nodes[i];
            let a = t.auf_t.get(i).copied().unwrap_or(if n.expanded { 1.0 } else { 0.0 });
            if n.has_children && a > 0.0 {
                Self::tree_sammeln_weich(t, i as i32, faktor * weich(a), out);
            }
        }
    }

    /// Sichtbare Knoten (id-Liste) in Anzeigereihenfolge: Vorfahren-expanded.
    pub(super) fn tree_visible(t: &TreeState) -> Vec<usize> {
        let mut out = Vec::new();
        Self::tree_collect(t, -1, &mut out);
        out
    }
    pub(super) fn tree_collect(t: &TreeState, parent: i32, out: &mut Vec<usize>) {
        for i in 0..t.nodes.len() {
            if t.nodes[i].parent == parent {
                out.push(i);
                if t.nodes[i].expanded && t.nodes[i].has_children {
                    Self::tree_collect(t, i as i32, out);
                }
            }
        }
    }

    pub(super) fn tree_hover(&mut self, wi: usize, idx: usize, my: i32, g: &mut Graphics) {
        let (_ax, ay, _w, h) = self.abs_rect(wi, &self.windows[wi].widgets[idx]);
        let vis = Self::tree_visible(self.windows[wi].widgets[idx].tree.as_ref().unwrap());
        let zeile_h = self.sk(TREE_ROW_H);   // vor dem mut-Zugriff auf den Baum
        let max_scroll = (vis.len() as i32 * zeile_h - (h - 2)).max(0);
        let wheel = Self::rad(g);
        let weich_an = self.m("uebergang") > 0;
        let w = &mut self.windows[wi].widgets[idx];
        let t = w.tree.as_mut().unwrap();
        t.scroll = t.scroll.clamp(0, max_scroll);
        if wheel != 0 {
            // Weich wie bei Liste und Tabelle: Radschritte addieren sich aufs Ziel.
            let basis = w.rad_ziel.unwrap_or(t.scroll as f64);
            let ziel = (basis as i32 - wheel as i32 * zeile_h).clamp(0, max_scroll);
            if weich_an { w.rad_ziel = Some(ziel as f64); w.rad_letzt = t.scroll as f64; }
            else { t.scroll = ziel; }
        }
        let row = (my - (ay + 1) + t.scroll) / zeile_h;
        t.hover = if row >= 0 && (row as usize) < vis.len() { vis[row as usize] as i32 } else { -1 };
    }

    /// GUI_TREE_ITEM_RECT: wo ein Knoten auf dem Bildschirm steht (x, y, b, h),
    /// in Bildschirmpunkten wie GUI_HIT_TEST -- x ist der Anfang seiner
    /// Beschriftung (hinter Einzug und Dreieck). Aus derselben Rechnung wie
    /// der Klick (`tree_press`). Ein Knoten unter einem zugeklappten liefert
    /// (0, 0, 0, 0). Gebraucht fuer Pruefungen mit echten Klicks.
    pub fn tree_item_rect(&self, h: i64, node: i64) -> Result<(i32, i32, i32, i32), String> {
        let w = self.wdg(h, "GUI_TREE_ITEM_RECT")?;
        let t = match w.tree.as_ref() { Some(t) if w.kind == Kind::Tree => t, _ => return Err("GUI_TREE_ITEM_RECT: Widget ist kein Baum".into()) };
        if node < 0 || node as usize >= t.nodes.len() {
            return Err(format!("GUI_TREE_ITEM_RECT: Knoten {} ausserhalb [0..{}]", node, t.nodes.len() as i64 - 1));
        }
        let (wi, _) = Self::dec_widget(h);
        let vis = Self::tree_visible(t);
        let row = match vis.iter().position(|&n| n == node as usize) { Some(r) => r as i32, None => return Ok((0, 0, 0, 0)) };
        let (ax, ay, aw, _) = self.abs_rect(wi, w);
        let rh = self.sk(TREE_ROW_H);
        let x = ax + 4 + self.tree_kast_w(t) + t.nodes[node as usize].level * self.sk(TREE_INDENT) + self.sk(TREE_TOGGLE_W);
        Ok((x, ay + 1 + row * rh - t.scroll, (aw - (x - ax)).max(0), rh))
    }
    pub(super) fn tree_press(&mut self, wi: usize, idx: usize, mx: i32, my: i32) {
        let (ax, ay, _w, _h) = self.abs_rect(wi, &self.windows[wi].widgets[idx]);
        let t = self.windows[wi].widgets[idx].tree.as_ref().unwrap();
        let vis = Self::tree_visible(t);
        let row = (my - (ay + 1) + t.scroll) / self.sk(TREE_ROW_H);
        if row < 0 || row as usize >= vis.len() { return; }
        let ni = vis[row as usize];
        let level = t.nodes[ni].level;
        let has_children = t.nodes[ni].has_children;
        let kast_w = self.tree_kast_w(t);
        let toggle_x = ax + 4 + kast_w + level * self.sk(TREE_INDENT);
        let on_toggle = has_children && mx >= toggle_x && mx < toggle_x + self.sk(TREE_TOGGLE_W);
        let kaestchen = t.kaestchen;
        let (kx, ks) = self.tree_kast(ax, level);
        // Ein Klick aufs Kaestchen kippt NUR den Haken -- die Auswahl bleibt,
        // wie bei der Liste; sonst waehlte man beim Abhaken jedes Mal um.
        let auf_kasten = kaestchen && !on_toggle && mx >= kx && mx < kx + ks;
        let dbl = self.dbl_click;
        if auf_kasten {
            let t = self.windows[wi].widgets[idx].tree.as_mut().unwrap();
            t.sync();
            let v = !t.checks[ni];
            t.checks[ni] = v;
            Self::ft_haken_merken(t, ni, v);
            let f = self.windows[wi].widgets[idx].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
            return;
        }
        let klick_klappt = t.datei.as_ref().map(|d| d.klick_klappt).unwrap_or(false);
        if on_toggle || (has_children && (dbl || klick_klappt)) {
            let t = self.windows[wi].widgets[idx].tree.as_mut().unwrap();
            // Ein Doppelklick klappt einen Ast um UND meldet sich -- ein
            // Programm, das Aeste "oeffnet", soll es erfahren.
            if dbl && !on_toggle { t.doppel = true; }
            let e = t.nodes[ni].expanded; t.nodes[ni].expanded = !e;
            // Beim Dateibaum entscheidet der WEG, was offen ist -- die Nummern
            // gelten nur bis zum naechsten Neuaufbau, und der kommt sofort.
            // Die Auswahl wird VOR dem Neuaufbau gesetzt, damit sie ihn
            // ueberlebt (sie wird dabei in einen Weg uebersetzt).
            if t.datei.is_some() {
                let weg = t.datei.as_ref().unwrap().pfade.get(ni).cloned();
                if let Some(w) = weg {
                    if klick_klappt && !on_toggle { t.selected = ni as i32; }
                    Self::ft_offen_setzen(t, &w, !e);
                    dateibaum_neu(t);
                }
            }
        } else {
            let (ctrl, shift) = self.tasten_mod;
            let changed = {
                let t = self.windows[wi].widgets[idx].tree.as_mut().unwrap();
                let mut ge = t.selected != ni as i32;
                t.selected = ni as i32;
                if t.multi {
                    t.sync();
                    if ctrl {
                        let v = t.sel[ni]; t.sel[ni] = !v; t.anker = ni as i32; ge = true;
                    } else if shift && t.anker >= 0 {
                        // Der Bereich gilt in der SICHTBAREN Reihenfolge -- ueber
                        // die Knoten-Nummern traefe er bei zugeklappten Aesten
                        // etwas anderes, als man vor sich sieht.
                        let vis = Self::tree_visible(t);
                        let pa = vis.iter().position(|&n| n as i32 == t.anker);
                        let pb = vis.iter().position(|&n| n == ni);
                        if let (Some(pa), Some(pb)) = (pa, pb) {
                            for s in t.sel.iter_mut() { *s = false; }
                            for p in pa.min(pb)..=pa.max(pb) { t.sel[vis[p]] = true; }
                            ge = true;
                        }
                    } else {
                        for s in t.sel.iter_mut() { *s = false; }
                        t.sel[ni] = true; t.anker = ni as i32; ge = true;
                    }
                }
                ge
            };
            // Doppelklick auf eine DATEI ist das Oeffnen -- ein Ereignis, kein
            // Zustand: es steht ein Bild lang und wird dann zurueckgesetzt.
            if dbl {
                let t = self.windows[wi].widgets[idx].tree.as_mut().unwrap();
                t.doppel = true;
                if let Some(d) = t.datei.as_mut() {
                    if let Some(w) = d.pfade.get(ni).cloned() { d.aktiviert = w; }
                }
            }
            if changed {
                let f = self.windows[wi].widgets[idx].on_change.clone();
                if let Some(f) = f { self.pending.push(f); }
            }
        }
    }

    // --- Uhrzeit (GUI_TIMEPICKER) -------------------------------------------
    /// Das Gegenstueck zum Datumswaehler. Drei Felder mit Pfeilen: Stunde,
    /// Minute und (auf Wunsch) Sekunde.
    ///
    /// **Nach aussen gilt EIN Format, `HH:MM:SS` wie `TIME$()`** -- auch wenn
    /// die Sekunden gar nicht dastehen. Zwei Formate waeren dieselbe
    /// Stolperfalle, die beim Datum vermieden wurde; hinein darf `HH:MM`.
    pub fn timepicker(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::TimePicker, x, y, self.tp_breite(w, false), if h > 0 { h } else { self.sk(34) });
        wd.zeit = [12, 0, 0];
        wd.sel = 1;   // die Minute ist das, was man meistens dreht
        wd.step = 1.0;
        self.add_widget(win, "GUI_TIMEPICKER", wd)
    }
    /// Breite, die drei bzw. zwei Felder brauchen (0 = Vorgabe).
    pub(super) fn tp_breite(&self, w: i32, sek: bool) -> i32 {
        if w > 0 { return w; }
        self.sk(if sek { 150 } else { 104 })
    }
    pub(super) fn tp_mut(&mut self, h: i64, fn_: &str) -> Result<&mut Widget, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::TimePicker { return Err(format!("{}: Widget ist kein Uhrzeitwaehler (GUI_TIMEPICKER)", fn_)); }
        Ok(w)
    }
    pub fn time_get(&self, h: i64) -> Result<String, String> {
        let w = self.wdg(h, "GUI_TIME")?;
        if w.kind != Kind::TimePicker { return Err("GUI_TIME: Widget ist kein Uhrzeitwaehler (GUI_TIMEPICKER)".into()); }
        Ok(format!("{:02}:{:02}:{:02}", w.zeit[0], w.zeit[1], w.zeit[2]))
    }
    pub fn set_time(&mut self, h: i64, t: &str) -> Result<(), String> {
        let teile: Vec<&str> = t.trim().split(':').collect();
        if teile.len() < 2 || teile.len() > 3 {
            return Err(format!("GUI_SET_TIME: '{}' ist keine Uhrzeit (HH:MM oder HH:MM:SS)", t));
        }
        let mut z = [0i32; 3];
        for (k, teil) in teile.iter().enumerate() {
            z[k] = teil.trim().parse::<i32>()
                .map_err(|_| format!("GUI_SET_TIME: '{}' ist keine Uhrzeit (HH:MM oder HH:MM:SS)", t))?;
        }
        if z[0] > 23 || z[1] > 59 || z[2] > 59 || z.iter().any(|&x| x < 0) {
            return Err(format!("GUI_SET_TIME: '{}' liegt ausserhalb von 00:00:00 bis 23:59:59", t));
        }
        self.tp_mut(h, "GUI_SET_TIME")?.zeit = z;
        Ok(())
    }
    pub fn timepicker_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let breite_vorgabe = self.tp_breite(0, false);
        let breite_sek = self.tp_breite(0, true);
        let w = self.tp_mut(h, "GUI_TIMEPICKER_SET")?;
        match key.to_lowercase().as_str() {
            "sekunden" | "seconds" => {
                let an = wert != 0.0;
                // Die Breite wandert mit, solange sie die Vorgabe ist -- wer
                // sie selbst gesetzt hat, behaelt seine.
                if w.zeit_sek != an && (w.w == breite_vorgabe || w.w == breite_sek) {
                    w.w = if an { breite_sek } else { breite_vorgabe };
                    w.bw = w.w; w.nat_w = w.w;
                }
                w.zeit_sek = an;
            }
            // Schrittweite der MINUTE (5 heisst: in Fuenferschritten).
            "schritt" | "step" => w.step = wert.max(1.0),
            _ => return Err(format!(
                "GUI_TIMEPICKER_SET: unbekannte Einstellung '{}' (gueltig: sekunden, schritt)", key)),
        }
        Ok(())
    }
    /// Lage der Felder: (x des ersten, Breite eines Feldes, Zahl der Felder).
    /// EINE Quelle fuer Zeichnen und Treffertest.
    pub(super) fn tp_geom(&self, w: &Widget, ax: i32, breite: i32) -> (i32, i32, usize) {
        let n = if w.zeit_sek { 3 } else { 2 };
        let luecke = self.sk(10);
        let fw = ((breite - luecke * (n as i32 - 1)) / n as i32).max(self.sk(24));
        (ax, fw + luecke, n)
    }
    /// Einen Teil der Uhrzeit verstellen (mit Ueberlauf in den naechsten).
    pub(super) fn tp_dreh(&mut self, wi: usize, i: usize, teil: usize, richtung: i32) {
        let w = &mut self.windows[wi].widgets[i];
        let schritt = if teil == 1 { w.step.max(1.0) as i32 } else { 1 };
        let grenze = if teil == 0 { 24 } else { 60 };
        let neu = w.zeit[teil] + richtung * schritt;
        // Umlaufen statt anschlagen: wer von 00 aus rueckwaerts dreht, will
        // 23 sehen und nicht dasselbe noch einmal.
        w.zeit[teil] = ((neu % grenze) + grenze) % grenze;
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }
    pub(super) fn tp_press(&mut self, wi: usize, i: usize, mx: i32, my: i32) {
        let (ax, ay, breite, h) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let (x0, schritt, n) = self.tp_geom(&self.windows[wi].widgets[i], ax, breite);
        let teil = (((mx - x0) / schritt.max(1)) as usize).min(n - 1);
        self.windows[wi].widgets[i].sel = teil as i32;
        // Obere Haelfte hoch, untere runter -- dieselbe Geste wie am Zahlenfeld.
        let hoch = my < ay + h / 2;
        self.tp_dreh(wi, i, teil, if hoch { 1 } else { -1 });
    }
}
