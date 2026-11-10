//! Datenbindung, Formularpruefung, Sinnbilder im Baum, rollendes Panel, Ziehen zwischen Widgets, Bedienung ohne Maus.
//!
//! Teil von `gui.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl Gui {
    // ---------------------------------------------------------------- Datenbindung
    /// GUI_BIND(widget, schluessel$[, formular$]): ein Widget traegt einen
    /// Schluessel -- den Spaltennamen, unter dem sein Wert gelesen und
    /// gesetzt wird. Mehrere Formulare in einem Fenster (Reiter!) halten
    /// sich mit dem Formularnamen auseinander.
    pub fn bind(&mut self, h: i64, key: String, form: String) -> Result<(), String> {
        if !key.is_empty() && !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err("GUI_BIND: der Schluessel darf nur Buchstaben, Ziffern und _ enthalten -- er wird ein Spaltenname".into());
        }
        let (wi, i) = Self::dec_widget(h);
        let w = self.wdg_mut(h, "GUI_BIND")?;
        if !matches!(w.kind, Kind::TextInput | Kind::TextArea | Kind::Label | Kind::Checkbox | Kind::Toggle
                            | Kind::Dropdown | Kind::Slider | Kind::Spinner | Kind::Knob | Kind::DatePicker | Kind::ColorPicker) {
            return Err("GUI_BIND: dieses Widget traegt keinen Wert (Textfeld, Kaestchen, Klappliste, Regler, Datum, Farbe)".into());
        }
        w.bind = key; w.form = form;
        // Der Stand beim Binden ist der saubere Stand -- meist leer.
        self.form_stand_merken(wi, i);
        Ok(())
    }
    /// Die gebundenen Widgets eines Formulars, in Reihenfolge: (Schluessel, Index).
    pub(super) fn form_widgets(&self, wi: usize, form: &str) -> Vec<(String, usize)> {
        self.windows[wi].widgets.iter().enumerate()
            .filter(|(_, w)| w.alive && !w.bind.is_empty() && w.form == form)
            .map(|(i, w)| (w.bind.clone(), i)).collect()
    }
    /// Wert eines Widgets als Text -- die Form fuer Vergleich und MAP.
    /// Die Werte einer Klappliste, wenn sie welche hat (GUI_LISTBOX_SET_DATA).
    pub(super) fn klapp_werte(w: &Widget) -> Option<&Vec<String>> {
        w.list.as_ref().map(|l| &l.daten).filter(|d| d.iter().any(|x| !x.is_empty()))
    }
    pub(super) fn wert_text(&self, w: &Widget) -> String {
        match w.kind {
            Kind::Checkbox | Kind::Toggle => if w.checked { "1".into() } else { "0".into() },
            Kind::Dropdown => match Self::klapp_werte(w) {
                Some(d) => if w.sel >= 0 { d.get(w.sel as usize).cloned().unwrap_or_default() } else { String::new() },
                None => w.sel.to_string(),
            },
            Kind::Slider | Kind::Spinner | Kind::Knob => Self::fmt_num(w.value),
            Kind::DatePicker => crate::kalender::format(w.datum[0], w.datum[1], w.datum[2]),
            Kind::ColorPicker => format!("#{:06X}", crate::farbraum::hsv_zu_rgb(w.hsv[0], w.hsv[1], w.hsv[2])),
            _ => w.text.clone(),
        }
    }
    /// Wert eines Widgets typisiert -- fuer die Datenbank: ein Zahlenfeld
    /// liefert eine Zahl, ein Kaestchen ein BOOLEAN, eine Klappliste ihren
    /// Index, ein Regler eine FLOAT, alles andere Text.
    pub(super) fn wert_typisiert(&self, w: &Widget) -> Value {
        match w.kind {
            Kind::Checkbox | Kind::Toggle => Value::Bool(w.checked),
            // Mit Werten je Eintrag: der Wert (als Zahl, wenn er eine ist),
            // ohne Auswahl NIL; sonst die Nummer des Eintrags.
            Kind::Dropdown => match Self::klapp_werte(w) {
                Some(d) => {
                    let t = if w.sel >= 0 { d.get(w.sel as usize).cloned().unwrap_or_default() } else { String::new() };
                    if w.sel < 0 { Value::Nil }
                    else if let Ok(n) = t.trim().parse::<i64>() { Value::Int(n) }
                    else if let Some(f) = zahl_lesen(&t).filter(|_| t.trim().contains('.')) { Value::Float(f) }
                    else { Value::str_rc(&t) }
                }
                None => Value::Int(w.sel as i64),
            },
            Kind::Slider | Kind::Knob => Value::Float(w.value),
            Kind::Spinner => if w.step.fract() == 0.0 && w.value.fract() == 0.0 { Value::Int(w.value as i64) } else { Value::Float(w.value) },
            // Ein Zahlenfeld liefert eine Zahl -- AUSSER es beginnt mit einer
            // Null und hat mehr als eine Stelle: das ist eine Postleitzahl
            // oder eine Nummer, und die verloere als INTEGER ihre Null.
            // Ein Geldfeld (zahlen 3) liefert ganze Cent.
            Kind::TextInput if w.zahlen > 0 && !nummer_mit_null(&w.text) => match zahl_lesen(&w.text) {
                Some(x) if w.zahlen == 3 => Value::Int((x * 100.0).round() as i64),
                Some(x) if w.zahlen == 1 && x.fract() == 0.0 => Value::Int(x as i64),
                Some(x) => Value::Float(x),
                None => Value::str_rc(&w.text),
            },
            _ => Value::str_rc(&self.wert_text(w)),
        }
    }
    /// Einen Wert in ein Widget schreiben -- so tolerant, wie eine Datenbank
    /// oder eine MAP ihn liefern kann (Zahl als Text, "1" als Haken ...).
    pub(super) fn wert_setzen(&mut self, wi: usize, i: usize, v: &Value) {
        let text = match v { Value::Str(s) => s.to_string(), other => other.fmt() };
        let zahl = match v {
            Value::Int(n) => Some(*n as f64), Value::Float(f) => Some(*f),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => zahl_lesen(&text),
        };
        let items = self.windows[wi].widgets[i].items.clone();
        let w = &mut self.windows[wi].widgets[i];
        match w.kind {
            Kind::Checkbox | Kind::Toggle => {
                w.checked = match v {
                    Value::Bool(b) => *b,
                    _ => zahl.map(|x| x != 0.0).unwrap_or_else(|| matches!(text.to_ascii_lowercase().as_str(), "true" | "ja" | "wahr" | "x")),
                };
                if w.kind == Kind::Toggle { w.value = if w.checked { 1.0 } else { 0.0 }; }
            }
            Kind::Dropdown if Self::klapp_werte(w).is_some() => {
                // Gesucht wird der WERT: als Text, und bei Zahlen auch dem Betrag
                // nach (aus der Datenbank kommt 19 oder 19.0 oder "19").
                let d = Self::klapp_werte(w).cloned().unwrap_or_default();
                w.sel = d.iter().position(|x| {
                    *x == text || (zahl.is_some() && zahl_lesen(x) == zahl)
                }).map(|p| p as i32).unwrap_or(-1);
            }
            Kind::Dropdown => {
                let n = items.len() as i32;
                w.sel = match v {
                    Value::Int(k) => (*k as i32).clamp(-1, n - 1),
                    Value::Float(f) => (*f as i32).clamp(-1, n - 1),
                    _ => items.iter().position(|it| it.eq_ignore_ascii_case(text.trim())).map(|p| p as i32)
                            .or_else(|| zahl.map(|x| (x as i32).clamp(-1, n - 1))).unwrap_or(-1),
                };
            }
            Kind::Slider | Kind::Spinner | Kind::Knob => {
                if let Some(x) = zahl { w.value = x.clamp(w.min, w.max); }
            }
            Kind::DatePicker => {
                if let Some((j, m, d)) = crate::kalender::parse(text.trim()) { w.datum = [j, m, d]; }
            }
            Kind::ColorPicker => {
                let rgb = match v {
                    Value::Int(n) => Some(*n & 0xFF_FFFF),
                    _ => i64::from_str_radix(text.trim().trim_start_matches('#'), 16).ok(),
                };
                if let Some(rgb) = rgb {
                    let (h, sa, va) = crate::farbraum::rgb_zu_hsv(rgb);
                    w.hsv = [h, sa, va];
                }
            }
            _ => {
                // Textfelder: eine Zahl aus der Datenbank kommt als Text an,
                // ein Zahlenfeld mit Komma-Stellen zeigt sie deutsch.
                // Ein Geldfeld bekommt eine ZAHL als Cent; ein Text steht so
                // da, wie er kommt (er ist schon die Anzeige).
                let t = match v {
                    Value::Float(f) if w.zahlen == 2 => format!("{:.2}", f).replace('.', ","),
                    Value::Int(n) if w.zahlen == 3 => cent_text(*n),
                    Value::Float(f) if w.zahlen == 3 => cent_text(f.round() as i64),
                    _ => text,
                };
                w.text = t;
                w.caret = 0; w.sel_anchor = 0; w.scroll = 0; w.scroll_x = 0;
                w.undo.clear(); w.redo.clear();
            }
        }
    }
    pub(super) fn form_stand_merken(&mut self, wi: usize, i: usize) {
        let (form, key, text) = {
            let w = &self.windows[wi].widgets[i];
            (w.form.clone(), w.bind.clone(), self.wert_text(w))
        };
        let st = &mut self.windows[wi].form_stand;
        st.retain(|(f, k, _)| !(*f == form && *k == key));
        st.push((form, key, text));
    }
    /// GUI_FORM_GET: alle gebundenen Werte eines Formulars, als Text.
    pub fn form_get(&self, win: i64, form: &str) -> Result<Vec<(String, String)>, String> {
        let wi = win as usize;
        self.windows.get(wi).ok_or("GUI_FORM_GET: erwartet GUI_WINDOW")?;
        Ok(self.form_widgets(wi, form).into_iter()
            .map(|(k, i)| (k, self.wert_text(&self.windows[wi].widgets[i]))).collect())
    }
    /// Dasselbe typisiert (fuer die Datenbank).
    pub fn form_get_typisiert(&self, win: i64, form: &str) -> Result<Vec<(String, Value)>, String> {
        let wi = win as usize;
        self.windows.get(wi).ok_or("GUI_FORM_SAVE: erwartet GUI_WINDOW")?;
        Ok(self.form_widgets(wi, form).into_iter()
            .map(|(k, i)| (k, self.wert_typisiert(&self.windows[wi].widgets[i]))).collect())
    }
    /// GUI_FORM_SET: Werte nach Schluessel eintragen; was nicht dabei ist,
    /// bleibt stehen. Danach gilt der Stand als sauber.
    pub fn form_set(&mut self, win: i64, form: &str, werte: &[(String, Value)]) -> Result<(), String> {
        let wi = win as usize;
        self.windows.get(wi).ok_or("GUI_FORM_SET: erwartet GUI_WINDOW")?;
        for (k, i) in self.form_widgets(wi, form) {
            if let Some((_, v)) = werte.iter().find(|(wk, _)| wk.eq_ignore_ascii_case(&k)) {
                self.wert_setzen(wi, i, v);
            }
        }
        self.form_clean(win, form)
    }
    /// GUI_FORM_CLEAR: alle gebundenen Widgets leeren (Text leer, Haken weg,
    /// Klappliste ohne Auswahl, Regler auf Minimum). Danach sauber.
    pub fn form_clear(&mut self, win: i64, form: &str) -> Result<(), String> {
        let wi = win as usize;
        self.windows.get(wi).ok_or("GUI_FORM_CLEAR: erwartet GUI_WINDOW")?;
        for (_, i) in self.form_widgets(wi, form) {
            let w = &mut self.windows[wi].widgets[i];
            match w.kind {
                Kind::Checkbox | Kind::Toggle => { w.checked = false; w.value = 0.0; }
                Kind::Dropdown => w.sel = -1,
                Kind::Slider | Kind::Spinner | Kind::Knob => w.value = w.min,
                Kind::DatePicker | Kind::ColorPicker => {}
                _ => { w.text.clear(); w.caret = 0; w.sel_anchor = 0; w.scroll = 0; w.scroll_x = 0; w.undo.clear(); w.redo.clear(); }
            }
            w.fehler.clear();
        }
        self.form_clean(win, form)
    }
    /// GUI_FORM_CLEAN: der jetzige Stand gilt als gespeichert.
    pub fn form_clean(&mut self, win: i64, form: &str) -> Result<(), String> {
        let wi = win as usize;
        self.windows.get(wi).ok_or("GUI_FORM_CLEAN: erwartet GUI_WINDOW")?;
        for (_, i) in self.form_widgets(wi, form) { self.form_stand_merken(wi, i); }
        Ok(())
    }
    /// GUI_FORM_CHANGED: weicht ein gebundener Wert vom sauberen Stand ab?
    pub fn form_changed(&self, win: i64, form: &str) -> Result<bool, String> {
        let wi = win as usize;
        let w = self.windows.get(wi).ok_or("GUI_FORM_CHANGED: erwartet GUI_WINDOW")?;
        for (k, i) in self.form_widgets(wi, form) {
            let jetzt = self.wert_text(&w.widgets[i]);
            let alt = w.form_stand.iter().find(|(f, kk, _)| f == form && *kk == k).map(|(_, _, t)| t.as_str()).unwrap_or("");
            if jetzt != alt { return Ok(true); }
        }
        Ok(false)
    }
    pub(super) fn form_db_mut(&mut self, wi: usize, form: &str) -> &mut FormDb {
        let v = &mut self.windows[wi].form_db;
        let pos = match v.iter().position(|f| f.form == form) {
            Some(p) => p,
            None => { v.push(FormDb { form: form.to_string(), ..Default::default() }); v.len() - 1 }
        };
        &mut v[pos]
    }
    /// GUI_FORM_VERSION(win, spalte$[, formular$]): diese Spalte zaehlt jedes
    /// Speichern hoch, und GUI_FORM_SAVE speichert nur, wenn sie noch den
    /// Stand vom Laden traegt. Leer schaltet ab.
    pub fn form_version_setzen(&mut self, win: i64, form: &str, spalte: &str) -> Result<(), String> {
        let wi = win as usize;
        self.windows.get(wi).ok_or("GUI_FORM_VERSION: erwartet GUI_WINDOW")?;
        if !spalte.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(format!("GUI_FORM_VERSION: '{}' ist kein Spaltenname (Buchstaben, Ziffern, _)", spalte));
        }
        if !spalte.is_empty() && self.form_widgets(wi, form).iter().any(|(k, _)| k.eq_ignore_ascii_case(spalte)) {
            return Err(format!("GUI_FORM_VERSION: '{}' ist an ein Widget gebunden -- die Versionsspalte fuehrt die Laufzeit selbst", spalte));
        }
        let f = self.form_db_mut(wi, form);
        f.spalte = spalte.to_string();
        f.stand = None;
        Ok(())
    }
    /// (Versionsspalte, beim Laden gelesener Stand) -- Spalte leer = keine.
    pub fn form_version(&self, win: i64, form: &str) -> (String, Option<i64>) {
        self.windows.get(win as usize)
            .and_then(|w| w.form_db.iter().find(|f| f.form == form))
            .map(|f| (f.spalte.clone(), f.stand)).unwrap_or_default()
    }
    pub fn form_stand_setzen(&mut self, win: i64, form: &str, stand: Option<i64>) {
        if (win as usize) < self.windows.len() { self.form_db_mut(win as usize, form).stand = stand; }
    }
    pub fn form_konflikt_setzen(&mut self, win: i64, form: &str, k: &str) {
        if (win as usize) < self.windows.len() { self.form_db_mut(win as usize, form).konflikt = k.to_string(); }
    }
    /// GUI_FORM_CONFLICT$: "" (gespeichert), "geaendert" oder "geloescht".
    pub fn form_konflikt(&self, win: i64, form: &str) -> Result<String, String> {
        let w = self.windows.get(win as usize).ok_or("GUI_FORM_CONFLICT$: erwartet GUI_WINDOW")?;
        Ok(w.form_db.iter().find(|f| f.form == form).map(|f| f.konflikt.clone()).unwrap_or_default())
    }
    /// Die Schluessel eines Formulars (fuer SELECT/INSERT/UPDATE in der VM).
    pub fn form_keys(&self, win: i64, form: &str) -> Result<Vec<String>, String> {
        let wi = win as usize;
        self.windows.get(wi).ok_or("GUI_FORM_LOAD: erwartet GUI_WINDOW")?;
        Ok(self.form_widgets(wi, form).into_iter().map(|(k, _)| k).collect())
    }

    // ---------------------------------------------------------------- Formularpruefung
    /// GUI_RULE(feld, art$, ...): eine Regel anhaengen. Erlaubt an Textfeld,
    /// Textbereich, Klappliste und Kaestchen (dort nur `pflicht`).
    pub fn rule_add(&mut self, h: i64, art: &str, a: f64, b: f64, text: String, meldung: String) -> Result<(), String> {
        let art = RegelArt::from_str(art).ok_or_else(|| format!(
            "GUI_RULE: '{}' ist keine Regel -- pflicht, zahl, ganz, bereich, laenge, email, datum oder muster", art))?;
        if art == RegelArt::Muster {
            // Die Meldung der regex-Crate ist mehrzeilig (mit Zeiger unter das
            // Muster) -- fuer eine Fehlermeldung reicht die letzte Zeile.
            regex::Regex::new(&text).map_err(|e| format!("GUI_RULE: ungueltiges Muster '{}': {}", text,
                e.to_string().lines().last().unwrap_or("").trim()))?;
        }
        if art == RegelArt::Bereich && a > b { return Err("GUI_RULE: bereich -- von muss <= bis sein".into()); }
        if art == RegelArt::Laenge && (a < 0.0 || a > b) { return Err("GUI_RULE: laenge -- min muss zwischen 0 und max liegen".into()); }
        let w = self.wdg_mut(h, "GUI_RULE")?;
        match w.kind {
            Kind::TextInput | Kind::TextArea => {}
            Kind::Dropdown | Kind::Checkbox | Kind::Toggle if art == RegelArt::Pflicht => {}
            Kind::Dropdown | Kind::Checkbox | Kind::Toggle =>
                return Err("GUI_RULE: an Klappliste und Kaestchen geht nur 'pflicht'".into()),
            _ => return Err("GUI_RULE: Regeln gehen an Textfeld, Textbereich, Klappliste und Kaestchen".into()),
        }
        w.regeln.push(Regel { art, a, b, text, meldung });
        Ok(())
    }
    pub fn rules_clear(&mut self, h: i64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_RULES_CLEAR")?;
        w.regeln.clear(); w.fehler.clear();
        Ok(())
    }
    /// Alle Regeln EINES Widgets pruefen; setzt seine Meldung und liefert sie.
    pub(super) fn widget_pruefen(&mut self, wi: usize, i: usize) -> String {
        let (text, gewaehlt, regeln) = {
            let w = &self.windows[wi].widgets[i];
            let gewaehlt = match w.kind {
                Kind::Dropdown => w.sel >= 0,
                Kind::Checkbox | Kind::Toggle => w.checked,
                _ => false,
            };
            let text = match w.kind { Kind::Dropdown | Kind::Checkbox | Kind::Toggle => String::new(), _ => w.text.clone() };
            (text, gewaehlt, w.regeln.clone())
        };
        let mut meldung = String::new();
        for r in &regeln {
            if let Some(m) = regel_pruefen(r, &text, gewaehlt) { meldung = m; break; }
        }
        self.fehler_setzen(wi, i, meldung.clone());
        meldung
    }
    /// Meldung an ein Widget haengen und, wenn eine Beschriftung gebunden
    /// ist, dort anzeigen -- EINE Stelle dafuer.
    pub(super) fn fehler_setzen(&mut self, wi: usize, i: usize, meldung: String) {
        let label = {
            let w = &mut self.windows[wi].widgets[i];
            w.fehler = meldung;
            w.fehler_label
        };
        if label < 0 { return; }
        // Mehrere Felder duerfen sich EINE Beschriftung teilen (ein Formular,
        // eine Fehlerzeile). Sie zeigt die Meldung des ersten Feldes, das
        // eine hat -- sonst loeschte das naechste fehlerfreie Feld die
        // Meldung des falschen wieder.
        let anzeige = self.windows[wi].widgets.iter()
            .find(|w| w.fehler_label == label && !w.fehler.is_empty())
            .map(|w| w.fehler.clone()).unwrap_or_default();
        if let Some(l) = self.windows[wi].widgets.get_mut(label as usize) { l.text = anzeige; }
    }
    /// GUI_VALIDATE(win): alle Widgets mit Regeln pruefen; Zahl der Fehler.
    /// Das erste fehlerhafte Feld bekommt den Fokus -- der Nutzer soll nicht
    /// suchen muessen, wo es hakt.
    pub fn validate(&mut self, win: i64) -> Result<i64, String> {
        let wi = win as usize;
        self.win_mut(win, "GUI_VALIDATE")?;
        let n = self.windows[wi].widgets.len();
        let mut fehler = 0;
        let mut erstes: Option<usize> = None;
        for i in 0..n {
            let w = &self.windows[wi].widgets[i];
            if w.regeln.is_empty() || !w.alive { continue; }
            let sichtbar = self.widget_shown(wi, w) && w.enabled;
            if !sichtbar { self.fehler_setzen(wi, i, String::new()); continue; }
            if !self.widget_pruefen(wi, i).is_empty() {
                fehler += 1;
                if erstes.is_none() { erstes = Some(i); }
            }
        }
        if let Some(i) = erstes {
            if self.windows[wi].widgets[i].kind.fokussierbar() {
                self.focus_widget = Some((wi, i));
                self.focus_window = Some(wi);
            }
        }
        Ok(fehler)
    }
    /// GUI_VALIDATE_WIDGET(feld): nur dieses Feld; liefert die Meldung ("" = ok).
    pub fn validate_widget(&mut self, h: i64) -> Result<String, String> {
        let (wi, i) = Self::dec_widget(h);
        self.wdg(h, "GUI_VALIDATE_WIDGET")?;
        Ok(self.widget_pruefen(wi, i))
    }
    pub fn error_get(&self, h: i64) -> Result<String, String> { Ok(self.wdg(h, "GUI_ERROR")?.fehler.clone()) }
    /// GUI_SET_ERROR(feld, meldung$): eine Meldung von aussen -- etwa aus der
    /// Datenbank ("Nummer schon vergeben"). Leer nimmt sie zurueck.
    pub fn error_set(&mut self, h: i64, meldung: String) -> Result<(), String> {
        let (wi, i) = Self::dec_widget(h);
        self.wdg(h, "GUI_SET_ERROR")?;
        self.fehler_setzen(wi, i, meldung);
        Ok(())
    }
    pub fn errors_clear(&mut self, win: i64) -> Result<(), String> {
        let wi = win as usize;
        self.win_mut(win, "GUI_CLEAR_ERRORS")?;
        for i in 0..self.windows[wi].widgets.len() {
            if !self.windows[wi].widgets[i].fehler.is_empty() { self.fehler_setzen(wi, i, String::new()); }
        }
        Ok(())
    }
    /// GUI_ERROR_LABEL(feld, label): die Beschriftung zeigt die Meldung des
    /// Feldes -- dort, wo das Programm sie haben will, statt nur im Tooltip.
    pub fn error_label(&mut self, h: i64, label: i64) -> Result<(), String> {
        let (wi, i) = Self::dec_widget(h);
        self.wdg(h, "GUI_ERROR_LABEL")?;
        if label < 0 {
            self.windows[wi].widgets[i].fehler_label = -1;
            return Ok(());
        }
        let (lw, li) = Self::dec_widget(label);
        let l = self.wdg(label, "GUI_ERROR_LABEL")?;
        if l.kind != Kind::Label { return Err("GUI_ERROR_LABEL: das zweite Widget muss eine Beschriftung sein".into()); }
        if lw != wi { return Err("GUI_ERROR_LABEL: Feld und Beschriftung liegen in verschiedenen Fenstern".into()); }
        self.windows[wi].widgets[i].fehler_label = li as i32;
        Ok(())
    }
    pub fn validate_live(&mut self, win: i64, an: bool) -> Result<(), String> {
        self.win_mut(win, "GUI_VALIDATE_LIVE")?.pruefung_live = an;
        Ok(())
    }

    // ---------------------------------------------------------------- Punkt 6
    /// Senkrechter Schieber: wie GUI_SLIDER, nur steht `h` statt `w`, und der
    /// Wert waechst nach oben.
    pub fn vslider(&mut self, win: i64, x: i32, y: i32, h: i32, mn: f64, mx: f64, default: f64) -> Result<i64, String> {
        if mx <= mn { return Err("GUI_VSLIDER: max muss > min sein".into()); }
        let sw = self.m_roh("slider_h");
        let mut wd = Self::blank(Kind::Slider, x, y, sw, h);
        wd.min = mn; wd.max = mx; wd.value = default.clamp(mn, mx); wd.vert = true;
        self.add_widget(win, "GUI_VSLIDER", wd)
    }
    pub fn progress_set(&mut self, h: i64, key: &str, wert: f64) -> Result<(), String> {
        let w = self.wdg_mut(h, "GUI_PROGRESS_SET")?;
        if w.kind != Kind::Progress { return Err("GUI_PROGRESS_SET: Widget ist kein Fortschrittsbalken".into()); }
        match key.to_ascii_lowercase().as_str() {
            "unbestimmt" | "indeterminate" => { w.unbestimmt = wert != 0.0; Ok(()) }
            "kreis" | "circle" => self.progress_kreis_setzen(h, wert != 0.0),
            k => Err(format!("GUI_PROGRESS_SET: unbekannter Schluessel '{}' -- erwartet unbestimmt oder kreis", k)),
        }
    }
    pub fn image_mode(&mut self, h: i64, modus: &str) -> Result<(), String> {
        let m = bildmodus_parsen(modus).ok_or_else(|| format!(
            "GUI_IMAGE_MODE: '{}' ist kein Modus -- strecken, einpassen, fuellen, mitte oder kacheln", modus))?;
        let w = self.wdg_mut(h, "GUI_IMAGE_MODE")?;
        if !matches!(w.kind, Kind::Image | Kind::Card) { return Err("GUI_IMAGE_MODE: Widget ist kein image und keine Kachel".into()); }
        w.bildmodus = m;
        Ok(())
    }
    pub fn image_mode_get(&self, h: i64) -> Result<String, String> {
        Ok(bildmodus_str(self.wdg(h, "GUI_IMAGE_MODE_GET")?.bildmodus).to_string())
    }
    pub fn cursors(&mut self, an: bool) { self.cursors = an; }

    /// GUI_SET_CURSOR(wdg, form$): eigene Zeigerform ueber diesem Widget
    /// ("auto" oder "" = die gui entscheidet) -- fuer Zeichenflaechen, auf
    /// denen das Programm selbst etwas anbietet.
    pub fn set_cursor(&mut self, h: i64, form: &str) -> Result<(), String> {
        let f = if matches!(form.trim().to_ascii_lowercase().as_str(), "" | "auto") { None } else {
            Some(crate::graphics::zeiger_form(form).ok_or_else(|| format!(
                "GUI_SET_CURSOR: unbekannte Form '{}' -- erwartet {} oder auto", form, crate::graphics::ZEIGER_LISTE))?)
        };
        self.wdg_mut(h, "GUI_SET_CURSOR")?.zeiger = f;
        Ok(())
    }
    pub fn get_cursor(&self, h: i64) -> Result<&'static str, String> {
        Ok(self.wdg(h, "GUI_GET_CURSOR$")?.zeiger.unwrap_or("auto"))
    }

    /// Welche Form die Maus bekommt -- je Bild als Wunsch an Graphics, das sie
    /// beim FLIP setzt (eine Form des Programms per MOUSE_CURSOR gewinnt).
    /// Zuerst laufende Gesten (sie gelten auch neben dem Widget), dann das
    /// oberste Widget unter der Maus, dort nach der STELLE (`zeiger_ueber`).
    pub(super) fn cursor_pass(&mut self, g: &mut Graphics, mx: i32, my: i32) {
        if !self.cursors { g.zeiger_gui(None); return; }
        let form = self.zeiger_wahl(g, mx, my);
        g.zeiger_gui(form);
    }

    pub(super) fn zeiger_wahl(&self, g: &Graphics, mx: i32, my: i32) -> Option<&'static str> {
        if let Some(d) = self.drag.as_ref().filter(|d| d.aktiv) {
            // Ueber einem Ablageziel oder der eigenen Quelle (dort sortiert sie
            // um) geht es, sonst nicht -- das soll man vor dem Loslassen sehen.
            let (qw, qi) = d.quelle;
            let ueber_quelle = self.windows.get(qw).and_then(|w| w.widgets.get(qi))
                .map(|w| Self::in_rect(mx, my, self.abs_rect(qw, w))).unwrap_or(false);
            return Some(if ueber_quelle || self.ablage_unter(mx, my).is_some() { "hand" } else { "not_allowed" });
        }
        if self.resize_window.is_some() { return Some("resize_nwse"); }
        if let Some(f) = self.dock_zeiger(mx, my) { return Some(f); }
        if let Some((sw, si)) = self.active_split {
            return Some(if self.windows[sw].widgets[si].group == "v" { "resize_ew" } else { "resize_ns" });
        }
        if let Some((tw, ti)) = self.active_table {
            if let Some(t) = self.windows.get(tw).and_then(|w| w.widgets.get(ti)).and_then(|w| w.tbl.as_ref()) {
                if t.drag_col >= 0 { return Some("resize_ew"); }
            }
        }
        let top = self.topmost_at(mx, my)?;
        let win = &self.windows[top];
        if win.chrome && win.resizable && Self::in_rect(mx, my, (win.x + win.w - 14, win.y + win.h - 14, 14, 14)) {
            return Some("resize_nwse");
        }
        let i = win.widgets.iter().rposition(|w| w.hovered)?;
        let w = &win.widgets[i];
        if let Some(f) = w.zeiger { return Some(f); }
        if !w.enabled { return None; }
        self.zeiger_ueber(g, top, i, mx, my)
    }

    /// Die Form ueber EINER Stelle eines Widgets. Jede Zeile fragt dieselbe
    /// Geometrie wie Treffertest und Zeichnen -- ein Zeiger, der etwas
    /// anbietet, was der Klick dort nicht tut, waere schlimmer als der Pfeil.
    pub(super) fn zeiger_ueber(&self, g: &Graphics, wi: usize, i: usize, mx: i32, my: i32) -> Option<&'static str> {
        let w = &self.windows[wi].widgets[i];
        let (ax, ay, bw, bh) = self.abs_rect(wi, w);
        match w.kind {
            Kind::TextInput => Some("ibeam"),
            Kind::TextArea => {
                if let Some((tx, ty, tb, th, _, _, _, _)) = self.ta_bar_geom(g, wi, i) {
                    if Self::in_rect(mx, my, (tx - 2, ty, tb + 4, th)) { return None; }
                }
                let chars: Vec<char> = w.text.chars().collect();
                let starts = Self::line_starts(&chars);
                if mx < ax + self.ta_gutter(g, w, starts.len()) { return None; }
                if w.hinweise_klickbar && !w.hinweise.is_empty() {
                    let rows = self.ta_rows(g, w, &chars, &starts, self.ta_breite(g, w, starts.len()));
                    let sicht = ((bh - 10) / self.ta_line_h(g, w)).max(1);
                    if self.ta_hinweis_rects(g, w, ax, ay, &chars, &starts, &rows, sicht)
                        .into_iter().any(|(_, hx, hy, hb, hh)| Self::in_rect(mx, my, (hx, hy, hb, hh))) {
                        return Some("hand");
                    }
                }
                if !w.farbfelder.is_empty() {
                    let rows = self.ta_rows(g, w, &chars, &starts, self.ta_breite(g, w, starts.len()));
                    let sicht = ((bh - 10) / self.ta_line_h(g, w)).max(1);
                    if self.ta_farbfeld_rects(g, w, ax, ay, &chars, &rows, starts.len(), sicht)
                        .into_iter().any(|(_, fx, fy, k)| Self::in_rect(mx, my, (fx, fy, k, k))) {
                        return Some("hand");
                    }
                }
                Some("ibeam")
            }
            Kind::Button | Kind::Card | Kind::Checkbox | Kind::Radio | Kind::Toggle => Some("hand"),
            Kind::Splitter => Some(if w.group == "v" { "resize_ew" } else { "resize_ns" }),
            Kind::Table => {
                let t = w.tbl.as_ref()?;
                if self.editing_table == Some((wi, i)) {
                    if let Some(r) = self.edit_cell_rect(wi, i) {
                        if Self::in_rect(mx, my, r) { return Some("ibeam"); }
                    }
                }
                let gm = self.table_geom(wi, i);
                if my >= gm.ay && my < gm.ay + t.header_h {
                    if t.resizable_cols && Self::edge_at(&gm, mx, self.sk(TBL_EDGE_GRAB)) >= 0 { return Some("resize_ew"); }
                    if t.sortable && Self::pos_at(&gm, mx) >= 0 { return Some("hand"); }
                    return None;
                }
                if t.filter_row && my < gm.ay + t.header_h + self.sk(TBL_FILTER_H) && Self::col_at(&gm, mx) >= 0 {
                    return Some("ibeam");
                }
                None
            }
            Kind::ListBox | Kind::Tree => {
                match self.edit_rect_any(wi, i) {
                    Some(r) if Self::in_rect(mx, my, r) => Some("ibeam"),
                    _ => None,
                }
            }
            Kind::Dropdown => {
                // Die bearbeitbare Klappliste ist links ein Textfeld, rechts
                // der Pfeil.
                if w.frei && mx < ax + bw - self.sk(DD_PFEIL_B) { Some("ibeam") } else { None }
            }
            Kind::RichText => {
                let r = w.rich.as_ref()?;
                if Self::rt_link_unter(r, mx - ax, my - ay + r.scroll).is_some() { Some("hand") } else { Some("ibeam") }
            }
            Kind::ColorPicker => {
                let (feld, ton, deck) = self.cp_geom(w, ax, ay, bw, bh);
                let drin = Self::in_rect(mx, my, feld) || Self::in_rect(mx, my, ton)
                    || deck.map(|d| Self::in_rect(mx, my, d)).unwrap_or(false);
                if drin { Some("crosshair") } else { None }
            }
            Kind::Toolbar => {
                let l = w.leiste.as_ref()?;
                if self.tb_chevron_hit(wi, i, mx, my) { return Some("hand"); }
                let knopf = self.tb_geom(ax, ay, bw, bh, w).into_iter()
                    .any(|(k, x, b)| mx >= x && mx < x + b && l.eintraege[k].art == 0 && !l.eintraege[k].aus);
                if knopf { Some("hand") } else { None }
            }
            Kind::StatusBar => {
                let s = w.status.as_ref()?;
                let feld = self.sb_geom(ax, bw, w).into_iter()
                    .any(|(k, x, b)| mx >= x && mx < x + b && s.felder[k].klickbar);
                if feld { Some("hand") } else { None }
            }
            Kind::Breadcrumb => {
                let (teile, punkte) = self.pf_geom(ax, bw, w);
                let n = w.pfad.as_ref().map(|p| p.texte.len()).unwrap_or(0);
                // Der letzte Teil ist die Stelle, an der man steht -- ein Klick
                // darauf tut nichts.
                let teil = teile.into_iter().any(|(k, x, b)| mx >= x && mx < x + b && k + 1 < n);
                let pkt = punkte.map(|(x, b)| mx >= x && mx < x + b).unwrap_or(false);
                if teil || pkt { Some("hand") } else { None }
            }
            _ => None,
        }
    }

    // --- Baum: Sinnbild und Farbe je Knoten ---
    pub fn tree_icon(&mut self, h: i64, node: i64, tex: i64) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_ICON")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_ICON: ungueltige Knoten-id {}", node));
        }
        t.nodes[node as usize].icon = tex.max(-1);
        t.nodes[node as usize].sym.clear();
        Ok(())
    }
    /// Dasselbe mit einem eingebauten Sinnbild statt eines Bildes.
    pub fn tree_icon_sym(&mut self, h: i64, node: i64, name: &str) -> Result<(), String> {
        let sym = if name.trim().is_empty() { String::new() } else { sinnbild_pruefen(name, "GUI_TREE_ICON")? };
        let t = self.tree_mut(h, "GUI_TREE_ICON")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_ICON: ungueltige Knoten-id {}", node));
        }
        t.nodes[node as usize].icon = -1;
        t.nodes[node as usize].sym = sym;
        Ok(())
    }
    pub fn tree_color(&mut self, h: i64, node: i64, farbe: i64) -> Result<(), String> {
        let t = self.tree_mut(h, "GUI_TREE_COLOR")?;
        if node < 0 || node >= t.nodes.len() as i64 {
            return Err(format!("GUI_TREE_COLOR: ungueltige Knoten-id {}", node));
        }
        t.nodes[node as usize].color = farbe; Ok(())
    }

    // --- Rollendes Panel ---
    pub(super) fn panel_pruefen(&self, h: i64, fn_: &str) -> Result<(usize, usize), String> {
        let w = self.wdg(h, fn_)?;
        if w.kind != Kind::Panel { return Err(format!("{}: Widget ist kein Panel", fn_)); }
        Ok(Self::dec_widget(h))
    }
    pub fn panel_add(&mut self, h: i64, kind: i64) -> Result<(), String> {
        let (wi, pi) = self.panel_pruefen(h, "GUI_PANEL_ADD")?;
        let (kw, ki) = Self::dec_widget(kind);
        if self.wdg(kind, "GUI_PANEL_ADD").is_err() { return Err("GUI_PANEL_ADD: ungueltiges GUI_WIDGET-Handle".into()); }
        if kw != wi { return Err("GUI_PANEL_ADD: das Widget liegt in einem anderen Fenster".into()); }
        if ki == pi { return Err("GUI_PANEL_ADD: ein Panel kann sich nicht selbst aufnehmen".into()); }
        if self.windows[kw].widgets[ki].panel.is_some() {
            return Err("GUI_PANEL_ADD: ein rollendes Panel in einem rollenden Panel gibt es nicht".into());
        }
        // Aus jedem anderen Panel loesen -- ein Widget rollt mit hoechstens einem.
        for w in self.windows[wi].widgets.iter_mut() {
            if let Some(p) = w.panel.as_mut() { p.kinder.retain(|&k| k != ki); }
        }
        let p = self.windows[wi].widgets[pi].panel.get_or_insert_with(Default::default);
        p.kinder.push(ki);
        Ok(())
    }
    pub fn panel_remove(&mut self, h: i64, kind: i64) -> Result<(), String> {
        let (wi, pi) = self.panel_pruefen(h, "GUI_PANEL_REMOVE")?;
        let (_, ki) = Self::dec_widget(kind);
        if let Some(p) = self.windows[wi].widgets[pi].panel.as_mut() { p.kinder.retain(|&k| k != ki); }
        if let Some(w) = self.windows[wi].widgets.get_mut(ki) { if w.panel_von == pi as i32 { w.panel_von = -1; } }
        Ok(())
    }
    pub fn panel_scroll(&mut self, h: i64, y: i32) -> Result<(), String> {
        let (wi, pi) = self.panel_pruefen(h, "GUI_PANEL_SCROLL")?;
        let max = self.panel_max_scroll(wi, pi);
        let y = self.sk(y);
        match self.windows[wi].widgets[pi].panel.as_mut() {
            Some(p) => { p.scroll = y.clamp(0, max); Ok(()) }
            None => Err("GUI_PANEL_SCROLL: das Panel rollt nichts -- erst GUI_PANEL_ADD".into()),
        }
    }
    pub fn panel_scroll_get(&self, h: i64) -> Result<i64, String> {
        let (wi, pi) = self.panel_pruefen(h, "GUI_PANEL_SCROLL_GET")?;
        Ok(self.unsk(self.windows[wi].widgets[pi].panel.as_ref().map(|p| p.scroll).unwrap_or(0)) as i64)
    }
    pub(super) fn panel_max_scroll(&self, wi: usize, pi: usize) -> i32 {
        let w = &self.windows[wi].widgets[pi];
        let kopf = if w.text.is_empty() { 0 } else { 18 };
        w.panel.as_ref().map(|p| (p.inhalt_h - (w.h - kopf)).max(0)).unwrap_or(0)
    }
    pub(super) fn panel_scroll_um(&mut self, wi: usize, pi: usize, dy: i32) {
        let max = self.panel_max_scroll(wi, pi);
        if let Some(p) = self.windows[wi].widgets[pi].panel.as_mut() { p.scroll = (p.scroll + dy).clamp(0, max); }
    }
    /// Rollbalken eines Panels: (x, y, breite, hoehe, griff_y, griff_h) --
    /// nur wenn der Inhalt hoeher ist als das Panel. EINE Quelle fuer
    /// Zeichnen und Klick.
    pub(super) fn panel_bar_geom(&self, wi: usize, pi: usize) -> Option<(i32, i32, i32, i32, i32, i32)> {
        let w = &self.windows[wi].widgets[pi];
        let p = w.panel.as_ref()?;
        if !self.widget_shown(wi, w) { return None; }
        let kopf = if w.text.is_empty() { 0 } else { 18 };
        let sicht = w.h - kopf - 2;
        if p.inhalt_h <= sicht || sicht <= 0 { return None; }
        let (ax, ay, aw, _) = self.abs_rect(wi, w);
        let bw = self.sk(8);
        let (tx, ty, th) = (ax + aw - 1 - bw, ay + kopf + 1, sicht);
        let thh = ((th as i64 * sicht as i64 / p.inhalt_h.max(1) as i64) as i32).clamp(self.sk(16).min(th), th);
        let max = (p.inhalt_h - sicht).max(1);
        let thy = ty + ((th - thh) as i64 * p.scroll as i64 / max as i64) as i32;
        Some((tx, ty, bw, th, thy, thh))
    }
    /// Je Bild: welches Widget in welchem Panel steckt (durch Layouts
    /// hindurch), und wie hoch der Inhalt jedes Panels ist.
    pub(super) fn panel_pass(&mut self) {
        for wi in 0..self.windows.len() {
            let n = self.windows[wi].widgets.len();
            for i in 0..n { self.windows[wi].widgets[i].panel_von = -1; }
            for p in 0..n {
                let kinder = match self.windows[wi].widgets[p].panel.as_ref() {
                    Some(ps) => ps.kinder.clone(),
                    None => continue,
                };
                let mut menge = Vec::new();
                for k in kinder { self.panel_sammeln(wi, k, &mut menge, 0); }
                let py = self.windows[wi].widgets[p].y;
                let mut unten = 0;
                for &k in &menge {
                    if k == p { continue; }
                    let w = &mut self.windows[wi].widgets[k];
                    w.panel_von = p as i32;
                    if w.alive && w.visible { unten = unten.max(w.y + w.h - py); }
                }
                let pad = self.m("pad");
                let ph = self.windows[wi].widgets[p].h;
                let kopf = if self.windows[wi].widgets[p].text.is_empty() { 0 } else { 18 };
                if let Some(ps) = self.windows[wi].widgets[p].panel.as_mut() {
                    ps.inhalt_h = unten + pad;
                    ps.scroll = ps.scroll.clamp(0, (ps.inhalt_h - (ph - kopf)).max(0));
                }
            }
        }
    }
    pub(super) fn panel_sammeln(&self, wi: usize, k: usize, aus: &mut Vec<usize>, tiefe: u32) {
        if tiefe > 16 || aus.contains(&k) { return; }
        let w = match self.windows[wi].widgets.get(k) { Some(w) => w, None => return };
        aus.push(k);
        if let Some(l) = w.layout.as_ref() {
            for &(c, _) in &l.kinder { if c >= 0 { self.panel_sammeln(wi, c as usize, aus, tiefe + 1); } }
        }
    }

    // --- Ziehen zwischen Widgets ---
    pub fn draggable(&mut self, h: i64, an: bool) -> Result<(), String> { self.wdg_mut(h, "GUI_DRAGGABLE")?.ziehbar = an; Ok(()) }
    pub fn drop_target(&mut self, h: i64, an: bool) -> Result<(), String> { self.wdg_mut(h, "GUI_DROP_TARGET")?.ablage = an; Ok(()) }
    pub fn dragging(&self) -> i64 {
        self.drag.as_ref().filter(|d| d.aktiv).map(|d| Self::enc_widget(d.quelle.0, d.quelle.1)).unwrap_or(-1)
    }
    pub fn dropped(&self, h: i64) -> Result<bool, String> { Ok(self.wdg(h, "GUI_DROPPED")?.abgelegt) }
    pub fn drop_text(&self) -> String {
        if let Some(d) = self.drop.as_ref() { return d.text.clone(); }
        self.drag.as_ref().filter(|d| d.aktiv).map(|d| d.text.clone()).unwrap_or_default()
    }
    pub fn drop_source(&self) -> i64 {
        self.drop.as_ref().map(|d| Self::enc_widget(d.quelle.0, d.quelle.1)).unwrap_or_else(|| self.dragging())
    }
    pub fn drop_index(&self) -> i64 { self.drop.as_ref().map(|d| d.index_ziel as i64).unwrap_or(-1) }
    pub fn drag_index(&self) -> i64 {
        if let Some(d) = self.drop.as_ref() { return d.index_quelle as i64; }
        self.drag.as_ref().filter(|d| d.aktiv).map(|d| d.index as i64).unwrap_or(-1)
    }
    /// Was ein Druck auf ein ziehbares Widget mitnimmt: bei Liste, Baum und
    /// Tabelle die getroffene Zeile (bei der Tabelle die Datenzeile mit dem
    /// Text ihrer ersten Zelle), sonst das Widget selbst mit seinem Text.
    pub(super) fn zieh_nutzlast(&self, wi: usize, i: usize, mx: i32, my: i32) -> (i32, String) {
        let w = &self.windows[wi].widgets[i];
        match w.kind {
            Kind::ListBox => {
                let k = self.zeile_unter(wi, i, mx, my);
                if k >= 0 { (k, w.items[k as usize].clone()) } else { (-1, String::new()) }
            }
            Kind::Tree => {
                let k = self.zeile_unter(wi, i, mx, my);
                let t = w.tree.as_ref().unwrap();
                if k >= 0 { (k, t.nodes[k as usize].label.clone()) } else { (-1, String::new()) }
            }
            Kind::Table => {
                let k = self.zeile_unter(wi, i, mx, my);
                let t = w.tbl.as_ref().unwrap();
                if k >= 0 { (k, t.cell(k as usize, 0).map(|z| z.text.clone()).unwrap_or_default()) } else { (-1, String::new()) }
            }
            _ => (-1, w.text.clone()),
        }
    }
    /// Listenzeile, Baumknoten (id) bzw. Datenzeile der Tabelle unter der
    /// Maus, sonst -1. Die Tabelle rechnet wie ihr Hover (`table_geom`) --
    /// vorher trug ein Zug aus ihr keine Zeile (gefunden mit der
    /// Kuehlschrank-App: Bestandszeile auf einen Lagerort ziehen).
    pub(super) fn zeile_unter(&self, wi: usize, i: usize, mx: i32, my: i32) -> i32 {
        let w = &self.windows[wi].widgets[i];
        let (_, ay, _, _) = self.abs_rect(wi, w);
        match w.kind {
            Kind::ListBox => {
                let ih = self.list_zeile_h(w);
                let k = (my - ay + w.value as i32) / ih;
                if k >= 0 && (k as usize) < w.items.len() { k } else { -1 }
            }
            Kind::Tree => {
                let t = w.tree.as_ref().unwrap();
                let vis = Self::tree_visible(t);
                let row = (my - (ay + 1) + t.scroll) / self.sk(TREE_ROW_H);
                if row >= 0 && (row as usize) < vis.len() { vis[row as usize] as i32 } else { -1 }
            }
            Kind::Table => {
                let gm = self.table_geom(wi, i);
                if !Self::in_rect(mx, my, (gm.body_x, gm.body_y, gm.body_w, gm.body_h)) { return -1; }
                let t = w.tbl.as_ref().unwrap();
                let hv = (my - gm.body_y + t.scroll_y) / gm.row_h.max(1);
                if hv >= 0 && hv < gm.n_rows as i32 { t.data_row(hv) } else { -1 }
            }
            _ => -1,
        }
    }
    /// Oberste Ablage unter der Maus (im obersten Fenster dort).
    pub(super) fn ablage_unter(&self, mx: i32, my: i32) -> Option<(usize, usize)> {
        let top = self.topmost_at(mx, my)?;
        let win = &self.windows[top];
        for i in (0..win.widgets.len()).rev() {
            let w = &win.widgets[i];
            if w.ablage && w.enabled && self.widget_shown(top, w) && Self::in_rect(mx, my, self.abs_rect(top, w))
                && self.im_panel_sichtbar(top, w, mx, my) {
                return Some((top, i));
            }
        }
        None
    }
    pub(super) fn ablegen(&mut self, d: DragState, mx: i32, my: i32) {
        let ziel = match self.ablage_unter(mx, my) { Some(z) => z, None => return };
        let index_ziel = self.zeile_unter(ziel.0, ziel.1, mx, my);
        // Innerhalb derselben Liste wird der Eintrag gleich umsortiert -- das
        // ist die eine Bedeutung, die ein Zug dort haben kann. Losgelassen
        // unter dem letzten Eintrag heisst: ans Ende.
        if ziel == d.quelle && d.index >= 0 && self.windows[ziel.0].widgets[ziel.1].kind == Kind::ListBox {
            let n = self.windows[ziel.0].widgets[ziel.1].items.len() as i32;
            let nach = if index_ziel < 0 { n - 1 } else { index_ziel.min(n - 1) };
            if nach >= 0 && nach != d.index {
                let _ = self.list_move(Self::enc_widget(ziel.0, ziel.1), d.index as i64, nach as i64);
            }
        }
        self.windows[ziel.0].widgets[ziel.1].abgelegt = true;
        self.drop = Some(DropInfo { ziel, quelle: d.quelle, index_quelle: d.index, index_ziel, text: d.text });
    }

    /// Was ueber allen Fenstern liegt: ein offenes Kontextmenue und der
    /// Tooltip (GUI_DRAW_TOP).
    pub fn draw_top(&self, g: &mut Graphics) {
        // Popover, Infoleisten, Toasts, Vorschlaege -- ueber den Fenstern und
        // dem, was ein Programm in eine Zeichenflaeche gemalt hat.
        self.ueber_zeichnen(g);
        self.rs_zeichnen(g);
        // Aufgeklapptes Leistenmenue und offene Klappliste: ueber allen
        // Fenstern und dem, was das Programm danach gemalt hat -- vorher
        // deckte sie jedes Fenster zu, das ueber ihrem eigenen lag (im
        // Studio das Entwurfsfenster), und der Klick traf sie trotzdem.
        let mut eigner: Vec<usize> = Vec::new();
        if let Some((mw, _)) = self.open_menu { eigner.push(mw); }
        if let Some((dw, _)) = self.open_dropdown { if !eigner.contains(&dw) { eigner.push(dw); } }
        for wi in eigner {
            if wi < self.windows.len() && self.popups_oben(wi) { self.popups_zeichnen(g, wi); }
        }
        // Kontextmenue ueber ALLEM (nach allen Fenstern).
        // Die Lagen kommen aus `popup_chain` -- derselben Quelle wie der
        // Treffertest, dort weicht ein Popup am Bildschirmrand aus.
        if let Some((wi, _, _, _)) = self.context_open {
            if self.windows.get(wi).map(|w| w.alive).unwrap_or(false) {
                for (pw, pm, px, py) in self.popup_chain(g) { self.draw_items_popup(g, pw, pm, px, py); }
            }
        }
        // Laufender Zug: die Ablage darunter bekommt einen Rahmen, der Text
        // haengt an der Maus -- sonst sieht man nicht, dass man etwas traegt.
        if let Some(d) = self.drag.as_ref().filter(|d| d.aktiv) {
            let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
            if let Some((zw, zi)) = self.ablage_unter(mx, my) {
                let (x, y, w, h) = self.abs_rect(zw, &self.windows[zw].widgets[zi]);
                let acc = self.th("accent");
                g.rect(x - 1, y - 1, x + w, y + h, acc);
                g.rect(x - 2, y - 2, x + w + 1, y + h + 1, acc);
            }
            let text = if d.text.is_empty() { "\u{25AA}".to_string() } else { d.text.clone() };
            let pad = 6;
            let tw = self.ctext_width(g, &text);
            let th = self.ctext_height(g);
            let (bx, by) = (mx + 12, my + 12);
            let rad = self.m("corner_radius").min(6);
            g.round_rect(bx + 2, by + 3, bx + tw + pad * 2 + 1, by + th + pad + 2, rad.max(2), (0x55i64 << 24) as i64, true);
            g.round_rect(bx, by, bx + tw + pad * 2 - 1, by + th + pad - 1, rad, self.th("widget_bg"), true);
            g.round_rect(bx, by, bx + tw + pad * 2 - 1, by + th + pad - 1, rad, self.th("accent"), false);
            self.ctext(g, bx + pad, by + pad / 2, text, self.th("text_fg"));
        }
        // Offene Auswahlliste einer Gitterzelle -- sie reicht gern ueber den
        // Tabellenrand hinaus.
        if let Some((ew, ei)) = self.editing_table {
            if self.windows.get(ew).map(|w| w.alive && w.visible).unwrap_or(false) {
                self.draw_wahl_popup(g, ew, ei);
            }
        }
        // Ein aufgeklappter Monat (Datumsfeld oder Datumsspalte).
        self.draw_datum_popup(g);
        // Ueberlauf-Menue einer Werkzeugleiste -- ueber allen Fenstern.
        if let Some((lw, li)) = self.leiste_popup {
            if self.windows.get(lw).map(|w| w.alive && w.visible).unwrap_or(false) {
                self.draw_leisten_popup(g, lw, li);
            }
        }
        // Tooltip ganz zuletzt, wenn die Maus lange genug ruht.
        self.draw_tooltip(g);
    }

    /// Alles hinter dem modalen Dialog abdunkeln. Das ist nicht Deko: ohne
    /// sichtbares Zeichen klickt man in den Hintergrund und wundert sich,
    /// warum nichts passiert.
    ///
    /// An EINER Stelle, weil `draw` und `draw_window_top` denselben Schleier
    /// brauchen -- zwei Kopien liefen frueher oder spaeter auseinander.
    pub(super) fn schleier(&self, g: &mut Graphics) {
        // 0xAARRGGBB -- Alpha im obersten Byte (0 waere DECKEND).
        let c = (110_i64 << 24) | 0x000000;
        g.box_fill(0, 0, g.screen_width() as i32, g.screen_height() as i32, c);
    }

    /// EIN Fenster noch einmal zeichnen -- ueber allem, was inzwischen im
    /// Bild steht (GUI_DRAW_WINDOW).
    ///
    /// Der Anlass ist eine Luecke, die erst der Sprite-Pilot aufdeckte:
    /// `GUI_DRAW` zeichnet Fenster und Zeichenflaechen in EINEM Durchgang,
    /// und der Inhalt einer Zeichenflaeche entsteht per Bauart DANACH -- das
    /// Programm malt selbst hinein (siehe `Kind::Canvas`). Ein Fenster, das
    /// ueber einer Zeichenflaeche liegt, verschwindet damit hinter deren
    /// Inhalt. Im Piloten war der Kasten "Neues Sprite" auf diese Weise
    /// unsichtbar: er ging auf, sperrte die Eingabe und war nicht zu sehen --
    /// das Programm wirkte eingefroren.
    ///
    /// Ein zweites `GUI_DRAW` hilft nicht: es zeichnet auch den
    /// Fenster-Hintergrund und die Zeichenflaechen neu, also genau das, was
    /// das Programm gerade gemalt hat.
    ///
    /// Ein unsichtbares oder zerstoertes Fenster zeichnet nichts (kein
    /// Fehler) -- so darf der Aufruf unbedingt in der Bildschleife stehen.
    ///
    /// Das Fenster kommt dabei auch in der REIHENFOLGE nach vorn (ohne
    /// Fokus): was obenauf zu sehen ist, muss auch die Klicks bekommen. Vorher
    /// lag es nach einem Klick ins Hauptfenster dahinter -- man sah die
    /// Suchleiste der IDE, aber ihre Knoepfe und Kaestchen taten nichts, und
    /// das Code-Feld darunter bekam den Klick auch nicht (Stresstest
    /// 2026-10-01).
    pub fn draw_window_top(&mut self, g: &mut Graphics, h: i64) -> Result<(), String> {
        let wi = h as usize;
        let w = match self.windows.get(wi) {
            Some(w) if w.alive => w,
            Some(_) => return Ok(()),          // zerstoert -- Handle bleibt gueltig
            None => return Err("GUI_DRAW_WINDOW: ungueltiges GUI_WINDOW-Handle".into()),
        };
        if !w.visible { return Ok(()); }
        if self.modal == Some(wi) { self.schleier(g); }
        if w.pop.is_some() { self.pop_zeichnen(g, wi); } else { self.draw_window_sicht(g, wi); }
        if self.dock_hat_angedockte(wi) { self.dock_oben_nachholen(g, wi); }
        self.bring_to_front(wi);
        Ok(())
    }

    /// Hilfetext (Tooltip) bzw. Fehlermeldung eines Widgets -- die Meldung
    /// gewinnt. None, wenn es nichts zu sagen gibt.
    pub(super) fn tip_text(&self, wi: usize, i: usize) -> Option<String> {
        let w = self.windows.get(wi)?.widgets.get(i)?;
        if !w.fehler.is_empty() { return Some(w.fehler.clone()); }
        if let Some(s) = w.status.as_ref() {
            if s.hover >= 0 {
                if let Some(f) = s.felder.get(s.hover as usize).filter(|f| !f.tip.is_empty()) {
                    return Some(f.tip.clone());
                }
            }
        }
        // Ueber dem "..." einer Pfadleiste: der ganze Pfad, der dort fehlt.
        if let Some(p) = w.pfad.as_ref() {
            if p.hover == -2 { return Some(p.texte.join("  >  ")); }
        }
        if let Some(l) = w.list.as_ref() {
            if l.hover >= 0 {
                if let Some(t) = l.tips.get(l.hover as usize).filter(|t| !t.is_empty()) {
                    return Some(t.clone());
                }
            }
        }
        // Ein gekuerzter Eintrag ("...") zeigt seinen ganzen Text.
        if matches!(w.kind, Kind::ListBox | Kind::Label) && !w.ganzer_text.is_empty() && w.tooltip.is_empty() {
            return Some(w.ganzer_text.clone());
        }
        if let Some(l) = w.leiste.as_ref() {
            if l.hover >= 0 {
                if let Some(e) = l.eintraege.get(l.hover as usize).filter(|e| !e.tip.is_empty()) {
                    return Some(e.tip.clone());
                }
            }
        }
        if !w.tooltip.is_empty() { return Some(w.tooltip.clone()); }
        None
    }

    /// Zeichnet den Hilfetext des aktuell verweilten Widgets (nahe der Maus),
    /// sobald TOOLTIP_DELAY ruhende Frames vergangen sind -- oder den des
    /// Widgets, das per Tastatur den Fokus bekam (unter dem Widget). Die
    /// Maus gewinnt. Mehrzeilig (\n), am Bildschirmrand zurueckgeklemmt.
    pub(super) fn draw_tooltip(&self, g: &mut Graphics) {
        let (mx, my) = (g.mouse_x() as i32, g.mouse_y() as i32);
        let (sw, sh) = (g.screen_width() as i32, g.screen_height() as i32);
        let per_maus = self.hover_w
            .filter(|_| self.frame_count - self.hover_frame >= TOOLTIP_DELAY)
            .and_then(|(wi, i)| self.tip_text(wi, i).map(|t| (t, mx + 14, my + 18)));
        let anker = per_maus.or_else(|| {
            let (wi, i) = self.tip_fokus?;
            if self.frame_count - self.tip_fokus_frame < TOOLTIP_DELAY { return None; }
            if self.focus_widget != Some((wi, i)) { return None; }
            if self.modal.is_some_and(|m| m != wi) { return None; }
            let w = self.windows.get(wi)?.widgets.get(i)?;
            if !self.widget_shown(wi, w) { return None; }
            let (x, y, _, h) = self.abs_rect(wi, w);
            self.tip_text(wi, i).map(|t| (t, x, y + h + 4))
        });
        let (text, ax, ay) = match anker { Some(a) => a, None => return };
        let pad = 6;
        let lh = (self.ctext_height(g) + self.sk(3)).max(self.sk(12));
        let lines: Vec<&str> = text.split('\n').collect();
        let tw = lines.iter().map(|l| self.ctext_width(g, l)).max().unwrap_or(0);
        let bw = tw + pad * 2;
        let bh = lines.len() as i32 * lh + pad * 2 - 3;
        // Position: am Anker, an den Bildschirmrand geklemmt.
        let mut bx = ax;
        let mut by = ay;
        if bx + bw > sw { bx = (sw - bw).max(0); }
        if by + bh > sh { by = (ay - bh - 24).max(0); }
        let rad = self.m("corner_radius").min(6);
        g.round_rect(bx + 2, by + 3, bx + bw - 1 + 2, by + bh - 1 + 3, rad.max(2), (0x55i64 << 24) as i64, true);
        g.round_rect(bx, by, bx + bw - 1, by + bh - 1, rad, self.th("widget_bg"), true);
        g.round_rect(bx, by, bx + bw - 1, by + bh - 1, rad, self.th("accent"), false);
        for (r, line) in lines.iter().enumerate() {
            self.ctext(g, bx + pad, by + pad + r as i32 * lh, line.to_string(), self.th("text_fg"));
        }
    }


    // --- Bedienung ohne Maus, zweiter Teil ------------------------------------
    /// Tab-Reihenfolge eines Fensters: bedienbare, sichtbare, eingeschaltete
    /// Widgets -- die mit `tab_index` > 0 zuerst (aufsteigend), der Rest in
    /// der Reihenfolge des Anlegens. EINE Quelle fuer den Tab-Zyklus.
    pub(super) fn tab_reihenfolge(&self, wi: usize) -> Vec<usize> {
        let win = &self.windows[wi];
        let mut idxs: Vec<usize> = (0..win.widgets.len())
            .filter(|&i| { let w = &win.widgets[i]; w.kind.fokussierbar() && self.widget_shown(wi, w) && w.enabled })
            .collect();
        idxs.sort_by_key(|&i| {
            let t = win.widgets[i].tab_index;
            (if t > 0 { 0 } else { 1 }, if t > 0 { t } else { 0 }, i)
        });
        idxs
    }
    pub fn set_tab_index(&mut self, h: i64, n: i64) -> Result<(), String> {
        if n < 0 { return Err("GUI_SET_TAB_INDEX: der Index ist 0 (Anlege-Reihenfolge) oder groesser".into()); }
        let w = self.wdg_mut(h, "GUI_SET_TAB_INDEX")?;
        w.tab_index = n as i32;
        Ok(())
    }

    /// Das tiefste offene Popup, Leisten- ODER Kontextmenue: dort gelten die
    /// Tasten. Bis 2026-09-23 kannte die Tastatur nur Leistenmenues -- ein
    /// Kontextmenue liess sich nur mit der Maus bedienen.
    pub(super) fn tiefstes_popup(&self) -> Option<(usize, usize)> {
        if let Some((wi, mi, _, _)) = self.context_open {
            return Some((wi, self.sub_chain.last().map(|s| s.0).unwrap_or(mi)));
        }
        self.tiefstes_menue()
    }
    /// Alle Popups schliessen (Leiste, Kontext, Untermenues).
    pub(super) fn menues_zu(&mut self) {
        self.open_menu = None; self.context_open = None; self.sub_chain.clear(); self.menu_cursor = None;
    }
    /// Einen Eintrag per Tastatur ausloesen: Untermenue oeffnen oder feuern.
    pub(super) fn menue_ausloesen(&mut self, g: &Graphics, wi: usize, mi: usize, ii: usize) {
        let sub = self.windows[wi].menus[mi].items.get(ii).map_or(false, |it| it.sub >= 0);
        if sub { self.untermenue_per_taste(g, ii); return; }
        self.menues_zu();
        self.fire_menu_item(wi, mi, ii);
    }
    /// Das tiefste offene Leistenmenue: (Fenster, Menue) -- das letzte
    /// Untermenue der Kette oder das Wurzelmenue.
    pub(super) fn tiefstes_menue(&self) -> Option<(usize, usize)> {
        let (wi, mi) = self.open_menu?;
        Some((wi, self.sub_chain.last().map(|s| s.0).unwrap_or(mi)))
    }
    /// Erster bedienbarer Eintrag eines Menues ab `von` in Richtung `d`
    /// (+1/-1), umlaufend; None, wenn es keinen gibt.
    pub(super) fn menue_schritt(&self, wi: usize, mi: usize, von: Option<usize>, d: i32) -> Option<usize> {
        let items = &self.windows.get(wi)?.menus.get(mi)?.items;
        let n = items.len() as i32;
        if n == 0 { return None; }
        let mut k = match von { Some(v) => v as i32 + d, None => if d > 0 { 0 } else { n - 1 } };
        for _ in 0..n {
            k = k.rem_euclid(n);
            let it = &items[k as usize];
            if !it.separator && it.enabled { return Some(k as usize); }
            k += d;
        }
        None
    }
    /// Menueleiste per Tastatur oeffnen: das erste Leistenmenue des Fensters
    /// mit Fokus (oder des obersten Fensters mit Leiste).
    pub(super) fn menue_oeffnen_per_taste(&mut self) {
        let kandidaten: Vec<usize> = self.focus_window.into_iter()
            .chain(self.z_order.iter().rev().copied()).collect();
        for wi in kandidaten {
            let win = match self.windows.get(wi) { Some(w) if w.alive && w.visible => w, _ => continue };
            if self.modal.map_or(false, |m| m != wi) { continue; }
            if let Some(mi) = win.menus.iter().position(|m| m.in_bar) {
                self.open_menu = Some((wi, mi));
                self.context_open = None;
                self.sub_chain.clear();
                self.open_dropdown = None;
                self.menu_cursor = self.menue_schritt(wi, mi, None, 1);
                return;
            }
        }
    }
    /// Zum naechsten/vorigen Leistenmenue wechseln (Kette schliessen).
    pub(super) fn menue_seitwaerts(&mut self, d: i32) {
        let (wi, mi) = match self.open_menu { Some(x) => x, None => return };
        let leiste: Vec<usize> = self.windows[wi].menus.iter().enumerate()
            .filter(|(_, m)| m.in_bar).map(|(k, _)| k).collect();
        if leiste.is_empty() { return; }
        let pos = leiste.iter().position(|&k| k == mi).unwrap_or(0) as i32;
        let neu = leiste[(pos + d).rem_euclid(leiste.len() as i32) as usize];
        self.open_menu = Some((wi, neu));
        self.sub_chain.clear();
        self.menu_cursor = self.menue_schritt(wi, neu, None, 1);
    }
    /// Untermenue des Eintrags `ii` per Tastatur oeffnen -- dieselbe Lage
    /// wie beim Ueberfahren mit der Maus (`untermenues_folgen`).
    pub(super) fn untermenue_per_taste(&mut self, g: &Graphics, ii: usize) {
        let chain = self.popup_chain(g);
        let &(wi, mi, px, py) = match chain.last() { Some(x) => x, None => return };
        let sub = match self.windows[wi].menus[mi].items.get(ii) { Some(it) if it.sub >= 0 => it.sub as usize, _ => return };
        let (wmax, _) = self.popup_size(g, wi, mi);
        let iy = py + 2 + ii as i32 * self.sk(MENU_ITEM_H);
        self.sub_chain.push((sub, px + wmax - 4, iy - 2));
        self.menu_cursor = self.menue_schritt(wi, sub, None, 1);
    }
    /// Tasten fuers Leistenmenue. Liefert true, wenn die Tasten dem Menue
    /// gehoeren (es ist offen oder wurde eben geoeffnet) -- dann bekommen
    /// die Widgets sie nicht.
    pub(super) fn menue_tasten(&mut self, g: &mut Graphics) -> bool {
        // Alt allein: gedrueckt merken, jede andere Taste loescht, das
        // Loslassen oeffnet. F10 oeffnet sofort.
        const K_LALT: i64 = 1073742050;
        const K_RALT: i64 = 1073742054;
        const K_F10: i64 = 1073741891;
        let alt = g.key_alt();
        if alt && !self.alt_allein && (g.key_pressed(K_LALT) || g.key_pressed(K_RALT)) { self.alt_allein = true; }
        if alt && self.alt_allein && g.key_any_pressed_except_alt() { self.alt_allein = false; }
        let alt_los = !alt && self.alt_allein && (g.key_released_edge(K_LALT) || g.key_released_edge(K_RALT));
        if !alt { self.alt_allein = false; }
        // F10 ist die Menue-Taste nur, wenn sie nichts anderes ist: mit Strg
        // gehoert sie einem Kuerzel ("bis zur Marke"), und hat ein Menue-
        // Kuerzel sie genommen (die IDE: F10 = Schritt drueber), klappte sonst
        // nebenbei die Menueleiste auf.
        let f10 = g.key_pressed(K_F10) && !g.key_ctrl() && !g.key_shift() && !self.kuerzel_gefeuert;
        let oeffnen = f10 || alt_los;
        let kontext = self.context_open.is_some();
        if self.open_menu.is_none() && !kontext {
            if oeffnen {
                self.menue_oeffnen_per_taste();
                return self.open_menu.is_some();
            }
            return false;
        }
        if oeffnen {   // F10/Alt bei offenem Menue: schliessen
            self.menues_zu();
            return true;
        }
        let (wi, mi) = match self.tiefstes_popup() { Some(x) => x, None => return false };
        let (auf, ab) = (g.key_pressed(KEY_UP), g.key_pressed(KEY_DOWN));
        let (links, rechts) = (g.key_pressed(KEY_LEFT), g.key_pressed(KEY_RIGHT));
        let waehlen = g.key_pressed(KEY_ENTER) || g.key_pressed(KEY_SPACE);
        if ab { self.menu_cursor = self.menue_schritt(wi, mi, self.menu_cursor, 1); }
        if auf { self.menu_cursor = self.menue_schritt(wi, mi, self.menu_cursor, -1); }
        // Pos1/Ende: erster und letzter bedienbarer Eintrag -- wie in der Liste.
        if g.key_pressed(KEY_HOME) { self.menu_cursor = self.menue_schritt(wi, mi, None, 1); }
        if g.key_pressed(KEY_END) { self.menu_cursor = self.menue_schritt(wi, mi, None, -1); }
        let hat_sub = |gui: &Gui, ii: usize| gui.windows[wi].menus[mi].items.get(ii).map_or(false, |it| it.sub >= 0);
        if rechts {
            match self.menu_cursor {
                Some(ii) if hat_sub(self, ii) => self.untermenue_per_taste(g, ii),
                // Seitwaerts zum naechsten Leistenmenue -- ein Kontextmenue hat keine Nachbarn.
                _ => if !kontext { self.menue_seitwaerts(1) },
            }
        }
        if links {
            if let Some((sub, _, _)) = self.sub_chain.pop() {
                // Zurueck auf den Eintrag, dessen Untermenue das war.
                let (_, eltern) = self.tiefstes_popup().unwrap_or((wi, mi));
                self.menu_cursor = self.windows[wi].menus[eltern].items.iter().position(|it| it.sub == sub as i32);
            } else if !kontext {
                self.menue_seitwaerts(-1);
            }
        }
        if waehlen {
            match self.menu_cursor {
                Some(ii) => self.menue_ausloesen(g, wi, mi, ii),
                None => { self.menu_cursor = self.menue_schritt(wi, mi, None, 1); }
            }
        }
        if g.key_pressed(KEY_ESC) {
            if let Some((sub, _, _)) = self.sub_chain.pop() {
                let (_, eltern) = self.tiefstes_popup().unwrap_or((wi, mi));
                self.menu_cursor = self.windows[wi].menus[eltern].items.iter().position(|it| it.sub == sub as i32);
            } else {
                self.menues_zu();
            }
        }
        // Tippen springt: der Anfangsbuchstabe markiert den naechsten Eintrag,
        // der so anfaengt; ist er EINDEUTIG, wird der Eintrag gleich
        // ausgeloest -- so kennt man es aus jedem Menue ohne Unterstreichung.
        if !g.key_ctrl() && !g.key_alt() && (self.open_menu.is_some() || self.context_open.is_some()) {
            let mut s = g.pop_text_input();
            s.retain(|c| !c.is_control() && c != ' ');
            if s.is_empty() {
                for c in (b'a'..=b'z').chain(b'0'..=b'9') {
                    if g.key_pressed(c as i64) { s.push(c as char); }
                }
            }
            if let Some(c) = s.to_lowercase().chars().next() {
                let (wi, mi) = self.tiefstes_popup().unwrap_or((wi, mi));
                let treffer: Vec<usize> = self.windows[wi].menus[mi].items.iter().enumerate()
                    .filter(|(_, it)| !it.separator && it.enabled && it.label.to_lowercase().starts_with(c))
                    .map(|(k, _)| k).collect();
                if treffer.len() == 1 {
                    self.menu_cursor = Some(treffer[0]);
                    self.menue_ausloesen(g, wi, mi, treffer[0]);
                } else if !treffer.is_empty() {
                    let ab_hier = self.menu_cursor.map(|k| k as i32).unwrap_or(-1);
                    let naechster = treffer.iter().copied().find(|&k| k as i32 > ab_hier).unwrap_or(treffer[0]);
                    self.menu_cursor = Some(naechster);
                }
            }
        }
        true
    }

    /// Der angezeigte Text eines Textfelds/Textbereichs: der Inhalt (im
    /// Passwortmodus Punkte) mit der IME-Vorschau an der Schreibmarke, und
    /// die Lage der Schreibmarke IN dieser Anzeige (hinter der Marke der
    /// Vorschau). EINE Quelle fuer Zeichnen, Unterstreichen und die Meldung
    /// an die IME.
    pub(super) fn anzeige_mit_vorschau(&self, wdg: &Widget) -> (Vec<char>, i32) {
        let mut chars: Vec<char> = if wdg.passwort {
            std::iter::repeat('\u{2022}').take(wdg.text.chars().count()).collect()
        } else { wdg.text.chars().collect() };
        let caret = wdg.caret.clamp(0, chars.len() as i32);
        if wdg.vorschau.is_empty() { return (chars, caret); }
        let v: Vec<char> = wdg.vorschau.chars().collect();
        let marke = wdg.vorschau_marke.clamp(0, v.len() as i32);
        for (k, ch) in v.iter().enumerate() { chars.insert(caret as usize + k, *ch); }
        (chars, caret + marke)
    }

    /// Eingabemethoden: hat ein Textfeld/Textbereich den Fokus, der Eingaben
    /// annimmt? Nur dann uebernimmt dhrt die Umwandlung (ime.rs).
    pub fn ime_feld_aktiv(&self) -> bool {
        let (wi, i) = match self.focus_widget { Some(f) => f, None => return false };
        if self.focus_window != Some(wi) { return false; }
        let win = match self.windows.get(wi) { Some(w) if w.alive && w.visible => w, _ => return false };
        match win.widgets.get(i) {
            Some(w) => matches!(w.kind, Kind::TextInput | Kind::TextArea) && self.widget_shown(wi, w) && w.enabled && !w.nur_lesen,
            None => false,
        }
    }

    /// Die laufende Vorschau der IME ans Feld mit Fokus haengen (None =
    /// keine Umwandlung). Alle anderen Felder verlieren ihre.
    pub fn ime_vorschau(&mut self, vorschau: Option<(String, i32)>) {
        let ziel = if self.ime_feld_aktiv() { self.focus_widget } else { None };
        for (wi, win) in self.windows.iter_mut().enumerate() {
            for (i, w) in win.widgets.iter_mut().enumerate() {
                match (&vorschau, ziel) {
                    (Some((t, m)), Some((zw, zi))) if zw == wi && zi == i => { w.vorschau = t.clone(); w.vorschau_marke = *m; }
                    _ => { w.vorschau.clear(); w.vorschau_marke = 0; }
                }
            }
        }
    }

    /// Ein Ergebnis der IME ins Feld mit Fokus einfuegen -- durch dieselben
    /// Grenzen wie Tippen (Hoechstlaenge, Zahlenfilter), mit Rueckgaengig
    /// und `on_change`; eine Markierung wird ersetzt.
    pub fn ime_ergebnis(&mut self, text: &str, jetzt: f64) {
        if text.is_empty() || !self.ime_feld_aktiv() { return; }
        let (wi, i) = match self.focus_widget { Some(f) => f, None => return };
        let w = &mut self.windows[wi].widgets[i];
        let before = w.text.clone();
        let mut chars: Vec<char> = before.chars().collect();
        let n = chars.len() as i32;
        let (lo, hi) = (w.caret.min(w.sel_anchor).clamp(0, n), w.caret.max(w.sel_anchor).clamp(0, n));
        if lo != hi { chars.drain(lo as usize..hi as usize); }
        let mut neu: Vec<char> = text.chars().filter(|c| !c.is_control() || *c == '\n').collect();
        if w.kind == Kind::TextInput {
            neu.retain(|c| *c != '\n');
            if w.maxlaenge > 0 {
                let platz = (w.maxlaenge as usize).saturating_sub(chars.len());
                neu.truncate(platz);
            }
        }
        for (k, ch) in neu.iter().enumerate() { chars.insert(lo as usize + k, *ch); }
        let text_neu: String = chars.iter().collect();
        if w.kind == Kind::TextInput && !zahl_erlaubt(&text_neu, w.zahlen) { return; }
        if text_neu == before { return; }
        let caret0 = w.caret;
        Self::undo_merken(w, &before, caret0, jetzt);
        w.text = text_neu;
        w.spans.clear();
        w.wellen.clear();
        w.caret = lo + neu.len() as i32;
        w.sel_anchor = w.caret;
        w.vorschau.clear(); w.vorschau_marke = 0;
        let f = w.on_change.clone();
        if let Some(f) = f { self.pending.push(f); }
    }

    /// Lage der Schreibmarke des Textfelds/Textbereichs mit Fokus in
    /// Bildschirm-Pixeln: (x, y_oben, hoehe). None ohne Text-Fokus. Dieselbe
    /// Rechnung wie das Zeichnen der Marke in `draw_widget` -- fuer das
    /// IME-Umwandlungsfenster (docs/entwurf-eingabemethoden.md, Weg B).
    pub fn schreibmarke(&self, g: &Graphics) -> Option<(i32, i32, i32)> {
        let (wi, i) = self.focus_widget?;
        if self.focus_window != Some(wi) { return None; }
        self.marke_von(g, wi, i)
    }

    /// GUI_TEXTAREA_CARET_XY: wo die Schreibmarke eines Textbereichs auf dem
    /// Bildschirm steht -- (x, y, hoehe) in logischen Punkten wie
    /// GUI_WINDOW_SET_BOUNDS, auch ohne Fokus. Fuer eine Vorschlagsliste an
    /// der Marke statt an einer festen Stelle. (-1, -1, 0), wenn das Feld
    /// nicht zu sehen ist.
    pub fn textarea_caret_xy(&self, g: &Graphics, h: i64) -> Result<(i64, i64, i64), String> {
        self.ta_wdg(h, "GUI_TEXTAREA_CARET_XY")?;
        let (wi, i) = Self::dec_widget(h);
        Ok(match self.marke_von(g, wi, i) {
            Some((x, y, hh)) => (self.unsk(x) as i64, self.unsk(y) as i64, self.unsk(hh) as i64),
            None => (-1, -1, 0),
        })
    }

    /// Die Bildschirmlage der Marke in Widget (wi, i): (x, y, hoehe).
    pub(super) fn marke_von(&self, g: &Graphics, wi: usize, i: usize) -> Option<(i32, i32, i32)> {
        let win = self.windows.get(wi)?;
        if !win.alive || !win.visible { return None; }
        let wdg = win.widgets.get(i)?;
        if !self.widget_shown(wi, wdg) { return None; }
        let (ax, ay, w, h) = self.abs_rect(wi, wdg);
        match wdg.kind {
            Kind::TextInput => {
                let (anz, caret_anz) = self.anzeige_mit_vorschau(wdg);
                let anzeige: String = anz.iter().collect();
                let (sl, sr) = self.ti_rand(wdg);
                let inner = (w - 10 - sl - sr).max(1);
                let tw = self.wtext_width(g, wdg, &anzeige);
                let (tx, scroll) = if wdg.align >= 1 && tw <= inner {
                    (if wdg.align == 1 { ax + 5 + sl + (inner - tw) / 2 } else { ax + 5 + sl + inner - tw }, 0)
                } else { (ax + 5 + sl, wdg.scroll) };
                let cx = tx + self.zeilen_x(g, wdg, &anz, caret_anz.max(0) as usize) - scroll;
                Some((cx, ay + 3, (h - 7).max(1)))
            }
            Kind::TextArea => {
                let pad = 5;
                let lh = self.ta_line_h(g, wdg);
                let (chars, caret_anz) = self.anzeige_mit_vorschau(wdg);
                let starts = Self::line_starts(&chars);
                let rows = self.ta_rows(g, wdg, &chars, &starts, self.ta_breite(g, wdg, starts.len()));
                let crow = Self::ta_row_of(&rows, caret_anz.max(0) as usize);
                let (_, lstart, lend) = *rows.get(crow)?;
                let cend = (lstart + (caret_anz - lstart as i32).max(0) as usize).min(chars.len());
                let cx = ax + pad + self.ta_gutter(g, wdg, starts.len()) - wdg.scroll_x
                         + self.zeilen_x(g, wdg, &chars[lstart..lend], cend - lstart);
                let cy = ay + pad + (crow as i32 - wdg.scroll) * lh;
                Some((cx, cy, lh))
            }
            _ => None,
        }
    }
}
