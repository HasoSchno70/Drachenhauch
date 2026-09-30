//! Klicks, die ganz zwischen zwei Bildern liegen.
//!
//! raylib liest die Maustasten als ZUSTAND: der GLFW-Rueckruf ueberschreibt
//! `currentButtonState`, einmal je Bild wird er nach `previous` kopiert
//! (rcore_desktop_glfw.c, MouseButtonCallback / PollInputEvents). Kommen
//! Druecken UND Loslassen vor dem naechsten Bild an, steht vorher wie jetzt
//! "los" -- der Klick hat nie stattgefunden, fuer MOUSE_HIT so wenig wie fuer
//! die gui. Gemessen mit echten Fensternachrichten: bei 60 Bildern je Sekunde
//! gehen Klicks unter ~16 ms zu einem grossen Teil verloren, zusammen
//! geschickt jeder; bei 15 Bildern je Sekunde noch Klicks von 40 ms. Das
//! trifft das Tippen aufs Touchpad und jedes Programm, das gerade langsam
//! laeuft.
//!
//! Abhilfe: ein eigener Rueckruf VOR dem von raylib zaehlt Druecken und
//! Loslassen je Taste; `Graphics::flip` holt die Zaehler einmal je Bild ab.
//! Liegt ein ganzer Klick in der Luecke, meldet Graphics die Taste EIN Bild
//! lang als gedrueckt und im naechsten als losgelassen -- so sieht jede
//! Stelle, die nur "ist gedrueckt" fragt, einen gewoehnlichen kurzen Klick.
//!
//! Fuer die TASTATUR gilt dasselbe (raylibs KeyCallback schreibt ebenso nur
//! `currentKeyState`), darum ein zweiter Satz Zaehler je GLFW-Tastencode und
//! ein zweiter Rueckruf. Getippte ZEICHEN sind nicht betroffen, sie laufen
//! ueber eine eigene Warteschlange.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

pub const TASTEN: usize = 5;

static DRUCK: [AtomicU32; TASTEN] = [const { AtomicU32::new(0) }; TASTEN];
static LOS: [AtomicU32; TASTEN] = [const { AtomicU32::new(0) }; TASTEN];
/// Der Rueckruf, den raylib gesetzt hatte (0 = keiner).
static VORHER: AtomicUsize = AtomicUsize::new(0);

/// GLFW-Tastencodes gehen bis 348 (GLFW_KEY_LAST); raylibs KeyboardKey
/// benutzt dieselben Nummern.
pub const TASTATUR: usize = 349;
static T_DRUCK: [AtomicU32; TASTATUR] = [const { AtomicU32::new(0) }; TASTATUR];
static T_LOS: [AtomicU32; TASTATUR] = [const { AtomicU32::new(0) }; TASTATUR];
static T_VORHER: AtomicUsize = AtomicUsize::new(0);

type TastenRueckruf = unsafe extern "C" fn(*mut std::ffi::c_void, i32, i32, i32, i32);

unsafe extern "C" fn tasten_rueckruf(fenster: *mut std::ffi::c_void, taste: i32, scan: i32, aktion: i32, mods: i32) {
    taste_zaehlen(taste, aktion);
    let v = T_VORHER.load(Ordering::SeqCst);
    if v != 0 {
        let f: TastenRueckruf = unsafe { std::mem::transmute::<usize, TastenRueckruf>(v) };
        unsafe { f(fenster, taste, scan, aktion, mods) };
    }
}

/// Wie `zaehlen`, fuer die Tastatur. Aktion 2 (Wiederholung beim Halten)
/// zaehlt nicht -- die Taste ist dabei die ganze Zeit unten.
pub fn taste_zaehlen(taste: i32, aktion: i32) {
    if taste < 0 || taste as usize >= TASTATUR { return; }
    let t = taste as usize;
    match aktion {
        1 => { T_DRUCK[t].fetch_add(1, Ordering::SeqCst); }
        0 => { T_LOS[t].fetch_add(1, Ordering::SeqCst); }
        _ => {}
    }
}

/// Tasten, die seit dem letzten Abholen gedrueckt oder losgelassen wurden:
/// (Code, Druecken, Loslassen). Meist leer oder sehr kurz -- darum eine
/// Liste statt 349 Paaren je Bild.
pub fn tasten_abholen() -> Vec<(usize, u32, u32)> {
    let mut v = Vec::new();
    for t in 0..TASTATUR {
        let d = T_DRUCK[t].swap(0, Ordering::SeqCst);
        let l = T_LOS[t].swap(0, Ordering::SeqCst);
        if d > 0 || l > 0 { v.push((t, d, l)); }
    }
    v
}

type Rueckruf = unsafe extern "C" fn(*mut std::ffi::c_void, i32, i32, i32);

unsafe extern "C" fn rueckruf(fenster: *mut std::ffi::c_void, taste: i32, aktion: i32, mods: i32) {
    zaehlen(taste, aktion);
    let v = VORHER.load(Ordering::SeqCst);
    if v != 0 {
        let f: Rueckruf = unsafe { std::mem::transmute::<usize, Rueckruf>(v) };
        unsafe { f(fenster, taste, aktion, mods) };
    }
}

/// Ein Druecken (aktion 1) oder Loslassen (0) mitzaehlen. Auch die
/// Wiedergabe einer Aufnahme ruft das -- sie laeuft an GLFW vorbei.
pub fn zaehlen(taste: i32, aktion: i32) {
    if taste < 0 || taste as usize >= TASTEN { return; }
    let t = taste as usize;
    match aktion {
        1 => { DRUCK[t].fetch_add(1, Ordering::SeqCst); }
        0 => { LOS[t].fetch_add(1, Ordering::SeqCst); }
        _ => {}
    }
}

