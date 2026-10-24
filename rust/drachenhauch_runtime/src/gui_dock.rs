//! Dock (GUI_DOCK): Leisten in den Bereichen links, rechts und unten, mehrere
//! Leisten in einer Gruppe als Reiter, ziehbare Trennlinien, eine Leiste per
//! Zug am Reiter in einen anderen Bereich legen oder als schwebende Leiste
//! abtrennen. Die Mitte bleibt dem Programm (dort steht etwa der Editor).
//!
//! Ein Dock gehoert einem Fenster (`Window::dock`), nicht einem Widget: es ist
//! kein Steuerelement, das man auf ein Formular zieht, sondern die Ordnung des
//! ganzen Fensters. Den INHALT einer Leiste liefert das Programm auf zwei
//! Wegen:
//!
//! * Widgets des Fensters, die per `GUI_DOCK_ADD` zur Leiste gehoeren. Das
//!   Dock blendet sie ein und aus (inaktiver Reiter, zugeklappter Bereich)
//!   und zeichnet sie obenauf, wenn ihre Leiste schwebt -- verteilen tut sie
//!   das Programm selbst in `GUI_DOCK_RECT` (es weiss, was in die Leiste
//!   gehoert; ein Dock, das Widgets selbst anordnet, haette jede IDE-Leiste
//!   umgebaut). `GUI_DOCK_CHANGED` sagt, wann sich ein Rechteck geaendert hat.
//! * Ein ganzes Fenster (`GUI_DOCK_WINDOW`), etwa aus `GUI_FROM_JSON`: das
//!   Dock setzt es randlos auf die Leiste und haelt es in der
//!   Fensterreihenfolge direkt ueber seinem Gastfenster. So kommen die Leisten
//!   der Plugins in die IDE -- ihre Widgets verweisen ueber Nummern
//!   aufeinander, in ein fremdes Fenster eingesetzt verrutschten die.
//!
//! Gerechnet wird alles in Inhaltskoordinaten des Fensters (wie die Lage der
//! Widgets) und an EINER Stelle (`DockState::rechnen`): Zeichnen, Treffertest,
//! Ziehen und `GUI_DOCK_RECT` lesen dasselbe Ergebnis.
use super::*;

/// Die Bereiche, in die eine Leiste gelegt werden kann (Index = Nummer).
pub(super) const BEREICH_NAMEN: [&str; 3] = ["links", "rechts", "unten"];

/// Ab so vielen Punkten Bewegung wird aus dem Druck auf einen Reiter ein Zug.
const ZUG_SCHWELLE: i32 = 5;
/// So nah am Rand der Mitte legt ein Zug die Leiste in den Bereich dort.
const RAND_FANG: i32 = 40;
/// Kleinste Groesse eines Bereichs und einer Gruppe.
const MIN_GROESSE: i32 = 60;

pub(super) fn bereich_nr(s: &str) -> Option<usize> {
    match s.to_lowercase().as_str() {
        "links" | "left" => Some(0),
        "rechts" | "right" => Some(1),
        "unten" | "bottom" => Some(2),
        _ => None,
    }
}

#[derive(Clone)]
pub(super) struct Leiste {
    pub(super) name: String,
    pub(super) titel: String,
    /// Ausgeblendet (GUI_DOCK_SHOW FALSE oder das Kreuz am Reiter). Die
    /// Leiste behaelt ihren Platz in der Gruppe und kommt dorthin zurueck.
    pub(super) zu: bool,
    pub(super) schliessbar: bool,
    /// Fenster als Inhalt (GUI_DOCK_WINDOW), -1 = Widgets des Gastfensters.
    pub(super) fenster: i64,
    /// Der Bereich, in dem sie zuletzt angedockt war -- dorthin geht eine
    /// schwebende Leiste beim Doppelklick auf ihren Kopf zurueck.
    pub(super) heim: usize,
    // --- gerechnet (rechnen) ---
    /// Inhaltsrechteck (unter dem Kopf), in Inhaltskoordinaten des Fensters.
    pub(super) rect: (i32, i32, i32, i32),
    /// Ist ihr Inhalt zu sehen (vorderer Reiter, Bereich an, nicht zu)?
    pub(super) sichtbar: bool,
    /// Index der schwebenden Gruppe, in der sie steckt, -1 = angedockt.
    pub(super) schwebt: i32,
}

#[derive(Clone, Default)]
pub(super) struct Gruppe {
    pub(super) leisten: Vec<usize>,
    /// Die Leiste vorne (Index in `DockState::leisten`).
    pub(super) aktiv: usize,
    /// Anteil am Bereich (wird beim Rechnen auf die sichtbaren Gruppen verteilt).
    pub(super) anteil: f32,
}

#[derive(Clone)]
pub(super) struct Bereich {
    pub(super) gruppen: Vec<Gruppe>,
    /// Breite (links/rechts) bzw. Hoehe (unten), skaliert.
    pub(super) groesse: i32,
    pub(super) sichtbar: bool,
}

#[derive(Clone)]
pub(super) struct Schwebe {
    pub(super) gruppe: Gruppe,
    pub(super) x: i32, pub(super) y: i32, pub(super) w: i32, pub(super) h: i32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Ort { Bereich(usize, usize), Schwebe(usize) }

/// Eine gezeichnete Gruppe: Ort, Rechteck (mit Kopf) und die Reiter im Kopf
/// als (Leiste, x0, x1).
pub(super) struct GGeom {
    pub(super) ort: Ort,
    pub(super) r: (i32, i32, i32, i32),
    pub(super) tabs: Vec<(usize, i32, i32)>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum TArt {
    /// Groesse eines Bereichs.
    Bereich(usize),
    /// Grenze zwischen zwei benachbarten Gruppen eines Bereichs.
    Gruppe(usize, usize, usize),
}

pub(super) struct Trenner {
    pub(super) art: TArt,
    pub(super) r: (i32, i32, i32, i32),
    /// Die Linie steht senkrecht, gezogen wird also waagerecht.
    pub(super) senkrecht: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Ziel {
    /// Als Reiter in die Gruppe der Leiste.
    Reiter(usize),
    /// Neue Gruppe neben der Gruppe der Leiste (vor = davor).
    Teilen(usize, bool),
    /// Neue Gruppe am Ende des Bereichs.
    Rand(usize),
    /// Abtrennen: schwebend an dieser Stelle.
    Schweben(i32, i32),
}

pub(super) enum Zug {
    Trenner { art: TArt, m0: i32, a0: i32, b0: i32 },
    Leiste { leiste: usize, x0: i32, y0: i32, aktiv: bool, ziel: Option<Ziel> },
    Schwebe { nr: usize, dx: i32, dy: i32 },
    Groesse { nr: usize, dx: i32, dy: i32 },
}

pub(super) struct DockState {
    pub(super) x: i32, pub(super) y: i32, pub(super) w: i32, pub(super) h: i32,
    pub(super) kopf: i32,
    pub(super) rand: i32,
    pub(super) abstand: i32,
    pub(super) fuge: i32,
    pub(super) leisten: Vec<Leiste>,
    pub(super) bereiche: [Bereich; 3],
    pub(super) schweben: Vec<Schwebe>,
    // --- gerechnet ---
    pub(super) geom: Vec<GGeom>,
    pub(super) trenner: Vec<Trenner>,
    pub(super) mitte: (i32, i32, i32, i32),
    /// Breite eines Reiters je Titel (gemessen in GUI_UPDATE, sonst geschaetzt).
    pub(super) tab_b: HashMap<String, i32>,
    /// Ereignisse dieses Bildes.
    pub(super) geaendert: bool,
    pub(super) geschlossen: String,
    /// Was sich seit dem letzten Bild getan hat (wird zu `geaendert`).
    pub(super) sig: Vec<i32>,
    pub(super) zug: Option<Zug>,
    /// Ein gesetzter Stand (GUI_DOCK_SET_STATE), der Leisten nennt, die es
    /// noch nicht gibt -- etwa die eines Plugins, das spaeter startet.
    pub(super) vorgemerkt: Option<serde_json::Value>,
}

impl DockState {
    pub(super) fn neu(x: i32, y: i32, w: i32, h: i32, kopf: i32, rand: i32) -> DockState {
        let b = |g: i32| Bereich { gruppen: Vec::new(), groesse: g, sichtbar: true };
        DockState {
            x, y, w, h, kopf, rand, abstand: rand, fuge: 2,
            leisten: Vec::new(),
            bereiche: [b(240), b(240), b(200)],
            schweben: Vec::new(),
            geom: Vec::new(), trenner: Vec::new(), mitte: (x, y, w, h),
            tab_b: HashMap::new(),
            geaendert: false, geschlossen: String::new(), sig: Vec::new(),
            zug: None, vorgemerkt: None,
        }
    }

    pub(super) fn leiste_nr(&self, name: &str) -> Option<usize> {
        let n = name.to_lowercase();
        self.leisten.iter().position(|l| l.name == n)
    }

    /// Wo steckt die Leiste?
    pub(super) fn ort_von(&self, li: usize) -> Option<Ort> {
        for (b, ber) in self.bereiche.iter().enumerate() {
            for (g, gr) in ber.gruppen.iter().enumerate() {
                if gr.leisten.contains(&li) { return Some(Ort::Bereich(b, g)); }
            }
        }
        for (k, s) in self.schweben.iter().enumerate() {
            if s.gruppe.leisten.contains(&li) { return Some(Ort::Schwebe(k)); }
        }
        None
    }

