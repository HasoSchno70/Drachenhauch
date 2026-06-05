//! `gbrt` -- native GameBasic-Runtime (Spike, Schritt 1+2).
//!
//! Laedt eine `.gbc`-Datei (vom Python-Compiler erzeugt, siehe
//! `gamebasic/serialize.py`) und fuehrt den VM-Kern aus. Ausgabe nach stdout
//! soll bit-identisch zur Python-VM sein.
//!
//! Verwendung: gbrt <datei.gbc> [quell-label]
//!
//! Das optionale `quell-label` (z.B. `spiel.gb`) wird nur fuer Laufzeitfehler-
//! Meldungen genutzt (`Laufzeitfehler in spiel.gb:Zeile: ...`). `gbrun.py
//! --native` reicht den Namen der `.gb`-Quelldatei durch.

mod ast;
mod astar;
#[cfg(feature = "graphics")]
mod audio;
mod builtins;
mod compiler;
mod parser;
mod controller;
#[cfg(feature = "db")]
mod db;
mod ecs;
#[cfg(feature = "http")]
mod html;
#[cfg(feature = "net")]
mod net;
#[cfg(feature = "bt")]
mod bt;
#[cfg(feature = "serial")]
mod serial;
#[cfg(feature = "usb")]
mod usb;
#[cfg(feature = "wifi")]
mod wifi;
#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
mod gui;
mod lexer;
mod model;
mod physics;
mod preprocess;
mod tiled;
mod value;
mod vm;

use std::io::Write;
use std::process::ExitCode;

/// Magic am Ende einer gebundelten `.exe`. Layout der letzten 16 Bytes:
/// `[u64 Laenge der .gbc-Bytes, little-endian][8 Byte Magic]`. Die `.gbc`-Bytes
/// liegen direkt vor diesem Footer. So wird `gbrt` selbst zur Spiel-Exe:
/// `export.py` haengt `<gbc><laenge><magic>` an eine Kopie von `gbrt.exe`.
const PAYLOAD_MAGIC: &[u8; 8] = b"GBRTPAY1";

/// Liest eine eingebettete `.gbc` aus der eigenen Exe (Bundle-Modus) -- oder
/// None, wenn kein Payload angehaengt ist (normaler Dev-Modus).
fn embedded_gbc() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let data = std::fs::read(&exe).ok()?;
    if data.len() < 16 { return None; }
    let footer = data.len() - 16;
    if &data[footer + 8..] != PAYLOAD_MAGIC { return None; }
    let len = u64::from_le_bytes(data[footer..footer + 8].try_into().ok()?) as usize;
    if len == 0 || len + 16 > data.len() { return None; }
    String::from_utf8(data[footer - len..footer].to_vec()).ok()
}

