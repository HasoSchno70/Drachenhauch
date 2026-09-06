//! Barrierefreiheit: die Bruecke zwischen dem gui-Modul und der
//! Barrierefreiheits-Schnittstelle des Systems (docs/entwurf-barrierefreiheit.md,
//! Weg C).
//!
//! Gemessen vor dem Bau: ein laufendes dhrt-Fenster hatte im UIA-Baum von
//! Windows NULL Nachkommen -- fuer einen Bildschirmleser, die Lupe und die
//! Sprachsteuerung war jedes Drachenhauch-Fenster ein Titel und sonst nichts.
//! raylib malt Pixel in ein GLFW-Fenster, und GLFW meldet dem System keine
//! Bedienelemente.
//!
//! Hier haengt sich AccessKit an dieses Fenster. Das gui-Modul baut je Bild
//! einen Baum aus Knoten (Rolle, Beschriftung, Wert, Rechteck, Zustaende,
//! erlaubte Aktionen), AccessKit uebersetzt ihn fuer die Schnittstelle des
//! Systems; Aktionen eines Hilfsprogramms (Fokus setzen, Klick, Wert setzen)
//! kommen ueber eine Warteschlange zurueck, die GUI_UPDATE leert. Der Baum
//! wird NUR gebaut, wenn ein Hilfsprogramm danach gefragt hat -- ein Spiel
//! ohne Bildschirmleser zahlt nichts ausser dem Einhaengen.
//!
//! Drei Systeme, EIN Vertrag (`A11y`): `neu`, `aktiv`, `aktionen`, `senden`,
//! `fenster_lage`, `fenster_fokus`.
//!
//! * **Windows** (UI Automation, `accesskit_windows`): Subclassing der
//!   Fensterprozedur. Der Adapter verlangt ein Fenster, das NOCH NICHT
//!   sichtbar war (sonst `panic!`); raylib zeigt es beim Anlegen sofort,
//!   deshalb legt `graphics.rs` es versteckt an, haengt den Adapter ein und
//!   zeigt es dann. Gemessen mit einem fremden UIA-Leser
//!   (`tests/test_gui_barrierefreiheit.py`).
//! * **macOS** (NSAccessibility, `accesskit_macos`): dynamisches
//!   Objective-C-Subclassing der Inhalts-View von GLFWs `NSWindow`; vorher
//!   bekommt die Fensterklasse `GLFWWindow` eine Fokus-Weiterleitung, weil
//!   GLFW den Tastaturfokus aufs Fenster legt statt auf die View. Die
//!   Fensterfokus-Aenderungen meldet FLIP nach.
//! * **Linux** (AT-SPI ueber D-Bus, `accesskit_unix` mit `async-io`): der
//!   Adapter laeuft auf einem eigenen Faden; Lage und Fokus des Fensters muss
//!   man ihm selbst sagen (FLIP tut es bei Aenderung; unter Wayland kennt ein
//!   Programm seine Lage nicht, dann fehlt sie). Ein Hilfsprogramm kann die
//!   Barrierefreiheit auch wieder abschalten (`DeactivationHandler`).
//!
//! In allen dreien kommt die Aktivierungs-Anfrage von aussen (Windows und
//! macOS in der Fensterprozedur, Linux auf dem D-Bus-Faden), wo das
//! gui-Modul nicht greifbar ist. Der Handler merkt sich nur, DASS gefragt
//! wurde, und antwortet mit `None`; laut Vertrag muss dann das naechste
//! `update_if_active` einen VOLLSTAENDIGEN Baum liefern -- GUI_UPDATE tut
//! das (und FLIP, falls ein Programm kein gui benutzt: dann ein Baum nur mit
//! dem Fenster).
//!
//! macOS und Linux sind hier nicht zu pruefen (kein Geraet); ihr Code wird
//! mit `cargo check --features a11y --target ...` uebersetzt und in der CI
//! auf den echten Systemen gebaut -- laufen gesehen hat ihn niemand. Das
//! steht so in der Doku.

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, TreeUpdate};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Hoechstzahl wartender Aktionen. Ein Hilfsprogramm schickt sie
/// stossweise; mehr als das haette kein Programm in einem Bild verarbeitet.
const MAX_AKTIONEN: usize = 256;

/// Antwortet auf die Aktivierungs-Anfrage: nur merken, dass gefragt wurde.
struct Aktivierung { aktiv: Arc<AtomicBool> }
impl ActivationHandler for Aktivierung {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.aktiv.store(true, Ordering::SeqCst);
        None
    }
}

