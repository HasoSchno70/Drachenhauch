//! Rechts-nach-links: die Reihenfolge, in der ein Text auf dem Schirm steht
//! (Unicode-Bidi ueber `unicode-bidi`), und die verbundenen Formen der
//! arabischen Buchstaben.
//!
//! raylib zeichnet Glyphe fuer Glyphe nach Codepunkt -- eine Formung nach
//! Glyphennummern (wie `rustybuzz` sie liefert) passt nicht zu seinen
//! Glyphenatlanten. Die arabischen Buchstaben bekommen darum ihre
//! Darstellungsformen (Unicode-Block "Arabic Presentation Forms-B", fuer
//! Persisch ein paar aus "-A"): Anfangs-, Mittel-, End- und Einzelform und die
//! Lam-Alef-Ligaturen. Die haben eigene Codepunkte, also eigene Glyphen in
//! jedem Atlas, und die Schriften des Systems (Segoe UI, Arial, Tahoma,
//! DejaVu) tragen sie.
//!
//! Ein Text ohne Zeichen aus einem Rechts-nach-links-Block geht unberuehrt
//! durch (`braucht`), damit Messen und Zeichnen der gewoehnlichen Faelle
//! nicht langsamer werden.

use unicode_bidi::{Level, ParagraphBidiInfo};

/// Steht im Text ein Zeichen, das Reihenfolge oder Form aendern kann?
pub fn braucht(s: &str) -> bool {
    !s.is_ascii() && s.chars().any(rtl_bereich)
}

/// Dasselbe fuer eine Zeichenfolge.
pub fn braucht_zeichen(z: &[char]) -> bool {
    z.iter().any(|&c| rtl_bereich(c))
}

fn rtl_bereich(c: char) -> bool {
    matches!(c as u32, 0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF
                     | 0x10800..=0x10FFF | 0x1E800..=0x1EFFF)
}

/// Eine Zeile so, wie sie dasteht.
#[derive(Debug, Clone, PartialEq)]
pub struct Zeile {
    /// Die Zeichen in Anzeige-Reihenfolge, geformt und ohne Steuerzeichen
    /// der Richtung.
    pub text: Vec<char>,
    /// Je Anzeige-Zeichen das erste logische Zeichen, aus dem es stammt.
    pub log_von_vis: Vec<usize>,
    /// Je logisches Zeichen sein Anzeige-Zeichen; `None` fuer ein
    /// Steuerzeichen, das nicht zu sehen ist.
    pub vis_von_log: Vec<Option<usize>>,
    /// Je logisches Zeichen: steht es in einem Lauf von rechts nach links?
    pub rtl: Vec<bool>,
    /// Richtung des Absatzes.
    pub basis_rtl: bool,
}

/// Die Richtung eines Absatzes nach seinem ersten starken Zeichen
/// (`None` = keines).
pub fn basis(z: &[char]) -> Option<bool> {
    let s: String = z.iter().collect();
    if !braucht(&s) { return None; }
    let info = ParagraphBidiInfo::new(&s, None);
    if info.levels.is_empty() { return None; }
    // Ohne starkes Zeichen waehlt unicode-bidi links nach rechts; das ist
    // hier dasselbe wie "keine Angabe".
    Some(info.paragraph_level.is_rtl())
}

