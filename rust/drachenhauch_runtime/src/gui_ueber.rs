//! Ueberlagerungen und Rueckmeldung (docs/entwurf-gui-wie-gtk.md, Paket 1):
//! Popover an einem Widget, Toasts, Infoleiste, Suchfeld und Sinnbilder im
//! Textfeld, Vorschlaege unter einem Textfeld.
//!
//! Alles hier liegt UEBER den Fenstern und wird in der oberen Schicht
//! gezeichnet (`draw_top`), wie Kontextmenue und Tooltip -- sonst laege es
//! hinter dem, was ein Programm in eine Zeichenflaeche malt. Ein Druck geht
//! zuerst durch `ueber_druck`: was hier getroffen wird (ein Toast, ein Knopf
//! der Infoleiste, ein Vorschlag, ein Sinnbild) oder was einen Popover
//! schliesst, kommt bei keinem Widget mehr an.
//!
//! Gemessen und gelegt wird an EINER Stelle je Teil (`pop_lage`,
//! `toast_lage`, `info_lage`, `vorschlag_rect`, `ti_rand`), die Treffertest
//! und Zeichnen gemeinsam fragen.
use super::*;

/// So weit steht ein Popover vom Widget ab -- die Spitze seines Pfeils.
const POP_PFEIL: i32 = 8;
/// Breite eines Sinnbild-Platzes im Textfeld.
const FELD_SYMBOL_B: i32 = 22;
/// So lange (Sekunden) wartet ein Suchfeld nach dem letzten Anschlag, bis es
/// GUI_SEARCH_CHANGED meldet -- wie GtkSearchEntry.
const SUCHE_PAUSE: f64 = 0.15;
/// Hoechstens so viele Vorschlaege stehen auf einmal da.
const VORSCHLAG_ZEILEN: i32 = 8;
/// Hoechstens so viele Toasts stehen auf einmal in einem Fenster.
const TOAST_MAX: usize = 4;

pub(super) const UEBER_ARTEN: &[&str] = &["info", "erfolg", "warnung", "fehler"];

fn art_nr(s: &str, f: &str) -> Result<u8, String> {
    match s.to_lowercase().as_str() {
        "" | "info" => Ok(0),
        "erfolg" | "success" | "ok" => Ok(1),
        "warnung" | "warning" => Ok(2),
        "fehler" | "error" => Ok(3),
        _ => Err(format!("{}: Art '{}' gibt es nicht -- bekannt sind: {}", f, s, UEBER_ARTEN.join(", "))),
    }
}

/// Woran ein Popover haengt (`Window::pop`).
#[derive(Clone)]
pub(super) struct PopAnker {
    /// Das Widget, an dem er haengt (Widget-Handle).
    pub(super) anker: i64,
    /// Gewuenschte Seite: 0 unten, 1 oben, 2 rechts, 3 links.
    pub(super) seite: u8,
    /// Wo er tatsaechlich steht (weicht am Bildschirmrand aus).
    pub(super) nach: u8,
    /// Lage der Pfeilspitze entlang der Kante (x bei oben/unten, y sonst).
    pub(super) pfeil: i32,
    /// Durch einen Klick daneben oder ESC geschlossen -- EIN Bild lang.
    pub(super) zu: bool,
}

/// Eine Meldung unten im Fenster (GUI_TOAST).
#[derive(Clone)]
pub(super) struct Toast {
    text: String,
    art: u8,
    /// Restzeit in Sekunden; < 0 = bleibt, bis man klickt.
    rest: f64,
    /// Einblendung 0..1.
    t: f32,
    weg: bool,
    /// Lage, in GUI_UPDATE gelegt (Treffertest und Zeichnen).
    rect: (i32, i32, i32, i32),
}

/// Die Leiste oben im Inhalt eines Fensters (GUI_INFOBAR).
#[derive(Clone)]
pub(super) struct InfoLeiste {
    text: String,
    art: u8,
    knoepfe: Vec<String>,
    /// Die Antwort -- EIN Bild lang: Knopfnummer ab 1, -1 = das Kreuz.
    antwort: i32,
    t: f32,
    weg: bool,
    /// Lage der Leiste, ihrer Knoepfe und des Kreuzes (in GUI_UPDATE gelegt).
    rect: (i32, i32, i32, i32),
    knopf_rects: Vec<(i32, i32, i32, i32)>,
    kreuz: (i32, i32, i32, i32),
}

/// Was ein Textfeld ueber das Gewoehnliche hinaus hat -- nur bei Bedarf
/// angelegt (`Widget::feld`).
#[derive(Clone, Default)]
pub(super) struct FeldZusatz {
    /// Sinnbild links und rechts ("" = keins).
    icon: [String; 2],
    /// In diesem Bild angeklickt: 1 links, 2 rechts, 0 keins.
    icon_klick: i32,
    /// Suchfeld: Lupe links, Kreuz rechts, Meldung nach einer Pause.
    suche: bool,
    suche_rest: f64,
    suche_neu: bool,
    /// Vorschlaege (GUI_TEXTINPUT_COMPLETION) und die, die zum Text passen.
    vorschlaege: Vec<String>,
    treffer: Vec<usize>,
    offen: bool,
    mark: i32,
    scroll: i32,
    hover: i32,
    /// In diesem Bild uebernommen: Index in `vorschlaege`, -1 = keiner.
    gewaehlt: i32,
}

impl FeldZusatz {
    fn neu() -> FeldZusatz { FeldZusatz { suche_rest: -1.0, mark: -1, hover: -1, gewaehlt: -1, ..Default::default() } }
}

