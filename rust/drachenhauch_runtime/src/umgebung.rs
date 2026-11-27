//! Umgebungsvariablen der Laufzeit: EINE Leitlinie (Bericht B8).
//!
//! Bis 2026.53 gab es drei Vorsilben (`DHRT_`, `DH_`) und zwei Sprachen
//! (`DHRT_FONT` neben `DHRT_SCHRIFT_ORDNER`, `DHRT_SCALE` neben
//! `DHRT_NATIV_MASSSTAB`, `KEIN_` neben `OHNE_`). Jetzt: Vorsilbe `DHRT_`,
//! englisch. Die alten Namen liest die Laufzeit weiter -- Skripte, CI und
//! Pruefsammlungen, die sie setzen, laufen unveraendert.
//!
//! Wer eine Variable liest, nimmt `var`/`var_os` von hier mit dem NEUEN
//! Namen; die Tafel sagt, welcher alte dazugehoert.

use std::ffi::OsString;

/// `(neu, alt)`.
pub const UMBENANNT: &[(&str, &str)] = &[
    ("DHRT_PATH", "DH_PATH"),
    ("DHRT_LOG", "DH_LOG"),
    ("DHRT_FONT_DIR", "DHRT_SCHRIFT_ORDNER"),
    ("DHRT_NATIVE_SCALE", "DHRT_NATIV_MASSSTAB"),
    ("DHRT_FRAME_END", "DHRT_BILD_ENDE"),
    ("DHRT_NO_WINDOW", "DHRT_KEIN_FENSTER"),
    ("DHRT_NO_GRAPHICS", "DHRT_OHNE_GRAFIK"),
    ("DHRT_NO_INPUT", "DHRT_OHNE_EINGABE"),
    ("DHRT_NO_POPUP_WINDOWS", "DHRT_OHNE_POPUP_FENSTER"),
    ("DHRT_NO_NATIVE_WINDOWS", "DHRT_OHNE_NATIVE_FENSTER"),
    ("DHRT_NO_FUSION", "DHRT_OHNE_VERSCHMELZEN"),
    ("DHRT_SYSTEM_DARK", "DHRT_SYSTEM_DUNKEL"),
    ("DHRT_SYSTEM_ACCENT", "DHRT_SYSTEM_AKZENT"),
    ("DHRT_SYSTEM_HIGH_CONTRAST", "DHRT_SYSTEM_HOHER_KONTRAST"),
    ("DHRT_SYSTEM_REDUCED_MOTION", "DHRT_SYSTEM_WENIGER_BEWEGUNG"),
    ("DHRT_CLIPBOARD", "DHRT_ZWISCHENABLAGE"),
    ("DHRT_DICTIONARIES", "DHRT_WOERTERBUECHER"),
    ("DHRT_CHECK_TYPES", "DHRT_TYPEN_PRUEFEN"),
    ("DHRT_TYPES_LIST", "DHRT_TYPEN_LISTE"),
    ("DHRT_OPEN_LOG", "DHRT_OEFFNEN_PROTOKOLL"),
    ("DHRT_MICROPHONE", "DHRT_MIKROFON"),
    ("DHRT_DEVICES", "DHRT_GERAETE"),
    ("DHRT_INSPECTOR", "DHRT_INSPEKTOR"),
    ("DHRT_GUI_STATE", "DHRT_GUI_ZUSTAND"),
    ("DHRT_GUI_STATE_FILE", "DHRT_GUI_ZUSTAND_DATEI"),
    ("DHRT_PLAY_VERSION", "DHRT_FASSUNG_SPIELEN"),
    ("DHRT_RECORD", "DHRT_AUFNEHMEN"),
    ("DHRT_PLAYBACK", "DHRT_ABSPIELEN"),
    ("DHRT_OFFSCREEN", "DHRT_ABSEITS"),
    ("DHRT_DROP", "DHRT_ABLEGEN"),
    ("DHRT_JIT_STATS", "DHRT_JIT_BILANZ"),
];

fn alt(neu: &str) -> Option<&'static str> {
    UMBENANNT.iter().find(|(n, _)| *n == neu).map(|(_, a)| *a)
}

/// Wie `std::env::var_os`, mit dem alten Namen als Rueckfall.
pub fn var_os(neu: &str) -> Option<OsString> {
    std::env::var_os(neu).or_else(|| alt(neu).and_then(std::env::var_os))
}

/// Wie `std::env::var`, mit dem alten Namen als Rueckfall.
pub fn var(neu: &str) -> Result<String, std::env::VarError> {
    match std::env::var(neu) {
        Err(std::env::VarError::NotPresent) => match alt(neu) {
            Some(a) => std::env::var(a),
            None => Err(std::env::VarError::NotPresent),
        },
        r => r,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leitlinie() {
        for (n, a) in UMBENANNT {
            assert!(n.starts_with("DHRT_"), "{}", n);
            assert!(n.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'), "{}", n);
            assert_ne!(n, a);
            assert!(alt(a).is_none(), "{} ist selbst ein neuer Name", a);
        }
    }
}
