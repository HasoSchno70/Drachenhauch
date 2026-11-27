//! Getaggte Handles (Bericht A4.1).
//!
//! Die meisten Module geben ihre Dinge als NUMMER heraus -- einen Index in
//! einem `Vec` der VM (ein PDF, eine Datenbankverbindung, ein Bild, ein
//! Widget). Bis 2026.53 war diese Nummer ein nacktes INTEGER: `TYPEOF(db)`
//! sagte INTEGER, `db IS DB_CONN` war FALSE, und ein PDF-Handle an einen
//! XLSX-Befehl traf still die Mappe mit derselben Nummer.
//!
//! Seitdem traegt die Nummer ihre Art: `Value::Handle(art, nr)`. Vergeben und
//! geprueft wird an EINER Stelle, beim Aufruf eines eingebauten Befehls
//! (`Vm::builtin_rufen`), nach den Typen in `daten/builtin_index.json`:
//!
//! * Liefert ein Befehl laut Index eine Handle-Art (`PDF_NEW() AS PDF`), wird
//!   seine Nummer getaggt (negative Nummern -- „kein Handle“ -- bleiben Zahlen).
//! * Erwartet ein Parameter eine Art (`PDF_TEXT(p: PDF, ...)`) und kommt ein
//!   Handle einer ANDEREN Art, ist das ein Fehler, der beide nennt. Eine nackte
//!   Zahl geht weiter durch (Programme, die Handles in `ARRAY OF INTEGER`
//!   halten). Welche Parameter welche Art nehmen, steht im Index; eingetragen
//!   hat sie 2026-11 ein Skript nach Parameternamen je Modul (`image`,
//!   `conn`, `win` ...) und nach dem Code (wer legt Texturen, Klaenge,
//!   Widgets an). Ein Befehl ohne Eintrag prueft nichts.
//! * Vor dem Befehl selbst werden Handles wieder zu Zahlen -- die Module
//!   sehen nichts davon. Ausnahmen: die Befehle, die einen Wert nur ablegen
//!   oder ansehen (`ROH`), damit das Tag in einem Feld erhalten bleibt.
//!
//! Nach aussen rechnet ein Handle wie seine Nummer (`h = 0`, `h < 0`,
//! `PRINT h`); eine Zuweisung an INTEGER nimmt die Nummer, eine an den Typ
//! der Art taggt eine nackte Zahl (`coerce`).

use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::OnceLock;

/// Die Handle-Arten, aus der Typtafel (`typtafel::TYPEN`, Art Handle). Die
/// Stelle in der Liste ist die Art im Wert (`u16`) -- nur fuer diesen Lauf,
/// gespeichert wird sie nirgends.
pub fn arten() -> &'static [&'static str] {
    static A: OnceLock<Vec<&'static str>> = OnceLock::new();
    A.get_or_init(|| crate::typtafel::der_art(crate::typtafel::Art::Handle).map(|t| t.name).collect())
}

/// Der Name einer Art (`TYPEOF`, Meldungen).
pub fn name(art: u16) -> &'static str {
    arten().get(art as usize).copied().unwrap_or("HANDLE")
}

/// Die Art zu einem Typnamen (jede Schreibweise), sonst `None`.
pub fn art(typ: &str) -> Option<u16> {
    arten().iter().position(|a| a.eq_ignore_ascii_case(typ)).map(|i| i as u16)
}

/// Was der Index ueber die Handles eines Befehls sagt.
#[derive(Default, Debug, PartialEq)]
pub struct Befehl {
    /// Die Art, die er liefert.
    pub liefert: Option<u16>,
    /// Die Parameter mit einer Art: (Stelle, Art).
    pub nimmt: Vec<(usize, u16)>,
}

/// Befehle, die einen Wert nur ablegen oder ansehen: ihre Argumente behalten
/// das Tag (ein Handle in einem `ARRAY OF ANY` bleibt ein Handle).
pub const ROH: &[&str] = &[
    "typeof", "__is_typ", "__array_of_type", "array_push", "array_insert", "array_fill",
    "array_index_of", "array_indexof", "array_contains", "array_count", "array_remove_value",
    "mapput", "map_put", "set_add", "set_has", "set_remove", "tuple", "assert_eq", "assert",
    "__comp_iter", "iif", "str$", "str", "print",
];

/// Die Tafel aller Befehle mit Handles, aus den Signaturen im Index
/// (Schluessel klein, wie die Laufzeit Namen sieht).
pub fn tafel() -> &'static FxHashMap<String, Befehl> {
    static T: OnceLock<FxHashMap<String, Befehl>> = OnceLock::new();
    T.get_or_init(|| {
        let mut m = FxHashMap::default();
        for (name, sig, _) in crate::compiler::builtin_eintraege() {
            let b = lesen(sig);
            if b.liefert.is_some() || !b.nimmt.is_empty() {
                m.insert(name.to_lowercase(), b);
            }
        }
        m
    })
}

