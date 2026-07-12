//! `gbrt` -- die native GameBasic-Runtime (und seit Stufe B die EINZIGE).
//!
//! Fuehrt `.gb`-Quelltext end-to-end aus (preprocess -> lex -> parse ->
//! compile -> VM, alles in Rust) oder eine fertige `.gbc`-Datei (JSON-
//! Bytecode, vom eigenen Compiler bzw. via `--export` erzeugt). Korrektheit
//! sichern die run_gb-Golden-Tests (tests/) -- die fruehere Python-Referenz
//! (Tree-Walker) ist geloescht.
//!
//! Verwendung: gbrt <datei.gbc> [quell-label]
//!
//! Das optionale `quell-label` (z.B. `spiel.gb`) wird nur fuer Laufzeitfehler-
//! Meldungen genutzt (`Laufzeitfehler in spiel.gb:Zeile: ...`). `gbrun.py
//! --native` reicht den Namen der `.gb`-Quelldatei durch.

mod animfsm;
mod ast;
mod astar;
// Audio-Backend (Kira/cpal, src/audio.rs) -- mit `graphics` eingebunden
// (raylib bleibt fuer Fenster/Input).
#[cfg(feature = "graphics")]
mod audio;
mod builtins;
mod compiler;
mod parser;
mod controller;
#[cfg(feature = "http")]
mod cloud;
#[cfg(feature = "db")]
mod db;
mod ecs;
#[cfg(feature = "http")]
mod html;
#[cfg(feature = "net")]
mod net;
mod text_stream;
#[cfg(feature = "bt")]
mod bt;
#[cfg(feature = "serial")]
mod serial;
#[cfg(feature = "usb")]
mod usb;
#[cfg(feature = "wifi")]
mod wifi;
#[cfg(feature = "graphics")]
mod filedialog;
#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
mod gui;
mod lexer;
mod model;
mod physics;
mod physics2d;
mod physics3d;
mod preprocess;
mod tiled;
mod timer;
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
        // Stufe B (Phase 3): `gbrt profile datei.gb` -- instrumentierter Lauf,
        // gibt pro-Zeile Count+Zeit als JSON-Blob aus (Editor aggregiert pro
        // Scope via symbols.scan_scopes). Ersetzt den Tree-Walker-Profiler.
        if raw.len() >= 3 && raw[1] == "profile" {
            // `--stoppable`: Editor-Stop-Button via stdin-Zeile/EOF (haelt stdin
            // belegt -> nur wenn der Aufrufer das will; ein direkter Terminal-Lauf
            // ohne Flag laesst stdin fuer INPUT frei).
            let stoppable = raw[2..].iter().any(|a| a == "--stoppable");
            match raw[2..].iter().find(|a| !a.starts_with("--")) {
                Some(p) => return profile_main(p, stoppable),
                None => { eprintln!("profile: keine Datei angegeben"); return ExitCode::from(1); }
            }
        }
        // Stufe B (Phase 3c): `gbrt debug datei.gb` -- interaktiver Debugger ueber
        // ein newline-JSON-Protokoll (stdin: Kommandos, stdout: Events). Ersetzt
        // den Tree-Walker-Debugger.
        if raw.len() >= 3 && raw[1] == "debug" {
            return debug_main(&raw[2]);
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
    // E1: Hardware-Module (serial/usb/bt/wifi) sind zwar importierbar, fehlen aber
    // im Default-Build. Frueh (beim IMPORT) warnen statt erst beim ersten Aufruf --
    // die Meldung ist nicht fatal, der Lauf geht weiter (der eigentliche Aufruf
    // wirft dann wie gehabt, falls das Modul wirklich genutzt wird).
    for m in preprocess::missing_hardware_modules(&imports) {
        eprintln!("{}: Warnung: {}", label, preprocess::hardware_missing_msg(m));
    }
    let toks = match lexer::Lexer::new(&source).tokenize() {
        Ok(t) => t,
        Err(e) => { eprintln!("{}:{}: Lexer-Fehler ({}): {}", label, e.line, e.col, e.msg); return Err(ExitCode::from(2)); }
    };
    let ast = match parser::Parser::new(toks).parse() {
        Ok(a) => a,
        Err(e) => { eprintln!("{}:{}: Parse-Fehler ({}): {}", label, e.line, e.col, e.msg); return Err(ExitCode::from(2)); }
    };
    match compiler::compile_to_gbc(&ast, &ext_types, &aliases) {
        Ok((j, warns)) => {
            // Nicht-fatale Compile-Warnungen (z.B. unbekanntes Builtin) vor dem
            // Lauf auf stderr -- der Lauf geht weiter, schlaegt aber spaeter ggf.
            // beim Aufruf fehl.
            for (line, msg) in warns {
                eprintln!("{}:{}: Warnung: {}", label, line, msg);
            }
            Ok(j)
        }
        Err((line, msg)) => {
            if line > 0 { eprintln!("{}:{}: Compile-Fehler: {}", label, line, msg); }
            else { eprintln!("{}: Compile-Fehler: {}", label, msg); }
            Err(ExitCode::from(3))
        }
    }
}

