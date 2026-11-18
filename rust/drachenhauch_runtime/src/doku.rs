//! `dhrt doku` -- die Doku-Werkzeuge, die bis 2026-09-06 in Python lagen
//! (Weg A aus `docs/intern/entwurf-python-abbau.md`):
//!
//!   dhrt doku prosa [--pruefen]          builtin_prosa.json aus docs/ (+ Referenzbuch)
//!   dhrt doku grammatik [--pruefen]      VS-Code-Grammatik aus Keywords + Index
//!   dhrt doku referenz <datei.dh ...> [-o ziel.md]   Referenz aus dem Quelltext
//!
//! `prosa` ersetzt `tools/gen_builtin_prosa.py`, `grammatik` ersetzt
//! `vscode-drachenhauch/build_grammar.py`, `referenz` ersetzt
//! `drachenhauch/doku.py` (das frueher NICHT in dhrt lag, "weil der Rust-Lexer
//! Kommentare wegwirft" -- `symbole.rs` liest den Text, nicht die Token, und
//! hat die Kommentare).
//!
//! `--pruefen` schreibt nichts und meldet mit Rueckgabe 1, wenn die
//! eingecheckte Datei nicht mehr zum Stand passt -- so haengen die
//! erzeugten Dateien an der Testsuite.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::OnceLock;

use regex::Regex;
use serde_json::{json, Value};

use crate::symbole;

fn re(q: &'static str, zelle: &'static OnceLock<Regex>) -> &'static Regex {
    zelle.get_or_init(|| Regex::new(q).expect("Regex im Quelltext"))
}

/// Das Repo: von hier aufwaerts der erste Ordner mit `docs/` und dem Index.
pub fn repo_wurzel() -> Result<PathBuf, String> {
    let start = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut p: Option<&Path> = Some(&start);
    while let Some(d) = p {
        if d.join("docs").is_dir() && d.join("daten/builtin_index.json").is_file() {
            return Ok(d.to_path_buf());
        }
        p = d.parent();
    }
    Err("dhrt doku: kein Drachenhauch-Repo ueber dem aktuellen Verzeichnis (docs/ und \
         daten/builtin_index.json fehlen)".into())
}

/// Index von der Platte (nicht der eingebettete): wer den Index aendert,
/// soll die Prosa neu ziehen koennen, ohne dhrt vorher neu zu bauen.
fn index_namen(wurzel: &Path) -> Result<HashSet<String>, String> {
    let raw = std::fs::read_to_string(wurzel.join("daten/builtin_index.json"))
        .map_err(|e| format!("builtin_index.json: {}", e))?;
    let v: Value = serde_json::from_str(&raw).map_err(|e| format!("builtin_index.json: {}", e))?;
    Ok(v["builtins"].as_array().map(|a| a.iter()
        .filter_map(|e| e["name"].as_str().map(|n| n.to_uppercase())).collect()).unwrap_or_default())
}

fn json_datei(text: &str) -> String {
    // Ein Leerzeichen Einzug, wie Pythons `json.dumps(indent=1)` -- so bleibt
    // der Diff zur bisher erzeugten Datei klein.
    text.to_string()
}

fn eins_eingerueckt(v: &Value) -> String {
    let mut puffer = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(&mut puffer, fmt);
    use serde::Serialize;
    v.serialize(&mut ser).expect("JSON");
    let mut s = String::from_utf8(puffer).expect("UTF-8");
    s.push('\n');
    json_datei(&s)
}

/// Schreiben oder pruefen -- der gemeinsame Schluss beider Generatoren.
fn abliefern(ziel: &Path, text: &str, pruefen: bool, was: &str, tipp: &str) -> ExitCode {
    if pruefen {
        // Zeilenenden nicht mitvergleichen: auf dem Windows-Laeufer der CI
        // checkt git mit CRLF aus, erzeugt wird mit LF -- byteweise war die
        // eingecheckte Grammatik dort immer "veraltet".
        let alt = std::fs::read_to_string(ziel).unwrap_or_default().replace("\r\n", "\n");
        if alt == text.replace("\r\n", "\n") {
            println!("{} ist aktuell.", ziel.display());
            return ExitCode::SUCCESS;
        }
        println!("{} weicht vom Stand ab -- `{}` laufen lassen.", ziel.display(), tipp);
        return ExitCode::from(1);
    }
    if let Some(d) = ziel.parent() { let _ = std::fs::create_dir_all(d); }
    match std::fs::write(ziel, text) {
        Ok(()) => { println!("{} -> {}", was, ziel.display()); ExitCode::SUCCESS }
        Err(e) => { eprintln!("{}: {}", ziel.display(), e); ExitCode::from(1) }
    }
}

// ============================================================ prosa

const MAX_LAENGE: usize = 400;

