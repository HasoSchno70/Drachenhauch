//! Gesetzter Text (GUI_RICHTEXT).
//!
//! Teil von `gui.rs`: dort stehen die Typen, hier die Methoden dieses Themas.

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
        if w.kind != Kind::RichText { return Err(format!("{}: das Widget ist kein GUI_RICHTEXT", fn_)); }
        Ok(w.rich.as_mut().unwrap())
    }
    pub(super) fn rt_ref(&self, h: i64, fn_: &str) -> Result<&RichState, String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::RichText { return Err(format!("{}: das Widget ist kein GUI_RICHTEXT", fn_)); }
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
        r.stand = (usize::MAX, 0, 0, 0, 0);
        r.satz_aktuell = false;
        r.ziel_ue = None;
        r.ziel_suche = None;
        Ok(())
    }
    pub fn richtext_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let r = self.rt_mut(h, "GUI_RICHTEXT_SET")?;
        let tafel = super::einstellungen::RICHTEXT;
        match super::einstellungen::schluessel("GUI_RICHTEXT_SET", key, tafel)? {
            "groesse" => { r.basis = wert.max(0.0) as i32; r.stand = (usize::MAX, 0, 0, 0, 0); }
            "codeschrift" => { r.code_font = wert as i64; r.stand = (usize::MAX, 0, 0, 0, 0); }
            // Die sechs Farben der Codebloecke, in der Reihenfolge der Tafel.
            k => {
                let n = tafel.iter().skip(2).position(|(de, _)| *de == k).unwrap_or(0);
                r.code_farben[n] = if wert < 0.0 { None } else { Some(wert as i64) };
            }
        }
        Ok(())
    }
    /// Knoepfe an jedem Codeblock ("Kopieren|Starten", leer = keine).
    pub fn richtext_code_buttons(&mut self, h: i64, knoepfe: &str) -> Result<(), String> {
        let r = self.rt_mut(h, "GUI_RICHTEXT_CODE_BUTTONS")?;
        r.code_knoepfe = knoepfe.split('|').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect();
        r.stand = (usize::MAX, 0, 0, 0, 0);
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
                // Mit dem Stand der Schriften: ein Emoji, das erst nach dem
                // ersten Satz nachgeladen wurde, ist breiter als sein `?`.
                let stand = (quell_len, breite, basis, (self.scale * 100.0) as i32, g.schrift_stand());
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

}
