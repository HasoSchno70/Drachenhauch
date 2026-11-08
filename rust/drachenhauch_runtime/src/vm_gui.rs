//! Befehle des gui-Moduls (Retained-Mode) und der Immediate-Mode-Tabelle `ui`.
//!
//! Teil von `vm.rs`: dort stehen die Typen; hier nur Methoden,
//! herausgeloest ohne Aenderung am Inhalt.

use super::*;

impl<'p> Vm<'p> {
    #[cfg(not(feature = "graphics"))]
    pub(super) fn try_graphics(&mut self, _name: &str, _args: &[Value]) -> R<Option<Value>> {
        Ok(None)
    }

    #[cfg(not(feature = "graphics"))]
    pub(super) fn try_gui(&mut self, _name: &str, _a: &[Value]) -> R<Option<Value>> { Ok(None) }

    /// Retained-Mode-GUI (Modul `gui`). Liefert Ok(None) wenn `name` kein
    /// gui-Builtin ist. GUI_UPDATE/GUI_DRAW brauchen die Grafik (self.gfx),
    /// alles andere nur den GUI-State (self.gui).
    ///
    /// Hat die gui eine eigene Schrift (GUI_FONT), gilt sie fuer die Dauer
    /// des Befehls als aktive -- gemessen und gezeichnet wird mit ihr, egal
    /// was das Programm davor mit SETFONT eingestellt hat. Danach wird
    /// zurueckgetauscht, auch nach einem Fehler.
    #[cfg(feature = "graphics")]
    pub(super) fn try_gui(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("gui_") { return Ok(None); }
        if name == "gui_font" || name == "gui_font_get" { return self.gui_font_befehl(name, a).map(Some); }
        // Ohne eigene Schrift der gui und ohne SETFONT des Programms nimmt die
        // gui die eingebaute Sans in der Textgroesse des Programms -- die
        // Pixelschrift von raylib sieht nach 1990 aus, nicht nach einer
        // Anwendung. GUI_FONT(-1) holt sie ausdruecklich zurueck.
        self.gui_tausch = match (self.gui.schrift, self.gfx.as_mut()) {
            (Some((f, n)), Some(g)) => Some(g.schrift_tauschen(f, n)),
            _ => self.sans_eintauschen(),
        };
        let r = self.try_gui_innen(name, a);
        self.gui_tausch_zurueck();
        r
    }

    /// Die eingebaute Sans fuer die Dauer eines Befehls eintauschen, wenn das
    /// Programm bei der Pixelschrift ist (kein SETFONT) und nicht per
    /// GUI_FONT(-1) die Pixelschrift will. Liefert, was zurueckzutauschen ist.
    /// Gilt fuer gui, `ui` und Diagramme -- die Oberflaeche einer Anwendung.
    #[cfg(feature = "graphics")]
    pub(super) fn sans_eintauschen(&mut self) -> Option<(i64, i32)> {
        if self.gui_pixel { return None; }
        let sans = &mut self.gui_sans;
        let g = self.gfx.as_mut()?;
        if g.active_font() != -1 { return None; }
        let h = *sans.get_or_insert_with(|| g.load_font("sans", 40).unwrap_or(-2));
        if h < 0 { return None; }
        let n = g.text_height();
        Some(g.schrift_tauschen(h, n))
    }

