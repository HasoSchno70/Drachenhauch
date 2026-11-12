//! Echte OS-Fenster fuer gui-Fenster (Weg E, docs/intern/entwurf-native-fenster.md).
//!
//! raylib behaelt sein EINES Fenster. Ein natives Fenster zeichnet nichts
//! selbst: die gui zeichnet das gui-Fenster im Hauptkontext in ein
//! Render-Ziel (graphics.rs, `render_ziel`), und dieses Modul zeigt dessen
//! Textur in einem zweiten GLFW-Fenster mit GETEILTEM Kontext -- eine eigene
//! FBO mit der geteilten Textur als Lesepuffer, `glBlitFramebuffer`,
//! `glfwSwapBuffers`, zurueck in den Hauptkontext (erprobt in Stufe 0,
//! examples/spike_zweitfenster.rs). Texturen teilen sich zwischen Kontexten,
//! FBOs und VAOs nicht -- darum braucht der zweite Kontext weder Shader noch
//! rlgl.
//!
//! Die Eingabe des Fensters kommt ueber GLFW-Rueckrufe an GENAU diesem
//! Fenster (raylib setzt seine nur am Hauptfenster) und sammelt sich hier je
//! Fenster; `Graphics::flip` holt sie einmal je Bild ab.
//!
//! Nur Windows und Linux: unter macOS ist der Weg ungeprueft (Stufe 0 lief
//! dort nicht, die CI-Rechner haben kein OpenGL fuer Fenster) -- `anlegen`
//! liefert dort `None`, und die gui bleibt beim Rueckfall (ein gui-Fenster im
//! Hauptfenster). Im Browser gibt es ohnehin nur eine Zeichenflaeche.

#![allow(dead_code)]

use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void, CString};
use std::sync::Mutex;

/// GLFW-Tastencodes gehen bis 348 (wie flanken::TASTATUR).
pub const TASTATUR: usize = 349;
/// Maustasten 0..4 (wie flanken::TASTEN).
pub const MAUSTASTEN: usize = 5;

/// Was seit dem letzten Abholen in EINEM Fenster geschah.
#[derive(Clone, Debug)]
pub struct Eingang {
    /// Zeiger in Fensterpunkten (nicht Bildpunkten -- bei hoher Pixeldichte
    /// rechnet GLFW in Fensterkoordinaten).
    pub maus: (f64, f64),
    /// Steht der Zeiger ueber dem Fenster?
    pub drinnen: bool,
    pub knopf_unten: [bool; MAUSTASTEN],
    pub knopf_druck: [u32; MAUSTASTEN],
    pub knopf_los: [u32; MAUSTASTEN],
    pub rad: (f64, f64),
    pub zeichen: Vec<char>,
    pub taste_unten: Vec<bool>,
    pub taste_druck: Vec<u32>,
    pub taste_los: Vec<u32>,
    /// Tasten mit Wiederholung (gehalten) seit dem letzten Abholen.
    pub wiederholt: Vec<usize>,
    /// Hat das Fenster den Tastaturfokus des Systems?
    pub fokus: bool,
    /// Neue Groesse (Fensterpunkte), wenn sie sich geaendert hat.
    pub groesse: Option<(i32, i32)>,
}

impl Default for Eingang {
    fn default() -> Self {
        Eingang {
            maus: (0.0, 0.0), drinnen: false,
            knopf_unten: [false; MAUSTASTEN], knopf_druck: [0; MAUSTASTEN], knopf_los: [0; MAUSTASTEN],
            rad: (0.0, 0.0), zeichen: Vec::new(),
            taste_unten: vec![false; TASTATUR], taste_druck: vec![0; TASTATUR], taste_los: vec![0; TASTATUR],
            wiederholt: Vec::new(), fokus: false, groesse: None,
        }
    }
}

impl Eingang {
    /// Was ein Bild lang gilt (Zaehler, Rad, Zeichen, Groesse) abgeben; der
    /// Zustand (unten, Lage, Fokus) bleibt stehen.
    fn abgeben(&mut self) -> Eingang {
        let aus = self.clone();
        self.knopf_druck = [0; MAUSTASTEN];
        self.knopf_los = [0; MAUSTASTEN];
        self.rad = (0.0, 0.0);
        self.zeichen.clear();
        for t in 0..TASTATUR { self.taste_druck[t] = 0; self.taste_los[t] = 0; }
        self.wiederholt.clear();
        self.groesse = None;
        aus
    }
}