/// Planungsdokumente beschreiben, was sein SOLL -- ein Hover, der einen
/// Entwurf zitiert, waere schlimmer als keiner.
fn quellen(docs: &Path) -> Vec<PathBuf> {
    static AUS: OnceLock<Regex> = OnceLock::new();
    let aus = re(r"^(entwurf-|allzweck-)|roadmap|-design\.md$|^PERFORMANCE\.md$", &AUS);
    let mut alle: Vec<PathBuf> = std::fs::read_dir(docs).map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |x| x == "md")).collect()).unwrap_or_default();
    alle.sort();
    alle.retain(|p| !aus.is_match(&p.file_name().unwrap().to_string_lossy()));
    let (vorn, rest): (Vec<PathBuf>, Vec<PathBuf>) = alle.into_iter().partition(|p| {
        let n = p.file_name().unwrap().to_string_lossy().to_string();
        n.starts_with("module-") || n.starts_with("builtins-")
    });
    vorn.into_iter().chain(rest).collect()
}

/// Zusammengezogene Schreibweisen (`SCENE_SET_INT/FLOAT/STRING/BOOL(...)`)
/// aufloesen -- nur, was danach WIRKLICH ein Builtin ist.
fn kurzformen(kopf: &str, namen: &HashSet<String>) -> Vec<String> {
    static G: OnceLock<Regex> = OnceLock::new();
    let gruppe = re(r"`\s*([A-Z][A-Z0-9_]*(?:/[A-Z][A-Z0-9_]*)+)", &G);
    let mut raus = Vec::new();
    for c in gruppe.captures_iter(kopf) {
        let teile: Vec<&str> = c[1].split('/').collect();
        let basis = teile[0];
        if !basis.contains('_') { continue; }
        let stuecke: Vec<&str> = basis.split('_').collect();
        for weiterer in &teile[1..] {
            for schnitt in (1..stuecke.len()).rev() {
                let kandidat = format!("{}_{}", stuecke[..schnitt].join("_"), weiterer);
                if namen.contains(&kandidat) || namen.contains(kandidat.trim_end_matches('$')) {
                    raus.push(kandidat);
                    break;
                }
            }
        }
    }
    raus
}

fn saeubern(roh: &str) -> String {
    static LINK: OnceLock<Regex> = OnceLock::new();
    static CODE: OnceLock<Regex> = OnceLock::new();
    static FETT: OnceLock<Regex> = OnceLock::new();
    static LEER: OnceLock<Regex> = OnceLock::new();
    let s = roh.trim().trim_matches('|').trim();
    let mut spalten: Vec<String> = s.split('|').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
    // Eine Spalte aus lauter Strichen ist ein LEERES Feld.
    spalten.retain(|t| !t.trim_matches(|c| c == '—' || c == '–' || c == '-' || c == ' ').is_empty());
    let mut s = if spalten.len() > 1 && spalten[0].chars().count() <= 12 {
        format!("{} — {}", spalten[0], spalten[1..].join(" "))
    } else { spalten.join(" ") };
    s = re(r"\[([^\]]+)\]\([^)]*\)", &LINK).replace_all(&s, "$1").into_owned();
    s = re(r"`([^`]*)`", &CODE).replace_all(&s, "$1").into_owned();
    s = re(r"\*\*([^*]*)\*\*", &FETT).replace_all(&s, "$1").into_owned();
    s = re(r"\s+", &LEER).replace_all(&s, " ").trim().to_string();
    s = s.trim_end_matches(':').trim_end().to_string();
    if s.chars().count() > MAX_LAENGE {
        let z: Vec<char> = s.chars().collect();
        let kopf: String = z[..MAX_LAENGE].iter().collect();
        let schnitt = kopf.rfind(' ').unwrap_or(kopf.len());
        s = format!("{} …", kopf[..schnitt].trim_end_matches(|c| c == ' ' || c == ',' || c == ';' || c == '-'));
    }
    s
}

/// Zeilen der Datei, umgebrochene Listeneintraege zu EINER Zeile
/// zusammengezogen -- sonst stuende im Hover ein halber Satz.
fn zeilen(datei: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(datei).unwrap_or_default();
    let mut raus: Vec<String> = Vec::new();
    for zeile in text.lines() {
        let rumpf = zeile.trim_start();
        let neuer_punkt = rumpf.starts_with("- ") || rumpf.starts_with("* ")
            || rumpf.starts_with('|') || rumpf.starts_with('#') || rumpf.starts_with("```");
        let fortsetzung = raus.last().map_or(false, |l| { let t = l.trim_start(); t.starts_with("- ") || t.starts_with("* ") })
            && (zeile.starts_with(' ') || zeile.starts_with('\t')) && !rumpf.is_empty() && !neuer_punkt;
        if fortsetzung {
            let letzte = raus.last_mut().unwrap();
            *letzte = format!("{} {}", letzte.trim_end(), zeile.trim());
        } else {
            raus.push(zeile.to_string());
        }
    }
    raus
}

fn ist_builtin(name: &str, namen: &HashSet<String>) -> bool {
    namen.contains(name) || namen.contains(name.trim_end_matches('$'))
        || namen.iter().any(|n| n.trim_end_matches('$') == name)
}

