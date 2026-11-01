//! Bausteine, die eine Anwendung erwartet (docs/entwurf-gui-wie-gtk.md,
//! Paket 2): Fliesslayout mit Auswahl, Alt-Buchstaben auf Knoepfen und
//! Beschriftungen, Beschriftung kuerzen und markieren, Marken am Regler,
//! Kreisel, markierte Tage im Datumswaehler, Groessengruppen, Schriftwahl.
//!
//! Was ein Widget dafuer zusaetzlich traegt, liegt in `Widget::bau`
//! (`Bauzusatz`, nur bei Bedarf angelegt) -- ein gewoehnliches Widget bleibt,
//! was es war.
use super::*;

/// Zusaetze eines Widgets aus Paket 2.
#[derive(Clone, Default)]
pub(super) struct Bauzusatz {
    /// Beschriftung: 0 nicht kuerzen, 1 am Ende, 2 in der Mitte, 3 am Anfang.
    kuerzen: u8,
    /// Beschriftung: mit der Maus markieren und kopieren.
    auswaehlbar: bool,
    /// Regler: so viele gleichmaessig verteilte Marken (0 = keine), benannte
    /// Marken, Wert anzeigen, an den Marken einrasten, Nachkommastellen.
    marken_n: i32,
    marken: Vec<(f64, String)>,
    wert_zeigen: bool,
    einrasten: bool,
    stellen: i32,
    /// Fortschritt als Kreisel.
    kreis: bool,
    /// Datumswaehler: markierte Tage samt Farbe (-1 = Akzent).
    tage: Vec<([i32; 3], i64)>,
    /// Groessengruppe: Name, 1 Breite, 2 Hoehe, 3 beides, und das eigene
    /// Mass beim Eintritt (fuer Widgets mit fester Groesse).
    gruppe: String,
    gruppe_art: u8,
    eigen: (i32, i32),
}

/// Eine offene Schriftwahl (GUI_FONT_DIALOG).
#[derive(Clone)]
pub(super) struct SchriftWahl {
    fenster: usize,
    filter: usize,
    liste: usize,
    groesse: usize,
    vorschau: usize,
    /// Alle Schriften (Anzeigename, Pfad) und die, die der Filter zeigt.
    alle: Vec<(String, String)>,
    sichtbar: Vec<usize>,
    filter_text: String,
    /// Was die Vorschau zeigt (Pfad, Groesse) und geladene Schriften.
    gezeigt: (String, i32),
    geladen: Vec<(String, i32, i64)>,
}

/// Text ohne Alt-Marken: `&` vor einem Buchstaben verschwindet (der
/// Buchstabe ist der Alt-Buchstabe), `&&` wird ein `&`. Liefert dazu die
/// Zeichenstelle des Alt-Buchstabens.
pub(super) fn ohne_marke(s: &str) -> (String, Option<usize>) {
    let mut o = String::with_capacity(s.len());
    let mut stelle = None;
    let mut n = 0;
    let mut z = s.chars().peekable();
    while let Some(c) = z.next() {
        if c == '&' {
            match z.peek() {
                Some('&') => { z.next(); o.push('&'); n += 1; continue; }
                Some(&d) if stelle.is_none() && !d.is_whitespace() => { stelle = Some(n); continue; }
                _ => {}
            }
        }
        o.push(c);
        n += 1;
    }
    (o, stelle)
}

/// Text auf `breite` kuerzen, das "..." am Ende, in der Mitte oder vorn.
fn kuerzen_mit(text: &str, art: u8, breite: i32, mass: &dyn Fn(&str) -> i32) -> String {
    if art == 0 || mass(text) <= breite { return text.to_string(); }
    let z: Vec<char> = text.chars().collect();
    let n = z.len();
    for weg in 1..=n {
        let rest = n - weg;
        let probe: String = match art {
            3 => "...".to_string() + &z[weg..].iter().collect::<String>(),
            2 => {
                let vorn = rest.div_ceil(2);
                z[..vorn].iter().collect::<String>() + "..." + &z[n - (rest - vorn)..].iter().collect::<String>()
            }
            _ => z[..rest].iter().collect::<String>() + "...",
        };
        if mass(&probe) <= breite { return probe; }
    }
    "...".to_string()
}

/// Die Systemschriften: (Anzeigename, Pfad). `DHRT_SCHRIFT_ORDNER` legt den
/// Ordner fest (fuer Pruefungen); vorn stehen die eingebauten.
fn systemschriften() -> Vec<(String, String)> {
    let mut ordner: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(o) = std::env::var("DHRT_SCHRIFT_ORDNER") {
        ordner.push(o.into());
    } else {
        #[cfg(windows)]
        {
            let win = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
            ordner.push(std::path::Path::new(&win).join("Fonts"));
            if let Ok(l) = std::env::var("LOCALAPPDATA") { ordner.push(std::path::Path::new(&l).join("Microsoft\\Windows\\Fonts")); }
        }
        #[cfg(target_os = "macos")]
        {
            ordner.push("/System/Library/Fonts".into());
            ordner.push("/Library/Fonts".into());
            if let Ok(h) = std::env::var("HOME") { ordner.push(std::path::Path::new(&h).join("Library/Fonts")); }
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            ordner.push("/usr/share/fonts".into());
            ordner.push("/usr/local/share/fonts".into());
            if let Ok(h) = std::env::var("HOME") {
                ordner.push(std::path::Path::new(&h).join(".local/share/fonts"));
                ordner.push(std::path::Path::new(&h).join(".fonts"));
            }
        }
    }
    let mut gefunden: Vec<(String, String)> = Vec::new();
    fn sammeln(d: &std::path::Path, tiefe: u32, aus: &mut Vec<(String, String)>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() { if tiefe < 4 { sammeln(&p, tiefe + 1, aus); } continue; }
            let end = p.extension().map(|x| x.to_string_lossy().to_lowercase()).unwrap_or_default();
            if !matches!(end.as_str(), "ttf" | "otf" | "ttc") { continue; }
            let name = p.file_stem().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default();
            aus.push((name, p.to_string_lossy().into_owned()));
        }
    }
    for o in &ordner { sammeln(o, 0, &mut gefunden); }
    gefunden.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()).then(a.1.cmp(&b.1)));
    gefunden.dedup_by(|a, b| a.0.eq_ignore_ascii_case(&b.0));
    let mut o: Vec<(String, String)> = Vec::new();
    if std::env::var("DHRT_SCHRIFT_ORDNER").is_err() {
        for n in ["sans", "serif", "mono"] { o.push((n.to_string(), n.to_string())); }
    }
    o.extend(gefunden);
    o
}

