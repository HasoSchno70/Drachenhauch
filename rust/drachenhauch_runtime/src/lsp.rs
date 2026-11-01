//! `dhrt lsp` -- der Sprachserver (LSP ueber stdio, JSON-RPC 2.0).
//!
//! Weg A aus `docs/entwurf-python-abbau.md`: bis hierher rechnete
//! `drachenhauch/lsp/` in Python nach, was dhrt beim Uebersetzen laengst
//! weiss. Jetzt liegen Diagnose (dieselbe Kette wie `--check`), Symbole
//! (`symbole.rs`), Index und Hover-Texte (eingebettet, wie der Index im
//! Compiler) in EINEM Prozess -- VS Code braucht kein Python mehr.
//!
//! Methoden: initialize/initialized/shutdown/exit, textDocument/didOpen,
//! didChange, didClose, completion, hover, definition, references,
//! documentSymbol, codeAction (Schnellkorrekturen). Voll-Sync. Alles Unbekannte mit `id` bekommt `null`.
//!
//! **Diagnose laeuft im Hintergrund.** Jeder Tastendruck schickt ein
//! volles didChange; die Pruefung einer 2 800-Zeilen-Datei kostet rund 90 ms.
//! Liefe sie in der Leseschleife, staenden Hover und Vervollstaendigung so
//! lange an. Darum je Dokument ein Faden mit Generationszaehler: er wartet
//! kurz (Tippen buendelt sich), prueft, ob er noch der neueste ist, und
//! schickt erst dann. Der Schreibzugriff auf stdout liegt hinter einem Mutex
//! -- zwei verschraenkte Nachrichten wuerden die Rahmung brechen.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};

use crate::symbole;

const NAME: &str = "drachenhauch-lsp";

// LSP CompletionItemKind / SymbolKind (Teilmenge).
const CK_FUNCTION: i64 = 3;
const CK_VARIABLE: i64 = 6;
const CK_METHOD: i64 = 2;
const CK_FIELD: i64 = 5;
const CK_PROPERTY: i64 = 10;
const CK_CLASS: i64 = 7;
const CK_ENUM: i64 = 13;
const CK_KEYWORD: i64 = 14;
const CK_CONSTANT: i64 = 21;
const SK_CLASS: i64 = 5;
const SK_PROPERTY: i64 = 7;
const SK_ENUM: i64 = 10;
const SK_FUNCTION: i64 = 12;
const SK_STRUCT: i64 = 23;

// ---------------------------------------------------------------- Rahmung

/// Eine Content-Length-gerahmte Nachricht lesen. `Ok(None)` NUR bei echtem
/// Dateiende; ein fehlender oder kaputter Kopf ist ein Fehler, der genau
/// diese eine Nachricht kostet, nicht die Sitzung.
pub fn nachricht_lesen<R: BufRead>(r: &mut R) -> Result<Option<Value>, String> {
    let mut laenge: Option<usize> = None;
    let mut zeile = String::new();
    loop {
        zeile.clear();
        let n = r.read_line(&mut zeile).map_err(|e| e.to_string())?;
        if n == 0 { return Ok(None); }
        let z = zeile.trim();
        if z.is_empty() { break; }
        if let Some((k, v)) = z.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                laenge = Some(v.trim().parse::<usize>()
                    .map_err(|_| format!("LSP-Nachricht mit ungueltigem Content-Length: {:?}", v.trim()))?);
            }
        }
    }
    let laenge = laenge.ok_or("LSP-Nachricht ohne Content-Length-Header")?;
    if laenge == 0 { return Err("LSP-Nachricht mit Content-Length 0".into()); }
    let mut body = vec![0u8; laenge];
    r.read_exact(&mut body).map_err(|e| e.to_string())?;
    serde_json::from_slice(&body).map(Some).map_err(|e| e.to_string())
}

pub fn nachricht_schreiben<W: Write>(w: &mut W, msg: &Value) -> io::Result<()> {
    let daten = serde_json::to_vec(msg)?;
    write!(w, "Content-Length: {}\r\n\r\n", daten.len())?;
    w.write_all(&daten)?;
    w.flush()
}

type Sender = Arc<Mutex<Box<dyn Write + Send>>>;

fn senden(s: &Sender, msg: Value) {
    if let Ok(mut w) = s.lock() { let _ = nachricht_schreiben(&mut *w, &msg); }
}

// ---------------------------------------------------------------- Diagnose

/// Die `--check`-Kette auf dem Puffertext, zurueckgerechnet auf die Zeilen
/// des Puffers: dhrt preprocesst IMPORTs hinein und meldet gemergte Zeilen;
/// ohne die Ruecknahme rutschten alle Marker um die Laenge des Inlinierten.
/// Ein Fehler in einer importierten Datei landet in Zeile 1 mit Herkunft.
pub fn diagnose(text: &str, basis: &Path) -> Vec<Value> {
    let kx = KorrKontext::neu(text);
    let mut aus = diagnose_uebersetzer(text, basis, &kx);
    let zeilen: &[&str] = &kx.zeilen;
    for (z0, name) in unbenutzte(text) {
        let meldung = format!("'{}' wird angelegt, aber nie benutzt.", name);
        let (von, bis) = fehler_bereich(zeilen.get(z0).copied().unwrap_or(""), 0, &meldung);
        let mut d = json!({
            "range": {"start": {"line": z0, "character": von}, "end": {"line": z0, "character": bis}},
            "severity": 4, "source": "drachenhauch", "message": crate::meldung::t(&meldung), "tags": [1],
        });
        let k = korrekturen_mit(&kx, z0, von, bis, &meldung);
        if !k.is_empty() {
            d["data"] = json!({"korrekturen": k.into_iter().map(|(titel, aend)| json!({
                "titel": crate::meldung::t(&titel),
                "aenderungen": aend.into_iter().map(|(z, a, b, t)| {
                    if b == usize::MAX { json!({"zeile": z, "von": a, "bis_zeile": z + 1, "bis": 0, "text": t}) }
                    else { json!({"zeile": z, "von": a, "bis_zeile": z, "bis": b, "text": t}) }
                }).collect::<Vec<_>>(),
            })).collect::<Vec<_>>()});
        }
        aus.push(d);
    }
    aus
}

/// Typ-Hinweise fuer den Editor: je Stelle ohne geschriebenen Typ (heute
/// die Variablen von FOR EACH) Zeile (ab 1, im Puffer), Name und Typ.
/// Stellen aus importierten Dateien fallen weg; was der Compiler nicht weiss,
/// steht nicht darin.
pub fn typ_hinweise(text: &str, basis: &Path) -> Vec<(u32, String, String)> {
    // Hinweise gibt es nur fuer FOR EACH, und die stehen im Puffer selbst
    // (Importe fallen unten weg). Ohne das Wort lohnt kein Uebersetzen --
    // bei 100000 Zeilen kostete es 0,5 s je Pruefung (Stresstest 2026-10-02).
    if !text.as_bytes().windows(4).any(|w| w.eq_ignore_ascii_case(b"each")) { return Vec::new(); }
    let roh = crate::compiler::typ_hinweise_sammeln(|| { let _ = crate::check_source(text, basis, "<editor>"); });
    let herkunft = crate::preprocess::process(text, basis).ok().map(|r| r.2);
    let mut aus: Vec<(u32, String, String)> = Vec::new();
    let mut gesehen: std::collections::HashSet<(u32, String)> = std::collections::HashSet::new();
    for (z, name, typ) in roh {
        let zeile = match herkunft.as_ref().and_then(|h| h.get((z as usize).saturating_sub(1))) {
            Some(h) if h.datei.is_empty() => h.zeile,
            Some(_) => continue,
            None => z,
        };
        // Ueber eine Menge, nicht `aus.iter().any`: das waere quadratisch in
        // der Zahl der Schleifen.
        if gesehen.insert((zeile, name.to_ascii_lowercase())) {
            aus.push((zeile, name, typ));
        }
    }
    aus.sort_by_key(|e| e.0);
    aus
}

fn diagnose_uebersetzer(text: &str, basis: &Path, kx: &KorrKontext) -> Vec<Value> {
    let roh = crate::check_source(text, basis, "<editor>");
    let herkunft = crate::preprocess::process(text, basis).ok().map(|r| r.2);
    let zeilen: Vec<&str> = text.split('\n').collect();
    roh.into_iter().map(|d| {
        let phase = d.get("phase").and_then(|p| p.as_str()).unwrap_or("compile").to_string();
        let mut zeile = d.get("line").and_then(|l| l.as_u64()).unwrap_or(0).max(1) as usize;
        let mut meldung = d.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
        // Was der Mensch liest -- uebersetzt VOR dem Vorsatz "in datei:zeile ->",
        // der zu keiner Vorlage gehoert. Gerechnet (Bereich, Korrekturen) wird
        // weiter mit dem deutschen Satz.
        let mut anzeige = crate::meldung::t(&meldung);
        if matches!(phase.as_str(), "lex" | "parse" | "compile" | "namensraum") {
            if let Some(h) = herkunft.as_ref().and_then(|h| h.get(zeile - 1)) {
                if h.datei.is_empty() { zeile = h.zeile as usize; }
                else {
                    meldung = format!("in {}:{} -> {}", h.datei, h.zeile, meldung);
                    anzeige = format!("in {}:{} -> {}", h.datei, h.zeile, anzeige);
                    zeile = 1;
                }
            }
        }
        let z0 = zeile.saturating_sub(1);
        let spalte = d.get("col").and_then(|c| c.as_u64()).unwrap_or(0) as usize;
        let text_zeile = zeilen.get(z0).copied().unwrap_or("");
        // Ein Fehler in einer importierten Datei steht an der IMPORT-Zeile --
        // seine Spalte gehoert zur anderen Datei, also die ganze Zeile.
        let importiert = meldung.starts_with("in ") && meldung.contains(" -> ");
        let (von, bis) = if importiert {
            (0, text_zeile.chars().count().max(1))
        } else {
            fehler_bereich(text_zeile, spalte, &meldung)
        };
        let schwere = if d.get("severity").and_then(|s| s.as_str()) == Some("warning") { 2 } else { 1 };
        let mut aus = json!({
            "range": {"start": {"line": z0, "character": von}, "end": {"line": z0, "character": bis}},
            "severity": schwere, "source": "drachenhauch", "message": anzeige,
        });
        let k = if importiert { Vec::new() } else { korrekturen_mit(kx, z0, von, bis, &meldung) };
        if !k.is_empty() {
            let liste: Vec<Value> = k.into_iter().map(|(titel, aend)| json!({
                "titel": crate::meldung::t(&titel),
                "aenderungen": aend.into_iter().map(|(z, a, b, t)| {
                    // bis == usize::MAX: die ganze Zeile samt Umbruch.
                    if b == usize::MAX { json!({"zeile": z, "von": a, "bis_zeile": z + 1, "bis": 0, "text": t}) }
                    else { json!({"zeile": z, "von": a, "bis_zeile": z, "bis": b, "text": t}) }
                }).collect::<Vec<_>>(),
            })).collect();
            aus["data"] = json!({"korrekturen": liste});
        }
        aus
    }).collect()
}

/// Wo in der Zeile eine Meldung hingehoert, als Zeichen (von, bis).
///
/// Der Uebersetzer kennt bei Syntaxfehlern eine Spalte (ab 1), bei den
/// meisten anderen nur die Zeile. Nennt die Meldung einen Namen in
/// Hochkommas (`'zaehlr' wird hier beschrieben ...`), gilt das erste
/// Vorkommen dieses Namens als ganzes Wort; sonst das Wort an der Spalte;
/// sonst die Zeile ohne ihre Einrueckung. Eine Wellenlinie unter der ganzen
/// Zeile sagt kaum mehr als die Marke am Rand -- darum so eng wie moeglich.
pub fn fehler_bereich(zeile: &str, spalte: usize, meldung: &str) -> (usize, usize) {
    let z: Vec<char> = zeile.chars().collect();
    let wortzeichen = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
    // Ein genannter Name, als ganzes Wort (Gross/klein egal).
    let mut rest = meldung;
    while let Some(a) = rest.find('\'') {
        let nach = &rest[a + 1..];
        let Some(b) = nach.find('\'') else { break };
        let name: Vec<char> = nach[..b].to_lowercase().chars().collect();
        if !name.is_empty() && name.iter().all(|&c| wortzeichen(c)) {
            let klein: Vec<char> = zeile.to_lowercase().chars().collect();
            if klein.len() == z.len() && klein.len() >= name.len() {
                for i in 0..=(klein.len() - name.len()) {
                    if klein[i..i + name.len()] == name[..]
                        && (i == 0 || !wortzeichen(klein[i - 1]))
                        && (i + name.len() == klein.len() || !wortzeichen(klein[i + name.len()])) {
                        return (i, i + name.len());
                    }
                }
            }
        }
        rest = &nach[b + 1..];
    }
    if spalte > 0 {
        let s = (spalte - 1).min(z.len());
        // Das Wort um die Spalte; steht sie am Ende oder auf einem Zeichen,
        // das kein Wort ist, ein einzelnes Zeichen (am Ende: das letzte).
        let (mut a, mut b) = (s, s);
        while a > 0 && z.get(a - 1).map(|&c| wortzeichen(c)).unwrap_or(false) { a -= 1; }
        while b < z.len() && wortzeichen(z[b]) { b += 1; }
        if b > a { return (a, b); }
        if s < z.len() { return (s, s + 1); }
        let letzt = z.iter().rposition(|c| !c.is_whitespace()).map(|p| p + 1).unwrap_or(0);
        return (letzt.saturating_sub(1), letzt.max(1));
    }
    let anfang = z.iter().position(|c| !c.is_whitespace()).unwrap_or(0);
    let ende = z.iter().rposition(|c| !c.is_whitespace()).map(|p| p + 1).unwrap_or(anfang + 1);
    (anfang, ende.max(anfang + 1))
}

