//! Modul `geld` -- ein Betrag als eigener Wert.
//!
//! Weg C aus `docs/entwurf-geldtyp.md`. Weg A (die Befehle `CENT`, `EURO$`,
//! `ROUND_HALF_UP`) ist eine Rechenweise: man rechnet in `INTEGER`-Cent und
//! muss daran denken. Hier ist es ein **Typ** -- und der denkt mit:
//!
//! ```text
//! DIM p AS GELD
//! p = GELD_NEU("19,99")
//! PRINT p * 3          ' 59,97 €
//! PRINT p + 1.0        ' Fehler: GELD und Kommazahl mischen sich nicht
//! ```
//!
//! **Innen: ein `i64` in Hundertstel-Cent**, also vier Nachkommastellen.
//! Vier, nicht zwei, weil Steuersätze und Rabatte Zwischenergebnisse mit
//! mehr Stellen erzeugen (19 % von 0,29 € sind 0,0551 €); gerundet wird erst
//! beim Ausweisen. Wertebereich damit rund ±922 Billionen Euro, und ein
//! Überlauf ist ein Fehler -- eine stumm umlaufende Summe wäre das
//! Schlimmste, was Geld passieren kann.
//!
//! **Gerechnet wird ganzzahlig, auch mit einem Faktor.** `betrag * 0.19`
//! geht nicht über `f64`, sondern zerlegt die 0.19 in ihre Dezimalziffern
//! und rechnet `wert * 19 / 100` in `i128`. Der Umweg über Fliesskomma wäre
//! genau die Ungenauigkeit, gegen die dieser Typ antritt.
//!
//! Die Zerlegung von Zahlen und Texten in Ziffern teilt sich das Modul mit
//! `CENT`/`ROUND_HALF_UP` (`builtins.rs`, Abschnitt „Geld") -- es gibt nur
//! **eine** Rundungsregel im ganzen Haus.

use crate::builtins::{dez_runden, geld_text_teile, skaliert_aus_teilen, tausender_punkte};

/// Ein GELD-Wert zählt Hundertstel-Cent: 1 € = 10 000.
pub const SKALA: i64 = 10_000;
/// So viele Nachkommastellen hat die Skala.
pub const STELLEN: u32 = 4;

fn zehn_hoch(n: u32) -> i64 {
    (0..n).fold(1i64, |a, _| a * 10)
}

/// Kaufmännisch auf ein Vielfaches von `m` runden -- rein ganzzahlig, also
/// ohne jede Fliesskomma-Beteiligung.
fn auf_vielfaches(w: i64, m: i64) -> i64 {
    if m <= 1 { return w; }
    let r = w % m;                       // Vorzeichen folgt w
    let basis = w - r;
    if r.abs() * 2 >= m {
        if w < 0 { basis - m } else { basis + m }
    } else {
        basis
    }
}

/// Auf `stellen` Nachkommastellen runden (0..=4).
pub fn runden(w: i64, stellen: u32) -> i64 {
    if stellen >= STELLEN { return w; }
    auf_vielfaches(w, zehn_hoch(STELLEN - stellen))
}

/// Aus einer geschriebenen Zahl: "19,99", "1.234,56", "-0,0551".
pub fn aus_text(s: &str) -> Result<i64, String> {
    let (neg, ganz, bruch) = geld_text_teile(s)?;
    skaliert_aus_teilen(neg, &ganz, &bruch, STELLEN as usize, "GELD_NEU")
}

/// Aus einer Zahl. Gerundet wird die Zahl, die DASTEHT (kürzeste
/// Dezimaldarstellung), nicht ihre Binärentwicklung -- Begründung in
/// `builtins.rs`.
pub fn aus_zahl(x: f64) -> Result<i64, String> {
    let (neg, ganz, bruch) = dez_runden(x, STELLEN as usize)?;
    skaliert_aus_teilen(neg, &ganz, &bruch, STELLEN as usize, "GELD_NEU")
}

