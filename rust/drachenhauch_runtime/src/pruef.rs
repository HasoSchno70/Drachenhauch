//! `dhrt pruef` -- die Doku-Pruefer, die bis 2026-09-06 in Python lagen
//! (`tools/pruef_docs.py`, `tools/pruef_doku_aussagen.py`; Weg A aus
//! `docs/entwurf-python-abbau.md`):
//!
//!   dhrt pruef                       alles (docs/ und CLAUDE.md)
//!   dhrt pruef bloecke [ordner]      jeden ```basic-Block durch die Front-End-Kette
//!   dhrt pruef namen [ordner] [--nur datei ...]   `NAME(` in Prosa/Tabellen muss ein Builtin sein
//!   dhrt pruef zaehlungen            "39 Module", "183 Beispiele" im README
//!   dhrt pruef konstanten [ordner]   jede Tasten-Konstante steht in der Doku
//!   dhrt pruef pfade [ordner] [--nur datei ...]   Pfade und Links zeigen auf Dateien, die es gibt
//!   dhrt pruef beispiele [repo]      Zahl der versionierten Beispiele (Hilfsausgabe)
//!   dhrt pruef meldungen             daten/meldungen.en.txt gegen den Quelltext
//!
//! Rueckgabe 0 = sauber, 1 = mindestens ein Befund, 2 = kein Repo.
//!
//! Die Codebloecke laufen IN-PROZESS durch `check_source` -- Python startete
//! dafuer einen dhrt je Buendel. Nur Syntax (lex/parse): die Doku zeigt fast
//! ueberall Ausschnitte, ein Compile-Fehler ("Variable nicht deklariert")
//! waere dort die Regel. Und was wie Schreibweise aussieht (`...`, `[x]`,
//! `->`, `FOR each`), zaehlt nicht -- ohne den Filter meldete der Pruefer
//! 27 "Fehler", die alle mit Absicht so dastehen.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::OnceLock;

use regex::Regex;

use crate::doku::repo_wurzel;

fn re(q: &'static str, zelle: &'static OnceLock<Regex>) -> &'static Regex {
    zelle.get_or_init(|| Regex::new(q).expect("Regex im Quelltext"))
}

/// Ein Befund: Datei, Zeile (0 = keine), was, warum.
pub type Befund = (String, usize, String, String);

fn md_dateien(ordner: &Path, nur: Option<&[String]>) -> Vec<PathBuf> {
    if let Some(namen) = nur {
        let mut v: Vec<String> = namen.to_vec();
        v.sort();
        return v.iter().map(|n| ordner.join(n)).filter(|p| p.exists()).collect();
    }
    let mut v: Vec<PathBuf> = std::fs::read_dir(ordner).map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |x| x == "md")).collect()).unwrap_or_default();
    v.sort();
    v
}

/// Datei als Text, Zeilenenden auf LF -- Python las mit Universal-Newlines,
/// und ein ```basic-Block in einer CRLF-Datei zaehlte sonst nicht.
fn lesen(p: &Path) -> String { std::fs::read_to_string(p).unwrap_or_default().replace("\r\n", "\n") }

fn name_von(p: &Path, wurzel: &Path) -> String {
    p.strip_prefix(wurzel).map(|r| r.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())
}

// ============================================================ 1. Codebloecke

/// Zeilen, die Schreibweise sind und kein Programm.
fn ist_notation(zeile: &str) -> bool {
    static N: [OnceLock<Regex>; 4] = [const { OnceLock::new() }; 4];
    let muster = [r"\.\.\.", r"\[.*\]", r"->", r"(?i)^\s*FOR each\b"];
    muster.iter().zip(N.iter()).any(|(q, z)| z.get_or_init(|| Regex::new(q).unwrap()).is_match(zeile))
}

/// Jeden ```basic-Block der `.md`-Dateien in `ordner` pruefen.
/// Liefert (Anzahl Bloecke, Befunde als (Datei, Zeile, Quellzeile, Meldung)).
pub fn bloecke(ordner: &Path) -> (usize, Vec<Befund>) {
    static BLOCK: OnceLock<Regex> = OnceLock::new();
    let block = re(r"(?is)```(?:basic|gb|dh|drachenhauch)\n(.*?)```", &BLOCK);
    let mut anzahl = 0usize;
    let mut funde: Vec<Befund> = Vec::new();
    for datei in md_dateien(ordner, None) {
        let text = lesen(&datei);
        let name = datei.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        for c in block.captures_iter(&text) {
            let code = &c[1];
            if code.trim().is_empty() { continue; }
            anzahl += 1;
            let md_zeile = text[..c.get(0).unwrap().start()].matches('\n').count() + 2;
            let zeilen: Vec<&str> = code.lines().collect();
            for d in crate::check_source(code, ordner, &name) {
                let phase = d["phase"].as_str().unwrap_or("");
                if phase != "lex" && phase != "parse" { continue; }
                if d["severity"].as_str() == Some("warning") { continue; }
                let nr = d["line"].as_u64().unwrap_or(1) as usize;
                let quelle = if nr >= 1 && nr <= zeilen.len() { zeilen[nr - 1].trim().to_string() } else { String::new() };
                if ist_notation(&quelle) { continue; }
                funde.push((name.clone(), md_zeile, quelle, d["message"].as_str().unwrap_or("").to_string()));
            }
        }
    }
    funde.sort();
    (anzahl, funde)
}

// ============================================================ 2. Befehlsnamen

/// Sprach-Schluesselwoerter, Typen und Konstanten -- keine Builtins.
const SPRACHE: &str = "DIM AS CONST IF THEN ELSE ELSEIF END WHILE WEND FOR TO STEP NEXT SUB FUNCTION \
RETURN CLASS NEW EXTENDS TRUE FALSE NIL AND OR NOT MOD BREAK CONTINUE IMAGE \
SOUND ARRAY OF STRUCT FILE MAP TRY CATCH THROW FINALLY IMPORT SELECT CASE IS \
REPEAT UNTIL DATA READ RESTORE BYREF ENUM BAND BOR BXOR BNOT SHL SHR TUPLE \
WITH STATIC FUNCREF IN WHERE PROPERTY OPERATOR YIELD COROUTINE PRINT INPUT \
INTEGER FLOAT STRING BOOLEAN BUFFER PI TAU SELF SUPER ABSTRACT PRIVATE GET SET \
LET REM ANY FUNKTION IIF";