// ---------------------------------------------------------- Schnellkorrektur

/// Eine Aenderung am Text: Zeile (ab 0), Zeichen von..bis (ab 0), neuer Text.
pub type Aenderung = (usize, usize, usize, String);

/// Die Woerter, die einen Block schliessen, und woran man seinen Kopf
/// erkennt (Zeile ohne Kommentar, gross geschrieben, ohne Einrueckung).
fn blockkopf(schluss: &str, kopf: &str) -> bool {
    let erstes = kopf.split_whitespace().next().unwrap_or("");
    match schluss {
        "END IF" => erstes == "IF" && kopf.ends_with("THEN"),
        "NEXT" => erstes == "FOR",
        "WEND" | "END WHILE" => erstes == "WHILE",
        "LOOP" => erstes == "DO",
        "END SELECT" => kopf.starts_with("SELECT CASE"),
        "END SUB" => erstes == "SUB",
        "END FUNCTION" => erstes == "FUNCTION",
        "END PROPERTY" => erstes == "PROPERTY",
        "END CLASS" => erstes == "CLASS",
        "END STRUCT" => erstes == "STRUCT",
        "END ENUM" => erstes == "ENUM" && !kopf.contains('='),
        "END TRY" => erstes == "TRY",
        "END WITH" => erstes == "WITH",
        _ => false,
    }
}

/// Wo ein fehlendes Blockende hingehoert: der letzte Kopf dieser Art, der
/// noch nicht auf gleicher Einrueckung geschlossen ist; eingefuegt wird
/// hinter der letzten Zeile seines (tiefer eingerueckten) Rumpfs, mit der
/// Einrueckung des Kopfes. (Zeile, Einrueckung) oder None.
fn blockende_stelle(zeilen: &[&str], schluss: &str) -> Option<(usize, String)> {
    let einzug = |t: &str| t.chars().take_while(|c| c.is_whitespace()).count();
    let rein = |t: &str| crate::symbole::ohne_inline_kommentar(t).trim().to_uppercase();
    for k in (0..zeilen.len()).rev() {
        let kopf = rein(zeilen[k]);
        if !blockkopf(schluss, &kopf) { continue; }
        let e = einzug(zeilen[k]);
        let geschlossen = zeilen[k + 1..].iter().any(|t| {
            let r = rein(t);
            einzug(t) == e && (r == schluss || r.starts_with(&format!("{} ", schluss)))
        });
        if geschlossen { continue; }
        let mut ende = k;
        for (j, t) in zeilen.iter().enumerate().skip(k + 1) {
            if t.trim().is_empty() { continue; }
            if einzug(t) > e { ende = j; } else { break; }
        }
        return Some((ende, zeilen[k].chars().take(e).collect()));
    }
    None
}

/// Hinweise, die der Uebersetzer nicht gibt: eine Variable, die in einem
/// Unterprogramm angelegt und dort nirgends benutzt wird. Nur dort -- eine
/// globale kann eine andere Datei benutzen, die diese importiert. Als
/// (Zeile ab 0, Name).
pub fn unbenutzte(text: &str) -> Vec<(usize, String)> {
    let zeilen: Vec<&str> = text.split('\n').collect();
    let mut aus = Vec::new();
    for b in crate::symbole::bereiche(text) {
        if !matches!(b.art, "sub" | "function" | "property") { continue; }
        let (von, bis) = (b.zeile, b.ende.min(zeilen.len()));    // Rumpf: Zeilen von..bis-1 (ab 0), ohne Kopf
        if von >= bis { continue; }
        let rumpf = zeilen[von..bis].join("\n");
        for (i, t) in zeilen[von..bis].iter().enumerate() {
            let Some(name) = einfaches_dim(t) else { continue };
            // Nur ein Vorkommen im Rumpf (Kommentare und Texte zaehlen nicht).
            if crate::symbole::fundstellen(&rumpf, &name).len() == 1 {
                aus.push((von + i, name));
            }
        }
    }
    aus
}

/// `DIM name AS T` (auch `= wert`) mit genau einem Namen -> der Name.
fn einfaches_dim(zeile: &str) -> Option<String> {
    let t = crate::symbole::ohne_inline_kommentar(zeile).trim().to_string();
    let gross = t.to_uppercase();
    if !gross.starts_with("DIM ") || t.contains(',') { return None; }
    let rest = t[4..].trim_start();
    let name: String = rest.chars().take_while(|&c| c.is_alphanumeric() || c == '_' || c == '$').collect();
    if name.is_empty() || !rest[name.len()..].trim_start().to_uppercase().starts_with("AS ") { return None; }
    Some(name)
}

/// Vorschlaege zu einer Meldung: (Titel, Aenderungen). Gelesen wird die
/// MELDUNG, nicht der Uebersetzer -- sie sagt schon, was gemeint war
/// ("Meintest du ...", "heisst in Drachenhauch ...", "nirgends ... angelegt").
/// So bleibt es EINE Quelle: was die Meldung vorschlaegt, ist auch das, was
/// die Korrektur tut. Nur Korrekturen, die sicher genau diese Stelle treffen;
/// lieber keine als eine, die etwas anderes aendert, als ihr Titel sagt.
/// Was `korrekturen` ueber den ganzen Text braucht -- EINMAL je Pruefung
/// gerechnet. Je Meldung neu zerlegt (Zeilen, Bereiche, Fundstellen) war das
/// Pruefen quadratisch in der Zahl der Meldungen: eine Datei mit 5000
/// Warnungen kostete je CODE_CHECK$ 6,6 s, und die IDE prueft nach jeder
/// Tipp-Pause (Stresstest 2026-10-02).
pub struct KorrKontext<'a> {
    text: &'a str,
    zeilen: Vec<&'a str>,
    bereiche: std::cell::OnceCell<Vec<crate::symbole::Bereich>>,
    kopf: std::cell::OnceCell<usize>,
    namen: std::cell::RefCell<std::collections::HashMap<String, String>>,
}

impl<'a> KorrKontext<'a> {
    pub fn neu(text: &'a str) -> Self {
        KorrKontext {
            text, zeilen: text.split('\n').collect(),
            bereiche: std::cell::OnceCell::new(), kopf: std::cell::OnceCell::new(),
            namen: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }
    fn bereiche(&self) -> &[crate::symbole::Bereich] { self.bereiche.get_or_init(|| crate::symbole::bereiche(self.text)) }
    fn kopf(&self) -> usize { *self.kopf.get_or_init(|| kopf_ende(&self.zeilen)) }
    /// Der Name so, wie er im Programm steht ("lSumme" statt "lsumme").
    fn schreibweise(&self, v: &str) -> String {
        if let Some(n) = self.namen.borrow().get(v) { return n.clone(); }
        let n = crate::symbole::fundstellen(self.text, v).into_iter().next()
            .map(|f| f.name).unwrap_or_else(|| v.to_string());
        self.namen.borrow_mut().insert(v.to_string(), n.clone());
        n
    }
}

pub fn korrekturen(text: &str, z0: usize, von: usize, bis: usize, meldung: &str) -> Vec<(String, Vec<Aenderung>)> {
    korrekturen_mit(&KorrKontext::neu(text), z0, von, bis, meldung)
}

pub fn korrekturen_mit(kx: &KorrKontext, z0: usize, von: usize, bis: usize, meldung: &str) -> Vec<(String, Vec<Aenderung>)> {
    let zeilen: &[&str] = &kx.zeilen;
    // Die Meldung am Programmende steht oft HINTER der letzten Zeile.
    let zeile = zeilen.get(z0).copied().unwrap_or("");
    let z: Vec<char> = zeile.chars().collect();
    let wort: String = z.get(von..bis.min(z.len())).map(|s| s.iter().collect()).unwrap_or_default();
    let wortzeichen = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
    let ist_wort = !wort.is_empty() && wort.chars().all(wortzeichen);
    let mut aus: Vec<(String, Vec<Aenderung>)> = Vec::new();

    // "Meintest du CIRCLE, CIRCLES?" / "Meintest du 'zaehler'?"
    if let Some(p) = meldung.find("Meintest du ") {
        if ist_wort {
            let rest = &meldung[p + "Meintest du ".len()..];
            let rest = rest.split('?').next().unwrap_or("");
            for v in rest.split(',') {
                let v = v.trim().trim_matches('\'');
                if v.is_empty() || !v.chars().all(wortzeichen) || v.eq_ignore_ascii_case(&wort) { continue; }
                // Der Compiler kennt Variablen nur klein ("lsumme"); geschrieben
                // wird der Name so, wie er im Programm steht ("lSumme").
                let v = kx.schreibweise(v);
                aus.push((format!("Ersetzen durch {}", v), vec![(z0, von, bis, v)]));
            }
        }
    }

    // "heisst in Drachenhauch UPPER$(text)" -- nur die Form mit genau
    // einem Namen vorne; "... bzw. ..." und Erklaerungen bleiben Text.
    if let Some(p) = meldung.find("heisst in Drachenhauch ") {
        let rest = &meldung[p + "heisst in Drachenhauch ".len()..];
        let neu: String = rest.chars().take_while(|&c| wortzeichen(c)).collect();
        let ein_name = rest[neu.len()..].starts_with('(') && !rest.contains(" bzw. ");
        if ist_wort && ein_name && !neu.is_empty() {
            aus.push((format!("Ersetzen durch {}", neu), vec![(z0, von, bis, neu)]));
        }
    }

    // "Ungleich schreibt man in Drachenhauch <>" -- `!=` an der Stelle.
    if meldung.starts_with("Ungleich schreibt man") {
        let s: String = z.get(von..(von + 2).min(z.len())).map(|s| s.iter().collect()).unwrap_or_default();
        if s == "!=" { aus.push(("Durch <> ersetzen".into(), vec![(z0, von, von + 2, "<>".into())])); }
    }

    // "Befehle und SUBs bekommen ihre Werte in Klammern: PRNT(...)" -- den
    // Rest der Zeile (ohne Kommentar) einklammern.
    if meldung.starts_with("Befehle und SUBs bekommen ihre Werte in Klammern") && ist_wort {
        // Das Ende des Codes: das Kommentarzeichen ausserhalb einer
        // Zeichenkette, sonst das Zeilenende.
        let mut ende = z.len();
        let mut in_text = false;
        for (k, &c) in z.iter().enumerate() {
            if c == '"' { in_text = !in_text; }
            if !in_text && c == '\'' { ende = k; break; }
        }
        while ende > bis && z[ende - 1].is_whitespace() { ende -= 1; }
        let mut a = bis;
        while a < ende && z[a].is_whitespace() { a += 1; }
        if a < ende && z[a] != '(' {
            let inhalt: String = z[a..ende].iter().collect();
            aus.push(("Klammern setzen".into(), vec![(z0, bis, ende, format!("({})", inhalt))]));
        }
    }

    // "END IF erwartet, Programmende erreicht" -- das Ende hinter den Rumpf.
    // Drei Formen: am Programmende, mitten in einem anderen Block ("END IF
    // fehlt -- hier steht END SUB, ..." bzw. "Erwartet END IF") und die
    // WHILE-Schleife mit eigenem Satz.
    let end_wort = |w: &str| -> Option<String> {
        let w = w.trim();
        if !w.is_empty() && w.chars().all(|c| c.is_ascii_uppercase()) { Some(format!("END {}", w)) } else { None }
    };
    let schluss: Option<String> = if let Some(p) = meldung.find(" erwartet, Programmende erreicht") {
        Some(meldung[..p].trim().to_string())
    } else if let Some(rest) = meldung.strip_prefix("END ").and_then(|r| r.split_once(" fehlt -- hier steht END ")) {
        end_wort(rest.0)
    } else if let Some(rest) = meldung.strip_prefix("Erwartet END ") {
        end_wort(rest)
    } else if meldung.starts_with("WEND (oder END WHILE) erwartet") {
        Some("WEND".to_string())
    } else { None };
    if let Some(schluss) = schluss.as_deref() {
        if let Some((ende, ein)) = blockende_stelle(zeilen, schluss) {
            let laenge = zeilen[ende].chars().count();
            aus.push((format!("{} ergaenzen", schluss), vec![(ende, laenge, laenge, format!("\n{}{}", ein, schluss))]));
        }
    }

    // "... IMPORT \"json\" fehlt" / "fehlt IMPORT \"vec2\"?" -- oben einfuegen,
    // hinter die IMPORTs, die schon da sind, sonst hinter den Kopfkommentar.
    if let Some(p) = meldung.find("IMPORT \"") {
        let rest = &meldung[p + 8..];
        if let Some(q) = rest.find('"') {
            let modul = &rest[..q];
            if !modul.is_empty() && modul.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                let ziel = kx.kopf();
                aus.push((format!("IMPORT \"{}\" einfuegen", modul), vec![(ziel, 0, 0, format!("IMPORT \"{}\"\n", modul))]));
            }
        }
    }

