//! Einfärbung von Drachenhauch-Quelltext (`SYNTAX_SPANS`).
//!
//! **Ein Hervorheber ist kein Lexer.** Der Lexer in `lexer.rs` wirft
//! Kommentare weg, expandiert f-Strings zu ganzen Token-Folgen und bricht bei
//! einem Fehler ab -- alles drei ist für das Einfärben genau falsch. Ein
//! Editor bekommt halb getippten Text zu sehen (`"abc` ohne schließendes
//! Anführungszeichen, `IF x THE`), und er muss ihn trotzdem darstellen.
//!
//! Darum ein eigener, absichtlich naiver Abtaster, der NIE fehlschlägt. Die
//! Wortliste teilt er sich mit dem Lexer (`lexer::keyword`) -- so kann ein
//! neues Schlüsselwort nicht an einer Stelle bekannt sein und an der anderen
//! nicht.
//!
//! Ausgabe sind Abschnitte `(start, länge, art)` in ZEICHEN (nicht Bytes),
//! damit sie zu den Zeichen-Positionen des `gui`-Textfelds passen.

/// Was ein Abschnitt ist. Die Namen gehen so nach außen (DH-Programm) und
/// werden dort auf Farben abgebildet -- absichtlich, denn welche Farbe ein
/// Kommentar hat, ist eine Frage des Themas und nicht der Sprache.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Art {
    Kommentar,
    Text,
    Zahl,
    Schluessel,
    Name,
    Operator,
}

impl Art {
    pub fn name(self) -> &'static str {
        match self {
            Art::Kommentar => "kommentar",
            Art::Text => "text",
            Art::Zahl => "zahl",
            Art::Schluessel => "schluessel",
            Art::Name => "name",
            Art::Operator => "operator",
        }
    }
}

fn ist_name_start(c: char) -> bool { c.is_alphabetic() || c == '_' }
fn ist_name_teil(c: char) -> bool { c.is_alphanumeric() || c == '_' }

/// Das Wort ab `i` (Leerraum davor uebersprungen, nicht ueber das
/// Zeilenende), klein -- leer, wenn dort kein Name steht.
fn wort_ab(z: &[char], mut i: usize) -> (String, usize) {
    while i < z.len() && (z[i] == ' ' || z[i] == '\t') { i += 1; }
    let a = i;
    while i < z.len() && ist_name_teil(z[i]) { i += 1; }
    (z[a..i].iter().collect::<String>().to_lowercase(), a)
}

/// Das Wort vor `ende` in derselben Zeile, klein; leer am Anweisungsanfang
/// (Zeilenanfang oder `:`).
fn wort_vor(z: &[char], ende: usize) -> String {
    let mut i = ende;
    while i > 0 && (z[i - 1] == ' ' || z[i - 1] == '\t') { i -= 1; }
    let e = i;
    while i > 0 && ist_name_teil(z[i - 1]) { i -= 1; }
    z[i..e].iter().collect::<String>().to_lowercase()
}

/// Steht vor `start` nur Leerraum bis zum Zeilenanfang oder einem `:`?
fn anweisungsanfang(z: &[char], start: usize) -> bool {
    let mut i = start;
    while i > 0 && (z[i - 1] == ' ' || z[i - 1] == '\t') { i -= 1; }
    i == 0 || z[i - 1] == '\n' || z[i - 1] == ':'
}

