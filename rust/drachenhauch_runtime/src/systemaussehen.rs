//! Das Aussehen des Betriebssystems: dunkler Modus und Akzentfarbe.
//!
//! `GUI_SYSTEM_DARK()`, `GUI_SYSTEM_ACCENT()` und `GUI_THEME_PRESET("system")`
//! fragen hier. Windows liest die Registry (`AppsUseLightTheme`,
//! `DWM\AccentColor`), macOS `defaults read -g`, Linux `gsettings`
//! (GNOME; `color-scheme`, `accent-color`, sonst ein `-dark` im GTK-Thema).
//! Was sich nicht feststellen laesst, ist `None` -- das Programm entscheidet
//! dann selbst.
//!
//! Fuer Pruefungen und zum Ausprobieren legen zwei Umgebungsvariablen die
//! Antwort fest: `DHRT_SYSTEM_DUNKEL=1|0` und `DHRT_SYSTEM_AKZENT=#RRGGBB`
//! (leer oder `-` = unbekannt). Unter macOS und Linux startet eine Abfrage
//! ein Programm; die Antwort gilt darum zwei Sekunden, damit ein Programm,
//! das je Bild fragt, nicht je Bild einen Prozess startet.

#![allow(dead_code)]

use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aussehen {
    pub dunkel: Option<bool>,
    /// 0xRRGGBB
    pub akzent: Option<i64>,
}

static MERK: Mutex<Option<(Instant, Aussehen)>> = Mutex::new(None);
const GILT: Duration = Duration::from_secs(2);

/// Das Aussehen jetzt (hoechstens zwei Sekunden alt).
pub fn jetzt() -> Aussehen {
    let mut a = {
        let mut m = MERK.lock().unwrap_or_else(|e| e.into_inner());
        match *m {
            Some((t, a)) if t.elapsed() < GILT => a,
            _ => {
                let a = vom_system();
                *m = Some((Instant::now(), a));
                a
            }
        }
    };
    // Die Umgebung gewinnt -- und wird nicht gemerkt, damit ein Test sie
    // zwischen zwei Faellen umstellen kann.
    if let Ok(v) = std::env::var("DHRT_SYSTEM_DUNKEL") { a.dunkel = dunkel_aus_text(&v); }
    if let Ok(v) = std::env::var("DHRT_SYSTEM_AKZENT") { a.akzent = farbe_aus_text(&v); }
    a
}

fn dunkel_aus_text(v: &str) -> Option<bool> {
    match v.trim().to_lowercase().as_str() {
        "1" | "ja" | "true" | "dunkel" | "dark" => Some(true),
        "0" | "nein" | "false" | "hell" | "light" => Some(false),
        _ => None,
    }
}

/// `#RRGGBB` oder `RRGGBB`; alles andere ist unbekannt.
fn farbe_aus_text(v: &str) -> Option<i64> {
    let t = v.trim().trim_start_matches('#');
    if t.len() != 6 { return None; }
    i64::from_str_radix(t, 16).ok()
}

/// Windows speichert die Akzentfarbe als 0xAABBGGRR.
pub fn abgr_zu_rgb(v: u32) -> i64 {
    let (r, g, b) = (v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF);
    ((r << 16) | (g << 8) | b) as i64
}

/// macOS `AppleAccentColor`: eine Nummer statt einer Farbe; fehlt der
/// Schluessel, ist es Blau. Die Werte sind die Systemfarben von macOS.
pub fn mac_akzent(nummer: Option<i32>) -> i64 {
    match nummer {
        Some(-1) => 0x8C8C8C, // Graphit
        Some(0) => 0xFF5257,  // Rot
        Some(1) => 0xF7821B,  // Orange
        Some(2) => 0xFFC600,  // Gelb
        Some(3) => 0x62BA46,  // Gruen
        Some(5) => 0xA550A7,  // Lila
        Some(6) => 0xF74F9E,  // Rosa
        _ => 0x007AFF,        // Blau (Vorgabe)
    }
}

/// GNOME `accent-color` (seit GNOME 47) -- die Farben von libadwaita.
pub fn gnome_akzent(name: &str) -> Option<i64> {
    Some(match name.trim().trim_matches('\'') {
        "blue" => 0x3584E4,
        "teal" => 0x2190A4,
        "green" => 0x3A944A,
        "yellow" => 0xC88800,
        "orange" => 0xED5B00,
        "red" => 0xE62D42,
        "pink" => 0xD56199,
        "purple" => 0x9141AC,
        "slate" => 0x6F8396,
        _ => return None,
    })
}