/// `gbrt profile <datei.gb>` -- fuehrt das Programm instrumentiert aus und gibt
/// einen JSON-Blob `{total_time, output, lines:[{line,count,time}], stopped}` auf
/// stdout aus. Programm-Output landet im `output`-Feld (kein stdout-Konflikt);
/// Laufzeitfehler kommen als `error`/`error_line` mit ins JSON. Exit 0 (der Editor
/// parst das JSON). chdir ins Datei-Verzeichnis wie `run`.
///
/// `stoppable`: ein Hintergrund-Thread liest stdin; sobald eine Zeile kommt (der
/// Editor schreibt `"stop"`) oder die Pipe EOF erreicht, wird das Programm beim
/// naechsten Zeilenwechsel sauber abgebrochen und die bis dahin gesammelten
/// Profile-Daten trotzdem ausgegeben (`stopped:true`). Damit lassen sich
/// Endlos-Loops (Grafik-Render-Loop, `WHILE TRUE`) profilieren, ohne dass ein
/// harter Prozess-Kill die Auswertung verschluckt.
fn profile_main(path: &str, stoppable: bool) -> ExitCode {
    let abs = std::fs::canonicalize(path).map(strip_extended_prefix).unwrap_or_else(|_| std::path::PathBuf::from(path));
    let base = abs.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."));
    let label = abs.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string());
    let raw_source = match std::fs::read_to_string(&abs) {
        Ok(t) => t,
        Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", path, e); return ExitCode::from(1); }
    };
    let _ = std::env::set_current_dir(&base);
    let json = match compile_source(&raw_source, &base, &label) {
        Ok(j) => j,
        Err(code) => return code,
    };
    let prog = match model::load_program(&json) {
        Ok(p) => p,
        Err(e) => { eprintln!("Lade-Fehler: {}", e); return ExitCode::from(1); }
    };
    let mut machine = vm::Vm::new(&prog);
    machine.enable_profiler();
    if stoppable {
        // Stop-Signal: Hintergrund-Thread blockiert auf stdin; die erste Zeile
        // (Editor schreibt "stop") oder EOF setzt das Flag -> sauberer Abbruch.
        // Detached -- bei Programm-Ende beendet der Prozess-Exit den Thread.
        use std::io::BufRead;
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let reader = flag.clone();
        std::thread::spawn(move || {
            let mut line = String::new();
            let _ = std::io::stdin().lock().read_line(&mut line);
            reader.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        machine.set_stop_flag(flag);
    }
    let run_res = machine.run();
    let stopped = matches!(&run_res, Err(e) if vm::Vm::was_stopped(e));
    let (total, lines) = machine.take_profile();
    let err_line = machine.error_line();
    let output = machine.take_output();
    let lines_json: Vec<serde_json::Value> = lines.iter()
        .map(|&(ln, c, t)| serde_json::json!({"line": ln, "count": c, "time": t}))
        .collect();
    let mut blob = serde_json::json!({
        "total_time": total, "output": output, "lines": lines_json, "stopped": stopped
    });
    // Echte Laufzeitfehler ins JSON -- der Stop-Sentinel ist KEIN Fehler.
    if let Err(e) = &run_res {
        if !vm::Vm::was_stopped(e) {
            blob["error"] = serde_json::json!(e);
            blob["error_line"] = serde_json::json!(err_line);
        }
    }
    println!("{}", serde_json::to_string(&blob).unwrap_or_else(|_| "{}".into()));
    ExitCode::SUCCESS
}

/// `gbrt debug <datei.gb>` -- interaktiver Debugger. Spricht ein
/// newline-delimited JSON-Protokoll: stdin = Kommandos ({"cmd":"continue"|
/// "step-over"|"step-into"|"step-out"|"stop"|"set-breakpoints"|"eval", ...}),
/// stdout = Events ({"event":"paused|output|eval-result|eval-error|finished|
/// error", ...}). Haelt initial an der ersten Zeile. chdir wie `run`.
fn debug_main(path: &str) -> ExitCode {
    let abs = std::fs::canonicalize(path).map(strip_extended_prefix).unwrap_or_else(|_| std::path::PathBuf::from(path));
    let base = abs.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."));
    let label = abs.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string());
    let raw_source = match std::fs::read_to_string(&abs) {
        Ok(t) => t,
        Err(e) => { eprintln!("Kann '{}' nicht lesen: {}", path, e); return ExitCode::from(1); }
    };
    let _ = std::env::set_current_dir(&base);
    let json = match compile_source(&raw_source, &base, &label) {
        Ok(j) => j,
        Err(code) => return code,
    };
    let prog = match model::load_program(&json) {
        Ok(p) => p,
        Err(e) => { eprintln!("Lade-Fehler: {}", e); return ExitCode::from(1); }
    };
    let mut machine = vm::Vm::new(&prog);
    machine.enable_debug();
    let res = machine.run();
    machine.debug_flush_output();
    let ev = match &res {
        Ok(()) => serde_json::json!({"event": "finished", "reason": "done"}),
        Err(e) if e == "__DEBUG_STOP__" =>
            serde_json::json!({"event": "finished", "reason": "stopped"}),
        Err(e) => serde_json::json!({
            "event": "error", "line": machine.error_line(), "message": e }),
    };
    println!("{}", serde_json::to_string(&ev).unwrap_or_default());
    ExitCode::SUCCESS
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
        // E1: Bei fehlerfreiem Compile noch Warnungen fuer IMPORTs von
        // Hardware-Modulen ergaenzen, die in diesem gbrt-Build fehlen -- damit
        // der Editor das schon auf der IMPORT-Zeile markiert (nicht erst beim
        // Lauf). Leer, wenn kein solches Modul importiert wird.
        Ok((_, warns)) => {
            let mut diags: Vec<serde_json::Value> =
                preprocess::missing_hardware_imports_with_lines(raw_source).into_iter()
                    .map(|(line, m)| serde_json::json!({
                        "line": line, "col": 0, "severity": "warning",
                        "phase": "preprocess", "message": preprocess::hardware_missing_msg(m) }))
                    .collect();
            // G1 (systemisch): Aufrufe von Builtins, die gbrt nicht kennt, schon
            // im Editor als Warnung zeigen -- statt erst zur Laufzeit zu crashen.
            for (line, msg) in warns {
                diags.push(serde_json::json!({
                    "line": line, "col": 0, "severity": "warning",
                    "phase": "compile", "message": msg }));
            }
            diags
        }
        Err((line, msg)) => vec![serde_json::json!({
            "line": line, "col": 0, "severity": "error",
            "phase": "compile", "message": msg })],
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
    let abs = std::fs::canonicalize(path).map(strip_extended_prefix)
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

/// Entfernt den Windows-Extended-Length-Prefix (`\\?\` bzw. `\\?\UNC\`) von
/// einem kanonisierten Pfad. `std::fs::canonicalize` liefert auf Windows immer
/// einen `\\?\`-Pfad. Als `cwd` gesetzt vergiftet dieser Prefix raylibs
/// `CORE.Storage.basePath` -> jede C-`fopen`/stb_image-FILEIO (Screenshots,
/// ExportImage, relative Asset-Pfade) schlaegt fehl, weil `\\?\`-Pfade dort
/// nicht geoeffnet werden koennen. Auf Nicht-Windows ein No-Op.
pub(crate) fn strip_extended_prefix(p: std::path::PathBuf) -> std::path::PathBuf {
    #[cfg(windows)]
    {
        if let Some(s) = p.to_str() {
            if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
                return std::path::PathBuf::from(format!(r"\\{}", rest));
            }
            if let Some(rest) = s.strip_prefix(r"\\?\") {
                return std::path::PathBuf::from(rest);
            }
        }
    }
    p
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
    // 4b) Zusaetzlich alle im Quelltext referenzierten Dateien einsammeln --
    // auch ueber `../` (z.B. LOADIMAGE("../assets/sprites/x.png")). Sie werden
    // mit abgestreiftem `../` ins Bundle gelegt; zur Laufzeit findet die
    // Pfad-Aufloesung (resolve_asset_path) sie dort wieder.
    let n = bundle_referenced_assets(&raw_source, &base, &out);
    if n > 0 {
        println!("  {} referenzierte Asset-Datei(en) mitkopiert.", n);
    }
    println!("Exportiert: {}", out_exe.display());
    ExitCode::SUCCESS
}

