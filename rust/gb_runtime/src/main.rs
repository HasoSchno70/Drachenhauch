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
mod builtins;
mod ecs;
#[cfg(feature = "graphics")]
mod graphics;
mod model;
mod value;
mod vm;

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
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
    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("JSON-Fehler in '{}': {}", path, e);
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