    // "'x' wird angelegt, aber nie benutzt" -- die Zeile weg, wenn dabei
    // nichts verloren geht, was etwas tut (ein Aufruf in `= wert`).
    if meldung.contains("wird angelegt, aber nie benutzt") && !zeile.contains('(') {
        aus.push(("Zeile entfernen".into(), vec![(z0, 0, usize::MAX, String::new())]));
    }

    // "'x' wird hier beschrieben/gelesen, aber nirgends ... angelegt" --
    // ein DIM davor. Im Unterprogramm gleich unter seinem Kopf, sonst vor dem
    // Block auf oberster Ebene: ein DIM in einer Schleife setzte die Variable
    // in jeder Runde zurueck.
    if meldung.contains("aber nirgends im Programm mit DIM oder CONST angelegt") && ist_wort {
        let rhs = zuweisung_rechts(zeile, &wort);
        let typ = typ_raten(&wort, rhs.as_deref());
        let einzug = |s: &str| s.chars().take_while(|c| c.is_whitespace()).collect::<String>();
        let bereiche = kx.bereiche();
        let ln = z0 + 1;
        let umgebend = bereiche.iter()
            .filter(|b| matches!(b.art, "sub" | "function" | "property") && b.zeile < ln && ln <= b.ende)
            .max_by_key(|b| b.zeile);
        let (ziel, ein) = if let Some(b) = umgebend {
            // Unter den Kopf, eingerueckt wie die erste Zeile des Rumpfs.
            let kopf = b.zeile - 1;
            let rumpf = zeilen.get(kopf + 1..z0 + 1).unwrap_or(&[]).iter()
                .find(|s| !s.trim().is_empty()).map(|s| einzug(s)).unwrap_or_default();
            let ein = if rumpf.is_empty() { format!("{}    ", einzug(zeilen[kopf])) } else { rumpf };
            (kopf + 1, ein)
        } else {
            let mut k = z0;
            while k > 0 && (zeilen[k].trim().is_empty() || zeilen[k].starts_with(|c: char| c.is_whitespace())) { k -= 1; }
            (k, String::new())
        };
        let zeile_neu = format!("{}DIM {} AS {}\n", ein, wort, typ);
        aus.push((format!("DIM {} AS {} anlegen", wort, typ), vec![(ziel, 0, 0, zeile_neu)]));
        // Im Unterprogramm ist das DIM dort lokal -- der Wert ist nach dem
        // Aufruf weg. Wer ihn behalten will (ein Zustand wie "welcher
        // Eintrag wird bearbeitet"), braucht ihn global: oben in der Datei,
        // nicht vor der SUB -- liefe das Hauptprogramm vorher, waere der
        // Platz beim ersten Aufruf noch leer.
        if umgebend.is_some() {
            let oben = kx.kopf();
            aus.push((format!("DIM {} AS {} global anlegen (oben in der Datei)", wort, typ),
                      vec![(oben, 0, 0, format!("DIM {} AS {}\n", wort, typ))]));
        }
    }

    // "NEW KATZE: die Klasse kuendigt eine Methode an, ohne sie auszufuellen
    // (laut). ..." -- das Geruest der fehlenden Methoden vor das END CLASS
    // der Klasse, mit der Kopfzeile der Ankuendigung (ohne ABSTRACT).
    if let (Some(a), Some(p)) = (meldung.find("NEW "), meldung.find("ohne sie auszufuellen (")) {
        let klasse = meldung[a + 4..].split(':').next().unwrap_or("").trim().to_string();
        let namen: Vec<String> = meldung[p + "ohne sie auszufuellen (".len()..].split(')').next().unwrap_or("")
            .split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
        if let Some(k) = fehlende_methoden(kx, &klasse, &namen) { aus.push(k); }
    }
    aus
}

/// Die Korrektur zu "die Klasse kuendigt ... an, ohne sie auszufuellen": je
/// fehlender Methode ihre Kopfzeile (aus der ABSTRACT-Zeile der naechsten
/// Vorfahrin, die sie ankuendigt) mit leerem Rumpf, eingefuegt vor END CLASS.
/// `None`, wenn die Klasse nicht in diesem Text steht oder eine Ankuendigung
/// nicht zu finden ist -- lieber keine Korrektur als eine halbe.
fn fehlende_methoden(kx: &KorrKontext, klasse: &str, namen: &[String]) -> Option<(String, Vec<Aenderung>)> {
    if namen.is_empty() { return None; }
    let bereiche = kx.bereiche();
    let finde = |n: &str| bereiche.iter().find(|b| matches!(b.art, "class" | "struct") && b.name.eq_ignore_ascii_case(n));
    let ziel = finde(klasse)?;
    if ziel.ende <= ziel.zeile || ziel.ende > kx.zeilen.len() { return None; }
    let einzug_von = |z: &str| z.chars().take_while(|c| *c == ' ' || *c == '\t').collect::<String>();
    let kopf_einzug = einzug_von(kx.zeilen[ziel.zeile - 1]);
    // Einrueckung der Mitglieder: die erste nicht leere Zeile im Rumpf, sonst vier mehr.
    let ein = (ziel.zeile..ziel.ende - 1).map(|i| kx.zeilen[i]).find(|z| !z.trim().is_empty())
        .map(einzug_von).unwrap_or_else(|| format!("{}    ", kopf_einzug));
    let mut text = String::new();
    let rumpf_leer = (ziel.zeile..ziel.ende - 1).all(|i| kx.zeilen[i].trim().is_empty());
    for (k, name) in namen.iter().enumerate() {
        // Die Kette der Vorfahrinnen hinauf bis zur ersten Ankuendigung.
        let mut gefunden: Option<String> = None;
        let mut aktuell = ziel;
        let mut schritte = 0;
        while gefunden.is_none() && schritte < 50 {
            schritte += 1;
            let kopf = crate::symbole::ohne_kommentare_und_texte(kx.zeilen[aktuell.zeile - 1]);
            let gross = kopf.to_ascii_uppercase();
            let Some(p) = gross.find(" EXTENDS ") else { break };
            let basis: String = kopf[p + 9..].trim_start().chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            let Some(b) = finde(&basis) else { break };
            for i in b.zeile..b.ende.saturating_sub(1) {
                let roh = kx.zeilen[i];
                let ohne = crate::symbole::ohne_kommentare_und_texte(roh);
                let t = ohne.trim();
                let tg = t.to_ascii_uppercase();
                let Some(rest) = tg.strip_prefix("ABSTRACT ").map(|_| t[9..].trim_start()) else { continue };
                let rg = rest.to_ascii_uppercase();
                let nach = if rg.starts_with("SUB ") { &rest[4..] } else if rg.starts_with("FUNCTION ") { &rest[9..] } else { continue };
                let n: String = nach.trim_start().chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if n.eq_ignore_ascii_case(name) {
                    // Die Kopfzeile wie geschrieben (samt Zeichenketten in Vorgabewerten).
                    // `ohne` hat so viele ZEICHEN wie `roh` (Kommentar und Texte als
                    // Leerzeichen) -- gezaehlt wird darum in Zeichen, nicht in Bytes.
                    let oz: Vec<char> = ohne.chars().collect();
                    let ab = (0..oz.len()).find(|&q| oz[q..].iter().take(8).collect::<String>().eq_ignore_ascii_case("abstract"))
                        .map(|q| q + 8).unwrap_or(0);
                    let laenge = ohne.trim_end().chars().count();
                    gefunden = Some(roh.chars().skip(ab).take(laenge.saturating_sub(ab)).collect::<String>().trim().to_string());
                    break;
                }
            }
            aktuell = b;
        }
        let kopf = gefunden?;
        let ende = if kopf.to_ascii_uppercase().starts_with("SUB") { "END SUB" } else { "END FUNCTION" };
        if k > 0 || !rumpf_leer { text.push('\n'); }
        text.push_str(&format!("{ein}{kopf}\n{ein}    \n{ein}{ende}\n"));
    }
    let titel = if namen.len() == 1 { format!("Fehlende Methode {} einfuegen", namen[0]) }
                else { format!("Fehlende Methoden einfuegen ({})", namen.join(", ")) };
    Some((titel, vec![(ziel.ende - 1, 0, 0, text)]))
}

/// Wo oben in der Datei etwas Neues hingehoert: hinter die IMPORTs, die schon
/// da sind, sonst hinter den Kopfkommentar.
fn kopf_ende(zeilen: &[&str]) -> usize {
    let mut ziel = 0;
    let mut hinter_import = None;
    for (k, t) in zeilen.iter().enumerate() {
        let r = t.trim().to_uppercase();
        if r.starts_with("IMPORT ") { hinter_import = Some(k + 1); continue; }
        if hinter_import.is_none() && (r.is_empty() || r.starts_with('\'') || r.starts_with("REM ")) { ziel = k + 1; continue; }
        break;
    }
    hinter_import.unwrap_or(ziel).min(zeilen.len())
}

/// Die rechte Seite von `name = ...` in dieser Zeile, falls sie so beginnt.
fn zuweisung_rechts(zeile: &str, name: &str) -> Option<String> {
    let t = zeile.trim_start();
    if t.len() < name.len() || !t[..name.len()].eq_ignore_ascii_case(name) { return None; }
    let rest = t[name.len()..].trim_start();
    let rest = rest.strip_prefix('=')?;
    Some(crate::symbole::ohne_inline_kommentar(rest).trim().to_string())
}

/// Ein Typ fuer ein neues DIM, geraten aus Name und zugewiesenem Wert.
fn typ_raten(name: &str, rechts: Option<&str>) -> &'static str {
    if name.ends_with('$') { return "STRING"; }
    let Some(r) = rechts else { return "INTEGER" };
    let gross = r.to_ascii_uppercase();
    if r.starts_with('"') || r.starts_with("f\"") || r.starts_with("!\"") || r.starts_with("f!\"") { return "STRING"; }
    if gross == "TRUE" || gross == "FALSE" { return "BOOLEAN"; }
    if r.parse::<i64>().is_ok() { return "INTEGER"; }
    if r.parse::<f64>().is_ok() { return "FLOAT"; }
    // Ein Aufruf eines Befehls mit bekanntem Ergebnis.
    let kopf: String = r.chars().take_while(|&c| c.is_alphanumeric() || c == '_' || c == '$').collect();
    if !kopf.is_empty() && r[kopf.len()..].trim_start().starts_with('(') {
        match crate::typen::builtin_typ(&kopf.to_lowercase()) {
            Some(crate::typen::Typ::Str) => return "STRING",
            Some(crate::typen::Typ::Float) => return "FLOAT",
            Some(crate::typen::Typ::Bool) => return "BOOLEAN",
            Some(crate::typen::Typ::Int) => return "INTEGER",
            _ => {}
        }
    }
    // Ein Text in der Rechnung macht sie zum Text, eine Kommazahl oder `/`
    // (liefert immer FLOAT) zur Kommazahl.
    if r.contains('"') { return "STRING"; }
    let ohne = crate::symbole::ohne_kommentare_und_texte(r);
    if ohne.contains('/') { return "FLOAT"; }
    let b: Vec<char> = ohne.chars().collect();
    for k in 1..b.len().saturating_sub(1) {
        if b[k] == '.' && b[k - 1].is_ascii_digit() && b[k + 1].is_ascii_digit() { return "FLOAT"; }
    }
    "INTEGER"
}

/// Die Korrekturen der Meldungen in den Zeilen von..bis als LSP-CodeActions
/// (`quickfix`), je eine mit ihrer Meldung als Bezug. Die Diagnose laeuft
/// dafuer frisch -- sie kostet Millisekunden, und ein zwischengespeicherter
/// Stand koennte zu einem anderen Text gehoeren.
pub fn schnellkorrekturen(text: &str, uri: &str, von: u64, bis: u64) -> Vec<Value> {
    let mut aus = Vec::new();
    for d in diagnose(text, &basis_von(uri)) {
        let z = d["range"]["start"]["line"].as_u64().unwrap_or(0);
        if z < von || z > bis { continue; }
        let Some(liste) = d["data"]["korrekturen"].as_array() else { continue };
        for k in liste {
            let edits: Vec<Value> = k["aenderungen"].as_array().cloned().unwrap_or_default().iter().map(|a| json!({
                "range": {"start": {"line": a["zeile"], "character": a["von"]},
                          "end": {"line": a["bis_zeile"], "character": a["bis"]}},
                "newText": a["text"],
            })).collect();
            let mut aenderung = serde_json::Map::new();
            aenderung.insert(uri.to_string(), Value::Array(edits));
            aus.push(json!({
                "title": k["titel"], "kind": "quickfix", "diagnostics": [d.clone()],
                "edit": {"changes": aenderung},
            }));
        }
    }
    aus
}

