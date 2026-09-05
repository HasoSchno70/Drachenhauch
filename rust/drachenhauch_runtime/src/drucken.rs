//! Drucken und Oeffnen -- Wege A und C aus `docs/entwurf-drucken.md`.
//!
//! **Weg A:** `OPENDOC(pfad$)` oeffnet eine lokale Datei mit ihrem
//! Standardprogramm -- das Gegenstueck zu `OPENURL`, das absichtlich nur
//! http/https darf. Begrenzt auf Dokument-Endungen, damit aus dem Befehl kein
//! Programmstarter wird (`OPENDOC("boese.exe")` ist ein Fehler, kein Start).
//!
//! **Weg C:** Das pdf-Modul zeichnet seine Seiten selbst (Text, Linien,
//! Rechtecke in Millimetern) und zeichnet sie seit dem Entwurf AUF
//! (`pdf::Op`). Dieselbe Seite geht damit auf drei Ziele: die PDF-Datei
//! (`PDF_SAVE`, wie immer), den Drucker (`PDF_PRINT`) und eine Vorschau als
//! Bild (`PDF_PREVIEW`, in graphics.rs). Es gibt kein raylib fuers Papier --
//! der Renderer ist das Betriebssystem:
//!
//! - **Windows:** GDI ueber das `windows`-Crate (das ohnehin im Baum liegt):
//!   `CreateDC` auf den Drucker, `StartDoc`/`StartPage`, Text mit den
//!   GDI-Geschwistern der Standardschriften (Helvetica -> Arial, Times ->
//!   Times New Roman, Courier -> Courier New), Linien und Rechtecke; der
//!   Treiber rastert. Millimeter werden ueber `GetDeviceCaps` in
//!   Geraeteeinheiten gerechnet, der nicht druckbare Rand (`PHYSICALOFFSET`)
//!   abgezogen -- unsere Masse gelten ab Papierkante, GDIs ab dem druckbaren
//!   Bereich. Eine `zieldatei` geht als `DOCINFO.lpszOutput` mit: "Microsoft
//!   Print to PDF" fragt dann NICHT nach, und genau so schickt der Test eine
//!   Rechnung durch einen echten Treiber und liest sie mit PyMuPDF zurueck.
//! - **macOS/Linux:** die PDF geht an CUPS (`lp`), das PDF versteht. Eine
//!   `zieldatei` ist dort die PDF selbst -- so laeuft derselbe Aufruf ueberall.
//!
//! Was hier NICHT ist: ein Renderer fuer fremde PDFs und ein nachgebauter
//! Druckdialog. Beides waere der Anfang von Weg D.

use crate::pdf::{Dokument, Op};

/// Endungen, die `OPENDOC` oeffnet -- Dokumente und Bilder, nichts, was laeuft.
const OEFFENBAR: &[&str] = &[
    "pdf", "png", "jpg", "jpeg", "gif", "bmp", "webp", "svg",
    "txt", "md", "csv", "json", "xml", "ini", "log", "html", "htm",
    "xlsx", "docx", "odt", "ods", "rtf", "wav", "mp3", "ogg", "mp4", "zip", "dh",
];

/// OPENDOC: eine lokale Datei mit dem Standardprogramm oeffnen.
pub fn oeffnen(pfad: &str) -> Result<(), String> {
    let p = std::path::Path::new(pfad);
    if !p.is_file() {
        return Err(format!("OPENDOC: '{}' gibt es nicht (oder ist keine Datei)", pfad));
    }
    let endung = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !OEFFENBAR.contains(&endung.as_str()) {
        return Err(format!(
            "OPENDOC: '.{}' wird nicht geoeffnet -- erlaubt sind Dokumente und Bilder ({}). \
             Fuer Programme gibt es SHELL, mit Absicht getrennt.", endung, OEFFENBAR.join(", ")));
    }
    let voll = std::fs::canonicalize(p).map_err(|e| format!("OPENDOC: {}: {}", pfad, e))?;
    plattform::oeffnen(&voll)
}

/// PRINTERS(): die Namen der Drucker, wie das System sie kennt.
pub fn drucker_liste() -> Result<Vec<String>, String> { plattform::drucker_liste() }

/// PRINTER_DEFAULT$(): der Standarddrucker, "" wenn es keinen gibt.
pub fn standard_drucker() -> Result<String, String> { plattform::standard_drucker() }