/// `en` = die englische Fassung: Tabellen aus docs/en und die Befehlstexte
/// des englischen Buchs (fuer den Hover mit `DHRT_LANG=en`).
pub fn prosa_sammeln(wurzel: &Path, en: bool) -> Result<BTreeMap<String, String>, String> {
    static TABELLE: OnceLock<Regex> = OnceLock::new();
    static TABELLE_2: OnceLock<Regex> = OnceLock::new();
    static LISTE: OnceLock<Regex> = OnceLock::new();
    static KOPF_NAME: OnceLock<Regex> = OnceLock::new();
    let tabelle = re(r"^\|([^|]*)\|(.+)$", &TABELLE);
    let tabelle_2 = re(r"^\|[^|]*\|([^|]*)\|(.+)$", &TABELLE_2);
    let liste = re(r"^\s*[-*]\s+((?:`[^`]+`[\s/,]*)+)\s*[—–:→]\s*(.+)$", &LISTE);
    let kopf_name = re(r"`\s*([A-Z][A-Z0-9_]*\$?)[^`]*`", &KOPF_NAME);
    let namen = index_namen(wurzel)?;
    let mut raus: BTreeMap<String, String> = BTreeMap::new();
    let docs = if en { wurzel.join("docs/en") } else { wurzel.join("docs") };
    for datei in quellen(&docs) {
        for zeile in zeilen(&datei) {
            let mut m = tabelle.captures(&zeile).or_else(|| liste.captures(&zeile));
            if let Some(c) = &m {
                if !kopf_name.is_match(&c[1]) {
                    if let Some(c2) = tabelle_2.captures(&zeile) { m = Some(c2); }
                }
            }
            let Some(c) = m else { continue };
            let mut treffer: Vec<String> = kopf_name.captures_iter(&c[1]).map(|k| k[1].to_uppercase()).collect();
            treffer.extend(kurzformen(&c[1], &namen));
            if treffer.is_empty() { continue; }
            let text = saeubern(&c[2]);
            if text.chars().count() < 8 { continue; }
            for name in treffer {
                if !ist_builtin(&name, &namen) { continue; }
                raus.entry(name).or_insert_with(|| text.clone());
            }
        }
    }
    for (name, text) in aus_dem_referenzbuch(wurzel, &namen, en) {
        raus.entry(name).or_insert(text);
    }
    Ok(raus)
}

/// Kurzbeschreibungen aus dem Referenzbuch -- nur Eintraege, die GENAU EINEN
/// Builtin nennen (Sammel-Eintraege beschreiben die Gruppe oder den falschen).
/// Ohne Node bleibt es bei `docs/`.
fn aus_dem_referenzbuch(wurzel: &Path, namen: &HashSet<String>, en: bool) -> BTreeMap<String, String> {
    static NAME: OnceLock<Regex> = OnceLock::new();
    let name_re = re(r"[A-Z][A-Z0-9_]*\$?", &NAME);
    let exporter = wurzel.join("tools/buch_cmd_export.js");
    let mut raus = BTreeMap::new();
    if !exporter.is_file() { return raus; }
    let mut befehl = std::process::Command::new("node");
    befehl.arg(&exporter).current_dir(wurzel);
    if en { befehl.arg("--en"); }
    let Ok(out) = befehl.output() else { return raus };
    let Ok(eintraege) = serde_json::from_slice::<Vec<(String, Value)>>(&out.stdout) else { return raus };
    for (name, text) in eintraege {
        let Some(text) = text.as_str() else { continue };
        let treffer: Vec<String> = name_re.find_iter(&name).map(|m| m.as_str().to_uppercase())
            .filter(|n| ist_builtin(n, namen)).collect();
        if treffer.len() != 1 { continue; }
        let satz = erster_satz(text.trim());
        let satz = saeubern(satz);
        if satz.chars().count() >= 8 { raus.entry(treffer[0].clone()).or_insert(satz); }
    }
    raus
}

/// Bis zum ersten Satzende, dem ein Leerzeichen folgt.
fn erster_satz(text: &str) -> &str {
    let b = text.as_bytes();
    for i in 0..b.len().saturating_sub(1) {
        if matches!(b[i], b'.' | b'!' | b'?') && b[i + 1].is_ascii_whitespace() {
            return &text[..=i];
        }
    }
    text
}

