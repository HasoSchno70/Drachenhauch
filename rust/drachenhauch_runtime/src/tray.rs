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
//! **macOS:** ein `NSStatusItem` in der Menueleiste; **Linux:** ein
//! StatusNotifierItem mit dbusmenu ueber D-Bus (Feature `tray`, beide unten
//! bei ihrem Modul beschrieben). Mitteilungen dort ueber `osascript` bzw.
//! `notify-send`; einen Klick darauf meldet keins von beiden.
//!
//! `dhrt pruef tray` laesst das Symbol des Systems einmal echt durchlaufen
//! (anlegen, zuruecklesen, Klick und Menuewahl) -- so prueft die CI macOS und
//! Linux auch in Bauten ohne Grafik.
// Ein Bau ohne Grafik (Feature tray allein) braucht nur den Selbsttest.
#![cfg_attr(not(feature = "graphics"), allow(dead_code, unused_imports))]

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

    /// Unter Windows braucht das Symbol ein Fenster; das pruefen die Faelle
    /// in tray.dhtest mit echten Fensternachrichten.
    pub fn selbsttest() -> Result<Vec<String>, String> {
        Err("UEBERSPRINGEN: unter Windows prueft tray.dhtest das Symbol".into())
    }
}

/// Mitteilungen ohne Tray-Symbol des Systems: macOS ueber `osascript`, Linux
/// ueber `notify-send`. Der Text geht als Argument bzw. ueber eine
/// AppleScript-Zeichenkette mit maskierten Anfuehrungszeichen -- nie ueber
/// eine Shell.
#[cfg(not(windows))]
fn mitteilen_extern(titel: &str, text: &str) -> Result<(), String> {
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

/// Ein Probebild fuer den Selbsttest: 32x32, oben links (255, 10, 20, 30).
#[allow(dead_code)]
fn probebild() -> Vec<u8> {
    let mut p = Vec::with_capacity(32 * 32 * 4);
    for i in 0..32 * 32 {
        if i == 0 { p.extend_from_slice(&[10, 20, 30, 255]); } else { p.extend_from_slice(&[200, 120, 40, 255]); }
    }
    p
}

// ------------------------------------------------------------------ Linux
#[cfg(all(unix, not(target_os = "macos"), not(target_os = "emscripten"), feature = "tray"))]
mod plattform {
    //! **Linux:** ein StatusNotifierItem (das Verfahren von KDE, das auch
    //! Xfce, LXQt, Cinnamon und GNOME mit der Erweiterung AppIndicator
    //! anzeigen) samt Menue ueber `com.canonical.dbusmenu`, beides ueber die
    //! D-Bus-Sitzung. Das Symbol meldet sich beim `StatusNotifierWatcher` an
    //! -- und erneut, sobald eine Leiste (neu) startet. Die Aufrufe der Leiste
    //! kommen auf zbus' eigenem Faden an und landen in denselben Ereignissen
    //! wie unter Windows. Einen Doppelklick kennt das Verfahren nicht.
    use super::Ereignisse;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use zbus::blocking::Connection;
    use zbus::object_server::SignalEmitter;
    use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

    pub(super) const ITEM: &str = "/StatusNotifierItem";
    pub(super) const MENUE: &str = "/MenuBar";
    pub(super) const WATCHER: &str = "org.kde.StatusNotifierWatcher";

    #[derive(Default)]
    struct Zustand {
        sichtbar: bool,
        tipp: String,
        menue: Vec<String>,
        /// Eigenes Bild als ARGB32 in Netzreihenfolge (Breite, Hoehe, Punkte).
        bild: Option<(i32, i32, Vec<u8>)>,
        revision: u32,
        offen: Ereignisse,
        jetzt: Ereignisse,
    }

    static ZUSTAND: Mutex<Option<Zustand>> = Mutex::new(None);
    static VERBINDUNG: Mutex<Option<Connection>> = Mutex::new(None);

    fn mit<R>(f: impl FnOnce(&mut Zustand) -> R) -> R {
        let mut g = ZUSTAND.lock().unwrap_or_else(|e| e.into_inner());
        f(g.get_or_insert_with(Zustand::default))
    }

    pub(super) fn dienstname() -> String { format!("org.kde.StatusNotifierItem-{}-1", std::process::id()) }

    fn ov(v: Value<'_>) -> OwnedValue { v.try_to_owned().unwrap_or_else(|_| OwnedValue::from(0i32)) }

    struct Item;

    #[zbus::interface(name = "org.kde.StatusNotifierItem")]
    impl Item {
        #[zbus(property)] fn category(&self) -> String { "ApplicationStatus".into() }
        #[zbus(property)] fn id(&self) -> String { "drachenhauch".into() }
        #[zbus(property)] fn title(&self) -> String { mit(|z| z.tipp.clone()) }
        #[zbus(property)] fn status(&self) -> String { "Active".into() }
        #[zbus(property)] fn window_id(&self) -> i32 { 0 }
        /// Ohne eigenes Bild das Programmsymbol des Themas.
        #[zbus(property)] fn icon_name(&self) -> String {
            mit(|z| if z.bild.is_some() { String::new() } else { "application-x-executable".into() })
        }
        #[zbus(property)] fn icon_pixmap(&self) -> Vec<(i32, i32, Vec<u8>)> { mit(|z| z.bild.clone().into_iter().collect()) }
        #[zbus(property)] fn overlay_icon_name(&self) -> String { String::new() }
        #[zbus(property)] fn overlay_icon_pixmap(&self) -> Vec<(i32, i32, Vec<u8>)> { Vec::new() }
        #[zbus(property)] fn attention_icon_name(&self) -> String { String::new() }
        #[zbus(property)] fn attention_icon_pixmap(&self) -> Vec<(i32, i32, Vec<u8>)> { Vec::new() }
        #[zbus(property)] fn attention_movie_name(&self) -> String { String::new() }
        #[zbus(property)] fn tool_tip(&self) -> (String, Vec<(i32, i32, Vec<u8>)>, String, String) {
            (String::new(), Vec::new(), mit(|z| z.tipp.clone()), String::new())
        }
        #[zbus(property)] fn item_is_menu(&self) -> bool { false }
        #[zbus(property)] fn menu(&self) -> OwnedObjectPath { OwnedObjectPath::try_from(MENUE).expect("fester Pfad") }

        fn activate(&self, _x: i32, _y: i32) { mit(|z| z.offen.klick = true) }
        fn secondary_activate(&self, _x: i32, _y: i32) {}
        fn context_menu(&self, _x: i32, _y: i32) {}
        fn scroll(&self, _delta: i32, _orientation: String) {}

        #[zbus(signal)] async fn new_title(e: &SignalEmitter<'_>) -> zbus::Result<()>;
        #[zbus(signal)] async fn new_icon(e: &SignalEmitter<'_>) -> zbus::Result<()>;
        #[zbus(signal)] async fn new_tool_tip(e: &SignalEmitter<'_>) -> zbus::Result<()>;
    }

    /// Die Eigenschaften eines Menueeintrags (Nummer = Platz + 1, 0 ist die Wurzel).
    fn eintrag(t: &str) -> HashMap<String, OwnedValue> {
        let mut m = HashMap::new();
        if t == "-" {
            m.insert("type".to_string(), ov(Value::from("separator")));
        } else {
            m.insert("label".to_string(), ov(Value::from(t)));
            m.insert("enabled".to_string(), ov(Value::from(true)));
            m.insert("visible".to_string(), ov(Value::from(true)));
        }
        m
    }

    fn kind(id: i32, props: HashMap<String, OwnedValue>) -> OwnedValue {
        let props: HashMap<String, Value<'_>> = props.into_iter().map(|(k, v)| (k, Value::from(v))).collect();
        ov(Value::from((id, props, Vec::<Value<'_>>::new())))
    }

    struct Menue;

    #[zbus::interface(name = "com.canonical.dbusmenu")]
    impl Menue {
        #[zbus(property)] fn version(&self) -> u32 { 3 }
        #[zbus(property)] fn text_direction(&self) -> String { "ltr".into() }
        #[zbus(property)] fn status(&self) -> String { "normal".into() }
        #[zbus(property)] fn icon_theme_path(&self) -> Vec<String> { Vec::new() }

        fn get_layout(&self, parent_id: i32, _recursion_depth: i32, _property_names: Vec<String>)
            -> (u32, (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>)) {
            let (rev, eintraege) = mit(|z| (z.revision, z.menue.clone()));
            if parent_id > 0 {
                let p = eintraege.get(parent_id as usize - 1).map(|t| eintrag(t)).unwrap_or_default();
                return (rev, (parent_id, p, Vec::new()));
            }
            let mut wurzel = HashMap::new();
            wurzel.insert("children-display".to_string(), ov(Value::from("submenu")));
            let kinder = eintraege.iter().enumerate().map(|(i, t)| kind(i as i32 + 1, eintrag(t))).collect();
            (rev, (0, wurzel, kinder))
        }

        fn get_group_properties(&self, ids: Vec<i32>, _property_names: Vec<String>) -> Vec<(i32, HashMap<String, OwnedValue>)> {
            let eintraege = mit(|z| z.menue.clone());
            ids.into_iter().filter_map(|id| {
                if id <= 0 { return None; }
                eintraege.get(id as usize - 1).map(|t| (id, eintrag(t)))
            }).collect()
        }

        fn get_property(&self, id: i32, name: String) -> zbus::fdo::Result<OwnedValue> {
            let eintraege = mit(|z| z.menue.clone());
            eintraege.get((id.max(1) - 1) as usize).filter(|_| id > 0)
                .and_then(|t| eintrag(t).remove(&name))
                .ok_or_else(|| zbus::fdo::Error::InvalidArgs(format!("kein Wert {} fuer {}", name, id)))
        }

        fn event(&self, id: i32, event_id: String, _data: OwnedValue, _timestamp: u32) {
            if event_id != "clicked" || id <= 0 { return; }
            mit(|z| if let Some(t) = z.menue.get(id as usize - 1).filter(|t| *t != "-") {
                z.offen.menue = Some(t.clone());
            });
        }

        fn event_group(&self, events: Vec<(i32, String, OwnedValue, u32)>) -> Vec<i32> {
            let n = mit(|z| z.menue.len()) as i32;
            let mut fehlt = Vec::new();
            for (id, ev, daten, zeit) in events {
                if id < 1 || id > n { fehlt.push(id); continue; }
                self.event(id, ev, daten, zeit);
            }
            fehlt
        }

        fn about_to_show(&self, _id: i32) -> bool { false }
        fn about_to_show_group(&self, _ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) { (Vec::new(), Vec::new()) }

        #[zbus(signal)] async fn layout_updated(e: &SignalEmitter<'_>, revision: u32, parent: i32) -> zbus::Result<()>;
    }

    fn verbindung() -> Option<Connection> { VERBINDUNG.lock().unwrap_or_else(|e| e.into_inner()).clone() }

    fn anmelden(conn: &Connection) {
        let _ = conn.call_method(Some(WATCHER), "/StatusNotifierWatcher", Some(WATCHER),
                                 "RegisterStatusNotifierItem", &(dienstname(),));
    }

    /// Kommt eine Leiste (neu), meldet sich das Symbol wieder an -- wie unter
    /// Windows nach einem Neustart des Explorers.
    fn waechter(conn: Connection) {
        let Ok(dbus) = zbus::blocking::fdo::DBusProxy::new(&conn) else { return };
        let Ok(signale) = dbus.receive_name_owner_changed() else { return };
        for s in signale {
            let Ok(a) = s.args() else { continue };
            if a.name().as_str() == WATCHER && a.new_owner().is_some() && mit(|z| z.sichtbar) {
                anmelden(&conn);
            }
        }
    }

    fn verbinden() -> Result<Connection, String> {
        if let Some(c) = verbindung() { return Ok(c); }
        let conn = zbus::blocking::connection::Builder::session()
            .and_then(|b| b.serve_at(ITEM, Item))
            .and_then(|b| b.serve_at(MENUE, Menue))
            .and_then(|b| b.build())
            .map_err(|e| format!("keine D-Bus-Sitzung erreichbar ({}) -- ohne sie zeigt Linux kein Symbol in der Leiste", e))?;
        let c2 = conn.clone();
        std::thread::spawn(move || waechter(c2));
        *VERBINDUNG.lock().unwrap_or_else(|e| e.into_inner()) = Some(conn.clone());
        Ok(conn)
    }

    fn melden_item(f: impl FnOnce(&SignalEmitter<'static>) -> zbus::Result<()>) {
        if let Some(c) = verbindung() {
            if let Ok(i) = c.object_server().interface::<_, Item>(ITEM) { let _ = f(i.signal_emitter()); }
        }
    }

    pub fn einhaengen(_handle: *mut std::ffi::c_void) {}

    pub fn zeigen(rgba: Option<(&[u8], i32, i32)>, tipp: Option<&str>) -> Result<(), String> {
        let argb = rgba.map(|(p, b, h)| (b, h, p.chunks_exact(4).flat_map(|q| [q[3], q[0], q[1], q[2]]).collect::<Vec<u8>>()));
        let war = mit(|z| z.sichtbar);
        let conn = verbinden()?;
        if !war {
            conn.request_name(dienstname().as_str())
                .map_err(|e| format!("der Name auf dem D-Bus ist nicht zu bekommen ({})", e))?;
        }
        mit(|z| {
            if let Some(a) = argb { z.bild = Some(a); }
            if let Some(t) = tipp { z.tipp = t.to_string(); }
            z.sichtbar = true;
        });
        if war {
            melden_item(|e| zbus::block_on(Item::new_icon(e)));
            melden_item(|e| zbus::block_on(Item::new_tool_tip(e)));
            melden_item(|e| zbus::block_on(Item::new_title(e)));
        } else {
            anmelden(&conn);
        }
        Ok(())
    }

    pub fn tipp(text: &str) -> Result<(), String> {
        let sichtbar = mit(|z| { z.tipp = text.to_string(); z.sichtbar });
        if sichtbar {
            melden_item(|e| zbus::block_on(Item::new_tool_tip(e)));
            melden_item(|e| zbus::block_on(Item::new_title(e)));
        }
        Ok(())
    }

    pub fn menue(eintraege: Vec<String>) {
        let (rev, sichtbar) = mit(|z| { z.menue = eintraege; z.revision += 1; (z.revision, z.sichtbar) });
        if !sichtbar { return; }
        if let Some(c) = verbindung() {
            if let Ok(i) = c.object_server().interface::<_, Menue>(MENUE) {
                let _ = zbus::block_on(Menue::layout_updated(i.signal_emitter(), rev, 0));
            }
        }
    }

    /// Den Namen abgeben: die Leiste sieht ihn verschwinden und nimmt das
    /// Symbol weg. Die Verbindung bleibt fuer ein spaeteres TRAY_SHOW.
    pub fn weg() {
        let war = mit(|z| { let w = z.sichtbar; z.sichtbar = false; w });
        if war { if let Some(c) = verbindung() { let _ = c.release_name(dienstname().as_str()); } }
    }

    pub fn sichtbar() -> bool { mit(|z| z.sichtbar) }

    pub fn mitteilen(titel: &str, text: &str, _standard_tipp: &str) -> Result<(), String> {
        super::mitteilen_extern(titel, text)
    }

    pub fn abholen() { mit(|z| z.jetzt = std::mem::take(&mut z.offen)); }
    pub fn bild() -> Ereignisse { mit(|z| z.jetzt.clone()) }

    // --- Selbsttest (dhrt pruef tray) ------------------------------------

    struct Watcher { liste: std::sync::Arc<Mutex<Vec<String>>> }

    #[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
    impl Watcher {
        fn register_status_notifier_item(&self, service: String) {
            self.liste.lock().unwrap_or_else(|e| e.into_inner()).push(service);
        }
        #[zbus(property)] fn is_status_notifier_host_registered(&self) -> bool { true }
        #[zbus(property)] fn protocol_version(&self) -> i32 { 0 }
    }

    #[zbus::proxy(interface = "org.kde.StatusNotifierItem", default_path = "/StatusNotifierItem")]
    trait ItemProbe {
        #[zbus(property)] fn title(&self) -> zbus::Result<String>;
        #[zbus(property)] fn tool_tip(&self) -> zbus::Result<(String, Vec<(i32, i32, Vec<u8>)>, String, String)>;
        #[zbus(property)] fn icon_pixmap(&self) -> zbus::Result<Vec<(i32, i32, Vec<u8>)>>;
        #[zbus(property)] fn menu(&self) -> zbus::Result<OwnedObjectPath>;
        fn activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    }

    #[zbus::proxy(interface = "com.canonical.dbusmenu", default_path = "/MenuBar")]
    trait MenueProbe {
        fn get_layout(&self, parent_id: i32, recursion_depth: i32, property_names: Vec<String>)
            -> zbus::Result<(u32, (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>))>;
        fn event(&self, id: i32, event_id: &str, data: &Value<'_>, timestamp: u32) -> zbus::Result<()>;
    }

    /// Kurz warten, bis ein Aufruf von aussen angekommen ist.
    fn warten_auf(f: impl Fn() -> bool) -> bool {
        for _ in 0..100 {
            if f() { return true; }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        false
    }

    /// Gegen einen eigenen `dbus-daemon`: ein Waechter wie der einer Leiste,
    /// ein fremder Teilnehmer liest Eigenschaften und Menue und loest Klick
    /// und Menuewahl aus.
    pub fn selbsttest() -> Result<Vec<String>, String> {
        use std::io::{BufRead, BufReader};
        let mut daemon = match std::process::Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address=1"])
            .stdout(std::process::Stdio::piped()).spawn() {
            Ok(d) => d,
            Err(_) => return Err("UEBERSPRINGEN: dbus-daemon fehlt".into()),
        };
        let mut adresse = String::new();
        let lesen = daemon.stdout.take().map(|s| BufReader::new(s).read_line(&mut adresse));
        let erg = (|| -> Result<Vec<String>, String> {
            if !matches!(lesen, Some(Ok(n)) if n > 0) { return Err("dbus-daemon nennt keine Adresse".into()); }
            std::env::set_var("DBUS_SESSION_BUS_ADDRESS", adresse.trim());
            let f = |e: zbus::Error| e.to_string();
            let liste = std::sync::Arc::new(Mutex::new(Vec::new()));
            let _watcher = zbus::blocking::connection::Builder::session().map_err(f)?
                .name(WATCHER).map_err(f)?
                .serve_at("/StatusNotifierWatcher", Watcher { liste: liste.clone() }).map_err(f)?
                .build().map_err(f)?;
            let mut aus = Vec::new();
            zeigen(Some((&super::probebild(), 32, 32)), Some("Probe"))?;
            menue(vec!["Eins".into(), "-".into(), "Zwei".into()]);
            let name = dienstname();
            let angemeldet = warten_auf(|| liste.lock().map(|l| l.contains(&name)).unwrap_or(false));
            aus.push(format!("angemeldet: {}", if angemeldet { "ja" } else { "nein" }));
            let client = Connection::session().map_err(f)?;
            let item = ItemProbeProxyBlocking::builder(&client).destination(name.as_str()).map_err(f)?
                .cache_properties(zbus::proxy::CacheProperties::No).build().map_err(f)?;
            aus.push(format!("titel: {}", item.title().map_err(f)?));
            aus.push(format!("hinweis: {}", item.tool_tip().map_err(f)?.2));
            let bilder = item.icon_pixmap().map_err(f)?;
            if let Some((b, h, p)) = bilder.first() {
                aus.push(format!("bild: {}x{}, erster Punkt ARGB {} {} {} {}", b, h, p[0], p[1], p[2], p[3]));
            }
            let menue_pfad = item.menu().map_err(f)?;
            let m = MenueProbeProxyBlocking::builder(&client).destination(name.as_str()).map_err(f)?
                .path(menue_pfad.as_str()).map_err(f)?.build().map_err(f)?;
            let (_, (_, _, kinder)) = m.get_layout(0, -1, Vec::new()).map_err(f)?;
            let mut texte = Vec::new();
            for k in kinder {
                let s = zbus::zvariant::Structure::try_from(k).map_err(|e| e.to_string())?;
                let felder = s.into_fields();
                let props = HashMap::<String, OwnedValue>::try_from(felder.get(1).cloned().ok_or("leeres Kind")?)
                    .map_err(|e| e.to_string())?;
                let text = if let Some(l) = props.get("label") { String::try_from(l.clone()).map_err(|e| e.to_string())? }
                    else { "-".into() };
                texte.push(text);
            }
            aus.push(format!("menue: {}", texte.join(" | ")));
            item.activate(0, 0).map_err(f)?;
            let klick = warten_auf(|| mit(|z| z.offen.klick));
            abholen();
            aus.push(format!("klick: {}", if klick && bild().klick { "ja" } else { "nein" }));
            m.event(3, "clicked", &Value::from(0i32), 0).map_err(f)?;
            warten_auf(|| mit(|z| z.offen.menue.is_some()));
            abholen();
            aus.push(format!("gewaehlt: {}", bild().menue.unwrap_or_default()));
            tipp("Neu")?;
            aus.push(format!("hinweis danach: {}", item.tool_tip().map_err(f)?.2));
            weg();
            let dbus = zbus::blocking::fdo::DBusProxy::new(&client).map_err(f)?;
            let n = zbus::names::BusName::try_from(name.as_str()).map_err(|e| e.to_string())?;
            let weg_ok = warten_auf(|| matches!(dbus.name_has_owner(n.clone()), Ok(false)));
            aus.push(format!("nach dem Ausblenden: {}", if weg_ok && !sichtbar() { "weg" } else { "noch da" }));
            Ok(aus)
        })();
        let _ = daemon.kill();
        let _ = daemon.wait();
        erg
    }
}

// ------------------------------------------------------------------ macOS
#[cfg(all(target_os = "macos", feature = "tray"))]
mod plattform {
    //! **macOS:** ein `NSStatusItem` in der Menueleiste, nur ueber die
    //! Objective-C-Laufzeit (wie `finder.rs`). Ein eigenes Zielobjekt
    //! (`DHTrayZiel`) nimmt Klicks und Menuewahl an; sie kommen waehrend
    //! GLFWs Ereignisschleife im Hauptfaden an. Links = Klick (zweimal schnell
    //! = Doppelklick), rechts oder Ctrl+Klick = das Menue. AppKit darf nur aus
    //! dem Hauptfaden benutzt werden -- die VM laeuft dort.
    use super::Ereignisse;
    use std::ffi::{c_char, c_void, CStr, CString};
    use std::sync::Mutex;

    type Id = *mut c_void;
    type Sel = *mut c_void;

    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn objc_allocateClassPair(oben: Id, name: *const c_char, extra: usize) -> Id;
        fn objc_registerClassPair(cls: Id);
        fn class_addMethod(cls: Id, sel: Sel, imp: *const c_void, types: *const c_char) -> bool;
        fn objc_autoreleasePoolPush() -> *mut c_void;
        fn objc_autoreleasePoolPop(pool: *mut c_void);
        fn objc_msgSend();
    }
    #[link(name = "AppKit", kind = "framework")]
    extern "C" {}
    #[link(name = "Foundation", kind = "framework")]
    extern "C" {}

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NsSize { b: f64, h: f64 }

    fn sel(n: &CStr) -> Sel { unsafe { sel_registerName(n.as_ptr()) } }
    fn klasse(n: &CStr) -> Id { unsafe { objc_getClass(n.as_ptr()) } }

    // objc_msgSend muss mit der genauen Signatur gerufen werden (arm64).
    macro_rules! senden {
        ($ziel:expr, $sel:expr $(, $a:expr => $t:ty)* ; -> $r:ty) => {{
            let f: extern "C" fn(Id, Sel $(, $t)*) -> $r = unsafe { std::mem::transmute(objc_msgSend as *const ()) };
            f($ziel, sel($sel) $(, $a)*)
        }};
    }

    fn ns_text(s: &str) -> Id {
        let c = CString::new(s.replace('\0', " ")).unwrap_or_default();
        senden!(klasse(c"NSString"), c"stringWithUTF8String:", c.as_ptr() => *const c_char; -> Id)
    }
    fn rust_text(s: Id) -> String {
        if s.is_null() { return String::new(); }
        let p = senden!(s, c"UTF8String"; -> *const c_char);
        if p.is_null() { String::new() } else { unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned() }
    }

    /// Ein Autorelease-Pool um jeden Aufruf: ein Konsolenprogramm hat keinen.
    fn im_pool<R>(f: impl FnOnce() -> R) -> R {
        let p = unsafe { objc_autoreleasePoolPush() };
        let r = f();
        unsafe { objc_autoreleasePoolPop(p) };
        r
    }

    #[derive(Default)]
    struct Zustand {
        /// Das NSStatusItem (gehalten), 0 = keins.
        item: usize,
        /// Das Zielobjekt fuer Klick und Menue (lebt bis zum Ende).
        ziel: usize,
        tipp: String,
        menue: Vec<String>,
        offen: Ereignisse,
        jetzt: Ereignisse,
    }

    static ZUSTAND: Mutex<Option<Zustand>> = Mutex::new(None);

    fn mit<R>(f: impl FnOnce(&mut Zustand) -> R) -> R {
        let mut g = ZUSTAND.lock().unwrap_or_else(|e| e.into_inner());
        f(g.get_or_insert_with(Zustand::default))
    }

    fn app() -> Id { senden!(klasse(c"NSApplication"), c"sharedApplication"; -> Id) }

    /// - (void)klick:(id)sender
    extern "C" fn bei_klick(_selbst: Id, _cmd: Sel, _sender: Id) {
        im_pool(|| {
            let ev = senden!(app(), c"currentEvent"; -> Id);
            let (rechts, doppelt) = if ev.is_null() { (false, false) } else {
                let art = senden!(ev, c"type"; -> usize);
                let tasten = senden!(ev, c"modifierFlags"; -> usize);
                let zahl = if art == 2 { senden!(ev, c"clickCount"; -> isize) } else { 1 };
                // NSEventTypeRightMouseUp = 4, Ctrl = 1 << 18
                (art == 4 || tasten & (1 << 18) != 0, zahl >= 2)
            };
            let (item, hat_menue) = mit(|z| (z.item, !z.menue.is_empty()));
            if rechts && hat_menue && item != 0 {
                let m = menue_bauen();
                senden!(item as Id, c"popUpStatusItemMenu:", m => Id; -> ());
                return;
            }
            mit(|z| { z.offen.klick = true; if doppelt { z.offen.doppel = true; } });
        });
    }

    /// - (void)menue:(NSMenuItem*)sender
    extern "C" fn bei_menue(_selbst: Id, _cmd: Sel, sender: Id) {
        if sender.is_null() { return; }
        let platz = senden!(sender, c"tag"; -> isize);
        mit(|z| if let Some(t) = z.menue.get(platz.max(0) as usize).filter(|t| *t != "-") {
            z.offen.menue = Some(t.clone());
        });
    }

    fn ziel() -> Id {
        let z = mit(|z| z.ziel);
        if z != 0 { return z as Id; }
        let mut cls = klasse(c"DHTrayZiel");
        if cls.is_null() {
            unsafe {
                cls = objc_allocateClassPair(klasse(c"NSObject"), c"DHTrayZiel".as_ptr(), 0);
                class_addMethod(cls, sel(c"klick:"), bei_klick as *const c_void, c"v@:@".as_ptr());
                class_addMethod(cls, sel(c"menue:"), bei_menue as *const c_void, c"v@:@".as_ptr());
                objc_registerClassPair(cls);
            }
        }
        let obj = senden!(senden!(cls, c"alloc"; -> Id), c"init"; -> Id);
        mit(|z| z.ziel = obj as usize);
        obj
    }

    /// Das Menue aus den Eintraegen; die Nummer des Eintrags steht im Tag.
    fn menue_bauen() -> Id {
        let eintraege = mit(|z| z.menue.clone());
        let m = senden!(senden!(klasse(c"NSMenu"), c"alloc"; -> Id), c"init"; -> Id);
        senden!(m, c"autorelease"; -> Id);
        let ziel = ziel();
        for (i, t) in eintraege.iter().enumerate() {
            if t == "-" {
                let s = senden!(klasse(c"NSMenuItem"), c"separatorItem"; -> Id);
                senden!(m, c"addItem:", s => Id; -> ());
                continue;
            }
            let e = senden!(klasse(c"NSMenuItem"), c"alloc"; -> Id);
            let e = senden!(e, c"initWithTitle:action:keyEquivalent:", ns_text(t) => Id, sel(c"menue:") => Sel, ns_text("") => Id; -> Id);
            senden!(e, c"setTarget:", ziel => Id; -> ());
            senden!(e, c"setTag:", i as isize => isize; -> ());
            senden!(m, c"addItem:", e => Id; -> ());
            senden!(e, c"release"; -> ());
        }
        m
    }

    /// Ein NSImage aus RGBA-Punkten (oben links zuerst), fuer die Menueleiste
    /// auf 18x18 Punkte gestellt. NSBitmapImageRep erwartet vormultiplizierte
    /// Farben.
    fn bild_aus(rgba: &[u8], b: i32, h: i32) -> Id {
        let rep = senden!(klasse(c"NSBitmapImageRep"), c"alloc"; -> Id);
        let rep = senden!(rep, c"initWithBitmapDataPlanes:pixelsWide:pixelsHigh:bitsPerSample:samplesPerPixel:hasAlpha:isPlanar:colorSpaceName:bytesPerRow:bitsPerPixel:",
            std::ptr::null_mut::<c_void>() => *mut c_void, b as isize => isize, h as isize => isize, 8isize => isize, 4isize => isize,
            true => bool, false => bool, ns_text("NSDeviceRGBColorSpace") => Id, (b * 4) as isize => isize, 32isize => isize; -> Id);
        if rep.is_null() { return std::ptr::null_mut(); }
        let ziel = senden!(rep, c"bitmapData"; -> *mut u8);
        if !ziel.is_null() {
            let z = unsafe { std::slice::from_raw_parts_mut(ziel, (b * h * 4) as usize) };
            for (d, q) in z.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
                let a = q[3] as u32;
                d[0] = (q[0] as u32 * a / 255) as u8; d[1] = (q[1] as u32 * a / 255) as u8;
                d[2] = (q[2] as u32 * a / 255) as u8; d[3] = q[3];
            }
        }
        let img = senden!(klasse(c"NSImage"), c"alloc"; -> Id);
        let img = senden!(img, c"initWithSize:", NsSize { b: b as f64, h: h as f64 } => NsSize; -> Id);
        senden!(img, c"addRepresentation:", rep => Id; -> ());
        senden!(rep, c"release"; -> ());
        senden!(img, c"setSize:", NsSize { b: 18.0, h: 18.0 } => NsSize; -> ());
        senden!(img, c"autorelease"; -> Id)
    }

    /// Ohne eigenes Bild das Symbol des Programms.
    fn standardbild() -> Id {
        let img = senden!(klasse(c"NSImage"), c"imageNamed:", ns_text("NSApplicationIcon") => Id; -> Id);
        if img.is_null() { return img; }
        let kopie = senden!(img, c"copy"; -> Id);
        senden!(kopie, c"setSize:", NsSize { b: 18.0, h: 18.0 } => NsSize; -> ());
        senden!(kopie, c"autorelease"; -> Id)
    }

    fn knopf() -> Id {
        let item = mit(|z| z.item);
        if item == 0 { std::ptr::null_mut() } else { senden!(item as Id, c"button"; -> Id) }
    }

    pub fn einhaengen(_handle: *mut std::ffi::c_void) {}

    pub fn zeigen(rgba: Option<(&[u8], i32, i32)>, tipp: Option<&str>) -> Result<(), String> {
        im_pool(|| {
            if app().is_null() { return Err("NSApplication fehlt".into()); }
            let mut item = mit(|z| z.item) as Id;
            let neu = item.is_null();
            if neu {
                let leiste = senden!(klasse(c"NSStatusBar"), c"systemStatusBar"; -> Id);
                // NSVariableStatusItemLength = -1
                item = senden!(leiste, c"statusItemWithLength:", -1.0f64 => f64; -> Id);
                if item.is_null() { return Err("die Menueleiste hat kein Symbol angenommen".into()); }
                senden!(item, c"retain"; -> Id);
                mit(|z| z.item = item as usize);
                let k = knopf();
                senden!(k, c"setTarget:", ziel() => Id; -> ());
                senden!(k, c"setAction:", sel(c"klick:") => Sel; -> ());
                // linke und rechte Taste: NSEventMaskLeftMouseUp | RightMouseUp
                senden!(k, c"sendActionOn:", (1usize << 2) | (1usize << 4) => usize; -> isize);
            }
            let k = knopf();
            match rgba {
                Some((p, b, h)) => { let img = bild_aus(p, b, h); senden!(k, c"setImage:", img => Id; -> ()); }
                None if neu => { let img = standardbild(); senden!(k, c"setImage:", img => Id; -> ()); }
                None => {}
            }
            if let Some(t) = tipp { mit(|z| z.tipp = t.to_string()); }
            let t = mit(|z| z.tipp.clone());
            senden!(k, c"setToolTip:", ns_text(&t) => Id; -> ());
            Ok(())
        })
    }

    pub fn tipp(text: &str) -> Result<(), String> {
        mit(|z| z.tipp = text.to_string());
        if sichtbar() { im_pool(|| senden!(knopf(), c"setToolTip:", ns_text(text) => Id; -> ())); }
        Ok(())
    }

    pub fn menue(eintraege: Vec<String>) { mit(|z| z.menue = eintraege); }

    pub fn weg() {
        let item = mit(|z| std::mem::take(&mut z.item));
        if item == 0 { return; }
        im_pool(|| {
            let leiste = senden!(klasse(c"NSStatusBar"), c"systemStatusBar"; -> Id);
            senden!(leiste, c"removeStatusItem:", item as Id => Id; -> ());
            senden!(item as Id, c"release"; -> ());
        });
    }

    pub fn sichtbar() -> bool { mit(|z| z.item != 0) }

    pub fn mitteilen(titel: &str, text: &str, _standard_tipp: &str) -> Result<(), String> {
        super::mitteilen_extern(titel, text)
    }

    pub fn abholen() { mit(|z| z.jetzt = std::mem::take(&mut z.offen)); }
    pub fn bild() -> Ereignisse { mit(|z| z.jetzt.clone()) }

    /// Mit echten AppKit-Objekten: Symbol anlegen, zuruecklesen, Klick ueber
    /// `performClick:` und Menuewahl ueber `performActionForItemAtIndex:`.
    pub fn selbsttest() -> Result<Vec<String>, String> {
        let mut aus = Vec::new();
        zeigen(Some((&super::probebild(), 32, 32)), Some("Probe"))?;
        menue(vec!["Eins".into(), "-".into(), "Zwei".into()]);
        aus.push(format!("angemeldet: {}", if sichtbar() { "ja" } else { "nein" }));
        im_pool(|| -> Result<(), String> {
            let k = knopf();
            aus.push(format!("hinweis: {}", rust_text(senden!(k, c"toolTip"; -> Id))));
            let img = senden!(k, c"image"; -> Id);
            if !img.is_null() {
                let g = senden!(img, c"size"; -> NsSize);
                aus.push(format!("bild: {}x{}", g.b, g.h));
            }
            let m = menue_bauen();
            let n = senden!(m, c"numberOfItems"; -> isize);
            let mut texte = Vec::new();
            for i in 0..n {
                let e = senden!(m, c"itemAtIndex:", i => isize; -> Id);
                texte.push(if senden!(e, c"isSeparatorItem"; -> bool) { "-".to_string() } else { rust_text(senden!(e, c"title"; -> Id)) });
            }
            aus.push(format!("menue: {}", texte.join(" | ")));
            senden!(k, c"performClick:", std::ptr::null_mut::<c_void>() => Id; -> ());
            abholen();
            aus.push(format!("klick: {}", if bild().klick { "ja" } else { "nein" }));
            senden!(m, c"performActionForItemAtIndex:", 2isize => isize; -> ());
            abholen();
            aus.push(format!("gewaehlt: {}", bild().menue.unwrap_or_default()));
            Ok(())
        })?;
        tipp("Neu")?;
        aus.push(format!("hinweis danach: {}", im_pool(|| rust_text(senden!(knopf(), c"toolTip"; -> Id)))));
        weg();
        aus.push(format!("nach dem Ausblenden: {}", if sichtbar() { "noch da" } else { "weg" }));
        Ok(aus)
    }
}

// ------------------------------------------------- ohne Tray dieses Systems
#[cfg(not(any(windows, all(target_os = "macos", feature = "tray"),
              all(unix, not(target_os = "macos"), not(target_os = "emscripten"), feature = "tray"))))]
