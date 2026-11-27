//! Was steckt in DIESER Laufzeit? Eine Tabelle fuer alle, die danach fragen.
//!
//! Bis 2026.53 wusste das jede Stelle selbst: `MODULE_AVAILABLE` kannte nur
//! die Hardware-Module (und sagte in `dhrt-mini` TRUE zu db, net, pdf ...),
//! die IMPORT-Warnung ebenso, jede Feature-Huelle hatte ihren eigenen Satz
//! (acht Wortlaute), und `dhrt --fehlende` erkannte nur vier davon -- httpd
//! nicht, also exportierte `--export` ein Programm mit HTTPD_START in die
//! Mini-Laufzeit, und es brach beim ersten Aufruf ab.
//!
//! Jetzt: [`FEATURES`] sagt, welche Cargo-Features dabei sind, [`MODULE`]
//! welches Modul welche braucht. Daraus speisen sich `dhrt --version`,
//! `MODULE_AVAILABLE`, die Warnung beim IMPORT, die EINE Meldung
//! „NAME: gibt es in dieser Laufzeit nicht (ohne netz gebaut)“ und
//! [`fehlt_im_bau`].

/// Ein Cargo-Feature: sein Name, wie `dhrt --version` es nennt, und ob es
/// in diesem Bau steckt.
pub(crate) struct Feature {
    pub name: &'static str,
    pub anzeige: &'static str,
    pub da: bool,
}

/// Die Features, nach denen ein Programm fragen kann -- in der Reihenfolge
/// von `dhrt --version`.
pub(crate) const FEATURES: &[Feature] = &[
    Feature { name: "graphics", anzeige: if cfg!(feature = "gles") { "grafik (OpenGL ES)" } else { "grafik" }, da: cfg!(feature = "graphics") },
    Feature { name: "dialogs", anzeige: "dialoge", da: cfg!(feature = "dialogs") },
    Feature { name: "tray", anzeige: "tray", da: cfg!(feature = "tray") },
    Feature { name: "video", anzeige: "video", da: cfg!(feature = "video") },
    Feature { name: "db", anzeige: "datenbank", da: cfg!(feature = "db") },
    Feature { name: "pg", anzeige: "postgres", da: cfg!(feature = "pg") },
    Feature { name: "mysql", anzeige: "mysql", da: cfg!(feature = "mysql") },
    Feature { name: "net", anzeige: "netz", da: cfg!(feature = "net") },
    Feature { name: "http", anzeige: "http", da: cfg!(feature = "http") },
    Feature { name: "smtp", anzeige: "mail", da: cfg!(feature = "smtp") },
    Feature { name: "jit", anzeige: "maschinencode", da: cfg!(feature = "jit") },
    Feature { name: "ffi", anzeige: "ffi", da: cfg!(feature = "ffi") },
    Feature { name: "pdf", anzeige: "pdf", da: cfg!(feature = "pdf") },
    Feature { name: "schriften", anzeige: "schriften", da: cfg!(feature = "schriften") },
    Feature { name: "physik", anzeige: "physik", da: cfg!(feature = "physik") },
    Feature { name: "serial", anzeige: "seriell", da: cfg!(feature = "serial") },
    Feature { name: "usb", anzeige: "usb", da: cfg!(feature = "usb") },
    Feature { name: "bt", anzeige: "bluetooth", da: cfg!(feature = "bt") },
    Feature { name: "wifi", anzeige: "wlan", da: cfg!(feature = "wifi") },
    Feature { name: "midi", anzeige: "midi", da: cfg!(feature = "midi") },
];

/// Die Features, die `rust/build_runtime.py` nur mit `--hardware` baut.
const HARDWARE: &[&str] = &["serial", "usb", "bt", "wifi", "midi"];