// ---------------------------------------------------------------- Hover-Daten

/// Handgepflegte Hover-Texte (`builtin_docs.json`, Name klein -> [Signatur, Text]).
fn handdoku() -> &'static HashMap<String, (String, String)> {
    static M: std::sync::OnceLock<HashMap<String, (String, String)>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        let mut m = HashMap::new();
        let raw = include_str!("../../../daten/builtin_docs.json");
        if let Ok(v) = serde_json::from_str::<Value>(raw) {
            if let Some(o) = v.get("docs").and_then(|d| d.as_object()) {
                for (k, e) in o {
                    let sig = e.get(0).and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let doc = e.get(1).and_then(|s| s.as_str()).unwrap_or("").to_string();
                    m.insert(k.to_lowercase(), (sig, doc));
                }
            }
        }
        m
    })
}

/// Kurzbeschreibungen aus `docs/` (`builtin_prosa.json`, Name gross -> Text).
fn prosa() -> &'static HashMap<String, String> {
    static M: std::sync::OnceLock<HashMap<String, String>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        let mut m = HashMap::new();
        let raw = include_str!("../../../daten/builtin_prosa.json");
        if let Ok(v) = serde_json::from_str::<Value>(raw) {
            if let Some(o) = v.get("docs").and_then(|d| d.as_object()) {
                for (k, e) in o { if let Some(t) = e.as_str() { m.insert(k.to_uppercase(), t.to_string()); } }
            }
        }
        m
    })
}

/// Dieselben Kurzbeschreibungen auf Englisch (`builtin_prosa.en.json`, aus
/// docs/en und dem englischen Buch) -- fuer den Hover mit `DHRT_LANG=en`.
fn prosa_en() -> &'static HashMap<String, String> {
    static M: std::sync::OnceLock<HashMap<String, String>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        let mut m = HashMap::new();
        let raw = include_str!("../../../daten/builtin_prosa.en.json");
        if let Ok(v) = serde_json::from_str::<Value>(raw) {
            if let Some(o) = v.get("docs").and_then(|d| d.as_object()) {
                for (k, e) in o { if let Some(t) = e.as_str() { m.insert(k.to_uppercase(), t.to_string()); } }
            }
        }
        m
    })
}

fn signatur(name: &str) -> Option<&'static str> {
    let klein = name.to_lowercase();
    crate::compiler::builtin_eintraege().iter()
        .find(|e| e.0.to_lowercase() == klein || e.0.to_lowercase() == format!("{}$", klein))
        .map(|e| e.1.as_str())
}

/// `(Signatur, Text)` fuer einen Builtin: Handdoku vor Prosa vor blosser
/// Signatur -- die handgepflegten Texte sind auf den Hover geschnitten, die
/// aus `docs/` sind Tabellenzellen. `name` kommt ohne `$` an.
pub fn builtin_doku(name: &str) -> Option<(String, String)> {
    let klein = name.to_lowercase();
    // Auf Englisch: die englische Kurzbeschreibung, sonst nur die Signatur --
    // die Handdoku und die deutsche Prosa sind deutsch.
    if crate::meldung::englisch() {
        let gross = name.to_uppercase();
        let sig = signatur(name).map(str::to_string);
        if let Some(t) = prosa_en().get(&gross).or_else(|| prosa_en().get(&format!("{}$", gross))) {
            return Some((sig.unwrap_or(gross), t.clone()));
        }
        return sig.map(|s| (s, String::new()));
    }
    if let Some(d) = handdoku().get(&klein).or_else(|| handdoku().get(&format!("{}$", klein))) {
        return Some(d.clone());
    }
    let gross = name.to_uppercase();
    if let Some(t) = prosa().get(&gross).or_else(|| prosa().get(&format!("{}$", gross))) {
        return Some((signatur(name).map(str::to_string).unwrap_or(gross), t.clone()));
    }
    signatur(name).map(|s| (s.to_string(), String::new()))
}

// ---------------------------------------------------------------- Features

pub fn vervollstaendigung(text: &str, basis: Option<&Path>, z0: usize, c0: usize) -> Vec<Value> {
    // Hinter `objekt.` nur, was das Objekt hat -- sofern der Uebersetzer die
    // Klasse kennt; sonst wie ueberall. Nur mit `basis`: dafuer wird der
    // ganze Text uebersetzt (CODE_COMPLETE ohne basis$ bleibt beim Text).
    if let (Some(basis), Some((a, e))) = (basis, mitglied_stelle(text, z0, c0, false)) {
        let z: Vec<char> = text.split('\n').nth(z0).unwrap_or("").chars().collect();
        let pl: String = z[a..c0.min(z.len())].iter().collect::<String>().to_lowercase();
        if let Some((_, defs)) = namensraum_mitglieder(text, basis, &wort_vor_punkt(&z, a)) {
            return defs.iter().filter(|d| d.name.to_lowercase().starts_with(&pl)).map(|d| {
                let art = match d.art { "sub" | "function" => CK_FUNCTION, "class" | "struct" => CK_CLASS,
                                        "enum" => CK_ENUM, "const" => CK_CONSTANT, _ => CK_VARIABLE };
                json!({"label": d.name, "kind": art, "detail": d.art})
            }).collect();
        }
        if let Some(fund) = mitglied_fund(text, basis, z0, a, e) {
            let z: Vec<char> = text.split('\n').nth(z0).unwrap_or("").chars().collect();
            let pl: String = z[a..c0.min(z.len())].iter().collect::<String>().to_lowercase();
            let namen = quell_namen(text, basis, &fund);
            return fund.mitglieder.iter()
                .filter(|m| m.name.to_lowercase().starts_with(&pl))
                .map(|m| {
                    let art = match m.art { "feld" => CK_FIELD, "property" => CK_PROPERTY, _ => CK_METHOD };
                    let name = namen.get(&m.name.to_lowercase()).cloned().unwrap_or_else(|| m.name.clone());
                    json!({"label": name, "kind": art, "detail": mitglied_kopf(m, &name)})
                }).collect();
        }
    }
    let (praefix, _) = symbole::praefix_bei(text, z0, c0);
    let pl = praefix.to_lowercase();
    let mut gesehen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out = Vec::new();
    let mut add = |label: &str, art: i64, detail: &str| {
        let k = label.to_lowercase();
        if gesehen.contains(&k) { return; }
        if !pl.is_empty() && !k.starts_with(&pl) { return; }
        gesehen.insert(k);
        out.push(json!({"label": label, "kind": art, "detail": detail}));
    };
    // Hinter einem Punkt, dessen Klasse nicht festzustellen war (ohne basis,
    // oder der Text uebersetzt gerade nicht): die Mitglieder ALLER Klassen
    // des Textes -- Befehle, Schluesselwoerter und Farben gehoeren nie hinter
    // einen Punkt (`t.la` bot sonst LAYER, LAYER_CLEAR ... an).
    if mitglied_stelle(text, z0, c0, false).is_some() {
        if let Value::Array(klassen) = klassen_struktur(text) {
            for k in &klassen {
                for m in k["mitglieder"].as_array().into_iter().flatten() {
                    let art = match m["art"].as_str().unwrap_or("") { "feld" | "konstante" => CK_FIELD,
                                                                      "property" => CK_PROPERTY, _ => CK_METHOD };
                    add(m["name"].as_str().unwrap_or(""), art, m["kopf"].as_str().unwrap_or(""));
                }
            }
        }
        return out;
    }
    for d in symbole::definitionen(text) {
        let art = match d.art { "sub" | "function" => CK_FUNCTION,
                                "class" | "struct" | "enum" | "const" => CK_CONSTANT, _ => CK_VARIABLE };
        add(&d.name, art, d.art);
    }
    for e in crate::compiler::builtin_eintraege() { add(&e.0.to_uppercase(), CK_FUNCTION, "Built-in"); }
    add("PI", CK_CONSTANT, "Konstante");
    add("TAU", CK_CONSTANT, "Konstante");
    for (n, _) in crate::vm::DEFAULT_COLORS { add(&n.to_uppercase(), CK_CONSTANT, "Konstante"); }
    for (n, _) in crate::vm::DEFAULT_KEYS { add(&n.to_uppercase(), CK_CONSTANT, "Konstante"); }
    for k in crate::lexer::KEYWORDS { add(&k.to_uppercase(), CK_KEYWORD, "Keyword"); }
    add("REM", CK_KEYWORD, "Keyword");
    out
}

pub fn hover(text: &str, basis: Option<&Path>, z0: usize, c0: usize) -> Option<Value> {
    let (wort, _, _) = symbole::wort_bei(text, z0, c0);
    if wort.is_empty() { return None; }
    // Hinter `objekt.` das Mitglied SEINER Klasse -- nur mit `basis`, weil
    // dafuer der ganze Text uebersetzt wird (die Parameternamen der IDE
    // fragen viele Stellen auf einmal und kommen ohne aus).
    if let Some(md) = basis.and_then(|b| namensraum_hover(text, b, z0, c0, &wort)
            .or_else(|| mitglied_hover(text, b, z0, c0, &wort))) {
        return Some(json!({"contents": {"kind": "markdown", "value": md}}));
    }
    let (sig, doc) = klassen_hover(text, z0, &wort)
        .or_else(|| builtin_doku(&wort))
        .or_else(|| symbole::nutzer_doku(text, &wort))?;
    let mut md = format!("```drachenhauch\n{}\n```", sig);
    if !doc.is_empty() { md.push_str("\n\n"); md.push_str(&doc); }
    Some(json!({"contents": {"kind": "markdown", "value": md}}))
}

// ------------------------------------------- Mitglieder hinter einem Punkt

/// Steht an (z0, c0) ein Mitglied hinter einem Punkt (`objekt.na|me`, auch
/// leer direkt hinter dem Punkt): Anfang und Ende des Worts in Zeichen.
/// `ganz` = das ganze Wort (Hover), sonst nur bis zur Marke (Vervollstaendigung).
/// Vor dem Punkt muss ein Name, `)` oder `]` stehen -- `1.5` ist keins.
fn mitglied_stelle(text: &str, z0: usize, c0: usize, ganz: bool) -> Option<(usize, usize)> {
    let z: Vec<char> = text.split('\n').nth(z0)?.chars().collect();
    // Ersetzt wird immer das ganze Wort, auch rechts der Marke -- sonst
    // haengte die Marke vor dem Rest und hiesse anders.
    let (_, wa, e) = symbole::wort_bei(text, z0, c0);
    let a = if ganz { wa } else { symbole::praefix_bei(text, z0, c0).1 };
    let e = e.max(a);
    if a < 2 || z[a - 1] != '.' { return None; }
    let davor = z[a - 2];
    if !(symbole::ist_ident(davor) || davor == ')' || davor == ']') { return None; }
    // `1.5`: eine Zahl vor dem Punkt ist kein Objekt.
    let mut k = a - 1;
    while k > 0 && symbole::ist_ident(z[k - 1]) { k -= 1; }
    if k < a - 1 && z[k..a - 1].iter().all(|c| c.is_ascii_digit()) { return None; }
    Some((a, e))
}

thread_local! {
    /// Das letzte Ergebnis -- die Statuszeile der IDE fragt bei jeder
    /// Bewegung der Marke, und an derselben Stelle steht dasselbe Mitglied.
    static LETZTER_FUND: std::cell::RefCell<Option<(String, PathBuf, Option<crate::compiler::MitgliedFund>)>> =
        const { std::cell::RefCell::new(None) };
}

/// Die Klasse vor dem Punkt samt Mitgliedern: das Wort zwischen `a` und `e`
/// wird durch eine Marke ersetzt und der Text uebersetzt -- der Uebersetzer
/// kennt Felder, Indizes, Aufrufe, `Self` und importierte Klassen. None, wenn
/// er die Klasse nicht weiss (auch, wenn der Text anderswo nicht uebersetzt).
fn mitglied_fund(text: &str, basis: &Path, z0: usize, a: usize, e: usize) -> Option<crate::compiler::MitgliedFund> {
    let mut zeilen: Vec<String> = text.split('\n').map(str::to_string).collect();
    let z: Vec<char> = zeilen.get(z0)?.chars().collect();
    let mut neu: String = z[..a].iter().collect();
    neu.push_str(crate::compiler::MITGLIED_MARKE);
    neu.extend(&z[e.min(z.len())..]);
    zeilen[z0] = neu;
    let neu_text = zeilen.join("\n");
    if let Some(f) = LETZTER_FUND.with(|l| l.borrow().as_ref()
            .filter(|(t, b, _)| *t == neu_text && b == basis).map(|(_, _, f)| f.clone())) {
        return f;
    }
    let fund = crate::compiler::mitglied_fund_sammeln(|| { let _ = crate::check_source(&neu_text, basis, "<editor>"); });
    LETZTER_FUND.with(|l| *l.borrow_mut() = Some((neu_text, basis.to_path_buf(), fund.clone())));
    fund
}

