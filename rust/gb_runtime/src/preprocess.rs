//! IMPORT-Preprocessor -- Stufe 4 der Front-End-Portierung.
//!
//! Port von `gamebasic/preprocess.py`. Expandiert `IMPORT "<name>"`-Zeilen
//! rekursiv VOR dem Lexen:
//!   - Quellcode-IMPORT (`IMPORT "helper.gb"` / relativer Pfad): Datei lesen,
//!     rekursiv preprocessen, mit `' === IMPORT ... ===`-Markern inlinen.
//!   - Built-in-Modul (`IMPORT "json"`): die Zeile wird zu einem Kommentar
//!     `' === IMPORT MODULE json ===` -- gbrt hat die Modul-Builtins nativ,
//!     es ist also kein echtes Inlining noetig.
//!
//! Wie in Python ist das KEIN Modulsystem mit Namespaces -- inkludierter Code
//! teilt den globalen Namensraum. Verifikation: Merge-Ergebnis-Gleichheit
//! gegen `preprocess.process()` (tests/test_rust_preprocess_parity.py).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;

/// Built-in-Modul-Namen (= `gamebasic/modules/*.py`, ohne `__init__`).
/// Muss mit `modules.discover_modules()` synchron bleiben. gbrt implementiert
/// diese Module nativ; ein `IMPORT "<modul>"` wird hier nur zu einem Kommentar.
const MODULES: &[&str] = &[
    "astar", "audio", "bt", "camera", "controller", "curves", "db", "ecs",
    "g3d", "gui", "html", "imgfx", "input", "json", "net", "particles",
    "physics", "regex", "save", "scene", "serial", "sprite", "tile_collide",
    "tiled", "tween", "ui", "usb", "vec2", "wifi",
];

/// Externe Typen, die ein Built-in-Modul registriert (lowercase, wie der
/// Lexer IDENTs liefert). Spiegelt `register_type(...)` der Module -- damit
/// der Rust-Compiler `DIM x AS VEC2` & Co. akzeptiert, sobald das Modul
/// importiert wurde (entspricht Pythons `EXTERNAL_TYPES` nach `load_module`).
/// Module ohne eigenen Typ stehen nicht in der Tabelle.
const MODULE_TYPES: &[(&str, &[&str])] = &[
    ("astar", &["astar_grid"]),
    ("audio", &["audio_channel"]),
    ("bt", &["bt_handle"]),
    ("controller", &["char_controller", "tiled_map"]),
    ("db", &["db_conn", "db_result"]),
    ("ecs", &["ecs_world"]),
    ("gui", &["gui_widget", "gui_window"]),
    ("json", &["json_handle"]),
    ("net", &["net_listener", "net_socket", "net_udp"]),
    ("particles", &["particle_system"]),
    ("physics", &["physics_broad"]),
    ("save", &["save_handle"]),
    ("serial", &["serial_handle"]),
    ("sprite", &["sprite"]),
    ("tile_collide", &["tiled_map"]),
    ("tiled", &["tiled_map"]),
    ("tween", &["tween"]),
    ("usb", &["usb_handle"]),
    ("vec2", &["vec2"]),
];

/// Liefert die externen Typ-Namen (lowercase), die die angegebenen (bereits
/// lowercase) Modul-Namen registrieren -- fuer den Compiler als zulaessige
/// DIM-Typen.
pub fn external_types(modules: &[String]) -> HashSet<String> {
    let mut out = HashSet::new();
    for m in modules {
        if let Some((_, types)) = MODULE_TYPES.iter().find(|(name, _)| name == m) {
            for t in *types {
                out.insert((*t).to_string());
            }
        }
    }
    out
}

#[derive(Debug)]
pub struct PreprocessError {
    pub line: usize,
    pub msg: String,
}

fn import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // 1:1 zu preprocess._IMPORT_RE (case-insensitive).
        Regex::new(
            r#"(?i)^\s*IMPORT\s+"([^"]+)"\s*(?:AS\s+([A-Za-z_][A-Za-z0-9_]*)\s*)?(?:'.*)?$"#,
        )
        .unwrap()
    })
}

