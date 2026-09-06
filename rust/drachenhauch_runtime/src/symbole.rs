//! Symbole im Quelltext: Definitionen, Fundstellen, Bloecke, Kommentar-Doku.
//!
//! Der Port von `editor_qt/symbols.py` (Weg A aus `docs/entwurf-python-abbau.md`):
//! ein zeilenweiser, stack-basierter Scanner, der Kommentare und
//! Zeichenketten ausblendet, aber bewusst KEIN Lexer ist -- ein Sprachserver
//! sieht halb getippten Text und muss trotzdem antworten (dieselbe
//! Ueberlegung wie bei `syntax.rs`). Spalten sind ZEICHEN, 1-basiert, so wie
//! sie der Editor und der LSP zaehlen.
//!
//! Benutzt von `lsp.rs` (Hover, Definition, Fundstellen, Gliederung) und von
//! `dhrt doku referenz` (Referenz aus dem Quelltext samt Kommentarblock).

use regex::Regex;
use std::sync::OnceLock;

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    pub name: String,
    /// "sub", "function", "class", "struct", "enum", "const", "dim",
    /// "property", "param".
    pub art: &'static str,
    /// 1-basiert.
    pub zeile: usize,
    /// 1-basiert, Anfang des Namens.
    pub spalte: usize,
    /// 1-basiert, exklusiv.
    pub spalte_ende: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fundstelle {
    pub name: String,
    pub zeile: usize,
    pub spalte: usize,
    pub spalte_ende: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bereich {
    /// "class", "struct", "sub", "function", "property".
    pub art: &'static str,
    pub name: String,
    /// 1-basiert, Zeile der Eroeffnung.
    pub zeile: usize,
    /// 1-basiert, Zeile des END (oder Dateiende bei offenem Block).
    pub ende: usize,
}

fn re(quelle: &'static str, zelle: &'static OnceLock<Regex>) -> &'static Regex {
    zelle.get_or_init(|| Regex::new(quelle).expect("Regex im Quelltext"))
}

fn ist_ident(c: char) -> bool { c.is_ascii_alphanumeric() || c == '_' }

