//! Die Einstellungen der Widgets: `GUI_<W>_SET(w, key$, value)` und
//! `GUI_<W>_GET(w, key$)` nach EINER Regel (Bericht A18).
//!
//! Bis 2026.53 hatte jede der 21 Arten ihre eigene: Mehrfachauswahl hiess
//! „multi“ oder „multi_select“, Akkordeon und Assistent unterschieden
//! Gross/klein, mal nahm der Wert eine Zahl, mal ein BOOLEAN, und nur drei
//! liessen sich abfragen -- mit drei Rueckgabetypen. Jetzt:
//!
//! * Schluessel in jeder Schreibweise, deutsch oder englisch (die Tafeln
//!   hier); ein unbekannter nennt alle in derselben Form.
//! * Der Wert ist eine Zahl oder ein BOOLEAN (`vm_gui::gwert`).
//! * Jede Art hat ihre Abfrage; sie liefert eine Zahl (ein Schalter 1/0).
//!
//! Die Setter selbst stehen bei ihren Widgets, die Abfragen hier (bis auf
//! die der Bauzusaetze, deren Felder `gui_bausteine.rs` gehoeren).

use super::*;

/// (deutsch, englisch). Ein deutscher Schluessel darf mehrere englische
/// Zweitnamen haben -- dann steht er mehrmals da.
pub(super) type Tafel = &'static [(&'static str, &'static str)];

pub(super) const LISTBOX: Tafel = &[("mehrfachauswahl", "multi_select"), ("mehrfachauswahl", "multi"),
    ("kaestchen", "checkboxes"), ("bearbeitbar", "editable")];
pub(super) const TREE: Tafel = LISTBOX;
pub(super) const FILETREE: Tafel = &[("ordner_zuerst", "dirs_first"), ("verborgene", "hidden"),
    ("nur_ordner", "dirs_only"), ("klick_klappt", "click_expands"), ("nach_art", "by_type"),
    ("mehrfachauswahl", "multi_select"), ("mehrfachauswahl", "multi"), ("kaestchen", "checkboxes"),
    ("auffrischen", "refresh")];
pub(super) const ACCORDION: Tafel = &[("mehrere", "multiple")];
pub(super) const WIZARD: Tafel = &[("pruefen", "validate")];
pub(super) const TABCONTROL: Tafel = &[("schliessbar", "closable")];
pub(super) const TIMEPICKER: Tafel = &[("sekunden", "seconds"), ("schritt", "step")];
pub(super) const SLIDER: Tafel = &[("marken", "ticks"), ("wert_zeigen", "draw_value"), ("einrasten", "snap"),
    ("stellen", "digits")];
pub(super) const DROPDOWN: Tafel = &[("bearbeitbar", "editable")];
pub(super) const PROGRESS: Tafel = &[("unbestimmt", "indeterminate"), ("kreis", "circle")];
pub(super) const LAYOUT: Tafel = &[("abstand", "gap"), ("abstand", "spacing"), ("rand", "padding"),
    ("ausrichtung", "align"), ("dehnen", "stretch"), ("rahmen", "frame"), ("auswahl", "selection")];
pub(super) const TEXTINPUT: Tafel = &[("passwort", "password"), ("nur_lesen", "readonly"),
    ("maxlaenge", "maxlen"), ("zahlen", "numeric"), ("suche", "search")];
pub(super) const RICHTEXT: Tafel = &[("groesse", "size"), ("codeschrift", "code_font"),
    ("farbe_kommentar", "color_comment"), ("farbe_text", "color_text"), ("farbe_zahl", "color_number"),
    ("farbe_schluessel", "color_keyword"), ("farbe_name", "color_name"), ("farbe_operator", "color_operator")];
pub(super) const TOOLBAR: Tafel = &[("beschriftung", "text"), ("symbolgroesse", "symbol")];
pub(super) const COLORPICKER: Tafel = &[("deckkraft", "alpha")];
pub(super) const DATEPICKER: Tafel = &[("wochenbeginn", "week_start")];
pub(super) const LABEL: Tafel = &[("kuerzen", "ellipsize"), ("auswaehlbar", "selectable")];
pub(super) const TEXTAREA: Tafel = &[("zeilennummern", "line_numbers"), ("aktive_zeile", "active_line"),
    ("tab_fuegt_ein", "tab_inserts"), ("tabbreite", "tab_width"), ("umbruch", "wrap"),
    ("formatiert", "formatted"), ("auto_einzug", "auto_indent"), ("einzugslinien", "indent_guides"),
    ("tab_meldet", "tab_reports"), ("kopfzeilen", "sticky_headers"), ("hinweise_klickbar", "hints_clickable")];
pub(super) const TABLE: Tafel = &[("zeilenhoehe", "row_height"), ("kopfhoehe", "header_height"),
    ("zebra", "zebra"), ("gitter", "grid"), ("filterzeile", "filter_row"), ("sortierbar", "sortable"),
    ("spalten_ziehbar", "resizable_cols"), ("feste_spalten", "frozen"),
    ("spalten_verschiebbar", "reorderable"), ("baum", "tree"), ("zellmodus", "cells"),
    ("zeilen_anhaengen", "append_rows"), ("mehrfachauswahl", "multi_select"), ("mehrfachauswahl", "multi"),
    ("spalten", "columns")];

