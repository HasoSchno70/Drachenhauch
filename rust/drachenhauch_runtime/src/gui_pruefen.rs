//! Barrierefreiheit pruefen (GUI_A11Y_CHECK$, Inspektor).
//!
//! Was eine Oberflaeche fuer Menschen mit schwacher Sicht, mit
//! Bildschirmleser oder mit unruhiger Hand unbedienbar macht, laesst sich
//! an der gui selbst ablesen -- die Werte hat sie alle:
//!
//! * **Kontrast** der Schrift auf ihrem Grund, mit denselben Farben, die
//!   das Zeichnen nimmt (WCAG 1.4.3: 4,5:1, grosse Schrift ab 24 Punkten
//!   3:1). Abgeschaltete Widgets zaehlen nicht (WCAG nimmt sie aus).
//! * **Name** fuer den Bildschirmleser: ein bedienbares Widget ohne Text,
//!   ohne Beschriftung daneben und ohne Hilfetext ist dort nur "Knopf".
//!   Dieselbe Regel wie der Baum fuer AccessKit (gui_a11y.rs).
//! * **Zielgroesse** (WCAG 2.5.8): kleiner als 24 x 24 Punkte nur, wenn ein
//!   Kreis von 24 Punkten um seine Mitte kein anderes Ziel beruehrt.
//!
//! * **Tab-Reihenfolge** (WCAG 2.4.3): springt Tab zurueck nach oben (ohne
//!   eine neue Spalte rechts zu beginnen) oder in derselben Zeile gegen die
//!   Leserichtung, verliert man sich ohne Maus. Den Befund bekommt das
//!   Widget, zu dem gesprungen wird.
//!
//! Gemessen wird auf dem Grund, auf dem ein Widget steht: dem eigenen,
//! sonst dem Fenster -- ein Panel mit eigener Farbe darunter kennt die
//! Pruefung nicht.

use super::*;

/// Mindestkontrast fuer Text (WCAG AA) und fuer grosse Schrift.
const KONTRAST_TEXT: f64 = 4.5;
const KONTRAST_GROSS: f64 = 3.0;
/// Ab dieser Schriftgroesse (Punkte) gilt Schrift als gross.
const GROSSE_SCHRIFT: i32 = 24;
/// Kleinste Zielgroesse (WCAG 2.5.8, Punkte).
const ZIEL: i32 = 24;

impl Gui {
    /// Ist das Widget etwas, das man bedient (und das darum einen Namen
    /// und eine greifbare Groesse braucht)?
    fn bedienbar(w: &Widget) -> bool {
        matches!(w.kind, Kind::Button | Kind::Card | Kind::Checkbox | Kind::Toggle | Kind::Radio | Kind::Eigen
            | Kind::TextInput | Kind::TextArea | Kind::Dropdown | Kind::Spinner | Kind::Slider | Kind::Knob
            | Kind::ListBox | Kind::Table | Kind::Tree | Kind::ColorPicker | Kind::DatePicker | Kind::TimePicker)
    }

    /// Hat das Widget einen Namen fuer den Bildschirmleser? Dieselbe Regel
    /// wie `a11y_widget`: eigener Text, sonst die Beschriftung daneben,
    /// sonst der Hilfetext.
    pub(super) fn a11y_hat_namen(&self, wi: usize, i: usize) -> bool {
        let w = &self.windows[wi].widgets[i];
        if !w.tooltip.trim().is_empty() { return true; }
        match w.kind {
            Kind::Button | Kind::Card | Kind::Checkbox | Kind::Toggle | Kind::Radio | Kind::Eigen => !w.text.trim().is_empty(),
            _ => self.a11y_beschriftung(wi, i).is_some(),
        }
    }

