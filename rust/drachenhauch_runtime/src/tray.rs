//! Tray-Symbol und System-Benachrichtigungen (TRAY_*, NOTIFY).
//!
//! Ein Werkzeug, das im Hintergrund laeuft, braucht einen Platz, an dem man
//! es findet, wenn sein Fenster weg ist, und einen Weg, sich zu melden.
//!
//! **Windows:** `Shell_NotifyIconW` mit raylibs Fenster als Empfaenger. Ein
//! weiterer Subclass am Fenster (wie `systemzeiger.rs` und `ime.rs`, die Kette
//! ruft den Vorgaenger) nimmt die Nachrichten des Symbols entgegen: Klick,
//! Doppelklick, rechte Taste (oeffnet das Menue), Klick auf eine
//! Benachrichtigung. Die Menuewahl kommt als `WM_COMMAND` an -- dieselbe
//! Nachricht kann eine Pruefung von aussen schicken, ohne das Menue zu
//! oeffnen. Die Ereignisse sammelt der Subclass; `abholen` macht sie beim
//! FLIP fuer genau ein Bild sichtbar, wie Klicks in der gui.
//!
//! Eine Benachrichtigung ist unter Windows eine Sprechblase am Tray-Symbol
//! (Windows 10/11 zeigt sie als Mitteilung); ohne Symbol legt NOTIFY eins an.
//!
//! **macOS/Linux:** Benachrichtigungen ueber `osascript` bzw. `notify-send`;
//! ein Tray-Symbol gibt es dort noch nicht (ein klarer Fehler).

/// Die Nachricht, mit der das Symbol meldet (WM_APP + 0x2D). Pruefungen
/// schicken sie selbst, um einen Klick nachzustellen.
pub const WM_TRAY: u32 = 0x8000 + 0x2D;
/// Menueeintraege kommen als WM_COMMAND mit dieser Nummer plus Platz (ab 0).
pub const MENUE_ID: usize = 0x7100;

/// Was in diesem Bild geschehen ist.
#[derive(Default, Clone)]
pub struct Ereignisse {
    pub klick: bool,
    pub doppel: bool,
    pub menue: Option<String>,
    pub mitteilung: bool,
}

#[cfg(windows)]
mod plattform {
    use super::{Ereignisse, MENUE_ID, WM_TRAY};
    use std::sync::atomic::{AtomicIsize, Ordering};
    use std::sync::Mutex;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows::Win32::Graphics::Gdi::{CreateBitmap, CreateDIBSection, DeleteObject, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ};
    use windows::Win32::UI::Shell::{Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_SHOWTIP,
        NIF_TIP, NIIF_INFO, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIN_BALLOONUSERCLICK, NOTIFYICONDATAW};
    use windows::Win32::UI::WindowsAndMessaging::{AppendMenuW, CallWindowProcW, CreateIconIndirect,
        CreatePopupMenu, DestroyIcon, DestroyMenu, GetCursorPos, LoadIconW, PostMessageW,
        RegisterWindowMessageW, SetForegroundWindow, SetWindowLongPtrW, TrackPopupMenu, GWLP_WNDPROC,
        HICON, ICONINFO, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, TPM_RIGHTBUTTON, WM_COMMAND,
        WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP, WNDPROC};

    #[derive(Default)]
    struct Zustand {
        hwnd: isize,
        sichtbar: bool,
        tipp: String,
        menue: Vec<String>,
        /// Eigenes Symbol (aus einem Bild), 0 = das Standardsymbol.
        icon: isize,
        offen: Ereignisse,
        bild: Ereignisse,
    }

    static ZUSTAND: Mutex<Option<Zustand>> = Mutex::new(None);
    static ALT_PROC: AtomicIsize = AtomicIsize::new(0);
    static NEU_GESTARTET: AtomicIsize = AtomicIsize::new(0);

    fn mit<R>(f: impl FnOnce(&mut Zustand) -> R) -> R {
        let mut g = ZUSTAND.lock().unwrap_or_else(|e| e.into_inner());
        f(g.get_or_insert_with(Zustand::default))
    }

    fn breit(s: &str, n: usize) -> Vec<u16> {
        let mut v: Vec<u16> = s.encode_utf16().take(n - 1).collect();
        v.resize(n, 0);
        v
    }

