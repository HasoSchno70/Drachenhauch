//! Textkodierungen jenseits von UTF-8 (Punkt 3 aus docs/allzweck-audit-2.md).
//!
//! Bis hierher war jede Textdatei UTF-8, sonst gar nicht lesbar:
//!
//! ```text
//! READLINES("kunden.csv")  ->  stream did not contain valid UTF-8
//! ```
//!
//! Genau diese Datei schreibt Excel auf einem deutschen Windows, wenn man
//! „CSV (Trennzeichen-getrennt)" waehlt: **Windows-1252**. Die haeufigste
//! Herkunft von Daten, die jemand auswerten will, war also die eine, die
//! nicht durch die Tuer passte.
//!
//! **Bewusst ohne Fremd-Crate.** `encoding_rs` kann alles, wiegt aber schwer
//! fuer zwei Ein-Byte-Kodierungen, deren Tabellen feststehen und oeffentlich
//! sind. Der Preis dafuer ist, dass die Tabelle stimmen MUSS -- sie wird
//! deshalb in `tests/test_kodierung.py` Byte fuer Byte gegen Pythons eigene
//! Codecs geprueft, nicht gegen sich selbst.
//!
//! **Nicht dabei: UTF-16.** Excel schreibt es bei „Unicode Text (*.txt)".
//! Es braucht BOM-Erkennung, zwei Byte-Reihenfolgen und Ersatzpaare -- das
//! ist eine eigene Entscheidung und kein Nachtrag zu dieser hier.

/// Die unterstuetzten Kodierungen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kodierung {
    Utf8,
    /// Windows-1252 -- was Excel und Notepad unter „ANSI" schreiben.
    Cp1252,
    /// ISO-8859-1: jedes Byte ist sein eigener Codepunkt. Kann nie scheitern.
    Latin1,
}

/// Alle Namen, die eine Kodierung annimmt -- fuer die Fehlermeldung an einer
/// Stelle, statt in jedem Builtin neu aufgezaehlt.
pub const NAMEN: &str = "utf8, cp1252 (= ansi, windows-1252), latin1 (= iso-8859-1)";

/// Namen aus dem GB-Programm auf die Kodierung abbilden.
///
/// Gross-/Kleinschreibung und Bindestriche sind egal (`UTF-8`, `utf8`,
/// `Windows-1252` -- alles derselbe Name); wer eine Kodierung angibt, hat sie
/// meist irgendwo abgeschrieben.
pub fn aus_name(name: &str, fn_: &str) -> Result<Kodierung, String> {
    let k: String = name.chars()
        .filter(|c| !matches!(c, '-' | '_' | ' '))
        .flat_map(|c| c.to_lowercase())
        .collect();
    match k.as_str() {
        "utf8" | "utf" => Ok(Kodierung::Utf8),
        "cp1252" | "windows1252" | "win1252" | "ansi" | "1252" => Ok(Kodierung::Cp1252),
        "latin1" | "iso88591" | "latin" | "88591" => Ok(Kodierung::Latin1),
        _ => Err(format!("{}: Kodierung '{}' kenne ich nicht -- moeglich sind: {}",
                         fn_, name, NAMEN)),
    }
}

/// Windows-1252 in den Bytes 0x80..0x9F. Alles darunter ist ASCII, alles
/// darueber deckt sich mit Latin-1.
///
/// Die fuenf Bytes, die Windows-1252 offiziell NICHT belegt (0x81, 0x8D,
/// 0x8F, 0x90, 0x9D), stehen hier als ihr eigener Codepunkt -- so macht es
/// die WHATWG-Norm, die jeder Browser umsetzt. Die Alternative waere ein
/// Fehler (so macht es Python), und der waere hier falsch: wer eine alte
/// Datei einliest, will sie lesen und nicht an einem Steuerzeichen scheitern,
/// das ohnehin niemand gemeint hat.
const CP1252_HOCH: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

/// Bytes zu Text. `pfad` erscheint in der Fehlermeldung (leer = weglassen).
pub fn dekodieren(bytes: &[u8], k: Kodierung, fn_: &str, pfad: &str)
    -> Result<String, String>
{
    match k {
        Kodierung::Utf8 => match std::str::from_utf8(bytes) {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(utf8_meldung(fn_, pfad, bytes, e.valid_up_to())),
        },
        Kodierung::Latin1 => Ok(bytes.iter().map(|&b| b as char).collect()),
        Kodierung::Cp1252 => Ok(bytes.iter().map(|&b| {
            if (0x80..0xA0).contains(&b) { CP1252_HOCH[(b - 0x80) as usize] }
            else { b as char }
        }).collect()),
    }
}

/// Die Meldung, um die es bei diesem ganzen Modul geht.
///
/// Vorher stand da wortwoertlich „stream did not contain valid UTF-8" -- ein
/// durchgereichter Rust-Fehler, der weder sagt, was los ist, noch was man tun
/// kann. Jetzt nennt sie die Stelle, die wahrscheinliche Ursache und den
/// Ausweg.
fn utf8_meldung(fn_: &str, pfad: &str, bytes: &[u8], bis: usize) -> String {
    let wo = if pfad.is_empty() { String::new() } else { format!("{}: ", pfad) };
    // Die Zeile ist fuer den Menschen nuetzlicher als der Byte-Versatz.
    let zeile = bytes[..bis].iter().filter(|&&b| b == b'\n').count() + 1;
    let byte = bytes.get(bis).copied().unwrap_or(0);
    format!("{}: {}Zeile {} ist kein UTF-8 (Byte 0x{:02X} an Stelle {}). \
             Kommt die Datei aus Excel oder einem alten Windows-Programm, ist sie \
             meist cp1252 -- dann als letztes Argument \"cp1252\" mitgeben. \
             Moeglich sind: {}",
            fn_, wo, zeile, byte, bis, NAMEN)
}