/// Eingang je Fenster, Schluessel ist das GLFW-Fenster als Zahl. Die
/// Rueckrufe kommen im Hauptfaden (in glfwPollEvents), die Sperre ist nur
/// fuer den Compiler noetig.
static EINGAENGE: Mutex<Option<HashMap<usize, Eingang>>> = Mutex::new(None);

fn mit_eingang(f: *mut c_void, tun: impl FnOnce(&mut Eingang)) {
    let mut m = EINGAENGE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(e) = m.get_or_insert_with(HashMap::new).get_mut(&(f as usize)) { tun(e); }
}

/// Pure Umrechnung, fuer Rueckrufe und Tests: eine GLFW-Aktion an einer
/// Taste (1 = gedrueckt, 0 = losgelassen, 2 = Wiederholung).
pub fn taste_buchen(e: &mut Eingang, taste: i32, aktion: i32) {
    if taste < 0 || taste as usize >= TASTATUR { return; }
    let t = taste as usize;
    match aktion {
        1 => { e.taste_unten[t] = true; e.taste_druck[t] += 1; }
        0 => { e.taste_unten[t] = false; e.taste_los[t] += 1; }
        2 => { if !e.wiederholt.contains(&t) { e.wiederholt.push(t); } }
        _ => {}
    }
}

pub fn knopf_buchen(e: &mut Eingang, knopf: i32, aktion: i32) {
    if knopf < 0 || knopf as usize >= MAUSTASTEN { return; }
    let k = knopf as usize;
    match aktion {
        1 => { e.knopf_unten[k] = true; e.knopf_druck[k] += 1; }
        0 => { e.knopf_unten[k] = false; e.knopf_los[k] += 1; }
        _ => {}
    }
}

// ----------------------------------------------------------------- GLFW

#[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
mod glfw {
    use super::*;