/// Den eigenen Rueckruf vor den von raylib haengen; einmal nach dem
/// Anlegen des Fensters (der GL-Kontext ist dann aktuell).
pub fn einhaengen() {
    // Ohne raylib (Rust-Tests ohne Grafik) gibt es kein GLFW zum Linken.
    #[cfg(all(feature = "graphics", not(target_os = "emscripten")))]
    unsafe {
        unsafe extern "C" {
            fn glfwGetCurrentContext() -> *mut std::ffi::c_void;
            fn glfwSetMouseButtonCallback(fenster: *mut std::ffi::c_void, f: Option<Rueckruf>) -> Option<Rueckruf>;
            fn glfwSetKeyCallback(fenster: *mut std::ffi::c_void, f: Option<TastenRueckruf>) -> Option<TastenRueckruf>;
        }
        let f = glfwGetCurrentContext();
        if f.is_null() { return; }
        let alt = glfwSetMouseButtonCallback(f, Some(rueckruf));
        // Zweimal eingehaengt waere der Vorgaenger der eigene Rueckruf -- eine
        // Schleife ohne Ende.
        if let Some(a) = alt {
            if a as usize != rueckruf as usize { VORHER.store(a as usize, Ordering::SeqCst); }
        }
        let alt = glfwSetKeyCallback(f, Some(tasten_rueckruf));
        if let Some(a) = alt {
            if a as usize != tasten_rueckruf as usize { T_VORHER.store(a as usize, Ordering::SeqCst); }
        }
    }
}

/// Zaehler seit dem letzten Abholen, danach wieder 0.
pub fn abholen() -> ([u32; TASTEN], [u32; TASTEN]) {
    let mut d = [0; TASTEN];
    let mut l = [0; TASTEN];
    for t in 0..TASTEN {
        d[t] = DRUCK[t].swap(0, Ordering::SeqCst);
        l[t] = LOS[t].swap(0, Ordering::SeqCst);
    }
    (d, l)
}

/// Der Zustand EINER Taste ueber die Verlaengerung.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Halt {
    /// in diesem Bild gedrueckt gemeldet, obwohl raylib "los" sagt
    pub unten: bool,
    /// ... und zwar ab diesem Bild (Flanke fuer MOUSE_HIT)
    pub neu: bool,
    /// die Verlaengerung endet in diesem Bild (Flanke fuer MOUSE_RELEASED)
    pub los: bool,
}

/// Einmal je Bild: `druck`/`los` seit dem letzten Bild, `unten` = raylibs
/// Zustand jetzt, `flanke` = raylib hat selbst ein Druecken gesehen.
pub fn weiter(alt: Halt, druck: u32, los: u32, unten: bool, flanke: bool) -> Halt {
    // Ein Klick liegt ganz in der Luecke: gedrueckt UND losgelassen, und
    // raylib sieht weder die Taste unten noch eine Flanke.
    let verschluckt = druck > 0 && los > 0 && !unten && !flanke;
    if verschluckt {
        // Folgte er einem verlaengerten, zaehlt der alte als losgelassen --
        // sonst verschmoelzen zwei schnelle Klicks zu einem.
        return Halt { unten: true, neu: true, los: alt.unten };
    }
    Halt { unten: false, neu: false, los: alt.unten && !unten }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ein_verschluckter_klick_haelt_ein_bild() {
        let h = weiter(Halt::default(), 1, 1, false, false);
        assert_eq!(h, Halt { unten: true, neu: true, los: false });
        let h = weiter(h, 0, 0, false, false);
        assert_eq!(h, Halt { unten: false, neu: false, los: true });
        let h = weiter(h, 0, 0, false, false);
        assert_eq!(h, Halt::default());
    }

    #[test]
    fn ein_gewoehnlicher_klick_bleibt_bei_raylib() {
        // Druecken in einer Luecke, Loslassen in der naechsten: raylib sieht beides.
        let h = weiter(Halt::default(), 1, 0, true, true);
        assert_eq!(h, Halt::default());
        let h = weiter(h, 0, 1, false, false);
        assert_eq!(h, Halt::default());
    }

    #[test]
    fn gehalten_und_neu_gedrueckt_ist_kein_verschluckter() {
        // Taste unten, kurz los und wieder runter: raylib sieht sie weiter unten.
        let h = weiter(Halt::default(), 1, 1, true, false);
        assert_eq!(h, Halt::default());
    }

    #[test]
    fn zwei_schnelle_klicks_bleiben_zwei() {
        let h = weiter(Halt::default(), 1, 1, false, false);
        let h = weiter(h, 1, 1, false, false);
        assert!(h.unten && h.neu && h.los);
    }

    #[test]
    fn tastenzaehler_liefert_nur_beruehrte() {
        let _ = tasten_abholen();
        taste_zaehlen(65, 1); taste_zaehlen(65, 0); taste_zaehlen(65, 2);
        taste_zaehlen(348, 1); taste_zaehlen(349, 1); taste_zaehlen(-1, 1);
        assert_eq!(tasten_abholen(), vec![(65, 1, 1), (348, 1, 0)]);
        assert!(tasten_abholen().is_empty());
    }

    #[test]
    fn zaehler_nimmt_nur_bekannte_tasten() {
        let _ = abholen();
        zaehlen(0, 1); zaehlen(0, 0); zaehlen(9, 1); zaehlen(-1, 0); zaehlen(1, 2);
        let (d, l) = abholen();
        assert_eq!((d[0], l[0]), (1, 1));
        assert_eq!(d.iter().sum::<u32>() + l.iter().sum::<u32>(), 2);
        assert_eq!(abholen(), ([0; TASTEN], [0; TASTEN]));
    }
}