impl Gui {
    // ================================================================ Popover
    /// GUI_POPOVER(anker, b, h [, seite$]) -> GUI_WINDOW: ein Kasten ohne
    /// Rahmen mit Pfeil, an ein Widget gehaengt. Er beginnt unsichtbar.
    pub fn popover(&mut self, anker: i64, b: i32, h: i32, seite: &str) -> Result<i64, String> {
        let f = "GUI_POPOVER";
        self.wdg(anker, f)?;
        let s = match seite.to_lowercase().as_str() {
            "" | "unten" | "bottom" => 0, "oben" | "top" => 1,
            "rechts" | "right" => 2, "links" | "left" => 3,
            _ => return Err(format!("{}: Seite '{}' gibt es nicht -- unten, oben, rechts oder links", f, seite)),
        };
        if b < 8 || h < 8 { return Err(format!("{}: Breite und Hoehe muessen mindestens 8 sein", f)); }
        let fokus = self.focus_window;
        let h_win = self.new_window(String::new(), 0, 0, b, h);
        let wi = h_win as usize;
        let w = &mut self.windows[wi];
        w.chrome = false; w.movable = false; w.closable = false; w.visible = false;
        w.pop = Some(Box::new(PopAnker { anker, seite: s, nach: s, pfeil: 0, zu: false }));
        self.focus_window = fokus;
        Ok(h_win)
    }
    fn pop_von(&self, h: i64, f: &str) -> Result<usize, String> {
        let wi = h as usize;
        match self.windows.get(wi) {
            Some(w) if w.pop.is_some() => Ok(wi),
            Some(_) => Err(format!("{}: dieses Fenster ist kein Popover (GUI_POPOVER)", f)),
            None => Err(format!("{}: ungueltiges GUI_WINDOW-Handle", f)),
        }
    }
    /// GUI_POPOVER_SHOW: zeigen, an das Widget legen, nach vorn holen.
    pub fn popover_show(&mut self, h: i64) -> Result<(), String> {
        let wi = self.pop_von(h, "GUI_POPOVER_SHOW")?;
        if !self.windows[wi].alive { return Ok(()); }
        self.windows[wi].visible = true;
        self.windows[wi].close_clicked = false;
        self.pop_lage(wi);
        self.bring_to_front(wi);
        self.focus_window = Some(wi);
        Ok(())
    }
    /// GUI_POPOVER_HIDE: verbergen (ohne GUI_POPOVER_CLOSED -- das meldet nur,
    /// was der Nutzer geschlossen hat).
    pub fn popover_hide(&mut self, h: i64) -> Result<(), String> {
        self.pop_von(h, "GUI_POPOVER_HIDE")?;
        self.window_visible(h, false)
    }
    pub fn popover_closed(&self, h: i64) -> Result<bool, String> {
        let wi = self.pop_von(h, "GUI_POPOVER_CLOSED")?;
        Ok(self.windows[wi].pop.as_ref().map(|p| p.zu).unwrap_or(false))
    }

    /// Ist das Fenster ein sichtbarer Popover?
    pub(super) fn ist_offener_pop(&self, wi: usize) -> bool {
        self.windows.get(wi).map(|w| w.alive && w.visible && w.pop.is_some()).unwrap_or(false)
    }
    /// Popover gehoeren in der Fensterreihenfolge nach oben -- auch wenn ein
    /// Programm danach ein anderes Fenster per GUI_DRAW_WINDOW vorholt; sonst
    /// faenge das die Klicks, die im Popover landen sollen.
    pub(super) fn pops_nach_vorn(&mut self) {
        let (mut rest, pops): (Vec<usize>, Vec<usize>) =
            self.z_order.iter().partition(|&&wi| !self.ist_offener_pop(wi));
        if pops.is_empty() { return; }
        rest.extend(pops);
        self.z_order = rest;
    }

    /// Den Popover an sein Widget legen; weicht am Bildschirmrand aus. Ist
    /// das Widget nicht mehr zu sehen, geht der Popover zu.
    pub(super) fn pop_lage(&mut self, wi: usize) {
        let Some(p) = self.windows[wi].pop.as_ref().map(|p| (**p).clone()) else { return };
        let (aw, ai) = Self::dec_widget(p.anker);
        let sichtbar = self.windows.get(aw).map(|w| w.alive && w.visible).unwrap_or(false)
            && self.windows[aw].widgets.get(ai).map(|x| self.widget_shown(aw, x)).unwrap_or(false);
        if !sichtbar {
            self.windows[wi].visible = false;
            self.clear_window_interactions(wi);
            return;
        }
        let (rx, ry, rw, rh) = self.abs_rect(aw, &self.windows[aw].widgets[ai]);
        let (w, h) = (self.windows[wi].w, self.windows[wi].h);
        let pf = self.sk(POP_PFEIL);
        let (sb, sh) = (self.schirm_b, self.schirm_h);
        let passt_unten = sh <= 0 || ry + rh + pf + h <= sh;
        let passt_oben = ry - pf - h >= 0;
        let passt_rechts = sb <= 0 || rx + rw + pf + w <= sb;
        let passt_links = rx - pf - w >= 0;
        let nach = match p.seite {
            0 => if !passt_unten && passt_oben { 1 } else { 0 },
            1 => if !passt_oben && passt_unten { 0 } else { 1 },
            2 => if !passt_rechts && passt_links { 3 } else { 2 },
            _ => if !passt_links && passt_rechts { 2 } else { 3 },
        };
        let (mut x, mut y) = match nach {
            0 => (rx + rw / 2 - w / 2, ry + rh + pf),
            1 => (rx + rw / 2 - w / 2, ry - pf - h),
            2 => (rx + rw + pf, ry + rh / 2 - h / 2),
            _ => (rx - pf - w, ry + rh / 2 - h / 2),
        };
        let rand = self.sk(4);
        if sb > 0 { x = x.min(sb - w - rand).max(rand); }
        if sh > 0 { y = y.min(sh - h - rand).max(rand); }
        let rad = self.m("corner_radius").min(8);
        let pfeil = if nach <= 1 { (rx + rw / 2).clamp(x + rad + pf, (x + w - rad - pf).max(x + rad + pf)) }
                    else { (ry + rh / 2).clamp(y + rad + pf, (y + h - rad - pf).max(y + rad + pf)) };
        let win = &mut self.windows[wi];
        win.x = x; win.y = y;
        if let Some(pp) = win.pop.as_mut() { pp.nach = nach; pp.pfeil = pfeil; }
    }

    /// Haelt der Popover `wi` den Punkt -- er selbst, seine offene
    /// Klappliste oder ein Popover, der an einem seiner Widgets haengt?
    fn pop_haelt(&self, wi: usize, mx: i32, my: i32, tiefe: u32) -> bool {
        if tiefe > 8 { return false; }
        let w = &self.windows[wi];
        if Self::in_rect(mx, my, (w.x, w.y, w.w, w.h)) { return true; }
        if let Some((dw, di)) = self.open_dropdown {
            if dw == wi && Self::in_rect(mx, my, self.dropdown_popup_rect(dw, di)) { return true; }
        }
        (0..self.windows.len()).any(|q| q != wi && self.ist_offener_pop(q)
            && self.windows[q].pop.as_ref().map(|p| Self::dec_widget(p.anker).0 == wi).unwrap_or(false)
            && self.pop_haelt(q, mx, my, tiefe + 1))
    }