/// Zeichenketten und Kommentare durch Leerzeichen gleicher Laenge ersetzen --
/// die Spalten bleiben, der Inhalt liefert keine Treffer mehr. Erkannt:
/// `"..."` mit `""` als Escape, f-Strings (die Ausdruecke in `{}` bleiben
/// sichtbar -- ein Rename muss sie treffen), `'`-Kommentar, `REM`.
pub fn ohne_kommentare_und_texte(zeile: &str) -> String {
    let z: Vec<char> = zeile.chars().collect();
    let n = z.len();
    let mut out: Vec<char> = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let ch = z[i];
        if ch == '\'' {
            out.extend(std::iter::repeat(' ').take(n - i));
            break;
        }
        if ch == '"' && i > 0 && (z[i - 1] == 'f' || z[i - 1] == 'F') {
            let letzte = out.len() - 1;
            out[letzte] = ' ';
            out.push(' ');
            i += 1;
            let mut tiefe = 0usize;
            while i < n {
                let c = z[i];
                if c == '"' && tiefe == 0 { out.push(' '); i += 1; break; }
                if c == '{' && i + 1 < n && z[i + 1] == '{' { out.push(' '); out.push(' '); i += 2; continue; }
                if c == '}' && i + 1 < n && z[i + 1] == '}' { out.push(' '); out.push(' '); i += 2; continue; }
                if c == '{' { tiefe += 1; out.push(' '); i += 1; continue; }
                if c == '}' && tiefe > 0 { tiefe -= 1; out.push(' '); i += 1; continue; }
                out.push(if tiefe > 0 { c } else { ' ' });
                i += 1;
            }
            continue;
        }
        if ch == '"' {
            out.push(' ');
            i += 1;
            while i < n {
                if z[i] == '"' {
                    out.push(' ');
                    i += 1;
                    if i < n && z[i] == '"' { out.push(' '); i += 1; continue; }
                    break;
                }
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if (ch == 'R' || ch == 'r') && i + 3 <= n
            && z[i..i + 3].iter().collect::<String>().eq_ignore_ascii_case("rem")
            && (i + 3 == n || !ist_ident(z[i + 3]))
        {
            let davor_ok = i == 0 || !ist_ident(z[i - 1]);
            if davor_ok {
                out.extend(std::iter::repeat(' ').take(n - i));
                break;
            }
        }
        out.push(ch);
        i += 1;
    }
    out.into_iter().collect()
}

/// Byte-Versatz in einer Zeile -> 1-basierte Zeichenspalte.
fn spalte(zeile: &str, byte: usize) -> usize { zeile[..byte].chars().count() + 1 }

struct Muster { re: &'static Regex, art: &'static str }

fn definitions_muster() -> Vec<Muster> {
    static S: [OnceLock<Regex>; 9] = [const { OnceLock::new() }; 9];
    const IDENT: &str = r"([A-Za-z_][A-Za-z0-9_]*)";
    let q: [(&'static str, &'static str); 9] = [
        (r"(?i)^\s*(?:PRIVATE\s+)?SUB\s+", "sub"),
        (r"(?i)^\s*(?:PRIVATE\s+)?FUNCTION\s+", "function"),
        (r"(?i)^\s*CLASS\s+", "class"),
        (r"(?i)^\s*STRUCT\s+", "struct"),
        (r"(?i)^\s*ENUM\s+", "enum"),
        (r"(?i)^\s*CONST\s+", "const"),
        (r"(?i)^\s*DIM\s+", "dim"),
        (r"(?i)^\s*STATIC\s+CONST\s+", "const"),
        // GET+SET desselben Namens: die erste gewinnt (find_definition-Regel).
        (r"(?i)^\s*PROPERTY\s+(?:GET|SET)\s+", "property"),
    ];
    q.iter().zip(S.iter()).map(|((kopf, art), zelle)| Muster {
        re: zelle.get_or_init(|| Regex::new(&format!("{}{}", kopf, IDENT)).expect("Regex")),
        art,
    }).collect()
}

/// Alle Definitionen in Quell-Reihenfolge -- auch `DIM` und `CONST`, damit
/// Springen und Umbenennen fuer Variablen gehen, und die Parameter von
/// SUB/FUNCTION/PROPERTY.
pub fn definitionen(quelle: &str) -> Vec<Definition> {
    static PARAM: OnceLock<Regex> = OnceLock::new();
    let param = re(r"(?i)^\s*(?:BYREF\s+|\.\.\.)?([A-Za-z_][A-Za-z0-9_]*)\$?", &PARAM);
    let muster = definitions_muster();
    let mut out = Vec::new();
    for (i, roh) in quelle.split('\n').enumerate() {
        let ln = i + 1;
        let sauber = ohne_kommentare_und_texte(roh);
        for m in &muster {
            if let Some(c) = m.re.captures(&sauber) {
                let g = c.get(1).unwrap();
                out.push(Definition {
                    name: g.as_str().to_string(), art: m.art, zeile: ln,
                    spalte: spalte(&sauber, g.start()), spalte_ende: spalte(&sauber, g.end()),
                });
                break;
            }
        }
        let kopf = sauber.trim_start().to_ascii_uppercase();
        if kopf.starts_with("SUB ") || kopf.starts_with("FUNCTION ") || kopf.starts_with("PROPERTY ")
            || kopf.starts_with("PRIVATE SUB ") || kopf.starts_with("PRIVATE FUNCTION ")
        {
            if let Some(auf) = sauber.find('(') {
                if let Some(zu_rel) = sauber[auf + 1..].find(')') {
                    let zone = &sauber[auf + 1..auf + 1 + zu_rel];
                    let mut versatz = auf + 1;
                    for seg in zone.split(',') {
                        if let Some(c) = param.captures(seg) {
                            let g = c.get(1).unwrap();
                            out.push(Definition {
                                name: g.as_str().to_string(), art: "param", zeile: ln,
                                spalte: spalte(&sauber, versatz + g.start()),
                                spalte_ende: spalte(&sauber, versatz + g.end()),
                            });
                        }
                        versatz += seg.len() + 1;
                    }
                }
            }
        }
    }
    out
}

/// Alle Vorkommen von `name` (ganzes Wort, Gross/Klein egal), Kommentare
/// und Zeichenketten ausgenommen. Ein `$` hinter dem Namen zaehlt nur, wenn
/// der gesuchte Name selbst darauf endet.
pub fn fundstellen(quelle: &str, name: &str) -> Vec<Fundstelle> {
    let Ok(muster) = Regex::new(&format!(r"(?i)\b{}\b(?:[^A-Za-z0-9_$]|$)", regex::escape(name))) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let laenge = name.chars().count();
    for (i, roh) in quelle.split('\n').enumerate() {
        let sauber = ohne_kommentare_und_texte(roh);
        for m in muster.find_iter(&sauber) {
            // Das Nachschau-Zeichen gehoert nicht zum Treffer.
            let anfang = spalte(&sauber, m.start());
            out.push(Fundstelle {
                name: sauber[m.start()..].chars().take(laenge).collect(),
                zeile: i + 1, spalte: anfang, spalte_ende: anfang + laenge,
            });
        }
    }
    out
}

/// Erste Definition zu `name` (Quell-Reihenfolge).
pub fn definition(quelle: &str, name: &str) -> Option<Definition> {
    let ziel = name.to_lowercase();
    definitionen(quelle).into_iter().find(|d| d.name.to_lowercase() == ziel)
}

/// CLASS/STRUCT/SUB/FUNCTION/PROPERTY-Bloecke mit Zeilenbereich. Ein
/// `END X` schliesst den naechsten passenden Block UND alles darueber --
/// fehlt ein inneres END (halb getippt), bliebe der Rest sonst bis zum
/// Dateiende offen.
pub fn bereiche(quelle: &str) -> Vec<Bereich> {
    static PROP: OnceLock<Regex> = OnceLock::new();
    static NAME: OnceLock<Regex> = OnceLock::new();
    let prop = re(r"(?i)^\s*PROPERTY\s+(?:GET|SET)\s+([A-Za-z_][A-Za-z0-9_]*)", &PROP);
    let name_re = re(r"^([A-Za-z_][A-Za-z0-9_]*)", &NAME);
    let zeilen: Vec<&str> = quelle.split('\n').collect();
    let n = zeilen.len();
    let mut stapel: Vec<usize> = Vec::new();   // Indizes in `out`
    let mut out: Vec<Bereich> = Vec::new();
    for (i, roh) in zeilen.iter().enumerate() {
        let ln = i + 1;
        let s = ohne_kommentare_und_texte(roh);
        let s = s.trim();
        if s.is_empty() { continue; }
        let gross = s.to_ascii_uppercase();
        if gross.starts_with("END ") {
            let zwei: Vec<&str> = gross.split_whitespace().take(2).collect();
            let ender = match zwei.join(" ").as_str() {
                "END CLASS" => Some("class"), "END STRUCT" => Some("struct"),
                "END SUB" => Some("sub"), "END FUNCTION" => Some("function"),
                "END PROPERTY" => Some("property"), _ => None,
            };
            if let Some(art) = ender {
                if let Some(pos) = stapel.iter().rposition(|&k| out[k].art == art) {
                    for &k in &stapel[pos..] { out[k].ende = ln; }
                    stapel.truncate(pos);
                }
            }
            continue;
        }
        if let Some(c) = prop.captures(s) {
            out.push(Bereich { art: "property", name: c[1].to_string(), zeile: ln, ende: n });
            stapel.push(out.len() - 1);
            continue;
        }
        let rest = gross.strip_prefix("PRIVATE ").map(|_| s[8..].trim_start()).unwrap_or(s);
        let rest_gross = rest.to_ascii_uppercase();
        for (kopf, art) in [("CLASS ", "class"), ("STRUCT ", "struct"), ("SUB ", "sub"), ("FUNCTION ", "function")] {
            if rest_gross.starts_with(kopf) {
                let nach = rest[kopf.len()..].trim_start();
                let name = name_re.captures(nach).map(|c| c[1].to_string()).unwrap_or_else(|| "?".into());
                out.push(Bereich { art, name, zeile: ln, ende: n });
                stapel.push(out.len() - 1);
                break;
            }
        }
    }
    out
}

/// `(signatur, doku)` zu einem im Text definierten Symbol: die
/// Deklarationszeile (bei SUB/FUNCTION bis zur schliessenden Klammer, auch
/// ueber Zeilen) und der Kommentarblock DIREKT darueber.
pub fn nutzer_doku(quelle: &str, name: &str) -> Option<(String, String)> {
    let d = definition(quelle, name)?;
    let zeilen: Vec<&str> = quelle.split('\n').collect();
    if d.zeile == 0 || d.zeile > zeilen.len() { return None; }
    let idx = d.zeile - 1;
    let mut sig = ohne_inline_kommentar(zeilen[idx].trim_end()).to_string();
    if matches!(d.art, "sub" | "function" | "property") && sig.contains('(') && !sig.contains(')') {
        let mut i = idx + 1;
        while i < zeilen.len() && !sig.contains(')') {
            sig = format!("{} {}", sig, ohne_inline_kommentar(zeilen[i].trim_end()).trim_start());
            i += 1;
        }
    }
    Some((sig.trim().to_string(), kommentar_darueber(&zeilen, idx)))
}

/// Kommentarzeilen rueckwaerts ab `idx - 1`, ohne `'`/`REM`, in Quell-Reihenfolge.
pub fn kommentar_darueber(zeilen: &[&str], idx: usize) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = idx;
    while i > 0 {
        i -= 1;
        let s = zeilen[i].trim();
        if s.is_empty() { break; }
        match kommentar_text(s) {
            Some(t) => out.push(t.to_string()),
            None => break,
        }
    }
    out.reverse();
    out.join("\n").trim().to_string()
}

