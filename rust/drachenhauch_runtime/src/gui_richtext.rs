//! Gesetzter Text (GUI_RICHTEXT).
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Gesetzter Text (GUI_RICHTEXT) --------------------------------------
    /// Ein Widget, das Markdown SETZT. Bis Stand 25 blieb einem Programm nur,
    /// den Text selbst auf eine Zeichenflaeche zu malen -- die IDE tat das
    /// mit 180 Zeilen und konnte dabei weder fett noch Tabellenspalten noch
    /// anklickbare Verweise.
    pub fn richtext(&mut self, win: i64, x: i32, y: i32, w: i32, h: i32, text: String) -> Result<i64, String> {
        let mut wd = Self::blank(Kind::RichText, x, y, w, h);
        wd.rich = Some(Box::new(RichState { quelle: text, code_font: -1, ..Default::default() }));
        self.add_widget(win, "GUI_RICHTEXT", wd)
    }
    pub(super) fn rt_mut(&mut self, h: i64, fn_: &str) -> Result<&mut RichState, String> {
        let w = self.wdg_mut(h, fn_)?;
        if w.kind != Kind::RichText { return Err(format!("{}: Widget ist kein gesetzter Text (GUI_RICHTEXT)", fn_)); }
        Ok(w.rich.as_mut().unwrap())
    }
    pub(super) fn rt_ref(&self, h: i64, fn_: &str) -> Result<&RichState, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::RichText { return Err(format!("{}: Widget ist kein gesetzter Text (GUI_RICHTEXT)", fn_)); }
        Ok(w.rich.as_ref().unwrap())
    }
    pub fn richtext_set_text(&mut self, h: i64, text: String) -> Result<(), String> {
        let r = self.rt_mut(h, "GUI_RICHTEXT_SET_TEXT")?;
        r.quelle = text;
        r.scroll = 0;
        r.hat_auswahl = false;
        r.ziehen = false;
        // Der Satz gehoert zum ALTEN Text -- `stand` zuruecksetzen heisst
        // "beim naechsten GUI_UPDATE neu setzen".
        r.stand = (usize::MAX, 0, 0, 0);
        r.satz_aktuell = false;
        r.ziel_ue = None;
        r.ziel_suche = None;
        Ok(())
    }
    pub fn richtext_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let r = self.rt_mut(h, "GUI_RICHTEXT_SET")?;
        const ARTEN: [&str; 6] = ["farbe_kommentar", "farbe_text", "farbe_zahl",
                                  "farbe_schluessel", "farbe_name", "farbe_operator"];
        let k = key.to_lowercase();
        match k.as_str() {
            "groesse" | "size" => { r.basis = wert.max(0.0) as i32; r.stand = (usize::MAX, 0, 0, 0); }
            "codeschrift" | "code_font" => { r.code_font = wert as i64; r.stand = (usize::MAX, 0, 0, 0); }
            _ if ARTEN.contains(&k.as_str()) => {
                let n = ARTEN.iter().position(|a| *a == k).unwrap();
                r.code_farben[n] = if wert < 0.0 { None } else { Some(wert as i64) };
            }
            _ => return Err(format!(
                "GUI_RICHTEXT_SET: unbekannte Einstellung '{}' (gueltig: groesse, codeschrift, {})", key, ARTEN.join(", "))),
        }
        Ok(())
    }
    /// Knoepfe an jedem Codeblock ("Kopieren|Starten", leer = keine).
    pub fn richtext_code_buttons(&mut self, h: i64, knoepfe: &str) -> Result<(), String> {
        let r = self.rt_mut(h, "GUI_RICHTEXT_CODE_BUTTONS")?;
        r.code_knoepfe = knoepfe.split('|').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect();
        r.stand = (usize::MAX, 0, 0, 0);
        r.satz_aktuell = false;
        Ok(())
    }
    /// Welcher Code-Knopf in diesem Bild gedrueckt wurde (leer = keiner).
    pub fn richtext_code_action(&self, h: i64) -> Result<String, String> {
        Ok(self.rt_ref(h, "GUI_RICHTEXT_CODE_ACTION$")?.code_aktion.clone())
    }
    /// Der Code des Blocks, an dem zuletzt ein Knopf gedrueckt wurde.
    pub fn richtext_code(&self, h: i64) -> Result<String, String> {
        Ok(self.rt_ref(h, "GUI_RICHTEXT_CODE$")?.code_text.clone())
    }
    /// Die Ueberschriften des Dokuments (Titel ohne Auszeichnung). Aus der
    /// QUELLE gelesen, damit sie gleich nach GUI_RICHTEXT_SET_TEXT stimmen
    /// -- gesetzt wird erst beim naechsten GUI_UPDATE.
    pub fn richtext_headings(&self, h: i64) -> Result<Vec<(u8, String)>, String> {
        Ok(rt_ueberschriften_quelle(&self.rt_ref(h, "GUI_RICHTEXT_HEADINGS")?.quelle))
    }
    /// Zur Ueberschrift Nummer `k` rollen. Ist der Text noch nicht gesetzt,
    /// geschieht es beim naechsten GUI_UPDATE.
    pub fn richtext_goto_heading(&mut self, h: i64, k: i64) -> Result<(), String> {
        let hoehe = self.wdg(h, "GUI_RICHTEXT_GOTO_HEADING")?.h;
        let rand = self.sk(6);
        let r = self.rt_mut(h, "GUI_RICHTEXT_GOTO_HEADING")?;
        if k < 0 { return Ok(()); }
        let k = k as usize;
        if r.satz_aktuell {
            if let Some(u) = r.ueberschriften.get(k) {
                let max = (r.inhalt_h - hoehe).max(0);
                r.scroll = (u.2 - rand).clamp(0, max);
            }
        } else {
            r.ziel_ue = Some(k);
        }
        Ok(())
    }
    /// Die Ueberschrift, in deren Abschnitt der obere Rand gerade steht
    /// (-1 = davor). Fuer ein Inhaltsverzeichnis, das mitgeht.
    pub fn richtext_heading_at(&self, h: i64) -> Result<i64, String> {
        let r = self.rt_ref(h, "GUI_RICHTEXT_HEADING_AT")?;
        if !r.satz_aktuell { return Ok(-1); }
        let oben = r.scroll + self.sk(40);
        let mut k = -1i64;
        for (i, u) in r.ueberschriften.iter().enumerate() {
            if u.2 <= oben { k = i as i64; } else { break; }
        }
        Ok(k)
    }
    pub fn richtext_scroll(&mut self, h: i64, y: i64) -> Result<(), String> {
        let hoehe = self.wdg(h, "GUI_RICHTEXT_SCROLL")?.h;
        let r = self.rt_mut(h, "GUI_RICHTEXT_SCROLL")?;
        // Vor dem Satz ist die Hoehe noch die des alten Texts: dann gilt
        // der Wert, und der Satz klemmt ihn.
        if !r.satz_aktuell { r.scroll = (y as i32).max(0); r.ziel_suche = None; r.ziel_ue = None; return Ok(()); }
        let max = (r.inhalt_h - hoehe).max(0);
        r.scroll = (y as i32).clamp(0, max);
        Ok(())
    }
    pub fn richtext_scroll_get(&self, h: i64) -> Result<i64, String> {
        Ok(self.rt_ref(h, "GUI_RICHTEXT_SCROLL_GET")?.scroll as i64)
    }
    pub fn richtext_height(&self, h: i64) -> Result<i64, String> {
        Ok(self.rt_ref(h, "GUI_RICHTEXT_HEIGHT")?.inhalt_h as i64)
    }
    /// Der in diesem Bild angeklickte Verweis -- transient wie GUI_CLICKED.
    pub fn richtext_link(&self, h: i64) -> Result<String, String> {
        Ok(self.rt_ref(h, "GUI_RICHTEXT_LINK")?.geklickt.clone())
    }
    /// Zur ersten Zeile rollen, die `text` enthaelt (ohne Ruecksicht auf
    /// Gross/klein); liefert deren y oder -1. Ohne das muesste ein Programm,
    /// das im Dokument sucht, den Satz selbst nachrechnen.
    pub fn richtext_find(&mut self, h: i64, text: &str, ab: i64) -> Result<i64, String> {
        let hoehe = self.wdg(h, "GUI_RICHTEXT_FIND")?.h;
        let r = self.rt_mut(h, "GUI_RICHTEXT_FIND")?;
        let nadel = text.to_lowercase();
        if nadel.is_empty() { return Ok(-1); }
        // Direkt nach GUI_RICHTEXT_SET_TEXT gibt es noch keinen Satz des
        // NEUEN Texts -- gesucht wuerde im alten, und gerollt an eine Stelle,
        // die mit dem Dokument nichts zu tun hat. Dann wartet die Suche auf
        // den Satz (0 = vorgemerkt).
        if !r.satz_aktuell { r.ziel_suche = Some((nadel, ab)); return Ok(0); }
        Ok(rt_finden(r, &nadel, ab, hoehe).map(|y| y as i64).unwrap_or(-1))
    }

    /// Die markierte Stelle als Text; Zeilen durch Umbruch getrennt.
    pub fn richtext_selection(&self, h: i64) -> Result<String, String> {
        Ok(Self::rt_auswahl_text(self.rt_ref(h, "GUI_RICHTEXT_SELECTION")?))
    }
    pub fn richtext_select_all(&mut self, h: i64) -> Result<(), String> {
        let r = self.rt_mut(h, "GUI_RICHTEXT_SELECT_ALL")?;
        Self::rt_alles(r);
        Ok(())
    }
    pub fn richtext_clear_selection(&mut self, h: i64) -> Result<(), String> {
        let r = self.rt_mut(h, "GUI_RICHTEXT_CLEAR_SELECTION")?;
        r.hat_auswahl = false; r.ziehen = false;
        Ok(())
    }
    /// Die naechste (bzw. vorige) Fundstelle ab der Auswahl, mit Umlauf am
    /// Ende. Sie wird MARKIERT und ins Bild gerollt; geliefert wird das y
    /// ihrer Zeile oder -1. Gesucht wird im Zeilentext, also auch ueber
    /// Wortgrenzen hinweg ("Gehe zu").
    pub fn richtext_find_step(&mut self, h: i64, text: &str, vorwaerts: bool) -> Result<i64, String> {
        let fn_ = if vorwaerts { "GUI_RICHTEXT_FIND_NEXT" } else { "GUI_RICHTEXT_FIND_PREV" };
        let hoehe = self.wdg(h, fn_)?.h;
        let r = self.rt_mut(h, fn_)?;
        let nadel: Vec<char> = text.to_lowercase().chars().collect();
        if nadel.is_empty() || r.zeilen.is_empty() { return Ok(-1); }
        let texte: Vec<Vec<char>> = r.zeilen.iter().map(|z| Self::rt_zeilentext(z).0.to_lowercase().chars().collect()).collect();
        let mut treffer: Vec<(usize, usize)> = Vec::new();
        for (zi, t) in texte.iter().enumerate() {
            if t.len() < nadel.len() { continue; }
            for c in 0..=(t.len() - nadel.len()) {
                if t[c..c + nadel.len()] == nadel[..] { treffer.push((zi, c)); }
            }
        }
        if treffer.is_empty() { return Ok(-1); }
        let ((a_z, a_c), (e_z, e_c)) = if r.hat_auswahl { Self::rt_bereich(r) } else { ((0, 0), (0, 0)) };
        let wahl = if vorwaerts {
            // Ab dem ANFANG der Auswahl + 1: so kommt man bei einer Fundstelle,
            // die schon markiert ist, zur naechsten -- und ohne Auswahl zur ersten.
            let ab = if r.hat_auswahl { (a_z, a_c + 1) } else { (0, 0) };
            treffer.iter().copied().find(|&t| t >= ab).unwrap_or(treffer[0])
        } else {
            let vor = if r.hat_auswahl { (a_z, a_c) } else { (e_z, e_c) };
            treffer.iter().rev().copied().find(|&t| t < vor || !r.hat_auswahl).unwrap_or(*treffer.last().unwrap())
        };
        r.anker = wahl;
        r.marke = (wahl.0, wahl.1 + nadel.len());
        r.hat_auswahl = true;
        r.ziehen = false;
        let z = &r.zeilen[wahl.0];
        let max = (r.inhalt_h - hoehe).max(0);
        if z.y < r.scroll || z.y + z.h > r.scroll + hoehe {
            r.scroll = (z.y - 20).clamp(0, max);
        }
        Ok(z.y as i64)
    }
    /// Alle Fundstellen hervorheben (leer = keine); liefert ihre Zahl.
    pub fn richtext_mark_all(&mut self, h: i64, text: &str) -> Result<i64, String> {
        let r = self.rt_mut(h, "GUI_RICHTEXT_MARK_ALL")?;
        r.nadel = text.to_lowercase();
        if r.nadel.is_empty() { return Ok(0); }
        let n = r.zeilen.iter().map(|z| Self::rt_zeilentext(z).0.to_lowercase().matches(r.nadel.as_str()).count()).sum::<usize>();
        Ok(n as i64)
    }

    /// Der Text einer gesetzten Zeile und, je Lauf, sein erstes Zeichen darin.
    /// Zwischen zwei Laeufen steht ein Leerzeichen, wenn eine Luecke dazwischen
    /// liegt -- `**fett**bar` ist ein Wort in zwei Gestalten und bleibt eins.
    pub(super) fn rt_zeilentext(z: &RtZeile) -> (String, Vec<usize>) {
        let mut s = String::new();
        let mut starts = Vec::with_capacity(z.laeufe.len());
        let mut n = 0usize;
        let mut ende = i32::MIN;
        for l in z.laeufe.iter() {
            if !s.is_empty() && l.x > ende { s.push(' '); n += 1; }
            starts.push(n);
            s.push_str(&l.text);
            n += l.text.chars().count();
            ende = l.x + l.breite;
        }
        (s, starts)
    }
    /// Anfang und Ende der Auswahl, geordnet.
    pub(super) fn rt_bereich(r: &RichState) -> ((usize, usize), (usize, usize)) {
        if r.anker <= r.marke { (r.anker, r.marke) } else { (r.marke, r.anker) }
    }
    pub(super) fn rt_alles(r: &mut RichState) {
        if r.zeilen.is_empty() { r.hat_auswahl = false; return; }
        let letzte = r.zeilen.len() - 1;
        r.anker = (0, 0);
        r.marke = (letzte, Self::rt_zeilentext(&r.zeilen[letzte]).0.chars().count());
        r.hat_auswahl = true;
        r.ziehen = false;
    }
    pub(super) fn rt_auswahl_text(r: &RichState) -> String {
        if !r.hat_auswahl || r.zeilen.is_empty() { return String::new(); }
        let ((az, ac), (ez, ec)) = Self::rt_bereich(r);
        let mut out: Vec<String> = Vec::new();
        for zi in az..=ez.min(r.zeilen.len() - 1) {
            let t: Vec<char> = Self::rt_zeilentext(&r.zeilen[zi]).0.chars().collect();
            let von = if zi == az { ac.min(t.len()) } else { 0 };
            let bis = if zi == ez { ec.min(t.len()) } else { t.len() };
            // Leerzeilen des Satzes (Abstand unter einer Ueberschrift) sind
            // Luft, kein Text -- kopiert ergaeben sie Luecken an Stellen, an
            // denen man keine sieht.
            if t.is_empty() { continue; }
            out.push(t[von..bis.max(von)].iter().collect());
        }
        out.join("\n")
    }
    /// Welche Stelle liegt unter (x, y) -- relativ zum Inhalt, mit Scroll?
    /// Ueber der ersten Zeile ist es ihr Anfang, unter der letzten ihr Ende;
    /// in einer Zeile das Zeichen, dessen Mitte der Maus am naechsten liegt.
    pub(super) fn rt_stelle_unter(&self, g: &Graphics, w: &Widget, r: &RichState, x: i32, y: i32) -> (usize, usize) {
        if r.zeilen.is_empty() { return (0, 0); }
        let letzte = r.zeilen.len() - 1;
        if y < r.zeilen[0].y { return (0, 0); }
        let zi = match r.zeilen.iter().position(|z| y >= z.y && y < z.y + z.h) {
            Some(zi) => zi,
            None => {
                if y >= r.zeilen[letzte].y + r.zeilen[letzte].h {
                    return (letzte, Self::rt_zeilentext(&r.zeilen[letzte]).0.chars().count());
                }
                // In einer Luecke zwischen zwei Zeilen: die darunter.
                r.zeilen.iter().position(|z| z.y > y).unwrap_or(letzte)
            }
        };
        let z = &r.zeilen[zi];
        let (text, starts) = Self::rt_zeilentext(z);
        let font = self.wfont(g, w);
        let cfont = if r.code_font >= 0 { r.code_font } else { font };
        for (k, l) in z.laeufe.iter().enumerate() {
            if x < l.x { return (zi, starts[k]); }
            if x < l.x + l.breite {
                let f = if l.code { cfont } else { font };
                let zeichen: Vec<char> = l.text.chars().collect();
                let mut vorher = 0;
                for c in 1..=zeichen.len() {
                    let b = g.text_width_stil(&zeichen[..c].iter().collect::<String>(), l.groesse, f, l.stil);
                    if x - l.x < (vorher + b) / 2 { return (zi, starts[k] + c - 1); }
                    vorher = b;
                }
                return (zi, starts[k] + zeichen.len());
            }
        }
        (zi, text.chars().count())
    }
    /// Je Bild: eine laufende Auswahl mit der Maus nachziehen. Hier und nicht
    /// im Druck, weil nur hier Grafik zum Messen greifbar ist.
    pub(super) fn rt_auswahl_pass(&mut self, g: &Graphics, is_down: bool, mx: i32, my: i32) {
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                let (ziehen, anker_neu, doppel) = match self.windows[wi].widgets[i].rich.as_ref() {
                    Some(r) if self.windows[wi].widgets[i].kind == Kind::RichText => (r.ziehen, r.anker_neu, r.doppel),
                    _ => continue,
                };
                if !ziehen { continue; }
                let (ax, ay, _, _) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
                let (anker, marke) = {
                    let w = &self.windows[wi].widgets[i];
                    let r = w.rich.as_ref().unwrap();
                    let anker = if anker_neu { self.rt_stelle_unter(g, w, r, r.druck.0, r.druck.1) } else { r.anker };
                    (anker, self.rt_stelle_unter(g, w, r, mx - ax, my - ay + r.scroll))
                };
                let r = self.windows[wi].widgets[i].rich.as_mut().unwrap();
                r.anker = anker;
                r.marke = marke;
                r.anker_neu = false;
                if doppel {
                    // Doppelklick: das Wort um die Stelle.
                    let t: Vec<char> = Self::rt_zeilentext(&r.zeilen[anker.0.min(r.zeilen.len().saturating_sub(1))]).0.chars().collect();
                    let ist_wort = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
                    let mut von = anker.1.min(t.len());
                    let mut bis = von;
                    while von > 0 && ist_wort(t[von - 1]) { von -= 1; }
                    while bis < t.len() && ist_wort(t[bis]) { bis += 1; }
                    r.anker = (anker.0, von);
                    r.marke = (anker.0, bis);
                    r.hat_auswahl = bis > von;
                    r.ziehen = false;
                    r.doppel = false;
                    continue;
                }
                r.hat_auswahl = r.anker != r.marke;
                if !is_down { r.ziehen = false; }
            }
        }
    }

    /// Je Bild: jeden gesetzten Text neu setzen, dessen Quelle, Breite oder
    /// Schriftgroesse sich geaendert hat. Alles andere bleibt stehen.
    /// Die Schnitte, die gleich gemessen werden, einmal laden: das Messen
    /// laeuft mit `&Graphics` und kann selbst nichts nachladen. Nach dem
    /// ersten Mal ist das ein Blick in den Vorrat.
    pub(super) fn schnitte_laden(&self, g: &mut Graphics) {
        use crate::schnitt::{FETT, KURSIV};
        let aktiv = g.active_font();
        for win in &self.windows {
            for w in &win.widgets {
                if !w.alive { continue; }
                let basis = font_wahl(w.font, aktiv);
                let st = self.w_stil(w);
                if st != 0 { g.schnitt(basis, st); }
                if w.kind == Kind::RichText {
                    for s in [FETT, KURSIV, FETT | KURSIV] { g.schnitt(basis, s); }
                }
            }
        }
    }

    pub(super) fn rt_pass(&mut self, g: &Graphics) {
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                if self.windows[wi].widgets[i].kind != Kind::RichText { continue; }
                let (breite, basis, quell_len) = {
                    let w = &self.windows[wi].widgets[i];
                    let r = w.rich.as_ref().unwrap();
                    let b = if r.basis > 0 { r.basis } else { self.wsize(g, w) };
                    (w.w, b, r.quelle.chars().count())
                };
                let stand = (quell_len, breite, basis, (self.scale * 100.0) as i32);
                if self.windows[wi].widgets[i].rich.as_ref().unwrap().stand == stand { continue; }
                self.rt_setzen(g, wi, i, breite, basis);
                let r = self.windows[wi].widgets[i].rich.as_mut().unwrap();
                r.stand = stand;
                // Die Stellen einer Auswahl zeigen in den ALTEN Satz.
                r.hat_auswahl = false;
                r.ziehen = false;
            }
        }
    }

    /// Markdown setzen: die Quelle in Zeilen mit Laeufen verwandeln.
    ///
    /// Bewusst KEIN vollstaendiger Markdown-Setzer -- Ueberschriften,
    /// Absaetze, Aufzaehlungen, Codebloecke, Tabellen, Zitate, Linien und
    /// Verweise. Was er nicht kennt, steht als Text da; ein Dokument darf an
    /// einer unbekannten Zeile nicht verschwinden.
    pub(super) fn rt_setzen(&mut self, g: &Graphics, wi: usize, idx: usize, breite: i32, basis: i32) {
        let quelle = self.windows[wi].widgets[idx].rich.as_ref().unwrap().quelle.clone();
        let code_font = self.windows[wi].widgets[idx].rich.as_ref().unwrap().code_font;
        let font = self.wfont(g, &self.windows[wi].widgets[idx]);
        let rand = self.sk(12);
        let innen = (breite - rand * 2 - self.sk(8)).max(self.sk(40));
        let mut zeilen: Vec<RtZeile> = Vec::new();
        let mut ziele: Vec<String> = Vec::new();
        let mut ues: Vec<(u8, String, i32)> = Vec::new();
        let mut y = rand;
        let (h1, h2, h3) = (basis + self.sk(12), basis + self.sk(7), basis + self.sk(3));
        let breite_von = |s: &str, gr: i32, code: bool, stil: u8| {
            g.text_width_stil(s, gr, if code && code_font >= 0 { code_font } else { font }, stil)
        };
        // Ein Band: eine Zeile ohne Text, die nur einen Grund legt (Codeblock,
        // Tabellenkopf, Zebra, Zitat, Strich unter der Hauptueberschrift). Es
        // steht VOR den Zeilen, die es unterlegt, und wird darum zuerst gemalt.
        let band = |y: i32, h: i32, art: u8, bx0: i32, bx1: i32| RtZeile {
            y, h, band: art, bx0, bx1, balken: -1, ..Default::default()
        };
        // Woerter zu Zeilen flechten -- die eine Stelle, an der umgebrochen
        // wird; Absatz, Aufzaehlung und Zitat gehen alle hier durch.
        let absatz = |woerter: &[RtWort], x0: i32, gr: i32, weite: i32,
                          y: &mut i32, zeilen: &mut Vec<RtZeile>, balken: i32| {
            let zh = gr + self.sk(6);
            let leer = breite_von(" ", gr, false, 0);
            let neu = |y: i32| RtZeile { y, h: zh, balken, ..Default::default() };
            let mut zeile = neu(*y);
            let mut x = x0;
            for w in woerter {
                let (wg, dy) = rt_code_gr(gr, w.code);
                let bw = breite_von(&w.text, wg, w.code, w.stil);
                let luecke = if zeile.laeufe.is_empty() || w.kleben { 0 } else { leer };
                if x + luecke + bw > x0 + weite && !zeile.laeufe.is_empty() {
                    zeilen.push(zeile.clone());
                    *y += zh;
                    zeile = neu(*y);
                    x = x0;
                }
                let lx = x + if zeile.laeufe.is_empty() { 0 } else { luecke };
                zeile.laeufe.push(RtLauf { x: lx, breite: bw, text: w.text.clone(), groesse: wg,
                                           rolle: w.rolle, stil: w.stil, code: w.code, link: w.link, art: 0, dy });
                x = lx + bw;
            }
            if !zeile.laeufe.is_empty() { zeilen.push(zeile); *y += zh; }
        };
        let roh: Vec<&str> = quelle.lines().collect();
        let mut i = 0usize;
        let mut im_code = false;
        let mut code_farbig = false;
        let knoepfe = self.windows[wi].widgets[idx].rich.as_ref().unwrap().code_knoepfe.clone();
        let kgr = (basis - 2).max(11);
        let kh = kgr + self.sk(10);
        let mut bloecke: Vec<(i32, i32, String, Vec<(i32, i32)>)> = Vec::new();
        let mut block_y = 0i32;
        let mut block_text = String::new();
        while i < roh.len() {
            let z = roh[i].trim_end();
            let t = z.trim_start();
            if t.starts_with("```") {
                // Ein Codeblock bekommt oben und unten etwas Luft IN seinem
                // Grund -- sonst klebt die erste Zeile am Rand.
                if !im_code {
                    let sprache = t[3..].trim().to_lowercase();
                    code_farbig = matches!(sprache.as_str(), "" | "dh" | "basic" | "drachenhauch" | "gb" | "gamebasic" | "vb");
                    y += self.sk(4);
                    block_y = y;
                    block_text.clear();
                }
                // Mit Knoepfen bekommt der Block oben einen Streifen fuer sie --
                // ueber der ersten Codezeile laegen sie auf dem Code.
                let oben = if !im_code && !knoepfe.is_empty() { kh + self.sk(6) } else { self.sk(6) };
                zeilen.push(band(y, oben, 1, 0, 0));
                y += oben;
                if im_code {
                    if !knoepfe.is_empty() {
                        // Rechtsbuendig, der erste Knopf ganz links.
                        let mut rects: Vec<(i32, i32)> = Vec::new();
                        let mut x = rand + innen - self.sk(8);
                        for k in knoepfe.iter().rev() {
                            let bw = breite_von(k, kgr, false, 0) + self.sk(18);
                            x -= bw;
                            rects.push((x, bw));
                            x -= self.sk(6);
                        }
                        rects.reverse();
                        bloecke.push((block_y + self.sk(3), kh, block_text.clone(), rects));
                    }
                    y += self.sk(4);
                }
                im_code = !im_code;
                i += 1;
                continue;
            }
            if im_code {
                let gr = (basis - 1).max(8);
                let zh = gr + self.sk(7);
                block_text.push_str(roh[i]);
                block_text.push('\n');
                let text = z.replace('\t', "    ");
                let x0 = rand + self.sk(12);
                let mut zeile = RtZeile { y, h: zh, band: 1, balken: -1, ..Default::default() };
                // Eingefaerbt wie im Editor: dieselben Abschnitte wie
                // SYNTAX_SPANS. Die Stuecke liegen LUECKENLOS nebeneinander
                // (Lage = Breite des Vorspanns), damit Kopieren den Text
                // Zeichen fuer Zeichen zurueckgibt.
                let zeichen: Vec<char> = text.chars().collect();
                let mut stuecke: Vec<(usize, usize, u8)> = Vec::new();
                let mut pos = 0usize;
                if code_farbig {
                    for (s, l, a) in crate::syntax::spans(&text) {
                        if s > pos { stuecke.push((pos, s, 0)); }
                        let art = match a {
                            crate::syntax::Art::Kommentar => 1, crate::syntax::Art::Text => 2,
                            crate::syntax::Art::Zahl => 3, crate::syntax::Art::Schluessel => 4,
                            crate::syntax::Art::Name => 5, crate::syntax::Art::Operator => 6,
                        };
                        stuecke.push((s, (s + l).min(zeichen.len()), art));
                        pos = s + l;
                    }
                }
                if pos < zeichen.len() { stuecke.push((pos, zeichen.len(), 0)); }
                for (von, bis, art) in stuecke {
                    if bis <= von { continue; }
                    let vor: String = zeichen[..von].iter().collect();
                    let bis_t: String = zeichen[..bis].iter().collect();
                    let a = breite_von(&vor, gr, true, 0);
                    let b = breite_von(&bis_t, gr, true, 0);
                    zeile.laeufe.push(RtLauf { x: x0 + a, breite: (b - a).max(0),
                        text: zeichen[von..bis].iter().collect(), groesse: gr, rolle: 8,
                        stil: 0, code: true, link: -1, art, dy: 0 });
                }
                zeilen.push(zeile);
                y += zh;
                i += 1;
                continue;
            }
            if t.is_empty() { y += self.sk(8); i += 1; continue; }
            // Ueberschriften
            let (stufe, rest) = rt_stufe(t);
            if stufe > 0 {
                let gr = match stufe { 1 => h1, 2 => h2, _ => h3 };
                y += self.sk(if stufe == 1 { 12 } else { 10 });
                ues.push((stufe, rt_titel(rest), y));
                let mut w = rt_inline(rest, 2, &mut ziele);
                for x in w.iter_mut() { x.stil |= crate::schnitt::FETT; x.rolle = 3 + stufe; }
                absatz(&w, rand, gr, innen, &mut y, &mut zeilen, -1);
                // Unter der ersten Ebene ein Band in der Akzentfarbe, unter
                // der zweiten ein feiner Strich -- das gliedert lange
                // Dokumente, ohne dass man die Ueberschrift lesen muss.
                if stufe == 1 {
                    zeilen.push(band(y + self.sk(1), self.sk(3), 2, rand, rand + innen));
                    y += self.sk(8);
                } else if stufe == 2 {
                    zeilen.push(RtZeile { y, h: self.sk(6), linie: true, balken: -1, ..Default::default() });
                    y += self.sk(6);
                }
                y += self.sk(4);
                i += 1;
                continue;
            }
            // Waagerechte Linie
            if t.len() >= 3 && t.chars().all(|c| c == '-' || c == '*' || c == '_') {
                zeilen.push(RtZeile { y: y + self.sk(4), h: self.sk(12), linie: true, balken: -1, ..Default::default() });
                y += self.sk(12);
                i += 1;
                continue;
            }
            // Tabelle: alle zusammenhaengenden Zeilen mit `|` auf einmal --
            // die Spaltenbreiten stehen erst fest, wenn man ALLE Zellen
            // gemessen hat.
            if t.starts_with('|') {
                let mut roh_zeilen: Vec<Vec<String>> = Vec::new();
                while i < roh.len() && roh[i].trim_start().starts_with('|') {
                    let zz = roh[i].trim().trim_matches('|');
                    let zellen: Vec<String> = zz.split('|').map(|c| c.trim().to_string()).collect();
                    // Die Trennzeile (`|---|---|`) traegt keinen Inhalt.
                    let nur_striche = zellen.iter().all(|c| !c.is_empty()
                        && c.chars().all(|ch| ch == '-' || ch == ':'));
                    if !nur_striche { roh_zeilen.push(zellen); }
                    i += 1;
                }
                y += self.rt_tabelle(&roh_zeilen, rand, basis, innen, y, &mut zeilen, &mut ziele, &breite_von);
                continue;
            }
            // Zitat: aufeinander folgende `>`-Zeilen sind EIN Block, darin
            // gelten dieselben Absatzregeln wie draussen (ein leeres `>`
            // trennt). Zeile fuer Zeile gesetzt zerfiel ein Hinweis in
            // einzelne Bruchstuecke mit Luecken im Strich.
            if t.starts_with('>') {
                let start_y = y;
                let start_n = zeilen.len();
                let mut absaetze: Vec<String> = vec![String::new()];
                while i < roh.len() {
                    let zz = roh[i].trim();
                    if !zz.starts_with('>') { break; }
                    let inhalt = zz.trim_start_matches('>').trim();
                    if inhalt.is_empty() {
                        if !absaetze.last().unwrap().is_empty() { absaetze.push(String::new()); }
                    } else {
                        let a = absaetze.last_mut().unwrap();
                        if !a.is_empty() { a.push(' '); }
                        a.push_str(inhalt);
                    }
                    i += 1;
                }
                y += self.sk(6);
                for a in absaetze.iter().filter(|a| !a.is_empty()) {
                    let w = rt_inline(a, 0, &mut ziele);
                    absatz(&w, rand + self.sk(16), basis, innen - self.sk(24), &mut y, &mut zeilen, -1);
                    y += self.sk(4);
                }
                y += self.sk(2);
                zeilen.insert(start_n, band(start_y, y - start_y, 5, rand, rand + innen));
                y += self.sk(6);
                continue;
            }
            // Aufzaehlung (auch verschachtelt: je zwei Leerzeichen eine Stufe)
            let tiefe = ((z.len() - t.len()) / 2).min(4) as i32;
            let punkt = t.starts_with("- ") || t.starts_with("* ") || t.starts_with("+ ");
            let nummer = t.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
                         && t.contains(". ") && t.split_once(". ").map(|(a, _)| a.chars().all(|c| c.is_ascii_digit())).unwrap_or(false);
            if punkt || nummer {
                let (marke, rest) = if punkt { ("\u{2022}".to_string(), &t[2..]) }
                                    else { let (a, b) = t.split_once(". ").unwrap(); (format!("{}.", a), b) };
                let ein = rand + tiefe * self.sk(16);
                let mw = breite_von(&marke, basis, false, 0);
                let zh = basis + self.sk(6);
                let start = zeilen.len();
                let w = rt_inline(rest, 0, &mut ziele);
                absatz(&w, ein + mw + self.sk(6), basis, innen - (ein - rand) - mw - self.sk(6), &mut y, &mut zeilen, -1);
                // Die Marke gehoert in die ERSTE Zeile des Eintrags -- der Rest
                // rueckt darunter ein, sonst laeuft der Text um die Marke herum.
                // Aufzaehlungspunkte in der Akzentfarbe: sie sind Gliederung.
                let lauf = RtLauf { x: ein, breite: mw, text: marke, groesse: basis,
                                    rolle: 2, stil: 0, code: false, link: -1, art: 0, dy: 0 };
                if let Some(erste) = zeilen.get_mut(start) {
                    erste.laeufe.insert(0, lauf);
                } else {
                    zeilen.push(RtZeile { y, h: zh, balken: -1, laeufe: vec![lauf], ..Default::default() });
                    y += zh;
                }
                i += 1;
                continue;
            }
            // Fliesstext: aufeinander folgende Zeilen sind EIN Absatz.
            //
            // Das ist die Regel von Markdown, und sie ist hier keine
            // Formsache: die Dokumente in `docs/` sind von Hand auf 76
            // Spalten umbrochen. Zeile fuer Zeile gesetzt ergaeben sie einen
            // ausgefransten Block, der bei jeder Fensterbreite gleich
            // schlecht aussieht -- als Absatz bricht der Satz dort um, wo
            // Platz ist.
            let mut absatz_text = String::new();
            while i < roh.len() {
                let zz = roh[i].trim();
                if zz.is_empty() || zz.starts_with("```") || zz.starts_with('|') || zz.starts_with('>')
                    || zz.starts_with("- ") || zz.starts_with("* ") || zz.starts_with("+ ")
                    || zz.starts_with('#')
                    || (zz.len() >= 3 && zz.chars().all(|c| c == '-' || c == '*' || c == '_'))
                    || (zz.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && zz.contains(". "))
                {
                    break;
                }
                if !absatz_text.is_empty() { absatz_text.push(' '); }
                absatz_text.push_str(zz);
                i += 1;
            }
            if absatz_text.is_empty() { i += 1; continue; }
            let w = rt_inline(&absatz_text, 0, &mut ziele);
            absatz(&w, rand, basis, innen, &mut y, &mut zeilen, -1);
            y += self.sk(4);
        }
        let hoehe = self.windows[wi].widgets[idx].h;
        let sprung_rand = self.sk(6);
        let r = self.windows[wi].widgets[idx].rich.as_mut().unwrap();
        r.inhalt_h = y + rand;
        r.zeilen = zeilen;
        r.ziele = ziele;
        r.ueberschriften = ues;
        r.codebloecke = bloecke;
        r.satz_aktuell = true;
        let max = (r.inhalt_h - hoehe).max(0);
        // Ein Sprung zu einer Ueberschrift, der vor dem Satz verlangt wurde,
        // wird jetzt eingeloest -- vorher gab es keine Lage, zu der er fuehren kann.
        if let Some(k) = r.ziel_ue.take() {
            if let Some(u) = r.ueberschriften.get(k) { r.scroll = u.2 - sprung_rand; }
        }
        if let Some((nadel, ab)) = r.ziel_suche.take() { rt_finden(r, &nadel, ab, hoehe); }
        let r = self.windows[wi].widgets[idx].rich.as_mut().unwrap();
        r.scroll = r.scroll.clamp(0, max);
    }

    /// Eine Tabelle setzen: Spaltenbreiten aus dem Inhalt, dann Zelle fuer
    /// Zelle. Liefert die gebrauchte Hoehe.
    ///
    /// Die erste Zeile ist der Kopf (fett, auf getoentem Grund), die
    /// uebrigen wechseln sich im Grund ab. Passt die Tabelle nicht in die
    /// Breite, werden die Spalten ANTEILIG gestaucht -- sie abzuschneiden
    /// verstecke die letzte Spalte, und die traegt oft die Erklaerung.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn rt_tabelle(&self, roh: &[Vec<String>], x0: i32, basis: i32, innen: i32, y0: i32,
                  zeilen: &mut Vec<RtZeile>, ziele: &mut Vec<String>,
                  breite_von: &dyn Fn(&str, i32, bool, u8) -> i32) -> i32 {
        if roh.is_empty() { return 0; }
        let spalten = roh.iter().map(|z| z.len()).max().unwrap_or(0);
        if spalten == 0 { return 0; }
        let luecke = self.sk(14);
        let x0 = x0 + self.sk(6);
        let innen = innen - self.sk(12);
        // Breiten wie bei HTML-Tabellen: jede Spalte hat ein MINDESTmass (ihr
        // laengstes Wort -- ein Wort bricht nicht um, es liefe sonst in die
        // Nachbarspalte) und ein WUNSCHmass (die laengste Zelle ohne Umbruch).
        // Passt alles, bekommt jede ihren Wunsch; sonst ihr Minimum plus einen
        // Anteil am Rest nach dem, was ihr zum Wunsch fehlt. Bis 2026-09-27
        // bekam jede Spalte bis zur HALBEN Breite -- eine Signaturspalte nahm
        // dann die Haelfte, und die Beschreibung daneben brach nach drei
        // Woertern um.
        let mut wunsch = vec![0i32; spalten];
        let mut mindest = vec![self.sk(24); spalten];
        for z in roh {
            for (c, zelle) in z.iter().enumerate() {
                let roh_text = zelle.replace("**", "").replace('`', "");
                wunsch[c] = wunsch[c].max(breite_von(&roh_text, basis, false, 0));
                for wort in roh_text.split(' ') {
                    mindest[c] = mindest[c].max(breite_von(wort, basis, false, 0).min(innen / 3));
                }
            }
        }
        for c in 0..spalten { wunsch[c] = wunsch[c].max(mindest[c]); }
        let platz = innen - luecke * (spalten as i32 - 1);
        let (s_min, s_wunsch): (i32, i32) = (mindest.iter().sum(), wunsch.iter().sum());
        let breiten: Vec<i32> = if s_wunsch <= platz {
            wunsch.clone()
        } else if s_min >= platz || s_wunsch == s_min {
            mindest.clone()
        } else {
            let f = (platz - s_min) as f64 / (s_wunsch - s_min) as f64;
            (0..spalten).map(|c| mindest[c] + ((wunsch[c] - mindest[c]) as f64 * f) as i32).collect()
        };
        let rechts = x0 + breiten.iter().sum::<i32>() + luecke * (spalten as i32 - 1);
        let mut y = y0 + self.sk(8);
        for (ri, z) in roh.iter().enumerate() {
            let zeile_start = zeilen.len();
            let mut hoechste = y;
            let mut x = x0;
            for (c, zelle) in z.iter().enumerate() {
                let mut yz = y;
                let mut w = rt_inline(zelle, 0, ziele);
                if ri == 0 { for x2 in w.iter_mut() { x2.stil |= crate::schnitt::FETT; } }
                let zh = basis + self.sk(6);
                let neu = |y: i32| RtZeile { y, h: zh, balken: -1, ..Default::default() };
                let mut zeile = neu(yz);
                let mut cx = x;
                let leer = breite_von(" ", basis, false, 0);
                for wort in &w {
                    let (wg, dy) = rt_code_gr(basis, wort.code);
                    let bw = breite_von(&wort.text, wg, wort.code, wort.stil);
                    let l = if zeile.laeufe.is_empty() || wort.kleben { 0 } else { leer };
                    if cx + l + bw > x + breiten[c] && !zeile.laeufe.is_empty() {
                        zeilen.push(zeile.clone());
                        yz += zh;
                        zeile = neu(yz);
                        cx = x;
                    }
                    let lx = cx + if zeile.laeufe.is_empty() { 0 } else { l };
                    zeile.laeufe.push(RtLauf { x: lx, breite: bw, text: wort.text.clone(), groesse: wg,
                                               rolle: wort.rolle, stil: wort.stil, code: wort.code, link: wort.link, art: 0, dy });
                    cx = lx + bw;
                }
                if !zeile.laeufe.is_empty() { zeilen.push(zeile); yz += zh; }
                hoechste = hoechste.max(yz);
                x += breiten[c] + luecke;
            }
            // Die Zellen einer Zeile stehen als EIGENE Zeilen im Satz (jede
            // Spalte bricht fuer sich um); zusammengehalten werden sie
            // dadurch, dass die naechste Zeile erst unter der hoechsten
            // beginnt -- und durch das Band darunter, das die ganze Zeile
            // unterlegt, auch wenn eine Zelle mehrzeilig ist.
            let art = if ri == 0 { 3 } else if ri % 2 == 0 { 4 } else { 0 };
            if art > 0 {
                zeilen.insert(zeile_start, RtZeile { y: y - self.sk(3), h: hoechste - y + self.sk(3),
                    band: art, bx0: x0 - self.sk(6), bx1: rechts + self.sk(6), balken: -1, ..Default::default() });
            }
            if ri == 0 { hoechste += self.sk(4); }
            y = hoechste + self.sk(3);
        }
        y + self.sk(6) - y0
    }

    /// Dateibaeume mit eingestelltem Takt neu von der Platte lesen. Ohne das
    /// zeigt ein Baum eine frisch angelegte Datei erst, wenn ihn jemand von
    /// Hand auffrischt -- und darauf zu kommen ist niemandes Aufgabe.
    pub(super) fn dateibaeume_auffrischen(&mut self, jetzt: f64) {
        for win in self.windows.iter_mut() {
            if !win.alive || !win.visible { continue; }
            for wdg in win.widgets.iter_mut() {
                let Some(t) = wdg.tree.as_mut() else { continue };
                let takt = match t.datei.as_ref() {
                    Some(d) if d.auffrischen > 0.0 && jetzt >= d.naechste => d.auffrischen,
                    _ => continue,
                };
                t.datei.as_mut().unwrap().naechste = jetzt + takt;
                dateibaum_neu(t);
            }
        }
    }

    /// Lage und Groesse des Kaestchens in einer Baumzeile -- EINE Quelle fuer
    /// Zeichnen und Treffertest; laufen sie auseinander, kippt der Haken neben
    /// dem, was man anklickt.
    pub(super) fn tree_kast(&self, ax: i32, level: i32) -> (i32, i32) {
        (ax + 2 + level * self.sk(TREE_INDENT), (self.sk(TREE_ROW_H) - 6).max(8))
    }

    /// Wie viel Platz die Kaestchen-Spalte wegnimmt (0, wenn es keine gibt).
    ///
    /// Sie steht GANZ LINKS, vor dem Auf-/Zuklapp-Dreieck. Zwischen Dreieck
    /// und Namen lag sie genau dort, wo man eine Zeile anklickt, um sie zu
    /// oeffnen -- ein Klick auf den Namen setzte dann einen Haken statt die
    /// Datei aufzumachen. Aufgefallen ist es an einem Test aus Stand 24, der
    /// eine Datei im Unterordner oeffnet.
    pub(super) fn tree_kast_w(&self, t: &TreeState) -> i32 {
        if t.kaestchen { (self.sk(TREE_ROW_H) - 6).max(8) + 4 } else { 0 }
    }

    pub(super) fn table_geom(&self, wi: usize, idx: usize) -> TGeom {
        let w = &self.windows[wi].widgets[idx];
        let (ax, ay, ww, hh) = self.abs_rect(wi, w);
        let t = w.tbl.as_ref().unwrap();
        let n_cols = t.n_cols();
        // SICHTBARE Zeilen -- Filter und Sortierung bestimmen, was gezeichnet
        // und getroffen wird. Der Rest der Geometrie kennt keinen Unterschied.
        let n_rows = t.view.len();
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
                let avail = ww - self.sk(TBL_SCROLL_W) - 2;
                let per = if n_cols > 0 { (avail / n_cols as i32).max(self.sk(TBL_MIN_COL_W)) } else { 0 };
                vec![per; n_cols]
            }
        };
        let col_widths: Vec<i32> = order.iter().map(|&c| roh[c]).collect();
        let body_x = ax + 1;
        let body_y = ay + header_h;
        let body_w_raw = ww - 2;
        let body_h_raw = hh - header_h - 1;
        let total_w: i32 = col_widths.iter().sum();
        let total_h = n_rows as i32 * row_h;
        let mut need_v = total_h > body_h_raw;
        let need_h = total_w > body_w_raw - if need_v { self.sk(TBL_SCROLL_W) } else { 0 };
        if need_h && total_h > body_h_raw - self.sk(TBL_SCROLL_W) { need_v = true; }
        let body_w = body_w_raw - if need_v { self.sk(TBL_SCROLL_W) } else { 0 };
        let body_h = body_h_raw - if need_h { self.sk(TBL_SCROLL_W) } else { 0 };
        let frozen_w: i32 = col_widths.iter().take(frozen).sum();
        TGeom {
            ax, ay, w: ww, h: hh, n_cols, n_rows, col_widths, row_h, header_h,
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
            let sb_x = gm.ax + gm.body_w_raw - self.sk(TBL_SCROLL_W) + 1;
            if Self::in_rect(mx, my, (sb_x, gm.body_y, self.sk(TBL_SCROLL_W), gm.body_h)) {
                let off = Self::handle_height(gm.body_h, gm.body_h, gm.total_h) / 2;
                { let t = self.windows[wi].widgets[idx].tbl.as_mut().unwrap(); t.drag_v = true; t.drag_off = off; }
                self.active_table = Some((wi, idx));
                self.table_drag(wi, idx, mx, my);
                return;
            }
        }
        if gm.need_h {
            let hs_y = gm.ay + gm.h - self.sk(TBL_SCROLL_W) - 1;
            if Self::in_rect(mx, my, (gm.body_x, hs_y, gm.body_w, self.sk(TBL_SCROLL_W))) {
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
        if r < 0 || c < 0 || r as usize >= t.rows.len() { return; }
        if !t.col_editable(c as usize) { return; }
        let art = t.cell(r as usize, c as usize).map(|x| x.kind()).unwrap_or(CellKind::Text);
        if art != CellKind::Text { return; }
        t.edit_text = t.cell(r as usize, c as usize).map(|x| x.text.clone()).unwrap_or_default();
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
                        if cell.text != neu { cell.text = neu; geaendert = true; }
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
        x
    }

    /// Bereich, in dem die Spalte `c` sichtbar sein darf. Ohne dieses Clipping
    /// schoebe sich eine gescrollte Spalte unter den festen Block.
    pub(super) fn col_clip(gm: &TGeom, c: usize) -> (i32, i32) {
        if c < gm.frozen {
            (gm.body_x, gm.frozen_w)
        } else {
            (gm.body_x + gm.frozen_w, (gm.body_w - gm.frozen_w).max(0))
        }
    }

    /// Liegt `mx` auf der rechten Kante einer Spalte? Liefert deren Index.
    /// Der Fangbereich ist bewusst breiter als ein Pixel -- eine Kante, die
    /// man auf den Punkt genau treffen muss, findet niemand.
    pub(super) fn edge_at(gm: &TGeom, mx: i32, grab: i32) -> i32 {
        for c in 0..gm.col_widths.len() {
            let rechts = Self::col_x(gm, c) + gm.col_widths[c];
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
        let (nv, np) = (t.view.len() as i32, order.len() as i32);
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
        let mut s = g.pop_text_input();
        s.retain(|c| !c.is_control() && c != ' ');
        if s.is_empty() && !g.key_ctrl() {
            for c in (b'a'..=b'z').chain(b'0'..=b'9') {
                if g.key_pressed(c as i64) { s.push(c as char); }
            }
        }
        if s.is_empty() || gm.order.is_empty() { return false; }
        let jetzt = g.get_time();
        let t = self.windows[wi].widgets[i].tbl.as_mut().unwrap();
        if jetzt - t.tipp_zeit > 1.0 { t.tipp.clear(); }
        t.tipp_zeit = jetzt;
        t.tipp.push_str(&s.to_lowercase());
        let tipp = t.tipp.clone();
        let erster = tipp.chars().next().unwrap();
        let wiederholt = tipp.chars().all(|c| c == erster);
        let such: String = if wiederholt { erster.to_string() } else { tipp };
        let spalte = if t.sort_col >= 0 { t.sort_col as usize } else { gm.order[0] };
        let n = t.view.len();
        if n == 0 { return true; }
        let pos = t.view_of(t.selected);
        let start = match pos { p if p >= 0 && wiederholt => p as usize + 1, p if p >= 0 => p as usize, _ => 0 };
        let treffer = (0..n).map(|d| (start + d) % n).find(|&v| {
            let r = t.view[v];
            t.cell(r, spalte).map(|c| c.text.to_lowercase().starts_with(&such)).unwrap_or(false)
        });
        if let Some(v) = treffer {
            let r = t.view[v] as i32;
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
                else if t.baum && t.selected >= 0 && (g.key_pressed(KEY_RIGHT) || g.key_pressed(KEY_LEFT)) {
                    // Rechts klappt auf bzw. geht zum ersten Kind, links klappt
                    // zu bzw. geht zu den Eltern -- wie in jedem Dateibaum.
                    let r = t.selected as usize;
                    let rechts = g.key_pressed(KEY_RIGHT);
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
            let mitte_links = if pos > 0 { Self::col_x(&gm, pos - 1) + gm.col_widths[pos - 1] / 2 } else { i32::MIN };
            let mitte_rechts = if pos + 1 < gm.n_cols { Self::col_x(&gm, pos + 1) + gm.col_widths[pos + 1] / 2 } else { i32::MAX };
            if mx < mitte_links {
                t.move_col(pos, pos - 1);
                pos -= 1;
                t.head_drag = Some((pos, x0, true));
            } else if mx > mitte_rechts {
                t.move_col(pos, pos + 1);
                pos += 1;
                t.head_drag = Some((pos, x0, true));
            }
            return;
        }
        if t.drag_col >= 0 {
            let c = t.drag_col as usize;
            let neu = (t.drag_col_w + (mx - t.drag_col_x)).max(spalte_min);
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