/// Nimmt Aktionen entgegen (auf einem fremden Faden moeglich) und stellt sie
/// in die Warteschlange, die GUI_UPDATE leert.
struct Aktionen { queue: Arc<Mutex<Vec<ActionRequest>>> }
impl ActionHandler for Aktionen {
    fn do_action(&mut self, request: ActionRequest) {
        if let Ok(mut q) = self.queue.lock() {
            if q.len() < MAX_AKTIONEN { q.push(request); }
        }
    }
}

/// Das Hilfsprogramm ist weg (Linux): nichts mehr bauen, bis wieder gefragt wird.
#[allow(dead_code)]
struct Deaktivierung { aktiv: Arc<AtomicBool> }
impl accesskit::DeactivationHandler for Deaktivierung {
    fn deactivate_accessibility(&mut self) { self.aktiv.store(false, Ordering::SeqCst); }
}

#[allow(dead_code)]
fn aktionen_abholen(queue: &Arc<Mutex<Vec<ActionRequest>>>) -> Vec<ActionRequest> {
    match queue.lock() { Ok(mut q) => std::mem::take(&mut *q), Err(_) => Vec::new() }
}

// ---------------------------------------------------------------- Windows
#[cfg(windows)]
mod plattform {
    use super::*;
    use accesskit_windows::{SubclassingAdapter, HWND};

    pub struct A11y {
        adapter: SubclassingAdapter,
        aktiv: Arc<AtomicBool>,
        queue: Arc<Mutex<Vec<ActionRequest>>>,
    }

    impl A11y {
        /// Haengt den Adapter an das (noch unsichtbare) Fenster. `None`, wenn
        /// es kein Handle gibt oder die Bibliothek das Fenster ablehnt -- dann
        /// laeuft das Programm wie bisher, nur ohne Baum.
        pub fn neu(handle: *mut std::ffi::c_void) -> Option<A11y> {
            if handle.is_null() { return None; }
            let aktiv = Arc::new(AtomicBool::new(false));
            let queue = Arc::new(Mutex::new(Vec::new()));
            let a = Aktivierung { aktiv: aktiv.clone() };
            let h = Aktionen { queue: queue.clone() };
            let adapter = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                SubclassingAdapter::new(HWND(handle), a, h)
            })).ok()?;
            Some(A11y { adapter, aktiv, queue })
        }
        pub fn aktiv(&self) -> bool { self.aktiv.load(Ordering::SeqCst) }
        pub fn aktionen(&mut self) -> Vec<ActionRequest> { aktionen_abholen(&self.queue) }
        pub fn senden(&mut self, baum: impl FnOnce() -> TreeUpdate) {
            if let Some(ereignisse) = self.adapter.update_if_active(baum) { ereignisse.raise(); }
        }
        /// UIA fragt das Fenster selbst nach seiner Lage -- nichts zu tun.
        pub fn fenster_lage(&mut self, _x: i32, _y: i32, _w: i32, _h: i32) {}
        /// Den Fensterfokus verfolgt der Subclass ueber WM_SETFOCUS/KILLFOCUS.
        pub fn fenster_fokus(&mut self, _fokus: bool) {}
    }
}

// ---------------------------------------------------------------- macOS
#[cfg(target_os = "macos")]
mod plattform {
    use super::*;
    use accesskit_macos::{add_focus_forwarder_to_window_class, SubclassingAdapter};

    pub struct A11y {
        adapter: SubclassingAdapter,
        aktiv: Arc<AtomicBool>,
        queue: Arc<Mutex<Vec<ActionRequest>>>,
        fokus: Option<bool>,
    }

    impl A11y {
        /// `handle` ist GLFWs `NSWindow` (raylib `GetWindowHandle`). Die
        /// Fokus-Weiterleitung an der Fensterklasse ist einmalig und nicht
        /// umkehrbar -- die Bibliothek ist statisch eingebunden, das ist die
        /// Bedingung dafuer.
        pub fn neu(handle: *mut std::ffi::c_void) -> Option<A11y> {
            if handle.is_null() { return None; }
            let aktiv = Arc::new(AtomicBool::new(false));
            let queue = Arc::new(Mutex::new(Vec::new()));
            let a = Aktivierung { aktiv: aktiv.clone() };
            let h = Aktionen { queue: queue.clone() };
            let adapter = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                add_focus_forwarder_to_window_class("GLFWWindow");
                SubclassingAdapter::for_window(handle, a, h)
            })).ok()?;
            Some(A11y { adapter, aktiv, queue, fokus: None })
        }
        pub fn aktiv(&self) -> bool { self.aktiv.load(Ordering::SeqCst) }
        pub fn aktionen(&mut self) -> Vec<ActionRequest> { aktionen_abholen(&self.queue) }
        pub fn senden(&mut self, baum: impl FnOnce() -> TreeUpdate) {
            if let Some(ereignisse) = self.adapter.update_if_active(baum) { ereignisse.raise(); }
        }
        /// Die View kennt ihre Lage selbst.
        pub fn fenster_lage(&mut self, _x: i32, _y: i32, _w: i32, _h: i32) {}
        /// Fokus der View: bei Aenderung melden (VoiceOver liest sonst ein
        /// Fenster im Hintergrund).
        pub fn fenster_fokus(&mut self, fokus: bool) {
            if self.fokus == Some(fokus) { return; }
            self.fokus = Some(fokus);
            if let Some(ereignisse) = self.adapter.update_view_focus_state(fokus) { ereignisse.raise(); }
        }
    }
}