pub fn node_da() -> bool {
    std::process::Command::new("node").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn prosa_main(pruefen: bool) -> ExitCode {
    let wurzel = match repo_wurzel() { Ok(w) => w, Err(e) => { eprintln!("{}", e); return ExitCode::from(2); } };
    let daten = match prosa_sammeln(&wurzel, false) { Ok(d) => d, Err(e) => { eprintln!("{}", e); return ExitCode::from(2); } };
    let kopf = "Erzeugt aus docs/ von `dhrt doku prosa` -- NICHT von Hand aendern. Ausfuehrlichere \
                Texte gehoeren in builtin_docs.json (die gewinnen), Korrekturen an einer \
                Beschreibung in das jeweilige docs/module-*.md.";
    let text = eins_eingerueckt(&json!({"_comment": kopf, "count": daten.len(), "docs": daten}));
    let ziel = wurzel.join("daten/builtin_prosa.json");
    // Dieselbe Sammlung auf Englisch: docs/en und das englische Buch.
    let daten_en = match prosa_sammeln(&wurzel, true) { Ok(d) => d, Err(e) => { eprintln!("{}", e); return ExitCode::from(2); } };
    let kopf_en = "Erzeugt aus docs/en und dem englischen Buch von `dhrt doku prosa` -- NICHT von Hand \
                   aendern. Der Hover nimmt sie mit DHRT_LANG=en.";
    let text_en = eins_eingerueckt(&json!({"_comment": kopf_en, "count": daten_en.len(), "docs": daten_en}));
    let ziel_en = wurzel.join("daten/builtin_prosa.en.json");
    if pruefen && !node_da() {
        // Ohne Node fehlt eine der Quellen -- ein Vergleich meldete dann
        // Abweichungen, die nur an der Umgebung liegen.
        println!("Node fehlt -- Pruefung uebersprungen (buecher/referenz nicht lesbar).");
        return ExitCode::SUCCESS;
    }
    let code = abliefern(&ziel, &text, pruefen, &format!("{} Beschreibungen", daten.len()), "dhrt doku prosa");
    let code_en = abliefern(&ziel_en, &text_en, pruefen, &format!("{} englische Beschreibungen", daten_en.len()), "dhrt doku prosa");
    let code = if code_en != ExitCode::SUCCESS { code_en } else { code };
    if !pruefen && !node_da() { println!("  Hinweis: ohne Node -- die Eintraege aus buecher/referenz fehlen."); }
    code
}

// ============================================================ grammatik

const TYPEN: &[&str] = &["INTEGER", "FLOAT", "STRING", "BOOLEAN", "ARRAY", "MAP", "TUPLE",
                          "FILE", "IMAGE", "SOUND", "FUNCREF", "COROUTINE", "OF"];
const BOOLS: &[&str] = &["TRUE", "FALSE"];
const DEKLARATION: &[&str] = &["DIM", "CONST", "SUB", "FUNCTION", "CLASS", "STRUCT", "ENUM",
                                "PROPERTY", "STATIC", "OPERATOR", "EXTENDS", "BYREF", "AS", "IMPORT"];

/// Alternativen fuer eine Regex: laengste zuerst, gleich lange alphabetisch
/// -- so ist die Ausgabe deterministisch und ein Diff zeigt nur Aenderungen.
fn alternativen(namen: &[String]) -> String {
    let mut v: Vec<&String> = namen.iter().collect::<HashSet<_>>().into_iter().collect();
    v.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    v.iter().map(|n| regex::escape(n)).collect::<Vec<_>>().join("|")
}

pub fn grammatik(wurzel: &Path) -> Result<Value, String> {
    let kw_alle: Vec<String> = { let mut v: Vec<String> = crate::lexer::KEYWORDS.iter().map(|k| k.to_uppercase()).collect(); v.sort(); v.dedup(); v };
    let skip: HashSet<&str> = TYPEN.iter().chain(BOOLS).copied().collect();
    let control: Vec<String> = kw_alle.iter().filter(|k| !skip.contains(k.as_str()) && !DEKLARATION.contains(&k.as_str())).cloned().collect();
    let decl: Vec<String> = kw_alle.iter().filter(|k| DEKLARATION.contains(&k.as_str())).cloned().collect();
    // Builtins: Index von der Platte plus die Handdoku (sie kannte auch
    // Namen, die frueher nur dort standen).
    let mut builtins: HashSet<String> = index_namen(wurzel)?;
    if let Ok(raw) = std::fs::read_to_string(wurzel.join("daten/builtin_docs.json")) {
        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
            if let Some(o) = v["docs"].as_object() { for k in o.keys() { builtins.insert(k.to_uppercase()); } }
        }
    }
    let mut builtins: Vec<String> = builtins.into_iter().collect();
    builtins.sort();
    let plain: Vec<String> = builtins.iter().filter(|b| !b.ends_with('$')).cloned().collect();
    let dollar: Vec<String> = builtins.iter().filter(|b| b.ends_with('$')).map(|b| b.trim_end_matches('$').to_string()).collect();
    let mut consts: Vec<String> = vec!["PI".into()];
    consts.extend(crate::vm::DEFAULT_COLORS.iter().map(|(n, _)| n.to_uppercase()));
    consts.extend(crate::vm::DEFAULT_KEYS.iter().map(|(n, _)| n.to_uppercase()));
    let consts: Vec<String> = consts.into_iter().filter(|c| !BOOLS.contains(&c.as_str())).collect();
    let wort = |a: &[String]| format!("(?i)\\b({})\\b", alternativen(a));
    let bools: Vec<String> = BOOLS.iter().map(|s| s.to_string()).collect();
    let typen: Vec<String> = TYPEN.iter().map(|s| s.to_string()).collect();
    let patterns = json!([
        {"name": "comment.line.apostrophe.drachenhauch", "match": "'.*$"},
        {"name": "comment.line.rem.drachenhauch", "match": "(?i)\\bREM\\b.*$"},
        // `end` faengt auch am Zeilenende: der Lexer verbietet Umbrueche im
        // Text, eine halb getippte Zeichenkette faerbte sonst alles bis zum
        // naechsten `"` irgendwo spaeter im Dokument.
        // `!"..."` (auch `f!"..."`) traegt Escape-Folgen -- erst dieses Muster,
        // sonst faengt das schlichte `"` die Kette ohne ihr `!`.
        {"name": "string.quoted.double.escaped.drachenhauch", "begin": "!\"", "end": "\"|$",
         "patterns": [{"name": "constant.character.escape.drachenhauch", "match": "\"\"|\\\\."}]},
        {"name": "string.quoted.double.drachenhauch", "begin": "\"", "end": "\"|$",
         "patterns": [{"name": "constant.character.escape.drachenhauch", "match": "\"\""}]},
        {"name": "constant.numeric.hex.drachenhauch", "match": "\\b0[xX][0-9a-fA-F]+\\b|&[Hh][0-9a-fA-F]+"},
        {"name": "constant.numeric.binary.drachenhauch", "match": "\\b0[bB][01]+\\b|&[Bb][01]+"},
        {"name": "constant.numeric.drachenhauch", "match": "\\b[0-9]+(\\.[0-9]+)?([eE][+-]?[0-9]+)?\\b"},
        {"name": "constant.language.boolean.drachenhauch", "match": wort(&bools)},
        {"name": "storage.type.drachenhauch", "match": wort(&typen)},
        {"name": "keyword.declaration.drachenhauch", "match": wort(&decl)},
        {"name": "keyword.control.drachenhauch", "match": wort(&control)},
        {"name": "support.function.dollar.drachenhauch", "match": format!("(?i)\\b({})\\$", alternativen(&dollar))},
        {"name": "support.function.drachenhauch", "match": wort(&plain)},
        {"name": "constant.language.drachenhauch", "match": wort(&consts)},
        {"name": "keyword.operator.drachenhauch", "match": "<=|>=|<>|[-+*/\\\\=<>^]"},
    ]);
    Ok(json!({
        "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
        "name": "Drachenhauch",
        "scopeName": "source.drachenhauch",
        "patterns": [{"include": "#root"}],
        "repository": {"root": {"patterns": patterns}},
    }))
}