/// Woerter, die nur an ihrer Stelle Schluesselwoerter sind (der Parser
/// liest sie als Namen mit Vorausschau, damit `DIM do` oder `DIM abstract`
/// in altem Code gueltig bleibt). Der Hervorheber faerbt sie darum auch nur
/// dort -- vorher standen `ABSTRACT FUNCTION`, `DO ... LOOP`, `FOR EACH` und
/// `EXIT FOR` halb weiss im Editor.
fn kontext_schluessel(z: &[char], start: usize, ende: usize, w: &str) -> bool {
    let (nach, nach_ab) = wort_ab(z, ende);
    let davor = wort_vor(z, start);
    let am_anfang = anweisungsanfang(z, start);
    // Nach dem Wort: Zeilenende, Kommentar oder `:` -- `DO` / `LOOP` allein.
    let rest_leer = {
        let mut i = ende;
        while i < z.len() && (z[i] == ' ' || z[i] == '\t') { i += 1; }
        i >= z.len() || matches!(z[i], '\n' | '\'' | ':' | '\r')
    };
    let name_folgt = !nach.is_empty() && nach_ab < z.len() && ist_name_start(z[nach_ab]);
    match w {
        "abstract" | "declare" => matches!(nach.as_str(), "sub" | "function"),
        "each" => davor == "for" && name_folgt,
        "exit" => am_anfang && matches!(nach.as_str(), "for" | "do" | "while" | "repeat" | "sub" | "function"),
        "do" => am_anfang && (rest_leer || matches!(nach.as_str(), "while" | "until")),
        "loop" => am_anfang && (rest_leer || matches!(nach.as_str(), "while" | "until")),
        "swap" | "let" | "call" => am_anfang && name_folgt,
        "byval" => matches!(davor.as_str(), "" | "sub" | "function") && name_folgt
                   && start > 0 && z[..start].iter().rev().find(|c| **c != ' ').map_or(false, |c| *c == '(' || *c == ','),
        "get" | "set" => davor == "property",
        "lib" | "alias" => {
            // nur in einer DECLARE-Zeile
            let mut a = start;
            while a > 0 && z[a - 1] != '\n' { a -= 1; }
            wort_ab(z, a).0 == "declare"
        }
        _ => false,
    }
}