/// Der Text hinter `'` bzw. `REM ` -- oder None, wenn die Zeile kein Kommentar ist.
pub fn kommentar_text(zeile: &str) -> Option<&str> {
    if let Some(r) = zeile.strip_prefix('\'') { return Some(r.trim()); }
    if zeile.len() >= 3 && zeile[..3].eq_ignore_ascii_case("rem") {
        let rest = &zeile[3..];
        if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') { return Some(rest.trim()); }
    }
    None
}

/// Nachgestellten `'`-Kommentar abschneiden, Zeichenketten uebersprungen.
pub fn ohne_inline_kommentar(zeile: &str) -> &str {
    let mut in_text = false;
    let mut iter = zeile.char_indices().peekable();
    while let Some((i, ch)) = iter.next() {
        if in_text {
            if ch == '"' {
                if matches!(iter.peek(), Some((_, '"'))) { iter.next(); continue; }
                in_text = false;
            }
            continue;
        }
        if ch == '"' { in_text = true; continue; }
        if ch == '\'' { return zeile[..i].trim_end(); }
    }
    zeile
}

/// Das Wort um (zeile0, zeichen0): `(wort ohne $, anfang, ende)` als
/// 0-basierte Zeichenspalten. Steht die Marke direkt hinter dem `$` einer
/// Textvariablen, zaehlt das Wort davor.
pub fn wort_bei(text: &str, zeile0: usize, zeichen0: usize) -> (String, usize, usize) {
    let z: Vec<char> = text.split('\n').nth(zeile0).unwrap_or("").chars().collect();
    let n = z.len();
    let c0 = zeichen0.min(n);
    let hat_dollar = c0 > 0 && z[c0 - 1] == '$';
    let von = if hat_dollar { c0 - 1 } else { c0 };
    let mut a = von;
    while a > 0 && ist_ident(z[a - 1]) { a -= 1; }
    let mut b = von;
    while b < n && ist_ident(z[b]) { b += 1; }
    let mut ende = b;
    if b < n && z[b] == '$' { ende = b + 1; }
    let wort: String = z[a..ende].iter().collect();
    (wort.trim_end_matches('$').to_string(), a, ende)
}