    /// Schrift und Grund, wie gezeichnet -- None, wenn das Widget keine
    /// Schrift zeigt, die zu pruefen waere.
    fn schrift_und_grund(&self, w: &Widget) -> Option<(i64, i64)> {
        let fenster = self.th("win_bg");
        match w.kind {
            Kind::Button | Kind::Card => {
                if w.text.trim().is_empty() { return None; }
                let fg = self.knopf_text(w);
                // Umriss, flach und Verweis haben keine Flaeche.
                let bg = if matches!(w.variante, 5 | 6 | 7) && self.knopf_grund(w).is_none() { fenster }
                    else { self.knopf_grund(w).unwrap_or_else(|| self.wcol(w, "bg", "widget_bg")) };
                // Ein Farbverlauf: gemessen gegen seine unguenstigste Farbe.
                let bg = match self.w_verlauf_farben(w) {
                    Some(f) => f.into_iter().min_by(|&a, &b| kontrast(fg, a & 0xFFFFFF).total_cmp(&kontrast(fg, b & 0xFFFFFF))).unwrap_or(bg),
                    None => bg,
                };
                Some((fg, bg))
            }
            Kind::Label => {
                if w.text.trim().is_empty() { return None; }
                let fg = if let Some(c) = w.ov.get("fg") { *c } else if w.color >= 0 { w.color }
                    else { self.stil_misch(w, "fg", self.stil_farbe(w, "fg", stil::BASIS).unwrap_or_else(|| self.th("text_fg"))) };
                let bg = w.ov.get("bg").copied().or_else(|| self.stil_farbe(w, "bg", stil::BASIS)).unwrap_or(fenster);
                Some((fg, bg))
            }
            Kind::Checkbox | Kind::Toggle | Kind::Radio => {
                if w.text.trim().is_empty() { return None; }
                Some((self.txt_col(w), fenster))
            }
            Kind::TextInput | Kind::TextArea => Some((self.txt_col(w), self.wcol(w, "bg", "win_bg"))),
            _ => None,
        }
    }

    /// Spruenge der Tab-Folge gegen die Lesereihenfolge: (von, nach) je
    /// Paar aufeinanderfolgender Widgets (der Kreis vom letzten zum ersten
    /// zaehlt nicht).
    pub(super) fn tab_spruenge(&self, wi: usize) -> Vec<(usize, usize)> {
        let folge = self.tab_reihenfolge(wi);
        let rtl = self.windows[wi].rtl;
        let lage = |k: usize| self.abs_rect(wi, &self.windows[wi].widgets[k]);
        folge.windows(2).filter_map(|p| {
            let ((ax, ay, ab, ah), (bx, by, bb, bh)) = (lage(p[0]), lage(p[1]));
            // Liegt b ganz vor a in Leserichtung (links, gespiegelt rechts)?
            let davor = if rtl { bx >= ax + ab } else { bx + bb <= ax };
            // ... oder beginnt es eine neue Spalte dahinter?
            let neue_spalte = if rtl { bx + bb <= ax } else { bx >= ax + ab };
            let darueber = by + bh <= ay;
            let selbe_zeile = by < ay + ah && ay < by + bh;
            ((darueber && !neue_spalte) || (selbe_zeile && davor)).then_some((p[0], p[1]))
        }).collect()
    }

