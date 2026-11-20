//! Eigene Namen so schreiben, wie sie angelegt sind.
//!
//! Drachenhauch unterscheidet nicht zwischen Gross und Klein: `cw` und `CW`
//! sind derselbe Name. Das bleibt so -- jedes Programm, Beispiel und Buch
//! verliesse sich sonst auf zwei verschiedene Dinge, wo heute eins ist. Wer
//! aber `CONST CW` anlegt und spaeter `cw` schreibt, liest zwei Dinge, wo
//! eins ist; mit einer lokalen `cw` daneben weiss niemand mehr, welches
//! gemeint ist (so geschehen in make_tiles.dh: `kante` verdeckte `KANTE`).
//! Darum gilt wie in VB: Gross/klein ist egal, aber einheitlich. Diese
//! Pruefung meldet jede Stelle, an der ein eigener Name anders geschrieben
//! ist als bei seiner Deklaration, und `angleichen` schreibt sie richtig
//! (Schnellkorrektur im Editor, `dhrt fmt --schreibweise`).
//!
//! Bewusst am Quelltext und nicht am AST: der Lexer schreibt alles klein.
//! Welche Deklaration eine Stelle meint, entscheidet der Bereich -- wie im
//! Uebersetzer: eine lokale Variable oder ein Parameter der Funktion um die
//! Stelle, dann ein Feld oder eine Methode der Klasse um sie herum, dann ein
//! globaler Name. Befehle (`LEN`, `Print`) und Schluesselwoerter schreibt
//! jeder, wie er will -- sie sind keine eigenen Namen.

use crate::symbole::{bereiche, definitionen, ohne_kommentare_und_texte, Bereich, Definition};
use std::collections::HashMap;

/// Eine Stelle, an der ein Name anders geschrieben ist als angelegt.
#[derive(Debug, Clone, PartialEq)]
pub struct Abweichung {
    /// 1-basiert.
    pub zeile: usize,
    /// 1-basiert, Zeichen.
    pub spalte: usize,
    /// 1-basiert, exklusiv.
    pub spalte_ende: usize,
    pub geschrieben: String,
    pub angelegt: String,
}

/// Wo eine Deklaration gilt.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Gilt {
    /// Im ganzen Programm.
    Global,
    /// In einer SUB/FUNCTION/PROPERTY (Zeilen von..=bis).
    Block(usize, usize),
    /// Als Mitglied (Feld, Methode, Konstante) der Klasse von..=bis.
    Mitglied(usize, usize),
}

fn aufrufbar(b: &Bereich) -> bool { matches!(b.art, "sub" | "function" | "property") }
fn klasse(b: &Bereich) -> bool { matches!(b.art, "class" | "struct") }

/// Der innerste Bereich der Art um Zeile `z` (Kopfzeile ausgeschlossen).
fn innerster<'a>(bl: &'a [Bereich], z: usize, art: fn(&Bereich) -> bool) -> Option<&'a Bereich> {
    bl.iter().filter(|b| art(b) && b.zeile < z && z <= b.ende).max_by_key(|b| b.zeile)
}

fn gilt(d: &Definition, bl: &[Bereich]) -> Gilt {
    if d.art == "param" {
        // Der Kopf darf ueber mehrere Zeilen gehen (in Klammern geht eine
        // Zeile weiter): ein Parameter auf der zweiten Zeile gehoert genauso
        // zur SUB. Vorher galt er dann als global, und neben `CONST BREITE`
        // hiess jedes `breite` im Rumpf "anders geschrieben als angelegt".
        return bl.iter().filter(|b| aufrufbar(b) && b.zeile <= d.zeile && d.zeile <= b.ende)
            .max_by_key(|b| b.zeile)
            .map(|b| Gilt::Block(b.zeile, b.ende)).unwrap_or(Gilt::Global);
    }
    match d.art {
        "class" | "struct" | "enum" => Gilt::Global,
        "sub" | "function" | "property" => match innerster(bl, d.zeile, klasse) {
            Some(k) => Gilt::Mitglied(k.zeile, k.ende),
            None => Gilt::Global,
        },
        _ => {
            if let Some(f) = innerster(bl, d.zeile, aufrufbar) { return Gilt::Block(f.zeile, f.ende); }
            match innerster(bl, d.zeile, klasse) {
                Some(k) => Gilt::Mitglied(k.zeile, k.ende),
                None => Gilt::Global,
            }
        }
    }
}

