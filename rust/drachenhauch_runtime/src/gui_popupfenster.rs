//! Popups ueber den Rand nativer Fenster (Weg E, nach Stufe 4).
//!
//! Teil von `gui.rs`. Ein Kontextmenue, eine Klappliste, die Vorschlaege
//! eines Felds, ein Monatsblatt oder das Rechtschreib-Menue, das ueber den
//! Rand seines nativen Fensters ragt, bekommt ein eigenes randloses Fenster
//! ohne Fokus. Es zeigt den Ausschnitt der Flaeche, zu der das Popup
//! gehoert -- in deren Koordinaten, auch ausserhalb ihres Fensters --, und
//! die Laufzeit rechnet Zeiger und Knoepfe darin auf diese Flaeche um
//! (`Graphics::nativ_popup_anlegen`). So bleiben alle Treffertests, wie sie
//! sind; nur die Grenze, an der ein Popup ausweicht, ist dann der
//! Arbeitsbereich des Schirms statt des Fensterrands (`grenze`).
//!
//! Wo es keine Popup-Fenster gibt (Wayland, `DHRT_NO_POPUP_WINDOWS=1`),
//! weicht ein Popup am Rand seines Fensters aus. Im Hauptfenster bleibt es
//! wie immer: dort weicht es am Rand des Hauptfensters aus.

use super::*;

/// Ein Popup-Fenster: zeigt den Ausschnitt `r` (x, y, b, h) der Flaeche
/// `flaeche`.
pub(crate) struct PopupFenster {
    pub(super) flaeche: u32,
    pub(super) id: u32,
    pub(super) r: (i32, i32, i32, i32),
}

impl Gui {
    /// Wie weit duerfen die Popups des Fensters reichen? (x0, y0, x1, y1) in
    /// den Koordinaten seiner Flaeche. EINE Quelle fuer alle Popups, die am
    /// Rand ausweichen (Kontextmenue, Leistenmenue, Klappliste, Vorschlaege,
    /// Monat, Rechtschreibung).
    pub(super) fn grenze(&self, wi: usize) -> (i32, i32, i32, i32) {
        let f = self.popup_flaeche(wi);
        if f != 0 {
            if let Some(&(_, r)) = self.grenzen.iter().find(|(id, _)| *id == f) { return r; }
        }
        let (b, h) = self.schirm();
        (0, 0, b, h)
    }

    /// Je Bild vor der Bedienung: die Grenzen der nativen Flaechen -- der
    /// Arbeitsbereich ihres Schirms, wenn Popups eigene Fenster bekommen,
    /// sonst ihr eigener Rand.
    pub(super) fn grenzen_auffrischen(&mut self, g: &Graphics) {
        self.grenzen.clear();
        let ids: Vec<u32> = self.windows.iter().filter(|w| w.alive && w.nativ != 0).map(|w| w.nativ)
            .chain(self.teil_ids.iter().copied()).collect();
        let fenster = g.nativ_popups_moeglich();
        for id in ids {
            let eigen = g.nativ_punkte(id).map(|(b, h)| (0, 0, b, h));
            let r = if fenster { g.nativ_arbeitsflaeche(id).or(eigen) } else { eigen };
            if let Some(r) = r { self.grenzen.push((id, r)); }
        }
    }

    /// Die offenen Popups der Flaeche `f`: (x, y, b, h) in ihren
    /// Koordinaten -- aus denselben Quellen, die Zeichnen und Treffertest
    /// fragen.
    pub(super) fn popup_rechtecke(&self, g: &Graphics, f: u32) -> Vec<(i32, i32, i32, i32)> {
        let hier = |wi: usize| self.windows.get(wi).map(|w| w.alive && w.visible).unwrap_or(false) && self.popup_flaeche(wi) == f;
        let mut out = Vec::new();
        let kette = self.context_open.map(|c| c.0).or(self.open_menu.map(|m| m.0));
        if kette.map(hier).unwrap_or(false) {
            for (wi, mi, px, py) in self.popup_chain(g) {
                let (b, h) = self.popup_size(g, wi, mi);
                out.push((px, py, b, h));
            }
        }
        if let Some((dw, di)) = self.open_dropdown.filter(|d| hier(d.0)) {
            if self.windows[dw].widgets.get(di).map(|x| x.alive && x.visible).unwrap_or(false) {
                out.push(self.dropdown_popup_rect(dw, di));
            }
        }
        if self.datum_auf.map(|d| hier(d.0)).unwrap_or(false) {
            if let Some(r) = self.datum_geom() { out.push(r); }
        }
        if self.rs_fenster().map(hier).unwrap_or(false) {
            if let Some((x, y, b, h, _)) = self.rs_rect(g) { out.push((x, y, b, h)); }
        }
        for (wi, w) in self.windows.iter().enumerate() {
            if !hier(wi) { continue; }
            for i in 0..w.widgets.len() {
                if let Some((x, y, b, h, _, _)) = self.vorschlag_rect(wi, i) { out.push((x, y, b, h)); }
            }
        }
        out
    }

