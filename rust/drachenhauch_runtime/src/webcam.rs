//! Webcam (Modul `webcam`, WEBCAM_*): Bilder einer Kamera in ein IMAGE.
//!
//! Je System ein eigener Weg, alle in einem eigenen Faden, der jedes neue Bild
//! als RGBA in eine Ablage legt (`Ablage`); die VM holt es ab, wenn das
//! Programm fragt, und schreibt es in ein gewoehnliches IMAGE (wie beim Video):
//!
//! - **Linux**: Video4Linux2 (`/dev/videoN`), Puffer per mmap; YUYV, MJPEG,
//!   RGB24/BGR24 und BGRX werden gelesen. Geprueft in der CI gegen `vivid`,
//!   den Testtreiber des Kernels.
//! - **Windows**: Media Foundation (Source Reader mit eingeschalteter
//!   Umwandlung nach RGB32).
//! - **macOS**: AVFoundation ueber die Objective-C-Laufzeit
//!   (AVCaptureVideoDataOutput, BGRA).
//!
//! MJPEG entpackt zune-jpeg (dieselbe Fassung, die krilla mitbringt).
//!
//! **Pruefen ohne Kamera:** `DHRT_WEBCAM=test` ersetzt alle Geraete durch
//! eine "Testbild"-Kamera: Farbbalken und ein Quadrat, das je Bild 8 Punkte
//! weiterwandert, 30 Bilder je Sekunde.
#![allow(dead_code)]

use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Geteilt {
    rgba: Vec<u8>,
    breite: u32,
    hoehe: u32,
    nr: u64,
    fehler: Option<String>,
}

pub type Ablage = Arc<Mutex<Geteilt>>;

fn sperren(ab: &Ablage) -> std::sync::MutexGuard<'_, Geteilt> {
    ab.lock().unwrap_or_else(|e| e.into_inner())
}

/// Ein fertiges Bild ablegen (RGBA, Zeile fuer Zeile von oben).
pub fn ablegen(ab: &Ablage, breite: u32, hoehe: u32, rgba: Vec<u8>) {
    let mut g = sperren(ab);
    g.rgba = rgba;
    g.breite = breite;
    g.hoehe = hoehe;
    g.nr += 1;
}

fn fehler_melden(ab: &Ablage, text: String) { sperren(ab).fehler = Some(text); }

fn mass_setzen(ab: &Ablage, breite: u32, hoehe: u32) {
    let mut g = sperren(ab);
    g.breite = breite;
    g.hoehe = hoehe;
}

/// Eine offene Kamera. Abraeumen (WEBCAM_CLOSE, Programmende) haelt den Faden
/// an und wartet auf ihn.
pub struct Kamera {
    pub name: String,
    ablage: Ablage,
    stop: Option<mpsc::Sender<()>>,
    faden: Option<JoinHandle<()>>,
    gesehen: u64,
}

impl Kamera {
    /// Ist ein Bild gekommen, seit zuletzt gefragt wurde? Dann steht es in
    /// `ziel` (der alte Inhalt wird getauscht, nicht kopiert).
    pub fn neues(&mut self, ziel: &mut Vec<u8>) -> Option<(u32, u32)> {
        let mut g = sperren(&self.ablage);
        if g.nr == self.gesehen || g.rgba.is_empty() { return None; }
        self.gesehen = g.nr;
        std::mem::swap(ziel, &mut g.rgba);
        g.rgba.clear();
        Some((g.breite, g.hoehe))
    }
    pub fn mass(&self) -> (u32, u32) { let g = sperren(&self.ablage); (g.breite, g.hoehe) }
    pub fn bilder(&self) -> u64 { sperren(&self.ablage).nr }
    pub fn fehler(&self) -> Option<String> { sperren(&self.ablage).fehler.clone() }
}

impl Drop for Kamera {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(f) = self.faden.take() { let _ = f.join(); }
    }
}

type Bereit = mpsc::Sender<Result<String, String>>;

/// Den Faden einer Kamera starten und warten, bis sie offen ist (oder nicht).
fn faden_starten<F>(f: F) -> Result<Kamera, String>
where F: FnOnce(Ablage, mpsc::Receiver<()>, Bereit) + Send + 'static
{
    let ablage: Ablage = Arc::new(Mutex::new(Geteilt::default()));
    let (tx_stop, rx_stop) = mpsc::channel::<()>();
    let (tx_bereit, rx_bereit) = mpsc::channel::<Result<String, String>>();
    let ab = ablage.clone();
    let faden = std::thread::spawn(move || f(ab, rx_stop, tx_bereit));
    match rx_bereit.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(name)) => Ok(Kamera { name, ablage, stop: Some(tx_stop), faden: Some(faden), gesehen: 0 }),
        Ok(Err(e)) => { let _ = faden.join(); Err(e) }
        Err(_) => Err("WEBCAM_OPEN: die Kamera antwortet nicht (10 s)".into()),
    }
}

/// Soll angehalten werden? (Sender weg oder Nachricht da.)
fn angehalten(stop: &mpsc::Receiver<()>) -> bool {
    !matches!(stop.try_recv(), Err(mpsc::TryRecvError::Empty))
}

fn testbild_an() -> bool {
    std::env::var("DHRT_WEBCAM").map(|v| v.eq_ignore_ascii_case("test")).unwrap_or(false)
}

/// Die Namen aller Kameras.
pub fn geraete() -> Result<Vec<String>, String> {
    if testbild_an() { return Ok(vec!["Testbild".into()]); }
    os::geraete()
}

/// Eine Kamera oeffnen: "" = die erste. Ein Name passt genau oder als
/// Teiltext ohne Gross/klein. `breite`/`hoehe` 0 = was die Kamera vorschlaegt;
/// sonst ein Wunsch, dem die Kamera so nah wie moeglich kommt.
pub fn oeffnen(name: &str, breite: u32, hoehe: u32) -> Result<Kamera, String> {
    let alle = geraete()?;
    let gewaehlt = waehlen(&alle, name)?;
    if testbild_an() {
        let (b, h) = if breite > 0 && hoehe > 0 { (breite, hoehe) } else { (320, 240) };
        return faden_starten(move |ab, stop, bereit| testbild_laufen(ab, stop, bereit, b, h));
    }
    os::starten(gewaehlt, breite, hoehe)
}