    pub type Fn4 = unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int);
    pub type Fn5 = unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, c_int);
    pub type FnPos = unsafe extern "C" fn(*mut c_void, f64, f64);
    pub type FnInt = unsafe extern "C" fn(*mut c_void, c_int);
    pub type FnUint = unsafe extern "C" fn(*mut c_void, u32);
    pub type FnGroesse = unsafe extern "C" fn(*mut c_void, c_int, c_int);

    unsafe extern "C" {
        pub fn glfwGetCurrentContext() -> *mut c_void;
        pub fn glfwDefaultWindowHints();
        pub fn glfwWindowHint(hint: c_int, wert: c_int);
        pub fn glfwCreateWindow(b: c_int, h: c_int, titel: *const c_char, monitor: *mut c_void, teilen: *mut c_void) -> *mut c_void;
        pub fn glfwDestroyWindow(f: *mut c_void);
        pub fn glfwMakeContextCurrent(f: *mut c_void);
        pub fn glfwSwapInterval(n: c_int);
        pub fn glfwSwapBuffers(f: *mut c_void);
        pub fn glfwGetProcAddress(name: *const c_char) -> *const c_void;
        pub fn glfwWindowShouldClose(f: *mut c_void) -> c_int;
        pub fn glfwSetWindowShouldClose(f: *mut c_void, wert: c_int);
        pub fn glfwGetFramebufferSize(f: *mut c_void, b: *mut c_int, h: *mut c_int);
        pub fn glfwGetWindowPos(f: *mut c_void, x: *mut c_int, y: *mut c_int);
        pub fn glfwSetWindowPos(f: *mut c_void, x: c_int, y: c_int);
        pub fn glfwSetWindowSize(f: *mut c_void, b: c_int, h: c_int);
        pub fn glfwSetWindowTitle(f: *mut c_void, titel: *const c_char);
        pub fn glfwShowWindow(f: *mut c_void);
        pub fn glfwHideWindow(f: *mut c_void);
        pub fn glfwFocusWindow(f: *mut c_void);
        pub fn glfwSetMouseButtonCallback(f: *mut c_void, r: Option<Fn4>) -> Option<Fn4>;
        pub fn glfwSetKeyCallback(f: *mut c_void, r: Option<Fn5>) -> Option<Fn5>;
        pub fn glfwSetCursorPosCallback(f: *mut c_void, r: Option<FnPos>) -> Option<FnPos>;
        pub fn glfwSetCursorEnterCallback(f: *mut c_void, r: Option<FnInt>) -> Option<FnInt>;
        pub fn glfwSetScrollCallback(f: *mut c_void, r: Option<FnPos>) -> Option<FnPos>;
        pub fn glfwSetCharCallback(f: *mut c_void, r: Option<FnUint>) -> Option<FnUint>;
        pub fn glfwSetWindowFocusCallback(f: *mut c_void, r: Option<FnInt>) -> Option<FnInt>;
        pub fn glfwSetWindowSizeCallback(f: *mut c_void, r: Option<FnGroesse>) -> Option<FnGroesse>;
        #[cfg(windows)]
        pub fn glfwGetWin32Window(f: *mut c_void) -> *mut c_void;
    }

    pub const CONTEXT_VERSION_MAJOR: c_int = 0x0002_2002;
    pub const CONTEXT_VERSION_MINOR: c_int = 0x0002_2003;
    pub const OPENGL_PROFILE: c_int = 0x0002_2008;
    pub const OPENGL_CORE_PROFILE: c_int = 0x0003_2001;
    pub const RESIZABLE: c_int = 0x0002_0003;
    pub const VISIBLE: c_int = 0x0002_0004;
    pub const FOCUS_ON_SHOW: c_int = 0x0002_000C;

    pub unsafe extern "C" fn r_knopf(f: *mut c_void, knopf: c_int, aktion: c_int, _m: c_int) {
        mit_eingang(f, |e| knopf_buchen(e, knopf, aktion));
    }
    pub unsafe extern "C" fn r_taste(f: *mut c_void, taste: c_int, _scan: c_int, aktion: c_int, _m: c_int) {
        mit_eingang(f, |e| taste_buchen(e, taste, aktion));
    }
    pub unsafe extern "C" fn r_lage(f: *mut c_void, x: f64, y: f64) {
        mit_eingang(f, |e| e.maus = (x, y));
    }
    pub unsafe extern "C" fn r_drinnen(f: *mut c_void, drin: c_int) {
        mit_eingang(f, |e| e.drinnen = drin != 0);
    }
    pub unsafe extern "C" fn r_rad(f: *mut c_void, x: f64, y: f64) {
        mit_eingang(f, |e| { e.rad.0 += x; e.rad.1 += y; });
    }
    pub unsafe extern "C" fn r_zeichen(f: *mut c_void, c: u32) {
        mit_eingang(f, |e| if let Some(z) = char::from_u32(c) { e.zeichen.push(z) });
    }
    pub unsafe extern "C" fn r_fokus(f: *mut c_void, an: c_int) {
        mit_eingang(f, |e| {
            e.fokus = an != 0;
            // Ohne Fokus kommt kein Loslassen mehr an -- nichts darf haengen.
            if an == 0 {
                for k in 0..MAUSTASTEN { if e.knopf_unten[k] { e.knopf_unten[k] = false; e.knopf_los[k] += 1; } }
                for t in 0..TASTATUR { if e.taste_unten[t] { e.taste_unten[t] = false; e.taste_los[t] += 1; } }
            }
        });
    }
    pub unsafe extern "C" fn r_groesse(f: *mut c_void, b: c_int, h: c_int) {
        mit_eingang(f, |e| if b > 0 && h > 0 { e.groesse = Some((b, h)) });
    }
}

/// Die GL-Befehle des zweiten Kontexts -- geladen, waehrend er aktuell ist.
#[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
struct Gl {
    gen_framebuffers: extern "system" fn(i32, *mut u32),
    delete_framebuffers: extern "system" fn(i32, *const u32),
    bind_framebuffer: extern "system" fn(u32, u32),
    framebuffer_texture_2d: extern "system" fn(u32, u32, u32, u32, i32),
    blit_framebuffer: extern "system" fn(i32, i32, i32, i32, i32, i32, i32, i32, u32, u32),
    clear_color: extern "system" fn(f32, f32, f32, f32),
    clear: extern "system" fn(u32),
}

/// Ein natives Fenster: das GLFW-Fenster und seine FBO im zweiten Kontext.
pub struct Fenster {
    handle: *mut c_void,
    #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
    gl: Gl,
    fbo: u32,
    titel: String,
}

const GL_READ_FRAMEBUFFER: u32 = 0x8CA8;
const GL_DRAW_FRAMEBUFFER: u32 = 0x8CA9;
const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
const GL_NEAREST: u32 = 0x2600;

/// Das GLFW-Fenster, dessen Kontext gerade aktuell ist -- direkt nach dem
/// Anlegen von raylibs Fenster ist das das Hauptfenster.
pub fn aktueller_kontext() -> usize {
    #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
    unsafe { return glfw::glfwGetCurrentContext() as usize; }
    #[allow(unreachable_code)]
    0
}

