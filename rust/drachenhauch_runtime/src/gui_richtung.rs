//! Rechts-nach-links in Textfeld und Textbereich (Paket 3, zweiter Teil).
//!
//! Gezeichnet und gemessen wird ein Text ohnehin so, wie er dasteht
//! (`Graphics::visuell`). Was die Felder selbst wissen muessen, ist die
//! Lage einer LOGISCHEN Stelle -- wo die Schreibmarke steht, welche Stuecke
//! eine Auswahl deckt, welche Stelle ein Klick trifft. Fuer eine Zeile ohne
//! solche Zeichen bleibt alles beim Messen des Vorspanns wie bisher; nur eine
//! Zeile mit Hebraeisch oder Arabisch geht ueber die Lage hier.

use super::*;
use crate::schreibrichtung::{braucht_zeichen, ordnen, Zeile};

/// Eine geordnete Zeile samt der Lage ihrer Anzeige-Zeichen.
pub(super) struct Lage {
    z: Zeile,
    /// `xs[v]` = linke Kante von Anzeige-Zeichen v, `xs[len]` = ganze Breite.
    xs: Vec<i32>,
}

impl Lage {
    /// Anzeige-Zeichen eines logischen Zeichens (ein Steuerzeichen hat keins).
    fn vis(&self, l: usize) -> Option<usize> { self.z.vis_von_log.get(l).copied().flatten() }

    /// x der Schreibmarke vor dem logischen Zeichen `i` (0..=Laenge).
    /// Entscheidend ist das Zeichen NACH der Marke: steht es rechts nach links,
    /// ist sein rechter Rand gemeint, sonst sein linker. Am Ende der Zeile
    /// zaehlt das Zeichen davor, mit der umgekehrten Kante.
    pub(super) fn marke(&self, i: usize) -> i32 {
        let n = self.z.vis_von_log.len();
        if let Some(l) = (i..n).find(|&l| self.vis(l).is_some()) {
            let v = self.vis(l).unwrap();
            return if self.z.rtl[l] { self.xs[v + 1] } else { self.xs[v] };
        }
        if let Some(l) = (0..i.min(n)).rev().find(|&l| self.vis(l).is_some()) {
            let v = self.vis(l).unwrap();
            return if self.z.rtl[l] { self.xs[v] } else { self.xs[v + 1] };
        }
        0
    }

    /// Die Stuecke (x0, x1), die die logischen Zeichen `a..b` decken -- in
    /// gemischtem Text sind es mehrere.
    pub(super) fn bereiche(&self, a: usize, b: usize) -> Vec<(i32, i32)> {
        let mut drin = vec![false; self.z.text.len()];
        for l in a..b.min(self.z.vis_von_log.len()) {
            if let Some(v) = self.vis(l) { drin[v] = true; }
        }
        let mut aus = Vec::new();
        let mut v = 0;
        while v < drin.len() {
            if !drin[v] { v += 1; continue; }
            let von = v;
            while v < drin.len() && drin[v] { v += 1; }
            aus.push((self.xs[von], self.xs[v]));
        }
        aus
    }

    /// Die logische Stelle, die ein Klick bei x trifft.
    pub(super) fn index(&self, x: i32) -> usize {
        let n = self.z.text.len();
        if n == 0 { return 0; }
        let v = (0..n).find(|&v| x < self.xs[v + 1]).unwrap_or(n - 1);
        let l = self.z.log_von_vis[v];
        // Eine Ligatur (Lam-Alef) deckt zwei logische Zeichen -- "dahinter"
        // heisst hinter dem letzten davon.
        let nach = (0..self.z.vis_von_log.len()).filter(|&k| self.vis(k) == Some(v)).max().unwrap_or(l) + 1;
        let linke_haelfte = x < (self.xs[v] + self.xs[v + 1]) / 2;
        match (self.z.rtl[l], linke_haelfte) {
            (true, true) | (false, false) => nach,
            _ => l,
        }
    }

    /// Das logische Zeichen unter x, `None` daneben.
    pub(super) fn zeichen_bei(&self, x: i32) -> Option<usize> {
        if x < 0 { return None; }
        (0..self.z.text.len()).find(|&v| x < self.xs[v + 1]).map(|v| self.z.log_von_vis[v])
    }

