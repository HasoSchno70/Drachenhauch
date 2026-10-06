//! Fenster, die keine echte Eingabe annehmen -- fuer `dhrt test`.
//!
//! Eine Pruefsammlung bedient ihre Fenster ueber Aufnahmen
//! (`AUTOMATION_PLAY`) und ueber Nachrichten an das Fenster
//! (`fenstersender.ps1`, PostMessage). Echte Tastatur und Maus braucht
//! keiner der Faelle -- bekam ein Fenster sie trotzdem, war der Fall
//! verdorben: wer waehrend eines Laufs im Chat tippte, schrieb in die IDE
//! eines Falls ("BSCREEN" statt "SCREEN", ein neuer Reiter mit nur einem
//! "e"), und eine Maus ueber dem Fenster nahm dem Fall seinen Tooltip.
//! Gesehen in schnellen Laeufen, einzeln war jeder Fall gruen.
//!
//! Darum setzt `dhrt test` jedem Programm `DHRT_OHNE_EINGABE`, und ein
//! Fenster unter dem Schalter
//! * geht ohne Fokus auf (GLFW_FOCUS_ON_SHOW),
//! * laesst die Maus durch auf das Fenster darunter (GLFW_MOUSE_PASSTHROUGH;
//!   nachgeschickte Nachrichten kommen trotzdem an, sie gehen an das
//!   Fenster selbst und nicht ueber die Trefferpruefung),
//! * gibt den Vordergrund zurueck, wenn es ihn doch bekommen hat
//!   (Maximieren, `WINDOW_FOCUS`) -- unter Windows, je Bild geprueft.
//!
//! Fuer das Programm gilt es trotzdem als vorn (`WINDOW_FOCUSED` ist TRUE):
//! was am Fokus haengt, soll nicht davon abhaengen, wohin der Nutzer gerade
//! geklickt hat.

use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::sync::atomic::AtomicIsize;

static AN: AtomicBool = AtomicBool::new(false);
static GEPRUEFT: AtomicBool = AtomicBool::new(false);
/// Das Fenster, das beim Anlegen vorn war (unter Windows), 0 = keins.
#[cfg(windows)]
static FREMD_VORN: AtomicIsize = AtomicIsize::new(0);

/// Ob der Schalter gesetzt ist (einmal gelesen).
pub fn an() -> bool {
    if !GEPRUEFT.swap(true, Ordering::SeqCst) {
        // "0" oder leer schaltet ab -- so laesst sich die Gegenprobe in einer
        // Sammlung schreiben, deren Laeufer den Schalter setzt.
        let wert = std::env::var("DHRT_OHNE_EINGABE").unwrap_or_default();
        AN.store(!wert.is_empty() && wert != "0", Ordering::SeqCst);
    }
    AN.load(Ordering::SeqCst)
}

/// Nach dem Anlegen, solange das Fenster noch versteckt ist.
pub fn einrichten(_hwnd: *mut std::ffi::c_void) {
    if !an() { return; }
    #[cfg(all(feature = "graphics", not(target_os = "emscripten")))]
    unsafe {
        unsafe extern "C" {
            fn glfwGetCurrentContext() -> *mut std::ffi::c_void;
            fn glfwSetWindowAttrib(fenster: *mut std::ffi::c_void, art: i32, wert: i32);
        }
        const GLFW_FOCUS_ON_SHOW: i32 = 0x0002_000C;
        const GLFW_MOUSE_PASSTHROUGH: i32 = 0x0002_000D;
        let f = glfwGetCurrentContext();
        if !f.is_null() {
            glfwSetWindowAttrib(f, GLFW_FOCUS_ON_SHOW, 0);
            glfwSetWindowAttrib(f, GLFW_MOUSE_PASSTHROUGH, 1);
        }
    }
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
        let vorn = unsafe { GetForegroundWindow() }.0 as isize;
        if vorn != 0 && vorn != _hwnd as isize { FREMD_VORN.store(vorn, Ordering::SeqCst); }
    }
}

/// Einmal je Bild: liegt das eigene Fenster vorn, geht der Vordergrund an
/// das Fenster zurueck, das ihn beim Anlegen hatte. Das darf ein Prozess,
/// solange er selbst vorn ist.
pub fn zurueckgeben(_hwnd: *mut std::ffi::c_void) {
    if !an() { return; }
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, IsWindow, SetForegroundWindow};
        let fremd = FREMD_VORN.load(Ordering::SeqCst);
        if fremd == 0 { return; }
        unsafe {
            if GetForegroundWindow().0 as isize != _hwnd as isize { return; }
            let f = HWND(fremd as *mut std::ffi::c_void);
            if IsWindow(Some(f)).as_bool() { let _ = SetForegroundWindow(f); }
        }
    }
}
