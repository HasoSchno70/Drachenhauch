//! Kalender-Rechnung fuer den Datums-Waehler (`GUI_DATEPICKER`).
//!
//! Absichtlich **freie Funktionen ohne Widget und ohne Fenster**: nur so
//! laesst sich der heikle Teil -- Schaltjahre, Monatslaengen, der Wochentag
//! des Ersten -- ohne Bildschirm pruefen. Haengte die Rechnung am Widget,
//! waere sie nur mit laufendem Fenster testbar, und genau dort sieht man
//! einen Fehler am schlechtesten: ein um einen Tag verschobener Monat faellt
//! beim Hinsehen kaum auf.
//!
//! Das Textformat ist `YYYY-MM-DD` -- dasselbe, das `DATE$()` liefert. Zwei
//! Datumsformate im selben System waeren eine Stolperfalle.

/// Schaltjahr nach dem gregorianischen Kalender.
pub fn schaltjahr(jahr: i32) -> bool {
    (jahr % 4 == 0 && jahr % 100 != 0) || jahr % 400 == 0
}

/// Tage im Monat (`monat` 1..12). Ein Monat ausserhalb liefert 0 -- der
/// Aufrufer klemmt vorher, hier wird nicht stillschweigend geraten.
pub fn tage_im_monat(jahr: i32, monat: i32) -> i32 {
    match monat {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if schaltjahr(jahr) { 29 } else { 28 },
        _ => 0,
    }
}

/// Wochentag als 0 = Montag .. 6 = Sonntag.
///
/// Zellers Kongruenz. Sie zaehlt Januar und Februar zum VORJAHR (als Monat 13
/// und 14) -- ohne diese Verschiebung stimmt die Schaltjahr-Rechnung im
/// Februar nicht, und der ganze Monat rutscht um einen Tag.
///
/// Montag als 0, weil der Waehler die Woche so anzeigt (in Deutschland
/// beginnt sie am Montag); Zeller selbst zaehlt ab Samstag.
pub fn wochentag(jahr: i32, monat: i32, tag: i32) -> i32 {
    let (m, j) = if monat < 3 { (monat + 12, jahr - 1) } else { (monat, jahr) };
    let k = j % 100;
    let s = j / 100;
    let h = (tag + (13 * (m + 1)) / 5 + k + k / 4 + s / 4 + 5 * s) % 7;
    // Zeller: 0 = Samstag, 1 = Sonntag, 2 = Montag ... -> Montag auf 0 drehen.
    (h + 5) % 7
}

/// `YYYY-MM-DD` einlesen. `None` bei allem, was nicht genau so aussieht --
/// ein halb erkanntes Datum waere schlimmer als eine klare Absage.
pub fn parse(s: &str) -> Option<(i32, i32, i32)> {
    let t: Vec<&str> = s.trim().split('-').collect();
    if t.len() != 3 { return None; }
    if t[0].len() != 4 || t[1].len() != 2 || t[2].len() != 2 { return None; }
    let j: i32 = t[0].parse().ok()?;
    let m: i32 = t[1].parse().ok()?;
    let d: i32 = t[2].parse().ok()?;
    if !(1..=9999).contains(&j) || !(1..=12).contains(&m) { return None; }
    if d < 1 || d > tage_im_monat(j, m) { return None; }
    Some((j, m, d))
}

pub fn format(jahr: i32, monat: i32, tag: i32) -> String {
    std::format!("{:04}-{:02}-{:02}", jahr, monat, tag)
}

/// Ein Datum um `tage` Tage verschieben (auch negativ).
///
/// Schrittweise ueber die Monatsgrenzen statt ueber eine Tageszahl seit einem
/// Stichtag: fuer die paar Tage, die ein Pfeiltastendruck bewegt, ist das
/// genauso schnell und deutlich leichter nachzulesen.
pub fn plus_tage(jahr: i32, monat: i32, tag: i32, tage: i32) -> (i32, i32, i32) {
    let (mut j, mut m, mut d) = (jahr, monat, tag);
    let mut rest = tage;
    while rest > 0 {
        let im_monat = tage_im_monat(j, m);
        if d < im_monat { d += 1; } else {
            d = 1;
            m += 1;
            if m > 12 { m = 1; j += 1; }
        }
        rest -= 1;
    }
    while rest < 0 {
        if d > 1 { d -= 1; } else {
            m -= 1;
            if m < 1 { m = 12; j -= 1; }
            d = tage_im_monat(j, m);
        }
        rest += 1;
    }
    (j, m, d)
}

/// Einen Monat weiter oder zurueck, mit geklemmtem Tag.
///
/// Der 31. Maerz minus einen Monat ist der 28. (oder 29.) Februar, nicht der
/// 3. Maerz. Ohne das Klemmen laeuft der Waehler beim Blaettern durch die
/// Monate langsam nach vorne davon.
pub fn plus_monate(jahr: i32, monat: i32, tag: i32, monate: i32) -> (i32, i32, i32) {
    let gesamt = (jahr * 12 + (monat - 1)) + monate;
    let j = gesamt.div_euclid(12);
    let m = gesamt.rem_euclid(12) + 1;
    let d = tag.min(tage_im_monat(j, m));
    (j, m, d)
}