fn grammatik_main(pruefen: bool) -> ExitCode {
    let wurzel = match repo_wurzel() { Ok(w) => w, Err(e) => { eprintln!("{}", e); return ExitCode::from(2); } };
    let g = match grammatik(&wurzel) { Ok(g) => g, Err(e) => { eprintln!("{}", e); return ExitCode::from(2); } };
    let mut text = serde_json::to_string_pretty(&g).unwrap_or_default();
    text.push('\n');
    let ziel = wurzel.join("vscode-drachenhauch/syntaxes/drachenhauch.tmLanguage.json");
    abliefern(&ziel, &text, pruefen, "Grammatik", "dhrt doku grammatik")
}

// ============================================================ referenz

/// Was in eine Referenz gehoert, und in welcher Reihenfolge. `param` und
/// `dim` fehlen mit Absicht: das sind Innereien.
const ARTEN: &[(&str, &str)] = &[
    ("const", "Konstanten"), ("enum", "Aufzaehlungen"), ("struct", "Strukturen"),
    ("class", "Klassen"), ("function", "Funktionen"), ("sub", "Prozeduren"),
];

fn ist_privat(zeilen: &[&str], zeile: usize) -> bool {
    zeile >= 1 && zeile <= zeilen.len() && zeilen[zeile - 1].trim_start().len() >= 7
        && zeilen[zeile - 1].trim_start()[..7].eq_ignore_ascii_case("private")
}

/// Der Kommentarblock am Dateianfang -- die Beschreibung der Datei.
fn kopfkommentar(zeilen: &[&str]) -> String {
    let mut raus: Vec<String> = Vec::new();
    for z in zeilen {
        let s = z.trim();
        if s.is_empty() { if raus.is_empty() { continue; } break; }
        if let Some(r) = s.strip_prefix('\'') { raus.push(r.trim_start_matches('\'').trim().to_string()); }
        else if s.len() >= 4 && s[..4].eq_ignore_ascii_case("rem ") { raus.push(s[4..].trim().to_string()); }
        else { break; }
    }
    raus.join("\n").trim().to_string()
}

/// Die Referenz EINER Datei als Markdown-Abschnitt.
pub fn datei_referenz(pfad: &Path) -> Result<String, String> {
    let quelle = std::fs::read_to_string(pfad).map_err(|e| format!("{}: {}", pfad.display(), e))?;
    let zeilen: Vec<&str> = quelle.split('\n').collect();
    let mut gesehen: HashSet<(String, String)> = HashSet::new();
    let mut nach_art: BTreeMap<&str, Vec<(String, String, String)>> = BTreeMap::new();
    for d in symbole::definitionen(&quelle) {
        if !ARTEN.iter().any(|(a, _)| *a == d.art) { continue; }
        if ist_privat(&zeilen, d.zeile) { continue; }
        if !gesehen.insert((d.art.to_string(), d.name.to_lowercase())) { continue; }
        let Some((sig, doc)) = symbole::nutzer_doku(&quelle, &d.name) else { continue };
        nach_art.entry(d.art).or_default().push((d.name.clone(), sig, doc));
    }
    let name = pfad.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut teile: Vec<String> = vec![format!("## {}\n", name)];
    let kopf = kopfkommentar(&zeilen);
    if !kopf.is_empty() { teile.push(format!("{}\n", kopf)); }
    let mut leer = true;
    for (art, ueberschrift) in ARTEN {
        let Some(eintraege) = nach_art.get(art) else { continue };
        if eintraege.is_empty() { continue; }
        leer = false;
        teile.push(format!("### {}\n", ueberschrift));
        for (name, sig, doc) in eintraege {
            teile.push(format!("#### `{}`\n", name));
            teile.push(format!("```basic\n{}\n```\n", sig));
            teile.push(format!("{}\n", if doc.trim().is_empty() { "*(nicht beschrieben)*" } else { doc.trim() }));
        }
    }
    if leer { teile.push("*(nichts Oeffentliches gefunden)*\n".into()); }
    Ok(teile.join("\n"))
}