/// Welche Nummer meint `name`? Genau, sonst Teiltext ohne Gross/klein.
pub fn waehlen(alle: &[String], name: &str) -> Result<usize, String> {
    if alle.is_empty() {
        return Err("WEBCAM_OPEN: dieser Rechner hat keine Kamera (keine gefunden)".into());
    }
    if name.is_empty() { return Ok(0); }
    let klein = name.to_lowercase();
    alle.iter().position(|n| n == name)
        .or_else(|| alle.iter().position(|n| n.to_lowercase().contains(&klein)))
        .ok_or_else(|| format!("WEBCAM_OPEN: keine Kamera '{}' -- es gibt: {}", name, alle.join(", ")))
}

// --- Testbild ---------------------------------------------------------------

/// Die acht Farbbalken des Testbilds, von links: weiss, gelb, tuerkis, gruen,
/// violett, rot, blau, schwarz.
pub const BALKEN: [[u8; 3]; 8] = [
    [255, 255, 255], [255, 255, 0], [0, 255, 255], [0, 255, 0],
    [255, 0, 255], [255, 0, 0], [0, 0, 255], [0, 0, 0],
];

/// Bild Nummer `nr` (ab 1) des Testbilds: Farbbalken, darin ein schwarzes
/// Quadrat (Kante h/4), das je Bild 8 Punkte nach rechts wandert.
pub fn testbild(breite: u32, hoehe: u32, nr: u64) -> Vec<u8> {
    let (b, h) = (breite as usize, hoehe as usize);
    let mut px = vec![0u8; b * h * 4];
    let kante = (h / 4).max(1);
    let weg = b.saturating_sub(kante).max(1);
    let qx = ((nr.saturating_sub(1) * 8) as usize) % weg;
    let qy = h / 2 - kante / 2;
    for y in 0..h {
        for x in 0..b {
            let f = BALKEN[x * 8 / b.max(1)];
            let im_quadrat = x >= qx && x < qx + kante && y >= qy && y < qy + kante;
            let p = (y * b + x) * 4;
            let farbe = if im_quadrat { [0, 0, 0] } else { f };
            px[p] = farbe[0]; px[p + 1] = farbe[1]; px[p + 2] = farbe[2]; px[p + 3] = 255;
        }
    }
    px
}