/// Modul -> die Features, ohne die KEIN Befehl des Moduls geht. Module, die
/// hier fehlen, stecken in jeder Laufzeit (oder fehlen nur zum Teil: midi
/// ohne `midi` kann Dateien und Noten, firmata ohne `serial` geht ueber TCP,
/// m3d rechnet ohne Grafik, sprite/particles/chart zeichnen nur nicht).
/// Gemessen 2026-11-25 mit `dhrt --fehlende` in einem Bau mit nur `jit ffi`
/// ueber alle Befehle des Index.
pub(crate) const MODULE: &[(&str, &[&str])] = &[
    ("audio", &["graphics"]),
    ("gui", &["graphics"]),
    ("ui", &["graphics"]),
    ("imgfx", &["graphics"]),
    ("g3d", &["graphics"]),
    ("camera", &["graphics"]),
    ("input", &["graphics"]),
    ("webcam", &["graphics"]),
    ("video", &["graphics", "video"]),
    ("db", &["db"]),
    ("net", &["net"]),
    ("mqtt", &["net"]),
    ("websocket", &["net"]),
    ("httpd", &["net"]),
    ("html", &["http"]),
    ("cloud", &["http"]),
    ("smtp", &["smtp"]),
    ("pdf", &["pdf"]),
    ("physics2d", &["physik"]),
    ("physics3d", &["physik"]),
    ("serial", &["serial"]),
    ("usb", &["usb"]),
    ("bt", &["bt"]),
    ("wifi", &["wifi"]),
];

fn feature(name: &str) -> Option<&'static Feature> {
    FEATURES.iter().find(|f| f.name == name)
}

/// Steckt dieses Feature in der Laufzeit? Unbekannte Namen: nein.
pub(crate) fn da(name: &str) -> bool {
    feature(name).is_some_and(|f| f.da)
}

/// Das erste Feature, das dem Modul fehlt (None: es ist ganz da).
pub(crate) fn modul_fehlt(modul: &str) -> Option<&'static str> {
    let m = modul.to_lowercase();
    let m = crate::namen::modul(&m);
    MODULE.iter().find(|(n, _)| *n == m)
        .and_then(|(_, fs)| fs.iter().copied().find(|f| !da(f)))
}

/// Das Feature, das diesem Befehl fehlt (None: er ist da, oder es haengt
/// nicht an einem Feature). Gefragt wird das Modul des Befehls im Index.
pub(crate) fn befehl_fehlt(name: &str) -> Option<&'static str> {
    let low = name.to_lowercase();
    if let Some(m) = crate::compiler::modul_von_builtin(&low) {
        if let Some(f) = modul_fehlt(&m) { return Some(f); }
        // midi: Ein- und Ausgaenge, Abspielen (alles ausser Dateien und Noten).
        if m == "midi" && !da("midi") && !midi_ohne_geraet(&low) { return Some("midi"); }
    }
    None
}

/// Die Teile von midi, die ohne das Feature gehen (vm_geraete.rs `try_midi`).
fn midi_ohne_geraet(name: &str) -> bool {
    matches!(name, "midi_note_name$" | "midi_note_name" | "midi_note_freq")
        || (name.starts_with("midi_file_")
            && !matches!(name, "midi_file_play" | "midi_file_stop" | "midi_file_playing" | "midi_file_position"))
}

/// Laeuft dieses dhrt aus einem Bauordner von cargo? Nur dort hilft der
/// Satz „neu bauen mit python rust/build_runtime.py“ -- in einer
/// installierten oder exportierten Laufzeit nicht. Erkannt am Ordner
/// `.fingerprint` neben der Exe (auch mit `CARGO_TARGET_DIR=C:/dht`).
fn entwicklerbau() -> bool {
    static JA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *JA.get_or_init(|| std::env::current_exe().ok()
        .and_then(|p| p.parent().map(|d| d.join(".fingerprint").is_dir()))
        .unwrap_or(false))
}

/// Der Schalter fuer `build_runtime.py`, der das Feature zurueckbringt.
fn schalter(feature: &str) -> &'static str {
    if HARDWARE.contains(&feature) { " --hardware" } else { "" }
}

/// Wie das Feature in Meldungen heisst.
fn anzeige(feature: &str) -> &'static str {
    self::feature(feature).map(|f| f.anzeige).unwrap_or("?")
}

/// DIE Meldung fuer einen Befehl, den diese Laufzeit nicht hat.
pub(crate) fn fehlt_meldung(befehl: &str, feature: &str) -> String {
    if entwicklerbau() {
        format!("{}: gibt es in dieser Laufzeit nicht (ohne {} gebaut) -- neu bauen mit: python rust/build_runtime.py{}",
                befehl.to_uppercase(), anzeige(feature), schalter(feature))
    } else {
        format!("{}: gibt es in dieser Laufzeit nicht (ohne {} gebaut)", befehl.to_uppercase(), anzeige(feature))
    }
}