/// Ein Schluessel: Gross/klein egal, deutsch oder englisch; geliefert wird
/// der deutsche. Unbekannt: eine Meldung, die alle nennt (deutsch/englisch).
pub(super) fn schluessel(fn_: &str, key: &str, tafel: Tafel) -> Result<&'static str, String> {
    let k = key.trim().to_lowercase();
    if let Some((de, _)) = tafel.iter().find(|(de, en)| *de == k || *en == k) { return Ok(de); }
    let mut liste: Vec<String> = Vec::new();
    for (de, en) in tafel {
        if liste.iter().any(|x| x == de || x.starts_with(&format!("{}/", de))) { continue; }
        liste.push(if de == en { de.to_string() } else { format!("{}/{}", de, en) });
    }
    Err(format!("{}: unbekannter Schluessel '{}' -- moeglich sind {}", fn_, key, liste.join(", ")))
}

fn b(x: bool) -> f64 { if x { 1.0 } else { 0.0 } }

impl Gui {
    /// `GUI_<ART>_GET(w, key$)`: `art` ist der Teil des Befehlsnamens
    /// (`listbox`, `tree` ...).
    pub fn einstellung_get(&self, art: &str, h: i64, key: &str) -> Result<f64, String> {
        let f = format!("GUI_{}_GET", art.to_uppercase());
        let f = f.as_str();
        Ok(match art {
            "listbox" => {
                let k = schluessel(f, key, LISTBOX)?;
                let w = self.list_ref(h, f)?;
                let l = w.list.as_deref();
                match k {
                    "mehrfachauswahl" => b(l.map(|l| l.multi).unwrap_or(false)),
                    "kaestchen" => b(l.map(|l| l.kaestchen).unwrap_or(false)),
                    _ => b(l.map(|l| l.bearbeitbar).unwrap_or(false)),
                }
            }
            "tree" | "filetree" => {
                if art == "filetree" {
                    let k = schluessel(f, key, FILETREE)?;
                    let (t, d) = self.ft_ref(h, f)?;
                    return Ok(match k {
                        "ordner_zuerst" => b(d.ordner_zuerst),
                        "verborgene" => b(d.verborgene),
                        "nur_ordner" => b(d.nur_ordner),
                        "klick_klappt" => b(d.klick_klappt),
                        "nach_art" => b(d.nach_art),
                        "mehrfachauswahl" => b(t.multi),
                        "kaestchen" => b(t.kaestchen),
                        _ => (d.auffrischen * 1000.0).round(),
                    });
                }
                let k = schluessel(f, key, TREE)?;
                let t = self.tree_ref(h, f)?;
                match k { "mehrfachauswahl" => b(t.multi), "kaestchen" => b(t.kaestchen), _ => b(t.bearbeitbar) }
            }
            "accordion" => {
                schluessel(f, key, ACCORDION)?;
                b(self.akk_ref(h, f)?.akk.as_ref().map(|a| a.mehrere).unwrap_or(false))
            }
            "wizard" => {
                schluessel(f, key, WIZARD)?;
                b(self.wz_ref(h, f)?.wz.as_ref().map(|z| z.pruefen).unwrap_or(false))
            }
            "tabcontrol" => {
                schluessel(f, key, TABCONTROL)?;
                b(self.tc_ref(h, f)?.tabctl.as_ref().map(|s| s.schliessbar).unwrap_or(false))
            }
            "timepicker" => {
                let k = schluessel(f, key, TIMEPICKER)?;
                let w = self.wdg(h, f)?;
                if w.kind != Kind::TimePicker { return Err(format!("{}: das Widget ist kein GUI_TIMEPICKER", f)); }
                if k == "sekunden" { b(w.zeit_sek) } else { w.step }
            }
            "dropdown" => {
                schluessel(f, key, DROPDOWN)?;
                let w = self.wdg(h, f)?;
                if w.kind != Kind::Dropdown { return Err(format!("{}: das Widget ist kein GUI_DROPDOWN", f)); }
                b(w.frei)
            }
            "layout" => {
                let k = schluessel(f, key, LAYOUT)?;
                let w = self.wdg(h, f)?;
                let l = w.layout.as_deref().filter(|_| w.kind == Kind::Layout)
                    .ok_or_else(|| format!("{}: das Widget ist kein GUI_LAYOUT", f))?;
                match k {
                    "abstand" => l.abstand as f64,
                    "rand" => l.rand as f64,
                    "ausrichtung" => l.ausrichtung as f64,
                    "dehnen" => b(l.dehnen),
                    "rahmen" => b(l.rahmen),
                    _ => b(l.auswahl),
                }
            }
            "richtext" => {
                let k = schluessel(f, key, RICHTEXT)?;
                let r = self.rt_ref(h, f)?;
                match k {
                    "groesse" => r.basis as f64,
                    "codeschrift" => r.code_font as f64,
                    _ => {
                        let n = RICHTEXT.iter().skip(2).position(|(de, _)| *de == k).unwrap_or(0);
                        r.code_farben[n].map(|c| c as f64).unwrap_or(-1.0)
                    }
                }
            }
            "colorpicker" => {
                schluessel(f, key, COLORPICKER)?;
                let w = self.wdg(h, f)?;
                if w.kind != Kind::ColorPicker { return Err(format!("{}: das Widget ist kein GUI_COLORPICKER", f)); }
                b(w.alpha_an)
            }
            "datepicker" => {
                schluessel(f, key, DATEPICKER)?;
                let w = self.wdg(h, f)?;
                if w.kind != Kind::DatePicker { return Err(format!("{}: das Widget ist kein GUI_DATEPICKER", f)); }
                w.wochenbeginn as f64
            }
            "textarea" => {
                let k = schluessel(f, key, TEXTAREA)?;
                let w = self.wdg(h, f)?;
                if w.kind != Kind::TextArea { return Err(format!("{}: das Widget ist kein GUI_TEXTAREA", f)); }
                match k {
                    "zeilennummern" => b(w.zeilennummern),
                    "aktive_zeile" => b(w.aktive_zeile),
                    "tab_fuegt_ein" => b(w.tab_fuegt_ein),
                    "tabbreite" => w.tabbreite as f64,
                    "umbruch" => b(w.umbruch),
                    "formatiert" => b(w.formatiert),
                    "auto_einzug" => b(w.auto_einzug),
                    "einzugslinien" => b(w.einzugslinien),
                    "tab_meldet" => b(w.tab_meldet),
                    "kopfzeilen" => w.kopfzeilen as f64,
                    _ => b(w.hinweise_klickbar),
                }
            }
            "slider" | "label" | "progress" => return self.bau_einstellung(art, h, key),
            "toolbar" => return self.toolbar_get(h, key),
            "textinput" => return self.textinput_get(h, key),
            "table" => return self.table_get_opt(h, key),
            _ => return Err(format!("{}: unbekannte Art", f)),
        })
    }
}