    pub(super) fn gruppe(&self, o: Ort) -> &Gruppe {
        match o { Ort::Bereich(b, g) => &self.bereiche[b].gruppen[g], Ort::Schwebe(k) => &self.schweben[k].gruppe }
    }
    pub(super) fn gruppe_mut(&mut self, o: Ort) -> &mut Gruppe {
        match o { Ort::Bereich(b, g) => &mut self.bereiche[b].gruppen[g], Ort::Schwebe(k) => &mut self.schweben[k].gruppe }
    }

    fn offen_in(&self, g: &Gruppe) -> Vec<usize> {
        g.leisten.iter().copied().filter(|&i| !self.leisten[i].zu).collect()
    }

    /// Hat der Bereich etwas zu zeigen?
    pub(super) fn bereich_an(&self, b: usize) -> bool {
        let ber = &self.bereiche[b];
        ber.sichtbar && ber.gruppen.iter().any(|g| g.leisten.iter().any(|&i| !self.leisten[i].zu))
    }

    /// Die Leiste aus ihrer Gruppe nehmen; eine leere Gruppe faellt weg.
    pub(super) fn herausnehmen(&mut self, li: usize) {
        let o = match self.ort_von(li) { Some(o) => o, None => return };
        if let Ort::Bereich(b, _) = o { self.leisten[li].heim = b; }
        let leer = {
            let g = self.gruppe_mut(o);
            g.leisten.retain(|&i| i != li);
            if g.aktiv == li { if let Some(&e) = g.leisten.first() { g.aktiv = e; } }
            g.leisten.is_empty()
        };
        if leer {
            match o {
                Ort::Bereich(b, g) => { self.bereiche[b].gruppen.remove(g); }
                Ort::Schwebe(k) => { self.schweben.remove(k); }
            }
        }
    }

    /// Die Leiste an ein Ziel legen. Ein Ziel, das die Leiste selbst
    /// betrifft (der eigene Reiter, die eigene Gruppe allein), aendert nichts.
    pub(super) fn einsetzen(&mut self, li: usize, ziel: Ziel) {
        match ziel {
            Ziel::Reiter(anker) | Ziel::Teilen(anker, _) if anker == li => return,
            Ziel::Reiter(anker) if self.ort_von(anker) == self.ort_von(li) => {
                // schon in dieser Gruppe -- nur nach vorn
                let o = self.ort_von(li).unwrap();
                self.gruppe_mut(o).aktiv = li;
                return;
            }
            Ziel::Teilen(anker, _) if self.ort_von(anker) == self.ort_von(li)
                && self.gruppe(self.ort_von(li).unwrap()).leisten.len() == 1 => return,
            _ => {}
        }
        self.herausnehmen(li);
        self.leisten[li].zu = false;
        match ziel {
            Ziel::Reiter(anker) => {
                if let Some(o) = self.ort_von(anker) {
                    let g = self.gruppe_mut(o);
                    g.leisten.push(li);
                    g.aktiv = li;
                } else { self.einsetzen(li, Ziel::Rand(self.leisten[li].heim)); }
            }
            Ziel::Teilen(anker, vor) => {
                match self.ort_von(anker) {
                    Some(Ort::Bereich(b, g)) => {
                        let halb = self.bereiche[b].gruppen[g].anteil / 2.0;
                        self.bereiche[b].gruppen[g].anteil = halb;
                        let neu = Gruppe { leisten: vec![li], aktiv: li, anteil: halb };
                        let at = if vor { g } else { g + 1 };
                        self.bereiche[b].gruppen.insert(at, neu);
                        self.leisten[li].heim = b;
                    }
                    _ => self.einsetzen(li, Ziel::Rand(self.leisten[li].heim)),
                }
            }
            Ziel::Rand(b) => {
                let anteil = if self.bereiche[b].gruppen.is_empty() { 1.0 } else {
                    let s: f32 = self.bereiche[b].gruppen.iter().map(|g| g.anteil).sum();
                    s / self.bereiche[b].gruppen.len() as f32
                };
                self.bereiche[b].gruppen.push(Gruppe { leisten: vec![li], aktiv: li, anteil });
                self.bereiche[b].sichtbar = true;
                self.leisten[li].heim = b;
            }
            Ziel::Schweben(x, y) => {
                let (w, h) = (self.sk_breite(), self.kopf * 10);
                self.schweben.push(Schwebe { gruppe: Gruppe { leisten: vec![li], aktiv: li, anteil: 1.0 }, x, y, w, h });
            }
        }
    }

    /// Vorgabebreite einer frisch abgetrennten Leiste.
    fn sk_breite(&self) -> i32 { (self.kopf * 14).max(200) }

    /// Eine ganze schwebende Gruppe an ihren Heimatbereich zurueckgeben.
    pub(super) fn heimholen(&mut self, k: usize) {
        if k >= self.schweben.len() { return; }
        let s = self.schweben.remove(k);
        let heim = self.leisten[s.gruppe.aktiv].heim;
        let anteil = if self.bereiche[heim].gruppen.is_empty() { 1.0 } else {
            let summe: f32 = self.bereiche[heim].gruppen.iter().map(|g| g.anteil).sum();
            summe / self.bereiche[heim].gruppen.len() as f32
        };
        self.bereiche[heim].gruppen.push(Gruppe { anteil, ..s.gruppe });
        self.bereiche[heim].sichtbar = true;
    }

    /// Alles rechnen: Bereiche, Gruppen, Reiter, Trennlinien, Mitte und je
    /// Leiste ihr Inhaltsrechteck. EINE Quelle fuer Zeichnen, Treffertest,
    /// Ziehen und GUI_DOCK_RECT.
    pub(super) fn rechnen(&mut self) {
        self.geom.clear();
        self.trenner.clear();
        for l in self.leisten.iter_mut() { l.sichtbar = false; l.schwebt = -1; l.rect = (0, 0, 0, 0); }
        let (dx, dy, dw, dh) = (self.x, self.y, self.w, self.h);
        let an = [self.bereich_an(0), self.bereich_an(1), self.bereich_an(2)];
        let uh = if an[2] { self.bereiche[2].groesse.clamp(self.kopf + 10, (dh - self.kopf - 10).max(self.kopf + 10)) } else { 0 };
        let mh = if an[2] { dh - uh - self.fuge } else { dh };
        let platz = (dw - MIN_GROESSE).max(0);
        let mut lw = if an[0] { self.bereiche[0].groesse.clamp(MIN_GROESSE.min(platz), platz.max(1)) } else { 0 };
        let mut rw = if an[1] { self.bereiche[1].groesse.clamp(MIN_GROESSE.min(platz), platz.max(1)) } else { 0 };
        if lw + rw > platz { let zuviel = lw + rw - platz; rw = (rw - zuviel).max(0); if lw + rw > platz { lw = platz - rw; } }
        self.mitte = (dx + lw, dy, (dw - lw - rw).max(0), mh.max(0));
        let rand = self.rand;
        if an[0] { self.trenner.push(Trenner { art: TArt::Bereich(0), r: (dx + lw - rand, dy, rand.max(4), mh), senkrecht: true }); }
        if an[1] { self.trenner.push(Trenner { art: TArt::Bereich(1), r: (dx + dw - rw, dy, rand.max(4), mh), senkrecht: true }); }
        if an[2] { self.trenner.push(Trenner { art: TArt::Bereich(2), r: (dx, dy + mh - 3, dw, self.fuge + 6), senkrecht: false }); }
        let flaechen = [(dx, dy, lw, mh), (dx + dw - rw, dy, rw, mh), (dx, dy + dh - uh, dw, uh)];
        for b in 0..3 {
            if !an[b] { continue; }
            let (ax, ay, aw, ah) = flaechen[b];
            let (ix, iy, iw, ih) = (ax + rand, ay, (aw - 2 * rand).max(0), ah);
            let sicht: Vec<usize> = (0..self.bereiche[b].gruppen.len())
                .filter(|&g| self.bereiche[b].gruppen[g].leisten.iter().any(|&i| !self.leisten[i].zu)).collect();
            let n = sicht.len() as i32;
            if n == 0 { continue; }
            let lang = if b < 2 { ih } else { iw };
            let gesamt = (lang - self.abstand * (n - 1)).max(0);
            let summe: f32 = sicht.iter().map(|&g| self.bereiche[b].gruppen[g].anteil.max(0.01)).sum();
            let mut pos = 0;
            for (k, &g) in sicht.iter().enumerate() {
                let teil = if k as i32 == n - 1 { gesamt - pos } else {
                    ((self.bereiche[b].gruppen[g].anteil.max(0.01) / summe) * gesamt as f32).round() as i32
                };
                let r = if b < 2 { (ix, iy + pos + self.abstand * k as i32, iw, teil) }
                        else { (ix + pos + self.abstand * k as i32, iy, teil, ih) };
                if k > 0 {
                    let vorher = sicht[k - 1];
                    let tr = if b < 2 { (ix, r.1 - self.abstand, iw, self.abstand) } else { (r.0 - self.abstand, iy, self.abstand, ih) };
                    self.trenner.push(Trenner { art: TArt::Gruppe(b, vorher, g), r: tr, senkrecht: b == 2 });
                }
                self.geom.push(GGeom { ort: Ort::Bereich(b, g), r, tabs: Vec::new() });
                pos += teil;
            }
        }
        for k in 0..self.schweben.len() {
            if !self.schweben[k].gruppe.leisten.iter().any(|&i| !self.leisten[i].zu) { continue; }
            // Im Dock halten: ein gesicherter Stand aus einem groesseren
            // Fenster legte sie sonst ausser Reichweite.
            {
                let kopf = self.kopf;
                let s = &mut self.schweben[k];
                s.x = s.x.clamp(dx - s.w + 60, (dx + dw - 60).max(dx - s.w + 60));
                s.y = s.y.clamp(dy, (dy + dh - kopf).max(dy));
            }
            let s = &self.schweben[k];
            self.geom.push(GGeom { ort: Ort::Schwebe(k), r: (s.x, s.y, s.w, s.h), tabs: Vec::new() });
        }
        // Reiter und Inhalte je Gruppe.
        for gi in 0..self.geom.len() {
            let o = self.geom[gi].ort;
            let offen = self.offen_in(self.gruppe(o));
            let aktiv = { let a = self.gruppe(o).aktiv; if offen.contains(&a) { a } else { offen[0] } };
            self.gruppe_mut(o).aktiv = aktiv;
            let r = self.geom[gi].r;
            let mut x = r.0;
            let mut tabs = Vec::new();
            for &li in &offen {
                let tb = self.tab_breite(li);
                tabs.push((li, x, x + tb));
                x += tb + 2;
            }
            self.geom[gi].tabs = tabs;
            let inhalt = match o {
                Ort::Bereich(..) => (r.0, r.1 + self.kopf, r.2, (r.3 - self.kopf).max(0)),
                Ort::Schwebe(_) => (r.0 + 1, r.1 + self.kopf, (r.2 - 2).max(0), (r.3 - self.kopf - 1).max(0)),
            };
            for &li in &self.gruppe(o).leisten.clone() {
                self.leisten[li].schwebt = match o { Ort::Schwebe(k) => k as i32, _ => -1 };
                if li == aktiv { self.leisten[li].sichtbar = true; self.leisten[li].rect = inhalt; }
            }
        }
    }