/// Liefert der Befehl ein Handle? (`model::func_bauen` merkt es sich je
/// Aufrufstelle, damit die Laufzeit nicht bei jedem Aufruf nachschlaegt.)
pub fn liefert(name: &str) -> Option<u16> {
    tafel().get(name).and_then(|b| b.liefert)
}

/// Eine Signatur lesen: `NAME(a: PDF, b [, c AS XLSX]) AS IMAGE`.
pub fn lesen(sig: &str) -> Befehl {
    let mut b = Befehl::default();
    let Some(auf) = sig.find('(') else { return b };
    // Die schliessende Klammer der Parameterliste (Klammern darin zaehlen).
    let mut tiefe = 0;
    let mut zu = None;
    for (i, c) in sig[auf..].char_indices() {
        match c {
            '(' => tiefe += 1,
            ')' => { tiefe -= 1; if tiefe == 0 { zu = Some(auf + i); break; } }
            _ => {}
        }
    }
    let Some(zu) = zu else { return b };
    let rest = sig[zu + 1..].trim();
    if let Some(t) = rest.strip_prefix("AS ").or_else(|| rest.strip_prefix("->")) {
        b.liefert = art(t.trim());
    }
    let innen: String = sig[auf + 1..zu].chars().filter(|c| *c != '[' && *c != ']').collect();
    for (i, p) in innen.split(',').enumerate() {
        let p = p.trim();
        let typ = p.split_once(':').map(|(_, t)| t)
            .or_else(|| p.split_once(" AS ").map(|(_, t)| t))
            .map(|t| t.split('=').next().unwrap_or("").trim());
        if let Some(a) = typ.and_then(art) {
            b.nimmt.push((i, a));
        }
    }
    b
}

/// Ein Handle an einem Parameter einer anderen Art ist ein Fehler (die Arten
/// stehen in den Signaturen des Index, `tafel`).
pub fn arten_pruefen(name: &str, a: &[crate::value::Value]) -> Result<(), String> {
    use crate::value::Value;
    if let Some(b) = tafel().get(name) {
        for &(stelle, soll) in &b.nimmt {
            if let Some(Value::Handle(ist, _)) = a.get(stelle) {
                if *ist != soll { return Err(verwechselt(name, stelle, soll, *ist)); }
            }
        }
    }
    Ok(())
}

/// Die Arten pruefen und die Handles zu nackten Nummern machen -- so sehen
/// die Befehle sie.
pub fn auspacken(name: &str, a: &[crate::value::Value]) -> Result<Vec<crate::value::Value>, String> {
    arten_pruefen(name, a)?;
    Ok(a.iter().map(|v| v.ohne_handle()).collect())
}

/// Nimmt der Befehl Handles roh (`ROH`)? Eine Menge statt der Liste: das
/// fragt jeder Aufruf mit Handle-Argument.
pub fn roh(name: &str) -> bool {
    static M: OnceLock<FxHashSet<&'static str>> = OnceLock::new();
    M.get_or_init(|| ROH.iter().copied().collect()).contains(name)
}

/// Ein Handle an einem Parameter einer anderen Art.
pub fn verwechselt(fn_: &str, stelle: usize, soll: u16, ist: u16) -> String {
    format!("{}: Argument {} erwartet ein {}-Handle, erhalten ein {}-Handle",
            fn_.to_uppercase(), stelle + 1, name(soll), name(ist))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signaturen_lesen() {
        let b = lesen("PDF_TEXT(p: PDF, x, y, text$) AS INTEGER");
        assert_eq!(b, Befehl { liefert: None, nimmt: vec![(0, art("pdf").unwrap())] });
        let b = lesen("XLSX_NEW(sheet: STRING = \"Tabelle1\") AS XLSX");
        assert_eq!(b.liefert, art("XLSX"));
        assert_eq!(lesen("SMTP_NEW() -> SMTP").liefert, art("SMTP"));
        let b = lesen("IMAGE_DRAW(img AS IMAGE, x [, y: Zahl = 0, ziel: IMAGE])");
        assert_eq!(b.nimmt, vec![(0, art("image").unwrap()), (3, art("image").unwrap())]);
        assert_eq!(lesen("ABS(x)"), Befehl::default());
    }

    #[test]
    fn jede_art_ist_ein_typ() {
        // Jede Art muss als Typ deklarierbar sein (DIM x AS PDF) -- sonst
        // liefert TYPEOF einen Namen, den das Programm nicht schreiben kann.
        for a in arten() {
            let t = a.to_lowercase();
            let bekannt = crate::preprocess::modul_typen().any(|m| m == t)
                || matches!(t.as_str(), "image" | "sound" | "sprite_atlas");
            assert!(bekannt, "{} ist kein Typ", a);
            assert!(crate::value::wert_typ(&t).is_none(), "{} ist ein Werttyp", a);
        }
        assert!(arten().len() < u16::MAX as usize);
        assert_eq!(arten().len(), 36, "eine Art mehr oder weniger -- gewollt?");
        // AUDIO_CHANNEL ist keine eigene Art: ein Kanal IST die Nummer seines
        // Klangs (`audio::ch_play`), beide Namen meinen dasselbe Ding.
        assert!(art("audio_channel").is_none());
    }
}