/// Ein Name im Code: Zeile, Spalten (1-basiert), Text ohne `$`, ob ein `$`
/// folgt (`text$` ist ein anderer Name als `text`), ob ein Punkt davor steht
/// und das Wort vor dem Punkt, ob `(` oder `:=` folgt, das Wort davor.
struct Wort { zeile: usize, spalte: usize, ende: usize, text: String, dollar: bool, nach_punkt: bool,
              objekt: String, aufruf: bool, benannt: bool, davor: String }

fn woerter(quelle: &str) -> Vec<Wort> {
    let mut aus = Vec::new();
    for (i, roh) in quelle.split('\n').enumerate() {
        let z: Vec<char> = ohne_kommentare_und_texte(roh).chars().collect();
        let mut k = 0;
        let mut letztes = String::new();
        let mut vor_punkt = String::new();
        while k < z.len() {
            let c = z[k];
            if c.is_ascii_alphabetic() || c == '_' {
                // Teil einer Zahl (`1e5`, `&HFF`)? Dann kein Name.
                let in_zahl = k > 0 && (z[k - 1].is_ascii_digit() || z[k - 1] == '&');
                let a = k;
                while k < z.len() && (z[k].is_ascii_alphanumeric() || z[k] == '_') { k += 1; }
                let text: String = z[a..k].iter().collect();
                let dollar = k < z.len() && z[k] == '$';
                if dollar { k += 1; }
                if in_zahl { letztes = text; continue; }
                let mut v = a;
                while v > 0 && z[v - 1] == ' ' { v -= 1; }
                let nach_punkt = v > 0 && z[v - 1] == '.';
                let mut n = k;
                while n < z.len() && z[n] == ' ' { n += 1; }
                let aufruf = n < z.len() && z[n] == '(';
                let benannt = n + 1 < z.len() && z[n] == ':' && z[n + 1] == '=';
                aus.push(Wort { zeile: i + 1, spalte: a + 1, ende: a + 1 + text.chars().count(),
                                text: text.clone(), dollar, nach_punkt,
                                objekt: if nach_punkt { vor_punkt.to_ascii_lowercase() } else { String::new() },
                                aufruf, benannt, davor: letztes.to_ascii_lowercase() });
                letztes = text;
                continue;
            }
            if c == '.' { vor_punkt = letztes.clone(); }
            else if !c.is_whitespace() { vor_punkt.clear(); }
            if !c.is_whitespace() { letztes.clear(); }
            k += 1;
        }
    }
    aus
}

