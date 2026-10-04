//! Meldungen auf Englisch (`DHRT_LANG=en`).
//!
//! Die Laufzeit schreibt ihre Meldungen an rund 1600 Stellen deutsch -- und
//! dort bleiben sie: der Code, die Pruefsammlungen und die Schnellkorrekturen
//! (`lsp::korrekturen` liest den deutschen Satz) haengen daran. Uebersetzt
//! wird erst an der GRENZE, dort wo eine Meldung den Menschen erreicht
//! (Konsole, `--check`, Sprachserver, `CODE_CHECK$`, Debugger), und zwar
//! ueber einen Katalog: je Zeile eine deutsche Vorlage und ihre englische
//! Fassung, `{}` steht fuer ein eingesetztes Stueck.
//!
//! Was eine Vorlage in einem Platzhalter einfaengt, wird seinerseits
//! uebersetzt. Damit setzen sich zusammengesetzte Meldungen von selbst
//! zusammen: `{} -- Aufruf: {}` uebersetzt den Teil vor dem Strich mit seiner
//! eigenen Vorlage, `{} Meintest du {}?` haengt den Vorschlag an, ohne dass
//! jede Kombination im Katalog stehen muesste.
//!
//! Was keine Vorlage trifft, bleibt deutsch. Das ist Absicht: eine halbe oder
//! geratene Uebersetzung waere schlimmer als die richtige deutsche Meldung.
//! `dhrt pruef meldungen` haelt den Katalog an den Quelltext.
//!
//! Was ein PROGRAMM sieht (der Wert einer CATCH-Variable), bleibt deutsch --
//! sonst verhielte sich ein Programm je nach Umgebungsvariable anders.

use regex::Regex;
use std::sync::OnceLock;

/// Der Katalog, eingebettet: eine fertig gebaute Laufzeit braucht keine
/// Datei daneben (ein exportiertes Spiel hat keine).
const KATALOG: &str = include_str!("../../../daten/meldungen.en.txt");

/// Tiefe, bis zu der eingefangene Stuecke weiter uebersetzt werden.
const TIEFE: usize = 6;

pub struct Vorlage {
    pub deutsch: String,
    pub englisch: String,
    muster: Regex,
    /// Laengstes festes Stueck -- ein billiger Vorfilter vor dem Muster.
    anker: String,
    /// Zahl der festen Zeichen: die Vorlage mit den meisten gewinnt.
    gewicht: usize,
}

/// Soll Englisch ausgegeben werden? Bei jedem Aufruf gelesen: die IDE setzt
/// die Variable erst nach dem Start (SETENV), und `CODE_CHECK$` laeuft im
/// selben Prozess.
pub fn englisch() -> bool {
    match std::env::var("DHRT_LANG") {
        Ok(s) => {
            let s = s.trim().to_ascii_lowercase();
            s == "en" || s.starts_with("en_") || s.starts_with("en-")
        }
        Err(_) => false,
    }
}

/// Ein festes Wort je Sprache (Rahmen wie "Laufzeitfehler in").
pub fn wort(de: &'static str, en: &'static str) -> &'static str {
    if englisch() { en } else { de }
}

/// Die Meldung in der eingestellten Sprache.
pub fn t(text: &str) -> String {
    if englisch() { uebersetzen(text) } else { text.to_string() }
}

/// Die Meldung auf Englisch, unabhaengig von der Einstellung.
pub fn uebersetzen(text: &str) -> String {
    uebersetzen_tief(text, 0)
}

fn uebersetzen_tief(text: &str, tiefe: usize) -> String {
    tief(text, tiefe).0
}

