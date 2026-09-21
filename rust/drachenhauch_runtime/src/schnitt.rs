//! Schriftschnitte: fett, kursiv, unterstrichen, durchgestrichen.
//!
//! Zwei reine Teile, ohne raylib und darum ueberall pruefbar:
//! `stil_parsen` liest die Schreibweise der Befehle (`"fett+kursiv"`,
//! `"bold, underline"`), `schnitt_kandidaten` sagt, welche Dateien zu einer
//! geladenen Schrift den fetten oder kursiven Schnitt tragen koennten.
//!
//! Warum Dateien: raylib zeichnet Glyphen aus EINER Schriftdatei, und aus
//! einer normalen Strichstaerke laesst sich keine fette rechnen -- eine
//! echte fette Segoe UI sieht anders aus als eine zweimal gezeichnete. Wo
//! kein Schnitt zu finden ist, faellt die Zeichenroutine auf den Ersatz
//! zurueck (fett = zweiter Zug, kursiv = geneigte Glyphen).

/// Bits eines Stils. Fett und kursiv waehlen einen Schnitt, unterstrichen
/// und durchgestrichen sind Linien -- die gibt es in jeder Schrift.
pub const FETT: u8 = 1;
pub const KURSIV: u8 = 2;
pub const UNTER: u8 = 4;
pub const DURCH: u8 = 8;
/// Die beiden Bits, die einen Schnitt waehlen.
pub const SCHNITT: u8 = FETT | KURSIV;

/// `"fett+kursiv"`, `"bold, underline"`, `"normal"`, `""` -> Bits.
/// Getrennt wird an `+`, `,`, Leerzeichen und `-` (so geht auch
/// `"fett-kursiv"` wie bei den PDF-Schriften). Ein unbekanntes Wort ist ein
/// Fehler mit der Liste -- ein Tippfehler wuerde sonst still normal zeichnen.
pub fn stil_parsen(s: &str) -> Result<u8, String> {
    let mut bits = 0u8;
    for teil in s.split(|c: char| c == '+' || c == ',' || c == ' ' || c == '-' || c == '|') {
        let t = teil.trim().to_lowercase();
        if t.is_empty() { continue; }
        bits |= match t.as_str() {
            "normal" | "regular" | "standard" => 0,
            "fett" | "bold" | "b" => FETT,
            "kursiv" | "italic" | "schraeg" | "schräg" | "i" => KURSIV,
            "unterstrichen" | "underline" | "unter" | "u" => UNTER,
            "durchgestrichen" | "strikethrough" | "durch" | "strike" | "s" => DURCH,
            _ => return Err(format!(
                "unbekannter Schriftstil '{}' (erlaubt: normal, fett, kursiv, unterstrichen, durchgestrichen -- verbunden mit + oder Komma)", teil.trim())),
        };
    }
    Ok(bits)
}

/// Die Bits wieder als Text, in fester Reihenfolge -- fuer Getter und die
/// `.dhform`. `0` ist `"normal"`.
pub fn stil_text(bits: u8) -> String {
    let mut t: Vec<&str> = Vec::new();
    if bits & FETT != 0 { t.push("fett"); }
    if bits & KURSIV != 0 { t.push("kursiv"); }
    if bits & UNTER != 0 { t.push("unterstrichen"); }
    if bits & DURCH != 0 { t.push("durchgestrichen"); }
    if t.is_empty() { "normal".into() } else { t.join("+") }
}

/// Bekannte Familien, deren Schnitte keiner Regel folgen: Windows benennt
/// sie mit angehaengten Buchstaben (segoeui -> segoeuib/segoeuii/segoeuiz).
/// Reihenfolge je Eintrag: fett, kursiv, fett+kursiv.
const FAMILIEN: &[(&str, &str, &str, &str)] = &[
    ("segoeui", "segoeuib", "segoeuii", "segoeuiz"),
    ("arial", "arialbd", "ariali", "arialbi"),
    ("consola", "consolab", "consolai", "consolaz"),
    ("times", "timesbd", "timesi", "timesbi"),
    ("calibri", "calibrib", "calibrii", "calibriz"),
    ("verdana", "verdanab", "verdanai", "verdanaz"),
    ("georgia", "georgiab", "georgiai", "georgiaz"),
    ("cour", "courbd", "couri", "courbi"),
    ("tahoma", "tahomabd", "", ""),
    ("trebuc", "trebucbd", "trebucit", "trebucbi"),
    ("cambria", "cambriab", "cambriai", "cambriaz"),
    ("candara", "candarab", "candarai", "candaraz"),
    ("corbel", "corbelb", "corbeli", "corbelz"),
    ("constan", "constanb", "constani", "constanz"),
    ("lucon", "", "", ""),
];