    /// Breite eines Reiters: Titel (gemessen oder geschaetzt) plus Rand und
    /// gegebenenfalls Kreuz.
    pub(super) fn tab_breite(&self, li: usize) -> i32 {
        let l = &self.leisten[li];
        let t = self.tab_b.get(&l.titel).copied().unwrap_or(l.titel.chars().count() as i32 * 8);
        t + 16 + if l.schliessbar { 16 } else { 0 }
    }

    /// Das Kreuz eines Reiters (x, y, Kante) -- eine Quelle fuer Zeichnen und Klick.
    pub(super) fn kreuz_rect(&self, gy: i32, x1: i32) -> (i32, i32, i32) {
        let k = 8;
        (x1 - 6 - k, gy + (self.kopf - k) / 2, k)
    }

    /// Eine Unterschrift dessen, was man sieht -- aendert sie sich, meldet
    /// GUI_DOCK_CHANGED ein Bild lang.
    pub(super) fn unterschrift(&self) -> Vec<i32> {
        let mut s = vec![self.mitte.0, self.mitte.1, self.mitte.2, self.mitte.3];
        for l in &self.leisten {
            s.extend_from_slice(&[l.rect.0, l.rect.1, l.rect.2, l.rect.3, l.sichtbar as i32, l.schwebt]);
        }
        s
    }

    /// Die Gruppe, deren Kopf unter (x, y) liegt -- schwebende zuerst (oben).
    pub(super) fn kopf_unter(&self, x: i32, y: i32) -> Option<usize> {
        let mut treffer = None;
        for (gi, g) in self.geom.iter().enumerate() {
            let (gx, gy, gw, _) = g.r;
            if x >= gx && x < gx + gw && y >= gy && y < gy + self.kopf {
                match (treffer, g.ort) {
                    (None, _) | (Some(_), Ort::Schwebe(_)) => treffer = Some(gi),
                    _ => {}
                }
            }
        }
        treffer
    }

    /// Die oberste schwebende Gruppe unter (x, y) (Index in `geom`).
    pub(super) fn schwebe_unter(&self, x: i32, y: i32) -> Option<usize> {
        (0..self.geom.len()).rev().find(|&gi| {
            let g = &self.geom[gi];
            matches!(g.ort, Ort::Schwebe(_)) && x >= g.r.0 && x < g.r.0 + g.r.2 && y >= g.r.1 && y < g.r.1 + g.r.3
        })
    }

    /// Wohin ein gezogener Reiter fiele, wenn man ihn hier losliesse.
    pub(super) fn ziel_unter(&self, li: usize, x: i32, y: i32) -> Ziel {
        if let Some(gi) = self.kopf_unter(x, y) {
            let anker = self.gruppe(self.geom[gi].ort).aktiv;
            return Ziel::Reiter(anker);
        }
        if self.schwebe_unter(x, y).is_none() {
            for g in &self.geom {
                if let Ort::Bereich(b, _) = g.ort {
                    let (gx, gy, gw, gh) = g.r;
                    if x >= gx && x < gx + gw && y >= gy && y < gy + gh {
                        let anker = self.gruppe(g.ort).aktiv;
                        let vor = if b < 2 { y < gy + gh / 2 } else { x < gx + gw / 2 };
                        return Ziel::Teilen(anker, vor);
                    }
                }
            }
            let (mx, my, mw, mh) = self.mitte;
            let in_mitte = x >= mx && x < mx + mw && y >= my && y < self.y + self.h;
            if in_mitte {
                if x < mx + RAND_FANG { return Ziel::Rand(0); }
                if x >= mx + mw - RAND_FANG { return Ziel::Rand(1); }
                if y >= my + mh - RAND_FANG && y < self.y + self.h { return Ziel::Rand(2); }
            }
        }
        let _ = li;
        Ziel::Schweben(x - 30, y - self.kopf / 2)
    }

    /// Wo das Ziel hinkaeme -- fuer den Hinweis waehrend des Zugs.
    pub(super) fn ziel_rect(&self, ziel: Ziel) -> (i32, i32, i32, i32) {
        let geom_von = |li: usize| self.geom.iter().find(|g| self.gruppe(g.ort).leisten.contains(&li));
        match ziel {
            Ziel::Reiter(a) => geom_von(a).map(|g| g.r).unwrap_or((0, 0, 0, 0)),
            Ziel::Teilen(a, vor) => match geom_von(a) {
                Some(g) => {
                    let (gx, gy, gw, gh) = g.r;
                    match g.ort {
                        Ort::Bereich(b, _) if b < 2 => if vor { (gx, gy, gw, gh / 2) } else { (gx, gy + gh / 2, gw, gh - gh / 2) },
                        _ => if vor { (gx, gy, gw / 2, gh) } else { (gx + gw / 2, gy, gw - gw / 2, gh) },
                    }
                }
                None => (0, 0, 0, 0),
            },
            Ziel::Rand(b) => {
                let (mx, my, mw, mh) = self.mitte;
                let breite = self.bereiche[b].groesse.min(mw / 2).max(MIN_GROESSE);
                match b {
                    0 => (mx, my, breite, mh),
                    1 => (mx + mw - breite, my, breite, mh),
                    _ => { let hoch = self.bereiche[2].groesse.min(mh / 2).max(MIN_GROESSE); (mx, my + mh - hoch, mw, hoch) }
                }
            }
            Ziel::Schweben(x, y) => (x, y, self.sk_breite(), self.kopf * 10),
        }
    }

    /// Einen Trennerzug anfangen: die Ausgangsmasse merken.
    pub(super) fn trenner_anfang(&self, art: TArt, m: i32) -> Zug {
        let (a0, b0) = match art {
            TArt::Bereich(b) => (self.bereiche[b].groesse, 0),
            TArt::Gruppe(b, ga, gb) => {
                let groesse = |g: usize| self.geom.iter().find(|x| x.ort == Ort::Bereich(b, g))
                    .map(|x| if b < 2 { x.r.3 } else { x.r.2 }).unwrap_or(0);
                (groesse(ga), groesse(gb))
            }
        };
        Zug::Trenner { art, m0: m, a0, b0 }
    }

    /// Den Trennerzug fortsetzen (m = Mauslage in der Zugrichtung).
    pub(super) fn trenner_ziehen(&mut self, art: TArt, m0: i32, a0: i32, b0: i32, m: i32) {
        let d = m - m0;
        match art {
            TArt::Bereich(b) => {
                let neu = match b { 0 => a0 + d, 1 => a0 - d, _ => a0 - d };
                let grenze = if b < 2 { self.w - MIN_GROESSE } else { self.h - self.kopf - 10 };
                self.bereiche[b].groesse = neu.clamp(MIN_GROESSE.min(grenze.max(1)), grenze.max(MIN_GROESSE));
            }
            TArt::Gruppe(b, ga, gb) => {
                let summe = a0 + b0;
                if summe <= 0 { return; }
                let neu_a = (a0 + d).clamp(MIN_GROESSE.min(summe / 2), (summe - MIN_GROESSE).max(summe / 2));
                let gr = &mut self.bereiche[b].gruppen;
                if ga >= gr.len() || gb >= gr.len() { return; }
                let anteile = gr[ga].anteil + gr[gb].anteil;
                gr[ga].anteil = anteile * neu_a as f32 / summe as f32;
                gr[gb].anteil = anteile - gr[ga].anteil;
            }
        }
    }