/// Namen, die absichtlich nicht existieren. Jeder Eintrag braucht einen
/// Grund -- so bleibt die Liste eine bewusste Entscheidung.
pub const GEDULDET: &[(&str, &str)] = &[
    ("ECS_ADD_TO", "module-ecs.md nennt sie ausdruecklich 'hypothetisch'"),
    ("TASK_START", "allzweck-roadmap.md: WP H, ausdruecklich NICHT umgesetzt"),
    ("ERROR_FILE$", "allzweck-roadmap.md: WP F, ausdruecklich gestrichen"),
    ("ERROR_TRACE$", "allzweck-roadmap.md: WP F, ausdruecklich gestrichen"),
    ("NAME", "CLAUDE.md: Platzhalter in der Builtin-Anleitung (`NAME(a, b [, c])`)"),
    ("BITAND", "CLAUDE.md: als ENTFERNT beschrieben (heute der Operator BAND)"),
    ("DECLARE_GLOBAL_SLOT", "CLAUDE.md: Bytecode-Opcode, kein Builtin"),
    ("DECLARE_GLOBAL_CONST_SLOT", "CLAUDE.md: Bytecode-Opcode, kein Builtin"),
];

fn bekannte_builtins(wurzel: &Path) -> HashSet<String> {
    let raw = lesen(&wurzel.join("daten/builtin_index.json"));
    let mut namen: HashSet<String> = HashSet::new();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
        for e in v["builtins"].as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
            if let Some(n) = e["name"].as_str() {
                namen.insert(n.to_uppercase());
                namen.insert(n.trim_end_matches('$').to_uppercase());
            }
        }
    }
    namen
}

/// Codebloecke ausblenden (Zeilenzahl bleibt), damit Zeilennummern stimmen.
fn ohne_bloecke(text: &str) -> String {
    static BLOCK: OnceLock<Regex> = OnceLock::new();
    re(r"(?s)```.*?```", &BLOCK).replace_all(text, |c: &regex::Captures| "\n".repeat(c[0].matches('\n').count())).into_owned()
}

pub fn namen(wurzel: &Path, ordner: &Path, nur: Option<&[String]>) -> Vec<Befund> {
    static INLINE: OnceLock<Regex> = OnceLock::new();
    static AUFRUF: OnceLock<Regex> = OnceLock::new();
    let inline = re(r"`([^`\n]+)`", &INLINE);
    let aufruf = re(r"^([A-Z][A-Z0-9_]{2,}\$?)\s*\(", &AUFRUF);
    let sprache: HashSet<&str> = SPRACHE.split_whitespace().collect();
    let bekannt = bekannte_builtins(wurzel);
    let mut funde = Vec::new();
    for datei in md_dateien(ordner, nur) {
        let ohne = ohne_bloecke(&lesen(&datei));
        let name = datei.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        for m in inline.captures_iter(&ohne) {
            let Some(k) = aufruf.captures(m[1].trim()) else { continue };
            let n = k[1].to_uppercase();
            let kurz = n.trim_end_matches('$').to_string();
            if sprache.contains(n.as_str()) || sprache.contains(kurz.as_str()) { continue; }
            if bekannt.contains(&n) || bekannt.contains(&kurz) { continue; }
            if GEDULDET.iter().any(|(g, _)| *g == n) { continue; }
            let zeile = ohne[..m.get(0).unwrap().start()].matches('\n').count() + 1;
            funde.push((name.clone(), zeile, n, "kein Builtin dieses Namens (Tippfehler? umbenannt? entfernt?)".into()));
        }
    }
    funde
}

// ============================================================ 3. Zaehlungen

/// Die Beispiele laut Versionsverwaltung (nur die obere Ebene) -- oder None
/// ohne git. Gezaehlt wird der Index, nicht das Arbeitsverzeichnis: die
/// Live-Diagnose des Editors legt ihre Temp-Datei NEBEN die Quelle, und ein
/// glob, der genau dahinein faellt, meldete "sagt 195, tatsaechlich 196".
pub fn versionierte_beispiele(wurzel: &Path) -> Option<Vec<String>> {
    let out = std::process::Command::new("git").args(["-C", &wurzel.to_string_lossy(), "ls-files", "-z", "--", "examples/*.dh"]).output().ok()?;
    if !out.status.success() { return None; }
    let pfade: Vec<String> = String::from_utf8_lossy(&out.stdout).split('\0')
        .filter(|x| x.ends_with(".dh") && x.matches('/').count() == 1).map(String::from).collect();
    if pfade.is_empty() { None } else { Some(pfade) }
}

/// (Anzahl, Quelle der Zaehlung).
pub fn beispiele(wurzel: &Path) -> (usize, &'static str) {
    if let Some(v) = versionierte_beispiele(wurzel) { return (v.len(), "versioniert"); }
    let n = std::fs::read_dir(wurzel.join("examples")).map(|rd| rd.filter_map(|e| e.ok())
        .filter(|e| { let n = e.file_name().to_string_lossy().into_owned(); n.ends_with(".dh") && !n.starts_with("_dhtmp_") })
        .count()).unwrap_or(0);
    (n, "Arbeitsverzeichnis, kein git")
}