/// Eine Zeile (ohne Zeilenumbruch) in Anzeige-Reihenfolge bringen.
/// `basis_rtl`: Richtung des Absatzes, `None` = nach dem ersten starken Zeichen.
pub fn ordnen(zeile: &[char], basis_rtl: Option<bool>) -> Zeile {
    let n = zeile.len();
    // 1. Formen: aus logischen Zeichen geformte (ein Lam-Alef wird eins).
    let (geformt, herkunft) = formen(zeile);
    let mut zu_geformt = vec![0usize; n];
    for (k, &l) in herkunft.iter().enumerate() {
        zu_geformt[l] = k;
        // Das Alef einer Ligatur zeigt auf dieselbe Stelle.
        let ende = herkunft.get(k + 1).copied().unwrap_or(n);
        for x in l + 1..ende { zu_geformt[x] = k; }
    }
    // 2. Reihenfolge der geformten Zeichen.
    let s: String = geformt.iter().collect();
    let level = basis_rtl.map(|r| if r { Level::rtl() } else { Level::ltr() });
    let info = ParagraphBidiInfo::new(&s, level);
    let ganz_rtl = info.paragraph_level.is_rtl();
    let (stufen, folge): (Vec<Level>, Vec<usize>) = if s.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        // Je Zeichen, nach den Regeln fuer eine Zeile (L1).
        let stufen: Vec<Level> = info.reordered_levels_per_char(0..s.len());
        let folge = ParagraphBidiInfo::reorder_visual(&stufen);
        (stufen, folge)
    };
    // 3. Anzeige bauen: Steuerzeichen fallen weg, Klammern spiegeln sich in
    // Laeufen von rechts nach links.
    let mut text = Vec::with_capacity(geformt.len());
    let mut log_von_vis = Vec::with_capacity(geformt.len());
    let mut vis_von_geformt: Vec<Option<usize>> = vec![None; geformt.len()];
    for &k in &folge {
        let c = geformt[k];
        if steuerzeichen(c) { continue; }
        let c = if stufen[k].is_rtl() { spiegeln(c) } else { c };
        vis_von_geformt[k] = Some(text.len());
        text.push(c);
        log_von_vis.push(herkunft[k]);
    }
    let vis_von_log: Vec<Option<usize>> = (0..n).map(|l| vis_von_geformt[zu_geformt[l]]).collect();
    let rtl: Vec<bool> = (0..n).map(|l| stufen.get(zu_geformt[l]).map_or(false, |s| s.is_rtl())).collect();
    Zeile { text, log_von_vis, vis_von_log, rtl, basis_rtl: ganz_rtl }
}

/// Ein ganzer Text, Zeile fuer Zeile, so wie er dasteht.
pub fn anzeige(s: &str) -> String {
    if !braucht(s) { return s.to_string(); }
    let mut aus = String::with_capacity(s.len());
    for (k, zeile) in s.split('\n').enumerate() {
        if k > 0 { aus.push('\n'); }
        let z: Vec<char> = zeile.chars().collect();
        if braucht_zeichen(&z) { aus.extend(ordnen(&z, None).text); } else { aus.push_str(zeile); }
    }
    aus
}

/// Dasselbe mit fester Richtung des Absatzes.
pub fn anzeige_mit(s: &str, rtl: bool) -> String {
    let mut aus = String::with_capacity(s.len());
    for (k, zeile) in s.split('\n').enumerate() {
        if k > 0 { aus.push('\n'); }
        let z: Vec<char> = zeile.chars().collect();
        aus.extend(ordnen(&z, Some(rtl)).text);
    }
    aus
}

fn steuerzeichen(c: char) -> bool {
    matches!(c as u32, 0x200E | 0x200F | 0x202A..=0x202E | 0x2066..=0x2069 | 0x061C)
}

fn spiegeln(c: char) -> char {
    match c {
        '(' => ')', ')' => '(', '[' => ']', ']' => '[', '{' => '}', '}' => '{',
        '<' => '>', '>' => '<', '«' => '»', '»' => '«', '‹' => '›', '›' => '‹',
        _ => c,
    }
}

// --- Arabische Formen -----------------------------------------------------

