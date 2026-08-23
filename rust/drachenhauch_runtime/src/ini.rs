//! Modul `ini` -- Einstellungsdateien im INI-Format (Punkt 7 des
//! Allzweck-Audits).
//!
//! Bis hierher gab es JSON und CSV. Fuer eine Einstellungsdatei, die ein
//! Mensch mit dem Editor anfassen soll, ist beides unhandlich: JSON verzeiht
//! kein Komma zu viel, CSV hat keine benannten Felder. INI ist das Format,
//! das seit Jahrzehnten genau dafuer da ist -- und das ein ESP32-Bastler
//! ohnehin schon auf der Platine hat.
//!
//! **Ohne eigenen Handle-Typ.** Eine INI-Datei IST hier eine `MAP OF STRING`
//! mit Punkt-Schluesseln:
//!
//! ```text
//! [fenster]          ->   "fenster.breite" -> "1280"
//! breite=1280             "fenster.titel"  -> "Mein Spiel"
//! titel=Mein Spiel
//! ```
//!
//! Das spart ein Dutzend Getter und Setter: `MAPGETOR`, `MAPPUT`, `MAPKEYS`
//! und `VAL` koennen das alles schon, und wer die Sprache kennt, kennt damit
//! auch dieses Modul. Der Preis ist eine Grenze, die JSON genauso hat: ein
//! Punkt IM Schluesselnamen ist nicht adressierbar.
//!
//! Reine Zeichenketten-Arbeit, deshalb hier mit eigenen Tests; die VM reicht
//! in `builtins.rs` nur durch.

/// Eine INI-Quelle in Paare `(punkt.schluessel, wert)` zerlegen.
///
/// **Absichtlich nachsichtig.** Eine Einstellungsdatei wird von Hand
/// bearbeitet, oft von jemandem, der kein Programmierer ist. Eine Zeile, die
/// nicht passt, wird deshalb uebersprungen statt den ganzen Start abzubrechen
/// -- anders als bei JSON, wo eine kaputte Datei fast immer ein Fehler des
/// Programms ist und nicht des Menschen.
///
/// Erkannt wird:
///   * `[abschnitt]` -- alles danach traegt `abschnitt.` als Vorsatz
///   * `name = wert` (Leerraum aussen faellt weg, im Wert erhalten)
///   * `;` und `#` als Kommentar am ZEILENANFANG
///   * Schluessel vor dem ersten Abschnitt: ohne Vorsatz
///   * `"wert"` in Anfuehrungszeichen -- sie fallen weg, der Leerraum darin
///     bleibt (der einzige Weg, fuehrende Leerzeichen zu behalten)
pub fn lesen(text: &str) -> Vec<(String, String)> {
    let mut raus: Vec<(String, String)> = Vec::new();
    let mut abschnitt = String::new();
    for zeile in text.lines() {
        let z = zeile.trim();
        if z.is_empty() || z.starts_with(';') || z.starts_with('#') { continue; }
        if z.starts_with('[') {
            if let Some(ende) = z.find(']') {
                abschnitt = z[1..ende].trim().to_string();
            }
            // Eine `[`-Zeile ohne `]` ist kaputt -- der bisherige Abschnitt
            // gilt weiter. Alles andere waere Raten.
            continue;
        }
        let Some((k, v)) = z.split_once('=') else { continue };
        let name = k.trim();
        if name.is_empty() { continue; }
        let wert = entkleiden(v.trim());
        let schluessel = if abschnitt.is_empty() { name.to_string() }
                         else { format!("{}.{}", abschnitt, name) };
        raus.push((schluessel, wert));
    }
    raus
}

/// Umschliessende Anfuehrungszeichen entfernen -- aber nur, wenn beide da
/// sind. Ein einzelnes ist wahrscheinlich Teil des Werts.
fn entkleiden(s: &str) -> String {
    let b: Vec<char> = s.chars().collect();
    if b.len() >= 2 && ((b[0] == '"' && b[b.len() - 1] == '"')
                        || (b[0] == '\'' && b[b.len() - 1] == '\'')) {
        return b[1..b.len() - 1].iter().collect();
    }
    s.to_string()
}

