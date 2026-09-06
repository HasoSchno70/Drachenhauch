//! Eingabemethoden (IME): die Umwandlung im Feld (docs/entwurf-eingabemethoden.md,
//! Weg C).
//!
//! Wer Japanisch, Chinesisch oder Koreanisch schreibt, tippt Silben, und eine
//! Eingabemethode wandelt sie in Zeichen um. Waehrend der Umwandlung gibt es
//! eine unfertige Zwischenstufe (die Vorschau) und am Ende ein Ergebnis.
//! Windows meldet beides als Fensternachrichten: `WM_IME_STARTCOMPOSITION`,
//! `WM_IME_COMPOSITION` (mit Vorschau `GCS_COMPSTR`, Schreibmarke darin
//! `GCS_CURSORPOS` und Ergebnis `GCS_RESULTSTR`), `WM_IME_ENDCOMPOSITION`.
//!
//! Ohne dieses Modul erledigt `DefWindowProc` die Vorschau in einem eigenen
//! kleinen Systemfenster und schickt das Ergebnis als `WM_CHAR`-Folge -- das
//! Verhalten aelterer Windows-Programme. Hier haengt sich dhrt mit einem
//! zweiten Subclass an raylibs GLFW-Fenster (derselbe Griff wie AccessKit in
//! a11y.rs, die Kette ruft den Vorgaenger), nimmt die drei Nachrichten
//! selbst und gibt sie NICHT weiter: dann entsteht das Systemfenster gar
//! nicht, die Vorschau steht im gui-Textfeld (unterstrichen, an der
//! Schreibmarke), das Ergebnis wird direkt eingefuegt, und `WM_IME_CHAR`
//! wird verschluckt, sonst kaeme alles doppelt.
//!
//! Das gilt NUR, solange ein Textfeld oder Textbereich des gui-Moduls den
//! Fokus hat (`feld_aktiv`). Ohne ein solches Feld -- ein Spiel mit `INKEY$`,
//! das `ui`-Modul -- laeuft alles wie bisher durch, damit dort weiter die
//! `WM_CHAR` ankommen. Waehrend einer Umwandlung filtert die IME die Tasten
//! (GLFW sieht `VK_PROCESSKEY` und meldet nichts), Pfeile und Enter gehoeren
//! also der IME, nicht dem Feld.
//!
//! Auf dieser Maschine ist keine Eingabemethode installiert -- der Weg ist
//! nach der Windows-Dokumentation gebaut und mit Rust-Tests fuer die
//! Zeichenkettenseite belegt, nicht mit einer echten IME. Wer eine hat, ist
//! die Abnahme.

#[cfg(windows)]
mod plattform {
    use std::sync::atomic::{AtomicIsize, Ordering};
    use std::sync::Mutex;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Input::Ime::{
        ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, GCS_COMPSTR, GCS_CURSORPOS,
        GCS_RESULTSTR, IME_COMPOSITION_STRING,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WM_IME_CHAR, WM_IME_COMPOSITION,
        WM_IME_ENDCOMPOSITION, WM_IME_STARTCOMPOSITION, WNDPROC,
    };

    #[derive(Default)]
    struct Stand {
        feld_aktiv: bool,
        vorschau: String,
        marke: i32,
        komponiert: bool,
        ergebnisse: Vec<String>,
    }

    static STAND: Mutex<Stand> = Mutex::new(Stand { feld_aktiv: false, vorschau: String::new(), marke: 0,
                                                   komponiert: false, ergebnisse: Vec::new() });
    static ALT_PROC: AtomicIsize = AtomicIsize::new(0);

    /// Zeichenkette der IME lesen (`GCS_COMPSTR` oder `GCS_RESULTSTR`):
    /// erst die Laenge in Bytes erfragen, dann den Puffer fuellen.
    unsafe fn ime_text(ctx: windows::Win32::UI::Input::Ime::HIMC, was: IME_COMPOSITION_STRING) -> String {
        let bytes = unsafe { ImmGetCompositionStringW(ctx, was, None, 0) };
        if bytes <= 0 { return String::new(); }
        let mut puffer: Vec<u16> = vec![0; (bytes as usize + 1) / 2];
        let gelesen = unsafe { ImmGetCompositionStringW(ctx, was, Some(puffer.as_mut_ptr() as *mut _), bytes as u32) };
        if gelesen <= 0 { return String::new(); }
        puffer.truncate(gelesen as usize / 2);
        String::from_utf16_lossy(&puffer)
    }