/// Das Wort direkt vor dem Punkt, der an Stelle `a - 1` steht.
fn wort_vor_punkt(z: &[char], a: usize) -> String {
    let ende = a.saturating_sub(1);
    let mut k = ende;
    while k > 0 && symbole::ist_ident(z[k - 1]) { k -= 1; }
    z[k..ende].iter().collect()
}

/// Ist `alias` ein Namensraum (`IMPORT "x.dh" AS alias`)? Dann der Text samt
/// Importen und die obersten Definitionen seiner Dateien -- SUB, FUNCTION,
/// CLASS, STRUCT, ENUM, CONST und globale DIMs, ohne PRIVATE. Ohne
/// Uebersetzen, also auch, wenn der Text gerade nicht uebersetzt.
fn namensraum_mitglieder(text: &str, basis: &Path, alias: &str) -> Option<(String, Vec<symbole::Definition>)> {
    if alias.is_empty() { return None; }
    let (quelle, _, herkunft, raeume) = crate::preprocess::process(text, basis).ok()?;
    let dateien: std::collections::HashSet<String> = raeume.iter()
        .filter(|(_, a)| a.eq_ignore_ascii_case(alias)).map(|(d, _)| d.clone()).collect();
    if dateien.is_empty() { return None; }
    let bereiche = symbole::bereiche(&quelle);
    let zeilen: Vec<&str> = quelle.split('\n').collect();
    let mut gesehen = std::collections::HashSet::new();
    let defs = symbole::definitionen(&quelle).into_iter().filter(|d| {
        let aus_raum = herkunft.get(d.zeile.saturating_sub(1)).map(|h| dateien.contains(&h.datei)).unwrap_or(false);
        // Oberste Ebene: in keinem Block (die Kopfzeile eines Blocks zaehlt).
        let oben = !bereiche.iter().any(|b| d.zeile > b.zeile && d.zeile <= b.ende);
        let privat = zeilen.get(d.zeile.saturating_sub(1))
            .map(|z| z.trim_start().to_ascii_uppercase().starts_with("PRIVATE ")).unwrap_or(false);
        aus_raum && oben && !privat && d.art != "param"
    }).filter(|d| gesehen.insert(d.name.to_lowercase())).collect();
    Some((quelle, defs))
}

/// Hover auf `kern.Name`: die Definition im Namensraum.
fn namensraum_hover(text: &str, basis: &Path, z0: usize, c0: usize, wort: &str) -> Option<String> {
    let (a, _) = mitglied_stelle(text, z0, c0, true)?;
    let z: Vec<char> = text.split('\n').nth(z0)?.chars().collect();
    let alias = wort_vor_punkt(&z, a);
    let (quelle, defs) = namensraum_mitglieder(text, basis, &alias)?;
    let Some(d) = defs.iter().find(|d| d.name.eq_ignore_ascii_case(wort)) else {
        return Some(crate::meldung::t(&format!("Der Namensraum {} hat nichts namens '{}'.", alias, wort)));
    };
    let (sig, doc) = symbole::doku_der_definition(&quelle, d)?;
    let wem = crate::meldung::t(&format!("aus dem Namensraum {}", alias));
    let mut md = format!("```drachenhauch\n{}\n```\n\n{}", sig, wem);
    if !doc.is_empty() { md.push_str("\n\n"); md.push_str(&doc); }
    Some(md)
}

/// Alle Methodenaufrufe des Textes, deren Klasse der Uebersetzer kennt, in
/// EINEM Lauf: (Zeile im Puffer, Methode, Klasse, Kopfzeile der Methode) --
/// fuer die Parameternamen an Aufrufen. Die Kopfzeile kommt aus der Klasse,
/// in der die Methode steht, in der Schreibweise des Quelltexts; fehlt sie
/// dort, baut der Server sie aus den Angaben des Uebersetzers.
pub fn methoden_aufrufe(text: &str, basis: &Path) -> Vec<(u32, String, String, String)> {
    let roh = crate::compiler::methoden_aufrufe_sammeln(|| { let _ = crate::check_source(text, basis, "<editor>"); });
    if roh.is_empty() { return Vec::new(); }
    let quelle = quelle_mit_importen(text, basis);
    let herkunft = crate::preprocess::process(text, basis).ok().map(|r| r.2);
    let mut kopf: HashMap<(String, String), String> = HashMap::new();
    let mut aus = Vec::new();
    let mut gesehen = std::collections::HashSet::new();
    for (z, name, von, param) in roh {
        let zeile = match herkunft.as_ref().and_then(|h| h.get((z as usize).saturating_sub(1))) {
            Some(h) if h.datei.is_empty() => h.zeile,
            Some(_) => continue,
            None => z,
        };
        if !gesehen.insert((zeile, name.clone())) { continue; }
        let sig = kopf.entry((von.clone(), name.clone())).or_insert_with(|| {
            mitglied_definitionen(&quelle, &von).into_iter()
                .find(|d| d.name.eq_ignore_ascii_case(&name) && matches!(d.art, "sub" | "function"))
                .and_then(|d| symbole::doku_der_definition(&quelle, &d)).map(|(s, _)| s)
                .unwrap_or_else(|| format!("SUB {}({})", name, param.join(", ")))
        }).clone();
        aus.push((zeile, name, crate::namensraum::lesbar_text(&von.to_uppercase()), sig));
    }
    aus.sort_by_key(|e| e.0);
    aus
}

/// Der Name einer Klasse ohne Namensraum (`kern@spieler` -> `spieler`).
fn schlichter_name(klasse: &str) -> &str {
    klasse.rsplit('@').next().unwrap_or(klasse)
}

/// Die Mitglieder einer Klasse (Bereich `k`): Felder auf Klassenebene --
/// nicht die lokalen Variablen ihrer Methoden -- und die Methoden selbst.
fn klassen_mitglieder(quelle: &str, k: &symbole::Bereich, bereiche: &[symbole::Bereich]) -> Vec<symbole::Definition> {
    let methoden: Vec<&symbole::Bereich> = bereiche.iter()
        .filter(|x| matches!(x.art, "sub" | "function" | "property") && x.zeile > k.zeile && x.ende <= k.ende).collect();
    symbole::definitionen(quelle).into_iter().filter(|d| d.zeile > k.zeile && d.zeile < k.ende && match d.art {
        "sub" | "function" | "property" => true,
        "dim" => !methoden.iter().any(|m| m.zeile <= d.zeile && d.zeile <= m.ende),
        _ => false,
    }).collect()
}

/// Der Typ hinter `AS` in einem Stueck Quelltext, bis zum Ende, einem `=`
/// oder einem Komma auf oberster Ebene; leer, wenn keiner dasteht.
fn typ_hinter_as(stueck: &str) -> String {
    let g = stueck.to_ascii_uppercase();
    let Some(p) = g.find(" AS ").map(|p| p + 4) else { return String::new() };
    let mut tiefe = 0i32;
    let mut ende = stueck.len();
    for (i, c) in stueck[p..].char_indices() {
        match c {
            '(' | '[' => tiefe += 1,
            ')' | ']' => tiefe -= 1,
            '=' | ',' | ':' if tiefe <= 0 => { ende = p + i; break; }
            _ => {}
        }
    }
    stueck[p..ende].trim().to_string()
}

/// Die Klassen und Structs eines Quelltexts mit ihren Mitgliedern, ohne zu
/// uebersetzen -- ein halb getippter Text liefert, was zu lesen ist. Je
/// Klasse: Name, Art, Vorfahrin (EXTENDS), Zeilen, Kommentar darueber; je
/// Mitglied: Name, Art (feld/konstante/sub/function/property), Typ, Kopfzeile,
/// Zeile, Kommentar. GET und SET einer PROPERTY sind EIN Mitglied.
pub fn klassen_struktur(quelle: &str) -> Value {
    static BASIS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let basis_re = BASIS.get_or_init(|| regex::Regex::new(r"(?i)\bEXTENDS\s+([A-Za-z_][A-Za-z0-9_.]*)").expect("Regex"));
    let zeilen: Vec<&str> = quelle.split('\n').collect();
    let bereiche = symbole::bereiche(quelle);
    let alle = symbole::definitionen(quelle);
    let mut aus = Vec::new();
    for k in bereiche.iter().filter(|x| matches!(x.art, "class" | "struct")) {
        let kopf = zeilen.get(k.zeile - 1).map(|z| symbole::ohne_kommentare_und_texte(z)).unwrap_or_default();
        let vorfahrin = basis_re.captures(&kopf).map(|c| c[1].to_string()).unwrap_or_default();
        let methoden: Vec<&symbole::Bereich> = bereiche.iter()
            .filter(|x| matches!(x.art, "sub" | "function" | "property") && x.zeile > k.zeile && x.ende <= k.ende).collect();
        let mut mitglieder = Vec::new();
        let mut gesehen = std::collections::HashSet::new();
        for d in alle.iter().filter(|d| d.zeile > k.zeile && d.zeile < k.ende) {
            let in_methode = methoden.iter().any(|m| m.zeile <= d.zeile && d.zeile <= m.ende);
            let art = match d.art {
                "sub" | "function" | "property" => d.art,
                "dim" if !in_methode => "feld",
                "const" if !in_methode => "konstante",
                _ => continue,
            };
            if !gesehen.insert((art, d.name.to_lowercase())) { continue; }
            let (sig, doku) = symbole::doku_der_definition(quelle, d).unwrap_or_default();
            let typ = match art {
                "feld" | "konstante" => {
                    let roh = symbole::ohne_kommentare_und_texte(zeilen[d.zeile - 1]);
                    // Ab dem Namen: `DIM a AS INTEGER, s AS STRING` gibt jedem seinen.
                    let ab: usize = roh.char_indices().nth(d.spalte - 1).map(|(b, _)| b).unwrap_or(0);
                    typ_hinter_as(&format!(" {}", &roh[ab..]))
                }
                "sub" => String::new(),
                _ => match sig.rfind(')') { Some(p) => typ_hinter_as(&sig[p..]), None => typ_hinter_as(&sig) },
            };
            mitglieder.push(json!({"name": d.name, "art": art, "typ": typ, "kopf": sig,
                                   "zeile": d.zeile, "kommentar": doku}));
        }
        aus.push(json!({"name": k.name, "art": k.art, "basis": vorfahrin, "zeile": k.zeile, "bis": k.ende,
                        "kommentar": symbole::kommentar_darueber(&zeilen, k.zeile - 1),
                        "mitglieder": mitglieder}));
    }
    Value::Array(aus)
}

/// Ein Name in einer Methode: erst ein Parameter oder eine Variable DIESER
/// Methode (die erste Definition im Text waere womoeglich ein gleichnamiges
/// Feld der Klasse), sonst das
/// Mitglied der umgebenden Klasse, dann ihrer Vorfahrinnen (EXTENDS). Ohne
/// das sprang F12 auf `laut()` in `Tier` in die erste Unterklasse, die
/// `laut` ausschreibt -- die erste Definition im Text.
fn mitglied_in_klassenkette(text: &str, zeile: usize, wort: &str) -> Option<symbole::Definition> {
    let bereiche = symbole::bereiche(text);
    let klasse = bereiche.iter().filter(|b| matches!(b.art, "class" | "struct") && b.zeile <= zeile && zeile <= b.ende)
        .max_by_key(|b| b.zeile)?;
    let ziel = wort.to_lowercase();
    let alle = symbole::definitionen(text);
    if let Some(m) = bereiche.iter().filter(|b| matches!(b.art, "sub" | "function" | "property")
                                         && b.zeile <= zeile && zeile <= b.ende).max_by_key(|b| b.zeile) {
        if let Some(d) = alle.iter().find(|d| matches!(d.art, "param" | "dim" | "const") && d.zeile >= m.zeile
                                         && d.zeile <= m.ende && d.name.to_lowercase() == ziel) {
            return Some(d.clone());
        }
    }
    let mut name = klasse.name.clone();
    let mut gesehen = std::collections::HashSet::new();
    while gesehen.insert(name.to_lowercase()) {
        let Some(k) = bereiche.iter().find(|b| matches!(b.art, "class" | "struct") && b.name.eq_ignore_ascii_case(&name)) else { break };
        if let Some(d) = klassen_mitglieder(text, k, &bereiche).into_iter().find(|d| d.name.to_lowercase() == ziel) {
            return Some(d);
        }
        let kopf = text.lines().nth(k.zeile - 1).map(symbole::ohne_kommentare_und_texte).unwrap_or_default();
        let gross = kopf.to_ascii_uppercase();
        let Some(p) = gross.find(" EXTENDS ") else { break };
        name = kopf[p + 9..].trim().split(|c: char| !(c.is_alphanumeric() || c == '_')).next().unwrap_or("").to_string();
        if name.is_empty() { break; }
    }
    None
}

/// Die Mitglieder der Klasse `klasse` im Quelltext samt Importen.
fn mitglied_definitionen(quelle: &str, klasse: &str) -> Vec<symbole::Definition> {
    let bereiche = symbole::bereiche(quelle);
    let name = schlichter_name(klasse);
    match bereiche.iter().find(|x| matches!(x.art, "class" | "struct") && x.name.eq_ignore_ascii_case(name)) {
        Some(k) => klassen_mitglieder(quelle, k, &bereiche),
        None => Vec::new(),
    }
}

