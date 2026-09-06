//! `dhrt` -- die native Drachenhauch-Runtime (und seit Stufe B die EINZIGE).
//!
//! Fuehrt `.dh`-Quelltext end-to-end aus (preprocess -> lex -> parse ->
//! compile -> VM, alles in Rust) oder eine fertige `.dhc`-Datei (JSON-
//! Bytecode, vom eigenen Compiler bzw. via `--export` erzeugt). Korrektheit
//! sichern die run_gb-Golden-Tests (tests/) -- die fruehere Python-Referenz
//! (Tree-Walker) ist geloescht.
//!
//! Verwendung: dhrt <datei.dhc> [quell-label]
//!
//! Das optionale `quell-label` (z.B. `spiel.dh`) wird nur fuer Laufzeitfehler-
//! Meldungen genutzt (`Laufzeitfehler in spiel.dh:Zeile: ...`). `dhrun.py
//! --native` reicht den Namen der `.dh`-Quelldatei durch.

mod animfsm;
mod ast;
mod namensraum;
mod astar;
// Audio-Backend (Kira/cpal, src/audio.rs) -- mit `graphics` eingebunden
// (raylib bleibt fuer Fenster/Input).
#[cfg(feature = "graphics")]
mod audio;
#[cfg(feature = "graphics")]
mod sprache;
// Im Browser gibt es weder cpal-Host noch Audio-Thread -- die Ausgabe laeuft
// ueber emscriptens OpenAL (das intern WebAudio bedient).
#[cfg(all(feature = "graphics", target_os = "emscripten"))]
mod web_audio;
mod chart;
mod csv;
mod kodierung;
mod ini;
mod xml;
mod pdf;
mod pdf_masse;
mod xlsx;
mod gifschreiber;
mod geld;
mod httpd;
#[cfg(feature = "smtp")]
mod smtp;
mod zipdatei;
mod builtins;
mod compiler;
mod parser;
mod controller;
#[cfg(feature = "http")]
mod cloud;
#[cfg(feature = "db")]
mod db;
mod ecs;
// Hintergrund-Auftraege (WP H) -- ungated: SHELL_START braucht kein http.
mod hintergrund;
// Fenster als Prozess (docs/entwurf-native-fenster.md, Weg B) -- ungated wie hintergrund.
mod fenster;
// Drucken + OPENDOC (docs/entwurf-drucken.md, Wege A und C) -- ungated.
mod drucken;
#[cfg(feature = "http")]
mod html;
#[cfg(feature = "net")]
mod net;
#[cfg(feature = "net")]
mod mqtt;
mod farbraum;
mod kalender;
mod syntax;
mod symbole;
mod lsp;
mod doku;
mod pruef;
mod text_stream;
#[cfg(feature = "bt")]
mod bt;
// midi.rs ist NICHT gegatet: die beiden Umrechner darin (Notenname,
// Frequenz) brauchen kein Geraet und sollen in jedem Bau stehen. Nur der
// Geraeteteil in der Datei haengt am Feature.
mod midi;
#[cfg(feature = "serial")]
mod serial;
#[cfg(feature = "serial")]
mod firmata;
#[cfg(feature = "usb")]
mod usb;
#[cfg(feature = "wifi")]
mod wifi;
#[cfg(feature = "dialogs")]
mod filedialog;
#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
mod gui;
// Barrierefreiheit: eigenes Feature (vom graphics-Feature eingeschaltet), damit
// die macOS-/Linux-Adapter mit `cargo check --features a11y --target ...`
// ohne raylib-Bau pruefbar sind. Ohne Grafik wird das Modul nicht benutzt.
#[cfg(feature = "a11y")]
#[allow(dead_code)]
mod a11y;
#[cfg(feature = "graphics")]
mod ime;
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
mod zeit;

use std::io::Write;
use std::process::ExitCode;

/// Magic am Ende einer gebundelten `.exe`. Layout der letzten 16 Bytes:
/// `[u64 Laenge der .dhc-Bytes, little-endian][8 Byte Magic]`. Die `.dhc`-Bytes
/// liegen direkt vor diesem Footer. So wird `dhrt` selbst zur Spiel-Exe:
/// `export.py` haengt `<gbc><laenge><magic>` an eine Kopie von `dhrt.exe`.
const PAYLOAD_MAGIC: &[u8; 8] = b"DHRTPAY1";

/// Liest eine eingebettete `.dhc` aus der eigenen Exe (Bundle-Modus) -- oder
/// None, wenn kein Payload angehaengt ist (normaler Dev-Modus).
///
/// Sucht die Kennung RUECKWAERTS, statt sie in den letzten 16 Bytes zu
/// erwarten. Der Grund ist das Signieren: `signtool` (und `codesign` auf
/// macOS) haengt den Zertifikatsblock ans Dateiende -- danach sind die letzten
/// 16 Bytes nicht mehr unsere, und ein exportiertes Spiel fand sich selbst
/// nicht mehr (es verhielt sich wieder wie ein blankes `dhrt`). Andersherum
/// geht es nicht: erst signieren, dann anhaengen zerstoert jede Signatur --
/// empirisch geprueft, aus `Valid` wird `NotSigned`. Also muss die Reihenfolge
/// „exportieren, dann signieren" moeglich sein, und dafuer darf der Footer
/// nicht am Dateiende kleben.
/// Quelltext oder Bytecode? Entschieden wird das allein an der Endung.
///
/// Die alte Endung bleibt neben `.dh` gueltig: sie stammt aus der
/// GameBasic-Zeit, und wer noch ein altes Programm herumliegen hat, soll es
/// starten koennen, ohne es vorher umzubenennen. Geschrieben wird nur `.dh`.
///
/// Die alte Endung steht hier bewusst zusammengesetzt und nicht als Literal:
/// eine Massenersetzung ".gb" -> ".dh" hat diese Funktion beim Umbenennen
/// schon einmal zu `.dh || .dh` verstuemmelt.
fn ist_quelldatei(pfad: &str) -> bool {
    let p = pfad.to_lowercase();
    p.ends_with(".dh") || p.ends_with(ALTE_QUELL_ENDUNG)
}

/// `.gb` -- getrennt gehalten, siehe `ist_quelldatei`.
const ALTE_QUELL_ENDUNG: &str = ".gb";

fn embedded_gbc() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let data = std::fs::read(&exe).ok()?;
    embedded_gbc_in(&data)
}

/// Der reine Teil davon -- ohne Dateizugriff, damit er testbar ist.
fn embedded_gbc_in(data: &[u8]) -> Option<String> {
    if data.len() < 16 { return None; }
    // Von hinten nach vorne. Der letzte Treffer ist der echte Footer: die
    // .dhc-Nutzlast liegt DAVOR, ein zufaelliges Vorkommen der acht Bytes im
    // JSON waere also weiter vorne. Trifft ein Kandidat nicht zu (etwa weil
    // die Bytes zufaellig im Signaturblock stehen), wird weiter vorne
    // gesucht, statt aufzugeben.
    let mut ende = data.len();
    while ende >= 16 {
        let pos = data[..ende].windows(8).rposition(|w| w == PAYLOAD_MAGIC)?;
        if pos >= 8 {
            let laengenfeld = pos - 8;
            let len = u64::from_le_bytes(
                data[laengenfeld..pos].try_into().ok()?) as usize;
            if len > 0 && len <= laengenfeld {
                if let Ok(s) = String::from_utf8(
                    data[laengenfeld - len..laengenfeld].to_vec()) {
                    return Some(s);
                }
            }
        }
        ende = pos;                     // strikt weiter vorne weitersuchen
    }
    None
}

/// WP A: die Argumente, die dem GB-PROGRAMM gehoeren (`ARGC`/`ARG$`), aus der
/// Runtime-Kommandozeile herausloesen -- alles hinter einem alleinstehenden
/// `--`.
///
/// Ohne `--` bekommt das Programm KEINE Argumente. Das ist Absicht: `dhrt run
/// spiel.dh --stoppable` soll `--stoppable` an die Runtime richten, nicht ans
/// Programm, und dhrt darf sich spaeter weitere Schalter zulegen, ohne dass ein
/// bestehendes Programm sie ploetzlich als seine liest.
///
/// **Nicht** fuer den Bundle-Modus (exportierte Exe) -- dort gibt es keine
/// Runtime-Argumente, also gehoeren alle dem Programm (siehe `main`).
fn setze_programm_args(raw: &[String]) {
    builtins::set_programm_args(match raw.iter().position(|s| s == "--") {
        Some(i) => raw[i + 1..].to_vec(),
        None => Vec::new(),
    });
}

/// Die Fassung, wie ein Mensch sie nennt: "2026.8" statt "2026.8.0".
///
/// Cargo verlangt drei Stellen; die dritte ist hier immer 0 und wuerde nur
/// verwirren. `tests/test_version.py` haelt Cargo.toml, pyproject.toml und
/// `drachenhauch/__init__.py` auf demselben Stand.
fn fassung() -> String {
    let v = env!("CARGO_PKG_VERSION");
    v.strip_suffix(".0").unwrap_or(v).to_string()
}

