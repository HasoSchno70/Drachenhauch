//! Dateien, die der Finder an die App uebergibt (macOS, 2026-09-19).
//!
//! macOS reicht eine per Doppelklick geoeffnete Datei NICHT als Argument
//! weiter, sondern als Apple-Event "Dokumente oeffnen" an die laufende App --
//! beim Start genauso wie spaeter. NSApplication gibt es an seinen Delegate
//! weiter (`application:openURLs:`), und der Delegate ist GLFWs
//! `GLFWApplicationDelegate`. Der kennt die Methode nicht: das Ereignis ging
//! bisher still verloren.
//!
//! Darum bekommt die Klasse die Methode hier nachgereicht (`class_addMethod`),
//! BEVOR GLFW sein NSApplication hochfaehrt -- das Start-Ereignis kommt schon
//! in dessen `[NSApp run]` an. Die Pfade landen in einer Warteschlange, die
//! `Graphics` je Bild abholt; nach aussen sind sie abgelegte Dateien wie beim
//! Hineinziehen (FILES_DROPPED/FILE_DROPPED).
//!
//! Nur die Objective-C-Laufzeit, keine Crates: das Modul uebersetzt so auf
//! jedem macOS-Lauf der CI mit, auch ohne Grafik. Auf anderen Systemen sind
//! beide Funktionen leer.
//!
//! `DHRT_OEFFNEN_PROTOKOLL` (nur fuer Pruefungen): jeder uebergebene Pfad
//! wird dort zusaetzlich angehaengt -- auf den macOS-Laeufern der CI geht
//! kein Fenster auf, die IDE kommt also nie dazu, die Datei zu zeigen; das
//! Protokoll belegt, dass das Ereignis angekommen ist.
#![allow(dead_code)]

use std::sync::Mutex;

static WARTEND: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Was seit dem letzten Abholen uebergeben wurde.
pub fn abholen() -> Vec<String> {
    match WARTEND.lock() {
        Ok(mut w) => std::mem::take(&mut *w),
        Err(_) => Vec::new(),
    }
}

fn merken(pfad: String) {
    if let Ok(p) = std::env::var("DHRT_OEFFNEN_PROTOKOLL") {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{}", pfad);
        }
    }
    if let Ok(mut w) = WARTEND.lock() {
        w.push(pfad);
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::{c_char, c_void, CStr};

    type Id = *mut c_void;
    type Sel = *mut c_void;

    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn class_addMethod(cls: Id, sel: Sel, imp: *const c_void, types: *const c_char) -> bool;
        fn objc_msgSend();
    }
    // NSArray/NSURL/NSString -- AppKit bringt Foundation mit, ausdruecklich
    // genannt, damit auch ein Bau ohne raylib linkt.
    #[link(name = "Foundation", kind = "framework")]
    extern "C" {}

    fn sel(name: &CStr) -> Sel {
        unsafe { sel_registerName(name.as_ptr()) }
    }

    // objc_msgSend muss auf arm64 mit der genauen Signatur gerufen werden.
    unsafe fn senden_usize(ziel: Id, s: Sel) -> usize {
        let f: extern "C" fn(Id, Sel) -> usize = std::mem::transmute(objc_msgSend as *const ());
        f(ziel, s)
    }
    unsafe fn senden_id(ziel: Id, s: Sel) -> Id {
        let f: extern "C" fn(Id, Sel) -> Id = std::mem::transmute(objc_msgSend as *const ());
        f(ziel, s)
    }
    unsafe fn senden_id_usize(ziel: Id, s: Sel, i: usize) -> Id {
        let f: extern "C" fn(Id, Sel, usize) -> Id = std::mem::transmute(objc_msgSend as *const ());
        f(ziel, s, i)
    }

    /// - (void)application:(NSApplication*)app openURLs:(NSArray<NSURL*>*)urls
    extern "C" fn oeffne_urls(_selbst: Id, _cmd: Sel, _app: Id, urls: Id) {
        if urls.is_null() { return; }
        unsafe {
            let n = senden_usize(urls, sel(c"count"));
            for i in 0..n {
                let url = senden_id_usize(urls, sel(c"objectAtIndex:"), i);
                if url.is_null() { continue; }
                let pfad = senden_id(url, sel(c"path"));
                if pfad.is_null() { continue; }
                let text = senden_id(pfad, sel(c"UTF8String")) as *const c_char;
                if text.is_null() { continue; }
                super::merken(CStr::from_ptr(text).to_string_lossy().into_owned());
            }
        }
    }

    pub fn einhaengen() {
        static EINMAL: std::sync::Once = std::sync::Once::new();
        EINMAL.call_once(|| unsafe {
            let klasse = objc_getClass(c"GLFWApplicationDelegate".as_ptr());
            if klasse.is_null() { return; }
            // v@:@@ = void, self, _cmd, NSApplication*, NSArray*
            class_addMethod(klasse, sel(c"application:openURLs:"),
                            oeffne_urls as *const c_void, c"v@:@@".as_ptr());
        });
    }
}

/// `DHRT_ABLEGEN` (nur fuer Pruefungen, Pfade mit `;`): in DIESELBE
/// Warteschlange wie der Finder -- so prueft jeder Test auch das Abholen.
/// Ein echtes Hineinziehen kann kein Test ausloesen. Einmal je Prozess.
pub fn vortaeuschen() {
    static EINMAL: std::sync::Once = std::sync::Once::new();
    EINMAL.call_once(|| {
        if let Ok(s) = std::env::var("DHRT_ABLEGEN") {
            for p in s.split(';').filter(|p| !p.is_empty()) {
                merken(p.to_string());
            }
        }
    });
}

/// Vor dem ersten Fenster aufrufen (GLFW faehrt NSApplication dort hoch).
pub fn einhaengen() {
    #[cfg(target_os = "macos")]
    mac::einhaengen();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warteschlange_liefert_einmal() {
        merken("/a.dh".into());
        merken("/b.dh".into());
        assert_eq!(abholen(), vec!["/a.dh".to_string(), "/b.dh".to_string()]);
        assert!(abholen().is_empty());
    }
}
