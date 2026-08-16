//! Modul `zeit` -- mit Datum und Uhrzeit rechnen.
//!
//! Bis hierher gab es nur `DATE$()`/`TIME$()` (Text) und `MILLIS()`. Damit
//! liess sich anzeigen und vergleichen, aber nicht rechnen: "Anstoss minus
//! 15 Minuten", "noch 2:15 h bis zum Anpfiff", "welcher Tag ist in 3 Tagen"
//! musste jedes Programm selbst aus Zeichenketten schneiden.
//!
//! **Der Zeitwert** dieses Moduls ist eine ganze Zahl: Sekunden seit dem
//! 1.1.1970, gerechnet in ORTSZEIT. Damit passen `ZEIT_JETZT()`,
//! `ZEIT_PARSE("2026-08-28 20:30:00")` und die Ausgabe von `DATE$`/`TIME$`
//! zusammen, ohne dass eine Zeitzonen-Datenbank noetig waere -- eine
//! Anwendung, die Anstosszeiten als Ortszeit speichert (der Normalfall),
//! rechnet damit richtig.
//!
//! **Grenze, bewusst so:** ueber eine Zeitumstellung hinweg kann eine
//! Differenz um eine Stunde danebenliegen. Das ist der Preis dafuer, keine
//! Zeitzonen-Datenbank mitzuschleppen; fuer Tippschluss, Countdown und
//! "welcher Spieltag ist heute" spielt es keine Rolle.
//!
//! Reine Mathematik ohne Zustand -- deshalb hier als freie Funktionen mit
//! eigenen Tests; die VM reicht in `try_zeit` (vm.rs) nur durch.

const SEK_PRO_TAG: i64 = 86_400;