/// Quelltext in Abschnitte zerlegen. Leerraum bekommt keinen Abschnitt --
/// was nicht genannt ist, zeichnet das Textfeld in seiner Grundfarbe.
pub fn spans(src: &str) -> Vec<(usize, usize, Art)> {
    let z: Vec<char> = src.chars().collect();
    let n = z.len();
    let mut out: Vec<(usize, usize, Art)> = Vec::new();
    let mut i = 0usize;
    while i < n {
        let c = z[i];

        if c == '\n' { i += 1; continue; }
        if c.is_whitespace() { i += 1; continue; }

        // Kommentar bis Zeilenende: `'` oder das Wort REM.
        if c == '\'' {
            let start = i;
            while i < n && z[i] != '\n' { i += 1; }
            out.push((start, i - start, Art::Kommentar));
            continue;
        }

        // Zeichenkette. Zwei Anfuehrungszeichen hintereinander sind EIN
        // Zeichen im Text und beenden sie NICHT. Fehlt das schliessende,
        // endet sie am Zeilenende -- beim Tippen ist das der Normalfall,
        // und den Rest der Datei rot zu faerben waere unbrauchbar.
        // `!"..."` traegt Escape-Folgen: dort beendet ein `\"` die Kette nicht.
        if c == '"' || (c == '!' && i + 1 < n && z[i + 1] == '"') {
            let start = i;
            let escapes = c == '!';
            i += if escapes { 2 } else { 1 };
            while i < n && z[i] != '\n' {
                if escapes && z[i] == '\\' && i + 1 < n && z[i + 1] != '\n' { i += 2; continue; }
                if z[i] == '"' {
                    if i + 1 < n && z[i + 1] == '"' { i += 2; continue; }
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.push((start, i - start, Art::Text));
            continue;
        }

        // Zahl -- auch `&H1F` / `&B1010`, die Drachenhauch-Schreibweise.
        if c.is_ascii_digit() || (c == '&' && i + 1 < n && matches!(z[i + 1], 'h' | 'H' | 'b' | 'B')) {
            let start = i;
            if c == '&' {
                i += 2;
                while i < n && (z[i].is_ascii_hexdigit() || z[i] == '_') { i += 1; }
            } else {
                while i < n && (z[i].is_ascii_digit() || z[i] == '_') { i += 1; }
                // Genau EIN Punkt, und nur wenn eine Ziffer folgt: `a.b` ist
                // ein Zugriff, kein Komma, und `1..5` waere ein Bereich.
                if i < n && z[i] == '.' && i + 1 < n && z[i + 1].is_ascii_digit() {
                    i += 1;
                    while i < n && z[i].is_ascii_digit() { i += 1; }
                }
                // Exponent.
                if i < n && matches!(z[i], 'e' | 'E') {
                    let mut j = i + 1;
                    if j < n && matches!(z[j], '+' | '-') { j += 1; }
                    if j < n && z[j].is_ascii_digit() {
                        i = j;
                        while i < n && z[i].is_ascii_digit() { i += 1; }
                    }
                }
            }
            out.push((start, i - start, Art::Zahl));
            continue;
        }

        // Name oder Schluesselwort.
        if ist_name_start(c) {
            let start = i;
            while i < n && ist_name_teil(z[i]) { i += 1; }
            // `name$` / `zahl%` -- das Typkennzeichen gehoert zum Namen.
            if i < n && matches!(z[i], '$' | '%' | '#' | '!') { i += 1; }
            let wort: String = z[start..i].iter().collect();
            let ohne_sigil = wort.trim_end_matches(['$', '%', '#', '!']);
            // REM leitet einen Kommentar ein, ist also kein gewoehnliches Wort.
            if ohne_sigil.eq_ignore_ascii_case("rem") {
                while i < n && z[i] != '\n' { i += 1; }
                out.push((start, i - start, Art::Kommentar));
                continue;
            }
            let klein = ohne_sigil.to_lowercase();
            let art = if crate::lexer::keyword(&klein).is_some()
                || (wort.len() == ohne_sigil.len() && kontext_schluessel(&z, start, i, &klein)) {
                Art::Schluessel
            } else {
                Art::Name
            };
            out.push((start, i - start, art));
            continue;
        }

        // Alles Uebrige ist Zeichensetzung/Rechenzeichen. Mehrstellige wie
        // `<=` oder `+=` bleiben zusammen -- getrennt saehen sie im Editor
        // aus wie zwei Zeichen mit einer unsichtbaren Fuge.
        let start = i;
        i += 1;
        if i < n {
            let paar: String = z[start..=i].iter().collect();
            if matches!(paar.as_str(), "<=" | ">=" | "<>" | "+=" | "-=" | "*=" | "/=" | "..") {
                i += 1;
            }
        }
        out.push((start, i - start, Art::Operator));
    }
    out
}

/// Wie oft jeder Name im Quelltext vorkommt (CODE_NAMES): klein
/// geschrieben, samt Typkennzeichen (`name$`), ohne Kommentare und
/// Zeichenketten -- und ohne die Stelle, an der eine SUB/FUNCTION ihn
/// DEFINIERT (der Name direkt hinter dem Schluesselwort). Was uebrig bleibt,
/// sind die Benutzungen: Aufrufe, FUNCREFs, Methoden hinter einem Punkt.
/// Namen in f-Strings zaehlen nicht (der Hervorheber sieht dort Text).
pub fn namen_zaehlen(src: &str) -> Vec<(String, i64)> {
    let z: Vec<char> = src.chars().collect();
    let mut zahl: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    // Hinter SUB/FUNCTION in DERSELBEN Zeile steht die Definition; hinter
    // `END SUB` / `EXIT SUB` (und einem Zeilenende) nicht.
    let mut nach_def: Option<usize> = None;
    let mut vorher_end = false;
    for (s, l, art) in spans(src) {
        let wort: String = z[s..s + l].iter().collect::<String>().to_lowercase();
        match art {
            Art::Schluessel => {
                nach_def = if (wort == "sub" || wort == "function") && !vorher_end { Some(s + l) } else { None };
                // Hinter DECLARE steht eine Definition (aus einer Bibliothek),
                // hinter END/EXIT keine.
                vorher_end = wort == "end" || wort == "exit";
            }
            Art::Name => {
                let def = nach_def.map_or(false, |e| !z[e..s].contains(&'\n'));
                if !def { *zahl.entry(wort).or_insert(0) += 1; }
                nach_def = None;
                vorher_end = false;
            }
            Art::Kommentar => {}
            _ => { nach_def = None; vorher_end = false; }
        }
    }
    let mut v: Vec<(String, i64)> = zahl.into_iter().collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namen_zaehlen_ohne_definition_kommentar_text() {
        let q = "SUB malen()\n    PRINT(1)\nEND SUB\nmalen() ' malen()\nPRINT(\"malen\")\nf = Malen\nx = held.malen()\nFUNCTION wert$()\nEND FUNCTION\nPRINT(wert$())";
        let v = namen_zaehlen(q);
        let hole = |n: &str| v.iter().find(|(k, _)| k == n).map(|p| p.1).unwrap_or(0);
        assert_eq!(hole("malen"), 3);
        assert_eq!(hole("wert$"), 1);
        assert_eq!(hole("held"), 1);
        // EXIT SUB / END SUB davor: der Name dahinter ist ein Aufruf
        let v2 = namen_zaehlen("SUB a()\n    EXIT SUB\nEND SUB : a()\nSUB b()\nEND SUB\nb()");
        let hole2 = |n: &str| v2.iter().find(|(k, _)| k == n).map(|p| p.1).unwrap_or(0);
        assert_eq!(hole2("a"), 1);
        assert_eq!(hole2("b"), 1);
    }

    fn arten(src: &str) -> Vec<(&'static str, String)> {
        let z: Vec<char> = src.chars().collect();
        spans(src).into_iter()
            .map(|(s, l, a)| (a.name(), z[s..s + l].iter().collect::<String>()))
            .collect()
    }

    #[test]
    fn schluesselwort_gegen_name() {
        assert_eq!(arten("DIM x AS INTEGER"), vec![
            ("schluessel", "DIM".into()),
            ("name", "x".into()),
            ("schluessel", "AS".into()),
            ("schluessel", "INTEGER".into()),
        ]);
    }

    #[test]
    fn woerter_mit_kontext() {
        let schl = |src: &str, w: &str| arten(src).iter().any(|(a, t)| *a == "schluessel" && t.eq_ignore_ascii_case(w));
        assert!(schl("    ABSTRACT FUNCTION laut() AS STRING", "abstract"));
        assert!(schl("FOR EACH e IN liste", "each"));
        assert!(schl("DO WHILE x < 3", "do"));
        assert!(schl("DO", "do"));
        assert!(schl("LOOP UNTIL fertig", "loop"));
        assert!(schl("    EXIT FOR", "exit"));
        assert!(schl("DECLARE FUNCTION f LIB \"c\" ALIAS \"g\" () AS LONG", "lib"));
        assert!(schl("DECLARE FUNCTION f LIB \"c\" ALIAS \"g\" () AS LONG", "alias"));
        assert!(schl("PROPERTY GET hp() AS INTEGER", "get"));
        assert!(schl("SWAP a, b", "swap"));
        assert!(schl("SUB s(BYVAL p AS Punkt)", "byval"));
        // Als Name bleiben sie Namen.
        assert!(!schl("DIM abstract AS INTEGER", "abstract"));
        assert!(!schl("DIM do AS INTEGER", "do"));
        assert!(!schl("do = 3", "do"));
        assert!(!schl("PRINT each", "each"));
        assert!(!schl("x = loop + 1", "loop"));
        assert!(!schl("exit = 2", "exit"));
        assert!(!schl("lib = 1", "lib"));
        // DECLARE FUNCTION f: f ist eine Definition, keine Benutzung.
        let v = namen_zaehlen("DECLARE FUNCTION f LIB \"c\" () AS LONG\nPRINT(f())");
        assert_eq!(v.iter().find(|(k, _)| k == "f").map(|p| p.1), Some(1));
    }

    #[test]
    fn kommentar_bis_zeilenende() {
        let a = arten("x = 1 ' und der Rest\ny = 2");
        assert!(a.contains(&("kommentar", "' und der Rest".into())));
        // Die naechste Zeile faengt wieder normal an.
        assert!(a.contains(&("name", "y".into())));
    }

    #[test]
    fn rem_ist_auch_ein_kommentar() {
        let a = arten("REM alles hier\nDIM x");
        assert_eq!(a[0], ("kommentar", "REM alles hier".into()));
        assert_eq!(a[1], ("schluessel", "DIM".into()));
    }

    #[test]
    fn rem_nur_als_ganzes_wort() {
        // `remote` faengt mit REM an, ist aber ein Name.
        let a = arten("remote = 1");
        assert_eq!(a[0], ("name", "remote".into()));
    }

    #[test]
    fn doppeltes_anfuehrungszeichen_beendet_nicht() {
        let a = arten(r#"s = "a""b" + x"#);
        assert!(a.contains(&("text", r#""a""b""#.into())), "{:?}", a);
        assert!(a.contains(&("name", "x".into())));
    }

    #[test]
    fn offene_zeichenkette_endet_an_der_zeile() {
        // Der Normalfall beim Tippen. Faerbte sie den Rest der Datei ein,
        // waere der Editor unbrauchbar.
        let a = arten("s = \"abc\nDIM y");
        assert_eq!(a[2], ("text", "\"abc".into()));
        assert_eq!(a[3], ("schluessel", "DIM".into()));
    }

    #[test]
    fn zahlen_in_allen_schreibweisen() {
        let a = arten("1 2.5 &H1F 1e3");
        assert_eq!(a.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
                   vec!["zahl", "zahl", "zahl", "zahl"]);
        assert_eq!(a[2].1, "&H1F");
        assert_eq!(a[3].1, "1e3");
    }

    #[test]
    fn punkt_nach_zahl_ist_kein_komma() {
        // `1..5` waere ein Bereich, kein Kommawert; und `a.b` ein Zugriff.
        let a = arten("a.b");
        assert_eq!(a[0], ("name", "a".into()));
        assert_eq!(a[1], ("operator", ".".into()));
        assert_eq!(a[2], ("name", "b".into()));
    }

    #[test]
    fn typkennzeichen_gehoert_zum_namen() {
        let a = arten("name$ = LEFT$(s, 2)");
        assert_eq!(a[0], ("name", "name$".into()));
    }

    #[test]
    fn mehrstellige_zeichen_bleiben_zusammen() {
        let a = arten("a <= b <> c += 1");
        let ops: Vec<String> = a.iter().filter(|(k, _)| *k == "operator")
            .map(|(_, t)| t.clone()).collect();
        assert_eq!(ops, vec!["<=", "<>", "+="]);
    }

    #[test]
    fn haelt_halb_getippten_text_aus() {
        // Kein Absturz, kein Verschlucken -- der Editor sieht so etwas
        // bei JEDEM Tastendruck.
        for src in ["IF x THE", "\"", "&H", "1.", "s$ = \"a", "'"] {
            let _ = spans(src);
        }
    }

    #[test]
    fn abschnitte_liegen_lueckenlos_und_der_reihe_nach() {
        let src = "FOR i = 0 TO 9 : PRINT i ' zaehlen\nNEXT";
        let mut ende = 0;
        for (s, l, _) in spans(src) {
            assert!(s >= ende, "Abschnitte ueberlappen bei {}", s);
            ende = s + l;
        }
        assert!(ende <= src.chars().count());
    }
}