fn testbild_laufen(ab: Ablage, stop: mpsc::Receiver<()>, bereit: Bereit, b: u32, h: u32) {
    mass_setzen(&ab, b, h);
    let _ = bereit.send(Ok("Testbild".into()));
    let anfang = Instant::now();
    let mut nr: u64 = 0;
    while !angehalten(&stop) {
        let soll = (anfang.elapsed().as_secs_f64() * 30.0) as u64 + 1;
        if soll > nr {
            nr = soll;
            ablegen(&ab, b, h, testbild(b, h, nr));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

// --- Umwandeln (rein) ---------------------------------------------------------

fn klemmen(v: i32) -> u8 { v.clamp(0, 255) as u8 }

/// YUV (BT.601, begrenzter Bereich 16..235) -> RGB.
pub fn yuv_zu_rgb(y: u8, u: u8, v: u8) -> [u8; 3] {
    let c = (y as i32 - 16).max(0) * 298;
    let d = u as i32 - 128;
    let e = v as i32 - 128;
    [klemmen((c + 409 * e + 128) >> 8),
     klemmen((c - 100 * d - 208 * e + 128) >> 8),
     klemmen((c + 516 * d + 128) >> 8)]
}

/// YUYV 4:2:2 (zwei Punkte teilen sich U und V) -> RGBA.
pub fn yuyv_zu_rgba(src: &[u8], b: usize, h: usize, zeile: usize) -> Vec<u8> {
    let mut out = vec![255u8; b * h * 4];
    for y in 0..h {
        let r = &src[(y * zeile).min(src.len())..];
        for x in (0..b).step_by(2) {
            let i = x * 2;
            if i + 3 >= r.len() { break; }
            let (y0, u, y1, v) = (r[i], r[i + 1], r[i + 2], r[i + 3]);
            let p = (y * b + x) * 4;
            out[p..p + 3].copy_from_slice(&yuv_zu_rgb(y0, u, v));
            if x + 1 < b { out[p + 4..p + 7].copy_from_slice(&yuv_zu_rgb(y1, u, v)); }
        }
    }
    out
}

/// Drei Bytes je Punkt (RGB oder BGR) -> RGBA.
pub fn rgb24_zu_rgba(src: &[u8], b: usize, h: usize, zeile: usize, bgr: bool) -> Vec<u8> {
    let mut out = vec![255u8; b * h * 4];
    for y in 0..h {
        for x in 0..b {
            let i = y * zeile + x * 3;
            if i + 2 >= src.len() { break; }
            let p = (y * b + x) * 4;
            let (r, g, bl) = if bgr { (src[i + 2], src[i + 1], src[i]) } else { (src[i], src[i + 1], src[i + 2]) };
            out[p] = r; out[p + 1] = g; out[p + 2] = bl;
        }
    }
    out
}

/// Vier Bytes je Punkt in der Reihenfolge B, G, R, X -> RGBA; `unten_zuerst`
/// fuer Puffer, die mit der untersten Zeile beginnen (Windows mit negativem
/// Zeilenabstand).
pub fn bgrx_zu_rgba(src: &[u8], b: usize, h: usize, zeile: usize, unten_zuerst: bool) -> Vec<u8> {
    let mut out = vec![255u8; b * h * 4];
    for y in 0..h {
        let quelle = if unten_zuerst { h - 1 - y } else { y };
        for x in 0..b {
            let i = quelle * zeile + x * 4;
            if i + 2 >= src.len() { break; }
            let p = (y * b + x) * 4;
            out[p] = src[i + 2]; out[p + 1] = src[i + 1]; out[p + 2] = src[i];
        }
    }
    out
}

/// Hat das JPEG vor seinem Scan eine Huffman-Tafel (DHT)?
pub fn hat_huffman_tafel(src: &[u8]) -> bool {
    let mut i = 2;
    while i + 4 <= src.len() {
        if src[i] != 0xFF { return false; }
        let m = src[i + 1];
        if m == 0xFF { i += 1; continue; }
        if m == 0xC4 { return true; }
        if m == 0xDA || m == 0xD9 { return false; }
        let n = u16::from_be_bytes([src[i + 2], src[i + 3]]) as usize;
        i += 2 + n;
    }
    false
}

/// MJPEG-Kameras lassen die Huffman-Tafeln weg -- gemeint sind dann die
/// Standardtafeln des JPEG-Normtexts (Anhang K). zune-jpeg setzt sie nur bei
/// der Kennung "AVI1" ein, und die schicken nicht alle Kameras: fehlt die
/// Tafel, kommt die Kennung hinter den Anfang.
pub fn mjpeg_vorbereiten(src: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    if src.len() < 4 || src[0] != 0xFF || src[1] != 0xD8 || hat_huffman_tafel(src) {
        return std::borrow::Cow::Borrowed(src);
    }
    let mut out = Vec::with_capacity(src.len() + 9);
    out.extend_from_slice(&src[..2]);
    out.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x07, b'A', b'V', b'I', b'1', 0x00]);
    out.extend_from_slice(&src[2..]);
    std::borrow::Cow::Owned(out)
}

/// Ein MJPEG-Bild -> (Breite, Hoehe, RGBA).
pub fn mjpeg_zu_rgba(src: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    use zune_jpeg::zune_core::colorspace::ColorSpace;
    use zune_jpeg::zune_core::options::DecoderOptions;
    let src = mjpeg_vorbereiten(src);
    let opt = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut d = zune_jpeg::JpegDecoder::new_with_options(std::io::Cursor::new(&src[..]), opt);
    let px = d.decode().map_err(|e| format!("MJPEG-Bild nicht lesbar -- {}", e))?;
    let info = d.info().ok_or("MJPEG-Bild ohne Kopf")?;
    Ok((info.width as u32, info.height as u32, px))
}

// --- Linux: Video4Linux2 --------------------------------------------------

#[cfg(target_os = "linux")]
mod os {
    use super::*;
    use std::ffi::CString;

    const fn ioc(dir: u32, nr: u32, groesse: usize) -> u64 {
        ((dir as u64) << 30) | ((groesse as u64) << 16) | ((b'V' as u64) << 8) | nr as u64
    }
    const R: u32 = 2;
    const W: u32 = 1;

    #[repr(C)]
    struct Faehigkeit { driver: [u8; 16], card: [u8; 32], bus: [u8; 32], version: u32, caps: u32, device_caps: u32, reserviert: [u32; 3] }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct PixFormat { breite: u32, hoehe: u32, format: u32, feld: u32, zeile: u32, groesse: u32, farbraum: u32, privat: u32, flags: u32, ycbcr: u32, quant: u32, xfer: u32 }
    #[repr(C)]
    struct Format { typ: u32, _luecke: u32, pix: PixFormat, rest: [u8; 152] }
    #[repr(C)]
    struct Anforderung { zahl: u32, typ: u32, speicher: u32, caps: u32, flags: u8, reserviert: [u8; 3] }
    #[repr(C)]
    struct Puffer { index: u32, typ: u32, belegt: u32, flags: u32, feld: u32, zeit: [i64; 2], zeitcode: [u32; 4], folge: u32, speicher: u32, versatz: u64, laenge: u32, reserviert2: u32, anfrage: u32 }

    const _: () = assert!(std::mem::size_of::<Faehigkeit>() == 104);
    const _: () = assert!(std::mem::size_of::<PixFormat>() == 48);
    const _: () = assert!(std::mem::size_of::<Format>() == 208);
    const _: () = assert!(std::mem::size_of::<Anforderung>() == 20);
    const _: () = assert!(std::mem::size_of::<Puffer>() == 88);

    const QUERYCAP: u64 = ioc(R, 0, 104);
    const S_FMT: u64 = ioc(R | W, 5, 208);
    const REQBUFS: u64 = ioc(R | W, 8, 20);
    const QUERYBUF: u64 = ioc(R | W, 9, 88);
    const QBUF: u64 = ioc(R | W, 15, 88);
    const DQBUF: u64 = ioc(R | W, 17, 88);
    const STREAMON: u64 = ioc(W, 18, 4);
    const STREAMOFF: u64 = ioc(W, 19, 4);

    const CAP_VIDEO_CAPTURE: u32 = 0x1;
    const CAP_STREAMING: u32 = 0x0400_0000;
    const CAP_DEVICE_CAPS: u32 = 0x8000_0000;
    const TYP_CAPTURE: u32 = 1;
    const SPEICHER_MMAP: u32 = 1;
    const FELD_NONE: u32 = 1;

    const fn fourcc(s: &[u8; 4]) -> u32 { (s[0] as u32) | (s[1] as u32) << 8 | (s[2] as u32) << 16 | (s[3] as u32) << 24 }
    const YUYV: u32 = fourcc(b"YUYV");
    const MJPG: u32 = fourcc(b"MJPG");
    const RGB3: u32 = fourcc(b"RGB3");
    const BGR3: u32 = fourcc(b"BGR3");
    const XR24: u32 = fourcc(b"XR24");
    const AR24: u32 = fourcc(b"AR24");
    const BGR4: u32 = fourcc(b"BGR4");

    fn text(b: &[u8]) -> String {
        let ende = b.iter().position(|&c| c == 0).unwrap_or(b.len());
        String::from_utf8_lossy(&b[..ende]).trim().to_string()
    }

    unsafe fn ioctl<T>(fd: i32, nr: u64, wert: &mut T) -> std::io::Result<()> {
        loop {
            let r = libc::ioctl(fd, nr as _, wert as *mut T);
            if r >= 0 { return Ok(()); }
            let e = std::io::Error::last_os_error();
            if e.raw_os_error() != Some(libc::EINTR) { return Err(e); }
        }
    }

    /// (Pfad, Name) aller Geraete, die Bilder aufnehmen koennen.
    fn liste() -> Vec<(String, String)> {
        let mut out = Vec::new();
        for i in 0..64 {
            let pfad = format!("/dev/video{}", i);
            let Ok(c) = CString::new(pfad.clone()) else { continue };
            let fd = unsafe { libc::open(c.as_ptr(), libc::O_RDWR | libc::O_NONBLOCK) };
            if fd < 0 { continue; }
            let mut f: Faehigkeit = unsafe { std::mem::zeroed() };
            let ok = unsafe { ioctl(fd, QUERYCAP, &mut f) }.is_ok();
            unsafe { libc::close(fd) };
            if !ok { continue; }
            let caps = if f.caps & CAP_DEVICE_CAPS != 0 { f.device_caps } else { f.caps };
            if caps & CAP_VIDEO_CAPTURE == 0 || caps & CAP_STREAMING == 0 { continue; }
            out.push((pfad, text(&f.card)));
        }
        // Zwei gleich benannte (zwei Kameras derselben Art) bekommen ihren Pfad dazu.
        let namen: Vec<String> = out.iter().map(|(_, n)| n.clone()).collect();
        for (p, n) in out.iter_mut() {
            if namen.iter().filter(|m| *m == n).count() > 1 { *n = format!("{} ({})", n, p); }
        }
        out
    }

    pub fn geraete() -> Result<Vec<String>, String> {
        Ok(liste().into_iter().map(|(_, n)| n).collect())
    }

    pub fn starten(nr: usize, breite: u32, hoehe: u32) -> Result<Kamera, String> {
        let (pfad, name) = liste().into_iter().nth(nr).ok_or("WEBCAM_OPEN: die Kamera ist weg")?;
        faden_starten(move |ab, stop, bereit| {
            if let Err(e) = laufen(&pfad, &name, breite, hoehe, &ab, &stop, &bereit) {
                // Kam der Fehler vor der Meldung "bereit", sieht ihn WEBCAM_OPEN;
                // danach WEBCAM_ERROR$.
                if bereit.send(Err(e.clone())).is_err() { fehler_melden(&ab, e); }
            }
        })
    }

    struct Abbild { zeiger: *mut libc::c_void, laenge: usize }

    fn laufen(pfad: &str, name: &str, breite: u32, hoehe: u32, ab: &Ablage,
              stop: &mpsc::Receiver<()>, bereit: &Bereit) -> Result<(), String> {
        let c = CString::new(pfad).map_err(|_| "WEBCAM_OPEN: Pfad mit Nullbyte".to_string())?;
        let fd = unsafe { libc::open(c.as_ptr(), libc::O_RDWR | libc::O_NONBLOCK) };
        if fd < 0 {
            let e = std::io::Error::last_os_error();
            return Err(if e.raw_os_error() == Some(libc::EACCES) {
                format!("WEBCAM_OPEN: kein Zugriff auf {} -- der Benutzer muss in der Gruppe 'video' sein", pfad)
            } else if e.raw_os_error() == Some(libc::EBUSY) {
                format!("WEBCAM_OPEN: {} ist belegt -- ein anderes Programm benutzt die Kamera", pfad)
            } else { format!("WEBCAM_OPEN: {} -- {}", pfad, e) });
        }
        let erg = (|| -> Result<(), String> {
            let (b, h) = if breite > 0 && hoehe > 0 { (breite, hoehe) } else { (640, 480) };
            // Grosse Bilder lieber als MJPEG: YUYV schafft bei 1280x720 ueber
            // USB 2 oft nur wenige Bilder je Sekunde.
            let reihe: &[u32] = if b * h > 640 * 480 { &[MJPG, YUYV, RGB3, BGR3, XR24, AR24, BGR4] }
                                else { &[YUYV, MJPG, RGB3, BGR3, XR24, AR24, BGR4] };
            let mut gewaehlt: Option<PixFormat> = None;
            let mut belegt = false;
            for &pf in reihe {
                let mut f: Format = unsafe { std::mem::zeroed() };
                f.typ = TYP_CAPTURE;
                f.pix.breite = b;
                f.pix.hoehe = h;
                f.pix.format = pf;
                f.pix.feld = FELD_NONE;
                match unsafe { ioctl(fd, S_FMT, &mut f) } {
                    Ok(()) if f.pix.format == pf => { gewaehlt = Some(f.pix); break; }
                    // Liefert ein anderes Programm schon Bilder aus der Kamera,
                    // laesst sich das Format nicht mehr umstellen -- das ist
                    // "belegt", nicht "kein Format".
                    Err(e) if e.raw_os_error() == Some(libc::EBUSY) => { belegt = true; break; }
                    _ => {}
                }
            }
            if belegt && gewaehlt.is_none() {
                return Err(format!("WEBCAM_OPEN: {} ist belegt -- ein anderes Programm benutzt die Kamera", name));
            }
            let pix = gewaehlt.ok_or_else(|| format!("WEBCAM_OPEN: {} liefert kein Format, das dhrt liest (YUYV, MJPEG, RGB24, BGRX)", name))?;
            let mut anf = Anforderung { zahl: 4, typ: TYP_CAPTURE, speicher: SPEICHER_MMAP, caps: 0, flags: 0, reserviert: [0; 3] };
            unsafe { ioctl(fd, REQBUFS, &mut anf) }.map_err(|e| format!("WEBCAM_OPEN: keine Puffer -- {}", e))?;
            if anf.zahl == 0 { return Err("WEBCAM_OPEN: die Kamera gibt keine Puffer".into()); }
            let mut abbilder: Vec<Abbild> = Vec::new();
            let aufraeumen = |abbilder: &Vec<Abbild>| for a in abbilder { unsafe { libc::munmap(a.zeiger, a.laenge); } };
            for i in 0..anf.zahl {
                let mut p: Puffer = unsafe { std::mem::zeroed() };
                p.index = i; p.typ = TYP_CAPTURE; p.speicher = SPEICHER_MMAP;
                if let Err(e) = unsafe { ioctl(fd, QUERYBUF, &mut p) } { aufraeumen(&abbilder); return Err(format!("WEBCAM_OPEN: Puffer {} -- {}", i, e)); }
                let z = unsafe { libc::mmap(std::ptr::null_mut(), p.laenge as usize, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, p.versatz as libc::off_t) };
                if z == libc::MAP_FAILED { aufraeumen(&abbilder); return Err("WEBCAM_OPEN: mmap gescheitert".into()); }
                abbilder.push(Abbild { zeiger: z, laenge: p.laenge as usize });
                if let Err(e) = unsafe { ioctl(fd, QBUF, &mut p) } { aufraeumen(&abbilder); return Err(format!("WEBCAM_OPEN: Puffer {} -- {}", i, e)); }
            }
            let mut typ: i32 = TYP_CAPTURE as i32;
            if let Err(e) = unsafe { ioctl(fd, STREAMON, &mut typ) } { aufraeumen(&abbilder); return Err(format!("WEBCAM_OPEN: die Kamera startet nicht -- {}", e)); }
            mass_setzen(ab, pix.breite, pix.hoehe);
            let _ = bereit.send(Ok(name.to_string()));
            let (bw, bh, zeile) = (pix.breite as usize, pix.hoehe as usize, pix.zeile as usize);
            let mut fehlschlaege = 0;
            while !angehalten(stop) {
                let mut pf = libc::pollfd { fd, events: libc::POLLIN, revents: 0 };
                let r = unsafe { libc::poll(&mut pf, 1, 100) };
                if r <= 0 { continue; }
                let mut p: Puffer = unsafe { std::mem::zeroed() };
                p.typ = TYP_CAPTURE; p.speicher = SPEICHER_MMAP;
                if let Err(e) = unsafe { ioctl(fd, DQBUF, &mut p) } {
                    if e.raw_os_error() == Some(libc::EAGAIN) { continue; }
                    fehlschlaege += 1;
                    if fehlschlaege > 20 { fehler_melden(ab, format!("die Kamera liefert nicht mehr -- {}", e)); break; }
                    continue;
                }
                fehlschlaege = 0;
                if let Some(a) = abbilder.get(p.index as usize) {
                    let n = (p.belegt as usize).min(a.laenge);
                    let daten = unsafe { std::slice::from_raw_parts(a.zeiger as *const u8, n) };
                    let zeile = if zeile > 0 { zeile } else { n / bh.max(1) };
                    let bild = match pix.format {
                        YUYV => Some((bw as u32, bh as u32, yuyv_zu_rgba(daten, bw, bh, zeile))),
                        RGB3 => Some((bw as u32, bh as u32, rgb24_zu_rgba(daten, bw, bh, zeile, false))),
                        BGR3 => Some((bw as u32, bh as u32, rgb24_zu_rgba(daten, bw, bh, zeile, true))),
                        XR24 | AR24 | BGR4 => Some((bw as u32, bh as u32, bgrx_zu_rgba(daten, bw, bh, zeile, false))),
                        MJPG => mjpeg_zu_rgba(daten).ok(),
                        _ => None,
                    };
                    if let Some((w, h, px)) = bild { ablegen(ab, w, h, px); }
                }
                let _ = unsafe { ioctl(fd, QBUF, &mut p) };
            }
            let _ = unsafe { ioctl(fd, STREAMOFF, &mut typ) };
            aufraeumen(&abbilder);
            Ok(())
        })();
        unsafe { libc::close(fd) };
        erg
    }
}

// --- Windows: Media Foundation ----------------------------------------------

#[cfg(windows)]
mod os {
    use super::*;
    use windows::core::{Interface, PWSTR};
    use windows::Win32::Media::MediaFoundation::*;
    use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, COINIT_MULTITHREADED};

    fn anmelden() -> Result<(), String> {
        unsafe {
            // Unter GLFW ist der Hauptfaden schon STA -- das meldet hier einen
            // Fehler, der nichts bedeutet.
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            MFStartup(MF_VERSION, MFSTARTUP_FULL).map_err(|e| format!("WEBCAM: Media Foundation startet nicht -- {}", e))
        }
    }

    fn aktivierungen() -> Result<Vec<IMFActivate>, String> {
        anmelden()?;
        unsafe {
            let mut attr: Option<IMFAttributes> = None;
            MFCreateAttributes(&mut attr, 1).map_err(|e| e.to_string())?;
            let attr = attr.ok_or("WEBCAM: keine Attribute")?;
            attr.SetGUID(&MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID)
                .map_err(|e| e.to_string())?;
            let mut feld: *mut Option<IMFActivate> = std::ptr::null_mut();
            let mut n: u32 = 0;
            MFEnumDeviceSources(&attr, &mut feld, &mut n).map_err(|e| format!("WEBCAM: die Kameras sind nicht lesbar -- {}", e))?;
            let mut out = Vec::new();
            for i in 0..n as usize {
                if let Some(a) = (*feld.add(i)).take() { out.push(a); }
            }
            if !feld.is_null() { CoTaskMemFree(Some(feld as *const _)); }
            Ok(out)
        }
    }

    fn name(a: &IMFActivate) -> String {
        unsafe {
            let mut p = PWSTR::null();
            let mut laenge: u32 = 0;
            if a.GetAllocatedString(&MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME, &mut p, &mut laenge).is_err() || p.is_null() {
                return "Kamera".into();
            }
            let s = p.to_string().unwrap_or_else(|_| "Kamera".into());
            CoTaskMemFree(Some(p.0 as *const _));
            s
        }
    }

    pub fn geraete() -> Result<Vec<String>, String> {
        Ok(aktivierungen()?.iter().map(name).collect())
    }

    pub fn starten(nr: usize, breite: u32, hoehe: u32) -> Result<Kamera, String> {
        faden_starten(move |ab, stop, bereit| {
            if let Err(e) = laufen(nr, breite, hoehe, &ab, &stop, &bereit) {
                if bereit.send(Err(e.clone())).is_err() { fehler_melden(&ab, e); }
            }
        })
    }

    fn laufen(nr: usize, breite: u32, hoehe: u32, ab: &Ablage,
              stop: &mpsc::Receiver<()>, bereit: &Bereit) -> Result<(), String> {
        let alle = aktivierungen()?;
        let akt = alle.get(nr).ok_or("WEBCAM_OPEN: die Kamera ist weg")?;
        let anzeige = name(akt);
        let strom = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;
        unsafe {
            let quelle: IMFMediaSource = akt.ActivateObject()
                .map_err(|e| format!("WEBCAM_OPEN: {} laesst sich nicht oeffnen -- {} (Datenschutz -> Kamera erlaubt?)", anzeige, e))?;
            let erg = (|| -> Result<(), String> {
                let mut ra: Option<IMFAttributes> = None;
                MFCreateAttributes(&mut ra, 1).map_err(|e| e.to_string())?;
                let ra = ra.ok_or("WEBCAM: keine Attribute")?;
                ra.SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1).map_err(|e| e.to_string())?;
                let leser = MFCreateSourceReaderFromMediaSource(&quelle, &ra)
                    .map_err(|e| format!("WEBCAM_OPEN: kein Leser fuer {} -- {}", anzeige, e))?;
                let typ_setzen = |mit_mass: bool| -> windows::core::Result<()> {
                    let typ = MFCreateMediaType()?;
                    typ.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
                    typ.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
                    if mit_mass { typ.SetUINT64(&MF_MT_FRAME_SIZE, ((breite as u64) << 32) | hoehe as u64)?; }
                    leser.SetCurrentMediaType(strom, None, &typ)
                };
                let gesetzt = if breite > 0 && hoehe > 0 { typ_setzen(true).or_else(|_| typ_setzen(false)) } else { typ_setzen(false) };
                gesetzt.map_err(|e| format!("WEBCAM_OPEN: {} liefert kein RGB -- {}", anzeige, e))?;
                let aktuell = leser.GetCurrentMediaType(strom).map_err(|e| e.to_string())?;
                let mass = aktuell.GetUINT64(&MF_MT_FRAME_SIZE).map_err(|e| e.to_string())?;
                let (bw, bh) = ((mass >> 32) as usize, (mass & 0xFFFF_FFFF) as usize);
                let schritt = aktuell.GetUINT32(&MF_MT_DEFAULT_STRIDE).map(|s| s as i32).unwrap_or((bw * 4) as i32);
                mass_setzen(ab, bw as u32, bh as u32);
                let _ = bereit.send(Ok(anzeige.clone()));
                while !angehalten(stop) {
                    let mut flags: u32 = 0;
                    let mut probe: Option<IMFSample> = None;
                    if let Err(e) = leser.ReadSample(strom, 0, None, Some(&mut flags), None, Some(&mut probe)) {
                        fehler_melden(ab, format!("die Kamera liefert nicht mehr -- {}", e));
                        break;
                    }
                    if flags & (MF_SOURCE_READERF_ENDOFSTREAM.0 as u32) != 0 { break; }
                    let Some(probe) = probe else { continue };
                    let Ok(puffer) = probe.ConvertToContiguousBuffer() else { continue };
                    let mut p: *mut u8 = std::ptr::null_mut();
                    let mut laenge: u32 = 0;
                    if puffer.Lock(&mut p, None, Some(&mut laenge)).is_err() || p.is_null() { continue; }
                    let daten = std::slice::from_raw_parts(p as *const u8, laenge as usize);
                    let px = bgrx_zu_rgba(daten, bw, bh, schritt.unsigned_abs() as usize, schritt < 0);
                    let _ = puffer.Unlock();
                    ablegen(ab, bw as u32, bh as u32, px);
                }
                Ok(())
            })();
            let _ = quelle.Shutdown();
            let _ = akt.ShutdownObject();
            let _ = Interface::as_raw(&quelle);
            erg
        }
    }
}

