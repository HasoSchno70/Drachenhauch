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

/// Unix-Sekunden (UTC) -> Zeitwert in Ortszeit; `versatz` ist der Abstand
/// der Ortszeit zu UTC fuer einen UTC-Zeitpunkt (samt Sommerzeit).
pub fn aus_unix_mit(unix: i64, versatz: impl Fn(i64) -> i64) -> i64 {
    unix + versatz(unix)
}

/// Zeitwert in Ortszeit -> Unix-Sekunden. Der Versatz haengt am UTC-Zeitpunkt,
/// den man erst sucht: einmal mit dem Versatz der Ortszeit geschaetzt, dann
/// mit dem Versatz an der Schaetzung. In der Stunde, die es bei der Umstellung
/// auf Sommerzeit nicht gibt, oder der doppelten im Herbst ist das Ergebnis
/// eine der beiden moeglichen Antworten.
pub fn nach_unix_mit(t: i64, versatz: impl Fn(i64) -> i64) -> i64 {
    let schaetzung = t - versatz(t);
    t - versatz(schaetzung)
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
/// Angenommen werden die Schreibweisen, die in Datenbanken, Schnittstellen
/// und Feeds tatsaechlich vorkommen:
///   `2026-08-28 20:30:00`, `2026-08-28T20:30:00` (ISO mit T),
///   `2026-08-28 20:30` (ohne Sekunden), `2026-08-28` (nur Datum -> 00:00:00),
///   `Sat, 28 Aug 2026 20:30:00 +0200` (RFC 822 -- RSS, HTTP-Kopf `Date`, E-Mail).
///
/// **Mit Zeitzone** (`Z`, `+02:00`, `-0500`, `GMT`, `CEST` ...) ist der Text ein
/// fester Zeitpunkt: er wird auf UTC zurueckgerechnet und in Ortszeit an
/// DIESEM Tag umgesetzt (`builtins::lokaler_versatz`). Bis 2026.27 wurde der
/// Anhang abgeschnitten -- `14:30Z` galt als 14:30 Ortszeit, und Feeds aus
/// verschiedenen Zonen sortierten um Stunden falsch. **Ohne Zeitzone** gilt
/// der Text als Ortszeit, wie immer.
///
/// `None` heisst: nicht lesbar. Der Aufrufer entscheidet, ob das ein Fehler
/// ist; `ZEIT_PARSE` macht daraus eine Meldung, `ZEIT_LESBAR` ein FALSE.
pub fn parse(text: &str) -> Option<i64> {
    parse_mit(text, crate::builtins::lokaler_versatz)
}

/// `parse` mit einer eigenen Ortszeit: `lokal(utc)` = Versatz der Ortszeit in
/// Sekunden fuer einen UTC-Zeitpunkt. Fuer die Tests, die sonst vom Rechner
/// abhingen.
pub fn parse_mit(text: &str, lokal: impl Fn(i64) -> i64) -> Option<i64> {
    let (t, versatz) = parse_teile(text)?;
    Some(match versatz {
        None => t,
        Some(v) => { let utc = t - v; utc + lokal(utc) }
    })
}

/// Der Zeitpunkt, wie er dasteht, und der Versatz zu UTC in Sekunden, falls
/// einer angegeben ist.
pub fn parse_teile(text: &str) -> Option<(i64, Option<i64>)> {
    let roh = text.trim();
    if roh.is_empty() { return None; }
    let b = roh.as_bytes();
    if b.len() >= 5 && b[..4].iter().all(|c| c.is_ascii_digit()) && b[4] == b'-' {
        iso_lesen(roh)
    } else {
        rfc822_lesen(roh)
    }
}

fn iso_lesen(roh: &str) -> Option<(i64, Option<i64>)> {
    let (datum_teil, rest) = match roh.find(['T', 't', ' ']) {
        Some(i) => (&roh[..i], roh[i + 1..].trim()),
        None => (roh, ""),
    };
    let d: Vec<&str> = datum_teil.split('-').collect();
    if d.len() != 3 { return None; }
    let jahr: i64 = d[0].parse().ok()?;
    let monat: i64 = d[1].parse().ok()?;
    let tag: i64 = d[2].parse().ok()?;
    if !(1..=12).contains(&monat) { return None; }
    if tag < 1 || tag > tage_im_monat(jahr, monat) { return None; }
    if rest.is_empty() { return Some((aus_teilen(jahr, monat, tag, 0, 0, 0), None)); }

    // Uhrzeit und Zone: "20:30:00Z", "20:30:00+02:00", "20:30:00-0500",
    // "20:30:00 UTC"
    let (uhr, zone) = if let Some(i) = rest.find(char::is_whitespace) {
        (&rest[..i], Some(rest[i..].trim()))
    } else if rest.ends_with(['Z', 'z']) {
        (&rest[..rest.len() - 1], Some("Z"))
    } else if let Some(i) = rest.rfind(['+', '-']) {
        (&rest[..i], Some(&rest[i..]))
    } else {
        (rest, None)
    };
    let (std, min, sek) = uhr_lesen(uhr)?;
    let versatz = match zone { Some(z) => Some(zone_wert(z)?), None => None };
    Some((aus_teilen(jahr, monat, tag, std, min, sek), versatz))
}

/// RFC 822/1123: `[Wochentag,] TT Mon JJJJ hh:mm[:ss] [Zone]`.
fn rfc822_lesen(roh: &str) -> Option<(i64, Option<i64>)> {
    let mut t: Vec<&str> = roh.split([' ', '\t', ',']).filter(|w| !w.is_empty()).collect();
    if t.first().is_some_and(|w| w.chars().all(|c| c.is_ascii_alphabetic())) {
        t.remove(0);    // Wochentag -- wird nicht nachgeprueft
    }
    if t.len() < 4 || t.len() > 5 { return None; }
    let tag: i64 = t[0].parse().ok()?;
    const MONATE: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun",
                                "jul", "aug", "sep", "oct", "nov", "dec"];
    let m = t[1].to_ascii_lowercase();
    if m.len() < 3 { return None; }
    let monat = MONATE.iter().position(|x| m.starts_with(x))? as i64 + 1;
    let mut jahr: i64 = t[2].parse().ok()?;
    if t[2].len() == 2 { jahr += if jahr < 50 { 2000 } else { 1900 }; }
    if tag < 1 || tag > tage_im_monat(jahr, monat) { return None; }
    let (std, min, sek) = uhr_lesen(t[3])?;
    let versatz = match t.get(4) { Some(z) => Some(zone_wert(z)?), None => None };
    Some((aus_teilen(jahr, monat, tag, std, min, sek), versatz))
}