    fn daten(z: &Zustand) -> NOTIFYICONDATAW {
        let mut d = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: HWND(z.hwnd as *mut _),
            uID: 1,
            uFlags: NIF_ICON | NIF_TIP | NIF_MESSAGE | NIF_SHOWTIP,
            uCallbackMessage: WM_TRAY,
            ..Default::default()
        };
        d.hIcon = if z.icon != 0 { HICON(z.icon as *mut _) }
            else { unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap_or_default() };
        d.szTip.copy_from_slice(&breit(&z.tipp, 128));
        d
    }

    unsafe extern "system" fn fensterprozedur(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        if msg == WM_TRAY {
            match lp.0 as u32 {
                WM_LBUTTONUP => mit(|z| z.offen.klick = true),
                WM_LBUTTONDBLCLK => mit(|z| z.offen.doppel = true),
                WM_RBUTTONUP => menue_zeigen(hwnd),
                NIN_BALLOONUSERCLICK => mit(|z| z.offen.mitteilung = true),
                _ => {}
            }
            return LRESULT(0);
        }
        if msg == WM_COMMAND {
            let id = wp.0 & 0xFFFF;
            if id >= MENUE_ID {
                let gefunden = mit(|z| {
                    let platz = id - MENUE_ID;
                    match z.menue.get(platz) {
                        Some(t) if t != "-" => { z.offen.menue = Some(t.clone()); true }
                        _ => false,
                    }
                });
                if gefunden { return LRESULT(0); }
            }
        }
        // Explorer neu gestartet: das Symbol ist weg und muss neu angemeldet werden.
        let neu = NEU_GESTARTET.load(Ordering::SeqCst);
        if neu != 0 && msg == neu as u32 {
            let d = mit(|z| if z.sichtbar { Some(daten(z)) } else { None });
            if let Some(d) = d { unsafe { let _ = Shell_NotifyIconW(NIM_ADD, &d); } }
        }
        let alt = ALT_PROC.load(Ordering::SeqCst);
        let vorgaenger: WNDPROC = unsafe { std::mem::transmute::<isize, WNDPROC>(alt) };
        unsafe { CallWindowProcW(vorgaenger, hwnd, msg, wp, lp) }
    }

    /// Das Menue an der Maus. Ohne SetForegroundWindow schliesst es sich
    /// nicht, wenn man daneben klickt; das WM_NULL danach ist Microsofts
    /// eigener Rat gegen ein Menue, das beim zweiten Mal sofort zugeht.
    fn menue_zeigen(hwnd: HWND) {
        let eintraege = mit(|z| z.menue.clone());
        if eintraege.is_empty() { return; }
        unsafe {
            let Ok(m) = CreatePopupMenu() else { return };
            for (i, t) in eintraege.iter().enumerate() {
                if t == "-" { let _ = AppendMenuW(m, MF_SEPARATOR, 0, PCWSTR::null()); }
                else {
                    let w: Vec<u16> = t.encode_utf16().chain(std::iter::once(0)).collect();
                    let _ = AppendMenuW(m, MF_STRING, MENUE_ID + i, PCWSTR(w.as_ptr()));
                }
            }
            let mut p = POINT::default();
            let _ = GetCursorPos(&mut p);
            let _ = SetForegroundWindow(hwnd);
            let _ = TrackPopupMenu(m, TPM_RIGHTBUTTON, p.x, p.y, None, hwnd, None);
            let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
            let _ = DestroyMenu(m);
        }
    }

    pub fn einhaengen(handle: *mut std::ffi::c_void) {
        if handle.is_null() { return; }
        mit(|z| z.hwnd = handle as isize);
        if ALT_PROC.load(Ordering::SeqCst) != 0 { return; }
        let alt = unsafe { SetWindowLongPtrW(HWND(handle), GWLP_WNDPROC, fensterprozedur as *const () as isize) };
        if alt != 0 { ALT_PROC.store(alt, Ordering::SeqCst); }
        let n = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
        NEU_GESTARTET.store(n as isize, Ordering::SeqCst);
    }

    /// Ein Symbol aus RGBA-Punkten (oben links zuerst): Farbbild mit
    /// Deckkraft, die Maske bleibt leer -- Windows nimmt bei 32 Bit den Alpha.
    fn symbol_aus(rgba: &[u8], b: i32, h: i32) -> Result<isize, String> {
        unsafe {
            let mut kopf = BITMAPINFO::default();
            kopf.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: b, biHeight: -h, biPlanes: 1, biBitCount: 32,
                biCompression: BI_RGB.0, ..Default::default()
            };
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let farbe = CreateDIBSection(None, &kopf, DIB_RGB_COLORS, &mut bits, None, 0)
                .map_err(|e| format!("Bild fuer das Symbol: {}", e))?;
            let ziel = std::slice::from_raw_parts_mut(bits as *mut u8, (b * h * 4) as usize);
            for (z, q) in ziel.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
                z[0] = q[2]; z[1] = q[1]; z[2] = q[0]; z[3] = q[3];
            }
            let maske = CreateBitmap(b, h, 1, 1, None);
            let info = ICONINFO { fIcon: true.into(), xHotspot: 0, yHotspot: 0, hbmMask: maske, hbmColor: farbe };
            let icon = CreateIconIndirect(&info);
            let _ = DeleteObject(HGDIOBJ(farbe.0));
            let _ = DeleteObject(HGDIOBJ(maske.0));
            Ok(icon.map_err(|e| format!("Symbol: {}", e))?.0 as isize)
        }
    }

    pub fn zeigen(rgba: Option<(&[u8], i32, i32)>, tipp: Option<&str>) -> Result<(), String> {
        let neues = match rgba { Some((p, b, h)) => Some(symbol_aus(p, b, h)?), None => None };
        let (d, war, altes) = mit(|z| {
            let altes = match neues { Some(n) => std::mem::replace(&mut z.icon, n), None => 0 };
            if let Some(t) = tipp { z.tipp = t.to_string(); }
            let war = z.sichtbar;
            z.sichtbar = true;
            (daten(z), war, altes)
        });
        if d.hWnd.0.is_null() { return Err("kein Fenster, an das sich das Symbol haengen koennte".into()); }
        let ok = unsafe { Shell_NotifyIconW(if war { NIM_MODIFY } else { NIM_ADD }, &d) }.as_bool();
        if altes != 0 { unsafe { let _ = DestroyIcon(HICON(altes as *mut _)); } }
        if !ok {
            mit(|z| z.sichtbar = war);
            return Err("Windows hat das Symbol nicht angenommen".into());
        }
        Ok(())
    }

    pub fn tipp(text: &str) -> Result<(), String> {
        let d = mit(|z| { z.tipp = text.to_string(); if z.sichtbar { Some(daten(z)) } else { None } });
        if let Some(d) = d { unsafe { let _ = Shell_NotifyIconW(NIM_MODIFY, &d); } }
        Ok(())
    }

    pub fn menue(eintraege: Vec<String>) { mit(|z| z.menue = eintraege); }

    pub fn weg() {
        let d = mit(|z| { let war = z.sichtbar; z.sichtbar = false; if war { Some(daten(z)) } else { None } });
        if let Some(d) = d { unsafe { let _ = Shell_NotifyIconW(NIM_DELETE, &d); } }
    }

    pub fn sichtbar() -> bool { mit(|z| z.sichtbar) }

    pub fn mitteilen(titel: &str, text: &str, standard_tipp: &str) -> Result<(), String> {
        if !sichtbar() {
            let t = mit(|z| if z.tipp.is_empty() { standard_tipp.to_string() } else { z.tipp.clone() });
            zeigen(None, Some(&t))?;
        }
        let mut d = mit(|z| daten(z));
        d.uFlags = NIF_INFO;
        d.szInfo.copy_from_slice(&breit(if text.is_empty() { " " } else { text }, 256));
        d.szInfoTitle.copy_from_slice(&breit(titel, 64));
        d.dwInfoFlags = NIIF_INFO;
        if unsafe { Shell_NotifyIconW(NIM_MODIFY, &d) }.as_bool() { Ok(()) }
        else { Err("Windows hat die Mitteilung nicht angenommen".into()) }
    }

    pub fn abholen() {
        mit(|z| z.bild = std::mem::take(&mut z.offen));
    }

    pub fn bild() -> Ereignisse { mit(|z| z.bild.clone()) }

}