/// Paare zurueck in INI-Text.
///
/// Die Abschnitte kommen in der Reihenfolge ihres ersten Auftretens, die
/// Schluessel in ihrer eigenen -- eine gespeicherte und wieder geschriebene
/// Datei sieht damit aus wie vorher, statt in jedem Durchlauf neu gemischt
/// zu werden.
///
/// **Kommentare gehen verloren.** Wer eine Datei mit Erklaerungen darin
/// einliest und zurueckschreibt, verliert sie; das steht in der Doku. Sie zu
/// erhalten hiesse, die urspruengliche Datei mitzufuehren -- und dann waere
/// es kein `MAP` mehr, sondern doch wieder ein Handle.
pub fn schreiben(paare: &[(String, String)]) -> String {
    // Nach Abschnitt buendeln, Reihenfolge des ersten Auftretens behalten.
    let mut ordnung: Vec<String> = Vec::new();
    let mut gebuendelt: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (k, v) in paare {
        let (abschnitt, name) = match k.split_once('.') {
            Some((a, n)) => (a.to_string(), n.to_string()),
            None => (String::new(), k.clone()),
        };
        match ordnung.iter().position(|a| *a == abschnitt) {
            Some(i) => gebuendelt[i].1.push((name, v.clone())),
            None => {
                ordnung.push(abschnitt.clone());
                gebuendelt.push((abschnitt, vec![(name, v.clone())]));
            }
        }
    }
    let mut raus = String::new();
    for (abschnitt, eintraege) in &gebuendelt {
        if !abschnitt.is_empty() {
            if !raus.is_empty() { raus.push('\n'); }
            raus.push_str(&format!("[{}]\n", abschnitt));
        }
        for (name, wert) in eintraege {
            // In Anfuehrungszeichen, wenn der Wert sonst beim Lesen anders
            // herauskaeme: Leerraum am Rand ginge verloren, ein `;` oder `#`
            // am Anfang machte die Zeile zu einem Kommentar.
            let braucht = wert != wert.trim()
                || wert.starts_with(';') || wert.starts_with('#')
                || wert.contains('\n');
            if braucht {
                raus.push_str(&format!("{}=\"{}\"\n", name, wert.replace('\n', " ")));
            } else {
                raus.push_str(&format!("{}={}\n", name, wert));
            }
        }
    }
    raus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paare(t: &str) -> Vec<(String, String)> { lesen(t) }

    #[test]
    fn abschnitte_werden_zu_punkt_schluesseln() {
        let p = paare("[fenster]\nbreite=1280\nhoehe = 720\n");
        assert_eq!(p, vec![("fenster.breite".into(), "1280".into()),
                           ("fenster.hoehe".into(), "720".into())]);
    }

    #[test]
    fn vor_dem_ersten_abschnitt_ohne_vorsatz() {
        let p = paare("name=Anna\n[a]\nx=1\n");
        assert_eq!(p[0], ("name".to_string(), "Anna".to_string()));
        assert_eq!(p[1], ("a.x".to_string(), "1".to_string()));
    }

    #[test]
    fn kommentare_und_leerzeilen_fallen_weg() {
        let p = paare("; ein Kommentar\n\n# noch einer\n[a]\nx=1\n");
        assert_eq!(p, vec![("a.x".into(), "1".into())]);
    }

    #[test]
    fn ein_semikolon_im_wert_bleibt_stehen() {
        // Nur am ZEILENANFANG ist es ein Kommentar -- sonst waere ein Pfad
        // wie `C:\a;C:\b` nicht speicherbar.
        let p = paare("pfad=C:/a;C:/b\n");
        assert_eq!(p[0].1, "C:/a;C:/b");
    }

    #[test]
    fn gleichheitszeichen_im_wert() {
        let p = paare("formel=a=b+c\n");
        assert_eq!(p[0].1, "a=b+c");
    }

    #[test]
    fn anfuehrungszeichen_erhalten_leerraum() {
        let p = paare("t=\"  mit Rand  \"\n");
        assert_eq!(p[0].1, "  mit Rand  ");
        // Ein einzelnes gehoert zum Wert.
        assert_eq!(paare("t=\"halb\n")[0].1, "\"halb");
    }

    #[test]
    fn kaputte_zeilen_werden_uebersprungen() {
        // Eine Einstellungsdatei bearbeitet ein Mensch -- eine unklare Zeile
        // darf nicht den ganzen Start verhindern.
        let p = paare("[a]\nkaputt ohne Gleichheitszeichen\nx=1\n=ohne Namen\n");
        assert_eq!(p, vec![("a.x".into(), "1".into())]);
    }

    #[test]
    fn hin_und_zurueck() {
        let quelle = "[fenster]\nbreite=1280\ntitel=Mein Spiel\n\n[ton]\nlaut=0.8\n";
        let p = lesen(quelle);
        let neu = schreiben(&p);
        assert_eq!(lesen(&neu), p);
        assert!(neu.contains("[fenster]"));
        assert!(neu.contains("[ton]"));
    }

    #[test]
    fn heikle_werte_kommen_in_anfuehrungszeichen() {
        let p = vec![("a.rand".to_string(), "  x  ".to_string()),
                     ("a.raute".to_string(), "#eins".to_string())];
        let t = schreiben(&p);
        assert!(t.contains("rand=\"  x  \""), "{}", t);
        assert!(t.contains("raute=\"#eins\""), "{}", t);
        assert_eq!(lesen(&t), p);
    }

    #[test]
    fn schluessel_ohne_abschnitt_stehen_oben() {
        let t = schreiben(&[("global".to_string(), "1".to_string()),
                            ("a.x".to_string(), "2".to_string())]);
        assert!(t.starts_with("global=1\n"), "{}", t);
    }
}