/// Alle Stellen, an denen ein eigener Name anders geschrieben ist als bei
/// seiner Deklaration (Quell-Reihenfolge).
pub fn pruefen(quelle: &str) -> Vec<Abweichung> {
    let bl = bereiche(quelle);
    let defs = definitionen(quelle);
    let mut nach_name: HashMap<String, Vec<(&Definition, Gilt)>> = HashMap::new();
    let mut ort: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let zeilen: Vec<Vec<char>> = quelle.split('\n').map(|z| z.chars().collect()).collect();
    let mut enums: std::collections::HashSet<String> = std::collections::HashSet::new();
    for d in &defs {
        // `FUNCTION text$` heisst `text$`, nicht `text`.
        let dollar = zeilen.get(d.zeile - 1).and_then(|z| z.get(d.spalte_ende - 1)) == Some(&'$');
        let schluessel = format!("{}{}", d.name.to_ascii_lowercase(), if dollar { "$" } else { "" });
        nach_name.entry(schluessel).or_default().push((d, gilt(d, &bl)));
        ort.insert((d.zeile, d.spalte));
        if d.art == "enum" { enums.insert(d.name.to_ascii_lowercase()); }
    }
    let mut aus = Vec::new();
    for w in woerter(quelle) {
        if ort.contains(&(w.zeile, w.spalte)) || w.benannt { continue; }
        let klein = w.text.to_ascii_lowercase();
        // Schluesselwoerter (auch die nur im Zusammenhang, `EACH`, `EXIT` ...)
        // schreibt jeder, wie er will -- auch neben einer Variable gleichen Namens.
        if crate::lexer::KEYWORDS.contains(&klein.as_str())
            || matches!(klein.as_str(), "swap" | "let" | "exit" | "do" | "each" | "iif" | "loop" | "until" | "case" | "select") {
            continue;
        }
        // `St.DIVE` ist ein Wert der Aufzaehlung St, keine Methode `Dive`.
        if w.nach_punkt && enums.contains(&w.objekt) { continue; }
        let schluessel = format!("{}{}", klein, if w.dollar { "$" } else { "" });
        let Some(kand) = nach_name.get(&schluessel) else { continue };
        let typ_stelle = matches!(w.davor.as_str(), "as" | "new" | "of" | "extends" | "is");
        let gemeint: Option<&Definition> = if w.nach_punkt {
            // Mitglied hinter einem Punkt: nur wenn alle Klassen es gleich schreiben.
            let m: Vec<&Definition> = kand.iter().filter(|(_, g)| matches!(g, Gilt::Mitglied(..))).map(|(d, _)| *d).collect();
            match m.first() {
                Some(e) if m.iter().all(|d| d.name == e.name) => Some(*e),
                _ => None,
            }
        } else if typ_stelle {
            kand.iter().find(|(d, _)| matches!(d.art, "class" | "struct" | "enum")).map(|(d, _)| *d)
        } else {
            let im_block = kand.iter().filter_map(|(d, g)| match g {
                Gilt::Block(a, b) if *a <= w.zeile && w.zeile <= *b => Some((*a, *d)), _ => None })
                .max_by_key(|(a, _)| *a).map(|(_, d)| d);
            let in_klasse = || kand.iter().filter_map(|(d, g)| match g {
                Gilt::Mitglied(a, b) if *a < w.zeile && w.zeile <= *b => Some((*a, *d)), _ => None })
                .max_by_key(|(a, _)| *a).map(|(_, d)| d);
            let global = || kand.iter().find(|(d, g)| *g == Gilt::Global && !matches!(d.art, "class" | "struct" | "enum")).map(|(d, _)| *d)
                .or_else(|| kand.iter().find(|(_, g)| *g == Gilt::Global).map(|(d, _)| *d));
            // Geerbt: in einer Klasse ein Mitglied einer anderen, wenn alle
            // Klassen es gleich schreiben.
            let geerbt = || {
                innerster(&bl, w.zeile, klasse)?;
                let m: Vec<&Definition> = kand.iter().filter(|(_, g)| matches!(g, Gilt::Mitglied(..))).map(|(d, _)| *d).collect();
                match m.first() { Some(e) if m.iter().all(|d| d.name == e.name) => Some(*e), _ => None }
            };
            im_block.or_else(in_klasse).or_else(geerbt).or_else(global)
        };
        let Some(d) = gemeint else { continue };
        // `LEN(s)` neben einer Variable `len` meint den Befehl.
        if w.aufruf && !matches!(d.art, "sub" | "function" | "property")
            && crate::compiler::is_known_builtin(&schluessel) { continue; }
        if w.text != d.name {
            aus.push(Abweichung { zeile: w.zeile, spalte: w.spalte, spalte_ende: w.ende,
                                  geschrieben: format!("{}{}", w.text, if w.dollar { "$" } else { "" }),
                                  angelegt: format!("{}{}", d.name, if w.dollar { "$" } else { "" }) });
        }
    }
    aus
}

/// Die Meldung zu einer Abweichung -- `korrekturen` liest die beiden Namen
/// wieder heraus.
pub fn meldung(a: &Abweichung) -> String {
    format!("'{}' ist als '{}' angelegt -- Gross-/Kleinschreibung zaehlt nicht, es ist derselbe Name. Bitte so schreiben wie dort.",
            a.geschrieben, a.angelegt)
}

/// Die beiden Namen aus einer Meldung von `meldung`.
pub fn aus_meldung(m: &str) -> Option<(String, String)> {
    let rest = m.strip_prefix('\'')?;
    let (geschrieben, rest) = rest.split_once("' ist als '")?;
    let (angelegt, _) = rest.split_once("' angelegt -- Gross-/Kleinschreibung zaehlt nicht")?;
    Some((geschrieben.to_string(), angelegt.to_string()))
}

/// Warnungen je Zeile (eine je Zeile und Schreibweise) fuer den Uebersetzer-Weg.
pub fn warnungen(quelle: &str) -> Vec<(u32, String)> {
    let mut gesehen = std::collections::HashSet::new();
    pruefen(quelle).into_iter()
        .filter(|a| gesehen.insert((a.zeile, a.geschrieben.clone())))
        .map(|a| (a.zeile as u32, meldung(&a)))
        .collect()
}