pub const MONATSNAMEN: [&str; 12] = [
    "Januar", "Februar", "Maerz", "April", "Mai", "Juni",
    "Juli", "August", "September", "Oktober", "November", "Dezember",
];

pub const WOCHENTAGE: [&str; 7] = ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schaltjahre_auch_die_jahrhundertregel() {
        assert!(schaltjahr(2024));
        assert!(!schaltjahr(2023));
        // Die Regel, die man gern vergisst: 1900 war KEIN Schaltjahr,
        // 2000 schon.
        assert!(!schaltjahr(1900));
        assert!(schaltjahr(2000));
        assert!(!schaltjahr(2100));
    }

    #[test]
    fn monatslaengen() {
        assert_eq!(tage_im_monat(2023, 2), 28);
        assert_eq!(tage_im_monat(2024, 2), 29);
        assert_eq!(tage_im_monat(2024, 4), 30);
        assert_eq!(tage_im_monat(2024, 12), 31);
        assert_eq!(tage_im_monat(2024, 13), 0, "Monat 13 darf nicht geraten werden");
    }

    #[test]
    fn wochentage_gegen_bekannte_daten() {
        // 2026-08-30 ist ein Sonntag.
        assert_eq!(wochentag(2026, 8, 30), 6);
        // 2000-01-01 war ein Samstag -- und der Januar ist der Fall, in dem
        // Zellers Monatsverschiebung greift.
        assert_eq!(wochentag(2000, 1, 1), 5);
        // 2024-02-29 (Schalttag) war ein Donnerstag.
        assert_eq!(wochentag(2024, 2, 29), 3);
        // 1970-01-01 war ein Donnerstag.
        assert_eq!(wochentag(1970, 1, 1), 3);
    }

    #[test]
    fn jeder_wochentag_kommt_in_einer_woche_genau_einmal_vor() {
        let mut gesehen = [false; 7];
        for t in 24..=30 { gesehen[wochentag(2026, 8, t) as usize] = true; }
        assert!(gesehen.iter().all(|&b| b));
    }

    #[test]
    fn einlesen_und_ausgeben_sind_umkehrbar() {
        for s in ["2026-08-30", "2000-01-01", "2024-02-29", "1999-12-31"] {
            let (j, m, d) = parse(s).expect(s);
            assert_eq!(format(j, m, d), s);
        }
    }

    #[test]
    fn unsinn_wird_abgelehnt() {
        for s in ["", "2026-8-30", "2026/08/30", "26-08-30", "2026-13-01",
                  "2026-02-30", "2023-02-29", "abc", "2026-08-30x"] {
            assert!(parse(s).is_none(), "'{}' haette abgelehnt werden muessen", s);
        }
        // Der Schalttag existiert nur im Schaltjahr.
        assert!(parse("2024-02-29").is_some());
    }

    #[test]
    fn tage_ueber_monats_und_jahresgrenzen() {
        assert_eq!(plus_tage(2026, 1, 31, 1), (2026, 2, 1));
        assert_eq!(plus_tage(2026, 12, 31, 1), (2027, 1, 1));
        assert_eq!(plus_tage(2027, 1, 1, -1), (2026, 12, 31));
        assert_eq!(plus_tage(2024, 2, 28, 1), (2024, 2, 29), "Schalttag");
        assert_eq!(plus_tage(2023, 2, 28, 1), (2023, 3, 1), "kein Schaltjahr");
        // Eine Woche vor und zurueck landet wieder am Ausgangspunkt.
        assert_eq!(plus_tage(2026, 3, 1, 7), (2026, 3, 8));
        assert_eq!(plus_tage(2026, 3, 8, -7), (2026, 3, 1));
    }

    #[test]
    fn monate_klemmen_den_tag() {
        // Der 31. minus ein Monat ist das Monatsende, nicht der 3. des
        // Folgemonats.
        assert_eq!(plus_monate(2026, 3, 31, -1), (2026, 2, 28));
        assert_eq!(plus_monate(2024, 3, 31, -1), (2024, 2, 29));
        assert_eq!(plus_monate(2026, 1, 31, 1), (2026, 2, 28));
        assert_eq!(plus_monate(2026, 12, 15, 1), (2027, 1, 15));
        assert_eq!(plus_monate(2026, 1, 15, -1), (2025, 12, 15));
    }

    #[test]
    fn zwoelf_monate_sind_ein_jahr() {
        assert_eq!(plus_monate(2026, 5, 10, 12), (2027, 5, 10));
        assert_eq!(plus_monate(2026, 5, 10, -12), (2025, 5, 10));
    }
}