/// Das Fenster des Betriebssystems zu einem GLFW-Fenster (Windows: HWND).
#[allow(unused_variables)]
pub fn os_handle(f: *mut c_void) -> usize {
    #[cfg(all(feature = "graphics", windows))]
    unsafe { return glfw::glfwGetWin32Window(f) as usize; }
    #[allow(unreachable_code)]
    0
}

/// Gibt es echte Fenster auf diesem System? (Fuer den Rueckfall der gui.)
pub fn moeglich() -> bool {
    cfg!(all(feature = "graphics", any(windows, target_os = "linux")))
        && std::env::var("DHRT_OHNE_NATIVE_FENSTER").map(|v| v != "1").unwrap_or(true)
}

impl Fenster {
    /// Ein Fenster anlegen, Kontext mit `haupt` geteilt. `None`, wo es keine
    /// echten Fenster gibt oder GLFW es ablehnt -- der Aufrufer bleibt dann
    /// beim gui-Fenster im Hauptfenster.
    #[allow(unused_variables)]
    pub fn anlegen(haupt: *mut c_void, titel: &str, b: i32, h: i32, x: i32, y: i32, groesse_aenderbar: bool) -> Option<Fenster> {
        if !moeglich() || haupt.is_null() { return None; }
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        unsafe {
            use glfw::*;
            glfwDefaultWindowHints();
            glfwWindowHint(CONTEXT_VERSION_MAJOR, 3);
            glfwWindowHint(CONTEXT_VERSION_MINOR, 3);
            glfwWindowHint(OPENGL_PROFILE, OPENGL_CORE_PROFILE);
            glfwWindowHint(RESIZABLE, groesse_aenderbar as c_int);
            // Erst unsichtbar anlegen, an die Stelle setzen, dann zeigen --
            // sonst springt es sichtbar von (0,0) an seinen Platz.
            glfwWindowHint(VISIBLE, 0);
            let t = CString::new(titel.replace('\0', " ")).ok()?;
            let f = glfwCreateWindow(b.max(1), h.max(1), t.as_ptr(), std::ptr::null_mut(), haupt);
            glfwDefaultWindowHints();
            if f.is_null() { return None; }
            {
                let mut m = EINGAENGE.lock().unwrap_or_else(|e| e.into_inner());
                m.get_or_insert_with(HashMap::new).insert(f as usize, Eingang::default());
            }
            glfwSetMouseButtonCallback(f, Some(r_knopf));
            glfwSetKeyCallback(f, Some(r_taste));
            glfwSetCursorPosCallback(f, Some(r_lage));
            glfwSetCursorEnterCallback(f, Some(r_drinnen));
            glfwSetScrollCallback(f, Some(r_rad));
            glfwSetCharCallback(f, Some(r_zeichen));
            glfwSetWindowFocusCallback(f, Some(r_fokus));
            glfwSetWindowSizeCallback(f, Some(r_groesse));
            glfwSetWindowPos(f, x, y);

            glfwMakeContextCurrent(f);
            // Sonst wartet jedes Fenster auf seinen eigenen Bildwechsel.
            glfwSwapInterval(0);
            let gl = Gl {
                gen_framebuffers: laden("glGenFramebuffers")?,
                delete_framebuffers: laden("glDeleteFramebuffers")?,
                bind_framebuffer: laden("glBindFramebuffer")?,
                framebuffer_texture_2d: laden("glFramebufferTexture2D")?,
                blit_framebuffer: laden("glBlitFramebuffer")?,
                clear_color: laden("glClearColor")?,
                clear: laden("glClear")?,
            };
            let mut fbo = 0u32;
            (gl.gen_framebuffers)(1, &mut fbo);
            glfwMakeContextCurrent(haupt);
            glfwShowWindow(f);
            Some(Fenster { handle: f, gl, fbo, titel: titel.to_string() })
        }
        #[cfg(not(all(feature = "graphics", any(windows, target_os = "linux"))))]
        None
    }