pub fn zaehlungen(wurzel: &Path) -> Vec<Befund> {
    static MODULE: OnceLock<Regex> = OnceLock::new();
    static NAMEN: OnceLock<Regex> = OnceLock::new();
    let text = lesen(&wurzel.join("README.md"));
    let (ist_beispiele, woher) = beispiele(wurzel);
    let quelle = lesen(&wurzel.join("rust/drachenhauch_runtime/src/preprocess.rs"));
    let ist_module = re(r"(?s)const MODULES[^=]*=\s*&\[(.*?)\];", &MODULE).captures(&quelle)
        .map(|c| re(r#""([a-z_0-9]+)""#, &NAMEN).find_iter(&c[1]).count()).unwrap_or(0);
    let mut funde = Vec::new();
    let pruefungen: [(&str, usize, String); 3] = [
        (r"alle (\d+) Beispiele", ist_beispiele, format!("Beispiele in examples/ ({})", woher)),
        (r"\*\*Module\*\* — (\d+) Stück", ist_module, "Module in preprocess.rs MODULES".into()),
        (r"(?m)^(\d+) Module, per `IMPORT", ist_module, "Module in preprocess.rs MODULES".into()),
    ];
    for (muster, ist, was) in pruefungen {
        let r = Regex::new(muster).unwrap();
        for c in r.captures_iter(&text) {
            let soll: usize = c[1].parse().unwrap_or(0);
            if soll != ist {
                let zeile = text[..c.get(0).unwrap().start()].matches('\n').count() + 1;
                funde.push(("README.md".into(), zeile, c[0].to_string(), format!("sagt {}, tatsaechlich {} ({})", soll, ist, was)));
            }
        }
    }
    funde
}

// ============================================================ 4. Konstanten

/// Jede Tasten-Konstante der Runtime muss in `docs/` stehen; Bereiche
/// ("`KEY_A` bis `KEY_Z`") gelten als Abdeckung.
pub fn konstanten(ordner: &Path) -> Vec<Befund> {
    let doku: String = md_dateien(ordner, None).iter().map(|d| lesen(d)).collect::<Vec<_>>().join("\n");
    let bereiche: [(&str, &str); 4] = [
        (r"^KEY_[A-Z]$", "`KEY_A` bis `KEY_Z`"), (r"^KEY_[0-9]$", "`KEY_0` bis `KEY_9`"),
        (r"^KEY_F\d+$", "`KEY_F1` bis `KEY_F12`"), (r"^KEY_KP\d$", "`KEY_KP0` bis `KEY_KP9`"),
    ];
    let bereich_re: Vec<(Regex, &str)> = bereiche.iter().map(|(m, t)| (Regex::new(m).unwrap(), *t)).collect();
    // Nur die KEY_*-Namen -- die JOY_*-Codes derselben Tabelle sind Gamepad-Knoepfe
    // und in module-input.md als Bind-Codes beschrieben, nicht als Tasten.
    let mut offen: Vec<String> = crate::vm::DEFAULT_KEYS.iter().map(|(n, _)| n.to_uppercase())
        .filter(|k| k.starts_with("KEY_"))
        .filter(|k| !doku.contains(k.as_str()) && !bereich_re.iter().any(|(m, t)| m.is_match(k) && doku.contains(t)))
        .collect();
    offen.sort();
    if offen.is_empty() { return Vec::new(); }
    vec![("docs/*.md".into(), 0, offen.join(", "),
          format!("{} Tasten-Konstante(n) gibt es in der Runtime, stehen aber in keiner Doku-Datei", offen.len()))]
}

// ============================================================ 5. Pfade und Links

/// Pfade, die es nicht (mehr) gibt und trotzdem stehen bleiben duerfen --
/// als Inline-Code in historischen Notizen. Fuer LINKS gilt die Liste NICHT:
/// ein Link verspricht "hier kannst du hinspringen".
pub const GEDULDETE_PFADE: &[(&str, &str)] = &[
    ("drachenhauch/interpreter.py", "Tree-Walker, mit Stufe B entfernt -- nur in historischen Notizen"),
    ("drachenhauch/parser.py", "Python-Parser, 2026-08-19 entfernt (Stufe C) -- nur noch im Entwurf genannt"),
    ("drachenhauch/ast_nodes.py", "AST-Knoten des Python-Parsers, mit ihm entfernt"),
    ("tests/test_rust_parser_parity.py", "Parity gegen den entfernten Python-Parser"),
    ("drachenhauch/serialize.py", "Python-Bytecode-Serializer, mit Stufe B entfernt"),
    ("drachenhauch/vm.py", "Python-Bytecode-VM, mit Stufe B entfernt"),
    ("drachenhauch/export.py", "Python-Export, von dhrts Bundler abgeloest"),
    ("drachenhauch/modules/gui.py", "Modul-Implementierung, in Rust reimplementiert"),
    ("drachenhauch/modules/ui.py", "Modul-Implementierung, in Rust reimplementiert"),
    ("tests/test_modules_gui.py", "Test der entfernten Python-Module"),
    ("tests/test_rust_compiler_parity.py", "Paritaets-Test gegen den entfernten Python-Compiler"),
    ("gb_native/src/broadphase.rs", "PyO3-Helfer-Crate, nach physics.rs portiert und entfernt"),
    ("drachenhauch/modules/x.py", "Platzhalter in einer Anleitung, kein echter Pfad"),
    ("examples/NN_gui.dh", "Platzhalter (NN = laufende Nummer), kein echter Pfad"),
    ("/program.dh", "virtueller Pfad im WASM-Dateisystem des Web-Playgrounds"),
    ("web/program.dh", "virtueller Pfad im WASM-Dateisystem des Web-Playgrounds"),
    ("modules/ecs_py.py", "Python-ECS, mit Stufe B entfernt -- nur in historischen Notizen"),
    ("rust/build.py", "Cython/PyO3-Build, mit Stufe B entfernt -- nur in historischen Notizen"),
    ("drachenhauch/lsp/features.py", "Python-Sprachserver, 2026-09-06 durch `dhrt lsp` abgeloest (Weg A)"),
    ("drachenhauch/lsp/server.py", "Python-Sprachserver, 2026-09-06 durch `dhrt lsp` abgeloest (Weg A)"),
    ("tests/test_lsp_features.py", "Test des entfernten Python-Sprachservers"),
    ("tests/test_lsp_server.py", "Test des entfernten Python-Sprachservers"),
    ("tools/gen_builtin_prosa.py", "Prosa-Generator, 2026-09-06 durch `dhrt doku prosa` abgeloest"),
    ("tools/pruef_docs.py", "Doku-Pruefer, 2026-09-06 durch `dhrt pruef bloecke` abgeloest"),
    ("tools/pruef_doku_aussagen.py", "Doku-Pruefer, 2026-09-06 durch `dhrt pruef` abgeloest"),
    ("tests/test_dhrt_call.py", "2026-09-16 nach tests/pruef/dhrt_call.dhtest umgezogen"),
    ("tests/test_dhrt_werkzeuge.py", "2026-09-16 nach tests/pruef/dhrt_werkzeuge.dhtest umgezogen"),
    ("vscode-drachenhauch/build_grammar.py", "Grammatik-Generator, 2026-09-06 durch `dhrt doku grammatik` abgeloest"),
    ("drachenhauch/doku.py", "Quelltext-Referenz, 2026-09-06 durch `dhrt doku referenz` abgeloest"),
    ("tests/test_ide.py", "IDE-Tests, bis Stufe 38 (2026-09-13) nach tests/pruef/werkzeug_ide*.dhtest gezogen"),
    ("tests/test_pilot_tracker.py", "Tracker-Pilot, in Stufe 40 (2026-09-13) nach tests/pruef/werkzeug_tracker.dhtest gezogen"),
    ("tests/test_pilot_rechnungen.py", "Rechnungen-Pilot, in Stufe 41 (2026-09-13) nach tests/pruef/werkzeug_rechnungen.dhtest gezogen"),
    ("tests/test_pilot_sprite_auswahl.py", "Sprite-Pilot, in Stufe 43 (2026-09-13) nach tests/pruef/werkzeug_sprite.dhtest gezogen"),
];

/// Alle Dateien und Verzeichnisse des Repos (ohne target/, __pycache__,
/// .venv, node_modules, .git) als repo-relative Pfade mit `/`.
fn repo_bestand(wurzel: &Path) -> (HashSet<String>, HashSet<String>) {
    fn gehe(d: &Path, wurzel: &Path, dateien: &mut HashSet<String>, ordner: &mut HashSet<String>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.filter_map(|e| e.ok()) {
            let p = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            // `.claude` haelt Arbeitskopien des Repos (worktrees) -- sie
            // zaehlen NICHT zum Bestand: eine geloeschte Datei lebte dort
            // weiter, die Pruefung war lokal gruen und in der CI rot.
            if matches!(name.as_str(), "target" | "__pycache__" | ".venv" | "node_modules" | ".git" | ".claude") { continue; }
            let rel = p.strip_prefix(wurzel).map(|r| r.to_string_lossy().replace('\\', "/")).unwrap_or_default();
            if p.is_dir() { ordner.insert(rel); gehe(&p, wurzel, dateien, ordner); }
            else { dateien.insert(rel); }
        }
    }
    let mut dateien = HashSet::new();
    let mut ordner = HashSet::new();
    gehe(wurzel, wurzel, &mut dateien, &mut ordner);
    (dateien, ordner)
}

pub fn pfade(wurzel: &Path, ordner: &Path, nur: Option<&[String]>) -> Vec<Befund> {
    static MUSTER: OnceLock<Regex> = OnceLock::new();
    static LINK: OnceLock<Regex> = OnceLock::new();
    static ZEILE: OnceLock<Regex> = OnceLock::new();
    let muster = re(r"`([A-Za-z0-9_./-]+\.(?:py|rs|dh|json|toml))`", &MUSTER);
    let link = re(r"\]\(([^)\s]+)\)", &LINK);
    let zeilennr = re(r":\d+$", &ZEILE);
    let (alle, verzeichnisse) = repo_bestand(wurzel);
    let lebt = |r: &str| { let r = r.trim_start_matches('/'); alle.iter().any(|a| a == r || a.ends_with(&format!("/{}", r))) };
    let ziel_lebt = |r: &str| {
        let mut r = r.split('#').next().unwrap_or("").trim_end_matches('/').to_string();
        r = zeilennr.replace(&r, "").into_owned();
        while let Some(rest) = r.strip_prefix("../") { r = rest.to_string(); }
        r.is_empty() || lebt(&r) || verzeichnisse.contains(&r)
    };
    let mut funde = Vec::new();
    for datei in md_dateien(ordner, nur) {
        let wo = name_von(&datei, wurzel);
        for (i, zeile) in lesen(&datei).lines().enumerate() {
            for m in muster.captures_iter(zeile) {
                let r = &m[1];
                if !r.contains('/') || lebt(r) || GEDULDETE_PFADE.iter().any(|(g, _)| *g == r) { continue; }
                funde.push((wo.clone(), i + 1, r.to_string(), "Pfad existiert nicht -- entfernt, umbenannt oder vertippt".into()));
            }
            for m in link.captures_iter(zeile) {
                let r = &m[1];
                if r.starts_with("http") || r.starts_with('#') || r.starts_with("mailto:") || ziel_lebt(r) { continue; }
                funde.push((wo.clone(), i + 1, format!("[...]({})", r), "Link zeigt ins Leere -- Ziel entfernt, umbenannt oder vertippt".into()));
            }
        }
    }
    funde
}

// ============================================================ Uebersetzung (docs/en)

/// Die Codebloecke einer Markdown-Datei in ihrer Reihenfolge, je mit der
/// Zeile des oeffnenden Zauns. Code wird nicht uebersetzt (wie im Buch) --
/// die englische Fassung traegt also dieselben Bloecke, und wer einen
/// deutschen Block aendert, sieht hier, welche Uebersetzung nachzuziehen ist.
/// Verglichen wird ohne Einrueckung und ohne Leerraum am Zeilenende: ein
/// Block in einer Aufzaehlung darf in der Uebersetzung anders eingerueckt sein.
fn codebloecke(text: &str) -> Vec<(usize, String)> {
    let mut aus = Vec::new();
    let mut offen: Option<(usize, String)> = None;
    for (i, zeile) in text.lines().enumerate() {
        let z = zeile.trim();
        if let Some((start, mut inhalt)) = offen.take() {
            if z.starts_with("```") { aus.push((start, inhalt)); }
            else { inhalt.push_str(z); inhalt.push('\n'); offen = Some((start, inhalt)); }
        } else if z.starts_with("```") {
            offen = Some((i + 1, format!("{}\n", z)));
        }
    }
    if let Some(o) = offen { aus.push(o); }
    aus
}

/// Die Sprungmarken einer Markdown-Datei, wie GitHub sie bildet (und die IDE,
/// `hbMarke$`): klein, Leerzeichen werden Striche, Buchstaben samt Umlauten,
/// Ziffern, `-` und `_` bleiben, alles andere faellt weg. Doppelte bekommen
/// `-1`, `-2`, ... angehaengt. Ueberschriften in Codebloecken zaehlen nicht.
pub fn marken(text: &str) -> HashSet<String> {
    let mut gesehen: std::collections::HashMap<String, usize> = Default::default();
    let mut aus = HashSet::new();
    let mut im_code = false;
    for zeile in text.lines() {
        let z = zeile.trim_start();
        if z.starts_with("```") { im_code = !im_code; continue; }
        if im_code || !z.starts_with('#') { continue; }
        let titel = z.trim_start_matches('#');
        if !titel.starts_with(' ') { continue; }
        let mut m = String::new();
        for c in titel.trim().to_lowercase().chars() {
            if c == ' ' { m.push('-'); }
            else if c == '-' || c == '_' || c.is_alphanumeric() { m.push(c); }
        }
        let n = gesehen.entry(m.clone()).or_insert(0);
        aus.insert(if *n == 0 { m.clone() } else { format!("{}-{}", m, n) });
        *n += 1;
    }
    aus
}

/// Verweise der `.md`-Dateien in `ordner` STRENG pruefen: der Pfad gilt
/// relativ zum Ordner der Datei (so liest ihn GitHub und die IDE), und eine
/// Sprungmarke muss es im Ziel geben. `pfade` prueft nur, ob es den
/// Dateinamen irgendwo gibt -- fuer docs/en reicht das nicht, weil dort jeder
/// Verweis auf eine deutsche Datei ein `../` braucht und jede Marke aus einer
/// englischen Ueberschrift entsteht.
pub fn verweise(wurzel: &Path, ordner: &Path) -> Vec<Befund> {
    static LINK: OnceLock<Regex> = OnceLock::new();
    let link = re(r"\]\(([^)\s]+)\)", &LINK);
    let mut funde = Vec::new();
    for datei in md_dateien(ordner, None) {
        let wo = name_von(&datei, wurzel);
        let text = lesen(&datei);
        let eigene = marken(&text);
        let mut im_code = false;
        for (i, zeile) in text.lines().enumerate() {
            if zeile.trim_start().starts_with("```") { im_code = !im_code; continue; }
            if im_code { continue; }
            for m in link.captures_iter(zeile) {
                let r = &m[1];
                if r.starts_with("http") || r.starts_with("mailto:") { continue; }
                let (pfad, marke) = match r.split_once('#') { Some((p, m)) => (p, m), None => (r, "") };
                let (ziel_marken, ziel) = if pfad.is_empty() {
                    (Some(eigene.clone()), datei.clone())
                } else {
                    let ziel = datei.parent().unwrap_or(ordner).join(pfad);
                    if !ziel.exists() {
                        funde.push((wo.clone(), i + 1, format!("[...]({})", r),
                            "Verweis zeigt ins Leere -- relativ zum Ordner der Datei gerechnet".into()));
                        continue;
                    }
                    let mk = if pfad.ends_with(".md") { Some(marken(&lesen(&ziel))) } else { None };
                    (mk, ziel)
                };
                if marke.is_empty() { continue; }
                if let Some(mk) = ziel_marken {
                    if !mk.contains(marke) {
                        funde.push((wo.clone(), i + 1, format!("[...]({})", r),
                            format!("Sprungmarke fehlt in {}", name_von(&ziel, wurzel))));
                    }
                }
            }
        }
    }
    funde
}

/// docs/en gegen docs: jede englische Datei hat ein deutsches Gegenstueck,
/// traegt dieselben Codebloecke in derselben Reihenfolge, und ihre Verweise
/// samt Sprungmarken loesen auf. Liefert (Zahl der englischen Dateien, Befunde).
pub fn uebersetzung(wurzel: &Path) -> (usize, Vec<Befund>) {
    let docs = wurzel.join("docs");
    let en = docs.join("en");
    let dateien = md_dateien(&en, None);
    let mut funde = Vec::new();
    for datei in &dateien {
        let wo = name_von(datei, wurzel);
        let name = datei.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let de = docs.join(&name);
        if !de.exists() {
            funde.push((wo, 1, name, "keine deutsche Fassung in docs/ -- die englische ist eine Uebersetzung, kein eigenes Dokument".into()));
            continue;
        }
        let b_en = codebloecke(&lesen(datei));
        let b_de = codebloecke(&lesen(&de));
        let erste = |b: &str| -> String { b.lines().nth(1).unwrap_or("").chars().take(60).collect() };
        for k in 0..b_en.len().max(b_de.len()) {
            match (b_en.get(k), b_de.get(k)) {
                (Some((z, a)), Some((zd, b))) if a != b => {
                    funde.push((wo.clone(), *z, erste(a),
                        format!("Codeblock {} weicht von docs/{}:{} ab -- Code wird nicht uebersetzt", k + 1, name, zd)));
                    break;
                }
                (Some((z, a)), None) => {
                    funde.push((wo.clone(), *z, erste(a), format!("Codeblock {} gibt es in docs/{} nicht", k + 1, name)));
                    break;
                }
                (None, Some((zd, b))) => {
                    funde.push((wo.clone(), 1, erste(b),
                        format!("Codeblock {} aus docs/{}:{} fehlt in der Uebersetzung", k + 1, name, zd)));
                    break;
                }
                _ => {}
            }
        }
    }
    funde.extend(verweise(wurzel, &en));
    funde.sort();
    (dateien.len(), funde)
}

// ============================================================ Aufruf

fn ausgeben(funde: &[Befund]) {
    for (datei, zeile, was, msg) in funde {
        println!("\n  {}:{}", datei, zeile);
        println!("     {}", was.chars().take(90).collect::<String>());
        println!("     -> {}", msg.chars().take(120).collect::<String>());
    }
}

fn code(funde: &[Befund]) -> ExitCode { if funde.is_empty() { ExitCode::SUCCESS } else { ExitCode::from(1) } }

/// `[ordner] [--nur datei ...]` zerlegen; ohne Ordner `docs/` des Repos.
fn ordner_und_nur(wurzel: &Path, args: &[String]) -> (PathBuf, Option<Vec<String>>) {
    let mut ordner: Option<PathBuf> = None;
    let mut nur: Vec<String> = Vec::new();
    let mut in_nur = false;
    for a in args {
        if a == "--nur" { in_nur = true; continue; }
        if in_nur { nur.push(a.clone()); } else if ordner.is_none() { ordner = Some(PathBuf::from(a)); }
    }
    (ordner.unwrap_or_else(|| wurzel.join("docs")), if nur.is_empty() { None } else { Some(nur) })
}

pub const HILFE: &str = "\
dhrt pruef                      alle Pruefungen (docs/ und CLAUDE.md)
dhrt pruef bloecke [ordner]     jeden ```basic-Block durch die Front-End-Kette (nur Syntax)
dhrt pruef namen [ordner] [--nur datei ...]
                                `NAME(` in Prosa und Tabellen muss ein Builtin sein
dhrt pruef zaehlungen           Zahlen im README (Beispiele, Module) nachzaehlen
dhrt pruef konstanten [ordner]  jede Tasten-Konstante steht in der Doku
dhrt pruef pfade [ordner] [--nur datei ...]
                                Pfade und Links zeigen auf Dateien, die es gibt
dhrt pruef beispiele [repo]     Zahl der versionierten Beispiele
dhrt pruef uebersetzung         docs/en gegen docs: Gegenstueck, gleiche Codebloecke, Verweise samt Marken
dhrt pruef meldungen            den englischen Katalog gegen den Quelltext pruefen
dhrt pruef meldungen --offen    Meldungen im Quelltext, die noch deutsch bleiben
  Rueckgabe 0 = sauber, 1 = Befund";

pub fn main(args: &[String]) -> ExitCode {
    let wurzel = match repo_wurzel() { Ok(w) => w, Err(e) => { eprintln!("{}", e); return ExitCode::from(2); } };
    let rest: Vec<String> = args.iter().skip(1).cloned().collect();
    match args.first().map(String::as_str) {
        Some("bloecke") => {
            let (ordner, _) = ordner_und_nur(&wurzel, &rest);
            let (n, funde) = bloecke(&ordner);
            println!("{} Codebloecke in {} geprueft -- {} Syntaxfehler", n, ordner.display(), funde.len());
            ausgeben(&funde);
            code(&funde)
        }
        Some("namen") => {
            let (ordner, nur) = ordner_und_nur(&wurzel, &rest);
            let funde = namen(&wurzel, &ordner, nur.as_deref());
            println!("Befehlsnamen geprueft -- {} Befund(e)", funde.len());
            ausgeben(&funde);
            code(&funde)
        }
        Some("zaehlungen") => { let f = zaehlungen(&wurzel); println!("Zaehlungen geprueft -- {} Befund(e)", f.len()); ausgeben(&f); code(&f) }
        Some("konstanten") => {
            let (ordner, _) = ordner_und_nur(&wurzel, &rest);
            let f = konstanten(&ordner); println!("Tasten-Konstanten geprueft -- {} Befund(e)", f.len()); ausgeben(&f); code(&f)
        }
        Some("pfade") => {
            let (ordner, nur) = ordner_und_nur(&wurzel, &rest);
            let f = pfade(&wurzel, &ordner, nur.as_deref());
            println!("Pfade und Links geprueft -- {} Befund(e)", f.len()); ausgeben(&f); code(&f)
        }
        Some("uebersetzung") => {
            let (n, f) = uebersetzung(&wurzel);
            println!("{} englische Dokumente geprueft -- {} Befund(e)", n, f.len());
            ausgeben(&f); code(&f)
        }
        Some("beispiele") => {
            let repo = rest.first().map(PathBuf::from).unwrap_or(wurzel);
            let (n, woher) = beispiele(&repo);
            println!("{} {}", n, woher);
            ExitCode::SUCCESS
        }
        Some("meldungen") if rest.iter().any(|a| a == "--offen") => {
            let offen = meldungen_offen(&wurzel);
            let mut je_datei: std::collections::BTreeMap<String, usize> = Default::default();
            for (d, z, t) in &offen {
                println!("{}:{}\t{}", d, z, t);
                *je_datei.entry(d.clone()).or_default() += 1;
            }
            println!("\n{} Meldung(en) ohne englische Vorlage:", offen.len());
            for (d, n) in je_datei { println!("  {:5}  {}", n, d); }
            ExitCode::SUCCESS
        }
        Some("meldungen") => {
            let (n, f) = meldungen(&wurzel);
            println!("{} englische Vorlagen geprueft -- {} Befund(e)", n, f.len());
            ausgeben(&f); code(&f)
        }
        Some("--help") | Some("-h") => { println!("{}", HILFE); ExitCode::SUCCESS }
        Some(x) => { eprintln!("dhrt pruef: unbekannt: {}\n{}", x, HILFE); ExitCode::from(2) }
        None => {
            // Alles -- CLAUDE.md mitgeprueft, mit denselben Regeln wie docs/.
            let docs = wurzel.join("docs");
            let (n, mut funde) = bloecke(&docs);
            let claude = vec!["CLAUDE.md".to_string()];
            funde.extend(namen(&wurzel, &docs, None));
            funde.extend(zaehlungen(&wurzel));
            funde.extend(konstanten(&docs));
            funde.extend(pfade(&wurzel, &docs, None));
            funde.extend(namen(&wurzel, &wurzel, Some(&claude)));
            funde.extend(pfade(&wurzel, &wurzel, Some(&claude)));
            funde.extend(meldungen(&wurzel).1);
            // Die englische Fassung mit denselben Regeln -- ohne die
            // Tasten-Konstanten: die verlangen jede Konstante im Ordner, und
            // was unuebersetzt ist, liegt nur in docs/.
            let en = docs.join("en");
            let mut n = n;
            if en.is_dir() {
                let (n_en, f_en) = bloecke(&en);
                n += n_en;
                funde.extend(f_en);
                funde.extend(namen(&wurzel, &en, None));
                funde.extend(pfade(&wurzel, &en, None));
                funde.extend(uebersetzung(&wurzel).1);
            }
            println!("{} Codebloecke geprueft; Doku-Aussagen geprueft -- {} Befund(e)", n, funde.len());
            ausgeben(&funde);
            println!("\n({} Namen und {} Pfade geduldet, siehe GEDULDET/GEDULDETE_PFADE in pruef.rs)",
                     GEDULDET.len(), GEDULDETE_PFADE.len());
            code(&funde)
        }
    }
}

/// Die Zeichenketten-Literale eines Rust-Quelltexts, entschluesselt
/// (`\"` -> `"`, Zeilenfortsetzung mit `\` weg) -- so, wie die Meldung
/// spaeter dasteht. Zeichen-Literale werden uebersprungen, damit ein `'"'`
/// nicht den Rest der Datei verschiebt; Kommentare ebenso.
fn rust_literale(text: &str) -> Vec<String> {
    let z: Vec<char> = text.chars().collect();
    let mut aus = Vec::new();
    let mut i = 0;
    while i < z.len() {
        let c = z[i];
        if c == '/' && z.get(i + 1) == Some(&'/') {
            while i < z.len() && z[i] != '\n' { i += 1; }
            continue;
        }
        // Rohe Zeichenkette r"..." / r#"..."#: kein Rueckstrich-Escape, und
        // ein " darin endet sie nicht -- sonst verrutscht der Rest der Datei.
        if c == 'r' && matches!(z.get(i + 1), Some('"') | Some('#'))
            && (i == 0 || !(z[i - 1].is_alphanumeric() || z[i - 1] == '_')) {
            let mut j = i + 1;
            let mut rauten = 0;
            while z.get(j) == Some(&'#') { rauten += 1; j += 1; }
            if z.get(j) == Some(&'"') {
                let mut s = String::new();
                j += 1;
                while j < z.len() {
                    if z[j] == '"' && (1..=rauten).all(|k| z.get(j + k) == Some(&'#')) { break; }
                    s.push(z[j]);
                    j += 1;
                }
                aus.push(s);
                i = j + 1 + rauten;
                continue;
            }
        }
        if c == '\'' {
            // Zeichen-Literal ('x', '\n', '\'') -- aber kein Lebensdauer-Name ('a).
            if z.get(i + 1) == Some(&'\\') && z.get(i + 3) == Some(&'\'') { i += 4; continue; }
            if z.get(i + 2) == Some(&'\'') { i += 3; continue; }
            i += 1;
            continue;
        }
        if c == '"' {
            let mut s = String::new();
            i += 1;
            while i < z.len() && z[i] != '"' {
                if z[i] == '\\' && i + 1 < z.len() {
                    match z[i + 1] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        '\n' | '\r' => {
                            i += 1;
                            while i < z.len() && z[i].is_whitespace() { i += 1; }
                            continue;
                        }
                        x => s.push(x),
                    }
                    i += 2;
                    continue;
                }
                s.push(z[i]);
                i += 1;
            }
            aus.push(s);
            i += 1;
            continue;
        }
        i += 1;
    }
    aus
}

/// Die Bruchstuecke eines Literals zwischen seinen Format-Platzhaltern
/// (`{}`, `{:?}`, `{name}`), `{{`/`}}` entschluesselt.
fn format_stuecke(lit: &str) -> Vec<String> {
    let z: Vec<char> = lit.chars().collect();
    let mut aus = vec![String::new()];
    let mut i = 0;
    while i < z.len() {
        match z[i] {
            '{' if z.get(i + 1) == Some(&'{') => { aus.last_mut().unwrap().push('{'); i += 2; }
            '}' if z.get(i + 1) == Some(&'}') => { aus.last_mut().unwrap().push('}'); i += 2; }
            '{' => match z[i..].iter().position(|&c| c == '}') {
                Some(p) if z[i + 1..i + p].iter().all(|c| c.is_alphanumeric() || matches!(c, '_' | ':' | '?' | '.')) => {
                    aus.push(String::new());
                    i += p + 1;
                }
                _ => { aus.last_mut().unwrap().push('{'); i += 1; }
            },
            c => { aus.last_mut().unwrap().push(c); i += 1; }
        }
    }
    aus
}

/// Steht dieses feste Stueck einer Vorlage im Quelltext? Es muss in EINEM
/// Bruchstueck eines Literals stehen -- zusammengesetzt aus beliebigen
/// Teilen fand sich im ganzen Quelltext fast alles. Wo eine Meldung wirklich
/// aus mehreren Literalen entsteht (`concat!`, ein eingesetztes Wort), sagt
/// der Katalog es: `[[wort]]` ist ein ganzes eigenes Literal, `[[]]` nur die
/// Naht dazwischen.
fn stueck_belegt(stueck: &str, korpus: &str, ganze: &HashSet<String>) -> bool {
    let mut rest = stueck;
    loop {
        let (draussen, drinnen, weiter) = match rest.find("[[") {
            Some(a) => match rest[a..].find("]]") {
                Some(e) => (&rest[..a], Some(&rest[a + 2..a + e]), &rest[a + e + 2..]),
                None => (rest, None, ""),
            },
            None => (rest, None, ""),
        };
        let d = draussen.trim();
        if d.chars().count() >= 3 && !korpus.contains(draussen) && !korpus.contains(d) { return false; }
        if let Some(w) = drinnen {
            if !w.trim().is_empty() && !ganze.contains(w.trim()) { return false; }
        }
        if drinnen.is_none() { return true; }
        rest = weiter;
    }
}

/// `dhrt pruef meldungen`: jede deutsche Vorlage aus daten/meldungen.en.txt
/// muss (in ihren festen Stuecken) noch im Quelltext stehen -- sonst
/// uebersetzt sie eine Meldung, die es nicht mehr gibt, und die geaenderte
/// bleibt still deutsch. Dazu: passende Platzhalter, keine Doppelten.
pub fn meldungen(wurzel: &Path) -> (usize, Vec<Befund>) {
    let mut funde: Vec<Befund> = Vec::new();
    let datei = wurzel.join("daten").join("meldungen.en.txt");
    let name = "daten/meldungen.en.txt".to_string();
    let Ok(katalog) = std::fs::read_to_string(&datei) else {
        return (0, vec![(name, 0, "fehlt".into(), "Katalog nicht lesbar".into())]);
    };
    let mut literale: Vec<String> = Vec::new();
    if let Ok(eintraege) = std::fs::read_dir(wurzel.join("rust/drachenhauch_runtime/src")) {
        let mut pfade: Vec<PathBuf> = eintraege.flatten().map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "rs") && !p.ends_with("meldung.rs"))
            .collect();
        pfade.sort();
        for p in pfade {
            if let Ok(t) = std::fs::read_to_string(&p) { literale.extend(rust_literale(&t)); }
        }
    }
    // Jedes Literal zweimal: wie es dasteht und in seinen Bruchstuecken
    // zwischen den Platzhaltern. Getrennt durch \0, damit kein Stueck ueber
    // zwei Literale hinweg passt.
    let mut korpus = String::new();
    let mut ganze: HashSet<String> = HashSet::new();
    for l in &literale {
        korpus.push('\u{0}'); korpus.push_str(l);
        ganze.insert(l.trim().to_string());
        for f in format_stuecke(l) {
            korpus.push('\u{0}'); korpus.push_str(&f);
            ganze.insert(f.trim().to_string());
        }
    }
    let zeilen = crate::meldung::katalog_zeilen(&katalog);
    let mut gesehen: HashSet<String> = HashSet::new();
    for (de, en, n) in &zeilen {
        if !gesehen.insert(de.clone()) {
            funde.push((name.clone(), *n, de.clone(), "doppelt -- die erste gewinnt".into()));
        }
        let teile = crate::meldung::stuecke(de);
        let deutsch = teile.len() - 1;
        let (leer, groesste) = crate::meldung::platzhalter_englisch(en);
        if (groesste == 0 && leer != deutsch) || (groesste > 0 && (leer > 0 || groesste > deutsch)) {
            funde.push((name.clone(), *n, de.clone(),
                        format!("{} Platzhalter deutsch, englisch {}", deutsch, leer.max(groesste))));
        }
        for t in &crate::meldung::stuecke_roh(de) {
            let t = t.trim();
            if t.chars().count() < 3 { continue; }
            if !stueck_belegt(t, &korpus, &ganze) {
                funde.push((name.clone(), *n, de.clone(), format!("steht nicht mehr im Quelltext: \"{}\"", t)));
                break;
            }
        }
    }
    (zeilen.len(), funde)
}

/// Woerter, an denen eine deutsche Meldung zu erkennen ist -- in einer
/// englischen Uebersetzung kommt keines davon vor.
const DEUTSCHE_WOERTER: &[&str] = &[" nicht", "erwartet", "Erwartet", " ist ", " fehlt", " kein",
    " muss ", "ungueltig", "Ungueltig", "unbekannt", "Unbekannt", "ausserhalb", " gibt es",
    " darf ", " zu gross", "erhalten", " oder ", " und ", " nur ", " noch ", " wird ", " werden "];

/// `dhrt pruef meldungen --offen`: jedes Literal im Quelltext, das nach
/// einer deutschen Meldung aussieht, mit Platzhaltern gefuellt durch den
/// Uebersetzer -- was danach noch deutsche Woerter traegt, steht in der
/// Liste. Eine Hilfe fuer die Katalogarbeit, keine Pruefung: ein Stueck
/// einer zusammengesetzten Meldung steht hier auch dann, wenn das Ganze
/// uebersetzt wird.
pub fn meldungen_offen(wurzel: &Path) -> Vec<(String, usize, String)> {
    let mut aus = Vec::new();
    let ordner = wurzel.join("rust/drachenhauch_runtime/src");
    let Ok(eintraege) = std::fs::read_dir(&ordner) else { return aus };
    let mut pfade: Vec<PathBuf> = eintraege.flatten().map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs")).collect();
    pfade.sort();
    let auslassen = ["meldung.rs", "pruef.rs", "doku.rs", "pruefsammlung.rs"];
    for p in pfade {
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if auslassen.contains(&name.as_str()) { continue; }
        let Ok(mut text) = std::fs::read_to_string(&p) else { continue };
        if let Some(i) = text.find("#[cfg(test)]\nmod tests") { text.truncate(i); }
        let mut gesehen: HashSet<String> = HashSet::new();
        for (zeile, lit) in rust_literale_mit_zeile(&text) {
            if lit.contains('\n') || lit.chars().count() < 12 || !lit.contains(' ') { continue; }
            if !DEUTSCHE_WOERTER.iter().any(|w| lit.contains(w)) { continue; }
            if !gesehen.insert(lit.clone()) { continue; }
            // Platzhalter mit Woertern fuellen, die keine Vorlage zufaellig
            // trifft und die selbst nicht deutsch aussehen.
            let teile = format_stuecke(&lit);
            let mut probe = String::new();
            for (i, t) in teile.iter().enumerate() {
                if i > 0 { probe.push_str(&format!("X{}", i)); }
                probe.push_str(t);
            }
            let en = crate::meldung::uebersetzen(&probe);
            if DEUTSCHE_WOERTER.iter().any(|w| en.contains(w)) {
                aus.push((name.clone(), zeile, lit));
            }
        }
    }
    aus
}

/// Wie `rust_literale`, mit der Zeile, in der das Literal beginnt.
fn rust_literale_mit_zeile(text: &str) -> Vec<(usize, String)> {
    // Die Zeile jedes Literals: der Text wird zeilenweise zerlegt, aber ein
    // Literal kann ueber eine Zeile hinaus gehen -- darum ueber die Lage im
    // Text, nicht ueber die Zeilen selbst.
    let mut aus = Vec::new();
    let mut zeile = 1;
    let mut rest = text;
    let mut verbraucht = 0usize;
    for lit in rust_literale(text) {
        // Das naechste Vorkommen des Anfangs suchen (grob, fuer die Anzeige).
        let anfang: String = lit.chars().take(12).collect();
        if let Some(i) = rest.find(&anfang) {
            zeile += rest[..i].matches('\n').count();
            verbraucht += i;
            rest = &text[verbraucht..];
        }
        aus.push((zeile, lit));
    }
    aus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literale_und_stuecke() {
        let l = rust_literale("let c = '\"'; f(\"a \\\"b\\\"\"); g(\"lang \\\n     weiter\"); // \"nicht\"");
        assert_eq!(l, vec!["a \"b\"".to_string(), "lang weiter".to_string()]);
        let k = "\u{0}Die Klasse kuendigt \u{0}eine Methode\u{0} an, ohne sie";
        let g: HashSet<String> = ["eine Methode".to_string(), "Zahl".to_string()].into_iter().collect();
        assert!(stueck_belegt("Die Klasse kuendigt [[eine Methode]] an, ohne sie", k, &g));
        assert!(!stueck_belegt("Die Klasse kuendigt eine Methode an, ohne sie", k, &g));
        assert!(!stueck_belegt("Die Klasse kuendigt [[keine Methode]] an, ohne sie", k, &g));
        assert_eq!(format_stuecke("a {} b {:?} c {{x}} {name}"), vec!["a ", " b ", " c {x} ", ""]);
    }

    #[test]
    fn marken_wie_github() {
        let m = marken("# Sprite-Atlas + Batch-Draw\n## Server: PostgreSQL und MySQL\n```\n# kein Kopf\n```\n## `DB_OPEN`\n## Größe\n## Größe\n");
        for k in ["sprite-atlas--batch-draw", "server-postgresql-und-mysql", "db_open", "größe", "größe-1"] {
            assert!(m.contains(k), "{} fehlt in {:?}", k, m);
        }
        assert!(!m.contains("kein-kopf"));
    }

    #[test]
    fn uebersetzung_folgt_der_deutschen() {
        let dir = std::env::temp_dir().join(format!("dh_pruef_en_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("docs/en")).unwrap();
        let de = "# Titel\n\nText.\n\n```basic\nPRINT 1 ' eins\n```\n\n## Zweiter Teil\n";
        std::fs::write(dir.join("docs/a.md"), de).unwrap();
        std::fs::write(dir.join("docs/b.md"), "# B\n").unwrap();
        // gleich: Code unveraendert (anders eingerueckt), Verweis auf die
        // deutsche Datei mit ../, Marke aus der englischen Ueberschrift
        std::fs::write(dir.join("docs/en/a.md"),
            "# Title\n\nText, see [b](../b.md#b) and [part](#second-part).\n\n  ```basic\n  PRINT 1 ' eins\n  ```\n\n## Second part\n").unwrap();
        let (n, f) = uebersetzung(&dir);
        assert_eq!((n, f.len()), (1, 0), "{:?}", f);
        // Code uebersetzt, Marke deutsch, Verweis ohne ../, Datei ohne Gegenstueck
        std::fs::write(dir.join("docs/en/a.md"),
            "# Title\n\n[b](b.md) [x](#zweiter-teil)\n\n```basic\nPRINT 1 ' one\n```\n\n## Second part\n").unwrap();
        std::fs::write(dir.join("docs/en/c.md"), "# C\n").unwrap();
        let (_, f) = uebersetzung(&dir);
        let msgs: Vec<&str> = f.iter().map(|b| b.3.as_str()).collect();
        assert!(msgs.iter().any(|m| m.contains("Code wird nicht uebersetzt")), "{:?}", msgs);
        assert!(msgs.iter().any(|m| m.contains("Sprungmarke fehlt")), "{:?}", msgs);
        assert!(msgs.iter().any(|m| m.contains("ins Leere")), "{:?}", msgs);
        assert!(msgs.iter().any(|m| m.contains("keine deutsche Fassung")), "{:?}", msgs);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn notation_wird_erkannt() {
        assert!(ist_notation("PRINT ..."));
        assert!(ist_notation("SPEAK(text$ [, unterbrechen])"));
        assert!(ist_notation("FOR each tile:"));
        assert!(!ist_notation("PRINT 1"));
    }

    #[test]
    fn geduldete_eintraege_sind_begruendet() {
        for (n, g) in GEDULDET { assert!(g.len() > 20, "{}: Grund zu duenn", n); }
        for (p, g) in GEDULDETE_PFADE { assert!(g.len() > 20, "{}: Grund zu duenn", p); }
    }

    #[test]
    fn links_werden_geprueft_pfade_geduldet() {
        let dir = std::env::temp_dir().join(format!("dhrt_pruef_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("docs")).unwrap();
        std::fs::create_dir_all(dir.join("rust")).unwrap();
        std::fs::write(dir.join("rust/build_wasm.py"), "").unwrap();
        std::fs::write(dir.join("docs/probe.md"),
            "[tot](drachenhauch/gibtsnicht.py)\n[lebt](rust/build_wasm.py)\n[ordner](docs/)\n[mit Zeile](rust/build_wasm.py:114)\n[netz](https://example.invalid/x.py)\n`drachenhauch/interpreter.py` ist geduldet\n[klick](drachenhauch/interpreter.py) nicht als Link\n").unwrap();
        let f = pfade(&dir, &dir.join("docs"), Some(&["probe.md".to_string()]));
        assert_eq!(f.len(), 2, "{:?}", f);
        assert!(f[0].2.contains("gibtsnicht.py"));
        assert!(f[1].2.contains("interpreter.py"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn codebloecke_ohne_syntaxfehler_und_mit() {
        let dir = std::env::temp_dir().join(format!("dhrt_pruef_b_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.md"), "Text\n\n```basic\nPRINT 1\n```\n\n```basic\nDIM x AS\n```\n\n```basic\nPRINT ...\n```\n").unwrap();
        let (n, funde) = bloecke(&dir);
        assert_eq!(n, 3);
        assert_eq!(funde.len(), 1, "{:?}", funde);
        assert_eq!(funde[0].1, 8);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