fn main() -> ExitCode {
    // Front-End-Debug: `gbrt --tokens <datei.gb>` gibt den Token-Strom als
    // kanonisches JSON aus (Parity-Vergleich mit dem Python-Lexer).
    {
        let raw: Vec<String> = std::env::args().collect();
        if raw.len() >= 3 && raw[1] == "--tokens" {
            return tokens_main(&raw[2]);
        }
        if raw.len() >= 3 && raw[1] == "--ast" {
            return ast_main(&raw[2]);
        }
        // Debug: kompiliert die Quelle und gibt das .gbc-JSON aus (Bytecode-Dump).
        if raw.len() >= 3 && raw[1] == "--dumpbc" {
            let src = match std::fs::read_to_string(&raw[2]) {
                Ok(t) => t,
                Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", raw[2], e); return ExitCode::from(1); }
            };
            let base = std::path::Path::new(&raw[2]).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."));
            return match compile_source(&src, &base, &raw[2]) {
                Ok(j) => { println!("{}", serde_json::to_string_pretty(&j).unwrap_or_default()); ExitCode::SUCCESS }
                Err(code) => code,
            };
        }
        // Editor/LSP-Diagnostik: kompiliert die Quelle und gibt gefundene
        // Probleme als JSON-Array auf stdout aus (leer = sauber). Exit 0 auch
        // bei Diagnosen -- so unterscheidet der Editor "Probleme gefunden" von
        // "Tool-Fehler" (I/O -> Exit 1). Ersetzt die Python-Compiler-Pruefung.
        if raw.len() >= 3 && raw[1] == "--check" {
            return check_main(&raw[2]);
        }
        // Stufe 4: IMPORT-Preprocessor -- gibt die gemergte Quelle aus
        // (Merge-Parity gegen preprocess.process()).
        if raw.len() >= 3 && raw[1] == "--preprocess" {
            return preprocess_main(&raw[2]);
        }
        // Stufe 3: Quelltext in Rust kompilieren + ausfuehren (Output-Parity).
        if raw.len() >= 3 && raw[1] == "--runsrc" {
            return runsrc_main(&raw[2]);
        }
        // Stufe 5: `gbrt run datei.gb` -- eigenstaendiger End-to-End-Lauf
        // (preprocess+lex+parse+compile+run, chdir ins Datei-Verzeichnis fuer
        // relative Asset-Pfade). gbrt ist damit ohne Python lauffaehig.
        if raw.len() >= 3 && raw[1] == "run" {
            return run_main(&raw[2]);
        }
        // Selbst-Export: `gbrt --export datei.gb [out_dir]` buendelt das
        // Programm aus Quelltext zu einer eigenstaendigen Exe (ohne Python).
        if raw.len() >= 3 && raw[1] == "--export" {
            return export_main(&raw[2], raw.get(3).map(|s| s.as_str()));
        }
    }

    // Bundle-Modus: eingebettete .gbc am Ende der eigenen Exe?
    if let Some(text) = embedded_gbc() {
        // Ins Exe-Verzeichnis wechseln, damit relative Asset-Pfade
        // (LOADIMAGE("assets/...")) auch beim Doppelklick von ueberall stimmen.
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() { let _ = std::env::set_current_dir(dir); }
        }
        let label = std::env::current_exe().ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "spiel".into());
        return run_gbc_text(&text, &label);
    }

    // WASM/Web (emscripten): Programm aus einem festen Pfad im virtuellen FS
    // (vom Build via --embed-file eingebettet bzw. vom JS-Harness per
    // FS.writeFile reingeschrieben). Seit dem Front-End-Port (Stufe 1-5) kann
    // gbrt die `.gb`-QUELLE selbst kompilieren -> der Playground braucht KEIN
    // vorab-kompiliertes .gbc (und kein Pyodide) mehr. `/program.gb` (Quelle)
    // hat Vorrang; `/program.gbc` (vorkompiliert) bleibt als Fallback. Siehe
    // web/ + docs/web-playground.md.
    #[cfg(target_os = "emscripten")]
    {
        if let Ok(src) = std::fs::read_to_string("/program.gb") {
            return compile_and_run_source(&src, std::path::Path::new("/"), "playground");
        }
        if let Ok(text) = std::fs::read_to_string("/program.gbc") {
            return run_gbc_text(&text, "playground");
        }
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Verwendung: {} <datei.gbc>", args[0]);
        return ExitCode::from(1);
    }
    let path = &args[1];
    // Komfort: `gbrt datei.gb` (ohne `run`) wird wie `gbrt run datei.gb`
    // behandelt -- aus Quelltext, mit chdir. `.gbc` laeuft den VM-Pfad.
    if path.to_lowercase().ends_with(".gb") {
        return run_main(path);
    }
    // Optionales Quell-Label fuer Fehlermeldungen; sonst der .gbc-Pfad.
    let source_label = args.get(2).cloned().unwrap_or_else(|| path.clone());
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Kann '{}' nicht lesen: {}", path, e);
            return ExitCode::from(1);
        }
    };
    run_gbc_text(&text, &source_label)
}

/// `gbrt --tokens <datei.gb>` -- lext die Quelldatei und gibt pro Token eine
/// JSON-Zeile `[TYP, wert, zeile]` aus (fuer Lexer-Parity gegen Python).
fn tokens_main(path: &str) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", path, e); return ExitCode::from(1); }
    };
    match lexer::dump_tokens_json(&source) {
        Ok(dump) => {
            let stdout = std::io::stdout();
            let mut h = stdout.lock();
            let _ = h.write_all(dump.as_bytes());
            let _ = h.flush();
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Lexer-Fehler {}:{}: {}", e.line, e.col, e.msg);
            ExitCode::from(2)
        }
    }
}