const VORSCHAU_TEXT: &str = "Franz jagt im Taxi quer durch Bayern 0123";

impl Gui {
    fn bau(w: &Widget) -> Option<&Bauzusatz> { w.bau.as_deref() }
    fn bau_mut<'a>(&'a mut self, h: i64, f: &str) -> Result<&'a mut Bauzusatz, String> {
        let w = self.wdg_mut(h, f)?;
        Ok(w.bau.get_or_insert_with(Default::default))
    }

    // ================================================================ Fliesslayout
    /// Die Kinder eines Fliesslayouts der Reihe nach, umbrechend, sobald die
    /// Zeile voll ist; jedes behaelt seine eigene Groesse.
    pub(super) fn fluss_legen(&self, wi: usize, kinder: &[(i32, i32)], ix: i32, iy: i32, iw: i32, ab: i32,
                              ausrichtung: u8) -> Vec<(usize, i32, i32, i32, i32)> {
        let mut aus = Vec::new();
        let mut zeile: Vec<(usize, i32, i32)> = Vec::new();
        let mut y = iy;
        let mut x = ix;
        let flush = |zeile: &mut Vec<(usize, i32, i32)>, y: i32, aus: &mut Vec<(usize, i32, i32, i32, i32)>| -> i32 {
            let zh = zeile.iter().map(|z| z.2).max().unwrap_or(0);
            let mut xx = ix;
            for &(k, w, h) in zeile.iter() {
                let ny = y + match ausrichtung { 1 => (zh - h) / 2, 2 => zh - h, _ => 0 };
                aus.push((k, xx, ny, w.max(1), h.max(1)));
                xx += w + ab;
            }
            zeile.clear();
            zh
        };
        for &(k, _) in kinder {
            if k < 0 { continue; }
            let (w, h) = { let wd = &self.windows[wi].widgets[k as usize]; (wd.w, wd.h) };
            if !zeile.is_empty() && x + w > ix + iw {
                y += flush(&mut zeile, y, &mut aus) + ab;
                x = ix;
            }
            zeile.push((k as usize, w, h));
            x += w + ab;
        }
        flush(&mut zeile, y, &mut aus);
        aus
    }

    /// Ein Druck: liegt er auf einem Kind eines Fliesslayouts mit Auswahl,
    /// wird es gewaehlt (der Druck geht weiter an das Kind).
    pub(super) fn fluss_druck(&mut self, mx: i32, my: i32) {
        let Some(wi) = self.topmost_at(mx, my) else { return };
        if self.windows[wi].entwurf || self.modal.is_some_and(|m| m != wi) { return; }
        for li in 0..self.windows[wi].widgets.len() {
            let (kinder, an) = match self.windows[wi].widgets[li].layout.as_ref() {
                Some(l) if l.art == 3 && l.auswahl => (l.kinder.clone(), true),
                _ => continue,
            };
            if !an || !self.widget_shown(wi, &self.windows[wi].widgets[li]) { continue; }
            for (k, _) in kinder {
                if k < 0 { continue; }
                let w = &self.windows[wi].widgets[k as usize];
                if !self.widget_shown(wi, w) || !w.enabled { continue; }
                if Self::in_rect(mx, my, self.abs_rect(wi, w)) && self.im_panel_sichtbar(wi, w, mx, my) {
                    let l = self.windows[wi].widgets[li].layout.as_mut().unwrap();
                    if l.gewaehlt != k {
                        l.gewaehlt = k;
                        let lw = &mut self.windows[wi].widgets[li];
                        lw.clicked = true;
                        let f = lw.on_change.clone();
                        if let Some(f) = f { self.pending.push(f); }
                    }
                    return;
                }
            }
        }
    }
    pub fn layout_selected(&self, h: i64) -> Result<i64, String> {
        let (wi, _) = Self::dec_widget(h);
        let w = self.wdg(h, "GUI_LAYOUT_SELECTED")?;
        let l = w.layout.as_ref().ok_or("GUI_LAYOUT_SELECTED: Widget ist kein Layout")?;
        Ok(if l.gewaehlt >= 0 { Self::enc_widget(wi, l.gewaehlt as usize) } else { -1 })
    }
    pub fn layout_select(&mut self, h: i64, kind: i64) -> Result<(), String> {
        let (wi, _) = Self::dec_widget(h);
        let k = if kind < 0 { -1 } else {
            let (kw, ki) = Self::dec_widget(kind);
            if kw != wi { return Err("GUI_LAYOUT_SELECT: das Widget liegt in einem anderen Fenster".into()); }
            ki as i32
        };
        let w = self.wdg_mut(h, "GUI_LAYOUT_SELECT")?;
        let l = w.layout.as_mut().ok_or("GUI_LAYOUT_SELECT: Widget ist kein Layout")?;
        if k >= 0 && !l.kinder.iter().any(|&(x, _)| x == k) {
            return Err("GUI_LAYOUT_SELECT: das Widget gehoert nicht zu diesem Layout".into());
        }
        l.gewaehlt = k;
        Ok(())
    }
    /// Die Auswahl eines Fliesslayouts: getoente Flaeche mit Akzentrand hinter
    /// dem gewaehlten Kind (das Layout zeichnet vor seinen Kindern).
    pub(super) fn fluss_auswahl_zeichnen(&self, g: &mut Graphics, wi: usize, wdg: &Widget) {
        let Some(l) = wdg.layout.as_ref() else { return };
        if l.art != 3 || !l.auswahl || l.gewaehlt < 0 { return; }
        let Some(k) = self.windows[wi].widgets.get(l.gewaehlt as usize) else { return };
        if !self.widget_shown(wi, k) { return; }
        let (x, y, w, h) = self.abs_rect(wi, k);
        let acc = self.wcol(wdg, "accent", "accent");
        let r = self.m("corner_radius").min(8).max(3);
        g.round_rect(x - 4, y - 4, x + w + 3, y + h + 3, r, deckkraft(acc, 0.25), true);
        g.round_rect(x - 4, y - 4, x + w + 3, y + h + 3, r, acc, false);
    }

    // ================================================================ Alt-Buchstaben
    pub fn mnemonics(&mut self, an: bool) { self.mnemonics = an; }
    /// Der Anzeigetext eines Widgets: ohne Alt-Marken, wenn sie an sind.
    pub(super) fn anzeige_text<'a>(&self, w: &'a Widget) -> std::borrow::Cow<'a, str> {
        if self.mnemonics && w.text.contains('&')
            && matches!(w.kind, Kind::Button | Kind::Checkbox | Kind::Radio | Kind::Toggle | Kind::Label) {
            std::borrow::Cow::Owned(ohne_marke(&w.text).0)
        } else { std::borrow::Cow::Borrowed(&w.text) }
    }
    /// Text eines Widgets zeichnen, mit Alt-Buchstabe: ohne `&`, und solange
    /// Alt gehalten wird, unterstrichen.
    pub(super) fn wtext_mn(&self, g: &mut Graphics, w: &Widget, x: i32, y: i32, s: &str, c: i64) {
        if !self.mnemonics || !s.contains('&') { self.wtext(g, w, x, y, s.to_string(), c); return; }
        let (t, stelle) = ohne_marke(s);
        self.wtext(g, w, x, y, t.clone(), c);
        if let (Some(i), true) = (stelle, g.key_alt()) {
            let z: Vec<char> = t.chars().collect();
            if i < z.len() {
                let vor: String = z[..i].iter().collect();
                let mit: String = z[..=i].iter().collect();
                let x0 = x + self.wtext_width(g, w, &vor);
                let x1 = x + self.wtext_width(g, w, &mit);
                let uy = y + self.wsize(g, w) + self.sk(1);
                g.line(x0, uy, x1.max(x0 + 2), uy, c);
            }
        }
    }
    /// Alt+Buchstabe: der Knopf, das Kaestchen, der Radioknopf oder das Feld
    /// hinter der Beschriftung mit diesem Alt-Buchstaben (nach den
    /// Menue-Kuerzeln, nur im Fenster mit Fokus bzw. im modalen).
    pub(super) fn mn_pruefen(&mut self, g: &Graphics) {
        if !self.mnemonics || self.kuerzel_gefeuert || !g.key_alt() || g.key_ctrl() { return; }
        let wi = match self.modal.or(self.focus_window) { Some(w) if w < self.windows.len() => w, _ => return };
        if !self.windows[wi].alive || !self.windows[wi].visible || self.windows[wi].entwurf { return; }
        let mut treffer = None;
        for (i, w) in self.windows[wi].widgets.iter().enumerate() {
            if !w.enabled || !self.widget_shown(wi, w) { continue; }
            if !matches!(w.kind, Kind::Button | Kind::Checkbox | Kind::Radio | Kind::Toggle | Kind::Label) { continue; }
            let (t, Some(stelle)) = ohne_marke(&w.text) else { continue };
            let Some(c) = t.chars().nth(stelle).filter(|c| c.is_ascii_alphanumeric()) else { continue };
            let Ok((_, code)) = kuerzel_parsen(&c.to_string()) else { continue };
            if kuerzel_taste(g, code) { treffer = Some(i); break; }
        }
        let Some(i) = treffer else { return };
        self.kuerzel_gefeuert = true;
        let kind = self.windows[wi].widgets[i].kind;
        match kind {
            Kind::Label => {
                // Das Feld hinter der Beschriftung: das naechste bedienbare in
                // der Tab-Reihenfolge, das nach ihr angelegt wurde.
                let reihe = self.tab_reihenfolge(wi);
                if let Some(&ziel) = reihe.iter().find(|&&k| k > i).or(reihe.first()) {
                    self.focus_widget = Some((wi, ziel));
                    if nimmt_text_w(&self.windows[wi].widgets[ziel]) {
                        let n = self.windows[wi].widgets[ziel].text.chars().count() as i32;
                        let w = &mut self.windows[wi].widgets[ziel];
                        w.caret = n; w.sel_anchor = 0;
                    }
                }
            }
            Kind::Button => {
                self.focus_widget = Some((wi, i));
                let w = &mut self.windows[wi].widgets[i];
                w.clicked = true;
                let f = w.on_click.clone();
                if let Some(f) = f { self.pending.push(f); }
            }
            Kind::Radio => {
                self.focus_widget = Some((wi, i));
                let grp = self.windows[wi].widgets[i].group.clone();
                for w in self.windows[wi].widgets.iter_mut() { if w.kind == Kind::Radio && w.group == grp { w.checked = false; } }
                let w = &mut self.windows[wi].widgets[i];
                w.checked = true; w.clicked = true;
                let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                if let Some(f) = oc { self.pending.push(f); }
                if let Some(f) = och { self.pending.push(f); }
            }
            _ => {
                self.focus_widget = Some((wi, i));
                let w = &mut self.windows[wi].widgets[i];
                w.checked = !w.checked; w.clicked = true;
                let (oc, och) = (w.on_click.clone(), w.on_change.clone());
                if let Some(f) = oc { self.pending.push(f); }
                if let Some(f) = och { self.pending.push(f); }
            }
        }
    }

    // ================================================================ Beschriftung
    pub fn label_set(&mut self, h: i64, key: &str, wert: &str) -> Result<(), String> {
        let f = "GUI_LABEL_SET";
        if self.wdg(h, f)?.kind != Kind::Label { return Err(format!("{}: Widget ist keine Beschriftung (GUI_LABEL)", f)); }
        let z = self.bau_mut(h, f)?;
        match key.to_lowercase().as_str() {
            "kuerzen" | "ellipsize" => z.kuerzen = match wert.to_lowercase().as_str() {
                "" | "0" | "aus" | "none" => 0, "1" | "ende" | "end" => 1, "mitte" | "middle" => 2, "anfang" | "start" => 3,
                _ => return Err(format!("{}: kuerzen ist aus, ende, mitte oder anfang -- nicht '{}'", f, wert)),
            },
            "auswaehlbar" | "selectable" => z.auswaehlbar = !matches!(wert.trim(), "" | "0" | "false" | "FALSE"),
            _ => return Err(format!("{}: unbekannter Schluessel '{}' -- kuerzen, auswaehlbar", f, key)),
        }
        Ok(())
    }
    pub fn label_selection(&self, h: i64) -> Result<String, String> {
        let w = self.wdg(h, "GUI_LABEL_SELECTION$")?;
        let t: Vec<char> = self.anzeige_text(w).chars().collect();
        let (lo, hi) = (w.caret.min(w.sel_anchor).max(0) as usize, w.caret.max(w.sel_anchor).max(0) as usize);
        Ok(t[lo.min(t.len())..hi.min(t.len())].iter().collect())
    }
    /// Die Zeilen einer Beschriftung, wie sie dastehen: (Text, erstes
    /// Zeichen im Anzeigetext). Gekuerzt wird nur eine nicht markierbare,
    /// nicht umbrechende.
    pub(super) fn label_zeilen(&self, g: &Graphics, w: &Widget, breite: i32) -> Vec<(String, usize)> {
        let text = self.anzeige_text(w).into_owned();
        let zeilen: Vec<String> = if w.wrap { self.umbrechen(g, w, &text, breite) }
                                  else { text.split('\n').map(str::to_string).collect() };
        let kz = Self::bau(w).map(|z| if z.auswaehlbar { 0 } else { z.kuerzen }).unwrap_or(0);
        let mut aus = Vec::with_capacity(zeilen.len());
        let mut pos = 0usize;   // Bytes in `text`
        for z in zeilen {
            let start_b = text[pos..].find(z.as_str()).map(|r| pos + r).unwrap_or(pos);
            let start_c = text[..start_b].chars().count();
            pos = (start_b + z.len()).min(text.len());
            let gezeigt = if kz > 0 && !w.wrap { kuerzen_mit(&z, kz, breite, &|s| self.wtext_width(g, w, s)) } else { z };
            aus.push((gezeigt, start_c));
        }
        aus
    }
    /// Eine Beschriftung zeichnen (aus draw_widget): Zeilen, Auswahl,
    /// Ausrichtung, Alt-Buchstabe.
    pub(super) fn label_zeichnen(&self, g: &mut Graphics, wdg: &Widget, ax: i32, ay: i32, w: i32, fg: i64) {
        let lh = self.wsize(g, wdg) + self.sk(4);
        let (lo, hi) = (wdg.caret.min(wdg.sel_anchor).max(0) as usize, wdg.caret.max(wdg.sel_anchor).max(0) as usize);
        let mn = if self.mnemonics { ohne_marke(&wdg.text).1 } else { None };
        for (k, (z, start)) in self.label_zeilen(g, wdg, w).into_iter().enumerate() {
            let zw = self.wtext_width(g, wdg, &z);
            let x = match wdg.align { 1 => ax + (w - zw) / 2, 2 => ax + w - zw, _ => ax };
            let y = ay + k as i32 * lh;
            let n = z.chars().count();
            if hi > lo && lo < start + n && hi > start {
                let zc: Vec<char> = z.chars().collect();
                let a = lo.saturating_sub(start).min(n);
                let b = (hi - start).min(n);
                let x0 = x + self.wtext_width(g, wdg, &zc[..a].iter().collect::<String>());
                let x1 = x + self.wtext_width(g, wdg, &zc[..b].iter().collect::<String>());
                g.box_fill(x0, y, x1.max(x0 + 1), y + lh - 2, self.selection_bg(wdg));
            }
            self.wtext(g, wdg, x, y, z.clone(), fg);
            if let (Some(i), true) = (mn, g.key_alt()) {
                if i >= start && i < start + n {
                    let zc: Vec<char> = z.chars().collect();
                    let x0 = x + self.wtext_width(g, wdg, &zc[..i - start].iter().collect::<String>());
                    let x1 = x + self.wtext_width(g, wdg, &zc[..=i - start].iter().collect::<String>());
                    let uy = y + self.wsize(g, wdg) + self.sk(1);
                    g.line(x0, uy, x1.max(x0 + 2), uy, fg);
                }
            }
        }
    }
    /// Zeichenstelle unter der Maus in einer markierbaren Beschriftung.
    fn label_stelle(&self, g: &Graphics, wi: usize, i: usize, mx: i32, my: i32) -> i32 {
        let w = &self.windows[wi].widgets[i];
        let (ax, ay, bw, _) = self.abs_rect(wi, w);
        // Dieselbe Innenflaeche wie beim Zeichnen (Innenabstand der Stildatei).
        let (pv, ph) = self.w_abstand(w).unwrap_or((0, 0));
        let (ax, ay, bw) = (ax + ph, ay + pv, (bw - 2 * ph).max(1));
        let lh = (self.wsize(g, w) + self.sk(4)).max(1);
        let zeilen = self.label_zeilen(g, w, bw);
        if zeilen.is_empty() { return 0; }
        let k = ((my - ay) / lh).clamp(0, zeilen.len() as i32 - 1) as usize;
        let (z, start) = &zeilen[k];
        let zw = self.wtext_width(g, w, z);
        let x = match w.align { 1 => ax + (bw - zw) / 2, 2 => ax + bw - zw, _ => ax };
        let chars: Vec<char> = z.chars().collect();
        *start as i32 + Self::caret_index_at(g, &chars, mx - x, self.mass(g, w)).max(0)
    }
    /// Je Bild (in GUI_UPDATE): Markieren mit der Maus, Strg+C, und der
    /// Hilfetext einer gekuerzten Beschriftung.
    pub(super) fn label_pass(&mut self, g: &mut Graphics, mx: i32, my: i32, druck: bool, unten: bool) {
        // Gekuerzt? Dann zeigt der Hilfetext den ganzen Text.
        for wi in 0..self.windows.len() {
            for i in 0..self.windows[wi].widgets.len() {
                let w = &self.windows[wi].widgets[i];
                if w.kind != Kind::Label || Self::bau(w).map(|z| z.kuerzen == 0 || z.auswaehlbar).unwrap_or(true) { continue; }
                let innen = (w.w - 2 * self.w_abstand(w).map(|a| a.1).unwrap_or(0)).max(1);
                let gekuerzt = self.label_zeilen(g, w, innen).iter().any(|(z, _)| z.ends_with("...") || z.starts_with("..."))
                    && !self.anzeige_text(w).contains("...");
                let ganz = if gekuerzt { self.anzeige_text(w).into_owned() } else { String::new() };
                self.windows[wi].widgets[i].ganzer_text = ganz;
            }
        }
        if druck {
            let alt = self.label_wahl.take();
            let mut neu = None;
            if let Some(wi) = self.topmost_at(mx, my) {
                if !self.windows[wi].entwurf && self.modal.is_none_or(|m| m == wi) {
                    for i in 0..self.windows[wi].widgets.len() {
                        let w = &self.windows[wi].widgets[i];
                        if w.kind != Kind::Label || !Self::bau(w).map(|z| z.auswaehlbar).unwrap_or(false) { continue; }
                        if !self.widget_shown(wi, w) || !Self::in_rect(mx, my, self.abs_rect(wi, w)) { continue; }
                        let k = self.label_stelle(g, wi, i, mx, my);
                        let jetzt = g.get_time();
                        let w = &mut self.windows[wi].widgets[i];
                        let doppelt = jetzt - w.klick_zeit < 0.4 && (k - w.klick_idx).abs() <= 2;
                        w.klick_zeit = jetzt; w.klick_idx = k;
                        if doppelt {
                            let t: Vec<char> = ohne_marke_wenn(self.mnemonics, &w.text).chars().collect();
                            let (a, e) = Self::wort_um(&t, k.max(0) as usize);
                            w.sel_anchor = a as i32; w.caret = e as i32; w.wort_zug = true;
                        } else {
                            w.caret = k; w.sel_anchor = k; w.wort_zug = false;
                        }
                        neu = Some((wi, i));
                        break;
                    }
                }
            }
            if let Some((wi, i)) = alt.filter(|a| Some(*a) != neu) {
                if let Some(w) = self.windows.get_mut(wi).and_then(|x| x.widgets.get_mut(i)) { w.sel_anchor = w.caret; }
            }
            self.label_wahl = neu;
            self.label_zug = neu;
        } else if unten {
            if let Some((wi, i)) = self.label_zug {
                if !self.windows[wi].widgets[i].wort_zug {
                    let k = self.label_stelle(g, wi, i, mx, my);
                    self.windows[wi].widgets[i].caret = k;
                }
            }
        } else {
            self.label_zug = None;
        }
        // Strg+C kopiert die Auswahl -- solange kein Textfeld die Taste hat.
        if let Some((wi, i)) = self.label_wahl {
            let feld = self.focus_widget.and_then(|(w, k)| self.windows.get(w).and_then(|x| x.widgets.get(k)))
                .map(nimmt_text_w).unwrap_or(false);
            if !feld && g.key_ctrl() && g.key_pressed(K_C) {
                let h = Self::enc_widget(wi, i);
                if let Ok(s) = self.label_selection(h) { if !s.is_empty() { g.clipboard_set(&s); } }
            }
        }
    }

    // ================================================================ Regler
    pub fn slider_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let f = "GUI_SLIDER_SET";
        if self.wdg(h, f)?.kind != Kind::Slider { return Err(format!("{}: Widget ist kein Regler (GUI_SLIDER)", f)); }
        let z = self.bau_mut(h, f)?;
        match key.to_lowercase().as_str() {
            "marken" | "ticks" => {
                if !(wert == 0.0 || (2.0..=101.0).contains(&wert)) { return Err(format!("{}: marken ist 0 oder 2..101", f)); }
                z.marken_n = wert as i32;
            }
            "wert_zeigen" | "draw_value" => z.wert_zeigen = wert != 0.0,
            "einrasten" | "snap" => z.einrasten = wert != 0.0,
            "stellen" | "digits" => {
                if !(0.0..=6.0).contains(&wert) { return Err(format!("{}: stellen ist 0..6", f)); }
                z.stellen = wert as i32;
            }
            _ => return Err(format!("{}: unbekannter Schluessel '{}' -- marken, wert_zeigen, einrasten, stellen", f, key)),
        }
        Ok(())
    }
    pub fn slider_mark(&mut self, h: i64, wert: f64, text: String) -> Result<(), String> {
        let f = "GUI_SLIDER_MARK";
        let w = self.wdg(h, f)?;
        if w.kind != Kind::Slider { return Err(format!("{}: Widget ist kein Regler (GUI_SLIDER)", f)); }
        if wert < w.min || wert > w.max { return Err(format!("{}: {} liegt ausserhalb des Reglers ({}..{})", f, wert, w.min, w.max)); }
        let z = self.bau_mut(h, f)?;
        z.marken.retain(|(v, _)| *v != wert);
        z.marken.push((wert, text));
        z.marken.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(())
    }
    pub fn slider_clear_marks(&mut self, h: i64) -> Result<(), String> {
        let z = self.bau_mut(h, "GUI_SLIDER_CLEAR_MARKS")?;
        z.marken.clear(); z.marken_n = 0;
        Ok(())
    }
    /// Alle Marken eines Reglers als Werte (benannte und gleichmaessige).
    fn regler_markenwerte(w: &Widget) -> Vec<f64> {
        let Some(z) = Self::bau(w) else { return Vec::new() };
        let mut v: Vec<f64> = z.marken.iter().map(|m| m.0).collect();
        if z.marken_n >= 2 {
            for k in 0..z.marken_n { v.push(w.min + (w.max - w.min) * k as f64 / (z.marken_n - 1) as f64); }
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        v
    }
    /// Einrasten: der Wert der naechsten Marke (ohne Einrasten unveraendert).
    pub(super) fn regler_einrasten(w: &Widget, v: f64) -> f64 {
        if !Self::bau(w).map(|z| z.einrasten).unwrap_or(false) { return v; }
        let m = Self::regler_markenwerte(w);
        m.into_iter().min_by(|a, b| (a - v).abs().partial_cmp(&(b - v).abs()).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(v)
    }
    /// Pfeiltaste bei eingerastetem Regler: zur naechsten Marke in dieser
    /// Richtung (None = gewoehnlich schrittweise).
    pub(super) fn regler_nachbar(w: &Widget, v: f64, hoch: bool) -> Option<f64> {
        if !Self::bau(w).map(|z| z.einrasten).unwrap_or(false) { return None; }
        let m = Self::regler_markenwerte(w);
        if m.is_empty() { return None; }
        if hoch { m.into_iter().find(|&x| x > v + 1e-9).or(Some(v)) }
        else { m.into_iter().rev().find(|&x| x < v - 1e-9).or(Some(v)) }
    }
    /// Marken unter dem Regler, ihre Beschriftungen und der Wert ueber dem
    /// Knauf.
    pub(super) fn regler_marken_zeichnen(&self, g: &mut Graphics, wdg: &Widget, ax: i32, ay: i32, w: i32, h: i32) {
        let Some(z) = Self::bau(wdg) else { return };
        let span = wdg.max - wdg.min;
        if span == 0.0 { return; }
        let handle_w = self.m("slider_handle_w");
        let xv = |v: f64| ax + handle_w / 2 + (((v - wdg.min) / span) * (w - handle_w) as f64) as i32;
        let c = self.leise(self.th("win_bg"));
        let fg = self.txt_col(wdg);
        for v in Self::regler_markenwerte(wdg) {
            let x = xv(v);
            g.line(x, ay + h + self.sk(1), x, ay + h + self.sk(4), c);
        }
        for (v, t) in &z.marken {
            if t.is_empty() { continue; }
            let x = xv(*v);
            let tw = self.wtext_width(g, wdg, t);
            self.wtext(g, wdg, x - tw / 2, ay + h + self.sk(6), t.clone(), c);
        }
        if z.wert_zeigen {
            let t = format!("{:.*}", z.stellen.max(0) as usize, wdg.value);
            let tw = self.wtext_width(g, wdg, &t);
            let x = xv(wdg.value);
            self.wtext(g, wdg, x - tw / 2, ay - self.wsize(g, wdg) - self.sk(3), t, fg);
        }
    }

    // ================================================================ Kreisel
    pub fn busy(&mut self, win: i64, x: i32, y: i32, d: i32) -> Result<i64, String> {
        if d < 8 { return Err("GUI_BUSY: der Durchmesser muss mindestens 8 sein".into()); }
        let h = self.progress(win, x, y, d, d)?;
        let w = self.wdg_mut(h, "GUI_BUSY")?;
        w.unbestimmt = true;
        w.bau.get_or_insert_with(Default::default).kreis = true;
        Ok(h)
    }
    pub(super) fn progress_kreis_setzen(&mut self, h: i64, an: bool) -> Result<(), String> {
        self.bau_mut(h, "GUI_PROGRESS_SET")?.kreis = an;
        Ok(())
    }
    pub(super) fn ist_kreisel(w: &Widget) -> bool { Self::bau(w).map(|z| z.kreis).unwrap_or(false) }
    /// Zwoelf Punkte im Kreis, der hellste laeuft herum (die Zeit treibt ihn).
    /// Mit einem Wert statt "unbestimmt" fuellt sich der Ring.
    pub(super) fn kreisel_zeichnen(&self, g: &mut Graphics, wdg: &Widget, ax: i32, ay: i32, w: i32, h: i32) {
        let acc = self.acc_col(wdg);
        let (cx, cy) = (ax + w / 2, ay + h / 2);
        let r = (w.min(h) / 2 - 1).max(3);
        if !wdg.unbestimmt {
            let span = wdg.max - wdg.min;
            let ratio = if span != 0.0 { ((wdg.value - wdg.min) / span).clamp(0.0, 1.0) } else { 0.0 };
            let dicke = (r / 4).max(2);
            g.ring(cx, cy, r - dicke, r, 0.0, 360.0, shade(self.th("widget_bg"), 10), true);
            if ratio > 0.0 { g.ring(cx, cy, r - dicke, r, -90.0, -90.0 + 360.0 * ratio, acc, true); }
            return;
        }
        let n = 12;
        let kopf = ((g.get_time() * 1.2).fract() * n as f64) as i32;
        let pr = (r / 5).max(2);
        for k in 0..n {
            let a = std::f64::consts::TAU * k as f64 / n as f64 - std::f64::consts::FRAC_PI_2;
            let x = cx + ((r - pr) as f64 * a.cos()).round() as i32;
            let y = cy + ((r - pr) as f64 * a.sin()).round() as i32;
            let alter = (kopf - k).rem_euclid(n);
            let anteil = 1.0 - alter as f64 / n as f64;
            g.circle(x, y, pr, deckkraft(acc, (0.15 + 0.85 * anteil).min(1.0)));
        }
    }

    // ================================================================ Kalendermarken
    fn dp_bau(&mut self, h: i64, f: &str) -> Result<&mut Bauzusatz, String> {
        if self.wdg(h, f)?.kind != Kind::DatePicker { return Err(format!("{}: Widget ist kein Datumswaehler (GUI_DATEPICKER)", f)); }
        self.bau_mut(h, f)
    }
    pub fn date_mark(&mut self, h: i64, datum: &str, farbe: i64) -> Result<(), String> {
        let f = "GUI_DATE_MARK";
        let d = crate::kalender::parse(datum).ok_or_else(|| format!("{}: '{}' ist kein Datum JJJJ-MM-TT", f, datum))?;
        let d = [d.0, d.1, d.2];
        let z = self.dp_bau(h, f)?;
        z.tage.retain(|(t, _)| *t != d);
        z.tage.push((d, farbe));
        Ok(())
    }
    pub fn date_unmark(&mut self, h: i64, datum: &str) -> Result<(), String> {
        let f = "GUI_DATE_UNMARK";
        let d = crate::kalender::parse(datum).ok_or_else(|| format!("{}: '{}' ist kein Datum JJJJ-MM-TT", f, datum))?;
        let z = self.dp_bau(h, f)?;
        z.tage.retain(|(t, _)| *t != [d.0, d.1, d.2]);
        Ok(())
    }
    pub fn date_clear_marks(&mut self, h: i64) -> Result<(), String> {
        self.dp_bau(h, "GUI_DATE_CLEAR_MARKS")?.tage.clear();
        Ok(())
    }
    pub fn date_marked(&self, h: i64, datum: &str) -> Result<bool, String> {
        let f = "GUI_DATE_MARKED";
        let d = crate::kalender::parse(datum).ok_or_else(|| format!("{}: '{}' ist kein Datum JJJJ-MM-TT", f, datum))?;
        let w = self.wdg(h, f)?;
        Ok(Self::bau(w).map(|z| z.tage.iter().any(|(t, _)| *t == [d.0, d.1, d.2])).unwrap_or(false))
    }
    /// Ist der Tag markiert? Dann die Farbe des Punkts.
    pub(super) fn dp_marke(&self, w: &Widget, d: [i32; 3]) -> Option<i64> {
        let (_, c) = Self::bau(w)?.tage.iter().find(|(t, _)| *t == d)?;
        Some(if *c < 0 { self.wcol(w, "accent", "accent") } else { *c })
    }

    // ================================================================ Groessengruppen
    pub fn size_group(&mut self, h: i64, name: &str, art: &str) -> Result<(), String> {
        let f = "GUI_SIZE_GROUP";
        let a = match art.to_lowercase().as_str() {
            "" | "breite" | "width" => 1, "hoehe" | "height" => 2, "beide" | "both" => 3,
            _ => return Err(format!("{}: '{}' -- breite, hoehe oder beide", f, art)),
        };
        let (ew, eh) = { let w = self.wdg(h, f)?; (w.w, w.h) };
        let z = self.bau_mut(h, f)?;
        z.gruppe = name.to_string();
        z.gruppe_art = if name.is_empty() { 0 } else { a };
        z.eigen = (ew, eh);
        Ok(())
    }
    /// Vor dem Verteilen (in GUI_UPDATE): je Gruppe das groesste Mass ihrer
    /// Mitglieder; Widgets mit fester Groesse bekommen es gleich, die nach
    /// Inhalt ueber `inhalt_mass`.
    pub(super) fn gruppen_pass(&mut self, g: &Graphics) {
        let mut m: HashMap<String, (i32, i32)> = HashMap::new();
        for win in &self.windows {
            if !win.alive { continue; }
            for w in &win.widgets {
                let Some(z) = Self::bau(w).filter(|z| z.gruppe_art > 0) else { continue };
                if !w.alive || !w.visible { continue; }
                let inhaltlich = w.auto_w || w.auto_h || matches!(w.kind, Kind::Label | Kind::Checkbox | Kind::Radio | Kind::Toggle);
                let (bw, bh) = if inhaltlich { self.inhalt_mass_roh(g, w) } else { z.eigen };
                let e = m.entry(z.gruppe.clone()).or_insert((0, 0));
                e.0 = e.0.max(bw); e.1 = e.1.max(bh);
            }
        }
        for win in self.windows.iter_mut() {
            for w in win.widgets.iter_mut() {
                let Some(z) = w.bau.as_deref().filter(|z| z.gruppe_art > 0) else { continue };
                let Some(&(gw, gh)) = m.get(&z.gruppe) else { continue };
                let inhaltlich = w.auto_w || w.auto_h || matches!(w.kind, Kind::Label | Kind::Checkbox | Kind::Radio | Kind::Toggle);
                if inhaltlich { continue; }
                if z.gruppe_art & 1 != 0 { w.w = gw; w.bw = gw; w.nat_w = gw; }
                if z.gruppe_art & 2 != 0 { w.h = gh; w.bh = gh; w.nat_h = gh; }
            }
        }
        self.gruppen = m;
    }
    /// Das Mass nach Inhalt, mit der Gruppe verrechnet.
    pub(super) fn gruppe_anwenden(&self, w: &Widget, mass: (i32, i32)) -> (i32, i32) {
        let Some(z) = Self::bau(w).filter(|z| z.gruppe_art > 0) else { return mass };
        let Some(&(gw, gh)) = self.gruppen.get(&z.gruppe) else { return mass };
        (if z.gruppe_art & 1 != 0 { mass.0.max(gw) } else { mass.0 },
         if z.gruppe_art & 2 != 0 { mass.1.max(gh) } else { mass.1 })
    }

    // ================================================================ Schriftwahl
    /// GUI_FONT_DIALOG(titel$ [, groesse]) -> GUI_WINDOW: modaler Dialog mit
    /// Suchfeld, Liste der Systemschriften, Groesse und Vorschau.
    pub fn font_dialog(&mut self, g: &Graphics, titel: String, groesse: i32) -> Result<i64, String> {
        let alle = systemschriften();
        let (ww, inhalt_h) = (470, 400);
        let wh = self.m_roh("title_h") + inhalt_h;
        let (sw, sh) = (self.unsk(g.screen_width() as i32), self.unsk(g.screen_height() as i32));
        let h = self.new_window(titel, ((sw - ww) / 2).max(0), ((sh - wh) / 2).max(0), ww, wh);
        let wi = h as usize;
        let filter = self.textinput(h, 12, 12, 446, 28, "Suchen ...".into())?;
        self.feld_suche_setzen(filter, true)?;
        let namen: Vec<String> = alle.iter().map(|a| a.0.clone()).collect();
        let liste = self.listbox(h, 12, 50, 290, 250, namen)?;
        self.label(h, "Groesse".into(), 314, 52, None)?;
        let gr = self.spinner(h, 314, 74, 144, 4.0, 200.0, groesse.clamp(4, 200) as f64, 1.0)?;
        let vorschau = self.label(h, VORSCHAU_TEXT.into(), 12, 310, None)?;
        let ok = self.button(h, "OK".into(), 258, inhalt_h - 42, 96, 30)?;
        let ab = self.button(h, "Abbrechen".into(), 362, inhalt_h - 42, 96, 30)?;
        let idx = |x: i64| Self::dec_widget(x).1;
        {
            let win = &mut self.windows[wi];
            win.dlg = true; win.closable = true;
            win.default_btn = idx(ok) as i32;
            win.cancel_btn = idx(ab) as i32;
        }
        let n = alle.len();
        if n > 0 { self.windows[wi].widgets[idx(liste)].sel = 0; }
        self.schriftwahl.push(SchriftWahl {
            fenster: wi, filter: idx(filter), liste: idx(liste), groesse: idx(gr), vorschau: idx(vorschau),
            sichtbar: (0..n).collect(), alle, filter_text: String::new(),
            gezeigt: (String::new(), 0), geladen: Vec::new(),
        });
        self.modal = Some(wi);
        self.bring_to_front(wi);
        self.focus_window = Some(wi);
        self.focus_widget = Some((wi, idx(filter)));
        Ok(h)
    }
    fn wahl_von(&self, h: i64, f: &str) -> Result<&SchriftWahl, String> {
        self.schriftwahl.iter().find(|s| s.fenster == h as usize)
            .ok_or_else(|| format!("{}: dieses Fenster ist keine Schriftwahl (GUI_FONT_DIALOG)", f))
    }
    pub fn font_dialog_path(&self, h: i64) -> Result<String, String> {
        let s = self.wahl_von(h, "GUI_FONT_DIALOG_PATH$")?;
        Ok(self.wahl_jetzt(s).0)
    }
    pub fn font_dialog_size(&self, h: i64) -> Result<i64, String> {
        let s = self.wahl_von(h, "GUI_FONT_DIALOG_SIZE")?;
        Ok(self.wahl_jetzt(s).1 as i64)
    }
    fn wahl_jetzt(&self, s: &SchriftWahl) -> (String, i32) {
        let win = &self.windows[s.fenster];
        let sel = win.widgets[s.liste].sel;
        let pfad = if sel >= 0 { s.sichtbar.get(sel as usize).map(|&k| s.alle[k].1.clone()).unwrap_or_default() } else { String::new() };
        (pfad, win.widgets[s.groesse].value as i32)
    }
    /// Je Bild: Filter anwenden, die Vorschau mit der gewaehlten Schrift
    /// zeichnen. Nach der Antwort bleibt das Fenster als Tombstone lesbar --
    /// die Wahl steht dann noch in Liste und Groessenfeld.
    pub(super) fn schriftwahl_pass(&mut self, g: &mut Graphics) {
        for k in 0..self.schriftwahl.len() {
            let wi = self.schriftwahl[k].fenster;
            if !self.windows[wi].alive { continue; }
            let s = &self.schriftwahl[k];
            let text = self.windows[wi].widgets[s.filter].text.to_lowercase();
            if text != s.filter_text {
                let sichtbar: Vec<usize> = s.alle.iter().enumerate()
                    .filter(|(_, a)| a.0.to_lowercase().contains(&text)).map(|(i, _)| i).collect();
                let namen: Vec<String> = sichtbar.iter().map(|&i| s.alle[i].0.clone()).collect();
                let li = s.liste;
                let w = &mut self.windows[wi].widgets[li];
                w.items = namen;
                w.sel = if w.items.is_empty() { -1 } else { 0 };
                w.value = 0.0;
                let s = &mut self.schriftwahl[k];
                s.sichtbar = sichtbar; s.filter_text = text;
            }
            let (pfad, gr) = self.wahl_jetzt(&self.schriftwahl[k]);
            if pfad.is_empty() || (pfad.clone(), gr) == self.schriftwahl[k].gezeigt { continue; }
            let geladen = self.schriftwahl[k].geladen.iter().find(|(p, _, _)| *p == pfad).map(|x| x.2);
            let font = match geladen {
                Some(f) => Some(f),
                None => match g.load_font(&pfad, 48) {
                    Ok(f) => { self.schriftwahl[k].geladen.push((pfad.clone(), 48, f)); Some(f) }
                    Err(_) => None,
                },
            };
            let vi = self.schriftwahl[k].vorschau;
            let v = &mut self.windows[wi].widgets[vi];
            v.font = font.unwrap_or(-1);
            v.font_size = gr;
            v.text = if font.is_some() { VORSCHAU_TEXT.into() } else { "(laesst sich nicht laden)".into() };
            v.w = 446; v.bw = 446;
            v.h = gr + 8; v.bh = gr + 8;
            self.schriftwahl[k].gezeigt = (pfad, gr);
        }
    }
}

/// Zusaetze in die `.dhform` (markierte Tage nicht -- das sind Daten).
pub(super) fn bau_json(w: &Widget, o: &mut serde_json::Value) {
    let Some(z) = w.bau.as_deref() else { return };
    if z.kuerzen > 0 { o["kuerzen"] = serde_json::json!(["", "ende", "mitte", "anfang"][z.kuerzen as usize]); }
    if z.auswaehlbar { o["auswaehlbar"] = serde_json::json!(true); }
    if z.marken_n > 0 { o["marken"] = serde_json::json!(z.marken_n); }
    if !z.marken.is_empty() {
        o["marken_texte"] = serde_json::json!(z.marken.iter().map(|(v, t)| serde_json::json!([v, t])).collect::<Vec<_>>());
    }
    if z.wert_zeigen { o["wert_zeigen"] = serde_json::json!(true); }
    if z.einrasten { o["einrasten"] = serde_json::json!(true); }
    if z.stellen > 0 { o["stellen"] = serde_json::json!(z.stellen); }
    if z.kreis { o["kreis"] = serde_json::json!(true); }
    if z.gruppe_art > 0 {
        o["groessengruppe"] = serde_json::json!(z.gruppe);
        o["groessengruppe_art"] = serde_json::json!(["", "breite", "hoehe", "beide"][z.gruppe_art as usize]);
    }
}
pub(super) fn bau_aus_json(w: &mut Widget, wj: &serde_json::Value) {
    let mut z = Bauzusatz {
        kuerzen: match wj["kuerzen"].as_str().unwrap_or("") { "ende" => 1, "mitte" => 2, "anfang" => 3, _ => 0 },
        auswaehlbar: wj["auswaehlbar"].as_bool().unwrap_or(false),
        marken_n: wj["marken"].as_i64().unwrap_or(0).clamp(0, 101) as i32,
        marken: wj["marken_texte"].as_array().map(|a| a.iter().filter_map(|p| {
            let p = p.as_array()?;
            Some((p.first()?.as_f64()?, p.get(1)?.as_str()?.to_string()))
        }).collect()).unwrap_or_default(),
        wert_zeigen: wj["wert_zeigen"].as_bool().unwrap_or(false),
        einrasten: wj["einrasten"].as_bool().unwrap_or(false),
        stellen: wj["stellen"].as_i64().unwrap_or(0).clamp(0, 6) as i32,
        kreis: wj["kreis"].as_bool().unwrap_or(false),
        ..Default::default()
    };
    if let Some(gn) = wj["groessengruppe"].as_str().filter(|s| !s.is_empty()) {
        z.gruppe = gn.to_string();
        z.gruppe_art = match wj["groessengruppe_art"].as_str().unwrap_or("breite") { "hoehe" => 2, "beide" => 3, _ => 1 };
        z.eigen = (w.w, w.h);
    }
    let leer = z.kuerzen == 0 && !z.auswaehlbar && z.marken_n == 0 && z.marken.is_empty() && !z.wert_zeigen
        && !z.einrasten && z.stellen == 0 && !z.kreis && z.gruppe_art == 0;
    if !leer { w.bau = Some(Box::new(z)); }
}

fn ohne_marke_wenn(an: bool, s: &str) -> String { if an { ohne_marke(s).0 } else { s.to_string() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alt_marken() {
        assert_eq!(ohne_marke("&Speichern"), ("Speichern".into(), Some(0)));
        assert_eq!(ohne_marke("Sp&eichern"), ("Speichern".into(), Some(2)));
        assert_eq!(ohne_marke("A && B"), ("A & B".into(), None));
        assert_eq!(ohne_marke("A & B"), ("A & B".into(), None));
        assert_eq!(ohne_marke("&A&B"), ("A&B".into(), Some(0)));
        assert_eq!(ohne_marke("Grö&ße"), ("Größe".into(), Some(3)));
    }

    #[test]
    fn kuerzen_drei_arten() {
        let m = |s: &str| s.chars().count() as i32;
        assert_eq!(kuerzen_mit("abcdefghij", 1, 7, &m), "abcd...");
        assert_eq!(kuerzen_mit("abcdefghij", 3, 7, &m), "...ghij");
        assert_eq!(kuerzen_mit("abcdefghij", 2, 7, &m), "ab...ij");
        assert_eq!(kuerzen_mit("abc", 1, 7, &m), "abc");
        assert_eq!(kuerzen_mit("abcdefghij", 0, 3, &m), "abcdefghij");
    }
}