/// Die Warnung beim IMPORT eines Moduls, das hier fehlt.
pub(crate) fn import_warnung(modul: &str, feature: &str) -> String {
    let teil = if modul == "midi" {
        // MIDI-Dateien und die Notenrechnung brauchen kein Geraet.
        format!("Modul 'midi' ist in dieser Laufzeit nur zum Teil enthalten -- MIDI-Dateien und \
                 MIDI_NOTE_NAME$/MIDI_NOTE_FREQ gehen, Ein- und Ausgaenge nicht (ohne {} gebaut)", anzeige(feature))
    } else {
        format!("Modul '{}' gibt es in dieser Laufzeit nicht (ohne {} gebaut) -- jeder Aufruf meldet einen Fehler",
                modul, anzeige(feature))
    };
    // Der Ausweg: im Entwicklerbau neu bauen; sonst (eine installierte oder
    // kopierte Laufzeit, auch die der CI) das Modul wahlweise benutzen.
    if entwicklerbau() {
        format!("{}. Neu bauen mit: python rust/build_runtime.py{}", teil, schalter(feature))
    } else {
        format!("{}. Wer ohne das Modul auskommt: IMPORT \"{}\" OPTIONAL und vorher MODULE_AVAILABLE(\"{}\") fragen", teil, modul, modul)
    }
}

/// Fehlt dem Modul in dieser Laufzeit etwas, wovor der IMPORT warnen soll?
/// (midi zaehlt mit, obwohl es nur zum Teil fehlt.)
pub(crate) fn import_fehlt(modul: &str) -> Option<&'static str> {
    if modul == "midi" && !da("midi") { return Some("midi"); }
    modul_fehlt(modul)
}

/// `dhrt --version`: was dabei ist und was fehlt.
pub(crate) fn version_zeilen() -> String {
    let an: Vec<&str> = FEATURES.iter().filter(|f| f.da).map(|f| f.anzeige).collect();
    let aus: Vec<&str> = FEATURES.iter().filter(|f| !f.da).map(|f| f.anzeige).collect();
    let mut s = format!("dabei: {}", if an.is_empty() { "nichts".to_string() } else { an.join(", ") });
    if !aus.is_empty() {
        let hw = FEATURES.iter().any(|f| !f.da && HARDWARE.contains(&f.name));
        let zusatz = if entwicklerbau() && hw { " (neu bauen mit: python rust/build_runtime.py --hardware)" } else { "" };
        // (`dhrt-mini` ohne Grafik: dort brachte --hardware nichts zurueck.)
        s.push_str(&format!("\nfehlt: {}{}", aus.join(", "), zusatz));
    }
    s
}

/// Sagt diese Meldung „den Befehl hat diese Laufzeit nicht“ (statt „falsche
/// Argumente“)? Fuer `dhrt --fehlende`, das die kleinen Laufzeiten fragt.
/// Die alten Wortlaute (bis 2026.53) werden eine Fassung lang mitgelesen:
/// eine kleine Laufzeit kann aelter sein als die, die sie fragt.
pub(crate) fn fehlt_im_bau(meldung: &str) -> bool {
    // Eine Meldung fuer alles, was der Bau nicht hat (fehlt_meldung). Die
    // alten Saetze ("noch nicht verfuegbar", "ist ohne das Feature", "hat
    // das Netz-Feature nicht") erzeugt seit 2026.54 nichts mehr.
    meldung.contains("gibt es in dieser Laufzeit nicht")
        || meldung.starts_with("Unbekannter Befehl")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jedes_feature_der_module_steht_in_der_liste() {
        for (m, fs) in MODULE {
            for f in *fs { assert!(feature(f).is_some(), "{}: Feature {} fehlt in FEATURES", m, f); }
        }
        for f in HARDWARE { assert!(feature(f).is_some(), "{}", f); }
    }

    #[test]
    fn die_meldung_wird_erkannt() {
        assert!(fehlt_im_bau(&fehlt_meldung("httpd_start", "net")));
        assert!(!fehlt_im_bau("httpd: dieser dhrt-Build hat das Netz-Feature nicht"));
        assert!(!fehlt_im_bau("HTTPD_START: erwartet INTEGER (Arg 1), erhalten NIL"));
    }
}
