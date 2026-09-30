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
    let mut aus = diagnose_uebersetzer(text, basis);
    let zeilen: Vec<&str> = text.split('\n').collect();
    for (z0, name) in unbenutzte(text) {
        let meldung = format!("'{}' wird angelegt, aber nie benutzt.", name);
        let (von, bis) = fehler_bereich(zeilen.get(z0).copied().unwrap_or(""), 0, &meldung);
        let mut d = json!({
            "range": {"start": {"line": z0, "character": von}, "end": {"line": z0, "character": bis}},
            "severity": 4, "source": "drachenhauch", "message": meldung, "tags": [1],
        });
        let k = korrekturen(text, z0, von, bis, &meldung);
        if !k.is_empty() {
            d["data"] = json!({"korrekturen": k.into_iter().map(|(titel, aend)| json!({
                "titel": titel,
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
    let roh = crate::compiler::typ_hinweise_sammeln(|| { let _ = crate::check_source(text, basis, "<editor>"); });
    let herkunft = crate::preprocess::process(text, basis).ok().map(|r| r.2);
    let mut aus: Vec<(u32, String, String)> = Vec::new();
    for (z, name, typ) in roh {
        let zeile = match herkunft.as_ref().and_then(|h| h.get((z as usize).saturating_sub(1))) {
            Some(h) if h.datei.is_empty() => h.zeile,
            Some(_) => continue,
            None => z,
        };
        if !aus.iter().any(|(a, b, _)| *a == zeile && b.eq_ignore_ascii_case(&name)) {
            aus.push((zeile, name, typ));
        }
    }
    aus.sort_by_key(|e| e.0);
    aus
}

fn diagnose_uebersetzer(text: &str, basis: &Path) -> Vec<Value> {
    let roh = crate::check_source(text, basis, "<editor>");
    let herkunft = crate::preprocess::process(text, basis).ok().map(|r| r.2);
    let zeilen: Vec<&str> = text.split('\n').collect();
    roh.into_iter().map(|d| {
        let phase = d.get("phase").and_then(|p| p.as_str()).unwrap_or("compile").to_string();
        let mut zeile = d.get("line").and_then(|l| l.as_u64()).unwrap_or(0).max(1) as usize;
        let mut meldung = d.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
        if matches!(phase.as_str(), "lex" | "parse" | "compile" | "namensraum") {
            if let Some(h) = herkunft.as_ref().and_then(|h| h.get(zeile - 1)) {
                if h.datei.is_empty() { zeile = h.zeile as usize; }
                else { meldung = format!("in {}:{} -> {}", h.datei, h.zeile, meldung); zeile = 1; }
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
            "severity": schwere, "source": "drachenhauch", "message": meldung,
        });
        let k = if importiert { Vec::new() } else { korrekturen(text, z0, von, bis, &meldung) };
        if !k.is_empty() {
            let liste: Vec<Value> = k.into_iter().map(|(titel, aend)| json!({
                "titel": titel,
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
pub fn korrekturen(text: &str, z0: usize, von: usize, bis: usize, meldung: &str) -> Vec<(String, Vec<Aenderung>)> {
    let zeilen: Vec<&str> = text.split('\n').collect();
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
                aus.push((format!("Ersetzen durch {}", v), vec![(z0, von, bis, v.to_string())]));
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
    // Drei Formen: am Programmende, mitten in einem anderen Block ("Erwartet
    // IF nach END" -- ein END SUB kam, waehrend das IF noch offen war) und
    // die WHILE-Schleife mit eigenem Satz.
    let schluss: Option<String> = if let Some(p) = meldung.find(" erwartet, Programmende erreicht") {
        Some(meldung[..p].trim().to_string())
    } else if let Some(rest) = meldung.strip_prefix("Erwartet ") {
        rest.strip_suffix(" nach END").map(|w| format!("END {}", w.trim()))
    } else if meldung.starts_with("WEND (oder END WHILE) erwartet") {
        Some("WEND".to_string())
    } else { None };
    if let Some(schluss) = schluss.as_deref() {
        if let Some((ende, ein)) = blockende_stelle(&zeilen, schluss) {
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
                let mut ziel = 0;
                let mut hinter_import = None;
                for (k, t) in zeilen.iter().enumerate() {
                    let r = t.trim().to_uppercase();
                    if r.starts_with("IMPORT ") { hinter_import = Some(k + 1); continue; }
                    if hinter_import.is_none() && (r.is_empty() || r.starts_with('\'') || r.starts_with("REM ")) { ziel = k + 1; continue; }
                    break;
                }
                let ziel = hinter_import.unwrap_or(ziel).min(zeilen.len());
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
        let bereiche = crate::symbole::bereiche(text);
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
    }
    aus
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

pub fn vervollstaendigung(text: &str, z0: usize, c0: usize) -> Vec<Value> {
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

pub fn hover(text: &str, z0: usize, c0: usize) -> Option<Value> {
    let (wort, _, _) = symbole::wort_bei(text, z0, c0);
    if wort.is_empty() { return None; }
    let (sig, doc) = builtin_doku(&wort).or_else(|| symbole::nutzer_doku(text, &wort))?;
    let mut md = format!("```drachenhauch\n{}\n```", sig);
    if !doc.is_empty() { md.push_str("\n\n"); md.push_str(&doc); }
    Some(json!({"contents": {"kind": "markdown", "value": md}}))
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
pub fn umbenennen(text: &str, z0: usize, c0: usize, neu: &str) -> Option<String> {
    let (wort, _, _) = symbole::wort_bei(text, z0, c0);
    if wort.is_empty() { return None; }
    // Ein Schluesselwort umzubenennen macht aus dem Programm Buchstabensalat,
    // und die Marke steht schnell einmal auf einem: nichts tun, statt es zu
    // tun. Builtins bleiben erlaubt -- eine eigene Variable darf seit
    // 2026-09-04 heissen wie eines, und die will man umbenennen koennen.
    if crate::lexer::keyword(&wort.to_lowercase()).is_some() { return None; }
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
    fn knoten(name: &str, art: i64, von: usize, bis: usize) -> Value {
        let r = json!({"start": {"line": von.saturating_sub(1), "character": 0},
                       "end": {"line": bis.saturating_sub(1), "character": 0}});
        json!({"name": name, "kind": art, "range": r, "selectionRange": r, "children": []})
    }
    let bereiche = symbole::bereiche(text);
    let mut wurzeln: Vec<Value> = Vec::new();
    // Stapel aus (Bereich, Pfad zum Knoten in `wurzeln`).
    let mut stapel: Vec<(symbole::Bereich, Vec<usize>)> = Vec::new();
    for b in bereiche {
        let art = match b.art { "class" => SK_CLASS, "struct" => SK_STRUCT, "property" => SK_PROPERTY, "enum" => SK_ENUM, _ => SK_FUNCTION };
        let k = knoten(&b.name, art, b.zeile, b.ende);
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
                    "textDocument/completion" => Value::Array(vervollstaendigung(&text, z0, c0)),
                    "textDocument/hover" => hover(&text, z0, c0).unwrap_or(Value::Null),
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
    fn blockende_import_und_unbenutzt() {
        // END IF hinter den Rumpf, mit der Einrueckung des Kopfes -- nicht
        // ans Dateiende hinter die Zeile, die schon wieder aussen steht.
        let t = "SUB tu()\n    IF x > 1 THEN\n        PRINT(1)\n        PRINT(2)\n    PRINT(3)\nEND SUB";
        let k = korrekturen(t, 6, 0, 1, "END IF erwartet, Programmende erreicht");
        // Dieselbe Stelle, wenn ein END SUB kam, waehrend das IF offen war.
        assert_eq!(korrekturen(t, 5, 4, 5, "Erwartet IF nach END")[0].1, vec![(3, 16, 16, "\n    END IF".to_string())]);
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
        let h = hover("DIM x AS INTEGER\nx = ABS(-5)\n", 1, 5).unwrap();
        assert!(h["contents"]["value"].as_str().unwrap().to_uppercase().contains("ABS"));
        let h = hover("DIM s AS STRING\ns = STR$(5)\n", 1, 6).unwrap();
        assert!(h["contents"]["value"].as_str().unwrap().contains("STR$"));
        // Nur im Index, keine Handdoku: wenigstens die Signatur.
        let h = hover("x = MODEL_TEXTURE(1, 2)\n", 0, 6).unwrap();
        assert!(h["contents"]["value"].as_str().unwrap().contains("MODEL_TEXTURE"));
        let src = "' Addiert.\nFUNCTION add(a AS INTEGER) AS INTEGER\nEND FUNCTION\nr = add(1)\n";
        let h = hover(src, 3, 5).unwrap();
        assert!(h["contents"]["value"].as_str().unwrap().contains("Addiert."));
        assert!(hover("   \n", 0, 1).is_none());
    }

    #[test]
    fn vervollstaendigung_filtert_und_kennt_eigene() {
        let items = vervollstaendigung("PRI", 0, 3);
        assert!(items.iter().all(|i| i["label"].as_str().unwrap().to_lowercase().starts_with("pri")));
        assert!(items.iter().any(|i| i["label"] == "PRINT"));
        let items = vervollstaendigung("CLASS Player\nEND CLASS\nPl", 2, 2);
        assert!(items.iter().any(|i| i["label"] == "Player"));
        let alle = vervollstaendigung("", 0, 0);
        assert!(alle.iter().any(|i| i["label"] == "KEY_SPACE"));
        assert!(alle.iter().any(|i| i["label"] == "WHILE"));
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
