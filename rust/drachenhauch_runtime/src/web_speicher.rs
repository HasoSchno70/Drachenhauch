//! Im Browser ueberlebt, was ein Programm schreibt, das Neuladen der Seite.
//!
//! emscripten legt Dateien in den Arbeitsspeicher (MEMFS): ein Highscore, ein
//! Spielstand, eine Einstellung -- nach dem Neuladen war alles weg, ohne ein
//! Wort (Messdaten-/Web-Runde, 2026-10-13). Jetzt ist das Arbeitsverzeichnis
//! `/spiel` ein IndexedDB-Ordner (IDBFS): beim Start wird er aus der Datenbank
//! gefuellt, waehrend des Laufs hoechstens einmal je Sekunde und am Ende
//! zurueckgeschrieben. Ein Programm schreibt also genau wie auf dem Desktop
//! `highscore.txt` -- relativ, ohne etwas vom Browser zu wissen.
//!
//! Die mitgelieferten Dateien (`--preload-file`, liegen unter `/assets`)
//! erreicht es weiter unter `assets/...`: `/spiel/assets` ist ein VERWEIS
//! darauf. IDBFS sammelt mit `lstat` und speichert einen Verweis als Verweis --
//! die Assets landen also nicht ein zweites Mal in der Datenbank (nachgesehen
//! in `libidbfs.js` der emsdk-Fassung, mit der gebaut wird).
//!
//! Der Ordner gehoert der SEITE, nicht dem Programm: alle Programme einer
//! Adresse teilen ihn wie einen Ordner auf der Platte.
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint};
use std::sync::Mutex;
use std::time::Instant;

extern "C" {
    fn emscripten_run_script(script: *const c_char);
    fn emscripten_run_script_int(script: *const c_char) -> c_int;
    fn emscripten_sleep(ms: c_uint);
}

pub const ORDNER: &str = "/spiel";

fn js(s: &str) {
    if let Ok(c) = CString::new(s) {
        unsafe { emscripten_run_script(c.as_ptr()) }
    }
}

fn js_zahl(s: &str) -> i32 {
    match CString::new(s) {
        Ok(c) => unsafe { emscripten_run_script_int(c.as_ptr()) },
        Err(_) => 0,
    }
}

static LETZTE: Mutex<Option<Instant>> = Mutex::new(None);

/// Vor dem Programm: Ordner einhaengen, aus der Datenbank fuellen (hoechstens
/// 5 s warten -- ohne IndexedDB, etwa in einem privaten Fenster mancher
/// Browser, laeuft das Programm trotzdem, nur eben ohne Gedaechtnis), Verweis
/// auf die Assets anlegen und hineinwechseln.
pub fn einrichten() {
    js("Module.dhSync = 0; \
        try { FS.mkdir('/spiel'); } catch (e) {} \
        try { FS.mount(IDBFS, {}, '/spiel'); \
              FS.syncfs(true, function (e) { Module.dhSync = e ? 2 : 1; }); } \
        catch (e) { Module.dhSync = 3; }");
    let mut n = 0;
    while js_zahl("Module.dhSync | 0") == 0 && n < 500 {
        unsafe { emscripten_sleep(10) };
        n += 1;
    }
    js("try { if (FS.analyzePath('/assets').exists && !FS.analyzePath('/spiel/assets').exists) \
              FS.symlink('/assets', '/spiel/assets'); } catch (e) {}");
    let _ = std::env::set_current_dir(ORDNER);
    if let Ok(mut l) = LETZTE.lock() { *l = Some(Instant::now()); }
}

/// Zurueckschreiben, aber nicht oefter als einmal je Sekunde und nie zwei
/// Laeufe gleichzeitig (`syncfs` arbeitet im Hintergrund weiter). Aus FLIP.
pub fn takt() {
    let faellig = match LETZTE.lock() {
        Ok(mut l) => match *l {
            Some(t) if t.elapsed().as_millis() >= 1000 => { *l = Some(Instant::now()); true }
            _ => false,
        },
        Err(_) => false,
    };
    if faellig { sichern(); }
}

/// Jetzt zurueckschreiben (am Programmende).
pub fn sichern() {
    js("if (Module.dhSync === 1) { Module.dhSync = 4; \
          FS.syncfs(false, function (e) { Module.dhSync = 1; }); }");
}