/// Hover auf einen Namen INNERHALB einer Klasse (`DIM name`, `hp` in einer
/// Methode): das Mitglied DIESER Klasse, nicht die erste gleichnamige
/// Definition im Text. Ein Parameter oder eine lokale Variable der Methode
/// gleichen Namens verdeckt das Feld -- dann gilt der gewoehnliche Weg.
fn klassen_hover(text: &str, z0: usize, wort: &str) -> Option<(String, String)> {
    let zeile = z0 + 1;
    let bereiche = symbole::bereiche(text);
    let k = bereiche.iter().filter(|x| matches!(x.art, "class" | "struct") && x.zeile <= zeile && zeile <= x.ende).last()?;
    if let Some(mt) = bereiche.iter().find(|x| matches!(x.art, "sub" | "function" | "property")
            && x.zeile > k.zeile && x.zeile <= zeile && zeile <= x.ende) {
        let lokal = symbole::definitionen(text).iter().any(|d| d.zeile >= mt.zeile && d.zeile <= mt.ende
            && d.name.eq_ignore_ascii_case(wort) && matches!(d.art, "dim" | "param"));
        if lokal { return None; }
    }
    let m = klassen_mitglieder(text, k, &bereiche).into_iter().find(|d| d.name.eq_ignore_ascii_case(wort))?;
    symbole::doku_der_definition(text, &m)
}

/// Der Quelltext, in dem die Klassen stehen: der Puffer samt Importen.
fn quelle_mit_importen(text: &str, basis: &Path) -> String {
    crate::preprocess::process(text, basis).map(|r| r.0).unwrap_or_else(|_| text.to_string())
}

/// Die Schreibweise der Mitglieder im Quelltext (der Uebersetzer kennt sie
/// klein): klein -> wie geschrieben.
fn quell_namen(text: &str, basis: &Path, fund: &crate::compiler::MitgliedFund) -> HashMap<String, String> {
    let quelle = quelle_mit_importen(text, basis);
    let mut klassen: Vec<&str> = fund.mitglieder.iter().map(|m| m.von.as_str()).collect();
    klassen.dedup();
    let mut namen = HashMap::new();
    for k in klassen {
        for d in mitglied_definitionen(&quelle, k) {
            namen.entry(d.name.to_lowercase()).or_insert(d.name);
        }
    }
    namen
}

/// Eine Zeile, wie das Mitglied aussieht, wenn der Quelltext es nicht zeigt.
fn mitglied_kopf(m: &crate::compiler::Mitglied, name: &str) -> String {
    let typ = |t: &str| if t.is_empty() { String::new() } else { format!(" AS {}", t) };
    match m.art {
        "feld" => format!("{}{}", name, typ(&m.typ)),
        "property" => format!("PROPERTY {}{}", name, typ(&m.typ)),
        art => {
            let p: Vec<String> = m.parameter.iter().map(|(n, t)| format!("{}{}", n, typ(t))).collect();
            format!("{} {}({}){}", if art == "function" { "FUNCTION" } else { "SUB" }, name, p.join(", "), typ(&m.typ))
        }
    }
}

/// Hover auf `objekt.mitglied`: die Zeile des Mitglieds in SEINER Klasse
/// (nicht die erste gleichnamige im Text), wem es gehoert, und der
/// Kommentar darueber. None, wo kein Punkt davor steht oder die Klasse
/// nicht feststeht -- dann gilt der gewoehnliche Hover.
fn mitglied_hover(text: &str, basis: &Path, z0: usize, c0: usize, wort: &str) -> Option<String> {
    let (a, e) = mitglied_stelle(text, z0, c0, true)?;
    let fund = mitglied_fund(text, basis, z0, a, e)?;
    let klasse = crate::namensraum::lesbar_text(&fund.klasse.to_uppercase());
    let Some(m) = fund.mitglieder.iter().find(|m| m.name.eq_ignore_ascii_case(wort)) else {
        return Some(crate::meldung::t(&format!("{} hat kein Mitglied '{}'.", klasse, wort)));
    };
    let quelle = quelle_mit_importen(text, basis);
    let def = mitglied_definitionen(&quelle, &m.von).into_iter().find(|d| d.name.eq_ignore_ascii_case(wort));
    let (sig, doc) = def.as_ref().and_then(|d| symbole::doku_der_definition(&quelle, d))
        .unwrap_or_else(|| (mitglied_kopf(m, wort), String::new()));
    let von = crate::namensraum::lesbar_text(&m.von.to_uppercase());
    let was = match m.art { "feld" => "Feld", "property" => "PROPERTY", _ => "Methode" };
    let mut wem = format!("{} von {}", was, klasse);
    if !m.von.eq_ignore_ascii_case(&fund.klasse) { wem.push_str(&format!(", geerbt von {}", von)); }
    let mut md = format!("```drachenhauch\n{}\n```\n\n{}", sig, crate::meldung::t(&wem));
    if !doc.is_empty() { md.push_str("\n\n"); md.push_str(&doc); }
    Some(md)
}

fn ort(uri: &str, zeile: usize, spalte: usize, spalte_ende: usize) -> Value {
    json!({"uri": uri, "range": {
        "start": {"line": zeile.saturating_sub(1), "character": spalte.saturating_sub(1)},
        "end": {"line": zeile.saturating_sub(1), "character": spalte_ende.saturating_sub(1)}}})
}

/// Ein Symbol im ganzen Text umbenennen -- der geaenderte Quelltext, oder
/// `None`, wenn an der Stelle kein Name steht oder der neue keiner ist.
///
/// Ersetzt wird ueber `symbole::fundstellen`, also ohne Kommentare und
/// Zeichenketten (die Ausdruecke in f-Strings zaehlen mit). Von hinten nach
/// vorn je Zeile, damit die Spalten der noch offenen Treffer stimmen.
/// Ein Name, den die Laufzeit selbst mitbringt: ein Befehl aus dem
/// Befehlsverzeichnis oder eine vorbelegte Konstante.
fn eingebaut(wort: &str) -> bool {
    crate::vm::ist_vorbelegter_name(&wort.to_lowercase())
        || crate::compiler::builtin_eintraege().iter().any(|(n, _, _)| n.eq_ignore_ascii_case(wort))
}

pub fn umbenennen(text: &str, z0: usize, c0: usize, neu: &str) -> Option<String> {
    let (wort, _, _) = symbole::wort_bei(text, z0, c0);
    if wort.is_empty() { return None; }
    // Ein Schluesselwort umzubenennen macht aus dem Programm Buchstabensalat,
    // und die Marke steht schnell einmal auf einem: nichts tun, statt es zu
    // tun. Builtins bleiben erlaubt -- eine eigene Variable darf seit
    // 2026-09-04 heissen wie eines, und die will man umbenennen koennen.
    if crate::lexer::keyword(&wort.to_lowercase()).is_some() { return None; }
    // Ein EINGEBAUTER Name (Befehl oder vorbelegte Konstante wie KEY_DOWN,
    // RED, PI), den das Programm nirgends selbst anlegt: umbenannt stuende
    // danach ein unbekannter Name da, wo der Befehl stand. Heisst eine
    // eigene Variable so (DIM red), gilt das Umbenennen weiter ihr.
    if eingebaut(&wort) && !symbole::definitionen(text).iter().any(|d| d.name.eq_ignore_ascii_case(&wort)) {
        return None;
    }
    let mut n = neu.chars();
    match n.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return None,
    }
    if !n.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$') { return None; }
    let mut zeilen: Vec<String> = text.split('\n').map(str::to_string).collect();
    let mut treffer = symbole::fundstellen(text, &wort);
    treffer.sort_by(|a, b| b.zeile.cmp(&a.zeile).then(b.spalte.cmp(&a.spalte)));
    for f in treffer {
        let Some(zeile) = zeilen.get_mut(f.zeile - 1) else { continue };
        let zeichen: Vec<char> = zeile.chars().collect();
        let (von, bis) = (f.spalte - 1, (f.spalte_ende - 1).min(zeichen.len()));
        if von > zeichen.len() { continue; }
        let mut aus: String = zeichen[..von].iter().collect();
        aus.push_str(neu);
        aus.extend(&zeichen[bis..]);
        *zeile = aus;
    }
    Some(zeilen.join("\n"))
}

pub fn definition(text: &str, uri: &str, z0: usize, c0: usize) -> Value {
    let (wort, _, _) = symbole::wort_bei(text, z0, c0);
    if wort.is_empty() { return Value::Null; }
    if let Some(d) = mitglied_in_klassenkette(text, z0 + 1, &wort) {
        return ort(uri, d.zeile, d.spalte, d.spalte_ende);
    }
    match symbole::definition(text, &wort) {
        Some(d) => ort(uri, d.zeile, d.spalte, d.spalte_ende),
        None => Value::Null,
    }
}

pub fn fundstellen(text: &str, uri: &str, z0: usize, c0: usize) -> Value {
    let (wort, _, _) = symbole::wort_bei(text, z0, c0);
    if wort.is_empty() { return json!([]); }
    Value::Array(symbole::fundstellen(text, &wort).into_iter()
        .map(|f| ort(uri, f.zeile, f.spalte, f.spalte_ende)).collect())
}

/// Gliederung: CLASS/STRUCT mit ihren Methoden und Properties verschachtelt
/// (ueber die Zeilenbereiche), dazu SUB/FUNCTION und ENUMs oben.
pub fn gliederung(text: &str) -> Value {
    // `detail` traegt die Art in Worten: die SymbolKind des Protokolls
    // kennt keine SUB, und ohne das hiess in Gliederung und Pfadleiste der
    // IDE jede SUB "function".
    fn knoten(name: &str, art: i64, wort: &str, von: usize, bis: usize) -> Value {
        let r = json!({"start": {"line": von.saturating_sub(1), "character": 0},
                       "end": {"line": bis.saturating_sub(1), "character": 0}});
        json!({"name": name, "detail": wort, "kind": art, "range": r, "selectionRange": r, "children": []})
    }
    let bereiche = symbole::bereiche(text);
    let mut wurzeln: Vec<Value> = Vec::new();
    // Stapel aus (Bereich, Pfad zum Knoten in `wurzeln`).
    let mut stapel: Vec<(symbole::Bereich, Vec<usize>)> = Vec::new();
    for b in bereiche {
        let art = match b.art { "class" => SK_CLASS, "struct" => SK_STRUCT, "property" => SK_PROPERTY, "enum" => SK_ENUM, _ => SK_FUNCTION };
        let k = knoten(&b.name, art, b.art, b.zeile, b.ende);
        while let Some((oben, _)) = stapel.last() {
            if oben.zeile <= b.zeile && b.zeile <= oben.ende { break; }
            stapel.pop();
        }
        let pfad = if let Some((_, eltern)) = stapel.last() {
            let mut ziel = &mut wurzeln;
            for &i in eltern { ziel = ziel[i]["children"].as_array_mut().unwrap(); }
            ziel.push(k);
            let mut p = eltern.clone(); p.push(ziel.len() - 1); p
        } else {
            wurzeln.push(k);
            vec![wurzeln.len() - 1]
        };
        stapel.push((b, pfad));
    }
    Value::Array(wurzeln)
}

// ---------------------------------------------------------------- Server

fn uri_zu_pfad(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let mut s = prozent_dekodieren(rest);
    // Windows: "/C:/..." -> "C:/..."
    if cfg!(windows) && s.len() >= 3 && s.starts_with('/') && s.as_bytes()[2] == b':' { s.remove(0); }
    Some(PathBuf::from(s))
}