/// Den Text mit allen Namen so geschrieben, wie sie angelegt sind.
pub fn angleichen(quelle: &str) -> String {
    let ab = pruefen(quelle);
    if ab.is_empty() { return quelle.to_string(); }
    let mut je_zeile: HashMap<usize, Vec<&Abweichung>> = HashMap::new();
    for a in &ab { je_zeile.entry(a.zeile).or_default().push(a); }
    quelle.split('\n').enumerate().map(|(i, z)| {
        let Some(l) = je_zeile.get(&(i + 1)) else { return z.to_string() };
        let mut c: Vec<char> = z.chars().collect();
        let mut l = l.clone();
        l.sort_by_key(|a| std::cmp::Reverse(a.spalte));
        for a in l {
            let neu: Vec<char> = a.angelegt.trim_end_matches('$').chars().collect();
            c.splice(a.spalte - 1..a.spalte_ende - 1, neu);
        }
        c.into_iter().collect()
    }).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn namen(q: &str) -> Vec<(usize, String, String)> {
        pruefen(q).into_iter().map(|a| (a.zeile, a.geschrieben, a.angelegt)).collect()
    }

    #[test]
    fn globale_konstante_und_variable() {
        let q = "CONST CW = 4\nDIM Wert AS INTEGER\nwert = cw * 2\nPRINT Wert, CW";
        assert_eq!(namen(q), vec![(3, "wert".into(), "Wert".into()), (3, "cw".into(), "CW".into())]);
    }

    #[test]
    fn lokale_verdeckt_globale() {
        // In der SUB meint `cw` die lokale -- gleich geschrieben, keine Meldung;
        // `CW` dort ist die lokale, anders geschrieben.
        let q = "CONST CW = 4\nSUB s()\n    DIM cw AS INTEGER\n    cw = 1\n    PRINT CW\nEND SUB\nPRINT CW";
        assert_eq!(namen(q), vec![(5, "CW".into(), "cw".into())]);
    }

    #[test]
    fn parameter_felder_methoden() {
        let q = "CLASS Punkt\n    DIM xPos AS INTEGER\n    SUB Setzen(neuX AS INTEGER)\n        xpos = NEUX\n    END SUB\nEND CLASS\nDIM p AS punkt\np = NEW Punkt()\np.setzen(3)\nPRINT p.XPos";
        assert_eq!(namen(q), vec![
            (4, "xpos".into(), "xPos".into()), (4, "NEUX".into(), "neuX".into()),
            (7, "punkt".into(), "Punkt".into()), (9, "setzen".into(), "Setzen".into()),
            (10, "XPos".into(), "xPos".into())]);
    }

    #[test]
    fn befehle_texte_kommentare_bleiben() {
        let q = "DIM len AS INTEGER\nlen = LEN(\"abc\")\nPRINT \"LEN\" ' Len\nprint len";
        assert!(pruefen(q).is_empty(), "{:?}", pruefen(q));
    }

    #[test]
    fn benannte_argumente_und_zahlen() {
        let q = "DIM X AS INTEGER\nSUB f(x AS INTEGER)\nEND SUB\nf(x := 1)\nPRINT &HFF, 1e5";
        assert!(pruefen(q).is_empty(), "{:?}", pruefen(q));
    }

    #[test]
    fn mitglieder_verschiedener_klassen_mit_eigener_schreibweise() {
        // Zwei Klassen schreiben `name` verschieden -- hinter dem Punkt ist
        // nicht zu sagen, welche gemeint ist, also keine Meldung.
        let q = "CLASS A\n    DIM Name AS STRING\nEND CLASS\nCLASS B\n    DIM name AS STRING\nEND CLASS\nDIM a AS A\nPRINT a.NAME";
        assert!(pruefen(q).is_empty(), "{:?}", pruefen(q));
    }

    #[test]
    fn dollar_gehoert_zum_namen_und_enum_werte_sind_keine_mitglieder() {
        let q = "FUNCTION text$(s AS STRING) AS STRING
    RETURN s
END FUNCTION
TEXT(1, 2, text$(\"a\"))
ENUM St = ENTER, DIVE
CLASS G
    SUB Dive()
    END SUB
END CLASS
PRINT St.DIVE
PRINT TEXT$(\"b\")";
        assert_eq!(namen(q), vec![(11, "TEXT$".into(), "text$".into())]);
    }

    #[test]
    fn escape_ketten_sind_text() {
        // Das Studio schreibt Formulare als `!"{\"text\":\"Eins\"}"` -- `Eins`
        // ist Text, kein Name (vorher endete die Kette am `\"`).
        let q = "DIM eins AS INTEGER\nPRINT !\"{\\\"text\\\":\\\"Eins\\\"}\", eins";
        assert!(pruefen(q).is_empty(), "{:?}", pruefen(q));
    }

    #[test]
    fn angleichen_schreibt_wie_angelegt() {
        let q = "CONST CW = 4\nDIM Wert AS INTEGER\nwert = cw * 2 + cw";
        assert_eq!(angleichen(q), "CONST CW = 4\nDIM Wert AS INTEGER\nWert = CW * 2 + CW");
        assert_eq!(aus_meldung(&meldung(&pruefen(q)[0])), Some(("wert".into(), "Wert".into())));
    }
}