// --- macOS: AVFoundation ------------------------------------------------------

#[cfg(target_os = "macos")]
mod os {
    use super::*;
    use std::ffi::{c_char, c_void, CStr, CString};

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
    #[link(name = "Foundation", kind = "framework")]
    extern "C" {}
    #[link(name = "AVFoundation", kind = "framework")]
    extern "C" {
        static AVMediaTypeVideo: Id;
        static AVCaptureSessionPreset640x480: Id;
        static AVCaptureSessionPreset1280x720: Id;
        static AVCaptureSessionPreset1920x1080: Id;
        static AVCaptureSessionPreset352x288: Id;
    }
    #[link(name = "CoreMedia", kind = "framework")]
    extern "C" {
        fn CMSampleBufferGetImageBuffer(probe: Id) -> Id;
    }
    #[link(name = "CoreVideo", kind = "framework")]
    extern "C" {
        static kCVPixelBufferPixelFormatTypeKey: Id;
        fn CVPixelBufferLockBaseAddress(pb: Id, flags: u64) -> i32;
        fn CVPixelBufferUnlockBaseAddress(pb: Id, flags: u64) -> i32;
        fn CVPixelBufferGetBaseAddress(pb: Id) -> *mut u8;
        fn CVPixelBufferGetBytesPerRow(pb: Id) -> usize;
        fn CVPixelBufferGetWidth(pb: Id) -> usize;
        fn CVPixelBufferGetHeight(pb: Id) -> usize;
    }
    extern "C" {
        fn dispatch_queue_create(label: *const c_char, attr: Id) -> Id;
    }