/// Buchstabe -> [einzeln, am Ende, am Anfang, in der Mitte] (0 = gibt es nicht).
/// Aus den Zerlegungen der Unicode-Darstellungsformen erzeugt.
const FORMEN: &[(u32, [u32; 4])] = &[
    (0x0621, [0xFE80, 0x0000, 0x0000, 0x0000]),
    (0x0622, [0xFE81, 0xFE82, 0x0000, 0x0000]),
    (0x0623, [0xFE83, 0xFE84, 0x0000, 0x0000]),
    (0x0624, [0xFE85, 0xFE86, 0x0000, 0x0000]),
    (0x0625, [0xFE87, 0xFE88, 0x0000, 0x0000]),
    (0x0626, [0xFE89, 0xFE8A, 0xFE8B, 0xFE8C]),
    (0x0627, [0xFE8D, 0xFE8E, 0x0000, 0x0000]),
    (0x0628, [0xFE8F, 0xFE90, 0xFE91, 0xFE92]),
    (0x0629, [0xFE93, 0xFE94, 0x0000, 0x0000]),
    (0x062A, [0xFE95, 0xFE96, 0xFE97, 0xFE98]),
    (0x062B, [0xFE99, 0xFE9A, 0xFE9B, 0xFE9C]),
    (0x062C, [0xFE9D, 0xFE9E, 0xFE9F, 0xFEA0]),
    (0x062D, [0xFEA1, 0xFEA2, 0xFEA3, 0xFEA4]),
    (0x062E, [0xFEA5, 0xFEA6, 0xFEA7, 0xFEA8]),
    (0x062F, [0xFEA9, 0xFEAA, 0x0000, 0x0000]),
    (0x0630, [0xFEAB, 0xFEAC, 0x0000, 0x0000]),
    (0x0631, [0xFEAD, 0xFEAE, 0x0000, 0x0000]),
    (0x0632, [0xFEAF, 0xFEB0, 0x0000, 0x0000]),
    (0x0633, [0xFEB1, 0xFEB2, 0xFEB3, 0xFEB4]),
    (0x0634, [0xFEB5, 0xFEB6, 0xFEB7, 0xFEB8]),
    (0x0635, [0xFEB9, 0xFEBA, 0xFEBB, 0xFEBC]),
    (0x0636, [0xFEBD, 0xFEBE, 0xFEBF, 0xFEC0]),
    (0x0637, [0xFEC1, 0xFEC2, 0xFEC3, 0xFEC4]),
    (0x0638, [0xFEC5, 0xFEC6, 0xFEC7, 0xFEC8]),
    (0x0639, [0xFEC9, 0xFECA, 0xFECB, 0xFECC]),
    (0x063A, [0xFECD, 0xFECE, 0xFECF, 0xFED0]),
    (0x0641, [0xFED1, 0xFED2, 0xFED3, 0xFED4]),
    (0x0642, [0xFED5, 0xFED6, 0xFED7, 0xFED8]),
    (0x0643, [0xFED9, 0xFEDA, 0xFEDB, 0xFEDC]),
    (0x0644, [0xFEDD, 0xFEDE, 0xFEDF, 0xFEE0]),
    (0x0645, [0xFEE1, 0xFEE2, 0xFEE3, 0xFEE4]),
    (0x0646, [0xFEE5, 0xFEE6, 0xFEE7, 0xFEE8]),
    (0x0647, [0xFEE9, 0xFEEA, 0xFEEB, 0xFEEC]),
    (0x0648, [0xFEED, 0xFEEE, 0x0000, 0x0000]),
    (0x0649, [0xFEEF, 0xFEF0, 0x0000, 0x0000]),
    (0x064A, [0xFEF1, 0xFEF2, 0xFEF3, 0xFEF4]),
    (0x067E, [0xFB56, 0xFB57, 0xFB58, 0xFB59]),
    (0x0686, [0xFB7A, 0xFB7B, 0xFB7C, 0xFB7D]),
    (0x0698, [0xFB8A, 0xFB8B, 0x0000, 0x0000]),
    (0x06A9, [0xFB8E, 0xFB8F, 0xFB90, 0xFB91]),
    (0x06AF, [0xFB92, 0xFB93, 0xFB94, 0xFB95]),
    (0x06CC, [0xFBFC, 0xFBFD, 0xFBFE, 0xFBFF]),
];

/// Lam + Alef -> [einzeln, am Ende].
const LAM_ALEF: &[(u32, [u32; 2])] = &[
    (0x0622, [0xFEF5, 0xFEF6]),
    (0x0623, [0xFEF7, 0xFEF8]),
    (0x0625, [0xFEF9, 0xFEFA]),
    (0x0627, [0xFEFB, 0xFEFC]),
];