/// Tage seit 1970-01-01 aus einem Kalenderdatum (Howard Hinnants
/// `days_from_civil`) -- gilt fuer den gesamten proleptisch-gregorianischen
/// Kalender, also auch vor 1970 (negative Werte).
pub fn tage_aus_datum(jahr: i64, monat: i64, tag: i64) -> i64 {
    let y = if monat <= 2 { jahr - 1 } else { jahr };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if monat > 2 { monat - 3 } else { monat + 9 };
    let doy = (153 * mp + 2) / 5 + tag - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Umkehrung: Kalenderdatum aus Tagen seit 1970-01-01.
pub fn datum_aus_tagen(tage: i64) -> (i64, i64, i64) {
    let z = tage + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let tag = doy - (153 * mp + 2) / 5 + 1;
    let monat = if mp < 10 { mp + 3 } else { mp - 9 };
    (if monat <= 2 { y + 1 } else { y }, monat, tag)
}

/// Zeitwert aus einzelnen Feldern.
pub fn aus_teilen(jahr: i64, monat: i64, tag: i64, stunde: i64, minute: i64, sekunde: i64) -> i64 {
    tage_aus_datum(jahr, monat, tag) * SEK_PRO_TAG + stunde * 3600 + minute * 60 + sekunde
}

/// Zeitwert in seine Felder zerlegen: (Jahr, Monat, Tag, Stunde, Minute, Sekunde).
pub fn in_teile(t: i64) -> (i64, i64, i64, i64, i64, i64) {
    let tage = t.div_euclid(SEK_PRO_TAG);
    let rest = t.rem_euclid(SEK_PRO_TAG);
    let (j, mo, d) = datum_aus_tagen(tage);
    (j, mo, d, rest / 3600, (rest % 3600) / 60, rest % 60)
}

/// Wochentag: 1 = Montag ... 7 = Sonntag (ISO).
///
/// Der 1.1.1970 war ein Donnerstag -- daher der Versatz.
pub fn wochentag(t: i64) -> i64 {
    (t.div_euclid(SEK_PRO_TAG) + 3).rem_euclid(7) + 1
}

fn ist_schaltjahr(jahr: i64) -> bool {
    (jahr % 4 == 0 && jahr % 100 != 0) || jahr % 400 == 0
}

fn tage_im_monat(jahr: i64, monat: i64) -> i64 {
    match monat {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if ist_schaltjahr(jahr) { 29 } else { 28 },
        _ => 0,
    }
}

/// Liest einen Zeitpunkt aus Text.
///
/// Angenommen werden die Schreibweisen, die in Datenbanken und Schnittstellen
/// tatsaechlich vorkommen:
///   `2026-08-28 20:30:00`, `2026-08-28T20:30:00` (ISO mit T),
///   `2026-08-28 20:30` (ohne Sekunden), `2026-08-28` (nur Datum -> 00:00:00).
/// Eine angehaengte Zeitzone (`Z`, `+02:00`) wird abgeschnitten, nicht
/// verrechnet -- dieses Modul rechnet in Ortszeit.
///
/// `None` heisst: nicht lesbar. Der Aufrufer entscheidet, ob das ein Fehler
/// ist; `ZEIT_PARSE` macht daraus eine Meldung, `ZEIT_LESBAR` ein FALSE.
pub fn parse(text: &str) -> Option<i64> {
    let roh = text.trim();
    if roh.is_empty() { return None; }

    // Zeitzonen-Anhang abschneiden (nach der Uhrzeit, nicht im Datum!)
    let ohne_zone = match roh.rfind(['+', 'Z', 'z']) {
        Some(i) if i > 10 => &roh[..i],
        _ => roh,
    };
    let ohne_zone = ohne_zone.trim();

    let (datum_teil, zeit_teil) = match ohne_zone.find(['T', 't', ' ']) {
        Some(i) => (&ohne_zone[..i], ohne_zone[i + 1..].trim()),
        None => (ohne_zone, ""),
    };

    let d: Vec<&str> = datum_teil.split('-').collect();
    if d.len() != 3 { return None; }
    let jahr: i64 = d[0].parse().ok()?;
    let monat: i64 = d[1].parse().ok()?;
    let tag: i64 = d[2].parse().ok()?;

    if !(1..=12).contains(&monat) { return None; }
    if tag < 1 || tag > tage_im_monat(jahr, monat) { return None; }

    let (mut std, mut min, mut sek) = (0i64, 0i64, 0i64);
    if !zeit_teil.is_empty() {
        let z: Vec<&str> = zeit_teil.split(':').collect();
        if z.len() < 2 || z.len() > 3 { return None; }
        std = z[0].parse().ok()?;
        min = z[1].parse().ok()?;
        if z.len() == 3 {
            // Bruchteile von Sekunden wegwerfen ("20:30:00.123")
            let s = z[2].split('.').next().unwrap_or("");
            sek = s.parse().ok()?;
        }
    }
    if !(0..=23).contains(&std) || !(0..=59).contains(&min) || !(0..=60).contains(&sek) {
        return None;
    }

    Some(aus_teilen(jahr, monat, tag, std, min, sek))
}

/// Formatiert einen Zeitpunkt.
///
/// Platzhalter: `JJJJ` Jahr, `MM` Monat, `TT` Tag, `hh` Stunde, `mm` Minute,
/// `ss` Sekunde, `WT` Wochentag kurz (Mo..So), `WTAG` ausgeschrieben.
/// Leeres Muster = `JJJJ-MM-TT hh:mm:ss`.
pub fn format(t: i64, muster: &str) -> String {
    let m = if muster.trim().is_empty() { "JJJJ-MM-TT hh:mm:ss" } else { muster };
    let (j, mo, d, h, mi, s) = in_teile(t);
    let wt = wochentag(t);

    const KURZ: [&str; 7] = ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];
    const LANG: [&str; 7] = ["Montag", "Dienstag", "Mittwoch", "Donnerstag",
                             "Freitag", "Samstag", "Sonntag"];

    // Laengere Platzhalter zuerst, sonst frisst "MM" das "M" von "WTAG" nicht,
    // aber "WT" wuerde vor "WTAG" greifen und "AG" stehen lassen.
    let mut aus = String::with_capacity(m.len() + 8);
    let zeichen: Vec<char> = m.chars().collect();
    let mut i = 0;
    while i < zeichen.len() {
        let rest: String = zeichen[i..].iter().collect();
        if rest.starts_with("JJJJ") { aus.push_str(&format!("{:04}", j)); i += 4; }
        else if rest.starts_with("WTAG") { aus.push_str(LANG[(wt - 1) as usize]); i += 4; }
        else if rest.starts_with("WT") { aus.push_str(KURZ[(wt - 1) as usize]); i += 2; }
        else if rest.starts_with("MM") { aus.push_str(&format!("{:02}", mo)); i += 2; }
        else if rest.starts_with("TT") { aus.push_str(&format!("{:02}", d)); i += 2; }
        else if rest.starts_with("hh") { aus.push_str(&format!("{:02}", h)); i += 2; }
        else if rest.starts_with("mm") { aus.push_str(&format!("{:02}", mi)); i += 2; }
        else if rest.starts_with("ss") { aus.push_str(&format!("{:02}", s)); i += 2; }
        else { aus.push(zeichen[i]); i += 1; }
    }
    aus
}

/// Eine Dauer in Sekunden als lesbarer Text: "45 s", "12 min", "2:15 h",
/// "3 Tage". Negative Dauern bekommen ein "vor " vorangestellt.
pub fn dauer(sekunden: i64) -> String {
    let vergangen = sekunden < 0;
    let s = sekunden.abs();

    let text = if s < 60 {
        format!("{} s", s)
    } else if s < 3600 {
        format!("{} min", s / 60)
    } else if s < SEK_PRO_TAG {
        format!("{}:{:02} h", s / 3600, (s % 3600) / 60)
    } else {
        let tage = s / SEK_PRO_TAG;
        let stunden = (s % SEK_PRO_TAG) / 3600;
        if stunden == 0 {
            format!("{} Tag{}", tage, if tage == 1 { "" } else { "e" })
        } else {
            format!("{} Tag{} {} h", tage, if tage == 1 { "" } else { "e" }, stunden)
        }
    };

    if vergangen { format!("vor {}", text) } else { text }
}