    // ================================================================ Toast
    pub fn toast(&mut self, win: i64, text: String, art: &str, ms: f64) -> Result<(), String> {
        let f = "GUI_TOAST";
        let art = art_nr(art, f)?;
        let w = self.win_mut(win, f)?;
        let offen = w.toasts.iter().filter(|t| !t.weg).count();
        if offen >= TOAST_MAX {
            if let Some(t) = w.toasts.iter_mut().find(|t| !t.weg) { t.weg = true; }
        }
        w.toasts.push(Toast { text: text.clone(), art, rest: if ms > 0.0 { ms / 1000.0 } else { -1.0 },
            t: 0.0, weg: false, rect: (0, 0, 0, 0) });
        // Ein Bildschirmleser liest die Meldung vor -- sonst stuende sie nur da.
        self.announce(text, art == 3);
        Ok(())
    }
    pub fn toast_count(&self, win: i64) -> Result<i64, String> {
        let w = self.windows.get(win as usize).ok_or("GUI_TOAST_COUNT: ungueltiges GUI_WINDOW-Handle")?;
        Ok(w.toasts.iter().filter(|t| !t.weg).count() as i64)
    }
    pub fn toast_clear(&mut self, win: i64) -> Result<(), String> {
        let w = self.win_mut(win, "GUI_TOAST_CLEAR")?;
        for t in w.toasts.iter_mut() { t.weg = true; }
        Ok(())
    }

    // ================================================================ Infoleiste
    pub fn infobar(&mut self, win: i64, text: String, art: &str, knoepfe: &str) -> Result<(), String> {
        let f = "GUI_INFOBAR";
        let art = art_nr(art, f)?;
        let knoepfe: Vec<String> = if knoepfe.trim().is_empty() { Vec::new() }
            else { knoepfe.split('|').map(|s| s.trim().to_string()).collect() };
        if knoepfe.len() > 4 { return Err(format!("{}: hoechstens vier Knoepfe", f)); }
        let w = self.win_mut(win, f)?;
        let t = w.info.as_ref().filter(|i| !i.weg).map(|i| i.t).unwrap_or(0.0);
        w.info = Some(Box::new(InfoLeiste { text: text.clone(), art, knoepfe, antwort: 0, t, weg: false,
            rect: (0, 0, 0, 0), knopf_rects: Vec::new(), kreuz: (0, 0, 0, 0) }));
        self.announce(text, art >= 2);
        Ok(())
    }
    pub fn infobar_hide(&mut self, win: i64) -> Result<(), String> {
        let w = self.win_mut(win, "GUI_INFOBAR_HIDE")?;
        if let Some(i) = w.info.as_mut() { i.weg = true; }
        Ok(())
    }
    pub fn infobar_answer(&self, win: i64) -> Result<i64, String> {
        let w = self.windows.get(win as usize).ok_or("GUI_INFOBAR_ANSWER: ungueltiges GUI_WINDOW-Handle")?;
        Ok(w.info.as_ref().map(|i| i.antwort as i64).unwrap_or(0))
    }
    /// GUI_INFOBAR_BUTTON_RECT(win, k) -> (x, y, b, h) eines Knopfs (ab 1;
    /// 0 = das Kreuz), logische Punkte; (-1, -1, 0, 0) ohne Leiste. Gelegt
    /// wird in GUI_UPDATE.
    pub fn infobar_button_rect(&self, win: i64, k: i64) -> Result<(i64, i64, i64, i64), String> {
        let w = self.windows.get(win as usize).ok_or("GUI_INFOBAR_BUTTON_RECT: ungueltiges GUI_WINDOW-Handle")?;
        let Some(i) = w.info.as_ref().filter(|i| !i.weg) else { return Ok((-1, -1, 0, 0)) };
        let r = if k == 0 { Some(i.kreuz) } else if k > 0 { i.knopf_rects.get(k as usize - 1).copied() } else { None };
        let Some((x, y, b, h)) = r else {
            return Err(format!("GUI_INFOBAR_BUTTON_RECT: die Leiste hat {} Knoepfe (1..{}, 0 = das Kreuz)", i.knoepfe.len(), i.knoepfe.len()));
        };
        Ok((self.unsk(x) as i64, self.unsk(y) as i64, self.unsk(b) as i64, self.unsk(h) as i64))
    }
    pub fn infobar_shown(&self, win: i64) -> Result<bool, String> {
        let w = self.windows.get(win as usize).ok_or("GUI_INFOBAR_SHOWN: ungueltiges GUI_WINDOW-Handle")?;
        Ok(w.info.as_ref().map(|i| !i.weg).unwrap_or(false))
    }