const LAM: char = '\u{0644}';

#[derive(Clone, Copy, PartialEq)]
enum Verbindung {
    Keine,      // verbindet sich mit nichts
    Rechts,     // nur mit dem Buchstaben davor (Alef, Dal, Reh, Waw ...)
    Beide,      // mit beiden Seiten
    Verbindend, // Tatweel, ZWJ: verbindet, ohne selbst Formen zu haben
    Durchlaessig, // Vokalzeichen: zaehlt nicht mit
}

fn formen_von(c: char) -> Option<[u32; 4]> {
    let c = c as u32;
    FORMEN.binary_search_by_key(&c, |e| e.0).ok().map(|i| FORMEN[i].1)
}

fn verbindung(c: char) -> Verbindung {
    let u = c as u32;
    if let Some(f) = formen_von(c) {
        return if f[2] != 0 { Verbindung::Beide } else if f[1] != 0 { Verbindung::Rechts } else { Verbindung::Keine };
    }
    match u {
        0x0640 | 0x200D => Verbindung::Verbindend,
        0x0610..=0x061A | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06DC
        | 0x06DF..=0x06E4 | 0x06E7 | 0x06E8 | 0x06EA..=0x06ED => Verbindung::Durchlaessig,
        _ => Verbindung::Keine,
    }
}

/// Kann sich ein Buchstabe mit dem FOLGENDEN verbinden?
fn nach_links(v: Verbindung) -> bool { matches!(v, Verbindung::Beide | Verbindung::Verbindend) }
/// ... mit dem VORIGEN?
fn nach_rechts(v: Verbindung) -> bool { matches!(v, Verbindung::Beide | Verbindung::Rechts | Verbindung::Verbindend) }

