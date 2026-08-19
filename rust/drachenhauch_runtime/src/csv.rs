//! CSV lesen und schreiben (WP J).
//!
//! Bis hierher war CSV in Drachenhauch Handarbeit mit `SPLIT$` -- und die geht
//! bei der ersten Zelle schief, die das Trennzeichen enthaelt:
//!
//! ```text
//! Mueller;"Berlin; Mitte";42     SPLIT$(";") liefert vier Felder statt drei
//! ```
//!
//! Umgesetzt ist RFC 4180 mit den Abweichungen, die in freier Wildbahn noetig
//! sind: frei waehlbares Trennzeichen (Excel im deutschen Gebietsschema nimmt
//! `;`), `\r\n` und `\n` gleichermassen als Zeilenende, und eine fehlende
//! Schluss-Anfuehrung wird nicht als Fehler behandelt, sondern liest bis zum
//! Ende. Kaputte Dateien sind der Normalfall; ein Abbruch mitten im Import
//! hilft niemandem.

/// Ein Feld beim SCHREIBEN in Anfuehrungszeichen setzen -- aber nur, wenn es
/// noetig ist. Unnoetige Anfuehrungszeichen sind zwar erlaubt, machen die
/// Datei aber unleserlich und den Diff groesser.
fn feld_schreiben(feld: &str, trenner: char) -> String {
    let noetig = feld.contains(trenner)
        || feld.contains('"')
        || feld.contains('\n')
        || feld.contains('\r')
        // Fuehrende/abschliessende Leerzeichen gehen sonst beim Lesen
        // verloren -- manche Programme trimmen ungefragt.
        || feld.starts_with(' ')
        || feld.ends_with(' ');
    if !noetig {
        return feld.to_string();
    }
    let mut s = String::with_capacity(feld.len() + 2);
    s.push('"');
    for c in feld.chars() {
        if c == '"' {
            s.push('"');      // RFC 4180: verdoppeln, nicht escapen
        }
        s.push(c);
    }
    s.push('"');
    s
}

/// Eine Zeile aus Feldern bauen.
pub fn zeile_schreiben(felder: &[String], trenner: char) -> String {
    felder.iter()
        .map(|f| feld_schreiben(f, trenner))
        .collect::<Vec<_>>()
        .join(&trenner.to_string())
}

/// Eine ganze Tabelle schreiben (mit `\n` als Zeilenende).
pub fn schreiben(zeilen: &[Vec<String>], trenner: char) -> String {
    let mut s = String::new();
    for z in zeilen {
        s.push_str(&zeile_schreiben(z, trenner));
        s.push('\n');
    }
    s
}

/// CSV-Text in Zeilen und Felder zerlegen.
///
/// Die Zeilen koennen unterschiedlich lang sein -- das Auffuellen auf ein
/// Rechteck passiert erst beim Bau des GB-Arrays, weil nur dort eine feste
/// Spaltenzahl noetig ist.
pub fn lesen(text: &str, trenner: char) -> Vec<Vec<String>> {
    let mut zeilen: Vec<Vec<String>> = Vec::new();
    let mut felder: Vec<String> = Vec::new();
    let mut feld = String::new();
    let mut in_anfuehrung = false;
    // Ob die aktuelle Zeile ueberhaupt etwas enthielt -- eine Datei endet fast
    // immer mit einem Zeilenumbruch, und der soll keine Leerzeile ergeben.
    let mut angefangen = false;

    let zeichen: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < zeichen.len() {
        let c = zeichen[i];
        if in_anfuehrung {
            if c == '"' {
                if zeichen.get(i + 1) == Some(&'"') {
                    feld.push('"');
                    i += 2;
                    continue;
                }
                in_anfuehrung = false;
            } else {
                feld.push(c);
            }
            i += 1;
            continue;
        }
        if c == '"' && feld.is_empty() {
            in_anfuehrung = true;
            angefangen = true;
        } else if c == trenner {
            felder.push(std::mem::take(&mut feld));
            angefangen = true;
        } else if c == '\n' || c == '\r' {
            if c == '\r' && zeichen.get(i + 1) == Some(&'\n') {
                i += 1;
            }
            if angefangen || !feld.is_empty() || !felder.is_empty() {
                felder.push(std::mem::take(&mut feld));
                zeilen.push(std::mem::take(&mut felder));
            }
            angefangen = false;
        } else {
            feld.push(c);
            angefangen = true;
        }
        i += 1;
    }
    if angefangen || !feld.is_empty() || !felder.is_empty() {
        felder.push(feld);
        zeilen.push(felder);
    }
    zeilen
}