/// Welche Cargo-Features stecken in DIESEM Binary?
///
/// Der Grund fuer diese Zeile: ein Bau ohne `--hardware` laesst `serial`,
/// `usb`, `bt` und `wifi` weg, ohne dass man es dem Programm ansieht -- und
/// die Meldung kommt dann erst beim ersten Aufruf, tief im Programm. Wer
/// `dhrt --version` tippt, soll das in einer Zeile sehen koennen.
fn eingebaut() -> String {
    let mut an: Vec<&str> = Vec::new();
    let mut aus: Vec<&str> = Vec::new();
    for (name, da) in [
        ("grafik", cfg!(feature = "graphics")),
        ("dialoge", cfg!(feature = "dialogs")),
        ("datenbank", cfg!(feature = "db")),
        ("netz", cfg!(feature = "net")),
        ("http", cfg!(feature = "http")),
        ("seriell", cfg!(feature = "serial")),
        ("usb", cfg!(feature = "usb")),
        ("bluetooth", cfg!(feature = "bt")),
        ("wlan", cfg!(feature = "wifi")),
        ("midi", cfg!(feature = "midi")),
    ] {
        if da { an.push(name) } else { aus.push(name) }
    }
    let mut s = format!("dabei: {}", if an.is_empty() { "nichts".to_string() } else { an.join(", ") });
    if !aus.is_empty() {
        s.push_str(&format!("\nfehlt: {} (neu bauen mit: python rust/build_runtime.py --hardware)", aus.join(", ")));
    }
    s
}

const HILFE: &str = "\
dhrt -- die Drachenhauch-Runtime

  dhrt <datei.dh>              Programm ausfuehren (wie `run`)
  dhrt run <datei.dh> [-- ...] ausfuehren; alles hinter `--` gehoert dem Programm
  dhrt test [pfad ...]         Pruefprogramme suchen und laufen lassen
  dhrt fmt <datei ...>         Schluesselwoerter gross schreiben
                               (--einruecken richtet auch die Einrueckung,
                                --pruefen schreibt nicht, meldet nur)
  dhrt --check <datei.dh> ...  nur uebersetzen und Probleme als JSON melden
  dhrt --export <datei.dh>     zu einer eigenstaendigen .exe buendeln
  dhrt debug <datei.dh>        Debug-Sitzung (JSON-Protokoll, fuer den Editor)
  dhrt profile <datei.dh>      Laufzeit je Zeile messen
  dhrt lsp                     Sprachserver (LSP ueber stdin/stdout, fuer VS Code)
  dhrt doku <prosa|grammatik|referenz>  Doku-Werkzeuge (dhrt doku fuer die Uebersicht)
  dhrt pruef [bloecke|namen|zaehlungen|konstanten|pfade]  Doku gegen die Wirklichkeit
  dhrt --version               Fassung und eingebaute Bestandteile
  dhrt --help                  diese Uebersicht

Entwickler-Einstiege: --tokens, --ast, --dumpbc, --preprocess, --runsrc, call

Handbuch: docs/README.md";