fn prozent_dekodieren(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) { out.push(v); i += 3; continue; }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn basis_von(uri: &str) -> PathBuf {
    uri_zu_pfad(uri).and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

pub struct Server {
    sender: Sender,
    docs: HashMap<String, String>,
    generation: Arc<Mutex<HashMap<String, u64>>>,
    pub beendet: bool,
    /// Wartezeit vor der Diagnose (Tests setzen 0).
    pub verzoegerung_ms: u64,
}

impl Server {
    pub fn neu(ausgabe: Box<dyn Write + Send>) -> Self {
        Server { sender: Arc::new(Mutex::new(ausgabe)), docs: HashMap::new(),
                 generation: Arc::new(Mutex::new(HashMap::new())), beendet: false, verzoegerung_ms: 120 }
    }

    fn antworten(&self, id: &Value, ergebnis: Value) {
        senden(&self.sender, json!({"jsonrpc": "2.0", "id": id, "result": ergebnis}));
    }

    fn melden(&self, methode: &str, params: Value) {
        senden(&self.sender, json!({"jsonrpc": "2.0", "method": methode, "params": params}));
    }

    /// Eine eingehende Nachricht bearbeiten. Kein Objekt (z. B. ein
    /// Batch-Array) wird uebergangen; ein unbekanntes Verfahren mit `id`
    /// bekommt `null`, damit der Client nicht wartet.
    pub fn bearbeiten(&mut self, msg: &Value) {
        let Some(obj) = msg.as_object() else { return };
        let Some(methode) = obj.get("method").and_then(|m| m.as_str()) else { return };
        let id = obj.get("id").cloned();
        let params = obj.get("params").cloned().unwrap_or_else(|| json!({}));
        let ergebnis = match methode {
            "initialize" => json!({
                "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": {"triggerCharacters": ["."]},
                    "hoverProvider": true, "definitionProvider": true,
                    "referencesProvider": true, "documentSymbolProvider": true,
                    "codeActionProvider": true,
                },
                "serverInfo": {"name": NAME, "version": crate::fassung()},
            }),
            "initialized" => Value::Null,
            "shutdown" => { self.beendet = true; Value::Null }
            "exit" => { self.beendet = true; Value::Null }
            "textDocument/didOpen" => {
                let doc = &params["textDocument"];
                let uri = doc["uri"].as_str().unwrap_or("").to_string();
                self.docs.insert(uri.clone(), doc["text"].as_str().unwrap_or("").to_string());
                self.diagnose_starten(&uri);
                return;
            }
            "textDocument/didChange" => {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                if let Some(letzte) = params["contentChanges"].as_array().and_then(|a| a.last()) {
                    self.docs.insert(uri.clone(), letzte["text"].as_str().unwrap_or("").to_string());
                }
                self.diagnose_starten(&uri);
                return;
            }
            "textDocument/didClose" => {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                self.docs.remove(&uri);
                // Ein laufender Faden soll fuer das geschlossene Dokument
                // nichts mehr schicken.
                if let Ok(mut g) = self.generation.lock() { *g.entry(uri.clone()).or_insert(0) += 1; }
                self.melden("textDocument/publishDiagnostics", json!({"uri": uri, "diagnostics": []}));
                return;
            }
            "textDocument/completion" | "textDocument/hover" | "textDocument/definition"
            | "textDocument/references" => {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                let z0 = params["position"]["line"].as_u64().unwrap_or(0) as usize;
                let c0 = params["position"]["character"].as_u64().unwrap_or(0) as usize;
                match methode {
                    "textDocument/completion" => Value::Array(vervollstaendigung(&text, Some(&basis_von(&uri)), z0, c0)),
                    "textDocument/hover" => hover(&text, Some(&basis_von(&uri)), z0, c0).unwrap_or(Value::Null),
                    "textDocument/definition" => definition(&text, &uri, z0, c0),
                    _ => fundstellen(&text, &uri, z0, c0),
                }
            }
            "textDocument/codeAction" => {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                let von = params["range"]["start"]["line"].as_u64().unwrap_or(0);
                let bis = params["range"]["end"]["line"].as_u64().unwrap_or(von);
                Value::Array(schnellkorrekturen(&text, &uri, von, bis))
            }
            "textDocument/documentSymbol" => {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                gliederung(self.docs.get(uri).map(String::as_str).unwrap_or(""))
            }
            _ => Value::Null,
        };
        if let Some(id) = id { self.antworten(&id, ergebnis); }
    }

    fn diagnose_starten(&self, uri: &str) {
        let text = self.docs.get(uri).cloned().unwrap_or_default();
        let gen = {
            let mut g = self.generation.lock().unwrap();
            let e = g.entry(uri.to_string()).or_insert(0);
            *e += 1;
            *e
        };
        let generation = Arc::clone(&self.generation);
        let sender = Arc::clone(&self.sender);
        let uri = uri.to_string();
        let basis = basis_von(&uri);
        let warte = self.verzoegerung_ms;
        std::thread::Builder::new().name("lsp-diagnose".into()).spawn(move || {
            if warte > 0 { std::thread::sleep(Duration::from_millis(warte)); }
            let aktuell = |g: &Arc<Mutex<HashMap<String, u64>>>| g.lock().map(|m| m.get(&uri) == Some(&gen)).unwrap_or(false);
            if !aktuell(&generation) { return; }
            let diags = diagnose(&text, &basis);
            if !aktuell(&generation) { return; }
            senden(&sender, json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                                   "params": {"uri": uri, "diagnostics": diags}}));
        }).ok();
    }
}

