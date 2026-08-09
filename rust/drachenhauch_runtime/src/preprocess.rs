//! IMPORT-Preprocessor -- Stufe 4 der Front-End-Portierung.
//!
//! Port von `gamebasic/preprocess.py`. Expandiert `IMPORT "<name>"`-Zeilen
//! rekursiv VOR dem Lexen:
//!   - Quellcode-IMPORT (`IMPORT "helper.gb"` / relativer Pfad): Datei lesen,
//!     rekursiv preprocessen, mit `' === IMPORT ... ===`-Markern inlinen.
//!   - Built-in-Modul (`IMPORT "json"`): die Zeile wird zu einem Kommentar
//!     `' === IMPORT MODULE json ===` -- dhrt hat die Modul-Builtins nativ,
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
/// Muss mit `modules.discover_modules()` synchron bleiben. dhrt implementiert
/// diese Module nativ; ein `IMPORT "<modul>"` wird hier nur zu einem Kommentar.
const MODULES: &[&str] = &[
    "animfsm", "astar", "audio", "bt", "camera", "chart", "cloud", "controller", "curves", "db", "ecs",
    "firmata", "g3d", "gui", "html", "imgfx", "input", "json", "m3d", "mqtt", "net", "particles",
    "physics", "physics2d", "physics3d", "regex", "save", "scene", "serial", "sprite",
    "tile_collide", "tiled", "timer", "tween", "ui", "usb", "vec2", "wifi",
];

/// Hardware-/IoT-Module hinter Cargo-Features (= die `--hardware`-Features in
/// `rust/build_runtime.py`). Importierbar (sie stehen in `MODULES`), aber im
/// Default-Build NICHT einkompiliert -- jeder Funktionsaufruf schluege sonst
/// erst zur Laufzeit fehl (vgl. `vm.rs::unknown_builtin_msg`). Darum wird beim
/// IMPORT frueh gewarnt.
const HARDWARE_MODULES: &[&str] = &["serial", "usb", "bt", "wifi", "firmata"];

/// Externe Typen, die ein Built-in-Modul registriert (lowercase, wie der
/// Lexer IDENTs liefert). Spiegelt `register_type(...)` der Module -- damit
/// der Rust-Compiler `DIM x AS VEC2` & Co. akzeptiert, sobald das Modul
/// importiert wurde (entspricht Pythons `EXTERNAL_TYPES` nach `load_module`).
/// Module ohne eigenen Typ stehen nicht in der Tabelle.
const MODULE_TYPES: &[(&str, &[&str])] = &[
    ("animfsm", &["anim_fsm"]),
    ("astar", &["astar_grid"]),
    ("chart", &["chart"]),
    ("audio", &["audio_channel", "sample", "audio_clock", "audio_listener", "audio_emitter", "audio_mod"]),
    ("bt", &["bt_handle"]),
    ("controller", &["char_controller", "tiled_map"]),
    ("db", &["db_conn", "db_result"]),
    ("ecs", &["ecs_world"]),
    ("firmata", &["firmata_handle"]),
    ("gui", &["gui_widget", "gui_window"]),
    ("json", &["json_handle"]),
    ("m3d", &["vec3", "vec4", "quat", "mat4"]),
    ("mqtt", &["mqtt_handle"]),
    ("net", &["net_listener", "net_socket", "net_udp"]),
    ("particles", &["particle_system"]),
    ("physics", &["physics_broad"]),
    ("physics2d", &["phys2d_world"]),
    ("physics3d", &["phys_world"]),
    ("save", &["save_handle"]),
    ("serial", &["serial_handle"]),
    ("sprite", &["sprite"]),
    ("tile_collide", &["tiled_map"]),
    ("tiled", &["tiled_map"]),
    ("tween", &["tween"]),
    ("usb", &["usb_handle"]),
    ("vec2", &["vec2"]),
];

/// Alle Built-in-Module, die den externen Typ `t` (lowercase, kanonisch)
/// bereitstellen. Fuer hilfreiche „IMPORT fehlt?"-Compiler-Fehlermeldungen,
/// wenn ein `DIM x AS <Modul-Typ>` ohne das passende IMPORT auftaucht.
pub fn modules_for_type(t: &str) -> Vec<&'static str> {
    MODULE_TYPES.iter()
        .filter(|(_, types)| types.contains(&t))
        .map(|(m, _)| *m)
        .collect()
}

/// Remap eines Namens unter einem Modul-Alias -- gleiche Konvention wie
/// `modules._apply_alias_by_convention`: `<modul>` -> `<alias>`,
/// `<modul>_rest` -> `<alias>_rest`. None, wenn der Name nicht aliasable ist.
fn remap_name(name: &str, module: &str, alias: &str) -> Option<String> {
    if name == module {
        Some(alias.to_string())
    } else {
        name.strip_prefix(&format!("{}_", module))
            .map(|rest| format!("{}_{}", alias, rest))
    }
}

