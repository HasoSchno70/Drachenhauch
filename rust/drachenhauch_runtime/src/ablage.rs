//! Bilder und Dateilisten ueber die Zwischenablage, und Ziehen aus der
//! Anwendung hinaus (gui-Luecke "nach aussen").
//!
//! Text geht weiter ueber GLFW (`Graphics::clipboard_get/set`). Bilder und
//! Dateilisten kann GLFW nicht -- hier uebernimmt `arboard` (Windows,
//! macOS, Linux mit X11; unter Wayland ueber XWayland). Mit
//! `DHRT_ZWISCHENABLAGE=privat` (jedes Kind von `dhrt test`) bleibt beides
//! wie der Text im Prozess.
//!
//! Ziehen hinaus (`ziehen`): unter Windows ein echtes OLE-Ziehen ueber die
//! Shell (`SHDoDragDrop`) -- Dateien als Shell-Objekte (Explorer, Mail,
//! jedes Programm, das Dateien annimmt), Text als CF_UNICODETEXT. Es startet
//! NUR, solange die linke Maustaste wirklich gedrueckt ist: ohne Taste
//! legte das System sofort dort ab, wo der Zeiger gerade steht.

#![allow(dead_code)]

use std::sync::Mutex;

/// Ein Bild als RGBA, Zeile fuer Zeile.
pub struct Bild {
    pub b: u32,
    pub h: u32,
    pub px: Vec<u8>,
}

static PRIVAT_BILD: Mutex<Option<(u32, u32, Vec<u8>)>> = Mutex::new(None);
static PRIVAT_DATEIEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn privat() -> bool {
    std::env::var("DHRT_ZWISCHENABLAGE").as_deref() == Ok("privat")
}

/// Ein Bild in die Zwischenablage legen. Err mit dem Grund.
pub fn bild_setzen(b: u32, h: u32, px: Vec<u8>) -> Result<(), String> {
    if (b as usize) * (h as usize) * 4 != px.len() || b == 0 || h == 0 {
        return Err("das Bild ist leer".into());
    }
    if privat() {
        *PRIVAT_BILD.lock().unwrap_or_else(|e| e.into_inner()) = Some((b, h, px));
        return Ok(());
    }
    #[cfg(feature = "dialogs")]
    {
        let mut a = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        return a.set_image(arboard::ImageData { width: b as usize, height: h as usize, bytes: px.into() })
            .map_err(|e| e.to_string());
    }
    #[allow(unreachable_code)]
    Err("Bilder in der Zwischenablage gibt es in diesem Bau nicht".into())
}

/// Das Bild aus der Zwischenablage -- None, wenn keines darin ist.
pub fn bild_holen() -> Option<Bild> {
    if privat() {
        return PRIVAT_BILD.lock().unwrap_or_else(|e| e.into_inner()).clone().map(|(b, h, px)| Bild { b, h, px });
    }
    #[cfg(feature = "dialogs")]
    {
        let mut a = arboard::Clipboard::new().ok()?;
        let i = a.get_image().ok()?;
        if i.width == 0 || i.height == 0 || i.bytes.len() < i.width * i.height * 4 { return None; }
        return Some(Bild { b: i.width as u32, h: i.height as u32, px: i.bytes.into_owned() });
    }
    #[allow(unreachable_code)]
    None
}

/// Dateien (Pfade) in die Zwischenablage legen -- im Explorer/Finder
/// "Einfuegen" kopiert sie.
pub fn dateien_setzen(pfade: &[String]) -> Result<(), String> {
    if pfade.is_empty() { return Err("keine Dateien".into()); }
    if privat() {
        *PRIVAT_DATEIEN.lock().unwrap_or_else(|e| e.into_inner()) = pfade.to_vec();
        return Ok(());
    }
    #[cfg(feature = "dialogs")]
    {
        let absolut: Vec<std::path::PathBuf> = pfade.iter()
            .map(|p| std::path::absolute(p).unwrap_or_else(|_| std::path::PathBuf::from(p))).collect();
        let mut a = arboard::Clipboard::new().map_err(|e| e.to_string())?;
        return a.set().file_list(&absolut).map_err(|e| e.to_string());
    }
    #[allow(unreachable_code)]
    Err("Dateien in der Zwischenablage gibt es in diesem Bau nicht".into())
}

/// Die Dateien aus der Zwischenablage (leer, wenn keine darin sind).
pub fn dateien_holen() -> Vec<String> {
    if privat() { return PRIVAT_DATEIEN.lock().unwrap_or_else(|e| e.into_inner()).clone(); }
    #[cfg(feature = "dialogs")]
    {
        let Ok(mut a) = arboard::Clipboard::new() else { return Vec::new() };
        return a.get().file_list().map(|l| l.into_iter().map(|p| p.to_string_lossy().into_owned()).collect()).unwrap_or_default();
    }
    #[allow(unreachable_code)]
    Vec::new()
}