/// Dateien, die zu `pfad` den Schnitt `bits & SCHNITT` tragen koennten, in
/// der Reihenfolge, in der sie versucht werden. Ob es sie gibt, prueft der
/// Aufrufer -- so bleibt die Regel ohne Dateisystem pruefbar.
///
/// Drei Wege: (1) die bekannten Familien oben; (2) Namen mit `-Regular`,
/// `Regular` oder `-Roman` am Ende (DejaVu, Noto, Liberation, Roboto, die
/// meisten Schriften aus dem Netz) bekommen `Bold`/`Italic`/`BoldItalic`,
/// ersatzweise `Oblique`/`BoldOblique`; (3) sonst wird `-Bold` usw. an den
/// Stamm gehaengt (`DejaVuSans.ttf` -> `DejaVuSans-Bold.ttf`).
pub fn schnitt_kandidaten(pfad: &str, bits: u8) -> Vec<String> {
    let bits = bits & SCHNITT;
    if bits == 0 { return Vec::new(); }
    let p = std::path::Path::new(pfad);
    let ordner = p.parent().map(|x| x.to_path_buf()).unwrap_or_default();
    let stamm = p.file_stem().and_then(|x| x.to_str()).unwrap_or("").to_string();
    let endung = p.extension().and_then(|x| x.to_str()).unwrap_or("ttf").to_string();
    let datei = |name: &str| -> String {
        ordner.join(format!("{}.{}", name, endung)).to_string_lossy().replace('\\', "/")
    };
    let mut raus: Vec<String> = Vec::new();
    let klein = stamm.to_lowercase();
    for (basis, b, i, z) in FAMILIEN {
        if klein == *basis {
            let name = match bits { FETT => *b, KURSIV => *i, _ => *z };
            if !name.is_empty() {
                // Die Gross-/Kleinschreibung der Vorlage uebernehmen
                // (ARIAL.TTF -> ARIALBD.TTF), sonst findet Linux die Datei nicht.
                let name = if stamm.chars().all(|c| !c.is_lowercase()) { name.to_uppercase() } else { name.to_string() };
                raus.push(datei(&name));
            }
            return raus;
        }
    }
    let (wurzel, trenner) = if let Some(r) = stamm.strip_suffix("-Regular") { (r.to_string(), "-") }
        else if let Some(r) = stamm.strip_suffix("_Regular") { (r.to_string(), "_") }
        else if let Some(r) = stamm.strip_suffix("-Roman") { (r.to_string(), "-") }
        else if let Some(r) = stamm.strip_suffix("Regular") { (r.to_string(), "") }
        else { (stamm.clone(), "-") };
    let namen: &[&str] = match bits {
        FETT => &["Bold", "Semibold", "SemiBold"],
        KURSIV => &["Italic", "Oblique"],
        _ => &["BoldItalic", "BoldOblique", "Bold Italic"],
    };
    for n in namen {
        raus.push(datei(&format!("{}{}{}", wurzel, trenner, n)));
    }
    raus
}

/// Wie weit der zweite Zug beim Ersatz-Fett versetzt wird: ein Punkt bei
/// Lesegroesse, mit der Schrift wachsend -- sonst wirkt eine Ueberschrift
/// in 40 Punkten gar nicht fett.
pub fn fett_versatz(groesse: f32) -> f32 { (groesse / 16.0).max(1.0).floor() }

/// Neigung beim Ersatz-Kursiv: waagerechter Versatz je senkrechtem Punkt.
/// 0.2 entspricht gut 11 Grad, der uebliche Winkel echter Kursivschriften.
pub const NEIGUNG: f32 = 0.2;