/// stdin/stdout bedienen, bis `exit` kommt oder stdin endet.
pub fn serve() -> std::process::ExitCode {
    let stdin = io::stdin();
    let mut eingang = stdin.lock();
    let mut server = Server::neu(Box::new(io::stdout()));
    loop {
        let msg = match nachricht_lesen(&mut eingang) {
            Ok(Some(m)) => m,
            Ok(None) => break,
            Err(e) => { eprintln!("[{}] {}", NAME, e); continue; }
        };
        let ist_exit = msg.get("method").and_then(|m| m.as_str()) == Some("exit");
        server.bearbeiten(&msg);
        if server.beendet && ist_exit { break; }
    }
    std::process::ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    #[test]
    fn klassen_mit_mitgliedern_ohne_uebersetzen() {
        // Ein Fehler im Text (das IF ohne END) haelt nichts auf.
        let text = "' Ein Tier\nCLASS Tier\n    DIM hp, mp AS INTEGER\n    STATIC CONST MAXHP AS INTEGER = 9\n\
            \x20   PROPERTY GET titel() AS STRING\n        IF x THEN\n    END PROPERTY\n\
            \x20   PROPERTY SET titel(v AS STRING)\n    END PROPERTY\n\
            \x20   SUB laut(x AS INTEGER)\n        DIM lokal AS INTEGER\n    END SUB\nEND CLASS\n\
            CLASS Hund EXTENDS Tier\n    FUNCTION alter(a AS INTEGER, _\n                   b AS FLOAT) AS INTEGER\n    END FUNCTION\nEND CLASS\n";
        let v = super::klassen_struktur(text);
        let k = v.as_array().unwrap();
        assert_eq!(k.len(), 2);
        assert_eq!(k[0]["name"], "Tier");
        assert_eq!(k[0]["kommentar"], "Ein Tier");
        assert_eq!(k[1]["basis"], "Tier");
        let m: Vec<String> = k[0]["mitglieder"].as_array().unwrap().iter()
            .map(|m| format!("{}:{}:{}", m["name"].as_str().unwrap(), m["art"].as_str().unwrap(), m["typ"].as_str().unwrap())).collect();
        // Die lokale Variable der Methode ist kein Feld, GET und SET sind EIN Mitglied.
        assert_eq!(m, ["hp:feld:INTEGER", "mp:feld:INTEGER", "MAXHP:konstante:INTEGER", "titel:property:STRING", "laut:sub:"]);
        let f = &k[1]["mitglieder"][0];
        assert_eq!(f["typ"], "INTEGER");
        assert_eq!(f["kopf"], "FUNCTION alter(a AS INTEGER, b AS FLOAT) AS INTEGER");
    }

    #[test]
    fn korrekturen_aus_der_meldung() {
        let text = "DIM zaehler AS INTEGER
SUB tu()
    FOR i = 1 TO 3
        zaehlr = zaehlr + 1
    NEXT
END SUB";
        let m = "'zaehlr' wird hier gelesen, aber nirgends im Programm mit DIM oder CONST angelegt. Meintest du 'zaehler'?";
        let k = korrekturen(text, 3, 8, 14, m);
        assert_eq!(k[0].0, "Ersetzen durch zaehler");
        assert_eq!(k[0].1, vec![(3, 8, 14, "zaehler".to_string())]);
        // Das DIM kommt unter den Kopf der SUB, nicht in die Schleife.
        assert_eq!(k[1].0, "DIM zaehlr AS INTEGER anlegen");
        assert_eq!(k[1].1, vec![(2, 0, 0, "    DIM zaehlr AS INTEGER
".to_string())]);
        // Auf oberster Ebene vor den Block, mit geratenem Typ.
        let t2 = "WHILE TRUE
    s = \"a\"
WEND";
        let k = korrekturen(t2, 1, 4, 5, "'s' wird hier beschrieben, aber nirgends im Programm mit DIM oder CONST angelegt.");
        assert_eq!(k[0].1, vec![(0, 0, 0, "DIM s AS STRING
".to_string())]);
        // Klammern, der Kommentar bleibt draussen.
        let k = korrekturen("PRNT \"x\" ' hallo", 0, 0, 4, "Befehle und SUBs bekommen ihre Werte in Klammern: PRNT(...)");
        assert_eq!(k[0].1, vec![(0, 4, 8, "(\"x\")".to_string())]);
        // Umsteiger: nur eindeutige Namen.
        let k = korrekturen("s = UCASE$(s)", 0, 4, 10, "'UCASE$' heisst in Drachenhauch UPPER$(text)");
        assert_eq!(k[0].0, "Ersetzen durch UPPER$");
        assert!(korrekturen("x = LEFT(s, 1)", 0, 4, 8, "heisst in Drachenhauch LEFT$(text, n) bzw. RIGHT$(text, n)").is_empty());
        assert_eq!(korrekturen("IF a != b THEN", 0, 5, 6, "Ungleich schreibt man in Drachenhauch <>")[0].1,
                   vec![(0, 5, 7, "<>".to_string())]);
        // Eine Meldung ohne Vorschlag bleibt ohne Korrektur.
        assert!(korrekturen("PRINT(1)", 0, 0, 5, "Irgendwas").is_empty());
    }

    #[test]
    fn schreibweise_und_globales_dim() {
        // Der Compiler schlaegt "lsumme" vor; eingesetzt wird "lSumme".
        let t = "IMPORT \"gui\"\nDIM lSumme AS INTEGER\nPRINT summ";
        let k = korrekturen(t, 2, 6, 10, "'summ' wird hier gelesen, aber nirgends im Programm mit DIM oder CONST angelegt. Meintest du 'lsumme'?");
        assert_eq!(k[0].0, "Ersetzen durch lSumme");
        assert_eq!(k[0].1, vec![(2, 6, 10, "lSumme".to_string())]);
        // Im Unterprogramm: lokal unter den Kopf UND global oben hinter die
        // IMPORTs (nicht vor die SUB -- das Hauptprogramm kann vorher laufen).
        let t = "' Kopf\nIMPORT \"gui\"\n\nSUB s()\n    id = 5\nEND SUB";
        let m = "'id' wird hier beschrieben, aber nirgends im Programm mit DIM oder CONST angelegt.";
        let k = korrekturen(t, 4, 4, 6, m);
        assert_eq!(k[0].1, vec![(4, 0, 0, "    DIM id AS INTEGER\n".to_string())]);
        assert_eq!(k[1].0, "DIM id AS INTEGER global anlegen (oben in der Datei)");
        assert_eq!(k[1].1, vec![(2, 0, 0, "DIM id AS INTEGER\n".to_string())]);
        // Auf oberster Ebene gibt es nur die eine.
        let k = korrekturen("x = 1", 0, 0, 1, "'x' wird hier beschrieben, aber nirgends im Programm mit DIM oder CONST angelegt.");
        assert_eq!(k.len(), 1);
    }

    #[test]
    fn blockende_import_und_unbenutzt() {
        // END IF hinter den Rumpf, mit der Einrueckung des Kopfes -- nicht
        // ans Dateiende hinter die Zeile, die schon wieder aussen steht.
        let t = "SUB tu()\n    IF x > 1 THEN\n        PRINT(1)\n        PRINT(2)\n    PRINT(3)\nEND SUB";
        let k = korrekturen(t, 6, 0, 1, "END IF erwartet, Programmende erreicht");
        // Dieselbe Stelle, wenn ein END SUB kam, waehrend das IF offen war.
        assert_eq!(korrekturen(t, 5, 4, 5, "END IF fehlt -- hier steht END SUB, aber das IF davor ist noch offen")[0].1, vec![(3, 16, 16, "\n    END IF".to_string())]);
        let tw = "SUB tu()\n    WHILE TRUE\n        PRINT(1)\nEND SUB";
        assert_eq!(korrekturen(tw, 3, 4, 5, "WEND (oder END WHILE) erwartet -- hier endet noch die WHILE-Schleife")[0].1,
                   vec![(2, 16, 16, "\n    WEND".to_string())]);
        assert_eq!(k[0].0, "END IF ergaenzen");
        assert_eq!(k[0].1, vec![(3, 16, 16, "\n    END IF".to_string())]);
        // Ein schon geschlossener Block zaehlt nicht.
        let t = "FOR i = 1 TO 2\nNEXT\nFOR j = 1 TO 3\n    PRINT(j)\n";
        assert_eq!(korrekturen(t, 4, 0, 1, "NEXT erwartet, Programmende erreicht")[0].1,
                   vec![(3, 12, 12, "\nNEXT".to_string())]);
        // IMPORT hinter die vorhandenen, sonst hinter den Kopfkommentar.
        let t = "' Kopf\n\nPRINT(JSON_STRINGIFY(x))";
        let k = korrekturen(t, 2, 0, 5, "JSON_STRINGIFY gehoert zum Modul 'json', aber IMPORT \"json\" fehlt.");
        assert_eq!(k[0].0, "IMPORT \"json\" einfuegen");
        assert_eq!(k[0].1, vec![(2, 0, 0, "IMPORT \"json\"\n".to_string())]);
        let t = "IMPORT \"gui\"\nDIM v AS VEC2";
        assert_eq!(korrekturen(t, 1, 0, 3, "Unbekannter Typ 'vec2' -- fehlt IMPORT \"vec2\"?")[0].1,
                   vec![(1, 0, 0, "IMPORT \"vec2\"\n".to_string())]);
        // Unbenutzt: nur in Unterprogrammen, nur bei einem Vorkommen.
        let t = "DIM g AS INTEGER\nSUB tu()\n    DIM a AS INTEGER\n    DIM b AS INTEGER ' b bleibt\n    PRINT(b)\nEND SUB";
        assert_eq!(unbenutzte(t), vec![(2, "a".to_string())]);
        let k = korrekturen(t, 2, 8, 9, "'a' wird angelegt, aber nie benutzt.");
        assert_eq!(k[0].1, vec![(2, 0, usize::MAX, String::new())]);
        // Mit einem Aufruf im Wert wird nicht angeboten, die Zeile zu loeschen.
        assert!(korrekturen("    DIM a AS INTEGER = f()", 0, 8, 9, "'a' wird angelegt, aber nie benutzt.").is_empty());
    }

    #[test]
    fn typ_raten_aus_dem_wert() {
        assert_eq!(typ_raten("n", Some("3")), "INTEGER");
        assert_eq!(typ_raten("n", Some("3.5")), "FLOAT");
        assert_eq!(typ_raten("n", Some("a / 2")), "FLOAT");
        assert_eq!(typ_raten("n", Some("TRUE")), "BOOLEAN");
        assert_eq!(typ_raten("n", Some("UPPER$(x)")), "STRING");
        assert_eq!(typ_raten("n$", None), "STRING");
        assert_eq!(typ_raten("n", Some("\"a\" + b")), "STRING");
    }

    #[test]
    fn fehler_bereich_findet_die_stelle() {
        // genannter Name, als ganzes Wort, nicht als Teil von "zaehlrx"
        assert_eq!(super::fehler_bereich("  zaehlrx = zaehlr + 1", 0, "'zaehlr' wird hier beschrieben"), (12, 18));
        // Spalte auf einem Wort: das ganze Wort
        assert_eq!(super::fehler_bereich("PRINT foo bar", 8, "Erwartet"), (6, 9));
        // Spalte am Zeilenende: das letzte Zeichen
        assert_eq!(super::fehler_bereich("PRINT (1 +", 11, "endet mitten drin"), (9, 10));
        // Spalte auf einem Satzzeichen: das Zeichen
        assert_eq!(super::fehler_bereich("x = (1 + 2", 5, "Klammer"), (4, 5));
        // nichts bekannt: die Zeile ohne Einrueckung
        assert_eq!(super::fehler_bereich("    CLS(1, 2, 3)  ", 0, "zu viele Argumente"), (4, 16));
        // leere Zeile: ein Zeichen
        assert_eq!(super::fehler_bereich("", 0, "x"), (0, 1));
    }

    use super::*;

    #[test]
    fn rahmung_hin_und_zurueck() {
        let mut puffer: Vec<u8> = Vec::new();
        nachricht_schreiben(&mut puffer, &json!({"jsonrpc": "2.0", "id": 1, "result": {"ok": true}})).unwrap();
        let mut leser = io::Cursor::new(puffer);
        let m = nachricht_lesen(&mut leser).unwrap().unwrap();
        assert_eq!(m["result"]["ok"], json!(true));
        assert!(nachricht_lesen(&mut io::Cursor::new(b"".to_vec())).unwrap().is_none());
        assert!(nachricht_lesen(&mut io::Cursor::new(b"X-Custom: 5\r\n\r\nhello".to_vec())).is_err());
    }

    #[test]
    fn uri_zu_pfad_dekodiert() {
        let p = uri_zu_pfad("file:///tmp/foo%20bar.dh").unwrap();
        assert!(p.to_string_lossy().ends_with("foo bar.dh"));
        assert!(uri_zu_pfad("untitled:1").is_none());
    }

    #[test]
    fn gliederung_verschachtelt_und_enum() {
        let src = "CLASS Player\n    SUB Init()\n    END SUB\nEND CLASS\nFUNCTION add() AS INTEGER\nEND FUNCTION\nENUM State\n  A = 0\nEND ENUM\n";
        let g = gliederung(src);
        let namen: Vec<&str> = g.as_array().unwrap().iter().map(|n| n["name"].as_str().unwrap()).collect();
        assert_eq!(namen, ["Player", "add", "State"]);
        assert_eq!(g[0]["children"][0]["name"], "Init");
        assert_eq!(g[0]["kind"], SK_CLASS);
        assert_eq!(g[2]["kind"], SK_ENUM);
        // Der Block reicht bis END ENUM (Zeile 9 = Index 8), nicht nur ueber
        // seine Kopfzeile -- sonst faltet die IDE ihn nicht, und die
        // Pfadleiste kennt ihn nur in der ersten Zeile.
        assert_eq!(g[2]["range"]["end"]["line"], 8);
    }

    #[test]
    fn hover_kennt_builtins_und_eigene() {
        let h = hover("DIM x AS INTEGER\nx = ABS(-5)\n", None, 1, 5).unwrap();
        assert!(h["contents"]["value"].as_str().unwrap().to_uppercase().contains("ABS"));
        let h = hover("DIM s AS STRING\ns = STR$(5)\n", None, 1, 6).unwrap();
        assert!(h["contents"]["value"].as_str().unwrap().contains("STR$"));
        // Nur im Index, keine Handdoku: wenigstens die Signatur.
        let h = hover("x = MODEL_TEXTURE(1, 2)\n", None, 0, 6).unwrap();
        assert!(h["contents"]["value"].as_str().unwrap().contains("MODEL_TEXTURE"));
        let src = "' Addiert.\nFUNCTION add(a AS INTEGER) AS INTEGER\nEND FUNCTION\nr = add(1)\n";
        let h = hover(src, None, 3, 5).unwrap();
        assert!(h["contents"]["value"].as_str().unwrap().contains("Addiert."));
        assert!(hover("   \n", None, 0, 1).is_none());
    }

    #[test]
    fn vervollstaendigung_filtert_und_kennt_eigene() {
        let items = vervollstaendigung("PRI", None, 0, 3);
        assert!(items.iter().all(|i| i["label"].as_str().unwrap().to_lowercase().starts_with("pri")));
        assert!(items.iter().any(|i| i["label"] == "PRINT"));
        let items = vervollstaendigung("CLASS Player\nEND CLASS\nPl", None, 2, 2);
        assert!(items.iter().any(|i| i["label"] == "Player"));
        let alle = vervollstaendigung("", None, 0, 0);
        assert!(alle.iter().any(|i| i["label"] == "KEY_SPACE"));
        assert!(alle.iter().any(|i| i["label"] == "WHILE"));
    }

    const TIERE: &str = "CLASS Tier\n    DIM name AS STRING\n    ABSTRACT FUNCTION laut(n AS INTEGER) AS STRING ' Ankuendigung\n    SUB vorstellen()\n        PRINT laut(1)\n    END SUB\nEND CLASS\nCLASS Hund EXTENDS Tier\n    FUNCTION laut(n AS INTEGER) AS STRING\n        RETURN laut(n)\n    END FUNCTION\nEND CLASS\nCLASS Katze EXTENDS Tier\nEND CLASS\nDIM t AS Tier\nt.la";

    #[test]
    fn vererbung_in_werkzeugen() {
        // F12 auf laut() in Tier: die Ankuendigung in Tier, nicht Hund.laut.
        assert_eq!(definition(TIERE, "u", 4, 15)["range"]["start"]["line"], 2);
        // In Hund: die eigene Methode.
        assert_eq!(definition(TIERE, "u", 9, 16)["range"]["start"]["line"], 8);
        // Ein Parameter der Methode gewinnt vor einem Mitglied gleichen Namens.
        let p = "CLASS A\n    DIM n AS INTEGER\n    SUB s(n AS INTEGER)\n        PRINT n\n    END SUB\nEND CLASS";
        assert_eq!(definition(p, "u", 3, 15)["range"]["start"]["line"], 2);
        // CODE_CLASSES$ kennt die ABSTRACT-Methode.
        let k = klassen_struktur(TIERE);
        assert!(k[0]["mitglieder"].as_array().unwrap().iter().any(|m| m["name"] == "laut" && m["typ"] == "STRING"));
        // Hinter einem Punkt ohne bekannte Klasse: nur Mitglieder, keine Befehle.
        let items = vervollstaendigung(TIERE, None, 15, 4);
        let namen: Vec<&str> = items.iter().map(|i| i["label"].as_str().unwrap()).collect();
        assert_eq!(namen, ["laut"]);
        // Die Gliederung zeigt sie als Kind von Tier.
        let g = gliederung(TIERE);
        assert!(g[0]["children"].as_array().unwrap().iter().any(|c| c["name"] == "laut"));
    }

    #[test]
    fn fehlende_methoden_einfuegen() {
        let m = "NEW KATZE: die Klasse kuendigt eine Methode an, ohne sie auszufuellen (laut). Entweder ...";
        let k = korrekturen(TIERE, 14, 0, 1, m);
        let e = k.iter().find(|x| x.0 == "Fehlende Methode laut einfuegen").expect("Korrektur");
        // Vor END CLASS von Katze (Zeile 14 = Index 13), Kommentar faellt weg.
        assert_eq!(e.1, vec![(13, 0, 0, "    FUNCTION laut(n AS INTEGER) AS STRING\n        \n    END FUNCTION\n".to_string())]);
        // Eine Klasse, die nicht im Text steht: keine Korrektur.
        assert!(korrekturen(TIERE, 14, 0, 1, &m.replace("KATZE", "MAUS")).iter().all(|x| !x.0.starts_with("Fehlende")));
    }

    #[test]
    fn mitglieder_hinter_dem_punkt() {
        let src = "CLASS Tier\n    ' Rufname.\n    DIM name AS STRING\nEND CLASS\n\
                   CLASS Spieler\n    ' Bestenliste.\n    DIM name AS STRING\n    DIM hp AS INTEGER\n\
                   \x20   SUB heilen(n AS INTEGER)\n        DIM name AS INTEGER\n    END SUB\nEND CLASS\n\
                   DIM leute AS ARRAY OF Spieler\nPRINT leute[0].name\nPRINT 1.5\n";
        let b = Some(Path::new("."));
        let md = |h: Option<Value>| h.map(|h| h["contents"]["value"].as_str().unwrap().to_string()).unwrap_or_default();
        // Hinter dem Punkt: das Feld von SPIELER, nicht das erste `name` im Text.
        let h = md(hover(src, b, 13, 16));
        assert!(h.contains("Feld von SPIELER") && h.contains("Bestenliste."), "{}", h);
        // Ohne basis bleibt es beim Text (die erste Definition).
        assert!(md(hover(src, None, 13, 16)).contains("Rufname."));
        // In der Klasse selbst gilt ihr Mitglied; die lokale DIM der Methode
        // zaehlt nicht als Feld.
        assert!(md(hover(src, None, 6, 9)).contains("Bestenliste."));
        // Direkt hinter dem Punkt, das Wort dahinter steht noch da.
        let namen: Vec<String> = vervollstaendigung(src, b, 13, 15).iter()
            .map(|i| i["label"].as_str().unwrap().to_string()).collect();
        assert_eq!(namen, vec!["name", "hp", "heilen"]);
        // Eine Zahl vor dem Punkt ist kein Objekt.
        assert!(mitglied_stelle(src, 14, 8, false).is_none());
    }

    #[test]
    fn diagnose_leer_bei_sauber_und_fehler_mit_zeile() {
        let d = diagnose("PRINT 1\n", Path::new("."));
        assert!(d.is_empty(), "{:?}", d);
        let d = diagnose("PRINT 1\nDIM x AS\n", Path::new("."));
        assert_eq!(d.len(), 1);
        assert_eq!(d[0]["severity"], 1);
        assert_eq!(d[0]["range"]["start"]["line"], 1);
    }

    #[test]
    fn server_antwortet_und_ignoriert_fremdes() {
        let puffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        struct Schreiber(Arc<Mutex<Vec<u8>>>);
        impl Write for Schreiber {
            fn write(&mut self, b: &[u8]) -> io::Result<usize> { self.0.lock().unwrap().extend_from_slice(b); Ok(b.len()) }
            fn flush(&mut self) -> io::Result<()> { Ok(()) }
        }
        let mut s = Server::neu(Box::new(Schreiber(Arc::clone(&puffer))));
        s.verzoegerung_ms = 0;
        s.bearbeiten(&json!([]));
        s.bearbeiten(&json!("nein"));
        s.bearbeiten(&json!({"jsonrpc": "2.0", "id": 7, "method": "textDocument/foobar", "params": {}}));
        s.bearbeiten(&json!({"jsonrpc": "2.0", "id": 8, "method": "shutdown"}));
        assert!(s.beendet);
        let text = String::from_utf8(puffer.lock().unwrap().clone()).unwrap();
        assert!(text.contains("\"id\":7,\"result\":null"), "{}", text);
        assert!(text.contains("\"id\":8"));
    }
}