pub fn referenz(pfade: &[PathBuf], titel: &str) -> Result<String, String> {
    let mut sortiert: Vec<&PathBuf> = pfade.iter().collect();
    sortiert.sort();
    let mut teile = vec![format!("# {}\n", titel), "*Erzeugt aus dem Quelltext -- nicht von Hand aendern.*\n".to_string()];
    for p in sortiert { teile.push(datei_referenz(p)?); }
    Ok(teile.join("\n"))
}

fn referenz_main(args: &[String]) -> ExitCode {
    let mut ziel: Option<PathBuf> = None;
    let mut quellen: Vec<PathBuf> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-o" {
            if i + 1 >= args.len() { println!("doku referenz: nach -o fehlt der Dateiname"); return ExitCode::from(2); }
            ziel = Some(PathBuf::from(&args[i + 1]));
            i += 2;
            continue;
        }
        if !args[i].starts_with('-') { quellen.push(PathBuf::from(&args[i])); }
        i += 1;
    }
    if quellen.is_empty() {
        println!("Verwendung: dhrt doku referenz <datei.dh ...> [-o referenz.md]");
        return ExitCode::from(2);
    }
    let fehlend: Vec<String> = quellen.iter().filter(|p| !p.is_file()).map(|p| p.display().to_string()).collect();
    if !fehlend.is_empty() {
        println!("doku referenz: gibt es nicht: {}", fehlend.join(", "));
        return ExitCode::from(2);
    }
    match referenz(&quellen, "Referenz") {
        Ok(text) => match ziel {
            None => { print!("{}", text); ExitCode::SUCCESS }
            Some(z) => match std::fs::write(&z, text) {
                Ok(()) => { println!("geschrieben: {}", z.display()); ExitCode::SUCCESS }
                Err(e) => { eprintln!("{}: {}", z.display(), e); ExitCode::from(1) }
            },
        },
        Err(e) => { eprintln!("{}", e); ExitCode::from(1) }
    }
}

// ============================================================ aenderungen

/// Die Ueberschrift des Abschnitts, den das Aenderungsprotokoll sammelt.
const AE_DE: &str = "## Was sich für bestehende Programme ändert";
const AE_EN: &str = "## What changes for existing programs";

/// `2026.51` aus `release-2026.51.md`, als sortierbares Paar.
fn fassung_von(name: &str) -> Option<(u32, u32, String)> {
    let kern = name.strip_prefix("release-")?.strip_suffix(".md")?;
    if kern.ends_with(".en") { return None; }
    let (j, n) = kern.split_once('.')?;
    Some((j.parse().ok()?, n.parse().ok()?, kern.to_string()))
}

/// Der Abschnitt unter `kopf` bis zur naechsten `## `-Ueberschrift, ohne
/// Leerzeilen am Rand. Unterueberschriften werden fett (sonst stuenden sie
/// im Protokoll auf derselben Ebene wie die Fassungen), Verweise zeigen vom
/// Protokoll aus (`zu_releases` = Weg vom Protokoll nach docs/releases/).
fn abschnitt(text: &str, kopf: &str, zu_releases: &str) -> Option<String> {
    let text = text.replace("\r\n", "\n");
    let mut zeilen = text.lines();
    zeilen.by_ref().find(|z| z.trim_end() == kopf)?;
    let mut raus: Vec<String> = Vec::new();
    let mut im_code = false;
    for z in zeilen {
        if z.starts_with("```") { im_code = !im_code; }
        if !im_code && z.starts_with("## ") { break; }
        if !im_code && z.starts_with('#') {
            raus.push(format!("**{}**", z.trim_start_matches('#').trim()));
        } else {
            raus.push(z.to_string());
        }
    }
    while raus.first().is_some_and(|z| z.trim().is_empty()) { raus.remove(0); }
    while raus.last().is_some_and(|z| z.trim().is_empty()) { raus.pop(); }
    if raus.is_empty() { return None; }
    static VERWEIS: OnceLock<Regex> = OnceLock::new();
    let verweis = VERWEIS.get_or_init(|| Regex::new(r"\]\(([^)#\s]+)(#[^)\s]*)?\)").unwrap());
    let roh = raus.join("\n");
    Some(verweis.replace_all(&roh, |c: &regex::Captures| {
        let ziel = &c[1];
        let marke = c.get(2).map(|m| m.as_str()).unwrap_or("");
        if ziel.contains("://") || ziel.starts_with('/') || ziel.starts_with("mailto:") {
            return c[0].to_string();
        }
        format!("]({}{})", pfad_kuerzen(&format!("{}{}", zu_releases, ziel)), marke)
    }).into_owned())
}