    /// Grafikbefehle; `UI_*` mit der Schrift der Oberflaeche (CHART_DRAW
    /// regelt es selbst, weil ein Diagramm eine eigene Schrift haben kann).
    #[cfg(feature = "graphics")]
    pub(super) fn try_graphics_app(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        if !name.starts_with("ui_") { return self.try_graphics(name, a); }
        let tausch = self.sans_eintauschen();
        let r = self.try_graphics(name, a);
        if let (Some((f, n)), Some(g)) = (tausch, self.gfx.as_mut()) { g.schrift_tauschen(f, n); }
        r
    }
    #[cfg(not(feature = "graphics"))]
    pub(super) fn try_graphics_app(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> { self.try_graphics(name, a) }

    #[cfg(feature = "graphics")]
    pub(super) fn gui_tausch_zurueck(&mut self) {
        if let (Some((f, n)), Some(g)) = (self.gui_tausch.take(), self.gfx.as_mut()) { g.schrift_tauschen(f, n); }
    }

    /// GUI_FONT(font [, groesse]): die eigene Schrift der gui; -1 = die
    /// aktive des Programms, auch die Pixelschrift (ohne GUI_FONT nimmt die
    /// gui sonst die eingebaute Sans). GUI_FONT_GET() -> TUPLE (font,
    /// groesse), (-1, 0) ohne eigene.
    #[cfg(feature = "graphics")]
    pub(super) fn gui_font_befehl(&mut self, name: &str, a: &[Value]) -> R<Value> {
        if name == "gui_font_get" {
            let (f, n) = self.gui.schrift.unwrap_or((-1, 0));
            return Ok(Value::Tuple(std::rc::Rc::new(vec![Value::Int(f), Value::Int(n as i64)])));
        }
        let f = match a.first() { Some(Value::Int(n)) => *n,
            _ => return Err("GUI_FONT: erwartet ein Font-Handle (LOADFONT) oder -1 (Arg 1)".into()) };
        if f < 0 { self.gui.schrift = None; self.gui_pixel = true; return Ok(Value::Nil); }
        let g = self.gfx.as_ref().ok_or("GUI_FONT: vor SCREEN aufgerufen")?;
        if !g.font_gueltig(f) { return Err(format!("GUI_FONT: {} ist kein geladener Font", f)); }
        let n = match a.get(1) {
            None => g.text_height(),
            Some(Value::Int(n)) if *n >= 4 && *n <= 400 => *n as i32,
            Some(Value::Float(x)) if *x >= 4.0 && *x <= 400.0 => *x as i32,
            _ => return Err("GUI_FONT: erwartet eine Groesse 4..400 (Arg 2)".into()),
        };
        self.gui.schrift = Some((f, n));
        self.gui_pixel = false;
        Ok(Value::Nil)
    }

    #[cfg(feature = "graphics")]
    pub(super) fn try_gui_innen(&mut self, name: &str, a: &[Value]) -> R<Option<Value>> {
        fn gi(a: &[Value], i: usize, f: &str) -> R<i64> {
            match a.get(i) { Some(Value::Int(n)) => Ok(*n),
                Some(Value::Float(x)) => crate::builtins::glatt_int(*x).ok_or_else(|| format!("{}: erwartet INTEGER (Arg {}), erhalten {}{}",
                    f, i + 1, crate::value::fmt_float(*x), crate::builtins::KEIN_GANZ_HINWEIS)),
                Some(Value::Str(t)) => Err(format!("{}: erwartet INTEGER (Arg {}){}", f, i + 1,
                    crate::umstieg::text_statt_zahl(f, t))),
                _ => Err(format!("{}: erwartet INTEGER (Arg {})", f, i + 1)) }
        }
        fn gs(a: &[Value], i: usize, f: &str) -> R<String> {
            match a.get(i) { Some(Value::Str(s)) => Ok(s.to_string()),
                _ => Err(format!("{}: erwartet STRING (Arg {})", f, i + 1)) }
        }
        /// Ein Sinnbild: der Name eines eingebauten (STRING) oder ein Bild
        /// (Textur-Handle, -1 = keins).
        fn gsym(a: &[Value], i: usize, f: &str) -> R<(Option<String>, i64)> {
            match a.get(i) {
                Some(Value::Str(s)) => Ok((Some(s.to_string()), -1)),
                Some(Value::Int(n)) => Ok((None, *n)),
                _ => Err(format!("{}: erwartet den Namen eines Sinnbilds (STRING) oder ein Bild (Arg {})", f, i + 1)),
            }
        }
        fn gbool(a: &[Value], i: usize, f: &str) -> R<bool> {
            match a.get(i) { Some(Value::Bool(b)) => Ok(*b),
                _ => Err(format!("{}: erwartet BOOLEAN (Arg {})", f, i + 1)) }
        }
        fn gnum(a: &[Value], i: usize, f: &str) -> R<f64> {
            match a.get(i) { Some(Value::Int(n)) => Ok(*n as f64), Some(Value::Float(x)) => Ok(*x),
                _ => Err(format!("{}: erwartet Zahl (Arg {})", f, i + 1)) }
        }
        // Ein ARRAY OF INTEGER als Vec -- Marken, Faltbereiche, Spans.
        fn ganze(v: &Value, fn_: &str) -> R<Vec<i64>> {
            match v {
                Value::Array(a) => {
                    let a = a.borrow();
                    let mut o = Vec::with_capacity(a.cells.len());
                    for x in a.cells.iter() {
                        match x {
                            Value::Int(i) => o.push(i),
                            Value::Float(f) => o.push(f as i64),
                            _ => return Err(format!("{}: ARRAY OF INTEGER noetig", fn_)),
                        }
                    }
                    Ok(o)
                }
                _ => Err(format!("{}: ARRAY OF INTEGER noetig", fn_)),
            }
        }
        // FUNCREF-Arg -> Option<Name>; NIL entfernt den Callback.
        fn gfunc(a: &[Value], i: usize, f: &str) -> R<Option<crate::value::Rueckruf>> {
            match a.get(i) {
                Some(Value::Nil) | None => Ok(None),
                Some(v) => match crate::value::Rueckruf::aus_wert(v) {
                    Some(cb) => Ok(Some(cb)),
                    None => Err(format!("{}: handler muss FUNCREF sein", f)),
                },
            }
        }
        // 1D ARRAY OF STRING -> Vec<String>.
        fn gstrs(a: &[Value], i: usize, f: &str) -> R<Vec<String>> {
            match a.get(i) {
                Some(Value::Array(arr)) => {
                    let arr = arr.borrow();
                    // Ein LEERES Array passt immer: es kann keinen falschen
                    // Wert enthalten. Sonst braeuchte ein Widget, das seine
                    // Daten erst spaeter bekommt, einen Platzhalter-Eintrag.
                    if arr.cells.len() == 0 && arr.dims.len() <= 1 { return Ok(Vec::new()); }
                    if arr.element_type != "string" || arr.dims.len() != 1 {
                        return Err(format!("{}: erwartet 1D ARRAY OF STRING", f));
                    }
                    Ok(arr.cells.iter().map(|v| match v { Value::Str(s) => s.to_string(), o => o.fmt() }).collect())
                }
                _ => Err(format!("{}: erwartet ARRAY OF STRING", f)),
            }
        }
        // 2D ARRAY OF STRING -> Vec<Vec<String>>.
        fn gstrs2(a: &[Value], i: usize, f: &str) -> R<Vec<Vec<String>>> {
            match a.get(i) {
                Some(Value::Array(arr)) => {
                    let arr = arr.borrow();
                    if arr.cells.len() == 0 { return Ok(Vec::new()); }
                    if arr.element_type != "string" || arr.dims.len() != 2 {
                        return Err(format!("{}: erwartet 2D ARRAY OF STRING", f));
                    }
                    let (rows, cols) = (arr.dims[0] as usize, arr.dims[1] as usize);
                    let mut out = Vec::with_capacity(rows);
                    for r in 0..rows {
                        let mut row = Vec::with_capacity(cols);
                        for c in 0..cols {
                            row.push(match &arr.cells.get(r * cols + c) { Value::Str(s) => s.to_string(), o => o.fmt() });
                        }
                        out.push(row);
                    }
                    Ok(out)
                }
                _ => Err(format!("{}: erwartet 2D ARRAY OF STRING", f)),
            }
        }
        // 1D ARRAY OF INTEGER -> Vec<i32> (oder None bei NIL).
        fn gints_opt(a: &[Value], i: usize, f: &str) -> R<Option<Vec<i32>>> {
            match a.get(i) {
                Some(Value::Nil) | None => Ok(None),
                Some(Value::Array(arr)) => {
                    let arr = arr.borrow();
                    if arr.element_type != "integer" || arr.dims.len() != 1 {
                        return Err(format!("{}: erwartet 1D ARRAY OF INTEGER", f));
                    }
                    Ok(Some(arr.cells.iter().map(|v| match v { Value::Int(n) => n as i32, _ => 0 }).collect()))
                }
                _ => Err(format!("{}: erwartet ARRAY OF INTEGER oder NIL", f)),
            }
        }
        let r = match name {
            "gui_window" => Value::Int(self.gui.new_window(
                gs(a,0,"GUI_WINDOW")?, gi(a,1,"GUI_WINDOW")? as i32, gi(a,2,"GUI_WINDOW")? as i32,
                gi(a,3,"GUI_WINDOW")? as i32, gi(a,4,"GUI_WINDOW")? as i32)),
            "gui_window_movable" => { self.gui.window_movable(gi(a,0,"GUI_WINDOW_MOVABLE")?, gbool(a,1,"GUI_WINDOW_MOVABLE")?)?; Value::Nil }
            "gui_window_title" => { self.gui.window_title(gi(a,0,"GUI_WINDOW_TITLE")?, &gs(a,1,"GUI_WINDOW_TITLE")?)?; Value::Nil }
            "gui_window_set_name" => { self.gui.window_set_name(gi(a,0,"GUI_WINDOW_SET_NAME")?, &gs(a,1,"GUI_WINDOW_SET_NAME")?)?; Value::Nil }
            "gui_window_get_name$" | "gui_window_get_name" => Value::str_rc(&self.gui.window_get_name(gi(a,0,"GUI_WINDOW_GET_NAME$")?)?),
            // Lichtstreif ueber ein Fenster (docs/module-gui.md, Abschnitt Glanz).
            "gui_window_glow" => {
                let n = "GUI_WINDOW_GLOW";
                if a.is_empty() || a.len() > 3 { return Err(format!("{}: erwartet (win [, farbe [, dauer_ms]])", n)); }
                let farbe = if a.len() > 1 { Some(gi(a, 1, n)?) } else { None };
                let dauer = if a.len() > 2 { Some(gnum(a, 2, n)?) } else { None };
                self.gui.window_glow(gi(a, 0, n)?, farbe, dauer)?;
                Value::Nil
            }
            "gui_window_glow_set" => {
                let n = "GUI_WINDOW_GLOW_SET";
                if a.len() != 3 { return Err(format!("{}: erwartet (win, schluessel$, wert)", n)); }
                let (zahl, text) = match &a[2] {
                    Value::Int(i) => (Some(*i as f64), i.to_string()),
                    Value::Float(f) => (Some(*f), f.to_string()),
                    Value::Bool(b) => (Some(if *b { 1.0 } else { 0.0 }), b.to_string()),
                    v => (None, v.fmt()),
                };
                self.gui.window_glow_set(gi(a, 0, n)?, &gs(a, 1, n)?, zahl, &text)?;
                Value::Nil
            }
            "gui_window_glow_stop" => { self.gui.window_glow_stop(gi(a, 0, "GUI_WINDOW_GLOW_STOP")?)?; Value::Nil }
            "gui_window_glowing" => Value::Bool(self.gui.window_glowing(gi(a, 0, "GUI_WINDOW_GLOWING")?)?),
            "gui_window_closable" => { self.gui.window_closable(gi(a,0,"GUI_WINDOW_CLOSABLE")?, gbool(a,1,"GUI_WINDOW_CLOSABLE")?)?; Value::Nil }
            "gui_window_front" => { self.gui.window_front(gi(a,0,"GUI_WINDOW_FRONT")?)?; Value::Nil }
            "gui_window_visible" => { self.gui.window_visible(gi(a,0,"GUI_WINDOW_VISIBLE")?, gbool(a,1,"GUI_WINDOW_VISIBLE")?)?; Value::Nil }
            "gui_window_shown" => Value::Bool(self.gui.window_shown(gi(a,0,"GUI_WINDOW_SHOWN")?)?),
            "gui_window_resizable" => { self.gui.window_resizable(gi(a,0,"GUI_WINDOW_RESIZABLE")?, gbool(a,1,"GUI_WINDOW_RESIZABLE")?)?; Value::Nil }
            "gui_window_scrollable" => { self.gui.window_scrollable(gi(a,0,"GUI_WINDOW_SCROLLABLE")?, gbool(a,1,"GUI_WINDOW_SCROLLABLE")?)?; Value::Nil }
            "gui_tabs" => { self.gui.set_tabs(gi(a,0,"GUI_TABS")?, gstrs(a,1,"GUI_TABS")?)?; Value::Nil }
            "gui_set_tab" => { self.gui.set_widget_tab(gi(a,0,"GUI_SET_TAB")?, gi(a,1,"GUI_SET_TAB")? as i32)?; Value::Nil }
            "gui_active_tab" => Value::Int(self.gui.active_tab(gi(a,0,"GUI_ACTIVE_TAB")?)?),
            "gui_tabs_closable" => { let n_ = "GUI_TABS_CLOSABLE";
                self.gui.tabs_closable(gi(a,0,n_)?, gbool(a,1,n_)?)?; Value::Nil }
            "gui_tab_closed" => Value::Int(self.gui.tab_closed(gi(a,0,"GUI_TAB_CLOSED")?)?),
            "gui_set_active_tab" => { self.gui.set_active_tab(gi(a,0,"GUI_SET_ACTIVE_TAB")?, gi(a,1,"GUI_SET_ACTIVE_TAB")? as i32)?; Value::Nil }
            "gui_window_chrome" => { self.gui.window_chrome(gi(a,0,"GUI_WINDOW_CHROME")?, gbool(a,1,"GUI_WINDOW_CHROME")?)?; Value::Nil }
            "gui_window_zoom" => { self.gui.window_zoom(gi(a,0,"GUI_WINDOW_ZOOM")?, gnum(a,1,"GUI_WINDOW_ZOOM")?)?; Value::Nil }
            "gui_window_get_zoom" => Value::Float(self.gui.window_get_zoom(gi(a,0,"GUI_WINDOW_GET_ZOOM")?)?),
            "gui_window_design" => { self.gui.window_design(gi(a,0,"GUI_WINDOW_DESIGN")?, gbool(a,1,"GUI_WINDOW_DESIGN")?)?; Value::Nil }
            "gui_window_set_min_size" => { self.gui.window_min_size(gi(a,0,"GUI_WINDOW_SET_MIN_SIZE")?, gi(a,1,"GUI_WINDOW_SET_MIN_SIZE")? as i32, gi(a,2,"GUI_WINDOW_SET_MIN_SIZE")? as i32)?; Value::Nil }
            "gui_window_set_max_size" => { self.gui.window_max_size(gi(a,0,"GUI_WINDOW_SET_MAX_SIZE")?, gi(a,1,"GUI_WINDOW_SET_MAX_SIZE")? as i32, gi(a,2,"GUI_WINDOW_SET_MAX_SIZE")? as i32)?; Value::Nil }
            "gui_window_closed" => Value::Bool(self.gui.window_closed(gi(a,0,"GUI_WINDOW_CLOSED")?)?),
            "gui_button" => Value::Int(self.gui.button(gi(a,0,"GUI_BUTTON")?, gs(a,1,"GUI_BUTTON")?,
                gi(a,2,"GUI_BUTTON")? as i32, gi(a,3,"GUI_BUTTON")? as i32,
                gi(a,4,"GUI_BUTTON")? as i32, gi(a,5,"GUI_BUTTON")? as i32)?),
            "gui_label" => {
                let color = if a.len() >= 5 { Some(gi(a,4,"GUI_LABEL")?) } else { None };
                Value::Int(self.gui.label(gi(a,0,"GUI_LABEL")?, gs(a,1,"GUI_LABEL")?,
                    gi(a,2,"GUI_LABEL")? as i32, gi(a,3,"GUI_LABEL")? as i32, color)?)
            }
            "gui_checkbox" => {
                let def = if a.len() >= 5 { gbool(a,4,"GUI_CHECKBOX")? } else { false };
                Value::Int(self.gui.checkbox(gi(a,0,"GUI_CHECKBOX")?, gs(a,1,"GUI_CHECKBOX")?,
                    gi(a,2,"GUI_CHECKBOX")? as i32, gi(a,3,"GUI_CHECKBOX")? as i32, def)?)
            }
            "gui_toggle" => {
                let def = if a.len() >= 5 { gbool(a,4,"GUI_TOGGLE")? } else { false };
                Value::Int(self.gui.toggle(gi(a,0,"GUI_TOGGLE")?, gs(a,1,"GUI_TOGGLE")?,
                    gi(a,2,"GUI_TOGGLE")? as i32, gi(a,3,"GUI_TOGGLE")? as i32, def)?)
            }
            "gui_knob" => {
                let mn = gnum(a,4,"GUI_KNOB")?; let mx = gnum(a,5,"GUI_KNOB")?;
                let def = if a.len() >= 7 { gnum(a,6,"GUI_KNOB")? } else { mn };
                Value::Int(self.gui.knob(gi(a,0,"GUI_KNOB")?, gi(a,1,"GUI_KNOB")? as i32,
                    gi(a,2,"GUI_KNOB")? as i32, gi(a,3,"GUI_KNOB")? as i32, mn, mx, def)?)
            }
            "gui_slider" => {
                let mn = gnum(a,4,"GUI_SLIDER")?; let mx = gnum(a,5,"GUI_SLIDER")?;
                let def = if a.len() >= 7 { gnum(a,6,"GUI_SLIDER")? } else { mn };
                Value::Int(self.gui.slider(gi(a,0,"GUI_SLIDER")?, gi(a,1,"GUI_SLIDER")? as i32,
                    gi(a,2,"GUI_SLIDER")? as i32, gi(a,3,"GUI_SLIDER")? as i32, mn, mx, def)?)
            }
            "gui_spinner" => {
                let mn = gnum(a,4,"GUI_SPINNER")?; let mx = gnum(a,5,"GUI_SPINNER")?;
                let def = if a.len() >= 7 { gnum(a,6,"GUI_SPINNER")? } else { mn };
                let step = if a.len() >= 8 { gnum(a,7,"GUI_SPINNER")? } else { 1.0 };
                Value::Int(self.gui.spinner(gi(a,0,"GUI_SPINNER")?, gi(a,1,"GUI_SPINNER")? as i32,
                    gi(a,2,"GUI_SPINNER")? as i32, gi(a,3,"GUI_SPINNER")? as i32, mn, mx, def, step)?)
            }
            "gui_splitter" => {
                Value::Int(self.gui.splitter(gi(a,0,"GUI_SPLITTER")?, gi(a,1,"GUI_SPLITTER")? as i32,
                    gi(a,2,"GUI_SPLITTER")? as i32, gi(a,3,"GUI_SPLITTER")? as i32, gs(a,4,"GUI_SPLITTER")?,
                    gi(a,5,"GUI_SPLITTER")? as i32, gi(a,6,"GUI_SPLITTER")? as i32)?)
            }
            "gui_panel" => {
                let title = if a.len() >= 6 { gs(a,5,"GUI_PANEL")? } else { String::new() };
                Value::Int(self.gui.panel(gi(a,0,"GUI_PANEL")?, gi(a,1,"GUI_PANEL")? as i32, gi(a,2,"GUI_PANEL")? as i32,
                    gi(a,3,"GUI_PANEL")? as i32, gi(a,4,"GUI_PANEL")? as i32, title)?)
            }
            "gui_separator" => Value::Int(self.gui.separator(gi(a,0,"GUI_SEPARATOR")?,
                gi(a,1,"GUI_SEPARATOR")? as i32, gi(a,2,"GUI_SEPARATOR")? as i32, gi(a,3,"GUI_SEPARATOR")? as i32)?),
            "gui_groupbox" => {
                let title = if a.len() >= 6 { gs(a,5,"GUI_GROUPBOX")? } else { String::new() };
                Value::Int(self.gui.groupbox(gi(a,0,"GUI_GROUPBOX")?, gi(a,1,"GUI_GROUPBOX")? as i32, gi(a,2,"GUI_GROUPBOX")? as i32,
                    gi(a,3,"GUI_GROUPBOX")? as i32, gi(a,4,"GUI_GROUPBOX")? as i32, title)?)
            }
            "gui_textinput" => {
                let ph = if a.len() >= 6 { gs(a,5,"GUI_TEXTINPUT")? } else { String::new() };
                Value::Int(self.gui.textinput(gi(a,0,"GUI_TEXTINPUT")?, gi(a,1,"GUI_TEXTINPUT")? as i32, gi(a,2,"GUI_TEXTINPUT")? as i32,
                    gi(a,3,"GUI_TEXTINPUT")? as i32, gi(a,4,"GUI_TEXTINPUT")? as i32, ph)?)
            }
            "gui_textarea" => {
                let ph = if a.len() >= 6 { gs(a,5,"GUI_TEXTAREA")? } else { String::new() };
                Value::Int(self.gui.textarea(gi(a,0,"GUI_TEXTAREA")?, gi(a,1,"GUI_TEXTAREA")? as i32, gi(a,2,"GUI_TEXTAREA")? as i32,
                    gi(a,3,"GUI_TEXTAREA")? as i32, gi(a,4,"GUI_TEXTAREA")? as i32, ph)?)
            }
            // --- Formular-Widgets (Phase 3) ---
            "gui_radio" => Value::Int(self.gui.radio(gi(a,0,"GUI_RADIO")?, gs(a,1,"GUI_RADIO")?, gs(a,2,"GUI_RADIO")?,
                gi(a,3,"GUI_RADIO")? as i32, gi(a,4,"GUI_RADIO")? as i32)?),
            "gui_radio_selected" => Value::Int(self.gui.radio_selected(gi(a,0,"GUI_RADIO_SELECTED")?)?),
            "gui_progress" => Value::Int(self.gui.progress(gi(a,0,"GUI_PROGRESS")?, gi(a,1,"GUI_PROGRESS")? as i32,
                gi(a,2,"GUI_PROGRESS")? as i32, gi(a,3,"GUI_PROGRESS")? as i32, gi(a,4,"GUI_PROGRESS")? as i32)?),
            "gui_dropdown" => Value::Int(self.gui.dropdown(gi(a,0,"GUI_DROPDOWN")?, gi(a,1,"GUI_DROPDOWN")? as i32,
                gi(a,2,"GUI_DROPDOWN")? as i32, gi(a,3,"GUI_DROPDOWN")? as i32, gi(a,4,"GUI_DROPDOWN")? as i32,
                gstrs(a,5,"GUI_DROPDOWN")?)?),
            "gui_dropdown_selected" => Value::Int(self.gui.dropdown_selected(gi(a,0,"GUI_DROPDOWN_SELECTED")?)?),
            "gui_dropdown_text" => Value::str_rc(&self.gui.dropdown_text(gi(a,0,"GUI_DROPDOWN_TEXT")?)?),
            "gui_dropdown_set_selected" => { self.gui.dropdown_set_selected(gi(a,0,"GUI_DROPDOWN_SET_SELECTED")?, gi(a,1,"GUI_DROPDOWN_SET_SELECTED")?)?; Value::Nil }
            "gui_set_dropdown" => { self.gui.set_dropdown_items(gi(a,0,"GUI_SET_DROPDOWN")?, gstrs(a,1,"GUI_SET_DROPDOWN")?)?; Value::Nil }
            // --- ListBox (teilt die item-Logik mit Dropdown) ---
            "gui_listbox" => Value::Int(self.gui.listbox(gi(a,0,"GUI_LISTBOX")?, gi(a,1,"GUI_LISTBOX")? as i32,
                gi(a,2,"GUI_LISTBOX")? as i32, gi(a,3,"GUI_LISTBOX")? as i32, gi(a,4,"GUI_LISTBOX")? as i32,
                gstrs(a,5,"GUI_LISTBOX")?)?),
            "gui_listbox_selected" => Value::Int(self.gui.dropdown_selected(gi(a,0,"GUI_LISTBOX_SELECTED")?)?),
            "gui_listbox_text" => Value::str_rc(&self.gui.dropdown_text(gi(a,0,"GUI_LISTBOX_TEXT")?)?),
            "gui_listbox_set_selected" => { self.gui.dropdown_set_selected(gi(a,0,"GUI_LISTBOX_SET_SELECTED")?, gi(a,1,"GUI_LISTBOX_SET_SELECTED")?)?; Value::Nil }
            "gui_set_listbox" => { self.gui.set_dropdown_items(gi(a,0,"GUI_SET_LISTBOX")?, gstrs(a,1,"GUI_SET_LISTBOX")?)?; Value::Nil }
            "gui_listbox_add" => {
                let pos = if a.len() > 2 { gi(a,2,"GUI_LISTBOX_ADD")? } else { -1 };
                Value::Int(self.gui.list_add(gi(a,0,"GUI_LISTBOX_ADD")?, gs(a,1,"GUI_LISTBOX_ADD")?.to_string(), pos)?)
            }
            "gui_listbox_remove" => { self.gui.list_remove(gi(a,0,"GUI_LISTBOX_REMOVE")?, gi(a,1,"GUI_LISTBOX_REMOVE")?)?; Value::Nil }
            "gui_listbox_clear" => { self.gui.list_clear(gi(a,0,"GUI_LISTBOX_CLEAR")?)?; Value::Nil }
            "gui_listbox_count" => Value::Int(self.gui.list_count(gi(a,0,"GUI_LISTBOX_COUNT")?)?),
            "gui_listbox_item" => Value::str_rc(&self.gui.list_item(gi(a,0,"GUI_LISTBOX_ITEM")?, gi(a,1,"GUI_LISTBOX_ITEM")?)?),
            "gui_listbox_set_item" => { self.gui.list_set_item(gi(a,0,"GUI_LISTBOX_SET_ITEM")?, gi(a,1,"GUI_LISTBOX_SET_ITEM")?, gs(a,2,"GUI_LISTBOX_SET_ITEM")?.to_string())?; Value::Nil }
            "gui_listbox_move" => { self.gui.list_move(gi(a,0,"GUI_LISTBOX_MOVE")?, gi(a,1,"GUI_LISTBOX_MOVE")?, gi(a,2,"GUI_LISTBOX_MOVE")?)?; Value::Nil }
            "gui_listbox_set" => { self.gui.list_set(gi(a,0,"GUI_LISTBOX_SET")?, &gs(a,1,"GUI_LISTBOX_SET")?, gnum(a,2,"GUI_LISTBOX_SET")?)?; Value::Nil }
            "gui_listbox_icon" => { self.gui.list_icon(gi(a,0,"GUI_LISTBOX_ICON")?, gi(a,1,"GUI_LISTBOX_ICON")?, gi(a,2,"GUI_LISTBOX_ICON")?)?; Value::Nil }
            "gui_listbox_color" => { self.gui.list_color(gi(a,0,"GUI_LISTBOX_COLOR")?, gi(a,1,"GUI_LISTBOX_COLOR")?, gi(a,2,"GUI_LISTBOX_COLOR")?)?; Value::Nil }
            "gui_listbox_checked" => Value::Bool(self.gui.list_checked(gi(a,0,"GUI_LISTBOX_CHECKED")?, gi(a,1,"GUI_LISTBOX_CHECKED")?)?),
            "gui_listbox_set_checked" => { self.gui.list_set_checked(gi(a,0,"GUI_LISTBOX_SET_CHECKED")?, gi(a,1,"GUI_LISTBOX_SET_CHECKED")?, gbool(a,2,"GUI_LISTBOX_SET_CHECKED")?)?; Value::Nil }
            "gui_listbox_is_selected" => Value::Bool(self.gui.list_is_selected(gi(a,0,"GUI_LISTBOX_IS_SELECTED")?, gi(a,1,"GUI_LISTBOX_IS_SELECTED")?)?),
            "gui_listbox_select" => { self.gui.list_select(gi(a,0,"GUI_LISTBOX_SELECT")?, gi(a,1,"GUI_LISTBOX_SELECT")?, gbool(a,2,"GUI_LISTBOX_SELECT")?)?; Value::Nil }
            "gui_listbox_sel_count" => Value::Int(self.gui.list_sel_count(gi(a,0,"GUI_LISTBOX_SEL_COUNT")?)?),
            "gui_listbox_sel_row" => Value::Int(self.gui.list_sel_row(gi(a,0,"GUI_LISTBOX_SEL_ROW")?, gi(a,1,"GUI_LISTBOX_SEL_ROW")?)?),
            "gui_listbox_clear_selection" => { self.gui.list_clear_selection(gi(a,0,"GUI_LISTBOX_CLEAR_SELECTION")?)?; Value::Nil }
            // --- Stufe 27: Listen komfortabel, Knopfarten ---
            "gui_listbox_set_data" => { self.gui.list_set_data(gi(a,0,"GUI_LISTBOX_SET_DATA")?, gi(a,1,"GUI_LISTBOX_SET_DATA")?, gs(a,2,"GUI_LISTBOX_SET_DATA")?)?; Value::Nil }
            "gui_listbox_data$" | "gui_listbox_data" => Value::str_rc(&self.gui.list_data(gi(a,0,"GUI_LISTBOX_DATA")?, gi(a,1,"GUI_LISTBOX_DATA")?)?),
            "gui_listbox_find_data" => Value::Int(self.gui.list_find_data(gi(a,0,"GUI_LISTBOX_FIND_DATA")?, &gs(a,1,"GUI_LISTBOX_FIND_DATA")?)?),
            "gui_listbox_spans" => {
                // GUI_LISTBOX_SPANS(lb, eintrag, starts, laengen, farben)
                fn ganze(v: &Value, fn_: &str) -> R<Vec<i64>> {
                    match v {
                        Value::Array(a) => {
                            let a = a.borrow();
                            let mut o = Vec::with_capacity(a.cells.len());
                            for x in a.cells.iter() {
                                match x {
                                    Value::Int(i) => o.push(i),
                                    Value::Float(f) => o.push(f as i64),
                                    _ => return Err(format!("{}: ARRAY OF INTEGER noetig", fn_)),
                                }
                            }
                            Ok(o)
                        }
                        _ => Err(format!("{}: ARRAY OF INTEGER noetig", fn_)),
                    }
                }
                let n = "GUI_LISTBOX_SPANS";
                if a.len() != 5 { return Err(format!("{}: erwartet (lb, eintrag, starts, laengen, farben)", n)); }
                self.gui.list_spans(gi(a, 0, n)?, gi(a, 1, n)?, ganze(&a[2], n)?, ganze(&a[3], n)?, ganze(&a[4], n)?)?;
                Value::Nil
            }
            "gui_listbox_detail" => { self.gui.list_detail(gi(a,0,"GUI_LISTBOX_DETAIL")?, gi(a,1,"GUI_LISTBOX_DETAIL")?, gs(a,2,"GUI_LISTBOX_DETAIL")?)?; Value::Nil }
            "gui_listbox_item_rect" => {
                let (x, y, w, h) = self.gui.list_item_rect(gi(a,0,"GUI_LISTBOX_ITEM_RECT")?, gi(a,1,"GUI_LISTBOX_ITEM_RECT")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(x as i64), Value::Int(y as i64), Value::Int(w as i64), Value::Int(h as i64)]))
            }
            "gui_listbox_tip" => { self.gui.list_tip(gi(a,0,"GUI_LISTBOX_TIP")?, gi(a,1,"GUI_LISTBOX_TIP")?, gs(a,2,"GUI_LISTBOX_TIP")?)?; Value::Nil }
            "gui_listbox_enable" => { self.gui.list_enable(gi(a,0,"GUI_LISTBOX_ENABLE")?, gi(a,1,"GUI_LISTBOX_ENABLE")?, gbool(a,2,"GUI_LISTBOX_ENABLE")?)?; Value::Nil }
            "gui_listbox_enabled" => Value::Bool(self.gui.list_enabled(gi(a,0,"GUI_LISTBOX_ENABLED")?, gi(a,1,"GUI_LISTBOX_ENABLED")?)?),
            "gui_listbox_header" => { self.gui.list_header(gi(a,0,"GUI_LISTBOX_HEADER")?, gi(a,1,"GUI_LISTBOX_HEADER")?, gbool(a,2,"GUI_LISTBOX_HEADER")?)?; Value::Nil }
            "gui_listbox_filter" => { self.gui.list_filter(gi(a,0,"GUI_LISTBOX_FILTER")?, gs(a,1,"GUI_LISTBOX_FILTER")?)?; Value::Nil }
            "gui_listbox_get_filter$" | "gui_listbox_get_filter" => Value::str_rc(&self.gui.list_get_filter(gi(a,0,"GUI_LISTBOX_GET_FILTER")?)?),
            "gui_listbox_view_count" => Value::Int(self.gui.list_view_count(gi(a,0,"GUI_LISTBOX_VIEW_COUNT")?)?),
            "gui_listbox_view_row" => Value::Int(self.gui.list_view_row(gi(a,0,"GUI_LISTBOX_VIEW_ROW")?, gi(a,1,"GUI_LISTBOX_VIEW_ROW")?)?),
            "gui_listbox_placeholder" => { self.gui.list_placeholder(gi(a,0,"GUI_LISTBOX_PLACEHOLDER")?, gs(a,1,"GUI_LISTBOX_PLACEHOLDER")?)?; Value::Nil }
            "gui_listbox_sort" => {
                let ab = if a.len() > 1 { gbool(a,1,"GUI_LISTBOX_SORT")? } else { false };
                self.gui.list_sort(gi(a,0,"GUI_LISTBOX_SORT")?, ab)?; Value::Nil
            }
            "gui_listbox_find" => {
                let ab = if a.len() > 2 { gi(a,2,"GUI_LISTBOX_FIND")? } else { 0 };
                Value::Int(self.gui.list_find(gi(a,0,"GUI_LISTBOX_FIND")?, &gs(a,1,"GUI_LISTBOX_FIND")?, ab)?)
            }
            "gui_listbox_scroll_to" => { self.gui.list_scroll_to(gi(a,0,"GUI_LISTBOX_SCROLL_TO")?, gi(a,1,"GUI_LISTBOX_SCROLL_TO")?)?; Value::Nil }
            "gui_dropdown_set" => { self.gui.dropdown_set(gi(a,0,"GUI_DROPDOWN_SET")?, &gs(a,1,"GUI_DROPDOWN_SET")?, gnum(a,2,"GUI_DROPDOWN_SET")?)?; Value::Nil }
            "gui_dropdown_placeholder" => { self.gui.dropdown_placeholder(gi(a,0,"GUI_DROPDOWN_PLACEHOLDER")?, gs(a,1,"GUI_DROPDOWN_PLACEHOLDER")?.to_string())?; Value::Nil }
            "gui_tree_item_rect" => {
                let (x, y, w, h) = self.gui.tree_item_rect(gi(a,0,"GUI_TREE_ITEM_RECT")?, gi(a,1,"GUI_TREE_ITEM_RECT")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(x as i64), Value::Int(y as i64), Value::Int(w as i64), Value::Int(h as i64)]))
            }
            "gui_tree_edit" => { self.gui.tree_edit(gi(a,0,"GUI_TREE_EDIT")?, gi(a,1,"GUI_TREE_EDIT")?)?; Value::Nil }
            "gui_tree_edited" => Value::Int(self.gui.tree_edited(gi(a,0,"GUI_TREE_EDITED")?)?),
            "gui_tree_editing" => Value::Int(self.gui.tree_editing(gi(a,0,"GUI_TREE_EDITING")?)?),
            "gui_tree_placeholder" => { self.gui.tree_placeholder(gi(a,0,"GUI_TREE_PLACEHOLDER")?, gs(a,1,"GUI_TREE_PLACEHOLDER")?.to_string())?; Value::Nil }
            "gui_listbox_edit" => { self.gui.list_edit(gi(a,0,"GUI_LISTBOX_EDIT")?, gi(a,1,"GUI_LISTBOX_EDIT")?)?; Value::Nil }
            "gui_listbox_edited" => Value::Int(self.gui.list_edited(gi(a,0,"GUI_LISTBOX_EDITED")?)?),
            "gui_listbox_editing" => Value::Int(self.gui.list_editing(gi(a,0,"GUI_LISTBOX_EDITING")?)?),
            // --- Stufe 29: gesetzter Text -- Auswahl und Suche ---
            "gui_richtext_selection$" | "gui_richtext_selection" => Value::str_rc(&self.gui.richtext_selection(gi(a,0,"GUI_RICHTEXT_SELECTION")?)?),
            "gui_richtext_select_all" => { self.gui.richtext_select_all(gi(a,0,"GUI_RICHTEXT_SELECT_ALL")?)?; Value::Nil }
            "gui_richtext_clear_selection" => { self.gui.richtext_clear_selection(gi(a,0,"GUI_RICHTEXT_CLEAR_SELECTION")?)?; Value::Nil }
            "gui_richtext_find_next" => Value::Int(self.gui.richtext_find_step(gi(a,0,"GUI_RICHTEXT_FIND_NEXT")?, &gs(a,1,"GUI_RICHTEXT_FIND_NEXT")?, true)?),
            "gui_richtext_find_prev" => Value::Int(self.gui.richtext_find_step(gi(a,0,"GUI_RICHTEXT_FIND_PREV")?, &gs(a,1,"GUI_RICHTEXT_FIND_PREV")?, false)?),
            "gui_richtext_mark_all" => Value::Int(self.gui.richtext_mark_all(gi(a,0,"GUI_RICHTEXT_MARK_ALL")?, &gs(a,1,"GUI_RICHTEXT_MARK_ALL")?)?),
            // --- Stufe 28: Statusleiste, Pfadleiste, Ueberlauf ---
            "gui_statusbar" => {
                let t = if a.len() > 5 { gs(a,5,"GUI_STATUSBAR")? } else { String::new() };
                Value::Int(self.gui.statusbar(gi(a,0,"GUI_STATUSBAR")?, gi(a,1,"GUI_STATUSBAR")? as i32,
                    gi(a,2,"GUI_STATUSBAR")? as i32, gi(a,3,"GUI_STATUSBAR")? as i32, gi(a,4,"GUI_STATUSBAR")? as i32, t)?)
            }
            "gui_statusbar_add" => {
                let ausr = if a.len() > 3 { gs(a,3,"GUI_STATUSBAR_ADD")? } else { String::new() };
                Value::Int(self.gui.statusbar_add(gi(a,0,"GUI_STATUSBAR_ADD")?, gs(a,1,"GUI_STATUSBAR_ADD")?, gi(a,2,"GUI_STATUSBAR_ADD")?, &ausr)?)
            }
            "gui_statusbar_set" => { self.gui.statusbar_set(gi(a,0,"GUI_STATUSBAR_SET")?, gi(a,1,"GUI_STATUSBAR_SET")?, gs(a,2,"GUI_STATUSBAR_SET")?)?; Value::Nil }
            "gui_statusbar_text$" | "gui_statusbar_text" => Value::str_rc(&self.gui.statusbar_text(gi(a,0,"GUI_STATUSBAR_TEXT")?, gi(a,1,"GUI_STATUSBAR_TEXT")?)?),
            "gui_statusbar_tip" => { self.gui.statusbar_tip(gi(a,0,"GUI_STATUSBAR_TIP")?, gi(a,1,"GUI_STATUSBAR_TIP")?, gs(a,2,"GUI_STATUSBAR_TIP")?)?; Value::Nil }
            "gui_statusbar_clickable" => { self.gui.statusbar_clickable(gi(a,0,"GUI_STATUSBAR_CLICKABLE")?, gi(a,1,"GUI_STATUSBAR_CLICKABLE")?, gbool(a,2,"GUI_STATUSBAR_CLICKABLE")?)?; Value::Nil }
            "gui_statusbar_clicked" => Value::Int(self.gui.statusbar_clicked(gi(a,0,"GUI_STATUSBAR_CLICKED")?)?),
            "gui_statusbar_count" => Value::Int(self.gui.statusbar_count(gi(a,0,"GUI_STATUSBAR_COUNT")?)?),
            "gui_statusbar_field_x" => Value::Int(self.gui.statusbar_field_rect(gi(a,0,"GUI_STATUSBAR_FIELD_X")?, gi(a,1,"GUI_STATUSBAR_FIELD_X")?)?.0),
            "gui_statusbar_field_w" => Value::Int(self.gui.statusbar_field_rect(gi(a,0,"GUI_STATUSBAR_FIELD_W")?, gi(a,1,"GUI_STATUSBAR_FIELD_W")?)?.1),
            "gui_breadcrumb" => Value::Int(self.gui.breadcrumb(gi(a,0,"GUI_BREADCRUMB")?, gi(a,1,"GUI_BREADCRUMB")? as i32,
                gi(a,2,"GUI_BREADCRUMB")? as i32, gi(a,3,"GUI_BREADCRUMB")? as i32, gi(a,4,"GUI_BREADCRUMB")? as i32)?),
            "gui_breadcrumb_add" => {
                let wert = if a.len() > 2 { gs(a,2,"GUI_BREADCRUMB_ADD")? } else { String::new() };
                Value::Int(self.gui.breadcrumb_add(gi(a,0,"GUI_BREADCRUMB_ADD")?, gs(a,1,"GUI_BREADCRUMB_ADD")?, wert)?)
            }
            "gui_breadcrumb_set" => { self.gui.breadcrumb_set(gi(a,0,"GUI_BREADCRUMB_SET")?, gstrs(a,1,"GUI_BREADCRUMB_SET")?)?; Value::Nil }
            "gui_breadcrumb_clear" => { self.gui.breadcrumb_clear(gi(a,0,"GUI_BREADCRUMB_CLEAR")?)?; Value::Nil }
            "gui_breadcrumb_count" => Value::Int(self.gui.breadcrumb_count(gi(a,0,"GUI_BREADCRUMB_COUNT")?)?),
            "gui_breadcrumb_clicked" => Value::Int(self.gui.breadcrumb_clicked(gi(a,0,"GUI_BREADCRUMB_CLICKED")?)?),
            "gui_breadcrumb_text$" | "gui_breadcrumb_text" => Value::str_rc(&self.gui.breadcrumb_text(gi(a,0,"GUI_BREADCRUMB_TEXT")?, gi(a,1,"GUI_BREADCRUMB_TEXT")?)?),
            "gui_breadcrumb_data$" | "gui_breadcrumb_data" => Value::str_rc(&self.gui.breadcrumb_data(gi(a,0,"GUI_BREADCRUMB_DATA")?, gi(a,1,"GUI_BREADCRUMB_DATA")?)?),
            "gui_breadcrumb_first_visible" => Value::Int(self.gui.breadcrumb_first_visible(gi(a,0,"GUI_BREADCRUMB_FIRST_VISIBLE")?)?),
            "gui_toolbar_overflow" => Value::Int(self.gui.toolbar_overflow(gi(a,0,"GUI_TOOLBAR_OVERFLOW")?)?),
            "gui_button_variant" => { self.gui.set_button_variant(gi(a,0,"GUI_BUTTON_VARIANT")?, &gs(a,1,"GUI_BUTTON_VARIANT")?)?; Value::Nil }
            "gui_button_get_variant$" | "gui_button_get_variant" => Value::str_rc(&self.gui.button_variant(gi(a,0,"GUI_BUTTON_GET_VARIANT")?)?),
            "gui_double_clicked" => Value::Bool(self.gui.double_clicked(gi(a,0,"GUI_DOUBLE_CLICKED")?)?),
            "gui_layout" => Value::Int(self.gui.layout_new(gi(a,0,"GUI_LAYOUT")?, &gs(a,1,"GUI_LAYOUT")?,
                gi(a,2,"GUI_LAYOUT")? as i32, gi(a,3,"GUI_LAYOUT")? as i32, gi(a,4,"GUI_LAYOUT")? as i32, gi(a,5,"GUI_LAYOUT")? as i32)?),
            "gui_layout_add" => {
                let gew = if a.len() > 2 { gi(a,2,"GUI_LAYOUT_ADD")? } else { 0 };
                self.gui.layout_add(gi(a,0,"GUI_LAYOUT_ADD")?, gi(a,1,"GUI_LAYOUT_ADD")?, gew)?; Value::Nil
            }
            "gui_layout_spacer" => {
                let gew = if a.len() > 1 { gi(a,1,"GUI_LAYOUT_SPACER")? } else { 1 };
                self.gui.layout_spacer(gi(a,0,"GUI_LAYOUT_SPACER")?, gew)?; Value::Nil
            }
            "gui_layout_remove" => { self.gui.layout_remove(gi(a,0,"GUI_LAYOUT_REMOVE")?, gi(a,1,"GUI_LAYOUT_REMOVE")?)?; Value::Nil }
            "gui_layout_set" => { self.gui.layout_set(gi(a,0,"GUI_LAYOUT_SET")?, &gs(a,1,"GUI_LAYOUT_SET")?, gnum(a,2,"GUI_LAYOUT_SET")?)?; Value::Nil }
            "gui_autosize" => { let g = self.gfx.as_ref(); self.gui.autosize(g, gi(a,0,"GUI_AUTOSIZE")?)?; Value::Nil }
            // --- Datenbindung ---
            "gui_bind" => {
                let form = if a.len() > 2 { gs(a,2,"GUI_BIND")? } else { String::new() };
                self.gui.bind(gi(a,0,"GUI_BIND")?, gs(a,1,"GUI_BIND")?, form)?; Value::Nil
            }
            "gui_form_get" => {
                let form = if a.len() > 1 { gs(a,1,"GUI_FORM_GET")? } else { String::new() };
                let mut m = DhMap::new("string".to_string());
                for (k, t) in self.gui.form_get(gi(a,0,"GUI_FORM_GET")?, &form)? { m.put(k, Value::str_rc(&t)); }
                Value::Map(Rc::new(RefCell::new(m)))
            }
            "gui_form_set" => {
                let form = if a.len() > 2 { gs(a,2,"GUI_FORM_SET")? } else { String::new() };
                let werte: Vec<(String, Value)> = match a.get(1) {
                    Some(Value::Map(m)) => m.borrow().entries().iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    _ => return Err("GUI_FORM_SET: erwartet eine MAP mit den Werten".into()),
                };
                self.gui.form_set(gi(a,0,"GUI_FORM_SET")?, &form, &werte)?; Value::Nil
            }
            "gui_form_clear" => {
                let form = if a.len() > 1 { gs(a,1,"GUI_FORM_CLEAR")? } else { String::new() };
                self.gui.form_clear(gi(a,0,"GUI_FORM_CLEAR")?, &form)?; Value::Nil
            }
            "gui_form_clean" => {
                let form = if a.len() > 1 { gs(a,1,"GUI_FORM_CLEAN")? } else { String::new() };
                self.gui.form_clean(gi(a,0,"GUI_FORM_CLEAN")?, &form)?; Value::Nil
            }
            "gui_form_changed" => {
                let form = if a.len() > 1 { gs(a,1,"GUI_FORM_CHANGED")? } else { String::new() };
                Value::Bool(self.gui.form_changed(gi(a,0,"GUI_FORM_CHANGED")?, &form)?)
            }
            // GUI_FORM_VERSION(win, spalte$[, formular$]) -- Versionsspalte fuer
            // GUI_FORM_LOAD/SAVE (optimistische Sperre), leer = aus.
            "gui_changed" => Value::Bool(self.gui.changed(gi(a,0,"GUI_CHANGED")?)?),
            "gui_form_version" => {
                let form = if a.len() > 2 { gs(a,2,"GUI_FORM_VERSION")? } else { String::new() };
                self.gui.form_version_setzen(gi(a,0,"GUI_FORM_VERSION")?, &form, &gs(a,1,"GUI_FORM_VERSION")?)?;
                Value::Nil
            }
            "gui_form_conflict$" | "gui_form_conflict" => {
                let form = if a.len() > 1 { gs(a,1,"GUI_FORM_CONFLICT$")? } else { String::new() };
                Value::str_rc(self.gui.form_konflikt(gi(a,0,"GUI_FORM_CONFLICT$")?, &form)?)
            }
            // GUI_FORM_LOAD(win, db, tabelle$, id[, formular$]) -> BOOLEAN
            // GUI_FORM_SAVE(win, db, tabelle$, id[, formular$[, ueberschreiben]]) -> id
            // Die Spalte heisst `id` -- die Konvention von SQLite (rowid-Alias)
            // und von jedem ORM; wer anders heisst, schreibt sein SQL selbst.
            //
            // SAVE liefert 0, wenn nichts gespeichert wurde: die Zeile gibt es
            // nicht mehr ("geloescht") oder -- mit GUI_FORM_VERSION -- ein
            // anderer hat sie seit dem Laden gespeichert ("geaendert");
            // GUI_FORM_CONFLICT$ sagt welches. Bis 2026.30 lieferte ein UPDATE
            // auf eine geloeschte Zeile die Kennung zurueck, als waere alles
            // gespeichert -- die Eingabe war still weg (Vereinsverwaltung).
            "gui_form_load" | "gui_form_save" => {
                #[cfg(feature = "db")]
                {
                    let fn_ = if name == "gui_form_load" { "GUI_FORM_LOAD" } else { "GUI_FORM_SAVE" };
                    let win = gi(a,0,fn_)?;
                    let conn_idx = gi(a,1,fn_)?;
                    let tabelle = gs(a,2,fn_)?;
                    let id = gi(a,3,fn_)?;
                    let form = if a.len() > 4 { gs(a,4,fn_)? } else { String::new() };
                    if tabelle.is_empty() || !tabelle.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        return Err(format!("{}: '{}' ist kein Tabellenname (Buchstaben, Ziffern, _)", fn_, tabelle));
                    }
                    let keys = self.gui.form_keys(win, &form)?;
                    if keys.is_empty() {
                        return Err(format!("{}: im Formular '{}' ist kein Widget gebunden (GUI_BIND)", fn_, form));
                    }
                    let (vspalte, vstand) = self.gui.form_version(win, &form);
                    let mit_version = !vspalte.is_empty();
                    if name == "gui_form_load" {
                        let spalten = if mit_version { format!("{}, {}", keys.join(", "), vspalte) } else { keys.join(", ") };
                        let sql = format!("SELECT {} FROM {} WHERE id = ?", spalten, tabelle);
                        let res = self.db_conn(conn_idx)?.query(&sql, &[crate::db::Param::Int(id)], fn_)?;
                        match res.rows.first() {
                            None => Value::Bool(false),
                            Some(row) => {
                                if mit_version {
                                    let stand = match row.last() {
                                        Some(crate::db::DbVal::Int(n)) => Some(*n),
                                        Some(crate::db::DbVal::Real(f)) => Some(*f as i64),
                                        Some(crate::db::DbVal::Text(t)) | Some(crate::db::DbVal::Zahl(t)) => t.trim().parse::<i64>().ok(),
                                        _ => None,
                                    };
                                    if stand.is_none() {
                                        return Err(format!("{}: die Versionsspalte '{}' ist leer oder keine ganze Zahl", fn_, vspalte));
                                    }
                                    self.gui.form_stand_setzen(win, &form, stand);
                                }
                                self.gui.form_konflikt_setzen(win, &form, "");
                                let werte: Vec<(String, Value)> = keys.iter().cloned().zip(row.iter().map(|z| match z {
                                    crate::db::DbVal::Null => Value::str_rc(""),
                                    crate::db::DbVal::Int(n) => Value::Int(*n),
                                    crate::db::DbVal::Real(f) => Value::Float(*f),
                                    crate::db::DbVal::Text(t) | crate::db::DbVal::Zahl(t) => Value::str_rc(t),
                                    crate::db::DbVal::Blob(b) => Value::str_rc(String::from_utf8_lossy(b).into_owned()),
                                })).collect();
                                self.gui.form_set(win, &form, &werte)?;
                                Value::Bool(true)
                            }
                        }
                    } else {
                        let ueberschreiben = a.len() > 5 && gbool(a,5,fn_)?;
                        if a.len() > 6 { return Err(format!("{}: zu viele Argumente", fn_)); }
                        let werte = self.gui.form_get_typisiert(win, &form)?;
                        let mut params: Vec<crate::db::Param> = Vec::new();
                        for (_, wv) in &werte { params.push(crate::db::dh_to_sql(wv, fn_)?); }
                        let neue_id = if id < 0 {
                            let (spalten, platz) = if mit_version {
                                (format!("{}, {}", keys.join(", "), vspalte),
                                 format!("{}, 1", keys.iter().map(|_| "?").collect::<Vec<_>>().join(", ")))
                            } else {
                                (keys.join(", "), keys.iter().map(|_| "?").collect::<Vec<_>>().join(", "))
                            };
                            let sql = format!("INSERT INTO {} ({}) VALUES ({})", tabelle, spalten, platz);
                            let nid = self.db_conn(conn_idx)?.einfuegen(&sql, &params, fn_)?;
                            if mit_version { self.gui.form_stand_setzen(win, &form, Some(1)); }
                            nid
                        } else {
                            let mut setz = keys.iter().map(|k| format!("{} = ?", k)).collect::<Vec<_>>().join(", ");
                            if mit_version { setz = format!("{}, {} = {} + 1", setz, vspalte, vspalte); }
                            params.push(crate::db::Param::Int(id));
                            let pruefen = mit_version && !ueberschreiben;
                            let sql = if pruefen {
                                let stand = vstand.ok_or_else(|| format!(
                                    "{}: dieses Formular hat die Versionsspalte '{}', wurde aber nicht mit GUI_FORM_LOAD geladen -- \
                                     es weiss nicht, welchen Stand es bearbeitet (oder ueberschreiben = TRUE)", fn_, vspalte))?;
                                params.push(crate::db::Param::Int(stand));
                                format!("UPDATE {} SET {} WHERE id = ? AND {} = ?", tabelle, setz, vspalte)
                            } else {
                                format!("UPDATE {} SET {} WHERE id = ?", tabelle, setz)
                            };
                            let n = self.db_conn(conn_idx)?.exec(&sql, &params, fn_)?;
                            if n == 0 {
                                // Getroffen hat es nichts: gibt es die Zeile noch?
                                let sql = format!("SELECT {} FROM {} WHERE id = ?",
                                                  if mit_version { vspalte.as_str() } else { "id" }, tabelle);
                                let da = !self.db_conn(conn_idx)?.query(&sql, &[crate::db::Param::Int(id)], fn_)?.rows.is_empty();
                                self.gui.form_konflikt_setzen(win, &form, if da && pruefen { "geaendert" } else { "geloescht" });
                                return Ok(Some(Value::Int(0)));
                            }
                            if mit_version {
                                let neu = match vstand { Some(s) if !ueberschreiben => Some(s + 1), _ => {
                                    let sql = format!("SELECT {} FROM {} WHERE id = ?", vspalte, tabelle);
                                    let res = self.db_conn(conn_idx)?.query(&sql, &[crate::db::Param::Int(id)], fn_)?;
                                    match res.rows.first().and_then(|z| z.first()) {
                                        Some(crate::db::DbVal::Int(n)) => Some(*n),
                                        Some(crate::db::DbVal::Real(f)) => Some(*f as i64),
                                        Some(crate::db::DbVal::Text(t)) | Some(crate::db::DbVal::Zahl(t)) => t.trim().parse::<i64>().ok(),
                                        _ => None,
                                    }
                                }};
                                self.gui.form_stand_setzen(win, &form, neu);
                            }
                            id
                        };
                        self.gui.form_konflikt_setzen(win, &form, "");
                        self.gui.form_clean(win, &form)?;
                        Value::Int(neue_id)
                    }
                }
                #[cfg(not(feature = "db"))]
                { return Err("GUI_FORM_LOAD/SAVE: dieser Bau hat kein db-Modul".into()); }
            }
            // --- Mindestmasse + Formularpruefung ---
            "gui_set_min_size" => { self.gui.set_min_size(gi(a,0,"GUI_SET_MIN_SIZE")?, gi(a,1,"GUI_SET_MIN_SIZE")? as i32, gi(a,2,"GUI_SET_MIN_SIZE")? as i32)?; Value::Nil }
            "gui_layout_min_w" => Value::Int(self.gui.layout_min(gi(a,0,"GUI_LAYOUT_MIN_W")?)?.0),
            "gui_layout_min_h" => Value::Int(self.gui.layout_min(gi(a,0,"GUI_LAYOUT_MIN_H")?)?.1),
            "gui_rule" => {
                // GUI_RULE(feld, art$[, a, b][, muster$][, meldung$]) -- die
                // Zahl der Argumente haengt an der Art; der letzte Text ist
                // immer die Meldung, wenn einer uebrig bleibt.
                let feld = gi(a,0,"GUI_RULE")?;
                let art = gs(a,1,"GUI_RULE")?.to_ascii_lowercase();
                let (mut x, mut y, mut muster, mut meldung) = (0.0, 0.0, String::new(), String::new());
                let mut rest = 2;
                match art.as_str() {
                    "bereich" | "range" | "laenge" | "länge" | "length" => {
                        if a.len() < 4 { return Err(format!("GUI_RULE: '{}' braucht von und bis", art)); }
                        x = gnum(a,2,"GUI_RULE")?; y = gnum(a,3,"GUI_RULE")?; rest = 4;
                    }
                    "muster" | "pattern" | "regex" => {
                        if a.len() < 3 { return Err("GUI_RULE: 'muster' braucht das Muster".into()); }
                        muster = gs(a,2,"GUI_RULE")?; rest = 3;
                    }
                    _ => {}
                }
                if a.len() > rest { meldung = gs(a,rest,"GUI_RULE")?; }
                if a.len() > rest + 1 { return Err("GUI_RULE: zu viele Argumente".into()); }
                self.gui.rule_add(feld, &art, x, y, muster, meldung)?; Value::Nil
            }
            "gui_rules_clear" => { self.gui.rules_clear(gi(a,0,"GUI_RULES_CLEAR")?)?; Value::Nil }
            "gui_validate" => Value::Int(self.gui.validate(gi(a,0,"GUI_VALIDATE")?)?),
            "gui_validate_widget" => Value::Str(Rc::from(self.gui.validate_widget(gi(a,0,"GUI_VALIDATE_WIDGET")?)?)),
            "gui_error" => Value::Str(Rc::from(self.gui.error_get(gi(a,0,"GUI_ERROR")?)?)),
            "gui_set_error" => { self.gui.error_set(gi(a,0,"GUI_SET_ERROR")?, gs(a,1,"GUI_SET_ERROR")?)?; Value::Nil }
            "gui_clear_errors" => { self.gui.errors_clear(gi(a,0,"GUI_CLEAR_ERRORS")?)?; Value::Nil }
            "gui_error_label" => { self.gui.error_label(gi(a,0,"GUI_ERROR_LABEL")?, gi(a,1,"GUI_ERROR_LABEL")?)?; Value::Nil }
            "gui_validate_live" => { self.gui.validate_live(gi(a,0,"GUI_VALIDATE_LIVE")?, gbool(a,1,"GUI_VALIDATE_LIVE")?)?; Value::Nil }
            // --- Punkt 6: senkrechter Schieber, Fortschritt, Bildmodi, Cursor, Baum, Panel, Ziehen ---
            "gui_vslider" => Value::Int(self.gui.vslider(gi(a,0,"GUI_VSLIDER")?, gi(a,1,"GUI_VSLIDER")? as i32,
                gi(a,2,"GUI_VSLIDER")? as i32, gi(a,3,"GUI_VSLIDER")? as i32,
                gnum(a,4,"GUI_VSLIDER")?, gnum(a,5,"GUI_VSLIDER")?, gnum(a,6,"GUI_VSLIDER")?)?),
            "gui_progress_set" => { self.gui.progress_set(gi(a,0,"GUI_PROGRESS_SET")?, &gs(a,1,"GUI_PROGRESS_SET")?, gnum(a,2,"GUI_PROGRESS_SET")?)?; Value::Nil }
            "gui_image_mode" => { self.gui.image_mode(gi(a,0,"GUI_IMAGE_MODE")?, &gs(a,1,"GUI_IMAGE_MODE")?)?; Value::Nil }
            "gui_image_mode_get" => Value::Str(Rc::from(self.gui.image_mode_get(gi(a,0,"GUI_IMAGE_MODE_GET")?)?)),
            "gui_cursors" => { self.gui.cursors(gbool(a,0,"GUI_CURSORS")?); Value::Nil }
            "gui_set_cursor" => { self.gui.set_cursor(gi(a,0,"GUI_SET_CURSOR")?, &gs(a,1,"GUI_SET_CURSOR")?)?; Value::Nil }
            "gui_get_cursor$" | "gui_get_cursor" => Value::str_rc(self.gui.get_cursor(gi(a,0,"GUI_GET_CURSOR$")?)?),
            "gui_tree_icon" => {
                // Ein Bild (Handle) oder der Name eines eingebauten Sinnbilds.
                if let Some(Value::Str(n)) = a.get(2) {
                    let n = n.to_string();
                    self.gui.tree_icon_sym(gi(a,0,"GUI_TREE_ICON")?, gi(a,1,"GUI_TREE_ICON")?, &n)?;
                } else {
                    self.gui.tree_icon(gi(a,0,"GUI_TREE_ICON")?, gi(a,1,"GUI_TREE_ICON")?, gi(a,2,"GUI_TREE_ICON")?)?;
                }
                Value::Nil
            }
            "gui_tree_color" => { self.gui.tree_color(gi(a,0,"GUI_TREE_COLOR")?, gi(a,1,"GUI_TREE_COLOR")?, gi(a,2,"GUI_TREE_COLOR")?)?; Value::Nil }
            "gui_panel_add" => { self.gui.panel_add(gi(a,0,"GUI_PANEL_ADD")?, gi(a,1,"GUI_PANEL_ADD")?)?; Value::Nil }
            "gui_panel_remove" => { self.gui.panel_remove(gi(a,0,"GUI_PANEL_REMOVE")?, gi(a,1,"GUI_PANEL_REMOVE")?)?; Value::Nil }
            "gui_panel_scroll" => { self.gui.panel_scroll(gi(a,0,"GUI_PANEL_SCROLL")?, gi(a,1,"GUI_PANEL_SCROLL")? as i32)?; Value::Nil }
            "gui_panel_scroll_get" => Value::Int(self.gui.panel_scroll_get(gi(a,0,"GUI_PANEL_SCROLL_GET")?)?),
            "gui_draggable" => { self.gui.draggable(gi(a,0,"GUI_DRAGGABLE")?, gbool(a,1,"GUI_DRAGGABLE")?)?; Value::Nil }
            "gui_drop_target" => { self.gui.drop_target(gi(a,0,"GUI_DROP_TARGET")?, gbool(a,1,"GUI_DROP_TARGET")?)?; Value::Nil }
            "gui_dragging" => Value::Int(self.gui.dragging()),
            "gui_dropped" => Value::Bool(self.gui.dropped(gi(a,0,"GUI_DROPPED")?)?),
            "gui_drop_text" => Value::Str(Rc::from(self.gui.drop_text())),
            "gui_drop_source" => Value::Int(self.gui.drop_source()),
            "gui_drop_index" => Value::Int(self.gui.drop_index()),
            "gui_drag_index" => Value::Int(self.gui.drag_index()),
            // --- Image + Canvas ---
            "gui_image" => Value::Int(self.gui.image(gi(a,0,"GUI_IMAGE")?, gi(a,1,"GUI_IMAGE")? as i32,
                gi(a,2,"GUI_IMAGE")? as i32, gi(a,3,"GUI_IMAGE")? as i32, gi(a,4,"GUI_IMAGE")? as i32, gi(a,5,"GUI_IMAGE")?)?),
            "gui_set_image" => { self.gui.set_image(gi(a,0,"GUI_SET_IMAGE")?, gi(a,1,"GUI_SET_IMAGE")?)?; Value::Nil }
            "gui_card" => {
                // GUI_CARD(win, x, y, w, h, bild [, titel$ [, text$]]) -- bild -1 = keins
                let f = "GUI_CARD";
                let titel = if a.len() >= 7 { gs(a,6,f)? } else { String::new() };
                let text = if a.len() >= 8 { gs(a,7,f)? } else { String::new() };
                Value::Int(self.gui.card(gi(a,0,f)?, gi(a,1,f)? as i32, gi(a,2,f)? as i32,
                    gi(a,3,f)? as i32, gi(a,4,f)? as i32, gi(a,5,f)?, titel, text)?)
            }
            "gui_card_set_text" => { self.gui.card_set_text(gi(a,0,"GUI_CARD_SET_TEXT")?, gs(a,1,"GUI_CARD_SET_TEXT")?)?; Value::Nil }
            "gui_card_text$" | "gui_card_text" => Value::str_rc(&self.gui.card_text(gi(a,0,"GUI_CARD_TEXT$")?)?),
            "gui_window_at" => Value::Int(self.gui.window_at(gi(a,0,"GUI_WINDOW_AT")? as i32, gi(a,1,"GUI_WINDOW_AT")? as i32)),
            "gui_icon_button" => {
                let text = if a.len() >= 7 { gs(a,6,"GUI_ICON_BUTTON")? } else { String::new() };
                let (sym, tex) = gsym(a,5,"GUI_ICON_BUTTON")?;
                let h = self.gui.icon_button(gi(a,0,"GUI_ICON_BUTTON")?, gi(a,1,"GUI_ICON_BUTTON")? as i32,
                    gi(a,2,"GUI_ICON_BUTTON")? as i32, gi(a,3,"GUI_ICON_BUTTON")? as i32, gi(a,4,"GUI_ICON_BUTTON")? as i32,
                    tex, text)?;
                if let Some(s) = sym { self.gui.set_icon_symbol(h, &s)?; }
                Value::Int(h)
            }
            "gui_set_icon" => {
                match gsym(a,1,"GUI_SET_ICON")? {
                    (Some(s), _) => self.gui.set_icon_symbol(gi(a,0,"GUI_SET_ICON")?, &s)?,
                    (None, t) => self.gui.set_icon(gi(a,0,"GUI_SET_ICON")?, t)?,
                }
                Value::Nil
            }
            // --- Werkzeugleiste mit Eintraegen ---
            "gui_toolbar_add" => {
                let (sym, tex) = gsym(a,1,"GUI_TOOLBAR_ADD")?;
                let tip = if a.len() > 2 { gs(a,2,"GUI_TOOLBAR_ADD")? } else { String::new() };
                let text = if a.len() > 3 { gs(a,3,"GUI_TOOLBAR_ADD")? } else { String::new() };
                Value::Int(self.gui.toolbar_add(gi(a,0,"GUI_TOOLBAR_ADD")?, sym, tex, tip, text)?)
            }
            "gui_toolbar_separator" => Value::Int(self.gui.toolbar_separator(gi(a,0,"GUI_TOOLBAR_SEPARATOR")?)?),
            "gui_toolbar_spacer" => Value::Int(self.gui.toolbar_spacer(gi(a,0,"GUI_TOOLBAR_SPACER")?)?),
            "gui_toolbar_clicked" => Value::Int(self.gui.toolbar_clicked(gi(a,0,"GUI_TOOLBAR_CLICKED")?)?),
            "gui_toolbar_count" => Value::Int(self.gui.toolbar_count(gi(a,0,"GUI_TOOLBAR_COUNT")?)?),
            "gui_toolbar_clear" => { self.gui.toolbar_clear(gi(a,0,"GUI_TOOLBAR_CLEAR")?)?; Value::Nil }
            "gui_toolbar_enable" => { self.gui.toolbar_enable(gi(a,0,"GUI_TOOLBAR_ENABLE")?, gi(a,1,"GUI_TOOLBAR_ENABLE")?, gbool(a,2,"GUI_TOOLBAR_ENABLE")?)?; Value::Nil }
            "gui_toolbar_enabled" => Value::Bool(self.gui.toolbar_enabled(gi(a,0,"GUI_TOOLBAR_ENABLED")?, gi(a,1,"GUI_TOOLBAR_ENABLED")?)?),
            "gui_toolbar_checkable" => { self.gui.toolbar_checkable(gi(a,0,"GUI_TOOLBAR_CHECKABLE")?, gi(a,1,"GUI_TOOLBAR_CHECKABLE")?, gbool(a,2,"GUI_TOOLBAR_CHECKABLE")?)?; Value::Nil }
            "gui_toolbar_set_checked" => { self.gui.toolbar_set_checked(gi(a,0,"GUI_TOOLBAR_SET_CHECKED")?, gi(a,1,"GUI_TOOLBAR_SET_CHECKED")?, gbool(a,2,"GUI_TOOLBAR_SET_CHECKED")?)?; Value::Nil }
            "gui_toolbar_checked" => Value::Bool(self.gui.toolbar_checked(gi(a,0,"GUI_TOOLBAR_CHECKED")?, gi(a,1,"GUI_TOOLBAR_CHECKED")?)?),
            "gui_toolbar_set_icon" => {
                let (sym, tex) = gsym(a,2,"GUI_TOOLBAR_SET_ICON")?;
                self.gui.toolbar_set_icon(gi(a,0,"GUI_TOOLBAR_SET_ICON")?, gi(a,1,"GUI_TOOLBAR_SET_ICON")?, sym, tex)?; Value::Nil
            }
            "gui_toolbar_set_tip" => { self.gui.toolbar_set_tip(gi(a,0,"GUI_TOOLBAR_SET_TIP")?, gi(a,1,"GUI_TOOLBAR_SET_TIP")?, gs(a,2,"GUI_TOOLBAR_SET_TIP")?)?; Value::Nil }
            "gui_toolbar_set_text" => { self.gui.toolbar_set_text(gi(a,0,"GUI_TOOLBAR_SET_TEXT")?, gi(a,1,"GUI_TOOLBAR_SET_TEXT")?, gs(a,2,"GUI_TOOLBAR_SET_TEXT")?)?; Value::Nil }
            "gui_toolbar_set" => { self.gui.toolbar_set(gi(a,0,"GUI_TOOLBAR_SET")?, &gs(a,1,"GUI_TOOLBAR_SET")?, gnum(a,2,"GUI_TOOLBAR_SET")?)?; Value::Nil }
            "gui_toolbar_item_x" => Value::Int(self.gui.toolbar_item_rect(gi(a,0,"GUI_TOOLBAR_ITEM_X")?, gi(a,1,"GUI_TOOLBAR_ITEM_X")?)?.0),
            "gui_toolbar_item_w" => Value::Int(self.gui.toolbar_item_rect(gi(a,0,"GUI_TOOLBAR_ITEM_W")?, gi(a,1,"GUI_TOOLBAR_ITEM_W")?)?.1),
            "gui_toolbar" => Value::Int(self.gui.toolbar(gi(a,0,"GUI_TOOLBAR")?, gi(a,1,"GUI_TOOLBAR")? as i32,
                gi(a,2,"GUI_TOOLBAR")? as i32, gi(a,3,"GUI_TOOLBAR")? as i32, gi(a,4,"GUI_TOOLBAR")? as i32)?),
            "gui_canvas" => Value::Int(self.gui.canvas(gi(a,0,"GUI_CANVAS")?, gi(a,1,"GUI_CANVAS")? as i32,
                gi(a,2,"GUI_CANVAS")? as i32, gi(a,3,"GUI_CANVAS")? as i32, gi(a,4,"GUI_CANVAS")? as i32)?),
            "gui_canvas_x" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_X")?)?.0 as i64),
            "gui_canvas_y" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_Y")?)?.1 as i64),
            "gui_canvas_w" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_W")?)?.2 as i64),
            "gui_canvas_h" => Value::Int(self.gui.canvas_rect(gi(a,0,"GUI_CANVAS_H")?)?.3 as i64),
            "gui_clicked" => Value::Bool(self.gui.clicked(gi(a,0,"GUI_CLICKED")?)?),
            "gui_menu" => Value::Int(self.gui.add_menu(gi(a,0,"GUI_MENU")?, gs(a,1,"GUI_MENU")?.to_string())?),
            "gui_context" => Value::Int(self.gui.add_context(gi(a,0,"GUI_CONTEXT")?)?),
            "gui_context_tabs" => { self.gui.context_tabs(gi(a,0,"GUI_CONTEXT_TABS")?)?; Value::Nil }
            "gui_tab_context" => Value::Int(self.gui.tab_context(gi(a,0,"GUI_TAB_CONTEXT")?)?),
            "gui_context_widget" => { self.gui.context_widget(gi(a,0,"GUI_CONTEXT_WIDGET")?, gi(a,1,"GUI_CONTEXT_WIDGET")?)?; Value::Nil }
            "gui_menu_item" => {
                // GUI_MENU_ITEM(menu, label$[, kuerzel$])
                let h = self.gui.add_menu_item(gi(a,0,"GUI_MENU_ITEM")?, gs(a,1,"GUI_MENU_ITEM")?.to_string())?;
                if a.len() > 2 {
                    let k = gs(a,2,"GUI_MENU_ITEM")?.to_string();
                    self.gui.menu_shortcut(h, &k).map_err(|e| e.replace("GUI_MENU_SHORTCUT", "GUI_MENU_ITEM"))?;
                }
                Value::Int(h)
            }
            "gui_submenu" => Value::Int(self.gui.add_submenu(gi(a,0,"GUI_SUBMENU")?, gs(a,1,"GUI_SUBMENU")?.to_string())?),
            "gui_menu_shortcut" => { let k = gs(a,1,"GUI_MENU_SHORTCUT")?.to_string(); self.gui.menu_shortcut(gi(a,0,"GUI_MENU_SHORTCUT")?, &k)?; Value::Nil }
            "gui_menu_enable" => { self.gui.menu_enable(gi(a,0,"GUI_MENU_ENABLE")?, gbool(a,1,"GUI_MENU_ENABLE")?)?; Value::Nil }
            "gui_menu_check" => { self.gui.menu_check(gi(a,0,"GUI_MENU_CHECK")?, gbool(a,1,"GUI_MENU_CHECK")?)?; Value::Nil }
            "gui_menu_checked" => Value::Bool(self.gui.menu_checked(gi(a,0,"GUI_MENU_CHECKED")?)?),
            "gui_menu_icon" => {
                match gsym(a,1,"GUI_MENU_ICON")? {
                    (Some(s), _) => self.gui.menu_icon_symbol(gi(a,0,"GUI_MENU_ICON")?, &s)?,
                    (None, t) => self.gui.menu_icon(gi(a,0,"GUI_MENU_ICON")?, t)?,
                }
                Value::Nil
            }
            "gui_menu_text" => { self.gui.menu_text(gi(a,0,"GUI_MENU_TEXT")?, gs(a,1,"GUI_MENU_TEXT")?.to_string())?; Value::Nil }
            "gui_menu_separator" => { self.gui.add_menu_separator(gi(a,0,"GUI_MENU_SEPARATOR")?)?; Value::Nil }
            "gui_hovered" => Value::Bool(self.gui.hovered(gi(a,0,"GUI_HOVERED")?)?),
            "gui_checked" => Value::Bool(self.gui.checked(gi(a,0,"GUI_CHECKED")?)?),
            "gui_value" => Value::Float(self.gui.value(gi(a,0,"GUI_VALUE")?)?),
            "gui_text" => Value::str_rc(&self.gui.text(gi(a,0,"GUI_TEXT")?)?),
            "gui_set_text" => { self.gui.set_text(gi(a,0,"GUI_SET_TEXT")?, gs(a,1,"GUI_SET_TEXT")?)?; Value::Nil }
            "gui_tooltip" => { self.gui.set_tooltip(gi(a,0,"GUI_TOOLTIP")?, gs(a,1,"GUI_TOOLTIP")?)?; Value::Nil }
            "gui_set_checked" => { self.gui.set_checked(gi(a,0,"GUI_SET_CHECKED")?, gbool(a,1,"GUI_SET_CHECKED")?)?; Value::Nil }
            "gui_set_value" => { self.gui.set_value(gi(a,0,"GUI_SET_VALUE")?, gnum(a,1,"GUI_SET_VALUE")?)?; Value::Nil }
            "gui_on_click" => { self.gui.on_click(gi(a,0,"GUI_ON_CLICK")?, gfunc(a,1,"GUI_ON_CLICK")?)?; Value::Nil }
            "gui_on_change" => { self.gui.on_change(gi(a,0,"GUI_ON_CHANGE")?, gfunc(a,1,"GUI_ON_CHANGE")?)?; Value::Nil }
            "gui_on_enter" => { self.gui.on_enter(gi(a,0,"GUI_ON_ENTER")?, gfunc(a,1,"GUI_ON_ENTER")?)?; Value::Nil }
            "gui_entered" => Value::Bool(self.gui.entered(gi(a,0,"GUI_ENTERED")?)?),
            "gui_set_align" => { self.gui.set_align(gi(a,0,"GUI_SET_ALIGN")?, &gs(a,1,"GUI_SET_ALIGN")?)?; Value::Nil }
            "gui_set_wrap" => { self.gui.set_wrap(gi(a,0,"GUI_SET_WRAP")?, gi(a,1,"GUI_SET_WRAP")? as i32)?; Value::Nil }
            "gui_textinput_set" => {
                self.gui.textinput_set(gi(a,0,"GUI_TEXTINPUT_SET")?, &gs(a,1,"GUI_TEXTINPUT_SET")?,
                                       gnum(a,2,"GUI_TEXTINPUT_SET")?)?; Value::Nil
            }
            // Ueberlagerungen und Rueckmeldung (gui_ueber.rs).
            "gui_textinput_icon" => { let n = "GUI_TEXTINPUT_ICON";
                if a.len() != 3 { return Err(format!("{}: erwartet (tf, links$, rechts$)", n)); }
                self.gui.textinput_icon(gi(a,0,n)?, &gs(a,1,n)?, &gs(a,2,n)?)?; Value::Nil }
            "gui_icon_clicked" => Value::Int(self.gui.icon_clicked(gi(a,0,"GUI_ICON_CLICKED")?)?),
            "gui_search_changed" => Value::Bool(self.gui.search_changed(gi(a,0,"GUI_SEARCH_CHANGED")?)?),
            "gui_textinput_completion" => { let n = "GUI_TEXTINPUT_COMPLETION";
                self.gui.textinput_completion(gi(a,0,n)?, gstrs(a,1,n)?)?; Value::Nil }
            "gui_completion_chosen" => Value::Int(self.gui.completion_chosen(gi(a,0,"GUI_COMPLETION_CHOSEN")?)?),
            "gui_completion_open" => Value::Bool(self.gui.completion_open(gi(a,0,"GUI_COMPLETION_OPEN")?)?),
            "gui_popover" => { let n = "GUI_POPOVER";
                if a.len() < 3 || a.len() > 4 { return Err(format!("{}: erwartet (anker, b, h [, seite$])", n)); }
                let seite = if a.len() == 4 { gs(a,3,n)? } else { String::new() };
                Value::Int(self.gui.popover(gi(a,0,n)?, gi(a,1,n)? as i32, gi(a,2,n)? as i32, &seite)?) }
            "gui_popover_show" => { self.gui.popover_show(gi(a,0,"GUI_POPOVER_SHOW")?)?; Value::Nil }
            "gui_popover_hide" => { self.gui.popover_hide(gi(a,0,"GUI_POPOVER_HIDE")?)?; Value::Nil }
            "gui_popover_closed" => Value::Bool(self.gui.popover_closed(gi(a,0,"GUI_POPOVER_CLOSED")?)?),
            "gui_toast" => { let n = "GUI_TOAST";
                if a.len() < 2 || a.len() > 4 { return Err(format!("{}: erwartet (win, text$ [, art$ [, ms]])", n)); }
                let art = if a.len() > 2 { gs(a,2,n)? } else { String::new() };
                let ms = if a.len() > 3 { gnum(a,3,n)? } else { 3000.0 };
                self.gui.toast(gi(a,0,n)?, gs(a,1,n)?, &art, ms)?; Value::Nil }
            "gui_spell" => { if a.len() != 2 { return Err("GUI_SPELL: erwartet (feld, woerterbuch) -- woerterbuch -1 schaltet ab".into()); }
                self.gui.spell(gi(a,0,"GUI_SPELL")?, gi(a,1,"GUI_SPELL")?)?; Value::Nil }
            "gui_spell_count" => Value::Int(self.gui.spell_count(gi(a,0,"GUI_SPELL_COUNT")?)?),
            "gui_toast_count" => Value::Int(self.gui.toast_count(gi(a,0,"GUI_TOAST_COUNT")?)?),
            "gui_toast_clear" => { self.gui.toast_clear(gi(a,0,"GUI_TOAST_CLEAR")?)?; Value::Nil }
            "gui_infobar" => { let n = "GUI_INFOBAR";
                if a.len() < 2 || a.len() > 4 { return Err(format!("{}: erwartet (win, text$ [, art$ [, knoepfe$]])", n)); }
                let art = if a.len() > 2 { gs(a,2,n)? } else { String::new() };
                let kn = if a.len() > 3 { gs(a,3,n)? } else { String::new() };
                self.gui.infobar(gi(a,0,n)?, gs(a,1,n)?, &art, &kn)?; Value::Nil }
            "gui_infobar_hide" => { self.gui.infobar_hide(gi(a,0,"GUI_INFOBAR_HIDE")?)?; Value::Nil }
            "gui_infobar_answer" => Value::Int(self.gui.infobar_answer(gi(a,0,"GUI_INFOBAR_ANSWER")?)?),
            "gui_infobar_button_rect" => { let n = "GUI_INFOBAR_BUTTON_RECT";
                let (x, y, b, h) = self.gui.infobar_button_rect(gi(a,0,n)?, gi(a,1,n)?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(x), Value::Int(y), Value::Int(b), Value::Int(h)])) }
            "gui_infobar_shown" => Value::Bool(self.gui.infobar_shown(gi(a,0,"GUI_INFOBAR_SHOWN")?)?),
            // Stildatei, Name, Klassen (gui_stil.rs).
            "gui_stylesheet" => { self.gui.stylesheet(&gs(a,0,"GUI_STYLESHEET")?)?; Value::Nil }
            "gui_stylesheet_load" => { self.gui.stylesheet_load(&gs(a,0,"GUI_STYLESHEET_LOAD")?)?; Value::Nil }
            "gui_set_name" => { self.gui.set_name(gi(a,0,"GUI_SET_NAME")?, &gs(a,1,"GUI_SET_NAME")?)?; Value::Nil }
            "gui_get_name$" | "gui_get_name" => Value::str_rc(&self.gui.get_name(gi(a,0,"GUI_GET_NAME$")?)?),
            "gui_add_class" => { self.gui.add_class(gi(a,0,"GUI_ADD_CLASS")?, &gs(a,1,"GUI_ADD_CLASS")?)?; Value::Nil }
            "gui_remove_class" => { self.gui.remove_class(gi(a,0,"GUI_REMOVE_CLASS")?, &gs(a,1,"GUI_REMOVE_CLASS")?)?; Value::Nil }
            "gui_get_color" => Value::Int(self.gui.get_color(gi(a,0,"GUI_GET_COLOR")?, &gs(a,1,"GUI_GET_COLOR")?)?),
            "gui_has_class" => Value::Bool(self.gui.has_class(gi(a,0,"GUI_HAS_CLASS")?, &gs(a,1,"GUI_HAS_CLASS")?)?),
            // Bausteine (gui_bausteine.rs).
            "gui_mnemonics" => { self.gui.mnemonics(gbool(a,0,"GUI_MNEMONICS")?); Value::Nil }
            "gui_layout_selected" => Value::Int(self.gui.layout_selected(gi(a,0,"GUI_LAYOUT_SELECTED")?)?),
            "gui_layout_select" => { self.gui.layout_select(gi(a,0,"GUI_LAYOUT_SELECT")?, gi(a,1,"GUI_LAYOUT_SELECT")?)?; Value::Nil }
            "gui_label_set" => { let n = "GUI_LABEL_SET";
                let wert = match a.get(2) { Some(Value::Str(s)) => s.to_string(), Some(Value::Int(i)) => i.to_string(),
                    Some(Value::Bool(b)) => (if *b { "1" } else { "0" }).to_string(), Some(Value::Float(f)) => f.to_string(),
                    _ => return Err(format!("{}: erwartet (lbl, schluessel$, wert)", n)) };
                self.gui.label_set(gi(a,0,n)?, &gs(a,1,n)?, &wert)?; Value::Nil }
            "gui_label_selection$" | "gui_label_selection" => Value::str_rc(&self.gui.label_selection(gi(a,0,"GUI_LABEL_SELECTION$")?)?),
            "gui_slider_set" => { let n = "GUI_SLIDER_SET"; self.gui.slider_set(gi(a,0,n)?, &gs(a,1,n)?, gnum(a,2,n)?)?; Value::Nil }
            "gui_slider_mark" => { let n = "GUI_SLIDER_MARK";
                let t = if a.len() > 2 { gs(a,2,n)? } else { String::new() };
                self.gui.slider_mark(gi(a,0,n)?, gnum(a,1,n)?, t)?; Value::Nil }
            "gui_slider_clear_marks" => { self.gui.slider_clear_marks(gi(a,0,"GUI_SLIDER_CLEAR_MARKS")?)?; Value::Nil }
            "gui_busy" => { let n = "GUI_BUSY"; Value::Int(self.gui.busy(gi(a,0,n)?, gi(a,1,n)? as i32, gi(a,2,n)? as i32, gi(a,3,n)? as i32)?) }
            "gui_date_mark" => { let n = "GUI_DATE_MARK";
                let c = if a.len() > 2 { gi(a,2,n)? } else { -1 };
                self.gui.date_mark(gi(a,0,n)?, &gs(a,1,n)?, c)?; Value::Nil }
            "gui_date_unmark" => { let n = "GUI_DATE_UNMARK"; self.gui.date_unmark(gi(a,0,n)?, &gs(a,1,n)?)?; Value::Nil }
            "gui_date_clear_marks" => { self.gui.date_clear_marks(gi(a,0,"GUI_DATE_CLEAR_MARKS")?)?; Value::Nil }
            "gui_date_marked" => { let n = "GUI_DATE_MARKED"; Value::Bool(self.gui.date_marked(gi(a,0,n)?, &gs(a,1,n)?)?) }
            "gui_size_group" => { let n = "GUI_SIZE_GROUP";
                let art = if a.len() > 2 { gs(a,2,n)? } else { String::new() };
                self.gui.size_group(gi(a,0,n)?, &gs(a,1,n)?, &art)?; Value::Nil }
            "gui_font_dialog" => { let n = "GUI_FONT_DIALOG";
                let gr = if a.len() > 1 { gi(a,1,n)? as i32 } else { 16 };
                let titel = gs(a,0,n)?;
                let g = self.gfx.as_ref().ok_or("GUI_FONT_DIALOG: vor SCREEN aufgerufen")?;
                Value::Int(self.gui.font_dialog(g, titel, gr)?) }
            "gui_font_dialog_path$" | "gui_font_dialog_path" => Value::str_rc(&self.gui.font_dialog_path(gi(a,0,"GUI_FONT_DIALOG_PATH$")?)?),
            "gui_font_dialog_size" => Value::Int(self.gui.font_dialog_size(gi(a,0,"GUI_FONT_DIALOG_SIZE")?)?),
            "gui_window_default" => { self.gui.window_default(gi(a,0,"GUI_WINDOW_DEFAULT")?, gi(a,1,"GUI_WINDOW_DEFAULT")?, false)?; Value::Nil }
            "gui_window_cancel" => { self.gui.window_default(gi(a,0,"GUI_WINDOW_CANCEL")?, gi(a,1,"GUI_WINDOW_CANCEL")?, true)?; Value::Nil }
            "gui_on_hover" => { self.gui.on_hover(gi(a,0,"GUI_ON_HOVER")?, gfunc(a,1,"GUI_ON_HOVER")?)?; Value::Nil }
            "gui_on_leave" => { self.gui.on_leave(gi(a,0,"GUI_ON_LEAVE")?, gfunc(a,1,"GUI_ON_LEAVE")?)?; Value::Nil }
            "gui_on_focus" => { self.gui.on_focus(gi(a,0,"GUI_ON_FOCUS")?, gfunc(a,1,"GUI_ON_FOCUS")?)?; Value::Nil }
            "gui_on_blur" => { self.gui.on_blur(gi(a,0,"GUI_ON_BLUR")?, gfunc(a,1,"GUI_ON_BLUR")?)?; Value::Nil }
            "gui_theme" => { self.gui.theme_accent(gi(a,0,"GUI_THEME")?); Value::Nil }
            "gui_theme_set" => { self.gui.theme_set(gs(a,0,"GUI_THEME_SET")?, gi(a,1,"GUI_THEME_SET")?)?; Value::Nil }
            "gui_theme_get" => Value::Int(self.gui.theme_get(&gs(a,0,"GUI_THEME_GET")?)?),
            "gui_metric_set" => { self.gui.metric_set(gs(a,0,"GUI_METRIC_SET")?, gi(a,1,"GUI_METRIC_SET")? as i32)?; Value::Nil }
            "gui_metric_get" => Value::Int(self.gui.metric_get(&gs(a,0,"GUI_METRIC_GET")?)? as i64),
            "gui_skin" => {
                let rand = if a.len() >= 3 { gi(a,2,"GUI_SKIN")? as i32 } else { 0 };
                self.gui.set_skin(gs(a,0,"GUI_SKIN")?, gi(a,1,"GUI_SKIN")?, rand)?;
                Value::Nil
            }
            "gui_set_round" => { self.gui.set_round(gi(a,0,"GUI_SET_ROUND")?, gbool(a,1,"GUI_SET_ROUND")?)?; Value::Nil }
            "gui_set_color" => { self.gui.set_color(gi(a,0,"GUI_SET_COLOR")?, gs(a,1,"GUI_SET_COLOR")?, gi(a,2,"GUI_SET_COLOR")?)?; Value::Nil }
            "gui_theme_preset" => { self.gui.theme_preset(&gs(a,0,"GUI_THEME_PRESET")?)?; Value::Nil }
            "gui_scale" => { self.gui.set_scale(gnum(a,0,"GUI_SCALE")?)?; Value::Nil }
            "gui_scale_get" => Value::Float(self.gui.get_scale()),
            // Direkt am Fenster gefragt, nicht am gui-Spiegel: so stimmt die
            // Antwort auch in einem Programm ohne GUI_UPDATE (SPEAK ohne gui).
            "gui_screenreader" => Value::Bool(self.gfx.as_ref().map_or(false, |g| g.a11y_aktiv())),
            "gui_announce" => {
                let text = gs(a,0,"GUI_ANNOUNCE")?;
                let dringend = if a.len() > 1 { gbool(a,1,"GUI_ANNOUNCE")? } else { false };
                if a.len() > 2 { return Err("GUI_ANNOUNCE: erwartet (text$[, dringend])".into()); }
                self.gui.announce(text, dringend); Value::Nil
            }
            "gui_set_tab_index" => { self.gui.set_tab_index(gi(a,0,"GUI_SET_TAB_INDEX")?, gi(a,1,"GUI_SET_TAB_INDEX")?)?; Value::Nil }
            "gui_answer" => Value::Int(self.gui.answer(gi(a,0,"GUI_ANSWER")?)?),
            "gui_modal" => Value::Bool(self.gui.modal_offen()),
            "gui_reset" => { self.gui.reset(); Value::Nil }
            // --- Laufzeit-Manipulation (Geometrie / Lifecycle / Hit-Test) ---
            "gui_set_bounds" => { self.gui.set_bounds(gi(a,0,"GUI_SET_BOUNDS")?, gi(a,1,"GUI_SET_BOUNDS")? as i32, gi(a,2,"GUI_SET_BOUNDS")? as i32, gi(a,3,"GUI_SET_BOUNDS")? as i32, gi(a,4,"GUI_SET_BOUNDS")? as i32)?; Value::Nil }
            "gui_get_x" => Value::Int(self.gui.widget_bounds(gi(a,0,"GUI_GET_X")?, "GUI_GET_X")?.0 as i64),
            "gui_get_y" => Value::Int(self.gui.widget_bounds(gi(a,0,"GUI_GET_Y")?, "GUI_GET_Y")?.1 as i64),
            "gui_get_w" => Value::Int(self.gui.widget_bounds(gi(a,0,"GUI_GET_W")?, "GUI_GET_W")?.2 as i64),
            "gui_get_h" => Value::Int(self.gui.widget_bounds(gi(a,0,"GUI_GET_H")?, "GUI_GET_H")?.3 as i64),
            "gui_destroy" => { self.gui.destroy(gi(a,0,"GUI_DESTROY")?)?; Value::Nil }
            "gui_set_visible" => { self.gui.set_widget_visible(gi(a,0,"GUI_SET_VISIBLE")?, gbool(a,1,"GUI_SET_VISIBLE")?)?; Value::Nil }
            "gui_visible" => Value::Bool(self.gui.widget_visible(gi(a,0,"GUI_VISIBLE")?)?),
            "gui_kind" => Value::str_rc(self.gui.kind_name(gi(a,0,"GUI_KIND")?)?),
            "gui_focus" => { self.gui.focus(gi(a,0,"GUI_FOCUS")?)?; Value::Nil }
            "gui_focused" => Value::Int(self.gui.focused()),
            "gui_colorpicker" => Value::Int(self.gui.colorpicker(
                gi(a,0,"GUI_COLORPICKER")?, gi(a,1,"GUI_COLORPICKER")? as i32,
                gi(a,2,"GUI_COLORPICKER")? as i32, gi(a,3,"GUI_COLORPICKER")? as i32,
                gi(a,4,"GUI_COLORPICKER")? as i32)?),
            "gui_picked_color" => Value::Int(self.gui.picked_color(gi(a,0,"GUI_PICKED_COLOR")?)?),
            "gui_set_picked_color" => {
                self.gui.set_picked_color(gi(a,0,"GUI_SET_PICKED_COLOR")?,
                                          gi(a,1,"GUI_SET_PICKED_COLOR")?)?;
                Value::Nil
            }
            "gui_datepicker" => Value::Int(self.gui.datepicker(
                gi(a,0,"GUI_DATEPICKER")?, gi(a,1,"GUI_DATEPICKER")? as i32,
                gi(a,2,"GUI_DATEPICKER")? as i32, gi(a,3,"GUI_DATEPICKER")? as i32,
                gi(a,4,"GUI_DATEPICKER")? as i32)?),
            "gui_date" => Value::str_rc(&self.gui.date(gi(a,0,"GUI_DATE")?)?),
            "gui_colorpicker_set" => {
                self.gui.colorpicker_set(gi(a,0,"GUI_COLORPICKER_SET")?,
                                         &gs(a,1,"GUI_COLORPICKER_SET")?,
                                         gnum(a,2,"GUI_COLORPICKER_SET")?)?;
                Value::Nil
            }
            "gui_datepicker_set" => {
                self.gui.datepicker_set(gi(a,0,"GUI_DATEPICKER_SET")?,
                                        &gs(a,1,"GUI_DATEPICKER_SET")?,
                                        gnum(a,2,"GUI_DATEPICKER_SET")?)?;
                Value::Nil
            }
            "gui_date_range" => {
                self.gui.date_range(gi(a,0,"GUI_DATE_RANGE")?,
                                    &gs(a,1,"GUI_DATE_RANGE")?,
                                    &gs(a,2,"GUI_DATE_RANGE")?)?;
                Value::Nil
            }
            "gui_set_date" => {
                self.gui.set_date(gi(a,0,"GUI_SET_DATE")?, &gs(a,1,"GUI_SET_DATE")?)?;
                Value::Nil
            }
            "gui_textarea_background" => {
                self.gui.textarea_background(gi(a, 0, "GUI_TEXTAREA_BACKGROUND")?,
                    gi(a, 1, "GUI_TEXTAREA_BACKGROUND")?,
                    if a.len() > 2 { gi(a, 2, "GUI_TEXTAREA_BACKGROUND")? } else { 24 })?;
                Value::Nil
            }
            "gui_textarea_set" => {
                self.gui.textarea_set(gi(a, 0, "GUI_TEXTAREA_SET")?,
                                      &gs(a, 1, "GUI_TEXTAREA_SET")?,
                                      gnum(a, 2, "GUI_TEXTAREA_SET")?)?;
                Value::Nil
            }
            "gui_textarea_spans" => {
                fn ganze(v: &Value, fn_: &str) -> R<Vec<i64>> {
                    match v {
                        Value::Array(a) => {
                            let a = a.borrow();
                            let mut o = Vec::with_capacity(a.cells.len());
                            for x in a.cells.iter() {
                                match x {
                                    Value::Int(i) => o.push(i),
                                    Value::Float(f) => o.push(f as i64),
                                    _ => return Err(format!("{}: ARRAY OF INTEGER noetig", fn_)),
                                }
                            }
                            Ok(o)
                        }
                        _ => Err(format!("{}: ARRAY OF INTEGER noetig", fn_)),
                    }
                }
                let n = "GUI_TEXTAREA_SPANS";
                self.gui.textarea_spans(gi(a, 0, n)?, ganze(&a[1], n)?,
                                        ganze(&a[2], n)?, ganze(&a[3], n)?)?;
                Value::Nil
            }
            "gui_textarea_squiggles" => {
                fn ganze(v: &Value, fn_: &str) -> R<Vec<i64>> {
                    match v {
                        Value::Array(a) => {
                            let a = a.borrow();
                            let mut o = Vec::with_capacity(a.cells.len());
                            for x in a.cells.iter() {
                                match x {
                                    Value::Int(i) => o.push(i),
                                    Value::Float(f) => o.push(f as i64),
                                    _ => return Err(format!("{}: ARRAY OF INTEGER noetig", fn_)),
                                }
                            }
                            Ok(o)
                        }
                        _ => Err(format!("{}: ARRAY OF INTEGER noetig", fn_)),
                    }
                }
                let n = "GUI_TEXTAREA_SQUIGGLES";
                self.gui.textarea_squiggles(gi(a, 0, n)?, ganze(&a[1], n)?,
                                            ganze(&a[2], n)?, ganze(&a[3], n)?)?;
                Value::Nil
            }
            "gui_set_enabled" => { self.gui.set_enabled(gi(a,0,"GUI_SET_ENABLED")?, gbool(a,1,"GUI_SET_ENABLED")?)?; Value::Nil }
            "gui_enabled" => Value::Bool(self.gui.enabled(gi(a,0,"GUI_ENABLED")?)?),
            "gui_set_font" => { self.gui.set_font(gi(a,0,"GUI_SET_FONT")?, gi(a,1,"GUI_SET_FONT")?)?; Value::Nil }
            "gui_set_font_style" => { self.gui.set_font_style(gi(a,0,"GUI_SET_FONT_STYLE")?, &gs(a,1,"GUI_SET_FONT_STYLE")?)?; Value::Nil }
            "gui_get_font_style$" | "gui_get_font_style" => Value::Str(self.gui.font_style(gi(a,0,"GUI_GET_FONT_STYLE$")?)?.into()),
            "gui_set_font_size" => { self.gui.set_font_size(gi(a,0,"GUI_SET_FONT_SIZE")?, gi(a,1,"GUI_SET_FONT_SIZE")?)?; Value::Nil }
            "gui_set_anchor" => { self.gui.set_anchor(gi(a,0,"GUI_SET_ANCHOR")?, &gs(a,1,"GUI_SET_ANCHOR")?)?; Value::Nil }
            "gui_style_set" => { self.gui.style_set(gs(a,0,"GUI_STYLE_SET")?, gs(a,1,"GUI_STYLE_SET")?, gi(a,2,"GUI_STYLE_SET")?)?; Value::Nil }
            "gui_apply_style" => { self.gui.apply_style(gi(a,0,"GUI_APPLY_STYLE")?, &gs(a,1,"GUI_APPLY_STYLE")?)?; Value::Nil }
            "gui_hit_test" => Value::Int(self.gui.hit_test(gi(a,0,"GUI_HIT_TEST")? as i32, gi(a,1,"GUI_HIT_TEST")? as i32)),
            "gui_window_set_bounds" => { self.gui.window_set_bounds(gi(a,0,"GUI_WINDOW_SET_BOUNDS")?, gi(a,1,"GUI_WINDOW_SET_BOUNDS")? as i32, gi(a,2,"GUI_WINDOW_SET_BOUNDS")? as i32, gi(a,3,"GUI_WINDOW_SET_BOUNDS")? as i32, gi(a,4,"GUI_WINDOW_SET_BOUNDS")? as i32)?; Value::Nil }
            "gui_window_get_x" => Value::Int(self.gui.window_bounds(gi(a,0,"GUI_WINDOW_GET_X")?)?.0 as i64),
            "gui_window_get_y" => Value::Int(self.gui.window_bounds(gi(a,0,"GUI_WINDOW_GET_Y")?)?.1 as i64),
            "gui_window_get_w" => Value::Int(self.gui.window_bounds(gi(a,0,"GUI_WINDOW_GET_W")?)?.2 as i64),
            "gui_window_get_h" => Value::Int(self.gui.window_bounds(gi(a,0,"GUI_WINDOW_GET_H")?)?.3 as i64),
            "gui_window_content_w" => Value::Int(self.gui.window_inhalt(gi(a,0,"GUI_WINDOW_CONTENT_W")?)?.0 as i64),
            "gui_window_content_h" => Value::Int(self.gui.window_inhalt(gi(a,0,"GUI_WINDOW_CONTENT_H")?)?.1 as i64),
            "gui_window_destroy" => { self.gui.window_destroy(gi(a,0,"GUI_WINDOW_DESTROY")?)?; Value::Nil }
            "gui_window_widget_count" => Value::Int(self.gui.window_widget_count(gi(a,0,"GUI_WINDOW_WIDGET_COUNT")?)?),
            "gui_window_widget" => Value::Int(self.gui.window_widget(gi(a,0,"GUI_WINDOW_WIDGET")?, gi(a,1,"GUI_WINDOW_WIDGET")?)?),
            // --- Serialisierung (Layout als JSON) ---
            "gui_to_json" => Value::str_rc(&self.gui.to_json(gi(a,0,"GUI_TO_JSON")?)?),
            "gui_from_json" => Value::Int(self.gui.from_json(&gs(a,0,"GUI_FROM_JSON")?)?),
            "gui_save" => {
                let s = self.gui.to_json(gi(a,0,"GUI_SAVE")?)?;
                std::fs::write(gs(a,1,"GUI_SAVE")?, s).map_err(|e| format!("GUI_SAVE: {}", e))?;
                Value::Nil
            }
            "gui_load" => {
                let s = std::fs::read_to_string(gs(a,0,"GUI_LOAD")?).map_err(|e| format!("GUI_LOAD: {}", e))?;
                Value::Int(self.gui.from_json(&s)?)
            }
            // --- Tabelle ---
            "gui_table" => {
                let h = self.gui.table(gi(a,0,"GUI_TABLE")?, gi(a,1,"GUI_TABLE")? as i32, gi(a,2,"GUI_TABLE")? as i32,
                    gi(a,3,"GUI_TABLE")? as i32, gi(a,4,"GUI_TABLE")? as i32)?;
                if a.len() == 7 {
                    self.gui.table_set_headers(h, gstrs(a,5,"GUI_TABLE")?)?;
                    self.gui.table_set_rows(h, gstrs2(a,6,"GUI_TABLE")?)?;
                } else if a.len() == 6 {
                    return Err("GUI_TABLE: entweder ohne Daten oder mit headers UND cells aufrufen".into());
                }
                Value::Int(h)
            }
            // --- Stufe 30: Gitter ---
            "gui_grid" => {
                let n = if a.len() > 6 { gi(a,6,"GUI_GRID")? } else { 0 };
                Value::Int(self.gui.grid(gi(a,0,"GUI_GRID")?, gi(a,1,"GUI_GRID")? as i32, gi(a,2,"GUI_GRID")? as i32,
                    gi(a,3,"GUI_GRID")? as i32, gi(a,4,"GUI_GRID")? as i32, gstrs(a,5,"GUI_GRID")?, n)?)
            }
            "gui_table_current_row" => Value::Int(self.gui.table_current(gi(a,0,"GUI_TABLE_CURRENT_ROW")?, false)?),
            "gui_table_current_col" => Value::Int(self.gui.table_current(gi(a,0,"GUI_TABLE_CURRENT_COL")?, true)?),
            "gui_table_set_current" => { self.gui.table_set_current(gi(a,0,"GUI_TABLE_SET_CURRENT")?, gi(a,1,"GUI_TABLE_SET_CURRENT")?, gi(a,2,"GUI_TABLE_SET_CURRENT")?)?; Value::Nil }
            "gui_table_range" => {
                let (ar, ac, r, c) = self.gui.table_range(gi(a,0,"GUI_TABLE_RANGE")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(ar), Value::Int(ac), Value::Int(r), Value::Int(c)]))
            }
            "gui_table_select_range" => { self.gui.table_select_range(gi(a,0,"GUI_TABLE_SELECT_RANGE")?, gi(a,1,"GUI_TABLE_SELECT_RANGE")?, gi(a,2,"GUI_TABLE_SELECT_RANGE")?, gi(a,3,"GUI_TABLE_SELECT_RANGE")?, gi(a,4,"GUI_TABLE_SELECT_RANGE")?)?; Value::Nil }
            "gui_table_copy$" | "gui_table_copy" => Value::str_rc(&self.gui.table_copy(gi(a,0,"GUI_TABLE_COPY$")?)?),
            "gui_table_paste" => Value::Int(self.gui.table_paste(gi(a,0,"GUI_TABLE_PASTE")?, &gs(a,1,"GUI_TABLE_PASTE")?)?),
            "gui_table_col_type" => { self.gui.table_col_type(gi(a,0,"GUI_TABLE_COL_TYPE")?, gi(a,1,"GUI_TABLE_COL_TYPE")?, &gs(a,2,"GUI_TABLE_COL_TYPE")?)?; Value::Nil }
            "gui_table_col_choices" => { self.gui.table_col_choices(gi(a,0,"GUI_TABLE_COL_CHOICES")?, gi(a,1,"GUI_TABLE_COL_CHOICES")?, gstrs(a,2,"GUI_TABLE_COL_CHOICES")?)?; Value::Nil }
            "gui_table_headers" => { self.gui.table_set_headers(gi(a,0,"GUI_TABLE_HEADERS")?, gstrs(a,1,"GUI_TABLE_HEADERS")?)?; Value::Nil }
            "gui_table_rows" => { self.gui.table_set_rows(gi(a,0,"GUI_TABLE_ROWS")?, gstrs2(a,1,"GUI_TABLE_ROWS")?)?; Value::Nil }
            "gui_table_col_widths" => { self.gui.table_set_col_widths(gi(a,0,"GUI_TABLE_COL_WIDTHS")?, gints_opt(a,1,"GUI_TABLE_COL_WIDTHS")?)?; Value::Nil }
            "gui_table_selected" => Value::Int(self.gui.table_selected(gi(a,0,"GUI_TABLE_SELECTED")?)?),
            "gui_table_set_selected" => { self.gui.table_set_selected(gi(a,0,"GUI_TABLE_SET_SELECTED")?, gi(a,1,"GUI_TABLE_SET_SELECTED")?)?; Value::Nil }
            "gui_table_clicked" => Value::Int(self.gui.table_clicked(gi(a,0,"GUI_TABLE_CLICKED")?)?),
            "gui_table_clicked_col" => Value::Int(self.gui.table_clicked_col(gi(a,0,"GUI_TABLE_CLICKED_COL")?)?),
            // --- Zellen einzeln ---
            "gui_table_set_cell" => { self.gui.table_set_cell(gi(a,0,"GUI_TABLE_SET_CELL")?, gi(a,1,"GUI_TABLE_SET_CELL")?, gi(a,2,"GUI_TABLE_SET_CELL")?, gs(a,3,"GUI_TABLE_SET_CELL")?.to_string())?; Value::Nil }
            "gui_table_get_cell" => Value::str_rc(&self.gui.table_get_cell(gi(a,0,"GUI_TABLE_GET_CELL")?, gi(a,1,"GUI_TABLE_GET_CELL")?, gi(a,2,"GUI_TABLE_GET_CELL")?)?),
            "gui_table_cell_color" => { self.gui.table_cell_color(gi(a,0,"GUI_TABLE_CELL_COLOR")?, gi(a,1,"GUI_TABLE_CELL_COLOR")?, gi(a,2,"GUI_TABLE_CELL_COLOR")?, gi(a,3,"GUI_TABLE_CELL_COLOR")?, gi(a,4,"GUI_TABLE_CELL_COLOR")?)?; Value::Nil }
            "gui_table_cell_align" => { self.gui.table_cell_align(gi(a,0,"GUI_TABLE_CELL_ALIGN")?, gi(a,1,"GUI_TABLE_CELL_ALIGN")?, gi(a,2,"GUI_TABLE_CELL_ALIGN")?, &gs(a,3,"GUI_TABLE_CELL_ALIGN")?)?; Value::Nil }
            "gui_table_cell_kind" => { self.gui.table_cell_kind(gi(a,0,"GUI_TABLE_CELL_KIND")?, gi(a,1,"GUI_TABLE_CELL_KIND")?, gi(a,2,"GUI_TABLE_CELL_KIND")?, &gs(a,3,"GUI_TABLE_CELL_KIND")?)?; Value::Nil }
            "gui_table_cell_image" => { self.gui.table_cell_image(gi(a,0,"GUI_TABLE_CELL_IMAGE")?, gi(a,1,"GUI_TABLE_CELL_IMAGE")?, gi(a,2,"GUI_TABLE_CELL_IMAGE")?, gi(a,3,"GUI_TABLE_CELL_IMAGE")?)?; Value::Nil }
            "gui_table_cell_value" => { self.gui.table_cell_value(gi(a,0,"GUI_TABLE_CELL_VALUE")?, gi(a,1,"GUI_TABLE_CELL_VALUE")?, gi(a,2,"GUI_TABLE_CELL_VALUE")?, gnum(a,3,"GUI_TABLE_CELL_VALUE")?)?; Value::Nil }
            "gui_table_get_value" => Value::Float(self.gui.table_get_value(gi(a,0,"GUI_TABLE_GET_VALUE")?, gi(a,1,"GUI_TABLE_GET_VALUE")?, gi(a,2,"GUI_TABLE_GET_VALUE")?)?),
            // --- Zeilen / Spalten ---
            "gui_table_row_color" => { self.gui.table_row_color(gi(a,0,"GUI_TABLE_ROW_COLOR")?, gi(a,1,"GUI_TABLE_ROW_COLOR")?, gi(a,2,"GUI_TABLE_ROW_COLOR")?, gi(a,3,"GUI_TABLE_ROW_COLOR")?)?; Value::Nil }
            "gui_table_col_align" => { self.gui.table_col_align(gi(a,0,"GUI_TABLE_COL_ALIGN")?, gi(a,1,"GUI_TABLE_COL_ALIGN")?, &gs(a,2,"GUI_TABLE_COL_ALIGN")?)?; Value::Nil }
            "gui_table_add_row" => Value::Int(self.gui.table_add_row(gi(a,0,"GUI_TABLE_ADD_ROW")?, gstrs(a,1,"GUI_TABLE_ADD_ROW")?)?),
            "gui_table_remove_row" => { self.gui.table_remove_row(gi(a,0,"GUI_TABLE_REMOVE_ROW")?, gi(a,1,"GUI_TABLE_REMOVE_ROW")?)?; Value::Nil }
            "gui_table_clear" => { self.gui.table_clear(gi(a,0,"GUI_TABLE_CLEAR")?)?; Value::Nil }
            // --- tabellenweite Einstellungen ---
            "gui_table_set" => { self.gui.table_set_opt(gi(a,0,"GUI_TABLE_SET")?, &gs(a,1,"GUI_TABLE_SET")?, gnum(a,2,"GUI_TABLE_SET")?)?; Value::Nil }
            "gui_table_get" => Value::Float(self.gui.table_get_opt(gi(a,0,"GUI_TABLE_GET")?, &gs(a,1,"GUI_TABLE_GET")?)?),
            // --- Spalten-Reihenfolge ---
            "gui_table_move_col" => { self.gui.table_move_col(gi(a,0,"GUI_TABLE_MOVE_COL")?, gi(a,1,"GUI_TABLE_MOVE_COL")?, gi(a,2,"GUI_TABLE_MOVE_COL")?)?; Value::Nil }
            "gui_table_col_at" => Value::Int(self.gui.table_col_at(gi(a,0,"GUI_TABLE_COL_AT")?, gi(a,1,"GUI_TABLE_COL_AT")?)?),
            "gui_table_col_pos" => Value::Int(self.gui.table_col_pos(gi(a,0,"GUI_TABLE_COL_POS")?, gi(a,1,"GUI_TABLE_COL_POS")?)?),
            "gui_table_reset_cols" => { self.gui.table_reset_cols(gi(a,0,"GUI_TABLE_RESET_COLS")?)?; Value::Nil }
            // --- Mehrfachauswahl ---
            "gui_table_sel_count" => Value::Int(self.gui.table_sel_count(gi(a,0,"GUI_TABLE_SEL_COUNT")?)?),
            "gui_table_sel_row" => Value::Int(self.gui.table_sel_row(gi(a,0,"GUI_TABLE_SEL_ROW")?, gi(a,1,"GUI_TABLE_SEL_ROW")?)?),
            "gui_table_is_selected" => Value::Bool(self.gui.table_is_selected(gi(a,0,"GUI_TABLE_IS_SELECTED")?, gi(a,1,"GUI_TABLE_IS_SELECTED")?)?),
            "gui_table_select" => { self.gui.table_select(gi(a,0,"GUI_TABLE_SELECT")?, gi(a,1,"GUI_TABLE_SELECT")?, gbool(a,2,"GUI_TABLE_SELECT")?)?; Value::Nil }
            "gui_table_clear_selection" => { self.gui.table_clear_selection(gi(a,0,"GUI_TABLE_CLEAR_SELECTION")?)?; Value::Nil }
            // --- Zellen bearbeiten ---
            "gui_table_col_edit" => { self.gui.table_col_edit(gi(a,0,"GUI_TABLE_COL_EDIT")?, gi(a,1,"GUI_TABLE_COL_EDIT")?, gbool(a,2,"GUI_TABLE_COL_EDIT")?)?; Value::Nil }
            "gui_table_editing_row" => Value::Int(self.gui.table_editing_row(gi(a,0,"GUI_TABLE_EDITING_ROW")?)?),
            "gui_table_editing_col" => Value::Int(self.gui.table_editing_col(gi(a,0,"GUI_TABLE_EDITING_COL")?)?),
            // --- Sortieren / Filtern ---
            "gui_table_sort" => { self.gui.table_sort(gi(a,0,"GUI_TABLE_SORT")?, gi(a,1,"GUI_TABLE_SORT")?, gbool(a,2,"GUI_TABLE_SORT")?)?; Value::Nil }
            "gui_table_sort_col" => Value::Int(self.gui.table_sort_col(gi(a,0,"GUI_TABLE_SORT_COL")?)?),
            "gui_table_sort_desc" => Value::Bool(self.gui.table_sort_desc(gi(a,0,"GUI_TABLE_SORT_DESC")?)?),
            "gui_table_placeholder" => { self.gui.table_placeholder(gi(a,0,"GUI_TABLE_PLACEHOLDER")?, gs(a,1,"GUI_TABLE_PLACEHOLDER")?.to_string())?; Value::Nil }
            "gui_table_filter" => { self.gui.table_filter(gi(a,0,"GUI_TABLE_FILTER")?, gi(a,1,"GUI_TABLE_FILTER")?, gs(a,2,"GUI_TABLE_FILTER")?.to_string())?; Value::Nil }
            "gui_table_get_filter" => Value::str_rc(&self.gui.table_get_filter(gi(a,0,"GUI_TABLE_GET_FILTER")?, gi(a,1,"GUI_TABLE_GET_FILTER")?)?),
            "gui_table_view_count" => Value::Int(self.gui.table_view_count(gi(a,0,"GUI_TABLE_VIEW_COUNT")?)?),
            "gui_table_view_row" => Value::Int(self.gui.table_view_row(gi(a,0,"GUI_TABLE_VIEW_ROW")?, gi(a,1,"GUI_TABLE_VIEW_ROW")?)?),
            "gui_table_row_count" => Value::Int(self.gui.table_row_count(gi(a,0,"GUI_TABLE_ROW_COUNT")?)?),
            "gui_tree" => Value::Int(self.gui.tree(gi(a,0,"GUI_TREE")?, gi(a,1,"GUI_TREE")? as i32,
                gi(a,2,"GUI_TREE")? as i32, gi(a,3,"GUI_TREE")? as i32, gi(a,4,"GUI_TREE")? as i32)?),
            "gui_tree_add" => Value::Int(self.gui.tree_add(gi(a,0,"GUI_TREE_ADD")?, gi(a,1,"GUI_TREE_ADD")?, gs(a,2,"GUI_TREE_ADD")?)?),
            "gui_tree_clear" => { self.gui.tree_clear(gi(a,0,"GUI_TREE_CLEAR")?)?; Value::Nil }
            "gui_tree_selected" => Value::Int(self.gui.tree_selected(gi(a,0,"GUI_TREE_SELECTED")?)?),
            "gui_tree_set_selected" => { self.gui.tree_set_selected(gi(a,0,"GUI_TREE_SET_SELECTED")?, gi(a,1,"GUI_TREE_SET_SELECTED")?)?; Value::Nil }
            "gui_tree_label" => Value::str_rc(&self.gui.tree_label(gi(a,0,"GUI_TREE_LABEL")?, gi(a,1,"GUI_TREE_LABEL")?)?),
            "gui_tree_set" => { self.gui.tree_set(gi(a,0,"GUI_TREE_SET")?, &gs(a,1,"GUI_TREE_SET")?, gnum(a,2,"GUI_TREE_SET")?)?; Value::Nil }
            "gui_tree_sel_count" => Value::Int(self.gui.tree_sel_count(gi(a,0,"GUI_TREE_SEL_COUNT")?)?),
            "gui_tree_sel_node" => Value::Int(self.gui.tree_sel_node(gi(a,0,"GUI_TREE_SEL_NODE")?, gi(a,1,"GUI_TREE_SEL_NODE")?)?),
            "gui_tree_is_selected" => Value::Bool(self.gui.tree_is_selected(gi(a,0,"GUI_TREE_IS_SELECTED")?, gi(a,1,"GUI_TREE_IS_SELECTED")?)?),
            "gui_tree_select" => { self.gui.tree_select(gi(a,0,"GUI_TREE_SELECT")?, gi(a,1,"GUI_TREE_SELECT")?, gbool(a,2,"GUI_TREE_SELECT")?)?; Value::Nil }
            "gui_tree_clear_selection" => { self.gui.tree_clear_selection(gi(a,0,"GUI_TREE_CLEAR_SELECTION")?)?; Value::Nil }
            "gui_tree_expand" => { self.gui.tree_expand(gi(a,0,"GUI_TREE_EXPAND")?, gi(a,1,"GUI_TREE_EXPAND")?, gbool(a,2,"GUI_TREE_EXPAND")?)?; Value::Nil }
            "gui_tree_checked" => Value::Bool(self.gui.tree_checked(gi(a,0,"GUI_TREE_CHECKED")?, gi(a,1,"GUI_TREE_CHECKED")?)?),
            "gui_tree_set_checked" => { self.gui.tree_set_checked(gi(a,0,"GUI_TREE_SET_CHECKED")?, gi(a,1,"GUI_TREE_SET_CHECKED")?, gbool(a,2,"GUI_TREE_SET_CHECKED")?)?; Value::Nil }
            // --- Dateibaum ---
            "gui_filetree" => Value::Int(self.gui.filetree(gi(a,0,"GUI_FILETREE")?, gi(a,1,"GUI_FILETREE")? as i32,
                gi(a,2,"GUI_FILETREE")? as i32, gi(a,3,"GUI_FILETREE")? as i32, gi(a,4,"GUI_FILETREE")? as i32,
                if a.len() > 5 { gs(a,5,"GUI_FILETREE")? } else { ".".to_string() })?),
            "gui_filetree_set" => { self.gui.filetree_set(gi(a,0,"GUI_FILETREE_SET")?, &gs(a,1,"GUI_FILETREE_SET")?, gnum(a,2,"GUI_FILETREE_SET")?)?; Value::Nil }
            "gui_filetree_filter" => { self.gui.filetree_filter(gi(a,0,"GUI_FILETREE_FILTER")?, &gs(a,1,"GUI_FILETREE_FILTER")?)?; Value::Nil }
            "gui_filetree_skip" => { self.gui.filetree_skip(gi(a,0,"GUI_FILETREE_SKIP")?, &gs(a,1,"GUI_FILETREE_SKIP")?)?; Value::Nil }
            "gui_filetree_set_root" => { self.gui.filetree_set_root(gi(a,0,"GUI_FILETREE_SET_ROOT")?, gs(a,1,"GUI_FILETREE_SET_ROOT")?)?; Value::Nil }
            "gui_filetree_root$" | "gui_filetree_root" => Value::str_rc(&self.gui.filetree_root(gi(a,0,"GUI_FILETREE_ROOT")?)?),
            "gui_filetree_refresh" => { self.gui.filetree_refresh(gi(a,0,"GUI_FILETREE_REFRESH")?)?; Value::Nil }
            "gui_filetree_selected$" | "gui_filetree_selected" => Value::str_rc(&self.gui.filetree_selected(gi(a,0,"GUI_FILETREE_SELECTED")?)?),
            "gui_filetree_activated$" | "gui_filetree_activated" => Value::str_rc(&self.gui.filetree_activated(gi(a,0,"GUI_FILETREE_ACTIVATED")?)?),
            "gui_filetree_is_dir" => Value::Bool(self.gui.filetree_is_dir(gi(a,0,"GUI_FILETREE_IS_DIR")?,
                &if a.len() > 1 { gs(a,1,"GUI_FILETREE_IS_DIR")? } else { String::new() })?),
            "gui_filetree_select" => { self.gui.filetree_select(gi(a,0,"GUI_FILETREE_SELECT")?, &gs(a,1,"GUI_FILETREE_SELECT")?)?; Value::Nil }
            "gui_filetree_expand" => { self.gui.filetree_expand(gi(a,0,"GUI_FILETREE_EXPAND")?, &gs(a,1,"GUI_FILETREE_EXPAND")?, gbool(a,2,"GUI_FILETREE_EXPAND")?)?; Value::Nil }
            "gui_filetree_count" => Value::Int(self.gui.filetree_count(gi(a,0,"GUI_FILETREE_COUNT")?)?),
            "gui_filetree_path$" | "gui_filetree_path" => Value::str_rc(&self.gui.filetree_path(gi(a,0,"GUI_FILETREE_PATH")?, gi(a,1,"GUI_FILETREE_PATH")?)?),
            "gui_filetree_sel_count" => Value::Int(self.gui.filetree_sel_count(gi(a,0,"GUI_FILETREE_SEL_COUNT")?)?),
            "gui_filetree_sel_path$" | "gui_filetree_sel_path" => Value::str_rc(&self.gui.filetree_sel_path(gi(a,0,"GUI_FILETREE_SEL_PATH")?, gi(a,1,"GUI_FILETREE_SEL_PATH")?)?),
            "gui_filetree_checked" => Value::Bool(self.gui.filetree_checked(gi(a,0,"GUI_FILETREE_CHECKED")?, &gs(a,1,"GUI_FILETREE_CHECKED")?)?),
            "gui_filetree_set_checked" => { self.gui.filetree_set_checked(gi(a,0,"GUI_FILETREE_SET_CHECKED")?, &gs(a,1,"GUI_FILETREE_SET_CHECKED")?, gbool(a,2,"GUI_FILETREE_SET_CHECKED")?)?; Value::Nil }
            "gui_filetree_checked_count" => Value::Int(self.gui.filetree_checked_count(gi(a,0,"GUI_FILETREE_CHECKED_COUNT")?)?),
            "gui_filetree_checked_path$" | "gui_filetree_checked_path" => Value::str_rc(&self.gui.filetree_checked_path(gi(a,0,"GUI_FILETREE_CHECKED_PATH")?, gi(a,1,"GUI_FILETREE_CHECKED_PATH")?)?),
            // --- Uhrzeit ---
            "gui_timepicker" => Value::Int(self.gui.timepicker(gi(a,0,"GUI_TIMEPICKER")?, gi(a,1,"GUI_TIMEPICKER")? as i32,
                gi(a,2,"GUI_TIMEPICKER")? as i32,
                if a.len() > 3 { gi(a,3,"GUI_TIMEPICKER")? as i32 } else { 0 },
                if a.len() > 4 { gi(a,4,"GUI_TIMEPICKER")? as i32 } else { 0 })?),
            "gui_time$" | "gui_time" => Value::str_rc(&self.gui.time_get(gi(a,0,"GUI_TIME")?)?),
            "gui_set_time" => { self.gui.set_time(gi(a,0,"GUI_SET_TIME")?, &gs(a,1,"GUI_SET_TIME")?)?; Value::Nil }
            "gui_timepicker_set" => { self.gui.timepicker_set(gi(a,0,"GUI_TIMEPICKER_SET")?, &gs(a,1,"GUI_TIMEPICKER_SET")?, gnum(a,2,"GUI_TIMEPICKER_SET")?)?; Value::Nil }
            // --- Gesetzter Text ---
            "gui_richtext" => Value::Int(self.gui.richtext(gi(a,0,"GUI_RICHTEXT")?, gi(a,1,"GUI_RICHTEXT")? as i32,
                gi(a,2,"GUI_RICHTEXT")? as i32, gi(a,3,"GUI_RICHTEXT")? as i32, gi(a,4,"GUI_RICHTEXT")? as i32,
                if a.len() > 5 { gs(a,5,"GUI_RICHTEXT")? } else { String::new() })?),
            "gui_richtext_set_text" => { self.gui.richtext_set_text(gi(a,0,"GUI_RICHTEXT_SET_TEXT")?, gs(a,1,"GUI_RICHTEXT_SET_TEXT")?)?; Value::Nil }
            "gui_richtext_set" => { self.gui.richtext_set(gi(a,0,"GUI_RICHTEXT_SET")?, &gs(a,1,"GUI_RICHTEXT_SET")?, gnum(a,2,"GUI_RICHTEXT_SET")?)?; Value::Nil }
            "gui_richtext_headings" => {
                let ue = self.gui.richtext_headings(gi(a,0,"GUI_RICHTEXT_HEADINGS")?)?;
                let mut arr = crate::value::DhArray::new("string".to_string(), vec![ue.len() as i64], || Value::str_rc(""));
                for (i, (_, t)) in ue.into_iter().enumerate() { arr.cells.set(i, Value::str_rc(t)); }
                Value::Array(std::rc::Rc::new(std::cell::RefCell::new(arr)))
            }
            "gui_richtext_heading_levels" => {
                let ue = self.gui.richtext_headings(gi(a,0,"GUI_RICHTEXT_HEADING_LEVELS")?)?;
                let mut arr = crate::value::DhArray::new("integer".to_string(), vec![ue.len() as i64], || Value::Int(0));
                for (i, (n, _)) in ue.into_iter().enumerate() { arr.cells.set(i, Value::Int(n as i64)); }
                Value::Array(std::rc::Rc::new(std::cell::RefCell::new(arr)))
            }
            "gui_richtext_code_buttons" => { self.gui.richtext_code_buttons(gi(a,0,"GUI_RICHTEXT_CODE_BUTTONS")?, &gs(a,1,"GUI_RICHTEXT_CODE_BUTTONS")?)?; Value::Nil }
            "gui_richtext_code_action$" | "gui_richtext_code_action" => Value::str_rc(self.gui.richtext_code_action(gi(a,0,"GUI_RICHTEXT_CODE_ACTION$")?)?),
            "gui_richtext_code$" | "gui_richtext_code" => Value::str_rc(self.gui.richtext_code(gi(a,0,"GUI_RICHTEXT_CODE$")?)?),
            "gui_richtext_goto_heading" => { self.gui.richtext_goto_heading(gi(a,0,"GUI_RICHTEXT_GOTO_HEADING")?, gi(a,1,"GUI_RICHTEXT_GOTO_HEADING")?)?; Value::Nil }
            "gui_richtext_heading_at" => Value::Int(self.gui.richtext_heading_at(gi(a,0,"GUI_RICHTEXT_HEADING_AT")?)?),
            "gui_richtext_scroll" => { self.gui.richtext_scroll(gi(a,0,"GUI_RICHTEXT_SCROLL")?, gi(a,1,"GUI_RICHTEXT_SCROLL")?)?; Value::Nil }
            "gui_richtext_scroll_get" => Value::Int(self.gui.richtext_scroll_get(gi(a,0,"GUI_RICHTEXT_SCROLL_GET")?)?),
            "gui_richtext_height" => Value::Int(self.gui.richtext_height(gi(a,0,"GUI_RICHTEXT_HEIGHT")?)?),
            "gui_richtext_link$" | "gui_richtext_link" => Value::str_rc(&self.gui.richtext_link(gi(a,0,"GUI_RICHTEXT_LINK")?)?),
            "gui_richtext_find" => Value::Int(self.gui.richtext_find(gi(a,0,"GUI_RICHTEXT_FIND")?, &gs(a,1,"GUI_RICHTEXT_FIND")?,
                if a.len() > 2 { gi(a,2,"GUI_RICHTEXT_FIND")? } else { -1 })?),
            // --- Reiter im Fenster ---
            "gui_tabcontrol" => Value::Int(self.gui.tabcontrol(gi(a,0,"GUI_TABCONTROL")?, gi(a,1,"GUI_TABCONTROL")? as i32,
                gi(a,2,"GUI_TABCONTROL")? as i32, gi(a,3,"GUI_TABCONTROL")? as i32, gi(a,4,"GUI_TABCONTROL")? as i32)?),
            "gui_tabcontrol_add" => Value::Int(self.gui.tabcontrol_add(gi(a,0,"GUI_TABCONTROL_ADD")?, gs(a,1,"GUI_TABCONTROL_ADD")?)?),
            "gui_tabcontrol_count" => Value::Int(self.gui.tabcontrol_count(gi(a,0,"GUI_TABCONTROL_COUNT")?)?),
            "gui_tabcontrol_page" => Value::Int(self.gui.tabcontrol_page(gi(a,0,"GUI_TABCONTROL_PAGE")?)?),
            "gui_tabcontrol_set_page" => { self.gui.tabcontrol_set_page(gi(a,0,"GUI_TABCONTROL_SET_PAGE")?, gi(a,1,"GUI_TABCONTROL_SET_PAGE")?)?; Value::Nil }
            "gui_tabcontrol_title$" | "gui_tabcontrol_title" => Value::str_rc(&self.gui.tabcontrol_title(gi(a,0,"GUI_TABCONTROL_TITLE")?, gi(a,1,"GUI_TABCONTROL_TITLE")?)?),
            "gui_tabcontrol_set_title" => { self.gui.tabcontrol_set_title(gi(a,0,"GUI_TABCONTROL_SET_TITLE")?, gi(a,1,"GUI_TABCONTROL_SET_TITLE")?, gs(a,2,"GUI_TABCONTROL_SET_TITLE")?)?; Value::Nil }
            "gui_tabcontrol_add_widget" => { self.gui.tabcontrol_add_widget(gi(a,0,"GUI_TABCONTROL_ADD_WIDGET")?, gi(a,1,"GUI_TABCONTROL_ADD_WIDGET")?, gi(a,2,"GUI_TABCONTROL_ADD_WIDGET")?)?; Value::Nil }
            "gui_tabcontrol_set" => {
                let schl = match a.get(1) { Some(Value::Str(s)) => s.to_string(), _ => return Err("GUI_TABCONTROL_SET: erwartet (reiterwerk, schluessel$, wert)".into()) };
                let wert = match a.get(2) { Some(Value::Bool(b)) => *b, Some(Value::Int(n)) => *n != 0, _ => return Err("GUI_TABCONTROL_SET: erwartet (reiterwerk, schluessel$, wert)".into()) };
                self.gui.tabcontrol_set(gi(a,0,"GUI_TABCONTROL_SET")?, &schl, wert)?; Value::Nil
            }
            "gui_tabcontrol_closed" => Value::Int(self.gui.tabcontrol_closed(gi(a,0,"GUI_TABCONTROL_CLOSED")?)?),
            "gui_tabcontrol_remove" => { self.gui.tabcontrol_remove(gi(a,0,"GUI_TABCONTROL_REMOVE")?, gi(a,1,"GUI_TABCONTROL_REMOVE")?)?; Value::Nil }
            // --- Akkordeon ---
            "gui_accordion" => Value::Int(self.gui.accordion(gi(a,0,"GUI_ACCORDION")?, gi(a,1,"GUI_ACCORDION")? as i32,
                gi(a,2,"GUI_ACCORDION")? as i32, gi(a,3,"GUI_ACCORDION")? as i32, gi(a,4,"GUI_ACCORDION")? as i32)?),
            "gui_accordion_add" => Value::Int(self.gui.accordion_add(gi(a,0,"GUI_ACCORDION_ADD")?, gs(a,1,"GUI_ACCORDION_ADD")?)?),
            "gui_accordion_add_widget" => { self.gui.accordion_add_widget(gi(a,0,"GUI_ACCORDION_ADD_WIDGET")?, gi(a,1,"GUI_ACCORDION_ADD_WIDGET")?, gi(a,2,"GUI_ACCORDION_ADD_WIDGET")?)?; Value::Nil }
            "gui_accordion_open" => { self.gui.accordion_open(gi(a,0,"GUI_ACCORDION_OPEN")?, gi(a,1,"GUI_ACCORDION_OPEN")?, gbool(a,2,"GUI_ACCORDION_OPEN")?)?; Value::Nil }
            "gui_accordion_is_open" => Value::Bool(self.gui.accordion_is_open(gi(a,0,"GUI_ACCORDION_IS_OPEN")?, gi(a,1,"GUI_ACCORDION_IS_OPEN")?)?),
            "gui_accordion_count" => Value::Int(self.gui.accordion_count(gi(a,0,"GUI_ACCORDION_COUNT")?)?),
            "gui_accordion_title$" | "gui_accordion_title" => Value::str_rc(&self.gui.accordion_title(gi(a,0,"GUI_ACCORDION_TITLE$")?, gi(a,1,"GUI_ACCORDION_TITLE$")?)?),
            "gui_accordion_set_title" => { self.gui.accordion_set_title(gi(a,0,"GUI_ACCORDION_SET_TITLE")?, gi(a,1,"GUI_ACCORDION_SET_TITLE")?, gs(a,2,"GUI_ACCORDION_SET_TITLE")?)?; Value::Nil }
            "gui_accordion_toggled" => Value::Int(self.gui.accordion_toggled(gi(a,0,"GUI_ACCORDION_TOGGLED")?)?),
            "gui_accordion_set" => { self.gui.accordion_set(gi(a,0,"GUI_ACCORDION_SET")?, &gs(a,1,"GUI_ACCORDION_SET")?, gi(a,2,"GUI_ACCORDION_SET")?)?; Value::Nil }
            "gui_accordion_set_height" => { self.gui.accordion_set_height(gi(a,0,"GUI_ACCORDION_SET_HEIGHT")?, gi(a,1,"GUI_ACCORDION_SET_HEIGHT")?, gi(a,2,"GUI_ACCORDION_SET_HEIGHT")?)?; Value::Nil }
            // --- Dock (gui_dock.rs) ---
            "gui_dock" => { self.gui.dock(gi(a,0,"GUI_DOCK")?, gi(a,1,"GUI_DOCK")? as i32, gi(a,2,"GUI_DOCK")? as i32,
                gi(a,3,"GUI_DOCK")? as i32, gi(a,4,"GUI_DOCK")? as i32)?; Value::Nil }
            "gui_dock_set" => { self.gui.dock_set(gi(a,0,"GUI_DOCK_SET")?, &gs(a,1,"GUI_DOCK_SET")?, gi(a,2,"GUI_DOCK_SET")? as i32)?; Value::Nil }
            "gui_dock_panel" => { self.gui.dock_panel(gi(a,0,"GUI_DOCK_PANEL")?, &gs(a,1,"GUI_DOCK_PANEL")?, &gs(a,2,"GUI_DOCK_PANEL")?, &gs(a,3,"GUI_DOCK_PANEL")?)?; Value::Nil }
            "gui_dock_tab" => { self.gui.dock_tab(gi(a,0,"GUI_DOCK_TAB")?, &gs(a,1,"GUI_DOCK_TAB")?, &gs(a,2,"GUI_DOCK_TAB")?, &gs(a,3,"GUI_DOCK_TAB")?)?; Value::Nil }
            "gui_dock_add" => { self.gui.dock_add(gi(a,0,"GUI_DOCK_ADD")?, &gs(a,1,"GUI_DOCK_ADD")?, gi(a,2,"GUI_DOCK_ADD")?)?; Value::Nil }
            "gui_dock_window" => { self.gui.dock_window(gi(a,0,"GUI_DOCK_WINDOW")?, &gs(a,1,"GUI_DOCK_WINDOW")?, gi(a,2,"GUI_DOCK_WINDOW")?)?; Value::Nil }
            "gui_dock_rect" => {
                let (x, y, w, h) = self.gui.dock_rect(gi(a,0,"GUI_DOCK_RECT")?, &gs(a,1,"GUI_DOCK_RECT")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(x as i64), Value::Int(y as i64), Value::Int(w as i64), Value::Int(h as i64)]))
            }
            "gui_dock_tab_rect" => {
                let (x, y, w, h) = self.gui.dock_tab_rect(gi(a,0,"GUI_DOCK_TAB_RECT")?, &gs(a,1,"GUI_DOCK_TAB_RECT")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(x as i64), Value::Int(y as i64), Value::Int(w as i64), Value::Int(h as i64)]))
            }
            "gui_dock_head_rect" => {
                let (x, y, w, h) = self.gui.dock_head_rect(gi(a,0,"GUI_DOCK_HEAD_RECT")?, &gs(a,1,"GUI_DOCK_HEAD_RECT")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(x as i64), Value::Int(y as i64), Value::Int(w as i64), Value::Int(h as i64)]))
            }
            "gui_dock_share" => {
                let f = match a.get(2) { Some(Value::Int(i)) => *i as f64, Some(Value::Float(f)) => *f,
                    _ => return Err("GUI_DOCK_SHARE: erwartet (win, name$, anteil)".into()) };
                self.gui.dock_share(gi(a,0,"GUI_DOCK_SHARE")?, &gs(a,1,"GUI_DOCK_SHARE")?, f)?; Value::Nil
            }
            "gui_dock_center" => {
                let (x, y, w, h) = self.gui.dock_center(gi(a,0,"GUI_DOCK_CENTER")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(x as i64), Value::Int(y as i64), Value::Int(w as i64), Value::Int(h as i64)]))
            }
            "gui_dock_shown" => Value::Bool(self.gui.dock_shown(gi(a,0,"GUI_DOCK_SHOWN")?, &gs(a,1,"GUI_DOCK_SHOWN")?)?),
            "gui_dock_where$" | "gui_dock_where" => Value::str_rc(self.gui.dock_where(gi(a,0,"GUI_DOCK_WHERE$")?, &gs(a,1,"GUI_DOCK_WHERE$")?)?),
            "gui_dock_show" => { self.gui.dock_show(gi(a,0,"GUI_DOCK_SHOW")?, &gs(a,1,"GUI_DOCK_SHOW")?, gbool(a,2,"GUI_DOCK_SHOW")?)?; Value::Nil }
            "gui_dock_activate" => { self.gui.dock_activate(gi(a,0,"GUI_DOCK_ACTIVATE")?, &gs(a,1,"GUI_DOCK_ACTIVATE")?)?; Value::Nil }
            "gui_dock_title" => { self.gui.dock_title(gi(a,0,"GUI_DOCK_TITLE")?, &gs(a,1,"GUI_DOCK_TITLE")?, &gs(a,2,"GUI_DOCK_TITLE")?)?; Value::Nil }
            "gui_dock_closable" => { self.gui.dock_closable(gi(a,0,"GUI_DOCK_CLOSABLE")?, &gs(a,1,"GUI_DOCK_CLOSABLE")?, gbool(a,2,"GUI_DOCK_CLOSABLE")?)?; Value::Nil }
            "gui_dock_closed$" | "gui_dock_closed" => Value::str_rc(self.gui.dock_closed(gi(a,0,"GUI_DOCK_CLOSED$")?)?),
            "gui_dock_changed" => Value::Bool(self.gui.dock_changed(gi(a,0,"GUI_DOCK_CHANGED")?)?),
            "gui_dock_area" => { self.gui.dock_area(gi(a,0,"GUI_DOCK_AREA")?, &gs(a,1,"GUI_DOCK_AREA")?, gbool(a,2,"GUI_DOCK_AREA")?)?; Value::Nil }
            "gui_dock_area_shown" => Value::Bool(self.gui.dock_area_shown(gi(a,0,"GUI_DOCK_AREA_SHOWN")?, &gs(a,1,"GUI_DOCK_AREA_SHOWN")?)?),
            "gui_dock_size" => { self.gui.dock_size(gi(a,0,"GUI_DOCK_SIZE")?, &gs(a,1,"GUI_DOCK_SIZE")?, gi(a,2,"GUI_DOCK_SIZE")? as i32)?; Value::Nil }
            "gui_dock_get_size" => Value::Int(self.gui.dock_get_size(gi(a,0,"GUI_DOCK_GET_SIZE")?, &gs(a,1,"GUI_DOCK_GET_SIZE")?)? as i64),
            "gui_dock_float" => { self.gui.dock_float(gi(a,0,"GUI_DOCK_FLOAT")?, &gs(a,1,"GUI_DOCK_FLOAT")?, gi(a,2,"GUI_DOCK_FLOAT")? as i32,
                gi(a,3,"GUI_DOCK_FLOAT")? as i32, gi(a,4,"GUI_DOCK_FLOAT")? as i32, gi(a,5,"GUI_DOCK_FLOAT")? as i32)?; Value::Nil }
            "gui_dock_move" => {
                let neben = if a.len() > 3 { gs(a,3,"GUI_DOCK_MOVE")? } else { String::new() };
                self.gui.dock_move(gi(a,0,"GUI_DOCK_MOVE")?, &gs(a,1,"GUI_DOCK_MOVE")?, &gs(a,2,"GUI_DOCK_MOVE")?, &neben)?; Value::Nil
            }
            "gui_dock_state$" | "gui_dock_state" => Value::str_rc(self.gui.dock_state(gi(a,0,"GUI_DOCK_STATE$")?)?),
            "gui_dock_set_state" => { self.gui.dock_set_state(gi(a,0,"GUI_DOCK_SET_STATE")?, &gs(a,1,"GUI_DOCK_SET_STATE")?)?; Value::Nil }
            // --- Assistent ---
            "gui_wizard" => Value::Int(self.gui.wizard(gi(a,0,"GUI_WIZARD")?, gi(a,1,"GUI_WIZARD")? as i32,
                gi(a,2,"GUI_WIZARD")? as i32, gi(a,3,"GUI_WIZARD")? as i32, gi(a,4,"GUI_WIZARD")? as i32)?),
            "gui_wizard_add" => Value::Int(self.gui.wizard_add(gi(a,0,"GUI_WIZARD_ADD")?, gs(a,1,"GUI_WIZARD_ADD")?)?),
            "gui_wizard_add_widget" => { self.gui.wizard_add_widget(gi(a,0,"GUI_WIZARD_ADD_WIDGET")?, gi(a,1,"GUI_WIZARD_ADD_WIDGET")?, gi(a,2,"GUI_WIZARD_ADD_WIDGET")?)?; Value::Nil }
            "gui_wizard_count" => Value::Int(self.gui.wizard_count(gi(a,0,"GUI_WIZARD_COUNT")?)?),
            "gui_wizard_step" => Value::Int(self.gui.wizard_step(gi(a,0,"GUI_WIZARD_STEP")?)?),
            "gui_wizard_set_step" => { self.gui.wizard_set_step(gi(a,0,"GUI_WIZARD_SET_STEP")?, gi(a,1,"GUI_WIZARD_SET_STEP")?)?; Value::Nil }
            "gui_wizard_changed" => Value::Bool(self.gui.wizard_changed(gi(a,0,"GUI_WIZARD_CHANGED")?)?),
            "gui_wizard_finished" => Value::Bool(self.gui.wizard_finished(gi(a,0,"GUI_WIZARD_FINISHED")?)?),
            "gui_wizard_cancelled" => Value::Bool(self.gui.wizard_cancelled(gi(a,0,"GUI_WIZARD_CANCELLED")?)?),
            "gui_wizard_enable_next" => { self.gui.wizard_enable_next(gi(a,0,"GUI_WIZARD_ENABLE_NEXT")?, gbool(a,1,"GUI_WIZARD_ENABLE_NEXT")?)?; Value::Nil }
            "gui_wizard_set" => { self.gui.wizard_set(gi(a,0,"GUI_WIZARD_SET")?, &gs(a,1,"GUI_WIZARD_SET")?, gi(a,2,"GUI_WIZARD_SET")?)?; Value::Nil }
            "gui_wizard_labels" => {
                let f = "GUI_WIZARD_LABELS";
                if a.len() != 5 { return Err(format!("{}: erwartet (wz, zurueck$, weiter$, fertig$, abbrechen$)", f)); }
                self.gui.wizard_labels(gi(a,0,f)?, [gs(a,1,f)?, gs(a,2,f)?, gs(a,3,f)?, gs(a,4,f)?])?; Value::Nil
            }
            "gui_wizard_next" => Value::Bool(self.gui.wizard_next(gi(a,0,"GUI_WIZARD_NEXT")?)?),
            "gui_wizard_back" => Value::Bool(self.gui.wizard_back(gi(a,0,"GUI_WIZARD_BACK")?)?),
            // --- Baum mit Spalten ---
            "gui_treetable" => {
                let f = "GUI_TREETABLE";
                let h = self.gui.treetable(gi(a,0,f)?, gi(a,1,f)? as i32, gi(a,2,f)? as i32, gi(a,3,f)? as i32, gi(a,4,f)? as i32)?;
                if a.len() > 5 { self.gui.table_set_headers(h, gstrs(a,5,f)?)?; }
                Value::Int(h)
            }
            "gui_treetable_add" => Value::Int(self.gui.treetable_add(gi(a,0,"GUI_TREETABLE_ADD")?, gi(a,1,"GUI_TREETABLE_ADD")?, gstrs(a,2,"GUI_TREETABLE_ADD")?)?),
            "gui_treetable_expand" => { self.gui.treetable_expand(gi(a,0,"GUI_TREETABLE_EXPAND")?, gi(a,1,"GUI_TREETABLE_EXPAND")?, gbool(a,2,"GUI_TREETABLE_EXPAND")?)?; Value::Nil }
            "gui_treetable_expand_all" => { self.gui.treetable_expand_all(gi(a,0,"GUI_TREETABLE_EXPAND_ALL")?, gbool(a,1,"GUI_TREETABLE_EXPAND_ALL")?)?; Value::Nil }
            "gui_treetable_expanded" => Value::Bool(self.gui.treetable_expanded(gi(a,0,"GUI_TREETABLE_EXPANDED")?, gi(a,1,"GUI_TREETABLE_EXPANDED")?)?),
            "gui_treetable_parent" => Value::Int(self.gui.treetable_parent(gi(a,0,"GUI_TREETABLE_PARENT")?, gi(a,1,"GUI_TREETABLE_PARENT")?)?),
            "gui_treetable_level" => Value::Int(self.gui.treetable_level(gi(a,0,"GUI_TREETABLE_LEVEL")?, gi(a,1,"GUI_TREETABLE_LEVEL")?)?),
            "gui_treetable_set_parent" => { self.gui.treetable_set_parent(gi(a,0,"GUI_TREETABLE_SET_PARENT")?, gi(a,1,"GUI_TREETABLE_SET_PARENT")?, gi(a,2,"GUI_TREETABLE_SET_PARENT")?)?; Value::Nil }
            "gui_filetree_color" => { self.gui.filetree_color(gi(a,0,"GUI_FILETREE_COLOR")?, &gs(a,1,"GUI_FILETREE_COLOR")?, gi(a,2,"GUI_FILETREE_COLOR")?)?; Value::Nil }
            "gui_filetree_icon" => { self.gui.filetree_icon(gi(a,0,"GUI_FILETREE_ICON")?, &gs(a,1,"GUI_FILETREE_ICON")?, &gs(a,2,"GUI_FILETREE_ICON")?)?; Value::Nil }
            "gui_filetree_icons" => { self.gui.filetree_icons(gi(a,0,"GUI_FILETREE_ICONS")?, gi(a,1,"GUI_FILETREE_ICONS")?, gi(a,2,"GUI_FILETREE_ICONS")?)?; Value::Nil }
            "gui_update" => {
                {
                    let g = self.gfx.as_mut().ok_or("GUI_UPDATE: vor SCREEN aufgerufen")?;
                    // ESC gehoert ab jetzt der gui, nicht mehr dem Beenden
                    // (graphics.rs `esc_der_gui`).
                    g.esc_der_gui();
                    // Barrierefreiheit: das Bild rechnen, DANN die Aktionen des
                    // Hilfsprogramms (Fokus, Klick, Wert) anwenden, dann den
                    // vollstaendigen Baum schicken -- nur, wenn ueberhaupt
                    // jemand zuhoert (a11y.rs). Die Aktionen kommen NACH
                    // `update`, weil das die transienten Flags (`clicked`)
                    // zuerst loescht: ein davor gesetzter Klick war fuer
                    // GUI_CLICKED im selben Bild schon wieder weg.
                    self.gui.screenreader = g.a11y_aktiv();
                    // Eingabemethoden (ime.rs): fertige Ergebnisse einfuegen,
                    // die laufende Vorschau ans Feld haengen -- VOR update,
                    // damit dasselbe Bild sie schon zeigt.
                    {
                        let (ergebnisse, vorschau) = g.ime_abholen();
                        let jetzt = g.get_time();
                        for t in ergebnisse { self.gui.ime_ergebnis(&t, jetzt); }
                        self.gui.ime_vorschau(vorschau);
                    }
                    self.gui.update(g);
                    // Danach: hat ein Textfeld den Fokus (dann gehoert die
                    // Umwandlung dem Feld), und wo steht seine Schreibmarke
                    // (Kandidatenliste der IME, Weg B).
                    g.ime_feld_aktiv(self.gui.ime_feld_aktiv());
                    if let Some((x, y, h)) = self.gui.schreibmarke(g) { g.ime_position(x, y, h); }
                    if self.gui.screenreader {
                        for req in g.a11y_aktionen() { self.gui.a11y_aktion(req, g); }
                        let titel = g.fenster_titel().to_string();
                        let baum = self.gui.a11y_baum(g, &titel);
                        g.a11y_senden(baum);
                    }
                }
                // Ausgeloeste FUNCREF-Callbacks feuern (parameterlos), nachdem
                // der State-Update fertig ist -- so kann ein Callback die GUI
                // sicher veraendern; neu ausgeloeste Events landen naechsten Frame.
                // Vorher die Schrift des Programms zurueck (GUI_FONT).
                self.gui_tausch_zurueck();
                for cb in self.gui.take_pending() {
                    self.rueckruf_rufen(&cb, "GUI-Callback")?;
                }
                Value::Nil
            }
            "gui_draw" => {
                // Ohne Argument wie bisher ALLES. `GUI_DRAW(FALSE)` laesst die
                // obenauf-Schicht (Kontextmenue, Tooltip) weg -- die holt sich
                // ein Programm, das danach selbst malt, mit GUI_DRAW_TOP.
                let obenauf = if a.is_empty() { true } else { gbool(a, 0, "GUI_DRAW")? };
                let g = self.gfx.as_mut().ok_or("GUI_DRAW: vor SCREEN aufgerufen")?;
                self.gui.draw(g, obenauf);
                Value::Nil
            }
            "gui_draw_top" => {
                let g = self.gfx.as_mut().ok_or("GUI_DRAW_TOP: vor SCREEN aufgerufen")?;
                self.gui.draw_top(g);
                Value::Nil
            }
            // Ein einzelnes Fenster NACH dem eigenen Zeichnen noch einmal
            // obendrauf. Fuer Programme, die in eine GUI_CANVAS malen: deren
            // Inhalt entsteht per Bauart nach GUI_DRAW und deckt sonst jedes
            // Fenster zu, das darueber liegt.
            "gui_draw_window" => {
                let h = gi(a, 0, "GUI_DRAW_WINDOW")?;
                let g = self.gfx.as_mut().ok_or("GUI_DRAW_WINDOW: vor SCREEN aufgerufen")?;
                self.gui.draw_window_top(g, h)?;
                Value::Nil
            }
            // Braucht die Zeilenhoehe (also die Schrift) -- darum hier bei
            // den Grafik-Builtins.
            "gui_textarea_view" => {
                let g = self.gfx.as_ref().ok_or("GUI_TEXTAREA_VIEW: vor SCREEN aufgerufen")?;
                let (z0, n, von, len) = self.gui.textarea_view(g, gi(a, 0, "GUI_TEXTAREA_VIEW")?)?;
                Value::Tuple(std::rc::Rc::new(vec![
                    Value::Int(z0), Value::Int(n), Value::Int(von), Value::Int(len),
                ]))
            }
            // Textbereich-Befehle fuer Editoren (Weg C): Marke, Auswahl, Suchen,
            // Einfuegen. Hier, weil GOTO die Zeilenhoehe braucht (Scroll) und
            // INSERT die Uhr (Undo-Schritte).
            "gui_textarea_cursor" => {
                let (z, s) = self.gui.textarea_cursor(gi(a, 0, "GUI_TEXTAREA_CURSOR")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(z), Value::Int(s)]))
            }
            "gui_textarea_pos_at" => {
                let g = self.gfx.as_ref().ok_or("GUI_TEXTAREA_POS_AT: vor SCREEN aufgerufen")?;
                let (z, s) = self.gui.textarea_pos_at(g, gi(a, 0, "GUI_TEXTAREA_POS_AT")?,
                    gi(a, 1, "GUI_TEXTAREA_POS_AT")?, gi(a, 2, "GUI_TEXTAREA_POS_AT")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(z), Value::Int(s)]))
            }
            "gui_textarea_goto" => {
                let g = self.gfx.as_ref().ok_or("GUI_TEXTAREA_GOTO: vor SCREEN aufgerufen")?;
                let spalte = if a.len() > 2 { gi(a, 2, "GUI_TEXTAREA_GOTO")? } else { 1 };
                self.gui.textarea_goto(g, gi(a, 0, "GUI_TEXTAREA_GOTO")?, gi(a, 1, "GUI_TEXTAREA_GOTO")?, spalte)?;
                Value::Nil
            }
            "gui_textarea_caret_xy" => {
                let g = self.gfx.as_ref().ok_or("GUI_TEXTAREA_CARET_XY: vor SCREEN aufgerufen")?;
                let (x, y, h) = self.gui.textarea_caret_xy(g, gi(a, 0, "GUI_TEXTAREA_CARET_XY")?)?;
                Value::Tuple(Rc::new(vec![Value::Int(x), Value::Int(y), Value::Int(h)]))
            }
            "gui_textarea_scroll" => {
                let g = self.gfx.as_ref().ok_or("GUI_TEXTAREA_SCROLL: vor SCREEN aufgerufen")?;
                self.gui.textarea_scroll(g, gi(a, 0, "GUI_TEXTAREA_SCROLL")?, gi(a, 1, "GUI_TEXTAREA_SCROLL")?)?;
                Value::Nil
            }
            "gui_textarea_selection$" | "gui_textarea_selection" =>
                Value::str_rc(&self.gui.textarea_selection(gi(a, 0, "GUI_TEXTAREA_SELECTION$")?)?),
            "gui_textarea_selection_range" => {
                let (z1, s1, z2, s2) = self.gui.textarea_selection_range(gi(a, 0, "GUI_TEXTAREA_SELECTION_RANGE")?)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(z1), Value::Int(s1), Value::Int(z2), Value::Int(s2)]))
            }
            "gui_textarea_select" => {
                let g = self.gfx.as_ref().ok_or("GUI_TEXTAREA_SELECT: vor SCREEN aufgerufen")?;
                self.gui.textarea_select(g, gi(a, 0, "GUI_TEXTAREA_SELECT")?, gi(a, 1, "GUI_TEXTAREA_SELECT")?,
                    gi(a, 2, "GUI_TEXTAREA_SELECT")?, gi(a, 3, "GUI_TEXTAREA_SELECT")?, gi(a, 4, "GUI_TEXTAREA_SELECT")?)?;
                Value::Nil
            }
            "gui_textarea_insert" => {
                let jetzt = self.gfx.as_ref().map(|g| g.get_time()).unwrap_or(0.0);
                self.gui.textarea_insert(gi(a, 0, "GUI_TEXTAREA_INSERT")?, &gs(a, 1, "GUI_TEXTAREA_INSERT")?, jetzt)?;
                Value::Nil
            }
            "gui_textarea_share" => {
                self.gui.textarea_share(gi(a, 0, "GUI_TEXTAREA_SHARE")?, gi(a, 1, "GUI_TEXTAREA_SHARE")?)?;
                Value::Nil
            }
            "gui_textarea_pairs" => {
                self.gui.textarea_pairs(gi(a, 0, "GUI_TEXTAREA_PAIRS")?, &gs(a, 1, "GUI_TEXTAREA_PAIRS")?)?;
                Value::Nil
            }
            "gui_textarea_style" => {
                let jetzt = self.gfx.as_ref().map(|g| g.get_time()).unwrap_or(0.0);
                let bits = crate::schnitt::stil_parsen(&gs(a, 1, "GUI_TEXTAREA_STYLE")?)
                    .map_err(|e| format!("GUI_TEXTAREA_STYLE: {}", e))?;
                let an = if a.len() > 2 { gbool(a, 2, "GUI_TEXTAREA_STYLE")? } else { true };
                self.gui.textarea_style(gi(a, 0, "GUI_TEXTAREA_STYLE")?, bits, an, jetzt)?;
                Value::Nil
            }
            "gui_textarea_get_style$" | "gui_textarea_get_style" =>
                Value::Str(self.gui.textarea_get_style(gi(a, 0, "GUI_TEXTAREA_GET_STYLE$")?)?.into()),
            "gui_textarea_markdown$" | "gui_textarea_markdown" =>
                Value::Str(self.gui.textarea_markdown(gi(a, 0, "GUI_TEXTAREA_MARKDOWN$")?)?.into()),
            "gui_textarea_set_markdown" => {
                self.gui.textarea_set_markdown(gi(a, 0, "GUI_TEXTAREA_SET_MARKDOWN")?, &gs(a, 1, "GUI_TEXTAREA_SET_MARKDOWN")?)?;
                Value::Nil
            }
            "gui_textarea_find" => {
                let ab_zeile = if a.len() > 2 { gi(a, 2, "GUI_TEXTAREA_FIND")? } else { 1 };
                let ab_spalte = if a.len() > 3 { gi(a, 3, "GUI_TEXTAREA_FIND")? } else { 1 };
                let genau = a.len() > 4 && gbool(a, 4, "GUI_TEXTAREA_FIND")?;
                let (z, s) = self.gui.textarea_find(gi(a, 0, "GUI_TEXTAREA_FIND")?, &gs(a, 1, "GUI_TEXTAREA_FIND")?, ab_zeile, ab_spalte, genau)?;
                Value::Tuple(std::rc::Rc::new(vec![Value::Int(z), Value::Int(s)]))
            }
            "gui_textarea_marks" => {
                let n = "GUI_TEXTAREA_MARKS";
                if a.len() != 3 { return Err(format!("{}: erwartet (ta, zeilen, farben)", n)); }
                self.gui.textarea_marks(gi(a, 0, n)?, ganze(&a[1], n)?, ganze(&a[2], n)?)?;
                Value::Nil
            }
            "gui_textarea_hints" => {
                let n = "GUI_TEXTAREA_HINTS";
                if a.len() < 3 || a.len() > 4 { return Err(format!("{}: erwartet (ta, zeilen, texte [, farbe])", n)); }
                let texte: Vec<String> = match &a[2] {
                    Value::Array(arr) => {
                        let arr = arr.borrow();
                        let mut o = Vec::with_capacity(arr.cells.len());
                        for x in arr.cells.iter() {
                            match x {
                                Value::Str(s) => o.push(s.to_string()),
                                _ => return Err(format!("{}: texte muss ARRAY OF STRING sein", n)),
                            }
                        }
                        o
                    }
                    _ => return Err(format!("{}: texte muss ein ARRAY sein", n)),
                };
                // farbe: eine fuer alle, oder ein Feld mit einer je Hinweis
                let (farbe, farben) = match a.get(3) {
                    None => (-1, Vec::new()),
                    Some(Value::Array(_)) => (-1, ganze(&a[3], n)?),
                    Some(_) => (gi(a, 3, n)?, Vec::new()),
                };
                self.gui.textarea_hints(gi(a, 0, n)?, ganze(&a[1], n)?, texte, farbe, farben)?;
                Value::Nil
            }
            "gui_textarea_line_colors" => {
                let n = "GUI_TEXTAREA_LINE_COLORS";
                if a.len() != 3 { return Err(format!("{}: erwartet (ta, zeilen, farben)", n)); }
                self.gui.textarea_line_colors(gi(a, 0, n)?, ganze(&a[1], n)?, ganze(&a[2], n)?)?;
                Value::Nil
            }
            "gui_textarea_line_colors_get" => {
                let n = "GUI_TEXTAREA_LINE_COLORS_GET";
                if a.len() != 1 { return Err(format!("{}: erwartet (ta)", n)); }
                crate::builtins::new_int_array(self.gui.textarea_line_colors_get(gi(a, 0, n)?)?)
            }
            "gui_textarea_marks_get" => {
                let n = "GUI_TEXTAREA_MARKS_GET";
                if a.len() != 1 { return Err(format!("{}: erwartet (ta)", n)); }
                crate::builtins::new_int_array(self.gui.textarea_marks_get(gi(a, 0, n)?)?)
            }
            // ===== Faltung (docs/module-gui.md, Abschnitt Faltung) =====
            // Welche Zeilen einen Block bilden, sagt der Aufrufer -- die
            // Laufzeit kennt hier keine Sprache und zaehlt keine Einrueckung.
            "gui_textarea_foldable" => {
                let n = "GUI_TEXTAREA_FOLDABLE";
                if a.len() != 3 { return Err(format!("{}: erwartet (ta, von_zeilen, bis_zeilen)", n)); }
                self.gui.textarea_foldable(gi(a, 0, n)?, ganze(&a[1], n)?, ganze(&a[2], n)?)?;
                Value::Nil
            }
            "gui_textarea_fold" => {
                let n = "GUI_TEXTAREA_FOLD";
                let an = if a.len() > 2 { Some(a[2].truthy()) } else { None };
                Value::Bool(self.gui.textarea_fold(gi(a, 0, n)?, gi(a, 1, n)?, an)?)
            }
            "gui_textarea_fold_all" => {
                let n = "GUI_TEXTAREA_FOLD_ALL";
                Value::Int(self.gui.textarea_fold_all(gi(a, 0, n)?, a.len() < 2 || a[1].truthy())?)
            }
            "gui_textarea_folded" => Value::Bool(self.gui.textarea_folded(
                gi(a, 0, "GUI_TEXTAREA_FOLDED")?, gi(a, 1, "GUI_TEXTAREA_FOLDED")?)?),
            "gui_textarea_swatches" => {
                let n = "GUI_TEXTAREA_SWATCHES";
                if a.len() != 4 { return Err(format!("{}: erwartet (ta, starts, laengen, farben)", n)); }
                self.gui.textarea_swatches(gi(a, 0, n)?, ganze(&a[1], n)?, ganze(&a[2], n)?, ganze(&a[3], n)?)?;
                Value::Nil
            }
            "gui_textarea_gutter_clicked" => {
                if a.is_empty() || a.len() > 2 {
                    return Err("GUI_TEXTAREA_GUTTER_CLICKED: erwartet (textarea [, taste])".into());
                }
                let taste = if a.len() == 2 { gi(a, 1, "GUI_TEXTAREA_GUTTER_CLICKED")? } else { 0 };
                if taste != 0 && taste != 1 {
                    return Err("GUI_TEXTAREA_GUTTER_CLICKED: taste ist 0 (links) oder 1 (rechts)".into());
                }
                Value::Int(self.gui.textarea_gutter_clicked(gi(a, 0, "GUI_TEXTAREA_GUTTER_CLICKED")?, taste)?)
            }
            "gui_textarea_hint_clicked" => Value::Int(
                self.gui.textarea_hint_clicked(gi(a, 0, "GUI_TEXTAREA_HINT_CLICKED")?)?),
            "gui_textarea_swatch_clicked" => Value::Int(
                self.gui.textarea_swatch_clicked(gi(a, 0, "GUI_TEXTAREA_SWATCH_CLICKED")?)?),
            "gui_textarea_abbrev" => {
                let n = "GUI_TEXTAREA_ABBREV";
                if a.len() != 2 { return Err(format!("{}: erwartet (ta, woerter)", n)); }
                let mut woerter = Vec::new();
                match &a[1] {
                    Value::Array(arr) => {
                        for x in arr.borrow().cells.iter() {
                            match x {
                                Value::Str(s) => woerter.push(s.to_string()),
                                _ => return Err(format!("{}: ARRAY OF STRING noetig", n)),
                            }
                        }
                    }
                    _ => return Err(format!("{}: ARRAY OF STRING noetig", n)),
                }
                self.gui.textarea_abbrev(gi(a, 0, n)?, woerter)?;
                Value::Nil
            }
            "gui_textarea_select_columns" => {
                let n = "GUI_TEXTAREA_SELECT_COLUMNS";
                if a.len() != 5 { return Err(format!("{}: erwartet (ta, z1, s1, z2, s2)", n)); }
                Value::Int(self.gui.textarea_select_columns(gi(a,0,n)?, gi(a,1,n)?, gi(a,2,n)?,
                                                            gi(a,3,n)?, gi(a,4,n)?)?)
            }
            "gui_textarea_abbrev_hit" => Value::Int(
                self.gui.textarea_abbrev_hit(gi(a, 0, "GUI_TEXTAREA_ABBREV_HIT")?)?),
            "gui_textarea_tab_hit" => Value::Bool(
                self.gui.textarea_tab_hit(gi(a, 0, "GUI_TEXTAREA_TAB_HIT")?)?),
            "gui_textarea_indent_words" => {
                let n = "GUI_TEXTAREA_INDENT_WORDS";
                if a.len() != 4 { return Err(format!("{}: erwartet (ta, anfang, ende, aus)", n)); }
                fn texte(v: &Value, fn_: &str) -> R<Vec<String>> {
                    match v {
                        Value::Array(arr) => {
                            let arr = arr.borrow();
                            let mut o = Vec::with_capacity(arr.cells.len());
                            for x in arr.cells.iter() {
                                match x {
                                    Value::Str(s) => o.push(s.to_string()),
                                    _ => return Err(format!("{}: ARRAY OF STRING noetig", fn_)),
                                }
                            }
                            Ok(o)
                        }
                        _ => Err(format!("{}: ARRAY OF STRING noetig", fn_)),
                    }
                }
                self.gui.textarea_indent_words(gi(a, 0, n)?, texte(&a[1], n)?, texte(&a[2], n)?, texte(&a[3], n)?)?;
                Value::Nil
            }
            "gui_textarea_close_words" => {
                let n = "GUI_TEXTAREA_CLOSE_WORDS";
                if a.len() != 3 { return Err(format!("{}: erwartet (ta, oeffner, schluesse)", n)); }
                fn texte(v: &Value, fn_: &str) -> R<Vec<String>> {
                    match v {
                        Value::Array(arr) => {
                            let arr = arr.borrow();
                            let mut o = Vec::with_capacity(arr.cells.len());
                            for x in arr.cells.iter() {
                                match x {
                                    Value::Str(s) => o.push(s.to_string()),
                                    _ => return Err(format!("{}: ARRAY OF STRING noetig", fn_)),
                                }
                            }
                            Ok(o)
                        }
                        _ => Err(format!("{}: ARRAY OF STRING noetig", fn_)),
                    }
                }
                self.gui.textarea_close_words(gi(a, 0, n)?, texte(&a[1], n)?, texte(&a[2], n)?)?;
                Value::Nil
            }
            "gui_textarea_add_caret" => {
                let n = "GUI_TEXTAREA_ADD_CARET";
                Value::Int(self.gui.textarea_add_caret(gi(a, 0, n)?, gi(a, 1, n)?,
                    if a.len() > 2 { gi(a, 2, n)? } else { 1 })?)
            }
            "gui_textarea_carets" => Value::Int(self.gui.textarea_carets(gi(a, 0, "GUI_TEXTAREA_CARETS")?)?),
            "gui_textarea_clear_carets" => {
                self.gui.textarea_clear_carets(gi(a, 0, "GUI_TEXTAREA_CLEAR_CARETS")?)?;
                Value::Nil
            }
            "gui_textarea_folds" => {
                let z = self.gui.textarea_folds(gi(a, 0, "GUI_TEXTAREA_FOLDS")?)?;
                let n = z.len() as i64;
                let mut arr = DhArray::new("integer".to_string(), vec![n], || Value::Int(0));
                for (k, l) in z.into_iter().enumerate() { arr.cells.set(k, Value::Int(l)); }
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            // Dialog IM Fenster (eigenes Thema, kein OS-Kasten). Braucht die
            // Bildschirmgroesse (Zentrieren) und die Textbreite (das Fenster
            // passt sich dem Text an) -- darum hier bei den Grafik-Builtins
            // und nicht bei den reinen Konstruktoren.
            //
            // NICHT zu verwechseln mit GUI_MESSAGE/GUI_CONFIRM: das sind die
            // nativen, BLOCKIERENDEN OS-Kaesten (rfd, weiter unten).
            "gui_dialog" => {
                let (titel, text) = (gs(a, 0, "GUI_DIALOG")?, gs(a, 1, "GUI_DIALOG")?);
                let stil = if a.len() > 2 { gs(a, 2, "GUI_DIALOG")? } else { String::new() };
                // Dieselben Stil-Woerter wie GUI_CONFIRM -- zwei Vokabulare
                // fuer dieselbe Wahl waeren eine Stolperfalle. Alles mit `|`
                // ist ein eigener Knopfsatz: "Speichern|Verwerfen|Abbrechen".
                let knoepfe: Vec<String> = match stil.to_lowercase().as_str() {
                    "" | "ok" => vec!["OK".into()],
                    "janein" | "ja/nein" | "frage" => vec!["Ja".into(), "Nein".into()],
                    _ if stil.contains('|') => stil.split('|').map(|k| k.trim().to_string()).collect(),
                    other => return Err(format!(
                        "GUI_DIALOG: '{}' ist kein Stil -- moeglich sind \"ok\" (Vorgabe), \"janein\" \
                         oder eigene Knoepfe mit | getrennt (\"Speichern|Verwerfen|Abbrechen\")", other)),
                };
                let g = self.gfx.as_ref().ok_or("GUI_DIALOG: vor SCREEN aufgerufen")?;
                Value::Int(self.gui.dialog_frei(g, titel, text, knoepfe, None)?)
            }
            "gui_prompt" => {
                // GUI_PROMPT(titel$, text$[, vorgabe$[, knoepfe$]]) -> GUI_WINDOW
                let (titel, text) = (gs(a, 0, "GUI_PROMPT")?, gs(a, 1, "GUI_PROMPT")?);
                let vorgabe = if a.len() > 2 { gs(a, 2, "GUI_PROMPT")? } else { String::new() };
                let knoepfe: Vec<String> = if a.len() > 3 {
                    gs(a, 3, "GUI_PROMPT")?.split('|').map(|k| k.trim().to_string()).collect()
                } else { vec!["OK".into(), "Abbrechen".into()] };
                let g = self.gfx.as_ref().ok_or("GUI_PROMPT: vor SCREEN aufgerufen")?;
                Value::Int(self.gui.dialog_frei(g, titel, text, knoepfe, Some(vorgabe))
                    .map_err(|e| e.replace("GUI_DIALOG", "GUI_PROMPT"))?)
            }
            "gui_dialog_text" => Value::str_rc(&self.gui.dialog_text(gi(a, 0, "GUI_DIALOG_TEXT")?)?),
            "gui_window_modal" => { self.gui.window_modal(gi(a, 0, "GUI_WINDOW_MODAL")?, gbool(a, 1, "GUI_WINDOW_MODAL")?)?; Value::Nil }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }

    /// Audio-Geraet lazy initialisieren (bei erstem Sound-/Musik-Builtin).
    #[cfg(feature = "graphics")]
    pub(super) fn audio_mut(&mut self) -> R<&mut crate::audio::Audio> {
        if self.audio.is_none() {
            self.audio = Some(crate::audio::Audio::new()?);
        }
        Ok(self.audio.as_mut().unwrap())
    }

    /// UI_TABLE (Immediate-Mode): scrollbare Tabelle mit fixierter Kopfzeile,
    /// optionalen Zell-Farben, Selektion + klickbaren Headern. 1:1-Port der
    /// Python-Referenz. Liefert die in diesem Frame geklickte Zeile (-1).
    #[cfg(feature = "graphics")]
    pub(super) fn ui_table(&mut self, a: &[Value]) -> R<Value> {
        const HDR: i32 = 22; const ROW: i32 = 20; const SCR: i32 = 12; const PAD: i32 = 6;
        // --- Arg-Extraktion (lokal) ---
        fn gi(a: &[Value], i: usize, f: &str) -> R<i64> {
            match a.get(i) { Some(Value::Int(n)) => Ok(*n),
                Some(Value::Float(x)) => crate::builtins::glatt_int(*x).ok_or_else(|| format!("{}: erwartet INTEGER (Arg {}), erhalten {}{}",
                    f, i + 1, crate::value::fmt_float(*x), crate::builtins::KEIN_GANZ_HINWEIS)),
                _ => Err(format!("{}: erwartet INTEGER (Arg {})", f, i + 1)) }
        }
        fn gid(a: &[Value], f: &str) -> R<String> {
            match a.first() { Some(Value::Str(s)) => Ok(s.to_string()), _ => Err(format!("{}: id muss STRING sein", f)) }
        }
        fn str1d(v: &Value, f: &str) -> R<Vec<String>> {
            match v { Value::Array(arr) => { let arr = arr.borrow();
                if arr.cells.len() > 0 && (arr.element_type != "string" || arr.dims.len() != 1) { return Err(format!("{}: headers muss 1D ARRAY OF STRING sein", f)); }
                Ok(arr.cells.iter().map(|x| match x { Value::Str(s) => s.to_string(), o => o.fmt() }).collect()) }
                _ => Err(format!("{}: headers muss ARRAY OF STRING sein", f)) }
        }
        fn str2d(v: &Value, ncols: usize, f: &str) -> R<Vec<Vec<String>>> {
            match v { Value::Array(arr) => { let arr = arr.borrow();
                if arr.cells.len() > 0 && (arr.element_type != "string" || arr.dims.len() != 2) { return Err(format!("{}: cells muss 2D ARRAY OF STRING sein", f)); }
                let (r, c) = (arr.dims[0] as usize, arr.dims[1] as usize);
                if c != ncols { return Err(format!("{}: cells hat {} Spalten, headers {}", f, c, ncols)); }
                Ok((0..r).map(|ri| (0..c).map(|ci| match &arr.cells.get(ri * c + ci) { Value::Str(s) => s.to_string(), o => o.fmt() }).collect()).collect()) }
                _ => Err(format!("{}: cells muss 2D ARRAY OF STRING sein", f)) }
        }
        // Optionales 2D INTEGER-Array -> flach (row-major), Validierung dims.
        fn int2d_opt(a: &[Value], i: usize, nr: usize, nc: usize, f: &str) -> R<Option<Vec<i64>>> {
            match a.get(i) { None | Some(Value::Nil) => Ok(None),
                Some(Value::Array(arr)) => { let arr = arr.borrow();
                    if arr.element_type != "integer" || arr.dims.len() != 2 || arr.dims[0] as usize != nr || arr.dims[1] as usize != nc {
                        return Err(format!("{}: erwartet 2D ARRAY OF INTEGER [{}, {}]", f, nr, nc)); }
                    Ok(Some(arr.cells.iter().map(|x| match x { Value::Int(n) => n, _ => 0 }).collect())) }
                _ => Err(format!("{}: erwartet 2D ARRAY OF INTEGER", f)) }
        }
        fn int1d_opt(a: &[Value], i: usize, n: usize, f: &str) -> R<Option<Vec<i32>>> {
            match a.get(i) { None | Some(Value::Nil) => Ok(None),
                Some(Value::Array(arr)) => { let arr = arr.borrow();
                    if arr.element_type != "integer" || arr.dims.len() != 1 || arr.dims[0] as usize != n {
                        return Err(format!("{}: col_widths muss 1D ARRAY OF INTEGER ({}) sein", f, n)); }
                    Ok(Some(arr.cells.iter().map(|x| match x { Value::Int(v) => v as i32, _ => 0 }).collect())) }
                _ => Err(format!("{}: col_widths muss ARRAY OF INTEGER sein", f)) }
        }
        let in_box = |mx: i32, my: i32, x: i32, y: i32, w: i32, h: i32| mx >= x && mx < x + w && my >= y && my < y + h;

        let id = gid(a, "UI_TABLE")?;
        let mut x = gi(a, 1, "UI_TABLE")? as i32 + self.ui_state.offset_x;
        let mut y = gi(a, 2, "UI_TABLE")? as i32 + self.ui_state.offset_y;
        let w = gi(a, 3, "UI_TABLE")? as i32;
        let h = gi(a, 4, "UI_TABLE")? as i32;
        let headers = str1d(a.get(5).ok_or("UI_TABLE: headers fehlt")?, "UI_TABLE")?;
        let n_cols = headers.len();
        if n_cols == 0 { return Err("UI_TABLE: headers darf nicht leer sein".into()); }
        let cells = str2d(a.get(6).ok_or("UI_TABLE: cells fehlt")?, n_cols, "UI_TABLE")?;
        let n_rows = cells.len();
        let cell_colors = int2d_opt(a, 7, n_rows, n_cols, "UI_TABLE")?;
        let col_widths: Vec<i32> = match int1d_opt(a, 8, n_cols, "UI_TABLE")? {
            Some(cw) => cw,
            None => { let avail = w - SCR - 2; let per = (avail / n_cols as i32).max(40); vec![per; n_cols] }
        };
        let cell_bg = int2d_opt(a, 9, n_rows, n_cols, "UI_TABLE")?;
        let _ = (&mut x, &mut y);

        // --- State init ---
        let st = self.ui_state.tables.entry(id.clone()).or_insert_with(|| UiTable { selected: -1, ..Default::default() });
        if st.selected >= n_rows as i32 { st.selected = -1; }
        st.header_col = -1;

        // --- Geometrie ---
        let body_x = x + 1; let body_y = y + HDR;
        let body_w_raw = w - 2; let body_h_raw = h - HDR - 1;
        let total_w: i32 = col_widths.iter().sum();
        let total_h = n_rows as i32 * ROW;
        let mut need_v = total_h > body_h_raw;
        let need_h = total_w > body_w_raw - if need_v { SCR } else { 0 };
        if need_h && total_h > body_h_raw - SCR { need_v = true; }
        let body_w = body_w_raw - if need_v { SCR } else { 0 };
        let body_h = body_h_raw - if need_h { SCR } else { 0 };
        let max_sy = (total_h - body_h).max(0);
        let max_sx = (total_w - body_w).max(0);

        // --- Maus / Wheel ---
        let (mx, my, down) = self.ui_mouse_gated()?;
        let wheel = { let g = self.gfx.as_ref().ok_or("UI_TABLE: vor SCREEN")?; g.pop_mouse_wheel() };
        let was_down = self.ui_state.was_mouse_down;
        let just_pressed = down && !was_down;
        let just_released = !down && was_down;
        let over_table = in_box(mx, my, x, y, w, h);

        let st = self.ui_state.tables.get_mut(&id).unwrap();
        st.scroll_y = st.scroll_y.clamp(0, max_sy);
        st.scroll_x = st.scroll_x.clamp(0, max_sx);
        if over_table && wheel != 0 {
            st.scroll_y = (st.scroll_y - wheel as i32 * ROW).clamp(0, max_sy);
        }
        // Vertikale Scrollbar
        if need_v {
            let sb_x = x + body_w_raw - SCR + 1; let sb_y = body_y; let sb_h = body_h;
            let handle_h = ((sb_h as f64 * (body_h as f64 / total_h as f64)) as i32).max(16);
            if just_pressed && in_box(mx, my, sb_x, sb_y, SCR, sb_h) {
                let ratio = if max_sy > 0 { st.scroll_y as f64 / max_sy as f64 } else { 0.0 };
                let handle_y = sb_y + ((sb_h - handle_h) as f64 * ratio) as i32;
                if in_box(mx, my, sb_x, handle_y, SCR, handle_h) {
                    st.drag_v = true; st.drag_off = my - handle_y;
                } else {
                    let hy = (my - handle_h / 2).clamp(sb_y, sb_y + sb_h - handle_h);
                    st.drag_v = true; st.drag_off = handle_h / 2;
                    let tp = (hy - sb_y) as f64 / (sb_h - handle_h).max(1) as f64;
                    st.scroll_y = (tp * max_sy as f64) as i32;
                }
            }
            if st.drag_v && down {
                let nh = (my - st.drag_off).clamp(sb_y, sb_y + sb_h - handle_h);
                let tp = (nh - sb_y) as f64 / (sb_h - handle_h).max(1) as f64;
                st.scroll_y = (tp * max_sy as f64) as i32;
            }
            if just_released { st.drag_v = false; }
        } else { st.drag_v = false; }
        // Horizontale Scrollbar
        if need_h {
            let hs_x = body_x; let hs_y = y + h - SCR - 1; let hs_w = body_w;
            let handle_w = ((hs_w as f64 * (body_w as f64 / total_w as f64)) as i32).max(16);
            if just_pressed && in_box(mx, my, hs_x, hs_y, hs_w, SCR) {
                let ratio = if max_sx > 0 { st.scroll_x as f64 / max_sx as f64 } else { 0.0 };
                let handle_x = hs_x + ((hs_w - handle_w) as f64 * ratio) as i32;
                if in_box(mx, my, handle_x, hs_y, handle_w, SCR) {
                    st.drag_h = true; st.drag_off = mx - handle_x;
                } else {
                    let hx = (mx - handle_w / 2).clamp(hs_x, hs_x + hs_w - handle_w);
                    st.drag_h = true; st.drag_off = handle_w / 2;
                    let tp = (hx - hs_x) as f64 / (hs_w - handle_w).max(1) as f64;
                    st.scroll_x = (tp * max_sx as f64) as i32;
                }
            }
            if st.drag_h && down {
                let nx = (mx - st.drag_off).clamp(hs_x, hs_x + hs_w - handle_w);
                let tp = (nx - hs_x) as f64 / (hs_w - handle_w).max(1) as f64;
                st.scroll_x = (tp * max_sx as f64) as i32;
            }
            if just_released { st.drag_h = false; }
        } else { st.drag_h = false; }

        let (scroll_x, scroll_y, drag_v, drag_h) = (st.scroll_x, st.scroll_y, st.drag_v, st.drag_h);
        // Hover-Zeile (ueber Body, nicht ueber Scrollbalken)
        let hover_row = if over_table && in_box(mx, my, body_x, body_y, body_w, body_h) && !drag_v && !drag_h {
            let hr = (my - body_y + scroll_y) / ROW;
            if hr >= 0 && hr < n_rows as i32 { hr } else { -1 }
        } else { -1 };

        // --- Klick-Erkennung Zeile (Press+Release auf derselben) ---
        let mut clicked_row = -1;
        let sb_block = over_table && in_box(mx, my, x + body_w_raw - SCR + 1, body_y, SCR, body_h);
        if over_table && just_pressed && hover_row >= 0 && !sb_block {
            self.ui_state.click_origin = Some(format!("{}#row#{}", id, hover_row));
        }
        if over_table && just_released && hover_row >= 0 {
            if self.ui_state.click_origin.as_deref() == Some(format!("{}#row#{}", id, hover_row).as_str()) {
                clicked_row = hover_row;
            }
        }
        if clicked_row >= 0 { self.ui_state.tables.get_mut(&id).unwrap().selected = clicked_row; }

        // --- Header-Klick (fuer Sortierung) ---
        let header_col_at = |px: i32, py: i32| -> i32 {
            if !in_box(px, py, x, y, body_w_raw, HDR) { return -1; }
            let rel = px - (body_x - scroll_x);
            let mut acc = 0;
            for c in 0..n_cols { if acc <= rel && rel < acc + col_widths[c] { return c as i32; } acc += col_widths[c]; }
            -1
        };
        if over_table && just_pressed {
            let hc = header_col_at(mx, my);
            if hc >= 0 { self.ui_state.click_origin = Some(format!("{}#hdr#{}", id, hc)); }
        }
        if over_table && just_released {
            if let Some(co) = self.ui_state.click_origin.clone() {
                if let Some(rest) = co.strip_prefix(&format!("{}#hdr#", id)) {
                    if let Ok(oc) = rest.parse::<i32>() {
                        if oc == header_col_at(mx, my) { self.ui_state.tables.get_mut(&id).unwrap().header_col = oc; }
                    }
                }
            }
        }
        let selected = self.ui_state.tables[&id].selected;

        // --- Zeichnen ---
        let g = self.gfx.as_mut().ok_or("UI_TABLE: vor SCREEN")?;
        g.box_fill(x, y, x + w - 1, y + h - 1, 0x1A1C2A);
        g.rect(x, y, x + w - 1, y + h - 1, 0x60607A);
        g.box_fill(x + 1, y + 1, x + w - 2, y + HDR - 1, 0x383C5C);
        g.rect(x, y, x + w - 1, y + HDR - 1, 0x60607A);
        // Kopf-Zellen
        g.push_clip(x + 1, y + 1, body_w_raw, HDR - 2);
        let mut cx = body_x - scroll_x;
        for c in 0..n_cols {
            if cx + col_widths[c] > x + 1 && cx < x + 1 + body_w_raw {
                g.text(cx + PAD, y + 4, headers[c].clone(), 0xFFFFFF);
                if c < n_cols - 1 { g.line(cx + col_widths[c], y + 1, cx + col_widths[c], y + HDR - 2, 0x60607A); }
            }
            cx += col_widths[c];
        }
        g.pop_clip();
        // Body
        g.push_clip(body_x, body_y, body_w, body_h);
        let first = (scroll_y / ROW).max(0);
        let last = ((scroll_y + body_h) / ROW + 1).min(n_rows as i32);
        for r in first..last {
            let row_y = body_y + r * ROW - scroll_y;
            // Pass 1: Zell-Hintergruende
            if let Some(ref bg) = cell_bg {
                let mut bx = body_x - scroll_x;
                for c in 0..n_cols {
                    let cw = col_widths[c];
                    if bx + cw > body_x && bx < body_x + body_w {
                        let v = bg[r as usize * n_cols + c];
                        if v >= 0 { g.box_fill(bx, row_y, bx + cw - 1, row_y + ROW - 1, v); }
                    }
                    bx += cw;
                }
            }
            if r == selected { g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + ROW - 1, 0x2A4E6A); }
            if r == hover_row { g.box_fill(body_x, row_y, body_x + body_w - 1, row_y + ROW - 1, 0x2A2E4A); }
            let mut cx = body_x - scroll_x;
            let row = &cells[r as usize];
            for c in 0..n_cols {
                let cw = col_widths[c];
                if cx + cw > body_x && cx < body_x + body_w {
                    let clip_x = (cx + 1).max(body_x);
                    let clip_w = (cw - 2).min((body_x + body_w) - clip_x);
                    g.push_clip(clip_x, row_y, clip_w, ROW);
                    let color = match &cell_colors { Some(cc) => cc[r as usize * n_cols + c], None => 0xFFFFFF };
                    g.text(cx + PAD, row_y + 3, row[c].clone(), color);
                    g.pop_clip();
                }
                cx += cw;
            }
            g.line(body_x, row_y + ROW - 1, body_x + body_w - 1, row_y + ROW - 1, 0x2A2E4A);
        }
        g.pop_clip();
        // Scrollbalken
        if need_v {
            let sb_x = x + body_w_raw - SCR + 1; let sb_h = body_h;
            let handle_h = ((sb_h as f64 * (body_h as f64 / total_h as f64)) as i32).max(16);
            let ratio = if max_sy > 0 { scroll_y as f64 / max_sy as f64 } else { 0.0 };
            let hy = body_y + ((sb_h - handle_h) as f64 * ratio) as i32;
            g.box_fill(sb_x, body_y, sb_x + SCR - 1, body_y + sb_h - 1, 0x252840);
            g.box_fill(sb_x + 2, hy, sb_x + SCR - 3, hy + handle_h - 1, if drag_v { 0x80C0FF } else { 0x60607A });
        }
        if need_h {
            let hs_y = y + h - SCR - 1; let hs_w = body_w;
            let handle_w = ((hs_w as f64 * (body_w as f64 / total_w as f64)) as i32).max(16);
            let ratio = if max_sx > 0 { scroll_x as f64 / max_sx as f64 } else { 0.0 };
            let hx = body_x + ((hs_w - handle_w) as f64 * ratio) as i32;
            g.box_fill(body_x, hs_y, body_x + hs_w - 1, hs_y + SCR - 1, 0x252840);
            g.box_fill(hx, hs_y + 2, hx + handle_w - 1, hs_y + SCR - 3, if drag_h { 0x80C0FF } else { 0x60607A });
        }
        Ok(Value::Int(clicked_row as i64))
    }
}
