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
//! documentSymbol. Voll-Sync. Alles Unbekannte mit `id` bekommt `null`.
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
        let ende = zeilen.get(z0).map(|l| l.chars().count()).unwrap_or(0).max(1);
        let schwere = if d.get("severity").and_then(|s| s.as_str()) == Some("warning") { 2 } else { 1 };
        json!({
            "range": {"start": {"line": z0, "character": 0}, "end": {"line": z0, "character": ende}},
            "severity": schwere, "source": "drachenhauch", "message": meldung,
        })
    }).collect()
}

// ---------------------------------------------------------------- Hover-Daten

/// Handgepflegte Hover-Texte (`builtin_docs.json`, Name klein -> [Signatur, Text]).
fn handdoku() -> &'static HashMap<String, (String, String)> {
    static M: std::sync::OnceLock<HashMap<String, (String, String)>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        let mut m = HashMap::new();
        let raw = include_str!("../../../drachenhauch/editor_qt/builtin_docs.json");
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
        let raw = include_str!("../../../drachenhauch/editor_qt/builtin_prosa.json");
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
        let art = match b.art { "class" => SK_CLASS, "struct" => SK_STRUCT, "property" => SK_PROPERTY, _ => SK_FUNCTION };
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
    for d in symbole::definitionen(text) {
        if d.art == "enum" { wurzeln.push(knoten(&d.name, SK_ENUM, d.zeile, d.zeile)); }
    }
    wurzeln.sort_by_key(|n| n["range"]["start"]["line"].as_u64().unwrap_or(0));
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