    /// Die Flaechen, auf denen gerade ein Popup offen ist (ohne das
    /// Hauptfenster).
    fn flaechen_mit_popups(&self) -> Vec<u32> {
        let mut eigner: Vec<usize> = Vec::new();
        eigner.extend(self.context_open.map(|c| c.0));
        eigner.extend(self.open_menu.map(|m| m.0));
        eigner.extend(self.open_dropdown.map(|d| d.0));
        eigner.extend(self.datum_auf.map(|d| d.0));
        eigner.extend(self.rs_fenster());
        for (wi, w) in self.windows.iter().enumerate() {
            if w.alive && self.vorschlaege_offen(wi) { eigner.push(wi); }
        }
        let mut out: Vec<u32> = Vec::new();
        for wi in eigner {
            if self.windows.get(wi).map(|w| w.alive).unwrap_or(false) {
                let f = self.popup_flaeche(wi);
                if f != 0 && !out.contains(&f) { out.push(f); }
            }
        }
        out
    }

    /// Nach der Bedienung: Popups, die ueber den Rand ihres nativen Fensters
    /// ragen, bekommen ein eigenes Fenster (vorhandene wandern mit); die
    /// uebrigen Popup-Fenster gehen zu.
    pub(super) fn popup_fenster_abgleich(&mut self, g: &mut Graphics) {
        let mut wunsch: Vec<(u32, (i32, i32, i32, i32))> = Vec::new();
        if g.nativ_popups_moeglich() {
            for f in self.flaechen_mit_popups() {
                let Some((b, h)) = g.nativ_punkte(f) else { continue };
                // Eine Teilflaeche (Stufe 3b) rechnet durch ihr ausgeliehenes
                // Gastfenster -- wie Bedienung und Zeichnen.
                let teil = self.teil_von_flaeche(f);
                if let Some((wi, k)) = teil { self.leihen(wi, k, f); }
                for r in self.popup_rechtecke(g, f) {
                    let drin = r.0 >= 0 && r.1 >= 0 && r.0 + r.2 <= b && r.1 + r.3 <= h;
                    if !drin && r.2 > 0 && r.3 > 0 { wunsch.push((f, r)); }
                }
                if teil.is_some() { self.zurueckgeben(); }
            }
        }
        let mut alt = std::mem::take(&mut self.popup_fenster);
        let mut neu = Vec::new();
        for (f, r) in wunsch {
            if let Some(i) = alt.iter().position(|p| p.flaeche == f) {
                let mut p = alt.remove(i);
                if p.r != r { g.nativ_popup_setzen(p.id, r.0, r.1, r.2, r.3); p.r = r; }
                neu.push(p);
            } else if let Some(id) = g.nativ_popup_anlegen(f, r.0, r.1, r.2, r.3) {
                neu.push(PopupFenster { flaeche: f, id, r });
            }
        }
        for p in alt { g.nativ_schliessen(p.id); }
        self.popup_fenster = neu;
    }

    /// Die Popup-Fenster der Flaeche `f` zeichnen -- mit der Sicht und
    /// (bei einer Teilflaeche) der Leihe, unter der auch `f` gezeichnet
    /// wird. Jedes zeigt die ganze obere Schicht von `f`, verschoben auf
    /// seinen Ausschnitt.
    pub(super) fn popup_fenster_zeichnen(&self, g: &mut Graphics, f: u32) {
        for p in self.popup_fenster.iter().filter(|p| p.flaeche == f) {
            if !g.nativ_beginnen(p.id) { continue; }
            let mut eigner: Vec<usize> = Vec::new();
            for wi in self.open_menu.map(|m| m.0).into_iter().chain(self.open_dropdown.map(|d| d.0)) {
                if !eigner.contains(&wi) && self.windows.get(wi).is_some() && self.popup_flaeche(wi) == f { eigner.push(wi); }
            }
            for wi in eigner { self.popups_zeichnen(g, wi); }
            self.draw_top_flaeche(g, f);
            g.nativ_beenden();
        }
    }

    /// GUI_POPUP_WINDOWS(): wie viele Popups gerade als eigenes Fenster ueber
    /// den Rand ihres nativen Fensters stehen.
    pub fn popup_windows(&self) -> i64 { self.popup_fenster.len() as i64 }
}