/// `gbrt --ast <datei.gb>` -- lext + parst und gibt den AST als JSON aus
/// (Parser-Parity gegen Python).
fn ast_main(path: &str) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", path, e); return ExitCode::from(1); }
    };
    match parser::dump_ast_json(&source) {
        Ok(json) => {
            let stdout = std::io::stdout();
            let mut h = stdout.lock();
            let _ = h.write_all(json.as_bytes());
            let _ = h.write_all(b"\n");
            let _ = h.flush();
            ExitCode::SUCCESS
        }
        Err(e) => { eprintln!("{}", e); ExitCode::from(2) }
    }
}

/// `gbrt --preprocess <datei.gb>` -- expandiert IMPORTs und gibt die gemergte
/// Quelle auf stdout aus (Merge-Parity gegen preprocess.process(), Stufe 4).
fn preprocess_main(path: &str) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", path, e); return ExitCode::from(1); }
    };
    let base = std::path::Path::new(path).parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    match preprocess::process(&source, &base) {
        Ok((merged, _mods)) => {
            let stdout = std::io::stdout();
            let mut h = stdout.lock();
            let _ = h.write_all(merged.as_bytes());
            let _ = h.flush();
            ExitCode::SUCCESS
        }
        Err(e) => { eprintln!("Preprocess {}: {}", e.line, e.msg); ExitCode::from(2) }
    }
}

/// Volle Front-End-Kette in Rust: Preprocess (IMPORT) -> Lexer -> Parser ->
/// Compiler. `base` = Verzeichnis fuer relative IMPORT-Pfade. Liefert das
/// `.gbc`-JSON oder einen Exit-Code (Fehler bereits auf stderr gemeldet).
/// Geteilt von `--runsrc`, `run` und `--export`.
fn compile_source(raw_source: &str, base: &std::path::Path, label: &str) -> Result<serde_json::Value, ExitCode> {
    // Fehler-Format `<label>:<zeile>: <msg>` -- so erkennt der Editor (Pattern
    // `(\S+\.gb):(\d+)`) die Zeile und macht sie klickbar (wie bei Laufzeitfehlern).
    let (source, imports) = match preprocess::process(raw_source, base) {
        Ok(r) => r,
        Err(e) => { eprintln!("{}:{}: Preprocess-Fehler: {}", label, e.line, e.msg); return Err(ExitCode::from(2)); }
    };
    let (ext_types, aliases) = preprocess::compile_env(&imports);
    let toks = match lexer::Lexer::new(&source).tokenize() {
        Ok(t) => t,
        Err(e) => { eprintln!("{}:{}: Lexer-Fehler ({}): {}", label, e.line, e.col, e.msg); return Err(ExitCode::from(2)); }
    };
    let ast = match parser::Parser::new(toks).parse() {
        Ok(a) => a,
        Err(e) => { eprintln!("{}:{}: Parse-Fehler ({}): {}", label, e.line, e.col, e.msg); return Err(ExitCode::from(2)); }
    };
    match compiler::compile_to_gbc(&ast, &ext_types, &aliases) {
        Ok(j) => Ok(j),
        Err(e) => { eprintln!("{}: Compile-Fehler: {}", label, e); Err(ExitCode::from(3)) }
    }
}

/// `gbrt --check <datei.gb>` -- Front-End-Diagnostik fuer Editor-Live-Error-Check
/// und LSP. Gibt ein JSON-Array `[{line,col,severity,phase,message}]` auf stdout
/// aus (leer = fehlerfrei). Exit 0 auch bei gefundenen Problemen; nur ein
/// I/O-Fehler liefert Exit 1. Zeilen beziehen sich auf die GEMERGTE Quelle
/// (nach IMPORT-Expansion) -- der Editor mappt via origins zurueck.
fn check_main(path: &str) -> ExitCode {
    let raw_source = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", path, e); return ExitCode::from(1); }
    };
    let base = std::path::Path::new(path).parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let diags = check_source(&raw_source, &base);
    println!("{}", serde_json::to_string(&diags).unwrap_or_else(|_| "[]".into()));
    ExitCode::SUCCESS
}