/// PDF_PRINT: das Dokument drucken. `drucker` leer = Standarddrucker;
/// `zieldatei` fuer Drucker, die in eine Datei schreiben.
pub fn drucken(doc: &Dokument, drucker: &str, kopien: i64, zieldatei: &str) -> Result<(), String> {
    if !(1..=99).contains(&kopien) {
        return Err(format!("PDF_PRINT: Kopien 1..99, nicht {}", kopien));
    }
    let drucker = if drucker.trim().is_empty() { standard_drucker()? } else { drucker.to_string() };
    if drucker.is_empty() {
        return Err("PDF_PRINT: kein Drucker -- es gibt keinen Standarddrucker, und keiner wurde genannt (PRINTERS() zeigt, was da ist)".into());
    }
    plattform::drucken(doc, &drucker, kopien as u32, zieldatei)
}

/// GDI-Schrift zu einer Standardschrift des pdf-Moduls: (Name, fett, kursiv).
pub fn gdi_schrift(programmname: &str) -> (&'static str, bool, bool) {
    let n = programmname.to_ascii_lowercase();
    let face = if n.starts_with("times") { "Times New Roman" }
               else if n.starts_with("courier") { "Courier New" }
               else if n.starts_with("symbol") { "Symbol" }
               else if n.starts_with("zapf") { "Wingdings" }
               else { "Arial" };
    (face, n.contains("fett"), n.contains("kursiv"))
}

// ================================================================ Windows: GDI + winspool
#[cfg(windows)]
mod plattform {
    use super::*;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{COLORREF, RECT};
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::Graphics::Printing::*;
    use windows::Win32::Storage::Xps::{EndDoc, EndPage, StartDocW, StartPage, DOCINFOW};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