    /// Der Stand als JSON (GUI_DOCK_STATE$): Groessen in LOGISCHEN Punkten
    /// (`unsk`), sonst wuechse er bei jedem Sichern um den Massstab.
    pub(super) fn stand(&self, unsk: &dyn Fn(i32) -> i32) -> serde_json::Value {
        let gruppe = |g: &Gruppe| serde_json::json!({
            "leisten": g.leisten.iter().map(|&i| self.leisten[i].name.clone()).collect::<Vec<_>>(),
            "aktiv": self.leisten[g.aktiv].name,
            "anteil": (g.anteil * 1000.0).round() / 1000.0,
        });
        let mut ber = serde_json::Map::new();
        for (b, n) in BEREICH_NAMEN.iter().enumerate() {
            let x = &self.bereiche[b];
            ber.insert(n.to_string(), serde_json::json!({
                "groesse": unsk(x.groesse),
                "sichtbar": x.sichtbar,
                "gruppen": x.gruppen.iter().map(gruppe).collect::<Vec<_>>(),
            }));
        }
        serde_json::json!({
            "bereiche": ber,
            "schweben": self.schweben.iter().map(|s| serde_json::json!({
                "x": unsk(s.x), "y": unsk(s.y), "w": unsk(s.w), "h": unsk(s.h),
                "gruppe": gruppe(&s.gruppe),
            })).collect::<Vec<_>>(),
            "zu": self.leisten.iter().filter(|l| l.zu).map(|l| l.name.clone()).collect::<Vec<_>>(),
        })
    }

    /// Einen Stand uebernehmen. Leisten, die er nicht nennt, bleiben, wo sie
    /// sind; Leisten, die es (noch) nicht gibt, merkt sich das Dock und setzt
    /// sie ein, sobald sie angemeldet werden (`nachtragen`).
    pub(super) fn stand_setzen(&mut self, v: &serde_json::Value, sk: &dyn Fn(i32) -> i32) -> Result<(), String> {
        if !v.is_object() { return Err("GUI_DOCK_SET_STATE: erwartet ein JSON-Objekt".into()); }
        let bekannt = |s: &DockState, n: &str| s.leiste_nr(n);
        // Erst alle genannten, bekannten Leisten herausnehmen ...
        let mut genannt: Vec<usize> = Vec::new();
        let sammeln = |g: &serde_json::Value, genannt: &mut Vec<usize>| {
            if let Some(a) = g["leisten"].as_array() {
                for n in a.iter().filter_map(|x| x.as_str()) {
                    if let Some(i) = bekannt(self, n) { if !genannt.contains(&i) { genannt.push(i); } }
                }
            }
        };
        for n in BEREICH_NAMEN.iter() {
            if let Some(gs) = v["bereiche"][*n]["gruppen"].as_array() { for g in gs { sammeln(g, &mut genannt); } }
        }
        if let Some(ss) = v["schweben"].as_array() { for s in ss { sammeln(&s["gruppe"], &mut genannt); } }
        for &i in &genannt { self.herausnehmen(i); }
        let gruppe_aus = |s: &DockState, g: &serde_json::Value| -> Option<Gruppe> {
            let leisten: Vec<usize> = g["leisten"].as_array()?.iter()
                .filter_map(|x| x.as_str()).filter_map(|n| bekannt(s, n)).collect();
            if leisten.is_empty() { return None; }
            let aktiv = g["aktiv"].as_str().and_then(|n| bekannt(s, n)).filter(|a| leisten.contains(a)).unwrap_or(leisten[0]);
            Some(Gruppe { leisten, aktiv, anteil: g["anteil"].as_f64().unwrap_or(1.0).max(0.01) as f32 })
        };
        // ... dann in der Ordnung des Stands einsetzen: die genannten Gruppen
        // VOR die verbliebenen, damit die Reihenfolge stimmt.
        for (b, n) in BEREICH_NAMEN.iter().enumerate() {
            let bv = &v["bereiche"][*n];
            if let Some(g) = bv["groesse"].as_i64() { self.bereiche[b].groesse = sk(g as i32).max(MIN_GROESSE); }
            if let Some(s) = bv["sichtbar"].as_bool() { self.bereiche[b].sichtbar = s; }
            if let Some(gs) = bv["gruppen"].as_array() {
                let neue: Vec<Gruppe> = gs.iter().filter_map(|g| gruppe_aus(self, g)).collect();
                for g in &neue { for &i in &g.leisten { self.leisten[i].heim = b; } }
                let alt = std::mem::take(&mut self.bereiche[b].gruppen);
                self.bereiche[b].gruppen = neue;
                self.bereiche[b].gruppen.extend(alt);
            }
        }
        if let Some(ss) = v["schweben"].as_array() {
            for s in ss {
                if let Some(g) = gruppe_aus(self, &s["gruppe"]) {
                    let gi = |k: &str, d: i32| s[k].as_i64().map(|x| sk(x as i32)).unwrap_or(d);
                    self.schweben.push(Schwebe { gruppe: g, x: gi("x", 40), y: gi("y", 40), w: gi("w", 300).max(MIN_GROESSE), h: gi("h", 240).max(MIN_GROESSE) });
                }
            }
        }
        if let Some(z) = v["zu"].as_array() {
            let zu: Vec<String> = z.iter().filter_map(|x| x.as_str().map(|s| s.to_lowercase())).collect();
            for l in self.leisten.iter_mut() { l.zu = zu.contains(&l.name); }
        }
        self.vorgemerkt = Some(v.clone());
        Ok(())
    }

    /// Eine eben angemeldete Leiste an den Platz legen, den ein frueher
    /// gesetzter Stand fuer sie vorsah: als Reiter neben eine Leiste, die dort
    /// schon steht, sonst in ihren Bereich.
    pub(super) fn nachtragen(&mut self, li: usize) {
        let v = match self.vorgemerkt.clone() { Some(v) => v, None => return };
        let name = self.leisten[li].name.clone();
        let nennt = |g: &serde_json::Value| g["leisten"].as_array()
            .map(|a| a.iter().any(|x| x.as_str().map(|s| s.to_lowercase()) == Some(name.clone()))).unwrap_or(false);
        let nachbar = |s: &DockState, g: &serde_json::Value| g["leisten"].as_array().and_then(|a| a.iter()
            .filter_map(|x| x.as_str()).filter_map(|n| s.leiste_nr(n)).find(|&i| i != li && s.ort_von(i).is_some()));
        for (b, n) in BEREICH_NAMEN.iter().enumerate() {
            if let Some(gs) = v["bereiche"][*n]["gruppen"].as_array() {
                for g in gs.iter().filter(|g| nennt(g)) {
                    let aktiv = g["aktiv"].as_str().map(|s| s.to_lowercase()) == Some(name.clone());
                    match nachbar(self, g) {
                        Some(a) => {
                            self.einsetzen(li, Ziel::Reiter(a));
                            if !aktiv { let o = self.ort_von(a).unwrap(); self.gruppe_mut(o).aktiv = a; }
                        }
                        None => self.einsetzen(li, Ziel::Rand(b)),
                    }
                    self.leisten[li].zu = v["zu"].as_array().map(|z| z.iter().any(|x| x.as_str().map(|s| s.to_lowercase()) == Some(name.clone()))).unwrap_or(false);
                    return;
                }
            }
        }
        if let Some(ss) = v["schweben"].as_array() {
            for s in ss.iter().filter(|s| nennt(&s["gruppe"])) {
                match nachbar(self, &s["gruppe"]) {
                    Some(a) => self.einsetzen(li, Ziel::Reiter(a)),
                    None => {
                        let x = s["x"].as_i64().unwrap_or(40) as i32;
                        let y = s["y"].as_i64().unwrap_or(40) as i32;
                        self.einsetzen(li, Ziel::Schweben(x, y));
                        if let Some(last) = self.schweben.last_mut() {
                            last.w = s["w"].as_i64().unwrap_or(last.w as i64) as i32;
                            last.h = s["h"].as_i64().unwrap_or(last.h as i64) as i32;
                        }
                    }
                }
                return;
            }
        }
    }
}

// --------------------------------------------------------------------------
// Anbindung an die gui: Befehle, Rechnen je Bild, Zeichnen, Maus.
// --------------------------------------------------------------------------
impl Gui {
    fn dock_mut(&mut self, win: i64, f: &str) -> Result<&mut DockState, String> {
        let w = self.win_mut(win, f)?;
        w.dock.as_deref_mut().ok_or_else(|| format!("{}: das Fenster hat kein Dock -- erst GUI_DOCK(win, x, y, w, h)", f))
    }
    fn dock_ref(&self, win: i64, f: &str) -> Result<&DockState, String> {
        let w = self.windows.get(win as usize).filter(|w| w.alive).ok_or_else(|| format!("{}: ungueltiges GUI_WINDOW-Handle", f))?;
        w.dock.as_deref().ok_or_else(|| format!("{}: das Fenster hat kein Dock -- erst GUI_DOCK(win, x, y, w, h)", f))
    }
    fn dock_leiste(&self, win: i64, name: &str, f: &str) -> Result<usize, String> {
        let d = self.dock_ref(win, f)?;
        d.leiste_nr(name).ok_or_else(|| {
            let namen: Vec<String> = d.leisten.iter().map(|l| l.name.clone()).collect();
            format!("{}: keine Leiste '{}' im Dock (da sind: {})", f, name, if namen.is_empty() { "keine".into() } else { namen.join(", ") })
        })
    }

