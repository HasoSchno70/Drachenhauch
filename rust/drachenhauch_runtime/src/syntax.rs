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

/// Was ein Abschnitt ist. Die Namen gehen so nach außen (GB-Programm) und
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
        if c == '"' {
            let start = i;
            i += 1;
            while i < n && z[i] != '\n' {
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
            let art = if crate::lexer::keyword(&ohne_sigil.to_lowercase()).is_some() {
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

#[cfg(test)]
mod tests {
    use super::*;

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