/// Text zu Bytes.
///
/// Latin-1 und Windows-1252 kennen nur 256 Zeichen. Ein Zeichen, das dort
/// fehlt, ist ein FEHLER und wird nicht durch `?` ersetzt: eine Rechnung, in
/// der aus „€" still ein Fragezeichen wird, ist schlimmer als eine, die gar
/// nicht erst geschrieben wird.
pub fn kodieren(text: &str, k: Kodierung, fn_: &str) -> Result<Vec<u8>, String> {
    match k {
        Kodierung::Utf8 => Ok(text.as_bytes().to_vec()),
        Kodierung::Latin1 => {
            let mut raus = Vec::with_capacity(text.len());
            for c in text.chars() {
                let n = c as u32;
                if n > 0xFF { return Err(unmappbar(fn_, c, "latin1")); }
                raus.push(n as u8);
            }
            Ok(raus)
        }
        Kodierung::Cp1252 => {
            let mut raus = Vec::with_capacity(text.len());
            for c in text.chars() {
                if let Some(i) = CP1252_HOCH.iter().position(|&t| t == c) {
                    raus.push(0x80 + i as u8);
                    continue;
                }
                let n = c as u32;
                // 0x80..0x9F sind in cp1252 die Tabelle oben -- ein Text, der
                // diese Codepunkte direkt enthaelt, darf nicht auf sie
                // abgebildet werden (ausser den fuenf unbelegten, die die
                // Tabelle selbst auf sich zeigen laesst und die oben schon
                // getroffen haben).
                if n <= 0xFF && !(0x80..0xA0).contains(&n) {
                    raus.push(n as u8);
                    continue;
                }
                return Err(unmappbar(fn_, c, "cp1252"));
            }
            Ok(raus)
        }
    }
}

fn unmappbar(fn_: &str, c: char, k: &str) -> String {
    format!("{}: das Zeichen '{}' (U+{:04X}) gibt es in {} nicht -- \
             die Datei in utf8 schreiben oder das Zeichen ersetzen",
            fn_, c, c as u32, k)
}

/// Ein fuehrendes BOM abschneiden.
///
/// Excel schreibt es, und ohne dieses Abschneiden heisst die erste Spalte
/// fuer immer `\u{feff}Name`. Stand vorher nur in `CSV_LOAD`; jetzt gilt es
/// fuer jeden Textleser, denn dieselbe Datei liest man mal so und mal so.
pub fn ohne_bom(s: String) -> String {
    match s.strip_prefix('\u{feff}') {
        Some(rest) => rest.to_string(),
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namen_sind_nachsichtig() {
        for n in ["utf8", "UTF-8", "Utf 8"] {
            assert_eq!(aus_name(n, "T").unwrap(), Kodierung::Utf8);
        }
        for n in ["cp1252", "CP-1252", "windows-1252", "ANSI"] {
            assert_eq!(aus_name(n, "T").unwrap(), Kodierung::Cp1252);
        }
        for n in ["latin1", "Latin-1", "ISO-8859-1"] {
            assert_eq!(aus_name(n, "T").unwrap(), Kodierung::Latin1);
        }
        let e = aus_name("klingonisch", "T").unwrap_err();
        assert!(e.contains("kenne ich nicht") && e.contains("cp1252"), "{}", e);
    }

    #[test]
    fn latin1_ist_die_identitaet_auf_bytes() {
        let bytes: Vec<u8> = (0u8..=255).collect();
        let s = dekodieren(&bytes, Kodierung::Latin1, "T", "").unwrap();
        assert_eq!(s.chars().count(), 256);
        assert_eq!(s.chars().nth(0xE4).unwrap(), 'ä');
        // ... und zurueck ergibt wieder dieselben Bytes.
        assert_eq!(kodieren(&s, Kodierung::Latin1, "T").unwrap(), bytes);
    }

    #[test]
    fn cp1252_hin_und_zurueck() {
        let bytes: Vec<u8> = (0u8..=255).collect();
        let s = dekodieren(&bytes, Kodierung::Cp1252, "T", "").unwrap();
        assert_eq!(kodieren(&s, Kodierung::Cp1252, "T").unwrap(), bytes);
        assert!(s.contains('€'));      // 0x80
        assert!(s.contains('ä'));      // 0xE4
    }

    #[test]
    fn euro_passt_nicht_in_latin1() {
        let e = kodieren("Preis: 5 €", Kodierung::Latin1, "T").unwrap_err();
        assert!(e.contains("U+20AC") && e.contains("utf8"), "{}", e);
    }

    #[test]
    fn kein_stilles_ersatzzeichen() {
        // Ein Zeichen, das in KEINER der beiden Ein-Byte-Kodierungen liegt.
        assert!(kodieren("Smiley 😀", Kodierung::Cp1252, "T").is_err());
        assert!(kodieren("Smiley 😀", Kodierung::Latin1, "T").is_err());
    }

    #[test]
    fn utf8_meldung_nennt_zeile_und_ausweg() {
        let mut bytes = b"erste\nzweite\n".to_vec();
        bytes.push(0xE4);              // ein einzelnes Latin-1-'ae'
        let e = dekodieren(&bytes, Kodierung::Utf8, "READLINES", "k.csv").unwrap_err();
        assert!(e.contains("Zeile 3"), "{}", e);
        assert!(e.contains("0xE4"), "{}", e);
        assert!(e.contains("cp1252"), "{}", e);
        assert!(e.contains("k.csv"), "{}", e);
    }

    #[test]
    fn bom_faellt_weg() {
        assert_eq!(ohne_bom("\u{feff}Name".to_string()), "Name");
        assert_eq!(ohne_bom("Name".to_string()), "Name");
    }
}