    /// GUI_DOCK(win, x, y, w, h): das Dock einrichten bzw. sein Rechteck
    /// setzen (Inhaltskoordinaten wie bei Widgets).
    pub fn dock(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32) -> Result<(), String> {
        let (x, y, w, h) = (self.sk(x), self.sk(y), self.sk(w.max(0)), self.sk(h.max(0)));
        let (kopf, rand) = (self.sk(22), self.sk(8));
        let wi = self.win_mut(win, "GUI_DOCK")?;
        match wi.dock.as_deref_mut() {
            Some(d) => { d.x = x; d.y = y; d.w = w; d.h = h; d.rechnen(); }
            None => { let mut d = DockState::neu(x, y, w, h, kopf, rand); d.rechnen(); wi.dock = Some(Box::new(d)); }
        }
        Ok(())
    }

    /// GUI_DOCK_SET(win, schluessel$, n): kopf, rand, abstand, fuge.
    pub fn dock_set(&mut self, win: i64, key: &str, n: i32) -> Result<(), String> {
        let v = self.sk(n.max(0));
        let d = self.dock_mut(win, "GUI_DOCK_SET")?;
        match key.to_lowercase().as_str() {
            "kopf" | "header" => d.kopf = v.max(12),
            "rand" | "margin" => d.rand = v,
            "abstand" | "gap" => d.abstand = v,
            "fuge" | "seam" => d.fuge = v,
            _ => return Err(format!("GUI_DOCK_SET: unbekannter Schluessel '{}' (kopf, rand, abstand, fuge)", key)),
        }
        d.rechnen();
        Ok(())
    }

    fn dock_neue_leiste(&mut self, win: i64, name: &str, titel: &str, f: &str) -> Result<usize, String> {
        if name.trim().is_empty() { return Err(format!("{}: die Leiste braucht einen Namen", f)); }
        let d = self.dock_mut(win, f)?;
        let n = name.to_lowercase();
        if d.leiste_nr(&n).is_some() { return Err(format!("{}: eine Leiste '{}' gibt es schon", f, name)); }
        d.leisten.push(Leiste { name: n, titel: titel.to_string(), zu: false, schliessbar: false, fenster: -1, heim: 0,
                                rect: (0, 0, 0, 0), sichtbar: false, schwebt: -1 });
        Ok(d.leisten.len() - 1)
    }

    /// GUI_DOCK_PANEL(win, name$, titel$, bereich$): eine Leiste als neue
    /// Gruppe am Ende des Bereichs (links, rechts, unten, schwebend).
    pub fn dock_panel(&mut self, win: i64, name: &str, titel: &str, bereich: &str) -> Result<(), String> {
        let f = "GUI_DOCK_PANEL";
        let ziel = match bereich.to_lowercase().as_str() {
            "schwebend" | "floating" => None,
            b => Some(bereich_nr(b).ok_or_else(|| format!("{}: unbekannter Bereich '{}' (links, rechts, unten, schwebend)", f, bereich))?),
        };
        let li = self.dock_neue_leiste(win, name, titel, f)?;
        let d = self.dock_mut(win, f)?;
        match ziel {
            Some(b) => d.einsetzen(li, Ziel::Rand(b)),
            None => { let (x, y) = (d.mitte.0 + 40, d.mitte.1 + 40); d.einsetzen(li, Ziel::Schweben(x, y)); }
        }
        if d.vorgemerkt.is_some() { d.nachtragen(li); }
        d.rechnen();
        Ok(())
    }

    /// GUI_DOCK_TAB(win, name$, titel$, neben$): als Reiter in die Gruppe der
    /// Leiste `neben$`.
    pub fn dock_tab(&mut self, win: i64, name: &str, titel: &str, neben: &str) -> Result<(), String> {
        let f = "GUI_DOCK_TAB";
        let anker = self.dock_leiste(win, neben, f)?;
        let li = self.dock_neue_leiste(win, name, titel, f)?;
        let d = self.dock_mut(win, f)?;
        d.einsetzen(li, Ziel::Reiter(anker));
        // Ein neuer Reiter kommt nicht nach vorn -- die Leiste, die schon da
        // war, bleibt zu sehen.
        let o = d.ort_von(anker).unwrap();
        d.gruppe_mut(o).aktiv = anker;
        if d.vorgemerkt.is_some() { d.nachtragen(li); }
        d.rechnen();
        Ok(())
    }

    /// GUI_DOCK_ADD(win, name$, wdg): das Widget gehoert zur Leiste.
    pub fn dock_add(&mut self, win: i64, name: &str, wdg: i64) -> Result<(), String> {
        let f = "GUI_DOCK_ADD";
        let li = self.dock_leiste(win, name, f)?;
        let (ww, _) = Self::dec_widget(wdg);
        if ww != win as usize { return Err(format!("{}: das Widget gehoert zu einem anderen Fenster als das Dock", f)); }
        self.wdg_mut(wdg, f)?.dock_von = li as i32;
        Ok(())
    }

    /// GUI_DOCK_WINDOW(win, name$, fenster): ein ganzes Fenster als Inhalt.
    pub fn dock_window(&mut self, win: i64, name: &str, kind: i64) -> Result<(), String> {
        let f = "GUI_DOCK_WINDOW";
        let li = self.dock_leiste(win, name, f)?;
        if kind == win { return Err(format!("{}: ein Fenster kann nicht in sein eigenes Dock", f)); }
        {
            let k = self.win_mut(kind, f)?;
            if k.dock.is_some() { return Err(format!("{}: ein Fenster mit eigenem Dock kann nicht angedockt werden", f)); }
            k.chrome = false;
            k.movable = false;
            k.dock_host = win as i32;
            if k.base_w <= 0 { k.base_w = k.w; }
            if k.base_h <= 0 { k.base_h = k.h; }
        }
        let d = self.dock_mut(win, f)?;
        d.leisten[li].fenster = kind;
        d.rechnen();
        self.dock_fenster_setzen(win as usize);
        Ok(())
    }