/// Einzelnes Feld eines Zeitpunkts. `None` = unbekannter Name.
pub fn teil(t: i64, was: &str) -> Option<i64> {
    let (j, mo, d, h, mi, s) = in_teile(t);
    match was.to_lowercase().as_str() {
        "jahr" => Some(j),
        "monat" => Some(mo),
        "tag" => Some(d),
        "stunde" => Some(h),
        "minute" => Some(mi),
        "sekunde" => Some(s),
        "wochentag" => Some(wochentag(t)),
        _ => None,
    }
}

/// Die Namen, die `teil` versteht -- fuer die Fehlermeldung.
pub const TEILE: &str = "jahr, monat, tag, stunde, minute, sekunde, wochentag";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datum_hin_und_zurueck() {
        for (j, m, t) in [(1970, 1, 1), (2000, 2, 29), (2026, 8, 28), (1969, 12, 31)] {
            let tage = tage_aus_datum(j, m, t);
            assert_eq!(datum_aus_tagen(tage), (j, m, t), "{}-{}-{}", j, m, t);
        }
    }

    #[test]
    fn epoche_beginnt_bei_null() {
        assert_eq!(aus_teilen(1970, 1, 1, 0, 0, 0), 0);
    }

    #[test]
    fn parse_versteht_die_ueblichen_schreibweisen() {
        let soll = aus_teilen(2026, 8, 28, 20, 30, 0);
        for text in ["2026-08-28 20:30:00", "2026-08-28T20:30:00",
                     "2026-08-28 20:30", "2026-08-28T20:30:00.500",
                     "2026-08-28T20:30:00Z", "2026-08-28T20:30:00+02:00",
                     "  2026-08-28 20:30:00  "] {
            assert_eq!(parse(text), Some(soll), "{}", text);
        }
    }

    #[test]
    fn parse_nur_datum_ist_mitternacht() {
        assert_eq!(parse("2026-08-28"), Some(aus_teilen(2026, 8, 28, 0, 0, 0)));
    }

    #[test]
    fn parse_lehnt_unsinn_ab() {
        for text in ["", "morgen", "2026-13-01", "2026-02-30", "2026-08-28 25:00",
                     "28.08.2026", "2026-08", "2026-08-28 20:61"] {
            assert_eq!(parse(text), None, "{}", text);
        }
    }

    #[test]
    fn schaltjahr_wird_beachtet() {
        assert!(parse("2024-02-29 12:00:00").is_some());
        assert_eq!(parse("2025-02-29 12:00:00"), None);
    }

    #[test]
    fn format_ohne_muster() {
        let t = aus_teilen(2026, 8, 28, 20, 30, 5);
        assert_eq!(format(t, ""), "2026-08-28 20:30:05");
    }

    #[test]
    fn format_mit_wochentag() {
        // 28.08.2026 ist ein Freitag
        let t = aus_teilen(2026, 8, 28, 20, 30, 0);
        assert_eq!(format(t, "WT TT.MM.JJJJ hh:mm"), "Fr 28.08.2026 20:30");
        assert_eq!(format(t, "WTAG"), "Freitag");
    }

    #[test]
    fn format_und_parse_sind_umkehrbar() {
        let t = aus_teilen(2026, 12, 31, 23, 59, 59);
        assert_eq!(parse(&format(t, "")), Some(t));
    }

    #[test]
    fn wochentage_stimmen() {
        assert_eq!(wochentag(aus_teilen(1970, 1, 1, 0, 0, 0)), 4);   // Donnerstag
        assert_eq!(wochentag(aus_teilen(2026, 8, 16, 0, 0, 0)), 7);  // Sonntag
        assert_eq!(wochentag(aus_teilen(2026, 8, 17, 0, 0, 0)), 1);  // Montag
    }

    #[test]
    fn dauer_liest_sich() {
        assert_eq!(dauer(45), "45 s");
        assert_eq!(dauer(12 * 60), "12 min");
        assert_eq!(dauer(2 * 3600 + 15 * 60), "2:15 h");
        assert_eq!(dauer(3 * 86_400), "3 Tage");
        assert_eq!(dauer(86_400), "1 Tag");
        assert_eq!(dauer(86_400 + 5 * 3600), "1 Tag 5 h");
        assert_eq!(dauer(-90), "vor 1 min");
    }

    #[test]
    fn teile_lesen() {
        let t = aus_teilen(2026, 8, 28, 20, 30, 5);
        assert_eq!(teil(t, "jahr"), Some(2026));
        assert_eq!(teil(t, "monat"), Some(8));
        assert_eq!(teil(t, "STUNDE"), Some(20));
        assert_eq!(teil(t, "quartal"), None);
    }

    #[test]
    fn negative_zeiten_vor_1970() {
        let t = aus_teilen(1969, 7, 20, 20, 17, 0);
        assert!(t < 0);
        assert_eq!(in_teile(t), (1969, 7, 20, 20, 17, 0));
    }
}