/// Lage und Staerke der Linien, gerechnet von der OBERKANTE des Textes
/// (so zaehlt TEXT seine y-Lage). Eine Stelle fuer Zeichnen und Pruefen.
pub fn linie_unter(groesse: f32) -> (f32, f32) { (groesse * 0.9, (groesse / 14.0).max(1.0)) }
pub fn linie_durch(groesse: f32) -> (f32, f32) { (groesse * 0.52, (groesse / 14.0).max(1.0)) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stile_lesen() {
        assert_eq!(stil_parsen("").unwrap(), 0);
        assert_eq!(stil_parsen("normal").unwrap(), 0);
        assert_eq!(stil_parsen("fett").unwrap(), FETT);
        assert_eq!(stil_parsen("Fett+Kursiv").unwrap(), FETT | KURSIV);
        assert_eq!(stil_parsen("fett-kursiv").unwrap(), FETT | KURSIV);
        assert_eq!(stil_parsen("bold, italic, underline").unwrap(), FETT | KURSIV | UNTER);
        assert_eq!(stil_parsen("durchgestrichen").unwrap(), DURCH);
        assert_eq!(stil_parsen("schräg").unwrap(), KURSIV);
        let e = stil_parsen("fet").unwrap_err();
        assert!(e.contains("'fet'") && e.contains("kursiv"), "{}", e);
    }

    #[test]
    fn stil_als_text() {
        assert_eq!(stil_text(0), "normal");
        assert_eq!(stil_text(FETT | UNTER), "fett+unterstrichen");
        assert_eq!(stil_parsen(&stil_text(FETT | KURSIV | DURCH)).unwrap(), FETT | KURSIV | DURCH);
    }

    #[test]
    fn familien_von_windows() {
        assert_eq!(schnitt_kandidaten("C:/Windows/Fonts/segoeui.ttf", FETT), vec!["C:/Windows/Fonts/segoeuib.ttf"]);
        assert_eq!(schnitt_kandidaten("C:/Windows/Fonts/segoeui.ttf", KURSIV), vec!["C:/Windows/Fonts/segoeuii.ttf"]);
        assert_eq!(schnitt_kandidaten("C:/Windows/Fonts/segoeui.ttf", FETT | KURSIV | UNTER), vec!["C:/Windows/Fonts/segoeuiz.ttf"]);
        assert_eq!(schnitt_kandidaten("C:/Windows/Fonts/consola.ttf", FETT), vec!["C:/Windows/Fonts/consolab.ttf"]);
        assert_eq!(schnitt_kandidaten("C:/Windows/Fonts/ARIAL.TTF", FETT), vec!["C:/Windows/Fonts/ARIALBD.TTF"]);
        // Tahoma hat keinen Kursivschnitt -- keine Kandidaten, also Ersatz.
        assert!(schnitt_kandidaten("C:/Windows/Fonts/tahoma.ttf", KURSIV).is_empty());
        // Nur Linien: kein Schnitt gefragt.
        assert!(schnitt_kandidaten("C:/Windows/Fonts/segoeui.ttf", UNTER).is_empty());
    }

    #[test]
    fn namen_mit_regular_und_ohne() {
        assert_eq!(schnitt_kandidaten("/usr/share/fonts/Noto/NotoSans-Regular.ttf", FETT)[0],
                   "/usr/share/fonts/Noto/NotoSans-Bold.ttf");
        assert_eq!(schnitt_kandidaten("fonts/Roboto-Regular.ttf", KURSIV)[0], "fonts/Roboto-Italic.ttf");
        assert_eq!(schnitt_kandidaten("fonts/Roboto-Regular.ttf", FETT | KURSIV)[0], "fonts/Roboto-BoldItalic.ttf");
        let k = schnitt_kandidaten("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", KURSIV);
        assert!(k.contains(&"/usr/share/fonts/truetype/dejavu/DejaVuSans-Oblique.ttf".to_string()), "{:?}", k);
        assert_eq!(schnitt_kandidaten("Foo.otf", FETT)[0], "Foo-Bold.otf");
    }

    #[test]
    fn versatz_und_linien() {
        assert_eq!(fett_versatz(12.0), 1.0);
        assert_eq!(fett_versatz(32.0), 2.0);
        let (u, d) = (linie_unter(20.0), linie_durch(20.0));
        assert!(u.0 > d.0 && u.0 < 20.0 && u.1 >= 1.0);
    }
}
