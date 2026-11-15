//! Der Zustand der gui als Text (GUI_STATE$) -- fuer Pruefungen und fuer
//! `dhrt aufnehmen` (aufnahme.rs), das ihn als Erwartung eines Falls
//! schreibt.
//!
//! Teil von `gui.rs`. Je sichtbarem Fenster eine Kopfzeile, darunter je
//! Widget mit einem Zustand eine Zeile: Texte von Feldern und Beschriftungen,
//! Haken, Werte, Auswahl. Knoepfe, Behaelter und Zierde haben keinen Zustand
//! und fehlen. Ein Widget heisst nach seinem Namen (`#name`), sonst nach
//! seiner Nummer unter den Widgets derselben Art im Fenster (`(2)`).

use super::*;

/// Ein Text in einer Zeile: Zeilenumbrueche sichtbar, lang gekuerzt.
fn kurz(s: &str) -> String {
    let t = s.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t").replace('\r', "");
    if t.chars().count() > 120 { format!("{}...", t.chars().take(120).collect::<String>()) } else { t }
}

/// Eine Zahl ohne unnoetige Nachkommastellen.
fn zahl(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 { format!("{}", v as i64) }
    else { format!("{:.3}", v).trim_end_matches('0').trim_end_matches('.').to_string() }
}

impl Gui {
    /// GUI_STATE$(): der Zustand aller sichtbaren Fenster, je Zeile ein
    /// Widget.
    pub fn state_text(&self) -> String {
        let mut z: Vec<String> = Vec::new();
        for (wi, win) in self.windows.iter().enumerate() {
            if !win.alive || !win.visible || win.entwurf { continue; }
            z.push(if win.name.is_empty() { format!("fenster \"{}\"", kurz(&win.title)) } else { format!("fenster #{}", win.name) });
            let mut zaehler: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
            for (i, w) in win.widgets.iter().enumerate() {
                if !w.alive { continue; }
                let art = w.kind.as_str();
                let nr = { let n = zaehler.entry(art).or_insert(0); *n += 1; *n };
                if !self.widget_shown(wi, w) { continue; }
                let h = Self::enc_widget(wi, i);
                let wer = if w.name.is_empty() { format!("{}({})", art, nr) } else { format!("{}#{}", art, w.name) };
                let zeile = match w.kind {
                    Kind::Label => format!("{} \"{}\"", wer, kurz(&w.text)),
                    Kind::TextInput | Kind::TextArea => {
                        let t = self.text(h).unwrap_or_default();
                        let t = if w.passwort { "*".repeat(t.chars().count()) } else { t };
                        format!("{} \"{}\"", wer, kurz(&t))
                    }
                    Kind::Checkbox | Kind::Toggle | Kind::Radio =>
                        format!("{} \"{}\" {}", wer, kurz(&w.text), if w.checked { "an" } else { "aus" }),
                    Kind::Slider | Kind::Spinner | Kind::Knob | Kind::Progress => format!("{} {}", wer, zahl(w.value)),
                    Kind::Dropdown | Kind::ListBox => {
                        let k = self.dropdown_selected(h).unwrap_or(-1);
                        let t = if k >= 0 { w.items.get(k as usize).map(|s| kurz(s)).unwrap_or_default() } else { String::new() };
                        format!("{} {} \"{}\"", wer, k, t)
                    }
                    _ => continue,
                };
                z.push(format!("  {}", zeile));
            }
        }
        z.join("\n")
    }
}