    // ================================================================ Textfeld
    fn feld_mut(&mut self, h: i64, f: &str) -> Result<&mut FeldZusatz, String> {
        let w = self.wdg_mut(h, f)?;
        if w.kind != Kind::TextInput { return Err(format!("{}: Widget ist kein Textfeld (GUI_TEXTINPUT)", f)); }
        Ok(w.feld.get_or_insert_with(|| Box::new(FeldZusatz::neu())))
    }
    /// GUI_TEXTINPUT_ICON(tf, links$, rechts$): ein Sinnbild links und rechts
    /// im Feld ("" = keins).
    pub fn textinput_icon(&mut self, h: i64, links: &str, rechts: &str) -> Result<(), String> {
        let f = "GUI_TEXTINPUT_ICON";
        for n in [links, rechts] {
            if !n.is_empty() && !SINNBILDER.contains(&n) {
                return Err(format!("{}: ein Sinnbild '{}' gibt es nicht -- bekannt sind: {}", f, n, SINNBILDER.join(", ")));
            }
        }
        let z = self.feld_mut(h, f)?;
        z.icon = [links.to_string(), rechts.to_string()];
        Ok(())
    }
    pub fn icon_clicked(&self, h: i64) -> Result<i64, String> {
        let w = self.wdg(h, "GUI_ICON_CLICKED")?;
        Ok(w.feld.as_ref().map(|z| z.icon_klick as i64).unwrap_or(0))
    }
    pub(super) fn feld_suche_setzen(&mut self, h: i64, an: bool) -> Result<(), String> {
        let z = self.feld_mut(h, "GUI_TEXTINPUT_SET")?;
        z.suche = an;
        Ok(())
    }
    pub fn search_changed(&self, h: i64) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_SEARCH_CHANGED")?;
        Ok(w.feld.as_ref().map(|z| z.suche_neu).unwrap_or(false))
    }
    /// GUI_TEXTINPUT_COMPLETION(tf, eintraege): Vorschlaege beim Tippen
    /// (leer = keine).
    pub fn textinput_completion(&mut self, h: i64, eintraege: Vec<String>) -> Result<(), String> {
        let z = self.feld_mut(h, "GUI_TEXTINPUT_COMPLETION")?;
        z.vorschlaege = eintraege;
        z.treffer.clear(); z.offen = false; z.mark = -1; z.scroll = 0;
        Ok(())
    }
    pub fn completion_chosen(&self, h: i64) -> Result<i64, String> {
        let w = self.wdg(h, "GUI_COMPLETION_CHOSEN")?;
        Ok(w.feld.as_ref().map(|z| z.gewaehlt as i64).unwrap_or(-1))
    }
    pub fn completion_open(&self, h: i64) -> Result<bool, String> {
        let w = self.wdg(h, "GUI_COMPLETION_OPEN")?;
        Ok(w.feld.as_ref().map(|z| z.offen).unwrap_or(false))
    }

    /// Wie viel Platz die Sinnbilder links und rechts im Textfeld nehmen --
    /// die EINE Stelle fuer Zeichnen, Klick, Schreibmarke und Rollen.
    pub(super) fn ti_rand(&self, w: &Widget) -> (i32, i32) {
        let Some(z) = w.feld.as_ref() else { return (0, 0) };
        let b = self.sk(FELD_SYMBOL_B);
        let l = if z.suche || !z.icon[0].is_empty() { b } else { 0 };
        let r = if z.suche || !z.icon[1].is_empty() { b } else { 0 };
        (l, r)
    }
    /// Das Sinnbild unter dem Punkt: 1 links, 2 rechts, 0 keins. Das Kreuz
    /// eines Suchfelds zaehlt nur, wenn Text darin steht (sonst ist es nicht
    /// zu sehen).
    pub(super) fn ti_icon_at(&self, wi: usize, i: usize, mx: i32, my: i32) -> i32 {
        let w = &self.windows[wi].widgets[i];
        if w.kind != Kind::TextInput { return 0; }
        let Some(z) = w.feld.as_ref() else { return 0 };
        let (ax, ay, fw, fh) = self.abs_rect(wi, w);
        if my < ay || my >= ay + fh { return 0; }
        let (l, r) = self.ti_rand(w);
        if l > 0 && mx >= ax && mx < ax + l { return 1; }
        if r > 0 && mx >= ax + fw - r && mx < ax + fw {
            if z.suche && z.icon[1].is_empty() && w.text.is_empty() { return 0; }
            return 2;
        }
        0
    }
    /// Was rechts im Feld steht: das eigene Sinnbild, sonst beim Suchfeld das
    /// Kreuz, solange Text darin ist.
    fn feld_symbol(w: &Widget, seite: usize) -> String {
        let Some(z) = w.feld.as_ref() else { return String::new() };
        if !z.icon[seite].is_empty() { return z.icon[seite].clone(); }
        if z.suche {
            if seite == 0 { return "suchen".into(); }
            if !w.text.is_empty() { return "schliessen".into(); }
        }
        String::new()
    }

    /// Ein Suchfeld leeren -- per Kreuz oder ESC. Meldet sofort, ohne Pause.
    pub(super) fn suche_leeren(&mut self, wi: usize, i: usize) {
        let w = &mut self.windows[wi].widgets[i];
        if w.text.is_empty() || w.nur_lesen { return; }
        w.undo.push((w.text.clone(), w.caret, Vec::new()));
        w.redo.clear();
        w.text.clear(); w.caret = 0; w.sel_anchor = 0; w.scroll = 0;
        if let Some(z) = w.feld.as_mut() { z.suche_rest = -1.0; z.suche_neu = true; z.offen = false; z.treffer.clear(); }
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }

    /// Nach einer Aenderung durch die Bedienung (aus edit_textinput): die
    /// Pause des Suchfelds neu beginnen, die Vorschlaege neu filtern.
    pub(super) fn feld_getippt(&mut self, wi: usize, i: usize) {
        let w = &mut self.windows[wi].widgets[i];
        let text = w.text.to_lowercase();
        let Some(z) = w.feld.as_mut() else { return };
        if z.suche { z.suche_rest = SUCHE_PAUSE; }
        if z.vorschlaege.is_empty() { return; }
        z.treffer.clear();
        if !text.is_empty() {
            // Was so ANFAENGT zuerst, dann was es irgendwo enthaelt -- ein
            // Eintrag, der genau dasteht, braucht keinen Vorschlag mehr.
            let mut vorn = Vec::new();
            let mut drin = Vec::new();
            for (k, v) in z.vorschlaege.iter().enumerate() {
                let l = v.to_lowercase();
                if l == text { continue; }
                if l.starts_with(&text) { vorn.push(k); } else if l.contains(&text) { drin.push(k); }
            }
            z.treffer = vorn; z.treffer.extend(drin);
        }
        z.offen = !z.treffer.is_empty();
        z.mark = -1; z.scroll = 0; z.hover = -1;
    }

    /// Einen Vorschlag uebernehmen (Index in `treffer`).
    fn vorschlag_nehmen(&mut self, wi: usize, i: usize, k: i32) {
        let w = &mut self.windows[wi].widgets[i];
        let Some(z) = w.feld.as_mut() else { return };
        let Some(&v) = z.treffer.get(k.max(0) as usize).filter(|_| k >= 0) else { return };
        let neu = z.vorschlaege[v].clone();
        z.offen = false; z.gewaehlt = v as i32; z.mark = -1;
        if z.suche { z.suche_rest = -1.0; z.suche_neu = true; }
        w.undo.push((w.text.clone(), w.caret, Vec::new()));
        w.redo.clear();
        let n = neu.chars().count() as i32;
        w.text = neu; w.caret = n; w.sel_anchor = n;
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }

    /// Tasten fuer die offene Vorschlagsliste und das Suchfeld -- VOR dem
    /// gewoehnlichen Tippen. true = die Taste ist verbraucht.
    pub(super) fn feld_tasten(&mut self, wi: usize, i: usize, g: &Graphics) -> bool {
        let w = &self.windows[wi].widgets[i];
        let Some(z) = w.feld.as_ref() else { return false };
        let (offen, n, mark, suche, leer) = (z.offen, z.treffer.len() as i32, z.mark, z.suche, w.text.is_empty());
        if offen && n > 0 {
            let runter = g.key_pressed(KEY_DOWN);
            let hoch = g.key_pressed(KEY_UP);
            if runter || hoch {
                let z = self.windows[wi].widgets[i].feld.as_mut().unwrap();
                z.mark = if runter { (mark + 1).min(n - 1) } else { (mark - 1).max(-1) };
                let zeilen = VORSCHLAG_ZEILEN.min(n);
                if z.mark >= z.scroll + zeilen { z.scroll = z.mark - zeilen + 1; }
                if z.mark >= 0 && z.mark < z.scroll { z.scroll = z.mark; }
                return true;
            }
            if g.key_pressed(KEY_ENTER) && mark >= 0 {
                self.vorschlag_nehmen(wi, i, mark);
                self.liste_taste = true;
                return true;
            }
            if g.key_pressed(KEY_ESC) {
                self.windows[wi].widgets[i].feld.as_mut().unwrap().offen = false;
                self.liste_taste = true;
                return true;
            }
        }
        if suche && g.key_pressed(KEY_ESC) && !leer {
            self.suche_leeren(wi, i);
            self.liste_taste = true;
            return true;
        }
        if suche && g.key_pressed(KEY_ENTER) {
            // Enter wartet nicht auf die Pause.
            let z = self.windows[wi].widgets[i].feld.as_mut().unwrap();
            if z.suche_rest >= 0.0 { z.suche_rest = -1.0; z.suche_neu = true; }
        }
        false
    }

    /// Lage der Vorschlagsliste: (x, y, b, h, zeilen, zeilenhoehe). Unter
    /// dem Feld, nach oben, wenn unten kein Platz ist.
    pub(super) fn vorschlag_rect(&self, wi: usize, i: usize) -> Option<(i32, i32, i32, i32, i32, i32)> {
        let w = self.windows.get(wi)?.widgets.get(i)?;
        let z = w.feld.as_ref()?;
        if !z.offen || z.treffer.is_empty() || !self.widget_shown(wi, w) { return None; }
        let (ax, ay, fw, fh) = self.abs_rect(wi, w);
        let ih = self.sk(DROPDOWN_ITEM_H);
        let zeilen = VORSCHLAG_ZEILEN.min(z.treffer.len() as i32);
        let h = zeilen * ih + 2;
        let y = if self.schirm_h > 0 && ay + fh + h > self.schirm_h && ay - h >= 0 { ay - h } else { ay + fh };
        Some((ax, y, fw.max(self.sk(120)), h, zeilen, ih))
    }

    // ================================================================ je Bild
    /// Am Anfang von GUI_UPDATE: Ereignisse des letzten Bildes vergessen.
    pub(super) fn ueber_zuruecksetzen(&mut self) {
        for win in self.windows.iter_mut() {
            if let Some(p) = win.pop.as_mut() { p.zu = false; }
            if let Some(i) = win.info.as_mut() { i.antwort = 0; }
            for w in win.widgets.iter_mut() {
                if let Some(z) = w.feld.as_mut() { z.icon_klick = 0; z.suche_neu = false; z.gewaehlt = -1; }
            }
        }
    }

    /// Je Bild: Zeit weiterzaehlen, Popover an ihr Widget legen, Toasts und
    /// Infoleisten messen.
    pub(super) fn ueber_pass(&mut self, g: &Graphics, mx: i32, my: i32) {
        let dt = g.delta();
        let ue = self.m("uebergang").max(0) as f64 / 1000.0;
        let schritt = |t: f32, auf: bool| -> f32 {
            if ue <= 0.0 { return if auf { 1.0 } else { 0.0 }; }
            let d = (dt / ue) as f32;
            if auf { (t + d).min(1.0) } else { (t - d).max(0.0) }
        };
        for wi in 0..self.windows.len() {
            if self.ist_offener_pop(wi) { self.pop_lage(wi); }
            let win = &mut self.windows[wi];
            for t in win.toasts.iter_mut() {
                if !t.weg && t.rest >= 0.0 {
                    t.rest -= dt;
                    if t.rest <= 0.0 { t.weg = true; }
                }
                t.t = schritt(t.t, !t.weg);
            }
            win.toasts.retain(|t| !(t.weg && t.t <= 0.0));
            if let Some(i) = win.info.as_mut() {
                i.t = schritt(i.t, !i.weg);
                if i.weg && i.t <= 0.0 && i.antwort == 0 { win.info = None; }
            }
            for w in win.widgets.iter_mut() {
                if let Some(z) = w.feld.as_mut() {
                    if z.suche_rest >= 0.0 {
                        z.suche_rest -= dt;
                        // Mit Spielraum: 9 x 1/60 s ist in Kommazahlen nicht ganz 0,15.
                        if z.suche_rest <= 1e-6 { z.suche_rest = -1.0; z.suche_neu = true; }
                    }
                }
            }
            self.toast_lage(g, wi);
            self.info_lage(g, wi);
        }
        // Eine Vorschlagsliste ohne Fokus geht zu; die Maus darueber markiert.
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                if self.windows[wi].widgets[i].feld.as_ref().map(|z| z.offen).unwrap_or(false) {
                    if self.focus_widget != Some((wi, i)) {
                        self.windows[wi].widgets[i].feld.as_mut().unwrap().offen = false;
                        continue;
                    }
                    let hov = self.vorschlag_rect(wi, i).and_then(|(x, y, b, h, _, ih)| {
                        if Self::in_rect(mx, my, (x, y, b, h)) { Some((my - y - 1) / ih.max(1)) } else { None }
                    });
                    let z = self.windows[wi].widgets[i].feld.as_mut().unwrap();
                    z.hover = hov.map(|k| z.scroll + k).unwrap_or(-1);
                }
            }
        }
    }

    fn toast_lage(&mut self, g: &Graphics, wi: usize) {
        if self.windows[wi].toasts.is_empty() { return; }
        let (x, y, w, h) = { let win = &self.windows[wi]; (win.x, win.y, win.w, win.h) };
        let pad = self.sk(14);
        let th = self.ctext_height(g) + self.sk(14);
        let luecke = self.sk(8);
        let max_b = (w - self.sk(32)).max(self.sk(80));
        let mut unten = y + h - self.sk(16);
        let n = self.windows[wi].toasts.len();
        for k in (0..n).rev() {
            let text = self.windows[wi].toasts[k].text.clone();
            let b = (self.ctext_width(g, &text) + pad * 2 + self.sk(18)).min(max_b);
            let t = weich(self.windows[wi].toasts[k].t);
            let versatz = ((1.0 - t) * self.sk(12) as f64) as i32;
            let r = (x + (w - b) / 2, unten - th + versatz, b, th);
            self.windows[wi].toasts[k].rect = r;
            unten -= ((th + luecke) as f64 * t) as i32;
        }
    }

    fn info_lage(&mut self, g: &Graphics, wi: usize) {
        if self.windows[wi].info.is_none() { return; }
        let (x, y, w) = { let win = &self.windows[wi]; (win.x, win.y, win.w) };
        let top = y + self.content_top(wi);
        let h = self.ctext_height(g) + self.sk(18);
        let pad = self.m("pad").max(self.sk(6));
        let kreuz_b = self.sk(24);
        let mut rx = x + w - pad - kreuz_b;
        let kreuz = (rx, top + (h - kreuz_b) / 2, kreuz_b, kreuz_b);
        let knoepfe = self.windows[wi].info.as_ref().unwrap().knoepfe.clone();
        let mut rects = vec![(0, 0, 0, 0); knoepfe.len()];
        let kh = h - self.sk(10);
        for (k, t) in knoepfe.iter().enumerate().rev() {
            let b = self.ctext_width(g, t) + self.sk(20);
            rx -= b + self.sk(6);
            rects[k] = (rx, top + (h - kh) / 2, b, kh);
        }
        let i = self.windows[wi].info.as_mut().unwrap();
        i.rect = (x + 1, top, w - 2, h);
        i.knopf_rects = rects;
        i.kreuz = kreuz;
    }

    /// Ein neuer Druck (links oder rechts) geht zuerst hierher. true = er
    /// ist verbraucht und kommt bei keinem Widget an.
    pub(super) fn ueber_druck(&mut self, mx: i32, my: i32, links: bool) -> bool {
        // Vorschlagsliste: ein Klick darauf uebernimmt, daneben schliesst sie.
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                if let Some((x, y, b, h, _, ih)) = self.vorschlag_rect(wi, i) {
                    if Self::in_rect(mx, my, (x, y, b, h)) {
                        if links {
                            let scroll = self.windows[wi].widgets[i].feld.as_ref().map(|z| z.scroll).unwrap_or(0);
                            self.vorschlag_nehmen(wi, i, scroll + (my - y - 1) / ih.max(1));
                        }
                        return true;
                    }
                    let (fx, fy, fw, fh) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
                    if !Self::in_rect(mx, my, (fx, fy, fw, fh)) {
                        self.windows[wi].widgets[i].feld.as_mut().unwrap().offen = false;
                    }
                }
            }
        }
        // Popover: ein Druck daneben schliesst ihn (und tut sonst nichts --
        // auch auf dem Widget, an dem er haengt, sonst ginge er im selben
        // Klick wieder auf).
        let offen: Vec<usize> = (0..self.windows.len()).filter(|&wi| self.ist_offener_pop(wi)).collect();
        let mut zu = false;
        for &wi in &offen {
            if !self.pop_haelt(wi, mx, my, 0) {
                self.windows[wi].visible = false;
                self.clear_window_interactions(wi);
                if let Some(p) = self.windows[wi].pop.as_mut() { p.zu = true; }
                zu = true;
            }
        }
        if zu { return true; }
        // Infoleiste: Knoepfe und Kreuz; die Leiste selbst nimmt jeden Druck.
        for &wi in self.z_order.clone().iter().rev() {
            if !self.windows[wi].alive || !self.windows[wi].visible { continue; }
            if self.modal.is_some_and(|m| m != wi) { continue; }
            let Some(i) = self.windows[wi].info.as_ref().filter(|i| !i.weg) else { continue };
            if !Self::in_rect(mx, my, i.rect) { continue; }
            if links {
                let k = i.knopf_rects.iter().position(|r| Self::in_rect(mx, my, *r));
                let kreuz = Self::in_rect(mx, my, i.kreuz);
                let i = self.windows[wi].info.as_mut().unwrap();
                if let Some(k) = k { i.antwort = k as i32 + 1; i.weg = true; }
                else if kreuz { i.antwort = -1; i.weg = true; }
            }
            return true;
        }
        // Toasts: ein Klick nimmt ihn weg.
        for &wi in self.z_order.clone().iter().rev() {
            if !self.windows[wi].alive || !self.windows[wi].visible { continue; }
            if let Some(t) = self.windows[wi].toasts.iter_mut().rev().find(|t| !t.weg && Self::in_rect(mx, my, t.rect)) {
                if links { t.weg = true; }
                return true;
            }
        }
        // Sinnbild im Textfeld.
        if links {
            if let Some(wi) = self.topmost_at(mx, my) {
                if self.windows[wi].entwurf || self.modal.is_some_and(|m| m != wi) { return false; }
                for i in 0..self.windows[wi].widgets.len() {
                    let ok = { let w = &self.windows[wi].widgets[i];
                        w.feld.is_some() && w.enabled && self.widget_shown(wi, w) && self.im_panel_sichtbar(wi, w, mx, my) };
                    if !ok { continue; }
                    let s = self.ti_icon_at(wi, i, mx, my);
                    if s == 0 { continue; }
                    let (suche, eigen) = { let z = self.windows[wi].widgets[i].feld.as_ref().unwrap(); (z.suche, z.icon[1].clone()) };
                    if s == 2 && suche && eigen.is_empty() { self.suche_leeren(wi, i); }
                    self.windows[wi].widgets[i].feld.as_mut().unwrap().icon_klick = s;
                    self.focus_window = Some(wi);
                    self.focus_widget = Some((wi, i));
                    return true;
                }
            }
        }
        false
    }

    /// ESC schliesst den obersten Popover. true = verbraucht.
    pub(super) fn pop_esc(&mut self) -> bool {
        let Some(wi) = self.z_order.iter().rev().copied().find(|&wi| self.ist_offener_pop(wi)) else { return false };
        self.windows[wi].visible = false;
        self.clear_window_interactions(wi);
        if let Some(p) = self.windows[wi].pop.as_mut() { p.zu = true; }
        true
    }

    // ================================================================ Zeichnen
    /// Farbe einer Meldung nach ihrer Art: (Grund, Rand, Text).
    fn art_farben(&self, art: u8) -> (i64, i64, i64) {
        let grund = match art { 1 => 0x2E9E5B, 2 => 0xD98E1F, 3 => 0xC94A3F, _ => self.th("widget_bg") };
        let rand = if art == 0 { self.th("accent") } else { shade(grund, -30) };
        let text = if art == 0 { self.th("text_fg") } else { lesbar_auf(grund) };
        (grund, rand, text)
    }
    /// Text, der nicht passt, endet mit "..." statt mitten im Buchstaben.
    fn kuerzen(&self, g: &Graphics, text: &str, breite: i32) -> String {
        if self.ctext_width(g, text) <= breite { return text.to_string(); }
        let z: Vec<char> = text.chars().collect();
        for n in (0..z.len()).rev() {
            let probe: String = z[..n].iter().collect::<String>() + "...";
            if self.ctext_width(g, &probe) <= breite { return probe; }
        }
        String::new()
    }
    fn art_sinnbild(art: u8) -> &'static str {
        match art { 1 => "pruefen", 2 | 3 => "warnung", _ => "info" }
    }

    /// Popover, Infoleisten, Toasts und Vorschlagslisten -- in der oberen
    /// Schicht, vor Kontextmenue und Tooltip.
    pub(super) fn ueber_zeichnen(&self, g: &mut Graphics) {
        for &wi in self.z_order.iter() {
            if self.ist_offener_pop(wi) { self.pop_zeichnen(g, wi); }
        }
        for &wi in self.z_order.iter() {
            let win = &self.windows[wi];
            if !win.alive || !win.visible { continue; }
            if let Some(i) = win.info.as_ref() { self.info_zeichnen(g, i); }
            for t in win.toasts.iter() { self.toast_zeichnen(g, t); }
        }
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                if self.windows[wi].widgets[i].feld.as_ref().map(|z| z.offen).unwrap_or(false) {
                    self.vorschlag_zeichnen(g, wi, i);
                }
            }
        }
    }

    pub(super) fn pop_zeichnen(&self, g: &mut Graphics, wi: usize) {
        let Some(p) = self.windows[wi].pop.as_ref() else { return };
        let (x, y, w, h) = { let win = &self.windows[wi]; (win.x, win.y, win.w, win.h) };
        let rad = self.m("corner_radius").min(8);
        // Schatten, dann das Fenster selbst (ohne Rahmen), dann Rand und Pfeil.
        for k in 0..3 {
            let off = 2 + k * 2;
            let alpha = (0x30 - k * 0x0E).max(0x0C) as i64;
            g.round_rect(x - k + 1, y + off, x + w - 1 + k + 1, y + h - 1 + off + k, (rad + k).max(0), alpha << 24, true);
        }
        self.draw_window_sicht(g, wi);
        let bg = self.th("win_bg");
        let rand = self.th("win_border");
        if rad > 0 { g.round_rect(x, y, x + w - 1, y + h - 1, rad, rand, false); }
        else { g.rect(x, y, x + w - 1, y + h - 1, rand); }
        let pf = self.sk(POP_PFEIL);
        let c = p.pfeil;
        // Spitze, zwei Ecken an der Kante (einen Punkt hinein, damit der
        // Rahmen dort verschwindet).
        let (s, a, b) = match p.nach {
            0 => ((c, y - pf), (c - pf, y + 1), (c + pf, y + 1)),
            1 => ((c, y + h - 1 + pf), (c - pf, y + h - 2), (c + pf, y + h - 2)),
            2 => ((x - pf, c), (x + 1, c - pf), (x + 1, c + pf)),
            _ => ((x + w - 1 + pf, c), (x + w - 2, c - pf), (x + w - 2, c + pf)),
        };
        g.triangle(s.0, s.1, a.0, a.1, b.0, b.1, bg);
        g.line(a.0, a.1, s.0, s.1, rand);
        g.line(s.0, s.1, b.0, b.1, rand);
    }

    fn toast_zeichnen(&self, g: &mut Graphics, t: &Toast) {
        let a = weich(t.t);
        if a <= 0.0 { return; }
        let (x, y, b, h) = t.rect;
        let (grund, rand, text) = self.art_farben(t.art);
        let rad = (h / 2).min(self.sk(12));
        g.round_rect(x + 1, y + 3, x + b, y + h + 2, rad, deckkraft((0x50i64) << 24, a), true);
        g.round_rect(x, y, x + b - 1, y + h - 1, rad, deckkraft(grund, a), true);
        g.round_rect(x, y, x + b - 1, y + h - 1, rad, deckkraft(rand, a), false);
        let s = self.sk(14);
        let pad = self.sk(12);
        sinnbild(g, Self::art_sinnbild(t.art), x + pad, y + (h - s) / 2, s, deckkraft(text, a), false);
        let tx = x + pad + s + self.sk(8);
        let platz = (x + b - pad - tx).max(1);
        let th = self.ctext_height(g);
        let s_text = self.kuerzen(g, &t.text, platz);
        self.ctext(g, tx, y + (h - th) / 2, s_text, deckkraft(text, a));
    }

    fn info_zeichnen(&self, g: &mut Graphics, i: &InfoLeiste) {
        let a = weich(i.t);
        if a <= 0.0 { return; }
        let (x, y, b, h) = i.rect;
        // Sie rollt von oben herab: sichtbar ist der untere Teil.
        let sichtbar = ((h as f64) * a).round() as i32;
        let dy = sichtbar - h;
        let (grund, rand, text) = self.art_farben(i.art);
        let grund = if i.art == 0 { mischen(self.th("widget_bg"), self.th("accent"), 0.18) } else { grund };
        g.push_clip(x, y, b, sichtbar.max(1));
        g.box_fill(x, y + dy, x + b - 1, y + dy + h - 1, grund);
        g.line(x, y + dy + h - 1, x + b - 1, y + dy + h - 1, rand);
        let s = self.sk(14);
        let pad = self.m("pad").max(self.sk(6));
        sinnbild(g, Self::art_sinnbild(i.art), x + pad, y + dy + (h - s) / 2, s, text, false);
        let tx = x + pad + s + self.sk(8);
        let rechts = i.knopf_rects.first().map(|r| r.0).unwrap_or(i.kreuz.0) - self.sk(8);
        let th = self.ctext_height(g);
        let s_text = self.kuerzen(g, &i.text, (rechts - tx).max(1));
        self.ctext(g, tx, y + dy + (h - th) / 2, s_text, text);
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        for (k, r) in i.knopf_rects.iter().enumerate() {
            let (kx, ky, kb, kh) = *r;
            let ueber = Self::in_rect(mx, my, *r);
            let kg = if ueber { shade(grund, 22) } else { shade(grund, 10) };
            let kr = self.m("corner_radius").min(6);
            g.round_rect(kx, ky + dy, kx + kb - 1, ky + dy + kh - 1, kr, kg, true);
            g.round_rect(kx, ky + dy, kx + kb - 1, ky + dy + kh - 1, kr, rand, false);
            let t = &i.knopfe_text(k);
            let tw = self.ctext_width(g, t);
            self.ctext(g, kx + (kb - tw) / 2, ky + dy + (kh - th) / 2, t.clone(), lesbar_auf(kg));
        }
        let (kx, ky, kb, _) = i.kreuz;
        let ks = self.sk(12);
        sinnbild(g, "schliessen", kx + (kb - ks) / 2, ky + dy + (kb - ks) / 2, ks, text, false);
        g.pop_clip();
    }

    fn vorschlag_zeichnen(&self, g: &mut Graphics, wi: usize, i: usize) {
        let Some((px, py, pw, ph, zeilen, ih)) = self.vorschlag_rect(wi, i) else { return };
        let wdg = &self.windows[wi].widgets[i];
        let z = wdg.feld.as_ref().unwrap();
        let bg = self.wcol(wdg, "bg", "widget_bg");
        let border = self.wcol(wdg, "border", "widget_border");
        let fg = self.wcol(wdg, "fg", "text_fg");
        let acc = self.wcol(wdg, "accent", "accent");
        let pad = self.m("pad");
        g.round_rect(px + 1, py + 3, px + pw, py + ph + 2, 3, (0x40i64) << 24, true);
        g.box_fill(px, py, px + pw - 1, py + ph - 1, bg);
        g.rect(px, py, px + pw - 1, py + ph - 1, border);
        let th = self.ctext_height(g);
        let tipp = wdg.text.to_lowercase();
        g.push_clip(px + 1, py + 1, pw - 2, ph - 2);
        for k in z.scroll..(z.scroll + zeilen).min(z.treffer.len() as i32) {
            let v = &z.vorschlaege[z.treffer[k as usize]];
            let iy = py + 1 + (k - z.scroll) * ih;
            if k == z.mark { g.box_fill(px + 1, iy, px + pw - 2, iy + ih - 1, shade(acc, -110)); }
            else if k == z.hover { g.box_fill(px + 1, iy, px + pw - 2, iy + ih - 1, shade(bg, 22)); }
            // Was zum Getippten passt, steht in der Akzentfarbe -- so sieht
            // man, warum der Eintrag dasteht.
            let ty = iy + (ih - th) / 2;
            let l = v.to_lowercase();
            match l.find(&tipp).filter(|_| !tipp.is_empty() && l.len() == v.len()) {
                Some(b) => {
                    let (vor, rest) = v.split_at(b);
                    let (treffer, nach) = rest.split_at(tipp.len().min(rest.len()));
                    // Gelegt wird ueber die Breite des GANZEN Textes minus den Rest:
                    // eine Textbreite enthaelt den Abstand zwischen Zeichen, aber
                    // keinen dahinter -- ueber die Breite des Vorspanns klebten die
                    // Teile an der Farbgrenze zusammen.
                    let x0 = px + pad;
                    let ganz = self.ctext_width(g, v);
                    let x1 = x0 + ganz - self.ctext_width(g, &format!("{}{}", treffer, nach));
                    let x2 = if nach.is_empty() { x0 + ganz } else { x0 + ganz - self.ctext_width(g, nach) };
                    self.ctext(g, x0, ty, vor.to_string(), fg);
                    self.ctext(g, x1, ty, treffer.to_string(), if k == z.mark { lesbar_auf(shade(acc, -110)) } else { acc });
                    self.ctext(g, x2, ty, nach.to_string(), fg);
                }
                None => self.ctext(g, px + pad, ty, v.clone(), fg),
            }
        }
        g.pop_clip();
    }

    /// Die Sinnbilder im Textfeld (aus `zeichne_textfeld`).
    pub(super) fn feld_symbole_zeichnen(&self, g: &mut Graphics, wdg: &Widget, ax: i32, ay: i32, w: i32, h: i32) {
        if wdg.feld.is_none() { return; }
        let (l, r) = self.ti_rand(wdg);
        let s = self.sk(14);
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        let leise = self.leise(self.wcol(wdg, "bg", "win_bg"));
        let voll = self.txt_col(wdg);
        for (seite, platz, x0) in [(0usize, l, ax), (1, r, ax + w - r)] {
            if platz == 0 { continue; }
            let name = Self::feld_symbol(wdg, seite);
            if name.is_empty() { continue; }
            let ueber = Self::in_rect(mx, my, (x0, ay, platz, h));
            let c = if ueber && wdg.enabled { voll } else { leise };
            sinnbild(g, &name, x0 + (platz - s) / 2, ay + (h - s) / 2, s, c, !wdg.enabled);
        }
    }
}