fn main() -> ExitCode {
    // MILLIS/TIMER zaehlen ab hier: "seit Programmstart" soll auch dann
    // stimmen, wenn das Programm erst nach dem Laden von Bildern misst.
    builtins::uhr_starten();
    // Front-End-Debug: `dhrt --tokens <datei.dh>` gibt den Token-Strom als
    // kanonisches JSON aus (Parity-Vergleich mit dem Python-Lexer).
    {
        // Review-Fund: `std::env::args()` paniked bei nicht-UTF8-Argumenten
        // (auf Unix moeglich, z.B. ein Dateiname in Latin-1) -- `args_os()` +
        // verlustbehaftete Konvertierung crasht dort nie, sondern ersetzt nur
        // ungueltige Bytes.
        let raw: Vec<String> = std::env::args_os().map(|s| s.to_string_lossy().into_owned()).collect();
        // Bewusst in DIESEM Block (nur wenn keine Exe mit eingebettetem
        // Programm vorliegt): eine exportierte .exe IST das Programm, ihr
        // gehoeren alle Argumente -- `meinspiel.exe --version` soll das
        // Spiel starten und nicht die Runtime ausfragen.
        if raw.len() >= 2 && matches!(raw[1].as_str(), "--version" | "-V") {
            println!("dhrt {}", fassung());
            println!("{}", eingebaut());
            return ExitCode::SUCCESS;
        }
        if raw.len() >= 2 && matches!(raw[1].as_str(), "--help" | "-h" | "/?") {
            println!("{}", HILFE);
            return ExitCode::SUCCESS;
        }
        if raw.len() >= 3 && raw[1] == "--tokens" {
            return tokens_main(&raw[2]);
        }
        if raw.len() >= 3 && raw[1] == "--ast" {
            return ast_main(&raw[2]);
        }
        // Debug: kompiliert die Quelle und gibt das .dhc-JSON aus (Bytecode-Dump).
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
            return check_main(&raw[2..]);
        }
        // Stufe 4: IMPORT-Preprocessor -- gibt die gemergte Quelle aus
        // (Merge-Parity gegen preprocess.process()).
        if raw.len() >= 3 && raw[1] == "--preprocess" {
            return preprocess_main(&raw[2]);
        }
        // Stufe 3: Quelltext in Rust kompilieren + ausfuehren (Output-Parity).
        if raw.len() >= 3 && raw[1] == "--runsrc" {
            setze_programm_args(&raw);
            return runsrc_main(&raw[2]);
        }
        // Stufe 5: `dhrt run datei.dh` -- eigenstaendiger End-to-End-Lauf
        // (preprocess+lex+parse+compile+run, chdir ins Datei-Verzeichnis fuer
        // relative Asset-Pfade). dhrt ist damit ohne Python lauffaehig.
        // `dhrt fmt [--pruefen] <datei ...>` -- Einrueckung richten.
        if raw.len() >= 2 && raw[1] == "fmt" {
            let nur_pruefen = raw[2..].iter().any(|a| a == "--pruefen");
            let einruecken = raw[2..].iter().any(|a| a == "--einruecken");
            let dateien: Vec<String> = raw[2..].iter()
                .filter(|a| !a.starts_with("--")).cloned().collect();
            return fmt_main(&dateien, nur_pruefen, einruecken);
        }
        // `dhrt test [pfad ...]` -- Pruefprogramme suchen und laufen lassen.
        // Ohne Pfad das aktuelle Verzeichnis.
        if raw.len() >= 2 && raw[1] == "test" {
            return test_main(&raw[2..]);
        }
        // `dhrt lsp` -- der Sprachserver fuer VS Code und jede andere IDE.
        if raw.len() >= 2 && raw[1] == "lsp" {
            return lsp::serve();
        }
        // `dhrt doku prosa|grammatik|referenz` -- die Doku-Werkzeuge.
        if raw.len() >= 2 && raw[1] == "doku" {
            return doku::main(&raw[2..]);
        }
        // `dhrt pruef ...` -- die Doku-Pruefer.
        if raw.len() >= 2 && raw[1] == "pruef" {
            return pruef::main(&raw[2..]);
        }
        if raw.len() >= 3 && raw[1] == "run" {
            setze_programm_args(&raw);
            return run_main(&raw[2]);
        }
        // `dhrt call <datei.dh> <funktion> [argument]` -- EINE Funktion
        // ausfuehren, ohne das Hauptprogramm zu fahren. Grundlage fuer
        // TASK_START (Auftrag als eigener Prozess statt als Thread, weil
        // `Program` weder Send noch Sync ist). Ausgabe ist EINE JSON-Zeile,
        // wie bei --check/profile/debug -- der Aufrufer ist eine Maschine.
        if raw.len() >= 4 && raw[1] == "call" {
            setze_programm_args(&raw);
            return call_main(&raw[2], &raw[3], raw[4..].to_vec());
        }
        // Stufe B (Phase 3): `dhrt profile datei.dh` -- instrumentierter Lauf,
        // gibt pro-Zeile Count+Zeit als JSON-Blob aus (Editor aggregiert pro
        // Scope via symbols.scan_scopes). Ersetzt den Tree-Walker-Profiler.
        if raw.len() >= 3 && raw[1] == "profile" {
            // `--stoppable`: Editor-Stop-Button via stdin-Zeile/EOF (haelt stdin
            // belegt -> nur wenn der Aufrufer das will; ein direkter Terminal-Lauf
            // ohne Flag laesst stdin fuer INPUT frei).
            let stoppable = raw[2..].iter().any(|a| a == "--stoppable");
            setze_programm_args(&raw);
            match raw[2..].iter().find(|a| !a.starts_with("--")) {
                Some(p) => return profile_main(p, stoppable),
                None => { eprintln!("profile: keine Datei angegeben"); return ExitCode::from(1); }
            }
        }
        // Stufe B (Phase 3c): `dhrt debug datei.dh` -- interaktiver Debugger ueber
        // ein newline-JSON-Protokoll (stdin: Kommandos, stdout: Events). Ersetzt
        // den Tree-Walker-Debugger.
        if raw.len() >= 3 && raw[1] == "debug" {
            setze_programm_args(&raw);
            return debug_main(&raw[2]);
        }
        // Selbst-Export: `dhrt --export datei.dh [out_dir]` buendelt das
        // Programm aus Quelltext zu einer eigenstaendigen Exe (ohne Python).
        if raw.len() >= 3 && raw[1] == "--export" {
            // `--mit-daten` darf an beliebiger Stelle stehen und ist KEIN
            // Ausgabeverzeichnis -- sonst landete das Bundle in einem Ordner
            // namens "--mit-daten".
            let mit_daten = raw.iter().any(|a| a == "--mit-daten");
            let out = raw.iter().skip(3).find(|a| !a.starts_with("--")).map(|s| s.as_str());
            return export_main(&raw[2], out, mit_daten);
        }
    }

    // Bundle-Modus: eingebettete .dhc am Ende der eigenen Exe?
    if let Some(text) = embedded_gbc() {
        // WP A: die exportierte Exe IST das Programm -- es gibt keine
        // Runtime-Argumente, von denen zu trennen waere. Also gehoeren alle
        // hinter dem Programmnamen ihm, ohne `--`-Konvention (`meins.exe datei.csv`
        // soll einfach funktionieren). Deshalb steht der Aufruf hier und nicht
        // gemeinsam mit den `dhrt`-Unterbefehlen weiter oben.
        builtins::set_programm_args(std::env::args_os().skip(1)
            .map(|s| s.to_string_lossy().into_owned()).collect());
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
    // dhrt die `.dh`-QUELLE selbst kompilieren -> der Playground braucht KEIN
    // vorab-kompiliertes .dhc (und kein Pyodide) mehr. `/program.dh` (Quelle)
    // hat Vorrang; `/program.dhc` (vorkompiliert) bleibt als Fallback. Hier
    // steht bewusst KEINE Duldung der alten Namen: das virtuelle Dateisystem
    // wird bei jedem Bau frisch befuellt, ein altes `/program.gb` kann es
    // also gar nicht geben. Siehe web/ + docs/web-playground.md.
    #[cfg(target_os = "emscripten")]
    {
        if let Ok(src) = std::fs::read_to_string("/program.dh") {
            return compile_and_run_source(&src, std::path::Path::new("/"), "playground");
        }
        if let Ok(text) = std::fs::read_to_string("/program.dhc") {
            return run_gbc_text(&text, "playground");
        }
    }

    let args: Vec<String> = std::env::args_os().map(|s| s.to_string_lossy().into_owned()).collect();
    if args.len() < 2 {
        // Review-Fund: `args[0]` indexierte unbedingt, aber `args.len() < 2`
        // schliesst auch `len == 0` (leeres argv, z.B. via execve) ein --
        // ein zweiter Panic direkt im Fehlerpfad. `argv[0]` als Programmname
        // ist per Konvention ohnehin praesent, `.get(0)` mit Fallback ist der
        // billige, sichere Weg.
        eprintln!("Verwendung: {} <datei.dh|datei.dhc>", args.first().map(String::as_str).unwrap_or("dhrt"));
        return ExitCode::from(1);
    }
    let path = &args[1];
    // Hier ist klar: keine eingebettete Exe (Bundle-Zweig oben hat nicht
    // gegriffen), also ein Runtime-Aufruf -- `--`-Konvention.
    setze_programm_args(&args);
    // Komfort: `dhrt datei.dh` (ohne `run`) wird wie `dhrt run datei.dh`
    // behandelt -- aus Quelltext, mit chdir. `.dhc` laeuft den VM-Pfad.
    if ist_quelldatei(path) {
        return run_main(path);
    }
    // Optionales Quell-Label fuer Fehlermeldungen; sonst der .dhc-Pfad.
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

/// Was auf dem Block-Stapel des Formatierers liegt.
///
/// `Case` braucht eine eigene Marke, weil `END SELECT` ZWEI Ebenen schliesst:
/// den Rumpf des letzten CASE und das SELECT selbst. Genau dieser Fall hat
/// die erste, stapellose Fassung verraten -- sie schrieb die CASE-Zeilen eine
/// Ebene zu weit links und war damit in 26 Beispieldateien anderer Meinung
/// als der Hausstil.
#[derive(PartialEq, Clone, Copy)]
enum Block { Select, Case, Andere }

/// Ist dieses Token ein Schluesselwort (und nicht Name, Zahl, Text, Zeichen)?
fn ist_schluesselwort(t: lexer::Tt) -> bool {
    use lexer::Tt::*;
    !matches!(t,
        Number | Str | Ident | Newline | Eof
        | Plus | Minus | StarT | Slash | Intdiv | Caret | Eq | Neq | Lt | Gt
        | Leq | Geq | Lparen | Rparen | Lbracket | Rbracket | Lbrace | Rbrace
        | Comma | Dot | Semicolon | Colon | PlusEq | MinusEq | StarEq | SlashEq
        | Ellipsis)
}

/// Schluesselwoerter gross schreiben -- und sonst NICHTS anfassen.
///
/// Das ist der Teil eines Formatierers, der **verlustfrei** ist: Drachenhauch
/// ignoriert Gross-/Kleinschreibung, also schreibt jeder anders (`If x Then`,
/// `if x then`, `IF x THEN`). Die Ersetzung geschieht an den Token-Positionen
/// des Lexers, nie am Text -- ein `end` in einer Zeichenkette oder einem
/// Kommentar bleibt damit unberuehrt.
///
/// Gross geschrieben wird das Wort, das DASTEHT (`elif` -> `ELIF`), nicht
/// sein kanonischer Name (`ELSEIF`): der Formatierer soll die Schreibweise
/// vereinheitlichen, nicht den Wortschatz.
fn schluesselwoerter_gross(quelle: &str, toks: &[lexer::Token]) -> Vec<String> {
    let mut zeilen: Vec<Vec<char>> = quelle.lines().map(|z| z.chars().collect()).collect();
    for t in toks {
        if !ist_schluesselwort(t.tt) { continue; }
        let Some(zeile) = zeilen.get_mut(t.line.saturating_sub(1)) else { continue };
        let start = t.col.saturating_sub(1);        // col ist 1-basiert
        // Das Wort an dieser Stelle abgreifen (Buchstaben/Ziffern/Unterstrich)
        // und in Grossbuchstaben zurueckschreiben. Gleiche Laenge, also
        // verschiebt sich nichts -- die uebrigen Token behalten ihre Spalte.
        let mut ende = start;
        while ende < zeile.len()
            && (zeile[ende].is_alphanumeric() || zeile[ende] == '_') { ende += 1; }
        if ende == start { continue; }
        let wort: String = zeile[start..ende].iter().collect();
        // GEGENPROBE: steht an dieser Stelle wirklich dieses Schluesselwort?
        //
        // Nicht jedes Token sitzt dort, wo sein Text steht: der Lexer loest
        // einen f-String noch waehrend des Lesens in eine ganze
        // Token-Folge auf (`("a" + STR$(x))`), und die geerbt alle die
        // Position des `f`. Ohne diese Probe wurde daraus `F"..."` -- in
        // 108_skeletal_anim.dh beim ersten Lauf ueber den Bestand
        // tatsaechlich passiert.
        if lexer::keyword(&wort.to_lowercase()) != Some(t.tt) { continue; }
        let gross: Vec<char> = wort.to_uppercase().chars().collect();
        // Nur ersetzen, wenn die Laenge stimmt: bei Sonderzeichen kann
        // to_uppercase laenger werden (das deutsche Eszett), und dann waere
        // jede folgende Spalte in dieser Zeile verschoben. Schluesselwoerter
        // sind reines ASCII, der Fall kann also nur bei einem Irrtum
        // auftreten -- dann lieber nichts tun.
        if gross.len() != ende - start { continue; }
        zeile[start..ende].copy_from_slice(&gross);
    }
    zeilen.into_iter().map(|z| z.into_iter().collect()).collect()
}

/// Neu einruecken -- nur auf ausdrueckliche Anforderung (`--einruecken`).
///
/// **Warum nicht von Haus aus:** der Formatierer kennt nur die Bloecke der
/// SPRACHE. Eine von Hand eingerueckte Gruppe, die die Sprache nicht kennt --
/// der Inhalt zwischen `RENDERTARGET_BEGIN` und `RENDERTARGET_END`, ein unter
/// seinem Vorgaenger ausgerichteter Kommentar -- wird dabei flachgezogen.
/// Beides ist in `examples/` echt vorgekommen. Ein Werkzeug, das die
/// Gliederungsabsicht seines Nutzers ueberschreibt, darf das nicht
/// nebenbei tun.
fn neu_einruecken(zeilen: &[String], toks: &[lexer::Token], einheit: &str) -> Vec<String> {
    use lexer::Tt::*;
    // LOGISCHE Zeilen aus dem Token-Strom bauen: eine Zeile, die per `_`
    // weitergeht, hat kein NEWLINE dazwischen -- ihre Fortsetzung liegt also
    // im selben Buendel.
    //
    // Bewusst NICHT am Text entschieden ("endet auf `_`"): ein Bezeichner
    // darf auf `_` enden (`DIM my_var`), und dann waere die naechste Zeile
    // faelschlich eine Fortsetzung. Der Lexer kennt den Unterschied, weil er
    // `_` nur direkt vor dem Zeilenende als Fortsetzung liest.
    struct LZeile { erste: usize, letzte: usize, toks: Vec<lexer::Tt> }
    let mut logisch: Vec<LZeile> = Vec::new();
    let mut offen: Option<LZeile> = None;
    for t in toks {
        match t.tt {
            Newline => {
                if let Some(mut z) = offen.take() { z.letzte = t.line.max(z.letzte); logisch.push(z); }
            }
            Eof => {}
            tt => match &mut offen {
                Some(z) => { z.letzte = t.line.max(z.letzte); z.toks.push(tt); }
                None => offen = Some(LZeile { erste: t.line, letzte: t.line, toks: vec![tt] }),
            },
        }
    }
    if let Some(z) = offen.take() { logisch.push(z); }
    let mut beginnt: std::collections::HashMap<usize, usize> = Default::default();
    for (i, z) in logisch.iter().enumerate() { beginnt.insert(z.erste, i); }

    let mut stapel: Vec<Block> = Vec::new();
    let mut raus: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < zeilen.len() {
        let nr = i + 1;                      // Zeilennummern sind 1-basiert
        let inhalt = zeilen[i].trim_end();
        let Some(&li) = beginnt.get(&nr) else {
            // Keine Anweisung hier: Leerzeile oder reiner Kommentar. Beide
            // bleiben, wie sie sind -- ein Kommentar steht oft mit Absicht,
            // wo er steht.
            raus.push(inhalt.to_string());
            i += 1;
            continue;
        };
        let z = &logisch[li];
        let erstes = z.toks.first().copied();

        // 1. Was schliesst diese Zeile, BEVOR sie selbst gesetzt wird?
        match erstes {
            Some(End) if z.toks.len() > 1 => {
                // Ein alleinstehendes END (Programmende) schliesst nichts.
                if z.toks.get(1) == Some(&Select) && stapel.last() == Some(&Block::Case) {
                    stapel.pop();
                }
                stapel.pop();
            }
            Some(Next) | Some(Wend) | Some(Until) => { stapel.pop(); }
            Some(Case) => { if stapel.last() == Some(&Block::Case) { stapel.pop(); } }
            _ => {}
        }

        // 2. ELSE/ELSEIF/CATCH/FINALLY gehoeren optisch zu ihrem Kopf, stehen
        //    also eine Ebene weiter links als der Rumpf.
        let ausruecken = matches!(erstes, Some(Else) | Some(Elseif) | Some(Catch) | Some(Finally));
        let tiefe = stapel.len().saturating_sub(if ausruecken { 1 } else { 0 });
        raus.push(format!("{}{}", einheit.repeat(tiefe), inhalt.trim()));

        // Fortsetzungszeilen unveraendert uebernehmen: wer seine Parameter
        // untereinander ausrichtet, hat sich etwas dabei gedacht.
        for k in (nr + 1)..=z.letzte.min(zeilen.len()) {
            raus.push(zeilen[k - 1].trim_end().to_string());
        }

        // 3. Was oeffnet diese Zeile?
        match erstes {
            Some(Select) => stapel.push(Block::Select),
            Some(Case) => stapel.push(Block::Case),
            Some(For) | Some(While) | Some(Sub) | Some(Function) | Some(Class)
            | Some(Struct) | Some(Try) | Some(With) | Some(Repeat)
            | Some(Property) | Some(Operator) => stapel.push(Block::Andere),
            // ENUM gibt es in zwei Formen: als Block (mit END ENUM) und
            // einzeilig (`ENUM State = MENU, PLAYING`). Nur die erste oeffnet
            // etwas -- das `=` unterscheidet sie.
            Some(Enum) => {
                if !z.toks.contains(&Eq) { stapel.push(Block::Andere); }
            }
            Some(If) => {
                // Ein einzeiliges IF (`IF x THEN y = 1`) oeffnet nichts --
                // erkennbar daran, dass hinter dem THEN noch etwas steht.
                if let Some(p) = z.toks.iter().position(|t| *t == Then) {
                    if p + 1 >= z.toks.len() { stapel.push(Block::Andere); }
                }
            }
            _ => {}
        }
        i = z.letzte.max(nr);
    }
    raus
}

/// Eine Quelle formatieren.
///
/// `None` heisst: die Datei liess sich nicht lexen (Syntaxfehler). Dann wird
/// nichts geschrieben -- an kaputtem Code herumzuruecken hilft niemandem.
fn formatiere(quelle: &str, einruecken: bool, einheit: &str) -> Option<String> {
    let toks = lexer::Lexer::new(quelle).tokenize().ok()?;
    let mut zeilen = schluesselwoerter_gross(quelle, &toks);
    if einruecken {
        zeilen = neu_einruecken(&zeilen, &toks, einheit);
    }
    // Leerraum am Zeilenende faellt immer weg -- er ist unsichtbar und
    // taucht in jedem Diff auf.
    Some(zeilen.iter().map(|z| z.trim_end()).collect::<Vec<_>>().join("\n"))
}

/// `dhrt fmt [--pruefen] [--einruecken] <datei ...>`
fn fmt_main(dateien: &[String], nur_pruefen: bool, einruecken: bool) -> ExitCode {
    if dateien.is_empty() {
        eprintln!("Verwendung: dhrt fmt [--pruefen] [--einruecken] <datei.dh ...>");
        return ExitCode::from(2);
    }
    let mut geaendert = 0usize;
    let mut fehler = 0usize;
    for d in dateien {
        let roh = match std::fs::read_to_string(d) {
            Ok(t) => t,
            Err(e) => { eprintln!("dhrt fmt: {}: {}", d, e); fehler += 1; continue; }
        };
        // Zeilenenden der Datei beibehalten: eine CRLF-Datei soll nicht
        // allein durch das Formatieren im Diff komplett neu erscheinen.
        let crlf = roh.contains("\r\n");
        let endet_mit_umbruch = roh.ends_with('\n');
        let Some(neu) = formatiere(&roh, einruecken, "    ") else {
            eprintln!("dhrt fmt: {}: laesst sich nicht lesen (Syntaxfehler?) -- unveraendert gelassen", d);
            fehler += 1;
            continue;
        };
        let mut neu = neu;
        if endet_mit_umbruch { neu.push('\n'); }
        let neu = if crlf { neu.replace('\n', "\r\n") } else { neu };
        if neu == roh { continue; }
        geaendert += 1;
        if nur_pruefen {
            println!("wuerde sich aendern: {}", d);
        } else if let Err(e) = std::fs::write(d, &neu) {
            eprintln!("dhrt fmt: {}: {}", d, e);
            fehler += 1;
        } else {
            println!("gerichtet: {}", d);
        }
    }
    if fehler > 0 { return ExitCode::from(2); }
    // `--pruefen` ist fuer eine Kette gedacht: "es gibt noch etwas zu tun"
    // muss sich am Rueckgabewert ablesen lassen.
    if nur_pruefen && geaendert > 0 { return ExitCode::from(1); }
    ExitCode::SUCCESS
}

/// Endung, an der ein Pruefprogramm erkannt wird.
///
/// Die Regel ist am Bestand abgelesen, nicht erfunden: `buch-tippspiel/code/`
/// nennt seine vier Pruefprogramme seit jeher `*_pruefung.dh`. Eine zweite,
/// englische Schreibweise daneben waere nur eine Quelle fuer "warum laeuft
/// meine Datei nicht mit".
const PRUEF_ENDUNG: &str = "_pruefung.dh";

/// Verzeichnisse, in denen nicht gesucht wird.
///
/// `target` und `.venv` enthalten fremden Code in Mengen; ohne diese Liste
/// laeuft ein `dhrt test` im Projektwurzel-Verzeichnis minutenlang durch
/// Abhaengigkeiten.
const NICHT_SUCHEN: &[&str] = &["target", ".git", ".venv", "node_modules", "__pycache__"];

/// Pruefprogramme unterhalb von `wurzel` einsammeln (rekursiv, sortiert).
fn pruefdateien(wurzel: &std::path::Path, raus: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(wurzel) else { return };
    let mut eintraege: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    eintraege.sort();
    for p in eintraege {
        let name = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        if p.is_dir() {
            if !NICHT_SUCHEN.contains(&name.as_str()) && !name.starts_with('.') {
                pruefdateien(&p, raus);
            }
        } else if name.ends_with(PRUEF_ENDUNG) {
            raus.push(p);
        }
    }
}

/// `dhrt test [pfad ...]` -- die Pruefprogramme laufen lassen und Bilanz ziehen.
///
/// Die Bausteine dafuer gibt es seit WP E (`ASSERT`, `ASSERT_COLLECT`,
/// `ASSERT_REPORT`, Rueckgabewert). Was fehlte, war das Dach: ohne es schreibt
/// sich jedes Projekt seinen eigenen Laeufer -- so wie `buch-tippspiel` sich
/// vor WP E sein eigenes `ASSERT` geschrieben hatte.
///
/// **Jede Datei laeuft als eigener Prozess.** Derselbe Grund wie bei
/// `TASK_START`: die Prozessgrenze ist die Zusage. Ein Pruefprogramm, das
/// abstuerzt, ein Fenster oeffnet oder Globals hinterlaesst, kann dem
/// naechsten nichts antun. ~12 ms je Start faellt bei einer Pruefung nicht
/// ins Gewicht.
///
/// **Die Standardeingabe des Kindes ist leer.** Ein Pruefprogramm mit einem
/// vergessenen `INPUT` wuerde sonst auf eine Eingabe warten, die nie kommt,
/// und der ganze Lauf haengt -- mit leerem stdin bekommt es sofort das Ende.
fn test_main(pfade: &[String]) -> ExitCode {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => { eprintln!("dhrt test: eigenen Pfad nicht gefunden: {}", e); return ExitCode::from(2); }
    };
    let ziele: Vec<String> = if pfade.is_empty() { vec![".".to_string()] }
                             else { pfade.to_vec() };
    let mut dateien: Vec<std::path::PathBuf> = Vec::new();
    for z in &ziele {
        let p = std::path::Path::new(z);
        if p.is_dir() {
            pruefdateien(p, &mut dateien);
        } else if p.is_file() {
            // Eine ausdruecklich genannte Datei laeuft, auch wenn sie nicht
            // so heisst -- wer sie hinschreibt, meint sie.
            dateien.push(p.to_path_buf());
        } else {
            eprintln!("dhrt test: '{}' gibt es nicht", z);
            return ExitCode::from(2);
        }
    }
    if dateien.is_empty() {
        eprintln!("dhrt test: keine Pruefprogramme gefunden (gesucht wird nach *{})",
                  PRUEF_ENDUNG);
        // Kein Fehler-Rueckgabewert: "nichts zu tun" ist kein Fehlschlag,
        // sonst faellt eine Kette ueber ein noch leeres Projekt.
        return ExitCode::SUCCESS;
    }
    let start = std::time::Instant::now();
    let mut fehler = 0usize;
    for d in &dateien {
        let t0 = std::time::Instant::now();
        let r = std::process::Command::new(&exe)
            .arg("run").arg(d)
            .stdin(std::process::Stdio::null())
            .output();
        let dauer = t0.elapsed().as_secs_f64();
        let name = d.display().to_string();
        match r {
            Ok(o) if o.status.success() => {
                println!("  ok      {}  ({:.2}s)", name, dauer);
            }
            Ok(o) => {
                fehler += 1;
                let code = o.status.code().unwrap_or(-1);
                println!("  FEHLER  {}  (Rueckgabewert {}, {:.2}s)", name, code, dauer);
                // Beide Kanaele zeigen: die Bilanz einer Pruefung geht nach
                // stdout, die einzelnen Fehlschlaege nach stderr (WP E).
                for (kanal, roh) in [("", &o.stdout), ("", &o.stderr)] {
                    let text = String::from_utf8_lossy(roh);
                    for zeile in text.lines() {
                        println!("          {}{}", kanal, zeile);
                    }
                }
            }
            Err(e) => {
                fehler += 1;
                println!("  FEHLER  {}  (Start fehlgeschlagen: {})", name, e);
            }
        }
    }
    println!("\n{} Datei(en), {} ok, {} mit Fehlern  ({:.2}s)",
             dateien.len(), dateien.len() - fehler, fehler, start.elapsed().as_secs_f64());
    if fehler > 0 { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

/// `dhrt --tokens <datei.dh>` -- lext die Quelldatei und gibt pro Token eine
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

/// `dhrt --ast <datei.dh>` -- lext + parst und gibt den AST als JSON aus
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

/// `dhrt --preprocess <datei.dh>` -- expandiert IMPORTs und gibt die gemergte
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
        Ok((merged, _mods, _herkunft, _ns)) => {
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
/// `.dhc`-JSON oder einen Exit-Code (Fehler bereits auf stderr gemeldet).
/// Geteilt von `--runsrc`, `run` und `--export`.
fn compile_source(raw_source: &str, base: &std::path::Path, label: &str) -> Result<serde_json::Value, ExitCode> {
    // Fehler-Format `<label>:<zeile>: <msg>` -- so erkennt der Editor (Pattern
    // `(\S+\.dh):(\d+)`) die Zeile und macht sie klickbar (wie bei Laufzeitfehlern).
    let (source, imports, herkunft, namensraeume) = match preprocess::process(raw_source, base) {
        Ok(r) => r,
        Err(e) => { eprintln!("{}:{}: Preprocess-Fehler: {}", label, e.line, e.msg); return Err(ExitCode::from(2)); }
    };
    // WP I.4: alle Meldungen der Uebersetzungs-Phasen zeigen auf die Datei und
    // Zeile, die der Nutzer VOR SICH HAT -- nicht auf die gemergte Quelle.
    // Ohne IMPORT ist beides dasselbe, mit IMPORT war es bisher irrefuehrend.
    let wo = |zeile: u32| preprocess::stelle(&herkunft, zeile, label);
    let (ext_types, aliases, module) = preprocess::compile_env(&imports);
    // E1: Hardware-Module (serial/usb/bt/wifi) sind zwar importierbar, fehlen aber
    // im Default-Build. Frueh (beim IMPORT) warnen statt erst beim ersten Aufruf --
    // die Meldung ist nicht fatal, der Lauf geht weiter (der eigentliche Aufruf
    // wirft dann wie gehabt, falls das Modul wirklich genutzt wird).
    for m in preprocess::missing_hardware_modules(&imports) {
        eprintln!("{}: Warnung: {}", label, preprocess::hardware_missing_msg(m));
    }
    let toks = match lexer::Lexer::new(&source).tokenize() {
        Ok(t) => t,
        Err(e) => { eprintln!("{}: Lexer-Fehler ({}): {}", wo(e.line as u32), e.col, e.msg); return Err(ExitCode::from(2)); }
    };
    let mut p = parser::Parser::new(toks);
    let mut ast = match p.parse() {
        Ok(a) => a,
        Err(e) => { eprintln!("{}: Parse-Fehler ({}): {}", wo(e.line as u32), e.col, e.msg); return Err(ExitCode::from(2)); }
    };
    // WP I.1: `IMPORT "x.dh" AS x` -- Top-Level-Namen der Datei bekommen ein
    // Praefix, `x.Name` an der Aufrufstelle wird darauf abgebildet. Ohne ein
    // solches IMPORT kehrt der Durchgang sofort zurueck.
    if let Err((zeile, msg)) = namensraum::anwenden(&mut ast, &herkunft, &namensraeume, p.private_namen()) {
        eprintln!("{}: Namensraum-Fehler: {}", wo(zeile), msg);
        return Err(ExitCode::from(3));
    }
    match compiler::compile_to_gbc(&ast, &ext_types, &aliases, &module, &herkunft, label) {
        Ok((j, warns)) => {
            // Nicht-fatale Compile-Warnungen (z.B. unbekanntes Builtin) vor dem
            // Lauf auf stderr -- der Lauf geht weiter, schlaegt aber spaeter ggf.
            // beim Aufruf fehl.
            for (line, msg) in warns {
                eprintln!("{}: Warnung: {}", wo(line), namensraum::lesbar_text(&msg));
            }
            Ok(j)
        }
        Err((line, msg)) => {
            let msg = namensraum::lesbar_text(&msg);
            if line > 0 { eprintln!("{}: Compile-Fehler: {}", wo(line), msg); }
            else { eprintln!("{}: Compile-Fehler: {}", label, msg); }
            Err(ExitCode::from(3))
        }
    }
}

/// `dhrt profile <datei.dh>` -- fuehrt das Programm instrumentiert aus und gibt
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
    let stopped = matches!(&run_res, Err(e) if machine.was_stopped(e));
    // WP A: `EXIT(code)` ist kein Fehler -- sonst zeigte das Profiler-Panel bei
    // jedem Programm, das sich selbst beendet, eine rote Meldung. Vor den
    // take_*-Aufrufen lesen.
    let selbst_beendet = machine.exit_code().is_some();
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
    // `stopped` (oben, VOR dem `take_output`-Move von `machine`) ist bereits
    // die richtige Antwort -- kein zweiter `was_stopped`-Aufruf noetig.
    if let Err(e) = &run_res {
        if !stopped && !selbst_beendet {
            blob["error"] = serde_json::json!(e);
            blob["error_line"] = serde_json::json!(err_line);
        }
    }
    println!("{}", serde_json::to_string(&blob).unwrap_or_else(|_| "{}".into()));
    ExitCode::SUCCESS
}

/// `dhrt debug <datei.dh>` -- interaktiver Debugger. Spricht ein
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
    // Review-Fund: verglich frueher den Fehlertext gegen "__DEBUG_STOP__" --
    // ein GB-Programm mit `THROW "__DEBUG_STOP__"` haette einen echten Fehler
    // so faelschlich als "sauber gestoppt" gemeldet. `was_debug_stopped()`
    // liest stattdessen das interne Flag, das THROW nie setzt.
    let ev = match &res {
        Ok(()) => serde_json::json!({"event": "finished", "reason": "done"}),
        Err(_) if machine.was_debug_stopped() =>
            serde_json::json!({"event": "finished", "reason": "stopped"}),
        // WP A: `EXIT(code)` ist ein regulaeres Ende, kein Fehler -- der Editor
        // soll dafuer keinen roten Fehlerbalken zeigen.
        Err(_) if machine.exit_code().is_some() =>
            serde_json::json!({"event": "finished", "reason": "done"}),
        Err(e) => serde_json::json!({
            "event": "error", "line": machine.error_line(), "message": e }),
    };
    println!("{}", serde_json::to_string(&ev).unwrap_or_default());
    ExitCode::SUCCESS
}

/// `dhrt --check <datei.dh>` -- Front-End-Diagnostik fuer Editor-Live-Error-Check
/// und LSP. Gibt ein JSON-Array `[{line,col,severity,phase,message}]` auf stdout
/// aus (leer = fehlerfrei). Exit 0 auch bei gefundenen Problemen; nur ein
/// I/O-Fehler liefert Exit 1. Zeilen beziehen sich auf die GEMERGTE Quelle
/// (nach IMPORT-Expansion) -- der Editor mappt via origins zurueck.
/// `dhrt --check <datei> [datei ...]`
///
/// EINE Datei: die Antwort ist das Diagnose-Array wie eh und je -- der Editor
/// (`error_check.py`) haengt daran, und das bleibt so.
///
/// MEHRERE Dateien: eine JSON-Zeile je Datei, `{"datei": ..., "probleme": [...]}`.
/// Der Grund ist gemessen: die Tests riefen `--check` einzeln fuer jede der
/// 185 Beispieldateien auf, dreimal ueber verschiedene Tests -- 555
/// Prozessstarts, und auf CI der teuerste Testblock ueberhaupt (32 s von
/// 232 s). Ein Aufruf statt 185 spart die Startzeit, nicht die Arbeit.
///
/// Nuetzlich ist es auch ausserhalb der Tests: der Editor koennte ein ganzes
/// Projekt in einem Rutsch pruefen, statt Datei fuer Datei einen Prozess zu
/// starten.
fn check_main(pfade: &[String]) -> ExitCode {
    let einzeln = pfade.len() == 1;
    for pfad in pfade {
        let raw_source = match std::fs::read_to_string(pfad) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Kann '{}' nicht lesen: {}", pfad, e);
                if einzeln { return ExitCode::from(1); }
                // Bei mehreren nicht abbrechen: eine unlesbare Datei soll die
                // 184 anderen nicht um ihre Diagnose bringen.
                println!("{}", serde_json::json!({
                    "datei": pfad,
                    "probleme": [{"line": 1, "col": 1, "severity": "error",
                                  "phase": "datei",
                                  "message": format!("nicht lesbar: {}", e)}]}));
                continue;
            }
        };
        let base = std::path::Path::new(pfad).parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        // Der Dateiname als Label -- sonst rendert ein Verweis auf die
        // Hauptdatei im Meldungstext als nacktes ":5".
        let label = std::path::Path::new(pfad).file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| pfad.to_string());
        let diags = check_source(&raw_source, &base, &label);
        if einzeln {
            println!("{}", serde_json::to_string(&diags).unwrap_or_else(|_| "[]".into()));
        } else {
            println!("{}", serde_json::json!({"datei": pfad, "probleme": diags}));
        }
    }
    ExitCode::SUCCESS
}