// ---------------------------------------------------------------- Linux (X11/Wayland)
#[cfg(all(unix, not(target_os = "macos"), not(target_os = "emscripten")))]
mod plattform {
    use super::*;
    use accesskit_unix::Adapter;

    pub struct A11y {
        adapter: Adapter,
        aktiv: Arc<AtomicBool>,
        queue: Arc<Mutex<Vec<ActionRequest>>>,
        lage: Option<(i32, i32, i32, i32)>,
        fokus: Option<bool>,
    }

    impl A11y {
        /// AT-SPI braucht kein Fensterhandle: der Adapter meldet sich am
        /// Sitzungs-Bus an (eigener Faden, `async-io`). Ohne Bus (kein
        /// Desktop, CI-Laeufer) bleibt er still inaktiv.
        pub fn neu(_handle: *mut std::ffi::c_void) -> Option<A11y> {
            let aktiv = Arc::new(AtomicBool::new(false));
            let queue = Arc::new(Mutex::new(Vec::new()));
            let a = Aktivierung { aktiv: aktiv.clone() };
            let h = Aktionen { queue: queue.clone() };
            let d = Deaktivierung { aktiv: aktiv.clone() };
            let adapter = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Adapter::new(a, h, d))).ok()?;
            Some(A11y { adapter, aktiv, queue, lage: None, fokus: None })
        }
        pub fn aktiv(&self) -> bool { self.aktiv.load(Ordering::SeqCst) }
        pub fn aktionen(&mut self) -> Vec<ActionRequest> { aktionen_abholen(&self.queue) }
        pub fn senden(&mut self, baum: impl FnOnce() -> TreeUpdate) { self.adapter.update_if_active(baum); }
        /// Lage des Fensters auf dem Schirm (X11; unter Wayland liefert GLFW
        /// 0/0). GLFW kennt den Rahmen nicht, aussen = innen.
        pub fn fenster_lage(&mut self, x: i32, y: i32, w: i32, h: i32) {
            if self.lage == Some((x, y, w, h)) { return; }
            self.lage = Some((x, y, w, h));
            let r = accesskit::Rect::new(x as f64, y as f64, (x + w) as f64, (y + h) as f64);
            self.adapter.set_root_window_bounds(r, r);
        }
        pub fn fenster_fokus(&mut self, fokus: bool) {
            if self.fokus == Some(fokus) { return; }
            self.fokus = Some(fokus);
            self.adapter.update_window_focus_state(fokus);
        }
    }
}

// ---------------------------------------------------------------- Rest (Web)
#[cfg(not(any(windows, target_os = "macos", all(unix, not(target_os = "emscripten")))))]
mod plattform {
    use super::*;

    /// Kein Adapter (Web-Bau): nie aktiv.
    pub struct A11y;
    impl A11y {
        pub fn neu(_handle: *mut std::ffi::c_void) -> Option<A11y> { None }
        pub fn aktiv(&self) -> bool { false }
        pub fn aktionen(&mut self) -> Vec<ActionRequest> { Vec::new() }
        pub fn senden(&mut self, _baum: impl FnOnce() -> TreeUpdate) {}
        pub fn fenster_lage(&mut self, _x: i32, _y: i32, _w: i32, _h: i32) {}
        pub fn fenster_fokus(&mut self, _fokus: bool) {}
    }
}

pub use plattform::A11y;