/// `releases/../builtins-namen.md` -> `builtins-namen.md`.
fn pfad_kuerzen(p: &str) -> String {
    let mut teile: Vec<&str> = Vec::new();
    for t in p.split('/') {
        match t {
            "." | "" => {}
            ".." if teile.last().is_some_and(|l| *l != "..") => { teile.pop(); }
            _ => teile.push(t),
        }
    }
    teile.join("/")
}

/// docs/aenderungen.md und docs/en/aenderungen.md aus den Notizen der
/// Fassungen: je Fassung ihr Abschnitt "Was sich fuer bestehende Programme
/// aendert", die neueste zuerst. Englisch, wo es englische Notizen gibt; sonst
/// steht der deutsche Abschnitt mit einem Hinweis da (gleiche Gliederung und
/// gleiche Codebloecke wie das deutsche Protokoll -- `dhrt pruef uebersetzung`).
pub fn aenderungen(wurzel: &Path) -> Result<(String, String), String> {
    let ordner = wurzel.join("docs/releases");
    let mut fassungen: Vec<(u32, u32, String)> = std::fs::read_dir(&ordner)
        .map_err(|e| format!("{}: {}", ordner.display(), e))?
        .filter_map(|e| e.ok())
        .filter_map(|e| fassung_von(&e.file_name().to_string_lossy()))
        .collect();
    fassungen.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));
    let mut de = String::from(
"# Änderungsprotokoll

Was sich in jeder Fassung für **bestehende Programme** ändert, die neueste
zuerst -- gesammelt aus den Notizen der Fassungen. Was fest bleibt und was
sich ändern darf, sagt [Stabilität](stabilitaet.md).

*Erzeugt mit `dhrt doku aenderungen` aus `docs/releases/`; Änderungen gehören
in die Notizen, nicht hierher.*
");
    let mut en = String::from(
"# Change log

What changes for **existing programs** in each version, newest first --
collected from the release notes. What stays fixed and what may change is
described in [Stability](stabilitaet.md).

*Generated with `dhrt doku aenderungen` from `docs/releases/`; changes belong
in the release notes, not here.*
");
    for (_, _, f) in &fassungen {
        let text = std::fs::read_to_string(ordner.join(format!("release-{}.md", f))).unwrap_or_default();
        let text_en = std::fs::read_to_string(ordner.join(format!("release-{}.en.md", f))).ok();
        let teil_de = abschnitt(&text, AE_DE, "releases/");
        de.push_str(&format!("\n## {}\n\n", f));
        match &teil_de {
            Some(t) => de.push_str(&format!("{}\n\n", t)),
            None => de.push_str("Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.\n\n"),
        }
        de.push_str(&format!("[Alle Notizen zu {}](releases/release-{}.md)\n", f, f));
        en.push_str(&format!("\n## {}\n\n", f));
        let teil_en = text_en.as_deref().and_then(|t| abschnitt(t, AE_EN, "../releases/"));
        // Englisch nur, wenn es dieselben Codebloecke hat -- sonst stimmten
        // die beiden Protokolle nicht mehr ueberein.
        let bloecke = |t: &str| t.split("```").skip(1).step_by(2).map(str::to_string).collect::<Vec<_>>();
        match (&teil_de, teil_en) {
            (Some(d), Some(e)) if bloecke(d) == bloecke(&e) => en.push_str(&format!("{}\n\n", e)),
            (Some(_), _) => {
                let d = abschnitt(&text, AE_DE, "../releases/").unwrap_or_default();
                en.push_str(&format!("*Only in German so far:*\n\n{}\n\n", d));
            }
            (None, _) => en.push_str("The notes for this version have no separate section for this.\n\n"),
        }
        let notiz = if text_en.is_some() { format!("release-{}.en.md", f) } else { format!("release-{}.md", f) };
        en.push_str(&format!("[All notes for {}](../releases/{})\n", f, notiz));
    }
    Ok((de, en))
}

fn aenderungen_main(pruefen: bool) -> ExitCode {
    let wurzel = match repo_wurzel() { Ok(w) => w, Err(e) => { eprintln!("{}", e); return ExitCode::from(2); } };
    let (de, en) = match aenderungen(&wurzel) { Ok(t) => t, Err(e) => { eprintln!("{}", e); return ExitCode::from(2); } };
    let a = abliefern(&wurzel.join("docs/aenderungen.md"), &de, pruefen, "Änderungsprotokoll", "dhrt doku aenderungen");
    let b = abliefern(&wurzel.join("docs/en/aenderungen.md"), &en, pruefen, "Change log", "dhrt doku aenderungen");
    if a == ExitCode::SUCCESS && b == ExitCode::SUCCESS { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

pub const HILFE: &str = "\
dhrt doku prosa [--pruefen]        builtin_prosa.json aus docs/ und dem Referenzbuch erzeugen
dhrt doku grammatik [--pruefen]    VS-Code-Grammatik aus Schluesselwoertern und Index erzeugen
dhrt doku aenderungen [--pruefen]  Aenderungsprotokoll (docs/aenderungen.md, docs/en/) aus den Notizen der Fassungen
dhrt doku referenz <datei.dh ...> [-o ziel.md]
                                   Referenz aus dem Quelltext (Signaturen + Kommentarbloecke)
  --pruefen schreibt nichts und meldet mit Rueckgabe 1, wenn die Datei nicht mehr passt";

pub fn main(args: &[String]) -> ExitCode {
    let pruefen = args.iter().any(|a| a == "--pruefen");
    match args.first().map(String::as_str) {
        Some("prosa") => prosa_main(pruefen),
        Some("grammatik") => grammatik_main(pruefen),
        Some("aenderungen") => aenderungen_main(pruefen),
        Some("referenz") => referenz_main(&args[1..]),
        _ => { println!("{}", HILFE); ExitCode::from(2) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aenderungen_nimmt_den_abschnitt_und_richtet_verweise() {
        let notiz = [
            "# Drachenhauch 2026.9", "", "## Was sich für bestehende Programme ändert", "",
            "* **A** siehe [Namen](../builtins-namen.md#regeln) und [Web](https://x.y/z)",
            "### Unterpunkt", "```basic", "## kein Kopf im Code", "```", "", "## Neu", "", "anderes", "",
        ].join("\n");
        let notiz = notiz.as_str();
        let t = abschnitt(notiz, AE_DE, "releases/").unwrap();
        assert!(t.starts_with("* **A** siehe [Namen](builtins-namen.md#regeln)"), "{}", t);
        assert!(t.contains("[Web](https://x.y/z)"));
        assert!(t.contains("**Unterpunkt**"));
        assert!(t.contains("## kein Kopf im Code"));
        assert!(!t.contains("anderes"));
        let en = abschnitt(notiz, AE_DE, "../releases/").unwrap();
        assert!(en.contains("[Namen](../builtins-namen.md#regeln)"), "{}", en);
        assert!(abschnitt(&["# x", "", "## Neu", ""].join("\n"), AE_DE, "releases/").is_none());
        assert_eq!(fassung_von("release-2026.9.md").map(|f| (f.0, f.1)), Some((2026, 9)));
        assert!(fassung_von("release-2026.9.en.md").is_none());
    }

    #[test]
    fn saeubern_entfernt_markdown_und_kuerzt() {
        assert_eq!(saeubern(" `ABS(x)` liefert den [Betrag](x.md) **immer**: "), "ABS(x) liefert den Betrag immer");
        assert_eq!(saeubern("BOOLEAN | ob es laeuft |"), "BOOLEAN — ob es laeuft");
        assert_eq!(saeubern(" — | nur Text"), "nur Text");
        let lang = format!("{} ende", "wort ".repeat(120));
        let s = saeubern(&lang);
        assert!(s.ends_with('…') && s.chars().count() <= MAX_LAENGE + 2);
    }

    #[test]
    fn kurzformen_nur_echte_builtins() {
        let namen: HashSet<String> = ["PHYS3D_BODY_X", "PHYS3D_BODY_Y", "SCENE_GET_INT_OR", "SCENE_GET_FLOAT_OR"]
            .iter().map(|s| s.to_string()).collect();
        assert_eq!(kurzformen("`PHYS3D_BODY_X/Y/Z(w, k)`", &namen), vec!["PHYS3D_BODY_Y"]);
        assert_eq!(kurzformen("`SCENE_GET_INT_OR/FLOAT_OR(k)`", &namen), vec!["SCENE_GET_FLOAT_OR"]);
    }

    #[test]
    fn erster_satz_und_alternativen() {
        assert_eq!(erster_satz("Haelt an. Und weiter."), "Haelt an.");
        assert_eq!(erster_satz("Ohne Ende"), "Ohne Ende");
        let a = alternativen(&["AB".into(), "ABC".into(), "AA".into()]);
        assert_eq!(a, "ABC|AA|AB");
    }

    #[test]
    fn pruefen_ist_blind_fuer_zeilenenden() {
        // Der CI-Fund: git auf Windows checkt mit CRLF aus, erzeugt wird LF.
        let dir = std::env::temp_dir().join(format!("dhrt_doku_crlf_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ziel = dir.join("g.json");
        std::fs::write(&ziel, "{\r\n \"a\": 1\r\n}\r\n").unwrap();
        assert_eq!(abliefern(&ziel, "{\n \"a\": 1\n}\n", true, "G", "t"), ExitCode::SUCCESS);
        assert_eq!(abliefern(&ziel, "{\n \"a\": 2\n}\n", true, "G", "t"), ExitCode::from(1));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn referenz_aus_quelltext() {
        let dir = std::env::temp_dir().join(format!("dhrt_doku_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("mathe.dh");
        std::fs::write(&f, "' Kleine Sammlung.\n\n' Grenze.\nCONST GRENZE AS FLOAT = 1.0\n\n' Abstand.\n' Immer positiv.\nFUNCTION Distanz(x1 AS FLOAT) AS FLOAT\n    RETURN x1\nEND FUNCTION\n\n' Intern.\nPRIVATE SUB pruefe(w AS FLOAT)\nEND SUB\n").unwrap();
        let md = referenz(&[f.clone()], "Referenz").unwrap();
        assert!(md.contains("Kleine Sammlung."));
        assert!(md.contains("FUNCTION Distanz(x1 AS FLOAT) AS FLOAT"));
        assert!(md.contains("Abstand.\nImmer positiv."));
        assert!(md.contains("### Konstanten"));
        assert!(!md.contains("pruefe"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