/// Suchfeld, Sinnbilder und Vorschlaege in die `.dhform`.
pub(super) fn feld_json(w: &Widget, o: &mut serde_json::Value) {
    let Some(z) = w.feld.as_ref() else { return };
    if z.suche { o["suche"] = serde_json::json!(true); }
    if z.icon.iter().any(|s| !s.is_empty()) { o["icons"] = serde_json::json!(z.icon); }
    if !z.vorschlaege.is_empty() { o["vorschlaege"] = serde_json::json!(z.vorschlaege); }
}
pub(super) fn feld_aus_json(w: &mut Widget, wj: &serde_json::Value) {
    let suche = wj["suche"].as_bool().unwrap_or(false);
    let icons: Vec<String> = wj["icons"].as_array().map(|a| a.iter()
        .map(|x| x.as_str().unwrap_or("").to_string())
        .map(|s| if SINNBILDER.contains(&s.as_str()) { s } else { String::new() }).collect()).unwrap_or_default();
    let vorschlaege: Vec<String> = wj["vorschlaege"].as_array().map(|a| a.iter()
        .filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
    if !suche && icons.iter().all(|s| s.is_empty()) && vorschlaege.is_empty() { return; }
    let mut z = FeldZusatz::neu();
    z.suche = suche;
    for (k, s) in icons.into_iter().take(2).enumerate() { z.icon[k] = s; }
    z.vorschlaege = vorschlaege;
    w.feld = Some(Box::new(z));
}

impl InfoLeiste {
    fn knopfe_text(&self, k: usize) -> String { self.knoepfe.get(k).cloned().unwrap_or_default() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arten_deutsch_und_englisch() {
        assert_eq!(art_nr("", "T").unwrap(), 0);
        assert_eq!(art_nr("Erfolg", "T").unwrap(), 1);
        assert_eq!(art_nr("warning", "T").unwrap(), 2);
        assert_eq!(art_nr("fehler", "T").unwrap(), 3);
        assert!(art_nr("rot", "T").unwrap_err().contains("info, erfolg, warnung, fehler"));
    }
}
