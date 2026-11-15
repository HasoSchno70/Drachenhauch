//! Schlafen, solange nichts passiert (WINDOW_WAIT_EVENTS).
//!
//! Ein Programm mit Oberflaeche zeichnete jedes Bild neu, sechzig Mal in der
//! Sekunde, auch wenn niemand etwas tat -- am Laptop kostet das Akku und
//! laesst den Luefter laufen. Mit `WINDOW_WAIT_EVENTS(sekunden)` wartet FLIP
//! auf eine Eingabe, hoechstens so lange. Damit dabei nichts stehen bleibt,
//! bestimmt `Graphics::flip` je Bild, wie lange es warten DARF:
//!
//! * gar nicht, solange eine Eingabe kurz zurueckliegt (Nachlauf: weiche
//!   Uebergaenge, die sie ausgeloest hat, laufen zu Ende) oder eine Maustaste
//!   gehalten wird (Ziehen ueber den Rand rollt ohne Bewegung weiter);
//! * hoechstens bis zu dem Bild, das die gui als naechstes braucht (die
//!   Schreibmarke blinkt um, ein Hilfetext ist faellig, eine Animation
//!   laeuft) -- `Gui::bild_bedarf_nehmen`;
//! * hoechstens so lange, wie das Programm erlaubt hat.
//!
//! Gewartet wird in raylibs `PollInputEvents` (glfwWaitEvents, EnableEvent-
//! Waiting) -- nur dort sieht raylib die Eingabe als Flanke dieses Bildes.
//! Eine Frist setzt ein Faden durch, der zur Frist ein leeres Ereignis
//! schickt (glfwPostEmptyEvent ist von jedem Faden aus erlaubt).
//!
//! Welche Eingabe es gab, zaehlen Rueckrufe VOR denen von raylib (Maus,
//! Rad, Zeichen, Ablegen; Tasten und Maustasten zaehlt `flanken.rs` mit,
//! die echten Fenster `nativfenster.rs`). Nur, was der Mensch tut: Fokus
//! und Groesse wecken das Warten ohnehin, und beim Start des Fensters
//! kaemen sie sonst als "Eingabe" und hielten es eine Weile wach.
//!
//! Mit fester Bildzahl (DHRT_FRAMES, Testlaeufe und Bildschirmfotos) wird
//! nie gewartet -- ein Lauf mit 300 Bildern dauerte sonst Minuten.

#![cfg_attr(not(feature = "graphics"), allow(dead_code))]

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::Instant;

/// So lange nach einer Eingabe wird ohne Warten gezeichnet (Sekunden).
pub const NACHLAUF: f64 = 0.35;
/// Kuerzere Fristen lohnen das Warten nicht -- dann gleich das naechste Bild.
pub const KLEINSTE_FRIST: f64 = 0.004;

static EREIGNISSE: AtomicU64 = AtomicU64::new(0);

/// Eine Eingabe ist angekommen (aus jedem Rueckruf).
pub fn ereignis() { EREIGNISSE.fetch_add(1, Ordering::Relaxed); }

/// Wie viele Eingaben bisher -- `Graphics` vergleicht mit dem letzten Stand.
pub fn ereignisse() -> u64 { EREIGNISSE.load(Ordering::Relaxed) }

/// Wie lange darf dieses Bild warten? Rein, damit es ohne Fenster pruefbar
/// ist. `max` = Grenze des Programms (0 = Warten aus), `ruhe` = Sekunden seit
/// der letzten Eingabe, `bedarf` = bis zum naechsten Bild der gui (unendlich =
/// keins), `gehalten` = eine Maustaste ist unten. 0 = nicht warten.
pub fn frist_berechnen(max: f64, ruhe: f64, bedarf: f64, gehalten: bool) -> f64 {
    if !(max > 0.0) || gehalten || ruhe < NACHLAUF { return 0.0; }
    let f = max.min(bedarf);
    if f < KLEINSTE_FRIST { 0.0 } else { f }
}

struct Wecker {
    frist: Mutex<Option<Instant>>,
    cv: Condvar,
}

static W: OnceLock<Wecker> = OnceLock::new();

fn wecker() -> &'static Wecker {
    static LAEUFT: AtomicUsize = AtomicUsize::new(0);
    let w = W.get_or_init(|| Wecker { frist: Mutex::new(None), cv: Condvar::new() });
    if LAEUFT.swap(1, Ordering::SeqCst) == 0 {
        std::thread::spawn(move || {
            let w = W.get().unwrap();
            let mut g = w.frist.lock().unwrap();
            loop {
                match *g {
                    None => { g = w.cv.wait(g).unwrap(); }
                    Some(f) => {
                        let jetzt = Instant::now();
                        if jetzt >= f {
                            *g = None;
                            #[cfg(feature = "graphics")]
                            unsafe { glfwPostEmptyEvent(); }
                        } else {
                            g = w.cv.wait_timeout(g, f - jetzt).unwrap().0;
                        }
                    }
                }
            }
        });
    }
    w
}

#[cfg(feature = "graphics")]
unsafe extern "C" {
    fn glfwPostEmptyEvent();
}

/// Fuer das naechste Bild: `sekunden` > 0 = auf eine Eingabe warten,
/// hoechstens so lange; 0 = nicht warten.
#[cfg(feature = "graphics")]
pub fn frist(sekunden: f64) {
    let w = wecker();
    {
        let mut g = w.frist.lock().unwrap();
        *g = if sekunden > 0.0 { Some(Instant::now() + std::time::Duration::from_secs_f64(sekunden)) } else { None };
    }
    w.cv.notify_one();
    unsafe {
        if sekunden > 0.0 { raylib::ffi::EnableEventWaiting(); } else { raylib::ffi::DisableEventWaiting(); }
    }
}