    /// Die Textur `textur` (b x h) zeigen. Danach ist wieder `haupt` aktuell.
    #[allow(unused_variables)]
    pub fn zeigen(&self, haupt: *mut c_void, textur: u32, b: i32, h: i32) {
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        unsafe {
            use glfw::*;
            glfwMakeContextCurrent(self.handle);
            let (mut fb, mut fh) = (0, 0);
            glfwGetFramebufferSize(self.handle, &mut fb, &mut fh);
            if fb > 0 && fh > 0 {
                (self.gl.bind_framebuffer)(GL_DRAW_FRAMEBUFFER, 0);
                (self.gl.clear_color)(0.0, 0.0, 0.0, 1.0);
                (self.gl.clear)(GL_COLOR_BUFFER_BIT);
                (self.gl.bind_framebuffer)(GL_READ_FRAMEBUFFER, self.fbo);
                // Jedes Bild neu anhaengen: nach einer Groessenaenderung ist
                // es eine andere Textur.
                (self.gl.framebuffer_texture_2d)(GL_READ_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, textur, 0);
                // NICHT spiegeln: dhrts Ziele (render_ziel, abgespielt mit
                // begin_texture_mode) stehen schon richtig herum -- der Spike
                // spiegelte, seine Proben lagen aber symmetrisch und sahen es
                // nicht. Bei hoher Pixeldichte ist der Bildpuffer groesser als
                // das Fenster -- gestreckt wird beim Kopieren.
                (self.gl.blit_framebuffer)(0, 0, b, h, 0, 0, fb, fh, GL_COLOR_BUFFER_BIT, GL_NEAREST);
                glfwSwapBuffers(self.handle);
            }
            glfwMakeContextCurrent(haupt);
        }
    }

    /// Was seit dem letzten Mal geschah (je Bild einmal).
    pub fn abholen(&self) -> Eingang {
        let mut m = EINGAENGE.lock().unwrap_or_else(|e| e.into_inner());
        match m.get_or_insert_with(HashMap::new).get_mut(&(self.handle as usize)) {
            Some(e) => e.abgeben(),
            None => Eingang::default(),
        }
    }

    /// Hat das System das Fenster schliessen wollen (Kreuz, Alt+F4)? Der
    /// Wunsch wird dabei zurueckgesetzt -- schliessen tut der Aufrufer.
    pub fn schliessen_gewuenscht(&self) -> bool {
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        unsafe {
            if glfw::glfwWindowShouldClose(self.handle) != 0 {
                glfw::glfwSetWindowShouldClose(self.handle, 0);
                return true;
            }
        }
        false
    }

    pub fn lage(&self) -> (i32, i32) {
        let (mut x, mut y) = (0, 0);
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        unsafe { glfw::glfwGetWindowPos(self.handle, &mut x, &mut y); }
        (x, y)
    }
    #[allow(unused_variables)]
    pub fn lage_setzen(&self, x: i32, y: i32) {
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        unsafe { glfw::glfwSetWindowPos(self.handle, x, y); }
    }
    #[allow(unused_variables)]
    pub fn groesse_setzen(&self, b: i32, h: i32) {
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        unsafe { glfw::glfwSetWindowSize(self.handle, b.max(1), h.max(1)); }
    }
    pub fn titel_setzen(&mut self, titel: &str) {
        if titel == self.titel { return; }
        self.titel = titel.to_string();
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        if let Ok(t) = CString::new(titel.replace('\0', " ")) {
            unsafe { glfw::glfwSetWindowTitle(self.handle, t.as_ptr()); }
        }
    }
    #[allow(unused_variables)]
    pub fn sichtbar(&self, an: bool) {
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        unsafe { if an { glfw::glfwShowWindow(self.handle) } else { glfw::glfwHideWindow(self.handle) } }
    }
    pub fn fokussieren(&self) {
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        unsafe { glfw::glfwFocusWindow(self.handle); }
    }
    /// Das Fenster des Betriebssystems (unter Windows das HWND), 0 sonst --
    /// fuer die Eingabemethode (ImmSetCompositionWindow).
    pub fn os_handle(&self) -> usize { os_handle(self.handle) }