    /// Die Befunde zu einem Widget (leer = in Ordnung). `ziele` sind die
    /// Rechtecke aller bedienbaren Widgets des Fensters (fuer den Abstand),
    /// `spruenge` die der Tab-Folge (`tab_spruenge`).
    pub(super) fn a11y_befunde(&self, wi: usize, i: usize, ziele: &[(usize, (i32, i32, i32, i32))], spruenge: &[(usize, usize)]) -> Vec<String> {
        let w = &self.windows[wi].widgets[i];
        let mut b = Vec::new();
        if !w.alive || !self.widget_shown(wi, w) { return b; }
        for &(von, _) in spruenge.iter().filter(|(_, nach)| *nach == i) {
            b.push(format!("Tab springt hierher zurueck (von {}) -- die Reihenfolge folgt nicht der Lage: Widgets in Lesereihenfolge anlegen oder GUI_SET_TAB_INDEX",
                self.a11y_bezeichnung(wi, &self.windows[wi].widgets[von])));
        }
        if w.enabled {
            if let Some((fg, bg)) = self.schrift_und_grund(w) {
                let k = kontrast(fg & 0xFFFFFF, bg & 0xFFFFFF);
                let gross = self.w_groesse(w) >= GROSSE_SCHRIFT;
                let soll = if gross { KONTRAST_GROSS } else { KONTRAST_TEXT };
                if k < soll {
                    b.push(format!("Kontrast {:.1}:1 -- mindestens {}:1 (Schrift #{:06X} auf #{:06X})",
                        k, if gross { "3" } else { "4,5" }, fg & 0xFFFFFF, bg & 0xFFFFFF).replace('.', ","));
                }
            }
        }
        if Self::bedienbar(w) {
            if !self.a11y_hat_namen(wi, i) {
                b.push("kein Name fuer den Bildschirmleser -- Text, eine Beschriftung davor (GUI_LABEL) oder GUI_TOOLTIP".into());
            }
            if w.enabled {
                let (x, y, bw, bh) = self.abs_rect(wi, w);
                let (lb, lh) = (self.unsk(bw), self.unsk(bh));
                if lb < ZIEL || lh < ZIEL {
                    // Ausnahme fuer Abstand: ein Kreis von 24 Punkten um die
                    // Mitte beruehrt kein anderes Ziel.
                    let r = self.sk(ZIEL) as f64 / 2.0;
                    let (cx, cy) = (x as f64 + bw as f64 / 2.0, y as f64 + bh as f64 / 2.0);
                    let stoesst = ziele.iter().filter(|(k, _)| *k != i).any(|&(_, (ox, oy, ob, oh))| {
                        let dx = (ox as f64 - cx).max(0.0).max(cx - (ox + ob) as f64);
                        let dy = (oy as f64 - cy).max(0.0).max(cy - (oy + oh) as f64);
                        dx * dx + dy * dy < r * r
                    });
                    if stoesst {
                        b.push(format!("Ziel {} x {} Punkte, zu dicht an einem anderen -- mindestens {} x {} oder mehr Abstand", lb, lh, ZIEL, ZIEL));
                    }
                }
            }
        }
        b
    }

    /// Die Rechtecke aller sichtbaren, bedienbaren Widgets eines Fensters.
    pub(super) fn a11y_ziele(&self, wi: usize) -> Vec<(usize, (i32, i32, i32, i32))> {
        self.windows[wi].widgets.iter().enumerate()
            .filter(|(_, w)| w.alive && w.enabled && Self::bedienbar(w) && self.widget_shown(wi, w))
            .map(|(k, w)| (k, self.abs_rect(wi, w)))
            .collect()
    }

    /// Wie ein Widget in einem Befund heisst: Art, Name oder Text, Fenster.
    fn a11y_bezeichnung(&self, wi: usize, w: &Widget) -> String {
        let wer = if !w.name.is_empty() { format!("#{}", w.name) }
            else if !w.text.trim().is_empty() { format!("\"{}\"", w.text.lines().next().unwrap_or("").chars().take(30).collect::<String>()) }
            else { format!("bei {},{}", self.unsk(w.x), self.unsk(w.y)) };
        format!("{} {} in \"{}\"", w.kind.as_str(), wer, self.windows[wi].title)
    }

    /// GUI_A11Y_CHECK$([win]): alle Befunde, eine Zeile je Befund
    /// ("" = nichts gefunden); ohne Fenster alle sichtbaren Fenster.
    pub fn a11y_check(&self, nur: Option<i64>) -> Result<String, String> {
        if let Some(h) = nur {
            if h < 0 || h as usize >= self.windows.len() { return Err("GUI_A11Y_CHECK$: ungueltiges GUI_WINDOW-Handle".into()); }
        }
        let mut zeilen = Vec::new();
        for wi in 0..self.windows.len() {
            if nur.is_some_and(|h| h as usize != wi) { continue; }
            let win = &self.windows[wi];
            if !win.alive || !win.visible { continue; }
            let ziele = self.a11y_ziele(wi);
            let spruenge = self.tab_spruenge(wi);
            for i in 0..win.widgets.len() {
                for b in self.a11y_befunde(wi, i, &ziele, &spruenge) {
                    zeilen.push(format!("{}: {}", self.a11y_bezeichnung(wi, &win.widgets[i]), b));
                }
            }
        }
        Ok(zeilen.join("\n"))
    }
}
