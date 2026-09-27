//! Echte Systemzeiger unter Windows: "warten" (die sich drehende Sanduhr),
//! "arbeitet" (Pfeil mit Sanduhr) und "hilfe" -- GLFW kennt sie nicht, und
//! nachgemalt drehte sich nichts.
//!
//! Windows fragt die Zeigerform bei jeder Mausbewegung mit `WM_SETCURSOR` ab;
//! GLFW antwortet dort mit seinem eigenen Zeiger. Ein weiterer Subclass am
//! Fenster (wie `ime.rs`, die Kette ruft den Vorgaenger) antwortet vorher,
//! solange ein Systemzeiger gilt, und reicht sonst alles durch.

#[cfg(windows)]
mod plattform {
    use std::sync::atomic::{AtomicIsize, Ordering};
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, LoadCursorW, SetCursor, SetWindowLongPtrW, GWLP_WNDPROC, HCURSOR, HTCLIENT,
        IDC_APPSTARTING, IDC_HELP, IDC_WAIT, WM_SETCURSOR, WNDPROC,
    };

    static ALT_PROC: AtomicIsize = AtomicIsize::new(0);
    /// Der geltende Systemzeiger (HCURSOR), 0 = GLFW entscheidet.
    static JETZT: AtomicIsize = AtomicIsize::new(0);

    unsafe extern "system" fn fensterprozedur(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        let h = JETZT.load(Ordering::SeqCst);
        if msg == WM_SETCURSOR && h != 0 && (lp.0 & 0xFFFF) as u32 == HTCLIENT {
            unsafe { SetCursor(Some(HCURSOR(h as *mut _))); }
            return LRESULT(1);
        }
        let alt = ALT_PROC.load(Ordering::SeqCst);
        let vorgaenger: WNDPROC = unsafe { std::mem::transmute::<isize, WNDPROC>(alt) };
        unsafe { CallWindowProcW(vorgaenger, hwnd, msg, wp, lp) }
    }

    pub fn einhaengen(handle: *mut std::ffi::c_void) {
        if handle.is_null() || ALT_PROC.load(Ordering::SeqCst) != 0 { return; }
        let alt = unsafe { SetWindowLongPtrW(HWND(handle), GWLP_WNDPROC, fensterprozedur as *const () as isize) };
        if alt != 0 { ALT_PROC.store(alt, Ordering::SeqCst); }
    }

    /// Einen Systemzeiger setzen (Name aus `hat`) oder mit None GLFW
    /// zurueckgeben. Gesetzt wird SOFORT -- WM_SETCURSOR kaeme erst mit der
    /// naechsten Bewegung, und eine stillstehende Maus zeigte den alten.
    pub fn setzen(name: Option<&str>) {
        let id: Option<PCWSTR> = match name {
            Some("warten") => Some(IDC_WAIT),
            Some("arbeitet") => Some(IDC_APPSTARTING),
            Some("hilfe") => Some(IDC_HELP),
            _ => None,
        };
        match id.and_then(|id| unsafe { LoadCursorW(None, id) }.ok()) {
            Some(h) => {
                JETZT.store(h.0 as isize, Ordering::SeqCst);
                if ALT_PROC.load(Ordering::SeqCst) != 0 { unsafe { SetCursor(Some(h)); } }
            }
            None => JETZT.store(0, Ordering::SeqCst),
        }
    }

    /// Gibt es diese Form als echten Systemzeiger?
    pub fn hat(name: &str) -> bool { matches!(name, "warten" | "arbeitet" | "hilfe") }
}

#[cfg(not(windows))]
mod plattform {
    pub fn einhaengen(_handle: *mut std::ffi::c_void) {}
    pub fn setzen(_name: Option<&str>) {}
    pub fn hat(_name: &str) -> bool { false }
}

pub use plattform::{einhaengen, hat, setzen};