/// Ein Faktor als Bruch aus ganzen Zahlen: 0.19 -> (19, 100).
///
/// Neun Nachkommastellen sind mehr, als jemand für einen Steuersatz oder
/// einen Rabatt hinschreibt, und `10^9` passt bequem in `i64`.
fn faktor_bruch(f: f64) -> Result<(i128, i128), String> {
    const G: u32 = 9;
    let (neg, ganz, bruch) = dez_runden(f, G as usize)?;
    let ziffern = format!("{}{}", ganz, bruch);
    let zaehler: i128 = ziffern.parse()
        .map_err(|_| format!("Faktor {} ist zu gross", f))?;
    Ok((if neg { -zaehler } else { zaehler }, zehn_hoch(G) as i128))
}

fn nach_i64(w: i128, was: &str) -> Result<i64, String> {
    if w > i64::MAX as i128 || w < i64::MIN as i128 {
        return Err(format!("{}: der Betrag ist zu gross (GELD reicht bis rund \
                            922 Billionen Euro)", was));
    }
    Ok(w as i64)
}

/// Kaufmännische Division zweier ganzer Zahlen (Rest >= halber Teiler
/// rundet von der Null weg).
fn teile_gerundet(zaehler: i128, nenner: i128) -> i128 {
    if nenner == 0 { return 0; }
    let (z, n) = if nenner < 0 { (-zaehler, -nenner) } else { (zaehler, nenner) };
    let q = z / n;
    let r = z % n;
    if r.abs() * 2 >= n {
        if z < 0 { q - 1 } else { q + 1 }
    } else {
        q
    }
}

pub fn mal(w: i64, f: f64) -> Result<i64, String> {
    let (zaehler, nenner) = faktor_bruch(f).map_err(|e| format!("GELD * Zahl: {}", e))?;
    nach_i64(teile_gerundet(w as i128 * zaehler, nenner), "GELD * Zahl")
}

pub fn durch(w: i64, f: f64) -> Result<i64, String> {
    let (zaehler, nenner) = faktor_bruch(f).map_err(|e| format!("GELD / Zahl: {}", e))?;
    if zaehler == 0 { return Err("GELD / Zahl: Division durch 0".to_string()); }
    nach_i64(teile_gerundet(w as i128 * nenner, zaehler), "GELD / Zahl")
}

/// Einen Betrag auf `n` Teile aufteilen, **ohne einen Cent zu verlieren**.
///
/// 10,00 € durch 3 sind nicht dreimal 3,33 € -- ein Cent bliebe liegen. Die
/// ersten Teile bekommen ihn: 3,34 / 3,33 / 3,33. Genau dafür lohnt sich ein
/// eigener Typ; von Hand vergisst man diesen Rest fast immer.
pub fn teilen(w: i64, n: i64) -> Result<Vec<i64>, String> {
    if n <= 0 {
        return Err("GELD_TEILEN: die Anzahl muss groesser als 0 sein".to_string());
    }
    // Aufgeteilt wird in ganzen Cent -- Bruchteile eines Cents kann niemand
    // ueberweisen.
    let cent = runden(w, 2) / 100;
    let grund = cent / n;
    let rest = cent % n;               // Vorzeichen folgt dem Betrag
    let schritt = if cent < 0 { -1 } else { 1 };
    Ok((0..n).map(|i| {
        let extra = if (i as i64) < rest.abs() { schritt } else { 0 };
        (grund + extra) * 100
    }).collect())
}

/// Als Text in deutscher Schreibweise, auf `stellen` Nachkommastellen.
pub fn text(w: i64, symbol: &str, stellen: u32) -> String {
    let w = runden(w, stellen.min(STELLEN));
    let neg = w < 0;
    let a = (w as i128).abs();
    let ganz = tausender_punkte(&(a / SKALA as i128).to_string());
    let bruch4 = format!("{:04}", a % SKALA as i128);
    let mut out = String::new();
    if neg && (a != 0) { out.push('-'); }
    out.push_str(&ganz);
    if stellen > 0 {
        out.push(',');
        out.push_str(&bruch4[..stellen.min(STELLEN) as usize]);
    }
    if !symbol.is_empty() {
        out.push(' ');
        out.push_str(symbol);
    }
    out
}