    unsafe extern "system" fn fensterprozedur(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        let alt = ALT_PROC.load(Ordering::SeqCst);
        let weiter = |hwnd, msg, wp, lp| -> LRESULT {
            let vorgaenger: WNDPROC = unsafe { std::mem::transmute::<isize, WNDPROC>(alt) };
            unsafe { CallWindowProcW(vorgaenger, hwnd, msg, wp, lp) }
        };
        let aktiv = STAND.lock().map(|s| s.feld_aktiv).unwrap_or(false);
        if !aktiv { return weiter(hwnd, msg, wp, lp); }
        match msg {
            WM_IME_STARTCOMPOSITION => {
                if let Ok(mut s) = STAND.lock() { s.komponiert = true; s.vorschau.clear(); s.marke = 0; }
                LRESULT(0)
            }
            WM_IME_COMPOSITION => {
                let ctx = unsafe { ImmGetContext(hwnd) };
                if !ctx.is_invalid() {
                    let flags = lp.0 as u32;
                    // Erst das Ergebnis (ein fertiger Satzteil), dann die
                    // Vorschau des Rests -- beide koennen in EINER Nachricht
                    // stehen.
                    if flags & GCS_RESULTSTR.0 != 0 {
                        let erg = unsafe { ime_text(ctx, GCS_RESULTSTR) };
                        if !erg.is_empty() { if let Ok(mut s) = STAND.lock() { s.ergebnisse.push(erg); } }
                    }
                    if flags & GCS_COMPSTR.0 != 0 {
                        let comp = unsafe { ime_text(ctx, GCS_COMPSTR) };
                        let marke = if flags & GCS_CURSORPOS.0 != 0 {
                            unsafe { ImmGetCompositionStringW(ctx, GCS_CURSORPOS, None, 0) }.max(0)
                        } else { comp.chars().count() as i32 };
                        if let Ok(mut s) = STAND.lock() { s.marke = marke.min(comp.chars().count() as i32); s.vorschau = comp; s.komponiert = true; }
                    }
                    let _ = unsafe { ImmReleaseContext(hwnd, ctx) };
                }
                LRESULT(0)
            }
            WM_IME_ENDCOMPOSITION => {
                if let Ok(mut s) = STAND.lock() { s.komponiert = false; s.vorschau.clear(); s.marke = 0; }
                LRESULT(0)
            }
            // Das Ergebnis kam ueber GCS_RESULTSTR -- die Zeichen hier noch
            // einmal durchzulassen gaebe jedes doppelt.
            WM_IME_CHAR => LRESULT(0),
            _ => weiter(hwnd, msg, wp, lp),
        }
    }

    /// Den Subclass an das Fenster haengen (einmal, beim Anlegen).
    pub fn einhaengen(handle: *mut std::ffi::c_void) {
        if handle.is_null() || ALT_PROC.load(Ordering::SeqCst) != 0 { return; }
        let hwnd = HWND(handle);
        let alt = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, fensterprozedur as *const () as isize) };
        if alt != 0 { ALT_PROC.store(alt, Ordering::SeqCst); }
    }

    /// Hat ein gui-Textfeld den Fokus? Nur dann nimmt dhrt die Umwandlung
    /// selbst in die Hand.
    pub fn feld_aktiv(an: bool) {
        if let Ok(mut s) = STAND.lock() {
            if s.feld_aktiv && !an { s.vorschau.clear(); s.marke = 0; s.komponiert = false; }
            s.feld_aktiv = an;
        }
    }

    /// Fertige Ergebnisse (FIFO) und die aktuelle Vorschau mit ihrer
    /// Schreibmarke -- None, wenn gerade keine Umwandlung laeuft.
    pub fn abholen() -> (Vec<String>, Option<(String, i32)>) {
        match STAND.lock() {
            Ok(mut s) => {
                let erg = std::mem::take(&mut s.ergebnisse);
                let vorschau = if s.komponiert && !s.vorschau.is_empty() { Some((s.vorschau.clone(), s.marke)) } else { None };
                (erg, vorschau)
            }
            Err(_) => (Vec::new(), None),
        }
    }
}

#[cfg(not(windows))]
mod plattform {
    pub fn einhaengen(_handle: *mut std::ffi::c_void) {}
    pub fn feld_aktiv(_an: bool) {}
    pub fn abholen() -> (Vec<String>, Option<(String, i32)>) { (Vec::new(), None) }
}

pub use plattform::{abholen, einhaengen, feld_aktiv};