    pub fn dock_rect(&self, win: i64, name: &str) -> Result<(i32, i32, i32, i32), String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_RECT")?;
        let l = &self.dock_ref(win, "GUI_DOCK_RECT")?.leisten[li];
        if !l.sichtbar { return Ok((0, 0, 0, 0)); }
        Ok((self.unsk(l.rect.0), self.unsk(l.rect.1), self.unsk(l.rect.2), self.unsk(l.rect.3)))
    }
    /// GUI_DOCK_TAB_RECT(win, name$): wo der Reiter der Leiste steht (auch
    /// eines hinteren); (0, 0, 0, 0), wenn sie ausgeblendet ist.
    pub fn dock_tab_rect(&self, win: i64, name: &str) -> Result<(i32, i32, i32, i32), String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_TAB_RECT")?;
        let d = self.dock_ref(win, "GUI_DOCK_TAB_RECT")?;
        for g in &d.geom {
            if let Some(&(_, x0, x1)) = g.tabs.iter().find(|t| t.0 == li) {
                return Ok((self.unsk(x0), self.unsk(g.r.1), self.unsk(x1 - x0), self.unsk(d.kopf)));
            }
        }
        Ok((0, 0, 0, 0))
    }
    /// GUI_DOCK_HEAD_RECT(win, name$): der freie Teil des Kopfs der Gruppe,
    /// in der die Leiste steht -- rechts neben dem letzten Reiter. Dort legt
    /// ein Programm Filterfeld, Zaehler oder Knopf der Leiste hin.
    pub fn dock_head_rect(&self, win: i64, name: &str) -> Result<(i32, i32, i32, i32), String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_HEAD_RECT")?;
        let d = self.dock_ref(win, "GUI_DOCK_HEAD_RECT")?;
        for g in &d.geom {
            if g.tabs.iter().any(|t| t.0 == li) {
                let ende = g.tabs.last().map(|t| t.2 + 2).unwrap_or(g.r.0);
                let rechts = match g.ort { Ort::Schwebe(_) => g.r.0 + g.r.2 - 1, _ => g.r.0 + g.r.2 };
                return Ok((self.unsk(ende), self.unsk(g.r.1), self.unsk((rechts - ende).max(0)), self.unsk(d.kopf)));
            }
        }
        Ok((0, 0, 0, 0))
    }
    /// GUI_DOCK_SHARE(win, name$, anteil): der Anteil der Gruppe der Leiste
    /// an ihrem Bereich (verglichen mit den anderen Gruppen dort).
    pub fn dock_share(&mut self, win: i64, name: &str, anteil: f64) -> Result<(), String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_SHARE")?;
        if !(anteil > 0.0) { return Err("GUI_DOCK_SHARE: der Anteil muss groesser als 0 sein".into()); }
        let d = self.dock_mut(win, "GUI_DOCK_SHARE")?;
        if let Some(o) = d.ort_von(li) { d.gruppe_mut(o).anteil = anteil as f32; }
        d.rechnen();
        Ok(())
    }
    pub fn dock_center(&self, win: i64) -> Result<(i32, i32, i32, i32), String> {
        let m = self.dock_ref(win, "GUI_DOCK_CENTER")?.mitte;
        Ok((self.unsk(m.0), self.unsk(m.1), self.unsk(m.2), self.unsk(m.3)))
    }
    pub fn dock_shown(&self, win: i64, name: &str) -> Result<bool, String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_SHOWN")?;
        Ok(self.dock_ref(win, "GUI_DOCK_SHOWN")?.leisten[li].sichtbar)
    }
    /// GUI_DOCK_WHERE$(win, name$): links, rechts, unten, schwebend oder zu.
    pub fn dock_where(&self, win: i64, name: &str) -> Result<String, String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_WHERE$")?;
        let d = self.dock_ref(win, "GUI_DOCK_WHERE$")?;
        if d.leisten[li].zu { return Ok("zu".into()); }
        Ok(match d.ort_von(li) {
            Some(Ort::Bereich(b, _)) => BEREICH_NAMEN[b].into(),
            Some(Ort::Schwebe(_)) => "schwebend".into(),
            None => "zu".into(),
        })
    }
    /// GUI_DOCK_SHOW(win, name$, an): ausblenden bzw. wieder zeigen (dann vorn).
    pub fn dock_show(&mut self, win: i64, name: &str, an: bool) -> Result<(), String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_SHOW")?;
        let d = self.dock_mut(win, "GUI_DOCK_SHOW")?;
        d.leisten[li].zu = !an;
        if an {
            if let Some(o) = d.ort_von(li) {
                d.gruppe_mut(o).aktiv = li;
                if let Ort::Bereich(b, _) = o { d.bereiche[b].sichtbar = true; }
            }
        }
        d.rechnen();
        Ok(())
    }
    /// GUI_DOCK_ACTIVATE(win, name$): den Reiter nach vorn holen.
    pub fn dock_activate(&mut self, win: i64, name: &str) -> Result<(), String> { self.dock_show(win, name, true) }
    pub fn dock_title(&mut self, win: i64, name: &str, titel: &str) -> Result<(), String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_TITLE")?;
        let d = self.dock_mut(win, "GUI_DOCK_TITLE")?;
        d.leisten[li].titel = titel.to_string();
        d.rechnen();
        Ok(())
    }
    pub fn dock_closable(&mut self, win: i64, name: &str, an: bool) -> Result<(), String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_CLOSABLE")?;
        let d = self.dock_mut(win, "GUI_DOCK_CLOSABLE")?;
        d.leisten[li].schliessbar = an;
        d.rechnen();
        Ok(())
    }
    pub fn dock_closed(&self, win: i64) -> Result<String, String> {
        Ok(self.dock_ref(win, "GUI_DOCK_CLOSED$")?.geschlossen.clone())
    }
    pub fn dock_changed(&self, win: i64) -> Result<bool, String> {
        Ok(self.dock_ref(win, "GUI_DOCK_CHANGED")?.geaendert)
    }
    fn bereich_arg(b: &str, f: &str) -> Result<usize, String> {
        bereich_nr(b).ok_or_else(|| format!("{}: unbekannter Bereich '{}' (links, rechts, unten)", f, b))
    }
    pub fn dock_area(&mut self, win: i64, bereich: &str, an: bool) -> Result<(), String> {
        let b = Self::bereich_arg(bereich, "GUI_DOCK_AREA")?;
        let d = self.dock_mut(win, "GUI_DOCK_AREA")?;
        d.bereiche[b].sichtbar = an;
        d.rechnen();
        Ok(())
    }
    pub fn dock_area_shown(&self, win: i64, bereich: &str) -> Result<bool, String> {
        let b = Self::bereich_arg(bereich, "GUI_DOCK_AREA_SHOWN")?;
        Ok(self.dock_ref(win, "GUI_DOCK_AREA_SHOWN")?.bereich_an(b))
    }
    pub fn dock_size(&mut self, win: i64, bereich: &str, px: i32) -> Result<(), String> {
        let b = Self::bereich_arg(bereich, "GUI_DOCK_SIZE")?;
        let v = self.sk(px.max(0));
        let d = self.dock_mut(win, "GUI_DOCK_SIZE")?;
        d.bereiche[b].groesse = v.max(MIN_GROESSE);
        d.rechnen();
        Ok(())
    }
    pub fn dock_get_size(&self, win: i64, bereich: &str) -> Result<i32, String> {
        let b = Self::bereich_arg(bereich, "GUI_DOCK_GET_SIZE")?;
        Ok(self.unsk(self.dock_ref(win, "GUI_DOCK_GET_SIZE")?.bereiche[b].groesse))
    }
    /// GUI_DOCK_FLOAT(win, name$, x, y, w, h): abtrennen (vom Programm aus).
    pub fn dock_float(&mut self, win: i64, name: &str, x: i32, y: i32, w: i32, h: i32) -> Result<(), String> {
        let li = self.dock_leiste(win, name, "GUI_DOCK_FLOAT")?;
        let (x, y, w, h) = (self.sk(x), self.sk(y), self.sk(w).max(MIN_GROESSE), self.sk(h).max(MIN_GROESSE));
        let d = self.dock_mut(win, "GUI_DOCK_FLOAT")?;
        if let Some(Ort::Schwebe(k)) = d.ort_von(li) {
            if d.schweben[k].gruppe.leisten.len() == 1 {
                let s = &mut d.schweben[k];
                s.x = x; s.y = y; s.w = w; s.h = h;
                d.rechnen();
                return Ok(());
            }
        }
        d.herausnehmen(li);
        d.leisten[li].zu = false;
        d.schweben.push(Schwebe { gruppe: Gruppe { leisten: vec![li], aktiv: li, anteil: 1.0 }, x, y, w, h });
        d.rechnen();
        Ok(())
    }
    /// GUI_DOCK_MOVE(win, name$, bereich$ [, neben$]): andocken -- als neue
    /// Gruppe am Ende des Bereichs, oder als Reiter neben `neben$`.
    pub fn dock_move(&mut self, win: i64, name: &str, bereich: &str, neben: &str) -> Result<(), String> {
        let f = "GUI_DOCK_MOVE";
        let li = self.dock_leiste(win, name, f)?;
        let ziel = if neben.is_empty() { Ziel::Rand(Self::bereich_arg(bereich, f)?) } else { Ziel::Reiter(self.dock_leiste(win, neben, f)?) };
        let d = self.dock_mut(win, f)?;
        d.einsetzen(li, ziel);
        d.rechnen();
        Ok(())
    }
    pub fn dock_state(&self, win: i64) -> Result<String, String> {
        let d = self.dock_ref(win, "GUI_DOCK_STATE$")?;
        let unsk = |n: i32| self.unsk(n);
        Ok(d.stand(&unsk).to_string())
    }
    pub fn dock_set_state(&mut self, win: i64, s: &str) -> Result<(), String> {
        let v: serde_json::Value = serde_json::from_str(s).map_err(|e| format!("GUI_DOCK_SET_STATE: ungueltiges JSON: {}", e))?;
        let faktor = self.scale;
        let sk = move |n: i32| (n as f64 * faktor).round() as i32;
        let d = self.dock_mut(win, "GUI_DOCK_SET_STATE")?;
        d.stand_setzen(&v, &sk)?;
        d.rechnen();
        Ok(())
    }

    // --- je Bild ---------------------------------------------------------

    /// In GUI_UPDATE: Reitertitel messen, rechnen, angedockte Fenster setzen,
    /// Fensterreihenfolge halten, GUI_DOCK_CHANGED setzen.
    pub(super) fn dock_pass(&mut self, g: &Graphics) {
        for wi in 0..self.windows.len() {
            if self.windows[wi].dock.is_none() || !self.windows[wi].alive { continue; }
            let titel: Vec<String> = self.windows[wi].dock.as_ref().unwrap().leisten.iter()
                .map(|l| l.titel.clone()).filter(|t| !self.windows[wi].dock.as_ref().unwrap().tab_b.contains_key(t)).collect();
            let gemessen: Vec<(String, i32)> = titel.into_iter().map(|t| { let b = self.ctext_width(g, &t); (t, b) }).collect();
            let d = self.windows[wi].dock.as_deref_mut().unwrap();
            for (t, b) in gemessen { d.tab_b.insert(t, b); }
            d.rechnen();
            let sig = d.unterschrift();
            if sig != d.sig { d.geaendert = !d.sig.is_empty() || d.geaendert; d.sig = sig; }
            self.dock_fenster_setzen(wi);
        }
        self.dock_ordnen();
    }

    /// Die angedockten Fenster auf ihre Leisten setzen.
    pub(super) fn dock_fenster_setzen(&mut self, wi: usize) {
        let (hx, hy) = (self.windows[wi].x, self.windows[wi].y + self.content_top(wi));
        let gast_sichtbar = self.windows[wi].visible;
        let auftraege: Vec<(i64, bool, (i32, i32, i32, i32))> = match self.windows[wi].dock.as_ref() {
            Some(d) => d.leisten.iter().filter(|l| l.fenster >= 0).map(|l| (l.fenster, l.sichtbar, l.rect)).collect(),
            None => return,
        };
        for (k, sichtbar, (x, y, w, h)) in auftraege {
            let ki = k as usize;
            if ki >= self.windows.len() || !self.windows[ki].alive { continue; }
            let kw = &mut self.windows[ki];
            kw.visible = sichtbar && gast_sichtbar;
            if !kw.visible { continue; }
            let (nx, ny) = (hx + x, hy + y);
            let neu = (kw.w, kw.h) != (w, h);
            kw.x = nx; kw.y = ny; kw.w = w; kw.h = h;
            if neu { self.relayout(ki); }
        }
    }

    /// Angedockte Fenster stehen in der Reihenfolge direkt ueber ihrem
    /// Gastfenster -- sonst laege das Gastfenster nach einem Klick darauf
    /// ueber seinen eigenen Leisten.
    pub(super) fn dock_ordnen(&mut self) {
        let kinder = |s: &Gui, h: usize| -> Vec<usize> {
            s.windows.iter().enumerate().filter(|(_, w)| w.alive && w.dock_host == h as i32).map(|(i, _)| i).collect()
        };
        if !self.windows.iter().any(|w| w.dock_host >= 0) { return; }
        let mut neu: Vec<usize> = Vec::with_capacity(self.z_order.len());
        for &wi in &self.z_order {
            if self.windows[wi].dock_host >= 0 && self.windows.get(self.windows[wi].dock_host as usize).map(|h| h.alive).unwrap_or(false) { continue; }
            neu.push(wi);
            let mut k = kinder(self, wi);
            // schwebende ueber angedockte
            if let Some(d) = self.windows[wi].dock.as_ref() {
                k.sort_by_key(|&c| d.leisten.iter().find(|l| l.fenster == c as i64).map(|l| l.schwebt).unwrap_or(-1));
            }
            for c in k { if self.z_order.contains(&c) { neu.push(c); } }
        }
        self.z_order = neu;
    }

    /// Ist der Punkt (Bildschirm) fuer dieses Widget frei -- oder liegt eine
    /// schwebende Leiste darueber, zu der es nicht gehoert?
    pub(super) fn dock_frei(&self, wi: usize, w: &Widget, mx: i32, my: i32) -> bool {
        let d = match self.windows[wi].dock.as_ref() { Some(d) if !d.schweben.is_empty() => d, _ => return true };
        let (cx, cy) = (mx - self.windows[wi].x, my - self.windows[wi].y - self.content_top(wi));
        let eigen = if w.dock_von >= 0 { d.leisten.get(w.dock_von as usize).map(|l| l.schwebt).unwrap_or(-1) } else { -1 };
        match d.schwebe_unter(cx, cy) {
            Some(gi) => match d.geom[gi].ort { Ort::Schwebe(k) => k as i32 <= eigen, _ => true },
            None => true,
        }
    }

    /// Ein Druck auf das Dock (Kopf, Reiter, Kreuz, Trennlinie, Griff) --
    /// TRUE, wenn er dem Dock gehoerte.
    pub(super) fn dock_druck(&mut self, wi: usize, mx: i32, my: i32) -> bool {
        let (cx, cy) = (mx - self.windows[wi].x, my - self.windows[wi].y - self.content_top(wi));
        let dbl = self.dbl_click;
        let d = match self.windows[wi].dock.as_deref_mut() { Some(d) => d, None => return false };
        // Schwebende Gruppen liegen oben: ihr Griff und ihr Kopf zuerst.
        if let Some(gi) = d.schwebe_unter(cx, cy) {
            let (gx, gy, gw, gh) = d.geom[gi].r;
            let k = match d.geom[gi].ort { Ort::Schwebe(k) => k, _ => unreachable!() };
            if cx >= gx + gw - 12 && cy >= gy + gh - 12 {
                d.zug = Some(Zug::Groesse { nr: k, dx: gx + gw - cx, dy: gy + gh - cy });
                return true;
            }
            // nach vorn
            if k + 1 != d.schweben.len() {
                let s = d.schweben.remove(k);
                d.schweben.push(s);
                d.rechnen();
                return self.dock_druck(wi, mx, my);
            }
        }
        if let Some(gi) = d.kopf_unter(cx, cy) {
            let (ort, (gx, gy, _, _)) = (d.geom[gi].ort, d.geom[gi].r);
            let tab = d.geom[gi].tabs.iter().find(|&&(_, x0, x1)| cx >= x0 && cx < x1).copied();
            match tab {
                Some((li, _, x1)) => {
                    if d.leisten[li].schliessbar {
                        let (kx, ky, k) = d.kreuz_rect(gy, x1);
                        if cx >= kx - 3 && cx < kx + k + 3 && cy >= ky - 3 && cy < ky + k + 3 {
                            d.leisten[li].zu = true;
                            d.geschlossen = d.leisten[li].name.clone();
                            d.rechnen();
                            return true;
                        }
                    }
                    d.gruppe_mut(ort).aktiv = li;
                    d.rechnen();
                    d.zug = Some(Zug::Leiste { leiste: li, x0: cx, y0: cy, aktiv: false, ziel: None });
                    return true;
                }
                None => {
                    if let Ort::Schwebe(k) = ort {
                        if dbl { d.heimholen(k); d.rechnen(); return true; }
                        d.zug = Some(Zug::Schwebe { nr: k, dx: cx - gx, dy: cy - gy });
                        return true;
                    }
                    // Freie Flaeche im Kopf einer angedockten Gruppe: gehoert
                    // dem Programm (es legt dort etwa ein Filterfeld hin).
                    return false;
                }
            }
        }
        if d.schwebe_unter(cx, cy).is_some() { return false; }
        for t in &d.trenner {
            let (tx, ty, tw, th) = t.r;
            if cx >= tx && cx < tx + tw && cy >= ty && cy < ty + th {
                let m = if t.senkrecht { cx } else { cy };
                let z = d.trenner_anfang(t.art, m);
                d.zug = Some(z);
                return true;
            }
        }
        false
    }

    /// Einen laufenden Zug am Dock weiterfuehren (jedes Bild, solange die
    /// Taste haengt) und beim Loslassen abschliessen.
    pub(super) fn dock_zug_pass(&mut self, is_down: bool, mx: i32, my: i32) {
        for wi in 0..self.windows.len() {
            let (ox, oy) = (self.windows[wi].x, self.windows[wi].y + self.content_top(wi));
            let d = match self.windows[wi].dock.as_deref_mut() { Some(d) if d.zug.is_some() => d, _ => continue };
            let (cx, cy) = (mx - ox, my - oy);
            let zug = d.zug.take().unwrap();
            d.zug = match zug {
                Zug::Trenner { art, m0, a0, b0 } => {
                    let senk = d.trenner.iter().find(|t| t.art == art).map(|t| t.senkrecht).unwrap_or(true);
                    d.trenner_ziehen(art, m0, a0, b0, if senk { cx } else { cy });
                    d.rechnen();
                    if is_down { Some(Zug::Trenner { art, m0, a0, b0 }) } else { None }
                }
                Zug::Schwebe { nr, dx, dy } => {
                    if nr < d.schweben.len() {
                        let s = &mut d.schweben[nr];
                        s.x = (cx - dx).clamp(d.x - s.w + 40, d.x + d.w - 40);
                        s.y = (cy - dy).clamp(d.y - 2, d.y + d.h - 20);
                        d.rechnen();
                    }
                    if is_down { Some(Zug::Schwebe { nr, dx, dy }) } else { None }
                }
                Zug::Groesse { nr, dx, dy } => {
                    if nr < d.schweben.len() {
                        let s = &mut d.schweben[nr];
                        s.w = (cx + dx - s.x).max(MIN_GROESSE * 2);
                        s.h = (cy + dy - s.y).max(d.kopf + MIN_GROESSE);
                        d.rechnen();
                    }
                    if is_down { Some(Zug::Groesse { nr, dx, dy }) } else { None }
                }
                Zug::Leiste { leiste, x0, y0, aktiv, ziel } => {
                    let aktiv = aktiv || (cx - x0).abs() >= ZUG_SCHWELLE || (cy - y0).abs() >= ZUG_SCHWELLE;
                    let ziel = if aktiv { Some(d.ziel_unter(leiste, cx, cy)) } else { ziel };
                    if is_down { Some(Zug::Leiste { leiste, x0, y0, aktiv, ziel }) } else {
                        if let (true, Some(z)) = (aktiv, ziel) { d.einsetzen(leiste, z); d.rechnen(); }
                        None
                    }
                }
            };
        }
    }

    /// Die Form des Mauszeigers ueber dem Dock (Trennlinie, Griff, Zug).
    pub(super) fn dock_zeiger(&self, mx: i32, my: i32) -> Option<&'static str> {
        for (wi, w) in self.windows.iter().enumerate() {
            let d = match w.dock.as_ref() { Some(d) if w.alive && w.visible => d, _ => continue };
            match &d.zug {
                Some(Zug::Trenner { art, .. }) => {
                    let senk = d.trenner.iter().find(|t| t.art == *art).map(|t| t.senkrecht).unwrap_or(true);
                    return Some(if senk { "resize_ew" } else { "resize_ns" });
                }
                Some(Zug::Groesse { .. }) => return Some("resize_nwse"),
                Some(Zug::Schwebe { .. }) => return Some("resize_all"),
                Some(Zug::Leiste { aktiv: true, .. }) => return Some("resize_all"),
                _ => {}
            }
            if self.topmost_at(mx, my) != Some(wi) { continue; }
            let (cx, cy) = (mx - w.x, my - w.y - self.content_top(wi));
            if let Some(gi) = d.schwebe_unter(cx, cy) {
                let (gx, gy, gw, gh) = d.geom[gi].r;
                if cx >= gx + gw - 12 && cy >= gy + gh - 12 { return Some("resize_nwse"); }
                return None;
            }
            for t in &d.trenner {
                let (tx, ty, tw, th) = t.r;
                if cx >= tx && cx < tx + tw && cy >= ty && cy < ty + th {
                    return Some(if t.senkrecht { "resize_ew" } else { "resize_ns" });
                }
            }
        }
        None
    }

    // --- Zeichnen --------------------------------------------------------

    fn dock_kopf_zeichnen(&self, g: &mut Graphics, d: &DockState, gg: &GGeom, ox: i32, oy: i32, schwebend: bool) {
        let (gx, gy, gw, _) = gg.r;
        let (x, y) = (ox + gx, oy + gy);
        let kopf = d.kopf;
        if schwebend {
            g.box_fill(x + 1, y + 1, x + gw - 2, y + kopf - 1, self.th("title_bg"));
        }
        g.line(x, y + kopf - 1, x + gw - 1, y + kopf - 1, self.th("win_border"));
        let aktiv = d.gruppe(gg.ort).aktiv;
        let th14 = self.ctext_height(g);
        for &(li, x0, x1) in &gg.tabs {
            let l = &d.leisten[li];
            let vorn = li == aktiv;
            if vorn {
                g.box_fill(ox + x0, y + 2, ox + x1 - 1, y + kopf - 2, self.th("widget_bg"));
                g.box_fill(ox + x0, y + kopf - 3, ox + x1 - 1, y + kopf - 2, self.th("accent"));
            }
            let fg = if vorn { self.th("text_fg") } else { self.th("muted_fg") };
            self.ctext(g, ox + x0 + 8, y + (kopf - th14) / 2, l.titel.clone(), fg);
            if l.schliessbar {
                let (kx, ky, k) = d.kreuz_rect(gy, x1);
                g.line(ox + kx, oy + ky, ox + kx + k - 1, oy + ky + k - 1, fg);
                g.line(ox + kx + k - 1, oy + ky, ox + kx, oy + ky + k - 1, fg);
            }
        }
    }

    /// Die Koepfe der angedockten Gruppen -- vor den Widgets, damit ein
    /// Programm in den freien Teil eines Kopfs eigene Widgets legen kann.
    pub(super) fn dock_zeichnen_unten(&self, g: &mut Graphics, wi: usize) {
        let d = match self.windows[wi].dock.as_ref() { Some(d) => d, None => return };
        let (ox, oy) = (self.windows[wi].x, self.windows[wi].y + self.content_top(wi));
        for gg in d.geom.iter().filter(|gg| matches!(gg.ort, Ort::Bereich(..))) {
            self.dock_kopf_zeichnen(g, d, gg, ox, oy, false);
        }
        // Waehrend eine Trennlinie gezogen wird, zeigt sie sich.
        if let Some(Zug::Trenner { art, .. }) = &d.zug {
            if let Some(t) = d.trenner.iter().find(|t| t.art == *art) {
                let (tx, ty, tw, th) = t.r;
                let c = with_alpha(self.th("accent"), 0x90);
                if t.senkrecht { let mx = ox + tx + tw / 2; g.box_fill(mx - 1, oy + ty, mx, oy + ty + th - 1, c); }
                else { let my = oy + ty + th / 2; g.box_fill(ox + tx, my - 1, ox + tx + tw - 1, my, c); }
            }
        }
    }

    /// Schwebende Gruppen samt ihrer Widgets, dann der Hinweis eines Zugs --
    /// nach allen anderen Widgets, also obenauf.
    pub(super) fn dock_zeichnen_oben(&self, g: &mut Graphics, wi: usize) {
        let d = match self.windows[wi].dock.as_ref() { Some(d) => d, None => return };
        let (ox, oy) = (self.windows[wi].x, self.windows[wi].y + self.content_top(wi));
        for gg in d.geom.iter().filter(|gg| matches!(gg.ort, Ort::Schwebe(_))) {
            let (gx, gy, gw, gh) = gg.r;
            let (x, y) = (ox + gx, oy + gy);
            for k in 0..3 {
                let off = 2 + k * 2;
                g.box_fill(x + off, y + off, x + gw - 1 + off, y + gh - 1 + off, ((0x30 - k * 0x0C) as i64) << 24);
            }
            g.box_fill(x, y, x + gw - 1, y + gh - 1, self.th("win_bg"));
            g.rect(x, y, x + gw - 1, y + gh - 1, self.th("win_border"));
            self.dock_kopf_zeichnen(g, d, gg, ox, oy, true);
            let c = self.th("win_border");
            for o in [2, 6, 10] { g.line(x + gw - 2 - o, y + gh - 2, x + gw - 2, y + gh - 2 - o, c); }
            let aktiv = d.gruppe(gg.ort).aktiv;
            if d.leisten[aktiv].fenster < 0 {
                let (rx, ry, rw, rh) = d.leisten[aktiv].rect;
                g.push_clip(ox + rx, oy + ry, rw.max(0), rh.max(0));
                for (i, wdg) in self.windows[wi].widgets.iter().enumerate() {
                    if wdg.dock_von == aktiv as i32 && self.widget_shown(wi, wdg) { self.draw_widget(g, wi, i, wdg); }
                }
                g.pop_clip();
            }
        }
        if let Some(Zug::Leiste { leiste, aktiv: true, ziel: Some(z), .. }) = &d.zug {
            let (zx, zy, zw, zh) = d.ziel_rect(*z);
            let c = self.th("accent");
            g.box_fill(ox + zx, oy + zy, ox + zx + zw - 1, oy + zy + zh - 1, with_alpha(c, 0x50));
            g.rect(ox + zx, oy + zy, ox + zx + zw - 1, oy + zy + zh - 1, c);
            let _ = leiste;
        }
    }
}