/// Streift fuehrende `../` / `./` (und `\`-Varianten) von einem relativen Pfad
/// ab -> die Position INNERHALB des Bundles.
fn strip_parent_prefix(p: &str) -> String {
    let mut s = p.replace('\\', "/");
    loop {
        if let Some(t) = s.strip_prefix("../").map(str::to_string) { s = t; }
        else if let Some(t) = s.strip_prefix("./").map(str::to_string) { s = t; }
        else { break; }
    }
    s
}

/// Liefert alle String-Literale (Inhalt zwischen `"..."`, mit GB-Escape `""`).
fn string_literals(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    let mut cur = String::new();
    let mut in_str = false;
    while let Some(c) = chars.next() {
        if in_str {
            if c == '"' {
                if chars.peek() == Some(&'"') { cur.push('"'); chars.next(); }
                else { out.push(std::mem::take(&mut cur)); in_str = false; }
            } else {
                cur.push(c);
            }
        } else if c == '"' {
            in_str = true;
            cur.clear();
        }
    }
    out
}

/// Kopiert alle im Quelltext als String-Literal referenzierten, relativ zu
/// `base` existierenden Dateien/Ordner ins Bundle (`out`), mit abgestreiftem
/// `../`. Liefert die Anzahl kopierter Dateien.
fn bundle_referenced_assets(source: &str, base: &std::path::Path, out: &std::path::Path) -> usize {
    let mut seen = std::collections::HashSet::new();
    let mut count = 0usize;
    for lit in string_literals(source) {
        if lit.is_empty() || lit.len() > 400 { continue; }
        // Absolute Pfade ueberspringen (nicht buendelbar): /... oder C:\...
        if lit.starts_with('/') || lit.starts_with('\\') { continue; }
        let b = lit.as_bytes();
        if b.len() >= 2 && b[1] == b':' { continue; }   // C:\ / D:/
        let src = base.join(&lit);
        if !src.exists() { continue; }
        let rel = strip_parent_prefix(&lit);
        if rel.is_empty() || !seen.insert(rel.clone()) { continue; }
        let dst = out.join(&rel);
        if src.is_dir() {
            if copy_dir_recursive(&src, &dst).is_ok() { count += 1; }
        } else {
            if let Some(p) = dst.parent() { let _ = std::fs::create_dir_all(p); }
            if std::fs::copy(&src, &dst).is_ok() { count += 1; }
        }
    }
    count
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