    pub fn oeffnen(pfad: &std::path::Path) -> Result<(), String> {
        let p = wide(&pfad.to_string_lossy());
        let verb = wide("open");
        let h = unsafe { ShellExecuteW(None, PCWSTR(verb.as_ptr()), PCWSTR(p.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
        // Rueckgaben <= 32 sind Fehlercodes -- so steht es in der Win32-Doku.
        if (h.0 as isize) <= 32 {
            return Err(format!("OPENDOC: Windows konnte '{}' nicht oeffnen (Code {})", pfad.display(), h.0 as isize));
        }
        Ok(())
    }

    pub fn standard_drucker() -> Result<String, String> {
        let mut len: u32 = 0;
        unsafe { let _ = GetDefaultPrinterW(None, &mut len); }
        if len == 0 { return Ok(String::new()); }
        let mut puffer = vec![0u16; len as usize];
        let ok = unsafe { GetDefaultPrinterW(Some(windows::core::PWSTR(puffer.as_mut_ptr())), &mut len) };
        if !ok.as_bool() { return Ok(String::new()); }
        Ok(String::from_utf16_lossy(&puffer[..(len as usize).saturating_sub(1)]))
    }

    pub fn drucker_liste() -> Result<Vec<String>, String> {
        let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
        let (mut noetig, mut anzahl) = (0u32, 0u32);
        unsafe { let _ = EnumPrintersW(flags, PCWSTR::null(), 4, None, &mut noetig, &mut anzahl); }
        if noetig == 0 { return Ok(Vec::new()); }
        let mut puffer = vec![0u8; noetig as usize];
        let ok = unsafe { EnumPrintersW(flags, PCWSTR::null(), 4, Some(&mut puffer), &mut noetig, &mut anzahl) };
        if ok.is_err() { return Err("PRINTERS: Windows liefert die Druckerliste nicht".into()); }
        let infos = puffer.as_ptr() as *const PRINTER_INFO_4W;
        let mut namen = Vec::new();
        for i in 0..anzahl as usize {
            let info = unsafe { &*infos.add(i) };
            if info.pPrinterName.is_null() { continue; }
            namen.push(unsafe { info.pPrinterName.to_string() }.unwrap_or_default());
        }
        Ok(namen)
    }

    fn farbe(f: (f64, f64, f64)) -> COLORREF {
        let k = |x: f64| (x.clamp(0.0, 1.0) * 255.0).round() as u32;
        COLORREF(k(f.0) | (k(f.1) << 8) | (k(f.2) << 16))
    }

    pub fn drucken(doc: &Dokument, drucker: &str, kopien: u32, zieldatei: &str) -> Result<(), String> {
        let name = wide(drucker);
        let hdc = unsafe { CreateDCW(PCWSTR::null(), PCWSTR(name.as_ptr()), PCWSTR::null(), None) };
        if hdc.is_invalid() {
            return Err(format!("PDF_PRINT: Drucker '{}' laesst sich nicht oeffnen (PRINTERS() zeigt die Namen)", drucker));
        }
        let ergebnis = unsafe { seiten_drucken(hdc, doc, kopien, zieldatei) };
        unsafe { let _ = DeleteDC(hdc); }
        ergebnis
    }

    unsafe fn seiten_drucken(hdc: HDC, doc: &Dokument, kopien: u32, zieldatei: &str) -> Result<(), String> {
        let dpi_x = GetDeviceCaps(Some(hdc), LOGPIXELSX) as f64;
        let dpi_y = GetDeviceCaps(Some(hdc), LOGPIXELSY) as f64;
        let off_x = GetDeviceCaps(Some(hdc), PHYSICALOFFSETX) as f64;
        let off_y = GetDeviceCaps(Some(hdc), PHYSICALOFFSETY) as f64;
        let px = |mm: f64| (mm / 25.4 * dpi_x - off_x).round() as i32;
        let py = |mm: f64| (mm / 25.4 * dpi_y - off_y).round() as i32;
        let titel = wide(if doc.titel.is_empty() { "Drachenhauch" } else { &doc.titel });
        let ziel = wide(zieldatei);
        let info = DOCINFOW {
            cbSize: std::mem::size_of::<DOCINFOW>() as i32,
            lpszDocName: PCWSTR(titel.as_ptr()),
            lpszOutput: if zieldatei.is_empty() { PCWSTR::null() } else { PCWSTR(ziel.as_ptr()) },
            lpszDatatype: PCWSTR::null(),
            fwType: 0,
        };
        if StartDocW(hdc, &info) <= 0 {
            return Err("PDF_PRINT: der Drucker nimmt den Auftrag nicht an (StartDoc) -- ist er eingeschaltet, und wenn er in eine Datei schreibt: ist der Pfad beschreibbar?".into());
        }
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextAlign(hdc, TEXT_ALIGN_OPTIONS(TA_TOP.0 | TA_LEFT.0));
        let leer: HGDIOBJ = GetStockObject(NULL_BRUSH);
        for _ in 0..kopien {
            for seite in &doc.seiten {
                if StartPage(hdc) <= 0 { let _ = EndDoc(hdc); return Err("PDF_PRINT: StartPage schlug fehl".into()); }
                for op in &seite.ops {
                    match op {
                        Op::Text { x, y, text, schrift, groesse_pt, farbe: f } => {
                            let (face, fett, kursiv) = gdi_schrift(crate::pdf::schrift_programmname(*schrift));
                            let hoehe = -((groesse_pt / 72.0 * dpi_y).round() as i32);
                            let facew = wide(face);
                            let font = CreateFontW(hoehe, 0, 0, 0, if fett { FW_BOLD.0 as i32 } else { FW_NORMAL.0 as i32 },
                                                   kursiv as u32, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
                                                   CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_PITCH.0 as u32,
                                                   PCWSTR(facew.as_ptr()));
                            let alt = SelectObject(hdc, font.into());
                            let _ = SetTextColor(hdc, farbe(*f));
                            let t: Vec<u16> = text.encode_utf16().collect();
                            let _ = TextOutW(hdc, px(*x), py(*y), &t);
                            SelectObject(hdc, alt);
                            let _ = DeleteObject(font.into());
                        }
                        Op::Linie { x1, y1, x2, y2, breite_mm, farbe: f } => {
                            let dick = ((breite_mm / 25.4 * dpi_x).round() as i32).max(1);
                            let stift = CreatePen(PS_SOLID, dick, farbe(*f));
                            let alt = SelectObject(hdc, stift.into());
                            let _ = MoveToEx(hdc, px(*x1), py(*y1), None);
                            let _ = LineTo(hdc, px(*x2), py(*y2));
                            SelectObject(hdc, alt);
                            let _ = DeleteObject(stift.into());
                        }
                        Op::Rechteck { x, y, b, h, fuellen, breite_mm, farbe: f } => {
                            let r = RECT { left: px(*x), top: py(*y), right: px(*x + *b), bottom: py(*y + *h) };
                            if *fuellen {
                                let pinsel = CreateSolidBrush(farbe(*f));
                                let _ = FillRect(hdc, &r, pinsel);
                                let _ = DeleteObject(pinsel.into());
                            } else {
                                let dick = ((breite_mm / 25.4 * dpi_x).round() as i32).max(1);
                                let stift = CreatePen(PS_SOLID, dick, farbe(*f));
                                let alt_s = SelectObject(hdc, stift.into());
                                let alt_b = SelectObject(hdc, leer);
                                let _ = Rectangle(hdc, r.left, r.top, r.right, r.bottom);
                                SelectObject(hdc, alt_b);
                                SelectObject(hdc, alt_s);
                                let _ = DeleteObject(stift.into());
                            }
                        }
                    }
                }
                if EndPage(hdc) <= 0 { let _ = EndDoc(hdc); return Err("PDF_PRINT: EndPage schlug fehl".into()); }
            }
        }
        if EndDoc(hdc) <= 0 { return Err("PDF_PRINT: EndDoc schlug fehl".into()); }
        Ok(())
    }
}

// ================================================================ macOS / Linux: CUPS
#[cfg(not(windows))]
mod plattform {
    use super::*;

    pub fn oeffnen(pfad: &std::path::Path) -> Result<(), String> {
        let prog = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
        std::process::Command::new(prog).arg(pfad).spawn()
            .map(|_| ())
            .map_err(|e| format!("OPENDOC: '{}' laesst sich nicht starten: {}", prog, e))
    }

    fn lpstat(args: &[&str]) -> Option<String> {
        let out = std::process::Command::new("lpstat").args(args).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    pub fn drucker_liste() -> Result<Vec<String>, String> {
        // `lpstat -a`: je Zeile "name accepting requests since ...".
        Ok(lpstat(&["-a"]).unwrap_or_default().lines()
            .filter_map(|z| z.split_whitespace().next().map(str::to_string)).collect())
    }

    pub fn standard_drucker() -> Result<String, String> {
        // `lpstat -d`: "system default destination: name" -- oder ein Satz ohne Doppelpunkt-Namen.
        let s = lpstat(&["-d"]).unwrap_or_default();
        Ok(s.split(':').nth(1).map(|n| n.trim().to_string()).unwrap_or_default())
    }

    pub fn drucken(doc: &Dokument, drucker: &str, kopien: u32, zieldatei: &str) -> Result<(), String> {
        let bytes = doc.bauen();
        if !zieldatei.is_empty() {
            // Ohne GDI ist die Zieldatei die PDF selbst -- so laeuft derselbe
            // Aufruf auf jedem System.
            return std::fs::write(zieldatei, &bytes).map_err(|e| format!("PDF_PRINT: {}: {}", zieldatei, e));
        }
        let pfad = std::env::temp_dir().join(format!("drachenhauch_druck_{}_{}.pdf", std::process::id(), doc.seiten.len()));
        std::fs::write(&pfad, &bytes).map_err(|e| format!("PDF_PRINT: {}: {}", pfad.display(), e))?;
        let out = std::process::Command::new("lp").arg("-d").arg(drucker).arg("-n").arg(kopien.to_string()).arg(&pfad).output()
            .map_err(|e| format!("PDF_PRINT: 'lp' (CUPS) laesst sich nicht starten: {} -- ohne CUPS gibt es hier keinen Drucker", e))?;
        if !out.status.success() {
            return Err(format!("PDF_PRINT: lp meldet: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gdi_schriften_folgen_dem_namen() {
        assert_eq!(gdi_schrift("helvetica"), ("Arial", false, false));
        assert_eq!(gdi_schrift("helvetica-fett-kursiv"), ("Arial", true, true));
        assert_eq!(gdi_schrift("times-kursiv"), ("Times New Roman", false, true));
        assert_eq!(gdi_schrift("courier-fett"), ("Courier New", true, false));
    }

    #[test]
    fn opendoc_oeffnet_nur_dokumente() {
        let e = oeffnen("gibtsnicht.pdf").unwrap_err();
        assert!(e.contains("gibt es nicht"));
        let p = std::env::temp_dir().join("drachenhauch_opendoc_test.exe");
        std::fs::write(&p, b"MZ").unwrap();
        let e = oeffnen(p.to_str().unwrap()).unwrap_err();
        assert!(e.contains("nicht geoeffnet"), "{}", e);
        let _ = std::fs::remove_file(p);
    }
}