    fn sel(n: &CStr) -> Sel { unsafe { sel_registerName(n.as_ptr()) } }
    fn klasse(n: &CStr) -> Id { unsafe { objc_getClass(n.as_ptr()) } }

    macro_rules! senden {
        ($ziel:expr, $sel:expr $(, $a:expr => $t:ty)* ; -> $r:ty) => {{
            let f: extern "C" fn(Id, Sel $(, $t)*) -> $r = unsafe { std::mem::transmute(objc_msgSend as *const ()) };
            f($ziel, sel($sel) $(, $a)*)
        }};
    }

    fn rust_text(s: Id) -> String {
        if s.is_null() { return String::new(); }
        let p = senden!(s, c"UTF8String"; -> *const c_char);
        if p.is_null() { String::new() } else { unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned() }
    }

    fn im_pool<R>(f: impl FnOnce() -> R) -> R {
        let p = unsafe { objc_autoreleasePoolPush() };
        let r = f();
        unsafe { objc_autoreleasePoolPop(p) };
        r
    }

    fn kameras() -> Vec<Id> {
        let feld = senden!(klasse(c"AVCaptureDevice"), c"devicesWithMediaType:", unsafe { AVMediaTypeVideo } => Id; -> Id);
        if feld.is_null() { return Vec::new(); }
        let n = senden!(feld, c"count"; -> usize);
        (0..n).map(|i| senden!(feld, c"objectAtIndex:", i => usize; -> Id)).collect()
    }