/// Was beim Ziehen hinaus herauskam: "kopieren", "verschieben",
/// "verknuepfen" -- oder "" (abgebrochen, nirgends abgelegt, keine Taste
/// gedrueckt, auf diesem System nicht moeglich).
pub fn wirkung_name(w: u32) -> &'static str {
    if w & 2 != 0 { "verschieben" } else if w & 1 != 0 { "kopieren" } else if w & 4 != 0 { "verknuepfen" } else { "" }
}

/// Ziehen aus dem Fenster `hwnd` hinaus: Dateien (wenn welche) oder Text.
/// Blockiert, bis abgelegt oder abgebrochen ist -- das System fuehrt den
/// Zug (Zeiger, Ablageziele, ESC).
#[allow(unused_variables)]
pub fn ziehen(hwnd: usize, dateien: &[String], text: &str) -> String {
    #[cfg(all(feature = "dialogs", windows))]
    {
        return match unsafe { ziehen_win(hwnd, dateien, text) } { Ok(w) => wirkung_name(w).to_string(), Err(_) => String::new() };
    }
    #[allow(unreachable_code)]
    String::new()
}

/// Ist die linke Maustaste gerade wirklich gedrueckt? (Nur dann darf ein
/// Zug hinaus beginnen.)
pub fn taste_unten() -> bool {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
        return (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0;
    }
    #[allow(unreachable_code)]
    false
}

#[cfg(all(feature = "dialogs", windows))]
unsafe fn ziehen_win(hwnd: usize, dateien: &[String], text: &str) -> windows::core::Result<u32> {
    use windows::core::HSTRING;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CoTaskMemFree, IDataObject, DVASPECT_CONTENT, FORMATETC, STGMEDIUM, STGMEDIUM_0, TYMED_HGLOBAL};
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Ole::{OleInitialize, CF_UNICODETEXT, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_LINK, DROPEFFECT_MOVE};
    use windows::Win32::UI::Shell::Common::ITEMIDLIST;
    use windows::Win32::UI::Shell::{BHID_DataObject, IShellItemArray, SHCreateDataObject, SHCreateShellItemArrayFromIDLists, SHDoDragDrop, SHParseDisplayName};
    if !taste_unten() { return Ok(0); }
    // Schon initialisiert (S_FALSE) ist recht; ein anderes Modell (MTA) laesst
    // OLE-Ziehen nicht zu -- dann scheitert SHDoDragDrop und es bleibt bei "".
    let _ = unsafe { OleInitialize(None) };
    let daten: IDataObject = if !dateien.is_empty() {
        let mut pidls: Vec<*const ITEMIDLIST> = Vec::new();
        for d in dateien {
            let pfad = std::path::absolute(d).unwrap_or_else(|_| std::path::PathBuf::from(d));
            let mut p: *mut ITEMIDLIST = std::ptr::null_mut();
            let r = unsafe { SHParseDisplayName(&HSTRING::from(pfad.as_os_str()), None, &mut p, 0, None) };
            if r.is_err() || p.is_null() {
                for q in pidls { unsafe { CoTaskMemFree(Some(q as *const _)) }; }
                return Err(r.err().unwrap_or_else(windows::core::Error::empty));
            }
            pidls.push(p as *const ITEMIDLIST);
        }
        let liste: windows::core::Result<IShellItemArray> = unsafe { SHCreateShellItemArrayFromIDLists(&pidls) };
        for q in &pidls { unsafe { CoTaskMemFree(Some(*q as *const _)) }; }
        unsafe { liste?.BindToHandler(None, &BHID_DataObject)? }
    } else {
        let d: IDataObject = unsafe { SHCreateDataObject(None, None, None)? };
        let w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let h = unsafe { GlobalAlloc(GMEM_MOVEABLE, w.len() * 2)? };
        let p = unsafe { GlobalLock(h) } as *mut u16;
        if p.is_null() { return Ok(0); }
        unsafe { std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len()); let _ = GlobalUnlock(h); }
        let fmt = FORMATETC { cfFormat: CF_UNICODETEXT.0, ptd: std::ptr::null_mut(), dwAspect: DVASPECT_CONTENT.0, lindex: -1, tymed: TYMED_HGLOBAL.0 as u32 };
        let med = STGMEDIUM { tymed: TYMED_HGLOBAL.0 as u32, u: STGMEDIUM_0 { hGlobal: h }, pUnkForRelease: std::mem::ManuallyDrop::new(None) };
        unsafe { d.SetData(&fmt, &med, true)? };
        d
    };
    let wirkung: DROPEFFECT = unsafe { SHDoDragDrop(Some(HWND(hwnd as *mut _)), &daten, None, DROPEFFECT_COPY | DROPEFFECT_MOVE | DROPEFFECT_LINK)? };
    Ok(wirkung.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wirkung_heisst() {
        assert_eq!(wirkung_name(0), "");
        assert_eq!(wirkung_name(1), "kopieren");
        assert_eq!(wirkung_name(2), "verschieben");
        assert_eq!(wirkung_name(3), "verschieben");
        assert_eq!(wirkung_name(4), "verknuepfen");
    }
}