fn uhr_lesen(uhr: &str) -> Option<(i64, i64, i64)> {
    let z: Vec<&str> = uhr.split(':').collect();
    if z.len() < 2 || z.len() > 3 { return None; }
    let std: i64 = z[0].parse().ok()?;
    let min: i64 = z[1].parse().ok()?;
    let mut sek = 0;
    if z.len() == 3 {
        // Bruchteile von Sekunden wegwerfen ("20:30:00.123")
        sek = z[2].split('.').next().unwrap_or("").parse().ok()?;
    }
    if !(0..=23).contains(&std) || !(0..=59).contains(&min) || !(0..=60).contains(&sek) {
        return None;
    }
    Some((std, min, sek))
}

/// Versatz zu UTC in Sekunden. Ein unbekannter Name ist NICHT lesbar statt
/// still UTC -- ein falscher Versatz fiele erst an einer falschen Sortierung auf.
fn zone_wert(z: &str) -> Option<i64> {
    let stunden = |h: i64| Some(h * 3600);
    match z.to_ascii_uppercase().as_str() {
        "Z" | "UT" | "UTC" | "GMT" => return Some(0),
        "EST" => return stunden(-5), "EDT" => return stunden(-4),
        "CST" => return stunden(-6), "CDT" => return stunden(-5),
        "MST" => return stunden(-7), "MDT" => return stunden(-6),
        "PST" => return stunden(-8), "PDT" => return stunden(-7),
        "CET" | "MEZ" => return stunden(1), "CEST" | "MESZ" => return stunden(2),
        _ => {}
    }
    let (vz, ziffern) = match z.as_bytes().first()? {
        b'+' => (1, &z[1..]),
        b'-' => (-1, &z[1..]),
        _ => return None,
    };
    let ziffern: String = ziffern.chars().filter(|c| *c != ':').collect();
    if !ziffern.chars().all(|c| c.is_ascii_digit()) { return None; }
    let (h, m): (i64, i64) = match ziffern.len() {
        2 => (ziffern.parse().ok()?, 0),
        4 => (ziffern[..2].parse().ok()?, ziffern[2..].parse().ok()?),
        _ => return None,
    };
    if h > 14 || m > 59 { return None; }
    Some(vz * (h * 3600 + m * 60))
}

/// Formatiert einen Zeitpunkt.
///
/// Platzhalter: `JJJJ` Jahr, `MM` Monat, `TT` Tag, `hh` Stunde, `mm` Minute,
/// `ss` Sekunde, `WT` Wochentag kurz (Mo..So), `WTAG` ausgeschrieben.
/// Leeres Muster = `JJJJ-MM-TT hh:mm:ss`.
pub fn format(t: i64, muster: &str) -> String {
    format_pruefen(t, muster).unwrap_or_else(|e| e)
}