    pub fn geraete() -> Result<Vec<String>, String> {
        Ok(im_pool(|| kameras().into_iter().map(|k| rust_text(senden!(k, c"localizedName"; -> Id))).collect()))
    }

    /// Wohin ein Delegat seine Bilder legt (Delegat-Zeiger -> Ablage).
    static ZIELE: Mutex<Vec<(usize, Ablage)>> = Mutex::new(Vec::new());

    /// - (void)captureOutput:(id)o didOutputSampleBuffer:(CMSampleBufferRef)sb fromConnection:(id)c
    extern "C" fn bei_bild(selbst: Id, _cmd: Sel, _ausgabe: Id, probe: Id, _verbindung: Id) {
        let ablage = {
            let z = ZIELE.lock().unwrap_or_else(|e| e.into_inner());
            z.iter().find(|(p, _)| *p == selbst as usize).map(|(_, a)| a.clone())
        };
        let Some(ab) = ablage else { return };
        unsafe {
            let pb = CMSampleBufferGetImageBuffer(probe);
            if pb.is_null() { return; }
            if CVPixelBufferLockBaseAddress(pb, 1) != 0 { return; }
            let basis = CVPixelBufferGetBaseAddress(pb);
            let zeile = CVPixelBufferGetBytesPerRow(pb);
            let (b, h) = (CVPixelBufferGetWidth(pb), CVPixelBufferGetHeight(pb));
            if !basis.is_null() && b > 0 && h > 0 {
                let daten = std::slice::from_raw_parts(basis as *const u8, zeile * h);
                let px = bgrx_zu_rgba(daten, b, h, zeile, false);
                ablegen(&ab, b as u32, h as u32, px);
            }
            CVPixelBufferUnlockBaseAddress(pb, 1);
        }
    }

