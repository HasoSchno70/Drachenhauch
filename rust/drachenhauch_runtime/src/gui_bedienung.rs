//! Spinner und Bedienung: Tasten je Widget, Klicks (`handle_press`).
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // --- Spinner (Zahlenfeld mit +/-) ---
    /// Rechtecke der Auf-/Ab-Schaltflaechen (rechte Spalte) + deren linke Kante.
    pub(super) fn spinner_btn_rects(ax: i32, ay: i32, w: i32, h: i32)
        -> ((i32, i32, i32, i32), (i32, i32, i32, i32), i32) {
        let bw = 20;
        let bx = ax + w - bw;
        let mid = h / 2;
        ((bx, ay, bw, mid), (bx, ay + mid, bw, h - mid), bx)
    }

    /// Wert um `steps` Schritte aendern (geklemmt); feuert on_change bei Aenderung.
    pub(super) fn spinner_step(&mut self, wi: usize, i: usize, steps: i32) {
        let w = &mut self.windows[wi].widgets[i];
        let nv = (w.value + steps as f64 * w.step).clamp(w.min, w.max);
        if nv != w.value {
            w.value = nv;
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Mausrad ueber dem Spinner: hoch = +, runter = -.
    pub(super) fn spinner_wheel(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let wheel = Self::rad(g);
        if wheel != 0 { self.spinner_step(wi, i, wheel as i32); }
    }

    /// Tastatur am fokussierten Spinner: Pfeil hoch/runter aendert den Wert.
    pub(super) fn spinner_keys(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        if !self.windows[wi].widgets[i].enabled { return; }
        let wert = self.windows[wi].widgets[i].value;
        // Neu oder von aussen geaendert: der Text aus dem Wert, ganz markiert --
        // das erste Getippte ersetzt ihn.
        let neu = match &self.spin_edit {
            Some((ewi, ei, _, _, _, w, _)) => (*ewi, *ei) != (wi, i) || *w != wert,
            None => true,
        };
        if neu {
            if let Some((ewi, ei, ..)) = self.spin_edit { if (ewi, ei) != (wi, i) { self.spin_fertig(); } }
            let chars: Vec<char> = Self::fmt_num(wert).chars().collect();
            let n = chars.len() as i32;
            self.spin_edit = Some((wi, i, chars, n, 0, wert, wert));
        }
        if g.key_pressed(KEY_UP) || g.key_pressed(KEY_DOWN) {
            self.spin_fertig();
            self.spinner_step(wi, i, if g.key_pressed(KEY_UP) { 1 } else { -1 });
            return;
        }
        if g.key_pressed(KEY_ENTER) {
            // Uebernehmen und weiter markiert; Enter darf danach noch den
            // Standard-Knopf druecken, und GUI_ENTERED meldet es wie beim
            // Textfeld.
            self.spin_fertig();
            self.windows[wi].widgets[i].entered = true;
            return;
        }
        let (mut chars, mut caret, mut anker, vorher_wert) = {
            let e = self.spin_edit.as_ref().unwrap();
            (e.2.clone(), e.3, e.4, e.6)
        };
        if g.key_pressed(KEY_ESC) {
            // Getipptes verwerfen, der Wert von vorher gilt wieder -- dann
            // gehoert ESC nicht dem Abbrechen-Knopf.
            let alt: Vec<char> = Self::fmt_num(vorher_wert).chars().collect();
            if alt != chars || wert != vorher_wert { self.liste_taste = true; }
            if wert != vorher_wert {
                let w = &mut self.windows[wi].widgets[i];
                w.value = vorher_wert;
                let f = w.on_change.clone();
                if let Some(f) = f { self.pending.push(f); }
            }
            let n = alt.len() as i32;
            self.spin_edit = Some((wi, i, alt, n, 0, vorher_wert, vorher_wert));
            return;
        }
        let vorher = chars.clone();
        Self::einzeiler_tasten(g, &mut chars, &mut caret, &mut anker, g.key_ctrl(), g.key_shift(), self.kuerzel_gefeuert);
        // Nur was in eine Zahl gehoert; die Marken rutschen mit.
        let erlaubt = |c: char| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | ',');
        let mut sauber = Vec::with_capacity(chars.len());
        let (mut nc, mut na) = (caret, anker);
        for (k, c) in chars.iter().enumerate() {
            if erlaubt(*c) { sauber.push(*c); }
            else {
                if (k as i32) < caret { nc -= 1; }
                if (k as i32) < anker { na -= 1; }
            }
        }
        let n = sauber.len() as i32;
        let (caret, anker) = (nc.clamp(0, n), na.clamp(0, n));
        let mut neu_wert = wert;
        if sauber != vorher {
            // Gilt schon beim Tippen, wenn es eine Zahl im Bereich ist -- so
            // folgt das Programm (GUI_CHANGED, on_change) jedem Anschlag.
            if let Some(v) = Self::spin_zahl(&sauber) {
                let w = &self.windows[wi].widgets[i];
                let v = Self::spin_runden(w, v);
                if v >= w.min && v <= w.max && v != w.value {
                    let w = &mut self.windows[wi].widgets[i];
                    w.value = v;
                    neu_wert = v;
                    let f = w.on_change.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
        }
        self.spin_edit = Some((wi, i, sauber, caret, anker, neu_wert, vorher_wert));
    }

    /// Der getippte Text als Zahl (Komma wie Punkt), sonst nichts.
    pub(super) fn spin_zahl(chars: &[char]) -> Option<f64> {
        let t: String = chars.iter().collect::<String>().replace(',', ".");
        let t = t.trim();
        if t.is_empty() || t == "-" || t == "+" || t == "." { return None; }
        t.parse::<f64>().ok().filter(|v| v.is_finite())
    }

    /// Ein ganzzahliges Drehfeld (Schritt und Untergrenze ganz) bleibt ganz.
    pub(super) fn spin_runden(w: &Widget, v: f64) -> f64 {
        if w.step.fract() == 0.0 && w.min.fract() == 0.0 { v.round() } else { v }
    }

    /// Das Tippen beenden: eine Zahl gilt, geklemmt auf den Bereich; was keine
    /// ist, faellt weg und der alte Wert bleibt.
    pub(super) fn spin_fertig(&mut self) {
        let Some((wi, i, chars, ..)) = self.spin_edit.take() else { return };
        if wi >= self.windows.len() || i >= self.windows[wi].widgets.len() { return; }
        let Some(v) = Self::spin_zahl(&chars) else { return };
        let w = &mut self.windows[wi].widgets[i];
        if w.kind != Kind::Spinner { return; }
        let v = Self::spin_runden(w, v.clamp(w.min, w.max));
        if v != w.value {
            w.value = v;
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Farbe aus einer Mausposition setzen. `cp_drag` merkt, ob die Geste im
    /// Feld oder im Ton-Streifen begonnen hat -- sonst springt die Farbe, wenn
    /// man beim Ziehen ueber die Grenze zwischen beiden kommt.
    pub(super) fn cp_zieh(&mut self, wi: usize, i: usize, mx: i32, my: i32) {
        let (ax, ay, w, h) = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let (feld, ton, deck) = {
            let wd = &self.windows[wi].widgets[i];
            self.cp_geom(wd, ax, ay, w, h)
        };
        let zone = self.cp_drag.map(|(_, _, z)| z)
            .unwrap_or_else(|| Self::cp_zone(mx, ton, deck));
        let vorher = (self.windows[wi].widgets[i].hsv, self.windows[wi].widgets[i].alpha);
        {
            let wd = &mut self.windows[wi].widgets[i];
            match zone {
                1 => {
                    let t = ((my - ton.1) as f32 / (ton.3 - 1).max(1) as f32).clamp(0.0, 1.0);
                    wd.hsv[0] = t * 360.0;
                }
                2 => if let Some(d) = deck {
                    let t = ((my - d.1) as f32 / (d.3 - 1).max(1) as f32).clamp(0.0, 1.0);
                    // 1..255, nicht 0..255: ein oberstes Byte von 0 liest die
                    // Laufzeit als DECKEND.
                    wd.alpha = 1 + (t * 254.0).round() as i32;
                },
                _ => {
                    let sx = ((mx - feld.0) as f32 / (feld.2 - 1).max(1) as f32).clamp(0.0, 1.0);
                    let sy = ((my - feld.1) as f32 / (feld.3 - 1).max(1) as f32).clamp(0.0, 1.0);
                    wd.hsv[1] = sx;
                    wd.hsv[2] = 1.0 - sy;
                }
            }
        }
        let nachher = (self.windows[wi].widgets[i].hsv, self.windows[wi].widgets[i].alpha);
        if nachher != vorher {
            let f = self.windows[wi].widgets[i].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// In welcher Zone liegt die Maus? Nach der WAAGERECHTEN Lage, weil die
    /// drei Bereiche nebeneinander liegen.
    pub(super) fn cp_zone(mx: i32, ton: (i32, i32, i32, i32),
               deck: Option<(i32, i32, i32, i32)>) -> u8 {
        if let Some(d) = deck { if mx >= d.0 { return 2; } }
        if mx >= ton.0 { 1 } else { 0 }
    }

    /// Klick im Datumswaehler: Pfeile blaettern, eine Zelle waehlt den Tag.
    pub(super) fn dp_press(&mut self, wi: usize, i: usize, mx: i32, my: i32) {
        if self.windows[wi].widgets[i].kompakt {
            self.datum_umschalten(wi, i);
            return;
        }
        let rect = self.abs_rect(wi, &self.windows[wi].widgets[i]);
        let (wb, d) = (self.windows[wi].widgets[i].wochenbeginn, self.windows[wi].widgets[i].datum);
        let (neu, _) = self.dp_rechne(wb, rect, d, mx, my);
        // Ein gesperrtes Datum wird nicht uebernommen -- ein Klick, der die
        // Marke auf einen Tag setzt, den der Waehler selbst ausschliesst,
        // waere eine Luege.
        if Self::dp_erlaubt(&self.windows[wi].widgets[i], neu) {
            self.dp_setze(wi, i, neu);
        }
    }

    /// Was ein Klick ins Monatsgitter `rect` bei Datum `d` bedeutet: das neue
    /// Datum, und ob ein TAG getroffen wurde (sonst Blaettern im Kopf). Eine
    /// Rechnung fuer den Waehler und den aufgeklappten Monat.
    pub(super) fn dp_rechne(&self, wochenbeginn: i32, rect: (i32, i32, i32, i32), d: [i32; 3],
                 mx: i32, my: i32) -> ([i32; 3], bool) {
        let (ax, ay, w, h) = rect;
        let (kopf, zw, zh) = self.dp_geom(w, h);
        let mut tag = false;
        let mut neu = d;
        if my < ay + kopf {
            // Vier Bereiche, von aussen nach innen: Jahr zurueck, Monat
            // zurueck, (Titel), Monat vor, Jahr vor. Grosszuegig geschnitten,
            // weil die Zeichen selbst winzig sind.
            let rand = self.sk(22);
            if mx < ax + rand { neu = self.dp_monat(d, -12); }
            else if mx < ax + w / 3 { neu = self.dp_monat(d, -1); }
            else if mx > ax + w - rand { neu = self.dp_monat(d, 12); }
            else if mx > ax + w * 2 / 3 { neu = self.dp_monat(d, 1); }
        } else {
            let gy = ay + kopf + self.sk(18);
            if my >= gy {
                let sp = ((mx - ax) / zw).clamp(0, 6);
                let ze = (my - gy) / zh;
                if (0..6).contains(&ze) {
                    // Wie das Zeichnen: die erste Spalte ist der Wochenbeginn.
                    // Vorher zaehlte der Klick immer ab Montag -- mit "Woche ab
                    // Sonntag" traf er den Tag daneben.
                    let erster = (crate::kalender::wochentag(d[0], d[1], 1) - wochenbeginn).rem_euclid(7);
                    let nr = ze * 7 + sp - erster + 1;
                    tag = true;
                    let im_monat = crate::kalender::tage_im_monat(d[0], d[1]);
                    // Ein Klick auf einen Nachbarmonats-Tag blaettert dorthin --
                    // das ist die Erwartung, nicht "passiert nichts".
                    if nr < 1 {
                        let (j, m, _) = crate::kalender::plus_monate(d[0], d[1], 1, -1);
                        let vm = crate::kalender::tage_im_monat(j, m);
                        neu = [j, m, (vm + nr).clamp(1, vm)];
                    } else if nr > im_monat {
                        let (j, m, _) = crate::kalender::plus_monate(d[0], d[1], 1, 1);
                        let nm = crate::kalender::tage_im_monat(j, m);
                        neu = [j, m, (nr - im_monat).clamp(1, nm)];
                    } else {
                        neu = [d[0], d[1], nr];
                    }
                }
            }
        }
        (neu, tag)
    }

    // --- Aufgeklappter Monat (Datumsfeld, Datumsspalte) ------------------------

    /// Woran der Monat haengt: das Datumsfeld oder die bearbeitete Zelle.
    pub(super) fn datum_anker(&self) -> Option<(i32, i32, i32, i32)> {
        let (wi, i, zelle) = self.datum_auf?;
        let w = self.windows.get(wi)?.widgets.get(i)?;
        if !self.windows[wi].alive || !self.windows[wi].visible || !w.alive || !w.visible { return None; }
        if zelle { self.edit_cell_rect(wi, i) } else { Some(self.abs_rect(wi, w)) }
    }

    /// Lage des aufgeklappten Monats: unter dem Anker, oder darueber, wenn
    /// unten kein Platz ist. EINE Quelle fuer Zeichnen und Klick.
    pub(super) fn datum_geom(&self) -> Option<(i32, i32, i32, i32)> {
        let (ax, ay, aw, ah) = self.datum_anker()?;
        let pw = aw.max(self.sk(252));
        let ph = self.sk(236);
        let (sb, sh) = self.schirm();
        let x = ax.min(sb - pw).max(0);
        let y = if ay + ah + ph > sh && ay - ph >= 0 { ay - ph } else { ay + ah };
        Some((x, y, pw, ph))
    }

    pub(super) fn datum_umschalten(&mut self, wi: usize, i: usize) {
        if self.datum_auf == Some((wi, i, false)) {
            self.datum_auf = None;
        } else {
            self.open_dropdown = None;
            self.datum_auf = Some((wi, i, false));
            self.datum_zeigt = self.windows[wi].widgets[i].datum;
        }
    }

    /// Ein Tag ist gewaehlt: ins Feld (wenn erlaubt) bzw. in die Zelle.
    pub(super) fn datum_waehlen(&mut self, neu: [i32; 3]) {
        let Some((wi, i, zelle)) = self.datum_auf else { return };
        if zelle {
            if let Some(t) = self.windows[wi].widgets[i].tbl.as_mut() {
                t.edit_text = crate::kalender::format(neu[0], neu[1], neu[2]);
            }
            self.datum_auf = None;
            self.table_end_edit(wi, i, true);
        } else if Self::dp_erlaubt(&self.windows[wi].widgets[i], neu) {
            self.datum_auf = None;
            self.dp_setze(wi, i, neu);
        }
    }

    /// Ein Klick in den aufgeklappten Monat: blaettern oder waehlen. FALSE,
    /// wenn er daneben ging.
    pub(super) fn datum_klick(&mut self, mx: i32, my: i32) -> bool {
        let Some(rect) = self.datum_geom() else { return false };
        if !Self::in_rect(mx, my, rect) { return false; }
        let Some((wi, i, zelle)) = self.datum_auf else { return false };
        let wb = if zelle { 0 } else { self.windows[wi].widgets[i].wochenbeginn };
        let (neu, tag) = self.dp_rechne(wb, rect, self.datum_zeigt, mx, my);
        if tag { self.datum_waehlen(neu); } else { self.datum_zeigt = neu; }
        true
    }

    pub(super) fn dp_monat(&self, d: [i32; 3], schritte: i32) -> [i32; 3] {
        let (j, m, t) = crate::kalender::plus_monate(d[0], d[1], d[2], schritte);
        [j, m, t]
    }

    pub(super) fn dp_setze(&mut self, wi: usize, i: usize, neu: [i32; 3]) {
        if self.windows[wi].widgets[i].datum == neu { return; }
        self.windows[wi].widgets[i].datum = neu;
        let f = self.windows[wi].widgets[i].on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }

    /// Wert eines Regler-artigen Widgets setzen (geklemmt) und bei Aenderung
    /// on_change feuern. Eine Stelle fuer Maus- und Tastatur-Weg.
    pub(super) fn set_value_fire(&mut self, wi: usize, i: usize, roh: f64) {
        let w = &mut self.windows[wi].widgets[i];
        let nv = roh.clamp(w.min, w.max);
        if (nv - w.value).abs() > f64::EPSILON {
            w.value = nv;
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Auswahl in Liste/Klappliste um `d` Zeilen bewegen (am Rand stehenbleiben,
    /// nicht umlaufen -- beim Halten der Taste kreiste man sonst ewig) und bei
    /// Aenderung on_change feuern.
    pub(super) fn liste_bewegen(&mut self, wi: usize, i: usize, d: i32) {
        let n = self.windows[wi].widgets[i].items.len() as i32;
        if n == 0 { return; }
        // Gelaufen wird ueber die SICHTBAREN, WAEHLBAREN Eintraege: ein Pfeil
        // ueberspringt Koepfe und gesperrte, und was der Filter verbirgt, ist
        // auch fuer die Tastatur nicht da.
        let wahl: Vec<usize> = {
            let w = &self.windows[wi].widgets[i];
            let l = w.list.as_deref();
            liste_ansicht(&w.items, l).into_iter().filter(|&k| l.map(|l| l.waehlbar(k)).unwrap_or(true)).collect()
        };
        if wahl.is_empty() { return; }
        let w = &mut self.windows[wi].widgets[i];
        let neu = match wahl.iter().position(|&k| k as i32 == w.sel) {
            Some(p) => wahl[(p as i32 + d).clamp(0, wahl.len() as i32 - 1) as usize] as i32,
            None => if d >= 0 { wahl[0] as i32 } else { wahl[wahl.len() - 1] as i32 },
        };
        if neu != w.sel {
            w.sel = neu;
            if let Some(l) = w.list.as_mut() {
                if l.multi { l.sync(n as usize); for k in 0..n { l.sel[k as usize] = k == neu; } l.anker = neu; }
            }
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Die gewaehlte Zeile einer ListBox in den sichtbaren Bereich scrollen --
    /// ohne das wandert die Auswahl beim Tippen aus dem Bild heraus.
    pub(super) fn liste_sichtbar(&mut self, wi: usize, i: usize) {
        let sel = self.windows[wi].widgets[i].sel;
        self.liste_rollen_zu(wi, i, sel);
    }
    /// So rollen, dass die Datenzeile `k` zu sehen ist -- gerechnet an ihrem
    /// Platz in der ANSICHT (mit Filter steht sie woanders als in den Daten).
    pub(super) fn liste_rollen_zu(&mut self, wi: usize, i: usize, k: i32) {
        let h = self.windows[wi].widgets[i].h;
        let zeile_h = self.list_zeile_h(&self.windows[wi].widgets[i]);   // vor dem mut-Zugriff
        let w = &mut self.windows[wi].widgets[i];
        if k < 0 { return; }
        let pos = match liste_ansicht(&w.items, w.list.as_deref()).iter().position(|&x| x as i32 == k) {
            Some(p) => p as i32,
            None => return,
        };
        let oben = pos * zeile_h;
        let unten = oben + zeile_h;
        let sicht = w.value as i32;
        let neu = if oben < sicht { oben }
                  else if unten > sicht + h { unten - h }
                  else { sicht };
        w.value = neu.max(0) as f64;
    }

    pub(super) fn tree_setze_offen(&mut self, wi: usize, i: usize, node: i32, offen: bool) {
        if node < 0 { return; }
        if let Some(t) = self.windows[wi].widgets[i].tree.as_mut() {
            if let Some(n) = t.nodes.get_mut(node as usize) { n.expanded = offen; }
            // Dateibaum: der Weg entscheidet, nicht die Nummer (siehe DateiBaum).
            if t.datei.is_some() {
                let weg = t.datei.as_ref().unwrap().pfade.get(node as usize).cloned();
                if let Some(w) = weg { Self::ft_offen_setzen(t, &w, offen); dateibaum_neu(t); }
            }
        }
    }

    /// Baum-Auswahl setzen, on_change feuern, Knoten ins Bild scrollen.
    pub(super) fn tree_setze_auswahl(&mut self, wi: usize, i: usize, node: i32) {
        // Ein Pfeil setzt die Menge auf EINEN Knoten -- wie in der Liste.
        let geaendert = match self.windows[wi].widgets[i].tree.as_mut() {
            Some(t) if t.selected != node => {
                t.selected = node;
                if t.multi {
                    t.sync();
                    for s in t.sel.iter_mut() { *s = false; }
                    if node >= 0 && (node as usize) < t.sel.len() { t.sel[node as usize] = true; }
                    t.anker = node;
                }
                true
            }
            _ => false,
        };
        if geaendert {
            let f = self.windows[wi].widgets[i].on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
            self.tree_sichtbar(wi, i);
        }
    }

    pub(super) fn tree_sichtbar(&mut self, wi: usize, i: usize) {
        let h = self.windows[wi].widgets[i].h;
        let zeile_h = self.sk(TREE_ROW_H);   // vor dem mut-Zugriff
        if let Some(t) = self.windows[wi].widgets[i].tree.as_mut() {
            let vis = Self::tree_visible(t);
            if let Some(p) = vis.iter().position(|&n| n as i32 == t.selected) {
                let oben = p as i32 * zeile_h;
                let unten = oben + zeile_h;
                if oben < t.scroll { t.scroll = oben; }
                else if unten > t.scroll + h { t.scroll = unten - h; }
                if t.scroll < 0 { t.scroll = 0; }
            }
        }
    }

    /// Tastatur am fokussierten Widget, das KEIN Textfeld ist.
    ///
    /// Leertaste/Enter loesen aus (Knopf, Kaestchen, Schalter, Radio), die
    /// Pfeile verstellen Werte (Regler, Drehknopf, Trenner) bzw. bewegen die
    /// Auswahl (Liste, Klappliste, Baum). Damit ist jede Art, die man anklicken
    /// kann, auch ohne Maus bedienbar -- vorher endete die Tastaturbedienung
    /// bei den Textfeldern, ein Formular liess sich also nicht abschicken, ohne
    /// zur Maus zu greifen.
    /// "Tippen springt": Buchstaben waehlen den naechsten Eintrag, der so
    /// anfaengt. Getippte Zeichen innerhalb einer Sekunde bilden EIN Wort
    /// ("ki" -> "Kirsche"); dieselbe Taste mehrmals laeuft durch alle, die mit
    /// diesem Buchstaben beginnen. Zeichen kommen aus der Tipp-Warteschlange;
    /// ist sie leer, zaehlen die Buchstaben- und Zifferntasten selbst (die
    /// Wiedergabe einer Aufnahme fuellt nur Tasten, keine Zeichen).
    pub(super) fn liste_tippen(&mut self, wi: usize, i: usize, g: &mut Graphics) {
        let mut s = g.pop_text_input();
        s.retain(|c| !c.is_control() && c != ' ');
        if s.is_empty() && !g.key_ctrl() {
            for c in (b'a'..=b'z').chain(b'0'..=b'9') {
                if g.key_pressed(c as i64) { s.push(c as char); }
            }
        }
        if s.is_empty() { return; }
        let jetzt = g.get_time();
        let (tipp, wahl, sel) = {
            let w = &mut self.windows[wi].widgets[i];
            let n = w.items.len();
            let l = w.list.get_or_insert_with(|| Box::new(ListState { anker: -1, ..Default::default() }));
            l.sync(n);
            if jetzt - l.tipp_zeit > 1.0 { l.tipp.clear(); }
            l.tipp_zeit = jetzt;
            l.tipp.push_str(&s.to_lowercase());
            let wahl: Vec<usize> = liste_ansicht(&w.items, Some(l)).into_iter().filter(|&k| l.waehlbar(k)).collect();
            (l.tipp.clone(), wahl, w.sel)
        };
        if wahl.is_empty() { return; }
        let erster = tipp.chars().next().unwrap();
        let wiederholt = tipp.chars().all(|c| c == erster);
        let such: String = if wiederholt { erster.to_string() } else { tipp };
        let pos = wahl.iter().position(|&k| k as i32 == sel);
        // Ein neues Wort sucht ab der AKTUELLEN Zeile (sie passt vielleicht
        // schon), ein wiederholter Buchstabe ab der naechsten.
        let start = match pos { Some(p) if wiederholt => p + 1, Some(p) => p, None => 0 };
        let items = &self.windows[wi].widgets[i].items;
        let treffer = (0..wahl.len()).map(|d| wahl[(start + d) % wahl.len()])
            .find(|&k| items[k].to_lowercase().starts_with(&such));
        if let Some(k) = treffer {
            self.a11y_eintrag_waehlen(wi, i, k);
            self.liste_sichtbar(wi, i);
        }
    }

    pub(super) fn widget_keys(&mut self, wi: usize, i: usize, kind: Kind, g: &mut Graphics) {
        let (leertaste, enter) = (g.key_pressed(KEY_SPACE), g.key_pressed(KEY_ENTER));
        let ausloesen = leertaste || enter;
        let (auf, ab) = (g.key_pressed(KEY_UP), g.key_pressed(KEY_DOWN));
        let (links, rechts) = (g.key_pressed(KEY_LEFT), g.key_pressed(KEY_RIGHT));
        match kind {
            Kind::Button | Kind::Card => {
                if ausloesen {
                    let w = &mut self.windows[wi].widgets[i];
                    w.clicked = true;
                    let f = w.on_click.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
            Kind::Checkbox | Kind::Toggle => {
                if leertaste {
                    let w = &mut self.windows[wi].widgets[i];
                    w.checked = !w.checked;
                    // `clicked` auch hier setzen. Vorher blieb GUI_CLICKED auf
                    // einem Kaestchen STUMM -- immer FALSE, ohne Fehler. Zwei
                    // von zwei Programmen, die es versucht haben (die Piloten
                    // 187 und 189), hatten damit einen toten Schalter: das
                    // Haekchen kippte, aber nichts passierte. Ein Klick auf
                    // ein Kaestchen IST ein Klick; das Schweigen war der
                    // Fehler, nicht die Erwartung.
                    w.clicked = true;
                    let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                    if let Some(f) = oc { self.pending.push(f); }
                    if let Some(f) = och { self.pending.push(f); }
                }
            }
            Kind::Radio => {
                if leertaste && !self.windows[wi].widgets[i].checked {
                    self.select_radio(wi, i);
                    self.windows[wi].widgets[i].clicked = true;   // wie beim Kaestchen
                    let w = &self.windows[wi].widgets[i];
                    let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                    if let Some(f) = oc { self.pending.push(f); }
                    if let Some(f) = och { self.pending.push(f); }
                }
            }
            Kind::Slider | Kind::Knob => {
                // Ein Zwanzigstel des Bereichs je Druck: von Anschlag zu Anschlag
                // sind es immer 20 Anschlaege, egal wie gross der Bereich ist.
                // Pos1/Ende springen an die Enden.
                let (lo, hi, v) = { let w = &self.windows[wi].widgets[i]; (w.min, w.max, w.value) };
                let schritt = (hi - lo) / 20.0;
                // Eingerastet: von Marke zu Marke.
                let hoch = Self::regler_nachbar(&self.windows[wi].widgets[i], v, true);
                let runter = Self::regler_nachbar(&self.windows[wi].widgets[i], v, false);
                if rechts || auf { self.set_value_fire(wi, i, hoch.unwrap_or(v + schritt)); }
                if links || ab { self.set_value_fire(wi, i, runter.unwrap_or(v - schritt)); }
                if g.key_pressed(KEY_HOME) { self.set_value_fire(wi, i, lo); }
                if g.key_pressed(KEY_END) { self.set_value_fire(wi, i, hi); }
            }
            Kind::Splitter => {
                let vert = self.windows[wi].widgets[i].group == "v";
                let d = if vert { rechts as i32 - links as i32 } else { ab as i32 - auf as i32 } * 8;
                if d != 0 {
                    let w = &mut self.windows[wi].widgets[i];
                    let (lo, hi) = (w.min as i32, w.max as i32);
                    let cur = if vert { w.x } else { w.y };
                    let nv = (cur + d).clamp(lo, hi);
                    if nv != cur {
                        if vert { w.x = nv; } else { w.y = nv; }
                        let f = w.on_change.clone();
                        if let Some(f) = f { self.pending.push(f); }
                    }
                }
            }
            Kind::Dropdown => {
                // Zu: Leertaste/Enter/Ab klappt auf. Offen: Pfeile waehlen,
                // Leertaste/Enter uebernimmt, ESC schliesst.
                // Offen bewegen Pfeile, Bild auf/ab, Pos1/Ende und Tippen nur die
                // MARKIERUNG; Enter/Leertaste uebernimmt sie, ESC schliesst ohne
                // Aenderung -- so wie jede Auswahlliste, und ein Irrtum beim
                // Blaettern feuert kein on_change.
                if self.open_dropdown == Some((wi, i)) {
                    // Verbraucht -- sonst saehe die Fensterregel unten eine
                    // geschlossene Liste und drueckte auch noch Abbrechen.
                    if g.key_pressed(KEY_ESC) { self.open_dropdown = None; self.liste_taste = true; return; }
                    if ausloesen {
                        let k = self.dd_mark;
                        self.open_dropdown = None;
                        self.dd_waehle(wi, i, k);
                        return;
                    }
                    let n = self.windows[wi].widgets[i].items.len() as i32;
                    if n == 0 { return; }
                    let rows = self.dd_zeilen(wi, i);
                    let m = self.dd_mark.max(-1);
                    let neu = if ab { Some(m + 1) }
                        else if auf { Some(if m < 0 { 0 } else { m - 1 }) }
                        else if g.key_pressed(KEY_PAGEDOWN) { Some(m + rows) }
                        else if g.key_pressed(KEY_PAGEUP) { Some(m - rows) }
                        else if g.key_pressed(KEY_HOME) { Some(0) }
                        else if g.key_pressed(KEY_END) { Some(n - 1) }
                        else { None };
                    if let Some(k) = neu {
                        self.dd_mark = k.clamp(0, n - 1);
                        self.dd_mark_zeigen(wi, i);
                    } else {
                        self.dd_tippen(wi, i, true, g);
                    }
                } else if ausloesen || ab {
                    self.open_dropdown = Some((wi, i));
                } else {
                    // Zu: Tippen waehlt gleich den passenden Eintrag.
                    self.dd_tippen(wi, i, false, g);
                }
            }
            Kind::ListBox if self.windows[wi].widgets[i].list.as_ref().map(|l| l.edit.is_some()).unwrap_or(false) => {
                // Umbenennen: die Tasten gehoeren dem Feld -- dieselbe Routine
                // wie Textfeld und Tabellenzelle (Markieren, Strg+A/C/V/X).
                if enter { self.liste_taste = true; self.liste_edit_ende(wi, i, true); return; }
                if g.key_pressed(KEY_ESC) { self.liste_taste = true; self.liste_edit_ende(wi, i, false); return; }
                let (ctrl, shift) = (g.key_ctrl(), g.key_shift());
                let kw = self.kuerzel_gefeuert;
                if let Some(e) = self.windows[wi].widgets[i].list.as_mut().and_then(|l| l.edit.as_mut()) {
                    Self::einzeiler_tasten(g, &mut e.text, &mut e.caret, &mut e.anker, ctrl, shift, kw);
                }
            }
            Kind::ListBox => {
                // F2 benennt den gewaehlten Eintrag um (wenn die Liste es erlaubt)
                // -- es sei denn, ein Menue-Kuerzel hat die Taste schon genommen
                // (die IDE legt F2 auf die Lesezeichen); eine Taste tut EINES.
                if g.key_pressed(KEY_F2) && !self.kuerzel_gefeuert {
                    let w = &self.windows[wi].widgets[i];
                    let k = w.sel;
                    let darf = w.list.as_ref().map(|l| l.bearbeitbar && k >= 0 && l.waehlbar(k as usize)).unwrap_or(false);
                    if darf && (k as usize) < w.items.len() { self.liste_edit_start(wi, i, k as usize); return; }
                }
                let seite = (self.windows[wi].widgets[i].h / self.list_zeile_h(&self.windows[wi].widgets[i])).max(1) - 1;
                let mut d = ab as i32 - auf as i32;
                if g.key_pressed(KEY_PAGEDOWN) { d += seite.max(1); }
                if g.key_pressed(KEY_PAGEUP) { d -= seite.max(1); }
                if d != 0 { self.liste_bewegen(wi, i, d); self.liste_sichtbar(wi, i); }
                // Enter ist der Doppelklick der Tastatur -- "oeffnen" soll ohne
                // Maus gehen, und das Programm fragt dafuer schon GUI_DOUBLE_CLICKED.
                if enter && self.windows[wi].widgets[i].sel >= 0 {
                    let w = &mut self.windows[wi].widgets[i];
                    w.list.get_or_insert_with(|| Box::new(ListState { anker: -1, ..Default::default() })).doppel = true;
                }
                self.liste_tippen(wi, i, g);
                if g.key_pressed(KEY_HOME) { let n = self.windows[wi].widgets[i].items.len() as i32; self.liste_bewegen(wi, i, -n); self.liste_sichtbar(wi, i); }
                if g.key_pressed(KEY_END) { let n = self.windows[wi].widgets[i].items.len() as i32; self.liste_bewegen(wi, i, n); self.liste_sichtbar(wi, i); }
                // Leertaste kippt den Haken der gewaehlten Zeile.
                if g.key_pressed(KEY_SPACE) {
                    let w = &mut self.windows[wi].widgets[i];
                    let sel = w.sel;
                    let n = w.items.len();
                    if let Some(l) = w.list.as_mut() {
                        if l.kaestchen && sel >= 0 && (sel as usize) < n {
                            l.sync(n);
                            l.checks[sel as usize] = !l.checks[sel as usize];
                            let f = w.on_change.clone();
                            if let Some(f) = f { self.pending.push(f); }
                        }
                    }
                }
            }
            Kind::ColorPicker => {
                // Pfeile verstellen Saettigung und Helligkeit, Bild auf/ab den
                // Farbton -- die dritte Achse braucht eine eigene Taste, und
                // die Blaettertasten sind dafuer frei.
                let alt = (self.windows[wi].widgets[i].hsv, self.windows[wi].widgets[i].alpha);
                let schritt = if g.key_shift() { 0.01 } else { 0.04 };
                let wd = &mut self.windows[wi].widgets[i];
                if rechts { wd.hsv[1] = (wd.hsv[1] + schritt).min(1.0); }
                if links { wd.hsv[1] = (wd.hsv[1] - schritt).max(0.0); }
                if auf { wd.hsv[2] = (wd.hsv[2] + schritt).min(1.0); }
                if ab { wd.hsv[2] = (wd.hsv[2] - schritt).max(0.0); }
                if g.key_pressed(KEY_PAGEUP) { wd.hsv[0] = (wd.hsv[0] - 10.0).rem_euclid(360.0); }
                if g.key_pressed(KEY_PAGEDOWN) { wd.hsv[0] = (wd.hsv[0] + 10.0).rem_euclid(360.0); }
                // Deckkraft mit Pos1/Ende -- nur wenn der Streifen ueberhaupt
                // da ist, sonst verstellte man etwas Unsichtbares.
                if wd.alpha_an {
                    if g.key_pressed(KEY_END) { wd.alpha = (wd.alpha + 16).min(255); }
                    if g.key_pressed(KEY_HOME) { wd.alpha = (wd.alpha - 16).max(1); }
                }
                if (self.windows[wi].widgets[i].hsv, self.windows[wi].widgets[i].alpha) != alt {
                    let f = self.windows[wi].widgets[i].on_change.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
            Kind::DatePicker => {
                // Datumsfeld: Leertaste, F4, Alt+Ab klappen auf; offen waehlen die
                // Pfeile den Tag, Enter/Leertaste uebernimmt, ESC klappt nur zu
                // (verbraucht -- sonst druckte die Fensterregel auch Abbrechen).
                if self.windows[wi].widgets[i].kompakt {
                    if self.datum_auf == Some((wi, i, false)) {
                        if g.key_pressed(KEY_ESC) { self.datum_auf = None; self.liste_taste = true; return; }
                        if ausloesen {
                            let z = self.datum_zeigt;
                            self.datum_waehlen(z);
                            self.liste_taste = true;
                            return;
                        }
                        let d = self.datum_zeigt;
                        let tage = (rechts as i32 - links as i32) + 7 * (ab as i32 - auf as i32);
                        let mut neu = d;
                        if tage != 0 {
                            let (j, m, t) = crate::kalender::plus_tage(d[0], d[1], d[2], tage);
                            neu = [j, m, t];
                        }
                        let weite = if g.key_shift() { 12 } else { 1 };
                        if g.key_pressed(KEY_PAGEUP) { neu = self.dp_monat(neu, -weite); }
                        if g.key_pressed(KEY_PAGEDOWN) { neu = self.dp_monat(neu, weite); }
                        self.datum_zeigt = neu;
                        return;
                    }
                    if leertaste || g.key_pressed(KEY_F4) || (g.key_alt() && ab) {
                        self.datum_umschalten(wi, i);
                        return;
                    }
                }
                let d = self.windows[wi].widgets[i].datum;
                let tage = (rechts as i32 - links as i32) + 7 * (ab as i32 - auf as i32);
                let mut neu = d;
                if tage != 0 {
                    let (j, m, t) = crate::kalender::plus_tage(d[0], d[1], d[2], tage);
                    neu = [j, m, t];
                }
                // Mit Umschalt springt es ein JAHR statt einen Monat.
                let weite = if g.key_shift() { 12 } else { 1 };
                if g.key_pressed(KEY_PAGEUP) { neu = self.dp_monat(neu, -weite); }
                if g.key_pressed(KEY_PAGEDOWN) { neu = self.dp_monat(neu, weite); }
                if Self::dp_erlaubt(&self.windows[wi].widgets[i], neu) {
                    self.dp_setze(wi, i, neu);
                }
            }
            Kind::TimePicker => {
                let n = if self.windows[wi].widgets[i].zeit_sek { 3 } else { 2 };
                let seit = rechts as i32 - links as i32;
                if seit != 0 {
                    let w = &mut self.windows[wi].widgets[i];
                    w.sel = (w.sel + seit).clamp(0, n as i32 - 1);
                }
                let d = auf as i32 - ab as i32;
                if d != 0 {
                    let teil = self.windows[wi].widgets[i].sel.clamp(0, n as i32 - 1) as usize;
                    self.tp_dreh(wi, i, teil, d);
                }
            }
            Kind::RichText => {
                let d = ab as i32 - auf as i32;
                if d != 0 {
                    let hoehe = self.windows[wi].widgets[i].h;
                    let r = self.windows[wi].widgets[i].rich.as_mut().unwrap();
                    let max = (r.inhalt_h - hoehe).max(0);
                    r.scroll = (r.scroll + d * 40).clamp(0, max);
                }
                // Strg+A waehlt alles, Strg+C kopiert die Auswahl -- was man in
                // einem Dokument eben tut, bevor man es woanders einfuegt.
                if g.key_ctrl() && g.key_pressed(b'a' as i64) {
                    Self::rt_alles(self.windows[wi].widgets[i].rich.as_mut().unwrap());
                }
                if g.key_ctrl() && g.key_pressed(b'c' as i64) {
                    let t = Self::rt_auswahl_text(self.windows[wi].widgets[i].rich.as_ref().unwrap());
                    if !t.is_empty() { g.clipboard_set(&t); }
                }
            }
            Kind::Accordion => {
                // Hoch/runter waehlt den Kopf, Enter/Leertaste klappt ihn um.
                let n = self.windows[wi].widgets[i].items.len() as i32;
                if n == 0 { return; }
                let f = self.windows[wi].widgets[i].akk.as_ref().map(|a| a.fokus).unwrap_or(0).clamp(0, n - 1);
                let neu = if ab { f + 1 } else if auf { f - 1 } else { f }.clamp(0, n - 1);
                let kopf_akk = self.sk(AKK_KOPF_H);
                if let Some(a) = self.windows[wi].widgets[i].akk.as_mut() {
                    a.fokus = neu;
                    // Den Kopf mit Fokus ins Bild holen.
                    if let Some(&(ky, _)) = a.koepfe.get(neu as usize) {
                        let kopf = kopf_akk;
                        let hh = self.windows[wi].widgets[i].h;
                        let a = self.windows[wi].widgets[i].akk.as_mut().unwrap();
                        if ky < a.scroll { a.scroll = ky; }
                        if ky + kopf > a.scroll + hh { a.scroll = ky + kopf - hh; }
                    }
                }
                if ausloesen { self.akk_umschalten(wi, i, neu as usize); }
            }
            Kind::Wizard => {
                // Enter ist Weiter (am Ende Fertig) -- der naheliegende Knopf.
                if enter { self.wz_knopf(wi, i, 1); }
            }
            Kind::TabControl => {
                // Links/rechts blaettert -- wie in jedem Karteikasten.
                let d = rechts as i32 - links as i32;
                if d != 0 {
                    let w = &mut self.windows[wi].widgets[i];
                    let n = w.items.len() as i32;
                    if n > 0 {
                        let neu = (w.sel + d).clamp(0, n - 1);
                        if neu != w.sel {
                            w.sel = neu;
                            let f = w.on_change.clone();
                            if let Some(f) = f { self.pending.push(f); }
                        }
                    }
                }
            }
            Kind::Tree if self.windows[wi].widgets[i].tree.as_ref().map(|t| t.edit.is_some()).unwrap_or(false) => {
                if enter { self.liste_taste = true; self.baum_edit_ende(wi, i, true); return; }
                if g.key_pressed(KEY_ESC) { self.liste_taste = true; self.baum_edit_ende(wi, i, false); return; }
                let (ctrl, shift) = (g.key_ctrl(), g.key_shift());
                let kw = self.kuerzel_gefeuert;
                if let Some(e) = self.windows[wi].widgets[i].tree.as_mut().and_then(|t| t.edit.as_mut()) {
                    Self::einzeiler_tasten(g, &mut e.text, &mut e.caret, &mut e.anker, ctrl, shift, kw);
                }
            }
            Kind::Tree => {
                let (vis, sel, hat_kinder, offen, eltern) = {
                    let t = match self.windows[wi].widgets[i].tree.as_ref() {
                        Some(t) => t, None => return,
                    };
                    let sel = t.selected;
                    let (hk, of, pa) = match t.nodes.get(sel.max(0) as usize) {
                        Some(n) if sel >= 0 => (n.has_children, n.expanded, n.parent),
                        _ => (false, false, -1),
                    };
                    (Self::tree_visible(t), sel, hk, of, pa)
                };
                if vis.is_empty() { return; }
                let pos = vis.iter().position(|&n| n as i32 == sel);
                // F2 benennt um (wenn der Baum es erlaubt und kein Menue-Kuerzel
                // die Taste schon genommen hat).
                if g.key_pressed(KEY_F2) && !self.kuerzel_gefeuert && sel >= 0 {
                    let darf = self.windows[wi].widgets[i].tree.as_ref().map(|t| t.bearbeitbar && t.datei.is_none()).unwrap_or(false);
                    if darf { self.baum_edit_start(wi, i, sel as usize); return; }
                }
                // Pos1/Ende und Bild auf/ab -- wie bei Liste und Tabelle.
                let seite = (self.windows[wi].widgets[i].h / self.sk(TREE_ROW_H)).max(2) - 1;
                let p0 = pos.map(|p| p as i32).unwrap_or(0);
                let spring = if g.key_pressed(KEY_HOME) { Some(0) }
                    else if g.key_pressed(KEY_END) { Some(vis.len() as i32 - 1) }
                    else if g.key_pressed(KEY_PAGEDOWN) { Some(p0 + seite) }
                    else if g.key_pressed(KEY_PAGEUP) { Some(p0 - seite) }
                    else { None };
                if let Some(p) = spring {
                    let p = p.clamp(0, vis.len() as i32 - 1) as usize;
                    self.tree_setze_auswahl(wi, i, vis[p] as i32);
                    return;
                }
                if self.baum_tippen(wi, i, &vis, g) { return; }
                let mut ziel: Option<i32> = None;
                let d = ab as i32 - auf as i32;
                if d != 0 {
                    let np = match pos {
                        Some(p) => (p as i32 + d).clamp(0, vis.len() as i32 - 1) as usize,
                        None => 0,
                    };
                    ziel = Some(vis[np] as i32);
                } else if rechts {
                    // Zu -> aufklappen, offen -> ins erste Kind. So kennt man es
                    // von jedem Dateibaum.
                    if hat_kinder && !offen { self.tree_setze_offen(wi, i, sel, true); }
                    else if let Some(p) = pos { if p + 1 < vis.len() { ziel = Some(vis[p + 1] as i32); } }
                } else if links {
                    if hat_kinder && offen { self.tree_setze_offen(wi, i, sel, false); }
                    else if eltern >= 0 { ziel = Some(eltern); }
                } else if leertaste && sel >= 0
                          && self.windows[wi].widgets[i].tree.as_ref().map(|t| t.kaestchen).unwrap_or(false) {
                    // Mit Kaestchen gehoert die LEERTASTE dem Haken, Enter dem
                    // Auf- und Zuklappen -- eine Taste fuer beides liesse den
                    // Haken jedes Mal nebenbei kippen.
                    if let Some(t) = self.windows[wi].widgets[i].tree.as_mut() {
                        t.sync();
                        let v = !t.checks[sel as usize];
                        t.checks[sel as usize] = v;
                        Self::ft_haken_merken(t, sel as usize, v);
                    }
                } else if ausloesen && hat_kinder {
                    self.tree_setze_offen(wi, i, sel, !offen);
                } else if enter && sel >= 0 {
                    // Enter auf einer Datei ist das Oeffnen (GUI_FILETREE_ACTIVATED$),
                    // auf einem Blatt der Doppelklick der Tastatur (GUI_DOUBLE_CLICKED).
                    if let Some(t) = self.windows[wi].widgets[i].tree.as_mut() {
                        t.doppel = true;
                        if let Some(d) = t.datei.as_mut() {
                            if let Some(w) = d.pfade.get(sel as usize).cloned() { d.aktiviert = w; }
                        }
                    }
                }
                if let Some(z) = ziel { self.tree_setze_auswahl(wi, i, z); }
            }
            _ => {}
        }
    }

    /// Zahl ohne ueberfluessige Nachkommastellen (ganze Zahl -> ohne Komma).
    pub(super) fn fmt_num(v: f64) -> String {
        if (v.fract()).abs() < 1e-9 { format!("{}", v.round() as i64) }
        else { format!("{:.2}", v) }
    }

    /// Splitter ziehen: neue fenster-relative Position (geklemmt auf [min,max]),
    /// feuert on_change bei Aenderung.
    pub(super) fn drag_split(&mut self, wi: usize, i: usize, mx: i32, my: i32) {
        let off = self.split_off;
        let (winx, winy) = (self.windows[wi].x, self.windows[wi].y);
        let vert = self.windows[wi].widgets[i].group == "v";
        let w = &mut self.windows[wi].widgets[i];
        let (lo, hi) = (w.min as i32, w.max as i32);
        let nv = if vert { (mx - winx - off).clamp(lo, hi) }
                 else { (my - winy - off).clamp(lo, hi) };
        let cur = if vert { w.x } else { w.y };
        if nv != cur {
            if vert { w.x = nv; } else { w.y = nv; }
            let f = w.on_change.clone();
            if let Some(f) = f { self.pending.push(f); }
        }
    }

    /// Das Popup einer Klappliste: hoechstens DD_MAX_ZEILEN Eintraege, unter
    /// der Box -- oder darueber, wenn unten kein Platz ist und oben mehr. EINE
    /// Quelle fuer Zeichnen, Klick, Maus-Zeile und Rollen.
    pub(super) fn dropdown_popup_rect(&self, wi: usize, idx: usize) -> (i32, i32, i32, i32) {
        let (ax, ay, w, h) = self.abs_rect(wi, &self.windows[wi].widgets[idx]);
        let n = self.windows[wi].widgets[idx].items.len() as i32;
        let ih = self.sk(DROPDOWN_ITEM_H);
        let rows = n.min(DD_MAX_ZEILEN).max(1);
        let schirm = if self.schirm_h > 0 { self.schirm_h } else { i32::MAX / 4 };
        let (unten, oben) = (schirm - (ay + h), ay);
        if rows * ih > unten && oben > unten {
            let r = rows.min((oben / ih).max(1));
            (ax, ay - r * ih, w, r * ih)
        } else {
            let r = rows.min((unten / ih).max(1));
            (ax, ay + h, w, r * ih)
        }
    }
    /// Wie viele Zeilen das Popup zeigt.
    pub(super) fn dd_zeilen(&self, wi: usize, idx: usize) -> i32 {
        (self.dropdown_popup_rect(wi, idx).3 / self.sk(DROPDOWN_ITEM_H)).max(1)
    }
    /// Die Markierung ins Bild rollen.
    pub(super) fn dd_mark_zeigen(&mut self, wi: usize, idx: usize) {
        let rows = self.dd_zeilen(wi, idx);
        let n = self.windows[wi].widgets[idx].items.len() as i32;
        if self.dd_mark < self.dd_scroll { self.dd_scroll = self.dd_mark; }
        if self.dd_mark >= self.dd_scroll + rows { self.dd_scroll = self.dd_mark - rows + 1; }
        self.dd_scroll = self.dd_scroll.clamp(0, (n - rows).max(0));
    }
    /// Uebernehmen: den Eintrag waehlen (mit on_change, wenn es ein anderer ist).
    pub(super) fn dd_waehle(&mut self, wi: usize, idx: usize, k: i32) {
        let w = &mut self.windows[wi].widgets[idx];
        if k < 0 || k as usize >= w.items.len() { return; }
        // Bearbeitbar: auch derselbe Eintrag schreibt seinen Text -- getippt
        // war vielleicht "rot", in der Liste steht "Rot".
        let text_vorher = w.text.clone();
        let anders = w.sel != k;
        w.sel = k;
        dd_frei_text_setzen(w);
        if !anders && w.text == text_vorher { return; }
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }
    /// Tippen springt: der naechste Eintrag, der so anfaengt -- dieselbe Regel
    /// wie bei der Liste. Offen bewegt es die Markierung, zu waehlt es gleich.
    pub(super) fn dd_tippen(&mut self, wi: usize, idx: usize, offen: bool, g: &mut Graphics) -> bool {
        let mut s = g.pop_text_input();
        s.retain(|c| !c.is_control() && c != ' ');
        if s.is_empty() && !g.key_ctrl() {
            for c in (b'a'..=b'z').chain(b'0'..=b'9') {
                if g.key_pressed(c as i64) { s.push(c as char); }
            }
        }
        if s.is_empty() { return false; }
        let jetzt = g.get_time();
        if jetzt - self.dd_tipp_zeit > 1.0 { self.dd_tipp.clear(); }
        self.dd_tipp_zeit = jetzt;
        self.dd_tipp.push_str(&s.to_lowercase());
        let tipp = self.dd_tipp.clone();
        let erster = tipp.chars().next().unwrap();
        let wiederholt = tipp.chars().all(|c| c == erster);
        let such: String = if wiederholt { erster.to_string() } else { tipp };
        let items = &self.windows[wi].widgets[idx].items;
        let n = items.len();
        if n == 0 { return true; }
        let von = if offen { self.dd_mark } else { self.windows[wi].widgets[idx].sel };
        let start = if von >= 0 { if wiederholt { von as usize + 1 } else { von as usize } } else { 0 };
        let treffer = (0..n).map(|d| (start + d) % n).find(|&k| items[k].to_lowercase().starts_with(&such));
        if let Some(k) = treffer {
            if offen { self.dd_mark = k as i32; self.dd_mark_zeigen(wi, idx); }
            else { self.dd_waehle(wi, idx, k as i32); }
        }
        true
    }
    /// `GUI_DROPDOWN_SET(dd, key$, wert)` -- heute nur `bearbeitbar`: in das
    /// Feld darf getippt werden, auch etwas, das nicht in der Liste steht.
    pub fn dropdown_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_DROPDOWN_SET")?;
        if w.kind != Kind::Dropdown { return Err("GUI_DROPDOWN_SET: Widget ist keine Klappliste (GUI_DROPDOWN)".into()); }
        match key.to_lowercase().as_str() {
            "bearbeitbar" | "editable" => {
                let an = wert != 0.0;
                if an && !w.frei {
                    w.frei = true;
                    w.text.clear();
                    dd_frei_text_setzen(w);
                }
                if !an { w.frei = false; }
            }
            k => return Err(format!("GUI_DROPDOWN_SET: unbekannter Schluessel '{}' (bekannt: bearbeitbar)", k)),
        }
        Ok(())
    }
    pub fn dropdown_placeholder(&mut self, h: i64, text: String) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_DROPDOWN_PLACEHOLDER")?;
        if w.kind != Kind::Dropdown { return Err("GUI_DROPDOWN_PLACEHOLDER: Widget ist keine Klappliste (GUI_DROPDOWN)".into()); }
        w.placeholder = text;
        Ok(())
    }

    pub(super) fn handle_press(&mut self, mx: i32, my: i32) {
        // Ein aufgeklappter Monat liegt ueber allem (wie eine Klappliste) --
        // vor der Zellbearbeitung, die jeden Klick daneben uebernimmt. Ein
        // Klick daneben klappt ihn zu; einer auf das Datumsfeld selbst nur das.
        if let Some((dw, di, zelle)) = self.datum_auf {
            if self.datum_geom().is_none() {
                self.datum_auf = None;
            } else if self.datum_klick(mx, my) {
                return;
            } else {
                let anker = self.datum_anker();
                self.datum_auf = None;
                let _ = (dw, di);
                if !zelle && anker.map(|a| Self::in_rect(mx, my, a)).unwrap_or(false) { return; }
            }
        }
        // Ein Klick irgendwohin uebernimmt eine laufende Zellbearbeitung.
        // Ohne das bliebe der Editor offen stehen, waehrend die Tastatur
        // laengst woanders hinhoert -- ein Feld, das aussieht als koennte man
        // hineintippen, aber nichts mehr annimmt. Uebernehmen (nicht
        // verwerfen), weil Wegklicken ueberall sonst auch bestaetigt;
        // zuruecknehmen geht mit ESC.
        //
        // AUSSER man klickt IN das Eingabefeld selbst -- dort setzt der Klick
        // die Schreibmarke bzw. beginnt eine Markierung. Ohne diese Ausnahme
        // koennte man im eigenen Text nicht klicken, ohne ihn zu schliessen.
        if let Some((ew, ei)) = self.editing_table {
            // Klick in die offene Auswahlliste waehlt den Eintrag.
            if let Some((px, py, pw, ph, erster, _, zh)) = self.wahl_geom(ew, ei) {
                if Self::in_rect(mx, my, (px, py, pw, ph)) {
                    let k = erster + ((my - py - 1) / zh.max(1)).max(0) as usize;
                    if let Some(t) = self.windows[ew].widgets[ei].tbl.as_mut() {
                        let w = t.col_wahl.get(t.edit_col.max(0) as usize).cloned().unwrap_or_default();
                        if k < w.len() { t.wahl_pos = k as i32; t.edit_text = w[k].clone(); }
                    }
                    self.table_end_edit(ew, ei, true);
                    return;
                }
            }
            let drin = self.edit_cell_rect(ew, ei)
                .map(|r| Self::in_rect(mx, my, r)).unwrap_or(false);
            if !drin { self.table_end_edit(ew, ei, true); }
        }
        // Offenes Ueberlauf-Menue einer Werkzeugleiste: ein Klick darauf fuehrt
        // den Eintrag aus, ein Klick daneben schliesst es -- auf dem >>-Knopf
        // selbst NUR schliessen, sonst ginge es im selben Klick wieder auf.
        if let Some((lw, li)) = self.leiste_popup.take() {
            if let Some((px, py, pw, zeilen)) = self.tb_popup_geom(lw, li) {
                let ph = zeilen.last().map(|z| z.2 - py).unwrap_or(0);
                if Self::in_rect(mx, my, (px, py, pw, ph)) {
                    let k = zeilen.iter().find(|z| my >= z.1 && my < z.2).map(|z| z.0);
                    let knopf = k.filter(|&k| self.windows[lw].widgets[li].leiste.as_ref()
                        .map(|l| l.eintraege[k].art == 0 && !l.eintraege[k].aus).unwrap_or(false));
                    match knopf {
                        Some(k) => self.tb_klick(lw, li, k as i32),
                        None => self.leiste_popup = Some((lw, li)),   // Trenner/gesperrt: offen lassen
                    }
                    return;
                }
                if self.tb_chevron_hit(lw, li, mx, my) { return; }
            }
        }
        // Offenes Dropdown hat Vorrang: das Popup liegt ueber allem und reicht
        // evtl. ueber den Fensterrand hinaus (topmost_at wuerde es verfehlen).
        if let Some((dw, di)) = self.open_dropdown {
            let valid = self.windows.get(dw).and_then(|w| w.widgets.get(di))
                .map(|x| x.alive && x.visible && x.kind == Kind::Dropdown).unwrap_or(false);
            if valid {
                let (px, py, pw, ph) = self.dropdown_popup_rect(dw, di);
                let (bx, by, bw, bh) = self.abs_rect(dw, &self.windows[dw].widgets[di]);
                if Self::in_rect(mx, my, (px, py, pw, ph)) {
                    let n = self.windows[dw].widgets[di].items.len() as i32;
                    let rows = ph / self.sk(DROPDOWN_ITEM_H).max(1);
                    // Klick in den Rollbalken: rollen, nicht waehlen -- und offen bleiben.
                    let bw = self.sk(8);
                    if n > rows && mx >= px + pw - 1 - bw {
                        let rel = ((my - py) as f64 / ph.max(1) as f64).clamp(0.0, 1.0);
                        self.dd_scroll = ((rel * n as f64) as i32 - rows / 2).clamp(0, (n - rows).max(0));
                        return;
                    }
                    let item = self.dd_scroll + (my - py) / self.sk(DROPDOWN_ITEM_H);
                    if item >= 0 && item < n { self.dd_waehle(dw, di, item); }
                    self.open_dropdown = None;
                    return;
                }
                self.open_dropdown = None;
                // Klick auf die zugeklappte Box selbst -> nur schliessen (kein Toggle-Reopen).
                if Self::in_rect(mx, my, (bx, by, bw, bh)) { return; }
            } else {
                self.open_dropdown = None;
            }
        }
        let win = match self.topmost_at(mx, my) {
            Some(w) => w,
            None => { self.focus_widget = None; return; }
        };
        // Modalitaet: steht ein Dialog offen, verschluckt er jeden Klick, der
        // woanders hingeht. Ohne das waere er nur ein Fenster, das obenauf
        // liegt -- man koennte daneben weiterarbeiten, und genau das soll eine
        // Rueckfrage verhindern.
        if let Some(m) = self.modal { if win != m { return; } }
        // Entwurfsmodus: der Klick gehoert dem Designer, nicht den Widgets --
        // und auch nicht dem Fensterrahmen (kein Ziehen, kein Schliessen).
        // Das Fenster kommt nach vorn, nimmt aber keinen Fokus, damit die
        // Kuerzel beim Fenster des Designers bleiben.
        if self.windows[win].entwurf { self.bring_to_front(win); return; }
        self.bring_to_front(win);
        self.focus_window = Some(win);
        let th = self.m("title_h");
        let (wx, wy, ww, wh) = (self.windows[win].x, self.windows[win].y,
                                self.windows[win].w, self.windows[win].h);
        // Resize-Griff unten-rechts?
        let has_chrome = self.windows[win].chrome;
        let grip = 14;
        if has_chrome && self.windows[win].resizable
            && Self::in_rect(mx, my, (wx + ww - grip, wy + wh - grip, grip, grip)) {
            self.resize_window = Some(win);
            self.resize_dx = (wx + ww) - mx;
            self.resize_dy = (wy + wh) - my;
            return;
        }
        // Schliess-Button? (nur mit Chrome)
        if has_chrome && self.windows[win].closable
            && Self::in_rect(mx, my, (wx + ww - th, wy, th, th)) {
            self.windows[win].close_clicked = true;
            self.windows[win].visible = false;
            // Review-Fund: derselbe haengende-Fokus-Bug wie GUI_WINDOW_VISIBLE
            // FALSE -- der Close-Button rief das bisher gar nicht auf.
            self.clear_window_interactions(win);
            return;
        }
        // Titelleiste -> Drag? (nur mit Chrome)
        if has_chrome && Self::in_rect(mx, my, (wx, wy, ww, th)) {
            if self.windows[win].movable {
                self.drag_window = Some(win);
                self.drag_dx = mx - wx;
                self.drag_dy = my - wy;
            }
            return;
        }
        // Das Dock (Reiter, Kreuz, Trennlinie, schwebender Kopf und Griff)
        // vor allem, was im Fenster liegt -- eine schwebende Leiste liegt
        // obenauf, und ihr Kopf darf nicht an den Editor darunter gehen.
        if self.dock_druck(win, mx, my) { return; }
        // Ein Klick neben das Umbenennen-Feld einer Liste uebernimmt es --
        // wie bei der Tabellenzelle. Ein Klick hinein laesst es offen.
        let n = self.windows[win].widgets.len();
        for i in 0..n {
            if let Some(r) = self.edit_rect_any(win, i) {
                if !Self::in_rect(mx, my, r) { self.edit_ende_any(win, i, true); } else { return; }
            }
        }
        // Rollbalken einer Liste: auf dem Griff ziehen, in der Rinne springt
        // der Griff unter die Maus (und laesst sich gleich weiterziehen).
        for i in 0..n {
            if let Some((tx, ty, bw, th, thy, thh)) = self.list_bar_geom(win, i) {
                if Self::in_rect(mx, my, (tx, ty, bw, th)) && self.dock_frei(win, &self.windows[win].widgets[i], mx, my) {
                    let griff = if my >= thy && my < thy + thh { my - thy } else {
                        let max = self.list_max_scroll(win, i);
                        let rel = ((my - ty - thh / 2) as f64 / (th - thh).max(1) as f64).clamp(0.0, 1.0);
                        let w = &mut self.windows[win].widgets[i];
                        Self::roll_setze(w, (rel * max as f64).round());
                        w.rad_ziel = None;
                        thh / 2
                    };
                    self.listbar_zug = Some((win, i, griff));
                    return;
                }
            }
        }
        // Rollbalken eines Panels: ein Klick springt an die Stelle.
        for i in 0..n {
            if let Some((tx, ty, bw, th, _thy, thh)) = self.panel_bar_geom(win, i) {
                if Self::in_rect(mx, my, (tx, ty, bw, th)) && self.dock_frei(win, &self.windows[win].widgets[i], mx, my) {
                    let rel = ((my - ty - thh / 2) as f32 / (th - thh).max(1) as f32).clamp(0.0, 1.0);
                    let max = self.panel_max_scroll(win, i);
                    if let Some(p) = self.windows[win].widgets[i].panel.as_mut() { p.scroll = (rel * max as f32) as i32; }
                    return;
                }
            }
        }
        // Widget unter der Maus.
        let mut hit = None;
        for i in 0..n {
            // Ein Layout-Behaelter ist Luft: er darf den Klick nicht schlucken,
            // der seinen Kindern gilt (sie kommen in der Reihenfolge NACH ihm).
            // Ein rollendes Panel ebenso -- der erste Treffer gewinnt hier,
            // und das Panel liegt vor seinen Kindern.
            let (r, active) = { let w = &self.windows[win].widgets[i]; (self.abs_rect(win, w), self.widget_shown(win, w) && w.enabled && w.kind != Kind::Layout && w.panel.is_none() && self.im_panel_sichtbar(win, w, mx, my)) };
            if active && Self::in_rect(mx, my, r) && !self.luft_fuer_kinder(win, i, mx, my) { hit = Some(i); break; }
        }
        let i = match hit { Some(i) => i, None => { self.focus_widget = None; return; } };
        // Ziehen beginnt mit dem Druck -- ob es eines wird, entscheidet die
        // Bewegung (siehe update). Der Druck selbst wirkt weiter wie immer.
        if self.windows[win].widgets[i].ziehbar {
            let (index, text) = self.zieh_nutzlast(win, i, mx, my);
            self.drag = Some(DragState { quelle: (win, i), index, text, x0: mx, y0: my, aktiv: false });
        }
        // Fokus folgt dem Klick -- an EINER Stelle fuer alle Arten, damit ein
        // spaeteres Tab dort weiterlaeuft, wo die Maus zuletzt war. Deko faengt
        // keinen Fokus (fokussierbar() ist die eine Quelle).
        let kind = self.windows[win].widgets[i].kind;
        self.focus_widget = if kind.fokussierbar() { Some((win, i)) } else { None };
        match kind {
            Kind::Button | Kind::Card => self.press_origin = Some((win, i)),
            Kind::StatusBar => {
                let k = self.sb_treffer(win, i, mx);
                let w = &mut self.windows[win].widgets[i];
                let klickbar = w.status.as_ref().and_then(|s| s.felder.get(k.max(0) as usize)).map(|f| k >= 0 && f.klickbar).unwrap_or(false);
                if klickbar {
                    if let Some(s) = w.status.as_mut() { s.geklickt = k; }
                    w.clicked = true;
                    let f = w.on_click.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
            Kind::Breadcrumb => {
                let k = self.pf_treffer(win, i, mx);
                if k >= 0 { self.pf_klick(win, i, k); }
            }
            Kind::Toolbar if self.tb_chevron_hit(win, i, mx, my) => {
                self.leiste_popup = Some((win, i));
            }
            Kind::Toolbar => {
                let k = self.tb_treffer(win, i, mx, my);
                let aus = self.windows[win].widgets[i].leiste.as_ref()
                    .and_then(|l| l.eintraege.get(k.max(0) as usize)).map(|e| e.aus).unwrap_or(true);
                if k >= 0 && !aus {
                    if let Some(l) = self.windows[win].widgets[i].leiste.as_mut() { l.gedrueckt = k; }
                    self.press_origin = Some((win, i));
                }
            }
            Kind::Checkbox => {
                let w = &mut self.windows[win].widgets[i];
                w.checked = !w.checked;
                // Auch hier `clicked`. Ein Knopf setzt es beim LOSLASSEN
                // (ueber press_origin), ein Kaestchen kippt schon beim
                // Druecken -- fuer den Abfragenden ist beides "in diesem
                // Bild angeklickt".
                w.clicked = true;
                let oc = w.on_click.clone();
                let och = w.on_change.clone();
                if let Some(f) = oc { self.pending.push(f); }
                if let Some(f) = och { self.pending.push(f); }
            }
            Kind::Slider => { self.active_slider = Some((win, i)); self.drag_slider(win, i, mx, my); }
            Kind::Toggle => {
                let w = &mut self.windows[win].widgets[i];
                w.checked = !w.checked;
                w.clicked = true;
                let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                if let Some(f) = oc { self.pending.push(f); }
                if let Some(f) = och { self.pending.push(f); }
            }
            Kind::Knob => { self.active_knob = Some((win, i, my, self.windows[win].widgets[i].value)); }
            Kind::Splitter => {
                self.active_split = Some((win, i));
                let vert = self.windows[win].widgets[i].group == "v";
                let (ax, ay, _w, _h) = self.abs_rect(win, &self.windows[win].widgets[i]);
                self.split_off = if vert { mx - ax } else { my - ay };
            }
            Kind::Spinner => {
                let (ax, ay, ww, hh) = self.abs_rect(win, &self.windows[win].widgets[i]);
                let (up, down, _bx) = Self::spinner_btn_rects(ax, ay, ww, hh);
                if Self::in_rect(mx, my, up) { self.spinner_step(win, i, 1); }
                else if Self::in_rect(mx, my, down) { self.spinner_step(win, i, -1); }
            }
            Kind::ColorPicker => {
                let (fx, fy, fw, fh) = self.abs_rect(win, &self.windows[win].widgets[i]);
                let (_, ton, deck) = {
                    let wd = &self.windows[win].widgets[i];
                    self.cp_geom(wd, fx, fy, fw, fh)
                };
                self.cp_drag = Some((win, i, Self::cp_zone(mx, ton, deck)));
                self.cp_zieh(win, i, mx, my);
            }
            Kind::DatePicker => self.dp_press(win, i, mx, my),
            Kind::TimePicker => self.tp_press(win, i, mx, my),
            Kind::TextInput | Kind::TextArea => {}   // Caret setzt die Editier-Routine
            Kind::Table => self.table_press(win, i, mx, my),
            Kind::Tree => self.tree_press(win, i, mx, my),
            Kind::RichText => {
                let (ax, ay, _, _) = self.abs_rect(win, &self.windows[win].widgets[i]);
                // Ein Knopf an einem Codeblock? Dann ist es kein Beginn einer Auswahl.
                let knopf = {
                    let r = self.windows[win].widgets[i].rich.as_ref().unwrap();
                    let (rx, ry) = (mx - ax, my - ay + r.scroll);
                    let mut t = None;
                    for (by, bh, code, rects) in &r.codebloecke {
                        if ry < *by || ry >= by + bh { continue; }
                        for (k, (bx, bw)) in rects.iter().enumerate() {
                            if rx >= *bx && rx < bx + bw { t = Some((r.code_knoepfe.get(k).cloned().unwrap_or_default(), code.clone())); }
                        }
                    }
                    t
                };
                if let Some((aktion, code)) = knopf {
                    let r = self.windows[win].widgets[i].rich.as_mut().unwrap();
                    r.code_aktion = aktion;
                    r.code_text = code;
                    let f = self.windows[win].widgets[i].on_click.clone();
                    if let Some(f) = f { self.pending.push(f); }
                    return;
                }
                let ziel = {
                    let r = self.windows[win].widgets[i].rich.as_ref().unwrap();
                    Self::rt_link_unter(r, mx - ax, my - ay + r.scroll)
                };
                // Ein Druck beginnt eine Auswahl -- gemessen wird im naechsten
                // Durchgang (`rt_auswahl_pass`), hier gibt es keine Grafik.
                {
                    let dbl = self.dbl_click;
                    let r = self.windows[win].widgets[i].rich.as_mut().unwrap();
                    r.druck = (mx - ax, my - ay + r.scroll);
                    r.ziehen = true;
                    r.anker_neu = true;
                    r.doppel = dbl;
                    r.hat_auswahl = false;
                }
                if let Some(z) = ziel {
                    let r = self.windows[win].widgets[i].rich.as_mut().unwrap();
                    r.geklickt = z;
                    let f = self.windows[win].widgets[i].on_click.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
            Kind::Accordion => {
                let (ax, ay, aw, _) = self.abs_rect(win, &self.windows[win].widgets[i]);
                let k = self.akk_kopf_unter(&self.windows[win].widgets[i], ay, my);
                if k >= 0 {
                    self.akk_umschalten(win, i, k as usize);
                } else if mx >= ax + aw - self.sk(8) {
                    // Klick in den Rollbalken: an die Stelle springen.
                    let w = &mut self.windows[win].widgets[i];
                    let (hh, a) = (w.h, w.akk.as_mut().unwrap());
                    let rel = ((my - ay) as f32 / hh.max(1) as f32).clamp(0.0, 1.0);
                    a.scroll = ((a.inhalt_h - hh).max(0) as f32 * rel) as i32;
                }
            }
            Kind::Wizard => {
                let (ax, ay, _, _) = self.abs_rect(win, &self.windows[win].widgets[i]);
                let gm = self.wz_geom(&self.windows[win].widgets[i], ax, ay);
                if Self::in_rect(mx, my, gm.weiter) { self.wz_knopf(win, i, 1); }
                else if Self::in_rect(mx, my, gm.zurueck) { self.wz_knopf(win, i, 0); }
                else if Self::in_rect(mx, my, gm.abbrechen) { self.wz_knopf(win, i, 2); }
            }
            Kind::TabControl => {
                let (ax, ay, bw, _) = self.abs_rect(win, &self.windows[win].widgets[i]);
                if my >= ay + self.sk(TC_KOPF_H) { return; }   // unter den Koepfen gehoert die Flaeche den Kindern
                let treffer = self.tc_koepfe(ax, bw, &self.windows[win].widgets[i])
                    .into_iter().find(|&(_, x0, x1)| mx >= x0 && mx < x1).map(|(k, _, x1)| (k as i32, x1));
                if let Some((k, x1)) = treffer {
                    if Self::tc_schliessbar(&self.windows[win].widgets[i]) {
                        let (kx, ky, kk) = self.tc_kreuz(x1, ay);
                        if mx >= kx - 3 && mx < kx + kk + 3 && my >= ky - 3 && my < ky + kk + 3 {
                            if let Some(st) = self.windows[win].widgets[i].tabctl.as_mut() { st.zu = Some(k as usize); }
                            return;
                        }
                    }
                    let w = &mut self.windows[win].widgets[i];
                    if w.sel != k {
                        w.sel = k;
                        let f = w.on_change.clone();
                        if let Some(f) = f { self.pending.push(f); }
                    }
                }
            }
            Kind::Radio => {
                let was = self.windows[win].widgets[i].checked;
                self.select_radio(win, i);
                if !was {
                    self.windows[win].widgets[i].clicked = true;
                    let w = &self.windows[win].widgets[i];
                    let oc = w.on_click.clone(); let och = w.on_change.clone();
                    if let Some(f) = oc { self.pending.push(f); }
                    if let Some(f) = och { self.pending.push(f); }
                }
            }
            Kind::Dropdown => {
                // Bearbeitbar: ein Klick ins Feld setzt die Schreibmarke
                // (edit_textinput), nur der Pfeil klappt auf.
                let w = &self.windows[win].widgets[i];
                if !w.frei { self.open_dropdown = Some((win, i)); }
                else {
                    let (ax, _, ww, _) = self.abs_rect(win, w);
                    if mx >= ax + ww - self.sk(DD_PFEIL_B) { self.open_dropdown = Some((win, i)); self.dd_pfeil = false; }
                }
            }
            Kind::ListBox => {
                let (ax, _, _, _) = self.abs_rect(win, &self.windows[win].widgets[i]);
                let row = self.liste_zeile(win, i, my);
                let n = self.windows[win].widgets[i].items.len() as i32;
                if row < 0 || row >= n { return; }
                // Koepfe und gesperrte Eintraege nehmen keinen Klick an.
                let ansicht: Vec<usize> = {
                    let w = &self.windows[win].widgets[i];
                    if w.list.as_ref().map(|l| !l.waehlbar(row as usize)).unwrap_or(false) { return; }
                    liste_ansicht(&w.items, w.list.as_deref())
                };
                let box_w = self.list_box_w(&self.windows[win].widgets[i]);
                let (ctrl, shift) = self.tasten_mod;
                let dbl = self.dbl_click;
                let w = &mut self.windows[win].widgets[i];
                let mut geaendert = false;
                // Klick auf das Kaestchen kippt NUR den Haken -- die Auswahl
                // bleibt, sonst waehlte man beim Abhaken jedes Mal um.
                if box_w > 0 && mx < ax + box_w {
                    if let Some(l) = w.list.as_mut() {
                        l.sync(n as usize);
                        l.checks[row as usize] = !l.checks[row as usize];
                        geaendert = true;
                    }
                } else {
                    // Ein Klick auf einen Eintrag meldet GUI_CLICKED (und
                    // on_click) ein Bild lang -- auch auf den schon gewaehlten.
                    // Vorher war GUI_CLICKED auf einer Liste STUMM, immer FALSE
                    // ohne Fehler (gefunden beim Dogfooding 2026-10-09: ein
                    // Notizzettel konnte die gewaehlte Notiz nicht neu oeffnen,
                    // die Auswahl aenderte sich ja nicht).
                    w.clicked = true;
                    if let Some(f) = w.on_click.clone() { self.pending.push(f); }
                    let multi = w.list.as_ref().map(|l| l.multi).unwrap_or(false);
                    if multi {
                        let l = w.list.as_mut().unwrap();
                        l.sync(n as usize);
                        if ctrl {
                            l.sel[row as usize] = !l.sel[row as usize];
                            l.anker = row;
                        } else if shift && l.anker >= 0 {
                            // Gespannt wird in der SICHTBAREN Reihenfolge, ueber
                            // waehlbare Eintraege -- wie bei der Tabelle.
                            let pa = ansicht.iter().position(|&k| k as i32 == l.anker);
                            let pb = ansicht.iter().position(|&k| k as i32 == row);
                            for k in 0..n as usize { l.sel[k] = false; }
                            if let (Some(pa), Some(pb)) = (pa, pb) {
                                for &k in &ansicht[pa.min(pb)..=pa.max(pb)] {
                                    if l.waehlbar(k) { l.sel[k] = true; }
                                }
                            }
                        } else {
                            for k in 0..n { l.sel[k as usize] = k == row; }
                            l.anker = row;
                        }
                        if dbl { l.doppel = true; }
                        geaendert = true;
                        w.sel = row;
                    } else {
                        if w.sel != row { w.sel = row; geaendert = true; }
                        if dbl {
                            let l = w.list.get_or_insert_with(|| Box::new(ListState { anker: -1, ..Default::default() }));
                            l.doppel = true;
                        }
                    }
                }
                if geaendert {
                    let f = w.on_change.clone();
                    if let Some(f) = f { self.pending.push(f); }
                }
            }
            _ => {}
        }
    }
}