/// Eine Farbe mit Deckkraft (1..255; 0 hiesse deckend).
fn with_alpha(c: i64, a: i64) -> i64 { (c & 0xFF_FFFF) | (a.clamp(1, 255) << 24) }

#[cfg(test)]
mod tests {
    use super::*;

    fn dock() -> DockState {
        let mut d = DockState::neu(0, 0, 1000, 600, 22, 8);
        for (n, b) in [("projekt", 0usize), ("gliederung", 0), ("ausgabe", 2), ("probleme", 2)] {
            d.leisten.push(Leiste { name: n.into(), titel: n.into(), zu: false, schliessbar: false, fenster: -1, heim: b,
                                    rect: (0, 0, 0, 0), sichtbar: false, schwebt: -1 });
            let li = d.leisten.len() - 1;
            d.einsetzen(li, Ziel::Rand(b));
        }
        d.bereiche[0].groesse = 240;
        d.bereiche[2].groesse = 202;
        d.rechnen();
        d
    }

    #[test]
    fn die_vorgabe_teilt_wie_die_ide() {
        let d = dock();
        // links: Rand 8, Breite 240 - 16
        assert_eq!(d.leisten[0].rect.0, 8);
        assert_eq!(d.leisten[0].rect.2, 224);
        assert_eq!(d.leisten[0].rect.1, 22);
        // unten: zwei Haelften mit Rand 8 und Abstand 8
        let (a, p) = (d.leisten[2].rect, d.leisten[3].rect);
        assert_eq!(a.0, 8);
        assert_eq!(p.0, 8 + a.2 + 8);
        assert_eq!(p.0 + p.2, 1000 - 8);
        // die Mitte beginnt hinter dem linken Bereich und endet ueber dem unteren
        assert_eq!(d.mitte.0, 240);
        assert_eq!(d.mitte.3, 600 - 202 - 2);
    }