/// Arabische Buchstaben in ihre Formen bringen. Liefert die geformten Zeichen
/// und je geformtem Zeichen das erste logische Zeichen.
fn formen(z: &[char]) -> (Vec<char>, Vec<usize>) {
    let n = z.len();
    if !z.iter().any(|&c| matches!(c as u32, 0x0600..=0x06FF)) {
        return (z.to_vec(), (0..n).collect());
    }
    let art: Vec<Verbindung> = z.iter().map(|&c| verbindung(c)).collect();
    // Der naechste Buchstabe davor/dahinter, der kein Vokalzeichen ist.
    let davor = |i: usize| -> Option<usize> { (0..i).rev().find(|&k| art[k] != Verbindung::Durchlaessig) };
    let danach = |i: usize| -> Option<usize> { (i + 1..n).find(|&k| art[k] != Verbindung::Durchlaessig) };
    let mut aus = Vec::with_capacity(n);
    let mut herkunft = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let c = z[i];
        let v = art[i];
        let rechts = nach_rechts(v) && davor(i).map_or(false, |k| nach_links(art[k]));
        // Lam + Alef wird eine Glyphe.
        if c == LAM && i + 1 < n {
            if let Some(&(_, lig)) = LAM_ALEF.iter().find(|e| e.0 == z[i + 1] as u32) {
                aus.push(char::from_u32(lig[if rechts { 1 } else { 0 }]).unwrap_or(c));
                herkunft.push(i);
                i += 2;
                continue;
            }
        }
        let geformt = match formen_von(c) {
            Some(f) => {
                let links = nach_links(v) && danach(i).map_or(false, |k| nach_rechts(art[k]));
                let wahl = match (rechts, links) {
                    (true, true) => if f[3] != 0 { f[3] } else { f[1] },
                    (true, false) => if f[1] != 0 { f[1] } else { f[0] },
                    (false, true) => if f[2] != 0 { f[2] } else { f[0] },
                    (false, false) => f[0],
                };
                char::from_u32(wahl).unwrap_or(c)
            }
            None => c,
        };
        aus.push(geformt);
        herkunft.push(i);
        i += 1;
    }
    (aus, herkunft)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(s: &str) -> Vec<char> { s.chars().collect() }
    fn t(z: &Zeile) -> String { z.text.iter().collect() }

    #[test]
    fn ohne_rtl_bleibt_alles() {
        assert!(!braucht("Hallo Welt äöü"));
        assert_eq!(anzeige("Hallo Welt"), "Hallo Welt");
    }

    #[test]
    fn hebraeisch_steht_umgekehrt() {
        let r = ordnen(&z("שלום"), None);
        assert_eq!(t(&r), "םולש");
        assert!(r.basis_rtl);
        assert_eq!(r.log_von_vis, vec![3, 2, 1, 0]);
        assert_eq!(r.vis_von_log, vec![Some(3), Some(2), Some(1), Some(0)]);
        assert!(r.rtl.iter().all(|&b| b));
    }

    #[test]
    fn gemischt_bleiben_lateinisch_und_zahlen_stehen() {
        // Absatz links nach rechts: das hebraeische Wort dreht sich, der Rest bleibt.
        assert_eq!(anzeige("abc שלום def"), "abc םולש def");
        // Absatz rechts nach links: die Zahl bleibt eine Zahl.
        assert_eq!(anzeige("שלום 123"), "123 םולש");
    }

    #[test]
    fn klammern_spiegeln_sich() {
        assert_eq!(anzeige("שלום (א)"), "(א) םולש");
    }

    #[test]
    fn arabisch_verbindet() {
        // ب ب ب: Anfang, Mitte, Ende -- umgedreht in der Anzeige.
        let r = ordnen(&z("ببب"), None);
        assert_eq!(r.text, vec!['\u{FE90}', '\u{FE92}', '\u{FE91}']);
        // ا verbindet nur nach rechts: بابا = Anfang, Ende, Anfang, Ende.
        let r = ordnen(&z("بابا"), None);
        assert_eq!(r.text, vec!['\u{FE8E}', '\u{FE91}', '\u{FE8E}', '\u{FE91}']);
        // Einzeln steht er einzeln.
        assert_eq!(ordnen(&z("ب"), None).text, vec!['\u{FE8F}']);
    }

    #[test]
    fn lam_alef_ist_eine_glyphe() {
        let r = ordnen(&z("لا"), None);
        assert_eq!(r.text, vec!['\u{FEFB}']);
        assert_eq!(r.vis_von_log, vec![Some(0), Some(0)]);
        // Hinter einem verbindenden Buchstaben die Endform: سلام
        let r = ordnen(&z("سلام"), None);
        assert_eq!(r.text, vec!['\u{FEE1}', '\u{FEFC}', '\u{FEB3}']);
        assert_eq!(r.log_von_vis, vec![3, 1, 0]);
    }

    #[test]
    fn vokalzeichen_unterbrechen_nicht() {
        // بَب: das Fatha zwischen zwei Beh laesst sie verbunden.
        let r = ordnen(&z("ب\u{064E}ب"), None);
        assert!(r.text.contains(&'\u{FE91}') && r.text.contains(&'\u{FE90}'));
    }

    #[test]
    fn steuerzeichen_fallen_weg() {
        // LRO haelt die Reihenfolge fest, ist aber selbst nicht zu sehen.
        let r = ordnen(&z("\u{202D}בא\u{202C}"), None);
        assert_eq!(t(&r), "בא");
        assert_eq!(r.vis_von_log, vec![None, Some(0), Some(1), None]);
    }

    #[test]
    fn karten_sind_umkehrbar() {
        for s in ["abc שלום def", "سلام عليكم 42", "x (ב) y", "لا إله"] {
            let r = ordnen(&z(s), None);
            for (v, &l) in r.log_von_vis.iter().enumerate() {
                assert_eq!(r.vis_von_log[l], Some(v), "{}", s);
            }
        }
    }

    #[test]
    fn feste_richtung() {
        assert_eq!(anzeige_mit("abc", true), "abc");
        assert_eq!(basis(&z("abc שלום")), Some(false));
        assert_eq!(basis(&z("שלום abc")), Some(true));
        assert_eq!(basis(&z("abc")), None);
    }
}