/// Knoten-Nummern des Baums, an EINER Stelle -- Bauen (gui.rs) und Zerlegen
/// einer Aktion muessen dieselbe Rechnung benutzen.
///
/// Aufbau: Bits 40.. das Fenster (+1, damit 0 frei bleibt), Bit 38 die
/// Menueleiste, Bit 37 die Reiterleiste, Bits 16..36 das Widget (+1), Bits
/// 0..15 ein Teil davon (Listeneintrag, Zelle, Baumknoten; +1).
pub mod ids {
    pub const ROOT: u64 = 1;
    pub const ANSAGE: u64 = 2;
    const MENUE: u64 = 1 << 38;
    const REITER: u64 = 1 << 37;
    pub fn fenster(wi: usize) -> u64 { (wi as u64 + 1) << 40 }
    pub fn widget(wi: usize, i: usize) -> u64 { fenster(wi) | ((i as u64 + 1) << 16) }
    pub fn teil(wi: usize, i: usize, k: usize) -> u64 { widget(wi, i) | (k as u64 + 1) }
    pub fn leiste(wi: usize) -> u64 { fenster(wi) | MENUE }
    pub fn menue(wi: usize, mi: usize) -> u64 { leiste(wi) | ((mi as u64 + 1) << 16) }
    pub fn eintrag(wi: usize, mi: usize, ii: usize) -> u64 { menue(wi, mi) | (ii as u64 + 1) }
    pub fn reiterleiste(wi: usize) -> u64 { fenster(wi) | REITER }
    pub fn reiter(wi: usize, ti: usize) -> u64 { reiterleiste(wi) | (ti as u64 + 1) }

    /// Wohin zeigt eine Knoten-Nummer?
    #[derive(Debug, PartialEq)]
    pub enum Ziel {
        Root,
        Fenster(usize),
        Widget(usize, usize),
        Teil(usize, usize, usize),
        Menue(usize, usize),
        Eintrag(usize, usize, usize),
        Reiter(usize, usize),
        Sonst,
    }

    pub fn zerlegen(id: u64) -> Ziel {
        if id == ROOT { return Ziel::Root; }
        if id < (1 << 40) { return Ziel::Sonst; }
        let wi = (id >> 40) as usize - 1;
        let mitte = ((id >> 16) & ((1 << 21) - 1)) as usize;   // Widget/Menue (+1)
        let unten = (id & 0xFFFF) as usize;                    // Teil/Eintrag (+1)
        if id & MENUE != 0 {
            if mitte == 0 { return Ziel::Sonst; }
            return if unten == 0 { Ziel::Menue(wi, mitte - 1) } else { Ziel::Eintrag(wi, mitte - 1, unten - 1) };
        }
        if id & REITER != 0 {
            return if unten == 0 { Ziel::Sonst } else { Ziel::Reiter(wi, unten - 1) };
        }
        if mitte == 0 { return if unten == 0 { Ziel::Fenster(wi) } else { Ziel::Sonst }; }
        if unten == 0 { Ziel::Widget(wi, mitte - 1) } else { Ziel::Teil(wi, mitte - 1, unten - 1) }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nummern_sind_umkehrbar() {
            assert_eq!(zerlegen(ROOT), Ziel::Root);
            assert_eq!(zerlegen(fenster(0)), Ziel::Fenster(0));
            assert_eq!(zerlegen(fenster(7)), Ziel::Fenster(7));
            assert_eq!(zerlegen(widget(3, 0)), Ziel::Widget(3, 0));
            assert_eq!(zerlegen(widget(3, 500)), Ziel::Widget(3, 500));
            assert_eq!(zerlegen(teil(3, 500, 0)), Ziel::Teil(3, 500, 0));
            assert_eq!(zerlegen(teil(1, 2, 65000)), Ziel::Teil(1, 2, 65000));
            assert_eq!(zerlegen(menue(2, 1)), Ziel::Menue(2, 1));
            assert_eq!(zerlegen(eintrag(2, 1, 4)), Ziel::Eintrag(2, 1, 4));
            assert_eq!(zerlegen(reiter(0, 2)), Ziel::Reiter(0, 2));
            assert_eq!(zerlegen(leiste(0)), Ziel::Sonst);
            assert_eq!(zerlegen(reiterleiste(0)), Ziel::Sonst);
            assert_eq!(zerlegen(ANSAGE), Ziel::Sonst);
        }

        #[test]
        fn nummern_kollidieren_nicht() {
            let alle = [ROOT, ANSAGE, fenster(0), widget(0, 0), teil(0, 0, 0), leiste(0), menue(0, 0),
                        eintrag(0, 0, 0), reiterleiste(0), reiter(0, 0), fenster(1), widget(0, 1)];
            for (a, x) in alle.iter().enumerate() {
                for (b, y) in alle.iter().enumerate() {
                    assert!(a == b || x != y, "{a} und {b} teilen sich {x}");
                }
            }
        }
    }
}