    fn delegat_klasse() -> Id {
        let mut cls = klasse(c"DHKameraZiel");
        if cls.is_null() {
            unsafe {
                cls = objc_allocateClassPair(klasse(c"NSObject"), c"DHKameraZiel".as_ptr(), 0);
                class_addMethod(cls, sel(c"captureOutput:didOutputSampleBuffer:fromConnection:"),
                    bei_bild as *const c_void, c"v@:@@@".as_ptr());
                objc_registerClassPair(cls);
            }
        }
        cls
    }

    pub fn starten(nr: usize, breite: u32, hoehe: u32) -> Result<Kamera, String> {
        faden_starten(move |ab, stop, bereit| {
            let erg = im_pool(|| laufen(nr, breite, hoehe, &ab, &bereit));
            match erg {
                Err(e) => { if bereit.send(Err(e.clone())).is_err() { fehler_melden(&ab, e); } }
                Ok((sitzung, delegat)) => {
                    while !angehalten(&stop) { std::thread::sleep(Duration::from_millis(20)); }
                    im_pool(|| {
                        senden!(sitzung, c"stopRunning"; -> ());
                        senden!(sitzung, c"release"; -> ());
                    });
                    ZIELE.lock().unwrap_or_else(|e| e.into_inner()).retain(|(p, _)| *p != delegat as usize);
                }
            }
        })
    }

    fn laufen(nr: usize, breite: u32, hoehe: u32, ab: &Ablage, bereit: &Bereit) -> Result<(Id, Id), String> {
        let alle = kameras();
        let kamera = *alle.get(nr).ok_or("WEBCAM_OPEN: die Kamera ist weg")?;
        let anzeige = rust_text(senden!(kamera, c"localizedName"; -> Id));
        // 0 offen, 1 eingeschraenkt, 2 verweigert, 3 erlaubt. "Offen" fragt
        // macOS beim Start der Sitzung selbst nach.
        let stand = senden!(klasse(c"AVCaptureDevice"), c"authorizationStatusForMediaType:", unsafe { AVMediaTypeVideo } => Id; -> isize);
        if stand == 1 || stand == 2 {
            return Err("WEBCAM_OPEN: der Zugriff auf die Kamera ist verweigert -- Systemeinstellungen -> Datenschutz & Sicherheit -> Kamera".into());
        }
        let mut fehler: Id = std::ptr::null_mut();
        let eingang = senden!(klasse(c"AVCaptureDeviceInput"), c"deviceInputWithDevice:error:", kamera => Id, &mut fehler as *mut Id => *mut Id; -> Id);
        if eingang.is_null() {
            let text = if fehler.is_null() { String::new() } else { rust_text(senden!(fehler, c"localizedDescription"; -> Id)) };
            return Err(format!("WEBCAM_OPEN: {} laesst sich nicht oeffnen -- {}", anzeige, text));
        }
        let sitzung = senden!(senden!(klasse(c"AVCaptureSession"), c"alloc"; -> Id), c"init"; -> Id);
        let voreinstellung = unsafe {
            match (breite, hoehe) {
                (b, _) if b >= 1920 => AVCaptureSessionPreset1920x1080,
                (b, _) if b >= 1280 => AVCaptureSessionPreset1280x720,
                (b, _) if b > 0 && b <= 352 => AVCaptureSessionPreset352x288,
                _ => AVCaptureSessionPreset640x480,
            }
        };
        if senden!(sitzung, c"canSetSessionPreset:", voreinstellung => Id; -> bool) {
            senden!(sitzung, c"setSessionPreset:", voreinstellung => Id; -> ());
        }
        if !senden!(sitzung, c"canAddInput:", eingang => Id; -> bool) {
            senden!(sitzung, c"release"; -> ());
            return Err(format!("WEBCAM_OPEN: {} ist belegt", anzeige));
        }
        senden!(sitzung, c"addInput:", eingang => Id; -> ());
        let ausgabe = senden!(senden!(klasse(c"AVCaptureVideoDataOutput"), c"alloc"; -> Id), c"init"; -> Id);
        let bgra: u32 = u32::from_be_bytes(*b"BGRA");
        let zahl = senden!(klasse(c"NSNumber"), c"numberWithUnsignedInt:", bgra => u32; -> Id);
        let einst = senden!(klasse(c"NSDictionary"), c"dictionaryWithObject:forKey:", zahl => Id, unsafe { kCVPixelBufferPixelFormatTypeKey } => Id; -> Id);
        senden!(ausgabe, c"setVideoSettings:", einst => Id; -> ());
        senden!(ausgabe, c"setAlwaysDiscardsLateVideoFrames:", true => bool; -> ());
        let delegat = senden!(senden!(delegat_klasse(), c"alloc"; -> Id), c"init"; -> Id);
        ZIELE.lock().unwrap_or_else(|e| e.into_inner()).push((delegat as usize, ab.clone()));
        let name = CString::new("de.drachenhauch.webcam").unwrap();
        let schlange = unsafe { dispatch_queue_create(name.as_ptr(), std::ptr::null_mut()) };
        senden!(ausgabe, c"setSampleBufferDelegate:queue:", delegat => Id, schlange => Id; -> ());
        if !senden!(sitzung, c"canAddOutput:", ausgabe => Id; -> bool) {
            senden!(sitzung, c"release"; -> ());
            return Err(format!("WEBCAM_OPEN: {} liefert kein Bild", anzeige));
        }
        senden!(sitzung, c"addOutput:", ausgabe => Id; -> ());
        senden!(sitzung, c"startRunning"; -> ());
        let (b, h) = match voreinstellung {
            p if p == unsafe { AVCaptureSessionPreset1920x1080 } => (1920, 1080),
            p if p == unsafe { AVCaptureSessionPreset1280x720 } => (1280, 720),
            p if p == unsafe { AVCaptureSessionPreset352x288 } => (352, 288),
            _ => (640, 480),
        };
        mass_setzen(ab, b, h);
        let _ = bereit.send(Ok(anzeige));
        Ok((sitzung, delegat))
    }
}