/// Aus den importierten Modulen (jeweils mit optionalem Alias) die Compile-
/// Umgebung ableiten:
///   - `external_types`: gueltige skalare DIM-Typen (kanonisch + aliasiert),
///   - `builtin_aliases`: `(alias, modul)`-Paare, mit denen der Compiler
///     aliasierte Builtin-Namen (`j_parse` -> `json_parse`) zurueckabbildet.
/// Alles lowercase (wie der Lexer IDENTs liefert).
pub fn compile_env(imports: &[(String, Option<String>)])
    -> (HashSet<String>, Vec<(String, String)>) {
    let mut types = HashSet::new();
    let mut aliases: Vec<(String, String)> = Vec::new();
    for (module, alias) in imports {
        let mod_types: &[&str] = MODULE_TYPES.iter()
            .find(|(n, _)| n == module).map(|(_, t)| *t).unwrap_or(&[]);
        for t in mod_types {
            types.insert((*t).to_string());
        }
        if let Some(a) = alias {
            if !aliases.iter().any(|(x, y)| x == a && y == module) {
                aliases.push((a.clone(), module.clone()));
            }
            for t in mod_types {
                if let Some(at) = remap_name(t, module, a) {
                    types.insert(at);
                }
            }
        }
    }
    (types, aliases)
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

/// Ist das Hardware-Modul `m` in DIESEM dhrt-Build einkompiliert? Fuer
/// Nicht-Hardware-Module immer `true`. Spiegelt die Cargo-Features, die
/// `vm.rs` zum Dispatch der `serial_`/`usb_`/`bt_`/`wifi_`-Builtins braucht.
fn hardware_module_compiled(m: &str) -> bool {
    match m {
        "serial" => cfg!(feature = "serial"),
        "firmata" => cfg!(feature = "serial"),
        "usb" => cfg!(feature = "usb"),
        "bt" => cfg!(feature = "bt"),
        "wifi" => cfg!(feature = "wifi"),
        _ => true,
    }
}

/// Einheitliche „Hardware-Modul fehlt im Build"-Meldung -- gleiche
/// Handlungsanweisung wie der spaetere Laufzeitfehler in
/// `vm.rs::unknown_builtin_msg`, damit die fruehe IMPORT-Warnung denselben
/// Wortlaut hat wie der Fehler beim ersten Aufruf.
pub fn hardware_missing_msg(module: &str) -> String {
    format!(
        "Hardware-Modul '{}' ist in diesem dhrt-Build nicht enthalten -- \
         Funktionsaufrufe schlagen zur Laufzeit fehl. Neu bauen mit: \
         python rust\\build_runtime.py --hardware",
        module)
}

/// Aus den importierten Modulen die Hardware-Module heraussuchen, die in diesem
/// Build fehlen (importierbar, aber jeder Aufruf wuerde zur Laufzeit scheitern).
/// Basis fuer eine Warnung schon beim IMPORT statt erst beim ersten Aufruf.
pub fn missing_hardware_modules(imports: &[(String, Option<String>)]) -> Vec<&'static str> {
    HARDWARE_MODULES.iter()
        .filter(|&&m| !hardware_module_compiled(m)
            && imports.iter().any(|(name, _)| name == m))
        .copied()
        .collect()
}

/// IMPORT-Zeilen fehlender Hardware-Module direkt in `source` (nur diese Quelle,
/// keine inkludierten Dateien). Liefert `(zeile, modul)` -- so kann die Editor-
/// Diagnostik die Warnung exakt auf die IMPORT-Zeile setzen.
pub fn missing_hardware_imports_with_lines(source: &str) -> Vec<(usize, &'static str)> {
    let mut out = Vec::new();
    for (idx0, raw_line) in source.split('\n').enumerate() {
        let raw = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if let Some(caps) = import_re().captures(raw) {
            let rel = caps.get(1).unwrap().as_str().to_lowercase();
            if let Some(&m) = HARDWARE_MODULES.iter().find(|&&h| h == rel) {
                if !hardware_module_compiled(m) {
                    out.push((idx0 + 1, m));
                }
            }
        }
    }
    out
}

/// Expandiert alle IMPORTs rekursiv. Liefert `(gemergte_quelle, imports)`,
/// wobei `imports` die Built-in-Modul-IMPORTs als `(modul_lc, alias_lc?)`
/// enthaelt -- Basis fuer `compile_env` (externe Typen + Builtin-Aliase).
/// `base` ist das Verzeichnis fuer relative IMPORT-Pfade.
pub fn process(source: &str, base: &Path)
    -> Result<(String, Vec<(String, Option<String>)>), PreprocessError> {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    let mut imports: Vec<(String, Option<String>)> = Vec::new();
    process_inner(source, base, &mut seen, &mut out, &mut imports)?;
    Ok((out.join("\n"), imports))
}

fn process_inner(
    source: &str,
    base: &Path,
    seen: &mut HashSet<PathBuf>,
    out: &mut Vec<String>,
    imports: &mut Vec<(String, Option<String>)>,
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
            // Fallback: Built-in-Modul (json/db/ui/...). dhrt hat sie nativ,
            // also nur einen Kommentar-Marker emittieren.
            if looks_like_module_name(&rel) && is_known_module(&rel) {
                let tag = match &alias {
                    Some(a) => format!(" AS {}", a),
                    None => String::new(),
                };
                out.push(format!("' === IMPORT MODULE {}{} ===", rel, tag));
                let low = rel.to_lowercase();
                let alias_lc = alias.as_ref().map(|a| a.to_lowercase());
                let entry = (low, alias_lc);
                if !imports.contains(&entry) {
                    imports.push(entry);
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
        // Ein IMPORT-Fehler TIEFER in der Kette trug bisher die Zeile der
        // INNEREN Datei unveraendert nach oben -- der Editor markierte damit
        // eine voellig unbeteiligte Zeile im Puffer des Nutzers. Die einzige
        // Koordinate, die der Nutzer anfassen kann, ist SEINE IMPORT-Zeile;
        // die innere Position gehoert in den Meldungstext (gleiche Behandlung
        // wie in gamebasic/preprocess.py).
        process_inner(&content, &inner_base, seen, out, imports)
            .map_err(|e| PreprocessError {
                line: line_idx,
                msg: format!("in {}: (Zeile {}) {}", rel, e.line, e.msg),
            })?;
        out.push(format!("' === END IMPORT {} ===", rel));
    }
    Ok(())
}