/// Uebersetzung samt ihrem GESAMTgewicht: die festen Zeichen aller Vorlagen,
/// die dabei greifen. Nach diesem Gewicht wird gewaehlt, nicht nach dem der
/// obersten Vorlage -- sonst schnitte eine allgemeine Fuge wie `{} -- {}`
/// einen Satz an der falschen Stelle durch, nur weil sie auf der obersten
/// Ebene mehr feste Zeichen hat als `{}: {}`, und der Rest bliebe deutsch.
fn tief(text: &str, tiefe: usize) -> (String, usize) {
    if tiefe > TIEFE || text.trim().is_empty() {
        return (text.to_string(), 0);
    }
    // Ein Leerraum vorn oder hinten gehoert nicht zur Vorlage -- Anhaenge wie
    // " -- 'kein Wert' heisst ..." bringen ihn mit.
    let mut vorn = text.len() - text.trim_start().len();
    let hinten = text.trim_end().len();
    let mut kern = &text[vorn..hinten];
    // Ein Anhang beginnt mit " -- " (`umstieg::name_hinweis`, NIL_NEW) --
    // der Strich gehoert zur Fuge, nicht zur Vorlage.
    if let Some(r) = kern.strip_prefix("-- ") { vorn += 3; kern = r; }
    // Ein Hinweis, der als Satz angehaengt wird, bringt seinen Punkt mit
    // (" BITAND: ist in Drachenhauch der Operator BAND: a BAND b."). Ohne den
    // Punkt passt womoeglich eine genauere Vorlage.
    let mit = treffer(kern, tiefe);
    let ohne = kern.strip_suffix('.').and_then(|k| treffer(k, tiefe));
    match (mit, ohne) {
        (Some((g1, t)), Some((g2, _))) if g1 >= g2 => (format!("{}{}{}", &text[..vorn], t, &text[hinten..]), g1),
        (_, Some((g, t))) => (format!("{}{}.{}", &text[..vorn], t, &text[hinten..]), g),
        (Some((g, t)), None) => (format!("{}{}{}", &text[..vorn], t, &text[hinten..]), g),
        (None, None) => (text.to_string(), 0),
    }
}

/// Die beste Vorlage fuer genau diesen Text: (Gesamtgewicht, englisch). Bei
/// Gleichstand gewinnt die zuerst gefundene -- die mit mehr eigenen festen
/// Zeichen (die Liste ist danach sortiert).
fn treffer(kern: &str, tiefe: usize) -> Option<(usize, String)> {
    let mut best: Option<(usize, String)> = None;
    for v in vorlagen() {
        if !kern.contains(v.anker.as_str()) { continue; }
        let Some(c) = v.muster.captures(kern) else { continue };
        let mut gewicht = v.gewicht;
        let stuecke: Vec<String> = (1..c.len())
            .map(|i| {
                let (t, g) = tief(c.get(i).map_or("", |m| m.as_str()), tiefe + 1);
                gewicht += g;
                t
            })
            .collect();
        if best.as_ref().is_none_or(|(b, _)| gewicht > *b) {
            best = Some((gewicht, einsetzen(&v.englisch, &stuecke)));
        }
    }
    best
}

/// `{}` der Reihe nach, `{2}` gezielt (ab 1) -- das Englische stellt Teile
/// gern um. `{{` und `}}` sind geschweifte Klammern.
fn einsetzen(vorlage: &str, stuecke: &[String]) -> String {
    let mut aus = String::new();
    let mut naechstes = 0;
    let z: Vec<char> = vorlage.chars().collect();
    let mut i = 0;
    while i < z.len() {
        match z[i] {
            '{' if z.get(i + 1) == Some(&'{') => { aus.push('{'); i += 2; }
            '}' if z.get(i + 1) == Some(&'}') => { aus.push('}'); i += 2; }
            '{' => {
                let ende = z[i..].iter().position(|&c| c == '}').map(|p| i + p);
                match ende {
                    Some(e) => {
                        let innen: String = z[i + 1..e].iter().collect();
                        let nr: usize = if innen.is_empty() { naechstes += 1; naechstes }
                                 else { innen.parse().unwrap_or(0) };
                        if let Some(s) = nr.checked_sub(1).and_then(|k| stuecke.get(k)) { aus.push_str(s); }
                        i = e + 1;
                    }
                    None => { aus.push('{'); i += 1; }
                }
            }
            c => { aus.push(c); i += 1; }
        }
    }
    aus
}