#[cfg(not(any(target_os = "linux", windows, target_os = "macos")))]
mod os {
    use super::*;
    pub fn geraete() -> Result<Vec<String>, String> { Ok(Vec::new()) }
    pub fn starten(_nr: usize, _b: u32, _h: u32) -> Result<Kamera, String> {
        Err("WEBCAM_OPEN: auf diesem System gibt es noch keine Kamera".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuv_grau_und_farben() {
        assert_eq!(yuv_zu_rgb(16, 128, 128), [0, 0, 0]);
        assert_eq!(yuv_zu_rgb(235, 128, 128), [255, 255, 255]);
        let rot = yuv_zu_rgb(81, 90, 240);
        assert!(rot[0] > 240 && rot[1] < 10 && rot[2] < 10, "{:?}", rot);
        let blau = yuv_zu_rgb(41, 240, 110);
        assert!(blau[2] > 240 && blau[0] < 10 && blau[1] < 10, "{:?}", blau);
    }

    #[test]
    fn yuyv_teilt_farbe_auf_zwei_punkte() {
        // Zwei Punkte: schwarz und weiss bei grauer Farbe; Zeilenabstand mit Rand.
        let src = [16u8, 128, 235, 128, 9, 9, 9, 9];
        let px = yuyv_zu_rgba(&src, 2, 1, 8);
        assert_eq!(&px, &[0, 0, 0, 255, 255, 255, 255, 255]);
    }

    #[test]
    fn rgb_bgr_und_bgrx() {
        let src = [1u8, 2, 3, 4, 5, 6];
        assert_eq!(rgb24_zu_rgba(&src, 2, 1, 6, false), vec![1, 2, 3, 255, 4, 5, 6, 255]);
        assert_eq!(rgb24_zu_rgba(&src, 2, 1, 6, true), vec![3, 2, 1, 255, 6, 5, 4, 255]);
        // Zwei Zeilen zu einem Punkt, unten zuerst: die erste Zeile im Speicher ist die untere.
        let bgrx = [10u8, 20, 30, 0, 40, 50, 60, 0];
        assert_eq!(bgrx_zu_rgba(&bgrx, 1, 2, 4, false), vec![30, 20, 10, 255, 60, 50, 40, 255]);
        assert_eq!(bgrx_zu_rgba(&bgrx, 1, 2, 4, true), vec![60, 50, 40, 255, 30, 20, 10, 255]);
    }

    #[test]
    fn testbild_balken_und_wanderndes_quadrat() {
        let a = testbild(320, 240, 1);
        let b = testbild(320, 240, 2);
        let px = |v: &Vec<u8>, x: usize, y: usize| { let p = (y * 320 + x) * 4; [v[p], v[p + 1], v[p + 2]] };
        assert_eq!(px(&a, 10, 10), [255, 255, 255]);
        assert_eq!(px(&a, 320 - 10, 10), [0, 0, 0]);
        assert_eq!(px(&a, 3 * 40 + 20, 10), [0, 255, 0]);
        // Bild 1: Quadrat ab x 0, Bild 2 ab x 8.
        assert_eq!(px(&a, 2, 120), [0, 0, 0]);
        assert_eq!(px(&b, 2, 120), [255, 255, 255]);
        assert_eq!(px(&b, 10, 120), [0, 0, 0]);
    }

    #[test]
    fn waehlen_genau_teil_und_fehler() {
        let alle = vec!["Integrated Camera".to_string(), "USB Kamera".to_string()];
        assert_eq!(waehlen(&alle, ""), Ok(0));
        assert_eq!(waehlen(&alle, "USB Kamera"), Ok(1));
        assert_eq!(waehlen(&alle, "usb"), Ok(1));
        assert!(waehlen(&alle, "Studio").unwrap_err().contains("es gibt: Integrated Camera, USB Kamera"));
        assert!(waehlen(&[], "").unwrap_err().contains("keine Kamera"));
    }

    #[test]
    fn mjpeg_ohne_huffman_tafeln() {
        // Ein 8x8-JPEG in Grau, wie es eine Webcam schickt: ohne DHT-Segment.
        let jpeg = include_bytes!("../testdaten/webcam/grau8x8_ohne_dht.jpg");
        let (b, h, px) = mjpeg_zu_rgba(jpeg).expect("lesbar");
        assert_eq!((b, h), (8, 8));
        assert_eq!(px.len(), 8 * 8 * 4);
        assert!(px.chunks(4).all(|p| (p[0] as i32 - 128).abs() < 6 && p[0] == p[1] && p[1] == p[2]), "{:?}", &px[..8]);
        // Gegenprobe: dasselbe Bild MIT Tafel bleibt unangetastet und gibt dasselbe.
        let mit = include_bytes!("../testdaten/webcam/grau8x8.jpg");
        assert!(hat_huffman_tafel(mit) && !hat_huffman_tafel(jpeg));
        assert!(matches!(mjpeg_vorbereiten(mit), std::borrow::Cow::Borrowed(_)));
        assert_eq!(mjpeg_zu_rgba(mit).expect("lesbar").2, px);
    }
}