/// Das Trennzeichen aus einem optionalen Argument holen.
///
/// Genau EIN Zeichen -- ein mehrstelliger "Trenner" ist in CSV nicht
/// vorgesehen, und ihn stillschweigend auf das erste Zeichen zu kuerzen waere
/// schlimmer als eine klare Meldung.
pub fn trenner_aus(s: Option<&str>, fn_: &str) -> Result<char, String> {
    match s {
        None => Ok(','),
        Some(t) => {
            let mut it = t.chars();
            match (it.next(), it.next()) {
                (Some(c), None) => Ok(c),
                _ => Err(format!(
                    "{}: das Trennzeichen muss genau ein Zeichen sein (bekommen: {:?})",
                    fn_, t)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trenner_im_feld_ueberlebt() {
        let z = lesen("Mueller;\"Berlin; Mitte\";42\n", ';');
        assert_eq!(z, vec![vec!["Mueller", "Berlin; Mitte", "42"]]);
    }

    #[test]
    fn verdoppelte_anfuehrung() {
        let z = lesen("\"sagt \"\"hallo\"\"\";x\n", ';');
        assert_eq!(z, vec![vec!["sagt \"hallo\"", "x"]]);
    }

    #[test]
    fn zeilenumbruch_im_feld() {
        let z = lesen("a;\"zwei\nzeilen\";c\n", ';');
        assert_eq!(z, vec![vec!["a", "zwei\nzeilen", "c"]]);
    }

    #[test]
    fn crlf_und_lf_gemischt() {
        let z = lesen("a;b\r\nc;d\ne;f", ';');
        assert_eq!(z.len(), 3);
        assert_eq!(z[2], vec!["e", "f"]);
    }

    #[test]
    fn leere_felder_bleiben_erhalten() {
        let z = lesen(";;\n", ';');
        assert_eq!(z, vec![vec!["", "", ""]]);
    }

    #[test]
    fn schlusszeilenumbruch_erzeugt_keine_leerzeile() {
        assert_eq!(lesen("a;b\n", ';').len(), 1);
        assert_eq!(lesen("a;b", ';').len(), 1);
    }

    #[test]
    fn fehlende_schlussanfuehrung_liest_bis_ende() {
        // Kaputte Dateien sind der Normalfall -- nicht abbrechen.
        let z = lesen("a;\"offen bis zum Schluss", ';');
        assert_eq!(z, vec![vec!["a", "offen bis zum Schluss"]]);
    }

    #[test]
    fn schreiben_setzt_nur_noetige_anfuehrungen() {
        let z = vec![vec!["a".into(), "b;c".into(), "d\"e".into(), " f".into()]];
        assert_eq!(schreiben(&z, ';'), "a;\"b;c\";\"d\"\"e\";\" f\"\n");
    }

    #[test]
    fn rundreise() {
        let vorher = vec![
            vec!["Name".to_string(), "Ort".to_string()],
            vec!["Mueller".to_string(), "Berlin; Mitte".to_string()],
            vec!["\"Anfuehrung\"".to_string(), "zwei\nzeilen".to_string()],
        ];
        let text = schreiben(&vorher, ';');
        assert_eq!(lesen(&text, ';'), vorher);
    }
}