/// Beim Abraeumen des Fensters: nicht mehr wecken (glfwPostEmptyEvent nach
/// glfwTerminate meldet einen GLFW-Fehler).
pub fn aus() {
    let Some(w) = W.get() else { return };
    *w.frist.lock().unwrap() = None;
    w.cv.notify_one();
}

// ------------------------------------------------------------ Rueckrufe
type FnPos = unsafe extern "C" fn(*mut std::ffi::c_void, f64, f64);
type FnUint = unsafe extern "C" fn(*mut std::ffi::c_void, u32);
type FnAblage = unsafe extern "C" fn(*mut std::ffi::c_void, i32, *const *const std::os::raw::c_char);

static V_LAGE: AtomicUsize = AtomicUsize::new(0);
static V_RAD: AtomicUsize = AtomicUsize::new(0);
static V_ZEICHEN: AtomicUsize = AtomicUsize::new(0);
static V_ABLAGE: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn r_lage(f: *mut std::ffi::c_void, x: f64, y: f64) {
    ereignis();
    let v = V_LAGE.load(Ordering::SeqCst);
    if v != 0 { unsafe { std::mem::transmute::<usize, FnPos>(v)(f, x, y) } }
}
unsafe extern "C" fn r_rad(f: *mut std::ffi::c_void, x: f64, y: f64) {
    ereignis();
    let v = V_RAD.load(Ordering::SeqCst);
    if v != 0 { unsafe { std::mem::transmute::<usize, FnPos>(v)(f, x, y) } }
}
unsafe extern "C" fn r_zeichen(f: *mut std::ffi::c_void, c: u32) {
    ereignis();
    let v = V_ZEICHEN.load(Ordering::SeqCst);
    if v != 0 { unsafe { std::mem::transmute::<usize, FnUint>(v)(f, c) } }
}
unsafe extern "C" fn r_ablage(f: *mut std::ffi::c_void, n: i32, p: *const *const std::os::raw::c_char) {
    ereignis();
    let v = V_ABLAGE.load(Ordering::SeqCst);
    if v != 0 { unsafe { std::mem::transmute::<usize, FnAblage>(v)(f, n, p) } }
}

/// Den Vorgaenger merken -- nur, wenn es nicht der eigene Rueckruf ist
/// (zweimal eingehaengt gaebe sonst eine Schleife ohne Ende).
fn merken(platz: &AtomicUsize, alt: usize, eigen: usize) {
    if alt != 0 && alt != eigen { platz.store(alt, Ordering::SeqCst); }
}

/// Die Rueckrufe vor die von raylib haengen; einmal nach dem Anlegen des
/// Hauptfensters (der GL-Kontext ist dann aktuell).
pub fn einhaengen() {
    #[cfg(all(feature = "graphics", not(target_os = "emscripten")))]
    unsafe {
        unsafe extern "C" {
            fn glfwGetCurrentContext() -> *mut std::ffi::c_void;
            fn glfwSetCursorPosCallback(f: *mut std::ffi::c_void, r: Option<FnPos>) -> Option<FnPos>;
            fn glfwSetScrollCallback(f: *mut std::ffi::c_void, r: Option<FnPos>) -> Option<FnPos>;
            fn glfwSetCharCallback(f: *mut std::ffi::c_void, r: Option<FnUint>) -> Option<FnUint>;
            fn glfwSetDropCallback(f: *mut std::ffi::c_void, r: Option<FnAblage>) -> Option<FnAblage>;
        }
        let f = glfwGetCurrentContext();
        if f.is_null() { return; }
        let a = glfwSetCursorPosCallback(f, Some(r_lage)).map(|x| x as usize).unwrap_or(0);
        merken(&V_LAGE, a, r_lage as *const () as usize);
        let a = glfwSetScrollCallback(f, Some(r_rad)).map(|x| x as usize).unwrap_or(0);
        merken(&V_RAD, a, r_rad as *const () as usize);
        let a = glfwSetCharCallback(f, Some(r_zeichen)).map(|x| x as usize).unwrap_or(0);
        merken(&V_ZEICHEN, a, r_zeichen as *const () as usize);
        let a = glfwSetDropCallback(f, Some(r_ablage)).map(|x| x as usize).unwrap_or(0);
        merken(&V_ABLAGE, a, r_ablage as *const () as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ohne_erlaubnis_wird_nie_gewartet() {
        assert_eq!(frist_berechnen(0.0, 10.0, f64::INFINITY, false), 0.0);
    }
    #[test]
    fn nach_einer_eingabe_laeuft_es_nach() {
        assert_eq!(frist_berechnen(1.0, 0.1, f64::INFINITY, false), 0.0);
        assert_eq!(frist_berechnen(1.0, NACHLAUF, f64::INFINITY, false), 1.0);
    }
    #[test]
    fn eine_gehaltene_taste_wartet_nicht() {
        assert_eq!(frist_berechnen(1.0, 10.0, f64::INFINITY, true), 0.0);
    }
    #[test]
    fn die_gui_kuerzt_die_frist() {
        assert_eq!(frist_berechnen(1.0, 10.0, 0.25, false), 0.25);
        assert_eq!(frist_berechnen(1.0, 10.0, 0.0, false), 0.0);
        assert_eq!(frist_berechnen(1.0, 10.0, 0.001, false), 0.0);
    }
}
