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

mod astar;
#[cfg(feature = "graphics")]
mod audio;
mod builtins;
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

    // WASM/Web (emscripten): die .gbc liegt unter einem festen Pfad im
    // virtuellen FS (vom Build via --embed-file program.gbc eingebettet bzw.
    // vom JS-Harness per FS.writeFile reingeschrieben). Siehe web/ + docs.
    #[cfg(target_os = "emscripten")]
    {
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