/// Die festen Stuecke einer deutschen Vorlage (zwischen den Platzhaltern).
/// `[[` und `]]` markieren fuer `dhrt pruef meldungen` die Nahtstellen
/// zwischen zwei Literalen und gehoeren nicht zum Text.
pub fn stuecke(vorlage: &str) -> Vec<String> {
    stuecke_roh(&vorlage.replace("[[", "").replace("]]", ""))
}

/// Wie `stuecke`, aber mit den Nahtmarken -- fuer die Pruefung.
pub fn stuecke_roh(vorlage: &str) -> Vec<String> {
    let mut teile = vec![String::new()];
    let z: Vec<char> = vorlage.chars().collect();
    let mut i = 0;
    while i < z.len() {
        if z[i] == '{' && z.get(i + 1) == Some(&'{') { teile.last_mut().unwrap().push('{'); i += 2; }
        else if z[i] == '}' && z.get(i + 1) == Some(&'}') { teile.last_mut().unwrap().push('}'); i += 2; }
        else if z[i] == '{' && z.get(i + 1) == Some(&'}') { teile.push(String::new()); i += 2; }
        else { teile.last_mut().unwrap().push(z[i]); i += 1; }
    }
    teile
}

/// Wie viele Stuecke die englische Fassung einsetzt (fuer die Pruefung).
pub fn platzhalter_englisch(vorlage: &str) -> (usize, usize) {
    // (Zahl der {}, groesste {n})
    let mut leer = 0;
    let mut groesste = 0;
    let z: Vec<char> = vorlage.chars().collect();
    let mut i = 0;
    while i < z.len() {
        if (z[i] == '{' || z[i] == '}') && z.get(i + 1) == Some(&z[i]) { i += 2; continue; }
        if z[i] == '{' {
            if let Some(p) = z[i..].iter().position(|&c| c == '}') {
                let innen: String = z[i + 1..i + p].iter().collect();
                if innen.is_empty() { leer += 1 } else if let Ok(n) = innen.parse::<usize>() { groesste = groesste.max(n) }
                i += p + 1;
                continue;
            }
        }
        i += 1;
    }
    (leer, groesste)
}

/// Die Zeilen des Katalogs als (deutsch, englisch, Zeilennummer). Eine Zeile
/// ohne Tabulator oder mit `#` vorn ist ein Kommentar.
pub fn katalog_zeilen(text: &str) -> Vec<(String, String, usize)> {
    text.lines().enumerate().filter_map(|(n, z)| {
        if z.starts_with('#') { return None; }
        let (de, en) = z.split_once('\t')?;
        if de.trim().is_empty() || en.trim().is_empty() { return None; }
        Some((de.to_string(), en.to_string(), n + 1))
    }).collect()
}

fn vorlage_bauen(de: &str, en: &str) -> Option<Vorlage> {
    let teile = stuecke(de);
    let mut muster = String::from("(?s)^");
    for (i, t) in teile.iter().enumerate() {
        if i > 0 { muster.push_str("(.*?)"); }
        muster.push_str(&regex::escape(t));
    }
    muster.push('$');
    let anker = teile.iter().max_by_key(|t| t.len()).cloned().unwrap_or_default();
    let gewicht = teile.iter().map(|t| t.chars().count()).sum();
    Some(Vorlage { deutsch: de.to_string(), englisch: en.to_string(),
                   muster: Regex::new(&muster).ok()?, anker, gewicht })
}

pub fn vorlagen() -> &'static [Vorlage] {
    static V: OnceLock<Vec<Vorlage>> = OnceLock::new();
    V.get_or_init(|| {
        let mut v: Vec<Vorlage> = katalog_zeilen(KATALOG).into_iter()
            .filter_map(|(de, en, _)| vorlage_bauen(&de, &en))
            .collect();
        // Die genaueste zuerst; bei Gleichstand die Reihenfolge der Datei.
        v.sort_by(|a, b| b.gewicht.cmp(&a.gewicht));
        v
    })
}

