//! Native OS-Datei-/Ordner-Dialoge (rfd). Nur im graphics-Build.
//!
//! Alle Funktionen sind BLOCKIEREND (modaler System-Dialog) und liefern den
//! gewaehlten Pfad als String -- bei Abbruch einen leeren String. `exts` ist
//! eine Liste von Dateiendungen ohne Punkt (z.B. ["png", "jpg"]).

/// "png,jpg, .gif" -> ["png", "jpg", "gif"] (Punkte/Whitespace toleriert).
pub fn parse_exts(s: &str) -> Vec<String> {
    s.split(',')
        .map(|e| e.trim().trim_start_matches('.').to_string())
        .filter(|e| !e.is_empty())
        .collect()
}

fn base(title: &str, exts: &[String]) -> rfd::FileDialog {
    let mut d = rfd::FileDialog::new();
    if !title.is_empty() {
        d = d.set_title(title);
    }
    if !exts.is_empty() {
        d = d.add_filter("Dateien", exts);
    }
    d
}

/// Datei zum OEFFNEN waehlen.
pub fn open(title: &str, exts: &[String]) -> String {
    base(title, exts)
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Datei zum SPEICHERN waehlen (mit optionalem Default-Namen).
pub fn save(title: &str, default_name: &str, exts: &[String]) -> String {
    let mut d = base(title, exts);
    if !default_name.is_empty() {
        d = d.set_file_name(default_name);
    }
    d.save_file()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Ordner waehlen.
pub fn folder(title: &str) -> String {
    let mut d = rfd::FileDialog::new();
    if !title.is_empty() {
        d = d.set_title(title);
    }
    d.pick_folder()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Modale Info-MessageBox (nur OK).
pub fn message(title: &str, text: &str) {
    rfd::MessageDialog::new()
        .set_title(title)
        .set_description(text)
        .set_level(rfd::MessageLevel::Info)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

/// Modaler Bestaetigen-Dialog (OK/Abbrechen) -> true bei OK.
pub fn confirm(title: &str, text: &str) -> bool {
    matches!(
        rfd::MessageDialog::new()
            .set_title(title)
            .set_description(text)
            .set_level(rfd::MessageLevel::Warning)
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show(),
        rfd::MessageDialogResult::Ok
    )
}