/// Front-End-Kette (preprocess -> lex -> parse -> compile) als Diagnostik.
/// MVP: bricht bei der ersten Fehlerstelle ab und liefert genau eine Diagnose
/// (wie error_check.py einen ParseProblem liefert). Leeres Array = fehlerfrei.
/// Compiler-Fehler tragen heute keine Zeile -> `line: 0` (spaetere Verfeinerung).
fn check_source(raw_source: &str, base: &std::path::Path, label: &str) -> Vec<serde_json::Value> {
    let (source, imports, herkunft, namensraeume) = match preprocess::process(raw_source, base) {
        Ok(r) => r,
        Err(e) => return vec![serde_json::json!({
            "line": e.line, "col": 0, "severity": "error",
            "phase": "preprocess", "message": e.msg })],
    };
    let (ext_types, aliases, module) = preprocess::compile_env(&imports);
    let toks = match lexer::Lexer::new(&source).tokenize() {
        Ok(t) => t,
        Err(e) => return vec![serde_json::json!({
            "line": e.line, "col": e.col, "severity": "error",
            "phase": "lex", "message": e.msg })],
    };
    let mut p = parser::Parser::new(toks);
    let mut ast = match p.parse() {
        Ok(a) => a,
        Err(e) => return vec![serde_json::json!({
            "line": e.line, "col": e.col, "severity": "error",
            "phase": "parse", "message": e.msg })],
    };
    // WP I.1: derselbe Durchgang wie im Run-Pfad -- sonst zeigt der Editor
    // gruen, und erst der Lauf meldet den Namensraum-Fehler.
    if let Err((zeile, msg)) = namensraum::anwenden(&mut ast, &herkunft, &namensraeume, p.private_namen()) {
        return vec![serde_json::json!({
            "line": zeile, "col": 1, "severity": "error",
            "phase": "namensraum", "message": msg })];
    }
    match compiler::compile_to_gbc(&ast, &ext_types, &aliases, &module, &herkunft, label) {
        // E1: Bei fehlerfreiem Compile noch Warnungen fuer IMPORTs von
        // Hardware-Modulen ergaenzen, die in diesem dhrt-Build fehlen -- damit
        // der Editor das schon auf der IMPORT-Zeile markiert (nicht erst beim
        // Lauf). Leer, wenn kein solches Modul importiert wird.
        Ok((_, warns)) => {
            let mut diags: Vec<serde_json::Value> =
                preprocess::missing_hardware_imports_with_lines(raw_source).into_iter()
                    .map(|(line, m)| serde_json::json!({
                        "line": line, "col": 0, "severity": "warning",
                        "phase": "preprocess", "message": preprocess::hardware_missing_msg(m) }))
                    .collect();
            // G1 (systemisch): Aufrufe von Builtins, die dhrt nicht kennt, schon
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

/// `dhrt --runsrc <datei.dh>` -- volle Front-End-Kette in Rust, OHNE chdir
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

/// `dhrt run <datei.dh>` (Stufe 5) -- eigenstaendiger End-to-End-Lauf aus
/// Quelltext, ohne Python. Wechselt wie `dhrun.py` ins Verzeichnis der Datei,
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
    // Ins Datei-Verzeichnis wechseln (wie dhrun.py os.chdir(file.parent)).
    let _ = std::env::set_current_dir(&base);
    // TASK_START braucht den eigenen Dateinamen, um `dhrt call` darauf zu
    // starten. Nach dem chdir genuegt der blosse Name.
    builtins::set_quelldatei(label.clone());
    compile_and_run_source(&raw_source, &base, &label)
}

/// `dhrt call <datei> <funktion> [arg]`.
fn call_main(path: &str, fn_name: &str, args: Vec<String>) -> ExitCode {
    let abs = std::fs::canonicalize(path).map(strip_extended_prefix)
        .unwrap_or_else(|_| std::path::PathBuf::from(path));
    let base = abs.parent().map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let label = abs.file_name().map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    let raw_source = match std::fs::read_to_string(&abs) {
        Ok(t) => t,
        Err(e) => { return call_fehler(&format!("Kann {} nicht lesen: {}", path, e)); }
    };
    let _ = std::env::set_current_dir(&base);
    builtins::set_quelldatei(label.clone());
    match compile_source(&raw_source, &base, &label) {
        Ok(json) => call_program_value(json, fn_name, args),
        Err(code) => code,
    }
}

/// Fehler als JSON-Zeile -- der Aufrufer soll nicht stderr parsen muessen.
fn call_fehler(msg: &str) -> ExitCode {
    println!("{}", serde_json::json!({"ok": false, "fehler": msg}));
    ExitCode::from(1)
}

fn call_program_value(json: serde_json::Value, fn_name: &str,
                      args: Vec<String>) -> ExitCode {
    let prog = match model::load_program(&json) {
        Ok(p) => p,
        Err(e) => return call_fehler(&format!("Lade-Fehler: {}", e)),
    };
    // Ein String-Argument, das wie eine Zahl aussieht, wird zur Zahl -- sonst
    // muesste jede Auftragsfunktion ihr Argument selbst umwandeln, und der
    // haeufigste Fall (eine Kennung, ein Zaehler) waere der unbequemste.
    let args: Vec<value::Value> = args.iter().map(|a| {
        match a.parse::<i64>() {
            Ok(i) => value::Value::Int(i),
            Err(_) => match a.parse::<f64>() {
                // Nur mit Punkt: "1e5" oder "inf" waeren als Zahl gemeint
                // ueberraschend, und ein Text bleibt lieber ein Text.
                Ok(f) if a.contains('.') => value::Value::Float(f),
                _ => value::Value::str_rc(a),
            },
        }
    }).collect();
    let mut machine = vm::Vm::new(&prog);
    match machine.call_named(fn_name, args) {
        Ok(wert) => {
            let ergebnis = match &wert {
                value::Value::Int(i) => serde_json::json!(i),
                value::Value::Float(f) => serde_json::json!(f),
                value::Value::Bool(b) => serde_json::json!(b),
                value::Value::Nil => serde_json::Value::Null,
                andere => serde_json::json!(builtins::str_of(andere)),
            };
            let out = machine.take_output();
            println!("{}", serde_json::json!({
                "ok": true, "ergebnis": ergebnis, "ausgabe": out}));
            ExitCode::SUCCESS
        }
        Err(e) => call_fehler(&e),
    }
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

/// `dhrt --export <datei.dh> [out_dir]` -- buendelt das Programm zu einer
/// eigenstaendigen Exe (Selbst-Export ohne Python): kompiliert Quelltext ->
/// .dhc und haengt den Payload (gbc + Footer `[u64 len][DHRTPAY1]`) an eine
/// Kopie der EIGENEN Runtime-Exe. `assets/` neben der Quelle wird mitkopiert.
/// Pendant zu drachenhauch/export.py.
fn export_main(path: &str, out_dir: Option<&str>, mit_daten: bool) -> ExitCode {
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
    // 1) Quelltext -> .dhc-JSON (kompakt).
    let label = abs.file_name().map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    let json = match compile_source(&raw_source, &base, &label) {
        Ok(j) => j,
        Err(code) => return code,
    };
    // Review-Fund: `unwrap_or_default()` liess einen Serialisierungsfehler
    // lautlos zu einem LEEREN Payload werden -- das Bundle wurde trotzdem
    // geschrieben und "Exportiert: ..." gemeldet, obwohl `embedded_gbc()`
    // `len == 0` ablehnt und die ausgelieferte Exe beim Start nur noch
    // "Verwendung: dhrt <datei.dhc>" ausgibt.
    let gbc_bytes = match serde_json::to_string(&json) {
        Ok(s) => s.into_bytes(),
        Err(e) => { eprintln!("Kann .dhc nicht serialisieren: {}", e); return ExitCode::from(1); }
    };
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
    let exe_suffix = if exe.extension().map(|e| e == "exe").unwrap_or(false) || cfg!(windows) {
        ".exe"
    } else { "" };
    // Den eigenen `_dist`-Ordner vor dem Schreiben raeumen. Sonst bleibt vom
    // vorigen Lauf alles liegen, was diesmal nicht mehr dazugehoert -- die
    // .exe wird ueberschrieben, eine Datenbank daneben nicht, und sie wandert
    // stillschweigend mit ausgeliefert.
    //
    // Nur der VOM EXPORT SELBST gewaehlte Ordner, und nur wenn eine Exe mit
    // unserem Payload darin liegt: das ist der Beweis, dass er von einem
    // frueheren Export stammt und niemandem sonst gehoert. Ein von Hand
    // angegebenes Verzeichnis bleibt unangetastet -- dort koennte alles
    // stehen.
    if out_dir.is_none() && out.is_dir() {
        let alte_exe = out.join(format!("{}{}", stem, exe_suffix));
        let ist_unser = std::fs::read(&alte_exe)
            .map(|d| d.len() > 8 && d[d.len() - 8..] == *PAYLOAD_MAGIC)
            .unwrap_or(false);
        if ist_unser {
            if let Err(e) = std::fs::remove_dir_all(&out) {
                eprintln!("Warnung: '{}' nicht geraeumt: {}", out.display(), e);
            }
        } else if alte_exe.exists() {
            eprintln!("Warnung: '{}' enthaelt eine fremde Datei gleichen Namens und wurde nicht geraeumt.", out.display());
        }
    } else if out_dir.is_some() && out.is_dir() {
        eprintln!("Hinweis: '{}' wird nicht geraeumt (selbst angegeben), Dateien aus frueheren Laeufen bleiben liegen.", out.display());
    }
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("Kann Ausgabeverzeichnis '{}' nicht anlegen: {}", out.display(), e);
        return ExitCode::from(1);
    }
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
    let bericht = bundle_referenced_assets(&raw_source, &base, &out, mit_daten);
    // Mit Namen, nicht als Zahl: "1 Datei mitkopiert" sagt nicht, WELCHE --
    // und genau daran ist eine fremde Datenbank im Bundle nicht aufgefallen.
    if !bericht.kopiert.is_empty() {
        println!("  {} referenzierte Datei(en) mitkopiert:", bericht.kopiert.len());
        for f in &bericht.kopiert { println!("    {}", f); }
    }
    if !bericht.uebersprungen.is_empty() {
        println!("  {} Datenbank(en) NICHT mitkopiert -- das Programm legt sie selbst an:", bericht.uebersprungen.len());
        for f in &bericht.uebersprungen { println!("    {}", f); }
        println!("  (mit --mit-daten trotzdem mitnehmen)");
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
/// Ist das eine SQLite-Datenbank? Am Inhalt erkannt, nicht am Namen: die
/// ersten 16 Bytes einer SQLite-Datei sind immer `SQLite format 3\0`.
///
/// Warum ueberhaupt: `bundle_referenced_assets` kann nicht wissen, WOZU ein
/// Dateiname im Quelltext steht. Bei einem Bild ist Mitkopieren richtig, bei
/// einer Datenbank, die das Programm selbst anlegt, falsch -- sie enthaelt
/// die Daten des Entwicklers, und die gehen den Empfaenger nichts an. Der
/// Inhalt verraet den Unterschied dort, wo der Name es nicht tut.
fn ist_sqlite_datei(pfad: &std::path::Path) -> bool {
    use std::io::Read;
    let mut f = match std::fs::File::open(pfad) { Ok(f) => f, Err(_) => return false };
    let mut kopf = [0u8; 16];
    match f.read_exact(&mut kopf) {
        Ok(()) => &kopf == b"SQLite format 3\0",
        Err(_) => false,
    }
}

/// Was der Export mit den referenzierten Dateien gemacht hat.
struct AssetBericht {
    kopiert: Vec<String>,
    uebersprungen: Vec<String>,
}

fn bundle_referenced_assets(source: &str, base: &std::path::Path, out: &std::path::Path,
                            mit_daten: bool) -> AssetBericht {
    let mut seen = std::collections::HashSet::new();
    let mut bericht = AssetBericht { kopiert: Vec::new(), uebersprungen: Vec::new() };
    for lit in string_literals(source) {
        if lit.is_empty() || lit.len() > 400 { continue; }
        // Review-Fund: "." bzw. ".." sind gaengige String-Literale in GANZ
        // GEWOEHNLICHEM Code (`IF ch = "."`, `JOIN(parts, ".")`, ...), keine
        // Asset-Pfade. `strip_parent_prefix(".")` liess sie aber unveraendert
        // durch ("." hat kein "../"/"./"-Praefix), und `base.join(".")` /
        // `base.join("..")` zeigen auf `base` bzw. dessen Elternverzeichnis
        // -- beide EXISTIEREN immer. `copy_dir_recursive(base, out)` kopiert
        // dann `out` (das gerade erst angelegte `_dist`-Verzeichnis) in sich
        // selbst hinein und rekursiert bis zur Pfadlaengen-Grenze, wobei bei
        // jeder Ebene die frisch geschriebene Bundle-Exe erneut kopiert wird
        // -- ein reproduzierbarer Festplatten-Fuellstand-Bug aus voellig
        // gewoehnlichem GB-Code.
        if lit == "." || lit == ".." { continue; }
        // Absolute Pfade ueberspringen (nicht buendelbar): /... oder C:\...
        if lit.starts_with('/') || lit.starts_with('\\') { continue; }
        let b = lit.as_bytes();
        if b.len() >= 2 && b[1] == b':' { continue; }   // C:\ / D:/
        let src = base.join(&lit);
        if !src.exists() { continue; }
        // Verteidigung in der Tiefe: `src` darf nicht `out` selbst oder ein
        // Vorfahre von `out` sein (sonst kopiert copy_dir_recursive das
        // Zielverzeichnis in sich selbst hinein, egal ueber welches Literal
        // das passiert waere).
        if let (Ok(src_c), Ok(out_c)) = (src.canonicalize(), out.canonicalize()) {
            if out_c.starts_with(&src_c) { continue; }
        }
        let rel = strip_parent_prefix(&lit);
        if rel.is_empty() || !seen.insert(rel.clone()) { continue; }
        // Eine Datenbank ist kein Asset, sondern der Datenstand dessen, der
        // exportiert. Wer wirklich eine vorbereitete Datenbank ausliefern
        // will (ein Lexikon, eine Level-Sammlung), sagt --mit-daten.
        if !mit_daten && src.is_file() && ist_sqlite_datei(&src) {
            bericht.uebersprungen.push(rel.clone());
            continue;
        }
        let dst = out.join(&rel);
        if src.is_dir() {
            if copy_dir_recursive(&src, &dst).is_ok() { bericht.kopiert.push(rel.clone()); }
        } else {
            if let Some(p) = dst.parent() { let _ = std::fs::create_dir_all(p); }
            if std::fs::copy(&src, &dst).is_ok() { bericht.kopiert.push(rel.clone()); }
        }
    }
    bericht
}

/// Laedt eine `.dhc` (JSON-Text) und fuehrt sie aus. Geteilt zwischen Dev-Modus
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

/// Laedt ein bereits geparstes `.dhc`-JSON-`Value` und fuehrt es aus.
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
            // WP A: `EXIT(code)` wickelt die VM ueber denselben Fehler-Kanal ab
            // wie die Stop-Signale, ist aber KEIN Fehler -- kein
            // "Laufzeitfehler" auf stderr, sondern nur der Rueckgabewert.
            // Ebenfalls vor take_output() lesen.
            let exit = machine.exit_code();
            // Bei Laufzeitfehler trotzdem bisherige Ausgabe zeigen.
            let out = machine.take_output();
            let stdout = std::io::stdout();
            let mut h = stdout.lock();
            let _ = h.write_all(out.as_bytes());
            let _ = h.flush();
            if let Some(code) = exit {
                return ExitCode::from(code as u8);
            }
            if line != 0 {
                eprintln!("Laufzeitfehler in {}:{}: {}", source_label, line, e);
            } else {
                eprintln!("Laufzeitfehler in {}: {}", source_label, e);
            }
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{embedded_gbc_in, ist_quelldatei, PAYLOAD_MAGIC};

    #[test]
    fn quelldatei_erkennt_beide_endungen_und_ignoriert_gross_klein() {
        let alt = super::ALTE_QUELL_ENDUNG;                     // ".gb"
        for gut in ["spiel.dh", "SPIEL.DH", "a/b/c.Dh",
                    &format!("alt{alt}"), &format!("ALT{}", alt.to_uppercase())] {
            assert!(ist_quelldatei(gut), "{gut} sollte Quelltext sein");
        }
        // Bytecode und Fremdes duerfen NICHT als Quelle durchgehen -- sonst
        // liefe eine .dhc durch den Compiler statt durch die VM.
        for schlecht in ["spiel.dhc", &format!("spiel{alt}c"), "spiel.dhx", "dh", "spiel.txt"] {
            assert!(!ist_quelldatei(schlecht), "{schlecht} ist keine Quelle");
        }
    }

    /// Baut, was `--export` schreibt: <exe><gbc><laenge><magic>.
    fn bundle(exe: &[u8], gbc: &str) -> Vec<u8> {
        let mut v = exe.to_vec();
        v.extend_from_slice(gbc.as_bytes());
        v.extend_from_slice(&(gbc.len() as u64).to_le_bytes());
        v.extend_from_slice(PAYLOAD_MAGIC);
        v
    }

    #[test]
    fn findet_die_nutzlast_am_dateiende() {
        let b = bundle(b"MZ....exe....", r#"{"main":1}"#);
        assert_eq!(embedded_gbc_in(&b).as_deref(), Some(r#"{"main":1}"#));
    }

    #[test]
    fn findet_sie_auch_hinter_einer_signatur() {
        // Der eigentliche Zweck des Umbaus: signtool haengt den
        // Zertifikatsblock HINTER unseren Footer.
        let mut b = bundle(b"MZ....exe....", r#"{"main":2}"#);
        b.extend_from_slice(&[0u8; 4096]);
        assert_eq!(embedded_gbc_in(&b).as_deref(), Some(r#"{"main":2}"#));
    }

    #[test]
    fn ohne_nutzlast_kein_treffer() {
        assert_eq!(embedded_gbc_in(b"MZ....einfach nur eine exe...."), None);
        assert_eq!(embedded_gbc_in(b"kurz"), None);          // < 16 Bytes
    }

    #[test]
    fn zufaelliges_vorkommen_im_json_verwirrt_nicht() {
        // Die acht Bytes stehen mitten in der Nutzlast -- der ECHTE Footer
        // liegt weiter hinten und muss gewinnen.
        let gbc = format!(r#"{{"s":"{}"}}"#, String::from_utf8_lossy(PAYLOAD_MAGIC));
        let b = bundle(b"MZ....exe....", &gbc);
        assert_eq!(embedded_gbc_in(&b).as_deref(), Some(gbc.as_str()));
    }

    #[test]
    fn ueberspringt_einen_falschen_treffer_im_signaturblock() {
        // Kennung im angehaengten Block, aber mit unbrauchbarer Laenge davor:
        // die Suche darf nicht aufgeben, sondern muss weiter vorne fuendig
        // werden.
        let mut b = bundle(b"MZ....exe....", r#"{"main":3}"#);
        b.extend_from_slice(&u64::MAX.to_le_bytes());   // unmoegliche Laenge
        b.extend_from_slice(PAYLOAD_MAGIC);
        assert_eq!(embedded_gbc_in(&b).as_deref(), Some(r#"{"main":3}"#));
    }
    // --- Export: Datenbanken sind keine Assets ---------------------------
    //
    // bundle_referenced_assets sammelt jede Datei ein, deren Name irgendwo im
    // Quelltext als Zeichenkette steht. Fuer ein Bild ist das richtig; fuer
    // eine Datenbank, die das Programm selbst anlegt, wanderte damit der
    // Datenstand des Entwicklers zum Empfaenger.

    fn temp_ordner(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("dhrt_export_test_{name}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn sqlite_wird_am_inhalt_erkannt_nicht_am_namen() {
        let d = temp_ordner("erkennung");
        // Eine echte SQLite-Datei faengt mit diesen 16 Bytes an.
        let echt = d.join("daten.irgendwas");
        std::fs::write(&echt, b"SQLite format 3\0und noch mehr").unwrap();
        assert!(super::ist_sqlite_datei(&echt),
                "Endung egal -- der Inhalt entscheidet");

        // Umgekehrt: was .db heisst, muss keine Datenbank sein.
        let getarnt = d.join("keine.db");
        std::fs::write(&getarnt, b"nur Text, kein SQLite").unwrap();
        assert!(!super::ist_sqlite_datei(&getarnt),
                "der Name allein macht keine Datenbank");

        // Zu kurz zum Erkennen -> keine Datenbank (und kein Absturz).
        let kurz = d.join("kurz.db");
        std::fs::write(&kurz, b"SQL").unwrap();
        assert!(!super::ist_sqlite_datei(&kurz));

        // Was es nicht gibt, ist auch keine.
        assert!(!super::ist_sqlite_datei(&d.join("gibtsnicht.db")));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn export_ueberspringt_datenbank_nimmt_bild_mit() {
        let d = temp_ordner("auswahl");
        let out = d.join("dist");
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(d.join("bild.png"), b"\x89PNG so tun als ob").unwrap();
        std::fs::write(d.join("spiel.db"), b"SQLite format 3\0xxxx").unwrap();
        let quelle = "LOADIMAGE(\"bild.png\")\nDB_OPEN(\"spiel.db\")\n";

        let b = super::bundle_referenced_assets(quelle, &d, &out, false);
        assert_eq!(b.kopiert, vec!["bild.png".to_string()]);
        assert_eq!(b.uebersprungen, vec!["spiel.db".to_string()]);
        assert!(out.join("bild.png").exists());
        assert!(!out.join("spiel.db").exists(), "die Datenbank darf nicht im Bundle liegen");

        // Mit --mit-daten will es der Autor ausdruecklich -- dann kommt sie mit.
        let out2 = d.join("dist2");
        std::fs::create_dir_all(&out2).unwrap();
        let b2 = super::bundle_referenced_assets(quelle, &d, &out2, true);
        assert!(b2.uebersprungen.is_empty());
        assert_eq!(b2.kopiert.len(), 2);
        assert!(out2.join("spiel.db").exists());
        let _ = std::fs::remove_dir_all(&d);
    }
}