/// Wie `format`, aber ein unbekannter `%`-Platzhalter ist ein Fehler.
///
/// Neben den deutschen Platzhaltern versteht es die Schreibweise von C,
/// Python und PHP (`%Y-%m-%d %H:%M:%S`). Bis 2026.30 ging sie STILL
/// woertlich durch -- 59580 Zeitstempel "%Y-%m-%d %H:%M:%S" in einer Datei,
/// ohne ein Wort (Messdaten-Runde, 2026-10-13).
pub fn format_pruefen(t: i64, muster: &str) -> Result<String, String> {
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
        if zeichen[i] == '%' && i + 1 < zeichen.len() {
            let c = zeichen[i + 1];
            match c {
                'Y' => aus.push_str(&format!("{:04}", j)),
                'y' => aus.push_str(&format!("{:02}", j.rem_euclid(100))),
                'm' => aus.push_str(&format!("{:02}", mo)),
                'd' => aus.push_str(&format!("{:02}", d)),
                'H' => aus.push_str(&format!("{:02}", h)),
                'M' => aus.push_str(&format!("{:02}", mi)),
                'S' => aus.push_str(&format!("{:02}", s)),
                'a' => aus.push_str(KURZ[(wt - 1) as usize]),
                'A' => aus.push_str(LANG[(wt - 1) as usize]),
                '%' => aus.push('%'),
                _ => return Err(format!(
                    "ZEIT_FORMAT$: '%{}' kennt das Muster nicht -- %Y %y %m %d %H %M %S %a %A %% oder JJJJ MM TT hh mm ss WT WTAG", c)),
            }
            i += 2;
            continue;
        }
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
    Ok(aus)
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
    fn format_versteht_auch_prozent_muster() {
        let t = parse("2026-08-28 20:30:05").unwrap();
        assert_eq!(format(t, "%Y-%m-%d %H:%M:%S"), "2026-08-28 20:30:05");
        assert_eq!(format(t, "%a %d.%m.%y"), "Fr 28.08.26");
        assert_eq!(format(t, "%A, 100%%"), "Freitag, 100%");
        assert_eq!(format(t, "TT.MM.JJJJ"), "28.08.2026");
        // ein Prozentzeichen ohne Buchstaben dahinter bleibt stehen
        assert_eq!(format(t, "MM %"), "08 %");
        assert!(format_pruefen(t, "%Q").unwrap_err().contains("'%Q'"));
    }

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
                     "  2026-08-28 20:30:00  "] {
            assert_eq!(parse(text), Some(soll), "{}", text);
        }
    }

    #[test]
    fn zeitzonen_werden_in_ortszeit_umgerechnet() {
        // Ortszeit fest auf UTC+2, damit der Test nicht am Rechner haengt
        let mesz = |_| 7200;
        let soll = aus_teilen(2026, 8, 28, 22, 30, 0);
        for text in ["2026-08-28T20:30:00Z", "2026-08-28T22:30:00+02:00",
                     "2026-08-28T15:30:00-05:00", "2026-08-28T16:30:00-0400",
                     "2026-08-28 20:30:00 UTC", "2026-08-29T02:00:00+05:30",
                     "Fri, 28 Aug 2026 20:30:00 GMT", "28 Aug 2026 22:30 +0200",
                     "Fri, 28 Aug 2026 16:30:00 EDT", "Fri, 28 Aug 26 22:30:00 CEST"] {
            assert_eq!(parse_mit(text, mesz), Some(soll), "{}", text);
        }
        // ohne Zone bleibt es, wie es dasteht
        assert_eq!(parse_mit("2026-08-28T22:30:00", mesz), Some(soll));
        assert_eq!(parse_mit("Fri, 28 Aug 2026 22:30", mesz), Some(soll));
        // die Ortszeit wird fuer den UTC-Zeitpunkt gefragt, nicht fuer den Text
        let gefragt = std::cell::Cell::new(0);
        parse_mit("2026-01-15T12:00:00Z", |utc| { gefragt.set(utc); 3600 });
        assert_eq!(gefragt.get(), aus_teilen(2026, 1, 15, 12, 0, 0));
    }

    #[test]
    fn rfc822_lehnt_unsinn_ab() {
        for text in ["Fri, 28 Aug 2026 20:30:00 XYZ", "31 Feb 2026 10:00",
                     "28 Foo 2026 10:00", "28 Aug 2026", "Fri, 28 Aug 2026 25:00",
                     "2026-08-28T20:30:00+99:00", "2026-08-28T20:30:00+2"] {
            assert_eq!(parse_teile(text), None, "{}", text);
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

    #[test]
    fn unix_hin_und_zurueck_ueber_die_sommerzeit() {
        // Mitteleuropa 2026: Sommerzeit ab 29.3. 01:00 UTC, bis 25.10. 01:00 UTC
        let anfang = aus_teilen(2026, 3, 29, 1, 0, 0);
        let ende = aus_teilen(2026, 10, 25, 1, 0, 0);
        let versatz = |u: i64| if u >= anfang && u < ende { 7200 } else { 3600 };
        for (j, mo, d, h) in [(2026, 1, 15, 12), (2026, 7, 1, 0), (2026, 3, 29, 0), (2026, 3, 29, 3), (2026, 10, 25, 3), (2026, 12, 31, 23)] {
            let u = aus_teilen(j, mo, d, h, 30, 0);
            let t = aus_unix_mit(u, versatz);
            assert_eq!(nach_unix_mit(t, versatz), u, "{}-{}-{} {}h", j, mo, d, h);
        }
        // Sommer: 12:00 UTC ist 14:00 Ortszeit
        let u = aus_teilen(2026, 7, 1, 12, 0, 0);
        assert_eq!(in_teile(aus_unix_mit(u, versatz)), (2026, 7, 1, 14, 0, 0));
    }
}