impl Gui {
    /// `GUI_<ART>_PLACEHOLDER(w, text$)`: der Hinweis, solange das Widget leer
    /// ist -- fuer jede Art, die einen hat, mit seiner Abfrage
    /// `GUI_<ART>_PLACEHOLDER$` (Bericht A18; bis 2026.53 liess sich der
    /// Hinweis mal nur setzen, mal nur lesen, beim Textfeld nur beim Anlegen).
    pub fn platzhalter_set(&mut self, art: &str, h: i64, text: String) -> Result<(), String> {
        let f = format!("GUI_{}_PLACEHOLDER", art.to_uppercase());
        match art {
            "listbox" => self.list_mut(h, &f)?.leer_text = text,
            "tree" => self.tree_mut(h, &f)?.leer_text = text,
            "table" => self.tbl_mut(h, &f)?.leer_text = text,
            _ => { self.platzhalter_widget(art, h, &f)?; self.wdg_mut(h, &f)?.placeholder = text; }
        }
        Ok(())
    }
    pub fn platzhalter_get(&self, art: &str, h: i64) -> Result<String, String> {
        let f = format!("GUI_{}_PLACEHOLDER$", art.to_uppercase());
        Ok(match art {
            "listbox" => self.list_ref(h, &f)?.list.as_ref().map(|l| l.leer_text.clone()).unwrap_or_default(),
            "tree" => self.tree_ref(h, &f)?.leer_text.clone(),
            "table" => self.tbl_ref(h, &f)?.leer_text.clone(),
            _ => { self.platzhalter_widget(art, h, &f)?; self.wdg(h, &f)?.placeholder.clone() }
        })
    }
    /// Textfeld, Textbereich, Klappliste: der Hinweis steht am Widget.
    fn platzhalter_widget(&self, art: &str, h: i64, f: &str) -> Result<(), String> {
        let w = self.wdg(h, f)?;
        let soll = match art { "textinput" => Kind::TextInput, "textarea" => Kind::TextArea, _ => Kind::Dropdown };
        // Das Textfeld nimmt auch einen Textbereich (wie bisher GUI_TEXTINPUT_PLACEHOLDER$).
        if w.kind == soll || (art == "textinput" && w.kind == Kind::TextArea) { return Ok(()); }
        Err(format!("{}: das Widget ist kein GUI_{}", f, art.to_uppercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schluessel_jede_schreibweise() {
        assert_eq!(schluessel("F", "Multi_Select", LISTBOX), Ok("mehrfachauswahl"));
        assert_eq!(schluessel("F", "MEHRERE", ACCORDION), Ok("mehrere"));
        let e = schluessel("GUI_LISTBOX_SET", "x", LISTBOX).unwrap_err();
        assert_eq!(e, "GUI_LISTBOX_SET: unbekannter Schluessel 'x' -- moeglich sind mehrfachauswahl/multi_select, kaestchen/checkboxes, bearbeitbar/editable");
        assert!(schluessel("F", "q", TABLE).unwrap_err().contains("zebra, gitter/grid"));
    }
}