    /// Je Anzeige-Zeichen das logische -- fuer das Zeichnen in Farben.
    pub(super) fn text(&self) -> &[char] { &self.z.text }
    pub(super) fn log_von_vis(&self) -> &[usize] { &self.z.log_von_vis }
    pub(super) fn x(&self, v: usize) -> i32 { self.xs[v] }
    pub(super) fn breite(&self) -> i32 { *self.xs.last().unwrap_or(&0) }
}

impl Gui {
    /// Die Lage einer Zeile, wenn sie Text von rechts nach links traegt --
    /// sonst `None`, und der Aufrufer misst den Vorspann wie bisher.
    /// `mess` misst ein Stueck schon geordneten Textes.
    pub(super) fn rtl_lage_mit(g: &Graphics, zeile: &[char], mess: impl Fn(&str) -> i32) -> Option<Lage> {
        if !braucht_zeichen(zeile) { return None; }
        let z = ordnen(zeile, None);
        let alt = g.richtung_roh(true);
        let mut xs = Vec::with_capacity(z.text.len() + 1);
        xs.push(0);
        for v in 1..=z.text.len() {
            let stueck: String = z.text[..v].iter().collect();
            xs.push(mess(&stueck));
        }
        g.richtung_roh(alt);
        Some(Lage { z, xs })
    }

    /// Dasselbe, gemessen wie `wtext` zeichnet.
    pub(super) fn rtl_lage(&self, g: &Graphics, w: &Widget, zeile: &[char]) -> Option<Lage> {
        Self::rtl_lage_mit(g, zeile, |s| self.wtext_width(g, w, s))
    }

    /// x der Schreibmarke an der logischen Stelle `i` einer Zeile, vom
    /// Zeilenanfang aus.
    pub(super) fn zeilen_x(&self, g: &Graphics, w: &Widget, zeile: &[char], i: usize) -> i32 {
        let i = i.min(zeile.len());
        match self.rtl_lage(g, w, zeile) {
            Some(l) => l.marke(i),
            None => self.wtext_width(g, w, &zeile[..i].iter().collect::<String>()),
        }
    }

    /// Die Stuecke, die die logischen Zeichen `a..b` einer Zeile decken.
    pub(super) fn zeilen_bereiche(&self, g: &Graphics, w: &Widget, zeile: &[char], a: usize, b: usize) -> Vec<(i32, i32)> {
        let (a, b) = (a.min(zeile.len()), b.min(zeile.len()));
        match self.rtl_lage(g, w, zeile) {
            Some(l) => l.bereiche(a, b),
            None => vec![(self.wtext_width(g, w, &zeile[..a].iter().collect::<String>()),
                          self.wtext_width(g, w, &zeile[..b].iter().collect::<String>()))],
        }
    }

    /// Eine Zeile mit Text von rechts nach links in Farben zeichnen: Stueck
    /// fuer Stueck in Anzeige-Reihenfolge, jedes in der Farbe seines
    /// logischen Zeichens. Geordnet ist schon -- gezeichnet wird roh, sonst
    /// drehte sich jedes Stueck noch einmal.
    pub(super) fn rtl_bunt(&self, g: &mut Graphics, w: &Widget, x: i32, y: i32, lage: &Lage, farbe: impl Fn(usize) -> i64) {
        let text = lage.text();
        let log = lage.log_von_vis();
        let alt = g.richtung_roh(true);
        let mut v = 0;
        while v < text.len() {
            let c = farbe(log[v]);
            let von = v;
            while v < text.len() && farbe(log[v]) == c { v += 1; }
            let stueck: String = text[von..v].iter().collect();
            self.wtext(g, w, x + lage.x(von), y, stueck, c);
        }
        g.richtung_roh(alt);
    }
}

/// Die Richtung eines Textfelds: steht der Text als Ganzes von rechts nach
/// links (erstes starkes Zeichen)? Dann tauschen die Pfeiltasten -- links
/// heisst dort im Text weiter.
pub(super) fn zeile_rtl(z: &[char]) -> bool {
    crate::schreibrichtung::basis(z).unwrap_or(false)
}
