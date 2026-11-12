//! Spike zu Weg E (docs/intern/entwurf-native-fenster.md, Stufe 0) --
//! Wegwerf-Code, nicht Teil von dhrt.
//!
//! Fragen:
//!   1. Laesst sich neben raylibs Fenster ein zweites GLFW-Fenster mit
//!      GETEILTEM GL-Kontext anlegen?
//!   2. Zeigt es ein Bild, das raylib im Hauptkontext in ein Render-Ziel
//!      zeichnet? (Texturen teilen sich zwischen Kontexten, Framebuffer
//!      nicht -- darum eine eigene FBO im zweiten Kontext und glBlitFramebuffer,
//!      ohne Shader und ohne VAO.)
//!   3. Kommen Klicks des zweiten Fensters an (eigene GLFW-Rueckrufe)?
//!   4. Bleibt raylib nach dem Wechsel der Kontexte heil, und wie viel
//!      kostet das Zeigen je Bild?
//!
//! Der Lauf prueft sich selbst: er liest das Pixel zurueck, das das zweite
//! Fenster zeigt, und klickt es (Windows: WM_LBUTTONDOWN an das Fenster).
//! Ausgabe: Zeilen "ERGEBNIS ..." -- und Exit 0 nur, wenn alles stimmt.
//!
//!   cargo build --release --example spike_zweitfenster --features graphics
//!   target/release/examples/spike_zweitfenster.exe

use std::ffi::{c_char, c_int, c_void, CString};
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Instant;

use raylib::ffi;

type GlfwFenster = *mut c_void;
type Rueckruf = extern "C" fn(GlfwFenster, c_int, c_int, c_int);

unsafe extern "C" {
    fn glfwGetCurrentContext() -> GlfwFenster;
    fn glfwDefaultWindowHints();
    fn glfwWindowHint(hint: c_int, wert: c_int);
    fn glfwCreateWindow(b: c_int, h: c_int, titel: *const c_char, monitor: *mut c_void, teilen: GlfwFenster) -> GlfwFenster;
    fn glfwDestroyWindow(f: GlfwFenster);
    fn glfwMakeContextCurrent(f: GlfwFenster);
    fn glfwSwapInterval(n: c_int);
    fn glfwSwapBuffers(f: GlfwFenster);
    fn glfwGetProcAddress(name: *const c_char) -> *const c_void;
    fn glfwSetMouseButtonCallback(f: GlfwFenster, r: Option<Rueckruf>) -> Option<Rueckruf>;
    fn glfwWindowShouldClose(f: GlfwFenster) -> c_int;
    fn glfwGetFramebufferSize(f: GlfwFenster, b: *mut c_int, h: *mut c_int);
    fn glfwSetWindowPos(f: GlfwFenster, x: c_int, y: c_int);
    #[cfg(windows)]
    fn glfwGetWin32Window(f: GlfwFenster) -> *mut c_void;
}

const GLFW_CONTEXT_VERSION_MAJOR: c_int = 0x0002_2002;
const GLFW_CONTEXT_VERSION_MINOR: c_int = 0x0002_2003;
const GLFW_OPENGL_FORWARD_COMPAT: c_int = 0x0002_2006;
const GLFW_OPENGL_PROFILE: c_int = 0x0002_2008;
const GLFW_OPENGL_CORE_PROFILE: c_int = 0x0003_2001;

const GL_READ_FRAMEBUFFER: u32 = 0x8CA8;
const GL_DRAW_FRAMEBUFFER: u32 = 0x8CA9;
const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
const GL_NEAREST: u32 = 0x2600;
const GL_FRAMEBUFFER_COMPLETE: u32 = 0x8CD5;
const GL_RGBA: u32 = 0x1908;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
const GL_BACK: u32 = 0x0405;

/// Die GL-Befehle des ZWEITEN Kontexts -- geladen, waehrend er aktuell ist.
struct Gl {
    gen_framebuffers: extern "system" fn(i32, *mut u32),
    bind_framebuffer: extern "system" fn(u32, u32),
    framebuffer_texture_2d: extern "system" fn(u32, u32, u32, u32, i32),
    check_framebuffer_status: extern "system" fn(u32) -> u32,
    blit_framebuffer: extern "system" fn(i32, i32, i32, i32, i32, i32, i32, i32, u32, u32),
    read_buffer: extern "system" fn(u32),
    read_pixels: extern "system" fn(i32, i32, i32, i32, u32, u32, *mut c_void),
    get_error: extern "system" fn() -> u32,
}