#[cfg(not(windows))]
mod plattform {
    use super::Ereignisse;

    pub fn einhaengen(_handle: *mut std::ffi::c_void) {}
    pub fn zeigen(_rgba: Option<(&[u8], i32, i32)>, _tipp: Option<&str>) -> Result<(), String> {
        Err("ein Tray-Symbol gibt es bisher nur unter Windows".into())
    }
    pub fn tipp(_text: &str) -> Result<(), String> { zeigen(None, None) }
    pub fn menue(_eintraege: Vec<String>) {}
    pub fn weg() {}
    pub fn sichtbar() -> bool { false }
    pub fn abholen() {}
    pub fn bild() -> Ereignisse { Ereignisse::default() }

    /// macOS: `osascript`; Linux: `notify-send`. Der Text geht als Argument
    /// bzw. ueber eine AppleScript-Zeichenkette mit verdoppelten
    /// Anfuehrungszeichen -- nie ueber eine Shell.
    pub fn mitteilen(titel: &str, text: &str, _standard_tipp: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let q = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
            let skript = format!("display notification \"{}\" with title \"{}\"", q(text), q(titel));
            return std::process::Command::new("osascript").arg("-e").arg(skript).spawn()
                .map(|_| ()).map_err(|e| format!("osascript laesst sich nicht starten ({})", e));
        }
        #[cfg(not(target_os = "macos"))]
        {
            std::process::Command::new("notify-send").arg("--").arg(titel).arg(text).spawn()
                .map(|_| ()).map_err(|e| format!("notify-send laesst sich nicht starten ({}) -- unter Debian/Ubuntu im Paket libnotify-bin", e))
        }
    }
}

pub use plattform::{abholen, bild, einhaengen, menue, mitteilen, sichtbar, tipp, weg, zeigen};