    #[test]
    fn reiter_teilen_schweben_und_zurueck() {
        let mut d = dock();
        // gliederung als Reiter zu projekt
        d.einsetzen(1, Ziel::Reiter(0));
        d.rechnen();
        assert_eq!(d.bereiche[0].gruppen.len(), 1);
        assert!(d.leisten[1].sichtbar && !d.leisten[0].sichtbar);
        // probleme nach rechts
        d.einsetzen(3, Ziel::Rand(1));
        d.rechnen();
        assert!(d.bereich_an(1));
        assert_eq!(d.bereiche[2].gruppen.len(), 1);
        // ausgabe abtrennen und heimholen
        d.einsetzen(2, Ziel::Schweben(300, 100));
        d.rechnen();
        assert_eq!(d.leisten[2].schwebt, 0);
        assert!(!d.bereich_an(2));
        d.heimholen(0);
        d.rechnen();
        assert!(d.bereich_an(2) && d.leisten[2].schwebt == -1);
        // auf sich selbst fallen lassen aendert nichts
        let vorher = d.unterschrift();
        d.einsetzen(2, Ziel::Teilen(2, true));
        d.rechnen();
        assert_eq!(d.unterschrift(), vorher);
    }

    #[test]
    fn der_stand_geht_hin_und_zurueck() {
        let mut d = dock();
        d.einsetzen(3, Ziel::Rand(1));
        d.einsetzen(2, Ziel::Schweben(300, 100));
        d.leisten[1].zu = true;
        d.rechnen();
        let id = |n: i32| n;
        let stand = d.stand(&id);
        let mut e = dock();
        e.stand_setzen(&stand, &id).unwrap();
        e.rechnen();
        assert_eq!(e.unterschrift(), d.unterschrift());
        // eine spaeter angemeldete Leiste findet ihren Platz
        let mut f = DockState::neu(0, 0, 1000, 600, 22, 8);
        f.stand_setzen(&stand, &id).unwrap();
        f.leisten.push(Leiste { name: "probleme".into(), titel: "p".into(), zu: false, schliessbar: false, fenster: -1, heim: 0,
                                rect: (0, 0, 0, 0), sichtbar: false, schwebt: -1 });
        f.einsetzen(0, Ziel::Rand(2));
        f.nachtragen(0);
        assert_eq!(f.ort_von(0), Some(Ort::Bereich(1, 0)));
    }

    #[test]
    fn trennlinien_haben_grenzen() {
        let mut d = dock();
        d.trenner_ziehen(TArt::Bereich(0), 0, 240, 0, -1000);
        assert_eq!(d.bereiche[0].groesse, MIN_GROESSE);
        let z = d.trenner_anfang(TArt::Gruppe(0, 0, 1), 0);
        if let Zug::Trenner { art, m0, a0, b0 } = z {
            d.trenner_ziehen(art, m0, a0, b0, 10000);
            d.rechnen();
            let unten = d.leisten[1].rect.3 + d.kopf;
            assert!(unten >= MIN_GROESSE, "die untere Gruppe behaelt ihr Mindestmass: {}", unten);
        } else { panic!() }
    }
}