/// Die Form, in der `PRINT` einen Betrag zeigt: mindestens zwei
/// Nachkommastellen, mehr nur wenn wirklich welche da sind.
///
/// Ein Zwischenergebnis wie 0,0551 € als „0,06 €" anzuzeigen wäre bequem und
/// irreführend -- man sähe der Zahl nicht mehr an, dass sie noch nicht
/// gerundet ist.
pub fn anzeige(w: i64) -> String {
    let stellen = if w % 100 == 0 { 2 } else if w % 10 == 0 { 3 } else { 4 };
    text(w, "\u{20ac}", stellen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aus_text_und_zurueck() {
        assert_eq!(aus_text("19,99").unwrap(), 199_900);
        assert_eq!(aus_text("1.234,56").unwrap(), 12_345_600);
        assert_eq!(aus_text("-0,0551").unwrap(), -551);
        assert_eq!(text(199_900, "\u{20ac}", 2), "19,99 \u{20ac}");
        assert_eq!(text(12_345_600, "", 2), "1.234,56");
    }

    #[test]
    fn aus_zahl_faellt_nicht_in_die_falle() {
        // Der Kern: 19.99 liegt als f64 minimal UNTER 19,99.
        assert_eq!(aus_zahl(19.99).unwrap(), 199_900);
        assert_eq!(aus_zahl(0.29).unwrap(), 2_900);
        assert_eq!(aus_zahl(0.1 + 0.2).unwrap(), 3_000);
    }

    #[test]
    fn mal_rechnet_ganzzahlig() {
        let netto = aus_text("72,71").unwrap();
        // 19 % von 72,71 sind 13,8149 -- exakt, nicht 13.814899999999998
        assert_eq!(mal(netto, 0.19).unwrap(), 138_149);
        assert_eq!(text(mal(netto, 0.19).unwrap(), "", 2), "13,81");
        assert_eq!(mal(aus_text("19,99").unwrap(), 3.0).unwrap(), 599_700);
    }

    #[test]
    fn durch_und_verhaeltnis() {
        assert_eq!(text(durch(aus_text("10,00").unwrap(), 4.0).unwrap(), "", 2), "2,50");
        assert!(durch(aus_text("1,00").unwrap(), 0.0).is_err());
    }

    #[test]
    fn runden_ist_kaufmaennisch() {
        assert_eq!(runden(aus_text("0,005").unwrap(), 2), 100);   // -> 0,01
        assert_eq!(runden(aus_text("-0,005").unwrap(), 2), -100);
        assert_eq!(runden(aus_text("0,0049").unwrap(), 2), 0);
        assert_eq!(runden(aus_text("2,675").unwrap(), 2), 26_800);
    }

    #[test]
    fn aufteilen_verliert_keinen_cent() {
        let zehn = aus_text("10,00").unwrap();
        let teile = teilen(zehn, 3).unwrap();
        assert_eq!(teile.iter().sum::<i64>(), zehn, "die Summe muss stimmen");
        assert_eq!(teile, vec![33_400, 33_300, 33_300]);

        let minus = teilen(aus_text("-10,00").unwrap(), 3).unwrap();
        assert_eq!(minus.iter().sum::<i64>(), -100_000);
        assert!(teilen(zehn, 0).is_err());
    }

    #[test]
    fn anzeige_verschweigt_keine_stellen() {
        assert_eq!(anzeige(199_900), "19,99 \u{20ac}");
        assert_eq!(anzeige(551), "0,0551 \u{20ac}");
        assert_eq!(anzeige(-199_900), "-19,99 \u{20ac}");
        assert_eq!(anzeige(0), "0,00 \u{20ac}");
    }

    #[test]
    fn zu_grosse_betraege_sind_ein_fehler() {
        assert!(mal(i64::MAX / 2, 1000.0).is_err());
    }
}