/// Wort-Praefix LINKS von der Marke (fuer die Vervollstaendigung).
pub fn praefix_bei(text: &str, zeile0: usize, zeichen0: usize) -> (String, usize) {
    let z: Vec<char> = text.split('\n').nth(zeile0).unwrap_or("").chars().collect();
    let a = zeichen0.min(z.len());
    let mut anfang = a;
    while anfang > 0 && ist_ident(z[anfang - 1]) { anfang -= 1; }
    (z[anfang..a].iter().collect(), anfang)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "' Spieler-Klasse\nCLASS Player\n    DIM hp AS INTEGER\n    SUB Init()\n        Self.hp = 100\n    END SUB\nEND CLASS\nFUNCTION add(a AS INTEGER, b AS INTEGER) AS INTEGER\n    RETURN a + b\nEND FUNCTION\nDIM result AS INTEGER\nresult = add(1, 2)\n";

    #[test]
    fn wort_bei_findet_und_laesst_leer() {
        assert_eq!(wort_bei(SRC, 11, 10).0, "add");
        assert_eq!(wort_bei(SRC, 11, 7).0, "");
        // Marke direkt hinter dem `$`.
        assert_eq!(wort_bei("x$ = \"hi\"", 0, 2), ("x".into(), 0, 2));
        assert_eq!(wort_bei("x$ = \"hi\"", 0, 1).0, "x");
    }

    #[test]
    fn definitionen_mit_spalten() {
        let d = definitionen(SRC);
        let add = d.iter().find(|x| x.name == "add").unwrap();
        assert_eq!((add.art, add.zeile, add.spalte, add.spalte_ende), ("function", 8, 10, 13));
        let namen: Vec<&str> = d.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(namen, ["Player", "hp", "Init", "add", "a", "b", "result"]);
        assert_eq!(d.iter().find(|x| x.name == "b").unwrap().art, "param");
        // Umlaute davor verschieben die Spalte in Zeichen, nicht in Bytes.
        let d2 = definitionen("DIM x AS STRING : x = \"äöü\" : DIM y AS INTEGER\nSUB f(ü AS INTEGER)\nEND SUB\n");
        assert_eq!(d2[0].name, "x");
        assert_eq!(definition(SRC, "ADD").map(|d| d.zeile), Some(8));
        assert!(definition("PRINT 1\n", "x").is_none());
    }

    #[test]
    fn fundstellen_ganze_woerter_ohne_texte() {
        let f = fundstellen(SRC, "add");
        let zeilen: Vec<usize> = f.iter().map(|x| x.zeile).collect();
        assert_eq!(zeilen, [8, 12]);
        assert_eq!((f[1].spalte, f[1].spalte_ende), (10, 13));
        assert!(fundstellen("xy = 1\nPRINT \"x\" ' x\n", "x").is_empty());
        assert_eq!(fundstellen("PRINT f\"{x} ok\"\n", "x").len(), 1);
        assert!(fundstellen("PRINT f\"x ok\"\n", "x").is_empty());
    }

    #[test]
    fn bereiche_verschachtelt_und_offen() {
        let b = bereiche(SRC);
        let player = b.iter().find(|x| x.name == "Player").unwrap();
        assert_eq!((player.art, player.zeile, player.ende), ("class", 2, 7));
        let init = b.iter().find(|x| x.name == "Init").unwrap();
        assert_eq!((init.zeile, init.ende), (4, 6));
        // Fehlendes inneres END: END CLASS schliesst auch die SUB.
        let b2 = bereiche("CLASS A\nSUB Foo()\nEND CLASS\nPRINT 1\n");
        assert_eq!(b2.iter().map(|x| x.ende).collect::<Vec<_>>(), [3, 3]);
        let b3 = bereiche("PROPERTY GET hp() AS INTEGER\nEND PROPERTY\nPRIVATE SUB x()\nEND SUB\n");
        assert_eq!(b3[0].art, "property");
        assert_eq!(b3[1].name, "x");
    }

    #[test]
    fn nutzer_doku_mit_kommentarblock() {
        let (sig, doc) = nutzer_doku(SRC, "Player").unwrap();
        assert_eq!(sig, "CLASS Player");
        assert_eq!(doc, "Spieler-Klasse");
        let src = "' Addiert.\n' Zweite Zeile.\nFUNCTION add(a AS INTEGER,\n    b AS INTEGER) AS INTEGER ' hinten\nEND FUNCTION\n";
        let (sig, doc) = nutzer_doku(src, "add").unwrap();
        assert_eq!(sig, "FUNCTION add(a AS INTEGER, b AS INTEGER) AS INTEGER");
        assert_eq!(doc, "Addiert.\nZweite Zeile.");
        assert!(nutzer_doku(SRC, "nix").is_none());
    }

    #[test]
    fn kommentare_und_texte_werden_ausgeblendet() {
        // 13 Zeichen: Text (5) und Kommentar (3) samt Leerzeichen werden zu 10 Leerzeichen.
        assert_eq!(ohne_kommentare_und_texte("a = \"x'y\" ' k"), format!("a ={}", " ".repeat(10)));
        assert_eq!(ohne_kommentare_und_texte("REM alles"), "         ");
        assert_eq!(ohne_kommentare_und_texte("remix = 1"), "remix = 1");
        assert_eq!(ohne_inline_kommentar("PRINT \"a'b\" ' k"), "PRINT \"a'b\"");
        assert_eq!(praefix_bei("PRI", 0, 3), ("PRI".into(), 0));
    }
}