/// Der eingebettete Katalog als Text (fuer `dhrt pruef meldungen`).
pub fn katalog_text() -> &'static str { KATALOG }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ganze_vorlagen_und_platzhalter() {
        assert_eq!(uebersetzen("Division durch 0"), "Division by zero");
        assert_eq!(uebersetzen("Erwartet THEN nach Bedingung"), "Expected THEN after the condition");
        assert_eq!(uebersetzen("Variable 'zaehlr' nicht deklariert (DIM fehlt?)"),
                   "Variable 'zaehlr' is not declared (missing DIM?)");
    }

    #[test]
    fn zusammengesetzt_uebersetzt_jeden_teil() {
        let de = "Variable 'x' nicht deklariert (DIM fehlt?) Meintest du XY?";
        assert_eq!(uebersetzen(de), "Variable 'x' is not declared (missing DIM?) Did you mean XY?");
        let de = "CIRCLE: erwartet Zahl, erhalten STRING -- Aufruf: CIRCLE(x, y, r [, farbe])";
        assert_eq!(uebersetzen(de), "CIRCLE: expected a number, got STRING -- usage: CIRCLE(x, y, r [, farbe])");
    }

    #[test]
    fn die_ganze_uebersetzung_zaehlt_nicht_die_oberste_vorlage() {
        // `{} -- {}` hat oben mehr feste Zeichen als `{}: {}`, schnitte den
        // Satz aber hinter "auch keine CONST" durch -- der Rest bliebe deutsch.
        let de = "TASK: Zugriff auf eine globale Variable, die noch nicht gesetzt ist. Laeuft das hier als \
                  Auftrag (`dhrt call` / TASK_START)? Dann ist das erwartet: das Hauptprogramm laeuft dabei \
                  NICHT, also ist kein Global gesetzt -- auch keine CONST, deren Wert erst beim Laufen \
                  feststeht (eine mit festem Wert wie `CONST BREITE = 640` setzt der Compiler ein). Gib der \
                  Funktion als Parameter mit, was sie braucht.";
        assert!(uebersetzen(de).starts_with("TASK: Access to a global variable"), "{}", uebersetzen(de));
        // Der Hinweis hinter "Unbekanntes Builtin" kommt mit seinem Punkt.
        let de = "Unbekanntes Builtin 'BITAND' -- dhrt kennt es nicht (Tippfehler? oder veraltet/entfernt). \
                  Der Aufruf schlaegt sonst erst zur Laufzeit fehl. BITAND: ist in Drachenhauch der Operator \
                  BAND: a BAND b.";
        assert!(uebersetzen(de).ends_with("BITAND: is the operator BAND in Drachenhauch: a BAND b."), "{}", uebersetzen(de));
    }

    #[test]
    fn unbekanntes_bleibt_wie_es_ist() {
        assert_eq!(uebersetzen("Das hier steht in keinem Katalog"), "Das hier steht in keinem Katalog");
        assert_eq!(uebersetzen("  "), "  ");
    }

    #[test]
    fn einsetzen_mit_nummern_und_klammern() {
        let s = vec!["a".to_string(), "b".to_string()];
        assert_eq!(einsetzen("{2} vor {1}", &s), "b vor a");
        assert_eq!(einsetzen("{{x}} {}", &s), "{x} a");
        assert_eq!(stuecke("ab{}c{{d}}"), vec!["ab".to_string(), "c{d}".to_string()]);
    }

    #[test]
    fn katalog_ist_in_sich_stimmig() {
        for (de, en, n) in katalog_zeilen(KATALOG) {
            let deutsch = stuecke(&de).len() - 1;
            let (leer, groesste) = platzhalter_englisch(&en);
            assert!(leer == 0 || groesste == 0, "Zeile {}: {{}} und {{n}} gemischt", n);
            let englisch = leer.max(groesste);
            assert!(englisch <= deutsch, "Zeile {}: {} Platzhalter englisch, {} deutsch", n, englisch, deutsch);
            if groesste == 0 { assert_eq!(leer, deutsch, "Zeile {}: {}", n, de); }
            assert!(vorlage_bauen(&de, &en).is_some(), "Zeile {}", n);
        }
    }
}