/// Front-End-Kette (preprocess -> lex -> parse -> compile) als Diagnostik.
/// MVP: bricht bei der ersten Fehlerstelle ab und liefert genau eine Diagnose
/// (wie error_check.py einen ParseProblem liefert). Leeres Array = fehlerfrei.
/// Compiler-Fehler tragen heute keine Zeile -> `line: 0` (spaetere Verfeinerung).
fn check_source(raw_source: &str, base: &std::path::Path) -> Vec<serde_json::Value> {
    let (source, imports) = match preprocess::process(raw_source, base) {
        Ok(r) => r,
        Err(e) => return vec![serde_json::json!({
            "line": e.line, "col": 0, "severity": "error",
            "phase": "preprocess", "message": e.msg })],
    };
    let (ext_types, aliases) = preprocess::compile_env(&imports);
    let toks = match lexer::Lexer::new(&source).tokenize() {
        Ok(t) => t,
        Err(e) => return vec![serde_json::json!({
            "line": e.line, "col": e.col, "severity": "error",
            "phase": "lex", "message": e.msg })],
    };
    let ast = match parser::Parser::new(toks).parse() {
        Ok(a) => a,
        Err(e) => return vec![serde_json::json!({
            "line": e.line, "col": e.col, "severity": "error",
            "phase": "parse", "message": e.msg })],
    };
    match compiler::compile_to_gbc(&ast, &ext_types, &aliases) {
        Ok(_) => vec![],
        Err(e) => vec![serde_json::json!({
            "line": 0, "col": 0, "severity": "error",
            "phase": "compile", "message": e })],
    }
}

/// Front-End-Kette + Ausfuehrung. `label` = Quell-Label fuer Laufzeitfehler.
fn compile_and_run_source(raw_source: &str, base: &std::path::Path, label: &str) -> ExitCode {
    match compile_source(raw_source, base, label) {
        Ok(json) => run_program_value(json, label),
        Err(code) => code,
    }
}

/// `gbrt --runsrc <datei.gb>` -- volle Front-End-Kette in Rust, OHNE chdir
/// (Dev-/Parity-Einstieg: Output gegen Python-Tree-Walker, Stufe 3/4).
fn runsrc_main(path: &str) -> ExitCode {
    let raw_source = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", path, e); return ExitCode::from(1); }
    };
    let base = std::path::Path::new(path).parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    compile_and_run_source(&raw_source, &base, path)
}

/// `gbrt run <datei.gb>` (Stufe 5) -- eigenstaendiger End-to-End-Lauf aus
/// Quelltext, ohne Python. Wechselt wie `gbrun.py` ins Verzeichnis der Datei,
/// damit relative Asset-Pfade (`LOADIMAGE("assets/...")`) stimmen; Label fuer
/// Laufzeitfehler ist der Dateiname.
fn run_main(path: &str) -> ExitCode {
    let abs = std::fs::canonicalize(path)
        .unwrap_or_else(|_| std::path::PathBuf::from(path));
    let base = abs.parent().map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let label = abs.file_name().map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    let raw_source = match std::fs::read_to_string(&abs) {
        Ok(t) => t,
        Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", path, e); return ExitCode::from(1); }
    };
    // Ins Datei-Verzeichnis wechseln (wie gbrun.py os.chdir(file.parent)).
    let _ = std::env::set_current_dir(&base);
    compile_and_run_source(&raw_source, &base, &label)
}