    /// Das Hauptfenster als Besitzer (Weg E, Stufe 2): unter Windows bleibt
    /// ein besessenes Fenster ueber seinem Besitzer und hat keinen eigenen
    /// Eintrag in der Taskleiste -- so steht ein modaler Dialog. Unter Linux
    /// (noch) nichts: GLFW kennt dort kein "transient for".
    #[allow(unused_variables)]
    pub fn besitzer_setzen(&self, haupt: *mut c_void, an: bool) {
        #[cfg(all(feature = "graphics", windows))]
        unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{SetWindowLongPtrW, GWLP_HWNDPARENT};
            let ich = glfw::glfwGetWin32Window(self.handle);
            if ich.is_null() { return; }
            let wert = if an { glfw::glfwGetWin32Window(haupt) as isize } else { 0 };
            SetWindowLongPtrW(HWND(ich), GWLP_HWNDPARENT, wert);
        }
    }

    /// Das Fenster als Zahl -- fuer Tests (Fensternachrichten) und Handles.
    pub fn handle(&self) -> usize { self.handle as usize }

    /// Weg damit. Danach ist wieder `haupt` aktuell.
    #[allow(unused_variables)]
    pub fn schliessen(self, haupt: *mut c_void) {
        {
            let mut m = EINGAENGE.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(map) = m.as_mut() { map.remove(&(self.handle as usize)); }
        }
        #[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
        unsafe {
            use glfw::*;
            glfwMakeContextCurrent(self.handle);
            (self.gl.delete_framebuffers)(1, &self.fbo);
            glfwMakeContextCurrent(haupt);
            glfwDestroyWindow(self.handle);
            glfwMakeContextCurrent(haupt);
        }
    }
}

#[cfg(all(feature = "graphics", any(windows, target_os = "linux")))]
unsafe fn laden<T>(name: &str) -> Option<T> {
    let c = CString::new(name).ok()?;
    let p = unsafe { glfw::glfwGetProcAddress(c.as_ptr()) };
    if p.is_null() { return None; }
    Some(unsafe { std::mem::transmute_copy(&p) })
}

/// Wo der Zeiger gerade ist: die Oberflaeche (0 = Hauptfenster), und ob eine
/// gedrueckte Taste sie festhaelt. Pure Regel, fuer Graphics und Tests:
/// solange eine Maustaste in einem Fenster gedrueckt ist, bleibt der Zeiger
/// dort (sonst risse ein Ziehen ueber den Rand ab); sonst gilt das Fenster,
/// ueber dem er steht, und ausserhalb aller nativen das Hauptfenster.
pub fn zeiger_flaeche(gefangen: Option<u32>, drinnen: &[(u32, bool)]) -> u32 {
    if let Some(k) = gefangen { return k; }
    drinnen.iter().find(|(_, d)| *d).map(|(k, _)| *k).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tasten_und_knoepfe_buchen() {
        let mut e = Eingang::default();
        taste_buchen(&mut e, 65, 1);
        taste_buchen(&mut e, 65, 2);
        taste_buchen(&mut e, 65, 2);
        assert!(e.taste_unten[65]);
        assert_eq!(e.taste_druck[65], 1);
        assert_eq!(e.wiederholt, vec![65]);
        taste_buchen(&mut e, 65, 0);
        assert!(!e.taste_unten[65]);
        assert_eq!(e.taste_los[65], 1);
        taste_buchen(&mut e, -1, 1);
        taste_buchen(&mut e, 999, 1);
        knopf_buchen(&mut e, 0, 1);
        knopf_buchen(&mut e, 0, 0);
        assert_eq!((e.knopf_druck[0], e.knopf_los[0], e.knopf_unten[0]), (1, 1, false));
    }

    #[test]
    fn abgeben_leert_nur_was_ein_bild_gilt() {
        let mut e = Eingang::default();
        knopf_buchen(&mut e, 1, 1);
        e.rad = (0.0, 2.0);
        e.zeichen.push('a');
        e.maus = (10.0, 20.0);
        e.groesse = Some((300, 200));
        let a = e.abgeben();
        assert_eq!(a.knopf_druck[1], 1);
        assert_eq!(a.zeichen, vec!['a']);
        assert_eq!(a.groesse, Some((300, 200)));
        // Zustand bleibt, Ereignisse sind weg.
        assert!(e.knopf_unten[1]);
        assert_eq!(e.maus, (10.0, 20.0));
        assert_eq!(e.knopf_druck[1], 0);
        assert!(e.zeichen.is_empty());
        assert_eq!(e.rad, (0.0, 0.0));
        assert_eq!(e.groesse, None);
    }

    #[test]
    fn der_zeiger_gehoert_einer_flaeche() {
        assert_eq!(zeiger_flaeche(None, &[]), 0);
        assert_eq!(zeiger_flaeche(None, &[(1, false), (2, true)]), 2);
        // Gedrueckt in 1, jetzt ueber 2: bleibt bei 1.
        assert_eq!(zeiger_flaeche(Some(1), &[(1, false), (2, true)]), 1);
        assert_eq!(zeiger_flaeche(None, &[(1, false)]), 0);
    }
}