/// GNOME `color-scheme`: `'prefer-dark'` ist dunkel, `'default'` und
/// `'prefer-light'` hell.
pub fn gnome_dunkel(antwort: &str) -> Option<bool> {
    match antwort.trim().trim_matches('\'') {
        "prefer-dark" => Some(true),
        "default" | "prefer-light" => Some(false),
        _ => None,
    }
}

#[cfg(windows)]
fn vom_system() -> Aussehen {
    fn dword(schluessel: &str, wert: &str) -> Option<u32> {
        use windows::core::HSTRING;
        use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
        let mut daten: u32 = 0;
        let mut groesse: u32 = 4;
        let r = unsafe {
            RegGetValueW(HKEY_CURRENT_USER, &HSTRING::from(schluessel), &HSTRING::from(wert),
                RRF_RT_REG_DWORD, None, Some(&mut daten as *mut u32 as *mut _), Some(&mut groesse))
        };
        if r.is_ok() { Some(daten) } else { None }
    }
    let hell = dword(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", "AppsUseLightTheme");
    let akzent = dword(r"Software\Microsoft\Windows\DWM", "AccentColor").map(abgr_zu_rgb);
    Aussehen { dunkel: hell.map(|h| h == 0), akzent }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn befehl(prog: &str, args: &[&str]) -> Option<String> {
    let aus = std::process::Command::new(prog).args(args)
        .stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null()).output().ok()?;
    if !aus.status.success() { return None; }
    Some(String::from_utf8_lossy(&aus.stdout).trim().to_string())
}

#[cfg(target_os = "macos")]
fn vom_system() -> Aussehen {
    // Ohne den Schluessel (Fehler) ist macOS hell.
    let dunkel = Some(befehl("defaults", &["read", "-g", "AppleInterfaceStyle"])
        .map(|s| s.eq_ignore_ascii_case("dark")).unwrap_or(false));
    let nummer = befehl("defaults", &["read", "-g", "AppleAccentColor"]).and_then(|s| s.parse().ok());
    Aussehen { dunkel, akzent: Some(mac_akzent(nummer)) }
}

#[cfg(target_os = "linux")]
fn vom_system() -> Aussehen {
    let schema = "org.gnome.desktop.interface";
    let mut dunkel = befehl("gsettings", &["get", schema, "color-scheme"]).and_then(|s| gnome_dunkel(&s));
    if dunkel.is_none() {
        dunkel = befehl("gsettings", &["get", schema, "gtk-theme"]).map(|s| s.to_lowercase().contains("dark"));
    }
    let akzent = befehl("gsettings", &["get", schema, "accent-color"]).and_then(|s| gnome_akzent(&s));
    Aussehen { dunkel, akzent }
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn vom_system() -> Aussehen { Aussehen { dunkel: None, akzent: None } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_farbe_ist_abgr() {
        // Windows-Blau 0xFFD77800 (A=FF, B=D7, G=78, R=00) -> #0078D7
        assert_eq!(abgr_zu_rgb(0xFFD7_7800), 0x0078D7);
        assert_eq!(abgr_zu_rgb(0x0000_00FF), 0xFF0000);
    }

    #[test]
    fn mac_ohne_schluessel_ist_blau() {
        assert_eq!(mac_akzent(None), 0x007AFF);
        assert_eq!(mac_akzent(Some(4)), 0x007AFF);
        assert_eq!(mac_akzent(Some(0)), 0xFF5257);
        assert_eq!(mac_akzent(Some(-1)), 0x8C8C8C);
    }

    #[test]
    fn gnome_antworten() {
        assert_eq!(gnome_dunkel("'prefer-dark'"), Some(true));
        assert_eq!(gnome_dunkel("'default'"), Some(false));
        assert_eq!(gnome_dunkel("Kein solches Schema"), None);
        assert_eq!(gnome_akzent("'teal'"), Some(0x2190A4));
        assert_eq!(gnome_akzent("'magenta'"), None);
    }

    #[test]
    fn umgebung_texte() {
        assert_eq!(dunkel_aus_text("1"), Some(true));
        assert_eq!(dunkel_aus_text("hell"), Some(false));
        assert_eq!(dunkel_aus_text("-"), None);
        assert_eq!(farbe_aus_text("#FF8800"), Some(0xFF8800));
        assert_eq!(farbe_aus_text("ff8800"), Some(0xFF8800));
        assert_eq!(farbe_aus_text("-"), None);
        assert_eq!(farbe_aus_text("#F80"), None);
    }
}