/// `^[a-zA-Z][a-zA-Z0-9_]*$` -- wie modules._MODULE_NAME_RE.
fn is_valid_module_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Heuristik wie preprocess._looks_like_module_name: kein Slash/Backslash,
/// keine `.gb`-Endung, gueltiger Modul-Name.
fn looks_like_module_name(rel: &str) -> bool {
    if rel.contains('/') || rel.contains('\\') {
        return false;
    }
    if rel.to_lowercase().ends_with(".gb") {
        return false;
    }
    is_valid_module_name(rel)
}

/// Existiert das Built-in-Modul? (entspricht `load_module(rel) == True`).
/// Modul-Aufloesung ist case-insensitiv (Python importiert `name.lower()`).
fn is_known_module(rel: &str) -> bool {
    let low = rel.to_lowercase();
    MODULES.contains(&low.as_str())
}

/// Expandiert alle IMPORTs rekursiv. Liefert `(gemergte_quelle,
/// importierte_modul_namen)`. Die Modul-Namen (lowercase, dedupliziert)
/// dienen dem Compiler zum Erkennen externer DIM-Typen (siehe
/// `external_types`). `base` ist das Verzeichnis fuer relative IMPORT-Pfade.
pub fn process(source: &str, base: &Path) -> Result<(String, Vec<String>), PreprocessError> {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    let mut mods: Vec<String> = Vec::new();
    process_inner(source, base, &mut seen, &mut out, &mut mods)?;
    Ok((out.join("\n"), mods))
}

fn process_inner(
    source: &str,
    base: &Path,
    seen: &mut HashSet<PathBuf>,
    out: &mut Vec<String>,
    mods: &mut Vec<String>,
) -> Result<(), PreprocessError> {
    for (idx0, raw_line) in source.split('\n').enumerate() {
        let line_idx = idx0 + 1;
        // `\r` am Zeilenende (CRLF-Dateien) vor dem Regex-Match entfernen --
        // Python liest Textmodus (\r\n -> \n), der Regex matcht sonst nicht
        // gegen das `$`-Ende.
        let raw = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let caps = match import_re().captures(raw) {
            Some(c) => c,
            None => {
                out.push(raw.to_string());
                continue;
            }
        };
        let rel = caps.get(1).unwrap().as_str().to_string();
        let alias = caps.get(2).map(|m| m.as_str().to_string());
        let joined = base.join(&rel);
        let exists = joined.exists();
        // Resolve (wie Python `.resolve()`) fuer den seen-Schluessel -- nur
        // sinnvoll, wenn die Datei existiert.
        let canon = if exists {
            joined.canonicalize().unwrap_or_else(|_| joined.clone())
        } else {
            joined.clone()
        };

        if exists && seen.contains(&canon) {
            out.push(format!("' [IMPORT bereits inkludiert: {}]", rel));
            continue;
        }

        if !exists {
            // Fallback: Built-in-Modul (json/db/ui/...). gbrt hat sie nativ,
            // also nur einen Kommentar-Marker emittieren.
            if looks_like_module_name(&rel) && is_known_module(&rel) {
                let tag = match &alias {
                    Some(a) => format!(" AS {}", a),
                    None => String::new(),
                };
                out.push(format!("' === IMPORT MODULE {}{} ===", rel, tag));
                let low = rel.to_lowercase();
                if !mods.contains(&low) {
                    mods.push(low);
                }
                continue;
            }
            return Err(PreprocessError {
                line: line_idx,
                msg: format!(
                    "IMPORT: Datei nicht gefunden: {} (gesucht: {})",
                    rel,
                    joined.display()
                ),
            });
        }

        let content = std::fs::read_to_string(&joined).map_err(|e| PreprocessError {
            line: line_idx,
            msg: format!("IMPORT: Lesefehler bei {}: {}", rel, e),
        })?;
        seen.insert(canon.clone());
        out.push(format!("' === IMPORT {} ===", rel));
        let inner_base = canon.parent().unwrap_or(base).to_path_buf();
        process_inner(&content, &inner_base, seen, out, mods)?;
        out.push(format!("' === END IMPORT {} ===", rel));
    }
    Ok(())
}