/// Verzeichnis rekursiv kopieren (std hat keine Funktion dafuer).
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// `gbrt --export <datei.gb> [out_dir]` -- buendelt das Programm zu einer
/// eigenstaendigen Exe (Selbst-Export ohne Python): kompiliert Quelltext ->
/// .gbc und haengt den Payload (gbc + Footer `[u64 len][GBRTPAY1]`) an eine
/// Kopie der EIGENEN Runtime-Exe. `assets/` neben der Quelle wird mitkopiert.
/// Pendant zu gamebasic/export.py.
fn export_main(path: &str, out_dir: Option<&str>) -> ExitCode {
    let abs = std::fs::canonicalize(path)
        .unwrap_or_else(|_| std::path::PathBuf::from(path));
    let base = abs.parent().map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let stem = abs.file_stem().map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "programm".into());
    let raw_source = match std::fs::read_to_string(&abs) {
        Ok(t) => t,
        Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", path, e); return ExitCode::from(1); }
    };
    // 1) Quelltext -> .gbc-JSON (kompakt).
    let label = abs.file_name().map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    let json = match compile_source(&raw_source, &base, &label) {
        Ok(j) => j,
        Err(code) => return code,
    };
    let gbc_bytes = serde_json::to_string(&json).unwrap_or_default().into_bytes();
    // 2) Eigene Exe lesen + Payload anhaengen.
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => { eprintln!("current_exe: {}", e); return ExitCode::from(1); }
    };
    let mut bundle = match std::fs::read(&exe) {
        Ok(b) => b,
        Err(e) => { eprintln!("Kann Runtime '{}' nicht lesen: {}", exe.display(), e); return ExitCode::from(1); }
    };
    bundle.extend_from_slice(&gbc_bytes);
    bundle.extend_from_slice(&(gbc_bytes.len() as u64).to_le_bytes());
    bundle.extend_from_slice(PAYLOAD_MAGIC);
    // 3) Zielordner + Exe-Name.
    let out = match out_dir {
        Some(d) => std::path::PathBuf::from(d),
        None => base.join(format!("{}_dist", stem)),
    };
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("Kann Ausgabeverzeichnis '{}' nicht anlegen: {}", out.display(), e);
        return ExitCode::from(1);
    }
    let exe_suffix = if exe.extension().map(|e| e == "exe").unwrap_or(false) || cfg!(windows) {
        ".exe"
    } else { "" };
    let out_exe = out.join(format!("{}{}", stem, exe_suffix));
    if let Err(e) = std::fs::write(&out_exe, &bundle) {
        eprintln!("Kann '{}' nicht schreiben: {}", out_exe.display(), e);
        return ExitCode::from(1);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&out_exe, std::fs::Permissions::from_mode(0o755));
    }
    // 4) assets/ neben der Quelle mitkopieren (Konvention).
    let assets = base.join("assets");
    if assets.is_dir() {
        if let Err(e) = copy_dir_recursive(&assets, &out.join("assets")) {
            eprintln!("Warnung: assets/ nicht vollstaendig kopiert: {}", e);
        }
    }
    println!("Exportiert: {}", out_exe.display());
    ExitCode::SUCCESS
}

/// Laedt eine `.gbc` (JSON-Text) und fuehrt sie aus. Geteilt zwischen Dev-Modus
/// (Datei aus Argumenten) und Bundle-Modus (eingebettet in die Exe).
fn run_gbc_text(text: &str, source_label: &str) -> ExitCode {
    let json: serde_json::Value = match serde_json::from_str(text) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("JSON-Fehler in '{}': {}", source_label, e);
            return ExitCode::from(1);
        }
    };
    run_program_value(json, source_label)
}

/// Laedt ein bereits geparstes `.gbc`-JSON-`Value` und fuehrt es aus.
fn run_program_value(json: serde_json::Value, source_label: &str) -> ExitCode {
    let prog = match model::load_program(&json) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Lade-Fehler: {}", e);
            return ExitCode::from(1);
        }
    };

    let mut machine = vm::Vm::new(&prog);
    match machine.run() {
        Ok(()) => {
            let out = machine.take_output();
            // stdout schreiben (Output wird gepuffert, damit es genau einmal
            // und ohne Zwischen-Flush-Artefakte erscheint).
            let stdout = std::io::stdout();
            let mut h = stdout.lock();
            let _ = h.write_all(out.as_bytes());
            let _ = h.flush();
            ExitCode::SUCCESS
        }
        Err(e) => {
            // Zeile VOR take_output() lesen (take_output konsumiert die VM).
            let line = machine.error_line();
            // Bei Laufzeitfehler trotzdem bisherige Ausgabe zeigen.
            let out = machine.take_output();
            let stdout = std::io::stdout();
            let mut h = stdout.lock();
            let _ = h.write_all(out.as_bytes());
            let _ = h.flush();
            if line != 0 {
                eprintln!("Laufzeitfehler in {}:{}: {}", source_label, line, e);
            } else {
                eprintln!("Laufzeitfehler in {}: {}", source_label, e);
            }
            ExitCode::from(2)
        }
    }
}