unsafe fn laden<T>(name: &str) -> T {
    let c = CString::new(name).unwrap();
    let p = unsafe { glfwGetProcAddress(c.as_ptr()) };
    assert!(!p.is_null(), "GL-Funktion {} fehlt", name);
    unsafe { std::mem::transmute_copy(&p) }
}

static KLICKS: AtomicI32 = AtomicI32::new(0);
static HAUPT_FALSCH: AtomicI32 = AtomicI32::new(0);
extern "C" fn klick(_f: GlfwFenster, knopf: c_int, aktion: c_int, _mod: c_int) {
    if knopf == 0 && aktion == 1 { KLICKS.fetch_add(1, Ordering::SeqCst); }
}

fn farbe(r: u8, g: u8, b: u8) -> ffi::Color { ffi::Color { r, g, b, a: 255 } }

fn main() {
    let (bw, bh) = (320, 240);
    unsafe {
        let titel = CString::new("Spike: Hauptfenster").unwrap();
        ffi::InitWindow(640, 360, titel.as_ptr());
        ffi::SetTargetFPS(60);
        let haupt = glfwGetCurrentContext();
        let ziel = ffi::LoadRenderTexture(bw, bh);

        // Zweites Fenster: dieselbe GL-Fassung wie raylib (3.3 core), Kontext
        // mit dem Hauptfenster GETEILT.
        glfwDefaultWindowHints();
        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
        glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
        if cfg!(target_os = "macos") { glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, 1); }
        let t2 = CString::new("Spike: zweites Fenster").unwrap();
        let zweit = glfwCreateWindow(bw, bh, t2.as_ptr(), std::ptr::null_mut(), haupt);
        println!("ERGEBNIS fenster_angelegt {}", !zweit.is_null());
        if zweit.is_null() { std::process::exit(2); }
        glfwSetWindowPos(zweit, 760, 120);
        glfwSetMouseButtonCallback(zweit, Some(klick));

        glfwMakeContextCurrent(zweit);
        glfwSwapInterval(0);   // sonst wartet jedes Fenster auf seinen eigenen Bildwechsel
        let gl = Gl {
            gen_framebuffers: laden("glGenFramebuffers"),
            bind_framebuffer: laden("glBindFramebuffer"),
            framebuffer_texture_2d: laden("glFramebufferTexture2D"),
            check_framebuffer_status: laden("glCheckFramebufferStatus"),
            blit_framebuffer: laden("glBlitFramebuffer"),
            read_buffer: laden("glReadBuffer"),
            read_pixels: laden("glReadPixels"),
            get_error: laden("glGetError"),
        };
        let mut fbo = 0u32;
        (gl.gen_framebuffers)(1, &mut fbo);
        (gl.bind_framebuffer)(GL_READ_FRAMEBUFFER, fbo);
        (gl.framebuffer_texture_2d)(GL_READ_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, ziel.texture.id, 0);
        let status = (gl.check_framebuffer_status)(GL_READ_FRAMEBUFFER);
        println!("ERGEBNIS fbo_vollstaendig {}", status == GL_FRAMEBUFFER_COMPLETE);
        glfwMakeContextCurrent(haupt);

        let fehler_art = std::env::var("SPIKE_FEHLER").unwrap_or_default();
        let mut bild = 0;
        let mut zeigen_us: Vec<u128> = Vec::new();
        let mut pixel_ok = 0;
        let mut pixel_falsch = 0;
        let mut zweit_offen = true;
        let mut fps_summe = 0;
        let start = Instant::now();
        while !ffi::WindowShouldClose() && bild < 240 {
            bild += 1;
            // 1) In das Render-Ziel zeichnen (Hauptkontext, raylib wie immer).
            //    Die Grundfarbe wechselt je Sekunde -- daran erkennt die
            //    Rueckleseprobe, dass das zweite Fenster das AKTUELLE Bild zeigt.
            let grund = if (bild / 60) % 2 == 0 { farbe(200, 60, 40) } else { farbe(40, 120, 200) };
            ffi::BeginTextureMode(ziel);
            ffi::ClearBackground(grund);
            let x = 20 + (bild * 3) % (bw - 40);
            ffi::DrawCircle(x, bh / 2, 18.0, farbe(250, 250, 250));
            let k = CString::new(format!("Klicks: {}", KLICKS.load(Ordering::SeqCst))).unwrap();
            ffi::DrawText(k.as_ptr(), 10, 10, 20, farbe(255, 255, 0));
            ffi::EndTextureMode();

            // 2) Das Hauptfenster selbst, ebenfalls mit raylib -- zeigt, dass
            //    raylib nach dem Wechsel der Kontexte weiter zeichnet.
            ffi::BeginDrawing();
            ffi::ClearBackground(farbe(20, 24, 30));
            ffi::DrawTexture(ziel.texture, 20, 20, farbe(255, 255, 255));
            let s = CString::new(format!("Hauptfenster, Bild {}  FPS {}", bild, ffi::GetFPS())).unwrap();
            ffi::DrawText(s.as_ptr(), 360, 20, 18, farbe(220, 220, 220));
            // Probe am HAUPTFENSTER: zeichnet raylib nach den Wechseln der
            // Kontexte noch richtig? Rechts im Bild des Render-Ziels (frei von
            // Kreis und Text, egal ob gespiegelt) muss die Grundfarbe stehen,
            // daneben der Hintergrund. Bild 230 liegt NACH dem Schliessen des
            // zweiten Fensters.
            if bild == 100 || bild == 230 {
                ffi::rlDrawRenderBatchActive();   // erst zeichnen, was gesammelt ist
                let img = ffi::LoadImageFromScreen();
                let s = img.width as f32 / 640.0;
                let p = |x: i32, y: i32| ffi::GetImageColor(img, (x as f32 * s) as i32, (y as f32 * s) as i32);
                let nah = |c: ffi::Color, z: ffi::Color| (c.r as i32 - z.r as i32).abs() < 8
                    && (c.g as i32 - z.g as i32).abs() < 8 && (c.b as i32 - z.b as i32).abs() < 8;
                let im_bild = nah(p(20 + bw - 5, 100), grund) && nah(p(20 + bw - 5, 179), grund);
                let daneben = nah(p(20 + bw + 15, 300), farbe(20, 24, 30));
                println!("ERGEBNIS hauptfenster_bild_{} {}", bild, if im_bild && daneben { "richtig" } else { "FALSCH" });
                let f = |c: ffi::Color| (c.r, c.g, c.b);
                println!("  bild {} groesse {}x{} oben {:?} unten {:?} daneben {:?} grund {:?}", bild, img.width, img.height,
                    f(p(20 + bw - 5, 100)), f(p(20 + bw - 5, 179)), f(p(20 + bw + 15, 300)), f(grund));
                if !(im_bild && daneben) { HAUPT_FALSCH.fetch_add(1, Ordering::SeqCst); }
                ffi::UnloadImage(img);
            }
            ffi::EndDrawing();   // tauscht die Puffer -> die Textur ist fertig
            if bild > 60 { fps_summe += ffi::GetFPS(); }

            // 3) Zeigen im zweiten Fenster: Kontext wechseln, blitten, tauschen.
            if zweit_offen {
                if glfwWindowShouldClose(zweit) != 0 {
                    glfwDestroyWindow(zweit);
                    zweit_offen = false;
                    println!("ERGEBNIS zweites_fenster_geschlossen_bei_bild {}", bild);
                } else {
                    let t0 = Instant::now();
                    glfwMakeContextCurrent(zweit);
                    let (mut fb, mut fh) = (0, 0);
                    glfwGetFramebufferSize(zweit, &mut fb, &mut fh);
                    (gl.bind_framebuffer)(GL_READ_FRAMEBUFFER, fbo);
                    (gl.bind_framebuffer)(GL_DRAW_FRAMEBUFFER, 0);
                    // raylib-Render-Ziele stehen auf dem Kopf (GL zaehlt von unten)
                    // -- beim Zeigen andersherum lesen.
                    // Gegenprobe: SPIKE_FEHLER=blit laesst das Zeigen aus.
                    if fehler_art != "blit" { (gl.blit_framebuffer)(0, 0, bw, bh, 0, fh, fb, 0, GL_COLOR_BUFFER_BIT, GL_NEAREST); }
                    // Probe: das Pixel oben links im zweiten Fenster (unter dem Text
                    // frei) muss die Grundfarbe DIESES Bildes haben.
                    if bild % 20 == 0 {
                        (gl.bind_framebuffer)(GL_READ_FRAMEBUFFER, 0);
                        (gl.read_buffer)(GL_BACK);
                        let mut px = [0u8; 4];
                        (gl.read_pixels)(fb - 5, fh / 2 + 40, 1, 1, GL_RGBA, GL_UNSIGNED_BYTE, px.as_mut_ptr() as *mut c_void);
                        let gleich = (px[0] as i32 - grund.r as i32).abs() < 8
                            && (px[1] as i32 - grund.g as i32).abs() < 8
                            && (px[2] as i32 - grund.b as i32).abs() < 8;
                        if gleich { pixel_ok += 1 } else {
                            pixel_falsch += 1;
                            println!("  bild {}: zweites Fenster zeigt {:?}, erwartet {:?}", bild, &px[..3], (grund.r, grund.g, grund.b));
                        }
                    }
                    let fehler = (gl.get_error)();
                    if fehler != 0 { println!("  GL-Fehler 0x{:X} im zweiten Kontext, bild {}", fehler, bild); }
                    glfwSwapBuffers(zweit);
                    // Gegenprobe: SPIKE_FEHLER=zurueck bleibt im zweiten Kontext.
                    if fehler_art != "zurueck" { glfwMakeContextCurrent(haupt); }
                    zeigen_us.push(t0.elapsed().as_micros());
                }
            }

            // 4) Ein Klick ins zweite Fenster, wie ihn Windows schickt.
            #[cfg(windows)]
            if zweit_offen && (bild == 100 || bild == 150) {
                // Selbst deklariert: das windows-Crate zoege user32 so herein,
                // dass `CloseWindow` doppelt da ist (raylib hat eins).
                #[link(name = "user32")]
                unsafe extern "system" { fn PostMessageW(h: *mut c_void, m: u32, w: usize, l: isize) -> i32; }
                const WM_LBUTTONDOWN: u32 = 0x0201;
                const WM_LBUTTONUP: u32 = 0x0202;
                let hwnd = glfwGetWin32Window(zweit);
                let lp = ((60isize) << 16) | 50;
                PostMessageW(hwnd, WM_LBUTTONDOWN, 1, lp);
                PostMessageW(hwnd, WM_LBUTTONUP, 0, lp);
            }
            // 5) Das Kreuz des zweiten Fensters, wie Windows es schickt: das
            //    Hauptfenster muss danach weiterlaufen und richtig zeichnen.
            #[cfg(windows)]
            if zweit_offen && bild == 200 {
                #[link(name = "user32")]
                unsafe extern "system" { fn PostMessageW(h: *mut c_void, m: u32, w: usize, l: isize) -> i32; }
                PostMessageW(glfwGetWin32Window(zweit), 0x0010 /* WM_CLOSE */, 0, 0);
            }
        }
        let dauer = start.elapsed().as_secs_f64();
        zeigen_us.sort();
        let median = zeigen_us.get(zeigen_us.len() / 2).copied().unwrap_or(0);
        let p95 = zeigen_us.get(zeigen_us.len() * 95 / 100).copied().unwrap_or(0);
        let fps_mittel = if bild > 60 { fps_summe / (bild - 60) } else { 0 };
        println!("ERGEBNIS bilder {} in {:.2} s, fps_mittel {}", bild, dauer, fps_mittel);
        println!("ERGEBNIS zeigen_mikrosekunden median {} p95 {}", median, p95);
        println!("ERGEBNIS pixel_ok {} pixel_falsch {}", pixel_ok, pixel_falsch);
        println!("ERGEBNIS klicks {}", KLICKS.load(Ordering::SeqCst));
        if zweit_offen { glfwDestroyWindow(zweit); }
        ffi::UnloadRenderTexture(ziel);
        // Kein ffi::CloseWindow(): der Name steht auch in user32 (LNK2005); der Prozess endet gleich.
        let gut = status == GL_FRAMEBUFFER_COMPLETE && pixel_ok > 0 && pixel_falsch == 0
            && (cfg!(not(windows)) || KLICKS.load(Ordering::SeqCst) == 2) && fps_mittel >= 55
            && HAUPT_FALSCH.load(Ordering::SeqCst) == 0 && (cfg!(not(windows)) || !zweit_offen);
        println!("ERGEBNIS gesamt {}", if gut { "gut" } else { "SCHLECHT" });
        std::process::exit(if gut { 0 } else { 1 });
    }
}