mod plattform {
    use super::Ereignisse;

    pub fn einhaengen(_handle: *mut std::ffi::c_void) {}
    pub fn zeigen(_rgba: Option<(&[u8], i32, i32)>, _tipp: Option<&str>) -> Result<(), String> {
        Err("ein Tray-Symbol gibt es in diesem Bau nicht (Feature tray)".into())
    }
    pub fn tipp(_text: &str) -> Result<(), String> { zeigen(None, None) }
    pub fn menue(_eintraege: Vec<String>) {}
    pub fn weg() {}
    pub fn sichtbar() -> bool { false }
    pub fn abholen() {}
    pub fn bild() -> Ereignisse { Ereignisse::default() }
    #[cfg(not(target_os = "emscripten"))]
    pub fn mitteilen(titel: &str, text: &str, _standard_tipp: &str) -> Result<(), String> {
        super::mitteilen_extern(titel, text)
    }
    #[cfg(target_os = "emscripten")]
    pub fn mitteilen(_titel: &str, _text: &str, _standard_tipp: &str) -> Result<(), String> {
        Err("Mitteilungen gibt es im Browser nicht".into())
    }
    pub fn selbsttest() -> Result<Vec<String>, String> {
        Err("UEBERSPRINGEN: dieser Bau hat kein Tray-Symbol (Feature tray)".into())
    }
}

pub use plattform::{abholen, bild, einhaengen, menue, mitteilen, sichtbar, tipp, weg, zeigen};

/// `dhrt pruef tray`: der Selbsttest des Systems. Eine Zeile je Befund; ein
/// Grund zum Ueberspringen beginnt mit `UEBERSPRINGEN:` (Rueckgabe 0).
pub fn selbsttest_main() -> std::process::ExitCode {
    match plattform::selbsttest() {
        Ok(zeilen) => { for z in zeilen { println!("{}", z); } std::process::ExitCode::SUCCESS }
        Err(e) if e.starts_with("UEBERSPRINGEN:") => { println!("{}", e); std::process::ExitCode::SUCCESS }
        Err(e) => { eprintln!("dhrt pruef tray: {}", e); std::process::ExitCode::from(1) }
    }
}
